## Wave 3 Notify Adapter Audit Report

**Scope:** `crates/adapters/notify/`, `docs/ports/notifications.md`,
`docs/handoff/PHASE-15-HANDOFF.md`, `Cargo.toml` (workspace
`reqwest` declaration).

**Total findings:** 74

---

### FINDING 1

- **id:** ADAPTER-NOT-001
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/notify/src/email.rs:142-189` (EmailProvider::send_bulk) and `crates/adapters/notify/src/sms.rs:358-392` (SmsProvider::send_bulk)
- **description:** Neither reference implementation honours the port-level batch boundary. The `EmailProvider::send_bulk` declares `const BULK_BATCH_SIZE: usize = 100;` at `email.rs:58` but the only reference to it (`email.rs:160-162`) is `if idx > 0 && idx % BULK_BATCH_SIZE == 0 { let _ = idx / BULK_BATCH_SIZE; }` — a no-op whose result is immediately discarded. Every recipient is dispatched via a separate `self.send(single).await` call inside one loop, with no SMTP/network batching. `SmsProvider::send_bulk` does chunk by `SMS_BULK_BATCH_SIZE` (sms.rs:375-389) but dispatches each row serially inside the chunk, so the gateway still receives N requests per chunk instead of one request carrying 100 recipients.
- **expected:** Per `docs/ports/notifications.md` § "Bulk Send": "The adapter batches them (per channel limits, e.g. 100 SMS per batch)." and § "Testing": "A test of bulk send with partial failure."
- **evidence:**
  ```rust
  // crates/adapters/notify/src/email.rs:58
  const BULK_BATCH_SIZE: usize = 100;
  // crates/adapters/notify/src/email.rs:159-186
  for (idx, row) in request.recipients.iter().enumerate() {
      if idx > 0 && idx % BULK_BATCH_SIZE == 0 {
          let _ = idx / BULK_BATCH_SIZE;
      }
      let single = SendNotification { ... };
      match self.send(single).await {
          Ok(r) => receipt.receipts.push(r),
          Err(e) => { ... }
      }
  }
  ```

---

### FINDING 2

- **id:** ADAPTER-NOT-002
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/notify/src/lib.rs:28-32` and `crates/adapters/notify/src/port.rs:930-986`
- **description:** Only two of the seven `Channel` variants defined in the port contract have reference implementations (`Email` → `EmailProvider`, `Sms` → `SmsProvider`). The port contract requires adapters for `Push`, `InApp`, `Chat` (WhatsApp/Telegram/Messenger/Signal), `Voice`, and `Webhook`. The handoff at `PHASE-15-HANDOFF.md:127-129` explicitly confirms "2 reference impls: EmailProvider + SmsProvider", and the crate's `Cargo.toml:8` description claims "Notification port, email, SMS, push, in-app, chat, voice, webhook adapters." while shipping 5 missing implementations. The handoff also wires up RBAC capabilities `NotifyPushSend`, `NotifyInApp`, `NotifyVoice`, `NotifyWebhook` (`PHASE-15-HANDOFF.md:138-141`) for which no provider code exists.
- **expected:** Per `docs/ports/notifications.md` § "Channel", every `Channel` variant is supported: Email, Sms, Push, InApp, Chat (with ChatProvider), Voice, Webhook.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/lib.rs:26-32
  pub mod sms;
  /// Email [`NotificationProvider`] reference
  /// implementation backed by SMTP via the `lettre` crate.
  pub mod email;
  ```
  No `pub mod push;`, `pub mod in_app;`, `pub mod chat;`, `pub mod voice;`, or `pub mod webhook;` files exist under `crates/adapters/notify/src/` (the only files are `email.rs`, `errors.rs`, `lib.rs`, `port.rs`, `services.rs`, `sms.rs`).

---

### FINDING 3

- **id:** ADAPTER-NOT-003
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/notify/src/email.rs:142-189` (EmailProvider::send_bulk)
- **description:** `EmailProvider::send_bulk` does not call any underlying batch API; it iterates the recipient list serially and re-enters `self.send(single).await` for every row. Each iteration re-builds a fresh `lettre::Message`, opens a fresh SMTP command/response cycle (or one connection per row depending on pool config), and re-renders the template per recipient. The port spec mandates batched dispatch (100/batch) for both cost (one transactional vs N) and latency reasons.
- **expected:** `docs/ports/notifications.md` § "Bulk Send": "The adapter batches them (per channel limits, e.g. 100 SMS per batch)."
- **evidence:**
  ```rust
  // crates/adapters/notify/src/email.rs:159-186
  for (idx, row) in request.recipients.iter().enumerate() {
      if idx > 0 && idx % BULK_BATCH_SIZE == 0 {
          let _ = idx / BULK_BATCH_SIZE;
      }
      let single = SendNotification { ... };
      match self.send(single).await { ... }
  }
  ```

---

### FINDING 4

- **id:** ADAPTER-NOT-004
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/notify/src/sms.rs:394-401` (SmsProvider::status) and `crates/adapters/notify/src/email.rs:191-193` (EmailProvider::status)
- **description:** Both reference implementations return `DeliveryStatus::Sent` unconditionally from `status()`, regardless of whether the notification has actually been delivered, bounced, opened, or clicked. The port contract specifies that `status` is used to "reconcile webhook status updates" (port.rs:1407-1411) and that the engine will reflect `Delivered`, `Opened`, `Clicked`, `Bounced`, `Failed`, `Rejected` states. A stub that returns `Sent` for every receipt means the engine can never observe a bounce, a click, or any provider-confirmed failure via the status API.
- **expected:** `docs/ports/notifications.md` § "DeliveryStatus": "The adapter updates the status as the provider reports it (via webhook). The engine polls or subscribes to status changes for reconciliation." and § "Testing": "A test of status updates (delivered, opened, clicked)."
- **evidence:**
  ```rust
  // crates/adapters/notify/src/sms.rs:394-401
  async fn status(&self, _receipt_id: NotificationReceiptId) -> Result<DeliveryStatus> {
      // Reference impl: status lookup is a stub. A production
      // adapter would call the gateway's status endpoint ...
      Ok(DeliveryStatus::Sent)
  }

  // crates/adapters/notify/src/email.rs:191-193
  async fn status(&self, _receipt_id: NotificationReceiptId) -> Result<DeliveryStatus> {
      Ok(DeliveryStatus::Sent)
  }
  ```

---

### FINDING 5

- **id:** ADAPTER-NOT-005
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/notify/src/email.rs:90-140` (EmailProvider::send)
- **description:** `EmailProvider::send` ignores `request.attachments`, `request.priority`, `request.scheduled_at`, `request.idempotency_key`, `request.correlation_id`, and never resolves the template body from the communication-domain template store. It uses a hardcoded `DEFAULT_TEMPLATE_BODY = "Hello {student_name}, this is a notification from Educore."` (line 67) and prepends only the template id as a `[Template: id]` prefix (line 324-329). The provider cannot render any real template, never sets `NotificationReceipt::cost`, never populates `NotificationReceipt::metadata`, and the `provider_message_id` it stores is `response.code().to_string()` (line 139) — `lettre`'s SMTP response code, not the provider's message id (e.g. SES `MessageId`, which arrives in `X-SES-Configuration-Set` headers or the server response).
- **expected:** Per `docs/ports/notifications.md` § "Templates": "The adapter resolves the template body, applies variables, and delivers." § "Cost Tracking": "`cost: Option<Money>` is set by the adapter (e.g. $0.0075 per SMS)." § "NotificationReceipt": `provider_message_id: Option<String>` is "The provider's message id (e.g. SES `MessageId`, Twilio `MessageSid`)."
- **evidence:**
  ```rust
  // crates/adapters/notify/src/email.rs:64-67
  const DEFAULT_TEMPLATE_BODY: &str =
      "Hello {student_name}, this is a notification from Educore.";
  // crates/adapters/notify/src/email.rs:320-330
  fn render_template_body(template: &TemplateRef,
      variables: &BTreeMap<String, TemplateValue>) -> String {
      let mut body = DEFAULT_TEMPLATE_BODY.to_owned();
      let prefix = match template {
          TemplateRef::Id(id) => format!("[Template: {}]\n\n", id.as_str()),
      };
      body = format!("{prefix}{body}");
      substitute_variables(&body, variables)
  }
  // crates/adapters/notify/src/email.rs:133-139
  Ok(NotificationReceipt::new(receipt_id, request.channel,
      DeliveryStatus::Sent, Timestamp::now())
      .with_provider_message_id(response.code().to_string()))
  ```

---

### FINDING 6

- **id:** ADAPTER-NOT-006
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/notify/src/email.rs:262-281` (EmailProviderBuilder::build)
- **description:** `EmailProviderBuilder::build` always calls `AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&relay_host)` (line 270). The module-level doc at lines 33-38 explicitly recommends `AsyncSmtpTransport::relay` and `AsyncSmtpTransport::starttls_relay` as the "recommended constructors" and `builder_dangerous` is the lowest-level API that "consumers can wire their own TLS configuration" — but the builder itself never wires any TLS. The only authentication configuration is `Credentials::new(user, String::new())` (line 273) which uses an empty password. The transport therefore connects to port 587 with **no TLS upgrade attempted** unless the consumer manually swaps the builder, which the public API does not expose.
- **expected:** AGENTS.md § "TLS/SSL Cross-Compilation" mandates `rustls`. The email provider must establish a STARTTLS or implicit TLS connection on every send; the `relay`/`starttls_relay` constructors in `lettre` enforce this by default.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/email.rs:30-38
  //! ## TLS
  //!
  //! The transport is built with the `tokio1-rustls-tls` feature, so
  //! all SMTP connections use `rustls` (ADR-015 forbids
  //! `native-tls`). The reference builder uses
  //! [`AsyncSmtpTransport::builder_dangerous`] so consumers can wire
  //! their own TLS configuration; the recommended constructors are
  //! [`AsyncSmtpTransport::relay`] and
  //! [`AsyncSmtpTransport::starttls_relay`].
  // crates/adapters/notify/src/email.rs:270-274
  let mut builder = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&relay_host);
  builder = builder.port(relay_port);
  if let Some(user) = self.credentials_user {
      builder = builder.credentials(Credentials::new(user, String::new()));
  }
  ```
  The builder **does not** call `starttls_relay` or `relay`; it uses `builder_dangerous`, the doc-recommended-but-unused path.

---

### FINDING 7

- **id:** ADAPTER-NOT-007
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/notify/src/port.rs:43-44` and `crates/adapters/notify/src/port.rs:1422-1423`
- **description:** `port.rs` opens with `#![allow(dead_code, clippy::all)]` and `#![allow(missing_docs)]` at lines 43-44, shadowing the crate-level `#![deny(missing_docs)]` declared in `lib.rs:10`. This means every public item in the most important file of the port (the one that defines the trait every adapter must implement) is published without rustdoc. The crate-level deny is silently inactive for this module.
- **expected:** AGENTS.md and `docs/code-standards.md`: "All public APIs are documented with rustdoc; `#![deny(missing_docs)]`."
- **evidence:**
  ```rust
  // crates/adapters/notify/src/lib.rs:9-10
  #![forbid(unsafe_code)]
  #![deny(missing_docs)]

  // crates/adapters/notify/src/port.rs:43-44
  #![allow(dead_code, clippy::all)]
  #![allow(missing_docs)]
  ```

---

### FINDING 8

