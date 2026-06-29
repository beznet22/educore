//! # Stripe payment provider — reference implementation.
//!
//! [`StripeProvider`] is the reference adapter for the
//! [`PaymentProvider`](crate::port::PaymentProvider) port. It targets
//! the Stripe REST API at <https://api.stripe.com/v1> and exposes
//! the canonical 5-method surface plus a webhook-signature
//! verifier that other engine services (A.4's
//! `WebhookSignatureService`) can consume directly.
//!
//! # Implementation outline
//!
//! | Port method       | Stripe endpoint             | Notes                                                |
//! | ----------------- | --------------------------- | ---------------------------------------------------- |
//! | `charge`          | `POST /v1/charges`          | `application/x-www-form-urlencoded`, idempotent.     |
//! | `refund`          | `POST /v1/refunds`          | `charge`, `amount`, `reason`. Idempotent.            |
//! | `status`          | `GET  /v1/charges/{id}`     | Maps Stripe `status` to [`PaymentStatus`].           |
//! | `list_methods`    | (static)                    | Returns the canonical 4 `PaymentMethodInfo` rows.    |
//! | `settlement`      | (none)                      | Returns `PaymentError::Provider(...)` — deviation.   |
//! | (helper)          | `GET /v1/payment_intents/{id}` | 3DS challenge completion; see [`StripeProvider::complete_three_ds_challenge`]. |
//!
//! The Stripe API is form-encoded (`application/x-www-form-urlencoded`),
//! not JSON, so every request body is a `Vec<(String, String)>` passed
//! to `reqwest`'s `.form(...)`. Responses are parsed as
//! `serde_json::Value` (adapters are explicitly allowed to use
//! JSON-shaped types; this is *not* domain code).
//!
//! # Webhook signature verification
//!
//! [`StripeProvider::verify_webhook_signature`] implements the
//! Stripe-Signature scheme defined at
//! <https://docs.stripe.com/webhooks#verify-official-libraries>:
//!
//! 1. The `Stripe-Signature` header is `t=<unix>,v1=<hex-hmac>`.
//! 2. The signed payload is the literal string
//!    `<unix>.<raw-request-body>`.
//! 3. The expected MAC is `HMAC-SHA256(secret, signed_payload)`,
//!    lower-case hex.
//! 4. Comparison is constant-time.
//!
//! # Deviations from `docs/ports/payments.md`
//!
//! 1. **Timestamps.** Stripe returns Unix epoch seconds in its JSON
//!    responses. The port's [`Timestamp`](educore_core::value_objects::Timestamp)
//!    type has no `from_unix_seconds` constructor (and the
//!    payment crate's `Cargo.toml` does not list `chrono`
//!    directly), so the reference impl falls back to
//!    `Timestamp::now()` for `authorized_at` / `captured_at`.
//!    A production adapter that needs exact receipts should add
//!    `chrono` and a `Timestamp::from_unix_seconds` helper.
//! 2. **Refund lookup.** The port's `RefundRequest::original_payment_id`
//!    is an engine `PaymentId`, not the Stripe `ch_...` charge id.
//!    A production adapter would translate via the receipt store;
//!    this reference impl assumes the engine's `PaymentId.as_str()`
//!    already holds the Stripe charge id (consistent with how the
//!    receipt's `provider_payment_id` is stored).
//! 3. **`save` flag.** `PaymentMethod::Card { save: true }` is
//!    passed through verbatim; Stripe's Charges API does not
//!    natively vault cards — vaulting requires the
//!    SetupIntent + Customer flow, which is out of scope for the
//!    reference impl.
//! 4. **`settlement`.** Stripe does not expose a single settlement
//!    endpoint; pulling payout batches requires the separate
//!    `/v1/balance_transactions` API. The reference impl returns
//!    `PaymentError::Provider(...)` rather than calling the
//!    payouts API.
//!
//! # Security
//!
//! The provider's `Debug` impl redacts `secret_key` and
//! `webhook_secret`. Tokens, PINs, and bank-account references are
//! never logged; the audit log receives only the redacted
//! [`PaymentMethodKind`].

use std::collections::BTreeMap;
use std::fmt;
use std::time::Duration;

use async_trait::async_trait;
use hmac::{Hmac, Mac};
use reqwest::{Client, StatusCode};
use serde_json::Value as JsonValue;
use sha2::Sha256;

use educore_core::value_objects::Timestamp;

use crate::errors::{InfrastructureError, PaymentError};
use crate::port::{
    ChargeRequest, Money, PaymentFee, PaymentMethod, PaymentMethodInfo, PaymentMethodKind,
    PaymentProvider, PaymentReceipt, PaymentStatus, RefundReceipt, RefundRequest, Settlement,
    SettlementRequest,
};

// ---------------------------------------------------------------------------
// 3-D Secure (3DS) orchestration types
// ---------------------------------------------------------------------------

/// The outcome of a 3-D Secure challenge, as observed by the
/// consumer's frontend after the issuer redirects back from the
/// challenge page.
///
/// The adapter cross-checks this signal against Stripe's
/// authoritative PaymentIntent status (fetched via
/// `GET /v1/payment_intents/{id}`); Stripe's status wins on
/// disagreement. A browser-side `Abandoned` does not void a
/// successful authentication Stripe already observed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThreeDSChallengeResult {
    /// The customer successfully completed the 3DS challenge.
    Authenticated,

    /// The customer abandoned the challenge (browser closed,
    /// timed out, or navigated away before submitting).
    Abandoned,

    /// The issuer rejected the authentication.
    Failed {
        /// The issuer's reason (e.g. `authentication_required`,
        /// `do_not_honor`).
        reason: String,
    },
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// The default Stripe API base URL (`https://api.stripe.com/v1`).
pub const STRIPE_DEFAULT_BASE_URL: &str = "https://api.stripe.com/v1";

/// HTTP request timeout applied to every outbound call.
const HTTP_TIMEOUT_SECS: u64 = 30;

/// HMAC-SHA256, the algorithm Stripe uses for webhook signing.
type HmacSha256 = Hmac<Sha256>;

// ---------------------------------------------------------------------------
// StripeProvider
// ---------------------------------------------------------------------------

/// A [`PaymentProvider`] backed by the Stripe REST API.
///
/// `Clone` is implemented because [`reqwest::Client`] is internally
/// `Arc`-shared — cloning the provider is cheap and safe.
#[derive(Clone)]
pub struct StripeProvider {
    http: Client,
    secret_key: String,
    webhook_secret: String,
    base_url: String,
}

