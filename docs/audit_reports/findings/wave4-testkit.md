## Wave 4 Tools Audit Report — `educore-testkit`

**Scope:** `crates/tools/testkit/`, `docs/handoff/PHASE-16-HANDOFF.md`, `docs/build-plan.md:1646-1702` (Phase 16 task 1), `docs/ports/{storage,authentication,notifications,payments,file-storage,integrations,event-bus}.md`, `AGENTS.md` engine rules 4-8.

**Total findings:** 28

---

### FINDING 1

- **id:** TOOL-TK-001
- **area:** tools
- **severity:** Critical
- **location:** `crates/tools/testkit/src/storage.rs:431-452`
- **description:** `InMemoryTransaction::commit` drains the outbox into a local `_pending` Vec and drops it. It never publishes the drained envelopes to the event bus even though the `bus` field is wired to `InProcessEventBus`. As a result, every domain command that writes to the outbox via the testkit adapter emits zero downstream events. The comment at lines 441-446 explicitly admits this: "the in-memory testkit does not republish envelopes to the bus". Any integration test that asserts "after `tx.commit()`, a subscriber on `world.bus` receives the event" will fail silently.
- **expected:** Per `docs/ports/storage.md:104-108`: "Every state change is written to the outbox in the same transaction as the aggregate mutation. A separate relay reads pending events and publishes them to the event bus. Consumers see at-least-once delivery." Per `docs/build-plan.md:1653-1656`: "in-memory impls of all 6 ports… Consumer tests use these to run domain commands without docker" — implying the in-memory world should preserve end-to-end event semantics.
- **evidence:**
  ```rust
  crates/tools/testkit/src/storage.rs:431-452
  #[async_trait]
  impl Transaction for InMemoryTransaction {
      async fn commit(self: Box<Self>) -> Result<()> {
          if self.rolled_back.load(Ordering::SeqCst) {
              return Err(DomainError::validation("transaction already rolled back"));
          }
          if self.committed.swap(true, Ordering::SeqCst) {
              return Err(DomainError::validation("transaction already committed"));
          }
          // Drain the outbox; the in-memory testkit does not
          // republish envelopes to the bus (the SerializedEnvelope
          // shape uses owned Strings while EventEnvelope expects
          // &'static str, so a strict conversion is awkward). The
          // outbox-drain test asserts that the outbox is empty
          // after commit; the bus is exercised separately by tests
          // that publish directly via the bus port.
          let _pending: Vec<SerializedEnvelope> = {
              let mut outbox = self.inner.outbox.lock();
              outbox.drain(..).collect()
          };
          Ok(())
      }
  ```

---

### FINDING 2

- **id:** TOOL-TK-002
- **area:** tools
- **severity:** Critical
- **location:** `crates/tools/testkit/src/storage.rs:454-461`
- **description:** `InMemoryTransaction::rollback` only flips the `rolled_back` `AtomicBool`; it does NOT discard any staged writes. All sub-port handles (`OutboxHandle::append`, `AuditLogHandle::append`, `EventLogHandle::append`, `IdempotencyHandle::record`) write directly to the shared `Arc<InMemoryInner>` state at call time (lines 86-218). So a rollback does not roll back; subsequent transactions observe the rolled-back writes.
- **expected:** Per `docs/ports/storage.md` and the `Transaction` trait doc at `crates/infra/storage/src/transaction.rs:45-47`: "Rolls the transaction back. All staged writes are discarded. Consumes the transaction."
- **evidence:**
  ```rust
  crates/tools/testkit/src/storage.rs:454-461
  async fn rollback(self: Box<Self>) -> Result<()> {
      if self.committed.load(Ordering::SeqCst) {
          return Err(DomainError::validation("transaction already committed"));
      }
      self.rolled_back.store(true, Ordering::SeqCst);
      Ok(())
  }
  ```
  And the test that codifies the broken behavior:
  ```rust
  crates/tools/testkit/src/storage.rs:647-663
  #[test]
  fn begin_rollback_discards_outbox() {
      ...
      tx.outbox().append(sample_envelope(school)).await.unwrap();
      tx.rollback().await.unwrap();
      let tx2 = adapter.begin().await.unwrap();
      let pending = tx2.outbox().pending(10).await.unwrap();
      // The first tx was rolled back so the outbox still
      // has the envelope; the second tx sees it.
      assert_eq!(pending.len(), 1);
  }
  ```

---

### FINDING 3

- **id:** TOOL-TK-003
- **area:** tools
- **severity:** High
- **location:** `crates/tools/testkit/src/storage.rs:431-477` (entire `impl Transaction for InMemoryTransaction`)
- **description:** `InMemoryTransaction` does not override `Transaction::bulk_insert_student_attendances` (defined at `crates/infra/storage/src/transaction.rs:86-91`). Because the trait's default implementation returns `DomainError::NotSupported`, any domain command that calls `tx.bulk_insert_student_attendances(&rows)` against the in-memory adapter will fail at runtime — yet the same call against Postgres/MySQL/SQLite adapters succeeds.
- **expected:** Per `crates/infra/storage/src/transaction.rs:66-86`: the trait explicitly notes the bulk-marking service uses the transactional form "so the outbox appends, the idempotency record, the audit row, and the `StudentAttendance` rows all commit atomically." The testkit is meant to exercise this path; the default `NotSupported` blocks that exercise.
- **evidence:**
  ```rust
  crates/infra/storage/src/transaction.rs:86-91
  async fn bulk_insert_student_attendances(&self, rows: &[StudentAttendanceRow]) -> Result<()> {
      let _ = rows;
      Err(educore_core::error::DomainError::not_supported(
          "Transaction::bulk_insert_student_attendances is not supported by this adapter",
      ))
  }
  ```
  And the storage-adapter form (which IS implemented):
  ```rust
  crates/tools/testkit/src/storage.rs:307-340
  async fn bulk_insert_student_attendances(
      &self,
      ctx: &TenantContext,
      rows: &[StudentAttendanceRow],
  ) -> Result<()> { ... }
  ```
  No override of the same method on `impl Transaction for InMemoryTransaction` at lines 432-477.

