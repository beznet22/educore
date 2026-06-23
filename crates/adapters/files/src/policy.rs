//! # Storage policy + tenant guard
//!
//! Per-school byte quota tracking and cross-tenant access enforcement
//! for the [`FileStorage`](crate::port::FileStorage) port. Closes
//! finding **ADAPTER-FILE-003** (tenant context enforcement) from
//! `docs/audit_reports/findings/wave3-files.md`.
//!
//! ## Scope
//!
//! This module is **not** wired into the [`FileStorage`](crate::port::FileStorage)
//! trait itself — the [`StoragePolicy::check_upload`] /
//! [`StoragePolicy::record_usage`] pair and the
//! [`TenantGuard::assert_same_tenant`] helper are pure value-level
//! utilities that adapters (or the consumer's upload service) call
//! **before** performing the I/O:
//!
//! ```text
//!   request.tenant.school_id  ─┐
//!                              │   ┌─ StoragePolicy::check_upload
//!   proposed_bytes_to_upload ──┼───┤   → Result<(), QuotaExceededError>
//!                              │   └─ on Ok, proceed with put()
//!                              │
//!   file.school_id          ───┼───┐
//!                              │   ├─ TenantGuard::assert_same_tenant
//!   request.tenant.school_id ──┘   │   → Result<(), CrossTenantError>
//!                                  └─ on Err, return PermissionDenied
//! ```
//!
//! The two checks are intentionally independent so adapters can
//! apply them at different layers (the quota check belongs in the
//! write path; the tenant check belongs on every read and write).
//!
//! ## State
//!
//! [`StoragePolicy`] owns its usage counter internally — adapters
//! do not need to thread the running total through the call site.
//! The counter is private; consult [`StoragePolicy::used_for`] for
//! read-only diagnostics. Callers that need to reset a school's
//! usage (e.g. on a fresh academic year) drop and re-add the
//! school via [`StoragePolicy::set_school_quota`] + a follow-up
//! [`StoragePolicy::record_usage`] with the new baseline.
//!
//! ## Errors
//!
//! Both [`QuotaExceededError`] and [`CrossTenantError`] implement
//! [`std::error::Error`] and carry the failure context in their
//! payload (used vs. quota; file school vs. request school). They
//! do **not** depend on `thiserror` (the crate is intentionally
//! minimal — see [`crate::errors`] for the same convention).

use std::collections::HashMap;
use std::error::Error as StdError;
use std::fmt;

use educore_core::ids::SchoolId;

/// Per-school storage quota policy.
///
/// Holds a default byte quota applied to every school that has no
/// explicit override, plus an optional map of per-school quota
/// overrides. Tracks usage internally so that both
/// [`check_upload`](Self::check_upload) and
/// [`record_usage`](Self::record_usage) can enforce the cap.
///
/// The struct is `Clone` so a long-lived policy can be snapshotted
/// for an audit-log entry without consuming the live instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoragePolicy {
    /// Default byte quota applied when no per-school override
    /// exists for the school in [`Self::per_school_quota_bytes`].
    pub default_quota_bytes: u64,

    /// Per-school byte quota overrides. A school's quota is the
    /// value stored here when present, otherwise
    /// [`Self::default_quota_bytes`].
    pub per_school_quota_bytes: HashMap<SchoolId, u64>,

    /// Current tracked usage per school. Private — read via
    /// [`Self::used_for`].
    used_bytes: HashMap<SchoolId, u64>,
}

impl Default for StoragePolicy {
    /// A **zero-quota** default — the safest starting state, since
    /// it rejects every upload until a non-zero quota is
    /// configured via [`Self::new`] or [`Self::set_school_quota`].
    fn default() -> Self {
        Self {
            default_quota_bytes: 0,
            per_school_quota_bytes: HashMap::new(),
            used_bytes: HashMap::new(),
        }
    }
}

impl StoragePolicy {
    /// Constructs a new policy with the given default byte quota
    /// and no per-school overrides. All schools start with zero
    /// bytes used.
    #[must_use]
    pub fn new(default_quota_bytes: u64) -> Self {
        Self {
            default_quota_bytes,
            per_school_quota_bytes: HashMap::new(),
            used_bytes: HashMap::new(),
        }
    }

    /// Adds or replaces the per-school quota override for the
    /// given school. Subsequent calls to
    /// [`quota_for`](Self::quota_for) return this value instead of
    /// [`Self::default_quota_bytes`].
    pub fn set_school_quota(&mut self, school_id: SchoolId, quota_bytes: u64) {
        self.per_school_quota_bytes.insert(school_id, quota_bytes);
    }

    /// Returns the effective byte quota for the given school: the
    /// per-school override if present, otherwise the default.
    #[must_use]
    pub fn quota_for(&self, school_id: SchoolId) -> u64 {
        self.per_school_quota_bytes
            .get(&school_id)
            .copied()
            .unwrap_or(self.default_quota_bytes)
    }

    /// Returns the current tracked usage for the given school, in
    /// bytes. Returns `0` if the school has no recorded usage.
    #[must_use]
    pub fn used_for(&self, school_id: SchoolId) -> u64 {
        self.used_bytes.get(&school_id).copied().unwrap_or(0)
    }

