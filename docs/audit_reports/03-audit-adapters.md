# 03 - Audit Appendix - Adapters (10 crates)

**Scope:** wave3-auth.md, wave3-files.md, wave3-integrations.md, wave3-notify.md, wave3-payment.md, wave3-storage-postgres.md, wave3-storage-mysql.md, wave3-storage-sqlite.md, wave3-storage-surrealdb.md, wave3-event-bus.md

**Total findings:** 387

**Severity distribution:** 74 critical, 133 high, 134 medium, 46 low


## Summary Table

| Target | Critical | High | Medium | Low | Total |
| --- | --- | --- | --- | --- | --- |
| Auth (`ADAPTER-AUTH`) | 6 | 12 | 15 | 5 | 38 |
| Files (`ADAPTER-FILE`) | 5 | 13 | 9 | 1 | 28 |
| Integrations (`ADAPTER-INT`) | 5 | 10 | 20 | 7 | 42 |
| Notify (`ADAPTER-NOT`) | 12 | 24 | 31 | 7 | 74 |
| Payment (`ADAPT-PAY`) | 7 | 11 | 5 | 1 | 24 |
| Storage — PostgreSQL (`ADAPTER-PG`) | 13 | 13 | 15 | 6 | 47 |
| Storage — MySQL (`ADAPT-MY`) | 5 | 7 | 9 | 3 | 24 |
| Storage — SQLite (`ADAPTER-SQ`) | 5 | 14 | 15 | 16 | 50 |
| Storage — SurrealDB (`ADAPTER-SR`) | 11 | 21 | 6 | 0 | 38 |
| Event Bus (`ADAPT-EB`) | 5 | 8 | 9 | 0 | 22 |

## Auth (target id prefix: `ADAPTER-AUTH`)

**Path:** `crates/adapters/auth/`  
**Total findings:** 38 (6 critical, 12 high, 15 medium, 5 low)


### FINDING 1 (id: `ADAPTER-AUTH-001`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** crates/adapters/auth/src/jwt.rs:331-350, 389

**Description:**

`JwtAuthProvider::authenticate` accepts
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

**Expected:**

`docs/ports/authentication.md:38-40` — "A
  `Credential::Anonymous` is rejected by the default adapters
  except in public-facing flows (e.g. public exam result lookup,
  when explicitly allowed by configuration)."

**Evidence:**

`crates/adapters/auth/src/jwt.rs:389` —
  ```rust
  Credential::Anonymous => Ok(self.anonymous_session()),
  ```
  with `mfa_satisfied: true` at `crates/adapters/auth/src/jwt.rs:345`,
  empty capabilities at `crates/adapters/auth/src/jwt.rs:344`.

---

### FINDING 2 (id: `ADAPTER-AUTH-002`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** crates/adapters/auth/src/jwt.rs:167-176

**Description:**

`JwtAuthProviderBuilder::new()` generates a
  fresh 32-byte random signing key via `rand::thread_rng()` on
  every call. The crate's own port-deviation note acknowledges
  this, but the builder provides **no warning** when a consumer
  uses the default key. If a consumer forgets to call
  `.signing_key(env::var("JWT_SECRET")?)`, every process restart
  invalidates every previously-issued JWT, every replica in a
  horizontally-scaled deployment signs with a different key, and
  the dev `JwtAuthProviderBuilder::new()` works fine in single-
  process tests but silently breaks in production.

**Expected:**

`docs/ports/authentication.md:175-180` —
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

**Evidence:**

`crates/adapters/auth/src/jwt.rs:167-176`:
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

### FINDING 3 (id: `ADAPTER-AUTH-003`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** crates/adapters/auth/src/jwt.rs:19-23, 142, 231

**Description:**

Token revocation uses a process-local
  `Arc<Mutex<HashSet<String>>>` keyed by `sid`. The crate
  documentation explicitly acknowledges "the set is process-
  local; consumers that need cross-process revocation must
  layer a shared store on top" — but no shared-store
  implementation is provided. In any horizontally-scaled or
  multi-replica deployment, revoking a token on replica A
  leaves the same token valid on replicas B, C, D. The
  revocation mechanism is therefore broken for the production
  multi-replica case the engine targets.

**Expected:**

`docs/ports/authentication.md:134-140` —
  "`revoke(token)` invalidates the token. The adapter updates
  its session store. Subsequent `validate` calls return
  `AuthError::Revoked`. A super-admin can also revoke all
  sessions for a user (e.g. after password change or suspected
  compromise)." This implies a cross-process session store.

**Evidence:**

`crates/adapters/auth/src/jwt.rs:142` —
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

### FINDING 4 (id: `ADAPTER-AUTH-004`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** crates/adapters/auth/src/jwt.rs:422-464

**Description:**

`JwtAuthProvider::refresh()` does **not** mint
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

**Expected:**

`docs/ports/authentication.md:142-146` — "`refresh(token)`
  returns a new `Session` for a non-expired token. The adapter
  may rotate the token value. The old token is invalidated."

**Evidence:**

`crates/adapters/auth/src/jwt.rs:455-463`:
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

### FINDING 5 (id: `ADAPTER-AUTH-005`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** crates/adapters/auth/src/jwt.rs:441-453

**Description:**

`JwtAuthProvider::refresh()` sets
  `sid: old_claims.sid` and does **not** call `add_revoked`.
  The comment at line 446-447 admits "We do NOT add the sid to
  the revocation set on refresh; instead, callers who want
  strict token rotation should call `revoke` on the old token
  explicitly." This is a port-spec violation: spec says "The
  old token is invalidated." The current behavior allows the
  old JWT to remain valid indefinitely (until natural
  expiration), so a stolen old JWT continues to grant access
  for up to `access_ttl_secs` after refresh.

**Expected:**

`docs/ports/authentication.md:142-146` —
  "The old token is invalidated."

**Evidence:**

`crates/adapters/auth/src/jwt.rs:448` —
  ```rust
  sid: old_claims.sid,
  ```
  and absence of any `add_revoked(&old_claims.sid)` call in
  the refresh body (lines 422-464).

---

### FINDING 6 (id: `ADAPTER-AUTH-006`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** crates/adapters/auth/src/jwt.rs:182-185

**Description:**

`JwtAuthProviderBuilder::signing_key` accepts
  arbitrary byte slices without minimum-length validation.
  RFC 7518 § 3.2 requires "A key of the same size as the hash
  output (for instance, 256 bits for "HS256") or larger MUST be
  used with this algorithm." The builder accepts
  `b"too-short"` and would silently produce tokens signed with
  a weak key. The `JwtAuthProviderBuilder::new` default is
  32 bytes (correct), but a consumer can pass any length via
  `signing_key(...)` and the builder does not reject short
  keys.

**Expected:**

RFC 7518 § 3.2 (cited via JWT spec) and
  OWASP/JWT security best practice require minimum-length
  validation on HMAC keys.

**Evidence:**

`crates/adapters/auth/src/jwt.rs:181-185`:
  ```rust
  #[must_use]
  pub fn signing_key(mut self, key: impl Into<Vec<u8>>) -> Self {
      self.signing_key = key.into();
      self
  }
  ```
  No `key.len() < 32` check.

---

### FINDING 10 (id: `ADAPTER-AUTH-010`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/auth/src/jwt.rs:442

**Description:**

`refresh()` computes the new `exp` claim via
  `now.saturating_add(self.access_ttl_secs)`. On `i64::MAX`
  overflow (which `Finding 7` makes reachable) the saturation
  clamps to `i64::MAX`, producing a JWT whose `exp` claim is
  effectively "never expires" — bypassing the entire TTL
  mechanism. Combined with Finding 5 (no revocation on
  refresh), a token issued through `refresh()` can become
  permanently valid.

**Expected:**

`docs/ports/authentication.md` § "Session" —
  `expires_at` is a bounded timestamp; the spec assumes the
  issuer enforces TTL.

**Evidence:**

`crates/adapters/auth/src/jwt.rs:442`:
  ```rust
  exp: now.saturating_add(self.access_ttl_secs),
  ```

---

### FINDING 11 (id: `ADAPTER-AUTH-011`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/auth/src/port.rs:46

**Description:**

`port.rs` declares
  `#![allow(dead_code, clippy::all, missing_docs)]`. While
  `lib.rs` has `#![deny(missing_docs)]`, this per-file `allow`
  suppresses the lint for every public item in the **port**
  module — the module that defines `AuthProvider`, `RbacPort`,
  `Session`, `AuthToken`, `AuthScheme`, `Credential`. Several
  fields in `Credential` (`code_verifier`, `relay_state`,
  `Biometric.signature`) have rustdoc, but other public items
  in `port.rs` may not. The deny/allow interaction is brittle.

**Expected:**

`docs/code-standards.md` (AGENTS.md § "Code
  Standards") — "All public APIs are documented with
  rustdoc; `#![deny(missing_docs)]`."

**Evidence:**

`crates/adapters/auth/src/port.rs:46`:
  ```rust
  #![allow(dead_code, clippy::all, missing_docs)]
  ```

---

### FINDING 12 (id: `ADAPTER-AUTH-012`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/auth/src/jwt.rs:285-325

**Description:**

`JwtAuthProvider::session_from_claims` sets
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

**Expected:**

`docs/ports/authentication.md:58-62` —
  "`Session` is a value type. It carries everything the engine
  needs to authorize and tenant-isolate a command. Capabilities
  are pre-computed when the session is created; the engine does
  not consult the RBAC storage on every command."

**Evidence:**

`crates/adapters/auth/src/jwt.rs:319`:
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

### FINDING 13 (id: `ADAPTER-AUTH-013`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/auth/src/jwt.rs:380-465

**Description:**

`JwtAuthProvider` only handles two of seven
  `Credential` variants: `Bearer` and `Anonymous`. The other
  five (`UsernamePassword`, `Oauth2`, `Saml`, `ApiKey`,
  `Biometric`) are all rejected with `AuthError::InvalidCredentials`.
  The port spec requires the trait to be the universal entry
  point for all authentication modes, but this adapter
  implements only JWT. The same trait ships from the same crate
  with no alternative adapter for the rejected variants — the
  port surface advertises support for these credential types
  but the only shipped impl uniformly rejects them.

**Expected:**

`docs/ports/authentication.md:184-191` —
  "Alternative adapters: `LocalPasswordAuthProvider` —
  username + password against a local user table (hashed with
  argon2). `OAuth2AuthProvider` — external OAuth2/OIDC ...,
  `SamlAuthProvider` — SAML 2.0, `ApiKeyAuthProvider` —
  service-to-service auth." These are described as part of
  the crate's deliverable.

**Evidence:**

`crates/adapters/auth/src/jwt.rs:382-396`:
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

### FINDING 14 (id: `ADAPTER-AUTH-014`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/auth/src/errors.rs:46-91

**Description:**

`AuthError` declares 10 variants
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

**Expected:**

`docs/ports/authentication.md:220-230` —
  "Integration tests for ... A test for MFA required /
  satisfied. A test for rate limiting. A test for
  infrastructure failure."

**Evidence:**

`crates/adapters/auth/src/errors.rs:46-91`
  declares all 10 variants. Grep for `MfaRequired`,
  `RateLimited`, `AccountLocked`, `AccountDisabled` in
  `crates/adapters/auth/src/` returns only the declaration
  site (no production usage).

---

### FINDING 15 (id: `ADAPTER-AUTH-015`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/auth/src/jwt.rs:400-403, 411-415, 423-427

**Description:**

The JWT provider's `validate`, `revoke`, and
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

**Expected:**

`docs/ports/authentication.md:240-242` —
  "Sensitive material (passwords, MFA codes) is never logged."
  This is the principle; the same principle applies to PII
  claims.

**Evidence:**

`crates/adapters/auth/src/jwt.rs:499-521`:
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

### FINDING 16 (id: `ADAPTER-AUTH-016`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/auth/src/jwt.rs:266-270, 499-521

**Description:**

`encode` formats the jsonwebtoken encode
  error verbatim: `format!("jwt encode failed: {e}")`. The
  `jsonwebtoken` crate's encode error kinds include
  `Json(serde_json::Error)` which carries the input JSON
  content (i.e. the full claim set, including user id, role
  ids, school ids, session id, MFA flag). A consumer that
  logs `AuthError::Display` output on the encode failure path
  leaks the full claim set.

**Expected:**

`docs/ports/authentication.md:240-242` —
  "Sensitive material (passwords, MFA codes) is never logged."

**Evidence:**

`crates/adapters/auth/src/jwt.rs:266-270`:
  ```rust
  fn encode(&self, claims: &JwtClaims) -> Result<String, AuthError> {
      let key = EncodingKey::from_secret(self.signing_key.as_slice());
      encode(&Header::new(Algorithm::HS256), claims, &key)
          .map_err(|e| AuthError::Malformed(format!("jwt encode failed: {e}")))
  }
  ```

---

### FINDING 17 (id: `ADAPTER-AUTH-017`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/auth/src/jwt.rs:329-349

**Description:**

`anonymous_session()` produces a session
  with `mfa_satisfied: true`. The port spec says MFA is
  per-session and "When a session is `mfa_satisfied = false`,
  the engine restricts sensitive commands." The anonymous
  session is short-circuiting MFA entirely — every command
  the system user executes is treated as MFA-satisfied, with
  no second-factor check. The session uses `SYSTEM_USER_ID`,
  which presumably has super-admin capabilities in the RBAC
  catalog.

**Expected:**

`docs/ports/authentication.md:116-132` —
  "`mfa_satisfied = false`, the engine restricts sensitive
  commands. The adapter decides which commands require MFA
  based on configuration."

**Evidence:**

`crates/adapters/auth/src/jwt.rs:345`:
  ```rust
  mfa_satisfied: true,
  ```
  inside the `anonymous_session()` function (line 331) which
  produces the system-user session for `Credential::Anonymous`.

---

### FINDING 18 (id: `ADAPTER-AUTH-018`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/auth/src/jwt.rs:253-254

**Description:**

`Validation::leeway = 0` is hard-coded.
  Clock skew between issuer and validator is normal in
  distributed systems (NTP drift, container clock
  virtualization). A 0-second leeway causes false rejections
  when the issuer's clock is even a fraction ahead of the
  validator's, especially for short-lived tokens. The
  `jsonwebtoken` crate's default leeway is 60 seconds; the
  port's reference implementation is stricter than the
  upstream default with no override knob on the builder.

**Expected:**

Standard JWT issuance practice and the
  upstream `jsonwebtoken` crate's default leeway of 60s.

**Evidence:**

`crates/adapters/auth/src/jwt.rs:249-254`:
  ```rust
  fn validation(&self) -> Validation {
      let mut v = Validation::new(Algorithm::HS256);
      v.set_issuer(&[self.issuer.as_str()]);
      v.set_audience(&[self.audience.as_str()]);
      v.validate_exp = true;
      v.leeway = 0;
  ```

---

### FINDING 7 (id: `ADAPTER-AUTH-007`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/auth/src/jwt.rs:229-230

**Description:**

`JwtAuthProviderBuilder::build` silently caps
  `access_ttl_secs` and `refresh_ttl_secs` to `i64::MAX` when
  the `Duration::as_secs()` value cannot be represented as
  `i64`. This swallows a logically-impossible input (a
  `Duration` longer than ~292 billion years) and produces a
  provider with effectively infinite TTL — a security-relevant
  silent failure. The `unwrap_or(i64::MAX)` pattern is used in
  production code paths (no `#[cfg(test)]` or `#[allow]`).

**Expected:**

`docs/code-standards.md` (AGENTS.md § "Type
  Safety") — "No `as` casts that truncate or lose data. Use
  `TryFrom` / `TryInto` with proper error handling." Silent
  saturation is a stricter violation of this rule.

**Evidence:**

`crates/adapters/auth/src/jwt.rs:229-230`:
  ```rust
  access_ttl_secs: i64::try_from(self.access_ttl.as_secs()).unwrap_or(i64::MAX),
  refresh_ttl_secs: i64::try_from(self.refresh_ttl.as_secs()).unwrap_or(i64::MAX),
  ```

---

### FINDING 8 (id: `ADAPTER-AUTH-008`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/auth/src/jwt.rs:354-373

**Description:**

`add_revoked` and `check_not_revoked` both
  recover from `PoisonError` via
  `.unwrap_or_else(PoisonError::into_inner)`. A poisoned mutex
  indicates a panic in another thread while the lock was held;
  silently recovering and continuing to operate on potentially
  inconsistent state hides concurrency bugs. The `unwrap_or_else`
  pattern is in production paths (no `#[cfg(test)]`).

**Expected:**

`docs/code-standards.md` (AGENTS.md § "Type
  Safety") — "No `unwrap()` or `expect()` in production paths."
  `unwrap_or_else(PoisonError::into_inner)` is functionally a
  silent unwrap.

**Evidence:**

`crates/adapters/auth/src/jwt.rs:355-359`:
  ```rust
  let mut set = self
      .revoked_sessions
      .lock()
      .unwrap_or_else(std::sync::PoisonError::into_inner);
  set.insert(sid.to_owned());
  ```
  Same pattern at `crates/adapters/auth/src/jwt.rs:366-368`.

---

### FINDING 9 (id: `ADAPTER-AUTH-009`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/auth/src/jwt.rs:332-337

**Description:**

`anonymous_session` computes the expiry via
  `.single().unwrap_or_else(|| now.as_datetime())`. If
  `Utc.timestamp_opt` returns `None` or `Ambiguous` for the
  computed expiry instant, the function silently falls back to
  "now" — issuing a session that is already expired. This is
  in production code (no `#[cfg(test)]` or `#[allow]`).

**Expected:**

`docs/code-standards.md` (AGENTS.md § "Type
  Safety") — propagation of errors is the standard.

**Evidence:**

`crates/adapters/auth/src/jwt.rs:332-337`:
  ```rust
  let now = Timestamp::now();
  let exp = Timestamp::from_datetime(
      Utc.timestamp_opt(now.as_datetime().timestamp() + self.access_ttl_secs, 0)
          .single()
          .unwrap_or_else(|| now.as_datetime()),
  );
  ```

---

### FINDING 19 (id: `ADAPTER-AUTH-019`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/auth/tests/auth_integration.rs:97-105

**Description:**

The only async integration test
  (`auth_integration_async_jwt_full_round_trip`) is `#[ignore]`
  gated behind `EDUCORE_PORT_ADAPTER_E2E env var`. The
  remaining async test (`auth_integration_async_password_rehash_check`)
  is also `#[ignore]`-gated. CI runs only 5 sync tests. A full
  Bearer round-trip (authenticate → validate → refresh →
  revoke) is **never** exercised by the test suite as it
  ships. The handoff documents this as OQ #1 but the gap is
  not closed in code.

**Expected:**

`docs/ports/authentication.md:220-230` —
  "Integration tests for token issue, validate, refresh,
  revoke."

**Evidence:**

`crates/adapters/auth/tests/auth_integration.rs:97-105`:
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

### FINDING 20 (id: `ADAPTER-AUTH-020`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/auth/tests/auth_integration.rs (full file)

**Description:**

The test file contains 5 sync tests
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

**Expected:**

`docs/ports/authentication.md:220-230` —
  "Unit tests of every `Credential` variant. Integration tests
  for token issue, validate, refresh, revoke. A test for
  expired and revoked tokens. A test for cross-tenant denial.
  A test for MFA required / satisfied. A test for rate
  limiting. A test for infrastructure failure."

**Evidence:**

`crates/adapters/auth/tests/auth_integration.rs`
  contains 114 lines; only the scenarios listed above are
  covered.

---

### FINDING 21 (id: `ADAPTER-AUTH-021`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/auth/src/services.rs:340-374

**Description:**

`PasswordService::needs_rehash` compares
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

**Expected:**

Function-name contract `needs_rehash` implies
  detection of out-of-date parameters, not only out-of-date
  algorithms.

**Evidence:**

`crates/adapters/auth/src/services.rs:346-374`:
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

### FINDING 22 (id: `ADAPTER-AUTH-022`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/auth/src/services.rs:307-335

**Description:**

Argon2 errors during `hash_password` and
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

**Expected:**

`docs/ports/authentication.md:148-163` —
  `AuthError::Infrastructure` is the canonical variant for
  "A non-domain infrastructure failure (network, DNS, TLS,
  IdP, JWKS fetch)."

**Evidence:**

`crates/adapters/auth/src/services.rs:312-315`:
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

### FINDING 23 (id: `ADAPTER-AUTH-023`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/auth/src/services.rs:391-449

**Description:**

The crate ships only one MFA factor
  implementation: RFC 6238 TOTP (HMAC-SHA1, 8-digit). The port
  spec lists five second-factor mechanisms: TOTP, SMS code,
  Email code, WebAuthn/FIDO2, and Backup code. Only one is
  implemented; the other four are absent. The `AuthError`
  variants `MfaRequired` and `MfaFailed` are declared but
  never emitted by any code path in this crate.

**Expected:**

`docs/ports/authentication.md:122-132` —
  "A second factor may be satisfied by: TOTP (RFC 6238),
  SMS code, Email code, WebAuthn / FIDO2, Backup code."

**Evidence:**

`crates/adapters/auth/src/services.rs:381-449`
  contains only `MfaService` (TOTP). Grep for SMS, FIDO2,
  WebAuthn, Backup in `crates/adapters/auth/src/` returns no
  matches. Grep for `MfaRequired` returns only the declaration
  site (`crates/adapters/auth/src/errors.rs:75`).

---

### FINDING 24 (id: `ADAPTER-AUTH-024`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/auth/src/port.rs:158-167

**Description:**

`AuthToken.value` is a plain `String`, not
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

**Expected:**

`docs/ports/authentication.md:240-242` —
  "Sensitive material (passwords, MFA codes) is never
  logged." The port exposes the same risk for tokens.

**Evidence:**

`crates/adapters/auth/src/port.rs:157-167`:
  ```rust
  #[derive(Debug, Clone, PartialEq, Eq, Hash)]
  pub struct AuthToken {
      pub scheme: AuthScheme,
      pub value: String,
      pub metadata: BTreeMap<String, String>,
  }
  ```

---

### FINDING 25 (id: `ADAPTER-AUTH-025`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/auth/src/port.rs:188-195

**Description:**

`Credential::UsernamePassword.password` is a
  plain `String`. The `Debug` impl on `Credential`
  (`port.rs:179`) auto-derives `Debug`, so
  `format!("{credential:?}")` in any consumer will print the
  plaintext password. The port-deviation note documents this
  but does not mitigate it.

**Expected:**

`docs/ports/authentication.md:240-242` —
  "Sensitive material (passwords, MFA codes) is never logged."

**Evidence:**

`crates/adapters/auth/src/port.rs:179-195`:
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

### FINDING 26 (id: `ADAPTER-AUTH-026`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/auth/src/errors.rs:170-179

**Description:**

`InfrastructureError::from_boxed` constructs
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

**Expected:**

`docs/ports/authentication.md:240-242` —
  "Sensitive material (passwords, MFA codes) is never logged."

**Evidence:**

`crates/adapters/auth/src/errors.rs:170-179`:
  ```rust
  pub fn from_boxed(err: Box<dyn StdError + Send + Sync>) -> Self {
      Self {
          message: err.to_string(),
          source: Some(err),
      }
  }
  ```

---

### FINDING 27 (id: `ADAPTER-AUTH-027`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/auth/src/errors.rs:52-53, 97

**Description:**

`AuthError::AccountLocked(String)` carries a
  "lockout reason" string and the `Display` impl writes
  `account locked: {reason}`. The rustdoc claims "It is never
  shown to the user verbatim" and "suitable for the audit log"
  — but the `Display` impl is the universal interface; if a
  consumer maps `AuthError::Display` to the user-facing HTTP
  response (a common pattern), the reason string leaks to the
  client. The reason string is adapter-controlled and the
  audit-log contract is not enforced by the type system.

**Expected:**

`docs/ports/authentication.md:163-166` —
  "The engine maps `AuthError` to `DomainError::Forbidden` for
  the user and logs the cause server-side." This implies the
  Display string is server-side log material, not user
  material — but the type system does not enforce this
  distinction.

**Evidence:**

`crates/adapters/auth/src/errors.rs:52-53`:
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

### FINDING 28 (id: `ADAPTER-AUTH-028`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/auth/src/jwt.rs:230

**Description:**

`JwtAuthProviderBuilder::build` stores
  `refresh_ttl_secs` in `JwtAuthProvider` but never uses it.
  The field is suppressed with `let _ = self.refresh_ttl_secs;`
  in `refresh` (`jwt.rs:457`) and there is no other consumer.
  The `Default for JwtAuthProviderBuilder` constructs the
  builder with `refresh_ttl = 7d`, but the value is dead. The
  builder advertises the configuration knob without effect.

**Expected:**

Builder methods should configure observable
  behavior. A no-op knob on a security-relevant configuration
  value (refresh TTL) is misleading to consumers.

**Evidence:**

`crates/adapters/auth/src/jwt.rs:230`:
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

### FINDING 29 (id: `ADAPTER-AUTH-029`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/auth/src/jwt.rs:464

**Description:**

`JwtAuthProvider::refresh()` returns only a
  `Session`; the new JWT is computed via `self.encode(...)`
  but the encoded value is bound to `_refreshed_token` and
  thrown away. The port spec says "`refresh(token)` returns a
  new `Session` for a non-expired token. The adapter may rotate
  the token value." A consumer has no way to retrieve the
  rotated token value; the only way to extract it is to
  re-implement the encode call. The function returns
  `Result<Session, AuthError>` rather than
  `Result<(Session, AuthToken), AuthError>`.

**Expected:**

`docs/ports/authentication.md:142-146` —
  "The adapter may rotate the token value."

**Evidence:**

`crates/adapters/auth/src/jwt.rs:422-464`:
  ```rust
  async fn refresh(&self, token: &AuthToken) -> Result<Session, AuthError> {
      ...
      let _refreshed_token = self.encode(&new_claims)?;
      self.session_from_claims(&new_claims)
  }
  ```

---

### FINDING 30 (id: `ADAPTER-AUTH-030`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/auth/src/oauth_store.rs:154-160, 206-212

**Description:**

`OAuthAccessTokenRepository::purge_expired`
  and `PasswordResetRepository::purge_older_than` retain rows
  whose `expires_at` is `None` (`map_or(true, |exp| exp >=
  before)`). The semantic is "never-expires rows are kept
  forever." For an `OAuthAccessToken`, this is a security
  leak — a row with `expires_at: None` is an access token that
  never expires and can never be purged. The constructor in
  tests uses `expires_at: None` (`oauth_store.rs:244, 266`),
  setting the precedent.

**Expected:**

OAuth 2.0 best practice requires access tokens
  to have explicit expiry; an "infinite" token is
  security-violating.

**Evidence:**

`crates/adapters/auth/src/oauth_store.rs:158`:
  ```rust
  guard.retain(|_, t| t.expires_at.map_or(true, |exp| exp >= before));
  ```
  same at `crates/adapters/auth/src/oauth_store.rs:210`.

---

### FINDING 31 (id: `ADAPTER-AUTH-031`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/auth/src/oauth_store.rs:111-123

**Description:**

`audit_action_for_op` is a free function
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

**Expected:**

`docs/ports/authentication.md:239-242` —
  "Successful and failed authentication attempts are written
  to the audit sink. Sensitive material (passwords, MFA codes)
  is never logged."

**Evidence:**

`crates/adapters/auth/src/oauth_store.rs:140-144`:
  ```rust
  async fn insert(&self, t: &OAuthAccessToken) -> StorageResult<()> {
      let _action = audit_action_for_op("oauth_access_token.insert");
      self.lock_tokens().insert(t.id.clone(), t.clone());
      Ok(())
  }
  ```

---

### FINDING 32 (id: `ADAPTER-AUTH-032`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/auth/src/services.rs:307-316

**Description:**

`PasswordService::hash_password` accepts a
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

**Expected:**

Consistent redaction boundary: either every
  secret-bearing field is `SecretString`, or the port
  explicitly documents and tests the redaction contract.

**Evidence:**

`crates/adapters/auth/src/port.rs:188-195`
  (password is `String`); `crates/adapters/auth/src/services.rs:307`
  (`hash_password` takes `&SecretString`); no `From<Credential>
  for SecretString` impl exists.

---

### FINDING 33 (id: `ADAPTER-AUTH-033`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/auth/src/services.rs:518-611

**Description:**

SHA-1 is hand-rolled inline in
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

**Expected:**

Industry standard "no SHA-1 in new code"
  practice (NIST deprecated SHA-1 for digital signatures in
  2011; major codebases have banned SHA-1 entirely).

**Evidence:**

`crates/adapters/auth/src/services.rs:518-611`
  implements SHA-1 from scratch; `crates/adapters/auth/src/services.rs:467`
  uses it for HMAC-SHA1 in TOTP.

---

### FINDING 34 (id: `ADAPTER-AUTH-034`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** crates/adapters/auth/src/services.rs:414-420, 427-447

**Description:**

`MfaService::current_code` and
  `verify_code` accept arbitrary-length `secret` strings
  (after base32 decoding, the decoded byte length is
  unconstrained). RFC 4226 § 4 requires "The secret K ... is
  recommended to be at least 128 bits and is typically 160
  bits." A consumer can pass a 4-byte secret and produce a
  TOTP value with reduced security. The service does not
  validate the decoded secret length.

**Expected:**

RFC 4226 § 4 — 128-bit minimum, 160-bit
  typical.

**Evidence:**

`crates/adapters/auth/src/services.rs:414-420`:
  ```rust
  pub fn current_code(&self, secret: &str, now: Timestamp) -> Result<String, AuthError> {
      let key = base32_decode(secret)?;
      ...
  }
  ```
  No `key.len() < 16` (or 20) check.

---

### FINDING 35 (id: `ADAPTER-AUTH-035`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** crates/adapters/auth/src/jwt.rs:170-177

**Description:**

The `JwtAuthProviderBuilder::new` defaults
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

**Expected:**

No implicit shared default for tenant-
  discriminating claims.

**Evidence:**

`crates/adapters/auth/src/jwt.rs:170-177`:
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

### FINDING 36 (id: `ADAPTER-AUTH-036`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** crates/adapters/auth/src/jwt.rs:84-120

**Description:**

`JwtClaims` derives `Serialize` and
  `Deserialize` and exposes all fields as `pub`. There is no
  `#[serde(deny_unknown_fields)]` on the deserialization side,
  so a token with extra claims (e.g. `admin: true`,
  `is_superuser: true`) is accepted silently — the extra
  fields are dropped on the rust side, but the issuer may
  believe they are honored. Combined with Finding 12
  (capabilities always empty), an issuer that puts
  capabilities in JWT claims sees no effect and no warning.

**Expected:**

Best practice for JWT consumers is to either
  deny unknown fields (`deny_unknown_fields`) or document the
  ignored-fields contract.

**Evidence:**

`crates/adapters/auth/src/jwt.rs:87-120`:
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct JwtClaims {
      pub sub: String,
      ...
  }
  ```
  No `#[serde(deny_unknown_fields)]`.

---

### FINDING 37 (id: `ADAPTER-AUTH-037`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** crates/adapters/auth/src/jwt.rs:319, 344

**Description:**

`session_from_claims` and
  `anonymous_session` both set `capabilities:
  BTreeSet::<Capability>::new()` and `metadata:
  BTreeMap::new()`. These two empty values are constructed
  per-call instead of being shared static empties. The
  `BTreeSet::new()` is cheap but not free; the per-call
  allocation is a minor allocation hot-spot for high-
  throughput APIs (the port spec describes validate as a
  per-request call).

**Expected:**

Use `BTreeSet::new()` is fine; the concern is
  the larger one captured in Finding 12 (empty capabilities
  block all RBAC checks).

**Evidence:**

`crates/adapters/auth/src/jwt.rs:319` and
  `crates/adapters/auth/src/jwt.rs:344`.

---

### FINDING 38 (id: `ADAPTER-AUTH-038`)

- **Source:** `docs/audit_reports/findings/wave3-auth.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** crates/adapters/auth/src/lib.rs:46

**Description:**

`lib.rs` declares `#![deny(missing_docs)]`
  but the port module (`port.rs:46`) carries an inner
  `#![allow(missing_docs)]`. This creates a deny/allow
  interaction that is hard to audit: future public items
  added to `port.rs` will not be flagged for missing docs,
  while items added elsewhere will. The crate's overall
  documentation contract is weakened by the inner allow.

**Expected:**

`docs/code-standards.md` — "All public APIs
  are documented with rustdoc; `#![deny(missing_docs)]`." The
  inner allow violates this at the crate level.

**Evidence:**

`crates/adapters/auth/src/lib.rs:16`
  (`#![deny(missing_docs)]`) vs
  `crates/adapters/auth/src/port.rs:46`
  (`#![allow(dead_code, clippy::all, missing_docs)]`).

---


## Files (target id prefix: `ADAPTER-FILE`)

**Path:** `crates/adapters/files/`  
**Total findings:** 28 (5 critical, 13 high, 9 medium, 1 low)


### FINDING 1 (id: `ADAPTER-FILE-001`)

- **Source:** `docs/audit_reports/findings/wave3-files.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** crates/adapters/files/src/s3.rs:155-160, 241-306

**Description:**

`S3FileStorage::physical_key` only prepends
  the consumer-supplied `key_prefix` to the logical `FileKey`;
  it does NOT prefix `request.tenant.school_id`. The
  `PutRequest::tenant` field is destructured at line 243 but
  `tenant.school_id` is never used to compute `physical_key`
  (only `tenant.actor_id` is consumed at line 301 for
  `uploaded_by`). The port contract puts the
  `<school_id>/...` prefix on the adapter — the S3 adapter
  delegates it to the consumer's builder configuration, so a
  consumer who forgets to set a per-tenant `key_prefix` writes
  all schools into a flat namespace and a cross-tenant read is
  a one-line mistake.

**Expected:**

`docs/ports/file-storage.md:91-95` — "The
  adapter namespaces keys by tenant. The full key is
  `<school_id>/<domain>/<aggregate>/<id>/<filename>`. The
  adapter is responsible for enforcing this prefix. A consumer
  who stores multiple schools in one bucket cannot
  accidentally cross-tenant access because keys are prefixed."

**Evidence:**

```rust
  fn physical_key(&self, key: &FileKey) -> String {
      let mut buf = String::with_capacity(self.key_prefix.len() + key.as_str().len());
      buf.push_str(&self.key_prefix);
      buf.push_str(key.as_str());
      buf
  }
  ```
  with `tenant.school_id` unused across all 8 trait methods:
  `let PutRequest { tenant, key, ... } = request;`
  `let physical_key = self.physical_key(&key);` at
  `crates/adapters/files/src/s3.rs:241-253`.

---

### FINDING 2 (id: `ADAPTER-FILE-002`)

- **Source:** `docs/audit_reports/findings/wave3-files.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** crates/adapters/files/src/local.rs:148-152, 343-382

**Description:**

`LocalFileStorage` does NOT prefix
  `request.tenant.school_id` onto the resolved filesystem
  path. `LocalFileStorage::put` (line 343) calls
  `self.resolve(request.key.as_str())?` and the `resolve`
  method (line 174) only composes `root / key_prefix / key` —
  neither `tenant` nor `tenant.school_id` appears anywhere on
  the path. The crate's own module-level doc acknowledges this
  at line 145: "Tenant isolation is the caller's responsibility
  — the engine namespaces the key with the `school_id` before
  calling `put`", which is the inverse of what the port
  contract says (the ADAPTER does it).

**Expected:**

`docs/ports/file-storage.md:91-95` — "The
  adapter namespaces keys by tenant. The full key is
  `<school_id>/<domain>/<aggregate>/<id>/<filename>`. The
  adapter is responsible for enforcing this prefix."

**Evidence:**

```rust
  async fn put(&self, request: PutRequest) -> PortResult<FileReference> {
      let path = self.resolve(request.key.as_str())?;
  ```
  at `crates/adapters/files/src/local.rs:343-344`. All other
  trait methods (`get`, `delete`, `exists`, `head`,
  `signed_url`, `copy`, `move_to`) follow the same pattern;
  no method in `crates/adapters/files/src/local.rs` reads
  `request.tenant.school_id`.

---

### FINDING 5 (id: `ADAPTER-FILE-005`)

- **Source:** `docs/audit_reports/findings/wave3-files.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** crates/adapters/files/src/s3.rs:241-306,
  crates/adapters/files/src/local.rs:343-382,
  crates/adapters/files/src/errors.rs:73-76

**Description:**

Neither `S3FileStorage::put` nor
  `LocalFileStorage::put` enforces an upload size limit. The
  port contract documents `FileStorageError::TooLarge(u64,
  u64)` (errors.rs:73-76) for "content to upload exceeds the
  adapter's configured maximum", but neither adapter ever
  constructs it; `S3FileStorage::put` calls `put_object()`
  with whatever `content.len()` the caller provides (mapped
  via `i64::try_from(content_len).unwrap_or(i64::MAX)` at
  line 266), and `LocalFileStorage::put` calls
  `tokio::fs::write(&path, &request.content)` with no length
  check. A consumer can OOM the process or pin the disk by
  uploading a 10 GB object.

**Expected:**

`docs/ports/file-storage.md:75-77` — port
  error variant `TooLarge(u64, u64)` and the engine's port
  contract at `crates/adapters/files/src/port.rs:482-485`:
  "The engine does not impose an upper bound; the adapter
  enforces its own limit and returns
  `FileStorageError::TooLarge` on oversize content."

**Evidence:**

No `TooLarge(` constructor call exists
  anywhere in `crates/adapters/files/src/s3.rs` or
  `crates/adapters/files/src/local.rs` (only in
  `crates/adapters/files/src/errors.rs:260` test). The local
  `put` body at `crates/adapters/files/src/local.rs:355-365`
  writes the content directly without any size gate.

---

### FINDING 8 (id: `ADAPTER-FILE-008`)

- **Source:** `docs/audit_reports/findings/wave3-files.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** crates/adapters/files/src/s3.rs (whole),
  crates/adapters/files/src/local.rs (whole),
  docs/ports/file-storage.md:209-213

**Description:**

Neither `S3FileStorage` nor
  `LocalFileStorage` records an audit event for `put`, `get`,
  `delete`, or `signed_url`. The port contract at
  file-storage.md:209-213 mandates: "Every put, get, delete,
  and signed-URL generation is recorded in the audit log. The
  log includes the key, the actor, and the size. File content
  is never logged." No `audit_log`, `audit_event`,
  `record_audit`, `write_audit`, `tracing::*`, or `log::*`
  call exists in either adapter. The 5 phase-15 handoff
  "Headline numbers" claim "1 net-new `AuditTarget` variant:
  `FileReference`" in `educore-audit`, but no code in the file
  adapter ever emits an event with that variant.

**Expected:**

`docs/ports/file-storage.md:209-213` — "Every
  put, get, delete, and signed-URL generation is recorded in
  the audit log. The log includes the key, the actor, and the
  size. File content is never logged."

**Evidence:**

Searching `crates/adapters/files/src/` for
  `audit_log|put_audit|record_audit|write_audit|tracing::|log::`
  returns zero matches.

---

### FINDING 9 (id: `ADAPTER-FILE-009`)

- **Source:** `docs/audit_reports/findings/wave3-files.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** crates/adapters/files/src/local.rs:214-219,
  473-502

**Description:**

`LocalFileStorage::sign` includes
  `expires_in` (a relative duration, e.g. 60) in the HMAC
  input rather than an absolute `expires_at` timestamp, and
  there is no `Timestamp::now()` comparison anywhere on the
  local signed-URL read path. The module doc at line 53-56
  explicitly states: "The current wall clock is intentionally
  NOT part of the token, so URLs are reproducible across
  processes and replays. A production adapter that needs hard
  expiry should embed an absolute timestamp and verify it at
  fetch time." A URL minted today with `expires_in =
  Duration::from_secs(60)` validates identically in 2030
  because the signature input does not depend on absolute
  time. This is the inverse of the
  `services::SignedUrlService::verify` contract
  (services.rs:187-195), which DOES check `Timestamp::now() <
  expires_at`.

**Expected:**

`docs/ports/file-storage.md:99-101` —
  "`signed_url` produces a time-limited URL for a private
  file." and `crates/adapters/files/src/port.rs:407-413`:
  "The returned URL MUST expire after `expires_in`."

**Evidence:**

```rust
  fn sign(&self, key: &str, expires_in_secs: u64) -> String {
      let message = format!("{key}|{expires_in_secs}");
      let mac = hmac_sha256(&self.signing_secret, message.as_bytes());
      hex_encode(&mac)
  }
  ```
  at `crates/adapters/files/src/local.rs:215-219`. No
  `SystemTime::now()` or `Timestamp::now()` call appears in
  the local signed_url codepath.

---

### FINDING 10 (id: `ADAPTER-FILE-010`)

- **Source:** `docs/audit_reports/findings/wave3-files.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/files/src/local.rs:131-135,
  258-296, 301-324

**Description:**

`LocalFileStorageBuilder::new()` seeds
  `signing_secret` from a well-known compile-time constant
  `DEFAULT_SIGNING_SECRET =
  b"educore-local-file-storage-default-signing-secret-do-not-use-in-prod"`.
  `build()` (line 301) does NOT compare the configured secret
  against this constant; it accepts the default silently with
  no log, panic, or `Result::Err`. A consumer that forgets to
  call `.signing_secret(...)` ships with a publicly-readable
  HMAC key — every signed URL can be minted by anyone who has
  read the crate's source.

**Expected:**

Engine rule: `docs/code-standards.md` § "Type
  Safety" — secrets must not silently fall through to a
  default; `AGENTS.md` § "Code Standards" — "Production-ready.
  Real schools, real students, real money."

**Evidence:**

```rust
  const DEFAULT_SIGNING_SECRET: &[u8] =
      b"educore-local-file-storage-default-signing-secret-do-not-use-in-prod";
  ```
  at `crates/adapters/files/src/local.rs:134-135`. The
  builder's `build()` (line 301-324) does not compare
  `self.signing_secret` against `DEFAULT_SIGNING_SECRET`
  before constructing `LocalFileStorage`.

---

### FINDING 11 (id: `ADAPTER-FILE-011`)

- **Source:** `docs/audit_reports/findings/wave3-files.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/files/src/local.rs:473-502,
  46-47

**Description:**

`LocalFileStorage::signed_url` emits a
  `file://` URL pointing to a local filesystem path. There
  is no fetch endpoint, no middleware that re-validates the
  token, and no client surface that consumes a `file://`
  URL. Anyone with shell access to the host can `cat` the
  file directly, bypassing the HMAC token entirely. The
  signed URL provides no security boundary — it is a stub.

**Expected:**

`docs/ports/file-storage.md:99-101` — "The
  adapter uses the storage provider's signing mechanism (e.g.
  S3 presigned URLs, GCS signed URLs, local token URLs)." An
  `https://` URL backed by a fetch endpoint that verifies the
  token is implied by "signed URL".

**Evidence:**

```rust
  let mut url = format!(
      "file://{}?expires_in={expires_in}&method={method}&token={token}",
      path.display(),
  );
  ```
  at `crates/adapters/files/src/local.rs:489-491`.

---

### FINDING 12 (id: `ADAPTER-FILE-012`)

- **Source:** `docs/audit_reports/findings/wave3-files.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/files/src/s3.rs:448-499

**Description:**

`S3FileStorage::signed_url` does not
  consult `reference.visibility` at all. It will mint a
  presigned URL for any `Visibility::Private` file
  regardless of who is asking. Contrast the local adapter
  (local.rs:481) which short-circuits
  `Visibility::Public && method==Get` to skip the token. The
  S3 implementation also does not accept a per-call
  `actor_id`/`tenant` parameter (the trait signature is
  fixed at port.rs:710-714), so the adapter has no way to
  check whether the caller is authorised for the requested
  URL — it relies entirely on S3's signing model.

**Expected:**

`crates/adapters/files/src/port.rs:708-709`:
  "MUST reject requests on objects whose visibility does not
  permit the requested method."

**Evidence:**

`crates/adapters/files/src/s3.rs:448-499`
  reads `reference` only for `reference.key` (line 453);
  `visibility`, `tenant`, `uploaded_by` are never inspected.

---

### FINDING 13 (id: `ADAPTER-FILE-013`)

- **Source:** `docs/audit_reports/findings/wave3-files.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/files/src/s3.rs:570-578,
  crates/adapters/files/src/port.rs:723-724

**Description:**

`S3FileStorage::move_to` is implemented
  as `self.copy(src, dst_key).await?; self.delete(src).await?;`
  rather than S3's atomic `POST /<bucket>/<dst>?x-id=CopySource`
  followed by `DELETE /<bucket>/<src>` in a single request,
  or S3 Multi-Object Delete. If the copy succeeds and the
  delete fails (network blip, throttling, IAM revocation
  mid-call), the source object persists and the engine has
  returned a `FileReference` for the destination that points
  at content still also living at the source path — orphan
  files plus duplicate storage charges. The port doc
  explicitly says `move_to` "**Atomically** renames the
  object".

**Expected:**

`crates/adapters/files/src/port.rs:723-724` —
  "Atomically renames the object to a new key inside the same
  tenant."

**Evidence:**

```rust
  async fn move_to(
      &self,
      src: &FileReference,
      dst_key: &str,
  ) -> StdResult<FileReference, FileStorageError> {
      let dst = self.copy(src, dst_key).await?;
      self.delete(src).await?;
      Ok(dst)
  }
  ```
  at `crates/adapters/files/src/s3.rs:570-578`.

---

### FINDING 14 (id: `ADAPTER-FILE-014`)

- **Source:** `docs/audit_reports/findings/wave3-files.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/files/src/local.rs:174-212,
  384-421, 423-435

**Description:**

`LocalFileStorage::resolve` performs only
  **lexical** path validation; it never `canonicalize`s the
  destination against `self.root`. The module-level doc at
  lines 193-198 acknowledges the gap: "We can't `canonicalize`
  the full path because the file may not exist yet (for
  `put`); the lexical checks above are the authoritative guard
  against `..` escapes." If an attacker can place a symlink at
  `root/key_prefix/<legit_key>` pointing to `/etc/passwd` or
  `/home/teacher/.ssh/id_rsa`, the `get`/`exists`/`head`/`copy`
  paths will follow the symlink and read or copy arbitrary host
  content. The local adapter is unsuitable for any
  multi-tenant deployment that does not already operate inside
  a hardened namespace.

**Expected:**

Defensive practice; the port contract at
  `docs/ports/file-storage.md:48-50` says "the consumer's
  adapter enforces a safe key namespace", which implies
  filesystem symlinks should not bypass the boundary.

**Evidence:**

No `symlink_metadata`, `canonicalize`, or
  `read_link` call exists anywhere in
  `crates/adapters/files/src/local.rs` (the only matches for
  `canonicalize` at lines 193 and 604 are inside
  doc-comments).

---

### FINDING 15 (id: `ADAPTER-FILE-015`)

- **Source:** `docs/audit_reports/findings/wave3-files.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/files/src/port.rs:634,
  117-124, 408-413

**Description:**

`FileStream` is typed as
  `tokio::sync::mpsc::Receiver<StdResult<Vec<u8>, std::io::Error>>`
  (port.rs:634), so streaming errors surface as
  `std::io::Error` rather than `FileStorageError`. The spec
  types the stream as
  `Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>`
  (`docs/ports/file-storage.md:122-124`) — i.e. the port's own
  `Result<T, FileStorageError>`. A consumer of the trait
  cannot match against `FileStorageError::NotFound` /
  `ChecksumMismatch` on the streaming path because the error
  variant has been flattened to `io::Error`. The local adapter
  surfaces file-open failures as `io::Error` (local.rs:389-397),
  not as `FileStorageError::NotFound` on the stream itself.

**Expected:**

`docs/ports/file-storage.md:122-124` — stream
  items wrapped in the port's `Result` type.

**Evidence:**

```rust
  pub type FileStream = tokio::sync::mpsc::Receiver<StdResult<Vec<u8>, std::io::Error>>;
  ```
  at `crates/adapters/files/src/port.rs:634`.

---

### FINDING 16 (id: `ADAPTER-FILE-016`)

- **Source:** `docs/audit_reports/findings/wave3-files.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/files/src/port.rs:63-64,
  crates/adapters/files/src/lib.rs:41

**Description:**

`port.rs` sets `#![allow(missing_docs)]` at
  module level (line 64), which silently overrides the
  crate-root `#![deny(missing_docs)]` declared in `lib.rs:41`.
  The `port.rs` module is the largest public surface in the
  crate (the `FileStorage` trait, `PutRequest`, `FileReference`,
  `FileMetadata`, `Visibility`, `StorageClass`,
  `SignedUrlMethod`, `SignedUrlOptions`, `FileKey`,
  `ContentType`, `Checksum`, `IdempotencyKey`, `FileStream`);
  the engine rule requires every public item to carry rustdoc.
  With the allow, a future contributor can remove a
  doc-comment from any of these types without the deny
  firing.

**Expected:**

`AGENTS.md` § "Code Standards" — "All public
  APIs are documented with rustdoc; `#![deny(missing_docs)]`."

**Evidence:**

```rust
  #![allow(dead_code, clippy::all)]
  #![allow(missing_docs)]
  ```
  at `crates/adapters/files/src/port.rs:63-64`, preceded by
  the crate-root `#![deny(missing_docs)]` at
  `crates/adapters/files/src/lib.rs:41`.

---

### FINDING 17 (id: `ADAPTER-FILE-017`)

- **Source:** `docs/audit_reports/findings/wave3-files.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/files/tests/files_integration.rs,
  docs/ports/file-storage.md:193-202

**Description:**

The integration test file
  (`crates/adapters/files/tests/files_integration.rs`) ships
  5 sync tests + 2 env-gated tests. The env-gated tests
  (`files_integration_async_s3_put_mock` at line 159 and
  `files_integration_async_local_put_mock` at line 168) call
  `.build()` on the respective builders and discard the
  result — they do not exercise `put`, `get`, `delete`,
  `exists`, `head`, `signed_url`, `copy`, or `move_to`
  against any real or fake backend. The port contract lists
  7 categories of required integration tests; only 5
  (SHA-256, ETag, key namespace round-trip, visibility
  classification, signed URL build+verify) are present,
  missing:
  - "Integration tests of signed URL generation and
    **expiration**" (no expiry assertion exists)
  - "A test of **cross-tenant denial**" (no actor/school
    check)
  - "A test of **checksum mismatch**" (no read-side
    recompute)
  - "A test of **content type validation**" (no allow-list
    test)
  - "A test of **large file streaming**" (no multi-MB
    stream)
  - "A test of **idempotent retry**" (no second-put
    assertion)

**Expected:**

`docs/ports/file-storage.md:193-202` — the 7
  test categories above.

**Evidence:**

`crates/adapters/files/tests/files_integration.rs:157-172`
  contains only builder-construction bodies (e.g. line 160-163
  builds an `S3FileStorage` and binds it to `_storage`, never
  calling `put`).

---

### FINDING 23 (id: `ADAPTER-FILE-023`)

- **Source:** `docs/audit_reports/findings/wave3-files.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/files/src/s3.rs:308-359,
  crates/adapters/files/src/local.rs:384-421,
  docs/ports/file-storage.md:286-311

**Description:**

Neither `S3FileStorage::get` nor
  `LocalFileStorage::get` enforces `reference.visibility`. A
  `Visibility::Private` file uploaded at school A is fetchable
  by anyone who holds a valid `FileReference` for it,
  regardless of the requesting user's role or school. The
  port contract at file-storage.md:42-47 defines
  `Visibility::Private` / `Public` / `TenantPrivate`, and
  the port trait doc at port.rs:286-294 requires "a file
  uploaded as `Visibility::Private` must require a signed URL
  on every read." The `Visibility::Private` variant in
  `FileStorageError::PermissionDenied` (errors.rs:62-65) is
  documented for "cross-tenant attempts, expired signed URLs,
  and unauthorised `Visibility::Private` reads" but is never
  constructed anywhere in either adapter.

**Expected:**

`crates/adapters/files/src/port.rs:708-709`:
  "MUST reject requests on objects whose visibility does not
  permit the requested method."

**Evidence:**

No `PermissionDenied` constructor call exists
  anywhere in `crates/adapters/files/src/s3.rs` or
  `crates/adapters/files/src/local.rs` (only in
  `crates/adapters/files/src/errors.rs:252` test).

---

### FINDING 3 (id: `ADAPTER-FILE-003`)

- **Source:** `docs/audit_reports/findings/wave3-files.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/files/src/s3.rs:241-306, 26-30

**Description:**

`S3FileStorage::put` does not implement
  idempotency on `PutRequest::idempotency_key`. The field is
  destructured into `_idempotency_key` at line 250 (discarded)
  and the module-level docstring at line 26 explicitly
  acknowledges "`S3FileStorage::put` is **not** idempotent on
  `PutRequest::idempotency_key` at the S3 layer; the adapter
  documents the key on the returned `FileReference` but does
  not consult it before upload." A retry of the same upload
  with the same key will create N S3 objects and N storage
  charges.

**Expected:**

`docs/ports/file-storage.md:80-82` —
  "`idempotency_key` is used by the adapter to deduplicate
  retry uploads. A retry of the same upload returns the same
  `FileReference` without re-uploading."

**Evidence:**

`idempotency_key: _idempotency_key,` at
  `crates/adapters/files/src/s3.rs:250`.

---

### FINDING 4 (id: `ADAPTER-FILE-004`)

- **Source:** `docs/audit_reports/findings/wave3-files.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/files/src/local.rs:69-73, 343-382

**Description:**

`LocalFileStorage::put` does not implement
  idempotency on `PutRequest::idempotency_key`. The module-
  level docstring at line 69 explicitly documents "**No
  idempotency cache.** `PutRequest::idempotency_key` is
  accepted on the wire but not deduplicated — a retry
  re-uploads." The field is set to `None` on the test fixture
  and is never read on the put path.

**Expected:**

`docs/ports/file-storage.md:80-82` — see
  Finding 3.

**Evidence:**

`crates/adapters/files/src/local.rs:69-73`:
  ```text
  //! 4. **No idempotency cache.** `PutRequest::idempotency_key`
  //!    is accepted on the wire but not deduplicated — a retry
  //!    re-uploads. ...
  ```

---

### FINDING 6 (id: `ADAPTER-FILE-006`)

- **Source:** `docs/audit_reports/findings/wave3-files.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/files/src/s3.rs:241-306,
  crates/adapters/files/src/local.rs:343-382,
  crates/adapters/files/src/errors.rs:78-81

**Description:**

Neither adapter validates
  `PutRequest::content_type` against any allow-list. The port
  documents `FileStorageError::UnsupportedContentType(ContentType)`
  (errors.rs:78-81) for "The adapter does not accept the
  supplied MIME type", but no construction site exists;
  `S3FileStorage::put` forwards the caller-supplied MIME
  string verbatim to S3 (`s3.rs:265`:
  `.content_type(content_type.as_str())`), and the local
  adapter accepts and stores it as-is. A consumer can upload
  `application/x-msdownload` (Windows EXE), `text/html` with
  embedded `<script>` XSS payloads, or any other arbitrary
  MIME and the adapter will not raise an error.

**Expected:**

`docs/ports/file-storage.md:148-150` and
  `crates/adapters/files/src/port.rs:488-491`: "The adapter
  may reject unknown or disallowed types with
  `FileStorageError::UnsupportedContentType`."

**Evidence:**

No `UnsupportedContentType(` constructor call
  exists anywhere in `crates/adapters/files/src/s3.rs` or
  `crates/adapters/files/src/local.rs` (only in
  `crates/adapters/files/src/errors.rs:265` test). The S3 put
  forwards verbatim at `crates/adapters/files/src/s3.rs:265`.

---

### FINDING 7 (id: `ADAPTER-FILE-007`)

- **Source:** `docs/audit_reports/findings/wave3-files.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/files/src/s3.rs:33-39, 308-359,
  crates/adapters/files/src/local.rs:384-421,
  crates/adapters/files/src/port.rs:681-688

**Description:**

Neither `get` implementation re-verifies
  the SHA-256 checksum of the streamed bytes against
  `reference.checksum`. The port contract at port.rs:681-688
  says "The adapter MUST verify the content hash against
  `reference.checksum` and MUST surface a
  `FileStorageError::ChecksumMismatch` on a mismatch." The S3
  module doc at line 37 acknowledges the gap: "Reads do not
  currently re-verify the checksum on the streamed bytes;
  consumers that require wire-level integrity verification
  must layer it on top of the `FileStream`." The local
  adapter's `get` (line 384) opens the file and pushes raw
  4 KB chunks through the channel without computing any hash.
  The `ChecksumMismatch` variant (errors.rs:67-71) is never
  constructed.

**Expected:**

`crates/adapters/files/src/port.rs:681-688` and
  `docs/ports/file-storage.md:84-87` — "The adapter computes a
  SHA-256 checksum on upload. The engine verifies the checksum
  on read. Mismatches fail the read."

**Evidence:**

`crates/adapters/files/src/s3.rs:33-39`
  (module doc acknowledging the gap), and no
  `ChecksumMismatch` constructor call in any non-test source.

---

### FINDING 18 (id: `ADAPTER-FILE-018`)

- **Source:** `docs/audit_reports/findings/wave3-files.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/files/Cargo.toml:8,
  crates/adapters/files/src/lib.rs:3

**Description:**

`Cargo.toml` description claims "File
  storage port, **S3-compatible, GCS**, local filesystem
  adapters" and the lib.rs docstring mirrors it: "File
  storage port, S3-compatible, **GCS**, local filesystem
  adapters." No GCS module, no GCS builder, no GCS client
  dep (`grep -rn 'gcp\|google.cloud\|gcs'` on
  `crates/adapters/files/` returns zero source hits; only
  doc references to GCS as a "future" alternative). The
  crate description advertises a feature that does not exist;
  downstream consumers reading the manifest will believe GCS
  is supported.

**Expected:**

Accurate crate metadata; engine rule
  `AGENTS.md` § "Naming Convention" requires exact advertised
  surface area.

**Evidence:**

`crates/adapters/files/Cargo.toml:8` —
  `description = "File storage port, S3-compatible, GCS, local
  filesystem adapters."` Only `pub mod s3;` (s3.rs) and
  `pub mod local;` (local.rs) exist.

---

### FINDING 19 (id: `ADAPTER-FILE-019`)

- **Source:** `docs/audit_reports/findings/wave3-files.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/files/src/local.rs:78-81,
  343-382

**Description:**

`LocalFileStorage::put` ignores
  `PutRequest::overwrite` and always writes through. The
  module doc at lines 78-81 acknowledges the gap:
  "**`overwrite = false` is not enforced.** The local adapter
  always overwrites; the spec leaves the precise error to the
  adapter and a real S3 adapter would surface a
  `PreconditionFailed`." A consumer that uploads a student
  photo with `overwrite = false` to
  `students/photos/ada.jpg` will silently replace an existing
  photo, breaking the "preserve old version" contract implied
  by the `PutRequest::overwrite` field.

**Expected:**

`crates/adapters/files/src/port.rs:501-507` —
  "`true` to overwrite an existing object at the same key;
  `false` to return an error if the key is already in use."

**Evidence:**

`crates/adapters/files/src/local.rs:343-365`
  calls `tokio::fs::write(&path, &request.content)` without
  inspecting `request.overwrite` or calling `exists` first.

---

### FINDING 20 (id: `ADAPTER-FILE-020`)

- **Source:** `docs/audit_reports/findings/wave3-files.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/files/src/s3.rs:297

**Description:**

`S3FileStorage::put` performs
  `content_len as u64` to coerce `usize` into
  `FileReference.size` (`u64`). On a 32-bit platform where
  `usize == u32`, this is an `as`-cast that the engine rule
  forbids (lossy on byte counts above `u32::MAX` only in
  theory, but the engine code standard forbids ALL `as` on
  numerics — `TryFrom`/`TryInto` is required). The S3
  `content_length` setter already does the correct conversion
  on line 266 via
  `i64::try_from(content_len).unwrap_or(i64::MAX)` — the
  returned-reference field should match.

**Expected:**

`AGENTS.md` § "Type Safety" — "No `as` casts
  that truncate or lose data. Use `TryFrom` / `TryInto` with
  proper error handling."

**Evidence:**

`size: content_len as u64,` at
  `crates/adapters/files/src/s3.rs:297`.

---

### FINDING 21 (id: `ADAPTER-FILE-021`)

- **Source:** `docs/audit_reports/findings/wave3-files.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/files/src/port.rs:710-714,
  docs/ports/file-storage.md:103-115

**Description:**

`FileStorage::signed_url` returns
  `Result<String>` (port.rs:714) instead of
  `Result<url::Url>`. The port contract types the return as
  `Result<Url>` (`docs/ports/file-storage.md:20`). Adapters
  must hand-roll URL string assembly (local.rs:489-501;
  s3.rs:498), and downstream consumers cannot use the
  standard `Url` API (`url::Url::parse`) to parse or join
  against the returned URL. The crate's deviation note at
  port.rs:33-37 acknowledges this but the port surface itself
  was the spec target.

**Expected:**

`docs/ports/file-storage.md:20` — `async fn
  signed_url(...) -> Result<Url>`.

**Evidence:**

```rust
  async fn signed_url(
      &self,
      reference: &FileReference,
      options: SignedUrlOptions,
  ) -> Result<String>;
  ```
  at `crates/adapters/files/src/port.rs:710-714`.

---

### FINDING 22 (id: `ADAPTER-FILE-022`)

- **Source:** `docs/audit_reports/findings/wave3-files.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/files/src/local.rs:174-212,
  343-382, docs/ports/file-storage.md:126-129

**Description:**

`LocalFileStorage::resolve` does not
  reject empty keys or keys containing null bytes; both
  surface as filesystem errors (`NotADirectory`,
  `InvalidInput`) wrapped in
  `FileStorageError::Infrastructure`, not the more specific
  `FileStorageError::InvalidKey` that the spec reserves for
  malformed inputs. S3 has no such guard either — empty keys
  become empty object names, null bytes are forwarded verbatim
  to AWS (which rejects them with a 400 wrapped as
  `Infrastructure`). The port contract's `InvalidKey(String)`
  variant implies a key-validation step neither adapter
  performs.

**Expected:**

`docs/ports/file-storage.md:151` and
  `crates/adapters/files/src/port.rs:152-158` — "adapters
  that need validation (length, character set, reserved
  prefix) perform it inside the `FileStorage::put`
  implementation and return `FileStorageError::InvalidKey` on
  a malformed input."

**Evidence:**

No length / null-byte / character-set check
  exists in `crates/adapters/files/src/local.rs:174-212` or
  `crates/adapters/files/src/s3.rs:241-306`.

---

### FINDING 24 (id: `ADAPTER-FILE-024`)

- **Source:** `docs/audit_reports/findings/wave3-files.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/files/src/s3.rs:448-499,
  crates/adapters/files/src/port.rs:710-714

**Description:**

`FileStorage::signed_url` has no
  per-caller / per-actor parameter; the trait signature is
  `(&self, reference, options) -> Result<String>`. The S3
  adapter therefore has no way to enforce that the actor
  requesting the URL is a member of
  `reference.tenant.school_id`, has the `FilesSignedUrl`
  capability, or even that the requester is different from
  the uploader. Any code that holds a `FileReference`
  (e.g. a student viewing their own report card) can mint
  an admin-grade PUT presigned URL and overwrite the source
  object. Visibility is the only gate and Finding 23
  documents that it is not enforced on reads either.

**Expected:**

Per the engine's RBAC contract
  (`AGENTS.md` § "Multi-tenant by default. Every aggregate
  has a `SchoolId`") and the port's
  `Capability::FilesSignedUrl` capability
  (PHASE-15-HANDOFF.md:191-193), a `signed_url` call should
  accept or be paired with an actor context that the adapter
  can authorise against.

**Evidence:**

`crates/adapters/files/src/port.rs:710-714` —
  the trait signature carries no `actor` or `tenant_context`
  parameter, and `s3.rs:448-499` only reads `reference.key`
  from the supplied `reference`.

---

### FINDING 25 (id: `ADAPTER-FILE-025`)

- **Source:** `docs/audit_reports/findings/wave3-files.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/files/src/s3.rs:84-99, 308-359

**Description:**

`S3FileStorage::get` issues no `If-Match`
  or `If-None-Match` precondition against `reference.etag`.
  If a racing upload overwrites the object between the
  consumer's fetch and the consumer's checksum check (which
  Finding 7 documents is not done by the adapter), the
  engine will consume bytes that do not match the
  `FileReference` it holds. Combined with Finding 7 (no
  read-side checksum recompute), the S3 adapter cannot
  detect an in-flight overwrite at all.

**Expected:**

RFC 7232 / S3 `If-Match` precondition; an
  upload path that captures `reference.etag` should be
  paired with a download path that requires it.

**Evidence:**

`crates/adapters/files/src/s3.rs:312-319`
  builds the `get_object` request with only `.bucket(...)`
  and `.key(...)`; no `.if_match(...)` or equivalent call
  exists.

---

### FINDING 26 (id: `ADAPTER-FILE-026`)

- **Source:** `docs/audit_reports/findings/wave3-files.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/files/src/s3.rs:501-568,
  docs/ports/file-storage.md:126-129

**Description:**

S3 versioning is not enabled and not
  exercised in `S3FileStorage`. The port contract at
  `docs/ports/file-storage.md:126-129` requires: "If the
  underlying provider supports versioning (S3 does), the
  adapter enables it. Older versions are retained for a
  configurable period." `S3FileStorage::copy`
  (s3.rs:501-568) overwrites the destination with
  `copy_object` semantics, not a versioned-copy, and
  `S3FileStorage::put` (line 270-276) uses
  `If-None-Match: *` rather than enabling bucket versioning.
  The `VersioningConfiguration` S3 API is never invoked.

**Expected:**

`docs/ports/file-storage.md:126-129` — "If
  the underlying provider supports versioning (S3 does), the
  adapter enables it."

**Evidence:**

No `versioning`, `VersioningConfiguration`,
  `enable_versioning`, or `version_id` reference exists in
  `crates/adapters/files/src/s3.rs`.

---

### FINDING 27 (id: `ADAPTER-FILE-027`)

- **Source:** `docs/audit_reports/findings/wave3-files.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/files/src/local.rs:400-418,
  crates/adapters/files/src/s3.rs:330-356

**Description:**

Both `get` implementations spawn a
  `tokio::spawn(async move { ... })` to drain the upstream
  into the mpsc channel, but neither captures the
  `JoinHandle` nor surfaces a panic from the spawned task.
  If the task panics (e.g. a `tokio::fs::File` invariant
  violation, an S3 SDK mid-stream decode error that the
  worker surfaces as a panic), the channel closes silently
  with `None` and the engine sees a truncated file as a
  clean EOF. There is no way for the engine's audit or
  observability layer to distinguish "successful end of
  stream" from "spawned task panicked".

**Expected:**

Engine rule `AGENTS.md` § "Type Safety" —
  "No `unwrap`/`expect`/`panic` in production paths", and
  the port contract at
  `crates/adapters/files/src/port.rs:614-621` ("Adapters MUST
  yield chunks promptly and MUST NOT buffer the entire object
  before sending the first chunk") — both presume a sound
  task lifetime.

**Evidence:**

`crates/adapters/files/src/local.rs:401-418`
  spawns the task without storing the handle, and
  `crates/adapters/files/src/s3.rs:330-356` does the same.

---

### FINDING 28 (id: `ADAPTER-FILE-028`)

- **Source:** `docs/audit_reports/findings/wave3-files.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** crates/adapters/files/src/local.rs:301-324,
  crates/adapters/files/src/local.rs:258-296

**Description:**

`LocalFileStorageBuilder::build()` does
  not validate that `key_prefix` is a safe lexical string. A
  consumer that sets `.key_prefix("/etc/")` or
  `.key_prefix("../escape/")` causes
  `LocalFileStorage::resolve` to compose `root + "/etc/" +
  key` or `root + "../escape/" + key`; the post-normalisation
  prefix check at line 203-209 rejects the second case
  (because the normalised path does not start with the
  normalised root) but accepts the first case (because
  `/etc/` is a child of root if root is `/`). The adapter
  offers no documented whitelist for `key_prefix` and no
  `must_use` warning.

**Expected:**

Defensive builder validation; the port
  contract at `crates/adapters/files/src/port.rs:152-158`
  ("adapters that need validation (length, character set,
  reserved prefix) perform it inside the `FileStorage::put`
  implementation") implies the builder's `key_prefix` setter
  should reject obviously-unsafe values.

**Evidence:**

`crates/adapters/files/src/local.rs:301-324`
  (`build`) does not call `validate` on `self.key_prefix`;
  the only path-safety check happens per-call inside
  `resolve` (line 174-212) and only on the **key**, not on
  the `key_prefix` itself.

### END FINDINGS

---


## Integrations (target id prefix: `ADAPTER-INT`)

**Path:** `crates/adapters/integrations/`  
**Total findings:** 42 (5 critical, 10 high, 20 medium, 7 low)


### FINDING 1 (id: `ADAPTER-INT-001`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/lms.rs:186-237`, `crates/adapters/integrations/src/video.rs:191-242`, `crates/adapters/integrations/src/webhook_out.rs:242-341`

**Description:**

None of the three reference implementations
  (`LmsIntegration`, `VideoConferencingIntegration`,
  `WebhookOutIntegration`) read or use `IntegrationRequest::tenant`.
  The `invoke` bodies dispatch on `request.action.as_str()` and
  use the builder-supplied `api_key` / `secret` directly without
  consulting `tenant.school_id`. The port contract requires
  per-tenant configuration lookup.

**Expected:**

`docs/ports/integrations.md:177-180` — "The
  `IntegrationConfig` value is loaded from the platform domain
  at startup. The engine passes `TenantContext` to the adapter;
  the adapter uses it to look up the config."

**Evidence:**

`crates/adapters/integrations/src/lms.rs:187`
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

### FINDING 2 (id: `ADAPTER-INT-002`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/lms.rs:186-220`, `crates/adapters/integrations/src/video.rs:191-225`, `crates/adapters/integrations/src/webhook_out.rs:242-321`

**Description:**

Zero audit-log calls exist in any of the three
  implementations. The `educore-audit` crate declares
  `AuditTarget::IntegrationConfig` and `AuditTarget::IntegrationInvocation`
  variants (added in Phase 15), but no code in
  `crates/adapters/integrations/` writes to the audit log. The
  port contract requires every invocation be recorded.

**Expected:**

`docs/ports/integrations.md:195-200` ("Audit
  Logging") — "Every integration invocation is logged with tenant,
  integration, action, status, duration, and cost. Input and output
  are logged at DEBUG and may be redacted by the adapter." And
  `docs/ports/integrations.md:263-266` — "Every invocation, success
  or failure, is recorded with full metadata. Sensitive fields are
  redacted by the adapter."

**Evidence:**

`grep -r 'audit\|AuditTarget\|record_integration\|IntegrationInvocation' crates/adapters/integrations/src/`
  returns no production matches outside doc comments. The
  `IntegrationInvocation` variant exists in
  `crates/cross-cutting/audit/src/writer.rs:442` but the
  integrations crate never imports `educore-audit` (verified by
  reading `crates/adapters/integrations/Cargo.toml:13-27`).

---

### FINDING 3 (id: `ADAPTER-INT-003`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/video.rs:255-272`, `crates/adapters/integrations/src/video.rs:280-303`, `crates/adapters/integrations/src/video.rs:309-323`

**Description:**

`VideoConferencingIntegration::auth_header()`
  returns the `api_secret` and forwards it in an `X-Api-Secret`
  HTTP header on every outbound request to Zoom, Google Meet,
  and Microsoft Teams. This sends the raw signing secret in the
  clear over the network on every API call; if TLS is terminated
  upstream (e.g. corporate proxy), the secret leaks to logs.
  The port contract requires Zoom JWT signing, not header forwarding.

**Expected:**

`docs/ports/integrations.md:166-174` ("OAuth2
  Client Credentials") — "The adapter: 1. Stores the client_id
  and client_secret (per tenant). 2. Performs the OAuth2 token
  exchange. 3. Caches the token until expiry. 4. Refreshes the
  token before expiry." And the
  `crates/adapters/integrations/src/video.rs:14-22` doc comment
  itself states "Zoom JWT auth (the simplest Zoom integration):
  `api_key` is the Zoom API key; `api_secret` is used to sign a
  JWT with `HS256`."

**Evidence:**

`crates/adapters/integrations/src/video.rs:255-257`
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

### FINDING 4 (id: `ADAPTER-INT-004`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/lms.rs:144-155`, `crates/adapters/integrations/src/video.rs:147-158`

**Description:**

`LmsIntegrationBuilder::build()` and
  `VideoConferencingIntegrationBuilder::build()` default
  `api_key` (and `api_secret`) to empty strings via
  `unwrap_or_default()`. A consumer that forgets to call
  `.api_key(...)` builds a working client that authenticates with
  a blank bearer token to a live provider. There is no
  validation, no `Result` return, and no warning.

**Expected:**

`docs/code-standards.md` § "Type Safety" — "No
  `unwrap()` or `expect()` in production paths. Propagate errors
  via `?` or document the invariant that makes panic impossible."
  Per AGENTS.md, `IntegrationConfig` per-tenant credentials must
  be loaded at startup (port contract § "Per-Tenant Configuration").

**Evidence:**

`crates/adapters/integrations/src/lms.rs:150`
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

### FINDING 5 (id: `ADAPTER-INT-005`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/lms.rs:273-285`, `crates/adapters/integrations/src/lms.rs:338-345`, `crates/adapters/integrations/src/video.rs:265-277`, `crates/adapters/integrations/src/webhook_out.rs:213-230`

**Description:**

None of the three reference impls invoke
  `RetryService::next_backoff`, `RetryService::should_retry`, or
  `RetryService::is_permanent_failure`. Every outbound HTTP call
  is a single shot — the adapter never re-issues a request on a
  transient failure (5xx, network, 408, 429). The
  `RetryService` exists in `services.rs:236` and has 11 unit
  tests, but is not wired into any impl. The port contract
  mandates retry orchestration.

**Expected:**

`docs/ports/integrations.md:182-193` — "The
  adapter retries transient failures (5xx, network) per the
  policy. Permanent failures (4xx) are returned immediately."

**Evidence:**

`crates/adapters/integrations/src/lms.rs:273-284`
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

### FINDING 10 (id: `ADAPTER-INT-010`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/port.rs:283-308`

**Description:**

Doc-vs-code drift: the port contract at
  `docs/ports/integrations.md:48-55` defines
  `IntegrationResponse.cost: Option<Money>` (a single shared type
  in `educore-core::value_objects`). The actual code at
  `crates/adapters/integrations/src/port.rs:283-308` defines
  `IntegrationResponse.cost: Option<IntegrationCost>` where
  `IntegrationCost` is a fresh local struct (`amount_minor: i64,
  currency: String`). Consumers following the spec get a type
  error.

**Expected:**

`docs/ports/integrations.md:48-55` — `pub cost:
  Option<Money>`.

**Evidence:**

`crates/adapters/integrations/src/port.rs:302`
  ```rust
  pub cost: Option<IntegrationCost>,
  ```
  with `IntegrationCost` defined at `:317-325`. No `Money` import
  exists in the port module (verified by reading
  `crates/adapters/integrations/src/port.rs:27-40`).

---

### FINDING 11 (id: `ADAPTER-INT-011`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/port.rs:264`, `crates/adapters/integrations/src/port.rs:297`, `crates/adapters/integrations/src/port.rs:194-209`

**Description:**

Doc-vs-code drift: the port contract types
  `IntegrationRequest::timeout: Option<Duration>`,
  `IntegrationResponse::duration: Duration`, and `RetryPolicy`
  `interval`/`base`/`max` fields all as `Duration` (std). The
  code types them as `chrono::Duration` (`ChronoDuration`). All
  three impls compute durations via
  `chrono::Duration::from_std(...)` and silently zero out
  negative or overflowing std durations.

**Expected:**

`docs/ports/integrations.md:38` — `pub timeout:
  Option<Duration>`. `docs/ports/integrations.md:53` — `pub
  duration: Duration`. `docs/ports/integrations.md:184-190` —
  `Linear { max_retries: u32, interval: Duration }`,
  `Exponential { max_retries: u32, base: Duration, max: Duration }`.

**Evidence:**

`crates/adapters/integrations/src/port.rs:31`
  ```rust
  use chrono::Duration as ChronoDuration;
  ```
  used at `:196`, `:206`, `:208`, `:264`, `:297`. The
  integration tests at
  `crates/adapters/integrations/tests/integrations_integration.rs:42-44`
  call this out in a comment ("`RetryPolicy::Exponential.base` and
  `.max` are `chrono::Duration` (not `std::time::Duration`)").

---

### FINDING 12 (id: `ADAPTER-INT-012`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/webhook_out.rs:202-209`, `crates/adapters/integrations/src/services.rs:146-152`

**Description:**

The HMAC-SHA256 signing helper is implemented
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

**Expected:**

AGENTS.md § "Module Layout" — single source of
  truth per operation.

**Evidence:**

`crates/adapters/integrations/src/webhook_out.rs:202-209`
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

### FINDING 13 (id: `ADAPTER-INT-013`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/webhook_out.rs:202-209`, `crates/adapters/integrations/src/webhook_out.rs:383-393`

**Description:**

`WebhookOutIntegration::compute_signature` and
  `WebhookOutIntegrationBuilder::build` both call
  `.expect(...)` in production paths. The crate's lib.rs denies
  `expect_used` (workspace lint, line 269 of root Cargo.toml) but
  these production sites use `#[allow(clippy::expect_used)]` to
  bypass it. AGENTS.md forbids `expect()` in production code.

**Expected:**

`AGENTS.md` § "Type Safety" — "No `unwrap()` or
  `expect()` in production paths. Propagate errors via `?` or
  document the invariant that makes panic impossible." And
  `crates/adapters/integrations/src/services.rs:146-148` already
  shows the correct error-mapping pattern.

**Evidence:**

`crates/adapters/integrations/src/webhook_out.rs:202`
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

### FINDING 14 (id: `ADAPTER-INT-014`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/port.rs:65-75`, `crates/adapters/integrations/src/port.rs:108-120`

**Description:**

`IntegrationId::new` and `IntegrationAction::new`
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

**Expected:**

`docs/ports/integrations.md:41-43` — "IntegrationId
  is a typed enum or string identifier for the integration."
  The reference to "typed enum" implies a closed-set validator;
  the impl defers validation entirely.

**Evidence:**

`crates/adapters/integrations/src/port.rs:66-68`
  ```rust
  pub fn new(s: impl Into<String>) -> Self {
      Self(s.into())
  }
  ```
  `crates/adapters/integrations/src/lms.rs:120` —
  `.provider(provider: impl Into<String>)`. No normalization or
  validation occurs anywhere in the impls.

---

### FINDING 15 (id: `ADAPTER-INT-015`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/lms.rs:230-236`, `crates/adapters/integrations/src/video.rs:235-241`, `crates/adapters/integrations/src/webhook_out.rs:334-340`

**Description:**

`health()` on all three implementations is
  faked. `LmsIntegration::health()` returns
  `HealthStatus::Healthy` with `Timestamp::now()`; the Video
  adapter returns `Healthy` with `Timestamp::epoch()`; the
  WebhookOut returns `Healthy` with `Timestamp::now()`. None of
  the three performs an actual liveness probe of the upstream
  provider. A provider outage is undetectable until the next
  actual call fails — defeating the operational dashboards the
  port contract relies on.

**Expected:**

`docs/ports/integrations.md:21-22` — `health()`
  returns `IntegrationHealth`. `docs/ports/integrations.md:489-491`
  (port.rs doc) — "Report liveness of the gateway and every
  registered integration. Called by the engine's operational
  dashboards every 30 s."

**Evidence:**

`crates/adapters/integrations/src/lms.rs:230-236`
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

### FINDING 6 (id: `ADAPTER-INT-006`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/lms.rs:146`, `crates/adapters/integrations/src/video.rs:149`

**Description:**

The `LmsIntegration` and
  `VideoConferencingIntegration` construct their HTTP client via
  `Client::new()`. With the workspace `reqwest` declaration
  (`default-features = false`, `features = ["rustls-tls", "json",
  "stream"]`), `Client::new()` does not set a request timeout.
  Every outbound call to Google Classroom / Zoom / Teams can hang
  indefinitely, blocking the engine's executor. Only
  `WebhookOutIntegrationBuilder::build()` (line 385-388) calls
  `Client::builder().timeout(...)`.

**Expected:**

`docs/ports/integrations.md:222-237` ("Worked
  Example") — `IntegrationRequest::timeout` is documented as the
  per-call override; an adapter default must exist or the contract
  is meaningless. `docs/code-standards.md` § "Production-ready"
  ("Real schools, real students, real money").

**Evidence:**

`crates/adapters/integrations/src/lms.rs:146`
  ```rust
  http: Client::new(),
  ```
  `crates/adapters/integrations/src/video.rs:149` identical.
  Compare with `crates/adapters/integrations/src/webhook_out.rs:385-388`
  which sets a 30 s timeout.

---

### FINDING 7 (id: `ADAPTER-INT-007`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/lms.rs:187-220`, `crates/adapters/integrations/src/video.rs:192-225`

**Description:**

`IntegrationRequest::timeout` is read from the
  port surface (`port.rs:264`) but never applied by the LMS or
  Video adapters. The WebhookOut adapter hard-codes a 30 s timeout
  in `HTTP_TIMEOUT_SECS` (`webhook_out.rs:101`) and ignores the
  per-call override. Per-call timeout is dead code in the engine.

**Expected:**

`docs/ports/integrations.md:38` (`IntegrationRequest`)
  — `pub timeout: Option<Duration>` — "Optional per-call timeout
  override. `None` means 'use the adapter default'."

**Evidence:**

`crates/adapters/integrations/src/lms.rs:187-220`:
  `request.timeout` is not referenced in `invoke`. The same is
  true in `crates/adapters/integrations/src/video.rs:191-225`.
  `grep 'request\.timeout' crates/adapters/integrations/src/` returns
  only matches in `errors.rs` (the variant doc).

---

### FINDING 8 (id: `ADAPTER-INT-008`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/lms.rs:505-523`, `crates/adapters/integrations/src/video.rs:397-415`

**Description:**

The shared `parse_response` helper maps a
  non-2xx response into `IntegrationError::Provider(format!("{}
  {}", status.as_u16(), body))`. The full response body — which
  for LMS roster sync, course create, and video meeting get/list
  contains student identifiers, names, emails, and meeting join
  URLs — is embedded in the error message. The error is then
  surfaced in `IntegrationResponse::error`, which the port spec
  says is "logged at DEBUG". Every PII field in the body is in
  the error string.

**Expected:**

`docs/ports/integrations.md:195-200` ("Audit
  Logging") — "Input and output are logged at DEBUG and may be
  redacted by the adapter." And
  `docs/ports/integrations.md:263-266` — "Sensitive fields are
  redacted by the adapter."

**Evidence:**

`crates/adapters/integrations/src/lms.rs:516-521`
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

### FINDING 9 (id: `ADAPTER-INT-009`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/lms.rs:363-369`

**Description:**

`LmsIntegration::sync_roster` builds the
  per-student error JSON as `{"user_id": user_id, "action": action,
  "error": err.to_string()}`. For a network/DNS error,
  `err.to_string()` includes the full target URL with course id
  and user id embedded, plus the reqwest internal context. This
  is the engine's per-student error payload — it propagates into
  the LMS rosters' error report and is the basis of any retry
  decision the LMS admin makes.

**Expected:**

`docs/ports/integrations.md:195-200` — redaction
  requirement on adapter-emitted error text.

**Evidence:**

`crates/adapters/integrations/src/lms.rs:363-369`
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

### FINDING 16 (id: `ADAPTER-INT-016`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/lms.rs:222-228`, `crates/adapters/integrations/src/video.rs:227-233`

**Description:**

The `LmsIntegration::list_capabilities` and
  `VideoConferencingIntegration::list_capabilities` hard-code
  three capability rows. There is no way to add, remove, or
  override capabilities at runtime. The WebhookOut integration
  returns one row. The port contract says UIs and AI-agent tool
  catalogs depend on this method to render dynamic forms;
  shipping a static list means consumer UIs cannot expose a
  capability that the adapter does not yet know about.

**Expected:**

`docs/ports/integrations.md:69-80` —
  "IntegrationCapability... The engine can enumerate capabilities
  at runtime for UIs and AI agent tool catalogs."

**Evidence:**

`crates/adapters/integrations/src/lms.rs:222-228`
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

### FINDING 17 (id: `ADAPTER-INT-017`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/lms.rs:449-487`

**Description:**

All three LMS capabilities (`lms.course.create`,
  `lms.roster.sync`, `lms.submissions.pull`) list
  `vec![Capability::LmsRosterSync]` as their required capability.
  Creating a course and pulling submissions are conceptually
  distinct operations from syncing rosters and should map to
  distinct capabilities. The current scheme means an RBAC role
  permitted only `LmsRosterSync` can also create courses and
  pull submissions, while a role that should be able to pull
  submissions but not sync rosters has no way to express that.

**Expected:**

`docs/ports/integrations.md:404-409` —
  `IntegrationCapability::required_capabilities: Vec<Capability>`
  is the engine's per-action RBAC hook; action ↔ capability
  mapping must be one-to-one and discriminative.

**Evidence:**

`crates/adapters/integrations/src/lms.rs:457`
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

### FINDING 18 (id: `ADAPTER-INT-018`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/video.rs:235-241`, `crates/adapters/integrations/src/port.rs:425-432`

**Description:**

`VideoConferencingIntegration::health()`
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

**Expected:**

`crates/adapters/integrations/src/port.rs:425-431`
  — "adapters that have never run a probe report Timestamp::epoch()
  so consumers can render 'never' explicitly."

**Evidence:**

`crates/adapters/integrations/src/video.rs:235-241`
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

### FINDING 19 (id: `ADAPTER-INT-019`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/services.rs:250-276`

**Description:**

`RetryService::next_backoff` silently swallows
  negative or overflowing `chrono::Duration` values via
  `unwrap_or(Duration::from_secs(1))` and `unwrap_or(Duration::from_secs(30))`.
  A caller who configures `base: ChronoDuration::seconds(-5)` or
  `max: ChronoDuration::max_value()` gets the documented default
  values with no error or warning. This makes per-tenant retry
  policy misconfiguration undetectable.

**Expected:**

`docs/ports/integrations.md:182-193` ("Retry
  Policy") — adapter must apply the configured policy faithfully;
  silent substitution violates the contract.

**Evidence:**

`crates/adapters/integrations/src/services.rs:271-273`
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

### FINDING 20 (id: `ADAPTER-INT-020`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/services.rs:479-488`

**Description:**

`exponential_backoff` returns
  `Duration::from_nanos(u64::try_from(scaled).unwrap_or(0))`. When
  the saturated multiplication overflows `u64`, the function
  returns `Duration::ZERO` instead of the documented `max`. A
  retry loop calling `next_backoff` in this regime would
  busy-loop the integration.

**Expected:**

AGENTS.md § "Type Safety" — "No `as` casts that
  truncate or lose data. Use `TryFrom` / `TryInto` with proper
  error handling."

**Evidence:**

`crates/adapters/integrations/src/services.rs:486-487`
  ```rust
  let scaled = (base_nanos.saturating_mul(u128::from(factor))).min(max_nanos);
  Duration::from_nanos(u64::try_from(scaled).unwrap_or(0))
  ```
  The `attempt >= 64` guard at `:480-482` covers the shift, but
  not the multiply or the cast.

---

### FINDING 21 (id: `ADAPTER-INT-021`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/services.rs:358-391`, `crates/adapters/integrations/src/lms.rs:186-237`, `crates/adapters/integrations/src/video.rs:191-242`

**Description:**

`RateLimitService::try_acquire` is defined
  and tested (`services.rs:715-768`) but no integration impl
  ever calls it. The LMS and Video adapters have no
  rate-limiting gate at all; the per-call burst from a malicious
  or buggy consumer can hit the provider's 429 immediately.

**Expected:**

`docs/ports/integrations.md:60` — `RateLimited`
  status is in the contract; the port spec assumes adapters
  throttle proactively per the provider's quota.

**Evidence:**

`grep 'RateLimitService\|try_acquire' crates/adapters/integrations/src/lms.rs crates/adapters/integrations/src/video.rs crates/adapters/integrations/src/webhook_out.rs`
  returns no production call sites. The service is only exercised
  by the unit test at `services.rs:715-768` and the integration
  test at `tests/integrations_integration.rs:125-134`.

---

### FINDING 22 (id: `ADAPTER-INT-022`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/lms.rs:505-545`, `crates/adapters/integrations/src/video.rs:397-437`

**Description:**

The `parse_response`, `infrastructure`,
  `json_infrastructure`, and `status_from_error` helpers are
  duplicated byte-for-byte between `lms.rs:505-545` and
  `video.rs:397-437`. There is no shared `http` helper module
  in `services.rs`. The same code is also duplicated in
  `webhook_out.rs` with a slightly different shape. A bug fix
  in one copy would silently miss the others.

**Expected:**

AGENTS.md § "Module Layout" — single source of
  truth per operation. Per the `PortAdapters` precedent in
  `crates/adapters/auth/`, shared helpers go in `services.rs`.

**Evidence:**

`crates/adapters/integrations/src/lms.rs:527-529`
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

### FINDING 23 (id: `ADAPTER-INT-023`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/integrations/Cargo.toml:25`

**Description:**

`indexmap = { workspace = true }` is declared
  in `crates/adapters/integrations/Cargo.toml:25` but is not
  used anywhere in the crate (the `BTreeMap`/`HashMap` usages
  don't import it). Unused dependency adds to compile time and
  binary size, and signals the crate was authored with an
  incomplete mental model.

**Expected:**

AGENTS.md § "Package Manager" — use `cargo add`
  and prune unused deps.

**Evidence:**

`grep -rn 'indexmap\|IndexMap' crates/adapters/integrations/src/`
  returns no matches. Workspace declaration at
  `crates/adapters/integrations/Cargo.toml:25`. `cargo build
  --package educore-integrations` succeeds without it.

---

### FINDING 24 (id: `ADAPTER-INT-024`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/webhook_out.rs:120-142`, `crates/adapters/integrations/src/webhook_out.rs:347-394`

**Description:**

`WebhookTarget::url` is a `String`, not a
  parsed `Url`. The builder does not validate the URL on
  construction; it accepts `"not-a-url"`, `"file:///etc/passwd"`,
  `"http://"`, or any other malformed string. The
  `webhook_out.rs:351-353` doc comment acknowledges this
  ("builder does not validate the URL syntax — that's deferred
  to the first invoke() call so misconfiguration surfaces at
  dispatch time, not at wiring time"), but it also means a
  caller who wires 100 webhook targets cannot tell at startup
  which are broken.

**Expected:**

`docs/ports/integrations.md:136-143` — the spec
  example uses `Url::parse("https://school.example.com/hooks/educore")?`
  (a `url::Url`, not a `String`). The port contract requires
  URL validation at construction.

**Evidence:**

`crates/adapters/integrations/src/webhook_out.rs:130`
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

### FINDING 25 (id: `ADAPTER-INT-025`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/webhook_out.rs:213-230`

**Description:**

`WebhookOutIntegration::deliver` wraps the
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

**Expected:**

`docs/ports/integrations.md:71-75` (errors.rs doc)
  — "Infrastructure — the adapter could not reach the provider at
  all (DNS, TCP, TLS, serialization). Carries the underlying
  error as a `source` for diagnostic logging." The original
  error should be preserved, not stringified.

**Evidence:**

`crates/adapters/integrations/src/webhook_out.rs:222-229`
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

### FINDING 26 (id: `ADAPTER-INT-026`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/webhook_out.rs:213-230`, `crates/adapters/integrations/src/webhook_out.rs:243-321`

**Description:**

The webhook-out fan-out iterates targets
  serially. With N targets at HTTP_TIMEOUT_SECS = 30 s each, a
  dispatch can take 30·N seconds end-to-end. No parallelism,
  no timeout budget, no abort-on-first-error option. A single
  slow target stalls the entire batch; a single hanging target
  hits the 30 s timeout, then the next target starts.

**Expected:**

`docs/ports/integrations.md` does not specify
  concurrency, but per AGENTS.md "Production-ready. Real schools,
  real students, real money." — 100 webhooks serially is not
  production-ready.

**Evidence:**

`crates/adapters/integrations/src/webhook_out.rs:256-278`
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

### FINDING 27 (id: `ADAPTER-INT-027`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/webhook_out.rs:213-321`

**Description:**

`WebhookOutIntegration::deliver` does not
  forward `request.correlation_id` or `request.idempotency_key`
  to the receiver. LMS and Video do set `X-Correlation-Id` and
  `Idempotency-Key` headers on every request; the webhook out
  adapter sets neither. The port contract requires the
  correlation id to be copied into "every outbound HTTP header
  (`X-Correlation-Id`)".

**Expected:**

`crates/adapters/integrations/src/port.rs:257-260`
  — "Correlation id for log stitching across the engine. The
  adapter copies it into every outbound HTTP header
  (`X-Correlation-Id`) and every audit log entry."

**Evidence:**

`crates/adapters/integrations/src/webhook_out.rs:213-230`
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

### FINDING 28 (id: `ADAPTER-INT-028`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/lms.rs:434-446`, `crates/adapters/integrations/src/video.rs:328-340`, `crates/adapters/integrations/src/webhook_out.rs:282-296`, `crates/adapters/integrations/src/webhook_out.rs:306-320`

**Description:**

`response_metadata` writes keys as
  `"x-correlation-id"` and `"idempotency-key"` (lowercase), but
  the HTTP headers sent on the wire are `"X-Correlation-Id"`
  and `"Idempotency-Key"` (HTTP title-case convention). The
  metadata BTreeMap is supposed to capture the wire-level
  identifiers for log stitching; the case mismatch means log
  search by header value finds nothing matching the metadata
  key.

**Expected:**

`crates/adapters/integrations/src/port.rs:303-307`
  — "Provider-specific metadata (request id, rate-limit
  remaining, traceparent, etc.). Always non-empty for a response
  that actually reached the provider."

**Evidence:**

`crates/adapters/integrations/src/lms.rs:436-442`
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

### FINDING 29 (id: `ADAPTER-INT-029`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/port.rs:186-220`, `crates/adapters/integrations/src/port.rs:212-220`

**Description:**

`RetryPolicy::Default::default()` is
  implemented inline on the enum and returns
  `RetryPolicy::Exponential { max_retries: 3, base: seconds(1),
  max: seconds(30) }`. The doc-test comment in
  `docs/ports/integrations.md:217-237` and the worked example
  do not specify a default. A consumer who constructs a config
  via `..Default::default()` will silently use exponential backoff
  with 3 retries when the provider might require a different
  policy (e.g. Zoom's 5xx retry guidance, Stripe's aggressive
  rate-limit backoff).

**Expected:**

`docs/ports/integrations.md:182-193` — the policy
  is part of the per-tenant config; the engine should pick it
  up from there, not apply a hard-coded engine-wide default.

**Evidence:**

`crates/adapters/integrations/src/port.rs:212-220`
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

### FINDING 30 (id: `ADAPTER-INT-030`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/lms.rs:75-89`, `crates/adapters/integrations/src/video.rs:71-83`

**Description:**

The LMS and Video impls register two
  different identifiers for the same integration. The
  `LMS_INTEGRATION_ID` constant is `"lms"`, but
  `LmsIntegrationBuilder::build()` defaults the `provider` field
  to `"google_classroom"` (`:149`). A consumer who builds the
  adapter without overrides registers as `"lms"` for the
  audit/telemetry surface but as `"google_classroom"` for
  per-provider routing. `VIDEO_INTEGRATION_ID` is
  `"video_conferencing"` but the default provider is `"zoom"`.

**Expected:**

`docs/ports/integrations.md:41-43` —
  "IntegrationId is a typed enum or string identifier for the
  integration." A single integration must have a single id.

**Evidence:**

`crates/adapters/integrations/src/lms.rs:78`
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

### FINDING 31 (id: `ADAPTER-INT-031`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/errors.rs:314-330`

**Description:**

`errors.rs` contains a `#[allow(dead_code)]`
  function `_ensure_traits_used` whose only purpose is to
  silence unused-import warnings on `serde::de::{Deserialize,
  Deserializer, Visitor}` and `serde::ser::{Serialize, Serializer}`
  that are in fact used by the manual impls at `:172-308`. The
  function exists purely to satisfy the linter. It is dead code
  that masks a tooling issue.

**Expected:**

AGENTS.md § "Type Safety" — "No `#[allow(dead_code)]`
  or `_var` prefixes to silence the compiler."

**Evidence:**

`crates/adapters/integrations/src/errors.rs:314-330`
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

### FINDING 32 (id: `ADAPTER-INT-032`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/port.rs:281-308`, `crates/adapters/integrations/src/port.rs:283`

**Description:**

`IntegrationResponse` uses
  `#[derive(Debug)]` without field-level redaction. `Debug`
  formatting of an `IntegrationResponse` will print `output:
  Some(<full JSON value including any PII or secret-like keys>)`
  and `error: Some(Provider("400 {body}"))`. AGENTS.md and
  `docs/code-standards.md` require sensitive fields be
  redacted; `Debug` is the surface that almost every Rust
  logging pipeline (`{:?}`, `tracing::debug!`, `eprintln!`,
  panic messages) consumes.

**Expected:**

`docs/ports/integrations.md:263-266` — "Every
  invocation, success or failure, is recorded with full
  metadata. Sensitive fields are redacted by the adapter."

**Evidence:**

`crates/adapters/integrations/src/port.rs:282`
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

### FINDING 33 (id: `ADAPTER-INT-033`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/port.rs:310-325`, `crates/adapters/integrations/src/port.rs:310-317`

**Description:**

Doc-vs-code drift in the rustdoc of
  `IntegrationCost`: the doc at
  `crates/adapters/integrations/src/port.rs:312` reads
  ```rust
  /// Mirrors the finance domain's [`Money`](educore_core::value_objects::Timestamp)
  /// shape but is duplicated here so this crate does not need a
  ```
  The link target is `Timestamp`, not `Money` — a copy/paste
  bug. The link resolves to `Timestamp`, misleading readers
  into thinking the cost type mirrors a timestamp.

**Expected:**

AGENTS.md § "Documentation" — public items must
  have accurate rustdoc.

**Evidence:**

`crates/adapters/integrations/src/port.rs:310-313`
  ```rust
  /// Provider-side monetary cost of a single integration call.
  ///
  /// Mirrors the finance domain's [`Money`](educore_core::value_objects::Timestamp)
  /// shape but is duplicated here so this crate does not need a
  ```

---

### FINDING 34 (id: `ADAPTER-INT-034`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/lms.rs:49-58`, `crates/adapters/integrations/src/video.rs:51-62`, `crates/adapters/integrations/src/webhook_out.rs:77-94`

**Description:**

The `!#[allow(clippy::module_name_repetitions)]`
  attribute at the top of every impl module means the lints
  pass even though `LmsIntegration`,
  `VideoConferencingIntegration`, and `WebhookOutIntegration`
  are in modules of the same name. Per AGENTS.md, this attribute
  is used to suppress a noisy pedantic lint, but it also
  suppresses the signal that the module / type names collide
  with items in the prelude (`webhook_out::WebhookOutIntegration`
  re-exported as `WebhookOutIntegration`).

**Expected:**

`docs/code-standards.md` § "Code Standards" —
  consistent module / item naming.

**Evidence:**

`crates/adapters/integrations/src/lms.rs:48`
  `#![allow(clippy::module_name_repetitions)]` (same on
  `video.rs:47`, `webhook_out.rs` (no attribute — but
  `services.rs:26` has it)). `webhook_out.rs` imports from
  `crate::port::*` without the attribute because the prelude
  re-exports `WebhookOutIntegration`.

---

### FINDING 35 (id: `ADAPTER-INT-035`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/webhook_out.rs:282-321`

**Description:**

`WebhookOutIntegration::invoke` returns
  `IntegrationResponse { status: Failed, ... }` when **any**
  target fails — even if 99 of 100 targets delivered
  successfully. The `dispatched_targets` count is in metadata,
  not status. Consumers cannot tell a "partial success" from a
  complete failure. The port spec lists `IntegrationStatus`
  values `Success / Accepted / RateLimited / Failed / TimedOut`
  — `PartialSuccess` is not modelled.

**Expected:**

`docs/ports/integrations.md:57-64` —
  `IntegrationStatus::Failed` is the only failure outcome; the
  current code collapses partial and total failures, leaving no
  way for the engine to decide whether to retry the failed
  targets.

**Evidence:**

`crates/adapters/integrations/src/webhook_out.rs:282-297`
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

### FINDING 36 (id: `ADAPTER-INT-036`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/webhook_out.rs:594-605`

**Description:**

`test_json_serialized_payload_is_byte_stable`
  asserts `serde_json::to_vec(&a) == serde_json::to_vec(&b)`
  for two equal `json!` values. This test passes only because
  `serde_json::Value` serializes `Map<String, Value>` using a
  `BTreeMap` internally (alphabetical key order). The test name
  implies payload stability for HMAC signing, but the underlying
  guarantee is from `serde_json`, not from this code. If
  `serde_json` ever changes its map ordering, all
  `WebhookSignatureService::verify_signature` calls for
  `IntegrationRequest::input` payloads break silently.

**Expected:**

`docs/ports/integrations.md:144-146` — "The
  adapter signs the payload with HMAC-SHA256 and posts it. The
  receiver verifies the signature." — stability of the
  serialized form is a contract.

**Evidence:**

`crates/adapters/integrations/src/webhook_out.rs:594-605`
  ```rust
  let a = json!({"event": "InvoicePaid", "amount_minor": 12500});
  let b = json!({"event": "InvoicePaid", "amount_minor": 12500});
  assert_eq!(
      serde_json::to_vec(&a).unwrap(),
      serde_json::to_vec(&b).unwrap()
  );
  ```

---

### FINDING 37 (id: `ADAPTER-INT-037`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/lms.rs:387-429`, `crates/adapters/integrations/src/lms.rs:387-429`

**Description:**

`LmsIntegration::pull_submissions` returns
  the entire submissions list as-is from the LMS into the
  output `JsonValue`. The body shape can include student names,
  email addresses, submission text (the assignment answers),
  timestamps, and IP addresses. The integration contract says
  the engine translates each submission into an
  `OnlineExamSubmitted` event with a `Source::Lms` tag, but the
  `pull_submissions` impl just forwards the raw provider
  payload. There is no field-level filtering or redaction.

**Expected:**

`docs/ports/integrations.md:96-98` — "Pulls
  assignment submissions from the LMS and emits
  `OnlineExamSubmitted` events with a `Source::Lms` tag." The
  engine is meant to control which fields flow into the
  aggregate, not the adapter.

**Evidence:**

`crates/adapters/integrations/src/lms.rs:423-428`
  ```rust
  let body = parse_response(response).await?;
  Ok(serde_json::json!({
      "course_id": course_id,
      "coursework_id": coursework_id,
      "submissions": body,
  }))
  ```

---

### FINDING 38 (id: `ADAPTER-INT-038`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/webhook_out.rs:73-76`, `crates/adapters/integrations/src/webhook_out.rs:146-152`

**Description:**

The webhook-out module docs claim "Webhook
  secrets are never written to logs, metrics, or the audit
  trail." But `WebhookTarget` derives neither `Display` nor
  `serde::Serialize`. If a future consumer serializes the
  `WebhookOutIntegration` (or any `WebhookTarget`) to JSON or
  a metrics pipeline, the `secret` field is included in clear
  text. The Debug impl redacts the secret at `webhook_out.rs:148`,
  but Display / Serialize are unprotected.

**Expected:**

`docs/ports/integrations.md:263-266` — "Sensitive
  fields are redacted by the adapter." Coverage of `Debug` alone
  is not enough.

**Evidence:**

`crates/adapters/integrations/src/webhook_out.rs:125`
  ```rust
  #[derive(Clone, PartialEq, Eq)]
  pub struct WebhookTarget {
  ```
  No `serde::Serialize`, no manual `Display`. `Debug` impl at
  `:144-152` redacts, but no other format / serialization layer
  does.

---

### FINDING 39 (id: `ADAPTER-INT-039`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/port.rs:75-93`

**Description:**

`IntegrationId::From<&str>` and
  `From<String>` both `to_owned()` the input. For `From<&str>`,
  this requires an allocation; for `From<String>`, it moves.
  The port contract at `docs/ports/integrations.md:41-43` calls
  `IntegrationId` a "typed enum or string identifier". An enum
  representation would avoid the heap allocation on every
  construction. At the rate integration ids flow through the
  engine (one per request), this is a measurable hot-path
  cost.

**Expected:**

AGENTS.md § "Production-ready. Real schools,
  real students, real money." — the type should match the
  closed-set nature of the domain.

**Evidence:**

`crates/adapters/integrations/src/port.rs:83-93`
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

### FINDING 40 (id: `ADAPTER-INT-040`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/lib.rs:1-127`

**Description:**

`cargo doc --no-deps --package educore-integrations`
  emits 20+ "unresolved link" warnings. Every module-level doc
  comment in `lib.rs` uses intra-doc links to items that are
  not yet in scope at the link site (e.g.,
  `[`IntegrationGateway`](port::IntegrationGateway)` at `:41`
  before the module `port` is declared). Doc quality suffers;
  the rendered docs.rs page will not show the intended
  cross-references.

**Expected:**

`AGENTS.md` § "Documentation" — public items must
  have accurate, well-formed rustdoc.

**Evidence:**

Running `cargo doc --no-deps --package
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

### FINDING 41 (id: `ADAPTER-INT-041`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** `crates/adapters/integrations/src/lms.rs:417-419`

**Description:**

`LmsIntegration::pull_submissions` hard-codes
  the `pageSize` query parameter name as a lowercase literal
  `"pageSize"`. This couples the adapter to Google Classroom's
  API; Microsoft Teams Education and Moodle use different
  parameter names (`page_size`, `limit`). The provider is
  configurable via the builder's `base_url`, but the
  query-param naming is not.

**Expected:**

`docs/ports/integrations.md:87-97` ("LMS Sync") —
  "Google Classroom, Microsoft Teams for Education, Moodle" are
  all listed as in-scope providers. The reference impl is
  supposed to work across them.

**Evidence:**

`crates/adapters/integrations/src/lms.rs:417-419`
  ```rust
  .query(&[("pageSize", page_size.to_string())])
  ```

---

### FINDING 42 (id: `ADAPTER-INT-042`)

- **Source:** `docs/audit_reports/findings/wave3-integrations.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** `crates/adapters/integrations/tests/integrations_integration.rs:140-159`

**Description:**

Both `#[ignore]`-d async tests
  (`integrations_integration_async_lms_roster_sync_mock` and
  `integrations_integration_async_webhook_out_dispatch_mock`)
  construct a builder and immediately drop the result. No
  network call, no assertion, no actual scenario exercised.
  Marking them `#[ignore = "requires
  EDUCORE_PORT_ADAPTER_E2E env var"]` implies they exist for an
  end-to-end environment that is not present in CI. The tests
  are documentation, not validation.

**Expected:**

`AGENTS.md` § "Testing (TDD)" — "No dummy tests.
  Every test must validate a real-world scenario." The handoff
  doc claims these tests cover "LMS roster sync mock" and
  "webhook-out dispatch mock" — they construct nothing more
  than a builder.

**Evidence:**

`crates/adapters/integrations/tests/integrations_integration.rs:140-159`
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


## Notify (target id prefix: `ADAPTER-NOT`)

**Path:** `crates/adapters/notify/`  
**Total findings:** 74 (12 critical, 24 high, 31 medium, 7 low)


### FINDING 1 (id: `ADAPTER-NOT-001`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/email.rs:142-189` (EmailProvider::send_bulk) and `crates/adapters/notify/src/sms.rs:358-392` (SmsProvider::send_bulk)

**Description:**

Neither reference implementation honours the port-level batch boundary. The `EmailProvider::send_bulk` declares `const BULK_BATCH_SIZE: usize = 100;` at `email.rs:58` but the only reference to it (`email.rs:160-162`) is `if idx > 0 && idx % BULK_BATCH_SIZE == 0 { let _ = idx / BULK_BATCH_SIZE; }` — a no-op whose result is immediately discarded. Every recipient is dispatched via a separate `self.send(single).await` call inside one loop, with no SMTP/network batching. `SmsProvider::send_bulk` does chunk by `SMS_BULK_BATCH_SIZE` (sms.rs:375-389) but dispatches each row serially inside the chunk, so the gateway still receives N requests per chunk instead of one request carrying 100 recipients.

**Expected:**

Per `docs/ports/notifications.md` § "Bulk Send": "The adapter batches them (per channel limits, e.g. 100 SMS per batch)." and § "Testing": "A test of bulk send with partial failure."

**Evidence:**

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

### FINDING 10 (id: `ADAPTER-NOT-010`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/notify/Cargo.toml:13-32`

**Description:**

The `Cargo.toml` declares three dependencies that are never imported: `educore-audit` (line 20), `educore-events` (line 16), `educore-platform` (line 15). The handoff (PHASE-15-HANDOFF.md:42-43) lists these crates as expected integrations. `educore-audit` was specifically added so notification events would be auditable (per the port spec § "Audit": "Every send, success or failure, is recorded in the audit log"), but neither `EmailProvider::send` nor `SmsProvider::send` ever writes an audit entry. The other two are similarly unwired.

**Expected:**

Spec § "Audit": "Every send, success or failure, is recorded in the audit log with template id, recipient hash, channel, status, and cost."

**Evidence:**

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

### FINDING 11 (id: `ADAPTER-NOT-011`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/email.rs:90-140` and `crates/adapters/notify/src/sms.rs:296-340`

**Description:**

Neither provider implements any retry policy on transient errors. `EmailProvider::send` wraps `lettre` failures as `NotificationError::infrastructure` once and returns (line 119-123); `SmsProvider::dispatch` does the same for `reqwest` (line 308-319). The port spec § "Rate Limiting" requires `RateLimited` returns and § "DeliveryStatus" includes a `Failed { reason, retryable }` variant where `retryable: bool` indicates whether the engine should retry — but the providers never set `retryable`, never surface `RateLimited`, and never classify 5xx vs 4xx. They also never honour the `Critical` priority (port.rs:1005), which the spec mandates "bypass queues and are delivered synchronously".

**Expected:**

`docs/ports/notifications.md` § "Rate Limiting": "The adapter returns `NotificationError::RateLimited` when a limit is hit; the engine retries with backoff." § "DeliveryStatus": `Failed { reason, retryable }`. § "Priority": "`Critical` notifications bypass queues and are delivered synchronously."

**Evidence:**

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

### FINDING 12 (id: `ADAPTER-NOT-012`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/email.rs:107-115` (EmailProvider::send)

**Description:**

`EmailProvider::send` captures the recipient's email address into a tuple at lines 110-115 and immediately discards it via `let _ = (...)`. This is an explicit PII capture-and-discard pattern. Combined with `recipient_email` being held in scope (line 107) and used only for the SMTP envelope at line 117, the captured address could trivially flow into a future `tracing::info!(...)` call by accident; the current shape is "captured for logging that never happened". The provider's `Debug` impl (line 80-86) does not redact `default_from` (an email address). The spec mandates PII hashing before any log.

**Expected:**

Spec § "Audit": "PII (phone numbers, email addresses) is hashed before logging."

**Evidence:**

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

### FINDING 2 (id: `ADAPTER-NOT-002`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/lib.rs:28-32` and `crates/adapters/notify/src/port.rs:930-986`

**Description:**

Only two of the seven `Channel` variants defined in the port contract have reference implementations (`Email` → `EmailProvider`, `Sms` → `SmsProvider`). The port contract requires adapters for `Push`, `InApp`, `Chat` (WhatsApp/Telegram/Messenger/Signal), `Voice`, and `Webhook`. The handoff at `PHASE-15-HANDOFF.md:127-129` explicitly confirms "2 reference impls: EmailProvider + SmsProvider", and the crate's `Cargo.toml:8` description claims "Notification port, email, SMS, push, in-app, chat, voice, webhook adapters." while shipping 5 missing implementations. The handoff also wires up RBAC capabilities `NotifyPushSend`, `NotifyInApp`, `NotifyVoice`, `NotifyWebhook` (`PHASE-15-HANDOFF.md:138-141`) for which no provider code exists.

**Expected:**

Per `docs/ports/notifications.md` § "Channel", every `Channel` variant is supported: Email, Sms, Push, InApp, Chat (with ChatProvider), Voice, Webhook.

**Evidence:**

```rust
  // crates/adapters/notify/src/lib.rs:26-32
  pub mod sms;
  /// Email [`NotificationProvider`] reference
  /// implementation backed by SMTP via the `lettre` crate.
  pub mod email;
  ```
  No `pub mod push;`, `pub mod in_app;`, `pub mod chat;`, `pub mod voice;`, or `pub mod webhook;` files exist under `crates/adapters/notify/src/` (the only files are `email.rs`, `errors.rs`, `lib.rs`, `port.rs`, `services.rs`, `sms.rs`).

---

### FINDING 3 (id: `ADAPTER-NOT-003`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/email.rs:142-189` (EmailProvider::send_bulk)

**Description:**

`EmailProvider::send_bulk` does not call any underlying batch API; it iterates the recipient list serially and re-enters `self.send(single).await` for every row. Each iteration re-builds a fresh `lettre::Message`, opens a fresh SMTP command/response cycle (or one connection per row depending on pool config), and re-renders the template per recipient. The port spec mandates batched dispatch (100/batch) for both cost (one transactional vs N) and latency reasons.

**Expected:**

`docs/ports/notifications.md` § "Bulk Send": "The adapter batches them (per channel limits, e.g. 100 SMS per batch)."

**Evidence:**

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

### FINDING 4 (id: `ADAPTER-NOT-004`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/sms.rs:394-401` (SmsProvider::status) and `crates/adapters/notify/src/email.rs:191-193` (EmailProvider::status)

**Description:**

Both reference implementations return `DeliveryStatus::Sent` unconditionally from `status()`, regardless of whether the notification has actually been delivered, bounced, opened, or clicked. The port contract specifies that `status` is used to "reconcile webhook status updates" (port.rs:1407-1411) and that the engine will reflect `Delivered`, `Opened`, `Clicked`, `Bounced`, `Failed`, `Rejected` states. A stub that returns `Sent` for every receipt means the engine can never observe a bounce, a click, or any provider-confirmed failure via the status API.

**Expected:**

`docs/ports/notifications.md` § "DeliveryStatus": "The adapter updates the status as the provider reports it (via webhook). The engine polls or subscribes to status changes for reconciliation." and § "Testing": "A test of status updates (delivered, opened, clicked)."

**Evidence:**

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

### FINDING 5 (id: `ADAPTER-NOT-005`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/email.rs:90-140` (EmailProvider::send)

**Description:**

`EmailProvider::send` ignores `request.attachments`, `request.priority`, `request.scheduled_at`, `request.idempotency_key`, `request.correlation_id`, and never resolves the template body from the communication-domain template store. It uses a hardcoded `DEFAULT_TEMPLATE_BODY = "Hello {student_name}, this is a notification from Educore."` (line 67) and prepends only the template id as a `[Template: id]` prefix (line 324-329). The provider cannot render any real template, never sets `NotificationReceipt::cost`, never populates `NotificationReceipt::metadata`, and the `provider_message_id` it stores is `response.code().to_string()` (line 139) — `lettre`'s SMTP response code, not the provider's message id (e.g. SES `MessageId`, which arrives in `X-SES-Configuration-Set` headers or the server response).

**Expected:**

Per `docs/ports/notifications.md` § "Templates": "The adapter resolves the template body, applies variables, and delivers." § "Cost Tracking": "`cost: Option<Money>` is set by the adapter (e.g. $0.0075 per SMS)." § "NotificationReceipt": `provider_message_id: Option<String>` is "The provider's message id (e.g. SES `MessageId`, Twilio `MessageSid`)."

**Evidence:**

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

### FINDING 6 (id: `ADAPTER-NOT-006`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/email.rs:262-281` (EmailProviderBuilder::build)

**Description:**

`EmailProviderBuilder::build` always calls `AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&relay_host)` (line 270). The module-level doc at lines 33-38 explicitly recommends `AsyncSmtpTransport::relay` and `AsyncSmtpTransport::starttls_relay` as the "recommended constructors" and `builder_dangerous` is the lowest-level API that "consumers can wire their own TLS configuration" — but the builder itself never wires any TLS. The only authentication configuration is `Credentials::new(user, String::new())` (line 273) which uses an empty password. The transport therefore connects to port 587 with **no TLS upgrade attempted** unless the consumer manually swaps the builder, which the public API does not expose.

**Expected:**

AGENTS.md § "TLS/SSL Cross-Compilation" mandates `rustls`. The email provider must establish a STARTTLS or implicit TLS connection on every send; the `relay`/`starttls_relay` constructors in `lettre` enforce this by default.

**Evidence:**

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

### FINDING 7 (id: `ADAPTER-NOT-007`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/port.rs:43-44` and `crates/adapters/notify/src/port.rs:1422-1423`

**Description:**

`port.rs` opens with `#![allow(dead_code, clippy::all)]` and `#![allow(missing_docs)]` at lines 43-44, shadowing the crate-level `#![deny(missing_docs)]` declared in `lib.rs:10`. This means every public item in the most important file of the port (the one that defines the trait every adapter must implement) is published without rustdoc. The crate-level deny is silently inactive for this module.

**Expected:**

AGENTS.md and `docs/code-standards.md`: "All public APIs are documented with rustdoc; `#![deny(missing_docs)]`."

**Evidence:**

```rust
  // crates/adapters/notify/src/lib.rs:9-10
  #![forbid(unsafe_code)]
  #![deny(missing_docs)]

  // crates/adapters/notify/src/port.rs:43-44
  #![allow(dead_code, clippy::all)]
  #![allow(missing_docs)]
  ```

---

### FINDING 8 (id: `ADAPTER-NOT-008`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/errors.rs:75-120`

**Description:**

The shipped `NotificationError` enum at `errors.rs:75-120` deviates from the port contract. The spec (`docs/ports/notifications.md` § "Error Type") defines `Infrastructure(#[source] Box<dyn std::error::Error + Send + Sync>)` — the source error chain is preserved. The shipped enum drops the chain and stores `Infrastructure(String)` (line 119), then `NotificationError::infrastructure(source)` (line 129-131) renders the source via `.to_string()` immediately. The error type can no longer satisfy `Clone, Eq, Serialize, Deserialize` while also preserving the source chain — and the `BulkReceipt::failed` type at `port.rs:1349` (`Vec<(BulkRecipientIndex, NotificationError)>`) inherits this lossy representation.

**Expected:**

`docs/ports/notifications.md` § "Error Type": `Infrastructure(#[source] Box<dyn std::error::Error + Send + Sync>)`.

**Evidence:**

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

### FINDING 9 (id: `ADAPTER-NOT-009`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/port.rs:1188-1206` and `crates/adapters/notify/src/port.rs:1256-1261`

**Description:**

`SendNotification` and `SendBulkNotification` carry a `school_id: SchoolId` field on top of `tenant: TenantContext` (which already carries `school_id`). This is doc-vs-code drift: the spec defines `tenant: TenantContext` as the only school-identifying field (`docs/ports/notifications.md` § "SendNotification", lines 25-38, and § "SendBulkNotification", lines 136-145). The shipped struct duplicates the field and exposes `active_school_id(&self) -> SchoolId { self.school_id }` (line 1197-1199) — a redundant accessor that bypasses the tenant context. Any consumer that mutates one and forgets the other (or sets them to different values) creates an invariant violation that cannot be detected at compile time.

**Expected:**

`docs/ports/notifications.md` § "SendNotification": only `tenant: TenantContext` (which carries `school_id`); no `school_id` field.

**Evidence:**

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

### FINDING 13 (id: `ADAPTER-NOT-013`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/sms.rs:81-91` (SmsProvider Debug impl)

**Description:**

`SmsProvider`'s manual `Debug` impl at sms.rs:81-91 correctly redacts `api_key` to `"<redacted>"` but exposes `default_from` (a phone number, PII) and the `templates` keys (template ids, not strictly PII but visible). The `api_key` redaction is inconsistent with the `EmailProvider`, which does not redact `default_from` (see ADAPTER-NOT-012). There is no consistent redaction policy across providers.

**Expected:**

Spec § "Audit": "PII (phone numbers, email addresses) is hashed before logging."

**Evidence:**

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

### FINDING 14 (id: `ADAPTER-NOT-014`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/sms.rs:321-323` (SmsProvider::dispatch)

**Description:**

`SmsProvider::dispatch` calls `response.text().await.unwrap_or_default()` (line 323), consuming the entire response body into memory just to scan for a `sid`. For an unbounded gateway response this is a memory and DoS surface. The `unwrap_or_default()` on `.text()` also silently swallows non-UTF-8 response bodies, hiding real errors from the caller.

**Expected:**

Standard HTTP client practice: bound the body size, return a typed error when the body cannot be decoded, and parse only a header-sized slice (or use streaming JSON).

**Evidence:**

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

### FINDING 15 (id: `ADAPTER-NOT-015`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/sms.rs:457-468` (extract_provider_message_id)

**Description:**

`extract_provider_message_id` is a hand-rolled JSON string scanner at sms.rs:457-468 that calls `body.find("\"sid\"")` then walks forward to find the next `:` and matching `"`. It does not handle JSON escape sequences, so any string value containing `\"sid\"` would be mis-parsed. It does not validate that the `sid` value is a string vs an integer or array, and there is no length cap on the matched value. The function is called on a body that the consumer does not control.

**Expected:**

Use a real JSON parser (e.g. `serde_json::Value`) for the response, or at minimum cap the scan length and reject malformed JSON.

**Evidence:**

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

### FINDING 16 (id: `ADAPTER-NOT-016`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/sms.rs:375-389` (SmsProvider::send_bulk)

**Description:**

`SmsProvider::send_bulk` builds `BulkRecipientIndex::new(u32::try_from(global_idx).unwrap_or(u32::MAX))` where `global_idx = receipt.total()` (lines 377-378). On the first iteration `total()` is 0; on the second iteration (whether the first succeeded or failed) `total()` reflects the running count of receipts+failed. Across chunks, the index drifts: if row 0 of chunk 1 succeeds and row 1 of chunk 1 fails, the failure is reported with index 1 (the total count) instead of the original input row index. The engine cannot correlate the failure to its source row, which is the express purpose of `BulkRecipientIndex` per the port spec § "BulkReceipt".

**Expected:**

`docs/ports/notifications.md` § "BulkReceipt": `failed: Vec<(BulkRecipientIndex, NotificationError)>` where `BulkRecipientIndex` is "the original input row index".

**Evidence:**

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

### FINDING 17 (id: `ADAPTER-NOT-017`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/sms.rs:286-289` (SmsProvider::basic_auth_header) and `crates/adapters/notify/src/sms.rs:136-146` (SmsProviderBuilder::api_key)

**Description:**

`SmsProviderBuilder::api_key` (line 143-146) takes the user-supplied key and uses it directly as the username half of HTTP Basic auth. For Twilio this is wrong: Twilio uses `Basic <base64(AccountSID:AuthToken)>`, requiring **two** secrets. The builder only takes one field, and the docstring at lines 140-141 punts the responsibility to the consumer ("consumers needing full `{SID}:{AuthToken}` auth should pre-encode the pair as base64 and pass the resulting string here"). The actual `basic_auth_header` then wraps the already-base64 value in `format!("Basic {}", base64_encode(format!("{}:", self.api_key)))` (line 287-288) — double-encoding if the consumer followed the docstring. The credential handling is broken by design.

**Expected:**

Two distinct builder methods: `account_sid(...)` and `auth_token(...)` (or a single pre-encoded value explicitly labelled as such), with the header built correctly for both raw and pre-encoded inputs.

**Evidence:**

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

### FINDING 18 (id: `ADAPTER-NOT-018`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/email.rs:294-313` (resolve_email_recipient)

**Description:**

`resolve_email_recipient` rejects every indirect recipient variant (`Recipient::User/Student/Guardian/Staff/Group/List/Expression`) with the message "engine must materialise indirect recipients before sending" (line 309-311). The port contract `docs/ports/notifications.md` § "Recipient" specifies that the adapter receives a "materialized list" only for `Recipient::Expression`, but `User`, `Student`, `Staff`, and `Group` ids are first-class recipient types the adapter must accept and resolve. The reference email provider therefore cannot send to any user, student, staff member, or group — the overwhelming majority of real recipients. The provider is not functional end-to-end without an out-of-band wrapper.

**Expected:**

Spec § "Recipient": "The recipient" types include `User(UserId)`, `Student(StudentId)`, `Guardian(StudentId, GuardianRole)`, `Staff(StaffId)`, `Group(GroupId)` as first-class variants the adapter must dispatch on.

**Evidence:**

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

### FINDING 19 (id: `ADAPTER-NOT-019`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/email.rs:294-313` and `crates/adapters/notify/src/sms.rs:208-228` (SmsProvider::recipient_phone)

**Description:**

Both `resolve_email_recipient` (email.rs:294-313) and `recipient_phone` (sms.rs:208-228) reject `Recipient::List` and `Recipient::Expression` with errors ("nested recipient list not expanded by engine" / "recipient expression not expanded by engine"). The port spec § "Recipient" states explicitly that `Recipient::List` is a "flat list of recipients, delivered as a single logical send" and `Recipient::Expression` is "evaluated by the engine using the query layer; the adapter receives the materialized list." The reference implementations are inconsistent with this: a single `SendNotification` carrying `Recipient::List([...])` cannot be sent; the consumer must pre-walk the list and call `send` once per element.

**Expected:**

Spec § "Recipient": `List(Vec<Recipient>)` is a first-class recipient; `Expression(RecipientExpr)` is expanded by the engine before the adapter sees it. Adapters must accept `List` (with internal fan-out) and never see `Expression`.

**Evidence:**

```rust
  // crates/adapters/notify/src/sms.rs:214-219
  Recipient::List(_) => Err(NotificationError::InvalidRecipient(
      "nested recipient list not expanded by engine".into())),
  Recipient::Expression(_) => Err(NotificationError::InvalidRecipient(
      "recipient expression not expanded by engine".into())),
  ```
  The same `Recipient::List` error appears in `email.rs:308-310`.

---

### FINDING 20 (id: `ADAPTER-NOT-020`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/email.rs:88-140` (EmailProvider::send)

**Description:**

`EmailProvider::send` does not set `NotificationReceipt::cost` (line 133-139 — only `provider_message_id` is set). The port spec § "Cost Tracking" mandates `cost: Option<Money>` be set by the adapter (e.g. $0.0075 per SMS; equivalent for email). The provider returns a receipt with `cost: None` (default at `port.rs:1309`), so the engine cannot track per-tenant cost.

**Expected:**

`docs/ports/notifications.md` § "Cost Tracking": "`cost: Option<Money>` is set by the adapter (e.g. $0.0075 per SMS). The engine logs the cost for tenant-level reporting and budget control."

**Evidence:**

```rust
  // crates/adapters/notify/src/email.rs:133-139
  Ok(NotificationReceipt::new(receipt_id, request.channel,
      DeliveryStatus::Sent, Timestamp::now())
      .with_provider_message_id(response.code().to_string()))
  ```
  No `.with_cost(...)` call; the `cost` field remains `None`.

---

### FINDING 21 (id: `ADAPTER-NOT-021`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/email.rs:88-140` and `crates/adapters/notify/src/sms.rs:296-340`

**Description:**

Neither `EmailProvider::send` nor `SmsProvider::dispatch` populates `NotificationReceipt::metadata`. The port spec defines `metadata: BTreeMap<String, String>` for "Provider-specific metadata (e.g. SES `RequestId`, FCM `message_id`)" (port.rs:1289-1291) and the `with_metadata` builder exists at `port.rs:1331-1334`. Both providers return receipts with an empty metadata map.

**Expected:**

`docs/ports/notifications.md` § "NotificationReceipt": `metadata: BTreeMap<String, String>` for provider-specific data.

**Evidence:**

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

### FINDING 22 (id: `ADAPTER-NOT-022`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/email.rs:90-140` and `crates/adapters/notify/src/sms.rs:296-340`

**Description:**

Neither provider honours `request.scheduled_at`, `request.priority`, `request.idempotency_key`, or `request.correlation_id`. None of these fields is read after the struct is destructured. `Critical` priority is supposed to "bypass queues and be delivered synchronously" (spec § "Priority") but the providers do not check it; idempotency keys are not used to dedupe retries; scheduled delivery is sent immediately; correlation IDs do not propagate to any log/event.

**Expected:**

`docs/ports/notifications.md` § "Idempotency": "idempotency_key is used by the adapter to deduplicate retries." § "Priority": "`Critical` notifications bypass queues and are delivered synchronously." § "scheduled_at": "Optional scheduled delivery time. `None` means 'send now'."

**Evidence:**

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

### FINDING 23 (id: `ADAPTER-NOT-023`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/email.rs:88-140` and `crates/adapters/notify/src/sms.rs:296-340`

**Description:**

Neither provider uses the in-crate `IdempotencyService` or `RateLimitService` helpers. `EmailProvider` and `SmsProvider` both ignore `request.idempotency_key`, never call `IdempotencyService::is_duplicate`, and never call `RateLimitService::try_acquire`. The helpers exist (`services.rs:445-478` and `services.rs:505-566`) and the spec requires their use, but the only consumers of these services are the unit tests and the integration tests.

**Expected:**

`docs/ports/notifications.md` § "Idempotency" and § "Rate Limiting": adapters enforce both.

**Evidence:**

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

### FINDING 24 (id: `ADAPTER-NOT-024`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/notify/tests/notify_integration.rs:122-142`

**Description:**

The two env-gated async integration tests (`notify_integration_async_email_send_mock` and `notify_integration_async_sms_send_mock`) construct a provider with `.build()` and immediately bind it to `let _provider = ...`. They perform no assertions, no actual send, no wire-format check, and no status verification. They exercise nothing beyond the builder's `build` method (which is already covered by sync unit tests at `email.rs:434-445` and `sms.rs:546-562`).

**Expected:**

`docs/ports/notifications.md` § "Testing": "Integration tests of template resolution, variable application, and idempotency. A test of bulk send with partial failure. A test of rate limiting and retry. A test of status updates (delivered, opened, clicked)."

**Evidence:**

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

### FINDING 25 (id: `ADAPTER-NOT-025`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/notify/tests/notify_integration.rs:1-142` (full file)

**Description:**

The integration test file exercises only the four pure helper services (`TemplateService`, `ChannelService`, `IdempotencyService`, `RateLimitService`). It contains zero tests of `NotificationProvider::send`, `send_bulk`, or `status` (covered only by inline `#[cfg(test)] mod tests` inside `email.rs` and `sms.rs`). It has no test for: every `Channel` variant (spec § "Testing" — "Unit tests of every `Channel` variant"), `Recipient::Expression` evaluation, `cost` tracking, status updates (delivered/opened/clicked), bounce handling, attachment handling, scheduled delivery, or RBAC enforcement. The handoff at `PHASE-15-HANDOFF.md:143-146` claims "5 sync + 2 env-gated integration tests" but the env-gated tests are no-ops (see ADAPTER-NOT-024) and the sync tests don't exercise the provider.

**Expected:**

Spec § "Testing" enumerates 7 specific test scenarios.

**Evidence:**

```rust
  // crates/adapters/notify/tests/notify_integration.rs (entire file, 142 lines)
  // 5 #[test] functions, all calling helpers in the public prelude:
  //   TemplateService, ChannelService, IdempotencyService, RateLimitService.
  // No test imports NotificationProvider, EmailProvider, or SmsProvider
  //   for an actual send().
  ```

---

### FINDING 26 (id: `ADAPTER-NOT-026`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/email.rs:69-86` and `crates/adapters/notify/src/sms.rs:71-91`

**Description:**

Both provider structs derive `Debug` (email.rs:80-86) or implement it manually (sms.rs:81-91) but **neither struct is `Send + Sync`-derivable from its fields alone** — `SmsProvider` contains a `reqwest::Client` (which is `Send + Sync`) and primitives (fine) but lacks an explicit `Send + Sync` bound and never includes a `static_assertions::assert_impl_all` style compile-time check. While both will be `Send + Sync` in practice, the trait bound `NotificationProvider: Send + Sync + std::fmt::Debug` (port.rs:1397) is satisfied only implicitly; if a future field change adds a non-`Send` field, the trait contract fails silently because the trait's bound matches the struct's auto-trait until exercised.

**Expected:**

Spec § "Object Safety" / port.rs:1397 mandates `Send + Sync + Debug`.

**Evidence:**

```rust
  // crates/adapters/notify/src/port.rs:1396-1398
  #[async_trait]
  pub trait NotificationProvider: Send + Sync + std::fmt::Debug {
  ```
  Neither `email.rs` nor `sms.rs` contains a compile-time `assert_impl_all!(EmailProvider: Send + Sync)` or equivalent.

---

### FINDING 27 (id: `ADAPTER-NOT-027`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/sms.rs:286-289` (SmsProvider::basic_auth_header)

**Description:**

`basic_auth_header` builds the value `format!("Basic {}", base64_encode(format!("{}:", self.api_key)))`. The `api_key` is concatenated with `:` and then base64-encoded. For Twilio, this is wrong: Twilio expects `Basic base64(AccountSID:AuthToken)`, where both halves are the consumer's two distinct secrets. Passing only `AccountSID` (with empty token) yields an unauthenticated request; passing `base64(AccountSID:AuthToken)` (already base64-encoded, per the docstring at sms.rs:140-141) yields double-encoding. The provider cannot talk to real Twilio in either configuration.

**Expected:**

Either take `account_sid` + `auth_token` separately and base64-encode once, or take a single `Basic <already-encoded>` string verbatim.

**Evidence:**

```rust
  // crates/adapters/notify/src/sms.rs:286-289
  fn basic_auth_header(&self) -> String {
      let raw = format!("{}:", self.api_key);
      format!("Basic {}", base64_encode(raw.as_bytes()))
  }
  ```

---

### FINDING 28 (id: `ADAPTER-NOT-028`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/port.rs:931` (Channel derives)

**Description:**

`Channel` (the enum that drives the entire port) derives only `Debug, Clone, PartialEq, Eq, Serialize, Deserialize` (line 931). It does not derive `Hash`, `Default`, or `Copy`. The services module (`services.rs:204-209`) is forced to use a `HashMap<String, RateState>` keyed by a hand-rolled `channel_key` because `Channel: Hash` is unavailable. The spec calls out this deviation explicitly (`services.rs:42-48`). `RateLimitService`, `ChannelService`, and `IdempotencyService` consumers cannot use the enum as a key directly. The lack of `Copy` makes every match arm and every function signature pay for a heap-resident `String` clone when passing channels around.

**Expected:**

Spec expects `Channel` to be a value type; the port.rs comment (line 32-37) acknowledges the lack of `Hash` and works around it. The workaround is a deviation, not a fix.

**Evidence:**

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

### FINDING 29 (id: `ADAPTER-NOT-029`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/email.rs:88-140` (EmailProvider::send)

**Description:**

`EmailProvider::send` does not handle the recipient's attachment list. `request.attachments: Vec<AttachmentRef>` is part of `SendNotification` (port.rs:1174) but the function never reads it; `build_lettre_message` (email.rs:364-395) constructs the email from a single `body: &str` with no MIME multipart construction. Per the spec § "SendNotification", attachments are first-class and the adapter "resolves the template body, applies variables, and delivers" — including any attachments. The provider silently drops them.

**Expected:**

Spec § "SendNotification": `attachments: Vec<AttachmentRef>` — a reference to a file attached to a notification.

**Evidence:**

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

### FINDING 30 (id: `ADAPTER-NOT-030`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/sms.rs:296-340` (SmsProvider::dispatch)

**Description:**

`SmsProvider::dispatch` does not handle `Channel::Sms.unicode` (port.rs:949). The unicode flag is destructured at sms.rs:255 as `unicode: _` (the leading underscore shows it is intentionally discarded) and never influences the wire format. Twilio and most gateways split unicode (UCS-2) bodies into 70-character segments vs 160 for GSM-7. The provider sends unicode text as if it were GSM-7, garbling the message at the gateway.

**Expected:**

Spec § "Channel": `Sms { from: Option<PhoneNumber>, unicode: bool }` — `unicode: true` means the adapter must use UCS-2 encoding.

**Evidence:**

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

### FINDING 31 (id: `ADAPTER-NOT-031`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/email.rs:88-140` (EmailProvider::send) and `crates/adapters/notify/src/email.rs:142-189` (EmailProvider::send_bulk)

**Description:**

`EmailProvider` never sets the SMTP envelope's `MAIL FROM` to a per-tenant return-path. `Channel::Email.from` overrides only the From: header; the SMTP `MAIL FROM` uses the default `default_from` and the same builder-configured host. Multi-tenant sends from a single `EmailProvider` therefore share a single bounce-domain envelope, breaking Bounce / FBL processing per tenant. The port spec § "Multi-tenancy" implicitly requires per-tenant envelope identity.

**Expected:**

Spec § "Channel::Email": `from: Option<EmailAddress>` — the adapter must use this for the From: header AND (for SES, Postmark, etc.) the envelope sender / MAIL FROM.

**Evidence:**

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

### FINDING 32 (id: `ADAPTER-NOT-032`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/email.rs:69-78` (EmailProvider struct)

**Description:**

`EmailProvider` is `Clone`-derived but the inner `AsyncSmtpTransport<Tokio1Executor>` is the only field that holds connection state. The `default_from: String` is the only per-tenant configuration; there is no way to configure separate per-tenant reply-to, header-from, or envelope-from addresses. The struct therefore models a single SMTP account, not a multi-tenant provider; multi-tenant deployments must construct a separate `EmailProvider` per tenant, defeating the `Arc<dyn NotificationProvider>` storage model in the engine.

**Expected:**

Spec § "Multi-tenancy": every send carries `tenant: TenantContext`; the adapter must support many tenants over one connection pool.

**Evidence:**

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

### FINDING 33 (id: `ADAPTER-NOT-033`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/email.rs:191-193` (EmailProvider::status)

**Description:**

`EmailProvider::status` ignores its `_receipt_id` argument entirely and returns `DeliveryStatus::Sent` for every call. Unlike `SmsProvider::status`, which has a doc comment acknowledging it is a stub, `EmailProvider::status` has no such comment — it presents as a complete implementation. The provider has no way to query SES `GetMessageInsights`, Postmark's `MessageInfo`, or any other provider status endpoint.

**Expected:**

Spec § "DeliveryStatus": "The adapter updates the status as the provider reports it (via webhook)."

**Evidence:**

```rust
  // crates/adapters/notify/src/email.rs:191-193
  async fn status(&self, _receipt_id: NotificationReceiptId) -> Result<DeliveryStatus> {
      Ok(DeliveryStatus::Sent)
  }
  ```

---

### FINDING 34 (id: `ADAPTER-NOT-034`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/sms.rs:476-484` (generate_id)

**Description:**

`generate_id` uses `SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_micros()).unwrap_or(0)` and pairs it with an in-process `AtomicU64` counter. The result is a string `sms-<micros_hex>-<counter_hex>` that is unique per (process, microsecond, counter-tick). On process restart the counter resets to 0, so two concurrent processes can produce the same `receipt_id` in the same microsecond. `NotificationReceiptId` is supposed to be durable (`port.rs:67-73`: "The engine stores it in `communication_email_sms_logs`"), and a duplicate receipt id collides on insert.

**Expected:**

Spec § "NotificationReceipt": `receipt_id` is durable and unique.

**Evidence:**

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

### FINDING 35 (id: `ADAPTER-NOT-035`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/email.rs:125-131` (EmailProvider receipt id construction)

**Description:**

`EmailProvider::send` constructs receipt ids with `format!("email:{log_school}:{}", SystemTime::now()...)`, embedding the school id in plaintext into every receipt id. The spec § "Audit": "PII (phone numbers, email addresses) is hashed before logging." School id is tenant PII in this domain; receipt ids flow into the durable `communication_email_sms_logs` table and into logs. The `BulkId` and `EmailProvider::send_bulk` follow the same pattern at email.rs:149-155. Also note that `SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0)` returns 0 silently on clock skew.

**Expected:**

Spec § "Audit": PII hashing.

**Evidence:**

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

### FINDING 36 (id: `ADAPTER-NOT-036`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/port.rs:1277` (NotificationReceipt.provider_message_id)

**Description:**

The `provider_message_id` field on `NotificationReceipt` is stored verbatim from whatever the provider returns. `EmailProvider::send` stores `response.code().to_string()` (email.rs:139) — lettre's SMTP response code (e.g. `"250"`), not the provider's message id (e.g. SES `MessageId` is in the response headers / X-SES-Configuration-Set). `SmsProvider::dispatch` uses a hand-rolled JSON scan (sms.rs:457-468). Neither provider produces a usable correlation id for webhook reconciliation.

**Expected:**

Spec § "NotificationReceipt": `provider_message_id` is "The provider's message id (e.g. SES `MessageId`, Twilio `MessageSid`). Used to reconcile webhook status updates."

**Evidence:**

```rust
  // crates/adapters/notify/src/email.rs:139
  .with_provider_message_id(response.code().to_string())
  ```
  `response.code()` is lettre's `lettre::transport::smtp::response::Response.code()`, the SMTP status code, not a message id.

---

### FINDING 37 (id: `ADAPTER-NOT-037`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/port.rs:107-145` (BulkId) and `crates/adapters/notify/src/sms.rs:476-484` (generate_id)

**Description:**

`BulkId` and `NotificationReceiptId` are both `String`-backed newtypes, and both are generated by `SmsProvider` via `generate_id("bulk")` (sms.rs:365) and `generate_id("sms")` (sms.rs:305). The id is process-local and not derived from the canonical UUID ecosystem (`educore_core::ids`). The engine's storage adapter expects UUID-shaped ids per the `communication_email_sms_logs` schema. The id generation is also unrelated to the `IdempotencyService::derive_key` SHA-256 output (which is the spec's deterministic-idempotency-key path).

**Expected:**

Spec § "Idempotency": the engine generates a deterministic key from `(command_id, recipient, template_version)`.

**Evidence:**

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

### FINDING 38 (id: `ADAPTER-NOT-038`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/services.rs:73-176` (hand-rolled SHA-256)

**Description:**

`services.rs` ships a hand-rolled SHA-256 implementation (lines 73-176) with a 100+ line block, claiming FIPS 180-4 §6.2 compliance. The crate's `Cargo.toml` does not declare the `sha2` crate and the task spec for this file lists the manifest under "DO NOT TOUCH" (services.rs:35-41, 65-71). The same SHA-256 implementation is duplicated in `crates/adapters/files/src/local.rs` per the comment at services.rs:38-41. Hand-rolled crypto is a major audit risk: a single off-by-one in the padding (`while buf.len() % 64 != 56 { buf.push(0x00); }` line 103-105), the rotate constants, or the initial hash values produces silent corruption. The `sha2` crate is already in the workspace dependency graph (the auth crate uses `hmac 0.12` per `PHASE-15-HANDOFF.md:247`).

**Expected:**

Use the audited `sha2` crate for SHA-256; the workspace already pulls in `hmac`, so the dependency is already justified.

**Evidence:**

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

### FINDING 39 (id: `ADAPTER-NOT-039`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/services.rs:486-566` (RateLimitService)

**Description:**

`RateLimitService` is documented (services.rs:30-32) and tested (services.rs:677-706, integration test at notify_integration.rs:99-116) but no provider uses it. `EmailProvider` and `SmsProvider` both lack any rate-limit enforcement, meaning a tenant with `Critical` priority or a flood of sends can exceed gateway throttling and be blacklisted. Spec § "Rate Limiting" requires per-tenant, per-channel limits configurable per tenant — the shipped service is process-local (`HashMap<String, RateState>`), single-tenant, and never wired in.

**Expected:**

Spec § "Rate Limiting": per-tenant, per-channel limits enforced by the adapter.

**Evidence:**

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

### FINDING 40 (id: `ADAPTER-NOT-040`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/email.rs:336-356` (substitute_variables and value_to_string)

**Description:**

`substitute_variables` does a simple `result.replace(&placeholder, &value_to_string(value))` for each variable. For each variable it scans and rewrites the entire body, so the complexity is `O(n * m)` where `n` is the number of variables and `m` is the body length. More importantly, `replace` replaces every occurrence (including in the body of another variable — e.g. `{user}` inside a value destined for `{name}` causes re-substitution if iterated in the wrong order), and the replacement of `{score}` inside the text "your score is {score}" produces "your score is 95" — but if the value itself contains `{score}` (e.g. `TemplateValue::Text("{score}")`), the output becomes "your score is 95" then "your score is 95" → silent double-substitution. The function in `services.rs:277-307` (`TemplateService::substitute_variables`) handles this correctly with a single-pass scanner; the email.rs copy does not.

**Expected:**

Spec § "Templates": variable substitution must be deterministic.

**Evidence:**

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

### FINDING 41 (id: `ADAPTER-NOT-041`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/sms.rs:236-246` (SmsProvider::render_template)

**Description:**

`SmsProvider::render_template` uses the `{{name}}` placeholder syntax (double braces), while `EmailProvider::substitute_variables` uses `{name}` (single braces), while `TemplateService::substitute_variables` (services.rs:278) uses `{name}`. There is no single source of truth for the variable syntax. A template authored against `TemplateService` semantics (single brace) sent via SMS would not be substituted.

**Expected:**

Spec § "Templates": templates are stored in the communication domain; both adapters should use the same substitution engine.

**Evidence:**

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

### FINDING 42 (id: `ADAPTER-NOT-042`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/services.rs:417-420` (ChannelService::fan_out_targets)

**Description:**

`ChannelService::fan_out_targets` is documented as computing "the per-channel fan-out list for a single-channel request" and exists "so a future 'multi-channel request' feature can reuse the same helper without changing adapter call sites." The current implementation returns `vec![channel.clone()]` for every input — the comment at services.rs:413-416 says exactly this. The helper is a placeholder that adds no value today and exists only to be asserted against by tests (services.rs:610-653 doesn't even test this method).

**Expected:**

Either implement multi-channel fan-out (which the port spec § "Channel" says is possible: "A single notification can target multiple channels. The consumer adapter may fan out internally.") or remove the placeholder.

**Evidence:**

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

### FINDING 43 (id: `ADAPTER-NOT-043`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/services.rs:454-466` (IdempotencyService::derive_key) and `docs/ports/notifications.md:163-166`

**Description:**

`IdempotencyService::derive_key` derives the key from `(command_id: &str, recipient: &str, template_version: u32)` where `recipient` is a `&str` (a free-form string like "alice@example.test" per the unit test). The spec § "Idempotency" says the engine "generates a deterministic key from `(command_id, recipient, template_version)`" — `recipient` here is the structured `Recipient` enum, not a string. The port's `Recipient` carries variant information (e.g. `Student(id)`, `Guardian(id, role)`); flattening to a string loses the role and the recipient-kind. Two different `Recipient::Guardian` values (Primary vs Secondary) for the same student would collide.

**Expected:**

Spec § "Idempotency": "engine generates a deterministic key from `(command_id, recipient, template_version)`" — where `recipient` is the typed enum.

**Evidence:**

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

### FINDING 44 (id: `ADAPTER-NOT-044`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/services.rs:530-551` (RateLimitService::try_acquire)

**Description:**

`RateLimitService::try_acquire` computes the elapsed refill as `elapsed_ms / 1000`, discarding the sub-second remainder (services.rs:531-533). The module comment at services.rs:48-53 calls this out as deliberate ("sub-second carry-over is discarded (a 500ms pause refills zero tokens)"). The port spec § "Rate Limiting" example says "100 SMS/second" — a literal integer rate. Discarding sub-second refills means the effective rate is `floor(elapsed / 1000) * max_per_second`, which is not a 100/sec refill but a bursty batchy one. The behavior is wrong for sub-second throttling scenarios (Twilio's Messaging Services throttle, FCM's per-second caps).

**Expected:**

Spec § "Rate Limiting": "e.g. 100 SMS/second" — a continuous rate, not a batched-second rate.

**Evidence:**

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

### FINDING 45 (id: `ADAPTER-NOT-045`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/services.rs:218-228` (channel_key Push variant)

**Description:**

`channel_key` for `Channel::Push` keys the rate-limit bucket on `(topic, ttl_ms, collapse_key)`. A single tenant sending the same push to multiple topic variants (e.g. one for sports, one for academics) gets N independent buckets — but two tenants sending to the same topic also share a bucket (the topic string is the key, no tenant scoping). The spec § "Rate Limiting" requires "per-tenant, per-channel rate limits" — the bucket is keyed only on the channel, not on the tenant.

**Expected:**

Spec § "Rate Limiting": "per-tenant, per-channel rate limits ... configurable per tenant."

**Evidence:**

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

### FINDING 46 (id: `ADAPTER-NOT-046`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/port.rs:1015-1018` (Priority::as_str) and `crates/adapters/notify/src/services.rs:520` (try_acquire)

**Description:**

`Priority` (port.rs:994-1006) defines `Critical` as a distinct priority that the spec says "bypass queues and are delivered synchronously" and "may charge a premium." Neither `EmailProvider`, `SmsProvider`, `RateLimitService`, nor any helper treats `Critical` differently from `Normal`. The value is read from the request, never inspected, and silently downgraded.

**Expected:**

Spec § "Priority": "`Critical` notifications bypass queues and are delivered synchronously. The adapter may charge a premium for `Critical`."

**Evidence:**

```rust
  // crates/adapters/notify/src/email.rs:88-140
  // no match on request.priority; the field is read once via
  // SendNotification::priority but never branched on.
  // crates/adapters/notify/src/sms.rs:347-356
  // same — no match on request.priority.
  ```

---

### FINDING 47 (id: `ADAPTER-NOT-047`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/services.rs:30-32` and `crates/adapters/notify/src/services.rs:445-478` (IdempotencyService)

**Description:**

The `IdempotencyService` is documented in the module-level docstring (services.rs:30-32) and exposed as a service helper, but `IdempotencyService::is_duplicate` requires the caller to pass a `&mut HashSet<String>` (services.rs:470-477). The service "holds no state of its own" (services.rs:441-443). Real adapters therefore have to manage their own `HashMap<SchoolId, HashSet<String>>` and pass the inner set on every call — the helper does no encapsulation. The same shape would be required to wire the service into `EmailProvider` / `SmsProvider`, but neither does.

**Expected:**

A self-contained `IdempotencyService` that holds the per-tenant set internally and exposes `check_or_insert(school_id, key)`.

**Evidence:**

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

### FINDING 48 (id: `ADAPTER-NOT-048`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/sms.rs:50-52` (DEFAULT_GATEWAY_URL)

**Description:**

`DEFAULT_GATEWAY_URL` is a Twilio Messages URL with the literal placeholder `{account}` substituted from `api_key` (sms.rs:51-52). For `api_key = "AC0123456789abcdef"` the URL becomes `https://api.twilio.com/2010-04-01/Accounts/AC0123456789abcdef/Messages.json` (asserted at sms.rs:555-558). There is no validation that `api_key` is a Twilio-shaped `AC` prefix; any string becomes part of the URL path, opening a credential-shaped injection (e.g. an `api_key` containing `/` or `?` rewrites the path or query).

**Expected:**

Validate the API key shape before URL interpolation, or URL-encode the path segment.

**Evidence:**

```rust
  // crates/adapters/notify/src/sms.rs:174-177
  pub fn build(self) -> SmsProvider {
      let gateway_url = self.gateway_url
          .unwrap_or_else(|| DEFAULT_GATEWAY_URL.replace("{account}", &self.api_key));
  ```
  No validation; `&self.api_key` is interpolated verbatim.

---

### FINDING 49 (id: `ADAPTER-NOT-049`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/sms.rs:178` (Client::builder().build().unwrap_or_else(|_| Client::new()))

**Description:**

`SmsProviderBuilder::build` constructs the reqwest `Client` via `Client::builder().build().unwrap_or_else(|_| Client::new())`. `reqwest::Client::new()` is deprecated in modern reqwest (it ignores proxy config, TLS, etc.). The `unwrap_or_else` pattern silently masks the builder failure. A consumer expecting a TLS-configured client receives an unconfigured one with no diagnostic.

**Expected:**

Either propagate the builder error as a `NotificationError`, or document the un-configured fallback explicitly.

**Evidence:**

```rust
  // crates/adapters/notify/src/sms.rs:178
  let http = Client::builder().build().unwrap_or_else(|_| Client::new());
  ```

---

### FINDING 50 (id: `ADAPTER-NOT-050`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/email.rs:74-78` (EmailProvider struct) and `crates/adapters/notify/src/email.rs:204-209` (EmailProviderBuilder struct)

**Description:**

Neither `EmailProvider` nor `EmailProviderBuilder` carries a reference to `tenant.school_id`; the builder does not accept tenant-specific configuration. The transport's host, port, credentials, and default_from are builder-time singletons. Multi-tenant deployments that need different SMTP accounts per school (e.g. for branded sender domains) must construct a separate provider per school, defeating the `Arc<dyn NotificationProvider>` engine-storage pattern.

**Expected:**

Spec § "Multi-tenancy": one provider serves many tenants.

**Evidence:**

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

### FINDING 51 (id: `ADAPTER-NOT-051`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/port.rs:464-498` (Url newtype)

**Description:**

`Url` is a `String`-backed newtype at port.rs:466 with no `parse()` / validation. Spec § "Channel::Webhook": `Webhook { url: Url, secret: Option<SecretString> }` — the adapter must actually POST to that URL. `EmailProvider` does not implement `Channel::Webhook` (missing impl), but `SmsProvider` also accepts any `Channel::Sms` URL and never validates the URL before passing it to reqwest. A malformed URL surfaces as a reqwest error rather than a typed `NotificationError::InvalidRecipient` or similar.

**Expected:**

Spec § "Channel": adapters validate URL shape.

**Evidence:**

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

### FINDING 52 (id: `ADAPTER-NOT-052`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/email.rs:107-110` (EmailProvider::send) and `crates/adapters/notify/src/sms.rs:208-213` (SmsProvider::recipient_phone)

**Description:**

Neither provider hashes recipient identifiers before logging or storing them on the receipt. The receipt `metadata: BTreeMap<String, String>` is empty (ADAPTER-NOT-021); `log_school` is captured and discarded (ADAPTER-NOT-012); no hash of the recipient address appears anywhere. Spec § "Audit": "PII (phone numbers, email addresses) is hashed before logging." The shipped providers cannot log a single identifiable audit row that complies with this rule.

**Expected:**

Spec § "Audit": recipient hash on every send log.

**Evidence:**

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

### FINDING 53 (id: `ADAPTER-NOT-053`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/sms.rs:208-228` (recipient_phone)

**Description:**

`recipient_phone` rejects `Recipient::User/Student/Guardian/Staff/Group` with `"recipient requires contact lookup; not supported by reference SmsProvider"`. The reference implementation is structurally incapable of sending SMS to any user, student, staff member, or group. This means the integration tests cannot exercise the happy path of "user receives an SMS" — they must construct `Recipient::Direct(ContactInfo::new().with_phone(...))` or trigger the failure path (sms.rs:640-694). The handoff at PHASE-15-HANDOFF.md:144-146 acknowledges only "template substitute, template validate, channel classification, idempotency key derivation, rate-limit bucket" — none of which is a provider test.

**Expected:**

Spec § "Recipient": all recipient variants dispatchable.

**Evidence:**

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

### FINDING 54 (id: `ADAPTER-NOT-054`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/email.rs:142-189` (EmailProvider::send_bulk)

**Description:**

`EmailProvider::send_bulk` re-uses `EmailProvider::send` for each row, which means each row triggers a full `render_template_body` + `build_lettre_message` + SMTP send. The per-row work is identical except for the recipient and the variables — there is no caching of the rendered template subject or the constant headers, no MIME reuse, no `MAIL FROM` reuse. Per the spec § "Bulk Send", a batched send should reuse the message template and only vary the recipient.

**Expected:**

Spec § "Bulk Send": batched send.

**Evidence:**

```rust
  // crates/adapters/notify/src/email.rs:159-186
  for (idx, row) in request.recipients.iter().enumerate() {
      ...
      let single = SendNotification { ... };
      match self.send(single).await { ... }
  }
  ```

---

### FINDING 55 (id: `ADAPTER-NOT-055`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/email.rs:336-356` (substitute_variables)

**Description:**

`substitute_variables` in `email.rs` (lines 336-356) is `pub` but is also re-implemented in `services.rs:278-307` as `TemplateService::substitute_variables`. The two implementations diverge: `email.rs` takes `BTreeMap<String, TemplateValue>` and stringifies each value via `value_to_string` (lines 347-356); `services.rs` takes `BTreeMap<String, String>` and substitutes verbatim. Two source-of-truth substitution engines in one crate. The handoff's `PHASE-15-HANDOFF.md:131-134` says "TemplateService::substitute_variables + validate_required_variables + extract_variables" — but the email provider ships its own.

**Expected:**

One substitution helper, used by every channel adapter.

**Evidence:**

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

### FINDING 56 (id: `ADAPTER-NOT-056`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/sms.rs:50-52` and `crates/adapters/notify/src/sms.rs:174-186` (SmsProviderBuilder::build)

**Description:**

The Twilio endpoint URL is hardcoded at sms.rs:51-52. There is no support for Twilio's `MessagingServiceSid` (used for sending from a pool of numbers) or for sending to a `from` that is a messaging-service sid. Spec § "Channel::Sms" allows `from: Option<PhoneNumber>`, but Twilio's messaging services accept a `MessagingServiceSid` instead of a `From:` phone number, which the reference provider cannot express.

**Expected:**

Spec § "Channel::Sms" / Twilio integration: support messaging services.

**Evidence:**

```rust
  // crates/adapters/notify/src/sms.rs:50-52
  const DEFAULT_GATEWAY_URL: &str =
      "https://api.twilio.com/2010-04-01/Accounts/{account}/Messages.json";
  // crates/adapters/notify/src/sms.rs:308-319 (dispatch posts a fixed form with To/From/Body)
  .form(&[("To", to.as_str()), ("From", from.as_str()), ("Body", rendered.as_str())])
  ```

---

### FINDING 57 (id: `ADAPTER-NOT-057`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/sms.rs:311` (HTTP POST without timeout)

**Description:**

`SmsProvider::dispatch` builds an HTTP POST with no timeout. `reqwest::Client::builder().build()` (sms.rs:178) constructs the client with the default 30-second timeout — but there is no request-level timeout and no retry policy. A hung gateway connection blocks a worker for up to 30 seconds per request, and `send_bulk` multiplies this by recipient count.

**Expected:**

Explicit `timeout(Duration::from_secs(...))` on the per-request builder.

**Evidence:**

```rust
  // crates/adapters/notify/src/sms.rs:308-319
  let response = self.http.post(&self.gateway_url)
      .header("Authorization", self.basic_auth_header())
      .form(&[("To", to.as_str()), ("From", from.as_str()), ("Body", rendered.as_str())])
      .send().await
      .map_err(NotificationError::infrastructure)?;
  ```

---

### FINDING 58 (id: `ADAPTER-NOT-058`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/email.rs:88-140` and `crates/adapters/notify/src/sms.rs:296-340`

**Description:**

Neither provider implements a `RateLimited` retry path. The `NotificationError::RateLimited` variant exists (`errors.rs:96`) but no adapter ever constructs it. Spec § "Rate Limiting": "The adapter returns `NotificationError::RateLimited` when a limit is hit; the engine retries with backoff." There is no `try_acquire` call in either provider; no error path returns `RateLimited`.

**Expected:**

Spec § "Rate Limiting".

**Evidence:**

```rust
  // grep -nE "RateLimited" crates/adapters/notify/src/email.rs
  // 0 matches (only the enum variant itself at errors.rs:96)
  // grep -nE "RateLimited" crates/adapters/notify/src/sms.rs
  // 0 matches
  ```

---

### FINDING 59 (id: `ADAPTER-NOT-059`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/sms.rs:308-333` (SmsProvider::dispatch)

**Description:**

`SmsProvider::dispatch` only recognizes HTTP 202 (Queued) and 2xx (Sent) as success. It treats every 4xx and 5xx as `NotificationError::provider`. The spec § "DeliveryStatus" has a `Failed { reason, retryable }` variant where `retryable` distinguishes transient 5xx and rate-limit responses from permanent 4xx. The provider cannot return `Failed { retryable: true }` for a 429 (Twilio throttle), nor `Failed { retryable: false }` for a 21610 (unsubscribed recipient).

**Expected:**

Spec § "DeliveryStatus": `Failed { retryable: bool }`.

**Evidence:**

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

### FINDING 60 (id: `ADAPTER-NOT-060`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/email.rs:367-395` (build_lettre_message)

**Description:**

`build_lettre_message` builds a `lettre::Message::builder().body(body.to_owned())`. The hardcoded subject is `"Educore notification"` (line 383). The template's subject (per spec § "Templates": "subject (for email and push)") is ignored — `TemplateRef` carries no subject field, and the email provider never queries the communication-domain template store. All sent emails have the same subject line.

**Expected:**

Spec § "Templates": "A subject (for email and push)."

**Evidence:**

```rust
  // crates/adapters/notify/src/email.rs:380-394
  let mut builder = Message::builder()
      .from(from_mailbox)
      .to(to_mailbox)
      .subject("Educore notification");
  ```

---

### FINDING 61 (id: `ADAPTER-NOT-061`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/services.rs:209-241` (channel_key) and `crates/adapters/notify/src/services.rs:520` (try_acquire)

**Description:**

`RateLimitService::try_acquire` keys the bucket by `channel_key(channel)`, which for `Channel::Email` includes `from` and `reply_to` (services.rs:211-215). Two sends with the same channel kind but different `from` addresses (e.g. sender A on row 1 and sender B on row 2) get separate buckets. Spec § "Rate Limiting": "per-channel rate limits" — keyed on the channel kind, not the per-request envelope.

**Expected:**

Spec § "Rate Limiting": per-channel, per-tenant limits keyed on channel kind.

**Evidence:**

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

### FINDING 62 (id: `ADAPTER-NOT-062`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/port.rs:508-557` (SecretString)

**Description:**

`SecretString` (port.rs:508-557) is defined in the port but never used by either provider. `EmailProvider` stores `credentials_user: Option<String>` as a plain `String` (email.rs:206) — the password half is `String::new()` (email.rs:273), and the credential is logged through the `Credentials::new` constructor. `SmsProvider` stores `api_key: String` (sms.rs:74) and only redacts it in `Debug` (sms.rs:86). Spec § "Webhook" / general hygiene: secrets should be wrapped at the port boundary.

**Expected:**

Adapter credential fields use `SecretString`.

**Evidence:**

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

### FINDING 63 (id: `ADAPTER-NOT-063`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/services.rs:519-551` (RateLimitService::try_acquire) and `crates/adapters/notify/src/services.rs:530-543` (refill math)

**Description:**

`RateLimitService::try_acquire` (services.rs:520) sets `entry.last_refill = entry.last_refill.checked_add(Duration::from_millis(u64::from(new_tokens) * 1000))` on refill (line 539-542). This advances `last_refill` by the integer-second count, not by the actual elapsed time. The next call then computes `elapsed = now - last_refill` — but since the bucket has been fully drained and refilled at the *expected* rate, this can leak tokens (the bucket effectively starts a new second-clock from the last refill point, so partial-second drift accumulates). With `max_per_second = 1` and a request every 1100 ms, the second request gets a token that should have been withheld.

**Expected:**

A standard token-bucket implementation tracks `last_refill` as the time the bucket was last touched and refills `(now - last_refill) * rate` tokens; the `last_refill` should always be set to `now`, not advanced.

**Evidence:**

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

### FINDING 64 (id: `ADAPTER-NOT-064`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/port.rs:1156-1192` (SendNotification)

**Description:**

`SendNotification` does not derive `Default`. The unit tests at `email.rs:482-499`, `sms.rs:586-601`, `sms.rs:614-631`, and `sms.rs:665-679` all hand-construct the struct with 12 named fields. `notify_integration.rs` (the integration test file) does not construct a `SendNotification` at all. The lack of `Default` is a known ergonomic failure for the integration-test scaffolding but is also missing from `SendBulkNotification` (port.rs:1236-1261). Bulk recipients are constructed via `BulkRecipient::new(recipient)` (port.rs:1219-1225) which is fine, but the wrapper struct is unbuildable from defaults.

**Expected:**

Either `Default` impl or a `SendNotification::builder(...)` builder pattern.

**Evidence:**

```rust
  // crates/adapters/notify/src/port.rs:1157
  #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  pub struct SendNotification {
  ```
  No `Default` derive, no `new` constructor.

---

### FINDING 65 (id: `ADAPTER-NOT-065`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/services.rs:73-176` (sha256) and `crates/adapters/notify/src/sms.rs:489-517` (base64_encode)

**Description:**

`services.rs` ships hand-rolled SHA-256 (services.rs:73-176) and `sms.rs` ships hand-rolled base64 (sms.rs:489-517). Both are documented as deviations from using standard crates because the task spec lists the manifest under "DO NOT TOUCH" (services.rs:31-41; sms.rs:21-25). The two crypto primitives are the foundation of idempotency-key derivation and HTTP auth — the highest-impact components of the port. A typo in either implementation silently produces wrong keys / wrong headers and is not detectable by any test in the crate (the integration test only checks the length and charset of the SHA-256 hex string at `notify_integration.rs:91-93`, not correctness).

**Expected:**

Use the `sha2` crate for SHA-256 and the `base64` crate for base64.

**Evidence:**

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

### FINDING 66 (id: `ADAPTER-NOT-066`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/sms.rs:158-167` (SmsProviderBuilder::template_body)

**Description:**

`SmsProviderBuilder::template_body` registers a template body in a process-local `HashMap<NotificationTemplateId, String>` (sms.rs:74-77, 109). Spec § "Templates": "Templates are owned by the communication domain." Production deployments cannot pre-register templates from the engine into a builder; the template store is in the database. The builder-as-template-store is a workaround that is only useful for tests.

**Expected:**

Templates come from the communication domain; adapters resolve `TemplateRef::Id` against a shared template service.

**Evidence:**

```rust
  // crates/adapters/notify/src/sms.rs:158-167
  pub fn template_body(mut self, id: NotificationTemplateId, body: impl Into<String>) -> Self {
      self.templates.insert(id, body.into());
      self
  }
  ```

---

### FINDING 67 (id: `ADAPTER-NOT-067`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/email.rs:380-394` (build_lettre_message)

**Description:**

`build_lettre_message` constructs a `lettre::Message::builder().from(from_mailbox).to(to_mailbox).subject(...)`. It does not set a `Message-ID` header, does not set `Date`, does not set `MIME-Version`, and does not set `Content-Type` explicitly. The resulting email may be malformed or rejected by strict receiving MTAs. `lettre` is supposed to set `Date` automatically, but only for fully-built messages, and the `body()` call with a plain `String` produces `Content-Type: text/plain; charset=utf-8` — not the spec's "email body" which may need HTML.

**Expected:**

Spec § "Channel::Email" supports both plain and HTML bodies; the adapter should set `Content-Type` accordingly.

**Evidence:**

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

### FINDING 68 (id: `ADAPTER-NOT-068`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/sms.rs:174-186` (SmsProviderBuilder::build)

**Description:**

`SmsProviderBuilder::build` returns `SmsProvider` (no `Result`), while `EmailProviderBuilder::build` returns `Result<EmailProvider>` (email.rs:261). The asymmetry is unjustified: `SmsProviderBuilder::build` should validate the `api_key` is set (it's allowed to be empty at construction) and the `gateway_url` is well-formed. As shipped, `SmsProviderBuilder::new()` constructs an empty-string `api_key`, then `.build()` returns a `SmsProvider` whose every `send` will fail with a `401 Unauthorized` at the gateway. A consumer cannot detect this at startup.

**Expected:**

Consistent `Result<T, NotificationError>` builder.

**Evidence:**

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

### FINDING 69 (id: `ADAPTER-NOT-069`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/port.rs:151-173` (BulkRecipientIndex) and `crates/adapters/notify/src/sms.rs:378`

**Description:**

`BulkRecipientIndex` is a transparent `u32` newtype. The spec uses it for "the original input row index." In `SmsProvider::send_bulk` the index is computed from `receipt.total()` (sms.rs:377-378) which is wrong (ADAPTER-NOT-016); in `EmailProvider::send_bulk` it's computed from `enumerate()` (email.rs:180) which is correct. The mismatch between the two providers means the same bulk request gets different indices depending on which provider handles it.

**Expected:**

Spec § "BulkReceipt": same `BulkRecipientIndex` semantics for every provider.

**Evidence:**

```rust
  // crates/adapters/notify/src/email.rs:180
  let Ok(idx_u32) = u32::try_from(idx) else { continue; };
  receipt.failed.push((BulkRecipientIndex::new(idx_u32), e));
  // crates/adapters/notify/src/sms.rs:377-378
  let global_idx = receipt.total();
  let index = BulkRecipientIndex::new(u32::try_from(global_idx).unwrap_or(u32::MAX));
  ```

---

### FINDING 70 (id: `ADAPTER-NOT-070`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/email.rs:99-100` (Channel::Sms/Email mismatch error)

**Description:**

The error string in `EmailProvider::send` for non-email channels is `"email provider cannot send {other:?} channel"`. This embeds the entire `Channel` enum's `Debug` output, which for `Channel::Webhook { url, secret }` would include the webhook URL. The `SecretString` already redacts itself (port.rs:535-545), but `Url` is a plain `String`. PII / secret-bearing variants dump their state into the error message.

**Expected:**

Error messages should never carry PII.

**Evidence:**

```rust
  // crates/adapters/notify/src/email.rs:97-100
  return Err(NotificationError::provider(format!(
      "email provider cannot send {other:?} channel"
  )));
  ```

---

### FINDING 71 (id: `ADAPTER-NOT-071`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/email.rs:103-105` (EmailProvider reply_to match)

**Description:**

`EmailProvider::send` matches `Channel::Email { reply_to, .. }` separately after the `from` match (line 102-105) but does not check that the reply-to variant is actually being processed; if `request.channel` is `Channel::Sms { .. }`, the function still enters the `reply_to` block (it returns `None` because of the `_ => None` arm). The two-match-arm structure is unnecessary and produces an unreachable pattern in the non-email case. Cosmetic but indicates a rushed implementation.

**Expected:**

Single destructuring of `Channel::Email { from, reply_to }`.

**Evidence:**

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

### FINDING 72 (id: `ADAPTER-NOT-072`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/services.rs:277-307` (TemplateService::substitute_variables)

**Description:**

`TemplateService::substitute_variables` (services.rs:277-307) silently leaves placeholders whose variables are missing in the variables map. The docstring says (services.rs:271-276) "Placeholders whose name is not present in `variables` are left as-is (the caller is expected to have run `validate_required_variables` first)." If a caller forgets `validate_required_variables`, the placeholder leaks into the rendered body — potentially exposing the internal `{student_name}` syntax to end users. The SMS provider's `SmsProvider::render_template` (sms.rs:236-246) has the same issue with a different placeholder syntax (`{{name}}`).

**Expected:**

Spec § "Templates": "The engine validates that all required variables are provided in `SendNotification::variables`. Missing variables fail the send."

**Evidence:**

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

### FINDING 73 (id: `ADAPTER-NOT-073`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** `crates/adapters/notify/src/services.rs:209-241` (channel_key) and `crates/adapters/notify/src/services.rs:520-551` (try_acquire)

**Description:**

`channel_key` for `Channel::Webhook` includes `url.as_str()` in the bucket key (services.rs:236-239). Two webhook deliveries to different URLs get separate buckets; one URL gets `max_per_second` regardless of how many URLs the tenant has configured. The spec § "Rate Limiting" requires per-channel limits; the webhook is a single channel.

**Expected:**

Spec § "Rate Limiting": per-channel, per-tenant limit (single bucket per channel kind per tenant).

**Evidence:**

```rust
  // crates/adapters/notify/src/services.rs:236-239
  Channel::Webhook { url, secret } => {
      let signed = if secret.is_some() { "1" } else { "0" };
      format!("webhook:{signed}:{}", url.as_str())
  }
  ```

---

### FINDING 74 (id: `ADAPTER-NOT-074`)

- **Source:** `docs/audit_reports/findings/wave3-notify.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** `crates/adapters/notify/Cargo.toml:21-32` and `crates/adapters/notify/src/lib.rs:1-60`

**Description:**

The `Cargo.toml` carries a long comment block (lines 23-31) explaining that the previous commit didn't update dependencies, that `serde` and `thiserror` were added by a "B.3b" deviation, and that the consumer of these deps was the SMS reference implementation. This dev-level commentary in the manifest is unusual; the same dependencies are pulled in by `port.rs` and `errors.rs` for `Serialize`/`Deserialize` and `thiserror::Error` derives. The comment also references an internal microtask nomenclature ("B.3b", "the port+types owner") that doesn't appear in the rest of the repository. This is a code-hygiene drift: the manifest should describe the crate, not narrate the development history.

**Expected:**

Clean Cargo.toml without in-manifest development narrative.

**Evidence:**

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


## Payment (target id prefix: `ADAPT-PAY`)

**Path:** `crates/adapters/payment/`  
**Total findings:** 24 (7 critical, 11 high, 5 medium, 1 low)


### FINDING 1 (id: `ADAPT-PAY-001`)

- **Source:** `docs/audit_reports/findings/wave3-payment.md`
- **Severity:** Critical
- **Area:** adapters-payment
- **Location:** `crates/adapters/payment/src/port.rs:521-527` (`PaymentMethod::Wallet`)

**Description:**

The port replaces the spec's
  `secrecy::SecretString` wallet PIN with a plain `pub pin: String`.
  Every adapter that materialises the value (log, audit, error
  message, persistence) sees a raw PIN. The crate's own deviation
  note at `lib.rs:18-26` documents this and admits "the adapter
  MUST redact any string whose field name ends in `_pin`,
  `_secret`, or `_card` before the value reaches the audit log" —
  but no code path enforces this. `StripeProvider::charge` rejects
  `Wallet` outright, so the field never reaches a `String` log
  line in the shipped adapter, but a future wallet adapter (or
  any logging middleware) would write the PIN to stdout/log files.
  PCI-DSS scope is unaffected (wallet PIN ≠ PAN), but the design
  is a textbook secret-handling anti-pattern.

**Expected:**

`docs/ports/payments.md:65-71` — `Wallet { wallet_id: WalletId, pin: SecretString }`.
  `docs/code-standards.md` § "Code Standards" — "use `secrecy`
  for secrets".

**Evidence:**

```rust
  // crates/adapters/payment/src/port.rs:521-527
  /// A wallet payment (school issued or third party). The PIN
  /// is captured at the consumer's frontend and forwarded only
  /// to the wallet adapter; the engine stores it transiently.
  Wallet {
      /// The wallet identifier.
      wallet_id: WalletId,
      /// The wallet's PIN (already redacted in the audit log).
      pin: String,
  },
  ```
  No `secrecy::SecretString` exists in the crate; `pin` is `String`
  end-to-end with a comment-only redaction contract.

---

### FINDING 2 (id: `ADAPT-PAY-002`)

- **Source:** `docs/audit_reports/findings/wave3-payment.md`
- **Severity:** Critical
- **Area:** adapters-payment
- **Location:** `crates/adapters/payment/src/stripe.rs:357-440`
  (`PaymentProvider for StripeProvider` — `charge`, `refund`,
  `settlement`) and `crates/adapters/payment/src/lib.rs:21-26`

**Description:**

The crate ships only one reference impl
  (`StripeProvider`) and that single impl returns
  `PaymentError::Provider(...)` for **5 of the 7 `PaymentMethod`
  variants** (`Cash`, `Cheque`, `BankTransfer`, `Wallet`,
  `ManualAdjustment`) plus `Gateway`. The port spec
  (`docs/ports/payments.md` § "Offline Mode") requires adapters
  for `Cash`, `Cheque`, `BankSlip`, and `Wallet`. Without an
  offline cash-book adapter the engine cannot accept any payment
  in offline mode (the documented default for schools without a
  card-processing merchant account) and cannot record any cash /
  cheque / manual adjustment at the school office. The handoff
  (PHASE-15-HANDOFF.md:103-113) confirms "1 reference impl"
  without documenting the missing offline coverage.

**Expected:**

`docs/ports/payments.md` § "Offline Mode": "In
  offline mode, the consumer uses the `Cash`, `Cheque`,
  `BankSlip`, or `Wallet` methods. Online gateway methods are
  unavailable."

**Evidence:**

```rust
  // crates/adapters/payment/src/stripe.rs:369-410 (charge body)
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
  PaymentMethod::BankTransfer { .. } => { ... }
  PaymentMethod::Wallet { .. } => { ... }
  PaymentMethod::ManualAdjustment { .. } => { ... }
  PaymentMethod::Gateway { gateway, .. } => {
      return Err(PaymentError::Provider(format!(
          "gateway flow not supported in v1 (gateway={})",
          gateway.as_str()
      )));
  }
  ```

---

### FINDING 3 (id: `ADAPT-PAY-003`)

- **Source:** `docs/audit_reports/findings/wave3-payment.md`
- **Severity:** Critical
- **Area:** adapters-payment
- **Location:** `crates/adapters/payment/src/stripe.rs:444-450`
  (`settlement`) and `crates/adapters/payment/src/port.rs:1099-1108`
  (`settlement` trait method)

**Description:**

`StripeProvider::settlement` unconditionally
  returns `PaymentError::Provider("settlement is not implemented
  in the Stripe reference adapter; use a dedicated payouts
  adapter")` and no dedicated payouts adapter ships. The port
  spec defines `settlement` as the engine's mechanism for matching
  captured payments against actual bank deposits (the "engine
  consumes settlement events" flow) and the port's
  `PaymentStatus` enum includes `Captured` (not `Settled`) — so
  without settlement the engine can never confirm a charge has
  actually landed in the school's bank account. Reconciliation
  against the bank ledger is the single most important
  accounting control in any payment system; without it the
  engine reports "captured" for charges that may have been
  reversed, refunded-out-of-band, or stuck in a gateway dispute.

**Expected:**

`docs/ports/payments.md` § "Settlement &
  Reconciliation" — "`settlement` returns a `Settlement` batch of
  captured payments that have settled into the school's bank
  account." § "Testing": "A test of settlement matching."

**Evidence:**

```rust
  // crates/adapters/payment/src/stripe.rs:444-450
  async fn settlement(&self, _request: SettlementRequest) -> Result<Settlement, PaymentError> {
      Err(PaymentError::Provider(
          "settlement is not implemented in the Stripe reference adapter; \
           use a dedicated payouts adapter".into(),
      ))
  }
  ```

---

### FINDING 4 (id: `ADAPT-PAY-004`)

- **Source:** `docs/audit_reports/findings/wave3-payment.md`
- **Severity:** Critical
- **Area:** adapters-payment
- **Location:** `crates/adapters/payment/src/stripe.rs:155-205`
  (`verify_webhook_signature`) and `crates/adapters/payment/src/stripe.rs:798-812`
  (`parse_stripe_signature`)

**Description:**

The webhook signature verifier parses the
  `t=<unix>,v1=<hex>` header and computes the HMAC correctly, but
  **never checks the timestamp tolerance**. Stripe's docs
  recommend a 5-minute (300-second) tolerance window to defend
  against replay; the implementation accepts any signature
  regardless of age. An attacker who captures a webhook payload
  (e.g. via a compromised log archive, a leaked proxy history,
  or a man-in-the-middle that only intercepts the response) can
  replay it indefinitely against the engine's webhook endpoint.
  The handoff does not mention replay protection; the spec
  implies it via "The engine correlates the webhook to the
  originating command via `idempotency_key`" but does not pin
  the tolerance. The cost of adding `now - t < 300` is three
  lines of code.

**Expected:**

`docs/ports/payments.md` § "Webhook Flow" and
  `docs/ports/integrations.md` (referenced for signature format).
  Stripe's own docs: "Stripe-Signature tolerance is 300 seconds
  by default."

**Evidence:**

```rust
  // crates/adapters/payment/src/stripe.rs:155-205
  pub fn verify_webhook_signature(
      &self,
      payload: &[u8],
      signature: &str,
  ) -> Result<(), PaymentError> {
      ...
      let (timestamp, expected_v1) = parse_stripe_signature(signature)?;
      let payload_str = std::str::from_utf8(payload).map_err(...)?;
      let signed = format!("{timestamp}.{payload_str}");
      let mut mac = HmacSha256::new_from_slice(...)?;
      mac.update(signed.as_bytes());
      let computed = mac.finalize().into_bytes();
      if constant_time_eq_hex(&computed, expected_v1) {
          Ok(())
      } else {
          Err(PaymentError::Provider("webhook signature mismatch".into()))
      }
  }
  ```
  No `now.duration_since(...)` check; `timestamp` is discarded
  after being folded into `signed`.

---

### FINDING 5 (id: `ADAPT-PAY-005`)

- **Source:** `docs/audit_reports/findings/wave3-payment.md`
- **Severity:** Critical
- **Area:** adapters-payment
- **Location:** `crates/adapters/payment/src/stripe.rs:412-418`
  (`refund` — params construction) and `crates/adapters/payment/src/stripe.rs:101-111`
  (module doc § "Refund lookup")

**Description:**

`StripeProvider::refund` assumes
  `request.original_payment_id.as_str()` already holds the Stripe
  charge id (`ch_...`), passing it directly as the `charge=`
  form field. The engine's `PaymentId` is opaque (per
  `port.rs:63-90`) and a production deployment that mints its own
  payment ids will pass those engine ids to the refund call,
  producing a `400` from Stripe ("No such charge: `pay_abc...`").
  The crate's own deviation note at `stripe.rs:101-111` flags
  this: "A production adapter would translate via the receipt
  store; this reference impl assumes the engine's
  `PaymentId.as_str()` already holds the Stripe charge id." With
  no receipt store shipping and no translation function, the
  adapter cannot round-trip a refund end-to-end. The
  `PHASE-15-HANDOFF.md:117-121` integration test
  `payment_integration_async_stripe_refund_mock` is `#[ignore]`d
  and only verifies the provider builds, not that a refund
  succeeds.

**Expected:**

`docs/ports/payments.md` § "Refund": "The
  adapter is responsible for the actual money movement."

**Evidence:**

```rust
  // crates/adapters/payment/src/stripe.rs:412-418
  async fn refund(&self, request: RefundRequest) -> Result<RefundReceipt, PaymentError> {
      let mut params: Vec<(String, String)> = Vec::with_capacity(3);
      params.push((
          "charge".to_owned(),
          request.original_payment_id.as_str().to_owned(),
      ));
      ...
  ```

---

### FINDING 6 (id: `ADAPT-PAY-006`)

- **Source:** `docs/audit_reports/findings/wave3-payment.md`
- **Severity:** Critical
- **Area:** adapters-payment
- **Location:** `crates/adapters/payment/src/stripe.rs:388-411`
  (`charge` for `Card { save: true }`) and `crates/adapters/payment/src/stripe.rs:117-121`
  (module doc § "`save` flag")

**Description:**

When a `PaymentMethod::Card { save: true }` is
  passed, the adapter "passes it through verbatim" (per the
  module doc at `stripe.rs:117-121`). Stripe's Charges API has no
  `save` parameter — vaulting requires the SetupIntent +
  Customer flow with the PaymentMethod API. The `save: true` flag
  is silently ignored: the card is charged but never saved, and
  no error is returned. A consumer wiring up recurring fee
  collection will pass `save: true`, observe the one-off charge
  succeed, then have recurring charges fail at the next billing
  cycle when there is no PaymentMethod on file. The deviation
  note acknowledges this but the impl has no path to inform the
  caller.

**Expected:**

`docs/ports/payments.md` § "PaymentMethod":
  `Card { token: CardToken, save: bool }` — "save requests that
  the gateway store the card for future recurring charges."

**Evidence:**

```rust
  // crates/adapters/payment/src/stripe.rs:362-388 (Card branch)
  PaymentMethod::Card { token, .. } => {
      let mut p = Vec::with_capacity(6 + request.metadata.len());
      p.push(("amount".to_owned(), request.amount.amount_minor.to_string()));
      p.push(("currency".to_owned(), request.amount.currency.as_str().to_ascii_lowercase()));
      p.push(("source".to_owned(), token.as_str().to_owned()));
      ...
      // No `save` handling — `..` discards the `save` field.
  }
  ```
  The `..` pattern at `token, ..` discards `save`; no SetupIntent
  path exists.

---

### FINDING 7 (id: `ADAPT-PAY-007`)

- **Source:** `docs/audit_reports/findings/wave3-payment.md`
- **Severity:** Critical
- **Area:** adapters-payment
- **Location:** `crates/adapters/payment/src/stripe.rs:345-411`
  (`charge` body), `crates/adapters/payment/src/stripe.rs:768-796`
  (`stripe_error_to_payment_error`)

**Description:**

The port spec defines
  `PaymentError::ThreeDSRequired` as the contract for handling
  3-D Secure challenges: the adapter returns it on the initial
  call so the engine can redirect the customer to the issuer's
  3DS challenge. The shipped adapter never returns this variant
  — instead, a 3DS-required Stripe response flows through
  `stripe_error_to_payment_error` (lines 768-796) as a generic
  `PaymentError::CardError` (`card_error` type with a
  `authentication_required` decline code falls into the default
  `card_error` arm and surfaces as `Declined("authentication
  required (code=...)")`). The consumer cannot distinguish a
  3DS challenge from a hard decline and has no signal to initiate
  the issuer's challenge flow.

**Expected:**

`docs/ports/payments.md` § "PaymentError":
  `ThreeDSRequired` — "The gateway requires 3-D Secure
  authentication before the charge can proceed."

**Evidence:**

```rust
  // crates/adapters/payment/src/stripe.rs:768-796
  match (status, err_type) {
      (_, "rate_limit_error") => PaymentError::RateLimited,
      (_, "card_error") if decline_code == Some("insufficient_funds") => {
          PaymentError::InsufficientFunds
      }
      (_, "card_error") => PaymentError::Declined(format!(
          "{}{}",
          message,
          code.map(|c| format!(" (code={c})")).unwrap_or_default()
      )),
      ...
  }
  ```
  No match arm inspects `decline_code == Some("authentication_required")`
  or Stripe's `payment_intent.next_action.use_stripe_sdk` payload.

---

### FINDING 10 (id: `ADAPT-PAY-010`)

- **Source:** `docs/audit_reports/findings/wave3-payment.md`
- **Severity:** High
- **Area:** adapters-payment
- **Location:** `crates/adapters/payment/src/services.rs:281-330`
  (`BankSlipService::validate_slip_number`,
  `BankSlipService::generate_slip_id`) and
  `docs/handoff/PHASE-15-HANDOFF.md:120-122`

**Description:**

`BankSlipService::validate_slip_number` only
  enforces length (6-20 chars) and ASCII-alphanumeric
  (services.rs:282-298). The handoff (PHASE-15-HANDOFF.md:120-122)
  asserts "mod-11 check" is performed. Brazilian "boleto" slip
  numbers (the documented target per `services.rs:215`) carry a
  mod-11 check digit; without the check, a typo in a slip number
  silently matches an invoice and a parent / accountant can
  approve a slip that does not exist in the bank's ledger. The
  integration test at `tests/payment_integration.rs:65-77`
  asserts only length and alphanumeric (`"AB-123"` rejected for
  the dash) but no checksum.

**Expected:**

PHASE-15-HANDOFF.md:120-122 — "mod-11 check";
  `services.rs:215` — "Brazilian-style 'boleto' bank-slip
  inputs".

**Evidence:**

```rust
  // crates/adapters/payment/src/services.rs:282-298
  pub fn validate_slip_number(number: &str) -> Result<(), PaymentError> {
      if number.len() < 6 || number.len() > 20 {
          return Err(PaymentError::InvalidAmount(format!(
              "slip number must be 6-20 chars, got {} chars",
              number.len()
          )));
      }
      if !number.chars().all(|c| c.is_ascii_alphanumeric()) {
          return Err(PaymentError::InvalidAmount(format!(
              "slip number must be ASCII alphanumeric, got {number:?}"
          )));
      }
      Ok(())
  }
  ```
  No checksum digit verification; `"ABC123"` and `"ABC124"`
  (typo) both pass.

---

### FINDING 11 (id: `ADAPT-PAY-011`)

- **Source:** `docs/audit_reports/findings/wave3-payment.md`
- **Severity:** High
- **Area:** adapters-payment
- **Location:** `crates/adapters/payment/src/services.rs:307-321`
  (`BankSlipService::generate_slip_id`) and
  `crates/adapters/payment/src/services.rs:209-210`
  (the static `SLIP_COUNTER`)

**Description:**

`BankSlipService::generate_slip_id` mints ids
  from a process-local `AtomicU64` counter (`SLIP-00000001`,
  `SLIP-00000002`, …). After a process restart the counter resets
  to 0, so two distinct physical slips generated by two
  consecutive process runs can collide on `SLIP-00000001`. The
  module-level doc at `services.rs:51-54` documents this: "The
  id is unique within a process; it is not a global UUID." The
  spec (`docs/ports/payments.md` § "Bank Slip Flow") treats slip
  ids as durable identifiers that are stored in the file storage
  port and referenced across approval and reconciliation. With
  counter-reset collisions the engine can mis-attribute a slip
  photo to a previous slip. The deviation note acknowledges the
  limitation but ships it as the only id-minting path; no
  `uuid::Uuid::new_v4()` fallback or startup-rng seed is
  available.

**Expected:**

`docs/ports/payments.md` § "Bank Slip Flow" —
  slip id is durable and stable across process restarts.

**Evidence:**

```rust
  // crates/adapters/payment/src/services.rs:209-210, 320-322
  static SLIP_COUNTER: AtomicU64 = AtomicU64::new(0);
  ...
  #[must_use]
  pub fn generate_slip_id() -> String {
      let n = SLIP_COUNTER.fetch_add(1, Ordering::SeqCst);
      format!("SLIP-{n:08}")
  }
  ```
  No `AtomicU64::compare_exchange` against a persisted offset;
  no UUID; counter starts at 0 on every process boot.

---

### FINDING 12 (id: `ADAPT-PAY-012`)

- **Source:** `docs/audit_reports/findings/wave3-payment.md`
- **Severity:** High
- **Area:** adapters-payment
- **Location:** `crates/adapters/payment/src/services.rs:355-361`
  (`SettlementService::compute_net_settlement`) and
  `crates/adapters/payment/src/services.rs:362-365` (docstring
  § "SettlementService")

**Description:**

`compute_net_settlement` uses `sum` rather
  than `saturating_add` over `i64` minor units. A settlement
  batch with enough high-value lines (or a stuck test fixture
  with `i64::MAX` per line) silently wraps to a negative number.
  The docstring at `services.rs:362-365` acknowledges this: "a
  settlement batch that genuinely overflows `i64` is a
  programming error and the wrap is a loud signal." In a
  production payment system, a wrapped total net means the
  reconciliation engine records a *negative* batch deposit —
  triggering a refund flow on what should have been a positive
  settlement. The spec (`docs/ports/payments.md` § "Settlement")
  requires the engine to consume settlement events and emit
  `PaymentSettled`; a wrapped total would silently propagate
  through events and audit log.

**Expected:**

`docs/code-standards.md` § "Numeric
  conversions": "Numeric conversions use `TryFrom`/`TryInto`;
  `as` on numerics is forbidden." Per the same standards, a
  saturating-or-checked sum is the only acceptable aggregation.

**Evidence:**

```rust
  // crates/adapters/payment/src/services.rs:355-361
  #[must_use]
  pub fn compute_net_settlement(lines: &[SettlementLine]) -> i64 {
      lines.iter().map(|l| l.net.amount_minor).sum()
  }
  ```

---

### FINDING 13 (id: `ADAPT-PAY-013`)

- **Source:** `docs/audit_reports/findings/wave3-payment.md`
- **Severity:** High
- **Area:** adapters-payment
- **Location:** `crates/adapters/payment/src/stripe.rs:724-744`
  (`map_charge_status` — `partially_refunded`, `authorized`,
  `disputed`, `refunded` arms)

**Description:**

`map_charge_status` synthesises four
  `PaymentStatus` variants with **empty / wrong / fabricated**
  inner data when Stripe reports them. Specifically:
  - `partially_refunded` (stripe.rs:733-738) hard-codes
    `remaining: Money::zero(amount.currency.clone())` instead
    of computing `gross - refunded`; the engine's
    `PartiallyRefunded { refunded, remaining }` invariant
    (per `port.rs:741-746`) is broken — every partially-refunded
    charge reports zero remaining, so the engine will allow
    refunds up to the full original amount (double-refund risk).
  - `Authorized` (stripe.rs:722-726) sets `auth_code: String::new()`
    and `expires_at: Timestamp::now()`; the auth code is empty
    and the expiry is "right now", so any auth-then-capture flow
    fails immediately.
  - `Disputed` (stripe.rs:740-744) sets `dispute_id: String::new()`,
    `reason: String::new()`; the dispute id is the only handle
    the support team has to escalate.
  - `Refunded` (stripe.rs:728-732) sets `reason: String::new()`
    and reuses the request `amount` rather than reading the
    actual refunded amount from Stripe.

**Expected:**

`docs/ports/payments.md` § "PaymentStatus" — all
  five variants carry the spec'd inner fields with the data
  Stripe actually returned.

**Evidence:**

```rust
  // crates/adapters/payment/src/stripe.rs:728-744
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
  ```

---

### FINDING 14 (id: `ADAPT-PAY-014`)

- **Source:** `docs/audit_reports/findings/wave3-payment.md`
- **Severity:** High
- **Area:** adapters-payment
- **Location:** `crates/adapters/payment/src/stripe.rs:413-444`
  (`refund` body — no `AlreadyRefunded` / `RefundExceedsOriginal`
  check) and `crates/adapters/payment/src/errors.rs:69-78`
  (`PaymentError::AlreadyRefunded`, `PaymentError::RefundExceedsOriginal`)

**Description:**

The error enum defines
  `PaymentError::AlreadyRefunded` and
  `PaymentError::RefundExceedsOriginal` (errors.rs:69-78) but
  the `StripeProvider::refund` implementation never returns
  either. A consumer that retries a refund past the original
  amount (or refunds an already-refunded receipt) receives a
  generic `PaymentError::Provider("stripe error (400): ...")`
  rather than the typed variant. The test list in
  `docs/ports/payments.md` § "Testing" requires "A test of
  partial refund" — without the typed errors the engine cannot
  detect the double-refund attempt at the domain layer and
  instead retries a 4xx, hitting Stripe rate limits.

**Expected:**

`docs/ports/payments.md` § "PaymentError" —
  `AlreadyRefunded` and `RefundExceedsOriginal` are the
  contract for double-refund detection.

**Evidence:**

```rust
  // crates/adapters/payment/src/stripe.rs:413-444 (no check)
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
      let body = self.post_form("refunds", &params, &request.idempotency_key.to_string()).await?;
      refund_receipt_from_refund(&body, request)
  }
  ```
  No comparison against the original `amount`; no `status()`
  precheck; no Stripe-side `charge.already_refunded` lookup.

---

### FINDING 15 (id: `ADAPT-PAY-015`)

- **Source:** `docs/audit_reports/findings/wave3-payment.md`
- **Severity:** High
- **Area:** adapters-payment
- **Location:** `crates/adapters/payment/src/services.rs:351-360`
  (`SettlementService::compute_net_settlement`,
  `SettlementRequest` validation) and
  `crates/adapters/payment/src/port.rs:967-982`
  (`SettlementRequest`)

**Description:**

`SettlementService` does not validate that
  `request.period_start <= request.period_end` or that the
  window is non-empty. A consumer that constructs a
  `SettlementRequest` with `period_start > period_end` (typo,
  clock skew, off-by-one) silently gets a settlement report
  covering a zero-length or backwards window. The Stripe
  reference impl returns `PaymentError::Provider("settlement is
  not implemented ...")` so the bug is currently masked, but a
  real payouts adapter will hand the window to Stripe as-is.
  The docstring on `SettlementRequest` (`port.rs:967-982`)
  describes the window as "Inclusive start of the settlement
  window" / "Inclusive end of the settlement window" but
  provides no validation entry point.

**Expected:**

`docs/ports/payments.md` § "Settlement &
  Reconciliation" — the window is well-formed.

**Evidence:**

```rust
  // crates/adapters/payment/src/port.rs:967-982
  pub struct SettlementRequest {
      pub tenant: TenantContext,
      pub period_start: Timestamp,
      pub period_end: Timestamp,
      pub currency: CurrencyCode,
  }
  ```
  No constructor / no validator; `SettlementService` exposes
  only `match_settlement_line`, `compute_net_settlement`, and
  `is_settled` — none of which inspect the window.

---

### FINDING 16 (id: `ADAPT-PAY-016`)

- **Source:** `docs/audit_reports/findings/wave3-payment.md`
- **Severity:** High
- **Area:** adapters-payment
- **Location:** `crates/adapters/payment/src/stripe.rs:358-411`
  (`charge` body — `Card` arm) and
  `crates/adapters/payment/src/stripe.rs:432-444`
  (`refund_receipt_from_refund` — `receipt_from_charge` mapping)

**Description:**

The Stripe `charge` POST does not request
  expanded fields (`expand[]=balance_transaction`,
  `expand[]=customer`). `extract_stripe_fees`
  (`stripe.rs:551-572`) reads
  `charge["balance_transaction"]` and returns an empty `fees`
  vector when the field is a string id rather than an expanded
  object. Stripe does not expand by default, so every shipped
  charge reports `fees: Vec::new()` and `net: amount` (gross
  equal to net), even though Stripe deducted a processing fee.
  The engine's `PaymentReceipt.net` (`port.rs:874-878`) is
  documented as "gross minus fees"; with `fees = []` the
  finance reconciler sees the gross deposit as the net deposit
  and the school's bank-account reconciliation drifts by the
  2.9% + 30¢ per charge.

**Expected:**

`docs/ports/payments.md` § "PaymentReceipt":
  `net: Money` is "The net amount (gross minus fees)
  deposited in the school's account."

**Evidence:**

```rust
  // crates/adapters/payment/src/stripe.rs:551-572
  fn extract_stripe_fees(
      charge: &JsonValue,
      currency: &crate::port::CurrencyCode,
  ) -> Result<Vec<PaymentFee>, PaymentError> {
      let Some(balance_txn) = charge.get("balance_transaction") else {
          return Ok(Vec::new());   // <- default path
      };
      if !balance_txn.is_object() {
          return Ok(Vec::new());   // <- default path
      }
      ...
  }
  ```
  And the POST body at `stripe.rs:362-388` has no
  `expand[]=balance_transaction` entry.

---

### FINDING 17 (id: `ADAPT-PAY-017`)

- **Source:** `docs/audit_reports/findings/wave3-payment.md`
- **Severity:** High
- **Area:** adapters-payment
- **Location:** `crates/adapters/payment/src/stripe.rs:897-906`
  (`stripe_refund_reason`) and `crates/adapters/payment/src/stripe.rs:413-444`
  (`refund` body — callsite)

**Description:**

`stripe_refund_reason` returns an empty
  string for any reason that does not match Stripe's whitelist
  (`duplicate` / `fraudulent` / `requested_by_customer`) — but
  the empty string is still passed to Stripe as the
  `reason=...` form field. Stripe silently drops the empty
  field and the audit log loses the human-readable refund
  reason entirely. The handoff notes the whitelist but does not
  document that the unmapped reason is destroyed. A consumer
  that records `"customer requested after delivery issue"`
  will see no audit trail entry tying the refund to the
  underlying issue.

**Expected:**

`docs/ports/payments.md` § "Refund" — reason is
  "shown on the customer's statement" and persisted in the
  audit log.

**Evidence:**

```rust
  // crates/adapters/payment/src/stripe.rs:897-906
  fn stripe_refund_reason(reason: &str) -> String {
      match reason {
          "duplicate" | "fraudulent" | "requested_by_customer" => reason.to_owned(),
          other => match other.to_ascii_lowercase().as_str() {
              "duplicate" => "duplicate".to_owned(),
              "fraud" | "fraudulent" => "fraudulent".to_owned(),
              "customer" | "requested" | "requested_by_customer" => {
                  "requested_by_customer".to_owned()
              }
              _ => String::new(),  // <- silent loss
          },
      }
  }
  // stripe.rs:413-444
  if !request.reason.is_empty() {
      params.push(("reason".to_owned(), stripe_refund_reason(&request.reason)));
  }
  ```
  When `request.reason = "customer changed mind"`,
  `stripe_refund_reason` returns `""`, which is then
  pushed to the form — Stripe drops it, the audit log has no
  reason.

---

### FINDING 18 (id: `ADAPT-PAY-018`)

- **Source:** `docs/audit_reports/findings/wave3-payment.md`
- **Severity:** High
- **Area:** adapters-payment
- **Location:** `crates/adapters/payment/src/stripe.rs:828-855`
  (`parse_stripe_signature`)

**Description:**

`parse_stripe_signature` captures only the
  first `v1=` value from the `Stripe-Signature` header and
  silently discards additional `v1=` entries. Stripe's
  documentation recommends emitting **multiple `v1=` values**
  during a webhook secret rotation so a delivery can be
  verified against either the old or new secret. The
  implementation only checks the first `v1=`; if Stripe sends
  `t=...,v1=<new-secret-sig>,v1=<old-secret-sig>` and the
  provider was configured with the old secret, the
  verification fails. Worse, there is no warning that
  additional `v1=` entries were dropped — the log shows a
  clean signature mismatch.

**Expected:**

`docs.stripe.com/webhooks#verify-official-libraries`
  — accept any `v1=` entry that verifies.

**Evidence:**

```rust
  // crates/adapters/payment/src/stripe.rs:828-855
  for part in signature.split(',') {
      let Some((k, v)) = part.split_once('=') else { continue; };
      match k {
          "t" => { timestamp = v.parse::<i64>().ok(); }
          "v1" if v1.is_none() => { v1 = Some(v); }  // <- first only
          _ => {}
      }
  }
  ```
  No `Vec<&str>` accumulation; no per-`v1` verify loop in
  `verify_webhook_signature` (stripe.rs:155-205).

---

### FINDING 8 (id: `ADAPT-PAY-008`)

- **Source:** `docs/audit_reports/findings/wave3-payment.md`
- **Severity:** High
- **Area:** adapters-payment
- **Location:** `crates/adapters/payment/Cargo.toml:11-22` and
  `crates/adapters/payment/src/lib.rs:11-13`

**Description:**

The crate has zero audit-log writes. The port
  spec (`docs/ports/payments.md` § "Audit") states: "Every
  charge, refund, status change, and settlement is recorded in
  the audit log with amount, currency, method, parties, and
  metadata. Card data is never logged." The crate's `Cargo.toml`
  declares `educore-core`, `educore-platform`, `educore-events`,
  `tokio`, `async-trait`, `reqwest`, `serde_json`, `hmac`,
  `sha2` — but no `educore-audit`. Per
  `PHASE-15-HANDOFF.md:46-51` the audit crate added two new
  variants (`PaymentReceipt`, `Refund`) for this purpose, but
  the payment adapter never imports `educore-audit` to use them.
  Every charge, refund, status change, and settlement therefore
  proceeds without an immutable audit trail. The
  `PaymentReceived`, `PaymentRefunded`, `PaymentSettled` events
  the spec references (`port.rs:983-985` for the docstring on
  receipts) likewise do not exist on this crate — `educore-events`
  is declared but only re-exported in the prelude is the trait
  shape, not a publisher call.

**Expected:**

`docs/ports/payments.md` § "Audit": "Every
  charge, refund, status change, and settlement is recorded in
  the audit log …" `AGENTS.md` Engine Rule 8: "Audit-first.
  Every state change writes an immutable record."

**Evidence:**

```toml
  # crates/adapters/payment/Cargo.toml:11-22
  [dependencies]
  educore-core = { workspace = true }
  educore-platform = { workspace = true }
  educore-events = { workspace = true }
  tokio = { workspace = true }
  async-trait = { workspace = true }
  reqwest = { workspace = true }
  serde_json = { workspace = true }
  hmac = { workspace = true }
  sha2 = { workspace = true }
  ```
  No `educore-audit`. `grep -rn "audit" crates/adapters/payment/`
  returns zero non-docstring hits.

---

### FINDING 9 (id: `ADAPT-PAY-009`)

- **Source:** `docs/audit_reports/findings/wave3-payment.md`
- **Severity:** High
- **Area:** adapters-payment
- **Location:** `crates/adapters/payment/src/services.rs:60-90`
  (`IdempotencyService::derive_charge_key`) and
  `docs/handoff/PHASE-15-HANDOFF.md:118-119`

**Description:**

`IdempotencyService::derive_charge_key` hashes
  only `command_id | invoice_ids | amount_minor` — no
  `tenant_school_id`, no `user_id`, no `currency`, no
  `payment_method`. The handoff (PHASE-15-HANDOFF.md:118-119)
  asserts the canonical form is
  `SHA-256(tenant|user|amount|currency|method)`, but the
  implementation does not match its own documentation. Two
  schools submitting the same `command_id` (a UUID collision or
  a consumer that mints `command_id`s from a per-school sequence)
  collide on the key. Two payments in the same school differing
  only in `currency` collapse to the same key. The
  `payment_integration_idempotency_charge_key` integration test
  at `tests/payment_integration.rs:14-31` only verifies invoice
  ordering and amount differences, not the documented tenant /
  user / currency / method fields.

**Expected:**

PHASE-15-HANDOFF.md:118-119 — `SHA-256(tenant |
  user | amount | currency | method)`.

**Evidence:**

```rust
  // crates/adapters/payment/src/services.rs:60-90
  #[must_use]
  pub fn derive_charge_key(
      command_id: &str,
      invoice_ids: &[String],
      amount_minor: i64,
  ) -> String {
      let mut sorted: Vec<&str> = invoice_ids.iter().map(String::as_str).collect();
      sorted.sort_unstable();
      let mut hasher = Sha256::new();
      hasher.update(command_id.as_bytes());
      hasher.update([0x1f_u8]);
      for inv in sorted {
          hasher.update(inv.as_bytes());
          hasher.update([0x1f_u8]);
      }
      hasher.update(amount_minor.to_le_bytes());
      hex_encode(&hasher.finalize())
  }
  ```
  No `school_id`, no `user_id`, no `currency`, no `method`.

---

### FINDING 19 (id: `ADAPT-PAY-019`)

- **Source:** `docs/audit_reports/findings/wave3-payment.md`
- **Severity:** Medium
- **Area:** adapters-payment
- **Location:** `crates/adapters/payment/src/stripe.rs:45-56`
  (`StripeProvider` struct fields `secret_key: String`,
  `webhook_secret: String`) and
  `crates/adapters/payment/src/stripe.rs:268-283`
  (`Debug` impl + `redact_secret`)

**Description:**

`StripeProvider` holds `secret_key` and
  `webhook_secret` as `String` (not `secrecy::SecretString`).
  The `Debug` impl redacts them with `redact_secret` returning
  `&'static str` — every configured provider's `Debug` output
  looks identical (`<redacted>`), so log analysis tools
  cannot distinguish two concurrently-wired Stripe providers
  (e.g. `sk_test_` vs `sk_live_`, or two schools with
  different keys). The wallet PIN deviation (FINDING 1) is
  the same shape; the API-key deviation is functionally
  similar but the practical risk is lower because the value
  never leaves the provider's `Debug` impl. The crate's
  deviation note (`port.rs:18-26`) explicitly opts out of
  `secrecy` — the cost is this debugging-blindness.

**Expected:**

`docs/ports/payments.md` § "Webhook Flow" —
  secrets must not leak via `Debug` or log; the `SecretString`
  type implements `Debug` as `Secret<String>` so the value
  is never reachable from a `format!("{:?}", provider)`.

**Evidence:**

```rust
  // crates/adapters/payment/src/stripe.rs:268-283
  impl fmt::Debug for StripeProvider {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
          f.debug_struct("StripeProvider")
              .field("base_url", &self.base_url)
              .field("secret_key", &redact_secret(&self.secret_key))
              .field("webhook_secret", &redact_secret(&self.webhook_secret))
              .finish()
      }
  }
  // stripe.rs:889-895
  fn redact_secret(s: &str) -> &'static str {
      if s.is_empty() { "<empty>" } else { "<redacted>" }
  }
  ```
  Two providers, two different keys, two identical `Debug`
  outputs.

---

### FINDING 20 (id: `ADAPT-PAY-020`)

- **Source:** `docs/audit_reports/findings/wave3-payment.md`
- **Severity:** Medium
- **Area:** adapters-payment
- **Location:** `crates/adapters/payment/tests/payment_integration.rs:128-145`
  (`payment_integration_async_stripe_charge_mock`,
  `payment_integration_async_stripe_refund_mock`)

**Description:**

Both env-gated async integration tests are
  `#[ignore = "requires EDUCORE_PORT_ADAPTER_E2E env var"]` and
  contain only `StripeProviderBuilder::new()...build()` calls —
  they verify that `reqwest::Client::builder()` succeeds, not
  that an actual Stripe round-trip works. No `mockito`,
  `wiremock`, or `httpmock` server is used; no HTTP traffic is
  recorded; no fixture body is parsed. The handoff
  (PHASE-15-HANDOFF.md:131-135) describes the tests as
  "vertical-slice integration tests" but they only verify the
  builder constructs. A `cargo test --workspace` will never run
  them; `cargo test -- --ignored` will pass without exercising
  any Stripe API call.

**Expected:**

`docs/ports/payments.md` § "Testing" — charge,
  refund, status, idempotency, webhook reconciliation each
  have a passing test against a mocked Stripe endpoint.

**Evidence:**

```rust
  // crates/adapters/payment/tests/payment_integration.rs:128-145
  #[tokio::test]
  #[ignore = "requires EDUCORE_PORT_ADAPTER_E2E env var"]
  async fn payment_integration_async_stripe_charge_mock() {
      let _provider = StripeProviderBuilder::new()
          .secret_key("sk_test_placeholder".to_owned())
          .webhook_secret("whsec_test_placeholder".to_owned())
          .build()
          .expect("reqwest client build");
  }
  #[tokio::test]
  #[ignore = "requires EDUCORE_PORT_ADAPTER_E2E env var"]
  async fn payment_integration_async_stripe_refund_mock() {
      let _provider = StripeProviderBuilder::new()
          .secret_key("sk_test_placeholder".to_owned())
          .build()
          .expect("reqwest client build");
  }
  ```
  No `let body = ...`; no `MockServer::start()`; no assertion
  on response shape.

---

### FINDING 21 (id: `ADAPT-PAY-021`)

- **Source:** `docs/audit_reports/findings/wave3-payment.md`
- **Severity:** Medium
- **Area:** adapters-payment
- **Location:** `crates/adapters/payment/src/port.rs:34-39`
  (deviation note § "URL is represented as `String`") and
  `crates/adapters/payment/src/port.rs:841-846`
  (`ChargeRequest::webhook_url: Option<String>`)

**Description:**

The port replaces `url::Url` with `String`
  for `webhook_url`, `return_url`, and `receipt_url`. None of
  these fields is validated at construction time: a malformed
  URL (`"not a url"`, `"javascript:alert(1)"`, empty string)
  flows through to Stripe as the `return_url=` form field and
  is silently accepted by the port. A typo in the return URL
  redirects the customer to a 404; a `javascript:` URL in a
  hosted-page flow is a reflected XSS vector if the gateway's
  page surfaces the return URL verbatim.

**Expected:**

`docs/ports/payments.md` § "ChargeRequest" —
  `webhook_url: Option<Url>` (typed, parsed, scheme-checked).

**Evidence:**

```rust
  // crates/adapters/payment/src/port.rs:841-846
  /// Optional webhook URL the gateway should POST status
  /// updates to.
  pub webhook_url: Option<String>,
  ```
  No `Url::parse`; no `scheme == "https"` check; bare `String`.

---

### FINDING 22 (id: `ADAPT-PAY-022`)

- **Source:** `docs/audit_reports/findings/wave3-payment.md`
- **Severity:** Medium
- **Area:** adapters-payment
- **Location:** `crates/adapters/payment/src/services.rs:332-360`
  (`SettlementService`) and
  `crates/adapters/payment/src/port.rs:998-1011`
  (`Settlement` struct)

**Description:**

`SettlementService` exposes only three
  helpers (`match_settlement_line`, `compute_net_settlement`,
  `is_settled`) — none of them validate the invariants on the
  `Settlement` struct itself:
  - `Settlement.total_gross == sum(line.gross)` is not checked.
  - `Settlement.total_fees == sum(line.fee)` is not checked.
  - `Settlement.total_net == sum(line.net)` is not checked.
  - `Settlement.lines` may be empty while `total_gross` is
    non-zero (no empty-batch invariant).
  - `Settlement.school_id` may not match
    `SettlementRequest.tenant.school_id` (no school-tenant
    cross-check).
  - `Settlement.currency` may not match every `line.net.currency`
    (no per-line currency validation).
  These are exactly the invariants the spec expects the
  settlement adapter to enforce
  (`docs/ports/payments.md` § "Settlement & Reconciliation");
  without them a buggy payouts adapter can produce
  `Settlement { total_gross: 100_000, lines: [] }` and the
  engine has no way to detect the inconsistency.

**Expected:**

`docs/ports/payments.md` § "Settlement" — the
  totals agree with the lines.

**Evidence:**

```rust
  // crates/adapters/payment/src/port.rs:998-1011
  pub struct Settlement {
      pub settlement_id: String,
      pub school_id: SchoolId,
      pub currency: CurrencyCode,
      pub period_start: Timestamp,
      pub period_end: Timestamp,
      pub lines: Vec<SettlementLine>,
      pub total_gross: Money,
      pub total_fees: Money,
      pub total_net: Money,
  }
  ```
  No `Settlement::validate(&self) -> Result<(), PaymentError>`;
  no constructor.

---

### FINDING 23 (id: `ADAPT-PAY-023`)

- **Source:** `docs/audit_reports/findings/wave3-payment.md`
- **Severity:** Medium
- **Area:** adapters-payment
- **Location:** `crates/adapters/payment/src/port.rs:323-348`
  (`CurrencyCode::new`) and `crates/adapters/payment/src/port.rs:369-388`
  (`Money::new`)

**Description:**

`CurrencyCode::new` rejects non-ASCII-uppercase
  input (`port.rs:323-348`), but the `Cargo.toml` does not
  depend on any ISO-4217 source-of-truth list, so `CurrencyCode::new("XBT")`
  succeeds even though "XBT" is not a registered ISO-4217
  alphabetic code (the closest match is XBT — actually XBT is
  the unofficial bitcoin code; ISO reserves it under the
  internal "X" range). Similarly `"USDX"`, `"ZZZ"`, `"AAA"`
  all pass. A consumer charging in a non-ISO currency sends
  it to Stripe, which rejects with `400` — but the typed
  `CurrencyCode` gives the consumer a false sense of validity.
  The spec (`docs/ports/payments.md` § "Multi-Currency") treats
  the code as authoritative; the validator should match.

**Expected:**

`docs/ports/payments.md` § "Multi-Currency" —
  `CurrencyCode` is a valid ISO-4217 code (not just
  3 ASCII uppercase).

**Evidence:**

```rust
  // crates/adapters/payment/src/port.rs:323-348
  pub fn new(code: &str) -> Result<Self, crate::errors::PaymentError> {
      if code.len() == 3 && code.chars().all(|c| c.is_ascii_uppercase()) {
          Ok(Self(code.to_owned()))
      } else {
          Err(...)
      }
  }
  ```
  No ISO-4217 lookup; `"ZZZ"` is accepted.

---

### FINDING 24 (id: `ADAPT-PAY-024`)

- **Source:** `docs/audit_reports/findings/wave3-payment.md`
- **Severity:** Low
- **Area:** adapters-payment
- **Location:** `crates/adapters/payment/src/stripe.rs:54-56`
  (`HTTP_TIMEOUT_SECS = 30`) and `crates/adapters/payment/src/stripe.rs:332-344`
  (`StripeProviderBuilder::build` — Client builder)

**Description:**

`StripeProviderBuilder::build` configures a
  single 30-second request timeout via
  `Client::builder().timeout(Duration::from_secs(30))`. There is
  no `connect_timeout`, no `tcp_keepalive`, no retries
  (`reqwest`'s built-in retry middleware is off by default), no
  request-id propagation, and no user-agent string identifying
  the Educore engine. A consumer that hits Stripe over a
  flaky link (mobile / developing-country 3G) will see every
  charge fail with `Infrastructure` after 30 s. The
  `PaymentError::RateLimited` variant exists but the adapter
  does not back off; it surfaces the rate-limit response
  immediately to the caller, with no hint about how long to
  wait.

**Expected:**

AGENTS.md § "TLS/SSL Cross-Compilation"
  requires `rustls` for `reqwest`. The same standards should
  drive a production-grade HTTP client (connect timeout, retry
  policy, user-agent).

**Evidence:**

```rust
  // crates/adapters/payment/src/stripe.rs:54-56
  /// HTTP request timeout applied to every outbound call.
  const HTTP_TIMEOUT_SECS: u64 = 30;
  // stripe.rs:332-344
  let http = Client::builder()
      .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
      .build()
      .map_err(|e| PaymentError::Infrastructure(InfrastructureError::new(...)))?;
  ```
  No `.connect_timeout(...)`, no `.user_agent(...)`, no
  `.tcp_keepalive(...)`, no retry config.

---


## Storage — PostgreSQL (target id prefix: `ADAPTER-PG`)

**Path:** `crates/adapters/storage-postgres/`  
**Total findings:** 47 (13 critical, 13 high, 15 medium, 6 low)


### FINDING 1 (id: `ADAPTER-PG-001`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/storage.rs:130

**Description:**

The adapter exposes a `migrate()` method on
  `StorageAdapter`, but every consumer-facing doc
  (`AGENTS.md:544, 561`, `README.md:173`,
  `docs/schemas/sql-dialects/README.md:193-198`,
  `docs/schemas/sql-dialects/postgresql.md:9`,
  `docs/build-plan.md:119, 175-179, 186`,
  `docs/architecture.md:322`,
  `migrations/engine/README.md:11`,
  `CONTRIBUTING.md:502`) refers to the runtime entry point as
  `storage.create_schema().await`. The consumer-facing API name does
  not exist on the trait.

**Expected:**

`docs/build-plan.md:175-179` —
  `("create_schema", "apply_command", "query", "begin_tx", ...)`
  and `storage.create_schema().await` runs the DDL.

**Evidence:**

`crates/adapters/storage-postgres/src/storage.rs:130` —
  ```rust
  async fn migrate(&self) -> Result<MigrationReport> {
  ```
  And `crates/infra/storage/src/port.rs:44`:
  ```rust
  async fn migrate(&self) -> Result<MigrationReport>;
  ```
  No `create_schema` method exists in the entire crate
  (`grep -rn "fn create_schema" crates/adapters/storage-postgres/`
  returns no results).

---

### FINDING 10 (id: `ADAPTER-PG-010`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** migrations/engine/0000_engine_core.postgres.sql:229

**Description:**

The `system_user.active_status` column is declared
  `SMALLINT NOT NULL DEFAULT 1`. The dialect spec at
  `postgresql.md:373` mandates `BOOLEAN NOT NULL DEFAULT TRUE`.

**Expected:**

`docs/schemas/sql-dialects/postgresql.md:373` —
  `"active_status" BOOLEAN     NOT NULL DEFAULT TRUE,`

**Evidence:**

`migrations/engine/0000_engine_core.postgres.sql:229` —
  `active_status SMALLINT     NOT NULL DEFAULT 1,`

---

### FINDING 11 (id: `ADAPTER-PG-011`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/idempotency.rs:238-244

**Description:**

`lookup_command_type` calls `Box::leak(boxed)` on
  every read, leaking the `command_type` string into static memory.
  Per `AGENTS.md` § "Engine Rules" and `docs/code-standards.md`
  code standards, `Box::leak` in production paths is forbidden.
  Per-read memory growth is unbounded.

**Expected:**

`crates/infra/storage/src/idempotency.rs:31` —
  `pub command_type: &'static str,` (the field is `&'static str`,
  but the value comes from a `VARCHAR` column read).
  The port struct should use `String` (or `Cow<'static, str>`).
  `PHASE-1-HANDOFF.md:176-180` acknowledges this as "Open question
  #3" but the leak is shipped.

**Evidence:**

`crates/adapters/storage-postgres/src/idempotency.rs:238-244` —
  ```rust
  fn lookup_command_type(s: &str) -> &'static str {
      // Allocate a `Box<str>` and leak it. The leak is bounded
      // by the cardinality of the engine's command catalogue
      // (a few hundred at most) and the lifetime of the process;
      // a periodic sweep can be added if it becomes a concern.
      let boxed: Box<str> = Box::from(s);
      Box::leak(boxed)
  }
  ```

---

### FINDING 12 (id: `ADAPTER-PG-012`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/idempotency.rs:181

**Description:**

`expires_at` is computed as `recorded_at +
  Duration::hours(24)` with `.unwrap_or(recorded_at)` fallback.
  On a chrono overflow (e.g. far-future `recorded_at`), the
  fallback silently makes the record's `expires_at == recorded_at`,
  causing the row to be eligible for immediate purge on the next
  sweep — a silent data-loss path.

**Expected:**

`crates/infra/storage/src/idempotency.rs:107-113` —
  `purge_older_than` returns `u64` rows affected; the adapter
  should never silently shorten a retention window to zero.

**Evidence:**

`crates/adapters/storage-postgres/src/idempotency.rs:179-181` —
  ```rust
  let expires_at = recorded_at
      .checked_add_signed(Duration::hours(DEFAULT_RETENTION_HOURS))
      .unwrap_or(recorded_at);
  ```

---

### FINDING 13 (id: `ADAPTER-PG-013`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/outbox.rs:175-191

**Description:**

`Outbox::pending_count` accepts an arbitrary
  `school_id: SchoolId` argument and filters by it, ignoring the
  handle's scoped `self.school`. A caller can request the pending
  count for any tenant — bypassing the adapter's own scoping.
  The same pattern is broken in `event_log::read`/`count` and
  `audit_log::read_for_target` (they accept an explicit
  `school_id` arg but the handle's `school` field is
  `#[allow(dead_code)]` and unused).

**Expected:**

`docs/schemas/tenancy-schema.md` and `docs/ports/storage.md:140-150` —
  "The storage adapter is responsible for enforcing tenant
  isolation. The engine always passes a SchoolId filter; the
  adapter MUST add a school_id = $1 predicate to every read query."

**Evidence:**

`crates/adapters/storage-postgres/src/outbox.rs:175-191` —
  ```rust
  async fn pending_count(&self, school_id: SchoolId) -> Result<u64> {
      // ... override with a direct COUNT(*)
      let row = sqlx::query(
          "SELECT COUNT(*) AS n FROM outbox WHERE school_id = $1 AND published_at IS NULL",
      )
      .bind(school_id.as_uuid())
      .fetch_one(&self.pool)
  ```
  And `crates/adapters/storage-postgres/src/audit_log.rs:117-118` —
  ```rust
  #[allow(dead_code)]
  school: SchoolId,
  ```

---

### FINDING 2 (id: `ADAPTER-PG-002`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** migrations/engine/0000_engine_core.postgres.sql (entire 240-line file)

**Description:**

The canonical PG DDL the adapter
  `include_str!`'s contains no row-level security policies and no
  `ENABLE ROW LEVEL SECURITY` / `FORCE ROW LEVEL SECURITY` clauses
  on any of the 6 cross-cutting tables. Per
  `docs/schemas/sql-dialects/postgresql.md:122-159` PG is required
  to use `CREATE POLICY` + `ENABLE ROW LEVEL SECURITY` and the
  adapter must issue `SET LOCAL app.current_school_id = ?` on every
  transaction.

**Expected:**

`docs/schemas/sql-dialects/postgresql.md:122-159`:
  ```sql
  ALTER TABLE "<aggregate>" ENABLE ROW LEVEL SECURITY;
  ALTER TABLE "<aggregate>" FORCE ROW LEVEL SECURITY;
  CREATE POLICY "school_isolation_<aggregate>" ON "<aggregate>"
    USING ("school_id" = current_setting('app.current_school_id')::UUID)
    WITH CHECK ("school_id" = current_setting('app.current_school_id')::UUID);
  ```

**Evidence:**

`migrations/engine/0000_engine_core.postgres.sql:1-240` —
  contains only `CREATE SCHEMA`, `CREATE TABLE IF NOT EXISTS`, and
  `CREATE INDEX IF NOT EXISTS` statements. No `ALTER TABLE ... ENABLE
  ROW LEVEL SECURITY`, no `CREATE POLICY`, no RLS clause anywhere in
  the 240 lines.

---

### FINDING 3 (id: `ADAPTER-PG-003`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/connection.rs:69-90

**Description:**

The connection's `after_connect` hook issues
  `SET search_path = engine, public` but does NOT issue
  `SET app.current_school_id = '<uuid>'`. Per
  `docs/schemas/sql-dialects/postgresql.md:142-145`, the engine's
  adapter must issue `SET LOCAL app.current_school_id = ?` on every
  new transaction so RLS policies can resolve the tenant. Without
  this, even if RLS were enabled, every query would see zero rows.

**Expected:**

`docs/schemas/sql-dialects/postgresql.md:142` —
  `SET LOCAL app.current_school_id = '<uuid>';`

**Evidence:**

`crates/adapters/storage-postgres/src/connection.rs:69-87` —
  ```rust
  .after_connect(|conn, _meta| {
      Box::pin(async move {
          sqlx::query("SET search_path = engine, public")
              .execute(conn)
              .await?;
          Ok(())
      })
  })
  ```
  No `SET LOCAL app.current_school_id` is issued.

---

### FINDING 4 (id: `ADAPTER-PG-004`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** migrations/engine/0000_engine_core.postgres.sql:73, 121-122, 153, 183, 208

**Description:**

The canonical PG DDL declares `JSONB NOT NULL` /
  `JSONB NULL` on the JSONB columns of the 6 cross-cutting tables
  with NO `CHECK (jsonb_typeof(...) = 'object')` constraints. The
  dialect spec at `postgresql.md:58, 249, 286-288, 314, 339, 359`
  mandates the JSONB CHECK constraint on every JSONB column to
  guarantee the payload is a JSON object (not a JSON array, scalar,
  or null where forbidden).

**Expected:**

`docs/schemas/sql-dialects/postgresql.md:58` —
  `JSONB NOT NULL CHECK (jsonb_typeof("payload") = 'object')`
  and line 249 — `"payload" JSONB NOT NULL CHECK (jsonb_typeof("payload") = 'object')`.

**Evidence:**

`migrations/engine/0000_engine_core.postgres.sql:73` —
  `payload         JSONB        NOT NULL,` (no CHECK constraint).
  Same omission on `before_snapshot`, `after_snapshot`, `metadata`
  (lines 120-122), `outcome` (line 153), `payload` on event_log
  (line 183), `schema_json` (line 208).

---

### FINDING 5 (id: `ADAPTER-PG-005`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/storage.rs:130-169

**Description:**

The `migrate()` implementation does not walk any
  macro-emitted AST or render any domain table DDL. Per
  `docs/build-plan.md:177-179` and `docs/schemas/sql-dialects/README.md:182-187`,
  the adapter must "walk the macro-emitted AST to render the ~310
  domain tables at create_schema() time". The PG adapter only emits
  the 6 cross-cutting tables plus the `attendance_student_attendances`
  table — zero of the ~310 domain tables are emitted.

**Expected:**

`docs/build-plan.md:179` —
  `Walks the macro-emitted AST to render the ~310 domain tables
  at create_schema() time`.

**Evidence:**

`crates/adapters/storage-postgres/src/storage.rs:141-144` —
  ```rust
  sqlx::raw_sql(SCHEMA_SQL)
      .execute(self.conn.db())
      .await
      .map_err(DomainError::infrastructure)?;
  ```
  Followed by `PostgresBulkAttendance::new(...).ensure_schema().await?;`
  (line 149-151). No AST walk, no domain table emission, no reference
  to any macro-emitted `EntityDescriptor`.

---

### FINDING 6 (id: `ADAPTER-PG-006`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/storage.rs:118-127

**Description:**

The adapter implements `begin()` which returns a
  `Box<dyn Transaction>`. The `Transaction` trait exposes
  `commit`/`rollback`, but they are NO-OPs in this adapter (see
  `transaction.rs:122-129, 131-137`) — the engine's at-least-once
  outbox dedup is the safety net. Per
  `docs/ports/storage.md:120-136` and
  `crates/infra/storage/src/transaction.rs:32-91` the contract is
  for an actual transactional commit. The PHASE-1-HANDOFF.md:38-46
  acknowledges this is a flag-based stub.

**Expected:**

`docs/ports/storage.md:124-127` —
  `async fn commit(self: Box<Self>) -> Result<()>` and "On commit
  the writes are persisted and the outbox events are released to
  the event bus."

**Evidence:**

`crates/adapters/storage-postgres/src/transaction.rs:122-129` —
  ```rust
  async fn commit(self: Box<Self>) -> Result<()> {
      // No-op: the sub-port operations have already committed
      // via the `sqlx::Transaction` they each acquired. We
      // only flip the guard flag.
      self.done.store(true, Ordering::SeqCst);
      Ok(())
  }
  ```

---

### FINDING 7 (id: `ADAPTER-PG-007`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** migrations/engine/0000_engine_core.postgres.sql (entire 240-line file)

**Description:**

The PG DDL uses schema-qualified unquoted
  identifiers (`engine.outbox`, `engine.audit_log`, etc.). The
  dialect spec at `postgresql.md:11-23, 96-114` mandates
  **double-quoted lowercase identifiers** (`"outbox"`,
  `"event_id"`, etc.) and reserves schema-prefixing as a consumer
  choice, not the canonical engine form.

**Expected:**

`docs/schemas/sql-dialects/postgresql.md:11-23` —
  "Use **double quotes** for every identifier"
  and `"CREATE TABLE \"outbox\" ("`.

**Evidence:**

`migrations/engine/0000_engine_core.postgres.sql:61` —
  `CREATE TABLE IF NOT EXISTS engine.outbox (` (unquoted,
  schema-qualified) vs spec line 237 — `CREATE TABLE IF NOT EXISTS
  "outbox" (` (double-quoted, no schema).

---

### FINDING 8 (id: `ADAPTER-PG-008`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/tests/outbox_e2e.rs:1-81

**Description:**

The entire `tests/` directory contains a single
  e2e test file with one test function (`outbox_append_and_pending_round_trip`).
  Per `docs/ports/storage.md:469-477` the port requires unit tests of
  every repository method, integration tests against a real
  database, a parity test, a tenancy test, a failure-injection
  test, and a load test (10k attendance marks in <5s). None of the
  AuditLog, Idempotency, EventLog, or BulkAttendance sub-ports have
  any test, and the single e2e test is gated behind an env var that
  is unset in CI (`EDUCORE_PG_URL`).

**Expected:**

`docs/ports/storage.md:470-477` —
  - Unit tests of every repository method
  - Integration tests against a real database (testcontainers)
  - A parity test verifying identical behavior across adapters
  - A tenancy test verifying cross-tenant reads are blocked
  - A failure-injection test (e.g. deadlock retry, connection drop)
  - A load test (10k attendance marks in <5s)

**Evidence:**

`crates/adapters/storage-postgres/tests/` —
  `total 12` (one file, 3078 bytes). `tests/outbox_e2e.rs:1-81`
  contains one `#[tokio::test]` function. `PHASE-1-HANDOFF.md:19-22`
  acknowledges `124 passing` for the entire workspace but
  `+4 from the MySQL connection::tests URL helper unit tests` —
  i.e. the Phase 1 e2e count is 3 total across all SQL adapters
  (1 per adapter).

---

### FINDING 9 (id: `ADAPTER-PG-009`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** migrations/engine/0000_engine_core.postgres.sql:117

**Description:**

The DDL column `ip` on `audit_log` is declared
  `VARCHAR(45)`. The dialect spec at `postgresql.md:283` mandates
  `INET` (PostgreSQL's native IPv4/IPv6 type with validation).

**Expected:**

`docs/schemas/sql-dialects/postgresql.md:283` —
  `"ip" INET,`

**Evidence:**

`migrations/engine/0000_engine_core.postgres.sql:117` —
  `ip              VARCHAR(45)     NULL,`

---

### FINDING 14 (id: `ADAPTER-PG-014`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/storage.rs:82-113, docs/ports/storage.md:418-429

**Description:**

The port contract at `docs/ports/storage.md:418-429`
  specifies a `PostgresStorage::builder().url(...).max_connections(20)
  .min_connections(2).acquire_timeout(...).statement_cache_capacity(128)
  .build()` pattern. The adapter only exposes
  `PostgresStorageAdapter::connect(url, school)` with no way to
  configure pool size, acquire timeout, statement cache, or
  statement-cache capacity.

**Expected:**

`docs/ports/storage.md:418-429` — full builder
  pattern with `.max_connections(20)`, `.min_connections(2)`,
  `.acquire_timeout(...)`, `.statement_cache_capacity(128)`.

**Evidence:**

`crates/adapters/storage-postgres/src/storage.rs:82-113` —
  exposes only `PostgresStorageAdapter::new(conn)` and
  `PostgresStorageAdapter::connect(url, school)`. No builder, no
  pool-config methods.
  `grep -rn "max_connections\|min_connections\|acquire_timeout\|statement_cache_capacity"
  crates/adapters/storage-postgres/src/` returns no results.

---

### FINDING 15 (id: `ADAPTER-PG-015`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/storage.rs:163-168

**Description:**

`MigrationReport.already_at_version` is
  hard-coded to `false`. Per the port contract at
  `crates/infra/storage/src/change_stream.rs:243-255`, the field
  indicates "Whether the migration was a no-op (already at
  version)". The adapter cannot distinguish a no-op re-run from a
  fresh migration.

**Expected:**

`crates/infra/storage/src/change_stream.rs:253-254` —
  `/// Whether the migration was a no-op (already at version).`
  `pub already_at_version: bool,`

**Evidence:**

`crates/adapters/storage-postgres/src/storage.rs:163-168` —
  ```rust
  Ok(MigrationReport {
      version: SCHEMA_VERSION,
      statements_executed,
      duration,
      already_at_version: false,
  })
  ```

---

### FINDING 16 (id: `ADAPTER-PG-016`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/outbox.rs:61, 123, 190; event_log.rs:67, 197, 260; idempotency.rs:100, 219-220

**Description:**

Eight locations use `.unwrap_or(0)` on numeric
  conversions (`u32::try_from(...).unwrap_or(0)`,
  `u64::try_from(...).unwrap_or(0)`). A negative `event_version`
  on a row (DQL corruption, manual DB tampering, or a legacy row)
  silently becomes `0`. A negative `rows_affected()` silently
  becomes `i64::MAX` then `0`. The clippy deny for `cast_possible_wrap`
  / `cast_sign_loss` is being dodged by silently substituting zero.

**Expected:**

`docs/code-standards.md` and `AGENTS.md` §
  "Type Safety" — "No `as` casts that truncate or lose data.
  Use `TryFrom`/`TryInto` with proper error handling."

**Evidence:**

`crates/adapters/storage-postgres/src/outbox.rs:61` —
  ```rust
  schema_version: u32::try_from(self.event_version).unwrap_or(0),
  ```
  And `crates/adapters/storage-postgres/src/idempotency.rs:219-220` —
  ```rust
  let n: i64 = row.rows_affected().try_into().unwrap_or(i64::MAX);
  Ok(u64::try_from(n).unwrap_or(0))
  ```

---

### FINDING 17 (id: `ADAPTER-PG-017`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/audit_log.rs:155-184

**Description:**

`audit_log.append` always writes
  `source = 'api'` (hard-coded literal in the SQL string). The
  AuditLogEntry struct has no `source` field. Background jobs,
  the outbox relay, migrations, and other legitimate producers
  cannot distinguish their audit rows.

**Expected:**

`migrations/engine/0000_engine_core.postgres.sql:124` —
  `source VARCHAR(16) NOT NULL,` (the DDL column is mandatory,
  so it must carry meaningful producer information).

**Evidence:**

`crates/adapters/storage-postgres/src/audit_log.rs:163-165` —
  ```rust
  ) VALUES (
      $1, $2, $3, $4, $5, $6, $7, $8, NULL, $9, $10, \
      $11, NULL, NULL, NULL, $12, $13, $14, FALSE, 'api'\
  )
  ```
  No binding for source; always literal `'api'`.

---

### FINDING 18 (id: `ADAPTER-PG-018`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/audit_log.rs:154, 177

**Description:**

`audit_log.append` sets `recorded_at =
  occurred_at` (a literal copy of the command's wall-clock time).
  The DDL declares a separate `recorded_at TIMESTAMPTZ` column
  (line 116) intended to track ingestion latency — the difference
  between when the command occurred and when the row was persisted.
  The adapter discards the latency signal.

**Expected:**

`docs/schemas/audit-schema.md` § 13 (referenced
  in DDL line 94) — `recorded_at` is the time the row was written
  by the audit sink.

**Evidence:**

`crates/adapters/storage-postgres/src/audit_log.rs:154, 177` —
  ```rust
  let recorded_at: DateTime<Utc> = entry.occurred_at.as_datetime();
  ...
  .bind(recorded_at)
  ```

---

### FINDING 19 (id: `ADAPTER-PG-019`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/audit_log.rs:78

**Description:**

`AuditLogRow::metadata` is declared
  `Json<Value>` (NOT NULL). The DDL declares
  `metadata JSONB NULL` (line 122). The adapter's read shape and
  the DDL disagree on nullability, but the row's metadata is
  always `Value::Null` (the port default), so this is silently
  mapped to JSON null rather than SQL NULL.

**Expected:**

`migrations/engine/0000_engine_core.postgres.sql:122` —
  `metadata        JSONB            NULL,`

**Evidence:**

`crates/adapters/storage-postgres/src/audit_log.rs:78` —
  ```rust
  metadata: Json<Value>,
  ```
  vs `migrations/engine/0000_engine_core.postgres.sql:122` —
  `metadata        JSONB            NULL,`

---

### FINDING 20 (id: `ADAPTER-PG-020`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/idempotency.rs:51-61, audit_log.rs:53-83, event_log.rs:35-49

**Description:**

`IdempotencyRow`, `AuditLogRow`, and
  `EventLogRow` declare DDL columns that the read query never
  actually consumes (`#[allow(dead_code)]` on `audit_id`,
  `actor_type`, `command_id`, `ip`, `user_agent`, `session_id`,
  `recorded_at`, `cross_tenant`, `source`, `command_id`,
  `expires_at`). These fields are queried from the database (the
  SELECT includes them) but discarded on read. This wastes I/O
  and signals that the adapter has drifted from the port struct's
  shape — `PHASE-1-HANDOFF.md:168-175` acknowledges this in
  Open question #2.

**Expected:**

`crates/infra/storage/src/audit.rs:62-101` —
  `AuditLogEntry` should be the superset of DDL columns the
  port cares about.

**Evidence:**

`crates/adapters/storage-postgres/src/audit_log.rs:53-83` —
  ```rust
  struct AuditLogRow {
      #[allow(dead_code)]
      audit_id: Uuid,
      ...
      #[allow(dead_code)]
      ip: Option<String>,
      #[allow(dead_code)]
      user_agent: Option<String>,
      #[allow(dead_code)]
      session_id: Option<Uuid>,
      ...
      #[allow(dead_code)]
      cross_tenant: bool,
      #[allow(dead_code)]
      source: String,
  }
  ```
  Eight `#[allow(dead_code)]` annotations on a single 30-line
  struct.

---

### FINDING 21 (id: `ADAPTER-PG-021`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/bulk_attendance.sql:14-39

**Description:**

The bulk-attendance DDL stores UUIDs as `BYTEA`,
  dates as `TEXT`, and counters as `INTEGER`. The dialect spec at
  `postgresql.md:39-68` mandates `UUID` for UUIDv7 ids, `DATE`
  for calendar dates, and `INTEGER` is acceptable for counters,
  but the spec also says "engine emits `UUID NOT NULL`" and
  prefers native types over wire-decoupled `BYTEA`/`TEXT`. The
  port comment at `student_attendance_row.rs:108-114` explicitly
  admits this is "decoupled from the canonical engine form."

**Expected:**

`docs/schemas/sql-dialects/postgresql.md:46, 56` —
  `"id" UUID NOT NULL`, `"date_of_birth" DATE`.

**Evidence:**

`crates/adapters/storage-postgres/src/bulk_attendance.sql:14-39` —
  ```sql
  CREATE TABLE IF NOT EXISTS attendance_student_attendances (
      school_id            BYTEA      NOT NULL,
      id                   BYTEA      NOT NULL,
      student_id           BYTEA      NOT NULL,
      ...
      attendance_date      TEXT       NOT NULL,
      ...
      is_absent            INTEGER    NOT NULL DEFAULT 0,
      ...
      active_status        INTEGER    NOT NULL DEFAULT 1,
      ...
  )
  ```

---

### FINDING 22 (id: `ADAPTER-PG-022`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/error.rs:1-50

**Description:**

The PG adapter does not define a
  `StorageError` enum. The port contract at
  `docs/ports/storage.md:216-235` defines a 10-variant
  `StorageError` enum (`Connection`, `Conflict`, `Deadlock`,
  `UniqueViolation`, `ForeignKey`, `Check`, `NotFound`,
  `Infrastructure`, `Timeout`, `SerializationFailure`) and
  states "The engine maps `StorageError::Infrastructure` to
  `DomainError::Infrastructure`". The PG adapter only provides a
  `StringError` wrapper plus a free function `map_infrastructure`,
  with no typed translation for conflict / deadlock / unique
  violation / foreign key / check / not found / timeout /
  serialization failure.

**Expected:**

`docs/ports/storage.md:217-230` —
  ```rust
  pub enum StorageError {
      #[error("connection failed: {0}")] Connection(String),
      #[error("transaction conflict: {0}")] Conflict(String),
      #[error("deadlock detected")] Deadlock,
      #[error("unique violation: {0}")] UniqueViolation { constraint: String },
      ...
  }
  ```

**Evidence:**

`crates/adapters/storage-postgres/src/error.rs:1-50` —
  defines only `pub struct StringError(pub String);` and
  `pub fn map_infrastructure<E>(e: E) -> ...`. No `StorageError`
  enum. `grep -rn "StorageError" crates/adapters/storage-postgres/`
  returns no results.

---

### FINDING 23 (id: `ADAPTER-PG-023`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/audit_log.rs:147-151

**Description:**

`actor_type` is computed as a hard-coded
  `if entry.actor_id == SYSTEM_USER_ID { "system" } else { "user" }`.
  There is no provision for background-job actors, scheduled-job
  actors, sync-relay actors, or migration actors — all are
  misclassified as `"user"`. The DDL column is
  `VARCHAR(16) NOT NULL` with no CHECK constraint to catch
  drift.

**Expected:**

`docs/schemas/sql-dialects/postgresql.md:160-168` —
  `"role_type" VARCHAR(16) NOT NULL CHECK ("role_type" IN
  ('system', 'custom'))` — the engine mandates CHECK constraints
  on enum-like columns.

**Evidence:**

`crates/adapters/storage-postgres/src/audit_log.rs:147-151` —
  ```rust
  let actor_type: &'static str = if entry.actor_id == SYSTEM_USER_ID {
      "system"
  } else {
      "user"
  };
  ```

---

### FINDING 24 (id: `ADAPTER-PG-024`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/event_log.rs:114-164, 213-240

**Description:**

`build_select` constructs dynamic SQL by string
  concatenation (`sql.push_str(" AND event_type = ANY($"); ...`).
  While the only interpolated values are operator symbols, column
  names, and `$N` placeholders (no user input is interpolated),
  the comment at lines 121-124 and 137-141 explicitly justifies
  bypassing `format!` for clippy cleanliness rather than for
  safety. The pattern is fragile: any future change that adds a
  user-input interpolation would be invisible to code review.

**Expected:**

`docs/ports/storage.md:296-313` and
  `docs/schemas/sql-dialects/README.md:103-200` — typed AST
  translation via `sqlx::QueryBuilder`.

**Evidence:**

`crates/adapters/storage-postgres/src/event_log.rs:135-144` —
  ```rust
  sql.push_str(" AND event_type = ANY($");
  // append the next index (params.len() before this push + 1)
  let idx = params.len();
  // We need to write the index without `format!` to keep
  // the build clippy-clean. Push the digit chars one by
  // one.
  let idx_str = idx.to_string();
  sql.push_str(&idx_str);
  sql.push(')');
  ```

---

### FINDING 25 (id: `ADAPTER-PG-025`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/bulk_attendance.rs:124-193

**Description:**

The bulk-attendance `bulk_insert_into` returns
  `DomainError::conflict` immediately on a unique-key violation
  (line 185-189) without any retry / backoff. Per
  `crates/infra/storage/src/transaction.rs:38-42` the engine
  expects "the engine retries the command automatically" on
  conflicts. No retry policy exists.

**Expected:**

`crates/infra/storage/src/transaction.rs:39-41` —
  "Conflict on a unique-key violation, deadlock, or serialisation
  failure (the engine retries the command automatically)."

**Evidence:**

`crates/adapters/storage-postgres/src/bulk_attendance.rs:182-192` —
  ```rust
  match qb.build().execute(pool).await {
      Ok(_) => Ok(()),
      Err(sqlx::Error::Database(db))
          if db.kind() == sqlx::error::ErrorKind::UniqueViolation =>
      {
          Err(DomainError::conflict(
              "bulk_insert_student_attendances: duplicate (school_id, student_id, attendance_date) row",
          ))
      }
      Err(other) => Err(DomainError::infrastructure(other)),
  }
  ```

---

### FINDING 26 (id: `ADAPTER-PG-026`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** High
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/transaction.rs:122-137

**Description:**

`PostgresTransaction::commit` and
  `PostgresTransaction::rollback` are no-ops. The
  `outbox().append(...)`, `audit_log().append(...)`,
  `event_log().append(...)`, and `idempotency().record(...)` calls
  each open their own short-lived `pool.begin()` inside the
  sub-port method and auto-commit on drop. Between any two of
  these calls, a duplicate dispatch can land in another
  transaction. PHASE-1-HANDOFF.md:38-46 acknowledges this as
  "Open question #1" — the engine's at-least-once dedup is the
  only safety net.

**Expected:**

`docs/ports/storage.md:131-137` —
  "Reads see writes from the same transaction. On commit the
  writes are persisted and the outbox events are released to the
  event bus. On rollback the writes are discarded."

**Evidence:**

`crates/adapters/storage-postgres/src/transaction.rs:122-129` —
  ```rust
  async fn commit(self: Box<Self>) -> Result<()> {
      // No-op: the sub-port operations have already committed
      // via the `sqlx::Transaction` they each acquired.
      self.done.store(true, Ordering::SeqCst);
      Ok(())
  }
  ```

---

### FINDING 27 (id: `ADAPTER-PG-027`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/storage.rs:186-194

**Description:**

`PostgresStorageAdapter::close` calls
  `self.conn.into_inner().close().await` (consuming the
  `PostgresConnection` to get the inner `PgPool`). The
  `sqlx::Pool::close` future returns `()`, but the `await` is
  performed without inspecting any error. The outer
  `Result<()>` is always `Ok(())` — close cannot fail per this
  signature, but the API surface suggests it can.

**Expected:**

`crates/infra/storage/src/port.rs:50-53` —
  "Closes the adapter, releasing all underlying connections.
  After `close`, any further call returns `Err(Infrastructure)`."

**Evidence:**

`crates/adapters/storage-postgres/src/storage.rs:186-194` —
  ```rust
  async fn close(self: Box<Self>) -> Result<()> {
      self.closed.store(true, Ordering::SeqCst);
      self.conn.into_inner().close().await;
      Ok(())
  }
  ```

---

### FINDING 28 (id: `ADAPTER-PG-028`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/storage.rs:119, 131, 173, 206, 227, 238, 254

**Description:**

Every public method on `PostgresStorageAdapter`
  checks `if self.closed.load(...) { return Err(DomainError::conflict(...)) }`
  and returns `DomainError::conflict` (a domain-level conflict
  variant). The port contract at `crates/infra/storage/src/port.rs:52-53`
  states "After `close`, any further call returns
  `Err(Infrastructure)`". Returning `Conflict` on a closed adapter
  is a wrong error variant.

**Expected:**

`crates/infra/storage/src/port.rs:53` —
  "After `close`, any further call returns `Err(Infrastructure)`."

**Evidence:**

`crates/adapters/storage-postgres/src/storage.rs:119-123` —
  ```rust
  if self.closed.load(Ordering::SeqCst) {
      return Err(DomainError::conflict(
          "StorageAdapter::begin called on a closed adapter",
      ));
  }
  ```

---

### FINDING 29 (id: `ADAPTER-PG-029`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/outbox.rs:175-191

**Description:**

`Outbox::pending_count` is a method that exists
  on the trait with a default impl. The adapter overrides it with
  a direct `COUNT(*)` (good), but ignores the `self.school` field
  and accepts any `school_id: SchoolId` argument. Combined with
  finding ADAPTER-PG-013 this means the trait API allows any
  tenant to query any other tenant's pending outbox count.

**Expected:**

Port should validate `school_id` against the
  handle's scope; doc-vs-code drift.

**Evidence:**

`crates/adapters/storage-postgres/src/outbox.rs:175-191` —
  ```rust
  async fn pending_count(&self, school_id: SchoolId) -> Result<u64> {
      // ... the default impl in the trait materialises every
      // pending row just to count them, which is O(n) memory
      // for a 1-line aggregate. Override with a direct
      // `COUNT(*)` for back-pressure sizing.
      let row = sqlx::query(
          "SELECT COUNT(*) AS n FROM outbox WHERE school_id = $1 AND published_at IS NULL",
      )
      .bind(school_id.as_uuid())
      ...
  ```

---

### FINDING 30 (id: `ADAPTER-PG-030`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/audit_log.rs:144

**Description:**

`audit_log.append` instruments with
  `fields(actor = %entry.actor_id, target_type = %entry.target_type)`.
  The `actor_id` (a `UserId`) and `target_type` (a free-form
  aggregate name) are exposed in tracing span fields. Per
  `AGENTS.md` and `docs/code-standards.md` PII (and tenant-scoped
  identifiers) should be filtered from tracing output.

**Expected:**

`docs/code-standards.md` § "PII Logging" (if it
  exists) — tracing spans should redact UserId, CorrelationId,
  and aggregate identifiers.

**Evidence:**

`crates/adapters/storage-postgres/src/audit_log.rs:144` —
  ```rust
  #[instrument(skip(self, entry), fields(actor = %entry.actor_id, target_type = %entry.target_type))]
  ```
  And `crates/adapters/storage-postgres/src/bulk_attendance.rs:101` —
  ```rust
  #[instrument(skip(self, rows), fields(n = rows.len(), school = %self.school))]
  ```
  `school = %self.school` exposes the tenant identifier in spans.

---

### FINDING 31 (id: `ADAPTER-PG-031`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/idempotency.rs:174-206

**Description:**

`Idempotency::record` uses
  `INSERT ... ON CONFLICT (school_id, command_type, idempotency_key)
  DO NOTHING`. The default port impl comment at
  `crates/infra/storage/src/idempotency.rs:94-100` says:
  "Returns `Err(Conflict)` if a record with the same
  `(school_id, command_type, idempotency_key)` already exists with
  a different outcome. Returns `Ok(())` if the record is a no-op
  write (same key, same outcome hash) — the engine uses this for
  at-least-once delivery of retries." The PG adapter conflates
  the "different outcome" case with the "same outcome" case —
  both are silently swallowed as `Ok(())`.

**Expected:**

`crates/infra/storage/src/idempotency.rs:94-100` —
  detect "same key, different outcome" and return
  `DomainError::Conflict`.

**Evidence:**

`crates/adapters/storage-postgres/src/idempotency.rs:188-206` —
  ```rust
  sqlx::query(
      "INSERT INTO idempotency (\
          school_id, command_type, idempotency_key, \
          command_id, outcome, recorded_at, expires_at\
      ) VALUES ($1, $2, $3, $4, $5, $6, $7) \
       ON CONFLICT (school_id, command_type, idempotency_key) DO NOTHING",
  )
  ```

---

### FINDING 32 (id: `ADAPTER-PG-032`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/outbox.rs:160-172

**Description:**

`Outbox::mark_published` uses `ANY($1)` against
  a `Vec<Uuid>` for the IN-list. PG allows up to
  ~32,000 parameters in a single statement; with no per-call cap,
  a single bulk publish call could exceed the limit and produce
  a runtime error.

**Expected:**

`docs/ports/storage.md:188-189` —
  "Timeouts are configurable per adapter."

**Evidence:**

`crates/adapters/storage-postgres/src/outbox.rs:160-172` —
  ```rust
  async fn mark_published(&self, ids: &[EventId]) -> Result<()> {
      if ids.is_empty() {
          return Ok(());
      }
      let id_uuids: Vec<Uuid> = ids.iter().map(|i| i.as_uuid()).collect();
      sqlx::query("UPDATE outbox SET published_at = NOW() WHERE event_id = ANY($1)")
          .bind(&id_uuids)
          .execute(&self.pool)
          ...
  ```

---

### FINDING 33 (id: `ADAPTER-PG-033`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/bulk_attendance.rs:42-44, 132-137

**Description:**

`MAX_ROWS_PER_CALL = 1000` is enforced as a
  hard cap (line 132-137 returns `Validation`). The comment at
  lines 42-44 cites "PostgreSQL caps a single prepared statement
  at 65,535 placeholders; 24 columns × 1,000 rows = 24,000
  placeholders (well under the cap)." With 24 columns × 2730 rows
  the cap would be exceeded — 1000 is a conservative choice but
  no chunking / batch path exists in the adapter. The caller
  must split the input themselves.

**Expected:**

`docs/ports/storage.md:477` —
  "A load test (10k attendance marks in <5s)." A 10k batch would
  require 10 adapter calls.

**Evidence:**

`crates/adapters/storage-postgres/src/bulk_attendance.rs:128-137` —
  ```rust
  if rows.is_empty() {
      return Ok(());
  }
  if rows.len() > MAX_ROWS_PER_CALL {
      return Err(DomainError::validation(format!(
          "bulk_insert_student_attendances: at most {MAX_ROWS_PER_CALL} rows per call, got {}",
          rows.len()
      )));
  }
  ```

---

### FINDING 34 (id: `ADAPTER-PG-034`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/transaction.rs:60-86, 156-167

**Description:**

`PostgresTransaction::bulk_insert_student_attendances`
  uses `self.bulk.school()` as the tenant anchor (line 166),
  while `PostgresStorageAdapter::bulk_insert_student_attendances`
  uses the caller-supplied `ctx.school_id` (storage.rs:260).
  These two paths can disagree on which school is authoritative.
  Additionally, neither path opens a real `sqlx::Transaction`
  for atomic commit with the surrounding outbox / audit appends.

**Expected:**

`docs/ports/storage.md:131-137` — the same
  transaction must own both the bulk insert and the outbox
  append.

**Evidence:**

`crates/adapters/storage-postgres/src/transaction.rs:166` —
  ```rust
  self.bulk.bulk_insert(self.bulk.school(), rows).await
  ```
  And `crates/adapters/storage-postgres/src/storage.rs:259-261` —
  ```rust
  let handle = PostgresBulkAttendance::new(self.conn.db().clone(), self.conn.school());
  handle.bulk_insert(ctx.school_id, rows).await
  ```

---

### FINDING 35 (id: `ADAPTER-PG-035`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/outbox.rs:140-158

**Description:**

`Outbox::pending` uses `LIMIT $2` (line 149)
  bound to `i64::from(limit)` (line 152). The port method takes
  `limit: u32`. Negative limits silently become very large
  positive numbers due to `i64::from(limit)` on a `u32` — but
  since the port passes `u32`, this is safe at the API boundary.
  However, the adapter does not enforce an upper cap on the
  limit value, so a caller could request billions of rows.

**Expected:**

`crates/infra/storage/src/event_log.rs:101-103` —
  "The cap is `filter.limit`; the adapter may enforce a lower cap
  for safety."

**Evidence:**

`crates/adapters/storage-postgres/src/outbox.rs:140-158` —
  ```rust
  async fn pending(&self, limit: u32) -> Result<Vec<SerializedEnvelope>> {
      let rows: Vec<OutboxRow> = sqlx::query_as::<_, OutboxRow>(
          "SELECT ... FROM outbox WHERE school_id = $1 AND published_at IS NULL \
           ORDER BY enqueued_at ASC LIMIT $2",
      )
      .bind(self.school.as_uuid())
      .bind(i64::from(limit))
      ...
  ```

---

### FINDING 36 (id: `ADAPTER-PG-036`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/storage.rs:197-216

**Description:**

`watch_changes` is implemented (rather than
  falling back to the trait's default `NotSupported`) by
  returning an empty `futures::stream::empty` boxed into a
  `ChangeStream`. This silently swallows subscribers — a sync
  client receives no events and no error. The trait default
  returns `NotSupported` so the sync engine "fails loudly at
  startup". The override masks the error.

**Expected:**

`docs/ports/storage.md:112-118` —
  "the sync engine, when it tries to subscribe on a non-sync
  adapter, fails loudly at startup — not silently at runtime".

**Evidence:**

`crates/adapters/storage-postgres/src/storage.rs:196-216` —
  ```rust
  async fn watch_changes(&self, _filter: ChangeFilter) -> Result<ChangeStream> {
      // ... keep the default `NotSupported` behaviour by simply
      // constructing the `ChangeStream` shell and returning it
      // ...
      let s = futures::stream::empty::<
          std::result::Result<educore_storage::change_stream::ChangeEvent, DomainError>,
      >();
      let pinned = Box::pin(s);
      Ok(ChangeStream { inner: pinned })
  }
  ```

---

### FINDING 37 (id: `ADAPTER-PG-037`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/storage.rs:225-235

**Description:**

`cursor_for` returns `VersionCursor::ZERO`
  unconditionally (line 234). `advance_cursor` returns `Ok(())`
  unconditionally (line 245). The sync engine, when it relies on
  these to track per-school replay position, will replay every
  event from scratch on every restart.

**Expected:**

`docs/ports/storage.md:108-110` —
  "The cursor is a per-school monotonically increasing `version`
  (or `event_id`). It's stored in a small engine-internal table;
  the adapter implements the read/write."

**Evidence:**

`crates/adapters/storage-postgres/src/storage.rs:225-235` —
  ```rust
  async fn cursor_for(&self, _school_id: SchoolId) -> Result<VersionCursor> {
      if self.closed.load(Ordering::SeqCst) {
          return Err(DomainError::conflict(
              "StorageAdapter::cursor_for called on a closed adapter",
          ));
      }
      Ok(VersionCursor::ZERO)
  }
  ```

---

### FINDING 38 (id: `ADAPTER-PG-038`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/connection.rs:30-95

**Description:**

`PostgresConnection` does not configure pool
  size, acquire timeout, idle timeout, max lifetime, or statement
  cache. The defaults (`max_connections = 10`, no acquire timeout
  in sqlx 0.8 defaults) are taken. A consumer that opens many
  concurrent commands will exhaust the pool with no graceful
  backpressure.

**Expected:**

`docs/ports/storage.md:418-429` — full builder
  pattern with `.max_connections(20)`, `.acquire_timeout(...)`.

**Evidence:**

`crates/adapters/storage-postgres/src/connection.rs:69-90` —
  ```rust
  let pool = PgPoolOptions::new()
      .after_connect(|conn, _meta| { ... })
      .connect(url)
      .await
      ...
  ```
  No `.max_connections(...)`, `.min_connections(...)`,
  `.acquire_timeout(...)`, or statement cache configuration.

---

### FINDING 39 (id: `ADAPTER-PG-039`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/storage.rs:99-102

**Description:**

The `connect()` constructor doesn't validate the
  URL beyond sqlx's own parser. A consumer that passes
  `postgres://localhost/nonexistent` will fail at the pool
  acquire step with no actionable error mapping. The error path
  is `DomainError::infrastructure(sqlx::Error)` which loses
  the "is the DB reachable? is the URL valid? are credentials
  correct?" diagnostic.

**Expected:**

`crates/infra/storage/src/port.rs:46-48` —
  "Liveness check. Returns `Ok(())` if the adapter is connected
  and responsive; `Err(Infrastructure)` otherwise."

**Evidence:**

`crates/adapters/storage-postgres/src/storage.rs:99-102` —
  ```rust
  pub async fn connect(url: &str, school: SchoolId) -> Result<Self> {
      let conn = PostgresConnection::connect(url, school).await?;
      Ok(Self::new(conn))
  }
  ```

---

### FINDING 40 (id: `ADAPTER-PG-040`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/idempotency.rs:208-221

**Description:**

`purge_older_than` issues
  `DELETE FROM idempotency WHERE school_id = $1 AND recorded_at < $2`
  but does not issue it inside a transaction. A long-running
  delete can take row-level locks for the duration. Combined with
  the per-call transaction model (each sub-port call opens its own
  short transaction), a concurrent insert may serialize against
  the purge.

**Expected:**

`docs/ports/storage.md:209-226` — bulk operations
  on cross-cutting tables should run inside an explicit
  transaction with appropriate batch sizing.

**Evidence:**

`crates/adapters/storage-postgres/src/idempotency.rs:208-221` —
  ```rust
  async fn purge_older_than(&self, school_id: SchoolId, cutoff: Timestamp) -> Result<u64> {
      let row = sqlx::query("DELETE FROM idempotency WHERE school_id = $1 AND recorded_at < $2")
          .bind(school_id.as_uuid())
          .bind(cutoff.as_datetime())
          .execute(&self.pool)
          .await
          .map_err(educore_core::error::DomainError::infrastructure)?;
      ...
  ```

---

### FINDING 41 (id: `ADAPTER-PG-041`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/connection_helpers.rs:20-23

**Description:**

`bytes_to_json_value` silently wraps
  non-JSON-serialisable bytes in `Value::String(...)` (the
  lossy-UTF8 fallback at line 22). A round-trip
  `bytes → JSONB → bytes` is no longer lossless for binary
  payloads; the JSONB CHECK constraint in the dialect spec
  (`jsonb_typeof(...) = 'object'`) would reject this at insert
  time, but the adapter never checks the constraint at insert.

**Expected:**

`docs/ports/storage.md:131-137` —
  lossless round-trip is the contract.

**Evidence:**

`crates/adapters/storage-postgres/src/connection_helpers.rs:18-23` —
  ```rust
  pub fn bytes_to_json_value(bytes: &Bytes) -> Value {
      serde_json::from_slice(bytes.as_ref())
          .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(bytes.as_ref()).into_owned()))
  }
  ```

---

### FINDING 42 (id: `ADAPTER-PG-042`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/storage.rs:130-169

**Description:**

`migrate()` executes the canonical DDL but
  reports a `statements_executed` count derived from
  `SCHEMA_SQL.split(';').filter(|s| !s.trim().is_empty()).count()`.
  This is a naive splitter that over-counts (a semicolon inside a
  string literal or a PL/pgSQL block would inflate the count) and
  under-counts (statements terminated by `;` followed by
  comments). The reported number is meaningless.

**Expected:**

`crates/infra/storage/src/change_stream.rs:249-250` —
  "The number of statements executed (DDL or DML)."

**Evidence:**

`crates/adapters/storage-postgres/src/storage.rs:156-162` —
  ```rust
  let statements_executed = u32::try_from(
      SCHEMA_SQL
          .split(';')
          .filter(|s| !s.trim().is_empty())
          .count(),
  )
  .unwrap_or(0);
  ```

---

### FINDING 43 (id: `ADAPTER-PG-043`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/storage.rs:117

**Description:**

`#[instrument(skip(self))]` on every public
  method but no structured event emission; tracing spans are
  scoped only to the method body. Cross-tenant operations
  (e.g. `cursor_for(school_id)`) log nothing about the tenant.

**Expected:**

`docs/code-standards.md` (Telemetry section if
  any) — structured events for tenant-scoped operations.

**Evidence:**

`crates/adapters/storage-postgres/src/storage.rs:225-235` —
  ```rust
  #[instrument(skip(self, _school_id))]
  async fn cursor_for(&self, _school_id: SchoolId) -> Result<VersionCursor> {
      ...
  ```
  The `_school_id` is explicitly skipped from tracing.

---

### FINDING 44 (id: `ADAPTER-PG-044`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/idempotency.rs:142-171

**Description:**

`Idempotency::lookup` returns the stored
  `command_type` after leaking it via `Box::leak` (finding
  ADAPTER-PG-011). On every read, a new `Box<str>` allocation is
  made and leaked. Under steady-state load (a single consumer
  with N tenants, each issuing K commands/hour, each retried
  once), the leak rate is `2NK` bytes/hour.

**Expected:**

`crates/infra/storage/src/idempotency.rs:31` —
  the port uses `&'static str`; the column type is `VARCHAR`.
  The port should change to `String` to avoid the leak.

**Evidence:**

`crates/adapters/storage-postgres/src/idempotency.rs:159-167` —
  ```rust
  let (payload, version, agg_ids) = unwrap_envelope(&r.outcome.0);
  Ok(Some(IdempotencyRecord {
      ...
      command_type: lookup_command_type(&r.command_type),
      ...
  }))
  ```

---

### FINDING 45 (id: `ADAPTER-PG-045`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/connection.rs:40-46

**Description:**

`PostgresConnection::fmt::Debug` omits the
  connection URL and pool stats, but also omits the
  `closed`-ness / readiness state of the underlying pool. A
  debugging session cannot tell from `Debug` whether the pool is
  exhausted or healthy.

**Expected:**

`crates/infra/storage/src/port.rs:28-34` —
  Object-safe, `Send + Sync`, `Debug`-able adapters.

**Evidence:**

`crates/adapters/storage-postgres/src/connection.rs:40-46` —
  ```rust
  impl fmt::Debug for PostgresConnection {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
          f.debug_struct("PostgresConnection")
              .field("school", &self.school)
              .finish_non_exhaustive()
      }
  }
  ```
  Only `school` is exposed; `closed`, pool size, in-flight
  connection count are all absent.

---

### FINDING 46 (id: `ADAPTER-PG-046`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/outbox.rs:139-158

**Description:**

`Outbox::pending` selects only 12 columns
  (event_id, event_type, event_version, school_id, aggregate_id,
  aggregate_type, actor_id, correlation_id, causation_id,
  occurred_at, payload) and does not read `recorded_at`,
  `enqueued_at`, `published_at`, `attempts`, or `last_error`. A
  consumer that wants to size the relay or back off on
  repeatedly-failing rows cannot read those columns without a
  separate query.

**Expected:**

`docs/schemas/event-schema.md` § 8 (referenced
  in DDL line 53).

**Evidence:**

`crates/adapters/storage-postgres/src/outbox.rs:139-158` —
  ```rust
  let rows: Vec<OutboxRow> = sqlx::query_as::<_, OutboxRow>(
      "SELECT \
          event_id, event_type, event_version, school_id, \
          aggregate_id, aggregate_type, actor_id, \
          correlation_id, causation_id, occurred_at, payload \
       FROM outbox \
       ...
  ```

---

### FINDING 47 (id: `ADAPTER-PG-047`)

- **Source:** `docs/audit_reports/findings/wave3-storage-postgres.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** crates/adapters/storage-postgres/src/bulk_attendance.rs:43-44

**Description:**

The comment claims
  `24 columns × 1,000 rows = 24,000 placeholders (well under the cap)`,
  but the actual INSERT column list (line 148-152) is 24 columns
  × N rows where N is `rows.len()`. With `MAX_ROWS_PER_CALL = 1000`,
  the placeholder count is `24 × 1000 = 24000`. PG's
  `MAX_BINNED_TYPES` / parameter limit (per sqlx 0.8 docs) is
  `u16::MAX = 65535`. The 24k figure is correct, but the comment
  does not mention that `?` placeholders are used twice (one for
  VALUES, one for column list) — i.e. the bound parameter count
  is `2 × 24 × N = 48 × N`. With N=1000, that's 48000 placeholders.
  The comment understates by 2x.

**Expected:**

`docs/schemas/sql-dialects/postgresql.md` and
  sqlx 0.8 documentation on parameter limits.

**Evidence:**

`crates/adapters/storage-postgres/src/bulk_attendance.rs:42-44` —
  ```rust
  /// The per-call row cap. PostgreSQL caps a single prepared
  /// statement at 65,535 placeholders; 24 columns × 1,000 rows
  /// = 24,000 placeholders (well under the cap).
  pub(crate) const MAX_ROWS_PER_CALL: usize = 1000;
  ```

---


## Storage — MySQL (target id prefix: `ADAPT-MY`)

**Path:** `crates/adapters/storage-mysql/`  
**Total findings:** 24 (5 critical, 7 high, 9 medium, 3 low)


### FINDING 1 (id: `ADAPT-MY-001`)

- **Source:** `docs/audit_reports/findings/wave3-storage-mysql.md`
- **Severity:** Critical
- **Area:** adapters-storage-mysql
- **Location:** `crates/adapters/storage-mysql/src/storage.rs:117-160`

**Description:**

The `migrate()` implementation only executes
  `migrations/engine/0000_engine_core.mysql.sql` (6 engine
  cross-cutting tables) plus `MysqlBulkAttendance::ensure_schema`
  (1 attendance domain table). It does **not** walk any
  macro-emitted AST to emit the ~310 domain tables the engine
  claims to ship, and it does not honour the dialect spec's
  per-table `SqlStorageAdapter::create_<table>_ddl()` contract.
  `Schema-registry` and `system_user` are emitted; no domain
  tables are emitted.

**Expected:**

"The schema is emitted by the storage adapter at
  startup via `storage.create_schema().await`. … 4. Adapter
  emission — `educore-storage-<db>` walks the AST at
  schema-creation time and emits the dialect-specific DDL
  string. … 5. Consumer startup — `storage.create_schema().await`
  runs the DDL once per process lifetime."
  (`docs/schemas/sql-dialects/README.md` § "Runtime DDL emission
  — end-to-end flow").

**Evidence:**

```rust
  // crates/adapters/storage-mysql/src/storage.rs:130-143
  sqlx::raw_sql(SCHEMA_SQL)
      .execute(self.conn.db())
      .await
      .map_err(DomainError::infrastructure)?;
  MysqlBulkAttendance::new(self.conn.db().clone(), self.conn.school())
      .ensure_schema()
      .await?;
  ```
  `crates/adapters/storage-mysql/src/storage.rs:58` —
  `const SCHEMA_SQL: &str = include_str!("../../../../migrations/engine/0000_engine_core.mysql.sql");`
  The crate has no `create_schema()`, no AST walk, and no
  domain-table emission code anywhere under `src/`.

---

### FINDING 2 (id: `ADAPT-MY-002`)

- **Source:** `docs/audit_reports/findings/wave3-storage-mysql.md`
- **Severity:** Critical
- **Area:** adapters-storage-mysql
- **Location:** `crates/adapters/storage-mysql/src/storage.rs:125` (`migrate` method signature)

**Description:**

The adapter exposes a `migrate()` method on
  `StorageAdapter`, but every consumer-facing doc
  (`AGENTS.md:544, 561`, `README.md:173`,
  `docs/schemas/sql-dialects/README.md:193-198`,
  `docs/schemas/sql-dialects/mysql.md:9`,
  `docs/build-plan.md:119, 175-179, 186`,
  `docs/architecture.md:322`,
  `migrations/engine/README.md:11`,
  `CONTRIBUTING.md:502`) refers to the runtime entry point as
  `storage.create_schema().await`. The consumer-facing API name
  does not exist on the trait.

**Expected:**

`docs/build-plan.md:175-179` —
  `("create_schema", "apply_command", "query", "begin_tx", ...)`
  and `storage.create_schema().await` runs the DDL.

**Evidence:**

`crates/adapters/storage-mysql/src/storage.rs:125`
  ```rust
  async fn migrate(&self) -> Result<MigrationReport> {
  ```
  And `crates/infra/storage/src/port.rs:44`:
  ```rust
  async fn migrate(&self) -> Result<MigrationReport>;
  ```
  No `create_schema` method exists anywhere in the MySQL crate
  (`grep -rn "fn create_schema" crates/adapters/storage-mysql/`
  returns no results).

---

### FINDING 3 (id: `ADAPT-MY-003`)

- **Source:** `docs/audit_reports/findings/wave3-storage-mysql.md`
- **Severity:** Critical
- **Area:** adapters-storage-mysql
- **Location:** `crates/adapters/storage-mysql/src/transaction.rs:107-117` (`commit`)

**Description:**

`MysqlTransaction::commit` is a documented no-op:
  the sub-port operations have already committed via their
  own `pool.begin()` calls. The port contract
  (`docs/ports/storage.md` § Transactions) requires atomic
  commit semantics across all four sub-port calls inside a
  single `Transaction`; the MySQL adapter delivers
  "commit-per-sub-port" semantics, so a crash between
  `outbox.append` and `audit_log.record` leaves the system in
  a torn state. This is a silent relaxation of the ACID
  contract.

**Expected:**

"A `Transaction` groups one or more sub-port
  writes into an atomic unit; `commit` makes them visible
  together, `rollback` discards them all." (`docs/ports/storage.md`
  § Transactions).

**Evidence:**

`crates/adapters/storage-mysql/src/transaction.rs:107-112`
  ```rust
  async fn commit(self: Box<Self>) -> Result<()> {
      // No-op: the sub-port operations have already committed
      // via the `sqlx::Transaction` they each acquired. We
      // only flip the guard flag.
      self.done.store(true, Ordering::SeqCst);
      Ok(())
  }
  ```

---

### FINDING 4 (id: `ADAPT-MY-004`)

- **Source:** `docs/audit_reports/findings/wave3-storage-mysql.md`
- **Severity:** Critical
- **Area:** adapters-storage-mysql
- **Location:** `crates/adapters/storage-mysql/src/outbox.rs:139-175` (`append`)

**Description:**

`Outbox::append` calls a plain
  `INSERT INTO outbox ...` and surfaces the underlying
  `sqlx::Error` (which includes the duplicate-key violation on
  `event_id`) as `DomainError::Infrastructure`. The port
  contract requires `DomainError::Conflict` on a duplicate
  `(school_id, event_id)`. The adapter silently downgrades a
  contract-mandated domain error to an infrastructure error.

**Expected:**

"`Conflict` if an envelope with the same
  `event_id` was already appended in the same school."
  (`crates/infra/storage/src/outbox.rs:99-101`).

**Evidence:**

The full method body in
  `crates/adapters/storage-mysql/src/outbox.rs:139-175` uses
  `sqlx::query::<sqlx::MySql>("INSERT INTO outbox ( ...")...`
  with `.execute(&self.pool).await.map_err(|e| ...)?;` and no
  `match` on `sqlx::Error::Database(db)` to map `Duplicate` /
  unique-key violations to `DomainError::conflict(...)`.

---

### FINDING 5 (id: `ADAPT-MY-005`)

- **Source:** `docs/audit_reports/findings/wave3-storage-mysql.md`
- **Severity:** Critical
- **Area:** adapters-storage-mysql
- **Location:** `crates/adapters/storage-mysql/src/idempotency.rs:123-152` (`record`)

**Description:**

The `Idempotency::record` implementation uses
  `INSERT INTO idempotency ... ON DUPLICATE KEY UPDATE command_id
  = VALUES(command_id)`, which silently overwrites any prior
  row with the same composite key and discards the previous
  `command_id`. The port contract requires `DomainError::Conflict`
  when a record with the same composite key exists with a
  different outcome, and `Ok(())` only when the new row is
  identical. The current behaviour violates both halves: it
  never returns `Conflict`, and it overwrites regardless of
  outcome equality.

**Expected:**

"Stores `record`. Returns `Err(Conflict)` if a
  record with the same `(school_id, command_type,
  idempotency_key)` already exists with a different outcome.
  Returns `Ok(())` if the record is a no-op write (same key,
  same outcome hash) — the engine uses this for at-least-once
  delivery of retries." (`crates/infra/storage/src/idempotency.rs:94-100`).

**Evidence:**

`crates/adapters/storage-mysql/src/idempotency.rs:123-152`
  uses `INSERT INTO idempotency ... ON DUPLICATE KEY UPDATE
  command_id = VALUES(command_id)` with no `outcome` comparison
  and no `Conflict` return path. The Postgres adapter (per
  `wave3-storage-postgres.md`) has the same gap; the SQLite
  adapter at `crates/adapters/storage-sqlite/src/idempotency.rs:134-149`
  documents it as `INSERT OR REPLACE`. Both are wrong.

---

### FINDING 16 (id: `ADAPT-MY-016`)

- **Source:** `docs/audit_reports/findings/wave3-storage-mysql.md`
- **Severity:** High
- **Area:** adapters-storage-mysql
- **Location:** `crates/adapters/storage-mysql/src/idempotency.rs:266-274` (`lookup_command_type`)

**Description:**

`lookup_command_type` calls `Box::leak` on every
  unique `command_type` value read from the `idempotency` table.
  The function is called from `lookup` for every record returned.
  Every new `command_type` value encountered allocates memory
  that is never freed for the lifetime of the process. On a
  long-running relay that drains the event log and re-reads
  historical idempotency records for many distinct command
  types, this is an unbounded, process-level memory leak.

**Expected:**

"Adapter-owned types MUST NOT introduce
  unbounded leaks; use `Arc<str>` or a small per-connection
  cache instead." (`docs/code-standards.md` § "Memory
  safety").

**Evidence:**

`crates/adapters/storage-mysql/src/idempotency.rs:266-274`
  ```rust
  fn lookup_command_type(s: &str) -> &'static str {
      let boxed: Box<str> = Box::from(s);
      Box::leak(boxed)
  }
  ```
  No cache, no `Arc<str>`. The Postgres and SQLite adapters
  have the same defect.

---

### FINDING 17 (id: `ADAPT-MY-017`)

- **Source:** `docs/audit_reports/findings/wave3-storage-mysql.md`
- **Severity:** High
- **Area:** adapters-storage-mysql
- **Location:** `crates/adapters/storage-mysql/src/connection_helpers.rs:18-25` (`bytes_to_json_value`)

**Description:**

`bytes_to_json_value` silently wraps
  invalid JSON in `Value::String(...)` on parse failure. A
  payload that is supposed to be a JSON object is stored as a
  MySQL JSON string column containing the raw bytes. On
  read, the round-trip looks correct (the column comes back
  as a `Value::String`), but downstream consumers that
  pattern-match on `Value::Object` will silently fall through
  to a default branch. The adapter has no way to detect the
  corruption after the fact, and there is no test that
  exercises the malformed-payload path.

**Expected:**

"Storage adapters MUST reject malformed
  payloads with `DomainError::Validation` rather than
  silently coercing them." (`docs/specs/event-schema.md` §
  "Payload integrity").

**Evidence:**

`crates/adapters/storage-mysql/src/connection_helpers.rs:18-25`
  ```rust
  pub fn bytes_to_json_value(bytes: &Bytes) -> Value {
      serde_json::from_slice(bytes.as_ref())
          .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(bytes.as_ref()).into_owned()))
  }
  ```
  Compare `outbox.rs:166-167` which calls
  `bytes_to_json_value(&envelope.payload)` — any malformed
  payload becomes a JSON string column.

---

### FINDING 18 (id: `ADAPT-MY-018`)

- **Source:** `docs/audit_reports/findings/wave3-storage-mysql.md`
- **Severity:** High
- **Area:** adapters-storage-mysql
- **Location:** `crates/adapters/storage-mysql/src/bulk_attendance.rs:50-58` (`ensure_schema` — no `SET FOREIGN_KEY_CHECKS` wrapper)

**Description:**

`bulk_attendance.sql` is loaded via
  `sqlx::raw_sql(SCHEMA_SQL)` without a `SET
  FOREIGN_KEY_CHECKS=0` / `=1` wrapper. The cross-cutting
  DDL uses this wrapper for idempotent re-runs (the
  `include_str!`'d file wraps every CREATE in the wrapper).
  The bulk-attendance DDL does not, so a re-migrate against
  a partially-migrated database (e.g. table created but
  unique index missing) will fail on the second migration
  with `Duplicate key name`. The `migrate()` method is
  documented as idempotent; this contradicts that.

**Expected:**

"Every DDL script that runs via
  `sqlx::raw_sql` MUST be wrapped in `SET
  FOREIGN_KEY_CHECKS=0; ... ; SET FOREIGN_KEY_CHECKS=1;`
  for idempotent re-runs."
  (`docs/schemas/sql-dialects/mysql.md` § "Migration
  safety").

**Evidence:**

The cross-cutting DDL at
  `migrations/engine/0000_engine_core.mysql.sql:43` opens
  with `SET FOREIGN_KEY_CHECKS=0;` and closes with `SET
  FOREIGN_KEY_CHECKS=1;` at line 215. The
  `crates/adapters/storage-mysql/src/bulk_attendance.rs:50`
  schema file does not contain this wrapper.

---

### FINDING 6 (id: `ADAPT-MY-006`)

- **Source:** `docs/audit_reports/findings/wave3-storage-mysql.md`
- **Severity:** High
- **Area:** adapters-storage-mysql
- **Location:** `crates/adapters/storage-mysql/src/storage.rs:201-213` (`watch_changes`)

**Description:**

`watch_changes` returns a `ChangeStream` backed
  by `futures::stream::empty()`, which yields zero events
  immediately. The port contract requires a live, push-based
  change feed that yields `(event_log_id, payload_json)` for
  every row appended to the outbox after the caller's cursor.
  The current behaviour silently breaks every offline / sync
  client that subscribes via this port. The code comment
  honestly states "Phase 1: not yet implemented" — this is a
  documented gap, not an accident — but it is still a High
  blocker for any sync engine consumer.

**Expected:**

"Returns a `ChangeStream` that yields one
  `ChangeEvent` per row appended to the outbox after the
  caller's cursor, scoped to the caller's school." (`docs/ports/storage.md`
  § `watch_changes`).

**Evidence:**

`crates/adapters/storage-mysql/src/storage.rs:201-213`
  ```rust
  let s = futures::stream::empty::<
      std::result::Result<educore_storage::change_stream::ChangeEvent, DomainError>,
  >();
  let pinned = Box::pin(s);
  Ok(ChangeStream { inner: pinned })
  ```

---

### FINDING 7 (id: `ADAPT-MY-007`)

- **Source:** `docs/audit_reports/findings/wave3-storage-mysql.md`
- **Severity:** High
- **Area:** adapters-storage-mysql
- **Location:** `crates/adapters/storage-mysql/src/transaction.rs:139-148` (`bulk_insert_student_attendances` on `Transaction`)

**Description:**

The `Transaction` impl does not pass
  `TenantContext` (the `ctx.school_id` + role + actor) into the
  bulk-insert path. The bare `MysqlStorageAdapter::bulk_insert_student_attendances`
  method (at `storage.rs:265-273`) accepts a `&TenantContext`,
  but the `Transaction::bulk_insert_student_attendances` method
  at `transaction.rs:139-148` does **not** accept a context
  argument and the trait port (`crates/infra/storage/src/transaction.rs`)
  only passes `&[StudentAttendanceRow]`. The tenant check is
  therefore a no-op on the transaction path: any caller can
  attempt to insert rows for a `school_id` that differs from the
  transaction's scoped school, and the only check is the
  per-row `school_id == self.bulk.school()` comparison inside
  `bulk_insert`, which silently **drops** mismatched rows
  rather than erroring.

**Expected:**

"Every write must be scoped to the caller's
  `TenantContext.school_id`; a request to write rows for a
  different school must be rejected with `DomainError::Forbidden`."
  (`docs/specs/tenancy-schema.md` § "Tenant isolation
  invariants").

**Evidence:**

`crates/adapters/storage-mysql/src/transaction.rs:139-148`
  ```rust
  async fn bulk_insert_student_attendances(&self, rows: &[StudentAttendanceRow]) -> Result<()> {
      ...
      self.bulk.bulk_insert(self.bulk.school(), rows).await
  }
  ```
  Compare with `crates/adapters/storage-mysql/src/storage.rs:265-273`,
  which accepts `ctx: &TenantContext` and uses `ctx.school_id`.

---

### FINDING 8 (id: `ADAPT-MY-008`)

- **Source:** `docs/audit_reports/findings/wave3-storage-mysql.md`
- **Severity:** High
- **Area:** adapters-storage-mysql
- **Location:** `migrations/engine/0000_engine_core.mysql.sql` (entire file, 6 cross-cutting tables)

**Description:**

The canonical MySQL DDL the adapter
  `include_str!`'s declares no tenant-isolation predicate, no
  `school_id` index, and no FK constraint on any of the 6
  cross-cutting tables (`outbox`, `audit_log`, `idempotency`,
  `event_log`, `schema_registry`, `system_user`). Per
  `docs/schemas/tenancy-schema.md`, every multi-tenant table
  must have a `school_id` index and a NOT-NULL `school_id`
  column; the DDL declares `school_id` columns but is missing
  the index. Without an index, every per-tenant query is a
  full-table scan, and accidental cross-tenant joins silently
  return wrong data.

**Expected:**

"`school_id BIGINT UNSIGNED NOT NULL` plus
  `INDEX idx_<table>_school (school_id)` on every multi-tenant
  table; composite indexes where the access pattern warrants
  it." (`docs/schemas/tenancy-schema.md` § "Per-tenant indexes").

**Evidence:**

A search of
  `migrations/engine/0000_engine_core.mysql.sql` for `INDEX` /
  `KEY` returns only the `event_log` lookup indexes and the
  PK definitions — no `idx_<table>_school` index on any of
  the 6 tables.

---

### FINDING 9 (id: `ADAPT-MY-009`)

- **Source:** `docs/audit_reports/findings/wave3-storage-mysql.md`
- **Severity:** High
- **Area:** adapters-storage-mysql
- **Location:** `crates/adapters/storage-mysql/src/storage.rs:233-245` (`apply_snapshot`)

**Description:**

`apply_snapshot` returns
  `DomainError::not_supported("...is not yet implemented
  (Phase 1)")`. The port contract (`docs/ports/storage.md` §
  "Snapshots") requires the adapter to upsert every aggregate
  in the snapshot into the corresponding domain tables within
  a single transaction; without it, an offline client cannot
  re-bootstrap its local state after a wipe, and the sync
  engine's cold-start path is broken.

**Expected:**

"`apply_snapshot(snapshot)` writes every
  aggregate in `snapshot` to the corresponding domain table
  atomically; existing rows for the same primary key are
  overwritten." (`docs/ports/storage.md` § Snapshots).

**Evidence:**

`crates/adapters/storage-mysql/src/storage.rs:233-235`
  ```rust
  async fn apply_snapshot(&self, _snapshot: SchoolSnapshot) -> Result<()> {
      Err(DomainError::not_supported(
          "MysqlStorageAdapter::apply_snapshot is not yet implemented (Phase 1)",
      ))
  }
  ```

---

### FINDING 10 (id: `ADAPT-MY-010`)

- **Source:** `docs/audit_reports/findings/wave3-storage-mysql.md`
- **Severity:** Medium
- **Area:** adapters-storage-mysql
- **Location:** `crates/adapters/storage-mysql/src/outbox.rs:166-167` (`append`)

**Description:**

`recorded_at` is bound to
  `envelope.occurred_at.as_datetime()` instead of
  `Utc::now()`. The DDL declares `recorded_at` as the
  persistence time (a separate column from `occurred_at`),
  and the engine invariant is that `recorded_at >= occurred_at`
  (it captures ingestion latency). Binding both to the same
  value obliterates that invariant, exactly as the SQLite
  adapter does.

**Expected:**

"`recorded_at` — Wall-clock time of the
  persistence (≥ `occurred_at`)"
  (`crates/infra/storage/src/event_log.rs:73`) and the
  outbox DDL column pair `occurred_at ... recorded_at ...`
  (`migrations/engine/0000_engine_core.mysql.sql:74-75`).

**Evidence:**

`crates/adapters/storage-mysql/src/outbox.rs:166-169`
  ```rust
  .bind(envelope.occurred_at.as_datetime()) // occurred_at
  .bind(envelope.occurred_at.as_datetime()) // recorded_at <- BUG
  ```
  Compare `audit_log.rs:151-152` which correctly binds
  `entry.occurred_at.as_datetime()` and
  `recorded_at = Utc::now()` separately.

---

### FINDING 11 (id: `ADAPT-MY-011`)

- **Source:** `docs/audit_reports/findings/wave3-storage-mysql.md`
- **Severity:** Medium
- **Area:** adapters-storage-mysql
- **Location:** `crates/adapters/storage-mysql/src/outbox.rs:80-110` (`Outbox::append` — payload handling)

**Description:**

The `Outbox::append` method binds
  `payload_json` directly to the `payload` JSON column. Per
  `docs/schemas/sql-dialects/mysql.md:88-95`, MySQL JSON
  columns require the payload to be a canonical JSON string
  with no trailing whitespace; the adapter does **not**
  canonicalise. A `serde_json::to_string` call returning
  `{"a":1, "b":2}` (with embedded space) will round-trip
  through MySQL's JSON normaliser, but a payload containing
  unicode-escape sequences in a different order will be
  silently re-canonicalised on read, breaking idempotent
  hash comparisons downstream. There is no
  `serde_json::to_string` / `serde_path_to_error` validation
  before the bind.

**Expected:**

"Use `serde_json::to_vec` for binary safety;
  canonicalise via `serde_json::value::to_value` + re-serialise
  with sorted keys before binding." (`docs/schemas/sql-dialects/mysql.md`
  § "JSON columns").

**Evidence:**

`crates/adapters/storage-mysql/src/outbox.rs:104-108`
  ```rust
  let payload_json = serde_json::to_string(&envelope.payload)
      .map_err(|e| DomainError::validation(format!("outbox payload: {e}")))?;
  ```
  No canonicalisation step. The Postgres adapter has the same
  gap (per `wave3-storage-postgres.md`).

---

### FINDING 12 (id: `ADAPT-MY-012`)

- **Source:** `docs/audit_reports/findings/wave3-storage-mysql.md`
- **Severity:** Medium
- **Area:** adapters-storage-mysql
- **Location:** `crates/adapters/storage-mysql/tests/outbox_e2e.rs` (82 lines, single test)

**Description:**

The crate ships a single integration test file
  with one `#[tokio::test]` exercising `Outbox::append`
  against a live MySQL connection (env-gated). The audit
  questions require integration coverage for: `audit_log`,
  `event_log`, `idempotency` conflict vs no-op, `apply_snapshot`,
  `watch_changes`, `bulk_insert_student_attendances`, and
  cross-adapter parity (the testkit crate at
  `crates/tools/storage-parity/` per `docs/build-plan.md`
  Phase 1 deliverable). None of these scenarios have a test.
  Per `docs/build-plan.md:175-179` the Phase 1 deliverable is
  "Adapter parity — same trait conformance for MySQL, Postgres,
  SQLite"; a single outbox test is insufficient.

**Expected:**

"`storage-parity` test suite runs every
  `StorageAdapter` + sub-port + `Transaction` method through
  every shipped adapter and asserts identical behaviour."
  (`docs/build-plan.md` Phase 1).

**Evidence:**

```bash
  $ find crates/adapters/storage-mysql -name "*.rs" -path "*test*"
  crates/adapters/storage-mysql/tests/outbox_e2e.rs
  ```
  And `wc -l crates/adapters/storage-mysql/tests/outbox_e2e.rs`
  returns `82` (single test).

---

### FINDING 13 (id: `ADAPT-MY-013`)

- **Source:** `docs/audit_reports/findings/wave3-storage-mysql.md`
- **Severity:** Medium
- **Area:** adapters-storage-mysql
- **Location:** `crates/adapters/storage-mysql/src/connection.rs:1-226`

**Description:**

The connection module's `after_connect` hook
  issues `SET NAMES utf8mb4 COLLATE utf8mb4_unicode_ci`, but
  this fires **only** on initial pool creation. sqlx 0.8 does
  not run `after_connect` on connections borrowed from the
  pool after the first N connections; it only runs on
  *initial* connections. Any connection established after the
  pool's `min_connections` threshold is reached inherits the
  MySQL server's default collation
  (`utf8mb4_0900_ai_ci` on MySQL 8.0+), which is
  accent-insensitive. Tenant-data sorting becomes
  non-deterministic across the pool.

**Expected:**

"Issue `SET NAMES utf8mb4 COLLATE
  utf8mb4_unicode_ci` on every connection acquisition, not
  only on pool creation." (`docs/schemas/sql-dialects/mysql.md`
  § "Connection-level collation").

**Evidence:**

`crates/adapters/storage-mysql/src/connection.rs`
  `after_connect` hook is registered once at pool build time;
  there is no `pool.acquire()` wrapper or `Executor::execute`
  hook on every borrowed connection. Compare with the
  PostgreSQL adapter (per `wave3-storage-postgres.md`) which
  correctly fires `SET search_path` on every connection via
  `acquire`.

---

### FINDING 14 (id: `ADAPT-MY-014`)

- **Source:** `docs/audit_reports/findings/wave3-storage-mysql.md`
- **Severity:** Medium
- **Area:** adapters-storage-mysql
- **Location:** `crates/adapters/storage-mysql/src/storage.rs:140` (`migrate` — `multi_statements` requirement)

**Description:**

`migrate()` calls `sqlx::raw_sql(SCHEMA_SQL)`
  which requires the connection URL to carry
  `multi_statements=true`. The `MysqlConnection::connect`
  helper appends the parameter if missing — but **only** when
  the URL contains no existing query string. If the URL ends
  in `?ssl-mode=REQUIRED` (a common production pattern with
  `rustls`), the helper appends `?multi_statements=true`
  instead of `&multi_statements=true`, producing an
  unparseable URL and a pool creation error.

**Expected:**

"The connector MUST splice
  `multi_statements=true` into the URL's query string with
  `&` when an existing parameter is present, with `?`
  otherwise." (`docs/schemas/sql-dialects/mysql.md` §
  "Multi-statement DDL").

**Evidence:**

`crates/adapters/storage-mysql/src/connection_helpers.rs`
  has 55 lines; the `multi_statements` injection logic
  checks for `?` but does not parse the URL with the `url`
  crate (per the SQLite / Postgres equivalents which use the
  `url::Url` parser). The Postgres adapter's
  `crates/adapters/storage-postgres/src/connection.rs` uses
  `url::Url::parse` and `query_pairs_mut`.

---

### FINDING 19 (id: `ADAPT-MY-019`)

- **Source:** `docs/audit_reports/findings/wave3-storage-mysql.md`
- **Severity:** Medium
- **Area:** adapters-storage-mysql
- **Location:** `crates/adapters/storage-mysql/src/outbox.rs:177-191` (`pending_count` — overflow trap)

**Description:**

`pending_count` casts `i64` to `u64` via
  `u64::try_from(n).unwrap_or(0)`. MySQL's `COUNT(*)` returns
  `i64::MAX` as the maximum representable count; on a
  hypothetical database with more than `i64::MAX` pending
  rows (effectively impossible in practice but contractually
  significant), the function silently returns `0` instead of
  propagating `DomainError::Infrastructure`. The same
  pattern is repeated in `mark_published` (rows_affected
  cast) and `purge_older_than` (`i64::MAX` fallback).

**Expected:**

"Numeric overflow MUST propagate via
  `DomainError::Infrastructure`, not be silently coerced to
  zero or `i64::MAX`." (`docs/code-standards.md` § "Numeric
  conversions").

**Evidence:**

`crates/adapters/storage-mysql/src/outbox.rs:177-191`
  ```rust
  let n: i64 = row.try_get("n")...?;
  Ok(u64::try_from(n).unwrap_or(0))
  ```
  `crates/adapters/storage-mysql/src/idempotency.rs:218-219`
  `let n: i64 = row.rows_affected().try_into().unwrap_or(i64::MAX);`
  `crates/adapters/storage-mysql/src/idempotency.rs:220`
  `Ok(u64::try_from(n).unwrap_or(0))` — the `i64::MAX`
  fallback is then silently clamped to `0` on the second
  `unwrap_or`.

---

### FINDING 20 (id: `ADAPT-MY-020`)

- **Source:** `docs/audit_reports/findings/wave3-storage-mysql.md`
- **Severity:** Medium
- **Area:** adapters-storage-mysql
- **Location:** `crates/adapters/storage-mysql/src/storage.rs:201-213` (`watch_changes` — filter ignored)

**Description:**

`watch_changes` ignores its `ChangeFilter`
  argument (the parameter is bound to `_filter`). The
  function returns an empty `ChangeStream` regardless of
  what `ChangeFilter` the caller passes (school, event
  types, since-cursor, etc.). Even when a future PR
  implements live streaming, the contract that "the filter
  is honoured" is not enforced at the type level.

**Expected:**

"`watch_changes(filter)` returns a stream
  that yields one `ChangeEvent` per outbox row matching
  `filter.school_id`, `filter.event_types`,
  `filter.since`, etc." (`docs/ports/storage.md` § Sync
  primitives).

**Evidence:**

`crates/adapters/storage-mysql/src/storage.rs:201-213`
  ```rust
  async fn watch_changes(&self, _filter: ChangeFilter) -> Result<ChangeStream> {
      ...
      let s = futures::stream::empty::<...>();
      let pinned = Box::pin(s);
      Ok(ChangeStream { inner: pinned })
  }
  ```
  The `ChangeFilter` is bound but never read.

---

### FINDING 21 (id: `ADAPT-MY-021`)

- **Source:** `docs/audit_reports/findings/wave3-storage-mysql.md`
- **Severity:** Medium
- **Area:** adapters-storage-mysql
- **Location:** `crates/adapters/storage-mysql/src/transaction.rs:139-148` (`bulk_insert_student_attendances` — tenant check missing)

**Description:**

The `Transaction::bulk_insert_student_attendances`
  method passes `self.bulk.school()` as the `school_id`
  argument to `bulk_insert_into`, which validates that
  every row's `school_id` matches. This is correct for the
  scoped school check — but the trait method does **not**
  also validate that the caller's `TenantContext` matches
  the transaction's scoped school (the
  `StorageAdapter::bulk_insert_student_attendances`
  implementation does, because it accepts `ctx`; the
  `Transaction` impl cannot, because the trait method does
  not pass `ctx`). A relay or saga that runs on a
  transaction scoped to school A but is misconfigured to
  read tenant context from school B will silently insert
  rows for school A.

**Expected:**

"Every write path that accepts a
  `TenantContext` MUST validate the context's `school_id`
  matches the transaction's scoped school before delegating
  to the bulk-insert helper." (`docs/specs/tenancy-schema.md`
  § "Per-transaction tenant checks").

**Evidence:**

`crates/adapters/storage-mysql/src/transaction.rs:139-148`
  ```rust
  async fn bulk_insert_student_attendances(&self, rows: &[StudentAttendanceRow]) -> Result<()> {
      ...
      self.bulk.bulk_insert(self.bulk.school(), rows).await
  }
  ```
  Compare `crates/infra/storage/src/transaction.rs:85-91` —
  the trait method signature is
  `async fn bulk_insert_student_attendances(&self, rows: &[StudentAttendanceRow])`,
  no `ctx` parameter; the engine-level check is impossible
  on this path.

---

### FINDING 22 (id: `ADAPT-MY-022`)

- **Source:** `docs/audit_reports/findings/wave3-storage-mysql.md`
- **Severity:** Medium
- **Area:** adapters-storage-mysql
- **Location:** `crates/adapters/storage-mysql/src/storage.rs:218-223` (`cursor_for` — never advances)

**Description:**

`cursor_for` returns `VersionCursor::ZERO`
  unconditionally. `advance_cursor` is a no-op (accepts the
  cursor, sets the closed guard, returns `Ok(())`). The
  sync engine, when wired to this adapter, will read cursor
  `0` for every school and "advance" to whatever cursor
  the caller asked for — but the cursor is not persisted
  anywhere. A restart of the adapter process resets the
  cursor to zero, so the sync engine re-delivers every
  outbox event on every restart.

**Expected:**

"`cursor_for` reads the persisted cursor
  from a `sync_state` table; `advance_cursor` writes it."
  (`docs/ports/sync.md` § "Cursor persistence").

**Evidence:**

`crates/adapters/storage-mysql/src/storage.rs:218-223`
  ```rust
  async fn cursor_for(&self, _school_id: SchoolId) -> Result<VersionCursor> {
      ...
      Ok(VersionCursor::ZERO)
  }
  ```
  And `crates/adapters/storage-mysql/src/storage.rs:226-233`
  ```rust
  async fn advance_cursor(&self, _school_id: SchoolId, _to: VersionCursor) -> Result<()> {
      // Phase 1 stub.
      Ok(())
  }
  ```

---

### FINDING 15 (id: `ADAPT-MY-015`)

- **Source:** `docs/audit_reports/findings/wave3-storage-mysql.md`
- **Severity:** Low
- **Area:** adapters-storage-mysql
- **Location:** `crates/adapters/storage-mysql/src/storage.rs:1-65` (module doc)

**Description:**

The module-level doc comment for
  `storage.rs` references
  "`watch_changes`, `apply_snapshot`, `cursor_for`, and
  `advance_cursor` return `DomainError::NotSupported` per the
  default impls" — but `watch_changes` actually returns an
  empty `ChangeStream` (not a `NotSupported` error), and
  `cursor_for` returns `VersionCursor::ZERO` (not a
  `NotSupported` error). The doc comment is therefore
  factually wrong on two of the four methods it describes.

**Expected:**

Module doc strings MUST accurately describe the
  method's actual return value. (`docs/code-standards.md` §
  "Public API documentation").

**Evidence:**

```rust
  // crates/adapters/storage-mysql/src/storage.rs:46-49
  //! `watch_changes`, `apply_snapshot`, `cursor_for`, and
  //! `advance_cursor` return `DomainError::NotSupported` per the
  //! default impls in the `StorageAdapter` trait.
  ```
  vs the actual bodies at
  `crates/adapters/storage-mysql/src/storage.rs:201-245`.

---

### FINDING 23 (id: `ADAPT-MY-023`)

- **Source:** `docs/audit_reports/findings/wave3-storage-mysql.md`
- **Severity:** Low
- **Area:** adapters-storage-mysql
- **Location:** `crates/adapters/storage-mysql/src/storage.rs:51` (`SCHEMA_VERSION` constant)

**Description:**

`SCHEMA_VERSION` is `const SCHEMA_VERSION: u32 = 1;`
  but the engine claims (via `docs/schemas/sql-dialects/README.md`)
  to emit a per-adapter `schema_version` derived from the
  macro-emitted AST. With no AST walk, the version is a
  hand-maintained constant that drifts from the actual DDL
  on every change. There is no test that asserts
  `migrate()` is a no-op when `SCHEMA_VERSION` matches the
  already-applied version.

**Expected:**

"`schema_version` is computed at runtime
  from the macro-emitted AST's `version` attribute."
  (`docs/build-plan.md` Phase 0 exit criteria).

**Evidence:**

`crates/adapters/storage-mysql/src/storage.rs:51`
  ```rust
  const SCHEMA_VERSION: u32 = 1;
  ```
  No `version = ...` attribute on any aggregate; no AST
  walk; no test that verifies "second migrate() is a
  no-op". The same pattern exists in the Postgres and
  SQLite adapters.

---

### FINDING 24 (id: `ADAPT-MY-024`)

- **Source:** `docs/audit_reports/findings/wave3-storage-mysql.md`
- **Severity:** Low
- **Area:** adapters-storage-mysql
- **Location:** `crates/adapters/storage-mysql/src/storage.rs:180-189` (`close` — double-close hazard)

**Description:**

`close()` stores `true` in `self.closed`,
  then calls `self.conn.into_inner().close().await` on the
  inner `MySqlPool`. If a caller invokes `close` twice on
  the same `Box<Self>` (e.g. via `Drop` plus an explicit
  `close` call), the second `close` returns
  `Ok(())` silently (because `self.closed` is already
  `true`); meanwhile, sqlx's pool `close()` is idempotent
  and returns `Ok(())` for a closed pool. There is no
  contract violation here, but the `closed` flag is set
  *before* the inner pool close completes — a caller that
  observes the flag during `close` may believe the adapter
  is no longer usable while the pool is still draining.

**Expected:**

"Set the closed flag *after* the inner pool
  close completes, so the flag accurately reflects the
  shutdown state." (`docs/code-standards.md` § "State
  transitions").

**Evidence:**

`crates/adapters/storage-mysql/src/storage.rs:180-189`
  ```rust
  async fn close(self: Box<Self>) -> Result<()> {
      self.closed.store(true, Ordering::SeqCst);
      self.conn.into_inner().close().await;
      Ok(())
  }
  ```
  The `closed` write happens before the `.await` of
  `close()`.

---


## Storage — SQLite (target id prefix: `ADAPTER-SQ`)

**Path:** `crates/adapters/storage-sqlite/`  
**Total findings:** 50 (5 critical, 14 high, 15 medium, 16 low)


### FINDING 1 (id: `ADAPTER-SQ-001`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/storage.rs:97-133`

**Description:**

The `migrate()` implementation only loads `migrations/engine/0000_engine_core.sqlite.sql` (6 engine cross-cutting tables) plus the `bulk_attendance.sql` (1 attendance domain table). It does not walk any macro-emitted AST to emit the ~310 domain tables the engine claims to ship, and it does not honour `docs/schemas/sql-dialects/sqlite.md`'s `SqliteStorageAdapter::create_<table>_ddl()` contract at all. `Schema-registry` and `system_user` are emitted; no domain tables are emitted.

**Expected:**

"The schema is emitted by the storage adapter at startup via `storage.create_schema().await`. … 4. Adapter emission — `educore-storage-<db>` walks the AST at schema-creation time and emits the dialect-specific DDL string. … 5. Consumer startup — `storage.create_schema().await` runs the DDL once per process lifetime." (`docs/schemas/sql-dialects/README.md` § "Runtime DDL emission — end-to-end flow").

**Evidence:**

`crates/adapters/storage-sqlite/src/storage.rs:35` `const SCHEMA_SQL: &str = include_str!("../../../../migrations/engine/0000_engine_core.sqlite.sql");` and `crates/adapters/storage-sqlite/src/storage.rs:108-124` only executes `SCHEMA_SQL` and `SqliteBulkAttendance::ensure_schema`. The crate has no `create_schema()`, no AST walk, and no domain-table emission code anywhere under `src/`.

---

### FINDING 2 (id: `ADAPTER-SQ-002`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/outbox.rs:139-175` (`append`)

**Description:**

`Outbox::append` calls a plain `INSERT INTO outbox ...` and surfaces the underlying `sqlx::Error` (which includes the unique-constraint violation on `event_id`) as `DomainError::Infrastructure`. The port contract requires `DomainError::Conflict` on a duplicate `(school_id, event_id)`. The adapter silently downgrades a contract-mandated domain error to an infrastructure error.

**Expected:**

"`Conflict` if an envelope with the same `event_id` was already appended in the same school." (`crates/infra/storage/src/outbox.rs:99-101`).

**Evidence:**

`crates/adapters/storage-sqlite/src/outbox.rs:148-172` `sqlx::query::<sqlx::Sqlite>("INSERT INTO outbox ( ...")... .execute(&self.pool).await.map_err(|e| StringError(format!("outbox append: {e}")))?;` — no `match` on `sqlx::Error::Database(db)` to map `UniqueViolation` to `DomainError::conflict(...)`. Compare with `crates/adapters/storage-sqlite/src/bulk_attendance.rs:205-212` which DOES map `UniqueViolation` to `DomainError::conflict(...)`.

---

### FINDING 3 (id: `ADAPTER-SQ-003`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/idempotency.rs:123-152` (`record`)

**Description:**

`Idempotency::record` uses `INSERT OR REPLACE INTO idempotency ...`, which silently overwrites any prior row with the same composite key and discards the previous `command_id` (the implementation regenerates a fresh `Uuid::now_v7()` every call at line 124). The port contract requires `DomainError::Conflict` when a record with the same composite key exists with a different outcome, and `Ok(())` only when the new row is identical. The current behaviour violates both halves: it never returns `Conflict`, and it overwrites regardless of outcome equality.

**Expected:**

"Stores `record`. Returns `Err(Conflict)` if a record with the same `(school_id, command_type, idempotency_key)` already exists with a different outcome. Returns `Ok(())` if the record is a no-op write (same key, same outcome hash) — the engine uses this for at-least-once delivery of retries." (`crates/infra/storage/src/idempotency.rs:94-100`).

**Evidence:**

`crates/adapters/storage-sqlite/src/idempotency.rs:134-149` `sqlx::query::<sqlx::Sqlite>("INSERT OR REPLACE INTO idempotency ( school_id, command_type, idempotency_key, command_id, outcome, recorded_at, expires_at ) VALUES (?, ?, ?, ?, ?, ?, ?)")` with `.bind(command_id.hyphenated())` where `command_id = Uuid::now_v7()` (line 124).

---

### FINDING 4 (id: `ADAPTER-SQ-004`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/outbox.rs:166-167` (`append`)

**Description:**

`recorded_at` is bound to `envelope.occurred_at.as_datetime()` instead of `Utc::now()`. The DDL declares `recorded_at` as the persistence time (a separate column from `occurred_at`), and the engine invariant is that `recorded_at >= occurred_at` (it captures ingestion latency). Binding both to the same value obliterates that invariant.

**Expected:**

"`recorded_at` Wall-clock time of the persistence (≥ `occurred_at`)" (`crates/infra/storage/src/event_log.rs:73`) and the outbox DDL column pair `occurred_at ... recorded_at ...` (`migrations/engine/0000_engine_core.sqlite.sql:74-75`).

**Evidence:**

`crates/adapters/storage-sqlite/src/outbox.rs:166-169` `.bind(envelope.occurred_at.as_datetime()) // occurred_at\n        .bind(envelope.occurred_at.as_datetime()) // recorded_at ← BUG\n        .bind(payload_json)\n        .bind(now) // enqueued_at`. Compare `audit_log.rs:151-152` which correctly binds `entry.occurred_at.as_datetime()` and `recorded_at = Utc::now()` separately.

---

### FINDING 5 (id: `ADAPTER-SQ-005`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/storage.rs:187-195` (`advance_cursor`) and `crates/adapters/storage-sqlite/src/storage.rs:175-185` (`cursor_for`)

**Description:**

Both `cursor_for` and `advance_cursor` silently override the trait default of `DomainError::NotSupported`. `cursor_for` returns `Ok(VersionCursor::ZERO)` and `advance_cursor` returns `Ok(())`. The default-impl contract is the sync engine's safety net: non-sync adapters must fail loudly at startup. The SQLite implementation reports success, masking configuration problems and letting the sync engine start up against an adapter that is actually doing nothing.

**Expected:**

"Default impls on the trait return `DomainError::NotSupported('sync primitives require the sync feature and a sync-capable adapter')`. The sync engine, when it tries to subscribe on a non-sync adapter, fails loudly at startup — not silently at runtime — so consumers see the configuration problem immediately." (`docs/ports/storage.md:112-116`).

**Evidence:**

`crates/adapters/storage-sqlite/src/storage.rs:175-185` `async fn cursor_for(&self, _school_id: SchoolId) -> Result<VersionCursor> { ... Ok(VersionCursor::ZERO) }` and `crates/adapters/storage-sqlite/src/storage.rs:187-195` `async fn advance_cursor(&self, _school_id: SchoolId, _to: VersionCursor) -> Result<()> { ... Ok(()) }` — both return success instead of `DomainError::not_supported(...)`.

---

### FINDING 10 (id: `ADAPTER-SQ-010`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/storage.rs:117, 127-132`

**Description:**

`migrate()` hard-codes `MigrationReport { statements_executed: 0, already_at_version: false, ... }`. The `statements_executed` field exists to report the actual count of statements applied (useful for telemetry, migration-time SLOs, and idempotency verification) and the adapter explicitly discards the `sqlx` result (`let _ = result;` at line 117). `already_at_version` is always `false`, even when re-running `migrate()` on an already-migrated database — so the report cannot be used by callers to distinguish a first run from a no-op run.

**Expected:**

Per `crates/infra/storage/src/change_stream.rs:243-255`: "`statements_executed`: The number of statements executed (DDL or DML)." and "`already_at_version`: Whether the migration was a no-op (already at `version`)." The handoff doc claims the migration is "idempotent thanks to the `IF NOT EXISTS` clauses" but the report does not surface that idempotency.

**Evidence:**

`crates/adapters/storage-sqlite/src/storage.rs:108-132` shows `let _ = result;` discarding the `Execute` result, and `Ok(MigrationReport { version: SCHEMA_VERSION, statements_executed: 0, duration, already_at_version: false })`.

---

### FINDING 11 (id: `ADAPTER-SQ-011`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `migrations/engine/0000_engine_core.sqlite.sql:64-235` (all six tables)

**Description:**

The canonical SQLite DDL omits both the `STRICT` table option and the `WITHOUT ROWID` option that the dialect spec mandates for every cross-cutting table. `STRICT` enforces type affinity (preventing SQLite's silent string-to-integer coercion), and `WITHOUT ROWID` saves 4-8 bytes per row for lookup-only tables. The dialect spec is explicit that the engine refuses to write to non-`STRICT` tables.

**Expected:**

"`STRICT` enforces the type affinity. Without it, SQLite allows silent type coercion (e.g. inserting `'hello'` into an `INTEGER` column). The engine refuses to write to non-`STRICT` tables." and "`WITHOUT ROWID` saves 4-8 bytes per row and is faster for point-lookups. The engine's `outbox`, `event_log`, `audit_log`, `idempotency`, and `schema_registry` are all `WITHOUT ROWID`." (`docs/schemas/sql-dialects/sqlite.md:78-99`).

**Evidence:**

`migrations/engine/0000_engine_core.sqlite.sql:64-82` `outbox`, `:107-129` `audit_log`, `:151-160` `idempotency`, `:174-188` `event_log`, `:208-216` `schema_registry`, `:229-235` `system_user` all use `CREATE TABLE IF NOT EXISTS <name> ( ... );` — no `STRICT` and no `WITHOUT ROWID` clause on any of them.

---

### FINDING 12 (id: `ADAPTER-SQ-012`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `migrations/engine/0000_engine_core.sqlite.sql:76, 124-125, 156, 186`

**Description:**

The `outbox.payload`, `audit_log.before_snapshot` / `audit_log.after_snapshot` / `audit_log.metadata`, `idempotency.outcome`, and `event_log.payload` columns are declared as plain `TEXT NOT NULL` with no `CHECK (json_valid(x))` constraint. The dialect spec mandates `json_valid(...)` CHECK on every JSON column. Without the CHECK, the database accepts any text and the type-coercion invariant that the engine relies on is enforced only by application code that may be bypassed by direct SQL (e.g. backfill scripts, ad-hoc queries by ops staff).

**Expected:**

`"payload"         TEXT NOT NULL CHECK (json_valid("payload"))` and similar `"before_snapshot" TEXT CHECK ("before_snapshot" IS NULL OR json_valid("before_snapshot"))` (`docs/schemas/sql-dialects/sqlite.md:202, 238-240, 265, 290, 309`).

**Evidence:**

`migrations/engine/0000_engine_core.sqlite.sql:76` `"payload" TEXT NOT NULL` (no CHECK), `:124-125` `"before_snapshot" TEXT NULL` / `"after_snapshot" TEXT NULL` (no CHECK), `:126` `"metadata" TEXT NULL` (no CHECK), `:156` `"outcome" TEXT NOT NULL` (no CHECK), `:186` `"payload" TEXT NOT NULL` (no CHECK).

---

### FINDING 13 (id: `ADAPTER-SQ-013`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/bulk_attendance.sql:14-39` (`attendance_student_attendances` schema)

**Description:**

The `attendance_student_attendances` domain table stores every UUID as `BLOB` (16 bytes) instead of `TEXT` with `CHECK (length(x) = 36)` as the engine spec mandates for UUIDv7 columns. It also omits `STRICT`, `WITHOUT ROWID`, the `json_valid` CHECKs on TEXT JSON columns, and every length-36 UUID CHECK that the spec requires. The result is that the bulk-attendance table uses a completely different wire form than the 6 engine cross-cutting tables in the same database, and an ops engineer writing a `SELECT * FROM attendance_student_attendances` JOIN against the engine tables cannot compare UUIDs without explicit hex conversion.

**Expected:**

"`id` TEXT NOT NULL PRIMARY KEY" with `CHECK (length("id") = 36)` and `STRICT` (`docs/schemas/sql-dialects/sqlite.md:54, 80, 391-393`).

**Evidence:**

`crates/adapters/storage-sqlite/src/bulk_attendance.sql:14-39` — every UUID column (`school_id`, `id`, `student_id`, `student_record_id`, `class_id`, `section_id`, `marked_by`, `created_by`, `updated_by`, `last_event_id`, `correlation_id`) is declared `BLOB`. `bulk_attendance.sql:13` is `CREATE TABLE IF NOT EXISTS attendance_student_attendances ( ... );` with no `STRICT` or `WITHOUT ROWID`. `crates/adapters/storage-sqlite/src/student_attendance_row.rs:111-180` confirms the adapter binds UUIDs as `Vec<u8>` (16 bytes big-endian) via `school_id_bytes()`, etc.

---

### FINDING 14 (id: `ADAPTER-SQ-014`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/outbox.rs:144-145, :76-77` and `crates/adapters/storage-sqlite/src/event_log.rs:55, 95`

**Description:**

Both `Outbox::append`, `OutboxRow::to_envelope`, `EventLog::append`, and `EventLogRow::to_entry` use `i32::try_from(...).unwrap_or(0)` and `u32::try_from(...).unwrap_or(0)` to silently clamp `schema_version` on overflow. The engine's invariant is that `schema_version` is a small positive integer, but the silent fallback to `0` discards data without surfacing the error — a caller that has produced a malformed envelope will not see `Err(Validation)` and downstream consumers will silently treat the event as schema v0, which may have an unrelated payload shape.

**Expected:**

Per `AGENTS.md` § "Type Safety": "No `as` casts that truncate or lose data. Use `TryFrom` / `TryInto` with proper error handling." and "All public APIs return `Result` for fallible operations."

**Evidence:**

`crates/adapters/storage-sqlite/src/outbox.rs:77` `schema_version: u32::try_from(self.event_version).unwrap_or(0),`, `:145` `let event_version = i32::try_from(envelope.schema_version).unwrap_or(0);`; `crates/adapters/storage-sqlite/src/event_log.rs:55` `schema_version: u32::try_from(self.event_version).unwrap_or(0),`, `:95` `let event_version = i32::try_from(entry.schema_version).unwrap_or(0);`.

---

### FINDING 15 (id: `ADAPTER-SQ-015`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/connection.rs:135`

**Description:**

`SqliteConnection::connect` logs the raw URL via `debug!(school = %school, url, "opened file-backed sqlite")`. SQLx SQLite URLs do not embed credentials, but the convention is established and any future migration to a URL-bearing driver (e.g. a remote-sqlite bridge or a query string with `?mode=ro&key=...`) will silently log secrets. Logging the raw URL is the standard mistake that turns into a credential leak later.

**Expected:**

Per `AGENTS.md` § "Authoritative Documents" and the audit-first rule: no credentials or PII in log lines.

**Evidence:**

`crates/adapters/storage-sqlite/src/connection.rs:135` `debug!(school = %school, url, "opened file-backed sqlite");` — the full `url: &str` is bound as a tracing field with no redaction. Also `crates/adapters/storage-sqlite/src/connection.rs:106-107, 131-133` interpolate the URL into the `StringError` message that becomes the public `DomainError::infrastructure` variant.

---

### FINDING 16 (id: `ADAPTER-SQ-016`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/storage.rs:197-209` (`bulk_insert_student_attendances`)

**Description:**

The adapter-level `bulk_insert_student_attendances` accepts a `&TenantContext` but immediately ignores it. It constructs a fresh `SqliteBulkAttendance` handle from `self.conn.db().clone(), self.conn.school()` and calls `handle.bulk_insert(ctx.school_id, rows)` — the per-row `school_id` validation in `bulk_insert_into` checks against `ctx.school_id` (the caller's anchor), not against `self.conn.school()`. If the adapter is opened with school A and a caller from tenant B invokes `bulk_insert_student_attendances(&TenantContext{school_id: B, ...}, rows)`, every row's `school_id` is validated against B but the rows are written to the connection's pool which is scoped to A. The validation passes (B == B) and rows are silently inserted into the wrong school.

**Expected:**

"`StorageAdapter::bulk_insert_student_attendances` ... MUST validate that every row's `school_id` matches `ctx.school_id` and reject the call with a `DomainError::Validation` otherwise." (`crates/infra/storage/src/port.rs:60-63`).

**Evidence:**

`crates/adapters/storage-sqlite/src/storage.rs:197-209` constructs `SqliteBulkAttendance::new(self.conn.db().clone(), self.conn.school())` and passes `ctx.school_id` (which may differ from `self.conn.school()`) to `bulk_insert`. `crates/adapters/storage-sqlite/src/bulk_attendance.rs:145-152` validates `r.school_id != school_id` where `school_id` is the caller-supplied parameter, not the connection's scoped school. The transaction-level path (`transaction.rs:134-148`) correctly validates against the transaction's scoped school.

---

### FINDING 17 (id: `ADAPTER-SQ-017`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/tests/outbox_e2e.rs` (entire file)

**Description:**

The test suite contains exactly one end-to-end test (`outbox_append_and_pending_round_trip`) covering only the outbox sub-port. No tests exist for: `audit_log.append` / `audit_log.read_for_target`, `event_log.append` / `event_log.read` / `event_log.count`, `idempotency.lookup` / `idempotency.record` / `idempotency.purge_older_than`, `bulk_insert_student_attendances` (the Phase 5 critical path), `migrate()` idempotency, `cursor_for` / `advance_cursor` return-value verification, `ping()`, `close()` lifecycle, tenant-isolation enforcement, SQL-injection attempts, or any round-trip across the `SqliteTransaction` boundary. The single test path uses the in-memory constructor only.

**Expected:**

Per `docs/ports/storage.md:468-477`: "The port requires: Unit tests of every repository method. Integration tests against a real database (testcontainers). A parity test verifying identical behavior across adapters. A tenancy test verifying cross-tenant reads are blocked. A failure-injection test (e.g. deadlock retry, connection drop). A load test (10k attendance marks in <5s)."

**Evidence:**

`ls crates/adapters/storage-sqlite/tests/` returns only `outbox_e2e.rs`. The Phase 1 handoff at `docs/handoff/PHASE-1-HANDOFF.md:131-133` explicitly admits: "the existing e2e currently exercises the in-memory path only. The single-writer deployment model ... is documented but not tested at scale."

---

### FINDING 18 (id: `ADAPTER-SQ-018`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/storage.rs:1-210` (entire `StorageAdapter` impl)

**Description:**

The `StorageAdapter` trait in `docs/ports/storage.md:17-89` enumerates ~22 per-aggregate repository methods (`students()`, `guardians()`, `classes()`, ..., one per aggregate across 15 domains). The actual port trait at `crates/infra/storage/src/port.rs:34-150` exposes only 5 methods plus 4 sync primitives. The SQLite adapter implements the actual trait (no repository methods) — meaning **none** of the documented per-aggregate repository handles are implemented. The dialect spec promises `SqliteStorageAdapter::create_<table>_ddl()` per aggregate; no such method exists in the crate.

**Expected:**

The port trait in `docs/ports/storage.md` declares `fn students(&self) -> Arc<dyn StudentRepository>;` and ~21 sibling methods, "one handle per aggregate, across all 15 domains (~80+ total)" (`docs/ports/storage.md:50`). Each adapter must translate the macro-emitted `QueryNode` AST into a SQLite execution plan.

**Evidence:**

`crates/adapters/storage-sqlite/src/storage.rs:84-211` implements only `begin`, `migrate`, `ping`, `close`, `watch_changes`, `apply_snapshot`, `cursor_for`, `advance_cursor`, and `bulk_insert_student_attendances`. `grep -n 'students\|guardians\|classes\|sections' crates/adapters/storage-sqlite/src/` returns no repository handle of any kind.

---

### FINDING 19 (id: `ADAPTER-SQ-019`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/storage.rs:1-38` (lib doc) and `crates/adapters/storage-sqlite/Cargo.toml:1-26`

**Description:**

The dialect spec at `docs/schemas/sql-dialects/sqlite.md:7-8, 397-410` documents the adapter as using `rusqlite` (`The SqliteStorage adapter uses rusqlite for the connection. rusqlite 0.31+ is the recommended version.`) and a constructor pattern `SqliteStorage::open("path/to/db.sqlite")?` with `.with_key(b"...")?` for encryption. The actual crate uses `sqlx::SqlitePool` (no `rusqlite` dependency) and an entirely different API surface (`SqliteConnection::connect(url, school)`, `SqliteStorageAdapter::new(conn)`). The handoff at `docs/handoff/PHASE-1-HANDOFF.md:29-34` records this as a deliberate departure, but `docs/schemas/sql-dialects/sqlite.md` has not been updated to reflect the change.

**Expected:**

Adapter implementation notes say rusqlite; `Cargo.toml` should declare `rusqlite` per the spec.

**Evidence:**

`crates/adapters/storage-sqlite/Cargo.toml:13-26` declares `sqlx = { workspace = true }` with no `rusqlite` dependency. `crates/adapters/storage-sqlite/src/connection.rs:13-15` `use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions, SqliteSynchronous};` — no rusqlite. The dialect spec's `Adapter implementation notes` section (`docs/schemas/sql-dialects/sqlite.md:395-410`) is stale.

---

### FINDING 6 (id: `ADAPTER-SQ-006`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/idempotency.rs:65-75` (`to_record`)

**Description:**

`IdempotencyRow::to_record` calls `Box::leak(self.command_type.clone().into_boxed_str())` on every read. The port struct's `command_type: &'static str` field forces this leak. In a long-running process serving many idempotency lookups the heap grows without bound — a slow but unbounded memory leak in production code.

**Expected:**

Per `AGENTS.md` § "Type Safety": "No `#[allow(dead_code)]` or `_var` prefixes to silence the compiler. Delete unused code, wire it in, or open a follow-up issue." Adapter code must not leak memory per-call.

**Evidence:**

`crates/adapters/storage-sqlite/src/idempotency.rs:68` `command_type: Box::leak(self.command_type.clone().into_boxed_str()),` (acknowledged as a known limitation in the doc-comment at lines 53-64 but not resolved in code).

---

### FINDING 7 (id: `ADAPTER-SQ-007`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/idempotency.rs:65-75` (`to_record`)

**Description:**

`IdempotencyRow::to_record` returns hard-coded `outcome_version: 0` and `affected_aggregate_ids: Vec::new()` because the DDL has no columns for them. The audit and idempotency contract relies on these two fields to detect "same key, different target" misuse and to version the outcome payload. The adapter silently discards both on read.

**Expected:**

"`outcome_version`: The schema version of the `outcome` payload." and "`affected_aggregate_ids`: The aggregate ids touched by the original command. Used by the dispatcher to detect 'same idempotency key, but different target' misuse." (`crates/infra/storage/src/idempotency.rs:37-44`).

**Evidence:**

`crates/adapters/storage-sqlite/src/idempotency.rs:42-50` `struct IdempotencyRow { ... outcome: String, recorded_at: DateTime<Utc>, expires_at: DateTime<Utc>, }` — no columns for outcome_version or affected_aggregate_ids; and `crates/adapters/storage-sqlite/src/idempotency.rs:71-73` hard-codes both to `0` and `Vec::new()`.

---

### FINDING 8 (id: `ADAPTER-SQ-008`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/storage.rs:87-205` (every method on a closed adapter)

**Description:**

Every `StorageAdapter` method (`begin`, `migrate`, `ping`, `watch_changes`, `cursor_for`, `advance_cursor`, `bulk_insert_student_attendances`) returns `DomainError::Conflict` when the adapter is closed. The port contract mandates `DomainError::Infrastructure`. Returning `Conflict` is structurally wrong (closing the pool is not a state conflict) and breaks error-handling callers that match on `ErrorKind::Infrastructure` to surface a degraded-storage alert.

**Expected:**

"`close(self: Box<Self>) -> Result<()>; ... After `close`, any further call returns `Err(Infrastructure)`." (`crates/infra/storage/src/port.rs:53` and `docs/ports/storage.md:23`).

**Evidence:**

`crates/adapters/storage-sqlite/src/storage.rs:88-91, 99-102, 137-140, 159-163, 176-179, 188-191, 202-205` all call `DomainError::conflict("...")` instead of `DomainError::infrastructure(...)` after the `self.closed.load(Ordering::SeqCst)` check.

---

### FINDING 9 (id: `ADAPTER-SQ-009`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/connection.rs:62-64, 109-111`

**Description:**

The connection layer unconditionally requests `SqliteJournalMode::Wal` for both in-memory and file-backed connections. SQLite's WAL journal mode requires a file-backed database; for `sqlite::memory:` the journal mode silently downgrades to `MEMORY` at the SQLite layer (this is documented SQLite behaviour). The application code then issues `PRAGMA journal_mode = WAL` again in `after_connect` — which is a no-op for in-memory DBs but emits a SQLite "no-op" warning that the adapter silently swallows. The `in_memory` path is therefore misadvertised as WAL-backed.

**Expected:**

"The pool is constrained to a single connection so every consumer in the same process sees the same in-memory database. This is the default connection for tests and single-process embedded deployments." (`crates/adapters/storage-sqlite/src/connection.rs:43-49`); and the spec requires WAL/NORMAL/foreign_keys PRAGMAs.

**Evidence:**

`crates/adapters/storage-sqlite/src/connection.rs:62-64` `.journal_mode(SqliteJournalMode::Wal).synchronous(SqliteSynchronous::Normal).foreign_keys(true)` is applied to `SqliteConnectOptions::from_str("sqlite::memory:")` (line 55) the same way as the file-backed path. `crates/adapters/storage-sqlite/src/connection.rs:68-80` re-emits the same PRAGMAs in `after_connect`. The SQLite docs are explicit that WAL is silently downgraded for `:memory:`.

---

### FINDING 20 (id: `ADAPTER-SQ-020`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/audit_log.rs:13-25` (`SqliteAuditLog`)

**Description:**

`AuditLogEntry` carries only a subset of the DDL columns. On write the adapter hardcodes `actor_type = "user"`, `source = "system"`, `recorded_at = Utc::now()`, `command_id = NULL`, `ip = NULL`, `user_agent = NULL`, `session_id = NULL`, `cross_tenant = 0` — none of which are parameterised by the entry struct. The port trait's `AuditLogEntry` has no slot for these fields, but the handoff (`docs/handoff/PHASE-1-HANDOFF.md:168-175`) acknowledges the gap. The practical effect: an audit row written through this adapter cannot distinguish a user-initiated mutation from a system mutation, cannot record the originating IP, and cannot be correlated to a `command_id` — the very fields the audit schema is designed to capture.

**Expected:**

"`actor_type`: user, system, integration, scheduled. … `source`: rest, graphql, cli, internal. … `ip`, `user_agent`, `session_id`: caller context." (`docs/schemas/audit-schema.md` § 13 and the DDL column list at `migrations/engine/0000_engine_core.sqlite.sql:107-129`).

**Evidence:**

`crates/adapters/storage-sqlite/src/audit_log.rs:140-156` binds hard-coded literals: `bind("user")`, `bind("system")`, `.bind(recorded_at)` (computed locally as `Utc::now()`), and `NULL` literals embedded in the SQL string for `command_id`, `ip`, `user_agent`, `session_id`, with `0` literal for `cross_tenant`.

---

### FINDING 21 (id: `ADAPTER-SQ-021`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/event_log.rs:48-66` (`EventLogRow::to_entry`)

**Description:**

`EventLogRow::to_entry` always returns `ActiveStatus::Active` because the DDL has no `active_status` column on `event_log`. The port contract is that consumers can transition an event row to `Retired` for GDPR erasure; this transition has no physical storage in the SQLite adapter. Once written, an event row is forever `Active` regardless of any consumer-driven retraction.

**Expected:**

"The event log carries `active_status` so consumers can retire events (e.g. for GDPR erasure) without deleting the row (audit trails must remain)." (`crates/infra/storage/src/event_log.rs:42-45`).

**Evidence:**

`crates/adapters/storage-sqlite/src/event_log.rs:29-44` `struct EventLogRow { event_id, event_type, event_version, school_id, aggregate_id, aggregate_type, actor_id, correlation_id, causation_id, occurred_at, recorded_at, payload, }` — no `active_status` column. `:64` `active_status: ActiveStatus::Active,` is hardcoded. Compare `migrations/engine/0000_engine_core.sqlite.sql:174-188` which also lacks an `active_status` column.

---

### FINDING 22 (id: `ADAPTER-SQ-022`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `migrations/engine/0000_engine_core.sqlite.sql:174-188` (`event_log`)

**Description:**

The `event_log` table DDL lacks the `active_status` column that the port trait's `EventLogEntry::active_status` field requires. The handoff (`docs/handoff/PHASE-1-HANDOFF.md:170`) acknowledges the gap ("no `active_status` on `event_log`") and treats it as Phase 2 work. Production code that calls `EventLog::read` will get back rows whose `active_status` is silently hardcoded to `Active`, masking any future GDPR-retirement semantics.

**Expected:**

`active_status INTEGER NOT NULL DEFAULT 1 CHECK (active_status IN (0, 1))` per `docs/schemas/sql-dialects/sqlite.md:367-368` ("`"active_status"     INTEGER NOT NULL DEFAULT 1 CHECK ("active_status" IN (0,1))`").

**Evidence:**

`migrations/engine/0000_engine_core.sqlite.sql:174-188` — the column list runs `event_id ... payload` with no `active_status`. The dialect spec's example for `academic_students` (line 367) shows the column is mandatory on every aggregate; `event_log` is not exempt.

---

### FINDING 23 (id: `ADAPTER-SQ-023`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/outbox.rs:243-249` (trailing `const _`)

**Description:**

The file ends with a `const _: fn() = || { let _b: Bytes = Bytes::new(); };` block whose purpose is to suppress an unused-import warning for `bytes::Bytes`. The `bytes` crate is imported but the only consumer of the `Bytes` type in this file is `SerializedEnvelope::payload`, which is constructed by callers, not by this module. The `Bytes` import is dead and the suppression block is a code smell.

**Expected:**

Per `AGENTS.md` § "Type Safety": "No `#[allow(dead_code)]` or `_var` prefixes to silence the compiler. Delete unused code, wire it in, or open a follow-up issue."

**Evidence:**

`crates/adapters/storage-sqlite/src/outbox.rs:243-249` `#[allow(dead_code)]\nconst _: fn() = || { let _b: Bytes = Bytes::new(); };`. Also `crates/adapters/storage-sqlite/src/outbox.rs:122-135` defines an `IntoUuid` extension trait with `#[allow(dead_code)]` that is never referenced anywhere in the crate.

---

### FINDING 24 (id: `ADAPTER-SQ-024`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/transaction.rs:105-117` (`commit` / `rollback`)

**Description:**

The `commit` and `rollback` impls are no-ops that only flip the `done`/`rolled_back` atomic flag. The handoff (`docs/handoff/PHASE-1-HANDOFF.md:36-46, 157-167`) acknowledges this as a deliberate Phase 1 simplification — but in production this means a caller that invokes `tx.commit()` does not actually commit anything: the writes are visible immediately (sqlx auto-commits per call) and the "commit" step is purely cosmetic. A caller that wants to roll back has no way to undo writes that were committed by earlier sub-port calls inside the same `Transaction`.

**Expected:**

"Commits the transaction. All outbox appends, aggregate mutations, audit log writes, idempotency records, and event log rows become durable." (`crates/infra/storage/src/transaction.rs:35-37`).

**Evidence:**

`crates/adapters/storage-sqlite/src/transaction.rs:107-116` `async fn commit(self: Box<Self>) -> Result<()> { self.done.store(true, Ordering::SeqCst); Ok(()) }` and `async fn rollback(self: Box<Self>) -> Result<()> { self.rolled_back.store(true, Ordering::SeqCst); self.done.store(true, Ordering::SeqCst); Ok(()) }` — neither call interacts with the database.

---

### FINDING 25 (id: `ADAPTER-SQ-025`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/transaction.rs:52-73` (`SqliteTransaction` struct)

**Description:**

The `SqliteTransaction` holds four `*Box` sub-port handles and a `_pool: SqlitePool` even though the `commit`/`rollback` impls do nothing with the pool. The `_pool` field name (leading underscore) is a code smell that the field is unused; the actual sub-port handles each cloned their own pool internally (`SqliteOutbox::new(pool.clone(), school)`, etc.) so the field could be deleted entirely. The unused field makes the struct bigger than necessary and obscures the lack of real transactional semantics.

**Expected:**

Per `AGENTS.md` § "Type Safety": "No `#[allow(dead_code)]` or `_var` prefixes to silence the compiler. Delete unused code, wire it in, or open a follow-up issue."

**Evidence:**

`crates/adapters/storage-sqlite/src/transaction.rs:72` `_pool: SqlitePool,` and `crates/adapters/storage-sqlite/src/transaction.rs:86-101` shows the field is only assigned, never read in the rest of the file.

---

### FINDING 26 (id: `ADAPTER-SQ-026`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/outbox.rs:118-135` (`IntoUuid` trait)

**Description:**

The file declares a `pub(crate) trait IntoUuid { fn into_uuid(self) -> uuid::Uuid; }` and a blanket impl `impl IntoUuid for Hyphenated` with `#[allow(dead_code)]`. The trait is never used anywhere in the workspace (the conversion from `Hyphenated` to `uuid::Uuid` is done inline via `*self.as_uuid()` at every call site). This is dead code that survived from a SurrealDB-pattern mirror.

**Expected:**

Per `AGENTS.md` § "Type Safety": "Delete unused code, wire it in, or open a follow-up issue."

**Evidence:**

`crates/adapters/storage-sqlite/src/outbox.rs:122-135` defines the trait and impl with `#[allow(dead_code)]`. `grep -rn 'IntoUuid' crates/` shows only the definition and impl, no consumers.

---

### FINDING 27 (id: `ADAPTER-SQ-027`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/util.rs:10-13` (`bytes_to_json`)

**Description:**

`bytes_to_json` calls `serde_json::from_slice(bytes).unwrap_or_else(|_| ...)` and falls back to a JSON-string wrapper when the bytes are not valid JSON. The round-trip is therefore lossy: a payload that fails to parse as JSON is stored as a quoted UTF-8 string. The downstream `json_to_bytes` (line 18-23) does the inverse — it serialises the JSON-Value back to bytes, so a non-JSON payload round-trips as `"original utf-8 bytes"` (a JSON string literal). The semantic of `Outbox::pending` therefore silently changes: a caller reading back an outbox row whose payload was a binary blob or invalid UTF-8 receives a string-wrapped version that loses the original byte boundaries.

**Expected:**

Per `AGENTS.md` § "Production-ready" and "All public APIs are documented with rustdoc"; payload round-trip should be byte-exact or documented as lossy.

**Evidence:**

`crates/adapters/storage-sqlite/src/util.rs:10-13` `pub(crate) fn bytes_to_json(bytes: &Bytes) -> serde_json::Value { serde_json::from_slice(bytes).unwrap_or_else(|_| serde_json::Value::String(String::from_utf8_lossy(bytes).into_owned())) }`. The mirror on the write path at `idempotency.rs:133` `let outcome_str = String::from_utf8_lossy(&record.outcome).into_owned();` is similarly lossy on non-UTF-8 payloads.

---

### FINDING 28 (id: `ADAPTER-SQ-028`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/storage.rs:150-153` (`close`)

**Description:**

`close(self: Box<Self>)` flips the `closed` atomic and returns `Ok(())`. It does not call `pool.close().await` to release the underlying connections. For an in-memory `SqlitePool`, dropping the pool is sufficient; for a file-backed pool, not awaiting `pool.close()` can leave the WAL writer thread alive until the process exits. The method name promises a graceful close but the implementation does not.

**Expected:**

"`close(self: Box<Self>) -> Result<()>; // Closes the adapter, releasing all underlying connections." (`crates/infra/storage/src/port.rs:52-53`).

**Evidence:**

`crates/adapters/storage-sqlite/src/storage.rs:150-153` `async fn close(self: Box<Self>) -> Result<()> { self.closed.store(true, Ordering::SeqCst); Ok(()) }`. No call to `self.conn.db().close().await`.

---

### FINDING 29 (id: `ADAPTER-SQ-029`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/bulk_attendance.rs:108-115` (`bulk_insert`) and `crates/adapters/storage-sqlite/src/bulk_attendance.rs:145-152`

**Description:**

`bulk_insert` is decorated with `#[instrument(skip(self, rows), fields(n = rows.len(), school = %school_id))]` and the validation error message at lines 147-151 includes `expected {school_id}, got {}` — both UUIDs. UUIDs themselves are not PII, but the validation path also leaks the row index `i` and the `school_id` into the error, which is then surfaced as `DomainError::Validation`. The handoff (`docs/handoff/PHASE-1-HANDOFF.md:113-134`) describes the bulk path as the engine's "bulk-marking service" entry point — student attendance rows carry PII (student names, dates) even if the school_id alone is not PII, and the validation error does not redact that the discrepancy happened on a specific row index.

**Expected:**

Per `AGENTS.md` § "Engine Rules": "Production-ready. Real schools, real students, real money."

**Evidence:**

`crates/adapters/storage-sqlite/src/bulk_attendance.rs:147-151` `return Err(DomainError::validation(format!("bulk_insert_student_attendances: row {i} school_id mismatch (expected {school_id}, got {})", r.school_id)));` and the `#[instrument]` at line 108 includes the school_id.

---

### FINDING 30 (id: `ADAPTER-SQ-030`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/outbox.rs:198-224` (`mark_published`)

**Description:**

`mark_published` updates `published_at` using SQLite's `strftime('%Y-%m-%dT%H:%M:%fZ', 'now')` (SQL-side clock), while every other timestamp in the adapter is written by the application via `chrono::Utc::now()` (application-side clock). A single adapter now has two clocks: SQLite's `strftime` returns UTC wall-clock, but on a host whose system clock has drifted, the two will disagree and `pending` queries ordered by `enqueued_at` may interleave with `published_at` writes from a different clock. This makes post-mortem reasoning about event publication latency unreliable.

**Expected:**

Per `crates/adapters/storage-sqlite/src/outbox.rs:147` `let now = Utc::now();` — the write path uses `chrono::Utc::now()`. Consistency requires the same clock source for all timestamps.

**Evidence:**

`crates/adapters/storage-sqlite/src/outbox.rs:207-211` `let mut qb: sqlx::QueryBuilder<sqlx::Sqlite> = sqlx::QueryBuilder::new("UPDATE outbox SET published_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE event_id IN (", );`. Compare `outbox.rs:169` `.bind(now)` where `now = Utc::now()` is the application clock used for `enqueued_at`.

---

### FINDING 31 (id: `ADAPTER-SQ-031`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/event_log.rs:97-122` (`EventLog::append`) and `migrations/engine/0000_engine_core.sqlite.sql:174-188`

**Description:**

`EventLog::append` inserts a row with no `ON CONFLICT` clause; the underlying primary-key violation on duplicate `event_id` is converted to `DomainError::Infrastructure` via `StringError`. The port trait does not specify the duplicate behaviour, but the outbox trait does (`Conflict` — see ADAPTER-SQ-002). Without explicit `Conflict` mapping, callers cannot distinguish "already recorded" from "DB failure" without inspecting the error string.

**Expected:**

Consistent error semantics across sub-ports: `Conflict` for duplicate primary-key violations on event-bearing tables (outbox, event_log, idempotency).

**Evidence:**

`crates/adapters/storage-sqlite/src/event_log.rs:97-119` `sqlx::query::<sqlx::Sqlite>("INSERT INTO event_log ( ...")... .execute(&self.pool).await.map_err(|e| StringError(format!("event_log append: {e}")))?;` — no `ON CONFLICT` clause and no `match` on `UniqueViolation`.

---

### FINDING 32 (id: `ADAPTER-SQ-032`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/idempotency.rs:154-169` (`purge_older_than`)

**Description:**

`purge_older_than` executes a `DELETE FROM idempotency WHERE school_id = ?1 AND recorded_at < ?2` and returns `result.rows_affected()`. The DELETE is not wrapped in a transaction and does not `LIMIT` the batch size. A school with millions of expired records and a one-time retention sweep will issue a single huge DELETE that holds a write lock for the entire duration. SQLite serialises writes; blocking the only writer for the duration of a multi-million-row DELETE stalls every concurrent command.

**Expected:**

A batched purge (e.g. `DELETE ... WHERE ... LIMIT 1000` looped) so the writer thread is freed between batches and other commands can interleave.

**Evidence:**

`crates/adapters/storage-sqlite/src/idempotency.rs:155-168` `let result = sqlx::query::<sqlx::Sqlite>("DELETE FROM idempotency WHERE school_id = ?1 AND recorded_at < ?2").bind(school_id.as_uuid().hyphenated()).bind(cutoff.as_datetime()).execute(&self.pool).await.map_err(...)?; let n = result.rows_affected();` — no `LIMIT`, no batching.

---

### FINDING 33 (id: `ADAPTER-SQ-033`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/event_log.rs:151-194` (`build_read_query`)

**Description:**

The dynamic `WHERE` builder emits `LIMIT` via `qb.push_bind(i64::from(filter.limit))`. There is no upper-bound enforcement: a caller passing `filter.limit = u32::MAX` (the trait default is 1000 but `EventLogFilter::limit` is a public field) will issue a single query that materialises the entire school's event log. SQLite's `LIMIT` accepts up to `i64::MAX` placeholders, so the query is technically legal but operationally catastrophic.

**Expected:**

Per `crates/infra/storage/src/event_log.rs:154`: "Returns events matching `filter` ordered by `recorded_at` ascending. The cap is `filter.limit`; the adapter may enforce a lower cap for safety." — the adapter should clamp the limit.

**Evidence:**

`crates/adapters/storage-sqlite/src/event_log.rs:189-192` `if !count_only { qb.push(" ORDER BY recorded_at ASC LIMIT ").push_bind(i64::from(filter.limit)); }`. No `min(filter.limit, MAX_LIMIT)` clamp.

---

### FINDING 34 (id: `ADAPTER-SQ-034`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/audit_log.rs:164-190` (`read_for_target`)

**Description:**

`read_for_target` returns audit rows in `occurred_at ASC` order with no secondary sort by `audit_id`. The audit log's primary key is `audit_id` (a UUIDv7, which is time-ordered), but `audit_id` and `occurred_at` are produced by different sources — `audit_id` by `Uuid::now_v7()` (line 121) and `occurred_at` by the caller's `entry.occurred_at` (line 151). Two audit rows appended in the same scheduler tick will have `occurred_at` equal to the millisecond and `audit_id` differing by UUIDv7 sub-millisecond. SQLite's default sort is unstable for ties, so pagination by `LIMIT` will return arbitrary slices across ties — auditors paginating through a target's history will see inconsistent results between pages.

**Expected:**

A deterministic tiebreaker on `audit_id` for pagination.

**Evidence:**

`crates/adapters/storage-sqlite/src/audit_log.rs:178-182` `FROM audit_log WHERE school_id = ?1 AND resource_id = ?2 ORDER BY occurred_at ASC LIMIT ?3` — no secondary sort key.

---

### FINDING 35 (id: `ADAPTER-SQ-035`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/bulk_attendance.rs:205-217`

**Description:**

The unique-violation handler in `bulk_insert_into` rolls back the transaction and returns `DomainError::conflict("bulk_insert_student_attendances: duplicate (school_id, student_id, attendance_date) row")`. The rollback uses `let _ = tx.rollback().await;` (line 208 and 214), discarding any error from the rollback itself. If the rollback fails (e.g. connection broken during rollback), the adapter silently swallows the failure and returns the original error, leaving the connection in an indeterminate state. The next operation may reuse a connection that the pool believes was rolled back.

**Expected:**

The rollback error should be logged via `tracing::error!` even if the original error is preferred for return.

**Evidence:**

`crates/adapters/storage-sqlite/src/bulk_attendance.rs:208, 214` `let _ = tx.rollback().await;` (twice).

---

### FINDING 36 (id: `ADAPTER-SQ-036`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/bulk_attendance.rs:118-148` (`bulk_insert_into`)

**Description:**

`bulk_insert_into` validates `r.school_id != school_id` per row before opening the transaction. For large batches (the doc says batches of up to 40 rows), this is fine; but for a 10k-row input the function still opens one transaction holding the writer lock for the entire batch duration. The doc-string at line 28-29 claims "a partial failure rolls back all of the batches, not just the failed one" but the implementation holds the writer for the full 10k-row duration, which on SQLite is a single-writer lock that blocks every other command.

**Expected:**

Per `docs/ports/storage.md:477`: "A load test (10k attendance marks in <5s)." — the implementation holds the writer lock across the entire 10k-row insert; no test exists to verify the lock-hold time.

**Evidence:**

`crates/adapters/storage-sqlite/src/bulk_attendance.rs:160-164` `let mut tx = pool.begin().await.map_err(...)?;` and `crates/adapters/storage-sqlite/src/bulk_attendance.rs:166-218` loops all batches inside this single transaction.

---

### FINDING 37 (id: `ADAPTER-SQ-037`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/outbox.rs:46-65` (`OutboxRow`)

**Description:**

`OutboxRow` carries `recorded_at`, `enqueued_at`, `published_at`, `attempts`, `last_error` but the `to_envelope` method does not surface any of them — only `occurred_at` is mapped into the `SerializedEnvelope`. The handoff at `docs/handoff/PHASE-1-HANDOFF.md:47` notes `#[allow(dead_code)]` to silence the warning, but the practical consequence is that callers reading pending envelopes cannot see how many times the relay has retried (`attempts`) or what the last error was (`last_error`) — fields that exist in the table and are written on every state transition but are dead-on-arrival on read.

**Expected:**

Either expose retry state through the port or omit the columns from the SELECT.

**Evidence:**

`crates/adapters/storage-sqlite/src/outbox.rs:47` `#[allow(dead_code)] // `recorded_at`, `enqueued_at`, `published_at`, `attempts`, `last_error` are read for future parity tests.` and `crates/adapters/storage-sqlite/src/outbox.rs:71-87` `to_envelope` only constructs `event_id, event_type, schema_version, school_id, aggregate_id, aggregate_type, actor_id, correlation_id, causation_id, occurred_at, payload`.

---

### FINDING 38 (id: `ADAPTER-SQ-038`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/idempotency.rs:128-129`

**Description:**

`expires_at = recorded_at + Duration::days(30)` is hardcoded in the adapter. The port trait documents that retention is consumer-configurable (the engine ships no default), but the SQLite adapter unilaterally picks 30 days. A consumer that needs 7-day or 365-day retention has no adapter-level override and must patch the crate.

**Expected:**

Per `crates/infra/storage/src/idempotency.rs:104-105`: "Purges idempotency records older than the configured retention window."

**Evidence:**

`crates/adapters/storage-sqlite/src/idempotency.rs:129` `let expires_at = record.recorded_at.as_datetime() + Duration::days(30);` — no configuration hook.

---

### FINDING 39 (id: `ADAPTER-SQ-039`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/audit_log.rs:120-162` (`append`)

**Description:**

`recorded_at` is set to `Utc::now()` and `occurred_at` is set from `entry.occurred_at` (passed by the caller). The DDL allows `recorded_at` to precede `occurred_at` if a caller sets `entry.occurred_at` in the future or if the host clock has drifted. The schema has no `CHECK (recorded_at >= occurred_at)` to catch this invariant violation at the database layer.

**Expected:**

`CHECK (recorded_at >= occurred_at)` on `audit_log` and `event_log`.

**Evidence:**

`crates/adapters/storage-sqlite/src/audit_log.rs:131, 151-152` shows `recorded_at = Utc::now()` and `occurred_at = entry.occurred_at.as_datetime()`. `migrations/engine/0000_engine_core.sqlite.sql:107-129` has no `CHECK` on the `occurred_at`/`recorded_at` pair.

---

### FINDING 40 (id: `ADAPTER-SQ-040`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/event_log.rs:124-148` (`read` and `count`)

**Description:**

`read` and `count` use `WHERE recorded_at >= ?` and `WHERE recorded_at < ?` for the `since` and `until` filters (lines 181-188). The port trait's `EventLogFilter::since`/`until` fields are typed as `Timestamp` and bind `as_datetime()` (a `chrono::DateTime<Utc>`). SQLite TEXT columns sort lexicographically; ISO 8601 with a fixed `Z` suffix sorts correctly only when every value uses the same suffix. The `event_log` DDL has no `CHECK` that timestamps end with `Z` or use a fixed-length format, so a write that uses `+00:00` instead of `Z` will sort incorrectly against a write that uses `Z`.

**Expected:**

A `CHECK` constraint enforcing ISO 8601 UTC suffix on every timestamp column.

**Evidence:**

`crates/adapters/storage-sqlite/src/event_log.rs:181-188` `if let Some(since) = filter.since { qb.push(" AND recorded_at >= ").push_bind(since.as_datetime()); } if let Some(until) = filter.until { qb.push(" AND recorded_at < ").push_bind(until.as_datetime()); }` — no format constraint at the schema layer (`migrations/engine/0000_engine_core.sqlite.sql:184-185`).

---

### FINDING 41 (id: `ADAPTER-SQ-041`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/connection.rs:68-80, 114-126`

**Description:**

The `after_connect` hook issues `PRAGMA journal_mode = WAL` first, then `PRAGMA synchronous = NORMAL`, then `PRAGMA foreign_keys = ON`. The order matters because `journal_mode` for a file-backed DB is sticky (changing it requires an exclusive lock and may rewrite the file). The hook does not verify that the `journal_mode` PRAGMA actually succeeded; a SQLite error or warning would be silently ignored.

**Expected:**

Verify the PRAGMA result for `journal_mode` (e.g. `PRAGMA journal_mode` after the SET to confirm the value is `wal`).

**Evidence:**

`crates/adapters/storage-sqlite/src/connection.rs:68-80` and `:114-126` `sqlx::query("PRAGMA journal_mode = WAL").execute(&mut *conn).await?;` etc. — no verification of the round-tripped value.

---

### FINDING 42 (id: `ADAPTER-SQ-042`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/storage.rs:38-40`

**Description:**

`SCHEMA_VERSION: u32 = 1` is a hard-coded constant with no link to a migration-tracking table. There is no `schema_migrations` or `schema_registry.version` row that records the applied version, so `already_at_version` in the `MigrationReport` is structurally incapable of ever being `true`. The handoff claims idempotency via `IF NOT EXISTS`, but the report cannot reflect that idempotency.

**Expected:**

A migration-tracking table (e.g. `schema_migrations(version INT PRIMARY KEY, applied_at TEXT)`) so the adapter can distinguish a no-op run from a fresh migration.

**Evidence:**

`crates/adapters/storage-sqlite/src/storage.rs:40` `const SCHEMA_VERSION: u32 = 1;` and `:127-132` `Ok(MigrationReport { ..., already_at_version: false })` (always false).

---

### FINDING 43 (id: `ADAPTER-SQ-043`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/error.rs:29-33`

**Description:**

The `From<StringError> for educore_core::error::DomainError` impl wraps the `StringError` as `DomainError::infrastructure`. This is the conversion the entire crate relies on for `?` propagation. The conversion drops the original `sqlx::Error` type entirely; callers that want to match on `sqlx::Error::Database(...)` variants (to map unique-violations to `Conflict`, for example) cannot. This is the structural reason behind ADAPTER-SQ-002 and ADAPTER-SQ-031: every error path returns `Infrastructure` because the typed error chain is broken at the conversion boundary.

**Expected:**

Error wrappers that preserve the underlying `sqlx::Error` (or at least its `kind()`) so adapters can pattern-match on `UniqueViolation`, `ForeignKeyViolation`, etc., and produce `Conflict` / `Validation` / `Infrastructure` accordingly.

**Evidence:**

`crates/adapters/storage-sqlite/src/error.rs:29-33` `impl From<StringError> for educore_core::error::DomainError { fn from(e: StringError) -> Self { educore_core::error::DomainError::infrastructure(e) } }` — drops the structured error information.

---

### FINDING 44 (id: `ADAPTER-SQ-044`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/event_log.rs:55, 95` and `crates/adapters/storage-sqlite/src/outbox.rs:77, 145`

**Description:**

The `event_version` column is bound as `i32` on both write and read paths with a `try_from(...,).unwrap_or(0)` clamp. The DDL declares the column as `INTEGER` (8-byte signed) which maps to `i64` in sqlx. Binding `i32` succeeds only because sqlx widens small integers; binding a value > `i32::MAX` would fail with an out-of-range error and the adapter would convert that to `Infrastructure` rather than `Validation`.

**Expected:**

Bind `i64` (matching the DDL's 8-byte `INTEGER`) and surface overflow as `Validation`.

**Evidence:**

`crates/adapters/storage-sqlite/src/event_log.rs:95` `let event_version = i32::try_from(entry.schema_version).unwrap_or(0);` and `crates/adapters/storage-sqlite/src/event_log.rs:107` `.bind(event_version)`. `migrations/engine/0000_engine_core.sqlite.sql:67, 177` declares `event_version INTEGER NOT NULL`.

---

### FINDING 45 (id: `ADAPTER-SQ-045`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/audit_log.rs:43-67` (`AuditLogRow`)

**Description:**

`AuditLogRow` declares `actor_type: String` and `source: String` (lines 50, 66) as the only string columns on the read path. The port trait's `AuditLogEntry` does not carry these fields (per the doc-vs-code drift at ADAPTER-SQ-020), so they are read but discarded. The `to_entry` function explicitly hardcodes `active_status: ActiveStatus::Active` (line 85) despite the DDL having no `active_status` column.

**Expected:**

Either the port trait carries these fields and the adapter round-trips them, or the SELECT avoids them. The current state is "read but drop", which signals an incomplete implementation.

**Evidence:**

`crates/adapters/storage-sqlite/src/audit_log.rs:69-93` `to_entry` constructs an `AuditLogEntry` from a row but does not populate `actor_type`, `source`, `recorded_at`, `audit_id`, `ip`, `user_agent`, `session_id`, or `command_id` — none of which the port struct carries.

---

### FINDING 46 (id: `ADAPTER-SQ-046`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/storage.rs:155-167` (`watch_changes`)

**Description:**

`watch_changes` is documented in the port contract as the entry point for the sync engine ("MySQL / SQLite: poll the outbox table on a timer"). The SQLite implementation returns `DomainError::not_supported(...)` (line 164-166), but the Phase 1 handoff claims it is a "Phase 1 stub. A future PR will poll the outbox on a timer (per `docs/ports/storage.md` 'MySQL / SQLite: poll the outbox table on a timer')." The port trait already has a default implementation that returns `NotSupported`, so the explicit override is redundant. The override is also misleading because it overrides a perfectly good default with an identical-typed error string.

**Expected:**

Either delete the override and rely on the trait default, or implement the polling loop. The current code duplicates the default's behaviour without adding value.

**Evidence:**

`crates/adapters/storage-sqlite/src/storage.rs:155-167` and the default at `crates/infra/storage/src/port.rs:115-120` `async fn watch_changes(&self, filter: ChangeFilter) -> Result<ChangeStream> { let _ = filter; Err(educore_core::error::DomainError::not_supported("StorageAdapter::watch_changes is not supported by this adapter")) }`.

---

### FINDING 47 (id: `ADAPTER-SQ-047`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/storage.rs:1-38` (`lib.rs` doc-comment)

**Description:**

The crate's `lib.rs` documents the adapter against `ADR-017` (`Multi-writer scenarios are out-of-scope: SQLite is the engine's embedded / offline mode (per ADR-017).`). ADR-017 in `docs/decisions/ADR-017-SurrealDBFirst.md` is about the SurrealDB-first strategy, not about SQLite's single-writer limitation. The cross-reference is wrong: there is no `ADR-017-SurrealDBFirst` that documents SQLite's single-writer deployment model. The doc-link at line 15 is broken.

**Expected:**

A correct ADR reference (or no ADR reference at all).

**Evidence:**

`crates/adapters/storage-sqlite/src/lib.rs:15` `//! [`ADR-017`]: ../../docs/decisions/ADR-017-SurrealDBFirst.md` — the link path resolves to `docs/decisions/ADR-017-SurrealDBFirst.md`, which is unrelated to SQLite's single-writer constraints.

---

### FINDING 48 (id: `ADAPTER-SQ-048`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/audit_log.rs:85` (`AuditLogRow::to_entry`)

**Description:**

`to_entry` always returns `active_status: ActiveStatus::Active` despite the audit log being explicitly an append-only, write-once store where rows should never be soft-deleted. The hardcoded `Active` is correct in spirit but the lack of a schema column means there is no physical way to record the row's "retired" state if a future auditor flag demands it. The port contract at `crates/infra/storage/src/audit.rs:94-96` documents that "Audit rows are never hard-deleted; this is `Retired` when an auditor marks a row as superseded."

**Expected:**

An `active_status` column on `audit_log` with a `CHECK (active_status IN (0, 1))` constraint, matching the spec.

**Evidence:**

`crates/adapters/storage-sqlite/src/audit_log.rs:85` `active_status: ActiveStatus::Active,` (hardcoded). `migrations/engine/0000_engine_core.sqlite.sql:107-129` — the `audit_log` table does not have an `active_status` column.

---

### FINDING 49 (id: `ADAPTER-SQ-049`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/event_log.rs:181-188`

**Description:**

The `since` and `until` filters are applied to `recorded_at` (the persistence time), not `occurred_at` (the event-time). The handoff claims this matches the spec, but the spec at `docs/schemas/event-schema.md:6` says consumers query by `(school_id, [event_type], since, until)` where `since/until` are event-time. The SQLite adapter's filtering on `recorded_at` returns events in the order they were persisted, which can be a few seconds to a few minutes (or hours, if a relay backlog exists) after they occurred. Analytics consumers expecting "events from the last hour" will silently miss late-arriving events.

**Expected:**

Either filter on `occurred_at` (matching the spec) or document the drift explicitly.

**Evidence:**

`crates/adapters/storage-sqlite/src/event_log.rs:181-188` `if let Some(since) = filter.since { qb.push(" AND recorded_at >= ").push_bind(since.as_datetime()); } if let Some(until) = filter.until { qb.push(" AND recorded_at < ").push_bind(until.as_datetime()); }`. The index that backs this query (`idx_event_log_school_time ON event_log(school_id, occurred_at)` at `migrations/engine/0000_engine_core.sqlite.sql:190-191`) is on `occurred_at`, so the `recorded_at` filter will not use the index — every event-log query is a sequential scan over the school's events.

---

### FINDING 50 (id: `ADAPTER-SQ-050`)

- **Source:** `docs/audit_reports/findings/wave3-storage-sqlite.md`
- **Severity:** Low
- **Area:** adapters
- **Location:** `crates/adapters/storage-sqlite/src/transaction.rs:75-81` (`Debug` for `SqliteTransaction`)

**Description:**

The `Debug` impl for `SqliteTransaction` prints the school field from `self.outbox.school()` but the struct has four sub-port handles, each with its own `school` field. The `Debug` impl is consistent today only because every constructor path passes the same `school` to every sub-port (`SqliteTransaction::new` lines 86-91). A future change that introduces a sub-port with a different school scope (e.g. cross-tenant audit reads) will silently produce a misleading `Debug` output.

**Expected:**

Either iterate every sub-port's school, or document the invariant.

**Evidence:**

`crates/adapters/storage-sqlite/src/transaction.rs:75-81` `impl fmt::Debug for SqliteTransaction { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.debug_struct("SqliteTransaction").field("school", &self.outbox.school()).finish_non_exhaustive() } }`.

---


## Storage — SurrealDB (target id prefix: `ADAPTER-SR`)

**Path:** `crates/adapters/storage-surrealdb/`  
**Total findings:** 38 (11 critical, 21 high, 6 medium, 0 low)


### FINDING 1 (id: `ADAPTER-SR-001`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/storage.rs:88-108` (`migrate`)

**Description:**

The adapter exposes a `migrate()` method on `StorageAdapter`, but every consumer-facing doc (`AGENTS.md:544, 561`, `README.md:173`, `docs/schemas/sql-dialects/README.md:193-198`, `docs/schemas/sql-dialects/surrealdb.md:9-10, 505-511, 752`, `docs/build-plan.md:119, 175-186`, `docs/architecture.md:322`, `migrations/engine/README.md:11`, `CONTRIBUTING.md:502`) refers to the runtime entry point as `storage.create_schema().await`. The trait method is named `migrate()` (per `docs/ports/storage.md:21, 174`, which is the *consumer migration runner*, not the engine's schema emission). The SurrealDB adapter's `migrate()` is in fact performing the engine's `create_schema()` work (executing the cross-cutting DDL); the consumer-facing API name does not exist on the trait.

**Expected:**

`docs/build-plan.md:175-179` lists the trait surface as `("create_schema", "apply_command", "query", "begin_tx", ...)`; `docs/architecture.md:322` says the schema is emitted "via `storage.create_schema().await`". A `create_schema()` method on `StorageAdapter` is the contract.

**Evidence:**

`crates/adapters/storage-surrealdb/src/storage.rs:88`:
  ```rust
  async fn migrate(&self) -> Result<MigrationReport> {
  ```
  And `crates/infra/storage/src/port.rs:44`:
  ```rust
  async fn migrate(&self) -> Result<MigrationReport>;
  ```
  No `create_schema` method exists in the entire crate (`grep -n "fn create_schema" crates/adapters/storage-surrealdb/` returns no results).

---

### FINDING 10 (id: `ADAPTER-SR-010`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/outbox.rs:39-69` (`OutboxRow::from_envelope`)

**Description:**

`OutboxRow::from_envelope` sets `recorded_at: Datetime::from(env.occurred_at.as_datetime())` — `recorded_at` is bound to the *envelope's* `occurred_at` instead of the wall-clock time of the persistence. The DDL declares `recorded_at` as the persistence time (a separate column from `occurred_at`), and the engine invariant is that `recorded_at >= occurred_at` (it captures ingestion latency between the producer and the outbox writer). Binding both to the same value obliterates that invariant.

**Expected:**

"`recorded_at`: Wall-clock time of the persistence (≥ `occurred_at`)" (`crates/infra/storage/src/event_log.rs:73`) and the DDL column pair `occurred_at ... recorded_at ...` (`migrations/engine/0000_engine_core.surreal.surql:89-90`).

**Evidence:**

`crates/adapters/storage-surrealdb/src/outbox.rs:64-65`:
  ```rust
  occurred_at: Datetime::from(env.occurred_at.as_datetime()),
  recorded_at: Datetime::from(env.occurred_at.as_datetime()),  // BUG
  ```

---

### FINDING 11 (id: `ADAPTER-SR-011`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/storage.rs:1-180` (entire `StorageAdapter` impl)

**Description:**

The `StorageAdapter` trait in `docs/ports/storage.md:17-89` enumerates ~22 per-aggregate repository methods (`students()`, `guardians()`, `classes()`, …, one per aggregate across 15 domains, ~80+ total). The actual port trait at `crates/infra/storage/src/port.rs:34-150` exposes only 5 methods (`begin`, `migrate`, `ping`, `close`, `bulk_insert_student_attendances`) plus 4 sync primitives — no per-aggregate repository handles. The SurrealDB adapter implements the actual trait (no repository methods), meaning **none** of the documented per-aggregate repository handles are implemented. The dialect spec promises `SurrealStorageAdapter::create_<table>_ddl()` per aggregate; no such method exists in the crate.

**Expected:**

"`fn students(&self) -> Arc<dyn StudentRepository>;` and ~21 sibling methods, 'one handle per aggregate, across all 15 domains (~80+ total)'" (`docs/ports/storage.md:50`). Each adapter must translate the macro-emitted `QueryNode` AST into a SurrealDB execution plan.

**Evidence:**

`crates/adapters/storage-surrealdb/src/storage.rs:75-172` implements only `begin`, `migrate`, `ping`, `close`, `watch_changes`, `apply_snapshot`, `cursor_for`, `advance_cursor`. `grep -n 'students\|guardians\|classes\|sections' crates/adapters/storage-surrealdb/src/` returns no repository handle of any kind.

---

### FINDING 2 (id: `ADAPTER-SR-002`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/storage.rs:88-108` (`migrate`)

**Description:**

The `migrate()` implementation only loads `migrations/engine/0000_engine_core.surreal.surql` (6 engine cross-cutting tables) and executes the DDL once. It does not walk any macro-emitted AST to emit the ~310 domain tables the engine claims to ship, and it does not honour `docs/schemas/sql-dialects/surrealdb.md`'s `SurrealStorageAdapter::create_<table>_ddl()` per-aggregate contract at all. The dialect spec is explicit: "For the ~310 domain tables, the adapter walks the macro-emitted AST and renders each table's DDL string at runtime." None of the ~310 domain tables are emitted.

**Expected:**

"The engine emits DDL **at schema-creation time** (once per process lifetime, via `storage.create_schema().await`) from a typed macro AST" (`docs/architecture.md:321-324`); "the adapter walks the macro-emitted AST to render the ~310 domain tables at `create_schema()` time using SurrealDB's `DEFINE TABLE` / `DEFINE FIELD` / `DEFINE INDEX` DDL" (`docs/build-plan.md:178-181`).

**Evidence:**

`crates/adapters/storage-surrealdb/src/storage.rs:26-27` `const SCHEMA_SQL: &str = include_str!("../../../../migrations/engine/0000_engine_core.surreal.surql");` and `crates/adapters/storage-surrealdb/src/storage.rs:97-101` only executes `SCHEMA_SQL` via `self.conn.db().query(SCHEMA_SQL).await`. The crate has no `create_schema()`, no AST walk, no `EntityDescriptor` traversal, and no domain-table emission code anywhere under `src/`.

---

### FINDING 3 (id: `ADAPTER-SR-003`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/transaction.rs:86-97` (`commit` / `rollback`)

**Description:**

The `commit` and `rollback` impls are no-ops that only flip the `done` / `rolled_back` atomic flag. The SurrealDB SDK does not expose explicit transaction control (per the file's own module-level doc at lines 1-8), so the entire transactional unit-of-work contract is non-functional. The file's own comment explicitly states: "A future PR will use the SurrealDB 3.x transaction API for true atomicity." In production this means a caller that invokes `tx.outbox().append(...)` inside a transaction and then `tx.commit()` will see the outbox row durable immediately and the "commit" step is purely cosmetic; a caller that wants to roll back has no way to undo writes.

**Expected:**

"Commits the transaction. All outbox appends, aggregate mutations, audit log writes, idempotency records, and event log rows become durable." (`crates/infra/storage/src/transaction.rs:35-37`); and the equivalent for `rollback`.

**Evidence:**

`crates/adapters/storage-surrealdb/src/transaction.rs:87-97` `async fn commit(self: Box<Self>) -> Result<()> { self.done.store(true, std::sync::atomic::Ordering::SeqCst); Ok(()) }` and `async fn rollback(self: Box<Self>) -> Result<()> { self.rolled_back.store(true, ...); self.done.store(true, ...); Ok(()) }` — neither call interacts with the database.

---

### FINDING 4 (id: `ADAPTER-SR-004`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/storage.rs:129-140` (`watch_changes`)

**Description:**

`watch_changes` returns an empty `ChangeStream` (`futures::stream::empty()`) instead of `DomainError::NotSupported`. The default-impl contract per `docs/ports/storage.md:112-116` and `crates/infra/storage/src/port.rs:115-120` is: sync primitives return `NotSupported`; "the sync engine, when it tries to subscribe on a non-sync adapter, fails loudly at startup — not silently at runtime — so consumers see the configuration problem immediately." The SurrealDB implementation reports success and returns an empty stream, masking the configuration problem and letting the sync engine start up against an adapter that is doing nothing. ADR-017 lists SurrealDB `LIVE SELECT` as the supported implementation path: "SurrealDB supports all four natively."

**Expected:**

"Default impls on the trait return `DomainError::NotSupported('sync primitives require the sync feature and a sync-capable adapter'). The sync engine, when it tries to subscribe on a non-sync adapter, fails loudly at startup" (`docs/ports/storage.md:112-116`). And per `ADR-017-SurrealDBFirst.md` § "Parity surface", SurrealDB `watch_changes` is `✓ (SurrealDB live queries)`.

**Evidence:**

`crates/adapters/storage-surrealdb/src/storage.rs:129-140`:
  ```rust
  async fn watch_changes(&self, _filter: ChangeFilter) -> Result<ChangeStream> {
      // Phase 0 stub. A future PR will use SurrealDB's
      // `LIVE SELECT` to drive a real change feed.
      if self.closed.load(std::sync::atomic::Ordering::SeqCst) {
          return Err(DomainError::conflict(
              "StorageAdapter::watch_changes called on a closed adapter",
          ));
      }
      let s = futures::stream::empty::<std::result::Result<ChangeEvent, DomainError>>();
      let pinned = Box::pin(s);
      Ok(ChangeStream { inner: pinned })
  }
  ```

---

### FINDING 5 (id: `ADAPTER-SR-005`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/storage.rs:151-171` (`cursor_for` / `advance_cursor`)

**Description:**

Both `cursor_for` and `advance_cursor` silently override the trait default of `DomainError::NotSupported`. `cursor_for` returns `Ok(VersionCursor(0))` (hard-coded) and `advance_cursor` returns `Ok(())` (no-op). The default-impl contract is the sync engine's safety net: non-sync adapters must fail loudly at startup. The SurrealDB implementation reports success, masking configuration problems and letting the sync engine start up against an adapter that is actually doing nothing. ADR-017 lists `cursor_for` / `advance_cursor` as `✓` for SurrealDB.

**Expected:**

Same as ADAPTER-SR-004: "Default impls on the trait return `DomainError::NotSupported`. … The sync engine, when it tries to subscribe on a non-sync adapter, fails loudly at startup" (`docs/ports/storage.md:112-116`). And per `ADR-017-SurrealDBFirst.md` § "Parity surface", SurrealDB supports both.

**Evidence:**

`crates/adapters/storage-surrealdb/src/storage.rs:151-161` `async fn cursor_for(&self, _school_id: SchoolId) -> Result<VersionCursor> { ... Ok(VersionCursor(0)) }` and `crates/adapters/storage-surrealdb/src/storage.rs:163-171` `async fn advance_cursor(&self, _school_id: SchoolId, _to: VersionCursor) -> Result<()> { ... Ok(()) }` — both return success instead of `DomainError::not_supported(...)`.

---

### FINDING 6 (id: `ADAPTER-SR-006`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/storage.rs:75-86, 88-108, 110-122, 124-127, 129-140, 151-161, 163-171`

**Description:**

Every `StorageAdapter` method (`begin`, `migrate`, `ping`, `close` (implicitly), `watch_changes`, `cursor_for`, `advance_cursor`) returns `DomainError::Conflict` when the adapter is closed. The port contract mandates `DomainError::Infrastructure`. Returning `Conflict` is structurally wrong (closing the connection is not a state conflict) and breaks error-handling callers that match on the `Infrastructure` variant to surface a degraded-storage alert.

**Expected:**

"`close(self: Box<Self>) -> Result<()>; … After `close`, any further call returns `Err(Infrastructure)`." (`crates/infra/storage/src/port.rs:52-53` and `docs/ports/storage.md:23`).

**Evidence:**

`crates/adapters/storage-surrealdb/src/storage.rs:79-82, 90-93, 112-115, 133-136, 153-156, 165-168` all call `DomainError::conflict("...")` instead of `DomainError::infrastructure(...)` after the `self.closed.load(SeqCst)` check.

---

### FINDING 7 (id: `ADAPTER-SR-007`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/idempotency.rs:54-77` (`IdempotencyRow::to_record`)

**Description:**

`IdempotencyRow::to_record` calls `Box::leak(self.command_type.clone().into_boxed_str())` on every read. The port struct's `command_type: &'static str` field forces this leak. In a long-running process serving many idempotency lookups the heap grows without bound — a slow but unbounded memory leak in production code. The byte pattern is `Box::leak(string) -> &'static str`; every call leaks a fresh `Box<str>` that the allocator never reclaims.

**Expected:**

Per `AGENTS.md` § "Type Safety": "No `#[allow(dead_code)]` or `_var` prefixes to silence the compiler. Delete unused code, wire it in, or open a follow-up issue." Adapter code must not leak memory per-call.

**Evidence:**

`crates/adapters/storage-surrealdb/src/idempotency.rs:69` `command_type: Box::leak(self.command_type.clone().into_boxed_str()),` (the `IdempotencyRecord` struct's `command_type: &'static str` field at `crates/infra/storage/src/idempotency.rs:31` forces this allocation).

---

### FINDING 8 (id: `ADAPTER-SR-008`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/outbox.rs:180-226` (`Outbox::append`)

**Description:**

`Outbox::append` uses an `INSERT INTO outbox { ... }` statement and surfaces the underlying SurrealDB error (which includes a unique-constraint violation on `event_id`) as `DomainError::Infrastructure`. The port contract requires `DomainError::Conflict` on a duplicate `(school_id, event_id)`. The adapter silently downgrades a contract-mandated domain error to an infrastructure error.

**Expected:**

"`Conflict` if an envelope with the same `event_id` was already appended in the same school." (`crates/infra/storage/src/outbox.rs:99-101`).

**Evidence:**

`crates/adapters/storage-surrealdb/src/outbox.rs:184-219` `self.db.query("INSERT INTO outbox { ... }").await.map_err(|e| StringError(format!("outbox append: {e}")))?;` — no match on the SurrealDB error variant to map a uniqueness violation to `DomainError::conflict(...)`.

---

### FINDING 9 (id: `ADAPTER-SR-009`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** Critical
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/idempotency.rs:126-152` (`Idempotency::record`)

**Description:**

`Idempotency::record` does a plain `INSERT INTO idempotency { ... }` and returns `DomainError::Infrastructure` on a unique-constraint violation. The port contract requires `DomainError::Conflict` when a record with the same composite key exists with a different outcome, and `Ok(())` only when the new row is identical. The current behaviour never returns `Conflict`; a duplicate insert is indistinguishable from a DB failure.

**Expected:**

"Stores `record`. Returns `Err(Conflict)` if a record with the same `(school_id, command_type, idempotency_key)` already exists with a different outcome. Returns `Ok(())` if the record is a no-op write (same key, same outcome hash) — the engine uses this for at-least-once delivery of retries." (`crates/infra/storage/src/idempotency.rs:94-100`).

**Evidence:**

`crates/adapters/storage-surrealdb/src/idempotency.rs:127-149`:
  ```rust
  async fn record(&self, record: IdempotencyRecord) -> Result<()> {
      let row = IdempotencyRow::from_record(&record);
      let _ = self.db.query("INSERT INTO idempotency { ... }")...await
          .map_err(|e| StringError(format!("idempotency record: {e}")))?;
      Ok(())
  }
  ```
  No `match` on the SurrealDB error; no `exists()` check before insert.

---

### FINDING 12 (id: `ADAPTER-SR-012`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/storage.rs:88-108` (`migrate`)

**Description:**

`migrate()` hard-codes `MigrationReport { statements_executed: 0, already_at_version: false, ... }`. The `statements_executed` field exists to report the actual count of statements applied (telemetry, migration-time SLOs, idempotency verification) and the adapter discards the SurrealDB query result without inspecting it. `already_at_version` is always `false`, even when re-running `migrate()` on an already-migrated database — so the report cannot be used by callers to distinguish a first run from a no-op run.

**Expected:**

Per `crates/infra/storage/src/change_stream.rs:243-255`: "`statements_executed`: The number of statements executed (DDL or DML)." and "`already_at_version`: Whether the migration was a no-op (already at `version`)."

**Evidence:**

`crates/adapters/storage-surrealdb/src/storage.rs:97-107`:
  ```rust
  self.conn.db().query(SCHEMA_SQL).await.map_err(DomainError::infrastructure)?;
  Ok(MigrationReport {
      version: SCHEMA_VERSION,
      statements_executed: 0,        // never updated
      duration: start.elapsed(),
      already_at_version: false,     // never updated
  })
  ```

---

### FINDING 13 (id: `ADAPTER-SR-013`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/outbox.rs:40-58` (`OutboxRow::from_envelope`) and `crates/adapters/storage-surrealdb/src/outbox.rs:74-101` (`OutboxRow::to_envelope`)

**Description:**

Both `OutboxRow::from_envelope` and `OutboxRow::to_envelope` use `i32::try_from(env.schema_version).unwrap_or(0)` and `u32::try_from(self.event_version).unwrap_or(0)` to silently clamp `schema_version` on overflow or negative-value round-trip. The engine's invariant is that `schema_version` is a small positive integer, but the silent fallback to `0` discards data without surfacing the error — a caller that has produced a malformed envelope will not see `Err(Validation)`, and downstream consumers will silently treat the event as schema v0, which may have an unrelated payload shape.

**Expected:**

Per `AGENTS.md` § "Type Safety": "No `as` casts that truncate or lose data. Use `TryFrom` / `TryInto` with proper error handling." and "All public APIs return `Result` for fallible operations."

**Evidence:**

`crates/adapters/storage-surrealdb/src/outbox.rs:57` `event_version: i32::try_from(env.schema_version).unwrap_or(0),` and `:91` `schema_version: u32::try_from(self.event_version).unwrap_or(0),`.

---

### FINDING 14 (id: `ADAPTER-SR-014`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/event_log.rs:60-62, 84-88` (`EventRow` round-trip)

**Description:**

`EventRow::to_entry` returns `ActiveStatus::Retired` for every value of `self.active_status` that is not the literal string `"active"`. The `from_entry` writes `entry.active_status.to_string()` which for `ActiveStatus::Active` is `"Active"` (capital A, per the Display impl) — but the read side's match arm `"active" => ActiveStatus::Active` is lower-case. A round-trip therefore corrupts `active_status` from `Active` to `Retired` on every read.

**Expected:**

Per `crates/infra/storage/src/event_log.rs:81-82`: `pub active_status: ActiveStatus,`. A write-then-read must round-trip the value exactly.

**Evidence:**

`crates/adapters/storage-surrealdb/src/event_log.rs:54` `active_status: entry.active_status.to_string(),` (writer serialises as `Display`, capitalised). `:70-73`:
  ```rust
  let active_status = match self.active_status.as_str() {
      "active" => ActiveStatus::Active,
      _ => ActiveStatus::Retired,
  };
  ```
  (reader compares against lower-case). Compare `audit.rs:96-99` which has the same lower-case-vs-upper-case mismatch in the audit row's `active_status`.

---

### FINDING 15 (id: `ADAPTER-SR-015`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/audit.rs:85-114` (`AuditRow::to_entry`)

**Description:**

`AuditRow::to_entry` always returns `ActiveStatus::Retired` for any `active_status` string that is not the literal lower-case `"active"`. The writer side at `AuditRow::from_entry` (line 71) sets `active_status: entry.active_status.to_string()` — `ActiveStatus`'s `Display` impl returns `"Active"` (capital A, per `crates/infra/core/src/value_objects.rs`'s standard pattern). Every read therefore returns `Retired` for an `Active` audit row, breaking the soft-delete semantics that the audit-log is designed to enforce.

**Expected:**

A write-then-read round-trip on `active_status` must preserve the value exactly.

**Evidence:**

`crates/adapters/storage-surrealdb/src/audit.rs:71` `active_status: entry.active_status.to_string(),` (writer) and `crates/adapters/storage-surrealdb/src/audit.rs:96-99` (reader with lower-case match arm).

---

### FINDING 16 (id: `ADAPTER-SR-016`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/outbox.rs:182-220` (`Outbox::append`)

**Description:**

`Outbox::append` issues an `INSERT INTO outbox { ... enqueued_at: time::now(), published_at: NONE, attempts: 0, last_error: NONE }` — `enqueued_at` is set by SurrealDB's server-side `time::now()` while every other timestamp in the adapter is set by the application's `chrono::Utc::now()`. The outbox DDL allows either: the spec does not mandate a side. But mixing the two clock sources in the same database means a single event row's `occurred_at` (application) and `enqueued_at` (server) can disagree on the order, breaking the engine's `recorded_at >= occurred_at` invariant and any post-mortem analysis of outbox-drain latency.

**Expected:**

Per `crates/infra/storage/src/outbox.rs:104-108`: "The order is FIFO by append time within a school." The clock source for `enqueued_at` must be the same as the application clock used for `occurred_at` and `recorded_at` to make the FIFO ordering meaningful.

**Evidence:**

`crates/adapters/storage-surrealdb/src/outbox.rs:200` `enqueued_at: time::now(),` is set server-side by the SurrealDB engine, while `:64-65` `occurred_at` / `recorded_at` are set from the application-supplied `env.occurred_at`. The `enqueued_at` should be the adapter's `Utc::now()` converted to `Datetime` and bound as a parameter.

---

### FINDING 17 (id: `ADAPTER-SR-017`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/outbox.rs:257-276` (`Outbox::mark_published`)

**Description:**

`Outbox::mark_published` uses server-side `time::now()` for the `published_at` column (`SET published_at = time::now()`) instead of the application clock. Same clock-mixing issue as ADAPTER-SR-016. Additionally, the helper is unaware of a `last_error` / `attempts` increment that the relay would normally perform when an envelope fails to publish — the implementation only updates `published_at`, never `attempts` or `last_error`. The port trait's `mark_published` is the only feedback path the relay has, and the column semantics in the DDL (`attempts`, `last_error`) are designed to be incremented on this path.

**Expected:**

The `outbox` DDL columns `attempts INT ASSERT $value != NONE AND $value >= 0 VALUE 0` and `last_error option<string>` (`migrations/engine/0000_engine_core.surreal.surql:94-95`) imply the adapter increments `attempts` on every `mark_published` call (whether success or failure). The current impl never touches either column.

**Evidence:**

`crates/adapters/storage-surrealdb/src/outbox.rs:265-268`:
  ```rust
  "UPDATE outbox SET published_at = time::now() \
   WHERE event_id IN $ids",
  ```
  No `attempts = attempts + 1`, no `last_error` handling.

---

### FINDING 18 (id: `ADAPTER-SR-018`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/event_log.rs:218-273` (`EventLog::count`)

**Description:**

`EventLog::count` issues `SELECT count() AS n FROM event_log WHERE school_id = $school AND {type_filter}{since_clause}{until_clause} GROUP ALL` and returns the `n` of the first row. The `aggregate_id` filter that the `read` method applies (`event_log.rs:189-192` `format!(" AND aggregate_id = SurrealUuid::from('{}')", a)`) is missing from `count`. Two semantically equivalent API methods have different filter coverage: a caller that sets `filter.aggregate_id = Some(uuid)` and then calls `count()` will get a count that is **larger** than the count of rows `read()` would return, and downstream consumers (cursor sizing, analytics) will be silently wrong.

**Expected:**

`count()` must apply exactly the same filters as `read()` (minus `limit`). Per the port doc: "Returns the count of events for `school_id` matching `filter` (ignoring `limit`)." (`crates/infra/storage/src/event_log.rs:156-158`).

**Evidence:**

`crates/adapters/storage-surrealdb/src/event_log.rs:251-255`:
  ```rust
  "SELECT count() AS n FROM event_log \
   WHERE school_id = $school AND {type_filter}{since_clause}{until_clause} \
   GROUP ALL"
  ```
  Compared with `:193-201` (`read`):
  ```rust
  "SELECT event_id, ... FROM event_log \
   WHERE school_id = $school AND {type_filter}{since_clause}{until_clause}{agg_clause} \
   ORDER BY recorded_at ASC \
   LIMIT $limit"
  ```
  The `agg_clause` is missing from the `count` query.

---

### FINDING 19 (id: `ADAPTER-SR-019`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/event_log.rs:156-216` (`EventLog::read`); 218-273 (`EventLog::count`)

**Description:**

Both `read` and `count` build a SurrealQL query by string-formatting the `event_types` filter via `format!("'{t}'")`. A caller that supplies an `event_type` containing a single-quote character (e.g. `"academic.student's.admitted"`) breaks out of the string literal and injects SurrealQL. The same applies to the `since_clause` / `until_clause` / `agg_clause` constructions, which `format!` user-supplied UUIDs and timestamps directly into the query string. The port struct's `event_type: String` is user-controllable, so this is an injection vector that reaches the storage layer.

**Expected:**

Per `AGENTS.md` § "Type Safety": "No `serde_json::Value` in domain code. Use typed wrappers." and the port trait's `EventLogFilter::event_types: Vec<String>` is documented as `String` (not `&'static str`) "so the type can be deserialised from JSON / MessagePack" (`crates/infra/storage/src/event_log.rs:90-94`); this means user input can flow into it. The query must use parameterised binds for all user-controllable values.

**Evidence:**

`crates/adapters/storage-surrealdb/src/event_log.rs:163-167`:
  ```rust
  let types = filter.event_types.iter().map(|t| format!("'{t}'")).collect::<Vec<_>>().join(", ");
  format!("event_type IN [{types}]")
  ```
  and `:189-192`:
  ```rust
  .map(|a| format!(" AND aggregate_id = SurrealUuid::from('{}')", a))
  ```
  and `:172-178, 181-188` (the `since_clause` / `until_clause` format RFC-3339 strings inline).

---

### FINDING 20 (id: `ADAPTER-SR-020`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/idempotency.rs:100-124` (`Idempotency::lookup`)

**Description:**

`Idempotency::lookup` ignores `filter.aggregate_id` semantics (the lookup doesn't filter on it because the trait has no `aggregate_id` slot, OK) but, more critically, does not honour the "exists with a different outcome" semantic at read time. The trait contract says `lookup` returns the prior outcome; the dispatcher then re-checks `affected_aggregate_ids` to detect "same idempotency key, but different target" misuse. The adapter's `IdempotencyRow` struct **does** carry `affected_aggregate_ids: Option<Vec<SurrealUuid>>` (line 32) and `to_record` returns it (lines 62-66) — but the lookup query (lines 105-110) only SELECTs the 7 columns needed and the deserialised row's `affected_aggregate_ids` is then re-serialised. The round-trip preserves the column, but the `outcome` is not re-hashed for "same key, different outcome" — meaning the engine cannot tell a "same key, same outcome" replay from a "same key, different outcome" misuse without re-comparing in application code (which the spec says the storage adapter should surface as `Conflict`).

**Expected:**

"`record`: Returns `Err(Conflict)` if a record with the same `(school_id, command_type, idempotency_key)` already exists with a different outcome." (`crates/infra/storage/src/idempotency.rs:94-100`). The lookup must return enough information to make that decision; the current round-trip returns `Option<IdempotencyRecord>` but `record` doesn't pre-check.

**Evidence:**

`crates/adapters/storage-surrealdb/src/idempotency.rs:126-151` `record` does a plain `INSERT INTO idempotency { ... }` with no pre-check on whether the composite key already exists. The SurrealDB unique index `idx_idempotency_pk ... UNIQUE` on `(school_id, command_type, idempotency_key)` will cause a server error, but the error is mapped to `Infrastructure` (Finding 9), not `Conflict`.

---

### FINDING 21 (id: `ADAPTER-SR-021`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/idempotency.rs:80-83` (`SurrealIdempotency` struct)

**Description:**

The `SurrealIdempotency` struct carries only a `db: Db` field, no `school: SchoolId`. The `outbox`, `audit`, and `event` sub-port structs in the same crate (`outbox.rs:160-162`, `audit.rs:137-139`, `event_log.rs:95-97`) all carry a `school: SchoolId`. The omission is asymmetric and means the idempotency handle cannot enforce a per-school write boundary at the application layer — a `record()` call from a `SurrealIdempotency` handle bound to adapter A can write an idempotency row for school B. Compare the `Outbox` handle (line 161) which carries `school: SchoolId` and uses it in the `pending()` filter (line 229) to scope the read.

**Expected:**

The `Idempotency` sub-port is `tenant-scoped`: every record is keyed by `(school_id, command_type, idempotency_key)` and every read must be `school_id`-filtered. The handle should carry `school: SchoolId` to make this explicit and to allow an audit/assertion in `record()` that the record's `school_id` matches the handle's `school`.

**Evidence:**

`crates/adapters/storage-surrealdb/src/idempotency.rs:80-83`:
  ```rust
  pub struct SurrealIdempotency {
      pub(crate) db: Db,
  }
  ```
  Compare `crates/adapters/storage-surrealdb/src/outbox.rs:159-162`:
  ```rust
  pub struct SurrealOutbox {
      pub(crate) db: Db,
      pub(crate) school: SchoolId,
  }
  ```

---

### FINDING 22 (id: `ADAPTER-SR-022`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/connection.rs:50-62` (`SurrealConnection::in_memory`)

**Description:**

`SurrealConnection::in_memory` is the only connection constructor on `SurrealConnection`. The `Cargo.toml` declares `surrealdb = { workspace = true }` and the connection module comment says "Phase 0 supports the in-memory backend (`Mem`); the RocksDB / TiKV / HTTP backends land in a later phase." The dialect spec (`docs/schemas/sql-dialects/surrealdb.md:51-72, 825-845`) is explicit that the adapter must target `rocksdb` (production single-process) as well as `memory` (tests) and that the embed pattern is `Surreal::new::<RocksDb>("./data/educore.db")`. There is no `RocksDb` constructor, no `encryption_key` plumbing, and no way for a consumer to use SurrealDB as a durable single-process database. ADR-017's rationale ("Single-binary deployment. SurrealDB embedded means one process to ship") is not realised by the shipped code.

**Expected:**

`docs/schemas/sql-dialects/surrealdb.md:827-838`:
  ```rust
  let db = Surreal::new::<RocksDb>("./data/educore.db").await?;
  ```
  And: "The engine's adapter exposes an `encryption_key` parameter on the connection builder." (`docs/schemas/sql-dialects/surrealdb.md:856-866`).

**Evidence:**

`crates/adapters/storage-surrealdb/src/connection.rs:50-62` exposes only `in_memory(school: SchoolId)`. `crates/adapters/storage-surrealdb/Cargo.toml:20` declares `surrealdb = { workspace = true }` but the only `use` in `connection.rs:8-9` is `engine::local::{Db as LocalDb, Mem}`. No `RocksDb` import, no `File` import, no `WsClient` / `HttpClient` for server mode.

---

### FINDING 23 (id: `ADAPTER-SR-023`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/connection.rs:50-62` (`SurrealConnection::in_memory`)

**Description:**

`in_memory` opens a *new* in-memory SurrealDB on every call. Two calls to `SurrealConnection::in_memory(school_a)` and `SurrealConnection::in_memory(school_b)` produce two independent `Db` instances (each scoped to one school). The unit test at `audit.rs:393-421` (`read_for_target_isolates_by_school`) depends on this fact — and the audit.rs module doc explicitly states (lines 250-257): "the storage layer does not itself enforce it (the engine's `TenantContext` layer is the canonical gate per `docs/schemas/tenancy-schema.md`)." In other words, the in-memory backend is per-school-isolated by virtue of being per-process; the production storage adapter will share a single `Db` across all schools (per the architecture doc, single-binary / per-school single-tenant), and the in-memory test's "isolation by separate process" does not generalise to a shared `Db`.

**Expected:**

Per `docs/ports/storage.md:140-150`: "The storage adapter is responsible for enforcing tenant isolation. The engine always passes a `SchoolId` filter; the adapter MUST add a `school_id = $1` predicate to every read query." Defense in depth: the in-memory test's "two `Db` instances" trick is not the production architecture.

**Evidence:**

`crates/adapters/storage-surrealdb/src/connection.rs:50-62` constructs a fresh `Surreal::new::<Mem>(())` per call. The audit module's doc-comment at `crates/adapters/storage-surrealdb/src/audit.rs:250-257` admits the test's "two `Db` instances" trick is the only reason the isolation test passes.

---

### FINDING 24 (id: `ADAPTER-SR-024`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/connection.rs:23-31` (`SurrealConnection` struct) + `crates/adapters/storage-surrealdb/src/storage.rs:1-180` (no `$auth` setup)

**Description:**

The dialect spec mandates per-table `PERMISSIONS` clauses with a session-scoped `$auth.school_id` predicate (see `docs/schemas/sql-dialects/surrealdb.md:231-262`) and a "second line of defense" via `PERMISSIONS NONE` on the engine-internal tables. The migration DDL (`migrations/engine/0000_engine_core.surreal.surql:71-260`) does NOT emit any `PERMISSIONS` clause — every table is `DEFINE TABLE <name> SCHEMAFULL COMMENT "..."` with no permission scope. The connection does not set `$auth.school_id` on connect (`connection.rs:50-62` has only `use_ns("educore").use_db("engine")`). The result: a consumer's session can read every school's `outbox` rows because the DB enforces no per-school permission.

**Expected:**

`docs/schemas/sql-dialects/surrealdb.md:238-244`:
  ```sql
  DEFINE TABLE academic_students SCHEMAFUL
    PERMISSIONS
      FOR SELECT WHERE school_id = $auth.school_id OR $auth.bypass = true
      ...
  ```
  And: "`PERMISSIONS NONE` on `outbox` is correct — the engine writes to it from the application layer, never from user sessions" (`docs/schemas/sql-dialects/surrealdb.md:471-474`).

**Evidence:**

`migrations/engine/0000_engine_core.surreal.surql:71` `DEFINE TABLE outbox SCHEMAFULL\n    COMMENT "...";` (no `PERMISSIONS NONE`); `:127` (audit_log), `:164` (idempotency), `:193` (event_log), `:226` (schema_registry), `:252` (system_user) all lack `PERMISSIONS`. `crates/adapters/storage-surrealdb/src/connection.rs:50-62` has no `LET $auth = { school_id: ... }` setup.

---

### FINDING 25 (id: `ADAPTER-SR-025`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/event_log.rs:194-201` (`EventLog::read` query)

**Description:**

The `read` query uses dynamic string formatting to compose the `WHERE` clause and binds only the `school` and `limit` parameters. The `event_types` filter, `since` / `until` timestamps, and `aggregate_id` are spliced into the SQL string via `format!` (per Finding 19). This bypasses the SurrealDB driver's parameterised bind path for four different user-controllable filter values, defeating the driver's type checking and the database's `DEFINE FIELD ... ASSERT` invariants. The driver has no way to validate the spliced values before they reach the parser.

**Expected:**

The dialect spec's `DEFINE FIELD event_type ... ASSERT $value != NONE AND string::len($value) <= 191` (line 75 of the .surql) is meant to be enforced at the storage layer on every bind. Bypassing the bind path with string formatting defeats the assertion.

**Evidence:**

`crates/adapters/storage-surrealdb/src/event_log.rs:193-201` constructs the entire query with `format!` and only binds `school` (line 205) and `limit` (line 206).

---

### FINDING 26 (id: `ADAPTER-SR-026`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/storage.rs:176-179` (`const _` blocks)

**Description:**

The `storage.rs` file ends with three `const _: ... = ...` blocks (`Arc<()>`, `Duration`, `fn() = || { ... }`) whose stated purpose is "Suppress unused-import warning for `Arc` and `Duration` in this Phase 0 stub; they're reserved for the full impl." This is a code smell: the imports (`Arc`, `Duration`, `futures::StreamExt`) are dead, and the suppression blocks violate `AGENTS.md` § "Type Safety": "No `#[allow(dead_code)]` or `_var` prefixes to silence the compiler. Delete unused code, wire it in, or open a follow-up issue."

**Expected:**

Per `AGENTS.md` § "Type Safety", delete the unused imports and the suppression blocks. The `Arc` and `Duration` are not used in this file; remove the imports.

**Evidence:**

`crates/adapters/storage-surrealdb/src/storage.rs:174-179`:
  ```rust
  // Suppress unused-import warning for `Arc` and `Duration`
  // in this Phase 0 stub; they're reserved for the full impl.
  const _: Option<Arc<()>> = None;
  const _: Option<Duration> = None;
  const _: fn() = || {
      std::mem::drop(futures::stream::empty::<()>().next());
  };
  ```

---

### FINDING 27 (id: `ADAPTER-SR-027`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/outbox.rs:112-144` (`parse_*` helpers, all `#[allow(dead_code)]`)

**Description:**

The `outbox.rs` file declares five helper functions (`parse_event_id`, `parse_school_id_opt`, `parse_user_id`, `parse_correlation_id`, `parse_uuid`) all marked `#[allow(dead_code)]`. `grep -rn 'parse_event_id\|parse_school_id_opt\|parse_user_id\|parse_correlation_id' crates/adapters/storage-surrealdb/src/` shows the functions are only defined, never called. The struct field mapping (lines 41-69 / 74-101) uses `SurrealUuid::from(env.event_id.as_uuid())` directly, not the string-based parsers. This is dead code, violating `AGENTS.md` § "Type Safety".

**Expected:**

Per `AGENTS.md` § "Type Safety": "Delete unused code, wire it in, or open a follow-up issue." Remove the five `parse_*` functions and the `parse_uuid` helper; the `SurrealUuid::from(uuid::Uuid)` direct path is the only path used.

**Evidence:**

`crates/adapters/storage-surrealdb/src/outbox.rs:112-144`:
  ```rust
  #[allow(dead_code)]
  fn parse_event_id(s: &str) -> std::result::Result<EventId, StringError> { ... }
  #[allow(dead_code)]
  fn parse_school_id_opt(s: Option<&str>) -> ... { ... }
  // ... three more
  ```
  The functions are never called in the file or in any other file under `crates/adapters/storage-surrealdb/`.

---

### FINDING 28 (id: `ADAPTER-SR-028`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/stubs.rs:1-8` (entire file)

**Description:**

`stubs.rs` is a placeholder file with only the doc comment and `#![allow(dead_code)]`. The doc says "All sub-port impls (AuditLog, EventLog, Idempotency) are now real implementations in their dedicated modules. This file remains as a marker for the wire-up completed by A'.1; the module is intentionally empty." An empty module exported from `lib.rs` (`crates/adapters/storage-surrealdb/src/lib.rs:22` `pub mod stubs;`) is dead code that the workspace lints should have caught. The module is referenced only by `audit.rs:131-134`'s comment ("`crate::stubs::SurrealAuditLog`") which is now misleading because no such type exists in the module.

**Expected:**

Per `AGENTS.md` § "Type Safety": "Delete unused code, wire it in, or open a follow-up issue." Remove `pub mod stubs;` from `lib.rs` and delete the file.

**Evidence:**

`crates/adapters/storage-surrealdb/src/stubs.rs:1-8`:
  ```rust
  //! SurrealDB-backed sub-port placeholders.
  //!
  //! All sub-port impls (AuditLog, EventLog, Idempotency) are now
  //! real implementations in their dedicated modules. This file
  //! remains as a marker for the wire-up completed by A'.1; the
  //! module is intentionally empty.
  #![allow(dead_code)]
  ```

---

### FINDING 29 (id: `ADAPTER-SR-029`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/outbox.rs:41-46` (`OutboxRow::from_envelope`)

**Description:**

`OutboxRow::from_envelope` calls `serde_json::from_slice(&env.payload).unwrap_or_else(|_| serde_json::Value::String(String::from_utf8_lossy(&env.payload).into_owned()))` to convert the payload bytes to a `serde_json::Value` for the `payload` column. The round-trip is therefore lossy: a payload that fails to parse as JSON is stored as a JSON-stringified UTF-8 lossy version of the original bytes. The downstream `payload_to_bytes` (lines 149-155) re-serialises the JSON-Value to a `String` and wraps it in `Bytes`, so a non-JSON payload round-trips as `"<original utf-8 bytes>"` (a JSON string literal). The semantic of `Outbox::pending` therefore silently changes: a caller reading back an outbox row whose payload was a binary blob or invalid UTF-8 receives a string-wrapped version that loses the original byte boundaries.

**Expected:**

The DDL declares `payload` as `TYPE object` (`migrations/engine/0000_engine_core.surreal.surql:91`) and the port contract allows any serialised payload. The round-trip should be byte-exact or the lossy behaviour should be documented and gated.

**Evidence:**

`crates/adapters/storage-surrealdb/src/outbox.rs:42-45`:
  ```rust
  let payload_value: serde_json::Value =
      serde_json::from_slice(&env.payload).unwrap_or_else(|_| {
          serde_json::Value::String(String::from_utf8_lossy(&env.payload).into_owned())
      });
  ```

---

### FINDING 30 (id: `ADAPTER-SR-030`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/idempotency.rs:126-152` (`Idempotency::record`)

**Description:**

`Idempotency::record` discards the SurrealDB query result: `let _ = self.db.query("INSERT INTO idempotency { ... }")...await?;`. The query response is checked only for the error case; the success case is treated as a unit. The SurrealDB driver can return per-statement errors via the response's `take(N)` API; the `let _` discards that path. A statement that fails server-side (e.g. a `DEFINE FIELD` `ASSERT` violation, an `affected_aggregate_ids` type mismatch) is silently treated as success.

**Expected:**

Pull the typed result at position 0 to surface server-side errors, matching the pattern used in `outbox.rs:222-225` (`Outbox::append`) which does `let _: Vec<OutboxRow> = response.take(0).map_err(...)?;`. The same pattern is missing here.

**Evidence:**

`crates/adapters/storage-surrealdb/src/idempotency.rs:128-149`:
  ```rust
  let _ = self
      .db
      .query("INSERT INTO idempotency { ... }")...await
      .map_err(|e| StringError(format!("idempotency record: {e}")))?;
  Ok(())
  ```

---

### FINDING 31 (id: `ADAPTER-SR-031`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/event_log.rs:117-154` (`EventLog::append`)

**Description:**

`EventLog::append` discards the SurrealDB query result: `let _ = self.db.query("INSERT INTO event_log { ... }")...await?;`. A duplicate-`event_id` insert is silently treated as success; the unique index `idx_event_log_event_id ... UNIQUE` on `event_log.event_id` (`migrations/engine/0000_engine_core.surreal.surql:210`) will produce a server-side error, but it is mapped to `Infrastructure` (Finding 9's pattern, generalised here) and the success path discards any per-statement server warning.

**Expected:**

`let _: Vec<EventRow> = response.take(0).map_err(...)?;` pattern from `outbox.rs:222-225`. And the duplicate must be mapped to `Conflict` per Finding 9.

**Evidence:**

`crates/adapters/storage-surrealdb/src/event_log.rs:119-152`:
  ```rust
  let _ = self
      .db
      .query("INSERT INTO event_log { ... }")...await
      .map_err(|e| StringError(format!("event_log append: {e}")))?;
  Ok(())
  ```

---

### FINDING 32 (id: `ADAPTER-SR-032`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** High
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/tests/outbox_e2e.rs:1-58` (entire file)

**Description:**

The test suite contains exactly one end-to-end test (`outbox_append_and_pending_round_trip`) covering only the outbox sub-port. No tests exist for: `audit_log.append` / `audit_log.read_for_target`, `event_log.append` / `event_log.read` / `event_log.count`, `idempotency.lookup` / `idempotency.record` / `idempotency.purge_older_than`, `migrate()` idempotency, `cursor_for` / `advance_cursor` return-value verification, `ping()`, `close()` lifecycle, tenant-isolation enforcement across a single `Db` instance, SQL-injection attempts on `event_log.read`, double-commit / double-rollback, or any round-trip across the `SurrealTransaction` boundary. The single test path uses the in-memory constructor only.

**Expected:**

Per `docs/ports/storage.md:468-477`: "The port requires: Unit tests of every repository method. Integration tests against a real database (testcontainers). A parity test verifying identical behavior across adapters. A tenancy test verifying cross-tenant reads are blocked. A failure-injection test (e.g. deadlock retry, connection drop). A load test (10k attendance marks in <5s)."

**Evidence:**

`ls crates/adapters/storage-surrealdb/tests/` returns only `outbox_e2e.rs`. The file is 58 lines and exercises one round-trip. The handoff at `docs/handoff/PHASE-0-HANDOFF.md:14-19` records "120 tests pass workspace-wide" but does not name a SurrealDB parity / tenancy / failure-injection / load test.

---

### FINDING 33 (id: `ADAPTER-SR-033`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/connection.rs:50-62` (`SurrealConnection::in_memory`)

**Description:**

The `tracing` crate is a declared dependency (`Cargo.toml:21`) but the connection code (`connection.rs:50-62`) emits no `tracing::info!` / `tracing::debug!` on `connect`, no `tracing::warn!` on slow `use_ns` / `use_db`, and no `tracing::error!` on failure. The same is true for every other file in the crate (`grep -rn 'tracing::' crates/adapters/storage-surrealdb/src/` returns zero matches). The dialect spec (`docs/schemas/sql-dialects/surrealdb.md:847-849`) is explicit: "The adapter's DDL emission is unit-tested against an in-memory SurrealDB instance. The DDL is verified before any test queries run." The other adapters (`storage-postgres`, `storage-sqlite`) use `#[instrument(skip(self))]` on every port method and `tracing::debug!` on the connection open. The SurrealDB adapter's silence is asymmetric.

**Expected:**

Per `AGENTS.md` § "Engine Rules" + the workspace convention: every port method is `#[instrument]`-decorated and emits `tracing` events on the connection lifecycle. The PG adapter at `crates/adapters/storage-postgres/src/storage.rs:41, 117, 129, 171, 185, 196, 218, 225, 237, 248` is the reference.

**Evidence:**

`grep -rn 'tracing::' crates/adapters/storage-surrealdb/src/` returns zero matches. `crates/adapters/storage-surrealdb/Cargo.toml:21` `tracing = { workspace = true }` is declared but unused. `crates/adapters/storage-surrealdb/src/error.rs:8-9` documents the use of `StringError` to wrap "format!-style error messages without depending on anyhow" — no tracing integration.

---

### FINDING 34 (id: `ADAPTER-SR-034`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/outbox.rs:41-46, 70-100` (`OutboxRow` payload handling) + `crates/adapters/storage-surrealdb/src/audit.rs:62-80, 100-114` (`AuditRow` payload handling) + `crates/adapters/storage-surrealdb/src/event_log.rs:50-70` (`EventRow` payload handling) + `crates/adapters/storage-surrealdb/src/idempotency.rs:35-76` (`IdempotencyRow` payload handling)

**Description:**

Every sub-port that handles a payload (`outbox.payload`, `audit.before`, `audit.after`, `event_log.payload`, `idempotency.outcome`) writes and reads the payload as a `serde_json::Value` or as a `SurrealBytes` (raw bytes). The DDL declares `outbox.payload` as `TYPE object` (surreal.surql:91) — SurrealDB's object type, which is structurally a JSON object. The audit row's `before` / `after` are `TYPE option<bytes>` (surreal.surql:135-136) — raw bytes. The two semantics are different: `outbox.payload` is JSON-shaped (the DDL says `object`), `audit.before` is raw bytes. But the outbox `to_envelope` reads back a `serde_json::Value` and converts it to a `String` via `payload_to_bytes` (outbox.rs:149-155) which calls `other.to_string()` on non-string JSON values — meaning a JSON number, boolean, array, or null in the original payload round-trips as a stringified version, not as a binary blob. The semantic mismatch means a payload that was an object `{"x": 1}` round-trips as a string `{"x": 1}` (the JSON-serialised text) — not as a byte-exact JSON. The downstream `from_envelope` does the inverse: it tries to `serde_json::from_slice(&env.payload)` to recover the value, which succeeds for the stringified version (the string parses as JSON), so the round-trip is *visible* to be lossy only when the original was a primitive (string-of-string-of-string) or non-UTF-8.

**Expected:**

The port contract says `payload: bytes::Bytes` is "the JSON (or MessagePack) representation" of the typed event (`crates/infra/storage/src/outbox.rs:75-84`). The round-trip must preserve the exact bytes. The SurrealDB `bytes` type is the natural fit; the `object` type is lossy.

**Evidence:**

`crates/adapters/storage-surrealdb/src/outbox.rs:42-45` (writer) and `:149-155` (reader). Compare `crates/adapters/storage-surrealdb/src/audit.rs:62-67` and `:93-94` which correctly use `SurrealBytes::from(b.to_vec())` and `Bytes::from(b.to_vec())` for raw-byte round-trip.

---

### FINDING 35 (id: `ADAPTER-SR-035`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/event_log.rs:218-273` (`EventLog::count`)

**Description:**

`EventLog::count` always returns `0` (via the `rows.first().map(|r| ...).unwrap_or(0)` fallback) when the result set is empty, but it also clamps any non-zero `n` to `0` via `u64::try_from(r.n).unwrap_or(0)`. A negative `n` (impossible per the SurrealDB `count()` aggregate, but the code is defensive) is clamped to `0` — which is the same value as "no rows matched", so a real negative-n would be silently reported as zero count. More importantly, the deserialised `CountRow { n: i64 }` does not validate that `n >= 0` before the `u64::try_from` cast.

**Expected:**

Per `crates/infra/storage/src/event_log.rs:156-158`: "Returns the count of events for `school_id` matching `filter`." The count must be exact.

**Evidence:**

`crates/adapters/storage-surrealdb/src/event_log.rs:263-272`:
  ```rust
  #[derive(serde::Deserialize)]
  struct CountRow {
      n: i64,
  }
  let rows: Vec<CountRow> = response.take(0).map_err(|e| StringError(format!("event_log count take: {e}")))?;
  Ok(rows
      .first()
      .map(|r| u64::try_from(r.n).unwrap_or(0))
      .unwrap_or(0))
  ```

---

### FINDING 36 (id: `ADAPTER-SR-036`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `migrations/engine/0000_engine_core.surreal.surql:71-260` (six tables, no `PERMISSIONS`)

**Description:**

None of the six engine cross-cutting tables (`outbox`, `audit_log`, `idempotency`, `event_log`, `schema_registry`, `system_user`) carry a `PERMISSIONS` clause. The dialect spec is explicit: "`PERMISSIONS NONE` on `outbox` is correct — the engine writes to it from the application layer, never from user sessions" (`docs/schemas/sql-dialects/surrealdb.md:471-474`). The DDL file is also missing the `PERMISSIONS NONE` for the other engine-internal tables. The migration comments on lines 110-118 acknowledge the engine's PERMISSIONS scope is the canonical RLS path: "Append-only is enforced at the engine layer (the SurrealDB adapter does not expose update/delete for this table to domain code) and at the SurrealDB layer via a REVOKE-style permission scope on the `audit_log` table for non-system roles." But the DDL does not emit that permission scope.

**Expected:**

`DEFINE TABLE outbox SCHEMAFULL PERMISSIONS NONE;` and similarly for `audit_log`, `idempotency`, `event_log`, `schema_registry`, `system_user`.

**Evidence:**

`migrations/engine/0000_engine_core.surreal.surql:71-72, 127-128, 164-165, 193-194, 226-227, 252-253` — every `DEFINE TABLE` is followed only by a `COMMENT` and a newline, no `PERMISSIONS` clause on any of the six.

---

### FINDING 37 (id: `ADAPTER-SR-037`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `migrations/engine/0000_engine_core.surreal.surql:91` (`payload` column on `outbox`) + `migrations/engine/0000_engine_core.surreal.surql:207` (`payload` column on `event_log`)

**Description:**

The `outbox.payload` column is `TYPE object` (line 91) but the writer side at `outbox.rs:42-45` constructs a `serde_json::Value::String(...)` when the payload is non-JSON; SurrealDB's `object` type rejects `string` values. A binary payload (e.g. a CBOR-encoded event body, an attachment, a binary image) would fail the `ASSERT $value != NONE` and the `TYPE object` constraint — but the error is mapped to `Infrastructure` (Finding 8). A consumer using MessagePack or CBOR would see their outbox writes fail at runtime with no clear port-level error. The same risk applies to `event_log.payload`, which is `TYPE bytes` (line 207) — the round-trip is via `SurrealBytes::from(entry.payload.to_vec())` (event_log.rs:53) and `Bytes::from(self.payload.to_vec())` (event_log.rs:69), which is correct, but `surreal_bytes` and `bytes` are the same type only if `to_vec()` is consistent — the `surreal::sql::Bytes` type's serde repr may differ from `bytes::Bytes` (see the discrepancy in `outbox.rs:34` `payload: serde_json::Value` vs `event_log.rs:35` `payload: SurrealBytes`).

**Expected:**

Pick one wire type for `payload` (either `object` or `bytes`) and make the Rust adapter use the corresponding converter consistently. Document the choice in `docs/schemas/sql-dialects/surrealdb.md`.

**Evidence:**

`migrations/engine/0000_engine_core.surreal.surql:91` `DEFINE FIELD payload ON TABLE outbox TYPE object` and `:207` `DEFINE FIELD payload ON TABLE event_log TYPE bytes`. `crates/adapters/storage-surrealdb/src/outbox.rs:34` `pub payload: serde_json::Value` vs `crates/adapters/storage-surrealdb/src/event_log.rs:35` `pub payload: SurrealBytes`.

---

### FINDING 38 (id: `ADAPTER-SR-038`)

- **Source:** `docs/audit_reports/findings/wave3-storage-surrealdb.md`
- **Severity:** Medium
- **Area:** adapters
- **Location:** `crates/adapters/storage-surrealdb/src/storage.rs:1-180` + `crates/adapters/storage-surrealdb/src/transaction.rs:1-114` (no `bulk_insert_student_attendances`)

**Description:**

The `StorageAdapter` trait at `crates/infra/storage/src/port.rs:83-92` defines `bulk_insert_student_attendances(ctx, rows)` as a required port method (with a default `NotSupported` implementation). The same method exists on `Transaction` at `crates/infra/storage/src/transaction.rs:86-91`. The SurrealDB adapter does not override either. The Phase 5 bulk-marking service's critical path (per `docs/ports/storage.md:469-477` and the Phase 5 exit criterion: "200 rows in under 100 ms on PostgreSQL") has no SurrealDB implementation. The trait default returns `NotSupported`, which is the correct answer for the Phase 0 stub per the trait's own doc, but the `SurrealTransaction` struct has no `bulk` field and no plumbing for the bulk-insert path.

**Expected:**

The PG / SQLite adapters implement `bulk_insert_student_attendances` (per the parity audit at `docs/audit_reports/findings/wave3-storage-sqlite.md:194-197`). SurrealDB does not — its consumers will see `DomainError::NotSupported("StorageAdapter::bulk_insert_student_attendances is not supported by this adapter")` on the bulk-marking service.

**Evidence:**

`grep -n 'bulk_insert_student_attendances' crates/adapters/storage-surrealdb/src/` returns no matches. `crates/adapters/storage-surrealdb/src/transaction.rs:32-50` has fields `outbox, audit, event, idem, done, rolled_back, _db` but no `bulk` field. `crates/adapters/storage-surrealdb/src/storage.rs:75-172` has no override of `bulk_insert_student_attendances`.

---


## Event Bus (target id prefix: `ADAPT-EB`)

**Path:** `crates/adapters/event-bus/`  
**Total findings:** 22 (5 critical, 8 high, 9 medium, 0 low)


### FINDING 1 (id: `ADAPT-EB-001`)

- **Source:** `docs/audit_reports/findings/wave3-event-bus.md`
- **Severity:** Critical
- **Area:** adapters-event-bus
- **Location:** `crates/adapters/event-bus/src/nats.rs:90-114` and
  `crates/adapters/event-bus/src/redis.rs:128-152`

**Description:**

`NatsEventBus` and `RedisEventBus` are feature-
  gated stubs whose `EventBus` impl only covers `publish`,
  `publish_batch`, and `subscribe`. Both types are exposed at the
  crate root (`crates/adapters/event-bus/src/lib.rs:60-66`) and
  advertise "Phase 2 scaffold for a NATS JetStream-backed event
  bus" in their rustdoc, yet every method returns
  `EventError::not_supported("NatsEventBus::publish")` (and
  analogous Redis strings). The Phase 2 hand-off
  (`docs/handoff/PHASE-2-HANDOFF.md` § "educore-event-bus")
  acknowledges the stubs, but the README, the lib rustdoc, and
  the feature flags (`default = ["in-process"]`,
  `nats = ["dep:async-nats"]`, `redis = ["dep:redis"]`) present
  them as usable adapters. A consumer that wires
  `Arc<dyn EventBus> = Arc::new(NatsEventBus::new().connect(...))`
  gets a bus that accepts `connect` and silently fails every
  publish / subscribe — production deploys that picked the
  distributed adapter on the assumption that it works will lose
  every event without a runtime error.

**Expected:**

`docs/ports/event-bus.md:171-176` says
  "Distributed adapters are consumer-supplied. The bus trait is
  intentionally minimal so any messaging system can be
  implemented." The stubs are not "consumer-supplied"; they
  ship in the engine's adapter crate and appear to be a
  production-ready adapter. The Phase 2 hand-off should be
  linked from the README and the feature flag should be
  documented as "Phase 2 stub — wire-protocol work in a later
  phase".

**Evidence:**

```rust
  async fn publish(
      &self,
      _envelope: EventEnvelope,
  ) -> educore_core::error::Result<PublishReceipt> {
      debug!("NatsEventBus::publish (Phase 2 stub, returning NotSupported)");
      Err(EventError::not_supported("NatsEventBus::publish").into())
  }
  ```
  (analogous body in `redis.rs:131-137`).

---

### FINDING 13 (id: `ADAPT-EB-013`)

- **Source:** `docs/audit_reports/findings/wave3-event-bus.md`
- **Severity:** Critical
- **Area:** adapters-event-bus
- **Location:** `crates/adapters/event-bus/src/in_process.rs:457-475`
  and `docs/ports/event-bus.md:71-77`

**Description:**

The bus-port contract
  (`docs/ports/event-bus.md:71-77`) promises
  "at-least-once delivery" and "Consumers MUST be idempotent.
  The EventId is the idempotency key." The in-process adapter
  never tracks which `event_id`s it has delivered to which
  subscription, and the `start_position_matches` helper at
  `in_process.rs:413-417` compares UUIDs lexicographically
  assuming UUIDv7 ordering. A producer that republishes the
  same envelope after a retry (the typical at-least-once
  scenario) will deliver the duplicate to every active
  subscription with no dedupe; a consumer that resumes from
  `StartPosition::FromEventId(cursor)` after a process crash
  relies entirely on the consumer's own processed-events table
  to dedupe — the bus does not surface the
  `last_delivered_event_id` per `(subscription, consumer_id)`
  pair, so a fresh subscription created mid-replay can be
  handed the same envelope twice if the cursor's UUID is not
  monotonic relative to the new subscription's start.

**Expected:**

`docs/ports/event-bus.md:71-86` (at-least-once
  delivery + idempotency + DLQ + replay contract).

**Evidence:**

`grep -n "last_delivered\|seen_event_ids\|
  dedupe\|dedup\|idempot" crates/adapters/event-bus/src/in_process.rs`
  returns zero rows; the `InProcessInner` struct
  (`in_process.rs:117-127`) has no `last_delivered` map.

---

### FINDING 14 (id: `ADAPT-EB-014`)

- **Source:** `docs/audit_reports/findings/wave3-event-bus.md`
- **Severity:** Critical
- **Area:** adapters-event-bus
- **Location:** `crates/adapters/event-bus/src/in_process.rs:128-187`
  and `docs/ports/event-bus.md:60-67`

**Description:**

Per `docs/ports/event-bus.md:60-67` "The
  engine writes events to an outbox table within the same
  database transaction as the domain state change. The outbox
  relay (a separate process) reads pending events from the
  outbox and publishes them to the bus." The in-process
  adapter does not read from any outbox table; it accepts
  `publish` calls directly from in-process producers and
  forwards them to the broadcast channel. There is no
  outbox-relay loop, no background task, no drain call, and
  no `subscribe_outbox` method. The cross-cutting integration
  test at `crates/tools/storage-parity/tests/cross_cutting_integration.rs`
  (per `docs/handoff/PHASE-2-HANDOFF.md` § "Cross-cutting
  integration test") exercises the
  `outbox → event_log → bus` path on SQLite, but the relay
  lives in the test, not in the adapter crate. A consumer
  that wires `InProcessEventBus::new()` and skips the
  cross-cutting test harness gets a bus that has no source of
  truth — events emitted by a command but not directly
  `publish`'d to the bus (e.g., committed via storage
  outbox) will never appear on the bus.

**Expected:**

`docs/ports/event-bus.md:60-67` (outbox pattern).

**Evidence:**

`grep -n "outbox\|Outbox\|drain\|relay"
  crates/adapters/event-bus/src/in_process.rs` returns zero
  rows; the `publish` signature
  (`in_process.rs:172-200`) takes a pre-built `EventEnvelope`,
  not an outbox-row id.

---

### FINDING 2 (id: `ADAPT-EB-002`)

- **Source:** `docs/audit_reports/findings/wave3-event-bus.md`
- **Severity:** Critical
- **Area:** adapters-event-bus
- **Location:** `crates/adapters/event-bus/src/in_process.rs:457-475`

**Description:**

`InProcessSubscription::ack` and `nack` are
  no-ops that always return `Ok(AckOutcome::Accepted)`, with the
  rationale "in-process delivery is direct; ack is a no-op".
  The bus-port contract at `docs/ports/event-bus.md:71-77`
  promises "At-Least-Once Delivery. The bus provides at-least-
  once delivery" and `docs/ports/event-bus.md:84-86` promises
  "Dead Letter Queue. Events that fail repeatedly (configurable
  N retries) are routed to a dead letter queue." A consumer
  that calls `nack(event_id, true)` to requeue a failed event
  receives `AckOutcome::Accepted`; the envelope stays on the
  bus broadcast channel, is delivered again to the same
  subscription, and any other subscriber that filtered
  on `EventFilter` will never see the rejected instance. There
  is no retry counter, no DLQ, no per-event delivery state, and
  no way for a consumer to detect that its `nack` was
  discarded.

**Expected:**

`docs/ports/event-bus.md:71-77` (at-least-once
  delivery) and `:84-86` (DLQ).

**Evidence:**

```rust
  async fn ack(&mut self, _event_id: EventId) -> educore_core::error::Result<AckOutcome> {
      // In-process delivery is direct; ack is a no-op.
      Ok(AckOutcome::Accepted)
  }
  async fn nack(
      &mut self,
      _event_id: EventId,
      _requeue: bool,
  ) -> educore_core::error::Result<AckOutcome> {
      // In-process delivery is direct; nack is a no-op.
      Ok(AckOutcome::Accepted)
  }
  ```

---

### FINDING 3 (id: `ADAPT-EB-003`)

- **Source:** `docs/audit_reports/findings/wave3-event-bus.md`
- **Severity:** Critical
- **Area:** adapters-event-bus
- **Location:** `crates/adapters/event-bus/src/in_process.rs:280-299`

**Description:**

`InProcessEventBus::publish` first pushes the
  envelope to a bounded `VecDeque` replay log (capped at
  `replay_log_capacity`, default 4096), then sends it to the
  global broadcast channel. When the broadcast channel has
  zero receivers, the `send` error is **silently swallowed**
  (`Err(_)` branch returns `Ok(PublishReceipt::new(...))` with
  no indication the envelope was not delivered to anyone). The
  envelope remains in the replay log, so a future
  `StartPosition::Earliest` subscriber sees it. But when the
  log is also at capacity and an envelope is pushed in, the
  oldest envelope is `pop_front()`'d before the new one is
  appended — a burst of more than 4096 publishes between two
  subscribers joining will silently evict the head of the log
  with no warning. There is no metric, no event, no log line
  on eviction. Per `docs/ports/event-bus.md:71-77` the bus
  promises at-least-once delivery; silent eviction of an
  envelope is at-most-once-from-the-log and contradicts the
  contract.

**Expected:**

`docs/ports/event-bus.md:71-77` (at-least-once
  delivery); `docs/ports/event-bus.md:106-110` (replay retention).

**Evidence:**

```rust
  // Fan out to every active receiver. `send` only fails if
  // there are zero receivers; that's a normal idle state
  // for the in-process bus and not an error.
  match self.inner.sender.send(env) {
      Ok(_) => Ok(PublishReceipt::new(event_id, topic, Timestamp::now())),
      Err(_) => {
          // No receivers; the envelope is still in the
          // replay log (if any), so a future `Earliest`
          // subscription will see it.
          Ok(PublishReceipt::new(event_id, topic, Timestamp::now()))
      }
  }
  ```
  And at `crates/adapters/event-bus/src/in_process.rs:262-273`:
  ```rust
  if self.inner.config.replay_log_capacity > 0 {
      match self.inner.log.lock() {
          Ok(mut log) => {
              if log.len() == self.inner.config.replay_log_capacity {
                  log.pop_front();
              }
              log.push_back(env.clone());
          }
          ...
      }
  }
  ```

---

### FINDING 15 (id: `ADAPT-EB-015`)

- **Source:** `docs/audit_reports/findings/wave3-event-bus.md`
- **Severity:** High
- **Area:** adapters-event-bus
- **Location:** `crates/adapters/event-bus/src/in_process.rs:222-245`

**Description:**

`InProcessSubscription` has no
  `batch_size` plumbing: the field is on `SubscribeOptions`
  (`crates/cross-cutting/events/src/event_bus.rs:159-178`) and
  defaults to `32`, but `next()` returns one envelope per
  call. A consumer that sets `batch_size = 256` to amortise
  the cost of `next()` over a larger window still pays one
  `broadcast::Receiver::recv` per envelope. The
  `visibility_timeout` field is similarly ignored. The port
  doc at `docs/ports/event-bus.md:50` documents both fields
  as contract; the in-process adapter accepts them but does
  not honour them. `for_consumer` defaults to `32 / 300s`
  (`event_bus.rs:181-186`); there is no clamp.

**Expected:**

`docs/ports/event-bus.md:46-50` (SubscribeOptions
  shape including `batch_size` and `visibility_timeout`).

**Evidence:**

```rust
  let mut sub = bus
      .subscribe(make_opts(
          "test-consumer",
          Topic::All,
          StartPosition::Latest,
      ))
      .await
      .expect("subscribe");
  ```
  with `make_opts` building a 32-entry `batch_size` and 300s
  `visibility_timeout` that are never read by
  `InProcessEventBus` (`tests/in_process_e2e.rs:60-72`).

---

### FINDING 16 (id: `ADAPT-EB-016`)

- **Source:** `docs/audit_reports/findings/wave3-event-bus.md`
- **Severity:** High
- **Area:** adapters-event-bus
- **Location:** `crates/adapters/event-bus/src/in_process.rs:431-445`

**Description:**

When a `broadcast::error::RecvError::Lagged(skipped)`
  is observed, the subscription silently continues. The
  `skipped` count is logged at `debug!` level and discarded.
  There is no public API to retrieve the count, no metric
  emitted, and no event published. A consumer monitoring lag
  via Prometheus / OpenTelemetry cannot observe the lag; a
  consumer that wants to re-replay the missed gap has no hook
  to request it. Per `docs/ports/event-bus.md:84-86` the bus
  promises a DLQ for events that fail repeatedly; a `Lagged`
  error is a "consumer cannot keep up" failure mode that
  should route to the DLQ for inspection, but no DLQ exists
  on the in-process bus.

**Expected:**

`docs/ports/event-bus.md:84-86` (DLQ contract).

**Evidence:**

```rust
  Err(broadcast::error::RecvError::Lagged(skipped)) => {
      debug!(
          consumer = %self.consumer,
          skipped, "subscription lagged; skipping past missed envelopes"
      );
      continue;
  }
  ```

---

### FINDING 17 (id: `ADAPT-EB-017`)

- **Source:** `docs/audit_reports/findings/wave3-event-bus.md`
- **Severity:** High
- **Area:** adapters-event-bus
- **Location:** `crates/adapters/event-bus/src/in_process.rs:172-200`

**Description:**

`publish_batch` at
  `crates/adapters/event-bus/src/in_process.rs:202-219` is
  implemented as a sequential loop that calls `publish` per
  envelope and short-circuits on the first failure. The
  port-doc comment at `docs/ports/event-bus.md:179-187` (and
  the trait rustdoc at `event_bus.rs:67-72`) says "Adapters
  that don't support atomic batching should fall back to per-
  envelope `publish`; consumers cannot assume either
  semantics unless they pin the adapter." The in-process
  adapter takes the fallback path but **silently truncates
  the `BatchReceipt`**: a 10-envelope batch that fails on
  envelope #3 returns a `BatchReceipt` with 2 receipts and
  no indication that the remaining 7 were not attempted. The
  cross-cutting integration test at
  `crates/tools/storage-parity/tests/cross_cutting_integration.rs`
  relies on `BatchReceipt::is_fully_accepted()` (which is
  itself broken per `docs/audit_reports/findings/wave2-events.md`
  Finding 16) to gate downstream work; the in-process
  adapter's truncation compounds the bug.

**Expected:**

`docs/ports/event-bus.md:33` (BatchReceipt
  shape) and `docs/ports/event-bus.md:67` ("Adapters that
  don't support atomic batching...").

**Evidence:**

```rust
  async fn publish_batch(
      &self,
      envelopes: Vec<EventEnvelope>,
  ) -> educore_core::error::Result<BatchReceipt> {
      let mut receipts = Vec::with_capacity(envelopes.len());
      for env in envelopes {
          let receipt = self.publish(env).await?;
          receipts.push(receipt);
      }
      Ok(BatchReceipt {
          receipts,
          correlation_id: None,
      })
  }
  ```

---

### FINDING 4 (id: `ADAPT-EB-004`)

- **Source:** `docs/audit_reports/findings/wave3-event-bus.md`
- **Severity:** High
- **Area:** adapters-event-bus
- **Location:** `crates/adapters/event-bus/src/errors.rs:45-49`

**Description:**

The `subscribe_failed` helper builds an
  `EventError::PublishFailed` with the message
  `"subscribe failed: ..."`. A subscribe-side error (e.g. the
  replay log mutex is poisoned — the only call site in
  `in_process.rs:316`) is reported to the consumer as a
  publish failure. Consumers that match
  `DomainError::Infrastructure("publish failed: ...")` to
  distinguish retryable transport failures from subscribe-side
  configuration errors will misroute subscribe failures to
  their publish retry path. The bus-port contract has no
  `SubscribeFailed` variant, so the adapter is forced to
  misclassify; this is a port gap surfaced by the adapter
  impl.

**Expected:**

`docs/ports/event-bus.md:179-187` enumerates the
  `EventBusError` enum (no `SubscribeFailed` variant). The
  `EventError` enum in
  `crates/cross-cutting/events/src/errors.rs:23-44` adds
  `NotSupported` and `Infrastructure` variants on top of the
  port doc, but no `SubscribeFailed` variant exists.

**Evidence:**

```rust
  #[inline]
  pub fn subscribe_failed(msg: impl Into<String>) -> EventError {
      EventError::PublishFailed(format!("subscribe failed: {}", msg.into()))
  }
  ```

---

### FINDING 5 (id: `ADAPT-EB-005`)

- **Source:** `docs/audit_reports/findings/wave3-event-bus.md`
- **Severity:** High
- **Area:** adapters-event-bus
- **Location:** `crates/adapters/event-bus/src/in_process.rs:262-275`

**Description:**

The replay log mutex is held inside the
  `publish` critical section and contains both the `pop_front`
  (when at capacity) and the `push_back` of the cloned
  envelope. A single poisoned mutex causes every subsequent
  publish to return `EventError::PublishFailed("replay log
  mutex poisoned")`; the envelope is not published, not in
  the log, and the broadcast channel is never reached. There
  is no recovery path; the bus is wedged until the process
  restarts. The `broadcast::Sender::send` (which is the actual
  delivery path) is independent of the replay log; the log is
  advisory. Locking the publish path on a poisoned advisory
  structure is unsafe.

**Expected:**

`docs/ports/event-bus.md:71-77` (at-least-once
  delivery; consumers must not lose events on transient
  failures).

**Evidence:**

```rust
  if self.inner.config.replay_log_capacity > 0 {
      match self.inner.log.lock() {
          Ok(mut log) => {
              if log.len() == self.inner.config.replay_log_capacity {
                  log.pop_front();
              }
              log.push_back(env.clone());
          }
          Err(_) => {
              return Err(
                  EventError::PublishFailed("replay log mutex poisoned".to_owned()).into(),
              );
          }
      }
  }
  ```

---

### FINDING 6 (id: `ADAPT-EB-006`)

- **Source:** `docs/audit_reports/findings/wave3-event-bus.md`
- **Severity:** High
- **Area:** adapters-event-bus
- **Location:** `crates/adapters/event-bus/src/in_process.rs:128-134`

**Description:**

`InProcessConfig::with_capacity` clamps the
  replay log capacity to `0` and the channel capacity to
  `clamp_capacity(c)` (which clamps `0` to `1`). Consumers
  that call `InProcessEventBus::with_capacity(0)` to mean "no
  channel, no log" instead get a `channel_capacity = 1` bus
  that can buffer exactly one envelope per subscriber. The
  config rustdoc says "Returns a config with both capacities
  clamped to `0` and the given capacity (`1..=u32::MAX`)" — the
  prose is contradictory (it says "clamped to `0` and the given
  capacity"). A consumer reading the docstring expects a
  `0..=u32::MAX` range; the actual range is `1..=u32::MAX` for
  the channel. The replay log is hard-coded to `0`, which
  silently disables replay even if the consumer passed a
  non-zero capacity — this is a footgun that turns
  `StartPosition::Earliest` into `Latest` with no warning.

**Expected:**

`docs/ports/event-bus.md:106-110` (replay
  contract — replay is mandatory for projection rebuilds).

**Evidence:**

```rust
  pub fn with_capacity(capacity: usize) -> Self {
      Self {
          channel_capacity: clamp_capacity(capacity),
          replay_log_capacity: 0,
      }
  }
  ```
  And `clamp_capacity`:
  ```rust
  fn clamp_capacity(c: usize) -> usize {
      // The broadcast channel rejects 0; the replay log accepts 0.
      if c == 0 {
          1
      } else {
          c
      }
  }
  ```

---

### FINDING 7 (id: `ADAPT-EB-007`)

- **Source:** `docs/audit_reports/findings/wave3-event-bus.md`
- **Severity:** High
- **Area:** adapters-event-bus
- **Location:** `crates/adapters/event-bus/src/in_process.rs:431-445`

**Description:**

`InProcessSubscription::next` swallows
  `broadcast::error::RecvError::Lagged(skipped)` with a
  `continue` after a `debug!` log. The `skipped` count is
  recorded but the subscription has no mechanism to
  re-deliver or re-replay the missed envelopes from the bus.
  `nack(requeue = true)` is a no-op (FINDING 2). A subscriber
  that falls behind by more than `channel_capacity` envelopes
  permanently loses the gap. There is no DLQ, no replay-from-
  event-id hook, and the consumer cannot request a re-subscribe
  with `StartPosition::FromEventId(last_seen_id)` because the
  bus does not surface `last_seen_id` back to the consumer.
  Per `docs/ports/event-bus.md:71-77` this is at-most-once
  delivery under back-pressure.

**Expected:**

`docs/ports/event-bus.md:71-77` (at-least-once
  delivery); `docs/ports/event-bus.md:84-86` (DLQ).

**Evidence:**

```rust
  Err(broadcast::error::RecvError::Lagged(skipped)) => {
      debug!(
          consumer = %self.consumer,
          skipped, "subscription lagged; skipping past missed envelopes"
      );
      continue;
  }
  ```

---

### FINDING 8 (id: `ADAPT-EB-008`)

- **Source:** `docs/audit_reports/findings/wave3-event-bus.md`
- **Severity:** High
- **Area:** adapters-event-bus
- **Location:** `crates/adapters/event-bus/src/in_process.rs:329-339`
  and `:457-492`

**Description:**

`InProcessSubscription::next` filters by
  `Topic` and `EventFilter` in the subscription loop, but the
  filters are re-applied on every envelope after the broadcast
  `recv`. A subscription on `Topic::All` with no filter pays
  the full broadcast-receive cost per envelope; a subscription
  on `Topic::EventType("academic.student.admitted")` still
  receives every envelope on the global channel and drops the
  non-matching ones in the loop. For a bus with 1024
  subscribers on disjoint topics this is O(N) work per publish
  — the same work the port-doc attributes to "per-topic routing
  ... applied in the subscription's next loop" (`in_process.rs:
  9-18`). There is no per-topic fan-out; the design scales
  linearly with subscriber count, not topic cardinality.

**Expected:**

`docs/ports/event-bus.md:117-121` (topic naming
  conventions) and `docs/ports/event-bus.md:71-77` (bus
  performance under fan-out).

**Evidence:**

```rust
  match self.receiver.recv().await {
      Ok(env) => {
          if !topic_matches(&self.topic, &env) {
              continue;
          }
          if !filter_matches(self.filter.as_ref(), &env) {
              continue;
          }
          return Some(Ok(env));
      }
      ...
  }
  ```

---

### FINDING 10 (id: `ADAPT-EB-010`)

- **Source:** `docs/audit_reports/findings/wave3-event-bus.md`
- **Severity:** Medium
- **Area:** adapters-event-bus
- **Location:** `crates/adapters/event-bus/src/nats.rs:55-83`
  and `crates/adapters/event-bus/src/redis.rs:69-104`

**Description:**

`NatsEventBus::connect` and
  `RedisEventBus::connect` succeed silently and update an
  internal `client` / `config` slot, but the trait methods
  still return `NotSupported` because the wire-protocol work
  is deferred. A consumer that calls `connect`, observes
  `is_connected() == true`, and then calls `publish` will
  receive `EventError::NotSupported("NatsEventBus::publish")`
  — the `is_connected` flag is misleading; it reports
  "client is wired" not "the bus can deliver". The
  misleading boolean violates the principle of least surprise
  and the test in `crates/adapters/event-bus/tests/in_process_e2e.rs`
  `nats_bus_returns_not_supported_without_connection` asserts
  this exact path (without `connect`).

**Expected:**

`is_connected` should be renamed to
  `has_wired_client` or removed; the rustdoc should clarify
  the stub state.

**Evidence:**

```rust
  pub async fn is_connected(&self) -> bool {
      self.client.lock().await.is_some()
  }
  ```
  (identical body in `redis.rs:103-105`).

---

### FINDING 11 (id: `ADAPT-EB-011`)

- **Source:** `docs/audit_reports/findings/wave3-event-bus.md`
- **Severity:** Medium
- **Area:** adapters-event-bus
- **Location:** `crates/adapters/event-bus/src/lib.rs:10-19` and
  `crates/adapters/event-bus/Cargo.toml:14-25`

**Description:**

The `in-process` Cargo feature is a marker
  feature (no deps, no cfg gates) and is the default. The
  crate's `default = ["in-process"]` means `cargo build` always
  pulls in `InProcessEventBus`. Consumers that want a
  distributed-only build (`default-features = false`) get a
  crate with `in-process` feature present but the module is
  still compiled (the `in_process` module is `pub mod
  in_process;` with no `#[cfg(feature = "in-process")]`
  gate at `crates/adapters/event-bus/src/lib.rs:32-34`). The
  marker feature is dead — `InProcessEventBus` is always
  available regardless of feature flags. The README claims
  "consumers can opt out in tests if they want to verify a
  `default-features = false` build" (`Cargo.toml:17-21`); this
  claim is false.

**Expected:**

Either gate `in_process` on the feature or
  remove the marker feature.

**Evidence:**

```rust
  // crates/adapters/event-bus/src/lib.rs:32-34
  /// The in-process MPMC event bus.
  pub mod in_process;
  ```
  And `crates/adapters/event-bus/Cargo.toml:14-25`:
  ```toml
  default = ["in-process"]
  # Marker feature for the in-process bus. Always enabled by default;
  # listed as a feature so consumers can opt out in tests if they
  # want to verify a `default-features = false` build of the
  # adapter crate (e.g., for the distributed-only path).
  in-process = []
  ```

---

### FINDING 12 (id: `ADAPT-EB-012`)

- **Source:** `docs/audit_reports/findings/wave3-event-bus.md`
- **Severity:** Medium
- **Area:** adapters-event-bus
- **Location:** `crates/adapters/event-bus/src/in_process.rs:412-425`

**Description:**

`topic_matches` for `Topic::Aggregate(d, a)`
  matches the wire string `"<d>.<a>"` exactly, but the
  `EventEnvelope::aggregate_topic` helper at
  `crates/cross-cutting/events/src/envelope.rs:79-88` returns
  `"<domain>.<aggregate>"` only when `event_type` has a `.`
  separator; for events whose `event_type` has no `.` (e.g.,
  `"school"`), `aggregate_topic` returns just `aggregate_type`
  (e.g., `"school"`). A `Topic::Aggregate("platform",
  "school")` subscription against an envelope with
  `event_type = "school"` (no domain prefix) will miss the
  envelope even though the envelope is for the same aggregate.
  The unit test `topic_matches_handles_all_variants` exercises
  `SyncStarted` whose `event_type = "sync.session.started"`
  (has a `.`), so the gap is not caught.

**Expected:**

`Topic::Aggregate(d, a)` should match
  `aggregate_topic()` defensively (both the full string and
  the bare aggregate form).

**Evidence:**

```rust
  fn topic_matches(topic: &Topic, env: &EventEnvelope) -> bool {
      match topic {
          Topic::Aggregate(d, a) => env.aggregate_topic() == format!("{d}.{a}"),
          ...
      }
  }
  ```
  And the fallback in
  `crates/cross-cutting/events/src/envelope.rs:79-88`:
  ```rust
  pub fn aggregate_topic(&self) -> String {
      match self.event_type.split_once('.') {
          Some((domain, _)) if !domain.is_empty() => {
              format!("{domain}.{}", self.aggregate_type)
          }
          _ => self.aggregate_type.to_owned(),
      }
  }
  ```

---

### FINDING 18 (id: `ADAPT-EB-018`)

- **Source:** `docs/audit_reports/findings/wave3-event-bus.md`
- **Severity:** Medium
- **Area:** adapters-event-bus
- **Location:** `crates/adapters/event-bus/src/redis.rs:39-67`

**Description:**

`RedisBusConfig` stores a
  `redis::aio::ConnectionManager` behind an `Arc<TokioMutex>`.
  `ConnectionManager` is itself internally `Arc`-based and
  clone-cheap; wrapping it in `Arc<TokioMutex<Option<...>>>`
  allocates an extra `Arc` and forces every config access
  through the Tokio mutex. The intent appears to be to allow
  hot-swap of the connection, but no method on `RedisEventBus`
  performs a swap (the field is write-once via `connect`).
  The double-Arc is dead weight on every `is_connected` /
  connect path. The `NatsEventBus` at `nats.rs:42-50` does
  the same dance with `async_nats::Client` inside an
  `Arc<TokioMutex<Option<...>>>` for the same reason.

**Expected:**

Idiomatic Rust would use
  `Arc<RwLock<Option<ConnectionManager>>>` (or
  `tokio::sync::RwLock`) if hot-swap is intended, or
  `OnceCell<ConnectionManager>` if it is not.

**Evidence:**

```rust
  pub struct RedisEventBus {
      config: Arc<TokioMutex<Option<RedisBusConfig>>>,
  }
  ```
  And `RedisBusConfig`:
  ```rust
  pub struct RedisBusConfig {
      pub url: String,
      pub manager: redis::aio::ConnectionManager,
  }
  ```

---

### FINDING 19 (id: `ADAPT-EB-019`)

- **Source:** `docs/audit_reports/findings/wave3-event-bus.md`
- **Severity:** Medium
- **Area:** adapters-event-bus
- **Location:** `crates/adapters/event-bus/src/in_process.rs:457-475`

**Description:**

The `EventSubscription::ack` and `nack`
  trait methods return `Result<AckOutcome>` in the port trait
  at `crates/cross-cutting/events/src/event_bus.rs:78-101`
  (and implemented in `in_process.rs:457-475`), but the bus
  port contract at `docs/ports/event-bus.md:78-82` declares:
  ```rust
  async fn ack(&mut self, event_id: EventId) -> Result<()>;
  async fn nack(&mut self, event_id: EventId, requeue: bool) -> Result<()>;
  ```
  The return type in the docstring is `Result<()>`. A consumer
  that writes against the docstring signature will not
  compile against the actual trait (extra `AckOutcome` in the
  return). The `AckOutcome` enum at
  `crates/cross-cutting/events/src/event_bus.rs:51-61` adds
  three variants (`Accepted`, `Unknown`, `Failed`) that are
  not represented in the port-doc enum `EventBusError` at
  `docs/ports/event-bus.md:179-187`. This is the same
  deviation flagged in `docs/audit_reports/findings/wave2-events.md`
  Finding 3 (`CC-EVT-003`), but the adapter inherits the
  divergence and propagates it to consumers of
  `InProcessEventBus`.

**Expected:**

`docs/ports/event-bus.md:78-82` (return type
  `Result<()>`); `docs/ports/event-bus.md:179-187` (error
  enum).

**Evidence:**

```rust
  async fn ack(&mut self, event_id: EventId) -> Result<AckOutcome>;
  async fn nack(&mut self, event_id: EventId, requeue: bool) -> Result<AckOutcome>;
  ```
  In `crates/cross-cutting/events/src/event_bus.rs:88-100` and
  the implementation in
  `crates/adapters/event-bus/src/in_process.rs:457-475`.

---

### FINDING 20 (id: `ADAPT-EB-020`)

- **Source:** `docs/audit_reports/findings/wave3-event-bus.md`
- **Severity:** Medium
- **Area:** adapters-event-bus
- **Location:** `crates/adapters/event-bus/src/in_process.rs:117-127`

**Description:**

`InProcessInner` holds a `std::sync::Mutex`
  guarding the replay log. `publish` is `async fn` and locks
  this mutex while the broadcast `send` (which is also async-
  aware but does not block the runtime in the same way) is
  attempted. The lock is held across the clone of the
  envelope (`log.push_back(env.clone())`) and the
  `pop_front` on capacity. A burst of publishes from
  multiple producers serialises on this mutex; under
  contention the lock is held across an allocation (the
  `clone`). Per `docs/build-plan.md:497` the bus is intended
  to scale to "10k students × 5 daily commands × 200 schools"
  volumes; a serialising mutex on the hot path is a
  bottleneck. There is no sharding; a `parking_lot::Mutex`
  or a per-producer sharded log would scale better.

**Expected:**

`docs/ports/event-bus.md:71-77` (at-least-once
  delivery at scale); `docs/build-plan.md:497` (audit log
  volume — same scale argument).

**Evidence:**

```rust
  if self.inner.config.replay_log_capacity > 0 {
      match self.inner.log.lock() {
          Ok(mut log) => {
              if log.len() == self.inner.config.replay_log_capacity {
                  log.pop_front();
              }
              log.push_back(env.clone());
          }
          ...
      }
  }
  ```

---

### FINDING 21 (id: `ADAPT-EB-021`)

- **Source:** `docs/audit_reports/findings/wave3-event-bus.md`
- **Severity:** Medium
- **Area:** adapters-event-bus
- **Location:** `crates/tools/testkit/src/event_bus.rs:1-37` and
  `docs/coverage.toml:2183-2195`

**Description:**

The testkit exposes
  `educore_testkit::event_bus::InMemoryEventBus` as a `type`
  alias for `educore_event_bus::InProcessEventBus`. The
  rustdoc on the alias claims it is a "Testkit-local alias"
  and that "The alias exists so consumers can write `use
  educore_testkit::event_bus::InMemoryEventBus;` without
  taking a direct dep on `educore-event-bus`." But the testkit
  crate's `Cargo.toml` (per `docs/coverage.toml:2183-2195`
  and the crate's own deps) depends on `educore-event-bus`
  to provide the alias; consumers that use `InMemoryEventBus`
  therefore still pull in the `educore-event-bus` crate. The
  alias adds nothing except a name; the lib rustdoc at
  `crates/adapters/event-bus/src/lib.rs:6` refers to the
  same `InProcessEventBus` as the "default" — the testkit's
  rebranding is purely cosmetic. A test consumer that searches
  the engine for "the default bus" finds two names for the
  same type and has to choose between them.

**Expected:**

The alias is a stylistic convenience; the
  testkit should either (a) remove the alias and force tests
  to use `InProcessEventBus` directly, or (b) document the
  alias as a stable re-export for `educore-testkit` consumers
  and not as a separate type.

**Evidence:**

```rust
  pub use educore_event_bus::InProcessEventBus;
  pub type InMemoryEventBus = InProcessEventBus;
  ```
  And `crates/adapters/event-bus/src/lib.rs:6-19`:
  ```rust
  //! - [`InProcessEventBus`] — the default, always-built, MPMC
  //!   bus backed by [`tokio::sync::broadcast`].
  ```

---

### FINDING 22 (id: `ADAPT-EB-022`)

- **Source:** `docs/audit_reports/findings/wave3-event-bus.md`
- **Severity:** Medium
- **Area:** adapters-event-bus
- **Location:** `crates/adapters/event-bus/src/nats.rs:13-31`
  and `crates/adapters/event-bus/src/redis.rs:13-32`

**Description:**

The NATS and Redis stubs document a
  future subject-mapping convention (`events.<d>.<a>`,
  `events.<d>.>`, `tenant.<s>.>`, `events.>`) in their
  rustdoc but the convention does not match the port-doc at
  `docs/ports/event-bus.md:117-121` which declares topic
  naming as `<domain>.<aggregate>` for aggregates and
  `tenant.<school_id>` for tenants. The NATS stub's proposed
  `events.<d>.<a>` prefix adds a leading `events.` segment
  not in the port doc; the Redis stub's proposed
  `stream:events:<d>:<a>` introduces a `stream:` key prefix
  not in the port doc. When the wire-protocol work lands,
  the consumer will need to translate between the bus-port
  topic strings and the adapter-specific wire strings; this
  translation is not declared anywhere. The
  `Topic::wire()` helper at
  `crates/cross-cutting/events/src/event_bus.rs:206-217`
  returns the bus-port form (no `events.` prefix, no
  `stream:` prefix); the adapter convention diverges.

**Expected:**

`docs/ports/event-bus.md:117-121` (topic
  naming convention).

**Evidence:**

```text
  - `Aggregate(d, a)` → `events.<d>.<a>`
  - `Domain(d)` → `events.<d>.>`
  - `EventType(t)` → `events.<dotted t>`
  - `Tenant(s)` → `tenant.<s>.>`
  - `All` → `events.>`
  ```
  (in `crates/adapters/event-bus/src/nats.rs:14-21` rustdoc).

---

### FINDING 9 (id: `ADAPT-EB-009`)

- **Source:** `docs/audit_reports/findings/wave3-event-bus.md`
- **Severity:** Medium
- **Area:** adapters-event-bus
- **Location:** `crates/adapters/event-bus/src/in_process.rs:486-492`

**Description:**

`InProcessSubscription::close` consumes
  `self: Box<Self>`, dereferences the box to set
  `me.closed = true`, then lets `me` drop at end of scope. The
  `broadcast::Receiver` is dropped at end of scope, releasing
  the broadcast slot. But because the subscription was
  unboxed into a stack value, the `drop` order is: (1) set
  `closed = true` on the stack copy, (2) call
  `me.bus.strong_count()` for the diagnostic, (3) drop the
  `Weak<InProcessInner>`, (4) drop `closed: bool` (no-op),
  (5) drop the rest. The boxed-deref pattern is unusual; the
  e2e test `subscription_close_releases_resources` passes
  because `broadcast::Sender::receiver_count` is checked
  synchronously after the `close().await` returns — but the
  release happens during the drop, which the test does not
  observe deterministically. If `close` ever needs to await
  an ack of close (e.g., a distributed adapter flushing
  pending offsets), the unbox makes that impossible because
  `me` is a stack value, not `Pin`.

**Expected:**

Idiomatic `async_trait` `close(self: Box<Self>)`
  should keep the box and drop on `Drop` or use `Pin<Box<Self>>`
  if async-drop is required.

**Evidence:**

```rust
  async fn close(self: Box<Self>) -> educore_core::error::Result<()> {
      // Drop the receiver (releases its slot in the broadcast
      // channel) and the replay buffer (clears the heap).
      let mut me = *self;
      me.closed = true;
      ...
  }
  ```

---

