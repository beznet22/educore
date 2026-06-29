//! # JWT-based [`AuthProvider`] reference implementation.
//!
//! See `docs/ports/authentication.md` § "Configuration" for the
//! builder usage example. This adapter is **minimal**: it validates
//! HMAC-SHA256 (`HS256`) JWTs, looks up nothing in user storage,
//! and produces a [`Session`] with an empty capability set. A
//! production adapter would plug the JWT into a user lookup and a
//! capability resolver; see A.4 (services.rs) for that wiring.
//!
//! ## Algorithm
//!
//! - Signing: HMAC-SHA256 with a shared secret. The secret is a
//!   `Vec<u8>` (raw key material). The builder defaults to a
//!   random 32-byte key; consumers are expected to override it
//!   with `env::var("JWT_SECRET")` or equivalent in production.
//! - Claims: see [`JwtClaims`]. The `sid` (session id), `sub`
//!   (user id), `schools`, `active_school`, and `roles` are all
//!   carried as UUIDv7 strings.
//! - Token revocation: an in-memory `HashSet<String>` keyed by
//!   `sid`. The set is process-local; consumers that need
//!   cross-process revocation must layer a shared store on top.
//!   See ADR-015 § "auth port implementations" for the
//!   rationale.
//!
//! ## Deviations from `docs/ports/authentication.md`
//!
//! - The default signing key is **generated**, not loaded from
//!   env. Production wiring MUST override it via
//!   [`JwtAuthProviderBuilder::signing_key`]. The default is
//!   suitable for tests and the worked example only.
//! - `Credential::UsernamePassword`, `Credential::Oauth2`,
//!   `Credential::Saml`, `Credential::ApiKey`, and
//!   `Credential::Biometric` are all rejected with
//!   [`AuthError::InvalidCredentials`]. The JWT provider only
//!   accepts `Credential::Bearer` and `Credential::Anonymous`
//!   (the latter yields a placeholder dev session).
//! - `Credential::Anonymous` is permitted (against the port
//!   spec, which says default adapters reject it) because the
//!   JWT provider is also the public-content gateway in the
//!   reference setup. Consumers that need strict anonymous
//!   rejection should layer a check upstream.
//! - The capability set on the returned [`Session`] is always
//!   empty; JWT claims do not carry capabilities in this
//!   implementation. A future revision will resolve capabilities
//!   from the RBAC port at validate-time.
//! - All `roles` in the claims are parsed as UUIDv7 strings and
//!   scoped to `active_school` to build [`RoleId`] values. The
//!   JWT issuer is responsible for emitting role UUIDs that are
//!   compatible with the engine's RBAC store.

#![allow(clippy::missing_docs_in_private_items)]

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::{Identifier, SchoolId, SessionId, UserId, PUBLIC_SCHOOL_ID};
use educore_core::value_objects::Timestamp;
use educore_rbac::ids::RoleId;
use educore_rbac::value_objects::Capability;

use crate::errors::{AuthError, InfrastructureError};
use crate::port::{AuthProvider, AuthScheme, AuthToken, Credential, Session};

// ---------------------------------------------------------------------------
// JwtClaims
// ---------------------------------------------------------------------------

/// The JWT claim set used by [`JwtAuthProvider`].
///
/// Field naming follows the JOSE / RFC 7519 conventions for the
/// standard claims (`sub`, `iss`, `aud`, `iat`, `exp`); the
/// engine-specific claims (`sid`, `roles`, `schools`,
/// `active_school`, `mfa`) live alongside them.
///
/// All UUID-bearing fields are stored as their hyphenated string
/// form (UUIDv7). The provider parses them back into typed
/// identifiers at validation time; non-UUID values are rejected
/// with [`AuthError::Malformed`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// The authenticated user id (UUIDv7 string).
    pub sub: String,
    /// The token issuer; validated against the builder's
    /// configured issuer at `validate` time.
    pub iss: String,
    /// The token audience; validated against the builder's
    /// configured audience at `validate` time.
    pub aud: String,
    /// Issued-at (Unix seconds).
    pub iat: i64,
    /// Expiry (Unix seconds). Validated against wall-clock at
    /// `validate` time (with the jsonwebtoken default leeway of
    /// zero seconds).
    pub exp: i64,
    /// Session id (UUIDv7 string). Used as the key in the
    /// provider's revocation set.
    pub sid: String,
    /// The user's role ids (UUIDv7 strings), scoped to
    /// `active_school` at the issuer.
    #[serde(default)]
    pub roles: Vec<String>,
    /// The schools the user belongs to (UUIDv7 strings). A
    /// cross-school user has more than one entry.
    #[serde(default)]
    pub schools: Vec<String>,
    /// The tenant for the current request (UUIDv7 string).
    pub active_school: String,
    /// Whether a second factor has been satisfied for this
    /// session.
    #[serde(default)]
    pub mfa: bool,
}

// ---------------------------------------------------------------------------
// JwtSecretSource
// ---------------------------------------------------------------------------

