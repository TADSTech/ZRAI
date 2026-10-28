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
- **Next (then).** Resolved below — see 2026-10-28b.

## 2026-10-28b — Git initialized; remote pushes live; hybrid stack decided

- **Git.** Repo initialized (`git init -b main`); initial scaffold committed and pushed to
  `https://github.com/TADSTech/ZRAI.git` (remote `origin`, branch `main`, commit `315d8df`).
  Auth via the Git Credential Manager with `GCM_INTERACTIVE=never` (HTTPS, no GUI). User
  directive: commit **and push every change**.
- **Stack decision — HYBRID.** User asked whether both codebases could be used, sharing
  benefits at the cost of slightly larger size/footprint. Decision: **Rust core
  (`crates/zrai-core`) + embedded TypeScript rule layer (`rules/`)** via QuickJS. Records updated
  in `stack-tradeoffs.md` (status → DECIDED), `architecture.md` (realization), `roadmap.md`
  (Phase 1 hybrid). The integration contract lives in `stack-tradeoffs.md`.
- **Next (then).** See 2026-10-28c.

## 2026-10-28c — Phase 1: Rust proxy core + TypeScript rules layer

- **What.** Added `crates/zrai-core` (axum + reqwest): `127.0.0.1:4040` listener, OpenAI
  `/v1/chat/completions` + Anthropic `/v1/messages` parity, header scrubbing, a `Sanitizer`
  trait (native impl now), configurable ZDR deep-merge, reqwest egress with SSE pass-through,
  in-memory only. Added the TypeScript rule layer in `rules/` with a 16-test Bun suite and a
  `bun build` step producing `dist/sanitize.js`.
- **Results.** `bun test` → 16 pass. `cargo test --workspace` → 13 pass (config, routes,
  sanitize, zdr); cold compile ~8m. The sandbox denied a few final test-binary writes to
  `target/debug/deps/*.exe`, but the suite ran green (environment quirk, not a code failure).
- **Decisions.** The native sanitizer is a fail-closed fallback; the TypeScript layer is the full
  source of truth and will be embedded via QuickJS next (C toolchain confirmed: MSVC/VS2022).
- **Lessons.** First compile of the axum/reqwest/rustls tree is slow; later builds are
  incremental. The sandbox's executable-write restriction is a CI-environment concern to note.
- **Next.** Embed `rules/dist/sanitize.js` in the Rust core (rquickjs) behind a `js` feature; add
  an integration test against a local upstream mock; then Phase 3 CLI/TUI.

## 2026-10-28d — Embedded TS engine (hybrid live) + end-to-end tests

- **What.** `js_sanitizer.rs`: QuickJS on a worker thread; loads `rules/dist/sanitize.js` once;
  `sanitize::build` factory picks JS when available, else native fallback. `js` cargo feature on
  by default. `proxy::router` extracted; `tests/proxy_e2e.rs` added (mock upstream).
- **Results.** `cargo test --workspace` → 17 pass (14 unit + 3 e2e). QuickJS compiled from source
  (MSVC) in ~1m15s incremental.
- **Gotchas.** rquickjs 0.9 has no `bundled` feature — `rquickjs-sys`'s build.rs compiles
  `quickjs.c` via `cc`. QuickJS is `!Send`, hence the worker-thread + mpsc design. Sandbox again
  blocked final exe writes but tests ran green.
- **Next.** SSE streaming e2e test; zero-disk IOPS verification; Phase 3 CLI/TUI; Phase 4
  benchmarks (TTFT, RAM).
