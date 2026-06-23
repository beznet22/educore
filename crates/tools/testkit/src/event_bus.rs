//! # In-memory event bus
//!
//! Thin re-export of
//! [`educore_event_bus::InProcessEventBus`](educore_event_bus::InProcessEventBus)
//! under the testkit-local name `InMemoryEventBus`.
//!
//! The bus is the engine's universal seam for in-process
//! pub/sub. The in-process adapter
//! ([`InProcessEventBus`](educore_event_bus::InProcessEventBus))
//! is MPMC, broadcast-backed, has a bounded replay log, and
//! supports `StartPosition::Latest`, `Earliest`, `FromEventId`,
//! and `FromTimestamp` cursor modes.
//!
//! The testkit does not need to re-implement the bus — it simply
//! re-exports the in-process adapter so consumers can write
//! `educore_testkit::event_bus::InMemoryEventBus` without taking
//! a direct dependency on `educore-event-bus`. The
//! [`TestkitWorld`](crate::TestkitWorld) constructor wires the
//! default instance (1024-channel-capacity, 4096-replay-log) and
//! hands an `Arc<dyn EventBus>` to consumers.
//!
//! See `docs/ports/event-bus.md` for the bus port contract.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub use educore_event_bus::InProcessEventBus;

/// Testkit-local alias for the in-process event bus.
///
/// The alias exists so consumers can write
/// `use educore_testkit::event_bus::InMemoryEventBus;` without
/// taking a direct dep on `educore-event-bus`. The underlying
/// type is `educore_event_bus::InProcessEventBus` (re-exported
/// above) — see that type's rustdoc for the full MPMC /
/// replay-log contract.
pub type InMemoryEventBus = InProcessEventBus;

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
    use educore_core::value_objects::Timestamp;
    use educore_events::envelope::EventEnvelope;
    use educore_events::event_bus::{EventBus, SubscribeOptions, Topic};

    use std::time::Duration;

    fn sample_envelope(event_type: &'static str, aggregate_type: &'static str) -> EventEnvelope {
        let g = SystemIdGen;
        let school = g.next_school_id();
        EventEnvelope {
            event_id: g.next_event_id(),
            event_type: event_type.to_owned(),
            schema_version: 1,
            school_id: school,
            aggregate_id: g.next_uuid(),
            aggregate_type: aggregate_type.to_owned(),
            actor_id: g.next_user_id(),
            correlation_id: g.next_correlation_id(),
            causation_id: None,
            occurred_at: Timestamp::now(),
            published_at: None,
            payload: serde_json::json!({}),
        }
    }

    #[test]
    fn alias_compiles_and_resolves_to_in_process_bus() {
        // The alias must resolve to the in-process bus type.
        let _: InMemoryEventBus = InMemoryEventBus::new();
    }

    #[tokio::test]
    async fn publish_and_subscribe_round_trip_through_alias() {
        // The point of the alias is that a consumer wiring
        // `Arc<dyn EventBus>` from the testkit works without
        // naming `educore_event_bus` directly.
        let bus: std::sync::Arc<dyn EventBus> = std::sync::Arc::new(InMemoryEventBus::new());

        let opts = SubscribeOptions::for_consumer(
            educore_events::event_bus::ConsumerId::new("testkit.alias"),
            Topic::All,
        );
        let mut sub = bus.subscribe(opts).await.expect("subscribe must succeed");

        let env = sample_envelope("platform.school.created", "school");
        let _receipt = bus
            .publish(env.clone())
            .await
            .expect("publish must succeed");

        // The in-process bus delivers asynchronously. We use a
        // short timeout to avoid hanging the test if the
        // implementation ever drifts.
        let next = tokio::time::timeout(Duration::from_secs(1), sub.next())
            .await
            .expect("no timeout")
            .expect("subscription still open")
            .expect("no error");

        assert_eq!(next.event_id, env.event_id);
        assert_eq!(next.event_type, "platform.school.created");
    }
}
