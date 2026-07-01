# 08 - Audit Appendix - Security + Tests (deep audits)

**Scope:** wave7-security.md (cross-cutting: auth, rbac, platform, audit, storage, secrets) + wave7-tests.md (test coverage, parity, harness)

**Total findings:** 94

**Severity distribution:** 26 critical, 39 high, 24 medium, 5 low


## Summary Table

| Target | Critical | High | Medium | Low | Total |
| --- | --- | --- | --- | --- | --- |
| Security — Auth (`SEC-AUTH`) | 5 | 6 | 5 | 0 | 16 |
| Security — RBAC (`SEC-RBAC`) | 2 | 3 | 1 | 0 | 6 |
| Security — Platform (`SEC-PLAT`) | 3 | 2 | 2 | 0 | 7 |
| Security — Audit log (`SEC-AUDIT`) | 3 | 2 | 2 | 0 | 7 |
| Security — Storage (`SEC-STORAGE`) | 0 | 3 | 1 | 0 | 4 |
| Security — Secrets handling (`SEC-SECRETS`) | 0 | 1 | 1 | 0 | 2 |
| Tests (deep audit) (`TST`) | 13 | 22 | 12 | 5 | 52 |

## Security — Auth (target id prefix: `SEC-AUTH`)

**Path:** `crates/adapters/auth/ + cross-cutting/platform`  
**Total findings:** 16 (5 critical, 6 high, 5 medium, 0 low)


### FINDING 10 (id: `SEC-AUTH-004`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** Critical
- **Area:** security
- **Location:** crates/adapters/auth/src/jwt.rs:111-116, 365-380

**Description:**

The token revocation set is a process-local `Arc<Mutex<HashSet<String>>>`. Any horizontally-scaled deployment (the default SaaS topology) has N replicas, each with its own revocation set. A token revoked on replica A remains valid on replicas B..N until expiry. The port spec acknowledges this as a "consumer-must-layer-shared-store" responsibility, but no shared-store adapter is shipped and the reference implementation exposes no hook (e.g. `RevocationStore` trait) for the consumer to plug one in. The result is a documented security gap with no production-safe default.

**Expected:**

`docs/ports/authentication.md` — "Cross-process token revocation is required for any horizontally-scaled deployment."

**Evidence:**

`crates/adapters/auth/src/jwt.rs:111-116`:
```rust
revoked_sessions: Arc<Mutex<HashSet<String>>>,
```
and the deviation note at lines 31-34: "Token revocation: an in-memory `HashSet<String>` keyed by `sid`. The set is process-local; consumers that need cross-process revocation must layer a shared store on top."

---

### FINDING 13 (id: `SEC-AUTH-007`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** Critical
- **Area:** security
- **Location:** crates/adapters/auth/src/services.rs:443-475, crates/adapters/auth/src/jwt.rs:330-345

**Description:**

There is no rate-limit, account-lockout, or brute-force protection anywhere in the auth crate. `Credential::UsernamePassword { username, password }` is routed to `Err(AuthError::InvalidCredentials)` in `JwtAuthProvider::authenticate` (jwt.rs:393-394), but no counter, no exponential backoff, no IP throttling. A leaked or weak password can be brute-forced at line-rate against `JwtAuthProvider::authenticate`. The `password_reset` table in `oauth_store.rs:181-205` similarly has no rate limit on `insert` / `get_by_email`.

**Expected:**

`docs/ports/authentication.md` — failed login attempts must be rate-limited; password reset endpoints must be rate-limited.

**Evidence:**

`crates/adapters/auth/src/jwt.rs:393-394`:
```rust
Credential::UsernamePassword { .. }
| Credential::Oauth2 { .. }
| Credential::Saml { .. }
| Credential::ApiKey { .. }
| Credential::Biometric { .. } => Err(AuthError::InvalidCredentials),
```
The early-return path has no counter, no audit log entry beyond the catch-all `Err(AuthError::InvalidCredentials)`. `auth_action_for_op` in `oauth_store.rs:111-120` only fires on insert/revoke/purge ops in the OAuth store, not on failed-credential attempts.

---

### FINDING 7 (id: `SEC-AUTH-001`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** Critical
- **Area:** security
- **Location:** crates/adapters/auth/src/jwt.rs:331-350, 389-392

**Description:**

`JwtAuthProvider::authenticate` accepts `Credential::Anonymous` and returns a fully-formed `Session` with `mfa_satisfied: true`, `user_id: SYSTEM_USER_ID`, `active_school_id: PUBLIC_SCHOOL_ID`, and an empty capability set. The crate's own port-deviation note documents this as a deviation from the spec, but no builder knob (`allow_anonymous`, `public_school_only`) is exposed — every consumer of the reference provider gets anonymous access to the platform school by default.

**Expected:**

`docs/ports/authentication.md:38-40` — "A `Credential::Anonymous` is rejected by the default adapters except in public-facing flows (e.g. public exam result lookup, when explicitly allowed by configuration)."

**Evidence:**

`crates/adapters/auth/src/jwt.rs:389-392`:
```rust
Credential::Anonymous => Ok(self.anonymous_session()),
```
with `mfa_satisfied: true` and `user_id: SYSTEM_USER_ID` set in `anonymous_session()` at lines 332-350.

---

### FINDING 8 (id: `SEC-AUTH-002`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** Critical
- **Area:** security
- **Location:** crates/adapters/auth/src/jwt.rs:167-176, 332-350

**Description:**

`JwtAuthProviderBuilder::new()` generates a fresh 32-byte random signing key via `rand::thread_rng()` on every call. The crate's own port-deviation note acknowledges this, but the builder emits no warning at runtime when a consumer forgets to call `.signing_key(env::var("JWT_SECRET")?)`. If a consumer wires the default builder into production, every process restart invalidates every previously-issued JWT, every replica in a horizontally-scaled deployment signs with a different key (no shared secret), and the JWTs issued in dev/tests are unverifiable in production.

**Expected:**

`docs/ports/authentication.md:175-180` (Configuration section) requires the signing key to be loaded from configuration; the default key path is documented for tests only.

**Evidence:**

`crates/adapters/auth/src/jwt.rs:167-176`:
```rust
#[must_use]
pub fn new() -> Self {
    let mut key = vec![0_u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    Self {
        signing_key: key,
        ...
```
No `tracing::warn!`, no panic in production builds, no env var probe, no `.signing_key` precondition check in `build()`.

---

### FINDING 9 (id: `SEC-AUTH-003`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** Critical
- **Area:** security
- **Location:** crates/adapters/auth/src/jwt.rs:417-450 (refresh method)

**Description:**

`JwtAuthProvider::refresh` validates the old token (signature + exp + not revoked), then mints a new token that reuses the same `sid`. The new token is returned as the result of `refresh` but the function drops the encoded token (the local `let _refreshed_token = self.encode(&new_claims)?;` is unused) and returns the new `Session` directly. Consumers therefore have no way to obtain the refreshed bearer token through the port — every refresh round-trip requires the consumer to either re-implement `encode` or call back through the provider's private path. From a security standpoint the bigger issue is that `refresh` does NOT add the old `sid` to the revocation set, so the original token remains valid after refresh. A leaked access token can be refreshed indefinitely; revocation only happens when the consumer explicitly calls `revoke`.

**Expected:**

`docs/ports/authentication.md` §"Refresh tokens" — refreshed tokens should rotate the session id and the old token should be revoked; the new token is the only credential that survives a refresh.

**Evidence:**

`crates/adapters/auth/src/jwt.rs:417-450`:
```rust
let new_claims = JwtClaims {
    ...
    sid: old_claims.sid,  // same sid
    ...
};
let _refreshed_token = self.encode(&new_claims)?;  // dropped
self.session_from_claims(&new_claims)
```
The docstring at line 437 explicitly states: "We do NOT add the sid to the revocation set on refresh".

---

### FINDING 11 (id: `SEC-AUTH-005`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** High
- **Area:** security
- **Location:** crates/adapters/auth/src/services.rs:283-360 (PasswordService)

**Description:**

`PasswordService::hash_password` calls `SaltString::generate(&mut argon2::password_hash::rand_core::OsRng)` per call. The Argon2 parameters are `Argon2::default()` (OWASP 2024 baseline). The `needs_rehash` function compares only the algorithm identifier (`"argon2id"`) and explicitly does NOT compare cost parameters (`m`, `t`, `p`). When the engine rotates its default cost parameters (the typical OWASP rotation cadence is every 2-3 years), existing hashes will NOT be detected as needing rehash, and the upgrade-on-login path will silently no-op — old params will be in production indefinitely.

**Expected:**

`docs/ports/authentication.md` §"Password hashing" — hashes must be transparent-rotated when engine parameters change.

**Evidence:**

`crates/adapters/auth/src/services.rs:329-358`:
```rust
pub fn needs_rehash(&self, hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else { return true; };
    parsed.algorithm.as_str() != "argon2id"
}
```
The comment at lines 339-355 acknowledges the gap ("The cost parameters are intentionally not compared here") and states "The algorithm check is the meaningful signal anyway — when the engine rotates its default parameters, the migration plan bumps the algorithm tag". The PHC `$argon2id$` tag does not change when only `m`, `t`, or `p` is rotated — the migration plan described does not exist.

---

### FINDING 12 (id: `SEC-AUTH-006`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** High
- **Area:** security
- **Location:** crates/adapters/auth/src/services.rs:443-475 (MfaService::verify_code)

**Description:**

`MfaService::verify_code` accepts codes from the previous, current, and next 30-second window (±1 step, RFC 6238 §4.2 tolerance). The implementation iterates `[-1, 0, 1]` and short-circuits on the first match via `constant_time_eq`. This is correct for valid users, but the loop structure means the **time** to verify a wrong code differs based on which counter step matches (if any) and on how many early-exits occur. Combined with the fact that the comparison happens off the locked critical section (the service is `Copy` + stateless), a timing side-channel could leak whether the secret's first byte matches the supplied code's first byte.

**Expected:**

`docs/ports/authentication.md` §"MFA / TOTP" — verification must be constant-time across the whole 8-digit space, not just per-step.

**Evidence:**

`crates/adapters/auth/src/services.rs:461-475`:
```rust
for delta in [-1_i64, 0, 1] {
    ...
    if constant_time_eq(candidate.as_bytes(), code.as_bytes()) {
        return true;
    }
}
false
```
The early-return per delta leaks which counter step matches; an attacker observing verification latency can narrow the search space.

---

### FINDING 14 (id: `SEC-AUTH-008`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** High
- **Area:** security
- **Location:** crates/adapters/auth/src/jwt.rs:404-413 (revoke method)

**Description:**

`JwtAuthProvider::revoke` decodes the token, adds the `sid` to the revocation set, and returns `Ok(())`. There is no audit log emission on revocation. A regulator audit ("which tokens were revoked, by whom, when?") cannot be answered because the revoke path skips the audit trail. The `access_ttl_secs` of 1 hour means revocation is the only mechanism to invalidate a leaked token before expiry, and there is no record of the revocation event itself.

**Expected:**

`docs/schemas/audit-schema.md` §4 item 3: "Every authentication event — login, logout, token issuance, password reset, 2FA challenge." Revocation is an authentication-related event and must be audited.

**Evidence:**

`crates/adapters/auth/src/jwt.rs:404-413`:
```rust
async fn revoke(&self, token: &AuthToken) -> Result<(), AuthError> {
    if !matches!(token.scheme, AuthScheme::Bearer) { ... }
    let claims = self.decode(&token.value)?;
    self.add_revoked(&claims.sid);
    Ok(())
}
```
No call to any audit writer; no event bus emission.

---

### FINDING 15 (id: `SEC-AUTH-009`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** High
- **Area:** security
- **Location:** crates/adapters/auth/src/services.rs:201-225 (OAuthScopeService)

**Description:**

`OAuthScopeService::required_scopes_for_action` returns an empty `Vec` for unknown actions (line 225: `_ => Vec::new()`). The comment states "Adapters that receive an empty list should deny by default (fail-closed)". However, the function returns the empty vector to the **caller** (the adapter) which must implement the fail-closed check itself. There is no enforcement layer in the port; the test at line 222 only documents the comment, not the enforcement. An adapter that consumes `required_scopes_for_action("unknown_action")` and treats empty as "no scopes needed" silently allows the action.

**Expected:**

`docs/ports/authentication.md` §"OAuth scope enforcement" — default-deny on unknown action verbs.

**Evidence:**

`crates/adapters/auth/src/services.rs:201-225`:
```rust
#[must_use]
pub fn required_scopes_for_action(action: &str) -> Vec<String> {
    match action {
        "read:user" => vec!["profile:read".to_owned()],
        ...
        // Unknown action: empty scope list. Adapters that
        // receive an empty list should deny by default
        // (fail-closed). Returning an empty Vec instead of
        // an error lets the caller decide its own policy.
        _ => Vec::new(),
    }
}
```
The "should deny by default" guidance is documented but not enforced.

---

### FINDING 20 (id: `SEC-AUTH-014`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** High
- **Area:** security
- **Location:** crates/adapters/auth/src/oauth_store.rs:111-205

**Description:**

`InMemoryOAuthStore` is the reference implementation of `OAuthAccessTokenRepository`, `OAuthClientRepository`, `PasswordResetRepository`, and `MigrationRepository`. Every state-changing method computes an `AuditAction` via `audit_action_for_op` and then **discards it** (`let _action = audit_action_for_op(...)` at lines 141, 147, 155, 174, 180, 200, 209, 218, 226). No audit writer is invoked. The crate's own comment at lines 56-60 admits: "The reference store itself does not write audit rows — audit emission belongs to the command handler, not the repository port." But the command handler that would perform the audit emission is not shipped; the `InMemoryOAuthStore` is the production reference.

