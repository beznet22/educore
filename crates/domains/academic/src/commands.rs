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

use educore_core::ids::{SchoolId, UserId};
use educore_rbac::value_objects::Capability;
use educore_core::tenant::TenantContext;

use crate::value_objects::{
    AcademicYearId, AcademicYearRange, CertificateId, ClassId, ClassRoomId, ClassRoutineId,
    ClassSectionId, ClassSubjectId, DayOfWeek, EmailAddress, FileId, GuardianId, HomeworkId,
    IdCardId, LessonId, LessonPlanId, LessonTopicId, OptionalSubjectAssignmentId, PhoneNumber,
    RegistrationFieldId, Relation, ResultStatus, SectionId, StudentCategoryId, StudentGroupId,
    StudentGuardianLinkId, StudentId, StudentPromotionId, SubjectId, ClassPeriod, ClassTimeId,
};

// =============================================================================
// Uniqueness checker (port)
// =============================================================================

/// A read-only uniqueness check the academic services use to
/// enforce per-school uniqueness constraints on
/// `Student.admission_no`, `Student.email`, class name,
/// section name, subject code, roll number within
/// `(school, class, section, year)`, and academic-year
/// overlap.
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
    /// Returns `true` if a roll number is already taken in
    /// the school for the given `(class, section,
    /// academic_year)`. Per Student I-3.
    fn roll_no_exists(
        &self,
        school: SchoolId,
        class_id: ClassId,
        section_id: SectionId,
        academic_year_id: AcademicYearId,
        roll_no: &str,
    ) -> bool;
    /// Returns `true` if a class with the given name
    /// already exists in the school. Per Class I-2.
    fn class_name_exists(&self, school: SchoolId, name: &str) -> bool;
    /// Returns `true` if a section with the given name
    /// already exists in the school. Per Section I-1.
    fn section_name_exists(&self, school: SchoolId, name: &str) -> bool;
    /// Returns `true` if a subject with the given code
    /// already exists in the school. Per Subject I-1.
    fn subject_code_exists(&self, school: SchoolId, code: &str) -> bool;
    /// Returns `true` if any academic year in the school
    /// has a date range that overlaps the given range. Per
    /// AcademicYear I-2.
    fn academic_year_overlaps(
        &self,
        school: SchoolId,
        range: AcademicYearRange,
        exclude_id: Option<AcademicYearId>,
    ) -> bool;
    /// Returns `true` if the student already has an optional
    /// subject assigned for the given academic year. Per
    /// Student I-4.
    fn optional_subject_assigned_exists(
        &self,
        school: SchoolId,
        student_id: StudentId,
        academic_year_id: AcademicYearId,
    ) -> bool;
    /// Returns `true` if a primary `StudentGuardianLink`
    /// already exists for the given student. Per Guardian
    /// I-4.
    fn primary_guardian_link_exists(
        &self,
        school: SchoolId,
        student_id: StudentId,
    ) -> bool;
    /// Returns `true` if a `ClassSection` already exists
    /// for the given `(class, section, academic_year)` in
    /// the school. Per ClassSection I-1.
    fn class_section_exists(
        &self,
        school: SchoolId,
        class_id: ClassId,
        section_id: SectionId,
        academic_year_id: AcademicYearId,
    ) -> bool;
    /// Returns `true` if any `StudentRecord` references the
    /// given `ClassSection`. Per ClassSection I-4
    /// (cannot delete a class-section while student records
    /// reference it).
    fn class_section_has_student_records(
        &self,
        school: SchoolId,
        class_section_id: ClassSectionId,
    ) -> bool;
    /// Returns `true` if the teacher (`UserId`) already has
    /// another period scheduled in the school at the same
    /// `(day, period_number)` slot. Per ClassRoutine I-4
    /// (a teacher cannot be in two places at the same
    /// time).
    fn teacher_has_conflict(
        &self,
        school: SchoolId,
        teacher_id: UserId,
        day: DayOfWeek,
        period_number: u8,
    ) -> bool;
    /// Returns `true` if the room (`ClassRoomId`) already
    /// hosts another period in the school at the same
    /// `(day, period_number)` slot. Per ClassRoutine I-5
    /// (a room cannot host two classes at the same time).
    fn room_has_conflict(
        &self,
        school: SchoolId,
        room_id: ClassRoomId,
        day: DayOfWeek,
        period_number: u8,
    ) -> bool;
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


