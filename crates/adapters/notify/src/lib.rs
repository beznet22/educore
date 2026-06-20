//! # educore-notify
//!
//!  Notification port, email, SMS, push, in-app, chat, voice, webhook adapters.
//!
//! This crate is a member of the Educore workspace. See
//! `docs/architecture.md` and the domain spec in
//! `docs/specs/` for behavioral details.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// Package name constant. Re-exported so consumers can assert they
/// are using the right crate version at compile time.
pub const PACKAGE_NAME: &str = "educore-notify";

/// Package version at compile time.
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Notification port trait and request / response / value types.
/// Owned by B.2; wired in here so downstream modules can reach it.
pub mod errors;
/// Notification port (request / response / value types). Owned by
/// B.2; wired in here so downstream modules can reach it.
pub mod port;

/// SMS [`NotificationProvider`](port::NotificationProvider) reference
/// implementation backed by an HTTP gateway.
pub mod sms;

/// Email [`NotificationProvider`](port::NotificationProvider) reference
/// implementation backed by SMTP via the `lettre` crate.
pub mod email;

// ---------------------------------------------------------------------------
// Re-exports (crate prelude)
// ---------------------------------------------------------------------------

pub use crate::email::{EmailProvider, EmailProviderBuilder};
pub use crate::sms::{SmsProvider, SmsProviderBuilder};

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn package_metadata_is_set() {
        assert_eq!(PACKAGE_NAME, "educore-notify");
        assert!(!PACKAGE_VERSION.is_empty());
    }
}
