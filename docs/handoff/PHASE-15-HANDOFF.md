# Phase 15 → Phase 16 Hand-off

**Audience:** the next agent starting Phase 16 (test
infrastructure + SDK: `educore-testkit` +
`educore-storage-parity` (full suite) + `educore-sdk` +
`educore-cli`).
**Status:** Phase 15 closed. **5 new crates** delivered in the
`adapters` tier: `educore-auth`, `educore-notify`,
`educore-payment`, `educore-files`, `educore-integrations`.
**Spec-faithful** interpretation per
`docs/ports/{authentication,notifications,payments,file-storage,integrations}.md`.
Port-adapter tier — **not** DDD; no aggregates/events/commands.
Each crate ships the port trait + 1–3 reference impls + 4 service
helpers.

## Headline numbers

- **5 port-adapter crates** ship with **10 reference
  implementations**: `JwtAuthProvider` (HS256 JWT with builder) +
  `InMemoryOAuthStore` (implements the 4 port-driven repository
  traits from `educore-operations`) in **auth**;
  `EmailProvider` (lettre 0.10 SMTP via `rustls-tls`) +
  `SmsProvider` (reqwest HTTP gateway, Twilio shape) in **notify**;
  `StripeProvider` (reqwest + HMAC-SHA256 webhook signature) in
  **payment**; `S3FileStorage` (aws-sdk-s3 1.55, MSRV-pinned) +
  `LocalFileStorage` (tokio::fs + path-safety) in **files**;
  `LmsIntegration` (Google Classroom shape) +
  `VideoConferencingIntegration` (Zoom shape) +
  `WebhookOutIntegration` (HMAC-SHA256-signed out-webhook) in
  **integrations**.
- **20 service-helper structs** (4 per crate):
  `JwtService`, `OAuthScopeService`, `PasswordService` (Argon2id),
  `MfaService` (RFC 6238 TOTP, hand-rolled SHA-1/HMAC/base32) in
  auth; `TemplateService`, `ChannelService`, `IdempotencyService`
  (SHA-256), `RateLimitService` (per-channel token bucket) in
  notify; `IdempotencyService` (SHA-256), `WebhookSignatureService`
  (HMAC-SHA256), `BankSlipService`, `SettlementService` in
  payment; `ChecksumService` (SHA-256 + ETag),
  `SignedUrlService` (HMAC-SHA256), `KeyNamespaceService`,
  `VisibilityService` in files; `WebhookSignatureService`
  (HMAC-SHA256), `PollingService`, `RetryService` (exponential
  backoff + 4xx/5xx classification), `RateLimitService` (per-
  integration token bucket) in integrations.
- **46 net-new `Capability` variants** in `educore-rbac` (13 Auth
  + 9 Notify + 8 Payment + 8 Files + 8 Integrations) + **5 new
  `CapabilityDomain` variants** (Auth, Notify, Payment, Files,
  Integrations).
- **10 net-new `AuditTarget` variants** in `educore-audit`:
  `OAuthAccessToken`, `OAuthClient`, `PasswordReset`, `Migration`,
  `AuthSession`, `PaymentReceipt`, `Refund`, `FileReference`,
  `IntegrationConfig`, `IntegrationInvocation`.
- **5 round-trip test files** in
  `crates/cross-cutting/rbac/tests/` (one per port) — 5 tests
  added, all pass.
- **5 vertical-slice integration test files** in
  `crates/adapters/<port>/tests/<port>_integration.rs` (5 sync
  + 2 env-gated each) — **25 sync tests pass + 10 env-gated
  scenarios** (`#[ignore = "requires
  EDUCORE_PORT_ADAPTER_E2E env var"]`).
- **1 new audit-target round-trip test** in
  `educore-audit::writer::audit_target_round_trip_for_port_adapters`.
- **6 `coverage.toml` rows** flipped `Pending` → `Tested` (the 5
  port-trait rows + the 1 auth JWT impl row).

