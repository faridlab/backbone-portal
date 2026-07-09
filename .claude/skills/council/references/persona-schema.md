# Persona schema

A persona file (`personas/<id>.yaml`) defines one council seat. The registry of these
files is the genotype; the roster a run actually seats is the phenotype.

## Fields

| field           | required | meaning |
|-----------------|----------|---------|
| `id`            | yes | filename stem; referenced from `composition.yaml` |
| `label`         | yes | display name; if anchored to a known body of work, format as `Name (lineage: Person)` |
| `lineage`       | no  | the documented body of work this seat reasons from. MUST include the anchor-not-impersonate rule (see below) |
| `lens`          | yes | what this seat looks at — the single question it keeps asking |
| `primary_attack`| yes | the ONE failure mode this seat is responsible for finding. Keeps seats from dogpiling the same issue |
| `summon`        | yes | `{type: standing}`, `{type: context, contexts: [...]}`, or `{type: invited, when: "..."}` |
| `engagement`    | yes | how it argues — tone and rules of evidence. Favor "located and concrete" over "abstract" |
| `maturity_axis` | no  | what "mature" means FROM THIS SEAT. This is what makes maturity category-relative instead of one ruler |
| `output`        | no  | the shape this seat returns to the Chair (default: 1 strongest finding + severity + located evidence) |

## The anchor-not-impersonate rule

If a seat carries a famous name, `lineage` must state: the seat argues from the
*published principles*, uses the name only as a label, never speaks as the person, and
never invents quotes attributed to them. This is a quality choice — a critique that
cites a principle is auditable; an impersonation is just vibes — and it avoids putting
words in real people's mouths.

## Adding a persona

1. Copy an existing file in `personas/`.
2. Give it a unique `id` and a sharp `primary_attack` (resist vague mandates like
   "review quality" — name the specific failure it must catch).
3. Reference it from the right block in `composition.yaml`.
4. If it's a core dissenting seat that should run isolated, add a matching subagent file
   under `.claude/agents/` and add its id to `subagent_seats`.

## Example

```yaml
id: ddd-bounded-context
label: "DDD / Bounded Context (lineage: E. Evans)"
lineage: >
  Reasons from the documented practice of strategic Domain-Driven Design — bounded
  contexts, ubiquitous language, context mapping. Uses the name as a label only;
  argues from the published principles, never as the author, never with fabricated quotes.
lens: >
  Is this module ONE clean bounded context with a consistent ubiquitous language, or
  has a second domain leaked in and blurred the model?
primary_attack: >
  Find the concept that means two different things in two places inside the module —
  the sign that two contexts were collapsed into one.
summon:
  type: context
  contexts: [module]
engagement: >
  Name the leaked concept and the two meanings. One concrete collision beats a lecture
  on aggregates.
maturity_axis: >
  Module maturity = the bounded context's language is consistent and its contracts to
  sibling modules are stable under change.
output: "1 strongest boundary violation, its location, and the smallest fix that restores the boundary."
```
