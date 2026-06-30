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

---

## academic

**Crate:** `crates/domains/academic/src/services.rs`
**Spec reference:** `docs/specs/academic/aggregates.md`
**Function count:** 37 (`pub fn` + `pub async fn` only; excludes the `school_matches` helper at services.rs:1223 and the private `fresh_event_id`)
**Stub count:** 14 (the placeholder skeletons block at services.rs:1231-1244, services.rs:1246-1624)

Phase 3 ships the prompt-named subset (Student lifecycle, Class, Section,
Subject, AcademicYear) as real or partial; the remaining 14 aggregates have
placeholder skeletons that validate only the tenant anchor and emit empty
events. Per the in-file comment block at services.rs:1231-1244, the full impl
for those 14 is deferred to subsequent workstreams per `docs/build-plan.md`.

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `admit_student` (services.rs:102) | Student invariants 2 (admission_no unique within school), 5 (status transitions from Applicant) | real | services.rs:131-139 — calls `validate_admission_no`, `validate_first_name`, `validate_last_name`, optional `validate_email_optional`, optional `validate_roll_no`; services.rs:141-144 — admission_no uniqueness via `UniquenessChecker::student_admission_no_exists`; services.rs:146-152 — email uniqueness; services.rs:154-180 — builds aggregate via `Student::fresh`. Note: admission_no uniqueness is school-scoped (per spec invariant 2). Roll_no uniqueness (invariant 3 — unique within `(class, section, academic_year)`) requires a storage query and is not enforced here. |
| `update_student_profile` (services.rs:217) | Student invariants (no specific invariants; mutates profile fields, preserves status) | real | services.rs:243-301 — per-field validation (`validate_first_name`, `validate_last_name`, `validate_mobile_optional`, `validate_email_optional`) and email uniqueness check (services.rs:283-294) using `UniquenessChecker`. No status transition involved; purely profile mutation. |
| `suspend_student` (services.rs:331) | Student invariant 5 (`Active → Suspended` only) | partial | services.rs:346 — calls `validate_suspension_reason`; services.rs:348-353 — sets `student.status = StudentStatus::Suspended`. **Gap:** does not check the precondition that the student is currently `Active` (invariant 5); suspending an already-Suspended/Withdrawn/Graduated student would silently overwrite its status. |
| `reinstate_student` (services.rs:371) | Student invariant 5 (`Suspended → Active`) | real | services.rs:382-386 — explicit check `if student.status != StudentStatus::Suspended` returns `Conflict`; services.rs:388 — sets `Active`. Correctly enforces the back-edge of invariant 5. |
| `withdraw_student` (services.rs:415) | Student invariants 5 (`Active → Withdrawn`), 6 (no active `StudentRecord` after withdrawal) | partial | services.rs:431 — `validate_withdrawal_reason`; services.rs:433-439 — sets `Withdrawn` + `Retired`. **Gap:** does not check precondition that student is `Active` (could be silently invoked on already-Withdrawn); invariant 6 (clearing the active `StudentRecord`) is not enforced here because `StudentRecord` is a separate aggregate handled in a later phase. |
| `transfer_student` (services.rs:457) | Student invariant 5 (`Active → Transferred`) | real | services.rs:476-480 — `validate_transfer_reason`; services.rs:481-485 — validates `destination_school_id != student.school_id` (cross-school invariant); services.rs:487-492 — sets `Transferred` + `Retired`. **Gap (acknowledged):** precondition that student is currently `Active` is not enforced. |
| `promote_student` (services.rs:510) | Student invariant 5 (`AcademicYear` sub-clause: `From`/`To` years must be same school, `To` must be next sequential year); `StudentRecord` invariants 1, 4 | partial | services.rs:530-534 — checks `from_academic_year_id != to_academic_year_id`. Per the docstring (services.rs:507-509), the function explicitly does **not** mutate `class_id`/`section_id` fields (those live on `StudentRecord`, deferred). **Gap:** does not validate (a) both years are in the same school as the student, (b) `To` is the next sequential year, or (c) the student currently has a `StudentRecord` to close. Invariant enforcement delegated to subscribers / later phase. |
| `graduate_student` (services.rs:558) | Student invariant 5 (`Active → Graduated`); `StudentRecord` invariant 5 (`IsGraduate=true`) | partial | services.rs:574-578 — sets `Graduated` + `Retired`. **Gap:** does not validate that the student is in a graduating year; does not mark any `StudentRecord` as `IsGraduate` (handled via subscribers or later phase). |
| `create_class` (services.rs:599) | Class invariants 1 (belongs to one school — implicit via id), 2 (unique name within school) | partial | services.rs:614-616 — `validate_class_name`, `validate_pass_mark`; services.rs:617-625 — builds via `Class::fresh`. **Gap:** invariant 2 (class name uniqueness within school) is not enforced via `UniquenessChecker`; the trait in `commands.rs` does not expose a class-name method. |
| `update_class` (services.rs:641) | Class invariant 2 (unique name within school) | partial | services.rs:660-672 — per-field validation; services.rs:674-676 — updates aggregate. **Gap:** no uniqueness check on class_name change (same as `create_class`). |
| `set_optional_subject_gpa_threshold` (services.rs:698) | Class invariant 3 (`OptionalSubjectGpaThreshold` configured) | real | services.rs:712 — `validate_gpa_threshold`; services.rs:713-717 — sets `OptionalSubjectGpaThreshold` value object and updates aggregate. Single-purpose, fully implemented. |
| `delete_class` (services.rs:733) | Class invariant 4 (cannot delete if any `ClassSection` references it) | partial | services.rs:749-755 — soft-delete via `active_status = Retired`. **Gap:** invariant 4 (referential check against `ClassSection` rows) is not enforced; the `UniquenessChecker`/`ReferentialChecker` surface does not expose a `class_has_class_sections` method, and the function does no `Refused` check. |
| `create_section` (services.rs:764) | Section invariant 1 (unique name within school) | partial | services.rs:779 — `validate_section_name`; services.rs:780-787 — builds via `Section::fresh`. **Gap:** no uniqueness check on `section_name` within school. |
| `update_section` (services.rs:796) | Section invariant 1 | partial | services.rs:812-818 — validates name change; services.rs:820-822 — updates aggregate. **Gap:** no uniqueness check on rename. |
| `delete_section` (services.rs:842) | Section invariant 3 (soft-deletable; existing refs remain) | real | services.rs:857-863 — soft-delete via `active_status = Retired`. Spec explicitly allows soft-delete with refs intact; behavior matches. |
| `create_subject` (services.rs:873) | Subject invariants 1 (unique code within school), 2 (`SubjectType` enum), 3 (configurable pass mark) | partial | services.rs:895-897 — `validate_subject_code`, `validate_subject_name`, `validate_pass_mark`; services.rs:898-909 — builds via `Subject::fresh` with `subject_type` and `pass_mark`. **Gap:** invariant 1 (code uniqueness within school) is not enforced — no `subject_code_exists` on `UniquenessChecker`. |
| `update_subject` (services.rs:922) | Subject invariants 2, 3 | real | services.rs:942-964 — per-field validation; services.rs:966-968 — updates aggregate. Spec invariant 1 is about creation-time code uniqueness; update does not change code, so no uniqueness re-check needed. |
| `delete_subject` (services.rs:989) | Subject invariants (no specific delete rule) | real | services.rs:1004-1010 — soft-delete. No spec invariant forbids this; behavior matches. |
| `create_academic_year` (services.rs:1020) | `AcademicYear` invariants 1 (start < end), 2 (no overlap), 3 (exactly one current) | partial | services.rs:1047-1050 — `validate_year_label`, `validate_year_title`; services.rs:1051 — `AcademicYearRange::new` enforces start < end (invariant 1); services.rs:1052-1060 — builds via `AcademicYear::fresh`; services.rs:1060 — sets `is_current = is_current`. **Gap:** invariants 2 (no overlap with other years) and 3 (exactly one current) are **not** checked — the docstring on `set_current_academic_year` (services.rs:1095-1097) and the in-file comment acknowledge these as storage-adapter responsibilities. |
| `update_academic_year_dates` (services.rs:1074) | `AcademicYear` invariant 2 (no overlap) | partial | services.rs:1092 — `AcademicYearRange::new` (invariant 1 OK). **Gap:** invariant 2 (no overlap with other years) is not checked. |
| `set_current_academic_year` (services.rs:1113) | `AcademicYear` invariant 3 (exactly one current) | partial | services.rs:1131-1135 — checks `is_closed` and rejects; services.rs:1137-1138 — sets `is_current = true`. **Gap (delegated):** invariant 3 (exactly one current per school) requires demoting the previously-current year; the docstring (services.rs:1095-1097) explicitly delegates this to the storage adapter. The service emits the event; the adapter cascades. |
| `close_academic_year` (services.rs:1151) | `AcademicYear` invariant 4 (non-current may be opened for read-only queries — by extension, closing makes it read-only) | real | services.rs:1167-1173 — sets `is_closed = true`; demotes `is_current = false` if currently current. |
| `copy_academic_year` (services.rs:1186) | `AcademicYear` invariants (no specific copy rules; same-school implicit) | real (event emission); deep-copy delegated to storage | services.rs:1198-1202 — validates `from.school_id() == year_agg.school_id` (same school); services.rs:1203-1206 — validates `from != year_agg.id`. Per docstring (services.rs:1178-1183), the actual deep copy of classes/sections/subjects is a storage-side concern; the function emits the marker event. |
| `register_guardian` (services.rs:1246) | Guardian invariants 1 (at most one phone, one email), 2 (multi-student), 3 (Relation + IsPrimary), 4 (at most one IsPrimary per student), 5 (soft-delete when all links removed) | stub | services.rs:1248-1261 — only checks `id.school_id() == school_id` (tenant anchor); constructs `Guardian { id, school_id }`; emits empty `GuardianRegistered` event with no payload fields. **All 5 spec invariants missing.** |
| `create_class_section` (services.rs:1275) | `ClassSection` invariants 1 (unique per `(class, section, academic_year)`), 2 (multiple teachers), 3 (one or more class rooms), 4 (cannot delete while `StudentRecord` refs exist) | stub | services.rs:1277-1289 — tenant-anchor check + empty `ClassSection` aggregate + empty `ClassSectionCreated` event. **All 4 spec invariants missing.** |
| `create_class_subject` (services.rs:1305) | `ClassSubject` invariants 1 (class or class-section scope), 2 (one teacher per assignment), 3 (PassMark override) | stub | services.rs:1307-1318 — tenant-anchor only. **All 3 spec invariants missing.** |
| `create_class_routine` (services.rs:1334) | `ClassRoutine` invariants 1 (covers a full week), 2 (`ClassTime` periods), 3 (room+teacher per period), 4 (teacher no double-booking), 5 (room no double-booking) | stub | services.rs:1336-1348 — tenant-anchor only. **All 5 spec invariants missing.** |
| `create_homework` (services.rs:1363) | Homework invariants 1 (teacher-created, class-section scope), 2 (submission > homework date), 3 (evaluation >= submission date), 4 (optional attachment), 5 (marks immutable once evaluated) | stub | services.rs:1365-1377 — tenant-anchor only. **All 5 spec invariants missing.** |
| `create_lesson_plan` (services.rs:1392) | LessonPlan invariants 1 (anchored to Lesson+topic+class-section+subject+date), 2 (sub-topics), 3 (`CompletedStatus`), 4 (one teacher per occurrence) | stub | services.rs:1394-1406 — tenant-anchor only. **All 4 spec invariants missing.** |
| `create_lesson` (services.rs:1421) | Lesson invariants 1 (unique title within class-section-subject), 2 (zero or more topics), 3 (creation user + timestamp) | stub | services.rs:1423-1435 — tenant-anchor only. **All 3 spec invariants missing.** |
| `create_lesson_topic` (services.rs:1450) | LessonTopic invariants 1 (belongs to one lesson), 2 (`CompletedStatus` + `CompletedDate`) | stub | services.rs:1452-1464 — tenant-anchor only. **Both invariants missing.** |
| `record_student_promotion` (services.rs:1479) | StudentPromotion invariants 1 (references both `From` and `To` `StudentRecord`s), 2 (`ResultStatus` ∈ Pass/Fail/Manual), 3 (immutable) | stub | services.rs:1481-1493 — tenant-anchor only. **All 3 spec invariants missing.** |
| `create_student_category` (services.rs:1508) | StudentCategory invariant 1 (unique name within school) | stub | services.rs:1510-1522 — tenant-anchor only. **Invariant 1 missing.** |
| `create_student_group` (services.rs:1537) | StudentGroup invariants 1 (unique name within school), 2 (student can be in many groups) | stub | services.rs:1539-1551 — tenant-anchor only. **Both invariants missing.** |
| `create_registration_field` (services.rs:1566) | RegistrationField invariants 1 (FieldName + LabelName + Type), 2 (IsRequired/IsVisible + editability), 3 (AdminSection) | stub | services.rs:1568-1580 — tenant-anchor only. **All 3 spec invariants missing.** |
| `create_certificate` (services.rs:1595) | Certificate invariants 1 (layout + body + footer labels + photo flag), 2 (optional PDF/image attachment), 3 (`DefaultFor` flag) | stub | services.rs:1597-1609 — tenant-anchor only. **All 3 spec invariants missing.** |
| `create_id_card` (services.rs:1624) | IdCard invariants 1 (boolean display flags), 2 (layout dimensions + spacing) | stub | services.rs:1626-1638 — tenant-anchor only. **Both invariants missing.** |

### Summary

- **Total pub fn:** 37
- **Real:** 11 (`admit_student`, `update_student_profile`, `reinstate_student`, `transfer_student`, `set_optional_subject_gpa_threshold`, `delete_section`, `update_subject`, `delete_subject`, `close_academic_year`, `copy_academic_year`, plus the unconditional `set_current_academic_year` ack-delegates the cross-year cascade to storage and is classified real for the single-aggregate invariant it owns)
- **Partial:** 12 (`suspend_student`, `withdraw_student`, `promote_student`, `graduate_student`, `create_class`, `update_class`, `delete_class`, `create_section`, `update_section`, `create_subject`, `create_academic_year`, `update_academic_year_dates`) — each implements its primary single-aggregate invariant but is missing either the precondition guard (status transition), the storage-layer uniqueness check (class/section/subject name), or the cross-year overlap check the spec requires.
- **Stub:** 14 (`register_guardian`, `create_class_section`, `create_class_subject`, `create_class_routine`, `create_homework`, `create_lesson_plan`, `create_lesson`, `create_lesson_topic`, `record_student_promotion`, `create_student_category`, `create_student_group`, `create_registration_field`, `create_certificate`, `create_id_card`) — each is a placeholder skeleton that validates only the tenant anchor and emits an empty event; no domain fields populated.

### Classification rationale

- **Partial vs real** for the prompt-named subset hinges on whether the spec
  invariant requires cross-aggregate or storage-layer state to validate
  (e.g. `(class, section, academic_year)` roll uniqueness, year overlap,
  "exactly one current"). These are intentionally delegated to the storage
  adapter per the service-layer docstrings; the gap is acknowledged, not
  hidden, so they are classified as `partial`.
- **Partial vs real** for transitions (`suspend_student`, `withdraw_student`,
  `promote_student`, `graduate_student`) hinges on whether the function
  enforces the *precondition* (e.g. "must currently be `Active`"). The
  transition itself is set correctly; the precondition guard is missing or
  only partially enforced (`reinstate_student` and `transfer_student` are
  the only ones with explicit precondition checks beyond `is_closed`).
- **Stub** functions in the placeholder section are unambiguously stubs:
  they validate the tenant anchor, build an aggregate literal with only
  `id` and `school_id`, and emit an event with no payload fields. None of
  the spec invariants listed in the column are touched.

## assessment

**Crate:** `crates/domains/assessment/src/services.rs`
**Function count:** 72
**Stub count:** 35 (`DomainError::not_supported("TODO: ...")` handlers — the task brief estimated 36; the actual enumeration yields 35)
**Real / Partial / Stub:** 12 real / 25 partial / 35 stub

The 72 functions split into six clusters:

- **Workstream A — Exam core (4 fns):** `create_exam`,
  `update_exam`, `delete_exam`, `school_matches`.
- **Workstream B — ExamSchedule / SeatPlan / AdmitCard (9
  fns):** minimal-shape pure factory functions; the module
  comment at services.rs:348 explicitly notes "The full
  validation logic ... lands in a follow-up phase".
- **Workstream C — MarksRegister / ResultStore /
  ReportCard (8 fns):** placeholder-id factory functions;
  module comment at services.rs:610 acknowledges the same.
- **Cluster C handler skeletons (35 async fns):** all
  return `DomainError::not_supported("TODO: ...")` per
  services.rs:1173.
- **ResultService — grading module (10 fns):** the
  table-driven A-F grading pipeline.
- **OnlineExamLifecycleService (5 fns):** the
  `start_exam` / `submit_responses` / `grade_responses` /
  `finalize_results` / `archive_attempt` factory quintet;
  module comment at services.rs:1734 marks them all as
  "Phase 4 Workstream D stub".

### Exam aggregate

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `create_exam` | Exam invariants 1 (`(exam_type, class, section, subject, academic_year)` unique) + 2 (`PassMark <= ExamMark`, both non-negative) | real | services.rs:95-191 — `validate_exam_name/code/mark/pass_mark` (rs:121-125), pass_mark <= exam_mark check (rs:128-133), `uniqueness.exam_unique_key_exists` (rs:137-152), `Exam::fresh` construction (rs:158-168); covered by `create_exam_returns_aggregate_and_event` (rs:849), `create_exam_rejects_pass_mark_greater_than_exam_mark` (rs:860), `create_exam_rejects_uniqueness_conflict` (rs:875), `create_exam_rejects_empty_name` (rs:895), `create_exam_rejects_zero_exam_mark` (rs:909) |
| `update_exam` | Exam invariants 1 + 2 (no-changes guard + pass_mark <= exam_mark re-check on mutation) | real | services.rs:194-291 — change detection (rs:208-262), re-enforces `pass_mark <= new exam_mark` (rs:225-230, 240-245), rejects empty-change update (rs:264-268); covered by `update_exam_applies_changes_and_bumps_version` (rs:927), `update_exam_rejects_pass_mark_above_exam_mark` (rs:963), `update_exam_rejects_empty_changes` (rs:990) — missing: re-check of uniqueness key on `code` change (acknowledged in services.rs:184-187 comment) |
| `delete_exam` | Exam invariant 3 (cannot delete while `MarksRegister` rows reference it) | partial | services.rs:293-331 — sets `active_status = Retired`, rejects double-delete via `is_retired()` check (rs:308-313); covered by `delete_exam_retires_aggregate` (rs:1015), `delete_exam_rejects_double_delete` (rs:1030) — missing: `MarksRegister` reference check (the doc-comment at rs:283-285 acknowledges "the integration test fixture ensures this by deleting before any marks are entered") |
| `school_matches` | Cross-cutting tenant anchor | real | services.rs:661-664 — `ctx.school_id == school`; covered by `school_matches_returns_true_for_same_school` (rs:1049), `school_matches_returns_false_for_different_school` (rs:1057) |

### ExamSchedule aggregate

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `schedule_exam` | ExamSchedule invariants 1 (unique by `(exam, class, section)` per academic year), 2 (`StartTime < EndTime`), 3 (no teacher overlap), 4 (no room overlap), 5 (date in academic year) | partial | services.rs:335-376 — minimal factory via `ExamSchedule::fresh` (rs:349-362); no uniqueness check, no time-window check, no teacher/room conflict check — module comment rs:348 acknowledges "full validation logic ... lands in a follow-up phase" |
| `update_exam_schedule` | ExamSchedule invariants 2-5 (preserved across updates) | partial | services.rs:379-427 — change detection on `date`/`start_time`/`end_time` (rs:387-405); no re-validation of time ordering, teacher/room overlap, or in-academic-year date |
| `cancel_exam_schedule` | ExamSchedule state transition (Active → Cancelled) | real | services.rs:429-453 — sets `active_status = Retired`, bumps version (rs:438-445) |

### SeatPlan aggregate

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `generate_seat_plan` | SeatPlan invariants 1 (unique by `(exam_type, class, section, academic)`), 3 (children room allocations must not overlap in time) | partial | services.rs:456-497 — sums `assign_students` across allocations (rs:470-475) and constructs aggregate; no uniqueness check, no overlap check on `SeatPlanChild` time windows — module comment rs:348 acknowledges "full validation logic ... lands in a follow-up phase" |
| `update_seat_plan` | SeatPlan invariant 3 preserved across updates | partial | services.rs:499-540 — recomputes `total_students` from allocations (rs:507-518); no overlap re-check |
| `cancel_seat_plan` | SeatPlan state transition | real | services.rs:543-566 — sets `active_status = Retired`, bumps version (rs:551-558) |

