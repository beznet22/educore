//! # SMS [`NotificationProvider`] reference implementation.
//!
//! The provider delivers notifications on the
//! [`Channel::Sms`](crate::port::Channel::Sms) transport by POSTing
//! `application/x-www-form-urlencoded` requests to a generic HTTP
//! gateway. The default gateway URL is the Twilio Messages shape,
//! but the builder accepts any compatible endpoint so the same
//! provider drives Twilio, Plivo, Vonage, MessageBird, an internal
//! proxy, etc.
//!
//! See `docs/ports/notifications.md` § "SMS" for the port-side
//! contract; this file is the adapter side.
//!
//! ## Authentication
//!
//! Every request carries an `Authorization: Basic` header built
//! from the configured `api_key` (the raw `api_key:` base64
//! encoded). For the default Twilio URL the `api_key` is the
//! Account SID and the URL has `{account}` interpolated from it;
//! consumers wiring real Twilio credentials should pre-encode
//! `{AccountSID}:{AuthToken}` as base64 and pass the resulting
//! string as `api_key` (or wire their own transport on top of
//! [`SmsProviderBuilder`]). The base64 alphabet is the standard
//! RFC 4648 one; a small inline encoder avoids pulling the
//! `base64` crate into the dep graph.

use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use reqwest::Client;

use educore_core::value_objects::Timestamp;

use crate::errors::{NotificationError, NotificationTemplateId};
use crate::port::{
    BulkId, BulkReceipt, BulkRecipient, BulkRecipientIndex, Channel, DeliveryStatus,
    NotificationProvider, NotificationReceipt, NotificationReceiptId, PhoneNumber, Recipient,
    SendBulkNotification, SendNotification,
};
// The port exposes a `Result<T>` type alias for
// `std::result::Result<T, NotificationError>`. Bring it into
// scope so the trait method signatures read naturally.
use crate::port::Result;

/// The default SMS gateway URL. Twilio's `Messages` endpoint with
/// `{account}` placeholder; the provider substitutes the configured
/// `api_key` (the Twilio Account SID) at send time.
const DEFAULT_GATEWAY_URL: &str =
    "https://api.twilio.com/2010-04-01/Accounts/{account}/Messages.json";

/// The bulk batch size per `docs/ports/notifications.md` § "Bulk
/// limits". Batches larger than this must be split.
const SMS_BULK_BATCH_SIZE: usize = 100;

// ---------------------------------------------------------------------------
// SmsProvider
// ---------------------------------------------------------------------------

/// The HTTP-backed SMS [`NotificationProvider`].
///
/// Holds a shared [`reqwest::Client`], the gateway URL, the
/// `api_key` used for HTTP Basic auth, the default originating
/// phone number (used when a `SendNotification` does not supply
/// `Channel::Sms.from`), and a registered template-body map.
///
/// Object safety: the trait is object-safe; the concrete type here
/// is `Send + Sync + Debug`.
pub struct SmsProvider {
    http: Client,
    gateway_url: String,
    api_key: String,
    default_from: String,
    templates: HashMap<NotificationTemplateId, String>,
}

/// Manual `Debug` to redact the `api_key` — the field is a
/// credential, not a value to leak into log output.
impl fmt::Debug for SmsProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SmsProvider")
            .field("http", &"reqwest::Client { .. }")
            .field("gateway_url", &self.gateway_url)
            .field("api_key", &"<redacted>")
            .field("default_from", &self.default_from)
            .field("templates", &self.templates.keys().collect::<Vec<_>>())
            .finish()
    }
}

// ---------------------------------------------------------------------------
// SmsProviderBuilder
// ---------------------------------------------------------------------------

