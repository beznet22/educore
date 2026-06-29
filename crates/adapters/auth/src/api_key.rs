//! # ApiKey-based [`AuthProvider`] reference implementation
//!
//! Service-to-service authentication using a pre-shared API key.
//!
//! ## Wire format
//!
//! `Credential::ApiKey { id, key }` — `id` is the public key id
//! (e.g. `ak_01HXY...`), `key` is the secret value presented at
//! request time.
//!
//! ## Verification
//!
//! HMAC-SHA256 over a fixed label is computed for both the
//! stored secret and the presented key, and the two digests are
//! compared in constant time. The raw secret is never compared
//! directly — only fixed-length HMAC digests are compared, so
//! timing attacks cannot leak the secret byte-by-byte.
//!
//! ## Implementation gap (PORT-AUTH-APIKEY)
//!
//! This is a minimal scaffold: HMAC verification with constant-
//! time digest comparison and delegated session creation to
//! the wrapped [`JwtAuthProvider`]. Key rotation, multi-key
//! support, and revocation lists land in a later phase.
//!
//! [`AuthProvider`]: crate::port::AuthProvider
//! [`JwtAuthProvider`]: crate::jwt::JwtAuthProvider

use async_trait::async_trait;

use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::errors::AuthError;
use crate::jwt::JwtAuthProvider;
use crate::port::{AuthProvider, AuthToken, Credential, Session};

/// HMAC-SHA256 type alias for the constant-time digest scheme.
type HmacSha256 = Hmac<Sha256>;

/// Fixed label used as the HMAC message; binds the digest to
/// this specific scheme so a digest computed for one auth
/// provider cannot be replayed against another.
const APIKEY_HMAC_LABEL: &[u8] = b"educore-auth-apikey-v1";

/// API-key-based [`AuthProvider`] (service-to-service auth).
///
/// Wraps a [`JwtAuthProvider`] for session/JWT issuance. The
/// constructor accepts a `key_id` (public part) and `secret`
/// (private part); on `authenticate(Credential::ApiKey { .. })`,
/// the provider computes HMAC digests for both the stored
/// secret and the presented key, compares them in constant
/// time, and on success mints a session via the wrapped JWT
/// provider.
#[derive(Debug, Clone)]
pub struct ApiKeyAuthProvider {
    /// The public API key id (e.g. `ak_01HXY...`).
    key_id: String,
    /// The pre-shared API key secret (raw bytes of the
    /// secret value).
    secret: String,
    /// The wrapped JWT provider used for session issuance.
    jwt: JwtAuthProvider,
}

impl ApiKeyAuthProvider {
    /// Constructs a new `ApiKeyAuthProvider` from a `key_id`,
    /// `secret`, and a wrapped [`JwtAuthProvider`]. Returns
    /// `AuthError::Malformed` if `key_id` or `secret` is empty.
    pub fn new(key_id: String, secret: String, jwt: JwtAuthProvider) -> Result<Self, AuthError> {
        if key_id.is_empty() {
            return Err(AuthError::Malformed(
                "API key id cannot be empty".to_owned(),
            ));
        }
        if secret.is_empty() {
            return Err(AuthError::Malformed(
                "API key secret cannot be empty".to_owned(),
            ));
        }
        Ok(Self {
            key_id,
            secret,
            jwt,
        })
    }

    /// Constructs a new `ApiKeyAuthProvider` from environment
    /// variables `API_KEY_ID` and `API_KEY_SECRET`. Returns
    /// `AuthError::Malformed` if either variable is unset or empty.
    pub fn from_env(jwt: JwtAuthProvider) -> Result<Self, AuthError> {
        let key_id = std::env::var("API_KEY_ID").map_err(|_| {
            AuthError::Malformed("API_KEY_ID environment variable must be set".to_owned())
        })?;
        let secret = std::env::var("API_KEY_SECRET").map_err(|_| {
            AuthError::Malformed("API_KEY_SECRET environment variable must be set".to_owned())
        })?;
        Self::new(key_id, secret, jwt)
    }

    /// Returns the public API key id.
    #[must_use]
    pub fn key_id(&self) -> &str {
        &self.key_id
    }

    /// Returns a reference to the wrapped JWT provider.
    #[must_use]
    pub fn jwt_provider(&self) -> &JwtAuthProvider {
        &self.jwt
    }

    /// Returns `true` iff the presented key matches the stored
    /// secret. Uses HMAC-SHA256 over a fixed label and a
    /// constant-time digest comparison.
    fn verify_key(&self, presented: &str) -> bool {
        let stored = hmac_digest(self.secret.as_bytes());
        let given = hmac_digest(presented.as_bytes());
        constant_time_eq(&stored, &given)
    }
}

