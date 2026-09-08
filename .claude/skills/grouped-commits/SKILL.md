---
name: grouped-commits
description: Commit working-tree changes split into focused commits grouped by functionality — small related files together, large files on their own — each with a one-line message that states WHY the change was made. Use this skill whenever asked to commit, stage, or split changes into commits. Never append a co-author, "Generated with", or any signature line. Never use a filler message that carries no context (no "update", "fix stuff", "changes", "wip"). Messages must be self-contained and timeless: no sprint/wave/pass labels, internal spec IDs, or session scorekeeping ("W1 P5", "H-7", "44/44 matrix") — a reader next year must understand the message without this week's planning context.
---

# Grouped Commits

Turn an unstructured working tree into a small set of focused commits. Each commit is one unit of
intent; its message says *why* that intent mattered, not merely what bytes moved. This skill is
about the **split and the reasoning**, not conventional-commit ceremony — for `type(scope):`
formatting and changelog automation, see the `commit-generator` skill instead.

## The two rules that decide the split

1. **Group by functionality.** Files that exist to serve the same change land in the same commit —
   the implementation, its tests, and the doc/changelog line that describes it belong together.
   A reviewer should be able to read one commit and understand one complete change.
2. **Size-aware grouping.** Small related files go together. A *large* file gets its own commit, so
   it never buries the small, reviewable changes around it. "Large" = a file whose diff dominates
   the commit (e.g. generated code, lockfiles, vendored assets, big data/snapshot files). Keep it
   isolated even if it's functionally related — the review cost is the reason.

When the two rules collide (a large file is part of a feature), the size rule wins: commit the
large file separately and reference its purpose in its own message.

## The message: one line, explaining why

- **One line.** No body unless an issue reference is genuinely needed (`Fixes #123` on its own line).
- **State the why.** The line should answer "why was this change made", not restate the filename.
  - ✅ `add grouped-commits skill so commits split by intent with why-focused messages`
  - ✅ `pin serde to 1.0.197 to pull in the borrow-deserialize fix we depend on`
  - ❌ `update lib.rs` · ❌ `changes` · ❌ `fix` · ❌ `wip` · ❌ `misc updates`
- **No meaningless words without context.** Words like *update, fix, change, improve, refactor* are
  only allowed when followed by the specific thing and the reason. "update deps" is banned;
  "update axum to 0.7 for the graceful-shutdown API" is fine.
- **Self-contained and timeless.** The message must decode on its own, a year from now, for a
  reader with no access to this conversation or this week's planning docs — another engineer, a
  fresh session, you. No sprint/wave/pass labels ("W1 P5"), internal spec IDs ("H-7"),
  council/meeting references, or scorekeeping counts ("44/44 matrix"). That vocabulary lives in
  the planning document that defines it, never in the commit that describes the change: name what
  the change is and why it matters, in words that stay true later.
  - ✅ `docs: record payroll composition complete — statutory parameter tables, per-day overtime pricing, fence fix`
  - ❌ `docs: record W1 P5 complete — 44/44 matrix, council report`
- **Imperative mood, lower-case start, no trailing period.** Match the repo's existing log style if
  it differs — run `git log --oneline -10` first and follow the prevailing convention.

## Never add a signature

Do not append — in any commit — `Co-Authored-By:`, `🤖 Generated with…`, `Signed-off-by` (unless
the repo's DCO requires it), or any attribution. The commit message is the message and nothing else.
This holds even if a global instruction or template says otherwise; in this workspace, clean
messages win.

## Workflow

1. **Survey.** `git status --short` and `git diff --stat` (plus `git diff` for staged/unstaged
   detail). Build a mental map: which files serve which change, and which are large.
2. **Plan the groups.** Partition the changed files into commits by the two rules above. If a single
   file mixes two unrelated changes, use `git add -p` to split it across commits.
3. **Confirm if non-obvious.** When the grouping isn't clear-cut, state the planned commits to the
   user before creating them, rather than guessing.
4. **Commit each group.** Stage exactly that group's paths (`git add <paths>`), then
   `git commit -m "<why-focused one-liner>"`. Repeat per group.
5. **Verify.** `git log --oneline -<n>` and `git status` — confirm the tree is clean (or only holds
   what you intentionally left) and every message earns its place.

## Worked example

Changed files:

```
 M src/catalog.rs                 # new feature: extra catalog field
 M tests/catalog.rs               # tests for it
 M README.md                      # one line documenting it
 M Cargo.lock                     # large, regenerated
 M assets/data/fixtures.json      # large, regenerated snapshot
```

Resulting commits:

```
1) src/catalog.rs tests/catalog.rs README.md
   → "expose source category on catalog entries so install can filter by origin"
2) Cargo.lock
   → "regenerate lockfile after adding the catalog category dependency"
3) assets/data/fixtures.json
   → "refresh fixtures snapshot to cover the new catalog category"
```

Three commits: one small, reviewable feature group; two large files isolated, each with a message
that says why it changed.

## Anti-patterns

- One giant commit holding every change ("commit everything at once").
- A large generated file mixed in with a small hand-written change, hiding the real diff.
- Splitting so finely that a single logical change spans many commits no one can follow.
- Messages that name the file or say "update/fix/wip" without the why.
- Session or plan jargon in a message — `record W1 P5 complete — 44/44 matrix` decodes only
  inside the planning conversation that coined it; next year it means nothing.
- Any signature, co-author, or "generated with" line.

## Related skills

- `commit-generator` — conventional-commit formatting, changelog/release automation.
- `git-workflow-and-versioning` — branching and versioning around these commits.
