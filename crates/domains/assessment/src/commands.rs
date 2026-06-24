//! # Assessment command shapes
//!
//! The 3 command shapes Phase 4 Workstream A ships:
//!
//! - [`CreateExamCommand`]
//! - [`UpdateExamCommand`]
//! - [`DeleteExamCommand`]
//!
//! The 9 Workstream B commands (`ScheduleExamCommand`,
//! `UpdateExamScheduleCommand`, `CancelExamScheduleCommand`,
//! `GenerateSeatPlanCommand`, `UpdateSeatPlanCommand`,
//! `CancelSeatPlanCommand`, `GenerateAdmitCardCommand`,
//! `RegenerateAdmitCardCommand`, `CancelAdmitCardCommand`)
//! follow the same shape; see the `commands` module of the
//! [`educore-academic`](::educore_academic) crate for the
//! canonical pattern.
//!
//! Plus the [`AssessmentUniquenessChecker`] port (the
//! per-academic-year uniqueness check the `create_exam`
//! service calls) and the [`validate_*`] helpers the
//! services call before mutating the aggregate.

#![allow(missing_docs)] // The command structs and their
                        // associated functions are self-documenting
                        // via the field/parameter names.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use educore_academic::value_objects::PassMark;
use educore_core::error::Result;
use educore_core::ids::SchoolId;
use educore_core::tenant::TenantContext;
use educore_core::value_objects::Timestamp;

use crate::value_objects::{
    AcademicYearId, AdmitCardId, AdmitCardSettingId, ClassId, CustomResultSettingId, ExamAttendanceId,
    ExamCode, ExamId, ExamMark, ExamName, ExamRoutinePageId, ExamScheduleId, ExamSettingId,
    ExamSignatureId, ExamStepSkipId, ExamTypeId, FrontendExamResultId, FrontExamRoutineId,
    FrontResultId, MarksGradeId, OnlineExamId, QuestionBankId, QuestionGroupId, QuestionLevelId,
    SeatPlanId, SectionId, SubjectId, TeacherEvaluationId, TeacherRemarkId,
};

// =============================================================================
// Module-level constants (command_type strings for the
// idempotency sub-port).
// =============================================================================
// =============================================================================

/// The canonical command_type for the
/// `CreateExamCommand` (used by the idempotency sub-port to
/// key the dedup record).
pub const ASSESSMENT_EXAM_CREATE_COMMAND_TYPE: &str = "assessment.exam.create";

/// The canonical command_type for the
/// `UpdateExamCommand`.
pub const ASSESSMENT_EXAM_UPDATE_COMMAND_TYPE: &str = "assessment.exam.update";

/// The canonical command_type for the
/// `DeleteExamCommand`.
pub const ASSESSMENT_EXAM_DELETE_COMMAND_TYPE: &str = "assessment.exam.delete";

pub const ASSESSMENT_EXAM_SCHEDULE_CREATE_COMMAND_TYPE: &str = "assessment.exam_schedule.create";
pub const ASSESSMENT_EXAM_SCHEDULE_UPDATE_COMMAND_TYPE: &str = "assessment.exam_schedule.update";
pub const ASSESSMENT_EXAM_SCHEDULE_CANCEL_COMMAND_TYPE: &str = "assessment.exam_schedule.cancel";
pub const ASSESSMENT_SEAT_PLAN_GENERATE_COMMAND_TYPE: &str = "assessment.seat_plan.generate";
pub const ASSESSMENT_SEAT_PLAN_UPDATE_COMMAND_TYPE: &str = "assessment.seat_plan.update";
pub const ASSESSMENT_SEAT_PLAN_CANCEL_COMMAND_TYPE: &str = "assessment.seat_plan.cancel";
pub const ASSESSMENT_ADMIT_CARD_GENERATE_COMMAND_TYPE: &str = "assessment.admit_card.generate";
pub const ASSESSMENT_ADMIT_CARD_REGENERATE_COMMAND_TYPE: &str = "assessment.admit_card.regenerate";
pub const ASSESSMENT_ADMIT_CARD_CANCEL_COMMAND_TYPE: &str = "assessment.admit_card.cancel";

