//! # educore-events
//!
//! The engine's domain-event crate. Owns:
//!
//! - The [`EventEnvelope`] wire format (locked to
//!   [`docs/ports/event-bus.md`](../docs/ports/event-bus.md)).
//! - The [`DomainEvent`] trait every typed event implements.
//! - The [`EventBus`] port and the [`EventSubscription`], [`Topic`],
//!   [`SubscribeOptions`], [`EventFilter`], [`StartPosition`],
//!   [`ConsumerId`], [`PublishReceipt`], [`BatchReceipt`]
//!   supporting types.
//! - The four typed sync events ([`sync::SyncStarted`],
//!   [`sync::SyncPaused`], [`sync::SyncResumed`],
//!   [`sync::SyncStopped`]) that replaced the ad-hoc `SyncEvent`
//!   enum in `educore-sync` (Phase 0 open question #2).
//! - The outbox bridge ([`outbox::to_serialized`]) that converts
//!   a typed `EventEnvelope` into the storage-port
//!   `SerializedEnvelope` for the transactional outbox pattern.
//!
//! Concrete bus adapters live in the `educore-event-bus` crate
//! (in-process, NATS, Redis). This crate is the port only.
//!
//! See `docs/build-plan.md` § "Phase 2" and `docs/ports/event-bus.md`.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// Package name constant. Re-exported so consumers can assert they
/// are using the right crate version at compile time.
pub const PACKAGE_NAME: &str = "educore-events";

/// Package version at compile time.
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The [`DomainEvent`] trait and the typed event envelope.
pub mod domain_event;

/// The bus-port wire envelope ([`EventEnvelope`]).
pub mod envelope;

/// The error type returned by the bus port.
pub mod errors;

/// The [`EventBus`] port and the [`EventSubscription`] /
/// [`Topic`] / [`SubscribeOptions`] / [`EventFilter`] /
/// [`StartPosition`] / [`ConsumerId`] / [`PublishReceipt`] /
/// [`BatchReceipt`] supporting types.
pub mod event_bus;

/// The outbox bridge: typed [`EventEnvelope`] → storage-port
/// `SerializedEnvelope`.
pub mod outbox;

/// The relay envelope — the concrete, serialization-ready
/// envelope stored in the outbox and consumed by
/// [`OutboxRelay`](relay::OutboxRelay).
pub mod relay_envelope;

/// The four typed sync events emitted by the sync engine.
pub mod sync;

/// The three typed bulk-command lifecycle events emitted by
/// the engine's bulk dispatcher (see
/// `docs/schemas/command-schema.md` § 12).
pub mod bulk_events;

/// The [`SchemaRegistry`](schema_registry::SchemaRegistry) port:
/// per-school registry of event / command / aggregate-state
/// schemas. Storage adapters implement the trait; consumers
/// consult it before publishing or consuming envelopes.
pub mod schema_registry;

/// The [`ConflictResolver`](conflict::ConflictResolver) trait
/// and supporting types (`Conflict<T>`, `ConflictKind`,
/// `ConflictResolution<T>`) per ADR-018 § 6 + ADR-008.
pub mod conflict;

/// The typed event the engine publishes when an event-log
/// retention sweep is due (see `docs/schemas/event-schema.md`
/// § 9). Parallel to the audit crate's `RetentionSweepDue`.
pub mod event_retention;

/// The typed event the engine publishes when an idempotency
/// TTL sweep is due (see `ADR-014-Idempotency.md`). Parallel
/// to the audit crate's `RetentionSweepDue` and the
/// event_log's `EventLogRetentionSweepDue`.
pub mod idempotency_retention;

/// Cross-cutting finance domain events (e.g.
/// [`PaymentSettled`](finance_events::PaymentSettled)). Lives in
/// the `educore-events` crate so consumers can subscribe
/// without depending on `educore-finance` directly.
pub mod finance_events;

// Cross-cutting finance domain events re-exported at the
// crate root for convenience.
pub use crate::finance_events::PaymentSettled;

/// The in-process [`SubscriberRegistry`](subscribe::SubscriberRegistry)
/// and the [`Subscriber`](subscribe::Subscriber) trait. Used to
/// wire spec-mandated cross-domain handlers to the bus.
pub mod subscribe;

/// The [`OutboxRelay`](relay::OutboxRelay) that drains the
/// storage-port `Outbox` and forwards envelopes to the
/// bus-port `EventBus`.
pub mod relay;

/// Convenience re-exports for the most-used types.
pub mod prelude {
    pub use crate::bulk_events::{
        BulkCommandCompleted, BulkCommandItemProcessed, BulkCommandStarted, BulkFailurePolicy,
    };
    pub use crate::conflict::{
        Conflict, ConflictKind, ConflictResolution, ConflictResolver, FirstWriteWinsResolver,
        KindPolicyResolver, LastWriteWinsResolver, RejectAllResolver,
    };
    pub use crate::domain_event::{DomainEvent, EmittedEvent, EventFactory};
    pub use crate::envelope::EventEnvelope;
    pub use crate::errors::EventError;
    pub use crate::event_bus::{
        AckOutcome, BatchReceipt, ConsumerId, DeadLetterEntry, DeadLetterQueue, DeadLetterReason,
        EventBus, EventFilter, EventFilterExpr, EventSubscription, NoopDeadLetterQueue,
        PublishReceipt, StartPosition, SubscribeOptions, Topic,
    };
    pub use crate::event_retention::{EventLogRetentionPolicy, EventLogRetentionSweepDue};
    pub use crate::finance_events::PaymentSettled;
    pub use crate::idempotency_retention::{IdempotencyRetentionPolicy, IdempotencyTtlSweepDue};
    pub use crate::relay::{OutboxRelay, RelayStats, DEFAULT_BATCH_SIZE, DEFAULT_IDLE_DELAY};
    pub use crate::schema_registry::{
        fresh_schema_cid, RegisteredSchema, SchemaKind, SchemaRegistry, SchemaVersion,
    };
    pub use crate::subscribe::{
        DispatchStats, Subscriber, SubscriberFailure, SubscriberRegistry, SubscriptionFilter,
    };
    pub use crate::sync::{SyncPaused, SyncResumed, SyncStarted, SyncStopped};
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::prelude::*;
    use super::*;

    #[test]
    fn package_metadata_is_set() {
        assert_eq!(PACKAGE_NAME, "educore-events");
        assert!(!PACKAGE_VERSION.is_empty());
    }

    #[test]
    fn sync_events_round_trip_through_envelope() {
        use educore_core::clock::{IdGenerator, SystemIdGen};
        use educore_core::tenant::{TenantContext, UserType};

        let g = SystemIdGen;
        let school = g.next_school_id();
        let ctx = TenantContext::for_user(
            school,
            g.next_user_id(),
            g.next_correlation_id(),
            UserType::Teacher,
        );
        let env = SyncStarted::now(school).into_envelope(&ctx);
        assert_eq!(env.event_type, "sync.session.started");
        assert_eq!(env.school_id, school);
        assert_eq!(env.actor_id, ctx.actor_id);
    }
}
