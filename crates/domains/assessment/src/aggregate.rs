//! # Assessment aggregate roots
//!
//! Phase 4 Workstream A ships the canonical [`Exam`]
//! aggregate. The `ExamSchedule`, `MarksRegister`,
//! `ResultStore`, `ReportCard` (projection), `OnlineExam`,
//! `SeatPlan`, and `AdmitCard` aggregates land in Workstreams
//! B, C, and D respectively.
//!
//! The `Exam` follows the "aggregate as a single struct"
//! pattern (mirroring `educore-academic`'s [`Student`](educore_academic::Student)
//! and `educore-platform`'s [`School`](educore_platform::School)):
//! the struct holds the full state, with `version` for
//! optimistic concurrency, `etag` for content hashing,
//! `active_status` for soft delete, and `last_event_id` /
//! `correlation_id` for the audit / outbox bridge.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};

use educore_academic::value_objects::PassMark;

use crate::value_objects::{
    AcademicYearId, ClassId, ExamCode, ExamId, ExamMark, ExamName, ExamTypeId, SectionId, SubjectId,
};

/// Returns the default etag for a freshly minted aggregate.
///
/// Delegates to [`Etag::placeholder`] (an infallible
/// constructor) so callers do not need to handle a `Result`.
fn fresh_etag() -> Etag {
    Etag::placeholder()
}

// =============================================================================
// Exam
// =============================================================================

/// A specific exam instance: one `(class, section, subject)`
/// under one `ExamType` for an academic year.
///
/// Carries the exam's name, code, full mark, pass mark,
/// exam date, publish state, and audit metadata. Children
/// (`ExamSetup`, `ExamSchedule`, `MarksRegister`, `SeatPlan`)
/// are tracked separately; per the Phase 4 prompt, only the
/// `Exam` root is in Workstream A scope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Exam {
    /// The exam's typed id.
    pub id: ExamId,
    /// The school the exam belongs to (tenant anchor; also
    /// embedded in the typed id).
    pub school_id: SchoolId,
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
    pub name: ExamName,
    /// The exam code (unique within `(school, academic_year)`).
    pub code: ExamCode,
    /// The exam's full mark (max obtainable score).
    pub exam_mark: ExamMark,
    /// The exam's pass mark.
    pub pass_mark: PassMark,
    /// The exam's date.
    pub exam_date: NaiveDate,
    /// Whether the exam is published (visible to teachers /
    /// students / parents). Drafts (unpublished) are visible
    /// only to the exam cell.
    pub is_published: bool,
    /// Optimistic-concurrency counter.
    pub version: Version,
    /// Content-hash (etag) for last-write-wins detection.
    pub etag: Etag,
    /// Creation timestamp.
    pub created_at: Timestamp,
    /// Last-update timestamp.
    pub updated_at: Timestamp,
    /// The user who created the exam.
    pub created_by: UserId,
    /// The user who last updated the exam.
    pub updated_by: UserId,
    /// Soft-delete flag.
    pub active_status: ActiveStatus,
    /// The last event id emitted by this aggregate.
    pub last_event_id: Option<EventId>,
    /// The request correlation id that originated the most
    /// recent state change.
    pub correlation_id: CorrelationId,
}

impl Exam {
    /// The 32-char zero etag for a freshly minted aggregate.
    pub const FRESH_ETAG: &'static str = "00000000000000000000000000000000";

    /// Constructs a new [`Exam`] aggregate.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn fresh(
        id: ExamId,
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
        actor: UserId,
        now: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            exam_type_id,
            class_id,
            section_id,
            subject_id,
            academic_year_id,
            name,
            code,
            exam_mark,
            pass_mark,
            exam_date,
            is_published: false,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: now,
            updated_at: now,
            created_by: actor,
            updated_by: actor,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Returns `true` if the exam is currently published.
    #[must_use]
    pub const fn is_published(&self) -> bool {
        self.is_published
    }

    /// Returns `true` if the exam is currently active (not
    /// soft-deleted).
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active_status.is_active()
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

    fn school() -> SchoolId {
        SchoolId(uuid::Uuid::now_v7())
    }

    fn now() -> Timestamp {
        Timestamp::now()
    }

    fn actor() -> UserId {
        UserId(uuid::Uuid::now_v7())
    }

    #[test]
    fn exam_fresh_sets_default_state() {
        let s = school();
        let corr = educore_core::ids::CorrelationId(uuid::Uuid::now_v7());
        let exam = Exam::fresh(
            ExamId::new(s, uuid::Uuid::now_v7()),
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
            actor(),
            now(),
            corr,
        );
        assert!(!exam.is_published());
        assert!(exam.is_active());
        assert_eq!(exam.version.get(), 1);
        assert_eq!(exam.school_id, s);
    }

    #[test]
    fn exam_fresh_etag_matches_constant() {
        let s = school();
        let corr = educore_core::ids::CorrelationId(uuid::Uuid::now_v7());
        let exam = Exam::fresh(
            ExamId::new(s, uuid::Uuid::now_v7()),
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
            actor(),
            now(),
            corr,
        );
        assert_eq!(exam.etag.as_str(), Exam::FRESH_ETAG);
    }
}
