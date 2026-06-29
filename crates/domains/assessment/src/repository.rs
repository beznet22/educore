//! # Assessment repository ports
//!
//! Twenty-six repository port traits in total: the six
//! Phase 4 Workstream A traits (Exam, ExamSchedule,
//! SeatPlan, AdmitCard, MarksRegister, ResultStore) plus
//! the twenty Wave 9.6a traits (AdmitCardSetting,
//! AllExamWisePosition, CustomResultSetting,
//! CustomTemporaryResult, ExamAttendance,
//! ExamAttendanceChild, ExamRoutinePage,
//! ExamScheduleSubject, ExamSetting, ExamSetup,
//! ExamSignature, ExamStepSkip, ExamType,
//! ExamWisePosition, FrontendExamResult,
//! FrontendExamRoutine, FrontendResult, MarksGrade,
//! MarksRegisterChild, MarkStore).
//!
//! All port traits are `#[async_trait] pub trait
//! XxxRepository: Send + Sync` per the academic crate's
//! pattern. The storage adapters (Phase 1) provide the
//! concrete implementations.

#![allow(missing_docs)] // The async trait method signatures
                        // are described by their parameter
                        // names; suppressing this lint for the
                        // file is the pragmatic choice for the
                        // 26 repo traits this file ships.

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

/// The storage-port contract for [`ExamSchedule`](crate::aggregate::ExamSchedule)
/// rows.
#[allow(dead_code)]
#[async_trait]
pub trait ExamScheduleRepository: Send + Sync {
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::ExamScheduleId,
    ) -> Result<Option<crate::aggregate::ExamSchedule>>;
    async fn find(
        &self,
        school: SchoolId,
        exam: ExamId,
        class: educore_academic::ClassId,
        section: educore_academic::SectionId,
        year: AcademicYearId,
    ) -> Result<Option<crate::aggregate::ExamSchedule>>;
    async fn list_for_section(
        &self,
        school: SchoolId,
        class: educore_academic::ClassId,
        section: educore_academic::SectionId,
        year: AcademicYearId,
    ) -> Result<Vec<crate::aggregate::ExamSchedule>>;
    async fn insert(&self, ctx: &TenantContext, s: &crate::aggregate::ExamSchedule) -> Result<()>;
    async fn update(&self, ctx: &TenantContext, s: &crate::aggregate::ExamSchedule) -> Result<()>;
    async fn delete(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::ExamScheduleId,
    ) -> Result<()>;
}

/// The storage-port contract for [`SeatPlan`](crate::aggregate::SeatPlan) rows.
#[allow(dead_code)]
#[async_trait]
pub trait SeatPlanRepository: Send + Sync {
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::SeatPlanId,
    ) -> Result<Option<crate::aggregate::SeatPlan>>;
    async fn list_for_exam(
        &self,
        school: SchoolId,
        exam: ExamId,
    ) -> Result<Vec<crate::aggregate::SeatPlan>>;
    async fn insert(&self, ctx: &TenantContext, p: &crate::aggregate::SeatPlan) -> Result<()>;
    async fn update(&self, ctx: &TenantContext, p: &crate::aggregate::SeatPlan) -> Result<()>;
    async fn delete(&self, ctx: &TenantContext, id: crate::value_objects::SeatPlanId)
        -> Result<()>;
}

/// The storage-port contract for [`AdmitCard`](crate::aggregate::AdmitCard) rows.
#[allow(dead_code)]
#[async_trait]
pub trait AdmitCardRepository: Send + Sync {
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::AdmitCardId,
    ) -> Result<Option<crate::aggregate::AdmitCard>>;
    async fn find(
        &self,
        school: SchoolId,
        student_record: crate::value_objects::StudentRecordId,
        exam_type: ExamTypeId,
        year: AcademicYearId,
    ) -> Result<Option<crate::aggregate::AdmitCard>>;
    async fn insert(&self, ctx: &TenantContext, c: &crate::aggregate::AdmitCard) -> Result<()>;
    async fn update(&self, ctx: &TenantContext, c: &crate::aggregate::AdmitCard) -> Result<()>;
    async fn delete(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::AdmitCardId,
    ) -> Result<()>;
}

