//! Engine-wide error type.
//!
//! Per `docs/code-standards.md` § 11: every public fallible function
//! returns `Result<T, DomainError>`. The engine has a single
//! `DomainError` enum with a `kind` discriminant
//! (`Validation`, `NotFound`, `Conflict`, `Forbidden`,
//! `Infrastructure`). Domain errors pass through unchanged; the
//! command dispatcher converts infrastructure errors into a generic
//! `Infrastructure` variant.
//!
//! Tests assert on variants, not on display strings.

use std::fmt;
use thiserror::Error;

/// The engine's universal error type. Public APIs return
/// `Result<T, DomainError>`.
#[derive(Debug, Error)]
pub enum DomainError {
    /// The input failed structural, reference, or business-rule
    /// validation. The string is a human-readable reason safe to
    /// surface to API callers (it MUST NOT contain PII, secrets,
    /// stack traces, or internal paths).
    #[error("validation: {0}")]
    Validation(String),

    /// The referenced aggregate, row, or external resource does not
    /// exist.
    #[error("not found: {0}")]
    NotFound(String),

    /// The operation conflicts with the current state of the
    /// aggregate (optimistic-concurrency mismatch, version stale,
    /// already-closed invoice, etc.).
    #[error("conflict: {0}")]
    Conflict(String),

    /// The actor lacks the capability required for this operation.
    /// Distinct from `Validation` so RBAC audits can be filtered.
    #[error("forbidden: {0}")]
    Forbidden(String),

    /// The command's `school_id` does not match the caller's
    /// `TenantContext::school_id`. Surfaced separately from
    /// `Forbidden` so cross-tenant intrusion attempts can be
    /// detected and reported.
    #[error("tenant violation: {0}")]
    TenantViolation(String),

    /// The storage adapter or external port returned an
    /// infrastructure-level error. The wrapped error preserves the
    /// source for `tracing` while the variant keeps the public API
    /// stable.
    #[error("infrastructure: {0}")]
    Infrastructure(#[source] Box<dyn std::error::Error + Send + Sync>),

    /// The requested capability is not supported by the wired
    /// adapter. Storage adapters use this for the four
    /// sync-primitive methods that return `NotSupported` when the
    /// adapter is not sync-capable (per ADR-017/018).
    #[error("not supported: {0}")]
    NotSupported(String),
}

impl DomainError {
    /// Returns a stable, machine-readable discriminant for the
    /// error. Useful for telemetry and for callers that need to
    /// branch on the kind without matching each variant.
    #[must_use]
    pub const fn kind(&self) -> ErrorKind {
        match self {
            Self::Validation(_) | Self::NotSupported(_) => ErrorKind::Validation,
            Self::NotFound(_) => ErrorKind::NotFound,
            Self::Conflict(_) => ErrorKind::Conflict,
            Self::Forbidden(_) => ErrorKind::Forbidden,
            Self::TenantViolation(_) => ErrorKind::TenantViolation,
            Self::Infrastructure(_) => ErrorKind::Infrastructure,
        }
    }

    /// Returns the human-readable reason, if the variant carries
    /// one. Returns `None` for `Infrastructure` (the source error's
    /// own message is the canonical reason).
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        match self {
            Self::Validation(m)
            | Self::NotFound(m)
            | Self::Conflict(m)
            | Self::Forbidden(m)
            | Self::TenantViolation(m)
            | Self::NotSupported(m) => Some(m),
            Self::Infrastructure(_) => None,
        }
    }

    /// Constructs a `Validation` error from a static reason.
    #[inline]
    pub fn validation(reason: impl Into<String>) -> Self {
        Self::Validation(reason.into())
    }

    /// Constructs a `NotFound` error from a static reason.
    #[inline]
    pub fn not_found(reason: impl Into<String>) -> Self {
        Self::NotFound(reason.into())
    }

    /// Constructs a `Conflict` error from a static reason.
    #[inline]
    pub fn conflict(reason: impl Into<String>) -> Self {
        Self::Conflict(reason.into())
    }

