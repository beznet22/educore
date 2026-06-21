//! # Integration port
//!
//! The [`IntegrationGateway`] trait is the engine's universal seam
//! for outbound calls to systems that don't fit the more specific
//! ports (auth, payment, notification, file storage). LMS sync,
//! video conferencing, identity federation, custom webhooks, and
//! polling adapters all hang off this single trait.
//!
//! Per `docs/ports/integrations.md`:
//!
//! - The trait is **object-safe** — adapters are typically held as
//!   `Arc<dyn IntegrationGateway>` so consumers can swap
//!   implementations without recompiling.
//! - All three methods return [`Result<_, IntegrationError>`] so
//!   the universal error taxonomy covers every failure mode.
//! - The port is deliberately generic: each integration defines
//!   its own command/event shapes inside the engine and a
//!   corresponding adapter that performs the actual I/O.
//!
//! Per ADR-015 (external crate selection): port-adapter code is
//! allowed to use `serde_json::Value` and other JSON-shaped types.
//! This crate is **not** a domain crate; the "no `serde_json::Value`
//! in domain code" rule does not apply here.

#![allow(clippy::module_name_repetitions)]

use std::collections::BTreeMap;
use std::fmt;

use async_trait::async_trait;
use chrono::Duration as ChronoDuration;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use educore_core::ids::{CorrelationId, IdempotencyKey};
use educore_core::tenant::TenantContext;
use educore_core::value_objects::Timestamp;
use educore_rbac::value_objects::Capability;

use crate::errors::{IntegrationError, Result};

// =============================================================================
// Identifier newtypes
// =============================================================================

/// A typed identifier for a registered integration (e.g. `"twilio"`,
/// `"google_classroom"`, `"stripe_connect"`).
///
/// The newtype prevents accidental cross-use with
/// [`IntegrationAction`] or any other `String`-shaped id in the
/// engine. The inner `String` is opaque — adapters MUST NOT parse
/// it; consumers MUST NOT compare two ids for "equality of
/// meaning" beyond byte equality.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IntegrationId(String);

impl IntegrationId {
    /// Constructs an `IntegrationId` from an opaque string.
    ///
    /// The constructor is infallible at the type level. Adapters
    /// that need validation (e.g. "must be ASCII lowercase")
    /// perform it inside their `list_capabilities()`-driven
    /// config load, not at every construction site.
    #[must_use]
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for IntegrationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for IntegrationId {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl From<String> for IntegrationId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// A typed identifier for a single action a registered integration
/// can perform (e.g. `"send_sms"`, `"link_course"`, `"end_meeting"`).
///
/// Each [`IntegrationId`] exposes a closed set of actions; the set
/// is reported by
/// [`IntegrationGateway::list_capabilities`] and the UIs / AI-agent
/// tool catalogs use it to drive validation. Adapters MUST reject
/// unknown actions with
/// [`IntegrationError::InvalidInput`](crate::errors::IntegrationError::InvalidInput).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IntegrationAction(String);

impl IntegrationAction {
    /// Constructs an `IntegrationAction` from an opaque string.
    #[must_use]
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for IntegrationAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for IntegrationAction {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl From<String> for IntegrationAction {
    fn from(s: String) -> Self {
        Self(s)
    }
}

// =============================================================================
// Schema reference
// =============================================================================

/// A reference to the input or output schema of an action.
///
/// Per `docs/ports/integrations.md` § "IntegrationCapability": the
/// engine exposes the schema so UIs and AI-agent tool catalogs can
/// build typed forms / tool invocations without hand-coding each
/// integration. The schema is stored at the URL or path pointed to
/// by `location` and identified by `format`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaRef {
    /// Where to fetch the schema document (URL or relative path
    /// inside the engine's asset store).
    pub location: String,
    /// The schema format. `JsonSchema` (Draft 7 / 2020-12),
    /// `OpenApi` (3.0 / 3.1), or `Protobuf` are the supported
    /// formats. Other values are reserved for future use.
    pub format: SchemaFormat,
}

/// The schema language a [`SchemaRef`] points at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchemaFormat {
    /// JSON Schema (Draft 7 or 2020-12).
    #[serde(rename = "json_schema")]
    JsonSchema,
    /// OpenAPI 3.x.
    #[serde(rename = "openapi")]
    OpenApi,
    /// Protocol Buffers (`.proto`).
    #[serde(rename = "protobuf")]
    Protobuf,
}

// =============================================================================
// Retry policy
// =============================================================================

/// How the adapter should retry transient failures (5xx, network,
/// timeouts). Permanent failures (4xx other than 429) are returned
/// immediately without consulting the policy.
///
/// Per `docs/ports/integrations.md` § "Retry Policy".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetryPolicy {
    /// No retries — propagate every error immediately.
    None,

    /// Linear backoff: retry every `interval` up to `max_retries`
    /// times.
    Linear {
        /// Maximum number of retry attempts after the first call.
        max_retries: u32,
        /// Delay between attempts.
        interval: ChronoDuration,
    },

    /// Exponential backoff: wait `base * 2^n` between attempts,
    /// capped at `max`. `max_retries` counts attempts after the
    /// first call.
    Exponential {
        /// Maximum number of retry attempts after the first call.
        max_retries: u32,
        /// Base delay for the first retry.
        base: ChronoDuration,
        /// Upper bound on the delay between any two attempts.
        max: ChronoDuration,
    },
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::Exponential {
            max_retries: 3,
            base: ChronoDuration::seconds(1),
            max: ChronoDuration::seconds(30),
        }
    }
}