/// Builder for [`SmsProvider`].
///
/// Defaults match the Twilio Messages shape: the gateway URL is
/// the Twilio endpoint with `{account}` placeholder, the `api_key`
/// is the Account SID, and the `default_from` is empty (the caller
/// must supply an originating phone number via `Channel::Sms.from`
/// or override the builder default).
#[derive(Debug, Clone)]
pub struct SmsProviderBuilder {
    gateway_url: Option<String>,
    api_key: String,
    default_from: String,
    templates: HashMap<NotificationTemplateId, String>,
}

impl SmsProviderBuilder {
    /// Constructs a builder with the Twilio-shaped defaults.
    /// Callers must override [`SmsProviderBuilder::api_key`] (and
    /// typically [`SmsProviderBuilder::default_from`]) before
    /// [`SmsProviderBuilder::build`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            gateway_url: None,
            api_key: String::new(),
            default_from: String::new(),
            templates: HashMap::new(),
        }
    }

    /// Sets the gateway URL. When `None` (the default), the URL
    /// is the Twilio Messages endpoint with `{account}`
    /// substituted from the configured [`SmsProviderBuilder::api_key`].
    #[must_use]
    pub fn gateway_url(mut self, url: impl Into<String>) -> Self {
        self.gateway_url = Some(url.into());
        self
    }

    /// Sets the `api_key`. For the default Twilio URL this is the
    /// Account SID; for custom gateways it is whatever the gateway
    /// accepts as the HTTP Basic auth user (the password half is
    /// left empty). Consumers needing full `{SID}:{AuthToken}`
    /// auth should pre-encode the pair as base64 and pass the
    /// resulting string here.
    #[must_use]
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = api_key.into();
        self
    }

    /// Sets the default originating phone number used when a
    /// `SendNotification` does not supply `Channel::Sms.from`.
    /// The value is in E.164 form (e.g. `+14155550101`); the
    /// adapter does not validate the format itself.
    #[must_use]
    pub fn default_from(mut self, from: impl Into<String>) -> Self {
        self.default_from = from.into();
        self
    }

    /// Registers a template body string for the given template id.
    /// The body may contain `{{variable_name}}` placeholders that
    /// are substituted from the request's `variables` map at send
    /// time. Calling with the same id twice overwrites the prior
    /// registration.
    #[must_use]
    pub fn template_body(mut self, id: NotificationTemplateId, body: impl Into<String>) -> Self {
        self.templates.insert(id, body.into());
        self
    }

    /// Consumes the builder and returns the configured provider.
    /// Resolves the gateway URL: an explicit URL wins; otherwise
    /// the Twilio default is used with `{account}` interpolated
    /// from `api_key`.
    ///
    /// # Errors
    ///
    /// Returns [`NotificationError::Infrastructure`] if the
    /// underlying [`reqwest::Client`] cannot be constructed (e.g.
    /// the process TLS configuration is malformed).
    pub fn build(self) -> Result<SmsProvider> {
        let gateway_url = self
            .gateway_url
            .unwrap_or_else(|| DEFAULT_GATEWAY_URL.replace("{account}", &self.api_key));
        let http = Client::builder()
            .build()
            .map_err(NotificationError::infrastructure)?;
        Ok(SmsProvider {
            http,
            gateway_url,
            api_key: self.api_key,
            default_from: self.default_from,
            templates: self.templates,
        })
    }
}

impl Default for SmsProviderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

impl SmsProvider {
    /// Returns the `To:` phone number for a [`Recipient`], or
    /// [`NotificationError::InvalidRecipient`] when the recipient
    /// cannot be resolved to a phone on the boundary (e.g. a
    /// `User`/`Student`/`Staff` whose contact record the adapter
    /// does not have). The reference impl only knows how to
    /// resolve direct contacts that already carry a phone number;
    /// consumers wiring this to a real user store should layer
    /// the lookup at the boundary before calling the adapter.
    fn recipient_phone(&self, recipient: &Recipient) -> Result<PhoneNumber> {
        match recipient {
            Recipient::Direct(info) => info
                .phone
                .clone()
                .ok_or_else(|| NotificationError::InvalidRecipient("no phone on contact".into())),
            Recipient::List(_) => Err(NotificationError::InvalidRecipient(
                "nested recipient list not expanded by engine".into(),
            )),
            Recipient::Expression(_) => Err(NotificationError::InvalidRecipient(
                "recipient expression not expanded by engine".into(),
            )),
            Recipient::User(_)
            | Recipient::Student(_)
            | Recipient::Guardian(_, _)
            | Recipient::Staff(_)
            | Recipient::Group(_) => Err(NotificationError::InvalidRecipient(
                "recipient requires contact lookup; not supported by reference SmsProvider".into(),
            )),
        }
    }