impl UpdateStudentProfileCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicStudentUpdate]
    }
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


impl SuspendStudentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicStudentUpdate]
    }
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


impl ReinstateStudentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicStudentUpdate]
    }
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


impl WithdrawStudentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicStudentUpdate]
    }
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


impl TransferStudentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicStudentUpdate]
    }
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


impl PromoteStudentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicStudentUpdate]
    }
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


impl GraduateStudentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicStudentUpdate]
    }
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


impl CreateClassCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicClassCreate]
    }
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


impl UpdateClassCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicClassUpdate]
    }
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


impl SetOptionalSubjectGpaThresholdCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicClassUpdate]
    }
}
/// Command: delete a class (soft-delete; existing references remain).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteClassCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The class's typed id.
    pub class_id: ClassId,
}


impl DeleteClassCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicClassDelete]
    }
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


impl CreateSectionCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicClassCreate]
    }
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


impl UpdateSectionCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicClassUpdate]
    }
}
/// Command: delete a section (soft-delete; existing references remain).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteSectionCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The section's typed id.
    pub section_id: SectionId,
}


impl DeleteSectionCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicClassDelete]
    }
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


impl CreateSubjectCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicClassCreate]
    }
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


impl UpdateSubjectCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicClassUpdate]
    }
}
/// Command: delete a subject.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteSubjectCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The subject's typed id.
    pub subject_id: SubjectId,
}


impl DeleteSubjectCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicClassDelete]
    }
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


impl UpdateAcademicYearDatesCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicAcademicYear]
    }
}
/// Command: set a new current academic year.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetCurrentAcademicYearCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The new current academic year's typed id.
    pub academic_year_id: AcademicYearId,
}


impl SetCurrentAcademicYearCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicAcademicYear]
    }
}
/// Command: close an academic year (make it read-only).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloseAcademicYearCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The academic year's typed id.
    pub academic_year_id: AcademicYearId,
}


impl CloseAcademicYearCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicAcademicYear]
    }
}
// =============================================================================
// Placeholder commands for the remaining 14 academic aggregates.
//
// Minimal `id + school_id` stubs so the spec is exhaustively
// representable in code. The full impl (capability check,
// domain fields, invariants, events) lands in subsequent
// workstreams per `docs/build-plan.md`.
//
// Each stub uses the typed id from `crate::value_objects` so the
// type system catches cross-tenant confusion at compile time
// (the `school_id` is derived from `id.school_id()` in real impl;
// it is held explicitly here so the placeholder round-trips
// through Serde without depending on a `Default::default` for
// the id).
// =============================================================================

macro_rules! academic_command_stub {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident {
            id: $id_ty:ty $(,)?
        }
    ) => {
        $(#[$attr])*
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        $vis struct $name {
            /// The typed id (school_id + uuid).
            pub id: $id_ty,
            /// The owning school (derived from `id.school_id()` in
            /// real impl; held explicitly here so the placeholder
            /// is self-contained).
            pub school_id: SchoolId,
        }
    };
}

// =============================================================================
// Guardian commands (full impl — Batch 1)
// =============================================================================

/// Command: register a new guardian.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterGuardianCommand {
    /// The guardian's typed id.
    pub guardian_id: GuardianId,
    /// The guardian's first name (1..=200 chars).
    pub first_name: String,
    /// The guardian's last name (1..=200 chars).
    pub last_name: String,
    /// Optional phone number of record (validated). Per I-1.
    pub phone: Option<PhoneNumber>,
    /// Optional email of record (validated). Per I-1.
    pub email: Option<EmailAddress>,
}


impl RegisterGuardianCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicStudentUpdate]
    }
}

/// Command: link a guardian to a student.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkGuardianToStudentCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The link's typed id.
    pub link_id: StudentGuardianLinkId,
    /// The guardian being linked.
    pub guardian_id: GuardianId,
    /// The student being linked.
    pub student_id: StudentId,
    /// The relationship.
    pub relation: Relation,
    /// Whether this guardian is the primary contact for the
    /// student. Per Guardian I-4, at most one guardian per
    /// student may be marked primary.
    pub is_primary: bool,
}


impl LinkGuardianToStudentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicStudentUpdate]
    }
}

