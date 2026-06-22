//! # In-memory [`AuthProvider`](educore_auth::port::AuthProvider)
//!
//! Test-only [`AuthProvider`](educore_auth::port::AuthProvider)
//! that backs every session with a `HashMap` and a fresh
//! [`SystemIdGen`](educore_core::clock::SystemIdGen).
//!
//! ## Behaviour
//!
//! - Every non-anonymous [`Credential`](educore_auth::port::Credential)
//!   mints a fresh [`Session`] with `mfa_satisfied = true`, an
//!   empty capability set, an empty role list, and an
//!   `expires_at` one hour in the future. The session is stored
//!   in an internal `HashMap` keyed by a stable token-derived
//!   string so the same session can be re-found by
//!   [`validate`](educore_auth::port::AuthProvider::validate),
//!   [`revoke`](educore_auth::port::AuthProvider::revoke), and
//!   [`refresh`](educore_auth::port::AuthProvider::refresh).
//! - [`Credential::Anonymous`](educore_auth::port::Credential::Anonymous)
//!   is rejected with
//!   [`AuthError::InvalidCredentials`](educore_auth::errors::AuthError::InvalidCredentials).
//!   Anonymous traffic is not part of the in-memory test surface;
//!   production adapters that need to gate anonymous flows wire
//!   the rejection at a higher layer.
//! - The token-lookup key for non-Bearer credentials is derived
//!   from the credential itself (e.g. `user:<username>`,
//!   `api-key:<id>`). This lets the in-memory adapter round-trip
//!   credentials that carry an opaque identifier even though the
//!   port's [`validate`](educore_auth::port::AuthProvider::validate)
//!   method only consumes [`Bearer`](educore_auth::port::AuthScheme::Bearer)
//!   tokens.
//!
//! The provider does **not** implement
//! [`RbacPort`](educore_auth::port::RbacPort); capability checks
//! live behind a separate trait and a separate test impl.

#![allow(clippy::missing_docs_in_private_items)]

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::{Mutex, PoisonError};

use async_trait::async_trait;
use chrono::{Duration, Utc};
use educore_auth::errors::AuthError;
use educore_auth::port::{AuthProvider, AuthScheme, AuthToken, Credential, Session};
use educore_core::clock::{IdGenerator, SystemIdGen};
use educore_core::ids::Identifier;
use educore_core::value_objects::Timestamp;
use educore_rbac::ids::RoleId;
use educore_rbac::value_objects::Capability;

/// The TTL of every session minted by this provider. The port
/// spec leaves the exact value up to the adapter; the test
/// surface uses a generous one-hour window so test scenarios do
/// not race the clock.
const SESSION_TTL_HOURS: i64 = 1;

/// In-memory [`AuthProvider`] backed by a process-local
/// `HashMap`. Cheap to construct (no I/O), safe to share across
/// tasks via `Arc`.
///
/// Every fresh credential produces a brand-new session; the
/// adapter does not persist or look up any user data — that is
/// the responsibility of the real auth adapters wired in
/// production builds.
#[derive(Debug, Default)]
pub struct InMemoryAuthProvider {
    sessions: Mutex<HashMap<String, Session>>,
}

impl InMemoryAuthProvider {
    /// Constructs a fresh in-memory auth provider with an empty
    /// session store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }

    /// Returns the number of sessions currently tracked by this
    /// provider. Intended for assertions in test bodies.
    #[must_use]
    pub fn session_count(&self) -> usize {
        self.sessions
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .len()
    }
}

#[async_trait]
impl AuthProvider for InMemoryAuthProvider {
    async fn authenticate(&self, credential: Credential) -> Result<Session, AuthError> {
        let id_gen = SystemIdGen;
        let user_id = id_gen.next_user_id();
        let school_id = id_gen.next_school_id();
        let session_id = id_gen.next_session_id();

        let now = Timestamp::now();
        let expires_at = Timestamp::from_datetime(
            (now.as_datetime() + Duration::hours(SESSION_TTL_HOURS)).with_timezone(&Utc),
        );

        let session = Session {
            session_id,
            user_id,
            school_ids: vec![school_id],
            active_school_id: school_id,
            roles: Vec::<RoleId>::new(),
            capabilities: BTreeSet::<Capability>::new(),
            mfa_satisfied: true,
            issued_at: now,
            expires_at,
            metadata: BTreeMap::new(),
        };

        let key = credential_key(&credential)?;

        let mut sessions = self.sessions.lock().unwrap_or_else(PoisonError::into_inner);
        sessions.insert(key, session.clone());
        Ok(session)
    }

    async fn validate(&self, token: &AuthToken) -> Result<Session, AuthError> {
        if !matches!(token.scheme, AuthScheme::Bearer) {
            return Err(AuthError::Expired);
        }
        let key = format!("{:?}", token.value);
        let sessions = self.sessions.lock().unwrap_or_else(PoisonError::into_inner);
        sessions.get(&key).cloned().ok_or(AuthError::Expired)
    }

    async fn revoke(&self, token: &AuthToken) -> Result<(), AuthError> {
        let key = format!("{:?}", token.value);
        let mut sessions = self.sessions.lock().unwrap_or_else(PoisonError::into_inner);
        sessions.remove(&key);
        Ok(())
    }

