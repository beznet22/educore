//! # Attendance domain events
//!
//! Phase 5 Workstream A ships 21 typed
//! [`DomainEvent`](educore_events::domain_event::DomainEvent)
//! implementations covering the 5 aggregate roots plus the
//! cross-cutting "absence notification requested" event:
//!
//! - **Student (5):** `StudentAttendanceMarked`,
//!   `StudentAttendanceUpdated`, `StudentAttendanceRestored`,
//!   `StudentAbsentForDay`, `StudentAttendanceImported`.
//! - **Subject (3):** `SubjectAttendanceMarked`,
//!   `SubjectAttendanceUpdated`,
//!   `SubjectAbsentNotificationRequested`.
//! - **Staff (3):** `StaffAttendanceMarked`,
//!   `StaffAttendanceUpdated`, `StaffAbsentForDay`.
//! - **Exam (2):** `ExamAttendanceMarked`,
//!   `ExamAttendanceUpdated`.
//! - **BulkImport (6):** `BulkImportStarted`,
//!   `BulkImportValidated`, `BulkImportCommitted`,
//!   `BulkImportFailed`, `BulkImportCancelled`,
//!   `AttendanceImported`.
//! - **Cross-cutting (2):** `AbsenceNotificationRequested`,
//!   `ClassAttendanceRecomputed`.
//!
//! Every event carries the standard 3-field footer
//! (`event_id`, `correlation_id`, `occurred_at`).

#![allow(clippy::too_many_arguments, missing_docs)]

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use educore_core::ids::{CorrelationId, EventId, SchoolId};
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;

use educore_assessment::ExamId;

use crate::value_objects::{
    AcademicYearId, AttendanceSource, AttendanceType, BulkAttendanceImportId, ClassAttendanceId,
    ClassId, ExamAttendanceId, SectionId, StaffAttendanceId, StaffId, StudentAttendanceId,
    StudentId, StudentRecordId, SubjectAttendanceId, SubjectId,
};

// =============================================================================
// StudentAttendanceMarked
// =============================================================================

/// Emitted when a [`StudentAttendance`](crate::aggregate::StudentAttendance)
/// is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StudentAttendanceMarked {
    pub student_attendance_id: StudentAttendanceId,
    pub student_id: StudentId,
    pub student_record_id: StudentRecordId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub attendance_date: NaiveDate,
    pub attendance_type: AttendanceType,
    pub notes: Option<String>,
    pub marked_by: educore_core::ids::UserId,
    pub marked_at: Timestamp,
    pub marked_from: AttendanceSource,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl StudentAttendanceMarked {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        student_attendance_id: StudentAttendanceId,
        student_id: StudentId,
        student_record_id: StudentRecordId,
        class_id: ClassId,
        section_id: SectionId,
        attendance_date: NaiveDate,
        attendance_type: AttendanceType,
        notes: Option<String>,
        marked_by: educore_core::ids::UserId,
        marked_at: Timestamp,
        marked_from: AttendanceSource,
        event_id: EventId,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            student_attendance_id,
            student_id,
            student_record_id,
            class_id,
            section_id,
            attendance_date,
            attendance_type,
            notes,
            marked_by,
            marked_at,
            marked_from,
            event_id,
            correlation_id,
            occurred_at: marked_at,
        }
    }
}