    /// Renders a template body by substituting `{{name}}`
    /// placeholders from the variable map. Variables that do not
    /// appear in the body are silently ignored; placeholders
    /// without a matching variable are left intact (so a missing
    /// variable surfaces as `{{name}}` in the final body rather
    /// than empty text, which would mask the bug).
    fn render_template(
        body: &str,
        variables: &std::collections::BTreeMap<String, crate::port::TemplateValue>,
    ) -> String {
        let mut out = body.to_owned();
        for (name, value) in variables {
            let needle = format!("{{{{{name}}}}}");
            out = out.replace(&needle, &template_value_as_str(value));
        }
        out
    }

    /// Encodes the `From:` value for the form. The `from` field
    /// on `Channel::Sms` wins; otherwise the builder's
    /// `default_from` is used. An empty `default_from` and a
    /// `None` channel-side `from` returns
    /// [`NotificationError::InvalidRecipient`] so the engine sees
    /// a structured failure instead of a silent provider error.
    fn resolve_from(&self, channel: &Channel) -> Result<String> {
        let Channel::Sms { from, unicode: _ } = channel else {
            return Err(NotificationError::Provider(
                "sms provider cannot send non-sms channels".into(),
            ));
        };
        if let Some(p) = from {
            return Ok(p.as_str().to_owned());
        }
        if self.default_from.is_empty() {
            return Err(NotificationError::InvalidRecipient(
                "no sms `from` configured and channel did not supply one".into(),
            ));
        }
        Ok(self.default_from.clone())
    }

    /// Looks up the template body registered for `template_id`,
    /// returning [`NotificationError::TemplateNotFound`] if no
    /// body is registered.
    fn lookup_body(&self, template_id: &NotificationTemplateId) -> Result<String> {
        self.templates
            .get(template_id)
            .cloned()
            .ok_or_else(|| NotificationError::TemplateNotFound(template_id.clone()))
    }

    /// Builds the HTTP Basic auth header value from the
    /// configured `api_key`. The username half is the raw
    /// `api_key`; the password half is empty. Uses the inline
    /// [`base64_encode`] helper so we do not pull `base64` into
    /// the dep graph.
    ///
    /// # Errors
    ///
    /// Returns [`NotificationError::Provider`] if the base64
    /// encoder fails (impossible in practice — the alphabet index
    /// is always in `[0, 64)` by construction — but the type
    /// system requires the propagation path).
    fn basic_auth_header(&self) -> Result<String> {
        let raw = format!("{}:", self.api_key);
        Ok(format!("Basic {}", base64_encode(raw.as_bytes())?))
    }