/// The storage-port contract for [`MarksRegister`](crate::aggregate::MarksRegister) rows.
#[allow(dead_code)]
#[async_trait]
pub trait MarksRegisterRepository: Send + Sync {
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::MarksRegisterId,
    ) -> Result<Option<crate::aggregate::MarksRegister>>;
    async fn find(
        &self,
        school: SchoolId,
        exam: ExamId,
        student: crate::value_objects::StudentId,
    ) -> Result<Option<crate::aggregate::MarksRegister>>;
    async fn list_for_exam(
        &self,
        school: SchoolId,
        exam: ExamId,
    ) -> Result<Vec<crate::aggregate::MarksRegister>>;
    async fn insert(&self, ctx: &TenantContext, m: &crate::aggregate::MarksRegister) -> Result<()>;
    async fn update(&self, ctx: &TenantContext, m: &crate::aggregate::MarksRegister) -> Result<()>;
}

/// The storage-port contract for [`ResultStore`](crate::aggregate::ResultStore) rows.
#[allow(dead_code)]
#[async_trait]
pub trait ResultRepository: Send + Sync {
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::ResultStoreId,
    ) -> Result<Option<crate::aggregate::ResultStore>>;
    async fn list_for_student(
        &self,
        school: SchoolId,
        student: crate::value_objects::StudentId,
    ) -> Result<Vec<crate::aggregate::ResultStore>>;
    async fn list_for_exam(
        &self,
        school: SchoolId,
        exam: ExamId,
    ) -> Result<Vec<crate::aggregate::ResultStore>>;
    async fn insert(&self, ctx: &TenantContext, r: &crate::aggregate::ResultStore) -> Result<()>;
    async fn update(&self, ctx: &TenantContext, r: &crate::aggregate::ResultStore) -> Result<()>;
}

// =============================================================================
// Wave 9.6a: additional repository port traits (20)
// =============================================================================

/// The storage-port contract for [`AdmitCardSetting`](crate::aggregate::AdmitCardSetting) rows.
#[allow(dead_code)]
#[async_trait]
pub trait AdmitCardSettingRepository: Send + Sync {
    /// Returns the [`AdmitCardSetting`](crate::aggregate::AdmitCardSetting) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::AdmitCardSettingId,
    ) -> Result<Option<crate::aggregate::AdmitCardSetting>>;
    /// Inserts a new [`AdmitCardSetting`](crate::aggregate::AdmitCardSetting) row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::AdmitCardSetting,
    ) -> Result<()>;
    /// Updates an existing [`AdmitCardSetting`](crate::aggregate::AdmitCardSetting) row.
    async fn update(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::AdmitCardSetting,
    ) -> Result<()>;
}

/// Object-safety smoke test for [`AdmitCardSettingRepository`].
fn _assert_admit_card_setting_repository_object_safe(_: Box<dyn AdmitCardSettingRepository>) {}

/// The storage-port contract for [`AllExamWisePosition`](crate::aggregate::AllExamWisePosition) rows.
#[allow(dead_code)]
#[async_trait]
pub trait AllExamWisePositionRepository: Send + Sync {
    /// Returns the [`AllExamWisePosition`](crate::aggregate::AllExamWisePosition) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::AllExamWisePositionId,
    ) -> Result<Option<crate::aggregate::AllExamWisePosition>>;
    /// Inserts a new [`AllExamWisePosition`](crate::aggregate::AllExamWisePosition) row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::AllExamWisePosition,
    ) -> Result<()>;
    /// Updates an existing [`AllExamWisePosition`](crate::aggregate::AllExamWisePosition) row.
    async fn update(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::AllExamWisePosition,
    ) -> Result<()>;
}

/// Object-safety smoke test for [`AllExamWisePositionRepository`].
fn _assert_all_exam_wise_position_repository_object_safe(
    _: Box<dyn AllExamWisePositionRepository>,
) {
}

/// The storage-port contract for [`CustomResultSetting`](crate::aggregate::CustomResultSetting) rows.
#[allow(dead_code)]
#[async_trait]
pub trait CustomResultSettingRepository: Send + Sync {
    /// Returns the [`CustomResultSetting`](crate::aggregate::CustomResultSetting) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::CustomResultSettingId,
    ) -> Result<Option<crate::aggregate::CustomResultSetting>>;
    /// Inserts a new [`CustomResultSetting`](crate::aggregate::CustomResultSetting) row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::CustomResultSetting,
    ) -> Result<()>;
    /// Updates an existing [`CustomResultSetting`](crate::aggregate::CustomResultSetting) row.
    async fn update(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::CustomResultSetting,
    ) -> Result<()>;
}

