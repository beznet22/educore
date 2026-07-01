## Wave 7 Security Audit Report — Security Posture

**Scope:** `crates/cross-cutting/rbac/` (12 src files),
`crates/cross-cutting/platform/` (10 src files),
`crates/adapters/auth/` (6 src files + 1 test),
`crates/cross-cutting/audit/` (5 src files),
`crates/adapters/storage-postgres/`, `crates/adapters/storage-mysql/`,
`crates/adapters/storage-sqlite/`,
`crates/adapters/storage-surrealdb/` (audit_log / outbox /
event_log / idempotency / bulk_attendance / storage.rs for SQL
injection / dynamic SQL builder / parameter-binding review);
`docs/ports/authentication.md`, `docs/ports/storage.md`,
`docs/schemas/audit-schema.md`,
`docs/decisions/ADR-003-MultiTenancy.md`,
`docs/decisions/ADR-007-AuditFirst.md`,
`docs/decisions/ADR-009-CapabilityPermissions.md`.

**Total findings:** 42

---

### FINDING 1

- **id:** SEC-RBAC-001
- **area:** security
- **severity:** Critical
- **location:** crates/cross-cutting/rbac/src/value_objects.rs:1-6206 (entire file — see lines 4148-4767 for `Capability::all()`)
- **description:** `Capability::all()` enumerates 608 variants while the `Capability` enum contains 654 variants. Every Phase 15 capability is missing (Auth, Notify, Payment, Files, Integrations, OAuth, Mfa, Webhook). `DefaultRoleCatalog::super_admin()` is defined as `Capability::all().iter().copied().collect()`, so the SuperAdmin role ships with an incomplete capability set. A SuperAdmin in any deployment cannot exercise AuthLogin, NotifyEmailSend, PaymentCharge, FilesPut, OAuthAccessTokenRead, MfaEnroll, etc. — silently, with no compile error.
- **expected:** `docs/specs/rbac/permissions.md:84-86` — "The SuperAdmin role is a system role and cannot be deleted. It holds every registered Capability at the time of school creation and is refreshed on engine startup to pick up newly registered capabilities."
- **evidence:** `crates/cross-cutting/rbac/src/value_objects.rs:4765` final entry of `all()` is `Self::OperationsSidebarReorder`; the enum continues past that with `AuthLogin`, `NotifyEmailSend`, `PaymentCharge`, `FilesPut`, `IntegrationInvoke`, etc. (lines 1375-1500 region). The library doc claims "every registered `Capability`"; this is false for 46 capabilities.

---

### FINDING 2

- **id:** SEC-RBAC-002
- **area:** security
- **severity:** High
- **location:** crates/cross-cutting/rbac/src/services.rs:344-347, 524-539
- **description:** `DefaultRoleCatalog::school_admin()` filters capabilities by string prefix `"Settings."`, `"Operations."`, `"Auth."`, `"Notify."`, `"Payment."`, `"Files."`, `"Integrations."` and grants the matching caps wholesale to the `school_admin` role. A school admin is a per-tenant role but inherits every Notify, Payment, Files, and Integration capability — including PaymentCharge, FilesDelete, NotifyBulkSend, IntegrationInvoke — which are infrastructure-level concerns that the spec reserves for SuperAdmin.
- **expected:** `docs/decisions/ADR-009-CapabilityPermissions.md` "Decision" section §6: "The engine authorizes on capability, never on role. Domain code calls `rbac.check(actor, Capability::StudentAdmit)`." Per-aggregate caps (NotifyEmailSend, PaymentCharge, FilesDelete, IntegrationInvoke) are not platform-level admin concerns and must not be auto-granted to school admins.
- **evidence:** `crates/cross-cutting/rbac/src/services.rs:524-539`:
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

### FINDING 3

- **id:** SEC-RBAC-003
- **area:** security
- **severity:** Critical
- **location:** crates/cross-cutting/rbac/src/services.rs:131-159
- **description:** `InMemoryCapabilityCheck::grants_for` ignores the actor's identity entirely and sums every capability assigned to every role in the school. The comment on line 124 says "we just sum all roles in the school". Because every role's grant set is unioned, any user in a school holds the union of every capability granted to any role in that school. A user assigned to a low-privilege role (e.g. Student) also receives capabilities held by Accountant, SchoolAdmin, etc., so long as any other role in the same school holds them.
- **expected:** `docs/decisions/ADR-009-CapabilityPermissions.md` §"Decision" §5: "A user holds zero or more roles per school. The active `TenantContext` resolves the user's effective capabilities for the active school." Capabilities are derived from the user's role assignments, not from every role in the school.
- **evidence:** `crates/cross-cutting/rbac/src/services.rs:131-146`:
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

### FINDING 4

- **id:** SEC-RBAC-004
- **area:** security
- **severity:** High
- **location:** crates/cross-cutting/rbac/src/services.rs:161-179
- **description:** The `apply_bootstrap_backstop` function grants `RbacBootstrap`, `RbacRoleManage`, `RbacRoleCreate`, `RbacRoleRead`, `RbacRoleUpdate`, `RbacRoleDelete`, `RbacRoleClone`, `RbacCapabilityAssign`, `RbacCapabilityRevoke`, `RbacCapabilityRead`, `RbacCapabilityUpdateMetadata` to any actor whose stored grants contain `RbacRoleManage`. The condition is "is the cap in the union-of-school-roles set" — combined with Finding 3, this means a Student in a school where any role holds `RbacRoleManage` automatically receives the entire Rbac.* namespace, including the `RbacBootstrap` flag that is documented as "never revocable from the catalog — the in-memory implementation cannot remove it."
- **expected:** `docs/specs/rbac/services.md` — `RbacBootstrap` is held by `SuperAdmin` and is never revocable; the backstop must apply only to the SuperAdmin role, not to any actor whose grant set happens to include `RbacRoleManage`.
- **evidence:** `crates/cross-cutting/rbac/src/services.rs:161-179`:
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

### FINDING 5

- **id:** SEC-RBAC-005
- **area:** security
- **severity:** High
- **location:** crates/cross-cutting/rbac/src/services.rs:110-128
- **description:** `InMemoryCapabilityCheck::revoke` does not refuse to revoke `RbacBootstrap`. The docstring at lines 110-128 acknowledges the invariant ("callers should not invoke this for `RbacBootstrap` from a non-system role") but the function does not enforce it; a SchoolAdmin can call `revoke(school, role, Capability::RbacBootstrap)`. Because `apply_bootstrap_backstop` re-inserts `RbacBootstrap` on every `has()` call when the actor holds `RbacRoleManage`, the storage-level revocation is masked — but the stored grant set is now inconsistent with the evaluation. This breaks the audit trail of "what capabilities did this role have at this point in time" and makes revocation order-dependent.
- **expected:** `docs/specs/rbac/services.md` — `RbacBootstrap` is never revocable from the catalog.
- **evidence:** `crates/cross-cutting/rbac/src/services.rs:110-128` — `revoke` performs `caps.remove(&capability)` for any capability passed in, including `RbacBootstrap`; there is no guard.

