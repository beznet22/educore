//! # Assessment command shapes
//!
//! The 3 command shapes Phase 4 Workstream A ships:
//!
//! - [`CreateExamCommand`]
//! - [`UpdateExamCommand`]
//! - [`DeleteExamCommand`]
//!
//! Plus the [`AssessmentUniquenessChecker`] port (the
//! per-academic-year uniqueness check the `create_exam`
//! service calls) and the [`validate_*`] helpers the
//! services call before mutating the aggregate.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use educore_core::error::Result;
use educore_core::ids::SchoolId;
use educore_core::tenant::TenantContext;

use educore_academic::value_objects::PassMark;

use crate::value_objects::{
    AcademicYearId, ClassId, ExamCode, ExamId, ExamMark, ExamName, ExamTypeId, SectionId, SubjectId,
};

// =============================================================================
// Module-level constants (command_type strings for the
// idempotency sub-port).
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
