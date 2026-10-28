# ZRAI Rules — TypeScript sanitization layer

This package is the **TypeScript half of ZRAI's hybrid stack**. It owns sanitization: the
regex + structural rules that scrub untrusted outbound context (absolute paths, PII, secrets,
git metadata) before it leaves the device.

## Why TypeScript here

Sanitization is the part of ZRAI that changes most often and benefits from rich tooling. Writing
and testing it in TypeScript (with first-class AST parsers available for source-bearing messages)
gives fast iteration. The Rust core owns the proxy hot path; this package is loaded by it via an
embedded JS engine, so we get both stacks' strengths. See
`../docs/design-decisions/stack-tradeoffs.md`.

## Layout

```
rules/
├── package.json        # scripts: test (bun test), build (bun build -> dist/sanitize.js)
├── src/
│   ├── sanitize.ts     # Source of truth: rules + sanitize(messages, opts)
│   └── sanitize.test.ts# bun test suite (fixtures per category)
└── dist/               # Compiled IIFE loaded by the Rust core (generated)
```

## Develop & test (no network needed)

```
bun install        # only if you add deps; bun:test is built in
bun test           # run the suite
bun run build      # compile src/sanitize.ts -> dist/sanitize.js (IIFE, sets globalThis.sanitize)
```

## Integration contract

`dist/sanitize.js` (built from `sanitize.ts`) must expose:

```ts
sanitize(messages: Message[], opts: SanitizeOpts): { messages: Message[]; stripped: StrippedItem[] }
```

The Rust core (`crates/zrai-core`) loads this file at startup and calls it per request. The
sanitized `messages` are forwarded upstream; the `stripped` counts feed the Phase 3 TUI only.
Prompt text is never logged.

## Adding a rule

Add a `BuiltinRule` to `BUILTIN` in `sanitize.ts`, then add a fixture to `sanitize.test.ts`.
Custom end-user rules come from `zrai.toml` and are passed via `opts.rules` (applied last).
