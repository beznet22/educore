//! Error types for the [`FileStorage`](crate::port::FileStorage) port.
//!
//! Per `docs/ports/file-storage.md` § "Error Type", every adapter
//! returns one of the eight variants in [`FileStorageError`]. The
//! enum is intentionally `Debug + PartialEq + Eq` so it can flow
//! through the engine's circuit-breaker / retry middleware without
//! losing context.
//!
//! The `Infrastructure` variant carries a structured
//! [`InfrastructureError`] (a typed wrapper around an opaque
//! message and a `Send + Sync` source chain) rather than
//! `Box<dyn std::error::Error + Send + Sync>` directly so the enum
//! stays `PartialEq + Eq`. Callers that need to lift an arbitrary
//! `Box<dyn Error>` into the port convert it via
//! [`InfrastructureError::from_boxed`].
//!
//! # Deviations from `docs/ports/file-storage.md`
//!
//! The crate's `Cargo.toml` is intentionally minimal (only
//! `core`, `platform`, `events`, `tokio`, `async-trait`), so the
//! error uses **stdlib-only** value representations:
//!
//! - `thiserror` is replaced by manual [`Display`](fmt::Display)
//!   + [`std::error::Error`] impls so the crate does not need to
//!   take a direct dependency on `thiserror`.
//! - `Infrastructure` wraps the structured [`InfrastructureError`]
//!   (an opaque message + `Send + Sync` source chain) rather than
//!   a bare `Box<dyn Error>`. The wrapping preserves
//!   `PartialEq + Eq` on the enum so callers can match against
//!   concrete errors in circuit-breaker middleware.
//! - The enum is **not** `Clone` (the boxed source chain is not
//!   cloneable). Callers that need to retain a copy of an error
//!   after the producer goes out of scope should clone the
//!   `FileStorageError` **before** materialising the
//!   `Infrastructure` variant.

use std::error::Error as StdError;
use std::fmt;

use crate::port::{ContentType, FileKey};

/// Errors returned by the [`FileStorage`](crate::port::FileStorage) port.
///
/// The eight variants are locked to `docs/ports/file-storage.md`
/// § "Error Type" with one deviation: `Infrastructure` wraps the
/// structured [`InfrastructureError`] (an opaque message + `Send +
/// Sync` source chain) rather than a bare `Box<dyn Error>`. The
/// wrapping preserves `PartialEq + Eq` on the enum so callers can
/// match against concrete errors in circuit-breaker middleware.
///
/// Adapters MUST NOT include file contents, signed-URL secrets,
/// or pre-shared signing keys in the diagnostic message; the
/// engine redacts strings whose field name ends in `_secret`,
/// `_token`, or `_key` on its way to the audit log.
#[derive(Debug, PartialEq, Eq)]
pub enum FileStorageError {
    /// The requested file does not exist at the underlying
    /// provider. Carries the [`FileKey`] so callers can correlate
    /// the failure with the original upload intent.
    NotFound(FileKey),

    /// The actor does not have permission to access the requested
    /// file. Returned on cross-tenant attempts, expired signed
    /// URLs, and unauthorised `Visibility::Private` reads.
    PermissionDenied,

    /// The content hash on the read does not match the hash
    /// recorded at upload time. Indicates either corruption in
    /// flight or a tampered object; the engine surfaces this as a
    /// hard failure.
    ChecksumMismatch,

    /// The content to upload exceeds the adapter's configured
    /// maximum. The first field is the actual size in bytes; the
    /// second field is the maximum in bytes.
    TooLarge(u64, u64),

    /// The adapter does not accept the supplied MIME type. Carries
    /// the [`ContentType`] so callers can map the rejection back
    /// to the upload form.
    UnsupportedContentType(ContentType),

    /// The supplied file key failed the adapter's validation
    /// (length, character set, reserved prefix). The inner string
    /// is the adapter's diagnostic message.
    InvalidKey(String),

    /// The requested [`StorageClass`](crate::port::StorageClass) is
    /// not available on the underlying provider / bucket. The
    /// inner string is the class identifier (e.g. `"archive"`).
    StorageClassUnavailable(String),

    /// A non-domain infrastructure failure (network, DNS, TLS,
    /// serialization, throttling). The wrapped
    /// [`InfrastructureError`] is the underlying cause; the
    /// engine surfaces it via [`StdError::source`].
    Infrastructure(InfrastructureError),
}

/// An opaque, `Send + Sync` wrapper around an infrastructure-level
/// error message and an optional boxed source.
///
/// The port cannot use `Box<dyn std::error::Error + Send + Sync>`
/// directly as the variant payload because the resulting
/// [`FileStorageError`] enum would lose the ability to derive
/// `PartialEq + Eq` (the boxed `dyn Error` is not `Eq`). The
/// [`InfrastructureError`] wrapper preserves `PartialEq + Eq` for
/// callers that need to compare errors (e.g. in circuit-breaker
/// middleware) while still retaining the `source()` chain for
/// diagnostics.
///
/// The wrapper is intentionally **not** `Clone` (the boxed source
/// cannot be cloned). Callers that need to retain a copy of an
/// infrastructure error should clone the [`FileStorageError`]
/// before the `Infrastructure` variant is materialised.
#[derive(Debug)]
pub struct InfrastructureError {
    message: String,
    source: Option<Box<dyn StdError + Send + Sync>>,
}