// =============================================================================
// Request
// =============================================================================

/// The input to [`IntegrationGateway::invoke`].
///
/// Carries everything an adapter needs to dispatch a single
/// integration call without consulting global state: tenant
/// context, which integration to invoke, what action to perform,
/// the JSON payload, the idempotency key (so retried calls produce
/// the same provider-side result), the correlation id (for log
/// stitching), and an optional per-call timeout override.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationRequest {
    /// The active tenant. Adapters look up the integration's
    /// per-tenant configuration using `tenant.school_id`.
    pub tenant: TenantContext,

    /// Which registered integration to call.
    pub integration: IntegrationId,

    /// Which action to perform on that integration.
    pub action: IntegrationAction,

    /// The JSON payload. The shape is constrained by the
    /// capability's `input_schema` (see
    /// [`IntegrationCapability`]).
    pub input: JsonValue,

    /// Caller-provided idempotency token. Two calls with the same
    /// `(school_id, integration, action, idempotency_key)` MUST
    /// produce the same provider-side effect and return the same
    /// response. Per `docs/schemas/command-schema.md` § 1.2.
    pub idempotency_key: IdempotencyKey,

    /// Correlation id for log stitching across the engine. The
    /// adapter copies it into every outbound HTTP header
    /// (`X-Correlation-Id`) and every audit log entry.
    pub correlation_id: CorrelationId,

    /// Optional per-call timeout override. `None` means "use the
    /// adapter default".
    pub timeout: Option<ChronoDuration>,
}

// =============================================================================
// Response
// =============================================================================

/// The outcome of a single [`IntegrationGateway::invoke`] call.
///
/// A successful call has `status == IntegrationStatus::Success`
/// (or `Accepted` for async deliveries) and a populated `output`.
/// A failed call has `status == IntegrationStatus::Failed` (or
/// `RateLimited` / `TimedOut`) and a populated `error`.
///
/// `duration` is always populated (even on failure) for billing
/// and SLO reporting. `cost` is populated only for metered
/// integrations. `metadata` carries provider-specific headers
/// (request id, rate-limit remaining, etc.) for log stitching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationResponse {
    /// High-level outcome.
    pub status: IntegrationStatus,

    /// The JSON payload returned by the provider. `None` for
    /// `Accepted`, `RateLimited`, and `TimedOut` responses where
    /// the provider returned no body.
    pub output: Option<JsonValue>,

    /// The structured error if `status` is `Failed`, `RateLimited`,
    /// or `TimedOut`. `None` for `Success` / `Accepted`.
    pub error: Option<IntegrationError>,

    /// Wall-clock duration of the call. Always populated.
    pub duration: ChronoDuration,

    /// Provider-side cost (metered integrations). `None` when the
    /// provider does not charge for the action or did not return
    /// a cost in the response.
    pub cost: Option<IntegrationCost>,

    /// Provider-specific metadata (request id, rate-limit
    /// remaining, traceparent, etc.). Always non-empty for a
    /// response that actually reached the provider.
    pub metadata: BTreeMap<String, String>,
}

/// Provider-side monetary cost of a single integration call.
///
/// Mirrors the finance domain's [`Money`](educore_core::value_objects::Timestamp)
/// shape but is duplicated here so this crate does not need a
/// hard dependency on `educore-finance`. Adapters that want to
/// convert to the engine's accounting record should call
/// `Money::new(cost.currency, cost.amount_minor)`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationCost {
    /// The amount in minor units (cents, paisa, etc.). Always
    /// non-negative; refunds are reported as negative `cost` on a
    /// later response, not as a negative value here.
    pub amount_minor: i64,
    /// ISO 4217 currency code (e.g. `"USD"`, `"INR"`).
    pub currency: String,
}

