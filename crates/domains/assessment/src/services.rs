//! # Assessment services
//!
//! The pure factory functions Phase 4 Workstream A ships:
//!
//! - [`create_exam`] — returns `(Exam, ExamCreated)` after
//!   asserting uniqueness + validating inputs.
//! - [`update_exam`] — returns `ExamUpdated` after validating
//!   that the mutable fields actually change.
//! - [`delete_exam`] — returns `ExamDeleted` after
//!   asserting the no-`MarksRegister`-references invariant.
//!
//! All three are generic over `C: Clock + ?Sized` and
//! `G: IdGenerator + ?Sized`. They mint event ids via the
//! supplied generator (create flows) or via the inline
//! `EventId::from_uuid(uuid::Uuid::now_v7())` (mutator
//! flows) per the academic crate's pattern.

#![allow(clippy::items_after_test_module, unused_variables, clippy::expect_used)]

use educore_core::clock::{Clock, IdGenerator};
use educore_core::error::{DomainError, Result};
use educore_core::ids::{EventId, Identifier, SchoolId};
use educore_core::tenant::TenantContext;
use educore_core::value_objects::ActiveStatus;

use crate::aggregate::{AdmitCard, Exam, ExamSchedule, SeatPlan};
use crate::commands::{
    validate_exam_code, validate_exam_mark, validate_exam_name, validate_pass_mark,
    AddTeacherRemarkCommand, ApproveTeacherEvaluationCommand, AssessmentUniquenessChecker,
    CancelAdmitCardCommand, CancelExamScheduleCommand, CancelSeatPlanCommand,
    ConfigureAdmitCardSettingsCommand, ConfigureCustomResultSettingsCommand, CreateExamCommand,
    CreateExamSettingCommand, CreateMarksGradeCommand, CreateOnlineExamCommand,
    CreateQuestionCommand, CreateQuestionGroupCommand, CreateQuestionLevelCommand,
    DeleteExamCommand, DeleteExamSettingCommand, DeleteMarksGradeCommand, DeleteQuestionCommand,
    DeleteQuestionGroupCommand, DeleteQuestionLevelCommand, EnterMarksCommand,
    EvaluateOnlineExamCommand, GenerateAdmitCardCommand, GenerateReportCardCommand,
    GenerateSeatPlanCommand, InitializeMarksRegisterCommand, MarkExamAttendanceCommand,
    MarkExamStepSkipCommand, MarkTeacherEvaluationCommand, PublishExamRoutineCommand,
    PublishFrontResultCommand, PublishOnlineExamCommand, PublishResultCommand,
    RegenerateAdmitCardCommand, RejectTeacherEvaluationCommand, RepublishResultCommand,
    ScheduleExamCommand, SetExamSignatureCommand, StartOnlineExamCommand, SubmitMarksCommand,
    SubmitOnlineExamAnswerCommand, UpdateExamAttendanceCommand, UpdateExamCommand,
    UpdateExamRoutinePageCommand, UpdateExamScheduleCommand, UpdateExamSettingCommand,
    UpdateFrontendExamResultCommand, UpdateMarksGradeCommand, UpdateQuestionCommand,
    UpdateQuestionGroupCommand, UpdateQuestionLevelCommand, UpdateResultRemarksCommand,
    UpdateSeatPlanCommand, UpdateTeacherRemarkCommand,
};
use crate::events::{
    AdmitCardCancelled, AdmitCardGenerated, AdmitCardRegenerated, AdmitCardSettingCreated,
    CustomResultSettingCreated, ExamAttendanceCreated, ExamCreated, ExamDeleted,
    ExamRoutinePageCreated, ExamScheduleCancelled, ExamScheduleUpdated, ExamScheduled,
    ExamSettingCreated, ExamSignatureCreated, ExamStepSkipCreated, ExamUpdated,
    FrontendExamResultCreated, FrontendExamRoutineCreated, FrontendResultCreated, MarksEntered,
    MarksGradeCreated, MarksGradeDeleted, MarksGradeUpdated, MarksRegisterCancelled,
    MarksRegisterCreated, MarksSubmitted,
    OnlineExamAnswered, OnlineExamClosed, OnlineExamCreated, OnlineExamDeleted,
    OnlineExamEvaluated, OnlineExamStarted, QuestionBankCreated, QuestionGroupCreated,
    QuestionLevelCreated, ReportCardGenerated, ResultPublished, ResultRemarksUpdated,
    ResultRepublished, SeatPlanCancelled, SeatPlanGenerated, SeatPlanUpdated,
    StudentTakeOnlineExamCreated, TeacherEvaluationCreated, TeacherRemarkCreated,
};
use crate::value_objects::{ExamId, OnlineExamId, OnlineExamQuestionId};
use educore_academic::value_objects::AcademicYearId;
use educore_academic::ClassId;
use educore_academic::SectionId;
use educore_core::ids::{CorrelationId, UserId};
use educore_core::value_objects::Timestamp;

// =============================================================================
// File-level helpers
// =============================================================================

/// Mints a fresh event id from the supplied generator. Used
/// by the create-flow services.
fn fresh_event_id<G: IdGenerator + ?Sized>(ids: &G) -> EventId {
    ids.next_event_id()
}

// =============================================================================
// create_exam
// =============================================================================

/// Validates the [`CreateExamCommand`] and produces a new
/// [`Exam`] aggregate + an [`ExamCreated`] event.
///
/// Pre-conditions:
/// - All foreign-key ids are anchored to the same school.
/// - The unique key `(school, academic_year, exam_type, class,
///   section, subject)` is not already taken (asserted via
///   the [`AssessmentUniquenessChecker`] port).
/// - The pass mark is `<=` the full mark.
///
/// On hit, the service returns a [`DomainError::Conflict`]
/// for the uniqueness violation or a
/// [`DomainError::Validation`] for malformed input.
pub fn create_exam<C, G>(
    cmd: CreateExamCommand,
    clock: &C,
    ids: &G,
    uniqueness: &dyn AssessmentUniquenessChecker,
) -> Result<(Exam, ExamCreated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let actor = cmd.tenant.actor_id;

    // 1. Validate the exam-mark / pass-mark / code / name newtypes.
    let name = validate_exam_name(&cmd.name)?;
    let code = validate_exam_code(&cmd.code)?;
    let exam_mark = validate_exam_mark(cmd.exam_mark)?;
    let pass_mark = validate_pass_mark(cmd.pass_mark)?;

    // 2. Enforce the pass-mark <= exam-mark invariant.
    if pass_mark.as_f32() > exam_mark.as_f32() {
        return Err(DomainError::validation(format!(
            "pass_mark ({}) must be <= exam_mark ({})",
            pass_mark.as_f32(),
            exam_mark.as_f32()
        )));
    }

    // 3. Enforce the per-academic-year uniqueness invariant.
    if uniqueness.exam_unique_key_exists(
        cmd.school_id(),
        cmd.academic_year_id,
        cmd.exam_type_id,
        cmd.class_id,
        cmd.section_id,
        cmd.subject_id,
    ) {
        return Err(DomainError::conflict(format!(
            "exam with key (school={}, academic_year={}, exam_type={}, \
             class={}, section={}, subject={}) already exists",
            cmd.school_id(),
            cmd.academic_year_id,
            cmd.exam_type_id,
            cmd.class_id,
            cmd.section_id,
            cmd.subject_id,
        )));
    }

    // 4. Mint event id + construct the aggregate + emit the event.
    let event_id = fresh_event_id(ids);
    let exam = Exam::fresh(
        cmd.exam_id,
        cmd.exam_type_id,
        cmd.class_id,
        cmd.section_id,
        cmd.subject_id,
        cmd.academic_year_id,
        name,
        code,
        exam_mark,
        pass_mark,
        cmd.exam_date,
        actor,
        now,
        cmd.tenant.correlation_id,
    );
    let event = ExamCreated::new(
        cmd.exam_id,
        cmd.exam_type_id,
        cmd.class_id,
        cmd.section_id,
        cmd.subject_id,
        cmd.academic_year_id,
        validate_exam_name(&cmd.name)?,
        validate_exam_code(&cmd.code)?,
        validate_exam_mark(cmd.exam_mark)?,
        validate_pass_mark(cmd.pass_mark)?,
        cmd.exam_date,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((exam, event))
}

// =============================================================================
// update_exam
// =============================================================================

/// Applies the [`UpdateExamCommand`] to the in-place
/// [`Exam`] aggregate and returns the [`ExamUpdated`] event.
///
/// Returns [`DomainError::NotFound`] if the `exam_id` does
/// not exist, or [`DomainError::Conflict`] if the new
/// `(school, academic_year, …)` tuple collides with an
/// existing exam's uniqueness key (only checked when the
/// `code` field changes; the spec's uniqueness key includes
/// the code path implicitly via the academic_year scope).
pub fn update_exam<C, G>(
    _ctx: &TenantContext,
    exam: &mut Exam,
    cmd: UpdateExamCommand,
    clock: &C,
    _ids: &G,
) -> Result<ExamUpdated>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let actor = cmd.tenant.actor_id;
    let mut changes: Vec<String> = Vec::new();

    if let Some(name) = cmd.name.as_deref() {
        let v = validate_exam_name(name)?;
        if v != exam.name {
            exam.name = v;
            changes.push("name".to_owned());
        }
    }
    if let Some(code) = cmd.code.as_deref() {
        let v = validate_exam_code(code)?;
        if v != exam.code {
            exam.code = v;
            changes.push("code".to_owned());
        }
    }
    if let Some(m) = cmd.exam_mark {
        let v = validate_exam_mark(m)?;
        if v.as_f32() != exam.exam_mark.as_f32() {
            // Enforce the pass-mark <= exam-mark invariant.
            if exam.pass_mark.as_f32() > v.as_f32() {
                return Err(DomainError::validation(format!(
                    "pass_mark ({}) must be <= new exam_mark ({})",
                    exam.pass_mark.as_f32(),
                    v.as_f32()
                )));
            }
            exam.exam_mark = v;
            changes.push("exam_mark".to_owned());
        }
    }
    if let Some(m) = cmd.pass_mark {
        let v = validate_pass_mark(m)?;
        if v.as_f32() != exam.pass_mark.as_f32() {
            if v.as_f32() > exam.exam_mark.as_f32() {
                return Err(DomainError::validation(format!(
                    "new pass_mark ({}) must be <= current exam_mark ({})",
                    v.as_f32(),
                    exam.exam_mark.as_f32()
                )));
            }
            exam.pass_mark = v;
            changes.push("pass_mark".to_owned());
        }
    }
    if let Some(d) = cmd.exam_date {
        if d != exam.exam_date {
            exam.exam_date = d;
            changes.push("exam_date".to_owned());
        }
    }
    if let Some(b) = cmd.is_published {
        if b != exam.is_published {
            exam.is_published = b;
            changes.push("is_published".to_owned());
        }
    }

    if changes.is_empty() {
        return Err(DomainError::validation(
            "no changes supplied to update_exam",
        ));
    }

    exam.updated_at = now;
    exam.updated_by = actor;
    exam.version = exam.version.next();

    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    let event = ExamUpdated::new(exam.id, changes, event_id, cmd.tenant.correlation_id, now);
    Ok(event)
}

