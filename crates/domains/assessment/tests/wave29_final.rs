//! Integration tests for **Wave 29 final** — the last 9
//! `DomainError::not_supported` stubs in `services.rs`
//! replaced with real, spec-compliant implementations:
//!
//! - `start_online_exam`
//! - `submit_online_exam_answer`
//! - `evaluate_online_exam`
//! - `delete_question_group`
//! - `configure_admit_card_settings`
//! - `approve_teacher_evaluation`
//! - `reject_teacher_evaluation`
//! - `configure_custom_result_settings`
//! - `mark_exam_step_skip`
//!
//! Each handler now enforces the typed-id school-anchoring
//! invariant (the id's `school_id` must equal the
//! command's `school_id`; cross-tenant references are
//! rejected with `DomainError::Validation`) and emits the
//! spec-defined `*Created` event stamped with a fresh
//! `event_id` (UUIDv7) and the command's school anchor.
//!
//! Mirrors `crates/domains/assessment/tests/question_bank.rs`
//! (lean — real-handler contract pin).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_assessment::commands::{
    ApproveTeacherEvaluationCommand, ConfigureAdmitCardSettingsCommand,
    ConfigureCustomResultSettingsCommand, DeleteQuestionGroupCommand, EvaluateOnlineExamCommand,
    MarkExamStepSkipCommand, RejectTeacherEvaluationCommand, StartOnlineExamCommand,
    SubmitOnlineExamAnswerCommand,
};
use educore_assessment::services::{
    approve_teacher_evaluation, configure_admit_card_settings, configure_custom_result_settings,
    delete_question_group, evaluate_online_exam, mark_exam_step_skip, reject_teacher_evaluation,
    start_online_exam, submit_online_exam_answer,
};
use educore_assessment::value_objects::{
    AdmitCardSettingId, CustomResultSettingId, ExamStepSkipId, OnlineExamId, QuestionGroupId,
    StudentTakeOnlineExamId, TeacherEvaluationId,
};
use educore_core::clock::{IdGenerator as _, SystemIdGen};
use educore_core::error::DomainError;
use educore_core::ids::{Identifier as _, SchoolId};
use educore_core::tenant::{TenantContext, UserType};

// =============================================================================
// Fixtures
// =============================================================================

fn admin_context() -> (TenantContext, SystemIdGen) {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    (
        TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin),
        g,
    )
}

// =============================================================================
// start_online_exam
// =============================================================================

#[tokio::test]
async fn start_online_exam_happy_path() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;

    let id = OnlineExamId::new(school, g.next_uuid());
    let cmd = StartOnlineExamCommand {
        school_id: school,
        online_exam_id: id,
    };

    let event = start_online_exam(cmd)
        .await
        .expect("real handler accepts well-formed input");
    assert_eq!(event.school_id, school);
    let _: StudentTakeOnlineExamId = event.student_take_online_exam_id;
    let _: uuid::Uuid = event.event_id.as_uuid();
}

#[tokio::test]
async fn start_online_exam_cross_tenant_rejected() {
    let (tenant, g) = admin_context();
    let actor_school = tenant.school_id;
    let other_school = g.next_school_id();

    let cmd = StartOnlineExamCommand {
        school_id: actor_school,
        online_exam_id: OnlineExamId::new(other_school, g.next_uuid()),
    };

    let err = start_online_exam(cmd)
        .await
        .expect_err("cross-tenant id must be rejected");
    assert!(matches!(err, DomainError::Validation(_)));
}

// =============================================================================
// submit_online_exam_answer
// =============================================================================

#[tokio::test]
async fn submit_online_exam_answer_happy_path() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;

    let id = OnlineExamId::new(school, g.next_uuid());
    let cmd = SubmitOnlineExamAnswerCommand {
        school_id: school,
        online_exam_id: id,
    };

    let event = submit_online_exam_answer(cmd)
        .await
        .expect("real handler accepts well-formed input");
    assert_eq!(event.school_id, school);
    let _: StudentTakeOnlineExamId = event.student_take_online_exam_id;
}

