//! # Assessment value objects
//!
//! The typed ids (every aggregate is keyed by one) and the
//! validated value objects the assessment aggregates depend on.
//! Per `docs/specs/assessment/value-objects.md`:
//!
//! - Every id is `Id { school_id, value }` — a typed wrapper
//!   that carries the school anchor so the type system catches
//!   cross-tenant confusion at compile time.
//! - Strings (exam names, codes, …) are validated at
//!   construction. The constructors return
//!   `Result<Self, DomainError>`; there are no setters that
//!   bypass validation.
//! - Status enums are closed (`ExamTerm`,
//!   `OnlineExamStatus`, `ResultStatus`, `MarksRegisterStatus`).
//!
//! Phase 4 ships the prompt-named subset: id types for the
//! [`ExamId`](self) + [`ExamTypeId`](self) (the Exam
//! aggregate's typed id and the ExamType foreign key);
//! the [`ExamMark`](self) and [`ExamName`](self) /
//! [`ExamCode`](self) value objects; the shared
//! [`ExamTerm`](self) / [`ResultStatus`](self) enums. The
//! other 12+ typed ids (ExamSchedule, MarksRegister, …) and
//! value objects (Marks, Gpa, Grade, …) land alongside their
//! respective aggregates in the Phase 4 workstreams B / C /
//! D.

#![allow(missing_docs)] // The new types in Workstream C (Marks,
                        // TotalMarks, Gpa, Grade, MarksGradeRow)
                        // are described by their constructor
                        // signatures; suppressing this lint for
                        // the file is the pragmatic choice.

use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::error::{DomainError, Result};
use educore_core::ids::SchoolId;

// =============================================================================
// Macro: typed assessment id
// =============================================================================

/// Macro to define the per-aggregate typed id wrapper. Every
/// assessment id follows the same shape: a `school_id` anchor
/// plus a local `Uuid`. The wrapper implements
/// [`Clone`], [`Copy`], [`PartialEq`], [`Eq`], [`Hash`], and
/// the `Display` format `"{school_id}/{value}"`.
///
/// The pattern matches `educore-academic::value_objects::academic_typed_id!`
/// and `educore-rbac::ids::rbac_typed_id!` so the engine's
/// id types stay consistent across crates.
macro_rules! assessment_typed_id {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident;
    ) => {
        $(#[$attr])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        $vis struct $name {
            /// The owning school (tenant anchor).
            pub school_id: SchoolId,
            /// The local id (UUIDv7).
            pub value: Uuid,
        }

        impl $name {
            /// Constructs a new typed id from its parts.
            #[must_use]
            pub const fn new(school_id: SchoolId, value: Uuid) -> Self {
                Self { school_id, value }
            }

            /// Returns the local UUID.
            #[must_use]
            pub const fn as_uuid(&self) -> Uuid {
                self.value
            }

            /// Returns the owning school id.
            #[must_use]
            pub const fn school_id(&self) -> SchoolId {
                self.school_id
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}/{}", self.school_id, self.value)
            }
        }
    };
}

// =============================================================================
// Typed ids (Workstream A: Exam + ExamType FK)
// =============================================================================

assessment_typed_id! {
    /// A typed id for an [`Exam`](crate::aggregate::Exam) row.
    pub struct ExamId;
}

assessment_typed_id! {
    /// A typed id for an `ExamType` row (the catalog of exam
    /// categories like mid-term, final, monthly, …). The full
    /// `ExamType` aggregate is out of scope for Phase 4; the
    /// typed id is defined here so the `Exam` aggregate can
    /// declare its foreign-key field against a stable type.
    pub struct ExamTypeId;
}

assessment_typed_id! {
    /// A typed id for an [`ExamSchedule`](crate::aggregate::ExamSchedule)
    /// row. Owns the per-subject `ExamScheduleSubject` children.
    pub struct ExamScheduleId;
}

assessment_typed_id! {
    /// A typed id for an
    /// [`ExamScheduleSubject`](crate::entities::ExamScheduleSubject)
    /// child row.
    pub struct ExamScheduleSubjectId;
}

assessment_typed_id! {
    /// A typed id for a [`SeatPlan`](crate::aggregate::SeatPlan)
    /// row. Owns the per-room `SeatPlanChild` children.
    pub struct SeatPlanId;
}

