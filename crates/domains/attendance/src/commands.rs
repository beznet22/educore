//! # Attendance command shapes
//!
//! Phase 5 Workstream A ships 14 command shapes:
//!
//! - **Student (3):** [`MarkStudentAttendanceCommand`],
//!   [`UpdateStudentAttendanceCommand`],
//!   [`BulkMarkStudentAttendanceCommand`].
//! - **Subject (2):** [`MarkSubjectAttendanceCommand`],
//!   [`UpdateSubjectAttendanceCommand`].
//! - **Staff (2):** [`MarkStaffAttendanceCommand`],
//!   [`UpdateStaffAttendanceCommand`].
//! - **Exam (2):** [`MarkExamAttendanceCommand`],
//!   [`UpdateExamAttendanceCommand`].
//! - **BulkImport (4):** [`ImportAttendanceCommand`],
//!   [`ValidateBulkImportCommand`],
//!   [`CommitBulkImportCommand`],
//!   [`CancelBulkImportCommand`].
//! - **Notification (1):** [`RequestAbsenceNotificationCommand`].
//!
//! Plus the [`AttendanceUniquenessChecker`] port (the
//! per-day per-student uniqueness check the
//! `mark_student_attendance` service calls) and the
//! [`validate_*`] helpers the services call before mutating
//! the aggregate.
//!
//! Plus the 14 `*_COMMAND_TYPE` constants for the
//! idempotency sub-port.

#![allow(missing_docs)]

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use educore_assessment::ExamId;
use educore_core::ids::SchoolId;
use educore_core::tenant::TenantContext;
use educore_core::value_objects::Timestamp;

use crate::value_objects::{
    AcademicYearId, AttendanceSource, AttendanceType, BulkAttendanceImportId, ClassId,
    ExamAttendanceId, SectionId, StaffAttendanceId, StaffId, StudentAttendanceId, StudentId,
    StudentRecordId, SubjectAttendanceId, SubjectId,
};
use educore_rbac::value_objects::Capability;

// =============================================================================
// Module-level constants (command_type strings for the
// idempotency sub-port).
// =============================================================================

/// The canonical command_type for the
/// `MarkStudentAttendanceCommand`.
pub const ATTENDANCE_STUDENT_MARK_COMMAND_TYPE: &str = "attendance.student.mark";

/// The canonical command_type for the
/// `UpdateStudentAttendanceCommand`.
pub const ATTENDANCE_STUDENT_UPDATE_COMMAND_TYPE: &str = "attendance.student.update";

/// The canonical command_type for the
/// `MarkSubjectAttendanceCommand`.
pub const ATTENDANCE_SUBJECT_MARK_COMMAND_TYPE: &str = "attendance.subject.mark";

/// The canonical command_type for the
/// `UpdateSubjectAttendanceCommand`.
pub const ATTENDANCE_SUBJECT_UPDATE_COMMAND_TYPE: &str = "attendance.subject.update";

/// The canonical command_type for the
/// `MarkStaffAttendanceCommand`.
pub const ATTENDANCE_STAFF_MARK_COMMAND_TYPE: &str = "attendance.staff.mark";

/// The canonical command_type for the
/// `UpdateStaffAttendanceCommand`.
pub const ATTENDANCE_STAFF_UPDATE_COMMAND_TYPE: &str = "attendance.staff.update";

/// The canonical command_type for the
/// `MarkExamAttendanceCommand`.
pub const ATTENDANCE_EXAM_MARK_COMMAND_TYPE: &str = "attendance.exam.mark";

/// The canonical command_type for the
/// `UpdateExamAttendanceCommand`.
pub const ATTENDANCE_EXAM_UPDATE_COMMAND_TYPE: &str = "attendance.exam.update";

/// The canonical command_type for the
/// `BulkMarkStudentAttendanceCommand`.
pub const ATTENDANCE_BULK_MARK_COMMAND_TYPE: &str = "attendance.bulk_mark";

/// The canonical command_type for the
/// `ImportAttendanceCommand`.
pub const ATTENDANCE_BULK_IMPORT_CREATE_COMMAND_TYPE: &str = "attendance.bulk_import.create";

