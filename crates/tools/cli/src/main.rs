//! `educore-cli` — sample binary demonstrating daily engine operations.
//!
//! Implementation lands in Phase 16 alongside `educore-sdk`
//! (the high-level consumer facade that wires the engine's port
//! adapters into a single configuration surface). Until then the
//! binary is a stub that exits cleanly so the workspace remains
//! `cargo build --workspace` green.
//!
//! See `docs/build-plan.md` § Phase 16 and `docs/guides/saas-backend.md`
//! for the consumer-facing workflow this binary will exercise.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// Entry point. Currently a no-op; see module docs for the Phase 16
/// implementation plan.
fn main() -> std::process::ExitCode {
    std::process::ExitCode::SUCCESS
}
