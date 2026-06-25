//! # Assessment domain events
//!
//! Phase 4 Workstream A ships the 3 [`Exam`] lifecycle
//! events: [`ExamCreated`], [`ExamUpdated`], [`ExamDeleted`].
//! Each implements [`DomainEvent`] with the
//! `"assessment.<aggregate>.<verb>"` event-type string.
//!
//! [`Exam`]: crate::aggregate::Exam
//! [`DomainEvent`]: educore_events::domain_event::DomainEvent
//!
//! The full assessment event set (ExamSchedule, MarksRegister,
//! ResultStore, OnlineExam, SeatPlan, AdmitCard, ReportCard)
//! lands in Workstreams B, C, and D. This file is extended
//! in those workstreams; the 3 events below follow the
//! same shape so the later additions are uniform.

#![allow(clippy::too_many_arguments)]
#![allow(missing_docs)] // The new() constructors are self-documenting
                        // via their parameter names; suppressing
                        // this lint for the file is the pragmatic
                        // choice for the 20+ event constructors
                        // Phase 4 ships.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use educore_core::ids::{CorrelationId, EventId, SchoolId};
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;

use educore_academic::value_objects::PassMark;

use crate::value_objects::{
    AcademicYearId, AdmitCardId, AdmitCardSettingId, ClassId, CustomResultSettingId,
    CustomTemporaryResultId, ExamAttendanceId, ExamCode, ExamId, ExamMark, ExamName,
    ExamRoutinePageId, ExamScheduleId, ExamSettingId, ExamSetupId, ExamSignatureId, ExamStepSkipId,
    ExamTypeId, FrontExamRoutineId, FrontResultId, FrontendExamResultId, MarkStoreId, MarksGradeId,
    OnlineExamId, OnlineExamMarkId, OnlineExamQuestionId, QuestionBankId, QuestionGroupId,
    QuestionLevelId, QuestionMuOptionId, ResultSettingId, SeatPlanId, SectionId,
    StudentTakeOnlineExamId, SubjectId, TeacherEvaluationId, TeacherRemarkId, TemporaryMeritListId,
};

// =============================================================================
// ExamCreated
// =============================================================================

/// Emitted when a new [`Exam`](crate::aggregate::Exam) is
/// admitted. Subscribers: the engine's `event_log` writer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExamCreated {
    /// The exam's typed id.
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
    /// The exam's human-readable name.
    pub name: String,
    /// The exam code.
    pub code: String,
    /// The exam's full mark.
    pub exam_mark: f32,
    /// The exam's pass mark.
    pub pass_mark: f32,
    /// The exam's date.
    pub exam_date: NaiveDate,
    /// The standard event-id / correlation / timestamp footer.
    /// Every event carries these three fields so the
    /// `event_log` writer can stamp them.
    pub event_id: EventId,
    /// The request correlation id (per `educore_events`).
    pub correlation_id: CorrelationId,
    /// The wall-clock time the event was minted.
    pub occurred_at: Timestamp,
}

