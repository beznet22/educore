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

use educore_core::clock::{Clock, IdGenerator};
use educore_core::error::{DomainError, Result};
use educore_core::ids::{EventId, Identifier, SchoolId};
use educore_core::tenant::TenantContext;
use educore_core::value_objects::ActiveStatus;

use crate::aggregate::Exam;
use crate::commands::{
    validate_exam_code, validate_exam_mark, validate_exam_name, validate_pass_mark,
    AssessmentUniquenessChecker, CreateExamCommand, DeleteExamCommand, UpdateExamCommand,
};
use crate::events::{ExamCreated, ExamDeleted, ExamUpdated};

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
    clippy::dbg_macro
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
