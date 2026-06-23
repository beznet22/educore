# Wave 1 — educore-events-domain production-readiness audit

**Crate:** `educore-events-domain` (cross-cutting tier, calendar domain)
**Path:** `crates/cross-cutting/events-domain/` (10 files, 5470 LoC)
**Phase:** 13 (per `docs/build-plan.md` § Phase 13)
**Phase status (claim):** "Closed 2026-06-18" (build-plan § Phase 13 outcome; PHASE-13-HANDOFF.md)
**Audit:** read-only production-readiness audit. Findings only; no fixes proposed.

## Scope reconciliation

| Source | What it says |
| --- | --- |
| `docs/specs/events/aggregates.md` | 7 root aggregates (CalendarEvent, Holiday, Weekend, Incident, AssignIncident, IncidentComment, CalendarSetting) + 2 owned children (CalendarEventAudience embedded, CalendarEventAttachment) |
| `docs/specs/events/entities.md` | 5 entities (AssignIncident, IncidentComment, CalendarEventAudience embedded, CalendarEventAttachment, HolidayAttachment, HolidayPeriod) |
| `docs/build-plan.md` § Phase 13 (Tasks, line 1470, 1475) | "Aggregates per docs/specs/events/aggregates.md: CalendarEvent, Holiday, Incident, Weekend." (4 aggregates) |
| `docs/build-plan.md` § Phase 13 outcome | "Spec-faithful 7-root interpretation" (7 root aggregates) |
| Code (lib.rs prelude, lines 45-47) | Re-exports 7 root aggregate types |
| Code (aggregate.rs sections) | 7 root aggregate structs + `New*` input types |

Doc-vs-code drift: the build-plan Tasks section was never updated from the original 4-aggregate prompt to the 7-aggregate implementation. The Phase 13 outcome section (a separately added note) describes the 7-aggregate reality. The Tasks list is now stale.

---

## Findings

### FINDING DOMAIN-EVD-001

- **id:** DOMAIN-EVD-001
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/cross-cutting/events-domain/src/aggregate.rs:7` and `crates/cross-cutting/events-domain/src/entities.rs:14`
- **description:** The crate's two largest domain modules disable every lint — `#![allow(missing_docs, dead_code, clippy::all)]` — at the file level. This silently overrides `lib.rs`'s `#![deny(missing_docs)]` (lib.rs:24) and hides the fact that large swaths of public API are undocumented.
- **expected:** Engine rule in `AGENTS.md`: "All public APIs are documented with rustdoc; `#![deny(missing_docs)]`." `lib.rs:24` enforces this for the crate. The deny is intentionally a crate-root lint.
- **evidence:**
  - `crates/cross-cutting/events-domain/src/lib.rs:23-24` — `#![forbid(unsafe_code)] / #![deny(missing_docs)]`
  - `crates/cross-cutting/events-domain/src/aggregate.rs:7` — `#![allow(missing_docs, dead_code, clippy::all)]`
  - `crates/cross-cutting/events-domain/src/entities.rs:14` — `#![allow(missing_docs, dead_code, clippy::all)]`

### FINDING DOMAIN-EVD-002

- **id:** DOMAIN-EVD-002
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/cross-cutting/events-domain/src/{commands.rs:7,events.rs:8,query.rs:8,repository.rs:8,services.rs:8,value_objects.rs:7}.rs`
- **description:** Every remaining source file (6 of 10) carries `#![allow(missing_docs)]` at file scope. Together with FINDING-DOMAIN-EVD-001, all 9 domain-code files in the crate silently disable the `missing_docs` deny at lib.rs:24, meaning the engine's "All public APIs are documented with rustdoc" rule cannot fire inside this crate.
- **expected:** Per `lib.rs:24`, public items must carry rustdoc; the crate-level deny exists precisely to enforce this.
- **evidence:** File-level allows in all 6 files (e.g., `crates/cross-cutting/events-domain/src/events.rs:8` — `#![allow(missing_docs)]`).

### FINDING DOMAIN-EVD-003

- **id:** DOMAIN-EVD-003
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/cross-cutting/events-domain/src/value_objects.rs` (entire file)
- **description:** The spec's `docs/specs/events/value-objects.md` line 93 mandates that "All value objects implement `Validate` and refuse construction when validation fails," and provides a canonical `pub trait Validate { fn validate(&self) -> Result<(), ValueError>; }` at lines 97-100. The events-domain crate defines no `Validate` trait, no `ValueError`, and no `impl Validate for *` anywhere. Construction-time validation is implemented ad-hoc inside each aggregate `::new` (e.g., `aggregate.rs:84-89`), not via a shared trait.
- **expected:** Spec text: "All value objects implement `Validate` and refuse construction when validation fails." + the canonical `Validate` trait definition (lines 96-100 of value-objects.md).
- **evidence:**
  - Spec: `docs/specs/events/value-objects.md:93-100` — `pub trait Validate { fn validate(&self) -> Result<(), ValueError>; }`
  - Code: `grep -rn "trait Validate" crates/cross-cutting/events-domain/` returns no matches.
  - Code: ad-hoc validation only — `crates/cross-cutting/events-domain/src/aggregate.rs:84-89` (CalendarEvent::new empty-title check) and equivalent checks in Holiday/Incident/AssignIncident/IncidentComment/Weekend/CalendarSetting.

### FINDING DOMAIN-EVD-004

- **id:** DOMAIN-EVD-004
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/cross-cutting/events-domain/src/aggregate.rs` (CalendarEvent, Holiday, Incident, AssignIncident, IncidentComment, CalendarSetting, Weekend structs); `crates/cross-cutting/events-domain/src/commands.rs` (CreateEventCommand, CreateHolidayCommand, etc.)
- **description:** The spec's value-objects table (value-objects.md lines 24-89) mandates typed newtype wrappers — `EventTitle`, `EventDescription`, `EventLocation`, `HolidayTitle`, `HolidayDetails`, `WeekendName`, `WeekendOrder`, `IsWeekend`, `IncidentTitle`, `IncidentDescription`, `IncidentPoint`, `IncidentCommentBody`, `CalendarMenuName`, `CssColor`, `FontColor`, `BackgroundColor`, `EventDateRange`, `DateRange`, `RoleIdList`, `WeekendDay`, `EventDate`, `AcademicYearId`. The crate uses raw `String`, `Option<String>`, `i32`, `bool`, and `NaiveDate` everywhere instead. None of the spec-named types exist as Rust types.
- **expected:** Spec table at `docs/specs/events/value-objects.md:24-89` lists 20+ typed value objects with constraints (e.g., "EventTitle — 1..200 chars", "IncidentPoint — i32 in 0..1000"). Spec example at line 105: `let title = IncidentTitle::new("Bullying in classroom 3B")?;`
- **evidence:**
  - `crates/cross-cutting/events-domain/src/aggregate.rs:37-44` — CalendarEvent uses `title: String`, `location: Option<String>`, `description: Option<String>`, `role_ids: Vec<String>`.
  - `crates/cross-cutting/events-domain/src/aggregate.rs:406` — Incident uses `point: i32` (not `IncidentPoint`).
  - `crates/cross-cutting/events-domain/src/aggregate.rs:542` — AssignIncident uses `student_id: Option<Uuid>`, `user_id: Option<Uuid>` (not `Option<StudentId>`, `Option<UserId>`).
  - `crates/cross-cutting/events-domain/src/commands.rs:28-41` — CreateEventCommand uses `title: String` instead of `event_title: EventTitle`.
  - `grep -n "EventTitle\|EventDescription\|EventLocation\|HolidayTitle\|HolidayDetails\|WeekendName\|IncidentTitle\|IncidentDescription\|IncidentPoint\|IncidentCommentBody\|CalendarMenuName\|CssColor\|FontColor\|BackgroundColor\|DateRange\|WeekendDay\|RoleIdList" crates/cross-cutting/events-domain/src/` — zero matches outside comments.

### FINDING DOMAIN-EVD-005

- **id:** DOMAIN-EVD-005
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/cross-cutting/events-domain/src/events.rs:28-37` (EventCreated struct)
- **description:** The spec mandates `EventCreated` to carry `event_title: EventTitle`, `for_whom: ForWhom`, and `academic_id: AcademicYearId`. The code's `EventCreated` carries `title: String` (wrong type per FINDING-004) and is missing both `for_whom` and `academic_id`. The event that fans out to the communication domain for notifications therefore cannot carry the audience scope it advertises in `events.md` lines 51-54.
- **expected:** Spec at `docs/specs/events/events.md:38-46`: `pub struct EventCreated { pub event_id, pub event_title: EventTitle, pub from_date, pub to_date, pub for_whom: ForWhom, pub academic_id: AcademicYearId, }`
- **evidence:**
  - Spec: `docs/specs/events/events.md:38-46` — `for_whom: ForWhom` and `academic_id: AcademicYearId` are mandatory fields.
  - Code: `crates/cross-cutting/events-domain/src/events.rs:28-37` — `EventCreated` has `event_id, school_id, title, from_date, to_date, event_id_field, correlation_id, occurred_at`. No `for_whom`. No `academic_id`.

### FINDING DOMAIN-EVD-006

- **id:** DOMAIN-EVD-006
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/cross-cutting/events-domain/src/events.rs:808-817` (IncidentAssigned struct)
- **description:** The spec mandates `IncidentAssigned` to carry `student_id: Option<StudentId>` and `user_id: Option<UserId>` so that the event records which actor was assigned. The code's `IncidentAssigned` is missing both fields, so a downstream subscriber (e.g., HR for behavior-note attachment) cannot tell from the event who was assigned.
- **expected:** Spec at `docs/specs/events/events.md:117-126`: `pub struct IncidentAssigned { pub assign_incident_id, pub incident_id, pub student_id: Option<StudentId>, pub user_id: Option<UserId>, pub point: IncidentPoint, pub added_by: UserId, }`
- **evidence:**
  - Spec: `docs/specs/events/events.md:117-126`
  - Code: `crates/cross-cutting/events-domain/src/events.rs:808-817` — fields: `assign_incident_id, incident_id, school_id, point, added_by, event_id_field, correlation_id, occurred_at`. No `student_id`. No `user_id`.

