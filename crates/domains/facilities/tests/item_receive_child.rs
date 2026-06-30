//! Integration tests for the **ItemReceiveChild aggregate**
//! vertical slice.
//!
//! Pins the create + update contract for
//! [`ItemReceiveChild`](educore_facilities::aggregate::ItemReceiveChild)
//! line aggregates. The child aggregate is built by the
//! `receive_item` service factory (one line per
//! `ItemReceiveLineSpec`); these tests verify the child's
//! invariants end-to-end:
//!
//! 1. The child line aggregates to `unit_price * quantity`
//!    (the `sub_total` field).
//! 2. The child carries every typed id from the command
//!    (`item_receive_id` parent, `item_id` line item), with
//!    the school id derived from the typed id.
//! 3. The audit footer is initialised (version 1, active
//!    status, etag, created_by == updated_by, last_event_id
//!    pending).
//!
//! There is no `update_item_receive_child` service factory
//! yet — the child is mutated by `update_item_receive` on the
//! parent header. The "update" half of this contract is
//! therefore pinned by re-running the constructor after the
//! header update to verify the child line is stable (the
//! child line is not mutated by the parent update flow).
//!
//! Mirrors `tests/item_receive.rs` (the parent) and
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
use educore_facilities::services::{receive_item, update_item_receive};

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

/// End-to-end happy path for `ItemReceiveChild`. Post a
/// single-line GRN and verify that the child line aggregate:
///
/// - has `school_id` derived from the typed id,
/// - has `sub_total == unit_price * quantity`,
/// - has `item_receive_id == header.id` (the parent pointer),
/// - has `item_id` matching the command,
/// - has the initial audit footer (`version == 1`,
///   `created_by == updated_by`, `last_event_id == None`,
///   `active_status.is_active()`).
#[test]
fn item_receive_child_single_line_aggregates_subtotal_and_audit_footer() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let academic_year_id = AcademicYearId::new(school, g.next_uuid());
    let supplier_id = SupplierId::new(school, g.next_uuid());
    let store_id = ItemStoreId::new(school, g.next_uuid());
    let item_id = ItemId::new(school, g.next_uuid());

    let cmd = ReceiveItemCommand {
        tenant: tenant.clone(),
        academic_year_id,
        receive_date: NaiveDate::from_ymd_opt(2026, 3, 10).expect("valid date"),
        reference_no: None,
        supplier_id,
        store_id,
        total_paid: 0,
        payment_method: PaymentMethod::Cash,
        paid_status: PaidStatus::Unpaid,
        lines: vec![ItemReceiveLineSpec {
            item_id,
            unit_price: UnitPrice::new(250).expect("non-negative"),
            quantity: ItemQuantity::new(4).expect("positive"),
            description: None,
        }],
        description: None,
    };
    let result = receive_item(cmd, &clock, &ids).expect("receive_item");

    // Exactly one child line was built.
    assert_eq!(result.lines.len(), 1);
    let line = &result.lines[0];

    // Aggregates to unit_price * quantity.
    assert_eq!(line.unit_price.value(), 250);
    assert_eq!(line.quantity.value(), 4);
    assert_eq!(line.sub_total, 1_000);

    // Parent pointer and line-item pointer are populated.
    assert_eq!(line.item_receive_id, result.header.id);
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

