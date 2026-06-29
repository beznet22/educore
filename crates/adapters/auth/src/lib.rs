//! # educore-auth
//!
//! Authentication port, local-password, OAuth2, SAML, JWT adapter
//! implementations.
//!
//! This crate is a member of the Educore workspace. See
//! `docs/architecture.md` and `docs/ports/authentication.md` for
//! the port specification. The full port surface lives in
//! [`port`]; the error type lives in [`errors`]; the reference
//! adapter implementations land in follow-up microtasks
//! (`educore-auth-jwt`, `educore-auth-local-password`,
//! `educore-auth-oauth2`, `educore-auth-saml`,
//! `educore-auth-apikey`).

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// Authentication port trait, request/response types, and the
/// `BearerToken` alias.
///
/// See `docs/ports/authentication.md` for the authoritative
/// specification. This module is the **port-only** surface;
/// reference implementations live in separate microtasks.
pub mod port;

/// JWT-based [`port::AuthProvider`] reference implementation.
///
/// Lands in microtask A.3a. See [`jwt::JwtAuthProvider`] for the
/// builder API.
pub mod jwt;

/// SAML 2.0 [`port::AuthProvider`] reference implementation.
///
/// Enterprise identity-provider integration (Okta, Azure AD /
/// Entra ID, OneLogin, Auth0 enterprise, PingFederate,
/// Shibboleth, ...). Delegates token validation, refresh, and
/// revocation to [`jwt::JwtAuthProvider`] so the post-callback
/// session is surfaced as a uniform `Authorization: Bearer ...`
/// JWT. Full SAML SSO callback validation, IdP metadata parsing,
/// and assertion extraction land in a later phase; see
/// [`saml::SamlAuthProvider`] for the implementation gap.
pub mod saml;

/// In-memory reference implementation of the four port-driven
/// OAuth-related repository traits declared in
/// [`educore_operations::repository`](educore_operations::repository):
/// `OAuthAccessTokenRepository`, `OAuthClientRepository`,
/// `PasswordResetRepository`, and `MigrationRepository`.
///
/// Lands in microtask A.3b. See [`oauth_store::InMemoryOAuthStore`]
/// for the reference impl.
pub mod oauth_store;

/// Pure-helper services for the auth port: JWT claim validation,
/// OAuth scope checks, Argon2id password hashing, and RFC 6238
/// TOTP code generation / verification.
///
/// Lands in microtask A.4. See [`services::JwtService`],
/// [`services::OAuthScopeService`], [`services::PasswordService`],
/// and [`services::MfaService`].
pub mod services;

/// The [`errors::AuthError`] enum — the universal error type for
/// the authentication port.
pub mod errors;

/// Per-IP rate limiter and lockout for credential endpoints.
///
/// Implements QW-8 (ADAPT-AUTH-007). See
/// [`rate_limit::RateLimiter`] for the limiter, [`rate_limit::Clock`]
/// for the time abstraction, and [`rate_limit::RateLimitError`]
/// for the failure modes.
pub mod rate_limit;

/// Convenience re-exports of the most-used types. Consumers of
/// the auth crate typically `use educore_auth::prelude::*;`
/// once at the top of a file.
pub mod prelude {
    pub use crate::errors::AuthError;
    pub use crate::jwt::{JwtAuthProvider, JwtAuthProviderBuilder, JwtClaims, JwtSecretSource};
    pub use crate::oauth_store::InMemoryOAuthStore;
    pub use crate::port::{
        AuthProvider, AuthScheme, AuthToken, BearerToken, Credential, RbacPort, Session,
    };
    pub use crate::rate_limit::{RateLimitError, RateLimiter, RateLimiterConfig};
    pub use crate::saml::SamlAuthProvider;
    pub use crate::services::{
        JwtService, MfaService, OAuthScopeService, PasswordService, SecretString,
    };
}

/// Package name constant. Re-exported so consumers can assert they
/// are using the right crate version at compile time.
pub const PACKAGE_NAME: &str = "educore-auth";

/// Package version at compile time.
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn package_metadata_is_set() {
        assert_eq!(PACKAGE_NAME, "educore-auth");
        assert!(!PACKAGE_VERSION.is_empty());
    }
}
