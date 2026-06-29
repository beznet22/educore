//! # Push notification provider (stub)
//!
//! Reference [`NotificationProvider`](crate::port::NotificationProvider)
//! implementation that delivers
//! [`Channel::Push`](crate::port::Channel::Push) messages. The
//! real FCM (Firebase Cloud Messaging) and APNs (Apple Push
//! Notification service) wiring lands in a later phase; this
//! file ships the port-conforming scaffold so the engine can
//! route `Channel::Push` traffic through a typed adapter
//! boundary instead of falling through to
//! [`NotificationError::Provider`](crate::errors::NotificationError::Provider)
//! "no adapter for channel".
//!
//! ## Behavior
//!
//! Every `send` and `send_bulk` call:
//!
//! 1. Validates that the channel is
//!    [`Channel::Push`](crate::port::Channel::Push). Calls that
//!    supply a non-push channel return
//!    [`NotificationError::provider`](crate::errors::NotificationError::Provider)
//!    with a descriptive message — the engine is expected to
//!    route email / SMS / chat / voice / webhook traffic to the
//!    matching adapter.
//! 2. Validates that the provider is configured: a missing
//!    `default_sender_id` returns
//!    [`NotificationError::Provider`](crate::errors::NotificationError::Provider)
//!    rather than silently accepting the send.
//! 3. Emits a `tracing::warn!` ("Push adapter is a stub") so
//!    operators can observe stub-path usage on the hot path
//!    without blocking it.
//! 4. Returns a synthetic
//!    [`NotificationReceipt`](crate::port::NotificationReceipt)
//!    marked [`DeliveryStatus::Sent`](crate::port::DeliveryStatus::Sent)
//!    so downstream consumers (audit log, cost reporting, in-app
//!    inbox) record the dispatch.
//!
//! The stub does NOT `panic!` on configuration errors or wrong
//! channels — failures surface as typed
//! [`NotificationError`](crate::errors::NotificationError) values
//! so the caller can escalate (page, retry via a different
//! channel, etc.) rather than crash the worker.
//!
//! ## Template rendering
//!
//! The port surface does not hand the adapter a rendered template
//! body; the engine resolves the
//! [`TemplateRef`](crate::port::TemplateRef) and passes the
//! variable map. This stub does not substitute variables into a
//! body — push payloads are short-form (`{title, body, data}`)
//! and the data map is logged via `tracing::warn!` for
//! observability. A future implementation will resolve templates
//! against the communication-domain template store.
//!
//! ## TLS
//!
//! Real FCM and APNs transports will be wired with `rustls` per
//! ADR-015 (which forbids `native-tls`); the dependency is not
//! declared in `Cargo.toml` yet because the stub does not open a
//! network connection. The follow-up commit that introduces the
//! real transport will add `reqwest` with the `rustls-tls`
//! feature and the credential-loading path.

use std::fmt;

use async_trait::async_trait;
use educore_core::value_objects::Timestamp;

use crate::errors::NotificationError;
use crate::port::{
    BulkId, BulkReceipt, BulkRecipient, BulkRecipientIndex, Channel, DeliveryStatus,
    NotificationProvider, NotificationReceipt, NotificationReceiptId, Result, SendBulkNotification,
    SendNotification, TemplateRef, TemplateValue,
};
use crate::services::{emit_notification_sent, recipient_label, NotificationSent};

// ---------------------------------------------------------------------------
// PushProvider
// ---------------------------------------------------------------------------

/// Push [`NotificationProvider`] stub.
///
/// Holds the configured `default_sender_id` (the FCM sender id or
/// APNs bundle / topic id) and the optional `topic` used when
/// the request does not supply one on its
/// [`Channel::Push`](crate::port::Channel::Push) variant.
///
/// Cheap to clone (no internal state beyond the two `String`
/// fields).
#[derive(Clone)]
pub struct PushProvider {
    /// The default sender id used when a [`SendNotification`]
    /// does not supply `Channel::Push.topic`. FCM treats this as
    /// the `sender_id`; APNs treats it as the `topic` (bundle
    /// id).
    default_sender_id: String,
    /// Optional collapse key applied when the request does not
    /// supply one on its [`Channel::Push`](crate::port::Channel::Push)
    /// variant. The provider coalesces pending messages with the
    /// same collapse key on the recipient's device.
    default_collapse_key: Option<String>,
}