/// Command: unlink a guardian from a student.
///
/// Per Guardian I-5, when the last link is removed the
/// guardian is soft-deleted (`active_status = Retired`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnlinkGuardianFromStudentCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The link's typed id.
    pub link_id: StudentGuardianLinkId,
}


impl UnlinkGuardianFromStudentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicStudentUpdate]
    }
}

/// Command: mark a guardian link as primary.
///
/// Per Guardian I-4, at most one guardian per student may
/// be marked primary. The implementation demotes any
/// previously-primary link for the same student in the
/// same transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkPrimaryGuardianCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The link's typed id.
    pub link_id: StudentGuardianLinkId,
}


impl MarkPrimaryGuardianCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicStudentUpdate]
    }
}

/// Command: update a guardian's mutable contact fields
/// (phone, email). Per Guardian I-1 the guardian carries
/// at most one phone and one email of record; the
/// `Option<Option<…>>` shape distinguishes "do not change"
/// (outer `None`) from "clear the field" (outer `Some(None)`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateGuardianContactCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The guardian's typed id.
    pub guardian_id: GuardianId,
    /// Optional new phone. Outer `None` means "do not change";
    /// outer `Some(None)` means "clear the phone"; outer
    /// `Some(Some(p))` means "set the phone to `p`". The
    /// validator on `PhoneNumber::new` enforces the E.164
    /// format before the service accepts the value.
    pub phone: Option<Option<PhoneNumber>>,
    /// Optional new email. Same outer/inner convention as
    /// `phone`.
    pub email: Option<Option<EmailAddress>>,
}


impl UpdateGuardianContactCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicStudentUpdate]
    }
}

/// Command: retire (soft-delete) a guardian explicitly.
///
/// Per Guardian I-5 a guardian is auto-retired when the
/// last student link is removed, but the dispatcher may
/// also retire an orphaned guardian on demand (e.g. when
/// the school decides the contact is no longer valid).
/// This command is the manual escape hatch for that flow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetireGuardianCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The guardian's typed id.
    pub guardian_id: GuardianId,
}


impl RetireGuardianCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicStudentUpdate]
    }
}
/// Command: create a [`ClassSection`](crate::aggregate::ClassSection)
/// pairing of a class, a section, and an academic year.
///
/// Per `docs/specs/academic/aggregates.md` § ClassSection,
/// the create flow enforces:
/// - I-1: the `(class, section, academic_year)` triple
///   must be unique within the school (checked via
///   `UniquenessChecker::class_section_exists`).
/// - I-3: `class_rooms` must contain at least one entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateClassSectionCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The class-section's typed id.
    pub class_section_id: ClassSectionId,
    /// The class this section is a division of.
    pub class_id: ClassId,
    /// The section within the class.
    pub section_id: SectionId,
    /// The academic year this pairing applies to.
    pub academic_year_id: AcademicYearId,
    /// The class rooms assigned to this section. Must be
    /// non-empty (I-3).
    pub class_rooms: Vec<ClassRoomId>,
}

impl CreateClassSectionCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicClassCreate]
    }
}
// =============================================================================
// ClassRoutine commands (Wave 51: full impl)
// =============================================================================

/// Command: create a [`ClassRoutine`](crate::aggregate::ClassRoutine)
/// for the given `class_section_id` and `academic_year_id`,
/// with the provided full-week period schedule.
///
/// Per `docs/specs/academic/aggregates.md` § ClassRoutine,
/// the create flow enforces:
///
/// - **I-1**: `periods` covers all 7 distinct days
///   (Mon-Sun). Enforced by
///   [`crate::aggregate::ClassRoutine::fresh`].
/// - **I-2**: no duplicate `ClassTimeId` within
///   `periods`. Enforced by
///   [`crate::aggregate::ClassRoutine::fresh`].
/// - **I-3**: every period carries both a `room_id` and
///   a `teacher_id` (compile-time enforced; the
///   `ClassPeriod` struct has no `None` slots for those
///   fields).
/// - **I-4**: teacher cannot be in two places at the
///   same time. Enforced via
///   `UniquenessChecker::teacher_has_conflict`.
/// - **I-5**: room cannot host two classes at the same
///   time. Enforced via
///   `UniquenessChecker::room_has_conflict`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateClassRoutineCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The class-routine's typed id.
    pub class_routine_id: ClassRoutineId,
    /// The class-section this routine is scheduled for.
    pub class_section_id: ClassSectionId,
    /// The academic year this routine applies to.
    pub academic_year_id: AcademicYearId,
    /// The full-week period schedule. Per I-1, must
    /// contain at least 7 periods covering all 7
    /// distinct days. Per I-2, no two periods may share a
    /// `class_time_id`.
    pub periods: Vec<ClassPeriod>,
}

