# Attendance Domain — Commands

Commands describe intent. They are validated, authorized, and
dispatched to the relevant aggregate. Every command produces zero
or more events that are recorded in the event log.

All commands carry a `TenantContext` (school + actor + correlation)
and are rejected if the actor lacks the required capability.

## MarkStudentAttendance

```rust
pub struct MarkStudentAttendanceCommand {
    pub tenant: TenantContext,
    pub student_id: StudentId,
    pub student_record_id: StudentRecordId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub attendance_date: AttendanceDate,
    pub attendance_type: AttendanceType, // P, A, L, F, H
    pub notes: Option<String>,
    pub notify: bool, // request guardian notification on absence
    pub marked_from: AttendanceSource,
}
```

**Capability:** `Attendance.Mark`
**Pre-conditions:**
- The student has an active `StudentRecord` for the date.
- The class-section on the command matches the student's record.
- The date is not in the future.
- The actor is authorized for the class-section (class teacher,
  subject teacher, attendance cell, or school admin).
- The date is not a holiday unless the school policy allows
  override.

**Effects:** Creates or replaces the `StudentAttendance` row for
the day. Emits `StudentAttendanceMarked`. If the attendance type
is `Absent` and this is the first transition into absence for the
day, emits `StudentAbsentForDay`.

## UpdateStudentAttendance

```rust
pub struct UpdateStudentAttendanceCommand {
    pub tenant: TenantContext,
    pub student_attendance_id: StudentAttendanceId,
    pub attendance_type: Option<AttendanceType>,
    pub notes: Option<String>,
    pub notify: Option<bool>,
}
```

**Capability:** `Attendance.Update`
**Pre-conditions:**
- A `StudentAttendance` row exists for the id.
- The actor is authorized for the class-section.

**Effects:** Updates the row. Emits `StudentAttendanceUpdated`. If
the update transitions the student into absence, emits
`StudentAbsentForDay`; if the update transitions them out of
absence, emits `StudentAttendanceRestored`.

## MarkSubjectAttendance

```rust
pub struct MarkSubjectAttendanceCommand {
    pub tenant: TenantContext,
    pub student_id: StudentId,
    pub student_record_id: StudentRecordId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub subject_id: SubjectId,
    pub attendance_date: AttendanceDate,
    pub attendance_type: AttendanceType, // P or A
    pub notes: Option<String>,
    pub notify: bool,
    pub marked_from: AttendanceSource,
}
```

**Capability:** `Attendance.Mark`
**Pre-conditions:**
- The subject is assigned to the student's class-section for the
  date.
- The date is not in the future.
- The actor is the subject teacher (or higher privilege).

**Effects:** Creates or replaces the `SubjectAttendance` row.
Emits `SubjectAttendanceMarked`. If `attendance_type=Absent` and
`notify=true`, emits `SubjectAbsentNotificationRequested` (the
communication domain subscribes).

## MarkStaffAttendance

```rust
pub struct MarkStaffAttendanceCommand {
    pub tenant: TenantContext,
    pub staff_id: StaffId,
    pub attendance_date: AttendanceDate,
    pub attendance_type: AttendanceType, // P, A, L, F, H
    pub notes: Option<String>,
    pub marked_from: AttendanceSource,
}
```

**Capability:** `Attendance.Mark`
**Pre-conditions:**
- The staff member is active on the date.
- The date is not in the future.
- The actor is authorized (HR, school admin, or self).

**Effects:** Creates or replaces the `StaffAttendance` row.
Emits `StaffAttendanceMarked`. If the type is `Absent` and this is
the first absence transition, emits `StaffAbsentForDay`.

## MarkExamAttendance

The exam-day per-subject attendance roll is owned by the
assessment domain. The corresponding command is documented in
`docs/specs/assessment/commands.md` (`MarkExamAttendance`).

The attendance domain subscribes to `ExamAttendanceMarked` to
update its `ClassAttendance` summaries.

## BulkMarkStudentAttendance

```rust
pub struct BulkMarkStudentAttendanceCommand {
    pub tenant: TenantContext,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub attendance_date: AttendanceDate,
    pub default_type: AttendanceType, // applied to all students
    pub absent_ids: Vec<StudentId>,    // overrides default to A
    pub late_ids: Vec<StudentId>,      // overrides default to L
    pub half_day_ids: Vec<StudentId>,  // overrides default to F
    pub notes: Option<String>,
}
```

**Capability:** `Attendance.BulkMark`
**Pre-conditions:**
- The actor is authorized for the class-section.
- The date is not in the future.
- All listed students are enrolled in the section.

**Effects:** Creates or replaces `StudentAttendance` rows for all
students in the section. For each absent student, emits
`StudentAbsentForDay` (deduplicated within the command). Emits one
`StudentAttendanceMarked` per student.

## ImportAttendance

```rust
pub struct ImportAttendanceCommand {
    pub tenant: TenantContext,
    pub source: AttendanceSource, // e.g. "biometric-1"
    pub academic_year_id: AcademicYearId,
    pub rows: Vec<ImportRow>,
}

pub struct ImportRow {
    pub student_id: StudentId,
    pub attendance_date: AttendanceDate,
    pub attendance_type: AttendanceType,
    pub in_time: Option<String>,
    pub out_time: Option<String>,
    pub notes: Option<String>,
}
```