impl InfrastructureError {
    /// Constructs an `InfrastructureError` from a free-form
    /// message. The message is stored verbatim; the adapter is
    /// responsible for redacting any sensitive data (signing
    /// keys, signed-URL tokens, internal hostnames) before
    /// construction.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    /// Constructs an `InfrastructureError` that wraps a boxed
    /// source error. The `Display` output of the source is
    /// appended to the engine's audit log via
    /// [`StdError::source`].
    #[must_use]
    pub fn with_source(
        message: impl Into<String>,
        source: Box<dyn StdError + Send + Sync>,
    ) -> Self {
        Self {
            message: message.into(),
            source: Some(source),
        }
    }

    /// Lifts an arbitrary `Box<dyn Error + Send + Sync>` into an
    /// `InfrastructureError`. The boxed error's `Display` output
    /// becomes the wrapped message and the box is retained as the
    /// `source` chain.
    #[must_use]
    pub fn from_boxed(err: Box<dyn StdError + Send + Sync>) -> Self {
        Self {
            message: err.to_string(),
            source: Some(err),
        }
    }

    /// Returns the wrapped diagnostic message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for InfrastructureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl StdError for InfrastructureError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_deref()
            .map(|e| e as &(dyn StdError + 'static))
    }
}

impl PartialEq for InfrastructureError {
    fn eq(&self, other: &Self) -> bool {
        self.message == other.message
    }
}

impl Eq for InfrastructureError {}

impl fmt::Display for FileStorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(key) => write!(f, "file not found: {key}"),
            Self::PermissionDenied => f.write_str("permission denied"),
            Self::ChecksumMismatch => f.write_str("checksum mismatch"),
            Self::TooLarge(actual, max) => {
                write!(f, "content too large: {actual} bytes, max {max}")
            }
            Self::UnsupportedContentType(ct) => {
                write!(f, "unsupported content type: {ct}")
            }
            Self::InvalidKey(msg) => write!(f, "key invalid: {msg}"),
            Self::StorageClassUnavailable(name) => {
                write!(f, "storage class not available: {name}")
            }
            Self::Infrastructure(err) => write!(f, "infrastructure error: {err}"),
        }
    }
}

impl StdError for FileStorageError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Infrastructure(err) => Some(err),
            _ => None,
        }
    }
}

impl From<InfrastructureError> for FileStorageError {
    fn from(err: InfrastructureError) -> Self {
        Self::Infrastructure(err)
    }
}

impl From<Box<dyn StdError + Send + Sync>> for FileStorageError {
    fn from(err: Box<dyn StdError + Send + Sync>) -> Self {
        Self::Infrastructure(InfrastructureError::from_boxed(err))
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;

    #[test]
    fn display_matches_spec_wording() {
        let key = FileKey::new("students/photos/ada.jpg");
        assert_eq!(
            FileStorageError::NotFound(key.clone()).to_string(),
            "file not found: students/photos/ada.jpg"
        );
        assert_eq!(
            FileStorageError::PermissionDenied.to_string(),
            "permission denied"
        );
        assert_eq!(
            FileStorageError::ChecksumMismatch.to_string(),
            "checksum mismatch"
        );
        assert_eq!(
            FileStorageError::TooLarge(1024, 512).to_string(),
            "content too large: 1024 bytes, max 512"
        );
        let ct = ContentType::new("image/x-bmp");
        assert_eq!(
            FileStorageError::UnsupportedContentType(ct).to_string(),
            "unsupported content type: image/x-bmp"
        );
        assert_eq!(
            FileStorageError::InvalidKey(String::from("empty")).to_string(),
            "key invalid: empty"
        );
        assert_eq!(
            FileStorageError::StorageClassUnavailable(String::from("archive")).to_string(),
            "storage class not available: archive"
        );
    }

    #[test]
    fn infrastructure_error_carries_source() {
        let io_err: Box<dyn StdError + Send + Sync> = Box::new(std::io::Error::new(
            std::io::ErrorKind::ConnectionReset,
            "peer closed",
        ));
        let infra = InfrastructureError::from_boxed(io_err);
        assert!(infra.source().is_some());
        let fs: FileStorageError = infra.into();
        assert!(matches!(fs, FileStorageError::Infrastructure(_)));
        assert!(fs.source().is_some());
    }

    #[test]
    fn error_is_eq() {
        let a = FileStorageError::InvalidKey(String::from("path traversal"));
        let b = FileStorageError::InvalidKey(String::from("path traversal"));
        assert_eq!(a, b);
    }

    #[test]
    fn not_found_carries_file_key() {
        let key = FileKey::new("missing.bin");
        let err = FileStorageError::NotFound(key.clone());
        match err {
            FileStorageError::NotFound(k) => assert_eq!(k, key),
            _ => panic!("expected NotFound"),
        }
    }
}
