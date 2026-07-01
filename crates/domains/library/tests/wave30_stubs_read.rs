//! Integration tests for the final Wave 30 stub replacements
//! in `crates/domains/library/src/services.rs` (batch 4 —
//! read-only query commands).
//!
//! Pins the contract for:
//!
//! - [`search_books`](educore_library::services::search_books)
//! - [`list_overdue_issues`](educore_library::services::list_overdue_issues)
//! - [`list_member_issues`](educore_library::services::list_member_issues)
//!
//! Each handler is a pure factory: it validates the command
//! invariants and returns `Ok(Vec::new())` because the actual
//! row fetching is the dispatcher's responsibility (per the
//! module docstring at `services.rs` § Phase 9). These tests
//! pin the validation contract — happy path + tenant-mismatch
//! + per-command validation failures — so the dispatcher can
//! safely rely on the factory before issuing the repository
//! query.
//!
//! Mirrors the pattern in `tests/wave30_stubs.rs`,
//! `tests/wave30_stubs_member.rs`, and
//! `tests/wave30_stubs_batch3.rs`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use chrono::NaiveDate;
use educore_core::clock::{IdGenerator as _, SystemIdGen, TestClock};
use educore_core::error::DomainError;
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_library::prelude::*;
use educore_library::services::{list_member_issues, list_overdue_issues, search_books};
use educore_library::value_objects::LibraryMemberId;

/// A fresh `TenantContext` for a `SchoolAdmin` acting on a
/// freshly-minted school.
fn admin() -> (TenantContext, SystemIdGen) {
    let g = SystemIdGen;
    let tenant = TenantContext::for_user(
        g.next_school_id(),
        g.next_user_id(),
        g.next_correlation_id(),
        UserType::SchoolAdmin,
    );
    (tenant, g)
}

/// Construct a `NaiveDate` from parts (test fixture only).
fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
}

// =============================================================================
// search_books
// =============================================================================

/// `search_books` validates the tenant anchor, the non-empty
/// query, and the limit bounds, then returns an empty `Vec`.
/// The empty `Vec` is correct because the factory does not
/// have access to the repository — the dispatcher does the
/// actual `BookRepository::search` call.
#[test]
fn search_books_validates_and_returns_empty_vec() {
    let (tenant, _g) = admin();
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = SearchBooksCommand {
        tenant: tenant.clone(),
        school_id: tenant.school_id,
        query: "rust programming".to_owned(),
        category: None,
        limit: 50,
    };
    let books = search_books(cmd, &clock, &ids).expect("search should succeed");

    assert!(
        books.is_empty(),
        "factory should return empty vec; dispatcher fills it"
    );
}

/// `search_books` rejects a command whose tenant school
/// does not match the command anchor with
/// `DomainError::TenantViolation`.
#[test]
fn search_books_tenant_mismatch_returns_tenant_violation() {
    let (tenant, _g) = admin();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let other_g = SystemIdGen;

    let cmd = SearchBooksCommand {
        tenant: TenantContext::for_user(
            other_g.next_school_id(),
            other_g.next_user_id(),
            other_g.next_correlation_id(),
            UserType::SchoolAdmin,
        ),
        school_id: tenant.school_id,
        query: "anything".to_owned(),
        category: None,
        limit: 50,
    };
    let err = search_books(cmd, &clock, &ids).expect_err("tenant mismatch must fail");
    assert!(
        matches!(err, DomainError::TenantViolation(_)),
        "expected TenantViolation, got {err:?}"
    );
}