#[tokio::test]
async fn submit_online_exam_answer_cross_tenant_rejected() {
    let (tenant, g) = admin_context();
    let actor_school = tenant.school_id;
    let other_school = g.next_school_id();

    let cmd = SubmitOnlineExamAnswerCommand {
        school_id: actor_school,
        online_exam_id: OnlineExamId::new(other_school, g.next_uuid()),
    };

    let err = submit_online_exam_answer(cmd)
        .await
        .expect_err("cross-tenant id must be rejected");
    assert!(matches!(err, DomainError::Validation(_)));
}

// =============================================================================
// evaluate_online_exam
// =============================================================================

#[tokio::test]
async fn evaluate_online_exam_happy_path() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;

    let id = OnlineExamId::new(school, g.next_uuid());
    let cmd = EvaluateOnlineExamCommand {
        school_id: school,
        online_exam_id: id,
    };

    let event = evaluate_online_exam(cmd)
        .await
        .expect("real handler accepts well-formed input");
    assert_eq!(event.school_id, school);
    let _: StudentTakeOnlineExamId = event.student_take_online_exam_id;
}

#[tokio::test]
async fn evaluate_online_exam_cross_tenant_rejected() {
    let (tenant, g) = admin_context();
    let actor_school = tenant.school_id;
    let other_school = g.next_school_id();

    let cmd = EvaluateOnlineExamCommand {
        school_id: actor_school,
        online_exam_id: OnlineExamId::new(other_school, g.next_uuid()),
    };

    let err = evaluate_online_exam(cmd)
        .await
        .expect_err("cross-tenant id must be rejected");
    assert!(matches!(err, DomainError::Validation(_)));
}

// =============================================================================
// delete_question_group
// =============================================================================

#[tokio::test]
async fn delete_question_group_happy_path() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;

    let id = QuestionGroupId::new(school, g.next_uuid());
    let cmd = DeleteQuestionGroupCommand {
        school_id: school,
        question_group_id: id,
    };

    let event = delete_question_group(cmd)
        .await
        .expect("real handler accepts well-formed input");
    assert_eq!(event.school_id, school);
    assert_eq!(event.question_group_id, id);
}

#[tokio::test]
async fn delete_question_group_cross_tenant_rejected() {
    let (tenant, g) = admin_context();
    let actor_school = tenant.school_id;
    let other_school = g.next_school_id();

    let cmd = DeleteQuestionGroupCommand {
        school_id: actor_school,
        question_group_id: QuestionGroupId::new(other_school, g.next_uuid()),
    };

    let err = delete_question_group(cmd)
        .await
        .expect_err("cross-tenant id must be rejected");
    assert!(matches!(err, DomainError::Validation(_)));
}

// =============================================================================
// configure_admit_card_settings
// =============================================================================

#[tokio::test]
async fn configure_admit_card_settings_happy_path() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;

    let id = AdmitCardSettingId::new(school, g.next_uuid());
    let cmd = ConfigureAdmitCardSettingsCommand {
        school_id: school,
        admit_card_setting_id: id,
    };

    let event = configure_admit_card_settings(cmd)
        .await
        .expect("real handler accepts well-formed input");
    assert_eq!(event.school_id, school);
    assert_eq!(event.admit_card_setting_id, id);
}

#[tokio::test]
async fn configure_admit_card_settings_cross_tenant_rejected() {
    let (tenant, g) = admin_context();
    let actor_school = tenant.school_id;
    let other_school = g.next_school_id();

    let cmd = ConfigureAdmitCardSettingsCommand {
        school_id: actor_school,
        admit_card_setting_id: AdmitCardSettingId::new(other_school, g.next_uuid()),
    };

    let err = configure_admit_card_settings(cmd)
        .await
        .expect_err("cross-tenant id must be rejected");
    assert!(matches!(err, DomainError::Validation(_)));
}

// =============================================================================
// approve_teacher_evaluation
// =============================================================================

#[tokio::test]
async fn approve_teacher_evaluation_happy_path() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;

    let id = TeacherEvaluationId::new(school, g.next_uuid());
    let cmd = ApproveTeacherEvaluationCommand {
        school_id: school,
        teacher_evaluation_id: id,
    };

    let event = approve_teacher_evaluation(cmd)
        .await
        .expect("real handler accepts well-formed input");
    assert_eq!(event.school_id, school);
    assert_eq!(event.teacher_evaluation_id, id);
}

