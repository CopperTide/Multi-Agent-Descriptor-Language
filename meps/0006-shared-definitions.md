# MEP 0006: Shared tools, a model per runtime, workspace defaults, shared values, call rules and a decision log

- **Status:** Accepted
- **Issue:** none (proposed with the reference implementation)
- **Revision:** draft

## Problem

Sculptured wanted to keep its teams as MADL documents, with nothing that describes a team left in
its own extension. Several things a team says could not be written in core MADL:

1. **Tools shared by many agents.** Tools were defined on each agent, so a server used by six agents
   was written six times. Hooks on tool calls, plugins and packs of skills had no form at all, nor
   did a tool's setup steps, the runtimes it works with, or the tools it needs.
2. **A default model for each runtime.** A team with Claude Code and Codex agents wants a default
   for each; `model` is one value for every runtime.
3. **A default workspace.** A folder and an own-git-copy setting for every agent had to be written on
   each agent.
4. **Values agents rely on**, such as a port or a field name another agent's code depends on, with
   the file that must hold them.
5. **Rules that refuse single calls**, such as "never `git push --force`". `tools.deny` refuses a
   whole tool.
6. **"The runtime's own permission prompts go to a person"**, and "nothing needs approval", which
   overrides a runner's default.
7. **The most agents at once**, spawned ones included. `max_working` counts only working agents.
8. **A decision log** people keep and agents read.
9. **`$schema`**, so editors validate and complete `.madl.json` files.

## Proposal

```yaml
$schema: https://raw.githubusercontent.com/Alexander-Aghili/Multi-Agent-Descriptor-Language/main/schema/draft/schema.json
madl: draft
system: { id: wing-team, name: Wing team, budget: { max_agents: 6 } }
defaults:
  preferences:
    models: { claude-code: anthropic/claude-sonnet-5-5, codex: openai/gpt-6-sol }
    workspace: { folder: ./project, copy: own }
  restrictions:
    approvals: { required_for: [runtime-permissions] }
    tools:
      refuse: [ { name: no-force-push, tool: Bash, conditions: ["contains(parameters, 'push --force')"] } ]
tools:
  - { id: notes, mcp: { command: notes-mcp }, install: cargo install notes-mcp }
  - { id: money, skills: [ { name: money-rules, description: Use when handling money., body: Decimal only. } ] }
channels: [ { id: decisions, medium: decision-log } ]
agents:
  - id: wing
    tools: [notes, money]
    data_access: { shared: [ { name: spar count, value: SPARS = 2, file: cad/wings/params.py } ] }
```

- **Tools.** A top-level `tools` list; an agent's `tools` entry is a tool or a tool id. A tool gains
  `hook` (`{ command, args }`, called before each tool call; `{call_rules}` in an argument is the
  path of the agent's call rules), `plugin` (`name@marketplace`) and `skills` (embedded), beside
  `mcp` and `builtin`; at most one of them (`tool-shape`). Any tool may have `name`,
  `instructions` (what agents given it are told), `runtimes`, `requires`, `install` and `homepage`.
  An MCP server gains `headers`; `${NAME}` in `url` or `headers` is a secret the runner fills in.
- **`preferences.models`**: a model per runtime, merged key by key. An agent's runtime's entry is its
  model unless a block after the one that set the entry sets `model`.
- **`preferences.workspace`**: `{ folder, copy }` for agents; each member an agent's own `workspace`
  leaves out comes from it. Merged key by key.
- **`data_access.shared`**: `{ name, value, file }`; a Runner SHOULD check at each start that the file
  holds the value.
- **`restrictions.tools.refuse`**: `{ name, tool, conditions, reason }`, in Signet-Eval's condition
  language; rules add up down the chain.
- **`approvals.required_for`**: the action `runtime-permissions`; an empty list says nothing needs
  approval, in place of the runner's default. An empty list under a non-empty one is loosening.
- **`system.budget.max_agents`**, at least 1 (`budget-no-room`).
- **The medium `decision-log`**: decisions people record; every agent reads it unless `access` says
  otherwise.
- **`$schema`** at the top level, with no meaning.

## Validation

New code `tool-shape`. `unknown-runtime` also covers `models` keys and a tool's `runtimes`;
`budget-no-room` covers `max_agents`; `restriction-loosened` covers an empty approvals list under a
non-empty one.

## Compatibility

Additive for documents: every existing document stays valid and means the same. For the Rust and
TypeScript types, `Agent.tools` holds `ToolRef` (a tool or an id) and `Approvals.required_for` is
optional, so code that builds these values changes.

## Conformance

`valid/shared-tools.yaml`; `invalid/tool-two-kinds`, `max-agents-zero`, `loosen-approvals`,
`unknown-runtime-model`; `resolve/models.yaml`.

## Alternatives considered

- **Tools by reference only, defined by the runner.** Documents would not carry what a tool is, and
  another runner could not set it up.
- **A team-wide `model` per agent kind as an extension.** It is how people describe mixed teams, so
  it belongs in the language.
