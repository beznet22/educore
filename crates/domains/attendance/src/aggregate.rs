//! # Attendance aggregate roots
//!
//! Phase 5 Workstream A ships 5 prompt-named aggregates:
//!
//! - [`StudentAttendance`] — daily student attendance.
//! - [`StaffAttendance`] — daily staff attendance.
//! - [`SubjectAttendance`] — per-period (per-subject) student
//!   attendance.
//! - [`ExamAttendance`] — per-exam student attendance.
//! - [`BulkAttendanceImport`] — a bulk import job (CSV /
//!   biometric / API).
//!
//! All 5 follow the "aggregate as a single struct" pattern
//! (mirroring `educore-academic`'s [`Student`](educore_academic::Student)
//! and `educore-platform`'s [`School`](educore_platform::School)):
//! the struct holds the full state, with `version` for
//! optimistic concurrency, `etag` for content hashing,
//! `active_status` for soft delete, and `last_event_id` /
//! `correlation_id` for the audit / outbox bridge.

#![allow(missing_docs)] // The 10 audit-metadata fields
                        // (version, etag, created_at, ...) on each
                        // aggregate are described by their type
                        // names; suppressing this lint for the
                        // file is the pragmatic choice for the
                        // 5 aggregates Phase 5 ships.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};

use educore_assessment::ExamId;

use crate::value_objects::{
    AcademicYearId, AttendanceSource, AttendanceType, BulkAttendanceImportId, ClassId,
    ExamAttendanceId, SectionId, StaffAttendanceId, StaffId, StudentAttendanceId, StudentId,
    StudentRecordId, SubjectAttendanceId, SubjectId,
};

/// Returns the default etag for a freshly minted aggregate.
///
/// Delegates to [`Etag::placeholder`] (an infallible
/// constructor) so callers do not need to handle a `Result`.
fn fresh_etag() -> Etag {
    Etag::placeholder()
}

// =============================================================================
// StudentAttendance
// =============================================================================

/// A daily student attendance mark. One row per
/// `(school, student, attendance_date)`. The unique key is
/// enforced by the storage adapter; the engine services
/// assert uniqueness in-process via the
/// [`AttendanceUniquenessChecker`](crate::commands::AttendanceUniquenessChecker)
/// port.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StudentAttendance {
    /// The attendance row's typed id.
    pub id: StudentAttendanceId,
    /// The school (tenant anchor; also embedded in the typed
    /// id).
    pub school_id: SchoolId,
    /// The student this row records attendance for.
    pub student_id: StudentId,
    /// The per-year enrollment handle for the student on the
    /// attendance date.
    pub student_record_id: StudentRecordId,
    /// The class the student was enrolled in on the
    /// attendance date.
    pub class_id: ClassId,
    /// The section the student was enrolled in on the
    /// attendance date.
    pub section_id: SectionId,
    /// The calendar day the row records attendance for.
    pub attendance_date: NaiveDate,
    /// The single-character attendance code.
    pub attendance_type: AttendanceType,
    /// The wall-clock time the student signed in (optional).
    pub in_time: Option<String>,
    /// The wall-clock time the student signed out (optional).
    pub out_time: Option<String>,
    /// Free-form notes (absence reason, late arrival, …).
    pub notes: Option<String>,
    /// Denormalised absence flag. Kept in sync with
    /// `attendance_type.is_absent()`; the storage adapter
    /// rejects writes that violate the invariant.
    pub is_absent: bool,
    /// The user (or `SYSTEM`) who recorded the mark.
    pub marked_by: UserId,
    /// The instant the mark was recorded.
    pub marked_at: Timestamp,
    /// The channel that produced the mark.
    pub marked_from: AttendanceSource,
    /// Standard 10-field audit-metadata footer (per the
    /// 17-field pattern).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl StudentAttendance {
    /// The 32-char zero etag for a freshly minted row.
    pub const FRESH_ETAG: &'static str = "00000000000000000000000000000000";

    /// Constructs a new [`StudentAttendance`] aggregate.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn fresh(
        id: StudentAttendanceId,
        student_id: StudentId,
        student_record_id: StudentRecordId,
        class_id: ClassId,
        section_id: SectionId,
        attendance_date: NaiveDate,
        attendance_type: AttendanceType,
        in_time: Option<String>,
        out_time: Option<String>,
        notes: Option<String>,
        is_absent: bool,
        marked_by: UserId,
        marked_at: Timestamp,
        marked_from: AttendanceSource,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            student_id,
            student_record_id,
            class_id,
            section_id,
            attendance_date,
            attendance_type,
            in_time,
            out_time,
            notes,
            is_absent,
            marked_by,
            marked_at,
            marked_from,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: marked_at,
            updated_at: marked_at,
            created_by: marked_by,
            updated_by: marked_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Returns `true` if the row is currently active (not
    /// soft-deleted).
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Returns `true` if the row was marked absent. Mirrors
    /// [`AttendanceType::is_absent`] on `attendance_type`.
    #[must_use]
    pub const fn is_absent(&self) -> bool {
        self.is_absent
    }

    // Note: the `to_row()` / `from_row()` conversion to the
    // storage port's `StudentAttendanceRow` wire type lives
    // in the engine facade (Phase 16), not in this domain
    // crate. Domain crates in the `domains` tier may not
    // depend on crates in the `infra` or `adapters` tiers
    // (per `AGENTS.md` § Tier System). The dispatcher
    // performs the conversion in the engine facade.
}