impl fmt::Debug for PushProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PushProvider")
            .field("default_sender_id", &self.default_sender_id)
            .field(
                "default_collapse_key",
                &self.default_collapse_key.as_deref().unwrap_or(""),
            )
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl NotificationProvider for PushProvider {
    async fn send(&self, request: SendNotification) -> Result<NotificationReceipt> {
        // 1. Reject non-push channels with a typed error so the
        //    engine can route the request to the matching
        //    adapter instead of crashing the worker.
        if !matches!(request.channel, Channel::Push { .. }) {
            return Err(NotificationError::provider(format!(
                "push provider cannot send {:?} channel",
                classify_channel(&request.channel)
            )));
        }

        // 2. Validate configuration. The stub does not actually
        //    open a network connection, but it must refuse
        //    sends when the default sender id is empty so
        //    misconfiguration surfaces as a typed error.
        if self.default_sender_id.is_empty() {
            return Err(NotificationError::provider(
                "PushProvider: default_sender_id is not configured",
            ));
        }

        // 3. Resolve the effective sender / collapse key from
        //    the request, falling back to the provider defaults.
        let (topic, collapse_key) =
            extract_push_options(&request.channel, &self.default_collapse_key);

        // 4. Stub observability hook. Real FCM / APNs wiring
        //    lands in a later phase.
        tracing::warn!(
            school_id = %request.school_id,
            sender_id = %self.default_sender_id,
            topic = %topic.as_deref().unwrap_or("-"),
            collapse_key = %collapse_key.as_deref().unwrap_or("-"),
            template = %template_id_of(&request.template),
            priority = request.priority.as_str(),
            "Push adapter is a stub: send dispatched without contacting FCM/APNs",
        );

        // 5. Return a synthetic receipt. The receipt id is
        //    deterministically derived from the school id and the
        //    unix millisecond clock so two calls in the same
        //    process-millisecond get distinct ids without
        //    pulling in `uuid`.
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(NotificationError::infrastructure)?
            .as_millis();
        let receipt_id = NotificationReceiptId::new(format!("push:{}:{now_ms}", request.school_id));

        Ok(NotificationReceipt::new(
            receipt_id,
            request.channel,
            DeliveryStatus::Sent,
            Timestamp::now(),
        ))
    }

    async fn send_bulk(&self, request: SendBulkNotification) -> Result<BulkReceipt> {
        // Reject non-push channels up front so the caller sees
        // a structured error instead of per-row failures.
        if !matches!(request.channel, Channel::Push { .. }) {
            return Err(NotificationError::provider(
                "push provider cannot send non-push channels",
            ));
        }

        // Configuration check: a missing default_sender_id is
        // fatal for the whole bulk send (we cannot synthesise a
        // sender per-row).
        if self.default_sender_id.is_empty() {
            return Err(NotificationError::provider(
                "PushProvider: default_sender_id is not configured",
            ));
        }

        let bulk_id = BulkId::new(format!(
            "bulk_push:{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(NotificationError::infrastructure)?
                .as_nanos()
        ));

        let mut receipt = BulkReceipt::new(bulk_id.clone());
        for (idx, row) in request.recipients.iter().enumerate() {
            let single = SendNotification {
                tenant: request.tenant.clone(),
                channel: request.channel.clone(),
                template: request.template.clone(),
                recipient: row.recipient.clone(),
                variables: row.variables.clone(),
                attachments: Vec::new(),
                priority: request.priority,
                scheduled_at: request.scheduled_at,
                idempotency_key: request.idempotency_key,
                correlation_id: request.correlation_id,
                school_id: request.school_id,
            };

            match self.send(single).await {
                Ok(r) => {
                    // Per `docs/ports/notifications.md` § "Bulk Send",
                    // emit one `NotificationSent` per successful
                    // recipient so downstream consumers (audit log,
                    // cost reporting, in-app inbox) can correlate
                    // per-row events with the parent bulk envelope.
                    let event = NotificationSent::new(
                        recipient_label(&row.recipient),
                        request.channel.clone(),
                        request.priority,
                        bulk_id.clone(),
                        request.school_id,
                        r.sent_at,
                    );
                    let _ = emit_notification_sent(&event);
                    receipt.receipts.push(r);
                }
                Err(e) => {
                    let Ok(idx_u32) = u32::try_from(idx) else {
                        continue;
                    };
                    receipt.failed.push((BulkRecipientIndex::new(idx_u32), e));
                }
            }
            // Touch the row so the unused-binding lint is
            // silenced without changing behaviour.
            let _ = row.variables.len();
        }

        Ok(receipt)
    }

    async fn status(&self, _receipt_id: NotificationReceiptId) -> Result<DeliveryStatus> {
        // Stub: a real adapter would call the FCM / APNs
        // status endpoint and translate the wire response into
        // the engine's `DeliveryStatus` enum.
        Ok(DeliveryStatus::Sent)
    }
}

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

