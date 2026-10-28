# Design Decisions

This folder holds consequential choices and their trade-offs. Each file records the options
considered, the decision (or its pending state), and the evidence. Read these before proposing
changes that touch the affected area.

## Index

- **[Stack trade-off](stack-tradeoffs.md)** — Rust vs Bun/TypeScript. **DECISION PENDING**
  (blocks all runtime implementation).

## How to add a decision

1. Create `design-decisions/<slug>.md` with: problem, options, decision, rationale, and what
   would cause a revisit.
2. Link it from this index.
3. If the decision changes component boundaries, update `architecture.md`; if it changes
   verification, update `testing.md`; if it changes phasing, update `roadmap.md`.
