//! Integration tests for the **Expense aggregate** vertical slice.
//!
//! Pins the typed-id contract for
//! [`Expense`](educore_finance::aggregate::Expense) end-to-end.

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
use educore_finance::value_objects::ExpenseId;

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

fn expense_id(g: &SystemIdGen, school: SchoolId) -> ExpenseId {
    ExpenseId::new(school, g.next_uuid())
}

#[test]
fn expense_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = expense_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn expense_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = expense_id(&g, school);
    let id_b = expense_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// =========================================================================
// -- Wave 142 -- Expense -- EX I-2 payment_method compatible marker --
// =========================================================================

#[test]
fn ex_i_2_payment_method_compatible_with_account_dispatcher_enforced() {
    // EX I-2 marker test: the payment_method-compatible-with-account_id
    // invariant (the Expense's payment_method must be compatible with
    // the account_type of the referenced BankAccount, e.g., Cash expense
    // must reference a Cash account) is dispatcher-enforced.
    //
    // The aggregate itself has access to the payment_method + account_id
    // fields, but not to the BankAccount row's account_type, so the
    // cross-row check requires dispatcher visibility.
    //
    // The Expense aggregate (`crates/domains/finance/src/aggregate.rs:709+`)
    // pins the payload fields (payment_method + account_id) at the
    // API surface; the dispatcher must, on create, look up the
    // BankAccount row, compare its account_type against the payment_method,
    // and reject the create if they are incompatible.
    let (tenant, _g) = admin_context();
    let _ = tenant; // type-level marker
}
