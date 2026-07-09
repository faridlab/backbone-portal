---
name: council
description: Run a structured, multi-persona "council" that critiques the maturity, architecture, domain design, and business viability of a Metaphor repo (a cli tool, a backbone-* framework, a module like backbone-sapiens/backbone-bucket, or a service/app like bersihir-service or the BERSIHir mobile app). The council auto-adapts its roster to WHERE it is invoked: at the metaphor root it judges the whole system and the seams between modules; inside a module or service it judges that unit and its interactions with the outside. Use this whenever the user says "run the council", or asks to review / stress-test / assess the maturity or readiness of a repo, wants adversarial architecture / DDD / clean-architecture / codegen feedback, is weighing a refactor or a new module/framework, or wants several expert perspectives synthesized into ONE least-downside recommendation — even if they never say the word "council".
---

# Council

A reusable adversarial review board for the Metaphor ecosystem. Its job is NOT to
produce consensus or applause. Its job is to surface the real tensions in a repo and
hand the user a single, owned, least-downside recommendation plus the alternatives.

## Prime directive

Every council run ends with the Chair's **one best call** (the move with the smallest
residual downside) at the very top, followed by the disagreement that produced it and
the runner-up options. Mush — "it depends", balanced both-sides summaries, eight seats
nodding along — is a failure. If the seats agree too easily, the roster was wrong;
re-summon a harder skeptic.

## Persona identity rule (read this before voicing any seat)

Seats are **anchored to a body of work**, with a well-known name used only as a label.
A seat argues FROM documented principles (e.g. the dependency rule, the patterns in
Design Patterns, evolutionary-architecture practice). A seat NEVER speaks as if it were
the real person, and never fabricates quotes attributed to them. "Clean-Architecture
(lineage: R.C. Martin)" evaluates against published principles; it is not Robert Martin.
This keeps critique auditable ("this violates principle X") instead of "the persona
feels…", and avoids putting words in real people's mouths.

## Procedure

### Step 1 — Resolve context

Determine two things before anything else:

1. **Repo type** — one of `cli | framework | module | service`. Read `metaphor.toml`
   at the invocation root and use its `type` field. If absent, infer from markers
   (a `backbone-*` name → framework or module; a `*-service` / app shell → service;
   a binary/CLI entrypoint → cli) and STATE the inference you made.
2. **Scope** — is this the metaphor root (judge the whole system + inter-module seams)
   or a single unit (judge this unit + its outward interactions)?

Also read `metaphor.toml`'s `domain` field (e.g. `laundry-ops`) — it decides which
business/domain expert gets invited.

### Step 2 — Set focus (scope governor)

A council with no focus boils the ocean, especially at root. Pick exactly one lens
from `composition.yaml` → `focus_lenses` (e.g. `coherence`, `leverage`, `pruning`,
`maturity`, `ops-ux-security-readiness`). Use the context's `default_focus` unless the
user named one. If the invocation is genuinely ambiguous about focus, ask the user with
AskUserQuestion — but only then. Bound the whole run to the chosen lens.

### Step 3 — Compose the roster

Read `composition.yaml`. The roster is three rings:

- **Standing seats** (always): chair, skeptic, steelman, yagni-business.
- **Context seats** (auto, from the repo type block).
- **Invited guests** (conditional): seat them only when their `when:` condition holds.
  This is the summon rule — decide it yourself from context. Use AskUserQuestion ONLY
  for true ambiguity (e.g. a service that spans two domains: which business expert?).
  You should usually already know.

Load each seated persona from `personas/<id>.yaml`. See `references/persona-schema.md`
for the fields. If a needed context seat has no file yet, create it from the schema
rather than skipping it.

### Step 4 — Run the council

Order of voices, every time:

1. **Steelman first.** Build the strongest honest case for the thing under review, so
   the critique attacks the real design, not a weak version of it.
2. **Context seats attack.** Each seat fires on its `primary_attack` only — its one
   assigned failure mode — so seats don't pile onto the same point.
3. **Skeptic** hunts the load-bearing assumption nobody questioned.
4. **yagni-business** asks what concrete pain this removes *this month* vs. at a scale
   the repo doesn't have yet.

The three core dissenting seats — **skeptic, steelman, chair** — run as isolated
Claude Code subagents (see `subagent_seats` in `composition.yaml`). Dispatch each via
the Task tool with the resolved context, the chosen focus, and the relevant files
packed in; they return their finding as a compact block. Play the remaining seats
in-context yourself. (If subagents are unavailable, voice all seats in-context, but
keep skeptic and steelman strictly separated — never let one voice do both.)

### Step 5 — Synthesize (Chair)

The Chair adjudicates; it does not average. Hand the Chair every seat's output and have
it emit the report in `references/output-contract.md` EXACTLY. The Chair must state, for
its top recommendation: the residual negative value, reversibility, and the evidence
that would flip the decision. Cross-scope ideas the seats raised but that fall outside
the focus (e.g. "spin up a new backbone-* for X") go in the parking lot, not lost.

### Step 6 — Persist the result

After the Chair's report is final, the orchestrator (not the read-only Chair subagent)
writes it to disk so the decision survives the session and the team can read it:

- **Location:** `docs/council/` at the invocation root. Create the directory if absent.
- **Filename:** `<YYYY-MM-DD>-<repo-type>-<unit>-<focus>.md` (e.g.
  `2026-06-28-service-bersihir-ops-ux-security-readiness.md`). Use the unit's directory
  name for `<unit>`, or `root` when judging the whole workspace. Slugify (lowercase,
  hyphens). If a file with that name already exists from a run the same day, append
  `-2`, `-3`, … rather than overwriting — each run is its own record.
- **Contents:** the Chair's report VERBATIM (the `references/output-contract.md` shape),
  preceded by a one-line front-matter header recording the date, repo type, unit, focus
  lens, and the roster that sat — so a later reader knows the conditions the call was made
  under.
- After writing, tell the user the path. The on-screen report and the file are identical;
  the file is the durable copy, not a summary.

## Files

- `composition.yaml` — context → roster rules and the focus-lens list. Edit this to add
  a new repo type (the "maybe another type" case) — no SKILL.md change needed.
- `personas/*.yaml` — the persona registry (the genotype). One file per seat.
- `references/persona-schema.md` — how a persona file is shaped and how to add one.
- `references/output-contract.md` — the exact report template the Chair must produce.
- `../agents/council-*.md` — the subagent deployments for the core dissenting seats.

## Genotype → phenotype note

`personas/*.yaml` is the source of truth. The subagent files in `.claude/agents/` are
deployment artifacts derived from it — the same single-schema-generates-artifacts
pattern Metaphor uses everywhere. Until `metaphor-cli council sync` exists to generate
them, keep an agent file and its `personas/<id>.yaml` in step by hand.