    /// Dispatches a single `SendNotification` to the configured
    /// gateway and returns the resulting [`NotificationReceipt`].
    /// Called by both [`SmsProvider::send`] and the per-row path
    /// inside [`SmsProvider::send_bulk`] so the two share the
    /// exact same wire-format and status mapping.
    async fn dispatch(&self, request: &SendNotification) -> Result<NotificationReceipt> {
        let from = self.resolve_from(&request.channel)?;
        let template_id = match &request.template {
            crate::port::TemplateRef::Id(id) => id.clone(),
        };
        let body = self.lookup_body(&template_id)?;
        let rendered = Self::render_template(&body, &request.variables);
        let to = self.recipient_phone(&request.recipient)?;

        let receipt_id = NotificationReceiptId::new(generate_id("sms")?);
        let channel = request.channel.clone();

        let response = self
            .http
            .post(&self.gateway_url)
            .header("Authorization", self.basic_auth_header()?)
            .form(&[
                ("To", to.as_str()),
                ("From", from.as_str()),
                ("Body", rendered.as_str()),
            ])
            .send()
            .await
            .map_err(NotificationError::infrastructure)?;

        let status_code = response.status();
        let body = response
            .text()
            .await
            .map_err(NotificationError::infrastructure)?;
        let provider_message_id = extract_provider_message_id(&body);

        let status = if status_code.as_u16() == 202 {
            DeliveryStatus::Queued
        } else if status_code.is_success() {
            DeliveryStatus::Sent
        } else {
            return Err(NotificationError::provider(format!(
                "sms gateway returned status {status_code}"
            )));
        };

        let mut receipt = NotificationReceipt::new(receipt_id, channel, status, Timestamp::now());
        if let Some(sid) = provider_message_id {
            receipt = receipt.with_provider_message_id(sid);
        }
        Ok(receipt)
    }
}

// ---------------------------------------------------------------------------
// NotificationProvider impl
// ---------------------------------------------------------------------------

#[async_trait]
impl NotificationProvider for SmsProvider {
    async fn send(&self, request: SendNotification) -> Result<NotificationReceipt> {
        if !matches!(request.channel, Channel::Sms { .. }) {
            return Err(NotificationError::Provider(
                "sms provider cannot send non-sms channels".into(),
            ));
        }
        self.dispatch(&request).await
    }

    async fn send_bulk(&self, request: SendBulkNotification) -> Result<BulkReceipt> {
        if !matches!(request.channel, Channel::Sms { .. }) {
            return Err(NotificationError::Provider(
                "sms provider cannot send non-sms channels".into(),
            ));
        }

        let bulk_id = BulkId::new(generate_id("bulk")?);
        let mut receipt = BulkReceipt::new(bulk_id);

        // Honor the per-request SMS bulk batch size from
        // `docs/ports/notifications.md` (100 per request).
        // We chunk the recipient vector into batches of
        // SMS_BULK_BATCH_SIZE and process each batch serially.
        // (Parallel fan-out would require a join semaphore; the
        // reference impl keeps it sequential so the on-the-wire
        // ordering matches input ordering.)
        for chunk in request.recipients.chunks(SMS_BULK_BATCH_SIZE) {
            for (offset, row) in chunk.iter().enumerate() {
                let global_idx = receipt.total();
                let index = BulkRecipientIndex::new(u32::try_from(global_idx).map_err(|e| {
                    NotificationError::provider(format!(
                        "recipient index overflow at global_idx={global_idx}: {e}"
                    ))
                })?);
                let single = build_single_from_bulk(&request, row);
                match self.dispatch(&single).await {
                    Ok(r) => receipt.receipts.push(r),
                    Err(e) => receipt.failed.push((index, e)),
                }
                // `offset` is reserved for any future
                // intra-batch ordering logic; reference impl is
                // strictly sequential so it goes unused here.
                let _ = offset;
            }
        }

        Ok(receipt)
    }

    async fn status(&self, _receipt_id: NotificationReceiptId) -> Result<DeliveryStatus> {
        // Reference impl: status lookup is a stub. A production
        // adapter would call the gateway's status endpoint
        // (e.g. Twilio `GET /Messages/{Sid}.json`) and translate
        // the wire response into the engine's DeliveryStatus
        // enum.
        Ok(DeliveryStatus::Sent)
    }
}

// ---------------------------------------------------------------------------
// Free helpers
// ---------------------------------------------------------------------------

