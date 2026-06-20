//! # Notification port error type
//!
//! The error type returned by every
//! [`NotificationProvider`](crate::port::NotificationProvider)
//! implementation. Per `docs/ports/notifications.md` § "Error Type",
//! the seven variants are stable; adding a new variant is a
//! non-breaking change for trait implementors, but a breaking
//! change for callers that match exhaustively.
//!
//! The engine maps provider errors to `DomainError::Infrastructure`
//! and logs the source. Missing templates and missing variables are
//! mapped to `DomainError::Validation`.

use std::fmt;

use serde::{Deserialize, Serialize};

/// A typed identifier for a notification template owned by the
/// communication domain.
///
/// Defined as a local newtype (a wrapper around `String`) so the
/// notify port does not need to take a direct dependency on the
/// `uuid` crate. The communication domain owns the canonical
/// `NotificationTemplateId` Uuid wrapper; the port uses this
/// opaque-string view at the API boundary. Consumers that have a
/// real `Uuid` should format it as the canonical hyphenated form
/// (or any opaque string the adapter recognises) before passing
/// it across the port.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NotificationTemplateId(pub String);

impl NotificationTemplateId {
    /// Wraps an opaque template id string.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the inner id string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NotificationTemplateId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for NotificationTemplateId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for NotificationTemplateId {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

/// The error type returned by the notification port.
///
/// The derives on this enum are required by `BulkReceipt::failed`
/// in `port.rs`, which holds a `Vec<(BulkRecipientIndex,
/// NotificationError)>` and derives `Clone, PartialEq, Eq,
/// Serialize, Deserialize`. The engine never stores a live source
/// error chain across a port boundary — it logs the source via
/// `tracing` immediately and serialises only the string
/// representation, so the `Infrastructure` variant is itself a
/// `String` (not a `Box<dyn Error>`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, thiserror::Error)]
pub enum NotificationError {
    /// The requested template was not found in the tenant's
    /// template store.
    #[error("template not found: {0}")]
    TemplateNotFound(NotificationTemplateId),

    /// A required template variable was not provided in
    /// [`SendNotification::variables`](crate::port::SendNotification::variables).
    #[error("missing variable: {0}")]
    MissingVariable(String),

    /// The recipient could not be resolved (unknown id, no
    /// contact channel, etc.).
    #[error("invalid recipient: {0}")]
    InvalidRecipient(String),

    /// The adapter hit a per-tenant, per-channel rate limit. The
    /// engine retries with backoff per `docs/ports/notifications.md`
    /// § "Rate Limiting".
    #[error("rate limited")]
    RateLimited,

    /// The underlying notification provider (SES, Twilio, FCM,
    /// etc.) returned an error. The wrapped string is the
    /// provider's error message; structured detail (status code,
    /// request id) is the consumer adapter's concern.
    #[error("provider error: {0}")]
    Provider(String),

    /// The tenant has exhausted its monthly / annual quota for
    /// this channel. The engine surfaces this as a hard failure;
    /// the caller must wait for quota rollover or contact their
    /// account manager.
    #[error("quota exceeded")]
    QuotaExceeded,

    /// An underlying infrastructure error (network, TLS, DNS,
    /// serialization). The string is the source error's
    /// `Display` rendering; the engine logs the source via
    /// `tracing` and stores only the string here so the value
    /// can satisfy `Clone / Eq / Serialize / Deserialize` and
    /// flow through `BulkReceipt::failed`.
    #[error("infrastructure error: {0}")]
    Infrastructure(String),
}

impl NotificationError {
    /// Wraps an arbitrary error source as an `Infrastructure`
    /// variant. The source is rendered to its `Display` string
    /// at construction time so the stored value satisfies
    /// `Clone / Eq / Serialize / Deserialize`. Used by adapter
    /// implementations to lift `reqwest`, `tokio`, or
    /// provider-specific errors into the port's surface.
    pub fn infrastructure(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Infrastructure(source.to_string())
    }

    /// Constructs a `Provider` variant from a string message.
    #[inline]
    pub fn provider(message: impl Into<String>) -> Self {
        Self::Provider(message.into())
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
    fn template_not_found_displays_id() {
        let id = NotificationTemplateId::new("tpl_abc123");
        let e = NotificationError::TemplateNotFound(id.clone());
        assert_eq!(e.to_string(), format!("template not found: {id}"));
    }

    #[test]
    fn missing_variable_displays_name() {
        let e = NotificationError::MissingVariable("student_name".into());
        assert_eq!(e.to_string(), "missing variable: student_name");
    }

    #[test]
    fn invalid_recipient_displays_reason() {
        let e = NotificationError::InvalidRecipient("no phone on file".into());
        assert_eq!(e.to_string(), "invalid recipient: no phone on file");
    }

    #[test]
    fn rate_limited_is_bare() {
        let e = NotificationError::RateLimited;
        assert_eq!(e.to_string(), "rate limited");
    }

    #[test]
    fn provider_displays_message() {
        let e = NotificationError::provider("ses throttled");
        assert_eq!(e.to_string(), "provider error: ses throttled");
    }

    #[test]
    fn quota_exceeded_is_bare() {
        let e = NotificationError::QuotaExceeded;
        assert_eq!(e.to_string(), "quota exceeded");
    }

    #[test]
    fn infrastructure_preserves_source() {
        let inner = std::io::Error::other("net down");
        let e = NotificationError::infrastructure(inner);
        match e {
            NotificationError::Infrastructure(src) => {
                assert_eq!(src.to_string(), "net down");
            }
            other => panic!("expected Infrastructure, got {other:?}"),
        }
    }

    #[test]
    fn template_id_round_trips_string() {
        let id = NotificationTemplateId::new("tpl_xyz");
        assert_eq!(id.as_str(), "tpl_xyz");
        assert_eq!(id.to_string(), "tpl_xyz");
        let from_str: NotificationTemplateId = "tpl_abc".into();
        assert_eq!(from_str.as_str(), "tpl_abc");
    }
}