/// The canonical command_type for the
/// `ValidateBulkImportCommand`.
pub const ATTENDANCE_BULK_IMPORT_VALIDATE_COMMAND_TYPE: &str = "attendance.bulk_import.validate";

/// The canonical command_type for the
/// `CommitBulkImportCommand`.
pub const ATTENDANCE_BULK_IMPORT_COMMIT_COMMAND_TYPE: &str = "attendance.bulk_import.commit";

/// The canonical command_type for the
/// `CancelBulkImportCommand`.
pub const ATTENDANCE_BULK_IMPORT_CANCEL_COMMAND_TYPE: &str = "attendance.bulk_import.cancel";

/// The canonical command_type for the
/// `RequestAbsenceNotificationCommand`.
pub const ATTENDANCE_NOTIFY_COMMAND_TYPE: &str = "attendance.notify";

// =============================================================================
// MarkStudentAttendanceCommand
// =============================================================================

/// The `mark_student_attendance` command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkStudentAttendanceCommand {
    pub tenant: TenantContext,
    pub student_id: StudentId,
    pub student_record_id: StudentRecordId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub attendance_date: NaiveDate,
    pub attendance_type: AttendanceType,
    pub notes: Option<String>,
    pub notify: bool,
    pub marked_from: AttendanceSource,
}

impl MarkStudentAttendanceCommand {
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AttendanceStudentCreate]
    }
}

// =============================================================================
// UpdateStudentAttendanceCommand
// =============================================================================

/// The `update_student_attendance` command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateStudentAttendanceCommand {
    pub tenant: TenantContext,
    pub student_attendance_id: StudentAttendanceId,
    pub attendance_type: Option<AttendanceType>,
    pub notes: Option<String>,
    pub notify: Option<bool>,
}

impl UpdateStudentAttendanceCommand {
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AttendanceStudentUpdate]
    }
}

// =============================================================================
// BulkMarkStudentAttendanceCommand
// =============================================================================

/// The `bulk_mark_student_attendance` command. Carries the
/// default `AttendanceType` for the section plus the
/// per-student overrides (absent / late / half-day).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BulkMarkStudentAttendanceCommand {
    pub tenant: TenantContext,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub attendance_date: NaiveDate,
    pub default_type: AttendanceType,
    pub absent_ids: Vec<StudentId>,
    pub late_ids: Vec<StudentId>,
    pub half_day_ids: Vec<StudentId>,
    pub notes: Option<String>,
}

impl BulkMarkStudentAttendanceCommand {
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AttendanceBulkMark]
    }
}

// =============================================================================
// MarkSubjectAttendanceCommand
// =============================================================================

/// The `mark_subject_attendance` command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkSubjectAttendanceCommand {
    pub tenant: TenantContext,
    pub student_id: StudentId,
    pub student_record_id: StudentRecordId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub subject_id: SubjectId,
    pub attendance_date: NaiveDate,
    pub attendance_type: AttendanceType,
    pub notes: Option<String>,
    pub notify: bool,
    pub marked_from: AttendanceSource,
}

impl MarkSubjectAttendanceCommand {
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AttendanceSubjectCreate]
    }
}

// =============================================================================
// UpdateSubjectAttendanceCommand
// =============================================================================

/// The `update_subject_attendance` command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateSubjectAttendanceCommand {
    pub tenant: TenantContext,
    pub subject_attendance_id: SubjectAttendanceId,
    pub attendance_type: Option<AttendanceType>,
    pub notes: Option<String>,
    pub notify: Option<bool>,
}

impl UpdateSubjectAttendanceCommand {
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AttendanceSubjectUpdate]
    }
}

// =============================================================================
// MarkStaffAttendanceCommand
// =============================================================================

/// The `mark_staff_attendance` command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkStaffAttendanceCommand {
    pub tenant: TenantContext,
    pub staff_id: StaffId,
    pub attendance_date: NaiveDate,
    pub attendance_type: AttendanceType,
    pub notes: Option<String>,
    pub marked_from: AttendanceSource,
}

impl MarkStaffAttendanceCommand {
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AttendanceStaffCreate]
    }
}

