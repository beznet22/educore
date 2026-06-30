//! Integration tests for the **ItemReceive aggregate** vertical slice.
//!
//! Pins the create + update contract for
//! [`ItemReceive`](educore_facilities::aggregate::ItemReceive)
//! end-to-end through the service layer:
//!
//! 1. `receive_item` validates that the command carries at least
//!    one line, builds the header aggregate + the
//!    [`ItemReceiveChild`](educore_facilities::aggregate::ItemReceiveChild)
//!    line aggregates (one per `ItemReceiveLineSpec`), and emits
//!    a typed [`ItemReceived`] event with the rolled-up
//!    `grand_total` / `total_quantity` / `total_due` fields.
//! 2. `update_item_receive` mutates the in-place aggregate
//!    (bumps `version`, swaps `total_paid`, recomputes
//!    `total_due`, updates `updated_at` / `updated_by`) and
//!    emits an [`ItemReceiveUpdated`] event whose `changes`
//!    list names the field(s) that actually moved.
//!
//! Mirrors `tests/vehicle.rs` and
//! `crates/domains/library/tests/aggregates.rs` (lean).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use chrono::NaiveDate;
use educore_core::clock::{SystemIdGen, TestClock};
use educore_core::error::DomainError;
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent;
use educore_facilities::prelude::*;
use educore_facilities::services::{receive_item, update_item_receive};

// =============================================================================
// Fixtures
// =============================================================================

/// A fresh `TenantContext` for a `SchoolAdmin` acting on a
/// freshly-minted school. Returns the context plus the
/// generator so tests can mint child ids from the same school.
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

fn make_receive_cmd(
    tenant: TenantContext,
    academic_year_id: AcademicYearId,
    supplier_id: SupplierId,
    store_id: ItemStoreId,
    item_id: ItemId,
    unit_price: i64,
    quantity: i64,
    total_paid: i64,
    paid_status: PaidStatus,
    payment_method: PaymentMethod,
) -> ReceiveItemCommand {
    ReceiveItemCommand {
        tenant,
        academic_year_id,
        receive_date: NaiveDate::from_ymd_opt(2026, 1, 15).expect("valid date"),
        reference_no: None,
        supplier_id,
        store_id,
        total_paid,
        payment_method,
        paid_status,
        lines: vec![ItemReceiveLineSpec {
            item_id,
            unit_price: UnitPrice::new(unit_price).expect("non-negative unit price"),
            quantity: ItemQuantity::new(quantity).expect("positive quantity"),
            description: None,
        }],
        description: None,
    }
}

// =============================================================================
// Happy path: create + update on ItemReceive
// =============================================================================

