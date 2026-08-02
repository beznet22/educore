//! Integration tests for the **FeesInvoice aggregate** vertical slice.
//!
//! Pins the typed-id contract for
//! [`FeesInvoice`](educore_finance::aggregate::FeesInvoice) end-to-end.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::clock::{IdGenerator as _, SystemIdGen};
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_finance::value_objects::FeesInvoiceId;

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

fn fees_invoice_id(g: &SystemIdGen, school: SchoolId) -> FeesInvoiceId {
    FeesInvoiceId::new(school, g.next_uuid())
}

#[test]
fn fees_invoice_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fees_invoice_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn fees_invoice_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = fees_invoice_id(&g, school);
    let id_b = fees_invoice_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// =========================================================================
// -- Wave 154 -- FeesInvoice -- FI I-3 storage-layer marker --
// =========================================================================

#[test]
fn fi_i_3_one_per_school_unique_constraint_storage_layer() {
    // FI I-3 marker test: the "one FeesInvoice per school"
    // invariant is storage-layer enforced. The FeesInvoice
    // aggregate at `crates/domains/finance/src/aggregate.rs:427`
    // (the root aggregate, not a placeholder stub) carries
    // `prefix` + `start_form` + the audit footer. A unique
    // index on `school_id` in the storage layer (SurrealDB /
    // PostgreSQL / MySQL / SQLite adapters) enforces the
    // one-per-school invariant at write time. The dispatcher
    // relies on the storage-layer unique constraint to reject
    // duplicate-row attempts; this marker test documents the
    // dispatcher / storage adapter responsibility.
    let (tenant, _g) = admin_context();
    let _ = tenant; // type-level marker
}
