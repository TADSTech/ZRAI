# Development Log

Factual record of significant work, decisions, and lessons. Entries are append-only; history is
not rewritten when direction changes.

## 2026-10-28 — Scaffold stage; stack decision open

- **Context.** Task: "Start building out ZRAI," referencing the *NullShield Daemon* blueprint
  ("Building Zero-Retention Private AI — Google Gemini"). ZRAI is a zero-retention, local-first
  AI privacy proxy (OpenAI/Anthropic-compatible, sanitizing, ZDR-enforcing).
- **Decision (scope).** Per user direction, this pass is **scaffold + docs only** — no runtime
  code. Goal: align on direction before implementation.
- **Decision (stack) — PENDING.** Both Rust (cargo/rustc) and Bun/TypeScript (bun, Node 22.22
  native type-stripping) are installed. Trade-off analysis written to
  `docs/design-decisions/stack-tradeoffs.md`; left open for user decision because it determines
  the entire codebase.
- **Artifacts created.** `README.md`, `.gitignore`, `docs/README.md`, `docs/architecture.md`,
  `docs/roadmap.md`, `docs/testing.md`, `docs/design-decisions/{README,stack-tradeoffs}.md`,
  `docs/guides/quickstart.md`, `src/README.md`, `tests/README.md`.
- **Toolchain probe.** Rust, Bun, Node 22.22 (with `--experimental-strip-types`) all present;
  not a git repository yet (intentionally left uninitialized).
- **Lessons.** The blueprint's success metrics (<20 MB idle RAM, single-binary story) favor
  Rust; the AST-sanitization richness and iteration speed favor Bun/TypeScript. This tension is
  the crux of the open decision.
- **Next.** User chooses the stack → Phase 1 (core proxy engine) begins, gated by the decision
  record in `stack-tradeoffs.md`.
