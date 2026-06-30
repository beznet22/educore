# Stub vs Implementation Audit

This file tracks the gap between spec invariants and the actual code
implementation for each domain crate's `services.rs`. The convention
is:

- **real** — the function implements every spec invariant the
  command is responsible for. Missing checks are nil.
- **partial** — the function implements at least one spec invariant
  but is missing others that the spec requires (auth checks, future-
  date validation, cross-aggregate lookups, etc).
- **stub** — the function carries self-documented "Phase 5 stub"
  placeholders (synthetic ids, epoch dates, fixed `false` returns)
  or is annotated for downstream resolution.

Spec invariants are drawn from
`docs/specs/<domain>/aggregates.md`.

---

## attendance

**Crate:** `crates/domains/attendance/src/services.rs`
**Function count:** 17
**Stub count:** 4 (one of which is `AttendanceService::is_late`)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `mark_student_attendance` (line 108) | StudentAttendance unique by `(school_id, student_id, attendance_date)`; `is_absent` derives from `attendance_type`; notes length validated | partial | Uniqueness via `uniqueness.student_day_exists` at `services.rs:122-125`; notes via `validate_notes` at `services.rs:118-120`; `is_absent` set from `attendance_type.is_absent()` at `services.rs:138`. **Missing:** invariant 2 (future-date check), invariant 6 (class-section must match student's `StudentRecord`), invariant 7 (`Attendance.Mark`/`Attendance.Update` RBAC). |
| `update_student_attendance` (line 182) | Updates append a new event; latest row replaces prior state; `no_changes` rejected | real | Field-level change tracking at `services.rs:195-203`; `no changes` rejection at `services.rs:213-216`; version bump at `services.rs:209`; `StudentAttendanceUpdated` event minted at `services.rs:217-224`. |
| `bulk_mark_student_attendance` (line 259) | Per-student uniqueness on `(school, student, date)`; roster-aware default-status emission for unmarked students | stub | Self-documented Phase 5 stub: "the service emits a single `default_type` aggregate per (class, section, date) for the unmarked students" at `services.rs:295-302`. The default aggregate uses a placeholder `StudentId` / `StudentRecordId` derived from the event UUID (`services.rs:308-311`) — real roster resolution is deferred to the dispatcher. `uniqueness` parameter is unused (`services.rs:262`). |
| `mark_subject_attendance` (line 486) | SubjectAttendance unique by `(school, student, subject, date)`; subject must be assigned to student's class-section | partial | Uniqueness via `uniqueness.subject_day_exists` at `services.rs:500-504`; notes via `validate_notes` at `services.rs:496-498`. **Missing:** invariant 2 (subject-to-class-section assignment lookup). |
| `update_subject_attendance` (line 559) | Updates append a new event; tracks `attendance_type` / `notes` / `notify` changes | real | Change tracking at `services.rs:571-587`; `no changes` rejection at `services.rs:589-592`; version bump at `services.rs:595`; `SubjectAttendanceUpdated` event at `services.rs:596-603`. |
| `mark_staff_attendance` (line 621) | StaffAttendance unique by `(school, staff, date)`; staff must be active on the date; `OnLeave` is distinct from `Absent` | partial | Uniqueness via `uniqueness.staff_day_exists` at `services.rs:635-639`; notes via `validate_notes` at `services.rs:631-633`. **Missing:** invariant 2 (active-roster check on date). |
| `update_staff_attendance` (line 681) | Updates append a new event; tracks `attendance_type` / `notes` changes | real | Change tracking at `services.rs:693-702`; `no changes` rejection at `services.rs:704-707`; version bump at `services.rs:710`; `StaffAttendanceUpdated` event at `services.rs:711-718`. |
| `mark_exam_attendance` (line 737) | ExamAttendance is owned by the assessment domain per spec (`docs/specs/attendance/aggregates.md:88`); uniqueness on `(exam, student, subject, date)` | partial | Aggregate construction at `services.rs:752-771`; event at `services.rs:772-783`. **Missing:** the `_uniqueness` parameter is ignored (`services.rs:741,749`); no future-date check; cross-domain ownership violation — function lives in `crates/domains/attendance/` but creates the assessment-owned `ExamAttendance` aggregate. |
| `update_exam_attendance` (line 798) | Updates append a new event; tracks `attendance_type` / `notes` changes | real | Change tracking at `services.rs:810-819`; `no changes` rejection at `services.rs:821-824`; version bump at `services.rs:827`; `ExamAttendanceUpdated` event at `services.rs:828-835`. |
| `import_attendance` (line 855) | BulkAttendanceImport idempotent on `(school_id, source, attendance_date)`; staging rows validated; one school, one academic year | partial | Idempotency via `uniqueness.import_source_date_exists` at `services.rs:887-893`; per-row notes validation at `services.rs:878-882`; staging rows constructed at `services.rs:906-922`. **Missing:** cross-row date uniqueness is deferred to dispatcher (`services.rs:884-886` comment); no per-row enrollment validation; no future-date check. |
| `validate_bulk_import` (line 962) | Status transitions `Pending -> Validated` or `Pending -> Failed`; failed rows do not produce attendance | real | Per-row well-formed check at `services.rs:984-992`; absent/failed counting at `services.rs:986-993`; status transition to `Validated` or `Failed` at `services.rs:996-1015`; either `Validated` or `Failed` event returned via `EitherImportEvent` (`services.rs:973-982`). |
| `commit_bulk_import` (line 1043) | `Validated -> Committed`; each validated row promotes to a `StudentAttendance` with real `student_record_id`, `class_id`, `section_id` resolved from enrollment | stub | Status guard at `services.rs:1067-1071`. **Self-documented Phase 5 stub:** "The staging row doesn't carry a `student_record_id` (Phase 5 stub); the dispatcher resolves it from the enrollment table on commit" at `services.rs:1098-1101`; same for `class_id` / `section_id` at `services.rs:1108-1113`. The promoted aggregate uses `event_id_to_uuid(event_id)` as the synthetic id for all three fields (`services.rs:1102-1113`). |
| `cancel_bulk_import` (line 1148) | Bulk import is cancellable only from a non-terminal state | real | Terminal-state guard at `services.rs:1163-1167`; status transition to `Cancelled` at `services.rs:1169`; `BulkImportCancelled` event at `services.rs:1173-1180`. |
| `request_absence_notification` (line 1190) | Emits `AbsenceNotificationRequested` for the resolved `(student_id, attendance_date)` of a `StudentAttendance` row | stub | Self-documented Phase 5 stub: "the Phase 5 stub carries the Unix epoch as a placeholder" for `attendance_date` at `services.rs:1203-1208`; placeholder `StudentId` at `services.rs:1213-1215`. Real values are deferred to the dispatcher. |
| `AttendanceService::is_late` (line 1242) | Late-arrival detection considering the school's `late_threshold_minutes` setting and day-of-week calendar | stub | Self-documented Phase 5 stub: "The full implementation considers the school's `late_threshold_minutes` setting and the day-of-week calendar. The integration test (Workstream D) exercises the production path" at `services.rs:1247-1252`. Function body returns `false` unconditionally. |
| `AttendanceService::emit_absence_event` (line 1259) | Mint `StudentAbsentForDay` from a `StudentAttendance` row iff the row is absent and carries a `last_event_id` | real | Absent-row guard at `services.rs:1262`; `last_event_id` invariant check at `services.rs:1273-1276` (comment notes the prior `unwrap_or_else` was removed to surface the invariant violation); `StudentAbsentForDay::new` at `services.rs:1277-1285`. |
| `AttendanceService::dedup_within_day` (line 1293) | Dedup `StudentAbsentForDay` events by `(student_id, attendance_date)`, first-wins | real | `HashSet<(Uuid, NaiveDate)>` dedup at `services.rs:1296-1304`; preserves input order via `Vec` accumulation. |

### Summary

- **Total pub fn:** 17
- **Real:** 8 (`update_student_attendance`, `update_subject_attendance`, `update_staff_attendance`, `update_exam_attendance`, `validate_bulk_import`, `cancel_bulk_import`, `emit_absence_event`, `dedup_within_day`)
- **Partial:** 5 (`mark_student_attendance`, `mark_subject_attendance`, `mark_staff_attendance`, `mark_exam_attendance`, `import_attendance`) — each implements its primary uniqueness invariant but is missing cross-aggregate lookups (class-section match, subject assignment, active-roster), future-date validation, or RBAC checks the spec requires.
- **Stub:** 4 (`bulk_mark_student_attendance`, `commit_bulk_import`, `request_absence_notification`, `is_late`) — each carries self-documented "Phase 5 stub" placeholders that defer real value resolution to the dispatcher.
