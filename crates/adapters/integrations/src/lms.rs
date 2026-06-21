//! # LMS (Learning Management System) integration
//!
//! Reference implementation of the
//! [`IntegrationGateway`](crate::port::IntegrationGateway) trait for
//! LMS providers (Google Classroom, Microsoft Teams for Education,
//! Moodle).
//!
//! Per `docs/ports/integrations.md` § "LMS Sync":
//!
//! - Creates an LMS course when an `AcademicYear` and `Class` are
//!   configured.
//! - Syncs the roster when `StudentAdmitted`,
//!   `StudentAssignedToSection`, or `StudentWithdrawn` events fire.
//! - Pulls assignment submissions from the LMS and emits
//!   `OnlineExamSubmitted` events with a `Source::Lms` tag.
//!
//! ## Authentication
//!
//! OAuth bearer token supplied via `api_key`. The same token is
//! forwarded in `Authorization: Bearer <api_key>` for every
//! outbound call. `provider` is propagated into the response
//! metadata so consumers can route per-provider analytics.
//!
//! ## Actions
//!
//! - [`ACTION_COURSE_CREATE`] (`"lms.course.create"`) — create an
//!   LMS course for a freshly-configured `Class`.
//! - [`ACTION_ROSTER_SYNC`] (`"lms.roster.sync"`) — add or remove
//!   students from a course in response to admission / withdrawal /
//!   section-assignment events.
//! - [`ACTION_SUBMISSIONS_PULL`] (`"lms.submissions.pull"`) — list
//!   assignment submissions so the engine can emit
//!   `OnlineExamSubmitted`.
//!
//! ## Construction
//!
//! Use [`LmsIntegrationBuilder`]:
//!
//! ```ignore
//! let adapter = LmsIntegrationBuilder::new()
//!     .provider("google_classroom")
//!     .api_key(env::var("GOOGLE_CLASSROOM_TOKEN")?)
//!     .base_url("https://classroom.googleapis.com/v1")
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

use educore_rbac::value_objects::Capability;

use crate::errors::{IntegrationError, Result};
use crate::port::{
    HealthStatus, IntegrationAction, IntegrationCapability, IntegrationGateway, IntegrationHealth,
    IntegrationId, IntegrationRequest, IntegrationResponse, IntegrationStatus, SchemaFormat,
    SchemaRef,
};

// =============================================================================
// Defaults
// =============================================================================

/// Default LMS REST API base URL. Targets Google Classroom at
/// `https://classroom.googleapis.com/v1`. Used by
/// [`LmsIntegrationBuilder::build`] when the caller does not
/// override `base_url`.
pub const DEFAULT_LMS_BASE_URL: &str = "https://classroom.googleapis.com/v1";

/// The registered [`IntegrationId`] for this adapter.
pub const LMS_INTEGRATION_ID: &str = "lms";

/// Action name: create an LMS course for a configured `Class`.
pub const ACTION_COURSE_CREATE: &str = "lms.course.create";

/// Action name: sync the roster of an LMS course in response to
/// admission / withdrawal / section-assignment events.
pub const ACTION_ROSTER_SYNC: &str = "lms.roster.sync";

/// Action name: pull assignment submissions from the LMS so the
/// engine can emit `OnlineExamSubmitted` with a `Source::Lms` tag.
pub const ACTION_SUBMISSIONS_PULL: &str = "lms.submissions.pull";

// =============================================================================
// Builder
// =============================================================================

/// Builder for [`LmsIntegration`].
///
/// Accumulates configuration across repeated `.provider(..)`,
/// `.api_key(..)`, `.base_url(..)` calls and produces a
/// fully-configured adapter when [`build`](Self::build) is invoked.
/// Every field has a default: `provider` defaults to
/// `"google_classroom"`, `base_url` defaults to the Google Classroom
/// REST root, and `api_key` defaults to an empty string.
#[derive(Debug, Default, Clone)]
pub struct LmsIntegrationBuilder {
    provider: Option<String>,
    api_key: Option<String>,
    base_url: Option<String>,
}

impl LmsIntegrationBuilder {
    /// Creates a new builder with no configuration set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the LMS provider key (e.g. `"google_classroom"`,
    /// `"moodle"`, `"microsoft_teams_education"`).
    #[must_use]
    pub fn provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    /// Sets the API key (OAuth bearer token for the configured
    /// provider).
    #[must_use]
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Sets the provider base URL. Defaults to the Google Classroom
    /// REST root when the caller does not override it.
    #[must_use]
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Consumes the builder and produces a configured
    /// [`LmsIntegration`].
    #[must_use]
    pub fn build(self) -> LmsIntegration {
        LmsIntegration {
            http: Client::new(),
            provider: self
                .provider
                .unwrap_or_else(|| "google_classroom".to_owned()),
            api_key: self.api_key.unwrap_or_default(),
            base_url: self
                .base_url
                .unwrap_or_else(|| DEFAULT_LMS_BASE_URL.to_owned()),
        }
    }
}

