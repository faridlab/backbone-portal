<!-- Reader: Maintainer · Mode: How-to -->
# Maintainer Guide

How to maintain this project and add a feature without breaking conventions.

## Before you touch anything

- Read this project's `CLAUDE.md` and the root `metaphor.yaml`.
- Identify the project type (`crate` / `module` / `backend-service` / `cli-tool` / …) — it dictates
  the rules below.
- For `module` projects: the **schema YAML is the source of truth**. Code is generated from it.

## Where code goes

| Layer | Holds | May depend on |
|-------|-------|---------------|
| Domain | Entities, value objects, invariants | nothing |
| Application | Use cases, orchestration | domain |
| Infrastructure | DB, network, adapters | domain, application |
| Presentation | HTTP/gRPC/CLI surface | application |

## Adding a feature (regen-safe)

1. Update the **schema YAML** (SSoT), not the generated code.
2. Regenerate: `metaphor <codegen cmd>`.
3. Put hand-written logic only inside `// <<< CUSTOM` … `// CUSTOM >>>` markers — everything
   outside them is overwritten on the next regen.
4. Build and test: `metaphor build && metaphor test`.

## Versioning & release

- Bump version (conventional-commit driven), update `CHANGELOG.md`.
- `metaphor build && metaphor test` clean, then publish/release.
- Commits: conventional commits, **no Claude signatures**.

## What will break things

- Editing generated code outside CUSTOM markers.
- Adding a `main.rs` to a `crate` (wrong project type).
- Cross-editing a sibling project's files instead of `metaphor sync`.