/// Builds a single-recipient `SendNotification` from a bulk row.
/// Mirrors the bulk request's channel, template, priority, and
/// scheduling onto the single send; the per-row variables map
/// (when `variables_per_recipient` is set) overrides the bulk
/// request's variables.
fn build_single_from_bulk(bulk: &SendBulkNotification, row: &BulkRecipient) -> SendNotification {
    // The bulk request carries no shared variables map; every
    // row's variables live on `BulkRecipient.variables` and the
    // `variables_per_recipient` flag is consumed by the engine at
    // request-build time. Use the row's map verbatim.
    let variables = row.variables.clone();
    SendNotification {
        tenant: bulk.tenant.clone(),
        channel: bulk.channel.clone(),
        template: bulk.template.clone(),
        recipient: row.recipient.clone(),
        variables,
        attachments: Vec::new(),
        priority: bulk.priority,
        scheduled_at: bulk.scheduled_at,
        idempotency_key: bulk.idempotency_key,
        correlation_id: bulk.correlation_id,
        school_id: bulk.school_id,
    }
}

/// Renders a [`TemplateValue`](crate::port::TemplateValue) to the
/// string form used inside the SMS body. Decimal and JSON values
/// pass through verbatim (they're already stringly encoded at the
/// port boundary); dates use the raw string the engine supplied.
fn template_value_as_str(value: &crate::port::TemplateValue) -> String {
    use crate::port::TemplateValue;
    match value {
        TemplateValue::Text(s)
        | TemplateValue::Decimal(s)
        | TemplateValue::Date(s)
        | TemplateValue::Json(s) => s.clone(),
        TemplateValue::Number(n) => n.to_string(),
        TemplateValue::Boolean(b) => b.to_string(),
    }
}

/// Tries to extract a Twilio `sid` (or any other gateway's
/// equivalent message id) from a JSON-ish response body using a
/// tiny hand-rolled string scan. Returns `None` if the body is
/// not JSON or has no `sid` field; the receipt will still be
/// returned successfully, just without a `provider_message_id`.
/// Avoids pulling `serde_json` into the dep graph for a single
/// optional field.
fn extract_provider_message_id(body: &str) -> Option<String> {
    let key = "\"sid\"";
    let key_pos = body.find(key)?;
    let after_key = &body[key_pos + key.len()..];
    let colon = after_key.find(':')?;
    let after_colon = after_key[colon + 1..].trim_start();
    let open = after_colon.find('"')?;
    let value_start = open + 1;
    let rest = &after_colon[value_start..];
    let close = rest.find('"')?;
    Some(rest[..close].to_owned())
}

/// Generates an opaque, monotonically increasing id string of the
/// form `{prefix}-{hex_unix_micros}-{hex_counter}`. We avoid
/// pulling `uuid` into the dep graph for what is ultimately a
/// non-cryptographic, engine-local identifier; the string is
/// unique per (process, microsecond, counter-tick) and is treated
/// as opaque by every downstream consumer.
///
/// # Errors
///
/// Returns [`NotificationError::Infrastructure`] if the system
/// clock is before the Unix epoch (which would indicate clock
/// skew so severe that no meaningful timestamp can be produced).
fn generate_id(prefix: &str) -> Result<String> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    let micros = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(NotificationError::infrastructure)?
        .as_micros();
    Ok(format!("{prefix}-{micros:x}-{counter:x}"))
}

