# Attendance Domain — Events

Domain events describe facts that have already happened. They are
immutable, append-only records used for cross-domain integration,
audit, and event sourcing.

All events implement:

```rust
pub trait DomainEvent: Serialize + DeserializeOwned + Send + Sync {
    const TYPE: &'static str;
    fn aggregate_id(&self) -> Uuid;
    fn school_id(&self) -> SchoolId;
    fn occurred_at(&self) -> Timestamp;
}
```

The event envelope wraps the event with metadata:

```rust
pub struct EventEnvelope<E> {
    pub event_id: EventId,
    pub event_type: &'static str,
    pub school_id: SchoolId,
    pub aggregate_id: Uuid,
    pub aggregate_type: &'static str,
    pub actor_id: UserId,
    pub correlation_id: CorrelationId,
    pub causation_id: Option<EventId>,
    pub occurred_at: Timestamp,
    pub payload: E,
}
```

## Student Attendance

```rust
pub struct StudentAttendanceMarked {
    pub student_attendance_id: StudentAttendanceId,
    pub student_id: StudentId,
    pub student_record_id: StudentRecordId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub attendance_date: AttendanceDate,
    pub attendance_type: AttendanceType,
    pub notes: Option<String>,
    pub marked_by: UserId,
    pub marked_at: Timestamp,
    pub marked_from: AttendanceSource,
}
```

**Subscribers:**
- `attendance` (self) — update `ClassAttendance` summary.
- `assessment` — none directly; the daily summary feeds reports.

```rust
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

pub struct StudentAttendanceRestored {
    pub student_attendance_id: StudentAttendanceId,
    pub student_id: StudentId,
    pub attendance_date: AttendanceDate,
    pub from_type: AttendanceType, // typically A
    pub to_type: AttendanceType,   // typically P
    pub restored_by: UserId,
    pub restored_at: Timestamp,
}

pub struct StudentAbsentForDay {
    pub student_id: StudentId,
    pub student_record_id: StudentRecordId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub attendance_date: AttendanceDate,
    pub detected_at: Timestamp,
    pub notify: bool, // mirrors the MarkStudentAttendance.notify flag
}
```

**Subscribers of `StudentAbsentForDay`:**
- `communication` — sends SMS/email/push to the guardian.
- `finance` — may apply absence fine if the school's policy
  applies.
- `academic` — flags the student in the daily summary for the
  class teacher.
- `assessment` — feeds the per-exam attendance summary used in
  report cards.

```rust
pub struct StudentAttendanceImported {
    pub import_id: BulkAttendanceImportId,
    pub student_id: StudentId,
    pub attendance_date: AttendanceDate,
    pub attendance_type: AttendanceType,
    pub source: AttendanceSource,
    pub imported_at: Timestamp,
}
```

## Subject Attendance

```rust
pub struct SubjectAttendanceMarked {
    pub subject_attendance_id: SubjectAttendanceId,
    pub student_id: StudentId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub subject_id: SubjectId,
    pub attendance_date: AttendanceDate,
    pub attendance_type: AttendanceType,
    pub notes: Option<String>,
    pub marked_by: UserId,
    pub marked_at: Timestamp,
    pub marked_from: AttendanceSource,
}

pub struct SubjectAttendanceUpdated {
    pub subject_attendance_id: SubjectAttendanceId,
    pub student_id: StudentId,
    pub subject_id: SubjectId,
    pub attendance_date: AttendanceDate,
    pub from_type: AttendanceType,
    pub to_type: AttendanceType,
    pub updated_by: UserId,
    pub updated_at: Timestamp,
}

pub struct SubjectAbsentNotificationRequested {
    pub subject_attendance_id: SubjectAttendanceId,
    pub student_id: StudentId,
    pub subject_id: SubjectId,
    pub attendance_date: AttendanceDate,
    pub channel: NotificationChannel,
    pub template: NotificationTemplate,
    pub requested_by: UserId,
    pub requested_at: Timestamp,
}
```

**Subscribers of `SubjectAbsentNotificationRequested`:**
- `communication` — sends the notification via the requested
  channel.

## Staff Attendance

```rust
pub struct StaffAttendanceMarked {
    pub staff_attendance_id: StaffAttendanceId,
    pub staff_id: StaffId,
    pub attendance_date: AttendanceDate,
    pub attendance_type: AttendanceType,
    pub notes: Option<String>,
    pub marked_by: UserId,
    pub marked_at: Timestamp,
    pub marked_from: AttendanceSource,
}

pub struct StaffAttendanceUpdated {
    pub staff_attendance_id: StaffAttendanceId,
    pub staff_id: StaffId,
    pub attendance_date: AttendanceDate,
    pub from_type: AttendanceType,
    pub to_type: AttendanceType,
    pub updated_by: UserId,
    pub updated_at: Timestamp,
}

pub struct StaffAbsentForDay {
    pub staff_id: StaffId,
    pub attendance_date: AttendanceDate,
    pub detected_at: Timestamp,
}
```

**Subscribers of `StaffAbsentForDay`:**
- `hr` — may deduct leave or trigger a substitute workflow.
- `finance` — applies no-pay rules where configured.
- `communication` — notifies the school admin and the staff's
  department.

## Bulk Import

```rust
pub struct BulkImportStarted {
    pub bulk_import_id: BulkAttendanceImportId,
    pub source: AttendanceSource,
    pub row_count: u32,
    pub started_by: UserId,
    pub started_at: Timestamp,
}

pub struct BulkImportValidated {
    pub bulk_import_id: BulkAttendanceImportId,
    pub validated_by: UserId,
    pub validated_at: Timestamp,
    pub row_count: u32,
}

pub struct BulkImportCommitted {
    pub bulk_import_id: BulkAttendanceImportId,
    pub committed_by: UserId,
    pub committed_at: Timestamp,
    pub row_count: u32,
    pub absent_count: u32,
}

pub struct BulkImportFailed {
    pub bulk_import_id: BulkAttendanceImportId,
    pub reason: String,
    pub row_errors: Vec<RowError>,
    pub failed_at: Timestamp,
}

pub struct BulkImportCancelled {
    pub bulk_import_id: BulkAttendanceImportId,
    pub cancelled_by: UserId,
    pub reason: String,
    pub cancelled_at: Timestamp,
}
```

`RowError` is a typed value object describing a single row's
validation failure (`RowIndex`, `StudentId`, `Reason`).

## Notification

```rust
pub struct AbsenceNotificationRequested {
    pub student_attendance_id: StudentAttendanceId,
    pub student_id: StudentId,
    pub attendance_date: AttendanceDate,
    pub channel: NotificationChannel,
    pub template: NotificationTemplate,
    pub requested_by: UserId,
    pub requested_at: Timestamp,
}
```

**Subscribers of `AbsenceNotificationRequested`:**
- `communication` — sends the notification.

## Class Attendance (projection)

`ClassAttendance` is recomputed from the underlying events; the
engine may publish a `ClassAttendanceRecomputed` event for
consumers that want a stream of summary updates:

```rust
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

## Audit

All events are recorded in the per-aggregate event log and emitted
on the event bus. Consumers and adapters consume from the bus to
update projections, send notifications, render reports, and feed
downstream domains.