// =============================================================================
// StaffAttendance
// =============================================================================

/// A daily staff attendance mark. One row per
/// `(school, staff, attendance_date)`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffAttendance {
    /// The attendance row's typed id.
    pub id: StaffAttendanceId,
    /// The school (tenant anchor).
    pub school_id: SchoolId,
    /// The staff member this row records attendance for.
    pub staff_id: StaffId,
    /// The calendar day the row records attendance for.
    pub attendance_date: NaiveDate,
    /// The single-character attendance code.
    pub attendance_type: AttendanceType,
    /// The wall-clock time the staff member signed in.
    pub in_time: Option<String>,
    /// The wall-clock time the staff member signed out.
    pub out_time: Option<String>,
    /// Free-form notes.
    pub notes: Option<String>,
    /// The user (or `SYSTEM`) who recorded the mark.
    pub marked_by: UserId,
    /// The instant the mark was recorded.
    pub marked_at: Timestamp,
    /// The channel that produced the mark.
    pub marked_from: AttendanceSource,
    /// Standard 10-field audit-metadata footer.
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl StaffAttendance {
    /// The 32-char zero etag for a freshly minted row.
    pub const FRESH_ETAG: &'static str = "00000000000000000000000000000000";

    /// Constructs a new [`StaffAttendance`] aggregate.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn fresh(
        id: StaffAttendanceId,
        staff_id: StaffId,
        attendance_date: NaiveDate,
        attendance_type: AttendanceType,
        in_time: Option<String>,
        out_time: Option<String>,
        notes: Option<String>,
        marked_by: UserId,
        marked_at: Timestamp,
        marked_from: AttendanceSource,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            staff_id,
            attendance_date,
            attendance_type,
            in_time,
            out_time,
            notes,
            marked_by,
            marked_at,
            marked_from,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: marked_at,
            updated_at: marked_at,
            created_by: marked_by,
            updated_by: marked_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Returns `true` if the row is currently active (not
    /// soft-deleted).
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Returns `true` if the staff member was marked absent.
    #[must_use]
    pub const fn is_absent(&self) -> bool {
        self.attendance_type.is_absent()
    }
}

// =============================================================================
// SubjectAttendance
// =============================================================================

/// A per-period (per-subject) student attendance mark. One
/// row per `(school, student, subject, attendance_date)`.
/// Used when a school marks attendance per class period
/// (typically secondary schools with multiple periods per
/// day).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubjectAttendance {
    /// The attendance row's typed id.
    pub id: SubjectAttendanceId,
    /// The school (tenant anchor).
    pub school_id: SchoolId,
    /// The student this row records attendance for.
    pub student_id: StudentId,
    /// The per-year enrollment handle for the student on the
    /// attendance date.
    pub student_record_id: StudentRecordId,
    /// The class the student was enrolled in.
    pub class_id: ClassId,
    /// The section the student was enrolled in.
    pub section_id: SectionId,
    /// The subject (period) this row covers.
    pub subject_id: SubjectId,
    /// The calendar day the row records attendance for.
    pub attendance_date: NaiveDate,
    /// The single-character attendance code.
    pub attendance_type: AttendanceType,
    /// Free-form notes.
    pub notes: Option<String>,
    /// Whether the absence should trigger a notification to
    /// the parent. Defaults to `true`; configurable per
    /// school's `NotificationSetting`.
    pub notify: bool,
    /// The user (or `SYSTEM`) who recorded the mark.
    pub marked_by: UserId,
    /// The instant the mark was recorded.
    pub marked_at: Timestamp,
    /// The channel that produced the mark.
    pub marked_from: AttendanceSource,
    /// Standard 10-field audit-metadata footer.
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl SubjectAttendance {
    /// The 32-char zero etag for a freshly minted row.
    pub const FRESH_ETAG: &'static str = "00000000000000000000000000000000";

    /// Constructs a new [`SubjectAttendance`] aggregate.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn fresh(
        id: SubjectAttendanceId,
        student_id: StudentId,
        student_record_id: StudentRecordId,
        class_id: ClassId,
        section_id: SectionId,
        subject_id: SubjectId,
        attendance_date: NaiveDate,
        attendance_type: AttendanceType,
        notes: Option<String>,
        notify: bool,
        marked_by: UserId,
        marked_at: Timestamp,
        marked_from: AttendanceSource,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
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
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: marked_at,
            updated_at: marked_at,
            created_by: marked_by,
            updated_by: marked_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Returns `true` if the row is currently active (not
    /// soft-deleted).
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Returns `true` if the student was marked absent for
    /// this subject.
    #[must_use]
    pub const fn is_absent(&self) -> bool {
        self.attendance_type.is_absent()
    }
}

// =============================================================================
// ExamAttendance
// =============================================================================

/// A per-exam student attendance mark. One row per
/// `(school, exam, student, exam_date)`. Cross-crate dep on
/// [`ExamId`](educore_assessment::ExamId) (per
/// `docs/decisions/ADR-013-CrateLayout.md` and the Phase 5
/// hand-off § "ExamAttendance ownership").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExamAttendance {
    /// The attendance row's typed id.
    pub id: ExamAttendanceId,
    /// The school (tenant anchor).
    pub school_id: SchoolId,
    /// The exam this row records attendance for.
    pub exam_id: ExamId,
    /// The student this row records attendance for.
    pub student_id: StudentId,
    /// The per-year enrollment handle for the student on the
    /// exam date.
    pub student_record_id: StudentRecordId,
    /// The class the exam is administered to.
    pub class_id: ClassId,
    /// The section the exam is administered to.
    pub section_id: SectionId,
    /// The subject the exam is for.
    pub subject_id: SubjectId,
    /// The exam date.
    pub exam_date: NaiveDate,
    /// The single-character attendance code.
    pub attendance_type: AttendanceType,
    /// Free-form notes.
    pub notes: Option<String>,
    /// The user (or `SYSTEM`) who recorded the mark.
    pub marked_by: UserId,
    /// The instant the mark was recorded.
    pub marked_at: Timestamp,
    /// The channel that produced the mark.
    pub marked_from: AttendanceSource,
    /// Standard 10-field audit-metadata footer.
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl ExamAttendance {
    /// The 32-char zero etag for a freshly minted row.
    pub const FRESH_ETAG: &'static str = "00000000000000000000000000000000";

    /// Constructs a new [`ExamAttendance`] aggregate.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn fresh(
        id: ExamAttendanceId,
        exam_id: ExamId,
        student_id: StudentId,
        student_record_id: StudentRecordId,
        class_id: ClassId,
        section_id: SectionId,
        subject_id: SubjectId,
        exam_date: NaiveDate,
        attendance_type: AttendanceType,
        notes: Option<String>,
        marked_by: UserId,
        marked_at: Timestamp,
        marked_from: AttendanceSource,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
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
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: marked_at,
            updated_at: marked_at,
            created_by: marked_by,
            updated_by: marked_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Returns `true` if the row is currently active (not
    /// soft-deleted).
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Returns `true` if the student was marked absent for
    /// the exam.
    #[must_use]
    pub const fn is_absent(&self) -> bool {
        self.attendance_type.is_absent()
    }
}

// =============================================================================
// BulkAttendanceImport
// =============================================================================

/// A bulk attendance import job. Captures a CSV / biometric /
/// API import batch. Owns a set of
/// [`StudentAttendanceImport`](crate::entities::StudentAttendanceImport)
/// and
/// [`StaffAttendanceImport`](crate::entities::StaffAttendanceImport)
/// child rows that are validated, then promoted into the
/// live `StudentAttendance` / `StaffAttendance` tables on
/// commit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BulkAttendanceImport {
    /// The import job's typed id.
    pub id: BulkAttendanceImportId,
    /// The school (tenant anchor).
    pub school_id: SchoolId,
    /// The academic year the import is for.
    pub academic_year_id: AcademicYearId,
    /// The channel that produced the import.
    pub source: AttendanceSource,
    /// The current job status.
    pub status: crate::value_objects::ImportStatus,
    /// The number of staging rows the import carries.
    pub row_count: u32,
    /// The number of staging rows the validator flagged as
    /// absent. Updated on `validate_bulk_import`.
    pub absent_count: u32,
    /// The number of staging rows the validator rejected on
    /// the last validation pass. `0` for a clean import.
    pub failed_count: u32,
    /// Free-form operator notes (e.g. file name, vendor
    /// feed name).
    pub notes: Option<String>,
    /// The user (or `SYSTEM`) who started the import.
    pub marked_by: UserId,
    /// The instant the import was started.
    pub marked_at: Timestamp,
    /// Standard 10-field audit-metadata footer.
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl BulkAttendanceImport {
    /// The 32-char zero etag for a freshly minted job.
    pub const FRESH_ETAG: &'static str = "00000000000000000000000000000000";

    /// Constructs a new [`BulkAttendanceImport`] aggregate.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn fresh(
        id: BulkAttendanceImportId,
        academic_year_id: AcademicYearId,
        source: AttendanceSource,
        row_count: u32,
        marked_by: UserId,
        marked_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            academic_year_id,
            source,
            status: crate::value_objects::ImportStatus::Pending,
            row_count,
            absent_count: 0,
            failed_count: 0,
            notes: None,
            marked_by,
            marked_at,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: marked_at,
            updated_at: marked_at,
            created_by: marked_by,
            updated_by: marked_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Returns `true` if the job is in a terminal state
    /// (Committed, Failed, or Cancelled).
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        self.status.is_terminal()
    }

    /// Returns `true` if the job is currently active (not
    /// soft-deleted).
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active_status.is_active()
    }
}
