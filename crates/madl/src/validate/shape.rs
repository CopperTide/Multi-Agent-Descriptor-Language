//! Fields that depend on a kind: steps, patterns and ban rules.
use crate::diagnostic::{Diagnostic, Report};
use crate::model::{Document, EmbeddedSkill, PatternKind, SkillRef, Step, StepKind, Tool, ToolRef};
use std::collections::BTreeSet;

pub fn check(doc: &Document, report: &mut Report) {
    for (i, step) in doc.steps.iter().enumerate() {
        step_fields(step, &format!("/steps/{i}"), report);
    }
    patterns(doc, report);
    skills(doc, report);
    tools(doc, report);
    if let Some(c) = doc.policies.as_ref().and_then(|p| p.changes.as_ref()) {
        change_rules(&c.rules, report);
    }
    let rules = doc.communication.iter().flat_map(|c| c.deny.iter().enumerate());
    for (i, rule) in rules {
        let pair = rule.from.is_some() && rule.to.is_some();
        let one_way = rule.from.is_some() || rule.to.is_some();
        if pair == rule.between.is_some() || (one_way && !pair) {
            report.push(Diagnostic::error(
                "deny-shape",
                format!("/communication/deny/{i}"),
                "a ban has either `from` and `to`, or `between`",
            ));
        }
    }
}

/// Each change rule has exactly one test, and a rule id is used once.
fn change_rules(rules: &[crate::model::ChangeRule], report: &mut Report) {
    let mut seen = BTreeSet::new();
    for (i, r) in rules.iter().enumerate() {
        let at = format!("/policies/changes/rules/{i}");
        if !seen.insert(r.id.as_str()) {
            report.push(Diagnostic::error(
                "duplicate-id",
                format!("{at}/id"),
                format!("`{}` is already a rule here", r.id),
            ));
        }
        let tests = [r.at_most.is_some(), r.at_least.is_some(), r.one_of.is_some(), r.present == Some(true)];
        if tests.iter().filter(|t| **t).count() != 1 {
            report.push(Diagnostic::error(
                "change-rule-shape",
                at,
                "a change rule has exactly one test: `at_most`, `at_least`, `one_of` or `present: true`",
            ));
        }
    }
}

fn step_fields(step: &Step, at: &str, report: &mut Report) {
    let (needs, present): (&str, bool) = match step.kind {
        StepKind::Check => ("command", step.command.is_some()),
        StepKind::Human => ("prompt", step.prompt.is_some()),
        StepKind::FanOut => ("over", step.over.is_some()),
        StepKind::Merge | StepKind::FanIn => ("", true),
    };
    if !present {
        report.push(Diagnostic::error(
            "step-field",
            format!("{at}/{needs}"),
            format!("a `{}` step needs `{needs}`", kind_name(step.kind)),
        ));
    }
    stray_fields(step, at, report);
}

/// Fields set on a step whose kind does not use them.
fn stray_fields(step: &Step, at: &str, report: &mut Report) {
    let stray = [
        ("command", step.command.is_some(), StepKind::Check),
        ("prompt", step.prompt.is_some(), StepKind::Human),
        ("outcomes", !step.outcomes.is_empty(), StepKind::Human),
        ("into", step.into.is_some(), StepKind::Merge),
        ("over", step.over.is_some(), StepKind::FanOut),
        ("copies", step.copies.is_some(), StepKind::FanOut),
        ("join", step.join.is_some(), StepKind::FanIn),
    ];
    for (field, set, owner) in stray {
        if set && step.kind != owner {
            report.push(Diagnostic::error(
                "step-field",
                format!("{at}/{field}"),
                format!("`{field}` belongs to `{}` steps, not `{}`", kind_name(owner), kind_name(step.kind)),
            ));
        }
    }
}

fn kind_name(kind: StepKind) -> &'static str {
    match kind {
        StepKind::Check => "check",
        StepKind::Human => "human",
        StepKind::Merge => "merge",
        StepKind::FanOut => "fan-out",
        StepKind::FanIn => "fan-in",
    }
}

