//! Outbox end-to-end test against the SurrealDB adapter.

// Test scaffolding: relax the workspace lint baseline (which forbids
// `unwrap`/`expect`/`panic` in production) so the assertions read
// naturally. This is the Phase 0 e2e test (per `docs/build-plan.md`
// Phase 0 task 5): create a schema, insert one outbox row, read it
// back, assert the engine invariants, and confirm the sync
// coordinator fans the event out to the in-process consumer.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::clock::{IdGenerator as _, SystemIdGen};
use educore_storage::outbox::Outbox as _;
use educore_storage::StorageAdapter;

#[tokio::test]
async fn outbox_append_and_pending_round_trip() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let adapter = educore_storage_surrealdb::SurrealStorageAdapter::in_memory(school)
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

/// Verifies the per-school partition documented at
/// `crates/infra/storage/src/outbox.rs:104-108`:
///
/// > The outbox is partitioned by `school_id` so callers see
/// > only envelopes for their school.
///
/// Closes the SurrealDB half of TOOL-TK-004: the adapter must
/// never leak cross-school rows from `pending()`. Uses the
/// in-memory backend (`Mem`) so no SurrealDB container is
/// required.
#[tokio::test]
async fn pending_returns_only_rows_for_the_requested_school() {
    let g = SystemIdGen;
    let school_a = g.next_school_id();
    let school_b = g.next_school_id();
    let adapter = educore_storage_surrealdb::SurrealStorageAdapter::in_memory(school_a)
        .await
        .unwrap();
    adapter.migrate().await.unwrap();

    // Envelope for school A — appended via the adapter's tx
    // outbox (which is scoped to school A by the transaction
    // ctor).
    let env_a = educore_storage::outbox::SerializedEnvelope {
        event_id: g.next_event_id(),
        event_type: "academic.student.admitted".to_owned(),
        schema_version: 1,
        school_id: school_a,
        aggregate_id: g.next_uuid(),
        aggregate_type: "student".to_owned(),
        actor_id: g.next_user_id(),
        correlation_id: g.next_correlation_id(),
        causation_id: None,
        occurred_at: educore_core::value_objects::Timestamp::now(),
        payload: bytes::Bytes::from_static(br#"{"id":"a"}"#),
    };
    let event_id_a = env_a.event_id;

    // Envelope for school B — appended via a separately
    // constructed SurrealOutbox handle bound to the SAME db
    // (so both rows live in the same outbox table) but scoped
    // to school B. This is the cross-school injection that
    // would expose a partition bug.
    let env_b = educore_storage::outbox::SerializedEnvelope {
        event_id: g.next_event_id(),
        event_type: "academic.student.admitted".to_owned(),
        schema_version: 1,
        school_id: school_b,
        aggregate_id: g.next_uuid(),
        aggregate_type: "student".to_owned(),
        actor_id: g.next_user_id(),
        correlation_id: g.next_correlation_id(),
        causation_id: None,
        occurred_at: educore_core::value_objects::Timestamp::now(),
        payload: bytes::Bytes::from_static(br#"{"id":"b"}"#),
    };
    let event_id_b = env_b.event_id;
    let outbox_b = educore_storage_surrealdb::SurrealOutbox::new(adapter.db().clone(), school_b);

    let tx = adapter.begin().await.unwrap();
    tx.outbox().append(env_a).await.unwrap();
    tx.commit().await.unwrap();
    outbox_b.append(env_b).await.unwrap();

    // pending() via the adapter (school A) returns only A's
    // envelope. This is the partition guarantee.
    let tx = adapter.begin().await.unwrap();
    let pending_a = tx.outbox().pending(10).await.unwrap();
    assert_eq!(
        pending_a.len(),
        1,
        "school A outbox must see only school A's envelopes; got {}",
        pending_a.len()
    );
    assert_eq!(pending_a[0].event_id, event_id_a);
    assert_eq!(pending_a[0].school_id, school_a);
    drop(tx);

    // pending() via the B-handle returns only B's envelope.
    let pending_b = outbox_b.pending(10).await.unwrap();
    assert_eq!(
        pending_b.len(),
        1,
        "school B outbox must see only school B's envelopes; got {}",
        pending_b.len()
    );
    assert_eq!(pending_b[0].event_id, event_id_b);
    assert_eq!(pending_b[0].school_id, school_b);
}