impl CreateClassRoutineCommand {
    /// Returns the school id (taken from the typed id).
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.class_routine_id.school_id()
    }

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicClassRoutineCreate]
    }
}

/// Command: replace the periods of an existing
/// [`ClassRoutine`](crate::aggregate::ClassRoutine).
///
/// Per `docs/specs/academic/aggregates.md` § ClassRoutine,
/// the update flow enforces:
///
/// - **I-1**: `new_periods` covers all 7 distinct days.
/// - **I-2**: no duplicate `ClassTimeId` within
///   `new_periods`.
/// - **I-3**: every period carries both a `room_id` and
///   a `teacher_id`.
/// - **I-4** / **I-5**: teacher / room no-conflict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateClassRoutinePeriodCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The class-routine's typed id.
    pub class_routine_id: ClassRoutineId,
    /// The replacement period schedule. Per I-1, must
    /// cover all 7 distinct days. Per I-2, no two
    /// periods may share a `class_time_id`.
    pub new_periods: Vec<ClassPeriod>,
}

impl UpdateClassRoutinePeriodCommand {
    /// Returns the school id (taken from the typed id).
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.class_routine_id.school_id()
    }

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicClassRoutineUpdate]
    }
}

/// Command: swap two periods in an existing
/// [`ClassRoutine`](crate::aggregate::ClassRoutine) by
/// index. The indices must refer to valid positions in
/// the current `periods` vector (the service validates
/// the bounds).
///
/// The swap exchanges the entire [`ClassPeriod`]
/// payloads (including `room_id`, `teacher_id`, `day`,
/// `period_number`, `class_time_id`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwapClassRoutinePeriodsCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The class-routine's typed id.
    pub class_routine_id: ClassRoutineId,
    /// The index of the first period to swap.
    pub period_a_idx: usize,
    /// The index of the second period to swap.
    pub period_b_idx: usize,
}

impl SwapClassRoutinePeriodsCommand {
    /// Returns the school id (taken from the typed id).
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.class_routine_id.school_id()
    }

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicClassRoutineUpdate]
    }
}

/// Command: soft-delete (retire) an existing
/// [`ClassRoutine`](crate::aggregate::ClassRoutine).
///
/// Per `docs/specs/academic/aggregates.md` § ClassRoutine,
/// the delete flow is unconditional: any active
/// `ClassRoutine` may be deleted; the service retires
/// the aggregate and emits the typed event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteClassRoutineCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The class-routine's typed id.
    pub class_routine_id: ClassRoutineId,
}

impl DeleteClassRoutineCommand {
    /// Returns the school id (taken from the typed id).
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.class_routine_id.school_id()
    }

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicClassRoutineDelete]
    }
}

// =============================================================================
// Homework commands (Wave 52: full impl)
// =============================================================================

/// Command: create a [`Homework`](crate::aggregate::Homework)
/// assignment.
///
/// Per `docs/specs/academic/aggregates.md` § Homework:
/// - **I-1**: created by a teacher (`tenant.user_type` must
///   be `UserType::Teacher`, enforced by `create_homework`).
/// - **I-2**: `submission_date` must be strictly after
///   `homework_date` (enforced by
///   [`crate::aggregate::Homework::fresh`]).
/// - **I-4**: `attachment_id` is `Option<FileId>` (`None`
///   means no attachment).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateHomeworkCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The homework's typed id.
    pub homework_id: HomeworkId,
    /// The class-section the homework is assigned to.
    pub class_section_id: ClassSectionId,
    /// The subject of the assignment.
    pub subject_id: SubjectId,
    /// The teacher creating the homework (per I-1).
    pub teacher_id: UserId,
    /// The date the homework is assigned.
    pub homework_date: NaiveDate,
    /// The submission deadline (per I-2, strictly after
    /// `homework_date`).
    pub submission_date: NaiveDate,
    /// The homework description (1..=2000 chars).
    pub description: String,
    /// The optional file attachment (per I-4). `None`
    /// means "no attachment".
    pub attachment_id: Option<FileId>,
}

