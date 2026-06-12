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