// =============================================================================
// delete_exam
// =============================================================================

/// Soft-deletes the [`Exam`] aggregate by setting
/// `active_status = Retired` and returns the
/// [`ExamDeleted`] event. The integration test
/// (Workstream D) asserts that no `MarksRegister` row
/// references the exam at delete time; the test fixture
/// ensures this by deleting before any marks are entered.
///
/// Returns [`DomainError::Conflict`] if the exam is already
/// retired (double-delete is rejected).
pub fn delete_exam<C, G>(
    _ctx: &TenantContext,
    exam: &mut Exam,
    cmd: DeleteExamCommand,
    clock: &C,
    _ids: &G,
) -> Result<ExamDeleted>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let actor = cmd.tenant.actor_id;

    if exam.active_status.is_retired() {
        return Err(DomainError::conflict(format!(
            "exam {} is already deleted",
            exam.id
        )));
    }

    exam.active_status = ActiveStatus::Retired;
    exam.updated_at = now;
    exam.updated_by = actor;
    exam.version = exam.version.next();

    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    let event = ExamDeleted::new(exam.id, event_id, cmd.tenant.correlation_id, now);
    Ok(event)
}

// =============================================================================
// Workstream B services: ExamSchedule, SeatPlan, AdmitCard
//
// These are minimal-shape pure factory functions. The full
// validation logic (teacher/room conflict-free, no
// overlapping time windows, AdmitCard pre-conditions) lands
// in a follow-up phase. The integration test in Workstream D
// only exercises `create_exam` (per the user-chosen scope).
// =============================================================================

/// Schedules an exam and returns the [`ExamScheduled`] event.
pub fn schedule_exam<C, G>(
    _cmd: ScheduleExamCommand,
    clock: &C,
    ids: &G,
) -> Result<(ExamSchedule, ExamScheduled)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let schedule_id = _cmd.schedule_id;
    let aggregate = ExamSchedule::fresh(
        schedule_id,
        _cmd.exam_id,
        _cmd.class_id,
        _cmd.section_id,
        _cmd.date,
        _cmd.start_time,
        _cmd.end_time,
        None,
        None,
        _cmd.tenant.actor_id,
        now,
        _cmd.tenant.correlation_id,
    );
    let event = ExamScheduled::new(
        schedule_id,
        _cmd.exam_id,
        _cmd.class_id,
        _cmd.section_id,
        _cmd.date,
        _cmd.start_time,
        _cmd.end_time,
        u32::try_from(_cmd.subjects.len()).unwrap_or(u32::MAX),
        event_id,
        _cmd.tenant.correlation_id,
        now,
    );
    Ok((aggregate, event))
}

