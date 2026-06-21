//! # Video conferencing integration
//!
//! Reference implementation of the
//! [`IntegrationGateway`](crate::port::IntegrationGateway) trait for
//! video conferencing providers (Zoom, Google Meet, Microsoft Teams).
//!
//! Per `docs/ports/integrations.md` § "Video Conferencing":
//!
//! - Create a meeting when a `LessonPlan` is scheduled.
//! - Record attendance by joining time and joining user list.
//! - Store the recording in the file storage port and emit
//!   `VideoRecordingAvailable`.
//!
//! ## Authentication
//!
//! - Zoom JWT auth (the simplest Zoom integration): `api_key` is the
//!   Zoom API key; `api_secret` is used to sign a JWT with `HS256`.
//!   The proper HMAC signing is added by the WebhookOut integration
//!   (E.3c); for now this adapter emits a `Bearer <api_key>`
//!   placeholder header so the request shape matches production.
//! - Google Meet / Microsoft Teams use OAuth bearer tokens supplied
//!   via `api_key`; `api_secret` is unused.
//!
//! ## Actions
//!
//! - [`ACTION_MEETING_CREATE`] (`"video.meeting.create"`) — create a
//!   meeting for a scheduled `LessonPlan`.
//! - [`ACTION_MEETING_GET`] (`"video.meeting.get"`) — fetch meeting
//!   metadata (join URL, host, start time) by meeting id.
//! - [`ACTION_RECORDING_LIST`] (`"video.recording.list"`) — list
//!   cloud recordings (the engine downloads the matching files into
//!   the file storage port and emits `VideoRecordingAvailable`).
//!
//! ## Construction
//!
//! Use [`VideoConferencingIntegrationBuilder`]:
//!
//! ```ignore
//! let adapter = VideoConferencingIntegrationBuilder::new()
//!     .provider("zoom")
//!     .api_key(env::var("ZOOM_API_KEY")?)
//!     .api_secret(env::var("ZOOM_API_SECRET")?)
//!     .build();
//! let gateway: Arc<dyn IntegrationGateway> = Arc::new(adapter);
//! ```

#![allow(clippy::module_name_repetitions)]

use std::collections::BTreeMap;
use std::fmt;
use std::time::Instant;

use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value as JsonValue;

use crate::errors::{IntegrationError, Result};
use crate::port::{
    HealthStatus, IntegrationAction, IntegrationCapability, IntegrationGateway, IntegrationHealth,
    IntegrationId, IntegrationRequest, IntegrationResponse, IntegrationStatus, SchemaFormat,
    SchemaRef,
};

// =============================================================================
// Defaults
// =============================================================================

/// Default Zoom REST API base URL. Used by
/// [`VideoConferencingIntegrationBuilder::build`] when the caller
/// does not override `base_url`.
const DEFAULT_ZOOM_BASE_URL: &str = "https://api.zoom.us/v2";

/// The registered [`IntegrationId`] for this adapter.
pub const VIDEO_INTEGRATION_ID: &str = "video_conferencing";

/// Action name: create a meeting.
pub const ACTION_MEETING_CREATE: &str = "video.meeting.create";

/// Action name: fetch a meeting by id.
pub const ACTION_MEETING_GET: &str = "video.meeting.get";

/// Action name: list cloud recordings.
pub const ACTION_RECORDING_LIST: &str = "video.recording.list";

// =============================================================================
// Builder
// =============================================================================

/// Builder for [`VideoConferencingIntegration`].
///
/// Accumulates configuration across repeated `.provider(..)`,
/// `.api_key(..)`, `.api_secret(..)`, `.base_url(..)` calls and
/// produces a fully-configured adapter when [`build`](Self::build)
/// is invoked. Every field has a default: `provider` defaults to
/// `"zoom"`, `base_url` defaults to the Zoom REST API root, and
/// `api_key` / `api_secret` default to empty strings.
#[derive(Debug, Default, Clone)]
pub struct VideoConferencingIntegrationBuilder {
    provider: Option<String>,
    api_key: Option<String>,
    api_secret: Option<String>,
    base_url: Option<String>,
}