**Expected:**

`docs/schemas/audit-schema.md` §4 item 3: "Every authentication event — login, logout, token issuance, password reset, 2FA challenge." OAuth access token issuance, client registration, and password reset operations are authentication events.

**Evidence:**

`crates/adapters/auth/src/oauth_store.rs:111-120, 141, 147, 155`:
```rust
fn audit_action_for_op(op: &str) -> AuditAction { ... }
async fn insert(&self, t: &OAuthAccessToken) -> StorageResult<()> {
    let _action = audit_action_for_op("oauth_access_token.insert");
    self.lock_tokens().insert(t.id.clone(), t.clone());
    Ok(())
}
```

---

### FINDING 21 (id: `SEC-AUTH-015`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** High
- **Area:** security
- **Location:** crates/adapters/auth/src/oauth_store.rs:91-105 (PasswordReset)

**Description:**

`PasswordReset` stores `token_hash` as a plain `String`. There is no rate-limit on `insert`, no single-use enforcement (a token can be queried multiple times via `get_by_email`), no expiry enforcement at the repository level (only an `expires_at` field that callers must check). The comment at line 182-184 implies purging is the consumer's job, but no helper enforces single-use. A leaked reset token is reusable until the consumer's purge job runs.

**Expected:**

`docs/ports/authentication.md` §"Password reset" — reset tokens must be single-use and have a short expiry enforced at the repository level.

**Evidence:**

`crates/adapters/auth/src/oauth_store.rs:194-202`:
```rust
async fn get_by_email(&self, email: &str) -> StorageResult<Option<PasswordReset>> {
    Ok(self.lock_resets().get(email).cloned())
}
```
No `used: bool` field check; no mutation of the row on read.

---

### FINDING 16 (id: `SEC-AUTH-010`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** Medium
- **Area:** security
- **Location:** crates/adapters/auth/src/jwt.rs:171-176, 251-260 (defaults)

**Description:**

The JWT issuer and audience both default to the literal string `"educore"`. If two consumers of the library deploy without overriding these, their tokens become cross-validatable — a token issued for consumer A's instance is accepted by consumer B's instance if they share the same signing key (which they do by default since both generate random keys at startup, but a misconfigured deployment that pins the same env var across consumers creates cross-tenant token acceptance).

**Expected:**

`docs/ports/authentication.md` §"Configuration" — issuer and audience must be unique per deployment.

**Evidence:**

`crates/adapters/auth/src/jwt.rs:171-176`:
```rust
Self {
    signing_key: key,
    issuer: "educore".to_owned(),
    audience: "educore".to_owned(),
    ...
```
There is no per-deployment unique value (e.g. `Uuid::new_v4()` or a deployment id from env). The defaults collide across any two consumers that don't override.

---

### FINDING 17 (id: `SEC-AUTH-011`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** Medium
- **Area:** security
- **Location:** crates/adapters/auth/src/services.rs:443-475 (MfaService)

**Description:**

`MfaService` uses HMAC-SHA1 as the underlying MAC for TOTP codes. RFC 6238 §1.2 notes that SHA-1 has known collision weaknesses and recommends SHA-256 or SHA-512 for new deployments. The implementation comment at line 138 ("SHA-1, HMAC-SHA1, and base32 (RFC 4648) are implemented inline... The implementation is exercised by the TOTP test vector in RFC 6238 Appendix B") admits the algorithm choice; HMAC-SHA1 is not currently exploitable for TOTP forgery in practice, but the algorithm is on the OWASP deprecation path and a regulator audit will flag it.

**Expected:**

`docs/ports/authentication.md` §"MFA" — algorithm selection per current OWASP / NIST guidance (HMAC-SHA256 minimum).

**Evidence:**

`crates/adapters/auth/src/services.rs:443-475`:
```rust
let mac = hmac_sha1(key, &counter_bytes);
```
and the comment on lines 130-141 acknowledging the SHA-1 choice.

---

### FINDING 18 (id: `SEC-AUTH-012`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** Medium
- **Area:** security
- **Location:** crates/adapters/auth/src/services.rs:419-441 (MfaService::generate_secret)

**Description:**

`MfaService::generate_secret` returns a base32-encoded 20-byte secret. The generation uses `rand::thread_rng()` rather than the OS CSPRNG directly. RFC 4226 §4 recommends a CSPRNG; `rand::thread_rng()` is generally thread-safe and uses ChaCha12 on supported platforms, but the comment at line 421 ("freshly-generated base32-encoded 20-byte secret... The bytes come from the thread-local CSPRNG") underspecifies the entropy source for an audit reviewer. There is no `getrandom` / `OsRng` direct path; if the consumer's platform defaults `rand` to a weak backend, secrets could be predictable.

**Expected:**

`docs/ports/authentication.md` §"MFA" — secrets must be generated from a CSPRNG.

**Evidence:**

`crates/adapters/auth/src/services.rs:419-425`:
```rust
#[must_use]
pub fn generate_secret() -> String {
    let mut bytes = [0_u8; 20];
    rand::thread_rng().fill_bytes(&mut bytes);
    base32_encode(&bytes)
}
```

---

### FINDING 19 (id: `SEC-AUTH-013`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** Medium
- **Area:** security
- **Location:** crates/adapters/auth/src/jwt.rs:181-186 (validate method, leeway = 0)

**Description:**

`JwtAuthProvider::validation()` sets `v.leeway = 0` (line 184). Clock skew between issuer and verifier (typical in distributed deployments) results in false `Expired` errors. The standard mitigation is a small leeway (30-60 seconds); the reference provider ships with no leeway, so a 1-second clock drift between replica A (issuer) and replica B (verifier) rejects every token issued within the last second. Operators disable clock sync because of false rejections; the absence of leeway is itself a security-relevant misconfiguration.

**Expected:**

`docs/ports/authentication.md` §"Configuration" — a small clock-skew leeway (30-60s) is required for distributed deployments.

**Evidence:**

`crates/adapters/auth/src/jwt.rs:181-186`:
```rust
fn validation(&self) -> Validation {
    let mut v = Validation::new(Algorithm::HS256);
    v.set_issuer(&[self.issuer.as_str()]);
    v.set_audience(&[self.audience.as_str()]);
    v.validate_exp = true;
    v.leeway = 0;
    ...
}
```

---

### FINDING 22 (id: `SEC-AUTH-016`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** Medium
- **Area:** security
- **Location:** crates/adapters/auth/src/oauth_store.rs:74-79

**Description:**

`InMemoryOAuthStore` uses `std::sync::Mutex` (line 74) for `access_tokens`, `oauth_clients`, `password_resets`, and `migrations`. The Mutex is `std::sync::Mutex`, not `tokio::sync::Mutex`. Holding a std::sync::Mutex across an `.await` point is a soundness-correctness hazard: if any future modification awaits while holding the lock, the lock spans a yield point and another task can deadlock the runtime. The current code is purely synchronous and the comment at lines 18-22 acknowledges the choice, but the OAuth scope check at `OAuthScopeService` is async and the trait is `async_trait` — a future PR that introduces a `verify_and_revoke` path that holds the lock across an async scope check will deadlock silently in production.

**Expected:**

`docs/code-standards.md` — async mutexes across `.await`.

**Evidence:**

`crates/adapters/auth/src/oauth_store.rs:74-88`:
```rust
access_tokens: Arc<Mutex<HashMap<String, OAuthAccessToken>>>,
oauth_clients: Arc<Mutex<HashMap<String, OAuthClient>>>,
password_resets: Arc<Mutex<HashMap<String, PasswordReset>>>,
migrations: Arc<Mutex<HashMap<String, Migration>>>,
```

---


## Security — RBAC (target id prefix: `SEC-RBAC`)

**Path:** `crates/cross-cutting/rbac/`  
**Total findings:** 6 (2 critical, 3 high, 1 medium, 0 low)


### FINDING 1 (id: `SEC-RBAC-001`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** Critical
- **Area:** security
- **Location:** crates/cross-cutting/rbac/src/value_objects.rs:1-6206 (entire file — see lines 4148-4767 for `Capability::all()`)

**Description:**

`Capability::all()` enumerates 608 variants while the `Capability` enum contains 654 variants. Every Phase 15 capability is missing (Auth, Notify, Payment, Files, Integrations, OAuth, Mfa, Webhook). `DefaultRoleCatalog::super_admin()` is defined as `Capability::all().iter().copied().collect()`, so the SuperAdmin role ships with an incomplete capability set. A SuperAdmin in any deployment cannot exercise AuthLogin, NotifyEmailSend, PaymentCharge, FilesPut, OAuthAccessTokenRead, MfaEnroll, etc. — silently, with no compile error.

**Expected:**

`docs/specs/rbac/permissions.md:84-86` — "The SuperAdmin role is a system role and cannot be deleted. It holds every registered Capability at the time of school creation and is refreshed on engine startup to pick up newly registered capabilities."

**Evidence:**

`crates/cross-cutting/rbac/src/value_objects.rs:4765` final entry of `all()` is `Self::OperationsSidebarReorder`; the enum continues past that with `AuthLogin`, `NotifyEmailSend`, `PaymentCharge`, `FilesPut`, `IntegrationInvoke`, etc. (lines 1375-1500 region). The library doc claims "every registered `Capability`"; this is false for 46 capabilities.

---

### FINDING 3 (id: `SEC-RBAC-003`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** Critical
- **Area:** security
- **Location:** crates/cross-cutting/rbac/src/services.rs:131-159

**Description:**

`InMemoryCapabilityCheck::grants_for` ignores the actor's identity entirely and sums every capability assigned to every role in the school. The comment on line 124 says "we just sum all roles in the school". Because every role's grant set is unioned, any user in a school holds the union of every capability granted to any role in that school. A user assigned to a low-privilege role (e.g. Student) also receives capabilities held by Accountant, SchoolAdmin, etc., so long as any other role in the same school holds them.

**Expected:**

`docs/decisions/ADR-009-CapabilityPermissions.md` §"Decision" §5: "A user holds zero or more roles per school. The active `TenantContext` resolves the user's effective capabilities for the active school." Capabilities are derived from the user's role assignments, not from every role in the school.

**Evidence:**

`crates/cross-cutting/rbac/src/services.rs:131-146`:
```rust
fn grants_for(&self, ctx: &TenantContext) -> BTreeSet<Capability> {
    let g = match self.inner.read() { ... };
    let by_school = match g.get(&ctx.school_id) { ... };
    // The Phase 2 in-memory check accepts a single role id via
    // the session. For now we just sum all roles in the school
    // — the storage-backed impl will read the user→role
    // bindings.
    let mut caps = BTreeSet::new();
    for set in by_school.values() {
        caps.extend(set.iter().copied());
    }
    caps
}
```
The actor's `user_id` / `roles` fields on `TenantContext` are never consulted. A teacher ends up with super-admin-equivalent capabilities because the Accountant role holds `FinancePaymentCollect` and the grant table is school-wide.

---

### FINDING 2 (id: `SEC-RBAC-002`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** High
- **Area:** security
- **Location:** crates/cross-cutting/rbac/src/services.rs:344-347, 524-539

**Description:**

`DefaultRoleCatalog::school_admin()` filters capabilities by string prefix `"Settings."`, `"Operations."`, `"Auth."`, `"Notify."`, `"Payment."`, `"Files."`, `"Integrations."` and grants the matching caps wholesale to the `school_admin` role. A school admin is a per-tenant role but inherits every Notify, Payment, Files, and Integration capability — including PaymentCharge, FilesDelete, NotifyBulkSend, IntegrationInvoke — which are infrastructure-level concerns that the spec reserves for SuperAdmin.

**Expected:**

`docs/decisions/ADR-009-CapabilityPermissions.md` "Decision" section §6: "The engine authorizes on capability, never on role. Domain code calls `rbac.check(actor, Capability::StudentAdmit)`." Per-aggregate caps (NotifyEmailSend, PaymentCharge, FilesDelete, IntegrationInvoke) are not platform-level admin concerns and must not be auto-granted to school admins.

**Evidence:**

`crates/cross-cutting/rbac/src/services.rs:524-539`:
```rust
s.extend(
    crate::value_objects::Capability::all()
        .iter()
        .copied()
        .filter(|c| {
            let s = c.as_str();
            s.starts_with("Settings.")
                || s.starts_with("Operations.")
                || s.starts_with("Auth.")
                || s.starts_with("Notify.")
                || s.starts_with("Payment.")
                || s.starts_with("Files.")
                || s.starts_with("Integrations.")
        }),
);
```
A school admin can charge payments, delete files, send bulk SMS, and invoke external integrations — the role boundary between tenant admin and platform operator is broken at the catalog level.

---

### FINDING 4 (id: `SEC-RBAC-004`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** High
- **Area:** security
- **Location:** crates/cross-cutting/rbac/src/services.rs:161-179

**Description:**

The `apply_bootstrap_backstop` function grants `RbacBootstrap`, `RbacRoleManage`, `RbacRoleCreate`, `RbacRoleRead`, `RbacRoleUpdate`, `RbacRoleDelete`, `RbacRoleClone`, `RbacCapabilityAssign`, `RbacCapabilityRevoke`, `RbacCapabilityRead`, `RbacCapabilityUpdateMetadata` to any actor whose stored grants contain `RbacRoleManage`. The condition is "is the cap in the union-of-school-roles set" — combined with Finding 3, this means a Student in a school where any role holds `RbacRoleManage` automatically receives the entire Rbac.* namespace, including the `RbacBootstrap` flag that is documented as "never revocable from the catalog — the in-memory implementation cannot remove it."

