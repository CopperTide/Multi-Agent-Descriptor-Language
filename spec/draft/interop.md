# Interoperability

These mappings are informative.

## Sculptured

[Sculptured](https://github.com/Alexander-Aghili/Sculptured) keeps every team as a MADL document: it exports
teams as `.madl.json` (or YAML), imports either form, and an exported team imported again is
unchanged. Everything that describes a team is in core MADL; what changes while a team runs
(progress, work items, budgets used, questions and answers) stays in Sculptured.

| Sculptured | MADL |
|---|---|
| team id, name | `system.id`, `system.name` |
| agent kind: Claude Code, Codex, Practice | `runtime: claude-code`, `codex`, `x-sculptured-practice` |
| agent Job | `instructions` |
| agent folder; the team's folder; own copies | `workspace.folder`; `defaults.preferences.workspace.folder`; `defaults.preferences.workspace.copy: own` |
| models: agent; group and team (per kind) | the agent's `model`; `preferences.models` on the group and in `defaults`, as `provider/model` |
| token, turn and working-time limits | `settings.restrictions.budget { tokens, counts, turns, working_time, scope }` |
| run budget, plan pauses, most agents at once | `system.budget`, with `plan[].pause` and `max_agents` |
| restricted permissions (team or agent) | `restrictions.approvals.required_for: [runtime-permissions]`; an unrestricted team: `[]` |
| question timeout | `defaults.preferences.questions.timeout` |
| protections; Claude Code setup | `defaults.preferences.context`; `defaults.preferences.config` |
| safety rules | `defaults.restrictions.tools.refuse` |
| components; an agent's components | `tools`; the agent's `tools`, by id |
| skills; contracts with shared values; check | `skills` (embedded); `data_access` and `authority`; `checks: [ { command, after: each-turn } ]` |
| `resume` | `resume` |
| groups with their boxes | `groups`, and `layout.groups` (a group with no box is not drawn) |
| hands work to, reports to, watches | `handoff`, `report`, `watch` |
| the Input | `inputs` |
| most levels of children | `defaults.restrictions.spawn.max_depth` |
| schedules | `policies.schedules` |
| the decision log | a `decision-log` channel |
| workflow, its guard, its board positions | `workflows`, `policies.guards`, `layout.stages` and `layout.nodes` |

## CAMEL

[CAMEL](https://github.com/camel-ai/camel)'s Workforce maps onto MADL as follows:

| CAMEL | MADL |
|---|---|
| `Workforce` | a group with a `supervisor` pattern, `routing: dynamic` |
| nested `Workforce` | a child group (`parent`) |
| `coordinator_agent` | the supervisor |
| `task_agent` (decomposition) | `policies.on_failure.strategy: decompose`, or an agent that delegates to the coordinator |
| `SingleAgentWorker` | an agent with `runtime: camel`, `tools`, `instructions` |
| `RolePlayingWorker` (`chat_turn_limit`) | a `role-play` pattern with `max_rounds` |
| `new_worker_agent` | `subagents.template` |
| `share_memory` | a `shared-memory` channel among the workers |
| `HumanToolkit` | a `human` participant with `ask` and `approve` connections |
| recovery: retry, replan, decompose | `policies.on_failure` |
| `task_timeout_seconds` | `restrictions.budget.wall_clock` |

What CAMEL defines in Python code (custom toolkits, callables) is referenced from
`extensions.ai.camel` by import path.

## A2A

An agent with `runtime: remote-a2a` has an `endpoint`: its A2A endpoint or Agent Card URL.
Connections that use `a2a@<version>` travel over A2A. A tool can publish an A2A Agent Card for each
agent from its name, its `description` (else its instructions' first paragraph) and its tools.

## Google ADK

`madl import adk <root_agent.yaml or its folder>` builds a document from an
[ADK](https://adk.dev) agent config tree, reading the YAML and never running Python. It lists every
part that did not carry over exactly as `approximated` or `unsupported`, with the source file and
JSON Pointer; what it does not list was kept.

| ADK | MADL |
|---|---|
| `LlmAgent`: `name`, `description`, `instruction`, `model` | an agent: `id`, `description`, `instructions`, `model: google/<model>`; `runtime: google-adk` in `defaults`. An agent without a model gets the nearest enclosing `LlmAgent`'s, as in ADK |
| `SequentialAgent` | a group (`purpose` from `description`); `handoff` from each sub-agent to the next |
| `LoopAgent` (`max_iterations`) | the same, and a `handoff` from the last back to the first with `max_rounds`. Leaving early when an agent escalates (`exit_loop`) is approximated |
| `ParallelAgent` | a group; the work goes to every sub-agent. ADK's wait for every branch is approximated: the next agent receives each branch's handoff |
| `sub_agents` of an `LlmAgent` (transfer) | `delegate` to each, `report` back unless `disallow_transfer_to_parent`, `handoff` to peers unless `disallow_transfer_to_peers` (the supervisor pattern); `communication.mode: closed` |
| `AgentTool` | `delegate` with `mode: sync` to the called agent |
| `output_key`, `{key}` in instructions | an output, and a `shared-memory` channel `session-state` with each agent's `read`/`write` access |
| built-in tools; Python functions | `builtin: google-adk:<name>`; `builtin: google-adk:<import path>` (only an ADK runner can call them) |
| `McpToolset` | an `mcp` tool (command or URL); `tool_filter` becomes `restrictions.tools.allow`; environment values become `{ secret: NAME }` |
| `generate_content_config`, `include_contents` | `preferences.options` |
| agents defined in Python (`code:`), custom agent classes, callbacks, schemas, tool `args` | kept under `extensions.dev.adk`, with a note |
| the user's message | an input without `text` to the root's first agents |

## Microsoft Agent Framework

`madl import maf <file or folder>` builds a document from
[Agent Framework](https://github.com/microsoft/agent-framework) declarative files: a workflow
(`kind: Workflow`, or top-level `actions` as the Python loader accepts), the `kind: Prompt` agents
it invokes by name, or Prompt agents alone. A folder holds at most one workflow; Prompt agents are
matched to the workflow's invocations by `name`. The importer reads the YAML and never runs it; it
lists every part that did not carry over exactly, as for ADK.

| Agent Framework | MADL |
|---|---|
| `InvokeAzureAgent` (`agent.name`) | one agent per name, from the Prompt agent of that name when there is one; `runtime: microsoft-agent-framework` in `defaults` |
| an invocation whose agent is an expression | one agent standing for whichever the expression picks, with a note |
| the order of actions, `ConditionGroup`, `If`, `GotoAction`, `Foreach`, `BreakLoop`, `ContinueLoop`, `End…` | a `handoff` from each agent to every agent the workflow can run next, past the actions between them; `communication.mode: closed`. A branch's Power Fx condition is kept under the connection's `extensions` (`when`), with a note: MADL's `when` is not Power Fx |
| `output.messages`, `output.responseObject` | an output named after the variable |
| `conversationId` shared by several agents; `Local.` variables | a `shared-memory` channel per conversation, and `workflow-variables`, with each agent's access |
| `OnConversationStart`; workflow `inputs` | an input without `text` to the agents the workflow starts with |
| Prompt `name`, `displayName`, `description`, `instructions` (+ `additionalInstructions`) | `id`, `name`, `description`, `instructions` |
| `model.id` with `provider` | `model: <provider>/<id>` (`AzureOpenAI` → `azure-openai`); a model read from the environment (`=Env.X`) is left unset |
| `model.options`, `model.apiType` | `preferences.options` |
| `mcp` tools | an `mcp` tool by URL; `allowedTools` becomes `restrictions.tools.allow`, `approvalMode` `restrictions.approvals.required_for` |
| `function`, `openapi` tools; `web_search`, `file_search`, `code_interpreter` | `builtin: microsoft-agent-framework:<name>`; `…:<kind>` |
| `Question`, `RequestExternalInput`; workflow-level tool and HTTP calls; variables and messages | not drawn, with a note; the agents before and after are linked directly |
| `model.connection`, `outputSchema`, `inputSchema`, tool settings, `maxTurns`, the workflow itself | kept under `extensions.com.microsoft.agent-framework`, without secret values written in the files |
