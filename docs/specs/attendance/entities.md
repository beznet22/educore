# Attendance Domain — Entities

Entities have identity and lifecycle but are not aggregate roots.
They are loaded and persisted only through their aggregate root.

## StudentAttendanceImport

**Identity:** `StudentAttendanceImportId(SchoolId, Uuid)`
**Owner:** `BulkAttendanceImport`

A staging row carrying `StudentId`, `AttendanceDate`, `InTime`,
`OutTime`, `AttendanceType`, `Notes`. On commit, the engine
promotes it into a `StudentAttendance`.

### Invariants

1. Belongs to exactly one `BulkAttendanceImport`.
2. The student must have an active `StudentRecord` on the date.
3. `InTime` and `OutTime` are free-text (the device's local time
   string); the engine does not parse them into a typed time.

## StaffAttendanceImport

**Identity:** `StaffAttendanceImportId(SchoolId, Uuid)`
**Owner:** `BulkAttendanceImport`

A staging row for a staff bulk import. `StaffId`, `AttendanceDate`,
`InTime`, `OutTime`, `AttendanceType`, `Notes`.

### Invariants

1. Belongs to exactly one `BulkAttendanceImport`.
2. The staff member must be active on the date.

## ClassAttendance

**Identity:** `ClassAttendanceId(SchoolId, Uuid)`
**Owner:** `School`

A per-(student, exam_type, academic_year) summary: `DaysOpened`,
`DaysAbsent`, `DaysPresent`. Used in report cards and reports.
Populated by the attendance service from the underlying events.

### Invariants

1. Unique by `(school_id, student_id, exam_type_id, academic_id)`.
2. `days_opened = days_present + days_absent + days_on_leave +
   days_half_day * 0.5`.

## ExamAttendanceChild (cross-reference)

The `ExamAttendanceChild` entity is owned by the assessment
domain. The attendance domain reads it for reports but does not
own its lifecycle.

## AttendanceBulk

**Identity:** `AttendanceBulkId(SchoolId, Uuid)`
**Owner:** `BulkAttendanceImport`

A denormalized staging row used by the consumer's bulk import
wizard. `AttendanceDate`, `AttendanceType`, `Note`, `StudentId`,
`StudentRecordId`, `ClassId`, `SectionId`.

### Invariants

1. Belongs to exactly one `BulkAttendanceImport`.
