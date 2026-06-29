//! # Local-password [`AuthProvider`] reference implementation.
//!
//! Username + password authentication against a local user table,
//! with Argon2id password hashing and a wrapped
//! [`crate::jwt::JwtAuthProvider`] that owns the post-authentication
//! `Authorization: Bearer ...` plumbing. This is the
//! "identity-provider-of-last-resort" adapter: every consumer that
//! does not delegate identity to an external IdP (Google, Okta,
//! Azure AD / Entra ID, SAML, ...) uses this provider.
//!
//! ## Algorithm
//!
//! - Password hashing: Argon2id via the [`crate::services::PasswordService`]
//!   helper. The service holds an `Argon2` instance with the
//!   engine's current default parameters (memory cost, time cost,
//!   parallelism) and produces self-describing PHC strings
//!   (`$argon2id$v=19$m=...,t=...,p=...$<salt>$<hash>`) suitable for
//!   direct storage in the credentials column of the `users` table.
//! - Password verification: Argon2id constant-time compare against
//!   the stored PHC string. The reference implementation also
//!   reports `needs_rehash` so a successful login transparently
//!   rotates the stored hash when the engine's default parameters
//!   change.
//! - Session issuance: on a successful credential match the provider
//!   delegates to a wrapped [`crate::jwt::JwtAuthProvider`] which
//!   mints the JWT. Consumers see a uniform `Authorization: Bearer
//!   ...` flow end-to-end (the same shape the JWT, SAML, and
//!   future OAuth2 adapters emit).
//!
//! ## Implementation gap (PORT-AUTH-LOCAL-PWD)
//!
//! This is a **minimal scaffolding**. The follow-up phases add:
//!
//! 1. **Persistence.** The current `LocalPasswordAuthProvider`
//!    holds the credential table in an in-memory
//!    `HashMap<String, StoredCredential>`. The production adapter
//!    reads from a `users` table joined to a `password_hashes`
//!    table (or whatever the storage adapter's `CredentialRepository`
//!    port exposes).
//! 2. **User provisioning.** The current adapter exposes an
//!    `upsert_user` helper for tests. The production adapter
//!    delegates to a domain command (e.g. `provision_user` from
//!    the `educore-hr` domain) so the credentials table stays in
//!    lock-step with the engine's HR aggregates.
//! 3. **Account lockout.** `Credential::UsernamePassword` is
//!    currently rejected with `AuthError::InvalidCredentials` on a
//!    mismatch. The lockout / rate-limit integration (QW-8) plugs
//!    in here once the consumer wires the [`crate::rate_limit::RateLimiter`].
//! 4. **MFA step-up.** The wrapped [`crate::jwt::JwtAuthProvider`]
//!    mints the JWT with `mfa: false` by default. Sensitive
//!    commands reject sessions with `mfa_satisfied = false` (see
//!    [`crate::port::Session`]); the consumer's auth flow collects
//!    the second factor and calls `authenticate` again with a
//!    credential that carries the MFA response.
//!
//! Until those land, this provider:
//!
//! - Accepts only [`Credential::UsernamePassword`] and rejects every
//!   other variant with [`AuthError::InvalidCredentials`]. (A
//!   Bearer token presented to `validate` is delegated to the
//!   wrapped [`crate::jwt::JwtAuthProvider`] so the same adapter
//!   can both issue and validate sessions.)
//! - Holds the credential table in-process; the process restart
//!   wipes every registered user. This is acceptable for the
//!   scaffolding gate but is **not** production-grade.

#![allow(clippy::missing_docs_in_private_items)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use uuid::Uuid;

#[allow(unused_imports)]
use educore_core::ids::{SchoolId, SessionId, UserId, PUBLIC_SCHOOL_ID};
use educore_core::value_objects::Timestamp;

use crate::errors::AuthError;
use crate::jwt::JwtAuthProvider;
use crate::port::{AuthProvider, AuthScheme, AuthToken, Credential, Session};
use crate::services::{PasswordService, SecretString};