// =============================================================================
// CreateExamCommand
// =============================================================================

/// The payload for the `create_exam` service. Carries the
/// full set of immutable + initial mutable fields the
/// `Exam` aggregate needs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateExamCommand {
    /// The tenant context (actor + correlation + school).
    pub tenant: TenantContext,
    /// The exam's typed id (caller-supplied; the service does
    /// not mint it).
    pub exam_id: ExamId,
    /// The exam type (mid-term, final, monthly, …).
    pub exam_type_id: ExamTypeId,
    /// The class the exam is administered to.
    pub class_id: ClassId,
    /// The section the exam is administered to.
    pub section_id: SectionId,
    /// The subject the exam is for.
    pub subject_id: SubjectId,
    /// The academic year the exam belongs to.
    pub academic_year_id: AcademicYearId,
    /// The human-readable exam name.
    pub name: String,
    /// The exam code (unique within `(school, academic_year)`).
    pub code: String,
    /// The exam's full mark.
    pub exam_mark: f32,
    /// The exam's pass mark.
    pub pass_mark: f32,
    /// The exam's date.
    pub exam_date: NaiveDate,
}

impl CreateExamCommand {
    /// Returns the `SchoolId` carried by the command's tenant
    /// context. Used by the service to assert that the
    /// `exam_id` is anchored to the same school.
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }
}

// =============================================================================
// UpdateExamCommand
// =============================================================================

/// The payload for the `update_exam` service. All mutable
/// fields are `Option<T>` for partial-update semantics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateExamCommand {
    /// The tenant context (actor + correlation + school).
    pub tenant: TenantContext,
    /// The exam's typed id.
    pub exam_id: ExamId,
    /// New exam name (None = no change).
    pub name: Option<String>,
    /// New exam code (None = no change).
    pub code: Option<String>,
    /// New exam mark (None = no change).
    pub exam_mark: Option<f32>,
    /// New pass mark (None = no change).
    pub pass_mark: Option<f32>,
    /// New exam date (None = no change).
    pub exam_date: Option<NaiveDate>,
    /// New publish state (None = no change; Some(b) = set
    /// to `b`).
    pub is_published: Option<bool>,
}

impl UpdateExamCommand {
    /// Returns the `SchoolId` carried by the command's tenant
    /// context.
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }
}

// =============================================================================
// DeleteExamCommand
// =============================================================================

/// The payload for the `delete_exam` service. Carries only
/// the exam id; the service asserts the invariant
/// (no `MarksRegister` references the exam) before mutating.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteExamCommand {
    /// The tenant context (actor + correlation + school).
    pub tenant: TenantContext,
    /// The exam's typed id.
    pub exam_id: ExamId,
}

impl DeleteExamCommand {
    /// Returns the `SchoolId` carried by the command's tenant
    /// context.
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }
}

// =============================================================================
// Workstream B: ExamSchedule, SeatPlan, AdmitCard commands
// (minimal shape; the full validation helpers land in a
// follow-up phase).
// =============================================================================

/// A per-subject slot in a `ScheduleExam` command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScheduleSubjectEntry {
    pub subject_id: SubjectId,
    pub date: chrono::NaiveDate,
    pub start_time: chrono::NaiveTime,
    pub end_time: chrono::NaiveTime,
    pub room_id: Option<crate::value_objects::ClassRoomId>,
    pub full_mark: f32,
    pub pass_mark: f32,
}

/// The `schedule_exam` command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScheduleExamCommand {
    pub tenant: TenantContext,
    pub schedule_id: ExamScheduleId,
    pub exam_id: ExamId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub date: chrono::NaiveDate,
    pub start_time: chrono::NaiveTime,
    pub end_time: chrono::NaiveTime,
    pub subjects: Vec<ScheduleSubjectEntry>,
}
impl ScheduleExamCommand {
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }
}