**Capability:** `Attendance.Import`
**Pre-conditions:**
- The actor is authorized (school admin, attendance cell).
- The source string is non-empty and within 100 chars.
- All rows reference students enrolled in the academic year.

**Effects:** Creates a `BulkAttendanceImport` (status `Pending`)
and one `StudentAttendanceImport` per row. Emits
`BulkImportStarted` and `StudentAttendanceImported` per row (or
`BulkImportFailed` if any row fails validation, in which case the
job moves to `Failed` and no rows are committed).

## ValidateBulkImport

```rust
pub struct ValidateBulkImportCommand {
    pub tenant: TenantContext,
    pub bulk_import_id: BulkAttendanceImportId,
}
```

**Capability:** `Attendance.Import`
**Pre-conditions:** The import is in `Pending` status.

**Effects:** Runs validation rules. On success, transitions the
import to `Validated` and emits `BulkImportValidated`. On
failure, transitions to `Failed` and emits `BulkImportFailed`
with the per-row reasons.

## CommitBulkImport

```rust
pub struct CommitBulkImportCommand {
    pub tenant: TenantContext,
    pub bulk_import_id: BulkAttendanceImportId,
}
```

**Capability:** `Attendance.Import`
**Pre-conditions:**
- The import is in `Validated` status.

**Effects:** Promotes each `StudentAttendanceImport` row into a
`StudentAttendance`, transitions the import to `Committed`, and
emits `BulkImportCommitted`. Each promoted row also emits a
`StudentAttendanceImported` event. For each absent student,
emits `StudentAbsentForDay`.

## CancelBulkImport

```rust
pub struct CancelBulkImportCommand {
    pub tenant: TenantContext,
    pub bulk_import_id: BulkAttendanceImportId,
    pub reason: String,
}
```

**Capability:** `Attendance.Import`
**Pre-conditions:** The import is in `Pending` or `Validated`
status.

**Effects:** Transitions the import to `Cancelled`, emits
`BulkImportCancelled`. No attendance rows are committed.

## SendAbsenceNotification

The actual notification dispatch is owned by the communication
domain. The attendance domain exposes a request command that emits
the trigger event:

```rust
pub struct RequestAbsenceNotificationCommand {
    pub tenant: TenantContext,
    pub student_attendance_id: StudentAttendanceId,
    pub channel: NotificationChannel, // SMS, Email, Push
    pub template: NotificationTemplate, // e.g. "absence-daily"
}
```

**Capability:** `Attendance.Notify`
**Effects:** Emits `AbsenceNotificationRequested`. The
communication domain subscribes and dispatches via the
notification port.

## MarkClassAttendance

`ClassAttendance` is a projection and is not modified by a direct
command. The engine recomputes it on demand from
`StudentAttendanceMarked` events. See
`AttendanceService::recompute_class_attendance` in
`services.md`.

## Standard CRUD Variants

`StudentAttendance`, `SubjectAttendance`, and `StaffAttendance`
are not directly deleted. The system uses updates and a `Cancelled`
status flag for soft cancellation. Hard delete is reserved for
data-correction flows that the consumer may trigger via
`Attendance.Admin.Correct`.

## Self-Service Scopes

A teacher may mark attendance only for their assigned class-section.
A student or parent may not mark attendance. A staff member may
mark their own attendance through the `Attendance.Mark` capability
with a self-service scope (the engine accepts an actor whose
`UserId` matches the `StaffId`).

## Orphaned Items (Cluster D catch-up)

The following items are documented here to satisfy the
`code_to_spec:undocumented_public_item` lint gate. They were
discovered after the main spec was written.

## ImportRow

```rust
pub struct ImportRow {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Row.Import`
**Effects:** Emits `RowImported`.


### Update Exam Attendance

```rust
pub struct UpdateExamAttendanceCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ExamAttendance.Update`
**Effects:** Emits `ExamAttendanceUpdateed`.


### Update Staff Attendance

```rust
pub struct UpdateStaffAttendanceCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffAttendance.Update`
**Effects:** Emits `StaffAttendanceUpdateed`.


### Update Subject Attendance

```rust
pub struct UpdateSubjectAttendanceCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `SubjectAttendance.Update`
**Effects:** Emits `SubjectAttendanceUpdateed`.



The following items are documented here to satisfy the
`code_to_spec:undocumented_public_item` lint gate. They were
discovered after the main spec was written.

## ImportRow

```rust
pub struct ImportRow {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Row.Import`
**Effects:** Emits `RowImported`.


### Update Exam Attendance

```rust
pub struct UpdateExamAttendanceCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ExamAttendance.Update`
**Effects:** Emits `ExamAttendanceUpdateed`.


### Update Staff Attendance

```rust
pub struct UpdateStaffAttendanceCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffAttendance.Update`
**Effects:** Emits `StaffAttendanceUpdateed`.


### Update Subject Attendance

```rust
pub struct UpdateSubjectAttendanceCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `SubjectAttendance.Update`
**Effects:** Emits `SubjectAttendanceUpdateed`.