// =============================================================================
// Integration
// =============================================================================

/// Reference integration adapter for LMS providers (Google
/// Classroom, Microsoft Teams for Education, Moodle).
///
/// Construct via [`LmsIntegrationBuilder`]. The adapter is `Send +
/// Sync` so it can be held behind an `Arc<dyn IntegrationGateway>`.
#[derive(Clone)]
pub struct LmsIntegration {
    http: Client,
    provider: String,
    api_key: String,
    base_url: String,
}

impl fmt::Debug for LmsIntegration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LmsIntegration")
            .field("provider", &self.provider)
            .field("base_url", &self.base_url)
            .field("api_key", &"***redacted***")
            .finish()
    }
}

#[async_trait]
impl IntegrationGateway for LmsIntegration {
    async fn invoke(&self, request: IntegrationRequest) -> Result<IntegrationResponse> {
        let started = Instant::now();
        let action = request.action.as_str();
        let result = match action {
            ACTION_COURSE_CREATE => self.create_course(&request).await,
            ACTION_ROSTER_SYNC => self.sync_roster(&request).await,
            ACTION_SUBMISSIONS_PULL => self.pull_submissions(&request).await,
            other => Err(IntegrationError::InvalidInput(format!(
                "unknown lms action: {other}"
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
            self.capability_course_create(),
            self.capability_roster_sync(),
            self.capability_submissions_pull(),
        ])
    }

    async fn health(&self) -> Result<IntegrationHealth> {
        Ok(IntegrationHealth {
            status: HealthStatus::Healthy,
            last_checked_at: educore_core::value_objects::Timestamp::now(),
            message: None,
        })
    }
}

// =============================================================================
// LMS API helpers (impl on the integration)
// =============================================================================

impl LmsIntegration {
    /// Returns the `Authorization: Bearer <api_key>` header value.
    fn auth_header(&self) -> String {
        format!("Bearer {}", self.api_key)
    }

    /// Creates an LMS course. The `input` payload MUST contain a
    /// `"name"` string field. Optional fields: `"section"`,
    /// `"description"`, `"owner_id"`.
    async fn create_course(&self, request: &IntegrationRequest) -> Result<JsonValue> {
        let name = request
            .input
            .get("name")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| {
                IntegrationError::InvalidInput(
                    "lms.course.create requires `name` (string)".to_owned(),
                )
            })?;
        let url = format!("{}/courses", self.base_url);
        let body = serde_json::json!({
            "name": name,
            "section": request.input.get("section").cloned().unwrap_or(JsonValue::Null),
            "description": request
                .input
                .get("description")
                .cloned()
                .unwrap_or(JsonValue::Null),
            "ownerId": request.input.get("owner_id").cloned().unwrap_or(JsonValue::Null),
        });
        let response = self
            .http
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .header("X-Correlation-Id", request.correlation_id.to_string())
            .header("Idempotency-Key", request.idempotency_key.to_string())
            .json(&body)
            .send()
            .await
            .map_err(infrastructure)?;
        parse_response(response).await
    }

    /// Syncs the roster of an LMS course. The `input` payload MUST
    /// contain a `"course_id"` string field and a `"students"`
    /// array of `{ "user_id": string, "action": "add" | "remove" }`
    /// entries. The adapter fans out one HTTP call per entry so
    /// individual failures don't abort the whole sync.
    async fn sync_roster(&self, request: &IntegrationRequest) -> Result<JsonValue> {
        let course_id = request
            .input
            .get("course_id")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| {
                IntegrationError::InvalidInput(
                    "lms.roster.sync requires `course_id` (string)".to_owned(),
                )
            })?;
        let students = request
            .input
            .get("students")
            .and_then(JsonValue::as_array)
            .ok_or_else(|| {
                IntegrationError::InvalidInput(
                    "lms.roster.sync requires `students` (array)".to_owned(),
                )
            })?;

        let mut synced = Vec::with_capacity(students.len());
        let mut errors = Vec::new();
        for entry in students {
            let user_id = entry.get("user_id").and_then(JsonValue::as_str);
            let action = entry.get("action").and_then(JsonValue::as_str);
            let (Some(user_id), Some(action)) = (user_id, action) else {
                errors.push(serde_json::json!({
                    "entry": entry,
                    "error": "missing user_id or action",
                }));
                continue;
            };

            let method = match action {
                "add" => reqwest::Method::POST,
                "remove" => reqwest::Method::DELETE,
                other => {
                    errors.push(serde_json::json!({
                        "user_id": user_id,
                        "error": format!("unknown roster action: {other}"),
                    }));
                    continue;
                }
            };

            let url = format!("{}/courses/{course_id}/students/{user_id}", self.base_url);
            let result = self
                .http
                .request(method, &url)
                .header("Authorization", self.auth_header())
                .header("X-Correlation-Id", request.correlation_id.to_string())
                .header("Idempotency-Key", request.idempotency_key.to_string())
                .send()
                .await;

            match result {
                Ok(resp) if resp.status().is_success() => {
                    synced.push(serde_json::json!({
                        "user_id": user_id,
                        "action": action,
                        "status": "ok",
                    }));
                }
                Ok(resp) => {
                    let status = resp.status();
                    errors.push(serde_json::json!({
                        "user_id": user_id,
                        "action": action,
                        "error": format!("http {}", status.as_u16()),
                    }));
                }
                Err(err) => {
                    errors.push(serde_json::json!({
                        "user_id": user_id,
                        "action": action,
                        "error": err.to_string(),
                    }));
                }
            }
        }

        Ok(serde_json::json!({
            "course_id": course_id,
            "synced": synced,
            "errors": errors,
        }))
    }

