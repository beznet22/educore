//! # Attendance-domain child entities
//!
//! Per-aggregate children that are not part of the aggregate
//! root's struct but are owned by it. Phase 5 ships:
//!
//! - [`StudentAttendanceImport`] — one validated row in a
//!   [`BulkAttendanceImport`](crate::aggregate::BulkAttendanceImport).
//!   Carries `StudentId`, `AttendanceDate`, `AttendanceType`,
//!   `InTime`, `OutTime`, `Notes`, `IsValidated`.
//! - [`StaffAttendanceImport`] — one validated staff row in a
//!   `BulkAttendanceImport`. Carries `StaffId` (placeholder
//!   until the HR domain, Phase 6) plus the per-row fields.

#![allow(missing_docs)] // The child-entity fields are
                        // self-documenting via the type names;
                        // suppressing this lint for the file is
                        // the pragmatic choice for the 2
                        // entities Phase 5 ships.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use educore_core::ids::SchoolId;
use educore_core::value_objects::ActiveStatus;

use crate::value_objects::{
    AttendanceType, BulkAttendanceImportId, StaffAttendanceImportId, StaffId,
    StudentAttendanceImportId, StudentId,
};

// =============================================================================
// StudentAttendanceImport
// =============================================================================

/// A validated row in a [`BulkAttendanceImport`](crate::aggregate::BulkAttendanceImport).
/// Carries the `(student_id, attendance_date, attendance_type)`
/// triple plus optional `in_time` / `out_time` and free-form
/// notes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StudentAttendanceImport {
    /// The child's typed id.
    pub id: StudentAttendanceImportId,
    /// The parent bulk-import job.
    pub bulk_import_id: BulkAttendanceImportId,
    /// The student this row records attendance for.
    pub student_id: StudentId,
    /// The calendar day the row records attendance for.
    pub attendance_date: NaiveDate,
    /// The single-character attendance code.
    pub attendance_type: AttendanceType,
    /// The wall-clock time the student signed in (optional).
    pub in_time: Option<String>,
    /// The wall-clock time the student signed out (optional).
    pub out_time: Option<String>,
    /// Free-form notes.
    pub notes: Option<String>,
    /// `true` if the row passed the `validate_bulk_import`
    /// rules (well-formed date, no duplicate key, etc.).
    pub is_validated: bool,
    /// Soft-delete flag.
    pub active_status: ActiveStatus,
}

impl StudentAttendanceImport {
    /// Returns `true` if the row is well-formed: the
    /// `attendance_date` is not in the future.
    #[must_use]
    pub fn is_well_formed(&self) -> bool {
        self.attendance_date <= chrono::Utc::now().date_naive()
    }

    /// Returns `true` if the row passed validation.
    #[must_use]
    pub const fn is_validated(&self) -> bool {
        self.is_validated
    }

    /// Returns the parent school id (the bulk import's
    /// tenant anchor). Convenience helper for the storage
    /// adapter's `school_id` check.
    #[must_use]
    pub const fn school_id(&self) -> SchoolId {
        self.id.school_id
    }
}

// =============================================================================
// StaffAttendanceImport
// =============================================================================

/// A validated staff row in a
/// [`BulkAttendanceImport`](crate::aggregate::BulkAttendanceImport).
/// Carries the `(staff_id, attendance_date, attendance_type)`
/// triple plus optional `in_time` / `out_time` and notes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffAttendanceImport {
    /// The child's typed id.
    pub id: StaffAttendanceImportId,
    /// The parent bulk-import job.
    pub bulk_import_id: BulkAttendanceImportId,
    /// The staff member this row records attendance for.
    /// Placeholder typed id (the full `Staff` aggregate
    /// lands in the HR domain, Phase 6).
    pub staff_id: StaffId,
    /// The calendar day the row records attendance for.
    pub attendance_date: NaiveDate,
    /// The single-character attendance code.
    pub attendance_type: AttendanceType,
    /// The wall-clock time the staff member signed in
    /// (optional).
    pub in_time: Option<String>,
    /// The wall-clock time the staff member signed out
    /// (optional).
    pub out_time: Option<String>,
    /// Free-form notes.
    pub notes: Option<String>,
    /// `true` if the row passed the `validate_bulk_import`
    /// rules.
    pub is_validated: bool,
    /// Soft-delete flag.
    pub active_status: ActiveStatus,
}

impl StaffAttendanceImport {
    /// Returns `true` if the row is well-formed: the
    /// `attendance_date` is not in the future.
    #[must_use]
    pub fn is_well_formed(&self) -> bool {
        self.attendance_date <= chrono::Utc::now().date_naive()
    }

    /// Returns `true` if the row passed validation.
    #[must_use]
    pub const fn is_validated(&self) -> bool {
        self.is_validated
    }

    /// Returns the parent school id.
    #[must_use]
    pub const fn school_id(&self) -> SchoolId {
        self.id.school_id
    }
}
