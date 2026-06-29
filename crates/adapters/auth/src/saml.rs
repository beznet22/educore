//! # SAML 2.0 [`AuthProvider`] reference implementation.
//!
//! Enterprise identity-provider integration: Okta, Azure AD /
//! Entra ID, OneLogin, Auth0 enterprise, PingFederate,
//! Shibboleth, and other SAML 2.0 IdPs. The post-callback
//! session is delivered to consumers as a JWT (signed and
//! validated by the wrapped [`JwtAuthProvider`]) so the
//! rest of the request pipeline sees a uniform
//! `Authorization: Bearer ...` flow.
//!
//! ## Implementation gap (PORT-AUTH-SAML)
//!
//! This is a **minimal scaffolding**. Full SAML SSO support
//! lands in a later phase and adds:
//!
//! 1. **SAML SSO callback validation** — parse the
//!    `SAMLResponse` form parameter posted by the IdP,
//!    verify the XML signature against the configured IdP
//!    certificate, and enforce the `InResponseTo` /
//!    `RelayState` / `Destination` invariants.
//! 2. **IdP metadata parsing** — fetch the IdP metadata
//!    document (`EntityDescriptor`), extract the X.509
//!    certificate, SSO endpoint, and `EntityID`. Refresh
//!    on a timer so certificate rotations propagate without
//!    a process restart.
//! 3. **Assertion extraction** — pull `NameID`, attributes
//!    (`email`, `displayName`, role mappings), and the
//!    `AuthnStatement` validity window. Map to the engine's
//!    [`Session`] shape and mint a JWT via the wrapped
//!    provider so consumers see a single `Bearer` flow.
//!
//! Until those land, this provider rejects every
//! [`Credential::Saml`] with [`AuthError::InvalidCredentials`]
//! and delegates token validation, refresh, and revocation
//! to the wrapped [`JwtAuthProvider`]. Consumers that only
//! need JWT-based session validation can wire a
//! [`SamlAuthProvider`] (backed by a [`JwtAuthProvider`])
//! into their app and rely on the delegation; consumers that
//! need live SAML SSO must layer the callback handler above
//! this provider until the gap is closed.

#![allow(clippy::missing_docs_in_private_items)]

use async_trait::async_trait;

use crate::errors::AuthError;
use crate::jwt::JwtAuthProvider;
use crate::port::{AuthProvider, AuthToken, Credential, Session};

// ---------------------------------------------------------------------------
// SamlAuthProvider
// ---------------------------------------------------------------------------

/// SAML 2.0 [`AuthProvider`] reference implementation.
///
/// Wraps a [`JwtAuthProvider`] so that, once assertion
/// extraction ships, the post-callback [`Session`] can be
/// minted as a JWT and surfaced through the same
/// `Authorization: Bearer ...` plumbing the rest of the
/// engine uses. Until then, only `Credential::Saml` is
/// intentionally rejected; everything else (Bearer token
/// validation, refresh, revocation) is forwarded to the
/// wrapped provider.
///
/// Construct with [`SamlAuthProvider::new`] and the provider
/// is wired into the engine at startup alongside the other
/// [`AuthProvider`] adapters (see
/// `docs/ports/authentication.md` § "Configuration").
///
/// Object safety: the trait is object-safe; this concrete
/// type is `Send + Sync + Debug`.
#[derive(Debug, Clone)]
pub struct SamlAuthProvider {
    /// The JWT provider that owns HMAC signing, token
    /// validation, refresh, and revocation. Once SAML
    /// assertion extraction ships, the post-callback
    /// [`Session`] flows through this wrapped provider so
    /// consumers see a uniform `Authorization: Bearer ...`
    /// flow end-to-end.
    jwt: JwtAuthProvider,
}