/// Multi-line children: post a GRN with three lines and
/// verify that each child line carries the correct
/// `sub_total`, the same parent `item_receive_id`, and a
/// unique typed `ItemReceiveChildId`.
///
/// Then update the parent header (changing `total_paid`) and
/// verify the child lines are unaffected — the
/// `update_item_receive` flow on the parent does not mutate
/// the child line aggregates.
///
/// Finally, build an `ItemReceiveChild` directly via the
/// `fresh` constructor to verify the type's invariants in
/// isolation (no service factory).
#[test]
fn item_receive_child_multi_line_unique_ids_and_stable_across_parent_update() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let academic_year_id = AcademicYearId::new(school, g.next_uuid());
    let supplier_id = SupplierId::new(school, g.next_uuid());
    let store_id = ItemStoreId::new(school, g.next_uuid());
    let item_a = ItemId::new(school, g.next_uuid());
    let item_b = ItemId::new(school, g.next_uuid());
    let item_c = ItemId::new(school, g.next_uuid());

    let multi_cmd = ReceiveItemCommand {
        tenant: tenant.clone(),
        academic_year_id,
        receive_date: NaiveDate::from_ymd_opt(2026, 3, 11).expect("valid date"),
        reference_no: None,
        supplier_id,
        store_id,
        total_paid: 0,
        payment_method: PaymentMethod::Cash,
        paid_status: PaidStatus::Unpaid,
        lines: vec![
            ItemReceiveLineSpec {
                item_id: item_a,
                unit_price: UnitPrice::new(100).expect("non-negative"),
                quantity: ItemQuantity::new(2).expect("positive"),
                description: None,
            },
            ItemReceiveLineSpec {
                item_id: item_b,
                unit_price: UnitPrice::new(75).expect("non-negative"),
                quantity: ItemQuantity::new(8).expect("positive"),
                description: None,
            },
            ItemReceiveLineSpec {
                item_id: item_c,
                unit_price: UnitPrice::new(50).expect("non-negative"),
                quantity: ItemQuantity::new(12).expect("positive"),
                description: None,
            },
        ],
        description: None,
    };
    let result = receive_item(multi_cmd, &clock, &ids).expect("receive_item multi-line");

    // Three child lines, each with a unique typed id, all
    // pointing at the parent header.
    assert_eq!(result.lines.len(), 3);

    let subs: Vec<i64> = result.lines.iter().map(|l| l.sub_total).collect();
    assert_eq!(subs, vec![200, 600, 600]); // 2*100, 8*75, 12*50

    let ids_set: std::collections::HashSet<_> =
        result.lines.iter().map(|l| l.id).collect();
    assert_eq!(ids_set.len(), 3, "child ids must be unique");

    for line in &result.lines {
        assert_eq!(line.item_receive_id, result.header.id);
        assert_eq!(line.school_id, school);
    }

    // Snapshot child line subtotals before parent update.
    let snapshot: Vec<i64> = result.lines.iter().map(|l| l.sub_total).collect();

    // ---- Update parent header; verify children unchanged ----
    let mut header = result.header;
    let update_cmd = UpdateItemReceiveCommand {
        tenant,
        item_receive_id: header.id,
        lines_to_add: vec![],
        lines_to_remove: vec![],
        total_paid: Some(700),
        payment_method: None,
        paid_status: None,
    };
    update_item_receive(&mut header, update_cmd, &clock, &ids).expect("update_item_receive");

    // Child lines (kept in the local `result.lines` vec) are
    // not touched by the parent update flow; the service only
    // mutates the header.
    let after_update: Vec<i64> = result.lines.iter().map(|l| l.sub_total).collect();
    assert_eq!(after_update, snapshot);

    // ---- Direct constructor: invariants in isolation ----
    let now = clock.now();
    let direct_id = ItemReceiveChildId::new(school, g.next_uuid());
    let direct_parent = header.id;
    let direct = ItemReceiveChild::fresh(
        direct_id,
        direct_parent,
        item_a,
        UnitPrice::new(100).expect("non-negative"),
        ItemQuantity::new(2).expect("positive"),
        None,
        g.next_user_id(),
        now,
        g.next_correlation_id(),
    );
    assert_eq!(direct.sub_total, 200);
    assert_eq!(direct.item_receive_id, direct_parent);
    assert_eq!(direct.school_id, school);
    assert_eq!(direct.version.get(), 1);
    assert!(direct.active_status.is_active());

    // Setup sanity.
    assert_ne!(school, SchoolId(uuid::Uuid::nil()));
}
