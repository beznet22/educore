//! # educore-auth port
//!
//! The authentication port defines how the engine obtains a
//! session for an incoming request. The engine does not own user
//! credentials, password hashing, OAuth flows, or session
//! storage; the consumer supplies an adapter that implements
//! [`AuthProvider`] and produces a [`Session`] value.
//!
//! This module is the **port-only** surface. Reference
//! implementations (JWT, local-password, OAuth2, SAML, API-key)
//! land in separate microtasks; see `docs/build-plan.md` § "Phase
//! 15" for the split.
//!
//! See `docs/ports/authentication.md` for the authoritative
//! specification.
//!
//! # Deviations from `docs/ports/authentication.md`
//!
//! The crate's `Cargo.toml` is intentionally minimal (only
//! `core`, `platform`, `rbac`, `events`, `tokio`, `async-trait`),
//! so the port uses **stdlib-only** value representations:
//!
//! - All opaque ID / secret newtypes are represented as `String`
//!   (or newtype wrappers around `String`). Adapters that need
//!   a richer type (UUID, `secrecy::SecretString`,
//!   `url::Url`, `serde_json::Value`) parse the inner string at
//!   their boundary.
//! - `Oauth2::redirect_uri` is a `String` (URL-formatted UTF-8)
//!   rather than `url::Url`. The adapter validates the URL
//!   shape at its boundary.
//! - `BearerToken` is `pub type BearerToken = String;` rather
//!   than `pub type BearerToken = secrecy::SecretString;`. The
//!   adapter MUST redact any string whose field name ends in
//!   `_token`, `_secret`, `_password`, or `_key` before the
//!   value reaches the audit log.
//! - The port types do not derive `Serialize` or `Deserialize`.
//!   Adapters that need to cross a wire boundary (e.g. a
//!   cross-process token cache) implement their own wire
//!   format.
//!
//! These deviations match the `educore-payment` port pattern
//! and are documented here so future ports that gain richer
//! dependencies can adopt the spec's idiomatic types without
//! changing the trait surface.

#![allow(dead_code, clippy::all, missing_docs)]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use async_trait::async_trait;

use educore_core::ids::{SchoolId, SessionId, UserId};
use educore_core::value_objects::Timestamp;
use educore_rbac::ids::RoleId;
use educore_rbac::value_objects::Capability;

// ---------------------------------------------------------------------------
// Bearer token alias
// ---------------------------------------------------------------------------

/// An opaque bearer-token string.
///
/// `BearerToken` is a type alias for `String` so that the wire
/// form (`Authorization: Bearer <token>`) is **statically
/// distinct** from raw `String` uses in other contexts (e.g.
/// API keys, passwords). Adapters that need a richer type
/// (`secrecy::SecretString`) can convert at their boundary;
/// the `Debug` impl does **not** auto-redact the value (the
/// adapter is responsible for redaction before logging).
pub type BearerToken = String;

// ---------------------------------------------------------------------------
// Session
// ---------------------------------------------------------------------------

