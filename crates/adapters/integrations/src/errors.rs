//! # Integration port errors
//!
//! The [`IntegrationError`] enum is the universal failure type for
//! every [`IntegrationGateway`](crate::port::IntegrationGateway)
//! implementation. The variants are exhaustive — adapters MUST
//! not invent new ones; they MUST map provider-specific failures
//! into the closest existing variant. If a provider failure has
//! no good fit, use [`IntegrationError::Provider`] with a
//! descriptive message; use [`IntegrationError::Infrastructure`]
//! only for plumbing-level errors (network, DNS, TLS, serialization)
//! that originate below the provider's HTTP boundary.
//!
//! Per `docs/ports/integrations.md` § "Error Type" and ADR-015
//! (external crate selection): port-adapter code is allowed to use
//! `serde_json::Value` and other JSON-shaped types; this crate is
//! **not** a domain crate and is therefore exempt from the
//! "no `serde_json::Value` in domain code" rule that applies to
//! `crates/domains/`.
//!
//! ## Serialization
//!
//! The `Infrastructure` variant carries `Box<dyn StdError + Send +
//! Sync>` for diagnostic logging. That trait object is not
//! `Serialize` / `Deserialize` / `Clone` by default, so this module
//! provides manual impls that round-trip the inner error's
//! `Display` string. The `Clone` impl is lossy on the
//! `Infrastructure` variant: the cloned error loses its `source()`
//! chain and keeps only the rendered message. This is acceptable
//! because cloned errors are used for log enrichment, never for
//! re-raising.

use std::error::Error as StdError;

use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::ser::{Serialize, Serializer};
use serde::de::{EnumAccess, VariantAccess};
use std::fmt;
use thiserror::Error;

use crate::port::IntegrationId;

/// The universal [`Result`] alias for [`IntegrationGateway`](crate::port::IntegrationGateway)
/// implementations: `Result<T, IntegrationError>`.
pub type Result<T> = std::result::Result<T, IntegrationError>;

/// The closed set of failure modes a single integration invocation
/// can produce.
///
/// Variants:
///
/// - [`NotConfigured`](IntegrationError::NotConfigured) — the
///   tenant has not registered the integration. The adapter has no
///   credentials, webhook URL, or other configuration to act on.
/// - [`NotFound`](IntegrationError::NotFound) — the integration is
///   configured but the addressed resource (e.g. a remote course
///   id) does not exist.
/// - [`InvalidInput`](IntegrationError::InvalidInput) — the
///   `IntegrationRequest::input` payload failed local validation
///   (schema mismatch, missing required field, wrong type). The
///   provider was never contacted.
/// - [`RateLimited`](IntegrationError::RateLimited) — the provider
///   returned `429 Too Many Requests` (or its async equivalent).
///   Callers should back off and retry per the adapter's
///   [`RetryPolicy`](crate::port::RetryPolicy).
/// - [`Timeout`](IntegrationError::Timeout) — the adapter exhausted
///   its budget without a response. Distinguished from a provider
///   timeout so callers can tell who gave up.
/// - [`Provider`](IntegrationError::Provider) — the provider
///   returned an error response (`4xx`/`5xx` other than the
///   ones above) or a non-recoverable business-logic rejection.
/// - [`Infrastructure`](IntegrationError::Infrastructure) — the
///   adapter could not reach the provider at all (DNS, TCP, TLS,
///   serialization). Carries the underlying error as a `source`
///   for diagnostic logging.
#[derive(Debug, Error)]
pub enum IntegrationError {
    /// Integration is not configured for the active tenant.
    #[error("integration not configured: {0}")]
    NotConfigured(IntegrationId),

    /// The integration is configured but the addressed resource
    /// does not exist on the provider side.
    #[error("integration not found: {0}")]
    NotFound(IntegrationId),

    /// The `IntegrationRequest::input` payload is malformed against
    /// the action's [`SchemaRef`](crate::port::SchemaRef). The
    /// provider was not contacted.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// The provider signalled rate limiting. Callers should back
    /// off and retry.
    #[error("rate limited")]
    RateLimited,

    /// The adapter exhausted its `IntegrationRequest::timeout`
    /// budget without a response.
    #[error("timeout after {0:?}")]
    Timeout(chrono::Duration),

    /// The provider returned an error response. The string is
    /// provider-specific (HTTP status + body snippet, error code,
    /// etc.) and SHOULD NOT be parsed by callers.
    #[error("provider error: {0}")]
    Provider(String),

