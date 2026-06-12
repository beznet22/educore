//! # NATS event bus stub
//!
//! Phase 2 scaffold for a NATS JetStream-backed event bus. The
//! type and trait surface are wired so consumers can build a
//! `NatsEventBus` today; the wire-protocol work lands in a
//! future phase. All trait methods currently return
//! [`EventError::NotSupported`].
//!
//! # Future work (not in Phase 2)
//!
//! - Construct a JetStream `async_nats::Client` from a connection
//!   URL with TLS via `rustls`.
//! - Map `Topic::Aggregate` / `Domain` / `EventType` / `Tenant` /
//!   `All` to NATS subject conventions:
//!   - `Aggregate(d, a)` → `events.<d>.<a>`
//!   - `Domain(d)` → `events.<d>.>`
//!   - `EventType(t)` → `events.<dotted t>`
//!   - `Tenant(s)` → `tenant.<s>.>`
//!   - `All` → `events.>`
//! - Use a durable consumer per `ConsumerId` so redelivery and
//!   offset tracking work across process restarts.
//! - Implement ack / nack semantics against JetStream
//!   `consumer.ack` / `consumer.nack`.

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

/// NATS JetStream-backed event bus. Phase 2 stub: constructed
/// but not yet wired to a live connection.
///
/// The struct is not `Clone`; share it across the engine via
/// `Arc<dyn EventBus>` (the trait object hides the inner
/// mutex). The internal `TokioMutex` is wrapped in `Arc` so
/// adapters that need to share the client handle across tasks
/// can clone the `Arc` cheaply.
pub struct NatsEventBus {
    /// The live NATS client, once `connect` is called. `None`
    /// means the bus is in the disconnected scaffold state and
    /// every operation returns `NotSupported`.
    client: Arc<TokioMutex<Option<async_nats::Client>>>,
}

impl fmt::Debug for NatsEventBus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NatsEventBus")
            .field("connected", &self.client.blocking_lock().is_some())
            .finish()
    }
}

impl Default for NatsEventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl NatsEventBus {
    /// Constructs a disconnected NATS bus. All trait methods
    /// return `NotSupported` until [`connect`](Self::connect) is
    /// called. Phase 2 only.
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: Arc::new(TokioMutex::new(None)),
        }
    }

    /// Constructs a NATS bus pre-wired to the given client. The
    /// wire-protocol work that consumes this client is not
    /// implemented in Phase 2; the bus still returns
    /// `NotSupported` for every operation.
    #[must_use]
    pub fn with_client(client: async_nats::Client) -> Self {
        Self {
            client: Arc::new(TokioMutex::new(Some(client))),
        }
    }

    /// Connects to a NATS server at the given URL. The
    /// connection is stored on the bus; future phases will use
    /// it to publish / subscribe via JetStream.
    ///
    /// # Errors
    ///
    /// Returns the underlying `async_nats::ConnectError` if the
    /// connection cannot be established.
    pub async fn connect(&self, url: impl AsRef<str>) -> Result<(), async_nats::ConnectError> {
        let client = async_nats::connect(url.as_ref()).await?;
        *self.client.lock().await = Some(client);
        Ok(())
    }

    /// Returns `true` if a live client is wired to this bus.
    #[must_use]
    pub async fn is_connected(&self) -> bool {
        self.client.lock().await.is_some()
    }
}

#[async_trait]
impl EventBus for NatsEventBus {
    async fn publish(
        &self,
        _envelope: EventEnvelope,
    ) -> educore_core::error::Result<PublishReceipt> {
        debug!("NatsEventBus::publish (Phase 2 stub, returning NotSupported)");
        Err(EventError::not_supported("NatsEventBus::publish").into())
    }

    async fn publish_batch(
        &self,
        _envelopes: Vec<EventEnvelope>,
    ) -> educore_core::error::Result<BatchReceipt> {
        debug!("NatsEventBus::publish_batch (Phase 2 stub, returning NotSupported)");
        Err(EventError::not_supported("NatsEventBus::publish_batch").into())
    }

    async fn subscribe(
        &self,
        _options: SubscribeOptions,
    ) -> educore_core::error::Result<Box<dyn EventSubscription>> {
        debug!("NatsEventBus::subscribe (Phase 2 stub, returning NotSupported)");
        Err(EventError::not_supported("NatsEventBus::subscribe").into())
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
    async fn nats_bus_is_constructable_and_disconnected_by_default() {
        let bus = NatsEventBus::new();
        assert!(!bus.is_connected().await, "new bus is disconnected");
    }

    #[tokio::test]
    async fn nats_bus_publish_returns_not_supported() {
        let bus = NatsEventBus::new();
        let env = sample_envelope();
        let err = match bus.publish(env).await {
            Ok(_) => panic!("must fail in Phase 2"),
            Err(e) => e,
        };
        // The bus port returns `DomainError`; the inner
        // `EventError::NotSupported` is mapped to
        // `DomainError::NotSupported` by the `From` impl.
        match err.kind() {
            educore_core::error::ErrorKind::Validation => { /* matches NotSupported */ }
            other => panic!("expected NotSupported, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn nats_bus_subscribe_returns_not_supported() {
        let bus = NatsEventBus::new();
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
