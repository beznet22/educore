//! Integration tests for the **OnlineExam aggregate** vertical slice.
//!
//! Pins the create contract for
//! [`OnlineExam`](educore_assessment::aggregate::OnlineExam)
//! end-to-end through the service layer.
//!
//! # Current contract (Wave 4 vertical slice)
//!
//! `create_online_exam` and `publish_online_exam` in
//! `services.rs` are **stubs** that unconditionally return
//! `DomainError::not_supported("TODO: ...")` before any
//! aggregate construction or event minting happens. The full
//! implementation lands in a follow-up phase. These tests
//! pin the **current** behaviour so the dispatcher / facade
//! work can rely on the error surface while the real
//! validation + lifecycle is being built out:
//!
//! 1. Happy path — any well-formed input is rejected with
//!    `DomainError::NotSupported`. No aggregate is built, no
//!    event is emitted.
//! 2. Validation-failure path — the stub does not validate
//!    its input, so any payload (including one that would
//!    fail the future spec invariants on
//!    `Status`/`IsTaken`/`IsClosed`/`IsWaiting`/`IsRunning`/
//!    `AutoMark`/`StartTime`/`EndTime`/`EndDateTime`) is
//!    rejected with the same `NotSupported` error before
//!    any field-level check runs.
//!
//! Once the real handler lands, the happy-path test will be
//! updated to assert `OnlineExamCreated` with `version == 1`
//! per the spec invariant
//! (`start < end <= end_date_time`); the validation-failure
//! test will then assert `DomainError::Validation` directly.
//!
//! Mirrors `crates/domains/assessment/tests/marks_grade.rs`
//! (lean — stub contract pin).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_assessment::commands::CreateOnlineExamCommand;
use educore_assessment::services::{create_online_exam, publish_online_exam};
use educore_assessment::value_objects::OnlineExamId;
use educore_core::clock::{IdGenerator, SystemIdGen};
use educore_core::error::DomainError;
use educore_core::tenant::{TenantContext, UserType};

// =============================================================================
// Fixtures
// =============================================================================

/// A fresh `TenantContext` for a `SchoolAdmin` acting on a
/// freshly-minted school. Returns the context plus the
/// generator so tests can mint child ids from the same
/// school.
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

/// Mint an `OnlineExamId` for the given school.
fn online_exam_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> OnlineExamId {
    OnlineExamId::new(school, g.next_uuid())
}

// =============================================================================
// Happy path: current contract — stub returns NotSupported
// =============================================================================

/// Pins the **current** contract of `create_online_exam` for
/// a well-formed payload. The handler is currently a stub
/// that returns `DomainError::NotSupported("TODO:
/// create_online_exam")` before any aggregate construction
/// or event minting happens. Once the real implementation
/// lands (carrying `Status`, `IsTaken`, `IsClosed`,
/// `IsWaiting`, `IsRunning`, `AutoMark`, `StartTime`,
/// `EndTime`, `EndDateTime` per
/// `docs/specs/assessment/aggregates.md` § OnlineExam), this
/// test will be updated to assert that:
///
/// - The returned event is `OnlineExamCreated` with
///   `version == 1`,
/// - The aggregate is school-scoped and active,
/// - `start < end <= end_date_time` is enforced.
#[tokio::test]
async fn online_exam_create_currently_returns_not_supported() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;

    let cmd = CreateOnlineExamCommand {
        school_id: school,
        online_exam_id: online_exam_id(&g, school),
    };

    let err = create_online_exam(cmd)
        .await
        .expect_err("create_online_exam is currently a stub");
    assert!(
        matches!(err, DomainError::NotSupported(_)),
        "expected NotSupported (current stub contract), got {err:?}"
    );
}

// =============================================================================
// Update path: current contract — stub returns NotSupported
// =============================================================================

/// Pins the **current** contract of `publish_online_exam` —
/// the natural "update" / state-transition handler on the
/// `OnlineExam` aggregate. The handler is currently a stub
/// that returns `DomainError::NotSupported("TODO:
/// publish_online_exam")` before any state transition or
/// event minting happens. Once the real implementation
/// lands, this test will be updated to assert that:
///
/// - The returned event is `OnlineExamCreated` (re-emitted
///   with the publish transition; see events.rs contract),
/// - The aggregate's `IsTaken` flag flips to `true` and
///   `version` increments.
#[tokio::test]
async fn online_exam_publish_currently_returns_not_supported() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;

    let cmd = educore_assessment::commands::PublishOnlineExamCommand {
        school_id: school,
        online_exam_id: online_exam_id(&g, school),
    };

    let err = publish_online_exam(cmd)
        .await
        .expect_err("publish_online_exam is currently a stub");
    assert!(
        matches!(err, DomainError::NotSupported(_)),
        "expected NotSupported (current stub contract), got {err:?}"
    );
}