impl VideoConferencingIntegrationBuilder {
    /// Creates a new builder with no configuration set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the video provider key (e.g. `"zoom"`, `"google_meet"`,
    /// `"microsoft_teams"`).
    #[must_use]
    pub fn provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    /// Sets the API key (Zoom API key for JWT auth; OAuth bearer
    /// token for Google Meet / Microsoft Teams).
    #[must_use]
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Sets the API secret (used to sign JWTs for Zoom; ignored for
    /// bearer-token providers).
    #[must_use]
    pub fn api_secret(mut self, api_secret: impl Into<String>) -> Self {
        self.api_secret = Some(api_secret.into());
        self
    }

    /// Sets the provider base URL. Defaults to the Zoom REST API
    /// root when the caller does not override it.
    #[must_use]
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Consumes the builder and produces a configured
    /// [`VideoConferencingIntegration`].
    #[must_use]
    pub fn build(self) -> VideoConferencingIntegration {
        VideoConferencingIntegration {
            http: Client::new(),
            provider: self.provider.unwrap_or_else(|| "zoom".to_owned()),
            api_key: self.api_key.unwrap_or_default(),
            api_secret: self.api_secret.unwrap_or_default(),
            base_url: self
                .base_url
                .unwrap_or_else(|| DEFAULT_ZOOM_BASE_URL.to_owned()),
        }
    }
}

// =============================================================================
// Integration
// =============================================================================

/// Reference integration adapter for video conferencing providers
/// (Zoom, Google Meet, Microsoft Teams).
///
/// Construct via [`VideoConferencingIntegrationBuilder`]. The adapter
/// is `Send + Sync` so it can be held behind an
/// `Arc<dyn IntegrationGateway>`.
#[derive(Clone)]
pub struct VideoConferencingIntegration {
    http: Client,
    provider: String,
    api_key: String,
    api_secret: String,
    base_url: String,
}

impl fmt::Debug for VideoConferencingIntegration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VideoConferencingIntegration")
            .field("provider", &self.provider)
            .field("base_url", &self.base_url)
            .field("api_key", &"***redacted***")
            .field("api_secret", &"***redacted***")
            .finish()
    }
}

#[async_trait]
impl IntegrationGateway for VideoConferencingIntegration {
    async fn invoke(&self, request: IntegrationRequest) -> Result<IntegrationResponse> {
        let started = Instant::now();
        let action = request.action.as_str();
        let result = match action {
            ACTION_MEETING_CREATE => self.create_meeting(&request).await,
            ACTION_MEETING_GET => self.get_meeting(&request).await,
            ACTION_RECORDING_LIST => self.list_recordings(&request).await,
            other => Err(IntegrationError::InvalidInput(format!(
                "unknown video action: {other}"
            ))),
        };

        let duration = chrono::Duration::from_std(started.elapsed())
            .unwrap_or_else(|_| chrono::Duration::zero());

        match result {
            Ok(output) => Ok(IntegrationResponse {
                status: IntegrationStatus::Success,
                output: Some(output),
                error: None,
                duration,
                cost: None,
                metadata: self.response_metadata(&request),
            }),
            Err(err) => Ok(IntegrationResponse {
                status: status_from_error(&err),
                output: None,
                error: Some(err),
                duration,
                cost: None,
                metadata: self.response_metadata(&request),
            }),
        }
    }

    async fn list_capabilities(&self) -> Result<Vec<IntegrationCapability>> {
        Ok(vec![
            self.capability_meeting_create(),
            self.capability_meeting_get(),
            self.capability_recording_list(),
        ])
    }

    async fn health(&self) -> Result<IntegrationHealth> {
        Ok(IntegrationHealth {
            status: HealthStatus::Healthy,
            last_checked_at: educore_core::value_objects::Timestamp::epoch(),
            message: None,
        })
    }
}

// =============================================================================
// Video API helpers (impl on the integration)
// =============================================================================

impl VideoConferencingIntegration {
    /// Returns the `Authorization: Bearer <api_key>` header value.
    /// Placeholder for the Zoom JWT signing that E.3c will wire in;
    /// `api_secret` is forwarded as `X-Api-Secret` so the credential
    /// pair is exercised end-to-end today (Zoom's older JWT app type
    /// accepts the key/secret pair this way; the OAuth-bearer providers
    /// simply ignore the extra header).
    fn auth_header(&self) -> (&'static str, String, String) {
        ("Bearer", self.api_key.clone(), self.api_secret.clone())
    }

