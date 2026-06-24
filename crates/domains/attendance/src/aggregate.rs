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
    AcademicYearId, AttendanceBulkId, AttendanceSource, AttendanceType, BulkAttendanceImportId,
    ClassAttendanceId, ClassId, ExamAttendanceId, ExamTypeId, SectionId, StaffAttendanceId,
    StaffAttendanceImportId, StaffId, StudentAttendanceId, StudentAttendanceImportId, StudentId,
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

// =============================================================================
// ClassAttendance (projection aggregate — spec aggregates.md:210-232)
// =============================================================================

/// A per-(student, exam_type, academic_year) summary of
/// days opened, days present, days absent, etc. Used in
/// report cards and reports. The engine recomputes this
/// projection on demand from `StudentAttendanceMarked` and
/// `ExamAttendanceMarked` events.
///
/// **Spec invariant:**
/// `days_opened = days_present + days_absent + days_on_leave
/// + days_half_day * 0.5 + days_late`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassAttendance {
    /// The summary row's typed id.
    pub id: ClassAttendanceId,
    /// The school (tenant anchor; also embedded in the typed id).
    pub school_id: SchoolId,
    /// The student the summary is for.
    pub student_id: StudentId,
    /// The exam type the summary is scoped to.
    pub exam_type_id: ExamTypeId,
    /// The academic year the summary is scoped to.
    pub academic_year_id: AcademicYearId,
    /// The total number of school days opened in the year.
    pub days_opened: u32,
    /// The number of days the student was present.
    pub days_present: u32,
    /// The number of days the student was absent.
    pub days_absent: u32,
    /// The number of days the student was late.
    pub days_late: u32,
    /// The number of half-days the student logged.
    pub days_half_day: u32,
    /// The number of days the student was on approved leave.
    pub days_on_leave: u32,
    /// The wall-clock instant the summary was last recomputed.
    pub recomputed_at: Timestamp,
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

impl ClassAttendance {
    /// Constructs a new [`ClassAttendance`] projection row.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn fresh(
        id: ClassAttendanceId,
        student_id: StudentId,
        exam_type_id: ExamTypeId,
        academic_year_id: AcademicYearId,
        days_opened: u32,
        days_present: u32,
        days_absent: u32,
        days_late: u32,
        days_half_day: u32,
        days_on_leave: u32,
        recomputed_at: Timestamp,
        recomputed_by: UserId,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            student_id,
            exam_type_id,
            academic_year_id,
            days_opened,
            days_present,
            days_absent,
            days_late,
            days_half_day,
            days_on_leave,
            recomputed_at,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: recomputed_at,
            updated_at: recomputed_at,
            created_by: recomputed_by,
            updated_by: recomputed_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Returns `true` if the row is currently active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Handler skeleton: validate the spec invariant
    /// `days_opened = days_present + days_absent +
    /// days_on_leave + days_half_day * 0.5 + days_late`.
    ///
    /// **TODO:** The full invariant check (and the
    /// `ClassAttendanceRecomputed` event emission) lands in
    /// Phase 5 Workstream C. This skeleton reserves the
    /// method signature so the storage-parity test
    /// `attendance_class_attendances_aggregate` can wire up
    /// against it.
    pub fn verify_invariants(&self) -> educore_core::error::Result<()> {
        Err(educore_core::error::DomainError::not_supported(
            "ClassAttendance::verify_invariants TODO: wire invariant check + emit ClassAttendanceRecomputed",
        ))
    }
}

// =============================================================================
// AttendanceBulk (bulk-import staging aggregate — spec aggregates.md:235-248)
// =============================================================================

/// A per-(student, date) row materialized during a bulk
/// import. This is the denormalized staging representation
/// used by the consumer's import wizard; on commit, the
/// engine promotes each row into a [`StudentAttendance`].
///
/// **Spec invariant:** belongs to exactly one
/// [`BulkAttendanceImport`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttendanceBulk {
    /// The bulk row's typed id.
    pub id: AttendanceBulkId,
    /// The school (tenant anchor; also embedded in the typed id).
    pub school_id: SchoolId,
    /// The parent bulk-import job.
    pub bulk_import_id: BulkAttendanceImportId,
    /// The student this row records attendance for.
    pub student_id: StudentId,
    /// The calendar day the row records attendance for.
    pub attendance_date: NaiveDate,
    /// The single-character attendance code.
    pub attendance_type: AttendanceType,
    /// Optional wall-clock sign-in time (string form, e.g. `"08:45"`).
    pub in_time: Option<String>,
    /// Optional wall-clock sign-out time.
    pub out_time: Option<String>,
    /// Free-form notes.
    pub notes: Option<String>,
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