/// RFC 4648 § 4 standard-alphabet base64 encoder. A handful of
/// lines of arithmetic so the crate does not need a `base64`
/// dependency for one header value.
///
/// # Errors
///
/// Returns [`NotificationError::Provider`] if the alphabet index
/// cannot be narrowed to `usize` (impossible on the supported
/// targets: the index is always in `[0, 64)` because of the
/// `& 0x3F` mask, but the encoder is typed as fallible so the
/// caller can `?` the result without a separate failure path).
fn base64_encode(input: &[u8]) -> Result<String> {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    let mut i = 0;
    while i + 3 <= input.len() {
        let n =
            (u32::from(input[i]) << 16) | (u32::from(input[i + 1]) << 8) | u32::from(input[i + 2]);
        push_base64_char(&mut out, ALPHABET, (n >> 18) & 0x3F)?;
        push_base64_char(&mut out, ALPHABET, (n >> 12) & 0x3F)?;
        push_base64_char(&mut out, ALPHABET, (n >> 6) & 0x3F)?;
        push_base64_char(&mut out, ALPHABET, n & 0x3F)?;
        i += 3;
    }
    let rem = input.len() - i;
    if rem == 1 {
        let n = u32::from(input[i]) << 16;
        push_base64_char(&mut out, ALPHABET, (n >> 18) & 0x3F)?;
        push_base64_char(&mut out, ALPHABET, (n >> 12) & 0x3F)?;
        out.push('=');
        out.push('=');
    } else if rem == 2 {
        let n = (u32::from(input[i]) << 16) | (u32::from(input[i + 1]) << 8);
        push_base64_char(&mut out, ALPHABET, (n >> 18) & 0x3F)?;
        push_base64_char(&mut out, ALPHABET, (n >> 12) & 0x3F)?;
        push_base64_char(&mut out, ALPHABET, (n >> 6) & 0x3F)?;
        out.push('=');
    }
    Ok(out)
}

/// Looks up a base64-alphabet index and pushes the corresponding
/// character to `out`. The two numeric narrowings use
/// [`usize::try_from`] and [`char::From<u8>`] (the latter is a
/// total conversion for any `u8`) so neither step relies on an
/// `as` cast; the only way this returns an error is if `index`
/// cannot be represented as `usize`, which the `& 0x3F` mask
/// forbids by construction.
fn push_base64_char(out: &mut String, alphabet: &[u8; 64], index: u32) -> Result<()> {
    let idx = usize::try_from(index)
        .map_err(|e| NotificationError::provider(format!("base64 alphabet index overflow: {e}")))?;
    out.push(char::from(alphabet[idx]));
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::missing_docs_in_private_items
)]
mod tests {
    use super::*;
    use crate::port::{ContactInfo, TemplateValue};
    use educore_core::clock::{IdGenerator, SystemIdGen};
    use educore_core::tenant::{TenantContext, UserType};
    use std::collections::BTreeMap;

    fn tenant() -> TenantContext {
        TenantContext::for_user(
            SystemIdGen.next_school_id(),
            SystemIdGen.next_user_id(),
            SystemIdGen.next_correlation_id(),
            UserType::Staff,
        )
    }

    #[test]
    fn test_sms_provider_builder_constructs_with_defaults() {
        let provider = SmsProviderBuilder::new()
            .api_key("AC0123456789abcdef")
            .default_from("+14155550101")
            .build()
            .expect("builder should succeed with api_key + default_from");

        // Default URL is the Twilio Messages endpoint with
        // {account} interpolated from api_key.
        assert_eq!(
            provider.gateway_url,
            "https://api.twilio.com/2010-04-01/Accounts/AC0123456789abcdef/Messages.json"
        );
        assert_eq!(provider.api_key, "AC0123456789abcdef");
        assert_eq!(provider.default_from, "+14155550101");
        assert!(provider.templates.is_empty());
    }

    #[test]
    fn test_sms_template_variable_substitution() {
        let body = "Hello {{name}}, your balance is {{amount}} {{currency}}.";
        let mut vars = BTreeMap::new();
        vars.insert("name".to_owned(), TemplateValue::text("Alice"));
        vars.insert("amount".to_owned(), TemplateValue::number(42));
        vars.insert("currency".to_owned(), TemplateValue::text("USD"));
        // `unused` placeholder maps to a key that is missing in
        // the body; verify the missing-key path is a no-op.
        vars.insert("unused".to_owned(), TemplateValue::text("ignored"));

        let rendered = SmsProvider::render_template(body, &vars);
        assert_eq!(rendered, "Hello Alice, your balance is 42 USD.");
    }