/// Where the HMAC signing key for [`JwtAuthProvider`] should be
/// loaded from.
///
/// This is the ADAPT-AUTH-002 surface: production deployments
/// must load the key from a stable, persistent source so that
/// every process restart (and every replica in a horizontally
/// scaled deployment) signs with the same key. The default
/// builder still generates a random key for tests and the
/// worked example; production wiring should use
/// [`JwtSecretSource::from_env`] via
/// [`JwtAuthProvider::new`] or pass an explicit
/// [`JwtSecretSource::EnvVar`] / [`JwtSecretSource::File`] to
/// [`JwtAuthProviderBuilder::signing_key_source`].
///
/// Resolution order in [`JwtSecretSource::from_env`]:
///
/// 1. The `JWT_SECRET` environment variable (raw key bytes).
/// 2. The `JWT_SECRET_FILE` environment variable (path to a
///    file containing the key bytes; useful for k8s `Secret`
///    mounts and Docker `--secret` mounts).
/// 3. [`JwtSecretSource::RandomDevFallback`] — a freshly
///    generated 32-byte key. **Debug builds only**: release
///    builds reject this fallback with [`AuthError::Malformed`]
///    so a misconfigured production deploy fails loudly rather
///    than silently signing tokens with a per-process random
///    key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JwtSecretSource {
    /// Read the key bytes from an environment variable (e.g.
    /// `JWT_SECRET=...`). The wrapped string is the secret
    /// value; it is converted to bytes verbatim by
    /// [`JwtSecretSource::into_bytes`].
    EnvVar(String),

    /// Read the key bytes from a file at the given path. Useful
    /// for Kubernetes `Secret` mounts and Docker `--secret`
    /// mounts where the secret is delivered as a file rather
    /// than an environment variable.
    File(PathBuf),

    /// Fall back to a freshly generated random 32-byte key.
    /// Debug builds accept this and emit a `tracing::warn!` so
    /// the developer sees the warning in their dev logs.
    /// Release builds reject this with [`AuthError::Malformed`]
    /// so a misconfigured production deploy fails loudly
    /// instead of silently invalidating every session on
    /// restart.
    RandomDevFallback,
}

impl JwtSecretSource {
    /// Resolves the secret source from environment variables.
    ///
    /// Order:
    ///
    /// 1. `JWT_SECRET` — if present, returns
    ///    [`JwtSecretSource::EnvVar`].
    /// 2. `JWT_SECRET_FILE` — if present, returns
    ///    [`JwtSecretSource::File`].
    /// 3. Otherwise, returns [`JwtSecretSource::RandomDevFallback`].
    ///
    /// This function does **not** read any files or generate any
    /// key material — it only inspects environment variables
    /// and returns the chosen variant. Call
    /// [`JwtSecretSource::into_bytes`] to materialise the bytes
    /// (which is where file reads and key generation happen).
    ///
    /// This is also the variant used by [`JwtAuthProvider::new`],
    /// so production wiring can rely on the same env-var
    /// convention in both the explicit-builder and default-
    /// constructor paths.
    #[must_use]
    pub fn from_env() -> Self {
        Self::from_env_with_reader(|key| std::env::var(key).ok())
    }

    /// Test-friendly variant of [`JwtSecretSource::from_env`]
    /// that accepts a custom env-var reader. Production code
    /// should call [`JwtSecretSource::from_env`]; tests use this
    /// to avoid mutating process-wide environment variables
    /// (which is `unsafe` in Rust 1.86+ and which this crate
    /// forbids via `#![forbid(unsafe_code)]`).
    #[must_use]
    pub fn from_env_with_reader(read: impl Fn(&str) -> Option<String>) -> Self {
        if let Some(value) = read("JWT_SECRET") {
            return Self::EnvVar(value);
        }
        if let Some(path) = read("JWT_SECRET_FILE") {
            return Self::File(PathBuf::from(path));
        }
        Self::RandomDevFallback
    }