impl ExamCreated {
    /// Constructs a new `ExamCreated` event. The `event_id` is
    /// supplied by the caller (typically the service that mints
    /// the event).
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        exam_id: ExamId,
        exam_type_id: ExamTypeId,
        class_id: ClassId,
        section_id: SectionId,
        subject_id: SubjectId,
        academic_year_id: AcademicYearId,
        name: ExamName,
        code: ExamCode,
        exam_mark: ExamMark,
        pass_mark: PassMark,
        exam_date: NaiveDate,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            exam_id,
            exam_type_id,
            class_id,
            section_id,
            subject_id,
            academic_year_id,
            name: name.as_str().to_owned(),
            code: code.as_str().to_owned(),
            exam_mark: exam_mark.as_f32(),
            pass_mark: pass_mark.as_f32(),
            exam_date,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ExamCreated {
    const EVENT_TYPE: &'static str = "assessment.exam.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "exam";

    fn event_id(&self) -> EventId {
        self.event_id
    }

    fn aggregate_id(&self) -> uuid::Uuid {
        self.exam_id.as_uuid()
    }

    fn school_id(&self) -> SchoolId {
        self.exam_id.school_id()
    }

    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// ExamUpdated
// =============================================================================

/// Emitted when an existing [`Exam`](crate::aggregate::Exam)'s
/// mutable fields change.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExamUpdated {
    /// The exam's typed id.
    pub exam_id: ExamId,
    /// The names of the fields that changed (e.g.
    /// `["exam_mark", "pass_mark"]`).
    pub changes: Vec<String>,
    /// The event id.
    pub event_id: EventId,
    /// The request correlation id.
    pub correlation_id: CorrelationId,
    /// The wall-clock time the event was minted.
    pub occurred_at: Timestamp,
}

impl ExamUpdated {
    /// Constructs a new `ExamUpdated` event.
    #[must_use]
    pub fn new(
        exam_id: ExamId,
        changes: Vec<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            exam_id,
            changes,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ExamUpdated {
    const EVENT_TYPE: &'static str = "assessment.exam.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "exam";

    fn event_id(&self) -> EventId {
        self.event_id
    }

    fn aggregate_id(&self) -> uuid::Uuid {
        self.exam_id.as_uuid()
    }

    fn school_id(&self) -> SchoolId {
        self.exam_id.school_id()
    }

    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// ExamDeleted
// =============================================================================

/// Emitted when an [`Exam`](crate::aggregate::Exam) is
/// soft-deleted. The integration test asserts that no
/// `MarksRegister` row references the deleted exam (per
/// the spec's invariant #3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExamDeleted {
    /// The exam's typed id.
    pub exam_id: ExamId,
    /// The event id.
    pub event_id: EventId,
    /// The request correlation id.
    pub correlation_id: CorrelationId,
    /// The wall-clock time the event was minted.
    pub occurred_at: Timestamp,
}

impl ExamDeleted {
    /// Constructs a new `ExamDeleted` event.
    #[must_use]
    pub const fn new(
        exam_id: ExamId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            exam_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ExamDeleted {
    const EVENT_TYPE: &'static str = "assessment.exam.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "exam";

    fn event_id(&self) -> EventId {
        self.event_id
    }

    fn aggregate_id(&self) -> uuid::Uuid {
        self.exam_id.as_uuid()
    }

    fn school_id(&self) -> SchoolId {
        self.exam_id.school_id()
    }

    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// ExamScheduled
// =============================================================================

/// Emitted when an [`ExamSchedule`](crate::aggregate::ExamSchedule)
/// is created. The integration test (Workstream D) asserts
/// the dispatch flow on this event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExamScheduled {
    /// The schedule's typed id.
    pub schedule_id: ExamScheduleId,
    /// The exam this schedule is for.
    pub exam_id: ExamId,
    /// The class this schedule covers.
    pub class_id: ClassId,
    /// The section this schedule covers.
    pub section_id: SectionId,
    /// The schedule's anchor date.
    pub date: chrono::NaiveDate,
    /// The schedule's start time.
    pub start_time: chrono::NaiveTime,
    /// The schedule's end time.
    pub end_time: chrono::NaiveTime,
    /// The number of `ExamScheduleSubject` children the
    /// dispatch created.
    pub subject_count: u32,
    /// Standard 3-field footer.
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ExamScheduled {
    /// Constructs a new `ExamScheduled` event.
    #[must_use]
    pub fn new(
        schedule_id: ExamScheduleId,
        exam_id: ExamId,
        class_id: ClassId,
        section_id: SectionId,
        date: chrono::NaiveDate,
        start_time: chrono::NaiveTime,
        end_time: chrono::NaiveTime,
        subject_count: u32,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            schedule_id,
            exam_id,
            class_id,
            section_id,
            date,
            start_time,
            end_time,
            subject_count,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ExamScheduled {
    const EVENT_TYPE: &'static str = "assessment.exam_scheduled.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "exam_schedule";

    fn event_id(&self) -> EventId {
        self.event_id
    }

    fn aggregate_id(&self) -> uuid::Uuid {
        self.schedule_id.as_uuid()
    }

    fn school_id(&self) -> SchoolId {
        self.schedule_id.school_id()
    }

    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when an `ExamSchedule` is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExamScheduleUpdated {
    pub schedule_id: ExamScheduleId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ExamScheduleUpdated {
    #[must_use]
    pub fn new(
        schedule_id: ExamScheduleId,
        changes: Vec<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            schedule_id,
            changes,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ExamScheduleUpdated {
    const EVENT_TYPE: &'static str = "assessment.exam_scheduled.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "exam_schedule";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.schedule_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.schedule_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when an `ExamSchedule` is cancelled.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExamScheduleCancelled {
    pub schedule_id: ExamScheduleId,
    pub reason: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ExamScheduleCancelled {
    #[must_use]
    pub fn new(
        schedule_id: ExamScheduleId,
        reason: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            schedule_id,
            reason,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ExamScheduleCancelled {
    const EVENT_TYPE: &'static str = "assessment.exam_scheduled.cancelled";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "exam_schedule";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.schedule_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.schedule_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// SeatPlan events
// =============================================================================

/// Emitted when a [`SeatPlan`](crate::aggregate::SeatPlan) is
/// generated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeatPlanGenerated {
    pub seat_plan_id: SeatPlanId,
    pub exam_id: ExamId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub total_students: u32,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SeatPlanGenerated {
    #[must_use]
    pub fn new(
        seat_plan_id: SeatPlanId,
        exam_id: ExamId,
        class_id: ClassId,
        section_id: SectionId,
        total_students: u32,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            seat_plan_id,
            exam_id,
            class_id,
            section_id,
            total_students,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SeatPlanGenerated {
    const EVENT_TYPE: &'static str = "assessment.seat_plan.generated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "seat_plan";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.seat_plan_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.seat_plan_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeatPlanUpdated {
    pub seat_plan_id: SeatPlanId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SeatPlanUpdated {
    #[must_use]
    pub fn new(
        seat_plan_id: SeatPlanId,
        changes: Vec<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            seat_plan_id,
            changes,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SeatPlanUpdated {
    const EVENT_TYPE: &'static str = "assessment.seat_plan.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "seat_plan";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.seat_plan_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.seat_plan_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeatPlanCancelled {
    pub seat_plan_id: SeatPlanId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SeatPlanCancelled {
    #[must_use]
    pub fn new(
        seat_plan_id: SeatPlanId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            seat_plan_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SeatPlanCancelled {
    const EVENT_TYPE: &'static str = "assessment.seat_plan.cancelled";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "seat_plan";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.seat_plan_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.seat_plan_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// AdmitCard events
// =============================================================================

/// Emitted when an [`AdmitCard`](crate::aggregate::AdmitCard)
/// is generated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdmitCardGenerated {
    pub admit_card_id: AdmitCardId,
    pub student_record_id: crate::value_objects::StudentRecordId,
    pub exam_type_id: ExamTypeId,
    pub academic_year_id: AcademicYearId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl AdmitCardGenerated {
    #[must_use]
    pub fn new(
        admit_card_id: AdmitCardId,
        student_record_id: crate::value_objects::StudentRecordId,
        exam_type_id: ExamTypeId,
        academic_year_id: AcademicYearId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            admit_card_id,
            student_record_id,
            exam_type_id,
            academic_year_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for AdmitCardGenerated {
    const EVENT_TYPE: &'static str = "assessment.admit_card.generated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "admit_card";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.admit_card_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.admit_card_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdmitCardRegenerated {
    pub admit_card_id: AdmitCardId,
    pub previous_id: AdmitCardId,
    pub reason: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl AdmitCardRegenerated {
    #[must_use]
    pub fn new(
        admit_card_id: AdmitCardId,
        previous_id: AdmitCardId,
        reason: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            admit_card_id,
            previous_id,
            reason,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for AdmitCardRegenerated {
    const EVENT_TYPE: &'static str = "assessment.admit_card.regenerated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "admit_card";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.admit_card_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.admit_card_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdmitCardCancelled {
    pub admit_card_id: AdmitCardId,
    pub reason: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl AdmitCardCancelled {
    #[must_use]
    pub fn new(
        admit_card_id: AdmitCardId,
        reason: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            admit_card_id,
            reason,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for AdmitCardCancelled {
    const EVENT_TYPE: &'static str = "assessment.admit_card.cancelled";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "admit_card";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.admit_card_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.admit_card_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
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
    use educore_core::tenant::{TenantContext, UserType};
    use educore_events::domain_event::DomainEvent;

    fn ctx(school: SchoolId) -> TenantContext {
        let actor = educore_core::ids::UserId(uuid::Uuid::now_v7());
        TenantContext::for_user(
            school,
            actor,
            educore_core::ids::CorrelationId(uuid::Uuid::now_v7()),
            UserType::SchoolAdmin,
        )
    }

    #[test]
    fn exam_created_event_type_and_aggregate_id() {
        let s = SchoolId(uuid::Uuid::now_v7());
        let exam_id = ExamId::new(s, uuid::Uuid::now_v7());
        let ev = ExamCreated::new(
            exam_id,
            ExamTypeId::new(s, uuid::Uuid::now_v7()),
            ClassId::new(s, uuid::Uuid::now_v7()),
            SectionId::new(s, uuid::Uuid::now_v7()),
            SubjectId::new(s, uuid::Uuid::now_v7()),
            AcademicYearId::new(s, uuid::Uuid::now_v7()),
            ExamName::new("Mid-Term Mathematics").unwrap(),
            ExamCode::new("MTH-MT-2024").unwrap(),
            ExamMark::new(100.0).unwrap(),
            PassMark::new(35.0).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2024, 9, 15).unwrap(),
            EventId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
            Timestamp::now(),
        );
        assert_eq!(
            <ExamCreated as DomainEvent>::EVENT_TYPE,
            "assessment.exam.created"
        );
        assert_eq!(<ExamCreated as DomainEvent>::AGGREGATE_TYPE, "exam");
        assert_eq!(<ExamCreated as DomainEvent>::SCHEMA_VERSION, 1);
        assert_eq!(
            <ExamCreated as DomainEvent>::aggregate_id(&ev),
            exam_id.as_uuid()
        );
        assert_eq!(<ExamCreated as DomainEvent>::school_id(&ev), s);
    }

    #[test]
    fn exam_updated_event_type_and_aggregate_id() {
        let s = SchoolId(uuid::Uuid::now_v7());
        let exam_id = ExamId::new(s, uuid::Uuid::now_v7());
        let ev = ExamUpdated::new(
            exam_id,
            vec!["exam_mark".to_owned(), "pass_mark".to_owned()],
            EventId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
            Timestamp::now(),
        );
        assert_eq!(
            <ExamUpdated as DomainEvent>::EVENT_TYPE,
            "assessment.exam.updated"
        );
        assert_eq!(
            <ExamUpdated as DomainEvent>::aggregate_id(&ev),
            exam_id.as_uuid()
        );
        assert_eq!(
            ev.changes,
            vec!["exam_mark".to_owned(), "pass_mark".to_owned()]
        );
    }

    #[test]
    fn exam_deleted_event_type_and_aggregate_id() {
        let s = SchoolId(uuid::Uuid::now_v7());
        let exam_id = ExamId::new(s, uuid::Uuid::now_v7());
        let ev = ExamDeleted::new(
            exam_id,
            EventId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
            Timestamp::now(),
        );
        assert_eq!(
            <ExamDeleted as DomainEvent>::EVENT_TYPE,
            "assessment.exam.deleted"
        );
        assert_eq!(
            <ExamDeleted as DomainEvent>::aggregate_id(&ev),
            exam_id.as_uuid()
        );
    }

    #[test]
    fn exam_created_envelope_round_trip() {
        let s = SchoolId(uuid::Uuid::now_v7());
        let exam_id = ExamId::new(s, uuid::Uuid::now_v7());
        let ev = ExamCreated::new(
            exam_id,
            ExamTypeId::new(s, uuid::Uuid::now_v7()),
            ClassId::new(s, uuid::Uuid::now_v7()),
            SectionId::new(s, uuid::Uuid::now_v7()),
            SubjectId::new(s, uuid::Uuid::now_v7()),
            AcademicYearId::new(s, uuid::Uuid::now_v7()),
            ExamName::new("Mid-Term Mathematics").unwrap(),
            ExamCode::new("MTH-MT-2024").unwrap(),
            ExamMark::new(100.0).unwrap(),
            PassMark::new(35.0).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2024, 9, 15).unwrap(),
            EventId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
            Timestamp::now(),
        );
        let envelope = ev.clone().into_envelope(&ctx(s));
        assert_eq!(envelope.event_type, "assessment.exam.created");
        assert_eq!(envelope.aggregate_type, "exam");
        assert_eq!(envelope.school_id, s);
        assert_eq!(envelope.aggregate_id, exam_id.as_uuid());
    }
}

// =============================================================================
// Workstream C events: MarksRegister, ResultStore, ReportCard
// =============================================================================

/// Emitted when a [`MarksRegister`](crate::aggregate::MarksRegister)
/// is initialised.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarksRegisterCreated {
    pub marks_register_id: crate::value_objects::MarksRegisterId,
    pub exam_id: ExamId,
    pub student_id: crate::value_objects::StudentId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}
impl DomainEvent for MarksRegisterCreated {
    const EVENT_TYPE: &'static str = "assessment.marks_register.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "marks_register";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.marks_register_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.marks_register_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}
impl MarksRegisterCreated {
    #[must_use]
    pub fn new(
        marks_register_id: crate::value_objects::MarksRegisterId,
        exam_id: ExamId,
        student_id: crate::value_objects::StudentId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            marks_register_id,
            exam_id,
            student_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

/// Emitted when a single marks row is entered (per subject).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarksEntered {
    pub marks_register_id: crate::value_objects::MarksRegisterId,
    pub subject_id: SubjectId,
    pub student_id: crate::value_objects::StudentId,
    pub marks: Option<f32>,
    pub is_absent: bool,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}
impl DomainEvent for MarksEntered {
    const EVENT_TYPE: &'static str = "assessment.marks.entered";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "marks_register";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.marks_register_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.marks_register_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}
impl MarksEntered {
    #[must_use]
    pub fn new(
        marks_register_id: crate::value_objects::MarksRegisterId,
        subject_id: SubjectId,
        student_id: crate::value_objects::StudentId,
        marks: Option<f32>,
        is_absent: bool,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            marks_register_id,
            subject_id,
            student_id,
            marks,
            is_absent,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

/// Emitted when a marks register is submitted (locked for
/// grading).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarksSubmitted {
    pub marks_register_id: crate::value_objects::MarksRegisterId,
    pub exam_id: ExamId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub subject_count: u32,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}
impl DomainEvent for MarksSubmitted {
    const EVENT_TYPE: &'static str = "assessment.marks.submitted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "marks_register";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.marks_register_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.marks_register_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}
impl MarksSubmitted {
    #[must_use]
    pub fn new(
        marks_register_id: crate::value_objects::MarksRegisterId,
        exam_id: ExamId,
        class_id: ClassId,
        section_id: SectionId,
        subject_count: u32,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            marks_register_id,
            exam_id,
            class_id,
            section_id,
            subject_count,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

/// Emitted when a marks register is cancelled (a teacher
/// withdraws a submission before publish).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarksRegisterCancelled {
    pub marks_register_id: crate::value_objects::MarksRegisterId,
    pub reason: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}
impl DomainEvent for MarksRegisterCancelled {
    const EVENT_TYPE: &'static str = "assessment.marks_register.cancelled";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "marks_register";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.marks_register_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.marks_register_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}
impl MarksRegisterCancelled {
    #[must_use]
    pub fn new(
        marks_register_id: crate::value_objects::MarksRegisterId,
        reason: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            marks_register_id,
            reason,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

/// Emitted when a [`ResultStore`](crate::aggregate::ResultStore)
/// row is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResultStoreCreated {
    pub result_store_id: crate::value_objects::ResultStoreId,
    pub exam_id: ExamId,
    pub student_id: crate::value_objects::StudentId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}
impl DomainEvent for ResultStoreCreated {
    const EVENT_TYPE: &'static str = "assessment.result_store.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "result_store";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.result_store_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.result_store_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}
impl ResultStoreCreated {
    #[must_use]
    pub fn new(
        result_store_id: crate::value_objects::ResultStoreId,
        exam_id: ExamId,
        student_id: crate::value_objects::StudentId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            result_store_id,
            exam_id,
            student_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

/// Emitted when a result's teacher remarks are updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResultRemarksUpdated {
    pub result_store_id: crate::value_objects::ResultStoreId,
    pub teacher_remarks: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}
impl DomainEvent for ResultRemarksUpdated {
    const EVENT_TYPE: &'static str = "assessment.result_store.remarks_updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "result_store";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.result_store_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.result_store_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}
impl ResultRemarksUpdated {
    #[must_use]
    pub fn new(
        result_store_id: crate::value_objects::ResultStoreId,
        teacher_remarks: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            result_store_id,
            teacher_remarks,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

/// Emitted when an exam's result is published.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResultPublished {
    pub exam_id: ExamId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub academic_year_id: AcademicYearId,
    pub student_count: u32,
    pub published_at: Timestamp,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
}
impl DomainEvent for ResultPublished {
    const EVENT_TYPE: &'static str = "assessment.result.published";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "result_store";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.exam_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.exam_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.published_at
    }
}
impl ResultPublished {
    #[must_use]
    pub fn new(
        exam_id: ExamId,
        class_id: ClassId,
        section_id: SectionId,
        academic_year_id: AcademicYearId,
        student_count: u32,
        published_at: Timestamp,
        event_id: EventId,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            exam_id,
            class_id,
            section_id,
            academic_year_id,
            student_count,
            published_at,
            event_id,
            correlation_id,
        }
    }
}

/// Emitted when an exam's result is re-published.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResultRepublished {
    pub exam_id: ExamId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub reason: String,
    pub republished_at: Timestamp,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
}
impl DomainEvent for ResultRepublished {
    const EVENT_TYPE: &'static str = "assessment.result.republished";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "result_store";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.exam_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.exam_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.republished_at
    }
}
impl ResultRepublished {
    #[must_use]
    pub fn new(
        exam_id: ExamId,
        class_id: ClassId,
        section_id: SectionId,
        reason: String,
        republished_at: Timestamp,
        event_id: EventId,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            exam_id,
            class_id,
            section_id,
            reason,
            republished_at,
            event_id,
            correlation_id,
        }
    }
}

/// Emitted when a report card is materialised from a
/// published result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportCardGenerated {
    pub result_store_id: crate::value_objects::ResultStoreId,
    pub student_id: crate::value_objects::StudentId,
    pub exam_id: ExamId,
    pub include_remarks: bool,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}
impl DomainEvent for ReportCardGenerated {
    const EVENT_TYPE: &'static str = "assessment.report_card.generated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "report_card";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.result_store_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.result_store_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}
impl ReportCardGenerated {
    #[must_use]
    pub fn new(
        result_store_id: crate::value_objects::ResultStoreId,
        student_id: crate::value_objects::StudentId,
        exam_id: ExamId,
        include_remarks: bool,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            result_store_id,
            student_id,
            exam_id,
            include_remarks,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

// =============================================================================
// Cluster C: event stubs for the structural-skeleton aggregates.
//
// Each stub carries the minimal `event_id` + `school_id` + aggregate id
// (no payload, no `DomainEvent` impl). The full event body and
// `DomainEvent` impl land in subsequent workstreams.
// =============================================================================

// --- Exam + ExamSetup cluster ----------------------------------------------

/// Event stub emitted when an [`ExamSetup`](crate::aggregate::ExamSetup)
/// is created.
#[derive(Debug, Clone)]
pub struct ExamSetupCreated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub exam_setup_id: ExamSetupId,
}

// --- MarkStore cluster ------------------------------------------------------

/// Event stub emitted when a [`MarkStore`](crate::aggregate::MarkStore)
/// row is inserted.
#[derive(Debug, Clone)]
pub struct MarkStoreCreated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub mark_store_id: MarkStoreId,
}

// --- Result publication cluster --------------------------------------------

/// Event stub emitted when a [`ResultSetting`](crate::aggregate::ResultSetting)
/// is created or updated.
#[derive(Debug, Clone)]
pub struct ResultSettingCreated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub result_setting_id: ResultSettingId,
}

/// Event stub emitted when a [`MarksGrade`](crate::aggregate::MarksGrade)
/// row is created.
#[derive(Debug, Clone)]
pub struct MarksGradeCreated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub marks_grade_id: MarksGradeId,
}

// --- Exam publication cluster ----------------------------------------------

/// Event stub emitted when an [`ExamSetting`](crate::aggregate::ExamSetting)
/// is created or updated.
#[derive(Debug, Clone)]
pub struct ExamSettingCreated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub exam_setting_id: ExamSettingId,
}

/// Event stub emitted when an [`ExamSignature`](crate::aggregate::ExamSignature)
/// is created or replaced.
#[derive(Debug, Clone)]
pub struct ExamSignatureCreated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub exam_signature_id: ExamSignatureId,
}

/// Event stub emitted when an [`ExamRoutinePage`](crate::aggregate::ExamRoutinePage)
/// is created or updated.
#[derive(Debug, Clone)]
pub struct ExamRoutinePageCreated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub exam_routine_page_id: ExamRoutinePageId,
}

/// Event stub emitted when a [`FrontendExamRoutine`](crate::aggregate::FrontendExamRoutine)
/// is published.
#[derive(Debug, Clone)]
pub struct FrontendExamRoutineCreated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub front_exam_routine_id: FrontExamRoutineId,
}

/// Event stub emitted when a [`FrontendResult`](crate::aggregate::FrontendResult)
/// is published.
#[derive(Debug, Clone)]
pub struct FrontendResultCreated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub front_result_id: FrontResultId,
}

/// Event stub emitted when a [`FrontendExamResult`](crate::aggregate::FrontendExamResult)
/// block is updated.
#[derive(Debug, Clone)]
pub struct FrontendExamResultCreated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub frontend_exam_result_id: FrontendExamResultId,
}

// --- OnlineExam cluster -----------------------------------------------------

/// Event stub emitted when an [`OnlineExam`](crate::aggregate::OnlineExam)
/// is created.
#[derive(Debug, Clone)]
pub struct OnlineExamCreated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub online_exam_id: OnlineExamId,
}

/// Event stub emitted when a [`QuestionBank`](crate::aggregate::QuestionBank)
/// entry is created.
#[derive(Debug, Clone)]
pub struct QuestionBankCreated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub question_bank_id: QuestionBankId,
}

/// Event stub emitted when a [`QuestionGroup`](crate::aggregate::QuestionGroup)
/// is created.
#[derive(Debug, Clone)]
pub struct QuestionGroupCreated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub question_group_id: QuestionGroupId,
}

/// Event stub emitted when a [`QuestionLevel`](crate::aggregate::QuestionLevel)
/// is created.
#[derive(Debug, Clone)]
pub struct QuestionLevelCreated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub question_level_id: QuestionLevelId,
}

/// Event stub emitted when a [`StudentTakeOnlineExam`](crate::aggregate::StudentTakeOnlineExam)
/// attempt is created.
#[derive(Debug, Clone)]
pub struct StudentTakeOnlineExamCreated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub student_take_online_exam_id: StudentTakeOnlineExamId,
}

// --- AdmitCard cluster ------------------------------------------------------

/// Event stub emitted when an [`AdmitCardSetting`](crate::aggregate::AdmitCardSetting)
/// is created or updated.
#[derive(Debug, Clone)]
pub struct AdmitCardSettingCreated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub admit_card_setting_id: AdmitCardSettingId,
}

// --- Teacher review cluster -------------------------------------------------

/// Event stub emitted when a [`TeacherEvaluation`](crate::aggregate::TeacherEvaluation)
/// is recorded.
#[derive(Debug, Clone)]
pub struct TeacherEvaluationCreated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub teacher_evaluation_id: TeacherEvaluationId,
}

/// Event stub emitted when a [`TeacherRemark`](crate::aggregate::TeacherRemark)
/// is recorded.
#[derive(Debug, Clone)]
pub struct TeacherRemarkCreated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub teacher_remark_id: TeacherRemarkId,
}

// --- Merit + position cluster -----------------------------------------------

/// Event stub emitted when a [`TemporaryMeritList`](crate::aggregate::TemporaryMeritList)
/// row is staged.
#[derive(Debug, Clone)]
pub struct TemporaryMeritListCreated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub temporary_merit_list_id: TemporaryMeritListId,
}

// --- Custom result cluster --------------------------------------------------

/// Event stub emitted when a [`CustomResultSetting`](crate::aggregate::CustomResultSetting)
/// is created or updated.
#[derive(Debug, Clone)]
pub struct CustomResultSettingCreated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub custom_result_setting_id: CustomResultSettingId,
}

/// Event stub emitted when a [`CustomTemporaryResult`](crate::aggregate::CustomTemporaryResult)
/// row is staged.
#[derive(Debug, Clone)]
pub struct CustomTemporaryResultCreated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub custom_temporary_result_id: CustomTemporaryResultId,
}

// --- Wizard-skip cluster ----------------------------------------------------

/// Event stub emitted when an [`ExamStepSkip`](crate::aggregate::ExamStepSkip)
/// flag is created or updated.
#[derive(Debug, Clone)]
pub struct ExamStepSkipCreated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub exam_step_skip_id: ExamStepSkipId,
}

// --- Exam attendance cluster ------------------------------------------------

/// Event stub emitted when an [`ExamAttendance`](crate::aggregate::ExamAttendance)
/// roll is opened.
#[derive(Debug, Clone)]
pub struct ExamAttendanceCreated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub exam_attendance_id: ExamAttendanceId,
}

// =============================================================================
// Cluster D: closing the spec_to_code:missing_event lint gap.
//
// Each stub carries the minimal `event_id` + `school_id` + aggregate id
// (no payload, no `DomainEvent` impl). The full event body and
// `DomainEvent` impl land in subsequent workstreams.
//
// Note: `SeatPlanSettingId` is not yet declared in `value_objects.rs`; this
// stub uses a raw `uuid::Uuid` for the aggregate id until the typed id
// lands. Same applies to `TeacherEvaluationConfigured`, which has no
// aggregate in the spec.
// =============================================================================

// --- ExamType cluster -------------------------------------------------------

/// Event stub emitted when an `ExamType` is created.
#[derive(Debug, Clone)]
pub struct ExamTypeCreated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub exam_type_id: ExamTypeId,
}

/// Event stub emitted when an `ExamType` is updated.
#[derive(Debug, Clone)]
pub struct ExamTypeUpdated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub exam_type_id: ExamTypeId,
}

/// Event stub emitted when an `ExamType` is deleted.
#[derive(Debug, Clone)]
pub struct ExamTypeDeleted {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub exam_type_id: ExamTypeId,
}

// --- MarkStore cluster (additional) -----------------------------------------

/// Event stub emitted when a [`MarkStore`](crate::aggregate::MarkStore)
/// row is deleted.
#[derive(Debug, Clone)]
pub struct MarkStoreDeleted {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub mark_store_id: MarkStoreId,
}

// --- Result publication cluster (additional) -------------------------------

/// Event stub emitted when a [`ResultSetting`](crate::aggregate::ResultSetting)
/// is updated.
#[derive(Debug, Clone)]
pub struct ResultSettingUpdated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub result_setting_id: ResultSettingId,
}

// --- Marks Grade cluster (additional) --------------------------------------

/// Event stub emitted when a [`MarksGrade`](crate::aggregate::MarksGrade)
/// row is updated.
#[derive(Debug, Clone)]
pub struct MarksGradeUpdated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub marks_grade_id: MarksGradeId,
}

/// Event stub emitted when a [`MarksGrade`](crate::aggregate::MarksGrade)
/// row is deleted.
#[derive(Debug, Clone)]
pub struct MarksGradeDeleted {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub marks_grade_id: MarksGradeId,
}

// --- Exam publication cluster (additional) ---------------------------------

/// Event stub emitted when an [`ExamSetting`](crate::aggregate::ExamSetting)
/// is updated.
#[derive(Debug, Clone)]
pub struct ExamSettingUpdated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub exam_setting_id: ExamSettingId,
}

/// Event stub emitted when an [`ExamSetting`](crate::aggregate::ExamSetting)
/// is deleted.
#[derive(Debug, Clone)]
pub struct ExamSettingDeleted {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub exam_setting_id: ExamSettingId,
}

/// Event stub emitted when an [`ExamSignature`](crate::aggregate::ExamSignature)
/// is updated.
#[derive(Debug, Clone)]
pub struct ExamSignatureUpdated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub exam_signature_id: ExamSignatureId,
}

/// Event stub emitted when an [`ExamSignature`](crate::aggregate::ExamSignature)
/// is deleted.
#[derive(Debug, Clone)]
pub struct ExamSignatureDeleted {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub exam_signature_id: ExamSignatureId,
}

/// Event stub emitted when an [`ExamRoutinePage`](crate::aggregate::ExamRoutinePage)
/// is updated.
#[derive(Debug, Clone)]
pub struct ExamRoutinePageUpdated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub exam_routine_page_id: ExamRoutinePageId,
}

/// Event stub emitted when a [`FrontendExamRoutine`](crate::aggregate::FrontendExamRoutine)
/// is published.
#[derive(Debug, Clone)]
pub struct FrontendExamRoutinePublished {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub front_exam_routine_id: FrontExamRoutineId,
}

/// Event stub emitted when a [`FrontendResult`](crate::aggregate::FrontendResult)
/// is published.
#[derive(Debug, Clone)]
pub struct FrontendResultPublished {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub front_result_id: FrontResultId,
}

/// Event stub emitted when a [`FrontendExamResult`](crate::aggregate::FrontendExamResult)
/// block is updated.
#[derive(Debug, Clone)]
pub struct FrontendExamResultUpdated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub frontend_exam_result_id: FrontendExamResultId,
}

// --- Custom result cluster (additional) -------------------------------------

/// Event stub emitted when a [`CustomResultSetting`](crate::aggregate::CustomResultSetting)
/// is updated.
#[derive(Debug, Clone)]
pub struct CustomResultSettingUpdated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub custom_result_setting_id: CustomResultSettingId,
}

// --- Exam Setup cluster (additional) ----------------------------------------

/// Event stub emitted when an [`ExamSetup`](crate::aggregate::ExamSetup)
/// is updated.
#[derive(Debug, Clone)]
pub struct ExamSetupUpdated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub exam_setup_id: ExamSetupId,
}

/// Event stub emitted when an [`ExamSetup`](crate::aggregate::ExamSetup)
/// is deleted.
#[derive(Debug, Clone)]
pub struct ExamSetupDeleted {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub exam_setup_id: ExamSetupId,
}

// --- OnlineExam cluster (additional) ----------------------------------------

/// Event stub emitted when an [`OnlineExam`](crate::aggregate::OnlineExam)
/// is updated.
#[derive(Debug, Clone)]
pub struct OnlineExamUpdated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub online_exam_id: OnlineExamId,
}

/// Event stub emitted when an [`OnlineExam`](crate::aggregate::OnlineExam)
/// is published.
#[derive(Debug, Clone)]
pub struct OnlineExamPublished {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub online_exam_id: OnlineExamId,
}

/// Event stub emitted when a student starts an online exam.
#[derive(Debug, Clone)]
pub struct OnlineExamStarted {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub online_exam_id: OnlineExamId,
    pub student_id: crate::value_objects::StudentId,
}

/// Event stub emitted when a student answers an online exam question.
#[derive(Debug, Clone)]
pub struct OnlineExamAnswered {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub online_exam_id: OnlineExamId,
    pub student_id: crate::value_objects::StudentId,
    pub question_id: OnlineExamQuestionId,
}

/// Event stub emitted when a student's online exam attempt is evaluated.
#[derive(Debug, Clone)]
pub struct OnlineExamEvaluated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub online_exam_id: OnlineExamId,
    pub student_id: crate::value_objects::StudentId,
}

/// Event stub emitted when an [`OnlineExam`](crate::aggregate::OnlineExam)
/// is closed (exam window has ended).
#[derive(Debug, Clone)]
pub struct OnlineExamClosed {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub online_exam_id: OnlineExamId,
}

/// Event stub emitted when an [`OnlineExam`](crate::aggregate::OnlineExam)
/// is deleted.
#[derive(Debug, Clone)]
pub struct OnlineExamDeleted {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub online_exam_id: OnlineExamId,
}

// --- Question Bank cluster (additional) -------------------------------------

/// Event stub emitted when a [`QuestionBank`](crate::aggregate::QuestionBank)
/// entry is created.
#[derive(Debug, Clone)]
pub struct QuestionCreated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub question_id: QuestionBankId,
}

/// Event stub emitted when a [`QuestionBank`](crate::aggregate::QuestionBank)
/// entry is updated.
#[derive(Debug, Clone)]
pub struct QuestionUpdated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub question_id: QuestionBankId,
}

/// Event stub emitted when a [`QuestionBank`](crate::aggregate::QuestionBank)
/// entry is deleted.
#[derive(Debug, Clone)]
pub struct QuestionDeleted {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub question_id: QuestionBankId,
}

/// Event stub emitted when a [`QuestionGroup`](crate::aggregate::QuestionGroup)
/// is updated.
#[derive(Debug, Clone)]
pub struct QuestionGroupUpdated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub question_group_id: QuestionGroupId,
}

/// Event stub emitted when a [`QuestionGroup`](crate::aggregate::QuestionGroup)
/// is deleted.
#[derive(Debug, Clone)]
pub struct QuestionGroupDeleted {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub question_group_id: QuestionGroupId,
}

/// Event stub emitted when a [`QuestionLevel`](crate::aggregate::QuestionLevel)
/// is updated.
#[derive(Debug, Clone)]
pub struct QuestionLevelUpdated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub question_level_id: QuestionLevelId,
}

