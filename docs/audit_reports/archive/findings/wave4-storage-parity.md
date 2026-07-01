## Wave 4 Tools Audit Report — `educore-storage-parity`

**Scope:** `crates/tools/storage-parity/` (26 test files, 10,608 lines), `docs/build-plan.md:1646-1702` (Phase 16), `docs/ports/storage.md` (port contract), `docs/coverage.toml` parity rows, `crates/tools/testkit/` (parity dependency).

**Total findings:** 31

---

### FINDING 1

- **id:** PAR-001
- **area:** tools-parity
- **severity:** Critical
- **location:** `crates/tools/storage-parity/tests/parity_transaction_commit_rollback.rs:21-32` (module doc)
- **description:** The transaction commit/rollback parity test explicitly admits that every shipped adapter (testkit, SQLite, SurrealDB, PostgreSQL, MySQL) implements `commit` and `rollback` as flag-only operations. A rolled-back transaction is documented to leave its writes visible to subsequent transactions. The test only asserts `commit` and `rollback` return `Ok(())` — never that atomicity holds — and the file declares the gap as an open backlog item. Any consumer relying on the engine's published "all sub-port writes are atomic with the command's mutation" guarantee is silently exposed to a write-skew.
- **expected:** Per `docs/ports/storage.md:133-136`: "On `commit` the writes are persisted and the outbox events are released to the event bus. On `rollback` the writes are discarded and the outbox is cleared." Per `docs/schemas/event-schema.md` (engine invariant): the outbox row is part of the same transaction as the aggregate mutation, guaranteeing atomicity.
- **evidence:**
  ```rust
  // crates/tools/storage-parity/tests/parity_transaction_commit_rollback.rs:21-32
  //! **Known limitation:** every storage adapter shipped in
  //! Phase 1 (testkit, SQLite, SurrealDB, PostgreSQL, MySQL)
  //! currently implements `Transaction::commit` and
  //! `Transaction::rollback` as flag-only operations. The
  //! sub-port writes are auto-committed at the query boundary
  //! (SQLite/SurrealDB) or live in shared state without
  //! per-transaction isolation (testkit). A rolled-back
  //! transaction therefore MAY leave its writes visible to a
  //! subsequent transaction.
  ```

---

### FINDING 2

