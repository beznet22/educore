//! # Assessment repository ports
//!
//! Forty-five repository port traits in total: the six
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
//! MarksRegisterChild, MarkStore) plus the nineteen
//! Wave 9.6b traits (MarkStoreEntry, MeritPosition,
//! OnlineExam, OnlineExamMark, OnlineExamQuestion,
//! OnlineExamStudentAnswerMarking, QuestionAssignment,
//! QuestionBank, QuestionGroup, QuestionLevel,
//! QuestionMuOption, ResultSetting, ResultStore,
//! SeatPlanChild, SeatPlanSetting, StudentTakeOnlineExam,
//! TeacherEvaluation, TeacherRemark, TemporaryMeritList).
//!
//! All port traits are `#[async_trait] pub trait
//! XxxRepository: Send + Sync` per the academic crate's
//! pattern. The storage adapters (Phase 1) provide the
//! concrete implementations.

#![allow(missing_docs)] // The async trait method signatures
                        // are described by their parameter
                        // names; suppressing this lint for the
                        // file is the pragmatic choice for the
                        // 45 repo traits this file ships.

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

// =============================================================================
// Wave 9.6b: additional repository port traits (19)
// =============================================================================

/// The storage-port contract for [`MarkStoreEntry`](crate::aggregate::MarkStoreEntry) rows.
#[allow(dead_code)]
#[async_trait]
pub trait MarkStoreEntryRepository: Send + Sync {
    /// Returns the [`MarkStoreEntry`](crate::aggregate::MarkStoreEntry) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::MarkStoreEntryId,
    ) -> Result<Option<crate::aggregate::MarkStoreEntry>>;
    /// Inserts a new [`MarkStoreEntry`](crate::aggregate::MarkStoreEntry) row.
    async fn insert(&self, ctx: &TenantContext, s: &crate::aggregate::MarkStoreEntry)
        -> Result<()>;
    /// Updates an existing [`MarkStoreEntry`](crate::aggregate::MarkStoreEntry) row.
    async fn update(&self, ctx: &TenantContext, s: &crate::aggregate::MarkStoreEntry)
        -> Result<()>;
}

/// Object-safety smoke test for [`MarkStoreEntryRepository`].
fn _assert_mark_store_entry_repository_object_safe(_: Box<dyn MarkStoreEntryRepository>) {}

/// The storage-port contract for [`MeritPosition`](crate::aggregate::MeritPosition) rows.
#[allow(dead_code)]
#[async_trait]
pub trait MeritPositionRepository: Send + Sync {
    /// Returns the [`MeritPosition`](crate::aggregate::MeritPosition) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::MeritPositionId,
    ) -> Result<Option<crate::aggregate::MeritPosition>>;
    /// Inserts a new [`MeritPosition`](crate::aggregate::MeritPosition) row.
    async fn insert(&self, ctx: &TenantContext, s: &crate::aggregate::MeritPosition) -> Result<()>;
    /// Updates an existing [`MeritPosition`](crate::aggregate::MeritPosition) row.
    async fn update(&self, ctx: &TenantContext, s: &crate::aggregate::MeritPosition) -> Result<()>;
}

/// Object-safety smoke test for [`MeritPositionRepository`].
fn _assert_merit_position_repository_object_safe(_: Box<dyn MeritPositionRepository>) {}

/// The storage-port contract for [`OnlineExam`](crate::aggregate::OnlineExam) rows.
#[allow(dead_code)]
#[async_trait]
pub trait OnlineExamRepository: Send + Sync {
    /// Returns the [`OnlineExam`](crate::aggregate::OnlineExam) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::OnlineExamId,
    ) -> Result<Option<crate::aggregate::OnlineExam>>;
    /// Inserts a new [`OnlineExam`](crate::aggregate::OnlineExam) row.
    async fn insert(&self, ctx: &TenantContext, s: &crate::aggregate::OnlineExam) -> Result<()>;
    /// Updates an existing [`OnlineExam`](crate::aggregate::OnlineExam) row.
    async fn update(&self, ctx: &TenantContext, s: &crate::aggregate::OnlineExam) -> Result<()>;
}

