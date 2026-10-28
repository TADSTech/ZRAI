//! ZRAI proxy core.
//!
//! The Rust half of ZRAI's hybrid stack: it owns the proxy hot path (ingress, header
//! scrubbing, ZDR injection, egress streaming, volatile memory) while sanitization is
//! delegated to the TypeScript rule layer via the [`sanitize::Sanitizer`] abstraction.
//!
//! See `docs/architecture.md` and `docs/design-decisions/stack-tradeoffs.md`.

pub mod config;
pub mod proxy;
pub mod routes;
pub mod sanitize;
pub mod zdr;