impl CreateHomeworkCommand {
    /// Returns the school id (taken from the typed id).
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.homework_id.school_id()
    }

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicHomeworkCreate]
    }
}

/// Command: update a [`Homework`](crate::aggregate::Homework)'s
/// mutable fields (description, submission_date,
/// attachment_id).
///
/// Per `docs/specs/academic/aggregates.md` § Homework § I-5,
/// this command is rejected once marks have been recorded
/// (status == `Evaluated`). Per I-2, the new
/// `submission_date` (when supplied) must be strictly after
/// `homework_date`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateHomeworkCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The homework's typed id.
    pub homework_id: HomeworkId,
    /// Optional new submission deadline. `None` means "do
    /// not change".
    pub submission_date: Option<NaiveDate>,
    /// Optional new description. `None` means "do not
    /// change".
    pub description: Option<String>,
    /// Optional new attachment. Outer `None` means "do not
    /// change"; outer `Some(None)` means "clear the
    /// attachment"; outer `Some(Some(fid))` means "set the
    /// attachment".
    pub attachment_id: Option<Option<FileId>>,
}

impl UpdateHomeworkCommand {
    /// Returns the school id (taken from the typed id).
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.homework_id.school_id()
    }

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicHomeworkUpdate]
    }
}

/// Command: cancel (soft-delete) a
/// [`Homework`](crate::aggregate::Homework).
///
/// Per `docs/specs/academic/aggregates.md` § Homework, the
/// cancel flow is unconditional for any non-cancelled
/// homework: the service retires the aggregate regardless of
/// marks or submission state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancelHomeworkCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The homework's typed id.
    pub homework_id: HomeworkId,
    /// The reason for cancellation (1..=500 chars).
    pub reason: String,
}

impl CancelHomeworkCommand {
    /// Returns the school id (taken from the typed id).
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.homework_id.school_id()
    }

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicHomeworkCancel]
    }
}
academic_command_stub! {
    /// Command stub: create a lesson plan. See
    /// `docs/specs/academic/aggregates.md` § LessonPlan.
    pub struct CreateLessonPlanCommand { id: LessonPlanId }
}
academic_command_stub! {
    /// Command stub: create a lesson. See
    /// `docs/specs/academic/aggregates.md` § Lesson.
    pub struct CreateLessonCommand { id: LessonId }
}
academic_command_stub! {
    /// Command stub: create a lesson topic. See
    /// `docs/specs/academic/aggregates.md` § LessonTopic.
    pub struct CreateLessonTopicCommand { id: LessonTopicId }
}
academic_command_stub! {
    /// Command stub: record a student promotion. See
    /// `docs/specs/academic/aggregates.md` § StudentPromotion.
    pub struct RecordStudentPromotionCommand { id: StudentPromotionId }
}
academic_command_stub! {
    /// Command stub: create a student category. See
    /// `docs/specs/academic/aggregates.md` § StudentCategory.
    pub struct CreateStudentCategoryCommand { id: StudentCategoryId }
}
academic_command_stub! {
    /// Command stub: create a student group. See
    /// `docs/specs/academic/aggregates.md` § StudentGroup.
    pub struct CreateStudentGroupCommand { id: StudentGroupId }
}
academic_command_stub! {
    /// Command stub: create a registration custom field. See
    /// `docs/specs/academic/aggregates.md` § RegistrationField.
    pub struct CreateRegistrationFieldCommand { id: RegistrationFieldId }
}
academic_command_stub! {
    /// Command stub: create a certificate template. See
    /// `docs/specs/academic/aggregates.md` § Certificate.
    pub struct CreateCertificateCommand { id: CertificateId }
}
academic_command_stub! {
    /// Command stub: create an ID card template. See
    /// `docs/specs/academic/aggregates.md` § IdCard.
    pub struct CreateIdCardCommand { id: IdCardId }
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

// =============================================================================
// Cluster D final 20%: spec commands previously missing
// (`AssignStudentToSection`, `ChangeStudentCategory`,
// `AssignOptionalSubject`, `UploadStudentDocument`,
// `AssignClassTeacher`, `AssignSubjectTeacher`, `AssignClassRoom`,
// `AssignSubjectToClass`, `SubmitHomework`, `EvaluateHomework`,
// `AddStudentToGroup`, `RegisterAdmissionQuery`).
//
// Each stub carries the minimal `tenant` + aggregate ids
// required to type-check. The full payload (reasons,
// effective dates, file references, marks, etc.) lands in a
// follow-up batch — the lint only enforces struct
// existence.
// =============================================================================

/// Command: assign a student to a class section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssignStudentToSectionCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The student being assigned.
    pub student_id: StudentId,
    /// The target section.
    pub section_id: SectionId,
}