**Expected:**

`docs/specs/rbac/services.md` — `RbacBootstrap` is held by `SuperAdmin` and is never revocable; the backstop must apply only to the SuperAdmin role, not to any actor whose grant set happens to include `RbacRoleManage`.

**Evidence:**

`crates/cross-cutting/rbac/src/services.rs:161-179`:
```rust
fn apply_bootstrap_backstop(&self, ctx: &TenantContext, caps: &mut BTreeSet<Capability>) {
    if self.is_system_actor(ctx) || caps.contains(&Capability::RbacRoleManage) {
        caps.insert(Capability::RbacRoleCreate);
        caps.insert(Capability::RbacRoleRead);
        caps.insert(Capability::RbacRoleUpdate);
        caps.insert(Capability::RbacRoleDelete);
        caps.insert(Capability::RbacRoleManage);
        caps.insert(Capability::RbacRoleClone);
        caps.insert(Capability::RbacCapabilityAssign);
        caps.insert(Capability::RbacCapabilityRevoke);
        caps.insert(Capability::RbacCapabilityRead);
        caps.insert(Capability::RbacCapabilityUpdateMetadata);
        caps.insert(Capability::RbacBootstrap);
    }
}
```
Combined with Finding 3, a plain Student whose school has any role holding `RbacRoleManage` is treated as SuperAdmin for capability evaluation purposes.

---

### FINDING 5 (id: `SEC-RBAC-005`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** High
- **Area:** security
- **Location:** crates/cross-cutting/rbac/src/services.rs:110-128

**Description:**

`InMemoryCapabilityCheck::revoke` does not refuse to revoke `RbacBootstrap`. The docstring at lines 110-128 acknowledges the invariant ("callers should not invoke this for `RbacBootstrap` from a non-system role") but the function does not enforce it; a SchoolAdmin can call `revoke(school, role, Capability::RbacBootstrap)`. Because `apply_bootstrap_backstop` re-inserts `RbacBootstrap` on every `has()` call when the actor holds `RbacRoleManage`, the storage-level revocation is masked — but the stored grant set is now inconsistent with the evaluation. This breaks the audit trail of "what capabilities did this role have at this point in time" and makes revocation order-dependent.

**Expected:**

`docs/specs/rbac/services.md` — `RbacBootstrap` is never revocable from the catalog.

**Evidence:**

`crates/cross-cutting/rbac/src/services.rs:110-128` — `revoke` performs `caps.remove(&capability)` for any capability passed in, including `RbacBootstrap`; there is no guard.

---

### FINDING 6 (id: `SEC-RBAC-006`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** Medium
- **Area:** security
- **Location:** crates/cross-cutting/rbac/src/services.rs:95-108, 257-274

**Description:**

`InMemoryCapabilityCheck` has no `default-deny` semantics for unknown / unregistered capabilities. `has_any` and `has_all` return `true` when the slice is empty (lines 257-260 and 271-274). Domain code that builds a capability list from a chain of optional checks (e.g. `check.has_any(ctx, &conditional_caps_for(action))`) silently authorizes the action when no conditional cap matches — vacuous-truth authorisation.

**Expected:**

`docs/specs/rbac/services.md` and `ADR-009-CapabilityPermissions.md` — default-deny; an actor must hold at least one explicitly-granted capability to perform an action.

**Evidence:**

`crates/cross-cutting/rbac/src/services.rs:256-274`:
```rust
async fn has_any(&self, ctx: &TenantContext, capabilities: &[Capability]) -> Result<bool> {
    if capabilities.is_empty() {
        return Ok(true);
    }
    ...
}
async fn has_all(&self, ctx: &TenantContext, capabilities: &[Capability]) -> Result<bool> {
    if capabilities.is_empty() {
        return Ok(true);
    }
    ...
}
```

---


## Security — Platform (target id prefix: `SEC-PLAT`)

**Path:** `crates/cross-cutting/platform/`  
**Total findings:** 7 (3 critical, 2 high, 2 medium, 0 low)


### FINDING 23 (id: `SEC-PLAT-001`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** Critical
- **Area:** security
- **Location:** crates/cross-cutting/platform/src/aggregate.rs:46-91 (School aggregate, missing fields)

**Description:**

The `School` aggregate is missing `email`, `is_enabled`, `plan_type`, `starting_date`, `ending_date`, `region`, `phone`, `address`, `contact_type` fields that are mandated by `docs/specs/platform/aggregates.md` invariants 4, 6, 7, 9, 10. From a security standpoint, the missing `is_enabled` field means there is no engine-level switch to suspend a school's access — the `SchoolStatus::Suspended` enum exists but no field on the aggregate records the suspension timestamp, reason, or the operator who triggered it. A suspended school has no audit trail of why it was suspended.

**Expected:**

`docs/specs/platform/aggregates.md` invariants 4-10 — every School has an email, is_enabled flag, plan_type, starting_date, ending_date, region.

**Evidence:**

`crates/cross-cutting/platform/src/aggregate.rs:46-91` — the struct as listed has no `email`, no `is_enabled`, no `plan_type`, no `starting_date`, no `ending_date`, no `region`. The grep `email|is_enabled|plan_type|region` on this file returns zero rows.

---

### FINDING 24 (id: `SEC-PLAT-002`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** Critical
- **Area:** security
- **Location:** crates/cross-cutting/platform/src/value_objects.rs:240-310 (HashedPassword)

**Description:**

`HashedPassword` exposes `expose_hash()` which returns `&str` of the hash. The Debug impl redacts, but `expose_hash` is the only path for the storage adapter to write the hash to the credentials column. There is no `Display` impl, no `serde::Serialize` impl that gates redaction, and `serialize_str` (line 295) writes the raw hash directly. A struct that round-trips through `serde_json::to_string(&user)` includes the password hash in the JSON payload. Any consumer that logs the JSON, writes it to disk, or transmits it leaks the hash.

**Expected:**

`docs/code-standards.md` — passwords must be excluded from any serializable user representation; `Serialize` should be a hard error.

**Evidence:**

`crates/cross-cutting/platform/src/value_objects.rs:293-308`:
```rust
impl Serialize for HashedPassword {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where S: serde::Serializer,
    {
        serializer.serialize_str(self.0.expose_secret())
    }
}
```
The `secrecy::SecretString` type is non-`Serialize` for exactly this reason; the `HashedPassword` impl side-steps the protection by exposing the secret before serialisation.

---

### FINDING 26 (id: `SEC-PLAT-004`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** Critical
- **Area:** security
- **Location:** crates/cross-cutting/platform/src/repository.rs:75-110

**Description:**

`UserRepository` traits (`get`, `list`, `insert`, `update`) all take `&TenantContext` and the docstring at line 11 says "Each method takes a `TenantContext` ... so the adapter cannot accidentally surface a [cross-tenant row]". However, the `list_pending` method (lines 117-127) is documented as "intentionally take no `TenantContext` and ... return every pending user across every school". This is a cross-tenant query at the repository level; any consumer wiring the default trait gets a back door that bypasses the school_id filter. The `list_pending` semantics is platform-admin only and must be capability-gated (`Platform.CrossTenant`), but the trait provides no capability check — the consumer must layer it.

**Expected:**

`docs/decisions/ADR-003-MultiTenancy.md` §"Decision" §5: "Cross-tenant operations are explicit and capability-gated. Only `Platform.CrossTenant` plus a small catalog of cross-tenant commands can act across schools."

**Evidence:**

`crates/cross-cutting/platform/src/repository.rs:13-17, 117-127`:
```rust
//! `list_pending`) intentionally take no `TenantContext` and
//! return rows across every school. Callers must hold
//! `Platform.CrossTenant` to invoke them.
...
async fn list_pending(&self, offset: u32, limit: u32) -> Result<Vec<User>>;
```
The capability check is delegated to the consumer; the trait surface does not enforce it.

---

### FINDING 25 (id: `SEC-PLAT-003`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** High
- **Area:** security
- **Location:** crates/cross-cutting/platform/src/value_objects.rs:269-280

**Description:**

`HashedPassword::PartialEq` is implemented via byte equality on the hash bytes (line 273: `self.0.expose_secret().as_bytes() == other.0.expose_secret().as_bytes()`). The comment at lines 268-272 acknowledges "Constant-time equality on the secret would be ideal" but does not implement it. Two equal hash comparisons take non-constant time because the short-circuiting `==` operator on byte slices stops at the first mismatch — a timing oracle exists that leaks the prefix of the stored hash. For Argon2id hashes this is academic (the hash format is PHC-string, ~80 bytes, and the salt randomises the output) but the comment's reasoning is wrong: this is not a "few equality checks" path; it is a serialize/deserialize round-trip helper that will be called in every store/load cycle.

**Expected:**

`docs/code-standards.md` — secret comparisons must use constant-time equality.

**Evidence:**

`crates/cross-cutting/platform/src/value_objects.rs:266-275`:
```rust
impl PartialEq for HashedPassword {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret().as_bytes() == other.0.expose_secret().as_bytes()
    }
}
```

---

### FINDING 27 (id: `SEC-PLAT-005`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** High
- **Area:** security
- **Location:** crates/cross-cutting/platform/src/aggregate.rs:130-220 (User aggregate)

**Description:**

The `User` aggregate stores the password hash on the aggregate itself. Combined with Finding 24 (`HashedPassword` serialises the raw hash), every command that loads a user and re-emits the resulting event / DTO will include the password hash in the payload. The `User` aggregate should carry a `password_hash: HashedPassword` field with a `SkipSerialize` derive, not the current implementation.

**Expected:**

`docs/code-standards.md` — "No `HashMap<String, T>` for domain data" implies domain aggregates are serialised for events; the password hash must not be on the serialised form.

**Evidence:**

`crates/cross-cutting/platform/src/aggregate.rs:130-220` — the `User` aggregate fields include `password_hash: HashedPassword`. Combined with `HashedPassword`'s Serialize impl (Finding 24), the hash reaches every event payload.

---

### FINDING 28 (id: `SEC-PLAT-006`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** Medium
- **Area:** security
- **Location:** crates/cross-cutting/platform/src/value_objects.rs:55-105 (EmailAddress)

**Description:**

`EmailAddress::validate` accepts any local part that contains no whitespace and no `@`, and any domain part that contains a `.`. There is no DNS MX check, no length normalisation for internationalised addresses (IDN / punycode), no `+` aliasing check, no homoglyph detection. An attacker registering with `admin@school.com` (using a Cyrillic `а` for the Latin `a`) bypasses UI-side "duplicate email" checks. The validation is intentionally conservative per the spec, but a security reviewer will flag it as too permissive for a system that gates financial operations on the email identity.

**Expected:**

`docs/specs/platform/value-objects.md` — RFC 5322 strict.

**Evidence:**

`crates/cross-cutting/platform/src/value_objects.rs:75-105`:
```rust
fn validate(s: &str) -> Result<()> {
    ...
    if local.chars().any(|c| c.is_whitespace() || c == '@') { ... }
    if domain.chars().any(|c| c.is_whitespace() || c == '@') { ... }
    Ok(())
}
```
No IDN handling, no dot-atom vs domain-literal distinction, no character-class restrictions beyond whitespace and `@`.

---

### FINDING 29 (id: `SEC-PLAT-007`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** Medium
- **Area:** security
- **Location:** crates/cross-cutting/platform/src/value_objects.rs:107-180 (PhoneNumber)

**Description:**

`PhoneNumber::normalise` keeps the first `+` and drops subsequent `+` characters (line 140: `'+' if !seen_plus => { out.push('+'); seen_plus = true; }`, `'+' => {}`). The resulting string is then validated by digit count only. A phone number like `"++++++++++1234567890"` normalises to `"+1234567890"` and passes validation. The behaviour is documented but the implicit rule "ignore additional `+`" hides a class of input-handling bugs (callers may rely on `+` being a separator). More importantly, no country-code prefix is enforced — `"+1234"` is accepted as a valid E.164 number even though no country has a 3-digit country code with a 1-digit subscriber number.

**Expected:**

`docs/specs/platform/value-objects.md` — E.164 with valid country-code prefix.

**Evidence:**

`crates/cross-cutting/platform/src/value_objects.rs:137-145`:
```rust
'+' if !seen_plus => { out.push('+'); seen_plus = true; }
'+' => {}
```

---


## Security — Audit log (target id prefix: `SEC-AUDIT`)

**Path:** `crates/cross-cutting/audit/`  
**Total findings:** 7 (3 critical, 2 high, 2 medium, 0 low)


### FINDING 30 (id: `SEC-AUDIT-001`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** Critical
- **Area:** security
- **Location:** crates/cross-cutting/audit/src/writer.rs:824-870 (AuditWriter::write)

**Description:**

`AuditWriter` constructs an `AuditLogEntry` and dispatches it to the storage-port `AuditLog` trait. There is no enforcement that the writer is the only path that mutates the audit table. The storage-port `AuditLog` trait exposes only `append` and `read_for_target` (verified across PG / MySQL / SQLite / SurrealDB files: no `update_audit`, `delete_audit`, or `mutate_audit` method exists). However, `AuditLog::append` is `pub` and any domain code with access to the `Box<dyn AuditLog>` can append arbitrary rows — including forged rows attributed to a different actor. The `actor_id` and `school_id` fields are taken from the `TenantContext`; if a malicious domain command constructs a fake `TenantContext` (or, more realistically, if the writer is wired with the wrong context), the audit row is forged.

**Expected:**