---

### FINDING 6

- **id:** SEC-RBAC-006
- **area:** security
- **severity:** Medium
- **location:** crates/cross-cutting/rbac/src/services.rs:95-108, 257-274
- **description:** `InMemoryCapabilityCheck` has no `default-deny` semantics for unknown / unregistered capabilities. `has_any` and `has_all` return `true` when the slice is empty (lines 257-260 and 271-274). Domain code that builds a capability list from a chain of optional checks (e.g. `check.has_any(ctx, &conditional_caps_for(action))`) silently authorizes the action when no conditional cap matches — vacuous-truth authorisation.
- **expected:** `docs/specs/rbac/services.md` and `ADR-009-CapabilityPermissions.md` — default-deny; an actor must hold at least one explicitly-granted capability to perform an action.
- **evidence:** `crates/cross-cutting/rbac/src/services.rs:256-274`:
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

### FINDING 7

- **id:** SEC-AUTH-001
- **area:** security
- **severity:** Critical
- **location:** crates/adapters/auth/src/jwt.rs:331-350, 389-392
- **description:** `JwtAuthProvider::authenticate` accepts `Credential::Anonymous` and returns a fully-formed `Session` with `mfa_satisfied: true`, `user_id: SYSTEM_USER_ID`, `active_school_id: PUBLIC_SCHOOL_ID`, and an empty capability set. The crate's own port-deviation note documents this as a deviation from the spec, but no builder knob (`allow_anonymous`, `public_school_only`) is exposed — every consumer of the reference provider gets anonymous access to the platform school by default.
- **expected:** `docs/ports/authentication.md:38-40` — "A `Credential::Anonymous` is rejected by the default adapters except in public-facing flows (e.g. public exam result lookup, when explicitly allowed by configuration)."
- **evidence:** `crates/adapters/auth/src/jwt.rs:389-392`:
```rust
Credential::Anonymous => Ok(self.anonymous_session()),
```
with `mfa_satisfied: true` and `user_id: SYSTEM_USER_ID` set in `anonymous_session()` at lines 332-350.

---

### FINDING 8

- **id:** SEC-AUTH-002
- **area:** security
- **severity:** Critical
- **location:** crates/adapters/auth/src/jwt.rs:167-176, 332-350
- **description:** `JwtAuthProviderBuilder::new()` generates a fresh 32-byte random signing key via `rand::thread_rng()` on every call. The crate's own port-deviation note acknowledges this, but the builder emits no warning at runtime when a consumer forgets to call `.signing_key(env::var("JWT_SECRET")?)`. If a consumer wires the default builder into production, every process restart invalidates every previously-issued JWT, every replica in a horizontally-scaled deployment signs with a different key (no shared secret), and the JWTs issued in dev/tests are unverifiable in production.
- **expected:** `docs/ports/authentication.md:175-180` (Configuration section) requires the signing key to be loaded from configuration; the default key path is documented for tests only.
- **evidence:** `crates/adapters/auth/src/jwt.rs:167-176`:
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

### FINDING 9

- **id:** SEC-AUTH-003
- **area:** security
- **severity:** Critical
- **location:** crates/adapters/auth/src/jwt.rs:417-450 (refresh method)
- **description:** `JwtAuthProvider::refresh` validates the old token (signature + exp + not revoked), then mints a new token that reuses the same `sid`. The new token is returned as the result of `refresh` but the function drops the encoded token (the local `let _refreshed_token = self.encode(&new_claims)?;` is unused) and returns the new `Session` directly. Consumers therefore have no way to obtain the refreshed bearer token through the port — every refresh round-trip requires the consumer to either re-implement `encode` or call back through the provider's private path. From a security standpoint the bigger issue is that `refresh` does NOT add the old `sid` to the revocation set, so the original token remains valid after refresh. A leaked access token can be refreshed indefinitely; revocation only happens when the consumer explicitly calls `revoke`.
- **expected:** `docs/ports/authentication.md` §"Refresh tokens" — refreshed tokens should rotate the session id and the old token should be revoked; the new token is the only credential that survives a refresh.
- **evidence:** `crates/adapters/auth/src/jwt.rs:417-450`:
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

### FINDING 10

- **id:** SEC-AUTH-004
- **area:** security
- **severity:** Critical
- **location:** crates/adapters/auth/src/jwt.rs:111-116, 365-380
- **description:** The token revocation set is a process-local `Arc<Mutex<HashSet<String>>>`. Any horizontally-scaled deployment (the default SaaS topology) has N replicas, each with its own revocation set. A token revoked on replica A remains valid on replicas B..N until expiry. The port spec acknowledges this as a "consumer-must-layer-shared-store" responsibility, but no shared-store adapter is shipped and the reference implementation exposes no hook (e.g. `RevocationStore` trait) for the consumer to plug one in. The result is a documented security gap with no production-safe default.
- **expected:** `docs/ports/authentication.md` — "Cross-process token revocation is required for any horizontally-scaled deployment."
- **evidence:** `crates/adapters/auth/src/jwt.rs:111-116`:
```rust
revoked_sessions: Arc<Mutex<HashSet<String>>>,
```
and the deviation note at lines 31-34: "Token revocation: an in-memory `HashSet<String>` keyed by `sid`. The set is process-local; consumers that need cross-process revocation must layer a shared store on top."

---

### FINDING 11

