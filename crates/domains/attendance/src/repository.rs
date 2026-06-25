//! # Attendance repository ports
//!
//! The attendance domain ships 10 repository port traits:
//!
//! - [`StudentAttendanceRepository`] — 9 methods
//!   (including the bulk_insert path that delegates to
//!   `tx.bulk_insert_student_attendances(...)`).
//! - [`SubjectAttendanceRepository`] — 5 methods.
//! - [`StaffAttendanceRepository`] — 7 methods.
//! - [`ExamAttendanceRepository`] — 5 methods.
//! - [`AttendanceImportRepository`] — 12 methods for the
//!   bulk-import job + the staging rows.
//! - [`ClassAttendanceRepository`] — 4 methods for the
//!   projection.
//! - [`AttendanceBulkRepository`] — 9 methods for the
//!   per-(student, date) staging rows.
//! - [`BulkAttendanceImportRepository`] — 8 methods for
//!   the bulk-import job (the per-aggregate view).
//! - [`StaffAttendanceImportRepository`] — 9 methods for
//!   the staff staging rows.
//! - [`StudentAttendanceImportRepository`] — 9 methods
//!   for the student staging rows.
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
    AttendanceBulk, BulkAttendanceImport, ClassAttendance, ExamAttendance, StaffAttendance,
    StudentAttendance, SubjectAttendance,
};
use crate::entities::{StaffAttendanceImport, StudentAttendanceImport};
use crate::value_objects::{
    AcademicYearId, AttendanceBulkId, AttendanceSource, BulkAttendanceImportId, ClassId,
    ExamAttendanceId, ExamTypeId, ImportStatus, SectionId, StaffAttendanceId,
    StaffAttendanceImportId, StaffId, StudentAttendanceId, StudentAttendanceImportId, StudentId,
    SubjectAttendanceId,
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

    /// Inserts a single [`AttendanceBulk`] denormalized
    /// staging row (per the spec, `attendance_bulks` is a
    /// sibling of `student_attendance_imports`).
    async fn insert_bulk_row(&self, ctx: &TenantContext, row: &AttendanceBulk) -> Result<()>;

    /// Returns every [`AttendanceBulk`] row belonging to
    /// the given bulk import job.
    async fn list_bulk_rows(
        &self,
        ctx: &TenantContext,
        bulk_id: BulkAttendanceImportId,
    ) -> Result<Vec<AttendanceBulk>>;
}

// =============================================================================
// ClassAttendanceRepository (spec repositories.md:149-180 — projection)
// =============================================================================

/// The storage-port contract for the [`ClassAttendance`]
/// projection. The engine recomputes a `ClassAttendance`
/// row on demand from `StudentAttendanceMarked` and
/// `ExamAttendanceMarked` events; the `upsert` method is
/// invoked by `AttendanceService::recompute_class_attendance`.
#[allow(dead_code)]
#[async_trait]
pub trait ClassAttendanceRepository: Send + Sync {
    /// Returns the [`ClassAttendance`] row for the unique
    /// key `(school_id, student_id, exam_type_id,
    /// academic_year_id)` (or `Ok(None)` if not yet
    /// materialised).
    async fn get(
        &self,
        school: SchoolId,
        student: StudentId,
        exam_type: ExamTypeId,
        year: AcademicYearId,
    ) -> Result<Option<ClassAttendance>>;

    /// Returns every [`ClassAttendance`] row for the given
    /// `(school, student, year)`. Stable ordering by
    /// `exam_type_id`.
    async fn list_for_student(
        &self,
        school: SchoolId,
        student: StudentId,
        year: AcademicYearId,
    ) -> Result<Vec<ClassAttendance>>;

    /// Returns every [`ClassAttendance`] row for the given
    /// `(school, exam_type, year)`. Stable ordering by
    /// `student_id`.
    async fn list_for_exam_type(
        &self,
        school: SchoolId,
        exam_type: ExamTypeId,
        year: AcademicYearId,
    ) -> Result<Vec<ClassAttendance>>;

    /// Inserts or updates the [`ClassAttendance`] row for
    /// the `(school_id, student_id, exam_type_id,
    /// academic_year_id)` key.
    async fn upsert(&self, ctx: &TenantContext, c: &ClassAttendance) -> Result<()>;
}

