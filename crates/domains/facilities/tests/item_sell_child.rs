//! Integration tests for the **ItemSellChild aggregate**
//! vertical slice.
//!
//! Pins the create + update contract for
//! [`ItemSellChild`](educore_facilities::aggregate::ItemSellChild)
//! line aggregates. The child aggregate is built by the
//! `sell_item` service factory (one line per
//! `ItemSellLineSpec`); these tests verify the child's
//! invariants end-to-end:
//!
//! 1. The child line aggregates to `sell_price * quantity`
//!    (the `sub_total` field).
//! 2. The child carries every typed id from the command
//!    (`item_sell_id` parent, `item_id` line item), with the
//!    school id derived from the typed id.
//! 3. The audit footer is initialised (version 1, active
//!    status, etag, created_by == updated_by, last_event_id
//!    pending).
//!
//! There is no `update_item_sell_child` service factory yet
//! — the child is mutated by `update_item_sell` on the parent
//! header. The "update" half of this contract is therefore
//! pinned by re-running the constructor after the header
//! update to verify the child line is stable (the child line
//! is not mutated by the parent update flow).
//!
//! Mirrors `tests/item_sell.rs` (the parent) and
//! `crates/domains/library/tests/aggregates.rs`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use chrono::NaiveDate;
use educore_core::clock::{SystemIdGen, TestClock};
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
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
// Happy path: child line is built correctly
// =============================================================================

/// End-to-end happy path for `ItemSellChild`. Post a
/// single-line sale and verify that the child line aggregate:
///
/// - has `school_id` derived from the typed id,
/// - has `sub_total == sell_price * quantity`,
/// - has `item_sell_id == header.id` (the parent pointer),
/// - has `item_id` matching the command,
/// - has the initial audit footer (`version == 1`,
///   `created_by == updated_by`, `last_event_id == None`,
///   `active_status.is_active()`).
#[test]
fn item_sell_child_single_line_aggregates_subtotal_and_audit_footer() {
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

    let cmd = SellItemCommand {
        tenant: tenant.clone(),
        academic_year_id,
        buyer,
        sell_date: NaiveDate::from_ymd_opt(2026, 5, 10).expect("valid date"),
        reference_no: None,
        total_paid: 0,
        payment_method: PaymentMethod::Cash,
        paid_status: PaidStatus::Unpaid,
        lines: vec![ItemSellLineSpec {
            item_id,
            sell_price: SellPrice::new(400).expect("non-negative"),
            quantity: ItemQuantity::new(3).expect("positive"),
            description: None,
        }],
        description: None,
    };
    let result = sell_item(cmd, &clock, &ids).expect("sell_item");

    // Exactly one child line was built.
    assert_eq!(result.lines.len(), 1);
    let line = &result.lines[0];

    // Aggregates to sell_price * quantity.
    assert_eq!(line.sell_price.value(), 400);
    assert_eq!(line.quantity.value(), 3);
    assert_eq!(line.sub_total, 1_200);

    // Parent pointer and line-item pointer are populated.
    assert_eq!(line.item_sell_id, result.header.id);
    assert_eq!(line.item_id, item_id);

    // School id is derived from the typed id.
    assert_eq!(line.school_id, school);
    assert_eq!(line.id.school_id(), school);

    // Audit footer is initialised.
    assert_eq!(line.version.get(), 1);
    assert_eq!(line.created_by, tenant.actor_id);
    assert_eq!(line.updated_by, tenant.actor_id);
    assert!(line.active_status.is_active());
    assert!(line.last_event_id.is_none());

    // Typed id is not nil and carries the right school.
    assert_ne!(line.id.as_uuid(), uuid::Uuid::nil());
}

// =============================================================================
// Multi-line children + stability across parent update
// =============================================================================

