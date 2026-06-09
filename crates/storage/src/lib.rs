//! # smsengine-storage
//!
//!  Storage port trait, repository traits, parity test suite, common types.
//!
//! This crate is a member of the SMSengine workspace. See
//! `docs/architecture.md` and the domain spec in
//! `docs/specs/` for behavioral details.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// Package name constant. Re-exported so consumers can assert they
/// are using the right crate version at compile time.
pub const PACKAGE_NAME: &str = "smsengine-storage";

/// Package version at compile time.
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn package_metadata_is_set() {
        assert_eq!(PACKAGE_NAME, "smsengine-storage");
        assert!(!PACKAGE_VERSION.is_empty());
    }
}
