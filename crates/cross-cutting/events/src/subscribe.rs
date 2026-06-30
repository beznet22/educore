//! # In-process subscriber registry
//!
//! Defines the [`Subscriber`] trait and the [`SubscriberRegistry`]
//! used to wire spec-mandated cross-domain subscribers to the
//! bus-port [`EventEnvelope`](crate::envelope::EventEnvelope)
//! stream.
//!
//! Per the audit (`docs/audit_reports/findings/wave7-workflows.md`
//! WF-002 / WF-030), the engine defines handler functions in each
//! domain crate but never registers them on the bus. This module
//! provides the registration pattern; concrete subscriber
//! implementations live in each domain's `subscribers.rs` (or in
//! `crates/educore/src/subscribers.rs` for the umbrella wiring).
//!
//! ## Idempotency contract
//!
//! Every subscriber MUST be idempotent. The bus-port contract is
//! at-least-once delivery; the `event_id` field of
//! [`EventEnvelope`](crate::envelope::EventEnvelope) is the
//! dedupe key. Subscriber implementations should consult the
//! idempotency port (or an internal `seen_events: HashSet<Uuid>`
//! for test cases) to discard re-deliveries.
//!
//! ## Async contract
//!
//! Subscribers are async because cross-domain handlers routinely
//! issue follow-up commands that round-trip through storage. The
//! trait's `handle` method uses `async_trait`'s boxed future so
//! `Box<dyn Subscriber>` stays object-safe.
//!
//! ## Filter model
//!
//! A [`SubscriptionFilter`] is a composable matcher over
//! `event_type`, `aggregate_type`, and `school_id`. All fields
//! are `Option`-typed; `None` means "wildcard on this dimension".
//! The AND of all set fields determines a match. This is the
//! minimum surface needed for the spec-mandated subscribers
//! (`"documents.form_download.uploaded"`, etc.). Consumers that
//! need richer expressions can layer
//! [`EventFilterExpr`](crate::event_bus::EventFilterExpr) on top
//! of this primitive.

use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;

use educore_core::error::Result;
use educore_core::ids::SchoolId;

use crate::envelope::EventEnvelope;

/// A subscriber that consumes bus-port envelopes and reacts by
/// issuing follow-up commands or projections.
///
/// # Object safety
///
/// The trait is object-safe: the async method uses
/// `async_trait`, which keeps the future boxed. Consumers hold
/// subscribers as `Arc<dyn Subscriber>` inside a
/// [`SubscriberRegistry`].
///
/// # Idempotency
///
/// Implementations MUST be idempotent. The bus provides
/// at-least-once delivery; the same `event_id` may arrive
/// multiple times. Discard re-deliveries via the idempotency
/// port, a local `seen_events` set, or by keying the downstream
/// command on `(event_id, ...)`.
#[async_trait]
pub trait Subscriber: Send + Sync + fmt::Debug {
    /// A stable, human-readable name for the subscriber. Used in
    /// logs, metrics, and the `DispatchStats::failures` map.
    fn name(&self) -> &'static str;

    /// Handle a single envelope. Returning `Ok(())` is an
    /// acknowledgement; the caller proceeds to the next matching
    /// subscriber. Returning `Err(_)` is recorded in
    /// `DispatchStats::failures` but does NOT halt the dispatch
    /// loop — other subscribers still see the envelope.
    async fn handle(&self, envelope: &EventEnvelope) -> Result<()>;
}

/// A composable subscription filter. `None` on any field means
/// "wildcard on this dimension"; all `Some`-fields must match.
///
/// This is intentionally simpler than
/// [`EventFilterExpr`](crate::event_bus::EventFilterExpr): the
/// spec-mandated subscribers all match on `event_type` alone,
/// with a few exceptions that also pin `aggregate_type` or
/// `school_id`. Callers that need boolean expressions can stack
/// a higher-level matcher on top of `SubscriptionFilter`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct SubscriptionFilter {
    /// Optional dotted event type, e.g.
    /// `"documents.form_download.uploaded"`. `None` matches any
    /// event type.
    pub event_type: Option<&'static str>,
    /// Optional aggregate type, e.g. `"form_download"`. `None`
    /// matches any aggregate.
    pub aggregate_type: Option<&'static str>,
    /// Optional tenant anchor. `None` matches any school.
    pub school_id: Option<SchoolId>,
}