/// `search_books` rejects an empty or whitespace-only query
/// with `DomainError::Validation`.
#[test]
fn search_books_empty_query_returns_validation_error() {
    let (tenant, _g) = admin();
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = SearchBooksCommand {
        tenant: tenant.clone(),
        school_id: tenant.school_id,
        query: "   ".to_owned(),
        category: None,
        limit: 50,
    };
    let err = search_books(cmd, &clock, &ids).expect_err("empty query must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

/// `search_books` rejects a zero or oversized limit with
/// `DomainError::Validation`.
#[test]
fn search_books_out_of_bounds_limit_returns_validation_error() {
    let (tenant, _g) = admin();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let school = tenant.school_id;

    let cmd_zero = SearchBooksCommand {
        tenant: tenant.clone(),
        school_id: school,
        query: "x".to_owned(),
        category: None,
        limit: 0,
    };
    let err = search_books(cmd_zero, &clock, &ids).expect_err("zero limit must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation for zero limit, got {err:?}"
    );

    let cmd_oversized = SearchBooksCommand {
        tenant: tenant.clone(),
        school_id: school,
        query: "x".to_owned(),
        category: None,
        limit: 10_000,
    };
    let err = search_books(cmd_oversized, &clock, &ids).expect_err("oversized limit must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation for oversized limit, got {err:?}"
    );
}

// =============================================================================
// list_overdue_issues
// =============================================================================

/// `list_overdue_issues` validates the tenant anchor and
/// the as_of horizon, then returns an empty `Vec`. The
/// factory does not fetch rows — the dispatcher does.
#[test]
fn list_overdue_issues_validates_and_returns_empty_vec() {
    let (tenant, _g) = admin();
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // TestClock defaults to the unix epoch (1970-01-01), so
    // pick an as_of within the 366-day horizon of the epoch.
    let cmd = ListOverdueIssuesCommand {
        tenant: tenant.clone(),
        school_id: tenant.school_id,
        as_of: date(1970, 6, 1),
    };
    let issues = list_overdue_issues(cmd, &clock, &ids).expect("list should succeed");

    assert!(
        issues.is_empty(),
        "factory should return empty vec; dispatcher fills it"
    );
}

/// `list_overdue_issues` rejects a command whose tenant
/// school does not match the command anchor with
/// `DomainError::TenantViolation`.
#[test]
fn list_overdue_issues_tenant_mismatch_returns_tenant_violation() {
    let (tenant, _g) = admin();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let other_g = SystemIdGen;

    let cmd = ListOverdueIssuesCommand {
        tenant: TenantContext::for_user(
            other_g.next_school_id(),
            other_g.next_user_id(),
            other_g.next_correlation_id(),
            UserType::SchoolAdmin,
        ),
        school_id: tenant.school_id,
        as_of: date(1970, 6, 1),
    };
    let err = list_overdue_issues(cmd, &clock, &ids).expect_err("tenant mismatch must fail");
    assert!(
        matches!(err, DomainError::TenantViolation(_)),
        "expected TenantViolation, got {err:?}"
    );
}

/// `list_overdue_issues` rejects an `as_of` more than one
/// year in the future with `DomainError::Validation` (guard
/// against typos / runaway reports).
#[test]
fn list_overdue_issues_as_of_too_far_in_future_returns_validation_error() {
    let (tenant, _g) = admin();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    // TestClock default is the unix epoch, so anything in the
    // far future is well past the 366-day horizon.
    let cmd = ListOverdueIssuesCommand {
        tenant: tenant.clone(),
        school_id: tenant.school_id,
        as_of: date(2099, 1, 1),
    };
    let err = list_overdue_issues(cmd, &clock, &ids).expect_err("future as_of must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

// =============================================================================
// list_member_issues
// =============================================================================

/// `list_member_issues` validates that the
/// `library_member_id` belongs to the tenant's school, then
/// returns an empty `Vec`. The factory does not fetch rows
/// — the dispatcher does.
#[test]
fn list_member_issues_validates_and_returns_empty_vec() {
    let (tenant, g) = admin();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let member = LibraryMemberId::new(tenant.school_id, g.next_uuid());

    let cmd = ListMemberIssuesCommand {
        tenant: tenant.clone(),
        library_member_id: member,
    };
    let issues = list_member_issues(cmd, &clock, &ids).expect("list should succeed");

    assert!(
        issues.is_empty(),
        "factory should return empty vec; dispatcher fills it"
    );
}

/// `list_member_issues` rejects a command whose
/// `library_member_id` belongs to a different school with
/// `DomainError::TenantViolation`.
#[test]
fn list_member_issues_cross_school_member_returns_tenant_violation() {
    let (tenant, g) = admin();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let other_g = SystemIdGen;
    let other_school: SchoolId = other_g.next_school_id();
    // Member belongs to a different school than the tenant.
    let cross_member = LibraryMemberId::new(other_school, g.next_uuid());

    let cmd = ListMemberIssuesCommand {
        tenant: tenant.clone(),
        library_member_id: cross_member,
    };
    let err = list_member_issues(cmd, &clock, &ids).expect_err("cross-school member must fail");
    assert!(
        matches!(err, DomainError::TenantViolation(_)),
        "expected TenantViolation, got {err:?}"
    );
}
