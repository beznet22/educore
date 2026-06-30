//! # PollingIntegration adapter
//!
//! Per `docs/ports/integrations.md` § "Polling Adapter",
//! some integrations are best driven by a periodic poll
//! against a remote API rather than by webhook callbacks:
//!
//! - The integration endpoint exposes a list / cursor API
//!   ("give me the rows that changed since cursor X").
//! - The integration is firewalled such that inbound webhooks
//!   can't reach the consumer.
//! - The integration has no webhook concept at all (only
//!   polling).
//!
//! This module provides the polling adapter scaffold:
//!
//! - [`PollingSchedule`] — the cadence (`Interval` or `Cron`).
//! - [`PollingCursor`] — an opaque cursor token returned by
//!   the previous poll and re-presented on the next.
//! - [`PollingConfig`] — the schedule, cursor, target event,
//!   and auth wiring flag.
//! - [`PollingIntegration`] — an [`IntegrationGateway`]
//!   implementation that wraps an inner gateway and exposes a
//!   [`poll_once`](PollingIntegration::poll_once) helper.
//!
//! ## Implementation gap (PORT-INT-POLLING)
//!
//! This is the **stub** scaffolding: `poll_once` emits a
//! `tracing::warn!` and returns the current cursor unchanged.
//! Full schedule execution (tokio interval / cron parser)
//! and cursor advancement land in a later phase; the trait
//! shape and config types are stable now so consumers can
//! program against them.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;

use crate::errors::{IntegrationError, Result as IntegrationResult};
use crate::port::{
    HealthStatus, IntegrationCapability, IntegrationGateway, IntegrationHealth, IntegrationRequest,
    IntegrationResponse, IntegrationStatus,
};
use educore_core::value_objects::Timestamp;

/// The cadence of the polling loop. Two flavors:
///
/// - `Interval(Duration)` — fire every N seconds (the common
///   case; uses `tokio::time::interval`).
/// - `Cron(String)` — fire on a cron schedule (parsed by
///   `croner` or `tokio-cron-scheduler`; the stub stores the
///   cron expression but does not execute it).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PollingSchedule {
    /// Fire every N seconds.
    Interval(Duration),
    /// Fire on a cron schedule (5-field Unix cron expression,
    /// e.g. `"*/15 * * * *"` for every 15 minutes).
    Cron(String),
}

impl Default for PollingSchedule {
    fn default() -> Self {
        Self::Interval(Duration::from_secs(300))
    }
}

/// The opaque cursor token returned by the previous poll and
/// re-presented on the next. The remote API's response
/// includes the next cursor; the adapter stores it in
/// [`PollingConfig::cursor`] and re-uses it on the next call.
///
/// `PollingCursor` is intentionally opaque: the remote API
/// may encode the cursor as a timestamp, an offset, an OPAQUE
/// token, or a structured JSON blob — the adapter does not
/// parse it.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PollingCursor(pub String);

impl PollingCursor {
    /// Constructs a `PollingCursor` from a raw string.
    #[must_use]
    pub const fn new(s: String) -> Self {
        Self(s)
    }

