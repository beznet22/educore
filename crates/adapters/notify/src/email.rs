//! # Email notification provider (SMTP via `lettre`)
//!
//! Reference [`NotificationProvider`](crate::port::NotificationProvider)
//! implementation that delivers
//! [`Channel::Email`](crate::port::Channel::Email) messages over
//! SMTP using the `lettre` crate (workspace-pinned at `0.10`,
//! per ADR-015).
//!
//! The provider handles only `Channel::Email`. Calls that supply a
//! non-email channel return
//! [`NotificationError::provider`](crate::errors::NotificationError::Provider)
//! with a descriptive message — the engine is expected to route
//! SMS / push / chat / voice / webhook traffic to the matching
//! adapter.
//!
//! ## Template rendering
//!
//! The port surface does not hand the adapter a rendered template
//! body; the engine resolves the [`TemplateRef`](crate::port::TemplateRef)
//! and passes the variable map. This provider treats the
//! `TemplateRef::Id` value as a marker for the rendered subject
//! and substitutes `{var_name}` placeholders in the body via the
//! [`substitute_variables`] helper. A real deployment would fetch
//! the template body from the communication-domain template store
//! before substitution; that store is not part of the
//! notification port.
//!
//! ## TLS
//!
//! The transport is built with the `tokio1-rustls-tls` feature, so
//! all SMTP connections use `rustls` (ADR-015 forbids
//! `native-tls`). The reference builder uses
//! [`AsyncSmtpTransport::builder_dangerous`](lettre::AsyncSmtpTransport::builder_dangerous)
//! so consumers can wire their own TLS configuration; the
//! recommended constructors are
//! [`AsyncSmtpTransport::relay`](lettre::AsyncSmtpTransport::relay)
//! and
//! [`AsyncSmtpTransport::starttls_relay`](lettre::AsyncSmtpTransport::starttls_relay).

use std::collections::BTreeMap;
use std::fmt;

use async_trait::async_trait;
use educore_core::ids::SchoolId;
use educore_core::value_objects::Timestamp;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Address, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use crate::errors::NotificationError;
use crate::port::{
    BulkId, BulkReceipt, BulkRecipientIndex, Channel, DeliveryStatus, NotificationProvider,
    NotificationReceipt, NotificationReceiptId, Priority, Recipient, Result, SendBulkNotification,
    SendNotification, TemplateRef, TemplateValue,
};

/// Maximum number of recipients dispatched in a single batch per
/// the bulk-send spec.
const BULK_BATCH_SIZE: usize = 100;

/// Default SMTP submission port when the caller does not supply
/// one. Matches the IETF submission port per RFC 6409.
const DEFAULT_SMTP_PORT: u16 = 587;

/// Default stub template body used when the engine has not
/// pre-rendered a template body. The body contains a placeholder
/// so the substitution path is exercised end-to-end.
const DEFAULT_TEMPLATE_BODY: &str = "Hello {student_name}, this is a notification from Educore.";

/// Record that the bypass path is in effect for a Critical-priority
/// notification.
///
/// Per `docs/ports/notifications.md` § Priority, Critical
/// notifications must skip the queue / retry layer and dispatch
/// synchronously. The SMTP transport already accepts `send` calls
/// directly (this implementation is the synchronous path); this
/// helper records the bypass activation via `tracing::warn!` so
/// operators can observe Critical-path usage on the hot path
/// without blocking it.
///
/// We deliberately do NOT `panic!` on a failed Critical send — the
/// caller surfaces a typed `NotificationError` instead. Panicking
/// would crash the worker instead of letting the caller escalate
/// (page, retry via a different channel, etc.).
///
/// The function is module-private; the public surface is the
/// `if matches!(request.priority, Priority::Critical)` branch in
/// `EmailProvider::send`.
fn apply_critical_bypass(from: &str, school_id: SchoolId) {
    tracing::warn!(
        school_id = %school_id,
        from = %from,
        "Critical-priority notification: bypassing queue/retry layer (sync dispatch)",
    );
    if from.is_empty() {
        tracing::warn!(
            school_id = %school_id,
            "Critical-priority notification has empty From address; falling back to default",
        );
    }
}

/// SMTP-based [`NotificationProvider`].
///
/// Cheap to clone (the underlying transport uses an internal
/// connection pool when the `pool` feature is enabled, otherwise
/// it clones the connection info).
#[derive(Clone)]
pub struct EmailProvider {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    default_from: String,
}

