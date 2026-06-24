//! # In-process bus end-to-end tests
//!
//! Integration tests for [`InProcessEventBus`]. Each test runs on
//! the in-memory bus; the bus is per-test (no shared state).
//!
//! # Test cases
//!
//! 1. `publish_then_subscribe_receives_envelope` — basic
//!    round-trip.
//! 2. `multi_subscriber_fan_out` — publish once, N subscribers
//!    all receive.
//! 3. `topic_filter_matches_aggregate_type` — `EventFilter::AggregateType`
//!    filter is applied.
//! 4. `topic_filter_school_id_blocks_cross_tenant` — `EventFilter::SchoolId`
//!    filter blocks cross-tenant envelopes.
//! 5. `subscription_close_releases_resources` — close, then
//!    `next()` returns `None` and the broadcast channel
//!    releases its slot.
//! 6. `start_position_latest_skips_historical` — `StartPosition::Latest`
//!    skips envelopes published before `subscribe`.
//! 7. `start_position_earliest_replays` — `StartPosition::Earliest`
//!    replays envelopes published before `subscribe`.
//! 8. `nats_bus_returns_not_supported_without_connection` — feature-gated.
//! 9. `redis_bus_returns_not_supported_without_connection` — feature-gated.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]

use std::sync::Arc;
use std::time::Duration;

use educore_core::clock::{IdGenerator, SystemIdGen};
use educore_core::ids::Identifier;
use educore_core::tenant::{TenantContext, UserType};
use educore_event_bus::InProcessEventBus;
use educore_events::domain_event::DomainEvent;
use educore_events::envelope::EventEnvelope;
use educore_events::event_bus::{
    ConsumerId, EventBus, EventFilter, StartPosition, SubscribeOptions, Topic,
};
use educore_events::sync::SyncStarted;
use uuid::Uuid;

fn sample_envelope(aggregate_topic_domain: &str, aggregate_type: &str) -> EventEnvelope {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let mut env = SyncStarted::now(school).into_envelope(&TenantContext::for_user(
        school,
        g.next_user_id(),
        g.next_correlation_id(),
        UserType::System,
    ));
    // Override the aggregate_type so we can drive Topic::Aggregate
    // filters in a way that's distinguishable from the
    // SyncStarted default ("sync_session").
    env.aggregate_type = match aggregate_type {
        "sync_session" => "sync_session".to_string(),
        "student" => "student".to_string(),
        other => panic!("unknown aggregate_type in test fixture: {other}"),
    };
    // Override event_type to drive the `aggregate_topic()` routing
    // key into the requested domain.
    env.event_type = match aggregate_topic_domain {
        "sync" => "sync.session.started".to_string(),
        "academic" => "academic.student.admitted".to_string(),
        other => panic!("unknown domain in test fixture: {other}"),
    };
    env
}

fn make_opts(consumer: &str, topic: Topic, start: StartPosition) -> SubscribeOptions {
    SubscribeOptions {
        consumer: ConsumerId::new(consumer),
        topic,
        filter: None,
        start,
        batch_size: 32,
        visibility_timeout: Duration::from_secs(300),
    }
}

#[tokio::test]
async fn publish_then_subscribe_receives_envelope() {
    let bus = InProcessEventBus::new();

    // Subscribe FIRST, then publish. The subscription uses
    // `StartPosition::Latest` (the default), so it sees only
    // envelopes published after `subscribe` returns.
    let mut sub = bus
        .subscribe(make_opts(
            "test-consumer",
            Topic::All,
            StartPosition::Latest,
        ))
        .await
        .expect("subscribe");

    let env = sample_envelope("sync", "sync_session");
    let receipt = bus.publish(env.clone()).await.expect("publish");
    assert_eq!(receipt.event_id, env.event_id);

    let got = sub.next().await.expect("Some").expect("Ok");
    assert_eq!(got.event_id, env.event_id);
    assert_eq!(got.event_type, "sync.session.started");
}

#[tokio::test]
async fn multi_subscriber_fan_out() {
    let bus = InProcessEventBus::new();
    let n: usize = 5;
    let mut subs = Vec::with_capacity(n);
    for i in 0..n {
        let s = bus
            .subscribe(make_opts(
                &format!("fan-{i}"),
                Topic::All,
                StartPosition::Latest,
            ))
            .await
            .expect("subscribe");
        subs.push(s);
    }

    // Bus should report one receiver per subscription.
    assert_eq!(bus.receiver_count(), n);

    let env = sample_envelope("sync", "sync_session");
    bus.publish(env.clone()).await.expect("publish");

    for (i, sub) in subs.iter_mut().enumerate() {
        let got = sub
            .next()
            .await
            .unwrap_or_else(|| panic!("sub {i} got None"))
            .expect("Ok");
        assert_eq!(got.event_id, env.event_id, "sub {i} saw wrong envelope");
    }
}

