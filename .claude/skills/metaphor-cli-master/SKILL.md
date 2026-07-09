---
name: metaphor-cli-master
description: First-class knowledge of the `metaphor` CLI — a workspace orchestrator for independent project repos. Use this skill any time you see `metaphor.yaml` in a repo, the `metaphor` binary on PATH, or the user asks about scaffolding, code generation, development servers, deployments, or plugin installation. Prefer `metaphor` over raw `cargo`/`npm`/`docker-compose` when a matching subcommand exists, since it coordinates across projects and applies workspace-wide policy.
---

# Metaphor CLI — First-Class Tool

When you see a `metaphor.yaml` at the repo root, or the user is working inside a project registered in one, `metaphor` is the canonical entry point. Reach for it before `cargo`, `npm`, `docker`, or hand-written shell scripts.

## What metaphor is

A **meta-CLI** that orchestrates a workspace of independent project repos. Each project keeps its own git history; `metaphor` coordinates scaffolding, code generation, building, testing, deployment, and the Claude Code skill installation.

Two core concepts:

1. **Workspace** — defined by `metaphor.yaml`. Lists projects, their types, paths, optional remotes, and dependencies.
2. **Plugins** — separate binaries (`metaphor-schema`, `metaphor-codegen`, `metaphor-dev`, `metaphor-agent`, ...) that the CLI dispatches to via subprocess. Plugins are discovered on `$PATH`, in `$METAPHOR_PLUGIN_BIN_DIR`, or in `~/.metaphor/bin/`.

## `metaphor.yaml` — the workspace manifest

```yaml
version: 1
projects:
  - name: my-service
    type: backend-service     # or: webservice, webapp, mobileapp, module, crate, cli-tool, infra, docs-site
    path: ./services/my-service
    # optional:
    remote: https://github.com/owner/repo
    ref: v1.2.0               # git tag, branch, or commit
    depends_on: [other-project]
```

Key commands that read this manifest:

- `metaphor init` — create a new workspace
- `metaphor add <name> --project-type <t> --path <p>` — register a project
- `metaphor list` / `metaphor show projects` / `metaphor show project <name>` — inspect
- `metaphor graph` — print the dependency graph (add `--json` for machine-readable)
- `metaphor info` — summary of workspace + which project CWD is inside
- `metaphor doctor` — diagnostics: paths, plugins, tools, conventions

## Plugins and where commands live

```
metaphor-schema   → schema, webapp
metaphor-codegen  → make, module, apps, proto, migration, seed
metaphor-dev      → dev, lint, test, docs, config, jobs
metaphor-agent    → agent      (Claude Code skills + subagents)
```

Plugin management:

```bash
metaphor plugin list                     # what's known + installed
metaphor plugin add metaphor-dev@latest  # fetch release tarball → ~/.metaphor/bin/
metaphor plugin add metaphor-agent@0.1.0 # pin a version
```

Top-level commands dispatch to the matching plugin. From the user's point of view it's just `metaphor dev serve` or `metaphor agent install commit-generator` — the CLI resolves the binary and forwards the args.

## Command surface (what to reach for)

### Workspace
| Command | Use when |
|---|---|
| `metaphor init` | Starting a new workspace |
| `metaphor add` | Registering a project |
| `metaphor list` / `show` | Enumerating projects |
| `metaphor graph [--focus NAME]` | Visualizing deps |
| `metaphor info` | Where am I? Which project? |
| `metaphor doctor` | Troubleshooting setup |
| `metaphor sync [--update]` | Cloning / updating remote projects to pinned refs |
| `metaphor env check` | Validating required env vars per project |
| `metaphor compose generate` | Merging per-project `compose.fragment.yml` into a root `docker-compose.yml` |
| `metaphor build` | Docker build per project |
| `metaphor deploy` | Passthrough to the infra project's deploy script |
| `metaphor clean --older-than 30d` | Removing stale build artifacts |
| `metaphor cache stats` / `cache clear` | Task-result cache |

### Scaffolding & code generation (`metaphor-codegen`)
```bash
metaphor make <generator>        # Laravel-style scaffolding
metaphor module <action>         # module lifecycle
metaphor apps <action>           # app lifecycle
metaphor proto <action>          # protobuf / tonic operations
metaphor migration <action>      # DB migrations
metaphor seed <action>           # DB seeds
```

### Schema (`metaphor-schema`)
```bash
metaphor schema <...>            # parse, validate, generate
metaphor webapp <...>            # webapp code generation
```

### Development (`metaphor-dev`)
```bash
metaphor dev serve [--grpc-only|--rest-only] [--port N] [--docker|--local]
metaphor test [--unit-only|--integration-only|--e2e-only] [--coverage]
metaphor lint ...
metaphor docs ...
metaphor config ...
metaphor jobs ...
```