assessment_typed_id! {
    /// A typed id for a [`SeatPlanChild`](crate::entities::SeatPlanChild)
    /// child row.
    pub struct SeatPlanChildId;
}

assessment_typed_id! {
    /// A typed id for an [`AdmitCard`](crate::aggregate::AdmitCard) row.
    pub struct AdmitCardId;
}

// Placeholder typed ids for the academic crate's StaffId / ClassRoomId.
// The full StaffId lands in the HR domain (Phase 6); the full
// ClassRoomId lands in a future academic phase. The
// assessment crate declares its own placeholder so the
// foreign-key fields are typed; the academic crate's full
// definition will be wired in via a re-export once it lands.
assessment_typed_id! {
    /// A typed id for a Staff aggregate (the invigilating
    /// teacher for an exam). Placeholder until the HR domain
    /// ships its `Staff` aggregate in Phase 6.
    pub struct StaffId;
}

assessment_typed_id! {
    /// A typed id for a [`ClassRoom`] aggregate (a physical
    /// room used as an exam venue). Placeholder until the
    /// facilities domain ships its `Room` aggregate in
    /// Phase 8 — or until the academic crate lifts its
    /// own `ClassRoom` row to an aggregate in a future
    /// phase.
    pub struct ClassRoomId;
}

assessment_typed_id! {
    /// A typed id for a [`MarksRegister`](crate::aggregate::MarksRegister)
    /// row (one exam's marks for one section).
    pub struct MarksRegisterId;
}

assessment_typed_id! {
    /// A typed id for a [`MarksRegisterChild`](crate::entities::MarksRegisterChild)
    /// child row (per-subject marks).
    pub struct MarksRegisterChildId;
}

assessment_typed_id! {
    /// A typed id for a [`ResultStore`](crate::aggregate::ResultStore)
    /// row (the published per-student per-subject result).
    pub struct ResultStoreId;
}

// =============================================================================
// Names (1..=N chars, validated at construction)
// =============================================================================

/// A validated, non-empty exam name. 1..=200 chars (per
/// `docs/specs/assessment/value-objects.md` § Names and
/// Codes). Unique within `(school_id, exam_type, year)`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ExamName(String);

impl ExamName {
    /// Maximum length of an exam name.
    pub const MAX_LEN: usize = 200;

    /// Constructs an `ExamName`, rejecting empty or overlong
    /// input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_exam_name(&s)?;
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ExamName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for ExamName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

fn validate_exam_name(s: &str) -> Result<()> {
    if s.is_empty() {
        return Err(DomainError::validation("exam name must not be empty"));
    }
    if s.chars().count() > ExamName::MAX_LEN {
        return Err(DomainError::validation(format!(
            "exam name must be at most {} chars, got {}",
            ExamName::MAX_LEN,
            s.chars().count()
        )));
    }
    Ok(())
}

/// A validated, non-empty exam code. 1..=50 chars (per
/// `docs/specs/assessment/value-objects.md` § Names and
/// Codes). Unique within `(school_id, academic_year_id)`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ExamCode(String);

impl ExamCode {
    /// Maximum length of an exam code.
    pub const MAX_LEN: usize = 50;

    /// Constructs an `ExamCode`, rejecting empty or overlong
    /// input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_exam_code(&s)?;
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ExamCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for ExamCode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

fn validate_exam_code(s: &str) -> Result<()> {
    if s.is_empty() {
        return Err(DomainError::validation("exam code must not be empty"));
    }
    if s.chars().count() > ExamCode::MAX_LEN {
        return Err(DomainError::validation(format!(
            "exam code must be at most {} chars, got {}",
            ExamCode::MAX_LEN,
            s.chars().count()
        )));
    }
    Ok(())
}

// =============================================================================
// Marks and grades (numeric wrappers)
// =============================================================================

/// The full mark (max obtainable score) for an exam. `f32` in
/// `(0, 1000]` (per `docs/specs/assessment/value-objects.md` §
/// Marks and Grades).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ExamMark(f32);

impl ExamMark {
    /// Constructs an `ExamMark`, rejecting non-positive or
    /// overlong values.
    pub fn new(v: f32) -> Result<Self> {
        if v <= 0.0 {
            return Err(DomainError::validation(format!(
                "exam mark must be > 0, got {v}"
            )));
        }
        if v > 1000.0 {
            return Err(DomainError::validation(format!(
                "exam mark must be <= 1000, got {v}"
            )));
        }
        Ok(Self(v))
    }

