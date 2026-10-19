# Topology

## The graph

Agents and steps are the nodes; connections are the edges, with groups standing for their members
and `direction: both` counted in both directions. Work moves along `handoff`, `delegate` and
`review` connections.

## Patterns

Patterns are shorthand for common shapes. A reader expands them into connections (and, for
`blackboard`, a channel) before applying the graph rules; `madl expand` prints the result.

```yaml
topology:
  patterns:
    - { id: assign, pattern: supervisor, supervisor: coordinator, members: [searcher, writer], routing: dynamic }
    - { pattern: debate, members: [for-rust, for-go], judge: judge, max_rounds: 3 }
  loops: { max_rounds: 3 }
```

| Pattern | Roles | Expands to |
|---|---|---|
| `pipeline` | `members` (2+) | `handoff` from each member to the next |
| `supervisor` | `supervisor`, `members` | `delegate` (`routing: dynamic`, the default) or `handoff` (`static`) to each member; `report` back |
| `auction` | `auctioneer`, `members` | `delegate` to each member; `report` back |
| `committee` | `source`, `members`, optional `collector` | `handoff` from the source to each member; `report` to the collector (default: the source) |
| `debate` | `members` (2+), optional `judge` | `review` both ways between every two members with `max_rounds`; `report` to the judge |
| `role-play` | exactly two `members` | `ask` both ways with `max_rounds` |
| `swarm` | `members` (2+) | `handoff` both ways between every two members with `max_rounds` |
| `blackboard` | `members` | a `blackboard` channel the members read and write (`via` names an existing one) |

A pattern missing a role it needs is an error (`pattern-shape`). Expanded connections take the
pattern's `via` and `protocol`, carry `pattern: <pattern id>`, and have ids
`<pattern id>-<from>-<kind>-<to>`. A connection the author already declared with the same `from`,
`to` and `kind` is kept, and not repeated.

## Starting points

The starting agents are the agents an input names, and every agent that no `handoff` or `delegate`
reaches. People (`runtime: human`) start only when an input names them. A document with agents
other than people, but no starting agent, is an error (`no-starting-point`).

## Loops

A cycle of `handoff`, `delegate` or `review` connections is allowed when work can reach it from a
starting agent (along `handoff`, `delegate`, `review` or `report`). One piece of work goes round at
most the smallest `max_rounds` on the cycle, else `topology.loops.max_rounds`, else the runner's
own limit. A cycle that no work can reach has no way in, and is an error (`loop-without-start`).

A step that no work reaches is a warning (`unreachable`).

## Workflows

A workflow passes items through stages ([MEP 0007](../../meps/0007-workflows.md)). Its `dispatcher` splits
the workflow's `input` into items and hands each to the `entry` stage; every stage reports an outcome,
and the stage's `routes` say where that outcome takes the item next. A route to `null` completes the
item, and the `collector` assembles the completed items. The outcome `blocked` always stops an item.

```yaml
workflows:
  - id: chapters
    input: topic
    dispatcher: editor
    entry: draft
    collector: editor
    max_items: 12            # absent: the runner's own limits
    max_steps: 20            # per item
    copies: per-item         # each item in its own git copy; default shared
    stages:
      - id: draft
        agent: writer
        workers: [writer-2]                    # more agents that take items here
        routes: { done: review }
        max_rounds: 3
        on_limit: sign-off                     # where an item goes past max_rounds; absent: blocked
        escalate: { after_rounds: 2, to: senior }
      - id: review
        reviewers: [critic, fact-checker]      # a panel: every reviewer takes the same item
        personas: { critic: simplifier }
        join: all                              # or any: the first approval decides
        priority: [blocked, changes, approved] # worst first
        routes: { approved: verify, changes: draft }
      - { id: verify, kind: check, command: make check, routes: { passed: sign-off, failed: draft } }
      - { id: sign-off, kind: human, prompt: Publish this chapter?, routes: { "yes": merge, "no": draft } }
      - { id: merge, kind: merge, into: main, routes: { merged: null } }
```

| `kind` | Requires | May have |
|---|---|---|
| `agent` (the default) | `agent`, or `reviewers` (at least two) | `workers`, `escalate`; for a panel `personas`, `join`, `priority` |
| `check` | `command`; its outcomes are `passed` and `failed` | `timeout_s` |
| `human` | `prompt`; its outcomes are the route keys | |
| `merge` | | `into` (default: the current branch) |

Every stage may have `name`, `max_rounds` and `on_limit`. A stage without what its kind needs, or
with what another kind uses, is an error (`stage-field`). Stage ids are unique within the workflow
(`duplicate-id`); `entry`, routes and `on_limit` name its stages, and `dispatcher`, `collector` and
every stage's agents name agents (`unknown-reference`). A persona is `red-team`, `adversarial`,
`simplifier`, `user`, `expert`, `good-friend`, `ownership` or the runner's own; a Runner SHOULD tell
the reviewer the angle it takes. `policies.guards` pause new items while they fail.

In the graph, the dispatcher hands work to every agent of the workflow's stages, so they are not
starting agents.

## Conditions

`when` on a connection and `over` on a fan-out step use a small expression language, a subset of
[CEL](https://cel.dev): field access (`review.outcome`), literals, `==`, `!=`, `<`, `<=`, `>`,
`>=`, `&&`, `||`, `!` and `in`. A field that is not there is false, never an error. What fields
exist (`outcome`, an agent's outputs, an input's members) is up to the runner, which SHOULD
document them.

## Policies

```yaml
policies:
  on_failure: { strategy: retry, max_attempts: 2, then: escalate }
  escalation: [ { from: cad, after_rounds: 3, to: senior-cad } ]
  schedules: [ { id: nightly, agent: reviewer, every: 24h, message: "Review today's changes." } ]
  guards: [ { command: "make health", every: 10m, timeout_s: 600 } ]
```

- `on_failure.strategy` and `then`: `retry`, `replan` (plan the remaining work again),
  `decompose` (split the failed work into smaller pieces), `escalate`, `stop`.
- `escalation` hands work from an agent or step to a stronger agent after a number of rounds.
- `schedules` send an agent a message at an interval.
- `guards` are commands that pause new work while they fail.
