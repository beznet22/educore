//! # educore-attendance
//!
//! Student, subject, staff, and exam attendance, bulk import, absence detection.
//!
//! This crate is a member of the Educore workspace.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
// The `prelude` module's re-exports are described by their
// paths; the `PACKAGE_NAME` / `PACKAGE_VERSION` constants
// are crate-metadata helpers. Suppressing this lint at the
// crate level matches the academic and assessment crate
// patterns; per-item docs live on the originals.
#![allow(missing_docs)]

pub const PACKAGE_NAME: &str = "educore-attendance";
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod commands;
pub mod errors;
pub mod events;
pub mod services;
pub mod value_objects;

mod aggregate;
mod entities;
mod query;
mod repository;

// Re-exports
pub use crate::value_objects::{
    AcademicYearId, AttendanceBulkId, AttendanceSource, AttendanceStatus, AttendanceType,
    BulkAttendanceImportId, ClassAttendanceId, ClassId, ExamAttendanceId, ExamId, ImportStatus,
    SectionId, StaffAttendanceId, StaffAttendanceImportId, StaffId, StudentAttendanceId,
    StudentAttendanceImportId, StudentId, StudentRecordId, SubjectAttendanceId, SubjectId,
};
pub use educore_core::ids::SchoolId;
pub mod prelude {
    pub use educore_core::clock::{Clock, IdGenerator, SystemClock, SystemIdGen};
    pub use educore_core::error::{DomainError, Result};
    pub use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
    pub use educore_core::tenant::{TenantContext, UserType};
    pub use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};
    pub use educore_events::domain_event::DomainEvent;
    pub use educore_events::envelope::EventEnvelope;
    pub use educore_rbac::value_objects::Capability;

    pub use crate::aggregate::{
        BulkAttendanceImport, ExamAttendance, StaffAttendance, StudentAttendance, SubjectAttendance,
    };
    pub use crate::commands::{
        AttendanceUniquenessChecker, BulkMarkStudentAttendanceCommand, CancelBulkImportCommand,
        CommitBulkImportCommand, ImportAttendanceCommand, ImportRow, MarkExamAttendanceCommand,
        MarkStaffAttendanceCommand, MarkStudentAttendanceCommand, MarkSubjectAttendanceCommand,
        RequestAbsenceNotificationCommand, UniquenessChecker, UpdateExamAttendanceCommand,
        UpdateStaffAttendanceCommand, UpdateStudentAttendanceCommand,
        UpdateSubjectAttendanceCommand, ValidateBulkImportCommand,
        ATTENDANCE_BULK_IMPORT_CANCEL_COMMAND_TYPE, ATTENDANCE_BULK_IMPORT_COMMIT_COMMAND_TYPE,
        ATTENDANCE_BULK_IMPORT_CREATE_COMMAND_TYPE, ATTENDANCE_BULK_IMPORT_VALIDATE_COMMAND_TYPE,
        ATTENDANCE_BULK_MARK_COMMAND_TYPE, ATTENDANCE_EXAM_MARK_COMMAND_TYPE,
        ATTENDANCE_EXAM_UPDATE_COMMAND_TYPE, ATTENDANCE_NOTIFY_COMMAND_TYPE,
        ATTENDANCE_STAFF_MARK_COMMAND_TYPE, ATTENDANCE_STAFF_UPDATE_COMMAND_TYPE,
        ATTENDANCE_STUDENT_MARK_COMMAND_TYPE, ATTENDANCE_STUDENT_UPDATE_COMMAND_TYPE,
        ATTENDANCE_SUBJECT_MARK_COMMAND_TYPE, ATTENDANCE_SUBJECT_UPDATE_COMMAND_TYPE,
    };
    pub use crate::entities::{StaffAttendanceImport, StudentAttendanceImport};
    pub use crate::errors::AttendanceError;
    pub use crate::events::{
        AbsenceNotificationRequested, AttendanceImported, BulkImportCancelled, BulkImportCommitted,
        BulkImportFailed, BulkImportStarted, BulkImportValidated, ClassAttendanceRecomputed,
        ExamAttendanceMarked, ExamAttendanceUpdated, StaffAbsentForDay, StaffAttendanceMarked,
        StaffAttendanceUpdated, StudentAbsentForDay, StudentAttendanceImported,
        StudentAttendanceMarked, StudentAttendanceRestored, StudentAttendanceUpdated,
        SubjectAbsentNotificationRequested, SubjectAttendanceMarked, SubjectAttendanceUpdated,
    };
    pub use crate::query::{
        BulkAttendanceImportQuery, ExamAttendanceQuery, StaffAttendanceQuery,
        StudentAttendanceQuery, SubjectAttendanceQuery,
    };
    pub use crate::repository::{
        AttendanceImportRepository, ExamAttendanceRepository, StaffAttendanceRepository,
        StudentAttendanceRepository, SubjectAttendanceRepository,
    };
    pub use crate::services::{
        bulk_mark_student_attendance, cancel_bulk_import, commit_bulk_import, import_attendance,
        mark_exam_attendance, mark_staff_attendance, mark_student_attendance,
        mark_subject_attendance, request_absence_notification, update_exam_attendance,
        update_staff_attendance, update_student_attendance, update_subject_attendance,
        validate_bulk_import, AttendanceService, BulkMarkResult, EitherImportEvent,
    };
    pub use crate::value_objects::{
        AcademicYearId, AttendanceBulkId, AttendanceSource, AttendanceStatus, AttendanceType,
        BulkAttendanceImportId, ClassAttendanceId, ClassId, ExamAttendanceId, ExamId, ImportStatus,
        SectionId, StaffAttendanceId, StaffAttendanceImportId, StaffId, StudentAttendanceId,
        StudentAttendanceImportId, StudentId, StudentRecordId, SubjectAttendanceId, SubjectId,
    };
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    clippy::items_after_test_module
)]
mod tests {
    use super::*;

    #[test]
    fn package_metadata_is_set() {
        assert_eq!(PACKAGE_NAME, "educore-attendance");
        assert!(!PACKAGE_VERSION.is_empty());
    }

    #[test]
    fn prelude_wires_expected_types() {
        use crate::prelude::*;
        use educore_core::clock::SystemIdGen;
        let g = SystemIdGen;
        let s = g.next_school_id();
        let _: Capability = Capability::AttendanceStudentCreate;
        let _: Capability = Capability::AttendanceBulkMark;
        let _: AttendanceType = AttendanceType::Present;
        let _: AttendanceStatus = AttendanceStatus::Present;
        let _: AttendanceSource = AttendanceSource::Manual;
        let _: ImportStatus = ImportStatus::Pending;
        let _: StudentAttendanceId = StudentAttendanceId::new(s, g.next_uuid());
        let _: StaffId = StaffId::new(s, g.next_uuid());
    }
}