/// Object-safety smoke test for [`CustomResultSettingRepository`].
fn _assert_custom_result_setting_repository_object_safe(_: Box<dyn CustomResultSettingRepository>) {
}

/// The storage-port contract for [`CustomTemporaryResult`](crate::aggregate::CustomTemporaryResult) rows.
#[allow(dead_code)]
#[async_trait]
pub trait CustomTemporaryResultRepository: Send + Sync {
    /// Returns the [`CustomTemporaryResult`](crate::aggregate::CustomTemporaryResult) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::CustomTemporaryResultId,
    ) -> Result<Option<crate::aggregate::CustomTemporaryResult>>;
    /// Inserts a new [`CustomTemporaryResult`](crate::aggregate::CustomTemporaryResult) row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::CustomTemporaryResult,
    ) -> Result<()>;
    /// Updates an existing [`CustomTemporaryResult`](crate::aggregate::CustomTemporaryResult) row.
    async fn update(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::CustomTemporaryResult,
    ) -> Result<()>;
}

/// Object-safety smoke test for [`CustomTemporaryResultRepository`].
fn _assert_custom_temporary_result_repository_object_safe(
    _: Box<dyn CustomTemporaryResultRepository>,
) {
}

/// The storage-port contract for [`ExamAttendance`](crate::aggregate::ExamAttendance) rows.
#[allow(dead_code)]
#[async_trait]
pub trait ExamAttendanceRepository: Send + Sync {
    /// Returns the [`ExamAttendance`](crate::aggregate::ExamAttendance) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::ExamAttendanceId,
    ) -> Result<Option<crate::aggregate::ExamAttendance>>;
    /// Inserts a new [`ExamAttendance`](crate::aggregate::ExamAttendance) row.
    async fn insert(&self, ctx: &TenantContext, s: &crate::aggregate::ExamAttendance)
        -> Result<()>;
    /// Updates an existing [`ExamAttendance`](crate::aggregate::ExamAttendance) row.
    async fn update(&self, ctx: &TenantContext, s: &crate::aggregate::ExamAttendance)
        -> Result<()>;
}

/// Object-safety smoke test for [`ExamAttendanceRepository`].
fn _assert_exam_attendance_repository_object_safe(_: Box<dyn ExamAttendanceRepository>) {}

/// The storage-port contract for [`ExamAttendanceChild`](crate::aggregate::ExamAttendanceChild) rows.
#[allow(dead_code)]
#[async_trait]
pub trait ExamAttendanceChildRepository: Send + Sync {
    /// Returns the [`ExamAttendanceChild`](crate::aggregate::ExamAttendanceChild) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::ExamAttendanceChildId,
    ) -> Result<Option<crate::aggregate::ExamAttendanceChild>>;
    /// Inserts a new [`ExamAttendanceChild`](crate::aggregate::ExamAttendanceChild) row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::ExamAttendanceChild,
    ) -> Result<()>;
    /// Updates an existing [`ExamAttendanceChild`](crate::aggregate::ExamAttendanceChild) row.
    async fn update(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::ExamAttendanceChild,
    ) -> Result<()>;
}

/// Object-safety smoke test for [`ExamAttendanceChildRepository`].
fn _assert_exam_attendance_child_repository_object_safe(_: Box<dyn ExamAttendanceChildRepository>) {
}

/// The storage-port contract for [`ExamRoutinePage`](crate::aggregate::ExamRoutinePage) rows.
#[allow(dead_code)]
#[async_trait]
pub trait ExamRoutinePageRepository: Send + Sync {
    /// Returns the [`ExamRoutinePage`](crate::aggregate::ExamRoutinePage) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::ExamRoutinePageId,
    ) -> Result<Option<crate::aggregate::ExamRoutinePage>>;
    /// Inserts a new [`ExamRoutinePage`](crate::aggregate::ExamRoutinePage) row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::ExamRoutinePage,
    ) -> Result<()>;
    /// Updates an existing [`ExamRoutinePage`](crate::aggregate::ExamRoutinePage) row.
    async fn update(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::ExamRoutinePage,
    ) -> Result<()>;
}