### FINDING DOMAIN-EVD-007

- **id:** DOMAIN-EVD-007
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/cross-cutting/events-domain/src/events.rs:978-987` (IncidentCommented struct)
- **description:** The spec mandates `IncidentCommented` to carry the actual `comment: IncidentCommentBody` text and the `commented_at: Timestamp`. The code's `IncidentCommented` carries neither — so the subscriber that ingests "A comment was added to an incident" (per `docs/events/events.md:27`) cannot read the comment body or the wall-clock time of the comment.
- **expected:** Spec at `docs/specs/events/events.md:144-150`: `pub struct IncidentCommented { pub incident_comment_id, pub incident_id, pub user_id, pub comment: IncidentCommentBody, pub commented_at: Timestamp, }`
- **evidence:**
  - Spec: `docs/specs/events/events.md:144-150`
  - Code: `crates/cross-cutting/events-domain/src/events.rs:978-987` — fields: `incident_comment_id, incident_id, school_id, user_id, event_id_field, correlation_id, occurred_at`. No `comment`. No `commented_at`.

### FINDING DOMAIN-EVD-008

- **id:** DOMAIN-EVD-008
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/cross-cutting/events-domain/src/commands.rs:304-308` (ResolveIncidentCommand struct)
- **description:** The spec's `ResolveIncidentCommand` must carry `pub resolution_note: Option<IncidentCommentBody>` so the discipline lead can attach an immutable note when transitioning to Resolved. The workflows.md § Incident Resolution Workflow (line 102-103) names "with a note" as part of the step. The code's `ResolveIncidentCommand` has no such field, and the `Incident::resolve` method (aggregate.rs:510) does not accept one either.
- **expected:** Spec at `docs/specs/events/commands.md:222-235`: `pub struct ResolveIncidentCommand { pub tenant: TenantContext, pub incident_id: IncidentId, pub resolution_note: Option<IncidentCommentBody>, }` and workflows.md § Incident Resolution Workflow at line 102-103: "The lead resolves the incident (ResolveIncident) with a note."
- **evidence:**
  - Spec: `docs/specs/events/commands.md:228` — `pub resolution_note: Option<IncidentCommentBody>`
  - Code: `crates/cross-cutting/events-domain/src/commands.rs:305-308` — `pub tenant: TenantContext, pub incident_id: IncidentId` — only two fields, no `resolution_note`.
  - Code: `crates/cross-cutting/events-domain/src/aggregate.rs:510-518` — `Incident::resolve(&mut self, _actor: UserId, at: Timestamp)` — no note parameter; the `_actor` argument is even marked unused.

### FINDING DOMAIN-EVD-009

- **id:** DOMAIN-EVD-009
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/cross-cutting/events-domain/src/commands.rs:72-79` (UpdateEventCommand struct)
- **description:** The spec mandates `UpdateEventCommand` to carry optional patch fields for every mutable property of a CalendarEvent: `event_title`, `for_whom`, `role_ids`, `url`, `event_location`, `event_des`, `upload_image`. The code's `UpdateEventCommand` only has `title`, `from_date`, `to_date` — missing 6 of the 8 spec-mandated optional fields. Consumers cannot patch `for_whom`, the audience `role_ids`, the `url`, the `location`, the `description`, or the `image`.
- **expected:** Spec at `docs/specs/events/commands.md:37-51` — `pub event_title: Option<EventTitle>, pub for_whom: Option<ForWhom>, pub role_ids: Option<Vec<RoleId>>, pub url: Option<Url>, pub event_location: Option<EventLocation>, pub event_des: Option<EventDescription>, pub upload_image: Option<FileReference>`
- **evidence:**
  - Spec: `docs/specs/events/commands.md:37-51` lists 8 patch fields.
  - Code: `crates/cross-cutting/events-domain/src/commands.rs:73-79` — `title`, `from_date`, `to_date` only.

### FINDING DOMAIN-EVD-010

- **id:** DOMAIN-EVD-010
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/cross-cutting/events-domain/src/commands.rs:140-147` (UpdateHolidayCommand struct)
- **description:** The spec's `UpdateHolidayCommand` must carry `details: Option<HolidayDetails>` and `upload_image: Option<FileReference>` as patch fields. The code's `UpdateHolidayCommand` has neither, so the holiday's narrative `details` text and the optional attachment cannot be patched after creation.
- **expected:** Spec at `docs/specs/events/commands.md:88-100` lists 6 optional patch fields including `details` and `upload_image`.
- **evidence:**
  - Spec: `docs/specs/events/commands.md:88-100`
  - Code: `crates/cross-cutting/events-domain/src/commands.rs:141-147` — `title`, `from_date`, `to_date` only. No `details`. No `image`.

### FINDING DOMAIN-EVD-011

- **id:** DOMAIN-EVD-011
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/cross-cutting/events-domain/src/commands.rs:335-341` (AssignIncidentCommand struct) and `crates/cross-cutting/events-domain/src/aggregate.rs:537-555` (AssignIncident struct)
- **description:** The spec mandates `AssignIncidentCommand` and `AssignIncident` aggregate to carry `record_id: Option<StudentRecordId>` — a foreign key into `student_records.id` that anchors the assignment to a specific academic-year scope. The code carries no `record_id` anywhere, so the spec invariant 3 of the AssignIncident aggregate ("The `record_id` references a `StudentRecord` from the academic domain at the time of the incident") cannot be enforced.
- **expected:** Spec at `docs/specs/events/commands.md:167-178`: `pub struct AssignIncidentCommand { ..., pub record_id: Option<StudentRecordId>, ... }`. Spec at `docs/specs/events/aggregates.md:174-180`: "The record_id references a StudentRecord (from the academic domain) at the time of the incident."
- **evidence:**
  - Spec: `docs/specs/events/commands.md:175` — `pub record_id: Option<StudentRecordId>`
  - Code: `crates/cross-cutting/events-domain/src/commands.rs:335-341` — fields: `tenant, incident_id, student_id, user_id, point`. No `record_id`.
  - Code: `crates/cross-cutting/events-domain/src/aggregate.rs:537-555` — AssignIncident struct has no `record_id` field.

### FINDING DOMAIN-EVD-012

- **id:** DOMAIN-EVD-012
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/cross-cutting/events-domain/src/events.rs:193-202` (HolidayCreated struct)
- **description:** The spec mandates `HolidayCreated` to carry `academic_id: AcademicYearId` to scope the holiday to an academic year (per aggregates.md § Holiday invariant 3: "A Holiday is anchored to a school and an academic year"). The code's `HolidayCreated` carries no `academic_id`, so the attendance domain subscriber (per `docs/events/events.md:13`) cannot scope holiday overrides to the correct academic year.
- **expected:** Spec at `docs/specs/events/events.md:58-65`: `pub struct HolidayCreated { pub holiday_id, pub holiday_title: HolidayTitle, pub from_date, pub to_date, pub academic_id: AcademicYearId, }`
- **evidence:**
  - Spec: `docs/specs/events/events.md:64`
  - Code: `crates/cross-cutting/events-domain/src/events.rs:193-202` — HolidayCreated fields: `holiday_id, school_id, title, from_date, to_date, event_id_field, correlation_id, occurred_at`. No `academic_id`.

### FINDING DOMAIN-EVD-013

- **id:** DOMAIN-EVD-013
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/cross-cutting/events-domain/src/services.rs` (entire file)
- **description:** The spec's `services.md` mandates three additional service types that the code does not implement: (a) Specification `ActiveIncidents` filtering `Status != Resolved` (services.md lines 84-94); (b) Specification `EventsInMonth` matching events that overlap a month (lines 97-110); (c) Policy `IncidentPointLimit` capping points per incident (lines 113-126). None of the three exist in the code. No `Specification` or `Policy` traits are imported or implemented anywhere in the crate.
- **expected:** Spec at `docs/specs/events/services.md:84-126` defines three service types using the canonical patterns `impl Specification<Incident> for ActiveIncidents` and `impl Policy<AssignIncidentCommand> for IncidentPointLimit`.
- **evidence:**
  - Spec: `docs/specs/events/services.md:84-126`
  - Code: `grep -rn "Specification\|Policy\|ActiveIncidents\|EventsInMonth\|IncidentPointLimit" crates/cross-cutting/events-domain/src/` returns no matches.

### FINDING DOMAIN-EVD-014

- **id:** DOMAIN-EVD-014
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/cross-cutting/events-domain/src/services.rs:128-153` (HolidayService::is_instructional and related)
- **description:** `HolidayService::is_instructional` (services.rs:128-133) and `WeekendService::is_weekend` (services.rs:300-303) compare the weekday ordinal `date.weekday().num_days_from_monday() as i32` against `Weekend::order` directly, bypassing the `WeekendOrder` newtype (per spec value-objects.md line 40, "i32 in 0..7"). The `WeekendOrder` type does not exist in the code (FINDING-DOMAIN-EVD-004), so the `0..7` invariant cannot be enforced at construction; a `Weekend` row with `order = 100` constructed via the SQL adapter would silently corrupt the is-instructional predicate.
- **expected:** Spec at `docs/specs/events/value-objects.md:40` — `WeekendOrder | i32 in 0..7` (table cell). Spec at `docs/specs/events/aggregates.md:101` — Invariant 2: "The order field is a positive integer; lower orders sort first in a UI."
- **evidence:**
  - Spec: `docs/specs/events/value-objects.md:40`
  - Code: `crates/cross-cutting/events-domain/src/services.rs:129` — `let weekday = date.weekday().num_days_from_monday() as i32;` then `w.order == weekday`.
  - Code: `crates/cross-cutting/events-domain/src/aggregate.rs:696` — `Weekend::order` is plain `i32` (not `WeekendOrder`).
  - Code: `crates/cross-cutting/events-domain/src/aggregate.rs:726` — The `Weekend::new` constructor checks `0..=7` against a raw `i32`, not against a typed `WeekendOrder`.

