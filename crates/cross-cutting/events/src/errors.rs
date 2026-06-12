//! # Event error type
//!
//! The error type returned by [`EventBus`](crate::event_bus::EventBus)
//! and [`EventSubscription`](crate::event_bus::EventSubscription)
//! implementations. Per the bus port, the variants are stable;
//! adding a new variant is a non-breaking change for trait
//! implementors, but a breaking change for callers that match
//! exhaustively.

use thiserror::Error;

use educore_core::error::DomainError;

use crate::event_bus::Topic;

/// The error type returned by the bus port.
#[derive(Debug, Error)]
pub enum EventError {
    /// The topic string could not be parsed or resolved.
    #[error("topic not found: {0:?}")]
    TopicNotFound(Topic),

    /// The subscription is closed.
    #[error("subscription closed")]
    SubscriptionClosed,

    /// The bus rejected a publish.
    #[error("publish failed: {0}")]
    PublishFailed(String),

    /// The payload could not be (de)serialised.
    #[error("deserialize failed: {0}")]
    DeserializeFailed(String),

    /// The requested operation is not supported by the wired
    /// adapter. Stub adapters (NATS, Redis, Kafka, ...) use this
    /// for methods that require a live connection until the
    /// wire-protocol impl lands in a future phase.
    #[error("not supported: {0}")]
    NotSupported(String),

    /// An underlying infrastructure error.
    #[error("infrastructure error: {0}")]
    Infrastructure(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl EventError {
    /// Wraps an arbitrary error source as an `Infrastructure`
    /// variant.
    pub fn infrastructure(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Infrastructure(Box::new(source))
    }

    /// Constructs a `NotSupported` variant from a static reason.
    #[inline]
    pub fn not_supported(reason: impl Into<String>) -> Self {
        Self::NotSupported(reason.into())
    }
}

impl From<EventError> for DomainError {
    /// Maps the bus-port `EventError` into the engine-wide
    /// `DomainError`. The mapping is total; lossy variants
    /// (`TopicNotFound`, `SubscriptionClosed`, `PublishFailed`,
    /// `DeserializeFailed`) drop their structured payload in
    /// favour of a human-readable message. `Infrastructure`
    /// preserves the source; `NotSupported` is forwarded.
    fn from(err: EventError) -> Self {
        match err {
            EventError::TopicNotFound(t) => {
                DomainError::NotFound(format!("topic not found: {t:?}"))
            }
            EventError::SubscriptionClosed => {
                DomainError::Conflict("subscription closed".to_owned())
            }
            EventError::PublishFailed(msg) => {
                DomainError::Infrastructure(EventErrorMessage::boxed("publish failed", msg))
            }
            EventError::DeserializeFailed(msg) => {
                DomainError::Infrastructure(EventErrorMessage::boxed("deserialize failed", msg))
            }
            EventError::NotSupported(msg) => DomainError::not_supported(msg),
            EventError::Infrastructure(src) => DomainError::Infrastructure(src),
        }
    }
}

/// A `String`-backed `std::error::Error` impl used by the
/// `From<EventError> for DomainError` conversion to carry the
/// variant's reason string across the `DomainError::Infrastructure`
/// boundary without losing context.
#[derive(Debug)]
struct EventErrorMessage {
    kind: &'static str,
    message: String,
}

impl EventErrorMessage {
    fn boxed(kind: &'static str, message: String) -> Box<dyn std::error::Error + Send + Sync> {
        Box::new(Self { kind, message })
    }
}

impl std::fmt::Display for EventErrorMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl std::error::Error for EventErrorMessage {}

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
    fn display_messages_are_stable() {
        let e = EventError::SubscriptionClosed;
        assert_eq!(e.to_string(), "subscription closed");
        let e = EventError::PublishFailed("bus down".to_owned());
        assert_eq!(e.to_string(), "publish failed: bus down");
        let e = EventError::DeserializeFailed("bad json".to_owned());
        assert_eq!(e.to_string(), "deserialize failed: bad json");
        let e = EventError::not_supported("nats publish");
        assert_eq!(e.to_string(), "not supported: nats publish");
    }

    #[test]
    fn infrastructure_wraps_source() {
        let inner = std::io::Error::other("boom");
        let e = EventError::infrastructure(inner);
        match e {
            EventError::Infrastructure(src) => {
                assert_eq!(src.to_string(), "boom");
            }
            other => panic!("expected Infrastructure, got {other:?}"),
        }
    }

    #[test]
    fn not_supported_constructor_preserves_message() {
        let e = EventError::not_supported("kafka subscribe");
        match e {
            EventError::NotSupported(msg) => assert_eq!(msg, "kafka subscribe"),
            other => panic!("expected NotSupported, got {other:?}"),
        }
    }

    #[test]
    fn maps_to_domain_error() {
        // NotSupported forwards verbatim.
        let d: DomainError = EventError::not_supported("x").into();
        assert!(matches!(d, DomainError::NotSupported(ref m) if m == "x"));

        // TopicNotFound maps to NotFound.
        use educore_core::ids::Identifier;
        let school = educore_core::ids::SchoolId::from_uuid(uuid::Uuid::nil());
        let d: DomainError =
            EventError::TopicNotFound(crate::event_bus::Topic::Tenant(school)).into();
        assert!(matches!(d, DomainError::NotFound(_)));

        // SubscriptionClosed maps to Conflict.
        let d: DomainError = EventError::SubscriptionClosed.into();
        assert!(matches!(d, DomainError::Conflict(_)));

        // PublishFailed / DeserializeFailed map to Infrastructure preserving the message.
        let d: DomainError = EventError::PublishFailed("bus down".to_owned()).into();
        match d {
            DomainError::Infrastructure(src) => assert!(src.to_string().contains("bus down")),
            other => panic!("expected Infrastructure, got {other:?}"),
        }
        let d: DomainError = EventError::DeserializeFailed("bad json".to_owned()).into();
        match d {
            DomainError::Infrastructure(src) => assert!(src.to_string().contains("bad json")),
            other => panic!("expected Infrastructure, got {other:?}"),
        }

        // Infrastructure is forwarded.
        let inner = std::io::Error::other("net down");
        let d: DomainError = EventError::infrastructure(inner).into();
        match d {
            DomainError::Infrastructure(src) => assert_eq!(src.to_string(), "net down"),
            other => panic!("expected Infrastructure, got {other:?}"),
        }
    }
}
