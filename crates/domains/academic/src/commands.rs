//! # Academic-domain commands
//!
//! Every command in the academic domain carries a
//! [`TenantContext`] (school + actor + correlation) and is
//! rejected if the actor lacks the required capability.
//! The capability check itself runs at the dispatcher layer
//! (matching the platform crate's pattern); the command
//! shape carries the inputs the service function needs to
//! mutate the aggregate.
//!
//! Per `docs/specs/academic/commands.md` and the engine
//! rule "compile-time safety over strings", the command
//! shapes use typed ids and value objects, not `String`
//! fields. Phase 3 ships the prompt-named subset: the
//! `Student` lifecycle (admit, update profile, suspend,
//! reinstate, withdraw, transfer, promote, graduate), the
//! `Class` / `Section` / `Subject` / `AcademicYear` CRUD
//! commands. The full command catalog in
//! `docs/commands/academic.md` lands in later phases.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use educore_core::ids::SchoolId;
use educore_core::tenant::TenantContext;

use crate::value_objects::{
    AcademicYearId, AcademicYearRange, ClassId, ResultStatus, SectionId, StudentId, SubjectId,
};

// =============================================================================
// Uniqueness checker (port)
// =============================================================================

/// A read-only uniqueness check the academic services use to
/// enforce per-school uniqueness constraints on
/// `Student.admission_no` and `Student.email` (when supplied).
///
/// The check is **pure** (no I/O): the production caller wires
/// it to a thin adapter over the storage port that returns
/// `true` if a row with the given key already exists; the
/// test caller wires it to an in-memory collection.
///
/// The trait is `Send + Sync` so the production wiring can
/// hold an `Arc<dyn UniquenessChecker>` and share it across
/// worker threads.
pub trait UniquenessChecker: Send + Sync {
    /// Returns `true` if a student with the given admission
    /// number already exists in the school.
    fn student_admission_no_exists(&self, school: SchoolId, admission_no: &str) -> bool;
    /// Returns `true` if a student with the given email
    /// already exists in the school. The check is
    /// case-insensitive; the caller is responsible for
    /// lowercasing before the call.
    fn student_email_exists(&self, school: SchoolId, email: &str) -> bool;
}

// =============================================================================
// Student commands (8)
// =============================================================================

/// Command: admit a new student.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdmitStudentCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The new student's typed id.
    pub student_id: StudentId,
    /// The student's admission number (1..=50 chars, unique
    /// within the school).
    pub admission_no: String,
    /// The student's first name (1..=200 chars).
    pub first_name: String,
    /// The student's last name (1..=200 chars).
    pub last_name: String,
    /// The student's date of birth (must imply age 2..=30).
    pub date_of_birth: NaiveDate,
    /// The student's gender.
    pub gender: crate::value_objects::Gender,
    /// Optional blood group.
    pub blood_group: Option<crate::value_objects::BloodGroup>,
    /// Optional religion (free-form, 1..=100 chars).
    pub religion: Option<String>,
    /// Optional caste (free-form, 1..=100 chars).
    pub caste: Option<String>,
    /// Optional mobile phone (E.164).
    pub mobile: Option<String>,
    /// Optional email (validated, lowercased).
    pub email: Option<String>,
    /// Optional current address.
    pub current_address: Option<String>,
    /// Optional permanent address.
    pub permanent_address: Option<String>,
    /// The admission date.
    pub admission_date: NaiveDate,
    /// The class the student is admitted into.
    pub class_id: ClassId,
    /// The section the student is admitted into.
    pub section_id: SectionId,
    /// The academic year the admission applies to.
    pub academic_year_id: AcademicYearId,
    /// Optional initial roll number.
    pub roll_no: Option<String>,
    /// Optional custom fields.
    pub custom_fields: std::collections::BTreeMap<String, String>,
}