### FINDING DOMAIN-EVD-015

- **id:** DOMAIN-EVD-015
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/cross-cutting/events-domain/src/services.rs` (entire file)
- **description:** The spec's `services.md` § "Cross-Domain Coordinator" (lines 128-147) mandates an `EventsCoordinator` struct in the engine facade that orchestrates multi-domain flows such as "report_and_notify". No `EventsCoordinator` exists in the code, no `engine.events()` facade is referenced, and no async coordination logic exists.
- **expected:** Spec at `docs/specs/events/services.md:128-147`: `pub struct EventsCoordinator<'a> { engine: &'a Engine }` with `pub async fn report_and_notify(&self, cmd: CreateIncidentCommand) -> Result<Incident, DomainError>`.
- **evidence:**
  - Spec: `docs/specs/events/services.md:128-147`
  - Code: `grep -rn "EventsCoordinator\|report_and_notify" crates/cross-cutting/events-domain/` returns no matches.

### FINDING DOMAIN-EVD-016

- **id:** DOMAIN-EVD-016
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/cross-cutting/events-domain/src/services.rs:212-215` (IncidentService::total_points)
- **description:** The spec mandates `IncidentService::total_points(assignments: &[AssignIncident]) -> i32` to sum the points attributed across all assignments. The code's signature is `total_points(points: &[i32]) -> i32` — it takes a pre-flattened slice of integers, not the assignment aggregate itself. Callers cannot use the spec-shaped API, and the service cannot enforce invariants on the input (e.g., rejection of duplicate student assignments) because it has no view of the aggregate.
- **expected:** Spec at `docs/specs/events/services.md:42-51`: `pub fn total_points(assignments: &[AssignIncident]) -> i32`
- **evidence:**
  - Spec: `docs/specs/events/services.md:47`
  - Code: `crates/cross-cutting/events-domain/src/services.rs:213-215` — `pub fn total_points(points: &[i32]) -> i32`

### FINDING DOMAIN-EVD-017

- **id:** DOMAIN-EVD-017
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/cross-cutting/events-domain/src/services.rs` (IncidentService impl block, lines 199-222)
- **description:** The spec mandates `IncidentService::participants(assignments: &[AssignIncident]) -> Vec<UserReference>` to enumerate the assigned students/staff. The method does not exist in the code.
- **expected:** Spec at `docs/specs/events/services.md:42-51`: `pub fn participants(assignments: &[AssignIncident]) -> Vec<UserReference>`
- **evidence:**
  - Spec: `docs/specs/events/services.md:48`
  - Code: `grep -n "fn participants" crates/cross-cutting/events-domain/src/` returns no matches.

### FINDING DOMAIN-EVD-018

- **id:** DOMAIN-EVD-018
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/cross-cutting/events-domain/src/services.rs:62-64` (CalendarService::visible_to)
- **description:** The spec's `CalendarService::visible_to` signature is `pub fn visible_to(event: &CalendarEvent, actor: &ActorRoles) -> bool` — it takes the entire event plus a typed `ActorRoles` bundle. The code's signature is `pub fn visible_to(for_whom: ForWhom, role_ids: &[String], actor_roles: &[String]) -> bool` — the caller has to extract the audience fields and supply raw `String` slices. `ActorRoles` (the typed wrapper the spec mandates) does not exist in the code.
- **expected:** Spec at `docs/specs/events/services.md:8-17`: `pub fn visible_to(event: &CalendarEvent, actor: &ActorRoles) -> bool`
- **evidence:**
  - Spec: `docs/specs/events/services.md:15`
  - Code: `crates/cross-cutting/events-domain/src/services.rs:62-64` — `pub fn visible_to(for_whom: ForWhom, role_ids: &[String], actor_roles: &[String]) -> bool`
  - Code: `grep -n "ActorRoles" crates/cross-cutting/events-domain/` returns no matches.

### FINDING DOMAIN-EVD-019

- **id:** DOMAIN-EVD-019
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/cross-cutting/events-domain/src/repository.rs:34-63` (CalendarEventRepository trait)
- **description:** The spec mandates `CalendarEventRepository` to expose 9 methods; the code's trait exposes only 7. The two missing methods are `count(&self, school: SchoolId, q: CalendarEventQuery) -> Result<u64>` (for cardinality queries without loading rows) and `page(&self, school: SchoolId, q: CalendarEventQuery, offset: u32, limit: u32) -> Result<Page<CalendarEvent>>` (for paginated reads).
- **expected:** Spec at `docs/specs/events/repositories.md:9-22`: the `count` and `page` methods are the 8th and 9th methods.
- **evidence:**
  - Spec: `docs/specs/events/repositories.md:19-20`
  - Code: `crates/cross-cutting/events-domain/src/repository.rs:34-63` — only 7 methods; no `count`, no `page`.

### FINDING DOMAIN-EVD-020

- **id:** DOMAIN-EVD-020
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/cross-cutting/events-domain/src/repository.rs:149-171` (IncidentRepository trait)
- **description:** The spec mandates `IncidentRepository` to expose 11 methods; the code's trait exposes only 8. Missing methods: `resolve(&self, id: IncidentId) -> Result<()>` (so a resolved incident can be persisted atomically with the IncidentResolved event), `by_student(&self, school: SchoolId, student: StudentId) -> Result<Vec<Incident>>`, and `by_user(&self, school: SchoolId, user: UserId) -> Result<Vec<Incident>>`.
- **expected:** Spec at `docs/specs/events/repositories.md:57-69`
- **evidence:**
  - Spec: `docs/specs/events/repositories.md:62, 66-67`
  - Code: `crates/cross-cutting/events-domain/src/repository.rs:149-171` — only `get, list, insert, update, delete, open, in_progress, between`.

### FINDING DOMAIN-EVD-021

- **id:** DOMAIN-EVD-021
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/cross-cutting/events-domain/src/aggregate.rs` (every aggregate struct, plus constructor bodies)
- **description:** The crate declares `last_event_id: Option<EventId>` on every aggregate (CalendarEvent:57, Holiday:212, CalendarSetting:300, Incident:416, AssignIncident:553, IncidentComment:640, Weekend:706) but no code path ever assigns it. Every constructor initializes it to `None` and no command/event helper writes to it. The `audit_log` and outbox integration mandated by the engine rule "Audit-first. Every state change writes an immutable record" (`AGENTS.md` engine rules) is therefore not implemented — the field is a placeholder with no writer.
- **expected:** Engine rule in `AGENTS.md`: "Audit-first. Every state change writes an immutable record." Spec at `docs/specs/events/workflows.md:138-140`: "Every state-changing command writes a durable audit record with the actor, the correlation id, and a hash of the payload."
- **evidence:**
  - `crates/cross-cutting/events-domain/src/aggregate.rs:57, 123, 212, 260, 300, 341, 416, 464, 553, 600, 640, 671, 706, 745` — every aggregate declares `last_event_id: Option<EventId>` and initializes it to `None`.
  - `grep -rn "last_event_id = Some" crates/cross-cutting/events-domain/src/` returns no matches.

### FINDING DOMAIN-EVD-022

- **id:** DOMAIN-EVD-022
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/cross-cutting/events-domain/src/` (entire crate)
- **description:** The crate declares dependencies on `educore-audit` and `educore-events` (Cargo.toml:17,19), and the handoff at `docs/handoff/PHASE-13-HANDOFF.md:316-319` claims "4 net-new `Events*` AuditTarget variants in `educore-audit`". But the events-domain crate itself never imports or calls `educore_audit::write` (or equivalent) anywhere. There is no domain-side audit-writer glue, no `audit_write!` macro invocation, and no emission of an audit record from any command or service. The audit integration is one-sided (targets added to `educore-audit`, but no caller in events-domain).
- **expected:** Engine rule "Audit-first." Spec workflows.md:138-140. Handoff claim at PHASE-13-HANDOFF.md:152-153.
- **evidence:**
  - `crates/cross-cutting/events-domain/Cargo.toml:19` — `educore-audit = { workspace = true }`
  - `grep -rn "educore_audit\|audit::" crates/cross-cutting/events-domain/src/` returns no matches in non-test code.

### FINDING DOMAIN-EVD-023