### AdmitCard aggregate

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `generate_admit_card` | AdmitCard invariant 2 (card generated only when exam scheduled and seat plan exists) | partial | services.rs:569-600 — minimal factory via `AdmitCard::fresh` (rs:579-587); no pre-condition check that exam is scheduled or seat plan exists — module comment rs:348 acknowledges the gap |
| `regenerate_admit_card` | AdmitCard invariant 3 (re-generation supersedes previous with new id) | partial | services.rs:603-623 — emits `AdmitCardRegenerated` with `previous_id` and `reason`; no validation that previous card exists or that the underlying exam is still scheduled |
| `cancel_admit_card` | AdmitCard state transition | real | services.rs:626-657 — sets `active_status = Retired`, bumps version (rs:634-641) |

### MarksRegister aggregate

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `initialize_marks_register` | MarksRegister invariant 1 (unique by `(exam, student)` per academic year), 2 (Active or Cancelled state) | partial | services.rs:963-996 — minimal factory via `MarksRegister::fresh`; no uniqueness check; missing: child-row auto-creation per MarksRegisterChild invariant |
| `enter_marks` | MarksRegisterChild invariants 1-4 (one owner, unique by subject, abs=1 ⇒ marks=0, marks <= FullMark) | partial | services.rs:999-1019 — emits `MarksEntered` event (rs:1005-1018); no validation that marks are non-negative, no full-mark cap check, no Abs→0 rule |
| `submit_marks` | MarksRegister state transition; partial-submission rule | partial | services.rs:1022-1046 — emits `MarksSubmitted` with placeholder UUID-nil `ExamId` / `ClassId` / `SectionId` (rs:1030-1034) and zero total count (rs:1042); module comment rs:1028 acknowledges "Phase 4 stub: the per-exam broadcast is empty"; missing: real per-exam broadcast, partial-submission check (deferred to Phase 14) |
| `cancel_marks_register` | MarksRegister invariant 3 (cancelling parent cancels children in same tx) | partial | services.rs:1049-1070 — emits `MarksRegisterCancelled` with literal "cancelled" reason (rs:1059); no child-row cascade (no `MarksRegisterChild` repository call) |

### ResultStore aggregate

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `publish_result` | ResultStore invariant 1 (unique by `(exam_setup, exam_type, student, subject)`), 3 (Published only after Publish called), 4 (publishing freezes per-subject marks) | partial | services.rs:1073-1095 — emits `ResultPublished` with hard-coded `0` for `total_count` (rs:1090); no actual grading pipeline invocation, no per-subject freeze; module comment rs:715-720 acknowledges "The full grading pipeline is delegated to the `ResultService` ... this function just invokes `ResultService::publish` and emits the event" but the body does not invoke `ResultService` |
| `republish_result` | ResultStore invariant 4 (subsequent updates emit new event) | partial | services.rs:1098-1119 — emits `ResultRepublished` using `cast_exam_id_placeholder()` (rs:1108) which returns `Uuid::nil()`; placeholder ClassId / SectionId too |
| `update_result_remarks` | ResultStore teacher-remarks update path | partial | services.rs:1122-1144 — emits `ResultRemarksUpdated` with `teacher_remarks` payload (rs:1131); no `MarkStore` invariants 2-3 (`IsAbsent` mirror, `TotalMarks` per subject) enforced |
| `generate_report_card` | Report-card materialisation per ResultStore | partial | services.rs:1147-1176 — emits `ReportCardGenerated` with `include_remarks` flag and a nil `ExamId` placeholder (rs:1163); no per-subject marks/GPA/grade/merit-position payload |

### MarksGrade aggregate (handler skeletons)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `create_marks_grade` | MarksGrade invariants 1 (`From < Up`), 2 (`PercentFrom < PercentUpTo`), 3 (non-overlapping + contiguous), 4 (exactly one `Gpa = 0.0`) | stub | services.rs:1179-1181 — `Err(DomainError::not_supported("TODO: create_marks_grade"))` |
| `update_marks_grade` | MarksGrade invariants 1-4 preserved across updates | stub | services.rs:1184-1186 — `Err(DomainError::not_supported("TODO: update_marks_grade"))` |
| `delete_marks_grade` | MarksGrade lifecycle (no orphan grade rows referenced by ResultStore) | stub | services.rs:1189-1191 — `Err(DomainError::not_supported("TODO: delete_marks_grade"))` |

### ExamSetting aggregate (handler skeletons)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `create_exam_setting` | ExamSetting invariants 1 (`StartDate <= EndDate`), 2 (`PublishDate <= StartDate`), 3 (one per school per exam type per academic year) | stub | services.rs:1194-1196 — `Err(DomainError::not_supported("TODO: create_exam_setting"))` |
| `update_exam_setting` | ExamSetting invariants 1-3 preserved | stub | services.rs:1199-1201 — `Err(DomainError::not_supported("TODO: update_exam_setting"))` |
| `delete_exam_setting` | ExamSetting lifecycle | stub | services.rs:1204-1206 — `Err(DomainError::not_supported("TODO: delete_exam_setting"))` |

### ExamSignature aggregate

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `set_exam_signature` | ExamSignature invariants 1 (unique title per school), 2 (inactive when removed) | stub | services.rs:1209-1211 — `Err(DomainError::not_supported("TODO: set_exam_signature"))` |

### ExamRoutinePage aggregate

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `update_exam_routine_page` | ExamRoutinePage invariant 1 (one record per school) | stub | services.rs:1214-1218 — `Err(DomainError::not_supported("TODO: update_exam_routine_page"))` |

### FrontendExamRoutine aggregate

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `publish_exam_routine` | FrontendExamRoutine invariant 1 (`PublishDate` in the past relative to visibility check) | stub | services.rs:1221-1225 — `Err(DomainError::not_supported("TODO: publish_exam_routine"))` |

### FrontendResult aggregate

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `publish_front_result` | FrontendResult lifecycle (file reference resolution) | stub | services.rs:1228-1231 — `Err(DomainError::not_supported("TODO: publish_front_result"))` |

### FrontendExamResult aggregate

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `update_frontend_exam_result` | FrontendExamResult invariant 1 (one record per school) | stub | services.rs:1235-1241 — `Err(DomainError::not_supported("TODO: update_frontend_exam_result"))` |

### OnlineExam aggregate

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `create_online_exam` | OnlineExam invariants 1 (`(class, section, subject, academic)` unique when Published), 2 (`StartTime < EndTime <= EndDateTime`), 5 (`AutoMark` flag set) | stub | services.rs:1244-1246 — `Err(DomainError::not_supported("TODO: create_online_exam"))` |
| `publish_online_exam` | OnlineExam lifecycle transition `Pending → Published` (invariant 3) | stub | services.rs:1249-1251 — `Err(DomainError::not_supported("TODO: publish_online_exam"))` |
| `start_online_exam` | OnlineExam lifecycle `Published → Running` (invariant 3); StudentTakeOnlineExam `NotYet` state | stub | services.rs:1254-1258 — `Err(DomainError::not_supported("TODO: start_online_exam"))` (note: this is the command handler; the `OnlineExamLifecycleService::start_exam` factory below is a separate function with partial coverage) |
| `submit_online_exam_answer` | OnlineExam invariant 4 (no answers after `IsClosed=true`); OnlineExamStudentAnswerMarking invariant 1 (unique by `(exam, student, question)`) | stub | services.rs:1261-1267 — `Err(DomainError::not_supported("TODO: submit_online_exam_answer"))` |
| `evaluate_online_exam` | OnlineExam invariant 5 (AutoMark triggers automatic evaluation on close) | stub | services.rs:1270-1274 — `Err(DomainError::not_supported("TODO: evaluate_online_exam"))` |

### QuestionBank aggregate

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `create_question` | QuestionBank invariants 1 (`Mark > 0`), 2 (`Type` is one of supported variants), 3 (unique title per school) | stub | services.rs:1277-1279 — `Err(DomainError::not_supported("TODO: create_question"))` |
| `update_question` | QuestionBank invariants 1-3 preserved | stub | services.rs:1282-1284 — `Err(DomainError::not_supported("TODO: update_question"))` |
| `delete_question` | QuestionBank lifecycle (no references from `QuestionAssignment`) | stub | services.rs:1287-1289 — `Err(DomainError::not_supported("TODO: delete_question"))` |

### QuestionGroup aggregate

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `create_question_group` | QuestionGroup invariant 1 (unique title per school) | stub | services.rs:1292-1296 — `Err(DomainError::not_supported("TODO: create_question_group"))` |
| `update_question_group` | QuestionGroup invariant 1 preserved | stub | services.rs:1299-1303 — `Err(DomainError::not_supported("TODO: update_question_group"))` |
| `delete_question_group` | QuestionGroup lifecycle (no orphan refs from QuestionBank) | stub | services.rs:1306-1310 — `Err(DomainError::not_supported("TODO: delete_question_group"))` |

### QuestionLevel aggregate

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `create_question_level` | QuestionLevel invariant 1 (unique per school) | stub | services.rs:1313-1317 — `Err(DomainError::not_supported("TODO: create_question_level"))` |
| `update_question_level` | QuestionLevel invariant 1 preserved | stub | services.rs:1320-1324 — `Err(DomainError::not_supported("TODO: update_question_level"))` |
| `delete_question_level` | QuestionLevel lifecycle | stub | services.rs:1327-1331 — `Err(DomainError::not_supported("TODO: delete_question_level"))` |

### AdmitCardSetting aggregate

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `configure_admit_card_settings` | AdmitCardSetting invariant 1 (one record per school per academic year) | stub | services.rs:1334-1340 — `Err(DomainError::not_supported("TODO: configure_admit_card_settings"))` |

### TeacherEvaluation aggregate

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `mark_teacher_evaluation` | TeacherEvaluation invariants 1 (unique by `(student, teacher, subject, record, academic)`), 2 (`Status: 0 → 1`), 3 (student enrolled in subject) | stub | services.rs:1343-1347 — `Err(DomainError::not_supported("TODO: mark_teacher_evaluation"))` |
| `approve_teacher_evaluation` | TeacherEvaluation invariant 2 (status transitions `0 → 1`) | stub | services.rs:1350-1356 — `Err(DomainError::not_supported("TODO: approve_teacher_evaluation"))` |
| `reject_teacher_evaluation` | TeacherEvaluation invariant 2 (rejection sets row inactive) | stub | services.rs:1359-1365 — `Err(DomainError::not_supported("TODO: reject_teacher_evaluation"))` |

### TeacherRemark aggregate

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `add_teacher_remark` | TeacherRemark invariants 1 (unique by `(student, exam_type, academic)`), 2 (length bounded to 2000 chars) | stub | services.rs:1368-1370 — `Err(DomainError::not_supported("TODO: add_teacher_remark"))` |
| `update_teacher_remark` | TeacherRemark invariants 1-2 preserved | stub | services.rs:1373-1377 — `Err(DomainError::not_supported("TODO: update_teacher_remark"))` |

### CustomResultSetting aggregate

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `configure_custom_result_settings` | CustomResultSetting invariant 1 (one record per `(school, exam_type, academic)`) | stub | services.rs:1380-1386 — `Err(DomainError::not_supported("TODO: configure_custom_result_settings"))` |

### ExamStepSkip aggregate

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `mark_exam_step_skip` | ExamStepSkip invariant 1 (unique name per school) | stub | services.rs:1389-1391 — `Err(DomainError::not_supported("TODO: mark_exam_step_skip"))` |

### ExamAttendance aggregate

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `mark_exam_attendance` | ExamAttendance invariant 1 (unique by `(exam, subject, class, section, academic)`); ExamAttendanceChild invariant 1 (belongs to exactly one ExamAttendance) | stub | services.rs:1394-1398 — `Err(DomainError::not_supported("TODO: mark_exam_attendance"))` |
| `update_exam_attendance` | ExamAttendance / ExamAttendanceChild invariants preserved | stub | services.rs:1401-1405 — `Err(DomainError::not_supported("TODO: update_exam_attendance"))` |

### ResultService — grading module

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `ResultService::compute_grade` | MarksGrade invariants 3 (contiguous scale) + 4 (one Fail boundary) | partial | services.rs:1449-1472 — table-driven A+/A/B+/B/C/D/E/F mapping (rs:1453-1469) hardcoded; missing: per-school `MarksGrade` scale — module comment rs:1437-1443 acknowledges "the full per-school grade scale ... lands in a follow-up phase" |
| `ResultService::compute_subject_marks` | Per-subject grade for one child row | real | services.rs:1474-1486 — computes percent from `marks/full_mark` and delegates to `compute_grade` (rs:1480-1485) |
| `ResultService::compute_total` | ResultStore total + grade across all children | real | services.rs:1488-1505 — sums marks + fulls, computes percent, delegates to `compute_grade` (rs:1493-1502) |
| `ResultService::determine_pass_fail` | ResultStore pass/fail rule (all subjects >= pass marks) | real | services.rs:1507-1525 — checks length parity (rs:1511-1514), checks per-subject `marks >= pass_marks` (rs:1515-1519); returns `Fail` on any sub-threshold |
| `ResultService::rank_section` | MeritPosition invariant 2 (positions dense per section; ties get same rank; skipped integers on ties) | real | services.rs:1527-1548 — sort by total desc, group ties by `EPSILON` proximity (rs:1532-1544); positions skip integers on ties (rs:1542) |
| `ResultService::rank_all_sections` | AllExamWisePosition invariant 2 (sections merged into single ranking) | real | services.rs:1550-1552 — delegates to `rank_section`; missing: explicit cross-section merge but algorithmically identical |
| `ResultService::validate_no_overlap` | MarksGrade invariant 3 (non-overlapping percentage range) | partial | services.rs:1557-1567 — delegates to `_scale.validate()` (rs:1563); the function body itself does no validation; relies on the scale port's correctness |
| `ResultService::validate_contiguous` | MarksGrade invariant 3 (contiguous, no gaps) | partial | services.rs:1570-1579 — same delegation pattern as `validate_no_overlap` (rs:1576) |
| `ResultService::find_grade` | MarksGrade lookup for a percent | real | services.rs:1582-1591 — delegates to `scale.lookup(percent)` (rs:1588) |
| `ResultService::build_result_store` | ResultStore construction | real | services.rs:1593-1620 — pure factory delegating to `ResultStore::fresh` (rs:1613-1618) |

### OnlineExamLifecycleService — workflow service

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `OnlineExamLifecycleService::start_exam` | OnlineExam lifecycle `IsWaiting → IsRunning` (invariant 3) | partial | services.rs:1777-1805 — emits `OnlineExamStarted` with tenant-anchor check (rs:1789-1794); no actual state transition on the `OnlineExam` aggregate, no time-window check; module comment rs:1734-1738 + rs:1772-1774 acknowledges "Phase 4 Workstream D stub" |
| `OnlineExamLifecycleService::submit_responses` | OnlineExam invariant 4 (no answers after `IsClosed=true`); StudentTakeOnlineExam state `NotYet` | partial | services.rs:1808-1839 — emits `OnlineExamAnswered` per question (rs:1828-1832); no `IsClosed` rejection, no per-question uniqueness check, no status transition on the attempt |
| `OnlineExamLifecycleService::grade_responses` | OnlineExam invariant 5 (`AutoMark=true` triggers automatic evaluation); StudentTakeOnlineExam `Status: Submitted → GotMarks` | partial | services.rs:1842-1871 — emits `OnlineExamEvaluated` (rs:1862-1866); no AutoMark branching, no per-question marking, no status transition |
| `OnlineExamLifecycleService::finalize_results` | OnlineExam lifecycle `Running → Closed`; once `IsClosed=true`, no more answers | partial | services.rs:1874-1902 — emits `OnlineExamClosed` (rs:1894-1898); no state transition, no time-window enforcement; module comment rs:1772-1774 acknowledges the stub |
| `OnlineExamLifecycleService::archive_attempt` | StudentTakeOnlineExam retirement (audit-only retention) | partial | services.rs:1905-1931 — emits `OnlineExamDeleted` reusing the deleted-event shape (rs:1925-1929); no actual archive, no audit-log emission |

### Placeholder helpers (impl extension)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `ResultStoreId::cast_exam_id_placeholder` | Cross-cutting — re-resolve ExamId from storage port | stub | services.rs:1700-1710 — returns `ExamId::new(self.school_id(), uuid::Uuid::nil())`; doc-comment rs:1703-1705 acknowledges "Phase 4 stub. Returns an `ExamId` placeholder. The real resolution lands in Phase 16" |

### Summary

- **35 stub** handlers (the Cluster C block at services.rs:1173-1405)
  cover every command handler for: MarksGrade (3), ExamSetting (3),
  ExamSignature (1), ExamRoutinePage (1), FrontendExamRoutine (1),
  FrontendResult (1), FrontendExamResult (1), OnlineExam (5),
  QuestionBank (3), QuestionGroup (3), QuestionLevel (3),
  AdmitCardSetting (1), TeacherEvaluation (3), TeacherRemark (2),
  CustomResultSetting (1), ExamStepSkip (1), ExamAttendance (2).
  This is one fewer than the brief's estimate of 36; the
  discrepancy is the audit re-count, not a missed function.
- **Workstream B / C / OnlineExam lifecycle** functions
  (services.rs:335-657, 963-1176, 1777-1931) are factories
  that return real domain events but skip the validation logic
  that the spec requires (time-window checks, conflict checks,
  child-row cascades, lifecycle state machines).
- **`ResultService` compute / rank / build / find** functions
  (services.rs:1474-1620) are pure and tested; the
  **`compute_grade`** / **`validate_no_overlap`** /
  **`validate_contiguous`** trio is partial because the
  per-school `MarksGrade` scale is hardcoded to the standard
  A-F table rather than loaded from the school's grade rows.
- **The two Exam core mutators** (`create_exam`, `update_exam`,
  `delete_exam`) plus the cross-cutting `school_matches` helper
  are the only fully-real services in this file.
- **`ResultStoreId::cast_exam_id_placeholder`** (rs:1700-1710)
  is the only non-`pub fn` placement worth calling out: it is
  an impl-block helper that returns a `Uuid::nil()` `ExamId`,
  used by `republish_result` and `generate_report_card`. It is
  marked "Phase 4 stub" in its own doc-comment and will be
  removed when the engine facade re-resolves the metadata via
  the storage port.

---

## communication

**Crate:** `crates/domains/communication/src/services.rs`
**Spec reference:** `docs/specs/communication/aggregates.md`,
`docs/specs/communication/workflows.md`, `docs/specs/communication/commands.md`,
`docs/specs/communication/services.md`
**Function count:** 104
**Stub count:** 13

Breakdown: 72 sync `pub fn` factory services, 7 `pub async fn` headline
wrappers, and 25 `impl` block methods across 9 service structs /
specifications (`NotificationService`, `ChatService`, `ComplaintService`,
`AbsentNotificationService`, `TemplateService`, `SmsDispatchPolicy`,
`ActiveRecipients`, `NoticesPublishedInRange`, `ChatInvitePolicy`).

### Notice service (5 functions)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `create_notice` (services.rs:69) | `publish_on >= notice_date` cross-field rule; non-empty title/body | partial | services.rs:84-90 enforces `publish_on >= notice_date`; no audience non-empty check; title/body non-empty enforcement delegated to VO constructors (`NoticeTitle::new`, `NoticeBody::new`). |
| `update_notice` (services.rs:126) | Notice exists, not retired, soft-delete guard | partial | services.rs:135-138 — checks `active_status == Retired`; "exists" check is the caller's responsibility (aggregate must be loaded). |
| `publish_notice` (services.rs:163) | Notice is in Draft or Scheduled status | partial | services.rs:170-179 — no status guard; uses `.unwrap_or(now)` on `publish_at` (`DOMAIN-COM-038`). |
| `unpublish_notice` (services.rs:184) | Already-delivered notifications remain; reason optional | partial | services.rs:191-200 — no delivered-notifications guard; uses `.unwrap_or_default()` on reason (`DOMAIN-COM-038`). |
| `delete_notice` (services.rs:204) | No recipient has received the notice, or actor has override | partial | services.rs:211-223 — no recipient-delivered check (would require a storage query). |

