# Document

## Format

A document is a YAML 1.2 or JSON object. JSON is the canonical form: a reader that writes a
document SHOULD write JSON or YAML that converts to the same JSON. YAML anchors and merge keys
(`<<`) MAY be used and are resolved before anything else. Files are named `<name>.madl.yaml` or
`<name>.madl.json`.

Top-level members:

| Member | Required | |
|---|---|---|
| `$schema` | no | The URL of the document's JSON Schema, for editors. It has no meaning |
| `madl` | yes | The revision the document is written for: `draft`, or a dated revision such as `2027-01-15` |
| `system` | yes | `id`, `name`, optional `description` and [`budget`](#run-budget) |
| `defaults` | no | The global [settings block](settings.md) |
| `inputs` | no | What the system is given when it starts |
| `groups` | no | Groups |
| `agents` | no | Agents |
| `tools` | no | [Tools](#tools) defined once and given to agents by id |
| `steps` | no | Non-agent nodes |
| `workflows` | no | [Workflows](topology.md#workflows): items passed through stages |
| `channels` | no | [Channels](communication.md#channels) |
| `connections` | no | [Connections](communication.md#connections) |
| `communication` | no | System-wide [mode and bans](communication.md#open-and-closed-communication) |
| `topology` | no | [Patterns and loop limits](topology.md) |
| `policies` | no | [Failure handling, escalation, schedules, guards](topology.md#policies) |
| `layout` | no | Positions for drawing |
| `extensions` | no | Vendor data |

A reader MUST reject a document with members not defined here (code `shape`). `signatures` is
reserved for a later revision.

A reader MUST reject a document whose `madl` names a revision it does not implement
(`unknown-revision`).

## Ids

Agents, groups, tools, steps, workflows, inputs, channels, connections, patterns and schedules have
ids. A workflow's stages have ids that are unique within their workflow. An id
MUST match `^[A-Za-z0-9][A-Za-z0-9_-]{0,63}$` (`bad-id`) and MUST be unique across all of these
in one document (`duplicate-id`). Because ids are unique document-wide, every reference is a plain
id.

Ids are stable: tools rely on them to keep drawings, history and review comments attached to the
same component across revisions. Renaming a component changes its `name`, never its `id`.

A connection or pattern MAY omit its id; its id is then `<from>-<kind>-<to>` for a connection and
`<pattern>-<position>` (counting from 1) for a pattern.

## `$ref`

Any object whose only member is `$ref` is replaced by what it refers to before the document is
validated:

```yaml
agents: { $ref: ./agents.yaml }              # a whole file
defaults: { $ref: ./shared.yaml#/defaults }  # a JSON Pointer into a file
instructions: { $ref: ./prompts/cad.md }     # a non-YAML/JSON file becomes its text
connections: { $ref: "#/x-shared/links" }    # a pointer into the same file
```

- A target ending in `.yaml`, `.yml` or `.json` is parsed, and its own `$ref`s are resolved
  relative to it. Any other file becomes a string holding its text; a pointer into it is an error
  (`ref-pointer`).
- Paths are relative to the file containing the `$ref`. A reader MUST NOT follow a `$ref` outside
  the folder of the document being loaded unless the person running it opts in (`ref-outside`),
  and MUST NOT fetch over a network.
- A `$ref` that leads back to itself is an error (`ref-cycle`); a missing target is `ref-missing`.

## System and inputs

```yaml
system: { id: cad-review, name: CAD review team, description: Designs and reviews a bracket. }
inputs:
  - id: brief
    text: { $ref: ./brief.md }
    to: [planner]
```

An input's `to` lists agents, groups or steps; empty means the [starting agents](topology.md#starting-points).
An agent an input names receives the input after its own instructions when the system starts; an
agent further along receives it with its first handoff.

An input without `text` (or with empty text) is given when the run starts, such as the message a
person opens a conversation with. A Runner SHOULD ask the person starting the run for it, and MUST
NOT start the agents it goes to with an empty input in its place.

## Run budget

`system.budget` is what one run of the whole system may use, every agent and everything they spawn
together ([MEP 0002](../../meps/0002-soft-and-hard-budgets.md)). Each limit is a number or duration,
or `{ soft, hard }` as in [settings](settings.md#limits).

```yaml
system:
  id: c172
  name: C172 model
  budget:
    plan: [ { provider: claude, window: five_hour, soft: 25, hard: 30 } ]
    tokens: { hard: 50000000 }
    wall_clock: { soft: 45m, hard: 1h }
    max_working: 6
    max_agents: 12
```

| Member | Meaning |
|---|---|
| `plan` | Shares of a provider's plan window in percent, above 0 and at most 100 (`budget-percent-out-of-range`). The run's share is the window's change since the run started, so use outside the run counts too. |
| `plan[].pause` | At the hard value, pause the run instead of stopping it, and continue once the window resets: `continue_with` (`notes`, the default: each agent starts afresh from the note it wrote when the run wrapped up; `resume`: each continues its own conversation) and `max_windows` (default 3, at least 1: `budget-no-windows`) ([MEP 0008](../../meps/0008-runner-settings.md)) |
| `tokens`, `cost_usd`, `wall_clock` | Limits for the run |
| `max_working` | Most agents working at the same time; the rest wait for a free slot; at least 1 (`budget-no-room`) |
| `max_agents` | Most agents in the system at once, its own and those they spawn together; at least 1 (`budget-no-room`) ([MEP 0006](../../meps/0006-shared-definitions.md)) |

A run budget and the agents' own budgets all apply; whichever is reached first acts.

## Groups

```yaml
groups:
  - id: design
    name: Design
    purpose: Produce a manufacturable bracket that meets the brief.
    parent: null
    settings: { … }
```

A group holds exactly the agents whose `groups` list it; an agent MAY be in no group. Groups nest
through `parent`; a group MUST NOT be inside itself (`group-cycle`). An agent in a group is also a
member of every group that encloses it. A runner SHOULD tell each member, and each agent connected
to the group, the group's name and purpose.

## Agents

| Member | |
|---|---|
| `id`, `name` | |
| `description` | What the agent does, in a sentence or two: how other agents and people see it. A Runner SHOULD give it to the agents connected to this one, so they can choose whom to hand work to |
| `instructions` | The agent's job: the first message it receives |
| `runtime`, `model`, `resume` | Shorthand for the same fields of `settings.preferences`; setting both is an error (`shorthand-conflict`) |
| `endpoint` | For `runtime: remote-a2a`: the agent's A2A endpoint or Agent Card URL; required for that runtime (`remote-needs-endpoint`) |
| `groups` | Group ids |
| `tools` | Tools the agent is given: a [tool](#tools), or the id of one of the document's `tools` (or of a tool the runner provides). Restrictions decide which may be used |
| `skills` | Skill paths or names, or embedded skills `{ name, description, body }` in the Agent Skills format (name: lowercase letters, digits and hyphens, at most 64; description: when to use it, at most 1024 characters) |
| `context` | Files given as context |
| `workspace` | `folder`, and `copy`: `own` (its own git copy), `shared`, or `per-item`. Each member left out comes from [`preferences.workspace`](settings.md#preferences) |
| `placement` | `machine`: properties a machine must have, such as `os` or `name` |
| `outputs` | `{ id, path, description, acceptance }`: what the agent produces; connections `carry` them. `acceptance`: checks that must all pass for the output to count as done |
| `data_access` | `{ reads, writes, side_effects, rationale, shared }`: the agent's data contract (see below) |
| `authority` | `{ domains, rationale }`: the decisions the agent owns (see below) |
| `checks` | `{ command, after: each-turn \| finish, timeout_s }`: exit 0 means passed |
| `settings`, `subagents` | See [Settings](settings.md) |
| `extensions` | Vendor data |

Runtimes are `claude-code`, `codex`, `camel`, `crewai`, `langgraph`, `openai-agents`, `autogen`,
`google-adk`, `microsoft-agent-framework`, `remote-a2a`, `human`, or an extension name
`x-<vendor>-<name>` (`unknown-runtime`). `human` makes
a person a participant: people receive handoffs, answer `ask` connections and give approvals.

### Data access and authority

`data_access` declares what an agent is for: the paths it reads and writes, relative to the
document (a trailing `/` or `/**` covers a folder), its effects beyond files in plain words, and why.
A Runner SHOULD refuse a write outside `writes` where it can, and MUST tell the person where it
cannot. `restrictions.files` stays the hard limit; the two combine by tightening.

`data_access.shared` lists values other agents rely on, such as a port, an endpoint or a field name:
`{ name, value, file }`, where `file` (relative to the document) must contain `value` exactly. A
Runner SHOULD check each at every start, tell the agents that rely on it, and say when a file no
longer holds it ([MEP 0006](../../meps/0006-shared-definitions.md)).

`authority` names the decisions an agent owns. A Runner SHOULD tell every agent who decides what,
and send a question within a domain to its owner. Its `scope` (a group id, or `system`) says how far
those decisions reach: a decision belongs to the agent whose scope contains its consequences.
Specialists settle what stays inside their own work with each other directly; a decision whose
consequences cross a group's boundary goes to the agent with that group as its scope. A Runner SHOULD
tell agents so. Two agents that may write the same path, or own
the same domain, get a warning (`write-overlap`, `authority-overlap`): either can be intended, but
the author should know.

An output's `acceptance` checks decide when it is done: every one exits 0. A Runner MUST NOT report
an output as verified without running them.

## Tools

A tool is what an agent is given beyond its instructions. It is written on the agent, or once in
the top-level `tools` and given to agents by id ([MEP 0006](../../meps/0006-shared-definitions.md)):

```yaml
tools:
  - id: notes
    name: Notes server
    description: Shared notes.
    instructions: Write what you learn with `note`.   # what agents given it are told
    mcp: { command: notes-mcp, args: [--stdio] }
    install: cargo install notes-mcp                  # for the person setting up a machine
  - id: docs
    mcp: { url: "https://docs.example.com/mcp", headers: { Authorization: "Bearer ${DOCS_TOKEN}" } }
    requires: [notes]
  - { id: money, skills: [ { name: money-rules, description: Use when handling money., body: Decimal only. } ] }
  - { id: guard, hook: { command: signet-eval, args: [--rules-path, "{call_rules}"] }, runtimes: [claude-code] }
  - { id: review, plugin: review@acme, runtimes: [claude-code] }
agents:
  - { id: wing, tools: [notes, docs, { id: search, builtin: "claude-code:WebSearch" }] }
```

At most one of these says what a tool is (`tool-shape`); with none, its id names a tool the runner
provides.

| Member | |
|---|---|
| `mcp` | An MCP server: `command`, `args` and `env` to start one (stdio), or `url` and `headers` to reach one (HTTP) |
| `builtin` | A runtime's own tool, as `<runtime>:<name>` |
| `hook` | `{ command, args }`: a program the runner calls before each of the agent's tool calls, with the call as JSON in the runtime's hook format; it may refuse the call. `{call_rules}` in an argument is replaced by the path of a file holding the agent's effective [`restrictions.tools.refuse`](settings.md#restrictions) |
| `plugin` | A plugin installed for the agent's runtime, as `name@marketplace` |
| `skills` | Embedded skills, added to the agent's own; nothing runs |

Any tool may have `name`, `description`, `instructions` (what agents given it are told), `runtimes`
(the runtimes it works with; empty: any, each a known runtime: `unknown-runtime`), `requires` (tool
ids that come with it), `install` (how to set it up on a machine) and `homepage`.

Environment values in an MCP server's `env` are either text or `{ secret: NAME }`, and `${NAME}` in
its `url` or `headers` names a secret the runner fills in; secret values MUST NOT be written into
documents.

## Steps

| `kind` | Requires | May have |
|---|---|---|
| `check` | `command` | `timeout_s` |
| `human` | `prompt` | `outcomes` |
| `merge` | | `into` (default: the current branch) |
| `fan-out` | `over` (a [condition expression](topology.md#conditions) naming a list) | `copies` |
| `fan-in` | | `join`: `all` or `any` |

A step with a field it does not use, or without one it requires, is an error (`step-field`).

## Layout

```yaml
layout:
  units: px
  nodes: { planner: { x: 0, y: 80 }, brief: { x: -220, y: 80 } }
  groups: { design: { x: -40, y: 20, w: 480, h: 180 } }
```

`nodes` holds positions for agents, steps and inputs; `groups` holds boxes, and in a document with a
`layout` a group without a box is not drawn; `stages` holds positions for [workflow](topology.md#workflows)
stages, by workflow id and then stage id. Layout has no meaning for execution. A reader MAY ignore it; a writer MUST keep the entries of components it did not
change.

## Extensions

`extensions` maps reverse-DNS namespaces (`com.usesculptured`, `ai.camel`) to any JSON value. It
MAY appear at the top level and on agents, groups, channels, connections, steps and workflows. A key that is
not a reverse-DNS name is an error (`bad-extension-key`). Readers MUST ignore namespaces they do
not know and MUST keep them when they write the document back.

Extension data MUST NOT change the meaning of core members: a reader that ignores every extension
still reads the same system.