// =============================================================================
// UpdateStaffAttendanceCommand
// =============================================================================

/// The `update_staff_attendance` command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateStaffAttendanceCommand {
    pub tenant: TenantContext,
    pub staff_attendance_id: StaffAttendanceId,
    pub attendance_type: Option<AttendanceType>,
    pub notes: Option<String>,
}

impl UpdateStaffAttendanceCommand {
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AttendanceStaffUpdate]
    }
}

// =============================================================================
// MarkExamAttendanceCommand
// =============================================================================

/// The `mark_exam_attendance` command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkExamAttendanceCommand {
    pub tenant: TenantContext,
    pub exam_id: ExamId,
    pub student_id: StudentId,
    pub student_record_id: StudentRecordId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub subject_id: SubjectId,
    pub exam_date: NaiveDate,
    pub attendance_type: AttendanceType,
    pub notes: Option<String>,
    pub marked_from: AttendanceSource,
}

impl MarkExamAttendanceCommand {
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AttendanceExamCreate]
    }
}

// =============================================================================
// UpdateExamAttendanceCommand
// =============================================================================

/// The `update_exam_attendance` command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateExamAttendanceCommand {
    pub tenant: TenantContext,
    pub exam_attendance_id: ExamAttendanceId,
    pub attendance_type: Option<AttendanceType>,
    pub notes: Option<String>,
}

impl UpdateExamAttendanceCommand {
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AttendanceExamUpdate]
    }
}

// =============================================================================
// ImportAttendanceCommand
// =============================================================================

/// A single row in a bulk import. Maps to a
/// [`StudentAttendanceImport`](crate::entities::StudentAttendanceImport)
/// staging row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportRow {
    pub student_id: StudentId,
    pub attendance_date: NaiveDate,
    pub attendance_type: AttendanceType,
    pub in_time: Option<String>,
    pub out_time: Option<String>,
    pub notes: Option<String>,
}

/// The `import_attendance` command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportAttendanceCommand {
    pub tenant: TenantContext,
    pub source: AttendanceSource,
    pub academic_year_id: AcademicYearId,
    pub rows: Vec<ImportRow>,
}

impl ImportAttendanceCommand {
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AttendanceImportCreate]
    }
}

// =============================================================================
// ValidateBulkImportCommand
// =============================================================================

/// The `validate_bulk_import` command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidateBulkImportCommand {
    pub tenant: TenantContext,
    pub bulk_import_id: BulkAttendanceImportId,
}

impl ValidateBulkImportCommand {
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AttendanceImportUpdate]
    }
}

// =============================================================================
// CommitBulkImportCommand
// =============================================================================

/// The `commit_bulk_import` command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommitBulkImportCommand {
    pub tenant: TenantContext,
    pub bulk_import_id: BulkAttendanceImportId,
    pub committed_at: Timestamp,
}

impl CommitBulkImportCommand {
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AttendanceImportUpdate]
    }
}

// =============================================================================
// CancelBulkImportCommand
// =============================================================================

/// The `cancel_bulk_import` command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CancelBulkImportCommand {
    pub tenant: TenantContext,
    pub bulk_import_id: BulkAttendanceImportId,
    pub reason: String,
}

impl CancelBulkImportCommand {
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AttendanceImportDelete]
    }
}

// =============================================================================
// RequestAbsenceNotificationCommand
// =============================================================================

/// The `request_absence_notification` command. Triggers a
/// parent-notification fan-out for a specific
/// `StudentAttendance` row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequestAbsenceNotificationCommand {
    pub tenant: TenantContext,
    pub student_attendance_id: StudentAttendanceId,
    pub channel: String,
    pub template: String,
}

impl RequestAbsenceNotificationCommand {
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AttendanceNotify]
    }
}

// =============================================================================
// AttendanceUniquenessChecker (port)
// =============================================================================

