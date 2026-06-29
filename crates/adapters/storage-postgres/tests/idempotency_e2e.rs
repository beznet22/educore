//! `Idempotency::record_outcome` end-to-end tests for the
//! PostgreSQL adapter (PR 4 Phase B / QW-12).
//!
//! Closes audit finding ADAPTER-PG-009. The tests are gated on
//! the `EDUCORE_PG_URL` environment variable, matching the
//! pattern in `outbox_e2e.rs`: when the variable is unset (the
//! default in CI), the tests log a skip notice and return early
//! (passing). When the variable is set, the tests run the full
//! round-trip and assert on the engine invariants.
//!
//! To run locally:
//!
//! ```text
//! docker run --rm -d --name educore-pg -p 5432:5432 \
//!     -e POSTGRES_USER=educore -e POSTGRES_PASSWORD=educore \
//!     -e POSTGRES_DB=educore postgres:16
//! export EDUCORE_PG_URL=postgres://educore:educore@localhost:5432/educore
//! cargo test -p educore-storage-postgres --test idempotency_e2e -- --nocapture
//! ```

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    clippy::print_stderr,
    missing_docs
)]

use bytes::Bytes;
use educore_core::clock::{IdGenerator as _, SystemIdGen};
use educore_core::value_objects::Timestamp;
use educore_storage::idempotency::{Idempotency, IdempotencyOutcome, IdempotencyRecord};
use educore_storage::StorageAdapter;

/// Skip-with-notice helper. Returns `Some(())` when the
/// `EDUCORE_PG_URL` env var is set, `None` otherwise. The
/// notice is routed through `tracing` so it shows up under the
/// test harness's normal logging.
fn pg_url_or_skip(test_name: &str) -> Option<String> {
    match std::env::var("EDUCORE_PG_URL") {
        Ok(u) => Some(u),
        Err(_) => {
            tracing::info!(
                test = test_name,
                "EDUCORE_PG_URL not set; skipping PG idempotency e2e",
            );
            None
        }
    }
}

/// Build an `IdempotencyRecord` whose composite key is unique
/// to this test invocation (the `SystemIdGen` produces
/// fresh UUIDv7 ids) and whose outcome bytes are `payload`.
fn make_record(
    g: &SystemIdGen,
    school: educore_core::ids::SchoolId,
    command_type: &'static str,
    payload: &'static [u8],
) -> IdempotencyRecord {
    IdempotencyRecord {
        school_id: school,
        command_type,
        idempotency_key: g.next_idempotency_key(),
        outcome: Bytes::from_static(payload),
        outcome_version: 1,
        recorded_at: Timestamp::now(),
        affected_aggregate_ids: Vec::new(),
        aggregate_version: 1,
        etag: None,
        duration_ms: 0,
        emitted_event_ids: Vec::new(),
    }
}

/// QW-12 (Phase B) test 1: a fresh composite key MUST yield
/// `IdempotencyOutcome::Recorded` on the first write. Verifies
/// that the new `INSERT`-without-`ON CONFLICT` path produces a
/// successful row, not a phantom duplicate-key error.
#[tokio::test]
async fn record_outcome_returns_recorded_for_new_key() {
    let Some(url) = pg_url_or_skip("record_outcome_returns_recorded_for_new_key") else {
        return;
    };

    let g = SystemIdGen;
    let school = g.next_school_id();
    let adapter = educore_storage_postgres::PostgresStorageAdapter::connect(&url, school)
        .await
        .unwrap();
    adapter.migrate().await.unwrap();

    let pool = adapter.db().clone();
    let idem = educore_storage_postgres::PostgresIdempotency::new(pool, school);

    let record = make_record(&g, school, "academic.student.admit", b"first-payload");

    let outcome = idem
        .record_outcome(record.clone())
        .await
        .expect("record_outcome on a fresh key must not fail");
    assert_eq!(
        outcome,
        IdempotencyOutcome::Recorded,
        "first write with a fresh composite key must report Recorded",
    );

    // Sanity: the row is now readable via lookup, with the
    // exact outcome bytes we just wrote (verifies the
    // envelope round-trip is intact).
    let key = IdempotencyRecord::composite_key(
        record.school_id,
        record.command_type,
        record.idempotency_key,
    );
    let fetched = idem
        .lookup(key)
        .await
        .expect("lookup must not fail")
        .expect("row written by record_outcome must be readable");
    assert_eq!(
        fetched.outcome,
        Bytes::from_static(b"first-payload"),
        "lookup must return the same outcome bytes that were written",
    );
}

