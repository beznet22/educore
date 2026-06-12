//! # Assessment repository ports
//!
//! The Phase 4 Workstream A `ExamRepository` port (8
//! methods). The other 7 repository port traits
//! (ExamSchedule, MarksRegister, ResultStore, OnlineExam,
//! SeatPlan, AdmitCard, plus ExamScheduleSubject and
//! MarksRegisterChild children) land in Workstreams B, C,
//! and D.
//!
//! All port traits are `#[async_trait] pub trait
//! XxxRepository: Send + Sync` per the academic crate's
//! pattern. The storage adapters (Phase 1) provide the
//! concrete implementations.

use async_trait::async_trait;

use educore_academic::value_objects::AcademicYearId;
use educore_core::error::Result;
use educore_core::ids::SchoolId;
use educore_core::tenant::TenantContext;

use crate::aggregate::Exam;
use crate::value_objects::{ExamId, ExamTypeId};

// =============================================================================
// ExamRepository
// =============================================================================

/// The storage-port contract for [`Exam`](crate::aggregate::Exam)
/// rows. Every method is tenant-scoped: the implementation
/// MUST filter on `ctx.school_id` (or reject commands that
/// do not match).
#[allow(dead_code)]
// The methods are not called from within the assessment crate;
// the storage adapters in the engine's `crates/adapters/`
// tree will implement them in a later phase.
#[async_trait]
pub trait ExamRepository: Send + Sync {
    /// Returns the [`Exam`] with the given id (or `Ok(None)`
    /// if not found).
    async fn get(&self, ctx: &TenantContext, id: ExamId) -> Result<Option<Exam>>;

    /// Returns the [`Exam`] for the unique key
    /// `(school_id, exam_type_id, class_id, section_id,
    /// subject_id, academic_year_id)` (or `Ok(None)` if not
    /// found). Used by the dispatcher to enforce the
    /// uniqueness invariant after the
    /// [`AssessmentUniquenessChecker`](crate::commands::AssessmentUniquenessChecker)
    /// port reports a miss.
    #[allow(clippy::too_many_arguments)]
    async fn find(
        &self,
        school: SchoolId,
        exam_type: ExamTypeId,
        academic_year: AcademicYearId,
        class: educore_academic::ClassId,
        section: educore_academic::SectionId,
        subject: educore_academic::SubjectId,
    ) -> Result<Option<Exam>>;

    /// Returns every [`Exam`] for the given `(school,
    /// academic_year)`. Implementation MUST apply a stable
    /// ordering (e.g. `ORDER BY exam_date ASC, code ASC`).
    async fn list_for_year(&self, school: SchoolId, year: AcademicYearId) -> Result<Vec<Exam>>;

    /// Returns every [`Exam`] for the given `(school, class,
    /// academic_year)`.
    async fn list_for_class(
        &self,
        school: SchoolId,
        class: educore_academic::ClassId,
        year: AcademicYearId,
    ) -> Result<Vec<Exam>>;

    /// Returns every [`Exam`] for the given `(school,
    /// exam_type, academic_year)`.
    async fn list_for_type(
        &self,
        school: SchoolId,
        exam_type: ExamTypeId,
        year: AcademicYearId,
    ) -> Result<Vec<Exam>>;

    /// Inserts a new [`Exam`] row. The implementation MUST
    /// reject the insert with a `Conflict` error if the
    /// unique key already exists.
    async fn insert(&self, ctx: &TenantContext, exam: &Exam) -> Result<()>;

    /// Updates an existing [`Exam`] row. The implementation
    /// MUST enforce the `version` optimistic-concurrency
    /// check (return `Conflict` on mismatch).
    async fn update(&self, ctx: &TenantContext, exam: &Exam) -> Result<()>;

    /// Soft-deletes an [`Exam`] row by id. The implementation
    /// MUST reject the delete with a `Conflict` error if any
    /// `MarksRegister` row references the exam (per the
    /// spec's invariant #3).
    async fn delete(&self, ctx: &TenantContext, id: ExamId) -> Result<()>;
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

    /// Compile-time check: confirm the repository trait is
    /// object-safe by naming `Box<dyn ...>` for it.
    #[test]
    fn trait_is_object_safe() {
        fn _exam_repository(_: Box<dyn ExamRepository>) {}
    }
}