/// Object-safety smoke test for [`ExamRoutinePageRepository`].
fn _assert_exam_routine_page_repository_object_safe(_: Box<dyn ExamRoutinePageRepository>) {}

/// The storage-port contract for [`ExamScheduleSubject`](crate::aggregate::ExamScheduleSubject) rows.
#[allow(dead_code)]
#[async_trait]
pub trait ExamScheduleSubjectRepository: Send + Sync {
    /// Returns the [`ExamScheduleSubject`](crate::aggregate::ExamScheduleSubject) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::ExamScheduleSubjectId,
    ) -> Result<Option<crate::aggregate::ExamScheduleSubject>>;
    /// Inserts a new [`ExamScheduleSubject`](crate::aggregate::ExamScheduleSubject) row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::ExamScheduleSubject,
    ) -> Result<()>;
    /// Updates an existing [`ExamScheduleSubject`](crate::aggregate::ExamScheduleSubject) row.
    async fn update(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::ExamScheduleSubject,
    ) -> Result<()>;
}

/// Object-safety smoke test for [`ExamScheduleSubjectRepository`].
fn _assert_exam_schedule_subject_repository_object_safe(_: Box<dyn ExamScheduleSubjectRepository>) {
}

/// The storage-port contract for [`ExamSetting`](crate::aggregate::ExamSetting) rows.
#[allow(dead_code)]
#[async_trait]
pub trait ExamSettingRepository: Send + Sync {
    /// Returns the [`ExamSetting`](crate::aggregate::ExamSetting) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::ExamSettingId,
    ) -> Result<Option<crate::aggregate::ExamSetting>>;
    /// Inserts a new [`ExamSetting`](crate::aggregate::ExamSetting) row.
    async fn insert(&self, ctx: &TenantContext, s: &crate::aggregate::ExamSetting) -> Result<()>;
    /// Updates an existing [`ExamSetting`](crate::aggregate::ExamSetting) row.
    async fn update(&self, ctx: &TenantContext, s: &crate::aggregate::ExamSetting) -> Result<()>;
}

/// Object-safety smoke test for [`ExamSettingRepository`].
fn _assert_exam_setting_repository_object_safe(_: Box<dyn ExamSettingRepository>) {}

/// The storage-port contract for [`ExamSetup`](crate::aggregate::ExamSetup) rows.
#[allow(dead_code)]
#[async_trait]
pub trait ExamSetupRepository: Send + Sync {
    /// Returns the [`ExamSetup`](crate::aggregate::ExamSetup) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::ExamSetupId,
    ) -> Result<Option<crate::aggregate::ExamSetup>>;
    /// Inserts a new [`ExamSetup`](crate::aggregate::ExamSetup) row.
    async fn insert(&self, ctx: &TenantContext, s: &crate::aggregate::ExamSetup) -> Result<()>;
    /// Updates an existing [`ExamSetup`](crate::aggregate::ExamSetup) row.
    async fn update(&self, ctx: &TenantContext, s: &crate::aggregate::ExamSetup) -> Result<()>;
}

/// Object-safety smoke test for [`ExamSetupRepository`].
fn _assert_exam_setup_repository_object_safe(_: Box<dyn ExamSetupRepository>) {}

/// The storage-port contract for [`ExamSignature`](crate::aggregate::ExamSignature) rows.
#[allow(dead_code)]
#[async_trait]
pub trait ExamSignatureRepository: Send + Sync {
    /// Returns the [`ExamSignature`](crate::aggregate::ExamSignature) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::ExamSignatureId,
    ) -> Result<Option<crate::aggregate::ExamSignature>>;
    /// Inserts a new [`ExamSignature`](crate::aggregate::ExamSignature) row.
    async fn insert(&self, ctx: &TenantContext, s: &crate::aggregate::ExamSignature) -> Result<()>;
    /// Updates an existing [`ExamSignature`](crate::aggregate::ExamSignature) row.
    async fn update(&self, ctx: &TenantContext, s: &crate::aggregate::ExamSignature) -> Result<()>;
}

/// Object-safety smoke test for [`ExamSignatureRepository`].
fn _assert_exam_signature_repository_object_safe(_: Box<dyn ExamSignatureRepository>) {}