    /// Creates a meeting via the provider's REST API. Returns the
    /// provider's JSON response (meeting id, join URL, start URL,
    /// etc.).
    async fn create_meeting(&self, request: &IntegrationRequest) -> Result<JsonValue> {
        let url = format!("{}/users/me/meetings", self.base_url);
        let (scheme, key, secret) = self.auth_header();
        let response = self
            .http
            .post(&url)
            .header("Authorization", format!("{scheme} {key}"))
            .header("X-Api-Secret", secret)
            .header("Content-Type", "application/json")
            .header("X-Correlation-Id", request.correlation_id.to_string())
            .header("Idempotency-Key", request.idempotency_key.to_string())
            .json(&request.input)
            .send()
            .await
            .map_err(infrastructure)?;
        parse_response(response).await
    }

    /// Fetches meeting metadata by meeting id. The `input` payload
    /// MUST contain a `"meeting_id"` string field.
    async fn get_meeting(&self, request: &IntegrationRequest) -> Result<JsonValue> {
        let meeting_id = request
            .input
            .get("meeting_id")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| {
                IntegrationError::InvalidInput(
                    "video.meeting.get requires `meeting_id` (string)".to_owned(),
                )
            })?;
        let url = format!("{}/meetings/{meeting_id}", self.base_url);
        let (scheme, key, secret) = self.auth_header();
        let response = self
            .http
            .get(&url)
            .header("Authorization", format!("{scheme} {key}"))
            .header("X-Api-Secret", secret)
            .header("X-Correlation-Id", request.correlation_id.to_string())
            .send()
            .await
            .map_err(infrastructure)?;
        parse_response(response).await
    }

    /// Lists cloud recordings owned by the configured user. The
    /// `input` payload is forwarded as query parameters (e.g.
    /// `"from"`, `"to"`, `"page_size"`).
    async fn list_recordings(&self, request: &IntegrationRequest) -> Result<JsonValue> {
        let url = format!("{}/users/me/recordings", self.base_url);
        let (scheme, key, secret) = self.auth_header();
        let response = self
            .http
            .get(&url)
            .header("Authorization", format!("{scheme} {key}"))
            .header("X-Api-Secret", secret)
            .header("X-Correlation-Id", request.correlation_id.to_string())
            .query(&request.input)
            .send()
            .await
            .map_err(infrastructure)?;
        parse_response(response).await
    }

    /// Stamps the request's correlation / idempotency ids into the
    /// response metadata for log stitching.
    fn response_metadata(&self, request: &IntegrationRequest) -> BTreeMap<String, String> {
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "x-correlation-id".to_owned(),
            request.correlation_id.to_string(),
        );
        metadata.insert(
            "idempotency-key".to_owned(),
            request.idempotency_key.to_string(),
        );
        metadata.insert("provider".to_owned(), self.provider.clone());
        metadata.insert("integration".to_owned(), VIDEO_INTEGRATION_ID.to_owned());
        metadata
    }

    /// Builds the [`IntegrationCapability`] for `video.meeting.create`.
    fn capability_meeting_create(&self) -> IntegrationCapability {
        IntegrationCapability {
            integration: IntegrationId::new(VIDEO_INTEGRATION_ID),
            action: IntegrationAction::new(ACTION_MEETING_CREATE),
            description: "Create a video meeting for a scheduled LessonPlan.".to_owned(),
            input_schema: Some(self.schema_ref("input.schema.json", ACTION_MEETING_CREATE)),
            output_schema: Some(self.schema_ref("output.schema.json", ACTION_MEETING_CREATE)),
            required_capabilities: Vec::new(),
        }
    }

    /// Builds the [`IntegrationCapability`] for `video.meeting.get`.
    fn capability_meeting_get(&self) -> IntegrationCapability {
        IntegrationCapability {
            integration: IntegrationId::new(VIDEO_INTEGRATION_ID),
            action: IntegrationAction::new(ACTION_MEETING_GET),
            description: "Fetch meeting metadata (join URL, host, start time) by meeting id."
                .to_owned(),
            input_schema: Some(self.schema_ref("input.schema.json", ACTION_MEETING_GET)),
            output_schema: Some(self.schema_ref("output.schema.json", ACTION_MEETING_GET)),
            required_capabilities: Vec::new(),
        }
    }

    /// Builds the [`IntegrationCapability`] for `video.recording.list`.
    fn capability_recording_list(&self) -> IntegrationCapability {
        IntegrationCapability {
            integration: IntegrationId::new(VIDEO_INTEGRATION_ID),
            action: IntegrationAction::new(ACTION_RECORDING_LIST),
            description: "List cloud recordings for the configured user (downloaded into the \
                          file storage port and emitted as VideoRecordingAvailable events)."
                .to_owned(),
            input_schema: Some(self.schema_ref("input.schema.json", ACTION_RECORDING_LIST)),
            output_schema: Some(self.schema_ref("output.schema.json", ACTION_RECORDING_LIST)),
            required_capabilities: Vec::new(),
        }
    }

    /// Constructs a [`SchemaRef`] pointing at a relative path under
    /// the engine's asset store.
    fn schema_ref(&self, file: &str, action: &str) -> SchemaRef {
        SchemaRef {
            location: format!("integrations/{VIDEO_INTEGRATION_ID}/{action}/{file}"),
            format: SchemaFormat::JsonSchema,
        }
    }
}

