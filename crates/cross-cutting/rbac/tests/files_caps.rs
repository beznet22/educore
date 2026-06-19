//! # Phase 15 Files caps round-trip test

#![cfg(test)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use educore_rbac::value_objects::Capability;

const FILES_VARIANTS: &[Capability] = &[
    Capability::FilesPut,
    Capability::FilesGet,
    Capability::FilesDelete,
    Capability::FilesSignedUrl,
    Capability::FilesCopy,
    Capability::FilesMove,
    Capability::FilesVisibilityChange,
    Capability::FilesLifecycle,
];

#[test]
fn files_capabilities_round_trip() {
    assert_eq!(FILES_VARIANTS.len(), 8);
    for cap in FILES_VARIANTS {
        let wire = cap.as_str();
        assert!(wire.starts_with("Files."), "{cap:?}.as_str() = {wire} should start with Files.");
        let parsed = Capability::from_str_opt(wire)
            .or_else(|| wire.parse::<Capability>().ok())
            .unwrap_or_else(|| panic!("failed to parse {wire}"));
        assert_eq!(parsed, *cap, "round-trip mismatch for {cap:?}");
    }
}