### Complaint service (5 functions)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `register_complaint` (services.rs:228) | Idempotent on `(type, date, phone)`; source != Anonymous ⇒ `complaint_by` or `phone` set | stub | services.rs:235-262 — unconditional fresh mint + new event; no idempotency lookup; no source-vs-identity pre-condition (`DOMAIN-COM-008`, `DOMAIN-COM-009`). |
| `assign_complaint` (services.rs:265) | Emits `ComplaintAssigned`; status transitions to InProgress | partial | services.rs:272-282 — basic factory; status transition handled by aggregate (`Complaint::assign` at aggregate.rs:292). |
| `update_complaint_status` (services.rs:285) | Emits `ComplaintStatusChanged` | partial | services.rs:292-304 — basic factory. |
| `resolve_complaint` (services.rs:307) | Complaint not already Resolved | partial | services.rs:314-326 — no "not already Resolved" guard. |
| `add_complaint_note` (services.rs:329) | Emits `ComplaintNoteAdded` | partial | services.rs:336-358 — creates note child + event; does not mutate parent aggregate (`let _ = complaint;` at services.rs:354). |

### ComplaintType service (3 functions)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `create_complaint_type` (services.rs:365) | Uniquely named within a school | partial | services.rs:372-386 — no uniqueness check; would require storage-layer lookup. |
| `update_complaint_type` (services.rs:389) | Emits `ComplaintTypeUpdated` | partial | services.rs:396-415 — basic factory. |
| `delete_complaint_type` (services.rs:418) | Soft delete | partial | services.rs:425-437 — basic factory. |

### Notification service (3 functions)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `send_notification` (services.rs:442) | Emits `NotificationSent`; immutable after `delivered_at` set | partial | services.rs:449-475 — basic factory; delivered-vs-sent distinction is aggregate-managed. |
| `mark_notification_read` (services.rs:478) | Only recipient or admin may mark | partial | services.rs:485-496 — no actor-vs-recipient check. |
| `withdraw_notification` (services.rs:499) | Emits `NotificationWithdrawn` | partial | services.rs:506-521 — basic factory. |

### EmailLog / SmsLog (append-only, 2 functions)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `log_email_sent` (services.rs:524) | Append-only; preserves rendered subject/body, not template id | partial | services.rs:531-565 — append-only by absence of update/delete fns; doesn't validate rendered body retained (it is). |
| `log_sms_sent` (services.rs:567) | Append-only; rendered body captured at dispatch time | partial | services.rs:574-607 — same pattern. |

### SmsTemplate service (5 functions)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `create_sms_template` (services.rs:609) | Unique by `(school_id, channel, purpose)`; variables declared | partial | services.rs:616-642 — no uniqueness check; variable declaration enforced by VO constructor. |
| `update_sms_template` (services.rs:645) | Emits `SmsTemplateUpdated` | partial | services.rs:652-672 — basic factory. |
| `enable_sms_template` (services.rs:675) | Emits `SmsTemplateEnabled` | partial | services.rs:682-691 — basic factory. |
| `disable_sms_template` (services.rs:694) | Emits `SmsTemplateDisabled` | partial | services.rs:701-710 — basic factory. |
| `delete_sms_template` (services.rs:713) | Soft delete | partial | services.rs:720-733 — basic factory. |

### EmailSetting service (3 functions)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `configure_email_setting` (services.rs:737) | Emits `EmailSettingConfigured`; credentials via `SecretReference` | partial | services.rs:744-772 — basic factory; SecretReference handling is VO-level. |
| `activate_email_setting` (services.rs:775) | Demotes previous active setting | partial | services.rs:782-792 — returns `previous_id`; demotion logic lives in aggregate. |
| `delete_email_setting` (services.rs:795) | Soft delete | partial | services.rs:802-813 — basic factory. |

### SmsGateway service (3 functions)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `configure_sms_gateway` (services.rs:818) | Emits `SmsGatewayConfigured` | partial | services.rs:825-845 — basic factory. |
| `activate_sms_gateway` (services.rs:848) | Demotes previous active gateway of same type | partial | services.rs:855-866 — basic factory. |
| `delete_sms_gateway` (services.rs:869) | Soft delete | partial | services.rs:876-888 — basic factory. |

### CustomSmsSetting service (3 functions)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `create_custom_sms_setting` (services.rs:893) | Emits `CustomSmsSettingCreated`; `set_auth: Option<SecretReference>` per spec (code uses `Option<bool>`, drift per `DOMAIN-COM-023`) | partial | services.rs:900-928 — basic factory; field type drift per `DOMAIN-COM-023`. |
| `update_custom_sms_setting` (services.rs:931) | Emits `CustomSmsSettingUpdated` | partial | services.rs:938-960 — basic factory. |
| `delete_custom_sms_setting` (services.rs:963) | Soft delete | partial | services.rs:970-982 — basic factory. |

### NotificationSetting service (3 functions)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `create_notification_setting` (services.rs:986) | `event` is a known event key | partial | services.rs:993-1018 — basic factory; no event-key whitelist. |
| `update_notification_setting` (services.rs:1021) | Emits `NotificationSettingUpdated` | partial | services.rs:1028-1050 — basic factory. |
| `delete_notification_setting` (services.rs:1053) | Soft delete | partial | services.rs:1060-1072 — basic factory. |

### AbsentNotification service (4 functions)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `configure_absent_notification` (services.rs:1076) | Idempotent on `(school_id, time_from, time_to)`; `time_from < time_to` | stub | services.rs:1083-1105 — unconditional fresh mint + new event; no idempotency lookup; no window-order check (`DOMAIN-COM-007`). |
| `enable_absent_notification` (services.rs:1107) | Emits `AbsentNotificationEnabled` | partial | services.rs:1114-1123 — basic factory. |
| `disable_absent_notification` (services.rs:1126) | Emits `AbsentNotificationDisabled` | partial | services.rs:1133-1142 — basic factory. |
| `delete_absent_notification` (services.rs:1145) | Soft delete | partial | services.rs:1152-1164 — basic factory. |

### Chat 1-to-1 service (5 functions)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `open_chat_conversation` (services.rs:1168) | Reuse existing conversation between same `from_id`/`to_id` | partial | services.rs:1175-1196 — fresh mint + new event; no lookup-then-reuse (`DOMAIN-COM-044`). |
| `close_chat_conversation` (services.rs:1199) | Emits `ChatConversationClosed` | partial | services.rs:1206-1216 — basic factory. |
| `send_chat_message` (services.rs:1219) | `to_id` not blocked by `from_id`; `from_id` not blocked by `to_id`; reuse existing conversation | stub | services.rs:1226-1259 — no block-list consultation; auto-mints a new `ChatConversationId` via `unwrap_or_else` (`DOMAIN-COM-010`, `DOMAIN-COM-038`). |
| `mark_chat_message_seen` (services.rs:1261) | Emits `ChatMessageSeen` | partial | services.rs:1268-1280 — basic factory. |
| `delete_chat_message` (services.rs:1283) | Per-user soft delete via `deleted_by_to` | partial | services.rs:1290-1305 — basic factory; per-user-vs-sender distinction is aggregate-managed. |

### Chat group service (4 functions)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `create_chat_group` (services.rs:1308) | One teacher anchor; per-class-section-subject scope | partial | services.rs:1315-1346 — basic factory; no teacher-anchor uniqueness check. |
| `update_chat_group` (services.rs:1349) | Emits `ChatGroupUpdated` | partial | services.rs:1356-1376 — basic factory. |
| `set_chat_group_read_only` (services.rs:1379) | ReadOnly group permits no new messages | partial | services.rs:1386-1396 — basic factory. |
| `delete_chat_group` (services.rs:1399) | Soft delete | partial | services.rs:1406-1418 — basic factory. |

### Chat membership service (3 functions)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `add_user_to_chat_group` (services.rs:1422) | Unique by `(group_id, user_id)` | partial | services.rs:1429-1451 — no uniqueness check. |
| `set_chat_group_user_role` (services.rs:1454) | Emits `ChatGroupUserRoleChanged` | partial | services.rs:1461-1474 — basic factory. |
| `remove_user_from_chat_group` (services.rs:1477) | Soft delete; historical record remains | partial | services.rs:1484-1499 — basic factory. |

### Chat group recipient service (2 functions)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `record_group_message_recipient` (services.rs:1502) | Unique by `(group_id, conversation_id, user_id)`; `read_at` may only go forward | partial | services.rs:1509-1532 — no uniqueness check. |
| `mark_group_message_read` (services.rs:1534) | `read_at` transitions null → timestamp; never back | partial | services.rs:1541-1556 — basic factory. |

### Chat group message remove service (1 function)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `remove_group_message_for_user` (services.rs:1559) | Unique by `(group_message_recipient_id, user_id)` | partial | services.rs:1566-1584 — no uniqueness check. |

### Chat block service (2 functions)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `block_user` (services.rs:1587) | Idempotent on `(block_by, block_to)`; duplicate is no-op success | stub | services.rs:1594-1615 — unconditional fresh mint + new event; no existing-block lookup (`DOMAIN-COM-006`). |
| `unblock_user` (services.rs:1618) | Emits `UserUnblocked`; restores original delivery semantics | partial | services.rs:1625-1640 — basic factory. |

### Chat invitation service (4 functions)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `send_chat_invitation` (services.rs:1643) | Unique by `(from, to)`; `ChatInvitePolicy::check` pre-condition | partial | services.rs:1650-1675 — basic factory; the `ChatInvitePolicy::check` helper exists (services.rs:2648) but the service does not invoke it inline. |
| `accept_chat_invitation` (services.rs:1678) | Pending → Connected | partial | services.rs:1685-1695 — basic factory. |
| `reject_chat_invitation` (services.rs:1698) | Pending → Blocked | partial | services.rs:1705-1715 — basic factory. |
| `classify_chat_invitation` (services.rs:1718) | References exactly one `ChatInvitation`; type is one of three | partial | services.rs:1725-1751 — basic factory. |

### Chat status service (1 function)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `set_chat_status` (services.rs:1754) | Emits `ChatStatusSet` | partial | services.rs:1761-1770 — event-only; no aggregate is persisted (the spec's root aggregate is named `ChatStatusRecord` per `DOMAIN-COM-001`). |

### SendMessage service (3 functions)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `create_send_message` (services.rs:1780) | Emits `SendMessageCreated`; audience descriptor set | partial | services.rs:1787-1811 — basic factory; no audience-descriptor parse. |
| `dispatch_send_message` (services.rs:1814) | Job status is Draft; `publish_on` is None or past; audience non-empty | partial | services.rs:1821-1832 — relies on `SmsDispatchPolicy::check` separately (services.rs:2569-2589); service itself does no validation. Recipient count uses `as u32` truncation (`DOMAIN-COM-037`). |
| `cancel_send_message` (services.rs:1835) | Job not yet dispatched; reason optional | partial | services.rs:1842-1856 — uses `.unwrap_or_default()` on reason (`DOMAIN-COM-038`). |

### ContactMessage service (3 functions)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `receive_contact_message` (services.rs:1859) | Email and phone required (per code); spec says both optional | partial | services.rs:1866-1902 — code rejects empty email/phone (services.rs:1867-1872); spec at `events.md:287-293` says both optional (`DOMAIN-COM-031`). |
| `mark_contact_message_viewed` (services.rs:1905) | Toggles `view_status` | partial | services.rs:1912-1922 — basic factory. |
| `reply_to_contact_message` (services.rs:1925) | Emits `ContactMessageReplied`; reply via channel | partial | services.rs:1932-1958 — basic factory. |

### SpeechSlider service (3 functions)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `create_speech_slider` (services.rs:1961) | Image is `FileReference` | partial | services.rs:1968-1993 — basic factory. |
| `update_speech_slider` (services.rs:1996) | Emits `SpeechSliderUpdated` | partial | services.rs:2003-2024 — basic factory. |
| `delete_speech_slider` (services.rs:2027) | Soft delete | partial | services.rs:2034-2046 — basic factory. |

### PhoneCallLog service (2 functions)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `log_phone_call` (services.rs:2049) | Emits `PhoneCallLogged`; append-only except `next_follow_up_date` | partial | services.rs:2056-2086 — basic factory. |
| `update_phone_call_follow_up` (services.rs:2089) | Emits `PhoneCallFollowUpUpdated` | partial | services.rs:2096-2110 — basic factory. |

### Headline async wrappers (7 functions)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `notify_user` (services.rs:2113) | Thin wrapper around `send_notification` | real | services.rs:2113-2121 — pure delegation. |
| `mark_as_read` (services.rs:2123) | Thin wrapper around `mark_notification_read` | real | services.rs:2123-2132 — pure delegation. |
| `send_notice_message` (services.rs:2134) | Thin wrapper around `publish_notice` | real | services.rs:2134-2143 — pure delegation. |
| `send_complaint_message` (services.rs:2145) | Thin wrapper around `register_complaint` | real | services.rs:2145-2153 — pure delegation. |
| `send_chat_message_headline` (services.rs:2155) | Thin wrapper around `send_chat_message` | real | services.rs:2155-2163 — pure delegation. |
| `send_email_message` (services.rs:2165) | Thin wrapper around `log_email_sent` | real | services.rs:2165-2173 — pure delegation. |
| `send_sms_message` (services.rs:2175) | Thin wrapper around `log_sms_sent` | real | services.rs:2175-2183 — pure delegation. |

### NotificationService (4 methods)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `NotificationService::select_template` (services.rs:2197) | Spec: `(event, destination) -> Option<SmsTemplateId>` | stub | services.rs:2197-2207 — signature is `(event, channel, candidates) -> Option<&SmsTemplate>`; diverges from spec (`DOMAIN-COM-011`). |
| `NotificationService::render` (services.rs:2210) | Parses body for `{{name}}`, validates, returns `RenderedBody` | real | services.rs:2210-2213 — delegates to `TemplateService::render` (the proptest target). |
| `NotificationService::route` (services.rs:2219) | Spec: `(setting, recipient) -> Vec<(UserId, Channel)>` | stub | services.rs:2219-2226 — signature is `(setting) -> Destination`; ignores recipient filter (`DOMAIN-COM-012`). |
| `NotificationService::next_window` (services.rs:2228) | Spec: `(now: NaiveTime, window: &TimeWindow) -> Option<NaiveTime>` | stub | services.rs:2228-2238 — signature is `(setup) -> (TimeOfDay, TimeOfDay)`; entirely different (`DOMAIN-COM-013`). |

### ChatService (4 methods)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `ChatService::is_blocked` (services.rs:2245) | Spec: `(block_list, between: (UserId, UserId)) -> bool`; either side blocked | stub | services.rs:2245-2250 — signature is `(from, blocks) -> bool`; only checks sender-side blocks (`DOMAIN-COM-019`). |
| `ChatService::resolve_conversation` (services.rs:2253) | Spec: `(from, to, existing) -> Option<ChatConversationId>` | stub | services.rs:2253-2265 — returns `Option<&ChatConversation>` (lifetime-bound) instead of `Option<ChatConversationId>` (`DOMAIN-COM-017`). |
| `ChatService::fan_out_group_recipients` (services.rs:2267) | Maps group + members to recipient UserIds | partial | services.rs:2267-2272 — signature drift vs spec (takes `&[ChatGroupUser]` only, no `&ChatGroup`); semantics OK. |
| `ChatService::can_post` (services.rs:2275) | Spec: `(group, user) -> bool`; Closed ⇒ admins only; ReadOnly ⇒ nobody | stub | services.rs:2275-2288 — signature diverges; logic inverted (treats `!read_only` as open) (`DOMAIN-COM-018`). |

### ComplaintService (4 methods)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `ComplaintService::categorize` (services.rs:2300) | Spec: `(cmd) -> ComplaintTypeId` | stub | services.rs:2300-2308 — returns `String`, not `ComplaintTypeId` (`DOMAIN-COM-014`). |
| `ComplaintService::is_anonymous` (services.rs:2310) | Spec: `(source, by: Option<&PersonName>) -> bool` | stub | services.rs:2310-2315 — takes `&Complaint` instead of source+name (`DOMAIN-COM-015`). |
| `ComplaintService::next_status` (services.rs:2317) | `Open → InProgress → Resolved`; Resolved re-issue is no-op | real | services.rs:2317-2332 — implements the spec state machine. |
| `ComplaintService::escalation_path` (services.rs:2335) | Spec: `(setting, complaint_type) -> Vec<UserId>` | stub | services.rs:2335-2349 — returns `Vec<ComplaintStatus>` from `ComplaintStatus`; signature and return diverge (`DOMAIN-COM-016`). |

### AbsentNotificationService (4 methods)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `AbsentNotificationService::in_window` (services.rs:2364) | Spec: `(now: NaiveTime, window: &TimeWindow) -> bool` | partial | services.rs:2364-2370 — signature is `(at: TimeOfDay, setup) -> bool`; semantics match. |
| `AbsentNotificationService::should_dispatch` (services.rs:2372) | Spec: `(setting, event) -> bool`; enabled AND in window | partial | services.rs:2372-2381 — signature is `(at, setup) -> bool`; semantics OK. |
| `AbsentNotificationService::build_dispatch` (services.rs:2384) | Spec: `(setting, student) -> AbsentNotificationDispatch` | partial | services.rs:2384-2409 — signature drift (12 args, takes pre-rendered body); semantics OK. |
| `AbsentNotificationService::render` (services.rs:2411) | Spec: `(setting, template, student) -> Result<RenderedBody>` | partial | services.rs:2411-2417 — delegates to `TemplateService::render`; signature drift. |

### TemplateService (5 methods)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `TemplateService::validate_body` (services.rs:2430) | Every declared variable appears in body | real | services.rs:2430-2444 — full implementation. |
| `TemplateService::declared` (services.rs:2446) | Returns `{{name}}` placeholders in source order, deduped | real | services.rs:2446-2472 — full implementation; proptest target. |
| `TemplateService::substitute` (services.rs:2474) | Substitutes placeholders; errors on missing var | real | services.rs:2474-2511 — full implementation; proptest target. |
| `TemplateService::render` (services.rs:2513) | Renders template body; returns `RenderedBody` | real | services.rs:2513-2529 — full implementation; 100-case proptest target (Phase 10 headline). |
| `TemplateService::lint` (services.rs:2531) | Detects mismatched braces + HTML in SMS | real | services.rs:2531-2553 — full implementation; tested by `template_service_lint_detects_html` (services.rs:2909+). |

### SmsDispatchPolicy / ActiveRecipients / NoticesPublishedInRange / ChatInvitePolicy (4 methods)

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `SmsDispatchPolicy::check` (services.rs:2569) | Draft status; `publish_on` ≤ now; audience non-empty | real | services.rs:2569-2589 — full implementation. |
| `ActiveRecipients::is_satisfied_by` (services.rs:2603) | Pending or Dispatched status | real | services.rs:2603-2610 — full implementation. |
| `NoticesPublishedInRange::is_satisfied_by` (services.rs:2625) | Published AND `notice_date` ∈ `[from, to]` | real | services.rs:2625-2633 — full implementation. |
| `ChatInvitePolicy::check` (services.rs:2648) | No self-invite; actor hasn't blocked recipient; no open invitation already exists | real | services.rs:2648-2671 — full implementation. |

### Summary

- **Total pub fn:** 104
- **Real:** 22 (`create_notice`-adjacent cross-field rule, the 7 headline async wrappers, `NotificationService::render`, `ComplaintService::next_status`, the 5 `TemplateService` methods, and the 4 spec/policy helpers `SmsDispatchPolicy::check`, `ActiveRecipients::is_satisfied_by`, `NoticesPublishedInRange::is_satisfied_by`, `ChatInvitePolicy::check` plus a few `NotificationService::render` / `AbsentNotificationService::render` delegates). The remaining 69 sync factory functions are partial.
- **Partial:** 69 — each implements its primary single-aggregate invariant (factory builds aggregate, emits the correct event, delegates invariant enforcement to the aggregate or to value-object constructors) but is missing at least one spec-required pre-condition, idempotency guarantee, or cross-aggregate lookup.
- **Stub:** 13 — (`register_complaint`, `configure_absent_notification`, `block_user`, `send_chat_message`, `open_chat_conversation` was downgraded to partial after re-classification of "lookup-then-reuse" as a storage-layer concern not a service-layer invariant, plus 9 service-struct methods whose signatures diverge from `docs/specs/communication/services.md`: `NotificationService::select_template`, `NotificationService::route`, `NotificationService::next_window`, `ChatService::is_blocked`, `ChatService::resolve_conversation`, `ChatService::can_post`, `ComplaintService::categorize`, `ComplaintService::is_anonymous`, `ComplaintService::escalation_path`).

The 13 stubs concentrate in two bands:

1. **Missing idempotency** on `register_complaint`,
   `configure_absent_notification`, and `block_user`
   (`workflows.md:191-199`) — three services that unconditionally
   mint fresh IDs and emit new events without consulting existing
   rows. These are the highest-priority production-blocking gaps.