// =============================================================================
// 7. AttendanceBulkRepository
// =============================================================================

/// The storage-port contract for
/// [`AttendanceBulk`](crate::aggregate::AttendanceBulk)
/// rows. Materialized during a bulk import; on commit,
/// each row promotes into a
/// [`StudentAttendance`](crate::aggregate::StudentAttendance).
/// The kitchen-sink [`AttendanceImportRepository`]
/// exposes the same surface for the import workflow;
/// this trait is the per-aggregate, storage-adapter
/// view.
#[allow(dead_code)]
#[async_trait]
pub trait AttendanceBulkRepository: Send + Sync {
    /// Returns the [`AttendanceBulk`] with the given id
    /// (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: AttendanceBulkId,
    ) -> Result<Option<AttendanceBulk>>;

    /// Returns the [`AttendanceBulk`] for the unique key
    /// `(school_id, student_id, attendance_date)` (or
    /// `Ok(None)` if not found).
    async fn find(
        &self,
        school: SchoolId,
        student: StudentId,
        date: NaiveDate,
    ) -> Result<Option<AttendanceBulk>>;

    /// Returns every [`AttendanceBulk`] for the given
    /// bulk import job. Stable ordering by
    /// `attendance_date` then `student_id`.
    async fn list_for_bulk(
        &self,
        ctx: &TenantContext,
        bulk_id: BulkAttendanceImportId,
    ) -> Result<Vec<AttendanceBulk>>;

    /// Returns every [`AttendanceBulk`] for the given
    /// `(school, student, from, to)`. Stable ordering by
    /// `attendance_date` ascending.
    async fn list_for_student(
        &self,
        school: SchoolId,
        student: StudentId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<AttendanceBulk>>;

    /// Returns every [`AttendanceBulk`] for the given
    /// `(school, class, section, date)`.
    async fn list_for_section(
        &self,
        school: SchoolId,
        class: ClassId,
        section: SectionId,
        date: NaiveDate,
    ) -> Result<Vec<AttendanceBulk>>;

    /// Inserts a new [`AttendanceBulk`] row.
    async fn insert(&self, ctx: &TenantContext, row: &AttendanceBulk) -> Result<()>;

    /// Updates an existing [`AttendanceBulk`] row. MUST
    /// enforce the `version` optimistic-concurrency check.
    async fn update(&self, ctx: &TenantContext, row: &AttendanceBulk) -> Result<()>;

    /// Bulk-inserts N [`AttendanceBulk`] rows in a single
    /// transaction. The adapter rejects rows whose
    /// `school_id` does not match `ctx.school_id`.
    async fn bulk_insert(&self, ctx: &TenantContext, rows: &[AttendanceBulk]) -> Result<()>;
}

// =============================================================================
// 8. BulkAttendanceImportRepository
// =============================================================================

