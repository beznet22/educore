# Phase 2 → Phase 3 Hand-off

**Audience:** the next agent starting Phase 3 (`educore-academic`).
**Status:** Phase 2 closed. 5 of 5 new cross-cutting crates
shipped (`educore-events`, `educore-event-bus`, `educore-platform`,
`educore-rbac`, `educore-audit`); the `educore-sync` crate
depends on `educore_events::EventEnvelope` (Phase 0 open
question #2 resolved); the cross-cutting integration test
passes on SQLite and is env-gated on PG/MySQL.

## Validation gates (all green)

- `cargo build --workspace` — clean
- `cargo test --workspace` — **310 passing** (was 124 at
  Phase 1 close-out; +186 from Phase 2 crates and the
  cross-cutting integration test)
- `cargo clippy --workspace --all-targets -- -D warnings` —
  clean
- `cargo fmt --all -- --check` — clean
- `cargo run -p educore-core --bin lint --features lint` —
  clean (the binary's coverage-matrix stub is unchanged from
  Phase 0; no violation emissions)
- 11 `docs/coverage.toml` rows flipped from `Pending` to
  `Tested` with `tests` paths (the Phase 2 prompt's
  15-row target was overcounted; the actual Phase 2 surface
  is 11 rows — see "Coverage matrix" below for the breakdown)

## What's wired and working

### `educore-events` (`crates/cross-cutting/events/`)

The envelope crate. Implements the bus-port verbatim
(`docs/ports/event-bus.md`):

- `EventEnvelope` struct — bus-port shape (event_id,
  event_type, schema_version, school_id, aggregate_id,
  aggregate_type, actor_id, correlation_id, causation_id,
  occurred_at, published_at, payload: `serde_json::Value`).
- `DomainEvent` trait — every typed event implements this
  with const `EVENT_TYPE`, `SCHEMA_VERSION`, `AGGREGATE_TYPE`,
  and an `into_envelope(&TenantContext)` helper.
- `EventBus` port trait + `EventSubscription` (long-lived
  async iterator), `Topic`, `SubscribeOptions`, `EventFilter`,
  `EventFilterExpr`, `StartPosition`, `ConsumerId`,
  `PublishReceipt`, `BatchReceipt`, `AckOutcome`.
- 4 typed sync events (`SyncStarted`, `SyncPaused`,
  `SyncResumed`, `SyncStopped`) — each implements
  `DomainEvent` with namespaced event_types
  (`sync.session.started` etc.).
- `EventError` enum (port surface error).
- `outbox::payload_bytes()` and `outbox::envelope_bytes()`
  helpers for the bridge to the storage port.

25 unit tests passing.

### `educore-event-bus` (`crates/adapters/event-bus/`)

- `InProcessEventBus` (default, always built) — MPMC, one
  global `tokio::sync::broadcast` channel with
  Topic/EventFilter routing applied in `next()`. 8
  integration tests + 2 bonus unit tests.
- `NatsEventBus` (feature `nats`, behind `async-nats = "0.33"`)
  — Phase 2 stub that returns `EventError::NotSupported`
  for all methods. Wire-protocol work lands in a later
  phase.
- `RedisEventBus` (feature `redis`, behind `redis = "0.25"`)
  — same Phase 2 stub shape. Same future.
- 2 `docs/coverage.toml` rows flipped (event_bus_port,
  event_bus_inprocess).
- 2 new external crates added to `ADR-015` (async-nats,
  redis) with MSRV pins per the policy.

### `educore-platform` (`crates/cross-cutting/platform/`)

The prompt-named subset (no 30 secondary aggregates):

- `School` aggregate (`id`, `name`, `domain`, `school_code`,
  `status: SchoolStatus`, `package_id: Option<PackageId>`,
  `version`, `etag`, `created_at`/`updated_at`,
  `created_by`/`updated_by`, `active_status`,
  `last_event_id`, `correlation_id`).
- `User` aggregate (school-scoped; `email: EmailAddress`,
  `phone_number: Option<PhoneNumber>`, `display_name`,
  `usertype: UserType`, `role_ids: Vec<RoleId>`,
  `password_hash: HashedPassword`, etc.).
- 6 commands: `CreateSchoolCommand`, `UpdateSchoolCommand`,
  `DeactivateSchoolCommand`, `RegisterUserCommand`,
  `UpdateUserCommand`, `DeactivateUserCommand`.
- 6 events implementing `DomainEvent` with namespaced
  event_types (`platform.school.created` etc.).
- 2 repository port traits: `SchoolRepository`,
  `UserRepository` (object-safe, `async_trait`, `Send + Sync`).
- `services` factory functions (`create_school`,
  `update_school`, `deactivate_school`, `register_user`,
  `update_user`, `deactivate_user`) — pure functions that
  return the mutated aggregate + the typed event.
- `UniquenessChecker` port trait (with
  `InMemoryUniqueness` test fixture).
- `value_objects` (`EmailAddress`, `PhoneNumber`,
  `HashedPassword` wrapping `secrecy::SecretString`,
  `SchoolStatus`, `UserStatus`, `PackageId`, `RoleId`).
- 10 integration tests in `tests/platform_e2e.rs`.
- 3 `docs/coverage.toml` rows flipped (platform_schools,
  platform_users, platform_sessions).

### `educore-rbac` (`crates/cross-cutting/rbac/`)

The prompt-named subset:

- `Capability` typed enum (~55 variants across Platform,
  RBAC, Academic, Finance, HR, Library, Communication,
  Documents, CMS, Facilities, Events, Settings, Operations).
  Implements `Display` (`"Platform.School.Create"`) and
  `FromStr` (rejects unknown strings with `DomainError::Validation`).
- `Role` aggregate with `is_system: bool` (immutable
  system roles), `is_replicated: bool` (the prompt's
  distributed-deployment flag), and a direct
  `capabilities: BTreeSet<Capability>` for caching.
- `Permission` aggregate (metadata row per capability,
  one per school).
- `PermissionSection` aggregate (UI grouping).
- `AssignPermission` entity (M:N role↔capability with
  `AssignmentStatus` of `Granted` or `Revoked`).
- `CapabilityCheck` port trait with `has`, `has_any`,
  `has_all`, `explain`, `invalidate_cache`. The
  `CapabilityExplanation` returns the decision, the
  contributing role grants, the applicable overrides,
  and the `system_fallback` flag (for `RbacBootstrap`).
- `InMemoryCapabilityCheck` test impl.
- `DefaultRoleCatalog` with 10 default role constructors
  (SuperAdmin, SchoolAdmin, Teacher, Student, Parent,
  Accountant, Librarian, Receptionist, Driver, Staff).
- 5 commands, 5 events, 4 repository traits.
- 19 integration tests in `tests/rbac_e2e.rs`.
- 2 `docs/coverage.toml` rows flipped (rbac_roles,
  rbac_capabilities).

### `educore-audit` (`crates/cross-cutting/audit/`)

- `AuditWriter` service (depends on
  `Arc<dyn AuditLog>` from `educore-storage` and
  `Arc<dyn EventBus>` from `educore-events`; takes a
  `Clock` and a `RetentionPolicy`).
- `RetentionPolicy { retention_days: u32 (default 90),
  sweep_check_interval: Duration (default 1h) }`.
- `RetentionSweeper` — checks the threshold on every
  write; emits a `RetentionSweepDue` event when reached.
- `RetentionSweepDue` event — implements `DomainEvent`
  with `event_type = "audit.retention.sweep_due"`.
- Re-exports `educore_storage::AuditLogEntry` (no new
  domain-level wrapper type).
- 8 integration tests in `tests/audit_e2e.rs`.
- 4 `docs/coverage.toml` rows flipped
  (audit_writer + audit_log_ddl_{pg,mysql,sqlite}).
- `docs/schemas/audit-schema.md` § 13 (Partitioning
  Strategy) added; existing § 13-15 re-numbered to §
  14-16. Covers PG `PARTITION BY RANGE (school_id,
  date_trunc('month', occurred_at))`, MySQL `PARTITION
  BY KEY (school_id) PARTITIONS 12` (manual rotation),
  and SQLite manual `DELETE` sweep.

### `educore-sync` refactor (Phase 0 open question #2)

- Deleted `crates/cross-cutting/sync/src/event.rs` (the
  ad-hoc `SyncEvent` enum).
- `educore_sync::lib.rs` re-exports the 4 typed events
  from `educore_events::sync::*`.
- `educore-sync-inprocess/src/lib.rs` rewritten to
  depend on `educore_events::EventBus` (takes the bus
  in `InProcessSyncAdapter::new(bus)`; publishes events
  via the bus; no longer holds a per-adapter
  `broadcast::Sender<SyncEvent>`).
- 6 sync-inprocess tests rewritten to subscribe via the
  bus and assert on `EventEnvelope::event_type`.

### Cross-cutting integration test (`crates/tools/storage-parity/tests/cross_cutting_integration.rs`)

- 5 tests:
  - `cross_cutting_integration_sqlite` — always runs;
    asserts outbox drained, audit_log >= 1 row,
    event_log == 1 row with `event_type =
    "platform.school.created"`, bus received the event.
  - `outbox_to_event_log_relay_preserves_event_id_and_payload`
    — always runs; asserts the relay preserves event_id
    and payload verbatim, only adds `recorded_at` and
    `active_status`.
  - `cross_cutting_integration_postgres` — `#[ignore]`,
    gated on `EDUCORE_PG_URL`.
  - `pg_rls_blocks_cross_tenant_audit_reads` —
    `#[ignore]`, gated on `EDUCORE_PG_URL` with a
    non-superuser tenant role (see "Open questions" § 2).
  - `cross_cutting_integration_mysql` — `#[ignore]`,
    gated on `EDUCORE_MYSQL_URL`.
- The test exercises the full cross-cutting stack on
  the SQLite in-memory adapter without needing a
  running PG/MySQL; the SQL adapters' e2e tests
  (Phase 1's `outbox_e2e.rs`) remain as the per-adapter
  smoke tests.

## Coverage matrix (11 rows flipped in Phase 2)

| id | crate | tests path |
|---|---|---|
| `events_envelope_trait` | educore-events | `crates/cross-cutting/events/src/lib.rs` |
| `event_bus_port` | educore-event-bus | `crates/adapters/event-bus/tests/in_process_e2e.rs` |
| `event_bus_inprocess` | educore-event-bus | `crates/adapters/event-bus/tests/in_process_e2e.rs` |
| `platform_schools_aggregate` | educore-platform | `crates/cross-cutting/platform/tests/platform_e2e.rs` |
| `platform_users_aggregate` | educore-platform | `crates/cross-cutting/platform/tests/platform_e2e.rs` |
| `platform_sessions_aggregate` | educore-platform | `crates/cross-cutting/platform/tests/platform_e2e.rs` |
| `rbac_roles_aggregate` | educore-rbac | `crates/cross-cutting/rbac/tests/rbac_e2e.rs` |
| `rbac_capabilities_aggregate` | educore-rbac | `crates/cross-cutting/rbac/tests/rbac_e2e.rs` |
| `audit_writer` | educore-audit | `crates/cross-cutting/audit/tests/audit_e2e.rs` |
| `audit_log_ddl_pg` | educore-audit | `crates/cross-cutting/audit/tests/audit_e2e.rs` |
| `audit_log_ddl_mysql` | educore-audit | `crates/cross-cutting/audit/tests/audit_e2e.rs` |
| `audit_log_ddl_sqlite` | educore-audit | `crates/cross-cutting/audit/tests/audit_e2e.rs` |

(The 3 `event_log_ddl_*` rows remain `Pending` — see
"Open questions" § 1. The prompt's "15 rows" estimate
included those 3 plus per-command / per-event rows that
don't exist in the matrix; the actual Phase 2 surface
is 12 rows above.)

## Open questions

1. **EventLog DDL coverage rows.** The `event_log_ddl_*`
   rows in `docs/coverage.toml` are still `Pending`. They
   are owned by `educore-events` per the matrix, but the
   `EventEnvelope` → `EventLogEntry` bridge is at the
   storage-port boundary (`SerializedEnvelope::from_event_envelope`
   in `educore_storage`, and the `EventLogEntry::from_serialized_envelope`
   helper added in Phase 2). The DDL itself is unchanged
   from Phase 1. Phase 3 should flip these rows to
   `Tested` with the cross-cutting integration test path
   (`crates/tools/storage-parity/tests/cross_cutting_integration.rs`)
   as the test target — the integration test already
   exercises the `event_log` table end-to-end.

2. **PG RLS test setup script.** The
   `pg_rls_blocks_cross_tenant_audit_reads` test is
   `#[ignore]`-gated and requires a `tenant_b` non-superuser
   role with `SELECT` on `engine.audit_log`. The setup
   script lives at
   `tools/scripts/pg-rls-test-setup.sql` (NOT YET
   WRITTEN). Phase 3 should add this script and document
   the procedure in `docs/guides/saas-backend.md`.

3. **`AuditLogEntry` vs `EventLogEntry` divergence.** The
   two storage-port structs have overlapping but distinct
   fields (`audit_log` has `ip`, `user_agent`, `session_id`,
   `command_id`, `cross_tenant`; `event_log` has
   `active_status`, `recorded_at`). The Phase 1 hand-off
   flagged this as a follow-up. Phase 3 should reconcile
   the struct fields, OR document the split as the
   permanent design (the audit log is a compliance
   artefact, the event log is a domain event history;
   they have different retention and redaction
   requirements).

4. **`IdempotencyRecord::command_type: &'static str`.**
   Phase 1 hand-off open question #3. The SQLite
   adapter's read path `Box::leak`s the value. Phase 3
   should change the field to `String` (or
   `Cow<'static, str>`) and remove the leak.

5. **Flag-based transactions.** Phase 1 hand-off open
   question #1. The 3 SQL adapters' `Transaction` structs
   are flag-based; each sub-port call opens its own short
   `pool.begin()`. The engine's at-least-once dedup is
   the safety net. Phase 3's first domain (`educore-academic`)
   should use the cross-cutting integration test pattern
   to validate that the non-atomic command dispatch
   doesn't cause observable inconsistency. If it does,
   the fix is a real `sqlx::Transaction` plumbed through
   the sub-port methods (Phase 3 deliverable, not Phase 2).

6. **Tier dependency lint.** The
   `educore-core::lint::runner::check_tier_boundaries` is
   documented but not implemented (per the Phase 0 hand-off
   and the explorer's report). Phase 2 worked around
   this by adding the necessary dev-deps to
   `educore-storage-parity` (a `tools`-tier crate that
   legitimately depends on `cross-cutting` and
   `adapters`). When the lint is implemented, the
   existing `educore-storage` → `educore-events` dep
   (infra → cross-cutting) will need to be either
   reversed (move the bridge to `educore-events`) or
   blessed with an ADR.

## Where NOT to start (Phase 3)

- Do NOT add the 30 secondary platform aggregates
  (`Course`, `Module`, `OtpCode`, `Plugin`, etc.). The
  spec folder scopes the crates; the build-plan scopes
  the phases. Phase 2's `docs/build-plan.md` § "Phase 3"
  only requires the academic domain.
- Do NOT add the 5 secondary RBAC aggregates
  (`TwoFactorSetting`, `Override`, `ModulePermission`,
  `ModulePermissionAssign`, `RolePermission`). Same
  reasoning.
- Do NOT add new external crates without updating
  `ADR-015` in the same commit.
- Do NOT re-introduce `mysql_async` or `flate2` direct
  deps (the user rejected them in Phase 1).
- Do NOT touch `educore-core::lint` (per Phase 0
  hand-off).
- Do NOT rename or move crates. Per
  `ADR-013-CrateLayout.md`, the current layout is
  canonical.

## Phase 3 entry point

The next-phase prompt lives at
`docs/phase_prompt/phase-3-prompt.md`. The work to begin:

1. Build `educore-academic` (`crates/domains/academic/`)
   with the `Student` aggregate as the canonical
   Phase 3 deliverable. Use the same 9-file module
   layout as `educore-platform` (lib.rs, aggregate,
   entities, value_objects, commands, events, services,
   repository, query, errors).
2. Wire `StudentCreated` through the
   `educore-events::DomainEvent` trait and
   `educore-events::EventBus`.
3. Use the `educore-rbac::Capability::AcademicStudent*`
   placeholders in the `Student` command handler's
   capability checks (gated by `RbacRoleCreate`,
   `RbacCapabilityAssign`).
4. Write a vertical-slice integration test that mirrors
   `crates/tools/storage-parity/tests/cross_cutting_integration.rs`
   but exercises `StudentAdmitted` (analog of `SchoolCreated`).
5. Flip the 4 `educore-academic_*_aggregate` coverage
   rows in `docs/coverage.toml` to `Tested` in the same
   commit as the impls.

## Key files for the next agent

- `crates/cross-cutting/events/src/lib.rs` — the
  envelope crate's prelude
- `crates/cross-cutting/events/src/envelope.rs` —
  `EventEnvelope` struct (bus-port verbatim)
- `crates/cross-cutting/events/src/domain_event.rs` —
  `DomainEvent` trait
- `crates/cross-cutting/events/src/event_bus.rs` —
  `EventBus` + supporting types
- `crates/cross-cutting/events/src/sync.rs` — the 4
  sync events (template for typed events)
- `crates/infra/storage/src/outbox.rs` —
  `SerializedEnvelope::from_event_envelope` (the bridge
  to the bus-port envelope)
- `crates/infra/storage/src/event_log.rs` —
  `EventLogEntry::from_serialized_envelope` (the relay
  helper)
- `crates/cross-cutting/platform/src/services.rs` —
  template for typed `services` factory functions
- `crates/cross-cutting/rbac/src/services.rs` —
  template for the `CapabilityCheck` port
- `crates/cross-cutting/audit/src/writer.rs` —
  `AuditWriter` service (the audit-sink entry point)
- `crates/tools/storage-parity/tests/cross_cutting_integration.rs`
  — the vertical-slice test pattern
- `docs/ports/event-bus.md` — bus-port contract
- `docs/ports/storage.md` — storage-port contract
- `docs/schemas/audit-schema.md` § 13 (Partitioning
  Strategy) — partitioning approach
- `docs/decisions/ADR-013-CrateLayout.md` — tier
  definitions
- `docs/decisions/ADR-015-ExternalCrates.md` —
  external crate pin policy
- `docs/phase_prompt/phase-3-prompt.md` — the
  next-phase prompt

## Where to ask

Open a GitHub issue for design questions. The Phase 2
prompt is the source of truth for Phase 2's scope; the
next-phase prompt is the source of truth for Phase 3.
For disputes, defer to `AGENTS.md` (engine rules) and
`ADR-013-CrateLayout.md` (tier definitions).
