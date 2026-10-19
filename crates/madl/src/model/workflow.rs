//! Workflows (MEP 0007): a dispatcher splits an input into items, and each item passes through
//! stages along routes chosen by the outcome of the stage before.
use super::{CopyMode, Extensions, Join};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use ts_rs::TS;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Workflow {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub name: Option<String>,
    /// The input the dispatcher splits, by id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub input: Option<String>,
    /// The agent that splits the input into items and hands each to the entry stage.
    pub dispatcher: String,
    /// The stage every item starts at.
    pub entry: String,
    /// The agent that assembles the completed items.
    pub collector: String,
    /// Most items one run may make. Absent: the runner's own limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_items: Option<u32>,
    /// Most steps one item may take through the stages. Absent: the runner's own limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_steps: Option<u32>,
    /// `per-item`: each item gets its own git copy. Absent: `shared`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub copies: Option<CopyMode>,
    pub stages: Vec<Stage>,
    #[serde(default, skip_serializing_if = "Extensions::is_empty")]
    #[ts(as = "Option<Extensions>", optional)]
    pub extensions: Extensions,
}

/// One stage. Ids are unique within the workflow. Which fields apply depends on `kind`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Stage {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub name: Option<String>,
    /// Absent: `agent`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub kind: Option<StageKind>,
    /// `agent`: the agent that does the stage's work.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub agent: Option<String>,
    /// `agent`: more agents that can take items here, one item each at a time.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<String>>", optional)]
    pub workers: Vec<String>,
    /// `agent`: a review panel instead of one agent: every reviewer takes the same item.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<String>>", optional)]
    pub reviewers: Vec<String>,
    /// The angle each reviewer takes, by agent id: `red-team`, `adversarial`, `simplifier`,
    /// `user`, `expert`, `good-friend`, `ownership`, or the runner's own.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[ts(as = "Option<BTreeMap<String, String>>", optional)]
    pub personas: BTreeMap<String, String>,
    /// A panel's decision: every reviewer's (`all`, the default) or the first approval (`any`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub join: Option<Join>,
    /// A panel's outcomes, worst first: the worst outcome any reviewer gives is the panel's.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<String>>", optional)]
    pub priority: Vec<String>,
    /// Outcome → next stage. `null` completes the item. The outcome `blocked` always stops it.
    #[ts(type = "Record<string, string | null>")]
    pub routes: BTreeMap<String, Option<String>>,
    /// `check`: the shell command; exit 0 is the outcome `passed`, anything else `failed`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub timeout_s: Option<u32>,
    /// `human`: what the person decides; the outcomes are the route keys.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub prompt: Option<String>,
    /// `merge`: the branch to merge into. Absent: the project's current branch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub into: Option<String>,
    /// How often one item may enter this stage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_rounds: Option<u32>,
    /// The stage an item goes to when it would enter past `max_rounds`. Absent: it is blocked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub on_limit: Option<String>,
    /// `agent`: after this many rounds here, items go to a stronger agent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub escalate: Option<StageEscalation>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum StageKind {
    /// An agent, or a review panel, does the work and reports an outcome.
    Agent,
    /// The runner runs a command.
    Check,
    /// A person decides.
    Human,
    /// The runner merges the item's branch.
    Merge,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct StageEscalation {
    pub after_rounds: u32,
    pub to: String,
}
