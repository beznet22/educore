//! # CommandDispatcher re-export
//!
//! The production [`CommandDispatcher`] lives in the
//! `educore-dispatcher` crate (per docs/architecture.md
//! "Command Bus + Dispatcher" layer). This module re-exports
//! it for consumers that already depend on `educore-events`.
//!
//! See `crates/cross-cutting/dispatcher/src/dispatcher.rs` for
//! the full implementation.

pub use educore_dispatcher::*;
