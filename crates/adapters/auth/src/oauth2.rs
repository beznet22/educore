//! # OAuth 2.0 / OpenID Connect [`AuthProvider`] reference implementation.
//!
//! Standards-compliant authorization-code (with optional PKCE,
//! per RFC 7636) integration for third-party identity providers:
//! Google, Microsoft Entra ID, Auth0, Okta CIC, GitHub, Keycloak,
//! etc. The post-callback session is delivered to consumers as a
//! JWT (signed and validated by the wrapped [`JwtAuthProvider`])
//! so the rest of the request pipeline sees a uniform
//! `Authorization: Bearer ...` flow.
//!
//! ## Implementation gap (PORT-AUTH-OAUTH2)
//!
//! This is a **minimal scaffolding**. Full OAuth 2.0 / OIDC
//! support lands in a later phase and adds:
//!
//! 1. **Authorization-code + PKCE exchange** — turn the
//!    `Credential::Oauth2 { code, redirect_uri, code_verifier }`
//!    into an IdP token set (access token, refresh token, ID
//!    token) by POSTing to `token_endpoint`. Verify the ID
//!    token signature against the IdP JWKS, enforce `iss` /
//!    `aud` / `nonce` / `exp`, and mint a session via the
//!    wrapped [`JwtAuthProvider`].
//! 2. **Discovery (OIDC)** — fetch the `.well-known/openid-configuration`
//!    document and resolve `authorization_endpoint` /
//!    `token_endpoint` / `jwks_uri` from the IdP. Refresh on a
//!    timer so endpoint rotations propagate without a process
//!    restart.
//! 3. **PKCE generation helpers** — `code_verifier` +
//!    `code_challenge` (S256) generators for the consumer's
//!    authorization-code redirect handler.
//! 4. **Refresh-token rotation** — POST to `token_endpoint`
//!    with `grant_type=refresh_token`, validate the new ID
//!    token, and rotate the in-process revocation entry via
//!    the wrapped provider.
//!
//! Until those land, this provider rejects every
//! [`Credential::Oauth2`] with [`AuthError::InvalidCredentials`]
//! and delegates token validation, refresh, and revocation to
//! the wrapped [`JwtAuthProvider`]. Consumers that only need
//! JWT-based session validation can wire an
//! [`OAuth2AuthProvider`] (backed by a [`JwtAuthProvider`])
//! into their app and rely on the delegation; consumers that
//! need live OAuth callbacks must layer the callback handler
//! above this provider until the gap is closed.

#![allow(clippy::missing_docs_in_private_items)]

use async_trait::async_trait;

use crate::errors::AuthError;
use crate::jwt::JwtAuthProvider;
use crate::port::{AuthProvider, AuthToken, Credential, Session};

// ---------------------------------------------------------------------------
// OAuth2Config
// ---------------------------------------------------------------------------

/// Configuration for an OAuth 2.0 / OpenID Connect identity
/// provider client.
///
/// Holds the OAuth client registration data plus the IdP
/// endpoint URLs. All fields are required: the consumer wires
/// them from environment variables (e.g. `OAUTH_CLIENT_ID`,
/// `OAUTH_CLIENT_SECRET`, `OAUTH_AUTHORIZATION_ENDPOINT`,
/// `OAUTH_TOKEN_ENDPOINT`) at startup.
///
/// Object safety: the trait is object-safe; this concrete
/// type is `Send + Sync + Debug`.
#[derive(Debug, Clone)]
pub struct OAuth2Config {
    /// The OAuth 2.0 `client_id` issued by the IdP during
    /// client registration.
    client_id: String,

    /// The OAuth 2.0 `client_secret` issued by the IdP during
    /// client registration. Confidential clients only;
    /// public clients (mobile / SPA using PKCE only) leave
    /// this empty.
    client_secret: String,