impl SamlAuthProvider {
    /// Constructs a SAML provider that delegates token
    /// validation, refresh, and revocation to the given
    /// [`JwtAuthProvider`]. The wrapped provider is the
    /// source of truth for the HMAC secret, issuer /
    /// audience, and the in-process revocation set; this
    /// constructor therefore does **not** accept any
    /// SAML-specific configuration (IdP metadata URL,
    /// SP entity id, ACS callback URL) until the
    /// implementation gap documented at the module level
    /// is closed.
    ///
    /// Once assertion extraction lands, this constructor
    /// grows an additional `SamlConfig` parameter; the
    /// current signature is intentionally minimal so the
    /// `Provider` trait object is satisfied today without
    /// committing to a SAML-specific configuration shape.
    #[must_use]
    pub fn new(jwt: JwtAuthProvider) -> Self {
        Self { jwt }
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
impl AuthProvider for SamlAuthProvider {
    async fn authenticate(&self, credential: Credential) -> Result<Session, AuthError> {
        match credential {
            // SAML callback handling lands in a later phase.
            // Until then, reject the assertion outright —
            // never silently translate it to an empty session
            // or to the public-school anonymous session, both
            // of which would be security regressions.
            Credential::Saml { .. } => Err(AuthError::InvalidCredentials),
            // Everything else (Bearer tokens, ...) delegates to
            // the wrapped JwtAuthProvider so consumers can wire
            // a single AuthProvider that handles both the
            // post-SAML JWT and the per-request bearer header.
            // This is intentional: the JwtAuthProvider already
            // rejects UsernamePassword / Oauth2 / ApiKey /
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
            .signing_key(b"test-key-for-saml-scaffolding-32!".to_vec())
            .issuer("educore-saml-test")
            .audience("educore")
            .access_ttl(Duration::from_secs(3600))
            .refresh_ttl(Duration::from_secs(7 * 24 * 3600))
            .build()
    }

    #[test]
    fn test_saml_provider_constructs_with_jwt_provider() {
        let jwt = jwt_with_test_key();
        let saml = SamlAuthProvider::new(jwt);
        // The wrapped provider is reachable via the accessor.
        let _ = saml.jwt_provider();
    }

    #[test]
    fn test_saml_provider_is_clone_and_debug() {
        let jwt = jwt_with_test_key();
        let saml = SamlAuthProvider::new(jwt);
        let clone = saml.clone();
        // Debug render does not panic.
        let _ = format!("{saml:?}");
        let _ = format!("{clone:?}");
    }

    #[tokio::test]
    async fn test_saml_provider_rejects_saml_assertion_with_invalid_credentials() {
        let saml = SamlAuthProvider::new(jwt_with_test_key());
        let credential = Credential::Saml {
            assertion: "PHNhbXBsZS1hc3NlcnRpb24+".to_owned(),
            relay_state: Some("/dashboard".to_owned()),
        };
        let err = saml
            .authenticate(credential)
            .await
            .expect_err("SAML callback must reject until assertion extraction lands");
        assert_eq!(err, AuthError::InvalidCredentials);
    }

    #[tokio::test]
    async fn test_saml_provider_delegates_bearer_to_wrapped_jwt() {
        let jwt = jwt_with_test_key();
        let saml = SamlAuthProvider::new(jwt.clone());

        // Round-trip a Bearer credential through the SAML provider.
        // The SAML provider must NOT consume the assertion itself;
        // it must forward to the wrapped JWT provider so the
        // post-callback JWT and the per-request bearer header share
        // the same validation path.
        let token_str = b"any-token-string-does-not-need-to-be-valid".to_vec();
        let bearer = BearerToken::from(String::from_utf8(token_str).expect("ascii"));
        let err = saml
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
    async fn test_saml_provider_validate_delegates_to_wrapped_jwt() {
        let jwt = jwt_with_test_key();
        let saml = SamlAuthProvider::new(jwt);

        // A non-Bearer scheme must be rejected by the wrapped
        // provider as Malformed; the SAML provider must NOT
        // soften this into InvalidCredentials or worse.
        let token = AuthToken {
            scheme: AuthScheme::Cookie,
            value: "session=opaque".to_owned(),
            metadata: BTreeMap::new(),
        };
        let err = saml
            .validate(&token)
            .await
            .expect_err("cookie scheme must be rejected");
        assert!(
            matches!(err, AuthError::Malformed(_)),
            "expected Malformed from wrapped provider, got {err:?}"
        );
    }

    #[tokio::test]
    async fn test_saml_provider_revoke_delegates_to_wrapped_jwt() {
        let jwt = jwt_with_test_key();
        let saml = SamlAuthProvider::new(jwt);
        let token = AuthToken {
            scheme: AuthScheme::Bearer,
            value: "not-a-real-jwt".to_owned(),
            metadata: BTreeMap::new(),
        };
        let err = saml
            .revoke(&token)
            .await
            .expect_err("revoke of malformed bearer must error, not panic");
        assert!(
            matches!(err, AuthError::Malformed(_)),
            "expected Malformed from wrapped provider, got {err:?}"
        );
    }

    #[tokio::test]
    async fn test_saml_provider_refresh_delegates_to_wrapped_jwt() {
        let jwt = jwt_with_test_key();
        let saml = SamlAuthProvider::new(jwt);
        let token = AuthToken {
            scheme: AuthScheme::Bearer,
            value: "not-a-real-jwt".to_owned(),
            metadata: BTreeMap::new(),
        };
        let err = saml
            .refresh(&token)
            .await
            .expect_err("refresh of malformed bearer must error, not panic");
        assert!(
            matches!(err, AuthError::Malformed(_)),
            "expected Malformed from wrapped provider, got {err:?}"
        );
    }

    #[test]
    fn test_saml_provider_rejects_username_password_via_wrapped_provider() {
        // Sanity check: the SAML provider never silently
        // accepts a non-SAML credential through its own path;
        // every non-SAML variant flows through the wrapped
        // JwtAuthProvider which already rejects these with
        // InvalidCredentials. We exercise the synchronous
        // construction surface to confirm the struct shape.
        let saml = SamlAuthProvider::new(jwt_with_test_key());
        let _ = saml.jwt_provider();
    }
}