/// The `update_exam_schedule` command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateExamScheduleCommand {
    pub tenant: TenantContext,
    pub schedule_id: ExamScheduleId,
    pub date: Option<chrono::NaiveDate>,
    pub start_time: Option<chrono::NaiveTime>,
    pub end_time: Option<chrono::NaiveTime>,
}
impl UpdateExamScheduleCommand {
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }
}

/// The `cancel_exam_schedule` command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CancelExamScheduleCommand {
    pub tenant: TenantContext,
    pub schedule_id: ExamScheduleId,
    pub reason: String,
}
impl CancelExamScheduleCommand {
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }
}

/// A per-room allocation in a `GenerateSeatPlan` command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeatPlanAllocation {
    pub room_id: crate::value_objects::ClassRoomId,
    pub assign_students: u32,
    pub start_time: chrono::NaiveTime,
    pub end_time: chrono::NaiveTime,
}

/// The `generate_seat_plan` command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerateSeatPlanCommand {
    pub tenant: TenantContext,
    pub seat_plan_id: SeatPlanId,
    pub exam_id: ExamId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub allocations: Vec<SeatPlanAllocation>,
}
impl GenerateSeatPlanCommand {
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }
}

/// The `update_seat_plan` command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateSeatPlanCommand {
    pub tenant: TenantContext,
    pub seat_plan_id: SeatPlanId,
    pub allocations: Option<Vec<SeatPlanAllocation>>,
}
impl UpdateSeatPlanCommand {
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }
}

/// The `cancel_seat_plan` command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CancelSeatPlanCommand {
    pub tenant: TenantContext,
    pub seat_plan_id: SeatPlanId,
}
impl CancelSeatPlanCommand {
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }
}

/// The `generate_admit_card` command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerateAdmitCardCommand {
    pub tenant: TenantContext,
    pub admit_card_id: AdmitCardId,
    pub student_record_id: crate::value_objects::StudentRecordId,
    pub exam_type_id: ExamTypeId,
    pub academic_year_id: AcademicYearId,
}
impl GenerateAdmitCardCommand {
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }
}

/// The `regenerate_admit_card` command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegenerateAdmitCardCommand {
    pub tenant: TenantContext,
    pub admit_card_id: AdmitCardId,
    pub previous_id: AdmitCardId,
    pub reason: String,
}
impl RegenerateAdmitCardCommand {
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }
}

/// The `cancel_admit_card` command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CancelAdmitCardCommand {
    pub tenant: TenantContext,
    pub admit_card_id: AdmitCardId,
    pub reason: String,
}
impl CancelAdmitCardCommand {
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }
}

// =============================================================================
// AssessmentUniquenessChecker (port)
// =============================================================================

/// The per-academic-year uniqueness check the
/// `create_exam` service calls. The contract is
/// per-school and per-academic-year:
/// `(school_id, academic_year_id, exam_type_id, class_id,
/// section_id, subject_id)` must be unique.
///
/// Production wiring: a thin adapter over the storage port.
/// Test wiring: an in-memory `Mutex<HashSet<(SchoolId,
/// AcademicYearId, ExamTypeId, ClassId, SectionId,
/// SubjectId)>>`.
pub trait AssessmentUniquenessChecker: Send + Sync {
    /// Returns `true` if an exam with the same
    /// `(school_id, academic_year_id, exam_type_id, class_id,
    /// section_id, subject_id)` tuple already exists.
    fn exam_unique_key_exists(
        &self,
        school: SchoolId,
        academic_year: AcademicYearId,
        exam_type: ExamTypeId,
        class: ClassId,
        section: SectionId,
        subject: SubjectId,
    ) -> bool;
}

/// Alias retained for the prelude re-export (matches the
/// academic crate's `UniquenessChecker` shape).
pub type UniquenessChecker = dyn AssessmentUniquenessChecker;