- **id:** ADAPTER-NOT-008
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/notify/src/errors.rs:75-120`
- **description:** The shipped `NotificationError` enum at `errors.rs:75-120` deviates from the port contract. The spec (`docs/ports/notifications.md` § "Error Type") defines `Infrastructure(#[source] Box<dyn std::error::Error + Send + Sync>)` — the source error chain is preserved. The shipped enum drops the chain and stores `Infrastructure(String)` (line 119), then `NotificationError::infrastructure(source)` (line 129-131) renders the source via `.to_string()` immediately. The error type can no longer satisfy `Clone, Eq, Serialize, Deserialize` while also preserving the source chain — and the `BulkReceipt::failed` type at `port.rs:1349` (`Vec<(BulkRecipientIndex, NotificationError)>`) inherits this lossy representation.
- **expected:** `docs/ports/notifications.md` § "Error Type": `Infrastructure(#[source] Box<dyn std::error::Error + Send + Sync>)`.
- **evidence:**
  ```rust
  // docs/ports/notifications.md:198-207
  pub enum NotificationError {
      #[error("template not found: {0}")] TemplateNotFound(NotificationTemplateId),
      #[error("missing variable: {0}")] MissingVariable(String),
      #[error("invalid recipient: {0}")] InvalidRecipient(String),
      #[error("rate limited")] RateLimited,
      #[error("provider error: {0}")] Provider(String),
      #[error("quota exceeded")] QuotaExceeded,
      #[error("infrastructure error: {0}")] Infrastructure(#[source] Box<dyn std::error::Error + Send + Sync>),
  }

  // crates/adapters/notify/src/errors.rs:118-131
  #[error("infrastructure error: {0}")]
  Infrastructure(String),
  ...
  pub fn infrastructure(source: impl std::error::Error + Send + Sync + 'static) -> Self {
      Self::Infrastructure(source.to_string())
  }
  ```

---

### FINDING 9

- **id:** ADAPTER-NOT-009
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/notify/src/port.rs:1188-1206` and `crates/adapters/notify/src/port.rs:1256-1261`
- **description:** `SendNotification` and `SendBulkNotification` carry a `school_id: SchoolId` field on top of `tenant: TenantContext` (which already carries `school_id`). This is doc-vs-code drift: the spec defines `tenant: TenantContext` as the only school-identifying field (`docs/ports/notifications.md` § "SendNotification", lines 25-38, and § "SendBulkNotification", lines 136-145). The shipped struct duplicates the field and exposes `active_school_id(&self) -> SchoolId { self.school_id }` (line 1197-1199) — a redundant accessor that bypasses the tenant context. Any consumer that mutates one and forgets the other (or sets them to different values) creates an invariant violation that cannot be detected at compile time.
- **expected:** `docs/ports/notifications.md` § "SendNotification": only `tenant: TenantContext` (which carries `school_id`); no `school_id` field.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/port.rs:1156-1199
  pub struct SendNotification {
      pub tenant: TenantContext,
      pub channel: Channel,
      pub template: TemplateRef,
      pub recipient: Recipient,
      pub variables: BTreeMap<String, TemplateValue>,
      pub attachments: Vec<AttachmentRef>,
      pub priority: Priority,
      pub scheduled_at: Option<Timestamp>,
      pub idempotency_key: Option<IdempotencyKey>,
      pub correlation_id: Option<CorrelationId>,
      pub school_id: SchoolId,        // <-- drift
  }
  impl SendNotification {
      #[must_use]
      pub fn active_school_id(&self) -> SchoolId {
          self.school_id    // <-- bypasses tenant
      }
  ```

---

### FINDING 10

- **id:** ADAPTER-NOT-010
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/notify/Cargo.toml:13-32`
- **description:** The `Cargo.toml` declares three dependencies that are never imported: `educore-audit` (line 20), `educore-events` (line 16), `educore-platform` (line 15). The handoff (PHASE-15-HANDOFF.md:42-43) lists these crates as expected integrations. `educore-audit` was specifically added so notification events would be auditable (per the port spec § "Audit": "Every send, success or failure, is recorded in the audit log"), but neither `EmailProvider::send` nor `SmsProvider::send` ever writes an audit entry. The other two are similarly unwired.
- **expected:** Spec § "Audit": "Every send, success or failure, is recorded in the audit log with template id, recipient hash, channel, status, and cost."
- **evidence:**
  ```toml
  # crates/adapters/notify/Cargo.toml:14-20
  educore-core = { workspace = true }
  educore-platform = { workspace = true }
  educore-events = { workspace = true }
  tokio = { workspace = true }
  async-trait = { workspace = true }
  lettre = { workspace = true, features = ["smtp-transport", "builder", "tokio1-rustls-tls"] }
  educore-audit = { workspace = true }
  ```
  `grep -nE "educore_audit|educore_events|educore_platform"` in `src/**/*.rs` returns no matches (0 hits).

---

### FINDING 11

- **id:** ADAPTER-NOT-011
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/notify/src/email.rs:90-140` and `crates/adapters/notify/src/sms.rs:296-340`
- **description:** Neither provider implements any retry policy on transient errors. `EmailProvider::send` wraps `lettre` failures as `NotificationError::infrastructure` once and returns (line 119-123); `SmsProvider::dispatch` does the same for `reqwest` (line 308-319). The port spec § "Rate Limiting" requires `RateLimited` returns and § "DeliveryStatus" includes a `Failed { reason, retryable }` variant where `retryable: bool` indicates whether the engine should retry — but the providers never set `retryable`, never surface `RateLimited`, and never classify 5xx vs 4xx. They also never honour the `Critical` priority (port.rs:1005), which the spec mandates "bypass queues and are delivered synchronously".
- **expected:** `docs/ports/notifications.md` § "Rate Limiting": "The adapter returns `NotificationError::RateLimited` when a limit is hit; the engine retries with backoff." § "DeliveryStatus": `Failed { reason, retryable }`. § "Priority": "`Critical` notifications bypass queues and are delivered synchronously."
- **evidence:**
  ```rust
  // crates/adapters/notify/src/email.rs:119-123
  let response = self.transport.send(message).await
      .map_err(NotificationError::infrastructure)?;
  // crates/adapters/notify/src/sms.rs:308-333
  let response = self.http.post(&self.gateway_url).header(...).form(&[...]).send().await
      .map_err(NotificationError::infrastructure)?;
  ...
  let status = if status_code.as_u16() == 202 { DeliveryStatus::Queued }
      else if status_code.is_success() { DeliveryStatus::Sent }
      else { return Err(NotificationError::provider(format!("sms gateway returned status {status_code}"))); };
  ```
  No `RateLimited` path, no 4xx-vs-5xx split, no `retryable` flag set anywhere.

---

### FINDING 12

- **id:** ADAPTER-NOT-012
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/notify/src/email.rs:107-115` (EmailProvider::send)
- **description:** `EmailProvider::send` captures the recipient's email address into a tuple at lines 110-115 and immediately discards it via `let _ = (...)`. This is an explicit PII capture-and-discard pattern. Combined with `recipient_email` being held in scope (line 107) and used only for the SMTP envelope at line 117, the captured address could trivially flow into a future `tracing::info!(...)` call by accident; the current shape is "captured for logging that never happened". The provider's `Debug` impl (line 80-86) does not redact `default_from` (an email address). The spec mandates PII hashing before any log.
- **expected:** Spec § "Audit": "PII (phone numbers, email addresses) is hashed before logging."
- **evidence:**
  ```rust
  // crates/adapters/notify/src/email.rs:107-115
  let recipient_email = resolve_email_recipient(&request.recipient)?;
  let body = render_template_body(&request.template, &request.variables);

  let log_school = request.tenant.school_id.to_string();
  let _ = (
      log_school.as_str(),
      template_id_of(&request.template).as_str(),
      recipient_email.as_str(),
  );

  // crates/adapters/notify/src/email.rs:80-86
  impl fmt::Debug for EmailProvider {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
          f.debug_struct("EmailProvider")
              .field("default_from", &self.default_from)   // <-- raw email in Debug
              .finish_non_exhaustive()
      }
  }
  ```

---

### FINDING 13

- **id:** ADAPTER-NOT-013
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/notify/src/sms.rs:81-91` (SmsProvider Debug impl)
- **description:** `SmsProvider`'s manual `Debug` impl at sms.rs:81-91 correctly redacts `api_key` to `"<redacted>"` but exposes `default_from` (a phone number, PII) and the `templates` keys (template ids, not strictly PII but visible). The `api_key` redaction is inconsistent with the `EmailProvider`, which does not redact `default_from` (see ADAPTER-NOT-012). There is no consistent redaction policy across providers.
- **expected:** Spec § "Audit": "PII (phone numbers, email addresses) is hashed before logging."
- **evidence:**
  ```rust
  // crates/adapters/notify/src/sms.rs:81-91
  impl fmt::Debug for SmsProvider {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
          f.debug_struct("SmsProvider")
              .field("http", &"reqwest::Client { .. }")
              .field("gateway_url", &self.gateway_url)
              .field("api_key", &"<redacted>")
              .field("default_from", &self.default_from)   // <-- raw phone in Debug
              .field("templates", &self.templates.keys().collect::<Vec<_>>())
              .finish()
      }
  }
  ```

---

### FINDING 14

- **id:** ADAPTER-NOT-014
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/notify/src/sms.rs:321-323` (SmsProvider::dispatch)
- **description:** `SmsProvider::dispatch` calls `response.text().await.unwrap_or_default()` (line 323), consuming the entire response body into memory just to scan for a `sid`. For an unbounded gateway response this is a memory and DoS surface. The `unwrap_or_default()` on `.text()` also silently swallows non-UTF-8 response bodies, hiding real errors from the caller.
- **expected:** Standard HTTP client practice: bound the body size, return a typed error when the body cannot be decoded, and parse only a header-sized slice (or use streaming JSON).
- **evidence:**
  ```rust
  // crates/adapters/notify/src/sms.rs:308-323
  let response = self.http.post(&self.gateway_url)
      .header("Authorization", self.basic_auth_header())
      .form(&[("To", to.as_str()), ("From", from.as_str()), ("Body", rendered.as_str())])
      .send().await
      .map_err(NotificationError::infrastructure)?;

  let status_code = response.status();
  let provider_message_id =
      extract_provider_message_id(&response.text().await.unwrap_or_default());
  ```

---

### FINDING 15

- **id:** ADAPTER-NOT-015
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/notify/src/sms.rs:457-468` (extract_provider_message_id)
- **description:** `extract_provider_message_id` is a hand-rolled JSON string scanner at sms.rs:457-468 that calls `body.find("\"sid\"")` then walks forward to find the next `:` and matching `"`. It does not handle JSON escape sequences, so any string value containing `\"sid\"` would be mis-parsed. It does not validate that the `sid` value is a string vs an integer or array, and there is no length cap on the matched value. The function is called on a body that the consumer does not control.
- **expected:** Use a real JSON parser (e.g. `serde_json::Value`) for the response, or at minimum cap the scan length and reject malformed JSON.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/sms.rs:457-468
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
  ```

---

### FINDING 16

- **id:** ADAPTER-NOT-016
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/notify/src/sms.rs:375-389` (SmsProvider::send_bulk)
- **description:** `SmsProvider::send_bulk` builds `BulkRecipientIndex::new(u32::try_from(global_idx).unwrap_or(u32::MAX))` where `global_idx = receipt.total()` (lines 377-378). On the first iteration `total()` is 0; on the second iteration (whether the first succeeded or failed) `total()` reflects the running count of receipts+failed. Across chunks, the index drifts: if row 0 of chunk 1 succeeds and row 1 of chunk 1 fails, the failure is reported with index 1 (the total count) instead of the original input row index. The engine cannot correlate the failure to its source row, which is the express purpose of `BulkRecipientIndex` per the port spec § "BulkReceipt".
- **expected:** `docs/ports/notifications.md` § "BulkReceipt": `failed: Vec<(BulkRecipientIndex, NotificationError)>` where `BulkRecipientIndex` is "the original input row index".
- **evidence:**
  ```rust
  // crates/adapters/notify/src/sms.rs:375-389
  for chunk in request.recipients.chunks(SMS_BULK_BATCH_SIZE) {
      for (offset, row) in chunk.iter().enumerate() {
          let global_idx = receipt.total();
          let index = BulkRecipientIndex::new(u32::try_from(global_idx).unwrap_or(u32::MAX));
          let single = build_single_from_bulk(&request, row);
          match self.dispatch(&single).await {
              Ok(r) => receipt.receipts.push(r),
              Err(e) => receipt.failed.push((index, e)),
          }
          ...
      }
  }
  ```
  `receipt.total()` advances on every iteration; `chunk` and `offset` are never combined into a stable index.

