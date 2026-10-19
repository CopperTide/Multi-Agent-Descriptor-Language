//! The document as a graph of agents and steps: groups become their members, `both` becomes two
//! directions.
use crate::model::{ConnectionKind, Direction, Document, Stage, Workflow};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub kind: ConnectionKind,
    /// The connection's index in `doc.connections`.
    pub connection: usize,
}

/// Agents in this group or any group inside it.
pub fn members(doc: &Document, group: &str) -> Vec<String> {
    let inside = |gid: &str| crate::resolve::ancestors(doc, gid).iter().any(|&i| doc.groups[i].id == group);
    doc.agents.iter().filter(|a| a.groups.iter().any(|g| inside(g))).map(|a| a.id.clone()).collect()
}

/// An id as the agents and steps it stands for.
pub fn nodes_of(doc: &Document, id: &str) -> Vec<String> {
    if doc.groups.iter().any(|g| g.id == id) { members(doc, id) } else { vec![id.to_string()] }
}

pub fn edges(doc: &Document) -> Vec<Edge> {
    let mut out = Vec::new();
    for (i, c) in doc.connections.iter().enumerate() {
        let mut pairs = vec![(c.from.as_str(), c.to.as_str())];
        if c.direction == Some(Direction::Both) {
            pairs.push((c.to.as_str(), c.from.as_str()));
        }
        for (from, to) in pairs {
            for f in nodes_of(doc, from) {
                for t in nodes_of(doc, to).into_iter().filter(|t| *t != f) {
                    out.push(Edge { from: f.clone(), to: t, kind: c.kind, connection: i });
                }
            }
        }
    }
    out
}

/// Kinds along which work moves (and so can loop).
pub fn carries_work(kind: ConnectionKind) -> bool {
    matches!(kind, ConnectionKind::Handoff | ConnectionKind::Delegate | ConnectionKind::Review)
}

/// Agents the work starts with: those an input names, and those nothing hands or delegates work
/// to. People (`runtime: human`) start only when an input names them.
pub fn starting_agents(doc: &Document) -> BTreeSet<String> {
    let edges = edges(doc);
    let mut fed: BTreeSet<&str> = edges
        .iter()
        .filter(|e| matches!(e.kind, ConnectionKind::Handoff | ConnectionKind::Delegate))
        .map(|e| e.to.as_str())
        .collect();
    for w in &doc.workflows {
        fed.extend(workflow_agents(w).into_iter().filter(|a| *a != w.dispatcher));
    }
    let mut out: BTreeSet<String> =
        doc.inputs.iter().flat_map(|i| i.to.iter().flat_map(|t| nodes_of(doc, t))).collect();
    let human = |id: &str| doc.agents.iter().any(|a| a.id == id && a.runtime.as_deref() == Some("human"));
    for a in &doc.agents {
        if !fed.contains(a.id.as_str()) && !human(&a.id) {
            out.insert(a.id.clone());
        }
    }
    out
}

/// Everything work can reach from the starting agents.
pub fn reachable(doc: &Document, starts: &BTreeSet<String>) -> BTreeSet<String> {
    let mut next: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let edges = edges(doc);
    for e in edges.iter().filter(|e| carries_work(e.kind) || e.kind == ConnectionKind::Report) {
        next.entry(e.from.as_str()).or_default().push(e.to.as_str());
    }
    for w in &doc.workflows {
        let mut team = workflow_agents(w);
        team.push(&w.collector);
        next.entry(w.dispatcher.as_str()).or_default().extend(team);
    }
    let mut seen: BTreeSet<String> = starts.clone();
    let mut queue: Vec<String> = starts.iter().cloned().collect();
    while let Some(n) = queue.pop() {
        for t in next.get(n.as_str()).into_iter().flatten() {
            if seen.insert(t.to_string()) {
                queue.push(t.to_string());
            }
        }
    }
    seen
}

/// The agents that do a workflow's stages: each stage's agent, workers, reviewers and escalation.
pub fn workflow_agents(w: &Workflow) -> Vec<&str> {
    w.stages.iter().flat_map(stage_agents).collect()
}

fn stage_agents(s: &Stage) -> Vec<&str> {
    let lists = s.workers.iter().chain(&s.reviewers).map(String::as_str);
    s.agent.as_deref().into_iter().chain(lists).chain(s.escalate.as_ref().map(|e| e.to.as_str())).collect()
}
