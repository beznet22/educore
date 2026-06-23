# Audit findings: educore-attendance (Phase 5)

**Scope:** `crates/domains/attendance/`, `docs/specs/attendance/`,
`docs/commands/attendance.md`, `docs/events/attendance.md`,
`docs/handoff/PHASE-5-HANDOFF.md`, `AGENTS.md` (the attendance row).

**Total findings:** 53

---

### FINDING 1

- **id:** DOMAIN-ATT-001
- **area:** domain-crates
- **severity:** Critical
- **location:** `docs/specs/attendance/permissions.md:16-19` vs `crates/cross-cutting/rbac/src/value_objects.rs:153-201`
- **description:** The spec mandates two-segment capability strings
  (`Attendance.Mark`, `Attendance.Update`, `Attendance.BulkMark`,
  `Attendance.Read`, `Attendance.Notify`, `Attendance.Report`,
  `Attendance.Subject.Mark`, `Attendance.Staff.Mark`,
  `Attendance.Import`, `Attendance.Import.Validate`,
  `Attendance.Import.Commit`, `Attendance.Import.Cancel`,
  `Attendance.Report.Daily`, `Attendance.Report.Weekly`, etc.) but
  the engine ships the three-segment form
  (`AttendanceStudentCreate`, `AttendanceStudentUpdate`,
  `AttendanceBulkMark`, `AttendanceNotify`,
  `AttendanceImportCreate`, `AttendanceImportValidate`, …). The
  capability string is a wire contract — consumers, audit logs,
  role catalogs, and event subscribers reference these strings —
  so this is a contract drift between the spec and the code.
- **expected:** `docs/specs/attendance/permissions.md:16` lists
  `Attendance.Mark`, `Attendance.Update`, `Attendance.BulkMark`,
  `Attendance.Read`, `Attendance.Notify`, `Attendance.Report`.
- **evidence:** `crates/cross-cutting/rbac/src/value_objects.rs:155-201`
  defines `AttendanceStudentCreate`, `AttendanceStudentUpdate`,
  `AttendanceStudentDelete`, `AttendanceSubjectNotify`,
  `AttendanceStaffCreate`, `AttendanceBulkMark`,
  `AttendanceImportCreate`, `AttendanceImportValidate`,
  `AttendanceReportRead`, `AttendanceNotify` (three-segment form)
  — no two-segment `Attendance.Mark` etc. The handoff at
  `docs/handoff/PHASE-5-HANDOFF.md:439-444` (OQ #3) acknowledges
  this divergence but does not resolve it.

---

### FINDING 2

- **id:** DOMAIN-ATT-002
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/attendance/src/events.rs:131-138` vs `docs/specs/attendance/events.md:58-67`
- **description:** `StudentAttendanceUpdated` carries
  `changes: Vec<String>` (a list of field names that changed)
  instead of the spec's `from_type: AttendanceType` /
  `to_type: AttendanceType` pair. Without the typed from/to pair,
  downstream consumers (notification fan-out, finance fine
  accrual, academic absence counters) cannot tell whether the
  student was transitioned *into* or *out of* absence — they
  only know the field names that changed.
- **expected:** `docs/specs/attendance/events.md:58-67`
  ```
  pub struct StudentAttendanceUpdated {
      pub student_attendance_id: StudentAttendanceId,
      pub student_id: StudentId,
      pub attendance_date: AttendanceDate,
      pub from_type: AttendanceType,
      pub to_type: AttendanceType,
      pub notes: Option<String>,
      pub updated_by: UserId,
      pub updated_at: Timestamp,
  }
  ```
- **evidence:** `crates/domains/attendance/src/events.rs:131-138` —
  ```rust
  pub struct StudentAttendanceUpdated {
      pub student_attendance_id: StudentAttendanceId,
      pub changes: Vec<String>,
      pub event_id: EventId,
      pub correlation_id: CorrelationId,
      pub occurred_at: Timestamp,
  }
  ```
  Missing fields: `student_id`, `attendance_date`, `from_type`,
  `to_type`, `notes`, `updated_by`, `updated_at`.

---

### FINDING 3

- **id:** DOMAIN-ATT-003
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/attendance/src/events.rs:186-192` vs `docs/specs/attendance/events.md:69-77`
- **description:** `StudentAttendanceRestored` is missing all six
  payload fields the spec mandates (`student_id`,
  `attendance_date`, `from_type`, `to_type`, `restored_by`,
  `restored_at`). Without the typed from/to fields, the
  notification fan-out cannot decide whether to send an "all
  clear" notice (the spec calls this out as an edge case at
  `workflows.md:67-71`).
- **expected:** `docs/specs/attendance/events.md:69-77` lists six
  payload fields beyond `student_attendance_id`.
- **evidence:** `crates/domains/attendance/src/events.rs:187-192` —
  ```rust
  pub struct StudentAttendanceRestored {
      pub student_attendance_id: StudentAttendanceId,
      pub event_id: EventId,
      pub correlation_id: CorrelationId,
      pub occurred_at: Timestamp,
  }
  ```
  No payload fields beyond the metadata footer.

---

### FINDING 4

- **id:** DOMAIN-ATT-004
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/attendance/src/events.rs:237-249` vs `docs/specs/attendance/events.md:79-87`
- **description:** `StudentAbsentForDay` is missing the spec's
  `notify: bool` field that mirrors the
  `MarkStudentAttendance.notify` flag. The whole point of the
  event is to drive the guardian notification fan-out (per
  `events.md:91-97`); without `notify`, the communication
  subscriber cannot tell whether to send an SMS/email/push or
  skip the dispatch.
- **expected:** `docs/specs/attendance/events.md:79-87` — the
  spec lists `notify: bool` with the comment
  `"// mirrors the MarkStudentAttendance.notify flag"`.
- **evidence:** `crates/domains/attendance/src/events.rs:238-249`
  declares no `notify` field. Compare with the
  `SubjectAbsentNotificationRequested` event at lines 509-519,
  which does carry `notify` via the
  `MarkSubjectAttendanceCommand.notify` flag.

---

### FINDING 5

- **id:** DOMAIN-ATT-005
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/attendance/src/events.rs:1127-1134` vs `docs/specs/attendance/events.md:224-229`
- **description:** `BulkImportCancelled` is missing
  `cancelled_by: UserId` and `cancelled_at: Timestamp`. The audit
  log cannot attribute the cancellation to a specific user or
  point in time.
- **expected:** `docs/specs/attendance/events.md:224-229`
  ```
  pub struct BulkImportCancelled {
      pub bulk_import_id: BulkAttendanceImportId,
      pub cancelled_by: UserId,
      pub reason: String,
      pub cancelled_at: Timestamp,
  }
  ```
- **evidence:** `crates/domains/attendance/src/events.rs:1128-1134`
  carries only `bulk_import_id` and `reason` — `cancelled_by`
  and `cancelled_at` are absent.

---

### FINDING 6

- **id:** DOMAIN-ATT-006
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/attendance/src/events.rs:897-907` vs `docs/specs/attendance/events.md:194-200`
- **description:** `BulkImportStarted` uses `marked_by: UserId`
  (the field name is borrowed from the `Mark*` family) instead
  of the spec's `started_by: UserId`, and uses `occurred_at:
  Timestamp` instead of the spec's `started_at: Timestamp`. The
  field-name mismatch is a wire-contract drift.
- **expected:** `docs/specs/attendance/events.md:194-200` — field
  name `started_by` (not `marked_by`).
- **evidence:** `crates/domains/attendance/src/events.rs:898-907`:
  `pub marked_by: educore_core::ids::UserId,` vs spec
  `pub started_by: UserId,`. Code also lacks `started_at` and
  only carries `occurred_at`.

---

### FINDING 7

- **id:** DOMAIN-ATT-007
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/attendance/src/events.rs:961-968` vs `docs/specs/attendance/events.md:202-207`
- **description:** `BulkImportValidated` is missing
  `validated_by: UserId` and `validated_at: Timestamp`. The
  spec also carries `row_count: u32` only, but the code adds
  `absent_count: u32` (which the spec reserves for
  `BulkImportCommitted`). The field swap is a wire-contract
  drift.