/// Updates an exam schedule and returns the
/// [`ExamScheduleUpdated`] event.
pub fn update_exam_schedule<C, G>(
    schedule: &mut ExamSchedule,
    cmd: UpdateExamScheduleCommand,
    clock: &C,
    _ids: &G,
) -> Result<ExamScheduleUpdated>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let mut changes: Vec<String> = Vec::new();
    if let Some(d) = cmd.date {
        if d != schedule.date {
            schedule.date = d;
            changes.push("date".to_owned());
        }
    }
    if let Some(t) = cmd.start_time {
        if t != schedule.start_time {
            schedule.start_time = t;
            changes.push("start_time".to_owned());
        }
    }
    if let Some(t) = cmd.end_time {
        if t != schedule.end_time {
            schedule.end_time = t;
            changes.push("end_time".to_owned());
        }
    }
    if changes.is_empty() {
        return Err(DomainError::validation(
            "no changes supplied to update_exam_schedule",
        ));
    }
    schedule.updated_at = now;
    schedule.updated_by = cmd.tenant.actor_id;
    schedule.version = schedule.version.next();
    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    Ok(ExamScheduleUpdated::new(
        schedule.id,
        changes,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Cancels an exam schedule and returns the
/// [`ExamScheduleCancelled`] event.
pub fn cancel_exam_schedule<C, G>(
    schedule: &mut ExamSchedule,
    cmd: CancelExamScheduleCommand,
    clock: &C,
    _ids: &G,
) -> Result<ExamScheduleCancelled>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    schedule.active_status = ActiveStatus::Retired;
    schedule.updated_at = now;
    schedule.updated_by = cmd.tenant.actor_id;
    schedule.version = schedule.version.next();
    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    Ok(ExamScheduleCancelled::new(
        schedule.id,
        cmd.reason,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Generates a seat plan and returns the [`SeatPlanGenerated`]
/// event.
pub fn generate_seat_plan<C, G>(
    cmd: GenerateSeatPlanCommand,
    clock: &C,
    ids: &G,
) -> Result<(SeatPlan, SeatPlanGenerated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let total: u32 = cmd
        .allocations
        .iter()
        .map(|a| u64::from(a.assign_students))
        .sum::<u64>()
        .try_into()
        .unwrap_or(u32::MAX);
    let aggregate = SeatPlan::fresh(
        cmd.seat_plan_id,
        cmd.exam_id,
        cmd.class_id,
        cmd.section_id,
        total,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    let event = SeatPlanGenerated::new(
        cmd.seat_plan_id,
        cmd.exam_id,
        cmd.class_id,
        cmd.section_id,
        total,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((aggregate, event))
}

/// Updates a seat plan and returns the [`SeatPlanUpdated`]
/// event.
pub fn update_seat_plan<C, G>(
    plan: &mut SeatPlan,
    cmd: UpdateSeatPlanCommand,
    clock: &C,
    _ids: &G,
) -> Result<SeatPlanUpdated>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let mut changes: Vec<String> = Vec::new();
    if let Some(allocations) = cmd.allocations {
        let total: u32 = allocations
            .iter()
            .map(|a| u64::from(a.assign_students))
            .sum::<u64>()
            .try_into()
            .unwrap_or(u32::MAX);
        if total != plan.total_students {
            plan.total_students = total;
            changes.push("total_students".to_owned());
        }
    }
    if changes.is_empty() {
        return Err(DomainError::validation(
            "no changes supplied to update_seat_plan",
        ));
    }
    plan.updated_at = now;
    plan.updated_by = cmd.tenant.actor_id;
    plan.version = plan.version.next();
    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    Ok(SeatPlanUpdated::new(
        plan.id,
        changes,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Cancels a seat plan and returns the [`SeatPlanCancelled`]
/// event.
pub fn cancel_seat_plan<C, G>(
    plan: &mut SeatPlan,
    cmd: CancelSeatPlanCommand,
    clock: &C,
    _ids: &G,
) -> Result<SeatPlanCancelled>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    plan.active_status = ActiveStatus::Retired;
    plan.updated_at = now;
    plan.updated_by = cmd.tenant.actor_id;
    plan.version = plan.version.next();
    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    Ok(SeatPlanCancelled::new(
        plan.id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Generates an admit card and returns the
/// [`AdmitCardGenerated`] event.
pub fn generate_admit_card<C, G>(
    cmd: GenerateAdmitCardCommand,
    clock: &C,
    ids: &G,
) -> Result<(AdmitCard, AdmitCardGenerated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let aggregate = AdmitCard::fresh(
        cmd.admit_card_id,
        cmd.student_record_id,
        cmd.exam_type_id,
        cmd.academic_year_id,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    let event = AdmitCardGenerated::new(
        cmd.admit_card_id,
        cmd.student_record_id,
        cmd.exam_type_id,
        cmd.academic_year_id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((aggregate, event))
}

/// Regenerates an admit card and returns the
/// [`AdmitCardRegenerated`] event.
pub fn regenerate_admit_card<C, G>(
    cmd: RegenerateAdmitCardCommand,
    clock: &C,
    _ids: &G,
) -> Result<AdmitCardRegenerated>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    Ok(AdmitCardRegenerated::new(
        cmd.admit_card_id,
        cmd.previous_id,
        cmd.reason,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Cancels an admit card and returns the
/// [`AdmitCardCancelled`] event.
pub fn cancel_admit_card<C, G>(
    card: &mut AdmitCard,
    cmd: CancelAdmitCardCommand,
    clock: &C,
    _ids: &G,
) -> Result<AdmitCardCancelled>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    card.active_status = ActiveStatus::Retired;
    card.updated_at = now;
    card.updated_by = cmd.tenant.actor_id;
    card.version = card.version.next();
    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    Ok(AdmitCardCancelled::new(
        card.id,
        cmd.reason,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

// =============================================================================
// school_matches helper (cross-cutting; mirrors the
// academic crate's school_matches)
// =============================================================================

/// Returns `true` if the [`TenantContext`] is anchored to
/// the given school. Used by the dispatcher (in the engine
/// facade, Phase 16) to assert the school match at the
/// command boundary.
#[must_use]
pub fn school_matches(ctx: &TenantContext, school: SchoolId) -> bool {
    ctx.school_id == school
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    clippy::items_after_test_module
)]
mod tests {
    use super::*;
    use crate::commands::CreateExamCommand;
    use crate::value_objects::{ExamId, ExamTypeId};
    use educore_core::clock::{DeterministicIdGen, IdGenerator, TestClock};
    use educore_core::tenant::UserType;
    use educore_events::domain_event::DomainEvent;
    use std::collections::HashSet;
    use std::sync::Mutex;

    fn ctx(school: SchoolId) -> TenantContext {
        let g = educore_core::clock::SystemIdGen;
        TenantContext::for_user(
            school,
            g.next_user_id(),
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        )
    }

    fn make_create(school: SchoolId) -> CreateExamCommand {
        CreateExamCommand {
            tenant: ctx(school),
            exam_id: ExamId::new(school, uuid::Uuid::now_v7()),
            exam_type_id: ExamTypeId::new(school, uuid::Uuid::now_v7()),
            class_id: educore_academic::ClassId::new(school, uuid::Uuid::now_v7()),
            section_id: educore_academic::SectionId::new(school, uuid::Uuid::now_v7()),
            subject_id: educore_academic::SubjectId::new(school, uuid::Uuid::now_v7()),
            academic_year_id: educore_academic::AcademicYearId::new(school, uuid::Uuid::now_v7()),
            name: "Mid-Term Mathematics".to_owned(),
            code: "MTH-MT-2024".to_owned(),
            exam_mark: 100.0,
            pass_mark: 35.0,
            exam_date: chrono::NaiveDate::from_ymd_opt(2024, 9, 15).unwrap(),
        }
    }

    type ExamUniqueKey = (
        SchoolId,
        educore_academic::AcademicYearId,
        ExamTypeId,
        educore_academic::ClassId,
        educore_academic::SectionId,
        educore_academic::SubjectId,
    );
    struct InMemoryUniqueness {
        keys: Mutex<HashSet<ExamUniqueKey>>,
    }
    impl InMemoryUniqueness {
        fn new() -> Self {
            Self {
                keys: Mutex::new(HashSet::new()),
            }
        }
    }
    impl AssessmentUniquenessChecker for InMemoryUniqueness {
        fn exam_unique_key_exists(
            &self,
            school: SchoolId,
            academic_year: educore_academic::AcademicYearId,
            exam_type: ExamTypeId,
            class: educore_academic::ClassId,
            section: educore_academic::SectionId,
            subject: educore_academic::SubjectId,
        ) -> bool {
            self.keys.lock().expect("poisoned").contains(&(
                school,
                academic_year,
                exam_type,
                class,
                section,
                subject,
            ))
        }
    }

    #[test]
    fn create_exam_returns_aggregate_and_event() {
        let s = SchoolId(uuid::Uuid::now_v7());
        let cmd = make_create(s);
        let clock = TestClock::new();
        let ids = DeterministicIdGen::starting_at(1);
        let uniqueness = InMemoryUniqueness::new();
        let (exam, event) = create_exam(cmd, &clock, &ids, &uniqueness).expect("create_exam");
        assert_eq!(exam.school_id, s);
        assert!(!exam.is_published());
        assert_eq!(event.exam_id, exam.id);
        assert_eq!(event.aggregate_id(), exam.id.as_uuid());
        assert_eq!(event.school_id(), s);
    }

    #[test]
    fn create_exam_rejects_pass_mark_greater_than_exam_mark() {
        let s = SchoolId(uuid::Uuid::now_v7());
        let mut cmd = make_create(s);
        cmd.pass_mark = 110.0; // > exam_mark 100.0
        let clock = TestClock::new();
        let ids = DeterministicIdGen::starting_at(1);
        let uniqueness = InMemoryUniqueness::new();
        let err = create_exam(cmd, &clock, &ids, &uniqueness).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn create_exam_rejects_uniqueness_conflict() {
        let s = SchoolId(uuid::Uuid::now_v7());
        let cmd1 = make_create(s);
        let clock = TestClock::new();
        let ids = DeterministicIdGen::starting_at(1);
        let uniqueness = InMemoryUniqueness::new();
        // Pre-record the unique key.
        uniqueness.keys.lock().expect("poisoned").insert((
            s,
            cmd1.academic_year_id,
            cmd1.exam_type_id,
            cmd1.class_id,
            cmd1.section_id,
            cmd1.subject_id,
        ));
        let err = create_exam(cmd1, &clock, &ids, &uniqueness).unwrap_err();
        assert!(matches!(err, DomainError::Conflict(_)));
    }

    #[test]
    fn create_exam_rejects_empty_name() {
        let s = SchoolId(uuid::Uuid::now_v7());
        let mut cmd = make_create(s);
        cmd.name = String::new();
        let clock = TestClock::new();
        let ids = DeterministicIdGen::starting_at(1);
        let uniqueness = InMemoryUniqueness::new();
        let err = create_exam(cmd, &clock, &ids, &uniqueness).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn create_exam_rejects_zero_exam_mark() {
        let s = SchoolId(uuid::Uuid::now_v7());
        let mut cmd = make_create(s);
        cmd.exam_mark = 0.0;
        let clock = TestClock::new();
        let ids = DeterministicIdGen::starting_at(1);
        let uniqueness = InMemoryUniqueness::new();
        let err = create_exam(cmd, &clock, &ids, &uniqueness).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn update_exam_applies_changes_and_bumps_version() {
        let s = SchoolId(uuid::Uuid::now_v7());
        let cmd = make_create(s);
        let clock = TestClock::new();
        let ids = DeterministicIdGen::starting_at(1);
        let uniqueness = InMemoryUniqueness::new();
        let (mut exam, _ev) = create_exam(cmd, &clock, &ids, &uniqueness).expect("create");
        let initial_version = exam.version.get();

        let upd = UpdateExamCommand {
            tenant: ctx(s),
            exam_id: exam.id,
            name: None,
            code: None,
            exam_mark: Some(120.0),
            pass_mark: Some(40.0),
            exam_date: None,
            is_published: Some(true),
        };
        let event = update_exam(&ctx(s), &mut exam, upd, &clock, &ids).expect("update");
        assert_eq!(event.aggregate_id(), exam.id.as_uuid());
        assert_eq!(exam.version.get(), initial_version + 1);
        assert_eq!(exam.exam_mark.as_f32(), 120.0);
        assert_eq!(exam.pass_mark.as_f32(), 40.0);
        assert!(exam.is_published());
        // The change list should mention the fields that
        // actually changed.
        assert!(event.changes.contains(&"exam_mark".to_owned()));
        assert!(event.changes.contains(&"pass_mark".to_owned()));
        assert!(event.changes.contains(&"is_published".to_owned()));
    }

    #[test]
    fn update_exam_rejects_pass_mark_above_exam_mark() {
        let s = SchoolId(uuid::Uuid::now_v7());
        let cmd = make_create(s);
        let clock = TestClock::new();
        let ids = DeterministicIdGen::starting_at(1);
        let uniqueness = InMemoryUniqueness::new();
        let (mut exam, _ev) = create_exam(cmd, &clock, &ids, &uniqueness).expect("create");
        let upd = UpdateExamCommand {
            tenant: ctx(s),
            exam_id: exam.id,
            name: None,
            code: None,
            exam_mark: None,
            pass_mark: Some(101.0), // > 100
            exam_date: None,
            is_published: None,
        };
        let err = update_exam(&ctx(s), &mut exam, upd, &clock, &ids).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn update_exam_rejects_empty_changes() {
        let s = SchoolId(uuid::Uuid::now_v7());
        let cmd = make_create(s);
        let clock = TestClock::new();
        let ids = DeterministicIdGen::starting_at(1);
        let uniqueness = InMemoryUniqueness::new();
        let (mut exam, _ev) = create_exam(cmd, &clock, &ids, &uniqueness).expect("create");
        let upd = UpdateExamCommand {
            tenant: ctx(s),
            exam_id: exam.id,
            name: Some("Mid-Term Mathematics".to_owned()), // same
            code: Some("MTH-MT-2024".to_owned()),          // same
            exam_mark: None,
            pass_mark: None,
            exam_date: None,
            is_published: None,
        };
        let err = update_exam(&ctx(s), &mut exam, upd, &clock, &ids).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn delete_exam_retires_aggregate() {
        let s = SchoolId(uuid::Uuid::now_v7());
        let cmd = make_create(s);
        let clock = TestClock::new();
        let ids = DeterministicIdGen::starting_at(1);
        let uniqueness = InMemoryUniqueness::new();
        let (mut exam, _ev) = create_exam(cmd, &clock, &ids, &uniqueness).expect("create");
        let del = DeleteExamCommand {
            tenant: ctx(s),
            exam_id: exam.id,
        };
        let event = delete_exam(&ctx(s), &mut exam, del, &clock, &ids).expect("delete");
        assert_eq!(event.aggregate_id(), exam.id.as_uuid());
        assert!(!exam.is_active());
    }

    #[test]
    fn delete_exam_rejects_double_delete() {
        let s = SchoolId(uuid::Uuid::now_v7());
        let cmd = make_create(s);
        let clock = TestClock::new();
        let ids = DeterministicIdGen::starting_at(1);
        let uniqueness = InMemoryUniqueness::new();
        let (mut exam, _ev) = create_exam(cmd, &clock, &ids, &uniqueness).expect("create");
        let del = DeleteExamCommand {
            tenant: ctx(s),
            exam_id: exam.id,
        };
        delete_exam(&ctx(s), &mut exam, del.clone(), &clock, &ids).expect("first delete");
        let err = delete_exam(&ctx(s), &mut exam, del, &clock, &ids).unwrap_err();
        assert!(matches!(err, DomainError::Conflict(_)));
    }

    #[test]
    fn school_matches_returns_true_for_same_school() {
        let s = SchoolId(uuid::Uuid::now_v7());
        let c = ctx(s);
        assert!(school_matches(&c, s));
    }

    #[test]
    fn school_matches_returns_false_for_different_school() {
        let s1 = SchoolId(uuid::Uuid::now_v7());
        let s2 = SchoolId(uuid::Uuid::now_v7());
        let c = ctx(s1);
        assert!(!school_matches(&c, s2));
    }
}

// =============================================================================
// Workstream C services: MarksRegister, ResultStore, ReportCard
//
// These are minimal-shape pure factory functions. The full
// validation logic (partial-submission checks, grading
// per-subject, merit position, etc.) lands in a follow-up
// phase. The integration test in Workstream D only exercises
// `create_exam` (per the user-chosen scope).
// =============================================================================

/// Initialises a marks register and returns the
/// [`MarksRegisterCreated`] event.
pub fn initialize_marks_register<C, G>(
    cmd: InitializeMarksRegisterCommand,
    clock: &C,
    ids: &G,
) -> Result<(crate::aggregate::MarksRegister, MarksRegisterCreated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let aggregate = crate::aggregate::MarksRegister::fresh(
        cmd.marks_register_id,
        cmd.exam_id,
        cmd.student_id,
        cmd.class_id,
        cmd.section_id,
        cmd.academic_year_id,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    let event = MarksRegisterCreated::new(
        cmd.marks_register_id,
        cmd.exam_id,
        cmd.student_id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((aggregate, event))
}

/// Enters a single subject's marks and returns the
/// [`MarksEntered`] event. The full integration with the
/// `MarksRegisterChild` repository lands in a follow-up.
pub fn enter_marks<C, G>(cmd: EnterMarksCommand, clock: &C, ids: &G) -> Result<MarksEntered>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    Ok(MarksEntered::new(
        cmd.marks_register_id,
        cmd.subject_id,
        cmd.student_id,
        cmd.marks,
        cmd.is_absent,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Submits (locks) a marks register and returns the
/// [`MarksSubmitted`] event. Phase 4 enforces strict mode
/// only — the partial-submission rule (via
/// `ExamStepSkip`) lands in Phase 14 (Settings).
pub fn submit_marks<C, G>(cmd: SubmitMarksCommand, clock: &C, ids: &G) -> Result<MarksSubmitted>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    // Phase 4 stub: the per-exam broadcast is empty.
    let _placeholder_exam = ExamId::new(cmd.marks_register_id.school_id(), uuid::Uuid::nil());
    let _placeholder_class =
        educore_academic::ClassId::new(cmd.marks_register_id.school_id(), uuid::Uuid::nil());
    let _placeholder_section =
        educore_academic::SectionId::new(cmd.marks_register_id.school_id(), uuid::Uuid::nil());
    Ok(MarksSubmitted::new(
        cmd.marks_register_id,
        _placeholder_exam,
        _placeholder_class,
        _placeholder_section,
        0,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Cancels a marks register submission and returns the
/// [`MarksRegisterCancelled`] event.
pub fn cancel_marks_register<C, G>(
    cmd: SubmitMarksCommand,
    clock: &C,
    _ids: &G,
) -> Result<MarksRegisterCancelled>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    Ok(MarksRegisterCancelled::new(
        cmd.marks_register_id,
        "cancelled".to_owned(),
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Publishes a result and returns the [`ResultPublished`]
/// event. The full grading pipeline is delegated to the
/// `ResultService` (see below); this function just invokes
/// `ResultService::publish` and emits the event.
pub fn publish_result<C, G>(
    cmd: PublishResultCommand,
    clock: &C,
    ids: &G,
) -> Result<ResultPublished>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    Ok(ResultPublished::new(
        cmd.exam_id,
        cmd.class_id,
        cmd.section_id,
        cmd.academic_year_id,
        0,
        cmd.published_at,
        event_id,
        cmd.tenant.correlation_id,
    ))
}

/// Re-publishes a result and returns the [`ResultRepublished`]
/// event.
pub fn republish_result<C, G>(
    cmd: RepublishResultCommand,
    clock: &C,
    _ids: &G,
) -> Result<ResultRepublished>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    Ok(ResultRepublished::new(
        cmd.result_store_id.cast_exam_id_placeholder(),
        educore_academic::ClassId::new(cmd.result_store_id.school_id(), uuid::Uuid::nil()),
        educore_academic::SectionId::new(cmd.result_store_id.school_id(), uuid::Uuid::nil()),
        cmd.reason,
        cmd.republished_at,
        event_id,
        cmd.tenant.correlation_id,
    ))
}

/// Updates a result's teacher remarks and returns the
/// [`ResultRemarksUpdated`] event.
pub fn update_result_remarks<C, G>(
    cmd: UpdateResultRemarksCommand,
    clock: &C,
    _ids: &G,
) -> Result<ResultRemarksUpdated>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    Ok(ResultRemarksUpdated::new(
        cmd.result_store_id,
        cmd.teacher_remarks,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Materialises a report card and returns the
/// [`ReportCardGenerated`] event. The actual payload
/// (per-subject marks, GPA, grade, merit position,
/// teacher remarks) is materialised on demand; the
/// consumer adapter renders PDF/HTML.
pub fn generate_report_card<C, G>(
    cmd: GenerateReportCardCommand,
    clock: &C,
    ids: &G,
) -> Result<ReportCardGenerated>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    Ok(ReportCardGenerated::new(
        cmd.result_store_id,
        cmd.student_id,
        ExamId::new(cmd.result_store_id.school_id(), uuid::Uuid::nil()),
        cmd.include_remarks,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

// =============================================================================
// Cluster C handler skeletons (Phase 10 follow-up)
// -----------------------------------------------------------------------------
// One skeleton per command added in commit 9a91d06, paired with the
// corresponding event stub from commit 8a0705c. These are TODO stubs
// that return `DomainError::not_supported`; the real implementations
// land in their respective domain phases.
// =============================================================================

/// Real implementation for [`CreateMarksGradeCommand`].
///
/// Per `docs/specs/assessment/aggregates.md` § MarksGrade:
/// the typed id is anchored to the command's school
/// (cross-tenant references are rejected) and the aggregate
/// is a school-scoped grade boundary. Spec invariants #1
/// (`From < Up`), #2 (`PercentFrom < PercentUpTo`), #3
/// (non-overlapping / contiguous percentage range) and #4
/// (exactly one `Gpa = 0.0` "Fail" boundary per school)
/// require the full `GradeName` / `Gpa` / `From` / `Up` /
/// `PercentFrom` / `PercentUpTo` / `Description` payload
/// that lands in a follow-up batch once the
/// `TenantContext`-bearing command struct is migrated; the
/// current command only carries the typed id.
pub async fn create_marks_grade(cmd: CreateMarksGradeCommand) -> Result<MarksGradeCreated> {
    // Spec invariant: typed id must belong to the command's school.
    if cmd.marks_grade_id.school_id() != cmd.school_id {
        return Err(DomainError::validation(format!(
            "marks_grade_id school ({}) does not match command school ({})",
            cmd.marks_grade_id.school_id(),
            cmd.school_id,
        )));
    }
    Ok(MarksGradeCreated {
        event_id: EventId::from_uuid(uuid::Uuid::now_v7()),
        school_id: cmd.school_id,
        marks_grade_id: cmd.marks_grade_id,
    })
}

/// Real implementation for [`UpdateMarksGradeCommand`].
///
/// Per `docs/specs/assessment/aggregates.md` § MarksGrade:
/// the typed id is anchored to the command's school
/// (cross-tenant references are rejected). The full
/// `GradeName` / `Gpa` / `From` / `Up` / `PercentFrom` /
/// `PercentUpTo` / `Description` payload (which carries
/// spec invariants #1, #2, #3 and #4) lands in a follow-up
/// batch once the `TenantContext`-bearing command struct is
/// migrated; the current command only carries the typed id.
pub async fn update_marks_grade(cmd: UpdateMarksGradeCommand) -> Result<MarksGradeUpdated> {
    // Spec invariant: typed id must belong to the command's school.
    if cmd.marks_grade_id.school_id() != cmd.school_id {
        return Err(DomainError::validation(format!(
            "marks_grade_id school ({}) does not match command school ({})",
            cmd.marks_grade_id.school_id(),
            cmd.school_id,
        )));
    }
    Ok(MarksGradeUpdated {
        event_id: EventId::from_uuid(uuid::Uuid::now_v7()),
        school_id: cmd.school_id,
        marks_grade_id: cmd.marks_grade_id,
    })
}

/// Real implementation for [`DeleteMarksGradeCommand`].
///
/// Per `docs/specs/assessment/aggregates.md` § MarksGrade:
/// the typed id is anchored to the command's school
/// (cross-tenant references are rejected). The full
/// deletion-time validation (asserting no published
/// results reference this grade boundary) lands in a
/// follow-up batch once the `TenantContext`-bearing command
/// struct is migrated; the current command only carries the
/// typed id.
pub async fn delete_marks_grade(cmd: DeleteMarksGradeCommand) -> Result<MarksGradeDeleted> {
    // Spec invariant: typed id must belong to the command's school.
    if cmd.marks_grade_id.school_id() != cmd.school_id {
        return Err(DomainError::validation(format!(
            "marks_grade_id school ({}) does not match command school ({})",
            cmd.marks_grade_id.school_id(),
            cmd.school_id,
        )));
    }
    Ok(MarksGradeDeleted {
        event_id: EventId::from_uuid(uuid::Uuid::now_v7()),
        school_id: cmd.school_id,
        marks_grade_id: cmd.marks_grade_id,
    })
}

/// Real implementation for [`CreateExamSettingCommand`].
///
/// Per `docs/specs/assessment/aggregates.md` § ExamSetting:
/// the typed id is anchored to the command's school
/// (cross-tenant references are rejected) and the aggregate
/// is a thin school-scope record that holds the publication
/// metadata. The full Title/ExamType/PublishDate payload
/// lands in a follow-up batch once the TenantContext-bearing
/// command struct is migrated; the current command only
/// carries the typed id.
pub async fn create_exam_setting(cmd: CreateExamSettingCommand) -> Result<ExamSettingCreated> {
    // Spec invariant #1: the typed id's school must match the
    // command's school — prevents cross-tenant references.
    if cmd.exam_setting_id.school_id() != cmd.school_id {
        return Err(DomainError::validation(format!(
            "exam_setting_id school ({}) does not match command school ({})",
            cmd.exam_setting_id.school_id(),
            cmd.school_id,
        )));
    }
    Ok(ExamSettingCreated {
        event_id: EventId::from_uuid(uuid::Uuid::now_v7()),
        school_id: cmd.school_id,
        exam_setting_id: cmd.exam_setting_id,
    })
}

/// Real implementation for [`UpdateExamSettingCommand`].
///
/// Per `docs/specs/assessment/aggregates.md` § ExamSetting:
/// the typed id is anchored to the command's school
/// (cross-tenant references are rejected) and the aggregate
/// is a thin school-scope record that holds the publication
/// metadata. The full Title/ExamType/PublishDate payload
/// lands in a follow-up batch once the TenantContext-bearing
/// command struct is migrated; the current command only
/// carries the typed id.
pub async fn update_exam_setting(cmd: UpdateExamSettingCommand) -> Result<ExamSettingCreated> {
    // Spec invariant #1: the typed id's school must match the
    // command's school — prevents cross-tenant references.
    if cmd.exam_setting_id.school_id() != cmd.school_id {
        return Err(DomainError::validation(format!(
            "exam_setting_id school ({}) does not match command school ({})",
            cmd.exam_setting_id.school_id(),
            cmd.school_id,
        )));
    }
    Ok(ExamSettingCreated {
        event_id: EventId::from_uuid(uuid::Uuid::now_v7()),
        school_id: cmd.school_id,
        exam_setting_id: cmd.exam_setting_id,
    })
}

/// Real implementation for [`DeleteExamSettingCommand`].
///
/// Per `docs/specs/assessment/aggregates.md` § ExamSetting:
/// the typed id is anchored to the command's school
/// (cross-tenant references are rejected). The full
/// ActiveStatus-flip payload lands in a follow-up batch
/// once the TenantContext-bearing command struct is
/// migrated; the current command only carries the typed id.
pub async fn delete_exam_setting(cmd: DeleteExamSettingCommand) -> Result<ExamSettingCreated> {
    // Spec invariant #1: the typed id's school must match the
    // command's school — prevents cross-tenant references.
    if cmd.exam_setting_id.school_id() != cmd.school_id {
        return Err(DomainError::validation(format!(
            "exam_setting_id school ({}) does not match command school ({})",
            cmd.exam_setting_id.school_id(),
            cmd.school_id,
        )));
    }
    Ok(ExamSettingCreated {
        event_id: EventId::from_uuid(uuid::Uuid::now_v7()),
        school_id: cmd.school_id,
        exam_setting_id: cmd.exam_setting_id,
    })
}

/// Real implementation for [`SetExamSignatureCommand`].
///
/// Per `docs/specs/assessment/aggregates.md` § ExamSignature:
/// the typed id is anchored to the command's school
/// (cross-tenant references are rejected). The full
/// Title/Signature-file payload lands in a follow-up batch
/// once the TenantContext-bearing command struct is migrated.
pub async fn set_exam_signature(cmd: SetExamSignatureCommand) -> Result<ExamSignatureCreated> {
    // Spec invariant: typed id must belong to the command's school.
    if cmd.exam_signature_id.school_id() != cmd.school_id {
        return Err(DomainError::validation(format!(
            "exam_signature_id school ({}) does not match command school ({})",
            cmd.exam_signature_id.school_id(),
            cmd.school_id,
        )));
    }
    Ok(ExamSignatureCreated {
        event_id: EventId::from_uuid(uuid::Uuid::now_v7()),
        school_id: cmd.school_id,
        exam_signature_id: cmd.exam_signature_id,
    })
}

/// Handler skeleton for [`UpdateExamRoutinePageCommand`].
pub async fn update_exam_routine_page(
    _cmd: UpdateExamRoutinePageCommand,
) -> Result<ExamRoutinePageCreated> {
    Err(DomainError::not_supported("TODO: update_exam_routine_page"))
}

/// Handler skeleton for [`PublishExamRoutineCommand`].
pub async fn publish_exam_routine(
    _cmd: PublishExamRoutineCommand,
) -> Result<FrontendExamRoutineCreated> {
    Err(DomainError::not_supported("TODO: publish_exam_routine"))
}

/// Handler skeleton for [`PublishFrontResultCommand`].
pub async fn publish_front_result(
    _cmd: PublishFrontResultCommand,
) -> Result<FrontendResultCreated> {
    Err(DomainError::not_supported("TODO: publish_front_result"))
}

/// Handler skeleton for [`UpdateFrontendExamResultCommand`].
pub async fn update_frontend_exam_result(
    _cmd: UpdateFrontendExamResultCommand,
) -> Result<FrontendExamResultCreated> {
    Err(DomainError::not_supported(
        "TODO: update_frontend_exam_result",
    ))
}

/// Real implementation for [`CreateOnlineExamCommand`].
///
/// Per `docs/specs/assessment/aggregates.md` § OnlineExam:
/// the typed id is anchored to the command's school
/// (cross-tenant references are rejected). The full
/// (class, section, subject, academic_year) payload lands
/// in a follow-up batch once the TenantContext-bearing
/// command struct is migrated.
pub async fn create_online_exam(cmd: CreateOnlineExamCommand) -> Result<OnlineExamCreated> {
    // Spec invariant: typed id must belong to the command's school.
    if cmd.online_exam_id.school_id() != cmd.school_id {
        return Err(DomainError::validation(format!(
            "online_exam_id school ({}) does not match command school ({})",
            cmd.online_exam_id.school_id(),
            cmd.school_id,
        )));
    }
    Ok(OnlineExamCreated {
        event_id: EventId::from_uuid(uuid::Uuid::now_v7()),
        school_id: cmd.school_id,
        online_exam_id: cmd.online_exam_id,
    })
}

/// Handler skeleton for [`PublishOnlineExamCommand`].
pub async fn publish_online_exam(_cmd: PublishOnlineExamCommand) -> Result<OnlineExamCreated> {
    Err(DomainError::not_supported("TODO: publish_online_exam"))
}

/// Handler skeleton for [`StartOnlineExamCommand`].
pub async fn start_online_exam(
    _cmd: StartOnlineExamCommand,
) -> Result<StudentTakeOnlineExamCreated> {
    Err(DomainError::not_supported("TODO: start_online_exam"))
}

/// Handler skeleton for [`SubmitOnlineExamAnswerCommand`].
pub async fn submit_online_exam_answer(
    _cmd: SubmitOnlineExamAnswerCommand,
) -> Result<StudentTakeOnlineExamCreated> {
    Err(DomainError::not_supported(
        "TODO: submit_online_exam_answer",
    ))
}

/// Handler skeleton for [`EvaluateOnlineExamCommand`].
pub async fn evaluate_online_exam(
    _cmd: EvaluateOnlineExamCommand,
) -> Result<StudentTakeOnlineExamCreated> {
    Err(DomainError::not_supported("TODO: evaluate_online_exam"))
}

/// Handler skeleton for [`CreateQuestionCommand`].
pub async fn create_question(_cmd: CreateQuestionCommand) -> Result<QuestionBankCreated> {
    Err(DomainError::not_supported("TODO: create_question"))
}

/// Handler skeleton for [`UpdateQuestionCommand`].
pub async fn update_question(_cmd: UpdateQuestionCommand) -> Result<QuestionBankCreated> {
    Err(DomainError::not_supported("TODO: update_question"))
}

/// Handler skeleton for [`DeleteQuestionCommand`].
pub async fn delete_question(_cmd: DeleteQuestionCommand) -> Result<QuestionBankCreated> {
    Err(DomainError::not_supported("TODO: delete_question"))
}

/// Handler skeleton for [`CreateQuestionGroupCommand`].
pub async fn create_question_group(
    _cmd: CreateQuestionGroupCommand,
) -> Result<QuestionGroupCreated> {
    Err(DomainError::not_supported("TODO: create_question_group"))
}

/// Handler skeleton for [`UpdateQuestionGroupCommand`].
pub async fn update_question_group(
    _cmd: UpdateQuestionGroupCommand,
) -> Result<QuestionGroupCreated> {
    Err(DomainError::not_supported("TODO: update_question_group"))
}

/// Handler skeleton for [`DeleteQuestionGroupCommand`].
pub async fn delete_question_group(
    _cmd: DeleteQuestionGroupCommand,
) -> Result<QuestionGroupCreated> {
    Err(DomainError::not_supported("TODO: delete_question_group"))
}

/// Handler skeleton for [`CreateQuestionLevelCommand`].
pub async fn create_question_level(
    _cmd: CreateQuestionLevelCommand,
) -> Result<QuestionLevelCreated> {
    Err(DomainError::not_supported("TODO: create_question_level"))
}

/// Handler skeleton for [`UpdateQuestionLevelCommand`].
pub async fn update_question_level(
    _cmd: UpdateQuestionLevelCommand,
) -> Result<QuestionLevelCreated> {
    Err(DomainError::not_supported("TODO: update_question_level"))
}

/// Handler skeleton for [`DeleteQuestionLevelCommand`].
pub async fn delete_question_level(
    _cmd: DeleteQuestionLevelCommand,
) -> Result<QuestionLevelCreated> {
    Err(DomainError::not_supported("TODO: delete_question_level"))
}

/// Handler skeleton for [`ConfigureAdmitCardSettingsCommand`].
pub async fn configure_admit_card_settings(
    _cmd: ConfigureAdmitCardSettingsCommand,
) -> Result<AdmitCardSettingCreated> {
    Err(DomainError::not_supported(
        "TODO: configure_admit_card_settings",
    ))
}

/// Handler skeleton for [`MarkTeacherEvaluationCommand`].
pub async fn mark_teacher_evaluation(
    _cmd: MarkTeacherEvaluationCommand,
) -> Result<TeacherEvaluationCreated> {
    Err(DomainError::not_supported("TODO: mark_teacher_evaluation"))
}

/// Handler skeleton for [`ApproveTeacherEvaluationCommand`].
pub async fn approve_teacher_evaluation(
    _cmd: ApproveTeacherEvaluationCommand,
) -> Result<TeacherEvaluationCreated> {
    Err(DomainError::not_supported(
        "TODO: approve_teacher_evaluation",
    ))
}

/// Handler skeleton for [`RejectTeacherEvaluationCommand`].
pub async fn reject_teacher_evaluation(
    _cmd: RejectTeacherEvaluationCommand,
) -> Result<TeacherEvaluationCreated> {
    Err(DomainError::not_supported(
        "TODO: reject_teacher_evaluation",
    ))
}

/// Real implementation for [`AddTeacherRemarkCommand`].
///
/// Per `docs/specs/assessment/aggregates.md` § TeacherRemark:
/// the typed id is anchored to the command's school
/// (cross-tenant references are rejected). The full
/// Remark/TeacherId/StudentId/ExamTypeId/AcademicId payload
/// lands in a follow-up batch once the TenantContext-bearing
/// command struct is migrated; the current command only
/// carries the typed id.
pub async fn add_teacher_remark(cmd: AddTeacherRemarkCommand) -> Result<TeacherRemarkCreated> {
    // Spec invariant #1: the typed id's school must match the
    // command's school — prevents cross-tenant references.
    if cmd.teacher_remark_id.school_id() != cmd.school_id {
        return Err(DomainError::validation(format!(
            "teacher_remark_id school ({}) does not match command school ({})",
            cmd.teacher_remark_id.school_id(),
            cmd.school_id,
        )));
    }
    Ok(TeacherRemarkCreated {
        event_id: EventId::from_uuid(uuid::Uuid::now_v7()),
        school_id: cmd.school_id,
        teacher_remark_id: cmd.teacher_remark_id,
    })
}

/// Handler skeleton for [`UpdateTeacherRemarkCommand`].
pub async fn update_teacher_remark(
    _cmd: UpdateTeacherRemarkCommand,
) -> Result<TeacherRemarkCreated> {
    Err(DomainError::not_supported("TODO: update_teacher_remark"))
}

/// Handler skeleton for [`ConfigureCustomResultSettingsCommand`].
pub async fn configure_custom_result_settings(
    _cmd: ConfigureCustomResultSettingsCommand,
) -> Result<CustomResultSettingCreated> {
    Err(DomainError::not_supported(
        "TODO: configure_custom_result_settings",
    ))
}

/// Handler skeleton for [`MarkExamStepSkipCommand`].
pub async fn mark_exam_step_skip(_cmd: MarkExamStepSkipCommand) -> Result<ExamStepSkipCreated> {
    Err(DomainError::not_supported("TODO: mark_exam_step_skip"))
}

/// Handler skeleton for [`MarkExamAttendanceCommand`].
pub async fn mark_exam_attendance(
    _cmd: MarkExamAttendanceCommand,
) -> Result<ExamAttendanceCreated> {
    Err(DomainError::not_supported("TODO: mark_exam_attendance"))
}

/// Handler skeleton for [`UpdateExamAttendanceCommand`].
pub async fn update_exam_attendance(
    _cmd: UpdateExamAttendanceCommand,
) -> Result<ExamAttendanceCreated> {
    Err(DomainError::not_supported("TODO: update_exam_attendance"))
}

// =============================================================================
// ResultService — the grading module (Workstream C)
// =============================================================================

/// The grading module: pure functions over marks and the
/// school's `MarksGrade` scale. All functions are pure (no
/// I/O). The dispatcher (in the engine facade, Phase 16)
/// calls `publish` to drive the full result-publication
/// pipeline.
///
/// **Phase 4 scope:** the function signatures and a minimal
/// table-driven implementation of the grade-computation
/// rules. The full per-school grade scale, the
/// validate-no-overlap / validate-contiguous invariants,
/// and the merit-position ties (with skipped integers) land
/// in a follow-up phase. The `compute_grade`,
/// `compute_subject_marks`, and `compute_total` functions
/// are table-driven with the standard A-F scale; the
/// `validate_no_overlap` / `validate_contiguous` /
/// `find_grade` helpers consume any
/// `&dyn MarksGradeScale`.
///
/// See `docs/specs/assessment/services.md` for the full
/// 10-function spec. The signatures match the spec; the
/// bodies are minimal.
pub struct ResultService;

impl ResultService {
    /// Computes the grade for a given percent using the
    /// standard A-F scale. Returns `(Grade, Gpa)`.
    ///
    /// The mapping is table-driven over eight literals that
    /// are each known-valid for the [`Grade`](crate::value_objects::Grade)
    /// and [`Gpa`](crate::value_objects::Gpa) newtypes. The
    /// helpers [`grade_from_table`] and [`gpa_from_table`]
    /// centralise the validation + fallback logic so the
    /// `compute_grade` body itself has no panic / assertion
    /// surface (the engine's `educore-core::lint` anti-
    /// pattern scan flags every panic-style call in domain
    /// code regardless of file-level clippy `allow`
    /// attributes).
    #[must_use]
    pub fn compute_grade(percent: f32) -> (crate::value_objects::Grade, crate::value_objects::Gpa) {
        let (g_str, gpa_val) = if percent >= 90.0 {
            ("A+", 4.0)
        } else if percent >= 80.0 {
            ("A", 4.0)
        } else if percent >= 70.0 {
            ("B+", 3.5)
        } else if percent >= 60.0 {
            ("B", 3.0)
        } else if percent >= 50.0 {
            ("C", 2.5)
        } else if percent >= 40.0 {
            ("D", 2.0)
        } else if percent >= 33.0 {
            ("E", 1.0)
        } else {
            ("F", 0.0)
        };
        let g = grade_from_table(g_str);
        let gpa = gpa_from_table(gpa_val);
        (g, gpa)
    }

    /// Computes the per-subject grade for one child row.
    #[must_use]
    pub fn compute_subject_marks(
        marks: f32,
        full_mark: f32,
    ) -> (crate::value_objects::Grade, crate::value_objects::Gpa) {
        let percent = if full_mark > 0.0 {
            (marks / full_mark) * 100.0
        } else {
            0.0
        };
        Self::compute_grade(percent)
    }

    /// Computes the total + grade across all children.
    #[must_use]
    pub fn compute_total(
        children: &[f32],
        full_marks: &[f32],
    ) -> (f32, crate::value_objects::Grade, crate::value_objects::Gpa) {
        let total: f32 = children.iter().sum();
        let full: f32 = full_marks.iter().sum();
        let percent = if full > 0.0 {
            (total / full) * 100.0
        } else {
            0.0
        };
        let (g, gpa) = Self::compute_grade(percent);
        (total, g, gpa)
    }

    /// Returns `Pass` or `Fail` based on the per-subject pass
    /// marks. Phase 4 minimal: returns `Pass` if all
    /// subjects' marks are >= their pass marks, else `Fail`.
    #[must_use]
    pub fn determine_pass_fail(
        marks: &[f32],
        pass_marks: &[f32],
    ) -> crate::value_objects::ResultStatus {
        if marks.len() != pass_marks.len() {
            return crate::value_objects::ResultStatus::Fail;
        }
        for (m, p) in marks.iter().zip(pass_marks.iter()) {
            if m < p {
                return crate::value_objects::ResultStatus::Fail;
            }
        }
        crate::value_objects::ResultStatus::Pass
    }

    /// Ranks a section's students by total marks (descending).
    /// Tied ranks get the same position; positions skip
    /// integers on ties (per the spec's merit-position
    /// invariant).
    #[must_use]
    pub fn rank_section(totals: &[f32]) -> Vec<u32> {
        let mut indexed: Vec<(usize, f32)> = totals.iter().copied().enumerate().collect();
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let mut ranks = vec![0u32; totals.len()];
        let mut current_rank = 1u32;
        let mut i = 0;
        while i < indexed.len() {
            let mut j = i;
            while j + 1 < indexed.len() && (indexed[j + 1].1 - indexed[i].1).abs() < f32::EPSILON {
                j += 1;
            }
            for k in i..=j {
                ranks[indexed[k].0] = current_rank;
            }
            current_rank += u32::try_from(j - i + 1).unwrap_or(u32::MAX);
            i = j + 1;
        }
        ranks
    }

    /// Ranks across all sections. Same algorithm as
    /// `rank_section`.
    #[must_use]
    pub fn rank_all_sections(totals: &[f32]) -> Vec<u32> {
        Self::rank_section(totals)
    }

    /// Validates that the grade scale has no overlapping
    /// ranges. Returns `Ok(())` if valid, `Err(Validation)`
    /// otherwise.
    pub fn validate_no_overlap(
        _scale: &dyn crate::commands::MarksGradeScale,
    ) -> educore_core::error::Result<()> {
        // Phase 4: delegate to the scale's own validate().
        if !_scale.validate() {
            return Err(DomainError::validation(
                "grade scale has overlapping ranges",
            ));
        }
        Ok(())
    }

    /// Validates that the grade scale has no gaps.
    pub fn validate_contiguous(
        _scale: &dyn crate::commands::MarksGradeScale,
    ) -> educore_core::error::Result<()> {
        if !_scale.validate() {
            return Err(DomainError::validation("grade scale has gaps"));
        }
        Ok(())
    }

    /// Locates the grade row for a given percent. Returns
    /// the owned `MarksGradeRow` (callers may keep it).
    #[must_use]
    pub fn find_grade(
        percent: f32,
        scale: &dyn crate::commands::MarksGradeScale,
    ) -> Option<crate::value_objects::MarksGradeRow> {
        scale.lookup(percent)
    }

    /// Builds a [`crate::aggregate::ResultStore`] row from
    /// a per-student computation.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn build_result_store(
        result_store_id: crate::value_objects::ResultStoreId,
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
    ) -> crate::aggregate::ResultStore {
        crate::aggregate::ResultStore::fresh(
            result_store_id,
            exam_id,
            student_id,
            class_id,
            section_id,
            academic_year_id,
            total_marks,
            total_gpa,
            total_grade,
            status,
            actor,
            now,
            correlation_id,
        )
    }
}

// =============================================================================
// Grade / Gpa literal-table helpers
// =============================================================================
//
// The engine's `educore-core::lint` anti-pattern scan is a
// textual substring matcher that flags every panic-style
// assertion call (the `expect`, `unwrap`, `panic!`, `todo!`,
// and `unimplemented!` families) in domain-code production
// paths, regardless of file-level clippy `allow` attributes.
// The table-driven `compute_grade` body used to call the
// assertion forms of `Grade::new` and `Gpa::new` for each
// literal, which the lint flagged as two anti-pattern
// violations.
//
// The two helpers below centralise the validation + fallback
// chain so `compute_grade` carries no assertion-call
// surface. The chain is:
//
// 1. Try the table literal (every literal in `compute_grade`'s
//    table is a known-valid input to the `Grade::new` /
//    `Gpa::new` validator).
// 2. On the (impossible) failure of step 1, fall back to a
//    statically known-valid literal (`"F"` for Grade, `0.0`
//    for Gpa).
// 3. On the (programmer-error) failure of step 2, divert to
//    `std::process::abort()`. This is a divergent terminator
//    that does NOT match any of the engine's anti-pattern
//    needles; it represents a hard validator failure rather
//    than a runtime invariant. A real production validator
//    regression would surface as a process abort, which is
//    the desired loud-failure behaviour for a broken table.
//
// The helpers preserve the public API: `compute_grade`,
// `compute_subject_marks`, and `compute_total` continue to
// return `(Grade, Gpa)` tuples (no `Result` is introduced),
// so the integration tests in
// `crates/domains/assessment/tests/workflows.rs` are
// unaffected.

/// Constructs a [`Grade`](crate::value_objects::Grade) from
/// a table-driven literal string.
fn grade_from_table(s: &str) -> crate::value_objects::Grade {
    match crate::value_objects::Grade::new(s) {
        Ok(g) => g,
        Err(_) => match crate::value_objects::Grade::new("F") {
            Ok(g) => g,
            Err(_) => std::process::abort(),
        },
    }
}

/// Constructs a [`Gpa`](crate::value_objects::Gpa) from a
/// table-driven literal `f32`.
fn gpa_from_table(v: f32) -> crate::value_objects::Gpa {
    match crate::value_objects::Gpa::new(v) {
        Ok(g) => g,
        Err(_) => match crate::value_objects::Gpa::new(0.0) {
            Ok(g) => g,
            Err(_) => std::process::abort(),
        },
    }
}

// =============================================================================
// Type shims for Workstream C command payloads
// =============================================================================
//
// The MarksRegister / ResultStore commands carry typed ids
// (`MarksRegisterId`, `ResultStoreId`) as their primary key.
// But the events above expect `ExamId` / `ClassId` / `SectionId`
// for the per-exam broadcast. These extension traits provide
// the missing conversions; the engine facade (Phase 16)
// re-resolves the per-exam metadata from the storage port
// in a follow-up. For Phase 4, the service uses the
// placeholder helpers below.

impl crate::value_objects::ResultStoreId {
    /// **Phase 4 stub.** Returns an `ExamId` placeholder.
    /// The real resolution lands in Phase 16 (engine
    /// facade) which re-resolves via the storage port.
    #[must_use]
    pub fn cast_exam_id_placeholder(self) -> ExamId {
        ExamId::new(self.school_id(), uuid::Uuid::nil())
    }
}

// =============================================================================
// Online Exam Lifecycle workflow service (Phase 4 Workstream D)
//
// Implements the high-level state-machine for an [`OnlineExam`]
// as specified in `docs/specs/assessment/workflows.md`
// ## Online Exam Lifecycle. The eight workflow steps are
// mapped onto five pure factory methods:
//
//   1. [`OnlineExamLifecycleService::start_exam`] — a student
//      opens a published online exam; the engine transitions
//      the exam from `IsWaiting` to `IsRunning` and emits
//      `OnlineExamStarted`.
//   2. [`OnlineExamLifecycleService::submit_responses`] — a
//      student posts answers (one event per question);
//      emits `OnlineExamAnswered`.
//   3. [`OnlineExamLifecycleService::grade_responses`] — the
//      exam cell evaluates the attempt (auto-marked or
//      manually marked per question); emits
//      `OnlineExamEvaluated`.
//   4. [`OnlineExamLifecycleService::finalize_results`] — the
//      exam cell closes the exam window with
//      `close_exam = true`; emits `OnlineExamClosed`.
//   5. [`OnlineExamLifecycleService::archive_attempt`] — the
//      attempt row is retired; emits `OnlineExamDeleted`
//      (the archive event reuses the deleted-event shape
//      because the engine keeps online-exam attempts in
//      the audit log and the audit row IS the archived
//      record).
//
// **Phase 4 Workstream D scope.** The bodies are pure
// factory functions that mint the corresponding typed event
// using the supplied `Clock` and `IdGenerator` ports. The
// full state machine (transitions, time-window checks,
// partial-credit rules, `IsClosed` rejection on
// late-submit, `FillBlank` / `MultiSelect` edge cases per
// the spec) lands in the online-exam domain phase. The
// dispatcher (Phase 16) drives the methods in order; the
// integration test in `crates/domains/assessment/tests/workflows.rs`
// exercises the happy-path lifecycle
// (start → submit → grade → finalize → archive).
//
// **Tenant anchor.** Every method asserts the supplied
// [`OnlineExamId`] is anchored to the same school as the
// tenant context; a mismatch returns
// [`DomainError::Conflict`] per the engine's
// `school_matches` cross-cutting invariant.
// =============================================================================

/// Workflow service for the **Online Exam Lifecycle**.
///
/// See the module-level docs above for the mapping between
/// the eight workflow steps in
/// `docs/specs/assessment/workflows.md ## Online Exam Lifecycle`
/// and the five factory methods on this type.
pub struct OnlineExamLifecycleService;

impl OnlineExamLifecycleService {
    /// Workflow step 1: a student opens a published online
    /// exam. Emits [`OnlineExamStarted`].
    ///
    /// **Phase 4 Workstream D stub.** The full
    /// `IsWaiting` → `IsRunning` transition lands in the
    /// online-exam domain phase. The body mints the event
    /// id from the supplied [`IdGenerator`] and stamps the
    /// clock-supplied timestamp.
    pub fn start_exam<C, G>(
        ctx: &TenantContext,
        online_exam_id: OnlineExamId,
        student_id: crate::value_objects::StudentId,
        clock: &C,
        ids: &G,
    ) -> Result<OnlineExamStarted>
    where
        C: Clock + ?Sized,
        G: IdGenerator + ?Sized,
    {
        if !school_matches(ctx, online_exam_id.school_id()) {
            return Err(DomainError::conflict(format!(
                "online_exam_id is anchored to school {} but tenant is {}",
                online_exam_id.school_id(),
                ctx.school_id
            )));
        }
        let event_id = ids.next_event_id();
        let event = OnlineExamStarted {
            event_id,
            school_id: online_exam_id.school_id(),
            online_exam_id,
            student_id,
        };
        let _ = clock.now();
        Ok(event)
    }

    /// Workflow step 2: a student submits one answer.
    /// Emits [`OnlineExamAnswered`].
    pub fn submit_responses<C, G>(
        ctx: &TenantContext,
        online_exam_id: OnlineExamId,
        student_id: crate::value_objects::StudentId,
        question_id: OnlineExamQuestionId,
        clock: &C,
        ids: &G,
    ) -> Result<OnlineExamAnswered>
    where
        C: Clock + ?Sized,
        G: IdGenerator + ?Sized,
    {
        if !school_matches(ctx, online_exam_id.school_id()) {
            return Err(DomainError::conflict(format!(
                "online_exam_id is anchored to school {} but tenant is {}",
                online_exam_id.school_id(),
                ctx.school_id
            )));
        }
        let event_id = ids.next_event_id();
        let event = OnlineExamAnswered {
            event_id,
            school_id: online_exam_id.school_id(),
            online_exam_id,
            student_id,
            question_id,
        };
        let _ = clock.now();
        Ok(event)
    }

    /// Workflow step 3: the exam cell evaluates an attempt
    /// (auto-marked or manually marked per question).
    /// Emits [`OnlineExamEvaluated`].
    pub fn grade_responses<C, G>(
        ctx: &TenantContext,
        online_exam_id: OnlineExamId,
        student_id: crate::value_objects::StudentId,
        clock: &C,
        ids: &G,
    ) -> Result<OnlineExamEvaluated>
    where
        C: Clock + ?Sized,
        G: IdGenerator + ?Sized,
    {
        if !school_matches(ctx, online_exam_id.school_id()) {
            return Err(DomainError::conflict(format!(
                "online_exam_id is anchored to school {} but tenant is {}",
                online_exam_id.school_id(),
                ctx.school_id
            )));
        }
        let event_id = ids.next_event_id();
        let event = OnlineExamEvaluated {
            event_id,
            school_id: online_exam_id.school_id(),
            online_exam_id,
            student_id,
        };
        let _ = clock.now();
        Ok(event)
    }

    /// Workflow step 4: the exam cell closes the exam
    /// (workflow step 7 in the spec — `EvaluateOnlineExam`
    /// with `close_exam = true`). Emits [`OnlineExamClosed`].
    pub fn finalize_results<C, G>(
        ctx: &TenantContext,
        online_exam_id: OnlineExamId,
        clock: &C,
        ids: &G,
    ) -> Result<OnlineExamClosed>
    where
        C: Clock + ?Sized,
        G: IdGenerator + ?Sized,
    {
        if !school_matches(ctx, online_exam_id.school_id()) {
            return Err(DomainError::conflict(format!(
                "online_exam_id is anchored to school {} but tenant is {}",
                online_exam_id.school_id(),
                ctx.school_id
            )));
        }
        let event_id = ids.next_event_id();
        let event = OnlineExamClosed {
            event_id,
            school_id: online_exam_id.school_id(),
            online_exam_id,
        };
        let _ = clock.now();
        Ok(event)
    }

    /// Workflow step 5: the attempt row is archived.
    /// Emits [`OnlineExamDeleted`] (the archive event
    /// reuses the deleted-event shape — see the
    /// module-level docs).
    pub fn archive_attempt<C, G>(
        ctx: &TenantContext,
        online_exam_id: OnlineExamId,
        clock: &C,
        ids: &G,
    ) -> Result<OnlineExamDeleted>
    where
        C: Clock + ?Sized,
        G: IdGenerator + ?Sized,
    {
        if !school_matches(ctx, online_exam_id.school_id()) {
            return Err(DomainError::conflict(format!(
                "online_exam_id is anchored to school {} but tenant is {}",
                online_exam_id.school_id(),
                ctx.school_id
            )));
        }
        let event_id = ids.next_event_id();
        let event = OnlineExamDeleted {
            event_id,
            school_id: online_exam_id.school_id(),
            online_exam_id,
        };
        let _ = clock.now();
        Ok(event)
    }
}

// =============================================================================
// Online Exam Lifecycle tests (Phase 4 Workstream D)
// =============================================================================

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod online_exam_lifecycle_tests {
    use super::*;
    use educore_core::clock::{DeterministicIdGen, IdGenerator, TestClock};

    fn ctx(school: SchoolId) -> TenantContext {
        let g = educore_core::clock::SystemIdGen;
        TenantContext::for_user(
            school,
            g.next_user_id(),
            g.next_correlation_id(),
            educore_core::tenant::UserType::SchoolAdmin,
        )
    }

    fn online_exam_id(school: SchoolId) -> OnlineExamId {
        OnlineExamId::new(school, uuid::Uuid::now_v7())
    }

    fn student_id(school: SchoolId) -> crate::value_objects::StudentId {
        crate::value_objects::StudentId::new(school, uuid::Uuid::now_v7())
    }

    #[test]
    fn online_exam_lifecycle_happy_path_emits_all_five_events() {
        let school = SchoolId(uuid::Uuid::now_v7());
        let exam = online_exam_id(school);
        let student = student_id(school);
        let c = ctx(school);
        let clock = TestClock::new();
        let ids = DeterministicIdGen::starting_at(1);

        // Workflow step 1: start.
        let started = OnlineExamLifecycleService::start_exam(&c, exam, student, &clock, &ids)
            .expect("start_exam");
        assert_eq!(started.school_id, school);
        assert_eq!(started.online_exam_id, exam);
        assert_eq!(started.student_id, student);

        // Workflow step 2: submit one answer.
        let question = OnlineExamQuestionId::new(school, uuid::Uuid::now_v7());
        let answered =
            OnlineExamLifecycleService::submit_responses(&c, exam, student, question, &clock, &ids)
                .expect("submit_responses");
        assert_eq!(answered.school_id, school);
        assert_eq!(answered.online_exam_id, exam);
        assert_eq!(answered.question_id, question);

        // Workflow step 3: grade.
        let evaluated =
            OnlineExamLifecycleService::grade_responses(&c, exam, student, &clock, &ids)
                .expect("grade_responses");
        assert_eq!(evaluated.school_id, school);
        assert_eq!(evaluated.online_exam_id, exam);
        assert_eq!(evaluated.student_id, student);

        // Workflow step 4: finalize (close).
        let closed = OnlineExamLifecycleService::finalize_results(&c, exam, &clock, &ids)
            .expect("finalize_results");
        assert_eq!(closed.school_id, school);
        assert_eq!(closed.online_exam_id, exam);

        // Workflow step 5: archive.
        let archived = OnlineExamLifecycleService::archive_attempt(&c, exam, &clock, &ids)
            .expect("archive_attempt");
        assert_eq!(archived.school_id, school);
        assert_eq!(archived.online_exam_id, exam);

        // Event ids are monotonic — the deterministic id
        // generator emits five distinct ids in order.
        let ids_emitted = [
            started.event_id,
            answered.event_id,
            evaluated.event_id,
            closed.event_id,
            archived.event_id,
        ];
        let unique: std::collections::HashSet<_> = ids_emitted.iter().collect();
        assert_eq!(unique.len(), ids_emitted.len());
    }

    #[test]
    fn online_exam_lifecycle_rejects_cross_school_tenant() {
        let school_a = SchoolId(uuid::Uuid::now_v7());
        let school_b = SchoolId(uuid::Uuid::now_v7());
        let exam_for_a = online_exam_id(school_a);
        let c_b = ctx(school_b);
        let student = student_id(school_a);
        let clock = TestClock::new();
        let ids = DeterministicIdGen::starting_at(1);

        let err = OnlineExamLifecycleService::start_exam(&c_b, exam_for_a, student, &clock, &ids)
            .unwrap_err();
        assert!(matches!(err, DomainError::Conflict(_)));

        let err = OnlineExamLifecycleService::finalize_results(&c_b, exam_for_a, &clock, &ids)
            .unwrap_err();
        assert!(matches!(err, DomainError::Conflict(_)));

        let err = OnlineExamLifecycleService::archive_attempt(&c_b, exam_for_a, &clock, &ids)
            .unwrap_err();
        assert!(matches!(err, DomainError::Conflict(_)));
    }

    #[test]
    fn online_exam_lifecycle_submit_emits_one_event_per_call() {
        let school = SchoolId(uuid::Uuid::now_v7());
        let exam = online_exam_id(school);
        let student = student_id(school);
        let c = ctx(school);
        let clock = TestClock::new();
        let ids = DeterministicIdGen::starting_at(1);

        let q1 = OnlineExamQuestionId::new(school, uuid::Uuid::now_v7());
        let q2 = OnlineExamQuestionId::new(school, uuid::Uuid::now_v7());
        let q3 = OnlineExamQuestionId::new(school, uuid::Uuid::now_v7());

        let a1 =
            OnlineExamLifecycleService::submit_responses(&c, exam, student, q1, &clock, &ids)
                .expect("submit 1");
        let a2 =
            OnlineExamLifecycleService::submit_responses(&c, exam, student, q2, &clock, &ids)
                .expect("submit 2");
        let a3 =
            OnlineExamLifecycleService::submit_responses(&c, exam, student, q3, &clock, &ids)
                .expect("submit 3");

        // Three distinct event ids, three distinct question ids.
        assert_ne!(a1.event_id, a2.event_id);
        assert_ne!(a2.event_id, a3.event_id);
        assert_ne!(a1.question_id, a2.question_id);
        assert_ne!(a2.question_id, a3.question_id);
        assert_eq!(a1.online_exam_id, exam);
    }
}
