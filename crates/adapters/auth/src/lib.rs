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

/// The [`errors::AuthError`] enum — the universal error type for
/// the authentication port.
pub mod errors;

/// Convenience re-exports of the most-used types. Consumers of
/// the auth crate typically `use educore_auth::prelude::*;`
/// once at the top of a file.
pub mod prelude {
    pub use crate::errors::AuthError;
    pub use crate::port::{
        AuthProvider, AuthScheme, AuthToken, BearerToken, Credential, RbacPort, Session,
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