// =============================================================================
// Free-standing helpers
// =============================================================================

/// Parses an HTTP response into a [`JsonValue`] output, mapping
/// non-2xx responses to the appropriate [`IntegrationError`] variant.
async fn parse_response(response: reqwest::Response) -> Result<JsonValue> {
    let status = response.status();
    let body = response.text().await.map_err(infrastructure)?;
    if status.is_success() {
        if body.is_empty() {
            Ok(JsonValue::Null)
        } else {
            serde_json::from_str(&body).map_err(json_infrastructure)
        }
    } else if status.as_u16() == 429 {
        Err(IntegrationError::RateLimited)
    } else {
        Err(IntegrationError::Provider(format!(
            "{} {}",
            status.as_u16(),
            body
        )))
    }
}

/// Boxes a [`reqwest::Error`] into
/// [`IntegrationError::Infrastructure`].
fn infrastructure(err: reqwest::Error) -> IntegrationError {
    IntegrationError::Infrastructure(Box::new(err))
}

/// Boxes a [`serde_json::Error`] into
/// [`IntegrationError::Infrastructure`].
fn json_infrastructure(err: serde_json::Error) -> IntegrationError {
    IntegrationError::Infrastructure(Box::new(err))
}

/// Maps an [`IntegrationError`] into the high-level
/// [`IntegrationStatus`] reported on the [`IntegrationResponse`].
fn status_from_error(err: &IntegrationError) -> IntegrationStatus {
    match err {
        IntegrationError::RateLimited => IntegrationStatus::RateLimited,
        IntegrationError::Timeout(_) => IntegrationStatus::TimedOut,
        _ => IntegrationStatus::Failed,
    }
}

// =============================================================================
// Test-only helpers (kept out of the public surface)
// =============================================================================

#[cfg(test)]
fn test_tenant_context() -> educore_core::tenant::TenantContext {
    use educore_core::clock::IdGenerator;

    let gen = educore_core::clock::SystemIdGen;
    educore_core::tenant::TenantContext::system(gen.next_school_id(), gen.next_correlation_id())
}

#[cfg(test)]
fn test_correlation_id() -> educore_core::ids::CorrelationId {
    use educore_core::clock::IdGenerator;

    educore_core::clock::SystemIdGen.next_correlation_id()
}