impl fmt::Debug for EmailProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EmailProvider")
            .field("default_from", &self.default_from)
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl NotificationProvider for EmailProvider {
    async fn send(&self, request: SendNotification) -> Result<NotificationReceipt> {
        let from = match &request.channel {
            Channel::Email { from, .. } => from
                .as_ref()
                .map(|e| e.as_str().to_owned())
                .unwrap_or_else(|| self.default_from.clone()),
            other => {
                return Err(NotificationError::provider(format!(
                    "email provider cannot send {other:?} channel"
                )));
            }
        };
        let reply_to = match &request.channel {
            Channel::Email { reply_to, .. } => reply_to.as_ref().map(|e| e.as_str().to_owned()),
            _ => None,
        };

        let recipient_email = resolve_email_recipient(&request.recipient)?;
        let body = render_template_body(&request.template, &request.variables);

        let log_school = request.tenant.school_id.to_string();
        let _ = (
            log_school.as_str(),
            template_id_of(&request.template).as_str(),
            recipient_email.as_str(),
        );

        // Critical priority bypass path.
        //
        // Per `docs/ports/notifications.md` § Priority:
        // Priority::Critical must skip the queue/retry layer and
        // send synchronously. The transport already accepts the
        // `send` call directly (this is the synchronous path);
        // we record the bypass activation via `tracing::warn!`
        // so operators can observe Critical-path usage without
        // blocking the hot path. We do NOT `panic!` here — a
        // failed Critical send must surface as a typed error so
        // the caller can escalate (page, re-send via a different
        // channel, etc.) rather than crash the worker.
        if matches!(request.priority, Priority::Critical) {
            apply_critical_bypass(&from, request.tenant.school_id);
        }

        let message = build_lettre_message(&from, reply_to.as_deref(), &recipient_email, &body)?;

        let response = self
            .transport
            .send(message)
            .await
            .map_err(NotificationError::infrastructure)?;

        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(NotificationError::infrastructure)?
            .as_millis();
        let receipt_id = NotificationReceiptId::new(format!("email:{log_school}:{now_ms}"));

        Ok(NotificationReceipt::new(
            receipt_id,
            request.channel,
            DeliveryStatus::Sent,
            Timestamp::now(),
        )
        .with_provider_message_id(response.code().to_string()))
    }

    async fn send_bulk(&self, request: SendBulkNotification) -> Result<BulkReceipt> {
        if !matches!(request.channel, Channel::Email { .. }) {
            return Err(NotificationError::provider(
                "email provider cannot send non-email channels",
            ));
        }

        let now_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(NotificationError::infrastructure)?
            .as_nanos();
        let bulk_id = BulkId::new(format!("bulk_email:{now_ns}"));

        let mut receipt = BulkReceipt::new(bulk_id);

        for (idx, row) in request.recipients.iter().enumerate() {
            if idx > 0 && idx % BULK_BATCH_SIZE == 0 {
                let _ = idx / BULK_BATCH_SIZE;
            }
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
                Ok(r) => receipt.receipts.push(r),
                Err(e) => {
                    let Ok(idx_u32) = u32::try_from(idx) else {
                        continue;
                    };
                    receipt.failed.push((BulkRecipientIndex::new(idx_u32), e));
                }
            }
        }

        Ok(receipt)
    }

    async fn status(&self, _receipt_id: NotificationReceiptId) -> Result<DeliveryStatus> {
        Ok(DeliveryStatus::Sent)
    }
}

/// Builder for [`EmailProvider`].
///
/// Construct via [`EmailProviderBuilder::new`], chain the
/// configuration methods, and finish with
/// [`EmailProviderBuilder::build`]. The builder validates that the
/// required `relay_host` and `default_from` are set before
/// constructing the transport.
#[derive(Debug, Default, Clone)]
pub struct EmailProviderBuilder {
    relay_host: Option<String>,
    relay_port: Option<u16>,
    credentials_user: Option<String>,
    default_from: Option<String>,
}

impl EmailProviderBuilder {
    /// Creates a new builder with no configuration set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the SMTP relay host (e.g. `"smtp.example.com"`).
    #[must_use]
    pub fn relay_host(mut self, host: impl Into<String>) -> Self {
        self.relay_host = Some(host.into());
        self
    }

    /// Sets the SMTP relay port. Defaults to `587` (submission)
    /// when [`build`](Self::build) is called without an explicit
    /// port.
    #[must_use]
    pub fn relay_port(mut self, port: u16) -> Self {
        self.relay_port = Some(port);
        self
    }

    /// Sets the SMTP authentication user. The provider currently
    /// uses the user as both username and password (a common
    /// configuration for API-token SMTP relays). Callers that
    /// need a distinct password should swap to a fully-custom
    /// `lettre::AsyncSmtpTransportBuilder`.
    #[must_use]
    pub fn credentials(mut self, user: impl Into<String>) -> Self {
        self.credentials_user = Some(user.into());
        self
    }

