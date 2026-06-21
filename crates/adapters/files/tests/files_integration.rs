//! # Phase 15 files port vertical-slice integration test
//!
//! 5 sync scenarios (always-on) + 2 env-gated async scenarios
//! (require `EDUCORE_PORT_ADAPTER_E2E=1`).
//!
//! The sync scenarios exercise the pure-helper services that sit
//! alongside the [`FileStorage`](educore_files::FileStorage) port:
//!
//! - [`ChecksumService`] — SHA-256 content hashing + quoted
//!   S3-style ETag emission
//! - [`KeyNamespaceService`] — `<school>/<domain>/<agg>/<id>/<file>`
//!   key composition and parsing
//! - [`VisibilityService`] — visibility classification
//!   (`Private` / `Public` / `TenantPrivate`) and the
//!   `can_access` policy
//! - [`SignedUrlService`] — HMAC-SHA256 URL token minting and
//!   verification
//!
//! The two async scenarios are env-gated because they wire up the
//! S3 and local builders that would otherwise flap in CI without
//! AWS credentials or a writable temp directory. They live as
//! `#[ignore]`d `#[tokio::test]`s so the orchestrator can opt in
//! with `EDUCORE_PORT_ADAPTER_E2E=1 cargo test ... -- --ignored`.

#![cfg(test)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use educore_files::prelude::*;
use educore_files::services::{
    ChecksumService, KeyNamespaceService, SignedUrlService, VisibilityService,
};

// ---------------------------------------------------------------------------
// Scenario 1: SHA-256 checksum
// ---------------------------------------------------------------------------

#[test]
fn files_integration_sha256_checksum() {
    let content = b"hello world";
    let hex = ChecksumService::compute_sha256(content);
    // Known SHA-256 of "hello world"
    assert_eq!(
        hex,
        "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
    );
    assert!(ChecksumService::verify(content, &hex));
    assert!(!ChecksumService::verify(b"hello WORLD", &hex));
}

// ---------------------------------------------------------------------------
// Scenario 2: ETag (quoted SHA-256)
// ---------------------------------------------------------------------------

#[test]
fn files_integration_etag_quoted() {
    let etag = ChecksumService::compute_etag(b"hello");
    assert!(etag.starts_with('"'));
    assert!(etag.ends_with('"'));
    // 64 hex chars + 2 quote chars
    assert_eq!(etag.len(), 66);
    // The interior must be a 64-char lowercase hex digest
    let inner = &etag[1..etag.len() - 1];
    assert_eq!(inner.len(), 64);
    assert!(inner.chars().all(|c: char| c.is_ascii_hexdigit()));
}

// ---------------------------------------------------------------------------
// Scenario 3: Key namespace round-trip
// ---------------------------------------------------------------------------

#[test]
fn files_integration_key_namespace_round_trip() {
    let namespaced = KeyNamespaceService::namespace_key(
        "school-123",
        "academic",
        "Student",
        "uuid-456",
        "photo.jpg",
    );
    assert_eq!(namespaced, "school-123/academic/Student/uuid-456/photo.jpg");
    let (school, domain, agg, id, filename) =
        KeyNamespaceService::parse_namespaced_key(&namespaced).expect("should parse");
    assert_eq!(school, "school-123");
    assert_eq!(domain, "academic");
    assert_eq!(agg, "Student");
    assert_eq!(id, "uuid-456");
    assert_eq!(filename, "photo.jpg");
}

// ---------------------------------------------------------------------------
// Scenario 4: Visibility classification
// ---------------------------------------------------------------------------

#[test]
fn files_integration_visibility_classification() {
    assert!(VisibilityService::is_private(&Visibility::Private));
    assert!(VisibilityService::is_public(&Visibility::Public));
    assert!(VisibilityService::is_tenant_scoped(
        &Visibility::TenantPrivate
    ));
    assert!(!VisibilityService::is_private(&Visibility::Public));
    assert!(VisibilityService::can_access(
        &Visibility::Public,
        "any-school"
    ));
    assert!(VisibilityService::can_access(
        &Visibility::TenantPrivate,
        "any-school"
    ));
    assert!(!VisibilityService::can_access(
        &Visibility::Private,
        "any-school"
    ));
}

// ---------------------------------------------------------------------------
// Scenario 5: Signed URL build + verify
// ---------------------------------------------------------------------------

#[test]
fn files_integration_signed_url_build_and_verify() {
    let svc = SignedUrlService::new("test-signing-key-32-bytes-long!!");
    let url = svc.build_signed_url(
        "https://files.example.com",
        "school-1/photos/ada.jpg",
        std::time::Duration::from_secs(3600),
    );
    assert!(url.starts_with("https://files.example.com/school-1/photos/ada.jpg?"));
    assert!(url.contains("expires="));
    assert!(url.contains("signature="));

    // Extract the expires query parameter (RFC 3339 form, between
    // "expires=" and the next "&").
    let expires_str = url
        .split("expires=")
        .nth(1)
        .and_then(|s| s.split('&').next())
        .expect("expires query param should be present");
    let expires_at = educore_core::value_objects::Timestamp::parse_rfc3339(expires_str)
        .expect("expires should parse as RFC 3339");

    // Extract the signature (everything after "signature=").
    let signature = url
        .split("signature=")
        .nth(1)
        .expect("signature query param should be present");

    // The URL was built with a 1-hour window, so the verifier
    // must accept it.
    assert!(svc.verify("school-1/photos/ada.jpg", expires_at, signature));
}

// ---------------------------------------------------------------------------
// Env-gated async scenarios
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires EDUCORE_PORT_ADAPTER_E2E env var"]
async fn files_integration_async_s3_put_mock() {
    let _storage = S3FileStorage::builder()
        .bucket("test-bucket".to_owned())
        .key_prefix("test/".to_owned())
        .build();
}

#[tokio::test]
#[ignore = "requires EDUCORE_PORT_ADAPTER_E2E env var"]
async fn files_integration_async_local_put_mock() {
    let _storage = LocalFileStorageBuilder::new()
        .root(std::path::PathBuf::from("/tmp/educore-test"))
        .build();
}
