# Stack Trade-off: Rust vs Bun/TypeScript

**Status: DECISION PENDING.** This document analyzes the two stacks the blueprint names as
options. It is a decision record in progress, not a concluded choice. Nothing in `src/` or
`tests/` should be written until this is resolved, because the choice determines the entire
codebase shape.

Both toolchains are **already installed** in this environment (cargo/rustc, bun, Node 22.22
with native type-stripping), so neither option requires new infrastructure.

## The constraints that actually matter

ZRAI is a **security/privacy tool that processes untrusted input** (whatever a dev tool sends)
and must hold to strict budgets from the blueprint:

| Metric | Target |
| --- | --- |
| Added latency on first token (TTFT) | < 5 ms |
| RAM idle (standby) | < 20 MB |
| RAM under heavy streaming | < 50 MB |
| Disk writes during a session | 0 IOPS |

Plus qualitative needs: **drop-in REST + SSE parity**, **regex + AST sanitization**, and a
**zero-dependency / local-first** distribution story.

## Option A — Rust (axum + tokio + reqwest)

**Pros**
- **Hits the memory/latency budgets most comfortably.** Native code, no GC, no VM. A standing
  axum proxy realistically idles well under 20 MB and stays under 50 MB under load. Sub-ms
  proxy overhead is routine; the <5 ms TTFT target is comfortably met.
- **True "zero-dependency" single binary.** `cargo build --release` yields one static-ish
  binary users drop in — no runtime to ship, no `node_modules`. This is the strongest match
  for the "local-first, zero-dependency" ethos and the cleanest distribution story.
- **Memory safety on untrusted input.** The sanitizer parses attacker-shaped content
  (paths, tokens, git metadata). Rust's ownership model removes whole classes of memory bugs
  that would be silent security holes in a C/unsafe path. Compile-time guarantees matter for a
  privacy tool.
- **First-class async SSE.** tokio + axum stream SSE with backpressure cleanly; reqwest does
  egress TLS. The volatile-memory invariant is easy to enforce (no implicit buffering to disk).
- **TOML config is trivial** via `serde` + `toml`.

**Cons**
- **Slower iteration.** Compile times, stricter types, more boilerplate for HTTP handlers and
  SSE body streaming. Each sanitization-rule tweak is a rebuild cycle.
- **AST sanitization is heavier.** Structural rules need a real parser. For Rust source that
  means `tree-sitter` or `syn`; wiring language grammars is more effort than in a TS ecosystem
  that already speaks JavaScript/TypeScript AST natively.
- **Smaller contributor pool / onboarding cost** if the team isn't Rust-fluent.
- AST-based rules will necessarily be *narrower* than what a TS-first stack offers (the TS
  stack can reuse the language's own parser).

## Option B — Bun/TypeScript

**Pros**
- **Fastest iteration.** Bun runs TypeScript directly (no build step); hot reload; quick
  experiments on sanitization rules. Node 22.22 also runs `.ts` via native type-stripping, so
  even without Bun the stack is viable.
- **Richer ecosystem for this exact problem.** HTTP/SSE libraries are abundant; regex
  sanitization is trivial; and **structural/ AST rules are far easier** — the TS stack can use
  `typescript`, `oxc`, or `ts-morph` to parse JavaScript/TypeScript source natively, which is
  exactly the dominant language in dev-tool contexts. This directly serves the blueprint's
  "structural AST rules" pillar.
- **Easy testing.** `bun test` / `node:test` with simple mocking of the upstream for
  sanitization-correctness, ZDR-injection, and memory-inspection tests.
- **Lower barrier for the target audience.** Dev-tool users are overwhelmingly TS-fluent.

**Cons**
- **Higher baseline memory.** A JS runtime idles higher than Rust (typically 30–60 MB) and can
  exceed the 50 MB load budget under heavy streaming. The <20 MB idle target is likely
  **missed**; the <50 MB load target is at risk. These are "targets" in the blueprint, not
  hard gates — but they are stated success metrics.
- **GC + event loop add latency variance.** Proxy overhead is usually <5 ms, but GC pauses can
  spike TTFT under load, making the latency claim noisier to verify.
- **Weaker "zero-dependency" story.** Shipping requires a runtime (Bun or Node) on the user's
  machine; it is not a single drop-in binary. A `bun build --compile` can produce a binary, but
  it embeds a runtime and is heavier/RAM-ier than a Rust binary.
- **Larger transitive-dependency surface.** More crates/npm packages → larger attack surface
  for a security tool. Sanitizer correctness relies on careful code rather than compile
  enforcement.

## Side-by-side

| Dimension | Rust (axum+tokio) | Bun/TypeScript |
| --- | --- | --- |
| Meets <20 MB idle / <50 MB load | Yes, comfortably | At risk / likely missed |
| Meets <5 ms TTFT (stable) | Yes | Usually, with GC variance |
| Single-binary distribution | Yes (best) | Partial (bundled runtime) |
| AST sanitization effort | Higher (parser wiring) | Lower (native TS AST) |
| Iteration speed | Slower | Faster |
| Untrusted-input safety | Compile-enforced | Runtime-discipline |
| Test ergonomics | Good (cargo test) | Very good (bun/node:test) |
| Contributor familiarity | Lower (if team ≠ Rust) | Higher |
| Dependency surface | Small, auditable | Larger |

## Recommendation (tentative)

- If the **strict memory budgets and the single-binary drop-in are treated as hard requirements**,
  **Rust** is the better fit and matches the blueprint's own primary suggestion.
- If **developer velocity, richest AST sanitization, and fast experimentation** matter more —
  and the RAM targets are accepted as aspirational rather than gating — **Bun/TypeScript** is
  the pragmatic choice and gets a runnable MVP fastest.

This is left **open** for your call. The architecture in `architecture.md` is stable regardless;
only the implementation language and module layout depend on this decision.

## Decision record

| Field | Value |
| --- | --- |
| Options considered | Rust (axum+tokio+reqwest); Bun/TypeScript |
| Decided | **PENDING** |
| Blocked by | Your input |
| Revisit if | A prototype shows one stack cannot meet a hard success metric |
