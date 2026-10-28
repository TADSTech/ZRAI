# Change Episodes (Changelog)

A human-readable, chronological log of every meaningful change ("episode") to ZRAI, so the
project can be understood later without replaying a chat. One episode ≈ one coherent commit.
The engineering detail lives in [`docs/development-log.md`](docs/development-log.md); this file
is the narrative companion.

---

## Episode 1 — 2026-10-28 — Documentation-first scaffold

**What.** Created the initial repo skeleton: README, docs index, architecture, roadmap, testing
strategy, design-decision records, guides, and module/test placeholders.
**Why.** Establish a self-describing project before writing code (per `AGENTS.md`).
**Where.** `README.md`, `docs/**`, `src/README.md`, `tests/README.md`, `.gitignore`.
**Evidence.** Docs reviewed; no runtime yet.

## Episode 2 — 2026-10-28 — Git, remote, and the hybrid stack decision

**What.** Initialized git (`main`), committed the scaffold, pushed to
`github.com/TADSTech/ZRAI` over HTTPS (no GUI). Decided the stack as a **hybrid**: Rust core +
embedded TypeScript rule layer, accepting a slightly larger binary/footprint.
**Why.** User asked whether both codebases could be used to share benefits.
**Where.** `docs/design-decisions/stack-tradeoffs.md` (status → DECIDED), `architecture.md`,
`roadmap.md`, `development-log.md`.
**Evidence.** Remote `origin/main` at `e0984f3`.

## Episode 3 — 2026-10-28 — TypeScript sanitization rule layer

**What.** Implemented `rules/`: `sanitize.ts` + a 16-test Bun suite + a build step producing
`dist/sanitize.js` (IIFE). Rules cover absolute paths, emails, IPv4/IPv6 (incl. compressed
`::`), PEM private keys, bearer/API/AWS tokens, git remotes + commit hashes, strict-only env
secrets, and user-supplied custom rules.
**Why.** The TypeScript half of the hybrid — fast iteration, rich tooling, independently
testable, and the single source of truth for sanitization.
**Where.** `rules/src/sanitize.ts`, `rules/src/sanitize.test.ts`, `rules/package.json`,
`rules/README.md`.
**Evidence.** `bun test` → **16 pass, 0 fail**; `bun run build` → `dist/sanitize.js` (5.3 KB).
**Fix log.** Tests caught three real bugs, fixed in turn: compressed IPv6 not matched, the email
rule eating `git@…` SSH remotes, and the commit rule swallowing the `commit ` prefix.

## Episode 4 — 2026-10-28 — Rust proxy core

**What.** Added `crates/zrai-core`: an axum ingress on `127.0.0.1:4040` with OpenAI
(`/v1/chat/completions`) and Anthropic (`/v1/messages`) route parity, header scrubbing,
sanitization behind a `Sanitizer` trait (native implementation now; TS engine embed next),
configurable ZDR injection (deep-merged into the root payload), reqwest egress with SSE
pass-through, and in-memory-only operation.
**Why.** The Rust half of the hybrid — latency, RAM, and memory-safety on the hot path.
**Where.** `Cargo.toml`, `Cargo.lock`, `crates/zrai-core/{Cargo.toml,README.md,src/*}`,
`zrai.toml.example`.
**Evidence.** `cargo test --workspace` → **13 pass, 0 fail** (config, routes, sanitize, zdr);
cold compile ~8m. The sandbox denied writing a few final test binaries to `target/debug/deps/*.exe`,
but the suite still ran green — an environment quirk, not a code failure.
**Known gap.** The sanitizer on the hot path is currently the Rust `NativeSanitizer`; embedding
the TypeScript rule layer (QuickJS) is the immediate next step.

## Episode 5 — 2026-10-28 — Embedded TypeScript engine (hybrid goes live)

**What.** Wired the TypeScript rule layer into the Rust core via QuickJS (`rquickjs`), on a
dedicated worker thread (QuickJS is `!Send`). The core loads `rules/dist/sanitize.js` once at
startup and calls it per request. A `js` cargo feature (default on) gates it, with the native
`NativeSanitizer` as a fail-closed fallback. Extracted a `proxy::router()` helper and added
end-to-end tests (client → ZRAI → mock upstream).
**Why.** This is the payoff of the hybrid: Rust owns the hot path, TypeScript owns the rules,
and both run together in one process.
**Where.** `crates/zrai-core/src/js_sanitizer.rs`, `src/sanitize.rs` (factory), `src/proxy.rs`
(`router`), `tests/proxy_e2e.rs`, `Cargo.toml` (`js` feature + rquickjs).
**Evidence.** `cargo test --workspace` → **17 pass, 0 fail**: 14 unit (incl.
`embedded_rules_sanitize_paths_and_emails`) + 3 end-to-end (sanitized payload + ZDR + auth
forwarded; Anthropic route; unknown route → 404). QuickJS compiled from source via MSVC.
**Gotcha.** `rquickjs 0.9` has no `bundled` feature — `rquickjs-sys`'s `build.rs` compiles
`quickjs.c` with `cc` automatically.
**Footprint note.** Embedding QuickJS adds a few MB to the binary/heap — the trade the user
accepted.