// =============================================================================
// Validators (called by the service before mutation)
// =============================================================================

/// Validates an [`ExamName`]. Returns the validated newtype
/// or a `Validation` error.
pub fn validate_exam_name(s: &str) -> Result<ExamName> {
    ExamName::new(s)
}

/// Validates an [`ExamCode`]. Returns the validated newtype
/// or a `Validation` error.
pub fn validate_exam_code(s: &str) -> Result<ExamCode> {
    ExamCode::new(s)
}

/// Validates an [`ExamMark`]. Returns the validated newtype
/// or a `Validation` error.
pub fn validate_exam_mark(v: f32) -> Result<ExamMark> {
    ExamMark::new(v)
}

/// Validates a [`PassMark`]. Returns the validated newtype
/// or a `Validation` error. (Re-export of
/// `educore_academic::value_objects::PassMark::new`.)
pub fn validate_pass_mark(v: f32) -> Result<PassMark> {
    PassMark::new(v)
}

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
    use educore_core::error::DomainError;
    use educore_core::ids::{CorrelationId, UserId};
    use educore_core::tenant::UserType;

    fn make_create() -> CreateExamCommand {
        let g = SystemIdGen;
        let s = g.next_school_id();
        CreateExamCommand {
            tenant: TenantContext::for_user(
                s,
                g.next_user_id(),
                g.next_correlation_id(),
                UserType::SchoolAdmin,
            ),
            exam_id: ExamId::new(s, uuid::Uuid::now_v7()),
            exam_type_id: ExamTypeId::new(s, uuid::Uuid::now_v7()),
            class_id: ClassId::new(s, uuid::Uuid::now_v7()),
            section_id: SectionId::new(s, uuid::Uuid::now_v7()),
            subject_id: SubjectId::new(s, uuid::Uuid::now_v7()),
            academic_year_id: AcademicYearId::new(s, uuid::Uuid::now_v7()),
            name: "Mid-Term Mathematics".to_owned(),
            code: "MTH-MT-2024".to_owned(),
            exam_mark: 100.0,
            pass_mark: 35.0,
            exam_date: chrono::NaiveDate::from_ymd_opt(2024, 9, 15).unwrap(),
        }
    }

    #[test]
    fn command_type_strings_are_stable() {
        assert_eq!(
            ASSESSMENT_EXAM_CREATE_COMMAND_TYPE,
            "assessment.exam.create"
        );
        assert_eq!(
            ASSESSMENT_EXAM_UPDATE_COMMAND_TYPE,
            "assessment.exam.update"
        );
        assert_eq!(
            ASSESSMENT_EXAM_DELETE_COMMAND_TYPE,
            "assessment.exam.delete"
        );
    }

    #[test]
    fn create_exam_school_id_matches_tenant() {
        let cmd = make_create();
        let s = cmd.tenant.school_id;
        assert_eq!(cmd.school_id(), s);
        assert_eq!(cmd.exam_id.school_id(), s);
    }

    #[test]
    fn update_exam_carries_partial_fields() {
        let s = SchoolId(uuid::Uuid::now_v7());
        let cmd = UpdateExamCommand {
            tenant: TenantContext::for_user(
                s,
                UserId(uuid::Uuid::now_v7()),
                CorrelationId(uuid::Uuid::now_v7()),
                UserType::SchoolAdmin,
            ),
            exam_id: ExamId::new(s, uuid::Uuid::now_v7()),
            name: None,
            code: None,
            exam_mark: Some(120.0),
            pass_mark: None,
            exam_date: None,
            is_published: Some(true),
        };
        assert_eq!(cmd.school_id(), s);
        assert_eq!(cmd.exam_mark, Some(120.0));
        assert_eq!(cmd.is_published, Some(true));
    }

    #[test]
    fn delete_exam_carries_only_id() {
        let s = SchoolId(uuid::Uuid::now_v7());
        let exam_id = ExamId::new(s, uuid::Uuid::now_v7());
        let cmd = DeleteExamCommand {
            tenant: TenantContext::for_user(
                s,
                UserId(uuid::Uuid::now_v7()),
                CorrelationId(uuid::Uuid::now_v7()),
                UserType::SchoolAdmin,
            ),
            exam_id,
        };
        assert_eq!(cmd.school_id(), s);
        assert_eq!(cmd.exam_id, exam_id);
    }

    #[test]
    fn validate_exam_mark_rejects_zero() {
        let err = validate_exam_mark(0.0).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn validate_exam_mark_accepts_positive() {
        let m = validate_exam_mark(100.0).unwrap();
        assert_eq!(m.as_f32(), 100.0);
    }

    #[test]
    fn validate_exam_name_rejects_empty() {
        let err = validate_exam_name("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn validate_exam_code_rejects_empty() {
        let err = validate_exam_code("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn validate_pass_mark_accepts_in_range() {
        let m = validate_pass_mark(35.0).unwrap();
        assert_eq!(m.as_f32(), 35.0);
    }

    #[test]
    fn validate_pass_mark_rejects_negative() {
        let err = validate_pass_mark(-1.0).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn validate_pass_mark_rejects_over_100() {
        let err = validate_pass_mark(101.0).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }
}

// =============================================================================
// Workstream C commands: MarksRegister, ResultStore, ReportCard
// =============================================================================

/// The `initialize_marks_register` command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InitializeMarksRegisterCommand {
    pub tenant: TenantContext,
    pub marks_register_id: crate::value_objects::MarksRegisterId,
    pub exam_id: ExamId,
    pub student_id: crate::value_objects::StudentId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub academic_year_id: AcademicYearId,
    pub subject_ids: Vec<SubjectId>,
}
impl InitializeMarksRegisterCommand {
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }
}

/// The `enter_marks` command. Enters a single subject's marks
/// for a student.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnterMarksCommand {
    pub tenant: TenantContext,
    pub marks_register_id: crate::value_objects::MarksRegisterId,
    pub subject_id: SubjectId,
    pub student_id: crate::value_objects::StudentId,
    pub marks: Option<f32>,
    pub is_absent: bool,
    pub comment: Option<String>,
}
impl EnterMarksCommand {
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }
}

