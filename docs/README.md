# Documentation Index

This is the reading order for ZRAI. Start at the top and proceed downward; each document
builds on the previous. The repository is the source of truth — if a document contradicts
the code, the code wins and the document must be updated.

## Reading order

1. **[Architecture](architecture.md)** — system structure, the five components, the request
   lifecycle, and the in-RAM memory boundaries. Read this first to understand *what* ZRAI is.
2. **[Design decisions](design-decisions/README.md)** — consequential choices and trade-offs.
   The most important open item is the **[stack tradeoff](design-decisions/stack-tradeoffs.md)**,
   which is currently undecided and blocks runtime implementation.
3. **[Roadmap](roadmap.md)** — the four implementation phases, their acceptance criteria, and
   the quantitative success metrics from the blueprint.
4. **[Testing](testing.md)** — how each behavior and each success metric will be verified.
5. **[Guides](guides/quickstart.md)** — intended usage. *Not runnable yet* (scaffold stage).
6. **[Development log](development-log.md)** — dated record of what was attempted, decided,
   and learned.

## Document ownership

To avoid competing sources of truth, each fact lives in one document:

| Fact | Owning document |
| --- | --- |
| What ZRAI is and why | `README.md` |
| Component boundaries & data flow | `architecture.md` |
| Stack / tooling choice | `design-decisions/stack-tradeoffs.md` |
| Phasing & acceptance criteria | `roadmap.md` |
| Verification method | `testing.md` |
| How to run it | `guides/quickstart.md` |
| Historical decisions | `development-log.md` |
| Agent workflow rules | `AGENTS.md` |

## Status of this repository

**Scaffold stage.** Documentation and planning only. No `src/` or `tests/` implementation
exists yet. The open question that gates implementation is the stack decision in
`design-decisions/stack-tradeoffs.md`.