    /// The adapter could not reach the provider (network, DNS,
    /// TLS, serialization). Carries the underlying error as
    /// `source` for diagnostic logging.
    #[error("infrastructure error: {0}")]
    Infrastructure(#[source] Box<dyn StdError + Send + Sync>),
}

impl IntegrationError {
    /// Returns `true` if the variant is one a caller should retry
    /// (after applying the adapter's [`RetryPolicy`](crate::port::RetryPolicy)
    /// and backoff).
    #[must_use]
    pub const fn is_retryable(&self) -> bool {
        matches!(self, Self::RateLimited | Self::Timeout(_) | Self::Infrastructure(_))
    }

    /// Returns `true` if the variant represents a configuration
    /// problem the caller can fix (missing integration, bad input).
    #[must_use]
    pub const fn is_configuration_error(&self) -> bool {
        matches!(self, Self::NotConfigured(_) | Self::NotFound(_) | Self::InvalidInput(_))
    }
}

// =============================================================================
// Manual Clone
//
// The `Infrastructure` variant carries a `Box<dyn StdError + Send + Sync>`,
// which is not `Clone`. We round-trip through `Display` and a freshly-
// boxed `std::io::Error::other` so cloned errors keep their message.
// The `source()` chain is lost on the cloned error; this is documented
// above and matches the "for log enrichment" use case.
// =============================================================================
impl Clone for IntegrationError {
    fn clone(&self) -> Self {
        match self {
            Self::NotConfigured(id) => Self::NotConfigured(id.clone()),
            Self::NotFound(id) => Self::NotFound(id.clone()),
            Self::InvalidInput(s) => Self::InvalidInput(s.clone()),
            Self::RateLimited => Self::RateLimited,
            Self::Timeout(d) => Self::Timeout(*d),
            Self::Provider(s) => Self::Provider(s.clone()),
            Self::Infrastructure(err) => {
                let msg = err.to_string();
                Self::Infrastructure(Box::new(std::io::Error::other(msg)))
            }
        }
    }
}

// =============================================================================
// Manual Serialize / Deserialize
//
// The wire form serializes `Infrastructure` as a single string field
// (the inner error's `Display` rendering). All other variants keep
// their natural single-field tuple shape. Discriminator: the
// variant name as `serde`'s default "externally tagged" enum tag.
// =============================================================================
impl Serialize for IntegrationError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::NotConfigured(id) => {
                serializer.serialize_newtype_variant("IntegrationError", 0, "NotConfigured", id)
            }
            Self::NotFound(id) => {
                serializer.serialize_newtype_variant("IntegrationError", 1, "NotFound", id)
            }
            Self::InvalidInput(s) => {
                serializer.serialize_newtype_variant("IntegrationError", 2, "InvalidInput", s)
            }
            Self::RateLimited => {
                serializer.serialize_unit_variant("IntegrationError", 3, "RateLimited")
            }
            Self::Timeout(d) => {
                serializer.serialize_newtype_variant("IntegrationError", 4, "Timeout", d)
            }
            Self::Provider(s) => {
                serializer.serialize_newtype_variant("IntegrationError", 5, "Provider", s)
            }
            Self::Infrastructure(err) => {
                let msg = err.to_string();
                serializer.serialize_newtype_variant("IntegrationError", 6, "Infrastructure", &msg)
            }
        }
    }
}

impl<'de> Deserialize<'de> for IntegrationError {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug)]
        enum Field {
            NotConfigured,
            NotFound,
            InvalidInput,
            RateLimited,
            Timeout,
            Provider,
            Infrastructure,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                        formatter.write_str(
                            "variant identifier (NotConfigured, NotFound, InvalidInput, \
                             RateLimited, Timeout, Provider, Infrastructure)",
                        )
                    }

                    fn visit_str<E>(self, value: &str) -> std::result::Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "NotConfigured" => Ok(Field::NotConfigured),
                            "NotFound" => Ok(Field::NotFound),
                            "InvalidInput" => Ok(Field::InvalidInput),
                            "RateLimited" => Ok(Field::RateLimited),
                            "Timeout" => Ok(Field::Timeout),
                            "Provider" => Ok(Field::Provider),
                            "Infrastructure" => Ok(Field::Infrastructure),
                            other => Err(de::Error::unknown_variant(other, VARIANTS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        const VARIANTS: &[&str] = &[
            "NotConfigured",
            "NotFound",
            "InvalidInput",
            "RateLimited",
            "Timeout",
            "Provider",
            "Infrastructure",
        ];

        struct EnumVisitor;
        impl<'de> Visitor<'de> for EnumVisitor {
            type Value = IntegrationError;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("IntegrationError")
            }

            fn visit_enum<A>(self, data: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                match data.variant()? {
                    (Field::NotConfigured, v) => v
                        .newtype_variant::<IntegrationId>()
                        .map(IntegrationError::NotConfigured),
                    (Field::NotFound, v) => v
                        .newtype_variant::<IntegrationId>()
                        .map(IntegrationError::NotFound),
                    (Field::InvalidInput, v) => v
                        .newtype_variant::<String>()
                        .map(IntegrationError::InvalidInput),
                    (Field::RateLimited, v) => v.unit_variant().map(|()| IntegrationError::RateLimited),
                    (Field::Timeout, v) => v
                        .newtype_variant::<chrono::Duration>()
                        .map(IntegrationError::Timeout),
                    (Field::Provider, v) => v
                        .newtype_variant::<String>()
                        .map(IntegrationError::Provider),
                    (Field::Infrastructure, v) => v
                        .newtype_variant::<String>()
                        .map(|msg| {
                            IntegrationError::Infrastructure(Box::new(std::io::Error::other(msg)))
                        }),
                }
            }
        }

        deserializer.deserialize_enum("IntegrationError", VARIANTS, EnumVisitor)
    }
}