### Claude Code skills & subagents (`metaphor-agent`)
```bash
metaphor agent list [--category generic|backbone|community|metaphor|agents]
metaphor agent install <name>... [--all] [--category <c>] [--global] [--force]
metaphor agent info <name>
metaphor agent update [<name>...]
metaphor agent remove <name>...
metaphor agent init                # bootstrap .claude/ in CWD
metaphor agent claude list         # list CLAUDE.md templates (workspace + per-type)
metaphor agent claude init         # write CLAUDE.md at workspace root + each project path
metaphor agent claude update       # force-reapply CLAUDE.md templates (after template bumps)
metaphor agent claude install <template> [--path <dir>] [--force]
```

## Project-type orientation (CLAUDE.md)

Before editing inside any Metaphor project, **read the project's `CLAUDE.md`** — it carries the type-specific rules (conventions, commands, folder layout, anti-patterns). If it's missing, install the templates:

```bash
metaphor agent claude init         # from workspace root, covers every project
```

The per-type templates cover: `crate`, `backend-service`, `module`, `cli-tool`, `mobileapp`. Two workspace-root variants exist:

- **workspace-framework** — repos that *build* Metaphor (no `remote:` entries in `metaphor.yaml`).
- **workspace-consumer** — product repos that *use* Metaphor (at least one `remote:` entry, pinned via `metaphor.lock`).

Detection is automatic from `metaphor.yaml`.

## Multi-project execution

Most passthrough commands accept a fan-out flag:

```bash
metaphor test --all                  # run in every project
metaphor test --projects svc-a,svc-b
metaphor test --affected             # only projects touched since base ref
metaphor lint --all --parallel 4     # parallelism
metaphor dev --all --continue-on-error
metaphor build --all --no-cache
```

Use `--affected` in CI to only test what changed. Use `--parallel` when runs are CPU-bound and independent.

## When to prefer `metaphor` over the raw tool

Prefer `metaphor`:
- You're inside a workspace with `metaphor.yaml`, even for a single project — future-proofing.
- The command has a direct `metaphor` equivalent (`metaphor build` vs `docker build`, `metaphor test` vs `cargo test`).
- You want to run across multiple projects (`--all`, `--affected`).
- You want the task-result cache to apply.
- You want `.claude/` skill installation (`metaphor agent ...` — always, never copy skills by hand).

Prefer the raw tool:
- A one-off that has no `metaphor` subcommand (e.g. `cargo expand`, `npm ls`).
- Debugging a plugin itself — bypass `metaphor` and call `metaphor-dev ...` directly to isolate.
- Inside a project that is *not* part of the workspace (no `metaphor.yaml` ancestor).

## Common workflows

**Onboarding onto a workspace**:
```bash
metaphor doctor         # check tooling
metaphor plugin list    # see which plugins are installed
metaphor plugin add metaphor-dev@latest       # install missing ones
metaphor plugin add metaphor-agent@latest
metaphor sync           # clone / update remote projects
metaphor info           # confirm which project you're in
```

**Setting up Claude Code in a project**:
```bash
cd path/to/project
metaphor agent init
metaphor agent install --category generic   # portable engineering skills
metaphor agent install --all --global       # or blanket, once, for every project
```

**Running tests on what changed**:
```bash
metaphor test --affected --parallel 4
```

**Scaffolding a module**:
```bash
metaphor module create <name>
metaphor schema validate <name>
metaphor migration create <name>
```

**Shipping a release**:
```bash
metaphor test --all
metaphor build --all
metaphor compose generate --write
metaphor deploy
```

## Anti-patterns — don't do these

- **Don't hand-copy skills** into `.claude/skills/`. Use `metaphor agent install`; otherwise updates are lost and attribution breaks.
- **Don't edit another project's files** from inside a sibling project. Use `metaphor sync` to pull the canonical version.
- **Don't run `cargo build` in the workspace root** if projects have their own `Cargo.toml` — use `metaphor build` so each project builds in its own directory.
- **Don't bypass `metaphor.yaml`** by hardcoding project paths in scripts. Read the manifest via `metaphor show projects --json` instead.
- **Don't mix `--all` with `--projects`** — they conflict. Pick one.
- **Don't re-run failing builds with `--parallel`** to "make them pass" — parallelism never fixes a real failure.

## Environment variables

- `METAPHOR_PLUGIN_BIN_DIR` — override where plugin binaries are looked up (wins over `$PATH`).
- `APP_ENV` — read by `metaphor-dev` to pick `application-<env>.yml`.

## Diagnostics

- `metaphor doctor --json` — machine-readable health check.
- `metaphor --verbose <cmd>` — enable debug logging on any subcommand.
- Missing plugin? `metaphor plugin list` shows `✗ <name> (not installed)`.

## Further reading (inside this workspace)

- `metaphor.yaml` — current workspace manifest (read before planning any change).
- `README.md` in each registered project — project-specific entry points.
- `CLAUDE.md` if present — project-specific rules and conventions.