---

### FINDING 17

- **id:** ADAPTER-NOT-017
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/notify/src/sms.rs:286-289` (SmsProvider::basic_auth_header) and `crates/adapters/notify/src/sms.rs:136-146` (SmsProviderBuilder::api_key)
- **description:** `SmsProviderBuilder::api_key` (line 143-146) takes the user-supplied key and uses it directly as the username half of HTTP Basic auth. For Twilio this is wrong: Twilio uses `Basic <base64(AccountSID:AuthToken)>`, requiring **two** secrets. The builder only takes one field, and the docstring at lines 140-141 punts the responsibility to the consumer ("consumers needing full `{SID}:{AuthToken}` auth should pre-encode the pair as base64 and pass the resulting string here"). The actual `basic_auth_header` then wraps the already-base64 value in `format!("Basic {}", base64_encode(format!("{}:", self.api_key)))` (line 287-288) — double-encoding if the consumer followed the docstring. The credential handling is broken by design.
- **expected:** Two distinct builder methods: `account_sid(...)` and `auth_token(...)` (or a single pre-encoded value explicitly labelled as such), with the header built correctly for both raw and pre-encoded inputs.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/sms.rs:286-289
  fn basic_auth_header(&self) -> String {
      let raw = format!("{}:", self.api_key);
      format!("Basic {}", base64_encode(raw.as_bytes()))
  }

  // crates/adapters/notify/src/sms.rs:136-146
  /// For the default Twilio URL this is the
  /// Account SID; for custom gateways it is whatever the gateway
  /// accepts as the HTTP Basic auth user (the password half is
  /// left empty). Consumers needing full `{SID}:{AuthToken}`
  /// auth should pre-encode the pair as base64 and pass the
  /// resulting string here.
  pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
      self.api_key = api_key.into();
      self
  }
  ```

---

### FINDING 18

- **id:** ADAPTER-NOT-018
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/notify/src/email.rs:294-313` (resolve_email_recipient)
- **description:** `resolve_email_recipient` rejects every indirect recipient variant (`Recipient::User/Student/Guardian/Staff/Group/List/Expression`) with the message "engine must materialise indirect recipients before sending" (line 309-311). The port contract `docs/ports/notifications.md` § "Recipient" specifies that the adapter receives a "materialized list" only for `Recipient::Expression`, but `User`, `Student`, `Staff`, and `Group` ids are first-class recipient types the adapter must accept and resolve. The reference email provider therefore cannot send to any user, student, staff member, or group — the overwhelming majority of real recipients. The provider is not functional end-to-end without an out-of-band wrapper.
- **expected:** Spec § "Recipient": "The recipient" types include `User(UserId)`, `Student(StudentId)`, `Guardian(StudentId, GuardianRole)`, `Staff(StaffId)`, `Group(GroupId)` as first-class variants the adapter must dispatch on.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/email.rs:294-313
  fn resolve_email_recipient(recipient: &Recipient) -> Result<String> {
      match recipient {
          Recipient::Direct(contact) => contact.email.as_ref()
              .map(|e| e.as_str().to_owned())
              .ok_or_else(|| NotificationError::InvalidRecipient(
                  "contact has no email address".to_string())),
          Recipient::User(_)
          | Recipient::Student(_)
          | Recipient::Guardian(_, _)
          | Recipient::Staff(_)
          | Recipient::Group(_)
          | Recipient::List(_)
          | Recipient::Expression(_) => Err(NotificationError::InvalidRecipient(
              "engine must materialise indirect recipients before sending".to_string())),
      }
  }
  ```

---

### FINDING 19

- **id:** ADAPTER-NOT-019
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/notify/src/email.rs:294-313` and `crates/adapters/notify/src/sms.rs:208-228` (SmsProvider::recipient_phone)
- **description:** Both `resolve_email_recipient` (email.rs:294-313) and `recipient_phone` (sms.rs:208-228) reject `Recipient::List` and `Recipient::Expression` with errors ("nested recipient list not expanded by engine" / "recipient expression not expanded by engine"). The port spec § "Recipient" states explicitly that `Recipient::List` is a "flat list of recipients, delivered as a single logical send" and `Recipient::Expression` is "evaluated by the engine using the query layer; the adapter receives the materialized list." The reference implementations are inconsistent with this: a single `SendNotification` carrying `Recipient::List([...])` cannot be sent; the consumer must pre-walk the list and call `send` once per element.
- **expected:** Spec § "Recipient": `List(Vec<Recipient>)` is a first-class recipient; `Expression(RecipientExpr)` is expanded by the engine before the adapter sees it. Adapters must accept `List` (with internal fan-out) and never see `Expression`.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/sms.rs:214-219
  Recipient::List(_) => Err(NotificationError::InvalidRecipient(
      "nested recipient list not expanded by engine".into())),
  Recipient::Expression(_) => Err(NotificationError::InvalidRecipient(
      "recipient expression not expanded by engine".into())),
  ```
  The same `Recipient::List` error appears in `email.rs:308-310`.

---

### FINDING 20

- **id:** ADAPTER-NOT-020
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/notify/src/email.rs:88-140` (EmailProvider::send)
- **description:** `EmailProvider::send` does not set `NotificationReceipt::cost` (line 133-139 — only `provider_message_id` is set). The port spec § "Cost Tracking" mandates `cost: Option<Money>` be set by the adapter (e.g. $0.0075 per SMS; equivalent for email). The provider returns a receipt with `cost: None` (default at `port.rs:1309`), so the engine cannot track per-tenant cost.
- **expected:** `docs/ports/notifications.md` § "Cost Tracking": "`cost: Option<Money>` is set by the adapter (e.g. $0.0075 per SMS). The engine logs the cost for tenant-level reporting and budget control."
- **evidence:**
  ```rust
  // crates/adapters/notify/src/email.rs:133-139
  Ok(NotificationReceipt::new(receipt_id, request.channel,
      DeliveryStatus::Sent, Timestamp::now())
      .with_provider_message_id(response.code().to_string()))
  ```
  No `.with_cost(...)` call; the `cost` field remains `None`.

---

### FINDING 21

- **id:** ADAPTER-NOT-021
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/notify/src/email.rs:88-140` and `crates/adapters/notify/src/sms.rs:296-340`
- **description:** Neither `EmailProvider::send` nor `SmsProvider::dispatch` populates `NotificationReceipt::metadata`. The port spec defines `metadata: BTreeMap<String, String>` for "Provider-specific metadata (e.g. SES `RequestId`, FCM `message_id`)" (port.rs:1289-1291) and the `with_metadata` builder exists at `port.rs:1331-1334`. Both providers return receipts with an empty metadata map.
- **expected:** `docs/ports/notifications.md` § "NotificationReceipt": `metadata: BTreeMap<String, String>` for provider-specific data.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/email.rs:133-139
  Ok(NotificationReceipt::new(receipt_id, request.channel,
      DeliveryStatus::Sent, Timestamp::now())
      .with_provider_message_id(response.code().to_string()))
  // crates/adapters/notify/src/sms.rs:335-339
  let mut receipt = NotificationReceipt::new(receipt_id, channel, status, Timestamp::now());
  if let Some(sid) = provider_message_id {
      receipt = receipt.with_provider_message_id(sid);
  }
  Ok(receipt)
  ```
  Neither path calls `with_metadata`.

---

### FINDING 22

- **id:** ADAPTER-NOT-022
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/notify/src/email.rs:90-140` and `crates/adapters/notify/src/sms.rs:296-340`
- **description:** Neither provider honours `request.scheduled_at`, `request.priority`, `request.idempotency_key`, or `request.correlation_id`. None of these fields is read after the struct is destructured. `Critical` priority is supposed to "bypass queues and be delivered synchronously" (spec § "Priority") but the providers do not check it; idempotency keys are not used to dedupe retries; scheduled delivery is sent immediately; correlation IDs do not propagate to any log/event.
- **expected:** `docs/ports/notifications.md` § "Idempotency": "idempotency_key is used by the adapter to deduplicate retries." § "Priority": "`Critical` notifications bypass queues and are delivered synchronously." § "scheduled_at": "Optional scheduled delivery time. `None` means 'send now'."
- **evidence:**
  ```rust
  // crates/adapters/notify/src/email.rs:88-117
  async fn send(&self, request: SendNotification) -> Result<NotificationReceipt> {
      let from = match &request.channel { ... };
      let reply_to = match &request.channel { ... };
      let recipient_email = resolve_email_recipient(&request.recipient)?;
      let body = render_template_body(&request.template, &request.variables);
      // no reference to scheduled_at, priority, idempotency_key, correlation_id
      let message = build_lettre_message(...)?;
      let response = self.transport.send(message).await...;
      ...
  }
  // crates/adapters/notify/src/sms.rs:296-340 (dispatch) likewise ignores these fields.
  ```

---

### FINDING 23

- **id:** ADAPTER-NOT-023
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/notify/src/email.rs:88-140` and `crates/adapters/notify/src/sms.rs:296-340`
- **description:** Neither provider uses the in-crate `IdempotencyService` or `RateLimitService` helpers. `EmailProvider` and `SmsProvider` both ignore `request.idempotency_key`, never call `IdempotencyService::is_duplicate`, and never call `RateLimitService::try_acquire`. The helpers exist (`services.rs:445-478` and `services.rs:505-566`) and the spec requires their use, but the only consumers of these services are the unit tests and the integration tests.
- **expected:** `docs/ports/notifications.md` § "Idempotency" and § "Rate Limiting": adapters enforce both.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/sms.rs:347-356
  #[async_trait]
  impl NotificationProvider for SmsProvider {
      async fn send(&self, request: SendNotification) -> Result<NotificationReceipt> {
          if !matches!(request.channel, Channel::Sms { .. }) {
              return Err(NotificationError::Provider(...));
          }
          self.dispatch(&request).await
      }
      ...
  ```
  No `IdempotencyService` / `RateLimitService` call anywhere in the `send` / `send_bulk` / `status` paths.

---

### FINDING 24

