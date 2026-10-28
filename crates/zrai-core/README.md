# zrai-core — Rust proxy core

The Rust half of ZRAI's hybrid stack. It owns the proxy hot path: ingress, header scrubbing,
ZDR injection, egress streaming, and the in-memory-only invariant. Sanitization is delegated to
a [`Sanitizer`](src/sanitize.rs) implementation — currently the native fallback; the embedded
TypeScript rule layer plugs in here next.

## Module map

| File | Responsibility |
| --- | --- |
| `src/main.rs` | Entry point: load config, build state, serve on `127.0.0.1:4040`. |
| `src/config.rs` | `zrai.toml` parsing (the only startup disk read). |
| `src/routes.rs` | OpenAI `/v1/chat/completions` + Anthropic `/v1/messages` route parity. |
| `src/sanitize.rs` | `Sanitizer` trait + `NativeSanitizer` fail-closed fallback. |
| `src/zdr.rs` | Deep-merge ZDR parameters into the request root. |
| `src/proxy.rs` | The handler: parse → sanitize → ZDR → egress → stream back. |

## Build, run, test

```bash
# from the repo root
cargo build --release          # binary at target/release/zrai
cargo test --workspace         # unit tests (routes, sanitize, zdr, config)
cargo run --release            # reads ./zrai.toml if present, listens on :4040
```

Then point a client at it:

```bash
export OPENAI_BASE_URL=http://localhost:4040/v1
```

## Configuration

Copy [`../../zrai.toml.example`](../../zrai.toml.example) to `zrai.toml` (gitignored). The file
is read **once at startup**; the daemon never writes to disk during a session.

## Sanitizer backends

- **Embedded TypeScript rules (default).** `rules/dist/sanitize.js` is loaded once at startup
  into QuickJS (`rquickjs`) running on a dedicated worker thread (QuickJS is `!Send`). This is
  the full rule set and the source of truth. Gated by the `js` cargo feature (on by default).
- **`NativeSanitizer` (fallback).** Compact Rust rules (paths, emails, IPv4, bearer/API tokens,
  PEM keys, strict env secrets). Used automatically if the JS engine or bundle is unavailable,
  so the proxy never forwards unsanitized context.

> QuickJS is compiled from source by `rquickjs-sys` (via `cc`), so a C toolchain is required to
> build the `js` feature. Build without it via `cargo build --no-default-features` (native
> sanitizer only). Build the bundle first: `cd rules && bun run build`.