/// The `submit_marks` command. Locks the register for grading.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubmitMarksCommand {
    pub tenant: TenantContext,
    pub marks_register_id: crate::value_objects::MarksRegisterId,
}
impl SubmitMarksCommand {
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }
}

/// The `publish_result` command. Materialises `ResultStore`
/// rows and emits `ResultPublished`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublishResultCommand {
    pub tenant: TenantContext,
    pub exam_id: ExamId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub academic_year_id: AcademicYearId,
    pub published_at: Timestamp,
}
impl PublishResultCommand {
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }
}

/// The `republish_result` command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepublishResultCommand {
    pub tenant: TenantContext,
    pub result_store_id: crate::value_objects::ResultStoreId,
    pub reason: String,
    pub republished_at: Timestamp,
}
impl RepublishResultCommand {
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }
}

/// The `update_result_remarks` command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateResultRemarksCommand {
    pub tenant: TenantContext,
    pub result_store_id: crate::value_objects::ResultStoreId,
    pub teacher_remarks: String,
}
impl UpdateResultRemarksCommand {
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }
}

/// The `generate_report_card` command (the report card is
/// a projection — it has no aggregate; the service
/// materialises a `ReportCardPayload` from a published
/// result).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerateReportCardCommand {
    pub tenant: TenantContext,
    pub result_store_id: crate::value_objects::ResultStoreId,
    pub student_id: crate::value_objects::StudentId,
    pub include_remarks: bool,
}
impl GenerateReportCardCommand {
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }
}

// =============================================================================
// MarksGradeScale port (Workstream C)
// =============================================================================

