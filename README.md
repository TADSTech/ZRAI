# ZRAI — Zero-Retention AI Sidecar

ZRAI is a lightweight, local-first **privacy sidecar for AI developer tooling**. It sits
between local tools (agentic IDEs, CLIs, custom agents) and upstream model providers as an
OpenAI/Anthropic-compatible HTTP proxy, sanitizing outbound context and enforcing
**Zero Data Retention (ZDR)** upstream — without writing anything to disk.

> **Lineage.** The design is derived from the *NullShield Daemon* project blueprint
> ("Building Zero-Retention Private AI — Google Gemini"). ZRAI is the codename for this
> implementation; the four architectural pillars and success metrics come from that blueprint.

## Why it exists

Agentic tools (Cursor, Aider, Claude Code, custom frameworks) automatically bundle local
context — absolute file paths, hostnames, environment variables, git history, and raw source —
into prompts sent to external LLM providers. On confidential or proprietary codebases this
leaks IP and leaves retained copies upstream. ZRAI intercepts that traffic locally, scrubs
it, and forces ZDR routing, with a verifiable audit trail.

## Status

**Hybrid is live.** A Rust proxy core (`crates/zrai-core`) runs the TypeScript sanitization rules
(`rules/`) inside an embedded QuickJS engine — both halves in one process.

- **Rust core:** `:4040` listener, OpenAI + Anthropic route parity, header scrubbing, ZDR
  injection, SSE pass-through, in-memory only.
- **TypeScript rules:** embedded via QuickJS (worker thread); 16 Bun tests pass; compiled to
  `dist/sanitize.js` and loaded at startup. Native sanitizer remains a fail-closed fallback.
- **Verified:** `cargo test --workspace` → 17 pass (14 unit + 3 end-to-end).
- **Next:** SSE streaming + zero-disk verification, Phase 3 CLI/TUI, Phase 4 benchmarks.

See [`stack-tradeoffs.md`](docs/design-decisions/stack-tradeoffs.md) for the decision,
[`roadmap.md`](docs/roadmap.md) for the plan, and [`testing.md`](docs/testing.md) for
verification.

## Four pillars

1. **Universal drop-in compatibility** — mimics OpenAI `/v1/chat/completions` and Anthropic
   `/v1/messages` REST + SSE streaming. Zero code changes to existing tools.
2. **Deterministic context sanitization** — strips filesystem metadata, git signatures, and
   PII before payloads leave the device.
3. **Volatile memory pipeline** — all request/response streams processed purely in RAM; no
   logs, temp files, or cache DBs on disk.
4. **Upstream ZDR enforcement** — programmatically appends provider ZDR routing parameters
   (e.g. `provider.zdr: true` on OpenRouter).

## Repository layout

```
ZRAI/
├── README.md                 # This file
├── AGENTS.md                 # Instructions for coding agents
├── .gitignore
├── docs/
│   ├── README.md             # Documentation index and reading order
│   ├── architecture.md       # System structure and component boundaries
│   ├── roadmap.md            # Phased plan and milestones
│   ├── testing.md            # Verification strategy
│   ├── development-log.md    # Significant experiments and decisions
│   ├── design-decisions/     # Trade-off records (incl. the stack decision)
│   └── guides/               # Usage and implementation guides
├── src/                      # Implementation (not yet created — see src/README.md)
└── tests/                    # Automated tests (not yet created — see tests/README.md)
```

## Reading path

1. [`docs/README.md`](docs/README.md) — index and recommended order.
2. [`docs/architecture.md`](docs/architecture.md) — how the system is structured.
3. [`docs/design-decisions/stack-tradeoffs.md`](docs/design-decisions/stack-tradeoffs.md) —
   the decided hybrid stack and its trade-offs.
4. [`docs/roadmap.md`](docs/roadmap.md) — what will be built, and when.
