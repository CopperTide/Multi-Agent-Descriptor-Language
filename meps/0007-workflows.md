# MEP 0007: Workflows

- **Status:** Accepted
- **Issue:** none (proposed with the reference implementation)
- **Revision:** draft

## Problem

Many systems split their input into items and pass each through the same stages: draft, review,
check, a person's sign-off, merge. Which stage an item goes to next depends on the outcome of the
last one, and an item may loop back. MADL could approximate this with steps and handoffs whose
`when` names an outcome, but it could not say which stages belong together, who splits the input
and who assembles the result, that a review panel decides together, where an item goes when it
has looped too often, how many items one run may make, or how many steps one item may take. Sculptured kept its workflows
in its own extension, which a reader that ignores extensions could not see.

## Proposal

A top-level `workflows` list. Each workflow has an `id`, a `name`, the `input` it splits, a
`dispatcher`, an `entry` stage, a `collector`, `max_items`, `max_steps` (per item), `copies` (`shared` or
`per-item`) and `stages`. In place of a `dispatcher` agent, a workflow may name a `dispatch` command
(`command`, `timeout_s`): the Runner runs it in the workspace when the system starts, and its standard
output is the items. A workflow names exactly one of the two.

```yaml
workflows:
  - id: chapters
    input: topic
    dispatcher: editor
    entry: draft
    collector: editor
    stages:
      - { id: draft, agent: writer, workers: [writer-2], routes: { done: review }, max_rounds: 3, on_limit: sign-off, escalate: { after_rounds: 2, to: senior } }
      - { id: review, reviewers: [critic, fact-checker], personas: { critic: simplifier }, join: all, priority: [blocked, changes, approved], routes: { approved: verify, changes: draft } }
      - { id: verify, kind: check, command: make check, routes: { passed: sign-off, failed: draft } }
      - { id: sign-off, kind: human, prompt: Publish this chapter?, routes: { "yes": merge, "no": draft } }
      - { id: merge, kind: merge, into: main, routes: { merged: null } }
layout:
  stages: { chapters: { draft: { x: 300, y: 40 } } }
```

- A stage's `kind` is `agent` (the default), `check`, `human` or `merge`. An agent stage has an
  `agent` (with optional `workers`) or a review panel of at least two `reviewers` (with `personas`,
  `join` and `priority`). Every stage has `routes` from outcome to the next stage, or `null` to
  complete the item; the outcome `blocked` always stops it.
- `max_rounds` limits how often one item enters a stage; past it the item goes to `on_limit`, or is
  blocked. `escalate` hands items to a stronger agent after a number of rounds.
- Stage ids are unique within their workflow, so stages may share ids with agents.
- `policies.guards` pause new items while they fail. `layout.stages` holds stage positions.
- In the graph, the dispatcher hands work to every agent of the workflow's stages, so they are not
  starting agents. `madl graph` draws each workflow as a box of its stages.

## Validation

New codes `stage-field` (a stage without what its kind needs, or with what another kind uses) and
`workflow-dispatch` (a workflow with both a `dispatcher` and a `dispatch` command, neither, or an empty
command).
`duplicate-id`, `bad-id` and `unknown-reference` cover stage ids, routes, `entry`, `on_limit`, the
workflow's agents and its input.

## Compatibility

Additive: every existing document stays valid and means the same.

## Conformance

`valid/workflow.yaml`, `examples/book.madl.json`; `invalid/stage-without-agent`,
`stage-stray-field`, `panel-of-one`, `route-to-unknown-stage`, `duplicate-stage`,
`stage-unknown-agent`.

## Alternatives considered

- **Stages as steps, routes as connections with `when`.** It loses which stages form one workflow,
  the dispatcher and collector, panels, `on_limit` and the run's limits, and stage ids would have to
  be unique across the whole document.
- **A `workflow` pattern that expands into connections.** Same loss, and patterns expand to plain
  connections by design.
