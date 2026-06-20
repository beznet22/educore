//! # educore-files
//!
//!  File storage port, S3-compatible, GCS, local filesystem adapters.
//!
//! This crate is a member of the Educore workspace. See
//! `docs/architecture.md` and the port contract in
//! `docs/ports/file-storage.md` for behavioral details.
//!
//! # Module map
//!
//! - [`port`] — the [`FileStorage`](port::FileStorage) trait
//!   (8 methods, object-safe) and all the supporting request,
//!   response, and value types (`PutRequest`, `FileReference`,
//!   `FileMetadata`, `SignedUrlOptions`, …).
//! - [`errors`] — the [`FileStorageError`](errors::FileStorageError)
//!   enum returned by every adapter method.
//! - [`s3`] — the AWS S3 reference implementation
//!   ([`S3FileStorage`](s3::S3FileStorage) +
//!   [`S3FileStorageBuilder`](s3::S3FileStorageBuilder)).
//! - [`local`] — the local filesystem reference implementation
//!   ([`LocalFileStorage`](local::LocalFileStorage) +
//!   [`LocalFileStorageBuilder`](local::LocalFileStorageBuilder))
//!   for development and offline-mode use.
//!
//! # Deviations from the spec
//!
//! The crate's `Cargo.toml` is intentionally minimal (only
//! `core`, `platform`, `events`, `tokio`, `async-trait`), so the
//! port uses **stdlib-only** value representations. See the
//! module-level doc in [`port`] for the full list. Adapters that
//! want the spec's idiomatic types (`uuid::Uuid`, `url::Url`,
//! `bytes::Bytes`, `futures::Stream`) may wrap the stdlib shapes
//! at their boundary.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// Error types for the [`FileStorage`](port::FileStorage) port.
pub mod errors;

/// The [`FileStorage`](port::FileStorage) trait and all
/// supporting types (`PutRequest`, `FileReference`,
/// `FileMetadata`, `SignedUrlOptions`, …).
pub mod port;

/// AWS S3 reference implementation of the
/// [`FileStorage`](port::FileStorage) port. See the module-level
/// doc in [`s3`] for the tenant-namespacing, content-hashing, and
/// builder API contracts.
pub mod s3;

/// Local filesystem reference implementation of the
/// [`FileStorage`](port::FileStorage) port. See the module-level
/// doc in [`local`] for the path-safety, HMAC-SHA256 signed-URL,
/// and streaming-download contracts.
pub mod local;

/// Package name constant. Re-exported so consumers can assert
/// they are using the right crate version at compile time.
pub const PACKAGE_NAME: &str = "educore-files";

/// Package version at compile time.
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Convenience re-exports of the port's most-used types.
///
/// Consumers of the port should
/// `use educore_files::prelude::*;` once at the top of a file
/// to pull in the trait, the request/response shapes, and the
/// error type without naming each module.
pub mod prelude {
    pub use crate::errors::{FileStorageError, InfrastructureError};
    pub use crate::local::{LocalFileStorage, LocalFileStorageBuilder};
    pub use crate::port::{
        Checksum, ContentType, FileKey, FileMetadata, FileReference, FileStorage, FileStream,
        IdempotencyKey, PutRequest, Result, SignedUrlMethod, SignedUrlOptions, StorageClass,
        Visibility,
    };
    pub use crate::s3::{S3FileStorage, S3FileStorageBuildError, S3FileStorageBuilder};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_metadata_is_set() {
        assert_eq!(PACKAGE_NAME, "educore-files");
        assert!(!PACKAGE_VERSION.is_empty());
    }
}
