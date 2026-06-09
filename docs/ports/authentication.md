# Authentication Port

## Purpose

The authentication port defines how the engine obtains a session for an
incoming request. The engine does not own user credentials, password
hashing, OAuth flows, or session storage. The consumer supplies an
adapter that produces a `Session` value.

## Trait: `AuthProvider`

```rust
#[async_trait]
pub trait AuthProvider: Send + Sync + std::fmt::Debug {
    async fn authenticate(&self, credential: Credential) -> Result<Session>;
    async fn validate(&self, token: &AuthToken) -> Result<Session>;
    async fn revoke(&self, token: &AuthToken) -> Result<()>;
    async fn refresh(&self, token: &AuthToken) -> Result<Session>;
}
```

The trait is object-safe.

## Credential

```rust
pub enum Credential {
    Bearer(BearerToken),
    UsernamePassword { username: String, password: SecretString },
    Oauth2 { code: String, redirect_uri: Url, code_verifier: Option<String> },
    Saml { assertion: String, relay_state: Option<String> },
    ApiKey { id: String, key: SecretString },
    Biometric { device_id: String, signature: Vec<u8>, timestamp: Timestamp },
    Anonymous,
}
```

A `Credential::Anonymous` is rejected by the default adapters except in
public-facing flows (e.g. public exam result lookup, when explicitly
allowed by configuration).

## Session

```rust
pub struct Session {
    pub session_id: SessionId,
    pub user_id: UserId,
    pub school_ids: Vec<SchoolId>,         // a user may belong to multiple schools
    pub active_school_id: SchoolId,        // the tenant for the current request
    pub roles: Vec<RoleId>,
    pub capabilities: BTreeSet<Capability>,
    pub mfa_satisfied: bool,
    pub issued_at: Timestamp,
    pub expires_at: Timestamp,
    pub metadata: BTreeMap<String, String>,
}
```

`Session` is a value type. It carries everything the engine needs to
authorize and tenant-isolate a command. Capabilities are pre-computed
when the session is created; the engine does not consult the RBAC
storage on every command.

## AuthToken

```rust
pub struct AuthToken {
    pub scheme: AuthScheme,
    pub value: SecretString,
    pub metadata: BTreeMap<String, String>,
}

pub enum AuthScheme {
    Bearer,
    Cookie,
    Custom(&'static str),
}
```

`validate(token)` produces a fresh `Session` for an incoming request.
The adapter may cache token-to-session mappings but MUST verify the
token's signature or validity on each call.

## Capability Check

The engine exposes a capability check helper that uses the session:

```rust
impl Engine {
    pub fn rbac(&self) -> &dyn RbacPort {
        &*self.rbac_port
    }
}

#[async_trait]
pub trait RbacPort: Send + Sync {
    async fn has(&self, session: &Session, capability: Capability) -> Result<bool>;
    async fn require(&self, session: &Session, capability: Capability) -> Result<()>;
}
```

`require` returns `DomainError::Forbidden` if the session lacks the
capability. The engine calls `require` at the command boundary.

## Multi-School Users

A parent may have children in two schools. Their session contains both
`SchoolId`s. The engine's `TenantContext` selects the active school
based on the request. Commands that target aggregates in the inactive
school are rejected.

A "switch school" action in the consumer application changes
`session.active_school_id` and the engine re-validates capabilities
for the new school.

## Two-Factor Authentication

When a session is `mfa_satisfied = false`, the engine restricts
sensitive commands. The adapter decides which commands require MFA
based on configuration.

A second factor may be satisfied by:

- TOTP (RFC 6238)
- SMS code
- Email code
- WebAuthn / FIDO2
- Backup code

The adapter produces a "pending MFA" session and the consumer collects
the second factor. On success, the session is upgraded to
`mfa_satisfied = true`.

## Session Revocation

`revoke(token)` invalidates the token. The adapter updates its session
store. Subsequent `validate` calls return `AuthError::Revoked`.

A super-admin can also revoke all sessions for a user (e.g. after
password change or suspected compromise).

## Token Refresh

`refresh(token)` returns a new `Session` for a non-expired token. The
adapter may rotate the token value. The old token is invalidated.

## Error Type

```rust
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("invalid credentials")] InvalidCredentials,
    #[error("account locked: {0}")] AccountLocked(String),
    #[error("account disabled")] AccountDisabled,
    #[error("token expired")] Expired,
    #[error("token revoked")] Revoked,
    #[error("malformed token: {0}")] Malformed(String),
    #[error("MFA required")] MfaRequired,
    #[error("MFA failed: {0}")] MfaFailed(String),
    #[error("rate limit exceeded")] RateLimited,
    #[error("infrastructure error: {0}")] Infrastructure(#[source] Box<dyn std::error::Error + Send + Sync>),
}
```

The engine maps `AuthError` to `DomainError::Forbidden` for the user
and logs the cause server-side.

## Configuration

The consumer constructs the adapter:

```rust
let auth: Arc<dyn AuthProvider> = Arc::new(
    JwtAuthProvider::builder()
        .signing_key(env::var("JWT_SECRET")?)
        .issuer("smsengine.example.com")
        .audience("smsengine")
        .access_ttl(Duration::from_hours(1))
        .refresh_ttl(Duration::from_days(7))
        .build()
);
```

Alternative adapters:

- `LocalPasswordAuthProvider` — username + password against a local
  user table (hashed with argon2).
- `OAuth2AuthProvider` — external OAuth2/OIDC (Google, Microsoft, etc.).
- `SamlAuthProvider` — SAML 2.0 (enterprise IdP).
- `ApiKeyAuthProvider` — service-to-service auth.

## Worked Example

A consumer middleware extracts the bearer token, validates it, and
passes the session to the engine:

```rust
async fn handle(req: HttpRequest) -> Result<HttpResponse, HttpError> {
    let token = req.bearer_token()
        .ok_or(HttpError::Unauthorized)?;
    let session = engine.auth().validate(&token).await
        .map_err(|_| HttpError::Unauthorized)?;

    let tenant = TenantContext::new(session.active_school_id, session.user_id);

    let student = engine
        .students()
        .with_tenant(&tenant)
        .admit(AdmitStudentCommand { tenant, ... })
        .await?;

    Ok(HttpResponse::json(student))
}
```

## Object Safety

`AuthProvider` and `RbacPort` are object-safe.

## Testing

The port requires:

- Unit tests of every `Credential` variant.
- Integration tests for token issue, validate, refresh, revoke.
- A test for expired and revoked tokens.
- A test for cross-tenant denial.
- A test for MFA required / satisfied.
- A test for rate limiting.
- A test for infrastructure failure.

## Offline Mode

In offline mode, the local storage adapter holds a cached user
directory. Sessions are issued locally and reconciled on reconnect. The
auth provider does not change; the consumer's adapter decides how to
cache and reconcile.

## Audit

Successful and failed authentication attempts are written to the audit
sink. Sensitive material (passwords, MFA codes) is never logged.
