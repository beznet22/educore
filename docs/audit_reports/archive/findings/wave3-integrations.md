# Audit findings: educore-integrations (Phase 15 / adapters)

**Scope:** `crates/adapters/integrations/` (7 src files:
`lib.rs`, `port.rs`, `errors.rs`, `lms.rs`, `video.rs`,
`webhook_out.rs`, `services.rs`; 1 test file:
`tests/integrations_integration.rs`), `docs/ports/integrations.md`,
`docs/handoff/PHASE-15-HANDOFF.md`, `docs/code-standards.md`,
`AGENTS.md`.

**Total findings:** 42

---

### FINDING 1

- **id:** ADAPTER-INT-001
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/integrations/src/lms.rs:186-237`, `crates/adapters/integrations/src/video.rs:191-242`, `crates/adapters/integrations/src/webhook_out.rs:242-341`
- **description:** None of the three reference implementations
  (`LmsIntegration`, `VideoConferencingIntegration`,
  `WebhookOutIntegration`) read or use `IntegrationRequest::tenant`.
  The `invoke` bodies dispatch on `request.action.as_str()` and
  use the builder-supplied `api_key` / `secret` directly without
  consulting `tenant.school_id`. The port contract requires
  per-tenant configuration lookup.
- **expected:** `docs/ports/integrations.md:177-180` — "The
  `IntegrationConfig` value is loaded from the platform domain
  at startup. The engine passes `TenantContext` to the adapter;
  the adapter uses it to look up the config."
- **evidence:** `crates/adapters/integrations/src/lms.rs:187`
  ```rust
  async fn invoke(&self, request: IntegrationRequest) -> Result<IntegrationResponse> {
      let started = Instant::now();
      let action = request.action.as_str();
      let result = match action {
          ACTION_COURSE_CREATE => self.create_course(&request).await,
  ```
  `request.tenant` is never referenced; `self.api_key` is used
  unconditionally at `crates/adapters/integrations/src/lms.rs:276`
  (`.header("Authorization", self.auth_header())`). The same
  pattern is present in `crates/adapters/integrations/src/video.rs:192`
  and `crates/adapters/integrations/src/webhook_out.rs:243`.

---

### FINDING 2

- **id:** ADAPTER-INT-002
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/integrations/src/lms.rs:186-220`, `crates/adapters/integrations/src/video.rs:191-225`, `crates/adapters/integrations/src/webhook_out.rs:242-321`
- **description:** Zero audit-log calls exist in any of the three
  implementations. The `educore-audit` crate declares
  `AuditTarget::IntegrationConfig` and `AuditTarget::IntegrationInvocation`
  variants (added in Phase 15), but no code in
  `crates/adapters/integrations/` writes to the audit log. The
  port contract requires every invocation be recorded.
- **expected:** `docs/ports/integrations.md:195-200` ("Audit
  Logging") — "Every integration invocation is logged with tenant,
  integration, action, status, duration, and cost. Input and output
  are logged at DEBUG and may be redacted by the adapter." And
  `docs/ports/integrations.md:263-266` — "Every invocation, success
  or failure, is recorded with full metadata. Sensitive fields are
  redacted by the adapter."
- **evidence:** `grep -r 'audit\|AuditTarget\|record_integration\|IntegrationInvocation' crates/adapters/integrations/src/`
  returns no production matches outside doc comments. The
  `IntegrationInvocation` variant exists in
  `crates/cross-cutting/audit/src/writer.rs:442` but the
  integrations crate never imports `educore-audit` (verified by
  reading `crates/adapters/integrations/Cargo.toml:13-27`).

---

### FINDING 3

- **id:** ADAPTER-INT-003
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/integrations/src/video.rs:255-272`, `crates/adapters/integrations/src/video.rs:280-303`, `crates/adapters/integrations/src/video.rs:309-323`
- **description:** `VideoConferencingIntegration::auth_header()`
  returns the `api_secret` and forwards it in an `X-Api-Secret`
  HTTP header on every outbound request to Zoom, Google Meet,
  and Microsoft Teams. This sends the raw signing secret in the
  clear over the network on every API call; if TLS is terminated
  upstream (e.g. corporate proxy), the secret leaks to logs.
  The port contract requires Zoom JWT signing, not header forwarding.
- **expected:** `docs/ports/integrations.md:166-174` ("OAuth2
  Client Credentials") — "The adapter: 1. Stores the client_id
  and client_secret (per tenant). 2. Performs the OAuth2 token
  exchange. 3. Caches the token until expiry. 4. Refreshes the
  token before expiry." And the
  `crates/adapters/integrations/src/video.rs:14-22` doc comment
  itself states "Zoom JWT auth (the simplest Zoom integration):
  `api_key` is the Zoom API key; `api_secret` is used to sign a
  JWT with `HS256`."
- **evidence:** `crates/adapters/integrations/src/video.rs:255-257`
  ```rust
  fn auth_header(&self) -> (&'static str, String, String) {
      ("Bearer", self.api_key.clone(), self.api_secret.clone())
  }
  ```
  Sent as a header at `crates/adapters/integrations/src/video.rs:269`,
  `:298`, `:316`:
  ```rust
  .header("X-Api-Secret", secret)
  ```

---

### FINDING 4

- **id:** ADAPTER-INT-004
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/integrations/src/lms.rs:144-155`, `crates/adapters/integrations/src/video.rs:147-158`
- **description:** `LmsIntegrationBuilder::build()` and
  `VideoConferencingIntegrationBuilder::build()` default
  `api_key` (and `api_secret`) to empty strings via
  `unwrap_or_default()`. A consumer that forgets to call
  `.api_key(...)` builds a working client that authenticates with
  a blank bearer token to a live provider. There is no
  validation, no `Result` return, and no warning.
- **expected:** `docs/code-standards.md` § "Type Safety" — "No
  `unwrap()` or `expect()` in production paths. Propagate errors
  via `?` or document the invariant that makes panic impossible."
  Per AGENTS.md, `IntegrationConfig` per-tenant credentials must
  be loaded at startup (port contract § "Per-Tenant Configuration").
- **evidence:** `crates/adapters/integrations/src/lms.rs:150`
  ```rust
  api_key: self.api_key.unwrap_or_default(),
  ```
  `crates/adapters/integrations/src/video.rs:151-152`
  ```rust
  api_key: self.api_key.unwrap_or_default(),
  api_secret: self.api_secret.unwrap_or_default(),
  ```
  Test at `crates/adapters/integrations/src/lms.rs:591` confirms
  default: `assert_eq!(adapter.api_key, "");`.

---

### FINDING 5

- **id:** ADAPTER-INT-005
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/integrations/src/lms.rs:273-285`, `crates/adapters/integrations/src/lms.rs:338-345`, `crates/adapters/integrations/src/video.rs:265-277`, `crates/adapters/integrations/src/webhook_out.rs:213-230`
- **description:** None of the three reference impls invoke
  `RetryService::next_backoff`, `RetryService::should_retry`, or
  `RetryService::is_permanent_failure`. Every outbound HTTP call
  is a single shot — the adapter never re-issues a request on a
  transient failure (5xx, network, 408, 429). The
  `RetryService` exists in `services.rs:236` and has 11 unit
  tests, but is not wired into any impl. The port contract
  mandates retry orchestration.
- **expected:** `docs/ports/integrations.md:182-193` — "The
  adapter retries transient failures (5xx, network) per the
  policy. Permanent failures (4xx) are returned immediately."
- **evidence:** `crates/adapters/integrations/src/lms.rs:273-284`
  ```rust
  let response = self
      .http
      .post(&url)
      .header("Authorization", self.auth_header())
      ...
      .send()
      .await
      .map_err(infrastructure)?;
  parse_response(response).await
  ```
  Single `.send().await`; no retry loop. Same pattern at
  `crates/adapters/integrations/src/lms.rs:338-345` (roster sync),
  `crates/adapters/integrations/src/video.rs:265-277`,
  `:294-303`, `:312-322`, and
  `crates/adapters/integrations/src/webhook_out.rs:216-229`.
  `grep` for `RetryService` in impl files returns no production
  call sites.

---

### FINDING 6

- **id:** ADAPTER-INT-006
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/integrations/src/lms.rs:146`, `crates/adapters/integrations/src/video.rs:149`
- **description:** The `LmsIntegration` and
  `VideoConferencingIntegration` construct their HTTP client via
  `Client::new()`. With the workspace `reqwest` declaration
  (`default-features = false`, `features = ["rustls-tls", "json",
  "stream"]`), `Client::new()` does not set a request timeout.
  Every outbound call to Google Classroom / Zoom / Teams can hang
  indefinitely, blocking the engine's executor. Only
  `WebhookOutIntegrationBuilder::build()` (line 385-388) calls
  `Client::builder().timeout(...)`.
- **expected:** `docs/ports/integrations.md:222-237` ("Worked
  Example") — `IntegrationRequest::timeout` is documented as the
  per-call override; an adapter default must exist or the contract
  is meaningless. `docs/code-standards.md` § "Production-ready"
  ("Real schools, real students, real money").
- **evidence:** `crates/adapters/integrations/src/lms.rs:146`
  ```rust
  http: Client::new(),
  ```
  `crates/adapters/integrations/src/video.rs:149` identical.
  Compare with `crates/adapters/integrations/src/webhook_out.rs:385-388`
  which sets a 30 s timeout.

---

### FINDING 7

- **id:** ADAPTER-INT-007
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/integrations/src/lms.rs:187-220`, `crates/adapters/integrations/src/video.rs:192-225`
- **description:** `IntegrationRequest::timeout` is read from the
  port surface (`port.rs:264`) but never applied by the LMS or
  Video adapters. The WebhookOut adapter hard-codes a 30 s timeout
  in `HTTP_TIMEOUT_SECS` (`webhook_out.rs:101`) and ignores the
  per-call override. Per-call timeout is dead code in the engine.
- **expected:** `docs/ports/integrations.md:38` (`IntegrationRequest`)
  — `pub timeout: Option<Duration>` — "Optional per-call timeout
  override. `None` means 'use the adapter default'."
- **evidence:** `crates/adapters/integrations/src/lms.rs:187-220`:
  `request.timeout` is not referenced in `invoke`. The same is
  true in `crates/adapters/integrations/src/video.rs:191-225`.
  `grep 'request\.timeout' crates/adapters/integrations/src/` returns
  only matches in `errors.rs` (the variant doc).

---

### FINDING 8

- **id:** ADAPTER-INT-008
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/integrations/src/lms.rs:505-523`, `crates/adapters/integrations/src/video.rs:397-415`
- **description:** The shared `parse_response` helper maps a
  non-2xx response into `IntegrationError::Provider(format!("{}
  {}", status.as_u16(), body))`. The full response body — which
  for LMS roster sync, course create, and video meeting get/list
  contains student identifiers, names, emails, and meeting join
  URLs — is embedded in the error message. The error is then
  surfaced in `IntegrationResponse::error`, which the port spec
  says is "logged at DEBUG". Every PII field in the body is in
  the error string.
- **expected:** `docs/ports/integrations.md:195-200` ("Audit
  Logging") — "Input and output are logged at DEBUG and may be
  redacted by the adapter." And
  `docs/ports/integrations.md:263-266` — "Sensitive fields are
  redacted by the adapter."
- **evidence:** `crates/adapters/integrations/src/lms.rs:516-521`
  ```rust
  Err(IntegrationError::Provider(format!(
      "{} {}",
      status.as_u16(),
      body
  )))
  ```
  Identical code at `crates/adapters/integrations/src/video.rs:408-413`.
  `body` is the raw response body captured at line `:507`:
  ```rust
  let body = response.text().await.map_err(infrastructure)?;
  ```

---

### FINDING 9

- **id:** ADAPTER-INT-009
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/integrations/src/lms.rs:363-369`
- **description:** `LmsIntegration::sync_roster` builds the
  per-student error JSON as `{"user_id": user_id, "action": action,
  "error": err.to_string()}`. For a network/DNS error,
  `err.to_string()` includes the full target URL with course id
  and user id embedded, plus the reqwest internal context. This
  is the engine's per-student error payload — it propagates into
  the LMS rosters' error report and is the basis of any retry
  decision the LMS admin makes.
- **expected:** `docs/ports/integrations.md:195-200` — redaction
  requirement on adapter-emitted error text.
- **evidence:** `crates/adapters/integrations/src/lms.rs:363-369`
  ```rust
  Err(err) => {
      errors.push(serde_json::json!({
          "user_id": user_id,
          "action": action,
          "error": err.to_string(),
      }));
  }
  ```

---

### FINDING 10

- **id:** ADAPTER-INT-010
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/integrations/src/port.rs:283-308`
- **description:** Doc-vs-code drift: the port contract at
  `docs/ports/integrations.md:48-55` defines
  `IntegrationResponse.cost: Option<Money>` (a single shared type
  in `educore-core::value_objects`). The actual code at
  `crates/adapters/integrations/src/port.rs:283-308` defines
  `IntegrationResponse.cost: Option<IntegrationCost>` where
  `IntegrationCost` is a fresh local struct (`amount_minor: i64,
  currency: String`). Consumers following the spec get a type
  error.
- **expected:** `docs/ports/integrations.md:48-55` — `pub cost:
  Option<Money>`.
- **evidence:** `crates/adapters/integrations/src/port.rs:302`
  ```rust
  pub cost: Option<IntegrationCost>,
  ```
  with `IntegrationCost` defined at `:317-325`. No `Money` import
  exists in the port module (verified by reading
  `crates/adapters/integrations/src/port.rs:27-40`).

---

### FINDING 11

- **id:** ADAPTER-INT-011
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/integrations/src/port.rs:264`, `crates/adapters/integrations/src/port.rs:297`, `crates/adapters/integrations/src/port.rs:194-209`
- **description:** Doc-vs-code drift: the port contract types
  `IntegrationRequest::timeout: Option<Duration>`,
  `IntegrationResponse::duration: Duration`, and `RetryPolicy`
  `interval`/`base`/`max` fields all as `Duration` (std). The
  code types them as `chrono::Duration` (`ChronoDuration`). All
  three impls compute durations via
  `chrono::Duration::from_std(...)` and silently zero out
  negative or overflowing std durations.
- **expected:** `docs/ports/integrations.md:38` — `pub timeout:
  Option<Duration>`. `docs/ports/integrations.md:53` — `pub
  duration: Duration`. `docs/ports/integrations.md:184-190` —
  `Linear { max_retries: u32, interval: Duration }`,
  `Exponential { max_retries: u32, base: Duration, max: Duration }`.
- **evidence:** `crates/adapters/integrations/src/port.rs:31`
  ```rust
  use chrono::Duration as ChronoDuration;
  ```
  used at `:196`, `:206`, `:208`, `:264`, `:297`. The
  integration tests at
  `crates/adapters/integrations/tests/integrations_integration.rs:42-44`
  call this out in a comment ("`RetryPolicy::Exponential.base` and
  `.max` are `chrono::Duration` (not `std::time::Duration`)").

---

### FINDING 12

- **id:** ADAPTER-INT-012
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/integrations/src/webhook_out.rs:202-209`, `crates/adapters/integrations/src/services.rs:146-152`
- **description:** The HMAC-SHA256 signing helper is implemented
  twice with identical semantics: once as the public associated
  function `WebhookOutIntegration::compute_signature` and once
  as `WebhookSignatureService::compute_signature`. The two
  implementations diverge only in error type — the impl-level
  version uses `.expect("HMAC accepts any key length")` and
  returns `String`; the service-level version uses
  `IntegrationError::Infrastructure(...)` and returns
  `Result<String>`. Consumers must choose between two APIs that
  do the same thing; the prelude re-exports the service-level
  helper but `webhook_out.rs::compute_signature` is also `pub`.
- **expected:** AGENTS.md § "Module Layout" — single source of
  truth per operation.
- **evidence:** `crates/adapters/integrations/src/webhook_out.rs:202-209`
  ```rust
  pub fn compute_signature(secret: &str, payload: &[u8]) -> String {
      let mut mac =
          HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key length");
      mac.update(payload);
      let bytes = mac.finalize().into_bytes();
      format!("sha256={}", hex_encode(&bytes))
  }
  ```
  vs. `crates/adapters/integrations/src/services.rs:146-152`
  ```rust
  pub fn compute_signature(secret: &str, payload: &[u8]) -> Result<String> {
      let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
          .map_err(|e| IntegrationError::Infrastructure(format!("HMAC key error: {e}").into()))?;
      ...
  }
  ```
  Both have separate `hex_encode` helpers
  (`webhook_out.rs:401-409` and `services.rs:442-450`) and
  identical `HmacSha256` type aliases (`:107` and `:46`).

---

### FINDING 13

- **id:** ADAPTER-INT-013
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/integrations/src/webhook_out.rs:202-209`, `crates/adapters/integrations/src/webhook_out.rs:383-393`
- **description:** `WebhookOutIntegration::compute_signature` and
  `WebhookOutIntegrationBuilder::build` both call
  `.expect(...)` in production paths. The crate's lib.rs denies
  `expect_used` (workspace lint, line 269 of root Cargo.toml) but
  these production sites use `#[allow(clippy::expect_used)]` to
  bypass it. AGENTS.md forbids `expect()` in production code.
- **expected:** `AGENTS.md` § "Type Safety" — "No `unwrap()` or
  `expect()` in production paths. Propagate errors via `?` or
  document the invariant that makes panic impossible." And
  `crates/adapters/integrations/src/services.rs:146-148` already
  shows the correct error-mapping pattern.
- **evidence:** `crates/adapters/integrations/src/webhook_out.rs:202`
  ```rust
  #[allow(clippy::expect_used)]
  pub fn compute_signature(secret: &str, payload: &[u8]) -> String {
      let mut mac =
          HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key length");
  ```
  `crates/adapters/integrations/src/webhook_out.rs:383-388`
  ```rust
  #[allow(clippy::expect_used)]
  pub fn build(self) -> WebhookOutIntegration {
      let http = Client::builder()
          .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT_SECS))
          .build()
          .expect("reqwest client construction with a valid timeout cannot fail");
  ```

---

### FINDING 14

- **id:** ADAPTER-INT-014
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/integrations/src/port.rs:65-75`, `crates/adapters/integrations/src/port.rs:108-120`
- **description:** `IntegrationId::new` and `IntegrationAction::new`
  are infallible constructors that accept any `String`. There is
  no validation (lowercase, ASCII, charset, length, no control
  characters). The library's `port.rs:62-64` doc explicitly says
  "Adapters that need validation (e.g. 'must be ASCII lowercase')
  perform it inside their `list_capabilities()`-driven config
  load, not at every construction site." But no adapter performs
  that validation either — the builders accept arbitrary strings
  via `provider: impl Into<String>`. A consumer can register an
  integration as `"   ZOOM  "` or `"Twilio/Calendar\nmalicious"`;
  every capability lookup and audit key uses the raw string.
- **expected:** `docs/ports/integrations.md:41-43` — "IntegrationId
  is a typed enum or string identifier for the integration."
  The reference to "typed enum" implies a closed-set validator;
  the impl defers validation entirely.
- **evidence:** `crates/adapters/integrations/src/port.rs:66-68`
  ```rust
  pub fn new(s: impl Into<String>) -> Self {
      Self(s.into())
  }
  ```
  `crates/adapters/integrations/src/lms.rs:120` —
  `.provider(provider: impl Into<String>)`. No normalization or
  validation occurs anywhere in the impls.

---

### FINDING 15

- **id:** ADAPTER-INT-015
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/integrations/src/lms.rs:230-236`, `crates/adapters/integrations/src/video.rs:235-241`, `crates/adapters/integrations/src/webhook_out.rs:334-340`
- **description:** `health()` on all three implementations is
  faked. `LmsIntegration::health()` returns
  `HealthStatus::Healthy` with `Timestamp::now()`; the Video
  adapter returns `Healthy` with `Timestamp::epoch()`; the
  WebhookOut returns `Healthy` with `Timestamp::now()`. None of
  the three performs an actual liveness probe of the upstream
  provider. A provider outage is undetectable until the next
  actual call fails — defeating the operational dashboards the
  port contract relies on.
- **expected:** `docs/ports/integrations.md:21-22` — `health()`
  returns `IntegrationHealth`. `docs/ports/integrations.md:489-491`
  (port.rs doc) — "Report liveness of the gateway and every
  registered integration. Called by the engine's operational
  dashboards every 30 s."
- **evidence:** `crates/adapters/integrations/src/lms.rs:230-236`
  ```rust
  async fn health(&self) -> Result<IntegrationHealth> {
      Ok(IntegrationHealth {
          status: HealthStatus::Healthy,
          last_checked_at: educore_core::value_objects::Timestamp::now(),
          message: None,
      })
  }
  ```
  Identical pattern (modulo the epoch vs now) in
  `crates/adapters/integrations/src/video.rs:235-241` and
  `crates/adapters/integrations/src/webhook_out.rs:334-340`.

---

### FINDING 16

- **id:** ADAPTER-INT-016
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/integrations/src/lms.rs:222-228`, `crates/adapters/integrations/src/video.rs:227-233`
- **description:** The `LmsIntegration::list_capabilities` and
  `VideoConferencingIntegration::list_capabilities` hard-code
  three capability rows. There is no way to add, remove, or
  override capabilities at runtime. The WebhookOut integration
  returns one row. The port contract says UIs and AI-agent tool
  catalogs depend on this method to render dynamic forms;
  shipping a static list means consumer UIs cannot expose a
  capability that the adapter does not yet know about.
- **expected:** `docs/ports/integrations.md:69-80` —
  "IntegrationCapability... The engine can enumerate capabilities
  at runtime for UIs and AI agent tool catalogs."
- **evidence:** `crates/adapters/integrations/src/lms.rs:222-228`
  ```rust
  async fn list_capabilities(&self) -> Result<Vec<IntegrationCapability>> {
      Ok(vec![
          self.capability_course_create(),
          self.capability_roster_sync(),
          self.capability_submissions_pull(),
      ])
  }
  ```

---

### FINDING 17

- **id:** ADAPTER-INT-017
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/integrations/src/lms.rs:449-487`
- **description:** All three LMS capabilities (`lms.course.create`,
  `lms.roster.sync`, `lms.submissions.pull`) list
  `vec![Capability::LmsRosterSync]` as their required capability.
  Creating a course and pulling submissions are conceptually
  distinct operations from syncing rosters and should map to
  distinct capabilities. The current scheme means an RBAC role
  permitted only `LmsRosterSync` can also create courses and
  pull submissions, while a role that should be able to pull
  submissions but not sync rosters has no way to express that.
- **expected:** `docs/ports/integrations.md:404-409` —
  `IntegrationCapability::required_capabilities: Vec<Capability>`
  is the engine's per-action RBAC hook; action ↔ capability
  mapping must be one-to-one and discriminative.
- **evidence:** `crates/adapters/integrations/src/lms.rs:457`
  ```rust
  required_capabilities: vec![Capability::LmsRosterSync],
  ```
  `:471` and `:485` are identical. The round-trip test at
  `:637-640` enforces this with a hard-coded assertion:
  ```rust
  assert_eq!(
      cap.required_capabilities,
      vec![Capability::LmsRosterSync],
      "every LMS capability must require LmsRosterSync"
  );
  ```

---

### FINDING 18

- **id:** ADAPTER-INT-018
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/integrations/src/video.rs:235-241`, `crates/adapters/integrations/src/port.rs:425-432`
- **description:** `VideoConferencingIntegration::health()`
  reports `last_checked_at: Timestamp::epoch()`. The port
  contract at `crates/adapters/integrations/src/port.rs:428-431`
  states "Never None — adapters that have never run a probe
  report Timestamp::epoch() so consumers can render 'never'
  explicitly." But the Video adapter's health endpoint always
  reports epoch even after the integration has been used for
  years — consumers cannot distinguish "never probed" from
  "probe at 1970-01-01". The LMS and WebhookOut adapters report
  `Timestamp::now()` always, masking "never probed" with the
  wall-clock time.
- **expected:** `crates/adapters/integrations/src/port.rs:425-431`
  — "adapters that have never run a probe report Timestamp::epoch()
  so consumers can render 'never' explicitly."
- **evidence:** `crates/adapters/integrations/src/video.rs:235-241`
  ```rust
  async fn health(&self) -> Result<IntegrationHealth> {
      Ok(IntegrationHealth {
          status: HealthStatus::Healthy,
          last_checked_at: educore_core::value_objects::Timestamp::epoch(),
          message: None,
      })
  }
  ```
  `crates/adapters/integrations/src/lms.rs:230-236` and
  `crates/adapters/integrations/src/webhook_out.rs:334-340`
  report `Timestamp::now()`.

---

### FINDING 19

- **id:** ADAPTER-INT-019
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/integrations/src/services.rs:250-276`
- **description:** `RetryService::next_backoff` silently swallows
  negative or overflowing `chrono::Duration` values via
  `unwrap_or(Duration::from_secs(1))` and `unwrap_or(Duration::from_secs(30))`.
  A caller who configures `base: ChronoDuration::seconds(-5)` or
  `max: ChronoDuration::max_value()` gets the documented default
  values with no error or warning. This makes per-tenant retry
  policy misconfiguration undetectable.
- **expected:** `docs/ports/integrations.md:182-193` ("Retry
  Policy") — adapter must apply the configured policy faithfully;
  silent substitution violates the contract.
- **evidence:** `crates/adapters/integrations/src/services.rs:271-273`
  ```rust
  let base_std = chrono_to_std(base).unwrap_or(Duration::from_secs(1));
  let max_std = chrono_to_std(max).unwrap_or(Duration::from_secs(30));
  Some(exponential_backoff(base_std, max_std, attempt))
  ```
  `chrono_to_std` (`:468-473`) returns `None` for negative or
  overflowing values:
  ```rust
  fn chrono_to_std(d: ChronoDuration) -> Option<Duration> {
      if d < ChronoDuration::zero() {
          return None;
      }
      d.to_std().ok()
  }
  ```

---

### FINDING 20

- **id:** ADAPTER-INT-020
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/integrations/src/services.rs:479-488`
- **description:** `exponential_backoff` returns
  `Duration::from_nanos(u64::try_from(scaled).unwrap_or(0))`. When
  the saturated multiplication overflows `u64`, the function
  returns `Duration::ZERO` instead of the documented `max`. A
  retry loop calling `next_backoff` in this regime would
  busy-loop the integration.
- **expected:** AGENTS.md § "Type Safety" — "No `as` casts that
  truncate or lose data. Use `TryFrom` / `TryInto` with proper
  error handling."
- **evidence:** `crates/adapters/integrations/src/services.rs:486-487`
  ```rust
  let scaled = (base_nanos.saturating_mul(u128::from(factor))).min(max_nanos);
  Duration::from_nanos(u64::try_from(scaled).unwrap_or(0))
  ```
  The `attempt >= 64` guard at `:480-482` covers the shift, but
  not the multiply or the cast.

---

### FINDING 21

- **id:** ADAPTER-INT-021
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/integrations/src/services.rs:358-391`, `crates/adapters/integrations/src/lms.rs:186-237`, `crates/adapters/integrations/src/video.rs:191-242`
- **description:** `RateLimitService::try_acquire` is defined
  and tested (`services.rs:715-768`) but no integration impl
  ever calls it. The LMS and Video adapters have no
  rate-limiting gate at all; the per-call burst from a malicious
  or buggy consumer can hit the provider's 429 immediately.
- **expected:** `docs/ports/integrations.md:60` — `RateLimited`
  status is in the contract; the port spec assumes adapters
  throttle proactively per the provider's quota.
- **evidence:** `grep 'RateLimitService\|try_acquire' crates/adapters/integrations/src/lms.rs crates/adapters/integrations/src/video.rs crates/adapters/integrations/src/webhook_out.rs`
  returns no production call sites. The service is only exercised
  by the unit test at `services.rs:715-768` and the integration
  test at `tests/integrations_integration.rs:125-134`.

---

### FINDING 22

- **id:** ADAPTER-INT-022
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/integrations/src/lms.rs:505-545`, `crates/adapters/integrations/src/video.rs:397-437`
- **description:** The `parse_response`, `infrastructure`,
  `json_infrastructure`, and `status_from_error` helpers are
  duplicated byte-for-byte between `lms.rs:505-545` and
  `video.rs:397-437`. There is no shared `http` helper module
  in `services.rs`. The same code is also duplicated in
  `webhook_out.rs` with a slightly different shape. A bug fix
  in one copy would silently miss the others.
- **expected:** AGENTS.md § "Module Layout" — single source of
  truth per operation. Per the `PortAdapters` precedent in
  `crates/adapters/auth/`, shared helpers go in `services.rs`.
- **evidence:** `crates/adapters/integrations/src/lms.rs:527-529`
  ```rust
  fn infrastructure(err: reqwest::Error) -> IntegrationError {
      IntegrationError::Infrastructure(Box::new(err))
  }
  ```
  Identical at `crates/adapters/integrations/src/video.rs:419-421`.
  `crates/adapters/integrations/src/lms.rs:539-545` and
  `crates/adapters/integrations/src/video.rs:431-437` are
  byte-identical `status_from_error` impls.

---

### FINDING 23

- **id:** ADAPTER-INT-023
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/integrations/Cargo.toml:25`
- **description:** `indexmap = { workspace = true }` is declared
  in `crates/adapters/integrations/Cargo.toml:25` but is not
  used anywhere in the crate (the `BTreeMap`/`HashMap` usages
  don't import it). Unused dependency adds to compile time and
  binary size, and signals the crate was authored with an
  incomplete mental model.
- **expected:** AGENTS.md § "Package Manager" — use `cargo add`
  and prune unused deps.
- **evidence:** `grep -rn 'indexmap\|IndexMap' crates/adapters/integrations/src/`
  returns no matches. Workspace declaration at
  `crates/adapters/integrations/Cargo.toml:25`. `cargo build
  --package educore-integrations` succeeds without it.

---

### FINDING 24

- **id:** ADAPTER-INT-024
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/integrations/src/webhook_out.rs:120-142`, `crates/adapters/integrations/src/webhook_out.rs:347-394`
- **description:** `WebhookTarget::url` is a `String`, not a
  parsed `Url`. The builder does not validate the URL on
  construction; it accepts `"not-a-url"`, `"file:///etc/passwd"`,
  `"http://"`, or any other malformed string. The
  `webhook_out.rs:351-353` doc comment acknowledges this
  ("builder does not validate the URL syntax — that's deferred
  to the first invoke() call so misconfiguration surfaces at
  dispatch time, not at wiring time"), but it also means a
  caller who wires 100 webhook targets cannot tell at startup
  which are broken.
- **expected:** `docs/ports/integrations.md:136-143` — the spec
  example uses `Url::parse("https://school.example.com/hooks/educore")?`
  (a `url::Url`, not a `String`). The port contract requires
  URL validation at construction.
- **evidence:** `crates/adapters/integrations/src/webhook_out.rs:130`
  ```rust
  pub url: String,
  ```
  `crates/adapters/integrations/src/webhook_out.rs:369`
  ```rust
  pub fn target(mut self, target: WebhookTarget) -> Self {
      self.targets.push(target);
      self
  }
  ```
  No validation anywhere. The `url` crate is not in the
  integrations `Cargo.toml`.

---

### FINDING 25

- **id:** ADAPTER-INT-025
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/integrations/src/webhook_out.rs:213-230`
- **description:** `WebhookOutIntegration::deliver` wraps the
  transport error in `IntegrationError::Infrastructure(Box::new(std::io::Error::other(format!(
  "webhook POST to {} failed: {e}",
  target.url
  ))))`. This double-wraps the original error (it was already a
  `reqwest::Error`, not an `io::Error`), discards the typed error
  variant, and constructs a new `io::Error` from the formatted
  message. The `reqwest::Error` chain is dropped; the caller
  cannot distinguish timeout from TLS handshake failure from DNS
  error from connection reset. They all become
  `Infrastructure("webhook POST to ... failed: ...")`.
- **expected:** `docs/ports/integrations.md:71-75` (errors.rs doc)
  — "Infrastructure — the adapter could not reach the provider at
  all (DNS, TCP, TLS, serialization). Carries the underlying
  error as a `source` for diagnostic logging." The original
  error should be preserved, not stringified.
- **evidence:** `crates/adapters/integrations/src/webhook_out.rs:222-229`
  ```rust
  .send()
  .await
  .map_err(|e| {
      IntegrationError::Infrastructure(Box::new(std::io::Error::other(format!(
          "webhook POST to {} failed: {e}",
          target.url
      ))))
  })
  ```
  Compare with the LMS / Video path at `lms.rs:283`:
  ```rust
  .map_err(infrastructure)?
  ```
  where `infrastructure` (`:527-529`) preserves the original
  `reqwest::Error`.

---

### FINDING 26

- **id:** ADAPTER-INT-026
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/integrations/src/webhook_out.rs:213-230`, `crates/adapters/integrations/src/webhook_out.rs:243-321`
- **description:** The webhook-out fan-out iterates targets
  serially. With N targets at HTTP_TIMEOUT_SECS = 30 s each, a
  dispatch can take 30·N seconds end-to-end. No parallelism,
  no timeout budget, no abort-on-first-error option. A single
  slow target stalls the entire batch; a single hanging target
  hits the 30 s timeout, then the next target starts.
- **expected:** `docs/ports/integrations.md` does not specify
  concurrency, but per AGENTS.md "Production-ready. Real schools,
  real students, real money." — 100 webhooks serially is not
  production-ready.
- **evidence:** `crates/adapters/integrations/src/webhook_out.rs:256-278`
  ```rust
  for target in &self.targets {
      if !target.matches(&request.action) {
          continue;
      }

      match self.deliver(target, &payload).await {
          ...
      }
  }
  ```
  No `futures::join_all`, `try_join_all`, or any bounded
  concurrency primitive. `futures` is a workspace dep
  (`Cargo.toml:100`) but unused in this file.

---

### FINDING 27

- **id:** ADAPTER-INT-027
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/integrations/src/webhook_out.rs:213-321`
- **description:** `WebhookOutIntegration::deliver` does not
  forward `request.correlation_id` or `request.idempotency_key`
  to the receiver. LMS and Video do set `X-Correlation-Id` and
  `Idempotency-Key` headers on every request; the webhook out
  adapter sets neither. The port contract requires the
  correlation id to be copied into "every outbound HTTP header
  (`X-Correlation-Id`)".
- **expected:** `crates/adapters/integrations/src/port.rs:257-260`
  — "Correlation id for log stitching across the engine. The
  adapter copies it into every outbound HTTP header
  (`X-Correlation-Id`) and every audit log entry."
- **evidence:** `crates/adapters/integrations/src/webhook_out.rs:213-230`
  ```rust
  async fn deliver(&self, target: &WebhookTarget, payload: &[u8]) -> Result<reqwest::Response> {
      let signature = Self::compute_signature(&target.secret, payload);

      self.http
          .post(&target.url)
          .header(SIGNATURE_HEADER, signature)
          .header(reqwest::header::CONTENT_TYPE, "application/json")
          .body(payload.to_vec())
          ...
  ```
  No `request.correlation_id` or `request.idempotency_key` is
  passed in (the function signature does not even take the
  `IntegrationRequest`).

---

### FINDING 28

- **id:** ADAPTER-INT-028
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/integrations/src/lms.rs:434-446`, `crates/adapters/integrations/src/video.rs:328-340`, `crates/adapters/integrations/src/webhook_out.rs:282-296`, `crates/adapters/integrations/src/webhook_out.rs:306-320`
- **description:** `response_metadata` writes keys as
  `"x-correlation-id"` and `"idempotency-key"` (lowercase), but
  the HTTP headers sent on the wire are `"X-Correlation-Id"`
  and `"Idempotency-Key"` (HTTP title-case convention). The
  metadata BTreeMap is supposed to capture the wire-level
  identifiers for log stitching; the case mismatch means log
  search by header value finds nothing matching the metadata
  key.
- **expected:** `crates/adapters/integrations/src/port.rs:303-307`
  — "Provider-specific metadata (request id, rate-limit
  remaining, traceparent, etc.). Always non-empty for a response
  that actually reached the provider."
- **evidence:** `crates/adapters/integrations/src/lms.rs:436-442`
  ```rust
  metadata.insert(
      "x-correlation-id".to_owned(),
      request.correlation_id.to_string(),
  );
  metadata.insert(
      "idempotency-key".to_owned(),
      request.idempotency_key.to_string(),
  );
  ```
  Compare with the HTTP header at `lms.rs:278`:
  ```rust
  .header("X-Correlation-Id", request.correlation_id.to_string())
  ```

---

### FINDING 29

- **id:** ADAPTER-INT-029
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/integrations/src/port.rs:186-220`, `crates/adapters/integrations/src/port.rs:212-220`
- **description:** `RetryPolicy::Default::default()` is
  implemented inline on the enum and returns
  `RetryPolicy::Exponential { max_retries: 3, base: seconds(1),
  max: seconds(30) }`. The doc-test comment in
  `docs/ports/integrations.md:217-237` and the worked example
  do not specify a default. A consumer who constructs a config
  via `..Default::default()` will silently use exponential backoff
  with 3 retries when the provider might require a different
  policy (e.g. Zoom's 5xx retry guidance, Stripe's aggressive
  rate-limit backoff).
- **expected:** `docs/ports/integrations.md:182-193` — the policy
  is part of the per-tenant config; the engine should pick it
  up from there, not apply a hard-coded engine-wide default.
- **evidence:** `crates/adapters/integrations/src/port.rs:212-220`
  ```rust
  impl Default for RetryPolicy {
      fn default() -> Self {
          Self::Exponential {
              max_retries: 3,
              base: ChronoDuration::seconds(1),
              max: ChronoDuration::seconds(30),
          }
      }
  }
  ```

---

### FINDING 30

- **id:** ADAPTER-INT-030
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/integrations/src/lms.rs:75-89`, `crates/adapters/integrations/src/video.rs:71-83`
- **description:** The LMS and Video impls register two
  different identifiers for the same integration. The
  `LMS_INTEGRATION_ID` constant is `"lms"`, but
  `LmsIntegrationBuilder::build()` defaults the `provider` field
  to `"google_classroom"` (`:149`). A consumer who builds the
  adapter without overrides registers as `"lms"` for the
  audit/telemetry surface but as `"google_classroom"` for
  per-provider routing. `VIDEO_INTEGRATION_ID` is
  `"video_conferencing"` but the default provider is `"zoom"`.
- **expected:** `docs/ports/integrations.md:41-43` —
  "IntegrationId is a typed enum or string identifier for the
  integration." A single integration must have a single id.
- **evidence:** `crates/adapters/integrations/src/lms.rs:78`
  ```rust
  pub const LMS_INTEGRATION_ID: &str = "lms";
  ```
  vs. `crates/adapters/integrations/src/lms.rs:149`
  ```rust
  provider: self
      .provider
      .unwrap_or_else(|| "google_classroom".to_owned()),
  ```
  Used in `response_metadata` at `:443-444`:
  ```rust
  metadata.insert("provider".to_owned(), self.provider.clone());
  metadata.insert("integration".to_owned(), LMS_INTEGRATION_ID.to_owned());
  ```

---

### FINDING 31

- **id:** ADAPTER-INT-031
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/integrations/src/errors.rs:314-330`
- **description:** `errors.rs` contains a `#[allow(dead_code)]`
  function `_ensure_traits_used` whose only purpose is to
  silence unused-import warnings on `serde::de::{Deserialize,
  Deserializer, Visitor}` and `serde::ser::{Serialize, Serializer}`
  that are in fact used by the manual impls at `:172-308`. The
  function exists purely to satisfy the linter. It is dead code
  that masks a tooling issue.
- **expected:** AGENTS.md § "Type Safety" — "No `#[allow(dead_code)]`
  or `_var` prefixes to silence the compiler."
- **evidence:** `crates/adapters/integrations/src/errors.rs:314-330`
  ```rust
  #[allow(dead_code)]
  fn _ensure_traits_used() {
      fn _needs_de<T>(_: &T)
      where
          T: serde::de::DeserializeOwned,
      {
      }
      fn _needs_ser<T>(_: &T)
      where
          T: serde::Serialize,
      {
      }
      let _: fn(&IntegrationError) = |e| {
          _needs_de(e);
          _needs_ser(e);
      };
  }
  ```
  Preceded by a comment at `:310-313` admitting it
  ("Silence the unused-imports lint for the imports that turned
  out not to be needed once the manual impls above replaced the
  derives.").

---

### FINDING 32

- **id:** ADAPTER-INT-032
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/integrations/src/port.rs:281-308`, `crates/adapters/integrations/src/port.rs:283`
- **description:** `IntegrationResponse` uses
  `#[derive(Debug)]` without field-level redaction. `Debug`
  formatting of an `IntegrationResponse` will print `output:
  Some(<full JSON value including any PII or secret-like keys>)`
  and `error: Some(Provider("400 {body}"))`. AGENTS.md and
  `docs/code-standards.md` require sensitive fields be
  redacted; `Debug` is the surface that almost every Rust
  logging pipeline (`{:?}`, `tracing::debug!`, `eprintln!`,
  panic messages) consumes.
- **expected:** `docs/ports/integrations.md:263-266` — "Every
  invocation, success or failure, is recorded with full
  metadata. Sensitive fields are redacted by the adapter."
- **evidence:** `crates/adapters/integrations/src/port.rs:282`
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct IntegrationResponse {
      pub status: IntegrationStatus,
      pub output: Option<JsonValue>,
      pub error: Option<IntegrationError>,
  ```
  No manual `Debug` impl, no `#[derive]` `Debug`-with-skip, no
  redacting wrapper. Compare with the `IntegrationError` impl
  at `errors.rs:147-162` which carefully handles the lossy clone.

---

### FINDING 33

- **id:** ADAPTER-INT-033
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/integrations/src/port.rs:310-325`, `crates/adapters/integrations/src/port.rs:310-317`
- **description:** Doc-vs-code drift in the rustdoc of
  `IntegrationCost`: the doc at
  `crates/adapters/integrations/src/port.rs:312` reads
  ```rust
  /// Mirrors the finance domain's [`Money`](educore_core::value_objects::Timestamp)
  /// shape but is duplicated here so this crate does not need a
  ```
  The link target is `Timestamp`, not `Money` — a copy/paste
  bug. The link resolves to `Timestamp`, misleading readers
  into thinking the cost type mirrors a timestamp.
- **expected:** AGENTS.md § "Documentation" — public items must
  have accurate rustdoc.
- **evidence:** `crates/adapters/integrations/src/port.rs:310-313`
  ```rust
  /// Provider-side monetary cost of a single integration call.
  ///
  /// Mirrors the finance domain's [`Money`](educore_core::value_objects::Timestamp)
  /// shape but is duplicated here so this crate does not need a
  ```

---

### FINDING 34

- **id:** ADAPTER-INT-034
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/integrations/src/lms.rs:49-58`, `crates/adapters/integrations/src/video.rs:51-62`, `crates/adapters/integrations/src/webhook_out.rs:77-94`
- **description:** The `!#[allow(clippy::module_name_repetitions)]`
  attribute at the top of every impl module means the lints
  pass even though `LmsIntegration`,
  `VideoConferencingIntegration`, and `WebhookOutIntegration`
  are in modules of the same name. Per AGENTS.md, this attribute
  is used to suppress a noisy pedantic lint, but it also
  suppresses the signal that the module / type names collide
  with items in the prelude (`webhook_out::WebhookOutIntegration`
  re-exported as `WebhookOutIntegration`).
- **expected:** `docs/code-standards.md` § "Code Standards" —
  consistent module / item naming.
- **evidence:** `crates/adapters/integrations/src/lms.rs:48`
  `#![allow(clippy::module_name_repetitions)]` (same on
  `video.rs:47`, `webhook_out.rs` (no attribute — but
  `services.rs:26` has it)). `webhook_out.rs` imports from
  `crate::port::*` without the attribute because the prelude
  re-exports `WebhookOutIntegration`.

---

### FINDING 35

- **id:** ADAPTER-INT-035
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/integrations/src/webhook_out.rs:282-321`
- **description:** `WebhookOutIntegration::invoke` returns
  `IntegrationResponse { status: Failed, ... }` when **any**
  target fails — even if 99 of 100 targets delivered
  successfully. The `dispatched_targets` count is in metadata,
  not status. Consumers cannot tell a "partial success" from a
  complete failure. The port spec lists `IntegrationStatus`
  values `Success / Accepted / RateLimited / Failed / TimedOut`
  — `PartialSuccess` is not modelled.
- **expected:** `docs/ports/integrations.md:57-64` —
  `IntegrationStatus::Failed` is the only failure outcome; the
  current code collapses partial and total failures, leaving no
  way for the engine to decide whether to retry the failed
  targets.
- **evidence:** `crates/adapters/integrations/src/webhook_out.rs:282-297`
  ```rust
  if let Some(err) = last_error {
      let mut metadata = BTreeMap::new();
      metadata.insert("dispatched_targets".to_owned(), dispatched.to_string());
      if let Some(status) = last_status {
          metadata.insert("last_status".to_owned(), status.as_u16().to_string());
      }

      return Ok(IntegrationResponse {
          status: IntegrationStatus::Failed,
          ...
      });
  }
  ```

---

### FINDING 36

- **id:** ADAPTER-INT-036
- **area:** adapters
- **severity:** Low
- **location:** `crates/adapters/integrations/src/webhook_out.rs:594-605`
- **description:** `test_json_serialized_payload_is_byte_stable`
  asserts `serde_json::to_vec(&a) == serde_json::to_vec(&b)`
  for two equal `json!` values. This test passes only because
  `serde_json::Value` serializes `Map<String, Value>` using a
  `BTreeMap` internally (alphabetical key order). The test name
  implies payload stability for HMAC signing, but the underlying
  guarantee is from `serde_json`, not from this code. If
  `serde_json` ever changes its map ordering, all
  `WebhookSignatureService::verify_signature` calls for
  `IntegrationRequest::input` payloads break silently.
- **expected:** `docs/ports/integrations.md:144-146` — "The
  adapter signs the payload with HMAC-SHA256 and posts it. The
  receiver verifies the signature." — stability of the
  serialized form is a contract.
- **evidence:** `crates/adapters/integrations/src/webhook_out.rs:594-605`
  ```rust
  let a = json!({"event": "InvoicePaid", "amount_minor": 12500});
  let b = json!({"event": "InvoicePaid", "amount_minor": 12500});
  assert_eq!(
      serde_json::to_vec(&a).unwrap(),
      serde_json::to_vec(&b).unwrap()
  );
  ```

---

### FINDING 37

- **id:** ADAPTER-INT-037
- **area:** adapters
- **severity:** Low
- **location:** `crates/adapters/integrations/src/lms.rs:387-429`, `crates/adapters/integrations/src/lms.rs:387-429`
- **description:** `LmsIntegration::pull_submissions` returns
  the entire submissions list as-is from the LMS into the
  output `JsonValue`. The body shape can include student names,
  email addresses, submission text (the assignment answers),
  timestamps, and IP addresses. The integration contract says
  the engine translates each submission into an
  `OnlineExamSubmitted` event with a `Source::Lms` tag, but the
  `pull_submissions` impl just forwards the raw provider
  payload. There is no field-level filtering or redaction.
- **expected:** `docs/ports/integrations.md:96-98` — "Pulls
  assignment submissions from the LMS and emits
  `OnlineExamSubmitted` events with a `Source::Lms` tag." The
  engine is meant to control which fields flow into the
  aggregate, not the adapter.
- **evidence:** `crates/adapters/integrations/src/lms.rs:423-428`
  ```rust
  let body = parse_response(response).await?;
  Ok(serde_json::json!({
      "course_id": course_id,
      "coursework_id": coursework_id,
      "submissions": body,
  }))
  ```

---

### FINDING 38

- **id:** ADAPTER-INT-038
- **area:** adapters
- **severity:** Low
- **location:** `crates/adapters/integrations/src/webhook_out.rs:73-76`, `crates/adapters/integrations/src/webhook_out.rs:146-152`
- **description:** The webhook-out module docs claim "Webhook
  secrets are never written to logs, metrics, or the audit
  trail." But `WebhookTarget` derives neither `Display` nor
  `serde::Serialize`. If a future consumer serializes the
  `WebhookOutIntegration` (or any `WebhookTarget`) to JSON or
  a metrics pipeline, the `secret` field is included in clear
  text. The Debug impl redacts the secret at `webhook_out.rs:148`,
  but Display / Serialize are unprotected.
- **expected:** `docs/ports/integrations.md:263-266` — "Sensitive
  fields are redacted by the adapter." Coverage of `Debug` alone
  is not enough.
- **evidence:** `crates/adapters/integrations/src/webhook_out.rs:125`
  ```rust
  #[derive(Clone, PartialEq, Eq)]
  pub struct WebhookTarget {
  ```
  No `serde::Serialize`, no manual `Display`. `Debug` impl at
  `:144-152` redacts, but no other format / serialization layer
  does.

---

### FINDING 39

- **id:** ADAPTER-INT-039
- **area:** adapters
- **severity:** Low
- **location:** `crates/adapters/integrations/src/port.rs:75-93`
- **description:** `IntegrationId::From<&str>` and
  `From<String>` both `to_owned()` the input. For `From<&str>`,
  this requires an allocation; for `From<String>`, it moves.
  The port contract at `docs/ports/integrations.md:41-43` calls
  `IntegrationId` a "typed enum or string identifier". An enum
  representation would avoid the heap allocation on every
  construction. At the rate integration ids flow through the
  engine (one per request), this is a measurable hot-path
  cost.
- **expected:** AGENTS.md § "Production-ready. Real schools,
  real students, real money." — the type should match the
  closed-set nature of the domain.
- **evidence:** `crates/adapters/integrations/src/port.rs:83-93`
  ```rust
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
  ```

---

### FINDING 40

- **id:** ADAPTER-INT-040
- **area:** adapters
- **severity:** Low
- **location:** `crates/adapters/integrations/src/lib.rs:1-127`
- **description:** `cargo doc --no-deps --package educore-integrations`
  emits 20+ "unresolved link" warnings. Every module-level doc
  comment in `lib.rs` uses intra-doc links to items that are
  not yet in scope at the link site (e.g.,
  `[`IntegrationGateway`](port::IntegrationGateway)` at `:41`
  before the module `port` is declared). Doc quality suffers;
  the rendered docs.rs page will not show the intended
  cross-references.
- **expected:** `AGENTS.md` § "Documentation" — public items must
  have accurate, well-formed rustdoc.
- **evidence:** Running `cargo doc --no-deps --package
  educore-integrations` (output captured during audit) lists
  `unresolved link to IntegrationGateway`,
  `unresolved link to IntegrationError`,
  `unresolved link to IntegrationError::Provider`,
  `unresolved link to IntegrationError::Infrastructure`,
  `unresolved link to WebhookOutIntegration`,
  `unresolved link to WebhookTarget`,
  `unresolved link to ACTION_MEETING_CREATE`,
  `unresolved link to ACTION_MEETING_GET`,
  `unresolved link to ACTION_RECORDING_LIST`,
  `unresolved link to VideoConferencingIntegrationBuilder`,
  `unresolved link to ACTION_COURSE_CREATE`,
  `unresolved link to ACTION_ROSTER_SYNC`,
  `unresolved link to ACTION_SUBMISSIONS_PULL`,
  `unresolved link to LmsIntegrationBuilder`,
  `unresolved link to WebhookSignatureService`,
  `unresolved link to PollingService`,
  `unresolved link to RetryService`,
  `unresolved link to RateLimitService`.

---

### FINDING 41

- **id:** ADAPTER-INT-041
- **area:** adapters
- **severity:** Low
- **location:** `crates/adapters/integrations/src/lms.rs:417-419`
- **description:** `LmsIntegration::pull_submissions` hard-codes
  the `pageSize` query parameter name as a lowercase literal
  `"pageSize"`. This couples the adapter to Google Classroom's
  API; Microsoft Teams Education and Moodle use different
  parameter names (`page_size`, `limit`). The provider is
  configurable via the builder's `base_url`, but the
  query-param naming is not.
- **expected:** `docs/ports/integrations.md:87-97` ("LMS Sync") —
  "Google Classroom, Microsoft Teams for Education, Moodle" are
  all listed as in-scope providers. The reference impl is
  supposed to work across them.
- **evidence:** `crates/adapters/integrations/src/lms.rs:417-419`
  ```rust
  .query(&[("pageSize", page_size.to_string())])
  ```

---

### FINDING 42

- **id:** ADAPTER-INT-042
- **area:** adapters
- **severity:** Low
- **location:** `crates/adapters/integrations/tests/integrations_integration.rs:140-159`
- **description:** Both `#[ignore]`-d async tests
  (`integrations_integration_async_lms_roster_sync_mock` and
  `integrations_integration_async_webhook_out_dispatch_mock`)
  construct a builder and immediately drop the result. No
  network call, no assertion, no actual scenario exercised.
  Marking them `#[ignore = "requires
  EDUCORE_PORT_ADAPTER_E2E env var"]` implies they exist for an
  end-to-end environment that is not present in CI. The tests
  are documentation, not validation.
- **expected:** `AGENTS.md` § "Testing (TDD)" — "No dummy tests.
  Every test must validate a real-world scenario." The handoff
  doc claims these tests cover "LMS roster sync mock" and
  "webhook-out dispatch mock" — they construct nothing more
  than a builder.
- **evidence:** `crates/adapters/integrations/tests/integrations_integration.rs:140-159`
  ```rust
  #[tokio::test]
  #[ignore = "requires EDUCORE_PORT_ADAPTER_E2E env var"]
  async fn integrations_integration_async_lms_roster_sync_mock() {
      let _integration = LmsIntegrationBuilder::new()
          .provider("google_classroom".to_owned())
          .api_key("test-api-key".to_owned())
          .build();
  }

  #[tokio::test]
  #[ignore = "requires EDUCORE_PORT_ADAPTER_E2E env var"]
  async fn integrations_integration_async_webhook_out_dispatch_mock() {
      let _integration = WebhookOutIntegrationBuilder::new()
          .target(WebhookTarget {
              url: "https://school.example.com/hooks/educore".to_owned(),
              secret: "test-secret".to_owned(),
              event_filter: Some("InvoicePaid".to_owned()),
          })
          .build();
  }
  ```

---

### END FINDINGS

Total: 42 findings.