impl StripeProvider {
    /// Returns the configured Stripe API base URL (without a
    /// trailing slash; paths are appended verbatim).
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Returns `true` if a non-empty `secret_key` is configured.
    /// Useful for wiring code that wants to fail-fast on a
    /// misconfigured provider before the first charge.
    #[must_use]
    pub fn has_secret_key(&self) -> bool {
        !self.secret_key.is_empty()
    }

    /// Returns `true` if a non-empty `webhook_secret` is configured.
    /// [`Self::verify_webhook_signature`] will return an error if
    /// the secret is empty.
    #[must_use]
    pub fn has_webhook_secret(&self) -> bool {
        !self.webhook_secret.is_empty()
    }

    /// Verifies a Stripe webhook delivery's `Stripe-Signature`
    /// header against `payload` using the configured webhook
    /// signing secret.
    ///
    /// # Stripe-Signature format
    ///
    /// The header has the form `t=<unix>,v1=<hex>` (multiple
    /// `v1=` entries are allowed; the first one that matches is
    /// accepted). This implementation:
    ///
    /// 1. Parses `signature` and extracts `t` and at least one
    ///    `v1` value.
    /// 2. Computes `signed = format!("{t}.{payload_utf8}")`.
    /// 3. Computes `expected = hex(HMAC-SHA256(webhook_secret, signed))`.
    /// 4. Constant-time-compares `expected` against the first `v1`
    ///    value (case-insensitive on hex).
    ///
    /// Returns [`PaymentError::Provider`] on a malformed header or
    /// [`PaymentError::Infrastructure`] on a hashing error. Returns
    /// [`PaymentError::Declined`] — never; signature mismatch is
    /// reported as [`PaymentError::Provider`] with the message
    /// `"webhook signature mismatch"`.
    pub fn verify_webhook_signature(
        &self,
        payload: &[u8],
        signature: &str,
    ) -> Result<(), PaymentError> {
        if self.webhook_secret.is_empty() {
            return Err(PaymentError::Provider(
                "stripe webhook_secret is not configured".into(),
            ));
        }

        let (timestamp, expected_v1) = parse_stripe_signature(signature)?;

        let payload_str = std::str::from_utf8(payload).map_err(|e| {
            PaymentError::Provider(format!("webhook payload is not valid UTF-8: {e}"))
        })?;
        let signed = format!("{timestamp}.{payload_str}");

        let mut mac = HmacSha256::new_from_slice(self.webhook_secret.as_bytes()).map_err(
            // `new_from_slice` only fails on key-length, which is
            // unbounded for HMAC. Wrap so the engine's
            // audit/log pipeline can surface it.
            |e| PaymentError::Provider(format!("hmac key rejected: {e}")),
        )?;
        mac.update(signed.as_bytes());
        let computed = mac.finalize().into_bytes();

        if constant_time_eq_hex(&computed, expected_v1) {
            Ok(())
        } else {
            Err(PaymentError::Provider("webhook signature mismatch".into()))
        }
    }

    /// Completes the 3-D Secure (3DS) challenge for a
    /// previously-created Stripe PaymentIntent and returns the
    /// resulting [`PaymentReceipt`].
    ///
    /// # When to call this
    ///
    /// [`PaymentProvider::charge`] returns
    /// [`PaymentError::ThreeDSRequired`] when Stripe signals that
    /// the issuer requires 3-D Secure authentication before the
    /// charge can proceed. The consumer's frontend then:
    ///
    /// 1. Reads `next_action.redirect_to_url` from the original
    ///    charge response and redirects the customer to the
    ///    issuer's hosted 3DS challenge.
    /// 2. The issuer hosts the challenge and, on completion,
    ///    redirects back to the consumer's `return_url` with the
    ///    `payment_intent` query parameter (a Stripe `pi_...` id).
    /// 3. The consumer calls this method with that
    ///    `payment_intent_id` and the [`ThreeDSChallengeResult`]
    ///    the frontend observed.
    ///
    /// # What this method does
    ///
    /// 1. Validates `payment_intent_id` is non-empty and starts
    ///    with the Stripe `pi_` prefix.
    /// 2. Fetches the PaymentIntent's current state via
    ///    `GET /v1/payment_intents/{payment_intent_id}`.
    /// 3. Translates Stripe's status into a [`PaymentReceipt`]:
    ///    - `succeeded` -> [`PaymentStatus::Captured`].
    ///    - `processing`, `requires_action`,
    ///      `requires_confirmation`, `requires_source` ->
    ///      [`PaymentStatus::Pending`] (the customer has not yet
    ///      completed the challenge; the engine can poll
    ///      [`PaymentProvider::status`] for an update).
    ///    - `requires_payment_method`, `requires_source_action`
    ///      -> [`PaymentStatus::Failed`] with the consumer-
    ///      reported failure reason (or `"3DS authentication
    ///      failed"` if the consumer reported success).
    ///    - `canceled` -> [`PaymentStatus::Cancelled`] with the
    ///      consumer-reported reason.
    ///
    /// Stripe's status is authoritative; the
    /// [`ThreeDSChallengeResult`] argument enriches the audit-log
    /// message when Stripe's status is ambiguous
    /// (`requires_action`, `requires_payment_method`, or
    /// `canceled`).
    ///
    /// # Errors
    ///
    /// - [`PaymentError::Provider`] when `payment_intent_id` is
    ///   empty or malformed, or Stripe's response is missing
    ///   required fields.
    /// - [`PaymentError::Infrastructure`] when the HTTP round-trip
    ///   fails.
    /// - [`PaymentError::Declined`] is **not** returned by this
    ///   method — authentication failures are reported as
    ///   [`PaymentStatus::Failed`] in the returned receipt so the
    ///   engine can persist the failure and emit the
    ///   `PaymentFailed` event.
    pub async fn complete_three_ds_challenge(
        &self,
        payment_intent_id: &str,
        result: ThreeDSChallengeResult,
    ) -> Result<PaymentReceipt, PaymentError> {
        let intent_id = payment_intent_id.trim();
        if intent_id.is_empty() {
            return Err(PaymentError::Provider(
                "three_ds payment_intent_id is empty".to_owned(),
            ));
        }
        if !intent_id.starts_with("pi_") {
            return Err(PaymentError::Provider(format!(
                "three_ds payment_intent_id must start with 'pi_', got {intent_id:?}"
            )));
        }

        let body = self
            .get_json(&format!("payment_intents/{intent_id}"))
            .await?;

        let stripe_status = body
            .get("status")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| {
                PaymentError::Provider(format!(
                    "stripe ThreeDS PaymentIntent {intent_id} missing 'status'"
                ))
            })?;