/// The high-level outcome category of a single integration call.
///
/// `Accepted` is distinct from `Success` because the provider has
/// acknowledged the request but the work will complete
/// asynchronously — the caller should subscribe to the relevant
/// webhook / polling channel for completion. `RateLimited` and
/// `TimedOut` are surfaced as distinct from `Failed` so callers
/// can pick the right retry strategy without parsing `error`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IntegrationStatus {
    /// The provider processed the call synchronously and the
    /// payload is in `output`.
    Success,
    /// The provider accepted the call for asynchronous processing.
    /// The result will arrive via webhook or polling.
    Accepted,
    /// The provider signalled rate limiting (HTTP 429 / equivalent).
    /// Caller should back off and retry per the adapter's
    /// [`RetryPolicy`].
    RateLimited,
    /// The call reached the provider and the provider returned a
    /// failure. `error` carries the structured cause.
    Failed,
    /// The adapter exhausted its timeout without a response.
    /// `error` carries the [`IntegrationError::Timeout`](crate::errors::IntegrationError::Timeout)
    /// variant.
    TimedOut,
}

impl IntegrationStatus {
    /// Returns `true` for the terminal-success states.
    #[must_use]
    pub const fn is_success(&self) -> bool {
        matches!(self, Self::Success | Self::Accepted)
    }

    /// Returns `true` if the caller should consult [`RetryPolicy`]
    /// before re-issuing the same `IntegrationRequest`.
    #[must_use]
    pub const fn is_retryable(&self) -> bool {
        matches!(self, Self::RateLimited | Self::TimedOut)
    }
}

// =============================================================================
// Capability
// =============================================================================

/// A single action exposed by a registered integration.
///
/// Returned by [`IntegrationGateway::list_capabilities`] so UIs
/// and AI-agent tool catalogs can render typed forms / invoke the
/// action without hand-coding each integration.
///
/// Per `docs/ports/integrations.md` § "IntegrationCapability".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationCapability {
    /// Which integration exposes this action.
    pub integration: IntegrationId,

    /// The action name. Matches the `IntegrationAction` string the
    /// caller would put in [`IntegrationRequest::action`].
    pub action: IntegrationAction,

    /// Human-readable description for UI rendering and AI-agent
    /// tool catalogs. Free-form, English, ≤ 200 chars by
    /// convention.
    pub description: String,

    /// Schema for `IntegrationRequest::input`. `None` if the
    /// action accepts no input or the schema is not exposed.
    pub input_schema: Option<SchemaRef>,

    /// Schema for `IntegrationResponse::output`. `None` if the
    /// action produces no output or the schema is not exposed.
    pub output_schema: Option<SchemaRef>,

    /// The engine capabilities the caller MUST hold to invoke this
    /// action (e.g. `Capability::LmsCourseLink`). The adapter does
    /// not enforce these — the engine's RBAC layer does — but the
    /// port exposes them so UIs can hide actions the caller can't
    /// use.
    pub required_capabilities: Vec<Capability>,
}

// =============================================================================
// Health
// =============================================================================

/// The overall health of the integration gateway as reported by
/// [`IntegrationGateway::health`].
///
/// The `last_checked_at` timestamp is the wall-clock instant the
/// adapter last performed a liveness probe; the engine records it
/// into the operational dashboards so silent adapters don't
/// masquerade as healthy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationHealth {
    /// High-level liveness verdict.
    pub status: HealthStatus,

    /// When the adapter last exercised the integration (liveness
    /// probe, smoke call, or actual invocation). Never `None` —
    /// adapters that have never run a probe report `Timestamp::epoch()`
    /// so consumers can render "never" explicitly.
    pub last_checked_at: Timestamp,

    /// Optional human-readable detail (latency to provider, error
    /// snippet from the last probe, etc.). Always `None` for
    /// [`HealthStatus::Healthy`].
    pub message: Option<String>,
}

/// The high-level verdict of [`IntegrationHealth`].
///
/// `Degraded` is distinct from `Unhealthy`: the gateway is
/// operational but at least one integration is failing or rate-
/// limited, so callers should expect some `IntegrationRequest`s
/// to fail. `Unhealthy` means no integration can be reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Every registered integration is reachable and responsive.
    Healthy,
    /// The gateway itself is up; at least one integration is
    /// failing, rate-limited, or slow. Most calls will still
    /// succeed.
    Degraded,
    /// The gateway itself is down or no integration is reachable.
    /// All `IntegrationRequest`s will fail.
    Unhealthy,
}

