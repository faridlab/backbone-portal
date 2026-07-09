---
name: business-flow-bdd
description: Capture every feature's business flow and turn each flow into executable BDD / Gherkin acceptance specs. Use this skill when documenting what a feature is supposed to do in business terms (actors, preconditions, paths, business rules, postconditions), when writing Given/When/Then scenarios, or when wiring those scenarios to acceptance tests. Pairs business documentation with verification: every documented flow becomes a runnable scenario. Defer unit/integration test mechanics to the `test-writer` agent; this skill owns business-level behavior.
---

# Business Flow — Document & Verify Behavior

You capture what each feature *does for the business* and prove it with executable acceptance
specs. Two halves, always together: a **business-flow document** (human-readable behavior) and a
set of **Gherkin scenarios** that encode the same behavior as runnable tests. A flow that is
documented but not tested rots; a test with no documented flow can't be reasoned about by the
business. Produce both.

This is not unit/integration testing — that's `test-writer`'s domain. You work at the
business-acceptance level: the language the product owner would recognize.

## Modeling a business flow

For each feature, capture these elements. Missing any one of them is the most common defect.

| Element | Question |
|---------|----------|
| **Actors** | Who initiates and who participates (user roles, external systems)? |
| **Preconditions** | What must already be true for the flow to start? |
| **Trigger** | The single event that starts the flow. |
| **Main path** | The happy-path steps, in order, in business terms (no UI/impl detail). |
| **Alternate paths** | Valid variations that reach a successful end differently. |
| **Exception paths** | What happens when a business rule blocks the flow. |
| **Business rules / invariants** | Constraints that must hold (limits, permissions, uniqueness). |
| **Postconditions** | What is true after success (and after each failure). |

Write the flow in business language. "The customer submits an order" — not "POST /orders returns
201". The implementation may change; the business flow should not.

## Mapping flows to the architecture

Business flows are not free-floating — they correspond to the system's structure:

- A flow's **main path** maps to a **use case** in the **application layer** of the owning module.
- **Business rules / invariants** live in the **domain layer** (entities, value objects).
- **Actors** correspond to roles in the ubiquitous language.

Use this to find the right module and to keep terminology consistent. Source the ubiquitous
language from the domain — defer to `creative-domain-architect` and `modules-orchestrator`. A flow
should never introduce a term the glossary doesn't have.

## Gherkin / BDD authoring

Each business flow becomes a `.feature` file. Rules that keep scenarios useful:

- **`Feature`** states the business value: *In order to … As a … I want …*.
- **One `Scenario` = one business rule or one path.** Don't pack a flow into a single scenario.
- **`Scenario Outline` + `Examples`** for the same rule across many inputs (boundary tables).
- **Given / When / Then** are strict: Given = preconditions (state), When = the single trigger,
  Then = observable postconditions. One `When` per scenario; if you need two, it's two scenarios.
- **Declarative, not imperative.** `When the customer places an order` — NOT `When the user clicks
  #submit-btn`. UI selectors and HTTP verbs do not belong in scenario text; they belong in step
  definitions. This keeps scenarios stable across implementation changes.
- **Tags** organize and select: `@happy-path`, `@edge`, `@module:orders`, `@wip`, `@manual`.
- **Cover the model:** at minimum one scenario for the main path, one per alternate path, one per
  exception path, and an outline for boundary values of each numeric/limited business rule.

```gherkin
Feature: Place an order
  In order to receive products
  As a customer
  I want to place an order from my cart

  Background:
    Given a customer with a verified account

  @happy-path @module:orders
  Scenario: Order placed with items in stock
    Given the cart contains 2 items that are in stock
    When the customer places the order
    Then the order is confirmed
    And the items are reserved from inventory

  @edge @module:orders
  Scenario Outline: Order rejected when over the per-order limit
    Given the cart contains <count> items
    When the customer places the order
    Then the order is rejected with reason "over limit"

    Examples:
      | count |
      | 101   |
      | 500   |
```

## Wiring scenarios to executable tests

Documented behavior must run. Discover the project's harness before choosing one — don't impose.

- **Discover first.** Look for existing `features/` directories and step definitions. Match the
  framework, layout, and step style already in use. If none exist, ask which BDD framework before
  guessing.
- **Rust projects:** `cucumber` (cucumber-rs) is the default. `.feature` files + a `World` struct
  holding scenario state + async step functions.
- **Step definitions** are the only place implementation detail lives — they translate declarative
  steps into real calls against the application layer (use cases), not the UI where avoidable.
- **Run via the workspace:** `metaphor test` (add `--affected --base=main` in CI). Never call
  `cargo test` from the workspace root.
- **Test at the seam that matches the flow.** Business flows usually drive the application/use-case
  layer; reach for the HTTP/UI layer only when the flow is genuinely about that surface.

## Artifacts & where they live

- `features/<feature>.feature` — Gherkin scenarios, one feature per business capability.
- step-definition files alongside, per the project's convention.
- A **flow catalog** doc (e.g. `docs/business-flows.md`) — index of every flow with: feature name,
  owning module, actors, link to its `.feature` file, and current status (documented / tested /
  passing). This is the at-a-glance map of business coverage.

`templates/` next to this skill provides a `feature.feature` skeleton and a cucumber-rs step
definition stub.

## Workflow

1. **Pick scope** — a feature or a module's features. Read the module's schema/domain and CLAUDE.md.
2. **Model the flow** — fill every element in the table above, in business language.
3. **Write the flow doc** — add/update the entry in the flow catalog.
4. **Author scenarios** — translate each path and business rule into Gherkin; cover happy +
   alternate + exception + boundary outlines.
5. **Wire steps** — discover conventions, implement/connect step definitions against the use-case
   layer.
6. **Run** — `metaphor test`; report pass/fail per scenario.
7. **Reconcile** — if a scenario can't pass because the flow is wrong, fix the flow doc; if the
   code is wrong, surface it (don't silently weaken the scenario).

## Anti-patterns

- Imperative, UI-coupled steps (`click #btn`) in scenario text — push detail into step defs.
- Scenarios that assert implementation details instead of business-observable outcomes.
- Aspirational flows for behavior that doesn't exist yet (tag `@wip` and don't claim coverage).
- One mega-scenario covering a whole flow — split per rule/path.
- Multiple `When`s in a scenario — that's two scenarios.
- Introducing terms not in the ubiquitous language / glossary.
- Running `cargo test` from the workspace root instead of `metaphor test`.
- Weakening a scenario to make it pass when the code is actually wrong.

## Related skills & agents

- `business-flow` (agent) — drives this skill in author + test modes.
- `test-writer` (agent) — unit/integration test mechanics below the business level.
- `spec-driven-development` / `test-driven-development` (skills) — spec-first discipline.
- `creative-domain-architect` / `modules-orchestrator` (skills) — ubiquitous language & module map.
- `framework-handbook` (skill) — when flows feed the framework handbook.