/// The per-(school, day) uniqueness check the
/// `mark_student_attendance` and `mark_subject_attendance`
/// services call. The contract is per-school:
///
/// - `(school_id, student_id, attendance_date)` must be
///   unique for `StudentAttendance`.
/// - `(school_id, student_id, subject_id, attendance_date)`
///   must be unique for `SubjectAttendance`.
/// - `(school_id, staff_id, attendance_date)` must be unique
///   for `StaffAttendance`.
/// - `(school_id, source, attendance_date)` is the dedup key
///   for bulk imports (one import per source per day).
///
/// Production wiring: a thin adapter over the storage port.
/// Test wiring: an in-memory `Mutex<HashSet<_>>`.
pub trait AttendanceUniquenessChecker: Send + Sync {
    /// Returns `true` if a `StudentAttendance` row for the
    /// given `(school, student, date)` already exists.
    fn student_day_exists(&self, school: SchoolId, student: StudentId, date: NaiveDate) -> bool;

    /// Returns `true` if a `SubjectAttendance` row for the
    /// given `(school, student, subject, date)` already
    /// exists.
    fn subject_day_exists(
        &self,
        school: SchoolId,
        student: StudentId,
        subject: SubjectId,
        date: NaiveDate,
    ) -> bool;

    /// Returns `true` if a `StaffAttendance` row for the
    /// given `(school, staff, date)` already exists.
    fn staff_day_exists(&self, school: SchoolId, staff: StaffId, date: NaiveDate) -> bool;

    /// Returns `true` if a bulk-import batch for the given
    /// `(school, source, date)` already exists.
    fn import_source_date_exists(
        &self,
        school: SchoolId,
        source: AttendanceSource,
        date: NaiveDate,
    ) -> bool;
}

/// Alias retained for the prelude re-export (matches the
/// academic crate's `UniquenessChecker` shape).
pub type UniquenessChecker = dyn AttendanceUniquenessChecker;

// =============================================================================
// Validators (called by the service before mutation)
// =============================================================================

// The validators live in the value_objects module (validate_notes,
// validate_source) so they're discoverable from the value-object
// surface. They're re-imported here for the prelude's convenience.