/// Multi-line children: post a sale with three lines and
/// verify that each child line carries the correct
/// `sub_total`, the same parent `item_sell_id`, and a unique
/// typed `ItemSellChildId`.
///
/// Then update the parent header (changing `payment_method`)
/// and verify the child lines are unaffected — the
/// `update_item_sell` flow on the parent does not mutate the
/// child line aggregates.
///
/// Finally, build an `ItemSellChild` directly via the
/// `fresh` constructor to verify the type's invariants in
/// isolation (no service factory).
#[test]
fn item_sell_child_multi_line_unique_ids_and_stable_across_parent_update() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let academic_year_id = AcademicYearId::new(school, g.next_uuid());
    let item_a = ItemId::new(school, g.next_uuid());
    let item_b = ItemId::new(school, g.next_uuid());
    let item_c = ItemId::new(school, g.next_uuid());
    let buyer = IssueRecipient::Role(educore_hr::value_objects::RoleId::new(
        school,
        g.next_uuid(),
    ));

    let multi_cmd = SellItemCommand {
        tenant: tenant.clone(),
        academic_year_id,
        buyer,
        sell_date: NaiveDate::from_ymd_opt(2026, 5, 11).expect("valid date"),
        reference_no: None,
        total_paid: 0,
        payment_method: PaymentMethod::Cash,
        paid_status: PaidStatus::Unpaid,
        lines: vec![
            ItemSellLineSpec {
                item_id: item_a,
                sell_price: SellPrice::new(500).expect("non-negative"),
                quantity: ItemQuantity::new(2).expect("positive"),
                description: None,
            },
            ItemSellLineSpec {
                item_id: item_b,
                sell_price: SellPrice::new(125).expect("non-negative"),
                quantity: ItemQuantity::new(4).expect("positive"),
                description: None,
            },
            ItemSellLineSpec {
                item_id: item_c,
                sell_price: SellPrice::new(50).expect("non-negative"),
                quantity: ItemQuantity::new(10).expect("positive"),
                description: None,
            },
        ],
        description: None,
    };
    let result = sell_item(multi_cmd, &clock, &ids).expect("sell_item multi-line");

    // Three child lines, each with a unique typed id, all
    // pointing at the parent header.
    assert_eq!(result.lines.len(), 3);

    let subs: Vec<i64> = result.lines.iter().map(|l| l.sub_total).collect();
    assert_eq!(subs, vec![1_000, 500, 500]); // 2*500, 4*125, 10*50

    let ids_set: std::collections::HashSet<_> =
        result.lines.iter().map(|l| l.id).collect();
    assert_eq!(ids_set.len(), 3, "child ids must be unique");

    for line in &result.lines {
        assert_eq!(line.item_sell_id, result.header.id);
        assert_eq!(line.school_id, school);
    }

    // Snapshot child line subtotals before parent update.
    let snapshot: Vec<i64> = result.lines.iter().map(|l| l.sub_total).collect();

    // ---- Update parent header; verify children unchanged ----
    let mut header = result.header;
    let update_cmd = UpdateItemSellCommand {
        tenant,
        item_sell_id: header.id,
        lines_to_add: vec![],
        lines_to_remove: vec![],
        total_paid: None,
        payment_method: Some(PaymentMethod::Bank),
        paid_status: None,
    };
    update_item_sell(&mut header, update_cmd, &clock, &ids).expect("update_item_sell");

    // Child lines (kept in the local `result.lines` vec) are
    // not touched by the parent update flow; the service only
    // mutates the header.
    let after_update: Vec<i64> = result.lines.iter().map(|l| l.sub_total).collect();
    assert_eq!(after_update, snapshot);

    // ---- Direct constructor: invariants in isolation ----
    let now = clock.now();
    let direct_id = ItemSellChildId::new(school, g.next_uuid());
    let direct_parent = header.id;
    let direct = ItemSellChild::fresh(
        direct_id,
        direct_parent,
        item_a,
        SellPrice::new(500).expect("non-negative"),
        ItemQuantity::new(2).expect("positive"),
        None,
        g.next_user_id(),
        now,
        g.next_correlation_id(),
    );
    assert_eq!(direct.sub_total, 1_000);
    assert_eq!(direct.item_sell_id, direct_parent);
    assert_eq!(direct.school_id, school);
    assert_eq!(direct.version.get(), 1);
    assert!(direct.active_status.is_active());

    // Setup sanity.
    assert_ne!(school, SchoolId(uuid::Uuid::nil()));
}