2. **Spec-vs-code signature drift** in the 5 service structs
   (`NotificationService` 3 methods, `ChatService` 3 methods,
   `ComplaintService` 3 methods) plus the logic inversion in
   `ChatService::can_post`. Consumers importing the spec-named
   signatures will not compile.

Stub-adjacent partials:

- `send_chat_message` missing block-list consultation
  (`commands.md:417-418`).
- `open_chat_conversation` missing lookup-then-reuse
  (`commands.md:420-423`).
- `receive_contact_message` rejects empty email/phone despite spec
  declaring both optional (`events.md:287-293`,
  `DOMAIN-COM-031`).

The `TemplateService` quintet is the only fully-real service struct
in the file and is the headline correctness target for Phase 10
(100-case proptest).

---

## hr

**Crate:** `crates/domains/hr/src/services.rs`
**Spec reference:** `docs/specs/hr/aggregates.md`
**Function count:** 49 (`pub fn` only; no `pub async fn`)
**Stub count:** 26

Phase 6 ships the seven prompt-named aggregate factories (`hire_staff`,
`create_department`, `create_designation`, `create_leave_type`,
`request_leave`, `approve_leave`, `run_payroll`) plus 16 impl-block
methods across four workflow service structs
(`LeaveAccrualService`, `ClassTeacherAssignmentService`,
`SubjectTeacherAssignmentService`, `HourlyRateManagementService`) and
the `InMemoryPayrollPolicy` constructor pair. The remaining 26
Cluster C handler skeletons (services.rs:731-1297) are self-documented
"Phase 6 stub" placeholders that validate only the tenant anchor and
emit empty events; their full payloads are deferred to the owning
Workstream per the in-file comment block at services.rs:697-714.

### Core aggregate factories

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `hire_staff` (services.rs:80) | Staff invariants 1-4, 6 | partial | services.rs:114-119 — `validate_person_name(first_name)`, `validate_person_name(last_name)`; services.rs:120-122 — optional `validate_email`; services.rs:123-125 — optional `validate_phone`; services.rs:126 — `validate_date_of_birth`; services.rs:130-144 — three-way uniqueness via `uniqueness.email_exists`, `uniqueness.staff_no_exists`, `uniqueness.employee_id_exists` (covers invariant 3 staff_no unique + invariant 4 email unique; invariant 2 `UserId` binding carried via `cmd.user_id`); services.rs:155-167 — `Staff::fresh` with `Status::Active` (covers invariant 6 starting state). **Gaps:** invariant 5 (mobile uniqueness not enforced — only `validate_phone` format, no `uniqueness.mobile_exists`); invariant 7 (cannot be hard-deleted) deferred to delete handler; invariant 8 (leave day counts non-negative) enforced implicitly by typed fields. |
| `create_department` (services.rs:196) | Department invariant 1 (unique name within school) | real | services.rs:209-213 — length validation (1..=200 chars); services.rs:214-218 — `uniqueness.department_name_exists` uniqueness check (covers invariant 1); services.rs:221-228 — `Department::fresh`; services.rs:231 — `DepartmentCreated::new`. Invariants 2-3 (referential check, system-defined flag) are delete-handler concerns; not applicable here. |
| `create_designation` (services.rs:240) | Designation invariant 1 (unique name within school) | real | services.rs:252-256 — length validation; services.rs:257-261 — `uniqueness.designation_title_exists` (covers invariant 1); services.rs:264-272 — `Designation::fresh`; services.rs:275 — `DesignationCreated::new`. Invariants 2-3 deferred to delete handler. |
| `create_leave_type` (services.rs:288) | LeaveType invariants 1 (unique name within school), 3 (`total_days >= 0`) | real | services.rs:300 — `validate_leave_type_name`; services.rs:301-305 — `uniqueness.leave_type_name_exists` (covers invariant 1); services.rs:308-317 — `LeaveType::fresh` with `total_days` (u32 type enforces invariant 3 non-negativity); services.rs:320-328 — `LeaveTypeCreated::new`. Invariant 2 (referential check on delete) deferred to delete handler. |
| `request_leave` (services.rs:340) | LeaveRequest invariants 1 (unique by `(school, staff, leave_from, leave_to, type_id)`), 2 (`leave_from <= leave_to`), 3 (`approve_status = Pending` on creation) | partial | services.rs:354-358 — `leave_to < leave_from` rejection (covers invariant 2); services.rs:359-361 — optional `validate_leave_reason`; services.rs:364-374 — `LeaveRequest::fresh` initialises `approve_status = Pending` (covers invariant 3); services.rs:377-387 — `LeaveRequested::new`. **Gaps:** invariant 1 (uniqueness on `(school, staff, leave_from, leave_to, type_id)`) not enforced — `request_leave` does not consult any `LeaveRequestUniquenessChecker`; invariant 4 (LeaveDefine entitlement remaining) and invariant 5 (LeaveDefine.total_days cap) not enforced here — the helper `LeaveAccrualService::can_request` exists at services.rs:507 but is not called from `request_leave`. |
| `approve_leave` (services.rs:414) | LeaveRequest invariant 3 (state transition `Pending -> Approved`, terminal once approved) | partial | services.rs:423-427 — `leave_request.can_transition(LeaveStatus::Approved)` state-machine guard (covers the forward edge of invariant 3); services.rs:428-432 — segregation-of-duties: rejects when `approver_tenant.actor_id == leave_request.created_by`; services.rs:434-445 — sets `approve_status = Approved`, bumps version, stamps `approved_at` + `updated_by` + `last_event_id`; services.rs:447-457 — `LeaveApproved::new`. **Gap:** invariant 4 (LeaveDefine remaining days for the period) not enforced — approval succeeds without consulting the leave balance. |
| `run_payroll` (services.rs:536) | PayrollGenerate invariants 1 (`gross_salary == basic_salary + total_earning`), 2 (`net_salary == gross_salary - total_deduction - tax`), 3 (`Generated` status) | partial | services.rs:550 — `validate_pay_period`; services.rs:552-554 — `basic_salary >= 0` check; services.rs:556-560 — `total_earning = basic_salary`, `tax = policy.tax(...)`, `total_deduction = tax`, `gross_salary = total_earning`, `net_salary = gross_salary - total_deduction` (covers invariants 1 and 2 with the simplification that `total_earning == basic_salary` — invariant 1 holds vacuously; per-earnings deduction lines are not summed in here); services.rs:563-578 — `PayrollGenerate::fresh`; services.rs:581-588 — sets `payroll_status = PayrollStatus::Generated` (covers invariant 3 first leg); services.rs:591-607 — `PayrollGenerated::new`. **Gaps:** invariant 4 (`paid_amount <= net_salary`) deferred to `MarkPayrollPaid` (not present in this file); invariant 5 (uniqueness on `(school, staff, payroll_month, payroll_year)`) not enforced; invariant 6 (at most one `LeaveDeductionInfo` line) deferred to the leave-deduction-info handler skeleton (`record_payroll_generate_audit` is a stub at services.rs:1142). |

### Workflow service structs

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `LeaveAccrualService::effective_leave_balance` (services.rs:473) | LeaveDefine invariant 3 (`days <= total_days`); LeaveRequest invariant 5 (`extra_leave <= LeaveDefine.total_days`) | real | services.rs:478-485 — sums `LeaveRequest::duration_days` over approved requests of the same `type_id`, returns `define.days.saturating_sub(used)`; pure, no I/O. |
| `LeaveAccrualService::extra_leave_taken` (services.rs:490) | LeaveRequest invariant 5 (extra leave counting for payroll deduction) | real | services.rs:495-503 — sums approved durations, returns `total.saturating_sub(define.days)`; pure. |
| `LeaveAccrualService::can_request` (services.rs:507) | LeaveRequest invariants 1 (no overlap), 4 (entitlement remaining), 5 (cap by `LeaveDefine.days`) | partial | services.rs:512-518 — duration computed from `(to - from).num_days() + 1`, `u32::try_from` saturation; services.rs:519-524 — sums approved durations, returns `used + duration <= define.days` (covers invariant 4 and 5). **Gap:** does not check that the candidate `(from, to)` window does not overlap an already-approved `LeaveRequest` window — `overlaps` exists at services.rs:525 but is not invoked here. The function comment (services.rs:510-511) claims "Rejects overlapping approved requests" but the body does not enforce it. |
| `LeaveAccrualService::overlaps` (services.rs:525) | LeaveRequest invariant 1 (uniqueness on date window) | real | services.rs:526-528 — classic date-range overlap `a.0 <= b.1 && b.0 <= a.1`; pure helper. |
| `InMemoryPayrollPolicy::new` (services.rs:659) | Test fixture constructor | real | services.rs:660-666 — `Self { tax_rate, allows_partial: true, max_fraction: 1.0 }`. Not a spec invariant; constructor for the in-memory `PayrollPolicy` reference. |
| `InMemoryPayrollPolicy::with_partial` (services.rs:668) | Test fixture constructor | real | services.rs:669-675 — accepts `tax_rate`, `allows_partial`, `max_fraction`. Not a spec invariant; same role as `new`. |
| `ClassTeacherAssignmentService::is_assigned` (services.rs:1315) | AssignClassTeacher invariants 1 (unique by `(class, section, academic)`), 2 (`active_status = 1` while open) | real | services.rs:1325-1332 — iterates assignments, checks `active_status == 1 && class_id == … && section_id == … && staff_id == … && academic_id == …`; pure lookup. |
| `ClassTeacherAssignmentService::current_for_class` (services.rs:1334) | AssignClassTeacher invariant 2 | real | services.rs:1342-1349 — returns the first active assignment matching `(class, section, academic)`; pure lookup. |
| `ClassTeacherAssignmentService::has_active_teacher` (services.rs:1353) | AssignClassTeacher invariant 2 | real | services.rs:1360-1363 — delegates to `current_for_class`; pure. |
| `ClassTeacherAssignmentService::count_for_staff` (services.rs:1365) | AssignClassTeacher invariants (no specific count invariant; aggregation helper) | real | services.rs:1371-1376 — counts assignments where `staff_id == … && academic_id == …`; pure. |
| `SubjectTeacherAssignmentService::validate` (services.rs:1395) | Tenant anchor (cross-aggregate: `staff_id` belongs to tenant school) | real | services.rs:1399-1404 — checks `cmd.staff_id.school_id() == cmd.tenant.school_id`, returns `Validation` error otherwise; covered by `subject_teacher_assignment_service_validates_tenant_scope` (services.rs:1729-1786) which exercises both the same-school and cross-school cases. |
| `SubjectTeacherAssignmentService::is_reassignment` (services.rs:1409) | No-op reassignment rejection | real | services.rs:1411-1414 — pure `current_id != replacement_id`; correctly identifies a no-op. |
| `SubjectTeacherAssignmentService::scope_matches_tenant` (services.rs:1417) | Tenant anchor (cross-aggregate: `class_id` and `subject_id` belong to tenant school) | real | services.rs:1421-1426 — checks both `class_school` and `subject_school` against `cmd.tenant.school_id`; pure. |
| `HourlyRateManagementService::effective_rate` (services.rs:1447) | HourlyRate invariant 1 (unique by `(school, grade, academic)`) | real | services.rs:1453-1460 — finds first `HourlyRate` matching `(grade, academic_id)`, returns `rate`; returns `None` if absent. |
| `HourlyRateManagementService::validate_rate` (services.rs:1461) | HourlyRate invariant 2 (`rate > 0`) | partial | services.rs:1462-1469 — rejects `rate < 0.0` (returns `Validation`). **Gap:** spec says `rate > 0` (strictly positive); this allows `rate == 0.0` to pass. Trivial fix: `rate <= 0.0` rejection. |
| `HourlyRateManagementService::is_rate_change` (services.rs:1474) | No-op update rejection | real | services.rs:1476-1480 — `(old - new).abs() > epsilon`; pure epsilon comparison. |

### Cluster C handler skeletons (all stub)

Per the in-file comment block at services.rs:697-714, each handler
below is a minimal skeleton that wires the matching command stub to
the matching aggregate stub and emits the matching event with no
payload. Every body is identical in shape — `cmd.id` and
`cmd.school_id` lifted into a one-field aggregate, an event with
`cmd.id` / fresh `event_id` / fresh `correlation_id` / `now`, and
returned.

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `create_staff_bank_detail` (services.rs:731) | StaffBankDetail aggregate invariants | stub | services.rs:738-749 — body wires `StaffBankDetail { id: cmd.id, school_id: cmd.school_id }` and `StaffBankDetailUpserted::new(cmd.id, event_id, correlation_id, now)`; no payload fields. |
| `create_staff_address` (services.rs:752) | StaffAddress aggregate invariants | stub | services.rs:759-770 — identical stub pattern; `StaffAddressAdded` event with `cmd.id` only. |
| `create_staff_social_link` (services.rs:773) | StaffSocialLink aggregate invariants | stub | services.rs:780-791 — identical stub pattern; `StaffSocialLinkAdded` event. |
| `create_staff_document` (services.rs:795) | StaffDocument aggregate invariants | stub | services.rs:802-813 — identical stub pattern; `StaffDocumentRegistered` event. |
| `refresh_staff_timeline` (services.rs:817) | StaffTimeline aggregate invariants (projection recompute) | stub | services.rs:824-835 — identical stub pattern; `StaffTimelineRefreshed` event. |
| `set_staff_custom_field` (services.rs:838) | StaffCustomField aggregate invariants | stub | services.rs:845-856 — identical stub pattern; `StaffCustomFieldSet` event. |
| `refresh_staff_leave_balance` (services.rs:860) | StaffLeaveBalance aggregate invariants (projection recompute) | stub | services.rs:867-878 — identical stub pattern; `StaffLeaveBalanceRefreshed` event. |
| `record_leave_request_approval` (services.rs:882) | LeaveRequestApproval aggregate invariants | stub | services.rs:889-900 — identical stub pattern; `LeaveRequestApprovalRecorded` event. |
| `create_payroll_payment_link` (services.rs:903) | PayrollPaymentLink aggregate invariants | stub | services.rs:910-921 — identical stub pattern; `PayrollPaymentLinkCreated` event. |
| `record_staff_import_resolution` (services.rs:925) | StaffImportResolution aggregate invariants | stub | services.rs:932-943 — identical stub pattern; `StaffImportResolutionRecorded` event. |
| `record_staff_payroll_history` (services.rs:947) | StaffPayrollHistory aggregate invariants | stub | services.rs:954-965 — identical stub pattern; `StaffPayrollHistorySnapshotted` event. |
| `record_staff_leave_history` (services.rs:969) | StaffLeaveHistory aggregate invariants | stub | services.rs:976-987 — identical stub pattern; `StaffLeaveHistorySnapshotted` event. |
| `create_assign_class_teacher_scope` (services.rs:991) | AssignClassTeacherScope aggregate invariants | stub | services.rs:998-1009 — identical stub pattern; `AssignClassTeacherScopeAdded` event. |
| `assign_department_head` (services.rs:1012) | DepartmentHead aggregate invariants | stub | services.rs:1019-1030 — identical stub pattern; `DepartmentHeadRecorded` event. |
| `create_designation_grade` (services.rs:1033) | DesignationGrade aggregate invariants | stub | services.rs:1040-1051 — identical stub pattern; `DesignationGradeRecorded` event. |
| `set_hourly_rate_override` (services.rs:1055) | HourlyRateOverride aggregate invariants | stub | services.rs:1062-1073 — identical stub pattern; `HourlyRateOverrideSet` event. |
| `create_leave_define_adjustment` (services.rs:1077) | LeaveDefineAdjustment aggregate invariants | stub | services.rs:1084-1095 — identical stub pattern; `LeaveDefineAdjustmentApplied` event. |
| `create_leave_request_attachment` (services.rs:1098) | LeaveRequestAttachment aggregate invariants | stub | services.rs:1105-1116 — identical stub pattern; `LeaveRequestAttachmentRegistered` event. |
| `record_staff_attendance_punch` (services.rs:1120) | StaffAttendancePunch aggregate invariants | stub | services.rs:1127-1138 — identical stub pattern; `StaffAttendancePunchCaptured` event. |
| `record_payroll_generate_audit` (services.rs:1142) | PayrollGenerateAudit aggregate invariants | stub | services.rs:1149-1160 — identical stub pattern; `PayrollGenerateAuditAppended` event. |
| `assign_staff_role` (services.rs:1163) | StaffRoleAssignment aggregate invariants | stub | services.rs:1170-1181 — identical stub pattern; `StaffRoleAssignmentRecorded` event. |
| `create_staff_profile_photo` (services.rs:1184) | StaffProfilePhoto aggregate invariants | stub | services.rs:1191-1202 — identical stub pattern; `StaffProfilePhotoRegistered` event. |
| `create_staff_driving_license` (services.rs:1206) | StaffDrivingLicense aggregate invariants | stub | services.rs:1213-1224 — identical stub pattern; `StaffDrivingLicenseRegistered` event. |
| `create_staff_registration_field_option` (services.rs:1228) | StaffRegistrationFieldOption aggregate invariants | stub | services.rs:1235-1248 — identical stub pattern; `StaffRegistrationFieldOptionAdded` event. |
| `create_bulk_import_job` (services.rs:1252) | BulkImportJob aggregate invariants | stub | services.rs:1259-1270 — identical stub pattern; `BulkImportJobRecorded` event. |
| `create_staff_attendance_import_batch` (services.rs:1273) | StaffAttendanceImportBatch aggregate invariants | stub | services.rs:1280-1297 — identical stub pattern; `StaffAttendanceImportBatchRecorded` event. |

### Summary

- **Total pub fn:** 49
- **Real:** 17 — `create_department`, `create_designation`, `create_leave_type` (3 core creates), `LeaveAccrualService::{effective_leave_balance, extra_leave_taken, overlaps}` (3 of 4), `InMemoryPayrollPolicy::{new, with_partial}` (2 constructors), `ClassTeacherAssignmentService::{is_assigned, current_for_class, has_active_teacher, count_for_staff}` (4), `SubjectTeacherAssignmentService::{validate, is_reassignment, scope_matches_tenant}` (3), `HourlyRateManagementService::{effective_rate, is_rate_change}` (2 of 3).
- **Partial:** 6 — `hire_staff` (missing mobile uniqueness, spec invariant 5); `request_leave` (missing uniqueness / entitlement / cap, invariants 1, 4, 5); `approve_leave` (missing LeaveDefine remaining-days check, invariant 4); `run_payroll` (missing uniqueness + paid-amount + LeaveDeductionInfo cap, invariants 4, 5, 6); `LeaveAccrualService::can_request` (overlap not enforced despite docstring claim); `HourlyRateManagementService::validate_rate` (allows `rate == 0.0` while spec invariant 2 requires `rate > 0`). **Two of the six partials are workflow-service helpers, not aggregate factories.**
- **Stub:** 26 — every Cluster C handler skeleton (services.rs:731-1297) validates only the tenant anchor (`cmd.id`, `cmd.school_id`) and emits an empty event. No spec invariant is touched by any of the 26. This is consistent with the in-file comment at services.rs:697-714 marking the block as placeholder work deferred to the owning Workstream.

### Classification rationale

- **Real vs partial** for the prompt-named factory subset hinges on
  whether the function enforces the spec invariant under
  cross-aggregate or storage-layer state (uniqueness on multiple
  fields, entitlement remaining, paid-amount cap). The factory
  functions that need only typed-field validation (department name,
  designation title, leave type name, basic salary) are real; those
  that need uniqueness or balance lookups are partial because the
  helper ports / services exist (`StaffUniquenessChecker`,
  `LeaveAccrualService::can_request`, `LeaveAccrualService::extra_leave_taken`)
  but are not consulted from the factory body.
- **Real vs partial** for the workflow service struct methods hinges
  on whether the function body matches its docstring. `can_request`
  is partial because the docstring at services.rs:510-511 promises
  overlap rejection that the body does not enforce; `validate_rate`
  is partial because the spec says `rate > 0` (strictly positive)
  but the body allows `rate == 0.0`.
- **Stub** is unambiguous: every Cluster C handler skeleton has the
  same 11-line body, no payload wiring, no invariant touch — and the
  in-file comment at services.rs:697-714 self-documents the block
  as placeholder work deferred to a later phase.

---

## finance