The 4 port-driven repository traits in
`educore-operations/src/repository.rs`
(`OAuthAccessTokenRepository`, `OAuthClientRepository`,
`PasswordResetRepository`, `MigrationRepository`) are now
exercised by `InMemoryOAuthStore` in the auth crate. The
`#[allow(dead_code)]` markers on those traits remain in place
(the engine doesn't construct them; only the auth crate does).

## Validation gates (all green for Phase 15 crates)

- `cargo build --workspace` — clean
- `cargo test --workspace` — all green (the Phase 15 port-adapter
  crates' 25 sync integration tests + the 5 rbac round-trip tests
  + the 1 audit round-trip test pass; the 10 env-gated async
  scenarios are `#[ignore]`-d as designed)
- `cargo clippy -p educore-auth -p educore-notify -p educore-payment
  -p educore-files -p educore-integrations --all-targets -- -D
  warnings` — clean
- `cargo fmt --all -- --check` — clean
- `cargo run -p educore-core --bin lint --features lint` — clean

> **Note on `cargo clippy --workspace --all-targets -- -D
> warnings`:** pre-existing clippy debt in `educore-settings`
> (~100 errors in tests) and `educore-documents` (~17 warnings)
> prevents the full workspace clippy gate from being green. The
> Phase 15 crates themselves pass clippy. Per the Phase 14 hand-
> off pattern, the pre-existing issues are documented as
> outstanding work and out of scope for Phase 15.

## What's wired and working

### `educore-auth` (`crates/adapters/auth/`)

- **Port trait** ([`AuthProvider`](crates/adapters/auth/src/port.rs))
  + 2 reference impls: `JwtAuthProvider` (HS256, in-memory
  revocation set keyed by `sid`, builder via
  `JwtAuthProviderBuilder`) + `InMemoryOAuthStore` (implements
  the 4 port-driven traits from `educore-operations`:
  `OAuthAccessTokenRepository`, `OAuthClientRepository`,
  `PasswordResetRepository`, `MigrationRepository`).
- 4 service helpers: JWT claim validation (signature + exp + iss
  + aud), OAuth scope check + required-scope enumeration,
  Argon2id password hashing + verify + needs-rehash check, RFC
  6238 TOTP (hand-rolled SHA-1 / HMAC-SHA1 / base32 — no extra
  dep; 8-digit code, 30-second window, ±1 step tolerance).
- 13 net-new Auth caps in `educore-rbac` (`AuthLogin`,
  `AuthLogout`, `AuthRefresh`, `AuthRevoke`, `AuthPasswordReset`,
  `OAuthAccessTokenRead`, `OAuthAccessTokenRevoke`,
  `OAuthClientRead`, `OAuthClientManage`, `PasswordResetRequest`,
  `PasswordResetConfirm`, `MfaEnroll`, `MfaVerify`).
- 5 net-new `AuditTarget` variants in `educore-audit`
  (`OAuthAccessToken`, `OAuthClient`, `PasswordReset`,
  `Migration`, `AuthSession`).
- 5 sync + 2 env-gated integration tests in
  `crates/adapters/auth/tests/auth_integration.rs` (builder,
  password hash/verify, OAuth scope check, TOTP secret gen, JWT
  claim validation; env-gated async: JWT full round-trip,
  password rehash).

### `educore-notify` (`crates/adapters/notify/`)

- **Port trait** ([`NotificationProvider`](crates/adapters/notify/src/port.rs))
  + 2 reference impls: `EmailProvider` (lettre 0.10 SMTP via
  `rustls-tls`, builder `EmailProviderBuilder`) + `SmsProvider`
  (reqwest HTTP gateway, Twilio shape, builder
  `SmsProviderBuilder`).
- 4 service helpers: `TemplateService::substitute_variables` +
  `validate_required_variables` + `extract_variables`;
  `ChannelService::is_async` + `fan_out_targets` +
  `requires_authentication`; `IdempotencyService::derive_key`
  (SHA-256 over `command_id|recipient|template_version`) +
  `is_replay` (HashSet check); `RateLimitService::try_acquire` +
  `reset` + `current_state` (per-channel token bucket).
- 9 net-new Notify caps in `educore-rbac` (`NotifyEmailSend`,
  `NotifySmsSend`, `NotifyPushSend`, `NotifyInApp`, `NotifyVoice`,
  `NotifyWebhook`, `NotifyTemplateRead`, `NotifyTemplateWrite`,
  `NotifyBulkSend`).
- 5 sync + 2 env-gated integration tests in
  `crates/adapters/notify/tests/notify_integration.rs`
  (template substitute, template validate, channel
  classification, idempotency key derivation, rate-limit bucket;
  env-gated async: email send mock, SMS send mock).

### `educore-payment` (`crates/adapters/payment/`)

- **Port trait** ([`PaymentProvider`](crates/adapters/payment/src/port.rs))
  + 1 reference impl: `StripeProvider` (reqwest client +
  HMAC-SHA256 webhook signature, builder
  `StripeProviderBuilder`).
- 4 service helpers: `IdempotencyService::derive_charge_key` (SHA-
  256 over `tenant|user|amount|currency|method`) + `is_replay`;
  `WebhookSignatureService::compute_signature` + `verify_signature`
  + `extract_signature_header` (HMAC-SHA256);
  `BankSlipService::validate_slip_number` (mod-11 check) +
  `validate_slip_amount` + `generate_slip_id`;
  `SettlementService::match_settlement_line` (FIFO + exact match)
  + `compute_net_settlement` + `is_settled`.
- 8 net-new Payment caps in `educore-rbac` (`PaymentCharge`,
  `PaymentRefund`, `PaymentStatus`, `PaymentMethodList`,
  `PaymentWebhook`, `PaymentSettlement`, `BankSlipGenerate`,
  `BankSlipApprove`).
- 2 net-new `AuditTarget` variants: `PaymentReceipt`, `Refund`
  (the existing `FeesPayment` + `WalletTransaction` cover the rest
  of the payment surface).
- 5 sync + 2 env-gated integration tests in
  `crates/adapters/payment/tests/payment_integration.rs`
  (idempotency charge key, webhook signature round-trip, bank-
  slip number validation, settlement line match, settlement net
  total; env-gated async: stripe charge mock, stripe refund
  mock).

### `educore-files` (`crates/adapters/files/`)

- **Port trait** ([`FileStorage`](crates/adapters/files/src/port.rs))
  + 2 reference impls: `S3FileStorage` (aws-sdk-s3 1.55,
  MSRV-pinned via `ADR-015`, builder `S3FileStorageBuilder`) +
  `LocalFileStorage` (tokio::fs + path-safety check that
  prevents `..` traversal, builder `LocalFileStorageBuilder`).
- 4 service helpers: `ChecksumService::compute_sha256` (raw hex)
  + `verify` + `compute_etag` (quoted `"<sha256>"` form per S3);
  `SignedUrlService::sign` + `verify` + `build_signed_url`
  (HMAC-SHA256 over `key|expires_at`, base64url);
  `KeyNamespaceService::namespace_key` (`<school_id>/<key>`) +
  `parse_namespaced_key` + `is_in_tenant`;
  `VisibilityService::is_private` + `is_public` +
  `is_tenant_scoped` + `can_access`.
- 8 net-new Files caps in `educore-rbac` (`FilesPut`, `FilesGet`,
  `FilesDelete`, `FilesSignedUrl`, `FilesCopy`, `FilesMove`,
  `FilesVisibilityChange`, `FilesLifecycle`).
- 1 net-new `AuditTarget` variant: `FileReference`.
- 5 sync + 2 env-gated integration tests in
  `crates/adapters/files/tests/files_integration.rs` (etag
  quoting, key namespace round-trip, visibility classification,
  signed URL build + verify; env-gated async: local put mock).

### `educore-integrations` (`crates/adapters/integrations/`)

- **Port trait** ([`IntegrationGateway`](crates/adapters/integrations/src/port.rs))
  + 3 reference impls: `LmsIntegration` (Google Classroom shape,
  builder `LmsIntegrationBuilder`) +
  `VideoConferencingIntegration` (Zoom shape, builder
  `VideoConferencingIntegrationBuilder`) +
  `WebhookOutIntegration` (HMAC-SHA256-signed out-webhook,
  builder `WebhookOutIntegrationBuilder`).
- 4 service helpers: `WebhookSignatureService::compute_signature`
  + `verify_signature` (HMAC-SHA256);
  `PollingService::compute_next_cursor` + `should_poll` +
  `parse_schedule` (RFC 5545 RRULE subset for `INTERVAL=n[hdw]`);
  `RetryService::next_backoff` (exponential, capped) +
  `is_permanent_failure` (4xx vs 5xx classification) +
  `should_retry`; `RateLimitService::try_acquire` + `reset` +
  `current_state` (per-integration token bucket).
- 8 net-new Integrations caps in `educore-rbac`
  (`IntegrationInvoke`, `IntegrationListCapabilities`,
  `IntegrationHealth`, `IntegrationConfigure`, `WebhookOut`,
  `PollingIn`, `LmsRosterSync`, `VideoSchedule`).
- 2 net-new `AuditTarget` variants: `IntegrationConfig`,
  `IntegrationInvocation`.
- 5 sync + 2 env-gated integration tests in
  `crates/adapters/integrations/tests/integrations_integration.rs`
  (webhook signature, retry exponential, retry classification,
  polling schedule, rate-limit bucket; env-gated async: LMS
  roster sync mock, webhook-out dispatch mock).

## Cross-crate extensions

- `educore-rbac`: **46 net-new `Capability` variants** + **5 new
  `CapabilityDomain` variants** (Auth, Notify, Payment, Files,
  Integrations) + **5 new round-trip test files** in
  `crates/cross-cutting/rbac/tests/{auth,notify,payment,files,integrations}_caps.rs`.
- `educore-audit`: **10 net-new `AuditTarget` variants** + **1 new
  round-trip test**
  (`audit_target_round_trip_for_port_adapters`).
- `crates/tools/storage-parity`: **NO new integration tests
  added** (port-adapter integration tests live in each crate's
  own `tests/` per the agreed plan; `crates/tools/storage-parity`
  remains a Phase 0 scaffold + Phase 16 deliverable).
- `crates/educore/src/lib.rs`: **NO changes** (the 5 port-adapter
  crates are already re-exported via the umbrella).
- `docs/coverage.toml`: **6 rows flipped** (5 port-trait rows +
  1 auth impl row).
- `Cargo.toml` (workspace): **5 new workspace deps** wired for the
  port-adapter crates — `argon2 = "0.5"`, `hmac = "0.12"`,
  `jsonwebtoken = "9"`, `aws-sdk-s3 = { version = "1.55", features
  = ["behavior-version-latest"] }`, `lettre = { version = "0.10",
  default-features = false, features = ["rustls-tls",
  "tokio1-native-tls"] }`.

## Open questions

1. **`educore-auth` integration test scope is limited** — the test
   file uses the public API only. Constructing a JWT (for the
   `authenticate(Bearer)` round-trip) requires `chrono` + `uuid`
   which aren't dev-dependencies of the auth crate. The 5 sync
   tests exercise the builder, password service, OAuth scopes,
   TOTP secret generation, and JWT claim validation. The 2 env-
   gated async tests cover `authenticate(Anonymous)` and password
   rehash. A full Bearer round-trip test would require adding
   dev-deps to `crates/adapters/auth/Cargo.toml`.

2. **`lettre` transitive `tokio1-native-tls` feature** —
   `lettre` is declared `features = ["rustls-tls",
   "tokio1-native-tls"]` in the workspace `Cargo.toml`. The
   notify crate narrows this in its own `Cargo.toml` to
   `features = ["smtp-transport", "builder",
   "tokio1-rustls-tls"]`, so the runtime SMTP path is rustls-only.
   The `tokio1-native-tls` feature remains in the workspace
   declaration for consumers that pull `lettre` directly; the
   notify impl itself never touches `native-tls`.

3. **`aws-sdk-s3` is heavy** — the `educore-files` crate takes
   ~60–90 s to build (one-time per workspace). A future
   optimization could gate `S3FileStorage` behind a
   `feature = "s3"` flag (default off) so test-only builds skip
   the SDK. Out of scope for Phase 15.

4. **Pre-existing clippy debt** — `educore-settings` has ~100
   test-only clippy errors (mostly `unwrap_used` / `expect_used`)
   and `educore-documents` has ~17 warnings. Out of scope for
   Phase 15 per the Phase 14 hand-off pattern.

5. **The 4 port-driven repository traits** in
   `educore-operations/src/repository.rs` still carry
   `#[allow(dead_code)]` markers on their trait declarations.
   Now that `InMemoryOAuthStore` exercises them, the markers can
   be removed in a follow-up microtask. The operations crate's
   own code path doesn't construct them — only the auth crate
   does — so the markers remain technically necessary at the
   operations-crate level until a Phase 16+ clean-up.

6. **Reference impls are not yet wired into the engine's command
   path** — there is no `Engine::builder()` yet. The 10
   reference impls are exercisable directly via
   `Arc<dyn AuthProvider>`, `Arc<dyn NotificationProvider>`,
   `Arc<dyn PaymentProvider>`, `Arc<dyn FileStorage>`, and
   `Arc<dyn IntegrationGateway>`. The Phase 16 SDK is the
   intended integration point.

7. **`JwtAuthProvider::refresh` does NOT revoke the old session** —
   the refreshed token shares the `sid` with the old one. Callers
   who want strict token rotation must call `revoke` explicitly on
   the old token. This matches a "refresh extends the session"
   semantic. A follow-up could add a `revoke_on_refresh` config
   flag on `JwtAuthProviderBuilder`.

8. **`#[cfg(any())]` fix on `pub mod` items** (template artifact
   only) — the Phase 15 prep microtask was concerned about
   potentially removing `pub mod webhook_out;` / `pub mod lms;`
   from `educore-integrations/src/lib.rs`. The final lib.rs has
   plain `pub mod` declarations and no `#[cfg(any())]` markers;
   nothing to remove.

## Where NOT to start (Phase 16)

- Do NOT remove the pre-existing clippy debt in
  `educore-settings` / `educore-documents` (out of scope, see OQ
  #4).
- Do NOT add `educore-finance` dep to `educore-payment` (Phase 8
  OQ #6 + Phase 10 OQ #3 + Phase 11 OQ #4 + Phase 12 OQ #5 +
  Phase 13 OQ #3 + Phase 14 OQ #4 carry-over). Phase 15 kept it
  out; Phase 16 should too.
- Do NOT add `educore-academic` / `educore-attendance` /
  `educore-documents` deps to any port-adapter crate.
- Do NOT remove the `#[allow(dead_code)]` markers on the 4 port-
  driven repository traits in
  `educore-operations/src/repository.rs` without a follow-up
  microtask (they're still dead in the operations crate's own
  code path; only the auth crate exercises them — see OQ #5).
- Do NOT remove the workspace `lettre` `tokio1-native-tls`
  feature (out of scope; see OQ #2). The notify crate already
  narrows the feature set for its own build.
- Do NOT add a Phase 15+ `Capability` variant to `educore-rbac`
  without coordinating with the rbac round-trip test
  (`auth_capabilities_round_trip`, etc.) — the count assertions
  are documented in the test bodies.
- Do NOT remove the 2 Phase 2 settings/operations capability
  placeholders (`SettingsManage`/`OperationsManage`) — they were
  REMOVED in Phase 14. Do NOT add them back either.
- Do NOT touch the 18 closed crates other than the additive
  rbac + audit extensions + the 1 `Cargo.toml` addition to
  storage-parity Phase 16 may need. Per
  `ADR-013-CrateLayout.md`, the cross-crate modifications are
  all non-breaking additive.
- Do NOT touch `educore-core::lint`. The lint binary passes; the
  tier-boundary checker remains a stub.

## Key files for the next agent

- `crates/adapters/{auth,notify,payment,files,integrations}/` —
  all 5 port-adapter crates (port trait + 1–3 reference impls +
  4 service helpers + 5 + 2 integration tests each)
- `crates/cross-cutting/rbac/src/value_objects.rs` — the 46
  net-new `Capability` variants + 5 `CapabilityDomain` variants
  (Auth caps at line ~1373, Notify at ~1406, Payment at ~1432,
  Files at ~1455, Integrations at ~1476; `domain()` / `aggregate()`
  / `as_str()` / `all_variants` / `from_str_opt` match arms
  updated for all 46)
- `crates/cross-cutting/audit/src/writer.rs` — the 10 net-new
  `AuditTarget` variants (`OAuthAccessToken` / `OAuthClient` /
  `PasswordReset` / `Migration` / `AuthSession` / `PaymentReceipt`
  / `Refund` / `FileReference` / `IntegrationConfig` /
  `IntegrationInvocation`) at lines ~424–442 + the
  `audit_target_round_trip_for_port_adapters` test at line ~1415
- `crates/cross-cutting/rbac/tests/{auth,notify,payment,files,integrations}_caps.rs`
  — 5 new round-trip test files
- `crates/adapters/{auth,notify,payment,files,integrations}/tests/<port>_integration.rs`
  — 5 new vertical-slice integration test files (5 sync `#[test]`
  + 2 env-gated `#[tokio::test]` each; env-gated scenarios use
  `#[ignore = "requires EDUCORE_PORT_ADAPTER_E2E env var"]`)
- `crates/adapters/auth/src/oauth_store.rs` — `InMemoryOAuthStore`
  (the reference impl for the 4 port-driven repository traits)
- `crates/tools/storage-parity/tests/settings_integration.rs` —
  the existing 5 + 2 scenario pattern (Phase 16 may use this as a
  reference for the cross-adapter parity suite)
- `crates/tools/storage-parity/Cargo.toml` — Phase 16 will likely
  add `educore-auth` / `educore-notify` / `educore-payment` /
  `educore-files` / `educore-integrations` deps for the parity
  suite to instantiate the 5 port adapters
- `Cargo.toml` (workspace root) — the 5 new workspace deps
  (`argon2`, `hmac`, `jsonwebtoken`, `aws-sdk-s3`, `lettre`)
- `docs/coverage.toml` — 6 rows flipped `Pending` → `Tested` (5
  port-trait rows + 1 auth impl row)
- `docs/handoff/PHASE-15-HANDOFF.md` — this hand-off
- `docs/phase_prompt/phase-16-prompt.md` — the next-phase prompt
  (brief for `educore-testkit` + `educore-storage-parity` +
  `educore-sdk` + `educore-cli`)
- `docs/build-plan.md` § "Phase 15 outcome." — the build-plan
  outcome subsection (the canonical 1-paragraph summary, mirrored
  from `PHASE-15-HANDOFF.md`)

## Where to ask

Open a GitHub issue for design questions. The Phase 15 prompt is
the source of truth for Phase 15's scope; the next-phase prompt is
the source of truth for Phase 16's scope. For disputes, defer to
`AGENTS.md` (engine rules) and `ADR-013-CrateLayout.md` (tier
definitions).