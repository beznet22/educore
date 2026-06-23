//! Outbox end-to-end test against the MySQL adapter.
//!
//! The test is gated on the `EDUCORE_MYSQL_URL` environment
//! variable. When the variable is unset (the default in CI), the
//! test logs a skip notice via `tracing` and returns early
//! (passing). When the variable is set (e.g. a contributor with
//! a local MySQL instance), the test runs the full round-trip
//! and asserts on the engine invariants.
//!
//! To run locally:
//!
//! ```text
//! docker run --rm -d --name educore-mysql -p 3306:3306 \
//!     -e MYSQL_ROOT_PASSWORD=educore -e MYSQL_DATABASE=educore \
//!     -e MYSQL_USER=educore -e MYSQL_PASSWORD=educore \
//!     mysql:8
//! export EDUCORE_MYSQL_URL='mysql://educore:educore@localhost:3306/educore'
//! cargo test -p educore-storage-mysql --test outbox_e2e -- --nocapture
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
    let url = match std::env::var("EDUCORE_MYSQL_URL") {
        Ok(u) => u,
        Err(_) => {
            // Use the tracing crate so the message is routed
            // through the test harness's normal logging. The
            // test still "passes" (returns early with no
            // assertions) so CI is green.
            tracing::info!("EDUCORE_MYSQL_URL not set; skipping MySQL e2e");
            return;
        }
    };
    let g = SystemIdGen;
    let school = g.next_school_id();
    let adapter = educore_storage_mysql::MysqlStorageAdapter::connect(&url, school)
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

/// Closes the MySQL half of audit finding TOOL-TK-004
/// (`wave4-testkit.md`): `Outbox::pending` must return only
/// envelopes for the requester's school, never rows belonging
/// to other tenants in the same database.
///
/// We open two adapters against the same MySQL instance with
/// two distinct `SchoolId`s, append one envelope to each
/// school's outbox, and then verify that each adapter's
/// `pending` drain returns only its own school's row. The
/// `mark_published` check at the end also covers the
/// defense-in-depth `school_id` filter on the UPDATE path
/// (a relay in school B must not be able to flip the
/// `published_at` of school A's row even if it somehow learns
/// the `event_id`).
#[tokio::test]
async fn outbox_pending_is_partitioned_by_school() {
    let url = match std::env::var("EDUCORE_MYSQL_URL") {
        Ok(u) => u,
        Err(_) => {
            tracing::info!("EDUCORE_MYSQL_URL not set; skipping MySQL e2e");
            return;
        }
    };
    let g = SystemIdGen;
    let school_a = g.next_school_id();
    let school_b = g.next_school_id();
    assert_ne!(school_a, school_b, "test fixture: schools must differ");

    // Two adapters, same MySQL URL, two different schools.
    // `migrate()` is idempotent so calling it on both is safe.
    let adapter_a = educore_storage_mysql::MysqlStorageAdapter::connect(&url, school_a)
        .await
        .unwrap();
    adapter_a.migrate().await.unwrap();
    let adapter_b = educore_storage_mysql::MysqlStorageAdapter::connect(&url, school_b)
        .await
        .unwrap();
    adapter_b.migrate().await.unwrap();

    // Build one envelope per school.
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
        payload: bytes::Bytes::from_static(br#"{"school":"a"}"#),
    };
    let event_id_a = env_a.event_id;
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
        payload: bytes::Bytes::from_static(br#"{"school":"b"}"#),
    };
    let event_id_b = env_b.event_id;

    // Append each envelope in its own school's transaction.
    let tx = adapter_a.begin().await.unwrap();
    tx.outbox().append(env_a).await.unwrap();
    tx.commit().await.unwrap();
    let tx = adapter_b.begin().await.unwrap();
    tx.outbox().append(env_b).await.unwrap();
    tx.commit().await.unwrap();

    // School A drain: must contain event_a, must NOT contain event_b.
    let tx = adapter_a.begin().await.unwrap();
    let pending_a = tx.outbox().pending(10).await.unwrap();
    assert!(
        pending_a.iter().any(|e| e.event_id == event_id_a),
        "school A must see its own envelope after pending() drain"
    );
    assert!(
        pending_a.iter().all(|e| e.school_id == school_a),
        "school A pending() must contain only school A rows"
    );
    assert!(
        pending_a.iter().all(|e| e.event_id != event_id_b),
        "school A pending() must not leak school B's envelope"
    );
    tx.commit().await.unwrap();

    // School B drain: must contain event_b, must NOT contain event_a.
    let tx = adapter_b.begin().await.unwrap();
    let pending_b = tx.outbox().pending(10).await.unwrap();
    assert!(
        pending_b.iter().any(|e| e.event_id == event_id_b),
        "school B must see its own envelope after pending() drain"
    );
    assert!(
        pending_b.iter().all(|e| e.school_id == school_b),
        "school B pending() must contain only school B rows"
    );
    assert!(
        pending_b.iter().all(|e| e.event_id != event_id_a),
        "school B pending() must not leak school A's envelope"
    );
    tx.commit().await.unwrap();

    // Defense in depth: `mark_published` from school B against
    // school A's `event_id` must NOT flip school A's row. The
    // `WHERE school_id = ?` predicate on the UPDATE closes that
    // gap (relay cross-tenant idempotency attack).
    let tx = adapter_b.begin().await.unwrap();
    tx.outbox().mark_published(&[event_id_a]).await.unwrap();
    tx.commit().await.unwrap();

    let tx = adapter_a.begin().await.unwrap();
    let pending_a_after = tx.outbox().pending(10).await.unwrap();
    assert!(
        pending_a_after.iter().any(|e| e.event_id == event_id_a),
        "school B's mark_published must not affect school A's outbox row"
    );
    tx.commit().await.unwrap();
}