- **expected:** `docs/specs/attendance/events.md:202-207`
  ```
  pub struct BulkImportValidated {
      pub bulk_import_id: BulkAttendanceImportId,
      pub validated_by: UserId,
      pub validated_at: Timestamp,
      pub row_count: u32,
  }
  ```
- **evidence:** `crates/domains/attendance/src/events.rs:962-968`
  — fields are `bulk_import_id`, `row_count`, `absent_count`,
  `event_id`, `correlation_id`, `occurred_at`. Missing
  `validated_by` and `validated_at`; the `absent_count` field
  belongs on `BulkImportCommitted` per the spec.

---

### FINDING 8

- **id:** DOMAIN-ATT-008
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/attendance/src/events.rs:1018-1024` vs `docs/specs/attendance/events.md:209-215`
- **description:** `BulkImportCommitted` is missing
  `committed_by: UserId` and `committed_at: Timestamp` and the
  spec's `row_count` field; the code renames `row_count` to
  `committed_count: u32` and drops the spec's `absent_count:
  u32` (which `BulkImportValidated` picked up incorrectly — see
  DOMAIN-ATT-007).
- **expected:** `docs/specs/attendance/events.md:209-215`
  ```
  pub struct BulkImportCommitted {
      pub bulk_import_id: BulkAttendanceImportId,
      pub committed_by: UserId,
      pub committed_at: Timestamp,
      pub row_count: u32,
      pub absent_count: u32,
  }
  ```
- **evidence:** `crates/domains/attendance/src/events.rs:1019-1024`
  — fields are `bulk_import_id`, `committed_count`, `event_id`,
  `correlation_id`, `occurred_at`. Missing `committed_by`,
  `committed_at`, `absent_count`.

---

### FINDING 9

- **id:** DOMAIN-ATT-009
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/attendance/src/events.rs:1072-1079` and `crates/domains/attendance/src/value_objects.rs`
- **description:** `BulkImportFailed` is missing the spec's
  `row_errors: Vec<RowError>` field. The `RowError` value object
  does not exist in `value_objects.rs` despite the spec
  (`events.md:232-233`) requiring it as
  `RowError { row_index, student_id, reason }`. Bulk-import
  diagnostics cannot surface per-row failure reasons.
- **expected:** `docs/specs/attendance/events.md:217-222` lists
  `pub row_errors: Vec<RowError>,` and `events.md:232-233`
  documents `RowError { RowIndex, StudentId, Reason }`.
- **evidence:**
  - `crates/domains/attendance/src/events.rs:1073-1079`
    fields are `bulk_import_id`, `failed_count`, `reason`,
    `event_id`, `correlation_id`, `occurred_at`. Missing
    `row_errors` (and the underlying `RowError` type itself).
  - `grep -n "RowError" crates/domains/attendance/src/value_objects.rs`
    returns no struct/enum definitions.

---

### FINDING 10

- **id:** DOMAIN-ATT-010
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/attendance/src/events.rs:1245-1254` vs `docs/specs/attendance/events.md:238-246`
- **description:** `AbsenceNotificationRequested` is missing
  `requested_by: UserId` and `requested_at: Timestamp`. The
  audit trail cannot identify the actor who requested the
  notification.
- **expected:** `docs/specs/attendance/events.md:238-246` — spec
  lists six payload fields beyond the event envelope
  (student_attendance_id, student_id, attendance_date, channel,
  template, requested_by, requested_at).
- **evidence:** `crates/domains/attendance/src/events.rs:1246-1254`
  carries `student_attendance_id`, `student_id`,
  `attendance_date`, `channel`, `template`, and the metadata
  footer — no `requested_by` or `requested_at`.

---

### FINDING 11

- **id:** DOMAIN-ATT-011
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/attendance/src/events.rs:644-651` vs `docs/specs/attendance/events.md:168-176`
- **description:** `StaffAttendanceUpdated` carries
  `changes: Vec<String>` instead of the spec's
  `from_type: AttendanceType` / `to_type: AttendanceType` pair.
  Same pattern as the student-attendance updated event (see
  DOMAIN-ATT-002). HR/finance subscribers cannot detect
  absence transitions.
- **expected:** `docs/specs/attendance/events.md:168-176`
  ```
  pub struct StaffAttendanceUpdated {
      pub staff_attendance_id: StaffAttendanceId,
      pub staff_id: StaffId,
      pub attendance_date: AttendanceDate,
      pub from_type: AttendanceType,
      pub to_type: AttendanceType,
      pub updated_by: UserId,
      pub updated_at: Timestamp,
  }
  ```
- **evidence:** `crates/domains/attendance/src/events.rs:645-651`
  has `changes: Vec<String>` only. Missing `staff_id`,
  `attendance_date`, `from_type`, `to_type`, `updated_by`,
  `updated_at`.

---

### FINDING 12

- **id:** DOMAIN-ATT-012
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/attendance/src/events.rs:454-461` vs `docs/specs/attendance/events.md:127-136`
- **description:** `SubjectAttendanceUpdated` carries
  `changes: Vec<String>` instead of the spec's
  `from_type: AttendanceType` / `to_type: AttendanceType` pair.
  Same pattern as DOMAIN-ATT-002 and DOMAIN-ATT-011.
- **expected:** `docs/specs/attendance/events.md:127-136` lists
  six payload fields (subject_attendance_id, student_id,
  subject_id, attendance_date, from_type, to_type, updated_by,
  updated_at).
- **evidence:** `crates/domains/attendance/src/events.rs:455-461`
  has `changes: Vec<String>` only. Missing
  `student_id`, `subject_id`, `attendance_date`, `from_type`,
  `to_type`, `updated_by`, `updated_at`.

---

### FINDING 13

- **id:** DOMAIN-ATT-013
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/attendance/src/events.rs:843-850` vs `docs/specs/attendance/events.md:283` (implied by aggregates.md)
- **description:** `ExamAttendanceUpdated` carries
  `changes: Vec<String>` instead of the spec's `from_type` /
  `to_type` pair. Same anti-pattern as
  DOMAIN-ATT-002/011/012; consistent bug across all
  `*AttendanceUpdated` events.
- **expected:** `docs/specs/assessment/events.md` (the cross-
  referenced source-of-truth for the `ExamAttendanceUpdated`
  event) lists `from_type`/`to_type` fields. The pattern is
  consistent across `docs/specs/attendance/events.md:58-67,
  127-136, 168-176`.
