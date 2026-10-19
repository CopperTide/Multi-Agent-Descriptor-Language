# Settings

## Settings blocks

The same shape appears at the root (`defaults`), on a group (`settings`), on an agent
(`settings`), and for what an agent spawns (`subagents.settings`):

```yaml
settings:
  preferences:            # overridable choices: the nearest value wins
    runtime: claude-code
    model: anthropic/claude-sonnet-5-5
    models: { codex: openai/gpt-6-sol }   # a model per runtime
    workspace: { folder: ./project, copy: own }
    protocol: a2a@1.2
    resume: resume-if-possible
    language: en
    questions: { timeout: 30m, on_timeout: continue }
    options: { effort: high }
    context: { read_max_kb: 100, tool_output_chars: 8000, refresh_at_tokens: 60000 }
    config: { user: false, project: true }
  restrictions:           # limits: they only tighten
    tools:
      allow: [fs, search]
      deny: [shell]
      refuse: [ { name: no-force-push, tool: Bash, conditions: ["contains(parameters, 'push --force')"] } ]
    files: { read: ["."], write: ["./out"] }
    network: { egress: [pypi.org, github.com] }
    budget: { tokens: 2000000, counts: new, cost_usd: 20, wall_clock: 2h, scope: shared }
    spawn: { allowed: true, max_children: 4, max_depth: 2 }
    approvals: { required_for: [shell, network, write-outside-workspace, runtime-permissions] }
    communication: { mode: closed, deny: [ { from: cad, to: customer } ] }
  applies_to: { preferences: self, restrictions: descendants }
```

Every member is optional. An absent value is inherited. A document with no settings at all has no
limits: MADL sets no limit the author did not set.

### Preferences