/// Object-safety smoke test for [`OnlineExamRepository`].
fn _assert_online_exam_repository_object_safe(_: Box<dyn OnlineExamRepository>) {}

/// The storage-port contract for [`OnlineExamMark`](crate::aggregate::OnlineExamMark) rows.
#[allow(dead_code)]
#[async_trait]
pub trait OnlineExamMarkRepository: Send + Sync {
    /// Returns the [`OnlineExamMark`](crate::aggregate::OnlineExamMark) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::OnlineExamMarkId,
    ) -> Result<Option<crate::aggregate::OnlineExamMark>>;
    /// Inserts a new [`OnlineExamMark`](crate::aggregate::OnlineExamMark) row.
    async fn insert(&self, ctx: &TenantContext, s: &crate::aggregate::OnlineExamMark)
        -> Result<()>;
    /// Updates an existing [`OnlineExamMark`](crate::aggregate::OnlineExamMark) row.
    async fn update(&self, ctx: &TenantContext, s: &crate::aggregate::OnlineExamMark)
        -> Result<()>;
}

/// Object-safety smoke test for [`OnlineExamMarkRepository`].
fn _assert_online_exam_mark_repository_object_safe(_: Box<dyn OnlineExamMarkRepository>) {}

/// The storage-port contract for [`OnlineExamQuestion`](crate::aggregate::OnlineExamQuestion) rows.
#[allow(dead_code)]
#[async_trait]
pub trait OnlineExamQuestionRepository: Send + Sync {
    /// Returns the [`OnlineExamQuestion`](crate::aggregate::OnlineExamQuestion) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::OnlineExamQuestionId,
    ) -> Result<Option<crate::aggregate::OnlineExamQuestion>>;
    /// Inserts a new [`OnlineExamQuestion`](crate::aggregate::OnlineExamQuestion) row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::OnlineExamQuestion,
    ) -> Result<()>;
    /// Updates an existing [`OnlineExamQuestion`](crate::aggregate::OnlineExamQuestion) row.
    async fn update(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::OnlineExamQuestion,
    ) -> Result<()>;
}

/// Object-safety smoke test for [`OnlineExamQuestionRepository`].
fn _assert_online_exam_question_repository_object_safe(_: Box<dyn OnlineExamQuestionRepository>) {}

/// The storage-port contract for [`OnlineExamStudentAnswerMarking`](crate::aggregate::OnlineExamStudentAnswerMarking) rows.
#[allow(dead_code)]
#[async_trait]
pub trait OnlineExamStudentAnswerMarkingRepository: Send + Sync {
    /// Returns the [`OnlineExamStudentAnswerMarking`](crate::aggregate::OnlineExamStudentAnswerMarking) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::OnlineExamStudentAnswerMarkingId,
    ) -> Result<Option<crate::aggregate::OnlineExamStudentAnswerMarking>>;
    /// Inserts a new [`OnlineExamStudentAnswerMarking`](crate::aggregate::OnlineExamStudentAnswerMarking) row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::OnlineExamStudentAnswerMarking,
    ) -> Result<()>;
    /// Updates an existing [`OnlineExamStudentAnswerMarking`](crate::aggregate::OnlineExamStudentAnswerMarking) row.
    async fn update(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::OnlineExamStudentAnswerMarking,
    ) -> Result<()>;
}

/// Object-safety smoke test for [`OnlineExamStudentAnswerMarkingRepository`].
fn _assert_online_exam_student_answer_marking_repository_object_safe(
    _: Box<dyn OnlineExamStudentAnswerMarkingRepository>,
) {
}

/// The storage-port contract for [`QuestionAssignment`](crate::aggregate::QuestionAssignment) rows.
#[allow(dead_code)]
#[async_trait]
pub trait QuestionAssignmentRepository: Send + Sync {
    /// Returns the [`QuestionAssignment`](crate::aggregate::QuestionAssignment) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::QuestionAssignmentId,
    ) -> Result<Option<crate::aggregate::QuestionAssignment>>;
    /// Inserts a new [`QuestionAssignment`](crate::aggregate::QuestionAssignment) row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::QuestionAssignment,
    ) -> Result<()>;
    /// Updates an existing [`QuestionAssignment`](crate::aggregate::QuestionAssignment) row.
    async fn update(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::QuestionAssignment,
    ) -> Result<()>;
}

