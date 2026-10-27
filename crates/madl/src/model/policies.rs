//! Failure handling, escalation, schedules, guards and change rules; drawing; extensions.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use ts_rs::TS;

/// Vendor data, keyed by reverse-DNS namespace (`com.usesculptured`, `ai.camel`). Readers keep
/// namespaces they don't know.
pub type Extensions = BTreeMap<String, Value>;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Policies {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub on_failure: Option<OnFailure>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<Escalation>>", optional)]
    pub escalation: Vec<Escalation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<Schedule>>", optional)]
    pub schedules: Vec<Schedule>,
    /// Project checks that pause new work while they fail.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<Guard>>", optional)]
    pub guards: Vec<Guard>,
    /// Rules for the changes agents propose to systems outside the run (MEP 0009).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub changes: Option<ChangePolicy>,
}

/// What a change an agent proposes (an order, a reschedule, a record in another system) must meet
/// before it is made, and whether one that meets every rule needs a person (MEP 0009).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct ChangePolicy {
    /// `true`: a change that passes every rule that applies to it is approved without a person. A
    /// change no rule applies to always waits for one. Default `false`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub approve_within_rules: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<ChangeRule>>", optional)]
    pub rules: Vec<ChangeRule>,
}

/// For changes of `action` (`*`, the default: every change), the change's `field` passes exactly one
/// test: `at_most`, `at_least`, `one_of` or `present`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct ChangeRule {
    pub id: String,
    /// The kind of change, as the agent names it (`create_purchase_order`), or `*`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub action: Option<String>,
    /// A dotted path into the change's fields, such as `amount` or `lines.0.item`.
    pub field: String,
    /// A number, or a number written as text, no greater than this.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub at_most: Option<f64>,
    /// A number no less than this.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub at_least: Option<f64>,
    /// One of these, compared as text, ignoring case and surrounding spaces.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub one_of: Option<Vec<String>>,
    /// `true`: present and not empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub present: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct OnFailure {
    pub strategy: FailureStrategy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_attempts: Option<u32>,
    /// What to do once `max_attempts` is used up.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub then: Option<FailureStrategy>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum FailureStrategy {
    Retry,
    /// Plan the remaining work again.
    Replan,
    /// Split the failed work into smaller pieces.
    Decompose,
    /// Hand it to the agent or person in `escalation`.
    Escalate,
    Stop,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Escalation {
    pub from: String,
    pub after_rounds: u32,
    pub to: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Schedule {
    pub id: String,
    pub agent: String,
    /// A duration such as `10m` or `24h`.
    pub every: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Guard {
    pub command: String,
    pub every: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub timeout_s: Option<u32>,
}

/// Positions for drawing. Writers keep the entries of components they did not change.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Layout {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub units: Option<String>,
    /// Agents, steps and inputs, by id.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[ts(as = "Option<BTreeMap<String, Point>>", optional)]
    pub nodes: BTreeMap<String, Point>,
    /// When the document has a layout, a group without a box here is not drawn.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[ts(as = "Option<BTreeMap<String, Rect>>", optional)]
    pub groups: BTreeMap<String, Rect>,
    /// Workflow stages, by workflow id and then stage id (MEP 0006).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[ts(as = "Option<BTreeMap<String, BTreeMap<String, Point>>>", optional)]
    pub stages: BTreeMap<String, BTreeMap<String, Point>>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}