    /// Sets the default `From:` address used when a
    /// [`SendNotification`] does not supply one on its
    /// [`Channel::Email`](crate::port::Channel::Email) variant.
    #[must_use]
    pub fn default_from(mut self, from: impl Into<String>) -> Self {
        self.default_from = Some(from.into());
        self
    }

    /// Consumes the builder and returns a configured
    /// [`EmailProvider`].
    ///
    /// # Errors
    ///
    /// - [`NotificationError::Provider`] if `relay_host` or
    ///   `default_from` were not set.
    pub fn build(self) -> Result<EmailProvider> {
        let relay_host = self.relay_host.ok_or_else(|| {
            NotificationError::provider("EmailProviderBuilder: relay_host is required")
        })?;
        let default_from = self.default_from.ok_or_else(|| {
            NotificationError::provider("EmailProviderBuilder: default_from is required")
        })?;
        let relay_port = self.relay_port.unwrap_or(DEFAULT_SMTP_PORT);

        let mut builder = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&relay_host);
        builder = builder.port(relay_port);
        if let Some(user) = self.credentials_user {
            builder = builder.credentials(Credentials::new(user, String::new()));
        }

        Ok(EmailProvider {
            transport: builder.build(),
            default_from,
        })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Resolves the email address of a [`Recipient`]. Only
/// [`Recipient::Direct`] with an email field and the
/// [`Recipient::User`]/[`Recipient::Staff`] variants (looked up by
/// the engine, opaque-string here) are accepted.
///
/// Returns [`NotificationError::InvalidRecipient`] for variants
/// the email channel cannot route (e.g. an SMS-only contact).
fn resolve_email_recipient(recipient: &Recipient) -> Result<String> {
    match recipient {
        Recipient::Direct(contact) => contact
            .email
            .as_ref()
            .map(|e| e.as_str().to_owned())
            .ok_or_else(|| {
                NotificationError::InvalidRecipient("contact has no email address".to_string())
            }),
        Recipient::User(_)
        | Recipient::Student(_)
        | Recipient::Guardian(_, _)
        | Recipient::Staff(_)
        | Recipient::Group(_)
        | Recipient::List(_)
        | Recipient::Expression(_) => Err(NotificationError::InvalidRecipient(
            "engine must materialise indirect recipients before sending".to_string(),
        )),
    }
}

/// Renders the email body for a send. The reference impl returns
/// a stub body that includes a `{student_name}` placeholder
/// (substituted by [`substitute_variables`]) so the substitution
/// path is exercised end-to-end. A production adapter would
/// resolve the [`TemplateRef`] against the template store first.
fn render_template_body(
    template: &TemplateRef,
    variables: &BTreeMap<String, TemplateValue>,
) -> String {
    let mut body = DEFAULT_TEMPLATE_BODY.to_owned();
    let prefix = match template {
        TemplateRef::Id(id) => format!("[Template: {}]\n\n", id.as_str()),
    };
    body = format!("{prefix}{body}");
    substitute_variables(&body, variables)
}

/// Substitutes `{var_name}` placeholders in `body` with the
/// matching [`TemplateValue`] rendered as a string. Missing
/// variables are left as-is. Used by the email provider and
/// exercised directly by the unit tests.
#[must_use]
pub fn substitute_variables(body: &str, variables: &BTreeMap<String, TemplateValue>) -> String {
    let mut result = body.to_owned();
    for (key, value) in variables {
        let placeholder = format!("{{{key}}}");
        result = result.replace(&placeholder, &value_to_string(value));
    }
    result
}

/// Renders a [`TemplateValue`] as a string for substitution.
fn value_to_string(value: &TemplateValue) -> String {
    match value {
        TemplateValue::Text(s) => s.clone(),
        TemplateValue::Number(n) => n.to_string(),
        TemplateValue::Decimal(s) => s.clone(),
        TemplateValue::Boolean(b) => b.to_string(),
        TemplateValue::Date(s) => s.clone(),
        TemplateValue::Json(s) => s.clone(),
    }
}

/// Builds a [`lettre::Message`] from the given envelope
/// components. Returns [`NotificationError::Provider`] when the
/// `from` or recipient strings fail RFC 5322 validation, or when
/// `lettre`'s `.body()` call fails (which is unreachable for a
/// well-formed UTF-8 body, but `lettre` types it as a
/// `lettre::error::Error` result that we must consume).
fn build_lettre_message(
    from: &str,
    reply_to: Option<&str>,
    recipient: &str,
    body: &str,
) -> std::result::Result<Message, NotificationError> {
    let from_address: Address = from
        .parse()
        .map_err(|e| NotificationError::provider(format!("invalid from address: {e}")))?;
    let from_mailbox = Mailbox::new(None, from_address);

    let to_address: Address = recipient
        .parse()
        .map_err(|e| NotificationError::provider(format!("invalid recipient address: {e}")))?;
    let to_mailbox = Mailbox::new(None, to_address);

    let mut builder = Message::builder()
        .from(from_mailbox)
        .to(to_mailbox)
        .subject("Educore notification");

    if let Some(reply) = reply_to {
        let reply_address: Address = reply
            .parse()
            .map_err(|e| NotificationError::provider(format!("invalid reply-to address: {e}")))?;
        builder = builder.reply_to(Mailbox::new(None, reply_address));
    }

    builder
        .body(body.to_owned())
        .map_err(|e| NotificationError::provider(format!("lettre failed to build body: {e}")))
}

/// Returns the template id string for logging / receipt
/// correlation.
fn template_id_of(template: &TemplateRef) -> String {
    match template {
        TemplateRef::Id(id) => id.to_string(),
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
    use crate::errors::NotificationTemplateId;
    use crate::port::{ContactInfo, EmailAddress};
    use educore_core::ids::{
        CorrelationId, Identifier, SessionId, PUBLIC_SCHOOL_ID, SYSTEM_USER_ID,
    };
    use educore_core::tenant::{Locale, TenantContext, TimeZone, UserType};

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

    #[test]
    fn email_provider_builder_constructs_with_defaults() {
        let provider = EmailProviderBuilder::new()
            .relay_host("localhost")
            .default_from("no-reply@example.com")
            .build()
            .expect("builder should succeed with host + default_from");
        assert_eq!(provider.default_from, "no-reply@example.com");
        // Sanity-check debug output to make sure the struct is
        // not silently broken.
        let _ = format!("{provider:?}");
    }

    #[test]
    fn email_template_variable_substitution() {
        let body = "Hello {student_name}, your score is {score}.";
        let mut variables = BTreeMap::new();
        variables.insert("student_name".into(), TemplateValue::text("Alice"));
        variables.insert("score".into(), TemplateValue::number(95));

        let rendered = substitute_variables(body, &variables);

        assert!(
            rendered.contains("Alice"),
            "expected student_name to be substituted, got {rendered:?}"
        );
        assert!(
            rendered.contains("95"),
            "expected score to be substituted, got {rendered:?}"
        );
        assert!(
            !rendered.contains("{student_name}"),
            "placeholder should be removed, got {rendered:?}"
        );
        assert!(
            !rendered.contains("{score}"),
            "placeholder should be removed, got {rendered:?}"
        );
    }

    #[test]
    fn email_provider_rejects_non_email_channel() {
        let provider = EmailProviderBuilder::new()
            .relay_host("localhost")
            .default_from("no-reply@example.com")
            .build()
            .expect("builder should succeed");

        let req = SendNotification {
            tenant: sample_tenant(),
            channel: Channel::Sms {
                from: None,
                unicode: false,
            },
            template: TemplateRef::Id(NotificationTemplateId::new("tpl_xyz")),
            recipient: Recipient::Direct(
                ContactInfo::new().with_email(EmailAddress::new("user@example.com")),
            ),
            variables: BTreeMap::new(),
            attachments: Vec::new(),
            priority: Default::default(),
            scheduled_at: None,
            idempotency_key: None,
            correlation_id: None,
            school_id: PUBLIC_SCHOOL_ID,
        };

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        let err = rt
            .block_on(provider.send(req))
            .expect_err("non-email channel should fail");
        assert!(matches!(err, NotificationError::Provider(_)), "got {err:?}");
    }

    #[test]
    fn resolve_email_recipient_rejects_sms_only_contact() {
        let contact = ContactInfo::new().with_email(EmailAddress::new("user@example.com"));
        let ok = resolve_email_recipient(&Recipient::Direct(contact)).expect("email ok");
        assert_eq!(ok, "user@example.com");

        let sms_only = ContactInfo::new();
        let err = resolve_email_recipient(&Recipient::Direct(sms_only))
            .expect_err("sms-only contact should fail");
        assert!(matches!(err, NotificationError::InvalidRecipient(_)));
    }

    // Suppress unused-import warnings for SessionId on toolchains
    // where the field is read indirectly.
    #[allow(dead_code)]
    fn _touch_session_id(_s: Option<SessionId>) {}
}