impl AdmitStudentCommand {
    /// Convenience constructor for tests and bootstrapping.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        tenant: TenantContext,
        student_id: StudentId,
        admission_no: String,
        first_name: String,
        last_name: String,
        date_of_birth: NaiveDate,
        gender: crate::value_objects::Gender,
        admission_date: NaiveDate,
        class_id: ClassId,
        section_id: SectionId,
        academic_year_id: AcademicYearId,
    ) -> Self {
        Self {
            tenant,
            student_id,
            admission_no,
            first_name,
            last_name,
            date_of_birth,
            gender,
            blood_group: None,
            religion: None,
            caste: None,
            mobile: None,
            email: None,
            current_address: None,
            permanent_address: None,
            admission_date,
            class_id,
            section_id,
            academic_year_id,
            roll_no: None,
            custom_fields: std::collections::BTreeMap::new(),
        }
    }

    /// Returns the school id (taken from the typed id).
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.student_id.school_id()
    }
}

/// Command: update a student's mutable profile fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateStudentProfileCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The student's typed id.
    pub student_id: StudentId,
    /// Optional new first name. `None` means "do not change".
    pub first_name: Option<String>,
    /// Optional new last name. `None` means "do not change".
    pub last_name: Option<String>,
    /// Optional new gender. `None` means "do not change".
    pub gender: Option<crate::value_objects::Gender>,
    /// Optional new mobile. Outer `None` means "do not
    /// change"; outer `Some(None)` means "clear the mobile".
    pub mobile: Option<Option<String>>,
    /// Optional new email. Outer `None` means "do not change".
    pub email: Option<Option<String>>,
    /// Optional new current address.
    pub current_address: Option<Option<String>>,
    /// Optional new permanent address.
    pub permanent_address: Option<Option<String>>,
    /// Optional custom-fields patch.
    pub custom_fields: Option<std::collections::BTreeMap<String, String>>,
}

/// Command: suspend a student.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuspendStudentCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The student's typed id.
    pub student_id: StudentId,
    /// The reason for the suspension (1..=500 chars).
    pub reason: String,
    /// The first day the suspension is effective.
    pub effective_from: NaiveDate,
    /// The expected return date (optional).
    pub expected_return: Option<NaiveDate>,
}

/// Command: reinstate a suspended student.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReinstateStudentCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The student's typed id.
    pub student_id: StudentId,
    /// The first day the reinstatement is effective.
    pub effective_from: NaiveDate,
    /// Optional note.
    pub note: Option<String>,
}

/// Command: withdraw a student.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WithdrawStudentCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The student's typed id.
    pub student_id: StudentId,
    /// The reason for the withdrawal (1..=500 chars).
    pub reason: String,
    /// The first day the withdrawal is effective.
    pub effective_from: NaiveDate,
    /// Optional note.
    pub note: Option<String>,
}

/// Command: transfer a student to another school.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferStudentCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The student's typed id.
    pub student_id: StudentId,
    /// The destination school's typed id.
    pub destination_school_id: SchoolId,
    /// The reason for the transfer (1..=500 chars).
    pub reason: String,
    /// The first day the transfer is effective.
    pub effective_from: NaiveDate,
}

/// Command: promote a student to the next academic year.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromoteStudentCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The student's typed id.
    pub student_id: StudentId,
    /// The source academic year.
    pub from_academic_year_id: AcademicYearId,
    /// The target academic year.
    pub to_academic_year_id: AcademicYearId,
    /// The target class.
    pub to_class_id: ClassId,
    /// The target section.
    pub to_section_id: SectionId,
    /// The new roll number.
    pub to_roll_no: String,
    /// The promotion result.
    pub result_status: ResultStatus,
}

/// Command: graduate a student.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraduateStudentCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The student's typed id.
    pub student_id: StudentId,
    /// The academic year the student graduates in.
    pub academic_year_id: AcademicYearId,
    /// The graduation date.
    pub graduation_date: NaiveDate,
}

// =============================================================================
// Class commands (4)
// =============================================================================

/// Command: create a new class.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateClassCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The new class's typed id.
    pub class_id: ClassId,
    /// The class's display name (1..=200 chars).
    pub class_name: String,
    /// The class's pass mark (0.0..=100.0).
    pub pass_mark: f32,
}