/// The result of a successful authentication.
///
/// `Session` is a value type. It carries everything the engine
/// needs to authorize and tenant-isolate a command.
/// Capabilities are pre-computed when the session is created;
/// the engine does not consult the RBAC storage on every
/// command.
///
/// Per `docs/ports/authentication.md` § "Session":
/// - `school_ids` is the set of schools the user belongs to (a
///   parent may have children in two schools);
/// - `active_school_id` is the tenant for the current request
///   and is the one selected by `TenantContext`;
/// - `mfa_satisfied` is `false` for a "pending MFA" session;
///   the engine restricts sensitive commands when it is `false`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Session {
    /// Unique session identifier (UUIDv7).
    pub session_id: SessionId,
    /// The authenticated principal.
    pub user_id: UserId,
    /// The schools the user belongs to. A user may belong to
    /// multiple schools; the engine rejects commands that
    /// target the inactive schools.
    pub school_ids: Vec<SchoolId>,
    /// The tenant for the current request. A "switch school"
    /// action in the consumer application changes this value
    /// and the engine re-validates capabilities for the new
    /// school.
    pub active_school_id: SchoolId,
    /// The roles the user holds, scoped to
    /// `active_school_id`.
    pub roles: Vec<RoleId>,
    /// The pre-computed capability set for this session. The
    /// engine does not consult the RBAC store on every command.
    pub capabilities: BTreeSet<Capability>,
    /// `true` once a second factor has been presented and
    /// accepted. Sensitive commands require this to be `true`.
    pub mfa_satisfied: bool,
    /// When the session was issued (RFC 3339 UTC).
    pub issued_at: Timestamp,
    /// When the session expires. After this instant the
    /// adapter returns [`crate::errors::AuthError::Expired`].
    pub expires_at: Timestamp,
    /// Free-form metadata (e.g. device fingerprint, IP,
    /// locale). Carried for audit and consumer-side
    /// customisation; the engine does not interpret the keys.
    pub metadata: BTreeMap<String, String>,
}

// ---------------------------------------------------------------------------
// AuthScheme + AuthToken
// ---------------------------------------------------------------------------

/// The transport scheme used to carry an [`AuthToken`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuthScheme {
    /// `Authorization: Bearer <token>` (RFC 6750).
    Bearer,
    /// A session cookie (e.g. `Cookie: session=<value>`).
    Cookie,
    /// A consumer-defined scheme (e.g. `Authorization: ApiKey
    /// <id>.<secret>`). The wrapped string is the scheme
    /// label used in the wire form.
    Custom(&'static str),
}

/// The opaque credential carried by an incoming request.
///
/// `AuthToken` is the validation-time counterpart of
/// [`Credential`]: the consumer middleware extracts the
/// `Authorization` header (or cookie) and hands the resulting
/// `AuthToken` to [`AuthProvider::validate`]. The adapter is
/// responsible for verifying the signature and producing a
/// [`Session`].
///
/// The `value` field is a plain `String` (not a
/// `secrecy::SecretString`) per the crate's stdlib-only
/// policy. Adapters that need a redacting wrapper convert
/// at their boundary.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AuthToken {
    /// The transport scheme (`Bearer`, `Cookie`, or a
    /// consumer-defined label).
    pub scheme: AuthScheme,
    /// The opaque token value.
    pub value: String,
    /// Free-form metadata (e.g. `issued_at` hint, device
    /// fingerprint, audit trail).
    pub metadata: BTreeMap<String, String>,
}

// ---------------------------------------------------------------------------
// Credential
// ---------------------------------------------------------------------------

/// A presented authentication credential.
///
/// `Credential::Anonymous` is rejected by the default adapters
/// except in public-facing flows (e.g. public exam result
/// lookup, when explicitly allowed by configuration). See
/// `docs/ports/authentication.md` § "Credential".
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Credential {
    /// A raw bearer token (e.g. an opaque cookie / JWT).
    Bearer(BearerToken),

    /// A username + password combination. The password is a
    /// plain `String` (not a `SecretString`) per the crate's
    /// stdlib-only policy. Adapters MUST redact the password
    /// before it reaches any audit log.
    UsernamePassword {
        /// The user identifier (username, email, or login id —
        /// adapter-specific).
        username: String,
        /// The plaintext password. Never logged; never carried
        /// beyond the adapter.
        password: String,
    },

    /// An OAuth 2.0 / OpenID Connect authorization code.
    ///
    /// `redirect_uri` is a `String` (not a `url::Url`) per the
    /// crate's stdlib-only policy. The adapter validates the
    /// URL shape at its boundary.
    Oauth2 {
        /// The authorization code issued by the IdP.
        code: String,
        /// The redirect URI registered with the OAuth client.
        redirect_uri: String,
        /// PKCE code verifier (RFC 7636), if the client used
        /// it.
        code_verifier: Option<String>,
    },

    /// A SAML 2.0 response (assertion).
    Saml {
        /// The base64-encoded SAML assertion.
        assertion: String,
        /// The SAML `RelayState` parameter, if present.
        relay_state: Option<String>,
    },

    /// A pre-shared API key (service-to-service auth).
    ApiKey {
        /// The public key id (e.g. `ak_01HXY...`).
        id: String,
        /// The secret key value.
        key: String,
    },

    /// A biometric / device-bound signature (WebAuthn, FIDO2,
    /// device fingerprint).
    Biometric {
        /// The device identifier (e.g. the WebAuthn credential
        /// id, or a stable device fingerprint).
        device_id: String,
        /// The signed challenge. Adapter-specific format
        /// (raw bytes, CBOR, base64-decoded).
        signature: Vec<u8>,
        /// When the signature was produced. Used by the
        /// adapter to enforce a freshness window.
        timestamp: Timestamp,
    },

    /// An unauthenticated request. Rejected by every default
    /// adapter except public-facing flows.
    Anonymous,
}

