//! Outbox end-to-end test against the PostgreSQL adapter.
//!
//! The test is gated on the `EDUCORE_PG_URL` environment
//! variable. When the variable is unset (the default in CI), the
//! test logs a skip notice via `tracing` and returns early
//! (passing). When the variable is set (e.g. a contributor with
//! a local PostgreSQL instance), the test runs the full
//! round-trip and asserts on the engine invariants.
//!
//! To run locally:
//!
//! ```text
//! docker run --rm -d --name educore-pg -p 5432:5432 \
//!     -e POSTGRES_USER=educore -e POSTGRES_PASSWORD=educore \
//!     -e POSTGRES_DB=educore postgres:16
//! export EDUCORE_PG_URL=postgres://educore:educore@localhost:5432/educore
//! cargo test -p educore-storage-postgres --test outbox_e2e -- --nocapture
//! ```

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    clippy::print_stderr,
    missing_docs
)]

use educore_core::clock::{IdGenerator as _, SystemIdGen};
use educore_storage::StorageAdapter;

#[tokio::test]
async fn outbox_append_and_pending_round_trip() {
    let url = match std::env::var("EDUCORE_PG_URL") {
        Ok(u) => u,
        Err(_) => {
            // Use the tracing crate so the message is routed
            // through the test harness's normal logging. The
            // test still "passes" (returns early with no
            // assertions) so CI is green.
            tracing::info!("EDUCORE_PG_URL not set; skipping PG e2e");
            return;
        }
    };
    let g = SystemIdGen;
    let school = g.next_school_id();
    let adapter = educore_storage_postgres::PostgresStorageAdapter::connect(&url, school)
        .await
        .unwrap();
    adapter.migrate().await.unwrap();
    let env = educore_storage::outbox::SerializedEnvelope {
        event_id: g.next_event_id(),
        event_type: "academic.student.admitted".to_owned(),
        schema_version: 1,
        school_id: school,
        aggregate_id: g.next_uuid(),
        aggregate_type: "student".to_owned(),
        actor_id: g.next_user_id(),
        correlation_id: g.next_correlation_id(),
        causation_id: None,
        occurred_at: educore_core::value_objects::Timestamp::now(),
        payload: bytes::Bytes::from_static(br#"{"id":"x"}"#),
    };
    let event_id = env.event_id;
    // Append via a transaction.
    let tx = adapter.begin().await.unwrap();
    tx.outbox().append(env).await.unwrap();
    tx.commit().await.unwrap();
    // Pending via another transaction.
    let tx = adapter.begin().await.unwrap();
    let pending = tx.outbox().pending(10).await.unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].event_id, event_id);
    assert_eq!(pending[0].event_type, "academic.student.admitted");
    assert_eq!(pending[0].aggregate_type, "student");
    assert_eq!(pending[0].schema_version, 1);
    // Mark published and verify pending is now empty.
    tx.outbox().mark_published(&[event_id]).await.unwrap();
    let pending = tx.outbox().pending(10).await.unwrap();
    assert!(pending.is_empty());
}
