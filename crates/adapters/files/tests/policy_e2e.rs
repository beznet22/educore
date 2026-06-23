//! # Cluster F: files storage policy + tenant guard (policy_e2e)
//!
//! Integration test for `crates/adapters/files/src/policy.rs`.
//! Closes finding **ADAPTER-FILE-003** (tenant context enforcement)
//! from `docs/audit_reports/findings/wave3-files.md`.
//!
//! `policy.rs` is intentionally not exported via `lib.rs` for this
//! drop — it is the first piece of the tenant-context hardening
//! stack and the public surface (re-exports, prelude entries,
//! storage-adapter wiring) will follow in subsequent commits. To
//! still exercise the public API from an integration test, the
//! source file is included directly via `#[path = ...]`, which is
//! the standard Cargo pattern for testing private modules without
//! polluting the library's public API.
//!
//! ## Scenarios (6)
//!
//! 1. `StoragePolicy::check_upload allows within quota`
//! 2. `StoragePolicy::check_upload rejects over quota`
//! 3. `StoragePolicy per-school quota overrides default`
//! 4. `StoragePolicy::record_usage updates used bytes`
//! 5. `TenantGuard::assert_same_tenant passes for same school`
//! 6. `TenantGuard::assert_same_tenant rejects cross-tenant access`

#![cfg(test)]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]

// Include `policy.rs` directly via `#[path]` so the test exercises
// the public API without requiring a `pub mod policy;` declaration
// in `crates/adapters/files/src/lib.rs`. The module becomes a child
// of this integration-test binary and inherits the same
// extern-crate graph (educore_core, …) as the test itself.
#[path = "../src/policy.rs"]
mod policy;

use educore_core::clock::{IdGenerator as _, SystemIdGen};
use educore_core::ids::SchoolId;

use policy::{CrossTenantError, QuotaExceededError, StoragePolicy, TenantGuard};

/// Builds a fresh [`SchoolId`] via [`SystemIdGen`].
///
/// Each call returns a distinct UUIDv7 value (the timestamp +
/// random bits differ across calls), so consecutive calls yield
/// two schools that compare unequal under `==`. We use the engine's
/// own generator (already in `educore_core`'s public API) instead
/// of `uuid::Uuid` directly so the integration test does not need
/// to add `uuid` as a `dev-dependency`.
fn school() -> SchoolId {
    SystemIdGen.next_school_id()
}

/// Returns two distinct [`SchoolId`]s in one call. Both ids are
/// fresh UUIDv7 values from [`SystemIdGen`].
fn two_schools() -> (SchoolId, SchoolId) {
    (school(), school())
}

// ---------------------------------------------------------------------------
// Scenario 1: StoragePolicy::check_upload allows within quota
// ---------------------------------------------------------------------------

#[test]
fn check_upload_allows_within_quota() {
    let policy = StoragePolicy::new(1024);
    let s = school();

    // Zero-bytes upload against a positive quota is allowed.
    assert_eq!(policy.check_upload(s, 0), Ok(()));
    // A upload well under the cap is allowed.
    assert_eq!(policy.check_upload(s, 512), Ok(()));
    // Exactly at the cap is allowed (rejection is strict greater-than).
    assert_eq!(policy.check_upload(s, 1024), Ok(()));
}

// ---------------------------------------------------------------------------
// Scenario 2: StoragePolicy::check_upload rejects over quota
// ---------------------------------------------------------------------------

#[test]
fn check_upload_rejects_over_quota() {
    let policy = StoragePolicy::new(1024);
    let s = school();

    let err = policy.check_upload(s, 1025).unwrap_err();
    assert_eq!(
        err,
        QuotaExceededError::OverQuota {
            used: 1025,
            quota: 1024,
        }
    );

    // Well over the cap also rejects, with the proposed total in `used`.
    let err_big = policy.check_upload(s, 4096).unwrap_err();
    assert_eq!(
        err_big,
        QuotaExceededError::OverQuota {
            used: 4096,
            quota: 1024,
        }
    );

    // A rejected check_upload MUST NOT advance the internal usage counter.
    assert_eq!(policy.used_for(s), 0);
}