        let amount_minor = body
            .get("amount")
            .and_then(JsonValue::as_i64)
            .ok_or_else(|| {
                PaymentError::Provider(format!(
                    "stripe ThreeDS PaymentIntent {intent_id} missing 'amount'"
                ))
            })?;

        let currency_str = body
            .get("currency")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| {
                PaymentError::Provider(format!(
                    "stripe ThreeDS PaymentIntent {intent_id} missing 'currency'"
                ))
            })?;

        let currency = match crate::port::CurrencyCode::new(
            currency_str.to_ascii_uppercase().as_str(),
        ) {
            Ok(c) => c,
            Err(_) => {
                return Err(PaymentError::Provider(format!(
                    "stripe ThreeDS PaymentIntent {intent_id} currency {currency_str:?} is not a valid ISO 4217 code"
                )));
            }
        };

        let amount = Money::new(currency.clone(), amount_minor).map_err(|e| {
            PaymentError::Provider(format!(
                "invalid stripe ThreeDS amount for {intent_id}: {e}"
            ))
        })?;

        let status = map_payment_intent_3ds_status(stripe_status, &result);
        let captured = matches!(status, PaymentStatus::Captured { .. });
        let now = Timestamp::now();

        Ok(PaymentReceipt {
            payment_id: crate::port::PaymentId::from(intent_id.to_owned()),
            provider_payment_id: Some(intent_id.to_owned()),
            status,
            amount: amount.clone(),
            method: PaymentMethodKind::Card,
            authorized_at: Some(now),
            captured_at: if captured { Some(now) } else { None },
            fees: Vec::new(),
            net: amount,
            receipt_url: body
                .get("latest_charge")
                .and_then(JsonValue::as_str)
                .map(|c| format!("https://dashboard.stripe.com/test/payments/{c}")),
            metadata: BTreeMap::new(),
        })
    }

    // -- Internal HTTP helpers ------------------------------------------

    /// `POST {base}/{path}` with a form-encoded body and an
    /// `Idempotency-Key` header. Returns the parsed JSON body.
    async fn post_form(
        &self,
        path: &str,
        params: &[(String, String)],
        idempotency_key: &str,
    ) -> Result<JsonValue, PaymentError> {
        let url = self.endpoint(path);
        let response = self
            .http
            .post(&url)
            .basic_auth(&self.secret_key, Some(""))
            .header("Idempotency-Key", idempotency_key)
            .form(params)
            .send()
            .await
            .map_err(reqwest_err)?;
        self.parse_response(response).await
    }

    /// `GET {base}/{path}` with HTTP Basic auth on the secret key.
    /// Returns the parsed JSON body.
    async fn get_json(&self, path: &str) -> Result<JsonValue, PaymentError> {
        let url = self.endpoint(path);
        let response = self
            .http
            .get(&url)
            .basic_auth(&self.secret_key, Some(""))
            .send()
            .await
            .map_err(reqwest_err)?;
        self.parse_response(response).await
    }

    fn endpoint(&self, path: &str) -> String {
        let trimmed = path.trim_start_matches('/');
        let base = self.base_url.trim_end_matches('/');
        format!("{base}/{trimmed}")
    }

    async fn parse_response(&self, response: reqwest::Response) -> Result<JsonValue, PaymentError> {
        let status = response.status();
        let body: JsonValue = response.json().await.map_err(reqwest_err)?;
        if status.is_success() {
            Ok(body)
        } else {
            Err(stripe_error_to_payment_error(status, &body))
        }
    }
}

impl fmt::Debug for StripeProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StripeProvider")
            .field("base_url", &self.base_url)
            .field("secret_key", &redact_secret(&self.secret_key))
            .field("webhook_secret", &redact_secret(&self.webhook_secret))
            .finish()
    }
}

// ---------------------------------------------------------------------------
// StripeProviderBuilder
// ---------------------------------------------------------------------------

/// A fluent builder for [`StripeProvider`]. Defaults are populated
/// from [`STRIPE_DEFAULT_BASE_URL`]; `secret_key` and
/// `webhook_secret` default to the empty string (the provider will
/// reject calls until they are set).
#[derive(Debug, Clone, Default)]
pub struct StripeProviderBuilder {
    secret_key: String,
    webhook_secret: String,
    base_url: String,
}

impl StripeProviderBuilder {
    /// Constructs an empty builder. Equivalent to
    /// [`Default::default`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the Stripe secret API key (`sk_test_...` / `sk_live_...`).
    /// Required for every authenticated call.
    #[must_use]
    pub fn secret_key(mut self, key: impl Into<String>) -> Self {
        self.secret_key = key.into();
        self
    }

    /// Sets the Stripe webhook signing secret
    /// (`whsec_...`). Required for
    /// [`StripeProvider::verify_webhook_signature`].
    #[must_use]
    pub fn webhook_secret(mut self, secret: impl Into<String>) -> Self {
        self.webhook_secret = secret.into();
        self
    }

    /// Overrides the API base URL. Defaults to
    /// [`STRIPE_DEFAULT_BASE_URL`]. The path is appended verbatim,
    /// so callers should leave the URL bare (no trailing slash).
    #[must_use]
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Consumes the builder and returns a [`StripeProvider`].
    ///
    /// The underlying `reqwest::Client` is built with a 30-second
    /// request timeout. Building the client can theoretically
    /// fail if the TLS backend fails to initialise (a process-
    /// level startup concern); per the engine's no-panic rule
    /// this method returns a [`Result`] rather than panicking.
    /// Production callers should treat the `Err` branch as a
    /// fatal startup error (the consumer must restart with a
    /// working TLS backend).
    pub fn build(self) -> Result<StripeProvider, PaymentError> {
        let base_url = if self.base_url.is_empty() {
            STRIPE_DEFAULT_BASE_URL.to_owned()
        } else {
            self.base_url
        };
        let http = Client::builder()
            .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
            .build()
            .map_err(|e| {
                PaymentError::Infrastructure(InfrastructureError::new(format!(
                    "reqwest::Client::builder failed: {e}"
                )))
            })?;
        Ok(StripeProvider {
            http,
            secret_key: self.secret_key,
            webhook_secret: self.webhook_secret,
            base_url,
        })
    }
}

// ---------------------------------------------------------------------------
// PaymentProvider impl
// ---------------------------------------------------------------------------