/// Command: update a class's mutable fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateClassCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The class's typed id.
    pub class_id: ClassId,
    /// Optional new class name.
    pub class_name: Option<String>,
    /// Optional new pass mark.
    pub pass_mark: Option<f32>,
}

/// Command: set a class's optional-subject GPA threshold.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetOptionalSubjectGpaThresholdCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The class's typed id.
    pub class_id: ClassId,
    /// The new threshold (0.0..=5.0).
    pub threshold: f32,
}

/// Command: delete a class (soft-delete; existing references remain).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteClassCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The class's typed id.
    pub class_id: ClassId,
}

// =============================================================================
// Section commands (3)
// =============================================================================

/// Command: create a new section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateSectionCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The new section's typed id.
    pub section_id: SectionId,
    /// The section's display name (1..=200 chars).
    pub section_name: String,
}

/// Command: update a section's mutable fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateSectionCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The section's typed id.
    pub section_id: SectionId,
    /// Optional new section name.
    pub section_name: Option<String>,
}

/// Command: delete a section (soft-delete; existing references remain).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteSectionCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The section's typed id.
    pub section_id: SectionId,
}

// =============================================================================
// Subject commands (3)
// =============================================================================

/// Command: create a new subject.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateSubjectCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The new subject's typed id.
    pub subject_id: SubjectId,
    /// The subject's code (1..=50 chars, unique within school).
    pub subject_code: String,
    /// The subject's display name.
    pub subject_name: String,
    /// The subject's type.
    pub subject_type: crate::value_objects::SubjectType,
    /// The subject's pass mark (0.0..=100.0).
    pub pass_mark: f32,
}

/// Command: update a subject's mutable fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateSubjectCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The subject's typed id.
    pub subject_id: SubjectId,
    /// Optional new name.
    pub subject_name: Option<String>,
    /// Optional new subject type.
    pub subject_type: Option<crate::value_objects::SubjectType>,
    /// Optional new pass mark.
    pub pass_mark: Option<f32>,
}

/// Command: delete a subject.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteSubjectCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The subject's typed id.
    pub subject_id: SubjectId,
}

// =============================================================================
// AcademicYear commands (5)
// =============================================================================

/// Command: create a new academic year.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateAcademicYearCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The new academic year's typed id.
    pub academic_year_id: AcademicYearId,
    /// The short label (e.g. "2026").
    pub year: String,
    /// The display title.
    pub title: String,
    /// The start date.
    pub starting_date: NaiveDate,
    /// The end date.
    pub ending_date: NaiveDate,
    /// Whether this is the current academic year.
    pub is_current: bool,
    /// Optional source academic year for deep-copy.
    pub copy_with_academic_year: Option<AcademicYearId>,
}

impl CreateAcademicYearCommand {
    /// Returns the date range as a typed object (after
    /// validating start < end).
    pub fn range(&self) -> educore_core::error::Result<AcademicYearRange> {
        AcademicYearRange::new(self.starting_date, self.ending_date)
    }
}

/// Command: update an academic year's date range.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateAcademicYearDatesCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The academic year's typed id.
    pub academic_year_id: AcademicYearId,
    /// The new start date.
    pub starting_date: NaiveDate,
    /// The new end date.
    pub ending_date: NaiveDate,
}

/// Command: set a new current academic year.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetCurrentAcademicYearCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The new current academic year's typed id.
    pub academic_year_id: AcademicYearId,
}

/// Command: close an academic year (make it read-only).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloseAcademicYearCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The academic year's typed id.
    pub academic_year_id: AcademicYearId,
}

// =============================================================================
// Internal: shared validation helpers
// =============================================================================

pub(crate) fn validate_first_name(name: &str) -> educore_core::error::Result<()> {
    use educore_core::error::DomainError;
    if name.is_empty() {
        return Err(DomainError::validation("first name must not be empty"));
    }
    if name.chars().count() > 200 {
        return Err(DomainError::validation(format!(
            "first name must be at most 200 chars, got {}",
            name.chars().count()
        )));
    }
    Ok(())
}