| Member | |
|---|---|
| `runtime` | See [Agents](document.md#agents) |
| `model` | `provider/model`, or `{ id, fallback: [ … ] }` tried in order |
| `models` | A model for each runtime, by runtime name (`unknown-runtime`): an agent uses its runtime's entry unless a nearer block sets `model` ([MEP 0006](../../meps/0006-shared-definitions.md)) |
| `workspace` | The agents' `folder` and `copy`; each member an agent's own `workspace` leaves out comes from here (MEP 0006) |
| `protocol` | The default [protocol](communication.md#protocols) |
| `resume` | `resume-if-possible`, `relaunch-fresh`, `checkpoint-only` |
| `briefing` | `full` (default): the agent is told the system's shape, its neighbours' jobs and the working rules; `minimal`: only its own instructions, contracts and how to finish |
| `language` | A BCP 47 tag |
| `questions` | `timeout` (a duration) and `on_timeout`: `continue`, `wait`, `stop` |
| `options` | Runtime-specific options, merged key by key |
| `context` | How the runner keeps what reaches each agent's context small; see [Runner settings](#runner-settings) |
| `config` | Which of its runtime's own configuration an agent loads; see [Runner settings](#runner-settings) |

Durations are written `90s`, `30m`, `2h`, `1d` or combined, as in `1h30m` (`bad-duration`).

### Restrictions

| Member | Meaning | Combines down the chain by | Loosening |
|---|---|---|---|
| `tools.allow` | Only these tools (absent: every tool the agent is given) | intersection | adding a tool an outer block does not allow, or one it denies |
| `tools.deny` | Never these tools | union | (cannot happen) |
| `tools.refuse` | Rules that refuse single calls (see below) | union | (cannot happen) |
| `files.read`, `files.write` | Paths (and everything under them) the agent may read or write | intersection of path sets | a path outside every outer path |
| `network.egress` | `deny`, a list of hosts, or `allow` (ordered tightest first) | the tighter; host lists intersect | a looser rule, or a host an outer list lacks |
| `budget.tokens`, `cost_usd`, `wall_clock`, `turns`, `working_time` | [Limits](#limits) | minimum (hard and soft values each) | a larger hard value |
| `budget.scope` | `shared`: the agent and what it spawns draw from one budget; `own`: each has its own | `shared` wins | `own` under `shared` |
| `budget.counts` | Which tokens count toward `tokens`: `all` (input, output, cache reads and writes; the default), `new` (all but cache reads), `output` | the tighter (`all`, then `new`, then `output`) | counting fewer tokens than the block that set an outer token limit |
| `spawn.allowed` | Whether the agent may start subagents | AND | `true` under `false` |
| `spawn.max_children`, `max_depth` | How many, and how many levels below the agent | minimum | a larger value |
| `approvals.required_for` | Actions that need a person's approval first: `shell`, `network`, `write-outside-workspace`, `runtime-permissions` (whatever the agent's runtime asks permission for), or a tool name. An empty list says that nothing does, in place of the runner's own default | union | an empty list under one that is not |
| `communication` | [Mode and bans](communication.md#open-and-closed-communication) | `closed` wins; bans add up | `open` under `closed` |

Tools are named by tool id (`fs`), `<tool id>/<tool name>` for one MCP tool, or
`<runtime>:<name>` for a runtime's own tool. Effective allowed tools are the allow list minus the
deny list.

A rule in `tools.refuse` (`{ name, tool, conditions, reason }`) refuses a call to a tool whose name
the regular expression `tool` matches (absent: every tool) when every condition holds; the agent is
told `reason`. Conditions are written in [Signet-Eval](https://github.com/wandercom/signet-eval)'s
condition language: `contains`, `any_of`, `matches`, `param_contains`, `param_eq`, `param_ne`,
`not`, `or`, `true` and `false`, over the call's `parameters` and named parameters such as
`file_path`. A Runner that cannot read a condition MUST say so, and SHOULD NOT let the rule block
calls it does not understand.

### Limits

A limit is a number or duration (its hard value) or `{ soft, hard }`. At the soft value a runtime
SHOULD start no new work for the agent and ask it to finish with a summary; at the hard value it MUST
stop the agent and what it started. Without `soft` a runtime may choose one. `soft` above `hard` is
`budget-soft-above-hard`. `turns` counts the agent's turns in one run; `working_time` is time spent
in turns. A budget for the whole run is [`system.budget`](document.md#run-budget).

### Runner settings

Two preferences say how a runner treats an agent rather than what the agent is
([MEP 0008](../../meps/0008-runner-settings.md)). A runner that cannot apply one MAY ignore it, and
SHOULD say so.

| Member | |
|---|---|
| `context.read_max_kb` | A read of a whole file larger than this many KB is refused; the agent reads a part instead |
| `context.tool_output_chars` | The most output of one tool call, in characters, that reaches the agent |
| `context.refresh_at_tokens` | Past this many tokens of context, the agent writes notes and continues in a fresh context |
| `context.managed` | `false`: the runner leaves the agent's context alone, with none of these limits or its own |
| `config.user` | Whether the agent loads the configuration of the person whose machine runs it, such as `~/.claude` or `~/.codex` |
| `config.project` | Whether it loads the configuration in its folder, such as `.claude/` |

Absent members are the runner's own choice.

A block that tries to loosen an inherited restriction is an error (`restriction-loosened`), reported
at the block's value. Resolution keeps the tighter value regardless. A runner meeting a loosening
at run time MUST keep the tighter value and SHOULD record that it did.

## Reach

`applies_to` sets how far a block's preferences and restrictions reach:

| Value | Reaches |
|---|---|
| `self` | The agents the block covers, not what they spawn |
| `subagents` | Only what those agents spawn, and what that spawns |
| `descendants` | The agents and everything they spawn |

Defaults: preferences `self`; restrictions `descendants`. So a restriction cannot be escaped by
spawning, and a preference does not leak into helpers unless the author says so.

## Subagents

```yaml
agents:
  - id: coordinator
    subagents:
      template:                     # what a spawned agent starts as
        instructions: You are a focused helper for one subtask.
        runtime: claude-code
        model: anthropic/claude-haiku-5-5
        tools: [ … ]
      settings:                     # settings for spawned agents only
        restrictions: { tools: { allow: [search] } }
```

A spawned agent runs on its parent's runtime unless its template or settings name another.

## Resolution

The effective settings of agent *A* are computed by applying these blocks in order, each only
where its reach covers *A* itself (`self` or `descendants`):

1. `defaults`
2. the root `communication`, as a restriction
3. *A*'s groups, each from its outermost enclosing group inwards; for groups that are not nested,
   in the order *A* lists them; a group applies once
4. *A*'s own `settings`
5. *A*'s shorthand (`runtime`, `model`, `resume`)

The effective settings of what *A* spawns are computed by applying the same blocks, each only
where its reach covers subagents (`subagents` or `descendants`), then *A*'s `subagents.settings`
and the template's `runtime` and `model`. Then: if no runtime is set, *A*'s runtime is used, and
`spawn.max_depth` is one less (at 0, `spawn.allowed` is `false`).

Preferences merge as follows: a scalar or list replaces the inherited value; `options` merges key
by key, recursively; `models`, `workspace`, `context` and `config` merge key by key. Then, if `models`
has an entry for the agent's runtime and no block after the one that set that entry sets `model`, the
entry is the agent's `model`. Restrictions combine as in the table above.

A Resolver reports, for every effective value, the JSON Pointer of the block value that set it
last. `conformance/resolve/` pins this output.
