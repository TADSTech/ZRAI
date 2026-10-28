# Tests (planned)

This directory does not exist yet. It will hold the automated test suites once the
[stack trade-off](../docs/design-decisions/stack-tradeoffs.md) is decided and Phase 1 begins.
The test plan and acceptance criteria are defined in [`../docs/testing.md`](../docs/testing.md).

## Intended suites

| File (placeholder) | Kind | Covers |
| --- | --- | --- |
| `sanitize.test` | Unit | Path/PII/token/git redaction; custom rules; idempotence |
| `inject.test` | Unit + integration | ZDR parameter injection per provider; configurable mapping |
| `routing.test` | Integration | Both API shapes accepted and mapped correctly |
| `streaming.test` | Integration | SSE passthrough in order; flush on close |
| `memory.test` | Invariant | No disk writes; buffers reclaimed after close |
| `bench` | Benchmark | TTFT delta <5 ms; RAM <20 MB idle / <50 MB load; 0 disk IOPS |

## Conventions (to be finalized with the stack)

- No external network in unit/integration tests; upstream is mocked locally.
- Deterministic fixtures; no live timestamps or randomness in oracles.
- Benchmark runs record measurement conditions.

See [`../docs/testing.md`](../docs/testing.md) for the full strategy and honesty rules.