- **id:** ADAPTER-NOT-024
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/notify/tests/notify_integration.rs:122-142`
- **description:** The two env-gated async integration tests (`notify_integration_async_email_send_mock` and `notify_integration_async_sms_send_mock`) construct a provider with `.build()` and immediately bind it to `let _provider = ...`. They perform no assertions, no actual send, no wire-format check, and no status verification. They exercise nothing beyond the builder's `build` method (which is already covered by sync unit tests at `email.rs:434-445` and `sms.rs:546-562`).
- **expected:** `docs/ports/notifications.md` § "Testing": "Integration tests of template resolution, variable application, and idempotency. A test of bulk send with partial failure. A test of rate limiting and retry. A test of status updates (delivered, opened, clicked)."
- **evidence:**
  ```rust
  // crates/adapters/notify/tests/notify_integration.rs:122-142
  #[tokio::test]
  #[ignore = "requires EDUCORE_PORT_ADAPTER_E2E env var"]
  async fn notify_integration_async_email_send_mock() {
      let _provider = EmailProviderBuilder::new()
          .relay_host("localhost").relay_port(1025)
          .credentials("test:test")
          .default_from("test@educore.local".to_owned())
          .build()
          .expect("smtp builder must succeed with relay_host + default_from");
  }
  #[tokio::test]
  #[ignore = "requires EDUCORE_PORT_ADAPTER_E2E env var"]
  async fn notify_integration_async_sms_send_mock() {
      let _provider = SmsProviderBuilder::new()
          .gateway_url("https://api.twilio.com/2010-04-01/Accounts/{account}/Messages.json")
          .api_key("ACtest_token").default_from("+15005550006".to_owned())
          .build();
  }
  ```

---

### FINDING 25

- **id:** ADAPTER-NOT-025
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/notify/tests/notify_integration.rs:1-142` (full file)
- **description:** The integration test file exercises only the four pure helper services (`TemplateService`, `ChannelService`, `IdempotencyService`, `RateLimitService`). It contains zero tests of `NotificationProvider::send`, `send_bulk`, or `status` (covered only by inline `#[cfg(test)] mod tests` inside `email.rs` and `sms.rs`). It has no test for: every `Channel` variant (spec § "Testing" — "Unit tests of every `Channel` variant"), `Recipient::Expression` evaluation, `cost` tracking, status updates (delivered/opened/clicked), bounce handling, attachment handling, scheduled delivery, or RBAC enforcement. The handoff at `PHASE-15-HANDOFF.md:143-146` claims "5 sync + 2 env-gated integration tests" but the env-gated tests are no-ops (see ADAPTER-NOT-024) and the sync tests don't exercise the provider.
- **expected:** Spec § "Testing" enumerates 7 specific test scenarios.
- **evidence:**
  ```rust
  // crates/adapters/notify/tests/notify_integration.rs (entire file, 142 lines)
  // 5 #[test] functions, all calling helpers in the public prelude:
  //   TemplateService, ChannelService, IdempotencyService, RateLimitService.
  // No test imports NotificationProvider, EmailProvider, or SmsProvider
  //   for an actual send().
  ```

---

### FINDING 26

- **id:** ADAPTER-NOT-026
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/notify/src/email.rs:69-86` and `crates/adapters/notify/src/sms.rs:71-91`
- **description:** Both provider structs derive `Debug` (email.rs:80-86) or implement it manually (sms.rs:81-91) but **neither struct is `Send + Sync`-derivable from its fields alone** — `SmsProvider` contains a `reqwest::Client` (which is `Send + Sync`) and primitives (fine) but lacks an explicit `Send + Sync` bound and never includes a `static_assertions::assert_impl_all` style compile-time check. While both will be `Send + Sync` in practice, the trait bound `NotificationProvider: Send + Sync + std::fmt::Debug` (port.rs:1397) is satisfied only implicitly; if a future field change adds a non-`Send` field, the trait contract fails silently because the trait's bound matches the struct's auto-trait until exercised.
- **expected:** Spec § "Object Safety" / port.rs:1397 mandates `Send + Sync + Debug`.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/port.rs:1396-1398
  #[async_trait]
  pub trait NotificationProvider: Send + Sync + std::fmt::Debug {
  ```
  Neither `email.rs` nor `sms.rs` contains a compile-time `assert_impl_all!(EmailProvider: Send + Sync)` or equivalent.

---

### FINDING 27

- **id:** ADAPTER-NOT-027
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/notify/src/sms.rs:286-289` (SmsProvider::basic_auth_header)
- **description:** `basic_auth_header` builds the value `format!("Basic {}", base64_encode(format!("{}:", self.api_key)))`. The `api_key` is concatenated with `:` and then base64-encoded. For Twilio, this is wrong: Twilio expects `Basic base64(AccountSID:AuthToken)`, where both halves are the consumer's two distinct secrets. Passing only `AccountSID` (with empty token) yields an unauthenticated request; passing `base64(AccountSID:AuthToken)` (already base64-encoded, per the docstring at sms.rs:140-141) yields double-encoding. The provider cannot talk to real Twilio in either configuration.
- **expected:** Either take `account_sid` + `auth_token` separately and base64-encode once, or take a single `Basic <already-encoded>` string verbatim.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/sms.rs:286-289
  fn basic_auth_header(&self) -> String {
      let raw = format!("{}:", self.api_key);
      format!("Basic {}", base64_encode(raw.as_bytes()))
  }
  ```

---

### FINDING 28

- **id:** ADAPTER-NOT-028
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/notify/src/port.rs:931` (Channel derives)
- **description:** `Channel` (the enum that drives the entire port) derives only `Debug, Clone, PartialEq, Eq, Serialize, Deserialize` (line 931). It does not derive `Hash`, `Default`, or `Copy`. The services module (`services.rs:204-209`) is forced to use a `HashMap<String, RateState>` keyed by a hand-rolled `channel_key` because `Channel: Hash` is unavailable. The spec calls out this deviation explicitly (`services.rs:42-48`). `RateLimitService`, `ChannelService`, and `IdempotencyService` consumers cannot use the enum as a key directly. The lack of `Copy` makes every match arm and every function signature pay for a heap-resident `String` clone when passing channels around.
- **expected:** Spec expects `Channel` to be a value type; the port.rs comment (line 32-37) acknowledges the lack of `Hash` and works around it. The workaround is a deviation, not a fix.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/port.rs:931
  #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  pub enum Channel {

  // crates/adapters/notify/src/services.rs:42-48
  //! - **`RateLimitService` uses `HashMap<String, RateState>` rather
  //!   than `HashMap<Channel, RateState>`.** [`Channel`](crate::port::Channel)
  //!   does not currently derive `Hash`, and the spec also lists
  //!   `crates/adapters/notify/src/port.rs` under "DO NOT TOUCH".
  ```

---

### FINDING 29

- **id:** ADAPTER-NOT-029
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/notify/src/email.rs:88-140` (EmailProvider::send)
- **description:** `EmailProvider::send` does not handle the recipient's attachment list. `request.attachments: Vec<AttachmentRef>` is part of `SendNotification` (port.rs:1174) but the function never reads it; `build_lettre_message` (email.rs:364-395) constructs the email from a single `body: &str` with no MIME multipart construction. Per the spec § "SendNotification", attachments are first-class and the adapter "resolves the template body, applies variables, and delivers" — including any attachments. The provider silently drops them.
- **expected:** Spec § "SendNotification": `attachments: Vec<AttachmentRef>` — a reference to a file attached to a notification.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/email.rs:364-395
  fn build_lettre_message(
      from: &str, reply_to: Option<&str>, recipient: &str, body: &str,
  ) -> std::result::Result<Message, NotificationError> {
      ...
      builder.body(body.to_owned())
          .map_err(|e| NotificationError::provider(format!("lettre failed to build body: {e}")))
  }
  ```
  No `.attachment(...)`, `.singlepart(...)`, or multipart builder calls. The `body` is a single string.

---

### FINDING 30

- **id:** ADAPTER-NOT-030
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/notify/src/sms.rs:296-340` (SmsProvider::dispatch)
- **description:** `SmsProvider::dispatch` does not handle `Channel::Sms.unicode` (port.rs:949). The unicode flag is destructured at sms.rs:255 as `unicode: _` (the leading underscore shows it is intentionally discarded) and never influences the wire format. Twilio and most gateways split unicode (UCS-2) bodies into 70-character segments vs 160 for GSM-7. The provider sends unicode text as if it were GSM-7, garbling the message at the gateway.
- **expected:** Spec § "Channel": `Sms { from: Option<PhoneNumber>, unicode: bool }` — `unicode: true` means the adapter must use UCS-2 encoding.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/sms.rs:254-269
  fn resolve_from(&self, channel: &Channel) -> Result<String> {
      let Channel::Sms { from, unicode: _ } = channel else { ... };
      if let Some(p) = from { return Ok(p.as_str().to_owned()); }
      ...
  }
  // crates/adapters/notify/src/sms.rs:296-340 (dispatch)
  // unicode flag never read; the body is sent as a single form field.
  ```

---

### FINDING 31

- **id:** ADAPTER-NOT-031
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/notify/src/email.rs:88-140` (EmailProvider::send) and `crates/adapters/notify/src/email.rs:142-189` (EmailProvider::send_bulk)
- **description:** `EmailProvider` never sets the SMTP envelope's `MAIL FROM` to a per-tenant return-path. `Channel::Email.from` overrides only the From: header; the SMTP `MAIL FROM` uses the default `default_from` and the same builder-configured host. Multi-tenant sends from a single `EmailProvider` therefore share a single bounce-domain envelope, breaking Bounce / FBL processing per tenant. The port spec § "Multi-tenancy" implicitly requires per-tenant envelope identity.
- **expected:** Spec § "Channel::Email": `from: Option<EmailAddress>` — the adapter must use this for the From: header AND (for SES, Postmark, etc.) the envelope sender / MAIL FROM.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/email.rs:90-117
  let from = match &request.channel {
      Channel::Email { from, .. } => from.as_ref()
          .map(|e| e.as_str().to_owned())
          .unwrap_or_else(|| self.default_from.clone()),
      ...
  };
  ...
  let message = build_lettre_message(&from, reply_to.as_deref(),
      &recipient_email, &body)?;
  ```
  Only the From: header is set; `lettre::Message::builder()` does not expose a `mail_from(...)` configuration here.

---

### FINDING 32

- **id:** ADAPTER-NOT-032
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/notify/src/email.rs:69-78` (EmailProvider struct)
- **description:** `EmailProvider` is `Clone`-derived but the inner `AsyncSmtpTransport<Tokio1Executor>` is the only field that holds connection state. The `default_from: String` is the only per-tenant configuration; there is no way to configure separate per-tenant reply-to, header-from, or envelope-from addresses. The struct therefore models a single SMTP account, not a multi-tenant provider; multi-tenant deployments must construct a separate `EmailProvider` per tenant, defeating the `Arc<dyn NotificationProvider>` storage model in the engine.
- **expected:** Spec § "Multi-tenancy": every send carries `tenant: TenantContext`; the adapter must support many tenants over one connection pool.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/email.rs:74-78
  #[derive(Clone)]
  pub struct EmailProvider {
      transport: AsyncSmtpTransport<Tokio1Executor>,
      default_from: String,
  }
  ```
  Two fields; no per-tenant lookup tables, no tenant-id keyed map.

---

### FINDING 33

