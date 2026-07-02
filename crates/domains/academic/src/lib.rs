//! # educore-academic
//!
//! Student lifecycle, classes, sections, subjects, and
//! academic year. The first domain crate Phase 3 ships.
//!
//! Phase 3 delivers the **prompt-named subset only**:
//! [`Student`], [`Class`], [`Section`], [`Subject`], and
//! [`AcademicYear`]. The remaining 27 academic aggregates
//! in `docs/specs/academic/aggregates.md` (Guardian,
//! ClassSection, ClassSubject, ClassRoutine, Homework,
//! Lesson, LessonTopic, LessonPlan, StudentRecord,
//! StudentPromotion, StudentCategory, StudentGroup,
//! RegistrationField, Certificate, IdCard, AdmissionQuery,
//! etc.) land in later phases.
//!
//! The crate follows the engine's 9-file module layout
//! (per `AGENTS.md` § "Module Layout (per domain)"):
//!
//! - [`lib.rs`](self) — re-exports + prelude
//! - [`aggregate`] — the 5 aggregate roots
//! - [`entities`] — child entities (placeholder for the
//!   full spec; Phase 3 ships the type shell only)
//! - [`value_objects`] — typed ids + value objects
//! - [`commands`] — the 23 typed command shapes
//! - [`events`] — the 19 typed events implementing
//!   `DomainEvent`
//! - [`services`] — the 19 pure factory functions
//! - [`repository`] — the 5 repository port traits
//! - [`query`] — the 5 typed query stubs
//! - [`errors`] — `AcademicError = DomainError` alias
//!
//! The crate depends on `educore-core`, `educore-platform`,
//! `educore-rbac`, `educore-events`, and `educore-settings`
//! (per the workspace `Cargo.toml`). It has no dependency
//! on the `adapters` or `tools` tiers. The
//! `StudentRepository` / `ClassRepository` / etc. traits
//! are ports the storage adapter crates implement.
//!
//! See `docs/specs/academic/`, `docs/ports/event-bus.md`,
//! `docs/ports/storage.md`, and `docs/handoff/PHASE-3-HANDOFF.md`
//! for the design contract and the Phase 3 hand-off.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod aggregate;
pub mod commands;
pub mod entities;
pub mod errors;
pub mod events;
mod query;
mod repository;
pub mod services;
pub mod value_objects;

/// Package name constant. Re-exported so consumers can
/// assert they are using the right crate version at
/// compile time.
pub const PACKAGE_NAME: &str = "educore-academic";

/// Package version at compile time.
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

// ---- Aggregate roots --------------------------------------------------------

/// The 5 prompt-named aggregate roots.
pub use crate::aggregate::{AcademicYear, Class, Section, Student, Subject};

/// The 14 placeholder aggregate stubs (Guardian, ClassSection,
/// ClassSubject, etc.). Exposed at the crate root so integration
/// tests can reach them.
pub use crate::aggregate::{
    Certificate, ClassRoutine, ClassSection, ClassSubject, Guardian, Homework, IdCard, Lesson,
    LessonPlan, OptionalSubjectAssignment, RegistrationField, StudentCategory, StudentGroup,
    StudentGuardianLink, StudentPromotion,
};

// ---- Typed events -----------------------------------------------------------

/// The 19 typed events emitted by the academic commands.
pub use crate::events::{
    AcademicYearClosed, AcademicYearCopied, AcademicYearCreated, AcademicYearDatesUpdated,
    CertificateCreated, ClassCreated, ClassDeleted, ClassRoomAssigned, ClassRoutineDeleted,
    ClassRoutinePeriodUpdated, ClassRoutinePeriodsSwapped, ClassRoutineScheduled,
    ClassSectionCreated, ClassSectionDeleted, ClassTeacherAssigned, ClassUpdated,
    CurrentAcademicYearSet, GuardianContactUpdated, GuardianLinkedToStudent, GuardianRegistered,
    GuardianRetired, GuardianUnlinkedFromStudent, HomeworkCreated, IdCardCreated, LessonCreated,
    LessonPlanCreated, LessonTopicCreated, OptionalSubjectAssignmentCreated,
    OptionalSubjectGpaThresholdSet, PrimaryGuardianMarked, RegistrationFieldCreated,
    SectionCreated, SectionDeleted, SectionUpdated, StudentAdmitted, StudentCategoryCreated,
    StudentGraduated, StudentGroupCreated, StudentProfileUpdated, StudentPromoted,
    StudentPromotionRecorded, StudentReinstated, StudentSuspended, StudentTransferred,
    StudentWithdrawn, SubjectAssignedToClass, SubjectCreated, SubjectDeleted, SubjectUnassigned,
    SubjectUpdated, SubjectTeacherAssigned, TeacherReassigned,
};