    /// The IdP's authorization endpoint (RFC 6749 § 3.1). The
    /// consumer's redirect handler sends the user-agent here
    /// with `response_type=code`, `client_id`, `redirect_uri`,
    /// `scope`, `state`, and (for PKCE) `code_challenge`.
    authorization_endpoint: String,

    /// The IdP's token endpoint (RFC 6749 § 3.2). The
    /// adapter POSTs the authorization-code exchange here
    /// once the IdP redirects back to the consumer's
    /// callback URL.
    token_endpoint: String,
}

impl OAuth2Config {
    /// Constructs a new [`OAuth2Config`]. All four fields are
    /// required; the consumer wires them from environment
    /// variables at startup.
    #[must_use]
    pub fn new(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        authorization_endpoint: impl Into<String>,
        token_endpoint: impl Into<String>,
    ) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            authorization_endpoint: authorization_endpoint.into(),
            token_endpoint: token_endpoint.into(),
        }
    }

    /// Returns the configured `client_id`.
    #[must_use]
    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    /// Returns the configured `client_secret`.
    #[must_use]
    pub fn client_secret(&self) -> &str {
        &self.client_secret
    }

    /// Returns the IdP's authorization endpoint URL.
    #[must_use]
    pub fn authorization_endpoint(&self) -> &str {
        &self.authorization_endpoint
    }

    /// Returns the IdP's token endpoint URL.
    #[must_use]
    pub fn token_endpoint(&self) -> &str {
        &self.token_endpoint
    }
}

// ---------------------------------------------------------------------------
// OAuth2AuthProvider
// ---------------------------------------------------------------------------

/// OAuth 2.0 / OpenID Connect [`AuthProvider`] reference
/// implementation.
///
/// Wraps a [`JwtAuthProvider`] so that, once authorization-code
/// + PKCE exchange ships, the post-callback [`Session`] can be
/// minted as a JWT and surfaced through the same
/// `Authorization: Bearer ...` plumbing the rest of the engine
/// uses. Until then, only `Credential::Oauth2` is intentionally
/// rejected; everything else (Bearer token validation, refresh,
/// revocation) is forwarded to the wrapped provider.
///
/// Construct with [`OAuth2AuthProvider::new`] and the provider
/// is wired into the engine at startup alongside the other
/// [`AuthProvider`] adapters (see
/// `docs/ports/authentication.md` § "Configuration").
///
/// Object safety: the trait is object-safe; this concrete
/// type is `Send + Sync + Debug`.
#[derive(Debug, Clone)]
pub struct OAuth2AuthProvider {
    /// The OAuth 2.0 / OIDC client configuration.
    config: OAuth2Config,
    /// The JWT provider that owns HMAC signing, token
    /// validation, refresh, and revocation. Once
    /// authorization-code + PKCE exchange ships, the
    /// post-callback [`Session`] flows through this wrapped
    /// provider so consumers see a uniform
    /// `Authorization: Bearer ...` flow end-to-end.
    jwt: JwtAuthProvider,
}