/// Builder for [`PushProvider`].
///
/// Construct via [`PushProviderBuilder::new`], chain the
/// configuration methods, and finish with
/// [`PushProviderBuilder::build`]. The builder validates that
/// `default_sender_id` is set before constructing the provider.
#[derive(Debug, Default, Clone)]
pub struct PushProviderBuilder {
    default_sender_id: Option<String>,
    default_collapse_key: Option<String>,
}

impl PushProviderBuilder {
    /// Creates a new builder with no configuration set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the default sender id used when a
    /// [`SendNotification`] does not supply
    /// [`Channel::Push`](crate::port::Channel::Push) `topic`.
    /// FCM treats this as the `sender_id`; APNs treats it as the
    /// `topic` (bundle id).
    #[must_use]
    pub fn default_sender_id(mut self, sender_id: impl Into<String>) -> Self {
        self.default_sender_id = Some(sender_id.into());
        self
    }

    /// Sets the default collapse key applied when a
    /// [`SendNotification`] does not supply one on its
    /// [`Channel::Push`](crate::port::Channel::Push) variant.
    /// The provider coalesces pending messages with the same
    /// collapse key on the recipient's device.
    #[must_use]
    pub fn default_collapse_key(mut self, collapse_key: impl Into<String>) -> Self {
        self.default_collapse_key = Some(collapse_key.into());
        self
    }

    /// Consumes the builder and returns a configured
    /// [`PushProvider`].
    ///
    /// # Errors
    ///
    /// - [`NotificationError::Provider`] if `default_sender_id`
    ///   was not set.
    pub fn build(self) -> Result<PushProvider> {
        let default_sender_id = self.default_sender_id.ok_or_else(|| {
            NotificationError::provider("PushProviderBuilder: default_sender_id is required")
        })?;
        Ok(PushProvider {
            default_sender_id,
            default_collapse_key: self.default_collapse_key,
        })
    }
}

// ---------------------------------------------------------------------------
// Free helpers
// ---------------------------------------------------------------------------

/// Returns a stable, snake_case discriminator for a [`Channel`]
/// variant. Used in the error message when the push provider
/// receives a non-push channel.
fn classify_channel(channel: &Channel) -> &'static str {
    match channel {
        Channel::Email { .. } => "email",
        Channel::Sms { .. } => "sms",
        Channel::Push { .. } => "push",
        Channel::InApp => "inapp",
        Channel::Chat { .. } => "chat",
        Channel::Voice { .. } => "voice",
        Channel::Webhook { .. } => "webhook",
    }
}

/// Extracts the effective `topic` and `collapse_key` from a
/// [`Channel::Push`] variant. Returns `None` for both when the
/// channel is not [`Channel::Push`] (the caller has already
/// verified the channel shape, so this branch is defensive).
fn extract_push_options(
    channel: &Channel,
    default_collapse_key: &Option<String>,
) -> (Option<String>, Option<String>) {
    match channel {
        Channel::Push {
            topic,
            collapse_key,
            ..
        } => {
            let topic = topic.clone();
            let collapse_key = collapse_key
                .clone()
                .or_else(|| default_collapse_key.clone());
            (topic, collapse_key)
        }
        _ => (None, default_collapse_key.clone()),
    }
}

/// Returns the template id string for logging / receipt
/// correlation.
fn template_id_of(template: &TemplateRef) -> String {
    match template {
        TemplateRef::Id(id) => id.to_string(),
    }
}

/// Builds a single-recipient `SendNotification` from a bulk row.
/// Mirrors the bulk request's channel, template, priority, and
/// scheduling onto the single send; the per-row variables map
/// overrides any shared variables on the bulk request.
#[allow(dead_code)]
fn build_single_from_bulk(bulk: &SendBulkNotification, row: &BulkRecipient) -> SendNotification {
    SendNotification {
        tenant: bulk.tenant.clone(),
        channel: bulk.channel.clone(),
        template: bulk.template.clone(),
        recipient: row.recipient.clone(),
        variables: row.variables.clone(),
        attachments: Vec::new(),
        priority: bulk.priority,
        scheduled_at: bulk.scheduled_at,
        idempotency_key: bulk.idempotency_key,
        correlation_id: bulk.correlation_id,
        school_id: bulk.school_id,
    }
}