impl SubscriptionFilter {
    /// Constructs a filter that matches a single event type
    /// (the most common case for spec-mandated subscribers).
    #[must_use]
    pub const fn on_event(event_type: &'static str) -> Self {
        Self {
            event_type: Some(event_type),
            aggregate_type: None,
            school_id: None,
        }
    }

    /// Constructs a filter that matches a single event type
    /// pinned to a specific school.
    #[must_use]
    pub const fn on_event_for_school(event_type: &'static str, school_id: SchoolId) -> Self {
        Self {
            event_type: Some(event_type),
            aggregate_type: None,
            school_id: Some(school_id),
        }
    }

    /// Constructs a filter that matches any event of a given
    /// aggregate type.
    #[must_use]
    pub const fn on_aggregate(aggregate_type: &'static str) -> Self {
        Self {
            event_type: None,
            aggregate_type: Some(aggregate_type),
            school_id: None,
        }
    }

    /// Returns `true` if the envelope matches every set field.
    /// Wildcards (`None`) match unconditionally.
    #[must_use]
    pub fn matches(&self, envelope: &EventEnvelope) -> bool {
        if let Some(t) = self.event_type {
            if envelope.event_type != t {
                return false;
            }
        }
        if let Some(a) = self.aggregate_type {
            if envelope.aggregate_type != a {
                return false;
            }
        }
        if let Some(s) = self.school_id {
            if envelope.school_id != s {
                return false;
            }
        }
        true
    }
}

/// Statistics from a single [`SubscriberRegistry::dispatch`]
/// call.
///
/// # Fields
///
/// - `delivered` — the number of `(filter, subscriber)` pairs
///   whose filter matched the envelope and whose `handle`
///   returned `Ok(())`.
/// - `skipped` — the number of pairs whose filter did NOT
///   match the envelope.
/// - `failed` — the number of pairs whose filter matched but
///   `handle` returned `Err(_)`. The map keys are the
///   subscriber's [`Subscriber::name`] for log correlation.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DispatchStats {
    /// Number of subscribers that handled the envelope
    /// successfully.
    pub delivered: usize,
    /// Number of registered subscribers whose filter did not
    /// match the envelope.
    pub skipped: usize,
    /// Per-subscriber failure map. Keyed by
    /// [`Subscriber::name`]. Only present when at least one
    /// handler returned an error.
    pub failures: Vec<SubscriberFailure>,
}

/// A single failure recorded during dispatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriberFailure {
    /// The subscriber that failed (stable name).
    pub subscriber: &'static str,
    /// The error's human-readable message. The full error is
    /// logged via `tracing`; this field exists so callers can
    /// surface a structured summary.
    pub error: String,
}

impl DispatchStats {
    /// Returns `true` if every matching subscriber handled the
    /// envelope without error.
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.failures.is_empty()
    }

    /// Returns the total number of subscribers considered
    /// (delivered + skipped + failed).
    #[must_use]
    pub fn total(&self) -> usize {
        self.delivered + self.skipped + self.failures.len()
    }
}

/// A registered `(filter, subscriber)` pair. Held by
/// [`SubscriberRegistry`].
#[derive(Debug)]
struct Registration {
    filter: SubscriptionFilter,
    subscriber: Arc<dyn Subscriber>,
}

/// A fan-out registry of in-process subscribers.
///
/// # Construction
///
/// Use [`SubscriberRegistry::new`] to construct an empty
/// registry, then call [`SubscriberRegistry::register`] for each
/// `(filter, subscriber)` pair. The umbrella crate's
/// `subscribers::register_all_subscribers` constructs the
/// spec-mandated set in one call.
///
/// # Dispatch
///
/// [`SubscriberRegistry::dispatch`] iterates the registry in
/// insertion order, evaluates the filter against the envelope,
/// and calls [`Subscriber::handle`] on every match. Failures are
/// recorded in [`DispatchStats::failures`] but do NOT halt the
/// loop; a failing subscriber cannot starve its peers.
///
/// # Thread safety
///
/// The registry is `Send + Sync` when the underlying
/// subscribers are. The internal `Vec<Registration>` is
/// append-only (no removal); readers (`dispatch`) never block
/// writers because writes happen at startup before the
/// registry is shared.
///
/// # Object safety
///
/// The registry itself is not object-safe (it has a `register`
/// method that consumes `Arc<dyn Subscriber>`). It is intended
/// to be constructed once at server startup and used by value
/// thereafter.
#[derive(Debug, Default)]
pub struct SubscriberRegistry {
    registrations: Vec<Registration>,
}