// ---- Pure factory functions -------------------------------------------------

/// The 19 pure factory functions that turn a command into
/// an aggregate plus a typed event.
pub use crate::services::{
    admit_student, assign_class_room, assign_class_teacher, assign_optional_subject,
    assign_subject_teacher, assign_subject_to_class, close_academic_year, copy_academic_year,
    create_academic_year, create_certificate, create_class, create_class_routine,
    create_class_section, create_class_subject, create_homework, create_id_card, create_lesson,
    create_lesson_plan, create_lesson_topic, create_registration_field, create_section,
    create_student_category, create_student_group, create_subject, delete_class,
    delete_class_routine, delete_class_section, delete_section, delete_subject, graduate_student,
    link_guardian_to_student, mark_primary_guardian, promote_student, reassign_teacher,
    record_student_promotion, register_guardian, reinstate_student, retire_guardian,
    school_matches, set_current_academic_year, set_optional_subject_gpa_threshold,
    suspend_student, swap_class_routine_periods, transfer_student, unassign_subject,
    unlink_guardian_from_student, update_academic_year_dates, update_class,
    update_class_routine_period, update_guardian_contact, update_section, update_student_profile,
    update_subject, withdraw_student,
};

// ---- Per-aggregate repository port traits -----------------------------------

/// The 5 repository port traits.
pub use crate::repository::{
    AcademicYearRepository, ClassRepository, SectionRepository, StudentRepository,
    SubjectRepository,
};

// ---- Typed query stubs ------------------------------------------------------

/// The 5 typed query stubs (the executor lands in Phase 4+).
pub use crate::query::{AcademicYearQuery, ClassQuery, SectionQuery, StudentQuery, SubjectQuery};

// ---- Errors ----------------------------------------------------------------

/// The academic error helper (alias for
/// [`educore_core::error::DomainError`]).
pub use crate::errors::AcademicError;

// ---- Uniqueness checker port ------------------------------------------------

/// The academic uniqueness-checker port.
pub use crate::commands::UniquenessChecker;

// ---- Value objects / typed ids ---------------------------------------------

/// Typed ids and value objects the academic crate re-exports
/// for downstream consumers.
pub use crate::value_objects::{
    AcademicYearId, AcademicYearRange, AcademicYearTitle, Address, AdmissionNumber, BloodGroup,
    CertificateId, ClassId, ClassName, ClassPeriod, ClassRoomId, ClassRoutineId, ClassSectionId,
    ClassSubjectId, ClassSubjectScope, ClassTimeId, DateOfBirth, DayOfWeek, EmailAddress,
    FileId, FullName, Gender, GuardianId, HomeworkId, HomeworkMark, HomeworkStatus, IdCardId,
    LessonId, LessonPlanId, LessonTopicId, OptionalSubjectAssignmentId, OptionalSubjectGpaThreshold,
    PassMark, PersonName, PhoneNumber, RegistrationFieldId, Relation, ResultStatus, RollNumber,
    SectionId, SectionName, StudentCategoryId, StudentGroupId, StudentGuardianLinkId, StudentId,
    StudentPromotionId, StudentRecordId, StudentStatus, SubjectCode, SubjectId, SubjectType,
    SuspensionReason, TransferReason, WithdrawalReason,
};

// ---- Re-exports of the engine types most commonly reached for ----------------