---

### FINDING 4

- **id:** TOOL-TK-004
- **area:** tools
- **severity:** High
- **location:** `crates/tools/testkit/src/storage.rs:81-107` (`impl Outbox for OutboxHandle`)
- **description:** The outbox sub-port does not enforce the per-school partition mandated by the port docstring. `OutboxHandle::append` accepts any envelope (no school validation); `OutboxHandle::pending` returns the first `limit` envelopes regardless of school. The port trait at `crates/infra/storage/src/outbox.rs:104-108` documents: "The outbox is partitioned by `school_id` so callers see only envelopes for their school."
- **expected:** Per `crates/infra/storage/src/outbox.rs:104-108` and `crates/infra/storage/src/outbox.rs:115` (`pending_count` takes `school_id`): `pending` should filter the drain by `school_id`.
- **evidence:**
  ```rust
  crates/tools/testkit/src/storage.rs:97-100
  async fn pending(&self, limit: u32) -> Result<Vec<SerializedEnvelope>> {
      let outbox = self.0.outbox.lock();
      Ok(outbox.iter().take(limit as usize).cloned().collect())
  }
  ```
  The `school_id` field on each envelope is ignored entirely.

---

### FINDING 5

- **id:** TOOL-TK-005
- **area:** tools
- **severity:** High
- **location:** `crates/tools/testkit/src/storage.rs:193-219` (`impl Idempotency for IdempotencyHandle`)
- **description:** `IdempotencyHandle` does not override the default `exists` method (defined at `crates/infra/storage/src/idempotency.rs:90-92`). The default calls `lookup` and checks `is_some`, which is functionally correct but allocates a full `IdempotencyRecord` clone for every existence check (the engine's dispatcher uses this on every retry). The testkit's port-completeness contract is to ship its own idiomatic override matching the in-memory backend, even if the trait default happens to work.
- **expected:** Per `crates/infra/storage/src/idempotency.rs:86-92`: "adapters with a cheap existence check may override." A `HashMap::contains_key` lookup is strictly cheaper than a `HashMap::get` + `Option::cloned`.
- **evidence:**
  ```rust
  crates/infra/storage/src/idempotency.rs:90-92
  async fn exists(&self, key: IdempotencyCompositeKey) -> Result<bool> {
      Ok(self.lookup(key).await?.is_some())
  }
  ```
  No `async fn exists` override on `IdempotencyHandle` at `crates/tools/testkit/src/storage.rs:193-219`.

---

### FINDING 6

- **id:** TOOL-TK-006
- **area:** tools
- **severity:** Medium
- **location:** `crates/tools/testkit/src/storage.rs:342-363`
- **description:** `InMemoryStorageAdapter::watch_changes` ignores the `ChangeFilter::since: Option<VersionCursor>` field. The trait documents `since` as an "Optional resume point; if `None`, the stream starts at the current cursor position for the school" (`crates/infra/storage/src/change_stream.rs:40-41`). With no plumbing to populate `self.inner.change_events` from outbox appends, `watch_changes` always returns an empty stream in practice, regardless of filter parameters.
- **expected:** Per `crates/infra/storage/src/change_stream.rs:39-41` (the `since` field doc) and `docs/ports/storage.md` § 3: sync consumers pass `since: Some(cursor)` to resume from a checkpoint. The testkit must either honor `since` or document that resume is not supported.
- **evidence:**
  ```rust
  crates/tools/testkit/src/storage.rs:342-363
  async fn watch_changes(&self, filter: ChangeFilter) -> Result<ChangeStream> {
      use futures::stream;
      let events = self.inner.change_events.lock().clone();
      let matching: Vec<std::result::Result<ChangeEvent, DomainError>> = events
          .into_iter()
          .filter(|e| e.school_id == filter.school_id)
          .filter(|e| {
              if filter.aggregate_types.is_empty() {
                  return true;
              }
              filter.aggregate_types.iter().any(|f| match f {
                  educore_storage::change_stream::AggregateTypeFilter::Exact(n) => {
                      &e.aggregate_type == n
                  }
                  educore_storage::change_stream::AggregateTypeFilter::Any => true,
              })
          })
          .map(Ok)
          .collect();
      let s = stream::iter(matching);
      Ok(ChangeStream { inner: Box::pin(s) })
  }
  ```
  The `filter.since` field is never read; `change_events` is never populated by `outbox().append(...)`.

---

### FINDING 7

- **id:** TOOL-TK-007
- **area:** tools
- **severity:** Medium
- **location:** `crates/tools/testkit/src/storage.rs:365-371`
- **description:** `InMemoryStorageAdapter::apply_snapshot` accepts a `SchoolSnapshot` and pushes its `aggregates` into a `Vec<SnapshotAggregate>` (`self.inner.snapshots`) but does not hydrate any in-memory aggregate store. A test that calls `apply_snapshot` then queries for the snapshot's aggregates by id would find nothing. The "apply" is a sink, not a hydration.
- **expected:** Per `crates/infra/storage/src/change_stream.rs:189-202`: `SchoolSnapshot` is "a bulk snapshot of a school used for first-time client hydration" — the contract is to make the snapshot's aggregates queryable.
- **evidence:**
  ```rust
  crates/tools/testkit/src/storage.rs:365-371
  async fn apply_snapshot(&self, snapshot: SchoolSnapshot) -> Result<()> {
      let mut store = self.inner.snapshots.lock();
      for agg in snapshot.aggregates {
          store.push(agg);
      }
      Ok(())
  }
  ```
  No aggregate-table read accessors exist on the in-memory backend.

---

### FINDING 8

- **id:** TOOL-TK-008
- **area:** tools
- **severity:** Medium
- **location:** `crates/tools/testkit/src/storage.rs:61-73` (`InMemoryInner`)
- **description:** The `_id_seq: AtomicU64` and `_next_id` method exist but are never read or incremented. The bus field on `InMemoryTransaction` (`_bus: Arc<dyn EventBus>`, line 396) is also never used — even though `commit` (lines 433-452) is the obvious site to republish to the bus. These are dead fields the build plan references as "imports model fields the trait surface doesn't exercise yet" (per PHASE-16-HANDOFF OQ #1), but they should be wired or removed.
- **expected:** Per `AGENTS.md` § Agent Instructions: "No `#[allow(dead_code)]` or `_var` prefixes to silence the compiler. Delete unused code, wire it in, or open a follow-up issue."
- **evidence:**
  ```rust
  crates/tools/testkit/src/storage.rs:61-73
  pub(crate) struct InMemoryInner {
      pub(crate) outbox: Mutex<Vec<SerializedEnvelope>>,
      pub(crate) audit_log: Mutex<Vec<AuditLogEntry>>,
      pub(crate) event_log: Mutex<Vec<EventLogEntry>>,
      pub(crate) idempotency: Mutex<HashMap<IdempotencyCompositeKey, IdempotencyRecord>>,
      pub(crate) bulk_attendance: Mutex<Vec<(SchoolId, Uuid, NaiveDate, Uuid)>>,
      pub(crate) change_events: Mutex<Vec<ChangeEvent>>,
      pub(crate) cursors: Mutex<HashMap<SchoolId, VersionCursor>>,
      pub(crate) snapshots: Mutex<Vec<SnapshotAggregate>>,
      pub(crate) migrated: AtomicBool,
      pub(crate) closed: AtomicBool,
      pub(crate) _id_seq: AtomicU64,
  }
  ```
  And:
  ```rust
  crates/tools/testkit/src/storage.rs:265-268
  fn _next_id(&self) -> u64 {
      self.inner._id_seq.fetch_add(1, Ordering::Relaxed)
  }
  ```

---

### FINDING 9

- **id:** TOOL-TK-009
- **area:** tools
- **severity:** Medium
- **location:** `crates/tools/testkit/src/auth.rs:140-167` (`refresh` impl)
- **description:** `InMemoryAuthProvider::refresh` mints a new `Session` with `capabilities: BTreeSet::<Capability>::new()` (empty) and `metadata: BTreeMap::new()` (empty). Per the `Session` doc at `crates/adapters/auth/src/port.rs:111-112`, `capabilities` is "the pre-computed capability set for this session" that the engine consults instead of the RBAC store. After refresh the user silently loses every granted capability, which would cascade to every subsequent command returning `Forbidden`.
- **expected:** Per `crates/adapters/auth/src/port.rs:108-125`: `capabilities`, `roles`, and `metadata` are pre-computed at session-issuance time and must be preserved across refresh.
- **evidence:**
  ```rust
  crates/tools/testkit/src/auth.rs:150-161
  let new_session = Session {
      session_id: new_session_id,
      user_id: old_session.user_id,
      school_ids: old_session.school_ids.clone(),
      active_school_id: old_session.active_school_id,
      roles: old_session.roles.clone(),
      capabilities: BTreeSet::<Capability>::new(),
      mfa_satisfied: old_session.mfa_satisfied,
      issued_at: now,
      expires_at,
      metadata: BTreeMap::new(),
  };
  ```
  The `refresh_mints_new_session_with_same_school` test (lines 269-279) does not assert `capabilities` is preserved.

---

### FINDING 10

- **id:** TOOL-TK-010
- **area:** tools
- **severity:** Medium
- **location:** `crates/tools/testkit/src/auth.rs:124-131` (`validate` impl)
- **description:** `InMemoryAuthProvider::validate` returns `AuthError::Expired` for any non-`Bearer` `AuthScheme` (Cookie or `Custom`). The port contract permits all three schemes (`Bearer`, `Cookie`, `Custom`), and `Cookie` is the second-most-common auth surface. Returning `Expired` for a never-presented cookie is semantically wrong; the correct variant is `Malformed` (matching `JwtAuthProvider::validate` at `crates/adapters/auth/src/jwt.rs:399-403`).
- **expected:** Per `crates/adapters/auth/src/port.rs:131-142` and the parallel `JwtAuthProvider::validate` (`crates/adapters/auth/src/jwt.rs:398-404`): the correct error for "wrong scheme" is `AuthError::Malformed`, not `AuthError::Expired`.
- **evidence:**
  ```rust
  crates/tools/testkit/src/auth.rs:124-131
  async fn validate(&self, token: &AuthToken) -> Result<Session, AuthError> {
      if !matches!(token.scheme, AuthScheme::Bearer) {
          return Err(AuthError::Expired);
      }
      let key = format!("{:?}", token.value);
      let sessions = self.sessions.lock().unwrap_or_else(PoisonError::into_inner);
      sessions.get(&key).cloned().ok_or(AuthError::Expired)
  }
  ```

---

### FINDING 11

- **id:** TOOL-TK-011
- **area:** tools
- **severity:** Medium
- **location:** `crates/tools/testkit/src/auth.rs:91-167` (`impl AuthProvider`)
- **description:** The credential-key derivation is unstable for `Credential::Bearer`: it uses `format!("{token:?}")`, which renders the inner `String` via `Debug`. Any non-printable byte, escape, or quote in the token would produce a different string from the value the caller passes back in a subsequent `AuthToken::value` lookup (because `validate` formats `token.value` via `{:?}` and `authenticate` also formats the same `String` via `Debug`). For typical printable ASCII this works, but the lookup key is bound to the `Debug` rendering rather than the value — making the round-trip fragile.
- **expected:** Per `crates/adapters/auth/src/port.rs:62-72` and the comment in `crates/tools/testkit/src/auth.rs:170-176`: the lookup key should be derived from the credential directly, not from a `Debug` rendering.
- **evidence:**
  ```rust
  crates/tools/testkit/src/auth.rs:117-119
  let key = credential_key(&credential)?;
  let mut sessions = self.sessions.lock().unwrap_or_else(PoisonError::into_inner);
  sessions.insert(key, session.clone());
  ```
  And:
  ```rust
  crates/tools/testkit/src/auth.rs:185-186
  Credential::Bearer(token) => Ok(format!("{token:?}")),
  ```

---

### FINDING 12

- **id:** TOOL-TK-012
- **area:** tools
- **severity:** Medium
- **location:** `crates/tools/testkit/src/notify.rs:108-115` (`send` impl)
- **description:** `InMemoryNotificationProvider::send` does NOT honor the `request.idempotency_key: Option<IdempotencyKey>` field. Every call generates a fresh `NotificationReceipt` with a new `receipt_id`. The port spec at `docs/ports/notifications.md:162-166` mandates: "`idempotency_key` is used by the adapter to deduplicate retries. The engine generates a deterministic key from `(command_id, recipient, template_version)` so the same logical send is not duplicated."
- **expected:** Per `docs/ports/notifications.md:162-166` and the port trait docstring at `crates/adapters/notify/src/port.rs:1180-1185`: `idempotency_key` deduplicates retries. The testkit `send` must check the key, return the stored receipt on match.
- **evidence:**
  ```rust
  crates/tools/testkit/src/notify.rs:110-115
  async fn send(&self, request: SendNotification) -> Result<NotificationReceipt> {
      let receipt = Self::make_receipt(&request.channel);
      let mut sends = self.sends.lock().unwrap_or_else(PoisonError::into_inner);
      sends.push(request);
      Ok(receipt)
  }
  ```
  No `idempotency_key` lookup; the `multiple_sends_are_recorded_in_order` test (lines 269-279) asserts the receipts are distinct (which is correct for distinct sends, but does not exercise idempotency).

---

### FINDING 13

- **id:** TOOL-TK-013
- **area:** tools
- **severity:** Medium
- **location:** `crates/tools/testkit/src/notify.rs:117-134` (`send_bulk` impl)
- **description:** `InMemoryNotificationProvider::send_bulk` does NOT honor `request.idempotency_key: Option<IdempotencyKey>` either. Each call mints a fresh `bulk_id` from `Uuid::new_v4()` and stores the bulk receipt under that key. A retry with the same `idempotency_key` would create a duplicate bulk send, violating the port spec.
- **expected:** Per `docs/ports/notifications.md:162-166` and `crates/adapters/notify/src/port.rs:1255-1258`: idempotency deduplication applies to bulk sends as well.
- **evidence:**
  ```rust
  crates/tools/testkit/src/notify.rs:117-134
  async fn send_bulk(&self, request: SendBulkNotification) -> Result<BulkReceipt> {
      let bulk_id = BulkId::new(format!("in-memory-bulk-{}", Uuid::new_v4()));
      let receipts: Vec<NotificationReceipt> = request
          .recipients
          .iter()
          .map(|_row| Self::make_receipt(&request.channel))
          .collect();

      let bulk_receipt = BulkReceipt {
          bulk_id: bulk_id.clone(),
          receipts,
          failed: Vec::new(),
      };

      let mut bulks = self.bulks.lock().unwrap_or_else(PoisonError::into_inner);
      bulks.insert(bulk_id, bulk_receipt.clone());
      Ok(bulk_receipt)
  }
  ```

---

### FINDING 14

- **id:** TOOL-TK-014
- **area:** tools
- **severity:** Medium
- **location:** `crates/tools/testkit/src/notify.rs:136-138` (`status` impl)
- **description:** `InMemoryNotificationProvider::status` returns `DeliveryStatus::Sent` for any `receipt_id`, even for ids that were never sent. The provider should look up its own `sends` Vec (keyed by `receipt_id`) and return the actual stored status, or `NotFound` for unknown ids.
- **expected:** Per `crates/adapters/notify/src/port.rs:1407-1411`: "Looks up the current delivery status of a previously sent notification." The lookup must consult the actual send store.
- **evidence:**
  ```rust
  crates/tools/testkit/src/notify.rs:136-138
  async fn status(&self, _receipt_id: NotificationReceiptId) -> Result<DeliveryStatus> {
      Ok(DeliveryStatus::Sent)
  }
  ```

---

### FINDING 15

- **id:** TOOL-TK-015
- **area:** tools
- **severity:** Low
- **location:** `crates/tools/testkit/src/files.rs:17-23` (module-level doc) and `crates/tools/testkit/src/files.rs:90-106` (checksum implementation)
- **description:** The module docstring says "the spec requires a content-addressable SHA-256 hex digest" but the in-memory checksum is `format!("{:x}", content.len())` — a length-derived placeholder. This is documented but the test at line 305 asserts the etag changes on overwrite (which works because length changes), so the placeholder is internally consistent. However, a test asserting two distinct content blobs of the same length produce different checksums would fail.
- **expected:** Per `docs/ports/file-storage.md:84-87`: "The adapter computes a SHA-256 checksum on upload. The engine verifies the checksum on read." The testkit doc acknowledges the gap and points consumers to the real adapters — this is acceptable but should be tested.
- **evidence:**
  ```rust
  crates/tools/testkit/src/files.rs:17-23
  //! # Checksum
  //!
  //! The spec requires a content-addressable SHA-256 hex digest.
  //! The in-memory adapter uses a length-based hex placeholder
  //! (`format!("{:x}", content.len())`) so the testkit does not
  //! need to take on a SHA-256 crate dependency. Tests that need a
  //! real content-addressable hash should exercise the
  //! `LocalFileStorage` or `S3FileStorage` reference
  //! implementations instead.
  ```
  And:
  ```rust
  crates/tools/testkit/src/files.rs:92-106
  let checksum = format!("{:x}", request.content.len());
  let now = Timestamp::now();

  let reference = FileReference {
      key: request.key.clone(),
      etag: format!("\"{checksum}\""),
      ...
      checksum: Checksum::new(checksum),
  };
  ```

---

### FINDING 16

- **id:** TOOL-TK-016
- **area:** tools
- **severity:** Medium
- **location:** `crates/tools/testkit/src/files.rs:60-118` (`put` impl)
- **description:** The idempotency-key lookup at lines 64-73 falls through to a fresh insert if the file referenced by the key has been deleted between the two phases. There is no test for the "idempotency key survives a delete" edge case. A consumer that puts, deletes, then retries the same idempotency-keyed put would see a new file (not the original reference) — violating the contract "A retry with the same token returns the same `FileReference` without re-uploading."
- **expected:** Per `docs/ports/file-storage.md:78-83`: idempotency is keyed on the `idempotency_key`, not on the underlying file's existence. The lookup must hold the original reference even after `delete`.
- **evidence:**
  ```rust
  crates/tools/testkit/src/files.rs:64-73
  if let Some(idempotency_key) = request.idempotency_key.as_ref() {
      let idem = self.idempotency_keys.lock();
      if let Some(existing_key) = idem.get(idempotency_key).cloned() {
          drop(idem);
          let store = self.store.lock();
          if let Some((existing_ref, _)) = store.get(&existing_key) {
              return Ok(existing_ref.clone());
          }
      }
  }
  ```
  No test exercises `put → delete → put(same idem key)`.

---

### FINDING 17

- **id:** TOOL-TK-017
- **area:** tools
- **severity:** Low
- **location:** `crates/tools/testkit/src/files.rs:24` (module doc) and `crates/tools/testkit/src/files.rs:29-33` (imports)
- **description:** The module doc says "put is idempotent on `PutRequest::idempotency_key`. A retry with the same token returns the original `FileReference` without re-uploading" — and the module docstring lists `parking_lot::Mutex<HashMap<...>>` as the storage type. But the imports at line 33 use `parking_lot::Mutex` while `InMemoryFileStorage` is `#[derive(Default)]` and does not pass `parking_lot::Mutex` correctly through `derive(Default)`. The default implementation (line 54) uses `Self::default()` which calls `Mutex::default()` which is fine for parking_lot. This is correct but worth noting: `InMemoryPaymentProvider` (line 41-55) uses `#[derive(Default)]` while the `charges` and `refunds` fields contain `parking_lot::Mutex`, which works. No actual bug, but inconsistent style.
- **expected:** Per `AGENTS.md` § Code Standards: idiomatic Rust with clear ownership. Mixing `parking_lot::Mutex` with `std::sync::Mutex` (auth.rs, notify.rs, integrations.rs all use `std::sync::Mutex`) without a documented rationale creates maintenance friction.
- **evidence:**
  ```rust
  crates/tools/testkit/src/files.rs:25-36
  use std::collections::HashMap;

  use async_trait::async_trait;
  use educore_core::value_objects::Timestamp;
  use educore_files::port::{
      Checksum, FileKey, FileMetadata, FileReference, FileStorage, FileStream, IdempotencyKey,
      PutRequest, SignedUrlOptions, StorageClass,
  };
  use parking_lot::Mutex;
  use tokio::sync::mpsc;

  use educore_files::errors::{FileStorageError, InfrastructureError};
  ```
  vs. `auth.rs:38-39`:
  ```rust
  use std::collections::{BTreeMap, BTreeSet, HashMap};
  use std::sync::{Mutex, PoisonError};
  ```

---

### FINDING 18

- **id:** TOOL-TK-018
- **area:** tools
- **severity:** High
- **location:** `crates/tools/testkit/src/payment.rs:65-83` and `crates/tools/testkit/src/payment.rs:97-114` (`charge` impl)
- **description:** `InMemoryPaymentProvider` uses an `AtomicU64::fetch_add(1, Ordering::Relaxed)` then `.wrapping_add(1)` to compute the next id. The `Relaxed` ordering combined with `.wrapping_add(1)` on the previous result means the first call returns 2 (because `fetch_add` returns the previous value 0, then `.wrapping_add(1)` makes it 1, but then `Self::default()` initialized the counter to 0 with a prior `fetch_add` returning 0+1=1). Actually the test `id_seq_starts_at_one_and_increments` confirms `peek_id()` returns 1, 2, 3. The `charge`/`refund` use the same `peek_id` so they mint `in-mem-charge-1`, `in-mem-charge-2`. The bug is subtler: `peek_id` returns `fetch_add(..).wrapping_add(1)` which means the first call to `peek_id` returns 1 (since `fetch_add` returns 0 → `0.wrapping_add(1)` = 1), but the underlying counter is now 1. The second call returns 2 (fetch_add returns 1, +1 = 2), counter now 2. This is internally consistent. **However**, `peek_id` mutates `id_seq` even though its name says "peek". The naming lies: every call increments.
- **expected:** Per `AGENTS.md` § Agent Instructions: "No `#[allow(dead_code)]` or `_var` prefixes to silence the compiler. Delete unused code, wire it in, or open a follow-up issue." `peek_id` should either be renamed to `next_id` (it mutates) or not mutate.
- **evidence:**
  ```rust
  crates/tools/testkit/src/payment.rs:65-67
  fn peek_id(&self) -> u64 {
      self.id_seq.fetch_add(1, Ordering::Relaxed).wrapping_add(1)
  }
  ```
  And the test that confirms the mutation-on-peek behavior:
  ```rust
  crates/tools/testkit/src/payment.rs:343-348
  #[test]
  fn id_seq_starts_at_one_and_increments() {
      let provider = InMemoryPaymentProvider::new();
      assert_eq!(provider.peek_id(), 1);
      assert_eq!(provider.peek_id(), 2);
      assert_eq!(provider.peek_id(), 3);
  }
  ```

---

### FINDING 19

- **id:** TOOL-TK-019
- **area:** tools
- **severity:** Medium
- **location:** `crates/tools/testkit/src/payment.rs:162-181` (`settlement` impl)
- **description:** `settlement` always returns an empty `Settlement` (zero totals, no lines) regardless of what charges have been minted. The module doc at lines 18-25 acknowledges this and tells consumers to "post-process the receipts directly; the in-memory adapter does not auto-link charges to settlement lines." But this makes `settlement` essentially useless for any test that wants to verify "the engine emits one `PaymentSettled` event per charged payment" — the test would need to inspect receipts manually instead of going through the port surface.
- **expected:** Per `crates/adapters/payment/src/port.rs:1113-1120`: "Reports the settlement batch covering the requested window. The engine matches settlement lines to `PaymentReceipt` rows by `provider_payment_id` and emits `PaymentSettled` events for each newly-settled line." The adapter should construct settlement lines from its internal charges store.
- **evidence:**
  ```rust
  crates/tools/testkit/src/payment.rs:162-181
  async fn settlement(
      &self,
      request: SettlementRequest,
  ) -> Result<Settlement, educore_payment::errors::PaymentError> {
      let zero = match Money::new(request.currency.clone(), 0) {
          Ok(m) => m,
          Err(_) => Money::zero(request.currency.clone()),
      };
      Ok(Settlement {
          settlement_id: "in-mem-settlement-1".to_owned(),
          school_id: request.tenant.school_id,
          currency: request.currency.clone(),
          period_start: request.period_start,
          period_end: request.period_end,
          lines: Vec::new(),
          total_gross: zero.clone(),
          total_fees: zero.clone(),
          total_net: zero,
      })
  }
  ```
  The `lines: Vec::new()` is hard-coded; `self.charges` is never consulted.

---

### FINDING 20

- **id:** TOOL-TK-020
- **area:** tools
- **severity:** Medium
- **location:** `crates/tools/testkit/src/payment.rs:143-148` (`status` impl)
- **description:** `status` returns `PaymentStatus::Failed { reason: "not found", code: None }` for any unknown `PaymentId`. The port trait has more nuanced status variants (`Pending`, `Authorized`, `Captured`, `Refunded`, `Voided`, `Failed`, `Disputed` per `crates/adapters/payment/src/port.rs`). For an unknown id, the correct response is a `Result::Err(PaymentError::NotFound)` rather than a synthetic `Failed` status — the real Stripe adapter returns 404 → `PaymentError::NotFound`.
- **expected:** Per `crates/adapters/payment/src/port.rs:1097-1103` and the parallel `stripe.rs:432-466`: `status(payment_id)` returns `Err(PaymentError::NotFound)` for unknown ids, not a synthetic status payload.
- **evidence:**
  ```rust
  crates/tools/testkit/src/payment.rs:73-83
  fn lookup_status(&self, payment_id: &PaymentId) -> PaymentStatus {
      let charges = self.charges.lock();
      charges
          .values()
          .find(|receipt| receipt.payment_id == *payment_id)
          .map(|receipt| receipt.status.clone())
          .unwrap_or_else(|| PaymentStatus::Failed {
              reason: "not found".to_owned(),
              code: None,
          })
  }
  ```
  The test at lines 297-307 codifies the synthetic-Failed behavior as correct.

---

### FINDING 21

- **id:** TOOL-TK-021
- **area:** tools
- **severity:** High
- **location:** `crates/tools/testkit/README.md:1-3`
- **description:** README states the testkit provides in-memory implementations of "the engine's six ports (storage, auth, notify, payment, files, and event-bus)". This list omits the `IntegrationGateway` port. The lib.rs doc-comment (line 3) and the handoff (line 22 of PHASE-16-HANDOFF.md) both say "seven ports" including `IntegrationGateway`. The `integrations.rs` module ships `InMemoryIntegrationGateway` as a real port impl. So the README is stale.
- **expected:** Per `docs/build-plan.md:1653-1656` (task 1, lists 6 ports) vs. `docs/handoff/PHASE-16-HANDOFF.md:22-24` and `crates/tools/testkit/src/lib.rs:3-6` (both say 7 ports including integrations): one source must be wrong. The crate actually delivers 7 ports; the build-plan/README are out of sync.
- **evidence:**
  ```markdown
  crates/tools/testkit/README.md:1-3
  # educore-testkit
  
  The testkit crate provides in-memory implementations of the engine's six ports (storage, auth, notify, payment, files, and event-bus) for use in unit and integration tests.
  ```
  vs.
  ```rust
  crates/tools/testkit/src/lib.rs:1-6
  //! # educore-testkit
  //!
  //! In-memory test adapters for the engine's seven ports
  //! (StorageAdapter + AuthProvider + NotificationProvider +
  //! PaymentProvider + FileStorage + IntegrationGateway +
  //! EventBus). For unit and integration tests only.
  ```
  And the build plan:
  ```
  docs/build-plan.md:1653-1656
  1. `educore-testkit`: in-memory impls of all 6 ports
     (`StorageAdapter`, `AuthProvider`, `NotificationProvider`,
     `PaymentProvider`, `FileStorage`, `EventBus`). Consumer tests use
     these to run domain commands without docker.
  ```

---

### FINDING 22

- **id:** TOOL-TK-022
- **area:** tools
- **severity:** Low
- **location:** `crates/tools/testkit/src/event_bus.rs:1-37`
- **description:** The `event_bus` module is a pure re-export of `educore_event_bus::InProcessEventBus` plus a type alias `InMemoryEventBus = InProcessEventBus`. The `TestkitWorld::bus` field is typed `Arc<dyn educore_events::event_bus::EventBus>`, which means consumers can already construct an `InProcessEventBus` themselves and hand it to `InMemoryStorageAdapter::new(bus)`. The re-export and alias add a layer of indirection without adding functionality.
- **expected:** Per `AGENTS.md` § Code Standards: "Avoid relative path dependencies outside the workspace." The re-export pattern is acceptable but the alias `InMemoryEventBus` adds no value over `educore_event_bus::InProcessEventBus` — both names refer to the same type.
- **evidence:**
  ```rust
  crates/tools/testkit/src/event_bus.rs:24-37
  #![forbid(unsafe_code)]
  #![deny(missing_docs)]

  pub use educore_event_bus::InProcessEventBus;

  /// Testkit-local alias for the in-process event bus.
  ///
  /// The alias exists so consumers can write
  /// `use educore_testkit::event_bus::InMemoryEventBus;` without
  /// taking a direct dep on `educore-event-bus`. The underlying
  /// type is `educore_event_bus::InProcessEventBus` (re-exported
  /// above) — see that type's rustdoc for the full MPMC /
  /// replay-log contract.
  pub type InMemoryEventBus = InProcessEventBus;
  ```

---

### FINDING 23

- **id:** TOOL-TK-023
- **area:** tools
- **severity:** Medium
- **location:** `crates/tools/testkit/src/sync.rs:1-53`
- **description:** The `sync` module is a placeholder — it exposes a single `dummy_witness()` no-op function and two trivial tests. The module-level doc at lines 6-22 acknowledges "The actual `ChangeStream` and per-school `VersionCursor` table live inside the in-memory storage adapter (see `storage::InMemoryStorageAdapter` and its `watch_changes`, `apply_snapshot`, `cursor_for`, `advance_cursor` methods)." So the sync module exists only because `lib.rs:76` declares `pub mod sync;` and that declaration would otherwise fail to resolve. The sync surface is therefore inaccessible as `educore_testkit::sync::*` — consumers must reach into `educore_testkit::storage::*` instead, which means the `TestkitWorld` does not expose a sync surface.
- **expected:** Per `docs/build-plan.md:1653-1656`: the testkit is the in-memory backplane for all sync engine integration tests. A `sync` module that is purely a placeholder contradicts the exit criterion "consumer tests use these to run domain commands without docker."
- **evidence:**
  ```rust
  crates/tools/testkit/src/sync.rs:1-22
  //! # In-memory sync primitives
  //!
  //! The testkit exposes a `sync` module because
  //! [`lib.rs`](crate) declared `pub mod sync;` and the test
  //! harness needs that module to resolve. The actual
  //! `ChangeStream` and per-school `VersionCursor` table live
  //! inside the in-memory storage adapter (see
  //! [`storage::InMemoryStorageAdapter`](crate::storage::InMemoryStorageAdapter)
  //! and its `watch_changes`, `apply_snapshot`, `cursor_for`,
  //! `advance_cursor` methods).
  ...
  ```
  And:
  ```rust
  crates/tools/testkit/src/sync.rs:28-37
  /// No-op witness function.
  ///
  /// Exists so the module compiles and the type system can verify
  /// the `sync` module is wired into the testkit. The actual
  /// sync primitives (`ChangeStream`, `VersionCursor`,
  /// `watch_changes`, `apply_snapshot`, `cursor_for`,
  /// `advance_cursor`) are exposed as methods on the in-memory
  /// storage adapter — see
  /// [`storage::InMemoryStorageAdapter`](crate::storage::InMemoryStorageAdapter).
  pub fn dummy_witness() {}
  ```

---

### FINDING 24

- **id:** TOOL-TK-024
- **area:** tools
- **severity:** Low
- **location:** `crates/tools/testkit/src/lib.rs:144-175` (test module)
- **description:** The lib.rs test module contains only three trivial tests (`package_metadata_is_set`, `testkit_world_constructs_with_all_seven_ports`, `test_world_function_constructs_testkit_world`). None of them exercise the integration between ports — for example, no test verifies "after a domain command writes to the outbox, a bus subscriber receives the event" (which would have caught finding TOOL-TK-001).
- **expected:** Per `AGENTS.md` § Validation Checklist: "At least one integration test added for new behavior." The `TestkitWorld` is the engine's pre-wired in-memory world; tests should exercise at least one cross-port flow.
- **evidence:**
  ```rust
  crates/tools/testkit/src/lib.rs:144-175
  #[cfg(test)]
  mod tests {
      use super::*;

      #[test]
      fn package_metadata_is_set() {
          assert_eq!(PACKAGE_NAME, "educore-testkit");
          assert!(!PACKAGE_VERSION.is_empty());
      }

      #[test]
      fn testkit_world_constructs_with_all_seven_ports() {
          let world = TestkitWorld::new();
          let _: &std::sync::Arc<storage::InMemoryStorageAdapter> = &world.storage;
          ...
      }

      #[test]
      fn test_world_function_constructs_testkit_world() {
          let _world = test_world();
      }
  }
  ```

---

### FINDING 25

- **id:** TOOL-TK-025
- **area:** tools
- **severity:** Low
- **location:** `crates/tools/testkit/src/storage.rs:26`
- **description:** `storage.rs` has a module-level `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]` at line 26, which silences clippy for ALL production code in the file (not just the test module). The other testkit modules (auth.rs, notify.rs, files.rs, payment.rs, etc.) correctly scope the allow to `#[allow(...)]` on the `mod tests` block. The broad module-level allow hides any future production-code violations from clippy.
- **expected:** Per `AGENTS.md` § Code Standards: "No `unwrap()` or `expect()` in production paths." Per AGENTS.md § Agent Instructions: "No `#[allow(dead_code)]` or `_var` prefixes to silence the compiler."
- **evidence:**
  ```rust
  crates/tools/testkit/src/storage.rs:26
  #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
  ```
  Compare with `files.rs:217-223`:
  ```rust
  #[cfg(test)]
  #[allow(
      clippy::unwrap_used,
      clippy::expect_used,
      clippy::panic,
      clippy::dbg_macro
  )]
  mod tests { ... }
  ```

---

### FINDING 26

- **id:** TOOL-TK-026
- **area:** tools
- **severity:** Low
- **location:** `crates/tools/testkit/src/integrations.rs:135-142`
- **description:** The `invoke` impl uses `match self.invocations.lock() { Ok(mut g) => g.push(request), Err(_) => return Err(IntegrationError::Infrastructure(...)) }` — manual lock-error handling. The same crate uses `parking_lot::Mutex` in storage.rs/files.rs/payment.rs (which never poisons) and `std::sync::Mutex` with `unwrap_or_else(PoisonError::into_inner)` in auth.rs/notify.rs. The integration gateway uses neither idiom: it maps poison to a domain error directly. Three inconsistent lock error-handling strategies across the testkit's in-memory backends.
- **expected:** Per `AGENTS.md` § Code Standards: idiomatic Rust; one consistent style per crate. The choice between `parking_lot::Mutex` and `std::sync::Mutex` should be deliberate, and the error-handling pattern should match across the same crate's modules.
- **evidence:**
  ```rust
  crates/tools/testkit/src/integrations.rs:135-142
  async fn invoke(&self, request: IntegrationRequest) -> IntegrationResult<IntegrationResponse> {
      match self.invocations.lock() {
          Ok(mut g) => g.push(request),
          Err(_) => {
              return Err(IntegrationError::Infrastructure(Box::new(
                  std::io::Error::other("InMemoryIntegrationGateway: invocations mutex poisoned"),
              )));
          }
      }
      Ok(IntegrationResponse { ... })
  }
  ```

---

### FINDING 27

- **id:** TOOL-TK-027
- **area:** tools
- **severity:** Low
- **location:** `crates/tools/testkit/src/auth.rs:202-292` (`mod tests`) and `crates/tools/testkit/src/notify.rs:145-307` (`mod tests`)
- **description:** Tests for `auth` and `notify` use `futures::executor::block_on` (their own `block_on` helper, defined inside each test module). Tests for `storage` and `files` use `tokio::test` or `tokio::runtime::Runtime::new().unwrap()` + `rt.block_on`. Two different async-execution styles in the same crate.
- **expected:** Per `AGENTS.md` § Code Standards: idiomatic Rust; consistent test style. The `tokio::test` pattern (used in `event_bus.rs`, `files.rs`, `integrations.rs`) is the workspace convention (per `PHASE-16-HANDOFF.md` test counts). `auth.rs`/`notify.rs`/`payment.rs` deviate.
- **evidence:**
  ```rust
  crates/tools/testkit/src/auth.rs:222-225
  fn block_on<F: std::future::Future>(future: F) -> F::Output {
      futures::executor::block_on(future)
  }
  ```
  vs.:
  ```rust
  crates/tools/testkit/src/event_bus.rs:80-82
  #[tokio::test]
  async fn publish_and_subscribe_round_trip_through_alias() { ... }
  ```

---

### FINDING 28

- **id:** TOOL-TK-028
- **area:** tools
- **severity:** Low
- **location:** `crates/tools/testkit/src/payment.rs:333-340` and `crates/tools/testkit/src/payment.rs:343-348`
- **description:** The test `settlement_returns_empty_batch_in_requested_currency` (lines 333-340) hard-codes the expected `settlement_id` as `"in-mem-settlement-1"`. But `peek_id` is called by `charge` and `refund` on every invocation, so by the time `settlement` is called in a real test scenario the counter has advanced. The hard-coded id would be wrong in any test that runs after a charge or refund. The test is only correct in isolation (no prior charges).
- **expected:** Per `AGENTS.md` § Agent Instructions: tests must validate real-world scenarios. A test that asserts a value only because the test runs in isolation is fragile.
- **evidence:**
  ```rust
  crates/tools/testkit/src/payment.rs:333-340
  let settlement = futures::executor::block_on(provider.settlement(req)).unwrap();
  assert_eq!(settlement.settlement_id, "in-mem-settlement-1");
  ```
  The `peek_id` call site:
  ```rust
  crates/tools/testkit/src/payment.rs:171
  settlement_id: "in-mem-settlement-1".to_owned(),
  ```
  (hard-coded; no id_seq involvement in settlement).

---

### END FINDINGS

**Count: 28 findings.**