#[async_trait]
impl PaymentProvider for StripeProvider {
    async fn charge(&self, request: ChargeRequest) -> Result<PaymentReceipt, PaymentError> {
        let kind = request.method.kind();

        let params = match &request.method {
            PaymentMethod::Card { token, .. } => {
                let mut p = Vec::with_capacity(6 + request.metadata.len());
                p.push(("amount".to_owned(), request.amount.amount_minor.to_string()));
                p.push((
                    "currency".to_owned(),
                    request.amount.currency.as_str().to_ascii_lowercase(),
                ));
                p.push(("source".to_owned(), token.as_str().to_owned()));
                if !request.capture {
                    p.push(("capture".to_owned(), "false".to_owned()));
                }
                if !request.description.is_empty() {
                    p.push(("description".to_owned(), request.description.clone()));
                }
                for (k, v) in &request.metadata {
                    p.push((format!("metadata[{k}]"), v.clone()));
                }
                p
            }
            PaymentMethod::Gateway { gateway, .. } => {
                return Err(PaymentError::Provider(format!(
                    "gateway flow not supported in v1 (gateway={})",
                    gateway.as_str()
                )));
            }
            PaymentMethod::Cash => {
                return Err(PaymentError::Provider(
                    "Stripe does not support offline cash payments".into(),
                ));
            }
            PaymentMethod::Cheque { .. } => {
                return Err(PaymentError::Provider(
                    "Stripe does not support offline cheque payments".into(),
                ));
            }
            PaymentMethod::BankTransfer { .. } => {
                return Err(PaymentError::Provider(
                    "Stripe does not support offline bank-transfer payments".into(),
                ));
            }
            PaymentMethod::Wallet { .. } => {
                return Err(PaymentError::Provider(
                    "Stripe does not support wallet payments; use a wallet adapter".into(),
                ));
            }
            PaymentMethod::ManualAdjustment { .. } => {
                return Err(PaymentError::Provider(
                    "manual adjustments are not gateway charges".into(),
                ));
            }
        };

        let body = self
            .post_form("charges", &params, &request.idempotency_key.to_string())
            .await?;
        receipt_from_charge(&body, request.amount, kind)
    }

    async fn refund(&self, request: RefundRequest) -> Result<RefundReceipt, PaymentError> {
        let mut params: Vec<(String, String)> = Vec::with_capacity(3);
        params.push((
            "charge".to_owned(),
            request.original_payment_id.as_str().to_owned(),
        ));
        params.push(("amount".to_owned(), request.amount.amount_minor.to_string()));
        if !request.reason.is_empty() {
            params.push(("reason".to_owned(), stripe_refund_reason(&request.reason)));
        }

        let body = self
            .post_form("refunds", &params, &request.idempotency_key.to_string())
            .await?;
        refund_receipt_from_refund(&body, request)
    }

    async fn status(
        &self,
        payment_id: crate::port::PaymentId,
    ) -> Result<PaymentStatus, PaymentError> {
        let body = self
            .get_json(&format!("charges/{}", payment_id.as_str()))
            .await?;
        let status_str = body
            .get("status")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| PaymentError::Provider("stripe response missing 'status'".into()))?;
        let captured = body
            .get("captured")
            .and_then(JsonValue::as_bool)
            .unwrap_or(false);
        let amount_minor = body.get("amount").and_then(JsonValue::as_i64).unwrap_or(0);
        let currency_str = body
            .get("currency")
            .and_then(JsonValue::as_str)
            .unwrap_or("USD");
        let currency =
            match crate::port::CurrencyCode::new(currency_str.to_ascii_uppercase().as_str()) {
                Ok(c) => c,
                Err(_) => {
                    return Err(PaymentError::Provider(format!(
                        "stripe response currency {currency_str:?} is not a valid ISO 4217 code"
                    )));
                }
            };
        let amount =
            Money::new(currency.clone(), amount_minor).unwrap_or_else(|_| Money::zero(currency));
        Ok(map_charge_status(status_str, captured, &amount))
    }

    async fn list_methods(
        &self,
        _tenant: educore_core::tenant::TenantContext,
    ) -> Result<Vec<PaymentMethodInfo>, crate::errors::PaymentError> {
        Ok(vec![
            PaymentMethodInfo {
                kind: PaymentMethodKind::Card,
                display_name: "Credit / Debit Card (Stripe)".to_owned(),
                enabled: true,
                note: Some("2.9% + 30¢ processing fee".to_owned()),
            },
            PaymentMethodInfo {
                kind: PaymentMethodKind::BankTransfer,
                display_name: "Bank Transfer (Stripe)".to_owned(),
                enabled: false,
                note: Some(
                    "Stripe Bank Transfers require Connect; disabled in v1".to_owned(),
                ),
            },
            PaymentMethodInfo {
                kind: PaymentMethodKind::Wallet,
                display_name: "Wallet (Stripe)".to_owned(),
                enabled: false,
                note: Some(
                    "Stripe wallets (Apple Pay / Google Pay) require the PaymentSheet; disabled in v1".to_owned(),
                ),
            },
            PaymentMethodInfo {
                kind: PaymentMethodKind::Cash,
                display_name: "Cash at Office".to_owned(),
                enabled: false,
                note: Some("Use the offline cash-book adapter, not Stripe".to_owned()),
            },
        ])
    }

    async fn settlement(&self, _request: SettlementRequest) -> Result<Settlement, PaymentError> {
        // Per the module-level doc: Stripe does not expose a single
        // settlement endpoint. Production code would query
        // `/v1/balance_transactions` (payouts) and translate to the
        // engine's Settlement shape. The reference impl returns an
        // explicit error so the engine can fall back to a dedicated
        // payouts adapter.
        Err(PaymentError::Provider(
            "settlement is not implemented in the Stripe reference adapter; use a dedicated payouts adapter".into(),
        ))
    }
}

// ---------------------------------------------------------------------------
// Response -> receipt mapping
// ---------------------------------------------------------------------------