impl OAuth2AuthProvider {
    /// Constructs an OAuth 2.0 / OIDC provider that delegates
    /// token validation, refresh, and revocation to the given
    /// [`JwtAuthProvider`]. The wrapped provider is the source
    /// of truth for the HMAC secret, issuer / audience, and
    /// the in-process revocation set; this constructor
    /// therefore accepts only the OAuth client registration
    /// data ([`OAuth2Config`]) and the wrapped JWT provider.
    ///
    /// Once authorization-code + PKCE exchange lands, this
    /// constructor grows an additional JWKS-fetching
    /// capability; the current signature is intentionally
    /// minimal so the `Provider` trait object is satisfied
    /// today without committing to a network-capable shape.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError::Malformed`] if any of the four
    /// configuration fields is empty. A misconfigured
    /// production deploy must fail loudly at startup rather
    /// than silently accepting every OAuth callback as
    /// invalid.
    pub fn new(config: OAuth2Config, jwt: JwtAuthProvider) -> Result<Self, AuthError> {
        if config.client_id.trim().is_empty() {
            return Err(AuthError::Malformed(
                "OAuth2Config.client_id is empty; set OAUTH_CLIENT_ID before starting the \
                 consumer (PORT-AUTH-OAUTH2 rejects empty client_id at construction)"
                    .to_owned(),
            ));
        }
        if config.client_secret.trim().is_empty() {
            return Err(AuthError::Malformed(
                "OAuth2Config.client_secret is empty; set OAUTH_CLIENT_SECRET before starting \
                 the consumer (confidential clients require a non-empty client_secret)"
                    .to_owned(),
            ));
        }
        if config.authorization_endpoint.trim().is_empty() {
            return Err(AuthError::Malformed(
                "OAuth2Config.authorization_endpoint is empty; set OAUTH_AUTHORIZATION_ENDPOINT \
                 before starting the consumer"
                    .to_owned(),
            ));
        }
        if config.token_endpoint.trim().is_empty() {
            return Err(AuthError::Malformed(
                "OAuth2Config.token_endpoint is empty; set OAUTH_TOKEN_ENDPOINT before starting \
                 the consumer"
                    .to_owned(),
            ));
        }
        Ok(Self { config, jwt })
    }

    /// Returns a reference to the [`OAuth2Config`] so callers
    /// can introspect or log the OAuth client registration
    /// data. **Never log the `client_secret` field**;
    /// consumers that need a redacted view must implement
    /// their own redaction.
    #[must_use]
    pub fn config(&self) -> &OAuth2Config {
        &self.config
    }

    /// Returns a reference to the wrapped [`JwtAuthProvider`]
    /// so callers can introspect or extend the underlying
    /// token plumbing (e.g. layer a custom claim shape, or
    /// read the configured issuer / audience for diagnostics).
    #[must_use]
    pub fn jwt_provider(&self) -> &JwtAuthProvider {
        &self.jwt
    }
}

// ---------------------------------------------------------------------------
// AuthProvider impl
// ---------------------------------------------------------------------------

#[async_trait]
impl AuthProvider for OAuth2AuthProvider {
    async fn authenticate(&self, credential: Credential) -> Result<Session, AuthError> {
        match credential {
            // OAuth callback handling (authorization-code +
            // PKCE exchange against token_endpoint, ID token
            // verification against JWKS, then session minting
            // through the wrapped JWT provider) lands in a
            // later phase. Until then, reject the credential
            // outright — never silently translate it to an
            // empty session or to the public-school anonymous
            // session, both of which would be security
            // regressions.
            Credential::Oauth2 { .. } => Err(AuthError::InvalidCredentials),
            // Everything else (Bearer tokens, ...) delegates
            // to the wrapped JwtAuthProvider so consumers can
            // wire a single AuthProvider that handles both the
            // post-OAuth JWT and the per-request bearer header.
            // This is intentional: the JwtAuthProvider already
            // rejects UsernamePassword / Saml / ApiKey /
            // Biometric / Anonymous credentials with
            // InvalidCredentials, so the delegation is a
            // no-op for those variants.
            other => self.jwt.authenticate(other).await,
        }
    }

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
    use crate::port::{AuthScheme, BearerToken};

    fn jwt_with_test_key() -> JwtAuthProvider {
        // 32-byte key, satisfying HS256 minimum-length
        // (RFC 7518 § 3.2) without colliding with the
        // "educore" default issuer used elsewhere.
        JwtAuthProviderBuilder::new()
            .signing_key(b"test-key-for-oauth2-scaffold-32!!".to_vec())
            .issuer("educore-oauth2-test")
            .audience("educore")
            .access_ttl(Duration::from_secs(3600))
            .refresh_ttl(Duration::from_secs(7 * 24 * 3600))
            .build()
    }

