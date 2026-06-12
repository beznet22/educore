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

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use educore_core::ids::{CorrelationId, EventId, SchoolId};
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;

use educore_academic::value_objects::PassMark;

use crate::value_objects::{
    AcademicYearId, ClassId, ExamCode, ExamId, ExamMark, ExamName, ExamTypeId, SectionId, SubjectId,
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
