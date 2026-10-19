# MEP 0008: Runner settings

- **Status:** Accepted
- **Issue:** none (proposed with the reference implementation)
- **Revision:** draft

## Problem

Some choices say how a runner treats an agent rather than what the agent is, yet they are part of
how a system is designed, and a team moved to another runner should keep them:

1. **What reaches an agent's context.** Refusing whole reads of large files, capping a tool call's
   output, and continuing in a fresh context past a size keep long runs affordable.
2. **Which of its runtime's own configuration an agent loads**: the person's (such as `~/.claude`)
   and the project's (such as `.claude/`).
3. **Which tokens count toward a token limit**: all of them, all but cache reads, or output only.
4. **Pausing at a plan share** instead of stopping, and continuing when the window resets.

## Proposal

```yaml
system:
  budget:
    plan: [ { provider: claude, window: five_hour, hard: 30, pause: { continue_with: resume, max_windows: 2 } } ]
defaults:
  preferences:
    context: { read_max_kb: 200, tool_output_chars: 4000, refresh_at_tokens: 80000 }
    config: { user: false, project: true }
agents:
  - id: parser
    settings: { restrictions: { budget: { tokens: 2000000, counts: output } } }
  - id: free
    settings: { preferences: { context: { managed: false } } }
```

- **`preferences.context`**: `read_max_kb`, `tool_output_chars`, `refresh_at_tokens`, and `managed`
  (`false`: none of these limits, nor the runner's own). Merged key by key.
- **`preferences.config`**: `user` and `project`, booleans. Merged key by key.
- **`restrictions.budget.counts`**: `all` (the default), `new` or `output`. `all` is the tightest;
  counting fewer tokens than the block that set an outer token limit is loosening.
- **`plan[].pause`**: `continue_with` (`notes`, the default, or `resume`) and `max_windows` (default 3,
  at least 1).

A runner that cannot apply a runner setting MAY ignore it, and SHOULD say so. Absent members are
the runner's own choice.

## Validation

New code `budget-no-windows`. `restriction-loosened` covers `counts`.

## Compatibility

Additive: every existing document stays valid and means the same.

## Conformance

`valid/runner-settings.yaml`; `invalid/pause-no-windows`, `loosen-token-counts`.

## Alternatives considered

- **Keep them in each runner's extension.** A team moved between runners would lose them, and each
  runner would invent its own names for the same ideas.
- **`options`.** It is per runtime and free-form; these mean the same on every runtime.