// ---------------------------------------------------------------------------
// AuthProvider
// ---------------------------------------------------------------------------

/// The authentication port.
///
/// Adapters that produce a [`Session`] for an incoming request
/// implement this trait and are wired into the engine at startup
/// (see `docs/ports/authentication.md` § "Configuration").
///
/// The trait is object-safe: every method takes `&self` and the
/// adapter is `Send + Sync + Debug`.
#[async_trait]
pub trait AuthProvider: Send + Sync + std::fmt::Debug {
    /// Exchange a [`Credential`] for a [`Session`].
    ///
    /// Returns [`crate::errors::AuthError::MfaRequired`] if a
    /// second factor must be presented; the consumer collects
    /// the factor and calls `authenticate` again with a
    /// credential that includes the factor response.
    async fn authenticate(
        &self,
        credential: Credential,
    ) -> Result<Session, crate::errors::AuthError>;

    /// Validate an existing [`AuthToken`] and return the
    /// corresponding [`Session`].
    ///
    /// The adapter may cache token-to-session mappings but
    /// MUST verify the token's signature or validity on each
    /// call.
    async fn validate(&self, token: &AuthToken) -> Result<Session, crate::errors::AuthError>;

    /// Invalidate an [`AuthToken`]. Subsequent `validate`
    /// calls for the same token return
    /// [`crate::errors::AuthError::Revoked`].
    async fn revoke(&self, token: &AuthToken) -> Result<(), crate::errors::AuthError>;

    /// Issue a new [`Session`] for a non-expired [`AuthToken`].
    /// The adapter may rotate the token value; the old token
    /// is invalidated.
    async fn refresh(&self, token: &AuthToken) -> Result<Session, crate::errors::AuthError>;
}

// ---------------------------------------------------------------------------
// RbacPort
// ---------------------------------------------------------------------------

/// The capability-check port.
///
/// The engine calls [`RbacPort::require`] at the command
/// boundary. The `require` implementation decides whether the
/// failure is reported as `InvalidCredentials` or `Forbidden`;
/// the spec instructs the engine to surface every failure as
/// `Forbidden` to the user, so the adapter translates the
/// capability-check outcome into the closest
/// [`crate::errors::AuthError`] variant.
///
/// The trait is object-safe.
#[async_trait]
pub trait RbacPort: Send + Sync {
    /// Returns `Ok(true)` if the session has the requested
    /// capability (taking `session.active_school_id` into
    /// account).
    async fn has(
        &self,
        session: &Session,
        capability: Capability,
    ) -> Result<bool, crate::errors::AuthError>;

    /// Asserts the session has the requested capability.
    /// Returns `Err(AuthError::InvalidCredentials)` if not.
    /// The engine maps this to `DomainError::Forbidden` for
    /// the user.
    async fn require(
        &self,
        session: &Session,
        capability: Capability,
    ) -> Result<(), crate::errors::AuthError>;
}