/// Event stub emitted when a [`QuestionLevel`](crate::aggregate::QuestionLevel)
/// is deleted.
#[derive(Debug, Clone)]
pub struct QuestionLevelDeleted {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub question_level_id: QuestionLevelId,
}

// --- Online Exam Question + Option cluster ---------------------------------

/// Event stub emitted when an `OnlineExamQuestion` is added to an online exam.
#[derive(Debug, Clone)]
pub struct OnlineExamQuestionAdded {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub question_id: OnlineExamQuestionId,
    pub online_exam_id: OnlineExamId,
}

/// Event stub emitted when an `OnlineExamQuestion` is updated.
#[derive(Debug, Clone)]
pub struct OnlineExamQuestionUpdated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub question_id: OnlineExamQuestionId,
}

/// Event stub emitted when an `OnlineExamQuestion` is deleted.
#[derive(Debug, Clone)]
pub struct OnlineExamQuestionDeleted {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub question_id: OnlineExamQuestionId,
}

/// Event stub emitted when a `QuestionMuOption` is added to a question.
#[derive(Debug, Clone)]
pub struct QuestionOptionAdded {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub option_id: QuestionMuOptionId,
    pub question_id: OnlineExamQuestionId,
}

/// Event stub emitted when a `QuestionMuOption` is updated.
#[derive(Debug, Clone)]
pub struct QuestionOptionUpdated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub option_id: QuestionMuOptionId,
}

