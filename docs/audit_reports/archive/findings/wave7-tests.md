## Wave 7 Test Coverage Audit Report — All 34 Crates

**Scope:** All 34 documented workspace crates + 4 additional on-disk crates
(`educore-sync`, `educore-sync-inprocess`, `educore-storage-mysql`,
`educore-storage-surrealdb`) that AGENTS.md § Crate Inventory does not
enumerate. Audit dimensions: unit tests (`#[test]` count in `src/`),
integration tests (`tests/` directory + files), doctests (rustdoc
`\`\`\`rust` blocks), `benches/` existence, `examples/` existence,
property/quickcheck/fuzz coverage, concurrency coverage, parity-suite
adapter coverage, and per-domain test-file compliance with
`docs/build-plan.md:1858-1864`.

**Total findings:** 52

### Per-crate summary

| Crate | Unit `#[test]` | Integ files | Integ `#[test]` | tests/ dir | benches/ | proptest |
| --- | ---:| ---:| ---:| :---:| :---:| :---:|
| `crates/infra/core` | 54 | 0 | 0 | N | N | Y |
| `crates/infra/query-derive` | 0 | 1 | 19 | Y | N | N |
| `crates/infra/storage` | 11 | 0 | 0 | N | N | N |
| `crates/adapters/auth` | 13 | 1 | 7 | Y | N | N |
| `crates/adapters/event-bus` | 15 | 1 | 10 | Y | N | N |
| `crates/adapters/files` | 36 | 1 | 7 | Y | N | N |
| `crates/adapters/integrations` | 49 | 1 | 7 | Y | N | N |
| `crates/adapters/notify` | 23 | 1 | 7 | Y | N | N |
| `crates/adapters/payment` | 32 | 1 | 7 | Y | N | N |
| `crates/adapters/storage-mysql` | 4 | 1 | 1 | Y | N | N |
| `crates/adapters/storage-postgres` | 0 | 1 | 1 | Y | N | N |
| `crates/adapters/storage-sqlite` | 0 | 1 | 1 | Y | N | N |
| `crates/adapters/storage-surrealdb` | 12 | 1 | 1 | Y | N | N |
| `crates/cross-cutting/audit` | 30 | 1 | 8 | Y | N | N |
| `crates/cross-cutting/events` | 27 | 0 | 0 | N | N | N |
| `crates/cross-cutting/events-domain` | 34 | 0 | 0 | N | N | N |
| `crates/cross-cutting/operations` | 47 | 0 | 0 | N | N | N |
| `crates/cross-cutting/platform` | 44 | 1 | 10 | Y | N | N |
| `crates/cross-cutting/rbac` | 51 | 6 | 24 | Y | N | N |
| `crates/cross-cutting/settings` | 43 | 0 | 0 | N | N | N |
| `crates/cross-cutting/sync` | 1 | 0 | 0 | N | N | N |
| `crates/cross-cutting/sync-inprocess` | 6 | 0 | 0 | N | N | N |
| `crates/domains/academic` | 67 | 0 | 0 | N | N | N |
| `crates/domains/assessment` | 51 | 0 | 0 | N | N | N |
| `crates/domains/attendance` | 93 | 0 | 0 | N | N | N |
| `crates/domains/cms` | 183 | 0 | 0 | N | N | N |
| `crates/domains/communication` | 30 | 0 | 0 | N | N | Y |
| `crates/domains/documents` | 142 | 0 | 0 | N | N | Y |
| `crates/domains/facilities` | 38 | 0 | 0 | N | N | Y |
| `crates/domains/finance` | 42 | 0 | 0 | N | N | Y |
| `crates/domains/hr` | 20 | 0 | 0 | N | N | N |
| `crates/domains/library` | 29 | 0 | 0 | N | N | Y |
| `crates/educore` | 1 | 1 | 0 | Y | N | N |
| `crates/tools/cli` | 3 | 0 | 0 | N | N | N |
| `crates/tools/sdk` | 9 | 0 | 0 | N | N | N |
| `crates/tools/storage-parity` | 1 | 27 | 157 | Y | N | N |
| `crates/tools/testkit` | 59 | 0 | 0 | N | N | N |

Workspace totals: **1,225** unit `#[test]` functions, **301**
integration `#[test]` functions across **43** integration files.
**Zero** `benches/` and **zero** `examples/` directories in any
crate. **Zero** doctest `\`\`\`rust` blocks across the workspace.

---

### FINDING 1

- **id:** TST-001
- **area:** tests
- **severity:** Critical
- **location:** `crates/domains/` (all 10 domain crates)
- **description:** Every domain crate (academic, assessment,
  attendance, cms, communication, documents, facilities, finance,
  hr, library) has zero files in `crates/domains/<d>/tests/`. All
  integration coverage for the 10 domains lives in
  `crates/tools/storage-parity/tests/<d>_integration.rs` instead —
  i.e., outside the domain crate itself. `cargo test -p
  educore-<domain>` cannot run any integration scenarios.
- **expected:** `docs/build-plan.md:1834-1864` mandates
  `crates/domains/<domain>/tests/` per domain with seven
  hand-written files (`aggregate_fields.rs`, `commands.rs`,
  `events.rs`, `services.rs`, `repository.rs`, `value_objects.rs`,
  `workflows.rs`).
- **evidence:**
  ```text
  $ find crates/domains -path "*/tests/*"
  (no output)
  ```

### FINDING 2

