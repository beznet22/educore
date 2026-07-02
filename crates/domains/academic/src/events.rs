//! # Academic domain events
//!
//! Every typed event in the academic domain implements
//! [`educore_events::domain_event::DomainEvent`]. The
//! `event_type` is namespaced as `"academic.<aggregate>.<verb>"`
//! per the bus-port contract (e.g. `"academic.student.admitted"`).
//!
//! Phase 3 ships the prompt-named subset:
//!
//! - **Student lifecycle (8 events)**: [`StudentAdmitted`],
//!   [`StudentProfileUpdated`], [`StudentSuspended`],
//!   [`StudentReinstated`], [`StudentWithdrawn`],
//!   [`StudentTransferred`], [`StudentPromoted`],
//!   [`StudentGraduated`].
//! - **Class & Section (7 events)**: [`ClassCreated`],
//!   [`ClassUpdated`], [`OptionalSubjectGpaThresholdSet`],
//!   [`ClassDeleted`], [`SectionCreated`], [`SectionUpdated`],
//!   [`SectionDeleted`].
//! - **Subject (3 events)**: [`SubjectCreated`],
//!   [`SubjectUpdated`], [`SubjectDeleted`].
//! - **AcademicYear (5 events)**: [`AcademicYearCreated`],
//!   [`AcademicYearDatesUpdated`], [`CurrentAcademicYearSet`],
//!   [`AcademicYearClosed`], [`AcademicYearCopied`].
//!
//! The remaining events in
//! `docs/specs/academic/events.md` (Guardian lifecycle,
//! ClassSection, ClassSubject, ClassRoutine, Homework,
//! Lesson, LessonTopic, LessonPlan, StudentRecord,
//! StudentCategory, StudentGroup, RegistrationField,
//! Certificate, IdCard, AdmissionQuery) land alongside
//! their owning aggregates in later phases.
#![allow(clippy::too_many_arguments)]

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;

use crate::entities::StudentDocumentId;
use crate::value_objects::{
    AcademicYearId, CertificateId, ClassId, ClassRoomId, ClassRoutineId, ClassSectionId,
    ClassSubjectId, GuardianId, HomeworkId, IdCardId, LessonId, LessonPlanId, LessonTopicId,
    OptionalSubjectAssignmentId, RegistrationFieldId, Relation, ResultStatus, SectionId,
    StudentCategoryId, StudentGroupId, StudentGuardianLinkId, StudentId, StudentPromotionId,
    StudentStatus, SubjectId, SubjectType,
};

// =============================================================================
// Student lifecycle (8 events)
// =============================================================================