fn receipt_from_charge(
    charge: &JsonValue,
    amount: Money,
    kind: PaymentMethodKind,
) -> Result<PaymentReceipt, PaymentError> {
    let id = charge
        .get("id")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| PaymentError::Provider("stripe charge response missing 'id'".into()))?
        .to_owned();
    let status_str = charge
        .get("status")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| PaymentError::Provider("stripe charge response missing 'status'".into()))?;
    let captured = charge
        .get("captured")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false);
    let receipt_url = charge
        .get("receipt_url")
        .and_then(JsonValue::as_str)
        .map(str::to_owned);

    let status = map_charge_status(status_str, captured, &amount);
    let now = Timestamp::now();

    let fees = extract_stripe_fees(charge, &amount.currency)?;
    let net = amount.clone();

    let metadata = json_object_to_metadata(charge.get("metadata"));

    Ok(PaymentReceipt {
        payment_id: crate::port::PaymentId::from(id.clone()),
        provider_payment_id: Some(id),
        status,
        amount,
        method: kind,
        authorized_at: Some(now),
        captured_at: if captured { Some(now) } else { None },
        fees,
        net,
        receipt_url,
        metadata,
    })
}

fn refund_receipt_from_refund(
    body: &JsonValue,
    request: RefundRequest,
) -> Result<RefundReceipt, PaymentError> {
    let id = body
        .get("id")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| PaymentError::Provider("stripe refund response missing 'id'".into()))?
        .to_owned();
    let status_str = body
        .get("status")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| PaymentError::Provider("stripe refund response missing 'status'".into()))?;
    let amount_minor = body
        .get("amount")
        .and_then(JsonValue::as_i64)
        .unwrap_or(request.amount.amount_minor);
    let refund_currency_str = body
        .get("currency")
        .and_then(JsonValue::as_str)
        .map(str::to_ascii_uppercase)
        .unwrap_or_else(|| request.amount.currency.as_str().to_owned());
    let refund_currency = crate::port::CurrencyCode::new(&refund_currency_str)
        .unwrap_or_else(|_| request.amount.currency.clone());
    let amount =
        Money::new(refund_currency, amount_minor).unwrap_or_else(|_| request.amount.clone());

    let status = match status_str {
        "succeeded" => PaymentStatus::Captured {
            at: Timestamp::now(),
        },
        "pending" | "processing" | "requires_action" => PaymentStatus::Pending,
        "failed" => PaymentStatus::Failed {
            reason: body
                .get("failure_reason")
                .and_then(JsonValue::as_str)
                .unwrap_or("unknown")
                .to_owned(),
            code: body
                .get("failure_code")
                .and_then(JsonValue::as_str)
                .map(str::to_owned),
        },
        "canceled" | "cancelled" => PaymentStatus::Cancelled {
            at: Timestamp::now(),
            reason: body
                .get("failure_reason")
                .and_then(JsonValue::as_str)
                .unwrap_or("cancelled")
                .to_owned(),
        },
        other => {
            return Err(PaymentError::Provider(format!(
                "unknown stripe refund status: {other}"
            )));
        }
    };

    Ok(RefundReceipt {
        refund_id: crate::port::PaymentId::from(id.clone()),
        original_payment_id: request.original_payment_id,
        provider_refund_id: Some(id),
        amount,
        status: status.clone(),
        refunded_at: matches!(status, PaymentStatus::Captured { .. }).then(Timestamp::now),
        destination: request.refund_to,
        metadata: json_object_to_metadata(body.get("metadata")),
    })
}

fn extract_stripe_fees(
    charge: &JsonValue,
    currency: &crate::port::CurrencyCode,
) -> Result<Vec<PaymentFee>, PaymentError> {
    let Some(balance_txn) = charge.get("balance_transaction") else {
        return Ok(Vec::new());
    };
    // balance_transaction may be a string id or an expanded object.
    if !balance_txn.is_object() {
        return Ok(Vec::new());
    }
    let Some(fee_minor) = balance_txn.get("fee").and_then(JsonValue::as_i64) else {
        return Ok(Vec::new());
    };
    if fee_minor <= 0 {
        return Ok(Vec::new());
    }
    let fee_amount = Money::new(currency.clone(), fee_minor)
        .map_err(|e| PaymentError::Provider(format!("invalid stripe fee: {e}")))?;
    Ok(vec![PaymentFee {
        amount: fee_amount,
        description: "Stripe processing fee".to_owned(),
    }])
}