/// Convenience re-exports of the engine types the academic
/// crate most commonly uses. Consumers should
/// `use educore_academic::prelude::*;` at the top of a file.
pub mod prelude {
    pub use educore_core::clock::{Clock, IdGenerator};
    pub use educore_core::error::{DomainError, Result};
    pub use educore_core::ids::{CorrelationId, EventId, Identifier, SchoolId, UserId};
    pub use educore_core::tenant::{TenantContext, UserType};
    pub use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};
    pub use educore_events::domain_event::DomainEvent;
    pub use educore_events::envelope::EventEnvelope;
    pub use educore_rbac::value_objects::Capability;

    pub use crate::aggregate::{AcademicYear, Class, Section, Student, Subject};
    pub use crate::aggregate::{
        Certificate, ClassRoutine, ClassSection, ClassSubject, Guardian, Homework, IdCard, Lesson,
        LessonPlan, OptionalSubjectAssignment, RegistrationField, StudentCategory, StudentGroup,
        StudentGuardianLink, StudentPromotion,
    };
    pub use crate::commands::{
        AdmitStudentCommand, AssignClassRoomCommand, AssignClassTeacherCommand,
        AssignOptionalSubjectCommand, AssignSubjectTeacherCommand, AssignSubjectToClassCommand,
        CloseAcademicYearCommand, CreateAcademicYearCommand, CreateCertificateCommand,
        CreateClassCommand, CreateClassRoutineCommand, CreateClassSectionCommand,
        CreateHomeworkCommand, CreateIdCardCommand, CreateLessonCommand, CreateLessonPlanCommand,
        CreateLessonTopicCommand, CreateRegistrationFieldCommand, CreateSectionCommand,
        CreateStudentCategoryCommand, CreateStudentGroupCommand, CreateSubjectCommand,
        DeleteClassCommand, DeleteClassRoutineCommand, DeleteClassSectionCommand,
        DeleteSectionCommand, DeleteSubjectCommand, GraduateStudentCommand,
        LinkGuardianToStudentCommand, MarkPrimaryGuardianCommand, PromoteStudentCommand,
        ReassignTeacherCommand, RecordStudentPromotionCommand, RegisterGuardianCommand,
        ReinstateStudentCommand, RetireGuardianCommand, SetCurrentAcademicYearCommand,
        SetOptionalSubjectGpaThresholdCommand, SuspendStudentCommand, SwapClassRoutinePeriodsCommand,
        TransferStudentCommand, UnassignSubjectCommand, UniquenessChecker,
        UnlinkGuardianFromStudentCommand, UpdateAcademicYearDatesCommand, UpdateClassCommand,
        UpdateClassRoutinePeriodCommand, UpdateGuardianContactCommand, UpdateSectionCommand,
        UpdateStudentProfileCommand, UpdateSubjectCommand, WithdrawStudentCommand,
    };
    pub use crate::entities::{DocumentType, StudentDocument, StudentDocumentId};
    pub use crate::errors::AcademicError;
    pub use crate::events::{
        AcademicYearClosed, AcademicYearCopied, AcademicYearCreated, AcademicYearDatesUpdated,
        CertificateCreated, ClassCreated, ClassDeleted, ClassRoomAssigned, ClassRoutineDeleted,
        ClassRoutinePeriodUpdated, ClassRoutinePeriodsSwapped, ClassRoutineScheduled,
        ClassSectionCreated, ClassSectionDeleted, ClassTeacherAssigned, ClassUpdated,
        CurrentAcademicYearSet, GuardianContactUpdated, GuardianLinkedToStudent, GuardianRegistered,
        GuardianRetired, GuardianUnlinkedFromStudent, HomeworkCreated, IdCardCreated, LessonCreated,
        LessonPlanCreated, LessonTopicCreated, OptionalSubjectAssignmentCreated,
        OptionalSubjectGpaThresholdSet, PrimaryGuardianMarked, RegistrationFieldCreated,
        SectionCreated, SectionDeleted, SectionUpdated, StudentAdmitted, StudentCategoryCreated,
        StudentGraduated, StudentGroupCreated, StudentProfileUpdated, StudentPromoted,
        StudentPromotionRecorded, StudentReinstated, StudentSuspended, StudentTransferred,
        StudentWithdrawn, SubjectAssignedToClass, SubjectCreated, SubjectDeleted, SubjectUnassigned,
        SubjectUpdated, SubjectTeacherAssigned, TeacherReassigned,
    };
    pub use crate::query::{
        AcademicYearQuery, ClassQuery, SectionQuery, StudentQuery, SubjectQuery,
    };
    pub use crate::repository::{
        AcademicYearRepository, ClassRepository, SectionRepository, StudentRepository,
        SubjectRepository,
    };
    pub use crate::services::{
        admit_student, assign_class_room, assign_class_teacher, assign_optional_subject,
        assign_subject_teacher, assign_subject_to_class, close_academic_year, copy_academic_year,
        create_academic_year, create_certificate, create_class, create_class_routine,
        create_class_section, create_class_subject, create_homework, create_id_card, create_lesson,
        create_lesson_plan, create_lesson_topic, create_registration_field, create_section,
        create_student_category, create_student_group, create_subject, delete_class,
        delete_class_routine, delete_class_section, delete_section, delete_subject,
        graduate_student, link_guardian_to_student, mark_primary_guardian, promote_student,
        reassign_teacher, record_student_promotion, register_guardian, reinstate_student,
        retire_guardian, school_matches, set_current_academic_year,
        set_optional_subject_gpa_threshold, suspend_student, swap_class_routine_periods,
        transfer_student, unassign_subject, unlink_guardian_from_student,
        update_academic_year_dates, update_class, update_class_routine_period,
        update_guardian_contact, update_section, update_student_profile, update_subject,
        withdraw_student,
    };
    pub use crate::value_objects::{
        AcademicYearId, AcademicYearRange, AcademicYearTitle, Address, AdmissionNumber, BloodGroup,
        CertificateId, ClassId, ClassName, ClassPeriod, ClassRoomId, ClassRoutineId,
        ClassSectionId, ClassSubjectId, ClassSubjectScope, ClassTimeId, DateOfBirth, DayOfWeek,
        EmailAddress, FullName, Gender, GuardianId, HomeworkId, IdCardId, LessonId, LessonPlanId,
        LessonTopicId, OptionalSubjectAssignmentId, OptionalSubjectGpaThreshold, PassMark,
        PersonName, PhoneNumber, RegistrationFieldId, Relation, ResultStatus, RollNumber,
        SectionId, SectionName, StudentCategoryId, StudentGroupId, StudentGuardianLinkId,
        StudentId, StudentPromotionId, StudentRecordId, StudentStatus, SubjectCode, SubjectId,
        SubjectType, SuspensionReason, TransferReason, WithdrawalReason,
    };
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::prelude::*;
    use super::{PACKAGE_NAME, PACKAGE_VERSION};

    #[test]
    fn package_metadata_is_set() {
        assert_eq!(PACKAGE_NAME, "educore-academic");
        assert!(!PACKAGE_VERSION.is_empty());
    }

    #[test]
    fn prelude_re_exports_aggregate_types() {
        // The academic crate's prelude must surface all 5
        // aggregate roots and the engine's cross-cutting
        // types so that consumers can
        // `use educore_academic::prelude::*;` in a single
        // statement.
        let _: Option<fn() -> Student> = None;
        let _: Option<fn() -> Class> = None;
        let _: Option<fn() -> Section> = None;
        let _: Option<fn() -> Subject> = None;
        let _: Option<fn() -> AcademicYear> = None;
    }

    #[test]
    fn prelude_re_exports_typed_ids() {
        let g = educore_core::clock::SystemIdGen;
        let school = g.next_school_id();
        let _: StudentId = StudentId::new(school, g.next_uuid());
        let _: ClassId = ClassId::new(school, g.next_uuid());
        let _: SectionId = SectionId::new(school, g.next_uuid());
        let _: SubjectId = SubjectId::new(school, g.next_uuid());
        let _: AcademicYearId = AcademicYearId::new(school, g.next_uuid());
    }

    #[test]
    fn student_record_id_round_trips() {
        let school = educore_core::ids::SchoolId(uuid::Uuid::now_v7());
        let value = uuid::Uuid::now_v7();
        let id = StudentRecordId::new(school, value);
        assert_eq!(id.school_id(), school);
        assert_eq!(id.as_uuid(), value);
        assert_eq!(id.to_string(), format!("{school}/{value}"));
    }
}