impl AssignStudentToSectionCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicStudentUpdate]
    }
}
/// Command: change a student's category.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeStudentCategoryCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The student whose category is changing.
    pub student_id: StudentId,
    /// The new category.
    pub category_id: StudentCategoryId,
    /// The effective date of the change.
    pub effective_from: NaiveDate,
}


impl ChangeStudentCategoryCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicStudentUpdate]
    }
}
/// Command: assign an optional subject to a student.
///
/// Per Student I-4, a student may have at most one optional
/// subject per academic year. The service enforces the
/// uniqueness via `UniquenessChecker::optional_subject_assigned_exists`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssignOptionalSubjectCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The new assignment's typed id.
    pub assignment_id: OptionalSubjectAssignmentId,
    /// The student receiving the optional subject.
    pub student_id: StudentId,
    /// The optional subject being assigned.
    pub subject_id: SubjectId,
    /// The academic year of the assignment.
    pub academic_year_id: AcademicYearId,
}


impl AssignOptionalSubjectCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicStudentUpdate]
    }
}
/// Command: upload a student document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UploadStudentDocumentCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The student the document belongs to.
    pub student_id: StudentId,
}


impl UploadStudentDocumentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicStudentDocumentUpload]
    }
}
/// Command: assign a class teacher to a class section.
///
/// Per `docs/specs/academic/aggregates.md` § ClassSection §
/// I-2 (permissive), a class-section may have multiple class
/// teachers; this command appends a teacher reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssignClassTeacherCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The target class section.
    pub class_section_id: ClassSectionId,
    /// The teacher being assigned (typed user id).
    pub teacher_id: UserId,
}


impl AssignClassTeacherCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicClassSubject]
    }
}
/// Command: assign a subject teacher to a class section.
///
/// Per `docs/specs/academic/aggregates.md` § ClassSection §
/// I-2 (permissive), a class-section may have multiple
/// subject teachers; this command appends a teacher
/// reference for the given subject.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssignSubjectTeacherCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The target class section.
    pub class_section_id: ClassSectionId,
    /// The subject the teacher is assigned to.
    pub subject_id: SubjectId,
    /// The teacher being assigned (typed user id).
    pub teacher_id: UserId,
}


impl AssignSubjectTeacherCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicClassSubject]
    }
}
/// Command: assign a classroom to a class section.
///
/// Per `docs/specs/academic/aggregates.md` § ClassSection §
/// I-3 (one or more class rooms), this command appends an
/// additional `ClassRoomId` to the section's `class_rooms`
/// list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssignClassRoomCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The target class section.
    pub class_section_id: ClassSectionId,
    /// The class room being assigned.
    pub class_room_id: ClassRoomId,
}


impl AssignClassRoomCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicClassSubject]
    }
}
/// Command: delete (soft-retire) a [`ClassSection`](crate::aggregate::ClassSection).
///
/// Per `docs/specs/academic/aggregates.md` § ClassSection §
/// I-4, deletion is rejected while any `StudentRecord`
/// references the class-section. The service checks this
/// via `UniquenessChecker::class_section_has_student_records`
/// and refuses to retire the aggregate when true.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteClassSectionCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The class section being deleted.
    pub class_section_id: ClassSectionId,
}


impl DeleteClassSectionCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicClassDelete]
    }
}
/// Command: assign a subject to a class (or class-section),
/// with a teacher.
///
/// Per `docs/specs/academic/aggregates.md` § ClassSubject:
/// - **I-1**: `scope == ClassOnly` requires
///   `class_section_id == None`; `scope == ClassSection`
///   requires `class_section_id == Some(_)`. Enforced by
///   [`ClassSubject::fresh`](crate::aggregate::ClassSubject::fresh).
/// - **I-3**: `pass_mark`, when `Some`, must be in
///   `0.0..=100.0`. Enforced by
///   [`ClassSubject::fresh`](crate::aggregate::ClassSubject::fresh)
///   via [`PassMark::new`](crate::value_objects::PassMark::new).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssignSubjectToClassCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The new class-subject's typed id.
    pub class_subject_id: ClassSubjectId,
    /// The class this subject is assigned to.
    pub class_id: ClassId,
    /// The class-section this subject is assigned to.
    /// `Some` when `scope == ClassSection`, `None` when
    /// `scope == ClassOnly`.
    pub class_section_id: Option<ClassSectionId>,
    /// The subject being assigned.
    pub subject_id: SubjectId,
    /// The teacher being assigned (typed user id).
    pub teacher_id: UserId,
    /// The scope of the assignment.
    pub scope: crate::value_objects::ClassSubjectScope,
    /// Optional per-class-subject `PassMark` override.
    pub pass_mark: Option<crate::value_objects::PassMark>,
}