/// Event stub emitted when a `QuestionMuOption` is deleted.
#[derive(Debug, Clone)]
pub struct QuestionOptionDeleted {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub option_id: QuestionMuOptionId,
}

/// Event stub emitted when a per-student `OnlineExamMark` is created.
#[derive(Debug, Clone)]
pub struct OnlineExamMarkCreated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub mark_id: OnlineExamMarkId,
}

// --- Seat Plan setting cluster ---------------------------------------------

/// Event stub emitted when a `SeatPlanSetting` is updated. Uses a raw
/// `uuid::Uuid` for the aggregate id because `SeatPlanSettingId` is not yet
/// declared in `value_objects.rs`.
#[derive(Debug, Clone)]
pub struct SeatPlanSettingUpdated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub setting_id: uuid::Uuid,
}

// --- AdmitCard setting cluster (additional) --------------------------------

/// Event stub emitted when an [`AdmitCardSetting`](crate::aggregate::AdmitCardSetting)
/// is updated.
#[derive(Debug, Clone)]
pub struct AdmitCardSettingUpdated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub setting_id: AdmitCardSettingId,
}

// --- Teacher review cluster (additional) ------------------------------------

/// Event stub emitted when a [`TeacherEvaluation`](crate::aggregate::TeacherEvaluation)
/// is completed by a student.
#[derive(Debug, Clone)]
pub struct TeacherEvaluationCompleted {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub evaluation_id: TeacherEvaluationId,
}