- **id:** ADAPTER-NOT-033
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/notify/src/email.rs:191-193` (EmailProvider::status)
- **description:** `EmailProvider::status` ignores its `_receipt_id` argument entirely and returns `DeliveryStatus::Sent` for every call. Unlike `SmsProvider::status`, which has a doc comment acknowledging it is a stub, `EmailProvider::status` has no such comment — it presents as a complete implementation. The provider has no way to query SES `GetMessageInsights`, Postmark's `MessageInfo`, or any other provider status endpoint.
- **expected:** Spec § "DeliveryStatus": "The adapter updates the status as the provider reports it (via webhook)."
- **evidence:**
  ```rust
  // crates/adapters/notify/src/email.rs:191-193
  async fn status(&self, _receipt_id: NotificationReceiptId) -> Result<DeliveryStatus> {
      Ok(DeliveryStatus::Sent)
  }
  ```

---

### FINDING 34

- **id:** ADAPTER-NOT-034
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/notify/src/sms.rs:476-484` (generate_id)
- **description:** `generate_id` uses `SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_micros()).unwrap_or(0)` and pairs it with an in-process `AtomicU64` counter. The result is a string `sms-<micros_hex>-<counter_hex>` that is unique per (process, microsecond, counter-tick). On process restart the counter resets to 0, so two concurrent processes can produce the same `receipt_id` in the same microsecond. `NotificationReceiptId` is supposed to be durable (`port.rs:67-73`: "The engine stores it in `communication_email_sms_logs`"), and a duplicate receipt id collides on insert.
- **expected:** Spec § "NotificationReceipt": `receipt_id` is durable and unique.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/sms.rs:476-484
  fn generate_id(prefix: &str) -> String {
      static COUNTER: AtomicU64 = AtomicU64::new(0);
      let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
      let micros = SystemTime::now()
          .duration_since(UNIX_EPOCH)
          .map(|d| d.as_micros())
          .unwrap_or(0);
      format!("{prefix}-{micros:x}-{counter:x}")
  }
  ```

---

### FINDING 35

- **id:** ADAPTER-NOT-035
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/notify/src/email.rs:125-131` (EmailProvider receipt id construction)
- **description:** `EmailProvider::send` constructs receipt ids with `format!("email:{log_school}:{}", SystemTime::now()...)`, embedding the school id in plaintext into every receipt id. The spec § "Audit": "PII (phone numbers, email addresses) is hashed before logging." School id is tenant PII in this domain; receipt ids flow into the durable `communication_email_sms_logs` table and into logs. The `BulkId` and `EmailProvider::send_bulk` follow the same pattern at email.rs:149-155. Also note that `SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0)` returns 0 silently on clock skew.
- **expected:** Spec § "Audit": PII hashing.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/email.rs:125-131
  let receipt_id = NotificationReceiptId::new(format!(
      "email:{log_school}:{}",
      std::time::SystemTime::now()
          .duration_since(std::time::UNIX_EPOCH)
          .map(|d| d.as_millis())
          .unwrap_or(0)
  ));
  ```

---

### FINDING 36

- **id:** ADAPTER-NOT-036
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/notify/src/port.rs:1277` (NotificationReceipt.provider_message_id)
- **description:** The `provider_message_id` field on `NotificationReceipt` is stored verbatim from whatever the provider returns. `EmailProvider::send` stores `response.code().to_string()` (email.rs:139) — lettre's SMTP response code (e.g. `"250"`), not the provider's message id (e.g. SES `MessageId` is in the response headers / X-SES-Configuration-Set). `SmsProvider::dispatch` uses a hand-rolled JSON scan (sms.rs:457-468). Neither provider produces a usable correlation id for webhook reconciliation.
- **expected:** Spec § "NotificationReceipt": `provider_message_id` is "The provider's message id (e.g. SES `MessageId`, Twilio `MessageSid`). Used to reconcile webhook status updates."
- **evidence:**
  ```rust
  // crates/adapters/notify/src/email.rs:139
  .with_provider_message_id(response.code().to_string())
  ```
  `response.code()` is lettre's `lettre::transport::smtp::response::Response.code()`, the SMTP status code, not a message id.

---

### FINDING 37

- **id:** ADAPTER-NOT-037
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/notify/src/port.rs:107-145` (BulkId) and `crates/adapters/notify/src/sms.rs:476-484` (generate_id)
- **description:** `BulkId` and `NotificationReceiptId` are both `String`-backed newtypes, and both are generated by `SmsProvider` via `generate_id("bulk")` (sms.rs:365) and `generate_id("sms")` (sms.rs:305). The id is process-local and not derived from the canonical UUID ecosystem (`educore_core::ids`). The engine's storage adapter expects UUID-shaped ids per the `communication_email_sms_logs` schema. The id generation is also unrelated to the `IdempotencyService::derive_key` SHA-256 output (which is the spec's deterministic-idempotency-key path).
- **expected:** Spec § "Idempotency": the engine generates a deterministic key from `(command_id, recipient, template_version)`.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/sms.rs:365
  let bulk_id = BulkId::new(generate_id("bulk"));
  // crates/adapters/notify/src/sms.rs:476-484
  fn generate_id(prefix: &str) -> String {
      static COUNTER: AtomicU64 = AtomicU64::new(0);
      ...
  }
  ```

---

### FINDING 38

- **id:** ADAPTER-NOT-038
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/notify/src/services.rs:73-176` (hand-rolled SHA-256)
- **description:** `services.rs` ships a hand-rolled SHA-256 implementation (lines 73-176) with a 100+ line block, claiming FIPS 180-4 §6.2 compliance. The crate's `Cargo.toml` does not declare the `sha2` crate and the task spec for this file lists the manifest under "DO NOT TOUCH" (services.rs:35-41, 65-71). The same SHA-256 implementation is duplicated in `crates/adapters/files/src/local.rs` per the comment at services.rs:38-41. Hand-rolled crypto is a major audit risk: a single off-by-one in the padding (`while buf.len() % 64 != 56 { buf.push(0x00); }` line 103-105), the rotate constants, or the initial hash values produces silent corruption. The `sha2` crate is already in the workspace dependency graph (the auth crate uses `hmac 0.12` per `PHASE-15-HANDOFF.md:247`).
- **expected:** Use the audited `sha2` crate for SHA-256; the workspace already pulls in `hmac`, so the dependency is already justified.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/services.rs:73-90
  const SHA256_K: [u32; 64] = [
      0x428a2f98, 0x71374491, ...
  ];
  const SHA256_H0: [u32; 8] = [
      0x6a09e667, 0xbb67ae85, ...
  ];
  // crates/adapters/notify/src/services.rs:92-176
  fn sha256(message: &[u8]) -> [u8; 32] {
      ...
  }

  // crates/adapters/notify/src/services.rs:31-41 (deviation comment)
  //! - **SHA-256 is hand-rolled, not pulled from the `sha2` crate.**
  //!   The crate's `Cargo.toml` does not declare `sha2` and the task
  //!   spec for this file (`Phase 15: educore-notify services (B)`)
  //!   explicitly lists `crates/adapters/notify/Cargo.toml` under
  //!   "DO NOT TOUCH".
  ```

---

### FINDING 39

- **id:** ADAPTER-NOT-039
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/notify/src/services.rs:486-566` (RateLimitService)
- **description:** `RateLimitService` is documented (services.rs:30-32) and tested (services.rs:677-706, integration test at notify_integration.rs:99-116) but no provider uses it. `EmailProvider` and `SmsProvider` both lack any rate-limit enforcement, meaning a tenant with `Critical` priority or a flood of sends can exceed gateway throttling and be blacklisted. Spec § "Rate Limiting" requires per-tenant, per-channel limits configurable per tenant — the shipped service is process-local (`HashMap<String, RateState>`), single-tenant, and never wired in.
- **expected:** Spec § "Rate Limiting": per-tenant, per-channel limits enforced by the adapter.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/services.rs:497-507
  /// In-memory token-bucket rate limiter, one bucket per channel.
  ///
  /// The bucket refills at one token per 1000 ms, capped at
  /// `max_per_second`. The bucket starts full on first use. The
  /// service is process-local; production deployments that span
  /// multiple processes or pods should back the limiter with a
  /// shared store.
  #[derive(Debug, Default)]
  pub struct RateLimitService {
      state: HashMap<String, RateState>,
  }
  ```
  No `RateLimitService` reference in `email.rs` or `sms.rs`.

---

### FINDING 40

- **id:** ADAPTER-NOT-040
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/notify/src/email.rs:336-356` (substitute_variables and value_to_string)
- **description:** `substitute_variables` does a simple `result.replace(&placeholder, &value_to_string(value))` for each variable. For each variable it scans and rewrites the entire body, so the complexity is `O(n * m)` where `n` is the number of variables and `m` is the body length. More importantly, `replace` replaces every occurrence (including in the body of another variable — e.g. `{user}` inside a value destined for `{name}` causes re-substitution if iterated in the wrong order), and the replacement of `{score}` inside the text "your score is {score}" produces "your score is 95" — but if the value itself contains `{score}` (e.g. `TemplateValue::Text("{score}")`), the output becomes "your score is 95" then "your score is 95" → silent double-substitution. The function in `services.rs:277-307` (`TemplateService::substitute_variables`) handles this correctly with a single-pass scanner; the email.rs copy does not.
- **expected:** Spec § "Templates": variable substitution must be deterministic.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/email.rs:336-356
  pub fn substitute_variables(body: &str, variables: &BTreeMap<String, TemplateValue>) -> String {
      let mut result = body.to_owned();
      for (key, value) in variables {
          let placeholder = format!("{{{key}}}");
          result = result.replace(&placeholder, &value_to_string(value));
      }
      result
  }
  fn value_to_string(value: &TemplateValue) -> String { ... }
  ```
  Compare with the correct single-pass implementation at services.rs:278-307 (`TemplateService::substitute_variables`), which the email provider does not call.

---

### FINDING 41

- **id:** ADAPTER-NOT-041
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/notify/src/sms.rs:236-246` (SmsProvider::render_template)
- **description:** `SmsProvider::render_template` uses the `{{name}}` placeholder syntax (double braces), while `EmailProvider::substitute_variables` uses `{name}` (single braces), while `TemplateService::substitute_variables` (services.rs:278) uses `{name}`. There is no single source of truth for the variable syntax. A template authored against `TemplateService` semantics (single brace) sent via SMS would not be substituted.
- **expected:** Spec § "Templates": templates are stored in the communication domain; both adapters should use the same substitution engine.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/sms.rs:236-246
  fn render_template(body: &str,
      variables: &std::collections::BTreeMap<String, crate::port::TemplateValue>) -> String {
      let mut out = body.to_owned();
      for (name, value) in variables {
          let needle = format!("{{{{{name}}}}}");   // <-- {{name}}, double braces
          out = out.replace(&needle, &template_value_as_str(value));
      }
      out
  }

  // crates/adapters/notify/src/email.rs:336-344
  pub fn substitute_variables(body: &str, variables: &BTreeMap<String, TemplateValue>) -> String {
      let mut result = body.to_owned();
      for (key, value) in variables {
          let placeholder = format!("{{{key}}}");   // <-- {name}, single braces
          ...
      }
      ...
  }
  ```

---

### FINDING 42

- **id:** ADAPTER-NOT-042
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/notify/src/services.rs:417-420` (ChannelService::fan_out_targets)
- **description:** `ChannelService::fan_out_targets` is documented as computing "the per-channel fan-out list for a single-channel request" and exists "so a future 'multi-channel request' feature can reuse the same helper without changing adapter call sites." The current implementation returns `vec![channel.clone()]` for every input — the comment at services.rs:413-416 says exactly this. The helper is a placeholder that adds no value today and exists only to be asserted against by tests (services.rs:610-653 doesn't even test this method).
- **expected:** Either implement multi-channel fan-out (which the port spec § "Channel" says is possible: "A single notification can target multiple channels. The consumer adapter may fan out internally.") or remove the placeholder.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/services.rs:410-420
  /// Returns the list of channels to dispatch to for a single
  /// channel input. The notify port currently routes each
  /// request through exactly one channel, so this returns a
  /// single-element vector; the method exists so a future
  /// "multi-channel request" feature can reuse the same helper
  /// without changing adapter call sites.
  #[must_use]
  pub fn fan_out_targets(channel: &Channel) -> Vec<Channel> {
      vec![channel.clone()]
  }
  ```

---

### FINDING 43

