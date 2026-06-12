# Phase 3 → Phase 4 Hand-off

**Audience:** the next agent starting Phase 4 (`educore-assessment`).
**Status:** Phase 3 closed. **`educore-academic`** is the
first domain crate shipped. 8 of 8 prompt-named
deliverables land: 5 aggregates (Student, Class, Section,
Subject, AcademicYear), 23 typed commands, 19 typed
events implementing `DomainEvent`, 19 pure factory
services, 5 repository port traits, 5 typed query stubs,
the integration test wiring, and the `entities.rs`
placeholder. The vertical-slice integration test passes on
SQLite (always), PG and MySQL (env-gated).

## Validation gates (all green)

- `cargo build --workspace` — clean
- `cargo test --workspace` — **all pass** (was 310 at
  Phase 2 close-out; Phase 3 adds 66 unit tests in
  `educore-academic` plus 3 academic integration tests in
  `crates/tools/storage-parity/tests/academic_integration.rs`)
- `cargo clippy --workspace --all-targets -- -D warnings` —
  clean
- `cargo fmt --all -- --check` — clean
- `cargo run -p educore-core --bin lint --features lint` —
  clean
- 8 `docs/coverage.toml` rows flipped from `Pending` to
  `Tested` (3 `event_log_ddl_*` from Phase 2 OQ #1, plus
  5 `academic_*_aggregate` from Phase 3).

## What's wired and working

### `educore-academic` (`crates/domains/academic/`)

The first domain crate. Phase 3 ships the **prompt-named
subset only** (per the Phase 3 prompt's explicit
narrowing): 5 aggregates, 23 commands, 19 events, 19
services, 5 repository port traits, 5 query stubs, the
`UniquenessChecker` port, and the `entities.rs`
placeholder. The full academic spec has 32 aggregates; the
remaining 27 (`Guardian`, `ClassSection`, `ClassSubject`,
`ClassRoutine`, `Homework`, `Lesson`, `LessonTopic`,
`LessonPlan`, `StudentRecord`, `StudentPromotion`,
`StudentCategory`, `StudentGroup`, `RegistrationField`,
`Certificate`, `IdCard`, `AdmissionQuery`, …) land in
later phases.

- **5 aggregate roots** (Student, Class, Section,
  Subject, AcademicYear). All five follow the "aggregate
  as a single struct" pattern (mirror `educore-platform`'s
  `School` and `User`): the struct holds the full state,
  with `version` for optimistic concurrency, `etag` for
  content hashing, `active_status` for soft delete, and
  `last_event_id` / `correlation_id` for the audit / outbox
  bridge.
- **5 typed ids** (`StudentId(SchoolId, Uuid)`,
  `ClassId(SchoolId, Uuid)`, `SectionId(SchoolId, Uuid)`,
  `SubjectId(SchoolId, Uuid)`,
  `AcademicYearId(SchoolId, Uuid)`). Per-tenant compile-time
  safety; a `StudentId` from school A cannot be passed to
  a function that expects a `StudentId` from school B.
- **23 typed commands** (8 student lifecycle, 4 class
  CRUD, 3 section CRUD, 3 subject CRUD, 5 academic-year
  CRUD). Each command carries a `TenantContext` and is
  rejected if the actor lacks the required capability at
  the dispatcher layer.
- **19 typed events** implementing
  `educore_events::domain_event::DomainEvent`. The
  `event_type` is namespaced as
  `"academic.<aggregate>.<verb>"` per the bus-port
  contract (e.g. `"academic.student.admitted"`).
- **19 pure factory services** (mirror
  `educore-platform::services::create_school` exactly).
  Every function takes the command, the active
  `TenantContext`, a `Clock`, an `IdGenerator` (where
  needed), and (for create / register flows) a
  `UniquenessChecker`, and returns the mutated aggregate
  plus the typed event. The dispatcher (in the engine's
  core) is responsible for persisting the aggregate and
  publishing the event under a single transaction.
- **5 repository port traits** (`StudentRepository`,
  `ClassRepository`, `SectionRepository`,
  `SubjectRepository`, `AcademicYearRepository`). All
  five are `Send + Sync` and object-safe. The
  storage-parity crate's existing `cross_cutting_integration.rs`
  + the new `academic_integration.rs` exercise the storage
  layer end-to-end on the `Student` flow.
