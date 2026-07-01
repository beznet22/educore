//! # Dispatcher wrapper layer
//!
//! This module hosts the spec-conformant wrapper pattern for routing
//! domain service calls through the [`CommandDispatcher`].
//!
//! Per `docs/architecture.md` § "Command Bus + Dispatcher" the engine
//! has two layers:
//!
//! 1. **Domain services** in `crates/domains/*/src/services.rs` —
//!    pure Rust functions taking `(Aggregate, Clock, Ids, …)` and
//!    returning `(Aggregate, Event)`. No auth, no outbox, no audit.
//! 2. **Dispatcher wrapper** in this module — thin wrappers that
//!    route each service call through [`CommandDispatcher::dispatch`]
//!    which adds RBAC, idempotency, outbox, audit, and bus publish
//!    in a single transaction.
//!
//! **No breaking change** to domain service signatures. Existing
//! aggregate-level tests remain valid. The wrappers are an addition,
//! not a replacement.
//!
//! See `docs/guides/dispatcher-wrapper-pattern.md` for the full pattern
//! and migration guide.
//!
//! [`CommandDispatcher`]: crate::dispatcher_wrapper::CommandDispatcher
//! [`CommandDispatcher::dispatch`]: crate::dispatcher_wrapper::CommandDispatcher::dispatch

#![allow(missing_docs)] // Per-wrapper docs land as each wrapper is implemented.

// `CommandDispatcher` is re-exported from `educore-dispatcher` once
// the dependency is wired in `crates/educore/Cargo.toml`. Until then,
// the skeleton compiles with just the docs.

// use educore_core::error::DomainError;
// use educore_dispatcher::CommandDispatcher;

// ---- Academic wrappers (Step 2: ~38 wrappers, deferred) -----------------
//
// Each public academic service function gets a `dispatch_<verb>` wrapper
// here. The pattern:
//
// ```rust
// pub async fn dispatch_admit_student(
//     dispatcher: &CommandDispatcher,
//     cmd: educore_academic::commands::AdmitStudentCommand,
// ) -> Result<educore_academic::events::StudentAdmitted, DomainError> {
//     dispatcher.dispatch(cmd, |cmd| async move {
//         educore_academic::services::admit_student(cmd, &clock, &ids, &uniqueness).await
//     }).await
// }
// ```
//
// Implementation deferred per Phase 6 Step 2 deferral pattern; ~358
// service functions across 10 domains each need focused sub-batches.
//
// See `docs/audit_reports/remediation/13-production-readiness-v2.md`
// for the full deferral list.

// ---- Assessment wrappers (Step 2: ~74 wrappers, deferred) ---------------
//
// Same pattern as academic. See comment above.

// ---- Attendance wrappers (Step 2: ~16 wrappers, deferred) ---------------
//
// Same pattern.

// ---- Communication wrappers (Step 3: ~104 wrappers, deferred) ----------
//
// Same pattern.

// ---- Documents wrappers (Step 3: ~18 wrappers, deferred) ---------------
//
// Same pattern.

// ---- Facilities wrappers (Step 3: ~59 wrappers, deferred) --------------
//
// Same pattern.

// ---- Finance wrappers (Step 3: ~66 wrappers, deferred) -----------------
//
// Same pattern.

// ---- HR wrappers (Step 3: ~49 wrappers, deferred) ----------------------
//
// Same pattern.

// ---- Library wrappers (Step 3: ~48 wrappers, deferred) -----------------
//
// Same pattern.

// ---- CMS wrappers (Step 3: ~37 wrappers, deferred) ---------------------
//
// Same pattern.