- **id:** ADAPTER-NOT-043
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/notify/src/services.rs:454-466` (IdempotencyService::derive_key) and `docs/ports/notifications.md:163-166`
- **description:** `IdempotencyService::derive_key` derives the key from `(command_id: &str, recipient: &str, template_version: u32)` where `recipient` is a `&str` (a free-form string like "alice@example.test" per the unit test). The spec § "Idempotency" says the engine "generates a deterministic key from `(command_id, recipient, template_version)`" — `recipient` here is the structured `Recipient` enum, not a string. The port's `Recipient` carries variant information (e.g. `Student(id)`, `Guardian(id, role)`); flattening to a string loses the role and the recipient-kind. Two different `Recipient::Guardian` values (Primary vs Secondary) for the same student would collide.
- **expected:** Spec § "Idempotency": "engine generates a deterministic key from `(command_id, recipient, template_version)`" — where `recipient` is the typed enum.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/services.rs:454-466
  /// Derives a deterministic SHA-256 hex idempotency key from
  /// `command_id`, `recipient`, and `template_version`.
  ///
  /// The canonical input form is:
  ///
  /// ```text
  /// <command_id>:<recipient>:<template_version as decimal>
  /// ```
  #[must_use]
  pub fn derive_key(command_id: &str, recipient: &str, template_version: u32) -> String {
      let input = format!("{command_id}:{recipient}:{template_version}");
      sha256_hex(input.as_bytes())
  }
  ```
  No `Recipient` parameter; the unit test uses `"alice@example.test"` and `"bob@example.test"` (services.rs:657-669) — stringly typed.

---

### FINDING 44

- **id:** ADAPTER-NOT-044
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/notify/src/services.rs:530-551` (RateLimitService::try_acquire)
- **description:** `RateLimitService::try_acquire` computes the elapsed refill as `elapsed_ms / 1000`, discarding the sub-second remainder (services.rs:531-533). The module comment at services.rs:48-53 calls this out as deliberate ("sub-second carry-over is discarded (a 500ms pause refills zero tokens)"). The port spec § "Rate Limiting" example says "100 SMS/second" — a literal integer rate. Discarding sub-second refills means the effective rate is `floor(elapsed / 1000) * max_per_second`, which is not a 100/sec refill but a bursty batchy one. The behavior is wrong for sub-second throttling scenarios (Twilio's Messaging Services throttle, FCM's per-second caps).
- **expected:** Spec § "Rate Limiting": "e.g. 100 SMS/second" — a continuous rate, not a batched-second rate.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/services.rs:530-543
  let elapsed_ms =
      u64::try_from(now.duration_since(entry.last_refill).as_millis()).unwrap_or(u64::MAX);
  let new_tokens = u32::try_from(elapsed_ms / 1000).unwrap_or(u32::MAX);
  if new_tokens > 0 {
      entry.tokens = entry.tokens
          .saturating_add(new_tokens).min(entry.max_tokens);
      entry.last_refill = entry.last_refill
          .checked_add(Duration::from_millis(u64::from(new_tokens) * 1000))
          .unwrap_or(now);
  }
  ```

---

### FINDING 45

- **id:** ADAPTER-NOT-045
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/notify/src/services.rs:218-228` (channel_key Push variant)
- **description:** `channel_key` for `Channel::Push` keys the rate-limit bucket on `(topic, ttl_ms, collapse_key)`. A single tenant sending the same push to multiple topic variants (e.g. one for sports, one for academics) gets N independent buckets — but two tenants sending to the same topic also share a bucket (the topic string is the key, no tenant scoping). The spec § "Rate Limiting" requires "per-tenant, per-channel rate limits" — the bucket is keyed only on the channel, not on the tenant.
- **expected:** Spec § "Rate Limiting": "per-tenant, per-channel rate limits ... configurable per tenant."
- **evidence:**
  ```rust
  // crates/adapters/notify/src/services.rs:218-228
  Channel::Push { topic, ttl, collapse_key } => {
      let topic = topic.as_deref().unwrap_or("-");
      let collapse = collapse_key.as_deref().unwrap_or("-");
      let ttl_ms = ttl.map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX));
      format!("push:{topic}:{ttl_ms}:{collapse}")
  }
  ```
  No `tenant_id` / `school_id` in the key. `channel_key` for `Email`, `Sms`, `Push`, etc. (services.rs:209-241) never references `school_id`.

---

### FINDING 46

