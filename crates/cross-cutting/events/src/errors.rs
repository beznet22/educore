//! # Event error type
//!
//! The error type returned by [`EventBus`](crate::event_bus::EventBus)
//! and [`EventSubscription`](crate::event_bus::EventSubscription)
//! implementations. Per the bus port, the variants are stable;
//! adding a new variant is a non-breaking change for trait
//! implementors, but a breaking change for callers that match
//! exhaustively.

use thiserror::Error;

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
    fn display_messages_are_stable() {
        let e = EventError::SubscriptionClosed;
        assert_eq!(e.to_string(), "subscription closed");
        let e = EventError::PublishFailed("bus down".to_owned());
        assert_eq!(e.to_string(), "publish failed: bus down");
        let e = EventError::DeserializeFailed("bad json".to_owned());
        assert_eq!(e.to_string(), "deserialize failed: bad json");
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
}