    /// Resolves the source to its raw key bytes.
    ///
    /// - [`JwtSecretSource::EnvVar`]: returns the wrapped string
    ///   as bytes (UTF-8 conversion).
    /// - [`JwtSecretSource::File`]: reads the file at the given
    ///   path. Trailing whitespace is **not** trimmed (the
    ///   caller is responsible for producing a clean file). A
    ///   missing or unreadable file returns
    ///   [`AuthError::Infrastructure`].
    /// - [`JwtSecretSource::RandomDevFallback`]: in debug builds
    ///   generates 32 bytes from the OS RNG and emits a
    ///   `tracing::warn!` so the developer sees the warning. In
    ///   release builds returns [`AuthError::Malformed`] with a
    ///   message explaining that `JWT_SECRET` or
    ///   `JWT_SECRET_FILE` must be set.
    pub fn into_bytes(self) -> Result<Vec<u8>, AuthError> {
        match self {
            Self::EnvVar(value) => Ok(value.into_bytes()),
            Self::File(path) => std::fs::read(&path).map_err(|err| {
                AuthError::Infrastructure(InfrastructureError::with_source(
                    format!("failed to read JWT secret file {path:?}"),
                    Box::new(err),
                ))
            }),
            Self::RandomDevFallback => {
                #[cfg(debug_assertions)]
                {
                    tracing::warn!(
                        "JWT_SECRET and JWT_SECRET_FILE are unset; generating a \
                         random 32-byte signing key. This is suitable for dev/test \
                         only — every process restart will invalidate every existing \
                         JWT. Set JWT_SECRET (or JWT_SECRET_FILE) before deploying."
                    );
                    let mut key = vec![0_u8; 32];
                    rand::thread_rng().fill_bytes(&mut key);
                    Ok(key)
                }
                #[cfg(not(debug_assertions))]
                {
                    Err(AuthError::Malformed(
                        "JWT_SECRET (or JWT_SECRET_FILE) must be set in release \
                         builds; refusing to fall back to a random per-process key."
                            .to_owned(),
                    ))
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// JwtAuthProvider
// ---------------------------------------------------------------------------

/// The JWT-based [`AuthProvider`].
///
/// Holds the HMAC secret, issuer / audience configuration, TTLs,
/// and an in-memory revocation set keyed by `sid`. The struct is
/// `Clone`; the revocation set is shared via `Arc<Mutex<_>>` so
/// multiple clones observe the same revocations.
///
/// Object safety: the trait is object-safe; the concrete type
/// here is `Send + Sync + Debug`.
#[derive(Debug, Clone)]
pub struct JwtAuthProvider {
    signing_key: Arc<Vec<u8>>,
    issuer: Arc<String>,
    audience: Arc<String>,
    access_ttl_secs: i64,
    refresh_ttl_secs: i64,
    revoked_sessions: Arc<Mutex<HashSet<String>>>,
}

// ---------------------------------------------------------------------------
// JwtAuthProviderBuilder
// ---------------------------------------------------------------------------

/// Builder for [`JwtAuthProvider`]. See
/// `docs/ports/authentication.md` § "Configuration" for the
/// intended usage.
#[derive(Debug, Clone)]
pub struct JwtAuthProviderBuilder {
    signing_key: Vec<u8>,
    issuer: String,
    audience: String,
    access_ttl: Duration,
    refresh_ttl: Duration,
}

impl JwtAuthProviderBuilder {
    /// Constructs a builder with the defaults documented at
    /// `docs/ports/authentication.md`: 32-byte random signing
    /// key, issuer `"educore"`, audience `"educore"`,
    /// `access_ttl = 1h`, `refresh_ttl = 7d`.
    #[must_use]
    pub fn new() -> Self {
        let mut key = vec![0_u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        Self {
            signing_key: key,
            issuer: "educore".to_owned(),
            audience: "educore".to_owned(),
            access_ttl: Duration::from_secs(60 * 60),
            refresh_ttl: Duration::from_secs(60 * 60 * 24 * 7),
        }
    }

    /// Sets the HMAC signing key (raw bytes). Production wiring
    /// should pass `env::var("JWT_SECRET")?.into_bytes()`.
    #[must_use]
    pub fn signing_key(mut self, key: impl Into<Vec<u8>>) -> Self {
        self.signing_key = key.into();
        self
    }

    /// Resolves the HMAC signing key from a [`JwtSecretSource`]
    /// (env var, file path, or random dev fallback) and
    /// installs it on the builder.
    ///
    /// This is the ADAPT-AUTH-002 surface: production code
    /// should pass [`JwtSecretSource::from_env`] here so the
    /// key is loaded from `JWT_SECRET` / `JWT_SECRET_FILE`
    /// rather than the per-process random default that
    /// [`JwtAuthProviderBuilder::new`] generates.
    ///
    /// Returns the builder on success so the calls chain.
    /// Returns [`AuthError::Malformed`] if the resolved key is
    /// shorter than 32 bytes (HS256 requires ≥256-bit keys per
    /// RFC 7518 § 3.2). Returns [`AuthError::Infrastructure`]
    /// if the source is [`JwtSecretSource::File`] and the file
    /// cannot be read. Returns [`AuthError::Malformed`] in
    /// release builds if the source falls back to
    /// [`JwtSecretSource::RandomDevFallback`].
    ///
    /// Errors are propagated via `Result` — this method does
    /// **not** panic on a missing or unreadable secret.
    pub fn signing_key_source(mut self, source: JwtSecretSource) -> Result<Self, AuthError> {
        let bytes = source.into_bytes()?;
        if bytes.len() < 32 {
            return Err(AuthError::Malformed(format!(
                "JWT signing key is {} bytes; HS256 requires at least 32 bytes \
                 (RFC 7518 § 3.2)",
                bytes.len()
            )));
        }
        self.signing_key = bytes;
        Ok(self)
    }

    /// Sets the issuer (`iss` claim). The provider rejects
    /// tokens whose `iss` does not match.
    #[must_use]
    pub fn issuer(mut self, issuer: impl Into<String>) -> Self {
        self.issuer = issuer.into();
        self
    }

    /// Sets the audience (`aud` claim). The provider rejects
    /// tokens whose `aud` does not match.
    #[must_use]
    pub fn audience(mut self, audience: impl Into<String>) -> Self {
        self.audience = audience.into();
        self
    }

    /// Sets the access-token TTL (the `exp` claim is set to
    /// `iat + access_ttl`).
    #[must_use]
    pub fn access_ttl(mut self, ttl: Duration) -> Self {
        self.access_ttl = ttl;
        self
    }

    /// Sets the refresh-token TTL. Currently informational: the
    /// reference provider does not mint refresh tokens distinct
    /// from access tokens, but the TTL is exposed so consumers
    /// can pick it up for their own refresh logic.
    #[must_use]
    pub fn refresh_ttl(mut self, ttl: Duration) -> Self {
        self.refresh_ttl = ttl;
        self
    }

    /// Consumes the builder and returns the configured
    /// provider.
    #[must_use]
    pub fn build(self) -> JwtAuthProvider {
        JwtAuthProvider {
            signing_key: Arc::new(self.signing_key),
            issuer: Arc::new(self.issuer),
            audience: Arc::new(self.audience),
            access_ttl_secs: i64::try_from(self.access_ttl.as_secs()).unwrap_or(i64::MAX),
            refresh_ttl_secs: i64::try_from(self.refresh_ttl.as_secs()).unwrap_or(i64::MAX),
            revoked_sessions: Arc::new(Mutex::new(HashSet::new())),
        }
    }
}

impl Default for JwtAuthProviderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

impl JwtAuthProvider {
    /// Constructs a provider that loads its HMAC signing key
    /// from [`JwtSecretSource::from_env`] (i.e. `JWT_SECRET`
    /// first, then `JWT_SECRET_FILE`, then a dev-only random
    /// fallback).
    ///
    /// This is the recommended way to construct a
    /// [`JwtAuthProvider`] in production code: it ensures every
    /// process restart (and every replica in a horizontally
    /// scaled deployment) signs with the same key, fixing
    /// ADAPT-AUTH-002 (per-process random key invalidating all
    /// sessions on restart).
    ///
    /// Returns `Err` rather than panicking if:
    ///
    /// - the `JWT_SECRET_FILE` path cannot be read;
    /// - the resolved key is shorter than 32 bytes (HS256
    ///   minimum-length, RFC 7518 § 3.2);
    /// - no env var is set and the build is release (no
    ///   random-fallback allowed in release).
    pub fn new() -> Result<Self, AuthError> {
        Ok(JwtAuthProviderBuilder::new()
            .signing_key_source(JwtSecretSource::from_env())?
            .build())
    }
}

impl JwtAuthProvider {
    /// Builds the `jsonwebtoken::Validation` for this provider's
    /// configuration. Helper to keep the per-method bodies short.
    fn validation(&self) -> Validation {
        let mut v = Validation::new(Algorithm::HS256);
        v.set_issuer(&[self.issuer.as_str()]);
        v.set_audience(&[self.audience.as_str()]);
        v.validate_exp = true;
        v.leeway = 0;
        v.required_spec_claims = ["exp", "iss", "aud", "sub"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        v
    }

    /// Encodes the given claims with the configured HMAC key.
    /// Returns `AuthError::Malformed` if encoding fails (the
    /// only realistic cause is a serde failure on the claims
    /// themselves, which is a programmer error).
    fn encode(&self, claims: &JwtClaims) -> Result<String, AuthError> {
        let key = EncodingKey::from_secret(self.signing_key.as_slice());
        encode(&Header::new(Algorithm::HS256), claims, &key)
            .map_err(|e| AuthError::Malformed(format!("jwt encode failed: {e}")))
    }

    /// Decodes and verifies the given JWT string, returning the
    /// claims on success. Maps every `jsonwebtoken::Error` to the
    /// closest [`AuthError`] variant.
    fn decode(&self, token: &str) -> Result<JwtClaims, AuthError> {
        let key = DecodingKey::from_secret(self.signing_key.as_slice());
        let validation = self.validation();
        decode::<JwtClaims>(token, &key, &validation)
            .map(|data| data.claims)
            .map_err(map_jwt_error)
    }

    /// Builds a [`Session`] from verified claims. Centralised so
    /// `validate` and `refresh` produce identical sessions for
    /// identical claims.
    fn session_from_claims(&self, claims: &JwtClaims) -> Result<Session, AuthError> {
        let session_id = SessionId::from_uuid(parse_uuid(&claims.sid, "sid")?);
        let user_id = UserId::from_uuid(parse_uuid(&claims.sub, "sub")?);
        let active_school_id =
            SchoolId::from_uuid(parse_uuid(&claims.active_school, "active_school")?);

        let school_ids = claims
            .schools
            .iter()
            .map(|s| {
                let uuid = parse_uuid(s, "schools")?;
                Ok(SchoolId::from_uuid(uuid))
            })
            .collect::<Result<Vec<_>, AuthError>>()?;

        let roles = claims
            .roles
            .iter()
            .map(|r| {
                let uuid = parse_uuid(r, "roles")?;
                Ok(RoleId::new(active_school_id, uuid))
            })
            .collect::<Result<Vec<_>, AuthError>>()?;

        let issued_at = unix_secs_to_timestamp(claims.iat, "iat")?;
        let expires_at = unix_secs_to_timestamp(claims.exp, "exp")?;

        Ok(Session {
            session_id,
            user_id,
            school_ids,
            active_school_id,
            roles,
            capabilities: BTreeSet::<Capability>::new(),
            mfa_satisfied: claims.mfa,
            issued_at,
            expires_at,
            metadata: BTreeMap::new(),
        })
    }

    /// Builds a placeholder dev session for
    /// [`Credential::Anonymous`]. Issued against the platform
    /// school with the [`educore_core::ids::SYSTEM_USER_ID`]
    /// actor and the configured access TTL.
    #[allow(dead_code)] // Reserved for Credential::Anonymous path; see FND-SEC-AUTH-001
    fn anonymous_session(&self) -> Session {
        let now = Timestamp::now();
        let exp = Timestamp::from_datetime(
            Utc.timestamp_opt(now.as_datetime().timestamp() + self.access_ttl_secs, 0)
                .single()
                .unwrap_or_else(|| now.as_datetime()),
        );
        Session {
            session_id: SessionId::from_uuid(Uuid::now_v7()),
            user_id: educore_core::ids::SYSTEM_USER_ID,
            school_ids: vec![PUBLIC_SCHOOL_ID],
            active_school_id: PUBLIC_SCHOOL_ID,
            roles: Vec::new(),
            capabilities: BTreeSet::<Capability>::new(),
            mfa_satisfied: true,
            issued_at: now,
            expires_at: exp,
            metadata: BTreeMap::new(),
        }
    }

    /// Mints a fresh JWT and [`Session`] for a service principal.
    /// Used by [`crate::api_key::ApiKeyAuthProvider`] after a
    /// successful API-key validation.
    ///
    /// The `service_id` is used to derive a deterministic
    /// UUIDv5 user id so each service principal has a stable
    /// identity across sessions (the same `service_id` always
    /// yields the same `user_id`, which makes audit
    /// traceability trivial). The session is anchored to the
    /// public school (services are not anchored to a single
    /// tenant) and carries no roles.
    #[allow(dead_code)] // Used by crate::api_key::ApiKeyAuthProvider; see PORT-AUTH-APIKEY
    pub(crate) fn mint_session_for_service(&self, service_id: &str) -> Result<Session, AuthError> {
        let now = Timestamp::now();
        let exp = Timestamp::from_datetime(
            Utc.timestamp_opt(now.as_datetime().timestamp() + self.access_ttl_secs, 0)
                .single()
                .unwrap_or_else(|| now.as_datetime()),
        );
        // UUIDv5 over the OID namespace keyed by the service
        // identifier — stable per service across processes.
        let service_user_id =
            UserId::from_uuid(Uuid::new_v5(&Uuid::NAMESPACE_OID, service_id.as_bytes()));
        Ok(Session {
            session_id: SessionId::from_uuid(Uuid::now_v7()),
            user_id: service_user_id,
            school_ids: vec![PUBLIC_SCHOOL_ID],
            active_school_id: PUBLIC_SCHOOL_ID,
            roles: Vec::new(),
            capabilities: BTreeSet::<Capability>::new(),
            mfa_satisfied: true,
            issued_at: now,
            expires_at: exp,
            metadata: BTreeMap::new(),
        })
    }

    /// Locks the revocation set and inserts the given `sid`.
    /// Used by both `revoke` and `refresh`.
    fn add_revoked(&self, sid: &str) {
        let mut set = self
            .revoked_sessions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        set.insert(sid.to_owned());
    }

    /// Returns `Err(AuthError::Revoked)` if the given `sid` has
    /// been revoked. Otherwise returns `Ok(())`.
    fn check_not_revoked(&self, sid: &str) -> Result<(), AuthError> {
        let set = self
            .revoked_sessions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if set.contains(sid) {
            return Err(AuthError::Revoked);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// AuthProvider impl
// ---------------------------------------------------------------------------

#[async_trait]
impl AuthProvider for JwtAuthProvider {
    async fn authenticate(&self, credential: Credential) -> Result<Session, AuthError> {
        match credential {
            Credential::Bearer(token) => {
                let claims = self.decode(&token)?;
                self.check_not_revoked(&claims.sid)?;
                self.session_from_claims(&claims)
            }
            // FND-SEC-AUTH-001: reject anonymous auth by default.
            // Callers needing an anonymous session should construct one
            // explicitly via SessionService::bootstrap_anonymous() (a
            // deliberate opt-in path). This prevents the prior auth bypass
            // where Credential::Anonymous returned a full Session with
            // mfa_enabled=false.
            Credential::Anonymous => Err(AuthError::InvalidCredentials),
            Credential::UsernamePassword { .. }
            | Credential::Oauth2 { .. }
            | Credential::Saml { .. }
            | Credential::ApiKey { .. }
            | Credential::Biometric { .. } => Err(AuthError::InvalidCredentials),
        }
    }

    async fn validate(&self, token: &AuthToken) -> Result<Session, AuthError> {
        if !matches!(token.scheme, AuthScheme::Bearer) {
            return Err(AuthError::Malformed(format!(
                "JwtAuthProvider only accepts Bearer tokens, got {:?}",
                token.scheme
            )));
        }
        let claims = self.decode(&token.value)?;
        self.check_not_revoked(&claims.sid)?;
        self.session_from_claims(&claims)
    }

    async fn revoke(&self, token: &AuthToken) -> Result<(), AuthError> {
        if !matches!(token.scheme, AuthScheme::Bearer) {
            return Err(AuthError::Malformed(format!(
                "JwtAuthProvider only accepts Bearer tokens, got {:?}",
                token.scheme
            )));
        }
        let claims = self.decode(&token.value)?;
        self.add_revoked(&claims.sid);
        Ok(())
    }

    async fn refresh(&self, token: &AuthToken) -> Result<Session, AuthError> {
        if !matches!(token.scheme, AuthScheme::Bearer) {
            return Err(AuthError::Malformed(format!(
                "JwtAuthProvider only accepts Bearer tokens, got {:?}",
                token.scheme
            )));
        }
        let old_claims = self.decode(&token.value)?;
        // Validate the OLD token (signature, exp, not revoked).
        // The decoder already enforced exp + iss + aud, but the
        // revocation check is explicit here because the new
        // session will reuse the same `sid`.
        self.check_not_revoked(&old_claims.sid)?;

        let now = Utc::now().timestamp();
        let new_claims = JwtClaims {
            sub: old_claims.sub,
            iss: old_claims.iss,
            aud: old_claims.aud,
            iat: now,
            exp: now.saturating_add(self.access_ttl_secs),
            // Same sid so the refreshed token belongs to the
            // same logical session. We do NOT add the sid to
            // the revocation set on refresh; instead, callers
            // who want strict token rotation should call
            // `revoke` on the old token explicitly.
            sid: old_claims.sid,
            roles: old_claims.roles,
            schools: old_claims.schools,
            active_school: old_claims.active_school,
            mfa: old_claims.mfa,
        };

        // Suppress the unused-field warning when `refresh_ttl_secs`
        // is not consumed by this minimal implementation.
        let _ = self.refresh_ttl_secs;

        // Touch the encoder so future revisions can plumb the
        // refreshed token back through the AuthToken channel
        // without re-deriving the encoding path.
        let _refreshed_token = self.encode(&new_claims)?;
        self.session_from_claims(&new_claims)
    }
}

// ---------------------------------------------------------------------------
// Free helpers
// ---------------------------------------------------------------------------

/// Parses a UUID string, tagging the failure with the claim
/// name so the resulting [`AuthError::Malformed`] is actionable.
fn parse_uuid(s: &str, claim: &str) -> Result<Uuid, AuthError> {
    Uuid::parse_str(s)
        .map_err(|e| AuthError::Malformed(format!("claim {claim:?} is not a valid UUID: {e}")))
}

/// Converts a Unix-seconds value to [`Timestamp`]. Out-of-range
/// values (negative or beyond chrono's `TimestampMax`) return
/// [`AuthError::Malformed`].
fn unix_secs_to_timestamp(secs: i64, claim: &str) -> Result<Timestamp, AuthError> {
    if secs < 0 {
        return Err(AuthError::Malformed(format!(
            "claim {claim:?} is negative ({secs}); expected Unix seconds"
        )));
    }
    match Utc.timestamp_opt(secs, 0) {
        chrono::LocalResult::Single(dt) => Ok(Timestamp::from_datetime(dt)),
        other => Err(AuthError::Malformed(format!(
            "claim {claim:?} ({secs}) cannot be represented as a UTC timestamp ({other:?})"
        ))),
    }
}

/// Maps every [`jsonwebtoken::Error`] variant to the closest
/// [`AuthError`] variant. The provider deliberately keeps the
/// mapping local (rather than introducing a `From` impl) so the
/// `jsonwebtoken` types do not leak into the public surface.
fn map_jwt_error(err: jsonwebtoken::errors::Error) -> AuthError {
    use jsonwebtoken::errors::ErrorKind;
    match err.kind() {
        ErrorKind::ExpiredSignature => AuthError::Expired,
        ErrorKind::InvalidSignature
        | ErrorKind::InvalidAlgorithm
        | ErrorKind::InvalidAlgorithmName
        | ErrorKind::InvalidKeyFormat
        | ErrorKind::InvalidEcdsaKey
        | ErrorKind::InvalidRsaKey(_) => {
            AuthError::Malformed(format!("jwt signature/algorithm: {err}"))
        }
        ErrorKind::InvalidIssuer
        | ErrorKind::InvalidAudience
        | ErrorKind::InvalidSubject
        | ErrorKind::ImmatureSignature
        | ErrorKind::MissingRequiredClaim(_)
        | ErrorKind::InvalidToken
        | ErrorKind::MissingAlgorithm => AuthError::Malformed(format!("jwt claim: {err}")),
        // ErrorKind is `#[non_exhaustive]`; catch any future variant
        // as Malformed so the match stays exhaustive without panicking.
        _ => AuthError::Malformed(format!("jwt: {err}")),
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
    clippy::missing_docs_in_private_items
)]
mod tests {
    use super::*;
    use crate::port::BearerToken;

    fn sample_claims(sid: Uuid, user: Uuid, school: Uuid) -> JwtClaims {
        let now = Utc::now().timestamp();
        JwtClaims {
            sub: user.to_string(),
            iss: "educore-test".to_owned(),
            aud: "educore".to_owned(),
            iat: now,
            exp: now + 3600,
            sid: sid.to_string(),
            roles: vec![],
            schools: vec![school.to_string()],
            active_school: school.to_string(),
            mfa: false,
        }
    }

    fn provider_with_key(key: &[u8]) -> JwtAuthProvider {
        JwtAuthProviderBuilder::new()
            .signing_key(key.to_vec())
            .issuer("educore-test")
            .audience("educore")
            .access_ttl(Duration::from_secs(3600))
            .refresh_ttl(Duration::from_secs(7 * 24 * 3600))
            .build()
    }

    #[test]
    fn test_jwt_provider_builder_constructs_with_defaults() {
        let provider = JwtAuthProviderBuilder::new().build();
        // Defaults: issuer = audience = "educore", 32-byte key.
        assert_eq!(&*provider.issuer, "educore");
        assert_eq!(&*provider.audience, "educore");
        assert_eq!(provider.signing_key.len(), 32);
        assert_eq!(provider.access_ttl_secs, 3600);
        assert_eq!(provider.refresh_ttl_secs, 7 * 24 * 3600);
        assert!(provider
            .revoked_sessions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_empty());
    }

    #[tokio::test]
    async fn test_jwt_provider_round_trip_session() {
        let key = b"test-key-for-round-trip-only-32b!!".to_vec();
        let provider = provider_with_key(&key);

        let sid = Uuid::now_v7();
        let user = Uuid::now_v7();
        let school = Uuid::now_v7();
        let claims = sample_claims(sid, user, school);

        let token_str = provider.encode(&claims).expect("encode succeeds");
        let bearer = BearerToken::from(token_str.clone());
        let auth_token = AuthToken {
            scheme: AuthScheme::Bearer,
            value: token_str,
            metadata: BTreeMap::new(),
        };

        // Authenticate via Credential::Bearer produces the same session.
        let session_via_authn = provider
            .authenticate(Credential::Bearer(bearer.clone()))
            .await
            .expect("authenticate succeeds");

        // Validate via AuthToken also produces the same session.
        let session_via_validate = provider
            .validate(&auth_token)
            .await
            .expect("validate succeeds");

        assert_eq!(session_via_authn, session_via_validate);
        assert_eq!(session_via_authn.session_id.as_uuid(), sid);
        assert_eq!(session_via_authn.user_id.as_uuid(), user);
        assert_eq!(session_via_authn.active_school_id.as_uuid(), school);
        assert_eq!(session_via_authn.school_ids.len(), 1);
        assert_eq!(session_via_authn.school_ids[0].as_uuid(), school);
        assert!(session_via_authn.capabilities.is_empty());
        assert!(!session_via_authn.mfa_satisfied);
        // Issued / expiry in the future.
        assert!(
            session_via_authn.expires_at.as_datetime() > session_via_authn.issued_at.as_datetime()
        );
    }

    #[tokio::test]
    async fn test_jwt_provider_revoked_token_returns_revoked_error() {
        let key = b"test-key-for-revocation-only-32b!".to_vec();
        let provider = provider_with_key(&key);

        let sid = Uuid::now_v7();
        let user = Uuid::now_v7();
        let school = Uuid::now_v7();
        let claims = sample_claims(sid, user, school);
        let token_str = provider.encode(&claims).expect("encode succeeds");
        let auth_token = AuthToken {
            scheme: AuthScheme::Bearer,
            value: token_str,
            metadata: BTreeMap::new(),
        };

        // Before revoke: validates fine.
        provider
            .validate(&auth_token)
            .await
            .expect("pre-revoke validate succeeds");

        // Revoke.
        provider.revoke(&auth_token).await.expect("revoke succeeds");

        // After revoke: AuthError::Revoked.
        let err = provider
            .validate(&auth_token)
            .await
            .expect_err("post-revoke validate fails");
        assert_eq!(err, AuthError::Revoked);

        // And authenticate(Bearer) also rejects.
        let bearer = BearerToken::from(auth_token.value.clone());
        let authn_err = provider
            .authenticate(Credential::Bearer(bearer))
            .await
            .expect_err("post-revoke authenticate fails");
        assert_eq!(authn_err, AuthError::Revoked);
    }

    // -----------------------------------------------------------------
    // QW-7: JwtSecretSource env-loading tests.
    //
    // We exercise the resolver via `from_env_with_reader` so the
    // tests don't mutate process-wide environment variables
    // (`std::env::set_var` is `unsafe` from Rust 1.86 and the crate
    // forbids `unsafe`). Production callers use `from_env`, which is
    // a thin wrapper that delegates to `from_env_with_reader`.
    // -----------------------------------------------------------------

    #[test]
    fn test_jwt_secret_source_from_env_reads_jwt_secret_var() {
        let source = JwtSecretSource::from_env_with_reader(|key| match key {
            "JWT_SECRET" => Some("a".repeat(32)),
            _ => None,
        });
        assert_eq!(source, JwtSecretSource::EnvVar("a".repeat(32)));
    }

    #[test]
    fn test_jwt_secret_source_from_env_reads_jwt_secret_file_var() {
        let source = JwtSecretSource::from_env_with_reader(|key| match key {
            "JWT_SECRET" => None,
            "JWT_SECRET_FILE" => Some("/run/secrets/jwt".to_owned()),
            _ => None,
        });
        assert_eq!(
            source,
            JwtSecretSource::File(PathBuf::from("/run/secrets/jwt"))
        );
    }

    #[test]
    fn test_jwt_secret_source_from_env_prefers_jwt_secret_over_file() {
        // Both set: JWT_SECRET wins.
        let source = JwtSecretSource::from_env_with_reader(|key| match key {
            "JWT_SECRET" => Some("a".repeat(32)),
            "JWT_SECRET_FILE" => Some("/run/secrets/jwt".to_owned()),
            _ => None,
        });
        assert_eq!(source, JwtSecretSource::EnvVar("a".repeat(32)));
    }

    #[test]
    fn test_jwt_secret_source_from_env_falls_back_to_random_dev() {
        // Neither env var set: RandomDevFallback.
        let source = JwtSecretSource::from_env_with_reader(|_| None);
        assert_eq!(source, JwtSecretSource::RandomDevFallback);
    }

    #[test]
    fn test_jwt_secret_source_env_var_into_bytes_returns_raw_bytes() {
        let bytes = JwtSecretSource::EnvVar("a".repeat(32))
            .into_bytes()
            .expect("env var resolves to bytes");
        assert_eq!(bytes.len(), 32);
        assert!(bytes.iter().all(|b| *b == b'a'));
    }

    #[test]
    fn test_jwt_secret_source_file_into_bytes_reads_file() {
        // Write a temp file with 32 bytes of 'b'.
        let dir =
            std::env::temp_dir().join(format!("educore-auth-jwt-secret-test-{}", Uuid::now_v7()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join("secret");
        let contents = "b".repeat(32).into_bytes();
        std::fs::write(&path, &contents).expect("write temp file");

        let bytes = JwtSecretSource::File(path.clone())
            .into_bytes()
            .expect("file resolves to bytes");
        assert_eq!(bytes, contents);

        // Cleanup.
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_jwt_secret_source_file_into_bytes_missing_file_errors() {
        let bogus = PathBuf::from("/nonexistent/educore-auth-missing-secret-xyzzy");
        let err = JwtSecretSource::File(bogus.clone())
            .into_bytes()
            .expect_err("missing file must error, not panic");
        assert!(
            matches!(err, AuthError::Infrastructure(_)),
            "expected Infrastructure error, got {err:?}"
        );
    }

    #[test]
    fn test_jwt_secret_source_random_dev_fallback_in_debug_emits_random_bytes() {
        // This is debug-only: in release builds the fallback errors
        // so a misconfigured production deploy fails loudly.
        #[cfg(debug_assertions)]
        {
            let a = JwtSecretSource::RandomDevFallback
                .into_bytes()
                .expect("debug build falls back to random");
            let b = JwtSecretSource::RandomDevFallback
                .into_bytes()
                .expect("debug build falls back to random");
            assert_eq!(a.len(), 32);
            assert_eq!(b.len(), 32);
            assert_ne!(
                a, b,
                "two consecutive fallbacks must produce different bytes"
            );
        }
    }

    #[test]
    fn test_signing_key_source_env_var_applies_bytes_to_builder() {
        let key_bytes: Vec<u8> = (0..32).collect();
        let provider = JwtAuthProviderBuilder::new()
            .signing_key_source(JwtSecretSource::EnvVar(
                String::from_utf8(key_bytes.clone()).expect("ascii"),
            ))
            .expect("valid key applies to builder")
            .issuer("educore-test")
            .audience("educore")
            .build();
        assert_eq!(&*provider.signing_key, &key_bytes[..]);
    }

    #[test]
    fn test_signing_key_source_rejects_short_key() {
        let short = JwtSecretSource::EnvVar("too-short".to_owned());
        let err = JwtAuthProviderBuilder::new()
            .signing_key_source(short)
            .expect_err("HS256 requires >= 32 bytes");
        assert!(
            matches!(err, AuthError::Malformed(_)),
            "expected Malformed error, got {err:?}"
        );
    }

    #[test]
    fn test_signing_key_source_random_dev_fallback_errors_in_release() {
        // Mirror the release-mode behaviour: a release build must
        // not silently accept the random fallback. We branch on the
        // same cfg the production code uses.
        #[cfg(not(debug_assertions))]
        {
            let err = JwtAuthProviderBuilder::new()
                .signing_key_source(JwtSecretSource::RandomDevFallback)
                .expect_err("release must refuse random fallback");
            assert!(
                matches!(err, AuthError::Malformed(_)),
                "expected Malformed error, got {err:?}"
            );
        }
    }

    #[test]
    fn test_signing_key_source_file_missing_returns_error_not_panic() {
        // Regression for the previous two-agent attempt: the bug
        // there was `unwrap_or_else(resolve_err_panic)` — a function
        // that doesn't exist. The new path propagates the I/O
        // failure as AuthError::Infrastructure via `?`, never panics.
        let bogus = PathBuf::from("/nonexistent/educore-auth-file-xyzzy");
        let result = JwtAuthProviderBuilder::new().signing_key_source(JwtSecretSource::File(bogus));
        assert!(
            matches!(result, Err(AuthError::Infrastructure(_))),
            "expected Infrastructure error from missing file, got {result:?}"
        );
    }
}