- **evidence:** `crates/domains/attendance/src/events.rs:844-850`
  has `changes: Vec<String>` only. Missing `from_type`,
  `to_type`, `updated_by`, `updated_at` (no spec file is
  shipped in `docs/specs/assessment/events.md` for
  `ExamAttendanceUpdated` either — the attendance crate is the
  authoritative source per Phase 5 handoff OQ #1).

---

### FINDING 14

- **id:** DOMAIN-ATT-014
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/attendance/src/events.rs:1308-1320` vs `docs/specs/attendance/events.md:259-271`
- **description:** `ClassAttendanceRecomputed` carries a
  different shape from the spec: the spec lists
  `student_id`, `exam_type_id`, `academic_year_id`,
  `days_opened`, `days_present`, `days_absent`, `days_late`,
  `days_half_day`, `days_on_leave`, while the code carries
  `class_id`, `section_id`, `attendance_date`, `total_students`,
  `absent_count`, `present_count`. The code event is a per-class-
  per-day roll-up; the spec event is a per-student-per-exam-
  per-year roll-up with typed `Days*` wrappers. This is a
  fundamental wire-shape divergence.
- **expected:** `docs/specs/attendance/events.md:259-271`
  ```
  pub struct ClassAttendanceRecomputed {
      pub class_attendance_id: ClassAttendanceId,
      pub student_id: StudentId,
      pub exam_type_id: ExamTypeId,
      pub academic_year_id: AcademicYearId,
      pub days_opened: DaysOpened,
      pub days_present: DaysPresent,
      pub days_absent: DaysAbsent,
      pub days_late: DaysLate,
      pub days_half_day: DaysHalfDay,
      pub days_on_leave: DaysOnLeave,
      pub recomputed_at: Timestamp,
  }
  ```
- **evidence:** `crates/domains/attendance/src/events.rs:1309-1320`
  declares a different shape (class_id/section_id/total_students/
  absent_count/present_count). Missing the typed `Days*`
  wrappers — none of which are defined in
  `crates/domains/attendance/src/value_objects.rs`.

---

### FINDING 15

- **id:** DOMAIN-ATT-015
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/attendance/src/events.rs:307-316` vs `docs/specs/attendance/events.md:100-107`
- **description:** `StudentAttendanceImported` is missing the
  spec's `source: AttendanceSource` and `imported_at:
  Timestamp` fields. The code also adds `student_attendance_id`
  (which is a derived value, not in the spec) and renames the
  spec's `import_id` to `bulk_import_id`. The wire contract
  drifts in three directions: missing fields, extra fields, and
  renamed fields.
- **expected:** `docs/specs/attendance/events.md:100-107`
  ```
  pub struct StudentAttendanceImported {
      pub import_id: BulkAttendanceImportId,
      pub student_id: StudentId,
      pub attendance_date: AttendanceDate,
      pub attendance_type: AttendanceType,
      pub source: AttendanceSource,
      pub imported_at: Timestamp,
  }
  ```
- **evidence:** `crates/domains/attendance/src/events.rs:308-316`
  fields are `student_attendance_id`, `bulk_import_id`,
  `student_id`, `attendance_date`, `attendance_type`, plus the
  metadata footer. Missing `source` and `imported_at`; the
  `import_id` field is renamed to `bulk_import_id`; the
  `student_attendance_id` field is added but not in the spec.

---

### FINDING 16

- **id:** DOMAIN-ATT-016
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/attendance/src/events.rs` (entire file)
- **description:** Every event payload uses raw `NaiveDate`
  instead of the spec's typed `AttendanceDate` wrapper. The
  spec at `value-objects.md:40` mandates `AttendanceDate:
  NaiveDate` as a typed value object; the value_objects module
  does not declare one, so every event field falls back to the
  underlying `chrono::NaiveDate`. This violates the engine
  rule "Compile-time safety over strings" (AGENTS.md) and
  makes it impossible to add cross-cutting invariants on
  attendance dates (e.g. "not in the future").
- **expected:** `docs/specs/attendance/value-objects.md:40` —
  `| AttendanceDate | NaiveDate |` is listed as a typed value
  object.
- **evidence:** `grep -n "pub struct AttendanceDate" crates/domains/attendance/src/value_objects.rs`
  returns no result. Compare with
  `crates/domains/attendance/src/value_objects.rs:107-120`
  declaring `StudentAttendanceId` etc. as typed wrappers but no
  `AttendanceDate`. The events at events.rs:58, 119, 178, 244,
  311, 376, 512, 577, 702, 766, 1248, 1313 all use
  `pub attendance_date: NaiveDate,`.

---

### FINDING 17

- **id:** DOMAIN-ATT-017
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/attendance/src/commands.rs:414-419`
- **description:** `RequestAbsenceNotificationCommand.channel`
  and `.template` are raw `String` fields. The spec mandates
  typed `NotificationChannel` (SMS, Email, Push) and
  `NotificationTemplate` enums (`commands.md:248-249`); neither
  is defined in `value_objects.rs` or any other crate. The
  same wire-form divergence applies to the
  `SubjectAbsentNotificationRequested` event at events.rs:514-515
  and the `AbsenceNotificationRequested` event at
  events.rs:1249-1250.
- **expected:** `docs/specs/attendance/commands.md:248-249`
  ```
  pub channel: NotificationChannel, // SMS, Email, Push
  pub template: NotificationTemplate, // e.g. "absence-daily"
  ```
- **evidence:**
  - `crates/domains/attendance/src/commands.rs:417-418`:
    `pub channel: String,` and `pub template: String,`.
  - `grep -rn "pub enum NotificationChannel\|pub enum NotificationTemplate" crates/`
    returns no results.
  - `crates/domains/attendance/src/events.rs:514-515, 1249-1250`
    also use raw `String`.

---

### FINDING 18

- **id:** DOMAIN-ATT-018
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/attendance/src/aggregate.rs` and `docs/specs/attendance/aggregates.md:210-232`
- **description:** The spec defines a `ClassAttendance` aggregate
  (`docs/specs/attendance/aggregates.md:210-232` and
  `entities.md:35-48`) with the invariant
  `"days_opened = days_present + days_absent + days_on_leave + days_half_day * 0.5"`.
  The attendance crate ships a `ClassAttendanceId` typed
  identifier but no `ClassAttendance` aggregate struct and no
  invariant enforcement. The integration test
  `attendance_class_attendances_aggregate` row stays
  `Pending` in `coverage.toml:694-701`.
- **expected:** `docs/specs/attendance/aggregates.md:210-232`
  declares `ClassAttendance` as a per-(student, exam_type,
  academic_year) summary aggregate.
- **evidence:**
  - `grep -n "pub struct ClassAttendance" crates/domains/attendance/src/aggregate.rs`
    returns no result.
  - `grep -rn "ClassAttendanceRepository\|recompute_class_attendance" crates/domains/attendance/src/`
    returns no service or repository.
  - `crates/domains/attendance/src/value_objects.rs:155-160` —
    only the `ClassAttendanceId` typed id is defined; no
    aggregate.

---

### FINDING 19

- **id:** DOMAIN-ATT-019
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/attendance/src/aggregate.rs` and `docs/specs/attendance/aggregates.md:235-248`
- **description:** The spec defines an `AttendanceBulk`
  aggregate (`docs/specs/attendance/aggregates.md:235-248` and
  `entities.md:56-65`) as a per-(student, date) denormalized
  staging row with the invariant "Belongs to exactly one
  `BulkAttendanceImport`". The crate ships an `AttendanceBulkId`
  typed identifier (as a Phase 5 placeholder per the handoff)
  but no `AttendanceBulk` aggregate struct. The integration
  test path `attendance_integration.rs` does not exercise the
  attendance_bulks table.
- **expected:** `docs/specs/attendance/aggregates.md:235-248`
  declares the `AttendanceBulk` aggregate.
- **evidence:**
  - `grep -n "pub struct AttendanceBulk" crates/domains/attendance/src/aggregate.rs`
    returns no result.
  - `crates/domains/attendance/src/value_objects.rs:166-173`
    declares `AttendanceBulkId` with the comment
    `"Placeholder until the bulk-import header aggregate lands
    in a follow-up phase"` (Phase 5 handoff's Phase 5 stub
    contract).

---

### FINDING 20