pub use crate::value_objects::{validate_notes, validate_source};

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;
    use educore_core::clock::{IdGenerator, SystemIdGen};
    use educore_core::ids::{CorrelationId, UserId};
    use educore_core::tenant::UserType;

    #[test]
    fn command_type_strings_are_stable() {
        assert_eq!(
            ATTENDANCE_STUDENT_MARK_COMMAND_TYPE,
            "attendance.student.mark"
        );
        assert_eq!(
            ATTENDANCE_STUDENT_UPDATE_COMMAND_TYPE,
            "attendance.student.update"
        );
        assert_eq!(
            ATTENDANCE_SUBJECT_MARK_COMMAND_TYPE,
            "attendance.subject.mark"
        );
        assert_eq!(
            ATTENDANCE_SUBJECT_UPDATE_COMMAND_TYPE,
            "attendance.subject.update"
        );
        assert_eq!(ATTENDANCE_STAFF_MARK_COMMAND_TYPE, "attendance.staff.mark");
        assert_eq!(
            ATTENDANCE_STAFF_UPDATE_COMMAND_TYPE,
            "attendance.staff.update"
        );
        assert_eq!(ATTENDANCE_EXAM_MARK_COMMAND_TYPE, "attendance.exam.mark");
        assert_eq!(
            ATTENDANCE_EXAM_UPDATE_COMMAND_TYPE,
            "attendance.exam.update"
        );
        assert_eq!(ATTENDANCE_BULK_MARK_COMMAND_TYPE, "attendance.bulk_mark");
        assert_eq!(
            ATTENDANCE_BULK_IMPORT_CREATE_COMMAND_TYPE,
            "attendance.bulk_import.create"
        );
        assert_eq!(
            ATTENDANCE_BULK_IMPORT_VALIDATE_COMMAND_TYPE,
            "attendance.bulk_import.validate"
        );
        assert_eq!(
            ATTENDANCE_BULK_IMPORT_COMMIT_COMMAND_TYPE,
            "attendance.bulk_import.commit"
        );
        assert_eq!(
            ATTENDANCE_BULK_IMPORT_CANCEL_COMMAND_TYPE,
            "attendance.bulk_import.cancel"
        );
        assert_eq!(ATTENDANCE_NOTIFY_COMMAND_TYPE, "attendance.notify");
    }

    fn make_tenant(school: SchoolId) -> TenantContext {
        TenantContext::for_user(
            school,
            UserId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
            UserType::SchoolAdmin,
        )
    }

    #[test]
    fn mark_student_attendance_school_id_matches_tenant() {
        let g = SystemIdGen;
        let s = g.next_school_id();
        let cmd = MarkStudentAttendanceCommand {
            tenant: make_tenant(s),
            student_id: StudentId::new(s, g.next_uuid()),
            student_record_id: StudentRecordId::new(s, g.next_uuid()),
            class_id: ClassId::new(s, g.next_uuid()),
            section_id: SectionId::new(s, g.next_uuid()),
            attendance_date: NaiveDate::from_ymd_opt(2024, 9, 15).unwrap(),
            attendance_type: AttendanceType::Present,
            notes: None,
            notify: false,
            marked_from: AttendanceSource::Manual,
        };
        assert_eq!(cmd.school_id(), s);
    }

    #[test]
    fn update_student_attendance_carries_partial_fields() {
        let g = SystemIdGen;
        let s = g.next_school_id();
        let cmd = UpdateStudentAttendanceCommand {
            tenant: make_tenant(s),
            student_attendance_id: StudentAttendanceId::new(s, g.next_uuid()),
            attendance_type: Some(AttendanceType::Late),
            notes: Some("traffic".to_owned()),
            notify: Some(true),
        };
        assert_eq!(cmd.school_id(), s);
        assert_eq!(cmd.attendance_type, Some(AttendanceType::Late));
    }

    #[test]
    fn bulk_mark_student_attendance_carries_lists() {
        let g = SystemIdGen;
        let s = g.next_school_id();
        let cmd = BulkMarkStudentAttendanceCommand {
            tenant: make_tenant(s),
            class_id: ClassId::new(s, g.next_uuid()),
            section_id: SectionId::new(s, g.next_uuid()),
            attendance_date: NaiveDate::from_ymd_opt(2024, 9, 15).unwrap(),
            default_type: AttendanceType::Present,
            absent_ids: vec![StudentId::new(s, g.next_uuid())],
            late_ids: vec![StudentId::new(s, g.next_uuid())],
            half_day_ids: vec![],
            notes: None,
        };
        assert_eq!(cmd.school_id(), s);
        assert_eq!(cmd.absent_ids.len(), 1);
        assert_eq!(cmd.late_ids.len(), 1);
        assert!(cmd.half_day_ids.is_empty());
    }

    #[test]
    fn mark_subject_attendance_school_id_matches_tenant() {
        let g = SystemIdGen;
        let s = g.next_school_id();
        let cmd = MarkSubjectAttendanceCommand {
            tenant: make_tenant(s),
            student_id: StudentId::new(s, g.next_uuid()),
            student_record_id: StudentRecordId::new(s, g.next_uuid()),
            class_id: ClassId::new(s, g.next_uuid()),
            section_id: SectionId::new(s, g.next_uuid()),
            subject_id: SubjectId::new(s, g.next_uuid()),
            attendance_date: NaiveDate::from_ymd_opt(2024, 9, 15).unwrap(),
            attendance_type: AttendanceType::Late,
            notes: None,
            notify: false,
            marked_from: AttendanceSource::Manual,
        };
        assert_eq!(cmd.school_id(), s);
    }

    #[test]
    fn mark_staff_attendance_school_id_matches_tenant() {
        let g = SystemIdGen;
        let s = g.next_school_id();
        let cmd = MarkStaffAttendanceCommand {
            tenant: make_tenant(s),
            staff_id: StaffId::new(s, g.next_uuid()),
            attendance_date: NaiveDate::from_ymd_opt(2024, 9, 15).unwrap(),
            attendance_type: AttendanceType::Present,
            notes: None,
            marked_from: AttendanceSource::Manual,
        };
        assert_eq!(cmd.school_id(), s);
    }

    #[test]
    fn mark_exam_attendance_school_id_matches_tenant() {
        let g = SystemIdGen;
        let s = g.next_school_id();
        let cmd = MarkExamAttendanceCommand {
            tenant: make_tenant(s),
            exam_id: ExamId::new(s, g.next_uuid()),
            student_id: StudentId::new(s, g.next_uuid()),
            student_record_id: StudentRecordId::new(s, g.next_uuid()),
            class_id: ClassId::new(s, g.next_uuid()),
            section_id: SectionId::new(s, g.next_uuid()),
            subject_id: SubjectId::new(s, g.next_uuid()),
            exam_date: NaiveDate::from_ymd_opt(2024, 9, 15).unwrap(),
            attendance_type: AttendanceType::Absent,
            notes: None,
            marked_from: AttendanceSource::Manual,
        };
        assert_eq!(cmd.school_id(), s);
    }

    #[test]
    fn import_attendance_carrows() {
        let g = SystemIdGen;
        let s = g.next_school_id();
        let cmd = ImportAttendanceCommand {
            tenant: make_tenant(s),
            source: AttendanceSource::BulkImport,
            academic_year_id: AcademicYearId::new(s, g.next_uuid()),
            rows: vec![ImportRow {
                student_id: StudentId::new(s, g.next_uuid()),
                attendance_date: NaiveDate::from_ymd_opt(2024, 9, 15).unwrap(),
                attendance_type: AttendanceType::Present,
                in_time: Some("08:30:00".to_owned()),
                out_time: Some("15:30:00".to_owned()),
                notes: None,
            }],
        };
        assert_eq!(cmd.school_id(), s);
        assert_eq!(cmd.rows.len(), 1);
        assert_eq!(cmd.rows[0].in_time.as_deref(), Some("08:30:00"));
    }

    #[test]
    fn validate_bulk_import_carries_id() {
        let g = SystemIdGen;
        let s = g.next_school_id();
        let cmd = ValidateBulkImportCommand {
            tenant: make_tenant(s),
            bulk_import_id: BulkAttendanceImportId::new(s, g.next_uuid()),
        };
        assert_eq!(cmd.school_id(), s);
    }

    #[test]
    fn commit_bulk_import_carries_committed_at() {
        let g = SystemIdGen;
        let s = g.next_school_id();
        let cmd = CommitBulkImportCommand {
            tenant: make_tenant(s),
            bulk_import_id: BulkAttendanceImportId::new(s, g.next_uuid()),
            committed_at: Timestamp::now(),
        };
        assert_eq!(cmd.school_id(), s);
    }

    #[test]
    fn cancel_bulk_import_carries_reason() {
        let g = SystemIdGen;
        let s = g.next_school_id();
        let cmd = CancelBulkImportCommand {
            tenant: make_tenant(s),
            bulk_import_id: BulkAttendanceImportId::new(s, g.next_uuid()),
            reason: "duplicate batch".to_owned(),
        };
        assert_eq!(cmd.school_id(), s);
        assert_eq!(cmd.reason, "duplicate batch");
    }

    #[test]
    fn request_absence_notification_carries_channel() {
        let g = SystemIdGen;
        let s = g.next_school_id();
        let cmd = RequestAbsenceNotificationCommand {
            tenant: make_tenant(s),
            student_attendance_id: StudentAttendanceId::new(s, g.next_uuid()),
            channel: "sms".to_owned(),
            template: "absent_v1".to_owned(),
        };
        assert_eq!(cmd.school_id(), s);
        assert_eq!(cmd.channel, "sms");
    }

    #[test]
    fn validate_notes_accepts_short() {
        validate_notes("late arrival").expect("ok");
    }

    #[test]
    fn validate_notes_rejects_too_long() {
        let s: String = "x".repeat(501);
        let err = validate_notes(&s).unwrap_err();
        assert!(matches!(
            err,
            educore_core::error::DomainError::Validation(_)
        ));
    }

    #[test]
    fn validate_source_accepts_short() {
        validate_source("biometric").expect("ok");
    }

    #[test]
    fn validate_source_rejects_too_long() {
        let s: String = "x".repeat(101);
        let err = validate_source(&s).unwrap_err();
        assert!(matches!(
            err,
            educore_core::error::DomainError::Validation(_)
        ));
    }
}
