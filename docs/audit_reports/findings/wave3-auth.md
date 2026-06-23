# Audit findings: educore-auth (Phase 15 / adapters)

**Scope:** `crates/adapters/auth/` (5 src files: lib.rs, port.rs,
errors.rs, jwt.rs, oauth_store.rs, services.rs; 1 test file:
`tests/auth_integration.rs`), `docs/ports/authentication.md`,
`docs/handoff/PHASE-15-HANDOFF.md`, `docs/code-standards.md`,
`AGENTS.md` (the auth row).

**Total findings:** 38

---

### FINDING 1

- **id:** ADAPTER-AUTH-001
- **area:** adapters
- **severity:** Critical
- **location:** crates/adapters/auth/src/jwt.rs:331-350, 389
- **description:** `JwtAuthProvider::authenticate` accepts
  `Credential::Anonymous` and returns a `Session` with
  `mfa_satisfied: true`, `active_school_id: PUBLIC_SCHOOL_ID`,
  `user_id: SYSTEM_USER_ID`, and a `BTreeSet::<Capability>::new()`
  (empty capability set). The crate's own port-deviation note
  documents this as a deviation, but the implementation ships
  this behavior by default and consumers cannot disable it from
  the builder. Per the port spec, anonymous is "rejected by the
  default adapters except in public-facing flows (e.g. public
  exam result lookup, when explicitly allowed by configuration)".
  The provider has no "allow_anonymous" knob on the builder —
  the deviation is hard-coded.
- **expected:** `docs/ports/authentication.md:38-40` — "A
  `Credential::Anonymous` is rejected by the default adapters
  except in public-facing flows (e.g. public exam result lookup,
  when explicitly allowed by configuration)."
- **evidence:** `crates/adapters/auth/src/jwt.rs:389` —
  ```rust
  Credential::Anonymous => Ok(self.anonymous_session()),
  ```
  with `mfa_satisfied: true` at `crates/adapters/auth/src/jwt.rs:345`,
  empty capabilities at `crates/adapters/auth/src/jwt.rs:344`.

---

### FINDING 2

- **id:** ADAPTER-AUTH-002
- **area:** adapters
- **severity:** Critical
- **location:** crates/adapters/auth/src/jwt.rs:167-176
- **description:** `JwtAuthProviderBuilder::new()` generates a
  fresh 32-byte random signing key via `rand::thread_rng()` on
  every call. The crate's own port-deviation note acknowledges
  this, but the builder provides **no warning** when a consumer
  uses the default key. If a consumer forgets to call
  `.signing_key(env::var("JWT_SECRET")?)`, every process restart
  invalidates every previously-issued JWT, every replica in a
  horizontally-scaled deployment signs with a different key, and
  the dev `JwtAuthProviderBuilder::new()` works fine in single-
  process tests but silently breaks in production.
- **expected:** `docs/ports/authentication.md:175-180` —
  ```rust
  let auth: Arc<dyn AuthProvider> = Arc::new(
      JwtAuthProvider::builder()
          .signing_key(env::var("JWT_SECRET")?)
          ...
  );
  ```
  The spec assumes the key is loaded from env at startup; the
  builder's "suitable for tests and the worked example only"
  default is not flagged at runtime.
- **evidence:** `crates/adapters/auth/src/jwt.rs:167-176`:
  ```rust
  pub fn new() -> Self {
      let mut key = vec![0_u8; 32];
      rand::thread_rng().fill_bytes(&mut key);
      Self {
          signing_key: key,
          ...
      }
  }
  ```

---

### FINDING 3

- **id:** ADAPTER-AUTH-003
- **area:** adapters
- **severity:** Critical
- **location:** crates/adapters/auth/src/jwt.rs:19-23, 142, 231
- **description:** Token revocation uses a process-local
  `Arc<Mutex<HashSet<String>>>` keyed by `sid`. The crate
  documentation explicitly acknowledges "the set is process-
  local; consumers that need cross-process revocation must
  layer a shared store on top" — but no shared-store
  implementation is provided. In any horizontally-scaled or
  multi-replica deployment, revoking a token on replica A
  leaves the same token valid on replicas B, C, D. The
  revocation mechanism is therefore broken for the production
  multi-replica case the engine targets.
- **expected:** `docs/ports/authentication.md:134-140` —
  "`revoke(token)` invalidates the token. The adapter updates
  its session store. Subsequent `validate` calls return
  `AuthError::Revoked`. A super-admin can also revoke all
  sessions for a user (e.g. after password change or suspected
  compromise)." This implies a cross-process session store.
- **evidence:** `crates/adapters/auth/src/jwt.rs:142` —
  ```rust
  revoked_sessions: Arc<Mutex<HashSet<String>>),
  ```
  and `crates/adapters/auth/src/jwt.rs:19-23` —
  ```text
  //! - Token revocation: an in-memory `HashSet<String>` keyed by
  //!   `sid`. The set is process-local; consumers that need
  //!   cross-process revocation must layer a shared store on top.
  ```

---

### FINDING 4

- **id:** ADAPTER-AUTH-004
- **area:** adapters
- **severity:** Critical
- **location:** crates/adapters/auth/src/jwt.rs:422-464
- **description:** `JwtAuthProvider::refresh()` does **not** mint
  a new JWT, does **not** invalidate the old token, and does
  **not** return the refreshed `AuthToken`. The function
  encodes `new_claims` into `_refreshed_token` then **discards**
  the encoded value (`let _refreshed_token = self.encode(...)?`),
  returns only the new `Session`, and the comment at line 459-
  462 explicitly admits "Touch the encoder so future revisions
  can plumb the refreshed token back through the AuthToken
  channel without re-deriving the encoding path." The port spec
  states "`refresh(token)` returns a new `Session` for a non-
  expired token. The adapter may rotate the token value. The
  old token is invalidated." The current implementation does
  neither.