- **id:** DOMAIN-ATT-020
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/attendance/src/services.rs` (entire file) and `docs/specs/attendance/services.md:62-83`
- **description:** `AbsenceDetectionService` is missing from
  `services.rs`. The spec (`services.md:62-83`) mandates three
  methods: `detect(school, date, section) -> Vec<StudentAbsentForDay>`,
  `should_notify(school, attendance) -> bool`,
  `dedup_within_day(events) -> Vec<StudentAbsentForDay>`. The
  third method is implemented as `AttendanceService::dedup_within_day`
  but in the wrong struct; the first two are absent.
- **expected:** `docs/specs/attendance/services.md:62-83`
  ```
  pub struct AbsenceDetectionService;
  impl AbsenceDetectionService {
      pub fn detect(...) -> Vec<StudentAbsentForDay>;
      pub fn should_notify(...) -> bool;
      pub fn dedup_within_day(...) -> Vec<StudentAbsentForDay>;
  }
  ```
- **evidence:**
  - `grep -n "AbsenceDetectionService" crates/domains/attendance/src/services.rs`
    returns no result.
  - `crates/domains/attendance/src/services.rs:1273-1284`
    implements `dedup_within_day` inside the `AttendanceService`
    struct (wrong struct per the spec).

---

### FINDING 21

- **id:** DOMAIN-ATT-021
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/attendance/src/services.rs` and `docs/specs/attendance/services.md:89-131`
- **description:** `AttendanceReportService` is missing
  entirely. The spec mandates six methods:
  `daily(school, date)`, `weekly(school, section, from, to)`,
  `monthly(school, section, month)`, `by_class(school, from, to)`,
  `by_student(school, student, from, to)`,
  `by_staff(school, from, to)`. The report capability
  `AttendanceReportRead` (rbac) is defined and the spec
  workflow `docs/specs/attendance/workflows.md:137-153` mandates
  the report pipeline.
- **expected:** `docs/specs/attendance/services.md:89-131`.
- **evidence:** `grep -n "AttendanceReportService" crates/domains/attendance/src/`
  returns no result.

---

### FINDING 22

- **id:** DOMAIN-ATT-022
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/attendance/src/services.rs` and `docs/specs/attendance/services.md:139-165`
- **description:** `AttendanceImportService` is missing as a
  service struct. The spec mandates four methods:
  `stage(source, rows)`, `validate(import, school)`,
  `commit(import, school, actor)`, `cancel(import, reason)`.
  The free-function services in
  `crates/domains/attendance/src/services.rs:855-1181`
  (`import_attendance`, `validate_bulk_import`,
  `commit_bulk_import`, `cancel_bulk_import`) implement the
  state-machine but live in the file's free-function namespace,
  not in a typed `AttendanceImportService` struct as the spec
  requires.
- **expected:** `docs/specs/attendance/services.md:139-165`.
- **evidence:** `grep -n "pub struct AttendanceImportService" crates/domains/attendance/src/`
  returns no result.

---

### FINDING 23

- **id:** DOMAIN-ATT-023
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/attendance/src/services.rs` (entire file) and `docs/specs/attendance/services.md:174-200`
- **description:** The spec mandates three policy structs:
  `AttendanceEligibility` (for `MarkStudentAttendanceCommand`),
  `BulkMarkEligibility` (for `BulkMarkStudentAttendanceCommand`),
  `NotificationEligibility` (for `StudentAbsentForDay`). None
  are declared in the crate.
- **expected:** `docs/specs/attendance/services.md:174-200`.
- **evidence:** `grep -rn "AttendanceEligibility\|BulkMarkEligibility\|NotificationEligibility" crates/domains/attendance/src/`
  returns no result.

---

### FINDING 24

- **id:** DOMAIN-ATT-024
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/attendance/src/services.rs` and `docs/specs/attendance/services.md:201-235`
- **description:** The spec mandates three specification
  structs: `ActiveOnDate`, `HasOutstandingAbsence`,
  `EligibleForExamAttendance`. None are declared in the crate.
  These are used by the `StudentAttendanceQuery`,
  `ClassAttendanceRecomputed`, and exam-day workflows
  respectively.
- **expected:** `docs/specs/attendance/services.md:201-235`.
- **evidence:** `grep -rn "ActiveOnDate\|HasOutstandingAbsence\|EligibleForExamAttendance" crates/domains/attendance/src/`
  returns no result.

---

### FINDING 25

- **id:** DOMAIN-ATT-025
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/attendance/src/repository.rs` and `docs/specs/attendance/repositories.md:149-180`
- **description:** `ClassAttendanceRepository` is missing. The
  spec mandates four methods: `get(school, student, exam_type,
  year)`, `list_for_student(school, student, year)`,
  `list_for_exam_type(school, exam_type, year)`,
  `upsert(c)`. Without this port, the storage adapter cannot
  persist the `ClassAttendance` projection.
- **expected:** `docs/specs/attendance/repositories.md:149-180`.
- **evidence:** `grep -n "ClassAttendanceRepository" crates/domains/attendance/src/repository.rs`
  returns no result; the 5 traits shipped are
  `StudentAttendanceRepository`, `SubjectAttendanceRepository`,
  `StaffAttendanceRepository`, `ExamAttendanceRepository`,
  `AttendanceImportRepository`.

---

### FINDING 26

- **id:** DOMAIN-ATT-026
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/attendance/src/services.rs:1225-1285`
- **description:** `AttendanceService` is missing four of the
  seven methods the spec mandates. The spec
  (`services.md:6-54`) requires: `mark`, `update`,
  `is_late(date, arrival, threshold) -> bool`,
  `is_half_day(school, attendance) -> bool`,
  `is_holiday(school, date) -> bool`,
  `emit_absence_event(attendance, school_id) -> Option<StudentAbsentForDay>`,
  `recompute_class_attendance(student, academic_year, events) -> ClassAttendance`.
  The code ships only `is_late` (as a stub returning `false`),
  `emit_absence_event` (different signature: takes `&StudentAttendance`
  not `(&StudentAttendance, SchoolId)`), and
  `dedup_within_day` (which is part of `AbsenceDetectionService`
  per the spec, not `AttendanceService`). The `mark`, `update`,
  `is_half_day`, `is_holiday`, and `recompute_class_attendance`
  methods are absent.
- **expected:** `docs/specs/attendance/services.md:6-54`.
- **evidence:**
  - `grep -nE "fn mark|fn update|fn is_half_day|fn is_holiday|fn recompute_class_attendance" crates/domains/attendance/src/services.rs`
    returns no result (the free-function `mark_*_attendance`
    services exist but live outside the `AttendanceService`
    struct).
  - `crates/domains/attendance/src/services.rs:1233-1243` —
    `is_late` is a stub returning `false`.

---

### FINDING 27

- **id:** DOMAIN-ATT-027
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/attendance/src/services.rs:1200-1216`
- **description:** The `request_absence_notification` service
  uses placeholder values for production-critical fields:
  `placeholder_uuid = uuid::Uuid::now_v7()` for `student_id`
  and `chrono::NaiveDate::from_ymd_opt(1970, 1, 1).expect("epoch")`
  for `attendance_date`. Both values reach the event payload and
  the bus. The `.expect("epoch")` is a production-code
  panic-prone call (the production date arithmetic is infallible
  for `1970-01-01`, but the call violates the engine rule
  against `expect()` in production code).
