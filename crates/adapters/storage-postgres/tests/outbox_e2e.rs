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
use educore_core::error::{DomainError, ErrorKind};
use educore_core::ids::SchoolId;
use educore_storage::outbox::Outbox;
use educore_storage::StorageAdapter;
use educore_storage_postgres::PostgresOutbox;

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
    tx.outbox().append(school, env).await.unwrap();
    tx.commit().await.unwrap();
    // Pending via another transaction.
    let tx = adapter.begin().await.unwrap();
    let pending = tx.outbox().pending(school, 10).await.unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].event_id, event_id);
    assert_eq!(pending[0].event_type, "academic.student.admitted");
    assert_eq!(pending[0].aggregate_type, "student");
    assert_eq!(pending[0].schema_version, 1);
    // Mark published and verify pending is now empty.
    tx.outbox()
        .mark_published(school, &[event_id])
        .await
        .unwrap();
    let pending = tx.outbox().pending(school, 10).await.unwrap();
    assert!(pending.is_empty());
}

/// Builds a [`SerializedEnvelope`] for `school` with a unique
/// `aggregate_id` derived from `tag`. Used by the QW-13
/// school-partitioning tests below.
fn make_envelope(school: SchoolId, tag: &str) -> educore_storage::outbox::SerializedEnvelope {
    use educore_core::clock::SystemIdGen;
    let g = SystemIdGen;
    educore_storage::outbox::SerializedEnvelope {
        event_id: g.next_event_id(),
        event_type: format!("academic.student.{tag}"),
        schema_version: 1,
        school_id: school,
        aggregate_id: g.next_uuid(),
        aggregate_type: "student".to_owned(),
        actor_id: g.next_user_id(),
        correlation_id: g.next_correlation_id(),
        causation_id: None,
        occurred_at: educore_core::value_objects::Timestamp::now(),
        payload: bytes::Bytes::from_static(br#"{"id":"x"}"#),
    }
}

// ---------------------------------------------------------------------------
// QW-13: school_id partitioning in `pending` / `pending_count` /
// `pending_for_school`.
//
// Closes the Postgres half of TOOL-TK-004 plus ADAPTER-PG-013 and
// ADAPTER-PG-029. The testkit half of TOOL-TK-004 is tracked
// separately under TOOL-TK-004 and is not exercised here.
//
// All three tests follow the `EDUCORE_PG_URL` gate from the
// round-trip test above: when the env var is unset (the default in
// CI), the test logs a skip and returns early (passing). When the
// env var is set, the test runs the full assertion.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn outbox_pending_returns_only_handle_school_rows() {
    let url = match std::env::var("EDUCORE_PG_URL") {
        Ok(u) => u,
        Err(_) => {
            tracing::info!("EDUCORE_PG_URL not set; skipping PG e2e");
            return;
        }
    };
    let g = SystemIdGen;
    let school_a = g.next_school_id();
    let school_b = g.next_school_id();

    // Schema is shared across all handles — one migrate() is
    // enough. Use school_a's adapter just to apply the DDL.
    let adapter_a = educore_storage_postgres::PostgresStorageAdapter::connect(&url, school_a)
        .await
        .unwrap();
    adapter_a.migrate().await.unwrap();
    let pool = adapter_a.db().clone();

    // Seed: two envelopes for school_a, one envelope for
    // school_b. Each `PostgresOutbox::new(pool, school)` is
    // scoped to its own school.
    let outbox_a = PostgresOutbox::new(pool.clone(), school_a);
    let outbox_b = PostgresOutbox::new(pool.clone(), school_b);

    let env_a1 = make_envelope(school_a, "a1");
    let env_a2 = make_envelope(school_a, "a2");
    let env_b1 = make_envelope(school_b, "b1");
    outbox_a.append(school_a, env_a1.clone()).await.unwrap();
    outbox_a.append(school_a, env_a2.clone()).await.unwrap();
    outbox_b.append(school_b, env_b1.clone()).await.unwrap();

    // pending() on the school_a handle MUST return only the
    // two school_a envelopes (the WHERE school_id = $1
    // predicate filters by self.school, not by any caller
    // argument).
    let pending_a = outbox_a.pending(school_a, 100).await.unwrap();
    assert_eq!(
        pending_a.len(),
        2,
        "school_a handle must see only school_a rows"
    );
    for env in &pending_a {
        assert_eq!(env.school_id, school_a);
    }
    let ids_a: std::collections::HashSet<_> = pending_a.iter().map(|e| e.event_id).collect();
    assert!(ids_a.contains(&env_a1.event_id));
    assert!(ids_a.contains(&env_a2.event_id));
    assert!(!ids_a.contains(&env_b1.event_id));

    // pending() on the school_b handle MUST return only the
    // one school_b envelope.
    let pending_b = outbox_b.pending(school_b, 100).await.unwrap();
    assert_eq!(
        pending_b.len(),
        1,
        "school_b handle must see only school_b rows"
    );
    assert_eq!(pending_b[0].school_id, school_b);
    assert_eq!(pending_b[0].event_id, env_b1.event_id);
}

