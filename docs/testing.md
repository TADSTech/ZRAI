# Testing & Verification Strategy

This document defines *how* ZRAI's behavior and success metrics are verified. It is
stack-agnostic; the exact runner (cargo test / bun test / node:test) is fixed by the
[stack trade-off](design-decisions/stack-tradeoffs.md). Tests are part of the implementation,
not a follow-up.

## Principles

- **Behavior over execution.** Tests assert what the system does, not merely that code runs.
- **Deterministic inputs.** Fixtures are fixed; no live timestamps, randomness, or network
  calls in unit tests. Upstream is mocked.
- **Separate the four test kinds** (per AGENTS.md §6): correctness, statistical/numerical
  validation, integration, and benchmarks. They are not interchangeable.
- **No fake integrations.** The upstream is a local mock that records what it received; we
  assert on that recording, not on a stubbed "success."

## Test layers

### 1. Sanitization correctness (unit)
Owning module: Context Engine + Header Sanitizer.

| Case | Input fixture | Expected |
| --- | --- | --- |
| Absolute Unix path | `/home/alice/src/secrets.py` | replaced with mask (e.g. `<path>`) |
| Absolute Windows path | `C:\Users\bob\repo\key.pem` | replaced |
| Email PII | `contact: jane@corp.com` | redacted |
| IPv4 / IPv6 | `10.0.0.5`, `fe80::1` | redacted |
| SSH private key block | `-----BEGIN OPENSSH PRIVATE KEY-----...` | redacted |
| Bearer token | `Authorization: Bearer sk-...` | redacted |
| Git metadata | commit hashes / author lines in messages | redacted |
| Custom rule (config) | user-supplied pattern + replacement | applied |
| Idempotence | sanitized output re-sanitized | unchanged |

### 2. ZDR injection (unit + integration)
Owning module: Payload Injector.

- OpenAI-shaped payload forwarded to a mock upstream contains the configured ZDR parameter
  (e.g. `provider.zdr: true` for OpenRouter).
- Per-provider mapping is configurable, not hard-coded; test two providers.
- Header-based ZDR (where a provider uses headers) is also injected when configured.

### 3. Routing parity (integration)
Owning module: Ingress Listener.

- `POST /v1/chat/completions` and `POST /v1/messages` are both accepted and mapped to the
  correct upstream route.
- Unknown routes return a clear error; the proxy does not silently misroute.

### 4. Streaming passthrough (integration)
Owning module: Egress Streamer.

- A mock upstream emitting SSE chunks streams to the client verbatim.
- Chunks arrive in order; connection close flushes state (see memory test).

### 5. Volatile-memory / zero-disk (the core invariant)
Owning invariant: whole system.

- **No disk writes:** under a heavy streaming session, syscall traces / `lsof` show 0 write
  IOPS to any working directory. The only allowed read is the config file at startup.
- **Memory flush:** after a connection closes, retained request state is dropped; a
  memory-inspection test confirms buffers are reclaimable / not growing across many requests
  (no leak). Measured, not asserted by "it finished."

### 6. Benchmarks (statistical)
Mapped to the success metrics:

| Benchmark | Method | Target |
| --- | --- | --- |
| Added TTFT | Local client → ZRAI → mock upstream; measure delta vs direct | < 5 ms |
| RAM idle | Sample RSS after startup, settled | < 20 MB |
| RAM under load | Sample RSS during sustained streaming | < 50 MB |
| Disk IOPS | `lsof` / syscall trace during session | 0 writes |

Each benchmark records its measurement conditions (hardware, runtime version, concurrency) so
results are interpretable and reproducible.

## Honesty rules (AGENTS.md §6)

- A green suite is not proof of correctness; a passing build is not proof of behavior.
- If a metric cannot be measured in this environment, the gap is recorded in
  `development-log.md`, not papered over.
- Benchmarks report conditions; no number is cited without its method.

## Prerequisites

- Stack decision made (gates Phases 1–4). **Resolved:** hybrid.
- Upstream mock harness available (no external network in tests).

## Current verification (2026-10-28)

- **TypeScript rules:** `cd rules && bun test` → 16 pass, 0 fail.
- **Rust core:** `cargo test --workspace` → 13 pass, 0 fail (config, routes, sanitize, zdr).
- **Not yet verified:** end-to-end proxy against a mock upstream, SSE pass-through, latency/RAM/
  disk benchmarks, and the embedded TS-engine path. These remain Phase 1/4 work.