/// The school's `MarksGrade` scale, supplied to
/// `publish_result` and the `ResultService` grading
/// functions. Production wires a per-school
/// `MarksGradeScale` (Phase 14, Settings); tests wire an
/// `InMemoryMarksGradeScale` (default A-F scale).
pub trait MarksGradeScale: Send + Sync {
    /// Returns the `MarksGradeRow` for the given percent.
    /// Returns `None` if the percent is outside the scale
    /// (which is a `Validation` error for the caller).
    fn lookup(&self, percent: f32) -> Option<crate::value_objects::MarksGradeRow>;
    /// Returns `true` if the scale is valid (no overlaps, no gaps).
    fn validate(&self) -> bool;
}

// =============================================================================
// Cluster C: command stubs for the new aggregates.
//
// Each stub carries the minimal `school_id` + aggregate id (plus 1-2
// common fields where the spec makes the dominant scalar obvious).
// The full TenantContext + payload lands in a follow-up batch.
//
// See `docs/specs/assessment/commands.md` for the canonical
// command shapes; aggregates without a corresponding spec
// command (`ExamSetup`, `MarkStore`, `ResultSetting`,
// `StudentTakeOnlineExam`, `TemporaryMeritList`,
// `CustomTemporaryResult`) are skipped because they are
// service-internal staging / projection roots.
// =============================================================================

// --- MarksGrade cluster -----------------------------------------------------

/// The `create_marks_grade` command.
#[derive(Debug, Clone)]
pub struct CreateMarksGradeCommand {
    pub school_id: SchoolId,
    pub marks_grade_id: MarksGradeId,
}

/// The `update_marks_grade` command.
#[derive(Debug, Clone)]
pub struct UpdateMarksGradeCommand {
    pub school_id: SchoolId,
    pub marks_grade_id: MarksGradeId,
}

/// The `delete_marks_grade` command.
#[derive(Debug, Clone)]
pub struct DeleteMarksGradeCommand {
    pub school_id: SchoolId,
    pub marks_grade_id: MarksGradeId,
}

// --- ExamSetting cluster ----------------------------------------------------

/// The `create_exam_setting` command.
#[derive(Debug, Clone)]
pub struct CreateExamSettingCommand {
    pub school_id: SchoolId,
    pub exam_setting_id: ExamSettingId,
}

/// The `update_exam_setting` command.
#[derive(Debug, Clone)]
pub struct UpdateExamSettingCommand {
    pub school_id: SchoolId,
    pub exam_setting_id: ExamSettingId,
}

/// The `delete_exam_setting` command.
#[derive(Debug, Clone)]
pub struct DeleteExamSettingCommand {
    pub school_id: SchoolId,
    pub exam_setting_id: ExamSettingId,
}

/// The `set_exam_signature` command.
#[derive(Debug, Clone)]
pub struct SetExamSignatureCommand {
    pub school_id: SchoolId,
    pub exam_signature_id: ExamSignatureId,
}

// --- ExamRoutinePage cluster ------------------------------------------------

/// The `update_exam_routine_page` command.
#[derive(Debug, Clone)]
pub struct UpdateExamRoutinePageCommand {
    pub school_id: SchoolId,
    pub exam_routine_page_id: ExamRoutinePageId,
}

/// The `publish_exam_routine` command.
#[derive(Debug, Clone)]
pub struct PublishExamRoutineCommand {
    pub school_id: SchoolId,
    pub front_exam_routine_id: FrontExamRoutineId,
}

/// The `publish_front_result` command.
#[derive(Debug, Clone)]
pub struct PublishFrontResultCommand {
    pub school_id: SchoolId,
    pub front_result_id: FrontResultId,
}

/// The `update_frontend_exam_result` command.
#[derive(Debug, Clone)]
pub struct UpdateFrontendExamResultCommand {
    pub school_id: SchoolId,
    pub frontend_exam_result_id: FrontendExamResultId,
}

// --- OnlineExam cluster -----------------------------------------------------