- **expected:** Per the engine rule in AGENTS.md ("No
  `unwrap()` or `expect()` in production paths"), no
  `expect()` is allowed.
- **evidence:** `crates/domains/attendance/src/services.rs:1200-1209`:
  ```rust
  let placeholder_uuid = uuid::Uuid::now_v7();
  ...
  crate::value_objects::StudentId::new(cmd.tenant.school_id, placeholder_uuid),
  chrono::NaiveDate::from_ymd_opt(1970, 1, 1).expect("epoch"),
  ```

---

### FINDING 28

- **id:** DOMAIN-ATT-028
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/attendance/src/services.rs:1233-1243`
- **description:** `AttendanceService::is_late` is a stub that
  unconditionally returns `false`. The spec calls this method
  out specifically: `"is_late checks the school-defined late
  threshold"`. A production caller invoking this method will
  silently treat all students as on-time. The Phase 5 handoff
  documents this as a stub (line 1238: `"// Phase 5 stub. The
  full implementation considers"`) but the stub body returns
  `false` rather than `unimplemented!()` or a `Result::Err`,
  making the failure mode invisible.
- **expected:** `docs/specs/attendance/services.md:27-31`
  mandates an actual implementation that compares the arrival
  time against the threshold.
- **evidence:** `crates/domains/attendance/src/services.rs:1233-1243`:
  ```rust
  pub const fn is_late(
      _date: chrono::NaiveDate,
      _arrival: chrono::NaiveTime,
      _threshold: chrono::NaiveTime,
  ) -> bool {
      // Phase 5 stub. The full implementation considers
      // the school's `late_threshold_minutes` setting and
      // the day-of-week calendar. The integration test
      // (Workstream D) exercises the production path.
      false
  }
  ```

---

### FINDING 29

- **id:** DOMAIN-ATT-029
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/attendance/src/services.rs` and `docs/specs/attendance/services.md:51`
- **description:** `recompute_class_attendance` references a
  `StudentAttendanceEvent` slice type that does not exist in
  the crate. The spec signature is
  `recompute_class_attendance(student, academic_year, events: &[StudentAttendanceEvent]) -> ClassAttendance`.
  No enum or trait alias named `StudentAttendanceEvent` is
  defined in `events.rs` or `value_objects.rs`.
- **expected:** `docs/specs/attendance/services.md:48-52`.
- **evidence:**
  - `grep -rn "StudentAttendanceEvent" crates/domains/attendance/src/`
    returns no struct/enum/trait definition.

---

### FINDING 30

- **id:** DOMAIN-ATT-030
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/attendance/src/services.rs` and `crates/domains/attendance/src/events.rs:179-228`
- **description:** `StudentAttendanceRestored` is a fully
  defined event struct but is never emitted by any service.
  The spec `events.md:69-77` and `workflows.md:68-71` mandate
  that when a teacher re-marks an absent student Present, the
  engine emits `StudentAttendanceRestored`. The
  `update_student_attendance` service
  (`services.rs:182-230`) only emits `StudentAttendanceUpdated`
  (and even that with a non-spec `changes: Vec<String>` shape;
  see DOMAIN-ATT-002). The "restore" transition is invisible to
  downstream subscribers.
- **expected:** `docs/specs/attendance/workflows.md:65-71` —
  spec step 5 mandates the restored event on transition out of
  absence.
- **evidence:**
  - `grep -rn "StudentAttendanceRestored::new" crates/domains/attendance/src/`
    returns 0 production call sites (1 in the unit test on
    events.rs:1471-1475).
  - `crates/domains/attendance/src/services.rs:182-230`
    emits only `StudentAttendanceUpdated`, never
    `StudentAttendanceRestored`.

---

### FINDING 31

- **id:** DOMAIN-ATT-031
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/attendance/src/value_objects.rs`
- **description:** Most value objects mandated by
  `docs/specs/attendance/value-objects.md` are missing. The
  spec lists (beyond the typed ids and the four closed enums
  already shipped): `InTime`, `OutTime`, `Period`, `TimeWindow`,
  `LateThreshold`, `DayOfWeek`, `DaysOpened`, `DaysPresent`,
  `DaysAbsent`, `DaysLate`, `DaysHalfDay`, `DaysOnLeave`,
  `Notify`, `IsAbsent`, `IsHoliday`, `MarkedBy`, `MarkedAt`,
  `MarkedFrom`, `AttendanceRange`, `AttendanceReportKind`,
  `AttendancePercentage`, `YearMonth`, and the `Validate`
  trait. None of these are declared in `value_objects.rs`. The
  spec also mandates a `NotificationChannel` and
  `NotificationTemplate` enum (already covered in
  DOMAIN-ATT-017).
- **expected:** `docs/specs/attendance/value-objects.md:6-90`
  lists 25+ value objects and the `Validate` trait.
- **evidence:** `grep -nE "pub struct (InTime|OutTime|Period|TimeWindow|LateThreshold|DayOfWeek|DaysOpened|DaysPresent|DaysAbsent|DaysLate|DaysHalfDay|DaysOnLeave|AttendanceRange|AttendanceReportKind|AttendancePercentage|YearMonth)" crates/domains/attendance/src/value_objects.rs`
  returns no result. `grep -n "trait Validate" crates/domains/attendance/src/`
  also returns no result.

---

### FINDING 32

- **id:** DOMAIN-ATT-032
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/attendance/src/entities.rs` and `docs/specs/attendance/tables.md`
- **description:** `tables.md` lists 9 tables (5 owned by the
  attendance domain plus the 2 exam-attendance tables
  delegated to assessment, plus the
  `student_attendance_bulks` and `class_attendances` tables).
  The crate's `entities.rs` ships only 2 child entity structs
  (`StudentAttendanceImport`, `StaffAttendanceImport`); no
  `#[derive(DomainQuery)]` macro is invoked anywhere in the
  crate (the macro emission is documented in AGENTS.md as the
  path that emits the typed AST for the storage adapter to
  translate).
- **expected:** `docs/specs/attendance/tables.md` lists 9
  tables; each should map to a `#[derive(DomainQuery)]` struct
  per the engine's "macro-emitted typed AST" rule (AGENTS.md
  § "Code Standards").
- **evidence:**
  - `grep -rn "DomainQuery" crates/domains/attendance/src/`
    returns no result.
  - `crates/domains/attendance/src/entities.rs` (143 lines
    total) declares only `StudentAttendanceImport` (line 40)
    and `StaffAttendanceImport` (line 96).

---

### FINDING 33

- **id:** DOMAIN-ATT-033
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/attendance/tests/` (absent) and `docs/handoff/PHASE-5-HANDOFF.md:140-145`
- **description:** The crate has no `tests/` directory; all
  integration testing lives in
  `crates/tools/storage-parity/tests/attendance_integration.rs`.
  AGENTS.md's Validation Checklist requires "At least one
  integration test added for new behavior" per PR, and the
  standard per-domain layout calls for a `tests/` subdirectory
  alongside `src/`. The Phase 5 handoff's "93 unit tests pass
  in `educore-attendance`" + 4 storage-parity integration tests
  covers the cross-cutting path but does not include per-
  aggregate behavioural tests that don't require the storage
  adapter (e.g. the `mark_staff_attendance` rejection paths
  for `is_absent` vs `is_holiday`, the `AttendanceService::dedup_within_day`
  edge cases beyond the single `tests.rs` test, the
  `commit_bulk_import` "import not in Validated state" path,
  etc.).
- **expected:** A `crates/domains/attendance/tests/` directory
  with per-aggregate integration tests per the module layout
  in AGENTS.md.
- **evidence:** `ls crates/domains/attendance/tests/` returns
  `No such file or directory`.

---

### FINDING 34

- **id:** DOMAIN-ATT-034
- **area:** domain-crates
- **severity:** High
- **location:** `crates/tools/storage-parity/tests/attendance_integration.rs` and `crates/domains/attendance/src/services.rs`
- **description:** Only 1 of the 14 service functions is
  exercised by the storage-parity integration test
  (`bulk_mark_student_attendance`). The other 13 services
  (`mark_student_attendance`, `update_student_attendance`,
  `mark_subject_attendance`, `update_subject_attendance`,
  `mark_staff_attendance`, `update_staff_attendance`,
  `mark_exam_attendance`, `update_exam_attendance`,
  `import_attendance`, `validate_bulk_import`,
  `commit_bulk_import`, `cancel_bulk_import`,
  `request_absence_notification`) have no storage-adapter
  integration coverage.
- **expected:** Per AGENTS.md "Validation Checklist" — "at
  least one integration test added for new behavior".
- **evidence:** `grep -rn "mark_student_attendance\|mark_subject_attendance\|mark_staff_attendance\|mark_exam_attendance\|update_.*_attendance\|import_attendance\|validate_bulk_import\|commit_bulk_import\|cancel_bulk_import\|request_absence_notification" crates/tools/storage-parity/tests/attendance_integration.rs`
  returns only the `bulk_mark_student_attendance` references
  (the helper `make_bulk_cmd` and `dispatch_bulk_mark`).

---

### FINDING 35

- **id:** DOMAIN-ATT-035
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/attendance/src/aggregate.rs` (all 5 aggregates)
- **description:** None of the 5 aggregates has a mutator
  method (`update`, `soft_delete`, `restore`, `transition_to_*`).
  Only the `fresh()` constructor and predicate getters
  (`is_active`, `is_absent`, `is_terminal`) exist. The
  `update_*_attendance` services mutate the aggregates through
  `&mut StudentAttendance` parameters but the mutation logic
  lives entirely in the services, not on the aggregate itself.
  This is a separation-of-concerns gap: the aggregate is a
  passive data record.
