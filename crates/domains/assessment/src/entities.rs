//! # Assessment-domain child entities
//!
//! Per-aggregate children that are not part of the aggregate
//! root's struct but are owned by it (e.g. a per-subject
//! `MarksRegisterChild` row belongs to a `MarksRegister`).
//!
//! Phase 4 Workstream B ships:
//!
//! - [`ExamScheduleSubject`] — the per-subject entry in an
//!   `ExamSchedule`. Owns `Date`, `StartTime`, `EndTime`,
//!   `Room` (a `ClassRoomId`), `FullMark`, `PassMark`.
//! - [`SeatPlanChild`] — the per-room allocation in a
//!   `SeatPlan`. Owns `RoomId`, `AssignStudents`,
//!   `StartTime`, `EndTime`.
//!
//! Phase 4 Workstream C adds `MarksRegisterChild`; Phase 4
//! Workstream D adds the `OnlineExam*` children.

use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};

use educore_academic::value_objects::PassMark;

use crate::value_objects::{
    ClassRoomId, ExamScheduleId, ExamScheduleSubjectId, FullMark, Gpa, Grade, Marks,
    MarksRegisterChildId, MarksRegisterId, SeatPlanChildId, SeatPlanId, StaffId, SubjectId,
};

// =============================================================================
// ExamScheduleSubject
// =============================================================================

/// A per-subject entry in an [`ExamSchedule`](crate::aggregate::ExamSchedule).
/// Carries `Date`, `StartTime`, `EndTime`, `Room`,
/// `FullMark`, `PassMark`, and the `SubjectId` /
/// `ExamScheduleId` foreign keys.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExamScheduleSubject {
    /// The child's typed id.
    pub id: ExamScheduleSubjectId,
    /// The parent schedule.
    pub exam_schedule_id: ExamScheduleId,
    /// The subject this slot is for.
    pub subject_id: SubjectId,
    /// The exam date (the schedule's `date` + this subject's
    /// per-subject date, which may differ from the schedule's
    /// date in multi-day exams).
    pub date: NaiveDate,
    /// The slot's start time.
    pub start_time: NaiveTime,
    /// The slot's end time.
    pub end_time: NaiveTime,
    /// The room the exam is held in (`ClassRoomId` from the
    /// academic crate).
    pub room_id: Option<ClassRoomId>,
    /// The teacher assigned to invigilate the exam.
    pub teacher_id: Option<StaffId>,
    /// The subject's full mark (may differ across subjects in
    /// a single exam schedule).
    pub full_mark: FullMark,
    /// The subject's pass mark.
    pub pass_mark: PassMark,
}

impl ExamScheduleSubject {
    /// Returns `true` if the slot's `end_time` is after its
    /// `start_time` (the start-before-end invariant).
    #[must_use]
    pub fn is_well_formed(&self) -> bool {
        self.end_time > self.start_time
    }
}

// =============================================================================
// SeatPlanChild
// =============================================================================

/// A per-room allocation in a [`SeatPlan`](crate::aggregate::SeatPlan).
/// Carries `RoomId`, `AssignStudents`, `StartTime`, `EndTime`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeatPlanChild {
    /// The child's typed id.
    pub id: SeatPlanChildId,
    /// The parent seat plan.
    pub seat_plan_id: SeatPlanId,
    /// The room this allocation covers.
    pub room_id: ClassRoomId,
    /// The number of students assigned to this room.
    pub assign_students: u32,
    /// The slot's start time.
    pub start_time: NaiveTime,
    /// The slot's end time.
    pub end_time: NaiveTime,
}

impl SeatPlanChild {
    /// Returns `true` if the allocation has at least one
    /// student and the time window is well-formed.
    #[must_use]
    pub fn is_well_formed(&self) -> bool {
        self.assign_students > 0 && self.end_time > self.start_time
    }
}

// =============================================================================
// MarksRegisterChild
// =============================================================================

/// A per-subject marks row belonging to a
/// [`MarksRegister`](crate::aggregate::MarksRegister).
/// Carries `Marks`, `Abs` (absent 0/1), `GpaPoint`,
/// `GpaGrade`, `ActiveStatus`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarksRegisterChild {
    /// The child's typed id.
    pub id: MarksRegisterChildId,
    /// The parent marks register.
    pub marks_register_id: MarksRegisterId,
    /// The subject this row is for.
    pub subject_id: SubjectId,
    /// The subject's full mark.
    pub full_mark: FullMark,
    /// The subject's pass mark.
    pub pass_mark: PassMark,
    /// The marks obtained (`None` if not yet entered).
    pub marks: Option<Marks>,
    /// Absent flag (0 = present, 1 = absent). Per the spec,
    /// if `Abs=1` then `Marks` is treated as zero and the
    /// grade follows the school's absent rule.
    pub is_absent: bool,
    /// Computed grade point (per the school's `MarksGrade`
    /// scale).
    pub gpa_point: Option<Gpa>,
    /// Computed grade (e.g. "A+", "B", "F").
    pub gpa_grade: Option<Grade>,
    /// Soft-delete flag.
    pub active_status: educore_core::value_objects::ActiveStatus,
    /// Optional free-text teacher comment.
    pub comment: Option<String>,
}

impl MarksRegisterChild {
    /// Returns `true` if the row is active (not soft-deleted)
    /// AND has marks entered (i.e. is past the `enter_marks`
    /// state).
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.active_status.is_active() && (self.marks.is_some() || self.is_absent)
    }
}