impl SubscriberRegistry {
    /// Constructs an empty registry.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            registrations: Vec::new(),
        }
    }

    /// Registers a `(filter, subscriber)` pair. The subscriber
    /// is invoked whenever an envelope matches `filter`. The
    /// same subscriber may be registered multiple times with
    /// different filters (e.g. one per event type).
    ///
    /// # Order
    ///
    /// Registrations are dispatched in the order they were
    /// added. Callers that need a deterministic fan-out
    /// (e.g. tests) should register in the desired order.
    pub fn register(&mut self, filter: SubscriptionFilter, subscriber: Arc<dyn Subscriber>) {
        self.registrations.push(Registration { filter, subscriber });
    }

    /// Returns the number of registered `(filter, subscriber)`
    /// pairs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.registrations.len()
    }

    /// Returns `true` if no subscribers are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.registrations.is_empty()
    }

    /// Fans an envelope out to every matching subscriber.
    ///
    /// # Semantics
    ///
    /// - The envelope is delivered to every registration whose
    ///   filter matches.
    /// - Subscribers run sequentially in registration order.
    ///   This is intentional: it preserves deterministic
    ///   ordering for test assertions and audit replay. Bus
    ///   adapters that need parallelism wrap the registry in
    ///   their own runtime.
    /// - A subscriber's `Err(_)` return is recorded in
    ///   [`DispatchStats::failures`] but does NOT halt the
    ///   fan-out. The dispatcher is "best-effort fan-out":
    ///   failing subscribers cannot starve their peers.
    ///
    /// # Returns
    ///
    /// A [`DispatchStats`] summarising delivered / skipped /
    /// failed counts. The caller can inspect
    /// [`DispatchStats::is_ok`] for a quick success check.
    pub async fn dispatch(&self, envelope: &EventEnvelope) -> Result<DispatchStats> {
        let mut stats = DispatchStats::default();
        for reg in &self.registrations {
            if !reg.filter.matches(envelope) {
                stats.skipped += 1;
                continue;
            }
            match reg.subscriber.handle(envelope).await {
                Ok(()) => stats.delivered += 1,
                Err(err) => {
                    tracing::warn!(
                        subscriber = reg.subscriber.name(),
                        event_id = %envelope.event_id,
                        event_type = envelope.event_type,
                        error = %err,
                        "subscriber dispatch failed"
                    );
                    stats.failures.push(SubscriberFailure {
                        subscriber: reg.subscriber.name(),
                        error: err.to_string(),
                    });
                }
            }
        }
        Ok(stats)
    }
}

/// Subscriber scaffolding for the event → audit mirror path.
///
/// Per `docs/schemas/event-schema.md` § 13 ("Every event must
/// be mirrored to the audit_log") the engine emits one audit
/// row per dispatched event so the audit trail captures the
/// full event lifecycle. The full subscriber implementation
/// lives in `educore-audit::subscriber::AuditMirrorSubscriber`
/// (which holds an `Arc<dyn AuditSink>` from
/// `educore_audit::sink::AuditSink`); this module declares the
/// trait shape so the events crate can document the mirror
/// contract without taking a dependency on the audit crate
/// (the dependency graph is `educore-audit → educore-events`,
/// not the reverse).
///
/// Wiring summary (in the umbrella's `subscribers.rs`):
///
/// 1. Construct `educore_audit::subscriber::AuditMirrorSubscriber`
///    wrapping the `AuditWriter` (which implements
///    `AuditSink`).
/// 2. Register it with the `SubscriberRegistry` alongside the
///    domain subscribers:
///    `registry.register(SubscriptionFilter::All, mirror);`
///
/// The mirror subscriber's `handle(envelope)` method:
///
/// - Builds an `AuditLogEntry` with `target_id = envelope.event_id`
///   and `action = "publish:<event_type>"`.
/// - Calls `audit_sink.write(entry).await` (returns the
///   wrapped AuditWriter's [`AuditSink::write`] result).
/// - Surfaces a typed error via [`crate::errors::EventError`]
///   on write failure so the dispatcher fails fast (per
///   FND-SEC-AUDIT-001, audit writes MUST NOT silently drop).
///
/// This module also exposes [`AuditMirrorConfig`] so consumers
/// can carry the configuration (which audit_log table to write
/// to, etc.) without referencing the audit crate directly.
pub struct AuditMirrorConfig {
    /// The action verb prefix for mirrored events.
    /// Default: `"publish"`. Consumers can override to
    /// `"event"` or `"bus.dispatch"` as the audit taxonomy
    /// requires.
    pub action_prefix: String,
}