- **expected:** Per the engine rule "Domain scopes via
  extension traits" (AGENTS.md) and the academic crate's
  pattern (see `crates/domains/academic/src/aggregate.rs` for
  `Student::promote`, `Student::transfer`), aggregates should
  expose mutators.
- **evidence:** `grep -nE "fn update|fn soft_delete|fn restore|fn promote|fn transfer" crates/domains/attendance/src/aggregate.rs`
  returns no result.

---

### FINDING 36

- **id:** DOMAIN-ATT-036
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/attendance/src/entities.rs:40-62`
- **description:** `StudentAttendanceImport` does not carry a
  `student_record_id` field despite the spec entities.md
  implicitly requiring it (the
  `MarkStudentAttendanceCommand` carries `student_record_id`
  and the staging row promotes to a `StudentAttendance`
  aggregate, which carries `student_record_id`). The bulk-
  commit code (`services.rs:1082-1090`) acknowledges this as a
  Phase 5 stub by synthesising a placeholder
  `StudentRecordId` from the event id, but the entity struct
  itself is missing the field.
- **expected:** Per `docs/specs/attendance/entities.md:6-20`,
  the staging row "carries `StudentId`, `AttendanceDate`,
  `InTime`, `OutTime`, `AttendanceType`, `Notes`" — the
  spec is silent on `student_record_id`, but the promotion
  flow requires it (the live `StudentAttendance` aggregate's
  `student_record_id` cannot be synthesised from the event id
  in production).
- **evidence:** `crates/domains/attendance/src/entities.rs:40-62`
  — fields are `id`, `bulk_import_id`, `student_id`,
  `attendance_date`, `attendance_type`, `in_time`, `out_time`,
  `notes`, `is_validated`, `active_status`. No
  `student_record_id`.

---

### FINDING 37

- **id:** DOMAIN-ATT-037
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/attendance/src/services.rs:276-302`
- **description:** `bulk_mark_student_attendance` synthesises a
  placeholder `StudentId` for the "default type" aggregate
  (the unmarked-students stub):
  `StudentId::new(cmd.tenant.school_id, event_id_to_uuid(event_id))`
  — the same UUID used for the aggregate id. This
  placeholder student id propagates into the persisted
  `StudentAttendance` aggregate and the
  `StudentAttendanceMarked` event payload. Production callers
  cannot distinguish the placeholder from a real student id.
- **expected:** The bulk-mark service should consume the
  section roster (per `services.md:8-53`'s example signature
  taking `Student` and `StudentRecord` parameters), not
  synthesise a fake student.
- **evidence:** `crates/domains/attendance/src/services.rs:286-288`:
  ```rust
  // The "default" student — Phase 5 stub. Replaced with
  // a real roster pull in the dispatcher.
  StudentId::new(cmd.tenant.school_id, event_id_to_uuid(event_id)),
  ```

---

### FINDING 38

- **id:** DOMAIN-ATT-038
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/attendance/src/aggregate.rs:1062-1119`
- **description:** `BulkAttendanceImport` aggregate carries a
  `notes: Option<String>` field but the spec's
  `BulkAttendanceImport` (`aggregates.md:129-175`) does not
  mandate it. The `BulkImportStarted` event payload also
  doesn't include notes. There is no path from the import
  command's notes through to the event or aggregate on
  commit. The field appears to be a Phase 5 addition without a
  spec basis.
- **expected:** Per `docs/specs/attendance/aggregates.md:129-175`
  the `BulkAttendanceImport` has no `notes` field.
- **evidence:** `crates/domains/attendance/src/aggregate.rs:556`
  declares `pub notes: Option<String>,` on `BulkAttendanceImport`.
  Compare with `crates/domains/attendance/src/aggregate.rs:103-108`
  (other aggregates) which carry the standard 10-field audit
  footer — `notes` is not in the standard footer.

---

### FINDING 39

- **id:** DOMAIN-ATT-039
- **area:** domain-crates
- **severity:** High
- **location:** `docs/specs/attendance/events.md:7-16` vs `crates/cross-cutting/events/src/domain_event.rs:55`
- **description:** The spec at `events.md:11` declares the
  `DomainEvent` trait with `const TYPE: &'static str;`. The
  actual trait at `crates/cross-cutting/events/src/domain_event.rs:55`
  uses `const EVENT_TYPE: &'static str;`. Every impl in
  `events.rs:107-121, 159-175, 211-227, 280-296, ...` (21
  events) uses `EVENT_TYPE` / `AGGREGATE_TYPE`. The spec is
  out of date; the spec-vs-code drift lives at the spec layer.
- **expected:** Spec mandates `const TYPE: &'static str;` but
  the canonical trait uses `EVENT_TYPE`. The spec should be
  updated to match the code (or vice versa); the current
  state has both layers disagreeing.
- **evidence:**
  - `docs/specs/attendance/events.md:7-16` declares the
    `DomainEvent` trait with `const TYPE: &'static str;` and
    `fn aggregate_id(&self) -> Uuid;`.
  - `crates/cross-cutting/events/src/domain_event.rs:52-75`
    uses `const EVENT_TYPE: &'static str;` and
    `const SCHEMA_VERSION: u32;` and `const AGGREGATE_TYPE: &'static str;`.

---

### FINDING 40

- **id:** DOMAIN-ATT-040
- **area:** domain-crates
- **severity:** High
- **location:** `crates/cross-cutting/rbac/src/services.rs:345-650` and `docs/specs/attendance/permissions.md:60-77`
- **description:** `DefaultRoleCatalog::school_admin()` (and
  the `super_admin()`, `class_teacher()`, `subject_teacher()`,
  `attendance_cell()`, `staff()`, `student()`, `parent()`,
  `hr()`, `accountant()`, `auditor()` methods) does not grant
  any of the 24 new `Attendance.*` capabilities. The Phase 5
  handoff acknowledges this as OQ #5 (lines 455-463) and
  defers it to consumer `seed.rs` initialisation, but the
  default role catalog's purpose is exactly this; deferring
  leaves the engine without a working default. Per
  `docs/specs/attendance/permissions.md:60-77`, `SchoolAdmin`
  should have `Attendance.*`, `AttendanceCell` should have
  `Attendance.* + Attendance.Import.* + Attendance.Report.*`,
  etc.
- **expected:** `docs/specs/attendance/permissions.md:60-77`
  table lists 10 default roles with capability grants.
