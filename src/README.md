# Source Layout (planned)

This directory does not exist yet. It will hold the ZRAI implementation once the
[stack trade-off](../docs/design-decisions/stack-tradeoffs.md) is decided. The component
responsibilities below are fixed by [`architecture.md`](../docs/architecture.md); only the
language and file names are pending.

## Intended modules (stack-agnostic)

| Module | Responsibility | Maps to component |
| --- | --- | --- |
| `listener` | HTTP server on `:4040`; route parity for `/v1/chat/completions` and `/v1/messages`; JSON body parse into memory. | Ingress Listener |
| `sanitize/headers` | Strip `User-Agent`, local IPs, tracking headers. | Header Sanitizer |
| `sanitize/context` | Regex + AST scrubbing of `messages`: paths, PII, tokens, git metadata. | Context Engine |
| `inject` | Append provider ZDR parameters from a configurable mapping. | Payload Injector |
| `egress` | TLS streaming to upstream; SSE pass-through. | Egress Streamer |
| `config` | Load `zrai.toml` once at startup. | — |
| `cli` | `zrai exec -- <cmd>` wrapper; TUI dashboard (Phase 3). | — |

## Invariants to preserve when implementing

- **No disk writes during a session.** Only `config` may touch disk, and only at startup.
- **In-memory only.** Request/response streams live in RAM and are dropped on connection close.
- **Deterministic sanitization.** Same input → same output; no randomness.
- **Streaming parity.** SSE chunks pass through in order with minimal overhead.

See [`../docs/testing.md`](../docs/testing.md) for what each module must prove.