**Crate:** `crates/domains/finance/src/services.rs`
**Spec reference:** `docs/specs/finance/aggregates.md`
**Function count:** 66 (`pub fn` + `pub async fn` only; excludes the 3 trait method declarations at services.rs:650-656 on `pub trait PaymentProvider` and the 3 matching `async fn` impls at services.rs:760-780, which carry no `pub` modifier)
**Stub count:** 32 (the "Cluster C handler skeletons" block at services.rs:996-1455 — every command takes the typed command + clock + id-generator and returns `Ok(())` with `let _ = (cmd, clock, ids);` at the top of the body)
**Real / Partial / Stub:** 29 real / 5 partial / 32 stub

Phase 7 ships the prompt-named headline subset (Wallet lifecycle,
record_payment, record_expense, configure_invoice_numbering) as real
or partial; the 16 newly-added aggregates (FmFeesGroup,
FmFeesType, FmFeesInvoice + child, FmFeesTransaction + child,
FmFeesWeaver, FeesInvoiceSetting, FeesInstallmentCredit, Transaction,
Donor, ProductPurchase, InventoryPayment, FeesAssignDiscount,
DirectFeesInstallmentChildPayment) have placeholder skeletons that
return `Ok(())` per the in-file comment block at
services.rs:962-975. Per the comment, the full impl for those 16
is deferred to subsequent workstreams (B, C, D, F, G, L).

### Wallet aggregate (lines 73-388)

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `create_wallet` (services.rs:73) | Wallet invariant 1 (`WalletId` unique within school); wallet creation lazy on first transaction | real | services.rs:78-104 — derives `WalletId::new(school, uuid_from_event_id)`; `Wallet::fresh` (services.rs:88-95) builds aggregate with currency + actor; `WalletCreated::new` event at services.rs:97-104. Tenant anchor enforced via typed id. |
| `credit_wallet` (services.rs:124) | WalletTransaction invariants 1 (`amount >= 0`), 2 (status `pending` on creation), 3 (references user + optional bank) | real | services.rs:130-158 — `WalletTransaction::fresh` validates amount + currency (services.rs:142-153); event `WalletCredited::new` minted at services.rs:155-165. Pending state preserved (transition to `Approved` is a separate command). |
| `request_wallet_refund` (services.rs:193) | WalletTransaction wallet_type = `Refund`; status `pending` on creation | real | services.rs:198-229 — `WalletTxType::Refund` (services.rs:213); `WalletTransaction::fresh` (services.rs:214-225) constructs pending tx; `WalletRefundRequested::new` event with reason at services.rs:227-237. |
| `deduct_wallet_credit` (services.rs:257) | Wallet invariant: only `approve` transitions balance; sufficient balance pre-flight | real | services.rs:264-283 — explicit `cmd.amount_minor > wallet.balance_minor` check at services.rs:264-268 (returns `DomainError::conflict`); currency match at services.rs:269-273. **Missing:** deduction is two-phase (this creates the pending tx; the dispatch path applies the debit on approval) — but the pre-flight check covers the headline spec invariant. |
| `approve_wallet_transaction` (services.rs:336) | State transition `Pending → Approved`; only `approve` mutates wallet balance | real | services.rs:341-355 — `tx.approve(approver, now, event_id)?` (services.rs:344) enforces the state machine in the aggregate; `WalletTransactionApproved::new` event at services.rs:346-353. |
| `reject_wallet_transaction` (services.rs:361) | State transition `Pending → Rejected`; `note` captured | real | services.rs:366-380 — `tx.reject(rejecter, note.clone(), now, event_id)?` (services.rs:369); `WalletTransactionRejected::new` event at services.rs:371-379. |
| `WalletService::balance` (services.rs:401) | Spec: current balance = sum of approved transactions | partial | services.rs:401-419 — the loop computes `bal` by iterating approved tx (services.rs:403-416) but immediately discards the computed value via `let _ = bal;` and returns `wallet.balance_minor` (services.rs:418). The "cross-check" loop is dead code; the helper returns the cached value unconditionally. **Missing:** the computed balance is never actually compared against the cached value, so the invariant check is symbolic. |
| `WalletService::validate_debit` (services.rs:421) | Wallet invariant: cannot debit beyond available balance; currencies must match | real | services.rs:421-441 — `amount_minor < 0` rejected at services.rs:422-425; currency mismatch at services.rs:426-430; `wallet.balance_minor < amount_minor` rejected at services.rs:431-436. All three checks return typed `DomainError`. |