    #[tokio::test]
    async fn test_sms_rejects_non_sms_channel() {
        let provider = SmsProviderBuilder::new()
            .api_key("k")
            .default_from("+15555550101")
            .build()
            .expect("builder should succeed");

        let request = SendNotification {
            tenant: tenant(),
            channel: Channel::Email {
                from: None,
                reply_to: None,
            },
            template: crate::port::TemplateRef::Id(NotificationTemplateId::new("t")),
            recipient: Recipient::Direct(ContactInfo::new()),
            variables: BTreeMap::new(),
            attachments: Vec::new(),
            priority: Default::default(),
            scheduled_at: None,
            idempotency_key: None,
            correlation_id: None,
            school_id: SystemIdGen.next_school_id(),
        };

        let result = provider.send(request).await;
        assert!(matches!(result, Err(NotificationError::Provider(_))));
    }

    #[tokio::test]
    async fn test_sms_missing_template_returns_template_not_found() {
        let provider = SmsProviderBuilder::new()
            .api_key("k")
            .default_from("+15555550101")
            .build()
            .expect("builder should succeed");

        let request = SendNotification {
            tenant: tenant(),
            channel: Channel::Sms {
                from: None,
                unicode: false,
            },
            template: crate::port::TemplateRef::Id(NotificationTemplateId::new("missing")),
            recipient: Recipient::Direct(
                ContactInfo::new().with_phone(PhoneNumber::new("+15555550199")),
            ),
            variables: BTreeMap::new(),
            attachments: Vec::new(),
            priority: Default::default(),
            scheduled_at: None,
            idempotency_key: None,
            correlation_id: None,
            school_id: SystemIdGen.next_school_id(),
        };

        let result = provider.send(request).await;
        assert!(matches!(
            result,
            Err(NotificationError::TemplateNotFound(_))
        ));
    }

    #[tokio::test]
    async fn test_sms_bulk_batches_and_collects_failures() {
        let provider = SmsProviderBuilder::new()
            .api_key("k")
            .default_from("+15555550101")
            .template_body(NotificationTemplateId::new("t"), "hi {{name}}")
            .build()
            .expect("builder should succeed");

        // Every row uses a recipient that the reference adapter
        // cannot resolve locally (`Recipient::User`), so the bulk
        // path never actually fires an HTTP request and the test
        // stays a pure unit test regardless of network access.
        let mut recipients = Vec::new();
        for i in 0..3_usize {
            let mut vars = BTreeMap::new();
            vars.insert("name".to_owned(), TemplateValue::text(format!("u{i}")));
            recipients.push(
                BulkRecipient::new(Recipient::User(SystemIdGen.next_user_id()))
                    .with_variables(vars),
            );
        }
        recipients.push(BulkRecipient::new(Recipient::User(
            SystemIdGen.next_user_id(),
        )));

        let request = SendBulkNotification {
            tenant: tenant(),
            template: crate::port::TemplateRef::Id(NotificationTemplateId::new("t")),
            recipients,
            variables_per_recipient: true,
            channel: Channel::Sms {
                from: None,
                unicode: false,
            },
            priority: Default::default(),
            scheduled_at: None,
            idempotency_key: None,
            correlation_id: None,
            school_id: SystemIdGen.next_school_id(),
        };

        let result = provider.send_bulk(request).await;
        // Every row fails recipient resolution; the four rows
        // are accounted for across `failed` and the bulk_id is
        // populated.
        let receipt = result.expect("bulk send returns Ok wrapping per-row failures");
        assert_eq!(receipt.total(), 4);
        assert_eq!(receipt.success_count(), 0);
        assert_eq!(receipt.failure_count(), 4);
        assert!(!receipt.bulk_id.as_str().is_empty());
        // BulkReceipt::failed preserves the input index so the
        // engine can correlate the failure back to the row.
        assert_eq!(receipt.failed[0].0, BulkRecipientIndex::new(0));
        assert_eq!(receipt.failed[3].0, BulkRecipientIndex::new(3));
    }
}