#[tokio::test]
async fn approve_teacher_evaluation_cross_tenant_rejected() {
    let (tenant, g) = admin_context();
    let actor_school = tenant.school_id;
    let other_school = g.next_school_id();

    let cmd = ApproveTeacherEvaluationCommand {
        school_id: actor_school,
        teacher_evaluation_id: TeacherEvaluationId::new(other_school, g.next_uuid()),
    };

    let err = approve_teacher_evaluation(cmd)
        .await
        .expect_err("cross-tenant id must be rejected");
    assert!(matches!(err, DomainError::Validation(_)));
}

// =============================================================================
// reject_teacher_evaluation
// =============================================================================

#[tokio::test]
async fn reject_teacher_evaluation_happy_path() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;

    let id = TeacherEvaluationId::new(school, g.next_uuid());
    let cmd = RejectTeacherEvaluationCommand {
        school_id: school,
        teacher_evaluation_id: id,
    };

    let event = reject_teacher_evaluation(cmd)
        .await
        .expect("real handler accepts well-formed input");
    assert_eq!(event.school_id, school);
    assert_eq!(event.teacher_evaluation_id, id);
}

#[tokio::test]
async fn reject_teacher_evaluation_cross_tenant_rejected() {
    let (tenant, g) = admin_context();
    let actor_school = tenant.school_id;
    let other_school = g.next_school_id();

    let cmd = RejectTeacherEvaluationCommand {
        school_id: actor_school,
        teacher_evaluation_id: TeacherEvaluationId::new(other_school, g.next_uuid()),
    };

    let err = reject_teacher_evaluation(cmd)
        .await
        .expect_err("cross-tenant id must be rejected");
    assert!(matches!(err, DomainError::Validation(_)));
}

// =============================================================================
// configure_custom_result_settings
// =============================================================================

#[tokio::test]
async fn configure_custom_result_settings_happy_path() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;

    let id = CustomResultSettingId::new(school, g.next_uuid());
    let cmd = ConfigureCustomResultSettingsCommand {
        school_id: school,
        custom_result_setting_id: id,
    };

    let event = configure_custom_result_settings(cmd)
        .await
        .expect("real handler accepts well-formed input");
    assert_eq!(event.school_id, school);
    assert_eq!(event.custom_result_setting_id, id);
}

#[tokio::test]
async fn configure_custom_result_settings_cross_tenant_rejected() {
    let (tenant, g) = admin_context();
    let actor_school = tenant.school_id;
    let other_school = g.next_school_id();

    let cmd = ConfigureCustomResultSettingsCommand {
        school_id: actor_school,
        custom_result_setting_id: CustomResultSettingId::new(other_school, g.next_uuid()),
    };

    let err = configure_custom_result_settings(cmd)
        .await
        .expect_err("cross-tenant id must be rejected");
    assert!(matches!(err, DomainError::Validation(_)));
}

// =============================================================================
// mark_exam_step_skip
// =============================================================================

#[tokio::test]
async fn mark_exam_step_skip_happy_path() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;

    let id = ExamStepSkipId::new(school, g.next_uuid());
    let cmd = MarkExamStepSkipCommand {
        school_id: school,
        exam_step_skip_id: id,
    };

    let event = mark_exam_step_skip(cmd)
        .await
        .expect("real handler accepts well-formed input");
    assert_eq!(event.school_id, school);
    assert_eq!(event.exam_step_skip_id, id);
}

#[tokio::test]
async fn mark_exam_step_skip_cross_tenant_rejected() {
    let (tenant, g) = admin_context();
    let actor_school = tenant.school_id;
    let other_school = g.next_school_id();

    let cmd = MarkExamStepSkipCommand {
        school_id: actor_school,
        exam_step_skip_id: ExamStepSkipId::new(other_school, g.next_uuid()),
    };

    let err = mark_exam_step_skip(cmd)
        .await
        .expect_err("cross-tenant id must be rejected");
    assert!(matches!(err, DomainError::Validation(_)));
}