/// End-to-end happy path for the ItemReceive aggregate. Post a
/// GRN for 10 units @ 100 minor units (grand total 1000, paid
/// 400, due 600), then update `total_paid` to 600, asserting
/// that:
///
/// 1. The create flow produces an `ItemReceive` aggregate + one
///    `ItemReceiveChild` line, carrying every field on the
///    command (school id derived from the typed id), and emits
///    an `ItemReceived` event with the right `event_type`,
///    `aggregate_type`, and `school_id`. Totals roll up
///    correctly (`grand_total = 1000`, `total_quantity = 10`,
///    `total_due = 600`).
/// 2. The update flow mutates the aggregate in place (bumps
///    `version`, swaps `total_paid`, recomputes `total_due`)
///    and emits an `ItemReceiveUpdated` event whose `changes`
///    list names `total_paid` (the only field that moved).
#[test]
fn item_receive_create_then_update_emits_events_and_rolls_up_totals() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let academic_year_id = AcademicYearId::new(school, g.next_uuid());
    let supplier_id = SupplierId::new(school, g.next_uuid());
    let store_id = ItemStoreId::new(school, g.next_uuid());
    let item_id = ItemId::new(school, g.next_uuid());

    // ---- Create flow ----
    let cmd = make_receive_cmd(
        tenant.clone(),
        academic_year_id,
        supplier_id,
        store_id,
        item_id,
        100, // unit_price
        10,  // quantity
        400, // total_paid
        PaidStatus::Partial,
        PaymentMethod::Cash,
    );
    let result = receive_item(cmd, &clock, &ids).expect("receive_item");

    // Header aggregate fields populated from the command.
    assert_eq!(result.header.school_id, school);
    assert_eq!(result.header.supplier_id, supplier_id);
    assert_eq!(result.header.store_id, store_id);
    assert_eq!(result.header.academic_year_id, academic_year_id);
    assert_eq!(result.header.total_quantity.value(), 10);
    assert_eq!(result.header.grand_total, 1_000);
    assert_eq!(result.header.total_paid, 400);
    assert_eq!(result.header.total_due, 600); // grand_total - total_paid
    assert_eq!(result.header.payment_method, PaymentMethod::Cash);
    assert_eq!(result.header.paid_status, PaidStatus::Partial);
    assert_eq!(result.header.created_by, tenant.actor_id);
    assert_eq!(result.header.updated_by, tenant.actor_id);
    assert_eq!(result.header.version.get(), 1);
    assert!(result.header.active_status.is_active());

    // Exactly one child line was built.
    assert_eq!(result.lines.len(), 1);
    let line = &result.lines[0];
    assert_eq!(line.item_receive_id, result.header.id);
    assert_eq!(line.item_id, item_id);
    assert_eq!(line.unit_price.value(), 100);
    assert_eq!(line.quantity.value(), 10);
    assert_eq!(line.sub_total, 1_000);
    assert_eq!(line.school_id, school);

    // Event metadata matches the DomainEvent contract.
    let event = &result.event;
    assert_eq!(
        <ItemReceived as DomainEvent>::EVENT_TYPE,
        "facilities.item_receive.received"
    );
    assert_eq!(
        <ItemReceived as DomainEvent>::AGGREGATE_TYPE,
        "item_receive"
    );
    assert_eq!(<ItemReceived as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(event.aggregate_id(), result.header.id.as_uuid());
    assert_eq!(event.school_id(), school);
    assert_eq!(event.grand_total, 1_000);
    assert_eq!(event.total_quantity, 10);
    assert_eq!(event.total_paid, 400);
    assert_eq!(event.total_due, 600);
    assert_eq!(event.paid_status, PaidStatus::Partial);
    assert_eq!(event.lines.len(), 1);
    assert_eq!(event.lines[0].item_id, item_id);
    assert_eq!(event.lines[0].unit_price.value(), 100);
    assert_eq!(event.lines[0].quantity.value(), 10);

    // ---- Update flow ----
    let mut header = result.header;
    let initial_version = header.version.get();
    let update_cmd = UpdateItemReceiveCommand {
        tenant: tenant.clone(),
        item_receive_id: header.id,
        lines_to_add: vec![],
        lines_to_remove: vec![],
        total_paid: Some(600),
        payment_method: None,
        paid_status: Some(PaidStatus::Paid),
    };
    let updated_event =
        update_item_receive(&mut header, update_cmd, &clock, &ids).expect("update_item_receive");

    // The aggregate is mutated in place.
    assert_eq!(header.total_paid, 600);
    assert_eq!(header.total_due, 400); // 1000 - 600
    assert_eq!(header.paid_status, PaidStatus::Paid);
    assert_eq!(header.version.get(), initial_version + 1);
    assert_eq!(header.updated_by, tenant.actor_id);
    assert!(header.last_event_id.is_some());

    // The event names the fields that actually moved.
    assert_eq!(
        <ItemReceiveUpdated as DomainEvent>::EVENT_TYPE,
        "facilities.item_receive.updated"
    );
    assert_eq!(
        <ItemReceiveUpdated as DomainEvent>::AGGREGATE_TYPE,
        "item_receive"
    );
    assert_eq!(<ItemReceiveUpdated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(updated_event.aggregate_id(), header.id.as_uuid());
    assert_eq!(updated_event.school_id(), school);
    assert!(updated_event.changes.contains(&"total_paid".to_owned()));
    assert!(updated_event.changes.contains(&"paid_status".to_owned()));
    assert_eq!(updated_event.changes.len(), 2);
}

// =============================================================================
// Multi-line rollup + validation
// =============================================================================

/// Multi-line rollup: post a GRN with two lines (3 units @ 200
/// + 5 units @ 150), verify the header's `grand_total` and
/// `total_quantity` reflect both lines (3*200 + 5*150 = 1350,
/// quantity 8), and that the event payload carries both line
/// specs.
///
/// Then validates that a zero-line command is rejected with
/// `DomainError::Validation` (no aggregate is built, no event
/// is minted).
#[test]
fn item_receive_multi_line_rolls_up_totals_and_empty_lines_are_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let academic_year_id = AcademicYearId::new(school, g.next_uuid());
    let supplier_id = SupplierId::new(school, g.next_uuid());
    let store_id = ItemStoreId::new(school, g.next_uuid());
    let item_a = ItemId::new(school, g.next_uuid());
    let item_b = ItemId::new(school, g.next_uuid());

    // ---- Multi-line create ----
    let multi_cmd = ReceiveItemCommand {
        tenant: tenant.clone(),
        academic_year_id,
        receive_date: NaiveDate::from_ymd_opt(2026, 2, 1).expect("valid date"),
        reference_no: None,
        supplier_id,
        store_id,
        total_paid: 0,
        payment_method: PaymentMethod::Bank,
        paid_status: PaidStatus::Unpaid,
        lines: vec![
            ItemReceiveLineSpec {
                item_id: item_a,
                unit_price: UnitPrice::new(200).expect("non-negative"),
                quantity: ItemQuantity::new(3).expect("positive"),
                description: None,
            },
            ItemReceiveLineSpec {
                item_id: item_b,
                unit_price: UnitPrice::new(150).expect("non-negative"),
                quantity: ItemQuantity::new(5).expect("positive"),
                description: None,
            },
        ],
        description: None,
    };
    let result = receive_item(multi_cmd, &clock, &ids).expect("receive_item multi-line");

    assert_eq!(result.lines.len(), 2);
    assert_eq!(result.header.total_quantity.value(), 8);
    assert_eq!(result.header.grand_total, 1_350); // 3*200 + 5*150
    assert_eq!(result.header.total_paid, 0);
    assert_eq!(result.header.total_due, 1_350);
    assert_eq!(result.event.lines.len(), 2);
    assert_eq!(result.event.grand_total, 1_350);
    assert_eq!(result.event.total_quantity, 8);

    // The two child lines each point back at the parent header.
    for line in &result.lines {
        assert_eq!(line.item_receive_id, result.header.id);
        assert_eq!(line.school_id, school);
    }

    // ---- Validation: empty lines rejected ----
    let empty_cmd = ReceiveItemCommand {
        tenant,
        academic_year_id,
        receive_date: NaiveDate::from_ymd_opt(2026, 2, 1).expect("valid date"),
        reference_no: None,
        supplier_id,
        store_id,
        total_paid: 0,
        payment_method: PaymentMethod::Cash,
        paid_status: PaidStatus::Unpaid,
        lines: vec![],
        description: None,
    };
    let err = receive_item(empty_cmd, &clock, &ids).expect_err("empty lines must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );

    // The setup sanity check: school id we minted is not nil.
    assert_ne!(school, SchoolId(uuid::Uuid::nil()));
}