/// Object-safety smoke test for [`QuestionAssignmentRepository`].
fn _assert_question_assignment_repository_object_safe(_: Box<dyn QuestionAssignmentRepository>) {}

/// The storage-port contract for [`QuestionBank`](crate::aggregate::QuestionBank) rows.
#[allow(dead_code)]
#[async_trait]
pub trait QuestionBankRepository: Send + Sync {
    /// Returns the [`QuestionBank`](crate::aggregate::QuestionBank) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::QuestionBankId,
    ) -> Result<Option<crate::aggregate::QuestionBank>>;
    /// Inserts a new [`QuestionBank`](crate::aggregate::QuestionBank) row.
    async fn insert(&self, ctx: &TenantContext, s: &crate::aggregate::QuestionBank) -> Result<()>;
    /// Updates an existing [`QuestionBank`](crate::aggregate::QuestionBank) row.
    async fn update(&self, ctx: &TenantContext, s: &crate::aggregate::QuestionBank) -> Result<()>;
}

/// Object-safety smoke test for [`QuestionBankRepository`].
fn _assert_question_bank_repository_object_safe(_: Box<dyn QuestionBankRepository>) {}

/// The storage-port contract for [`QuestionGroup`](crate::aggregate::QuestionGroup) rows.
#[allow(dead_code)]
#[async_trait]
pub trait QuestionGroupRepository: Send + Sync {
    /// Returns the [`QuestionGroup`](crate::aggregate::QuestionGroup) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::QuestionGroupId,
    ) -> Result<Option<crate::aggregate::QuestionGroup>>;
    /// Inserts a new [`QuestionGroup`](crate::aggregate::QuestionGroup) row.
    async fn insert(&self, ctx: &TenantContext, s: &crate::aggregate::QuestionGroup) -> Result<()>;
    /// Updates an existing [`QuestionGroup`](crate::aggregate::QuestionGroup) row.
    async fn update(&self, ctx: &TenantContext, s: &crate::aggregate::QuestionGroup) -> Result<()>;
}

/// Object-safety smoke test for [`QuestionGroupRepository`].
fn _assert_question_group_repository_object_safe(_: Box<dyn QuestionGroupRepository>) {}

/// The storage-port contract for [`QuestionLevel`](crate::aggregate::QuestionLevel) rows.
#[allow(dead_code)]
#[async_trait]
pub trait QuestionLevelRepository: Send + Sync {
    /// Returns the [`QuestionLevel`](crate::aggregate::QuestionLevel) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::QuestionLevelId,
    ) -> Result<Option<crate::aggregate::QuestionLevel>>;
    /// Inserts a new [`QuestionLevel`](crate::aggregate::QuestionLevel) row.
    async fn insert(&self, ctx: &TenantContext, s: &crate::aggregate::QuestionLevel) -> Result<()>;
    /// Updates an existing [`QuestionLevel`](crate::aggregate::QuestionLevel) row.
    async fn update(&self, ctx: &TenantContext, s: &crate::aggregate::QuestionLevel) -> Result<()>;
}

/// Object-safety smoke test for [`QuestionLevelRepository`].
fn _assert_question_level_repository_object_safe(_: Box<dyn QuestionLevelRepository>) {}

/// The storage-port contract for [`QuestionMuOption`](crate::aggregate::QuestionMuOption) rows.
#[allow(dead_code)]
#[async_trait]
pub trait QuestionMuOptionRepository: Send + Sync {
    /// Returns the [`QuestionMuOption`](crate::aggregate::QuestionMuOption) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::QuestionMuOptionId,
    ) -> Result<Option<crate::aggregate::QuestionMuOption>>;
    /// Inserts a new [`QuestionMuOption`](crate::aggregate::QuestionMuOption) row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::QuestionMuOption,
    ) -> Result<()>;
    /// Updates an existing [`QuestionMuOption`](crate::aggregate::QuestionMuOption) row.
    async fn update(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::QuestionMuOption,
    ) -> Result<()>;
}

