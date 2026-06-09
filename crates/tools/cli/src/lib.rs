//! # smsengine-cli
//!
//!  Sample CLI — demonstrates how a consumer wires the engine for daily operations. The engine itself is library-only; this CLI is a reference implementation.
//!
//! This crate is a member of the SMSengine workspace. See
//! `docs/architecture.md` and the domain spec in
//! `docs/specs/` for behavioral details.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// Package name constant. Re-exported so consumers can assert they
/// are using the right crate version at compile time.
pub const PACKAGE_NAME: &str = "smsengine-cli";

/// Package version at compile time.
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn package_metadata_is_set() {
        assert_eq!(PACKAGE_NAME, "smsengine-cli");
        assert!(!PACKAGE_VERSION.is_empty());
    }
}