impl AttendanceBulk {
    /// Constructs a new [`AttendanceBulk`] staging row.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn fresh(
        id: AttendanceBulkId,
        bulk_import_id: BulkAttendanceImportId,
        student_id: StudentId,
        attendance_date: NaiveDate,
        attendance_type: AttendanceType,
        in_time: Option<String>,
        out_time: Option<String>,
        notes: Option<String>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            bulk_import_id,
            student_id,
            attendance_date,
            attendance_type,
            in_time,
            out_time,
            notes,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Returns `true` if the row is currently active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Handler skeleton: promote the staging row into a
    /// fully-fledged [`StudentAttendance`].
    ///
    /// **TODO:** The promote logic (and the
    /// `StudentAttendanceImported` event emission) lands in
    /// Phase 5 Workstream C. This skeleton reserves the
    /// method signature so the storage-parity test
    /// `attendance_attendance_bulks_aggregate` can wire up
    /// against it.
    pub fn promote_to_student_attendance(
        &self,
    ) -> educore_core::error::Result<crate::aggregate::StudentAttendance> {
        Err(educore_core::error::DomainError::not_supported(
            "AttendanceBulk::promote_to_student_attendance TODO: build StudentAttendance + emit StudentAttendanceImported",
        ))
    }
}

// =============================================================================
// Happy-path tests for the 2 new aggregates
// =============================================================================

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]

// =============================================================================
// StudentAttendanceImport / StaffAttendanceImport (Cluster D mop-up)
// =============================================================================

/// Staging row for a student bulk import.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StudentAttendanceImport {
    pub id: StudentAttendanceImportId,
    pub school_id: SchoolId,
}

/// Staging row for a staff bulk import.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffAttendanceImport {
    pub id: StaffAttendanceImportId,
    pub school_id: SchoolId,
}

mod tests {
    use super::*;
    use educore_core::clock::{IdGenerator, SystemIdGen};

    /// Happy-path: `ClassAttendance::fresh` populates all
    /// required fields and `verify_invariants` returns
    /// `Err(not_supported)` (the Phase 5 Workstream C stub).
    #[test]
    fn class_attendance_fresh_and_verify_invariants_stub() {
        let gen = SystemIdGen;
        let school = gen.next_school_id();
        let id = ClassAttendanceId::new(school, gen.next_uuid());
        let student = StudentId::new(school, gen.next_uuid());
        let exam_type = ExamTypeId::new(school, gen.next_uuid());
        let year = AcademicYearId::new(school, gen.next_uuid());
        let actor = UserId(uuid::Uuid::now_v7());
        let now = Timestamp::now();
        let corr = CorrelationId(uuid::Uuid::now_v7());

        let row = ClassAttendance::fresh(
            id,
            student,
            exam_type,
            year,
            200, // days_opened
            180, // days_present
            10,  // days_absent
            5,   // days_late
            4,   // days_half_day
            1,   // days_on_leave
            now,
            actor,
            corr,
        );

        assert_eq!(row.school_id, school);
        assert_eq!(row.student_id, student);
        assert_eq!(row.days_opened, 200);
        assert!(row.is_active());
        assert!(row.verify_invariants().is_err());
    }

    /// Happy-path: `AttendanceBulk::fresh` populates all
    /// required fields and `promote_to_student_attendance`
    /// returns `Err(not_supported)` (the Phase 5 Workstream C
    /// stub).
    #[test]
    fn attendance_bulk_fresh_and_promote_stub() {
        let gen = SystemIdGen;
        let school = gen.next_school_id();
        let id = AttendanceBulkId::new(school, gen.next_uuid());
        let job = BulkAttendanceImportId::new(school, gen.next_uuid());
        let student = StudentId::new(school, gen.next_uuid());
        let actor = UserId(uuid::Uuid::now_v7());
        let now = Timestamp::now();
        let corr = CorrelationId(uuid::Uuid::now_v7());

        let row = AttendanceBulk::fresh(
            id,
            job,
            student,
            NaiveDate::from_ymd_opt(2026, 5, 14).unwrap(),
            AttendanceType::Present,
            Some("08:45".to_string()),
            Some("15:30".to_string()),
            None,
            actor,
            now,
            corr,
        );

        assert_eq!(row.school_id, school);
        assert_eq!(row.bulk_import_id, job);
        assert_eq!(row.attendance_type, AttendanceType::Present);
        assert!(row.is_active());
        assert!(row.promote_to_student_attendance().is_err());
    }
}