    /// Checks whether uploading `new_total_bytes` would exceed the
    /// school's quota. This is a **pure** check — it does NOT
    /// update the internal usage counter. Callers that want to
    /// deduct the upload should follow up with
    /// [`record_usage`](Self::record_usage) on success.
    ///
    /// # Errors
    ///
    /// - [`QuotaExceededError::OverQuota`] if `new_total_bytes`
    ///   strictly exceeds the school's effective quota.
    pub fn check_upload(
        &self,
        school_id: SchoolId,
        new_total_bytes: u64,
    ) -> Result<(), QuotaExceededError> {
        let quota = self.quota_for(school_id);
        if new_total_bytes > quota {
            return Err(QuotaExceededError::OverQuota {
                used: new_total_bytes,
                quota,
            });
        }
        Ok(())
    }

    /// Records a usage delta for the given school and enforces the
    /// quota on the new total. Positive deltas add to the school's
    /// usage; negative deltas subtract (e.g. on delete).
    ///
    /// The internal usage counter is updated **only on success**;
    /// a failed call leaves the previous usage intact, so the
    /// caller can retry without first having to undo a partial
    /// write.
    ///
    /// # Errors
    ///
    /// - [`QuotaExceededError::OverQuota`] if the resulting usage
    ///   strictly exceeds the school's quota. The `used` field
    ///   reports the would-be new total.
    /// - [`QuotaExceededError::InvalidDelta`] if `delta_bytes`
    ///   would underflow the current usage below zero (negative
    ///   delta larger than the recorded usage) or overflow the
    ///   `u64` counter (positive delta that exceeds `u64::MAX -
    ///   current`).
    pub fn record_usage(
        &mut self,
        school_id: SchoolId,
        delta_bytes: i64,
    ) -> Result<(), QuotaExceededError> {
        let current = self.used_for(school_id);
        let quota = self.quota_for(school_id);

        // `unsigned_abs` is the only safe conversion: it never
        // panics (unlike `as u64` on `i64::MIN`) and never
        // truncates. `i64::MIN.unsigned_abs() == 2^63`, which is
        // representable in `u64`.
        let new_used: u64 = if delta_bytes >= 0 {
            let delta_u =
                u64::try_from(delta_bytes).map_err(|_| QuotaExceededError::InvalidDelta)?;
            current
                .checked_add(delta_u)
                .ok_or(QuotaExceededError::InvalidDelta)?
        } else {
            let abs_delta = delta_bytes.unsigned_abs();
            current
                .checked_sub(abs_delta)
                .ok_or(QuotaExceededError::InvalidDelta)?
        };

        if new_used > quota {
            return Err(QuotaExceededError::OverQuota {
                used: new_used,
                quota,
            });
        }

        self.used_bytes.insert(school_id, new_used);
        Ok(())
    }
}

/// Returned by [`StoragePolicy::check_upload`] and
/// [`StoragePolicy::record_usage`] when an operation would exceed
/// the school's storage quota or when the supplied delta is not
/// representable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuotaExceededError {
    /// The proposed new total exceeds the school's quota.
    /// `used` is the would-be new total (after the upload), and
    /// `quota` is the school's effective quota.
    OverQuota {
        /// The would-be new total in bytes.
        used: u64,
        /// The school's effective quota in bytes.
        quota: u64,
    },
    /// The delta was not representable: the positive delta would
    /// overflow the `u64` counter, the negative delta would
    /// underflow the current usage below zero, or the absolute
    /// value was not representable as `u64`.
    InvalidDelta,
}

impl fmt::Display for QuotaExceededError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OverQuota { used, quota } => {
                write!(f, "storage quota exceeded: {used} bytes > {quota} bytes")
            }
            Self::InvalidDelta => f.write_str("invalid storage usage delta"),
        }
    }
}

impl StdError for QuotaExceededError {}

/// Validates cross-tenant access for file operations.
///
/// A typed wrapper around the single
/// [`assert_same_tenant`](Self::assert_same_tenant) function so
/// adapters and the engine's upload service can share a single
/// call site. The struct itself carries no state — the call site
/// looks like
/// `TenantGuard::assert_same_tenant(file_school, request_school)?`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TenantGuard;

impl TenantGuard {
    /// Asserts that the file's school matches the request's
    /// school. Returns `Ok(())` if they match; otherwise returns
    /// [`CrossTenantError::CrossTenant`] carrying both ids for
    /// diagnostic / audit-log purposes.
    ///
    /// # Errors
    ///
    /// - [`CrossTenantError::CrossTenant`] if the file's school
    ///   differs from the request's school.
    pub fn assert_same_tenant(
        file_school: SchoolId,
        request_school: SchoolId,
    ) -> Result<(), CrossTenantError> {
        if file_school == request_school {
            Ok(())
        } else {
            Err(CrossTenantError::CrossTenant {
                file: file_school,
                request: request_school,
            })
        }
    }
}

/// Returned by [`TenantGuard::assert_same_tenant`] when the
/// file's tenant does not match the requester's tenant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossTenantError {
    /// Cross-tenant access was attempted. `file` is the school
    /// that owns the file; `request` is the school on the
    /// requester's context.
    CrossTenant {
        /// The school that owns the file.
        file: SchoolId,
        /// The school on the requester's context.
        request: SchoolId,
    },
}

impl fmt::Display for CrossTenantError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CrossTenant { file, request } => write!(
                f,
                "cross-tenant access denied: file belongs to school {file}, request from school {request}"
            ),
        }
    }
}

impl StdError for CrossTenantError {}