pub(crate) fn validate_last_name(name: &str) -> educore_core::error::Result<()> {
    use educore_core::error::DomainError;
    if name.is_empty() {
        return Err(DomainError::validation("last name must not be empty"));
    }
    if name.chars().count() > 200 {
        return Err(DomainError::validation(format!(
            "last name must be at most 200 chars, got {}",
            name.chars().count()
        )));
    }
    Ok(())
}

pub(crate) fn validate_admission_no(s: &str) -> educore_core::error::Result<()> {
    use educore_core::error::DomainError;
    if s.is_empty() {
        return Err(DomainError::validation(
            "admission number must not be empty",
        ));
    }
    if s.chars().count() > 50 {
        return Err(DomainError::validation(format!(
            "admission number must be at most 50 chars, got {}",
            s.chars().count()
        )));
    }
    Ok(())
}

pub(crate) fn validate_class_name(s: &str) -> educore_core::error::Result<()> {
    use educore_core::error::DomainError;
    if s.is_empty() {
        return Err(DomainError::validation("class name must not be empty"));
    }
    if s.chars().count() > 200 {
        return Err(DomainError::validation(format!(
            "class name must be at most 200 chars, got {}",
            s.chars().count()
        )));
    }
    Ok(())
}

pub(crate) fn validate_section_name(s: &str) -> educore_core::error::Result<()> {
    use educore_core::error::DomainError;
    if s.is_empty() {
        return Err(DomainError::validation("section name must not be empty"));
    }
    if s.chars().count() > 200 {
        return Err(DomainError::validation(format!(
            "section name must be at most 200 chars, got {}",
            s.chars().count()
        )));
    }
    Ok(())
}

pub(crate) fn validate_subject_code(s: &str) -> educore_core::error::Result<()> {
    use educore_core::error::DomainError;
    if s.is_empty() {
        return Err(DomainError::validation("subject code must not be empty"));
    }
    if s.chars().count() > 50 {
        return Err(DomainError::validation(format!(
            "subject code must be at most 50 chars, got {}",
            s.chars().count()
        )));
    }
    Ok(())
}

pub(crate) fn validate_subject_name(s: &str) -> educore_core::error::Result<()> {
    use educore_core::error::DomainError;
    if s.is_empty() {
        return Err(DomainError::validation("subject name must not be empty"));
    }
    if s.chars().count() > 200 {
        return Err(DomainError::validation(format!(
            "subject name must be at most 200 chars, got {}",
            s.chars().count()
        )));
    }
    Ok(())
}

pub(crate) fn validate_year_label(s: &str) -> educore_core::error::Result<()> {
    use educore_core::error::DomainError;
    if s.is_empty() {
        return Err(DomainError::validation("year label must not be empty"));
    }
    if s.chars().count() > 20 {
        return Err(DomainError::validation(format!(
            "year label must be at most 20 chars, got {}",
            s.chars().count()
        )));
    }
    Ok(())
}

pub(crate) fn validate_year_title(s: &str) -> educore_core::error::Result<()> {
    use educore_core::error::DomainError;
    if s.is_empty() {
        return Err(DomainError::validation(
            "academic year title must not be empty",
        ));
    }
    if s.chars().count() > 200 {
        return Err(DomainError::validation(format!(
            "academic year title must be at most 200 chars, got {}",
            s.chars().count()
        )));
    }
    Ok(())
}

pub(crate) fn validate_email_optional(s: &str) -> educore_core::error::Result<()> {
    use educore_core::error::DomainError;
    if s.is_empty() {
        return Err(DomainError::validation("email must not be empty"));
    }
    if s.chars().count() > 200 {
        return Err(DomainError::validation(format!(
            "email must be at most 200 chars, got {}",
            s.chars().count()
        )));
    }
    if !s.contains('@') {
        return Err(DomainError::validation(format!("email missing '@': {s:?}")));
    }
    Ok(())
}

