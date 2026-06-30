//! Integration tests for the **ItemSell aggregate** vertical slice.
//!
//! Pins the create + update contract for
//! [`ItemSell`](educore_facilities::aggregate::ItemSell)
//! end-to-end through the service layer:
//!
//! 1. `sell_item` validates that the command carries at least
//!    one line, builds the header aggregate + the
//!    [`ItemSellChild`](educore_facilities::aggregate::ItemSellChild)
//!    line aggregates (one per `ItemSellLineSpec`), and emits
//!    a typed [`ItemSold`] event with the rolled-up
//!    `grand_total` / `total_quantity` / `total_due` fields.
//! 2. `update_item_sell` mutates the in-place aggregate
//!    (bumps `version`, swaps `total_paid`, recomputes
//!    `total_due`, updates `updated_at` / `updated_by`) and
//!    emits an [`ItemSellUpdated`] event whose `changes`
//!    list names the field(s) that actually moved.
//!
//! Mirrors `tests/item_receive.rs` (the symmetric goods-in
//! flow) and `tests/vehicle.rs`.

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
use educore_facilities::services::{sell_item, update_item_sell};

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
// Happy path: create + update on ItemSell
// =============================================================================

/// End-to-end happy path for the ItemSell aggregate. Post a
/// sale for 4 units @ 250 minor units (grand total 1000, paid
/// 1000, due 0), then update `paid_status` to `Partial` and
/// `total_paid` to 400, asserting that:
///
/// 1. The create flow produces an `ItemSell` aggregate + one
///    `ItemSellChild` line, carrying every field on the
///    command (school id derived from the typed id), and emits
///    an `ItemSold` event with the right `event_type`,
///    `aggregate_type`, and `school_id`. Totals roll up
///    correctly (`grand_total = 1000`, `total_quantity = 4`,
///    `total_due = 0`).
/// 2. The update flow mutates the aggregate in place (bumps
///    `version`, swaps `total_paid`, recomputes `total_due`)
///    and emits an `ItemSellUpdated` event whose `changes`
///    list names the fields that actually moved.
#[test]
fn item_sell_create_then_update_emits_events_and_rolls_up_totals() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let academic_year_id = AcademicYearId::new(school, g.next_uuid());
    let item_id = ItemId::new(school, g.next_uuid());
    let buyer = IssueRecipient::Role(educore_hr::value_objects::RoleId::new(
        school,
        g.next_uuid(),
    ));

    // ---- Create flow ----
    let cmd = SellItemCommand {
        tenant: tenant.clone(),
        academic_year_id,
        buyer: buyer.clone(),
        sell_date: NaiveDate::from_ymd_opt(2026, 4, 5).expect("valid date"),
        reference_no: None,
        total_paid: 1_000,
        payment_method: PaymentMethod::Cash,
        paid_status: PaidStatus::Paid,
        lines: vec![ItemSellLineSpec {
            item_id,
            sell_price: SellPrice::new(250).expect("non-negative"),
            quantity: ItemQuantity::new(4).expect("positive"),
            description: None,
        }],
        description: None,
    };
    let result = sell_item(cmd, &clock, &ids).expect("sell_item");

    // Header aggregate fields populated from the command.
    assert_eq!(result.header.school_id, school);
    assert_eq!(result.header.buyer, buyer);
    assert_eq!(result.header.academic_year_id, academic_year_id);
    assert_eq!(result.header.total_quantity.value(), 4);
    assert_eq!(result.header.grand_total, 1_000);
    assert_eq!(result.header.total_paid, 1_000);
    assert_eq!(result.header.total_due, 0); // grand_total - total_paid
    assert_eq!(result.header.payment_method, PaymentMethod::Cash);
    assert_eq!(result.header.paid_status, PaidStatus::Paid);
    assert_eq!(result.header.created_by, tenant.actor_id);
    assert_eq!(result.header.updated_by, tenant.actor_id);
    assert_eq!(result.header.version.get(), 1);
    assert!(result.header.active_status.is_active());

    // Exactly one child line was built.
    assert_eq!(result.lines.len(), 1);
    let line = &result.lines[0];
    assert_eq!(line.item_sell_id, result.header.id);
    assert_eq!(line.item_id, item_id);
    assert_eq!(line.sell_price.value(), 250);
    assert_eq!(line.quantity.value(), 4);
    assert_eq!(line.sub_total, 1_000);
    assert_eq!(line.school_id, school);

    // Event metadata matches the DomainEvent contract.
    let event = &result.event;
    assert_eq!(
        <ItemSold as DomainEvent>::EVENT_TYPE,
        "facilities.item_sell.sold"
    );
    assert_eq!(
        <ItemSold as DomainEvent>::AGGREGATE_TYPE,
        "item_sell"
    );
    assert_eq!(<ItemSold as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(event.aggregate_id(), result.header.id.as_uuid());
    assert_eq!(event.school_id(), school);
    assert_eq!(event.grand_total, 1_000);
    assert_eq!(event.total_quantity, 4);
    assert_eq!(event.total_paid, 1_000);
    assert_eq!(event.total_due, 0);
    assert_eq!(event.paid_status, PaidStatus::Paid);
    assert_eq!(event.buyer, buyer);
    assert_eq!(event.lines.len(), 1);
    assert_eq!(event.lines[0].item_id, item_id);
    assert_eq!(event.lines[0].sell_price.value(), 250);
    assert_eq!(event.lines[0].quantity.value(), 4);

    // ---- Update flow ----
    let mut header = result.header;
    let initial_version = header.version.get();
    let update_cmd = UpdateItemSellCommand {
        tenant: tenant.clone(),
        item_sell_id: header.id,
        lines_to_add: vec![],
        lines_to_remove: vec![],
        total_paid: Some(400),
        payment_method: None,
        paid_status: Some(PaidStatus::Partial),
    };
    let updated_event =
        update_item_sell(&mut header, update_cmd, &clock, &ids).expect("update_item_sell");

    // The aggregate is mutated in place.
    assert_eq!(header.total_paid, 400);
    assert_eq!(header.total_due, 600); // 1000 - 400
    assert_eq!(header.paid_status, PaidStatus::Partial);
    assert_eq!(header.version.get(), initial_version + 1);
    assert_eq!(header.updated_by, tenant.actor_id);
    assert!(header.last_event_id.is_some());

    // The event names the fields that actually moved.
    assert_eq!(
        <ItemSellUpdated as DomainEvent>::EVENT_TYPE,
        "facilities.item_sell.updated"
    );
    assert_eq!(
        <ItemSellUpdated as DomainEvent>::AGGREGATE_TYPE,
        "item_sell"
    );
    assert_eq!(<ItemSellUpdated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(updated_event.aggregate_id(), header.id.as_uuid());
    assert_eq!(updated_event.school_id(), school);
    assert!(updated_event.changes.contains(&"total_paid".to_owned()));
    assert!(updated_event.changes.contains(&"paid_status".to_owned()));
    assert_eq!(updated_event.changes.len(), 2);
}

// =============================================================================
// Multi-line rollup + validation
// =============================================================================

/// Multi-line rollup: post a sale with two lines (2 units @
/// 300 + 6 units @ 100), verify the header's `grand_total`
/// and `total_quantity` reflect both lines (2*300 + 6*100 =
/// 1200, quantity 8), and that the event payload carries
/// both line specs.
///
/// Then validates that a zero-line command is rejected with
/// `DomainError::Validation` (no aggregate is built, no event
/// is minted).
#[test]
fn item_sell_multi_line_rolls_up_totals_and_empty_lines_are_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let academic_year_id = AcademicYearId::new(school, g.next_uuid());
    let item_a = ItemId::new(school, g.next_uuid());
    let item_b = ItemId::new(school, g.next_uuid());
    let buyer = IssueRecipient::Role(educore_hr::value_objects::RoleId::new(
        school,
        g.next_uuid(),
    ));

    // ---- Multi-line create ----
    let multi_cmd = SellItemCommand {
        tenant: tenant.clone(),
        academic_year_id,
        buyer: buyer.clone(),
        sell_date: NaiveDate::from_ymd_opt(2026, 4, 6).expect("valid date"),
        reference_no: None,
        total_paid: 0,
        payment_method: PaymentMethod::Bank,
        paid_status: PaidStatus::Unpaid,
        lines: vec![
            ItemSellLineSpec {
                item_id: item_a,
                sell_price: SellPrice::new(300).expect("non-negative"),
                quantity: ItemQuantity::new(2).expect("positive"),
                description: None,
            },
            ItemSellLineSpec {
                item_id: item_b,
                sell_price: SellPrice::new(100).expect("non-negative"),
                quantity: ItemQuantity::new(6).expect("positive"),
                description: None,
            },
        ],
        description: None,
    };
    let result = sell_item(multi_cmd, &clock, &ids).expect("sell_item multi-line");

    assert_eq!(result.lines.len(), 2);
    assert_eq!(result.header.total_quantity.value(), 8);
    assert_eq!(result.header.grand_total, 1_200); // 2*300 + 6*100
    assert_eq!(result.header.total_paid, 0);
    assert_eq!(result.header.total_due, 1_200);
    assert_eq!(result.event.lines.len(), 2);
    assert_eq!(result.event.grand_total, 1_200);
    assert_eq!(result.event.total_quantity, 8);
    assert_eq!(result.event.buyer, buyer);

    // The two child lines each point back at the parent header.
    for line in &result.lines {
        assert_eq!(line.item_sell_id, result.header.id);
        assert_eq!(line.school_id, school);
    }

    // ---- Validation: empty lines rejected ----
    let empty_cmd = SellItemCommand {
        tenant,
        academic_year_id,
        buyer,
        sell_date: NaiveDate::from_ymd_opt(2026, 4, 6).expect("valid date"),
        reference_no: None,
        total_paid: 0,
        payment_method: PaymentMethod::Cash,
        paid_status: PaidStatus::Unpaid,
        lines: vec![],
        description: None,
    };
    let err = sell_item(empty_cmd, &clock, &ids).expect_err("empty lines must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );

    // The setup sanity check: school id we minted is not nil.
    assert_ne!(school, SchoolId(uuid::Uuid::nil()));
}
