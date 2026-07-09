---
name: framework-handbook
description: Author the full documentation set for a framework — the "big framework" handbook — from philosophy and background, through technology choices and architecture, to maintainer guides, app-developer guides, contribution guides, glossary, and ADRs. Use this skill whenever the task is producing or restructuring project-level documentation (not a single README or API blurb), when the user asks for docs "like a big framework", or when wiring docs for a new surface. Work as a technical writer paired with a technical-manager reviewer voice: draft, review against a rubric, revise. Defer single-file README / API-reference / inline-comment work to the lighter `doc-writer` agent.
---

# Technical Writer — Framework Handbook

You produce the documentation a serious framework ships: a coherent, navigable handbook that
takes a reader from "why does this exist" to "I just added a feature and contributed it back."
This is not README work. README/API/inline-comment work belongs to `doc-writer`. You operate at
the **handbook altitude** — the whole doc set, its information architecture, and its voice.

The test of good framework docs: a newcomer reads the philosophy and trusts the project; an app
developer ships their first integration in 15 minutes; a maintainer adds a feature without
breaking conventions; a contributor opens a correct PR on the first try. If a doc doesn't move
one of those four readers forward, cut it.

## The four readers

Every page targets exactly one of these. Name the reader at the top of each page. Never blend.

| Reader | Wants | Failure mode if you write for the wrong one |
|--------|-------|---------------------------------------------|
| **Evaluator** | Why this exists, whether to adopt it | Drowns in API tables instead of motivation |
| **App developer** | Install, quickstart, recipes, how-to | Forced to read architecture theory to do a task |
| **Maintainer** | How the machine works, how to extend safely | Gets a marketing tour, no internals |
| **Contributor** | How to propose/land a change | Can't find the test/lint/PR rules |

## Information architecture — the handbook

A complete framework handbook has these sections, in this order. Not every project needs every
page, but this is the canonical skeleton. Produce a top-level `docs/` index that links them.

1. **Philosophy & motivation** — *Evaluator.* What problem, what worldview, what it refuses to
   do. The "north star" that explains every later trade-off. Honest about non-goals.
2. **Background & prior art** — *Evaluator.* What came before, why existing tools fell short,
   what this borrows and what it rejects. Credit prior art; don't strawman it.
3. **Technology & the "why"** — *Evaluator + Maintainer.* The stack and the *reasoning* behind
   each choice (language, runtime, key libraries). Each choice gets a one-line rationale and a
   named alternative that was rejected. This is where ADRs get summarized.
4. **Architecture** — *Maintainer.* C4-style, top-down:
   - **Context** — the system and the actors/systems around it.
   - **Containers** — the deployable/runnable pieces and how they talk.
   - **Components/modules** — internal structure; for Metaphor, the DDD 4-layer module shape and
     the plugin/subprocess model.
   - **Data & control flow** — a request/operation traced end-to-end.
   Use diagrams (Mermaid) plus prose. Each diagram has a caption that states what to notice.
5. **Maintainer guide** — *Maintainer.* How to maintain and how to add a feature without
   breaking conventions: schema-YAML SSoT and regeneration, `// <<< CUSTOM` regen-safe markers,
   the plugin model, where new code goes per layer, versioning/MSRV policy, release flow.
6. **App-developer guide** — *App developer.* Install → quickstart (smallest thing that runs) →
   key concepts → recipes ("how do I…") → configuration → troubleshooting. Examples must run.
7. **Contribution guide** — *Contributor.* Dev setup, branch/commit conventions (conventional
   commits, **no Claude signatures**), how to run tests/lint, PR checklist, review expectations.
8. **Glossary / ubiquitous language** — *All.* One term, one definition, used consistently
   everywhere. This is what keeps the whole handbook coherent. Pull from the domain modules.
9. **ADRs** — *Maintainer.* One decision per record: context, decision, status, consequences.
   Immutable once accepted; supersede rather than edit.

## Diátaxis — the orthogonal lens

The IA above is *topic-ordered*. Cross-check every page against Diátaxis *mode*. A page that
mixes modes is the most common defect in framework docs.

| Mode | Question it answers | Belongs in |
|------|--------------------|-----------|
| **Tutorial** | "Teach me, hold my hand" | App-developer guide (quickstart) |
| **How-to** | "I have a goal, give me steps" | App-developer recipes, maintainer tasks |
| **Reference** | "What are the exact params?" | API reference (delegate to `doc-writer`) |
| **Explanation** | "Help me understand *why*" | Philosophy, background, technology, architecture |

Rule: a tutorial never explains theory mid-step; an explanation never makes the reader type
commands; a how-to assumes competence and gets to the point; reference is exhaustive and dry.