- **id:** ADAPTER-NOT-046
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/notify/src/port.rs:1015-1018` (Priority::as_str) and `crates/adapters/notify/src/services.rs:520` (try_acquire)
- **description:** `Priority` (port.rs:994-1006) defines `Critical` as a distinct priority that the spec says "bypass queues and are delivered synchronously" and "may charge a premium." Neither `EmailProvider`, `SmsProvider`, `RateLimitService`, nor any helper treats `Critical` differently from `Normal`. The value is read from the request, never inspected, and silently downgraded.
- **expected:** Spec § "Priority": "`Critical` notifications bypass queues and are delivered synchronously. The adapter may charge a premium for `Critical`."
- **evidence:**
  ```rust
  // crates/adapters/notify/src/email.rs:88-140
  // no match on request.priority; the field is read once via
  // SendNotification::priority but never branched on.
  // crates/adapters/notify/src/sms.rs:347-356
  // same — no match on request.priority.
  ```

---

### FINDING 47

- **id:** ADAPTER-NOT-047
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/notify/src/services.rs:30-32` and `crates/adapters/notify/src/services.rs:445-478` (IdempotencyService)
- **description:** The `IdempotencyService` is documented in the module-level docstring (services.rs:30-32) and exposed as a service helper, but `IdempotencyService::is_duplicate` requires the caller to pass a `&mut HashSet<String>` (services.rs:470-477). The service "holds no state of its own" (services.rs:441-443). Real adapters therefore have to manage their own `HashMap<SchoolId, HashSet<String>>` and pass the inner set on every call — the helper does no encapsulation. The same shape would be required to wire the service into `EmailProvider` / `SmsProvider`, but neither does.
- **expected:** A self-contained `IdempotencyService` that holds the per-tenant set internally and exposes `check_or_insert(school_id, key)`.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/services.rs:469-477
  pub fn is_duplicate(key: &str, seen_keys: &mut HashSet<String>) -> bool {
      if seen_keys.contains(key) {
          true
      } else {
          seen_keys.insert(key.to_owned());
          false
      }
  }
  ```

---

### FINDING 48

- **id:** ADAPTER-NOT-048
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/notify/src/sms.rs:50-52` (DEFAULT_GATEWAY_URL)
- **description:** `DEFAULT_GATEWAY_URL` is a Twilio Messages URL with the literal placeholder `{account}` substituted from `api_key` (sms.rs:51-52). For `api_key = "AC0123456789abcdef"` the URL becomes `https://api.twilio.com/2010-04-01/Accounts/AC0123456789abcdef/Messages.json` (asserted at sms.rs:555-558). There is no validation that `api_key` is a Twilio-shaped `AC` prefix; any string becomes part of the URL path, opening a credential-shaped injection (e.g. an `api_key` containing `/` or `?` rewrites the path or query).
- **expected:** Validate the API key shape before URL interpolation, or URL-encode the path segment.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/sms.rs:174-177
  pub fn build(self) -> SmsProvider {
      let gateway_url = self.gateway_url
          .unwrap_or_else(|| DEFAULT_GATEWAY_URL.replace("{account}", &self.api_key));
  ```
  No validation; `&self.api_key` is interpolated verbatim.

---

### FINDING 49

- **id:** ADAPTER-NOT-049
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/notify/src/sms.rs:178` (Client::builder().build().unwrap_or_else(|_| Client::new()))
- **description:** `SmsProviderBuilder::build` constructs the reqwest `Client` via `Client::builder().build().unwrap_or_else(|_| Client::new())`. `reqwest::Client::new()` is deprecated in modern reqwest (it ignores proxy config, TLS, etc.). The `unwrap_or_else` pattern silently masks the builder failure. A consumer expecting a TLS-configured client receives an unconfigured one with no diagnostic.
- **expected:** Either propagate the builder error as a `NotificationError`, or document the un-configured fallback explicitly.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/sms.rs:178
  let http = Client::builder().build().unwrap_or_else(|_| Client::new());
  ```

---

### FINDING 50

- **id:** ADAPTER-NOT-050
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/notify/src/email.rs:74-78` (EmailProvider struct) and `crates/adapters/notify/src/email.rs:204-209` (EmailProviderBuilder struct)
- **description:** Neither `EmailProvider` nor `EmailProviderBuilder` carries a reference to `tenant.school_id`; the builder does not accept tenant-specific configuration. The transport's host, port, credentials, and default_from are builder-time singletons. Multi-tenant deployments that need different SMTP accounts per school (e.g. for branded sender domains) must construct a separate provider per school, defeating the `Arc<dyn NotificationProvider>` engine-storage pattern.
- **expected:** Spec § "Multi-tenancy": one provider serves many tenants.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/email.rs:204-209
  #[derive(Debug, Default, Clone)]
  pub struct EmailProviderBuilder {
      relay_host: Option<String>,
      relay_port: Option<u16>,
      credentials_user: Option<String>,
      default_from: Option<String>,
  }
  ```

---

### FINDING 51

- **id:** ADAPTER-NOT-051
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/notify/src/port.rs:464-498` (Url newtype)
- **description:** `Url` is a `String`-backed newtype at port.rs:466 with no `parse()` / validation. Spec § "Channel::Webhook": `Webhook { url: Url, secret: Option<SecretString> }` — the adapter must actually POST to that URL. `EmailProvider` does not implement `Channel::Webhook` (missing impl), but `SmsProvider` also accepts any `Channel::Sms` URL and never validates the URL before passing it to reqwest. A malformed URL surfaces as a reqwest error rather than a typed `NotificationError::InvalidRecipient` or similar.
- **expected:** Spec § "Channel": adapters validate URL shape.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/port.rs:461-498
  /// A URL. The port treats the value as opaque; adapters parse and
  /// validate. Stored as a `String` so the port crate does not take
  /// a direct dependency on the `url` crate.
  #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
  #[serde(transparent)]
  pub struct Url(pub String);
  ```

---

### FINDING 52

- **id:** ADAPTER-NOT-052
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/notify/src/email.rs:107-110` (EmailProvider::send) and `crates/adapters/notify/src/sms.rs:208-213` (SmsProvider::recipient_phone)
- **description:** Neither provider hashes recipient identifiers before logging or storing them on the receipt. The receipt `metadata: BTreeMap<String, String>` is empty (ADAPTER-NOT-021); `log_school` is captured and discarded (ADAPTER-NOT-012); no hash of the recipient address appears anywhere. Spec § "Audit": "PII (phone numbers, email addresses) is hashed before logging." The shipped providers cannot log a single identifiable audit row that complies with this rule.
- **expected:** Spec § "Audit": recipient hash on every send log.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/email.rs:107-115
  let recipient_email = resolve_email_recipient(&request.recipient)?;
  let body = render_template_body(&request.template, &request.variables);
  let log_school = request.tenant.school_id.to_string();
  let _ = (log_school.as_str(),
      template_id_of(&request.template).as_str(),
      recipient_email.as_str());
  ```
  No hash is computed; `recipient_email.as_str()` is captured as-is.

---

### FINDING 53

- **id:** ADAPTER-NOT-053
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/notify/src/sms.rs:208-228` (recipient_phone)
- **description:** `recipient_phone` rejects `Recipient::User/Student/Guardian/Staff/Group` with `"recipient requires contact lookup; not supported by reference SmsProvider"`. The reference implementation is structurally incapable of sending SMS to any user, student, staff member, or group. This means the integration tests cannot exercise the happy path of "user receives an SMS" — they must construct `Recipient::Direct(ContactInfo::new().with_phone(...))` or trigger the failure path (sms.rs:640-694). The handoff at PHASE-15-HANDOFF.md:144-146 acknowledges only "template substitute, template validate, channel classification, idempotency key derivation, rate-limit bucket" — none of which is a provider test.
- **expected:** Spec § "Recipient": all recipient variants dispatchable.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/sms.rs:220-227
  Recipient::User(_)
  | Recipient::Student(_)
  | Recipient::Guardian(_, _)
  | Recipient::Staff(_)
  | Recipient::Group(_) => Err(NotificationError::InvalidRecipient(
      "recipient requires contact lookup; not supported by reference SmsProvider".into())),
  ```

---

### FINDING 54

- **id:** ADAPTER-NOT-054
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/notify/src/email.rs:142-189` (EmailProvider::send_bulk)
- **description:** `EmailProvider::send_bulk` re-uses `EmailProvider::send` for each row, which means each row triggers a full `render_template_body` + `build_lettre_message` + SMTP send. The per-row work is identical except for the recipient and the variables — there is no caching of the rendered template subject or the constant headers, no MIME reuse, no `MAIL FROM` reuse. Per the spec § "Bulk Send", a batched send should reuse the message template and only vary the recipient.
- **expected:** Spec § "Bulk Send": batched send.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/email.rs:159-186
  for (idx, row) in request.recipients.iter().enumerate() {
      ...
      let single = SendNotification { ... };
      match self.send(single).await { ... }
  }
  ```

---

### FINDING 55

- **id:** ADAPTER-NOT-055
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/notify/src/email.rs:336-356` (substitute_variables)
- **description:** `substitute_variables` in `email.rs` (lines 336-356) is `pub` but is also re-implemented in `services.rs:278-307` as `TemplateService::substitute_variables`. The two implementations diverge: `email.rs` takes `BTreeMap<String, TemplateValue>` and stringifies each value via `value_to_string` (lines 347-356); `services.rs` takes `BTreeMap<String, String>` and substitutes verbatim. Two source-of-truth substitution engines in one crate. The handoff's `PHASE-15-HANDOFF.md:131-134` says "TemplateService::substitute_variables + validate_required_variables + extract_variables" — but the email provider ships its own.
- **expected:** One substitution helper, used by every channel adapter.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/email.rs:336-344
  pub fn substitute_variables(body: &str, variables: &BTreeMap<String, TemplateValue>) -> String {
      let mut result = body.to_owned();
      for (key, value) in variables {
          let placeholder = format!("{{{key}}}");
          result = result.replace(&placeholder, &value_to_string(value));
      }
      result
  }
  // crates/adapters/notify/src/services.rs:277-307
  pub fn substitute_variables(body: &str, variables: &BTreeMap<String, String>) -> String {
      ...
  }
  ```
  Different signatures, different semantics.

---

### FINDING 56

- **id:** ADAPTER-NOT-056
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/notify/src/sms.rs:50-52` and `crates/adapters/notify/src/sms.rs:174-186` (SmsProviderBuilder::build)
- **description:** The Twilio endpoint URL is hardcoded at sms.rs:51-52. There is no support for Twilio's `MessagingServiceSid` (used for sending from a pool of numbers) or for sending to a `from` that is a messaging-service sid. Spec § "Channel::Sms" allows `from: Option<PhoneNumber>`, but Twilio's messaging services accept a `MessagingServiceSid` instead of a `From:` phone number, which the reference provider cannot express.
- **expected:** Spec § "Channel::Sms" / Twilio integration: support messaging services.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/sms.rs:50-52
  const DEFAULT_GATEWAY_URL: &str =
      "https://api.twilio.com/2010-04-01/Accounts/{account}/Messages.json";
  // crates/adapters/notify/src/sms.rs:308-319 (dispatch posts a fixed form with To/From/Body)
  .form(&[("To", to.as_str()), ("From", from.as_str()), ("Body", rendered.as_str())])
  ```

---

### FINDING 57

- **id:** ADAPTER-NOT-057
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/notify/src/sms.rs:311` (HTTP POST without timeout)
- **description:** `SmsProvider::dispatch` builds an HTTP POST with no timeout. `reqwest::Client::builder().build()` (sms.rs:178) constructs the client with the default 30-second timeout — but there is no request-level timeout and no retry policy. A hung gateway connection blocks a worker for up to 30 seconds per request, and `send_bulk` multiplies this by recipient count.
- **expected:** Explicit `timeout(Duration::from_secs(...))` on the per-request builder.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/sms.rs:308-319
  let response = self.http.post(&self.gateway_url)
      .header("Authorization", self.basic_auth_header())
      .form(&[("To", to.as_str()), ("From", from.as_str()), ("Body", rendered.as_str())])
      .send().await
      .map_err(NotificationError::infrastructure)?;
  ```

---

### FINDING 58

- **id:** ADAPTER-NOT-058
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/notify/src/email.rs:88-140` and `crates/adapters/notify/src/sms.rs:296-340`
- **description:** Neither provider implements a `RateLimited` retry path. The `NotificationError::RateLimited` variant exists (`errors.rs:96`) but no adapter ever constructs it. Spec § "Rate Limiting": "The adapter returns `NotificationError::RateLimited` when a limit is hit; the engine retries with backoff." There is no `try_acquire` call in either provider; no error path returns `RateLimited`.
- **expected:** Spec § "Rate Limiting".
- **evidence:**
  ```rust
  // grep -nE "RateLimited" crates/adapters/notify/src/email.rs
  // 0 matches (only the enum variant itself at errors.rs:96)
  // grep -nE "RateLimited" crates/adapters/notify/src/sms.rs
  // 0 matches
  ```

---

### FINDING 59

- **id:** ADAPTER-NOT-059
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/notify/src/sms.rs:308-333` (SmsProvider::dispatch)
- **description:** `SmsProvider::dispatch` only recognizes HTTP 202 (Queued) and 2xx (Sent) as success. It treats every 4xx and 5xx as `NotificationError::provider`. The spec § "DeliveryStatus" has a `Failed { reason, retryable }` variant where `retryable` distinguishes transient 5xx and rate-limit responses from permanent 4xx. The provider cannot return `Failed { retryable: true }` for a 429 (Twilio throttle), nor `Failed { retryable: false }` for a 21610 (unsubscribed recipient).
- **expected:** Spec § "DeliveryStatus": `Failed { retryable: bool }`.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/sms.rs:325-333
  let status = if status_code.as_u16() == 202 {
      DeliveryStatus::Queued
  } else if status_code.is_success() {
      DeliveryStatus::Sent
  } else {
      return Err(NotificationError::provider(format!(
          "sms gateway returned status {status_code}"
      )));
  };
  ```

---

### FINDING 60

- **id:** ADAPTER-NOT-060
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/notify/src/email.rs:367-395` (build_lettre_message)
- **description:** `build_lettre_message` builds a `lettre::Message::builder().body(body.to_owned())`. The hardcoded subject is `"Educore notification"` (line 383). The template's subject (per spec § "Templates": "subject (for email and push)") is ignored — `TemplateRef` carries no subject field, and the email provider never queries the communication-domain template store. All sent emails have the same subject line.
- **expected:** Spec § "Templates": "A subject (for email and push)."
- **evidence:**
  ```rust
  // crates/adapters/notify/src/email.rs:380-394
  let mut builder = Message::builder()
      .from(from_mailbox)
      .to(to_mailbox)
      .subject("Educore notification");
  ```

---

### FINDING 61

- **id:** ADAPTER-NOT-061
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/notify/src/services.rs:209-241` (channel_key) and `crates/adapters/notify/src/services.rs:520` (try_acquire)
- **description:** `RateLimitService::try_acquire` keys the bucket by `channel_key(channel)`, which for `Channel::Email` includes `from` and `reply_to` (services.rs:211-215). Two sends with the same channel kind but different `from` addresses (e.g. sender A on row 1 and sender B on row 2) get separate buckets. Spec § "Rate Limiting": "per-channel rate limits" — keyed on the channel kind, not the per-request envelope.
- **expected:** Spec § "Rate Limiting": per-channel, per-tenant limits keyed on channel kind.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/services.rs:209-219
  Channel::Email { from, reply_to } => {
      let from = from.as_ref().map_or("-", EmailAddress::as_str);
      let reply = reply_to.as_ref().map_or("-", EmailAddress::as_str);
      format!("email:{from}:{reply}")
  }
  Channel::Sms { from, unicode } => {
      let from = from.as_ref().map_or("-", PhoneNumber::as_str);
      format!("sms:{from}:{unicode}")
  }
  ```
  Bucket key varies with envelope fields, not channel kind.

---

### FINDING 62

- **id:** ADAPTER-NOT-062
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/notify/src/port.rs:508-557` (SecretString)
- **description:** `SecretString` (port.rs:508-557) is defined in the port but never used by either provider. `EmailProvider` stores `credentials_user: Option<String>` as a plain `String` (email.rs:206) — the password half is `String::new()` (email.rs:273), and the credential is logged through the `Credentials::new` constructor. `SmsProvider` stores `api_key: String` (sms.rs:74) and only redacts it in `Debug` (sms.rs:86). Spec § "Webhook" / general hygiene: secrets should be wrapped at the port boundary.
- **expected:** Adapter credential fields use `SecretString`.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/email.rs:204-274
  pub struct EmailProviderBuilder {
      ...
      credentials_user: Option<String>,
      ...
  }
  // crates/adapters/notify/src/email.rs:272-274
  if let Some(user) = self.credentials_user {
      builder = builder.credentials(Credentials::new(user, String::new()));
  }
  ```
  Plain `String`; never wrapped in `SecretString`.

---

### FINDING 63

- **id:** ADAPTER-NOT-063
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/notify/src/services.rs:519-551` (RateLimitService::try_acquire) and `crates/adapters/notify/src/services.rs:530-543` (refill math)
- **description:** `RateLimitService::try_acquire` (services.rs:520) sets `entry.last_refill = entry.last_refill.checked_add(Duration::from_millis(u64::from(new_tokens) * 1000))` on refill (line 539-542). This advances `last_refill` by the integer-second count, not by the actual elapsed time. The next call then computes `elapsed = now - last_refill` — but since the bucket has been fully drained and refilled at the *expected* rate, this can leak tokens (the bucket effectively starts a new second-clock from the last refill point, so partial-second drift accumulates). With `max_per_second = 1` and a request every 1100 ms, the second request gets a token that should have been withheld.
- **expected:** A standard token-bucket implementation tracks `last_refill` as the time the bucket was last touched and refills `(now - last_refill) * rate` tokens; the `last_refill` should always be set to `now`, not advanced.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/services.rs:530-543
  let elapsed_ms =
      u64::try_from(now.duration_since(entry.last_refill).as_millis()).unwrap_or(u64::MAX);
  let new_tokens = u32::try_from(elapsed_ms / 1000).unwrap_or(u32::MAX);
  if new_tokens > 0 {
      entry.tokens = entry.tokens.saturating_add(new_tokens).min(entry.max_tokens);
      entry.last_refill = entry.last_refill
          .checked_add(Duration::from_millis(u64::from(new_tokens) * 1000))
          .unwrap_or(now);
  }
  ```

---

### FINDING 64

- **id:** ADAPTER-NOT-064
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/notify/src/port.rs:1156-1192` (SendNotification)
- **description:** `SendNotification` does not derive `Default`. The unit tests at `email.rs:482-499`, `sms.rs:586-601`, `sms.rs:614-631`, and `sms.rs:665-679` all hand-construct the struct with 12 named fields. `notify_integration.rs` (the integration test file) does not construct a `SendNotification` at all. The lack of `Default` is a known ergonomic failure for the integration-test scaffolding but is also missing from `SendBulkNotification` (port.rs:1236-1261). Bulk recipients are constructed via `BulkRecipient::new(recipient)` (port.rs:1219-1225) which is fine, but the wrapper struct is unbuildable from defaults.
- **expected:** Either `Default` impl or a `SendNotification::builder(...)` builder pattern.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/port.rs:1157
  #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  pub struct SendNotification {
  ```
  No `Default` derive, no `new` constructor.

---

### FINDING 65

- **id:** ADAPTER-NOT-065
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/notify/src/services.rs:73-176` (sha256) and `crates/adapters/notify/src/sms.rs:489-517` (base64_encode)
- **description:** `services.rs` ships hand-rolled SHA-256 (services.rs:73-176) and `sms.rs` ships hand-rolled base64 (sms.rs:489-517). Both are documented as deviations from using standard crates because the task spec lists the manifest under "DO NOT TOUCH" (services.rs:31-41; sms.rs:21-25). The two crypto primitives are the foundation of idempotency-key derivation and HTTP auth — the highest-impact components of the port. A typo in either implementation silently produces wrong keys / wrong headers and is not detectable by any test in the crate (the integration test only checks the length and charset of the SHA-256 hex string at `notify_integration.rs:91-93`, not correctness).
- **expected:** Use the `sha2` crate for SHA-256 and the `base64` crate for base64.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/sms.rs:489-517
  fn base64_encode(input: &[u8]) -> String {
      const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
      let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
      let mut i = 0;
      while i + 3 <= input.len() {
          ...
      }
      ...
  }
  ```
  Hand-rolled base64.

---

### FINDING 66

- **id:** ADAPTER-NOT-066
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/notify/src/sms.rs:158-167` (SmsProviderBuilder::template_body)
- **description:** `SmsProviderBuilder::template_body` registers a template body in a process-local `HashMap<NotificationTemplateId, String>` (sms.rs:74-77, 109). Spec § "Templates": "Templates are owned by the communication domain." Production deployments cannot pre-register templates from the engine into a builder; the template store is in the database. The builder-as-template-store is a workaround that is only useful for tests.
- **expected:** Templates come from the communication domain; adapters resolve `TemplateRef::Id` against a shared template service.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/sms.rs:158-167
  pub fn template_body(mut self, id: NotificationTemplateId, body: impl Into<String>) -> Self {
      self.templates.insert(id, body.into());
      self
  }
  ```

---

### FINDING 67

- **id:** ADAPTER-NOT-067
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/notify/src/email.rs:380-394` (build_lettre_message)
- **description:** `build_lettre_message` constructs a `lettre::Message::builder().from(from_mailbox).to(to_mailbox).subject(...)`. It does not set a `Message-ID` header, does not set `Date`, does not set `MIME-Version`, and does not set `Content-Type` explicitly. The resulting email may be malformed or rejected by strict receiving MTAs. `lettre` is supposed to set `Date` automatically, but only for fully-built messages, and the `body()` call with a plain `String` produces `Content-Type: text/plain; charset=utf-8` — not the spec's "email body" which may need HTML.
- **expected:** Spec § "Channel::Email" supports both plain and HTML bodies; the adapter should set `Content-Type` accordingly.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/email.rs:380-394
  let mut builder = Message::builder()
      .from(from_mailbox)
      .to(to_mailbox)
      .subject("Educore notification");
  ...
  builder.body(body.to_owned())
      .map_err(|e| NotificationError::provider(format!("lettre failed to build body: {e}")))
  ```

---

### FINDING 68

- **id:** ADAPTER-NOT-068
- **area:** adapters
- **severity:** Low
- **location:** `crates/adapters/notify/src/sms.rs:174-186` (SmsProviderBuilder::build)
- **description:** `SmsProviderBuilder::build` returns `SmsProvider` (no `Result`), while `EmailProviderBuilder::build` returns `Result<EmailProvider>` (email.rs:261). The asymmetry is unjustified: `SmsProviderBuilder::build` should validate the `api_key` is set (it's allowed to be empty at construction) and the `gateway_url` is well-formed. As shipped, `SmsProviderBuilder::new()` constructs an empty-string `api_key`, then `.build()` returns a `SmsProvider` whose every `send` will fail with a `401 Unauthorized` at the gateway. A consumer cannot detect this at startup.
- **expected:** Consistent `Result<T, NotificationError>` builder.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/sms.rs:173-187
  pub fn build(self) -> SmsProvider {
      let gateway_url = self.gateway_url
          .unwrap_or_else(|| DEFAULT_GATEWAY_URL.replace("{account}", &self.api_key));
      let http = Client::builder().build().unwrap_or_else(|_| Client::new());
      SmsProvider {
          http, gateway_url, api_key: self.api_key,
          default_from: self.default_from, templates: self.templates,
      }
  }
  // crates/adapters/notify/src/email.rs:261
  pub fn build(self) -> Result<EmailProvider> {
  ```

---

### FINDING 69

- **id:** ADAPTER-NOT-069
- **area:** adapters
- **severity:** Low
- **location:** `crates/adapters/notify/src/port.rs:151-173` (BulkRecipientIndex) and `crates/adapters/notify/src/sms.rs:378`
- **description:** `BulkRecipientIndex` is a transparent `u32` newtype. The spec uses it for "the original input row index." In `SmsProvider::send_bulk` the index is computed from `receipt.total()` (sms.rs:377-378) which is wrong (ADAPTER-NOT-016); in `EmailProvider::send_bulk` it's computed from `enumerate()` (email.rs:180) which is correct. The mismatch between the two providers means the same bulk request gets different indices depending on which provider handles it.
- **expected:** Spec § "BulkReceipt": same `BulkRecipientIndex` semantics for every provider.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/email.rs:180
  let Ok(idx_u32) = u32::try_from(idx) else { continue; };
  receipt.failed.push((BulkRecipientIndex::new(idx_u32), e));
  // crates/adapters/notify/src/sms.rs:377-378
  let global_idx = receipt.total();
  let index = BulkRecipientIndex::new(u32::try_from(global_idx).unwrap_or(u32::MAX));
  ```

---

### FINDING 70

- **id:** ADAPTER-NOT-070
- **area:** adapters
- **severity:** Low
- **location:** `crates/adapters/notify/src/email.rs:99-100` (Channel::Sms/Email mismatch error)
- **description:** The error string in `EmailProvider::send` for non-email channels is `"email provider cannot send {other:?} channel"`. This embeds the entire `Channel` enum's `Debug` output, which for `Channel::Webhook { url, secret }` would include the webhook URL. The `SecretString` already redacts itself (port.rs:535-545), but `Url` is a plain `String`. PII / secret-bearing variants dump their state into the error message.
- **expected:** Error messages should never carry PII.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/email.rs:97-100
  return Err(NotificationError::provider(format!(
      "email provider cannot send {other:?} channel"
  )));
  ```

---

### FINDING 71

- **id:** ADAPTER-NOT-071
- **area:** adapters
- **severity:** Low
- **location:** `crates/adapters/notify/src/email.rs:103-105` (EmailProvider reply_to match)
- **description:** `EmailProvider::send` matches `Channel::Email { reply_to, .. }` separately after the `from` match (line 102-105) but does not check that the reply-to variant is actually being processed; if `request.channel` is `Channel::Sms { .. }`, the function still enters the `reply_to` block (it returns `None` because of the `_ => None` arm). The two-match-arm structure is unnecessary and produces an unreachable pattern in the non-email case. Cosmetic but indicates a rushed implementation.
- **expected:** Single destructuring of `Channel::Email { from, reply_to }`.
- **evidence:**
  ```rust
  // crates/adapters/notify/src/email.rs:91-105
  let from = match &request.channel {
      Channel::Email { from, .. } => from.as_ref()
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
  ```

---

### FINDING 72

- **id:** ADAPTER-NOT-072
- **area:** adapters
- **severity:** Low
- **location:** `crates/adapters/notify/src/services.rs:277-307` (TemplateService::substitute_variables)
- **description:** `TemplateService::substitute_variables` (services.rs:277-307) silently leaves placeholders whose variables are missing in the variables map. The docstring says (services.rs:271-276) "Placeholders whose name is not present in `variables` are left as-is (the caller is expected to have run `validate_required_variables` first)." If a caller forgets `validate_required_variables`, the placeholder leaks into the rendered body — potentially exposing the internal `{student_name}` syntax to end users. The SMS provider's `SmsProvider::render_template` (sms.rs:236-246) has the same issue with a different placeholder syntax (`{{name}}`).
- **expected:** Spec § "Templates": "The engine validates that all required variables are provided in `SendNotification::variables`. Missing variables fail the send."
- **evidence:**
  ```rust
  // crates/adapters/notify/src/services.rs:278-307
  pub fn substitute_variables(body: &str, variables: &BTreeMap<String, String>) -> String {
      let mut out = String::with_capacity(body.len());
      let bytes = body.as_bytes();
      let mut i = 0;
      while i < bytes.len() {
          if bytes[i] == b'{' {
              if let Some((end, name)) = scan_variable(&bytes[i + 1..]) {
                  if let Some(value) = variables.get(name) {
                      out.push_str(value);
                      i += 1 + end;
                      continue;
                  }
              }
          }
          ...
      }
      out
  }
  ```

---

### FINDING 73

- **id:** ADAPTER-NOT-073
- **area:** adapters
- **severity:** Low
- **location:** `crates/adapters/notify/src/services.rs:209-241` (channel_key) and `crates/adapters/notify/src/services.rs:520-551` (try_acquire)
- **description:** `channel_key` for `Channel::Webhook` includes `url.as_str()` in the bucket key (services.rs:236-239). Two webhook deliveries to different URLs get separate buckets; one URL gets `max_per_second` regardless of how many URLs the tenant has configured. The spec § "Rate Limiting" requires per-channel limits; the webhook is a single channel.
- **expected:** Spec § "Rate Limiting": per-channel, per-tenant limit (single bucket per channel kind per tenant).
- **evidence:**
  ```rust
  // crates/adapters/notify/src/services.rs:236-239
  Channel::Webhook { url, secret } => {
      let signed = if secret.is_some() { "1" } else { "0" };
      format!("webhook:{signed}:{}", url.as_str())
  }
  ```

---

### FINDING 74

- **id:** ADAPTER-NOT-074
- **area:** adapters
- **severity:** Low
- **location:** `crates/adapters/notify/Cargo.toml:21-32` and `crates/adapters/notify/src/lib.rs:1-60`
- **description:** The `Cargo.toml` carries a long comment block (lines 23-31) explaining that the previous commit didn't update dependencies, that `serde` and `thiserror` were added by a "B.3b" deviation, and that the consumer of these deps was the SMS reference implementation. This dev-level commentary in the manifest is unusual; the same dependencies are pulled in by `port.rs` and `errors.rs` for `Serialize`/`Deserialize` and `thiserror::Error` derives. The comment also references an internal microtask nomenclature ("B.3b", "the port+types owner") that doesn't appear in the rest of the repository. This is a code-hygiene drift: the manifest should describe the crate, not narrate the development history.
- **expected:** Clean Cargo.toml without in-manifest development narrative.
- **evidence:**
  ```toml
  # crates/adapters/notify/Cargo.toml:21-32
  # B.3b: HTTP gateway calls (Twilio / generic) for the SmsProvider reference impl.
  reqwest = { workspace = true }
  # B.3b (deviation): the `port.rs` + `errors.rs` files committed in the
  # previous Phase 15 step (`Phase 15: educore-notify port + types (B)`)
  # use `serde::{Deserialize, Serialize}` derives and `thiserror::Error`
  # but the commit did not update `Cargo.toml`. Without these the notify
  # crate cannot compile. Added here (in addition to `reqwest`) so the
  # SMS reference implementation has a crate to live in. A follow-up
  # commit by the port+types owner can move these into a normal dep
  # block once the orchestrator re-balances file ownership.
  serde = { workspace = true }
  thiserror = { workspace = true }
  ```

---

### END FINDINGS

**Total findings:** 74
