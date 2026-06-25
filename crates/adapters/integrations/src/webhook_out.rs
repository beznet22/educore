//! # Custom Webhook Out — reference implementation
//!
//! [`WebhookOutIntegration`] is the reference adapter for the
//! `IntegrationGateway` port's "Custom Webhook (Out)" pattern, per
//! `docs/ports/integrations.md` § "Custom Webhook (Out)".
//!
//! The engine can publish events to a configured webhook URL:
//!
//! ```ignore
//! let adapter = WebhookOutIntegrationBuilder::new()
//!     .target(WebhookTarget {
//!         url: "https://school.example.com/hooks/educore".into(),
//!         secret: "shared-secret".into(),
//!         event_filter: Some("InvoicePaid".into()),
//!     })
//!     .target(WebhookTarget {
//!         url: "https://audit.example.com/sink".into(),
//!         secret: "other-secret".into(),
//!         event_filter: None,
//!     })
//!     .build();
//! ```
//!
//! Each configured target is delivered the JSON-serialised
//! [`IntegrationRequest::input`](crate::port::IntegrationRequest::input)
//! payload via `POST <url>`, with the body signed using
//! HMAC-SHA256 and the signature transmitted in the
//! `X-Educore-Signature: sha256=<hex>` header. Receivers verify
//! by recomputing the HMAC over the raw body and constant-time
//! comparing it against the header value.
//!
//! # Implementation outline
//!
//! | Port method           | Behaviour                                       |
//! | --------------------- | ----------------------------------------------- |
//! | `invoke`              | Fan-out POST to every matching target.          |
//! | `list_capabilities`   | One static `IntegrationCapability` row.         |
//! | `health`              | Always `Healthy` (no provider-side probe).      |
//!
//! # Event filtering
//!
//! Each [`WebhookTarget`] may carry an `event_filter`. The filter
//! is matched against the dispatched request's
//! [`IntegrationAction`](crate::port::IntegrationAction) string:
//! `Some(filter)` means "deliver only when `request.action.as_str()
//! == filter`"; `None` means "deliver every action". Targets whose
//! filter does not match the action are skipped silently (no
//! error, no retry).
//!
//! # Deviations from `docs/ports/integrations.md`
//!
//! 1. **Filter shape.** The port spec sketches
//!    `EventFilter::EventType(name)` as the filter type. This
//!    reference impl stores the filter as `Option<String>` and
//!    matches by `IntegrationAction` string equality — the
//!    `event_type` concept collapses to "what action was
//!    requested" at the `IntegrationGateway` boundary. A
//!    production adapter that needs richer filtering (regex,
//!    multi-event, per-tenant) would extend `WebhookTarget` with
//!    those fields.
//! 2. **Retry policy.** The port spec suggests `RetryPolicy::Exponential`
//!    on the underlying `WebhookConfig`. This reference impl
//!    leaves retry orchestration to the engine's outbox
//!    dispatcher (`docs/schemas/event-schema.md` § 4) — the
//!    adapter itself performs exactly one POST per target and
//!    reports the outcome.
//! 3. **Auth.** No OAuth2 / bearer / mTLS support. The HMAC
//!    signature is the sole auth mechanism; the receiver is
//!    responsible for verifying it.
//!
//! # Security
//!
//! The integration's `Debug` impl redacts every target's
//! `secret` field. Webhook secrets are never written to logs,
//! metrics, or the audit trail.

use std::collections::BTreeMap;
use std::fmt;
use std::time::Instant;

use async_trait::async_trait;
use hmac::{Hmac, Mac};
use reqwest::{Client, StatusCode};
use serde_json::Value as JsonValue;
use sha2::Sha256;

use educore_core::value_objects::Timestamp;
use educore_rbac::value_objects::Capability;

