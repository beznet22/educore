//! # Attendance repository ports
//!
//! Phase 5 Workstream A ships 5 repository port traits:
//!
//! - [`StudentAttendanceRepository`] — 9 methods
//!   (including the bulk_insert path that delegates to
//!   `tx.bulk_insert_student_attendances(...)`).
//! - [`SubjectAttendanceRepository`] — 5 methods.
//! - [`StaffAttendanceRepository`] — 7 methods.
//! - [`ExamAttendanceRepository`] — 5 methods.
//! - [`AttendanceImportRepository`] — 10 methods for the
//!   bulk-import job + the staging rows.
//!
//! All port traits are `#[async_trait] pub trait
//! XxxRepository: Send + Sync` per the academic crate's
//! pattern. The storage adapters (Phase 1) provide the
//! concrete implementations.

#![allow(missing_docs)] // The async trait method signatures
                        // are described by their parameter
                        // names; suppressing this lint for the
                        // file is the pragmatic choice for the
                        // 5 repo traits Phase 5 ships.

use async_trait::async_trait;

use chrono::NaiveDate;

use educore_assessment::ExamId;
use educore_core::error::Result;
use educore_core::ids::SchoolId;
use educore_core::tenant::TenantContext;

use crate::aggregate::{
    BulkAttendanceImport, ExamAttendance, StaffAttendance, StudentAttendance, SubjectAttendance,
};
use crate::entities::{StaffAttendanceImport, StudentAttendanceImport};
use crate::value_objects::{
    AcademicYearId, AttendanceSource, BulkAttendanceImportId, ClassId, ExamAttendanceId, SectionId,
    StaffAttendanceId, StaffId, StudentAttendanceId, StudentId, SubjectAttendanceId,
};

// =============================================================================
// StudentAttendanceRepository
// =============================================================================