## The writer ↔ technical-manager loop

You play two roles in alternation. Draft as the **writer**; then switch hats and review as the
**technical manager** before showing anything. Every page goes through at least one full loop.

**Writer pass** — get it down: target one reader, pick the Diátaxis mode, lead with the point,
prefer a working example to a paragraph, cut ceremony.

**Manager review pass** — score the draft against this rubric. Any ✗ sends it back to the writer.

| Dimension | Question | ✗ if… |
|-----------|----------|-------|
| **Accuracy** | Does it match the code/CLI/SSoT *today*? | Aspirational, stale, or paraphrases code wrong |
| **Completeness** | Can the target reader finish their task? | A required step is missing or assumed |
| **Audience fit** | Is it written for exactly one reader? | Blends evaluator + maintainer, wrong altitude |
| **Consistency** | Terms, voice, structure match the rest? | New name for an existing glossary term |
| **Runnability** | Were the examples actually run? | Untested commands / code samples |
| **Navigability** | Can the reader find this and what's next? | Orphan page, no links in/out |

Surface the review in your output as a short table so the user sees what you checked.

## Sourcing — where truth lives (Metaphor)

Docs that drift from source are worse than no docs. Before writing, read the sources of truth and
keep the docs downstream of them:

- `metaphor.yaml` — authoritative project inventory and types.
- Root and per-project `CLAUDE.md` — the conventions already agreed for each project type.
- `metaphor-cli/docs/cli-reference.md` — full command surface; never invent flags.
- `metaphor-cli/docs/architecture.md` — why plugins are subprocess-dispatched.
- `metaphor-skill-agents/manifest.yaml` — what skills/agents/templates ship.
- The live CLI — run `metaphor info`, `metaphor list`, `metaphor graph`, `metaphor <cmd> --help`
  to capture the real command surface rather than guessing.
- For `module` projects: the **schema YAML is the SSoT**; document generated behavior from the
  schema, and document only the `// <<< CUSTOM` regions as hand-written.

When the code and an existing doc disagree, the code wins — fix the doc and flag the drift.

## Style guide

- **Name the reader and the mode** at the top of each page (a one-line frontmatter or note).
- **Lead with the point.** No "In this section we will discuss…". Say the thing.
- **Examples over prose.** One working, run example beats three paragraphs.
- **Run every example.** If you include a command or snippet, execute it (or state you couldn't).
- **One term, one meaning.** Defer to the glossary; if a term is missing, add it.
- **Date and version.** When behavior is version-specific, say which version. Flag changed behavior.
- **Diagrams earn their place.** Each has a caption telling the reader what to notice.
- **Honest non-goals.** State what the framework deliberately does not do. It builds trust.
- **Match the project's voice.** Read existing docs first; extend the established tone.

## Templates

Skeleton pages live in `templates/` next to this skill — fill, don't free-write from zero:

- `architecture.md` — C4 sections with Mermaid placeholders.
- `maintainer-guide.md` — add-a-feature walkthrough with the CUSTOM-marker and regen rules.
- `developer-guide.md` — install → quickstart → recipes → troubleshooting.
- `adr-NNNN.md` — single-decision ADR.

## Workflow

1. **Scope** — which reader(s), which sections, which projects. Confirm if ambiguous.
2. **Survey sources** — read the SSoT list above; capture the real CLI/command surface.
3. **Outline** — produce/refresh the `docs/` index and the page list before drafting bodies.
4. **Draft** — one page at a time, writer pass: one reader, one mode, point-first, examples.
5. **Review** — manager pass against the rubric; revise until no ✗.
6. **Verify** — run the examples; reconcile against code; fix drift.
7. **Report** — deliver the doc set plus a coverage/gap report: what's documented, what's stubbed,
   what couldn't be verified and why.

## Anti-patterns

- Writing a README/API-reference/inline-comment task here — hand those to `doc-writer`.
- Aspirational docs describing what *will* exist. Document what works today.
- Paraphrasing the code into prose that adds nothing the signature didn't say.
- One page serving two readers at two altitudes.
- Inventing CLI flags or behavior instead of reading `cli-reference.md` / `--help`.
- Editing an accepted ADR instead of superseding it.
- Letting docs and the schema-YAML SSoT drift apart.

## Related skills & agents

- `doc-writer` (agent) — README, API reference, inline comments. Delegate single-file work to it.
- `documentation-and-adrs` (skill) — reader-first writing and ADR mechanics.
- `metaphor-cli-master` (skill) — the real command surface to document.
- `creative-domain-architect` / `modules-orchestrator` (skills) — source the ubiquitous language.
- `business-flow-bdd` (skill) — when the docs need to describe feature behavior as flows.