use crate::errors::{IntegrationError, Result};
use crate::port::{
    HealthStatus, IntegrationAction, IntegrationCapability, IntegrationGateway, IntegrationHealth,
    IntegrationId, IntegrationRequest, IntegrationResponse, IntegrationStatus,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// HTTP request timeout applied to every outbound webhook call.
const HTTP_TIMEOUT_SECS: u64 = 30;

/// The HTTP header carrying the HMAC-SHA256 signature of the body.
pub const SIGNATURE_HEADER: &str = "X-Educore-Signature";

/// The HMAC-SHA256, typed alias used everywhere in this module.
type HmacSha256 = Hmac<Sha256>;

/// The single capability exposed by this adapter.
const WEBHOOK_OUT_CAPABILITY_DESCRIPTION: &str =
    "Publish an event to every configured webhook out-target. \
     The payload is the JSON-serialised request input; the \
     body is signed with HMAC-SHA256 and the signature is \
     transmitted in the X-Educore-Signature response header.";

// ---------------------------------------------------------------------------
// WebhookTarget
// ---------------------------------------------------------------------------

/// A single outbound webhook destination.
///
/// Each target owns its URL, its signing secret, and an optional
/// filter that limits which actions the target receives. A target
/// with `event_filter == None` receives every dispatched action.
#[derive(Clone, PartialEq, Eq)]
pub struct WebhookTarget {
    /// The absolute URL the webhook POST is delivered to. Must be
    /// an `http://` or `https://` URL parseable by
    /// `reqwest::Client`.
    pub url: String,

    /// The HMAC-SHA256 signing secret shared with the receiver.
    /// The receiver uses the same secret to recompute and verify
    /// the `X-Educore-Signature` header.
    pub secret: String,

    /// Optional action filter. When `Some(name)`, the target
    /// receives only requests whose
    /// [`IntegrationAction::as_str`](crate::port::IntegrationAction::as_str)
    /// equals `name`. When `None`, every action is delivered.
    pub event_filter: Option<String>,
}

impl fmt::Debug for WebhookTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebhookTarget")
            .field("url", &self.url)
            .field("secret", &"<redacted>")
            .field("event_filter", &self.event_filter)
            .finish()
    }
}

impl WebhookTarget {
    /// Returns `true` if this target should receive the given
    /// action. A target with no filter receives every action;
    /// otherwise the action string must equal the filter exactly.
    #[must_use]
    pub fn matches(&self, action: &IntegrationAction) -> bool {
        match &self.event_filter {
            None => true,
            Some(filter) => filter == action.as_str(),
        }
    }
}

// ---------------------------------------------------------------------------
// WebhookOutIntegration
// ---------------------------------------------------------------------------

/// A [`IntegrationGateway`] that fans out events to one or more
/// webhook endpoints, signing each POST with HMAC-SHA256.
///
/// `Clone` is implemented because [`reqwest::Client`] is internally
/// `Arc`-shared — cloning the integration is cheap and safe.
#[derive(Clone)]
pub struct WebhookOutIntegration {
    http: Client,
    targets: Vec<WebhookTarget>,
}

impl WebhookOutIntegration {
    /// Returns the configured targets, in insertion order.
    #[must_use]
    pub fn targets(&self) -> &[WebhookTarget] {
        &self.targets
    }

    /// Returns the number of configured targets.
    #[must_use]
    pub fn target_count(&self) -> usize {
        self.targets.len()
    }

    /// Computes the `X-Educore-Signature` value for the given
    /// payload and secret. Exposed publicly so callers can verify
    /// signatures in tests and in receiver-side adapters.
    ///
    /// Returns the hex-encoded digest prefixed with `sha256=`,
    /// matching the wire format documented in
    /// `docs/ports/integrations.md`. The `Result` wrapper exists
    /// to thread HMAC key errors through without `unwrap`; in
    /// practice HMAC-SHA256 accepts any key length, so the
    /// `Err` arm is unreachable for well-formed inputs.
    pub fn compute_signature(secret: &str, payload: &[u8]) -> Result<String> {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).map_err(|e| {
            IntegrationError::Infrastructure(format!("webhook HMAC key rejected: {e}").into())
        })?;
        mac.update(payload);
        let bytes = mac.finalize().into_bytes();
        Ok(format!("sha256={}", hex_encode(&bytes)))
    }

    /// Delivers a single payload to a single target. Internal —
    /// the public surface is [`IntegrationGateway::invoke`].
    async fn deliver(&self, target: &WebhookTarget, payload: &[u8]) -> Result<reqwest::Response> {
        let signature = Self::compute_signature(&target.secret, payload)?;

        self.http
            .post(&target.url)
            .header(SIGNATURE_HEADER, signature)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(payload.to_vec())
            .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT_SECS))
            .send()
            .await
            .map_err(|e| {
                IntegrationError::Infrastructure(Box::new(std::io::Error::other(format!(
                    "webhook POST to {} failed: {e}",
                    target.url
                ))))
            })
    }
}