- **id:** PAR-002
- **area:** tools-parity
- **severity:** Critical
- **location:** `crates/tools/storage-parity/tests/parity_idempotency_collision.rs:23-30` (module doc) + `parity_idempotency_collision.rs:177-182` (test)
- **description:** The idempotency parity suite asserts the "same outcome = no-op" path on every backend but explicitly skips the "same key + different outcome = Conflict" path on the three reference adapters (SQLite, SurrealDB, PG, MySQL). Only the in-memory testkit adapter is verified to surface `ErrorKind::Conflict`. The file's module doc admits "the SQLite + SurrealDB reference adapters currently implement `record` as a plain `INSERT` / `INSERT OR REPLACE` and therefore accept a re-write with a different outcome." This is the head-of-line contract from `docs/ports/storage.md § 6` (at-least-once delivery) and `docs/decisions/ADR-014-Idempotency.md`; the parity suite claims all 5 backends support it while only 1 actually does.
- **expected:** Per `docs/ports/storage.md` § 6 + `ADR-014`: a duplicate idempotency key with a different outcome must surface `Conflict`. The parity matrix at `parity_behavior_matrix.rs:69-74` declares all 5 backends `supported = true` for `idempotency_collision` — that is materially false for 4 of them.
- **evidence:**
  ```rust
  // crates/tools/storage-parity/tests/parity_idempotency_collision.rs:23-30
  //! 2. Storing the same `(school_id, command_type,
  //!    idempotency_key)` with the **same outcome** is a no-op
  //!    (returns `Ok(())`).
  //!
  //! The "same key + different outcome = Conflict" half of the
  //! contract is enforced by the testkit in-memory backend (see
  //! `crates/tools/testkit/src/storage.rs::IdempotencyHandle`).
  //! The SQLite + SurrealDB reference adapters currently
  //! implement `record` as a plain `INSERT` / `INSERT OR REPLACE`
  ```

---

### FINDING 3

- **id:** PAR-003
- **area:** tools-parity
- **severity:** Critical
- **location:** `crates/tools/storage-parity/tests/parity_outbox_to_event_log_relay.rs:34-43` (module doc) + `parity_outbox_to_event_log_relay.rs:139-150` (test)
- **description:** The outbox → event log payload-round-trip parity test explicitly skips the payload semantic comparison on SurrealDB because the SurrealDB outbox column is typed `object`, which collapses to `Object {}` on read-back. The test code at lines 139-150 carries a `is_surrealdb_deviation` heuristic that silences the assertion. The matrix at `parity_behavior_matrix.rs` nevertheless marks this `(feature, backend) = (outbox_append, surrealdb)` as `supported = true`. A core parity invariant (the event payload survives the relay without mutation) is unenforced on the engine's primary Phase 0 adapter.
- **expected:** Per `docs/ports/storage.md § 4` + `docs/schemas/event-schema.md § 1.1`: "the relay is a pure transformation — `event_id` is the canonical primary key, and the payload must survive the round-trip without mutation." The matrix must mark `outbox_append` on SurrealDB as `supported = false` (or `partial`) until the deviation is fixed.
- **evidence:**
  ```rust
  // crates/tools/storage-parity/tests/parity_outbox_to_event_log_relay.rs:34-43
  //! **Known deviation:** the SurrealDB outbox/event_log adapter
  //! pair is currently known to drop the payload on the
  //! outbox → event_log hop (the outbox column is typed
  //! `object`, which collapses to `Object {}` on the read-back).
  ```
  ```rust
  // crates/tools/storage-parity/tests/parity_outbox_to_event_log_relay.rs:139-150
  let is_surrealdb_deviation = expected_payload
      != serde_json::Value::Object(serde_json::Map::new())
      && actual_payload == serde_json::Value::Object(serde_json::Map::new());
  if !is_surrealdb_deviation {
      assert_eq!(
          actual_payload, expected_payload,
          ...
      );
  }
  ```

---

### FINDING 4

- **id:** PAR-004
- **area:** tools-parity
- **severity:** Critical
- **location:** `crates/tools/storage-parity/tests/parity_event_log_filter.rs:140-159` (test) + module doc lines 19-22
- **description:** The event log filter parity test on the `aggregate_id` axis explicitly catches and swallows an `Err(_)` that contains the substring `"SurrealUuid"` in its debug string. The test does not fail on SurrealDB; it accepts the error as a "documented deviation" of the engine's primary Phase 0 backend. The deviation marker is `aggregate_id` filter emits `aggregate_id = SurrealUuid::from('<uuid>')` which is not valid SurrealQL. Any consumer that subscribes to event log changes filtered by aggregate id on SurrealDB will hit this bug silently.
- **expected:** Per `docs/schemas/event-schema.md § 6` + `docs/ports/storage.md § 7`: every axis of `EventLogFilter` must be honored identically across all backends. The matrix row `("event_log_filter", "surrealdb", "surql", true)` at `parity_behavior_matrix.rs:61` is false.
- **evidence:**
  ```rust
  // crates/tools/storage-parity/tests/parity_event_log_filter.rs:140-159
  match rows {
      Ok(rows) => {
          assert_eq!(rows.len(), 1, ...);
      }
      Err(e) if format!("{e:?}").contains("SurrealUuid") => {
          // SurrealDB known deviation: invalid SurrealQL
          // syntax in the aggregate_id filter. Skipped.
      }
      Err(e) => panic!(...),
  }
  ```

---

### FINDING 5

- **id:** PAR-005
- **area:** tools-parity
- **severity:** Critical
- **location:** `crates/tools/storage-parity/tests/` — every domain integration file except `parity_*.rs` (e.g., `academic_integration.rs:332-440`, `assessment_integration.rs:329-425`, `finance_integration.rs:0`, `library_integration.rs:0`, `hr_integration.rs:0`)
- **description:** Of the 14 domain vertical-slice integration tests (`academic`, `assessment`, `attendance`, `cms`, `communication`, `documents`, `events`, `facilities`, `finance`, `hr`, `library`, `operations`, `settings`), only `academic`, `assessment`, `attendance`, `cms`, `documents`, `events`, `operations`, `settings` have an `#[ignore]`-d Postgres + MySQL variant; `finance`, `hr`, `library`, `facilities`, `communication` have zero `#[ignore]` variants — they run on SQLite only and provide no parity coverage for the production-target adapters. The crate's `Cargo.toml` dev-dependencies list all four adapters, so the wiring exists; the test surface simply does not use it for 5 of 14 domains.
- **expected:** Per `docs/build-plan.md:1653-1656` Phase 16 task 2: "a cross-adapter parity test suite that runs the same scenario against PG, MySQL, SQLite, and the in-memory testkit impl, asserting identical observable behavior." Per the README's stated mission: "runs the same schema-creation and CRUD scenarios against all three shipped storage adapters."
- **evidence:** `grep -c "#\[ignore"` on `tests/finance_integration.rs`, `tests/hr_integration.rs`, `tests/library_integration.rs`, `tests/facilities_integration.rs`, `tests/communication_integration.rs` all return `0`. No `setup_pg`, `setup_mysql`, or `setup_surrealdb` calls appear in any of these 5 files.

---

### FINDING 6

- **id:** PAR-006
- **area:** tools-parity
- **severity:** Critical
- **location:** `crates/tools/storage-parity/tests/cms_integration.rs:1141-1156` (`cms_integration_pg_vertical_slice`, `cms_integration_mysql_vertical_slice`) + `operations_integration.rs:184-194` + `settings_integration.rs:183-193`
- **description:** The CMS, Operations, and Settings PG/MySQL `#[ignore]`-d vertical-slice variants are empty stubs: they construct `setup_test_env()` (or a bare `_school`) but perform zero storage operations and make zero assertions. The function body is `let _env = setup_test_env().await;` for CMS and `let _school = SchoolId::from_uuid(uuid::Uuid::new_v4());` for the other two. These "tests" cannot pass or fail meaningfully; if `cargo test -- --ignored` is run with the env vars set, they will pass vacuously regardless of whether PG/MySQL actually implement the domain correctly.
- **expected:** Per `docs/build-plan.md:1653-1656` Phase 16 task 2: the parity test must assert identical observable behavior across PG, MySQL, SQLite, and testkit. An empty stub satisfies neither the build-plan nor the README.
- **evidence:**
  ```rust
  // crates/tools/storage-parity/tests/cms_integration.rs:1141-1146
  #[tokio::test]
  #[ignore = "requires EDUCORE_PG_URL env var"]
  async fn cms_integration_pg_vertical_slice() {
      // The PG adapter is wired in `educore-storage-parity`; for
      // Phase 12 the SQLite scenario covers the headline path.
      // This test is a placeholder that triggers when the PG
      // URL is set in CI.
      let _env = setup_test_env().await;
  }
  ```
  ```rust
  // crates/tools/storage-parity/tests/operations_integration.rs:184-194
  #[tokio::test]
  #[ignore = "requires EDUCORE_PG_URL env var"]
  async fn operations_integration_pg_vertical_slice() {
      let _school = SchoolId::from_uuid(uuid::Uuid::new_v4());
  }
  ```

---

### FINDING 7

- **id:** PAR-007
- **area:** tools-parity
- **severity:** Critical
- **location:** `crates/tools/storage-parity/tests/documents_integration.rs:1028-1068` (`documents_integration_postgres`, `documents_integration_mysql`)
- **description:** The Documents PG/MySQL `#[ignore]`-d variants connect, migrate, and immediately discard the adapter (`let _adapter: Arc<dyn educore_storage::StorageAdapter> = Arc::new(adapter);`). They perform zero writes, zero reads, zero assertions. The test passes if the connection succeeds, regardless of whether the documents schema actually round-trips on PG/MySQL. The module doc at line 1022-1026 frames this as "the SQLite scenario covers the headline path" — meaning the entire Documents domain has no parity coverage on the production-target adapters.
- **expected:** Per `docs/build-plan.md:1653-1656` + `docs/coverage.toml:697` (Documents): parity coverage must extend to PG/MySQL.
- **evidence:**
  ```rust
  // crates/tools/storage-parity/tests/documents_integration.rs:1028-1068
  #[tokio::test]
  #[ignore = "requires EDUCORE_PG_URL; run with: EDUCORE_PG_URL=postgres://... cargo test -- --ignored"]
  async fn documents_integration_postgres() {
      let url = match std::env::var("EDUCORE_PG_URL") {
          Ok(s) if !s.is_empty() => s,
          _ => return,
      };
      ...
      let adapter = educore_storage_postgres::PostgresStorageAdapter::connect(&url, school)
          .await
          .expect("connect pg");
      adapter.migrate().await.expect("migrate pg");
      let _adapter: Arc<dyn educore_storage::StorageAdapter> = Arc::new(adapter);
  }
  ```

---

### FINDING 8

- **id:** PAR-008
- **area:** tools-parity
- **severity:** High
- **location:** `crates/tools/storage-parity/tests/parity_behavior_matrix.rs:36-94` (entire `PARITY_MATRIX` const) + `parity_behavior_matrix.rs:115-133` (`every_feature_is_either_5_supported_or_fully_unsupported`)
- **description:** The behavior matrix declares 6 features × 5 backends = 30 rows, but the `every_feature_is_either_5_supported_or_fully_unsupported` test enforces an "all-or-nothing" invariant that the file itself then violates in production. Findings PAR-002/003/004 above document three features (idempotency_collision, outbox_append, event_log_filter) where 1–4 backends silently do not honor the contract yet all are marked `supported = true`. The matrix's "all-or-nothing" assertion passes because every row is `true`, so it actively prevents CI from flagging the partial-coverage gaps that the actual scenario tests admit. The "documentation test" gives a false sense of coverage.
- **expected:** Per the file's own module doc lines 12-19: the matrix is "the shape the engine contract requires" and "tests assert behaviour, not dialect strings." The whole point is to surface partial coverage. Either the matrix must mark `false` for non-conforming rows, or the suite must add per-row `assert_supported` tests that fail CI when a partial-feature backend is shipped.
- **evidence:**
  ```rust
  // crates/tools/storage-parity/tests/parity_behavior_matrix.rs:36-94
  const PARITY_MATRIX: &[(&str, &str, &str, bool)] = &[
      // ---- outbox_append (5/5) ----
      ("outbox_append", "testkit", "in-memory", true),
      ("outbox_append", "sqlite", "sqlite3", true),
      ("outbox_append", "surrealdb", "surql", true), // FALSE per PAR-003
      ("outbox_append", "postgres", "pg", true),
      ("outbox_append", "mysql", "mysql", true),
      ...
      ("idempotency_collision", "surrealdb", "surql", true), // FALSE per PAR-002
      ...
  ];
  ```

---

### FINDING 9

- **id:** PAR-009
- **area:** tools-parity
- **severity:** High
- **location:** `crates/tools/storage-parity/tests/parity_cross_backend_equivalence.rs:130-170` (`dispatch_create_school`) + `parity_idempotency_collision.rs:127-151`
- **description:** The cross-backend parity suite duplicates the `relay_outbox_to_event_log` logic from `common::mod.rs:62-78` in the `dispatch_create_school` helper at `parity_cross_backend_equivalence.rs:130-170`. The duplicated helper drains the outbox **inside** the original transaction rather than as a separate relay pass, then calls `relay(adapter)` after commit, which is a no-op because the outbox is already empty. The module doc at lines 95-99 acknowledges the in-tx drain is what makes the testkit backend (which drains on commit) work. This means the parity test does NOT exercise the production relay path; it only exercises a within-transaction drain that no real consumer code uses.
- **expected:** Per `docs/ports/storage.md § 4` + `docs/schemas/event-schema.md`: the outbox is consumed by an external relay process; the parity suite should stand up the relay as a separate component and assert the end-to-end flow (write → outbox → relay → event_log). The current shape hides a bug where the relay is broken (writes survive the relay without reaching the event_log) because the test is wired to short-circuit.
- **evidence:**
  ```rust
  // crates/tools/storage-parity/tests/parity_cross_backend_equivalence.rs:144-170
  // Drain the outbox to the event log WITHIN the transaction
  // so the testkit backend (which drains on commit) does not
  // lose the envelope before the relay runs.
  let pending = tx.outbox().pending(100).await.expect("pending");
  for env in &pending {
      let entry = educore_storage::event_log::EventLogEntry::from_serialized_envelope(env);
      tx.event_log().append(entry).await.expect("event_log append");
      tx.outbox().mark_published(&[env.event_id]).await.expect(...);
  }
  tx.commit().await.expect("commit");
  bus.publish(envelope).await.expect("bus publish");
  let _ = relay(adapter).await; // <-- no-op: outbox already drained
  ```

---

### FINDING 10

- **id:** PAR-010
- **area:** tools-parity
- **severity:** High
- **location:** `crates/tools/storage-parity/tests/cross_cutting_integration.rs:400-410` (`cross_cutting_integration_postgres`, `cross_cutting_integration_mysql`)
- **description:** The PG/MySQL variants of the cross-cutting integration test do not exercise the bus. The SQLite variant subscribes to `test-cross-cutting` via `InProcessEventBus` and asserts the event reaches subscribers (lines 252-267); the PG and MySQL variants only check `outbox.pending() == 0` and `event_log` row count (lines 386-398, 451-461). A PG/MySQL adapter that correctly appends to the event log but never publishes to the bus would pass these tests. The asymmetry is undocumented in the test code.
- **expected:** Per `docs/ports/storage.md § 4`: the relay publishes to the bus. The parity suite must assert the bus receives the envelope on every backend, including PG/MySQL.
- **evidence:**
  ```rust
  // crates/tools/storage-parity/tests/cross_cutting_integration.rs:386-398 (PG)
  let tx = adapter.begin().await.expect("begin");
  let pending = tx.outbox().pending(10).await.expect("pending");
  assert!(pending.is_empty(), "PG outbox should be drained");
  let events = tx.event_log().read(...).await.expect("read");
  assert_eq!(events.len(), 1, "PG event_log should have 1 row");
  // No bus.subscribe / bus.next() / EventSubscription check.
  ```

---

### FINDING 11

- **id:** PAR-011
- **area:** tools-parity
- **severity:** High
- **location:** `crates/tools/storage-parity/tests/cross_cutting_integration.rs:361-410` (`pg_rls_blocks_cross_tenant_audit_reads`)
- **description:** The PG RLS tenant-isolation test depends on a `tenant_b` non-superuser role provisioned by `tools/scripts/pg-rls-test-setup.sql`, per its module doc. The script does not exist in the repository (`find . -name "pg-rls-test-setup.sql"` returns nothing) and the test fallbacks to `EDUCORE_PG_TENANT_B_URL` with `unwrap_or_else(|_| url.clone())` — meaning if the env var is unset, the test silently connects as the superuser and "passes" without exercising RLS. The test cannot fail meaningfully in CI without manual setup that the repo does not encode.
- **expected:** Per `docs/ports/storage.md:140-149`: tenant isolation is enforced at the storage adapter layer and "a test suite that attempts to read across tenants and fails" must run as a CI gate. The current wiring makes the gate optional and undocumented.
- **evidence:**
  ```rust
  // crates/tools/storage-parity/tests/cross_cutting_integration.rs:362-365
  /// Phase 2 OQ coverage. Requires the test runner to provision
  /// a non-superuser `tenant_b` role with SELECT on
  /// engine.audit_log BEFORE running this test. The setup
  /// script lives at `tools/scripts/pg-rls-test-setup.sql`
  ```
  ```bash
  $ find . -name "pg-rls-test-setup.sql"
  (no results)
  ```
  ```rust
  // crates/tools/storage-parity/tests/cross_cutting_integration.rs:407-410
  let url_b = std::env::var("EDUCORE_PG_TENANT_B_URL").unwrap_or_else(|_| url.clone());
  // If unset, falls back to superuser URL -> RLS bypassed -> assertion trivially passes.
  ```

---

### FINDING 12

- **id:** PAR-012
- **area:** tools-parity
- **severity:** High
- **location:** `crates/tools/storage-parity/tests/` (12 of 14 domain integration files; e.g., `finance_integration.rs`, `hr_integration.rs`, `library_integration.rs`, `facilities_integration.rs`, `communication_integration.rs`)
- **description:** No parity test exercises a cross-domain workflow — none of the 14 domain integration files pair commands from two domains. The `finance` → `academic` flow ("student promoted → fee structure regenerates"), the `hr` → `payroll` → `finance` flow, the `library` → `communication` (fine notification) flow, the `attendance` → `communication` (absent notification) flow, the `documents` → `cms` (`form_uploaded_public_indexing_subscriber`) cross-domain subscriber all lack end-to-end parity coverage. The `cms_form_uploaded_public_indexing_subscriber_*` tests at `cms_integration.rs:1079-1115` only verify the pure function (mapping envelope → `FormIndexAction`) and never run it against a real bus + storage adapter.
- **expected:** Per `docs/specs/academic/workflows.md`, `docs/specs/finance/workflows.md`, `docs/specs/hr/workflows.md`: cross-domain flows are documented scenarios and require parity coverage per `docs/build-plan.md:1653-1656`.
- **evidence:** `grep -rl "cross_domain\|cross-domain\|cross domain" crates/tools/storage-parity/tests/` returns no results. None of the 26 test files imports from more than one domain crate (verified by inspecting imports in `finance_integration.rs`, `hr_integration.rs`, `library_integration.rs`, `attendance_integration.rs`).

---

### FINDING 13

- **id:** PAR-013
- **area:** tools-parity
- **severity:** High
- **location:** `crates/tools/storage-parity/tests/parity_event_log_filter.rs:174-184` (`event_log_filter_sqlite`)
- **description:** No parity test exercises `since` and `until` time-range filters on `EventLogFilter`. The module doc at lines 19-22 lists `since` and `until` as filter axes that "must be honored identically by every backend", but the test body never sets `filter.since` or `filter.until` — only school, event_types, aggregate_id, and limit are exercised (the test's own numbered comments 1-6 also omit since/until). The `since`/`until` axes silently bypass CI parity.
- **expected:** Per `docs/ports/storage.md § 7` + `docs/schemas/event-schema.md § 6`: `EventLogFilter` includes `since` and `until` time bounds that must be honored identically.
- **evidence:**
  ```rust
  // crates/tools/storage-parity/tests/parity_event_log_filter.rs:19-22 (module doc)
  //! Asserts that `EventLogFilter` (school_id + event_types +
  //! since + until + aggregate_id) is honored identically by
  //! every backend.
  ```
  The test body's filter mutations (lines 87, 100, 134, 161, 173, 181) never reference `filter.since` or `filter.until`.

---

### FINDING 14

- **id:** PAR-014
- **area:** tools-parity
- **severity:** High
- **location:** `crates/tools/storage-parity/tests/parity_idempotency_collision.rs:78-105` (`assert_outcome_conflict_on_testkit`)
- **description:** The testkit-only outcome-conflict path is the canonical idempotency contract path, yet it has no `#[ignore]`-d PG/MySQL/SurrealDB counterpart and the conflict-detection logic on the SQL adapters is never asserted anywhere. The module doc at lines 23-30 admits the SQL adapters "currently implement `record` as a plain `INSERT` / `INSERT OR REPLACE` and therefore accept a re-write with a different outcome." The matrix at `parity_behavior_matrix.rs:69-74` nonetheless marks all 5 backends `supported = true` for `idempotency_collision`. A consumer relying on at-least-once delivery on PG/MySQL will silently double-write on retry.
- **expected:** Per `ADR-014-Idempotency.md` + `docs/ports/storage.md § 6`: the conflict path is a hard contract that all 4 production adapters must implement. The matrix must mark the 4 non-testkit rows as `false` until they are wired.
- **evidence:**
  ```rust
  // crates/tools/storage-parity/tests/parity_idempotency_collision.rs:177-182
  #[tokio::test]
  async fn idempotency_outcome_conflict_testkit() {
      ...
  }
  ```
  No corresponding `idempotency_outcome_conflict_sqlite`, `_postgres`, `_mysql`, or `_surrealdb` test exists in the file.

---

### FINDING 15

- **id:** PAR-015
- **area:** tools-parity
- **severity:** High
- **location:** `crates/tools/storage-parity/tests/parity_audit_cross_tenant_isolation.rs:170-175` (file end) + `crates/tools/storage-parity/tests/cross_cutting_integration.rs:380-410` (`pg_rls_blocks_cross_tenant_audit_reads`)
- **description:** Tenant isolation is asserted at the application layer (`read_for_target` filters by `school_id`) but the underlying SQL RLS or SurrealDB access scopes are never verified. A PG/MySQL/SurrealDB backend that forgets the `WHERE school_id = ?` clause in its `read_for_target` implementation would pass the application-level assertion if every other backend also forgot the clause — but would fail on real production traffic where the RLS layer is the only line of defense. No parity test stands up a DB-side admin role and confirms it cannot bypass the application filter.
- **expected:** Per `docs/schemas/tenancy-schema.md § 4` + `docs/ports/storage.md:140-149`: tenant isolation is enforced at the storage adapter layer; the engine refuses to issue queries lacking a tenant filter. The parity suite must include a "DB-admin bypass" test on PG (RLS) and SurrealDB (session auth scopes) to confirm the database-level enforcement.
- **evidence:** No test file in `crates/tools/storage-parity/tests/` contains the strings `RLS`, `bypass`, `admin`, `policy`, or `EXPLAIN` (verified by `grep -rl "RLS\|bypass\|policy" crates/tools/storage-parity/tests/` — only `pg_rls_blocks_cross_tenant_audit_reads` matches, and it depends on a missing script per PAR-011).

---

### FINDING 16

- **id:** PAR-016
- **area:** tools-parity
- **severity:** High
- **location:** `crates/tools/storage-parity/tests/` (entire suite, 26 files)
- **description:** No parity test exercises bulk-insert operations. The trait method `Transaction::bulk_insert_student_attendances` (`crates/infra/storage/src/transaction.rs:86-91`) is a documented performance path for bulk attendance marking, and the storage-port contract documents bulk snapshot writes at `docs/ports/storage.md:106`. None of the 158 test attributes in the parity suite cover bulk-insert — no test invokes a service function that internally calls `bulk_insert_student_attendances`, no test imports the method, and the attendance_integration.rs file only inserts single-row attendances.
- **expected:** Per `docs/build-plan.md:1721-1735` Phase 17 task 2: "Load test: 10k students, bulk fee invoice generation … Target: p95 < 500 ms for a bulk-invoice-of-10k-rows command on PG." The Phase 16 parity suite must assert bulk-insert parity across backends before Phase 17 can establish the p95 baseline.
- **evidence:** `grep -rln "bulk_insert\|bulk_insert_student" crates/tools/storage-parity/tests/` returns zero results.

---

### FINDING 17

- **id:** PAR-017
- **area:** tools-parity
- **severity:** High
- **location:** `crates/tools/storage-parity/tests/` (entire suite, 26 files)
- **description:** No parity test exercises concurrent writes. The 158-test suite is entirely sequential; no test uses `tokio::join!`, `tokio::spawn`, or any concurrent primitive. A storage-port contract violation where the SQLite adapter uses a single-writer mutex or the testkit uses a non-locking `RefCell` for outbox/audit/idempotency state — and the SQL adapters use row-level locking — cannot be caught by parity because concurrency is never introduced. This is critical because the parity suite is the engine's only cross-adapter gate.
- **expected:** Per `docs/ports/storage.md § 3` + § "transaction conflict": the `Conflict(String)` error variant at `docs/ports/storage.md:220` exists precisely to surface write-write conflicts; the parity suite must exercise that path.
- **evidence:** `grep -rln "join!\|tokio::spawn\|tokio::select" crates/tools/storage-parity/tests/` returns zero results.

---

### FINDING 18

- **id:** PAR-018
- **area:** tools-parity
- **severity:** High
- **location:** `crates/tools/storage-parity/tests/port_auth_integration.rs:127-147` + `port_files_integration.rs:107-128` + `port_integrations_integration.rs:113-134` + `port_notify_integration.rs:117-138` + `port_payment_integration.rs:159-180`
- **description:** All five Phase 15 port integration files (`port_auth_integration.rs`, `port_files_integration.rs`, `port_integrations_integration.rs`, `port_notify_integration.rs`, `port_payment_integration.rs`) follow an identical structure: 5 sync scenarios (always-on, exercising pure functions + trait surface) + 2 env-gated `#[ignore]`-d async scenarios that **do not actually call the network**. The async scenarios are bare builder calls (`JwtAuthProviderBuilder::new().build()`, `S3FileStorage::builder()...build()`, `LmsIntegrationBuilder::new()...build()`, `StripeProviderBuilder::new()...build()`, `EmailProviderBuilder::new()...build()`) — no `authenticate()` is called, no file is uploaded, no webhook is dispatched. They pass vacuously regardless of whether the port implementations work.
- **expected:** Per `docs/ports/event-bus.md`, `docs/ports/authentication.md`, `docs/ports/file-storage.md`: the port contracts describe end-to-end behavior (authenticate, put_object, send, etc.) and the parity suite must exercise that path on at least the testkit impl and the reference impl.
- **evidence:**
  ```rust
  // crates/tools/storage-parity/tests/port_auth_integration.rs:127-138
  #[tokio::test]
  #[ignore = "requires EDUCORE_PORT_ADAPTER_E2E env var; run with: cargo test -- --ignored"]
  async fn port_auth_async_jwt_full_round_trip() {
      let provider = JwtAuthProviderBuilder::new().build();
      let _session = provider
          .authenticate(Credential::Anonymous)
          .await
          .expect("anonymous auth should succeed");
  }
  ```
  Note: the 5 port files for files, integrations, notify, and payment contain only `_storage = ...build()` builders — they never call any port method.

---

### FINDING 19

- **id:** PAR-019
- **area:** tools-parity
- **severity:** Medium
- **location:** `crates/tools/storage-parity/tests/common/mod.rs:75-86` (`setup_testkit`) + `tests/parity_cross_backend_equivalence.rs:217-241` (`cross_backend_create_school_and_audit_equivalence_testkit` + sqlite + surrealdb)
- **description:** The 3 always-on backends (testkit, SQLite, SurrealDB) all run in-process via `in_memory(...)`. The parity suite never exercises a file-backed SQLite, a remote SurrealDB, or any other "real" deployment. The module doc at lines 9-22 promises "the test surface for unit / integration tests that do not need a real DB" — but the matrix labels them with `in-memory` / `sqlite3` / `surql` dialects, implying they are representative of those backends. A parity gap between file-backed SQLite and in-memory SQLite would be invisible to the suite.
- **expected:** Per `docs/schemas/sql-dialects/sqlite.md` (dialect-specific quirks around WAL mode, temp tables, etc.) + `docs/ports/storage.md`: the parity matrix must distinguish between in-memory and persistent backends when dialect differences exist. The current setup conflates them.
- **evidence:**
  ```rust
  // crates/tools/storage-parity/tests/common/mod.rs:75-86
  pub fn setup_testkit() -> (Arc<dyn StorageAdapter>, SchoolId, TenantContext) {
      let bus: Arc<dyn EventBus> = Arc::new(InProcessEventBus::new());
      let adapter = educore_testkit::storage::InMemoryStorageAdapter::new(bus);
      ...
  }
  pub async fn setup_sqlite() -> ... {
      let adapter = educore_storage_sqlite::SqliteStorageAdapter::in_memory(school).await...;
  }
  pub async fn setup_surrealdb() -> ... {
      let adapter = educore_storage_surrealdb::SurrealStorageAdapter::in_memory(school).await...;
  }
  ```
  No `setup_sqlite_file()` or `setup_surrealdb_remote()` helper exists.

---

### FINDING 20

- **id:** PAR-020
- **area:** tools-parity
- **severity:** Medium
- **location:** `crates/tools/storage-parity/tests/parity_audit_cross_tenant_isolation.rs:68-100` (entire `assert_cross_tenant_isolation`)
- **description:** The cross-tenant isolation test only exercises the audit_log sub-port. The parity suite never asserts cross-tenant isolation for the event_log, outbox, or idempotency sub-ports. A PG/MySQL/SQLite/SurrealDB adapter that scopes audit reads correctly but leaks event_log rows across tenants (e.g., the `aggregate_id` filter in `parity_event_log_filter.rs:140-159` already has a known SurrealDB syntax bug) would pass this audit-only test.
- **expected:** Per `docs/schemas/tenancy-schema.md § 4` + `docs/ports/storage.md:140-149`: tenant isolation is a cross-cutting invariant on every sub-port (outbox, audit, event_log, idempotency). The parity suite must assert isolation on each.
- **evidence:**
  ```rust
  // crates/tools/storage-parity/tests/parity_audit_cross_tenant_isolation.rs:1-15 (module doc)
  //! Asserts that an audit row written for `school_a` is NOT
  //! visible to `read_for_target` when the caller passes
  //! `school_b`...
  ```
  No equivalent `parity_event_log_cross_tenant_isolation.rs`, `parity_outbox_cross_tenant_isolation.rs`, or `parity_idempotency_cross_tenant_isolation.rs` exists.

---

### FINDING 21

- **id:** PAR-021
- **area:** tools-parity
- **severity:** Medium
- **location:** `crates/tools/storage-parity/tests/parity_event_log_filter.rs:174-189` + `parity_audit_cross_tenant_isolation.rs:155-175`
- **description:** No parity test exercises the `cancelled` / `soft_deleted` / `archived` `ActiveStatus` axis. The `EventLogEntry` struct carries an `active_status: ActiveStatus` field, and the audit log carries an immutable history, but the parity suite never inserts an entry with `ActiveStatus::SoftDeleted` or `Archived` and never verifies that `read` filters on `active_status` or that the event log distinguishes them. The test `cross_backend_create_school_and_audit_equivalence_*` at `parity_cross_backend_equivalence.rs:178-204` asserts `active_status == ActiveStatus::Active` only by default.
- **expected:** Per `docs/schemas/event-schema.md` + `docs/ports/storage.md § 7`: the engine distinguishes `Active`, `SoftDeleted`, `Archived`; the parity suite must cover all three.
- **evidence:** `grep -rln "SoftDeleted\|Archived\|ActiveStatus::" crates/tools/storage-parity/tests/` returns zero results.

---

### FINDING 22

- **id:** PAR-022
- **area:** tools-parity
- **severity:** Medium
- **location:** `crates/tools/storage-parity/tests/` (entire suite, 26 files)
- **description:** No parity test exercises unique-constraint violations. The storage-port sub-ports do not surface a `UniqueViolation` error path explicitly, but the engine's uniqueness invariants (school code, admission no, employee id, ISBN, etc.) are enforced at the application layer. None of the 158 tests attempt to insert a duplicate (school_code, admission_no) pair and assert that the engine surfaces a `Conflict` error. The 5 parity suites assert the happy path only.
- **expected:** Per `docs/specs/platform/overview.md` + `docs/specs/academic/aggregates.md`: the engine guarantees uniqueness on admission_no, employee_id, ISBN, school_code, and the parity suite must verify that the storage adapter surfaces a conflict error on duplicate insert.
- **evidence:** `grep -rln "UniqueViolation\|unique_violation\|duplicate" crates/tools/storage-parity/tests/` returns zero results. None of the tests in the 14 domain integration files attempt to insert a duplicate primary key.

---

### FINDING 23

- **id:** PAR-023
- **area:** tools-parity
- **severity:** Medium
- **location:** `crates/tools/storage-parity/tests/` (entire suite, 26 files)
- **description:** No parity test exercises cascading deletes. The 10 domain crates have aggregate roots with children (e.g., `School → Class → Section → Student → Attendance`, `Wallet → WalletTransaction`, `Library → BookIssue → BookReturn`). None of the tests call a delete that would cascade and assert the children are gone (or correctly preserved per the soft-delete convention). The parity suite does not surface a SQL `FOREIGN KEY … ON DELETE CASCADE` semantic across backends.
- **expected:** Per `docs/specs/academic/tables.md` + `docs/schemas/sql-dialects/comparison.md`: SQLite defaults to `ON DELETE RESTRICT`, PG/MySQL default varies; the parity suite must assert the cascade behavior is identical.
- **evidence:** `grep -rln "cascad\|ON DELETE\|delete_cascade" crates/tools/storage-parity/tests/` returns zero results.

---

### FINDING 24

- **id:** PAR-024
- **area:** tools-parity
- **severity:** Medium
- **location:** `crates/tools/storage-parity/tests/parity_cross_backend_equivalence.rs:181-184` + `parity_audit_cross_tenant_isolation.rs:135-140` + `parity_event_log_filter.rs:191-194` + `parity_idempotency_collision.rs:155-158` + `parity_outbox_to_event_log_relay.rs:160-163` + `parity_transaction_commit_rollback.rs:179-182`
- **description:** The "always-on" trio of backends (testkit + SQLite + SurrealDB) all share the `educore-storage-surrealdb` crate dependency for `setup_surrealdb`. The SurrealDB backend is documented as `Phase 0 primary per ADR-017` (per `common/mod.rs:96`), yet the env-gated PG/MySQL variants outnumber the always-on backends by 2:1. In practice CI on a developer laptop runs 3/5 backends; in CI with env vars set, it runs 5/5 — but only the SurrealDB parity is asserted at every PR. The "always-on" surface is the de-facto engine contract; if SurrealDB is the primary, the test surface should mirror that, not skip it.
- **expected:** Per `docs/decisions/ADR-017-StorageStrategy.md` (cited at `common/mod.rs:96`): the primary storage backend is the canonical contract. The 5 parity suites should run the same number of assertions on SurrealDB as on PG/MySQL.
- **evidence:** In each of the 6 parity files, the always-on trio (testkit, sqlite, surrealdb) is structurally identical and asserts identical invariants; the env-gated pair (pg, mysql) does the same. No asymmetry should exist, but per PAR-002/003/004, the SurrealDB row is the one that fails to honor 3 of the 6 contracts.

---

### FINDING 25

- **id:** PAR-025
- **area:** tools-parity
- **severity:** Medium
- **location:** `crates/tools/storage-parity/tests/finance_integration.rs:417` + `crates/tools/storage-parity/tests/library_integration.rs:394` + `crates/tools/storage-parity/tests/facilities_integration.rs:552` + `crates/tools/storage-parity/tests/communication_integration.rs:812`
- **description:** Four domain integration tests (`finance_integration.rs`, `library_integration.rs`, `facilities_integration.rs`, `communication_integration.rs`) end without a `#[tokio::test]` PG/MySQL/SurrealDB variant. None of them carry `setup_pg`, `setup_mysql`, or `setup_surrealdb` imports; their Cargo.toml-dev-dep adapters (`educore-storage-postgres`, `educore-storage-mysql`, `educore-storage-surrealdb`) are listed but never instantiated. The five backend crate dependencies in `Cargo.toml:33-38` therefore ship for parity but only `educore-storage-sqlite` is exercised by 4 of the 14 domain integration files.
- **expected:** Per `docs/coverage.toml:697-710`: each domain's parity row names `crates/tools/storage-parity/tests/<domain>_integration.rs`. The test file is required to exercise parity for that domain; the file's existence is not proof of parity coverage.
- **evidence:** Per-file `#[ignore]` counts: `finance_integration.rs: 0`, `library_integration.rs: 0`, `facilities_integration.rs: 0`, `communication_integration.rs: 0`. The four files contain zero `EDUCORE_PG_URL` / `EDUCORE_MYSQL_URL` references (verified by grep above).

---

### FINDING 26

- **id:** PAR-026
- **area:** tools-parity
- **severity:** Medium
- **location:** `crates/tools/storage-parity/tests/communication_integration.rs:33-100` + `crates/tools/storage-parity/tests/communication_integration.rs:195-770` (`mod compile_full_prelude_scenarios`)
- **description:** The Communication integration test does not exercise any communication domain command. The always-on tests are `communication_package_metadata_is_set` and `communication_full_prelude_scenarios_compile_only_when_wired` (lines 33-100), which only assert the `PACKAGE_NAME` constant and document scenarios as code comments. The `mod compile_full_prelude_scenarios` nested block at lines 195-770 contains six `#[tokio::test]` attributes, but the module's compile flag is off by default (per the file's module doc lines 41-44: "stubbed behind `compile_full_prelude_scenarios`") and therefore the six scenarios never run under `cargo test`.
- **expected:** Per `docs/build-plan.md:1653-1656` + `docs/specs/communication/workflows.md`: the parity suite must execute the Communication headline scenarios end-to-end.
- **evidence:**
  ```rust
  // crates/tools/storage-parity/tests/communication_integration.rs:42-44 (module doc)
  //! Scenarios that depend on symbols
  //! not yet wired into `educore-communication`'s prelude are
  //! stubbed behind `compile_full_prelude_scenarios` (off by
  //! default)
  ```

---

### FINDING 27

- **id:** PAR-027
- **area:** tools-parity
- **severity:** Medium
- **location:** `crates/tools/storage-parity/tests/parity_idempotency_collision.rs:177-182` + `parity_transaction_commit_rollback.rs:147-155` + `parity_event_log_filter.rs:115-160`
- **description:** The parity suite duplicates the `Transaction::rollback` and `EventLogFilter::aggregate_id` workarounds between files but does not centralize the "known deviations" list. Each file documents its deviation in a free-text module doc paragraph (e.g., PAR-001, PAR-002, PAR-003 above). There is no single registry of known parity gaps that a new developer can consult before adding a backend. The `PARITY_MATRIX` const at `parity_behavior_matrix.rs:36-94` is the closest analogue but lists `supported = true` everywhere, contradicting the actual scenario test outcomes.
- **expected:** Per `docs/code-standards.md` + `docs/ports/storage.md`: known deviations must be tracked in a single source (e.g., `docs/audit_reports/known-deviations.md` or a `KNOWN_PARITY_DEVIATIONS` const that the matrix itself checks against).
- **evidence:** Three separate `**Known deviation:**` / `**Known limitation:**` paragraphs at `parity_outbox_to_event_log_relay.rs:34-43`, `parity_idempotency_collision.rs:23-30`, `parity_event_log_filter.rs:53-61` — plus the rollback-known-limitation paragraph at `parity_transaction_commit_rollback.rs:21-32`. None reference each other; the matrix at `parity_behavior_matrix.rs` does not flag them.

---

### FINDING 28

- **id:** PAR-028
- **area:** tools-parity
- **severity:** Low
- **location:** `crates/tools/storage-parity/tests/academic_integration.rs:444-491` + `assessment_integration.rs:455-499` + `attendance_integration.rs:619-784` + `finance_integration.rs:267-417` + `hr_integration.rs:260-385` + `library_integration.rs:246-394` + `facilities_integration.rs:345-552` + `cms_integration.rs:649-1079` + `documents_integration.rs:689-1025`
- **description:** The 14 domain integration tests follow a "mirrors the Phase N pattern" naming convention documented in each module doc, but the actual test surface is not parity-shaped: 12 of the 14 files run only on SQLite and perform only service-fn + bus assertion, with zero cross-adapter shape. The "parity" claim in the file-level docs (e.g., "Runs on SQLite (always) + PG/MySQL (env-gated)") is misleading because the env-gated variants are absent or empty in 5 of 14 files (see PAR-005/006/007). A reader scanning the module docs would believe parity coverage exists where it does not.
- **expected:** Per `docs/build-plan.md:1653-1656`: the module doc must accurately describe what the test exercises.
- **evidence:** Compare `finance_integration.rs:1-16` ("Runs on SQLite (always) + PG/MySQL (env-gated)") with the file's actual contents — no `#[ignore]`-d PG/MySQL test exists, and no `setup_pg`/`setup_mysql` call appears anywhere in the file.

---

### FINDING 29

- **id:** PAR-029
- **area:** tools-parity
- **severity:** Low
- **location:** `crates/tools/storage-parity/tests/finance_integration.rs:376-393` + `library_integration.rs:356-393` + `attendance_integration.rs:619-784` (entire file end)
- **description:** The `#[test]` (sync) variants of the event-type round-trip tests duplicate `assert_eq!` checks on `EVENT_TYPE` and `AGGREGATE_TYPE` constants, but they exercise the constants directly without going through `serde_json::to_value` + `from_value`. A back-end that returns a payload whose `event_type` field is serialized differently (e.g., the engine's `surrealdb` payload collapse documented in PAR-003) would not be caught by these sync round-trip tests because they only verify the constant string, not the wire form.
- **expected:** Per `docs/schemas/event-schema.md § 5` (event schema) + `docs/ports/storage.md § 4`: event types are wire-form identifiers and must round-trip through JSON serialization identically on every backend.
- **evidence:**
  ```rust
  // crates/tools/storage-parity/tests/finance_integration.rs:267-280
  #[test]
  fn finance_event_type_round_trip_for_all_headline_aggregates() {
      ...
      assert_eq!(
          <InvoiceNumberingConfigured as DomainEvent>::EVENT_TYPE,
          "finance.fees_invoice.configured"
      );
  ```
  No `serde_json::to_string(&ev)` + `from_str` round-trip is performed.

---

### FINDING 30

- **id:** PAR-030
- **area:** tools-parity
- **severity:** Low
- **location:** `crates/tools/storage-parity/tests/cms_integration.rs:495-647` + `documents_integration.rs:577-885` + `events_integration.rs:371-685`
- **description:** The CMS, Documents, and Events integration tests construct their own in-memory mocks (`InMemoryAuditLog`, `InMemoryPageRepo`, `InMemoryFormRepo`) rather than going through the `educore-storage-surrealdb` adapter or `educore-testkit::storage`. The mocks implement `AuditLog`, `PageRepository`, etc. directly via `async_trait`, bypassing the parity suite's own setup helpers (`common::setup_sqlite`, `setup_surrealdb`, `setup_testkit`). A parity gap between the mock impl and the real adapter impl is invisible — the test passes when the mocks behave correctly, regardless of whether the adapters do.
- **expected:** Per `crates/tools/testkit/src/storage.rs` (in-memory adapter exists) + `crates/adapters/storage-sqlite/src/`: the parity suite must use the same backend plumbing the engine ships, not bespoke in-memory mocks.
- **evidence:** `grep -rln "InMemoryAuditLog\|InMemoryPageRepo\|InMemoryFormRepo" crates/tools/storage-parity/tests/` returns 3 files (`cms_integration.rs`, `documents_integration.rs`, `events_integration.rs`). None of them use `common::setup_testkit()` or `common::setup_sqlite()`.

---

### FINDING 31

- **id:** PAR-031
- **area:** tools-parity
- **severity:** Low
- **location:** `crates/tools/storage-parity/Cargo.toml:32-38` (`[dev-dependencies]`) + `docs/build-plan.md:1699` (Exit criteria 4: "`cargo test --workspace` green")
- **description:** The `Cargo.toml` `[dev-dependencies]` list pulls all four production adapters (`educore-storage-sqlite`, `educore-storage-postgres`, `educore-storage-mysql`, `educore-storage-surrealdb`) plus `educore-testkit` plus all five Phase 15 port adapters. This list is a single source of CI compile time pressure — every PR's `cargo test --workspace` must compile all 10 adapter crates even if the parity test surface uses only `educore-storage-sqlite` (per PAR-005/006/007/019/025/026). The dependency graph does not feature-gate the adapters, so a CI runner that lacks network access to crates.io cannot run the suite even though all 14 domain integration tests are SQLite-only.
- **expected:** Per `docs/code-standards.md` + ADR-015-ExternalCrates.md: external crate selection must consider cross-compile and CI isolation; a parity suite that requires 10 adapter crates to compile but exercises only 1–2 of them in practice is a maintenance and CI cost without a corresponding test coverage benefit.
- **evidence:**
  ```toml
  # crates/tools/storage-parity/Cargo.toml:32-38
  [dev-dependencies]
  educore-storage-sqlite = { workspace = true }
  educore-storage-postgres = { workspace = true }
  educore-storage-mysql = { workspace = true }
  educore-storage-surrealdb = { workspace = true }
  educore-testkit = { workspace = true }
  educore-auth = { workspace = true }
  educore-notify = { workspace = true }
  educore-payment = { workspace = true }
  educore-files = { workspace = true }
  educore-integrations = { workspace = true }
  ```