/// The storage-port contract for [`ExamStepSkip`](crate::aggregate::ExamStepSkip) rows.
#[allow(dead_code)]
#[async_trait]
pub trait ExamStepSkipRepository: Send + Sync {
    /// Returns the [`ExamStepSkip`](crate::aggregate::ExamStepSkip) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::ExamStepSkipId,
    ) -> Result<Option<crate::aggregate::ExamStepSkip>>;
    /// Inserts a new [`ExamStepSkip`](crate::aggregate::ExamStepSkip) row.
    async fn insert(&self, ctx: &TenantContext, s: &crate::aggregate::ExamStepSkip) -> Result<()>;
    /// Updates an existing [`ExamStepSkip`](crate::aggregate::ExamStepSkip) row.
    async fn update(&self, ctx: &TenantContext, s: &crate::aggregate::ExamStepSkip) -> Result<()>;
}

/// Object-safety smoke test for [`ExamStepSkipRepository`].
fn _assert_exam_step_skip_repository_object_safe(_: Box<dyn ExamStepSkipRepository>) {}

/// The storage-port contract for [`ExamType`](crate::aggregate::ExamType) rows.
#[allow(dead_code)]
#[async_trait]
pub trait ExamTypeRepository: Send + Sync {
    /// Returns the [`ExamType`](crate::aggregate::ExamType) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: ExamTypeId,
    ) -> Result<Option<crate::aggregate::ExamType>>;
    /// Inserts a new [`ExamType`](crate::aggregate::ExamType) row.
    async fn insert(&self, ctx: &TenantContext, s: &crate::aggregate::ExamType) -> Result<()>;
    /// Updates an existing [`ExamType`](crate::aggregate::ExamType) row.
    async fn update(&self, ctx: &TenantContext, s: &crate::aggregate::ExamType) -> Result<()>;
}

/// Object-safety smoke test for [`ExamTypeRepository`].
fn _assert_exam_type_repository_object_safe(_: Box<dyn ExamTypeRepository>) {}

/// The storage-port contract for [`ExamWisePosition`](crate::aggregate::ExamWisePosition) rows.
#[allow(dead_code)]
#[async_trait]
pub trait ExamWisePositionRepository: Send + Sync {
    /// Returns the [`ExamWisePosition`](crate::aggregate::ExamWisePosition) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::ExamWisePositionId,
    ) -> Result<Option<crate::aggregate::ExamWisePosition>>;
    /// Inserts a new [`ExamWisePosition`](crate::aggregate::ExamWisePosition) row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::ExamWisePosition,
    ) -> Result<()>;
    /// Updates an existing [`ExamWisePosition`](crate::aggregate::ExamWisePosition) row.
    async fn update(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::ExamWisePosition,
    ) -> Result<()>;
}

/// Object-safety smoke test for [`ExamWisePositionRepository`].
fn _assert_exam_wise_position_repository_object_safe(_: Box<dyn ExamWisePositionRepository>) {}

/// The storage-port contract for [`FrontendExamResult`](crate::aggregate::FrontendExamResult) rows.
#[allow(dead_code)]
#[async_trait]
pub trait FrontendExamResultRepository: Send + Sync {
    /// Returns the [`FrontendExamResult`](crate::aggregate::FrontendExamResult) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::FrontendExamResultId,
    ) -> Result<Option<crate::aggregate::FrontendExamResult>>;
    /// Inserts a new [`FrontendExamResult`](crate::aggregate::FrontendExamResult) row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::FrontendExamResult,
    ) -> Result<()>;
    /// Updates an existing [`FrontendExamResult`](crate::aggregate::FrontendExamResult) row.
    async fn update(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::FrontendExamResult,
    ) -> Result<()>;
}

/// Object-safety smoke test for [`FrontendExamResultRepository`].
fn _assert_frontend_exam_result_repository_object_safe(_: Box<dyn FrontendExamResultRepository>) {}

/// The storage-port contract for [`FrontendExamRoutine`](crate::aggregate::FrontendExamRoutine) rows.
#[allow(dead_code)]
#[async_trait]
pub trait FrontendExamRoutineRepository: Send + Sync {
    /// Returns the [`FrontendExamRoutine`](crate::aggregate::FrontendExamRoutine) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::FrontExamRoutineId,
    ) -> Result<Option<crate::aggregate::FrontendExamRoutine>>;
    /// Inserts a new [`FrontendExamRoutine`](crate::aggregate::FrontendExamRoutine) row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::FrontendExamRoutine,
    ) -> Result<()>;
    /// Updates an existing [`FrontendExamRoutine`](crate::aggregate::FrontendExamRoutine) row.
    async fn update(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::FrontendExamRoutine,
    ) -> Result<()>;
}