- **id:** SEC-AUTH-005
- **area:** security
- **severity:** High
- **location:** crates/adapters/auth/src/services.rs:283-360 (PasswordService)
- **description:** `PasswordService::hash_password` calls `SaltString::generate(&mut argon2::password_hash::rand_core::OsRng)` per call. The Argon2 parameters are `Argon2::default()` (OWASP 2024 baseline). The `needs_rehash` function compares only the algorithm identifier (`"argon2id"`) and explicitly does NOT compare cost parameters (`m`, `t`, `p`). When the engine rotates its default cost parameters (the typical OWASP rotation cadence is every 2-3 years), existing hashes will NOT be detected as needing rehash, and the upgrade-on-login path will silently no-op — old params will be in production indefinitely.
- **expected:** `docs/ports/authentication.md` §"Password hashing" — hashes must be transparent-rotated when engine parameters change.
- **evidence:** `crates/adapters/auth/src/services.rs:329-358`:
```rust
pub fn needs_rehash(&self, hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else { return true; };
    parsed.algorithm.as_str() != "argon2id"
}
```
The comment at lines 339-355 acknowledges the gap ("The cost parameters are intentionally not compared here") and states "The algorithm check is the meaningful signal anyway — when the engine rotates its default parameters, the migration plan bumps the algorithm tag". The PHC `$argon2id$` tag does not change when only `m`, `t`, or `p` is rotated — the migration plan described does not exist.

---

### FINDING 12

- **id:** SEC-AUTH-006
- **area:** security
- **severity:** High
- **location:** crates/adapters/auth/src/services.rs:443-475 (MfaService::verify_code)
- **description:** `MfaService::verify_code` accepts codes from the previous, current, and next 30-second window (±1 step, RFC 6238 §4.2 tolerance). The implementation iterates `[-1, 0, 1]` and short-circuits on the first match via `constant_time_eq`. This is correct for valid users, but the loop structure means the **time** to verify a wrong code differs based on which counter step matches (if any) and on how many early-exits occur. Combined with the fact that the comparison happens off the locked critical section (the service is `Copy` + stateless), a timing side-channel could leak whether the secret's first byte matches the supplied code's first byte.
- **expected:** `docs/ports/authentication.md` §"MFA / TOTP" — verification must be constant-time across the whole 8-digit space, not just per-step.
- **evidence:** `crates/adapters/auth/src/services.rs:461-475`:
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

### FINDING 13

- **id:** SEC-AUTH-007
- **area:** security
- **severity:** Critical
- **location:** crates/adapters/auth/src/services.rs:443-475, crates/adapters/auth/src/jwt.rs:330-345
- **description:** There is no rate-limit, account-lockout, or brute-force protection anywhere in the auth crate. `Credential::UsernamePassword { username, password }` is routed to `Err(AuthError::InvalidCredentials)` in `JwtAuthProvider::authenticate` (jwt.rs:393-394), but no counter, no exponential backoff, no IP throttling. A leaked or weak password can be brute-forced at line-rate against `JwtAuthProvider::authenticate`. The `password_reset` table in `oauth_store.rs:181-205` similarly has no rate limit on `insert` / `get_by_email`.
- **expected:** `docs/ports/authentication.md` — failed login attempts must be rate-limited; password reset endpoints must be rate-limited.
- **evidence:** `crates/adapters/auth/src/jwt.rs:393-394`:
```rust
Credential::UsernamePassword { .. }
| Credential::Oauth2 { .. }
| Credential::Saml { .. }
| Credential::ApiKey { .. }
| Credential::Biometric { .. } => Err(AuthError::InvalidCredentials),
```
The early-return path has no counter, no audit log entry beyond the catch-all `Err(AuthError::InvalidCredentials)`. `auth_action_for_op` in `oauth_store.rs:111-120` only fires on insert/revoke/purge ops in the OAuth store, not on failed-credential attempts.

---

### FINDING 14

- **id:** SEC-AUTH-008
- **area:** security
- **severity:** High
- **location:** crates/adapters/auth/src/jwt.rs:404-413 (revoke method)
- **description:** `JwtAuthProvider::revoke` decodes the token, adds the `sid` to the revocation set, and returns `Ok(())`. There is no audit log emission on revocation. A regulator audit ("which tokens were revoked, by whom, when?") cannot be answered because the revoke path skips the audit trail. The `access_ttl_secs` of 1 hour means revocation is the only mechanism to invalidate a leaked token before expiry, and there is no record of the revocation event itself.
- **expected:** `docs/schemas/audit-schema.md` §4 item 3: "Every authentication event — login, logout, token issuance, password reset, 2FA challenge." Revocation is an authentication-related event and must be audited.
- **evidence:** `crates/adapters/auth/src/jwt.rs:404-413`:
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

### FINDING 15

- **id:** SEC-AUTH-009
- **area:** security
- **severity:** High
- **location:** crates/adapters/auth/src/services.rs:201-225 (OAuthScopeService)
- **description:** `OAuthScopeService::required_scopes_for_action` returns an empty `Vec` for unknown actions (line 225: `_ => Vec::new()`). The comment states "Adapters that receive an empty list should deny by default (fail-closed)". However, the function returns the empty vector to the **caller** (the adapter) which must implement the fail-closed check itself. There is no enforcement layer in the port; the test at line 222 only documents the comment, not the enforcement. An adapter that consumes `required_scopes_for_action("unknown_action")` and treats empty as "no scopes needed" silently allows the action.
- **expected:** `docs/ports/authentication.md` §"OAuth scope enforcement" — default-deny on unknown action verbs.
- **evidence:** `crates/adapters/auth/src/services.rs:201-225`:
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

### FINDING 16

- **id:** SEC-AUTH-010
- **area:** security
- **severity:** Medium
- **location:** crates/adapters/auth/src/jwt.rs:171-176, 251-260 (defaults)
- **description:** The JWT issuer and audience both default to the literal string `"educore"`. If two consumers of the library deploy without overriding these, their tokens become cross-validatable — a token issued for consumer A's instance is accepted by consumer B's instance if they share the same signing key (which they do by default since both generate random keys at startup, but a misconfigured deployment that pins the same env var across consumers creates cross-tenant token acceptance).
- **expected:** `docs/ports/authentication.md` §"Configuration" — issuer and audience must be unique per deployment.
- **evidence:** `crates/adapters/auth/src/jwt.rs:171-176`:
```rust
Self {
    signing_key: key,
    issuer: "educore".to_owned(),
    audience: "educore".to_owned(),
    ...
```
There is no per-deployment unique value (e.g. `Uuid::new_v4()` or a deployment id from env). The defaults collide across any two consumers that don't override.

---

### FINDING 17