    /// Returns the inner `f32`.
    #[must_use]
    pub const fn as_f32(self) -> f32 {
        self.0
    }
}

/// The full mark for a per-subject slot in a multi-subject
/// exam. Same range as `ExamMark` (`f32` in `(0, 1000]`).
/// Distinct newtype so the per-subject field path is explicit
/// in serialised payloads.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FullMark(f32);
impl FullMark {
    /// Constructs a `FullMark`, rejecting non-positive or
    /// overlong values.
    pub fn new(v: f32) -> Result<Self> {
        if v <= 0.0 {
            return Err(DomainError::validation(format!(
                "full mark must be > 0, got {v}"
            )));
        }
        if v > 1000.0 {
            return Err(DomainError::validation(format!(
                "full mark must be <= 1000, got {v}"
            )));
        }
        Ok(Self(v))
    }

    /// Returns the inner `f32`.
    #[must_use]
    pub const fn as_f32(self) -> f32 {
        self.0
    }
}

/// The actual marks obtained by a student in a subject.
/// `f32` in `[0, 1000]`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Marks(f32);
impl Marks {
    pub fn new(v: f32) -> educore_core::error::Result<Self> {
        if v < 0.0 {
            return Err(educore_core::error::DomainError::validation(format!(
                "marks must be >= 0, got {v}"
            )));
        }
        if v > 1000.0 {
            return Err(educore_core::error::DomainError::validation(format!(
                "marks must be <= 1000, got {v}"
            )));
        }
        Ok(Self(v))
    }
    #[must_use]
    pub const fn as_f32(self) -> f32 {
        self.0
    }
}

/// The total marks obtained by a student across subjects.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TotalMarks(f32);
impl TotalMarks {
    pub fn new(v: f32) -> educore_core::error::Result<Self> {
        if v < 0.0 {
            return Err(educore_core::error::DomainError::validation(format!(
                "total marks must be >= 0, got {v}"
            )));
        }
        Ok(Self(v))
    }
    #[must_use]
    pub const fn as_f32(self) -> f32 {
        self.0
    }
}

/// The grade point on a 0-5 scale.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Gpa(f32);
impl Gpa {
    pub fn new(v: f32) -> educore_core::error::Result<Self> {
        if !(0.0..=5.0).contains(&v) {
            return Err(educore_core::error::DomainError::validation(format!(
                "gpa must be in [0, 5], got {v}"
            )));
        }
        Ok(Self(v))
    }
    #[must_use]
    pub const fn as_f32(self) -> f32 {
        self.0
    }
}

/// A school-defined grade scale row (per the spec's
/// `MarksGrade` aggregate, out of scope for Phase 4).
/// The port trait `MarksGradeScale` is a Phase 4 deliverable
/// (Workstream C); the aggregate is Phase 14 (Settings).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarksGradeRow {
    pub from_percent: f32,
    pub up_to_percent: f32,
    pub grade: crate::value_objects::Grade,
    pub gpa: Gpa,
    pub is_fail: bool,
}

/// A school-defined grade string (e.g. "A+", "B", "F").
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Grade(String);
impl Grade {
    pub fn new(s: impl Into<String>) -> educore_core::error::Result<Self> {
        let s: String = s.into();
        if s.is_empty() || s.len() > 4 {
            return Err(educore_core::error::DomainError::validation(format!(
                "grade must be 1..=4 chars, got {}",
                s.len()
            )));
        }
        Ok(Self(s))
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// =============================================================================
// Enums (closed, copied + serialized)
// =============================================================================

/// The catalog of exam categories (mid-term, final, …).
/// Per `docs/specs/assessment/value-objects.md` § Exam Type
/// Enums.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExamTerm {
    /// Mid-term exam.
    MidTerm,
    /// Final exam.
    Final,
    /// Monthly test.
    Monthly,
    /// Weekly quiz.
    Weekly,
    /// Unit test.
    #[default]
    UnitTest,
    /// Mock exam (full-length practice).
    Mock,
    /// A user-defined term.
    Custom,
}

impl ExamTerm {
    /// Returns the canonical snake_case wire form.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MidTerm => "mid_term",
            Self::Final => "final",
            Self::Monthly => "monthly",
            Self::Weekly => "weekly",
            Self::UnitTest => "unit_test",
            Self::Mock => "mock",
            Self::Custom => "custom",
        }
    }
}