/// Object-safety smoke test for [`FrontendExamRoutineRepository`].
fn _assert_frontend_exam_routine_repository_object_safe(_: Box<dyn FrontendExamRoutineRepository>) {
}

/// The storage-port contract for [`FrontendResult`](crate::aggregate::FrontendResult) rows.
#[allow(dead_code)]
#[async_trait]
pub trait FrontendResultRepository: Send + Sync {
    /// Returns the [`FrontendResult`](crate::aggregate::FrontendResult) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::FrontResultId,
    ) -> Result<Option<crate::aggregate::FrontendResult>>;
    /// Inserts a new [`FrontendResult`](crate::aggregate::FrontendResult) row.
    async fn insert(&self, ctx: &TenantContext, s: &crate::aggregate::FrontendResult)
        -> Result<()>;
    /// Updates an existing [`FrontendResult`](crate::aggregate::FrontendResult) row.
    async fn update(&self, ctx: &TenantContext, s: &crate::aggregate::FrontendResult)
        -> Result<()>;
}

/// Object-safety smoke test for [`FrontendResultRepository`].
fn _assert_frontend_result_repository_object_safe(_: Box<dyn FrontendResultRepository>) {}

/// The storage-port contract for [`MarksGrade`](crate::aggregate::MarksGrade) rows.
#[allow(dead_code)]
#[async_trait]
pub trait MarksGradeRepository: Send + Sync {
    /// Returns the [`MarksGrade`](crate::aggregate::MarksGrade) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::MarksGradeId,
    ) -> Result<Option<crate::aggregate::MarksGrade>>;
    /// Inserts a new [`MarksGrade`](crate::aggregate::MarksGrade) row.
    async fn insert(&self, ctx: &TenantContext, s: &crate::aggregate::MarksGrade) -> Result<()>;
    /// Updates an existing [`MarksGrade`](crate::aggregate::MarksGrade) row.
    async fn update(&self, ctx: &TenantContext, s: &crate::aggregate::MarksGrade) -> Result<()>;
}

/// Object-safety smoke test for [`MarksGradeRepository`].
fn _assert_marks_grade_repository_object_safe(_: Box<dyn MarksGradeRepository>) {}

/// The storage-port contract for [`MarksRegisterChild`](crate::aggregate::MarksRegisterChild) rows.
#[allow(dead_code)]
#[async_trait]
pub trait MarksRegisterChildRepository: Send + Sync {
    /// Returns the [`MarksRegisterChild`](crate::aggregate::MarksRegisterChild) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::MarksRegisterChildId,
    ) -> Result<Option<crate::aggregate::MarksRegisterChild>>;
    /// Inserts a new [`MarksRegisterChild`](crate::aggregate::MarksRegisterChild) row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::MarksRegisterChild,
    ) -> Result<()>;
    /// Updates an existing [`MarksRegisterChild`](crate::aggregate::MarksRegisterChild) row.
    async fn update(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::MarksRegisterChild,
    ) -> Result<()>;
}

/// Object-safety smoke test for [`MarksRegisterChildRepository`].
fn _assert_marks_register_child_repository_object_safe(_: Box<dyn MarksRegisterChildRepository>) {}

/// The storage-port contract for [`MarkStore`](crate::aggregate::MarkStore) rows.
#[allow(dead_code)]
#[async_trait]
pub trait MarkStoreRepository: Send + Sync {
    /// Returns the [`MarkStore`](crate::aggregate::MarkStore) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::MarkStoreId,
    ) -> Result<Option<crate::aggregate::MarkStore>>;
    /// Inserts a new [`MarkStore`](crate::aggregate::MarkStore) row.
    async fn insert(&self, ctx: &TenantContext, s: &crate::aggregate::MarkStore) -> Result<()>;
    /// Updates an existing [`MarkStore`](crate::aggregate::MarkStore) row.
    async fn update(&self, ctx: &TenantContext, s: &crate::aggregate::MarkStore) -> Result<()>;
}

/// Object-safety smoke test for [`MarkStoreRepository`].
fn _assert_mark_store_repository_object_safe(_: Box<dyn MarkStoreRepository>) {}

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