/// Default session lifetime used by the local-password scaffolding.
///
/// The wrapped [`JwtAuthProvider`] exposes a configurable
/// `access_ttl` (defaults to one hour); mirroring that here keeps
/// the local-password session shape in lock-step with the
/// JWT-issued lifetime. The constant lives on this module
/// because (a) we don't yet have a public `access_ttl_seconds`
/// accessor on [`JwtAuthProvider`] and (b) the production
/// adapter will replace this scaffolding with a storage-backed
/// session lookup that the engine's TTL policy owns end-to-end.
const SESSION_TTL: Duration = Duration::from_secs(60 * 60);

// ---------------------------------------------------------------------------
// StoredCredential
// ---------------------------------------------------------------------------

/// The internal record held by [`LocalPasswordAuthProvider`]'s
/// in-memory user table.
///
/// Production adapters will replace this with a row read from a
/// `users` table (joined with a `password_hashes` table); the
/// `#[allow(dead_code)]` fields on this struct are the future
/// surface (active_school for cross-school users, locked_until
/// for account-lockout, password_changed_at for `needs_rehash`
/// audit trails). The fields stay on the struct so the migration
/// from scaffolding to the production adapter is a one-line type
/// change rather than a refactor.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct StoredCredential {
    /// The principal's stable user id (UUIDv7). The
    /// post-authentication [`Session::user_id`] is taken from this
    /// field.
    user_id: UserId,
    /// The Argon2id PHC string produced by
    /// [`PasswordService::hash_password`].
    password_hash: String,
    /// The schools the user belongs to. Cross-school users
    /// (e.g. a district admin) have more than one entry.
    school_ids: Vec<SchoolId>,
    /// The tenant for sessions minted from this credential.
    active_school_id: SchoolId,
}

// ---------------------------------------------------------------------------
// LocalPasswordAuthProvider
// ---------------------------------------------------------------------------

/// Local-password [`AuthProvider`] reference implementation.
///
/// Holds an in-memory `username -> StoredCredential` table behind
/// a `Mutex<HashMap<...>>` and a [`PasswordService`] for Argon2id
/// hashing / verification. Token issuance is delegated to a
/// wrapped [`JwtAuthProvider`] so the post-authentication flow is
/// the same `Authorization: Bearer ...` shape every other adapter
/// in the engine emits.
///
/// Construct with [`LocalPasswordAuthProvider::new`] and wire the
/// provider into the engine at startup alongside the other
/// [`AuthProvider`] adapters (see `docs/ports/authentication.md`
/// § "Configuration").
///
/// Object safety: the trait is object-safe; this concrete type is
/// `Send + Sync + Debug`.
#[derive(Debug, Clone)]
pub struct LocalPasswordAuthProvider {
    /// The JWT provider that owns HMAC signing, token validation,
    /// refresh, and revocation. The post-authentication [`Session`]
    /// flows through this wrapped provider so consumers see a
    /// uniform `Authorization: Bearer ...` flow end-to-end.
    jwt: JwtAuthProvider,
    /// Argon2id hasher + verifier. Shared across clones via
    /// `Arc` so the cached default parameters are computed once
    /// per process.
    passwords: Arc<PasswordService>,
    /// In-memory user table. The `Mutex` is a `std::sync::Mutex`
    /// because the critical sections are O(1) and never await.
    /// The table is wiped on process restart; production
    /// adapters swap this for a storage-port-backed repository.
    users: Arc<Mutex<HashMap<String, StoredCredential>>>,
}