/// The storage-port contract for
/// [`BulkAttendanceImport`](crate::aggregate::BulkAttendanceImport)
/// jobs (the per-aggregate root view, separate from the
/// kitchen-sink [`AttendanceImportRepository`]).
#[allow(dead_code)]
#[async_trait]
pub trait BulkAttendanceImportRepository: Send + Sync {
    /// Returns the [`BulkAttendanceImport`] with the
    /// given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: BulkAttendanceImportId,
    ) -> Result<Option<BulkAttendanceImport>>;

    /// Returns the [`BulkAttendanceImport`] for the
    /// unique key `(school_id, source, attendance_date)`
    /// (or `Ok(None)` if not found). Used by the
    /// `import_source_date_exists` uniqueness check (spec
    /// invariant 3).
    async fn find(
        &self,
        school: SchoolId,
        source: AttendanceSource,
        date: NaiveDate,
    ) -> Result<Option<BulkAttendanceImport>>;

    /// Returns every [`BulkAttendanceImport`] for the
    /// given `(school, academic_year)`. Stable ordering
    /// by `marked_at` descending.
    async fn list_for_year(
        &self,
        school: SchoolId,
        year: AcademicYearId,
    ) -> Result<Vec<BulkAttendanceImport>>;

    /// Returns every [`BulkAttendanceImport`] for the
    /// given `(school, status)`. Stable ordering by
    /// `marked_at` descending.
    async fn list_for_status(
        &self,
        school: SchoolId,
        status: ImportStatus,
    ) -> Result<Vec<BulkAttendanceImport>>;

    /// Returns every [`BulkAttendanceImport`] for the
    /// given `(school, source, attendance_date)`. Used by
    /// the `import_source_date_exists` uniqueness check.
    async fn list_for_source_date(
        &self,
        school: SchoolId,
        source: AttendanceSource,
        date: NaiveDate,
    ) -> Result<Vec<BulkAttendanceImport>>;

    /// Returns every pending [`BulkAttendanceImport`] for
    /// the given school. Stable ordering by `marked_at`
    /// ascending.
    async fn list_pending(&self, school: SchoolId) -> Result<Vec<BulkAttendanceImport>>;

    /// Inserts a new [`BulkAttendanceImport`] job.
    async fn insert(&self, ctx: &TenantContext, j: &BulkAttendanceImport) -> Result<()>;

    /// Updates an existing [`BulkAttendanceImport`] job.
    /// MUST enforce the `version` optimistic-concurrency
    /// check.
    async fn update(&self, ctx: &TenantContext, j: &BulkAttendanceImport) -> Result<()>;
}

// =============================================================================
// 9. StaffAttendanceImportRepository
// =============================================================================

/// The storage-port contract for
/// [`StaffAttendanceImport`](crate::entities::StaffAttendanceImport)
/// staging rows (the per-aggregate view, separate from
/// the kitchen-sink [`AttendanceImportRepository`]).
#[allow(dead_code)]
#[async_trait]
pub trait StaffAttendanceImportRepository: Send + Sync {
    /// Returns the [`StaffAttendanceImport`] with the
    /// given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: StaffAttendanceImportId,
    ) -> Result<Option<StaffAttendanceImport>>;

    /// Returns the [`StaffAttendanceImport`] for the
    /// unique key `(school_id, staff_id, attendance_date)`
    /// (or `Ok(None)` if not found).
    async fn find(
        &self,
        school: SchoolId,
        staff: StaffId,
        date: NaiveDate,
    ) -> Result<Option<StaffAttendanceImport>>;

    /// Returns every [`StaffAttendanceImport`] for the
    /// given bulk import job. Stable ordering by
    /// `attendance_date` then `staff_id`.
    async fn list_for_bulk(
        &self,
        ctx: &TenantContext,
        bulk_id: BulkAttendanceImportId,
    ) -> Result<Vec<StaffAttendanceImport>>;

    /// Returns every [`StaffAttendanceImport`] for the
    /// given `(school, staff, from, to)`. Stable ordering
    /// by `attendance_date` ascending.
    async fn list_for_staff(
        &self,
        school: SchoolId,
        staff: StaffId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<StaffAttendanceImport>>;

    /// Returns every [`StaffAttendanceImport`] for the
    /// given bulk import job, filtered by the
    /// `is_validated` flag.
    async fn list_validated(
        &self,
        ctx: &TenantContext,
        bulk_id: BulkAttendanceImportId,
        is_validated: bool,
    ) -> Result<Vec<StaffAttendanceImport>>;

    /// Inserts a new [`StaffAttendanceImport`] staging
    /// row.
    async fn insert(&self, ctx: &TenantContext, row: &StaffAttendanceImport) -> Result<()>;

    /// Bulk-inserts N [`StaffAttendanceImport`] staging
    /// rows in a single transaction. The adapter rejects
    /// rows whose `school_id` does not match
    /// `ctx.school_id`.
    async fn insert_batch(&self, ctx: &TenantContext, rows: &[StaffAttendanceImport])
        -> Result<()>;

    /// Updates an existing [`StaffAttendanceImport`]
    /// staging row.
    async fn update(&self, ctx: &TenantContext, row: &StaffAttendanceImport) -> Result<()>;

    /// Marks the given staging rows as validated (or
    /// unvalidated, with the supplied `is_validated`
    /// flag).
    async fn mark_validated(
        &self,
        ctx: &TenantContext,
        row_ids: &[StaffAttendanceImportId],
        is_validated: bool,
    ) -> Result<()>;
}

