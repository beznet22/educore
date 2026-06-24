# Attendance Domain — Aggregates

## StudentAttendance

**Root type:** `StudentAttendance`
**Identity:** `StudentAttendanceId(SchoolId, Uuid)`
**Tenant:** `SchoolId`

### Purpose

The daily presence record for a single student in a class-section.
There is at most one `StudentAttendance` per `(student, date)`.

### Owned Children

- (None — daily attendance is a single row.)

### Invariants

1. Unique by `(school_id, student_id, attendance_date)` per
   academic year.
2. The `attendance_date` is not in the future.
3. A student cannot be both `Present` and `Absent`.
4. Updates append a new event; the latest row replaces the
   previous state for read.
5. If `is_absent=true`, then `attendance_type=Absent` and
   `notes` may record the reason.
6. The class-section recorded on the row must match the student's
   `StudentRecord` for the date.
7. The `MarkedBy` user must be authorized (`Attendance.Mark` or
   `Attendance.Update`).

### Commands

- `MarkStudentAttendance`
- `UpdateStudentAttendance`
- `BulkMarkStudentAttendance`

### Events

- `StudentAttendanceMarked`
- `StudentAttendanceUpdated`
- `StudentAbsentForDay` (derived: emitted on the first transition
  into `Absent` for the day)
- `StudentAttendanceImported` (when produced by a bulk import)

---

## SubjectAttendance

**Root type:** `SubjectAttendance`
**Identity:** `SubjectAttendanceId(SchoolId, Uuid)`

### Purpose

The per-period (per-subject) presence record for a single student.
There is at most one `SubjectAttendance` per
`(student, subject, date)`. Used when a school takes attendance
per period instead of (or in addition to) once per day.

### Invariants

1. Unique by `(school_id, student_id, subject_id,
   attendance_date)`.
2. The subject must be assigned to the student's class-section
   for the date.
3. A subject marked `Absent` and the same student marked `Present`
   for the day is a conflict; the operator must reconcile.
4. `Notify=true` indicates a notification has been requested for
   this absence (e.g. parent SMS).

### Commands

- `MarkSubjectAttendance`
- `UpdateSubjectAttendance`

### Events

- `SubjectAttendanceMarked`
- `SubjectAttendanceUpdated`

---

## StaffAttendance

**Root type:** `StaffAttendance`
**Identity:** `StaffAttendanceId(SchoolId, Uuid)`

### Purpose

The daily presence record for a staff member (teacher or
non-teaching staff). There is at most one `StaffAttendance` per
`(staff, date)`.

### Invariants

1. Unique by `(school_id, staff_id, attendance_date)`.
2. The staff member must be active (not terminated) on the date.
3. A staff member on approved leave is `OnLeave`, not `Absent`.
4. Late arrival is allowed; `Late` is a status, not an automatic
   deduction.

### Commands

- `MarkStaffAttendance`
- `UpdateStaffAttendance`

### Events

- `StaffAttendanceMarked`
- `StaffAttendanceUpdated`
- `StaffAbsentForDay` (derived)

---

## ExamAttendance

The exam-day per-subject attendance aggregate is owned by the
**assessment** domain. It is documented in
`docs/specs/assessment/aggregates.md`. The attendance domain
consumes its `ExamAttendanceMarked` events to produce the
`ClassAttendance` summary used in reports.

The `ExamAttendanceChild` entity (per-student mark) is similarly
owned by the assessment domain.

---

## BulkAttendanceImport

**Root type:** `BulkAttendanceImport`
**Identity:** `BulkAttendanceImportId(SchoolId, Uuid)`

### Purpose

A bulk import job that ingests attendance from a CSV, an external
biometric device, or another system. The job produces zero or more
`StudentAttendance` rows.

### Owned Children

- `StudentAttendanceImport` — staging rows.
- (On commit, each row promotes into a `StudentAttendance` and a
  matching event is emitted.)

### Invariants

1. A bulk import belongs to exactly one school and one academic
   year.
2. The import's `Source` is a string identifier (e.g. "biometric-1",
   "csv-may-2026").
3. The import is idempotent on
   `(school_id, source, attendance_date)`. A duplicate is rejected.
4. The import may be `Pending`, `Validated`, `Committed`, or
   `Failed`.
5. A failed import does not produce any attendance rows; the
   staging rows carry the failure reason.
6. The import's `MarkedBy` is the user that initiated the upload.

### Commands

- `ImportAttendance` (creates the job and stages the rows)
- `ValidateBulkImport` (runs validation; emits validation events)
- `CommitBulkImport` (commits the rows as `StudentAttendance` rows)
- `CancelBulkImport`

### Events

- `BulkImportStarted`
- `BulkImportValidated`
- `BulkImportCommitted`
- `BulkImportFailed`
- `BulkImportCancelled`
- `AttendanceImported` (one per committed row)

---

## StudentAttendanceImport

**Identity:** `StudentAttendanceImportId(SchoolId, Uuid)`
**Owner:** `BulkAttendanceImport`

A staging row. Carries the `StudentId`, `AttendanceDate`, `InTime`,
`OutTime`, `AttendanceType`, `Notes`. On commit, promotes to a
`StudentAttendance`.

### Invariants

1. Belongs to exactly one `BulkAttendanceImport`.
2. Validates against the school's `StudentRecord` for the date
   (the student must be enrolled).

---

## StaffAttendanceImport

**Identity:** `StaffAttendanceImportId(SchoolId, Uuid)`
**Owner:** `BulkAttendanceImport`

A staging row for a staff bulk import (when the school bulk-imports
staff attendance from a biometric or HR system).

### Invariants

1. Belongs to exactly one `BulkAttendanceImport`.
2. Validates against the active staff roster for the date.

---

## ClassAttendance

**Identity:** `ClassAttendanceId(SchoolId, Uuid)`
**Owner:** `School`

A per-(student, exam_type, academic_year) summary of days opened,
days present, and days absent. Used in report cards and reports.
Populated by domain services that consume `StudentAttendanceMarked`
and `ExamAttendanceMarked` events.

### Invariants

1. Unique by `(school_id, student_id, exam_type_id,
   academic_id)`.
2. `days_opened = days_present + days_absent + days_on_leave +
   days_half_day * 0.5 + days_late` (the school may define a
   custom mapping).

### Commands

- (None — `ClassAttendance` is a projection; the engine
  recomputes it on demand from the underlying events and rows.)

---

## AttendanceBulk

**Identity:** `AttendanceBulkId(SchoolId, Uuid)`
**Owner:** `BulkAttendanceImport`

A per-(student, date) row materialized during a bulk import. The
table is a denormalized staging representation used by the
consumer's import wizard. On commit, the engine promotes each row
into a `StudentAttendance`.

### Invariants

1. Belongs to exactly one `BulkAttendanceImport`.

---


The `ExamAttendanceChild` entity is owned by the assessment domain.
The attendance domain reads it for report generation but does not
own its lifecycle.
