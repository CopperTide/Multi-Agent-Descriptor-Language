//! Workflows (MEP 0007): references to agents and inputs, stage ids and routes within each
//! workflow, and the fields each kind of stage uses.
use super::index::Kind;
use super::refs::{Refs, well_formed};
use crate::diagnostic::{Diagnostic, Report};
use crate::model::{Document, Stage, StageKind, Workflow};
use std::collections::BTreeSet;

impl Refs<'_> {
    pub(super) fn workflows(&mut self, doc: &Document) {
        for (w, flow) in doc.workflows.iter().enumerate() {
            let at = format!("/workflows/{w}");
            if let Some(input) = &flow.input {
                self.expect(input, format!("{at}/input"), &[Kind::Input], false);
            }
            match (&flow.dispatcher, &flow.dispatch) {
                (Some(d), None) => self.expect(d, format!("{at}/dispatcher"), &[Kind::Agent], false),
                (None, Some(d)) if d.command.trim().is_empty() => self.report.push(Diagnostic::error(
                    "workflow-dispatch",
                    format!("{at}/dispatch/command"),
                    "the dispatch command is empty",
                )),
                (None, Some(_)) => {}
                (Some(_), Some(_)) => self.report.push(Diagnostic::error(
                    "workflow-dispatch",
                    format!("{at}/dispatch"),
                    "a workflow names a dispatcher or a dispatch command, not both",
                )),
                (None, None) => self.report.push(Diagnostic::error(
                    "workflow-dispatch",
                    at.clone(),
                    "a workflow needs a dispatcher agent or a dispatch command",
                )),
            }
            self.expect(&flow.collector, format!("{at}/collector"), &[Kind::Agent], false);
            for (s, stage) in flow.stages.iter().enumerate() {
                self.stage_agents(stage, &format!("{at}/stages/{s}"));
            }
            stage_ids(flow, &at, self.report);
        }
    }

    fn stage_agents(&mut self, stage: &Stage, at: &str) {
        let named = stage.agent.iter().map(|a| (a.clone(), format!("{at}/agent")));
        let lists = [("workers", &stage.workers), ("reviewers", &stage.reviewers)];
        let listed = lists
            .into_iter()
            .flat_map(|(f, l)| l.iter().enumerate().map(move |(i, a)| (a.clone(), format!("{at}/{f}/{i}"))));
        let personas = stage.personas.keys().map(|a| (a.clone(), format!("{at}/personas/{a}")));
        let escalate = stage.escalate.iter().map(|e| (e.to.clone(), format!("{at}/escalate/to")));
        for (id, path) in named.chain(listed).chain(personas).chain(escalate).collect::<Vec<_>>() {
            self.expect(&id, path, &[Kind::Agent], false);
        }
    }
}

/// Stage ids are well formed and unique within their workflow, and every stage a workflow names is
/// one of its own.
fn stage_ids(flow: &Workflow, at: &str, report: &mut Report) {
    let mut seen = BTreeSet::new();
    for (s, stage) in flow.stages.iter().enumerate() {
        well_formed(&stage.id, &format!("{at}/stages/{s}/id"), report);
        if !seen.insert(stage.id.as_str()) {
            let why = format!("workflow `{}` has two stages with id `{}`", flow.id, stage.id);
            report.push(Diagnostic::error("duplicate-id", format!("{at}/stages/{s}/id"), why));
        }
    }
    let mut named = vec![(flow.entry.as_str(), format!("{at}/entry"))];
    for (s, stage) in flow.stages.iter().enumerate() {
        let routes =
            stage.routes.iter().filter_map(|(k, v)| Some((v.as_deref()?, format!("{at}/stages/{s}/routes/{k}"))));
        named.extend(routes);
        named.extend(stage.on_limit.as_deref().map(|l| (l, format!("{at}/stages/{s}/on_limit"))));
    }
    for (id, path) in named.into_iter().filter(|(id, _)| !seen.contains(id)) {
        let why = format!("`{id}` is not a stage of workflow `{}`", flow.id);
        report.push(Diagnostic::error("unknown-reference", path, why));
    }
}

/// Each kind of stage has the fields it needs, and none it does not use (`stage-field`).
pub fn check(doc: &Document, report: &mut Report) {
    for (w, flow) in doc.workflows.iter().enumerate() {
        for (s, stage) in flow.stages.iter().enumerate() {
            stage_fields(stage, &format!("/workflows/{w}/stages/{s}"), report);
        }
    }
}

fn stage_fields(stage: &Stage, at: &str, report: &mut Report) {
    let mut field =
        |name: &str, why: String| report.push(Diagnostic::error("stage-field", format!("{at}/{name}"), why));
    if let Some((name, why)) = missing(stage) {
        field(name, why.into());
    }
    let kind = stage.kind.unwrap_or(StageKind::Agent);
    let panel_only = !stage.personas.is_empty() || stage.join.is_some() || !stage.priority.is_empty();
    let agent_only = panel_only || !stage.workers.is_empty() || !stage.reviewers.is_empty();
    let stray = [
        ("agent", agent_only || stage.escalate.is_some() || stage.agent.is_some(), StageKind::Agent),
        ("command", stage.command.is_some(), StageKind::Check),
        ("prompt", stage.prompt.is_some(), StageKind::Human),
        ("into", stage.into.is_some(), StageKind::Merge),
    ];
    for (name, set, owner) in stray {
        if set && kind != owner {
            field(name, format!("`{name}` belongs to `{}` stages", kind_name(owner)));
        }
    }
    if stage.reviewers.is_empty() && panel_only {
        field("reviewers", "`personas`, `join` and `priority` belong to a review panel".into());
    }
}

/// What the stage's kind needs and the stage lacks.
fn missing(stage: &Stage) -> Option<(&'static str, &'static str)> {
    let panel = !stage.reviewers.is_empty();
    match stage.kind.unwrap_or(StageKind::Agent) {
        StageKind::Agent if stage.agent.is_none() && !panel => {
            Some(("agent", "an agent stage needs `agent` or `reviewers`"))
        }
        StageKind::Agent if panel && stage.reviewers.len() < 2 => {
            Some(("reviewers", "a review panel has at least two reviewers"))
        }
        StageKind::Check if stage.command.is_none() => Some(("command", "a `check` stage needs `command`")),
        StageKind::Human if stage.prompt.is_none() => Some(("prompt", "a `human` stage needs `prompt`")),
        _ => None,
    }
}

fn kind_name(kind: StageKind) -> &'static str {
    match kind {
        StageKind::Agent => "agent",
        StageKind::Check => "check",
        StageKind::Human => "human",
        StageKind::Merge => "merge",
    }
}
