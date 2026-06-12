//! # RBAC error helpers
//!
//! The RBAC domain reuses [`DomainError`](educore_core::error::DomainError)
//! as its error type (per the engine rule "a single `DomainError`
//! enum"). This module adds RBAC-specific helper constructors for
//! the common cases the spec calls out (system role immutability,
//! missing bootstrap capability, etc.) and an alias for the
//! engine-wide `Result`.
//!
//! Per `docs/build-plan.md` and `docs/code-standards.md`, the
//! engine has a single `DomainError` enum; we do not introduce a
//! per-domain error type. RBAC-specific messages are surfaced as
//! `DomainError::Validation`, `DomainError::Conflict`, or
//! `DomainError::Forbidden`.

use educore_core::error::{DomainError, Result};

/// The RBAC-specific result alias. Equivalent to
/// [`educore_core::error::Result`].
pub type RbacResult<T> = Result<T>;

/// Returns a `Validation` error stating the role is a system role
/// and cannot be deleted.
#[must_use]
pub fn system_role_immutable() -> DomainError {
    DomainError::conflict("system roles are immutable and cannot be deleted")
}

/// Returns a `Validation` error stating the role is a system role
/// and cannot be renamed without the `RbacRoleManage` capability.
#[must_use]
pub fn system_role_rename_denied() -> DomainError {
    DomainError::forbidden("renaming a system role requires the RbacRoleManage capability")
}

/// Returns a `Forbidden` error stating the actor lacks the required
/// capability. The error kind matches the spec's filter contract:
/// `Forbidden` lets RBAC audits distinguish "missing capability"
/// from generic validation failures.
#[must_use]
pub fn missing_capability(cap: crate::value_objects::Capability) -> DomainError {
    DomainError::forbidden(format!("missing capability: {cap}"))
}

/// Returns a `Conflict` error stating the role has live user
/// bindings and cannot be deleted.
#[must_use]
pub fn role_has_bindings(count: u64) -> DomainError {
    DomainError::conflict(format!(
        "role has {count} user binding(s); unbind all users before deleting"
    ))
}

/// Returns a `Validation` error stating the role name is not unique
/// within the school.
#[must_use]
pub fn role_name_not_unique(name: &str) -> DomainError {
    DomainError::validation(format!(
        "role name {name:?} is not unique within the school"
    ))
}

/// Returns a `NotFound` error for a missing role.
#[must_use]
pub fn role_not_found() -> DomainError {
    DomainError::not_found("role not found")
}

/// Returns a `NotFound` error for a missing permission row.
#[must_use]
pub fn permission_not_found() -> DomainError {
    DomainError::not_found("permission row not found")
}

/// Returns a `Conflict` error stating the revoke would leave the
/// school without a `RbacCapabilityRevoke`-holding role.
#[must_use]
pub fn self_revocation_violation() -> DomainError {
    DomainError::conflict(
        "revoking this capability would leave the school without an \
         RbacCapabilityRevoke grant; assign the capability to another role first",
    )
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;
    use crate::value_objects::Capability;

    #[test]
    fn helpers_produce_documented_variants() {
        assert!(matches!(system_role_immutable(), DomainError::Conflict(_)));
        assert!(matches!(
            system_role_rename_denied(),
            DomainError::Forbidden(_)
        ));
        assert!(matches!(
            missing_capability(Capability::RbacRoleCreate),
            DomainError::Forbidden(_)
        ));
        assert!(matches!(role_has_bindings(3), DomainError::Conflict(_)));
        assert!(matches!(
            role_name_not_unique("Teacher"),
            DomainError::Validation(_)
        ));
        assert!(matches!(role_not_found(), DomainError::NotFound(_)));
        assert!(matches!(permission_not_found(), DomainError::NotFound(_)));
        assert!(matches!(
            self_revocation_violation(),
            DomainError::Conflict(_)
        ));
    }
}