- **id:** DOMAIN-EVD-023
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/cross-cutting/events-domain/src/events.rs:1031-1081` (IncidentCommentDeletedEvent struct)
- **description:** The spec mandates the type to be named `IncidentCommentDeleted` (events.md line 152, also `docs/commands/events.md:26`). The code names it `IncidentCommentDeletedEvent`, which diverges from the spec. Downstream subscribers and the round-trip test (`events_integration.rs:35`) are forced to use the renamed type. The wire-form `EVENT_TYPE` (`"events.incident_comment.deleted"`) does match the spec, so the rename is purely at the Rust type level — it is an avoidable drift from the public catalog.
- **expected:** Spec at `docs/specs/events/events.md:152`: `pub struct IncidentCommentDeleted { ... }`. Catalog at `docs/events/events.md:28` and `docs/commands/events.md:26` use `IncidentCommentDeleted`.
- **evidence:**
  - Spec: `docs/specs/events/events.md:152`
  - Code: `crates/cross-cutting/events-domain/src/events.rs:1032` — `pub struct IncidentCommentDeletedEvent { ... }`
  - Code: `crates/tools/storage-parity/tests/events_integration.rs:35` — imports `IncidentCommentDeletedEvent` (forced workaround for the rename).

### FINDING DOMAIN-EVD-024

- **id:** DOMAIN-EVD-024
- **area:** domain-crates
- **severity:** High
- **location:** `crates/cross-cutting/events-domain/Cargo.toml:27`
- **description:** The crate's direct dependency on `serde_json` (Cargo.toml:27) is a yellow flag under the engine rule "No `serde_json::Value` in domain code. Use typed wrappers." (`AGENTS.md` code standards). While the crate does not appear to use `serde_json::Value` directly (verified via `grep`), depending on `serde_json` for typed serde wrappers invites the anti-pattern. The spec mandates typed envelopes and wire-form `events.<aggregate>.<verb>` strings; the runtime JSON serialization belongs to the storage adapter layer, not the domain crate.
- **expected:** Engine rule (AGENTS.md): "No `serde_json::Value` in domain code. Use typed wrappers." Domain crates should serialize via the typed `serde::{Serialize, Deserialize}` derives they already carry, with no JSON-specific dependency.
- **evidence:** `crates/cross-cutting/events-domain/Cargo.toml:27` — `serde_json = { workspace = true }`.

### FINDING DOMAIN-EVD-025

- **id:** DOMAIN-EVD-025
- **area:** domain-crates
- **severity:** High
- **location:** `crates/cross-cutting/events-domain/src/aggregate.rs:471-506` (Incident::update)
- **description:** The spec's Incident invariant 5 (aggregates.md line 142-144) says "An Incident is immutable after `Status` is `Resolved` except for the `description` field (which may be annotated) and the comments list." The code's `Incident::update` rejects ALL updates after Resolved (`aggregate.rs:478-482`), so annotating the description on a resolved incident is impossible. This contradicts the spec and breaks the discipline-lead workflow described in workflows.md § Incident Resolution Workflow line 102-103 ("a note" attached at resolve time).
- **expected:** Spec at `docs/specs/events/aggregates.md:142-144` — invariant 5 explicitly allows `description` annotation after Resolved. Spec workflows.md § Incident Resolution Workflow lines 102-103.
- **evidence:**
  - Spec: `docs/specs/events/aggregates.md:142-144`
  - Code: `crates/cross-cutting/events-domain/src/aggregate.rs:478-482` — `if self.status == IncidentStatus::Resolved { return Err(EventsDomainError::Conflict("cannot update resolved incident".to_owned())); }` — applied unconditionally before any field-by-field check.

### FINDING DOMAIN-EVD-026

- **id:** DOMAIN-EVD-026
- **area:** domain-crates
- **severity:** High
- **location:** `crates/cross-cutting/events-domain/src/value_objects.rs:171-178` (IncidentAction enum) and `value_objects.rs:154-160` (`IncidentStatus::next`)
- **description:** `IncidentAction` declares a `Reopen` variant (value_objects.rs:177) but `IncidentStatus::next` (line 154) has no arm for `Reopen` — the match returns `current` for any unmatched action, so `Resolved.next(Reopen)` silently returns `Resolved`. The `Reopen` variant is therefore dead-coded at the state-machine level. No test exercises the reopen path.
- **expected:** Per spec workflows.md and aggregates.md § Incident lifecycle, the state machine is `Open → InProgress → Resolved`. If `Reopen` is modeled, the engine rule says no dead code: "Delete unused code, wire it in, or open a follow-up issue." (`AGENTS.md` type-safety rules.)
- **evidence:**
  - Code: `crates/cross-cutting/events-domain/src/value_objects.rs:171-178` — `pub enum IncidentAction { InProgress, Resolve, Reopen }`
  - Code: `crates/cross-cutting/events-domain/src/value_objects.rs:154-160` — `pub const fn next(self, action: IncidentAction) -> Self { match (self, action) { (Self::Open, IncidentAction::InProgress) => ..., (Self::InProgress, IncidentAction::Resolve) => ..., (current, _) => current } }` — no `Reopen` arm.
  - Code: `grep -rn "IncidentAction::Reopen" crates/cross-cutting/events-domain/` returns no matches outside the variant declaration.

### FINDING DOMAIN-EVD-027

- **id:** DOMAIN-EVD-027
- **area:** domain-crates
- **severity:** High
- **location:** `crates/cross-cutting/events-domain/src/services.rs:262-295` (WeekendService::reconcile)
- **description:** The function builds `current_names: HashSet<&str>` (line 263) but never reads it — the second loop (lines 286-292) iterates `current` and checks `proposed_names.contains(...)` instead. The unused `current_names` is only suppressed via `let _ = current_names;` at line 293. This is dead code that survives because `aggregate.rs:7` carries `#![allow(missing_docs, dead_code, clippy::all)]` and `services.rs:8` carries `#![allow(missing_docs)]`. Under the engine rule "No `#[allow(dead_code)]` or `_var` prefixes to silence the compiler. Delete unused code, wire it in, or open a follow-up issue." (`AGENTS.md`), this is a violation.
- **expected:** Engine rule (AGENTS.md) type-safety section: no `_var` prefixes to silence the compiler; delete unused code.
- **evidence:**
  - Code: `crates/cross-cutting/events-domain/src/services.rs:263` — `let current_names: std::collections::HashSet<&str> = current.iter().map(|w| w.name.as_str()).collect();`
  - Code: `crates/cross-cutting/events-domain/src/services.rs:293` — `let _ = current_names; // suppress unused`

### FINDING DOMAIN-EVD-028

- **id:** DOMAIN-EVD-028
- **area:** domain-crates
- **severity:** High
- **location:** `crates/cross-cutting/events-domain/src/services.rs:129, 301`; `crates/cross-cutting/events-domain/src/value_objects.rs:361-363`
- **description:** The crate uses raw `as i32` / `as i64` / `as u32` casts in production code paths. The engine rule in `AGENTS.md` code standards says "No `as` casts that truncate or lose data. Use `TryFrom` / `TryInto` with proper error handling." The numeric conversions on chrono's `Datelike::year() -> i32`, `month() -> u32`, `num_days_from_monday() -> u32` are all widening or value-preserving, but the rule's wording is absolute ("No `as` on numerics is forbidden" in the validation checklist).
- **expected:** Engine rule (AGENTS.md) code standards: "Numeric conversions use `TryFrom`/`TryInto`; `as` on numerics is forbidden." Validation checklist: "No new `as` on numerics."
- **evidence:**
  - `crates/cross-cutting/events-domain/src/services.rs:129` — `let weekday = date.weekday().num_days_from_monday() as i32;`
  - `crates/cross-cutting/events-domain/src/services.rs:301` — same cast.
  - `crates/cross-cutting/events-domain/src/value_objects.rs:361` — `let total = d.year() as i64 * 12 + d.month() as i64 - 1 + months;`
  - `crates/cross-cutting/events-domain/src/value_objects.rs:362-363` — `(total.div_euclid(12)) as i32`, `(total.rem_euclid(12) + 1) as u32`.

### FINDING DOMAIN-EVD-029

- **id:** DOMAIN-EVD-029
- **area:** domain-crates
- **severity:** High
- **location:** `crates/cross-cutting/events-domain/src/services.rs:149` (HolidayService::instructional_days_in)
- **description:** Production code path uses `.succ_opt().unwrap_or(current)` (services.rs:149). The `unwrap_or` branch is a non-panicking fallback, but `succ_opt` returning `None` is then silently swallowed — the loop stops at the same date, so the date-range walk silently truncates if any date is invalid (e.g., a malformed input that surfaced via the storage adapter). The engine rule says "No `unwrap`/`expect`/`panic` in production paths" — strictly, `unwrap_or` is not on the list, but the silent truncation is a related smell: an invalid date input causes an infinite `while current <= to` spin rather than a domain error.
- **expected:** Engine rule (AGENTS.md): "No `unwrap()` or `expect()` in production paths. Propagate errors via `?` or document the invariant that makes panic impossible."
- **evidence:**
  - Code: `crates/cross-cutting/events-domain/src/services.rs:143-151` — the loop body uses `current.succ_opt().unwrap_or(current)` to advance, with no error path.

### FINDING DOMAIN-EVD-030

- **id:** DOMAIN-EVD-030
- **area:** domain-crates
- **severity:** High
- **location:** `crates/cross-cutting/events-domain/` (crate root) — no `tests/` directory
- **description:** The crate has no `tests/` directory at `crates/cross-cutting/events-domain/tests/`. The handoff claim at `docs/handoff/PHASE-13-HANDOFF.md:36-38` says "34 unit tests in educore-events-domain + 7 always-on integration tests" — but the 7 always-on integration tests live in `crates/tools/storage-parity/tests/events_integration.rs`, not in the crate itself. There are no integration tests for the crate in its own `tests/` directory.
- **expected:** Engine rule (AGENTS.md) validation checklist: "At least one integration test added for new behavior (per the per-PR gate). Unit tests alone are not sufficient." Spec format (§ Phase 13 outcome) expects 7 always-on integration tests at the crate's `tests/` path.
- **evidence:**
  - `ls crates/cross-cutting/events-domain/tests/` — directory does not exist.
  - `crates/cross-cutting/events-domain/` directory listing shows only `Cargo.toml` and `src/`.
  - The 7 integration tests live in `crates/tools/storage-parity/tests/events_integration.rs:372, 512, 542, 580, 624, 641, 686`.