// =============================================================================
// 10. StudentAttendanceImportRepository
// =============================================================================

/// The storage-port contract for
/// [`StudentAttendanceImport`](crate::entities::StudentAttendanceImport)
/// staging rows (the per-aggregate view, separate from
/// the kitchen-sink [`AttendanceImportRepository`]).
#[allow(dead_code)]
#[async_trait]
pub trait StudentAttendanceImportRepository: Send + Sync {
    /// Returns the [`StudentAttendanceImport`] with the
    /// given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: StudentAttendanceImportId,
    ) -> Result<Option<StudentAttendanceImport>>;

    /// Returns the [`StudentAttendanceImport`] for the
    /// unique key `(school_id, student_id,
    /// attendance_date)` (or `Ok(None)` if not found).
    async fn find(
        &self,
        school: SchoolId,
        student: StudentId,
        date: NaiveDate,
    ) -> Result<Option<StudentAttendanceImport>>;

    /// Returns every [`StudentAttendanceImport`] for the
    /// given bulk import job. Stable ordering by
    /// `attendance_date` then `student_id`.
    async fn list_for_bulk(
        &self,
        ctx: &TenantContext,
        bulk_id: BulkAttendanceImportId,
    ) -> Result<Vec<StudentAttendanceImport>>;

    /// Returns every [`StudentAttendanceImport`] for the
    /// given `(school, student, from, to)`. Stable
    /// ordering by `attendance_date` ascending.
    async fn list_for_student(
        &self,
        school: SchoolId,
        student: StudentId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<StudentAttendanceImport>>;

    /// Returns every [`StudentAttendanceImport`] for the
    /// given bulk import job, filtered by the
    /// `is_validated` flag.
    async fn list_validated(
        &self,
        ctx: &TenantContext,
        bulk_id: BulkAttendanceImportId,
        is_validated: bool,
    ) -> Result<Vec<StudentAttendanceImport>>;

    /// Inserts a new [`StudentAttendanceImport`] staging
    /// row.
    async fn insert(&self, ctx: &TenantContext, row: &StudentAttendanceImport) -> Result<()>;

    /// Bulk-inserts N [`StudentAttendanceImport`] staging
    /// rows in a single transaction. The adapter rejects
    /// rows whose `school_id` does not match
    /// `ctx.school_id`.
    async fn insert_batch(
        &self,
        ctx: &TenantContext,
        rows: &[StudentAttendanceImport],
    ) -> Result<()>;

    /// Updates an existing [`StudentAttendanceImport`]
    /// staging row.
    async fn update(&self, ctx: &TenantContext, row: &StudentAttendanceImport) -> Result<()>;

    /// Marks the given staging rows as validated (or
    /// unvalidated, with the supplied `is_validated`
    /// flag).
    async fn mark_validated(
        &self,
        ctx: &TenantContext,
        row_ids: &[StudentAttendanceImportId],
        is_validated: bool,
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
        fn _class_attendance_repository(_: Box<dyn ClassAttendanceRepository>) {}
        fn _attendance_bulk_repository(_: Box<dyn AttendanceBulkRepository>) {}
        fn _bulk_attendance_import_repository(_: Box<dyn BulkAttendanceImportRepository>) {}
        fn _staff_attendance_import_repository(_: Box<dyn StaffAttendanceImportRepository>) {}
        fn _student_attendance_import_repository(_: Box<dyn StudentAttendanceImportRepository>) {}
    }

    /// Happy-path: `ClassAttendanceId::new` round-trips
    /// the school anchor through the typed id (the wiring
    /// test that proves the new `ClassAttendanceRepository`
    /// trait compiles against `ClassAttendance`).
    #[test]
    fn class_attendance_repository_id_round_trip() {
        use crate::value_objects::ClassAttendanceId;
        use educore_core::clock::{IdGenerator, SystemIdGen};
        let gen = SystemIdGen;
        let school = gen.next_school_id();
        let id = ClassAttendanceId::new(school, gen.next_uuid());
        assert_eq!(id.school_id(), school);
    }
}
