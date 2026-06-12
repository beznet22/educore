//! # Redis event bus stub
//!
//! Phase 2 scaffold for a Redis Streams-backed event bus. The
//! type and trait surface are wired so consumers can build a
//! `RedisEventBus` today; the wire-protocol work lands in a
//! future phase. All trait methods currently return
//! [`EventError::NotSupported`].
//!
//! # Future work (not in Phase 2)
//!
//! - Build a `redis::aio::ConnectionManager` from a Redis URL
//!   with `rustls` for TLS.
//! - Map `Topic::Aggregate` / `Domain` / `EventType` / `Tenant` /
//!   `All` to Redis Stream key conventions:
//!   - `Aggregate(d, a)` → `stream:events:<d>:<a>`
//!   - `Domain(d)` → consumer-group over `stream:events:<d>:*`
//!   - `EventType(t)` → `stream:events:type:<dotted t>`
//!   - `Tenant(s)` → `stream:tenant:<s>`
//!   - `All` → wildcard scan over `stream:events:*`
//! - Use consumer groups with `XREADGROUP` for offset tracking
//!   and `XACK` for ack semantics. `nack` requeue = true maps to
//!   `XCLAIM` / pending-entry re-delivery.

use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex as TokioMutex;
use tracing::debug;

use educore_events::envelope::EventEnvelope;
use educore_events::errors::EventError;
use educore_events::event_bus::{
    BatchReceipt, EventBus, EventSubscription, PublishReceipt, SubscribeOptions,
};

/// Redis Streams-backed event bus. Phase 2 stub: constructed
/// but not yet wired to a live connection.
///
/// The struct is not `Clone`; share it across the engine via
/// `Arc<dyn EventBus>` (the trait object hides the inner
/// mutex). The internal `TokioMutex` is wrapped in `Arc` so
/// adapters that need to share the connection manager across
/// tasks can clone the `Arc` cheaply.
pub struct RedisEventBus {
    /// The Redis client config, once `connect` is called. `None`
    /// means the bus is in the disconnected scaffold state and
    /// every operation returns `NotSupported`. The full client
    /// is captured for future use; Phase 2 only stores the URL.
    config: Arc<TokioMutex<Option<RedisBusConfig>>>,
}

/// Configuration snapshot for a connected Redis bus. Captured at
/// `connect` time so the bus can reconnect after a network
/// blip (future work).
#[derive(Clone)]
pub struct RedisBusConfig {
    /// The Redis connection URL the bus was last connected with.
    pub url: String,
    /// The connection manager, kept so the bus can re-establish
    /// the stream consumer after a reconnect. Phase 2 stores it
    /// but does not consume it. `ConnectionManager` does not
    /// implement `Debug`; the manual `Debug` impl below redacts
    /// the manager and only surfaces the URL.
    pub manager: redis::aio::ConnectionManager,
}

impl fmt::Debug for RedisBusConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RedisBusConfig")
            .field("url", &self.url)
            .field("manager", &"<redis::aio::ConnectionManager>")
            .finish()
    }
}

impl fmt::Debug for RedisEventBus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RedisEventBus")
            .field("connected", &self.config.blocking_lock().is_some())
            .finish()
    }
}

impl Default for RedisEventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl RedisEventBus {
    /// Constructs a disconnected Redis bus. All trait methods
    /// return `NotSupported` until [`connect`](Self::connect) is
    /// called. Phase 2 only.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: Arc::new(TokioMutex::new(None)),
        }
    }

    /// Connects to Redis at the given URL. The connection is
    /// stored on the bus; future phases will use it to publish
    /// / subscribe via Streams + consumer groups.
    ///
    /// # Errors
    ///
    /// Returns the underlying `redis::RedisError` if the
    /// connection cannot be established.
    pub async fn connect(&self, url: impl Into<String>) -> Result<(), redis::RedisError> {
        let url = url.into();
        let client = redis::Client::open(url.clone())?;
        let manager = redis::aio::ConnectionManager::new(client).await?;
        *self.config.lock().await = Some(RedisBusConfig { url, manager });
        Ok(())
    }

    /// Returns `true` if a live client is wired to this bus.
    #[must_use]
    pub async fn is_connected(&self) -> bool {
        self.config.lock().await.is_some()
    }
}

#[async_trait]
impl EventBus for RedisEventBus {
    async fn publish(
        &self,
        _envelope: EventEnvelope,
    ) -> educore_core::error::Result<PublishReceipt> {
        debug!("RedisEventBus::publish (Phase 2 stub, returning NotSupported)");
        Err(EventError::not_supported("RedisEventBus::publish").into())
    }

    async fn publish_batch(
        &self,
        _envelopes: Vec<EventEnvelope>,
    ) -> educore_core::error::Result<BatchReceipt> {
        debug!("RedisEventBus::publish_batch (Phase 2 stub, returning NotSupported)");
        Err(EventError::not_supported("RedisEventBus::publish_batch").into())
    }

    async fn subscribe(
        &self,
        _options: SubscribeOptions,
    ) -> educore_core::error::Result<Box<dyn EventSubscription>> {
        debug!("RedisEventBus::subscribe (Phase 2 stub, returning NotSupported)");
        Err(EventError::not_supported("RedisEventBus::subscribe").into())
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
    use educore_core::clock::{IdGenerator, SystemIdGen};
    use educore_core::tenant::TenantContext;
    use educore_events::domain_event::DomainEvent;
    use educore_events::event_bus::Topic;
    use educore_events::sync::SyncStarted;

    fn sample_envelope() -> EventEnvelope {
        let g = SystemIdGen;
        let school = g.next_school_id();
        SyncStarted::now(school)
            .into_envelope(&TenantContext::system(school, g.next_correlation_id()))
    }

    #[tokio::test]
    async fn redis_bus_is_constructable_and_disconnected_by_default() {
        let bus = RedisEventBus::new();
        assert!(!bus.is_connected().await, "new bus is disconnected");
    }

    #[tokio::test]
    async fn redis_bus_publish_returns_not_supported() {
        let bus = RedisEventBus::new();
        let env = sample_envelope();
        let err = match bus.publish(env).await {
            Ok(_) => panic!("must fail in Phase 2"),
            Err(e) => e,
        };
        match err.kind() {
            educore_core::error::ErrorKind::Validation => { /* matches NotSupported */ }
            other => panic!("expected NotSupported, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn redis_bus_subscribe_returns_not_supported() {
        let bus = RedisEventBus::new();
        let opts = SubscribeOptions::for_consumer(
            educore_events::event_bus::ConsumerId::new("test"),
            Topic::All,
        );
        let err = match bus.subscribe(opts).await {
            Ok(_) => panic!("must fail in Phase 2"),
            Err(e) => e,
        };
        match err.kind() {
            educore_core::error::ErrorKind::Validation => { /* matches NotSupported */ }
            other => panic!("expected NotSupported, got {other:?}"),
        }
    }
}