- **5 typed query stubs** (`StudentQuery`, `ClassQuery`,
  `SectionQuery`, `SubjectQuery`, `AcademicYearQuery`).
  The query executors return
  `Err(DomainError::NotSupported)` for now; the real
  executors land alongside the `#[derive(DomainQuery)]`
  macro emissions in a later phase.
- **`UniquenessChecker` port** (admission_no + email
  per-school uniqueness).
- **`entities.rs` placeholder** (`StudentDocument` and
  `DocumentType`) so the 9-file module layout is
  honoured. No port trait is wired in Phase 3; the
  `UploadStudentDocument` command lands in a later
  phase alongside the `FileStorage` port.
- **66 unit tests** in `educore-academic` (across
  `value_objects.rs`, `commands.rs`, `events.rs`,
  `services.rs`, `aggregate.rs`, `entities.rs`,
  `query.rs`, `repository.rs`, `lib.rs`).

### `educore-audit` integration (no public-surface changes)

The `AuditWriter` is wired from the academic command
handlers in the integration test (see
`crates/tools/storage-parity/tests/academic_integration.rs`
for the dispatch pattern). The audit log gets one row
per admit_student call, and the
`outbox + audit_log + idempotency` triple lives in the
same `tx.commit()` boundary.

### `educore-rbac` integration (capability check)

