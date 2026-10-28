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

- `NativeSanitizer` — compact Rust fallback covering paths, emails, IPv4, bearer/API tokens, PEM
  keys, and (strict) env secrets. Used so the proxy never forwards unsanitized context if the
  JS bundle is missing.
- Embedded TypeScript rules — `rules/dist/sanitize.js` loaded via a JS engine (next step). The
  TypeScript layer is the full source of truth and the richer rule set.