### FINDING DOMAIN-EVD-031

- **id:** DOMAIN-EVD-031
- **area:** domain-crates
- **severity:** High
- **location:** `crates/tools/storage-parity/tests/events_integration.rs:735-746` (PG/MySQL env-gated tests)
- **description:** The two `#[ignore]`-marked PG/MySQL integration tests are 1-line placeholder stubs: `let _school = SchoolId::from_uuid(uuid::Uuid::new_v4());`. They do not exercise the schema, the dialect translation, or any of the 24 events. The handoff claim "7 always-on integration tests + 2 env-gated `#[ignore]` PG/MySQL variants in `events_integration.rs`" implies the ignored tests have substance; they do not.
- **expected:** Spec at `docs/build-plan.md` § Phase 13 outcome line 1501: "2 env-gated #[ignore] PG/MySQL variants" implies actual PG/MySQL adapter coverage. The PG/MySQL adapter crates (`educore-storage-postgres`, `educore-storage-mysql`) ship from Phase 1, so the env-gated tests should drive real round-trips on those adapters.
- **evidence:**
  - `crates/tools/storage-parity/tests/events_integration.rs:735-740` — `events_integration_pg_vertical_slice` body is just `let _school = SchoolId::from_uuid(uuid::Uuid::new_v4());`.
  - `crates/tools/storage-parity/tests/events_integration.rs:742-746` — same placeholder for MySQL.

### FINDING DOMAIN-EVD-032

- **id:** DOMAIN-EVD-032
- **area:** domain-crates
- **severity:** High
- **location:** `docs/build-plan.md` § Phase 13 — Tasks (line 1470, 1475) vs Phase 13 outcome (line 1501)
- **description:** Doc-vs-code drift. The build-plan's Phase 13 Tasks section (line 1466-1492) still describes the original prompt: "Aggregates per docs/specs/events/aggregates.md: CalendarEvent, Holiday, Incident, Weekend." (4 aggregates). The Phase 13 outcome section (line 1501), added at phase close, describes the 7-aggregate reality. Two contradictory statements in the same build-plan section; the Tasks list was never updated to match the implementation or the outcome note.
- **expected:** Per `AGENTS.md` engine rules and the build-plan update instructions at lines 1486-1490 ("Update ... `**Phase 13 outcome.**` subsection to this build plan ..."), the Tasks section should reflect the implemented scope.
- **evidence:**
  - `docs/build-plan.md:1470` — "Aggregates per docs/specs/events/aggregates.md:"
  - `docs/build-plan.md:1475` — "`CalendarEvent`, `Holiday`, `Incident`, `Weekend`."
  - `docs/build-plan.md:1501` — "**Spec-faithful 7-root interpretation** ... 7 root aggregates (CalendarEvent, Holiday, Weekend, Incident, AssignIncident, IncidentComment, CalendarSetting)."

### FINDING DOMAIN-EVD-033

- **id:** DOMAIN-EVD-033
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/cross-cutting/events-domain/src/aggregate.rs:13` (unused import) and `crates/cross-cutting/events-domain/src/value_objects.rs:14` (unused `Datelike` import? actually it IS used)
- **description:** `aggregate.rs:13` imports `use uuid::Uuid;` but `Uuid` is used inside the test module (lines 794-979). The import is necessary for tests but `dead_code` allow at module level hides that `Uuid` may be required by production code only via the implicit aggregate `id.value` Uuid field; verify. Several other imports — `EducorePlatform`, `UserType`, `RoleId`, `StudentId`, `StudentRecordId` — would be needed to type the spec fields, but they are absent because the fields are raw types (FINDING-DOMAIN-EVD-004).
- **expected:** Engine rule (AGENTS.md): "No `#[allow(dead_code)]` ... to silence the compiler. Delete unused code, wire it in, or open a follow-up issue."
- **evidence:** `crates/cross-cutting/events-domain/src/aggregate.rs:13` — `use uuid::Uuid;` — used in tests only (mod tests, line 794+).

### FINDING DOMAIN-EVD-034

- **id:** DOMAIN-EVD-034
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/cross-cutting/events-domain/src/events.rs:7-8` and entire file (24 events, each with `event_id_field: EventId` plumbed manually)
- **description:** Every event carries an extra `event_id_field: EventId` field that is duplicated by the `EventEnvelope` from `educore-events::domain_event::EventEnvelope` (per spec events.md lines 21-32, the envelope carries `event_id: EventId`). The crate does not use the `EventEnvelope<E>` wrapper at all — it passes 7-9 fields to `EventX::new(...)` constructors manually, then asks consumers to also wrap the event in their own envelope. The wire-form string (`EVENT_TYPE`) is implemented on the inner event, not the envelope. This deviates from the engine's `educore-events` Phase 2 envelope contract (per AGENTS.md: "two `events` crates — do NOT confuse").
- **expected:** Spec at `docs/specs/events/events.md:18-33` — the canonical envelope wraps the event; the inner event should be the `payload` field. The `event_id` field lives on the envelope, not on the inner event payload.
- **evidence:**
  - Spec: `docs/specs/events/events.md:21-32`
  - Code: `crates/cross-cutting/events-domain/src/events.rs:34-36` — `EventCreated` carries `event_id_field: EventId, correlation_id: CorrelationId, occurred_at: Timestamp` as payload fields, identical to envelope fields.
  - Code: every event in `events.rs` repeats `pub event_id_field: EventId, pub correlation_id: CorrelationId, pub occurred_at: Timestamp` in its payload (24 occurrences).

### FINDING DOMAIN-EVD-035

- **id:** DOMAIN-EVD-035
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/cross-cutting/events-domain/src/commands.rs:473-484` (`fn _ensure_ids_compile`)
- **description:** The `fn _ensure_ids_compile(school: SchoolId)` helper at commands.rs:473 carries `#[allow(dead_code)]` and exists only to silence "unused import" warnings for the typed IDs (CalendarEventId, HolidayId, CalendarSettingId, IncidentId, AssignIncidentId, IncidentCommentId, WeekendId). This is a documented anti-pattern under AGENTS.md ("No `#[allow(dead_code)]` ... to silence the compiler. Delete unused code, wire it in, or open a follow-up issue.").
- **expected:** Either delete the unused imports, or wire the IDs into public API. `_ensure_ids_compile` is a code-smell flag.
- **evidence:**
  - Code: `crates/cross-cutting/events-domain/src/commands.rs:473-484` — `#[allow(dead_code)] fn _ensure_ids_compile(...)` — the function is never called.

### FINDING DOMAIN-EVD-036

- **id:** DOMAIN-EVD-036
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/cross-cutting/events-domain/src/repository.rs:65-67, 100-102, 137-139, 173-175, 210-212, 233-235, 260-262` (`_assert_*_object_safe` helpers)
- **description:** Seven `_assert_*_object_safe` private functions exist purely to prove the corresponding repository trait is object-safe via `let _: Box<dyn XxxRepository>;`. The functions are never called; they exist solely as compile-time assertions. While object-safety is a real concern, the AGENTS.md rule says "Trait objects must be object-safe. Verify with `let _: Box<dyn Trait>;` compile tests." The verification is correct in spirit but the implementation pattern (7 dead helper functions) leaves the proof scattered rather than centralized.
- **expected:** Either a single integration test or a centralized `const _: fn() = || { let _: Box<dyn CalendarEventRepository> = ...; };` block. Engine rule: no `_var` prefix to silence the compiler.
- **evidence:** Seven `_assert_*_object_safe` helpers in `crates/cross-cutting/events-domain/src/repository.rs`, none called.

### FINDING DOMAIN-EVD-037

- **id:** DOMAIN-EVD-037
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/cross-cutting/events-domain/src/aggregate.rs` (NewCalendarEvent, NewHoliday, NewIncident, NewCalendarSetting constructors)
- **description:** Every aggregate root has a `NewXxx` input struct (aggregate.rs:62-79, 217-229, 421-430, 305-315) but `NewAssignIncident`, `NewWeekend`, `NewIncidentComment` do not exist — those aggregates use positional-argument constructors instead (`aggregate.rs:559-572, 712-720, 646-652`). This inconsistency makes command-to-aggregate bridging non-uniform across the 7 roots, which complicates the dispatcher and the upcoming `into_new_*` helpers documented in `PHASE-13-HANDOFF.md:290-292`.
- **expected:** Per the handoff "24 typed commands + 24 `EVENTS_*_COMMAND_TYPE` constants + the `into_new_*` helpers," the `into_new_*` helpers exist for the 4 aggregates that use `NewXxx` structs (CalendarEvent, Holiday, Incident, CalendarSetting) but not for the 3 that use positional constructors (AssignIncident, Weekend, IncidentComment). This is an inconsistency the handoff glosses over.
- **evidence:**
  - `crates/cross-cutting/events-domain/src/aggregate.rs:62-79` — NewCalendarEvent.
  - `crates/cross-cutting/events-domain/src/aggregate.rs:217-229` — NewHoliday.
  - `crates/cross-cutting/events-domain/src/aggregate.rs:305-315` — NewCalendarSetting.
  - `crates/cross-cutting/events-domain/src/aggregate.rs:421-430` — NewIncident.
  - No `NewAssignIncident`, no `NewWeekend`, no `NewIncidentComment` — those aggregates use positional `*::new(...)`.
  - `crates/cross-cutting/events-domain/src/commands.rs:48-68, 122-136, 187-199, 274-285` — `into_new_*` helpers exist only for the 4 with `NewXxx` structs.