/// Event stub emitted when a [`TeacherEvaluation`](crate::aggregate::TeacherEvaluation)
/// is approved.
#[derive(Debug, Clone)]
pub struct TeacherEvaluationApproved {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub evaluation_id: TeacherEvaluationId,
}

/// Event stub emitted when a [`TeacherEvaluation`](crate::aggregate::TeacherEvaluation)
/// is rejected.
#[derive(Debug, Clone)]
pub struct TeacherEvaluationRejected {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub evaluation_id: TeacherEvaluationId,
}

/// Event stub emitted when the teacher-evaluation subsystem is configured.
/// This is a configuration event with no aggregate id; only the
/// `event_id` and `school_id` are stamped.
#[derive(Debug, Clone)]
pub struct TeacherEvaluationConfigured {
    pub event_id: EventId,
    pub school_id: SchoolId,
}

/// Event stub emitted when a [`TeacherRemark`](crate::aggregate::TeacherRemark)
/// is added.
#[derive(Debug, Clone)]
pub struct TeacherRemarkAdded {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub remark_id: TeacherRemarkId,
}

/// Event stub emitted when a [`TeacherRemark`](crate::aggregate::TeacherRemark)
/// is updated.
#[derive(Debug, Clone)]
pub struct TeacherRemarkUpdated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub remark_id: TeacherRemarkId,
}

/// Event stub emitted when a [`TeacherRemark`](crate::aggregate::TeacherRemark)
/// is deleted.
#[derive(Debug, Clone)]
pub struct TeacherRemarkDeleted {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub remark_id: TeacherRemarkId,
}

// --- Exam attendance cluster (additional) -----------------------------------

/// Event stub emitted when an [`ExamAttendance`](crate::aggregate::ExamAttendance)
/// roll is marked.
#[derive(Debug, Clone)]
pub struct ExamAttendanceMarked {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub exam_attendance_id: ExamAttendanceId,
}

/// Event stub emitted when an [`ExamAttendance`](crate::aggregate::ExamAttendance)
/// roll is updated.
#[derive(Debug, Clone)]
pub struct ExamAttendanceUpdated {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub exam_attendance_id: ExamAttendanceId,
}

// --- Wizard-skip cluster (additional) ---------------------------------------

/// Event stub emitted when an [`ExamStepSkip`](crate::aggregate::ExamStepSkip)
/// flag is set.
#[derive(Debug, Clone)]
pub struct ExamStepSkipSet {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub skip_id: ExamStepSkipId,
}
