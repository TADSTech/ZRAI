# Architecture

This document describes ZRAI's structure as designed. It is the canonical reference for
*how the system is structured*. It is intentionally stack-agnostic: component names and
responsibilities are fixed by the blueprint, but the file/module layout is decided with the
[stack tradeoff](design-decisions/stack-tradeoffs.md).

## Position in the toolchain

ZRAI is a local HTTP proxy. Existing tools are pointed at it instead of the provider; ZRAI
forwards to the real provider. No tool code changes are required.

```
┌─────────────────────┐      ┌──────────────────────────┐      ┌────────────────────┐
│  Local AI tool      │      │  ZRAI (localhost:4040)   │      │  Upstream provider │
│  (Cursor/Aider/     │─────▶│  sanitize + ZDR enforce  │─────▶│  (OpenAI /         │
│   Claude Code /     │      │  all in RAM, no disk     │      │   Anthropic /      │
│   custom agent)     │◀─────│  SSE passthrough         │◀─────│   OpenRouter ...)  │
└─────────────────────┘      └──────────────────────────┘      └────────────────────┘
     OPENAI_BASE_URL = http://localhost:4040/v1
```

The tool's only change is an environment variable:
`OPENAI_BASE_URL=http://localhost:4040/v1` (or `ANTHROPIC_BASE_URL`).

## Components

| Component | Responsibility | Memory boundary |
| --- | --- | --- |
| **Ingress Listener** | Terminates the standard REST/SSE endpoint on `:4040`; parses the JSON payload into an in-memory buffer. | In-memory socket buffer |
| **Header Sanitizer** | Strips `User-Agent`, local network IPs, and custom tracking headers; removes client traces. | Ephemeral heap string |
| **Context Engine** | Regex + structural (AST) scrubbing of the `messages` array: absolute paths (`/home/...`, `C:\Users\...`), PII (emails, IPv4/IPv6, SSH keys, bearer tokens), git metadata. | Zero-allocation regex matcher (target) |
| **Payload Injector** | Appends provider-specific ZDR flags to the root payload (e.g. `provider.zdr: true` on OpenRouter). | Mutable payload buffer |
| **Egress Streamer** | Multiplexed TLS SSE streaming to the upstream provider; streams chunks straight back to the client. | Direct streaming chunk pass-through |

## Request lifecycle

1. **Ingress** — Local tool `POST`s to `http://localhost:4040/v1/chat/completions`
   (OpenAI) or `/v1/messages` (Anthropic).
2. **Interception & parsing** — ZRAI parses the JSON body into an in-memory buffer. No disk write.
3. **Sanitization** —
   - *Header scrubbing*: strip `User-Agent`, local IPs, tracking headers.
   - *Context cleaning*: run regex + AST rules over `messages` to neutralize absolute paths,
     environment dumps, and API-key patterns.
4. **ZDR injection** — inject required provider parameters into the root payload.
5. **Egress & streaming** — forward the sanitized payload over TLS to the target upstream and
   stream SSE chunks back to the client in real time.
6. **Flush** — drop all request state from memory immediately on TCP connection close.

## Memory model (the core invariant)

Everything is **volatile**:

- Request and response streams live only in RAM.
- No logs, no temporary files, no cache databases are written during operation.
- The only disk touch permitted is reading the static config file **once at startup** (not per
  request). This is the single allowed exception and must be documented in code.
- On connection close, references are dropped so the garbage collector / allocator reclaims
  the buffers.

This invariant is what makes the "zero disk footprint" success metric verifiable (see
[testing.md](testing.md)).

## Four pillars → where they live

| Pillar | Primary component(s) |
| --- | --- |
| Universal drop-in compatibility | Ingress Listener (route parity for both API shapes) |
| Deterministic context sanitization | Header Sanitizer + Context Engine |
| Volatile memory pipeline | All components (memory boundaries above) |
| Upstream ZDR enforcement | Payload Injector + Egress Streamer |

## Cross-cutting concerns (planned)

- **Configurability** — a `zrai.toml` (or equivalent) lets users add custom token-replacement
  rules and path masks, and select the upstream + ZDR parameters. See the roadmap Phase 2.
- **Observability** — a TUI/dashboard shows live throughput, active stream states, stripped
  item counts, and the zero-retention status, **without rendering prompt text** (privacy).
  See roadmap Phase 3.
- **Verification** — automated memory-inspection and latency tests. See [testing.md](testing.md).

## Realization (hybrid, decided)

The stack trade-off is resolved as a **hybrid**: a Rust core
([`zrai-core`](../crates/zrai-core)) for the proxy hot path, with an embedded TypeScript rule
layer ([`rules/`](../rules)) for sanitization, loaded via QuickJS. See
[stack-tradeoffs.md](design-decisions/stack-tradeoffs.md) for the full rationale and the accepted
footprint trade-off. Component names above are unchanged; their realization is:

- **Ingress Listener, Header Sanitizer (transport parts), Payload Injector, Egress Streamer →
  Rust** (`zrai-core`).
- **Context Engine (regex + AST scrubbing) → TypeScript** (`rules/`), executed inside the Rust
  core via the embedded engine.

Open item:

- **Exact upstream ZDR parameter names per provider** — these vary (OpenRouter uses
  `provider.zdr: true`; others differ). The injector must be a configurable mapping, not
  hard-coded per call site.