    /// Returns `true` iff this is the empty / unset cursor
    /// (the first poll of a fresh integration).
    #[must_use]
    pub fn is_initial(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the cursor string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The polling configuration. The consumer constructs this at
/// integration-registration time; the adapter persists it via
/// the standard repository pattern.
#[derive(Debug, Clone)]
pub struct PollingConfig {
    /// The polling cadence.
    pub schedule: PollingSchedule,
    /// The current cursor (mutable; the adapter advances this
    /// after each successful poll).
    pub cursor: PollingCursor,
    /// The integration target event (the event the polling
    /// loop synthesizes per fetched row, e.g.
    /// `"integration.row.fetched"`).
    pub target_event: String,
    /// Whether the remote API requires OAuth2 client-credentials
    /// auth. The actual token cache wiring lands in the OAuth2
    /// helper (see [`crate::oauth2`]); the polling adapter
    /// reads this flag to decide whether to attach a
    /// `Bearer` token to outbound polls.
    pub auth_required: bool,
}

impl PollingConfig {
    /// Constructs a new `PollingConfig` with the given
    /// schedule, cursor, target event, and auth-required flag.
    #[must_use]
    pub const fn new(
        schedule: PollingSchedule,
        cursor: PollingCursor,
        target_event: String,
        auth_required: bool,
    ) -> Self {
        Self {
            schedule,
            cursor,
            target_event,
            auth_required,
        }
    }
}

/// The polling adapter. Wraps an inner [`IntegrationGateway`]
/// (the one that performs the actual HTTP / SDK call) and
/// adds a cursor / schedule layer on top.
#[derive(Debug)]
pub struct PollingIntegration<G: IntegrationGateway> {
    /// The wrapped gateway (performs the actual I/O).
    inner: G,
    /// The polling configuration.
    config: Arc<Mutex<PollingConfig>>,
}

impl<G: IntegrationGateway> PollingIntegration<G> {
    /// Constructs a new `PollingIntegration` wrapping `inner`
    /// with the given config.
    pub fn new(inner: G, config: PollingConfig) -> Self {
        Self {
            inner,
            config: Arc::new(Mutex::new(config)),
        }
    }

    /// Returns a snapshot of the current cursor.
    #[must_use]
    pub fn cursor(&self) -> PollingCursor {
        self.config
            .lock()
            .map(|c| c.cursor.clone())
            .unwrap_or_default()
    }

    /// Updates the cursor (typically called by the polling
    /// loop after a successful poll that returned a new
    /// cursor token).
    pub fn set_cursor(&self, new_cursor: PollingCursor) -> IntegrationResult<()> {
        let mut guard = self.config.lock().map_err(|e| {
            IntegrationError::Provider(format!("polling config mutex poisoned: {e}"))
        })?;
        guard.cursor = new_cursor;
        Ok(())
    }

    /// Returns a snapshot of the current schedule.
    #[must_use]
    pub fn schedule(&self) -> PollingSchedule {
        self.config
            .lock()
            .map(|c| c.schedule.clone())
            .unwrap_or_default()
    }

    /// Returns `true` iff the polling config requires OAuth2
    /// auth.
    #[must_use]
    pub fn auth_required(&self) -> bool {
        self.config.lock().map(|c| c.auth_required).unwrap_or(false)
    }

    /// Returns a snapshot of the configured target event.
    #[must_use]
    pub fn target_event(&self) -> String {
        self.config
            .lock()
            .map(|c| c.target_event.clone())
            .unwrap_or_default()
    }

    /// Stub poll-once helper. The full implementation lands
    /// in a later phase; for now this emits a `tracing::warn!`
    /// and returns the current cursor unchanged so callers
    /// can wire up the polling loop without blocking on the
    /// real implementation.
    pub async fn poll_once(&self) -> IntegrationResult<PollingCursor> {
        let cursor = self.cursor();
        let target = self.target_event();
        tracing::warn!(
            target_event = %target,
            cursor = %cursor.as_str(),
            "PollingIntegration::poll_once is a stub — no I/O performed",
        );
        Ok(cursor)
    }
}

#[async_trait]
impl<G: IntegrationGateway + Send + Sync + 'static> IntegrationGateway for PollingIntegration<G> {
    async fn invoke(&self, request: IntegrationRequest) -> IntegrationResult<IntegrationResponse> {
        // Delegate the underlying invoke to the wrapped gateway.
        // The polling-specific behaviour lives in `poll_once`;
        // single-shot `invoke` calls pass through unchanged.
        self.inner.invoke(request).await
    }

    async fn list_capabilities(&self) -> IntegrationResult<Vec<IntegrationCapability>> {
        self.inner.list_capabilities().await
    }

    async fn health(&self) -> IntegrationResult<IntegrationHealth> {
        // The polling adapter's health is the inner gateway's
        // health PLUS a "schedule is configured" check.
        let inner_health = self.inner.health().await?;
        let schedule_ok = self
            .config
            .lock()
            .map(|c| !matches!(c.schedule, PollingSchedule::Cron(ref expr) if expr.is_empty()))
            .unwrap_or(false);
        Ok(IntegrationHealth {
            status: if schedule_ok {
                inner_health.status
            } else {
                HealthStatus::Degraded
            },
            last_checked_at: Timestamp::now(),
            message: if schedule_ok {
                inner_health.message
            } else {
                Some("polling schedule is empty cron expression".to_owned())
            },
        })
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
    use serde_json::Value as JsonValue;

    use crate::port::{
        HealthStatus, IntegrationAction, IntegrationCapability, IntegrationId, IntegrationRequest,
        IntegrationResponse, IntegrationStatus,
    };
    use educore_core::clock::IdGenerator;
    use educore_core::ids::{SchoolId, UserId};
    use educore_core::tenant::{Locale, TenantContext, TimeZone, UserType};
    use educore_core::value_objects::Timestamp;

    #[derive(Debug)]
    struct StubGateway;

    #[async_trait]
    impl IntegrationGateway for StubGateway {
        async fn invoke(
            &self,
            _request: IntegrationRequest,
        ) -> IntegrationResult<IntegrationResponse> {
            Ok(IntegrationResponse {
                status: IntegrationStatus::Success,
                output: None,
                error: None,
                duration: chrono::Duration::zero(),
                cost: None,
                metadata: std::collections::BTreeMap::new(),
            })
        }
        async fn list_capabilities(&self) -> IntegrationResult<Vec<IntegrationCapability>> {
            Ok(Vec::new())
        }
        async fn health(&self) -> IntegrationResult<IntegrationHealth> {
            Ok(IntegrationHealth {
                status: HealthStatus::Healthy,
                last_checked_at: Timestamp::now(),
                message: None,
            })
        }
    }

    fn sample_tenant() -> TenantContext {
        let g = educore_core::clock::SystemIdGen;
        TenantContext {
            school_id: g.next_school_id(),
            actor_id: g.next_user_id(),
            session_id: None,
            correlation_id: g.next_correlation_id(),
            causation_id: None,
            user_type: UserType::System,
            locale: Locale::default(),
            timezone: TimeZone::default(),
        }
    }

    fn sample_request() -> IntegrationRequest {
        let g = educore_core::clock::SystemIdGen;
        IntegrationRequest {
            tenant: sample_tenant(),
            integration: IntegrationId::new("polling-test"),
            action: IntegrationAction::new("default"),
            input: JsonValue::Null,
            timeout: Some(chrono::Duration::seconds(30)),
            idempotency_key: g.next_idempotency_key(),
            correlation_id: g.next_correlation_id(),
        }
    }

    fn sample_config() -> PollingConfig {
        PollingConfig::new(
            PollingSchedule::Interval(Duration::from_secs(60)),
            PollingCursor::default(),
            "integration.row.fetched".to_owned(),
            false,
        )
    }

    #[tokio::test]
    async fn new_constructs_with_initial_cursor() {
        let p = PollingIntegration::new(StubGateway, sample_config());
        assert!(p.cursor().is_initial());
    }

    #[tokio::test]
    async fn set_cursor_updates_state() {
        let p = PollingIntegration::new(StubGateway, sample_config());
        p.set_cursor(PollingCursor::new("abc123".to_owned()))
            .expect("set");
        assert_eq!(p.cursor().as_str(), "abc123");
    }

    #[tokio::test]
    async fn schedule_accessor_returns_configured_value() {
        let p = PollingIntegration::new(StubGateway, sample_config());
        assert_eq!(
            p.schedule(),
            PollingSchedule::Interval(Duration::from_secs(60))
        );
    }

    #[tokio::test]
    async fn auth_required_accessor() {
        let p = PollingIntegration::new(StubGateway, sample_config());
        assert!(!p.auth_required());
    }

    #[tokio::test]
    async fn target_event_accessor() {
        let p = PollingIntegration::new(StubGateway, sample_config());
        assert_eq!(p.target_event(), "integration.row.fetched");
    }

    #[tokio::test]
    async fn poll_once_returns_current_cursor_unchanged() {
        let p = PollingIntegration::new(StubGateway, sample_config());
        p.set_cursor(PollingCursor::new("cursor-x".to_owned()))
            .expect("set");
        let cursor = p.poll_once().await.expect("poll_once");
        assert_eq!(cursor.as_str(), "cursor-x");
    }

    #[tokio::test]
    async fn invoke_delegates_to_inner() {
        let p = PollingIntegration::new(StubGateway, sample_config());
        let request = sample_request();
        let response = p.invoke(request).await.expect("invoke");
        assert_eq!(response.status, IntegrationStatus::Success);
    }

    #[tokio::test]
    async fn health_propagates_inner_health() {
        let p = PollingIntegration::new(StubGateway, sample_config());
        let h = p.health().await.expect("health");
        assert_eq!(h.status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn health_degrades_on_empty_cron() {
        let config = PollingConfig::new(
            PollingSchedule::Cron(String::new()),
            PollingCursor::default(),
            "x".to_owned(),
            false,
        );
        let p = PollingIntegration::new(StubGateway, config);
        let h = p.health().await.expect("health");
        assert_eq!(h.status, HealthStatus::Degraded);
    }

    #[test]
    fn polling_cursor_is_initial_empty() {
        assert!(PollingCursor::default().is_initial());
        assert!(!PollingCursor::new("x".to_owned()).is_initial());
    }

    #[test]
    fn polling_cursor_as_str() {
        assert_eq!(PollingCursor::new("foo".to_owned()).as_str(), "foo");
    }

    #[test]
    fn polling_schedule_default_is_5min_interval() {
        assert_eq!(
            PollingSchedule::default(),
            PollingSchedule::Interval(Duration::from_secs(300))
        );
    }

    #[test]
    fn trait_object_safe() {
        // Compile-time check: a `Box<dyn IntegrationGateway>` is
        // valid (the trait is object-safe).
        let _: Box<dyn IntegrationGateway> = Box::new(StubGateway);
    }

    // Suppress dead_code for unused imports of test helpers
    #[allow(dead_code)]
    fn _unused(_: SchoolId, _: UserId) {}
}