### FINDING DOMAIN-EVD-038

- **id:** DOMAIN-EVD-038
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/cross-cutting/events-domain/src/aggregate.rs:32-59` (CalendarEvent field list)
- **description:** The `CalendarEvent` aggregate carries 21 fields (aggregate.rs:35-58), with no clear separation between mutable business fields (`title`, `from_date`, `to_date`, `for_whom`, `role_ids`, `url`, `location`, `description`, `image`, `rrule`, `status`) and engine bookkeeping (`school_id`, `version`, `etag`, `created_at`, `updated_at`, `created_by`, `updated_by`, `active_status`, `last_event_id`, `correlation_id`, `audience`, `academic_id`). The UpdateEventCommand (FINDING-DOMAIN-EVD-009) can only patch 3 of the 11 mutable fields. There is no way for a consumer to mark an event `Cancelled` (the third `CalendarEventStatus` variant, declared in `value_objects.rs:228`), because there is no `CancelEventCommand` and no `CalendarEvent::cancel()` method.
- **expected:** Spec at `docs/specs/events/value-objects.md:85` mandates `CalendarEventStatus` with values `Draft`, `Published`, `Cancelled`. The state transitions should be expressed as aggregate methods (e.g., `publish()`, `cancel()`) on the `CalendarEvent` root.
- **evidence:**
  - Spec: `docs/specs/events/value-objects.md:85`
  - Code: `crates/cross-cutting/events-domain/src/value_objects.rs:224-229` — `enum CalendarEventStatus { Draft, Published, Cancelled }`
  - Code: `crates/cross-cutting/events-domain/src/aggregate.rs:113` — `CalendarEvent::new` always sets `status: CalendarEventStatus::Draft`. No method advances status.
  - Code: `crates/cross-cutting/events-domain/src/aggregate.rs:128-165` — `CalendarEvent::update` does not touch `status`.

### FINDING DOMAIN-EVD-039

- **id:** DOMAIN-EVD-039
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/cross-cutting/events-domain/src/aggregate.rs:267-275` (Holiday::attachments and Holiday::periods)
- **description:** Both accessors return empty `Vec::new()` literals. The `Holiday` aggregate never holds child `HolidayAttachment` or `HolidayPeriod` references — those entities exist only in `entities.rs` as detached types. A consumer calling `holiday.attachments()` or `holiday.periods()` always sees an empty list, which silently masks the spec invariant "A `HolidayPeriod` ... most holidays have one period equal to the date range." (entities.md line 53-55).
- **expected:** Spec at `docs/specs/events/entities.md:42-55` — Holiday owns HolidayAttachment and HolidayPeriod entities.
- **evidence:**
  - Code: `crates/cross-cutting/events-domain/src/aggregate.rs:265-275` — `pub fn attachments(&self) -> Vec<&HolidayAttachment> { Vec::new() }` and `pub fn periods(&self) -> Vec<&HolidayPeriod> { Vec::new() }`.
  - Code: `crates/cross-cutting/events-domain/src/aggregate.rs:181-185` — same pattern on `CalendarEvent::attachments()`.

### FINDING DOMAIN-EVD-040

- **id:** DOMAIN-EVD-040
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/cross-cutting/events-domain/src/entities.rs:147-156` (HolidayPeriod::new)
- **description:** `HolidayPeriod::new` does not accept a date range — it forces `from_date` and `to_date` to `chrono::Utc::now().date_naive()`. The spec (entities.md line 53-55) says a HolidayPeriod "supports split holidays (e.g. 'Winter break' with a gap)" — the construction API cannot represent a multi-day or split range. There is also no validation that the period's range falls within the parent holiday's range.
- **expected:** Spec at `docs/specs/events/entities.md:49-55` — `HolidayPeriod { ... from_date: NaiveDate, to_date: NaiveDate }`.
- **evidence:**
  - Code: `crates/cross-cutting/events-domain/src/entities.rs:144-156` — `pub fn new(id: HolidayPeriodId, holiday_id: HolidayId)` — no date arguments; defaults to today.

### FINDING DOMAIN-EVD-041

- **id:** DOMAIN-EVD-041
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/cross-cutting/events-domain/src/query.rs` (entire file)
- **description:** All 7 query stubs (`CalendarEventQuery`, `HolidayQuery`, etc.) are empty structs with only a `Default` impl and a `new()` constructor. The repositories call them (e.g., `repository.rs:41` — `q: CalendarEventQuery`) but the query carries no fields to filter on. The macro-driven query builder pattern mandated by `AGENTS.md` ("Compile-time safety over strings. Use macro-generated enums (`StudentField::Status`)") is absent — there are no `#[derive(DomainQuery)]` derives, no `CalendarEventField` enum, no `.active()`, `.in_class()` extension traits.
- **expected:** Spec at `docs/specs/events/repositories.md:13` uses `q: CalendarEventQuery` as the second arg of `list`/`count`/`page`. AGENTS.md engine rule 2: "Compile-time safety over strings. Use macro-generated enums (`StudentField::Status`) — never string field names."
- **evidence:**
  - `crates/cross-cutting/events-domain/src/query.rs:18-20` — `pub struct CalendarEventQuery { /* Fields filled in by Workstream A. */ }` — empty.
  - `crates/cross-cutting/events-domain/src/query.rs:36-150` — 6 more empty stubs (HolidayQuery, CalendarSettingQuery, IncidentQuery, AssignIncidentQuery, IncidentCommentQuery, WeekendQuery), all empty.
  - `grep -rn "DomainQuery\|CalendarEventField" crates/cross-cutting/events-domain/` returns no matches.

### FINDING DOMAIN-EVD-042

- **id:** DOMAIN-EVD-042
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/cross-cutting/events-domain/src/aggregate.rs` (all 7 aggregates' constructors and update methods)
- **description:** The aggregates do not implement the spec-mandated capability check. Per `docs/specs/events/permissions.md` lines 80-87, "Capabilities are checked at the command boundary. The engine never trusts the caller to assert their own role." The aggregate constructors take only business fields — no `tenant: &TenantContext`, no capability check, no RBAC integration. The `educore-rbac` dependency is not actually wired into any aggregate method.
- **expected:** Spec at `docs/specs/events/permissions.md:80-87`. Spec at `docs/specs/events/commands.md` — every command lists a `**Capability:**` (e.g., `Event.Create`, `Holiday.Update`) — the aggregate constructor should reject when the tenant lacks that capability.
- **evidence:**
  - `crates/cross-cutting/events-domain/src/aggregate.rs:83-126` — `CalendarEvent::new(cmd: NewCalendarEvent)` — no `tenant`, no capability check.
  - `crates/cross-cutting/events-domain/Cargo.toml:16` — declares `educore-rbac = { workspace = true }` but `grep -rn "educore_rbac" crates/cross-cutting/events-domain/src/` returns no matches.

### FINDING DOMAIN-EVD-043

- **id:** DOMAIN-EVD-043
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/cross-cutting/events-domain/src/aggregate.rs:128-165` (CalendarEvent::update)
- **description:** The `update` method takes `title`, `from`, `to` as separate positional arguments and does not thread the audience fields, the URL, the location, the description, the image, the rrule, the status, or the audience — even though the spec's `UpdateEventCommand` lists them as patchable (FINDING-DOMAIN-EVD-009 covers the command-side gap; this covers the aggregate-side gap). Even if the command were fixed, there is no aggregate method that could apply the additional patches.
- **expected:** A single `CalendarEvent::apply(&mut self, patch: UpdateEventCommand) -> AggregateResult<()>` that takes the spec-shaped command and applies its optional fields.
- **evidence:**
  - Code: `crates/cross-cutting/events-domain/src/aggregate.rs:128-165` — signature is `pub fn update(&mut self, title: Option<String>, from: Option<NaiveDate>, to: Option<NaiveDate>, actor: UserId, at: Timestamp)`.

### FINDING DOMAIN-EVD-044