#[tokio::test]
async fn topic_filter_matches_aggregate_type() {
    let bus = InProcessEventBus::new();

    let mut sub = bus
        .subscribe(SubscribeOptions {
            consumer: ConsumerId::new("school-listener"),
            topic: Topic::All,
            filter: Some(EventFilter::AggregateType("school")),
            start: StartPosition::Latest,
            batch_size: 32,
            visibility_timeout: Duration::from_secs(300),
        })
        .await
        .expect("subscribe");

    // Publish a "sync_session" envelope — should be filtered out.
    let sync_env = sample_envelope("sync", "sync_session");
    bus.publish(sync_env).await.expect("publish sync");

    // Publish a "student" envelope (using the academic event_type
    // override) — should be delivered.
    let student_env = sample_envelope("academic", "student");
    bus.publish(student_env.clone())
        .await
        .expect("publish student");

    // The "school" filter still doesn't match "student"; let's
    // actually send an envelope that matches the "school"
    // aggregate_type so the test is self-consistent. The
    // fixture helper only emits "sync_session" or "student";
    // we patch one envelope's aggregate_type by hand.
    let mut school_env = sample_envelope("sync", "sync_session");
    school_env.aggregate_type = "school".to_string();
    bus.publish(school_env.clone())
        .await
        .expect("publish school");

    // Skip the first two (filter blocks "sync_session" and
    // "student"); the third matches.
    let got = sub.next().await.expect("Some").expect("Ok");
    assert_eq!(got.aggregate_type, "school");
    assert_eq!(got.event_id, school_env.event_id);
}

#[tokio::test]
async fn topic_filter_school_id_blocks_cross_tenant() {
    let bus = InProcessEventBus::new();
    let g = SystemIdGen;
    let my_school = g.next_school_id();
    let other_school = g.next_school_id();

    let mut sub = bus
        .subscribe(SubscribeOptions {
            consumer: ConsumerId::new("tenant-a-listener"),
            topic: Topic::All,
            filter: Some(EventFilter::SchoolId(my_school)),
            start: StartPosition::Latest,
            batch_size: 32,
            visibility_timeout: Duration::from_secs(300),
        })
        .await
        .expect("subscribe");

    // Publish a cross-tenant envelope first — should be filtered out.
    let mut other_env = sample_envelope("sync", "sync_session");
    other_env.school_id = other_school;
    bus.publish(other_env).await.expect("publish other");

    // Publish a same-tenant envelope — should be delivered.
    let mut my_env = sample_envelope("sync", "sync_session");
    my_env.school_id = my_school;
    bus.publish(my_env.clone()).await.expect("publish mine");

    let got = sub.next().await.expect("Some").expect("Ok");
    assert_eq!(got.school_id, my_school);
    assert_eq!(got.event_id, my_env.event_id);
}

#[tokio::test]
async fn subscription_close_releases_resources() {
    let bus = InProcessEventBus::new();
    let sub = bus
        .subscribe(make_opts("closer", Topic::All, StartPosition::Latest))
        .await
        .expect("subscribe");
    assert_eq!(bus.receiver_count(), 1);

    sub.close().await.expect("close");
    assert_eq!(
        bus.receiver_count(),
        0,
        "broadcast slot should be released after close"
    );

    // After close, a new subscription for the same consumer
    // should still work.
    let mut sub2 = bus
        .subscribe(make_opts("closer", Topic::All, StartPosition::Latest))
        .await
        .expect("subscribe again");
    let env = sample_envelope("sync", "sync_session");
    bus.publish(env.clone()).await.expect("publish");
    let got = sub2.next().await.expect("Some").expect("Ok");
    assert_eq!(got.event_id, env.event_id);
}