impl Default for AuditMirrorConfig {
    fn default() -> Self {
        Self {
            action_prefix: "publish".to_owned(),
        }
    }
}

impl AuditMirrorConfig {
    /// Returns a new `AuditMirrorConfig` with the default
    /// action verb prefix.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
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
    use async_trait::async_trait;
    use educore_core::clock::{IdGenerator, SystemIdGen};
    use educore_core::ids::Identifier;
    use educore_core::value_objects::Timestamp;
    use futures::executor::block_on;
    use serde_json::json;
    use uuid::Uuid;

    /// A subscriber that records every envelope it sees.
    #[derive(Debug)]
    struct Recording {
        name: &'static str,
        seen: std::sync::Mutex<Vec<(String, Uuid)>>,
    }

    impl Recording {
        fn new(name: &'static str) -> Arc<Self> {
            Arc::new(Self {
                name,
                seen: std::sync::Mutex::new(Vec::new()),
            })
        }
        fn snapshot(&self) -> Vec<(String, Uuid)> {
            self.seen.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl Subscriber for Recording {
        fn name(&self) -> &'static str {
            self.name
        }
        async fn handle(&self, envelope: &EventEnvelope) -> Result<()> {
            self.seen
                .lock()
                .unwrap()
                .push((envelope.event_type.to_owned(), envelope.event_id.as_uuid()));
            Ok(())
        }
    }

    /// A subscriber that always fails.
    #[derive(Debug)]
    struct AlwaysFailing {
        name: &'static str,
    }

    #[async_trait]
    impl Subscriber for AlwaysFailing {
        fn name(&self) -> &'static str {
            self.name
        }
        async fn handle(&self, _envelope: &EventEnvelope) -> Result<()> {
            Err(educore_core::error::DomainError::validation("forced"))
        }
    }

    fn envelope(event_type: &'static str, aggregate_type: &'static str) -> EventEnvelope {
        let g = SystemIdGen;
        EventEnvelope {
            event_id: g.next_event_id(),
            event_type: event_type.to_owned(),
            schema_version: 1,
            school_id: g.next_school_id(),
            aggregate_id: g.next_uuid(),
            aggregate_type: aggregate_type.to_owned(),
            actor_id: g.next_user_id(),
            correlation_id: g.next_correlation_id(),
            causation_id: None,
            occurred_at: Timestamp::now(),
            published_at: None,
            payload: json!({}),
        }
    }

    #[test]
    fn filter_on_event_matches_exact_type_only() {
        let g = SystemIdGen;
        let env = envelope("documents.form_download.uploaded", "form_download");
        let f = SubscriptionFilter::on_event("documents.form_download.uploaded");
        assert!(f.matches(&env));
        assert!(!f.matches(&envelope(
            "documents.form_download.updated",
            "form_download"
        )));
        assert!(!f.matches(&envelope("academic.student.admitted", "student")));

        // School-pinned filter rejects other schools.
        let pinned = SubscriptionFilter::on_event_for_school(
            "documents.form_download.uploaded",
            env.school_id,
        );
        assert!(pinned.matches(&env));
        let other_school = SubscriptionFilter::on_event_for_school(
            "documents.form_download.uploaded",
            g.next_school_id(),
        );
        assert!(!other_school.matches(&env));
    }

    #[test]
    fn filter_on_aggregate_matches_aggregate_only() {
        let f = SubscriptionFilter::on_aggregate("form_download");
        let env = envelope("documents.form_download.uploaded", "form_download");
        assert!(f.matches(&env));
        let env = envelope("documents.form_download.updated", "form_download");
        assert!(f.matches(&env));
        let env = envelope("documents.postal_dispatch.dispatched", "postal_dispatch");
        assert!(!f.matches(&env));
    }

    #[test]
    fn filter_default_matches_everything() {
        let f = SubscriptionFilter::default();
        let env = envelope("anything.at.all", "whatever");
        assert!(f.matches(&env));
    }