- **evidence:** `grep -nE "AttendanceStudent|AttendanceSubject|AttendanceStaff|AttendanceImport|AttendanceExam|AttendanceBulkMark|AttendanceReport|AttendanceNotify" crates/cross-cutting/rbac/src/services.rs`
  returns only `HrAttendance*` capabilities (the HR-domain
  capabilities added in Phase 6), no `Attendance*` capabilities
  (the attendance-domain ones added in Phase 5).

---

### FINDING 41

- **id:** DOMAIN-ATT-041
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/attendance/src/services.rs` (entire file)
- **description:** The 14 service functions are not gated by
  capability checks. AGENTS.md § "Engine Rules" requires
  "Capabilities are checked at the command boundary. The
  engine never trusts the caller to assert their own role."
  Per `docs/specs/attendance/permissions.md:82-91` and the
  spec commands.md (e.g. `MarkStudentAttendance` capability
  `Attendance.Mark`, `BulkMarkStudentAttendance` capability
  `Attendance.BulkMark`, etc.), every service should check the
  capability before mutating. The Phase 5 handoff (lines
  363-391) documents this as a deliberate boundary (matching
  the academic/assessment pattern), but the absence of the
  check means the capability check is the dispatcher's job —
  and the dispatcher has not been built yet.
- **expected:** `docs/specs/attendance/permissions.md:82-91`
  shows the canonical capability check pattern.
- **evidence:** `grep -nE "capability_check|has\(|Capability::Attendance" crates/domains/attendance/src/services.rs`
  returns no result.

---

### FINDING 42

- **id:** DOMAIN-ATT-042
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/attendance/src/aggregate.rs:60-109`
- **description:** `StudentAttendance` carries both a
  `school_id: SchoolId` field (line 66) and the typed id's
  embedded `school_id` (`id.school_id()`). The two fields can
  drift out of sync. The same redundancy exists on
  `StaffAttendance` (line 197), `SubjectAttendance` (line 300),
  `ExamAttendance` (line 419), and `BulkAttendanceImport`
  (line 539). The engine pattern is to derive `school_id` from
  the typed id; carrying a duplicate field opens the door to
  invariant violations.
- **expected:** A single source of truth for the school anchor
  — the typed id.
- **evidence:** `crates/domains/attendance/src/aggregate.rs:66`:
  `pub school_id: SchoolId,` and the `fresh()` constructor at
  line 136: `school_id: id.school_id(),`.

---

### FINDING 43