#[tokio::test]
async fn start_position_latest_skips_historical() {
    let bus = InProcessEventBus::new();
    let env = sample_envelope("sync", "sync_session");

    // Publish BEFORE subscribe.
    bus.publish(env.clone()).await.expect("publish");

    // Subscribe with `Latest` (the default).
    let mut sub = bus
        .subscribe(make_opts(
            "latest-listener",
            Topic::All,
            StartPosition::Latest,
        ))
        .await
        .expect("subscribe");

    // Publish AFTER subscribe; the subscription sees this.
    let post = sample_envelope("sync", "sync_session");
    bus.publish(post.clone()).await.expect("publish post");

    // Drain the next event; it MUST be `post`, not `env`.
    let got = sub.next().await.expect("Some").expect("Ok");
    assert_eq!(got.event_id, post.event_id);
    assert_ne!(got.event_id, env.event_id, "historical envelope leaked");
}

#[tokio::test]
async fn start_position_earliest_replays() {
    let bus = InProcessEventBus::new();

    // Publish a handful of envelopes BEFORE subscribe.
    let mut published: Vec<EventEnvelope> = Vec::new();
    for _ in 0..3 {
        let env = sample_envelope("sync", "sync_session");
        bus.publish(env.clone()).await.expect("publish");
        published.push(env);
    }

    // Subscribe with `Earliest`.
    let mut sub = bus
        .subscribe(make_opts(
            "earliest-listener",
            Topic::All,
            StartPosition::Earliest,
        ))
        .await
        .expect("subscribe");

    // Each replayed envelope should arrive in order.
    for (i, expected) in published.iter().enumerate() {
        let got = sub
            .next()
            .await
            .unwrap_or_else(|| panic!("replay {i} got None"))
            .expect("Ok");
        assert_eq!(got.event_id, expected.event_id, "replay {i} mismatch");
    }
}

#[cfg(feature = "nats")]
#[tokio::test]
async fn nats_bus_returns_not_supported_without_connection() {
    use educore_event_bus::NatsEventBus;
    use educore_events::event_bus::EventBus as _;

    let bus = NatsEventBus::new();
    assert!(
        !bus.is_connected().await,
        "default NATS bus is disconnected"
    );

    let env = sample_envelope("sync", "sync_session");
    let err = match bus.publish(env).await {
        Ok(_) => panic!("publish on disconnected NATS must fail"),
        Err(e) => e,
    };
    let msg = format!("{err}");
    assert!(msg.contains("not supported"), "unexpected error: {msg}");

    let opts = SubscribeOptions::for_consumer(ConsumerId::new("stub"), Topic::All);
    let err = match bus.subscribe(opts).await {
        Ok(_) => panic!("subscribe on disconnected NATS must fail"),
        Err(e) => e,
    };
    let msg = format!("{err}");
    assert!(msg.contains("not supported"), "unexpected error: {msg}");
}

#[cfg(feature = "redis")]
#[tokio::test]
async fn redis_bus_returns_not_supported_without_connection() {
    use educore_event_bus::RedisEventBus;
    use educore_events::event_bus::EventBus as _;

    let bus = RedisEventBus::new();
    assert!(
        !bus.is_connected().await,
        "default Redis bus is disconnected"
    );

    let env = sample_envelope("sync", "sync_session");
    let err = match bus.publish(env).await {
        Ok(_) => panic!("publish on disconnected Redis must fail"),
        Err(e) => e,
    };
    let msg = format!("{err}");
    assert!(msg.contains("not supported"), "unexpected error: {msg}");

    let opts = SubscribeOptions::for_consumer(ConsumerId::new("stub"), Topic::All);
    let err = match bus.subscribe(opts).await {
        Ok(_) => panic!("subscribe on disconnected Redis must fail"),
        Err(e) => e,
    };
    let msg = format!("{err}");
    assert!(msg.contains("not supported"), "unexpected error: {msg}");
}

// Use Arc<InProcessEventBus> to confirm the bus is `Send + Sync`
// and can be shared across an `Arc` like a real adapter would.
#[tokio::test]
async fn bus_is_send_sync_via_arc() {
    let bus: Arc<dyn EventBus> = Arc::new(InProcessEventBus::new());
    let mut sub = bus
        .subscribe(make_opts("arc-test", Topic::All, StartPosition::Latest))
        .await
        .expect("subscribe");

    let env = sample_envelope("sync", "sync_session");
    bus.publish(env.clone()).await.expect("publish");

    let got = sub.next().await.expect("Some").expect("Ok");
    assert_eq!(got.event_id, env.event_id);
    // Make sure `Uuid` is in scope; this is a no-op assertion
    // but it keeps the unused-import lint from firing if the
    // test list shrinks.
    let _: Uuid = env.event_id.as_uuid();
}
