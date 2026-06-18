# Phase 13 → Phase 14 Hand-off

**Audience:** the next agent starting Phase 14
(`educore-settings` + `educore-operations`).
**Status:** Phase 13 closed. **`educore-events-domain`** is the
eleventh domain-context crate shipped. **Spec-faithful 7-root
interpretation** (per the Phase 13 scope decision): the Calendar
domain owns 7 root aggregates (CalendarEvent, Holiday, Weekend,
Incident, AssignIncident, IncidentComment, CalendarSetting) + 4
child entities (CalendarEventAudience embedded,
CalendarEventAttachment, HolidayAttachment, HolidayPeriod).

## Headline numbers

- **7 root aggregates** (spec-faithful 7-root, not the
  prompt's 4-root headline) with full domain fields, validation,
  and state transitions.
- **24 typed events** implementing `DomainEvent` (wire form
  `events.<aggregate>.<verb>`).
- **24 typed commands** with `into_new_*` helpers and
  `COMMAND_TYPE` constants.
- **7 repository port traits** with 7-9 methods each
  (object-safe).
- **7 typed query stubs** with builder methods.
- **5 service factory structs** (CalendarService,
  RecurrenceService, HolidayService, CalendarSettingService,
  IncidentService, WeekendService) + 1 `WeekendChange` enum.
- **30 net-new `Capability` variants** in `educore-rbac` (4
  retained Phase 2 `EventsCalendar*` placeholders + 30 net-new
  = 34 total Events caps).
- **4 net-new `AuditTarget` variants** in `educore-audit` (3
  retained Phase 2 placeholders `CalendarEvent`/`Holiday`/
  `Incident` + 4 net-new `Weekend`/`AssignIncident`/
  `IncidentComment`/`CalendarSetting` = 7 total Events audit
  targets).
- **34 unit tests** in `educore-events-domain` + **7 always-on
  integration tests** + 2 env-gated `#[ignore]` PG/MySQL variants
  in `events_integration.rs`.
- **9 `coverage.toml` rows** added (1 existing + 8 net-new); 7
  flip from `Pending` → `Tested`.

## Validation gates (all green)

- `cargo build -p educore-events-domain` — clean
- `cargo build --workspace` — clean
- `cargo test -p educore-events-domain --lib` — **34 passed, 0
  failed**
- `cargo test -p educore-rbac --lib` — passed (the new
  `events_capabilities_round_trip_and_resolve_to_events_domain`
  test asserts ≥ 30 Events caps)
- `cargo test -p educore-audit --lib` — passed (the new
  `events_audit_target_round_trip_for_all_aggregates` test asserts
  7 Events targets, all snake_case, no duplicates)
- `cargo test -p educore-storage-parity --test
  events_integration` — **7 passed, 2 ignored** (env-gated PG/MySQL)
- `cargo test --workspace` — all green (Phase 12 baseline
  preserved)
- `cargo fmt --all -- --check` — clean
- `cargo run -p educore-core --bin lint --features lint` — clean

> **Note on `cargo clippy --workspace --all-targets -- -D warnings`:**
> pre-existing clippy debt in `educore-finance` (Phase 7 WIP),
> `educore-hr` (Phase 6 WIP), and `educore-facilities` (Phase 8
> WIP) prevents this gate from being green at the workspace
> level. The events-domain crate itself passes clippy. The
> pre-existing issues are unrelated to Phase 13 and are documented
> as outstanding work in `docs/progress-tracker.md`.

## What's wired and working

### `educore-events-domain` (`crates/cross-cutting/events-domain/`)

The eleventh domain-context crate. Lives in the **cross-cutting
tier** (per `docs/build-plan.md` § "Phase 13"), NOT in
`crates/domains/`. 9-file module layout honored exactly. **Two
`events` crates are easy to confuse** — this is the Calendar
domain, NOT the envelope crate (`crates/cross-cutting/events/`
which is the Phase 2 locked envelope + bus port). Documented
in both `lib.rs` headers.

- [`CalendarEvent`](crates/cross-cutting/events-domain/src/aggregate.rs) —
  the headline. 3 events (Created/Updated/Deleted) + 3 commands.
  `RRULE` subset (DAILY/WEEKLY/MONTHLY/YEARLY + INTERVAL + COUNT +
  UNTIL) via `RecurrenceRule`. `RecurrenceService::expand` applies
  holiday overrides. `CalendarService::audience_resolves_to` +
  `visible_to` + `in_range` + `overlaps`. `RecurrenceService` is
  the canonical "what dates does this recurring event fire on?"
  answer.
- [`Holiday`](crates/cross-cutting/events-domain/src/aggregate.rs) —
  date range with title + details + image. 3 events + 3 commands.
  `HolidayService::is_instructional` is the canonical "is school
  in session on date X?" answer (false if weekend day or holiday).
- [`Weekend`](crates/cross-cutting/events-domain/src/aggregate.rs) —
  ordered list of weekend days. 4 events (Created/Updated/
  Configured/Deleted) + 4 commands. `WeekendService::reconcile`
  is the canonical diff for `ConfigureWeekends` (3-way create/
  update/delete).
- [`Incident`](crates/cross-cutting/events-domain/src/aggregate.rs) —
  reported incident with point + description + status. 4 events
  (Reported/Updated/Resolved/Deleted) + 4 commands.
  `IncidentService::next_status` is the canonical state machine:
  `Open → InProgress → Resolved`. Re-issuing `Resolved` is a
  no-op. Body is immutable after Resolved (per spec invariant 5).
- [`AssignIncident`](crates/cross-cutting/events-domain/src/aggregate.rs) —
  mapping of an Incident to a student/staff. 3 events (Assigned/
  Reassigned/Unassigned) + 3 commands. Exactly one of
  `student_id` or `user_id` must be set.
- [`IncidentComment`](crates/cross-cutting/events-domain/src/aggregate.rs) —
  append-only comment. 2 events (Commented/Deleted) + 2 commands.
- [`CalendarSetting`](crates/cross-cutting/events-domain/src/aggregate.rs) —
  categorical label for the calendar UI with display colors.
  5 events (Created/Updated/Enabled/Disabled/Deleted) + 5
  commands. `validate_css_color` accepts hex (#RGB/#RRGGBB/
  #RRGGBBAA), rgb()/rgba(), or alphabetic named colors.

### Child entities (4)

- `CalendarEventAudience` (embedded in CalendarEvent)
- `CalendarEventAttachment` (owned by CalendarEvent)
- `HolidayAttachment` (owned by Holiday)
- `HolidayPeriod` (owned by Holiday; supports split holidays)

### Typed ids

7 root ids + 3 child entity ids + `AcademicYearRef` (local
`Uuid` newtype, no `educore-academic` dep). All use the
`events_typed_id!` macro mirroring `educore-cms`'s
`cms_typed_id!`.

### 5 service factory structs

- `CalendarService` (in_range, audience_resolves_to, overlaps,
  visible_to, is_published, validate_url)
- `RecurrenceService` (expand with holiday overrides — the
  headline correctness check)
- `HolidayService` (contains, overlaps, is_instructional,
  instructional_days_in)
- `CalendarSettingService` (validate_color, visible)
- `IncidentService` (next_status, total_points, is_resolved)
- `WeekendService` (reconcile, is_weekend, ordered)

### Cross-crate extensions

- `educore-rbac`: 30 net-new `Events*` Capability variants
  (Event 5 + Holiday 4 + Weekend 5 + Incident 9 +
  IncidentComment 1 + CalendarSetting 6). The 4 Phase 2
  `EventsCalendar{Create,Read,Update,Delete}` placeholders are
  retained for `DefaultRoleCatalog` consistency (mirrors the
  Phase 12 CMS pattern).
- `educore-audit`: 4 net-new `Events*` AuditTarget variants
  (Weekend, AssignIncident, IncidentComment, CalendarSetting).
  The 3 Phase 2 placeholders (CalendarEvent, Holiday, Incident)
  are retained.
- `crates/tools/storage-parity`: `educore-events-domain` dep
  added.

### `educore-storage-parity` integration test

`crates/tools/storage-parity/tests/events_integration.rs` —
mirrors the Phase 9–12 `cms_integration.rs` pattern. 7 always-on
scenarios + 2 env-gated `#[ignore]` PG/MySQL variants:

1. `events_integration_sqlite_vertical_slice` — CalendarEvent +
   Holiday + Incident + Weekend creation
2. `events_capability_check_gates_event_publish` — capability
   gate + wire form assertion
3. `events_event_type_round_trip_for_all_aggregates` — 24 event
   types resolve to expected `events.<aggregate>.<verb>` strings
4. `events_rrule_expansion_subset` — DAILY/WEEKLY/MONTHLY/YEARLY
   × INTERVAL × COUNT × UNTIL matrix
5. `events_holiday_overrides_recurring_event` — RRULE + holiday
   override (the headline RRULE × Holiday interaction)
6. `events_incident_status_state_machine` — Open→InProgress→
   Resolved transitions + immutability after Resolved
7. `events_weekend_reconcile_diff` — 3-way diff (create/update/
   delete) + idempotent re-issue

## Cross-crate placeholders

**4 retained** Phase 2 `EventsCalendar*` capability placeholders.
**30 net-new** variants. **3 retained** Phase 2 AuditTarget
placeholders (CalendarEvent, Holiday, Incident). **4 net-new**
AuditTarget variants. No `CommunicationMessage*` / `Documents*` /
finance placeholders touched.

## Concurrency strategy

Per the Phase 9–12 hand-off template: **Phase 13 has no new
concurrency strategy**; append-only invariants are enforced at
the trait level; `Incident` and `Weekend` state-machine
transitions are enforced at the aggregate level via the
`next_status` and `reconcile` methods.

The same row-level lock strategy as Phases 7–12 applies: the
dispatcher acquires the row-level lock on the relevant row
(PG `SELECT ... FOR UPDATE` or SQLite write lock) before calling
the service and writing audit / outbox / idempotency rows in a
single transaction.

Soft-delete pattern: all 7 root aggregates set `active_status =
false` on delete; the row is never hard-deleted; `find_*`
queries filter on `active_status = true`.

## Headline correctness check

The `RecurrenceService::expand` method is the headline
correctness check. It:
1. Expands the RRULE from the start date (DAILY/WEEKLY/MONTHLY/
   YEARLY × INTERVAL × COUNT × UNTIL)
2. Excludes any date that falls within a holiday range
   (holidays override recurring events per
   `docs/specs/events/aggregates.md` CalendarEvent rule 4)

Verified by `events_holiday_overrides_recurring_event` and
`events_rrule_expansion_subset` integration tests.

## Open questions

1. **`AcademicYearRef` strategy** (NEW) — local `Uuid` newtype
   used in `value_objects.rs` (no `educore-academic` dep, per
   the prompt's gotcha). The foreign-key relationship to
   `academic_academic_years.id` is enforced at the storage
   adapter layer. A follow-up phase may promote to a true
   cross-crate `AcademicYearId` re-export.
2. **RFC 5545 BYDAY/BYHOUR/MIN/SECOND** (NEW) — NOT in scope
   for Phase 13. `RecurrenceRule` supports FREQ + INTERVAL +
   COUNT + UNTIL only. Follow-up phase may add.
3. **No `educore-finance` dep** (carry-over from Phase 8 OQ
   #6, Phase 10 OQ #3, Phase 11 OQ #4). Phase 13 had no
   finance touch.
4. **No `educore-notify` dep** (carry-over from Phase 10 OQ
   #4, Phase 11 OQ #4). Phase 13 had no notify fan-out.
5. **No `educore-attendance` dep** (carry-over from Phase 10
   OQ #5). Phase 13 had no attendance integration.
6. **No `educore-documents` dep** (Phase 11 OQ #6). Phase 13
   has no documents integration.
7. **No `educore-academic` dep** (Phase 13 spec). The Calendar
   domain does not reference class/section/year (per the
   prompt's gotcha). `AcademicYearRef` is a local Uuid newtype.

## Where NOT to start (Phase 14)

- Do NOT add a `educore-finance` dep (Phase 8 OQ #6 carry-over
  + Phase 10 OQ #3 + Phase 11 OQ #4 + Phase 12 OQ #5 +
  Phase 13 OQ #3).
- Do NOT add a `educore-notify` dep (Phase 10 OQ #4 carry-over
  + Phase 11 OQ #4 + Phase 12 OQ #6 — port lands in Phase 15).
- Do NOT add a `educore-attendance` dep (Phase 10 OQ #5
  carry-over + Phase 12 OQ #7 + Phase 13 OQ #5).
- Do NOT add a `educore-documents` dep (Phase 11 OQ #6 +
  Phase 12 OQ #8 + Phase 13 OQ #6).
- Do NOT add a `educore-academic` dep (Phase 13 OQ #7 — Phase 14
  Settings + Operations may need it; the `educore-settings`
  crate is already a dep of `educore-events-domain`).
- Do NOT remove the 4 Phase 2 `EventsCalendar*` capability
  placeholders or add them back. They were preserved in
  Phase 13.
- Do NOT remove the 3 Phase 2 AuditTarget placeholders
  (CalendarEvent, Holiday, Incident) or add them back. They
  were preserved in Phase 13.
- Do NOT touch the 18 closed crates other than the additive
  rbac + audit extensions + the 1 `Cargo.toml` addition to
  storage-parity. Per `ADR-013-CrateLayout.md`, the
  cross-crate modifications are all non-breaking additive.
- Do NOT touch `educore-core::lint`. The lint binary passes;
  the tier-boundary checker remains a stub.
- Do NOT remove the 4 Phase 2 `CommunicationMessage*`
  capability placeholders or add them back. They were
  deduplicated in Phase 10.
- Do NOT remove the 4 Phase 2 `CmsPage*` capability placeholders
  or add them back. They were preserved in Phase 12.

## Key files for the next agent

- `crates/cross-cutting/events-domain/.phase13-manifest.md` —
  the Phase 13 manifest (the canonical spec, single source of
  truth)
- `crates/cross-cutting/events-domain/src/value_objects.rs` —
  7 root typed ids + 3 child ids + `AcademicYearRef` + 5 enums +
  RRULE subset + `apply_holiday_overrides` helper
- `crates/cross-cutting/events-domain/src/aggregate.rs` — 7 root
  aggregates with full domain fields + constructors + state
  transitions
- `crates/cross-cutting/events-domain/src/entities.rs` — 4 child
  entities (CalendarEventAudience embedded, CalendarEventAttachment,
  HolidayAttachment, HolidayPeriod)
- `crates/cross-cutting/events-domain/src/events.rs` — 24 typed
  events implementing `DomainEvent` (wire form
  `events.<aggregate>.<verb>`)
- `crates/cross-cutting/events-domain/src/commands.rs` — 24
  typed command shapes + 24 `EVENTS_*_COMMAND_TYPE` constants +
  the `into_new_*` helpers
- `crates/cross-cutting/events-domain/src/repository.rs` — 7
  `pub trait XxxRepository: Send + Sync` port traits
  (object-safety smoke tests included)
- `crates/cross-cutting/events-domain/src/query.rs` — 7 typed
  query stubs + builder methods
- `crates/cross-cutting/events-domain/src/services.rs` — 5
  service structs + 1 `WeekendChange` enum + the
  `RecurrenceService` (headline correctness check) + proptest-style
  unit tests
- `crates/cross-cutting/events-domain/src/errors.rs` — the
  `EventsDomainError` enum + `Result` alias + `From<DomainError>`
  and `From<EventError>` impls
- `crates/cross-cutting/events-domain/src/lib.rs` — the 9-file
  prelude + the package-name constants + the **critical
  two-`events`-crates confusion** note
- `crates/tools/storage-parity/tests/events_integration.rs` —
  the 7-scenario vertical-slice test + 2 env-gated
  PG/MySQL variants
- `crates/cross-cutting/rbac/src/value_objects.rs` — the 30
  net-new `Capability` variants (4 retained
  `EventsCalendar*` placeholders + 30 net-new) + the
  round-trip test
- `crates/cross-cutting/audit/src/writer.rs` — the 4 net-new
  `Events*` AuditTarget variants (3 retained
  CalendarEvent/Holiday/Incident + 4 net-new Weekend/
  AssignIncident/IncidentComment/CalendarSetting) + the
  round-trip test
- `crates/tools/storage-parity/Cargo.toml` — the new
  `educore-events-domain` dep
- `crates/cross-cutting/events-domain/Cargo.toml` — the 11
  deps (5 from scaffold + 6 new: `educore-audit`,
  `educore-storage`, `async-trait`, `thiserror`, `bytes`,
  `tracing`, plus `chrono`/`serde`/`serde_json`/`uuid`)
- `docs/coverage.toml` — 9 rows added (1 existing
  `events_calendar_events_aggregate` + 8 net-new); 7 flip from
  `Pending` to `Tested`
- `docs/handoff/PHASE-13-HANDOFF.md` — this hand-off
- `docs/phase_prompt/phase-14-prompt.md` — the next-phase
  brief

## Where to ask

Open a GitHub issue for design questions. The Phase 13 prompt
is the source of truth for Phase 13's scope; the next-phase
prompt is the source of truth for Phase 14's. For disputes,
defer to `AGENTS.md` (engine rules) and `ADR-013-CrateLayout.md`
(tier definitions).