    /// Constructs a `Forbidden` error from a static reason.
    #[inline]
    pub fn forbidden(reason: impl Into<String>) -> Self {
        Self::Forbidden(reason.into())
    }

    /// Constructs a `TenantViolation` error from a static reason.
    #[inline]
    pub fn tenant_violation(reason: impl Into<String>) -> Self {
        Self::TenantViolation(reason.into())
    }

    /// Constructs an `Infrastructure` error wrapping a
    /// `Send + Sync` source error.
    #[inline]
    pub fn infrastructure(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Infrastructure(Box::new(source))
    }

    /// Constructs a `NotSupported` error from a static reason.
    #[inline]
    pub fn not_supported(reason: impl Into<String>) -> Self {
        Self::NotSupported(reason.into())
    }
}

/// Stable, machine-readable discriminant for [`DomainError`]. The
/// set of variants is closed; new kinds require a major version
/// bump.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    /// Validation failed, or the operation is unsupported.
    Validation,
    /// The referenced resource does not exist.
    NotFound,
    /// The operation conflicts with the current state.
    Conflict,
    /// The actor lacks the required capability.
    Forbidden,
    /// Cross-tenant operation was attempted without authorization.
    TenantViolation,
    /// An infrastructure-level error occurred.
    Infrastructure,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Validation => "validation",
            Self::NotFound => "not_found",
            Self::Conflict => "conflict",
            Self::Forbidden => "forbidden",
            Self::TenantViolation => "tenant_violation",
            Self::Infrastructure => "infrastructure",
        };
        f.write_str(s)
    }
}

/// Engine-wide `Result` alias. Every fallible API in the engine
/// returns `Result<T, DomainError>`.
pub type Result<T> = std::result::Result<T, DomainError>;

impl From<String> for DomainError {
    #[inline]
    fn from(s: String) -> Self {
        Self::Validation(s)
    }
}

impl From<&str> for DomainError {
    #[inline]
    fn from(s: &str) -> Self {
        Self::Validation(s.to_owned())
    }
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

    #[test]
    fn kind_is_stable_per_variant() {
        assert_eq!(DomainError::validation("x").kind(), ErrorKind::Validation);
        assert_eq!(DomainError::not_found("x").kind(), ErrorKind::NotFound);
        assert_eq!(DomainError::conflict("x").kind(), ErrorKind::Conflict);
        assert_eq!(DomainError::forbidden("x").kind(), ErrorKind::Forbidden);
        assert_eq!(
            DomainError::tenant_violation("x").kind(),
            ErrorKind::TenantViolation
        );
        assert_eq!(
            DomainError::infrastructure(std::io::Error::other("x")).kind(),
            ErrorKind::Infrastructure
        );
        assert_eq!(
            DomainError::not_supported("x").kind(),
            ErrorKind::Validation
        );
    }

    #[test]
    fn message_is_some_for_carriers() {
        let e = DomainError::validation("bad input");
        assert_eq!(e.message(), Some("bad input"));
    }

    #[test]
    fn message_is_none_for_infrastructure() {
        let e = DomainError::infrastructure(std::io::Error::other("disk full"));
        assert_eq!(e.message(), None);
    }

    #[test]
    fn from_string_is_validation() {
        let e: DomainError = "oops".into();
        assert!(matches!(e, DomainError::Validation(_)));
    }

    #[test]
    fn infrastructure_preserves_source() {
        let src = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "nope");
        let e = DomainError::infrastructure(src);
        match e {
            DomainError::Infrastructure(boxed) => {
                let s = boxed.downcast_ref::<std::io::Error>().unwrap();
                assert_eq!(s.kind(), std::io::ErrorKind::PermissionDenied);
            }
            _ => panic!("expected Infrastructure"),
        }
    }

    #[test]
    fn display_includes_message() {
        let e = DomainError::validation("negative count");
        assert_eq!(e.to_string(), "validation: negative count");
    }
}