pub(crate) fn validate_mobile_optional(s: &str) -> educore_core::error::Result<()> {
    use educore_core::error::DomainError;
    if s.is_empty() {
        return Err(DomainError::validation("mobile number must not be empty"));
    }
    if !s.starts_with('+') {
        return Err(DomainError::validation(format!(
            "mobile number must start with '+': {s:?}"
        )));
    }
    let digits = &s[1..];
    if digits.len() < 4 || digits.len() > 15 {
        return Err(DomainError::validation(format!(
            "mobile number digit count {} outside 4..=15",
            digits.len()
        )));
    }
    if !digits.chars().all(|c| c.is_ascii_digit()) {
        return Err(DomainError::validation(format!(
            "mobile number contains non-digit characters: {s:?}"
        )));
    }
    Ok(())
}

pub(crate) fn validate_pass_mark(v: f32) -> educore_core::error::Result<()> {
    use educore_core::error::DomainError;
    if !(0.0..=100.0).contains(&v) {
        return Err(DomainError::validation(format!(
            "pass mark {v} must be in 0.0..=100.0"
        )));
    }
    Ok(())
}

pub(crate) fn validate_gpa_threshold(v: f32) -> educore_core::error::Result<()> {
    use educore_core::error::DomainError;
    if !(0.0..=5.0).contains(&v) {
        return Err(DomainError::validation(format!(
            "optional subject GPA threshold {v} must be in 0.0..=5.0"
        )));
    }
    Ok(())
}

pub(crate) fn validate_suspension_reason(s: &str) -> educore_core::error::Result<()> {
    use educore_core::error::DomainError;
    if s.is_empty() {
        return Err(DomainError::validation(
            "suspension reason must not be empty",
        ));
    }
    if s.chars().count() > 500 {
        return Err(DomainError::validation(format!(
            "suspension reason must be at most 500 chars, got {}",
            s.chars().count()
        )));
    }
    Ok(())
}

pub(crate) fn validate_withdrawal_reason(s: &str) -> educore_core::error::Result<()> {
    use educore_core::error::DomainError;
    if s.is_empty() {
        return Err(DomainError::validation(
            "withdrawal reason must not be empty",
        ));
    }
    if s.chars().count() > 500 {
        return Err(DomainError::validation(format!(
            "withdrawal reason must be at most 500 chars, got {}",
            s.chars().count()
        )));
    }
    Ok(())
}

pub(crate) fn validate_transfer_reason(s: &str) -> educore_core::error::Result<()> {
    use educore_core::error::DomainError;
    if s.is_empty() {
        return Err(DomainError::validation("transfer reason must not be empty"));
    }
    if s.chars().count() > 500 {
        return Err(DomainError::validation(format!(
            "transfer reason must be at most 500 chars, got {}",
            s.chars().count()
        )));
    }
    Ok(())
}

