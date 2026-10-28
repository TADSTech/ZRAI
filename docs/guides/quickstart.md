# Quickstart Guide (intended usage)

> **Status: NOT YET RUNNABLE.** ZRAI is at the scaffold stage; `src/` and `tests/` do not exist
> yet. This guide describes the *intended* workflow once Phase 1 lands. Treat it as the target
> interface, not current behavior.

## 1. Install / build

```
# Rust (if chosen)
cargo build --release
# or Bun/TypeScript (if chosen)
bun build --compile zrai.ts -o zrai      # single binary
```

## 2. Configure

Create `zrai.toml` (do **not** commit it — see `.gitignore`):

```toml
upstream = "https://openrouter.ai/api"
zdr = { openrouter = { provider_zdr = true } }
sanitize = { strict = true }
rules = [
  { pattern = "API_KEY_[A-Z0-9]+", replace = "<redacted>" }
]
```

## 3. Run

```
zrai                    # listens on http://localhost:4040
```

## 4. Point your tool at ZRAI

No code changes — only an environment variable:

```
export OPENAI_BASE_URL=http://localhost:4040/v1
# or for Anthropic-shaped tools:
export ANTHROPIC_BASE_URL=http://localhost:4040/v1
```

Now your IDE/CLI sends traffic to ZRAI, which sanitizes it and enforces ZDR upstream.

## 5. Verify zero-retention

```
zrai exec -- <your-ai-tool>     # sets env vars and launches the tool
```

The TUI (Phase 3) shows live throughput, stripped-item counts, and zero-retention status
without ever printing prompt text.

## What to expect

- Outbound prompts have local paths, PII, and tokens stripped.
- Upstream receives the ZDR flag, so it does not retain the conversation.
- Nothing is written to disk during a session.