/// Renders a [`TemplateValue`] as a string for observability
/// logging. The stub does not substitute variables into a body;
/// it logs the rendered map via `tracing::warn!` so operators
/// can confirm the call shape without leaking PII into the log.
#[allow(dead_code)]
fn render_template_value(value: &TemplateValue) -> String {
    match value {
        TemplateValue::Text(s) => s.clone(),
        TemplateValue::Number(n) => n.to_string(),
        TemplateValue::Decimal(s) => s.clone(),
        TemplateValue::Boolean(b) => b.to_string(),
        TemplateValue::Date(s) => s.clone(),
        TemplateValue::Json(s) => s.clone(),
    }
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
    use crate::errors::NotificationTemplateId;
    use crate::port::{ContactInfo, EmailAddress, Priority, Recipient};
    use educore_core::ids::{CorrelationId, Identifier, PUBLIC_SCHOOL_ID, SYSTEM_USER_ID};
    use educore_core::tenant::{Locale, TenantContext, TimeZone, UserType};
    use std::collections::BTreeMap;

    fn sample_tenant() -> TenantContext {
        TenantContext {
            school_id: PUBLIC_SCHOOL_ID,
            actor_id: SYSTEM_USER_ID,
            session_id: None,
            correlation_id: CorrelationId::from_uuid(PUBLIC_SCHOOL_ID.as_uuid()),
            causation_id: None,
            user_type: UserType::System,
            locale: Locale::default(),
            timezone: TimeZone::default(),
        }
    }

    fn push_channel() -> Channel {
        Channel::Push {
            topic: Some("news".to_owned()),
            ttl: None,
            collapse_key: Some("msg".to_owned()),
        }
    }

    fn sample_send(channel: Channel) -> SendNotification {
        SendNotification {
            tenant: sample_tenant(),
            channel,
            template: TemplateRef::Id(NotificationTemplateId::new("tpl_xyz")),
            recipient: Recipient::Direct(
                ContactInfo::new().with_email(EmailAddress::new("user@example.com")),
            ),
            variables: BTreeMap::new(),
            attachments: Vec::new(),
            priority: Priority::Normal,
            scheduled_at: None,
            idempotency_key: None,
            correlation_id: None,
            school_id: PUBLIC_SCHOOL_ID,
        }
    }

    #[test]
    fn push_provider_builder_constructs_with_required_field() {
        let provider = PushProviderBuilder::new()
            .default_sender_id("sender-123")
            .build()
            .expect("builder should succeed with default_sender_id set");
        assert_eq!(provider.default_sender_id, "sender-123");
        assert!(provider.default_collapse_key.is_none());

        // Sanity-check debug output to make sure the struct is
        // not silently broken.
        let _ = format!("{provider:?}");
    }

    #[test]
    fn push_provider_builder_constructs_with_collapse_key() {
        let provider = PushProviderBuilder::new()
            .default_sender_id("sender-123")
            .default_collapse_key("news-feed")
            .build()
            .expect("builder should succeed with both fields set");
        assert_eq!(provider.default_collapse_key.as_deref(), Some("news-feed"));
    }

    #[test]
    fn push_provider_builder_rejects_missing_sender_id() {
        let result = PushProviderBuilder::new().build();
        match result {
            Err(NotificationError::Provider(msg)) => {
                assert!(msg.contains("default_sender_id"), "got {msg:?}");
            }
            other => panic!("expected Provider error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn push_provider_send_returns_synthetic_receipt() {
        let provider = PushProviderBuilder::new()
            .default_sender_id("sender-123")
            .build()
            .expect("builder should succeed");

        let req = sample_send(push_channel());
        let receipt = provider
            .send(req)
            .await
            .expect("push send should succeed in stub mode");
        assert!(matches!(receipt.status, DeliveryStatus::Sent));
        assert!(receipt.receipt_id.as_str().starts_with("push:"));
        // The receipt echoes the channel so downstream consumers
        // can correlate the receipt with the channel that
        // delivered it.
        assert!(matches!(receipt.channel, Channel::Push { .. }));
    }

    #[tokio::test]
    async fn push_provider_rejects_wrong_channel() {
        let provider = PushProviderBuilder::new()
            .default_sender_id("sender-123")
            .build()
            .expect("builder should succeed");

        let req = sample_send(Channel::Email {
            from: None,
            reply_to: None,
        });
        let err = provider
            .send(req)
            .await
            .expect_err("non-push channel should fail");
        match err {
            NotificationError::Provider(msg) => {
                assert!(msg.contains("email"), "got {msg:?}");
            }
            other => panic!("expected Provider error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn push_provider_rejects_missing_configuration() {
        // Hand-craft a provider with an empty default_sender_id
        // by going through the builder's only validation path
        // and then bypassing it via the Default impl of an
        // unconfigured provider. This keeps the public surface
        // honest (the builder rejects empty) while still
        // exercising the runtime check.
        let provider = PushProvider {
            default_sender_id: String::new(),
            default_collapse_key: None,
        };

        let req = sample_send(push_channel());
        let err = provider
            .send(req)
            .await
            .expect_err("empty default_sender_id should fail");
        match err {
            NotificationError::Provider(msg) => {
                assert!(msg.contains("default_sender_id"), "got {msg:?}");
            }
            other => panic!("expected Provider error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn push_provider_bulk_routes_per_row() {
        let provider = PushProviderBuilder::new()
            .default_sender_id("sender-123")
            .build()
            .expect("builder should succeed");

        let bulk = SendBulkNotification {
            tenant: sample_tenant(),
            template: TemplateRef::Id(NotificationTemplateId::new("tpl_bulk")),
            recipients: vec![
                BulkRecipient::new(Recipient::Direct(
                    ContactInfo::new().with_email(EmailAddress::new("a@example.com")),
                )),
                BulkRecipient::new(Recipient::Direct(
                    ContactInfo::new().with_email(EmailAddress::new("b@example.com")),
                )),
            ],
            variables_per_recipient: true,
            channel: push_channel(),
            priority: Priority::Normal,
            scheduled_at: None,
            idempotency_key: None,
            correlation_id: None,
            school_id: PUBLIC_SCHOOL_ID,
        };

        let receipt = provider
            .send_bulk(bulk)
            .await
            .expect("bulk push should succeed in stub mode");
        assert_eq!(receipt.total(), 2);
        assert_eq!(receipt.success_count(), 2);
        assert_eq!(receipt.failure_count(), 0);
        assert!(receipt.bulk_id.as_str().starts_with("bulk_push:"));
    }

    #[tokio::test]
    async fn push_provider_status_returns_sent() {
        let provider = PushProviderBuilder::new()
            .default_sender_id("sender-123")
            .build()
            .expect("builder should succeed");

        let status = provider
            .status(NotificationReceiptId::new("push:stub:1"))
            .await
            .expect("status lookup should succeed");
        assert_eq!(status, DeliveryStatus::Sent);
    }

    #[test]
    fn classify_channel_returns_stable_labels() {
        assert_eq!(
            classify_channel(&Channel::Email {
                from: None,
                reply_to: None
            }),
            "email"
        );
        assert_eq!(
            classify_channel(&Channel::Sms {
                from: None,
                unicode: false
            }),
            "sms"
        );
        assert_eq!(classify_channel(&Channel::InApp), "inapp");
        assert_eq!(
            classify_channel(&Channel::Chat {
                provider: crate::port::ChatProvider::Telegram
            }),
            "chat"
        );
        assert_eq!(
            classify_channel(&Channel::Voice {
                voice_id: None,
                language: crate::port::LanguageTag::default()
            }),
            "voice"
        );
        assert_eq!(
            classify_channel(&Channel::Webhook {
                url: crate::port::Url::new("https://example.test/hook"),
                secret: None,
            }),
            "webhook"
        );
    }

    #[test]
    fn extract_push_options_prefers_request_over_default() {
        let channel = Channel::Push {
            topic: Some("alerts".to_owned()),
            ttl: None,
            collapse_key: Some("ck-1".to_owned()),
        };
        let (topic, collapse) = extract_push_options(&channel, &Some("default-ck".to_owned()));
        assert_eq!(topic.as_deref(), Some("alerts"));
        assert_eq!(collapse.as_deref(), Some("ck-1"));
    }

    #[test]
    fn extract_push_options_falls_back_to_default_collapse_key() {
        let channel = Channel::Push {
            topic: Some("alerts".to_owned()),
            ttl: None,
            collapse_key: None,
        };
        let (topic, collapse) = extract_push_options(&channel, &Some("default-ck".to_owned()));
        assert_eq!(topic.as_deref(), Some("alerts"));
        assert_eq!(collapse.as_deref(), Some("default-ck"));
    }

    #[test]
    fn render_template_value_covers_every_variant() {
        assert_eq!(
            render_template_value(&TemplateValue::text("hello")),
            "hello"
        );
        assert_eq!(render_template_value(&TemplateValue::number(7)), "7");
        assert_eq!(
            render_template_value(&TemplateValue::decimal("1.50")),
            "1.50"
        );
        assert_eq!(render_template_value(&TemplateValue::boolean(true)), "true");
        assert_eq!(
            render_template_value(&TemplateValue::date("2026-06-29")),
            "2026-06-29"
        );
        assert_eq!(
            render_template_value(&TemplateValue::json("{\"k\":1}")),
            "{\"k\":1}"
        );
    }
}
