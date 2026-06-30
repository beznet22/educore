//! Integration tests for the **ItemIssue aggregate** vertical slice.
//!
//! Pins the create + return-issued-item contract for
//! [`ItemIssue`](educore_facilities::aggregate::ItemIssue)
//! end-to-end through the service layer:
//!
//! 1. `issue_item` validates the input (rejects zero
//!    quantities), constructs the aggregate (with an initial
//!    `Issued` status and zero `returned_quantity`), and
//!    emits an [`ItemIssued`] event.
//! 2. `return_issued_item` mutates the in-place aggregate
//!    (bumps `returned_quantity`, transitions
//!    `issue_status` to `PartiallyReturned` / `Returned`,
//!    bumps `version`) and emits an [`IssuedItemReturned`]
//!    event carrying the returned quantity + new status.
//!
//! The tests use the same fixture pattern as
//! `tests/vehicle.rs` (`TestClock` + `SystemIdGen`). The
//! handlers / outbox / audit fan-out are not yet wired
//! end-to-end; these tests pin the **service layer** contract
//! that the dispatcher will eventually wrap.
//!
//! Mirrors `crates/domains/library/tests/aggregates.rs` and
//! `crates/domains/attendance/tests/aggregates.rs` (lean).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::clock::{SystemIdGen, TestClock};
use educore_core::error::DomainError;
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent;
use educore_facilities::prelude::*;
use educore_facilities::services::{issue_item, return_issued_item};

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

/// Mint a typed `AcademicYearId` for the given school.
fn academic_year_id(g: &SystemIdGen, school: SchoolId) -> AcademicYearId {
    AcademicYearId::new(school, g.next_uuid())
}

/// Mint a typed `ItemId` for the given school.
fn item_id(g: &SystemIdGen, school: SchoolId) -> ItemId {
    ItemId::new(school, g.next_uuid())
}

/// Mint a typed `ItemCategoryId` for the given school.
fn item_category_id(g: &SystemIdGen, school: SchoolId) -> ItemCategoryId {
    ItemCategoryId::new(school, g.next_uuid())
}

/// Build a `Role(RoleId)` recipient for the issue command.
fn role_recipient(g: &SystemIdGen, school: SchoolId) -> IssueRecipient {
    IssueRecipient::Role(RoleId::new(school, g.next_uuid()))
}

// =============================================================================
// Happy path: issue + return
// =============================================================================