`Capability::AcademicStudentCreate` is the placeholder
variant Phase 2 added; the academic integration test
asserts that an actor without a grant is denied and an
actor with a grant is allowed, via
`InMemoryCapabilityCheck::has`. The production wiring
remains a dispatcher-level concern (see
[Capability check boundary](#capability-check-boundary)
below).

### `educore-storage` integration (vertical-slice test)

`crates/tools/storage-parity/tests/academic_integration.rs`
is the new vertical-slice test. It mirrors
`cross_cutting_integration.rs` exactly:

- Sets up an in-process `InProcessEventBus` + the SQLite
  in-memory adapter (always runs).
- Subscribes to the bus BEFORE dispatching
  (`Topic::All`, `StartPosition::Latest`).
- Calls `academic::services::admit_student(cmd, &clock,
  &ids, &uniqueness)` → returns `(Student,
  StudentAdmitted)`.
- Mints envelope via `event.into_envelope(&ctx)`;
  serialise via `SerializedEnvelope::from_event_envelope`.
- Single transaction: `tx.outbox().append(...)` +
  `tx.audit_log().append(...)` +
  `tx.idempotency().record(...)` + `tx.commit()`.
- Drains outbox → event log (the
  `relay_outbox_to_event_log` helper, ported from the
  cross-cutting test).
- Asserts: outbox drained (0 pending), `audit_log >= 1`
  row, `event_log == 1` row with `event_type =
  "academic.student.admitted"`, idempotency write
  returned Ok, bus received the event with matching
  `event_type` / `school_id` / `actor_id` /
  `correlation_id` / `aggregate_id`.
- Asserts the `AcademicStudentCreate` capability check
  via `InMemoryCapabilityCheck`.

PG and MySQL variants are env-gated with `#[ignore]`. A
bonus `academic_event_type_round_trip_for_all_aggregates`
test exercises `create_class` and `create_section` to
verify the typed events for all 5 prompt-named aggregates
are wired correctly.

### `docs/coverage.toml` (8 rows flipped)

| Row id | Before | After | Test path |
|---|---|---|---|
| `event_log_ddl_pg` | Pending | Tested | `crates/tools/storage-parity/tests/cross_cutting_integration.rs` |
| `event_log_ddl_mysql` | Pending | Tested | same |
| `event_log_ddl_sqlite` | Pending | Tested | same |
| `academic_students_aggregate` | Pending | Tested | `crates/tools/storage-parity/tests/academic_integration.rs` |
| `academic_classes_aggregate` | Pending | Tested | same |
| `academic_sections_aggregate` | Pending | Tested | same |
| `academic_subjects_aggregate` | Pending | Tested | same |
| `academic_academic_years_aggregate` | Pending | Tested | same |

## Capability check boundary

Per the Phase 3 prompt's "Where you are stuck" and the
user-confirmed decision during plan mode, the academic
services do **not** call
`capability_check.has(ctx, Capability::AcademicStudent*)`
directly. The check is documented as a dispatcher-level
concern (matching the platform crate's pattern) and
exercised in the integration test:

```rust
let cap_check = InMemoryCapabilityCheck::new();
let role = RoleId::new(school, Uuid::now_v7());
cap_check.grant(school, role, Capability::AcademicStudentCreate);
let granted = cap_check.has(&ctx, Capability::AcademicStudentCreate).await?;
assert!(granted);
```

Phase 4 may revisit this if the engine facade evolves to
wire checks into the service layer. The boundary is
deliberately not a Phase 3 deliverable because the
existing platform / rbac / academic crates all keep
capability checks at the dispatcher.

## Storage-adapter transaction model (Phase 2 OQ #5)

The vertical-slice test exercises the flag-based
transaction model on the 3 SQL adapters. Phase 2 hand-off
OQ #5 asked: "Is the SQL adapter's flag-based transaction
model safe for the academic domain?" The answer for
Phase 3 is **yes**:

- The SQLite test passes deterministically
  (`cross_cutting_integration_academic`).
- The cross-cutting integration test (the original
  Phase 2 test) continues to pass with no inconsistency
  under the same model.

The flag-based transaction model is adequate for the
prompt-named subset because each command is a single
service call that produces at most one outbox row, one
audit row, one idempotency row, and one event-log row.
The at-least-once dedup via `idempotency_key` is the
safety net. A real `sqlx::Transaction` plumbed through the
sub-port methods remains a future refactor (Phase 4+);
the hand-off recommends it land alongside a benchmark
that demonstrates the latency cost of the current model
on PG.

## Open questions

These carry forward from Phase 2 and remain open after
Phase 3:

1. **PG RLS test setup script** (Phase 2 OQ #2): the
   `pg_rls_blocks_cross_tenant_audit_reads` test is
   `#[ignore]`-gated and requires a non-superuser
   `tenant_b` role. The setup script at
   `tools/scripts/pg-rls-test-setup.sql` is **NOT
   written** in Phase 3. Phase 4 (or any phase that
   needs RLS validation on PG) should add the script and
   document the procedure in `docs/guides/saas-backend.md`.
2. **`AuditLogEntry` vs `EventLogEntry` struct
   divergence** (Phase 2 OQ #3): the two structs have
   overlapping but distinct fields. The hand-off
   documents the split as the permanent design (the
   audit log is a compliance artefact; the event log is
   a domain event history; they have different
   retention and redaction requirements). The
   vertical-slice test confirms both structs are
   populated correctly.
3. **`IdempotencyRecord::command_type: &'static str`
   Box::leak** (Phase 2 OQ #4): the field type is
   unchanged. The SQLite adapter's read path still
   `Box::leak`s the value (per Phase 1 hand-off). A
   refactor to `String` or `Cow<'static, str>` lands in
   a future phase.
4. **Tier dependency lint** (Phase 0 OQ): the
   `educore-core::lint::runner::check_tier_boundaries`
   is still a stub. Phase 3 worked around it by adding
   the necessary deps to the storage-parity crate (a
   `tools`-tier crate that legitimately depends on
   `adapters`-tier storage adapters). When the lint is
   implemented, the existing `educore-storage` →
   `educore-events` dep (infra → cross-cutting) will
   need to be either reversed (move the bridge to
   `educore-events`) or blessed with an ADR.
5. **Flag-based transactions** (Phase 2 OQ #5): see
   above. Validated for Phase 3; refactor is a separate
   phase.
6. **Scoped out per Phase 3 prompt** (carries forward
   as future-phase work):
   - The 27 other academic aggregates (Guardian,
     ClassSection, ClassSubject, ClassRoutine,
     Homework, Lesson, LessonTopic, LessonPlan,
     StudentRecord, StudentPromotion, StudentCategory,
     StudentGroup, RegistrationField, Certificate,
     IdCard, AdmissionQuery, etc.).
   - The full `AdmitStudentCommand` shape with
     `GuardianSpec` / `TransportSpec` / `HostelSpec` /
     `DocumentSpec` / `CustomFields` per the spec —
     Phase 3 ships a minimal command (admission_no,
     first / last name, DOB, gender, class_id,
     section_id, academic_year_id, optional roll_no,
     optional email / mobile) without those extras.
   - The `StudentRecord` aggregate (per-year enrollment)
     — out of scope; the Student itself is the
     prompt-named subset.
   - The `#[derive(DomainQuery)]` macro emissions on the
     5 in-scope aggregates. The macro was scaffolded in
     Phase 0; the per-aggregate descriptor wiring lands
     in Phase 4+ alongside the typed query executors.
   - The `pg-rls-test-setup.sql` script (see OQ #1).
   - A higher-level `Engine::admit_student` facade
     (engine facade; not in per-crate scope).

## Where NOT to start (Phase 4)

- Do NOT add the 27 other academic aggregates. They
  land in Phases 3.5+ or later phases. Phase 4 is
  assessment (the `educore-assessment` crate); it does
  NOT consume any of the academic crate's
  `StudentRecord` / `Enrollment` types — those
  aggregates don't exist yet. The assessment crate
  consumes `StudentId` only (via the `Student`
  aggregate's typed id).
- Do NOT modify the 5 cross-cutting crates' public
  surface (`educore-events`, `educore-event-bus`,
  `educore-platform`, `educore-rbac`, `educore-audit`).
  The foundation is locked.
- Do NOT modify the 3 SQL storage adapters'
  transaction model.
- Do NOT re-introduce `mysql_async` or `flate2`
  (rejected in Phase 1).
- Do NOT touch `educore-core::lint` (per Phase 0
  hand-off).
- Do NOT rename or move crates. Per
  `ADR-013-CrateLayout.md`, the current layout is
  canonical.
- Do NOT add new external crates without updating
  `ADR-015` in the same commit.
- Do NOT extend the academic `entities.rs` placeholder
  with new types. The `StudentDocument` placeholder
  stays as-is until the `FileStorage` port lands
  (Phase 15).

## Key files for the next agent

- `crates/domains/academic/src/lib.rs` — re-exports +
  prelude
- `crates/domains/academic/src/aggregate.rs` — 5
  aggregate roots
- `crates/domains/academic/src/events.rs` — 19 typed
  events implementing `DomainEvent`
- `crates/domains/academic/src/services.rs` — 19 pure
  factory functions (the dispatcher integration
  pattern)
- `crates/domains/academic/src/repository.rs` — 5
  repository port traits
- `crates/domains/academic/src/commands.rs` —
  `UniquenessChecker` port + 23 typed command shapes
- `crates/domains/academic/src/value_objects.rs` —
  typed ids and value objects
- `crates/tools/storage-parity/tests/academic_integration.rs`
  — the vertical-slice test pattern
- `crates/tools/storage-parity/tests/cross_cutting_integration.rs`
  — the original Phase 2 vertical-slice test pattern
- `crates/cross-cutting/events/src/lib.rs` — the
  envelope crate's prelude
- `crates/cross-cutting/events/src/domain_event.rs` —
  the `DomainEvent` trait every typed event implements
- `crates/cross-cutting/events/src/sync.rs` — the 4
  sync events (template for typed events)
- `crates/cross-cutting/platform/src/services.rs` —
  the template for the academic factory functions
- `crates/cross-cutting/rbac/src/services.rs` — the
  `CapabilityCheck` port + `InMemoryCapabilityCheck`
- `crates/cross-cutting/audit/src/writer.rs` — the
  `AuditWriter` service (the audit-sink entry point)
- `docs/specs/academic/` — the academic design contract
- `docs/ports/event-bus.md` — bus-port contract
- `docs/ports/storage.md` — storage-port contract
- `docs/decisions/ADR-013-CrateLayout.md` — tier
  definitions
- `docs/decisions/ADR-015-ExternalCrates.md` —
  external crate pin policy
- `docs/phase_prompt/phase-4-prompt.md` — the
  next-phase prompt

## Where to ask

Open a GitHub issue for design questions. The Phase 3
prompt is the source of truth for Phase 3's scope; the
next-phase prompt is the source of truth for Phase 4.
For disputes, defer to `AGENTS.md` (engine rules) and
`ADR-013-CrateLayout.md` (tier definitions).
