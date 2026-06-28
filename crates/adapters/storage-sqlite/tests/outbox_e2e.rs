//! Outbox end-to-end test against the SQLite adapter.
//!
//! Mirrors the SurrealDB `outbox_e2e.rs` Phase 0 test
//! verbatim. SQLite is the engine's embedded / offline mode
//! (per `docs/schemas/sql-dialects/sqlite.md`), so the test
//! always runs in CI without any external infrastructure.

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
    let adapter = educore_storage_sqlite::SqliteStorageAdapter::in_memory(school)
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

/// QW-13 (SQLite) regression test for ADAPTER-SQ-OUTBOX-001:
/// `Outbox::pending` and `Outbox::pending_count` must be
/// partitioned by `school_id`, never returning rows belonging
/// to a different tenant. Mirrors the audit finding
/// `TOOL-TK-004` (testkit side of QW-13).
///
/// This test opens two adapters against the same file-backed
/// SQLite database (different `SchoolId`s) and verifies that
/// each adapter's `pending` / `pending_count` only sees rows
/// for its own school. `in_memory` adapters are not viable
/// here because each call constructs a fresh
/// `sqlite::memory:` engine — the two adapters would not
/// share state. The test cleans up its temp file on exit.
#[tokio::test]
async fn outbox_pending_is_partitioned_by_school() {
    use educore_storage_sqlite::{SqliteConnection, SqliteStorageAdapter};

    let g = SystemIdGen;
    let school_a = g.next_school_id();
    let school_b = g.next_school_id();
    assert_ne!(school_a, school_b, "schools must differ");

    // Unique file path per run: pid + monotonic nanos keeps
    // parallel test invocations independent.
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let path = std::env::temp_dir().join(format!(
        "educore-qw13-sqlite-outbox-{}-{}.db",
        std::process::id(),
        nonce
    ));
    let url = format!("sqlite://{}?mode=rwc", path.display());
    // Helper: build an adapter scoped to `school` against the
    // shared file. `SqliteConnection::connect` sets WAL +
    // NORMAL + foreign_keys on every new connection, so two
    // connections to the same file coexist cleanly under WAL.
    async fn adapter_for(url: &str, school: educore_core::ids::SchoolId) -> SqliteStorageAdapter {
        let conn = SqliteConnection::connect(url, school).await.unwrap();
        SqliteStorageAdapter::new(conn)
    }

    let adapter_a = adapter_for(&url, school_a).await;
    let adapter_b = adapter_for(&url, school_b).await;

    // Migrate the schema once. The DDL is `IF NOT EXISTS` so
    // the second `migrate()` is a no-op for tables and
    // indexes (and `ensure_schema()` for bulk_attendance is
    // idempotent).
    adapter_a.migrate().await.unwrap();
    adapter_b.migrate().await.unwrap();

    // Each adapter appends one envelope.
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
    let env_b = educore_storage::outbox::SerializedEnvelope {
        event_id: g.next_event_id(),
        event_type: "hr.staff.hired".to_owned(),
        schema_version: 1,
        school_id: school_b,
        aggregate_id: g.next_uuid(),
        aggregate_type: "staff".to_owned(),
        actor_id: g.next_user_id(),
        correlation_id: g.next_correlation_id(),
        causation_id: None,
        occurred_at: educore_core::value_objects::Timestamp::now(),
        payload: bytes::Bytes::from_static(br#"{"school":"b"}"#),
    };
    let event_id_a = env_a.event_id;
    let event_id_b = env_b.event_id;

    // Append via adapter_a's transaction.
    let tx = adapter_a.begin().await.unwrap();
    tx.outbox().append(school_a, env_a).await.unwrap();
    tx.commit().await.unwrap();
    // Append via adapter_b's transaction.
    let tx = adapter_b.begin().await.unwrap();
    tx.outbox().append(school_b, env_b).await.unwrap();
    tx.commit().await.unwrap();

    // Cross-school isolation: adapter_a only sees school_a's
    // row, adapter_b only sees school_b's row. `pending`
    // enforces the partition internally via `self.school`
    // (the school bound at adapter construction); per QW-13 /
    // ADAPTER-SQ-OUTBOX-001 + ADAPTER-SQ-OUTBOX-002 the
    // explicit-school variants (`pending_for_school`,
    // `pending_count`) MUST reject any caller-supplied
    // `school_id` that doesn't match the handle's scope with
    // `DomainError::TenantViolation`.
    let tx = adapter_a.begin().await.unwrap();
    let pending_a = tx.outbox().pending(school_a, 10).await.unwrap();
    assert_eq!(
        pending_a.len(),
        1,
        "adapter_a.pending must return exactly one row (school_a's)"
    );
    assert_eq!(pending_a[0].event_id, event_id_a);
    assert_eq!(pending_a[0].school_id, school_a);
    assert_eq!(
        tx.outbox().pending_count(school_a).await.unwrap(),
        1,
        "adapter_a.pending_count(school_a) must be 1"
    );
    // Cross-tenant probe must be rejected with
    // `TenantViolation`, NOT silently return the other
    // tenant's count.
    let cross = tx.outbox().pending_count(school_b).await;
    assert!(
        matches!(cross, Err(educore_core::error::DomainError::TenantViolation(_))),
        "adapter_a.pending_count(school_b) must reject cross-tenant probe with TenantViolation, got {cross:?}"
    );
    let cross = tx.outbox().pending(school_b, 10).await;
    assert!(
        matches!(cross, Err(educore_core::error::DomainError::TenantViolation(_))),
        "adapter_a.pending(school_b, _) must reject cross-tenant probe with TenantViolation, got {cross:?}"
    );
    // Matching-school explicit-school path works.
    let pending_a_explicit = tx.outbox().pending(school_a, 10).await.unwrap();
    assert_eq!(pending_a_explicit.len(), 1);
    assert_eq!(pending_a_explicit[0].event_id, event_id_a);
    drop(tx);

    let tx = adapter_b.begin().await.unwrap();
    let pending_b = tx.outbox().pending(school_b, 10).await.unwrap();
    assert_eq!(
        pending_b.len(),
        1,
        "adapter_b.pending must return exactly one row (school_b's)"
    );
    assert_eq!(pending_b[0].event_id, event_id_b);
    assert_eq!(pending_b[0].school_id, school_b);
    assert_eq!(
        tx.outbox().pending_count(school_b).await.unwrap(),
        1,
        "adapter_b.pending_count(school_b) must be 1"
    );
    let cross = tx.outbox().pending_count(school_a).await;
    assert!(
        matches!(cross, Err(educore_core::error::DomainError::TenantViolation(_))),
        "adapter_b.pending_count(school_a) must reject cross-tenant probe with TenantViolation, got {cross:?}"
    );
    drop(tx);

    // Mark adapter_a's row published via adapter_a; adapter_b's
    // row must remain pending.
    let tx = adapter_a.begin().await.unwrap();
    tx.outbox()
        .mark_published(school_a, &[event_id_a])
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let tx = adapter_a.begin().await.unwrap();
    assert!(
        tx.outbox().pending(school_a, 10).await.unwrap().is_empty(),
        "adapter_a must see no pending rows after mark_published"
    );
    let tx = adapter_b.begin().await.unwrap();
    let still_pending = tx.outbox().pending(school_b, 10).await.unwrap();
    assert_eq!(
        still_pending.len(),
        1,
        "adapter_b must still see school_b's row after adapter_a marked its own row"
    );
    assert_eq!(still_pending[0].event_id, event_id_b);

    // Clean up the temp file. WAL mode produces `-wal` and
    // `-shm` sidecar files.
    drop(tx);
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(path.with_extension("db-wal"));
    let _ = std::fs::remove_file(path.with_extension("db-shm"));
}