/// Object-safety smoke test for [`QuestionMuOptionRepository`].
fn _assert_question_mu_option_repository_object_safe(_: Box<dyn QuestionMuOptionRepository>) {}

/// The storage-port contract for [`ResultSetting`](crate::aggregate::ResultSetting) rows.
#[allow(dead_code)]
#[async_trait]
pub trait ResultSettingRepository: Send + Sync {
    /// Returns the [`ResultSetting`](crate::aggregate::ResultSetting) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::ResultSettingId,
    ) -> Result<Option<crate::aggregate::ResultSetting>>;
    /// Inserts a new [`ResultSetting`](crate::aggregate::ResultSetting) row.
    async fn insert(&self, ctx: &TenantContext, s: &crate::aggregate::ResultSetting) -> Result<()>;
    /// Updates an existing [`ResultSetting`](crate::aggregate::ResultSetting) row.
    async fn update(&self, ctx: &TenantContext, s: &crate::aggregate::ResultSetting) -> Result<()>;
}

/// Object-safety smoke test for [`ResultSettingRepository`].
fn _assert_result_setting_repository_object_safe(_: Box<dyn ResultSettingRepository>) {}

/// The storage-port contract for [`ResultStore`](crate::aggregate::ResultStore) rows.
#[allow(dead_code)]
#[async_trait]
pub trait ResultStoreRepository: Send + Sync {
    /// Returns the [`ResultStore`](crate::aggregate::ResultStore) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::ResultStoreId,
    ) -> Result<Option<crate::aggregate::ResultStore>>;
    /// Inserts a new [`ResultStore`](crate::aggregate::ResultStore) row.
    async fn insert(&self, ctx: &TenantContext, s: &crate::aggregate::ResultStore) -> Result<()>;
    /// Updates an existing [`ResultStore`](crate::aggregate::ResultStore) row.
    async fn update(&self, ctx: &TenantContext, s: &crate::aggregate::ResultStore) -> Result<()>;
}

/// Object-safety smoke test for [`ResultStoreRepository`].
fn _assert_result_store_repository_object_safe(_: Box<dyn ResultStoreRepository>) {}

/// The storage-port contract for [`SeatPlanChild`](crate::aggregate::SeatPlanChild) rows.
#[allow(dead_code)]
#[async_trait]
pub trait SeatPlanChildRepository: Send + Sync {
    /// Returns the [`SeatPlanChild`](crate::aggregate::SeatPlanChild) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::SeatPlanChildId,
    ) -> Result<Option<crate::aggregate::SeatPlanChild>>;
    /// Inserts a new [`SeatPlanChild`](crate::aggregate::SeatPlanChild) row.
    async fn insert(&self, ctx: &TenantContext, s: &crate::aggregate::SeatPlanChild) -> Result<()>;
    /// Updates an existing [`SeatPlanChild`](crate::aggregate::SeatPlanChild) row.
    async fn update(&self, ctx: &TenantContext, s: &crate::aggregate::SeatPlanChild) -> Result<()>;
}

/// Object-safety smoke test for [`SeatPlanChildRepository`].
fn _assert_seat_plan_child_repository_object_safe(_: Box<dyn SeatPlanChildRepository>) {}

/// The storage-port contract for [`SeatPlanSetting`](crate::aggregate::SeatPlanSetting) rows.
#[allow(dead_code)]
#[async_trait]
pub trait SeatPlanSettingRepository: Send + Sync {
    /// Returns the [`SeatPlanSetting`](crate::aggregate::SeatPlanSetting) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::SeatPlanSettingId,
    ) -> Result<Option<crate::aggregate::SeatPlanSetting>>;
    /// Inserts a new [`SeatPlanSetting`](crate::aggregate::SeatPlanSetting) row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::SeatPlanSetting,
    ) -> Result<()>;
    /// Updates an existing [`SeatPlanSetting`](crate::aggregate::SeatPlanSetting) row.
    async fn update(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::SeatPlanSetting,
    ) -> Result<()>;
}

/// Object-safety smoke test for [`SeatPlanSettingRepository`].
fn _assert_seat_plan_setting_repository_object_safe(_: Box<dyn SeatPlanSettingRepository>) {}