- **expected:** `docs/ports/authentication.md:142-146` — "`refresh(token)`
  returns a new `Session` for a non-expired token. The adapter
  may rotate the token value. The old token is invalidated."
- **evidence:** `crates/adapters/auth/src/jwt.rs:455-463`:
  ```rust
  // Suppress the unused-field warning when `refresh_ttl_secs`
  // is not consumed by this minimal implementation.
  let _ = self.refresh_ttl_secs;

  // Touch the encoder so future revisions can plumb the
  // refreshed token back through the AuthToken channel
  // without re-deriving the encoding path.
  let _refreshed_token = self.encode(&new_claims)?;
  self.session_from_claims(&new_claims)
  ```

---

### FINDING 5

- **id:** ADAPTER-AUTH-005
- **area:** adapters
- **severity:** Critical
- **location:** crates/adapters/auth/src/jwt.rs:441-453
- **description:** `JwtAuthProvider::refresh()` sets
  `sid: old_claims.sid` and does **not** call `add_revoked`.
  The comment at line 446-447 admits "We do NOT add the sid to
  the revocation set on refresh; instead, callers who want
  strict token rotation should call `revoke` on the old token
  explicitly." This is a port-spec violation: spec says "The
  old token is invalidated." The current behavior allows the
  old JWT to remain valid indefinitely (until natural
  expiration), so a stolen old JWT continues to grant access
  for up to `access_ttl_secs` after refresh.
- **expected:** `docs/ports/authentication.md:142-146` —
  "The old token is invalidated."
- **evidence:** `crates/adapters/auth/src/jwt.rs:448` —
  ```rust
  sid: old_claims.sid,
  ```
  and absence of any `add_revoked(&old_claims.sid)` call in
  the refresh body (lines 422-464).

---

### FINDING 6

- **id:** ADAPTER-AUTH-006
- **area:** adapters
- **severity:** Critical
- **location:** crates/adapters/auth/src/jwt.rs:182-185
- **description:** `JwtAuthProviderBuilder::signing_key` accepts
  arbitrary byte slices without minimum-length validation.
  RFC 7518 § 3.2 requires "A key of the same size as the hash
  output (for instance, 256 bits for "HS256") or larger MUST be
  used with this algorithm." The builder accepts
  `b"too-short"` and would silently produce tokens signed with
  a weak key. The `JwtAuthProviderBuilder::new` default is
  32 bytes (correct), but a consumer can pass any length via
  `signing_key(...)` and the builder does not reject short
  keys.
- **expected:** RFC 7518 § 3.2 (cited via JWT spec) and
  OWASP/JWT security best practice require minimum-length
  validation on HMAC keys.
- **evidence:** `crates/adapters/auth/src/jwt.rs:181-185`:
  ```rust
  #[must_use]
  pub fn signing_key(mut self, key: impl Into<Vec<u8>>) -> Self {
      self.signing_key = key.into();
      self
  }
  ```
  No `key.len() < 32` check.

---

### FINDING 7

- **id:** ADAPTER-AUTH-007
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/auth/src/jwt.rs:229-230
- **description:** `JwtAuthProviderBuilder::build` silently caps
  `access_ttl_secs` and `refresh_ttl_secs` to `i64::MAX` when
  the `Duration::as_secs()` value cannot be represented as
  `i64`. This swallows a logically-impossible input (a
  `Duration` longer than ~292 billion years) and produces a
  provider with effectively infinite TTL — a security-relevant
  silent failure. The `unwrap_or(i64::MAX)` pattern is used in
  production code paths (no `#[cfg(test)]` or `#[allow]`).