impl AssignSubjectToClassCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicClassSubject]
    }
}

/// Command: reassign a different teacher to an existing
/// active [`ClassSubject`](crate::aggregate::ClassSubject).
///
/// Per `docs/specs/academic/aggregates.md` § ClassSubject,
/// the reassignment is unconditional (I-2 is permissive:
/// a teacher may be assigned to multiple class-subjects).
/// The service enforces that the target aggregate is
/// `Active` before applying.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReassignTeacherCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The target class-subject.
    pub class_subject_id: ClassSubjectId,
    /// The new teacher id.
    pub new_teacher_id: UserId,
}

impl ReassignTeacherCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicClassSubject]
    }
}

/// Command: unassign (soft-retire) a
/// [`ClassSubject`](crate::aggregate::ClassSubject).
///
/// Per `docs/specs/academic/aggregates.md` § ClassSubject,
/// unassignment is unconditional: any active class-subject
/// may be retired by the service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnassignSubjectCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The class-subject being unassigned.
    pub class_subject_id: ClassSubjectId,
}

impl UnassignSubjectCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicClassSubject]
    }
}

/// Backward-compatible alias for [`AssignSubjectToClassCommand`].
///
/// The earlier (placeholder-era) name `CreateClassSubjectCommand`
/// is retained so older call sites compile unchanged. New
/// callers should prefer [`AssignSubjectToClassCommand`].
#[deprecated(
    since = "0.1.0",
    note = "renamed to AssignSubjectToClassCommand per spec; alias retained for backward compat"
)]
pub type CreateClassSubjectCommand = AssignSubjectToClassCommand;
/// Command: submit homework for a student.
///
/// Per `docs/specs/academic/commands.md` § SubmitHomework,
/// the student (or a parent on their behalf) submits the
/// work; the service records the submission and emits the
/// `HomeworkSubmitted` event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmitHomeworkCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The homework being submitted.
    pub homework_id: HomeworkId,
    /// The student submitting the homework.
    pub student_id: StudentId,
    /// Optional student-supplied description / notes.
    pub description: Option<String>,
    /// Optional submitted file (the homework's worked
    /// solution upload).
    pub file: Option<FileId>,
}


impl SubmitHomeworkCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicStudentHomeworkSubmit]
    }
}
/// Command: evaluate a student's homework submission.
///
/// Per `docs/specs/academic/commands.md` § EvaluateHomework,
/// the teacher records the marks for the student. Per
/// `docs/specs/academic/aggregates.md` § Homework § I-5, the
/// marks are immutable per student once recorded. Per I-3,
/// the `evaluation_date` (the mint-time of this command) must
/// be `>= submission_date`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvaluateHomeworkCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The homework being evaluated.
    pub homework_id: HomeworkId,
    /// The student whose submission is being evaluated.
    pub student_id: StudentId,
    /// The awarded marks (0..=100).
    pub marks: u8,
    /// Optional teacher comments.
    pub teacher_comments: Option<String>,
}


impl EvaluateHomeworkCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicStudentHomeworkEvaluate]
    }
}
/// Command: add a student to a student group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddStudentToGroupCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The target student group.
    pub group_id: StudentGroupId,
    /// The student being added.
    pub student_id: StudentId,
}


impl AddStudentToGroupCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicStudentUpdate]
    }
}
/// Command: register a new admission query (inquiry).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterAdmissionQueryCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The class the inquiry concerns.
    pub class_id: ClassId,
    /// The date the inquiry was registered.
    pub date: NaiveDate,
}


impl RegisterAdmissionQueryCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::AcademicStudentUpdate]
    }
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