impl fmt::Debug for WebhookOutIntegration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebhookOutIntegration")
            .field("target_count", &self.targets.len())
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl IntegrationGateway for WebhookOutIntegration {
    async fn invoke(&self, request: IntegrationRequest) -> Result<IntegrationResponse> {
        let started = Instant::now();

        let payload = serde_json::to_vec(&request.input).map_err(|e| {
            IntegrationError::InvalidInput(format!(
                "webhook payload must be JSON-serialisable: {e}"
            ))
        })?;

        let mut dispatched = 0_u32;
        let mut last_error: Option<IntegrationError> = None;
        let mut last_status: Option<StatusCode> = None;

        for target in &self.targets {
            if !target.matches(&request.action) {
                continue;
            }

            match self.deliver(target, &payload).await {
                Ok(response) => {
                    let status = response.status();
                    last_status = Some(status);
                    dispatched = dispatched.saturating_add(1);

                    if !status.is_success() {
                        last_error = Some(IntegrationError::Provider(format!(
                            "webhook target {} returned {}",
                            target.url, status
                        )));
                    }
                }
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }

        let duration = chrono::Duration::from_std(started.elapsed()).unwrap_or_default();

        if let Some(err) = last_error {
            let mut metadata = BTreeMap::new();
            metadata.insert("dispatched_targets".to_owned(), dispatched.to_string());
            if let Some(status) = last_status {
                metadata.insert("last_status".to_owned(), status.as_u16().to_string());
            }

            return Ok(IntegrationResponse {
                status: IntegrationStatus::Failed,
                output: None,
                error: Some(err),
                duration,
                cost: None,
                metadata,
            });
        }

        if dispatched == 0 {
            return Err(IntegrationError::InvalidInput(format!(
                "no webhook target matched action {}",
                request.action
            )));
        }

        let mut metadata = BTreeMap::new();
        metadata.insert("dispatched_targets".to_owned(), dispatched.to_string());

        Ok(IntegrationResponse {
            status: IntegrationStatus::Success,
            output: Some(JsonValue::Object({
                let mut map = serde_json::Map::new();
                map.insert("dispatched".into(), JsonValue::from(dispatched));
                map
            })),
            error: None,
            duration,
            cost: None,
            metadata,
        })
    }

    async fn list_capabilities(&self) -> Result<Vec<IntegrationCapability>> {
        Ok(vec![IntegrationCapability {
            integration: IntegrationId::new("webhook_out"),
            action: IntegrationAction::new("webhook.dispatch"),
            description: WEBHOOK_OUT_CAPABILITY_DESCRIPTION.to_owned(),
            input_schema: None,
            output_schema: None,
            required_capabilities: vec![Capability::WebhookOut],
        }])
    }

    async fn health(&self) -> Result<IntegrationHealth> {
        Ok(IntegrationHealth {
            status: HealthStatus::Healthy,
            last_checked_at: Timestamp::now(),
            message: None,
        })
    }
}

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

/// Builder for [`WebhookOutIntegration`].
///
/// Accumulates [`WebhookTarget`]s via repeated `.target(...)`
/// calls and assembles the final integration on `.build()`. The
/// builder does not validate the URL syntax — that's deferred to
/// the first `.invoke()` call so misconfiguration surfaces at
/// dispatch time, not at wiring time.
#[derive(Debug, Clone, Default)]
pub struct WebhookOutIntegrationBuilder {
    targets: Vec<WebhookTarget>,
}

impl WebhookOutIntegrationBuilder {
    /// Creates a new, empty builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a target to the fan-out list. May be called multiple
    /// times to configure multiple webhook endpoints.
    #[must_use]
    pub fn target(mut self, target: WebhookTarget) -> Self {
        self.targets.push(target);
        self
    }

    /// Adds multiple targets at once.
    #[must_use]
    pub fn targets(mut self, targets: impl IntoIterator<Item = WebhookTarget>) -> Self {
        self.targets.extend(targets);
        self
    }

    /// Assembles the final [`WebhookOutIntegration`].
    ///
    /// The `Result` wrapper exists to thread `reqwest::Client`
    /// construction failures through without `unwrap`; in
    /// practice a builder configured with a valid timeout always
    /// succeeds, so the `Err` arm is unreachable for
    /// well-formed configurations. `Result` is already
    /// `#[must_use]`, so no extra attribute is needed.
    pub fn build(self) -> Result<WebhookOutIntegration> {
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT_SECS))
            .build()
            .map_err(|e| {
                IntegrationError::Infrastructure(
                    format!("webhook reqwest client construction failed: {e}").into(),
                )
            })?;
        Ok(WebhookOutIntegration {
            http,
            targets: self.targets,
        })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Lower-case hex encoding without pulling in the `hex` crate.
///
/// `usize::from` is used instead of `as usize` to satisfy the
/// engine rule that forbids numeric `as` casts; `u8 → usize` is
/// a lossless widening that is always defined on every platform
/// the engine targets.
fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[usize::from(byte >> 4)] as char);
        out.push(HEX[usize::from(byte & 0x0f)] as char);
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;

    use serde_json::json;

    use educore_core::value_objects::Timestamp;

    use crate::port::IntegrationAction;

    #[test]
    fn test_webhook_out_builder_constructs_with_defaults() {
        let target = WebhookTarget {
            url: "https://example.com/hooks/educore".to_owned(),
            secret: "shared-secret".to_owned(),
            event_filter: None,
        };

        let integration = WebhookOutIntegrationBuilder::new()
            .target(target.clone())
            .build()
            .expect("builder with valid timeout succeeds");

        assert_eq!(integration.target_count(), 1);
        assert_eq!(integration.targets()[0], target);
        assert_eq!(
            integration.targets()[0].url,
            "https://example.com/hooks/educore"
        );
        assert_eq!(integration.targets()[0].event_filter, None);
    }

    #[test]
    fn test_webhook_out_builder_accumulates_multiple_targets() {
        let integration = WebhookOutIntegrationBuilder::new()
            .target(WebhookTarget {
                url: "https://a.example.com/hook".into(),
                secret: "s1".into(),
                event_filter: None,
            })
            .target(WebhookTarget {
                url: "https://b.example.com/hook".into(),
                secret: "s2".into(),
                event_filter: Some("InvoicePaid".into()),
            })
            .build()
            .expect("builder with valid timeout succeeds");

        assert_eq!(integration.target_count(), 2);
        assert_eq!(
            integration.targets()[1].event_filter.as_deref(),
            Some("InvoicePaid")
        );
    }

    #[test]
    fn test_webhook_signature_is_hmac_sha256() {
        let secret = "shared-secret";
        let payload = br#"{"event":"InvoicePaid","amount_minor":12500}"#;

        let sig = WebhookOutIntegration::compute_signature(secret, payload)
            .expect("HMAC-SHA256 accepts any key length");

        assert!(
            sig.starts_with("sha256="),
            "signature must carry the sha256= prefix, got {sig}"
        );

        let hex_part = &sig["sha256=".len()..];
        assert_eq!(hex_part.len(), 64, "SHA-256 hex digest must be 64 chars");

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("hmac key");
        mac.update(payload);
        let expected_bytes = mac.finalize().into_bytes();
        let expected_hex: String = expected_bytes
            .iter()
            .flat_map(|b| format!("{:02x}", b).into_bytes())
            .map(|c| c as char)
            .collect();
        assert_eq!(hex_part, expected_hex);
    }

    #[test]
    fn test_webhook_signature_changes_with_payload() {
        let secret = "k";
        let sig_a = WebhookOutIntegration::compute_signature(secret, b"alpha")
            .expect("HMAC-SHA256 accepts any key length");
        let sig_b = WebhookOutIntegration::compute_signature(secret, b"beta")
            .expect("HMAC-SHA256 accepts any key length");
        assert_ne!(sig_a, sig_b);
    }

    #[test]
    fn test_webhook_signature_changes_with_secret() {
        let payload = b"payload";
        let sig_a = WebhookOutIntegration::compute_signature("secret-a", payload)
            .expect("HMAC-SHA256 accepts any key length");
        let sig_b = WebhookOutIntegration::compute_signature("secret-b", payload)
            .expect("HMAC-SHA256 accepts any key length");
        assert_ne!(sig_a, sig_b);
    }

    #[test]
    fn webhook_target_matches_action() {
        let target_open = WebhookTarget {
            url: "https://a".into(),
            secret: "s".into(),
            event_filter: None,
        };
        let target_filtered = WebhookTarget {
            url: "https://b".into(),
            secret: "s".into(),
            event_filter: Some("InvoicePaid".into()),
        };

        let action = IntegrationAction::new("InvoicePaid");
        assert!(target_open.matches(&action));
        assert!(target_filtered.matches(&action));

        let other_action = IntegrationAction::new("StudentEnrolled");
        assert!(target_open.matches(&other_action));
        assert!(!target_filtered.matches(&other_action));
    }

    #[tokio::test]
    async fn list_capabilities_returns_single_webhook_dispatch_row() {
        let integration = WebhookOutIntegrationBuilder::new()
            .target(WebhookTarget {
                url: "https://example.com".into(),
                secret: "s".into(),
                event_filter: None,
            })
            .build()
            .expect("builder with valid timeout succeeds");

        let caps = integration.list_capabilities().await.expect("caps");
        assert_eq!(caps.len(), 1);
        assert_eq!(caps[0].integration.as_str(), "webhook_out");
        assert_eq!(caps[0].action.as_str(), "webhook.dispatch");
        assert_eq!(caps[0].required_capabilities, vec![Capability::WebhookOut]);
    }

    #[tokio::test]
    async fn health_reports_healthy_with_current_timestamp() {
        let integration = WebhookOutIntegrationBuilder::new()
            .build()
            .expect("builder with valid timeout succeeds");
        let health = integration.health().await.expect("health");
        assert_eq!(health.status, HealthStatus::Healthy);
        assert!(health.message.is_none());
        assert!(health.last_checked_at >= Timestamp::epoch());
    }

    #[test]
    fn debug_redacts_secrets() {
        let target = WebhookTarget {
            url: "https://example.com".into(),
            secret: "super-secret-value".into(),
            event_filter: None,
        };
        let dbg = format!("{target:?}");
        assert!(
            !dbg.contains("super-secret-value"),
            "debug must redact secrets: {dbg}"
        );
        assert!(
            dbg.contains("<redacted>"),
            "debug must mark redacted: {dbg}"
        );
    }

    #[test]
    fn signature_header_constant_matches_spec() {
        assert_eq!(SIGNATURE_HEADER, "X-Educore-Signature");
    }

    #[test]
    fn hex_encode_round_trip_known_vector() {
        let bytes = [0x00_u8, 0xff, 0x10, 0xab];
        assert_eq!(hex_encode(&bytes), "00ff10ab");
    }

    #[test]
    fn json_serialized_payload_is_byte_stable() {
        // Smoke test: the payload posted to each webhook is the
        // JSON-serialised form of `IntegrationRequest::input`, so
        // re-serialising the same value produces the same bytes
        // (and therefore the same signature).
        let a = json!({"event": "InvoicePaid", "amount_minor": 12500});
        let b = json!({"event": "InvoicePaid", "amount_minor": 12500});
        assert_eq!(
            serde_json::to_vec(&a).unwrap(),
            serde_json::to_vec(&b).unwrap()
        );
    }
}
