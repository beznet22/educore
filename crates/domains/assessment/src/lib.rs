//! # educore-assessment
//!
//!  Exams, marks registers, results, report cards, online exams, seat plans, admit cards.
//!
//! This crate is a member of the Educore workspace. See
//! `docs/architecture.md` and the domain spec in
//! `docs/specs/` for behavioral details.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// Package name constant. Re-exported so consumers can assert they
/// are using the right crate version at compile time.
pub const PACKAGE_NAME: &str = "educore-assessment";

/// Package version at compile time.
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

// ---- Module tree (9-file layout per AGENTS.md) ---------------------------

/// The 5 assessment aggregate roots shipped in Phase 4 Workstream A
/// + the typed events / commands / services / repositories that
/// bind to them.
pub mod commands;
/// The error helper. Re-exports the engine's `DomainError` as
/// `AssessmentError` for symmetry with the academic crate.
pub mod errors;
/// The 2 typed `DomainEvent` implementations shipped in Phase 4
/// Workstream A.
pub mod events;
/// The assessment domain's services: pure factory functions
/// that return the mutated aggregate + the typed event.
pub mod services;
/// The assessment domain's value objects: typed ids (per
/// aggregate), validated string wrappers, numeric newtypes, and
/// closed status enums.
pub mod value_objects;

// Crate-private modules (re-exported selectively below).
mod aggregate;
mod entities;
mod query;
mod repository;

// ---- Re-exports of the public surface ------------------------------------
/// Typed ids and value objects the assessment crate re-exports
/// for downstream consumers.
pub use crate::value_objects::{
    AcademicYearId, AcademicYearRange, AdmitCardId, ClassId, ClassRoomId, DateOfBirth, ExamCode,
    ExamId, ExamMark, ExamName, ExamScheduleId, ExamScheduleSubjectId, ExamTerm, ExamTypeId,
    FullMark, PassMark, ResultStatus, SeatPlanChildId, SeatPlanId, SectionId, StaffId, StudentId,
    StudentRecordId, SubjectId,
};

/// Convenience re-exports of the engine types the assessment
/// crate most commonly uses. Consumers should
/// `use educore_assessment::prelude::*;` at the top of a file.
pub mod prelude {
    pub use educore_core::clock::{Clock, IdGenerator};
    pub use educore_core::error::{DomainError, Result};
    pub use educore_core::ids::{CorrelationId, EventId, Identifier, SchoolId, UserId};
    pub use educore_core::tenant::{TenantContext, UserType};
    pub use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};
    pub use educore_events::domain_event::DomainEvent;
    pub use educore_events::envelope::EventEnvelope;
    pub use educore_rbac::value_objects::Capability;

    pub use crate::aggregate::{AdmitCard, Exam, ExamSchedule, SeatPlan};
    pub use crate::commands::{
        validate_exam_code, validate_exam_mark, validate_exam_name, validate_pass_mark,
        AssessmentUniquenessChecker, CancelAdmitCardCommand, CancelExamScheduleCommand,
        CancelSeatPlanCommand, CreateExamCommand, DeleteExamCommand, GenerateAdmitCardCommand,
        GenerateSeatPlanCommand, RegenerateAdmitCardCommand, ScheduleExamCommand,
        UniquenessChecker, UpdateExamCommand, UpdateExamScheduleCommand, UpdateSeatPlanCommand,
        ASSESSMENT_ADMIT_CARD_CANCEL_COMMAND_TYPE, ASSESSMENT_ADMIT_CARD_GENERATE_COMMAND_TYPE,
        ASSESSMENT_ADMIT_CARD_REGENERATE_COMMAND_TYPE, ASSESSMENT_EXAM_CREATE_COMMAND_TYPE,
        ASSESSMENT_EXAM_DELETE_COMMAND_TYPE, ASSESSMENT_EXAM_SCHEDULE_CANCEL_COMMAND_TYPE,
        ASSESSMENT_EXAM_SCHEDULE_CREATE_COMMAND_TYPE, ASSESSMENT_EXAM_SCHEDULE_UPDATE_COMMAND_TYPE,
        ASSESSMENT_EXAM_UPDATE_COMMAND_TYPE, ASSESSMENT_SEAT_PLAN_CANCEL_COMMAND_TYPE,
        ASSESSMENT_SEAT_PLAN_GENERATE_COMMAND_TYPE, ASSESSMENT_SEAT_PLAN_UPDATE_COMMAND_TYPE,
    };
    pub use crate::entities::{ExamScheduleSubject, SeatPlanChild};
    pub use crate::errors::AssessmentError;
    pub use crate::events::{
        AdmitCardCancelled, AdmitCardGenerated, AdmitCardRegenerated, ExamCreated, ExamDeleted,
        ExamScheduleCancelled, ExamScheduleUpdated, ExamScheduled, SeatPlanCancelled,
        SeatPlanGenerated, SeatPlanUpdated, ExamUpdated,
    };
    pub use crate::repository::{
        AdmitCardRepository, ExamRepository, ExamScheduleRepository, SeatPlanRepository,
    };
    pub use crate::services::{
        cancel_admit_card, cancel_exam_schedule, cancel_seat_plan, create_exam, delete_exam,
        generate_admit_card, generate_seat_plan, regenerate_admit_card, schedule_exam,
        school_matches, update_exam, update_exam_schedule, update_seat_plan,
    };
    pub use crate::value_objects::{
        AcademicYearId, AcademicYearRange, AdmitCardId, ClassId, ClassRoomId, DateOfBirth,
        ExamCode, ExamId, ExamMark, ExamName, ExamScheduleId, ExamScheduleSubjectId, ExamTerm,
        ExamTypeId, FullMark, PassMark, ResultStatus, SeatPlanChildId, SeatPlanId, SectionId,
        StaffId, StudentId, StudentRecordId, SubjectId,
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
    use super::*;
    use educore_core::clock::SystemIdGen;

    #[test]
    fn package_metadata_is_set() {
        assert_eq!(PACKAGE_NAME, "educore-assessment");
        assert!(!PACKAGE_VERSION.is_empty());
    }

    #[test]
    fn prelude_wires_expected_types() {
        use crate::prelude::*;
        let _: Capability = Capability::AssessmentExamCreate;
        let _: ExamTerm = ExamTerm::UnitTest;
        let _: ResultStatus = ResultStatus::Pass;
        let g = SystemIdGen;
        let school = g.next_school_id();
        let _: ExamId = ExamId::new(school, g.next_uuid());
        let _: ExamTypeId = ExamTypeId::new(school, g.next_uuid());
        let _: ExamMark = ExamMark::new(100.0).expect("mark");
        let _: ExamName = ExamName::new("Mid-Term Mathematics").expect("name");
        let _: ExamCode = ExamCode::new("MTH-MT-2024").expect("code");
    }
}
