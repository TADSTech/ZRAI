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

**Phase 1 in progress.** The stack is a decided **hybrid**: a Rust proxy core
(`crates/zrai-core`) + an embedded TypeScript sanitization rule layer (`rules/`).

- **Rust core:** builds; 13 unit tests pass (routes, sanitize, zdr, config). Proxies with the
  native sanitizer today.
- **TypeScript rules:** implemented; 16 Bun tests pass; compiles to `dist/sanitize.js`.
- **Next:** embed the TS rule layer in the Rust core (QuickJS) so the full rule set runs on the
  hot path.

See [`stack-tradeoffs.md`](docs/design-decisions/stack-tradeoffs.md) for the decision,
[`roadmap.md`](docs/roadmap.md) for the plan, and [`testing.md`](docs/testing.md) for
verification. The core proxy is runnable (`cargo run --release`); the embedded TS rules are the
next step.

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