    async fn refresh(&self, token: &AuthToken) -> Result<Session, AuthError> {
        let old_session = self.validate(token).await?;

        let id_gen = SystemIdGen;
        let now = Timestamp::now();
        let new_session_id = id_gen.next_session_id();
        let expires_at = Timestamp::from_datetime(
            (now.as_datetime() + Duration::hours(SESSION_TTL_HOURS)).with_timezone(&Utc),
        );

        let new_session = Session {
            session_id: new_session_id,
            user_id: old_session.user_id,
            school_ids: old_session.school_ids.clone(),
            active_school_id: old_session.active_school_id,
            roles: old_session.roles.clone(),
            capabilities: BTreeSet::<Capability>::new(),
            mfa_satisfied: old_session.mfa_satisfied,
            issued_at: now,
            expires_at,
            metadata: BTreeMap::new(),
        };

        let key = format!("session:{}", new_session.session_id.as_uuid());
        let mut sessions = self.sessions.lock().unwrap_or_else(PoisonError::into_inner);
        sessions.insert(key, new_session.clone());
        Ok(new_session)
    }
}

/// Derives a stable session-store key from a [`Credential`].
///
/// For [`Credential::Bearer`] the key is the Debug rendering of
/// the token string (which matches the lookup performed by
/// [`validate`]), so the same bearer can be re-found across
/// `authenticate` / `validate` / `revoke` / `refresh` calls.
///
/// For the other variants we synthesise a stable key from the
/// identifier field on each variant (`username`, `code`, `id`,
/// `device_id`). For SAML we take a 32-character prefix of the
/// base64-encoded assertion, which is enough to disambiguate
/// tests without dragging megabytes of assertion through the
/// map key.
fn credential_key(credential: &Credential) -> Result<String, AuthError> {
    match credential {
        Credential::Bearer(token) => Ok(format!("{token:?}")),
        Credential::UsernamePassword { username, .. } => Ok(format!("user:{username}")),
        Credential::Oauth2 { code, .. } => Ok(format!("oauth:{code}")),
        Credential::Saml { assertion, .. } => {
            let head: String = assertion.chars().take(32).collect();
            Ok(format!("saml:{head}"))
        }
        Credential::ApiKey { id, .. } => Ok(format!("api-key:{id}")),
        Credential::Biometric { device_id, .. } => Ok(format!("device:{device_id}")),
        Credential::Anonymous => Err(AuthError::InvalidCredentials),
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
    clippy::dbg_macro
)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    use educore_auth::port::{AuthScheme, AuthToken};

    fn bearer_token(value: &str) -> AuthToken {
        AuthToken {
            scheme: AuthScheme::Bearer,
            value: value.to_owned(),
            metadata: BTreeMap::new(),
        }
    }

    fn block_on<F: std::future::Future>(future: F) -> F::Output {
        futures::executor::block_on(future)
    }

    #[test]
    fn authenticate_bearer_mints_session() {
        let provider = InMemoryAuthProvider::new();
        let session =
            block_on(provider.authenticate(Credential::Bearer("bearer-token-1".to_owned())))
                .unwrap();

        assert!(session.mfa_satisfied, "test sessions must skip MFA");
        assert!(session.capabilities.is_empty());
        assert_eq!(provider.session_count(), 1);
    }

    #[test]
    fn authenticate_anonymous_returns_invalid_credentials() {
        let provider = InMemoryAuthProvider::new();
        let result = block_on(provider.authenticate(Credential::Anonymous));
        assert_eq!(result, Err(AuthError::InvalidCredentials));
        assert_eq!(provider.session_count(), 0);
    }

    #[test]
    fn validate_unknown_token_returns_expired() {
        let provider = InMemoryAuthProvider::new();
        let result = block_on(provider.validate(&bearer_token("never-issued")));
        assert_eq!(result, Err(AuthError::Expired));
    }

    #[test]
    fn revoke_removes_session() {
        let provider = InMemoryAuthProvider::new();
        let _ =
            block_on(provider.authenticate(Credential::Bearer("revoke-me".to_owned()))).unwrap();
        assert_eq!(provider.session_count(), 1);

        block_on(provider.revoke(&bearer_token("revoke-me"))).unwrap();
        assert_eq!(provider.session_count(), 0);

        let after_revoke = block_on(provider.validate(&bearer_token("revoke-me")));
        assert_eq!(after_revoke, Err(AuthError::Expired));
    }

    #[test]
    fn refresh_mints_new_session_with_same_school() {
        let provider = InMemoryAuthProvider::new();
        let first =
            block_on(provider.authenticate(Credential::Bearer("refresh-me".to_owned()))).unwrap();
        let second = block_on(provider.refresh(&bearer_token("refresh-me"))).unwrap();

        assert_ne!(first.session_id, second.session_id);
        assert_eq!(first.active_school_id, second.active_school_id);
        assert_eq!(first.user_id, second.user_id);
        assert!(second.mfa_satisfied);
    }

    #[test]
    fn authenticate_username_password_mints_session() {
        let provider = InMemoryAuthProvider::new();
        let session = block_on(provider.authenticate(Credential::UsernamePassword {
            username: "alice".to_owned(),
            password: "hunter2".to_owned(),
        }))
        .unwrap();

        assert!(session.mfa_satisfied);
        assert_eq!(provider.session_count(), 1);
    }
}