impl DomainEvent for StudentAttendanceMarked {
    const EVENT_TYPE: &'static str = "attendance.student.marked";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "student_attendance";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.student_attendance_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.student_attendance_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// StudentAttendanceUpdated
// =============================================================================

/// Emitted when a [`StudentAttendance`](crate::aggregate::StudentAttendance)
/// is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StudentAttendanceUpdated {
    pub student_attendance_id: StudentAttendanceId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl StudentAttendanceUpdated {
    #[must_use]
    pub fn new(
        student_attendance_id: StudentAttendanceId,
        changes: Vec<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            student_attendance_id,
            changes,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StudentAttendanceUpdated {
    const EVENT_TYPE: &'static str = "attendance.student.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "student_attendance";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.student_attendance_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.student_attendance_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// StudentAttendanceRestored
// =============================================================================

/// Emitted when a soft-deleted
/// [`StudentAttendance`](crate::aggregate::StudentAttendance)
/// is restored (the row's `active_status` flips back to
/// `Active`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StudentAttendanceRestored {
    pub student_attendance_id: StudentAttendanceId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl StudentAttendanceRestored {
    #[must_use]
    pub fn new(
        student_attendance_id: StudentAttendanceId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            student_attendance_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StudentAttendanceRestored {
    const EVENT_TYPE: &'static str = "attendance.student.restored";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "student_attendance";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.student_attendance_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.student_attendance_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// StudentAbsentForDay
// =============================================================================

/// Emitted when a student is marked absent for the day. The
/// notification fan-out (parent SMS / push / email) subscribes
/// to this event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StudentAbsentForDay {
    pub student_attendance_id: StudentAttendanceId,
    pub student_id: StudentId,
    pub student_record_id: StudentRecordId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub attendance_date: NaiveDate,
    pub notes: Option<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl StudentAbsentForDay {
    #[must_use]
    pub fn new(
        student_attendance_id: StudentAttendanceId,
        student_id: StudentId,
        student_record_id: StudentRecordId,
        class_id: ClassId,
        section_id: SectionId,
        attendance_date: NaiveDate,
        notes: Option<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            student_attendance_id,
            student_id,
            student_record_id,
            class_id,
            section_id,
            attendance_date,
            notes,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StudentAbsentForDay {
    const EVENT_TYPE: &'static str = "attendance.student.absent";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "student_attendance";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.student_attendance_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.student_attendance_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// StudentAttendanceImported
// =============================================================================

/// Emitted when a single row of a bulk import is committed
/// into the live `StudentAttendance` table. One event per
/// row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StudentAttendanceImported {
    pub student_attendance_id: StudentAttendanceId,
    pub bulk_import_id: BulkAttendanceImportId,
    pub student_id: StudentId,
    pub attendance_date: NaiveDate,
    pub attendance_type: AttendanceType,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl StudentAttendanceImported {
    #[must_use]
    pub fn new(
        student_attendance_id: StudentAttendanceId,
        bulk_import_id: BulkAttendanceImportId,
        student_id: StudentId,
        attendance_date: NaiveDate,
        attendance_type: AttendanceType,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            student_attendance_id,
            bulk_import_id,
            student_id,
            attendance_date,
            attendance_type,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StudentAttendanceImported {
    const EVENT_TYPE: &'static str = "attendance.student.imported";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "student_attendance";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.student_attendance_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.student_attendance_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// SubjectAttendanceMarked
// =============================================================================

/// Emitted when a [`SubjectAttendance`](crate::aggregate::SubjectAttendance)
/// is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubjectAttendanceMarked {
    pub subject_attendance_id: SubjectAttendanceId,
    pub student_id: StudentId,
    pub student_record_id: StudentRecordId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub subject_id: SubjectId,
    pub attendance_date: NaiveDate,
    pub attendance_type: AttendanceType,
    pub notes: Option<String>,
    pub notify: bool,
    pub marked_by: educore_core::ids::UserId,
    pub marked_at: Timestamp,
    pub marked_from: AttendanceSource,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SubjectAttendanceMarked {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        subject_attendance_id: SubjectAttendanceId,
        student_id: StudentId,
        student_record_id: StudentRecordId,
        class_id: ClassId,
        section_id: SectionId,
        subject_id: SubjectId,
        attendance_date: NaiveDate,
        attendance_type: AttendanceType,
        notes: Option<String>,
        notify: bool,
        marked_by: educore_core::ids::UserId,
        marked_at: Timestamp,
        marked_from: AttendanceSource,
        event_id: EventId,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            subject_attendance_id,
            student_id,
            student_record_id,
            class_id,
            section_id,
            subject_id,
            attendance_date,
            attendance_type,
            notes,
            notify,
            marked_by,
            marked_at,
            marked_from,
            event_id,
            correlation_id,
            occurred_at: marked_at,
        }
    }
}

impl DomainEvent for SubjectAttendanceMarked {
    const EVENT_TYPE: &'static str = "attendance.subject.marked";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "subject_attendance";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.subject_attendance_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.subject_attendance_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// SubjectAttendanceUpdated
// =============================================================================

/// Emitted when a [`SubjectAttendance`](crate::aggregate::SubjectAttendance)
/// is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubjectAttendanceUpdated {
    pub subject_attendance_id: SubjectAttendanceId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SubjectAttendanceUpdated {
    #[must_use]
    pub fn new(
        subject_attendance_id: SubjectAttendanceId,
        changes: Vec<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            subject_attendance_id,
            changes,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SubjectAttendanceUpdated {
    const EVENT_TYPE: &'static str = "attendance.subject.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "subject_attendance";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.subject_attendance_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.subject_attendance_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// SubjectAbsentNotificationRequested
// =============================================================================

/// Emitted when a subject-level absence triggers a parent
/// notification request (i.e. the attendance row's `notify`
/// flag is `true` and `attendance_type` is `Absent`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubjectAbsentNotificationRequested {
    pub subject_attendance_id: SubjectAttendanceId,
    pub student_id: StudentId,
    pub subject_id: SubjectId,
    pub attendance_date: NaiveDate,
    pub channel: String,
    pub template: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SubjectAbsentNotificationRequested {
    #[must_use]
    pub fn new(
        subject_attendance_id: SubjectAttendanceId,
        student_id: StudentId,
        subject_id: SubjectId,
        attendance_date: NaiveDate,
        channel: String,
        template: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            subject_attendance_id,
            student_id,
            subject_id,
            attendance_date,
            channel,
            template,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SubjectAbsentNotificationRequested {
    const EVENT_TYPE: &'static str = "attendance.subject.absent_notification_requested";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "subject_attendance";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.subject_attendance_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.subject_attendance_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// StaffAttendanceMarked
// =============================================================================

/// Emitted when a [`StaffAttendance`](crate::aggregate::StaffAttendance)
/// is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffAttendanceMarked {
    pub staff_attendance_id: StaffAttendanceId,
    pub staff_id: StaffId,
    pub attendance_date: NaiveDate,
    pub attendance_type: AttendanceType,
    pub notes: Option<String>,
    pub marked_by: educore_core::ids::UserId,
    pub marked_at: Timestamp,
    pub marked_from: AttendanceSource,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl StaffAttendanceMarked {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        staff_attendance_id: StaffAttendanceId,
        staff_id: StaffId,
        attendance_date: NaiveDate,
        attendance_type: AttendanceType,
        notes: Option<String>,
        marked_by: educore_core::ids::UserId,
        marked_at: Timestamp,
        marked_from: AttendanceSource,
        event_id: EventId,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            staff_attendance_id,
            staff_id,
            attendance_date,
            attendance_type,
            notes,
            marked_by,
            marked_at,
            marked_from,
            event_id,
            correlation_id,
            occurred_at: marked_at,
        }
    }
}

impl DomainEvent for StaffAttendanceMarked {
    const EVENT_TYPE: &'static str = "attendance.staff.marked";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "staff_attendance";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.staff_attendance_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.staff_attendance_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// StaffAttendanceUpdated
// =============================================================================

/// Emitted when a [`StaffAttendance`](crate::aggregate::StaffAttendance)
/// is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffAttendanceUpdated {
    pub staff_attendance_id: StaffAttendanceId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl StaffAttendanceUpdated {
    #[must_use]
    pub fn new(
        staff_attendance_id: StaffAttendanceId,
        changes: Vec<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            staff_attendance_id,
            changes,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StaffAttendanceUpdated {
    const EVENT_TYPE: &'static str = "attendance.staff.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "staff_attendance";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.staff_attendance_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.staff_attendance_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// StaffAbsentForDay
// =============================================================================

/// Emitted when a staff member is marked absent for the
/// day. The payroll deduction fan-out (Phase 7) subscribes
/// to this event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffAbsentForDay {
    pub staff_attendance_id: StaffAttendanceId,
    pub staff_id: StaffId,
    pub attendance_date: NaiveDate,
    pub notes: Option<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl StaffAbsentForDay {
    #[must_use]
    pub fn new(
        staff_attendance_id: StaffAttendanceId,
        staff_id: StaffId,
        attendance_date: NaiveDate,
        notes: Option<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            staff_attendance_id,
            staff_id,
            attendance_date,
            notes,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StaffAbsentForDay {
    const EVENT_TYPE: &'static str = "attendance.staff.absent";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "staff_attendance";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.staff_attendance_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.staff_attendance_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// ExamAttendanceMarked
// =============================================================================

/// Emitted when an [`ExamAttendance`](crate::aggregate::ExamAttendance)
/// is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExamAttendanceMarked {
    pub exam_attendance_id: ExamAttendanceId,
    pub exam_id: ExamId,
    pub student_id: StudentId,
    pub student_record_id: StudentRecordId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub subject_id: SubjectId,
    pub exam_date: NaiveDate,
    pub attendance_type: AttendanceType,
    pub notes: Option<String>,
    pub marked_by: educore_core::ids::UserId,
    pub marked_at: Timestamp,
    pub marked_from: AttendanceSource,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ExamAttendanceMarked {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        exam_attendance_id: ExamAttendanceId,
        exam_id: ExamId,
        student_id: StudentId,
        student_record_id: StudentRecordId,
        class_id: ClassId,
        section_id: SectionId,
        subject_id: SubjectId,
        exam_date: NaiveDate,
        attendance_type: AttendanceType,
        notes: Option<String>,
        marked_by: educore_core::ids::UserId,
        marked_at: Timestamp,
        marked_from: AttendanceSource,
        event_id: EventId,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            exam_attendance_id,
            exam_id,
            student_id,
            student_record_id,
            class_id,
            section_id,
            subject_id,
            exam_date,
            attendance_type,
            notes,
            marked_by,
            marked_at,
            marked_from,
            event_id,
            correlation_id,
            occurred_at: marked_at,
        }
    }
}

impl DomainEvent for ExamAttendanceMarked {
    const EVENT_TYPE: &'static str = "attendance.exam.marked";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "exam_attendance";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.exam_attendance_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.exam_attendance_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// ExamAttendanceUpdated
// =============================================================================

/// Emitted when an [`ExamAttendance`](crate::aggregate::ExamAttendance)
/// is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExamAttendanceUpdated {
    pub exam_attendance_id: ExamAttendanceId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ExamAttendanceUpdated {
    #[must_use]
    pub fn new(
        exam_attendance_id: ExamAttendanceId,
        changes: Vec<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            exam_attendance_id,
            changes,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ExamAttendanceUpdated {
    const EVENT_TYPE: &'static str = "attendance.exam.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "exam_attendance";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.exam_attendance_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.exam_attendance_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// BulkImportStarted
// =============================================================================

/// Emitted when a [`BulkAttendanceImport`](crate::aggregate::BulkAttendanceImport)
/// is created (the staging rows have been received but not
/// yet validated).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BulkImportStarted {
    pub bulk_import_id: BulkAttendanceImportId,
    pub academic_year_id: AcademicYearId,
    pub source: AttendanceSource,
    pub row_count: u32,
    pub marked_by: educore_core::ids::UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BulkImportStarted {
    #[must_use]
    pub fn new(
        bulk_import_id: BulkAttendanceImportId,
        academic_year_id: AcademicYearId,
        source: AttendanceSource,
        row_count: u32,
        marked_by: educore_core::ids::UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            bulk_import_id,
            academic_year_id,
            source,
            row_count,
            marked_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BulkImportStarted {
    const EVENT_TYPE: &'static str = "attendance.bulk_import.started";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "bulk_attendance_import";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.bulk_import_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.bulk_import_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// BulkImportValidated
// =============================================================================

/// Emitted when a [`BulkAttendanceImport`](crate::aggregate::BulkAttendanceImport)
/// passes the validator (no duplicate keys, all dates
/// well-formed, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BulkImportValidated {
    pub bulk_import_id: BulkAttendanceImportId,
    pub row_count: u32,
    pub absent_count: u32,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BulkImportValidated {
    #[must_use]
    pub fn new(
        bulk_import_id: BulkAttendanceImportId,
        row_count: u32,
        absent_count: u32,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            bulk_import_id,
            row_count,
            absent_count,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BulkImportValidated {
    const EVENT_TYPE: &'static str = "attendance.bulk_import.validated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "bulk_attendance_import";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.bulk_import_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.bulk_import_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// BulkImportCommitted
// =============================================================================

/// Emitted when a [`BulkAttendanceImport`](crate::aggregate::BulkAttendanceImport)
/// is committed (the staging rows have been promoted into
/// the live `StudentAttendance` / `StaffAttendance` tables).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BulkImportCommitted {
    pub bulk_import_id: BulkAttendanceImportId,
    pub committed_count: u32,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BulkImportCommitted {
    #[must_use]
    pub fn new(
        bulk_import_id: BulkAttendanceImportId,
        committed_count: u32,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            bulk_import_id,
            committed_count,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BulkImportCommitted {
    const EVENT_TYPE: &'static str = "attendance.bulk_import.committed";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "bulk_attendance_import";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.bulk_import_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.bulk_import_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// BulkImportFailed
// =============================================================================

/// Emitted when a [`BulkAttendanceImport`](crate::aggregate::BulkAttendanceImport)
/// fails validation (duplicate keys, malformed dates, …).
/// No rows are committed when this event fires.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BulkImportFailed {
    pub bulk_import_id: BulkAttendanceImportId,
    pub failed_count: u32,
    pub reason: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BulkImportFailed {
    #[must_use]
    pub fn new(
        bulk_import_id: BulkAttendanceImportId,
        failed_count: u32,
        reason: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            bulk_import_id,
            failed_count,
            reason,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BulkImportFailed {
    const EVENT_TYPE: &'static str = "attendance.bulk_import.failed";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "bulk_attendance_import";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.bulk_import_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.bulk_import_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// BulkImportCancelled
// =============================================================================

/// Emitted when a [`BulkAttendanceImport`](crate::aggregate::BulkAttendanceImport)
/// is cancelled by the operator before commit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BulkImportCancelled {
    pub bulk_import_id: BulkAttendanceImportId,
    pub reason: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BulkImportCancelled {
    #[must_use]
    pub fn new(
        bulk_import_id: BulkAttendanceImportId,
        reason: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            bulk_import_id,
            reason,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BulkImportCancelled {
    const EVENT_TYPE: &'static str = "attendance.bulk_import.cancelled";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "bulk_attendance_import";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.bulk_import_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.bulk_import_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// AttendanceImported (per-row, bulk path)
// =============================================================================

/// Emitted per row when a bulk import commits a single
/// `StudentAttendance` row. The `BulkImportCommitted` event
/// is the roll-up; this is the per-row fan-out for downstream
/// subscribers that need to react to every committed row
/// (e.g. the absence-notification dispatcher).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttendanceImported {
    pub bulk_import_id: BulkAttendanceImportId,
    pub student_id: StudentId,
    pub attendance_date: NaiveDate,
    pub attendance_type: AttendanceType,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl AttendanceImported {
    #[must_use]
    pub fn new(
        bulk_import_id: BulkAttendanceImportId,
        student_id: StudentId,
        attendance_date: NaiveDate,
        attendance_type: AttendanceType,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            bulk_import_id,
            student_id,
            attendance_date,
            attendance_type,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for AttendanceImported {
    const EVENT_TYPE: &'static str = "attendance.bulk_import.row_imported";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "bulk_attendance_import";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.bulk_import_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.bulk_import_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// AbsenceNotificationRequested (cross-cutting)
// =============================================================================

/// Emitted when an operator or a service requests that a
/// notification be sent to a parent about a specific
/// student-attendance row. The notification adapter (SMS /
/// push / email) subscribes to this event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AbsenceNotificationRequested {
    pub student_attendance_id: StudentAttendanceId,
    pub student_id: StudentId,
    pub attendance_date: NaiveDate,
    pub channel: String,
    pub template: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl AbsenceNotificationRequested {
    #[must_use]
    pub fn new(
        student_attendance_id: StudentAttendanceId,
        student_id: StudentId,
        attendance_date: NaiveDate,
        channel: String,
        template: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            student_attendance_id,
            student_id,
            attendance_date,
            channel,
            template,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for AbsenceNotificationRequested {
    const EVENT_TYPE: &'static str = "attendance.notification.requested";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "notification";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.student_attendance_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.student_attendance_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// ClassAttendanceRecomputed (cross-cutting)
// =============================================================================

/// Emitted when the per-(class, section, date) attendance
/// roll-up is recomputed. Drives the dashboard's "today's
/// attendance" widget and the per-section daily summary
/// reports.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassAttendanceRecomputed {
    pub class_attendance_id: ClassAttendanceId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub attendance_date: NaiveDate,
    pub total_students: u32,
    pub absent_count: u32,
    pub present_count: u32,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ClassAttendanceRecomputed {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        class_attendance_id: ClassAttendanceId,
        class_id: ClassId,
        section_id: SectionId,
        attendance_date: NaiveDate,
        total_students: u32,
        absent_count: u32,
        present_count: u32,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            class_attendance_id,
            class_id,
            section_id,
            attendance_date,
            total_students,
            absent_count,
            present_count,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ClassAttendanceRecomputed {
    const EVENT_TYPE: &'static str = "attendance.class_attendance.recomputed";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "class_attendance";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.class_attendance_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.class_attendance_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    unused_variables
)]
mod tests {
    use super::*;
    use educore_core::ids::{SchoolId, UserId};
    use educore_core::tenant::{TenantContext, UserType};

    fn ctx(school: SchoolId) -> TenantContext {
        TenantContext::for_user(
            school,
            UserId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
            UserType::SchoolAdmin,
        )
    }

    fn s() -> SchoolId {
        SchoolId(uuid::Uuid::now_v7())
    }

    fn actor() -> UserId {
        UserId(uuid::Uuid::now_v7())
    }

    fn date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2024, 9, 15).expect("valid date")
    }

    #[test]
    fn student_attendance_marked_metadata() {
        let school = s();
        let id = StudentAttendanceId::new(school, uuid::Uuid::now_v7());
        let ev = StudentAttendanceMarked::new(
            id,
            StudentId::new(school, uuid::Uuid::now_v7()),
            StudentRecordId::new(school, uuid::Uuid::now_v7()),
            ClassId::new(school, uuid::Uuid::now_v7()),
            SectionId::new(school, uuid::Uuid::now_v7()),
            date(),
            AttendanceType::Absent,
            Some("sick".to_owned()),
            actor(),
            Timestamp::now(),
            AttendanceSource::Manual,
            EventId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
        );
        assert_eq!(
            <StudentAttendanceMarked as DomainEvent>::EVENT_TYPE,
            "attendance.student.marked"
        );
        assert_eq!(
            <StudentAttendanceMarked as DomainEvent>::AGGREGATE_TYPE,
            "student_attendance"
        );
        assert_eq!(
            <StudentAttendanceMarked as DomainEvent>::aggregate_id(&ev),
            id.as_uuid()
        );
        assert_eq!(
            <StudentAttendanceMarked as DomainEvent>::school_id(&ev),
            school
        );
    }

    #[test]
    fn student_attendance_updated_metadata() {
        let school = s();
        let id = StudentAttendanceId::new(school, uuid::Uuid::now_v7());
        let ev = StudentAttendanceUpdated::new(
            id,
            vec!["notes".to_owned()],
            EventId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
            Timestamp::now(),
        );
        assert_eq!(
            <StudentAttendanceUpdated as DomainEvent>::EVENT_TYPE,
            "attendance.student.updated"
        );
        assert_eq!(
            <StudentAttendanceUpdated as DomainEvent>::aggregate_id(&ev),
            id.as_uuid()
        );
    }

    #[test]
    fn student_attendance_restored_metadata() {
        let school = s();
        let id = StudentAttendanceId::new(school, uuid::Uuid::now_v7());
        let ev = StudentAttendanceRestored::new(
            id,
            EventId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
            Timestamp::now(),
        );
        assert_eq!(
            <StudentAttendanceRestored as DomainEvent>::EVENT_TYPE,
            "attendance.student.restored"
        );
        assert_eq!(
            <StudentAttendanceRestored as DomainEvent>::aggregate_id(&ev),
            id.as_uuid()
        );
    }

    #[test]
    fn student_absent_for_day_metadata() {
        let school = s();
        let id = StudentAttendanceId::new(school, uuid::Uuid::now_v7());
        let ev = StudentAbsentForDay::new(
            id,
            StudentId::new(school, uuid::Uuid::now_v7()),
            StudentRecordId::new(school, uuid::Uuid::now_v7()),
            ClassId::new(school, uuid::Uuid::now_v7()),
            SectionId::new(school, uuid::Uuid::now_v7()),
            date(),
            Some("sick".to_owned()),
            EventId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
            Timestamp::now(),
        );
        assert_eq!(
            <StudentAbsentForDay as DomainEvent>::EVENT_TYPE,
            "attendance.student.absent"
        );
        assert_eq!(
            <StudentAbsentForDay as DomainEvent>::AGGREGATE_TYPE,
            "student_attendance"
        );
    }

    #[test]
    fn student_attendance_imported_metadata() {
        let school = s();
        let id = StudentAttendanceId::new(school, uuid::Uuid::now_v7());
        let bulk = BulkAttendanceImportId::new(school, uuid::Uuid::now_v7());
        let ev = StudentAttendanceImported::new(
            id,
            bulk,
            StudentId::new(school, uuid::Uuid::now_v7()),
            date(),
            AttendanceType::Present,
            EventId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
            Timestamp::now(),
        );
        assert_eq!(
            <StudentAttendanceImported as DomainEvent>::EVENT_TYPE,
            "attendance.student.imported"
        );
        assert_eq!(
            <StudentAttendanceImported as DomainEvent>::AGGREGATE_TYPE,
            "student_attendance"
        );
        assert_eq!(
            <StudentAttendanceImported as DomainEvent>::aggregate_id(&ev),
            id.as_uuid()
        );
    }

    #[test]
    fn subject_attendance_marked_metadata() {
        let school = s();
        let id = SubjectAttendanceId::new(school, uuid::Uuid::now_v7());
        let ev = SubjectAttendanceMarked::new(
            id,
            StudentId::new(school, uuid::Uuid::now_v7()),
            StudentRecordId::new(school, uuid::Uuid::now_v7()),
            ClassId::new(school, uuid::Uuid::now_v7()),
            SectionId::new(school, uuid::Uuid::now_v7()),
            SubjectId::new(school, uuid::Uuid::now_v7()),
            date(),
            AttendanceType::Late,
            None,
            true,
            actor(),
            Timestamp::now(),
            AttendanceSource::Biometric,
            EventId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
        );
        assert_eq!(
            <SubjectAttendanceMarked as DomainEvent>::EVENT_TYPE,
            "attendance.subject.marked"
        );
        assert_eq!(
            <SubjectAttendanceMarked as DomainEvent>::AGGREGATE_TYPE,
            "subject_attendance"
        );
        assert_eq!(
            <SubjectAttendanceMarked as DomainEvent>::aggregate_id(&ev),
            id.as_uuid()
        );
    }

    #[test]
    fn subject_attendance_updated_metadata() {
        let school = s();
        let id = SubjectAttendanceId::new(school, uuid::Uuid::now_v7());
        let ev = SubjectAttendanceUpdated::new(
            id,
            vec!["notes".to_owned()],
            EventId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
            Timestamp::now(),
        );
        assert_eq!(
            <SubjectAttendanceUpdated as DomainEvent>::EVENT_TYPE,
            "attendance.subject.updated"
        );
    }

    #[test]
    fn subject_absent_notification_requested_metadata() {
        let school = s();
        let id = SubjectAttendanceId::new(school, uuid::Uuid::now_v7());
        let ev = SubjectAbsentNotificationRequested::new(
            id,
            StudentId::new(school, uuid::Uuid::now_v7()),
            SubjectId::new(school, uuid::Uuid::now_v7()),
            date(),
            "sms".to_owned(),
            "absent_sms_v1".to_owned(),
            EventId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
            Timestamp::now(),
        );
        assert_eq!(
            <SubjectAbsentNotificationRequested as DomainEvent>::EVENT_TYPE,
            "attendance.subject.absent_notification_requested"
        );
    }

    #[test]
    fn staff_attendance_marked_metadata() {
        let school = s();
        let id = StaffAttendanceId::new(school, uuid::Uuid::now_v7());
        let ev = StaffAttendanceMarked::new(
            id,
            StaffId::new(school, uuid::Uuid::now_v7()),
            date(),
            AttendanceType::Present,
            None,
            actor(),
            Timestamp::now(),
            AttendanceSource::Manual,
            EventId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
        );
        assert_eq!(
            <StaffAttendanceMarked as DomainEvent>::EVENT_TYPE,
            "attendance.staff.marked"
        );
        assert_eq!(
            <StaffAttendanceMarked as DomainEvent>::AGGREGATE_TYPE,
            "staff_attendance"
        );
    }

    #[test]
    fn staff_attendance_updated_metadata() {
        let school = s();
        let id = StaffAttendanceId::new(school, uuid::Uuid::now_v7());
        let ev = StaffAttendanceUpdated::new(
            id,
            vec!["notes".to_owned()],
            EventId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
            Timestamp::now(),
        );
        assert_eq!(
            <StaffAttendanceUpdated as DomainEvent>::EVENT_TYPE,
            "attendance.staff.updated"
        );
    }

    #[test]
    fn staff_absent_for_day_metadata() {
        let school = s();
        let id = StaffAttendanceId::new(school, uuid::Uuid::now_v7());
        let ev = StaffAbsentForDay::new(
            id,
            StaffId::new(school, uuid::Uuid::now_v7()),
            date(),
            None,
            EventId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
            Timestamp::now(),
        );
        assert_eq!(
            <StaffAbsentForDay as DomainEvent>::EVENT_TYPE,
            "attendance.staff.absent"
        );
    }

    #[test]
    fn exam_attendance_marked_metadata() {
        let school = s();
        let id = ExamAttendanceId::new(school, uuid::Uuid::now_v7());
        let ev = ExamAttendanceMarked::new(
            id,
            ExamId::new(school, uuid::Uuid::now_v7()),
            StudentId::new(school, uuid::Uuid::now_v7()),
            StudentRecordId::new(school, uuid::Uuid::now_v7()),
            ClassId::new(school, uuid::Uuid::now_v7()),
            SectionId::new(school, uuid::Uuid::now_v7()),
            SubjectId::new(school, uuid::Uuid::now_v7()),
            date(),
            AttendanceType::Absent,
            None,
            actor(),
            Timestamp::now(),
            AttendanceSource::Manual,
            EventId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
        );
        assert_eq!(
            <ExamAttendanceMarked as DomainEvent>::EVENT_TYPE,
            "attendance.exam.marked"
        );
        assert_eq!(
            <ExamAttendanceMarked as DomainEvent>::AGGREGATE_TYPE,
            "exam_attendance"
        );
    }

    #[test]
    fn exam_attendance_updated_metadata() {
        let school = s();
        let id = ExamAttendanceId::new(school, uuid::Uuid::now_v7());
        let ev = ExamAttendanceUpdated::new(
            id,
            vec!["notes".to_owned()],
            EventId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
            Timestamp::now(),
        );
        assert_eq!(
            <ExamAttendanceUpdated as DomainEvent>::EVENT_TYPE,
            "attendance.exam.updated"
        );
    }

    #[test]
    fn bulk_import_started_metadata() {
        let school = s();
        let id = BulkAttendanceImportId::new(school, uuid::Uuid::now_v7());
        let ev = BulkImportStarted::new(
            id,
            AcademicYearId::new(school, uuid::Uuid::now_v7()),
            AttendanceSource::BulkImport,
            100,
            actor(),
            EventId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
            Timestamp::now(),
        );
        assert_eq!(
            <BulkImportStarted as DomainEvent>::EVENT_TYPE,
            "attendance.bulk_import.started"
        );
        assert_eq!(
            <BulkImportStarted as DomainEvent>::AGGREGATE_TYPE,
            "bulk_attendance_import"
        );
    }

    #[test]
    fn bulk_import_validated_metadata() {
        let school = s();
        let id = BulkAttendanceImportId::new(school, uuid::Uuid::now_v7());
        let ev = BulkImportValidated::new(
            id,
            100,
            5,
            EventId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
            Timestamp::now(),
        );
        assert_eq!(
            <BulkImportValidated as DomainEvent>::EVENT_TYPE,
            "attendance.bulk_import.validated"
        );
    }

    #[test]
    fn bulk_import_committed_metadata() {
        let school = s();
        let id = BulkAttendanceImportId::new(school, uuid::Uuid::now_v7());
        let ev = BulkImportCommitted::new(
            id,
            100,
            EventId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
            Timestamp::now(),
        );
        assert_eq!(
            <BulkImportCommitted as DomainEvent>::EVENT_TYPE,
            "attendance.bulk_import.committed"
        );
    }

    #[test]
    fn bulk_import_failed_metadata() {
        let school = s();
        let id = BulkAttendanceImportId::new(school, uuid::Uuid::now_v7());
        let ev = BulkImportFailed::new(
            id,
            3,
            "duplicate keys".to_owned(),
            EventId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
            Timestamp::now(),
        );
        assert_eq!(
            <BulkImportFailed as DomainEvent>::EVENT_TYPE,
            "attendance.bulk_import.failed"
        );
    }

    #[test]
    fn bulk_import_cancelled_metadata() {
        let school = s();
        let id = BulkAttendanceImportId::new(school, uuid::Uuid::now_v7());
        let ev = BulkImportCancelled::new(
            id,
            "operator cancelled".to_owned(),
            EventId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
            Timestamp::now(),
        );
        assert_eq!(
            <BulkImportCancelled as DomainEvent>::EVENT_TYPE,
            "attendance.bulk_import.cancelled"
        );
    }

    #[test]
    fn attendance_imported_metadata() {
        let school = s();
        let bulk = BulkAttendanceImportId::new(school, uuid::Uuid::now_v7());
        let ev = AttendanceImported::new(
            bulk,
            StudentId::new(school, uuid::Uuid::now_v7()),
            date(),
            AttendanceType::Present,
            EventId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
            Timestamp::now(),
        );
        assert_eq!(
            <AttendanceImported as DomainEvent>::EVENT_TYPE,
            "attendance.bulk_import.row_imported"
        );
        assert_eq!(
            <AttendanceImported as DomainEvent>::AGGREGATE_TYPE,
            "bulk_attendance_import"
        );
        assert_eq!(
            <AttendanceImported as DomainEvent>::aggregate_id(&ev),
            bulk.as_uuid()
        );
    }

    #[test]
    fn absence_notification_requested_metadata() {
        let school = s();
        let id = StudentAttendanceId::new(school, uuid::Uuid::now_v7());
        let ev = AbsenceNotificationRequested::new(
            id,
            StudentId::new(school, uuid::Uuid::now_v7()),
            date(),
            "sms".to_owned(),
            "absent_v1".to_owned(),
            EventId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
            Timestamp::now(),
        );
        assert_eq!(
            <AbsenceNotificationRequested as DomainEvent>::EVENT_TYPE,
            "attendance.notification.requested"
        );
        assert_eq!(
            <AbsenceNotificationRequested as DomainEvent>::AGGREGATE_TYPE,
            "notification"
        );
    }

    #[test]
    fn class_attendance_recomputed_metadata() {
        let school = s();
        let id = ClassAttendanceId::new(school, uuid::Uuid::now_v7());
        let ev = ClassAttendanceRecomputed::new(
            id,
            ClassId::new(school, uuid::Uuid::now_v7()),
            SectionId::new(school, uuid::Uuid::now_v7()),
            date(),
            40,
            4,
            36,
            EventId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
            Timestamp::now(),
        );
        assert_eq!(
            <ClassAttendanceRecomputed as DomainEvent>::EVENT_TYPE,
            "attendance.class_attendance.recomputed"
        );
        assert_eq!(
            <ClassAttendanceRecomputed as DomainEvent>::AGGREGATE_TYPE,
            "class_attendance"
        );
        assert_eq!(
            <ClassAttendanceRecomputed as DomainEvent>::aggregate_id(&ev),
            id.as_uuid()
        );
    }

    #[test]
    fn student_attendance_marked_envelope_round_trip() {
        let school = s();
        let id = StudentAttendanceId::new(school, uuid::Uuid::now_v7());
        let ev = StudentAttendanceMarked::new(
            id,
            StudentId::new(school, uuid::Uuid::now_v7()),
            StudentRecordId::new(school, uuid::Uuid::now_v7()),
            ClassId::new(school, uuid::Uuid::now_v7()),
            SectionId::new(school, uuid::Uuid::now_v7()),
            date(),
            AttendanceType::Present,
            None,
            actor(),
            Timestamp::now(),
            AttendanceSource::Manual,
            EventId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
        );
        let envelope = ev.into_envelope(&ctx(school));
        assert_eq!(envelope.event_type, "attendance.student.marked");
        assert_eq!(envelope.aggregate_type, "student_attendance");
        assert_eq!(envelope.school_id, school);
        assert_eq!(envelope.aggregate_id, id.as_uuid());
    }

    #[test]
    fn staff_attendance_marked_envelope_round_trip() {
        let school = s();
        let id = StaffAttendanceId::new(school, uuid::Uuid::now_v7());
        let ev = StaffAttendanceMarked::new(
            id,
            StaffId::new(school, uuid::Uuid::now_v7()),
            date(),
            AttendanceType::Present,
            None,
            actor(),
            Timestamp::now(),
            AttendanceSource::Manual,
            EventId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
        );
        let envelope = ev.into_envelope(&ctx(school));
        assert_eq!(envelope.event_type, "attendance.staff.marked");
        assert_eq!(envelope.aggregate_type, "staff_attendance");
    }

    #[test]
    fn bulk_import_committed_envelope_round_trip() {
        let school = s();
        let id = BulkAttendanceImportId::new(school, uuid::Uuid::now_v7());
        let ev = BulkImportCommitted::new(
            id,
            50,
            EventId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
            Timestamp::now(),
        );
        let envelope = ev.into_envelope(&ctx(school));
        assert_eq!(envelope.event_type, "attendance.bulk_import.committed");
        assert_eq!(envelope.aggregate_type, "bulk_attendance_import");
    }

    #[test]
    fn exam_attendance_marked_envelope_round_trip() {
        let school = s();
        let id = ExamAttendanceId::new(school, uuid::Uuid::now_v7());
        let ev = ExamAttendanceMarked::new(
            id,
            ExamId::new(school, uuid::Uuid::now_v7()),
            StudentId::new(school, uuid::Uuid::now_v7()),
            StudentRecordId::new(school, uuid::Uuid::now_v7()),
            ClassId::new(school, uuid::Uuid::now_v7()),
            SectionId::new(school, uuid::Uuid::now_v7()),
            SubjectId::new(school, uuid::Uuid::now_v7()),
            date(),
            AttendanceType::Present,
            None,
            actor(),
            Timestamp::now(),
            AttendanceSource::Manual,
            EventId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
        );
        let envelope = ev.into_envelope(&ctx(school));
        assert_eq!(envelope.event_type, "attendance.exam.marked");
        assert_eq!(envelope.aggregate_type, "exam_attendance");
    }
}