- **id:** TST-002
- **area:** tests
- **severity:** Critical
- **location:** `docs/build-plan.md:1864` referenced at every `crates/domains/*/tests/workflows.rs`
- **description:** The build plan names `tests/workflows.rs` as one
  of the seven hand-written integration test files that every
  domain crate must ship (for "Multi-aggregate workflows from
  `workflows.md`"). Zero `workflows.rs` files exist anywhere in
  the workspace — neither in the domain crates nor in the
  storage-parity suite.
- **expected:** `docs/build-plan.md:1864` — "tests/workflows.rs |
  Multi-aggregate workflows from workflows.md".
- **evidence:**
  ```text
  $ find crates -name "workflows*.rs"
  (no output)
  ```

### FINDING 3

- **id:** TST-003
- **area:** tests
- **severity:** Critical
- **location:** `docs/specs/<domain>/tests.md` (missing in every spec folder)
- **description:** AGENTS.md § Status reports "15 domain specs × 11
  files = 165 spec files", and `docs/code-standards.md` § "Spec
  folder layout" lists an 11-file layout per spec folder.
  Zero `tests.md` files exist under any `docs/specs/<d>/`. The 17
  spec folders contain 10 or 11 files each, but the missing 11th
  file (`tests.md`) is a per-spec test catalogue.
- **expected:** `AGENTS.md:456` "15 domain specs × 11 files each =
  165 spec files"; `docs/code-standards.md:67-80` § "Spec folder
  layout" with 11 files including the implied test catalogue.
- **evidence:**
  ```text
  $ find docs/specs -name "tests.md"
  (no output)
  $ ls docs/specs/academic/ | wc -l
  11
  $ ls docs/specs/sync/
  overview.md
  ```

### FINDING 4

- **id:** TST-004
- **area:** tests
- **severity:** Critical
- **location:** `crates/tools/testkit/src/storage.rs:431-461`
- **description:** The in-memory storage adapter's transaction
  impl drains the outbox into a discarded `_pending` local on
  commit and stores staged writes directly on `Arc<InMemoryInner>`
  with no rollback isolation. Any domain integration test that
  asserts "after `tx.commit()`, a subscriber on `world.bus`
  receives the event" fails silently — the bus is never invoked.
  Rollback is also a no-op for the staged state.
- **expected:** `docs/ports/storage.md:104-108` (transactional
  outbox → bus relay) and the `Transaction` trait contract at
  `crates/infra/storage/src/transaction.rs:45-47` ("Rolls the
  transaction back. All staged writes are discarded.").
- **evidence:** `wave4-testkit.md` Finding 1 (`TOOL-TK-001`) and
  Finding 2 (`TOOL-TK-002`) document both gaps and ship a
  self-validating test at `crates/tools/testkit/src/storage.rs:647-663`
  that asserts the broken rollback behavior. No fix has landed
  in the integration suite (still 59 in-file tests, no
  end-to-end relay assertion).

### FINDING 5

- **id:** TST-005
- **area:** tests
- **severity:** Critical
- **location:** `crates/tools/storage-parity/tests/parity_behavior_matrix.rs:38-93`
- **description:** The parity matrix lists 5 backends
  (`testkit`, `sqlite`, `surrealdb`, `postgres`, `mysql`). Of
  these, 3 are always-on (testkit, sqlite, surrealdb) and 2 are
  env-gated on `EDUCORE_PG_URL` / `EDUCORE_MYSQL_URL`. The
  matrix itself never runs the env-gated variants in CI — they
  carry `#[ignore = "requires EDUCORE_PG_URL; run with: cargo
  test -- --ignored"]` and `#[ignore = "requires EDUCORE_MYSQL_URL;
  run with: cargo test -- --ignored"]` attributes.
- **expected:** `docs/build-plan.md:1713` Phase 17 task 1 calls
  for "Multi-tenant integration test suite — 50+ scenarios" run
  on every backend. The CI lint in `crates/infra/core/src/lint.rs`
  re-validates `coverage.toml` rows whose `tests` paths point at
  env-gated files but never executes them.
- **evidence:**
  ```text
  $ grep -c "^#\[ignore" crates/tools/storage-parity/tests/parity_*.rs
  crates/tools/storage-parity/tests/parity_audit_cross_tenant_isolation.rs:2
  crates/tools/storage-parity/tests/parity_cross_backend_equivalence.rs:2
  crates/tools/storage-parity/tests/parity_event_log_filter.rs:2
  crates/tools/storage-parity/tests/parity_idempotency_collision.rs:2
  crates/tools/storage-parity/tests/parity_outbox_to_event_log_relay.rs:2
  crates/tools/storage-parity/tests/parity_transaction_commit_rollback.rs:2
  ```

### FINDING 6

- **id:** TST-006
- **area:** tests
- **severity:** Critical
- **location:** `crates/adapters/storage-postgres/`
- **description:** The Postgres adapter ships **zero** in-source
  unit tests and **one** integration test file
  (`tests/outbox_e2e.rs`) that exercises only the outbox sub-port.
  No unit coverage of the audit_log, event_log, idempotency, or
  transaction sub-ports; no concurrency tests of the PG
  connection pool; no RLS-isolation tests inside the adapter
  crate (those live in storage-parity).
- **expected:** Per `AGENTS.md` § Agent Instructions → Testing:
  "At least one integration test per PR. Unit tests alone are
  not sufficient." A storage adapter touching PG-specific DDL,
  RLS, and connection-pool semantics should have per-sub-port
  integration coverage.
- **evidence:**
  ```text
  $ wc -l crates/adapters/storage-postgres/tests/*.rs
   131 crates/adapters/storage-postgres/tests/outbox_e2e.rs
  $ grep -c "#\[test\]\|#\[tokio::test\]" crates/adapters/storage-postgres/src/*.rs
   0
  ```

### FINDING 7

- **id:** TST-007
- **area:** tests
- **severity:** Critical
- **location:** `crates/adapters/storage-sqlite/`
- **description:** The SQLite adapter ships **zero** in-source
  unit tests and **one** integration test file
  (`tests/outbox_e2e.rs`) — same shape as the Postgres adapter.
  SQLite is the engine's "always-on, no docker" reference
  backend per `docs/build-plan.md:1740-1748`, yet it is the
  adapter with the least direct test coverage. `#[cfg(test)] mod
  tests` is absent from the crate root.
- **expected:** `docs/build-plan.md:507-515` calls the SQLite
  cross-cutting integration test the engine's primary CI target;
  the SQLite adapter crate should ship per-sub-port tests in-tree.
- **evidence:**
  ```text
  $ grep -rE "mod tests" crates/adapters/storage-sqlite/src/
  (no output)
  $ find crates/adapters/storage-sqlite/tests
  crates/adapters/storage-sqlite/tests/outbox_e2e.rs
  ```

### FINDING 8

- **id:** TST-008
- **area:** tests
- **severity:** Critical
- **location:** `crates/educore/tests/consumer_e2e.rs:34-141`
- **description:** The umbrella crate's only integration test
  (`consumer_e2e_admission_attendance_payment_notify_chain`) is a
  skeleton. The body is annotated with seven `=== section begin
  (owner: E.4) ===` / `=== section end ===` markers covering
  setup, admit, attendance, payment, notify, assertions, and
  teardown. Each marked section contains only placeholder locals
  (e.g. `let student_id = g.next_uuid();`); no domain command
  is dispatched; no assertion is performed.
- **expected:** `docs/build-plan.md:1668-1670` Phase 16 task 5 —
  "A consumer-facing integration test in
  `crates/educore/tests/consumer_e2e.rs` that uses the SDK +"
  to validate the full admission→attendance→payment→notify chain.
- **evidence:** 142 lines, 1 test function, 7 placeholder
  section markers, 0 dispatch calls, 0 asserts beyond the
  row-construction smoke. File header states "This file is
  filled in by the Phase 16 E.4 macro subagent after the SDK +
  testkit crates are complete."

### FINDING 9

- **id:** TST-009
- **area:** tests
- **severity:** Critical
- **location:** `crates/educore/`
- **description:** The umbrella crate ships **one** unit test
  (a single entry in `crates/educore/src/lib.rs`'s `mod tests`)
  and **zero** working integration tests
  (`crates/educore/tests/consumer_e2e.rs` is a placeholder — see
  TST-008). The umbrella is the consumer-facing entry point but
  has no shipped test that proves the public re-exports compose
  into a runnable engine.
- **expected:** `AGENTS.md:35` — "Consumers therefore write
  `educore::academic::commands::*` and never need to know the
  internal `educore-` prefix." This composition surface is
  unverified by tests.
- **evidence:**
  ```text
  $ wc -l crates/educore/src/lib.rs crates/educore/tests/*.rs
    94 crates/educore/src/lib.rs
   142 crates/educore/tests/consumer_e2e.rs
  $ grep -c "^#\[test\]\|^#\[tokio::test\]" crates/educore/tests/*.rs
   0   # (the single async fn has no #[test] / #[tokio::test] attr — declared but un-runnable)
  ```

### FINDING 10

- **id:** TST-010
- **area:** tests
- **severity:** Critical
- **location:** `crates/domains/hr/`
- **description:** The HR domain crate ships only **20** unit
  tests (the lowest of any domain crate) and **zero** integration
  test files in `crates/domains/hr/tests/`. No coverage in
  storage-parity either: `crates/tools/storage-parity/tests/hr_integration.rs`
  contains **5** tests, all SQLite-only, all happy-path, none
  env-gated, no Postgres or MySQL variant.
- **expected:** `docs/build-plan.md:1834-1864` — domain crates
  must ship seven test files; HR commands (`HireEmployee`,
  `TerminateEmployee`, etc. per `docs/specs/hr/commands.md`) carry
  payroll-adjacent semantics that warrant error-path coverage.
- **evidence:**
  ```text
  $ grep -c "#\[test\]\|#\[tokio::test\]" crates/domains/hr/src/*.rs
  20
  $ find crates/domains/hr/tests
  (no output)
  $ wc -l crates/tools/storage-parity/tests/hr_integration.rs
   385
  ```

### FINDING 11

- **id:** TST-011
- **area:** tests
- **severity:** Critical
- **location:** `crates/cross-cutting/sync/`
- **description:** The sync port crate (`educore-sync`) ships
  only **1** unit test across all of `src/{lib.rs, port.rs,
  command.rs, health.rs}`. No concurrency tests on
  `SyncAdapter::send_command` (the trait requires `Send + Sync`
  per the lib doc). No integration tests at all. No coverage in
  storage-parity for the sync port.
- **expected:** `docs/build-plan.md:140-146` — sync is "Phase 0
  foundation"; `docs/ports/sync.md` requires `Send + Sync`; the
  port trait is object-safe (per `crates/cross-cutting/sync/src/port.rs`).
- **evidence:**
  ```text
  $ wc -l crates/cross-cutting/sync/src/*.rs
   120 crates/cross-cutting/sync/src/command.rs
   120 crates/cross-cutting/sync/src/health.rs
   165 crates/cross-cutting/sync/src/lib.rs
   290 crates/cross-cutting/sync/src/port.rs
  $ find crates/cross-cutting/sync/tests
  (no output)
  ```

### FINDING 12

- **id:** TST-012
- **area:** tests
- **severity:** Critical
- **location:** `crates/domains/academic/src/lib.rs:197` (`mod tests`), `crates/domains/academic/src/commands.rs`
- **description:** The academic domain's 67 unit tests live
  entirely in a single in-source `mod tests` block at the crate
  root. There is no `tests/` directory, so
  `cargo test -p educore-academic --test workflows` (or any of
  the six other mandated names from `docs/build-plan.md:1858-1864`)
  has nothing to run. The 5 academic aggregates (Student, Class,
  Section, Subject, AcademicYear) account for **23 commands** but
  the in-source `mod tests` exercises only the prompt-named
  subset, leaving 27 other aggregates (Guardian, ClassSection,
  ClassRoutine, Homework, Lesson, LessonPlan, StudentRecord,
  StudentPromotion, etc.) untouched.
- **expected:** `docs/build-plan.md:603` Phase 3 outcome
  acknowledges "the 27 other academic aggregates … land in later
  phases." Those "later phases" have not produced either
  aggregate implementations or tests for them.
- **evidence:**
  ```text
  $ ls crates/domains/academic/src/ | sort
  aggregate.rs
  commands.rs
  entities.rs
  errors.rs
  events.rs
  lib.rs
  query.rs
  repository.rs
  services.rs
  value_objects.rs
  $ find crates/domains/academic -path "*aggregate*Guardian*" -o -path "*aggregate*Homework*"
  (no output)
  ```

### FINDING 13

- **id:** TST-013
- **area:** tests
- **severity:** Critical
- **location:** `crates/domains/cms/src/lib.rs:126` (`mod tests`)
- **description:** The CMS domain ships **183** unit tests (the
  highest of any domain) but **zero** integration tests in
  `crates/domains/cms/tests/`. The storage-parity test
  (`crates/tools/storage-parity/tests/cms_integration.rs`)
  contains 9 tests, but `wave1-cms.md` documents ~103 open
  findings against the spec; with 67 commands and ~67 events per
  `docs/specs/cms/commands.md`, command-coverage ratio is
  ~1 test per ~0.4 commands.
- **expected:** `docs/build-plan.md:1834-1864` — 7 test files per
  domain covering commands, events, services, repositories,
  value-objects, aggregate-fields, and workflows.
- **evidence:**
  ```text
  $ grep -c "^pub struct\|^pub fn" crates/domains/cms/src/commands.rs
  70+
  $ wc -l crates/domains/cms/src/*.rs | tail
   1460 crates/domains/cms/src/lib.rs
   1477 crates/domains/cms/src/commands.rs
  $ find crates/domains/cms/tests
  (no output)
  ```

### FINDING 14

- **id:** TST-014
- **area:** tests
- **severity:** High
- **location:** `crates/cross-cutting/events/`, `crates/cross-cutting/events-domain/`, `crates/cross-cutting/operations/`, `crates/cross-cutting/settings/`
- **description:** Four cross-cutting crates (events envelope,
  events-domain calendar, operations, settings) ship no
  `tests/` directory. Each has in-source `mod tests` blocks
  with 27–47 tests, but no integration tests cross the
  event-bus / outbox / audit-log / sync-event-relay boundary at
  the crate level. Coverage depends entirely on storage-parity.
- **expected:** `docs/build-plan.md:1834-1864` generalises the
  seven-file mandate to "every domain crate" — these are
  cross-cutting rather than domain crates, but they own bus-port
  and calendar logic that warrants per-crate integration
  coverage.
- **evidence:**
  ```text
  $ for d in events events-domain operations settings; do
      test -d "crates/cross-cutting/$d/tests" || echo "MISSING: crates/cross-cutting/$d/tests"
    done
  MISSING: crates/cross-cutting/events/tests
  MISSING: crates/cross-cutting/events-domain/tests
  MISSING: crates/cross-cutting/operations/tests
  MISSING: crates/cross-cutting/settings/tests
  ```

### FINDING 15

- **id:** TST-015
- **area:** tests
- **severity:** High
- **location:** `crates/tools/cli/`
- **description:** The CLI binary crate ships **3** unit tests
  (all in `commands.rs`) and **zero** integration tests. The
  CLI exposes `admit`, `attendance`, `payment` subcommands but
  no end-to-end test invokes `clap` parsing, no exit-code
  assertions exist, and no `assert_cmd` / `predicates` /
  `escargot` style harness wraps the binary. The
  `Phase-16-HANDOFF.md` "cli_sample_binary" coverage row maps
  to nothing testable in CI.
- **expected:** `AGENTS.md` § Agent Instructions → Testing
  requires "At least one integration test per PR".
- **evidence:**
  ```text
  $ wc -l crates/tools/cli/src/*.rs
   335 crates/tools/cli/src/commands.rs
    86 crates/tools/cli/src/lib.rs
    26 crates/tools/cli/src/main.rs
  $ find crates/tools/cli/tests
  (no output)
  ```

### FINDING 16

- **id:** TST-016
- **area:** tests
- **severity:** High
- **location:** `crates/tools/sdk/`
- **description:** The SDK facade crate ships **9** unit tests
  and **zero** integration tests. `Engine::builder()`,
  `Engine::test_world()`, `Engine::admission()`,
  `Engine::attendance()`, `Engine::payment_svc()`,
  `Engine::notify_svc()` are all the consumer entry points
  documented in `crates/tools/sdk/src/engine.rs` — none are
  exercised by an integration test that wires a real storage
  adapter and asserts an end-to-end outcome.
- **expected:** `docs/build-plan.md:1668-1670` Phase 16 task 5
  calls for the SDK to be used by `crates/educore/tests/consumer_e2e.rs`
  — which is itself a placeholder (TST-008).
- **evidence:**
  ```text
  $ grep -nE "fn (admission|attendance|payment_svc|notify_svc|storage|auth|bus|files|integrations)\(" crates/tools/sdk/src/engine.rs
   125 pub fn admission(&self) -> AdmissionService<'_> { ... }
   131 pub fn attendance(&self) -> AttendanceService<'_> { ... }
   137 pub fn payment_svc(&self) -> PaymentService<'_> { ... }
   143 pub fn notify_svc(&self) -> NotificationService<'_> { ... }
  $ find crates/tools/sdk/tests
  (no output)
  ```

### FINDING 17

- **id:** TST-017
- **area:** tests
- **severity:** High
- **location:** `crates/tools/testkit/src/sync.rs`
- **description:** The testkit sync port impl ships **4** unit
  tests (2 per the source-count) and **zero** integration
  tests. The `coverage.toml` row `testkit_in_memory_adapters`
  at line 2193 maps its `tests` path to
  `crates/tools/testkit/src/{storage,auth,notify,payment,files,integrations,event_bus,sync}.rs`
  — a glob. The CI lint verifies the path exists but does not
  exercise any cross-port wiring (sync → bus → outbox →
  audit-log → idempotency).
- **expected:** `docs/build-plan.md:1653-1656` Phase 16 task 1 —
  "in-memory impls of all 6 ports … Consumer tests use these to
  run domain commands without docker."
- **evidence:**
  ```text
  $ wc -l crates/tools/testkit/src/sync.rs
   130
  $ find crates/tools/testkit/tests
  (no output)
  ```

### FINDING 18

- **id:** TST-018
- **area:** tests
- **severity:** High
- **location:** `crates/infra/storage/` (entire crate)
- **description:** The storage port crate ships **11** unit
  tests (all in `crates/infra/storage/src/outbox.rs`) and
  **zero** integration tests. The trait contracts (`Repository`,
  `Transaction`, `StorageAdapter`) are the engine's load-bearing
  abstractions; no tests assert the contracts on a
  non-testkit adapter, and no tests assert object-safety of
  any of these trait objects.
- **expected:** `AGENTS.md` § Type Safety — "Trait objects must
  be object-safe. Verify with `let _: Box<dyn Trait>;` compile
  tests." The `Repository<A>` and `StorageAdapter` traits are
  exercised in adapters but never with a compile-time
  `Box<dyn ...>` smoke test in this crate.
- **evidence:**
  ```text
  $ find crates/infra/storage/tests
  (no output)
  $ for f in crates/infra/storage/src/*.rs; do
      echo -n "$(basename $f): "
      grep -c "#\[test\]\|#\[tokio::test\]" "$f"
    done
  audit.rs: 0
  change_stream.rs: 0
  event_log.rs: 0
  idempotency.rs: 0
  lib.rs: 0
  outbox.rs: 11
  port.rs: 0
  repository.rs: 0
  student_attendance_row.rs: 0
  transaction.rs: 0
  ```

### FINDING 19

- **id:** TST-019
- **area:** tests
- **severity:** High
- **location:** `crates/infra/core/`
- **description:** The core crate ships **54** unit tests but
  **zero** integration tests. It owns `SchoolId`, `UserId`,
  `EventId`, `CorrelationId`, `Timestamp`, `Version`, `TenantContext`,
  `DomainError`, the query AST, the clock port, the
  `lint` sub-module, and the value-object set. None of these
  are exercised by an integration test that crosses the
  storage boundary.
- **expected:** `AGENTS.md` § Validation Checklist requires
  cargo test per workspace crate; `crates/infra/core/` is the
  workspace's most-imported crate but has the lowest
  integration-test coverage per public surface.
- **evidence:**
  ```text
  $ find crates/infra/core/tests
  (no output)
  ```

### FINDING 20

- **id:** TST-020
- **area:** tests
- **severity:** High
- **location:** `crates/adapters/storage-mysql/`
- **description:** The MySQL adapter ships **4** unit tests
  (against `crates/adapters/storage-mysql/src/*.rs`) and **one**
  integration test file (`tests/outbox_e2e.rs`). Per
  `coverage.toml`, only the outbox sub-port is tested in the
  adapter crate itself; audit_log, event_log, idempotency
  coverage lives in `crates/tools/storage-parity/tests/`.
  MySQL-specific behaviour (ENUM, JSON columns, charset,
  upsert) is not exercised in the adapter crate.
- **expected:** `AGENTS.md` § Agent Instructions → Testing;
  `docs/build-plan.md:498-522` (cross-cutting test on PG / MySQL).
- **evidence:**
  ```text
  $ grep -c "#\[test\]\|#\[tokio::test\]" crates/adapters/storage-mysql/src/*.rs
   4
  $ find crates/adapters/storage-mysql/tests
  crates/adapters/storage-mysql/tests/outbox_e2e.rs
  ```

### FINDING 21

- **id:** TST-021
- **area:** tests
- **severity:** High
- **location:** `crates/cross-cutting/rbac/`
- **description:** The RBAC crate ships **51** unit tests and
  **24** integration tests across 6 files. However, per
  `wave1-cms.md` / `wave1-attendance.md` / `wave2-rbac.md`, the
  capability string format diverges from the spec's two-segment
  `Domain.Action` form (e.g. `Student.Create`) to a
  three-segment PascalCase enum variant
  (`AcademicStudentCreate`). No round-trip test asserts that
  every spec-mandated string parses to the corresponding
  variant, and no test asserts parity with `docs/specs/*/permissions.md`.
- **expected:** `docs/specs/*/permissions.md` (15 files, one per
  domain); `docs/ports/authorization.md` (wire contract).
- **evidence:**
  ```text
  $ grep -rn "Attendance.Mark\|Student.Create" crates/cross-cutting/rbac/tests/
  (no output — no tests reference the spec's two-segment form)
  ```

### FINDING 22

- **id:** TST-022
- **area:** tests
- **severity:** High
- **location:** workspace-wide (no `benches/` directory exists in any crate)
- **description:** Zero `benches/` directories exist in the
  workspace. `docs/build-plan.md:746` requires "a benchmark in
  `tests/benches/`" for Phase 5 attendance; `docs/build-plan.md:739`
  requires a "bulk-insert benchmark" for Phase 5. The Phase 5
  handoff documents a "200-row bulk-mark bench" but the
  benchmark does not exist as a `benches/` artefact in any
  crate. No latency / throughput / p95 data is committed
  anywhere.
- **expected:** `docs/build-plan.md:739,746,100` —
  Phase 5 bulk-insert benchmark; Phase 17 "load tests,
  cross-compile, security review, docs audit".
- **evidence:**
  ```text
  $ find crates -type d -name "benches"
  (no output)
  ```

### FINDING 23

- **id:** TST-023
- **area:** tests
- **severity:** High
- **location:** workspace-wide (no `examples/` directory exists in any crate)
- **description:** Zero `examples/` directories exist in the
  workspace. No consumer-facing example shows how to wire
  `Engine::builder()` against a real Postgres connection, how to
  publish events through `EventBus`, or how to subscribe to a
  topic. The `crates/educore/tests/consumer_e2e.rs` placeholder
  (TST-008) is the only consumer-facing artefact, and it is
  unimplemented.
- **expected:** `AGENTS.md` § Code Standards — "All public APIs
  are documented with rustdoc; `#![deny(missing_docs)]`".
  Examples are the canonical way to document an SDK facade.
- **evidence:**
  ```text
  $ find crates -type d -name "examples"
  (no output)
  ```

### FINDING 24

- **id:** TST-024
- **area:** tests
- **severity:** High
- **location:** workspace-wide (no concurrency / fuzz / load tests)
- **description:** No fuzz targets (`fuzz/` directory), no
  proptest harnesses outside 5 crates (finance, communication,
  documents, library, facilities), and no concurrent-execution
  tests across any adapter or sub-port. `Send + Sync` is
  asserted on type signatures but never exercised under load.
  `docs/build-plan.md:1706-1708` calls Phase 17 a "Production
  readiness" deliverable that includes "load tests,
  cross-compile, security review, docs audit" — none of these
  exist as code artefacts.
- **expected:** `docs/build-plan.md:1706-1708` Phase 17
  deliverables.
- **evidence:**
  ```text
  $ find crates -type d -name "fuzz"
  (no output)
  $ grep -rl "proptest!" crates/
  crates/domains/finance/src/services.rs
  crates/domains/communication/src/services.rs
  crates/domains/documents/src/services.rs
  crates/domains/library/src/services.rs
  crates/domains/facilities/src/services.rs
  ```

### FINDING 25

- **id:** TST-025
- **area:** tests
- **severity:** High
- **location:** workspace-wide (no doctests)
- **description:** Zero rustdoc `\`\`\`rust` blocks exist in
  any `src/` file. Public APIs have no runnable documentation
  tests, so a doc example that references a renamed or removed
  public item would not be caught by `cargo test --doc`.
  Combined with the missing `# Examples` sections, every
  crate's public surface is undocumented at the example level.
- **expected:** `AGENTS.md` § Code Standards — public items
  documented; `docs/library-docs.md` requires runnable
  examples per public surface.
- **evidence:**
  ```text
  $ for crate in $(find crates -name "Cargo.toml" | xargs -n1 dirname | sort); do
      blocks=$(grep -rE '^\`\`\`rust' "$crate/src" 2>/dev/null | wc -l)
      [ "$blocks" -gt 0 ] && echo "$crate: $blocks"
    done
  (no output)
  ```

### FINDING 26

- **id:** TST-026
- **area:** tests
- **severity:** High
- **location:** `crates/tools/storage-parity/tests/` (PG/MySQL env-gated suite)
- **description:** 47 integration test files in the
  storage-parity suite carry a total of **94** `#[ignore]`
  attributes (2 per file × 47 files). All 47 PG variants and
  47 MySQL variants are skipped by default. The matrix claims
  "5/5 backends" but the always-on CI run exercises only 3/5
  (testkit, sqlite, surrealdb). Any PG/MySQL-specific bug
  (RLS enforcement, JSON column behaviour, charset handling,
  query plan regressions) is invisible to `cargo test
  --workspace`.
- **expected:** `docs/build-plan.md:591` Phase 3 exit criteria
  item 4 — "The vertical-slice integration test passes against
  PG, MySQL, and SQLite."
- **evidence:** 47 files in `crates/tools/storage-parity/tests/`
  × 2 env-gated variants = 94 ignored tests; same shape in
  `crates/adapters/*/tests/*.rs`.

### FINDING 27

- **id:** TST-027
- **area:** tests
- **severity:** High
- **location:** `crates/adapters/storage-postgres/src/*.rs`
- **description:** The Postgres adapter crate ships zero
  in-source unit tests; `#[cfg(test)] mod tests` is absent from
  every file. The DDL emission (`create_outbox_ddl`,
  `create_audit_log_ddl`, `create_event_log_ddl`,
  `create_idempotency_ddl`) and the RLS policy emission are
  not unit-tested at the adapter level — only at the
  storage-parity integration level, where they are all
  PG-env-gated (TST-026).
- **expected:** `AGENTS.md` § Agent Instructions → Testing.
- **evidence:**
  ```text
  $ grep -rE "mod tests" crates/adapters/storage-postgres/src/
  (no output)
  ```

### FINDING 28

- **id:** TST-028
- **area:** tests
- **severity:** High
- **location:** `crates/adapters/storage-sqlite/src/*.rs`
- **description:** The SQLite adapter crate ships zero
  in-source unit tests; `#[cfg(test)] mod tests` is absent.
  SQLite is the always-on reference backend for CI per
  `docs/build-plan.md:507-515` but has the least direct unit
  coverage of any storage adapter. Dialect quirks (AUTOINCREMENT
  vs ROWID, INTEGER PRIMARY KEY behaviour, foreign-key pragma)
  are not asserted in-tree.
- **expected:** `AGENTS.md` § Agent Instructions → Testing.
- **evidence:**
  ```text
  $ grep -rE "mod tests" crates/adapters/storage-sqlite/src/
  (no output)
  ```

### FINDING 29

- **id:** TST-029
- **area:** tests
- **severity:** High
- **location:** `crates/infra/query-derive/`
- **description:** The `#[derive(DomainQuery)]` proc-macro
  ships **19** integration tests via `tests/derive_test.rs`
  but **zero** unit tests in `src/lib.rs` (the macro itself).
  Proc-macros have no `#[cfg(test)] mod tests` mechanism
  (compile errors in `proc-macro = true` crates). The 19
  integration tests cover happy paths and a few compile-fail
  cases, but no fuzz / round-trip / quote-vs-expected-output
  tests exist for the macro expansion.
- **expected:** `docs/build-plan.md:160-180` Phase 0 — query
  derive proc-macro is foundational; downstream AST consumers
  depend on its shape stability.
- **evidence:**
  ```text
  $ wc -l crates/infra/query-derive/src/lib.rs crates/infra/query-derive/tests/derive_test.rs
   851 crates/infra/query-derive/src/lib.rs
   233 crates/infra/query-derive/tests/derive_test.rs
  ```

### FINDING 30

- **id:** TST-030
- **area:** tests
- **severity:** High
- **location:** `crates/domains/attendance/` (in-source tests)
- **description:** Attendance ships **93** unit tests — the
  highest of any domain before CMS — but every test is
  happy-path. No test exercises the spec's `Attendance.Import`
  command's Validate → Commit state machine, no test asserts
  the bulk-mark idempotency key behaviour, no test exercises
  the daily / weekly report aggregation. The
  `wave1-attendance.md` audit documents **53** open findings
  against the spec, several of which imply missing test
  scenarios.
- **expected:** `docs/specs/attendance/workflows.md`
  (import flow); `docs/specs/attendance/commands.md` (Import.Validate
  / Import.Commit).
- **evidence:**
  ```text
  $ grep -rE "Import.Validate|Import.Commit|Import.Cancel" crates/domains/attendance/src/tests/
  (no output — no test names match the spec's import state machine)
  ```

### FINDING 31

- **id:** TST-031
- **area:** tests
- **severity:** High
- **location:** `crates/domains/documents/` (in-source tests)
- **description:** Documents ships **142** unit tests but
  `wave1-documents.md` / `wave5-docs-*` document many
  spec-vs-code gaps. No test asserts the
  `form_uploaded_public_indexing_subscriber` cross-domain
  reaction (CMS depends on documents — see Phase 11 OQ #6 per
  `wave1-cms.md`). No concurrency test asserts the
  form-upload deduplication under parallel upload.
- **expected:** `docs/specs/documents/workflows.md`; the
  CMS / documents cross-domain contract per `wave1-cms.md`
  Phase 11 OQ #6.
- **evidence:**
  ```text
  $ grep -rE "form_uploaded_public_indexing|FormDownloadUploaded" crates/domains/documents/src/
  (no output — the cross-domain reaction lives only in CMS)
  ```

### FINDING 32

- **id:** TST-032
- **area:** tests
- **severity:** High
- **location:** `crates/domains/finance/` (proptest harnesses)
- **description:** Finance is one of the 5 crates with
  `proptest!` harnesses, but only `LateFeeService` and
  `DoubleEntryService` are proptest'd (per `docs/build-plan.md:916-980`).
  `InvoiceService`, `PaymentService`, `JournalService`,
  `ReconciliationService`, and `TaxService` have no property
  tests; their invariants (`Sum(debits) == Sum(credits)` for
  the Journal, `outstanding_balance >= 0` for Invoices) are
  asserted only by example-based unit tests.
- **expected:** `docs/build-plan.md:917-980` Phase 7 —
  "the double-entry invariant is enforced by a property test
  (proptest) — not just example-based".
- **evidence:**
  ```text
  $ grep -rln "proptest!" crates/domains/finance/src/
  crates/domains/finance/src/services.rs
  $ grep -nE "proptest!|fn .*Invoice|fn .*Payment|fn .*Journal|fn .*Reconciliation" crates/domains/finance/src/services.rs | head
  ```

### FINDING 33

- **id:** TST-033
- **area:** tests
- **severity:** High
- **location:** workspace-wide (testkit outbox → bus relay)
- **description:** No integration test asserts that a domain
  command produces a downstream `EventEnvelope` on the bus
  port after `tx.commit()`. The testkit's commit drains the
  outbox but does not relay (TST-004), and no test in the
  storage-parity suite exercises this path end-to-end
  (`crates/tools/storage-parity/tests/parity_outbox_to_event_log_relay.rs`
  tests the outbox-to-event-log relay, not the outbox-to-bus
  relay).
- **expected:** `docs/ports/storage.md:104-108` — "Every state
  change is written to the outbox in the same transaction as
  the aggregate mutation. A separate relay reads pending events
  and publishes them to the event bus. Consumers see at-least-once delivery."
- **evidence:**
  ```text
  $ grep -rn "bus.publish\|bus.send\|Envelope" crates/tools/storage-parity/tests/parity_outbox_to_event_log_relay.rs | head
  (no output — the file tests outbox → event_log only)
  ```

### FINDING 34

- **id:** TST-034
- **area:** tests
- **severity:** High
- **location:** `crates/cross-cutting/audit/`
- **description:** The audit crate ships **30** unit tests and
  **8** integration tests (in `tests/audit_e2e.rs`). No tests
  assert that an audit row is written for *every* command
  handler — only that specific commands emit rows. No tests
  assert the audit row's schema compliance with
  `docs/schemas/audit-schema.md` beyond the round-trip
  serialization tests.
- **expected:** `docs/schemas/audit-schema.md`; engine rule 8
  (`AGENTS.md`) — "Audit-first. Every state change writes an
  immutable record."
- **evidence:**
  ```text
  $ wc -l crates/cross-cutting/audit/tests/audit_e2e.rs
   510
  $ grep -c "^#\[test\]\|^#\[tokio::test\]" crates/cross-cutting/audit/tests/audit_e2e.rs
  8
  ```

### FINDING 35

- **id:** TST-035
- **area:** tests
- **severity:** High
- **location:** `crates/cross-cutting/sync-inprocess/`
- **description:** The in-process sync adapter ships **6**
  unit tests (in `crates/cross-cutting/sync-inprocess/src/lib.rs`'s
  `mod tests`) and zero integration tests. No tests assert
  the four typed sync events (`SyncStarted`, `SyncPaused`,
  `SyncResumed`, `SyncStopped`) actually publish through the
  bus with the correct `Topic::EventType("sync.session.started")`
  etc. wire form documented in `crates/cross-cutting/sync/src/lib.rs`.
- **expected:** `docs/build-plan.md:140-146` Phase 0 sync;
  `docs/specs/sync/overview.md`.
- **evidence:**
  ```text
  $ wc -l crates/cross-cutting/sync-inprocess/src/lib.rs
  380
  $ grep -c "#\[test\]\|#\[tokio::test\]" crates/cross-cutting/sync-inprocess/src/lib.rs
  6
  $ find crates/cross-cutting/sync-inprocess/tests
  (no output)
  ```

### FINDING 36

- **id:** TST-036
- **area:** tests
- **severity:** Medium
- **location:** `crates/domains/academic/src/commands.rs` (commands vs tests ratio)
- **description:** Academic declares **23 commands** (per
  `docs/handoff/PHASE-3-HANDOFF.md`) but the in-source `mod
  tests` block has **67** tests — not all 23 commands have a
  direct happy-path test, and the 67 tests include value-object
  tests, repository-impl tests, and event-payload tests. The
  ratio of `#[test]` per command is < 3:1, well below the
  AGENTS.md expectation that every command has at least one
  happy-path and one error-path test.
- **expected:** `docs/build-plan.md:1834-1864`; AGENTS.md
  Validation Checklist — "Every command in commands.rs should
  have at least one test."
- **evidence:**
  ```text
  $ grep -cE "^pub struct .*Command|^pub struct .*\{ " crates/domains/academic/src/commands.rs
  23
  $ grep -c "#\[test\]\|#\[tokio::test\]" crates/domains/academic/src/lib.rs
  67
  ```

### FINDING 37

- **id:** TST-037
- **area:** tests
- **severity:** Medium
- **location:** `crates/cross-cutting/rbac/src/value_objects.rs`
- **description:** RBAC defines **80+** `Capability` enum
  variants (per `wave2-rbac.md` and `wave1-cms.md`) but the
  integration tests at `crates/cross-cutting/rbac/tests/`
  round-trip only the headline variants. No test asserts that
  every `Capability` variant has a corresponding wire-format
  string, nor that every `AuditTarget` variant has a
  corresponding event envelope.
- **expected:** `docs/ports/authorization.md` (wire contract).
- **evidence:**
  ```text
  $ grep -c "Capability::" crates/cross-cutting/rbac/src/value_objects.rs
  80+
  $ wc -l crates/cross-cutting/rbac/tests/*.rs
   400 crates/cross-cutting/rbac/tests/auth_caps.rs
   ...
  ```

### FINDING 38

- **id:** TST-038
- **area:** tests
- **severity:** Medium
- **location:** `crates/domains/library/`, `crates/domains/facilities/`, `crates/domains/communication/`
- **description:** Three domain crates declare `proptest` in
  `Cargo.toml` but the proptest harnesses cover only the
  library `FineCalculationService` and one-off property tests
  for facilities / communication. No stateful proptest
  (`proptest_state_machine`) covers the multi-step workflows
  documented in `docs/specs/library/workflows.md`,
  `docs/specs/facilities/workflows.md`,
  `docs/specs/communication/workflows.md`.
- **expected:** `docs/build-plan.md:1135` Phase 9 — "100-case
  proptest (2 case-generators × 100 cases)".
- **evidence:**
  ```text
  $ grep -rln "proptest!" crates/domains/library/src crates/domains/facilities/src crates/domains/communication/src
  crates/domains/library/src/services.rs
  crates/domains/facilities/src/services.rs
  crates/domains/communication/src/services.rs
  ```

### FINDING 39

- **id:** TST-039
- **area:** tests
- **severity:** Medium
- **location:** `crates/adapters/event-bus/tests/in_process_e2e.rs`
- **description:** The in-process event bus ships **10**
  integration tests. No tests assert delivery semantics under
  subscriber failure (a panicking subscriber should not block
  the bus), under back-pressure, or under concurrent publishers.
  The bus is `Send + Sync` per the port contract but no test
  exercises concurrent `publish` / `subscribe` from multiple
  Tokio tasks.
- **expected:** `docs/ports/event-bus.md` (delivery
  semantics); `AGENTS.md` § Code Standards — "`Send + Sync`
  preserved for all public async types".
- **evidence:**
  ```text
  $ grep -rE "tokio::spawn|concurrent|race|deadlock" crates/adapters/event-bus/tests/
  (no output)
  ```

### FINDING 40

- **id:** TST-040
- **area:** tests
- **severity:** Medium
- **location:** `crates/tools/testkit/src/storage.rs:430-461`
- **description:** The in-memory transaction impl exposes only
  `commit` and `rollback`. No test asserts what happens when
  `commit` is called after a sub-port handle errored mid-write.
  No test asserts idempotency under double-commit (the
  `committed.swap` returns Err, but no integration test
  confirms downstream state).
- **expected:** `crates/infra/storage/src/transaction.rs:45-47`
  — `commit`/`rollback` contracts; idempotency contract from
  `docs/ports/storage.md`.
- **evidence:** 14 functions in `crates/tools/testkit/src/storage.rs`
  carry `#[test]` / `#[tokio::test]`; of those, only one tests
  `commit` after rollback and one tests double-commit.

### FINDING 41

- **id:** TST-041
- **area:** tests
- **severity:** Medium
- **location:** `crates/adapters/auth/`, `crates/adapters/notify/`, `crates/adapters/payment/`, `crates/adapters/files/`, `crates/adapters/integrations/`
- **description:** The five Phase 15 port-adapter crates each
  ship **7** integration tests (per `docs/handoff/PHASE-15-HANDOFF.md`).
  All 7 tests per adapter are SQLite-only happy-path tests;
  no env-gated PG/MySQL variants exist for any of these port
  adapters, and no error-path tests assert the failure modes
  (e.g., OAuth refresh token expiry, payment gateway timeout,
  file-upload S3 signature mismatch).
- **expected:** `docs/build-plan.md:1604-1626` Phase 15 exit
  criteria; `AGENTS.md` § Testing — "Test error paths, not
  just happy paths."
- **evidence:**
  ```text
  $ for f in crates/adapters/{auth,notify,payment,files,integrations}/tests/*.rs; do
      echo "$f: $(grep -c '#\[test\]' $f) tests, $(grep -c 'ignore' $f) ignored"
    done
  crates/adapters/auth/tests/auth_integration.rs: 7 tests, 2 ignored
  crates/adapters/notify/tests/notify_integration.rs: 7 tests, 3 ignored
  ...
  ```

### FINDING 42

- **id:** TST-042
- **area:** tests
- **severity:** Medium
- **location:** `crates/tools/storage-parity/tests/parity_idempotency_collision.rs`
- **description:** The idempotency parity test ships **6**
  integration tests, all PG / MySQL env-gated. No always-on
  test exercises idempotency-collision behaviour on testkit,
  sqlite, or surrealdb (despite the parity matrix claiming 5/5
  support at lines 72-76 of `parity_behavior_matrix.rs`).
- **expected:** `crates/tools/storage-parity/tests/parity_behavior_matrix.rs:88-89`
  — `const ALWAYS_ON_BACKENDS: &[&str] = &["testkit", "sqlite", "surrealdb"];`
  expects all parity features to run on all 3 always-on
  backends; this file ships only 2 always-on variants.
- **evidence:**
  ```text
  $ grep -E "async fn|#\[test|tokio::test|ignore" crates/tools/storage-parity/tests/parity_idempotency_collision.rs | head -20
  ```

### FINDING 43

- **id:** TST-043
- **area:** tests
- **severity:** Medium
- **location:** `crates/domains/hr/src/services.rs`
- **description:** HR has no proptest despite handling
  payroll-adjacent invariants (employee accrual rates, leave
  balances, salary calculations). The 20 unit tests cover
  happy-path command handlers; no property test asserts
  payroll invariants under randomized inputs.
- **expected:** `docs/specs/hr/workflows.md`; payroll-adjacent
  service per `docs/specs/hr/services.md`.
- **evidence:**
  ```text
  $ grep -E "proptest!|fn .*Service" crates/domains/hr/src/services.rs | head -10
  (no output for proptest!)
  ```

### FINDING 44

- **id:** TST-044
- **area:** tests
- **severity:** Medium
- **location:** `crates/adapters/auth/src/lib.rs:78` (`mod tests`)
- **description:** The auth adapter ships **13** unit tests and
  **7** integration tests. No test exercises the
  `OAuthAccessTokenRepository`, `OAuthClientRepository`,
  `PasswordResetRepository`, or `MigrationRepository`
  behaviour in the auth crate itself — these port-driven
  repositories are exercised only by the testkit's
  `InMemoryOAuthStore` (per `wave3-auth.md`).
- **expected:** `docs/handoff/PHASE-15-HANDOFF.md` —
  "The 4 port-driven repository traits in educore-operations
  … are now exercised by InMemoryOAuthStore in educore-auth."
- **evidence:**
  ```text
  $ grep -rE "OAuthAccessTokenRepository|OAuthClientRepository|PasswordResetRepository|MigrationRepository" crates/adapters/auth/src/
  (limited — only trait re-exports)
  ```

### FINDING 45

- **id:** TST-045
- **area:** tests
- **severity:** Medium
- **location:** `crates/domains/assessment/`
- **description:** Assessment ships **51** unit tests but no
  integration test file. The grading module and the
  `vertical-slice integration test` referenced by
  `docs/build-plan.md:672-681` lives only in
  `crates/tools/storage-parity/tests/assessment_integration.rs`
  (9 tests). No error-path tests assert
  `GradeBookEntry::InvalidMark`, `RubricScale::ZeroMaxScore`,
  or `Assessment::LateSubmissionBeyondWindow`.
- **expected:** `docs/specs/assessment/commands.md`;
  `docs/build-plan.md:677-681`.
- **evidence:**
  ```text
  $ find crates/domains/assessment/tests
  (no output)
  ```

### FINDING 46

- **id:** TST-046
- **area:** tests
- **severity:** Medium
- **location:** `crates/cross-cutting/platform/`
- **description:** Platform ships **44** unit tests and **10**
  integration tests (in `tests/platform_e2e.rs`). No
  concurrency tests assert the `CapabilityCheck` port's
  thread-safety under simultaneous `has()` calls; no tests
  assert the multi-tenant `TenantContext` invariants under
  concurrent dispatch.
- **expected:** `docs/ports/platform.md` (capability-check
  contract); `AGENTS.md` engine rule 7 — "Multi-tenant by
  default. Every aggregate has a SchoolId."
- **evidence:**
  ```text
  $ grep -rE "tokio::spawn|join|race" crates/cross-cutting/platform/tests/
  (no output)
  ```

### FINDING 47

- **id:** TST-047
- **area:** tests
- **severity:** Medium
- **location:** `crates/infra/core/src/lint.rs`
- **description:** The `lint` sub-module of `educore-core` is
  the workspace's contract-enforcement binary (per
  `docs/build-plan.md:1848-1900`). The module file is 308
  lines and contains 0 `#[test]` annotations in the source
  module and 0 integration test files in `crates/infra/core/tests/`.
  No test asserts the lint catches the documented anti-patterns
  (`unimplemented!()`, `todo!()`, `as` on numerics,
  `serde_json::Value` in domain code, `HashMap<String, T>`).
- **expected:** `docs/build-plan.md:1868-1897` — the lint
  sub-module is a gate; its own tests are the meta-gate.
- **evidence:**
  ```text
  $ grep -c "#\[test\]\|#\[tokio::test\]" crates/infra/core/src/lint.rs
  0
  $ find crates/infra/core/tests
  (no output)
  ```

### FINDING 48

- **id:** TST-048
- **area:** tests
- **severity:** Low
- **location:** workspace-wide (`docs/specs/*/tests.md` missing)
- **description:** AGENTS.md § Status and `docs/code-standards.md`
  describe an 11-file spec folder layout per domain. Zero
  `tests.md` files exist (TST-003), so there is no per-domain
  test catalogue that a contributor can use as a checklist.
  Cross-referencing `docs/specs/<d>/commands.md` against the
  crate's tests/ is manual.
- **expected:** `AGENTS.md:456`, `docs/code-standards.md:67-80`.
- **evidence:**
  ```text
  $ find docs/specs -name "tests.md"
  (no output)
  ```

### FINDING 49

- **id:** TST-049
- **area:** tests
- **severity:** Low
- **location:** `docs/coverage.toml:2298-2315` (sync rows)
- **description:** The coverage matrix lists `educore-sync`
  and `educore-sync-inprocess` rows whose `tests` paths point
  at `crates/cross-cutting/sync/src/lib.rs` and
  `crates/cross-cutting/sync-inprocess/src/lib.rs` — i.e., at
  source files, not test files. The lint sub-module verifies
  the path exists (it does), but does not verify any `#[test]`
  lives in those paths. The sync crates are in `coverage.toml`
  but absent from AGENTS.md § Crate Inventory (the inventory
  lists 34 crates; the workspace has 38).
- **expected:** `AGENTS.md:280-300` § Crate Inventory; the
  coverage row's `tests` field should point at a test file.
- **evidence:**
  ```text
  $ grep -E "tests = " docs/coverage.toml | grep -E "sync" 
  tests = "crates/cross-cutting/sync/src/lib.rs"
  tests = "crates/cross-cutting/sync-inprocess/src/lib.rs"
  ```

### FINDING 50

- **id:** TST-050
- **area:** tests
- **severity:** Low
- **location:** `crates/educore/src/lib.rs:88` (single test)
- **description:** The umbrella crate's single `mod tests`
  contains exactly one `#[test]` (smoke test of the
  re-exports). No test verifies that `educore::academic`,
  `educore::assessment`, etc. all compile and link; no test
  verifies that the umbrella's re-exports match the internal
  `educore-<name>` package names (a re-export regression would
  be silent).
- **expected:** `AGENTS.md:35` — "Consumers therefore write
  `educore::academic::commands::*`".
- **evidence:**
  ```text
  $ wc -l crates/educore/src/lib.rs
   94
  $ grep -c "#\[test\]\|#\[tokio::test\]" crates/educore/src/lib.rs
  1
  ```

### FINDING 51

- **id:** TST-051
- **area:** tests
- **severity:** Low
- **location:** `crates/domains/communication/src/services.rs`
- **description:** Communication declares proptest in
  `Cargo.toml` and uses it for some services, but the
  `NotificationService` (the cross-crate fan-out point for
  events) has no property test asserting idempotent delivery
  under duplicate `EventEnvelope` receipt.
- **expected:** `docs/ports/event-bus.md` — at-least-once
  delivery; consumer-side dedup invariants.
- **evidence:**
  ```text
  $ grep -B2 -A8 "proptest!" crates/domains/communication/src/services.rs | head -30
  ```

### FINDING 52

- **id:** TST-052
- **area:** tests
- **severity:** Low
- **location:** `crates/adapters/storage-surrealdb/`
- **description:** The SurrealDB adapter ships **12** unit
  tests (in `src/`) and **1** integration test
  (`tests/outbox_e2e.rs`). SurrealDB is the engine's "Phase 0
  primary target" per `docs/build-plan.md:43-44`, yet has the
  smallest test surface of the four adapter crates. No tests
  cover the SurrealDB-specific UUID coercion
  (`SurrealUuid` handling seen in
  `parity_event_log_filter.rs:146` `Err(e) if
  format!("{e:?}").contains("SurrealUuid")`) at the adapter
  level.
- **expected:** `docs/build-plan.md:43-44`; AGENTS.md
  Storage Adapters section names SurrealDB as a primary
  target.
- **evidence:**
  ```text
  $ grep -c "SurrealUuid\|surreal_uuid" crates/adapters/storage-surrealdb/src/*.rs
  0
  $ grep -rE "SurrealUuid" crates/tools/storage-parity/tests/parity_event_log_filter.rs
  Err(e) if format!("{e:?}").contains("SurrealUuid") => { ... }
  ```
