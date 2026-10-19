//! Limits with a soft and a hard value (MEP-0002), and the budget of a whole run.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A limit's soft value (no new work; wrap up) and hard value (stop at once).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct SoftHard<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub soft: Option<T>,
    pub hard: T,
}

/// A limit: a bare value is its hard value; `{ soft, hard }` adds a soft value at or below it.
/// Without a soft value, a runtime may choose one (for example 85% of hard).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(untagged)]
#[ts(export)]
pub enum Limit<T> {
    Hard(T),
    SoftHard(SoftHard<T>),
}

impl<T> Limit<T> {
    pub fn hard(&self) -> &T {
        match self {
            Limit::Hard(h) | Limit::SoftHard(SoftHard { hard: h, .. }) => h,
        }
    }

    pub fn soft(&self) -> Option<&T> {
        match self {
            Limit::Hard(_) => None,
            Limit::SoftHard(s) => s.soft.as_ref(),
        }
    }
}

/// A share of one plan window (such as a subscription's 5-hour window) that a run may add.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct PlanShare {
    /// The plan's provider, e.g. `claude` or `codex`.
    pub provider: String,
    /// The window, e.g. `five_hour` or `seven_day`.
    pub window: String,
    /// Percent of the window, above 0 and at most 100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub soft: Option<f64>,
    pub hard: f64,
    /// At the hard value, pause the run instead of stopping it, and continue once the window resets
    /// (MEP 0006).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub pause: Option<PlanPause>,
}

/// How a run paused at a plan share continues.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct PlanPause {
    /// Absent: `notes`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub continue_with: Option<ContinueWith>,
    /// Most windows the run may pause for, at least 1. Absent: 3.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_windows: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum ContinueWith {
    /// Each agent starts a fresh conversation from the note it wrote when the run wrapped up.
    Notes,
    /// Each agent continues its own conversation.
    Resume,
}

/// What one run of the whole system may use, every agent and everything they spawn together.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct RunBudget {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<PlanShare>>", optional)]
    pub plan: Vec<PlanShare>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "Limit<number>")]
    pub tokens: Option<Limit<u64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub cost_usd: Option<Limit<f64>>,
    /// A duration such as `90s`, `30m`, `2h` or `1h30m`, from the run's start.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub wall_clock: Option<Limit<String>>,
    /// Most agents working at the same time; the rest wait for a free slot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_working: Option<u32>,
    /// Most agents in the system at once, its own and the ones they spawn together (MEP 0006).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_agents: Option<u32>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Budget {
    /// Tokens of every kind. A bare number is the hard value; `{ soft, hard }` adds a soft one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "Limit<number>")]
    pub tokens: Option<Limit<u64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub cost_usd: Option<f64>,
    /// A duration such as `90s`, `30m`, `2h` or `1h30m`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub wall_clock: Option<Limit<String>>,
    /// Turns (model replies to a message) in one run (MEP-0002).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "Limit<number>")]
    pub turns: Option<Limit<u64>>,
    /// Time spent working (in turns), as a duration (MEP-0002).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub working_time: Option<Limit<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub scope: Option<BudgetScope>,
    /// Which tokens count toward `tokens`. Absent: `all` (MEP 0006).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub counts: Option<super::TokenCount>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum BudgetScope {
    /// The agent and everything it spawns draw from one budget.
    Shared,
    /// Each agent has a budget of its own.
    Own,
}
