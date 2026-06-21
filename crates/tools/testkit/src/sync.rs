//! # In-memory sync primitives
//!
//! The testkit exposes a `sync` module because
//! [`lib.rs`](crate) declared `pub mod sync;` and the test
//! harness needs that module to resolve. The actual
//! `ChangeStream` and per-school `VersionCursor` table live
//! inside the in-memory storage adapter (see
//! [`storage::InMemoryStorageAdapter`](crate::storage::InMemoryStorageAdapter)
//! and its `watch_changes`, `apply_snapshot`, `cursor_for`,
//! `advance_cursor` methods).
//!
//! This module therefore provides only:
//!
//! - A `dummy_witness` function that exercises the module-load
//!   path and is callable by consumers that want a no-op
//!   witness for "the testkit's sync module is wired".
//! - A `#[cfg(test)] mod tests` block that proves the module
//!   loads under `cargo test`.
//!
//! See `docs/architecture.md` § "Sync" and
//! `docs/decisions/ADR-018-SyncEngine.md` for the engine-level
//! sync contract.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// No-op witness function.
///
/// Exists so the module compiles and the type system can verify
/// the `sync` module is wired into the testkit. The actual
/// sync primitives (`ChangeStream`, `VersionCursor`,
/// `watch_changes`, `apply_snapshot`, `cursor_for`,
/// `advance_cursor`) are exposed as methods on the in-memory
/// storage adapter — see
/// [`storage::InMemoryStorageAdapter`](crate::storage::InMemoryStorageAdapter).
///
/// Returns the unit value; the function is marked `#[must_use]`
/// so callers don't accidentally call it for side effects (it
/// has none).
#[must_use]
pub fn dummy_witness() -> () {}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    #[test]
    fn module_loads() {}

    #[test]
    fn dummy_witness_returns_unit() {
        let _: () = super::dummy_witness();
    }
}
