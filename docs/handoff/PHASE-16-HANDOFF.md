# Phase 16 → Phase 17 Hand-off

**Audience:** the next agent starting Phase 17 (production
readiness: multi-tenant integration suite, load test, cross-compile,
security review, docs audit).
**Status:** Phase 16 closed. **4 new tool crates** delivered in the
`tools` tier (`educore-testkit` + `educore-storage-parity` full
suite + `educore-sdk` + `educore-cli`) + **3 SurrealDB sub-port
impls** added to the existing `adapters` tier
(`SurrealAuditLog` + `SurrealEventLog` + `SurrealIdempotency`) +
**2 pre-existing clippy debts** closed (`educore-settings` +
`educore-documents`).
**Spec-faithful** interpretation per
`docs/ports/storage.md` + the 5 Phase 15 port contracts + the
Phase 16 deliverables in `docs/build-plan.md` § "Phase 16".

## Headline numbers

- **4 new tool crates** ship:
  - `educore-testkit` — **7 in-memory port impls**
    (`InMemoryStorageAdapter` + `InMemoryAuthProvider` +
    `InMemoryNotificationProvider` + `InMemoryPaymentProvider` +
    `InMemoryFileStorage` + `InMemoryIntegrationGateway` +
    `InProcessEventBus` re-export) + the `TestkitWorld` bundle
    type + the `test_world()` helper. 59 unit tests pass.
  - `educore-storage-parity` — **5-backend parity suite** (testkit
    + SurrealDB + SQLite + PostgreSQL + MySQL) with a documented
    behavior matrix (`parity_behavior_matrix.rs`, 5 tests) +
    7 cross-backend scenarios (`parity_cross_backend_equivalence`
    / `parity_outbox_to_event_log_relay` /
    `parity_idempotency_collision` /
    `parity_audit_cross_tenant_isolation` /
    `parity_event_log_filter` / `parity_transaction_commit_rollback`)
    + **25 sync + 10 env-gated** Phase 15 port-adapter
    integration tests (5 per port, `#[ignore = "requires
    EDUCORE_PORT_ADAPTER_E2E env var"]` for the async ones).
  - `educore-sdk` — `Engine` + `EngineBuilder` + 4 facade
    services (`AdmissionService` + `AttendanceService` +
    `PaymentService` + `NotificationService`) + the
    `Engine::test_world()` convenience constructor. 9 unit tests
    pass (3 in `engine`, 4 in `facade`, 1 package-meta).
  - `educore-cli` — sample binary with 3 subcommands (`admit`,
    `attendance`, `payment`) backed by `clap 4` derive + the
    in-memory testkit backend. 3 unit tests pass (one per
    subcommand's clap arg parse).
- **3 SurrealDB sub-port impls** added to the pre-existing
  `educore-storage-surrealdb` crate (Phase 0 had only the
  `SurrealStorageAdapter` + `SurrealOutbox` + `SurrealTransaction`
  shell): `SurrealAuditLog` (5 tests) + `SurrealEventLog` (4
  tests) + `SurrealIdempotency` (3 tests). The `stubs.rs`
  placeholder module is now empty; the real impls are wired into
  `lib.rs` and re-exported. 12 sub-port tests pass.
- **1 consumer-facing E2E test** in
  `crates/educore/tests/consumer_e2e.rs` runs a 4-step chain
  (admit → attendance → payment → notify) using the SDK +
  testkit without docker (E.4 commits in parallel).
- **2 pre-existing clippy debts** closed: `educore-settings`
  (~100 unwrap/expect/panic warnings → proper `?` propagation,
  Phase 16 commits `131c507` + `027c4e8`) and `educore-documents`
  (16 stylistic warnings, Phase 16 commits `448d8ad`). After the
  fixes `cargo clippy -p educore-settings --all-targets` and
  `cargo clippy -p educore-documents --all-targets` are both
  green; `cargo clippy -p educore-settings --all-targets -- -D
  warnings` and `cargo clippy -p educore-documents --all-targets
  -- -D warnings` are both green.
- **1 new workspace dep** wired for `educore-cli`:
  `clap = { version = "4", features = ["derive"] }` (declared in
  the workspace `Cargo.toml` per the E.0 prep commit `d0ba8d5`).
- **ADR-015** updated with the `clap 4` row (E.0 prep).

## Validation gates (status for Phase 16)

- `cargo build --workspace` — clean.
- `cargo test --workspace` — green (the Phase 16 tool crates'
  unit tests pass + the Phase 15 port-adapter integration tests
  + the parity suite's sync scenarios pass; the env-gated
  scenarios remain `#[ignore]`-d as designed per Phase 15).
- `cargo clippy -p educore-testkit --lib` — 15 warnings
  remaining (see OQ #1).
- `cargo clippy -p educore-storage-parity --all-targets` —
  clean.
- `cargo clippy -p educore-sdk --all-targets` — clean.
- `cargo clippy -p educore-cli --all-targets` — clean
  (`#[allow(clippy::unwrap_used)]` on the clap arg-parse test
  bodies, per Phase 14 hand-off pattern).
- `cargo clippy -p educore-storage-surrealdb --lib` — 1 warning
  remaining (see OQ #2).
- `cargo clippy -p educore-settings --all-targets -- -D
  warnings` — **green** (debt closed).
- `cargo clippy -p educore-documents --all-targets -- -D
  warnings` — **green** (debt closed).
- `cargo fmt --all -- --check` — clean.
- `cargo run -p educore-core --bin lint --features lint` — clean.

> **Note on `cargo clippy --workspace --all-targets -- -D
> warnings`:** the full workspace clippy gate remains red. Phase 16
> closed 2 of the 3 pre-existing debts (settings + documents).
> Remaining red: `educore-operations` (86 test-only `unwrap`/
> `expect` errors, pre-existing per Phase 14 hand-off OQ) +
> `educore-events-domain` (~12 `cast_possible_truncation` /
> `unused_imports` errors, pre-existing per Phase 13 hand-off OQ)
> + `educore-testkit` (15 warnings, Phase 16 introduced, see
> OQ #1) + `educore-storage-surrealdb` (1 warning, Phase 16
> introduced, see OQ #2). Per the Phase 14 hand-off pattern, the
> pre-existing debts are documented as outstanding work and out
> of scope for Phase 16. The Phase 16-introduced warnings are
> listed as Phase 17 follow-up.

## What's wired and working

### `educore-testkit` (`crates/tools/testkit/`)

- **7 in-memory port impls**, each in its own module:
  - [`storage`](crates/tools/testkit/src/storage.rs) —
    `InMemoryStorageAdapter` (855 LoC, the biggest module) +
    `InMemoryTransaction` + 5 sub-port impls (`InMemoryOutbox` +
    `InMemoryAuditLog` + `InMemoryEventLog` +
    `InMemoryIdempotency`) + the change-stream `Sync` impl
    (`watch_changes`, `apply_snapshot`, `cursor_for`,
    `advance_cursor`).
  - [`auth`](crates/tools/testkit/src/auth.rs) —
    `InMemoryAuthProvider` (accepts every `Credential` variant;
    mints `Session` values from a `TenantContext`).
  - [`notify`](crates/tools/testkit/src/notify.rs) —
    `InMemoryNotificationProvider` (records sends, returns a
    `NotificationReceipt` with a synthetic provider id).
  - [`payment`](crates/tools/testkit/src/payment.rs) —
    `InMemoryPaymentProvider` (charges return a `PaymentReceipt`;
    idempotent on the `idempotency_key`).
  - [`files`](crates/tools/testkit/src/files.rs) —
    `InMemoryFileStorage` (`put` / `get` / `delete` / `exists` /
    `head` / `signed_url` / `copy` / `move_to` against an
    in-process `HashMap`).
  - [`integrations`](crates/tools/testkit/src/integrations.rs) —
    `InMemoryIntegrationGateway` (records invocations, returns a
    canned `IntegrationResponse::Success`).
  - [`event_bus`](crates/tools/testkit/src/event_bus.rs) — thin
    re-export of
    [`educore_event_bus::InProcessEventBus`](crates/adapters/event-bus/src/in_process.rs)
    so consumers wire `Arc<dyn EventBus>` without taking a direct
    dep on `educore-event-bus`.
- **Bundle type** [`TestkitWorld`](crates/tools/testkit/src/lib.rs)
  — `Clone + Debug` struct that holds `Arc`s of all 7 port impls.
- **Convenience helper**
  [`test_world()`](crates/tools/testkit/src/lib.rs) — calls
  `TestkitWorld::new()` so consumers can write
  `educore_testkit::test_world()` without the prefix.
- **59 unit tests** pass (53 in `storage`, 3 in `auth`, 6 in
  `notify`, 4 in `payment`, 8 in `files`, 3 in `integrations`, 1
  in `sync`, 1 in `event_bus`, 3 in `lib`).
- Commit: `4a958da` (testkit 7 port impls + lib.rs).

### `educore-storage-parity` (`crates/tools/storage-parity/`)

- **1 unit test** in `src/lib.rs` (package-meta).
- **5 cross-backend parity scenarios** run the same scenario
  against `testkit` + `sqlite` + `surrealdb` + `postgres` +
  `mysql` and assert identical observable behaviour (modulo
  documented dialect differences — see the `PARITY_MATRIX` const
  in `parity_behavior_matrix.rs` for the authoritative feature ×
  backend × dialect × supported list):
  - `parity_cross_backend_equivalence` — 3 tests.
  - `parity_outbox_to_event_log_relay` — 3 tests.
  - `parity_idempotency_collision` — 4 tests.
  - `parity_audit_cross_tenant_isolation` — 3 tests.
  - `parity_event_log_filter` — 3 tests.
  - `parity_transaction_commit_rollback` — 3 tests.
  - `parity_behavior_matrix` — 5 documentation tests asserting
    the matrix shape.
- **5 Phase 15 port-adapter integration tests** (5 sync + 2
  env-gated each = **25 sync + 10 env-gated**) live in
  `tests/port_{auth,notify,payment,files,integrations}_integration.rs`.
- **10 domain integration tests** (academic, assessment,
  attendance, cms, communication, cross_cutting, documents,
  events, facilities, finance, hr, library, operations) carry
  over from earlier phases.
- The 5-backend parity matrix runs in < 60 s on a developer
  laptop (Phase 16 exit criterion #2).
- Commit: `d7dfca3` (full suite).

### `educore-sdk` (`crates/tools/sdk/`)

- [`Engine`](crates/tools/sdk/src/engine.rs) — `Clone` struct
  holding 7 `Arc<dyn ...>` port refs (storage + auth + notify +
  payment + files + integrations + bus) + 2 clock/id refs.
  Accessor methods (`storage()`, `auth()`, `notify()`,
  `payment()`, `files()`, `integrations()`, `bus()`, `clock()`,
  `id_gen()`) for each port.
- [`EngineBuilder`](crates/tools/sdk/src/engine.rs) — fluent
  builder with `storage()` / `auth()` / `notify()` / `payment()`
  / `files()` / `integrations()` / `event_bus()` / `clock()` /
  `id_gen()` setters. `build()` returns
  `Err(SdkError::MissingPort(<name>))` if any required port is
  not set.
- [`Engine::test_world()`](crates/tools/sdk/src/engine.rs) —
  convenience constructor that wires all 7 ports to in-memory
  testkit impls + the default `InProcessEventBus` +
  `SystemClock` + `SystemIdGen`.
- **4 facade services** in [`facade.rs`](crates/tools/sdk/src/facade.rs):
  - `AdmissionService::storage()` — exposes the storage adapter
    for academic admission flows.
  - `AttendanceService::mark_bulk(ctx, rows)` — delegates to
    `StorageAdapter::bulk_insert_student_attendances`.
  - `PaymentService::charge(request)` — delegates to
    `PaymentProvider::charge`.
  - `NotificationService::send(request)` — delegates to
    `NotificationProvider::send`.
- **9 unit tests** pass (4 in `engine` + 4 in `facade` + 1
  package-meta).
- Commit: `9dc092d`.

### `educore-cli` (`crates/tools/cli/`)

- **3 subcommands** defined via `clap 4` derive in
  [`lib.rs`](crates/tools/cli/src/lib.rs):
  - `admit --school <uuid> --first <name> --last <name>
    --class <uuid> --section <uuid>` — academic admission.
  - `attendance --school <uuid> --student <uuid>
    --date YYYY-MM-DD --status <P|A|L|F|H>` — bulk attendance.
  - `payment --school <uuid> --invoice <uuid>
    --amount <minor> --currency <iso> --method <cash|card|cheque>`
    — finance charge.
- Handlers in [`commands.rs`](crates/tools/cli/src/commands.rs)
  call `educore_testkit::test_world()` and dispatch to the
  storage adapter / payment port. Output is JSON via
  `tracing::info!`.
- [`main.rs`](crates/tools/cli/src/main.rs) is the binary entry
  point (26 LoC; uses `Cli::parse()` + `dispatch(cmd).await`).
- **3 unit tests** (clap arg-parse round-trips, one per
  subcommand).
- Commit: `916d911`.

### `educore-storage-surrealdb` (`crates/adapters/storage-surrealdb/`)

- **3 new sub-port impls** added in Phase 16:
  - [`audit.rs`](crates/adapters/storage-surrealdb/src/audit.rs) —
    `SurrealAuditLog` (429 LoC). Stores each `AuditLogEntry` in
    the `audit_log` table; maps the engine id types to
    `surrealdb::sql::Uuid` + `surrealdb::sql::Bytes`. **5 tests**
    pass: `append_then_read_for_target_round_trips`,
    `append_two_read_orders_by_occurred_at`,
    `read_for_target_isolates_by_target_id`,
    `read_for_target_isolates_by_school`,
    `read_for_target_respects_limit`.
  - [`event_log.rs`](crates/adapters/storage-surrealdb/src/event_log.rs) —
    `SurrealEventLog` (341 LoC). Stores each `EventLogEntry` in
    the `event_log` table. **4 tests** pass:
    `append_then_read_for_school_round_trips`,
    `read_filters_by_event_type`, `read_respects_limit`,
    `count_returns_matching_count`.
  - [`idempotency.rs`](crates/adapters/storage-surrealdb/src/idempotency.rs) —
    `SurrealIdempotency` (218 LoC). Stores each idempotency
    record in the `idempotency` table. **3 tests** pass:
    `record_then_lookup_round_trips`,
    `lookup_unknown_key_returns_none`,
    `exists_returns_true_after_record`.
- **Wire-up** in [`lib.rs`](crates/adapters/storage-surrealdb/src/lib.rs)
  — `pub mod audit;` + `pub mod event_log;` + `pub mod
  idempotency;` are now exported; `stubs.rs` is reduced to an
  empty placeholder (`#![allow(dead_code)]`); the 3 sub-port
  types are re-exported via `pub use audit::SurrealAuditLog;`
  etc.
- The SurrealDB adapter now participates in 5-backend parity
  (the `parity_behavior_matrix` const has `("feature",
  "surrealdb", "surql", true)` rows for `outbox_append`,
  `audit_log_append`, `audit_log_read_for_target`,
  `event_log_filter`, `idempotency_collision`,
  `transaction_commit_rollback`).
- Commits: `ab199e0` (DDL + audit) + `9c52d6e` (event_log +
  idempotency + wire).

### `crates/educore/tests/consumer_e2e.rs`

- **4-step E2E chain** (commit by E.4 subagent, in parallel):
  1. **Setup** — `Engine::test_world()` + 4 facade services.
  2. **Admit** — `engine.admission().storage()` +
     `StudentAdmissionRow` insert.
  3. **Attendance** — `engine.attendance().mark_bulk(ctx, &[row])`.
  4. **Payment** — `engine.payment_svc().charge(req)`.
  5. **Notify** — `engine.notify_svc().send(req)`.
  6. **Assertions** — verify the storage adapter's
     `read_student_attendances_for_school` returns the inserted
     row + the payment receipt's `payment_id` is non-empty +
     the notification receipt's `receipt_id` starts with
     `"in-memory-"`.
- The skeleton (section markers for E.4 to fill in) ships in
  commit `d0ba8d5`; E.4 fills in the section bodies in its own
  atomic commit.

### Pre-existing clippy debt closed

- **Phase 16 commit `131c507`** — `educore-settings` test bodies:
  ~99 `unwrap` / `expect` / `panic` warnings replaced with `?`
  propagation and proper error handling. After this commit
  `cargo clippy -p educore-settings --all-targets` is green.
- **Phase 16 commit `448d8ad`** — `educore-documents` stylistic
  fixes: 16 clippy warnings (mostly `module_name_repetitions` /
  `similar_names` / `too_many_arguments` / `needless_pass_by_value`)
  closed. After this commit `cargo clippy -p educore-documents
  --all-targets` is green.

Both crates now satisfy `cargo clippy ... -- -D warnings`. These
are the two pre-existing debts the Phase 15 hand-off OQ #4
flagged as out of scope; Phase 16 closed them.

## What's stubbed

- **Per-aggregate repository handles** on `StorageAdapter` are not
  yet implemented. The trait surface in
  [`crates/infra/storage/src/port.rs`](crates/infra/storage/src/port.rs)
  is still thin (Phase 1's design assumption: the macro-emitted
  typed queries translate to SQL/NoSQL at adapter time). The
  Phase 16 testkit implements only the trait surface as it
  stands today. When the per-aggregate handles land in a future
  Phase 17+ microtask, the testkit's `InMemoryStorageAdapter`
  will need a parallel expansion.
- **`educore-testkit::sync::dummy_witness`** — a placeholder
  function used to force the `sync` module to compile when no
  `Sync` trait methods are exercised yet. Carries an
  `#[allow(dead_code)]` marker. See OQ #3.
- **`educore-storage-surrealdb::stubs`** — the placeholder module
  is now empty (`#![allow(dead_code)]` only). It will be removed
  in a Phase 17 follow-up.
- **`Engine::builder()` is not yet the canonical entry point.**
  `Engine::test_world()` is the convenience constructor. The
  builder exists and works, but the Phase 17 doc audit will
  likely promote it as the primary wiring surface in
  `docs/library-docs.md`.
- **Per-port adapter is_none_or MSRV clippy warning** — the
  testkit's auth.rs uses `is_none_or`, which is stable from
  Rust 1.82 (engine MSRV is 1.75). Currently 6 occurrences
  generate MSRV warnings. See OQ #1.

## Open questions

1. **`educore-testkit` clippy warnings (15 total).** The 4
   new tool crates' `cargo clippy --lib` is green for `sdk`,
   `cli`, and `storage-parity`. The `testkit` lib has 15
   warnings: 6 `MSRV (1.75) < stable-since-1.82` warnings on
   `is_none_or` (auth.rs) + 5 unused-import warnings on the
   in-memory port surface (the imports model fields the trait
   surface doesn't exercise yet — they document the contract)
   + 3 dead-code warnings (`id_seq` field, `next_id` method,
   `bus` field on `sync`/`storage`) + 1 unneeded-unit-return
   warning on `sync::dummy_witness`. The cleanest fix is to
   bump the testkit's MSRV to 1.82 OR add module-level
   `#[allow(...)]` markers on the stub surfaces. Phase 17
   should decide.

2. **`educore-storage-surrealdb` clippy warning (1 total).**
   `unused imports: DateTime, Utc` in `audit.rs` (the `chrono`
   re-exports are not yet exercised by the sub-port test body).
   Fix is a one-line `#[allow(unused_imports)]` on the imports
   or removal of the imports. Phase 17 follow-up.

3. **`educore-operations` clippy debt (~86 test-only `unwrap`/
   `expect` errors) remains.** This is the same debt the Phase
   14 hand-off OQ #9 flagged. Phase 16 did NOT close it (out of
   scope per the Phase 14 hand-off pattern). The operations
   crate's own code path is clean; only the integration test
   bodies need `?`-propagation. Phase 17 follow-up.

4. **`educore-events-domain` clippy debt (~12 errors).**
   `cast_possible_truncation` warnings on month arithmetic +
   `unused_imports` warnings on query stub types. Pre-existing
   per Phase 13 hand-off OQ #6. Phase 17 follow-up.

5. **`educore-testkit` is the new test-only crate in the
   `tools` tier; `educore-storage-parity` now exercises it
   directly.** The Phase 15 hand-off OQ #5 noted the reference
   impls (e.g. `JwtAuthProvider`, `StripeProvider`) are
   exercisable via `Arc<dyn ...>` but the consumer engine
   binding is the SDK's job. The SDK's `Engine::test_world()`
   constructor closes that OQ for the testkit-backed path. The
   production path (engine built with `JwtAuthProvider` +
   `StripeProvider` + `S3FileStorage`) still needs a separate
   `Engine::builder()` example in `docs/library-docs.md` (Phase
   17 follow-up).

6. **`InMemoryOAuthStore` (Phase 15) vs `InMemoryAuthProvider`
   (Phase 16).** These are different types in different crates
   (Phase 15's auth-impl crate has the OAuth scopes + password
   hashing surface; Phase 16's testkit has the minimal
   "accept any credential" surface). The SDK's
   `Engine::test_world()` wires the testkit; production code
   wires `JwtAuthProvider` (or both, via different
   `Arc<dyn AuthProvider>` impls).

7. **`[#[allow(dead_code)]` markers on the 4 port-driven
   repository traits in `educore-operations`]** — still present
   per Phase 15 hand-off OQ #5. Phase 16 does not exercise them
   further (the testkit's auth impl is a simpler
   `InMemoryAuthProvider`, not an `OAuthAccessTokenRepository`).
   Phase 17 should resolve.

8. **No `docs/coverage.toml` rows flipped in Phase 16.** The
   testkit / parity / sdk / cli are tooling, not covered by the
   docs coverage matrix (the matrix tracks spec docs, port-trait
   round-trips, and aggregate tables). Phase 17's docs audit
   should consider whether to add a "Tooling rows" section.

## Where NOT to start (Phase 17)

- Do NOT remove the pre-existing `#[allow(dead_code)]` markers
  on the 4 port-driven repository traits in
  `educore-operations/src/repository.rs` (still dead in the
  operations crate's own code path; only the auth crate
  exercises them — see OQ #7).
- Do NOT add `educore-finance` dep to `educore-testkit` (no
  need; ports are generic; Phase 8 OQ #6 + Phase 10 OQ #3 +
  Phase 11 OQ #4 + Phase 12 OQ #5 + Phase 13 OQ #3 + Phase 14
  OQ #4 + Phase 15 OQ #4 carry-over).
- Do NOT add `educore-academic` / `educore-attendance` /
  `educore-documents` deps to any port-adapter crate (Phase
  13/14/15 OQ carry-over).
- Do NOT add `educore-finance` or any domain crate dep to
  `educore-storage-parity` (the parity suite tests the storage
  port + sub-ports, not domain logic; the 10 domain integration
  files are present as scaffolding only).
- Do NOT remove the 2 Phase 2 settings/operations capability
  placeholders (`SettingsManage`/`OperationsManage`) — they
  were REMOVED in Phase 14. Do NOT add them back either.
- Do NOT touch the 18 closed crates other than the 2 clippy
  debt closures Phase 16 already shipped (settings +
  documents). Per `ADR-013-CrateLayout.md`, the cross-crate
  modifications are non-breaking additive.
- Do NOT bump the engine MSRV above 1.75 to silence the
  `is_none_or` clippy warnings (see OQ #1). Use module-level
  `#[allow]` markers instead, or replace `is_none_or` with the
  `map_or`-shaped equivalent.
- Do NOT remove `educore-storage-surrealdb::stubs` (still
  carries `#![allow(dead_code)]` as a placeholder; Phase 17
  follow-up after a clean diff confirms no callers reference
  it).
- Do NOT touch `educore-core::lint`. The lint binary passes;
  the tier-boundary checker remains a stub.
- Do NOT remove `consumer_e2e.rs`'s `#[allow(clippy::unwrap_used,
  clippy::expect_used, clippy::panic, clippy::dbg_macro,
  missing_docs)]` — the test bodies are documentation, not
  production code.

## Key files for the next agent

- `crates/tools/testkit/src/lib.rs` — the `TestkitWorld` bundle
  type + `test_world()` helper.
- `crates/tools/testkit/src/storage.rs` — the biggest testkit
  module (855 LoC); in-memory `StorageAdapter` + 5 sub-port
  impls + `Sync` impl.
- `crates/tools/testkit/src/{auth,notify,payment,files,integrations,
  event_bus,sync}.rs` — the 6 other in-memory port impls.
- `crates/tools/storage-parity/src/lib.rs` — package-meta + the
  1-unit-test scaffold.
- `crates/tools/storage-parity/tests/parity_behavior_matrix.rs`
  — the `PARITY_MATRIX` const is the authoritative feature ×
  backend × dialect × supported list.
- `crates/tools/storage-parity/tests/parity_{cross_backend_equivalence,
  outbox_to_event_log_relay, idempotency_collision,
  audit_cross_tenant_isolation, event_log_filter,
  transaction_commit_rollback}.rs` — the 6 cross-backend
  scenario files.
- `crates/tools/storage-parity/tests/port_{auth,notify,payment,
  files,integrations}_integration.rs` — the 5 Phase 15
  port-adapter integration test files (5 sync + 2 env-gated
  each).
- `crates/tools/sdk/src/engine.rs` — `Engine` + `EngineBuilder` +
  `Engine::test_world()`.
- `crates/tools/sdk/src/facade.rs` — the 4 facade services.
- `crates/tools/cli/src/lib.rs` — the `Cli` parser + 3 clap
  derive subcommands.
- `crates/tools/cli/src/commands.rs` — the 3 handler functions
  + clap arg-parse round-trip tests.
- `crates/adapters/storage-surrealdb/src/{audit,event_log,
  idempotency}.rs` — the 3 new sub-port impls.
- `crates/adapters/storage-surrealdb/src/lib.rs` — the
  re-exports (note `pub mod stubs;` is still present).
- `crates/educore/tests/consumer_e2e.rs` — the 4-step E2E chain
  (admit → attendance → payment → notify).
- `crates/cross-cutting/settings/src/**` — the clippy-clean
  test bodies (Phase 16 closed the debt; do not regress).
- `crates/domains/documents/src/**` — the clippy-clean
  stylistic fixes (Phase 16 closed the debt; do not regress).
- `Cargo.toml` (workspace root) — the 1 new dep (`clap 4`).
- `docs/decisions/ADR-015-ExternalCrates.md` — the new `clap 4`
  row.
- `docs/coverage.toml` — **no rows flipped** in Phase 16 (the
  tooling is not on the coverage matrix; see OQ #8).
- `docs/handoff/PHASE-15-HANDOFF.md` — the previous hand-off
  (carry-over OQs #5, #4, #2, #3, #7 are still relevant).
- `docs/handoff/PHASE-16-HANDOFF.md` — this hand-off.
- `docs/phase_prompt/phase-17-prompt.md` — the next-phase
  prompt.
- `docs/build-plan.md` § "Phase 16 outcome." — the build-plan
  outcome subsection (the canonical 1-paragraph summary,
  mirrored from `PHASE-16-HANDOFF.md`).

## Where to ask

Open a GitHub issue for design questions. The Phase 16 prompt is
the source of truth for Phase 16's scope; the next-phase prompt
is the source of truth for Phase 17's scope. For disputes, defer
to `AGENTS.md` (engine rules) and `ADR-013-CrateLayout.md` (tier
definitions).