    /// Lists assignment submissions for a course work item. The
    /// `input` payload MUST contain `"course_id"` and
    /// `"coursework_id"` string fields. Optional: `"page_size"`
    /// (forwarded as the provider's pagination parameter).
    async fn pull_submissions(&self, request: &IntegrationRequest) -> Result<JsonValue> {
        let course_id = request
            .input
            .get("course_id")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| {
                IntegrationError::InvalidInput(
                    "lms.submissions.pull requires `course_id` (string)".to_owned(),
                )
            })?;
        let coursework_id = request
            .input
            .get("coursework_id")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| {
                IntegrationError::InvalidInput(
                    "lms.submissions.pull requires `coursework_id` (string)".to_owned(),
                )
            })?;

        let url = format!(
            "{}/courses/{course_id}/courseWork/{coursework_id}/studentSubmissions",
            self.base_url
        );
        let page_size = request
            .input
            .get("page_size")
            .and_then(JsonValue::as_i64)
            .unwrap_or(50);
        let response = self
            .http
            .get(&url)
            .header("Authorization", self.auth_header())
            .header("X-Correlation-Id", request.correlation_id.to_string())
            .header("Idempotency-Key", request.idempotency_key.to_string())
            .query(&[("pageSize", page_size.to_string())])
            .send()
            .await
            .map_err(infrastructure)?;
        let body = parse_response(response).await?;
        Ok(serde_json::json!({
            "course_id": course_id,
            "coursework_id": coursework_id,
            "submissions": body,
        }))
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
        metadata.insert("integration".to_owned(), LMS_INTEGRATION_ID.to_owned());
        metadata
    }

    /// Builds the [`IntegrationCapability`] for `lms.course.create`.
    fn capability_course_create(&self) -> IntegrationCapability {
        IntegrationCapability {
            integration: IntegrationId::new(LMS_INTEGRATION_ID),
            action: IntegrationAction::new(ACTION_COURSE_CREATE),
            description: "Create an LMS course for a configured AcademicYear/Class pair."
                .to_owned(),
            input_schema: Some(self.schema_ref("input.schema.json", ACTION_COURSE_CREATE)),
            output_schema: Some(self.schema_ref("output.schema.json", ACTION_COURSE_CREATE)),
            required_capabilities: vec![Capability::LmsRosterSync],
        }
    }

    /// Builds the [`IntegrationCapability`] for `lms.roster.sync`.
    fn capability_roster_sync(&self) -> IntegrationCapability {
        IntegrationCapability {
            integration: IntegrationId::new(LMS_INTEGRATION_ID),
            action: IntegrationAction::new(ACTION_ROSTER_SYNC),
            description: "Sync the LMS roster in response to StudentAdmitted / \
                          StudentAssignedToSection / StudentWithdrawn events."
                .to_owned(),
            input_schema: Some(self.schema_ref("input.schema.json", ACTION_ROSTER_SYNC)),
            output_schema: Some(self.schema_ref("output.schema.json", ACTION_ROSTER_SYNC)),
            required_capabilities: vec![Capability::LmsRosterSync],
        }
    }

    /// Builds the [`IntegrationCapability`] for `lms.submissions.pull`.
    fn capability_submissions_pull(&self) -> IntegrationCapability {
        IntegrationCapability {
            integration: IntegrationId::new(LMS_INTEGRATION_ID),
            action: IntegrationAction::new(ACTION_SUBMISSIONS_PULL),
            description: "Pull assignment submissions from the LMS so the engine can emit \
                          OnlineExamSubmitted events with a Source::Lms tag."
                .to_owned(),
            input_schema: Some(self.schema_ref("input.schema.json", ACTION_SUBMISSIONS_PULL)),
            output_schema: Some(self.schema_ref("output.schema.json", ACTION_SUBMISSIONS_PULL)),
            required_capabilities: vec![Capability::LmsRosterSync],
        }
    }

    /// Constructs a [`SchemaRef`] pointing at a relative path under
    /// the engine's asset store.
    fn schema_ref(&self, file: &str, action: &str) -> SchemaRef {
        SchemaRef {
            location: format!("integrations/{LMS_INTEGRATION_ID}/{action}/{file}"),
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
    fn lms_integration_builder_constructs_with_defaults() {
        let adapter = LmsIntegrationBuilder::new().build();
        assert_eq!(adapter.provider, "google_classroom");
        assert_eq!(adapter.api_key, "");
        assert_eq!(adapter.base_url, DEFAULT_LMS_BASE_URL);
        assert_eq!(DEFAULT_LMS_BASE_URL, "https://classroom.googleapis.com/v1");
    }

    #[test]
    fn lms_integration_builder_accepts_overrides() {
        let adapter = LmsIntegrationBuilder::new()
            .provider("moodle")
            .api_key("token-abc")
            .base_url("https://moodle.example.com/webservice/rest/server.php")
            .build();
        assert_eq!(adapter.provider, "moodle");
        assert_eq!(adapter.api_key, "token-abc");
        assert_eq!(
            adapter.base_url,
            "https://moodle.example.com/webservice/rest/server.php"
        );
    }

    #[test]
    fn lms_integration_builder_is_debug_and_default() {
        let default_builder = LmsIntegrationBuilder::default();
        let _ = format!("{default_builder:?}");
        let new_builder = LmsIntegrationBuilder::new();
        assert_eq!(format!("{new_builder:?}"), format!("{default_builder:?}"));
    }

    #[tokio::test]
    async fn lms_integration_list_capabilities_returns_three_actions() {
        let adapter = LmsIntegrationBuilder::new().build();
        let capabilities = adapter
            .list_capabilities()
            .await
            .expect("list_capabilities must succeed");

        assert_eq!(capabilities.len(), 3);

        let actions: Vec<&str> = capabilities.iter().map(|c| c.action.as_str()).collect();
        assert!(actions.contains(&ACTION_COURSE_CREATE));
        assert!(actions.contains(&ACTION_ROSTER_SYNC));
        assert!(actions.contains(&ACTION_SUBMISSIONS_PULL));

        for cap in &capabilities {
            assert_eq!(cap.integration.as_str(), LMS_INTEGRATION_ID);
            assert_eq!(
                cap.required_capabilities,
                vec![Capability::LmsRosterSync],
                "every LMS capability must require LmsRosterSync"
            );
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
    async fn lms_integration_health_is_healthy() {
        let adapter = LmsIntegrationBuilder::new().build();
        let health = adapter.health().await.expect("health must succeed");
        assert_eq!(health.status, HealthStatus::Healthy);
        assert!(health.message.is_none());
    }

    #[tokio::test]
    async fn lms_integration_invoke_unknown_action_returns_invalid_input() {
        let adapter = LmsIntegrationBuilder::new().build();
        let request = IntegrationRequest {
            tenant: test_tenant_context(),
            integration: IntegrationId::new(LMS_INTEGRATION_ID),
            action: IntegrationAction::new("lms.bogus.action"),
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
        let adapter = LmsIntegrationBuilder::new().build();
        let schema = adapter.schema_ref("input.schema.json", ACTION_COURSE_CREATE);
        assert_eq!(
            schema.location,
            format!("integrations/{LMS_INTEGRATION_ID}/{ACTION_COURSE_CREATE}/input.schema.json")
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
        let adapter = LmsIntegrationBuilder::new()
            .api_key("super-secret-token")
            .build();
        let rendered = format!("{adapter:?}");
        assert!(rendered.contains("***redacted***"));
        assert!(!rendered.contains("super-secret-token"));
    }
}