fn patterns(doc: &Document, report: &mut Report) {
    let Some(t) = &doc.topology else { return };
    for (i, p) in t.patterns.iter().enumerate() {
        let at = format!("/topology/patterns/{i}");
        let problem = match p.pattern {
            PatternKind::Supervisor if p.supervisor.is_none() => Some("a `supervisor` pattern needs `supervisor`"),
            PatternKind::Auction if p.auctioneer.is_none() => Some("an `auction` pattern needs `auctioneer`"),
            PatternKind::Committee if p.source.is_none() => Some("a `committee` pattern needs `source`"),
            PatternKind::RolePlay if p.members.len() != 2 => Some("a `role-play` pattern has exactly two members"),
            PatternKind::Pipeline | PatternKind::Debate | PatternKind::Swarm if p.members.len() < 2 => {
                Some("this pattern needs at least two members")
            }
            _ if p.members.is_empty() => Some("a pattern needs members"),
            _ => None,
        };
        if let Some(message) = problem {
            report.push(Diagnostic::error("pattern-shape", at, message));
        }
    }
}

/// Embedded skills follow the Agent Skills format, and an agent's skills have different names.
fn skills(doc: &Document, report: &mut Report) {
    for (a, agent) in doc.agents.iter().enumerate() {
        let mut seen = BTreeSet::new();
        for (i, skill) in agent.skills.iter().enumerate() {
            if let SkillRef::Embedded(s) = skill {
                embedded(s, &format!("/agents/{a}/skills/{i}"), &mut seen, report);
            }
        }
    }
}

fn embedded<'a>(s: &'a EmbeddedSkill, at: &str, seen: &mut BTreeSet<&'a str>, report: &mut Report) {
    let n = &s.name;
    let good_name = !n.is_empty()
        && n.len() <= 64
        && n.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !n.starts_with('-')
        && !n.ends_with('-')
        && !n.contains("--");
    if !good_name {
        let why = "a skill's name is 1–64 lowercase letters, digits and single hyphens";
        report.push(Diagnostic::error("skill-shape", format!("{at}/name"), why));
    }
    let d = s.description.trim().chars().count();
    if d == 0 || d > 1024 {
        let why = "a skill's description says when to use it, in 1–1024 characters";
        report.push(Diagnostic::error("skill-shape", format!("{at}/description"), why));
    }
    if s.body.trim().is_empty() {
        report.push(Diagnostic::error("skill-shape", format!("{at}/body"), "an embedded skill needs a body"));
    }
    if s.source
        .as_ref()
        .is_some_and(|u| !u.starts_with("https://") || u.len() > 500 || u.chars().any(char::is_whitespace))
    {
        let why = "a skill's source is the https:// address it was copied from, at most 500 characters";
        report.push(Diagnostic::error("skill-shape", format!("{at}/source"), why));
    }
    if !seen.insert(n.as_str()) {
        let why = format!("two skills are named `{n}`");
        report.push(Diagnostic::error("duplicate-skill", format!("{at}/name"), why));
    }
}

/// A tool says what it is at most once, and the skills it carries are well formed (MEP 0006).
fn tools(doc: &Document, report: &mut Report) {
    let inline = doc.agents.iter().enumerate().flat_map(|(a, agent)| {
        agent.tools.iter().enumerate().filter_map(move |(i, t)| match t {
            ToolRef::Def(t) => Some((&**t, format!("/agents/{a}/tools/{i}"))),
            ToolRef::Ref(_) => None,
        })
    });
    let shared = doc.tools.iter().enumerate().map(|(i, t)| (t, format!("/tools/{i}")));
    for (tool, at) in shared.chain(inline) {
        tool_shape(tool, &at, report);
    }
}

fn tool_shape(t: &Tool, at: &str, report: &mut Report) {
    let kinds = [
        ("mcp", t.mcp.is_some()),
        ("builtin", t.builtin.is_some()),
        ("hook", t.hook.is_some()),
        ("plugin", t.plugin.is_some()),
        ("skills", !t.skills.is_empty()),
    ];
    let set: Vec<&str> = kinds.iter().filter(|(_, s)| *s).map(|(k, _)| *k).collect();
    if set.len() > 1 {
        let why = format!("a tool is one thing; this one sets {}", set.join(" and "));
        report.push(Diagnostic::error("tool-shape", format!("{at}/{}", set[1]), why));
    }
    let mut seen = BTreeSet::new();
    for (i, s) in t.skills.iter().enumerate() {
        embedded(s, &format!("{at}/skills/{i}"), &mut seen, report);
    }
}