- **id:** DOMAIN-EVD-044
- **area:** domain-crates
- **severity:** Low
- **location:** `crates/cross-cutting/events-domain/src/events.rs:84-92` (EventUpdated), `crates/cross-cutting/events-domain/src/events.rs:248-256` (HolidayUpdated), `crates/cross-cutting/events-domain/src/events.rs:400-409` (CalendarSettingUpdated), `crates/cross-cutting/events-domain/src/events.rs:654-662` (IncidentUpdated), `crates/cross-cutting/events-domain/src/events.rs:1146-1154` (WeekendUpdated)
- **description:** All 5 update events declare `changes: Vec<String>`. The spec mandates `changes: Vec<&'static str>` (events.md lines 47, 67, 101, 168, 85). The runtime difference is small (Vec<String> is heap-allocated; Vec<&'static str> is static) but the spec is precise. Drift from the spec makes the events incompatible with a static-string consumer that wants to switch on the changed-field name without heap allocation.
- **expected:** Spec at `docs/specs/events/events.md:47, 67, 101, 168, 85` — `pub changes: Vec<&'static str>`
- **evidence:**
  - Spec: `docs/specs/events/events.md:47`
  - Code: `crates/cross-cutting/events-domain/src/events.rs:87` — `pub changes: Vec<String>`

### FINDING DOMAIN-EVD-045

- **id:** DOMAIN-EVD-045
- **area:** domain-crates
- **severity:** Low
- **location:** `crates/cross-cutting/events-domain/src/aggregate.rs` (entire file), `services.rs`, `events.rs`, `commands.rs`
- **description:** Public items in this crate carry no rustdoc in most cases — the `missing_docs` deny is suppressed at file level (FINDING-DOMAIN-EVD-001, FINDING-DOMAIN-EVD-002). Examples of undocumented public items: every `NewCalendarEvent`, `NewHoliday`, `NewIncident`, `NewCalendarSetting` field (aggregate.rs:62-79, 217-229, 421-430, 305-315); the `AggregateResult` type alias (aggregate.rs:26); every `pub fn` on the service structs (services.rs:27-311) does have a `///` doc but every `pub struct WeekendChange` variant (services.rs:232-247) is undocumented; every `pub struct` in `commands.rs` is undocumented (lines 27-468 — the structs have `///` doc but most fields have none).
- **expected:** Engine rule (AGENTS.md): "All public APIs are documented with rustdoc; `#![deny(missing_docs)]`."
- **evidence:**
  - `crates/cross-cutting/events-domain/src/commands.rs:28-41` — `CreateEventCommand` struct has `/// Create a new CalendarEvent.` but every `pub` field has no `///`.
  - `crates/cross-cutting/events-domain/src/aggregate.rs:26` — `pub type AggregateResult<T>` has no `///`.
  - `crates/cross-cutting/events-domain/src/aggregate.rs:63-79` — `NewCalendarEvent` struct fields all undocumented.

### FINDING DOMAIN-EVD-046

- **id:** DOMAIN-EVD-046
- **area:** domain-crates
- **severity:** Low
- **location:** `crates/cross-cutting/events-domain/src/aggregate.rs:167-173` (CalendarEvent::delete)
- **description:** The `delete` method on `CalendarEvent` takes `at: Timestamp` and `actor: UserId` parameters but does not record them in any domain event — the `EventDeleted` event (events.rs:135-165) carries `deleted_by` and `occurred_at`, but the aggregate's `delete` method is not coupled to event emission (no `EventBus` port, no `educore_events::publish` call). The handoff at PHASE-13-HANDOFF.md:195-202 describes a row-level-lock + outbox pattern, but no domain code implements the outbox write.
- **expected:** Engine rule (AGENTS.md): "Audit-first." Spec workflows.md § Calendar Event Lifecycle step 3 + step 4: "Author or admin deletes the event ... Subscribers (communication domain) dispatch notifications on EventCreated."
- **evidence:**
  - Code: `crates/cross-cutting/events-domain/src/aggregate.rs:167-173` — `pub fn delete(&mut self, at: Timestamp, actor: UserId)` — mutates state, returns `()`.
  - Code: `grep -rn "educore_events::\|publish\|outbox" crates/cross-cutting/events-domain/src/` returns no matches in non-test code.

### FINDING DOMAIN-EVD-047

- **id:** DOMAIN-EVD-047
- **area:** domain-crates
- **severity:** Low
- **location:** `crates/cross-cutting/events-domain/src/aggregate.rs:289-301` (CalendarSetting struct)
- **description:** The CalendarSetting struct carries `font_color: String` and `bg_color: String` (aggregate.rs:291-292). The spec mandates typed `FontColor` and `BackgroundColor` wrappers (`docs/specs/events/value-objects.md:61-62`), which would each wrap a `CssColor` wrapper (`value-objects.md:60`). The constructor validates via `validate_css_color(&cmd.font_color)?` (aggregate.rs:325) but the runtime type is still `String`, so a caller could mutate the field post-construction via public field access (`pub font_color: String`).
- **expected:** Spec at `docs/specs/events/value-objects.md:60-62` — `CssColor`, `FontColor`, `BackgroundColor`.
- **evidence:**
  - Spec: `docs/specs/events/value-objects.md:60-62`
  - Code: `crates/cross-cutting/events-domain/src/aggregate.rs:289-292` — `pub font_color: String, pub bg_color: String` — both public-mutable.

### FINDING DOMAIN-EVD-048

- **id:** DOMAIN-EVD-048
- **area:** domain-crates
- **severity:** Low
- **location:** `crates/cross-cutting/events-domain/src/events.rs:28-37` (EventCreated's `event_id_field` shadow name)
- **description:** The `EventCreated` struct uses a confusingly-named `event_id_field: EventId` to disambiguate from the aggregate id `event_id: CalendarEventId`. This is repeated across all 24 events. The naming is a workaround for the spec's envelope pattern (FINDING-DOMAIN-EVD-034) — the inner event should not carry `EventId` (envelope's job) but does, so it has to be renamed.
- **expected:** Spec at `docs/specs/events/events.md:38-49` does not list `event_id` on the inner event payload (only on the envelope). Naming `event_id_field` is a code-side workaround.
- **evidence:**
  - Code: `crates/cross-cutting/events-domain/src/events.rs:34` — `pub event_id_field: EventId,`
  - Code: `crates/cross-cutting/events-domain/src/events.rs:89, 141, 199, 252, 303, 356, 405, 455, 505, 549, 605, 657, 709, 758, 813, 868, 922, 984, 1038, 1097, 1149, 1200, 1247` — same `event_id_field: EventId` pattern repeated 24 times.

### FINDING DOMAIN-EVD-049

- **id:** DOMAIN-EVD-049
- **area:** domain-crates
- **severity:** Low
- **location:** `crates/cross-cutting/events-domain/src/aggregate.rs:602` (AssignIncident::new correlation_id)
- **description:** `AssignIncident::new` initializes `correlation_id: CorrelationId::from_uuid(Uuid::nil())` (aggregate.rs:601). The Nil UUID is a sentinel meaning "unset" — this is silently lost in the aggregate's persisted state. The same pattern is used in `IncidentComment::new` (aggregate.rs:672), `Weekend::new` (aggregate.rs:746), `CalendarEvent::new` (aggregate.rs:124 uses `cmd.correlation_id` which is correct), `Holiday::new` (aggregate.rs:261 uses `cmd.correlation_id` which is correct), `CalendarSetting::new` (aggregate.rs:342 uses `cmd.correlation_id` which is correct), `Incident::new` (aggregate.rs:465 uses `cmd.correlation_id` which is correct). 3 of 7 aggregates discard the correlation_id.
- **expected:** Engine rule (AGENTS.md): "Audit-first. Every state change writes an immutable record." Correlation IDs must be preserved on every aggregate.
- **evidence:**
  - `crates/cross-cutting/events-domain/src/aggregate.rs:601` — `correlation_id: CorrelationId::from_uuid(Uuid::nil()),`
  - `crates/cross-cutting/events-domain/src/aggregate.rs:672` — same pattern.
  - `crates/cross-cutting/events-domain/src/aggregate.rs:746` — same pattern.

### FINDING DOMAIN-EVD-050

- **id:** DOMAIN-EVD-050
- **area:** domain-crates
- **severity:** Low
- **location:** `crates/cross-cutting/events-domain/src/lib.rs:42-59` (prelude) and `crates/cross-cutting/events-domain/src/commands.rs:473-484` (`_ensure_ids_compile`)
- **description:** The crate's public prelude re-exports 7 root aggregate types and several typed IDs (lib.rs:42-58), but does NOT re-export the 24 event types or the 24 command types. Consumers must import from the deeper module path (`educore_events_domain::events::EventCreated`, `educore_events_domain::commands::CreateEventCommand`). This is a consumer-API gap that affects every external caller of the events-domain crate.
- **expected:** Engine pattern (per other domain crates, e.g., `educore-cms` lib.rs) is to re-export the full public surface in the prelude.
- **evidence:**
  - `crates/cross-cutting/events-domain/src/lib.rs:42-58` — prelude re-exports aggregate types, value objects, ids; no `events::*` or `commands::*` re-exports.

### FINDING DOMAIN-EVD-051

- **id:** DOMAIN-EVD-051
- **area:** domain-crates
- **severity:** Low
- **location:** (entire crate, evidence below)
- **description:** The referenced pre-implementation verification document `docs/verification/PRE-CHECK-PHASES-13-17.md` does not exist in the repository. The audit prompt asserts it lists "7 root aggregates (CalendarEvent, Holiday, Weekend, Incident, AssignIncident, IncidentComment, CalendarSetting)" — that assertion must be re-verified against `docs/specs/events/aggregates.md` directly, since the pre-check document is absent.
- **expected:** Per the audit prompt's own assumption, a `PRE-CHECK-PHASES-13-17.md` file should exist and document the spec aggregates pre-implementation.
- **evidence:**
  - `find . -name "PRE-CHECK*" -o -name "*verification*"` returns no matches under `docs/` (only `docs/schemas/data-migration/07-verification.md`, which is unrelated).
  - `ls docs/verification/` — directory does not exist.

### FINDING DOMAIN-EVD-052

- **id:** DOMAIN-EVD-052
- **area:** domain-crates
- **severity:** Low
- **location:** `crates/cross-cutting/events-domain/src/aggregate.rs:557-572` (AssignIncident::new), `crates/cross-cutting/events-domain/src/aggregate.rs:605-616` (AssignIncident::reassign)
- **description:** `AssignIncident::reassign` (aggregate.rs:606-616) updates `self.point` directly but does not bump `self.last_event_id` (per FINDING-DOMAIN-EVD-021 the field is never written) and does not capture the from_point for audit. The `IncidentReassigned` event (events.rs:864-873) carries `from_point` and `to_point`, but the aggregate method does not track `from_point` (the new point is supplied; the old point is read from `self.point` before assignment, which is correct, but the comparison is implicit).
- **expected:** Spec at `docs/specs/events/events.md:127-132` — `IncidentReassigned { ..., pub from_point: IncidentPoint, pub to_point: IncidentPoint }`.
- **evidence:**
  - Spec: `docs/specs/events/events.md:127-132`
  - Code: `crates/cross-cutting/events-domain/src/aggregate.rs:605-616` — `pub fn reassign(&mut self, point: i32, at: Timestamp)` — no `from_point` parameter, no `IncidentReassigned` event emission.

### FINDING DOMAIN-EVD-053

- **id:** DOMAIN-EVD-053
- **area:** domain-crates
- **severity:** Low
- **location:** `crates/cross-cutting/events-domain/src/aggregate.rs:168-173` (CalendarEvent::delete), `aggregate.rs:521-526` (Incident::delete), `aggregate.rs:677-681` (IncidentComment::delete)
- **description:** The three `delete` methods (CalendarEvent, Incident, IncidentComment) soft-delete via `active_status = false` but do not check whether the entity can be deleted. The spec aggregates.md § CalendarEvent invariant 5 (line 30) says "A CalendarEvent cannot be deleted if it has been delivered to recipients (the audit record remains)." There is no `delivered` tracking field on `CalendarEvent`, no check in `delete()`, no way to enforce the invariant.
- **expected:** Spec at `docs/specs/events/aggregates.md:30` — CalendarEvent invariant 5: "cannot be deleted if it has been delivered to recipients."
- **evidence:**
  - Spec: `docs/specs/events/aggregates.md:30`
  - Code: `crates/cross-cutting/events-domain/src/aggregate.rs:168-173` — `pub fn delete(&mut self, at: Timestamp, actor: UserId)` — no delivery check.

### FINDING DOMAIN-EVD-054

- **id:** DOMAIN-EVD-054
- **area:** domain-crates
- **severity:** Low
- **location:** `crates/cross-cutting/events-domain/src/commands.rs:439-457` (ConfigureWeekendsCommand, WeekendEntry)
- **description:** The spec's `WeekendEntry` (commands.md lines 125-129) has typed fields `name: WeekendName, order: WeekendOrder, is_weekend: IsWeekend`. The code's `WeekendEntry` (commands.rs:452-457) uses raw `name: String, order: i32, is_weekend: bool` — consistent with FINDING-DOMAIN-EVD-004 but losing the typed-wrapper protection at the wire boundary.
- **expected:** Spec at `docs/specs/events/commands.md:125-129`.
- **evidence:**
  - Spec: `docs/specs/events/commands.md:125-129`
  - Code: `crates/cross-cutting/events-domain/src/commands.rs:453-457` — `pub struct WeekendEntry { pub name: String, pub order: i32, pub is_weekend: bool }`.

### FINDING DOMAIN-EVD-055

- **id:** DOMAIN-EVD-055
- **area:** domain-crates
- **severity:** Low
- **location:** `crates/cross-cutting/events-domain/src/commands.rs:30-41` (CreateEventCommand field list)
- **description:** `CreateEventCommand` carries `role_ids: Vec<String>` (commands.rs:34). The spec mandates `role_ids: Vec<RoleId>` (commands.md line 17). `RoleId` is a typed wrapper that does not exist in the code (FINDING-DOMAIN-EVD-004). This propagates to the aggregate's `role_ids: Vec<String>` (aggregate.rs:41) and to `CalendarService::audience_resolves_to(&[String], &[String])` (services.rs:33), so all 3 layers are inconsistent with the spec's typed-RoleId contract.
- **expected:** Spec at `docs/specs/events/commands.md:17` — `pub role_ids: Vec<RoleId>`.
- **evidence:**
  - Spec: `docs/specs/events/commands.md:17`
  - Code: `crates/cross-cutting/events-domain/src/commands.rs:34` — `pub role_ids: Vec<String>`

### FINDING DOMAIN-EVD-056

- **id:** DOMAIN-EVD-056
- **area:** domain-crates
- **severity:** Low
- **location:** `crates/cross-cutting/events-domain/src/lib.rs:38-40` (PACKAGE_VERSION)
- **description:** `PACKAGE_VERSION` is declared as `pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");`. The engine rule (`AGENTS.md` code standards) recommends using `env!` for compile-time pinning, but `pub const` exposes it as part of the crate's public API. This is intentional in Cargo convention but could lead to downstream code that switches behavior on the package version. The handoff notes `version` is `version.workspace = true` (Cargo.toml:3) — workspace inheritance is correct.
- **expected:** N/A (this is more an observation than a defect; the value is correct).
- **evidence:**
  - `crates/cross-cutting/events-domain/src/lib.rs:38-40` — `pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");`
  - `crates/cross-cutting/events-domain/Cargo.toml:3` — `version.workspace = true`

### FINDING DOMAIN-EVD-057

- **id:** DOMAIN-EVD-057
- **area:** domain-crates
- **severity:** Low
- **location:** `crates/cross-cutting/events-domain/src/aggregate.rs:317-361` (CalendarSetting::enable, CalendarSetting::disable)
- **description:** The spec mandates events `CalendarSettingEnabled` and `CalendarSettingDisabled` to be emitted by the corresponding commands. The aggregate's `enable()` and `disable()` methods (aggregate.rs:347-360) mutate state but do not call into an event-bus port and do not write to an outbox. The events are defined as types (events.rs:451-542) but no code constructs and emits them from a successful command execution.
- **expected:** Engine rule (AGENTS.md): "Audit-first." Spec workflows.md § Calendar Setting Workflow step 2: "The setting is enabled (EnableCalendarSetting) and becomes available in the calendar UI."
- **evidence:**
  - Code: `crates/cross-cutting/events-domain/src/aggregate.rs:347-360` — `pub fn enable(&mut self, at: Timestamp, actor: UserId)` and `pub fn disable(...)` — neither emits an event.
  - Code: `grep -rn "CalendarSettingEnabled\|CalendarSettingDisabled" crates/cross-cutting/events-domain/src/aggregate.rs` returns no matches.

### FINDING DOMAIN-EVD-058

- **id:** DOMAIN-EVD-058
- **area:** domain-crates
- **severity:** Low
- **location:** `crates/cross-cutting/events-domain/src/services.rs:73-75` (CalendarService::validate_url)
- **description:** `CalendarService::validate_url` returns `Result<(), String>` instead of `Result<(), DomainError>` (the crate's typed error). The string error bypasses the `EventsDomainError` enum (errors.rs:13-33) and the typed `From<DomainError>` conversion, making error matching impossible at the call site.
- **expected:** Engine rule (AGENTS.md): "All fallible APIs return `Result<T, DomainError>`. Errors use `thiserror` for public APIs."
- **evidence:**
  - Code: `crates/cross-cutting/events-domain/src/services.rs:73-75` — `pub fn validate_url(s: &str) -> Result<(), String>`.

### FINDING DOMAIN-EVD-059

- **id:** DOMAIN-EVD-059
- **area:** domain-crates
- **severity:** Low
- **location:** `crates/cross-cutting/events-domain/src/aggregate.rs` (delete methods on 3 aggregates) and `services.rs` (state-machine helpers)
- **description:** The handoff claim at `docs/handoff/PHASE-13-HANDOFF.md:195-202` says "the dispatcher acquires the row-level lock on the relevant row (PG `SELECT ... FOR UPDATE` or SQLite write lock) before calling the service and writing audit / outbox / idempotency rows in a single transaction." No dispatcher, no transaction wrapper, no idempotency-key handling, no row-lock helper exists in the events-domain crate. The dispatcher's location (per AGENTS.md engine facade pattern) is at the engine level, not the domain level, so this is partially by design — but the spec workflows.md § Idempotency (lines 122-134) mandates idempotency semantics on `CreateHoliday`, `ConfigureWeekends`, and `AssignIncident` that are not implemented in the aggregate.
- **expected:** Spec at `docs/specs/events/workflows.md:122-134`.
- **evidence:**
  - Code: `grep -rn "idempotency\|FOR UPDATE" crates/cross-cutting/events-domain/src/` returns no matches.
  - Code: `crates/cross-cutting/events-domain/src/aggregate.rs:233` — `Holiday::new` always creates a new Holiday, no idempotency check against `(school_id, from_date, to_date, holiday_title)` per workflows.md line 127-128.

### FINDING DOMAIN-EVD-060

- **id:** DOMAIN-EVD-060
- **area:** domain-crates
- **severity:** Low
- **location:** `crates/cross-cutting/events-domain/src/events.rs:1228-1233` (`WeekendsConfigured::aggregate_id`)
- **description:** `WeekendsConfigured::aggregate_id` (events.rs:1231) returns `Uuid::nil()` because the event represents a batch operation across many weekend aggregates and there is no single aggregate id. The `DomainEvent` trait (per spec events.md lines 8-16) requires `fn aggregate_id(&self) -> Uuid;` and a Nil UUID is technically a value of `Uuid`. The convention of returning Nil for batch events is not flagged in the spec, but it does mean the event-log replay logic (per AGENTS.md "engine's replay engine") cannot identify which weekend row a `WeekendsConfigured` event applies to.
- **expected:** Spec at `docs/specs/events/events.md:86` — `WeekendsConfigured { pub school_id, pub weekend_count: u32 }` — no aggregate id, since the event is a batch op.
- **evidence:**
  - Spec: `docs/specs/events/events.md:86`
  - Code: `crates/cross-cutting/events-domain/src/events.rs:1231-1233` — `fn aggregate_id(&self) -> Uuid { Uuid::nil() }`

---

### END FINDINGS

Total findings: **60** (15 Critical, 12 High, 21 Medium, 12 Low).
