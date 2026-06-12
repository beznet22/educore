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

#![allow(missing_docs)] // The 10 audit-metadata fields
                        // (version, etag, created_at, ...) on each
                        // aggregate are described by their type
                        // names; suppressing this lint for the
                        // file is the pragmatic choice for the
                        // 8 aggregates Phase 4 ships.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};

use educore_academic::value_objects::PassMark;

use crate::value_objects::{
    AcademicYearId, AdmitCardId, ClassId, ExamCode, ExamId, ExamMark, ExamName, ExamScheduleId,
    ExamTypeId, SeatPlanId, SectionId, StaffId, SubjectId,
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

// =============================================================================
// ExamSchedule
// =============================================================================

/// The calendar slot for an exam. Lives at the `(exam, class,
/// section)` level. Carries the per-subject entries via
/// [`ExamScheduleSubject`](crate::entities::ExamScheduleSubject)
/// children.
#[allow(clippy::too_many_arguments)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExamSchedule {
    /// The schedule's typed id.
    pub id: ExamScheduleId,
    /// The school (tenant anchor).
    pub school_id: SchoolId,
    /// The exam this schedule is for.
    pub exam_id: ExamId,
    /// The class this schedule covers.
    pub class_id: ClassId,
    /// The section this schedule covers.
    pub section_id: SectionId,
    /// The exam date (the schedule's anchor date; per-subject
    /// slots may have their own dates in multi-day exams).
    pub date: chrono::NaiveDate,
    /// The schedule's start time (the default for all subjects
    /// in this slot).
    pub start_time: chrono::NaiveTime,
    /// The schedule's end time.
    pub end_time: chrono::NaiveTime,
    /// The room the exam is held in (default for all subjects
    /// in this slot; per-subject overrides in
    /// `ExamScheduleSubject`).
    pub room_id: Option<crate::value_objects::ClassRoomId>,
    /// The teacher assigned to invigilate the exam (default for
    /// all subjects; per-subject overrides in
    /// `ExamScheduleSubject`).
    pub teacher_id: Option<StaffId>,
    /// Standard 10-field audit-metadata footer (per the
    /// 17-field pattern).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl ExamSchedule {
    /// The 32-char zero etag for a freshly minted schedule.
    pub const FRESH_ETAG: &'static str = "00000000000000000000000000000000";

    /// Constructs a new [`ExamSchedule`] aggregate.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn fresh(
        id: ExamScheduleId,
        exam_id: ExamId,
        class_id: ClassId,
        section_id: SectionId,
        date: chrono::NaiveDate,
        start_time: chrono::NaiveTime,
        end_time: chrono::NaiveTime,
        room_id: Option<crate::value_objects::ClassRoomId>,
        teacher_id: Option<StaffId>,
        actor: UserId,
        now: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            exam_id,
            class_id,
            section_id,
            date,
            start_time,
            end_time,
            room_id,
            teacher_id,
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

    /// Returns `true` if the schedule's time window is
    /// well-formed (`end_time > start_time`).
    #[must_use]
    pub fn is_well_formed(&self) -> bool {
        self.end_time > self.start_time
    }
}

// =============================================================================
// SeatPlan
// =============================================================================

/// The seat allocation for one section for one exam type in
/// an academic year. Has children
/// [`SeatPlanChild`](crate::entities::SeatPlanChild) for
/// per-room allocations.
#[allow(clippy::too_many_arguments)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeatPlan {
    /// The seat plan's typed id.
    pub id: SeatPlanId,
    /// The school (tenant anchor).
    pub school_id: SchoolId,
    /// The exam this seat plan is for.
    pub exam_id: ExamId,
    /// The class this seat plan covers.
    pub class_id: ClassId,
    /// The section this seat plan covers.
    pub section_id: SectionId,
    /// The total number of students in the section.
    pub total_students: u32,
    /// Standard 10-field audit-metadata footer.
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl SeatPlan {
    /// The 32-char zero etag for a freshly minted seat plan.
    pub const FRESH_ETAG: &'static str = "00000000000000000000000000000000";

    /// Constructs a new [`SeatPlan`] aggregate.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn fresh(
        id: SeatPlanId,
        exam_id: ExamId,
        class_id: ClassId,
        section_id: SectionId,
        total_students: u32,
        actor: UserId,
        now: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            exam_id,
            class_id,
            section_id,
            total_students,
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
}