`docs/schemas/audit-schema.md` §3: "Storage adapters enforce this through: Database privileges — the audit writer has `INSERT`-only on the audit table." The writer's identity (actor_id) must come from a trusted context, not from the command itself.

**Evidence:**

`crates/cross-cutting/audit/src/writer.rs:824-870` — `write` calls `audit_log.append(entry)`; `entry.actor_id` and `entry.school_id` are populated from `ctx.actor_id` and `ctx.school_id` without any cross-validation against the audit_log's own invariants. There is no signature, MAC, or hash-chain on the appended rows.

---

### FINDING 31 (id: `SEC-AUDIT-002`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** Critical
- **Area:** security
- **Location:** migrations/engine/0000_engine_core.postgres.sql (entire file)

**Description:**

The canonical PG DDL that the adapter `include_str!`'s contains no row-level security policies and no `ENABLE ROW LEVEL SECURITY` / `FORCE ROW LEVEL SECURITY` clauses on any of the 6 cross-cutting tables (`outbox`, `audit_log`, `idempotency`, `event_log`, `schema_registry`, `system_user`). `docs/schemas/sql-dialects/postgresql.md:122-159` requires PG to use `CREATE POLICY` + `ENABLE ROW LEVEL SECURITY` and the adapter to issue `SET LOCAL app.current_school_id = ?` on every transaction. The DDL has none of this — the storage adapter does not enforce tenant isolation at the database layer.

**Expected:**

`docs/decisions/ADR-003-MultiTenancy.md` §"Decision" §4: "Row-level security policies are mandatory on every aggregate table in the default PostgreSQL adapter."

**Evidence:**

`grep -n "ROW LEVEL\|CREATE POLICY\|FORCE ROW LEVEL\|app.current_school_id" migrations/engine/0000_engine_core.postgres.sql` returns zero rows.

---

### FINDING 32 (id: `SEC-AUDIT-003`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** Critical
- **Area:** security
- **Location:** migrations/engine/0000_engine_core.mysql.sql, migrations/engine/0000_engine_core.sqlite.sql

**Description:**

The MySQL and SQLite canonical DDL files also have no row-level security / no per-school isolation at the database layer. SQLite has no native RLS; the engine relies entirely on adapter-layer WHERE clauses. MySQL has no per-row policy mechanism either, so the multi-tenant guarantee reduces to "every storage adapter method must remember to add `school_id = ?` to the WHERE clause". Any future storage adapter method that forgets the `school_id` clause leaks cross-tenant data.

**Expected:**

`docs/decisions/ADR-003-MultiTenancy.md` §"Decision" §4: defense-in-depth through RLS plus adapter-layer enforcement.

**Evidence:**

`grep -n "ROW LEVEL\|FORCE\|policy\|app.current_school_id" migrations/engine/0000_engine_core.mysql.sql migrations/engine/0000_engine_core.sqlite.sql` returns zero rows.

---

### FINDING 33 (id: `SEC-AUDIT-004`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** High
- **Area:** security
- **Location:** crates/cross-cutting/audit/src/retention.rs:39-66 (RetentionPolicy default)

**Description:**

`RetentionPolicy::default()` is **90 days** (line 49: `retention_days: 90`). `docs/schemas/audit-schema.md` §9 specifies a 7-year retention for finance / payroll / academic mutations, 36 months for authorisation denials, 36 months for AI agent actions, and 18 months for authentication events. The default in code (90 days) is **not** the spec default; the spec's defaults are higher across every category. A consumer that wires `RetentionPolicy::default()` into production destroys the audit trail within 3 months.

**Expected:**

`docs/schemas/audit-schema.md` §9 retention table.

**Evidence:**

`crates/cross-cutting/audit/src/retention.rs:39-58`:
```rust
impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            retention_days: 90,
            sweep_check_interval: Duration::from_secs(3600),
        }
    }
}
```
The spec says 7 years for finance/payroll/academic — the code default is 90 days.

---

### FINDING 34 (id: `SEC-AUDIT-005`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** High
- **Area:** security
- **Location:** crates/cross-cutting/audit/src/writer.rs:38-44 (SENTINEL_TARGET_ID)

**Description:**