    fn valid_config() -> OAuth2Config {
        OAuth2Config::new(
            "test-client-id",
            "test-client-secret",
            "https://idp.example.com/oauth2/authorize",
            "https://idp.example.com/oauth2/token",
        )
    }

    // -----------------------------------------------------------------
    // Construction / configuration
    // -----------------------------------------------------------------

    #[test]
    fn test_oauth2_provider_constructs_with_valid_config() {
        let provider =
            OAuth2AuthProvider::new(valid_config(), jwt_with_test_key()).expect("valid config");
        let cfg = provider.config();
        assert_eq!(cfg.client_id(), "test-client-id");
        assert_eq!(cfg.client_secret(), "test-client-secret");
        assert_eq!(
            cfg.authorization_endpoint(),
            "https://idp.example.com/oauth2/authorize"
        );
        assert_eq!(cfg.token_endpoint(), "https://idp.example.com/oauth2/token");
        // The wrapped JWT provider is reachable.
        let _ = provider.jwt_provider();
    }

    #[test]
    fn test_oauth2_provider_is_clone_and_debug() {
        let provider =
            OAuth2AuthProvider::new(valid_config(), jwt_with_test_key()).expect("valid config");
        let clone = provider.clone();
        // Debug render does not panic.
        let _ = format!("{provider:?}");
        let _ = format!("{clone:?}");
    }

    // -----------------------------------------------------------------
    // Missing-config rejection (PORT-AUTH-OAUTH2 gap-closing)
    // -----------------------------------------------------------------

    #[test]
    fn test_oauth2_provider_rejects_empty_client_id() {
        let cfg = OAuth2Config::new(
            "",
            "test-client-secret",
            "https://idp.example.com/oauth2/authorize",
            "https://idp.example.com/oauth2/token",
        );
        let err = OAuth2AuthProvider::new(cfg, jwt_with_test_key())
            .expect_err("empty client_id must error, not panic");
        assert!(
            matches!(err, AuthError::Malformed(_)),
            "expected Malformed error, got {err:?}"
        );
    }

    #[test]
    fn test_oauth2_provider_rejects_empty_client_secret() {
        let cfg = OAuth2Config::new(
            "test-client-id",
            "",
            "https://idp.example.com/oauth2/authorize",
            "https://idp.example.com/oauth2/token",
        );
        let err = OAuth2AuthProvider::new(cfg, jwt_with_test_key())
            .expect_err("empty client_secret must error, not panic");
        assert!(
            matches!(err, AuthError::Malformed(_)),
            "expected Malformed error, got {err:?}"
        );
    }

    #[test]
    fn test_oauth2_provider_rejects_empty_authorization_endpoint() {
        let cfg = OAuth2Config::new(
            "test-client-id",
            "test-client-secret",
            "",
            "https://idp.example.com/oauth2/token",
        );
        let err = OAuth2AuthProvider::new(cfg, jwt_with_test_key())
            .expect_err("empty authorization_endpoint must error, not panic");
        assert!(
            matches!(err, AuthError::Malformed(_)),
            "expected Malformed error, got {err:?}"
        );
    }

    #[test]
    fn test_oauth2_provider_rejects_empty_token_endpoint() {
        let cfg = OAuth2Config::new(
            "test-client-id",
            "test-client-secret",
            "https://idp.example.com/oauth2/authorize",
            "",
        );
        let err = OAuth2AuthProvider::new(cfg, jwt_with_test_key())
            .expect_err("empty token_endpoint must error, not panic");
        assert!(
            matches!(err, AuthError::Malformed(_)),
            "expected Malformed error, got {err:?}"
        );
    }

    // -----------------------------------------------------------------
    // Credential handling: OAuth2 rejected, others delegated
    // -----------------------------------------------------------------