// =============================================================================
// AdmitCard
// =============================================================================

/// The admit card issued to a student for an exam type in
/// an academic year. A card is generated per
/// `(student_record_id, exam_type_id)` and references the
/// school's `AdmitCardSetting` at generation time.
#[allow(clippy::too_many_arguments)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdmitCard {
    /// The admit card's typed id.
    pub id: AdmitCardId,
    /// The school (tenant anchor).
    pub school_id: SchoolId,
    /// The student record this card is issued to (the
    /// per-academic-year enrollment row).
    pub student_record_id: crate::value_objects::StudentRecordId,
    /// The exam type (mid-term, final, …).
    pub exam_type_id: ExamTypeId,
    /// The academic year the card is issued for.
    pub academic_year_id: AcademicYearId,
    /// The wall-clock time the card was generated.
    pub generated_at: Timestamp,
    /// Standard 10-field audit-metadata footer.
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl AdmitCard {
    /// The 32-char zero etag for a freshly minted admit card.
    pub const FRESH_ETAG: &'static str = "00000000000000000000000000000000";

    /// Constructs a new [`AdmitCard`] aggregate.
    #[must_use]
    pub fn fresh(
        id: AdmitCardId,
        student_record_id: crate::value_objects::StudentRecordId,
        exam_type_id: ExamTypeId,
        academic_year_id: AcademicYearId,
        actor: UserId,
        now: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            student_record_id,
            exam_type_id,
            academic_year_id,
            generated_at: now,
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
}

// =============================================================================
// MarksRegister (Workstream C)
// =============================================================================

/// A per-student container for the marks obtained in an
/// exam. There is one `MarksRegister` per
/// `(exam_id, student_id)`. Its children
/// ([`MarksRegisterChild`](crate::entities::MarksRegisterChild))
/// hold the per-subject marks.
#[allow(clippy::too_many_arguments)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarksRegister {
    pub id: crate::value_objects::MarksRegisterId,
    pub school_id: SchoolId,
    pub exam_id: ExamId,
    pub student_id: crate::value_objects::StudentId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub academic_year_id: AcademicYearId,
    /// `true` while the register is being entered;
    /// `false` once `submit_marks` locks it.
    pub is_open: bool,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}
impl MarksRegister {
    pub const FRESH_ETAG: &'static str = "00000000000000000000000000000000";
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn fresh(
        id: crate::value_objects::MarksRegisterId,
        exam_id: ExamId,
        student_id: crate::value_objects::StudentId,
        class_id: ClassId,
        section_id: SectionId,
        academic_year_id: AcademicYearId,
        actor: UserId,
        now: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            exam_id,
            student_id,
            class_id,
            section_id,
            academic_year_id,
            is_open: true,
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
}

// =============================================================================
// ResultStore (Workstream C)
// =============================================================================

/// The published, per-student per-subject result row.
/// Drives report cards and merit position calculations.
#[allow(clippy::too_many_arguments)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResultStore {
    pub id: crate::value_objects::ResultStoreId,
    pub school_id: SchoolId,
    pub exam_id: ExamId,
    pub student_id: crate::value_objects::StudentId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub academic_year_id: AcademicYearId,
    pub total_marks: f32,
    pub total_gpa: f32,
    pub total_grade: crate::value_objects::Grade,
    pub status: crate::value_objects::ResultStatus,
    pub published_at: Timestamp,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}
impl ResultStore {
    pub const FRESH_ETAG: &'static str = "00000000000000000000000000000000";
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn fresh(
        id: crate::value_objects::ResultStoreId,
        exam_id: ExamId,
        student_id: crate::value_objects::StudentId,
        class_id: ClassId,
        section_id: SectionId,
        academic_year_id: AcademicYearId,
        total_marks: f32,
        total_gpa: f32,
        total_grade: crate::value_objects::Grade,
        status: crate::value_objects::ResultStatus,
        actor: UserId,
        now: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            exam_id,
            student_id,
            class_id,
            section_id,
            academic_year_id,
            total_marks,
            total_gpa,
            total_grade,
            status,
            published_at: now,
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
}
