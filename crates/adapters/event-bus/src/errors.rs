//! # Adapter error mapping
//!
//! Bridges the bus-port's [`EventError`] and the engine-wide
//! [`DomainError`](educore_core::error::DomainError) for adapter
//! code that surfaces richer internal errors. The in-process
//! bus maps most error sources to the bus-port enum directly;
//! the NATS and Redis stubs return
//! [`EventError::NotSupported`] for methods that require a live
//! connection (Phase 2).
//!
//! # When to use
//!
//! - Adapter code that needs to surface a structured
//!   `EventError` should construct one of the variants
//!   directly. The `?` operator works because of the
//!   `From<EventError> for DomainError` impl in `educore-events`.
//! - Adapter code that surfaces a free-form message (e.g., a
//!   connection error from `async_nats`) should use
//!   [`EventError::infrastructure`] and pass the source error
//!   through; the `From` impl preserves the source.
//!
//! # Why a separate module
//!
//! The adapter crate is in the `adapters` tier; the bus port's
//! `EventError` lives in the `cross-cutting` tier. Keeping the
//! mapping in the adapter (rather than the port) preserves the
//! port's neutrality: an adapter that wants to surface its own
//! richer error type can add a `From` impl on the bus-port
//! side without leaking the new variant to other consumers.

use educore_events::errors::EventError;

/// Convenience: build a [`EventError::PublishFailed`] with a
/// free-form message. Used by adapter code that does not have
/// a richer source error to attach.
#[inline]
#[must_use]
pub fn publish_failed(msg: impl Into<String>) -> EventError {
    EventError::PublishFailed(msg.into())
}

/// Convenience: build a [`EventError::DeserializeFailed`] with a
/// free-form message. Used by adapter code that fails to decode
/// a wire-format envelope.
#[inline]
#[must_use]
pub fn deserialize_failed(msg: impl Into<String>) -> EventError {
    EventError::DeserializeFailed(msg.into())
}

/// Convenience: build a [`EventError::NotSupported`] with a
/// free-form reason. Used by Phase 2 stub adapters (NATS,
/// Redis) for methods that require a live connection.
#[inline]
#[must_use]
pub fn not_supported(msg: impl Into<String>) -> EventError {
    EventError::not_supported(msg)
}

/// Build a subscribe-path failure. Reuses the bus-port's
/// `PublishFailed` variant because the bus port only
/// distinguishes failure modes at the semantic level (publish
/// failure vs. subscription closure), not the call-site level.
#[inline]
pub fn subscribe_failed(msg: impl Into<String>) -> EventError {
    EventError::PublishFailed(format!("subscribe failed: {}", msg.into()))
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
    fn convenience_constructors_build_expected_variants() {
        match publish_failed("boom") {
            EventError::PublishFailed(s) => assert_eq!(s, "boom"),
            other => panic!("expected PublishFailed, got {other:?}"),
        }
        match deserialize_failed("bad bytes") {
            EventError::DeserializeFailed(s) => assert_eq!(s, "bad bytes"),
            other => panic!("expected DeserializeFailed, got {other:?}"),
        }
        match not_supported("kafka") {
            EventError::NotSupported(s) => assert_eq!(s, "kafka"),
            other => panic!("expected NotSupported, got {other:?}"),
        }
    }
}
