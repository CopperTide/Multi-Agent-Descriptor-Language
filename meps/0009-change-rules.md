# MEP 0009: Change rules

- **Status:** Accepted
- **Issue:** none (proposed with the reference implementation)
- **Revision:** draft

## Problem

Agents increasingly change systems outside the run: they place an order, reschedule a work order,
update a record in an ERP. Who may change what, and within which limits, is part of how a system is
designed, and a team moved to another runner should keep it. Today a document can say which tools
need a person's approval (`restrictions.approvals`), but not what a *change* must meet: an order up
to an amount, from approved suppliers only, with a reason. Instructions in the prompt are no answer,
because an agent can ignore or be talked out of them.

## Proposal

```yaml
policies:
  changes:
    approve_within_rules: true
    rules:
      - { id: limit, action: create_purchase_order, field: amount, at_most: 5000 }
      - { id: vendors, action: create_purchase_order, field: supplier, one_of: [Acme Hydraulics, Bolt & Co] }
      - { id: why, field: reason, present: true }
```

- **A change** is what an agent proposes before making it: a kind (`action`, such as
  `create_purchase_order`) and its fields as they would be written. How an agent proposes one is
  the runner's (Sculptured gives agents a `propose_change` tool).
- **`rules[]`**: `id`; `action` (`*`, the default, is every change; compared ignoring case);
  `field`, a dotted path into the change's fields where a number indexes a list (`lines.0.item`);
  and exactly one test:
  - `at_most` / `at_least`: a number. A number written as text counts, ignoring `$ € £` and `,`.
  - `one_of`: a list of text, compared ignoring case and surrounding spaces.
  - `present: true`: there and not empty.

  A missing field fails its rule.
- **`approve_within_rules`** (default `false`): a change that passes every rule that applies to it
  is approved without a person. A change no rule applies to always waits for one: no rule is no
  evidence.
- **The runner evaluates the rules itself**, never the agent, and records each result. A person
  not answering in time means not approved. An agent cannot change the rules.

## Validation

New code `change-rule-shape`: a rule with no test or more than one. A rule id used twice is
`duplicate-id`.

## Compatibility

Additive: every existing document stays valid and means the same.

## Conformance

`valid/change-rules.yaml`; `invalid/change-rule-two-tests`, `change-rule-no-test`.

## Alternatives considered

- **`restrictions.approvals` on a tool.** It says *whether* a call needs a person, not what the
  change must meet; every order would wait, including a $40 one from an approved supplier.
- **Rules as conditions in the expression language.** More general, but a person setting limits for
  a team should not have to write expressions, and four tests cover spending limits, approved lists
  and required fields. More tests can be added the same way.
- **Who approves.** Naming the person or role who approves needs identities MADL does not have yet;
  a later MEP.
