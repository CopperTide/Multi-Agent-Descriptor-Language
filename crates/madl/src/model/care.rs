//! What a runner does around an agent (MEP 0006): how much reaches its context, which of its
//! runtime's own configuration it loads, rules that refuse tool calls, and how tokens count.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Limits that keep an agent's context small. Absent members: the runner's own values.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct ContextCare {
    /// `false`: the runner leaves the agent's context alone, with none of these limits or its own.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub managed: Option<bool>,
    /// A read of a whole file larger than this many KB is refused; the agent reads a part instead.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub read_max_kb: Option<u32>,
    /// The most output of one tool call, in characters, that reaches the agent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub tool_output_chars: Option<u32>,
    /// Past this many tokens of context, the agent writes notes and continues in a fresh context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "number")]
    pub refresh_at_tokens: Option<u64>,
}

/// Which of its runtime's own configuration an agent loads. Absent members: the runner's choice.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct RuntimeConfig {
    /// The configuration of the person whose machine runs it (such as `~/.claude` or `~/.codex`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub user: Option<bool>,
    /// The configuration in the agent's folder (such as `.claude/`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub project: Option<bool>,
}

/// A rule that refuses tool calls: every condition holds for a call to a matching tool.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct CallRule {
    pub name: String,
    /// A regular expression over the tool's name. Absent: every tool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub tool: Option<String>,
    /// In Signet-Eval's condition language, such as `contains(parameters, 'rm -rf')`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<String>>", optional)]
    pub conditions: Vec<String>,
    /// What the agent is told when the rule refuses its call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub reason: Option<String>,
}

/// Which tokens count toward a token limit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum TokenCount {
    /// Input, output, cache reads and cache writes.
    All,
    /// Everything except cache reads.
    New,
    Output,
}
