# Roadmap & Milestones

ZRAI is built in four phases adapted from the blueprint. Each phase lists its scope and
acceptance criteria. **Phases 1–4 are not started** — this repository is at the scaffold stage
(see `development-log.md`). Phase 0 (documentation scaffold) is complete.

> **Gate:** Resolved — hybrid (Rust core + embedded TypeScript rule layer). See
> [stack-tradeoffs.md](design-decisions/stack-tradeoffs.md).

## Success metrics (apply to all phases)

These are the quantitative targets the finished system must meet. They are restated from the
blueprint and verified per [`testing.md`](testing.md).

| Metric | Target |
| --- | --- |
| Added latency on first token (TTFT) | < 5 ms |
| RAM idle (standby) | < 20 MB |
| RAM under heavy streaming | < 50 MB |
| Disk writes during a session | 0 IOPS |

## Phase 0 — Documentation scaffold *(complete)*

- README, AGENTS.md, docs index, architecture, design-decision records, roadmap, testing
  strategy, guides placeholders.
- **Acceptance:** A reader can understand what ZRAI is, its structure, the open stack decision,
  and the plan — without reading a chat transcript.

## Phase 1 — Core Proxy Engine (hybrid)

- **Stack:** Rust core (`crates/zrai-core`) + embedded TypeScript rule layer (`rules/`).
- **Scope:**
  - **Rust core:** listener on `localhost:4040` with route parity for `POST /v1/chat/completions`
    (OpenAI) and `POST /v1/messages` (Anthropic); bidirectionally buffered SSE handler with
    minimal overhead; egress over TLS to a configurable upstream; ZDR injection.
  - **TypeScript rule layer:** sanitization rules (paths, PII, tokens, git metadata) authored in
    `rules/`, compiled to `rules/dist/sanitize.js`, loaded by the Rust core via the embedded
    QuickJS engine. Independently testable with `bun test`.
- **Acceptance:**
  - Both API shapes are accepted and forwarded correctly (verified against an upstream mock).
  - SSE streams end-to-end with minimal proxy overhead in local measurement.
  - Sanitization is verified by the Bun test suite on known fixtures.
  - No disk writes occur during a session.

## Phase 2 — Sanitization & Rule Engine

- **Scope:**
  - Header scrubbing (`User-Agent`, local IPs, tracking headers).
  - Context engine: zero-allocation-target regex matching for absolute paths
    (`/home/...`, `C:\Users\...`), PII (emails, IPv4/IPv6, SSH keys, bearer tokens), and git
    metadata.
  - Structural (AST) rules for source-bearing messages.
  - Configurable strictness via `zrai.toml`: custom token-replacement rules and path masks.
- **Acceptance:**
  - Unit tests prove each PII/path/token class is neutralized on known fixtures.
  - Custom rules from config are applied.
  - Sanitization is deterministic (same input → same output).

## Phase 3 — Developer CLI & Sidecar Integration

- **Scope:**
  - CLI wrapper `zrai exec -- <command>` that sets the tool's environment variables.
  - TUI status dashboard: live throughput, active stream states, stripped-item counts, and
    zero-retention status — **without rendering prompt text**.
- **Acceptance:**
  - `zrai exec` launches a subprocess whose `OPENAI_BASE_URL`/`ANTHROPIC_BASE_URL` point at
    the daemon.
  - Dashboard shows live stats and never prints prompt content.

## Phase 4 — Verification & Benchmarking

- **Scope:**
  - Memory-inspection tests: process memory is cleared and no temp files are generated during
    heavy streaming.
  - Latency benchmarks: <5 ms added TTFT.
  - Resource benchmarks: <20 MB idle / <50 MB load; 0 disk IOPS (verified via syscall traces /
    `lsof`).
- **Acceptance:** All four success metrics are measured under a documented procedure and meet
  target, or deviations are recorded honestly in `development-log.md`.

## Status summary

| Phase | Status |
| --- | --- |
| 0 — Scaffold | Complete |
| 1 — Core Proxy (hybrid) | Core built & tested; TS engine embed next |
| 2 — Sanitization | Partly covered by the `rules/` layer (Phase 1) |
| 3 — CLI & TUI | Not started |
| 4 — Verification | Not started |