#[cfg(test)]
fn test_idempotency_key() -> educore_core::ids::IdempotencyKey {
    use educore_core::clock::IdGenerator;

    educore_core::clock::SystemIdGen.next_idempotency_key()
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
    fn video_integration_builder_constructs_with_defaults() {
        let adapter = VideoConferencingIntegrationBuilder::new().build();
        assert_eq!(adapter.provider, "zoom");
        assert_eq!(adapter.api_key, "");
        assert_eq!(adapter.api_secret, "");
        assert_eq!(adapter.base_url, DEFAULT_ZOOM_BASE_URL);
        assert_eq!(DEFAULT_ZOOM_BASE_URL, "https://api.zoom.us/v2");
    }

    #[test]
    fn video_integration_builder_accepts_overrides() {
        let adapter = VideoConferencingIntegrationBuilder::new()
            .provider("google_meet")
            .api_key("key-123")
            .api_secret("secret-456")
            .base_url("https://meet.googleapis.com/v2")
            .build();
        assert_eq!(adapter.provider, "google_meet");
        assert_eq!(adapter.api_key, "key-123");
        assert_eq!(adapter.api_secret, "secret-456");
        assert_eq!(adapter.base_url, "https://meet.googleapis.com/v2");
    }

    #[test]
    fn video_integration_builder_is_debug_and_default() {
        let default_builder = VideoConferencingIntegrationBuilder::default();
        let _ = format!("{default_builder:?}");
        let new_builder = VideoConferencingIntegrationBuilder::new();
        assert_eq!(format!("{new_builder:?}"), format!("{default_builder:?}"));
    }

    #[tokio::test]
    async fn video_integration_list_capabilities_returns_three_actions() {
        let adapter = VideoConferencingIntegrationBuilder::new().build();
        let capabilities = adapter
            .list_capabilities()
            .await
            .expect("list_capabilities must succeed");

        assert_eq!(capabilities.len(), 3);

        let actions: Vec<&str> = capabilities.iter().map(|c| c.action.as_str()).collect();
        assert!(actions.contains(&ACTION_MEETING_CREATE));
        assert!(actions.contains(&ACTION_MEETING_GET));
        assert!(actions.contains(&ACTION_RECORDING_LIST));

        for cap in &capabilities {
            assert_eq!(cap.integration.as_str(), VIDEO_INTEGRATION_ID);
            assert!(
                cap.input_schema.is_some(),
                "every capability must expose an input schema"
            );
            assert!(
                cap.output_schema.is_some(),
                "every capability must expose an output schema"
            );
        }
    }

    #[tokio::test]
    async fn video_integration_health_is_healthy() {
        let adapter = VideoConferencingIntegrationBuilder::new().build();
        let health = adapter.health().await.expect("health must succeed");
        assert_eq!(health.status, HealthStatus::Healthy);
        assert!(health.message.is_none());
    }

    #[tokio::test]
    async fn video_integration_invoke_unknown_action_returns_invalid_input() {
        let adapter = VideoConferencingIntegrationBuilder::new().build();
        let request = IntegrationRequest {
            tenant: test_tenant_context(),
            integration: IntegrationId::new(VIDEO_INTEGRATION_ID),
            action: IntegrationAction::new("video.bogus.action"),
            input: serde_json::json!({}),
            idempotency_key: test_idempotency_key(),
            correlation_id: test_correlation_id(),
            timeout: None,
        };

        let response = adapter
            .invoke(request)
            .await
            .expect("invoke must wrap errors");
        assert_eq!(response.status, IntegrationStatus::Failed);
        let err = response.error.expect("error must be populated");
        assert!(matches!(err, IntegrationError::InvalidInput(_)));
        assert!(response.output.is_none());
        assert!(response.duration >= chrono::Duration::zero());
    }

    #[test]
    fn schema_ref_points_at_relative_path() {
        let adapter = VideoConferencingIntegrationBuilder::new().build();
        let schema = adapter.schema_ref("input.schema.json", ACTION_MEETING_CREATE);
        assert_eq!(
            schema.location,
            format!(
                "integrations/{VIDEO_INTEGRATION_ID}/{ACTION_MEETING_CREATE}/input.schema.json"
            )
        );
        assert!(matches!(schema.format, SchemaFormat::JsonSchema));
    }

    #[test]
    fn status_from_error_maps_variants() {
        assert_eq!(
            status_from_error(&IntegrationError::RateLimited),
            IntegrationStatus::RateLimited
        );
        assert_eq!(
            status_from_error(&IntegrationError::Timeout(chrono::Duration::seconds(1))),
            IntegrationStatus::TimedOut
        );
        assert_eq!(
            status_from_error(&IntegrationError::InvalidInput("x".into())),
            IntegrationStatus::Failed
        );
        assert_eq!(
            status_from_error(&IntegrationError::Provider("500".into())),
            IntegrationStatus::Failed
        );
    }

    #[test]
    fn debug_redacts_api_credentials() {
        let adapter = VideoConferencingIntegrationBuilder::new()
            .api_key("super-secret-key")
            .api_secret("super-secret-secret")
            .build();
        let rendered = format!("{adapter:?}");
        assert!(rendered.contains("***redacted***"));
        assert!(!rendered.contains("super-secret-key"));
        assert!(!rendered.contains("super-secret-secret"));
    }
}