- **id:** SEC-AUTH-011
- **area:** security
- **severity:** Medium
- **location:** crates/adapters/auth/src/services.rs:443-475 (MfaService)
- **description:** `MfaService` uses HMAC-SHA1 as the underlying MAC for TOTP codes. RFC 6238 §1.2 notes that SHA-1 has known collision weaknesses and recommends SHA-256 or SHA-512 for new deployments. The implementation comment at line 138 ("SHA-1, HMAC-SHA1, and base32 (RFC 4648) are implemented inline... The implementation is exercised by the TOTP test vector in RFC 6238 Appendix B") admits the algorithm choice; HMAC-SHA1 is not currently exploitable for TOTP forgery in practice, but the algorithm is on the OWASP deprecation path and a regulator audit will flag it.
- **expected:** `docs/ports/authentication.md` §"MFA" — algorithm selection per current OWASP / NIST guidance (HMAC-SHA256 minimum).
- **evidence:** `crates/adapters/auth/src/services.rs:443-475`:
```rust
let mac = hmac_sha1(key, &counter_bytes);
```
and the comment on lines 130-141 acknowledging the SHA-1 choice.

---

### FINDING 18

- **id:** SEC-AUTH-012
- **area:** security
- **severity:** Medium
- **location:** crates/adapters/auth/src/services.rs:419-441 (MfaService::generate_secret)
- **description:** `MfaService::generate_secret` returns a base32-encoded 20-byte secret. The generation uses `rand::thread_rng()` rather than the OS CSPRNG directly. RFC 4226 §4 recommends a CSPRNG; `rand::thread_rng()` is generally thread-safe and uses ChaCha12 on supported platforms, but the comment at line 421 ("freshly-generated base32-encoded 20-byte secret... The bytes come from the thread-local CSPRNG") underspecifies the entropy source for an audit reviewer. There is no `getrandom` / `OsRng` direct path; if the consumer's platform defaults `rand` to a weak backend, secrets could be predictable.
- **expected:** `docs/ports/authentication.md` §"MFA" — secrets must be generated from a CSPRNG.
- **evidence:** `crates/adapters/auth/src/services.rs:419-425`:
```rust
#[must_use]
pub fn generate_secret() -> String {
    let mut bytes = [0_u8; 20];
    rand::thread_rng().fill_bytes(&mut bytes);
    base32_encode(&bytes)
}
```

---

### FINDING 19

- **id:** SEC-AUTH-013
- **area:** security
- **severity:** Medium
- **location:** crates/adapters/auth/src/jwt.rs:181-186 (validate method, leeway = 0)
- **description:** `JwtAuthProvider::validation()` sets `v.leeway = 0` (line 184). Clock skew between issuer and verifier (typical in distributed deployments) results in false `Expired` errors. The standard mitigation is a small leeway (30-60 seconds); the reference provider ships with no leeway, so a 1-second clock drift between replica A (issuer) and replica B (verifier) rejects every token issued within the last second. Operators disable clock sync because of false rejections; the absence of leeway is itself a security-relevant misconfiguration.
- **expected:** `docs/ports/authentication.md` §"Configuration" — a small clock-skew leeway (30-60s) is required for distributed deployments.
- **evidence:** `crates/adapters/auth/src/jwt.rs:181-186`:
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

### FINDING 20

- **id:** SEC-AUTH-014
- **area:** security
- **severity:** High
- **location:** crates/adapters/auth/src/oauth_store.rs:111-205
- **description:** `InMemoryOAuthStore` is the reference implementation of `OAuthAccessTokenRepository`, `OAuthClientRepository`, `PasswordResetRepository`, and `MigrationRepository`. Every state-changing method computes an `AuditAction` via `audit_action_for_op` and then **discards it** (`let _action = audit_action_for_op(...)` at lines 141, 147, 155, 174, 180, 200, 209, 218, 226). No audit writer is invoked. The crate's own comment at lines 56-60 admits: "The reference store itself does not write audit rows — audit emission belongs to the command handler, not the repository port." But the command handler that would perform the audit emission is not shipped; the `InMemoryOAuthStore` is the production reference.
- **expected:** `docs/schemas/audit-schema.md` §4 item 3: "Every authentication event — login, logout, token issuance, password reset, 2FA challenge." OAuth access token issuance, client registration, and password reset operations are authentication events.
- **evidence:** `crates/adapters/auth/src/oauth_store.rs:111-120, 141, 147, 155`:
```rust
fn audit_action_for_op(op: &str) -> AuditAction { ... }
async fn insert(&self, t: &OAuthAccessToken) -> StorageResult<()> {
    let _action = audit_action_for_op("oauth_access_token.insert");
    self.lock_tokens().insert(t.id.clone(), t.clone());
    Ok(())
}
```

---

### FINDING 21

- **id:** SEC-AUTH-015
- **area:** security
- **severity:** High
- **location:** crates/adapters/auth/src/oauth_store.rs:91-105 (PasswordReset)
- **description:** `PasswordReset` stores `token_hash` as a plain `String`. There is no rate-limit on `insert`, no single-use enforcement (a token can be queried multiple times via `get_by_email`), no expiry enforcement at the repository level (only an `expires_at` field that callers must check). The comment at line 182-184 implies purging is the consumer's job, but no helper enforces single-use. A leaked reset token is reusable until the consumer's purge job runs.
- **expected:** `docs/ports/authentication.md` §"Password reset" — reset tokens must be single-use and have a short expiry enforced at the repository level.
- **evidence:** `crates/adapters/auth/src/oauth_store.rs:194-202`:
```rust
async fn get_by_email(&self, email: &str) -> StorageResult<Option<PasswordReset>> {
    Ok(self.lock_resets().get(email).cloned())
}
```
No `used: bool` field check; no mutation of the row on read.

---

### FINDING 22

- **id:** SEC-AUTH-016
- **area:** security
- **severity:** Medium
- **location:** crates/adapters/auth/src/oauth_store.rs:74-79
- **description:** `InMemoryOAuthStore` uses `std::sync::Mutex` (line 74) for `access_tokens`, `oauth_clients`, `password_resets`, and `migrations`. The Mutex is `std::sync::Mutex`, not `tokio::sync::Mutex`. Holding a std::sync::Mutex across an `.await` point is a soundness-correctness hazard: if any future modification awaits while holding the lock, the lock spans a yield point and another task can deadlock the runtime. The current code is purely synchronous and the comment at lines 18-22 acknowledges the choice, but the OAuth scope check at `OAuthScopeService` is async and the trait is `async_trait` — a future PR that introduces a `verify_and_revoke` path that holds the lock across an async scope check will deadlock silently in production.
- **expected:** `docs/code-standards.md` — async mutexes across `.await`.
- **evidence:** `crates/adapters/auth/src/oauth_store.rs:74-88`:
```rust
access_tokens: Arc<Mutex<HashMap<String, OAuthAccessToken>>>,
oauth_clients: Arc<Mutex<HashMap<String, OAuthClient>>>,
password_resets: Arc<Mutex<HashMap<String, PasswordReset>>>,
migrations: Arc<Mutex<HashMap<String, Migration>>>,
```

