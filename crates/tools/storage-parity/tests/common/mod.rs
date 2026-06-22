//! Shared helpers for the storage-parity test suite.
//!
//! Each test file in `tests/parity_*.rs` and `tests/port_*_integration.rs`
//! uses these helpers to construct a per-backend storage adapter (or
//! port-adapter impl), generate a fresh `SchoolId`, and mint a
//! `TenantContext`. Backends are:
//!
//! - **testkit** — always-on, in-process, the test surface for
//!   unit / integration tests that do not need a real DB.
//! - **sqlite** — always-on, in-memory SQLite.
//! - **surrealdb** — always-on, in-memory SurrealDB (`Mem`
//!   backend).
//! - **postgres** — env-gated on `EDUCORE_PG_URL`.
//! - **mysql** — env-gated on `EDUCORE_MYSQL_URL`.
//!
//! The pattern matches the existing `cross_cutting_integration.rs`
//! `setup_sqlite` / `dispatch_create_school` helpers; this module
//! lifts the boilerplate so the 7 parity scenarios and the 5 port
//! integration tests can share it.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use std::sync::Arc;

use educore_core::clock::{IdGenerator, SystemIdGen};
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_event_bus::InProcessEventBus;
use educore_events::event_bus::EventBus;
use educore_storage::event_log::EventLogEntry;
use educore_storage::StorageAdapter;

// ---------------------------------------------------------------------------
// TenantContext / SchoolId helpers
// ---------------------------------------------------------------------------

/// Builds a `TenantContext` for `school` with a freshly-minted
/// actor + correlation id. Convenience so callers do not need to
/// thread the same boilerplate through every test.
#[must_use]
pub fn make_ctx(school: SchoolId) -> TenantContext {
    let g = SystemIdGen;
    TenantContext::for_user(
        school,
        g.next_user_id(),
        g.next_correlation_id(),
        UserType::SchoolAdmin,
    )
}

/// Drains the outbox into the event log. Mirrors the existing
/// `cross_cutting_integration::relay_outbox_to_event_log` helper.
/// Inlined here so the new parity tests can stand on their own.
pub async fn relay_outbox_to_event_log(adapter: &dyn StorageAdapter) {
    let tx = adapter.begin().await.expect("begin");
    let pending = tx.outbox().pending(100).await.expect("pending");
    for env in &pending {
        let entry = EventLogEntry::from_serialized_envelope(env);
        tx.event_log()
            .append(entry)
            .await
            .expect("event_log append");
        tx.outbox()
            .mark_published(&[env.event_id])
            .await
            .expect("mark_published");
    }
    tx.commit().await.expect("commit");
}

// ---------------------------------------------------------------------------
// Backend setup helpers
// ---------------------------------------------------------------------------

/// Always-on setup for the `testkit` in-memory backend.
#[must_use]
pub fn setup_testkit() -> (Arc<dyn StorageAdapter>, SchoolId, TenantContext) {
    let bus: Arc<dyn EventBus> = Arc::new(InProcessEventBus::new());
    let adapter = educore_testkit::storage::InMemoryStorageAdapter::new(bus);
    let g = SystemIdGen;
    let school = g.next_school_id();
    let ctx = make_ctx(school);
    (Arc::new(adapter), school, ctx)
}

/// Always-on setup for the SQLite in-memory backend.
pub async fn setup_sqlite() -> (Arc<dyn StorageAdapter>, SchoolId, TenantContext) {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let adapter = educore_storage_sqlite::SqliteStorageAdapter::in_memory(school)
        .await
        .expect("in-memory sqlite");
    adapter.migrate().await.expect("migrate sqlite");
    let ctx = make_ctx(school);
    (Arc::new(adapter), school, ctx)
}

/// Always-on setup for the SurrealDB in-memory backend (Phase 0
/// primary per ADR-017).
pub async fn setup_surrealdb() -> (Arc<dyn StorageAdapter>, SchoolId, TenantContext) {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let adapter = educore_storage_surrealdb::SurrealStorageAdapter::in_memory(school)
        .await
        .expect("in-memory surrealdb");
    adapter.migrate().await.expect("migrate surrealdb");
    let ctx = make_ctx(school);
    (Arc::new(adapter), school, ctx)
}

/// Env-gated setup for the PostgreSQL backend. Returns `None`
/// when `EDUCORE_PG_URL` is unset; the caller should bail out
/// (the test is skipped via `#[ignore]` + `match`).
pub async fn setup_pg() -> Option<(Arc<dyn StorageAdapter>, SchoolId, TenantContext)> {
    let url = match std::env::var("EDUCORE_PG_URL") {
        Ok(s) if !s.is_empty() => s,
        _ => return None,
    };
    let g = SystemIdGen;
    let school = g.next_school_id();
    let adapter = educore_storage_postgres::PostgresStorageAdapter::connect(&url, school)
        .await
        .expect("connect pg");
    adapter.migrate().await.expect("migrate pg");
    let ctx = make_ctx(school);
    Some((Arc::new(adapter), school, ctx))
}

/// Env-gated setup for the MySQL backend. Returns `None` when
/// `EDUCORE_MYSQL_URL` is unset; the caller should bail out.
pub async fn setup_mysql() -> Option<(Arc<dyn StorageAdapter>, SchoolId, TenantContext)> {
    let url = match std::env::var("EDUCORE_MYSQL_URL") {
        Ok(s) if !s.is_empty() => s,
        _ => return None,
    };
    let g = SystemIdGen;
    let school = g.next_school_id();
    let adapter = educore_storage_mysql::MysqlStorageAdapter::connect(&url, school)
        .await
        .expect("connect mysql");
    adapter.migrate().await.expect("migrate mysql");
    let ctx = make_ctx(school);
    Some((Arc::new(adapter), school, ctx))
}