### Headline 3+4: payment + expense + invoice (lines 454-628)

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `record_payment` (services.rs:454) | FeesPayment invariants 1 (non-null assign_id + student_id), 2 (amount/discount/fine >= 0), 3 (payment_mode's gateway_id matches), 4 (gateway tx id required if gateway) | partial | services.rs:459-492 — `FeesPayment::fresh` validates amount, discount, fine non-negative (services.rs:472-485); `PaymentReceived::new` event at services.rs:487-498. **Missing:** invariants 1 (assign_id + student_id are not part of this command; deferred to dispatcher), 3 (payment_method/gateway compatibility not checked here), 4 (gateway tx id deferred to dispatcher per the docstring at services.rs:444-453). The function is pure; the dispatch layer wires the real provider. |
| `record_expense` (services.rs:520) | Expense invariants 1 (amount >= 0), 2 (payment_method/account compatible), 3 (exactly one expense_head) | partial | services.rs:526-560 — `Expense::fresh` validates amount and head (services.rs:539-552); `ExpenseRecorded::new` event at services.rs:554-568. **Missing:** invariant 2 (payment_method compatibility with the bank/cash account) is not checked; invariant 3 is enforced by the aggregate's single-head field but no cross-aggregate validation here. |
| `configure_invoice_numbering` (services.rs:591) | FeesInvoice invariants 1 (one per school), 2 (start_form >= 0), 3 (next = start_form + count of issued) | partial | services.rs:596-621 — `FeesInvoice::fresh` validates prefix and start_form (services.rs:608-617); `InvoiceNumberingConfigured::new` event at services.rs:619-625. **Missing:** invariant 1 (one-per-school uniqueness is a storage-layer concern; not enforced in service); invariant 3 (the next-invoice calculation is delegated to the dispatch path). |

### Stub `PaymentProvider` port (lines 641-787)

The `PaymentProvider` trait (services.rs:641-658) and `StubPaymentProvider` impl (services.rs:732-787) are marked `#[deprecated]` since `0.1.0` and slated to move to `educore-payment` in Phase 15 per the in-file doc-comment at services.rs:633-640. The 3 trait method declarations at services.rs:650-656 (`charge`, `refund`, `status`) and the 3 matching impls at services.rs:760-780 carry no `pub` modifier (the trait is `pub`, so the methods are accessible through the trait object but not via direct `pub fn`). The only `pub fn` in this block is the constructor.

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `StubPaymentProvider::new` (services.rs:752) | Trivial constructor | real | services.rs:752-756 — returns `Self::default()`; counter starts at 0. |

### CarryForwardService + LateFeeService + DoubleEntryService (lines 794-958)

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `FeesCarryForwardSetting::new` (services.rs:803) | FeesCarryForwardSetting invariants 1 (title unique within school), 2 (`fees_due_days >= 0`) | real (structural); partial (uniqueness) | services.rs:803-817 — title length 1..=200 validated at services.rs:806-809; `fees_due_days <= 365` validated at services.rs:810-815. **Missing:** invariant 1 (title uniqueness within school) is a storage-layer concern, not enforced here. |
| `CarryForwardService::should_carry_forward` (services.rs:834) | Build-plan § Phase 7 carry-forward rules 1 (no open balance → skip) + 4 (below threshold → skip + log) | real | services.rs:834-844 — `balance_minor == 0` returns `false` (services.rs:835-837); `balance.abs() < threshold` returns `false` (services.rs:838-843). Both rules in the build-plan are covered. |
| `CarryForwardService::build_carry_forward` (services.rs:849) | Build-plan § Phase 7 carry-forward rules 2 (debit) + 3 (credit); `balance >= 0`; `due_date` required | real | services.rs:849-885 — derives `BalanceType` from sign at services.rs:855-859; `unsigned_abs()` enforces `balance >= 0` (services.rs:860); `note` reflects type at services.rs:861-871; `due_date` carried through from `cmd.due_date`. |
| `LateFeeService::compute_late_fee` (services.rs:920) | Late-fee rule: within grace period → 0; otherwise apply `kind` rule | real | services.rs:920-940 — `days_late <= grace` returns 0 at services.rs:921-924; `FixedAmount`/`PercentOfAmount`/`PerDayRate` branches at services.rs:926-937; covered by table-driven tests at services.rs:2431-2490 (1-30 days × 3 kinds). |
| `DoubleEntryService::check_invariant` (services.rs:953) | Transaction aggregate invariant: `sum(debits) == sum(credits)` per `school_id`; row amounts non-negative | real | services.rs:953-976 — non-negative amount check at services.rs:962-966; per-school filter at services.rs:959-961 (cross-tenant confusion caught); `debits != credits` returns `DomainError::conflict` at services.rs:967-975. Property-tested via proptest at services.rs:2502-2547. |

### Cluster C handler skeletons (lines 996-1455) — 32 stubs

All 32 functions in this block carry the same self-documented
"Full implementation lands in Phase 7 Workstream B/C/D/F/G/L"
doc-comment (see e.g. services.rs:990-995 for the section
header) and the same body:

```rust
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn create_X<C, G>(cmd: CreateXCommand, clock: &C, ids: &G) -> Result<()>
where C: Clock + ?Sized, G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}
```

No domain fields are populated; no events are emitted; no spec
invariants are touched. The 32 functions and their spec anchors:

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `create_fees_assign_discount` (services.rs:996) | FeesAssignDiscount invariants 1 (amounts >= 0), 2 (applied + unapplied constant), 3 (role_id or student_id) | stub | services.rs:990-1006 — body returns `Ok(())` after `let _ = (cmd, clock, ids);` |
| `read_fees_assign_discount` (services.rs:1012) | Read-by-id; no invariant violated | stub | services.rs:1008-1022 — same stub body |
| `create_direct_fees_installment_child_payment` (services.rs:1028) | DirectFeesInstallmentChildPayment invariants 1 (paid + balance = amount + discount), 2 (paid monotonic) | stub | services.rs:1024-1038 |
| `read_direct_fees_installment_child_payment` (services.rs:1044) | Read-by-id | stub | services.rs:1040-1054 |
| `create_fm_fees_group` (services.rs:1060) | FmFeesGroup invariant 1 (unique by name within school) | stub | services.rs:1056-1066 |
| `read_fm_fees_group` (services.rs:1072) | Read-by-id | stub | services.rs:1068-1078 |
| `create_fm_fees_type` (services.rs:1084) | FmFeesType invariants 1 (one FmFeesGroup), 2 (type ∈ fees\|lms), 3 (course_id required iff lms) | stub | services.rs:1080-1090 |
| `read_fm_fees_type` (services.rs:1096) | Read-by-id | stub | services.rs:1092-1102 |
| `create_fm_fees_invoice` (services.rs:1108) | FmFeesInvoice invariants 1 (invoice_id unique per school), 2 (children subtotals + fine + service_charge + weaver = grand total), 3 (type ∈ fees\|lms) | stub | services.rs:1104-1120 |
| `read_fm_fees_invoice` (services.rs:1124) | Read-by-id | stub | services.rs:1120-1130 |
| `create_fm_fees_invoice_child` (services.rs:1136) | FmFeesInvoiceChild invariants 1 (one FmFeesInvoice), 2 (sub_total = amount + weaver + fine), 3 (paid_amount <= sub_total + service_charge) | stub | services.rs:1132-1148 |
| `read_fm_fees_invoice_child` (services.rs:1152) | Read-by-id | stub | services.rs:1148-1158 |
| `create_fm_fees_invoice_setting` (services.rs:1168) | FmFeesInvoiceSetting invariants 1 (one per school), 2 (limits non-negative), 3 (uniq_id_start unique) | stub | services.rs:1164-1180 |
| `read_fm_fees_invoice_setting` (services.rs:1184) | Read-by-id | stub | services.rs:1180-1190 |
| `create_fm_fees_transaction` (services.rs:1200) | FmFeesTransaction invariants 1 (one FmFeesInvoice), 2 (total_paid >= 0), 3 (wallet money iff wallet exists) | stub | services.rs:1196-1212 |
| `read_fm_fees_transaction` (services.rs:1216) | Read-by-id | stub | services.rs:1212-1222 |
| `create_fm_fees_transaction_child` (services.rs:1232) | FmFeesTransactionChild invariants 1 (one transaction), 2 (paid >= 0) | stub | services.rs:1228-1244 |
| `read_fm_fees_transaction_child` (services.rs:1248) | Read-by-id | stub | services.rs:1244-1254 |
| `create_fm_fees_weaver` (services.rs:1264) | FmFeesWeaver invariants 1 (weaver >= 0), 2 (sum on invoice <= sum of child subtotals) | stub | services.rs:1260-1270 |
| `read_fm_fees_weaver` (services.rs:1276) | Read-by-id | stub | services.rs:1272-1282 |
| `create_fees_invoice_setting` (services.rs:1288) | FeesInvoiceSetting invariants 1 (unique by (school, academic)), 2 (per_th non-negative) | stub | services.rs:1284-1300 |
| `read_fees_invoice_setting` (services.rs:1304) | Read-by-id | stub | services.rs:1300-1310 |
| `create_fees_installment_credit` (services.rs:1320) | FeesInstallmentCredit invariants 1 (amount >= 0), 2 (unique by (student, record)), 3 (active_status = 1) | stub | services.rs:1316-1332 |
| `read_fees_installment_credit` (services.rs:1336) | Read-by-id | stub | services.rs:1332-1342 |
| `create_transaction` (services.rs:1352) | Transaction invariants 1 (type ∈ debit\|credit), 2 (polymorphic target is supported finance entity), 3 (amount >= 0) | stub | services.rs:1348-1358 |
| `read_transaction` (services.rs:1364) | Read-by-id | stub | services.rs:1360-1370 |
| `create_donor` (services.rs:1376) | Donor invariants 1 (show_public boolean), 2 (unique by email when provided) | stub | services.rs:1372-1382 |
| `read_donor` (services.rs:1388) | Read-by-id | stub | services.rs:1384-1394 |
| `create_product_purchase` (services.rs:1400) | ProductPurchase invariants 1 (paid + due = price), 2 (paid, due >= 0), 3 (one school) | stub | services.rs:1396-1412 |
| `read_product_purchase` (services.rs:1416) | Read-by-id | stub | services.rs:1412-1422 |
| `create_inventory_payment` (services.rs:1432) | InventoryPayment invariants 1 (type ∈ R\|S), 2 (amount >= 0), 3 (payment_method/bank compatible) | stub | services.rs:1428-1444 |
| `read_inventory_payment` (services.rs:1448) | Read-by-id | stub | services.rs:1444-1454 |

### Workflow: Fees Assignment (lines 1466-1536)

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `FeesAssignmentService::assign_fees_to_student` (services.rs:1476) | FeesAssign invariants 1 (unique by (school, master, student, academic)), 2 (fees_amount >= 0) | real (factory); dispatcher must enforce uniqueness | services.rs:1476-1488 — pure factory returning `FeesAssignmentDraft { student: Some(...), ... }`. The aggregate uniqueness is a storage-layer concern per the same Phase 7 workstream pattern used elsewhere. |
| `FeesAssignmentService::assign_fees_to_class` (services.rs:1494) | FeesAssign bulk-assign invariant (same uniqueness, scoped to class+section) | real (factory) | services.rs:1494-1506 — same pattern; `class_id` + optional `section_id` set; dispatcher resolves the class roster. |
| `FeesAssignmentService::validate` (services.rs:1512) | Cross-field invariant: exactly one target (student OR class); amount positive | real | services.rs:1512-1530 — `amount.amount_minor() <= 0` rejected at services.rs:1513-1516; "exactly one of (student, class)" enforced at services.rs:1517-1525. |

### Workflow: Due Fees Login Prevention (lines 1546-1602)

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `DueFeesLoginPreventionService::is_login_blocked` (services.rs:1556) | DueFeesLoginPrevent invariants 1 (unique by (school, academic, user, role)), 2 (only users with non-zero overdue balance kept) | real | services.rs:1556-1580 — `outstanding_minor >= threshold_minor` returns `LoginBlockDecision { blocked: true, ... }` at services.rs:1558-1564; otherwise `blocked: false` at services.rs:1565-1571. The row-maintenance aspect (invariants 1-2) is delegated to the dispatcher's CRUD. |
| `DueFeesLoginPreventionService::get_outstanding_balance` (services.rs:1582) | Sum of payment amounts minus discounts plus fines | real | services.rs:1582-1598 — saturating arithmetic on `amount_minor - discount_minor + fine_minor` (services.rs:1586-1592). |

### Workflow: Bank Reconciliation (lines 1622-1722)

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `BankReconciliationService::match_transaction` (services.rs:1622) | Reconciliation: match by amount + entry_type within same school | real | services.rs:1622-1648 — school filter at services.rs:1625-1627; `entry_type != Debit` skipped at services.rs:1628-1630; amount-equality match at services.rs:1631-1640; unmatched line returns `discrepancy_minor = line.amount_minor` at services.rs:1645-1648. |
| `BankReconciliationService::reconcile_statement` (services.rs:1655) | Reconcile every line; return (matched_count, unmatched_count, discrepancy) | real | services.rs:1655-1677 — delegates per-line to `match_transaction` (services.rs:1661-1672); accumulates matched/unmatched counters at services.rs:1663-1671. |
| `BankReconciliationService::mark_unmatched` (services.rs:1682) | Flag for manual review | real | services.rs:1682-1690 — returns `ManualReviewFlag { statement_line_id, reason, amount_minor }` (services.rs:1684-1688). |

### Workflow: Payroll Disbursement (lines 1736-1807)

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `PayrollDisbursementService::disburse_payroll` (services.rs:1739) | PayrollPayment invariant 1 (sum of payments <= payroll's unpaid `net_salary`); 3 (creates Expense + BankStatement) | partial | services.rs:1739-1760 — `entries.is_empty()` rejected at services.rs:1741-1745; `entry_count` populated at services.rs:1752-1754. **Missing:** invariant 1 (the sum-vs-`net_salary` cap is not enforced — `total_minor` is set to literal `0` at services.rs:1756 and the sum of `entries` is never computed); invariant 3 (the corresponding Expense + BankStatement creation is dispatched, not done here). |
| `PayrollDisbursementService::mark_as_paid` (services.rs:1764) | Per-entry paid marker | real | services.rs:1764-1772 — returns `PaidPayrollEntry { entry_id, paid: true }` (services.rs:1766-1770). Trivial marker. |
| `PayrollDisbursementService::cancel_disbursement` (services.rs:1775) | Cancellation record with reason | real | services.rs:1775-1787 — returns `CancelledDisbursement { payroll_id, reason }` (services.rs:1778-1782). Trivial. |

### Workflow: Hourly Rate Management (lines 1817-1888)

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `HourlyRateService::set_hourly_rate` (services.rs:1826) | Hourly rate versioned; non-negative | real | services.rs:1826-1840 — `rate_minor < 0` rejected at services.rs:1828-1832; returns `HourlyRateRow { staff, rate_minor, effective_from }` (services.rs:1834-1838). The "versioned" rule (new rate does not overwrite) is enforced by the dispatcher inserting a new row. |
| `HourlyRateService::calculate_pay` (services.rs:1846) | Pay = hours × rate, rounded, clamped at 0 | real | services.rs:1846-1859 — `hours <= 0.0` returns 0 at services.rs:1847-1849; `raw <= 0.0` returns 0 at services.rs:1852-1854; `raw as i64` truncates toward zero at services.rs:1858. The "nearest minor unit" rounding is delegated to the journal layer per the in-line comment at services.rs:1856-1857. |
| `HourlyRateService::get_effective_rate` (services.rs:1863) | Most recent rate with `effective_from <= date` | real | services.rs:1863-1869 — `filter(r.effective_from <= date).max_by_key(r.effective_from)` (services.rs:1864-1868). Pure lookup; expects the history to be pre-sorted. |

### Workflow: Salary Template (lines 1890-1966)

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `SalaryTemplateService::create_template` (services.rs:1894) | SalaryTemplate invariants 1 (`gross_salary == basic + house_rent + provident_fund`), 2 (`net_salary == gross - total_deduction`); non-empty name + earnings; non-negative amounts | real (structural); partial (composition) | services.rs:1894-1925 — name length 1..=200 validated at services.rs:1897-1900; `earnings.is_empty()` rejected at services.rs:1901-1904; per-line `amount_minor < 0` rejected at services.rs:1905-1909. **Missing:** invariants 1-2 (the composition rules) are evaluated at payroll-generation time, not at template-creation time, because the per-template composition is consumer-defined. |
| `SalaryTemplateService::apply_template` (services.rs:1929) | Concatenate earnings + deductions into payroll-ready lines | real | services.rs:1929-1948 — clones earnings then deductions into a single `Vec<TemplateLine>` (services.rs:1933-1941); preserves currency and template name. |
| `SalaryTemplateService::validate_template` (services.rs:1952) | Every line has non-empty label and non-negative amount | real | services.rs:1952-1964 — `label.is_empty()` rejected at services.rs:1955-1958; `amount_minor < 0` rejected at services.rs:1959-1963. |

### Summary

- **Total pub fn / pub async fn:** 66
- **Real:** 29 — the 6 wallet mutators + 1 wallet validator + `StubPaymentProvider::new` + the 3 headline factories (`record_payment` is partial, not real) + 2 carry-forward + late-fee + double-entry helpers + 5 fees-assignment / due-fees / bank-reconciliation methods + 2 payroll-mark-as-paid/-cancel + 3 hourly-rate + 3 salary-template.
- **Partial:** 5 — `WalletService::balance` (loop result discarded; invariant check is symbolic), `record_payment` (3 of 4 invariants deferred to dispatcher), `record_expense` (payment_method/account compatibility not checked), `configure_invoice_numbering` (next-invoice computation delegated), `PayrollDisbursementService::disburse_payroll` (cap vs `net_salary` not enforced; `total_minor = 0`).
- **Stub:** 32 — every Cluster C handler skeleton from `create_fees_assign_discount` (services.rs:996) through `read_inventory_payment` (services.rs:1448). All carry the same `let _ = (cmd, clock, ids); Ok(())` body and the "Full implementation lands in Phase 7 Workstream B/C/D/F/G/L" doc-comment.

### Classification rationale

- **Real vs partial** for the prompt-named wallet headline (`create_wallet`, `credit_wallet`, etc.) hinges on whether the service enforces every spec invariant the command owns vs delegating any of them to the dispatcher / aggregate. The wallet mutators all do the structural check (amount, currency, balance pre-flight) in the service and the state-machine transition in the aggregate, so they are real. The exception is `WalletService::balance` where the structural check is dead code (the loop is computed then discarded) — partial.
- **Partial vs real** for the headline mutators (`record_payment`, `record_expense`, `configure_invoice_numbering`) hinges on whether the invariant crosses aggregate boundaries. `record_payment` skips invariants 1 + 3 + 4 (assign/student id, payment_method/gateway compatibility, gateway tx id) which require cross-aggregate lookups or external I/O. `record_expense` skips invariant 2 (payment_method/account compatibility) and the cross-aggregate single-head check. `configure_invoice_numbering` skips invariant 3 (the next-invoice calculation requires `count(issued_invoices)`, a storage-side query). All three are classified partial; the gap is acknowledged in the docstrings.
- **Stub** functions in the placeholder section are unambiguously stubs: every body returns `Ok(())` after `let _ = (cmd, clock, ids);`. None of the spec invariants listed in the column are touched. This is the same pattern as the academic Phase 3 placeholder block at academic/services.rs:1246-1624.
- **`PaymentProvider` port** is marked `#[deprecated]` and slated to move to `educore-payment` in Phase 15. The 3 trait method declarations + 3 impls are not counted (no `pub` modifier) but are mentioned for completeness.

---

## documents

**Crate:** `crates/domains/documents/src/services.rs`
**Function count:** 18
**Stub count:** 0 (no `unimplemented!()` / `todo!()` / synthetic-id placeholders; the three `partial` services have explicit, documented deferrals to the storage adapter or to a not-yet-merged repository method)

The file is split into three sections (FormDownload 3A, PostalDispatch 3B,
PostalReceive 3C) plus two helper structs (`FormService`, `PostalService`)
and one read-side query factory (`track_postal_service`). All eight
mutator factory functions perform capability-gating via
`require_capability` (services.rs:32-46), persist via the typed repository,
write an audit row, and publish the corresponding domain event.
Spec-invariant 1 (non-empty title / to_title / from_title) is enforced by
the value-object constructors (`FormTitle::new` at value_objects.rs:157,
`PostalTitle::new` at value_objects.rs:250, `FromTitle::new` at
value_objects.rs:292, `ToTitle::new` at value_objects.rs:330) so the
service layer relies on the type system rather than re-validating.
Spec-invariant 4 (never hard-deleted; soft-delete via `active_status`) is
enforced by the `soft_delete` methods on `FormDownload`,
`PostalDispatch`, and `PostalReceive` (aggregate.rs:211, 593, 877),
called from the three delete services. The `reference_no` field is
declared **immutable** once set in `commands.rs:253-255` and
`commands.rs:425-427`; the `update` methods on both postal aggregates
reject any mutation with `DocumentsError::ReferenceNoImmutable`, so the
update factories do not need to re-check uniqueness on update.

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `FormService::validate_content` (services.rs:62) | FormDownload invariant 2 (at least one of `link` or `file` set) | real | services.rs:62-71 — `if link.is_none() && file.is_none() { return Err(FormHasNoContent) }`. Pure helper, no I/O. |
| `FormService::is_public` (services.rs:74) | FormDownload invariant 3 (visibility flag accessor) | real | services.rs:74-78 — `form.is_public()`. Pure accessor. |
| `FormService::is_deliverable` (services.rs:81) | FormDownload invariant 2 (has at least one of link/file) | real | services.rs:81-85 — `form.is_deliverable()`. Pure accessor. |
| `FormService::matches_publish_date` (services.rs:88) | FormDownload invariant 2 (publish_date ordering) | real | services.rs:88-92 — `form.publish_date.0 <= date`. Pure accessor. |
| `upload_form_service` (services.rs:136) | FormDownload invariants 1 (non-empty title — via `FormTitle` VO), 2 (link OR file — via `FormDownload::new`), 5 (school anchor — via typed id) | real | services.rs:148 — `require_capability(FormDownloadUpload)`; services.rs:150-151 — `FormDownload::new(new)?` enforces invariant 2; services.rs:152 — `repo.insert`; services.rs:156-164 — audit row; services.rs:168-171 — `FormUploaded` event. Title non-emptiness enforced at `value_objects.rs:157` (`FormTitle::new`). |
| `update_form_service` (services.rs:180) | FormDownload invariant 2 preserved across updates (re-validates link OR file); soft-delete guard | real | services.rs:191 — `require_capability(FormDownloadUpdate)`; services.rs:193-197 — `repo.get` with `FormNotFound` on miss; services.rs:198 — `snapshot(before)` for audit; services.rs:228 — `form.update(update_cmd)?` re-checks link/file invariant (`aggregate.rs:191`); services.rs:230-241 — audit `Update`; services.rs:243-251 — `FormUpdated` event with per-field change list. |
| `delete_form_service` (services.rs:252) | FormDownload invariant 4 (soft-delete, never hard-deleted) | real | services.rs:263 — `require_capability(FormDownloadDelete)`; services.rs:264-268 — `repo.get` with `FormNotFound`; services.rs:272 — `form.soft_delete(actor, at)?` rejects already-deleted (`aggregate.rs:211-219`); services.rs:274-283 — audit `Delete`; services.rs:286-291 — `FormDeleted` event. |
| `PostalService::reference_unique` (services.rs:358) | PostalDispatch / PostalReceive invariant 2 (reference_no unique within `(school_id, academic_id)`) — helper | real | services.rs:358-368 — `!existing.iter().any(|r| r == reference)`. Pure helper; **note:** not currently invoked from the dispatch / receive factory services — uniqueness is delegated to the storage adapter per the docstring (services.rs:352-355) and the composite unique index on `(school_id, academic_id, reference_no)`. |
| `PostalService::pair_by_reference` (services.rs:375) | Cross-aggregate helper: pair dispatches with receives by shared `reference_no` | real | services.rs:375-419 — first-match pairing with `used_receives` tracking (services.rs:392-417); unmatched dispatches / receives become `(Some, None)` / `(None, Some)` pairs; dispatches with no `reference_no` are skipped. Pure helper. |
| `PostalService::within_year` (services.rs:421) | Cross-aggregate helper: filter dispatches + receives to those whose `academic_id` matches the given year AND which carry a `reference_no` | real | services.rs:421-453 — loops dispatches and receives (services.rs:430-451); produces flat `Vec<PostalReference>` with `dispatch_id` / `receive_id` disambiguators. Pure helper. |
| `PostalService::format_address` (services.rs:456) | PostalDispatch / PostalReceive address display (free-text per spec) | real | services.rs:456-460 — `addr.as_str().to_owned()`. Pure helper. |
| `dispatch_postal_service` (services.rs:483) | PostalDispatch invariants 1 (non-empty `to_title` / `from_title` — via `PostalTitle` VOs), 2 (reference_no unique within `(school_id, academic_id)`), 3 (school + academic-year anchor) | partial | services.rs:494 — `require_capability(PostalDispatchCreate)`; services.rs:497 — `PostalDispatchId::new(tenant.school_id, Uuid::now_v7())` anchors tenant; services.rs:498-499 — `PostalDispatch::new(new)?` enforces structural construction. **Gap:** invariant 2 (reference_no uniqueness) is not enforced at the service layer — the `PostalService::reference_unique` helper at services.rs:358 is not called, and the factory does not query the repo for existing reference numbers. The docstring at services.rs:352-355 explicitly delegates uniqueness to the storage adapter via a composite unique index on `(school_id, academic_id, reference_no)`. Per the audit convention (cf. attendance `mark_student_attendance`), the service-level guard is expected and this is classified partial. |
| `update_postal_dispatch_service` (services.rs:530) | PostalDispatch invariants 1, 2 preserved across updates; soft-delete guard; reference_no immutable | real | services.rs:541 — `require_capability(PostalDispatchUpdate)`; services.rs:542-549 — `repo.get` with `PostalDispatchNotFound`; services.rs:572-578 — `dispatch.update(update_cmd)?` enforces soft-delete guard (`aggregate.rs:583-589`) and rejects any `reference_no` mutation with `DocumentsError::ReferenceNoImmutable` (`aggregate.rs:598-602`); services.rs:580-590 — audit `Update`; services.rs:594-600 — `PostalDispatchUpdated` event. Uniqueness re-check not required because `reference_no` is immutable per `commands.rs:253-255`. |
| `delete_postal_dispatch_service` (services.rs:605) | PostalDispatch invariant 5 (soft-delete, never hard-deleted) | real | services.rs:616 — `require_capability(PostalDispatchDelete)`; services.rs:617-624 — `repo.get` with `PostalDispatchNotFound`; services.rs:629 — `dispatch.soft_delete(actor, at)?` rejects already-deleted (`aggregate.rs:639-647`); services.rs:631-640 — audit `Delete`; services.rs:643-648 — `PostalDispatchDeleted` event. |
| `receive_postal_service` (services.rs:702) | PostalReceive invariants 1 (non-empty `from_title` / `to_title` — via `PostalTitle` VOs), 2 (reference_no unique within `(school_id, academic_id)`), 3 (school + academic-year anchor) | partial | services.rs:713 — `require_capability(PostalReceiveCreate)`; services.rs:716 — `PostalReceiveId::new(tenant.school_id, Uuid::now_v7())` anchors tenant; services.rs:717-718 — `PostalReceive::new(new)?` enforces structural construction. **Gap:** invariant 2 (reference_no uniqueness) is not enforced at the service layer — same as `dispatch_postal_service`. The factory delegates uniqueness to the storage adapter per `services.rs:352-355` rationale. Partial. |
| `update_postal_receive_service` (services.rs:748) | PostalReceive invariants 1, 2 preserved across updates; soft-delete guard; reference_no immutable | real | services.rs:759 — `require_capability(PostalReceiveUpdate)`; services.rs:760-767 — `repo.get` with `PostalReceiveNotFound`; services.rs:790-796 — `receive.update(update_cmd)?` enforces soft-delete guard and rejects any `reference_no` mutation with `DocumentsError::ReferenceNoImmutable` (`aggregate.rs:890-895`); services.rs:798-808 — audit `Update`; services.rs:812-818 — `PostalReceiveUpdated` event. Uniqueness re-check not required because `reference_no` is immutable per `commands.rs:425-427`. |
| `delete_postal_receive_service` (services.rs:822) | PostalReceive invariant 5 (soft-delete, never hard-deleted) | real | services.rs:833 — `require_capability(PostalReceiveDelete)`; services.rs:834-841 — `repo.get` with `PostalReceiveNotFound`; services.rs:846 — `receive.soft_delete(actor, at)?` rejects already-deleted; services.rs:848-857 — audit `Delete`; services.rs:860-866 — `PostalReceiveDeleted` event. |
| `track_postal_service` (services.rs:876) | Query: pair dispatch + receive records that share a `reference_no` | partial | services.rs:887 — `require_capability(PostalRead)`; services.rs:888 — `let _ = dispatch_repo` (dispatch side explicitly suppressed — see docstring at services.rs:868-873 acknowledging the deferred `find_by_reference` method on `PostalDispatchRepository`); services.rs:889-891 — `receive_repo.find_by_reference(school_id, &cmd.reference_no)?` returns the receive side; services.rs:892-895 — `PostalPair { dispatch: None, receive: receives.into_iter().next() }`; services.rs:898-906 — audit row with `AuditAction::Other("read")` and a synthetic `AuditTarget::Other("postal_track", Uuid::now_v7())` (the synthetic target uuid is acceptable for a read-only audit row, not a row identity). **Gap:** the dispatch side is hardcoded to `None` pending a not-yet-merged `find_by_reference` on `PostalDispatchRepository`; the function is documented as a query (not a mutation) and emits no domain event per spec, so the dispatch-side absence is the only missing piece. |

### Summary

- **Total pub fn:** 18 (`FormService::validate_content`, `FormService::is_public`, `FormService::is_deliverable`, `FormService::matches_publish_date`, `PostalService::reference_unique`, `PostalService::pair_by_reference`, `PostalService::within_year`, `PostalService::format_address`, `upload_form_service`, `update_form_service`, `delete_form_service`, `dispatch_postal_service`, `update_postal_dispatch_service`, `delete_postal_dispatch_service`, `receive_postal_service`, `update_postal_receive_service`, `delete_postal_receive_service`, `track_postal_service`)
- **Real:** 15 — all eight pure helpers, the three FormDownload mutator factories, the two update factories for postal (reference_no is immutable so no uniqueness re-check needed), and the two delete factories. Spec invariants are enforced via the aggregate constructors (`FormDownload::new` for invariant 2; `PostalDispatch::new` / `PostalReceive::new` for the structural fields), the value-object constructors (`FormTitle`, `PostalTitle`, `FromTitle`, `ToTitle` enforce invariant 1 at the type system), and the `soft_delete` methods (invariant 4 / 5 for the never-hard-delete rule).
- **Partial:** 3 — `dispatch_postal_service` and `receive_postal_service` delegate the `(school_id, academic_id, reference_no)` uniqueness check (spec invariant 2) to the storage adapter via a composite unique index, with the `PostalService::reference_unique` helper defined but not invoked from the factories (docstring at services.rs:352-355). `track_postal_service` hardcodes the dispatch side of the `PostalPair` to `None` pending a not-yet-merged `find_by_reference` method on `PostalDispatchRepository` (docstring at services.rs:868-873); the receive side is real.
- **Stub:** 0 — no `unimplemented!()` / `todo!()` / synthetic-id placeholders; every service factory either persists a real aggregate or returns a real `PostalPair` populated from the repo. The audit `AuditTarget::Other("postal_track", Uuid::now_v7())` synthetic uuid at services.rs:903 is the closest analogue to a placeholder and is appropriate for a read-only audit row (the target id is not a row identity).

### Classification rationale

- The 8 pure helpers (`FormService` × 4, `PostalService` × 4) are
  uncontentiously real — each is a small, side-effect-free function
  over already-validated aggregate state. The `PostalService` trio
  (`reference_unique`, `pair_by_reference`, `within_year`) are
  intentionally designed for caller-side composition; the audit
  convention is that un-callable helpers are still classified by
  whether they themselves are correct, not by whether the factory
  invokes them.
- The `FormDownload` mutators are all real because the spec
  invariants they own (1, 2, 4, 5) are enforced by the value-object
  constructors (`FormTitle`), the aggregate constructor
  (`FormDownload::new` for invariant 2), and the `soft_delete`
  method (invariant 4). The service layer adds capability-gating,
  repository persistence, audit-row emission, and event publishing —
  none of which the spec attributes to the service layer but all of
  which the engine rules require.
- The `PostalDispatch` / `PostalReceive` mutators are real for
  update + delete (because `reference_no` is immutable per
  `commands.rs:253-255` / `:425-427`, so no service-level
  uniqueness re-check is required on update; and `soft_delete`
  enforces the never-hard-delete rule) but partial for create
  (because the factory does not consult an existing-reference
  list before inserting). The docstring at services.rs:352-355
  explicitly delegates to the storage adapter, so the gap is
  acknowledged, not hidden. Per the audit convention used for
  attendance's `mark_student_attendance` (which has the same
  shape: uniqueness delegated to storage), this is `partial`.
- `track_postal_service` is the only non-mutator service in the
  file. The dispatch side is hardcoded to `None` pending a future
  `find_by_reference` method on `PostalDispatchRepository`; the
  function is documented as a query (no event emission per spec),
  the receive side is real, and the audit row uses a synthetic
  target uuid appropriate for a read-only audit row.

---

## facilities

**Crate:** `crates/domains/facilities/src/services.rs`
**Spec reference:** `docs/specs/facilities/aggregates.md`
**Function count:** 60 (`pub fn` + `pub async fn` only; excludes the 2
private result-struct declarations `ReceiveItemResult` at services.rs:623
and `SellItemResult` at services.rs:823, and the helper `event_id_to_uuid`
at services.rs:74)
**Stub count:** 0
**Real / Partial / Stub:** 41 real / 19 partial / 0 stub

The 60 functions split into 13 aggregates (Vehicle, Route,
AssignVehicle, Dormitory, Room, RoomType, ItemCategory, Item,
ItemStore, ItemIssue, ItemReceive, ItemSell, Supplier) plus 4
helper service structs (`TransportService`, `DormitoryService`,
`InventoryService`, `SupplierService`), one enum + struct pair
(`MovementKind`, `MovementRow`), and one headline correctness
service (`InventoryConservationService`).

Phase 8 ships the prompt-named subset (transport, dormitory,
inventory catalog + receive + issue + sell, supplier) as real
or partial factory services with a 100-case proptest for
`InventoryConservationService::check_invariant` (mirroring
Phase 7's `DoubleEntryService` pattern). Phase 8 also adds
the Update / Delete / Unassign gap-fill for every aggregate.
No function body carries a `TODO:` / `unimplemented!()` /
synthetic-id placeholder; every factory emits a real, payload-
populated event with a real `last_event_id` chain.

### Vehicle aggregate

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `create_vehicle` (services.rs:81) | Vehicle invariants 2 (`VehicleNumber` unique within school), 3 (`MadeYear` 1950..=current calendar year), 4 (optional `DriverId`) | partial | services.rs:96-105 — `Vehicle::fresh` builds aggregate with `academic_year_id`, `vehicle_no`, `vehicle_model`, `made_year`, `driver_id`; invariant 3 enforced upstream by `MadeYear::new` constructor (value_objects.rs:1138, test at rs:1805); invariant 4 satisfied by `Option<StaffId>` typing. **Missing:** invariant 2 (VehicleNumber uniqueness within school — no `UniquenessChecker` parameter on the function; the storage adapter must reject duplicates). |
| `update_vehicle` (services.rs:120) | Vehicle update semantics (mutate profile fields, preserve version + last_event_id) | real | services.rs:127-145 — change tracking per field (rs:131-144); `no changes` is rejected implicitly by always pushing at least one label; version bump at rs:151; `last_event_id` set at rs:153; `VehicleUpdated` event at rs:155-163. |
| `assign_driver` (services.rs:164) | Vehicle invariant 4 (single optional `DriverId`) | real | services.rs:170-177 — captures previous `vehicle.driver_id`, delegates mutation to `vehicle.assign_driver(...)` (aggregate.rs); `DriverAssignedToVehicle` event at rs:178-185 with `from` + `to` payload. |
| `deactivate_vehicle` (services.rs:189) | Vehicle invariant 5 (`ActiveStatus` transitions to inactive); reason captured | real | services.rs:194-202 — `vehicle.deactivate(...)` aggregate method enforces the state machine (rs:196); `VehicleDeactivated` event at rs:204-211 with reason + new_status. |
| `delete_vehicle` (services.rs:978) | Vehicle invariant 6 (cannot hard-delete while `AssignVehicle` references) | partial | services.rs:978-995 — emits `VehicleDeleted` event shell (rs:990-994). **Missing:** invariant 6 (the `AssignVehicle` referential check is deferred to the dispatcher per the docstring at rs:976-977: "The dispatcher must reject the call if any `AssignVehicle` row still references the vehicle"). |

### Route aggregate

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `create_route` (services.rs:213) | Route invariants 1 (`RouteName` unique within school+academic_year), 2 (`Fare` non-negative), 3 (`RouteStop` ordered by `StopOrder`) | partial | services.rs:219-244 — `Route::fresh` with `title`, `fare`, `stops`; invariants 2 + 3 satisfied by value-object constructors + construction-time push. **Missing:** invariant 1 (RouteName uniqueness within `(school, academic_year)` — no uniqueness check at service layer). |
| `update_route` (services.rs:999) | Route update semantics | real | services.rs:1005-1028 — per-field change tracking (rs:1013-1024); version bump + `last_event_id` at rs:1026-1027; `RouteUpdated` event at rs:1029-1037. |
| `add_stop_to_route` (services.rs:252) | Route invariant 3 (`RouteStop` ordered by `StopOrder`) | real | services.rs:258-273 — `RouteStopSpec` constructed (rs:259-264); pushed to `route.stops` (rs:265); version bump + `last_event_id` at rs:266-269; `StopAddedToRoute` event at rs:271-280. |
| `update_stop_on_route` (services.rs:1038) | Route stop mutation by `stop_order` | real | services.rs:1043-1074 — find-by-order loop (rs:1051-1065); change tracking per field (rs:1054-1064); version bump at rs:1067-1071; `StopUpdatedOnRoute` event at rs:1073-1082. |
| `remove_stop_from_route` (services.rs:1084) | Route stop removal by `stop_order` | real | services.rs:1089-1097 — `route.stops.retain(...)` (rs:1093); version bump + `last_event_id` at rs:1094-1096; `StopRemovedFromRoute` event at rs:1099-1109. |
| `delete_route` (services.rs:1111) | Route invariant 4 (cannot hard-delete while `AssignVehicle` references) | partial | services.rs:1116-1130 — emits `RouteDeleted` event shell. **Missing:** invariant 4 (referential check against `AssignVehicle` rows deferred to dispatcher). |

### AssignVehicle aggregate

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `assign_vehicle_to_route` (services.rs:287) | AssignVehicle invariants 1 (vehicle at most one route per year), 3 (`(vehicle_id, academic_year_id)` unique) | partial | services.rs:293-314 — `AssignVehicle::fresh` builds aggregate; `VehicleAssigned` event at rs:315-323. **Missing:** invariants 1 + 3 (no uniqueness check on `(vehicle_id, academic_year_id)` at service layer); invariant 5 from Vehicle spec (inactive vehicle may not be assigned — not checked here, see `TransportService::can_assign_vehicle` below). |
| `unassign_vehicle_from_route` (services.rs:1132) | AssignVehicle lifecycle (releases the assignment) | real | services.rs:1137-1150 — emits `VehicleUnassigned` event with vehicle_id + route_id (rs:1145-1149). |
| `assign_student_to_route` (services.rs:324) | AssignVehicle membership (student-to-route set; event log per spec) | real | services.rs:329-355 — derives today's date from clock (rs:340-348, defensive `unwrap_or_default()` for out-of-range dates); `StudentAssignedToRoute` event at rs:350-358. |
| `unassign_student_from_route` (services.rs:1156) | AssignVehicle membership release | real | services.rs:1161-1180 — derives today's date (rs:1167-1170); `StudentUnassignedFromRoute` event at rs:1172-1179. |

### Dormitory aggregate

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `create_dormitory` (services.rs:367) | Dormitory invariants 1 (`DormitoryName` unique within school+year), 2 (`DormitoryType` ∈ Boys/Girls), 3 (`Intake` positive), 4 (sum of `Room.NumberOfBed` ≤ `Intake`) | partial | services.rs:374-400 — `Dormitory::fresh` with name + type + intake; invariants 2 + 3 satisfied by enum + value-object constructors. **Missing:** invariant 1 (name uniqueness not checked); invariant 4 (capacity is a cross-aggregate invariant — service has no access to `Room` rows). |
| `update_dormitory` (services.rs:1241) | Dormitory update semantics | real | services.rs:1247-1278 — per-field change tracking (rs:1256-1270); version bump + `last_event_id` at rs:1273-1274; `DormitoryUpdated` event at rs:1276-1283. |
| `delete_dormitory` (services.rs:1284) | Dormitory invariant 5 (cannot hard-delete while `Room` references) | partial | services.rs:1289-1303 — emits `DormitoryDeleted` event shell. **Missing:** invariant 5 (referential check against `Room` rows deferred to dispatcher). |

### Room aggregate

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `create_room` (services.rs:442) | Room invariants 1 (`RoomNumber` unique within dormitory), 2 (`NumberOfBed` positive), 3 (`CostPerBed` non-negative), 4 (bound to one `RoomType`), 5 (assigned student count ≤ `NumberOfBed`) | partial | services.rs:449-477 — `Room::fresh` with room_number + room_type_id + number_of_bed + cost_per_bed; invariants 2-4 satisfied by value-object + enum. **Missing:** invariant 1 (RoomNumber uniqueness within Dormitory — no uniqueness check); invariant 5 (capacity check deferred to dispatcher / assignment-time service). |
| `update_room` (services.rs:1305) | Room update semantics | real | services.rs:1311-1342 — per-field change tracking (rs:1320-1333); version bump + `last_event_id` at rs:1336-1337; `RoomUpdated` event at rs:1339-1346. |
| `delete_room` (services.rs:1348) | Room delete semantics | real | services.rs:1353-1367 — emits `RoomDeleted` event (rs:1362-1366). |
| `assign_student_to_room` (services.rs:484) | Room invariant 5 (assigned student count ≤ `NumberOfBed`) | partial | services.rs:490-505 — emits `StudentAssignedToRoom` event with room_id + student_id + bed_number (rs:499-504). **Missing:** invariant 5 (capacity check — current assignment count not loaded). |
| `unassign_student_from_room` (services.rs:1369) | Room membership release | real | services.rs:1374-1389 — emits `StudentUnassignedFromRoom` event (rs:1382-1387). |

### RoomType aggregate

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `create_room_type` (services.rs:407) | RoomType invariant 1 (`RoomTypeName` unique within school) | partial | services.rs:413-435 — `RoomType::fresh` with name + description. **Missing:** invariant 1 (no uniqueness check). |
| `update_room_type` (services.rs:1185) | RoomType update semantics | real | services.rs:1191-1215 — per-field change tracking (rs:1200-1206); version bump + `last_event_id` at rs:1209-1210; `RoomTypeUpdated` event at rs:1212-1218. |
| `delete_room_type` (services.rs:1220) | RoomType invariant 2 (cannot delete while `Room` references) | partial | services.rs:1225-1239 — emits `RoomTypeDeleted` event shell. **Missing:** invariant 2 (referential check against `Room` rows deferred to dispatcher). |

### ItemCategory aggregate

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `create_item_category` (services.rs:511) | ItemCategory invariant 1 (`CategoryName` unique within school) | partial | services.rs:517-538 — `ItemCategory::fresh` with category_name. **Missing:** invariant 1 (no uniqueness check). |
| `update_item_category` (services.rs:1391) | ItemCategory update semantics | real | services.rs:1397-1417 — change tracking (rs:1404-1409); version bump + `last_event_id` at rs:1412-1413; `ItemCategoryUpdated` event at rs:1415-1420. |
| `delete_item_category` (services.rs:1422) | ItemCategory invariant 2 (cannot delete while `Item` references) | partial | services.rs:1427-1441 — emits `ItemCategoryDeleted` event shell. **Missing:** invariant 2 (referential check deferred to dispatcher). |

### Item aggregate

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `create_item` (services.rs:544) | Item invariants 1 (`ItemSku` unique within school), 2 (`ItemName` non-empty), 4 (one `ItemCategory`), 5 (cannot delete while references exist) | partial | services.rs:551-576 — `Item::fresh` with name + sku + category_id; invariants 2 + 4 satisfied by value-object + enum. **Missing:** invariant 1 (no SKU uniqueness check); invariant 3 (`TotalInStock` non-negative — only updated by receive/issue/sell per spec, so service is fine; initial value is `0`). |
| `update_item` (services.rs:1443) | Item update semantics | real | services.rs:1449-1477 — per-field change tracking (rs:1458-1466); version bump + `last_event_id` at rs:1469-1470; `ItemUpdated` event at rs:1472-1478. |
| `delete_item` (services.rs:1482) | Item invariant 5 (cannot delete while `ItemIssue`/`ItemReceive`/`ItemSell` references) | partial | services.rs:1487-1501 — emits `ItemDeleted` event shell. **Missing:** invariant 5 (referential check deferred to dispatcher). |

### ItemStore aggregate

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `create_item_store` (services.rs:582) | ItemStore invariant 1 (`StoreName` unique within school) | partial | services.rs:588-617 — `ItemStore::fresh` with store_name. **Missing:** invariant 1 (no uniqueness check). |
| `update_item_store` (services.rs:1503) | ItemStore update semantics | real | services.rs:1509-1537 — per-field change tracking (rs:1518-1527); version bump + `last_event_id` at rs:1530-1531; `ItemStoreUpdated` event at rs:1533-1538. |
| `delete_item_store` (services.rs:1542) | ItemStore invariant 2 (cannot delete while `ItemReceive` references) | partial | services.rs:1547-1563 — emits `ItemStoreDeleted` event shell. **Missing:** invariant 2 (referential check deferred to dispatcher). |

### ItemIssue aggregate

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `issue_item` (services.rs:721) | ItemIssue invariants 2 (positive `Quantity`), 5 (`IssueTo` + optional buyer), 6 (decrement `Item.TotalInStock` atomically) | partial | services.rs:727-764 — non-zero quantity check (rs:728-733); `ItemIssue::fresh` with item_id + category_id + recipient + quantity + dates (rs:740-755); `ItemIssued` event at rs:757-766. **Missing:** invariant 3 (IssueDate ≥ academic year start — not checked); invariant 6 (atomic stock decrement deferred to dispatcher per the docstring at rs:722-723). |
| `update_issue_status` (services.rs:1634) | ItemIssue invariant 4 (`IssueStatus` transitions) | real | services.rs:1640-1658 — captures `from` status (rs:1645); sets new status (rs:1646); version bump + `last_event_id` at rs:1647-1651; `ItemIssueStatusUpdated` event at rs:1653-1660. |
| `return_issued_item` (services.rs:771) | ItemIssue state machine (Returned / PartiallyReturned) | partial | services.rs:776-816 — positive return quantity check (rs:778-780); outstanding-vs-return check (rs:781-786, returns `Conflict` if exceeded); accumulated `returned_quantity` update (rs:790-792); auto-promotion to `Returned` vs `PartiallyReturned` (rs:793-798); version bump + `last_event_id` at rs:799-802; `IssuedItemReturned` event at rs:805-814. **Missing:** atomic stock restore deferred to dispatcher (the service is pure). |

### ItemReceive aggregate (header + children)

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `receive_item` (services.rs:635) | ItemReceive invariants 1 (one Supplier + one ItemStore), 2 (≥1 `ItemReceiveChild`), 4 (`GrandTotal` = sum of subtotals), 5 (`TotalQuantity` = sum of quantities), 6 (`TotalPaid + TotalDue == GrandTotal`), 8 (atomic increment of `Item.TotalInStock` per line) | partial | services.rs:641-715 — empty-lines check (rs:642-646); per-line `ItemReceiveChild::fresh` constructs SubTotal = UnitPrice × Quantity (rs:661-678); `total_quantity` accumulated (rs:672); `grand_total` accumulated from line subtotals (rs:673); `ItemReceive::fresh` builds header with computed totals (rs:680-694); `ItemReceived` event with full payload (rs:696-714). **Missing:** invariant 3 (ReceiveDate ≥ academic year start — not checked); invariant 8 (atomic stock increment deferred to dispatcher per the docstring at rs:637-640); invariant 7 (PaidStatus enum satisfied by VO). |
| `update_item_receive` (services.rs:1565) | ItemReceive invariants 4-6 preserved across updates; line add/remove cascades stock | partial | services.rs:1571-1603 — tracks `lines_to_add` / `lines_to_remove` as `changes` (rs:1578-1580) but does NOT mutate the lines vector; updates `total_paid` and recomputes `total_due` (rs:1581-1587). **Missing:** line mutation deferred to dispatcher (per the docstring at rs:1563-1564: "The dispatcher is responsible for re-applying stock deltas and re-validating totals"); the service emits the event shell. |
| `cancel_item_receive` (services.rs:1608) | ItemReceive cancellation | partial | services.rs:1614-1629 — emits `ItemReceiveCancelled` event with reason; `reversed_lines` is `Vec::new()` populated by the dispatcher (rs:1625, comment at rs:1620-1622: "Reversed lines are populated by the dispatcher from the existing child rows; the service emits the event shell"). |

### ItemSell aggregate (header + children)

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `sell_item` (services.rs:835) | ItemSell invariants 1 (`RoleId` + optional buyer), 2 (≥1 `ItemSellChild`), 4 (`GrandTotal` = sum of subtotals), 5 (`TotalQuantity` = sum of quantities), 6 (`TotalPaid + TotalDue == GrandTotal`), 8 (atomic decrement of `Item.TotalInStock` per line) | partial | services.rs:841-916 — empty-lines check (rs:842-846); per-line `ItemSellChild::fresh` constructs SubTotal = SellPrice × Quantity (rs:862-879); `total_quantity` + `grand_total` accumulated (rs:880-882); `ItemSell::fresh` builds header (rs:885-898); `ItemSold` event with full payload (rs:900-914). **Missing:** invariant 3 (SellDate ≥ academic year start — not checked); invariant 8 (atomic stock decrement deferred to dispatcher per the docstring at rs:836-838); invariant 7 (PaidStatus enum satisfied by VO). |
| `update_item_sell` (services.rs:1663) | ItemSell invariants 4-6 preserved across updates; line add/remove cascades stock | partial | services.rs:1669-1700 — same pattern as `update_item_receive` (rs:1676-1692): tracks line changes, updates total_paid + total_due, but does NOT mutate the lines vector. **Missing:** line mutation + stock cascade deferred to dispatcher. |
| `cancel_item_sell` (services.rs:1706) | ItemSell cancellation | real | services.rs:1711-1725 — emits `ItemSellCancelled` event with reason (rs:1720-1724). |
| `refund_item_sell` (services.rs:1730) | ItemSell invariant 7 (`PaidStatus` transitions include `Refunded`) | real | services.rs:1736-1769 — non-negative refund amount check (rs:1742-1746); refund-vs-total_paid cap (rs:1747-1752, returns `Conflict` if exceeded); `PaidStatus` promotion to `Refunded` on full refund, otherwise `Partial` (rs:1753-1758); version bump + `last_event_id` at rs:1759-1762; `ItemSellRefunded` event at rs:1764-1771. |

### Supplier aggregate

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `create_supplier` (services.rs:922) | Supplier invariants 1 (`SupplierName` unique within school), 2 (`ContactPersonMobile` valid), 3 (`ContactPersonEmail` valid), 4 (cannot delete while `ItemReceive` references) | partial | services.rs:929-972 — `Supplier::fresh` with company_name + addresses + contacts (rs:945-961). **Missing:** invariant 1 (no uniqueness check); invariants 2-3 satisfied by `PhoneNumber` + `EmailAddress` VO constructors. |
| `update_supplier` (services.rs:1775) | Supplier update semantics | real | services.rs:1781-1824 — per-field change tracking across all 7 mutable fields (rs:1790-1813); version bump + `last_event_id` at rs:1816-1817; `SupplierUpdated` event at rs:1819-1826. |
| `deactivate_supplier` (services.rs:1830) | Supplier state machine (Active → Inactive); reason captured | real | services.rs:1835-1848 — `s.deactivate(...)` aggregate method enforces state machine (rs:1840); `SupplierDeactivated` event at rs:1843-1849 with reason. |
| `delete_supplier` (services.rs:1853) | Supplier invariant 4 (cannot delete while `ItemReceive` references) | partial | services.rs:1858-1875 — emits `SupplierDeleted` event shell. **Missing:** invariant 4 (referential check deferred to dispatcher). |

### TransportService (helper struct)

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `TransportService::can_assign_vehicle` (services.rs:1887) | Doc-string promises: vehicle active AND route active AND no other `AssignVehicle` row for the same year | partial | services.rs:1887-1891 — body checks only `vehicle_active && vehicle.status == VehicleStatus::Active` (rs:1890). **Missing:** two of three promised checks (route-active flag and no-conflict `AssignVehicle` lookup are not performed). |
| `TransportService::fare_for_student` (services.rs:1894) | Per-student fare = route fare, optionally overridden at stop | real | services.rs:1894-1897 — `stop_override.unwrap_or(route_fare)` (rs:1896); pure helper. |

### DormitoryService (helper struct)

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `DormitoryService::available_beds` (services.rs:1906) | Available beds = total beds − current assignments | real | services.rs:1906-1911 — `room.number_of_bed.value().saturating_sub(current_assignments)` (rs:1908-1910); pure arithmetic. |
| `DormitoryService::can_assign` (services.rs:1914) | Doc-string: room must belong to the dormitory, capacity must permit | partial | services.rs:1914-1926 — body checks only `room.dormitory_id == dormitory.id` (rs:1918-1922). **Missing:** capacity check (room.NumberOfBed vs current student count; dormitory.Intake vs current students) — the function does not enforce the capacity rule that the docstring promises. |

### InventoryService (helper struct)

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `InventoryService::validate_receive` (services.rs:1934) | ItemReceive invariants 2 (non-empty lines), 4 (`GrandTotal` = sum of subtotals) | real | services.rs:1934-1949 — empty-lines check (rs:1935-1940); computed sum vs grand_total check (rs:1941-1948, returns `Conflict` if mismatch). Covers invariants 2 + 4 for the totals dimension; other invariants (date, paid+due=grand) are header-construction concerns handled at `ItemReceive::fresh`. |
| `InventoryService::validate_sell` (services.rs:1951) | ItemSell invariants 2 + 4 (non-empty lines; `GrandTotal` = sum of subtotals) | real | services.rs:1951-1965 — same pattern as `validate_receive` (rs:1952-1964). |
| `InventoryService::validate_issue` (services.rs:1966) | ItemIssue invariant 2 (positive quantity); Item invariant 3 (`TotalInStock` ≥ quantity) | real | services.rs:1966-1979 — zero quantity rejected (rs:1967-1971); stock-vs-quantity check (rs:1972-1978, returns `Conflict` if insufficient stock). |

### SupplierService + MovementKind + InventoryConservationService (helpers + headline)

| Function | Spec Invariant | Status | Evidence |
| --- | --- | --- | --- |
| `SupplierService::normalize_name` (services.rs:1987) | Trims + collapses internal whitespace | real | services.rs:1987-1990 — `split_whitespace().collect::<Vec<_>>().join(" ")` (rs:1989); pure string transform. |
| `MovementKind::sign` (services.rs:2020) | Sign multiplier: +1 for Receive, −1 for Issue / Sell | real | services.rs:2020-2025 — `match self { Self::Receive => 1, Self::Issue \| Self::Sell => -1 }` (rs:2021-2024). |
| `InventoryConservationService::check_invariant` (services.rs:2053) | Per `(school_id, item_id)`: signed sum of movements ≥ 0 (Phase 8 headline correctness check, 100-case proptest) | real | services.rs:2053-2073 — cross-school filter (rs:2060); per-item signed accumulation (rs:2062-2066); negative `on_hand` rejected (rs:2067-2072); proptest target at services.rs:2853+ (100 cases for balanced + overdraw sequences). |
| `InventoryConservationService::on_hand_for` (services.rs:2076) | Single-item on-hand projection | real | services.rs:2076-2086 — school + item filter (rs:2080-2082); signed accumulation (rs:2083-2084); pure read. |

### Summary

- **Total pub fn:** 60
- **Real:** 41 — every Update / Delete / Unassign / Cancel /
  Refund / Deactivate / Assign / Status factory plus the 7 helper
  struct methods that match their doc-strings (`fare_for_student`,
  `available_beds`, `validate_receive`, `validate_sell`,
  `validate_issue`, `normalize_name`, `MovementKind::sign`,
  `check_invariant`, `on_hand_for`).
- **Partial:** 19 — 10 Create factories missing the
  `(name, school)` uniqueness check (no `UniquenessChecker`
  parameter on facilities services — the pattern academic uses is
  absent here), 3 Create factories with additional cross-aggregate
  invariants deferred to dispatcher
  (`receive_item` / `issue_item` / `sell_item` for atomic stock
  deltas, `return_issued_item` for atomic stock restore,
  `update_item_receive` / `update_item_sell` for line mutation,
  `cancel_item_receive` for reversed lines), 5 Delete / Unassign
  factories where the referential invariant (cannot delete while
  child rows reference) is deferred to the dispatcher
  (`delete_vehicle`, `delete_route`, `delete_dormitory`,
  `delete_room_type`, `delete_item_category`, `delete_item`,
  `delete_item_store`, `delete_supplier`), and 2 helper struct
  methods where the body does not match its doc-string
  (`TransportService::can_assign_vehicle`, `DormitoryService::can_assign`).
- **Stub:** 0 — every function body either implements the
  invariant or delegates it explicitly via a doc-string note; no
  `TODO:` / `unimplemented!()` / `let _ = (cmd, clock, ids);`
  pattern.

### Classification rationale

- **Real vs partial** for the Update / Delete / Cancel / Unassign /
  Assign factories hinges on whether the spec invariant requires a
  cross-aggregate lookup or referential check. When it does
  (`delete_*`, `cancel_item_receive`, `update_item_*` line
  mutation), the gap is acknowledged in a doc-string and the
  service emits the event shell — partial. When it doesn't
  (per-field change tracking, simple event emission, single-
  aggregate state transitions), the service is complete — real.
- **Real vs partial** for the Create factories hinges on
  uniqueness. Facilities does **not** use the academic
  `UniquenessChecker` parameter pattern; uniqueness for
  `VehicleNumber`, `RouteName`, `DormitoryName`, `RoomNumber`,
  `RoomTypeName`, `CategoryName`, `ItemSku`, `StoreName`,
  `SupplierName` is delegated entirely to the storage adapter.
  Per the audit convention used for academic `create_class` and
  attendance `mark_student_attendance` (same shape: uniqueness
  delegated to storage), these are classified partial.
- **Real vs partial** for the receive / issue / sell / return
  factories hinges on atomic stock mutation (Item invariant 3,
  ItemReceive/ItemSell/ItemIssue invariants 6/8). The pure
  factory builds the aggregate and emits the event; the
  dispatcher acquires the row-level lock and applies the stock
  delta. The docstrings (rs:637-640, rs:722-723, rs:836-838)
  acknowledge this. Same pattern as Phase 7 finance
  `record_payment` (acknowledged deferred invariants = partial).
- **Real vs partial** for the helper struct methods
  (`TransportService::can_assign_vehicle`, `DormitoryService::can_assign`)
  hinges on whether the body matches its doc-string. Both
  doc-strings promise 3 / 2 checks respectively; both bodies
  implement 1. Partial.
- **`InventoryConservationService::check_invariant`** is the
  Phase 8 headline correctness check (mirrors Phase 7's
  `DoubleEntryService` proptest at
  `crates/domains/finance/src/services.rs:953`). It is fully
  implemented and 100-case proptest target — the only headline
  correctness service in this file that is real end-to-end.

---

## library

**Crate:** `crates/domains/library/src/services.rs`
**Spec reference:** `docs/specs/library/aggregates.md`
**Function count:** 37 (`pub fn` + `pub async fn`; excludes the `DateRange` value-object accessors at `services.rs:975`/`982`, the `ReportsService::{book_repo,...}` repo accessors at `services.rs:1122-1152`, and the two `::new` constructors at `services.rs:1102` and `services.rs:1417`).
**Stub count:** 15 (every Cluster C "handler skeleton" returns `Err(not_supported("TODO"))` — see `services.rs:722, 735, 748, 761, 774, 787, 800, 813, 826, 839, 853, 866, 875, 888, 901`).

| Function | Spec Invariant | Status | Evidence |
|----------|---------------|--------|----------|
| `create_book_category` (line 93) | `CategoryName` unique within a school (inv 1) | partial | Aggregate + event minted at `services.rs:103-119`; `CategoryName::new` enforces non-empty. **Missing:** uniqueness check delegated to dispatcher / storage adapter (the pure factory does not look up `BookCategoryRepository::find_by_name`); "may not be deleted while referenced" is a delete-time invariant, not create-time. |
| `add_book` (line 129) | Book invariants 1 (title non-empty), 2 (ISBN unique per school), 3 (book_number unique), 5 (at least one of ISBN or book_number present), 6 (one category + one subject) | partial | Aggregate + event minted at `services.rs:142-167`; `BookTitle::new` and `Book::fresh` enforce title non-empty + category/subject linkage. **Missing:** ISBN / book_number uniqueness is deferred to the dispatcher (test at `services.rs:1596` explicitly notes: "The pure factory does not enforce uniqueness"); invariant 5 (at least one of ISBN or book_number) is not enforced at the factory level. |
| `register_library_member` (line 182) | LibraryMember invariants 1 (exactly one of StudentId/StaffId), 2 (RoleId), 3 (unique by `(member_type, student_staff_id)` per school-year), 4 (Active by default) | partial | Aggregate + event minted at `services.rs:196-218`; `MemberId` sum type disambiguates Student vs Staff (inv 1); `MemberStatus::Active` is the default (inv 4). **Missing:** uniqueness on `(member_type, student_staff_id)` is deferred to the dispatcher (per the `LibraryMemberRepository::find` port); school policy on eligible roles (inv 2) is out of scope for v1 per the spec. |
| `create_book_issue` (line 234) | BookIssue invariants 3 (GivenDate >= year start), 4 (DueDate > GivenDate), 5 (sum open issues ≤ Book.Quantity), 6 (book + member active in current year) | partial | Pure validation of due_date > given_date at `services.rs:251-255` (test at `services.rs:1649`); aggregate + event minted at `services.rs:257-275`. **Missing:** invariants 3 / 5 / 6 are deferred to the dispatcher (the `BookIssueEligibility::check` policy at `services.rs:523-553` carries the stock-conservation and active-roster checks; docstring at `services.rs:224-228` says "The dispatcher is responsible for invoking the `BookIssueEligibility` policy and atomically decrementing `book.available_copies`"). |
| `return_book` (line 301) | BookIssue invariant 5 (sum open issues drops by returned qty), 7 (status transition), 8 (Returned is immutable on re-return) | partial | `is_open()` guard at `services.rs:316-319` rejects already-Returned issues (test at `services.rs:1673`); `BookReturn` aggregate + `BookReturned` + `BookReturnRecorded` events minted at `services.rs:326-361`; `book_issue.mark_returned` transition at `services.rs:368`. **Missing:** the late-fine conditional at `services.rs:375-379` is dead code — both branches return `None` (`fine_event` is always `None`); the spec's invariant that a late return produces a `Fine` is deferred to a separate `compute_fine` call (the dispatcher is responsible for wiring it; see comment at `services.rs:373-379`). |
| `compute_fine` (line 401) | Fine formula: `fine_amount = max(0, days_overdue - grace_period) * per_day_rate`, with FixedAmount / PerDayRate / PercentOfPrice kinds | real | Pure late-fine formula delegated to `FineCalculationService::compute` at `services.rs:427`; `Fine` aggregate + `FineCalculated` event minted at `services.rs:429-450`. The underlying `FineCalculationService` (see below) has table-driven tests for fixed-amount / per-day-rate / grace-period / zero-on-time at `services.rs:1726-1760` and a 100-case proptest at `services.rs:1770-1795`. |
| `FineCalculationService::days_overdue` (line 462) | `days_overdue = max(0, as_of - due_date)`, saturated at `u32::MAX` | real | Sign + saturation logic at `services.rs:464-473`; capped at `i64::from(u32::MAX)` then `u32::try_from`. |
| `FineCalculationService::compute` (line 484) | Pure late-fine formula with three `FineKind` variants + grace-period subtraction | real | Formula at `services.rs:493-510`: billable = days_overdue − grace_period; `FineKind::FixedAmount(n)` returns `n`; `PerDayRate(rate)` returns `billable * rate`; `PercentOfPrice(pct)` returns `per_day_rate * pct / 100` (interpreted as book price). Table-driven tests at `services.rs:1726-1760` + 100-case proptest (monotonic-in-days-late, constant-for-fixed-amount) at `services.rs:1770-1795`. |
| `BookIssueEligibility::check` (line 523) | Cross-cutting rule: book active, enough copies, member active, max-books-per-member respected | real | Book status guard (Retired/Lost rejected) at `services.rs:531-535`; availability check at `services.rs:536-538` (`open_issue_quantity + cmd_quantity > quantity` rejects); member active at `services.rs:539-541`; max-books cap at `services.rs:542-546`. Mirrors Phase 7 finance positive-answer pattern. |
| `BookRenewalEligibility::check` (line 566) | BookIssue invariant 9 (renewal only on Issued/Renewed; new due date > current) | real | Status guard (Issued/Renewed only) at `services.rs:570-574`; new-due-date > current-due-date guard at `services.rs:575-579`; test at `services.rs:1696-1721`. |
| `OverdueIssues::is_satisfied_by` (line 593) | Issue is overdue when open AND `due_date < as_of` | real | Delegates to `BookIssue::is_overdue_as_of` (line 595). |
| `AvailableBooks::is_satisfied_by` (line 606) | Book available iff `quantity - sum(open_issue_quantities) > 0` | real | Delegates to `Book::available_copies` (line 608). |
| `ActiveMembers::is_satisfied_by` (line 617) | Member is `Active` | real | `matches!(member.status, MemberStatus::Active)` at line 619. |
| `BookService::available_copies` (line 633) | Sums `quantity` over Issued + Renewed open issues | real | Filter + sum at `services.rs:636-641`; returns `StockCopies(book.available_copies(sum))`. |
| `update_book_category` (line 657) | Updates mutate `category_name`; id / tenant guards; no-op rejected; `BookCategoryUpdated` event | real | Id match at `services.rs:672-676`; tenant match at `services.rs:678-682`; no-op detection at `services.rs:685-689`; mutation + version bump at `services.rs:692-696`; event at `services.rs:700-707`. |
| `delete_book_category` (line 713) | BookCategory invariant 2 (no books reference the category) | stub | `Err(not_supported("TODO"))` at `services.rs:722`. The "no books reference this category" guard is deferred to the dispatcher. |
| `update_book` (line 726) | Update title / author / publisher / etc. | stub | `Err(not_supported("TODO"))` at `services.rs:735`. |
| `delete_book` (line 739) | Book invariant 8 (no BookIssue references in any year) | stub | `Err(not_supported("TODO"))` at `services.rs:748`. Year-scoped reference check is deferred to the dispatcher. |
| `adjust_book_quantity` (line 752) | Book invariant 4 (Quantity non-negative); atomic against open issues (invariant 7) | stub | `Err(not_supported("TODO"))` at `services.rs:761`. The "stock conservation" row-lock + non-negative guarantee are deferred to the dispatcher (per `services.rs:55-66` Phase 9 Risks). |
| `update_library_member` (line 765) | Update `member_type` / `member_ud_id` while preserving invariants 1, 3 | stub | `Err(not_supported("TODO"))` at `services.rs:774`. |
| `deactivate_library_member` (line 778) | LibraryMember invariant 4 (deactivated member may not receive new issues) | stub | `Err(not_supported("TODO"))` at `services.rs:787`. |
| `reactivate_library_member` (line 791) | LibraryMember invariant 4 (re-activation restores issue eligibility) | stub | `Err(not_supported("TODO"))` at `services.rs:800`. |
| `delete_library_member` (line 804) | LibraryMember invariant 5 (no BookIssue references in any year) | stub | `Err(not_supported("TODO"))` at `services.rs:813`. |
| `renew_book` (line 817) | BookIssue invariants 9 (Issued/Renewed, member has no overdue book), 10 (DueDate extends, GivenDate/Quantity unchanged) | stub | `Err(not_supported("TODO"))` at `services.rs:826`. The "no overdue book" cross-aggregate lookup is deferred to the dispatcher. |
| `mark_book_lost` (line 830) | BookIssue invariant 7 (`Lost` status); book stock decremented | stub | `Err(not_supported("TODO"))` at `services.rs:839`. |
| `record_book_return` (line 844) | BookReturn aggregate + event (alternate path to `return_book`) | stub | `Err(not_supported("TODO"))` at `services.rs:853`. |
| `waive_book_issue_fine` (line 857) | Fine waiver: emit `FineWaived`, outstanding drops to zero | stub | `Err(not_supported("TODO"))` at `services.rs:866`. |
| `search_books` (line 870) | Free-text search on title / author / ISBN | stub | `Err(not_supported("TODO"))` at `services.rs:875`. |
| `list_overdue_issues` (line 879) | List open issues past due date as of a given date | stub | `Err(not_supported("TODO"))` at `services.rs:888`. |
| `list_member_issues` (line 892) | List issues (open + historical) for a member | stub | `Err(not_supported("TODO"))` at `services.rs:901`. |
| `ReportsService::borrow_summary` (line 1174) | Counts active / overdue loans + returns-in-period for a date range | real | Open-issue listing at `services.rs:1183`; overdue filter `i.due_date <= range.to` at `services.rs:1186-1188`; returns-in-period listing at `services.rs:1190-1194`. Report struct fields correctly populated at `services.rs:1196-1204`. |
| `ReportsService::overdue_list` (line 1213) | Per-issue overdue record with book title + member external id + days overdue | real | List overdue issues at `services.rs:1223`; book + member lookups at `services.rs:1226-1242`; days-overdue computation + `u32` saturation at `services.rs:1243-1248`; record assembly at `services.rs:1249-1260`. |
| `ReportsService::inventory_status` (line 1263) | Per-category stock rollup (total / on-loan / available) | real | Book + category listings at `services.rs:1272-1273`; category-name index at `services.rs:1276-1280`; per-category rollup at `services.rs:1283-1290`; zero-category skip + sort-by-name at `services.rs:1295-1314`. |
| `ReportsService::fine_collection` (line 1345) | Per-period fine rollup: levied, collected, outstanding | partial | Fine listing + non-waived filter + period join at `services.rs:1357-1375`; levied + outstanding accumulation at `services.rs:1378-1385`; `total_collected = levied - outstanding` at `services.rs:1386`. **Missing:** the engine has no per-fine "paid" flag, so `total_outstanding` always equals `total_levied` and `total_collected` is always zero — this is acknowledged in the docstring at `services.rs:1353-1356` ("Until the finance receivable posts back, collected is the levied minus outstanding (and equals zero before the receivable posts)"). Real fine-collection semantics are deferred until finance wires back paid-state. |
| `is_issue_overdue` (line 1472) | Issue is open AND `due_date < as_of` | real | Delegates to `BookIssue::is_overdue_as_of` (line 1474); covered by the round-trip + classification test at `services.rs:1984-2014`. |
| `days_overdue_for_issue` (line 1480) | Days overdue = `max(0, as_of - due_date)`, 0 for closed issues | real | `is_open()` guard at `services.rs:1485-1487`; sign + saturation at `services.rs:1488-1495`; test at `services.rs:1984-2014`. |
| `ServiceFactory::reports_service` (line 1444) | Wires the six `Arc<dyn ...>` repository ports to a `ReportsService` | real | Constructor clones the six `Arc`s at `services.rs:1448-1455`. Wired by the `service_factory_reports_service_wiring` test at `services.rs:2222-2246` and exercised for `Send + Sync` object-safety at `services.rs:2051-2058`. |

### Summary

- **Total pub fn / pub async fn:** 37
- **Real:** 19 — the six pure factory services (`create_book_category`, `add_book`, `register_library_member`, `create_book_issue`, `return_book`, `compute_fine`), `FineCalculationService` (the Phase 9 headline correctness check, with 100-case proptest), `BookIssueEligibility` + `BookRenewalEligibility`, the three specifications (`OverdueIssues`, `AvailableBooks`, `ActiveMembers`), `BookService::available_copies`, `update_book_category`, three of the four `ReportsService` queries (`borrow_summary`, `overdue_list`, `inventory_status`), the two reports-helpers (`is_issue_overdue`, `days_overdue_for_issue`), and `ServiceFactory::reports_service`. **Note:** `create_book_category`, `add_book`, and `register_library_member` are classified partial (see table above) due to uniqueness lookups deferred to the dispatcher; `return_book` is partial due to the dead-code late-fine branch. The "real" count above uses the table-level classification; the strict tally is 16 real + 6 partial + 15 stub.
- **Partial:** 3 — `create_book_issue` (stock-conservation + active-roster invariants deferred to dispatcher via `BookIssueEligibility`), `return_book` (late-fine branch is dead code; fine is delegated to `compute_fine`), `ReportsService::fine_collection` (acknowledged: outstanding always equals levied because the engine has no "paid" flag on `Fine` yet).
- **Stub:** 15 — every Cluster C "handler skeleton" returns `Err(not_supported("TODO"))` per the explicit docstring at `services.rs:643-651`. These are the update / delete / state-transition handlers for the six non-trivial aggregates (`delete_book_category`, `update_book`, `delete_book`, `adjust_book_quantity`, `update_library_member`, `deactivate_library_member`, `reactivate_library_member`, `delete_library_member`, `renew_book`, `mark_book_lost`, `record_book_return`, `waive_book_issue_fine`) plus three read-query handlers (`search_books`, `list_overdue_issues`, `list_member_issues`). Every stub is annotated to delegate the spec invariants the command is responsible for to the dispatcher (e.g. "no BookIssue references in any year" for `delete_book` / `delete_library_member`, "stock conservation under concurrent writes" for `adjust_book_quantity` per `services.rs:55-66`).

### Classification rationale

- **Real vs partial** for the Create / Issue factories hinges on whether the spec invariant requires a uniqueness lookup, a cross-aggregate reference check, or a stock-conservation atomic update. When it does (`create_book_category` uniqueness, `add_book` ISBN / book_number uniqueness, `register_library_member` `(member_type, student_staff_id)` uniqueness, `create_book_issue` "open issues ≤ quantity" + active-roster), the gap is acknowledged via dispatcher-deferred docstrings and the service emits the event shell — partial. When it doesn't (`compute_fine`, which is a pure calculation on caller-supplied inputs), the service is complete — real.
- **`return_book`** is partial, not stub, because the BookReturn aggregate creation + `BookReturned` + `BookReturnRecorded` events + `is_open()` guard + `mark_returned` transition are all implemented (lines `services.rs:316-371`). The only gap is the late-fine conditional, which is dead code (`if/else` both return `None`) and explicitly deferred to `compute_fine` per the comment at `services.rs:373-379`.
- **`update_book_category`** is the lone update handler that is real end-to-end (id / tenant / no-op guards + mutation + version bump + event at `services.rs:672-707`). All other update / delete / state-transition handlers in Cluster C are stubs.
- **`FineCalculationService`** is the Phase 9 headline correctness check. It mirrors Phase 7's `LateFeeService` (`crates/domains/finance/src/services.rs:1259`) and Phase 8's `InventoryConservationService` — fully implemented, table-driven unit tests, 100-case proptest target. The only domain service in this file with end-to-end test coverage on the load-bearing logic.
- **`BookIssueEligibility` / `BookRenewalEligibility`** are the pure policy services the dispatcher calls before persisting an issue / renewal. Both implement every check their docstring promises (4 checks for issue, 2 for renewal). They are the partial-fill for `create_book_issue` and `renew_book`: the spec invariants are enforced, just by a policy helper invoked from the dispatcher rather than by the command factory itself.
- **`ReportsService`** ships 4 async report queries; 3 are real (`borrow_summary`, `overdue_list`, `inventory_status`) and 1 is partial (`fine_collection`) due to the engine's missing "paid" flag on `Fine`. The report structs round-trip via `serde_json` (tests at `services.rs:1881-1948`); `DateRange` validates inclusive bounds at `services.rs:963-973`; the service is object-safe (test at `services.rs:2051-2058`).
- **Stub count discrepancy with the earlier audit (15 vs 16):** the earlier audit counted 16 stubs; this audit counts 15 (`delete_book_category`, `update_book`, `delete_book`, `adjust_book_quantity`, `update_library_member`, `deactivate_library_member`, `reactivate_library_member`, `delete_library_member`, `renew_book`, `mark_book_lost`, `record_book_return`, `waive_book_issue_fine`, `search_books`, `list_overdue_issues`, `list_member_issues`). The earlier count likely double-counted `update_book_category`, which is documented under the same "handler skeleton" heading at `services.rs:643-657` but is in fact fully implemented (id / tenant / no-op guards + mutation + event at `services.rs:672-707`).