- **id:** DOMAIN-ATT-043
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/attendance/src/aggregate.rs:81-91`
- **description:** `StudentAttendance` carries a
  `is_absent: bool` field (line 91) that duplicates
  `attendance_type.is_absent()`. The same redundancy is
  declared in the spec language at `aggregates.md:25-29`
  (invariant 5: "If `is_absent=true`, then
  `attendance_type=Absent`"), which means the spec mandates
  the duplication. The fields can drift out of sync; the
  `update_student_attendance` service (`services.rs:200`)
  updates both atomically, but the aggregate struct allows the
  invariant to be violated via direct field assignment in
  tests. The engine pattern is to derive `is_absent` from
  `attendance_type` rather than store it as a separate field.
- **expected:** `docs/specs/attendance/aggregates.md:25-29` —
  invariant 5 couples the two, but the spec doesn't say the
  field is required.
- **evidence:** `crates/domains/attendance/src/aggregate.rs:81, 91`:
  both `pub attendance_type: AttendanceType,` and
  `pub is_absent: bool,`. Compare with
  `StaffAttendance::is_absent()` (line 282) which is a
  const-method that derives from `attendance_type`.

---

### FINDING 44

- **id:** DOMAIN-ATT-044
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/attendance/src/aggregate.rs:540-541`
- **description:** `BulkAttendanceImport` carries an
  `academic_year_id: AcademicYearId` field on the aggregate
  but the spec (`aggregates.md:148-149`) lists it only as
  invariant 1 ("A bulk import belongs to exactly one school
  and one academic year") without specifying it as a field on
  the root. The `ImportAttendanceCommand` carries
  `academic_year_id` (`commands.rs:339`), but no spec file
  declares it on the aggregate.
- **expected:** `docs/specs/attendance/aggregates.md:129-175`
  doesn't list `academic_year_id` as a field, only as an
  invariant.
- **evidence:** `crates/domains/attendance/src/aggregate.rs:541`:
  `pub academic_year_id: AcademicYearId,`.

---

### FINDING 45

- **id:** DOMAIN-ATT-045
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/attendance/src/services.rs:1083-1095`
- **description:** `commit_bulk_import` synthesises placeholder
  typed ids for `student_record_id`, `class_id`, and
  `section_id` from the event id:
  `StudentRecordId::new(import.school_id, event_id_to_uuid(event_id))`,
  `ClassId::new(import.school_id, event_id_to_uuid(event_id))`,
  `SectionId::new(import.school_id, event_id_to_uuid(event_id))`.
  These placeholder ids propagate into the persisted
  `StudentAttendance` aggregate and the
  `StudentAttendanceImported` event payload. The comment
  acknowledges the placeholder but the production path
  cannot recover the original ids.
- **expected:** The commit path should resolve the typed ids
  from the enrollment table; the Phase 5 stub is acceptable
  for Phase 5 but the placeholder UUID must not reach
  production.
- **evidence:** `crates/domains/attendance/src/services.rs:1083-1095`:
  ```rust
  crate::value_objects::StudentRecordId::new(
      import.school_id,
      event_id_to_uuid(event_id),
  ),
  crate::value_objects::ClassId::new(import.school_id, event_id_to_uuid(event_id)),
  crate::value_objects::SectionId::new(import.school_id, event_id_to_uuid(event_id)),
  ```

---

### FINDING 46

- **id:** DOMAIN-ATT-046
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/attendance/src/services.rs:286-302`
- **description:** `bulk_mark_student_attendance` produces 1
  extra `StudentAttendance` aggregate for the "default type"
  cohort of unmarked students. This is a stub path: the spec
  (`commands.md:128-151`) requires
  `BulkMarkStudentAttendance` to "creates or replaces
  `StudentAttendance` rows for all students in the section".
  The service emits one aggregate per absent / late / half-day
  id plus one extra aggregate for the unmarked cohort
  (totaling `absent_ids.len() + late_ids.len() + half_day_ids.len() + 1`
  per `BulkMarkResult`). The unmarked aggregate has a
  placeholder `StudentId` (see DOMAIN-ATT-037). The
  integration test at
  `crates/tools/storage-parity/tests/attendance_integration.rs:330-338`
  documents this as a "Phase 5 stub" but the assertion is
  embedded in the test as if it were production behaviour.
- **expected:** `docs/specs/attendance/commands.md:128-151`
  mandates per-student rows, not per-overridden-id + 1.
- **evidence:** `crates/domains/attendance/src/services.rs:283-333`:
  the unmarked default-type aggregate is emitted before the
  per-id loops. The integration test
  `attendance_integration.rs:331-338` asserts
  `outcome.aggregates_len == 201` for 200 absent ids (200 +
  the default-type aggregate).

---

### FINDING 47

- **id:** DOMAIN-ATT-047
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/attendance/src/services.rs:1187-1216` and `docs/specs/attendance/services.md:51`
- **description:** `AttendanceService::emit_absence_event`
  (line 1249) takes a single `&StudentAttendance` parameter
  instead of the spec's two-parameter signature
  `emit_absence_event(attendance: &StudentAttendance, school_id: SchoolId) -> Option<StudentAbsentForDay>`
  (`services.md:43-46`). The spec signature separates the
  attendance row from the school id; the code merges them.
  This is a wire-contract drift between the service helper
  and the spec.
- **expected:** `docs/specs/attendance/services.md:43-46`:
  `pub fn emit_absence_event(attendance: &StudentAttendance, school_id: SchoolId) -> Option<StudentAbsentForDay>;`
- **evidence:** `crates/domains/attendance/src/services.rs:1249-1266`:
  `pub fn emit_absence_event(row: &StudentAttendance) -> Option<StudentAbsentForDay>`.

---

### FINDING 48

- **id:** DOMAIN-ATT-048
- **area:** domain-crates
- **severity:** Low
- **location:** `crates/domains/attendance/src/services.rs:1274`
- **description:** The `dedup_within_day` helper uses
  `std::collections::HashSet<(uuid::Uuid, chrono::NaiveDate)>`
  to dedup `StudentAbsentForDay` events. Per the engine rule
  "Multi-tenant by default" (AGENTS.md) and the spec invariant
  "every attendance aggregate is anchored to exactly one
  `SchoolId`" (`overview.md:61`), the dedup key must include
  the `SchoolId` to prevent cross-tenant collisions. The
  current key uses `(student_uuid, date)` and would conflate
  two schools' students with id collision (unlikely but
  possible).
- **expected:** The dedup key should be
  `(school_id, student_id, attendance_date)` per the spec's
  uniqueness invariants.
- **evidence:** `crates/domains/attendance/src/services.rs:1274-1284`:
  ```rust
  let mut seen: std::collections::HashSet<(uuid::Uuid, chrono::NaiveDate)> =
      std::collections::HashSet::with_capacity(events.len());
  ```

---

### FINDING 49

- **id:** DOMAIN-ATT-049
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/attendance/src/repository.rs:54-127`
- **description:** `StudentAttendanceRepository::find` does
  not match the spec's
  `find(school: SchoolId, student: StudentId, date: NaiveDate)`
  signature exactly. The code matches it on parameters, but
  the spec also mandates a `find` for
  `SubjectAttendanceRepository`
  (`repositories.md:53-63`:
  `find(school, student, subject, date)`), which is missing
  from the code's `SubjectAttendanceRepository` trait
  (`repository.rs:138-168`). The trait ships `get` and
  `list_for_student` and `list_for_section` but no
  `find(school, student, subject, date)`. The `mark_subject_attendance`
  service calls
  `uniqueness.subject_day_exists(school, student, subject, date)`
  which is the spec's behaviour, but the repository trait
  itself cannot serve a query for an individual row.
- **expected:** `docs/specs/attendance/repositories.md:53-63`.
- **evidence:**
  - `docs/specs/attendance/repositories.md:53-63` mandates
    `async fn find(school, student, subject, date)`.
  - `grep -n "fn find" crates/domains/attendance/src/repository.rs`
    only finds `find` on `StudentAttendanceRepository`
    (line 66) and `StaffAttendanceRepository` (line 186), not
    on `SubjectAttendanceRepository`.

---

### FINDING 50

- **id:** DOMAIN-ATT-050
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/attendance/src/repository.rs:138-168`
- **description:** `SubjectAttendanceRepository` ships only 5
  methods. The spec
  (`repositories.md:53-80`) mandates 6 methods including
  `list_for_section_date(school, class, section, subject,
  date)`. The code trait has `list_for_section(school, class,
  section)` (no date filter) and no method that filters by
  subject and date.
- **expected:** `docs/specs/attendance/repositories.md:63-70`.
- **evidence:**
  - `docs/specs/attendance/repositories.md:63-70` mandates
    `list_for_section_date`.
  - `grep -n "list_for_section_date" crates/domains/attendance/src/repository.rs`
    returns no result.

---

### FINDING 51

- **id:** DOMAIN-ATT-051
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/attendance/src/repository.rs:179-213`
- **description:** `StaffAttendanceRepository` ships 7 methods.
  The spec (`repositories.md:85-115`) mandates 8 methods,
  including `list_for_school_in_range(school, from, to)`. The
  code ships `list_for_day(school, date)` (a single-date
  query, not a range query) and `list_for_staff(school, staff,
  from, to)` (per-staff range query), but no
  `list_for_school_in_range` for all-staff-per-range queries.
- **expected:** `docs/specs/attendance/repositories.md:102-107`.
- **evidence:**
  - `docs/specs/attendance/repositories.md:102-107` mandates
    `list_for_school_in_range(school, from, to)`.
  - `grep -n "list_for_school_in_range" crates/domains/attendance/src/repository.rs`
    returns no result; `list_for_day` (line 201) is the
    closest match.

---

### FINDING 52

- **id:** DOMAIN-ATT-052
- **area:** domain-crates
- **severity:** Low
- **location:** `crates/domains/attendance/src/events.rs:3-23`
- **description:** The events module's documentation claims
  "Phase 5 Workstream A ships 21 typed DomainEvent
  implementations" but the actual count is 22
  (`StudentAttendanceMarked`, `StudentAttendanceUpdated`,
  `StudentAttendanceRestored`, `StudentAbsentForDay`,
  `StudentAttendanceImported`, `SubjectAttendanceMarked`,
  `SubjectAttendanceUpdated`,
  `SubjectAbsentNotificationRequested`, `StaffAttendanceMarked`,
  `StaffAttendanceUpdated`, `StaffAbsentForDay`,
  `ExamAttendanceMarked`, `ExamAttendanceUpdated`,
  `BulkImportStarted`, `BulkImportValidated`,
  `BulkImportCommitted`, `BulkImportFailed`,
  `BulkImportCancelled`, `AttendanceImported`,
  `AbsenceNotificationRequested`, `ClassAttendanceRecomputed`).
  The handoff (line 89) also says "21 typed events". The
  module-level doc and the handoff both undercount by 1.
- **expected:** The events module's module-level documentation
  should list the correct count.
- **evidence:**
  - `crates/domains/attendance/src/events.rs:3-23` declares
    "Phase 5 Workstream A ships 21 typed DomainEvent
    implementations" and lists 21 in the bullet list but the
    bullet lists 21 (Student=5, Subject=3, Staff=3, Exam=2,
    BulkImport=6, Cross-cutting=2 = 21); the actual count
    in the file (grep `impl DomainEvent for`) is 22.
  - `grep -c "impl DomainEvent for" crates/domains/attendance/src/events.rs`
    returns 22.

---

### FINDING 53

- **id:** DOMAIN-ATT-053
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/attendance/src/lib.rs:1-97` and `docs/handoff/PHASE-5-HANDOFF.md:104-122`
- **description:** `lib.rs` is missing `AttendanceEligibility`,
  `AbsenceDetectionService`, `AttendanceReportService`,
  `AttendanceImportService`, `ClassAttendanceRepository`,
  `ClassAttendance`, `AttendanceBulk`, the `*Event` alias
  types, and `Validate` trait from its prelude re-exports —
  none are declared in the crate. The handoff at
  `docs/handoff/PHASE-5-HANDOFF.md:104-122` is internally
  consistent (it doesn't claim these exist) but the lib.rs
  re-exports section will need to be updated when the missing
  types land.
- **expected:** The lib.rs prelude should re-export the
  complete Phase 5 public surface.
- **evidence:**
  - `crates/domains/attendance/src/lib.rs:30-97` re-exports
    aggregates, commands, events, services, repositories,
    value objects — but no `AbsenceDetectionService`,
    `AttendanceReportService`, `AttendanceImportService`,
    `ClassAttendanceRepository`, `ClassAttendance`,
    `AttendanceBulk`, etc.
  - The handoff
    (`docs/handoff/PHASE-5-HANDOFF.md:115-122`) only
    describes the 5 repository traits shipped.

---

### END FINDINGS

Total findings: **53**.