    #[test]
    fn dispatch_fans_out_to_all_matching_subscribers() {
        let mut registry = SubscriberRegistry::new();
        let a = Recording::new("a");
        let b = Recording::new("b");
        registry.register(
            SubscriptionFilter::on_event("documents.form_download.uploaded"),
            a.clone(),
        );
        registry.register(
            SubscriptionFilter::on_event("documents.form_download.uploaded"),
            b.clone(),
        );
        let env = envelope("documents.form_download.uploaded", "form_download");
        let stats = block_on(registry.dispatch(&env)).unwrap();
        assert_eq!(stats.delivered, 2);
        assert_eq!(stats.skipped, 0);
        assert!(stats.is_ok());
        assert_eq!(a.snapshot().len(), 1);
        assert_eq!(b.snapshot().len(), 1);
    }

    #[test]
    fn dispatch_skips_non_matching_subscribers() {
        let mut registry = SubscriberRegistry::new();
        let matching = Recording::new("matching");
        let non_matching = Recording::new("non_matching");
        registry.register(
            SubscriptionFilter::on_event("documents.form_download.uploaded"),
            matching.clone(),
        );
        registry.register(
            SubscriptionFilter::on_event("academic.student.admitted"),
            non_matching.clone(),
        );
        let env = envelope("documents.form_download.uploaded", "form_download");
        let stats = block_on(registry.dispatch(&env)).unwrap();
        assert_eq!(stats.delivered, 1);
        assert_eq!(stats.skipped, 1);
        assert_eq!(matching.snapshot().len(), 1);
        assert_eq!(non_matching.snapshot().len(), 0);
    }

    #[test]
    fn dispatch_records_failure_but_continues_to_peers() {
        let mut registry = SubscriberRegistry::new();
        let a = Recording::new("a");
        registry.register(
            SubscriptionFilter::on_event("documents.form_download.uploaded"),
            Arc::new(AlwaysFailing { name: "failer" }),
        );
        registry.register(
            SubscriptionFilter::on_event("documents.form_download.uploaded"),
            a.clone(),
        );
        let env = envelope("documents.form_download.uploaded", "form_download");
        let stats = block_on(registry.dispatch(&env)).unwrap();
        assert_eq!(stats.delivered, 1);
        assert_eq!(stats.failures.len(), 1);
        assert_eq!(stats.failures[0].subscriber, "failer");
        assert!(!stats.is_ok());
        assert_eq!(a.snapshot().len(), 1);
    }

    #[test]
    fn empty_registry_dispatch_is_noop() {
        let registry = SubscriberRegistry::new();
        let env = envelope("documents.form_download.uploaded", "form_download");
        let stats = block_on(registry.dispatch(&env)).unwrap();
        assert_eq!(stats.delivered, 0);
        assert_eq!(stats.skipped, 0);
        assert!(stats.failures.is_empty());
        assert!(stats.is_ok());
    }

    #[test]
    fn registry_len_and_is_empty() {
        let mut registry = SubscriberRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
        registry.register(SubscriptionFilter::on_event("x"), Recording::new("r"));
        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn same_subscriber_can_be_registered_for_multiple_event_types() {
        let mut registry = SubscriberRegistry::new();
        let rec = Recording::new("multi");
        registry.register(
            SubscriptionFilter::on_event("documents.form_download.uploaded"),
            rec.clone(),
        );
        registry.register(
            SubscriptionFilter::on_event("documents.form_download.updated"),
            rec.clone(),
        );
        let env1 = envelope("documents.form_download.uploaded", "form_download");
        let env2 = envelope("documents.form_download.updated", "form_download");
        let s1 = block_on(registry.dispatch(&env1)).unwrap();
        let s2 = block_on(registry.dispatch(&env2)).unwrap();
        assert_eq!(s1.delivered, 1);
        assert_eq!(s2.delivered, 1);
        assert_eq!(rec.snapshot().len(), 2);
    }

    #[test]
    fn dispatch_stats_total_counts_all_subscribers() {
        let mut registry = SubscriberRegistry::new();
        let matching = Recording::new("m");
        let non_matching = Recording::new("nm");
        registry.register(
            SubscriptionFilter::on_event("academic.student.admitted"),
            matching,
        );
        registry.register(
            SubscriptionFilter::on_event("academic.student.withdrawn"),
            non_matching,
        );
        let env = envelope("academic.student.admitted", "student");
        let stats = block_on(registry.dispatch(&env)).unwrap();
        assert_eq!(stats.total(), 2);
        assert_eq!(stats.delivered, 1);
        assert_eq!(stats.skipped, 1);
    }
}