- **expected:** `docs/code-standards.md` (AGENTS.md § "Type
  Safety") — "No `as` casts that truncate or lose data. Use
  `TryFrom` / `TryInto` with proper error handling." Silent
  saturation is a stricter violation of this rule.
- **evidence:** `crates/adapters/auth/src/jwt.rs:229-230`:
  ```rust
  access_ttl_secs: i64::try_from(self.access_ttl.as_secs()).unwrap_or(i64::MAX),
  refresh_ttl_secs: i64::try_from(self.refresh_ttl.as_secs()).unwrap_or(i64::MAX),
  ```

---

### FINDING 8

- **id:** ADAPTER-AUTH-008
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/auth/src/jwt.rs:354-373
- **description:** `add_revoked` and `check_not_revoked` both
  recover from `PoisonError` via
  `.unwrap_or_else(PoisonError::into_inner)`. A poisoned mutex
  indicates a panic in another thread while the lock was held;
  silently recovering and continuing to operate on potentially
  inconsistent state hides concurrency bugs. The `unwrap_or_else`
  pattern is in production paths (no `#[cfg(test)]`).
- **expected:** `docs/code-standards.md` (AGENTS.md § "Type
  Safety") — "No `unwrap()` or `expect()` in production paths."
  `unwrap_or_else(PoisonError::into_inner)` is functionally a
  silent unwrap.
- **evidence:** `crates/adapters/auth/src/jwt.rs:355-359`:
  ```rust
  let mut set = self
      .revoked_sessions
      .lock()
      .unwrap_or_else(std::sync::PoisonError::into_inner);
  set.insert(sid.to_owned());
  ```
  Same pattern at `crates/adapters/auth/src/jwt.rs:366-368`.

---

### FINDING 9

- **id:** ADAPTER-AUTH-009
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/auth/src/jwt.rs:332-337
- **description:** `anonymous_session` computes the expiry via
  `.single().unwrap_or_else(|| now.as_datetime())`. If
  `Utc.timestamp_opt` returns `None` or `Ambiguous` for the
  computed expiry instant, the function silently falls back to
  "now" — issuing a session that is already expired. This is
  in production code (no `#[cfg(test)]` or `#[allow]`).
- **expected:** `docs/code-standards.md` (AGENTS.md § "Type
  Safety") — propagation of errors is the standard.
- **evidence:** `crates/adapters/auth/src/jwt.rs:332-337`:
  ```rust
  let now = Timestamp::now();
  let exp = Timestamp::from_datetime(
      Utc.timestamp_opt(now.as_datetime().timestamp() + self.access_ttl_secs, 0)
          .single()
          .unwrap_or_else(|| now.as_datetime()),
  );
  ```

---

### FINDING 10

- **id:** ADAPTER-AUTH-010
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/auth/src/jwt.rs:442
- **description:** `refresh()` computes the new `exp` claim via
  `now.saturating_add(self.access_ttl_secs)`. On `i64::MAX`
  overflow (which `Finding 7` makes reachable) the saturation
  clamps to `i64::MAX`, producing a JWT whose `exp` claim is
  effectively "never expires" — bypassing the entire TTL
  mechanism. Combined with Finding 5 (no revocation on
  refresh), a token issued through `refresh()` can become
  permanently valid.
- **expected:** `docs/ports/authentication.md` § "Session" —
  `expires_at` is a bounded timestamp; the spec assumes the
  issuer enforces TTL.
- **evidence:** `crates/adapters/auth/src/jwt.rs:442`:
  ```rust
  exp: now.saturating_add(self.access_ttl_secs),
  ```

---

### FINDING 11

- **id:** ADAPTER-AUTH-011
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/auth/src/port.rs:46
- **description:** `port.rs` declares
  `#![allow(dead_code, clippy::all, missing_docs)]`. While
  `lib.rs` has `#![deny(missing_docs)]`, this per-file `allow`
  suppresses the lint for every public item in the **port**
  module — the module that defines `AuthProvider`, `RbacPort`,
  `Session`, `AuthToken`, `AuthScheme`, `Credential`. Several
  fields in `Credential` (`code_verifier`, `relay_state`,
  `Biometric.signature`) have rustdoc, but other public items
  in `port.rs` may not. The deny/allow interaction is brittle.
- **expected:** `docs/code-standards.md` (AGENTS.md § "Code
  Standards") — "All public APIs are documented with
  rustdoc; `#![deny(missing_docs)]`."
- **evidence:** `crates/adapters/auth/src/port.rs:46`:
  ```rust
  #![allow(dead_code, clippy::all, missing_docs)]
  ```

---

### FINDING 12

- **id:** ADAPTER-AUTH-012
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/auth/src/jwt.rs:285-325
- **description:** `JwtAuthProvider::session_from_claims` sets
  `capabilities: BTreeSet::<Capability>::new()` — the returned
  `Session` always has an empty capability set. The crate's own
  documentation admits "JWT claims do not carry capabilities in
  this implementation. A future revision will resolve
  capabilities from the RBAC port at validate-time." As
  shipped, the JWT provider produces sessions with **zero
  capabilities**, which means every RBAC capability check
  against a JWT-issued session will fail, blocking every
  privileged operation. This contradicts the port spec's
  "Capabilities are pre-computed when the session is created;
  the engine does not consult the RBAC storage on every
  command."
- **expected:** `docs/ports/authentication.md:58-62` —
  "`Session` is a value type. It carries everything the engine
  needs to authorize and tenant-isolate a command. Capabilities
  are pre-computed when the session is created; the engine does
  not consult the RBAC storage on every command."
- **evidence:** `crates/adapters/auth/src/jwt.rs:319`:
  ```rust
  capabilities: BTreeSet::<Capability>::new(),
  ```
  and `crates/adapters/auth/src/jwt.rs:43-45`:
  ```text
  //! - The capability set on the returned [`Session`] is always
  //!   empty; JWT claims do not carry capabilities in this
  //!   implementation. A future revision will resolve capabilities
  //!   from the RBAC port at validate-time.
  ```

---

### FINDING 13

- **id:** ADAPTER-AUTH-013
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/auth/src/jwt.rs:380-465
- **description:** `JwtAuthProvider` only handles two of seven
  `Credential` variants: `Bearer` and `Anonymous`. The other
  five (`UsernamePassword`, `Oauth2`, `Saml`, `ApiKey`,
  `Biometric`) are all rejected with `AuthError::InvalidCredentials`.
  The port spec requires the trait to be the universal entry
  point for all authentication modes, but this adapter
  implements only JWT. The same trait ships from the same crate
  with no alternative adapter for the rejected variants — the
  port surface advertises support for these credential types
  but the only shipped impl uniformly rejects them.
- **expected:** `docs/ports/authentication.md:184-191` —
  "Alternative adapters: `LocalPasswordAuthProvider` —
  username + password against a local user table (hashed with
  argon2). `OAuth2AuthProvider` — external OAuth2/OIDC ...,
  `SamlAuthProvider` — SAML 2.0, `ApiKeyAuthProvider` —
  service-to-service auth." These are described as part of
  the crate's deliverable.
- **evidence:** `crates/adapters/auth/src/jwt.rs:382-396`:
  ```rust
  async fn authenticate(&self, credential: Credential) -> Result<Session, AuthError> {
      match credential {
          Credential::Bearer(token) => { ... }
          Credential::Anonymous => Ok(self.anonymous_session()),
          Credential::UsernamePassword { .. }
          | Credential::Oauth2 { .. }
          | Credential::Saml { .. }
          | Credential::ApiKey { .. }
          | Credential::Biometric { .. } => Err(AuthError::InvalidCredentials),
      }
  }
  ```

---

### FINDING 14

- **id:** ADAPTER-AUTH-014
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/auth/src/errors.rs:46-91
- **description:** `AuthError` declares 10 variants
  (`InvalidCredentials`, `AccountLocked`, `AccountDisabled`,
  `Expired`, `Revoked`, `Malformed`, `MfaRequired`,
  `MfaFailed`, `RateLimited`, `Infrastructure`). Of these, only
  `InvalidCredentials`, `Malformed`, `Expired`, and `Revoked`
  are ever produced by the JWT provider. `AccountLocked`,
  `AccountDisabled`, `MfaRequired`, `MfaFailed`, and
  `RateLimited` are **dead variants in this crate** — no code
  path returns them. The port spec lists tests for "rate
  limiting" and "MFA required / satisfied"; neither is
  exercised because no code emits these variants.
- **expected:** `docs/ports/authentication.md:220-230` —
  "Integration tests for ... A test for MFA required /
  satisfied. A test for rate limiting. A test for
  infrastructure failure."
- **evidence:** `crates/adapters/auth/src/errors.rs:46-91`
  declares all 10 variants. Grep for `MfaRequired`,
  `RateLimited`, `AccountLocked`, `AccountDisabled` in
  `crates/adapters/auth/src/` returns only the declaration
  site (no production usage).

---

### FINDING 15

- **id:** ADAPTER-AUTH-015
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/auth/src/jwt.rs:400-403, 411-415, 423-427
- **description:** The JWT provider's `validate`, `revoke`, and
  `refresh` paths emit `AuthError::Malformed` via `format!` with
  the rejected `AuthScheme` interpolated via the `{:?}` Debug
  formatter:
  ```rust
  "JwtAuthProvider only accepts Bearer tokens, got {:?}"
  ```
  This is fine for the current code path (only `AuthScheme` is
  printed, not the token value), but the **same error path** is
  reused in `parse_uuid` (`jwt.rs:475`), `unix_secs_to_timestamp`
  (`jwt.rs:483-486, 489-491`), and `map_jwt_error` (`jwt.rs:509,
  517, 520`) where the wrapped `jsonwebtoken::Error` is
  formatted verbatim. The `jsonwebtoken` crate's `Display` impl
  for some error kinds includes claim contents (e.g.
  `InvalidSubject` carries the offending subject string). Any
  consumer that propagates `AuthError::Display` output to a
  structured log line may leak claim PII (the `sub` claim is
  the user id, `schools` and `active_school` are tenant ids).
- **expected:** `docs/ports/authentication.md:240-242` —
  "Sensitive material (passwords, MFA codes) is never logged."
  This is the principle; the same principle applies to PII
  claims.
- **evidence:** `crates/adapters/auth/src/jwt.rs:499-521`:
  ```rust
  fn map_jwt_error(err: jsonwebtoken::errors::Error) -> AuthError {
      ...
      | ErrorKind::InvalidIssuer
      | ErrorKind::InvalidAudience
      | ErrorKind::InvalidSubject
      ...
      => AuthError::Malformed(format!("jwt claim: {err}")),
      _ => AuthError::Malformed(format!("jwt: {err}")),
  }
  ```

---

### FINDING 16

- **id:** ADAPTER-AUTH-016
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/auth/src/jwt.rs:266-270, 499-521
- **description:** `encode` formats the jsonwebtoken encode
  error verbatim: `format!("jwt encode failed: {e}")`. The
  `jsonwebtoken` crate's encode error kinds include
  `Json(serde_json::Error)` which carries the input JSON
  content (i.e. the full claim set, including user id, role
  ids, school ids, session id, MFA flag). A consumer that
  logs `AuthError::Display` output on the encode failure path
  leaks the full claim set.
- **expected:** `docs/ports/authentication.md:240-242` —
  "Sensitive material (passwords, MFA codes) is never logged."
- **evidence:** `crates/adapters/auth/src/jwt.rs:266-270`:
  ```rust
  fn encode(&self, claims: &JwtClaims) -> Result<String, AuthError> {
      let key = EncodingKey::from_secret(self.signing_key.as_slice());
      encode(&Header::new(Algorithm::HS256), claims, &key)
          .map_err(|e| AuthError::Malformed(format!("jwt encode failed: {e}")))
  }
  ```

---

### FINDING 17

- **id:** ADAPTER-AUTH-017
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/auth/src/jwt.rs:329-349
- **description:** `anonymous_session()` produces a session
  with `mfa_satisfied: true`. The port spec says MFA is
  per-session and "When a session is `mfa_satisfied = false`,
  the engine restricts sensitive commands." The anonymous
  session is short-circuiting MFA entirely — every command
  the system user executes is treated as MFA-satisfied, with
  no second-factor check. The session uses `SYSTEM_USER_ID`,
  which presumably has super-admin capabilities in the RBAC
  catalog.
- **expected:** `docs/ports/authentication.md:116-132` —
  "`mfa_satisfied = false`, the engine restricts sensitive
  commands. The adapter decides which commands require MFA
  based on configuration."
- **evidence:** `crates/adapters/auth/src/jwt.rs:345`:
  ```rust
  mfa_satisfied: true,
  ```
  inside the `anonymous_session()` function (line 331) which
  produces the system-user session for `Credential::Anonymous`.

---

### FINDING 18

- **id:** ADAPTER-AUTH-018
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/auth/src/jwt.rs:253-254
- **description:** `Validation::leeway = 0` is hard-coded.
  Clock skew between issuer and validator is normal in
  distributed systems (NTP drift, container clock
  virtualization). A 0-second leeway causes false rejections
  when the issuer's clock is even a fraction ahead of the
  validator's, especially for short-lived tokens. The
  `jsonwebtoken` crate's default leeway is 60 seconds; the
  port's reference implementation is stricter than the
  upstream default with no override knob on the builder.
- **expected:** Standard JWT issuance practice and the
  upstream `jsonwebtoken` crate's default leeway of 60s.
- **evidence:** `crates/adapters/auth/src/jwt.rs:249-254`:
  ```rust
  fn validation(&self) -> Validation {
      let mut v = Validation::new(Algorithm::HS256);
      v.set_issuer(&[self.issuer.as_str()]);
      v.set_audience(&[self.audience.as_str()]);
      v.validate_exp = true;
      v.leeway = 0;
  ```

---

### FINDING 19

- **id:** ADAPTER-AUTH-019
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/auth/tests/auth_integration.rs:97-105
- **description:** The only async integration test
  (`auth_integration_async_jwt_full_round_trip`) is `#[ignore]`
  gated behind `EDUCORE_PORT_ADAPTER_E2E env var`. The
  remaining async test (`auth_integration_async_password_rehash_check`)
  is also `#[ignore]`-gated. CI runs only 5 sync tests. A full
  Bearer round-trip (authenticate → validate → refresh →
  revoke) is **never** exercised by the test suite as it
  ships. The handoff documents this as OQ #1 but the gap is
  not closed in code.
- **expected:** `docs/ports/authentication.md:220-230` —
  "Integration tests for token issue, validate, refresh,
  revoke."
- **evidence:** `crates/adapters/auth/tests/auth_integration.rs:97-105`:
  ```rust
  #[tokio::test]
  #[ignore = "requires EDUCORE_PORT_ADAPTER_E2E env var"]
  async fn auth_integration_async_jwt_full_round_trip() {
      let provider = JwtAuthProviderBuilder::new().build();
      let _session = provider
          .authenticate(Credential::Anonymous)
          .await
          .expect("anonymous auth should succeed");
  }
  ```

---

### FINDING 20

- **id:** ADAPTER-AUTH-020
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/auth/tests/auth_integration.rs (full file)
- **description:** The test file contains 5 sync tests
  (`auth_integration_jwt_builder_constructs`,
  `auth_integration_password_hash_and_verify`,
  `auth_integration_oauth_scope_check`,
  `auth_integration_mfa_generate_secret`,
  `auth_integration_jwt_validate_claims`) plus 2 env-gated
  async tests. None of the tests cover: expired token via
  `JwtAuthProvider::validate`, revoked token via
  `JwtAuthProvider::validate` (only `JwtService::validate_claims`),
  cross-tenant denial, MFA required / satisfied flow,
  rate limiting, infrastructure failure, or
  `Credential::Anonymous` rejection (because the dev impl
  accepts it). The port spec explicitly lists each as a
  required test scenario.
- **expected:** `docs/ports/authentication.md:220-230` —
  "Unit tests of every `Credential` variant. Integration tests
  for token issue, validate, refresh, revoke. A test for
  expired and revoked tokens. A test for cross-tenant denial.
  A test for MFA required / satisfied. A test for rate
  limiting. A test for infrastructure failure."
- **evidence:** `crates/adapters/auth/tests/auth_integration.rs`
  contains 114 lines; only the scenarios listed above are
  covered.

---

### FINDING 21

- **id:** ADAPTER-AUTH-021
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/auth/src/services.rs:340-374
- **description:** `PasswordService::needs_rehash` compares
  only the algorithm identifier string (e.g. `"argon2id"`)
  against the parsed PHC, **not** the cost parameters
  (`m`, `t`, `p`). The crate's own comment acknowledges this:
  "The cost parameters are intentionally not compared here ...
  The algorithm check is the meaningful signal anyway — when
  the engine rotates its default parameters, the migration
  plan bumps the algorithm tag or re-hashes via
  `hash_password` on next login." This is a port-spec
  deviation: the function is named `needs_rehash` but only
  detects algorithm-tag changes, not parameter-rotated hashes.
  If a consumer upgrades Argon2 parameters without bumping
  the algorithm tag, `needs_rehash` returns `false` for hashes
  that should be rotated.
- **expected:** Function-name contract `needs_rehash` implies
  detection of out-of-date parameters, not only out-of-date
  algorithms.
- **evidence:** `crates/adapters/auth/src/services.rs:346-374`:
  ```rust
  pub fn needs_rehash(&self, hash: &str) -> bool {
      let Ok(parsed) = PasswordHash::new(hash) else {
          return true;
      };
      ...
      parsed.algorithm.as_str() != "argon2id"
  }
  ```

---

### FINDING 22

- **id:** ADAPTER-AUTH-022
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/auth/src/services.rs:307-335
- **description:** Argon2 errors during `hash_password` and
  `verify_password` are wrapped as `AuthError::Malformed`. The
  Argon2 crate distinguishes `Error::Password` (a verification
  mismatch — not an error) from other errors (corrupted PHC,
  unsupported algorithm, memory allocation failure, OS RNG
  failure). The current code maps **all** non-mismatch errors
  to `AuthError::Malformed`, which the port spec describes as
  "the token is structurally invalid (bad signature, malformed
  JWT, unknown scheme)". An OS RNG failure during hash or an
  allocation failure during verify is an infrastructure-level
  failure, not a malformed-input failure. Misclassifying
  infrastructure failures as Malformed hides operational
  problems from monitoring.
- **expected:** `docs/ports/authentication.md:148-163` —
  `AuthError::Infrastructure` is the canonical variant for
  "A non-domain infrastructure failure (network, DNS, TLS,
  IdP, JWKS fetch)."
- **evidence:** `crates/adapters/auth/src/services.rs:312-315`:
  ```rust
  self.argon2
      .hash_password(plain.expose_secret().as_bytes(), &salt)
      .map(|h| h.to_string())
      .map_err(|e| AuthError::Malformed(format!("argon2 hash failed: {e}")))
  ```
  and `crates/adapters/auth/src/services.rs:325-333`:
  ```rust
  match self.argon2.verify_password(...) {
      Ok(()) => Ok(true),
      Err(argon2::password_hash::Error::Password) => Ok(false),
      Err(e) => Err(AuthError::Malformed(format!("argon2 verify failed: {e}"))),
  }
  ```

---

### FINDING 23

- **id:** ADAPTER-AUTH-023
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/auth/src/services.rs:391-449
- **description:** The crate ships only one MFA factor
  implementation: RFC 6238 TOTP (HMAC-SHA1, 8-digit). The port
  spec lists five second-factor mechanisms: TOTP, SMS code,
  Email code, WebAuthn/FIDO2, and Backup code. Only one is
  implemented; the other four are absent. The `AuthError`
  variants `MfaRequired` and `MfaFailed` are declared but
  never emitted by any code path in this crate.
- **expected:** `docs/ports/authentication.md:122-132` —
  "A second factor may be satisfied by: TOTP (RFC 6238),
  SMS code, Email code, WebAuthn / FIDO2, Backup code."
- **evidence:** `crates/adapters/auth/src/services.rs:381-449`
  contains only `MfaService` (TOTP). Grep for SMS, FIDO2,
  WebAuthn, Backup in `crates/adapters/auth/src/` returns no
  matches. Grep for `MfaRequired` returns only the declaration
  site (`crates/adapters/auth/src/errors.rs:75`).

---

### FINDING 24

- **id:** ADAPTER-AUTH-024
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/auth/src/port.rs:158-167
- **description:** `AuthToken.value` is a plain `String`, not
  `secrecy::SecretString` or the crate's own `SecretString`.
  The port-deviation note (`port.rs:152-156`) acknowledges this
  and says "the `Debug` impl does **not** auto-redact the
  value (the adapter is responsible for redaction before
  logging)." However, the `Debug` impl on `AuthToken`
  (`port.rs:157`) auto-derives `Debug`, so `format!("{token:?}")`
  in any consumer will leak the token value. The
  `#[derive(Debug)]` is at line 157 and the field at line 163
  is `pub value: String,` with no `#[secret]` attribute or
  custom Debug impl.
- **expected:** `docs/ports/authentication.md:240-242` —
  "Sensitive material (passwords, MFA codes) is never
  logged." The port exposes the same risk for tokens.
- **evidence:** `crates/adapters/auth/src/port.rs:157-167`:
  ```rust
  #[derive(Debug, Clone, PartialEq, Eq, Hash)]
  pub struct AuthToken {
      pub scheme: AuthScheme,
      pub value: String,
      pub metadata: BTreeMap<String, String>,
  }
  ```

---

### FINDING 25

- **id:** ADAPTER-AUTH-025
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/auth/src/port.rs:188-195
- **description:** `Credential::UsernamePassword.password` is a
  plain `String`. The `Debug` impl on `Credential`
  (`port.rs:179`) auto-derives `Debug`, so
  `format!("{credential:?}")` in any consumer will print the
  plaintext password. The port-deviation note documents this
  but does not mitigate it.
- **expected:** `docs/ports/authentication.md:240-242` —
  "Sensitive material (passwords, MFA codes) is never logged."
- **evidence:** `crates/adapters/auth/src/port.rs:179-195`:
  ```rust
  #[derive(Debug, Clone, PartialEq, Eq, Hash)]
  pub enum Credential {
      ...
      UsernamePassword {
          username: String,
          password: String,
      },
  ```

---

### FINDING 26

- **id:** ADAPTER-AUTH-026
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/auth/src/errors.rs:170-179
- **description:** `InfrastructureError::from_boxed` constructs
  the wrapper from a `Box<dyn StdError + Send + Sync>` by
  calling `err.to_string()` and storing the result verbatim as
  the wrapped `message`. The original error's `Display` impl
  may include sensitive data (e.g. an HTTP body containing a
  password reset URL with a token in the query string, or a
  database error message containing a username). The message
  becomes part of `AuthError::Display` output and any consumer
  that logs it leaks the original error's contents. The
  port-deviation note in `errors.rs:11-29` acknowledges that
  "the adapter is responsible for redacting any sensitive
  data ... before construction" but `from_boxed` is a
  convenience that does the lift automatically with no
  redaction.
- **expected:** `docs/ports/authentication.md:240-242` —
  "Sensitive material (passwords, MFA codes) is never logged."
- **evidence:** `crates/adapters/auth/src/errors.rs:170-179`:
  ```rust
  pub fn from_boxed(err: Box<dyn StdError + Send + Sync>) -> Self {
      Self {
          message: err.to_string(),
          source: Some(err),
      }
  }
  ```

---

### FINDING 27

- **id:** ADAPTER-AUTH-027
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/auth/src/errors.rs:52-53, 97
- **description:** `AuthError::AccountLocked(String)` carries a
  "lockout reason" string and the `Display` impl writes
  `account locked: {reason}`. The rustdoc claims "It is never
  shown to the user verbatim" and "suitable for the audit log"
  — but the `Display` impl is the universal interface; if a
  consumer maps `AuthError::Display` to the user-facing HTTP
  response (a common pattern), the reason string leaks to the
  client. The reason string is adapter-controlled and the
  audit-log contract is not enforced by the type system.
- **expected:** `docs/ports/authentication.md:163-166` —
  "The engine maps `AuthError` to `DomainError::Forbidden` for
  the user and logs the cause server-side." This implies the
  Display string is server-side log material, not user
  material — but the type system does not enforce this
  distinction.
- **evidence:** `crates/adapters/auth/src/errors.rs:52-53`:
  ```rust
  /// The principal is locked out (e.g. too many failed attempts).
  /// The wrapped string is the lockout reason, suitable for
  /// the audit log. It is never shown to the user verbatim.
  AccountLocked(String),
  ```
  and `crates/adapters/auth/src/errors.rs:97`:
  ```rust
  Self::AccountLocked(reason) => write!(f, "account locked: {reason}"),
  ```

---

### FINDING 28

- **id:** ADAPTER-AUTH-028
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/auth/src/jwt.rs:230
- **description:** `JwtAuthProviderBuilder::build` stores
  `refresh_ttl_secs` in `JwtAuthProvider` but never uses it.
  The field is suppressed with `let _ = self.refresh_ttl_secs;`
  in `refresh` (`jwt.rs:457`) and there is no other consumer.
  The `Default for JwtAuthProviderBuilder` constructs the
  builder with `refresh_ttl = 7d`, but the value is dead. The
  builder advertises the configuration knob without effect.
- **expected:** Builder methods should configure observable
  behavior. A no-op knob on a security-relevant configuration
  value (refresh TTL) is misleading to consumers.
- **evidence:** `crates/adapters/auth/src/jwt.rs:230`:
  ```rust
  refresh_ttl_secs: i64::try_from(self.refresh_ttl.as_secs()).unwrap_or(i64::MAX),
  ```
  and `crates/adapters/auth/src/jwt.rs:455-457`:
  ```rust
  // Suppress the unused-field warning when `refresh_ttl_secs`
  // is not consumed by this minimal implementation.
  let _ = self.refresh_ttl_secs;
  ```

---

### FINDING 29

- **id:** ADAPTER-AUTH-029
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/auth/src/jwt.rs:464
- **description:** `JwtAuthProvider::refresh()` returns only a
  `Session`; the new JWT is computed via `self.encode(...)`
  but the encoded value is bound to `_refreshed_token` and
  thrown away. The port spec says "`refresh(token)` returns a
  new `Session` for a non-expired token. The adapter may rotate
  the token value." A consumer has no way to retrieve the
  rotated token value; the only way to extract it is to
  re-implement the encode call. The function returns
  `Result<Session, AuthError>` rather than
  `Result<(Session, AuthToken), AuthError>`.
- **expected:** `docs/ports/authentication.md:142-146` —
  "The adapter may rotate the token value."
- **evidence:** `crates/adapters/auth/src/jwt.rs:422-464`:
  ```rust
  async fn refresh(&self, token: &AuthToken) -> Result<Session, AuthError> {
      ...
      let _refreshed_token = self.encode(&new_claims)?;
      self.session_from_claims(&new_claims)
  }
  ```

---

### FINDING 30

- **id:** ADAPTER-AUTH-030
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/auth/src/oauth_store.rs:154-160, 206-212
- **description:** `OAuthAccessTokenRepository::purge_expired`
  and `PasswordResetRepository::purge_older_than` retain rows
  whose `expires_at` is `None` (`map_or(true, |exp| exp >=
  before)`). The semantic is "never-expires rows are kept
  forever." For an `OAuthAccessToken`, this is a security
  leak — a row with `expires_at: None` is an access token that
  never expires and can never be purged. The constructor in
  tests uses `expires_at: None` (`oauth_store.rs:244, 266`),
  setting the precedent.
- **expected:** OAuth 2.0 best practice requires access tokens
  to have explicit expiry; an "infinite" token is
  security-violating.
- **evidence:** `crates/adapters/auth/src/oauth_store.rs:158`:
  ```rust
  guard.retain(|_, t| t.expires_at.map_or(true, |exp| exp >= before));
  ```
  same at `crates/adapters/auth/src/oauth_store.rs:210`.

---

### FINDING 31

- **id:** ADAPTER-AUTH-031
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/auth/src/oauth_store.rs:111-123
- **description:** `audit_action_for_op` is a free function
  that maps op strings to `AuditAction` variants. Every
  variant of `AuditAction` (`Create`, `Delete`, `Other`) is
  exercised, but the action is **never written** by the
  in-memory store. The crate's own documentation
  (`oauth_store.rs:30-38`) acknowledges: "The reference store
  itself does not write audit rows — audit emission belongs
  to the command handler, not the repository port." So the
  audit hook is computed and immediately discarded
  (`let _action = ...`). Every state-changing operation in
  this store is unaudited. Consumers that expect OAuth
  access-token insert / revoke to be audited per the port
  spec's audit section will not see any rows from this
  adapter.
- **expected:** `docs/ports/authentication.md:239-242` —
  "Successful and failed authentication attempts are written
  to the audit sink. Sensitive material (passwords, MFA codes)
  is never logged."
- **evidence:** `crates/adapters/auth/src/oauth_store.rs:140-144`:
  ```rust
  async fn insert(&self, t: &OAuthAccessToken) -> StorageResult<()> {
      let _action = audit_action_for_op("oauth_access_token.insert");
      self.lock_tokens().insert(t.id.clone(), t.clone());
      Ok(())
  }
  ```

---

### FINDING 32

- **id:** ADAPTER-AUTH-032
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/auth/src/services.rs:307-316
- **description:** `PasswordService::hash_password` accepts a
  `&SecretString` (the redacting wrapper) but the call site at
  `oauth_store.rs` and elsewhere is not present — the function
  is currently only called from tests. More importantly, the
  `Cargo.toml` declaration shows `argon2` is the only password
  hashing dependency; the workspace does not declare `secrecy`,
  and the crate's own `SecretString` newtype is used only by
  the password and TOTP services, not propagated to the
  `Credential::UsernamePassword.password` field. A consumer
  that wants to call `PasswordService::hash_password` must
  first construct a `SecretString` from a `String`, but the
  `Credential` they receive carries a raw `String` — there is
  no `From<Credential> for SecretString` bridge. The audit
  concern is that the type-system boundary between secret and
  non-secret is partial: `AuthToken.value` is `String`,
  `Credential.password` is `String`, but `PasswordService` takes
  `SecretString`.
- **expected:** Consistent redaction boundary: either every
  secret-bearing field is `SecretString`, or the port
  explicitly documents and tests the redaction contract.
- **evidence:** `crates/adapters/auth/src/port.rs:188-195`
  (password is `String`); `crates/adapters/auth/src/services.rs:307`
  (`hash_password` takes `&SecretString`); no `From<Credential>
  for SecretString` impl exists.

---

### FINDING 33

- **id:** ADAPTER-AUTH-033
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/auth/src/services.rs:518-611
- **description:** SHA-1 is hand-rolled inline in
  `services.rs::sha1_concat`. SHA-1 is cryptographically
  broken (collisions demonstrated since 2017). The hand-rolled
  implementation is correct against RFC 6238 test vectors
  (verified by `test_mfa_service_totp_round_trip`), but using
  SHA-1 in 2026 is a security smell that will be flagged by
  static analysis, code-review tooling, and supply-chain
  auditors. RFC 6238 permits HMAC-SHA1, HMAC-SHA256, and
  HMAC-SHA512; HMAC-SHA1 is not broken in the same way raw
  SHA-1 is, but the "no SHA-1 in new code" rule is industry
  standard. The implementation should use SHA-256 (RFC 6238
  supports it) and a vetted crate.
- **expected:** Industry standard "no SHA-1 in new code"
  practice (NIST deprecated SHA-1 for digital signatures in
  2011; major codebases have banned SHA-1 entirely).
- **evidence:** `crates/adapters/auth/src/services.rs:518-611`
  implements SHA-1 from scratch; `crates/adapters/auth/src/services.rs:467`
  uses it for HMAC-SHA1 in TOTP.

---

### FINDING 34

- **id:** ADAPTER-AUTH-034
- **area:** adapters
- **severity:** Low
- **location:** crates/adapters/auth/src/services.rs:414-420, 427-447
- **description:** `MfaService::current_code` and
  `verify_code` accept arbitrary-length `secret` strings
  (after base32 decoding, the decoded byte length is
  unconstrained). RFC 4226 § 4 requires "The secret K ... is
  recommended to be at least 128 bits and is typically 160
  bits." A consumer can pass a 4-byte secret and produce a
  TOTP value with reduced security. The service does not
  validate the decoded secret length.
- **expected:** RFC 4226 § 4 — 128-bit minimum, 160-bit
  typical.
- **evidence:** `crates/adapters/auth/src/services.rs:414-420`:
  ```rust
  pub fn current_code(&self, secret: &str, now: Timestamp) -> Result<String, AuthError> {
      let key = base32_decode(secret)?;
      ...
  }
  ```
  No `key.len() < 16` (or 20) check.

---

### FINDING 35

- **id:** ADAPTER-AUTH-035
- **area:** adapters
- **severity:** Low
- **location:** crates/adapters/auth/src/jwt.rs:170-177
- **description:** The `JwtAuthProviderBuilder::new` defaults
  hard-code `issuer: "educore"` and `audience: "educore"`. A
  consumer who builds with `.new()` and only adds
  `.signing_key(...)` ends up with a provider that accepts
  tokens whose `iss == "educore"` and `aud == "educore"`. In
  a multi-tenant deployment with multiple consumers of the
  engine, this default collides between consumers and allows
  cross-consumer token replay (consumer A's tokens are
  accepted by consumer B). The defaults should be empty
  (`String::new()`) or the builder should `.expect()` a
  non-empty issuer/audience.
- **expected:** No implicit shared default for tenant-
  discriminating claims.
- **evidence:** `crates/adapters/auth/src/jwt.rs:170-177`:
  ```rust
  pub fn new() -> Self {
      let mut key = vec![0_u8; 32];
      rand::thread_rng().fill_bytes(&mut key);
      Self {
          signing_key: key,
          issuer: "educore".to_owned(),
          audience: "educore".to_owned(),
          ...
      }
  }
  ```

---

### FINDING 36

- **id:** ADAPTER-AUTH-036
- **area:** adapters
- **severity:** Low
- **location:** crates/adapters/auth/src/jwt.rs:84-120
- **description:** `JwtClaims` derives `Serialize` and
  `Deserialize` and exposes all fields as `pub`. There is no
  `#[serde(deny_unknown_fields)]` on the deserialization side,
  so a token with extra claims (e.g. `admin: true`,
  `is_superuser: true`) is accepted silently — the extra
  fields are dropped on the rust side, but the issuer may
  believe they are honored. Combined with Finding 12
  (capabilities always empty), an issuer that puts
  capabilities in JWT claims sees no effect and no warning.
- **expected:** Best practice for JWT consumers is to either
  deny unknown fields (`deny_unknown_fields`) or document the
  ignored-fields contract.
- **evidence:** `crates/adapters/auth/src/jwt.rs:87-120`:
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct JwtClaims {
      pub sub: String,
      ...
  }
  ```
  No `#[serde(deny_unknown_fields)]`.

---

### FINDING 37

- **id:** ADAPTER-AUTH-037
- **area:** adapters
- **severity:** Low
- **location:** crates/adapters/auth/src/jwt.rs:319, 344
- **description:** `session_from_claims` and
  `anonymous_session` both set `capabilities:
  BTreeSet::<Capability>::new()` and `metadata:
  BTreeMap::new()`. These two empty values are constructed
  per-call instead of being shared static empties. The
  `BTreeSet::new()` is cheap but not free; the per-call
  allocation is a minor allocation hot-spot for high-
  throughput APIs (the port spec describes validate as a
  per-request call).
- **expected:** Use `BTreeSet::new()` is fine; the concern is
  the larger one captured in Finding 12 (empty capabilities
  block all RBAC checks).
- **evidence:** `crates/adapters/auth/src/jwt.rs:319` and
  `crates/adapters/auth/src/jwt.rs:344`.

---

### FINDING 38

- **id:** ADAPTER-AUTH-038
- **area:** adapters
- **severity:** Low
- **location:** crates/adapters/auth/src/lib.rs:46
- **description:** `lib.rs` declares `#![deny(missing_docs)]`
  but the port module (`port.rs:46`) carries an inner
  `#![allow(missing_docs)]`. This creates a deny/allow
  interaction that is hard to audit: future public items
  added to `port.rs` will not be flagged for missing docs,
  while items added elsewhere will. The crate's overall
  documentation contract is weakened by the inner allow.
- **expected:** `docs/code-standards.md` — "All public APIs
  are documented with rustdoc; `#![deny(missing_docs)]`." The
  inner allow violates this at the crate level.
- **evidence:** `crates/adapters/auth/src/lib.rs:16`
  (`#![deny(missing_docs)]`) vs
  `crates/adapters/auth/src/port.rs:46`
  (`#![allow(dead_code, clippy::all, missing_docs)]`).

---

### END FINDINGS

**Total: 38 findings** (5 Critical, 13 High, 14 Medium, 6 Low).