/// End-to-end happy path for the ItemIssue aggregate. Issue
/// 10 units of an item to a `Role` recipient, then return 4
/// units (partial), asserting that:
///
/// 1. The create flow produces an `ItemIssue` aggregate
///    carrying every field on the command (school id derived
///    from the typed id) and emits an `ItemIssued` event with
///    the right `event_type`, `aggregate_type`, and
///    `school_id`.
/// 2. The return-partial flow mutates the aggregate in place
///    (bumps `returned_quantity` to 4, transitions
///    `issue_status` to `PartiallyReturned`, bumps `version`)
///    and emits an `IssuedItemReturned` event carrying the
///    returned quantity + new status.
/// 3. The remaining 6 units are still outstanding via
///    `outstanding_quantity()`.
#[test]
fn item_issue_create_then_partial_return_mutates_aggregate_and_emits_events() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // ---- Create (issue) flow ----
    let create_cmd = IssueItemCommand {
        tenant: tenant.clone(),
        academic_year_id: academic_year_id(&g, school),
        issue_to: role_recipient(&g, school),
        issue_by: tenant.actor_id,
        issue_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 13).expect("valid date"),
        due_date: None,
        item_category_id: item_category_id(&g, school),
        item_id: item_id(&g, school),
        quantity: ItemQuantity(10),
        note: None,
    };
    let (mut issue, issued_event) = issue_item(create_cmd, &clock, &ids).expect("issue item");

    // Aggregate fields are populated from the command.
    assert_eq!(issue.school_id, school);
    assert_eq!(issue.quantity.value(), 10);
    assert_eq!(issue.returned_quantity.value(), 0);
    assert_eq!(issue.issue_status, IssueStatus::Issued);
    assert_eq!(issue.created_by, tenant.actor_id);
    assert_eq!(issue.updated_by, tenant.actor_id);
    // Audit metadata footer is initialised.
    assert_eq!(issue.version.get(), 1);
    assert!(issue.active_status.is_active());
    // Outstanding = issued - returned = 10.
    assert_eq!(issue.outstanding_quantity().value(), 10);

    // Event metadata matches the aggregate's typed id and
    // the DomainEvent trait's contract.
    assert_eq!(
        <ItemIssued as DomainEvent>::EVENT_TYPE,
        "facilities.item_issue.issued"
    );
    assert_eq!(
        <ItemIssued as DomainEvent>::AGGREGATE_TYPE,
        "item_issue"
    );
    assert_eq!(<ItemIssued as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(issued_event.aggregate_id(), issue.id.as_uuid());
    assert_eq!(issued_event.school_id(), school);
    assert_eq!(issued_event.quantity, 10);
    assert_eq!(issued_event.item_id, issue.item_id);

    // ---- Return partial flow ----
    let initial_version = issue.version.get();
    let return_cmd = ReturnIssuedItemCommand {
        tenant: tenant.clone(),
        item_issue_id: issue.id,
        returned_quantity: ItemQuantity(4),
    };
    let returned_event =
        return_issued_item(&mut issue, return_cmd, &clock, &ids).expect("return partial");

    // The aggregate is mutated in place.
    assert_eq!(issue.returned_quantity.value(), 4);
    assert_eq!(issue.issue_status, IssueStatus::PartiallyReturned);
    assert_eq!(issue.version.get(), initial_version + 1);
    assert_eq!(issue.updated_by, tenant.actor_id);
    // Outstanding = issued - returned = 10 - 4 = 6.
    assert_eq!(issue.outstanding_quantity().value(), 6);

    // The event carries the returned quantity and the new
    // status (PartiallyReturned).
    assert_eq!(
        <IssuedItemReturned as DomainEvent>::EVENT_TYPE,
        "facilities.item_issue.returned"
    );
    assert_eq!(
        <IssuedItemReturned as DomainEvent>::AGGREGATE_TYPE,
        "item_issue"
    );
    assert_eq!(<IssuedItemReturned as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(returned_event.aggregate_id(), issue.id.as_uuid());
    assert_eq!(returned_event.school_id(), school);
    assert_eq!(returned_event.returned_quantity, 4);
    assert_eq!(returned_event.new_status, IssueStatus::PartiallyReturned);
}

// =============================================================================
// Full return transitions status to Returned
// =============================================================================

/// Returning the full issued quantity transitions
/// `issue_status` from `Issued` to `Returned`, and
/// `outstanding_quantity()` drops to zero.
#[test]
fn item_issue_full_return_transitions_to_returned_status() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let create_cmd = IssueItemCommand {
        tenant: tenant.clone(),
        academic_year_id: academic_year_id(&g, school),
        issue_to: role_recipient(&g, school),
        issue_by: tenant.actor_id,
        issue_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 13).expect("valid date"),
        due_date: None,
        item_category_id: item_category_id(&g, school),
        item_id: item_id(&g, school),
        quantity: ItemQuantity(5),
        note: None,
    };
    let (mut issue, _ev) = issue_item(create_cmd, &clock, &ids).expect("issue item");

    let return_cmd = ReturnIssuedItemCommand {
        tenant: tenant.clone(),
        item_issue_id: issue.id,
        returned_quantity: ItemQuantity(5),
    };
    let returned_event =
        return_issued_item(&mut issue, return_cmd, &clock, &ids).expect("full return");

    assert_eq!(issue.issue_status, IssueStatus::Returned);
    assert_eq!(issue.returned_quantity.value(), 5);
    assert_eq!(issue.outstanding_quantity().value(), 0);
    assert_eq!(returned_event.new_status, IssueStatus::Returned);
}

// =============================================================================
// Validation failure: zero quantity is rejected
// =============================================================================

/// Validation-failure path on the create flow: when the
/// `quantity` is zero, the service factory `issue_item`
/// returns `DomainError::Validation` before any aggregate is
/// constructed and no event is minted.
#[test]
fn item_issue_create_with_zero_quantity_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let create_cmd = IssueItemCommand {
        tenant: tenant.clone(),
        academic_year_id: academic_year_id(&g, school),
        issue_to: role_recipient(&g, school),
        issue_by: tenant.actor_id,
        issue_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 13).expect("valid date"),
        due_date: None,
        item_category_id: item_category_id(&g, school),
        item_id: item_id(&g, school),
        quantity: ItemQuantity(0),
        note: None,
    };
    let err = issue_item(create_cmd, &clock, &ids).expect_err("zero quantity must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}
