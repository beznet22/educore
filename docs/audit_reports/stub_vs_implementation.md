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