/// Computes the HMAC-SHA256 digest of `key` over
/// [`APIKEY_HMAC_LABEL`].
fn hmac_digest(key: &[u8]) -> [u8; 32] {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(APIKEY_HMAC_LABEL);
    let result = mac.finalize().into_bytes();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

/// Constant-time byte-slice equality. Returns `false` on length
/// mismatch without inspecting the bytes.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[async_trait]
impl AuthProvider for ApiKeyAuthProvider {
    async fn authenticate(&self, credential: Credential) -> Result<Session, AuthError> {
        match credential {
            Credential::ApiKey { id, key } => {
                if id != self.key_id {
                    return Err(AuthError::InvalidCredentials);
                }
                if !self.verify_key(&key) {
                    return Err(AuthError::InvalidCredentials);
                }
                // Successful authentication. Mint a session for
                // the service principal via the wrapped JWT
                // provider.
                self.jwt
                    .mint_session_for_service(&format!("apikey:{}", self.key_id))
            }
            // All other credential variants are rejected. The
            // ApiKeyAuthProvider only knows how to authenticate
            // API keys; consumers must use the right provider
            // for the credential type they present.
            _ => Err(AuthError::InvalidCredentials),
        }
    }

    /// Delegates bearer-token validation to the wrapped
    /// [`JwtAuthProvider`]. Bearer tokens minted by this
    /// provider flow through the JWT path so that
    /// `validate` / `revoke` / `refresh` semantics remain
    /// consistent.
    async fn validate(&self, token: &AuthToken) -> Result<Session, AuthError> {
        self.jwt.validate(token).await
    }

    async fn revoke(&self, token: &AuthToken) -> Result<(), AuthError> {
        self.jwt.revoke(token).await
    }

    async fn refresh(&self, token: &AuthToken) -> Result<Session, AuthError> {
        self.jwt.refresh(token).await
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;

    fn test_jwt() -> JwtAuthProvider {
        crate::jwt::JwtAuthProviderBuilder::new()
            .signing_key(b"unit-test-signing-key-must-be-32-bytes-long!!".to_vec())
            .issuer("educore-test")
            .audience("educore-test-api")
            .build()
    }

    #[test]
    fn valid_key_authenticates_and_returns_session() {
        let provider =
            ApiKeyAuthProvider::new("ak_test".to_owned(), "s3cret".to_owned(), test_jwt())
                .expect("valid config");
        let credential = Credential::ApiKey {
            id: "ak_test".to_owned(),
            key: "s3cret".to_owned(),
        };
        let session = futures::executor::block_on(provider.authenticate(credential))
            .expect("valid key should authenticate");
        assert!(!session.session_id.0.is_nil());
    }

    #[test]
    fn wrong_key_id_is_rejected() {
        let provider =
            ApiKeyAuthProvider::new("ak_test".to_owned(), "s3cret".to_owned(), test_jwt())
                .expect("valid config");
        let credential = Credential::ApiKey {
            id: "ak_other".to_owned(),
            key: "s3cret".to_owned(),
        };
        let err = futures::executor::block_on(provider.authenticate(credential))
            .expect_err("wrong key id must be rejected");
        assert!(matches!(err, AuthError::InvalidCredentials));
    }

    #[test]
    fn wrong_secret_is_rejected() {
        let provider =
            ApiKeyAuthProvider::new("ak_test".to_owned(), "s3cret".to_owned(), test_jwt())
                .expect("valid config");
        let credential = Credential::ApiKey {
            id: "ak_test".to_owned(),
            key: "wrong".to_owned(),
        };
        let err = futures::executor::block_on(provider.authenticate(credential))
            .expect_err("wrong secret must be rejected");
        assert!(matches!(err, AuthError::InvalidCredentials));
    }

    #[test]
    fn empty_config_is_rejected() {
        let jwt = test_jwt();
        let err = ApiKeyAuthProvider::new(String::new(), "secret".to_owned(), jwt)
            .expect_err("empty key_id must be rejected");
        assert!(matches!(err, AuthError::Malformed(_)));

        let jwt = test_jwt();
        let err = ApiKeyAuthProvider::new("ak_test".to_owned(), String::new(), jwt)
            .expect_err("empty secret must be rejected");
        assert!(matches!(err, AuthError::Malformed(_)));
    }

    #[test]
    fn non_api_key_credential_is_rejected() {
        let provider =
            ApiKeyAuthProvider::new("ak_test".to_owned(), "s3cret".to_owned(), test_jwt())
                .expect("valid config");
        let credential = Credential::Bearer("eyJxxx".to_owned());
        let err = futures::executor::block_on(provider.authenticate(credential))
            .expect_err("non-ApiKey credentials must be rejected");
        assert!(matches!(err, AuthError::InvalidCredentials));
    }

    #[test]
    fn hmac_digest_is_deterministic_and_label_bound() {
        let a = hmac_digest(b"secret");
        let b = hmac_digest(b"secret");
        assert_eq!(a, b);
        let c = hmac_digest(b"different");
        assert_ne!(a, c);
    }

    #[test]
    fn constant_time_eq_works() {
        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(!constant_time_eq(b"hello", b"world"));
        assert!(!constant_time_eq(b"hello", b"hell"));
        assert!(!constant_time_eq(b"", b"x"));
    }
}