pub(crate) fn validate_roll_no(s: &str) -> educore_core::error::Result<()> {
    use educore_core::error::DomainError;
    if s.is_empty() {
        return Err(DomainError::validation("roll number must not be empty"));
    }
    if s.chars().count() > 50 {
        return Err(DomainError::validation(format!(
            "roll number must be at most 50 chars, got {}",
            s.chars().count()
        )));
    }
    Ok(())
}

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
    use educore_core::tenant::{TenantContext, UserType};

    fn ctx() -> TenantContext {
        let g = SystemIdGen;
        TenantContext::for_user(
            g.next_school_id(),
            g.next_user_id(),
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        )
    }

    #[test]
    fn admit_student_command_minimal_constructor() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let student_id = StudentId::new(school, g.next_uuid());
        let class = ClassId::new(school, g.next_uuid());
        let section = SectionId::new(school, g.next_uuid());
        let year = AcademicYearId::new(school, g.next_uuid());
        let cmd = AdmitStudentCommand::new(
            ctx(),
            student_id,
            "ADM-001".to_owned(),
            "Ada".to_owned(),
            "Lovelace".to_owned(),
            chrono::NaiveDate::from_ymd_opt(2016, 1, 1).unwrap(),
            crate::value_objects::Gender::Female,
            chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            class,
            section,
            year,
        );
        assert_eq!(cmd.admission_no, "ADM-001");
        assert_eq!(cmd.roll_no, None);
        assert!(cmd.custom_fields.is_empty());
        assert_eq!(cmd.school_id(), school);
        assert_eq!(cmd.student_id.school_id(), school);
    }

    #[test]
    fn validate_first_name_rejects_empty_and_overlong() {
        assert!(validate_first_name("").is_err());
        assert!(validate_first_name("Ada").is_ok());
        assert!(validate_first_name(&"a".repeat(201)).is_err());
    }

    #[test]
    fn validate_admission_no_rejects_empty_and_overlong() {
        assert!(validate_admission_no("").is_err());
        assert!(validate_admission_no("ADM-001").is_ok());
        assert!(validate_admission_no(&"a".repeat(51)).is_err());
    }

    #[test]
    fn validate_class_section_subject_name() {
        assert!(validate_class_name("").is_err());
        assert!(validate_class_name("Grade 1").is_ok());
        assert!(validate_section_name("").is_err());
        assert!(validate_section_name("A").is_ok());
        assert!(validate_subject_code("").is_err());
        assert!(validate_subject_code("MATH").is_ok());
        assert!(validate_subject_name("").is_err());
        assert!(validate_subject_name("Mathematics").is_ok());
    }

    #[test]
    fn email_optional_validates() {
        assert!(validate_email_optional("").is_err());
        assert!(validate_email_optional("ada@example.com").is_ok());
        assert!(validate_email_optional("no-at-sign").is_err());
    }

    #[test]
    fn mobile_optional_validates() {
        assert!(validate_mobile_optional("").is_err());
        assert!(validate_mobile_optional("+14155552671").is_ok());
        assert!(validate_mobile_optional("14155552671").is_err());
        assert!(validate_mobile_optional("+abc").is_err());
    }

    #[test]
    fn validate_pass_mark_and_gpa_threshold() {
        assert!(validate_pass_mark(0.0).is_ok());
        assert!(validate_pass_mark(100.0).is_ok());
        assert!(validate_pass_mark(-0.1).is_err());
        assert!(validate_pass_mark(100.1).is_err());
        assert!(validate_gpa_threshold(0.0).is_ok());
        assert!(validate_gpa_threshold(5.0).is_ok());
        assert!(validate_gpa_threshold(-0.1).is_err());
        assert!(validate_gpa_threshold(5.1).is_err());
    }

    #[test]
    fn validate_reasons_and_roll_no() {
        assert!(validate_suspension_reason("").is_err());
        assert!(validate_suspension_reason("medical").is_ok());
        assert!(validate_withdrawal_reason("").is_err());
        assert!(validate_withdrawal_reason("moved").is_ok());
        assert!(validate_transfer_reason("").is_err());
        assert!(validate_transfer_reason("parent's job").is_ok());
        assert!(validate_roll_no("").is_err());
        assert!(validate_roll_no("1").is_ok());
    }

    #[test]
    fn validate_year_label_and_title() {
        assert!(validate_year_label("").is_err());
        assert!(validate_year_label("2026").is_ok());
        assert!(validate_year_label(&"a".repeat(21)).is_err());
        assert!(validate_year_title("").is_err());
        assert!(validate_year_title("Academic Year 2026-2027").is_ok());
        assert!(validate_year_title(&"a".repeat(201)).is_err());
    }

    #[test]
    fn create_academic_year_command_range_constructs_typed_range() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let id = AcademicYearId::new(school, g.next_uuid());
        let cmd = CreateAcademicYearCommand {
            tenant: ctx(),
            academic_year_id: id,
            year: "2026".to_owned(),
            title: "Academic Year 2026-2027".to_owned(),
            starting_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            ending_date: chrono::NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
            is_current: true,
            copy_with_academic_year: None,
        };
        let range = cmd.range().unwrap();
        assert_eq!(
            range.start,
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
        );
        assert_eq!(
            range.end,
            chrono::NaiveDate::from_ymd_opt(2027, 1, 1).unwrap()
        );
    }
}