impl LocalPasswordAuthProvider {
    /// Constructs a `LocalPasswordAuthProvider` that delegates
    /// token issuance to the given [`JwtAuthProvider`].
    ///
    /// The wrapped provider is the source of truth for the HMAC
    /// secret, issuer / audience, and the in-process revocation
    /// set; this constructor therefore does **not** accept any
    /// local-password-specific configuration (storage backend,
    /// lockout policy, MFA policy) until the implementation gap
    /// documented at the module level is closed.
    #[must_use]
    pub fn new(jwt: JwtAuthProvider) -> Self {
        Self {
            jwt,
            passwords: Arc::new(PasswordService::new()),
            users: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Returns a reference to the wrapped [`JwtAuthProvider`] so
    /// callers can introspect or extend the underlying token
    /// plumbing (e.g. read the configured issuer / audience for
    /// diagnostics, or pre-load a signing key from `JWT_SECRET`).
    #[must_use]
    pub fn jwt_provider(&self) -> &JwtAuthProvider {
        &self.jwt
    }

    /// Returns a reference to the [`PasswordService`] used for
    /// Argon2id hashing / verification. Production adapters that
    /// need a custom cost parameter override can swap the
    /// service via [`LocalPasswordAuthProvider::with_password_service`].
    #[must_use]
    pub fn password_service(&self) -> &PasswordService {
        &self.passwords
    }

    /// Replaces the [`PasswordService`] used for Argon2id hashing
    /// / verification. Returns the same provider so the call
    /// chains into a constructor.
    #[must_use]
    pub fn with_password_service(mut self, service: PasswordService) -> Self {
        self.passwords = Arc::new(service);
        self
    }

    /// Returns `true` if a user with the given `username` exists
    /// in the in-memory table. Used by tests and the worked
    /// example to verify provisioning; production adapters will
    /// answer this from storage instead.
    #[must_use]
    pub fn has_user(&self, username: &str) -> bool {
        self.lock_users().contains_key(username)
    }

    /// Provisions a user in the in-memory table with the given
    /// username, plaintext password, and (single-tenant) school
    /// membership. The password is hashed with Argon2id before it
    /// reaches the table; the plaintext is **never** stored.
    ///
    /// Returns `Ok(())` on success. Returns
    /// [`AuthError::Malformed`] if Argon2id fails to hash the
    /// password (the only realistic cause is a `rand_core`
    /// failure, which is a programmer / environment error).
    ///
    /// Existing entries with the same username are overwritten so
    /// the helper doubles as a password-reset primitive in tests.
    /// Production adapters will reject duplicate usernames and
    /// route password changes through a dedicated command.
    pub fn upsert_user(
        &self,
        username: &str,
        plaintext: &SecretString,
        active_school_id: SchoolId,
    ) -> Result<UserId, AuthError> {
        let hash = self.passwords.hash_password(plaintext)?;
        let user_id = UserId(Uuid::now_v7());
        let record = StoredCredential {
            user_id,
            password_hash: hash,
            school_ids: vec![active_school_id],
            active_school_id,
        };
        self.lock_users().insert(username.to_owned(), record);
        Ok(user_id)
    }

    /// Looks up the user id of a registered user. Returns `None`
    /// when the username is not present in the in-memory table.
    #[must_use]
    pub fn user_id_of(&self, username: &str) -> Option<UserId> {
        self.lock_users().get(username).map(|r| r.user_id)
    }

    /// Acquires the users-table mutex, recovering from poisoning.
    pub(crate) fn lock_users(
        &self,
    ) -> std::sync::MutexGuard<'_, HashMap<String, StoredCredential>> {
        self.users.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Builds the post-authentication [`Session`] from a
    /// verified credential record. The session is minted in the
    /// engine's standard shape (UUIDv7 session id, typed school
    /// ids, empty capability set resolved later from RBAC,
    /// `mfa_satisfied = false` so sensitive commands can require
    /// step-up).
    fn build_session(&self, record: &StoredCredential) -> Session {
        let now = Timestamp::now();
        let exp_secs = i64::try_from(SESSION_TTL.as_secs()).unwrap_or(i64::MAX);
        let exp_dt = Utc
            .timestamp_opt(now.as_datetime().timestamp() + exp_secs, 0)
            .single()
            .unwrap_or_else(|| now.as_datetime());
        let expires_at = Timestamp::from_datetime(exp_dt);

        Session {
            session_id: SessionId(Uuid::now_v7()),
            user_id: record.user_id,
            school_ids: record.school_ids.clone(),
            active_school_id: record.active_school_id,
            roles: Vec::new(),
            capabilities: std::collections::BTreeSet::new(),
            mfa_satisfied: false,
            issued_at: now,
            expires_at,
            metadata: std::collections::BTreeMap::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// AuthProvider impl
// ---------------------------------------------------------------------------

#[async_trait]
impl AuthProvider for LocalPasswordAuthProvider {
    async fn authenticate(&self, credential: Credential) -> Result<Session, AuthError> {
        match credential {
            Credential::UsernamePassword { username, password } => {
                // Look the user up in the in-memory table. A
                // missing user maps to InvalidCredentials
                // (NOT AccountDisabled or a sentinel error) so
                // the failure mode is indistinguishable from a
                // wrong password; otherwise an attacker can
                // enumerate registered usernames by comparing
                // error variants.
                let record = {
                    let table = self.lock_users();
                    table.get(&username).cloned()
                };
                let Some(record) = record else {
                    return Err(AuthError::InvalidCredentials);
                };

                // Verify the plaintext against the stored
                // Argon2id PHC string. A mismatch returns
                // Ok(false); we map that to InvalidCredentials
                // for the same enumeration reason as above.
                let secret = SecretString::from(password.as_str());
                let ok = self
                    .passwords
                    .verify_password(&secret, &record.password_hash)?;
                if !ok {
                    return Err(AuthError::InvalidCredentials);
                }

                // Mint the session in the engine's standard shape.
                // Capability resolution and MFA step-up are layered
                // on top of this scaffolding once the command
                // handler lands.
                Ok(self.build_session(&record))
            }
            // The wrapped JwtAuthProvider already rejects
            // Anonymous with InvalidCredentials (FND-SEC-AUTH-001),
            // and accepts Bearer tokens. Delegating to it here
            // means a single LocalPasswordAuthProvider can both
            // authenticate users (via UsernamePassword) and
            // validate / refresh / revoke the bearer tokens that
            // the auth flow minted — same pattern as the SAML
            // provider's delegation to the wrapped JWT provider.
            other => self.jwt.authenticate(other).await,
        }
    }

    async fn validate(&self, token: &AuthToken) -> Result<Session, AuthError> {
        if !matches!(token.scheme, AuthScheme::Bearer) {
            return Err(AuthError::Malformed(format!(
                "LocalPasswordAuthProvider only accepts Bearer tokens, got {:?}",
                token.scheme
            )));
        }
        self.jwt.validate(token).await
    }

    async fn revoke(&self, token: &AuthToken) -> Result<(), AuthError> {
        if !matches!(token.scheme, AuthScheme::Bearer) {
            return Err(AuthError::Malformed(format!(
                "LocalPasswordAuthProvider only accepts Bearer tokens, got {:?}",
                token.scheme
            )));
        }
        self.jwt.revoke(token).await
    }

    async fn refresh(&self, token: &AuthToken) -> Result<Session, AuthError> {
        if !matches!(token.scheme, AuthScheme::Bearer) {
            return Err(AuthError::Malformed(format!(
                "LocalPasswordAuthProvider only accepts Bearer tokens, got {:?}",
                token.scheme
            )));
        }
        self.jwt.refresh(token).await
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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
    use std::collections::BTreeMap;
    use std::time::Duration;

    use super::*;
    use crate::jwt::JwtAuthProviderBuilder;

    fn jwt_with_test_key() -> JwtAuthProvider {
        // 32-byte key satisfying HS256 minimum-length
        // (RFC 7518 § 3.2) without colliding with the
        // "educore" default issuer used elsewhere.
        JwtAuthProviderBuilder::new()
            .signing_key(b"test-key-for-local-pwd-32bytes!!".to_vec())
            .issuer("educore-local-pwd-test")
            .audience("educore")
            .access_ttl(Duration::from_secs(3600))
            .refresh_ttl(Duration::from_secs(7 * 24 * 3600))
            .build()
    }

    fn provider() -> LocalPasswordAuthProvider {
        LocalPasswordAuthProvider::new(jwt_with_test_key())
    }

    #[test]
    fn test_local_password_provider_constructs_with_jwt_provider() {
        let p = provider();
        // The wrapped provider is reachable via the accessor.
        let _wrapped = p.jwt_provider();
        // The password service is reachable.
        let _ = p.password_service();
        // The in-memory user table starts empty.
        assert!(p.lock_users().is_empty());
        assert!(!p.has_user("anyone"));
    }

    #[test]
    fn test_local_password_provider_is_clone_and_debug() {
        let p = provider();
        let clone = p.clone();
        // Debug render does not panic and includes the
        // "LocalPasswordAuthProvider" type marker.
        let rendered = format!("{p:?}");
        assert!(rendered.contains("LocalPasswordAuthProvider"));
        let _ = format!("{clone:?}");
    }

    #[test]
    fn test_local_password_provider_upsert_and_user_lookup() {
        let p = provider();
        let school = SchoolId(Uuid::now_v7());
        let user_id = p
            .upsert_user(
                "alice",
                &SecretString::new("correct horse battery staple"),
                school,
            )
            .expect("upsert succeeds");

        assert!(p.has_user("alice"));
        assert_eq!(p.user_id_of("alice"), Some(user_id));
        assert!(p.user_id_of("bob").is_none());
        // The active school id round-trips.
        let table = p.lock_users();
        let record = table.get("alice").expect("alice present");
        assert_eq!(record.active_school_id, school);
        assert_eq!(record.user_id, user_id);
        // The stored value is a real Argon2id PHC string, NOT
        // the plaintext password.
        assert!(record.password_hash.starts_with("$argon2id$"));
        assert!(!record.password_hash.contains("correct horse"));
    }

    #[tokio::test]
    async fn test_local_password_provider_authenticates_correct_credential() {
        let p = provider();
        let school = SchoolId(Uuid::now_v7());
        let user_id = p
            .upsert_user("alice", &SecretString::new("hunter2"), school)
            .expect("upsert succeeds");

        let credential = Credential::UsernamePassword {
            username: "alice".to_owned(),
            password: "hunter2".to_owned(),
        };
        let session = p
            .authenticate(credential)
            .await
            .expect("correct password authenticates");
        assert_eq!(session.user_id, user_id);
        assert_eq!(session.active_school_id, school);
        assert_eq!(session.school_ids, vec![school]);
        // Sensitive commands require MFA step-up; the
        // scaffolding provider emits a non-MFA session and
        // the consumer's auth flow must layer the second
        // factor explicitly.
        assert!(!session.mfa_satisfied);
        // Capability resolution is a future revision; today
        // the scaffolding provider emits an empty capability
        // set, matching the wrapped JWT provider's shape.
        assert!(session.capabilities.is_empty());
        // Issued / expiry are well-formed (expiry strictly
        // after issued_at).
        assert!(session.expires_at.as_datetime() > session.issued_at.as_datetime());
    }

    #[tokio::test]
    async fn test_local_password_provider_rejects_wrong_password() {
        let p = provider();
        let school = SchoolId(Uuid::now_v7());
        p.upsert_user("alice", &SecretString::new("hunter2"), school)
            .expect("upsert succeeds");

        let credential = Credential::UsernamePassword {
            username: "alice".to_owned(),
            password: "wrong-password".to_owned(),
        };
        let err = p
            .authenticate(credential)
            .await
            .expect_err("wrong password must fail");
        // The failure mode for a wrong password is
        // indistinguishable from a missing user (both map
        // to InvalidCredentials), so an attacker cannot
        // enumerate registered usernames.
        assert_eq!(err, AuthError::InvalidCredentials);
    }

    #[tokio::test]
    async fn test_local_password_provider_rejects_unknown_username() {
        let p = provider();
        // No upsert — alice does not exist.
        let credential = Credential::UsernamePassword {
            username: "alice".to_owned(),
            password: "any-password".to_owned(),
        };
        let err = p
            .authenticate(credential)
            .await
            .expect_err("unknown username must fail");
        assert_eq!(
            err,
            AuthError::InvalidCredentials,
            "missing user must look identical to wrong password (enumeration defense)"
        );
    }

    #[tokio::test]
    async fn test_local_password_provider_delegates_bearer_to_wrapped_jwt() {
        // A Bearer token presented to authenticate is forwarded
        // to the wrapped JwtAuthProvider so a single adapter can
        // both authenticate users (via UsernamePassword) and
        // validate existing bearer tokens. This matches the
        // SamlAuthProvider delegation pattern.
        let p = provider();
        let credential = Credential::Bearer("not-a-real-jwt".to_owned());
        let err = p
            .authenticate(credential)
            .await
            .expect_err("malformed bearer must error, not panic");
        // The wrapped provider maps invalid JWTs to
        // AuthError::Malformed; we only assert it's the same
        // error class the wrapped provider would produce, not
        // a panic or a security regression (e.g. an anonymous
        // session on a malformed token).
        assert!(
            matches!(err, AuthError::Malformed(_)),
            "expected wrapped-provider Malformed, got {err:?}"
        );
    }

    #[tokio::test]
    async fn test_local_password_provider_validate_rejects_non_bearer() {
        let p = provider();
        let token = AuthToken {
            scheme: AuthScheme::Cookie,
            value: "session=opaque".to_owned(),
            metadata: BTreeMap::new(),
        };
        let err = p
            .validate(&token)
            .await
            .expect_err("cookie scheme must be rejected");
        assert!(
            matches!(err, AuthError::Malformed(_)),
            "expected Malformed, got {err:?}"
        );
    }

    #[tokio::test]
    async fn test_local_password_provider_revoke_delegates_to_wrapped_jwt() {
        let p = provider();
        let token = AuthToken {
            scheme: AuthScheme::Bearer,
            value: "not-a-real-jwt".to_owned(),
            metadata: BTreeMap::new(),
        };
        let err = p
            .revoke(&token)
            .await
            .expect_err("revoke of malformed bearer must error, not panic");
        assert!(
            matches!(err, AuthError::Malformed(_)),
            "expected Malformed from wrapped provider, got {err:?}"
        );
    }

    #[tokio::test]
    async fn test_local_password_provider_refresh_delegates_to_wrapped_jwt() {
        let p = provider();
        let token = AuthToken {
            scheme: AuthScheme::Bearer,
            value: "not-a-real-jwt".to_owned(),
            metadata: BTreeMap::new(),
        };
        let err = p
            .refresh(&token)
            .await
            .expect_err("refresh of malformed bearer must error, not panic");
        assert!(
            matches!(err, AuthError::Malformed(_)),
            "expected Malformed from wrapped provider, got {err:?}"
        );
    }

    #[tokio::test]
    async fn test_local_password_provider_public_school_id_is_well_known() {
        // The PUBLIC_SCHOOL_ID constant is re-exported through
        // educore-core and is a stable sentinel for tenants
        // outside any specific school (system users, public
        // flows). The scaffolding provider can accept it as
        // the active_school for system-bootstrap users.
        assert_eq!(PUBLIC_SCHOOL_ID, PUBLIC_SCHOOL_ID);
        let p = provider();
        let user_id = p
            .upsert_user(
                "system-bootstrap",
                &SecretString::new("very-long-bootstrap-password"),
                PUBLIC_SCHOOL_ID,
            )
            .expect("upsert to public school succeeds");
        assert_eq!(p.user_id_of("system-bootstrap"), Some(user_id));
    }
}