/// The storage-port contract for
/// [`StudentAttendance`](crate::aggregate::StudentAttendance)
/// rows. Every method is tenant-scoped: the implementation
/// MUST filter on `ctx.school_id` (or reject commands that
/// do not match).
#[allow(dead_code)]
#[async_trait]
pub trait StudentAttendanceRepository: Send + Sync {
    /// Returns the [`StudentAttendance`] with the given id
    /// (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: StudentAttendanceId,
    ) -> Result<Option<StudentAttendance>>;

    /// Returns the [`StudentAttendance`] for the unique key
    /// `(school_id, student_id, attendance_date)` (or
    /// `Ok(None)` if not found).
    async fn find(
        &self,
        school: SchoolId,
        student: StudentId,
        date: NaiveDate,
    ) -> Result<Option<StudentAttendance>>;

    /// Returns every [`StudentAttendance`] for the given
    /// `(school, student, from, to)`. Stable ordering.
    async fn list_for_student(
        &self,
        school: SchoolId,
        student: StudentId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<StudentAttendance>>;

    /// Returns every [`StudentAttendance`] for the given
    /// `(school, class, section, date)`.
    async fn list_for_section(
        &self,
        school: SchoolId,
        class: ClassId,
        section: SectionId,
        date: NaiveDate,
    ) -> Result<Vec<StudentAttendance>>;

    /// Returns every [`StudentAttendance`] for the given
    /// `(school, class, from, to)`.
    async fn list_for_class_in_range(
        &self,
        school: SchoolId,
        class: ClassId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<StudentAttendance>>;

    /// Returns every absent [`StudentAttendance`] for the
    /// given `(school, date)`. Used by the daily absence
    /// report and the notification fan-out.
    async fn list_absent_for_day(
        &self,
        school: SchoolId,
        date: NaiveDate,
    ) -> Result<Vec<StudentAttendance>>;

    /// Inserts a new [`StudentAttendance`] row.
    async fn insert(&self, ctx: &TenantContext, a: &StudentAttendance) -> Result<()>;

    /// Updates an existing [`StudentAttendance`] row. MUST
    /// enforce the `version` optimistic-concurrency check.
    async fn update(&self, ctx: &TenantContext, a: &StudentAttendance) -> Result<()>;

    /// Bulk-inserts N [`StudentAttendance`] rows in a
    /// single transaction. The implementation MUST use the
    /// storage port's `tx.bulk_insert_student_attendances(...)`
    /// method (multi-row `INSERT` on PG / MySQL;
    /// transaction-grouped inserts on SQLite). The adapter
    /// rejects rows whose `school_id` does not match
    /// `ctx.school_id`.
    async fn bulk_insert(&self, ctx: &TenantContext, rows: &[StudentAttendance]) -> Result<()>;
}

// =============================================================================
// SubjectAttendanceRepository
// =============================================================================

/// The storage-port contract for
/// [`SubjectAttendance`](crate::aggregate::SubjectAttendance)
/// rows.
#[allow(dead_code)]
#[async_trait]
pub trait SubjectAttendanceRepository: Send + Sync {
    async fn get(
        &self,
        ctx: &TenantContext,
        id: SubjectAttendanceId,
    ) -> Result<Option<SubjectAttendance>>;

    /// Returns every [`SubjectAttendance`] for the given
    /// `(school, student, from, to)`.
    async fn list_for_student(
        &self,
        school: SchoolId,
        student: StudentId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<SubjectAttendance>>;

    /// Returns every [`SubjectAttendance`] for the given
    /// `(school, class, section, date)`.
    async fn list_for_section(
        &self,
        school: SchoolId,
        class: ClassId,
        section: SectionId,
        date: NaiveDate,
    ) -> Result<Vec<SubjectAttendance>>;

    async fn insert(&self, ctx: &TenantContext, a: &SubjectAttendance) -> Result<()>;

    async fn update(&self, ctx: &TenantContext, a: &SubjectAttendance) -> Result<()>;
}

// =============================================================================
// StaffAttendanceRepository
// =============================================================================

/// The storage-port contract for
/// [`StaffAttendance`](crate::aggregate::StaffAttendance)
/// rows.
#[allow(dead_code)]
#[async_trait]
pub trait StaffAttendanceRepository: Send + Sync {
    async fn get(
        &self,
        ctx: &TenantContext,
        id: StaffAttendanceId,
    ) -> Result<Option<StaffAttendance>>;

    async fn find(
        &self,
        school: SchoolId,
        staff: StaffId,
        date: NaiveDate,
    ) -> Result<Option<StaffAttendance>>;

    async fn list_for_staff(
        &self,
        school: SchoolId,
        staff: StaffId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<StaffAttendance>>;

    async fn list_for_day(&self, school: SchoolId, date: NaiveDate)
        -> Result<Vec<StaffAttendance>>;

    async fn list_absent_for_day(
        &self,
        school: SchoolId,
        date: NaiveDate,
    ) -> Result<Vec<StaffAttendance>>;

    async fn insert(&self, ctx: &TenantContext, a: &StaffAttendance) -> Result<()>;

    async fn update(&self, ctx: &TenantContext, a: &StaffAttendance) -> Result<()>;
}

// =============================================================================
// ExamAttendanceRepository
// =============================================================================

/// The storage-port contract for
/// [`ExamAttendance`](crate::aggregate::ExamAttendance) rows.
#[allow(dead_code)]
#[async_trait]
pub trait ExamAttendanceRepository: Send + Sync {
    async fn get(
        &self,
        ctx: &TenantContext,
        id: ExamAttendanceId,
    ) -> Result<Option<ExamAttendance>>;

    /// Returns every [`ExamAttendance`] for the given
    /// `(school, exam)`.
    async fn list_for_exam(&self, school: SchoolId, exam: ExamId) -> Result<Vec<ExamAttendance>>;

    /// Returns every [`ExamAttendance`] for the given
    /// `(school, exam, student)`.
    async fn list_for_student(
        &self,
        school: SchoolId,
        exam: ExamId,
        student: StudentId,
    ) -> Result<Vec<ExamAttendance>>;

    async fn insert(&self, ctx: &TenantContext, a: &ExamAttendance) -> Result<()>;

    async fn update(&self, ctx: &TenantContext, a: &ExamAttendance) -> Result<()>;
}

// =============================================================================
// AttendanceImportRepository
// =============================================================================

/// The storage-port contract for
/// [`BulkAttendanceImport`](crate::aggregate::BulkAttendanceImport)
/// jobs and their staging rows.
#[allow(dead_code)]
#[async_trait]
pub trait AttendanceImportRepository: Send + Sync {
    /// Returns the [`BulkAttendanceImport`] with the given id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: BulkAttendanceImportId,
    ) -> Result<Option<BulkAttendanceImport>>;

    /// Returns every [`BulkAttendanceImport`] for the given
    /// `(school, academic_year)`. Stable ordering by
    /// `marked_at` descending.
    async fn list_for_year(
        &self,
        school: SchoolId,
        year: AcademicYearId,
    ) -> Result<Vec<BulkAttendanceImport>>;

    /// Returns every [`BulkAttendanceImport`] for the given
    /// `(school, source, date)`. Used by the
    /// `import_source_date_exists` uniqueness check.
    async fn list_for_source_date(
        &self,
        school: SchoolId,
        source: AttendanceSource,
        date: NaiveDate,
    ) -> Result<Vec<BulkAttendanceImport>>;

    /// Inserts a new [`BulkAttendanceImport`] job.
    async fn insert(&self, ctx: &TenantContext, i: &BulkAttendanceImport) -> Result<()>;

    /// Updates an existing [`BulkAttendanceImport`] job.
    async fn update(&self, ctx: &TenantContext, i: &BulkAttendanceImport) -> Result<()>;

    /// Returns every [`StudentAttendanceImport`] staging row
    /// for the given bulk import.
    async fn list_staging_rows(
        &self,
        ctx: &TenantContext,
        bulk_id: BulkAttendanceImportId,
    ) -> Result<Vec<StudentAttendanceImport>>;

    /// Inserts a batch of [`StudentAttendanceImport`] staging
    /// rows.
    async fn insert_staging_rows(
        &self,
        ctx: &TenantContext,
        rows: &[StudentAttendanceImport],
    ) -> Result<()>;

    /// Marks the given staging rows as validated (or
    /// unvalidated, with the supplied `is_validated` flag).
    async fn mark_staging_validated(
        &self,
        ctx: &TenantContext,
        bulk_id: BulkAttendanceImportId,
        row_ids: &[crate::value_objects::StudentAttendanceImportId],
        is_validated: bool,
    ) -> Result<()>;

    /// Returns every [`StaffAttendanceImport`] staging row
    /// for the given bulk import.
    async fn list_staff_staging_rows(
        &self,
        ctx: &TenantContext,
        bulk_id: BulkAttendanceImportId,
    ) -> Result<Vec<StaffAttendanceImport>>;

    /// Inserts a batch of [`StaffAttendanceImport`] staging
    /// rows.
    async fn insert_staff_staging_rows(
        &self,
        ctx: &TenantContext,
        rows: &[StaffAttendanceImport],
    ) -> Result<()>;
}

// =============================================================================
// Object-safety test
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

    /// Compile-time check: confirm every repository trait is
    /// object-safe by naming `Box<dyn ...>` for it. If a
    /// trait gains a generic method, this assertion fails
    /// to compile.
    #[test]
    fn traits_are_object_safe() {
        fn _student_attendance_repository(_: Box<dyn StudentAttendanceRepository>) {}
        fn _subject_attendance_repository(_: Box<dyn SubjectAttendanceRepository>) {}
        fn _staff_attendance_repository(_: Box<dyn StaffAttendanceRepository>) {}
        fn _exam_attendance_repository(_: Box<dyn ExamAttendanceRepository>) {}
        fn _attendance_import_repository(_: Box<dyn AttendanceImportRepository>) {}
    }
}