/// The storage-port contract for [`StudentTakeOnlineExam`](crate::aggregate::StudentTakeOnlineExam) rows.
#[allow(dead_code)]
#[async_trait]
pub trait StudentTakeOnlineExamRepository: Send + Sync {
    /// Returns the [`StudentTakeOnlineExam`](crate::aggregate::StudentTakeOnlineExam) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::StudentTakeOnlineExamId,
    ) -> Result<Option<crate::aggregate::StudentTakeOnlineExam>>;
    /// Inserts a new [`StudentTakeOnlineExam`](crate::aggregate::StudentTakeOnlineExam) row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::StudentTakeOnlineExam,
    ) -> Result<()>;
    /// Updates an existing [`StudentTakeOnlineExam`](crate::aggregate::StudentTakeOnlineExam) row.
    async fn update(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::StudentTakeOnlineExam,
    ) -> Result<()>;
}

/// Object-safety smoke test for [`StudentTakeOnlineExamRepository`].
fn _assert_student_take_online_exam_repository_object_safe(
    _: Box<dyn StudentTakeOnlineExamRepository>,
) {
}

/// The storage-port contract for [`TeacherEvaluation`](crate::aggregate::TeacherEvaluation) rows.
#[allow(dead_code)]
#[async_trait]
pub trait TeacherEvaluationRepository: Send + Sync {
    /// Returns the [`TeacherEvaluation`](crate::aggregate::TeacherEvaluation) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::TeacherEvaluationId,
    ) -> Result<Option<crate::aggregate::TeacherEvaluation>>;
    /// Inserts a new [`TeacherEvaluation`](crate::aggregate::TeacherEvaluation) row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::TeacherEvaluation,
    ) -> Result<()>;
    /// Updates an existing [`TeacherEvaluation`](crate::aggregate::TeacherEvaluation) row.
    async fn update(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::TeacherEvaluation,
    ) -> Result<()>;
}

/// Object-safety smoke test for [`TeacherEvaluationRepository`].
fn _assert_teacher_evaluation_repository_object_safe(_: Box<dyn TeacherEvaluationRepository>) {}

/// The storage-port contract for [`TeacherRemark`](crate::aggregate::TeacherRemark) rows.
#[allow(dead_code)]
#[async_trait]
pub trait TeacherRemarkRepository: Send + Sync {
    /// Returns the [`TeacherRemark`](crate::aggregate::TeacherRemark) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::TeacherRemarkId,
    ) -> Result<Option<crate::aggregate::TeacherRemark>>;
    /// Inserts a new [`TeacherRemark`](crate::aggregate::TeacherRemark) row.
    async fn insert(&self, ctx: &TenantContext, s: &crate::aggregate::TeacherRemark) -> Result<()>;
    /// Updates an existing [`TeacherRemark`](crate::aggregate::TeacherRemark) row.
    async fn update(&self, ctx: &TenantContext, s: &crate::aggregate::TeacherRemark) -> Result<()>;
}

/// Object-safety smoke test for [`TeacherRemarkRepository`].
fn _assert_teacher_remark_repository_object_safe(_: Box<dyn TeacherRemarkRepository>) {}

/// The storage-port contract for [`TemporaryMeritList`](crate::aggregate::TemporaryMeritList) rows.
#[allow(dead_code)]
#[async_trait]
pub trait TemporaryMeritListRepository: Send + Sync {
    /// Returns the [`TemporaryMeritList`](crate::aggregate::TemporaryMeritList) with the given id (or `Ok(None)` if not found).
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::TemporaryMeritListId,
    ) -> Result<Option<crate::aggregate::TemporaryMeritList>>;
    /// Inserts a new [`TemporaryMeritList`](crate::aggregate::TemporaryMeritList) row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::TemporaryMeritList,
    ) -> Result<()>;
    /// Updates an existing [`TemporaryMeritList`](crate::aggregate::TemporaryMeritList) row.
    async fn update(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::TemporaryMeritList,
    ) -> Result<()>;
}

/// Object-safety smoke test for [`TemporaryMeritListRepository`].
fn _assert_temporary_merit_list_repository_object_safe(_: Box<dyn TemporaryMeritListRepository>) {}

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