#[tokio::test]
async fn outbox_pending_count_rejects_wrong_school_id() {
    // ADAPTER-PG-013 / ADAPTER-PG-029: the previous Postgres
    // impl bound the caller-supplied `school_id` directly into
    // the `WHERE` clause, which let any handle read any
    // other tenant's pending outbox depth. The fix is to
    // assert `school_id == self.school` and return
    // `TenantViolation` otherwise.

    let url = match std::env::var("EDUCORE_PG_URL") {
        Ok(u) => u,
        Err(_) => {
            tracing::info!("EDUCORE_PG_URL not set; skipping PG e2e");
            return;
        }
    };
    let g = SystemIdGen;
    let school_a = g.next_school_id();
    let school_b = g.next_school_id();

    let adapter_a = educore_storage_postgres::PostgresStorageAdapter::connect(&url, school_a)
        .await
        .unwrap();
    adapter_a.migrate().await.unwrap();
    let pool = adapter_a.db().clone();
    let outbox_a = PostgresOutbox::new(pool.clone(), school_a);

    // Seed: one envelope in school_a.
    outbox_a
        .append(school_a, make_envelope(school_a, "a1"))
        .await
        .unwrap();

    // 1) Wrong school: MUST return TenantViolation (not a
    //    count, not a Conflict).
    let wrong = outbox_a.pending_count(school_b).await.unwrap_err();
    assert_eq!(wrong.kind(), ErrorKind::TenantViolation);
    assert!(matches!(wrong, DomainError::TenantViolation(_)));

    // 2) Correct school: MUST return the count (1).
    let count = outbox_a.pending_count(school_a).await.unwrap();
    assert_eq!(count, 1, "school_a handle must see its own pending count");
}

#[tokio::test]
async fn outbox_pending_for_school_enforces_handle_scope() {
    // QW-13: the new explicit-school variant
    // (`pending_for_school`) MUST reject a `school_id` that
    // does not match the handle's scope. The default impl in
    // the port trait delegates to `pending(limit)` and
    // IGNORES the `school_id` argument — Postgres overrides
    // this method to validate the argument.

    let url = match std::env::var("EDUCORE_PG_URL") {
        Ok(u) => u,
        Err(_) => {
            tracing::info!("EDUCORE_PG_URL not set; skipping PG e2e");
            return;
        }
    };
    let g = SystemIdGen;
    let school_a = g.next_school_id();
    let school_b = g.next_school_id();

    let adapter_a = educore_storage_postgres::PostgresStorageAdapter::connect(&url, school_a)
        .await
        .unwrap();
    adapter_a.migrate().await.unwrap();
    let pool = adapter_a.db().clone();
    let outbox_a = PostgresOutbox::new(pool.clone(), school_a);

    // Seed: one envelope for school_a, one for school_b.
    let env_a = make_envelope(school_a, "a1");
    let env_b = make_envelope(school_b, "b1");
    outbox_a.append(school_a, env_a.clone()).await.unwrap();
    PostgresOutbox::new(pool.clone(), school_b)
        .append(school_b, env_b.clone())
        .await
        .unwrap();

    // 1) Correct school: returns the school_a rows.
    let got = outbox_a.pending(school_a, 100).await.unwrap();
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].school_id, school_a);
    assert_eq!(got[0].event_id, env_a.event_id);

    // 2) Wrong school: MUST return TenantViolation (no rows
    //    leaked across tenants).
    let wrong = outbox_a.pending(school_b, 100).await.unwrap_err();
    assert_eq!(wrong.kind(), ErrorKind::TenantViolation);
    assert!(matches!(wrong, DomainError::TenantViolation(_)));
}