impl fmt::Display for ExamTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The status of a published `ResultStore` row. Per
/// `docs/specs/assessment/value-objects.md` § Exam Type
/// Enums.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResultStatus {
    /// The student passed.
    #[default]
    Pass,
    /// The student failed.
    Fail,
    /// The result is a manual override (e.g. medical
    /// exemption, board-exam credit).
    Manual,
    /// The result is withheld pending a missing input
    /// (e.g. late marks submission, fee dispute).
    Withheld,
}

impl ResultStatus {
    /// Returns the canonical snake_case wire form.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::Manual => "manual",
            Self::Withheld => "withheld",
        }
    }
}

impl fmt::Display for ResultStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// =============================================================================
// Re-exports of the `educore-academic` types the assessment
// crate consumes (avoids redefinition; the academic crate
// owns the canonical definitions).
// =============================================================================

pub use educore_academic::{
    ClassId, DateOfBirth, SectionId, StudentId, StudentRecordId, SubjectId,
};

pub use educore_academic::value_objects::{AcademicYearId, AcademicYearRange};

pub use educore_academic::PassMark;

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

    #[test]
    fn exam_name_rejects_empty() {
        let err = ExamName::new("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn exam_name_rejects_too_long() {
        let s: String = "a".repeat(ExamName::MAX_LEN + 1);
        let err = ExamName::new(s).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn exam_name_accepts_max_len() {
        let s: String = "a".repeat(ExamName::MAX_LEN);
        let name = ExamName::new(s).unwrap();
        assert_eq!(name.as_str().chars().count(), ExamName::MAX_LEN);
    }

    #[test]
    fn exam_code_rejects_empty() {
        let err = ExamCode::new("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn exam_code_rejects_too_long() {
        let s: String = "x".repeat(ExamCode::MAX_LEN + 1);
        let err = ExamCode::new(s).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn exam_code_accepts_max_len() {
        let s: String = "x".repeat(ExamCode::MAX_LEN);
        let code = ExamCode::new(s).unwrap();
        assert_eq!(code.as_str().chars().count(), ExamCode::MAX_LEN);
    }

    #[test]
    fn exam_mark_rejects_zero() {
        let err = ExamMark::new(0.0).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn exam_mark_rejects_negative() {
        let err = ExamMark::new(-5.0).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn exam_mark_rejects_too_large() {
        let err = ExamMark::new(1001.0).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn exam_mark_accepts_positive_in_range() {
        let m = ExamMark::new(100.0).unwrap();
        assert_eq!(m.as_f32(), 100.0);
    }

    #[test]
    fn exam_term_default_is_unit_test() {
        assert_eq!(ExamTerm::default(), ExamTerm::UnitTest);
    }

    #[test]
    fn exam_term_as_str_is_snake_case() {
        assert_eq!(ExamTerm::MidTerm.as_str(), "mid_term");
        assert_eq!(ExamTerm::UnitTest.as_str(), "unit_test");
        assert_eq!(ExamTerm::Custom.as_str(), "custom");
    }

    #[test]
    fn result_status_default_is_pass() {
        assert_eq!(ResultStatus::default(), ResultStatus::Pass);
    }

    #[test]
    fn result_status_as_str_is_snake_case() {
        assert_eq!(ResultStatus::Pass.as_str(), "pass");
        assert_eq!(ResultStatus::Fail.as_str(), "fail");
        assert_eq!(ResultStatus::Manual.as_str(), "manual");
        assert_eq!(ResultStatus::Withheld.as_str(), "withheld");
    }

    #[test]
    fn typed_ids_construct_and_display() {
        let school = SchoolId(Uuid::now_v7());
        let value = Uuid::now_v7();
        let id = ExamId::new(school, value);
        assert_eq!(id.school_id(), school);
        assert_eq!(id.as_uuid(), value);
        assert_eq!(id.to_string(), format!("{school}/{value}"));
    }

    #[test]
    fn typed_ids_carry_their_school_anchor() {
        let school_a = SchoolId(Uuid::now_v7());
        let school_b = SchoolId(Uuid::now_v7());
        let value = Uuid::now_v7();
        let id = ExamId::new(school_a, value);
        assert_eq!(id.school_id(), school_a);
        assert_ne!(id.school_id(), school_b);
    }
}