    #[tokio::test]
    async fn test_oauth2_provider_rejects_oauth_credential_with_invalid_credentials() {
        let provider =
            OAuth2AuthProvider::new(valid_config(), jwt_with_test_key()).expect("valid config");
        let credential = Credential::Oauth2 {
            code: "auth-code-from-idp".to_owned(),
            redirect_uri: "https://app.example.com/callback".to_owned(),
            code_verifier: Some("verifier-string".to_owned()),
        };
        let err = provider
            .authenticate(credential)
            .await
            .expect_err("OAuth credential must reject until exchange lands");
        assert_eq!(err, AuthError::InvalidCredentials);
    }

    #[tokio::test]
    async fn test_oauth2_provider_delegates_bearer_to_wrapped_jwt() {
        let jwt = jwt_with_test_key();
        let provider = OAuth2AuthProvider::new(valid_config(), jwt.clone()).expect("valid config");

        // Round-trip a Bearer credential through the OAuth2
        // provider. The OAuth2 provider must NOT consume the
        // token itself; it must forward to the wrapped JWT
        // provider so the post-callback JWT and the per-request
        // bearer header share the same validation path.
        let token_str = b"any-token-string-does-not-need-to-be-valid".to_vec();
        let bearer = BearerToken::from(String::from_utf8(token_str).expect("ascii"));
        let err = provider
            .authenticate(Credential::Bearer(bearer))
            .await
            .expect_err("malformed bearer must error, not panic");
        // The wrapped provider maps invalid JWTs to
        // AuthError::Malformed; we only assert it's the same
        // error class the wrapped provider would produce, not
        // a panic or a security-regression (e.g. an anonymous
        // session on a malformed token).
        assert!(
            matches!(
                err,
                AuthError::Malformed(_) | AuthError::Expired | AuthError::Revoked
            ),
            "expected wrapped-provider error variant, got {err:?}"
        );

        // Reference jwt to keep the variable live across the test.
        let _ = jwt;
    }

    #[tokio::test]
    async fn test_oauth2_provider_validate_delegates_to_wrapped_jwt() {
        let jwt = jwt_with_test_key();
        let provider = OAuth2AuthProvider::new(valid_config(), jwt).expect("valid config");

        // A non-Bearer scheme must be rejected by the wrapped
        // provider as Malformed; the OAuth2 provider must NOT
        // soften this into InvalidCredentials or worse.
        let token = AuthToken {
            scheme: AuthScheme::Cookie,
            value: "session=opaque".to_owned(),
            metadata: BTreeMap::new(),
        };
        let err = provider
            .validate(&token)
            .await
            .expect_err("cookie scheme must be rejected");
        assert!(
            matches!(err, AuthError::Malformed(_)),
            "expected Malformed from wrapped provider, got {err:?}"
        );
    }

    #[tokio::test]
    async fn test_oauth2_provider_revoke_delegates_to_wrapped_jwt() {
        let jwt = jwt_with_test_key();
        let provider = OAuth2AuthProvider::new(valid_config(), jwt).expect("valid config");
        let token = AuthToken {
            scheme: AuthScheme::Bearer,
            value: "not-a-real-jwt".to_owned(),
            metadata: BTreeMap::new(),
        };
        let err = provider
            .revoke(&token)
            .await
            .expect_err("revoke of malformed bearer must error, not panic");
        assert!(
            matches!(err, AuthError::Malformed(_)),
            "expected Malformed from wrapped provider, got {err:?}"
        );
    }

    #[tokio::test]
    async fn test_oauth2_provider_refresh_delegates_to_wrapped_jwt() {
        let jwt = jwt_with_test_key();
        let provider = OAuth2AuthProvider::new(valid_config(), jwt).expect("valid config");
        let token = AuthToken {
            scheme: AuthScheme::Bearer,
            value: "not-a-real-jwt".to_owned(),
            metadata: BTreeMap::new(),
        };
        let err = provider
            .refresh(&token)
            .await
            .expect_err("refresh of malformed bearer must error, not panic");
        assert!(
            matches!(err, AuthError::Malformed(_)),
            "expected Malformed from wrapped provider, got {err:?}"
        );
    }
}
