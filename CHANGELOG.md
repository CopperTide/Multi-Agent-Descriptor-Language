# Changelog

## draft (unreleased)

The first public draft.

- MEP 0009: `policies.changes`: rules a change an agent proposes to an outside system must meet
  (`at_most`, `at_least`, `one_of`, `present` on a field of the change, for one `action` or all),
  and `approve_within_rules`. New code `change-rule-shape`.

- An embedded skill may say where it was copied from: `source`, an `https://` address (such as a
  catalog's `SKILL.md` at a commit). A source that is not one is `skill-shape`.

- MEP 0008: runner settings: `preferences.context` (large reads, tool output, a fresh context past a
  size, or `managed: false`), `preferences.briefing` (`full` or `minimal`), `preferences.config` (the runtime's user and project configuration),
  `restrictions.budget.counts` (`all`, `new`, `output`) and `system.budget.plan[].pause`
  (`continue_with`, `max_windows`). New code `budget-no-windows`.

- MEP 0007: `workflows`: a dispatcher splits an input into items that pass through `agent`, review
  panel, `check`, `human` and `merge` stages along outcome routes, with round limits, `on_limit`,
  escalation, `max_items`, `max_steps` and per-item copies; `layout.stages`; `madl graph` draws them.
  A workflow may name a `dispatch` command instead of a `dispatcher` agent: its output is the items.
  New codes `stage-field` and `workflow-dispatch`.

- MEP 0006: top-level `tools` given to agents by id, with `hook`, `plugin` and `skills` tools,
  `headers`, `instructions`, `runtimes`, `requires`, `install` and `homepage`; `preferences.models`
  (a model per runtime) and `preferences.workspace`; `data_access.shared`;
  `restrictions.tools.refuse` (call rules in Signet-Eval's condition language); the approval
  `runtime-permissions` and an explicit empty list; `system.budget.max_agents`; the medium
  `decision-log`; `$schema`; examples may be `.madl.json`. New code `tool-shape`. In the Rust and
  TypeScript types `Agent.tools` holds `ToolRef` and `Approvals.required_for` is optional.

- `madl import maf`: builds a document from Microsoft Agent Framework declarative files (a workflow,
  the `kind: Prompt` agents it invokes, or Prompt agents alone) without running them: a `handoff`
  from each agent to every agent the workflow can run next, with the branch conditions, variables
  and actions between agents in the notes.

- `madl import adk`: builds a document from a Google ADK agent config tree (`root_agent.yaml` and
  the files it names) without running Python, and lists every part that was approximated or not
  supported, by source file and JSON Pointer.

- MEP 0005: agents take a `description` (how the others see them); an input without `text` is given
  when the run starts; `google-adk` and `microsoft-agent-framework` are known runtimes.

- MEP 0004: `skills` entries may embed the skill itself (`{ name, description, body }`, the Agent
  Skills format), so a document carries its skills. New errors `skill-shape` and `duplicate-skill`.

- MEP 0003: agents may declare `data_access` (reads, writes, side effects, rationale) and `authority`
  (the decisions they own, and with `scope` how far across groups they reach); outputs take `acceptance` checks that decide when they are done. New
  warnings `write-overlap` and `authority-overlap`. Ideas from Pact (github.com/wandercom/pact).

- MEP 0002: limits take `{ soft, hard }` (wrap up, then stop); per-agent budgets gain `turns` and
  `working_time`; `system.budget` sets a budget for the whole run (plan-window shares, tokens, cost,
  wall clock, most agents working at once). New codes `budget-soft-above-hard`,
  `budget-percent-out-of-range`, `budget-no-room`.

- Document: system, defaults, inputs, groups (nesting), agents, steps, channels, connections,
  communication, topology, policies, layout, extensions; `$ref` across files.
- Settings: preferences override, restrictions only tighten; `applies_to` reach (`self`,
  `subagents`, `descendants`); subagent templates and settings; a stated resolution order.
- Communication: protocols (`native`, `a2a@…`, `mcp@…`, `x-…`), channels with mediums and access,
  connections with kind, direction, initiator, mode, contents, round limits and conditions; open or
  closed mode; bans that always win.
- Topology: patterns that expand into connections; starting points; loops allowed when work can
  reach them.
- Reference implementation (`madl` crate and CLI), JSON Schema, TypeScript bindings, examples and
  conformance fixtures.