// ---------------------------------------------------------------------------
// Scenario 3: StoragePolicy per-school quota overrides default
// ---------------------------------------------------------------------------

#[test]
fn per_school_quota_overrides_default() {
    let mut policy = StoragePolicy::new(1024);
    let (s_privileged, s_default) = two_schools();

    // Privilege s_privileged with a 4x override.
    policy.set_school_quota(s_privileged, 4096);

    // quota_for honours the override for s_privileged …
    assert_eq!(policy.quota_for(s_privileged), 4096);
    // … and falls back to the default for s_default.
    assert_eq!(policy.quota_for(s_default), 1024);

    // s_default is rejected at the default cap (1024).
    assert!(matches!(
        policy.check_upload(s_default, 2048),
        Err(QuotaExceededError::OverQuota {
            used: 2048,
            quota: 1024,
        })
    ));

    // s_privileged is allowed up to its per-school override (4096).
    assert!(policy.check_upload(s_privileged, 2048).is_ok());
    assert!(policy.check_upload(s_privileged, 4096).is_ok());

    // s_privileged is rejected at 4097 (override + 1).
    assert!(matches!(
        policy.check_upload(s_privileged, 4097),
        Err(QuotaExceededError::OverQuota {
            used: 4097,
            quota: 4096,
        })
    ));
}

// ---------------------------------------------------------------------------
// Scenario 4: StoragePolicy::record_usage updates used bytes
// ---------------------------------------------------------------------------

#[test]
fn record_usage_updates_used_bytes() {
    let mut policy = StoragePolicy::new(1024);
    let s = school();

    // Fresh state.
    assert_eq!(policy.used_for(s), 0);

    // First positive delta lands in the counter.
    policy.record_usage(s, 100).unwrap();
    assert_eq!(policy.used_for(s), 100);

    // Second positive delta accumulates.
    policy.record_usage(s, 50).unwrap();
    assert_eq!(policy.used_for(s), 150);

    // Negative delta subtracts.
    policy.record_usage(s, -50).unwrap();
    assert_eq!(policy.used_for(s), 100);

    // Exactly at the cap is allowed (100 + 924 = 1024).
    policy.record_usage(s, 924).unwrap();
    assert_eq!(policy.used_for(s), 1024);

    // One byte over the cap rejects with the would-be total.
    let err = policy.record_usage(s, 1).unwrap_err();
    assert_eq!(
        err,
        QuotaExceededError::OverQuota {
            used: 1025,
            quota: 1024,
        }
    );
    // Failed record_usage leaves the prior usage intact.
    assert_eq!(policy.used_for(s), 1024);

    // A negative delta larger than the recorded usage rejects as
    // `InvalidDelta` (underflow guard).
    let err_neg = policy.record_usage(s, -2048).unwrap_err();
    assert_eq!(err_neg, QuotaExceededError::InvalidDelta);
    assert_eq!(policy.used_for(s), 1024);
}

// ---------------------------------------------------------------------------
// Scenario 5: TenantGuard::assert_same_tenant passes for same school
// ---------------------------------------------------------------------------

#[test]
fn tenant_guard_passes_for_same_school() {
    let s = school();
    assert_eq!(TenantGuard::assert_same_tenant(s, s), Ok(()));
}

// ---------------------------------------------------------------------------
// Scenario 6: TenantGuard::assert_same_tenant rejects cross-tenant access
// ---------------------------------------------------------------------------

#[test]
fn tenant_guard_rejects_cross_tenant() {
    let (s_file, s_request) = two_schools();

    let err = TenantGuard::assert_same_tenant(s_file, s_request).unwrap_err();
    assert_eq!(
        err,
        CrossTenantError::CrossTenant {
            file: s_file,
            request: s_request,
        }
    );

    // Swapping the order is also cross-tenant (the error reports
    // whichever school was on the file vs. the request).
    let err_swapped = TenantGuard::assert_same_tenant(s_request, s_file).unwrap_err();
    assert_eq!(
        err_swapped,
        CrossTenantError::CrossTenant {
            file: s_request,
            request: s_file,
        }
    );
}
