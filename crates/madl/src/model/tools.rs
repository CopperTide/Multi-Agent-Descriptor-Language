//! Tools: MCP servers, a runtime's own tools, hooks on tool calls, plugins and packs of skills,
//! defined on an agent or once for the document and given to agents by id (MEP 0006).
use super::EmbeddedSkill;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use ts_rs::TS;

/// A tool given to an agent: the tool itself, or the id of one of the document's `tools` (or of a
/// tool the runner provides).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(untagged)]
#[ts(export)]
pub enum ToolRef {
    Ref(String),
    Def(Box<Tool>),
}

impl ToolRef {
    pub fn id(&self) -> &str {
        match self {
            ToolRef::Ref(id) => id,
            ToolRef::Def(t) => &t.id,
        }
    }
}

impl From<Tool> for ToolRef {
    fn from(t: Tool) -> Self {
        ToolRef::Def(Box::new(t))
    }
}

/// What an agent is given beyond its instructions. At most one of `mcp`, `builtin`, `hook`,
/// `plugin` and `skills` says what it is; with none, the id names a tool the runner provides.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Tool {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
    /// What agents given the tool are told about it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub instructions: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub mcp: Option<McpServer>,
    /// A runtime's own tool, as `<runtime>:<name>`, e.g. `claude-code:Bash`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub builtin: Option<String>,
    /// A program the runner calls before each of the agent's tool calls; it may refuse the call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub hook: Option<Hook>,
    /// A plugin installed for the agent's runtime, as `name@marketplace`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub plugin: Option<String>,
    /// Skills added to the agent's own; nothing runs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<EmbeddedSkill>>", optional)]
    pub skills: Vec<EmbeddedSkill>,
    /// The runtimes it works with. Empty: any.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<String>>", optional)]
    pub runtimes: Vec<String>,
    /// Other tools it needs, by id; an agent given this one is given them too.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<String>>", optional)]
    pub requires: Vec<String>,
    /// How to set it up on a machine, for the person running the system.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub install: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub homepage: Option<String>,
}

/// How to reach an MCP server: a command to start (stdio) or a URL (HTTP).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct McpServer {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<String>>", optional)]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[ts(as = "Option<BTreeMap<String, EnvValue>>", optional)]
    pub env: BTreeMap<String, EnvValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub url: Option<String>,
    /// HTTP request headers. `${NAME}` in a value (or in `url`) is a secret the runner fills in.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[ts(as = "Option<BTreeMap<String, String>>", optional)]
    pub headers: BTreeMap<String, String>,
}

/// A program run with the pending call as JSON on its input, in the runtime's hook format.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Hook {
    pub command: String,
    /// `{call_rules}` in an argument is replaced by the path of a file holding the agent's
    /// effective `restrictions.tools.refuse`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<String>>", optional)]
    pub args: Vec<String>,
}

/// A value given as text, or a secret the runner resolves. Secrets never appear in documents.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(untagged)]
#[ts(export)]
pub enum EnvValue {
    Text(String),
    Secret(SecretRef),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct SecretRef {
    /// The secret's name in the runner's secret store.
    pub secret: String,
}
