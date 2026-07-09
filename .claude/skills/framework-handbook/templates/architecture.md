<!-- Reader: Maintainer · Mode: Explanation -->
# Architecture

> One paragraph: what this system is, at the altitude a new maintainer needs before reading on.

## 1. Context

Who/what uses this system, and what it depends on.

```mermaid
C4Context
    title System Context
    Person(dev, "App developer")
    System(this, "<system>", "<one line>")
    System_Ext(ext, "<external system>", "<one line>")
    Rel(dev, this, "uses")
    Rel(this, ext, "calls")
```

*What to notice: <the one thing this diagram is meant to show>.*

## 2. Containers

The deployable/runnable pieces and how they communicate.

```mermaid
flowchart LR
    A[<container>] -->|<protocol>| B[<container>]
```

*What to notice: <…>.*

## 3. Components / modules

Internal structure. For Metaphor modules, the DDD 4-layer shape; for the CLI, the
plugin/subprocess model.

- **<layer/component>** — responsibility, what it may and may not depend on.

## 4. Data & control flow

Trace one representative operation end-to-end.

1. Trigger → 2. … → 3. … → 4. Result.

```mermaid
sequenceDiagram
    actor User
    User->>System: <operation>
    System->>Module: <call>
    Module-->>User: <result>
```

*What to notice: <…>.*

## Key decisions

Link to the ADRs that explain why this architecture, not another.

- [ADR-NNNN](./adr-NNNN.md) — <decision>.
