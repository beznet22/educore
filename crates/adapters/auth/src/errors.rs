//! # educore-auth error types
//!
//! Per `docs/ports/authentication.md` § "Error Type" the engine
//! maps every variant of [`AuthError`] to `DomainError::Forbidden`
//! for the user-facing response and logs the cause server-side.
//!
//! The enum is intentionally `Clone + PartialEq + Eq` so it can
//! flow through the engine's audit / circuit-breaker middleware
//! without losing context.
//!
//! # Deviations from `docs/ports/authentication.md`
//!
//! The crate's `Cargo.toml` is intentionally minimal (only
//! `core`, `platform`, `rbac`, `events`, `tokio`, `async-trait`),
//! so the port uses **stdlib-only** value representations:
//!
//! - The `Infrastructure` variant wraps a structured
//!   [`InfrastructureError`] (an opaque message + `Send + Sync`
//!   source chain) rather than `Box<dyn std::error::Error +
//!   Send + Sync>`. The wrapping preserves `PartialEq + Eq` on
//!   the enum so callers can match against concrete errors.
//! - The error variants are not derived via `thiserror`; the
//!   `Display` and `Error` impls are written by hand to keep the
//!   crate free of `thiserror`.
//!
//! These deviations match the `educore-payment` port pattern and
//! are documented here so future ports that gain richer
//! dependencies can adopt the spec's idiomatic `thiserror`
//! derives without changing the public surface.

use std::error::Error as StdError;
use std::fmt;

/// The full catalog of authentication errors.
///
/// Adapters return `Result<T, AuthError>` from every
/// [`crate::port::AuthProvider`] and [`crate::port::RbacPort`]
/// method. The engine maps each variant to `DomainError::Forbidden`
/// for the user and writes the cause to the audit log.
///
/// Sensitive material (passwords, MFA codes, raw tokens) is
/// **never** carried in the variant payload; the `Infrastructure`
/// variant carries a free-form message that the adapter is
/// responsible for redacting before construction.
#[derive(Debug, PartialEq, Eq)]
pub enum AuthError {
    /// The supplied credentials did not match a known principal.
    InvalidCredentials,

    /// The principal is locked out (e.g. too many failed attempts).
    /// The wrapped string is the lockout reason, suitable for
    /// the audit log. It is never shown to the user verbatim.
    AccountLocked(String),

    /// The principal exists but has been disabled by an administrator.
    AccountDisabled,

    /// The token has passed its `expires_at` and is no longer
    /// acceptable. The consumer should re-authenticate.
    Expired,

    /// The token has been explicitly revoked (by `revoke` or by a
    /// super-admin force-logout) and will not be re-issued.
    Revoked,

    /// The token is structurally invalid (bad signature, malformed
    /// JWT, unknown scheme). The wrapped string is a
    /// machine-readable reason.
    Malformed(String),

    /// The session requires a second factor to be presented
    /// before the requested operation can be authorised. The
    /// consumer should collect the second factor and re-issue
    /// the credential.
    MfaRequired,

    /// The presented second factor was rejected. The wrapped
    /// string is a machine-readable reason.
    MfaFailed(String),

    /// The caller has exceeded the configured rate limit. The
    /// consumer should back off.
    RateLimited,

    /// A non-domain infrastructure failure (network, DNS, TLS,
    /// IdP, JWKS fetch). The wrapped [`InfrastructureError`] is
    /// the underlying cause; the consumer's error-handling
    /// middleware should log it and return a generic 5xx to the
    /// user.
    Infrastructure(InfrastructureError),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCredentials => f.write_str("invalid credentials"),
            Self::AccountLocked(reason) => write!(f, "account locked: {reason}"),
            Self::AccountDisabled => f.write_str("account disabled"),
            Self::Expired => f.write_str("token expired"),
            Self::Revoked => f.write_str("token revoked"),
            Self::Malformed(reason) => write!(f, "malformed token: {reason}"),
            Self::MfaRequired => f.write_str("MFA required"),
            Self::MfaFailed(reason) => write!(f, "MFA failed: {reason}"),
            Self::RateLimited => f.write_str("rate limit exceeded"),
            Self::Infrastructure(err) => write!(f, "infrastructure error: {err}"),
        }
    }
}

impl StdError for AuthError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Infrastructure(err) => err.source(),
            _ => None,
        }
    }
}

/// An opaque, `Send + Sync` wrapper around an infrastructure-level
/// error message and an optional boxed source.
///
/// The port cannot use `Box<dyn std::error::Error + Send + Sync>`
/// directly as the variant payload because the resulting
/// [`AuthError`] enum would lose the ability to derive
/// `PartialEq + Eq` (the boxed `dyn Error` is not `Eq`). The
/// [`InfrastructureError`] wrapper preserves `PartialEq + Eq` for
/// callers that need to compare errors (e.g. in circuit-breaker
/// middleware) while still retaining the `source()` chain for
/// diagnostics.
///
/// The wrapper is intentionally **not** `Clone` (the boxed source
/// cannot be cloned). Callers that need to retain a copy of an
/// infrastructure error should clone the [`AuthError`] before the
/// `Infrastructure` variant is materialised.
#[derive(Debug)]
pub struct InfrastructureError {
    message: String,
    source: Option<Box<dyn StdError + Send + Sync>>,
}

impl InfrastructureError {
    /// Constructs an `InfrastructureError` from a free-form
    /// message. The message is stored verbatim; the adapter is
    /// responsible for redacting any sensitive data (password,
    /// MFA code, raw token) before construction.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    /// Constructs an `InfrastructureError` that wraps a boxed
    /// source error. The `Display` output of the source is
    /// appended to the engine's audit log via
    /// [`StdError::source`].
    #[must_use]
    pub fn with_source(
        message: impl Into<String>,
        source: Box<dyn StdError + Send + Sync>,
    ) -> Self {
        Self {
            message: message.into(),
            source: Some(source),
        }
    }

    /// Lifts an arbitrary `Box<dyn Error + Send + Sync>` into an
    /// `InfrastructureError`. The boxed error's `Display` output
    /// becomes the wrapped message and the box is retained as the
    /// `source` chain.
    #[must_use]
    pub fn from_boxed(err: Box<dyn StdError + Send + Sync>) -> Self {
        Self {
            message: err.to_string(),
            source: Some(err),
        }
    }

    /// Returns the wrapped diagnostic message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl PartialEq for InfrastructureError {
    fn eq(&self, other: &Self) -> bool {
        self.message == other.message
    }
}

impl Eq for InfrastructureError {}

impl fmt::Display for InfrastructureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl StdError for InfrastructureError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_ref()
            .map(|s| s.as_ref() as &(dyn StdError + 'static))
    }
}