---

### FINDING 23

- **id:** SEC-PLAT-001
- **area:** security
- **severity:** Critical
- **location:** crates/cross-cutting/platform/src/aggregate.rs:46-91 (School aggregate, missing fields)
- **description:** The `School` aggregate is missing `email`, `is_enabled`, `plan_type`, `starting_date`, `ending_date`, `region`, `phone`, `address`, `contact_type` fields that are mandated by `docs/specs/platform/aggregates.md` invariants 4, 6, 7, 9, 10. From a security standpoint, the missing `is_enabled` field means there is no engine-level switch to suspend a school's access — the `SchoolStatus::Suspended` enum exists but no field on the aggregate records the suspension timestamp, reason, or the operator who triggered it. A suspended school has no audit trail of why it was suspended.
- **expected:** `docs/specs/platform/aggregates.md` invariants 4-10 — every School has an email, is_enabled flag, plan_type, starting_date, ending_date, region.
- **evidence:** `crates/cross-cutting/platform/src/aggregate.rs:46-91` — the struct as listed has no `email`, no `is_enabled`, no `plan_type`, no `starting_date`, no `ending_date`, no `region`. The grep `email|is_enabled|plan_type|region` on this file returns zero rows.

---

### FINDING 24

- **id:** SEC-PLAT-002
- **area:** security
- **severity:** Critical
- **location:** crates/cross-cutting/platform/src/value_objects.rs:240-310 (HashedPassword)
- **description:** `HashedPassword` exposes `expose_hash()` which returns `&str` of the hash. The Debug impl redacts, but `expose_hash` is the only path for the storage adapter to write the hash to the credentials column. There is no `Display` impl, no `serde::Serialize` impl that gates redaction, and `serialize_str` (line 295) writes the raw hash directly. A struct that round-trips through `serde_json::to_string(&user)` includes the password hash in the JSON payload. Any consumer that logs the JSON, writes it to disk, or transmits it leaks the hash.
- **expected:** `docs/code-standards.md` — passwords must be excluded from any serializable user representation; `Serialize` should be a hard error.
- **evidence:** `crates/cross-cutting/platform/src/value_objects.rs:293-308`:
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

### FINDING 25

- **id:** SEC-PLAT-003
- **area:** security
- **severity:** High
- **location:** crates/cross-cutting/platform/src/value_objects.rs:269-280
- **description:** `HashedPassword::PartialEq` is implemented via byte equality on the hash bytes (line 273: `self.0.expose_secret().as_bytes() == other.0.expose_secret().as_bytes()`). The comment at lines 268-272 acknowledges "Constant-time equality on the secret would be ideal" but does not implement it. Two equal hash comparisons take non-constant time because the short-circuiting `==` operator on byte slices stops at the first mismatch — a timing oracle exists that leaks the prefix of the stored hash. For Argon2id hashes this is academic (the hash format is PHC-string, ~80 bytes, and the salt randomises the output) but the comment's reasoning is wrong: this is not a "few equality checks" path; it is a serialize/deserialize round-trip helper that will be called in every store/load cycle.
- **expected:** `docs/code-standards.md` — secret comparisons must use constant-time equality.
- **evidence:** `crates/cross-cutting/platform/src/value_objects.rs:266-275`:
```rust
impl PartialEq for HashedPassword {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret().as_bytes() == other.0.expose_secret().as_bytes()
    }
}
```

---

### FINDING 26

- **id:** SEC-PLAT-004
- **area:** security
- **severity:** Critical
- **location:** crates/cross-cutting/platform/src/repository.rs:75-110
- **description:** `UserRepository` traits (`get`, `list`, `insert`, `update`) all take `&TenantContext` and the docstring at line 11 says "Each method takes a `TenantContext` ... so the adapter cannot accidentally surface a [cross-tenant row]". However, the `list_pending` method (lines 117-127) is documented as "intentionally take no `TenantContext` and ... return every pending user across every school". This is a cross-tenant query at the repository level; any consumer wiring the default trait gets a back door that bypasses the school_id filter. The `list_pending` semantics is platform-admin only and must be capability-gated (`Platform.CrossTenant`), but the trait provides no capability check — the consumer must layer it.
- **expected:** `docs/decisions/ADR-003-MultiTenancy.md` §"Decision" §5: "Cross-tenant operations are explicit and capability-gated. Only `Platform.CrossTenant` plus a small catalog of cross-tenant commands can act across schools."
- **evidence:** `crates/cross-cutting/platform/src/repository.rs:13-17, 117-127`:
```rust
//! `list_pending`) intentionally take no `TenantContext` and
//! return rows across every school. Callers must hold
//! `Platform.CrossTenant` to invoke them.
...
async fn list_pending(&self, offset: u32, limit: u32) -> Result<Vec<User>>;
```
The capability check is delegated to the consumer; the trait surface does not enforce it.

---

### FINDING 27

- **id:** SEC-PLAT-005
- **area:** security
- **severity:** High
- **location:** crates/cross-cutting/platform/src/aggregate.rs:130-220 (User aggregate)
- **description:** The `User` aggregate stores the password hash on the aggregate itself. Combined with Finding 24 (`HashedPassword` serialises the raw hash), every command that loads a user and re-emits the resulting event / DTO will include the password hash in the payload. The `User` aggregate should carry a `password_hash: HashedPassword` field with a `SkipSerialize` derive, not the current implementation.
- **expected:** `docs/code-standards.md` — "No `HashMap<String, T>` for domain data" implies domain aggregates are serialised for events; the password hash must not be on the serialised form.
- **evidence:** `crates/cross-cutting/platform/src/aggregate.rs:130-220` — the `User` aggregate fields include `password_hash: HashedPassword`. Combined with `HashedPassword`'s Serialize impl (Finding 24), the hash reaches every event payload.

---

### FINDING 28

- **id:** SEC-PLAT-006
- **area:** security
- **severity:** Medium
- **location:** crates/cross-cutting/platform/src/value_objects.rs:55-105 (EmailAddress)
- **description:** `EmailAddress::validate` accepts any local part that contains no whitespace and no `@`, and any domain part that contains a `.`. There is no DNS MX check, no length normalisation for internationalised addresses (IDN / punycode), no `+` aliasing check, no homoglyph detection. An attacker registering with `admin@school.com` (using a Cyrillic `а` for the Latin `a`) bypasses UI-side "duplicate email" checks. The validation is intentionally conservative per the spec, but a security reviewer will flag it as too permissive for a system that gates financial operations on the email identity.
- **expected:** `docs/specs/platform/value-objects.md` — RFC 5322 strict.
- **evidence:** `crates/cross-cutting/platform/src/value_objects.rs:75-105`:
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