/// QW-12 (Phase B) test 2: a duplicate composite key with a
/// **different** outcome MUST yield
/// `IdempotencyOutcome::Conflict { existing }`, where
/// `existing` carries the **originally-recorded** payload (not
/// the rejected second payload). This is the audit finding
/// ADAPTER-PG-009: the prior `record()` impl silently
/// overwrote via `ON CONFLICT DO NOTHING`, which the engine's
/// dispatcher cannot distinguish from a successful write and
/// which corrupts the at-least-once retry semantics.
#[tokio::test]
async fn record_outcome_returns_conflict_for_duplicate_key() {
    let Some(url) = pg_url_or_skip("record_outcome_returns_conflict_for_duplicate_key") else {
        return;
    };

    let g = SystemIdGen;
    let school = g.next_school_id();
    let adapter = educore_storage_postgres::PostgresStorageAdapter::connect(&url, school)
        .await
        .unwrap();
    adapter.migrate().await.unwrap();

    let pool = adapter.db().clone();
    let idem = educore_storage_postgres::PostgresIdempotency::new(pool, school);

    // Build the first record with a fresh composite key and
    // the canonical "first" payload.
    let first = make_record(&g, school, "academic.student.admit", b"first-payload");

    // Clone the composite key (school_id + command_type +
    // idempotency_key) onto a second record with a different
    // outcome payload — this is the duplicate-with-different-
    // outcome case the port contract surfaces as `Conflict`.
    let second = IdempotencyRecord {
        outcome: Bytes::from_static(b"second-payload"),
        recorded_at: Timestamp::now(),
        ..first.clone()
    };

    // First write: Recorded.
    let first_outcome = idem
        .record_outcome(first.clone())
        .await
        .expect("first record_outcome must not fail");
    assert_eq!(
        first_outcome,
        IdempotencyOutcome::Recorded,
        "first write must report Recorded",
    );

    // Second write with the same composite key but a
    // different outcome: MUST surface as Conflict carrying
    // the original payload.
    let second_outcome = idem.record_outcome(second).await.expect(
        "second record_outcome must not fail (duplicate is a business outcome, not an error)",
    );
    match second_outcome {
        IdempotencyOutcome::Conflict { existing } => {
            assert_eq!(
                existing.outcome,
                Bytes::from_static(b"first-payload"),
                "Conflict::existing must carry the ORIGINAL outcome bytes \
                 (the first write), not the rejected second payload",
            );
            assert_eq!(
                existing.outcome_version, 1,
                "Conflict::existing must carry the original outcome_version",
            );
            assert_eq!(
                existing.school_id, school,
                "Conflict::existing must carry the original school_id",
            );
            assert_eq!(
                existing.idempotency_key, first.idempotency_key,
                "Conflict::existing must carry the original idempotency_key",
            );
            assert_eq!(
                existing.command_type, "academic.student.admit",
                "Conflict::existing must carry the original command_type",
            );
        }
        other => panic!(
            "expected IdempotencyOutcome::Conflict on duplicate key \
             with different outcome, got {other:?}",
        ),
    }

    // Sanity: the underlying row was NOT overwritten by the
    // rejected second write. The engine relies on this — the
    // rejected write must not corrupt the canonical record.
    let key = IdempotencyRecord::composite_key(
        first.school_id,
        first.command_type,
        first.idempotency_key,
    );
    let fetched = idem
        .lookup(key)
        .await
        .expect("lookup must not fail")
        .expect("row must still be present after a duplicate-key write");
    assert_eq!(
        fetched.outcome,
        Bytes::from_static(b"first-payload"),
        "the rejected second write must NOT overwrite the original outcome bytes",
    );
}

/// QW-12 (Phase B) bonus test: a no-op re-insert (same
/// composite key, same outcome bytes) MUST be reported as
/// `Recorded`, NOT `Conflict`. The engine relies on this case
/// for at-least-once delivery of retries — a duplicate
/// dispatch with an identical payload is a successful retry,
/// not a business-level conflict. Mirrors the port-trait
/// regression test but exercises the Postgres override path.
#[tokio::test]
async fn record_outcome_returns_recorded_for_no_op_reinsert() {
    let Some(url) = pg_url_or_skip("record_outcome_returns_recorded_for_no_op_reinsert") else {
        return;
    };

    let g = SystemIdGen;
    let school = g.next_school_id();
    let adapter = educore_storage_postgres::PostgresStorageAdapter::connect(&url, school)
        .await
        .unwrap();
    adapter.migrate().await.unwrap();

    let pool = adapter.db().clone();
    let idem = educore_storage_postgres::PostgresIdempotency::new(pool, school);

    let record = make_record(&g, school, "academic.student.admit", b"identical-payload");

    let first = idem
        .record_outcome(record.clone())
        .await
        .expect("first record_outcome must not fail");
    assert_eq!(first, IdempotencyOutcome::Recorded);

    // Same composite key + same outcome bytes → Recorded.
    let second = idem
        .record_outcome(record)
        .await
        .expect("no-op reinsert must not fail");
    assert_eq!(
        second,
        IdempotencyOutcome::Recorded,
        "no-op reinsert (same key + same outcome) must report Recorded, \
         not Conflict — engine relies on this for retry replay",
    );
}