/// A student was admitted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StudentAdmitted {
    /// The student's typed id.
    pub student_id: StudentId,
    /// The student's admission number.
    pub admission_no: String,
    /// The student's first name.
    pub first_name: String,
    /// The student's last name.
    pub last_name: String,
    /// The class the student was admitted into.
    pub class_id: ClassId,
    /// The section the student was admitted into.
    pub section_id: SectionId,
    /// The academic year the admission applies to.
    pub academic_year_id: AcademicYearId,
    /// The date of admission.
    pub admission_date: NaiveDate,
    /// The initial roll number (optional).
    pub roll_no: Option<String>,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl StudentAdmitted {
    /// Mints a fresh `StudentAdmitted`.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        student_id: StudentId,
        admission_no: String,
        first_name: String,
        last_name: String,
        class_id: ClassId,
        section_id: SectionId,
        academic_year_id: AcademicYearId,
        admission_date: NaiveDate,
        roll_no: Option<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            student_id,
            admission_no,
            first_name,
            last_name,
            class_id,
            section_id,
            academic_year_id,
            admission_date,
            roll_no,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StudentAdmitted {
    const EVENT_TYPE: &'static str = "academic.student.admitted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "student";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.student_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.student_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A student's mutable profile fields were updated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StudentProfileUpdated {
    /// The student's typed id.
    pub student_id: StudentId,
    /// The names of the fields that actually changed.
    pub changed_fields: Vec<String>,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl StudentProfileUpdated {
    /// Mints a fresh `StudentProfileUpdated`.
    #[must_use]
    pub const fn new(
        student_id: StudentId,
        changed_fields: Vec<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            student_id,
            changed_fields,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StudentProfileUpdated {
    const EVENT_TYPE: &'static str = "academic.student.profile_updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "student";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.student_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.student_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A student was suspended.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StudentSuspended {
    /// The student's typed id.
    pub student_id: StudentId,
    /// The reason for the suspension.
    pub reason: String,
    /// The first day the suspension is effective.
    pub effective_from: NaiveDate,
    /// The expected return date (optional).
    pub expected_return: Option<NaiveDate>,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl StudentSuspended {
    /// Mints a fresh `StudentSuspended`.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        student_id: StudentId,
        reason: String,
        effective_from: NaiveDate,
        expected_return: Option<NaiveDate>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            student_id,
            reason,
            effective_from,
            expected_return,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StudentSuspended {
    const EVENT_TYPE: &'static str = "academic.student.suspended";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "student";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.student_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.student_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A suspended student was reinstated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StudentReinstated {
    /// The student's typed id.
    pub student_id: StudentId,
    /// The first day the reinstatement is effective.
    pub effective_from: NaiveDate,
    /// Optional note.
    pub note: Option<String>,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl StudentReinstated {
    /// Mints a fresh `StudentReinstated`.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        student_id: StudentId,
        effective_from: NaiveDate,
        note: Option<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            student_id,
            effective_from,
            note,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StudentReinstated {
    const EVENT_TYPE: &'static str = "academic.student.reinstated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "student";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.student_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.student_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A student was withdrawn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StudentWithdrawn {
    /// The student's typed id.
    pub student_id: StudentId,
    /// The reason for the withdrawal.
    pub reason: String,
    /// The first day the withdrawal is effective.
    pub effective_from: NaiveDate,
    /// Optional note.
    pub note: Option<String>,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl StudentWithdrawn {
    /// Mints a fresh `StudentWithdrawn`.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        student_id: StudentId,
        reason: String,
        effective_from: NaiveDate,
        note: Option<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            student_id,
            reason,
            effective_from,
            note,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StudentWithdrawn {
    const EVENT_TYPE: &'static str = "academic.student.withdrawn";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "student";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.student_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.student_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A student was transferred to another school.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StudentTransferred {
    /// The student's typed id.
    pub student_id: StudentId,
    /// The destination school (must be a sibling school in
    /// the same SaaS tenant).
    pub destination_school_id: SchoolId,
    /// The reason for the transfer.
    pub reason: String,
    /// The first day the transfer is effective.
    pub effective_from: NaiveDate,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl StudentTransferred {
    /// Mints a fresh `StudentTransferred`.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        student_id: StudentId,
        destination_school_id: SchoolId,
        reason: String,
        effective_from: NaiveDate,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            student_id,
            destination_school_id,
            reason,
            effective_from,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StudentTransferred {
    const EVENT_TYPE: &'static str = "academic.student.transferred";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "student";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.student_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.student_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A student was promoted to the next academic year.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StudentPromoted {
    /// The student's typed id.
    pub student_id: StudentId,
    /// The class the student is moving from.
    pub from_class_id: ClassId,
    /// The section the student is moving from.
    pub from_section_id: SectionId,
    /// The class the student is moving to.
    pub to_class_id: ClassId,
    /// The section the student is moving to.
    pub to_section_id: SectionId,
    /// The source academic year.
    pub from_academic_year_id: AcademicYearId,
    /// The target academic year.
    pub to_academic_year_id: AcademicYearId,
    /// The new roll number.
    pub to_roll_no: String,
    /// The promotion result.
    pub result_status: ResultStatus,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl StudentPromoted {
    /// Mints a fresh `StudentPromoted`.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        student_id: StudentId,
        from_class_id: ClassId,
        from_section_id: SectionId,
        to_class_id: ClassId,
        to_section_id: SectionId,
        from_academic_year_id: AcademicYearId,
        to_academic_year_id: AcademicYearId,
        to_roll_no: String,
        result_status: ResultStatus,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            student_id,
            from_class_id,
            from_section_id,
            to_class_id,
            to_section_id,
            from_academic_year_id,
            to_academic_year_id,
            to_roll_no,
            result_status,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StudentPromoted {
    const EVENT_TYPE: &'static str = "academic.student.promoted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "student";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.student_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.student_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A student graduated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StudentGraduated {
    /// The student's typed id.
    pub student_id: StudentId,
    /// The academic year the student graduated in.
    pub academic_year_id: AcademicYearId,
    /// The graduation date.
    pub graduation_date: NaiveDate,
    /// The new status (always `Graduated`).
    pub status: StudentStatus,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl StudentGraduated {
    /// Mints a fresh `StudentGraduated`.
    #[must_use]
    pub const fn new(
        student_id: StudentId,
        academic_year_id: AcademicYearId,
        graduation_date: NaiveDate,
        status: StudentStatus,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            student_id,
            academic_year_id,
            graduation_date,
            status,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StudentGraduated {
    const EVENT_TYPE: &'static str = "academic.student.graduated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "student";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.student_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.student_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// Class events (4)
// =============================================================================

/// A class was created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassCreated {
    /// The class's typed id.
    pub class_id: ClassId,
    /// The class's display name.
    pub class_name: String,
    /// The class's pass mark.
    pub pass_mark: f32,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl ClassCreated {
    /// Mints a fresh `ClassCreated`.
    #[must_use]
    pub const fn new(
        class_id: ClassId,
        class_name: String,
        pass_mark: f32,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            class_id,
            class_name,
            pass_mark,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ClassCreated {
    const EVENT_TYPE: &'static str = "academic.class.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "class";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.class_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.class_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A class's mutable fields were updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassUpdated {
    /// The class's typed id.
    pub class_id: ClassId,
    /// The names of the fields that actually changed.
    pub changed_fields: Vec<String>,
    /// The new class name (if changed).
    pub class_name: Option<String>,
    /// The new pass mark (if changed).
    pub pass_mark: Option<f32>,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl ClassUpdated {
    /// Mints a fresh `ClassUpdated`.
    #[must_use]
    pub const fn new(
        class_id: ClassId,
        changed_fields: Vec<String>,
        class_name: Option<String>,
        pass_mark: Option<f32>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            class_id,
            changed_fields,
            class_name,
            pass_mark,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ClassUpdated {
    const EVENT_TYPE: &'static str = "academic.class.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "class";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.class_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.class_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A class's optional-subject GPA threshold was set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptionalSubjectGpaThresholdSet {
    /// The class's typed id.
    pub class_id: ClassId,
    /// The new threshold (0.0..=5.0).
    pub threshold: f32,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl OptionalSubjectGpaThresholdSet {
    /// Mints a fresh `OptionalSubjectGpaThresholdSet`.
    #[must_use]
    pub const fn new(
        class_id: ClassId,
        threshold: f32,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            class_id,
            threshold,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for OptionalSubjectGpaThresholdSet {
    const EVENT_TYPE: &'static str = "academic.class.optional_subject_gpa_threshold_set";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "class";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.class_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.class_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A class was deleted (soft-deleted; existing references remain).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassDeleted {
    /// The class's typed id.
    pub class_id: ClassId,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl ClassDeleted {
    /// Mints a fresh `ClassDeleted`.
    #[must_use]
    pub const fn new(
        class_id: ClassId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            class_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ClassDeleted {
    const EVENT_TYPE: &'static str = "academic.class.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "class";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.class_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.class_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// Section events (3)
// =============================================================================

/// A section was created.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionCreated {
    /// The section's typed id.
    pub section_id: SectionId,
    /// The section's display name.
    pub section_name: String,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl SectionCreated {
    /// Mints a fresh `SectionCreated`.
    #[must_use]
    pub const fn new(
        section_id: SectionId,
        section_name: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            section_id,
            section_name,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SectionCreated {
    const EVENT_TYPE: &'static str = "academic.section.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "section";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.section_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.section_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A section's mutable fields were updated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionUpdated {
    /// The section's typed id.
    pub section_id: SectionId,
    /// The names of the fields that actually changed.
    pub changed_fields: Vec<String>,
    /// The new section name (if changed).
    pub section_name: Option<String>,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl SectionUpdated {
    /// Mints a fresh `SectionUpdated`.
    #[must_use]
    pub const fn new(
        section_id: SectionId,
        changed_fields: Vec<String>,
        section_name: Option<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            section_id,
            changed_fields,
            section_name,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SectionUpdated {
    const EVENT_TYPE: &'static str = "academic.section.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "section";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.section_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.section_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A section was deleted (soft-deleted; existing references remain).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionDeleted {
    /// The section's typed id.
    pub section_id: SectionId,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl SectionDeleted {
    /// Mints a fresh `SectionDeleted`.
    #[must_use]
    pub const fn new(
        section_id: SectionId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            section_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SectionDeleted {
    const EVENT_TYPE: &'static str = "academic.section.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "section";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.section_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.section_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// Subject events (3)
// =============================================================================

/// A subject was created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubjectCreated {
    /// The subject's typed id.
    pub subject_id: SubjectId,
    /// The subject's code.
    pub code: String,
    /// The subject's display name.
    pub name: String,
    /// The subject's type.
    pub subject_type: SubjectType,
    /// The subject's pass mark.
    pub pass_mark: f32,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl SubjectCreated {
    /// Mints a fresh `SubjectCreated`.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        subject_id: SubjectId,
        code: String,
        name: String,
        subject_type: SubjectType,
        pass_mark: f32,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            subject_id,
            code,
            name,
            subject_type,
            pass_mark,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SubjectCreated {
    const EVENT_TYPE: &'static str = "academic.subject.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "subject";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.subject_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.subject_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A subject's mutable fields were updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubjectUpdated {
    /// The subject's typed id.
    pub subject_id: SubjectId,
    /// The names of the fields that actually changed.
    pub changed_fields: Vec<String>,
    /// The new name (if changed).
    pub name: Option<String>,
    /// The new subject type (if changed).
    pub subject_type: Option<SubjectType>,
    /// The new pass mark (if changed).
    pub pass_mark: Option<f32>,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl SubjectUpdated {
    /// Mints a fresh `SubjectUpdated`.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        subject_id: SubjectId,
        changed_fields: Vec<String>,
        name: Option<String>,
        subject_type: Option<SubjectType>,
        pass_mark: Option<f32>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            subject_id,
            changed_fields,
            name,
            subject_type,
            pass_mark,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SubjectUpdated {
    const EVENT_TYPE: &'static str = "academic.subject.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "subject";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.subject_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.subject_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A subject was deleted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubjectDeleted {
    /// The subject's typed id.
    pub subject_id: SubjectId,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl SubjectDeleted {
    /// Mints a fresh `SubjectDeleted`.
    #[must_use]
    pub const fn new(
        subject_id: SubjectId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            subject_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SubjectDeleted {
    const EVENT_TYPE: &'static str = "academic.subject.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "subject";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.subject_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.subject_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// AcademicYear events (5)
// =============================================================================

/// An academic year was created.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcademicYearCreated {
    /// The academic year's typed id.
    pub academic_year_id: AcademicYearId,
    /// The short label (e.g. "2026").
    pub year: String,
    /// The display title.
    pub title: String,
    /// The start date.
    pub start_date: NaiveDate,
    /// The end date.
    pub end_date: NaiveDate,
    /// Whether this is the current academic year.
    pub is_current: bool,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl AcademicYearCreated {
    /// Mints a fresh `AcademicYearCreated`.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        academic_year_id: AcademicYearId,
        year: String,
        title: String,
        start_date: NaiveDate,
        end_date: NaiveDate,
        is_current: bool,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            academic_year_id,
            year,
            title,
            start_date,
            end_date,
            is_current,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for AcademicYearCreated {
    const EVENT_TYPE: &'static str = "academic.academic_year.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "academic_year";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.academic_year_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.academic_year_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// An academic year's date range was updated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcademicYearDatesUpdated {
    /// The academic year's typed id.
    pub academic_year_id: AcademicYearId,
    /// The new start date.
    pub from: NaiveDate,
    /// The new end date.
    pub to: NaiveDate,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl AcademicYearDatesUpdated {
    /// Mints a fresh `AcademicYearDatesUpdated`.
    #[must_use]
    pub const fn new(
        academic_year_id: AcademicYearId,
        from: NaiveDate,
        to: NaiveDate,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            academic_year_id,
            from,
            to,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for AcademicYearDatesUpdated {
    const EVENT_TYPE: &'static str = "academic.academic_year.dates_updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "academic_year";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.academic_year_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.academic_year_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A new current academic year was set. The previous current
/// year (if any) is demoted by the storage adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentAcademicYearSet {
    /// The newly-current academic year's typed id.
    pub academic_year_id: AcademicYearId,
    /// The previous current academic year's typed id, if any.
    pub previous_id: Option<AcademicYearId>,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl CurrentAcademicYearSet {
    /// Mints a fresh `CurrentAcademicYearSet`.
    #[must_use]
    pub const fn new(
        academic_year_id: AcademicYearId,
        previous_id: Option<AcademicYearId>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            academic_year_id,
            previous_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for CurrentAcademicYearSet {
    const EVENT_TYPE: &'static str = "academic.academic_year.current_set";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "academic_year";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.academic_year_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.academic_year_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// An academic year was closed (made read-only).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcademicYearClosed {
    /// The academic year's typed id.
    pub academic_year_id: AcademicYearId,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl AcademicYearClosed {
    /// Mints a fresh `AcademicYearClosed`.
    #[must_use]
    pub const fn new(
        academic_year_id: AcademicYearId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            academic_year_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for AcademicYearClosed {
    const EVENT_TYPE: &'static str = "academic.academic_year.closed";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "academic_year";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.academic_year_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.academic_year_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// An academic year was deep-copied from a source year
/// (classes, sections, subjects, class-subjects, routines
/// — but not students).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcademicYearCopied {
    /// The newly-created academic year's typed id.
    pub to_academic_year_id: AcademicYearId,
    /// The source academic year's typed id.
    pub from_academic_year_id: AcademicYearId,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl AcademicYearCopied {
    /// Mints a fresh `AcademicYearCopied`.
    #[must_use]
    pub const fn new(
        to_academic_year_id: AcademicYearId,
        from_academic_year_id: AcademicYearId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            to_academic_year_id,
            from_academic_year_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for AcademicYearCopied {
    const EVENT_TYPE: &'static str = "academic.academic_year.copied";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "academic_year";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.to_academic_year_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.to_academic_year_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// Placeholder events for the remaining 14 academic aggregates
// (introduced in commit 18d67df).
//
// Minimal `id + school_id + aggregate_id + occurred_at` stubs so
// the spec is exhaustively representable in code and so that the
// `DomainEvent` trait is satisfied for downstream subscribers and
// envelope emission. The full impl (domain payload fields,
// causation metadata, factory constructors) lands in subsequent
// workstreams per `docs/build-plan.md`.
//
// Each stub carries:
//   - `event_id`:      the canonical EventId stamped on mint
//   - `school_id`:     the tenant anchor (must match
//                      `aggregate_id.school_id()`)
//   - `aggregate_id`:  the typed id of the owning aggregate
//                      (school_id + uuid, derived)
//   - `occurred_at`:   the mint-time timestamp required by the
//                      `DomainEvent` trait
//
// The type system catches cross-tenant confusion at compile time:
// the typed aggregate id embeds `school_id`, and the explicit
// `school_id` field is asserted to match in the eventual full impl.
// =============================================================================

macro_rules! academic_event_stub {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident {
            aggregate_id: $id_ty:ty,
            event_type: $event_type:expr,
            aggregate_type: $aggregate_type:expr $(,)?
        }
    ) => {
        $(#[$attr])*
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        $vis struct $name {
            /// The canonical event id.
            pub event_id: EventId,
            /// The owning school (derived from
            /// `aggregate_id.school_id()` in real impl; held
            /// explicitly here so the placeholder is
            /// self-contained).
            pub school_id: SchoolId,
            /// The typed id (school_id + uuid) of the owning
            /// aggregate.
            pub aggregate_id: $id_ty,
            /// Clock time of the event (required by
            /// `DomainEvent`).
            pub occurred_at: Timestamp,
        }

        impl DomainEvent for $name {
            const EVENT_TYPE: &'static str = $event_type;
            const SCHEMA_VERSION: u32 = 1;
            const AGGREGATE_TYPE: &'static str = $aggregate_type;

            fn event_id(&self) -> EventId {
                self.event_id
            }
            fn aggregate_id(&self) -> Uuid {
                self.aggregate_id.as_uuid()
            }
            fn school_id(&self) -> SchoolId {
                self.school_id
            }
            fn occurred_at(&self) -> Timestamp {
                self.occurred_at
            }
        }
    };
}

// =============================================================================
// Guardian events (full impl — Batch 1)
// =============================================================================

/// A guardian was registered.
///
/// Per `docs/specs/academic/aggregates.md` § Guardian, the
/// event carries the typed id, the contact details
/// (validated), and the audit metadata. Per I-1, the
/// `phone` and `email` fields are `Option<…>` and at most
/// one of each is carried per guardian.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardianRegistered {
    /// The guardian's typed id.
    pub guardian_id: GuardianId,
    /// The guardian's first name.
    pub first_name: String,
    /// The guardian's last name.
    pub last_name: String,
    /// The guardian's phone of record (validated).
    pub phone: Option<crate::value_objects::PhoneNumber>,
    /// The guardian's email of record (validated).
    pub email: Option<crate::value_objects::EmailAddress>,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl GuardianRegistered {
    /// Mints a fresh `GuardianRegistered`.
    #[must_use]
    pub const fn new(
        guardian_id: GuardianId,
        first_name: String,
        last_name: String,
        phone: Option<crate::value_objects::PhoneNumber>,
        email: Option<crate::value_objects::EmailAddress>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            guardian_id,
            first_name,
            last_name,
            phone,
            email,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for GuardianRegistered {
    const EVENT_TYPE: &'static str = "academic.guardian.registered";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "guardian";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.guardian_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.guardian_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A guardian was linked to a student.
///
/// Per `docs/specs/academic/aggregates.md` § Guardian § I-2
/// and § I-3.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardianLinkedToStudent {
    /// The link's typed id.
    pub link_id: StudentGuardianLinkId,
    /// The guardian being linked.
    pub guardian_id: GuardianId,
    /// The student being linked.
    pub student_id: StudentId,
    /// The relationship.
    pub relation: crate::value_objects::Relation,
    /// Whether this link is primary for the student.
    pub is_primary: bool,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl GuardianLinkedToStudent {
    /// Mints a fresh `GuardianLinkedToStudent`.
    #[must_use]
    pub const fn new(
        link_id: StudentGuardianLinkId,
        guardian_id: GuardianId,
        student_id: StudentId,
        relation: crate::value_objects::Relation,
        is_primary: bool,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            link_id,
            guardian_id,
            student_id,
            relation,
            is_primary,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for GuardianLinkedToStudent {
    const EVENT_TYPE: &'static str = "academic.guardian.linked_to_student";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "student_guardian_link";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.link_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.link_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A guardian was unlinked from a student.
///
/// Per `docs/specs/academic/aggregates.md` § Guardian § I-5,
/// when the last link for a guardian is removed the
/// guardian is soft-deleted (the service emits an
/// additional `GuardianSoftDeleted` companion event when
/// `active_status` flips to `Retired`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardianUnlinkedFromStudent {
    /// The link's typed id.
    pub link_id: StudentGuardianLinkId,
    /// The guardian that was linked.
    pub guardian_id: GuardianId,
    /// The student that was linked.
    pub student_id: StudentId,
    /// Whether the guardian was retired by this unlink
    /// (true when this was the last link for the guardian).
    pub guardian_retired: bool,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl GuardianUnlinkedFromStudent {
    /// Mints a fresh `GuardianUnlinkedFromStudent`.
    #[must_use]
    pub const fn new(
        link_id: StudentGuardianLinkId,
        guardian_id: GuardianId,
        student_id: StudentId,
        guardian_retired: bool,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            link_id,
            guardian_id,
            student_id,
            guardian_retired,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for GuardianUnlinkedFromStudent {
    const EVENT_TYPE: &'static str = "academic.guardian.unlinked_from_student";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "student_guardian_link";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.link_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.link_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A guardian link was marked as primary.
///
/// Per `docs/specs/academic/aggregates.md` § Guardian § I-4,
/// at most one guardian per student may be primary. The
/// event records the new primary link; downstream consumers
/// can derive the demoted link from the link that is no
/// longer primary for the same student.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimaryGuardianMarked {
    /// The link's typed id.
    pub link_id: StudentGuardianLinkId,
    /// The guardian being marked primary.
    pub guardian_id: GuardianId,
    /// The student whose primary is being set.
    pub student_id: StudentId,
    /// The link (if any) that was previously primary for
    /// the same student. None if no prior primary existed.
    pub demoted_from: Option<StudentGuardianLinkId>,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl PrimaryGuardianMarked {
    /// Mints a fresh `PrimaryGuardianMarked`.
    #[must_use]
    pub const fn new(
        link_id: StudentGuardianLinkId,
        guardian_id: GuardianId,
        student_id: StudentId,
        demoted_from: Option<StudentGuardianLinkId>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            link_id,
            guardian_id,
            student_id,
            demoted_from,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for PrimaryGuardianMarked {
    const EVENT_TYPE: &'static str = "academic.guardian.primary_marked";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "student_guardian_link";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.link_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.link_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A guardian's mutable contact fields were updated.
///
/// Per `docs/specs/academic/aggregates.md` § Guardian § I-1,
/// the phone and email fields are `Option<…>` and at most
/// one of each is carried per guardian. The event payload
/// preserves the new (post-mutation) values; consumers that
/// need the diff compare against the prior aggregate state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardianContactUpdated {
    /// The guardian's typed id.
    pub guardian_id: GuardianId,
    /// The new phone of record (validated; `None` if cleared).
    pub phone: Option<crate::value_objects::PhoneNumber>,
    /// The new email of record (validated; `None` if cleared).
    pub email: Option<crate::value_objects::EmailAddress>,
    /// The names of the fields that actually changed.
    pub changed_fields: Vec<String>,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl GuardianContactUpdated {
    /// Mints a fresh `GuardianContactUpdated`.
    #[must_use]
    pub const fn new(
        guardian_id: GuardianId,
        phone: Option<crate::value_objects::PhoneNumber>,
        email: Option<crate::value_objects::EmailAddress>,
        changed_fields: Vec<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            guardian_id,
            phone,
            email,
            changed_fields,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for GuardianContactUpdated {
    const EVENT_TYPE: &'static str = "academic.guardian.contact_updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "guardian";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.guardian_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.guardian_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A guardian was soft-deleted (`active_status = Retired`).
///
/// Per `docs/specs/academic/aggregates.md` § Guardian § I-5,
/// the engine emits this companion event when the last
/// student link is removed (the `unlink_guardian_from_student`
/// service signals the transition via
/// `guardian_retired: true` on the unlink event; the
/// `retire_guardian` service emits this event directly).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardianRetired {
    /// The guardian's typed id.
    pub guardian_id: GuardianId,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl GuardianRetired {
    /// Mints a fresh `GuardianRetired`.
    #[must_use]
    pub const fn new(
        guardian_id: GuardianId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            guardian_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for GuardianRetired {
    const EVENT_TYPE: &'static str = "academic.guardian.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "guardian";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.guardian_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.guardian_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// An optional subject was assigned to a student for an academic year.
///
/// Per `docs/specs/academic/aggregates.md` § Student § I-4.
/// This is the **new** event variant emitted by the
/// `OptionalSubjectAssignment` aggregate (the older
/// `OptionalSubjectAssigned` event from the Student
/// aggregate remains for backward compatibility).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionalSubjectAssignmentCreated {
    /// The assignment's typed id.
    pub assignment_id: OptionalSubjectAssignmentId,
    /// The student receiving the optional subject.
    pub student_id: StudentId,
    /// The optional subject's typed id.
    pub subject_id: SubjectId,
    /// The academic year the assignment applies to.
    pub academic_year_id: AcademicYearId,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl OptionalSubjectAssignmentCreated {
    /// Mints a fresh `OptionalSubjectAssignmentCreated`.
    #[must_use]
    pub const fn new(
        assignment_id: OptionalSubjectAssignmentId,
        student_id: StudentId,
        subject_id: SubjectId,
        academic_year_id: AcademicYearId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            assignment_id,
            student_id,
            subject_id,
            academic_year_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for OptionalSubjectAssignmentCreated {
    const EVENT_TYPE: &'static str = "academic.optional_subject_assignment.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "optional_subject_assignment";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.assignment_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.assignment_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}
// =============================================================================
// ClassSection events (full impl — Batch 2)
// =============================================================================

/// A [`ClassSection`](crate::aggregate::ClassSection) pairing
/// of a class, a section, and an academic year was created.
///
/// Per `docs/specs/academic/aggregates.md` § ClassSection,
/// the event carries the typed id plus the three
/// dimension ids (`class_id`, `section_id`,
/// `academic_year_id`), the initial `class_rooms` list,
/// and the audit metadata. Per I-3, `class_rooms` is
/// non-empty by construction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassSectionCreated {
    /// The class-section's typed id.
    pub class_section_id: ClassSectionId,
    /// The class this section is a division of.
    pub class_id: ClassId,
    /// The section within the class.
    pub section_id: SectionId,
    /// The academic year this pairing applies to.
    pub academic_year_id: AcademicYearId,
    /// The class rooms assigned to this section. Per I-3,
    /// at least one entry.
    pub class_rooms: Vec<ClassRoomId>,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered
    /// the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl ClassSectionCreated {
    /// Mints a fresh `ClassSectionCreated`.
    #[must_use]
    pub const fn new(
        class_section_id: ClassSectionId,
        class_id: ClassId,
        section_id: SectionId,
        academic_year_id: AcademicYearId,
        class_rooms: Vec<ClassRoomId>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            class_section_id,
            class_id,
            section_id,
            academic_year_id,
            class_rooms,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ClassSectionCreated {
    const EVENT_TYPE: &'static str = "academic.class_section.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "class_section";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.class_section_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.class_section_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A class teacher was assigned to a [`ClassSection`].
///
/// Per `docs/specs/academic/aggregates.md` § ClassSection §
/// I-2 (permissive), multiple class teachers per section
/// are allowed; this event records the assignment of one
/// additional teacher.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassTeacherAssigned {
    /// The class-section's typed id.
    pub class_section_id: ClassSectionId,
    /// The teacher being assigned.
    pub teacher_id: educore_core::ids::UserId,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered
    /// the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl ClassTeacherAssigned {
    /// Mints a fresh `ClassTeacherAssigned`.
    #[must_use]
    pub const fn new(
        class_section_id: ClassSectionId,
        teacher_id: educore_core::ids::UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            class_section_id,
            teacher_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ClassTeacherAssigned {
    const EVENT_TYPE: &'static str = "academic.class_section.class_teacher_assigned";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "class_section";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.class_section_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.class_section_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A subject teacher was assigned to a [`ClassSection`] for
/// a specific subject.
///
/// Per `docs/specs/academic/aggregates.md` § ClassSection §
/// I-2 (permissive), multiple subject teachers per section
/// are allowed; this event records the assignment of one
/// additional teacher for the given subject.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubjectTeacherAssigned {
    /// The class-section's typed id.
    pub class_section_id: ClassSectionId,
    /// The subject the teacher is assigned to.
    pub subject_id: SubjectId,
    /// The teacher being assigned.
    pub teacher_id: educore_core::ids::UserId,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered
    /// the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl SubjectTeacherAssigned {
    /// Mints a fresh `SubjectTeacherAssigned`.
    #[must_use]
    pub const fn new(
        class_section_id: ClassSectionId,
        subject_id: SubjectId,
        teacher_id: educore_core::ids::UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            class_section_id,
            subject_id,
            teacher_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SubjectTeacherAssigned {
    const EVENT_TYPE: &'static str = "academic.class_section.subject_teacher_assigned";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "class_section";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.class_section_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.class_section_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// An additional class room was assigned to a
/// [`ClassSection`].
///
/// Per `docs/specs/academic/aggregates.md` § ClassSection §
/// I-3, every class-section has one or more class rooms.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassRoomAssigned {
    /// The class-section's typed id.
    pub class_section_id: ClassSectionId,
    /// The class room being assigned.
    pub class_room_id: ClassRoomId,
    /// The class-section's full `class_rooms` list after
    /// the append (the union of the prior list and the
    /// new entry).
    pub class_rooms: Vec<ClassRoomId>,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered
    /// the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl ClassRoomAssigned {
    /// Mints a fresh `ClassRoomAssigned`.
    #[must_use]
    pub const fn new(
        class_section_id: ClassSectionId,
        class_room_id: ClassRoomId,
        class_rooms: Vec<ClassRoomId>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            class_section_id,
            class_room_id,
            class_rooms,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ClassRoomAssigned {
    const EVENT_TYPE: &'static str = "academic.class_section.class_room_assigned";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "class_section";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.class_section_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.class_section_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A [`ClassSection`] was soft-deleted.
///
/// Per `docs/specs/academic/aggregates.md` § ClassSection §
/// I-4, the service emits this event only after verifying
/// that no `StudentRecord` references the class-section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassSectionDeleted {
    /// The class-section's typed id.
    pub class_section_id: ClassSectionId,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered
    /// the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl ClassSectionDeleted {
    /// Mints a fresh `ClassSectionDeleted`.
    #[must_use]
    pub const fn new(
        class_section_id: ClassSectionId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            class_section_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ClassSectionDeleted {
    const EVENT_TYPE: &'static str = "academic.class_section.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "class_section";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.class_section_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.class_section_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}
// =============================================================================
// ClassSubject events (full impl — Batch 2)
// =============================================================================

/// A subject was assigned to a class (or class-section),
/// with a teacher.
///
/// Per `docs/specs/academic/aggregates.md` § ClassSubject,
/// the event carries the typed id plus the dimension ids
/// (`class_id`, `class_section_id`, `subject_id`,
/// `teacher_id`), the [`ClassSubjectScope`], the optional
/// `PassMark` override, and the audit metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubjectAssignedToClass {
    /// The class-subject's typed id.
    pub class_subject_id: ClassSubjectId,
    /// The class this subject is assigned to.
    pub class_id: ClassId,
    /// The class-section this subject is assigned to
    /// (`None` when `scope == ClassOnly`).
    pub class_section_id: Option<ClassSectionId>,
    /// The subject being assigned.
    pub subject_id: SubjectId,
    /// The teacher being assigned.
    pub teacher_id: UserId,
    /// The scope of the assignment.
    pub scope: crate::value_objects::ClassSubjectScope,
    /// Optional per-class-subject `PassMark` override.
    pub pass_mark: Option<crate::value_objects::PassMark>,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered
    /// the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl SubjectAssignedToClass {
    /// Mints a fresh `SubjectAssignedToClass`.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        class_subject_id: ClassSubjectId,
        class_id: ClassId,
        class_section_id: Option<ClassSectionId>,
        subject_id: SubjectId,
        teacher_id: UserId,
        scope: crate::value_objects::ClassSubjectScope,
        pass_mark: Option<crate::value_objects::PassMark>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            class_subject_id,
            class_id,
            class_section_id,
            subject_id,
            teacher_id,
            scope,
            pass_mark,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SubjectAssignedToClass {
    const EVENT_TYPE: &'static str = "academic.class_subject.assigned";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "class_subject";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.class_subject_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.class_subject_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A [`ClassSubject`](crate::aggregate::ClassSubject) had its
/// teacher reassigned to a different user.
///
/// Per `docs/specs/academic/aggregates.md` § ClassSubject §
/// I-2 (permissive), the same teacher may be assigned to
/// multiple class-subjects, and a class-subject may be
/// reassigned to a different teacher without constraint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeacherReassigned {
    /// The class-subject's typed id.
    pub class_subject_id: ClassSubjectId,
    /// The previous teacher id.
    pub previous_teacher_id: UserId,
    /// The new teacher id.
    pub new_teacher_id: UserId,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered
    /// the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl TeacherReassigned {
    /// Mints a fresh `TeacherReassigned`.
    #[must_use]
    pub const fn new(
        class_subject_id: ClassSubjectId,
        previous_teacher_id: UserId,
        new_teacher_id: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            class_subject_id,
            previous_teacher_id,
            new_teacher_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for TeacherReassigned {
    const EVENT_TYPE: &'static str = "academic.class_subject.teacher_reassigned";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "class_subject";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.class_subject_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.class_subject_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A [`ClassSubject`](crate::aggregate::ClassSubject) was
/// unassigned (soft-retired).
///
/// Per `docs/specs/academic/aggregates.md` § ClassSubject,
/// unassignment is unconditional: the service retires the
/// aggregate regardless of any cross-references.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubjectUnassigned {
    /// The class-subject's typed id.
    pub class_subject_id: ClassSubjectId,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered
    /// the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl SubjectUnassigned {
    /// Mints a fresh `SubjectUnassigned`.
    #[must_use]
    pub const fn new(
        class_subject_id: ClassSubjectId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            class_subject_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SubjectUnassigned {
    const EVENT_TYPE: &'static str = "academic.class_subject.unassigned";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "class_subject";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.class_subject_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.class_subject_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Backward-compatible alias for [`SubjectAssignedToClass`].
///
/// The earlier (placeholder-era) name `ClassSubjectAssigned`
/// is retained so older call sites compile unchanged. New
/// callers should prefer [`SubjectAssignedToClass`].
#[deprecated(
    since = "0.1.0",
    note = "renamed to SubjectAssignedToClass per spec; alias retained for backward compat"
)]
pub type ClassSubjectAssigned = SubjectAssignedToClass;

// =============================================================================
// ClassRoutine events (Wave 51: full impl)
// =============================================================================

/// A [`ClassRoutine`](crate::aggregate::ClassRoutine) was
/// scheduled.
///
/// Per `docs/specs/academic/aggregates.md` § ClassRoutine,
/// the event carries the typed id plus the dimension ids
/// (`class_section_id`, `academic_year_id`), the
/// full-week period schedule, and the audit metadata.
/// The create flow enforces I-1 / I-2 / I-3 / I-4 / I-5
/// (see [`crate::aggregate::ClassRoutine::fresh`] and
/// `crate::services::create_class_routine`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassRoutineScheduled {
    /// The class-routine's typed id.
    pub class_routine_id: ClassRoutineId,
    /// The class-section this routine is scheduled for.
    pub class_section_id: ClassSectionId,
    /// The academic year this routine applies to.
    pub academic_year_id: AcademicYearId,
    /// The full-week period schedule. Per I-1, covers
    /// all 7 distinct days. Per I-2, no duplicate
    /// `ClassTimeId`.
    pub periods: Vec<crate::value_objects::ClassPeriod>,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered
    /// the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl ClassRoutineScheduled {
    /// Mints a fresh `ClassRoutineScheduled`.
    #[must_use]
    pub const fn new(
        class_routine_id: ClassRoutineId,
        class_section_id: ClassSectionId,
        academic_year_id: AcademicYearId,
        periods: Vec<crate::value_objects::ClassPeriod>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            class_routine_id,
            class_section_id,
            academic_year_id,
            periods,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ClassRoutineScheduled {
    const EVENT_TYPE: &'static str = "academic.class_routine.scheduled";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "class_routine";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.class_routine_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.class_routine_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A [`ClassRoutine`](crate::aggregate::ClassRoutine) had
/// its period set replaced.
///
/// Per `docs/specs/academic/aggregates.md` § ClassRoutine,
/// the update flow enforces I-1 / I-2 / I-3 on the new
/// period set (see
/// [`crate::aggregate::ClassRoutine::replace_periods`]).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassRoutinePeriodUpdated {
    /// The class-routine's typed id.
    pub class_routine_id: ClassRoutineId,
    /// The previous period set (before the update).
    pub previous_periods: Vec<crate::value_objects::ClassPeriod>,
    /// The replacement period set (after the update).
    pub new_periods: Vec<crate::value_objects::ClassPeriod>,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered
    /// the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl ClassRoutinePeriodUpdated {
    /// Mints a fresh `ClassRoutinePeriodUpdated`.
    #[must_use]
    pub const fn new(
        class_routine_id: ClassRoutineId,
        previous_periods: Vec<crate::value_objects::ClassPeriod>,
        new_periods: Vec<crate::value_objects::ClassPeriod>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            class_routine_id,
            previous_periods,
            new_periods,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ClassRoutinePeriodUpdated {
    const EVENT_TYPE: &'static str = "academic.class_routine.period_updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "class_routine";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.class_routine_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.class_routine_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A [`ClassRoutine`](crate::aggregate::ClassRoutine) had
/// two of its periods swapped by index.
///
/// Per `docs/specs/academic/aggregates.md` § ClassRoutine,
/// the swap exchanges the full [`ClassPeriod`]
/// payloads (including `room_id`, `teacher_id`, `day`,
/// `period_number`, `class_time_id`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassRoutinePeriodsSwapped {
    /// The class-routine's typed id.
    pub class_routine_id: ClassRoutineId,
    /// The index of the first period swapped.
    pub period_a_idx: usize,
    /// The index of the second period swapped.
    pub period_b_idx: usize,
    /// The previous period at index `period_a_idx`.
    pub previous_period_a: crate::value_objects::ClassPeriod,
    /// The previous period at index `period_b_idx`.
    pub previous_period_b: crate::value_objects::ClassPeriod,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered
    /// the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl ClassRoutinePeriodsSwapped {
    /// Mints a fresh `ClassRoutinePeriodsSwapped`.
    #[must_use]
    pub const fn new(
        class_routine_id: ClassRoutineId,
        period_a_idx: usize,
        period_b_idx: usize,
        previous_period_a: crate::value_objects::ClassPeriod,
        previous_period_b: crate::value_objects::ClassPeriod,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            class_routine_id,
            period_a_idx,
            period_b_idx,
            previous_period_a,
            previous_period_b,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ClassRoutinePeriodsSwapped {
    const EVENT_TYPE: &'static str = "academic.class_routine.periods_swapped";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "class_routine";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.class_routine_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.class_routine_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A [`ClassRoutine`](crate::aggregate::ClassRoutine) was
/// soft-deleted (retired).
///
/// Per `docs/specs/academic/aggregates.md` § ClassRoutine,
/// the delete flow is unconditional: any active
/// `ClassRoutine` may be deleted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassRoutineDeleted {
    /// The class-routine's typed id.
    pub class_routine_id: ClassRoutineId,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered
    /// the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl ClassRoutineDeleted {
    /// Mints a fresh `ClassRoutineDeleted`.
    #[must_use]
    pub const fn new(
        class_routine_id: ClassRoutineId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            class_routine_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ClassRoutineDeleted {
    const EVENT_TYPE: &'static str = "academic.class_routine.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "class_routine";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.class_routine_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.class_routine_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}
academic_event_stub! {
    /// Event stub: a homework assignment was issued. See
    /// `docs/specs/academic/aggregates.md` § Homework.
    pub struct HomeworkAssigned {
        aggregate_id: HomeworkId,
        event_type: "academic.homework.assigned",
        aggregate_type: "homework",
    }
}
academic_event_stub! {
    /// Event stub: a lesson plan was drafted. See
    /// `docs/specs/academic/aggregates.md` § LessonPlan.
    pub struct LessonPlanCreated {
        aggregate_id: LessonPlanId,
        event_type: "academic.lesson_plan.created",
        aggregate_type: "lesson_plan",
    }
}
academic_event_stub! {
    /// Event stub: a lesson was created. See
    /// `docs/specs/academic/aggregates.md` § Lesson.
    pub struct LessonCreated {
        aggregate_id: LessonId,
        event_type: "academic.lesson.created",
        aggregate_type: "lesson",
    }
}
academic_event_stub! {
    /// Event stub: a lesson topic was created. See
    /// `docs/specs/academic/aggregates.md` § LessonTopic.
    pub struct LessonTopicCreated {
        aggregate_id: LessonTopicId,
        event_type: "academic.lesson_topic.created",
        aggregate_type: "lesson_topic",
    }
}
academic_event_stub! {
    /// Event stub: a student promotion was recorded. See
    /// `docs/specs/academic/aggregates.md` § StudentPromotion.
    pub struct StudentPromotionRecorded {
        aggregate_id: StudentPromotionId,
        event_type: "academic.student_promotion.recorded",
        aggregate_type: "student_promotion",
    }
}
academic_event_stub! {
    /// Event stub: a student category was created. See
    /// `docs/specs/academic/aggregates.md` § StudentCategory.
    pub struct StudentCategoryCreated {
        aggregate_id: StudentCategoryId,
        event_type: "academic.student_category.created",
        aggregate_type: "student_category",
    }
}
academic_event_stub! {
    /// Event stub: a student group was created. See
    /// `docs/specs/academic/aggregates.md` § StudentGroup.
    pub struct StudentGroupCreated {
        aggregate_id: StudentGroupId,
        event_type: "academic.student_group.created",
        aggregate_type: "student_group",
    }
}
academic_event_stub! {
    /// Event stub: a registration custom field was created. See
    /// `docs/specs/academic/aggregates.md` § RegistrationField.
    pub struct RegistrationFieldCreated {
        aggregate_id: RegistrationFieldId,
        event_type: "academic.registration_field.created",
        aggregate_type: "registration_field",
    }
}
academic_event_stub! {
    /// Event stub: a certificate template was created. See
    /// `docs/specs/academic/aggregates.md` § Certificate.
    pub struct CertificateCreated {
        aggregate_id: CertificateId,
        event_type: "academic.certificate.created",
        aggregate_type: "certificate",
    }
}
academic_event_stub! {
    /// Event stub: an ID card template was created. See
    /// `docs/specs/academic/aggregates.md` § IdCard.
    pub struct IdCardCreated {
        aggregate_id: IdCardId,
        event_type: "academic.id_card.created",
        aggregate_type: "id_card",
    }
}

// =============================================================================
// Newly added events (Cluster D final — minimal placeholder structs so the
// `educore-core::lint` spec_to_code check passes). Each carries the typed
// fields declared in `docs/specs/academic/events.md` plus the standard
// `event_id` / `correlation_id` / `occurred_at` envelope fields. The full
// event payload (factory constructors, causation metadata, storage-side
// publish paths) lands alongside the owning aggregates in later workstreams.
// =============================================================================

/// A student was assigned to a section within their current class.
///
/// Per `docs/specs/academic/events.md` § StudentAssignedToSection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StudentAssignedToSection {
    /// The student's typed id.
    pub student_id: StudentId,
    /// The section the student is moving from (if any).
    pub from_section_id: Option<SectionId>,
    /// The section the student is being assigned to.
    pub to_section_id: SectionId,
    /// The reason for the assignment.
    pub reason: String,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl StudentAssignedToSection {
    /// Mints a fresh `StudentAssignedToSection`.
    #[must_use]
    pub const fn new(
        student_id: StudentId,
        from_section_id: Option<SectionId>,
        to_section_id: SectionId,
        reason: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            student_id,
            from_section_id,
            to_section_id,
            reason,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StudentAssignedToSection {
    const EVENT_TYPE: &'static str = "academic.student.assigned_to_section";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "student";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.student_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.student_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A student's category assignment changed.
///
/// Per `docs/specs/academic/events.md` § StudentCategoryChanged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StudentCategoryChanged {
    /// The student's typed id.
    pub student_id: StudentId,
    /// The category the student was previously assigned to (if any).
    pub from_category_id: Option<StudentCategoryId>,
    /// The category the student is being assigned to.
    pub to_category_id: StudentCategoryId,
    /// The first day the new category is effective.
    pub effective_from: NaiveDate,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl StudentCategoryChanged {
    /// Mints a fresh `StudentCategoryChanged`.
    #[must_use]
    pub const fn new(
        student_id: StudentId,
        from_category_id: Option<StudentCategoryId>,
        to_category_id: StudentCategoryId,
        effective_from: NaiveDate,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            student_id,
            from_category_id,
            to_category_id,
            effective_from,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StudentCategoryChanged {
    const EVENT_TYPE: &'static str = "academic.student.category_changed";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "student";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.student_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.student_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// An optional subject was assigned to a student for an academic year.
///
/// Per `docs/specs/academic/events.md` § OptionalSubjectAssigned.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionalSubjectAssigned {
    /// The student's typed id.
    pub student_id: StudentId,
    /// The optional subject's typed id.
    pub subject_id: SubjectId,
    /// The academic year the assignment applies to.
    pub academic_year_id: AcademicYearId,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl OptionalSubjectAssigned {
    /// Mints a fresh `OptionalSubjectAssigned`.
    #[must_use]
    pub const fn new(
        student_id: StudentId,
        subject_id: SubjectId,
        academic_year_id: AcademicYearId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            student_id,
            subject_id,
            academic_year_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for OptionalSubjectAssigned {
    const EVENT_TYPE: &'static str = "academic.optional_subject.assigned";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "student";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.student_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.student_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A document was uploaded for a student.
///
/// Per `docs/specs/academic/events.md` § StudentDocumentUploaded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StudentDocumentUploaded {
    /// The student's typed id.
    pub student_id: StudentId,
    /// The uploaded document's typed id.
    pub document_id: StudentDocumentId,
    /// The document's display title.
    pub title: String,
    /// The file path or URI of the uploaded document.
    pub file: String,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl StudentDocumentUploaded {
    /// Mints a fresh `StudentDocumentUploaded`.
    #[must_use]
    pub const fn new(
        student_id: StudentId,
        document_id: StudentDocumentId,
        title: String,
        file: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            student_id,
            document_id,
            title,
            file,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StudentDocumentUploaded {
    const EVENT_TYPE: &'static str = "academic.student_document.uploaded";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "student_document";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.document_id.1
    }
    fn school_id(&self) -> SchoolId {
        self.document_id.0
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
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
    use educore_core::ids::{Identifier, UserId};
    use educore_core::tenant::{TenantContext, UserType};

    fn school() -> SchoolId {
        SchoolId::from_uuid(Uuid::now_v7())
    }

    fn ctx() -> TenantContext {
        let user = UserId::from_uuid(Uuid::now_v7());
        TenantContext::for_user(
            school(),
            user,
            educore_core::ids::CorrelationId::from_uuid(Uuid::now_v7()),
            UserType::SchoolAdmin,
        )
    }

    #[test]
    fn student_admitted_event_type_is_namespaced() {
        let s = school();
        let id = StudentId::new(s, Uuid::now_v7());
        let class = ClassId::new(s, Uuid::now_v7());
        let section = SectionId::new(s, Uuid::now_v7());
        let year = AcademicYearId::new(s, Uuid::now_v7());
        let corr = educore_core::ids::CorrelationId::from_uuid(Uuid::now_v7());
        let event_id = EventId::from_uuid(Uuid::now_v7());
        let event = StudentAdmitted::new(
            id,
            "ADM-001".to_owned(),
            "Ada".to_owned(),
            "Lovelace".to_owned(),
            class,
            section,
            year,
            NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            Some("1".to_owned()),
            event_id,
            corr,
            Timestamp::now(),
        );
        assert_eq!(
            <StudentAdmitted as DomainEvent>::EVENT_TYPE,
            "academic.student.admitted"
        );
        assert_eq!(<StudentAdmitted as DomainEvent>::AGGREGATE_TYPE, "student");
        assert_eq!(event.school_id(), s);
        assert_eq!(event.aggregate_id(), id.as_uuid());
        let env = event.into_envelope(&ctx());
        assert_eq!(env.event_type, "academic.student.admitted");
        assert_eq!(env.school_id, s);
    }

    #[test]
    fn all_student_event_types_are_namespaced() {
        let id = StudentId::new(school(), Uuid::now_v7());
        let class = ClassId::new(school(), Uuid::now_v7());
        let section = SectionId::new(school(), Uuid::now_v7());
        let year = AcademicYearId::new(school(), Uuid::now_v7());
        let corr = educore_core::ids::CorrelationId::from_uuid(Uuid::now_v7());
        let event_id = EventId::from_uuid(Uuid::now_v7());
        let now = Timestamp::now();
        let nd = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let dst = school();

        let p = StudentProfileUpdated::new(id, vec!["first_name".to_owned()], event_id, corr, now);
        assert_eq!(
            <StudentProfileUpdated as DomainEvent>::EVENT_TYPE,
            "academic.student.profile_updated"
        );
        let s = StudentSuspended::new(id, "medical".to_owned(), nd, None, event_id, corr, now);
        assert_eq!(
            <StudentSuspended as DomainEvent>::EVENT_TYPE,
            "academic.student.suspended"
        );
        let r = StudentReinstated::new(id, nd, None, event_id, corr, now);
        assert_eq!(
            <StudentReinstated as DomainEvent>::EVENT_TYPE,
            "academic.student.reinstated"
        );
        let w = StudentWithdrawn::new(id, "moved".to_owned(), nd, None, event_id, corr, now);
        assert_eq!(
            <StudentWithdrawn as DomainEvent>::EVENT_TYPE,
            "academic.student.withdrawn"
        );
        let t =
            StudentTransferred::new(id, dst, "parent's job".to_owned(), nd, event_id, corr, now);
        assert_eq!(
            <StudentTransferred as DomainEvent>::EVENT_TYPE,
            "academic.student.transferred"
        );
        let pr = StudentPromoted::new(
            id,
            class,
            section,
            class,
            section,
            year,
            year,
            "2".to_owned(),
            ResultStatus::Pass,
            event_id,
            corr,
            now,
        );
        assert_eq!(
            <StudentPromoted as DomainEvent>::EVENT_TYPE,
            "academic.student.promoted"
        );
        let g = StudentGraduated::new(id, year, nd, StudentStatus::Graduated, event_id, corr, now);
        assert_eq!(
            <StudentGraduated as DomainEvent>::EVENT_TYPE,
            "academic.student.graduated"
        );
        let _ = (p, s, r, w, t, pr, g);
    }

    #[test]
    fn class_section_subject_academic_year_event_types_are_namespaced() {
        let class = ClassId::new(school(), Uuid::now_v7());
        let section = SectionId::new(school(), Uuid::now_v7());
        let subject = SubjectId::new(school(), Uuid::now_v7());
        let year = AcademicYearId::new(school(), Uuid::now_v7());
        let corr = educore_core::ids::CorrelationId::from_uuid(Uuid::now_v7());
        let event_id = EventId::from_uuid(Uuid::now_v7());
        let now = Timestamp::now();
        let nd = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();

        assert_eq!(
            <ClassCreated as DomainEvent>::EVENT_TYPE,
            "academic.class.created"
        );
        assert_eq!(
            <ClassUpdated as DomainEvent>::EVENT_TYPE,
            "academic.class.updated"
        );
        assert_eq!(
            <OptionalSubjectGpaThresholdSet as DomainEvent>::EVENT_TYPE,
            "academic.class.optional_subject_gpa_threshold_set"
        );
        assert_eq!(
            <ClassDeleted as DomainEvent>::EVENT_TYPE,
            "academic.class.deleted"
        );
        assert_eq!(
            <SectionCreated as DomainEvent>::EVENT_TYPE,
            "academic.section.created"
        );
        assert_eq!(
            <SectionUpdated as DomainEvent>::EVENT_TYPE,
            "academic.section.updated"
        );
        assert_eq!(
            <SectionDeleted as DomainEvent>::EVENT_TYPE,
            "academic.section.deleted"
        );
        assert_eq!(
            <SubjectCreated as DomainEvent>::EVENT_TYPE,
            "academic.subject.created"
        );
        assert_eq!(
            <SubjectUpdated as DomainEvent>::EVENT_TYPE,
            "academic.subject.updated"
        );
        assert_eq!(
            <SubjectDeleted as DomainEvent>::EVENT_TYPE,
            "academic.subject.deleted"
        );
        assert_eq!(
            <AcademicYearCreated as DomainEvent>::EVENT_TYPE,
            "academic.academic_year.created"
        );
        assert_eq!(
            <AcademicYearDatesUpdated as DomainEvent>::EVENT_TYPE,
            "academic.academic_year.dates_updated"
        );
        assert_eq!(
            <CurrentAcademicYearSet as DomainEvent>::EVENT_TYPE,
            "academic.academic_year.current_set"
        );
        assert_eq!(
            <AcademicYearClosed as DomainEvent>::EVENT_TYPE,
            "academic.academic_year.closed"
        );
        assert_eq!(
            <AcademicYearCopied as DomainEvent>::EVENT_TYPE,
            "academic.academic_year.copied"
        );
        let _ = (class, section, subject, year, corr, event_id, now, nd);
    }

    #[test]
    fn envelope_payload_serialization_round_trip() {
        let id = StudentId::new(school(), Uuid::now_v7());
        let class = ClassId::new(school(), Uuid::now_v7());
        let section = SectionId::new(school(), Uuid::now_v7());
        let year = AcademicYearId::new(school(), Uuid::now_v7());
        let event = StudentAdmitted::new(
            id,
            "ADM-001".to_owned(),
            "Ada".to_owned(),
            "Lovelace".to_owned(),
            class,
            section,
            year,
            NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            Some("1".to_owned()),
            EventId::from_uuid(Uuid::now_v7()),
            educore_core::ids::CorrelationId::from_uuid(Uuid::now_v7()),
            Timestamp::now(),
        );
        let env = event.into_envelope(&ctx());
        let value = serde_json::to_value(&env.payload).unwrap();
        assert_eq!(value["admission_no"], "ADM-001");
        assert_eq!(value["first_name"], "Ada");
        assert_eq!(value["last_name"], "Lovelace");
        assert_eq!(value["roll_no"], "1");
    }
}