// =============================================================================
// Gateway trait
// =============================================================================

/// The integration port. Connect the engine to external systems
/// that don't fit the more specific ports (auth, payment,
/// notification, file storage): LMS sync, video conferencing,
/// identity providers, custom webhooks, polling adapters.
///
/// The trait is **object-safe** — adapters are typically held as
/// `Arc<dyn IntegrationGateway>` so consumers can swap
/// implementations without recompiling.
///
/// Per `docs/ports/integrations.md` § "Trait: `IntegrationGateway`".
#[async_trait]
pub trait IntegrationGateway: Send + Sync + fmt::Debug {
    /// Invoke a single integration action.
    ///
    /// The returned [`IntegrationResponse`] always carries a
    /// populated `duration`. `error` is `Some` for any
    /// non-success / non-accepted status; `output` is `Some` for
    /// `Success` (and may be `Some` for `Accepted` if the provider
    /// returned an acknowledgement body).
    async fn invoke(&self, request: IntegrationRequest) -> Result<IntegrationResponse>;

    /// Enumerate every action every registered integration
    /// exposes. The engine calls this at startup, on cache TTL
    /// expiry, and when an AI agent requests the tool catalog.
    async fn list_capabilities(&self) -> Result<Vec<IntegrationCapability>>;

    /// Report liveness of the gateway and every registered
    /// integration. Called by the engine's operational dashboards
    /// every 30 s.
    async fn health(&self) -> Result<IntegrationHealth>;
}

// =============================================================================
// Tests
// =============================================================================

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
    fn identifier_newtypes_round_trip_through_strings() {
        let id = IntegrationId::new("twilio");
        assert_eq!(id.as_str(), "twilio");
        assert_eq!(id.to_string(), "twilio");
        assert_eq!(IntegrationId::from("stripe"), IntegrationId::new("stripe"));
        assert_eq!(IntegrationId::from(String::from("zoom")), IntegrationId::new("zoom"));

        let action = IntegrationAction::new("send_sms");
        assert_eq!(action.as_str(), "send_sms");
        assert_eq!(action.to_string(), "send_sms");
    }

    #[test]
    fn integration_status_helpers_classify_correctly() {
        assert!(IntegrationStatus::Success.is_success());
        assert!(IntegrationStatus::Accepted.is_success());
        assert!(!IntegrationStatus::Failed.is_success());

        assert!(IntegrationStatus::RateLimited.is_retryable());
        assert!(IntegrationStatus::TimedOut.is_retryable());
        assert!(!IntegrationStatus::Success.is_retryable());
        assert!(!IntegrationStatus::Failed.is_retryable());
        assert!(!IntegrationStatus::Accepted.is_retryable());
    }

    #[test]
    fn retry_policy_default_is_exponential() {
        let policy = RetryPolicy::default();
        match policy {
            RetryPolicy::Exponential { max_retries, base, max } => {
                assert_eq!(max_retries, 3);
                assert_eq!(base, ChronoDuration::seconds(1));
                assert_eq!(max, ChronoDuration::seconds(30));
            }
            other => panic!("default must be Exponential, got {other:?}"),
        }
    }

    #[test]
    fn schema_format_serializes_to_lowercase_string() {
        let json = serde_json::to_string(&SchemaFormat::JsonSchema).unwrap();
        assert_eq!(json, "\"json_schema\"");
        let json = serde_json::to_string(&SchemaFormat::OpenApi).unwrap();
        assert_eq!(json, "\"openapi\"");
        let json = serde_json::to_string(&SchemaFormat::Protobuf).unwrap();
        assert_eq!(json, "\"protobuf\"");
    }

    #[test]
    fn health_status_distinguishes_degraded_from_unhealthy() {
        assert_ne!(HealthStatus::Healthy, HealthStatus::Degraded);
        assert_ne!(HealthStatus::Degraded, HealthStatus::Unhealthy);
        assert_ne!(HealthStatus::Healthy, HealthStatus::Unhealthy);
    }

    #[test]
    fn integration_cost_round_trips_through_json() {
        let cost = IntegrationCost {
            amount_minor: 1_00,
            currency: "USD".into(),
        };
        let json = serde_json::to_string(&cost).unwrap();
        let parsed: IntegrationCost = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, cost);
    }
}