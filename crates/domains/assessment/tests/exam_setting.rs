//! Integration tests for the **ExamSetting aggregate** vertical slice.
//!
//! Pins the create command struct surface and the
//! `ExamSettingCreated` event struct for
//! [`ExamSetting`](educore_assessment::aggregate::ExamSetting).
//!
//! # Current contract (Wave 4 vertical slice)
//!
//! `create_exam_setting` in `services.rs` is a **stub**
//! (`DomainError::not_supported("TODO: create_exam_setting")`).
//! The full implementation lands in a follow-up phase. These
//! tests pin the **current** behaviour so the dispatcher /
//! facade work can rely on the error surface while the real
//! validation + aggregate construction is being built out.
//!
//! Mirrors `crates/domains/assessment/tests/marks_grade.rs`
//! (stub pattern).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_assessment::commands::CreateExamSettingCommand;
use educore_assessment::events::ExamSettingCreated;
use educore_assessment::services::create_exam_setting;
use educore_assessment::value_objects::ExamSettingId;
use educore_core::clock::{IdGenerator as _, SystemIdGen};
use educore_core::error::DomainError;
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
// Happy path: current contract — stub returns NotSupported
// =============================================================================

#[tokio::test]
async fn create_exam_setting_stub_returns_not_supported() {
    let (tenant, g) = admin_context();
    let cmd = CreateExamSettingCommand {
        school_id: tenant.school_id,
        exam_setting_id: ExamSettingId::new(tenant.school_id, g.next_uuid()),
    };
    let result = create_exam_setting(cmd).await;
    assert!(matches!(result, Err(DomainError::NotSupported(_))));
}

#[test]
fn exam_setting_created_event_carries_typed_id_and_school() {
    let (tenant, g) = admin_context();
    let setting_id = ExamSettingId::new(tenant.school_id, g.next_uuid());
    let event = ExamSettingCreated {
        event_id: g.next_event_id(),
        school_id: tenant.school_id,
        exam_setting_id: setting_id,
    };
    assert_eq!(event.school_id, tenant.school_id);
    assert_eq!(event.exam_setting_id, setting_id);
}