/// The `create_online_exam` command.
#[derive(Debug, Clone)]
pub struct CreateOnlineExamCommand {
    pub school_id: SchoolId,
    pub online_exam_id: OnlineExamId,
}

/// The `publish_online_exam` command.
#[derive(Debug, Clone)]
pub struct PublishOnlineExamCommand {
    pub school_id: SchoolId,
    pub online_exam_id: OnlineExamId,
}

/// The `start_online_exam` command.
#[derive(Debug, Clone)]
pub struct StartOnlineExamCommand {
    pub school_id: SchoolId,
    pub online_exam_id: OnlineExamId,
}

/// The `submit_online_exam_answer` command.
#[derive(Debug, Clone)]
pub struct SubmitOnlineExamAnswerCommand {
    pub school_id: SchoolId,
    pub online_exam_id: OnlineExamId,
}

/// The `evaluate_online_exam` command.
#[derive(Debug, Clone)]
pub struct EvaluateOnlineExamCommand {
    pub school_id: SchoolId,
    pub online_exam_id: OnlineExamId,
}

// --- QuestionBank cluster ---------------------------------------------------

/// The `create_question` command.
#[derive(Debug, Clone)]
pub struct CreateQuestionCommand {
    pub school_id: SchoolId,
    pub question_bank_id: QuestionBankId,
}

/// The `update_question` command.
#[derive(Debug, Clone)]
pub struct UpdateQuestionCommand {
    pub school_id: SchoolId,
    pub question_bank_id: QuestionBankId,
}

/// The `delete_question` command.
#[derive(Debug, Clone)]
pub struct DeleteQuestionCommand {
    pub school_id: SchoolId,
    pub question_bank_id: QuestionBankId,
}

// --- QuestionGroup cluster --------------------------------------------------

/// The `create_question_group` command.
#[derive(Debug, Clone)]
pub struct CreateQuestionGroupCommand {
    pub school_id: SchoolId,
    pub question_group_id: QuestionGroupId,
}

/// The `update_question_group` command.
#[derive(Debug, Clone)]
pub struct UpdateQuestionGroupCommand {
    pub school_id: SchoolId,
    pub question_group_id: QuestionGroupId,
}

/// The `delete_question_group` command.
#[derive(Debug, Clone)]
pub struct DeleteQuestionGroupCommand {
    pub school_id: SchoolId,
    pub question_group_id: QuestionGroupId,
}

// --- QuestionLevel cluster --------------------------------------------------

/// The `create_question_level` command.
#[derive(Debug, Clone)]
pub struct CreateQuestionLevelCommand {
    pub school_id: SchoolId,
    pub question_level_id: QuestionLevelId,
}

/// The `update_question_level` command.
#[derive(Debug, Clone)]
pub struct UpdateQuestionLevelCommand {
    pub school_id: SchoolId,
    pub question_level_id: QuestionLevelId,
}

/// The `delete_question_level` command.
#[derive(Debug, Clone)]
pub struct DeleteQuestionLevelCommand {
    pub school_id: SchoolId,
    pub question_level_id: QuestionLevelId,
}

// --- AdmitCardSetting cluster -----------------------------------------------

/// The `configure_admit_card_settings` command.
#[derive(Debug, Clone)]
pub struct ConfigureAdmitCardSettingsCommand {
    pub school_id: SchoolId,
    pub admit_card_setting_id: AdmitCardSettingId,
}

// --- TeacherEvaluation cluster ----------------------------------------------

/// The `mark_teacher_evaluation` command.
#[derive(Debug, Clone)]
pub struct MarkTeacherEvaluationCommand {
    pub school_id: SchoolId,
    pub teacher_evaluation_id: TeacherEvaluationId,
}

/// The `approve_teacher_evaluation` command.
#[derive(Debug, Clone)]
pub struct ApproveTeacherEvaluationCommand {
    pub school_id: SchoolId,
    pub teacher_evaluation_id: TeacherEvaluationId,
}