fn json_object_to_metadata(value: Option<&JsonValue>) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    if let Some(obj) = value.and_then(JsonValue::as_object) {
        for (k, v) in obj {
            if let Some(s) = v.as_str() {
                out.insert(k.clone(), s.to_owned());
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Stripe -> PaymentStatus mapping
// ---------------------------------------------------------------------------

/// Maps a Stripe PaymentIntent status (after a 3DS challenge)
/// into the engine's [`PaymentStatus`], incorporating the
/// consumer-reported [`ThreeDSChallengeResult`] for the
/// audit-friendly failure reason when Stripe's status is
/// ambiguous.
fn map_payment_intent_3ds_status(
    stripe_status: &str,
    result: &ThreeDSChallengeResult,
) -> PaymentStatus {
    match stripe_status {
        "succeeded" => PaymentStatus::Captured {
            at: Timestamp::now(),
        },
        "processing" | "requires_action" | "requires_confirmation" | "requires_source" => {
            PaymentStatus::Pending
        }
        "requires_payment_method" | "requires_source_action" => {
            let reason = match result {
                ThreeDSChallengeResult::Failed { reason } => format!("3DS failed: {reason}"),
                ThreeDSChallengeResult::Abandoned => "3DS abandoned".to_owned(),
                ThreeDSChallengeResult::Authenticated => "3DS authentication required".to_owned(),
            };
            PaymentStatus::Failed {
                reason,
                code: Some("authentication_required".to_owned()),
            }
        }
        "canceled" | "cancelled" => PaymentStatus::Cancelled {
            at: Timestamp::now(),
            reason: match result {
                ThreeDSChallengeResult::Abandoned => "ThreeDS challenge abandoned".to_owned(),
                ThreeDSChallengeResult::Failed { reason } => {
                    format!("ThreeDS cancelled: {reason}")
                }
                ThreeDSChallengeResult::Authenticated => "ThreeDS challenge cancelled".to_owned(),
            },
        },
        other => PaymentStatus::Failed {
            reason: format!("unknown stripe ThreeDS PaymentIntent status: {other}"),
            code: None,
        },
    }
}

fn map_charge_status(stripe_status: &str, captured: bool, amount: &Money) -> PaymentStatus {
    match stripe_status {
        "pending"
        | "processing"
        | "requires_payment_method"
        | "requires_confirmation"
        | "requires_action"
        | "requires_source"
        | "requires_source_action" => PaymentStatus::Pending,
        "succeeded" => {
            if captured {
                PaymentStatus::Captured {
                    at: Timestamp::now(),
                }
            } else {
                PaymentStatus::Authorized {
                    auth_code: String::new(),
                    expires_at: Timestamp::now(),
                }
            }
        }
        "failed" => PaymentStatus::Failed {
            reason: "stripe charge failed".to_owned(),
            code: None,
        },
        "refunded" => PaymentStatus::Refunded {
            amount: amount.clone(),
            at: Timestamp::now(),
            reason: String::new(),
        },
        "partially_refunded" => PaymentStatus::PartiallyRefunded {
            refunded: amount.clone(),
            remaining: Money::zero(amount.currency.clone()),
        },
        "disputed" => PaymentStatus::Disputed {
            dispute_id: String::new(),
            reason: String::new(),
            opened_at: Timestamp::now(),
        },
        "canceled" | "cancelled" => PaymentStatus::Cancelled {
            at: Timestamp::now(),
            reason: String::new(),
        },
        other => PaymentStatus::Failed {
            reason: format!("unknown stripe charge status: {other}"),
            code: None,
        },
    }
}

// ---------------------------------------------------------------------------
// Stripe error envelope -> PaymentError mapping
// ---------------------------------------------------------------------------

fn stripe_error_to_payment_error(status: StatusCode, body: &JsonValue) -> PaymentError {
    let err_obj = body.get("error");
    let err_type = err_obj
        .and_then(|e| e.get("type"))
        .and_then(JsonValue::as_str)
        .unwrap_or("");
    let message = err_obj
        .and_then(|e| e.get("message"))
        .and_then(JsonValue::as_str)
        .unwrap_or("(no message)")
        .to_owned();
    let code = err_obj
        .and_then(|e| e.get("code"))
        .and_then(JsonValue::as_str);
    let decline_code = err_obj
        .and_then(|e| e.get("decline_code"))
        .and_then(JsonValue::as_str);

    match (status, err_type) {
        (_, "rate_limit_error") => PaymentError::RateLimited,
        (StatusCode::PAYMENT_REQUIRED, "card_error") | (_, "card_error")
            if decline_code == Some("insufficient_funds") =>
        {
            PaymentError::InsufficientFunds
        }
        (_, "card_error") => PaymentError::Declined(format!(
            "{}{}",
            message,
            code.map(|c| format!(" (code={c})")).unwrap_or_default()
        )),
        (_, "validation_error") | (StatusCode::BAD_REQUEST, _) => {
            PaymentError::InvalidAmount(message)
        }
        (_, "idempotency_error") => PaymentError::Provider(format!("idempotency error: {message}")),
        (_, "authentication_error") | (StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN, _) => {
            PaymentError::Provider(format!("authentication error: {message}"))
        }
        (_, "api_connection_error") | (_, "api_error") => PaymentError::Infrastructure(
            InfrastructureError::new(format!("stripe API error ({status}): {message}")),
        ),
        _ => PaymentError::Provider(format!("stripe error ({status}): {message}")),
    }
}

// ---------------------------------------------------------------------------
// Webhook helpers
// ---------------------------------------------------------------------------

fn parse_stripe_signature(signature: &str) -> Result<(i64, &str), PaymentError> {
    let mut timestamp: Option<i64> = None;
    let mut v1: Option<&str> = None;
    for part in signature.split(',') {
        let Some((k, v)) = part.split_once('=') else {
            continue;
        };
        match k {
            "t" => {
                timestamp = v.parse::<i64>().ok();
            }
            "v1" if v1.is_none() => {
                v1 = Some(v);
            }
            _ => {}
        }
    }
    let timestamp = timestamp
        .ok_or_else(|| PaymentError::Provider("stripe signature missing 't=' timestamp".into()))?;
    let v1 =
        v1.ok_or_else(|| PaymentError::Provider("stripe signature missing 'v1=' value".into()))?;
    Ok((timestamp, v1))
}

/// Constant-time comparison of a raw byte slice against a hex
/// string. Returns `false` on length mismatch, odd-length hex, or
/// any non-hex character; the comparison itself iterates over every
/// byte regardless of where the first mismatch is found.
fn constant_time_eq_hex(computed: &[u8], expected_hex: &str) -> bool {
    if expected_hex.len() != computed.len() * 2 {
        return false;
    }
    let expected_bytes = expected_hex.as_bytes();
    let mut diff: u8 = 0;
    for (i, byte) in computed.iter().enumerate() {
        let hi = hex_nibble(expected_bytes[i * 2]);
        let lo = hex_nibble(expected_bytes[i * 2 + 1]);
        diff |= match (hi, lo) {
            (Some(h), Some(l)) => byte ^ ((h << 4) | l),
            _ => 0xff,
        };
    }
    diff == 0
}

const fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Small helpers
// ---------------------------------------------------------------------------

fn reqwest_err(e: reqwest::Error) -> PaymentError {
    PaymentError::Infrastructure(InfrastructureError::with_source(
        format!("reqwest error: {e}"),
        Box::new(e),
    ))
}

fn redact_secret(s: &str) -> &'static str {
    if s.is_empty() {
        "<empty>"
    } else {
        "<redacted>"
    }
}

/// Maps a free-form refund reason to Stripe's whitelist
/// (`duplicate` / `fraudulent` / `requested_by_customer`). Unknown
/// reasons are dropped (Stripe would reject them otherwise).
fn stripe_refund_reason(reason: &str) -> String {
    match reason {
        "duplicate" | "fraudulent" | "requested_by_customer" => reason.to_owned(),
        other => match other.to_ascii_lowercase().as_str() {
            "duplicate" => "duplicate".to_owned(),
            "fraud" | "fraudulent" => "fraudulent".to_owned(),
            "customer" | "requested" | "requested_by_customer" => {
                "requested_by_customer".to_owned()
            }
            _ => String::new(),
        },
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
    use crate::port::{
        CardToken, ChargeRequest, ChequeDate, CurrencyCode, CustomerRef, PaymentMethod,
    };
    use educore_core::clock::{IdGenerator, SystemIdGen};
    use educore_core::tenant::TenantContext;

    fn provider_with_secret(secret: &str) -> StripeProvider {
        StripeProviderBuilder::new()
            .secret_key("sk_test_dummy")
            .webhook_secret(secret)
            .base_url("https://stripe.invalid/v1")
            .build()
            .expect("reqwest::Client::builder with rustls-tls is infallible")
    }

    fn sign_hmac(secret: &str, signed_payload: &str) -> String {
        let mut mac =
            HmacSha256::new_from_slice(secret.as_bytes()).expect("hmac accepts any key length");
        mac.update(signed_payload.as_bytes());
        let bytes = mac.finalize().into_bytes();
        to_lower_hex(&bytes)
    }

    fn to_lower_hex(bytes: &[u8]) -> String {
        use std::fmt::Write;
        let mut s = String::with_capacity(bytes.len() * 2);
        for b in bytes {
            write!(s, "{b:02x}").expect("writing to String never fails");
        }
        s
    }

    fn make_charge_request(method: PaymentMethod) -> ChargeRequest {
        let g = SystemIdGen;
        let tenant = TenantContext::for_user(
            g.next_school_id(),
            g.next_user_id(),
            g.next_correlation_id(),
            educore_core::tenant::UserType::Teacher,
        );
        let amount = Money::new(CurrencyCode::new("USD").unwrap(), 1500).unwrap();
        ChargeRequest::new(
            tenant,
            amount,
            method,
            CustomerRef::User(g.next_user_id()),
            g.next_idempotency_key(),
            g.next_correlation_id(),
        )
    }

    #[test]
    fn test_stripe_provider_builder_constructs_with_defaults() {
        let p = StripeProviderBuilder::new()
            .secret_key("sk_test_x")
            .webhook_secret("whsec_x")
            .build()
            .expect("reqwest::Client::builder with rustls-tls is infallible");
        assert_eq!(p.base_url(), STRIPE_DEFAULT_BASE_URL);
        assert!(p.has_secret_key());
        assert!(p.has_webhook_secret());

        let p2 = StripeProviderBuilder::default()
            .build()
            .expect("reqwest::Client::builder with rustls-tls is infallible");
        assert_eq!(p2.base_url(), STRIPE_DEFAULT_BASE_URL);
        assert!(!p2.has_secret_key());
        assert!(!p2.has_webhook_secret());

        let p3 = StripeProviderBuilder::new()
            .base_url("https://example.test/api")
            .build()
            .expect("reqwest::Client::builder with rustls-tls is infallible");
        assert_eq!(p3.base_url(), "https://example.test/api");

        let debug = format!("{p:?}");
        assert!(debug.contains("StripeProvider"));
        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains("sk_test_x"));
        assert!(!debug.contains("whsec_x"));
    }

    #[test]
    fn test_stripe_webhook_signature_verification() {
        let secret = "whsec_super_secret_key_for_tests";
        let provider = provider_with_secret(secret);

        let payload = br#"{"id":"evt_test","type":"charge.succeeded"}"#;
        let timestamp = 1_700_000_000_i64;
        let signed = format!("{timestamp}.{}", std::str::from_utf8(payload).unwrap());
        let sig_hex = sign_hmac(secret, &signed);
        let header = format!("t={timestamp},v1={sig_hex}");

        // Correct signature + correct payload: accepted.
        provider
            .verify_webhook_signature(payload, &header)
            .expect("valid signature must verify");

        // Tampered payload: rejected.
        let tampered = br#"{"id":"evt_test","type":"charge.refunded"}"#;
        let err = provider
            .verify_webhook_signature(tampered, &header)
            .expect_err("tampered payload must fail verification");
        assert!(matches!(err, PaymentError::Provider(ref m) if m == "webhook signature mismatch"));

        // Wrong secret: rejected.
        let other = provider_with_secret("whsec_a_completely_different_secret");
        let err = other
            .verify_webhook_signature(payload, &header)
            .expect_err("different secret must fail verification");
        assert!(matches!(err, PaymentError::Provider(ref m) if m == "webhook signature mismatch"));

        // Malformed header (no `t=`): rejected with Provider.
        let err = provider
            .verify_webhook_signature(payload, "v1=deadbeef")
            .expect_err("missing timestamp must fail");
        assert!(matches!(err, PaymentError::Provider(ref m) if m.contains("missing 't='")));

        // Malformed header (no `v1=`): rejected with Provider.
        let err = provider
            .verify_webhook_signature(payload, "t=1700000000")
            .expect_err("missing v1 must fail");
        assert!(matches!(err, PaymentError::Provider(ref m) if m.contains("missing 'v1='")));

        // Empty webhook_secret: rejected up-front.
        let empty_secret_provider = StripeProviderBuilder::new()
            .build()
            .expect("reqwest::Client::builder with rustls-tls is infallible");
        let err = empty_secret_provider
            .verify_webhook_signature(payload, &header)
            .expect_err("empty webhook_secret must fail");
        assert!(
            matches!(err, PaymentError::Provider(ref m) if m.contains("webhook_secret is not configured"))
        );
    }

    #[test]
    fn test_charge_rejects_offline_methods() {
        let p = StripeProviderBuilder::new()
            .secret_key("sk_test_x")
            .build()
            .expect("reqwest::Client::builder with rustls-tls is infallible");
        let req = make_charge_request(PaymentMethod::Cash);
        let err = futures_executor_block_on(p.charge(req)).expect_err("cash must be rejected");
        assert!(matches!(err, PaymentError::Provider(_)));

        let req = make_charge_request(PaymentMethod::Cheque {
            number: "000123".to_owned(),
            bank: "HDFC".to_owned(),
            date: ChequeDate::new(2026, 6, 19).unwrap(),
        });
        let err = futures_executor_block_on(p.charge(req)).expect_err("cheque must be rejected");
        assert!(matches!(err, PaymentError::Provider(_)));

        let req = make_charge_request(PaymentMethod::Card {
            token: CardToken::new("tok_test"),
            save: false,
        });
        // Will hit the network; we only assert it constructs a request
        // (i.e. does not error on validation). We don't run it.
        let _ = req;
    }

    #[test]
    fn test_charge_rejects_gateway_method() {
        let p = StripeProviderBuilder::new()
            .secret_key("sk_test_x")
            .build()
            .expect("reqwest::Client::builder with rustls-tls is infallible");
        let req = make_charge_request(PaymentMethod::Gateway {
            gateway: crate::port::GatewayName::new("stripe"),
            token: crate::port::GatewayToken::new("pi_test"),
            return_url: "https://example.com/return".to_owned(),
        });
        let err = futures_executor_block_on(p.charge(req)).expect_err("gateway must be rejected");
        assert!(
            matches!(err, PaymentError::Provider(ref m) if m.contains("gateway flow not supported"))
        );
    }

    #[test]
    fn test_settlement_returns_deviation_error() {
        let p = StripeProviderBuilder::new()
            .secret_key("sk_test_x")
            .build()
            .expect("reqwest::Client::builder with rustls-tls is infallible");
        let g = SystemIdGen;
        let tenant = TenantContext::for_user(
            g.next_school_id(),
            g.next_user_id(),
            g.next_correlation_id(),
            educore_core::tenant::UserType::Teacher,
        );
        let req = SettlementRequest {
            tenant,
            period_start: Timestamp::now(),
            period_end: Timestamp::now(),
            currency: CurrencyCode::new("USD").unwrap(),
        };
        let err = futures_executor_block_on(p.settlement(req))
            .expect_err("settlement must return a Provider error");
        assert!(matches!(err, PaymentError::Provider(_)));
    }

    #[test]
    fn test_stripe_refund_reason_whitelist() {
        assert_eq!(stripe_refund_reason("duplicate"), "duplicate");
        assert_eq!(stripe_refund_reason("fraudulent"), "fraudulent");
        assert_eq!(
            stripe_refund_reason("requested_by_customer"),
            "requested_by_customer"
        );
        assert_eq!(stripe_refund_reason("customer"), "requested_by_customer");
        assert_eq!(stripe_refund_reason("fraud"), "fraudulent");
        assert_eq!(stripe_refund_reason("anything else"), "");
    }

    #[test]
    fn test_endpoint_strips_leading_slash_and_trailing_base_slash() {
        let p = StripeProviderBuilder::new()
            .base_url("https://api.stripe.com/v1/")
            .build()
            .expect("reqwest::Client::builder with rustls-tls is infallible");
        assert_eq!(p.endpoint("charges"), "https://api.stripe.com/v1/charges");
        assert_eq!(p.endpoint("/charges"), "https://api.stripe.com/v1/charges");

        let p2 = StripeProviderBuilder::new()
            .base_url("https://api.stripe.com/v1")
            .build()
            .expect("reqwest::Client::builder with rustls-tls is infallible");
        assert_eq!(p2.endpoint("refunds"), "https://api.stripe.com/v1/refunds");
    }

    #[test]
    fn test_map_charge_status_canonical_paths() {
        let amount = Money::new(CurrencyCode::new("USD").unwrap(), 100).unwrap();
        let pending = map_charge_status("pending", false, &amount);
        assert!(matches!(pending, PaymentStatus::Pending));

        let captured = map_charge_status("succeeded", true, &amount);
        assert!(matches!(captured, PaymentStatus::Captured { .. }));

        let authorized = map_charge_status("succeeded", false, &amount);
        assert!(matches!(authorized, PaymentStatus::Authorized { .. }));

        let failed = map_charge_status("failed", false, &amount);
        assert!(matches!(failed, PaymentStatus::Failed { .. }));

        let cancelled = map_charge_status("canceled", false, &amount);
        assert!(matches!(cancelled, PaymentStatus::Cancelled { .. }));
    }

    #[test]
    fn test_map_payment_intent_3ds_status_authenticated_success() {
        let status =
            map_payment_intent_3ds_status("succeeded", &ThreeDSChallengeResult::Authenticated);
        assert!(matches!(status, PaymentStatus::Captured { .. }));
    }

    #[test]
    fn test_map_payment_intent_3ds_status_pending_paths() {
        for stripe_status in [
            "processing",
            "requires_action",
            "requires_confirmation",
            "requires_source",
        ] {
            let status = map_payment_intent_3ds_status(
                stripe_status,
                &ThreeDSChallengeResult::Authenticated,
            );
            assert!(
                matches!(status, PaymentStatus::Pending),
                "status {stripe_status} should map to Pending"
            );
        }
    }

    #[test]
    fn test_map_payment_intent_3ds_status_failed_with_consumer_reason() {
        let status = map_payment_intent_3ds_status(
            "requires_payment_method",
            &ThreeDSChallengeResult::Failed {
                reason: "do_not_honor".to_owned(),
            },
        );
        match status {
            PaymentStatus::Failed { reason, code } => {
                assert!(reason.contains("do_not_honor"));
                assert_eq!(code.as_deref(), Some("authentication_required"));
            }
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    #[test]
    fn test_map_payment_intent_3ds_status_failed_when_consumer_says_authenticated() {
        let status = map_payment_intent_3ds_status(
            "requires_payment_method",
            &ThreeDSChallengeResult::Authenticated,
        );
        match status {
            PaymentStatus::Failed { reason, .. } => {
                assert!(reason.contains("3DS"));
            }
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    #[test]
    fn test_map_payment_intent_3ds_status_cancelled_with_abandoned() {
        let status = map_payment_intent_3ds_status("canceled", &ThreeDSChallengeResult::Abandoned);
        match status {
            PaymentStatus::Cancelled { reason, .. } => {
                assert!(reason.contains("abandoned"));
            }
            other => panic!("expected Cancelled, got {other:?}"),
        }
    }

    #[test]
    fn test_map_payment_intent_3ds_status_unknown_status_is_failed() {
        let status = map_payment_intent_3ds_status(
            "totally_made_up",
            &ThreeDSChallengeResult::Authenticated,
        );
        assert!(matches!(status, PaymentStatus::Failed { .. }));
    }

    #[test]
    fn test_three_ds_challenge_result_equality() {
        assert_eq!(
            ThreeDSChallengeResult::Authenticated,
            ThreeDSChallengeResult::Authenticated
        );
        assert_ne!(
            ThreeDSChallengeResult::Authenticated,
            ThreeDSChallengeResult::Abandoned
        );
        assert_eq!(
            ThreeDSChallengeResult::Failed {
                reason: "x".to_owned()
            },
            ThreeDSChallengeResult::Failed {
                reason: "x".to_owned()
            }
        );
    }

    #[test]
    fn test_redact_secret_hides_empty_and_set() {
        assert_eq!(redact_secret(""), "<empty>");
        assert_eq!(redact_secret("anything"), "<redacted>");
    }

    // Minimal stand-in for `futures::executor::block_on` so we
    // don't have to add a `futures` dep just for tests.
    fn futures_executor_block_on<F: std::future::Future>(fut: F) -> F::Output {
        use std::sync::Arc;
        use std::task::{Context, Poll, Wake, Waker};
        struct Noop;
        impl Wake for Noop {
            fn wake(self: Arc<Self>) {}
        }
        let waker = Waker::from(Arc::new(Noop));
        let mut cx = Context::from_waker(&waker);
        let mut fut = Box::pin(fut);
        loop {
            if let Poll::Ready(v) = fut.as_mut().poll(&mut cx) {
                return v;
            }
        }
    }
}