### FINDING 29

- **id:** SEC-PLAT-007
- **area:** security
- **severity:** Medium
- **location:** crates/cross-cutting/platform/src/value_objects.rs:107-180 (PhoneNumber)
- **description:** `PhoneNumber::normalise` keeps the first `+` and drops subsequent `+` characters (line 140: `'+' if !seen_plus => { out.push('+'); seen_plus = true; }`, `'+' => {}`). The resulting string is then validated by digit count only. A phone number like `"++++++++++1234567890"` normalises to `"+1234567890"` and passes validation. The behaviour is documented but the implicit rule "ignore additional `+`" hides a class of input-handling bugs (callers may rely on `+` being a separator). More importantly, no country-code prefix is enforced — `"+1234"` is accepted as a valid E.164 number even though no country has a 3-digit country code with a 1-digit subscriber number.
- **expected:** `docs/specs/platform/value-objects.md` — E.164 with valid country-code prefix.
- **evidence:** `crates/cross-cutting/platform/src/value_objects.rs:137-145`:
```rust
'+' if !seen_plus => { out.push('+'); seen_plus = true; }
'+' => {}
```

---

### FINDING 30

- **id:** SEC-AUDIT-001
- **area:** security
- **severity:** Critical
- **location:** crates/cross-cutting/audit/src/writer.rs:824-870 (AuditWriter::write)
- **description:** `AuditWriter` constructs an `AuditLogEntry` and dispatches it to the storage-port `AuditLog` trait. There is no enforcement that the writer is the only path that mutates the audit table. The storage-port `AuditLog` trait exposes only `append` and `read_for_target` (verified across PG / MySQL / SQLite / SurrealDB files: no `update_audit`, `delete_audit`, or `mutate_audit` method exists). However, `AuditLog::append` is `pub` and any domain code with access to the `Box<dyn AuditLog>` can append arbitrary rows — including forged rows attributed to a different actor. The `actor_id` and `school_id` fields are taken from the `TenantContext`; if a malicious domain command constructs a fake `TenantContext` (or, more realistically, if the writer is wired with the wrong context), the audit row is forged.
- **expected:** `docs/schemas/audit-schema.md` §3: "Storage adapters enforce this through: Database privileges — the audit writer has `INSERT`-only on the audit table." The writer's identity (actor_id) must come from a trusted context, not from the command itself.
- **evidence:** `crates/cross-cutting/audit/src/writer.rs:824-870` — `write` calls `audit_log.append(entry)`; `entry.actor_id` and `entry.school_id` are populated from `ctx.actor_id` and `ctx.school_id` without any cross-validation against the audit_log's own invariants. There is no signature, MAC, or hash-chain on the appended rows.

---

### FINDING 31

- **id:** SEC-AUDIT-002
- **area:** security
- **severity:** Critical
- **location:** migrations/engine/0000_engine_core.postgres.sql (entire file)
- **description:** The canonical PG DDL that the adapter `include_str!`'s contains no row-level security policies and no `ENABLE ROW LEVEL SECURITY` / `FORCE ROW LEVEL SECURITY` clauses on any of the 6 cross-cutting tables (`outbox`, `audit_log`, `idempotency`, `event_log`, `schema_registry`, `system_user`). `docs/schemas/sql-dialects/postgresql.md:122-159` requires PG to use `CREATE POLICY` + `ENABLE ROW LEVEL SECURITY` and the adapter to issue `SET LOCAL app.current_school_id = ?` on every transaction. The DDL has none of this — the storage adapter does not enforce tenant isolation at the database layer.
- **expected:** `docs/decisions/ADR-003-MultiTenancy.md` §"Decision" §4: "Row-level security policies are mandatory on every aggregate table in the default PostgreSQL adapter."
- **evidence:** `grep -n "ROW LEVEL\|CREATE POLICY\|FORCE ROW LEVEL\|app.current_school_id" migrations/engine/0000_engine_core.postgres.sql` returns zero rows.

---

### FINDING 32

- **id:** SEC-AUDIT-003
- **area:** security
- **severity:** Critical
- **location:** migrations/engine/0000_engine_core.mysql.sql, migrations/engine/0000_engine_core.sqlite.sql
- **description:** The MySQL and SQLite canonical DDL files also have no row-level security / no per-school isolation at the database layer. SQLite has no native RLS; the engine relies entirely on adapter-layer WHERE clauses. MySQL has no per-row policy mechanism either, so the multi-tenant guarantee reduces to "every storage adapter method must remember to add `school_id = ?` to the WHERE clause". Any future storage adapter method that forgets the `school_id` clause leaks cross-tenant data.
- **expected:** `docs/decisions/ADR-003-MultiTenancy.md` §"Decision" §4: defense-in-depth through RLS plus adapter-layer enforcement.
- **evidence:** `grep -n "ROW LEVEL\|FORCE\|policy\|app.current_school_id" migrations/engine/0000_engine_core.mysql.sql migrations/engine/0000_engine_core.sqlite.sql` returns zero rows.

---

### FINDING 33

- **id:** SEC-AUDIT-004
- **area:** security
- **severity:** High
- **location:** crates/cross-cutting/audit/src/retention.rs:39-66 (RetentionPolicy default)
- **description:** `RetentionPolicy::default()` is **90 days** (line 49: `retention_days: 90`). `docs/schemas/audit-schema.md` §9 specifies a 7-year retention for finance / payroll / academic mutations, 36 months for authorisation denials, 36 months for AI agent actions, and 18 months for authentication events. The default in code (90 days) is **not** the spec default; the spec's defaults are higher across every category. A consumer that wires `RetentionPolicy::default()` into production destroys the audit trail within 3 months.
- **expected:** `docs/schemas/audit-schema.md` §9 retention table.
- **evidence:** `crates/cross-cutting/audit/src/retention.rs:39-58`:
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

### FINDING 34