/// The `reject_teacher_evaluation` command.
#[derive(Debug, Clone)]
pub struct RejectTeacherEvaluationCommand {
    pub school_id: SchoolId,
    pub teacher_evaluation_id: TeacherEvaluationId,
}

// --- TeacherRemark cluster --------------------------------------------------

/// The `add_teacher_remark` command.
#[derive(Debug, Clone)]
pub struct AddTeacherRemarkCommand {
    pub school_id: SchoolId,
    pub teacher_remark_id: TeacherRemarkId,
}

/// The `update_teacher_remark` command.
#[derive(Debug, Clone)]
pub struct UpdateTeacherRemarkCommand {
    pub school_id: SchoolId,
    pub teacher_remark_id: TeacherRemarkId,
}

// --- CustomResultSetting cluster --------------------------------------------

/// The `configure_custom_result_settings` command.
#[derive(Debug, Clone)]
pub struct ConfigureCustomResultSettingsCommand {
    pub school_id: SchoolId,
    pub custom_result_setting_id: CustomResultSettingId,
}

// --- ExamStepSkip cluster ---------------------------------------------------

/// The `mark_exam_step_skip` command.
#[derive(Debug, Clone)]
pub struct MarkExamStepSkipCommand {
    pub school_id: SchoolId,
    pub exam_step_skip_id: ExamStepSkipId,
}

// --- ExamAttendance cluster -------------------------------------------------

/// The `mark_exam_attendance` command.
#[derive(Debug, Clone)]
pub struct MarkExamAttendanceCommand {
    pub school_id: SchoolId,
    pub exam_attendance_id: ExamAttendanceId,
}

/// The `update_exam_attendance` command.
#[derive(Debug, Clone)]
pub struct UpdateExamAttendanceCommand {
    pub school_id: SchoolId,
    pub exam_attendance_id: ExamAttendanceId,
}

// =============================================================================
// Cluster D final 20%: spec commands previously missing
// (`CreateExamType`, `UpdateExamType`, `DeleteExamType`,
// `ConfigureSeatPlanSettings`, `ConfigureTeacherEvaluation`,
// `RequestAbsenceNotification`).
//
// Each stub carries the minimal `school_id` + aggregate id
// (plus the natural primary key where the spec makes the
// upsert target obvious). The full TenantContext + payload
// lands in a follow-up batch.
// =============================================================================

/// The `create_exam_type` command.
#[derive(Debug, Clone)]
pub struct CreateExamTypeCommand {
    pub school_id: SchoolId,
    pub exam_type_id: ExamTypeId,
}

/// The `update_exam_type` command.
#[derive(Debug, Clone)]
pub struct UpdateExamTypeCommand {
    pub school_id: SchoolId,
    pub exam_type_id: ExamTypeId,
}

/// The `delete_exam_type` command.
#[derive(Debug, Clone)]
pub struct DeleteExamTypeCommand {
    pub school_id: SchoolId,
    pub exam_type_id: ExamTypeId,
}

/// The `configure_seat_plan_settings` command. Upserts the
/// per-(school, academic-year) `SeatPlanSetting`.
#[derive(Debug, Clone)]
pub struct ConfigureSeatPlanSettingsCommand {
    pub school_id: SchoolId,
    pub academic_year_id: AcademicYearId,
}

/// The `configure_teacher_evaluation` command. Upserts the
/// per-school `TeacherEvaluationSetting`.
#[derive(Debug, Clone)]
pub struct ConfigureTeacherEvaluationCommand {
    pub school_id: SchoolId,
}

/// The `request_absence_notification` command. Emits the
/// `ExamAbsenceNotificationRequested` trigger event that the
/// communication domain subscribes to.
#[derive(Debug, Clone)]
pub struct RequestAbsenceNotificationCommand {
    pub school_id: SchoolId,
    pub exam_attendance_id: ExamAttendanceId,
}