// ============================================================================
// Silence the unused-imports lint for the imports that turned out not to be
// needed once the manual impls above replaced the derives.
// ============================================================================
#[allow(dead_code)]
fn _ensure_traits_used() {
    fn _needs_de<T>(_: &T)
    where
        T: serde::de::DeserializeOwned,
    {
    }
    fn _needs_ser<T>(_: &T)
    where
        T: serde::Serialize,
    {
    }
    let _: fn(&IntegrationError) = |e| {
        _needs_de(e);
        _needs_ser(e);
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id() -> IntegrationId {
        IntegrationId::new("twilio")
    }

    #[test]
    fn retryable_classification_is_correct() {
        assert!(IntegrationError::RateLimited.is_retryable());
        assert!(IntegrationError::Timeout(chrono::Duration::seconds(5)).is_retryable());
        assert!(IntegrationError::Infrastructure(Box::new(std::io::Error::other("net"))).is_retryable());
        assert!(!IntegrationError::Provider("400".into()).is_retryable());
        assert!(!IntegrationError::InvalidInput("bad".into()).is_retryable());
    }

    #[test]
    fn configuration_error_classification_is_correct() {
        assert!(IntegrationError::NotConfigured(id()).is_configuration_error());
        assert!(IntegrationError::NotFound(id()).is_configuration_error());
        assert!(IntegrationError::InvalidInput("missing".into()).is_configuration_error());
        assert!(!IntegrationError::Provider("500".into()).is_configuration_error());
        assert!(!IntegrationError::RateLimited.is_configuration_error());
    }

    #[test]
    fn error_messages_match_spec() {
        assert_eq!(
            IntegrationError::NotConfigured(id()).to_string(),
            "integration not configured: twilio"
        );
        assert_eq!(
            IntegrationError::NotFound(id()).to_string(),
            "integration not found: twilio"
        );
        assert_eq!(
            IntegrationError::InvalidInput("oops".into()).to_string(),
            "invalid input: oops"
        );
        assert_eq!(IntegrationError::RateLimited.to_string(), "rate limited");
        assert_eq!(
            IntegrationError::Timeout(chrono::Duration::seconds(10)).to_string(),
            "timeout after 10s"
        );
        assert_eq!(
            IntegrationError::Provider("400 bad request".into()).to_string(),
            "provider error: 400 bad request"
        );
    }

    #[test]
    fn infrastructure_error_carries_source() {
        let inner = std::io::Error::other("connection reset");
        let err = IntegrationError::Infrastructure(Box::new(inner));
        let source = err.source().expect("infrastructure variant carries a source");
        assert!(source.to_string().contains("connection reset"));
    }

    #[test]
    fn infrastructure_error_clone_is_lossy_but_preserves_message() {
        let err = IntegrationError::Infrastructure(Box::new(std::io::Error::other("net fail")));
        let cloned = err.clone();
        assert_eq!(cloned.to_string(), "infrastructure error: net fail");
    }

    #[test]
    fn round_trip_through_json() {
        for original in [
            IntegrationError::NotConfigured(id()),
            IntegrationError::NotFound(id()),
            IntegrationError::InvalidInput("bad payload".into()),
            IntegrationError::RateLimited,
            IntegrationError::Timeout(chrono::Duration::seconds(7)),
            IntegrationError::Provider("400".into()),
            IntegrationError::Infrastructure(Box::new(std::io::Error::other("dns"))),
        ] {
            let json = serde_json::to_string(&original).expect("serialize");
            let parsed: IntegrationError =
                serde_json::from_str(&json).expect("deserialize");
            assert_eq!(parsed.to_string(), original.to_string());
        }
    }
}