//! `StorageError` — the structured error type for the storage port.
//!
//! Per `docs/ports/storage.md` § Hydration Atomicity:
//!
//! > Hydration failures are surfaced as `StorageError::HydrationFailure`.
//! > The adapter does not return a partially populated aggregate. The
//! > engine maps this to `DomainError::Infrastructure`.
//!
//! This module owns the port-level error type so storage adapters can
//! produce structured failures without depending on the storage-port
//! sub-traits. The blanket `From<StorageError> for DomainError` impl
//! below lets port methods continue to return
//! `Result<T, DomainError>` while adapter code can `?`-propagate
//! `StorageError::HydrationFailure` (and any future variants).

use thiserror::Error;
use uuid::Uuid;

use educore_core::error::DomainError;

/// The structured error type produced by storage adapters when they
/// fail to honor the port contract.
///
/// Storage adapters return `Result<T, DomainError>` from every port
/// method. Internally they may construct `StorageError` values and
/// convert them via `?` thanks to the blanket
/// `From<StorageError> for DomainError` impl below. The variants
/// encode failure modes that require distinct caller behaviour;
///
/// - [`StorageError::HydrationFailure`] — the adapter could not fully
///   hydrate an aggregate from its related rows (missing join row,
///   FK violation during a batched secondary load, etc.). Per
///   `docs/ports/storage.md` § Hydration Atomicity, partial
///   hydration is **forbidden**; this variant signals the atomicity
///   violation.
#[derive(Debug, Error)]
pub enum StorageError {
    /// Aggregate hydration failed. The adapter was unable to
    /// complete every join or batched secondary load implied by the
    /// query's hydration set before returning control to the
    /// application layer.
    ///
    /// Raised when:
    ///
    /// - A related row referenced by an eager-loaded relation is
    ///   missing (orphaned FK, deleted parent, malformed join key).
    /// - The adapter's batched secondary load fails mid-flight
    ///   (network error, FK violation, schema drift).
    /// - The hydration set is internally inconsistent (for example,
    ///   a one-to-many relation reports zero rows where the FK
    ///   pointer expects a non-empty collection).
    ///
    /// Per `docs/ports/storage.md` § Hydration Atomicity, the
    /// adapter does **not** return a partially populated aggregate.
    /// The engine maps this to `DomainError::Infrastructure`.
    #[error(
        "hydration failure: aggregate_type={aggregate_type} \
         aggregate_id={aggregate_id} reason={reason}"
    )]
    HydrationFailure {
        /// The fully-qualified aggregate type (for example
        /// `"Student"`, `"Invoice"`). Used in error messages and
        /// tracing spans.
        aggregate_type: String,
        /// The aggregate's primary key (UUID). Identifies which row
        /// the adapter was attempting to hydrate.
        aggregate_id: Uuid,
        /// Human-readable failure reason. Safe to log; not
        /// guaranteed safe to surface to API callers without
        /// downstream filtering.
        reason: String,
    },
}

impl StorageError {
    /// Constructs a [`StorageError::HydrationFailure`] from the three
    /// fields the variant carries.
    #[inline]
    #[must_use]
    pub fn hydration_failure(
        aggregate_type: impl Into<String>,
        aggregate_id: Uuid,
        reason: impl Into<String>,
    ) -> Self {
        Self::HydrationFailure {
            aggregate_type: aggregate_type.into(),
            aggregate_id,
            reason: reason.into(),
        }
    }
}

/// `StorageError` converts into `DomainError::Infrastructure` so the
/// port methods can `?`-propagate adapter failures while keeping the
/// `Result<T, DomainError>` return type. The original `StorageError`
/// is preserved as the [`std::error::Error::source`] of the
/// infrastructure variant, so `tracing` and downstream error chains
/// can recover the structured cause.
impl From<StorageError> for DomainError {
    #[inline]
    fn from(e: StorageError) -> Self {
        DomainError::infrastructure(e)
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
    use std::error::Error as StdError;

    use super::*;
    use educore_core::error::ErrorKind;

    #[test]
    fn hydration_failure_carries_fields() {
        let id = Uuid::nil();
        let e = StorageError::hydration_failure("Student", id, "join miss");
        match e {
            StorageError::HydrationFailure {
                aggregate_type,
                aggregate_id,
                reason,
            } => {
                assert_eq!(aggregate_type, "Student");
                assert_eq!(aggregate_id, id);
                assert_eq!(reason, "join miss");
            }
        }
    }

    #[test]
    fn hydration_failure_display_includes_fields() {
        let id = Uuid::nil();
        let e = StorageError::hydration_failure("Invoice", id, "fk violation");
        let s = e.to_string();
        assert!(s.contains("Invoice"), "display missing aggregate_type: {s}");
        assert!(
            s.contains(&id.to_string()),
            "display missing aggregate_id: {s}"
        );
        assert!(s.contains("fk violation"), "display missing reason: {s}");
    }

    #[test]
    fn into_domain_error_is_infrastructure() {
        let e = StorageError::hydration_failure("Student", Uuid::nil(), "x");
        let d: DomainError = e.into();
        assert!(matches!(d, DomainError::Infrastructure(_)));
        assert_eq!(d.kind(), ErrorKind::Infrastructure);
    }

    #[test]
    fn source_chain_preserves_storage_error() {
        let e = StorageError::hydration_failure("Student", Uuid::nil(), "x");
        let d: DomainError = e.into();
        let source = d
            .source()
            .expect("Infrastructure must carry a source error");
        let s = source.to_string();
        assert!(
            s.contains("hydration failure"),
            "source chain lost StorageError context: {s}"
        );
    }
}