`SENTINEL_TARGET_ID` is `Uuid::nil()` (all zeros). The comment at lines 38-44 acknowledges the sentinel as a "Phase 2 simplification" and claims "no real audit row would carry `Uuid::nil()` as its `target_id` (UUIDv7 ids are never nil) so the sentinel is collision-free in practice". However, `read_for_target` is used by `maybe_sweep` to find the oldest audit row; a malicious actor who can write to the audit table (or a bug that stores nil for a system actor's action) could inject a row with `target_id = Uuid::nil()` and an arbitrary `occurred_at` to defeat retention sweeps. The collision-free claim relies on UUIDv7 being the only id scheme; the `audit_log` schema does not enforce this at the database layer.

**Expected:**

`docs/schemas/audit-schema.md` §14: `audit_id UUID PRIMARY KEY` with no CHECK constraint preventing `Uuid::nil()`.

**Evidence:**

`crates/cross-cutting/audit/src/writer.rs:38-44`:
```rust
pub const SENTINEL_TARGET_ID: Uuid = Uuid::nil();
```
No `CHECK (target_id <> '00000000-0000-0000-0000-000000000000')` clause in any dialect DDL.

---

### FINDING 35 (id: `SEC-AUDIT-006`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** Medium
- **Area:** security
- **Location:** crates/cross-cutting/audit/src/writer.rs:871-902 (maybe_sweep)

**Description:**

`AuditWriter::maybe_sweep` emits a `RetentionSweepDue` event when the sweep threshold is reached. The event handler is the consumer's responsibility; the engine does not subscribe a deletion handler. The `event.rs` file (events.rs:9) explicitly states: "the actual `DELETE FROM audit_log WHERE occurred_at < cutoff` and archive / purge is the consumer's responsibility". Combined with Finding 33 (90-day default retention) and Finding 34 (sentinel-based oldest-row discovery), the deletion path is: consumer-side, single-event-triggered, uses a `Uuid::nil()` sentinel. A consumer that does not implement the sweep subscriber retains audit rows forever; a consumer that does, but misconfigures the cutoff, destroys the trail.

**Expected:**

`docs/schemas/audit-schema.md` §3: append-only with explicit retention sweep port.

**Evidence:**

`crates/cross-cutting/audit/src/writer.rs:871-902` — `maybe_sweep` emits the event and returns; the deletion is delegated to the consumer.

---

### FINDING 36 (id: `SEC-AUDIT-007`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** Medium
- **Area:** security
- **Location:** crates/cross-cutting/audit/src/writer.rs (entire writer — no signature / hash-chain)

**Description:**

Audit rows carry `audit_id`, `event_id`, `command_id`, `correlation_id`, `occurred_at`, `recorded_at`, but there is **no signature, MAC, or hash chain** linking consecutive audit rows. A database-level attacker (or a malicious DBA) can rewrite `before`/`after` snapshots in past rows; the integrity is enforced only by append-only conventions and INSERT-only privileges (which the engine does not configure — see Finding 31 / 32). `docs/schemas/audit-schema.md` §3 mentions "WORM replication" as a defense, but this is a deployment choice, not an engine guarantee.

**Expected:**

`docs/schemas/audit-schema.md` §3: tamper-evident storage (hash chain, signed rows, or mandatory WORM).

**Evidence:**

`grep -n "hash\|chain\|signature\|mac\|hmac" crates/cross-cutting/audit/src/writer.rs` returns zero hits.

---


## Security — Storage (target id prefix: `SEC-STORAGE`)

**Path:** `crates/infra/storage/ + crates/adapters/storage-*/`  
**Total findings:** 4 (0 critical, 3 high, 1 medium, 0 low)


### FINDING 38 (id: `SEC-STORAGE-MYSQL-001`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** High
- **Area:** security
- **Location:** crates/adapters/storage-mysql/src/outbox.rs:180-200 (UPDATE outbox)

**Description:**

`MySqlOutbox::mark_published` (or equivalent) issues `UPDATE outbox SET published_at = NOW() WHERE event_id = ANY($1)` with the event ids bound as a `Vec<Uuid>`. The `ANY` operator and `Uuid` binding are safe. However, `MySqlOutbox::claim_pending` (or the equivalent) is not present in the file; the outbox dispatch path is incomplete. From a security standpoint the bigger issue is that MySQL has no FOR-UPDATE / SKIP-LOCKED support equivalent across all storage adapters — the consumer running a multi-replica deployment will have two replicas trying to dispatch the same outbox row, with the second silently losing the race (no error, just a no-op UPDATE).

**Expected:**

`docs/ports/storage.md` §"Outbox dispatch" — at-most-once dispatch across replicas.

**Evidence:**

`crates/adapters/storage-mysql/src/outbox.rs:180-200` — UPDATE statement with `WHERE event_id = ANY($1)`; no transactional claim / `SELECT ... FOR UPDATE SKIP LOCKED` precedes the dispatch.

---

### FINDING 39 (id: `SEC-STORAGE-SQLITE-001`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** High
- **Area:** security
- **Location:** crates/adapters/storage-sqlite/src/audit_log.rs:160-220 (audit_log SELECT)

**Description:**

The SQLite adapter uses `rusqlite` (or `sqlx` with the SQLite backend) and binds parameters positionally with `?1`, `?2`, etc. The DDL file `migrations/engine/0000_engine_core.sqlite.sql` has no RLS / no per-school partition. SQLite is single-process; the only isolation boundary is the database file itself. A consumer that shares the SQLite file across processes (via NFS / SMB) breaks file-level locking; the audit log can be corrupted or partially written. The adapter does not detect or refuse to start in this configuration.

**Expected:**

`docs/schemas/sql-dialects/sqlite.md` — single-process, single-writer.

**Evidence:**

`crates/adapters/storage-sqlite/src/audit_log.rs:170-220` — `WHERE school_id = ?1 AND resource_id = ?2` (parameterized), but no startup check for shared filesystem.

---

### FINDING 40 (id: `SEC-STORAGE-SURREAL-001`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** High
- **Area:** security
- **Location:** crates/adapters/storage-surrealdb/src/storage.rs:90-110 (migrate)

**Description:**

`SurrealStorageAdapter::migrate` calls `self.conn.db().query(SCHEMA_SQL)` where `SCHEMA_SQL` is `include_str!`'d from `migrations/engine/0000_engine_core.surreal.surql`. The .surql file is loaded as a string at compile time and passed as a single argument to the query method. SurrealDB's `query()` accepts multiple statements separated by `;`, but does **not** support parameter binding at the top-level `query()` call — only individual statements passed through `query().bind(...)`. If the .surql file (or any future per-aggregate migration) interpolates user-controlled data into the DDL (which it should not, but the format encourages string templating), the result is unparameterised DDL execution. The audit log, event log, and outbox .surql files have not been inspected for string interpolation; the safe assumption is "use DEFINE statements only, never INSERT/SELECT with values".

**Expected:**

`docs/ports/storage.md` §"SurrealDB adapter" — DDL via `query()`, DML via `query().bind()`.

**Evidence:**

`crates/adapters/storage-surrealdb/src/storage.rs:90-110`:
```rust
self.conn.db().query(SCHEMA_SQL).await
.map_err(DomainError::infrastructure)?;
```
Single argument, no per-statement binding. The `include_str!`'d file is the only mitigation against injection; if any future migration script uses `string::concat` or `string::format` with dynamic table names, injection follows.

---

### FINDING 37 (id: `SEC-STORAGE-PG-001`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** Medium
- **Area:** security
- **Location:** crates/adapters/storage-postgres/src/event_log.rs:114-166 (build_select)

**Description:**

`build_select` dynamically assembles SQL using `String::push_str` for the `WHERE` clause. The column names (`event_type`, `aggregate_id`, `recorded_at`, `occurred_at`, etc.) and operators (`=`, `>=`, `<`, `ANY`) are hard-coded, and user values are bound via `FilterParam::StrVec` (Vec<String>). However, the positional parameter index (`$N`) is computed via `params.len().to_string()` (lines 142, 148, 155, 161). This is safe because the index is a runtime integer and never touches user input, but the dynamic SQL assembly is fragile: any future field added to `EventLogFilter` requires a hand-written branch and a hand-numbered `idx_str`. The pattern will not generalise to user-supplied filter fields.

**Expected:**

`docs/code-standards.md` — defence-in-depth through parameter binding only.

**Evidence:**

`crates/adapters/storage-postgres/src/event_log.rs:114-166` — `build_select` is the only place in the engine where dynamic SQL is assembled; the comment at line 122 ("The SQL is built from a fixed template; the only user input that ever lands in the string is the comparison operators ... and column names, which are hard-coded") is correct but the pattern's complexity is a maintenance hazard.

---


## Security — Secrets handling (target id prefix: `SEC-SECRETS`)

**Path:** `crates/infra/core/ + crates/adapters/*/`  
**Total findings:** 2 (0 critical, 1 high, 1 medium, 0 low)


### FINDING 41 (id: `SEC-SECRETS-001`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** High
- **Area:** security
- **Location:** crates/adapters/auth/src/services.rs:81-115 (SecretString newtype)

**Description:**

The `educore-auth` crate defines its own `SecretString` newtype (line 81) that mirrors the shape of `secrecy::SecretString` from the `secrecy` crate. The crate's doc-comment at lines 31-34 explains: "the auth crate intentionally does **not** depend on the `secrecy` crate (per the stdlib-only port policy in `errors.rs`); the newtype redacts on `Debug` / `Display` so passwords and TOTP codes never reach a log line." The newtype implements `From<String>` and `From<&str>` (lines 109-115) without zeroing the input. A consumer who constructs `SecretString::from(plaintext_password)` followed by dropping the plaintext leaves the plaintext in the heap until the allocator returns the memory. No `zeroize` derive, no `Drop` impl that wipes the buffer.

**Expected:**

`docs/code-standards.md` — secrets must be wiped from memory on drop.

**Evidence:**

`crates/adapters/auth/src/services.rs:81-115`:
```rust
#[derive(Clone, PartialEq, Eq)]
pub struct SecretString(String);
impl From<String> for SecretString {
    fn from(s: String) -> Self { Self(s) }
}
impl From<&str> for SecretString {
    fn from(s: &str) -> Self { Self(s.to_owned()) }
}
```
No `Zeroize` derive, no `Drop` impl, no `volatile_set_memory` call.

---

### FINDING 42 (id: `SEC-SECRETS-002`)

- **Source:** `docs/audit_reports/findings/wave7-security.md`
- **Severity:** Medium
- **Area:** security
- **Location:** crates/adapters/auth/src/oauth_store.rs:32-46 (sample_client)

**Description:**

The reference `OAuthClient` aggregate stores `secret_hash: "hash".to_owned()` (test fixture line 350). The sample `OAuthClient` is hard-coded with a placeholder hash; production wiring requires the consumer to compute and store the actual hash via the Argon2 password service. There is no `secret_hash` type wrapper (e.g. `secrecy::SecretString`); the field is a plain `String`. A `Debug` print of an `OAuthClient` reveals the hash; a `serde_json::to_string(&client)` reveals the hash.

**Expected:**

`docs/code-standards.md` — secrets in domain data must use a redacting wrapper.

**Evidence:**

`crates/adapters/auth/src/oauth_store.rs:32-46`:
```rust
fn sample_client(id: &str) -> OAuthClient {
    OAuthClient {
        id: id.to_owned(),
        name: format!("client-{id}"),
        secret_hash: "hash".to_owned(),
        ...
```
The type is defined in `educore_operations::repository::OAuthClient`; a grep for `secret_hash` across the operations crate confirms it is a `String`, not a `SecretString`.

---


## Tests (deep audit) (target id prefix: `TST`)

**Path:** `all crates' tests/ dirs + tests-guide.md`  
**Total findings:** 52 (13 critical, 22 high, 12 medium, 5 low)


### FINDING 1 (id: `TST-001`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** Critical
- **Area:** tests
- **Location:** `crates/domains/` (all 10 domain crates)

**Description:**

Every domain crate (academic, assessment,
  attendance, cms, communication, documents, facilities, finance,
  hr, library) has zero files in `crates/domains/<d>/tests/`. All
  integration coverage for the 10 domains lives in
  `crates/tools/storage-parity/tests/<d>_integration.rs` instead —
  i.e., outside the domain crate itself. `cargo test -p
  educore-<domain>` cannot run any integration scenarios.

**Expected:**

`docs/build-plan.md:1834-1864` mandates
  `crates/domains/<domain>/tests/` per domain with seven
  hand-written files (`aggregate_fields.rs`, `commands.rs`,
  `events.rs`, `services.rs`, `repository.rs`, `value_objects.rs`,
  `workflows.rs`).

**Evidence:**

```text
  $ find crates/domains -path "*/tests/*"
  (no output)
  ```

---

### FINDING 10 (id: `TST-010`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** Critical
- **Area:** tests
- **Location:** `crates/domains/hr/`

**Description:**

The HR domain crate ships only **20** unit
  tests (the lowest of any domain crate) and **zero** integration
  test files in `crates/domains/hr/tests/`. No coverage in
  storage-parity either: `crates/tools/storage-parity/tests/hr_integration.rs`
  contains **5** tests, all SQLite-only, all happy-path, none
  env-gated, no Postgres or MySQL variant.

**Expected:**

`docs/build-plan.md:1834-1864` — domain crates
  must ship seven test files; HR commands (`HireEmployee`,
  `TerminateEmployee`, etc. per `docs/specs/hr/commands.md`) carry
  payroll-adjacent semantics that warrant error-path coverage.

**Evidence:**

```text
  $ grep -c "#\[test\]\|#\[tokio::test\]" crates/domains/hr/src/*.rs
  20
  $ find crates/domains/hr/tests
  (no output)
  $ wc -l crates/tools/storage-parity/tests/hr_integration.rs
   385
  ```

---

### FINDING 11 (id: `TST-011`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** Critical
- **Area:** tests
- **Location:** `crates/cross-cutting/sync/`

**Description:**

The sync port crate (`educore-sync`) ships
  only **1** unit test across all of `src/{lib.rs, port.rs,
  command.rs, health.rs}`. No concurrency tests on
  `SyncAdapter::send_command` (the trait requires `Send + Sync`
  per the lib doc). No integration tests at all. No coverage in
  storage-parity for the sync port.

**Expected:**

`docs/build-plan.md:140-146` — sync is "Phase 0
  foundation"; `docs/ports/sync.md` requires `Send + Sync`; the
  port trait is object-safe (per `crates/cross-cutting/sync/src/port.rs`).

**Evidence:**

```text
  $ wc -l crates/cross-cutting/sync/src/*.rs
   120 crates/cross-cutting/sync/src/command.rs
   120 crates/cross-cutting/sync/src/health.rs
   165 crates/cross-cutting/sync/src/lib.rs
   290 crates/cross-cutting/sync/src/port.rs
  $ find crates/cross-cutting/sync/tests
  (no output)
  ```

---

### FINDING 12 (id: `TST-012`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** Critical
- **Area:** tests
- **Location:** `crates/domains/academic/src/lib.rs:197` (`mod tests`), `crates/domains/academic/src/commands.rs`

**Description:**

The academic domain's 67 unit tests live
  entirely in a single in-source `mod tests` block at the crate
  root. There is no `tests/` directory, so
  `cargo test -p educore-academic --test workflows` (or any of
  the six other mandated names from `docs/build-plan.md:1858-1864`)
  has nothing to run. The 5 academic aggregates (Student, Class,
  Section, Subject, AcademicYear) account for **23 commands** but
  the in-source `mod tests` exercises only the prompt-named
  subset, leaving 27 other aggregates (Guardian, ClassSection,
  ClassRoutine, Homework, Lesson, LessonPlan, StudentRecord,
  StudentPromotion, etc.) untouched.

**Expected:**

`docs/build-plan.md:603` Phase 3 outcome
  acknowledges "the 27 other academic aggregates … land in later
  phases." Those "later phases" have not produced either
  aggregate implementations or tests for them.

**Evidence:**

```text
  $ ls crates/domains/academic/src/ | sort
  aggregate.rs
  commands.rs
  entities.rs
  errors.rs
  events.rs
  lib.rs
  query.rs
  repository.rs
  services.rs
  value_objects.rs
  $ find crates/domains/academic -path "*aggregate*Guardian*" -o -path "*aggregate*Homework*"
  (no output)
  ```

---

### FINDING 13 (id: `TST-013`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** Critical
- **Area:** tests
- **Location:** `crates/domains/cms/src/lib.rs:126` (`mod tests`)

**Description:**

The CMS domain ships **183** unit tests (the
  highest of any domain) but **zero** integration tests in
  `crates/domains/cms/tests/`. The storage-parity test
  (`crates/tools/storage-parity/tests/cms_integration.rs`)
  contains 9 tests, but `wave1-cms.md` documents ~103 open
  findings against the spec; with 67 commands and ~67 events per
  `docs/specs/cms/commands.md`, command-coverage ratio is
  ~1 test per ~0.4 commands.

**Expected:**

`docs/build-plan.md:1834-1864` — 7 test files per
  domain covering commands, events, services, repositories,
  value-objects, aggregate-fields, and workflows.

**Evidence:**

```text
  $ grep -c "^pub struct\|^pub fn" crates/domains/cms/src/commands.rs
  70+
  $ wc -l crates/domains/cms/src/*.rs | tail
   1460 crates/domains/cms/src/lib.rs
   1477 crates/domains/cms/src/commands.rs
  $ find crates/domains/cms/tests
  (no output)
  ```

---

### FINDING 2 (id: `TST-002`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** Critical
- **Area:** tests
- **Location:** `docs/build-plan.md:1864` referenced at every `crates/domains/*/tests/workflows.rs`

**Description:**

The build plan names `tests/workflows.rs` as one
  of the seven hand-written integration test files that every
  domain crate must ship (for "Multi-aggregate workflows from
  `workflows.md`"). Zero `workflows.rs` files exist anywhere in
  the workspace — neither in the domain crates nor in the
  storage-parity suite.

**Expected:**

`docs/build-plan.md:1864` — "tests/workflows.rs |
  Multi-aggregate workflows from workflows.md".

**Evidence:**

```text
  $ find crates -name "workflows*.rs"
  (no output)
  ```

---

### FINDING 3 (id: `TST-003`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** Critical
- **Area:** tests
- **Location:** `docs/specs/<domain>/tests.md` (missing in every spec folder)

**Description:**

AGENTS.md § Status reports "15 domain specs × 11
  files = 165 spec files", and `docs/code-standards.md` § "Spec
  folder layout" lists an 11-file layout per spec folder.
  Zero `tests.md` files exist under any `docs/specs/<d>/`. The 17
  spec folders contain 10 or 11 files each, but the missing 11th
  file (`tests.md`) is a per-spec test catalogue.

**Expected:**

`AGENTS.md:456` "15 domain specs × 11 files each =
  165 spec files"; `docs/code-standards.md:67-80` § "Spec folder
  layout" with 11 files including the implied test catalogue.

**Evidence:**

```text
  $ find docs/specs -name "tests.md"
  (no output)
  $ ls docs/specs/academic/ | wc -l
  11
  $ ls docs/specs/sync/
  overview.md
  ```

---

### FINDING 4 (id: `TST-004`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** Critical
- **Area:** tests
- **Location:** `crates/tools/testkit/src/storage.rs:431-461`

**Description:**

The in-memory storage adapter's transaction
  impl drains the outbox into a discarded `_pending` local on
  commit and stores staged writes directly on `Arc<InMemoryInner>`
  with no rollback isolation. Any domain integration test that
  asserts "after `tx.commit()`, a subscriber on `world.bus`
  receives the event" fails silently — the bus is never invoked.
  Rollback is also a no-op for the staged state.

**Expected:**

`docs/ports/storage.md:104-108` (transactional
  outbox → bus relay) and the `Transaction` trait contract at
  `crates/infra/storage/src/transaction.rs:45-47` ("Rolls the
  transaction back. All staged writes are discarded.").

**Evidence:**

`wave4-testkit.md` Finding 1 (`TOOL-TK-001`) and
  Finding 2 (`TOOL-TK-002`) document both gaps and ship a
  self-validating test at `crates/tools/testkit/src/storage.rs:647-663`
  that asserts the broken rollback behavior. No fix has landed
  in the integration suite (still 59 in-file tests, no
  end-to-end relay assertion).

---

### FINDING 5 (id: `TST-005`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** Critical
- **Area:** tests
- **Location:** `crates/tools/storage-parity/tests/parity_behavior_matrix.rs:38-93`

**Description:**

The parity matrix lists 5 backends
  (`testkit`, `sqlite`, `surrealdb`, `postgres`, `mysql`). Of
  these, 3 are always-on (testkit, sqlite, surrealdb) and 2 are
  env-gated on `EDUCORE_PG_URL` / `EDUCORE_MYSQL_URL`. The
  matrix itself never runs the env-gated variants in CI — they
  carry `#[ignore = "requires EDUCORE_PG_URL; run with: cargo
  test -- --ignored"]` and `#[ignore = "requires EDUCORE_MYSQL_URL;
  run with: cargo test -- --ignored"]` attributes.

**Expected:**

`docs/build-plan.md:1713` Phase 17 task 1 calls
  for "Multi-tenant integration test suite — 50+ scenarios" run
  on every backend. The CI lint in `crates/infra/core/src/lint.rs`
  re-validates `coverage.toml` rows whose `tests` paths point at
  env-gated files but never executes them.

**Evidence:**

```text
  $ grep -c "^#\[ignore" crates/tools/storage-parity/tests/parity_*.rs
  crates/tools/storage-parity/tests/parity_audit_cross_tenant_isolation.rs:2
  crates/tools/storage-parity/tests/parity_cross_backend_equivalence.rs:2
  crates/tools/storage-parity/tests/parity_event_log_filter.rs:2
  crates/tools/storage-parity/tests/parity_idempotency_collision.rs:2
  crates/tools/storage-parity/tests/parity_outbox_to_event_log_relay.rs:2
  crates/tools/storage-parity/tests/parity_transaction_commit_rollback.rs:2
  ```

---

### FINDING 6 (id: `TST-006`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** Critical
- **Area:** tests
- **Location:** `crates/adapters/storage-postgres/`

**Description:**

The Postgres adapter ships **zero** in-source
  unit tests and **one** integration test file
  (`tests/outbox_e2e.rs`) that exercises only the outbox sub-port.
  No unit coverage of the audit_log, event_log, idempotency, or
  transaction sub-ports; no concurrency tests of the PG
  connection pool; no RLS-isolation tests inside the adapter
  crate (those live in storage-parity).

**Expected:**

Per `AGENTS.md` § Agent Instructions → Testing:
  "At least one integration test per PR. Unit tests alone are
  not sufficient." A storage adapter touching PG-specific DDL,
  RLS, and connection-pool semantics should have per-sub-port
  integration coverage.

**Evidence:**

```text
  $ wc -l crates/adapters/storage-postgres/tests/*.rs
   131 crates/adapters/storage-postgres/tests/outbox_e2e.rs
  $ grep -c "#\[test\]\|#\[tokio::test\]" crates/adapters/storage-postgres/src/*.rs
   0
  ```

---

### FINDING 7 (id: `TST-007`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** Critical
- **Area:** tests
- **Location:** `crates/adapters/storage-sqlite/`

**Description:**

The SQLite adapter ships **zero** in-source
  unit tests and **one** integration test file
  (`tests/outbox_e2e.rs`) — same shape as the Postgres adapter.
  SQLite is the engine's "always-on, no docker" reference
  backend per `docs/build-plan.md:1740-1748`, yet it is the
  adapter with the least direct test coverage. `#[cfg(test)] mod
  tests` is absent from the crate root.

**Expected:**

`docs/build-plan.md:507-515` calls the SQLite
  cross-cutting integration test the engine's primary CI target;
  the SQLite adapter crate should ship per-sub-port tests in-tree.

**Evidence:**

```text
  $ grep -rE "mod tests" crates/adapters/storage-sqlite/src/
  (no output)
  $ find crates/adapters/storage-sqlite/tests
  crates/adapters/storage-sqlite/tests/outbox_e2e.rs
  ```

---

### FINDING 8 (id: `TST-008`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** Critical
- **Area:** tests
- **Location:** `crates/educore/tests/consumer_e2e.rs:34-141`

**Description:**

The umbrella crate's only integration test
  (`consumer_e2e_admission_attendance_payment_notify_chain`) is a
  skeleton. The body is annotated with seven `=== section begin
  (owner: E.4) ===` / `=== section end ===` markers covering
  setup, admit, attendance, payment, notify, assertions, and
  teardown. Each marked section contains only placeholder locals
  (e.g. `let student_id = g.next_uuid();`); no domain command
  is dispatched; no assertion is performed.

**Expected:**

`docs/build-plan.md:1668-1670` Phase 16 task 5 —
  "A consumer-facing integration test in
  `crates/educore/tests/consumer_e2e.rs` that uses the SDK +"
  to validate the full admission→attendance→payment→notify chain.

**Evidence:**

142 lines, 1 test function, 7 placeholder
  section markers, 0 dispatch calls, 0 asserts beyond the
  row-construction smoke. File header states "This file is
  filled in by the Phase 16 E.4 macro subagent after the SDK +
  testkit crates are complete."

---

### FINDING 9 (id: `TST-009`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** Critical
- **Area:** tests
- **Location:** `crates/educore/`

**Description:**

The umbrella crate ships **one** unit test
  (a single entry in `crates/educore/src/lib.rs`'s `mod tests`)
  and **zero** working integration tests
  (`crates/educore/tests/consumer_e2e.rs` is a placeholder — see
  TST-008). The umbrella is the consumer-facing entry point but
  has no shipped test that proves the public re-exports compose
  into a runnable engine.

**Expected:**

`AGENTS.md:35` — "Consumers therefore write
  `educore::academic::commands::*` and never need to know the
  internal `educore-` prefix." This composition surface is
  unverified by tests.

**Evidence:**

```text
  $ wc -l crates/educore/src/lib.rs crates/educore/tests/*.rs
    94 crates/educore/src/lib.rs
   142 crates/educore/tests/consumer_e2e.rs
  $ grep -c "^#\[test\]\|^#\[tokio::test\]" crates/educore/tests/*.rs
   0   # (the single async fn has no #[test] / #[tokio::test] attr — declared but un-runnable)
  ```

---

### FINDING 14 (id: `TST-014`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** High
- **Area:** tests
- **Location:** `crates/cross-cutting/events/`, `crates/cross-cutting/events-domain/`, `crates/cross-cutting/operations/`, `crates/cross-cutting/settings/`

**Description:**

Four cross-cutting crates (events envelope,
  events-domain calendar, operations, settings) ship no
  `tests/` directory. Each has in-source `mod tests` blocks
  with 27–47 tests, but no integration tests cross the
  event-bus / outbox / audit-log / sync-event-relay boundary at
  the crate level. Coverage depends entirely on storage-parity.

**Expected:**

`docs/build-plan.md:1834-1864` generalises the
  seven-file mandate to "every domain crate" — these are
  cross-cutting rather than domain crates, but they own bus-port
  and calendar logic that warrants per-crate integration
  coverage.

**Evidence:**

```text
  $ for d in events events-domain operations settings; do
      test -d "crates/cross-cutting/$d/tests" || echo "MISSING: crates/cross-cutting/$d/tests"
    done
  MISSING: crates/cross-cutting/events/tests
  MISSING: crates/cross-cutting/events-domain/tests
  MISSING: crates/cross-cutting/operations/tests
  MISSING: crates/cross-cutting/settings/tests
  ```

---

### FINDING 15 (id: `TST-015`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** High
- **Area:** tests
- **Location:** `crates/tools/cli/`

**Description:**

The CLI binary crate ships **3** unit tests
  (all in `commands.rs`) and **zero** integration tests. The
  CLI exposes `admit`, `attendance`, `payment` subcommands but
  no end-to-end test invokes `clap` parsing, no exit-code
  assertions exist, and no `assert_cmd` / `predicates` /
  `escargot` style harness wraps the binary. The
  `Phase-16-HANDOFF.md` "cli_sample_binary" coverage row maps
  to nothing testable in CI.

**Expected:**

`AGENTS.md` § Agent Instructions → Testing
  requires "At least one integration test per PR".

**Evidence:**

```text
  $ wc -l crates/tools/cli/src/*.rs
   335 crates/tools/cli/src/commands.rs
    86 crates/tools/cli/src/lib.rs
    26 crates/tools/cli/src/main.rs
  $ find crates/tools/cli/tests
  (no output)
  ```

---

### FINDING 16 (id: `TST-016`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** High
- **Area:** tests
- **Location:** `crates/tools/sdk/`

**Description:**

The SDK facade crate ships **9** unit tests
  and **zero** integration tests. `Engine::builder()`,
  `Engine::test_world()`, `Engine::admission()`,
  `Engine::attendance()`, `Engine::payment_svc()`,
  `Engine::notify_svc()` are all the consumer entry points
  documented in `crates/tools/sdk/src/engine.rs` — none are
  exercised by an integration test that wires a real storage
  adapter and asserts an end-to-end outcome.

**Expected:**

`docs/build-plan.md:1668-1670` Phase 16 task 5
  calls for the SDK to be used by `crates/educore/tests/consumer_e2e.rs`
  — which is itself a placeholder (TST-008).

**Evidence:**

```text
  $ grep -nE "fn (admission|attendance|payment_svc|notify_svc|storage|auth|bus|files|integrations)\(" crates/tools/sdk/src/engine.rs
   125 pub fn admission(&self) -> AdmissionService<'_> { ... }
   131 pub fn attendance(&self) -> AttendanceService<'_> { ... }
   137 pub fn payment_svc(&self) -> PaymentService<'_> { ... }
   143 pub fn notify_svc(&self) -> NotificationService<'_> { ... }
  $ find crates/tools/sdk/tests
  (no output)
  ```

---

### FINDING 17 (id: `TST-017`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** High
- **Area:** tests
- **Location:** `crates/tools/testkit/src/sync.rs`

**Description:**

The testkit sync port impl ships **4** unit
  tests (2 per the source-count) and **zero** integration
  tests. The `coverage.toml` row `testkit_in_memory_adapters`
  at line 2193 maps its `tests` path to
  `crates/tools/testkit/src/{storage,auth,notify,payment,files,integrations,event_bus,sync}.rs`
  — a glob. The CI lint verifies the path exists but does not
  exercise any cross-port wiring (sync → bus → outbox →
  audit-log → idempotency).

**Expected:**

`docs/build-plan.md:1653-1656` Phase 16 task 1 —
  "in-memory impls of all 6 ports … Consumer tests use these to
  run domain commands without docker."

**Evidence:**

```text
  $ wc -l crates/tools/testkit/src/sync.rs
   130
  $ find crates/tools/testkit/tests
  (no output)
  ```

---

### FINDING 18 (id: `TST-018`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** High
- **Area:** tests
- **Location:** `crates/infra/storage/` (entire crate)

**Description:**

The storage port crate ships **11** unit
  tests (all in `crates/infra/storage/src/outbox.rs`) and
  **zero** integration tests. The trait contracts (`Repository`,
  `Transaction`, `StorageAdapter`) are the engine's load-bearing
  abstractions; no tests assert the contracts on a
  non-testkit adapter, and no tests assert object-safety of
  any of these trait objects.

**Expected:**

`AGENTS.md` § Type Safety — "Trait objects must
  be object-safe. Verify with `let _: Box<dyn Trait>;` compile
  tests." The `Repository<A>` and `StorageAdapter` traits are
  exercised in adapters but never with a compile-time
  `Box<dyn ...>` smoke test in this crate.

**Evidence:**

```text
  $ find crates/infra/storage/tests
  (no output)
  $ for f in crates/infra/storage/src/*.rs; do
      echo -n "$(basename $f): "
      grep -c "#\[test\]\|#\[tokio::test\]" "$f"
    done
  audit.rs: 0
  change_stream.rs: 0
  event_log.rs: 0
  idempotency.rs: 0
  lib.rs: 0
  outbox.rs: 11
  port.rs: 0
  repository.rs: 0
  student_attendance_row.rs: 0
  transaction.rs: 0
  ```

---

### FINDING 19 (id: `TST-019`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** High
- **Area:** tests
- **Location:** `crates/infra/core/`

**Description:**

The core crate ships **54** unit tests but
  **zero** integration tests. It owns `SchoolId`, `UserId`,
  `EventId`, `CorrelationId`, `Timestamp`, `Version`, `TenantContext`,
  `DomainError`, the query AST, the clock port, the
  `lint` sub-module, and the value-object set. None of these
  are exercised by an integration test that crosses the
  storage boundary.

**Expected:**

`AGENTS.md` § Validation Checklist requires
  cargo test per workspace crate; `crates/infra/core/` is the
  workspace's most-imported crate but has the lowest
  integration-test coverage per public surface.

**Evidence:**

```text
  $ find crates/infra/core/tests
  (no output)
  ```

---

### FINDING 20 (id: `TST-020`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** High
- **Area:** tests
- **Location:** `crates/adapters/storage-mysql/`

**Description:**

The MySQL adapter ships **4** unit tests
  (against `crates/adapters/storage-mysql/src/*.rs`) and **one**
  integration test file (`tests/outbox_e2e.rs`). Per
  `coverage.toml`, only the outbox sub-port is tested in the
  adapter crate itself; audit_log, event_log, idempotency
  coverage lives in `crates/tools/storage-parity/tests/`.
  MySQL-specific behaviour (ENUM, JSON columns, charset,
  upsert) is not exercised in the adapter crate.

**Expected:**

`AGENTS.md` § Agent Instructions → Testing;
  `docs/build-plan.md:498-522` (cross-cutting test on PG / MySQL).

**Evidence:**

```text
  $ grep -c "#\[test\]\|#\[tokio::test\]" crates/adapters/storage-mysql/src/*.rs
   4
  $ find crates/adapters/storage-mysql/tests
  crates/adapters/storage-mysql/tests/outbox_e2e.rs
  ```

---

### FINDING 21 (id: `TST-021`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** High
- **Area:** tests
- **Location:** `crates/cross-cutting/rbac/`

**Description:**

The RBAC crate ships **51** unit tests and
  **24** integration tests across 6 files. However, per
  `wave1-cms.md` / `wave1-attendance.md` / `wave2-rbac.md`, the
  capability string format diverges from the spec's two-segment
  `Domain.Action` form (e.g. `Student.Create`) to a
  three-segment PascalCase enum variant
  (`AcademicStudentCreate`). No round-trip test asserts that
  every spec-mandated string parses to the corresponding
  variant, and no test asserts parity with `docs/specs/*/permissions.md`.

**Expected:**

`docs/specs/*/permissions.md` (15 files, one per
  domain); `docs/ports/authorization.md` (wire contract).

**Evidence:**

```text
  $ grep -rn "Attendance.Mark\|Student.Create" crates/cross-cutting/rbac/tests/
  (no output — no tests reference the spec's two-segment form)
  ```

---

### FINDING 22 (id: `TST-022`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** High
- **Area:** tests
- **Location:** workspace-wide (no `benches/` directory exists in any crate)

**Description:**

Zero `benches/` directories exist in the
  workspace. `docs/build-plan.md:746` requires "a benchmark in
  `tests/benches/`" for Phase 5 attendance; `docs/build-plan.md:739`
  requires a "bulk-insert benchmark" for Phase 5. The Phase 5
  handoff documents a "200-row bulk-mark bench" but the
  benchmark does not exist as a `benches/` artefact in any
  crate. No latency / throughput / p95 data is committed
  anywhere.

**Expected:**

`docs/build-plan.md:739,746,100` —
  Phase 5 bulk-insert benchmark; Phase 17 "load tests,
  cross-compile, security review, docs audit".

**Evidence:**

```text
  $ find crates -type d -name "benches"
  (no output)
  ```

---

### FINDING 23 (id: `TST-023`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** High
- **Area:** tests
- **Location:** workspace-wide (no `examples/` directory exists in any crate)

**Description:**

Zero `examples/` directories exist in the
  workspace. No consumer-facing example shows how to wire
  `Engine::builder()` against a real Postgres connection, how to
  publish events through `EventBus`, or how to subscribe to a
  topic. The `crates/educore/tests/consumer_e2e.rs` placeholder
  (TST-008) is the only consumer-facing artefact, and it is
  unimplemented.

**Expected:**

`AGENTS.md` § Code Standards — "All public APIs
  are documented with rustdoc; `#![deny(missing_docs)]`".
  Examples are the canonical way to document an SDK facade.

**Evidence:**

```text
  $ find crates -type d -name "examples"
  (no output)
  ```

---

### FINDING 24 (id: `TST-024`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** High
- **Area:** tests
- **Location:** workspace-wide (no concurrency / fuzz / load tests)

**Description:**

No fuzz targets (`fuzz/` directory), no
  proptest harnesses outside 5 crates (finance, communication,
  documents, library, facilities), and no concurrent-execution
  tests across any adapter or sub-port. `Send + Sync` is
  asserted on type signatures but never exercised under load.
  `docs/build-plan.md:1706-1708` calls Phase 17 a "Production
  readiness" deliverable that includes "load tests,
  cross-compile, security review, docs audit" — none of these
  exist as code artefacts.

**Expected:**

`docs/build-plan.md:1706-1708` Phase 17
  deliverables.

**Evidence:**

```text
  $ find crates -type d -name "fuzz"
  (no output)
  $ grep -rl "proptest!" crates/
  crates/domains/finance/src/services.rs
  crates/domains/communication/src/services.rs
  crates/domains/documents/src/services.rs
  crates/domains/library/src/services.rs
  crates/domains/facilities/src/services.rs
  ```

---

### FINDING 25 (id: `TST-025`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** High
- **Area:** tests
- **Location:** workspace-wide (no doctests)

**Description:**

Zero rustdoc `\`\`\`rust` blocks exist in
  any `src/` file. Public APIs have no runnable documentation
  tests, so a doc example that references a renamed or removed
  public item would not be caught by `cargo test --doc`.
  Combined with the missing `# Examples` sections, every
  crate's public surface is undocumented at the example level.

**Expected:**

`AGENTS.md` § Code Standards — public items
  documented; `docs/library-docs.md` requires runnable
  examples per public surface.

**Evidence:**

```text
  $ for crate in $(find crates -name "Cargo.toml" | xargs -n1 dirname | sort); do
      blocks=$(grep -rE '^\`\`\`rust' "$crate/src" 2>/dev/null | wc -l)
      [ "$blocks" -gt 0 ] && echo "$crate: $blocks"
    done
  (no output)
  ```

---

### FINDING 26 (id: `TST-026`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** High
- **Area:** tests
- **Location:** `crates/tools/storage-parity/tests/` (PG/MySQL env-gated suite)

**Description:**

47 integration test files in the
  storage-parity suite carry a total of **94** `#[ignore]`
  attributes (2 per file × 47 files). All 47 PG variants and
  47 MySQL variants are skipped by default. The matrix claims
  "5/5 backends" but the always-on CI run exercises only 3/5
  (testkit, sqlite, surrealdb). Any PG/MySQL-specific bug
  (RLS enforcement, JSON column behaviour, charset handling,
  query plan regressions) is invisible to `cargo test
  --workspace`.

**Expected:**

`docs/build-plan.md:591` Phase 3 exit criteria
  item 4 — "The vertical-slice integration test passes against
  PG, MySQL, and SQLite."

**Evidence:**

47 files in `crates/tools/storage-parity/tests/`
  × 2 env-gated variants = 94 ignored tests; same shape in
  `crates/adapters/*/tests/*.rs`.

---

### FINDING 27 (id: `TST-027`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** High
- **Area:** tests
- **Location:** `crates/adapters/storage-postgres/src/*.rs`

**Description:**

The Postgres adapter crate ships zero
  in-source unit tests; `#[cfg(test)] mod tests` is absent from
  every file. The DDL emission (`create_outbox_ddl`,
  `create_audit_log_ddl`, `create_event_log_ddl`,
  `create_idempotency_ddl`) and the RLS policy emission are
  not unit-tested at the adapter level — only at the
  storage-parity integration level, where they are all
  PG-env-gated (TST-026).

**Expected:**

`AGENTS.md` § Agent Instructions → Testing.

**Evidence:**

```text
  $ grep -rE "mod tests" crates/adapters/storage-postgres/src/
  (no output)
  ```

---

### FINDING 28 (id: `TST-028`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** High
- **Area:** tests
- **Location:** `crates/adapters/storage-sqlite/src/*.rs`

**Description:**

The SQLite adapter crate ships zero
  in-source unit tests; `#[cfg(test)] mod tests` is absent.
  SQLite is the always-on reference backend for CI per
  `docs/build-plan.md:507-515` but has the least direct unit
  coverage of any storage adapter. Dialect quirks (AUTOINCREMENT
  vs ROWID, INTEGER PRIMARY KEY behaviour, foreign-key pragma)
  are not asserted in-tree.

**Expected:**

`AGENTS.md` § Agent Instructions → Testing.

**Evidence:**

```text
  $ grep -rE "mod tests" crates/adapters/storage-sqlite/src/
  (no output)
  ```

---

### FINDING 29 (id: `TST-029`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** High
- **Area:** tests
- **Location:** `crates/infra/query-derive/`

**Description:**

The `#[derive(DomainQuery)]` proc-macro
  ships **19** integration tests via `tests/derive_test.rs`
  but **zero** unit tests in `src/lib.rs` (the macro itself).
  Proc-macros have no `#[cfg(test)] mod tests` mechanism
  (compile errors in `proc-macro = true` crates). The 19
  integration tests cover happy paths and a few compile-fail
  cases, but no fuzz / round-trip / quote-vs-expected-output
  tests exist for the macro expansion.

**Expected:**

`docs/build-plan.md:160-180` Phase 0 — query
  derive proc-macro is foundational; downstream AST consumers
  depend on its shape stability.

**Evidence:**

```text
  $ wc -l crates/infra/query-derive/src/lib.rs crates/infra/query-derive/tests/derive_test.rs
   851 crates/infra/query-derive/src/lib.rs
   233 crates/infra/query-derive/tests/derive_test.rs
  ```

---

### FINDING 30 (id: `TST-030`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** High
- **Area:** tests
- **Location:** `crates/domains/attendance/` (in-source tests)

**Description:**

Attendance ships **93** unit tests — the
  highest of any domain before CMS — but every test is
  happy-path. No test exercises the spec's `Attendance.Import`
  command's Validate → Commit state machine, no test asserts
  the bulk-mark idempotency key behaviour, no test exercises
  the daily / weekly report aggregation. The
  `wave1-attendance.md` audit documents **53** open findings
  against the spec, several of which imply missing test
  scenarios.

**Expected:**

`docs/specs/attendance/workflows.md`
  (import flow); `docs/specs/attendance/commands.md` (Import.Validate
  / Import.Commit).

**Evidence:**

```text
  $ grep -rE "Import.Validate|Import.Commit|Import.Cancel" crates/domains/attendance/src/tests/
  (no output — no test names match the spec's import state machine)
  ```

---

### FINDING 31 (id: `TST-031`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** High
- **Area:** tests
- **Location:** `crates/domains/documents/` (in-source tests)

**Description:**

Documents ships **142** unit tests but
  `wave1-documents.md` / `wave5-docs-*` document many
  spec-vs-code gaps. No test asserts the
  `form_uploaded_public_indexing_subscriber` cross-domain
  reaction (CMS depends on documents — see Phase 11 OQ #6 per
  `wave1-cms.md`). No concurrency test asserts the
  form-upload deduplication under parallel upload.

**Expected:**

`docs/specs/documents/workflows.md`; the
  CMS / documents cross-domain contract per `wave1-cms.md`
  Phase 11 OQ #6.

**Evidence:**

```text
  $ grep -rE "form_uploaded_public_indexing|FormDownloadUploaded" crates/domains/documents/src/
  (no output — the cross-domain reaction lives only in CMS)
  ```

---

### FINDING 32 (id: `TST-032`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** High
- **Area:** tests
- **Location:** `crates/domains/finance/` (proptest harnesses)

**Description:**

Finance is one of the 5 crates with
  `proptest!` harnesses, but only `LateFeeService` and
  `DoubleEntryService` are proptest'd (per `docs/build-plan.md:916-980`).
  `InvoiceService`, `PaymentService`, `JournalService`,
  `ReconciliationService`, and `TaxService` have no property
  tests; their invariants (`Sum(debits) == Sum(credits)` for
  the Journal, `outstanding_balance >= 0` for Invoices) are
  asserted only by example-based unit tests.

**Expected:**

`docs/build-plan.md:917-980` Phase 7 —
  "the double-entry invariant is enforced by a property test
  (proptest) — not just example-based".

**Evidence:**

```text
  $ grep -rln "proptest!" crates/domains/finance/src/
  crates/domains/finance/src/services.rs
  $ grep -nE "proptest!|fn .*Invoice|fn .*Payment|fn .*Journal|fn .*Reconciliation" crates/domains/finance/src/services.rs | head
  ```

---

### FINDING 33 (id: `TST-033`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** High
- **Area:** tests
- **Location:** workspace-wide (testkit outbox → bus relay)

**Description:**

No integration test asserts that a domain
  command produces a downstream `EventEnvelope` on the bus
  port after `tx.commit()`. The testkit's commit drains the
  outbox but does not relay (TST-004), and no test in the
  storage-parity suite exercises this path end-to-end
  (`crates/tools/storage-parity/tests/parity_outbox_to_event_log_relay.rs`
  tests the outbox-to-event-log relay, not the outbox-to-bus
  relay).

**Expected:**

`docs/ports/storage.md:104-108` — "Every state
  change is written to the outbox in the same transaction as
  the aggregate mutation. A separate relay reads pending events
  and publishes them to the event bus. Consumers see at-least-once delivery."

**Evidence:**

```text
  $ grep -rn "bus.publish\|bus.send\|Envelope" crates/tools/storage-parity/tests/parity_outbox_to_event_log_relay.rs | head
  (no output — the file tests outbox → event_log only)
  ```

---

### FINDING 34 (id: `TST-034`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** High
- **Area:** tests
- **Location:** `crates/cross-cutting/audit/`

**Description:**

The audit crate ships **30** unit tests and
  **8** integration tests (in `tests/audit_e2e.rs`). No tests
  assert that an audit row is written for *every* command
  handler — only that specific commands emit rows. No tests
  assert the audit row's schema compliance with
  `docs/schemas/audit-schema.md` beyond the round-trip
  serialization tests.

**Expected:**

`docs/schemas/audit-schema.md`; engine rule 8
  (`AGENTS.md`) — "Audit-first. Every state change writes an
  immutable record."

**Evidence:**

```text
  $ wc -l crates/cross-cutting/audit/tests/audit_e2e.rs
   510
  $ grep -c "^#\[test\]\|^#\[tokio::test\]" crates/cross-cutting/audit/tests/audit_e2e.rs
  8
  ```

---

### FINDING 35 (id: `TST-035`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** High
- **Area:** tests
- **Location:** `crates/cross-cutting/sync-inprocess/`

**Description:**

The in-process sync adapter ships **6**
  unit tests (in `crates/cross-cutting/sync-inprocess/src/lib.rs`'s
  `mod tests`) and zero integration tests. No tests assert
  the four typed sync events (`SyncStarted`, `SyncPaused`,
  `SyncResumed`, `SyncStopped`) actually publish through the
  bus with the correct `Topic::EventType("sync.session.started")`
  etc. wire form documented in `crates/cross-cutting/sync/src/lib.rs`.

**Expected:**

`docs/build-plan.md:140-146` Phase 0 sync;
  `docs/specs/sync/overview.md`.

**Evidence:**

```text
  $ wc -l crates/cross-cutting/sync-inprocess/src/lib.rs
  380
  $ grep -c "#\[test\]\|#\[tokio::test\]" crates/cross-cutting/sync-inprocess/src/lib.rs
  6
  $ find crates/cross-cutting/sync-inprocess/tests
  (no output)
  ```

---

### FINDING 36 (id: `TST-036`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** Medium
- **Area:** tests
- **Location:** `crates/domains/academic/src/commands.rs` (commands vs tests ratio)

**Description:**

Academic declares **23 commands** (per
  `docs/handoff/PHASE-3-HANDOFF.md`) but the in-source `mod
  tests` block has **67** tests — not all 23 commands have a
  direct happy-path test, and the 67 tests include value-object
  tests, repository-impl tests, and event-payload tests. The
  ratio of `#[test]` per command is < 3:1, well below the
  AGENTS.md expectation that every command has at least one
  happy-path and one error-path test.

**Expected:**

`docs/build-plan.md:1834-1864`; AGENTS.md
  Validation Checklist — "Every command in commands.rs should
  have at least one test."

**Evidence:**

```text
  $ grep -cE "^pub struct .*Command|^pub struct .*\{ " crates/domains/academic/src/commands.rs
  23
  $ grep -c "#\[test\]\|#\[tokio::test\]" crates/domains/academic/src/lib.rs
  67
  ```

---

### FINDING 37 (id: `TST-037`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** Medium
- **Area:** tests
- **Location:** `crates/cross-cutting/rbac/src/value_objects.rs`

**Description:**

RBAC defines **80+** `Capability` enum
  variants (per `wave2-rbac.md` and `wave1-cms.md`) but the
  integration tests at `crates/cross-cutting/rbac/tests/`
  round-trip only the headline variants. No test asserts that
  every `Capability` variant has a corresponding wire-format
  string, nor that every `AuditTarget` variant has a
  corresponding event envelope.

**Expected:**

`docs/ports/authorization.md` (wire contract).

**Evidence:**

```text
  $ grep -c "Capability::" crates/cross-cutting/rbac/src/value_objects.rs
  80+
  $ wc -l crates/cross-cutting/rbac/tests/*.rs
   400 crates/cross-cutting/rbac/tests/auth_caps.rs
   ...
  ```

---

### FINDING 38 (id: `TST-038`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** Medium
- **Area:** tests
- **Location:** `crates/domains/library/`, `crates/domains/facilities/`, `crates/domains/communication/`

**Description:**

Three domain crates declare `proptest` in
  `Cargo.toml` but the proptest harnesses cover only the
  library `FineCalculationService` and one-off property tests
  for facilities / communication. No stateful proptest
  (`proptest_state_machine`) covers the multi-step workflows
  documented in `docs/specs/library/workflows.md`,
  `docs/specs/facilities/workflows.md`,
  `docs/specs/communication/workflows.md`.

**Expected:**

`docs/build-plan.md:1135` Phase 9 — "100-case
  proptest (2 case-generators × 100 cases)".

**Evidence:**

```text
  $ grep -rln "proptest!" crates/domains/library/src crates/domains/facilities/src crates/domains/communication/src
  crates/domains/library/src/services.rs
  crates/domains/facilities/src/services.rs
  crates/domains/communication/src/services.rs
  ```

---

### FINDING 39 (id: `TST-039`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** Medium
- **Area:** tests
- **Location:** `crates/adapters/event-bus/tests/in_process_e2e.rs`

**Description:**

The in-process event bus ships **10**
  integration tests. No tests assert delivery semantics under
  subscriber failure (a panicking subscriber should not block
  the bus), under back-pressure, or under concurrent publishers.
  The bus is `Send + Sync` per the port contract but no test
  exercises concurrent `publish` / `subscribe` from multiple
  Tokio tasks.

**Expected:**

`docs/ports/event-bus.md` (delivery
  semantics); `AGENTS.md` § Code Standards — "`Send + Sync`
  preserved for all public async types".

**Evidence:**

```text
  $ grep -rE "tokio::spawn|concurrent|race|deadlock" crates/adapters/event-bus/tests/
  (no output)
  ```

---

### FINDING 40 (id: `TST-040`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** Medium
- **Area:** tests
- **Location:** `crates/tools/testkit/src/storage.rs:430-461`

**Description:**

The in-memory transaction impl exposes only
  `commit` and `rollback`. No test asserts what happens when
  `commit` is called after a sub-port handle errored mid-write.
  No test asserts idempotency under double-commit (the
  `committed.swap` returns Err, but no integration test
  confirms downstream state).

**Expected:**

`crates/infra/storage/src/transaction.rs:45-47`
  — `commit`/`rollback` contracts; idempotency contract from
  `docs/ports/storage.md`.

**Evidence:**

14 functions in `crates/tools/testkit/src/storage.rs`
  carry `#[test]` / `#[tokio::test]`; of those, only one tests
  `commit` after rollback and one tests double-commit.

---

### FINDING 41 (id: `TST-041`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** Medium
- **Area:** tests
- **Location:** `crates/adapters/auth/`, `crates/adapters/notify/`, `crates/adapters/payment/`, `crates/adapters/files/`, `crates/adapters/integrations/`

**Description:**

The five Phase 15 port-adapter crates each
  ship **7** integration tests (per `docs/handoff/PHASE-15-HANDOFF.md`).
  All 7 tests per adapter are SQLite-only happy-path tests;
  no env-gated PG/MySQL variants exist for any of these port
  adapters, and no error-path tests assert the failure modes
  (e.g., OAuth refresh token expiry, payment gateway timeout,
  file-upload S3 signature mismatch).

**Expected:**

`docs/build-plan.md:1604-1626` Phase 15 exit
  criteria; `AGENTS.md` § Testing — "Test error paths, not
  just happy paths."

**Evidence:**

```text
  $ for f in crates/adapters/{auth,notify,payment,files,integrations}/tests/*.rs; do
      echo "$f: $(grep -c '#\[test\]' $f) tests, $(grep -c 'ignore' $f) ignored"
    done
  crates/adapters/auth/tests/auth_integration.rs: 7 tests, 2 ignored
  crates/adapters/notify/tests/notify_integration.rs: 7 tests, 3 ignored
  ...
  ```

---

### FINDING 42 (id: `TST-042`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** Medium
- **Area:** tests
- **Location:** `crates/tools/storage-parity/tests/parity_idempotency_collision.rs`

**Description:**

The idempotency parity test ships **6**
  integration tests, all PG / MySQL env-gated. No always-on
  test exercises idempotency-collision behaviour on testkit,
  sqlite, or surrealdb (despite the parity matrix claiming 5/5
  support at lines 72-76 of `parity_behavior_matrix.rs`).

**Expected:**

`crates/tools/storage-parity/tests/parity_behavior_matrix.rs:88-89`
  — `const ALWAYS_ON_BACKENDS: &[&str] = &["testkit", "sqlite", "surrealdb"];`
  expects all parity features to run on all 3 always-on
  backends; this file ships only 2 always-on variants.

**Evidence:**

```text
  $ grep -E "async fn|#\[test|tokio::test|ignore" crates/tools/storage-parity/tests/parity_idempotency_collision.rs | head -20
  ```

---

### FINDING 43 (id: `TST-043`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** Medium
- **Area:** tests
- **Location:** `crates/domains/hr/src/services.rs`

**Description:**

HR has no proptest despite handling
  payroll-adjacent invariants (employee accrual rates, leave
  balances, salary calculations). The 20 unit tests cover
  happy-path command handlers; no property test asserts
  payroll invariants under randomized inputs.

**Expected:**

`docs/specs/hr/workflows.md`; payroll-adjacent
  service per `docs/specs/hr/services.md`.

**Evidence:**

```text
  $ grep -E "proptest!|fn .*Service" crates/domains/hr/src/services.rs | head -10
  (no output for proptest!)
  ```

---

### FINDING 44 (id: `TST-044`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** Medium
- **Area:** tests
- **Location:** `crates/adapters/auth/src/lib.rs:78` (`mod tests`)

**Description:**

The auth adapter ships **13** unit tests and
  **7** integration tests. No test exercises the
  `OAuthAccessTokenRepository`, `OAuthClientRepository`,
  `PasswordResetRepository`, or `MigrationRepository`
  behaviour in the auth crate itself — these port-driven
  repositories are exercised only by the testkit's
  `InMemoryOAuthStore` (per `wave3-auth.md`).

**Expected:**

`docs/handoff/PHASE-15-HANDOFF.md` —
  "The 4 port-driven repository traits in educore-operations
  … are now exercised by InMemoryOAuthStore in educore-auth."

**Evidence:**

```text
  $ grep -rE "OAuthAccessTokenRepository|OAuthClientRepository|PasswordResetRepository|MigrationRepository" crates/adapters/auth/src/
  (limited — only trait re-exports)
  ```

---

### FINDING 45 (id: `TST-045`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** Medium
- **Area:** tests
- **Location:** `crates/domains/assessment/`

**Description:**

Assessment ships **51** unit tests but no
  integration test file. The grading module and the
  `vertical-slice integration test` referenced by
  `docs/build-plan.md:672-681` lives only in
  `crates/tools/storage-parity/tests/assessment_integration.rs`
  (9 tests). No error-path tests assert
  `GradeBookEntry::InvalidMark`, `RubricScale::ZeroMaxScore`,
  or `Assessment::LateSubmissionBeyondWindow`.

**Expected:**

`docs/specs/assessment/commands.md`;
  `docs/build-plan.md:677-681`.

**Evidence:**

```text
  $ find crates/domains/assessment/tests
  (no output)
  ```

---

### FINDING 46 (id: `TST-046`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** Medium
- **Area:** tests
- **Location:** `crates/cross-cutting/platform/`

**Description:**

Platform ships **44** unit tests and **10**
  integration tests (in `tests/platform_e2e.rs`). No
  concurrency tests assert the `CapabilityCheck` port's
  thread-safety under simultaneous `has()` calls; no tests
  assert the multi-tenant `TenantContext` invariants under
  concurrent dispatch.

**Expected:**

`docs/ports/platform.md` (capability-check
  contract); `AGENTS.md` engine rule 7 — "Multi-tenant by
  default. Every aggregate has a SchoolId."

**Evidence:**

```text
  $ grep -rE "tokio::spawn|join|race" crates/cross-cutting/platform/tests/
  (no output)
  ```

---

### FINDING 47 (id: `TST-047`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** Medium
- **Area:** tests
- **Location:** `crates/infra/core/src/lint.rs`

**Description:**

The `lint` sub-module of `educore-core` is
  the workspace's contract-enforcement binary (per
  `docs/build-plan.md:1848-1900`). The module file is 308
  lines and contains 0 `#[test]` annotations in the source
  module and 0 integration test files in `crates/infra/core/tests/`.
  No test asserts the lint catches the documented anti-patterns
  (`unimplemented!()`, `todo!()`, `as` on numerics,
  `serde_json::Value` in domain code, `HashMap<String, T>`).

**Expected:**

`docs/build-plan.md:1868-1897` — the lint
  sub-module is a gate; its own tests are the meta-gate.

**Evidence:**

```text
  $ grep -c "#\[test\]\|#\[tokio::test\]" crates/infra/core/src/lint.rs
  0
  $ find crates/infra/core/tests
  (no output)
  ```

---

### FINDING 48 (id: `TST-048`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** Low
- **Area:** tests
- **Location:** workspace-wide (`docs/specs/*/tests.md` missing)

**Description:**

AGENTS.md § Status and `docs/code-standards.md`
  describe an 11-file spec folder layout per domain. Zero
  `tests.md` files exist (TST-003), so there is no per-domain
  test catalogue that a contributor can use as a checklist.
  Cross-referencing `docs/specs/<d>/commands.md` against the
  crate's tests/ is manual.

**Expected:**

`AGENTS.md:456`, `docs/code-standards.md:67-80`.

**Evidence:**

```text
  $ find docs/specs -name "tests.md"
  (no output)
  ```

---

### FINDING 49 (id: `TST-049`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** Low
- **Area:** tests
- **Location:** `docs/coverage.toml:2298-2315` (sync rows)

**Description:**

The coverage matrix lists `educore-sync`
  and `educore-sync-inprocess` rows whose `tests` paths point
  at `crates/cross-cutting/sync/src/lib.rs` and
  `crates/cross-cutting/sync-inprocess/src/lib.rs` — i.e., at
  source files, not test files. The lint sub-module verifies
  the path exists (it does), but does not verify any `#[test]`
  lives in those paths. The sync crates are in `coverage.toml`
  but absent from AGENTS.md § Crate Inventory (the inventory
  lists 34 crates; the workspace has 38).

**Expected:**

`AGENTS.md:280-300` § Crate Inventory; the
  coverage row's `tests` field should point at a test file.

**Evidence:**

```text
  $ grep -E "tests = " docs/coverage.toml | grep -E "sync" 
  tests = "crates/cross-cutting/sync/src/lib.rs"
  tests = "crates/cross-cutting/sync-inprocess/src/lib.rs"
  ```

---

### FINDING 50 (id: `TST-050`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** Low
- **Area:** tests
- **Location:** `crates/educore/src/lib.rs:88` (single test)

**Description:**

The umbrella crate's single `mod tests`
  contains exactly one `#[test]` (smoke test of the
  re-exports). No test verifies that `educore::academic`,
  `educore::assessment`, etc. all compile and link; no test
  verifies that the umbrella's re-exports match the internal
  `educore-<name>` package names (a re-export regression would
  be silent).

**Expected:**

`AGENTS.md:35` — "Consumers therefore write
  `educore::academic::commands::*`".

**Evidence:**

```text
  $ wc -l crates/educore/src/lib.rs
   94
  $ grep -c "#\[test\]\|#\[tokio::test\]" crates/educore/src/lib.rs
  1
  ```

---

### FINDING 51 (id: `TST-051`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** Low
- **Area:** tests
- **Location:** `crates/domains/communication/src/services.rs`

**Description:**

Communication declares proptest in
  `Cargo.toml` and uses it for some services, but the
  `NotificationService` (the cross-crate fan-out point for
  events) has no property test asserting idempotent delivery
  under duplicate `EventEnvelope` receipt.

**Expected:**

`docs/ports/event-bus.md` — at-least-once
  delivery; consumer-side dedup invariants.

**Evidence:**

```text
  $ grep -B2 -A8 "proptest!" crates/domains/communication/src/services.rs | head -30
  ```

---

### FINDING 52 (id: `TST-052`)

- **Source:** `docs/audit_reports/findings/wave7-tests.md`
- **Severity:** Low
- **Area:** tests
- **Location:** `crates/adapters/storage-surrealdb/`

**Description:**

The SurrealDB adapter ships **12** unit
  tests (in `src/`) and **1** integration test
  (`tests/outbox_e2e.rs`). SurrealDB is the engine's "Phase 0
  primary target" per `docs/build-plan.md:43-44`, yet has the
  smallest test surface of the four adapter crates. No tests
  cover the SurrealDB-specific UUID coercion
  (`SurrealUuid` handling seen in
  `parity_event_log_filter.rs:146` `Err(e) if
  format!("{e:?}").contains("SurrealUuid")`) at the adapter
  level.

**Expected:**

`docs/build-plan.md:43-44`; AGENTS.md
  Storage Adapters section names SurrealDB as a primary
  target.

**Evidence:**

```text
  $ grep -c "SurrealUuid\|surreal_uuid" crates/adapters/storage-surrealdb/src/*.rs
  0
  $ grep -rE "SurrealUuid" crates/tools/storage-parity/tests/parity_event_log_filter.rs
  Err(e) if format!("{e:?}").contains("SurrealUuid") => { ... }
  ```

---