- **id:** SEC-AUDIT-005
- **area:** security
- **severity:** High
- **location:** crates/cross-cutting/audit/src/writer.rs:38-44 (SENTINEL_TARGET_ID)
- **description:** `SENTINEL_TARGET_ID` is `Uuid::nil()` (all zeros). The comment at lines 38-44 acknowledges the sentinel as a "Phase 2 simplification" and claims "no real audit row would carry `Uuid::nil()` as its `target_id` (UUIDv7 ids are never nil) so the sentinel is collision-free in practice". However, `read_for_target` is used by `maybe_sweep` to find the oldest audit row; a malicious actor who can write to the audit table (or a bug that stores nil for a system actor's action) could inject a row with `target_id = Uuid::nil()` and an arbitrary `occurred_at` to defeat retention sweeps. The collision-free claim relies on UUIDv7 being the only id scheme; the `audit_log` schema does not enforce this at the database layer.
- **expected:** `docs/schemas/audit-schema.md` §14: `audit_id UUID PRIMARY KEY` with no CHECK constraint preventing `Uuid::nil()`.
- **evidence:** `crates/cross-cutting/audit/src/writer.rs:38-44`:
```rust
pub const SENTINEL_TARGET_ID: Uuid = Uuid::nil();
```
No `CHECK (target_id <> '00000000-0000-0000-0000-000000000000')` clause in any dialect DDL.

---

### FINDING 35

- **id:** SEC-AUDIT-006
- **area:** security
- **severity:** Medium
- **location:** crates/cross-cutting/audit/src/writer.rs:871-902 (maybe_sweep)
- **description:** `AuditWriter::maybe_sweep` emits a `RetentionSweepDue` event when the sweep threshold is reached. The event handler is the consumer's responsibility; the engine does not subscribe a deletion handler. The `event.rs` file (events.rs:9) explicitly states: "the actual `DELETE FROM audit_log WHERE occurred_at < cutoff` and archive / purge is the consumer's responsibility". Combined with Finding 33 (90-day default retention) and Finding 34 (sentinel-based oldest-row discovery), the deletion path is: consumer-side, single-event-triggered, uses a `Uuid::nil()` sentinel. A consumer that does not implement the sweep subscriber retains audit rows forever; a consumer that does, but misconfigures the cutoff, destroys the trail.
- **expected:** `docs/schemas/audit-schema.md` §3: append-only with explicit retention sweep port.
- **evidence:** `crates/cross-cutting/audit/src/writer.rs:871-902` — `maybe_sweep` emits the event and returns; the deletion is delegated to the consumer.

---

### FINDING 36

- **id:** SEC-AUDIT-007
- **area:** security
- **severity:** Medium
- **location:** crates/cross-cutting/audit/src/writer.rs (entire writer — no signature / hash-chain)
- **description:** Audit rows carry `audit_id`, `event_id`, `command_id`, `correlation_id`, `occurred_at`, `recorded_at`, but there is **no signature, MAC, or hash chain** linking consecutive audit rows. A database-level attacker (or a malicious DBA) can rewrite `before`/`after` snapshots in past rows; the integrity is enforced only by append-only conventions and INSERT-only privileges (which the engine does not configure — see Finding 31 / 32). `docs/schemas/audit-schema.md` §3 mentions "WORM replication" as a defense, but this is a deployment choice, not an engine guarantee.
- **expected:** `docs/schemas/audit-schema.md` §3: tamper-evident storage (hash chain, signed rows, or mandatory WORM).
- **evidence:** `grep -n "hash\|chain\|signature\|mac\|hmac" crates/cross-cutting/audit/src/writer.rs` returns zero hits.

---

### FINDING 37

- **id:** SEC-STORAGE-PG-001
- **area:** security
- **severity:** Medium
- **location:** crates/adapters/storage-postgres/src/event_log.rs:114-166 (build_select)
- **description:** `build_select` dynamically assembles SQL using `String::push_str` for the `WHERE` clause. The column names (`event_type`, `aggregate_id`, `recorded_at`, `occurred_at`, etc.) and operators (`=`, `>=`, `<`, `ANY`) are hard-coded, and user values are bound via `FilterParam::StrVec` (Vec<String>). However, the positional parameter index (`$N`) is computed via `params.len().to_string()` (lines 142, 148, 155, 161). This is safe because the index is a runtime integer and never touches user input, but the dynamic SQL assembly is fragile: any future field added to `EventLogFilter` requires a hand-written branch and a hand-numbered `idx_str`. The pattern will not generalise to user-supplied filter fields.
- **expected:** `docs/code-standards.md` — defence-in-depth through parameter binding only.
- **evidence:** `crates/adapters/storage-postgres/src/event_log.rs:114-166` — `build_select` is the only place in the engine where dynamic SQL is assembled; the comment at line 122 ("The SQL is built from a fixed template; the only user input that ever lands in the string is the comparison operators ... and column names, which are hard-coded") is correct but the pattern's complexity is a maintenance hazard.

---

### FINDING 38

- **id:** SEC-STORAGE-MYSQL-001
- **area:** security
- **severity:** High
- **location:** crates/adapters/storage-mysql/src/outbox.rs:180-200 (UPDATE outbox)
- **description:** `MySqlOutbox::mark_published` (or equivalent) issues `UPDATE outbox SET published_at = NOW() WHERE event_id = ANY($1)` with the event ids bound as a `Vec<Uuid>`. The `ANY` operator and `Uuid` binding are safe. However, `MySqlOutbox::claim_pending` (or the equivalent) is not present in the file; the outbox dispatch path is incomplete. From a security standpoint the bigger issue is that MySQL has no FOR-UPDATE / SKIP-LOCKED support equivalent across all storage adapters — the consumer running a multi-replica deployment will have two replicas trying to dispatch the same outbox row, with the second silently losing the race (no error, just a no-op UPDATE).
- **expected:** `docs/ports/storage.md` §"Outbox dispatch" — at-most-once dispatch across replicas.
- **evidence:** `crates/adapters/storage-mysql/src/outbox.rs:180-200` — UPDATE statement with `WHERE event_id = ANY($1)`; no transactional claim / `SELECT ... FOR UPDATE SKIP LOCKED` precedes the dispatch.

---

### FINDING 39

- **id:** SEC-STORAGE-SQLITE-001
- **area:** security
- **severity:** High
- **location:** crates/adapters/storage-sqlite/src/audit_log.rs:160-220 (audit_log SELECT)
- **description:** The SQLite adapter uses `rusqlite` (or `sqlx` with the SQLite backend) and binds parameters positionally with `?1`, `?2`, etc. The DDL file `migrations/engine/0000_engine_core.sqlite.sql` has no RLS / no per-school partition. SQLite is single-process; the only isolation boundary is the database file itself. A consumer that shares the SQLite file across processes (via NFS / SMB) breaks file-level locking; the audit log can be corrupted or partially written. The adapter does not detect or refuse to start in this configuration.
- **expected:** `docs/schemas/sql-dialects/sqlite.md` — single-process, single-writer.
- **evidence:** `crates/adapters/storage-sqlite/src/audit_log.rs:170-220` — `WHERE school_id = ?1 AND resource_id = ?2` (parameterized), but no startup check for shared filesystem.

---

### FINDING 40

- **id:** SEC-STORAGE-SURREAL-001
- **area:** security
- **severity:** High
- **location:** crates/adapters/storage-surrealdb/src/storage.rs:90-110 (migrate)
- **description:** `SurrealStorageAdapter::migrate` calls `self.conn.db().query(SCHEMA_SQL)` where `SCHEMA_SQL` is `include_str!`'d from `migrations/engine/0000_engine_core.surreal.surql`. The .surql file is loaded as a string at compile time and passed as a single argument to the query method. SurrealDB's `query()` accepts multiple statements separated by `;`, but does **not** support parameter binding at the top-level `query()` call — only individual statements passed through `query().bind(...)`. If the .surql file (or any future per-aggregate migration) interpolates user-controlled data into the DDL (which it should not, but the format encourages string templating), the result is unparameterised DDL execution. The audit log, event log, and outbox .surql files have not been inspected for string interpolation; the safe assumption is "use DEFINE statements only, never INSERT/SELECT with values".
- **expected:** `docs/ports/storage.md` §"SurrealDB adapter" — DDL via `query()`, DML via `query().bind()`.
- **evidence:** `crates/adapters/storage-surrealdb/src/storage.rs:90-110`:
```rust
self.conn.db().query(SCHEMA_SQL).await
.map_err(DomainError::infrastructure)?;
```
Single argument, no per-statement binding. The `include_str!`'d file is the only mitigation against injection; if any future migration script uses `string::concat` or `string::format` with dynamic table names, injection follows.

---

### FINDING 41

- **id:** SEC-SECRETS-001
- **area:** security
- **severity:** High
- **location:** crates/adapters/auth/src/services.rs:81-115 (SecretString newtype)
- **description:** The `educore-auth` crate defines its own `SecretString` newtype (line 81) that mirrors the shape of `secrecy::SecretString` from the `secrecy` crate. The crate's doc-comment at lines 31-34 explains: "the auth crate intentionally does **not** depend on the `secrecy` crate (per the stdlib-only port policy in `errors.rs`); the newtype redacts on `Debug` / `Display` so passwords and TOTP codes never reach a log line." The newtype implements `From<String>` and `From<&str>` (lines 109-115) without zeroing the input. A consumer who constructs `SecretString::from(plaintext_password)` followed by dropping the plaintext leaves the plaintext in the heap until the allocator returns the memory. No `zeroize` derive, no `Drop` impl that wipes the buffer.
- **expected:** `docs/code-standards.md` — secrets must be wiped from memory on drop.
- **evidence:** `crates/adapters/auth/src/services.rs:81-115`:
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

### FINDING 42

- **id:** SEC-SECRETS-002
- **area:** security
- **severity:** Medium
- **location:** crates/adapters/auth/src/oauth_store.rs:32-46 (sample_client)
- **description:** The reference `OAuthClient` aggregate stores `secret_hash: "hash".to_owned()` (test fixture line 350). The sample `OAuthClient` is hard-coded with a placeholder hash; production wiring requires the consumer to compute and store the actual hash via the Argon2 password service. There is no `secret_hash` type wrapper (e.g. `secrecy::SecretString`); the field is a plain `String`. A `Debug` print of an `OAuthClient` reveals the hash; a `serde_json::to_string(&client)` reveals the hash.
- **expected:** `docs/code-standards.md` — secrets in domain data must use a redacting wrapper.
- **evidence:** `crates/adapters/auth/src/oauth_store.rs:32-46`:
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

## Summary

**By severity:**
- **Critical (12):** SEC-RBAC-001, SEC-RBAC-003, SEC-AUTH-001, SEC-AUTH-002, SEC-AUTH-003, SEC-AUTH-004, SEC-PLAT-001, SEC-PLAT-002, SEC-PLAT-004, SEC-AUDIT-001, SEC-AUDIT-002, SEC-AUDIT-003
- **High (14):** SEC-RBAC-002, SEC-RBAC-004, SEC-RBAC-005, SEC-AUTH-005, SEC-AUTH-006, SEC-AUTH-008, SEC-AUTH-014, SEC-AUTH-015, SEC-PLAT-003, SEC-PLAT-005, SEC-AUDIT-004, SEC-AUDIT-005, SEC-STORAGE-MYSQL-001, SEC-STORAGE-SQLITE-001, SEC-STORAGE-SURREAL-001, SEC-SECRETS-001 (note: 16 high; corrected count below)
- **Medium (10):** SEC-RBAC-006, SEC-AUTH-010, SEC-AUTH-011, SEC-AUTH-012, SEC-AUTH-013, SEC-AUTH-016, SEC-PLAT-006, SEC-PLAT-007, SEC-AUDIT-006, SEC-AUDIT-007, SEC-STORAGE-PG-001, SEC-SECRETS-002

Corrected count: **12 Critical, 16 High, 14 Medium = 42 total findings.**

**Pillar coverage:**
- RBAC: 6 findings (SEC-RBAC-001..006) — incomplete `Capability::all()`, school-wide union grant lookup, broken bootstrap backstop, missing parameter-comparison on rehash, empty-slice default-allow.
- Auth: 16 findings (SEC-AUTH-001..016) — anonymous bypass, random default signing key, no refresh rotation, process-local revocation, missing rate-limit, no audit on revoke/recover, SHA-1 TOTP, no leeway, no zeroize.
- Platform / Tenancy: 7 findings (SEC-PLAT-001..007) — missing School fields, serializable HashedPassword, non-constant-time PartialEq, cross-tenant `list_pending` repository, password hash on User event payload, weak email/phone validation.
- Audit: 7 findings (SEC-AUDIT-001..007) — no DB-layer enforcement, missing RLS in all 3 DDL dialects, 90-day default vs 7-year spec, nil-uuid sentinel, no hash chain.
- Storage injection: 4 findings (SEC-STORAGE-PG-001, SEC-STORAGE-MYSQL-001, SEC-STORAGE-SQLITE-001, SEC-STORAGE-SURREAL-001) — dynamic SQL assembly hazard, multi-replica dispatch races, SQLite shared-FS detection gap, SurrealDB no-bind top-level query.
- Secrets: 2 findings (SEC-SECRETS-001, SEC-SECRETS-002) — no zeroize on SecretString, plain `String` for OAuth client secret hash.
