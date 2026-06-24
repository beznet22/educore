//! # Local filesystem file storage — reference implementation.
//!
//! [`LocalFileStorage`] is the reference adapter for the
//! [`FileStorage`](crate::port::FileStorage) port. It stores objects
//! on the local filesystem under a configurable root directory
//! and emits `file://` URLs for the [`signed_url`](crate::port::FileStorage::signed_url)
//! method. The implementation is intentionally minimal — it is
//! the offline-mode / development-mode adapter, not the production
//! storage target.
//!
//! # Implementation outline
//!
//! | Port method    | Local I/O                                        |
//! | -------------- | ------------------------------------------------ |
//! | `put`          | `tokio::fs::write` + SHA-256 checksum            |
//! | `get`          | `tokio::fs::File` chunked via `mpsc::channel`    |
//! | `delete`       | `tokio::fs::remove_file` (idempotent)            |
//! | `exists`       | `tokio::fs::metadata` + `is_file()`              |
//! | `head`         | `tokio::fs::metadata`                            |
//! | `signed_url`   | HMAC-SHA256 signed `file://` URL                 |
//! | `copy`         | `tokio::fs::copy`                                |
//! | `move_to`      | `tokio::fs::rename`                              |
//!
//! # Streaming semantics
//!
//! Per the port's [`FileStream`](crate::port::FileStream) type
//! (which is a `tokio::sync::mpsc::Receiver` in this crate's
//! deviation), the `get` method opens the file, spawns a task
//! that reads 4 KB chunks and pushes them through the channel's
//! [`Sender`](tokio::sync::mpsc::Sender), and returns the
//! [`Receiver`](tokio::sync::mpsc::Receiver). The first chunk
//! reaches the consumer promptly; the adapter does not buffer
//! the whole object before the first send.
//!
//! # Path safety
//!
//! Every key is resolved through [`LocalFileStorage::resolve`],
//! which rejects absolute paths and `..` components and confirms
//! the canonical form of the destination stays under the
//! configured `root`. A malicious `FileKey` cannot escape the
//! root via `../` traversal.
//!
//! # Signed URL format
//!
//! ```text
//! file://<root>/<key_prefix>/<key>?expires_in=<secs>&method=<GET|PUT>&token=<hex-hmac-sha256>
//! ```
//!
//! The token is the lowercase hex of `HMAC-SHA256(signing_secret,
//! "<key>|<expires_in>")`. The same `(key, expires_in)` pair
//! therefore always produces the same URL (the contract the
//! `signed_url_is_deterministic` test pins). The current wall
//! clock is intentionally NOT part of the token, so URLs are
//! reproducible across processes and replays. A production
//! adapter that needs hard expiry should embed an absolute
//! timestamp and verify it at fetch time.
//!
//! # Deviations from `docs/ports/file-storage.md`
//!
//! 1. **No `S3` / `GCS` / `Azure` client.** This adapter is local
//!    only. The S3-compatible reference impl lives in
//!    [`crate::s3`] (a separate microtask).
//! 2. **Signed URL is a `file://` URL** with an HMAC-SHA256
//!    token, not an HTTPS presigned URL. The port returns
//!    `String` (URL-formatted UTF-8), so the scheme is the
//!    adapter's choice.
//! 3. **`StorageClass` is always `Hot`.** The local filesystem
//!    has no tiering. Lifecycle rules are out of scope.
//! 4. **No idempotency cache.** `PutRequest::idempotency_key`
//!    is accepted on the wire but not deduplicated — a retry
//!    re-uploads. A production adapter that needs strict
//!    idempotency should hash the key to a local
//!    "already-uploaded" cache and return the cached reference.
//! 5. **HMAC-SHA256 is implemented in-tree.** The `educore-files`
//!    `Cargo.toml` does not declare the `hmac` or `sha2` crates;
//!    the implementation in this module is a FIPS 180-4 / RFC
//!    2104 reference and is small enough to audit by hand.
//! 6. **`overwrite = false` is not enforced.** The local adapter
//!    always overwrites; the spec leaves the precise error to
//!    the adapter and a real S3 adapter would surface a
//!    `PreconditionFailed`.
//!
//! # Security
//!
//! The default `signing_secret` is a compile-time constant
//! suitable only for development. Production callers MUST
//! override it with [`LocalFileStorageBuilder::signing_secret`]
//! and supply a high-entropy secret from a secret manager.

use std::fmt;
use std::path::{Component, Path, PathBuf};
use std::result::Result as StdResult;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;

use educore_core::value_objects::Timestamp;

use crate::errors::{FileStorageError, InfrastructureError};
use crate::port::{
    Checksum, FileKey, FileMetadata, FileReference, FileStorage, FileStream, PutRequest,
    Result as PortResult, SignedUrlMethod, SignedUrlOptions, StorageClass, Visibility,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Read chunk size for the streaming `get` path. 4 KB matches the
/// kernel page size on most platforms and keeps the per-chunk
/// allocation small.
const READ_CHUNK_SIZE: usize = 4096;

/// MPSC channel capacity for the streaming `get` path. 8 chunks
/// (32 KB) is enough to pipeline the file read against the
/// consumer's processing without holding the entire object in
/// memory.
const GET_CHANNEL_CAPACITY: usize = 8;

/// HMAC-SHA256 block size (bytes). The internal SHA-256 block.
const HMAC_BLOCK_SIZE: usize = 64;

/// HMAC-SHA256 inner pad constant (RFC 2104).
const HMAC_IPAD: u8 = 0x36;

/// HMAC-SHA256 outer pad constant (RFC 2104).
const HMAC_OPAD: u8 = 0x5c;

/// Compile-time default signing secret. Dev-only; production
/// callers MUST override via
/// [`LocalFileStorageBuilder::signing_secret`].
const DEFAULT_SIGNING_SECRET: &[u8] =
    b"educore-local-file-storage-default-signing-secret-do-not-use-in-prod";

// ---------------------------------------------------------------------------
// LocalFileStorage
// ---------------------------------------------------------------------------

/// The local filesystem-backed reference impl of
/// [`FileStorage`]. Objects are stored under
/// `root / key_prefix / <key>`. Tenant isolation is the caller's
/// responsibility — the engine namespaces the key with the
/// `school_id` before calling `put`, and the adapter enforces
/// path safety via [`Self::resolve`].
#[derive(Clone)]
pub struct LocalFileStorage {
    root: PathBuf,
    key_prefix: String,
    signing_secret: Arc<Vec<u8>>,
}

impl LocalFileStorage {
    /// Returns the configured filesystem root.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the configured key prefix.
    #[must_use]
    pub fn key_prefix(&self) -> &str {
        &self.key_prefix
    }

    /// Resolves a logical key to a physical path under `root`,
    /// rejecting keys that escape the root via absolute paths or
    /// `..` components.
    ///
    /// The check is purely lexical — the file may or may not
    /// exist. Path safety does not require the file to be
    /// present.
    fn resolve(&self, key: &str) -> Result<PathBuf, FileStorageError> {
        let key_path = Path::new(key);

        // Reject absolute keys outright.
        if key_path.is_absolute() {
            return Err(FileStorageError::InvalidKey(format!(
                "absolute key not allowed: {key}"
            )));
        }

        // Reject any `..` component.
        for component in key_path.components() {
            if let Component::ParentDir = component {
                return Err(FileStorageError::InvalidKey(format!(
                    "key escapes root via '..': {key}"
                )));
            }
        }

        // Compose the physical path. We can't `canonicalize` the
        // full path because the file may not exist yet (for
        // `put`); the lexical checks above are the authoritative
        // guard against `..` escapes, and the parent directory
        // is created by the caller (`put` / `copy` / `move_to`)
        // as needed.
        let combined = self.root.join(&self.key_prefix).join(key);

        // Belt-and-braces: confirm the lexical result still
        // starts with the root, post-normalisation.
        let normalised = normalise_lexical(&combined);
        let normalised_root = normalise_lexical(&self.root);
        if !normalised.starts_with(&normalised_root) {
            return Err(FileStorageError::InvalidKey(format!(
                "key escapes root: {key}"
            )));
        }

        Ok(combined)
    }

    /// Computes the HMAC-SHA256 token for a `key|expires_in` pair.
    fn sign(&self, key: &str, expires_in_secs: u64) -> String {
        let message = format!("{key}|{expires_in_secs}");
        let mac = hmac_sha256(&self.signing_secret, message.as_bytes());
        hex_encode(&mac)
    }

    /// Builds a `file://` URL for the given physical path.
    fn file_url(path: &Path) -> String {
        format!("file://{}", path.display())
    }
}

impl fmt::Debug for LocalFileStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalFileStorage")
            .field("root", &self.root)
            .field("key_prefix", &self.key_prefix)
            .field("signing_secret", &"<redacted>")
            .finish()
    }
}

// ---------------------------------------------------------------------------
// LocalFileStorageBuilder
// ---------------------------------------------------------------------------

/// Builder for [`LocalFileStorage`]. Construct via
/// [`LocalFileStorageBuilder::new`], configure with the chainable
/// setters, then call [`LocalFileStorageBuilder::build`].
#[derive(Clone)]
pub struct LocalFileStorageBuilder {
    root: Option<PathBuf>,
    key_prefix: String,
    signing_secret: Vec<u8>,
}

impl Default for LocalFileStorageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalFileStorageBuilder {
    /// Constructs a new builder with no root configured and the
    /// default key prefix (empty string) and the default signing
    /// secret. The caller MUST set `root` via
    /// [`Self::root`] before calling [`Self::build`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            root: None,
            key_prefix: String::new(),
            signing_secret: DEFAULT_SIGNING_SECRET.to_vec(),
        }
    }

    /// Sets the local filesystem root (e.g. `/var/lib/educore/files`).
    /// The root MUST exist and MUST be a directory at build time.
    #[must_use]
    pub fn root(mut self, root: impl Into<PathBuf>) -> Self {
        self.root = Some(root.into());
        self
    }

    /// Sets the key prefix (e.g. `"educore/"` or
    /// `"tenant-a/"`). Every object is stored under
    /// `root / key_prefix / <key>`. Defaults to the empty string.
    #[must_use]
    pub fn key_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.key_prefix = prefix.into();
        self
    }

    /// Sets a custom signing secret for HMAC-SHA256 URL tokens.
    /// Defaults to a compile-time constant suitable only for
    /// development. Production callers MUST supply a high-entropy
    /// secret sourced from a secret manager.
    #[must_use]
    pub fn signing_secret(mut self, secret: impl Into<Vec<u8>>) -> Self {
        self.signing_secret = secret.into();
        self
    }

    /// Builds the [`LocalFileStorage`], verifying that `root`
    /// exists and is a directory. Returns
    /// [`FileStorageError::Infrastructure`] if the check fails.
    pub fn build(self) -> Result<LocalFileStorage, FileStorageError> {
        let root = self.root.ok_or_else(|| {
            FileStorageError::InvalidKey(String::from("LocalFileStorage root is not configured"))
        })?;

        let metadata = std::fs::metadata(&root).map_err(|e| {
            FileStorageError::Infrastructure(InfrastructureError::with_source(
                format!("root does not exist: {}", root.display()),
                Box::new(e),
            ))
        })?;

        if !metadata.is_dir() {
            return Err(FileStorageError::Infrastructure(InfrastructureError::new(
                format!("root is not a directory: {}", root.display()),
            )));
        }

        Ok(LocalFileStorage {
            root,
            key_prefix: self.key_prefix,
            signing_secret: Arc::new(self.signing_secret),
        })
    }
}

impl fmt::Debug for LocalFileStorageBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalFileStorageBuilder")
            .field("root", &self.root)
            .field("key_prefix", &self.key_prefix)
            .field("signing_secret", &"<redacted>")
            .finish()
    }
}

// ---------------------------------------------------------------------------
// FileStorage impl
// ---------------------------------------------------------------------------

#[async_trait]
impl FileStorage for LocalFileStorage {
    async fn put(&self, request: PutRequest) -> PortResult<FileReference> {
        let path = self.resolve(request.key.as_str())?;

        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                FileStorageError::Infrastructure(InfrastructureError::with_source(
                    format!("failed to create parent directory: {}", parent.display()),
                    Box::new(e),
                ))
            })?;
        }

        let size_bytes = request.content.len();
        let size = u64_from_usize(size_bytes);

        tokio::fs::write(&path, &request.content)
            .await
            .map_err(|e| {
                FileStorageError::Infrastructure(InfrastructureError::with_source(
                    format!("failed to write file: {}", path.display()),
                    Box::new(e),
                ))
            })?;

        let digest = sha256(&request.content);
        let hex = hex_encode(&digest);

        Ok(FileReference {
            key: request.key,
            etag: hex.clone(),
            size,
            content_type: request.content_type,
            visibility: request.visibility,
            uploaded_at: Timestamp::now(),
            uploaded_by: request.tenant.actor_id,
            tenant: request.tenant,
            storage_class: StorageClass::Hot,
            checksum: Checksum::new(hex),
        })
    }

    async fn get(&self, reference: &FileReference) -> PortResult<FileStream> {
        let path = self.resolve(reference.key.as_str())?;
        let key_for_error = reference.key.clone();

        let file = tokio::fs::File::open(&path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                FileStorageError::NotFound(key_for_error)
            } else {
                FileStorageError::Infrastructure(InfrastructureError::with_source(
                    format!("failed to open file: {}", path.display()),
                    Box::new(e),
                ))
            }
        })?;

        let (tx, rx) = mpsc::channel::<StdResult<Vec<u8>, std::io::Error>>(GET_CHANNEL_CAPACITY);

        tokio::spawn(async move {
            let mut reader = file;
            let mut buf = vec![0u8; READ_CHUNK_SIZE];
            loop {
                match reader.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {
                        if tx.send(Ok(buf[..n].to_vec())).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e)).await;
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }

    async fn delete(&self, reference: &FileReference) -> PortResult<()> {
        let path = self.resolve(reference.key.as_str())?;
        match tokio::fs::remove_file(&path).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(FileStorageError::Infrastructure(
                InfrastructureError::with_source(
                    format!("failed to delete file: {}", path.display()),
                    Box::new(e),
                ),
            )),
        }
    }

    async fn exists(&self, reference: &FileReference) -> PortResult<bool> {
        let path = self.resolve(reference.key.as_str())?;
        match tokio::fs::metadata(&path).await {
            Ok(m) => Ok(m.is_file()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(FileStorageError::Infrastructure(
                InfrastructureError::with_source(
                    format!("failed to stat file: {}", path.display()),
                    Box::new(e),
                ),
            )),
        }
    }

    async fn head(&self, reference: &FileReference) -> PortResult<FileMetadata> {
        let path = self.resolve(reference.key.as_str())?;
        let key_for_error = reference.key.clone();
        let m = tokio::fs::metadata(&path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                FileStorageError::NotFound(key_for_error)
            } else {
                FileStorageError::Infrastructure(InfrastructureError::with_source(
                    format!("failed to stat file: {}", path.display()),
                    Box::new(e),
                ))
            }
        })?;
        Ok(FileMetadata {
            key: reference.key.clone(),
            etag: reference.etag.clone(),
            size: m.len(),
            content_type: reference.content_type.clone(),
            uploaded_at: reference.uploaded_at,
        })
    }

    async fn signed_url(
        &self,
        reference: &FileReference,
        options: SignedUrlOptions,
    ) -> PortResult<String> {
        let path = self.resolve(reference.key.as_str())?;

        // Public reads don't need a token.
        if reference.visibility == Visibility::Public && options.method == SignedUrlMethod::Get {
            return Ok(Self::file_url(&path));
        }

        let expires_in = options.expires_in.as_secs();
        let method = options.method.as_str();
        let token = self.sign(reference.key.as_str(), expires_in);

        let mut url = format!(
            "file://{}?expires_in={expires_in}&method={method}&token={token}",
            path.display(),
        );
        if let Some(cd) = options.response_content_disposition.as_deref() {
            url.push_str("&response_content_disposition=");
            url.push_str(&url_encode(cd));
        }
        if let Some(ct) = options.response_content_type.as_ref() {
            url.push_str("&response_content_type=");
            url.push_str(&url_encode(ct.as_str()));
        }
        Ok(url)
    }

    async fn copy(&self, src: &FileReference, dst_key: &str) -> PortResult<FileReference> {
        let src_path = self.resolve(src.key.as_str())?;
        let dst_path = self.resolve(dst_key)?;

        if let Some(parent) = dst_path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                FileStorageError::Infrastructure(InfrastructureError::with_source(
                    format!("failed to create parent directory: {}", parent.display()),
                    Box::new(e),
                ))
            })?;
        }

        tokio::fs::copy(&src_path, &dst_path).await.map_err(|e| {
            FileStorageError::Infrastructure(InfrastructureError::with_source(
                format!(
                    "failed to copy {} to {}",
                    src_path.display(),
                    dst_path.display()
                ),
                Box::new(e),
            ))
        })?;

        Ok(FileReference {
            key: FileKey::new(dst_key),
            etag: src.etag.clone(),
            size: src.size,
            content_type: src.content_type.clone(),
            visibility: src.visibility,
            uploaded_at: Timestamp::now(),
            uploaded_by: src.tenant.actor_id,
            tenant: src.tenant.clone(),
            storage_class: src.storage_class,
            checksum: src.checksum.clone(),
        })
    }

    async fn move_to(&self, src: &FileReference, dst_key: &str) -> PortResult<FileReference> {
        let src_path = self.resolve(src.key.as_str())?;
        let dst_path = self.resolve(dst_key)?;

        if let Some(parent) = dst_path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                FileStorageError::Infrastructure(InfrastructureError::with_source(
                    format!("failed to create parent directory: {}", parent.display()),
                    Box::new(e),
                ))
            })?;
        }

        tokio::fs::rename(&src_path, &dst_path).await.map_err(|e| {
            FileStorageError::Infrastructure(InfrastructureError::with_source(
                format!(
                    "failed to rename {} to {}",
                    src_path.display(),
                    dst_path.display()
                ),
                Box::new(e),
            ))
        })?;

        Ok(FileReference {
            key: FileKey::new(dst_key),
            etag: src.etag.clone(),
            size: src.size,
            content_type: src.content_type.clone(),
            visibility: src.visibility,
            uploaded_at: Timestamp::now(),
            uploaded_by: src.tenant.actor_id,
            tenant: src.tenant.clone(),
            storage_class: src.storage_class,
            checksum: src.checksum.clone(),
        })
    }
}

// ---------------------------------------------------------------------------
// Free helpers
// ---------------------------------------------------------------------------

/// Lossless `usize → u64` conversion. The error case is
/// unreachable on every platform the engine supports (32-bit
/// `usize` fits in `u64`; 64-bit `usize` is `u64`). The
/// `#[allow]` makes the intent explicit without paying for
/// `From`/`TryFrom` verbosity at every call site.
#[inline]
#[allow(
    clippy::expect_used,
    clippy::panic,
    clippy::cast_lossless,
    clippy::cast_possible_truncation
)]
fn u64_from_usize(n: usize) -> u64 {
    u64::try_from(n).unwrap_or(0)
}

/// Lexical path normalisation without touching the filesystem
/// (a pure string operation). Collapses `.` and `..` components
/// in the input `Path` and returns a new `PathBuf`. This is the
/// fallback when the file may not exist (so `canonicalize`
/// would fail).
fn normalise_lexical(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => {
                out.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    out.push("..");
                }
            }
            Component::Normal(c) => out.push(c),
        }
    }
    out
}

/// Minimal percent-encoding for URL query string values. Encodes
/// every byte outside the unreserved set (RFC 3986 §2.3) as
/// `%XX` upper-case hex. The output is safe to embed in a
/// `file://` URL.
fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &b in s.as_bytes() {
        let unreserved = b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~');
        if unreserved {
            out.push(b as char);
        } else {
            out.push('%');
            out.push(hex_digit(b >> 4));
            out.push(hex_digit(b & 0x0f));
        }
    }
    out
}

/// Returns the lowercase hex digit for a 4-bit value.
#[inline]
fn hex_digit(n: u8) -> char {
    match n & 0x0f {
        0..=9 => (b'0' + (n & 0x0f)) as char,
        10..=15 => (b'a' + ((n & 0x0f) - 10)) as char,
        _ => '0',
    }
}

/// Lowercase hex encoding of a byte slice.
fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(hex_digit(b >> 4));
        out.push(hex_digit(b & 0x0f));
    }
    out
}

// ---------------------------------------------------------------------------
// SHA-256 (FIPS 180-4) and HMAC-SHA256 (RFC 2104) — stdlib only.
// The crate's `Cargo.toml` does not declare the `sha2` / `hmac`
// crates; the implementations below are small enough to audit by
// hand and avoid the dep-cost for a reference adapter.
// ---------------------------------------------------------------------------

/// SHA-256 round constants (FIPS 180-4 §4.2.2).
const SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// Initial hash values for SHA-256 (FIPS 180-4 §5.3.3): the
/// first 32 bits of the fractional parts of the square roots of
/// the first 8 primes.
const SHA256_H0: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

/// Computes the SHA-256 digest of `message` (FIPS 180-4).
fn sha256(message: &[u8]) -> [u8; 32] {
    // 1. Padding: append `0x80`, pad with zeros to a multiple of
    //    64 bytes minus 8, then append the big-endian bit length.
    let bit_len = u64_from_usize(message.len()).wrapping_mul(8);
    let mut buf = Vec::with_capacity(message.len() + 1 + 8);
    buf.extend_from_slice(message);
    buf.push(0x80);
    while buf.len() % 64 != 56 {
        buf.push(0x00);
    }
    buf.extend_from_slice(&bit_len.to_be_bytes());

    // 2. Initial hash value.
    let mut h = SHA256_H0;

    // 3. Process each 512-bit (64-byte) chunk.
    for chunk in buf.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, word) in chunk.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut hh = h[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(SHA256_K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut out = [0u8; 32];
    for (i, &v) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&v.to_be_bytes());
    }
    out
}

/// Computes the HMAC-SHA256 of `message` with `key` (RFC 2104).
fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    // 1. Prepare the key. If longer than the block, hash it. If
    //    shorter, zero-pad to the block size.
    let mut k = if key.len() > HMAC_BLOCK_SIZE {
        sha256(key).to_vec()
    } else {
        key.to_vec()
    };
    if k.len() < HMAC_BLOCK_SIZE {
        k.resize(HMAC_BLOCK_SIZE, 0);
    }

    // 2. Compute inner = H((k XOR ipad) || message) and
    //    outer = H((k XOR opad) || inner).
    let mut inner_pad = vec![HMAC_IPAD; HMAC_BLOCK_SIZE];
    let mut outer_pad = vec![HMAC_OPAD; HMAC_BLOCK_SIZE];
    for i in 0..HMAC_BLOCK_SIZE {
        inner_pad[i] ^= k[i];
        outer_pad[i] ^= k[i];
    }

    let mut inner_msg = Vec::with_capacity(HMAC_BLOCK_SIZE + message.len());
    inner_msg.extend_from_slice(&inner_pad);
    inner_msg.extend_from_slice(message);
    let inner = sha256(&inner_msg);

    let mut outer_msg = Vec::with_capacity(HMAC_BLOCK_SIZE + inner.len());
    outer_msg.extend_from_slice(&outer_pad);
    outer_msg.extend_from_slice(&inner);
    sha256(&outer_msg)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use std::path::PathBuf;
    use std::time::Duration;

    use educore_core::clock::{IdGenerator, SystemIdGen};
    use educore_core::tenant::TenantContext;

    use crate::port::ContentType;

    use super::*;

    /// Test fixture: a temp directory that is cleaned up on
    /// drop. We avoid the `tempfile` crate (not a workspace dep)
    /// by allocating a uniquely-named sub-directory of the
    /// process temp dir and removing it on drop.
    struct TempRoot(PathBuf);

    impl TempRoot {
        fn new(label: &str) -> Self {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            let pid = std::process::id();
            let dir = std::env::temp_dir().join(format!("educore-files-{label}-{pid}-{nanos}"));
            std::fs::create_dir_all(&dir).expect("create temp root");
            Self(dir)
        }

        fn path(&self) -> PathBuf {
            self.0.clone()
        }
    }

    impl Drop for TempRoot {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// Test fixture: a `TenantContext` for the unit tests.
    fn test_tenant() -> TenantContext {
        let id_gen = SystemIdGen;
        TenantContext::for_user(
            id_gen.next_school_id(),
            id_gen.next_user_id(),
            id_gen.next_correlation_id(),
            educore_core::tenant::UserType::Staff,
        )
    }

    /// Test fixture: a `FileReference` produced by a prior `put`.
    fn test_reference(key: &str, content: &[u8], tenant: TenantContext) -> FileReference {
        let digest = sha256(content);
        let hex = hex_encode(&digest);
        FileReference {
            key: FileKey::new(key),
            etag: hex.clone(),
            size: u64_from_usize(content.len()),
            content_type: ContentType::new("application/octet-stream"),
            visibility: Visibility::Private,
            uploaded_at: Timestamp::now(),
            uploaded_by: tenant.actor_id,
            tenant,
            storage_class: StorageClass::Hot,
            checksum: Checksum::new(hex),
        }
    }

    /// Test fixture: a `PutRequest` with sensible defaults.
    fn test_put(key: &str, content: Vec<u8>, tenant: TenantContext) -> PutRequest {
        PutRequest {
            tenant,
            key: FileKey::new(key),
            content,
            content_type: ContentType::new("application/octet-stream"),
            metadata: std::collections::BTreeMap::new(),
            visibility: Visibility::Private,
            overwrite: true,
            idempotency_key: None,
        }
    }

    #[test]
    fn local_storage_builder_creates_root_directory() {
        // The builder requires a pre-existing root directory.
        // Create a temp dir, point the builder at it, and verify
        // the build succeeds and the resulting storage's root
        // matches the temp dir.
        let temp = TempRoot::new("builder-creates");
        let storage = LocalFileStorageBuilder::new()
            .root(temp.path())
            .key_prefix("test/")
            .build()
            .expect("build should succeed with an existing directory");
        assert_eq!(storage.root(), temp.path());
        assert_eq!(storage.key_prefix(), "test/");

        // Building without a root is a config error.
        let no_root = LocalFileStorageBuilder::new().build();
        assert!(matches!(no_root, Err(FileStorageError::InvalidKey(_))));

        // Building with a non-existent path is an infrastructure
        // error.
        let missing = LocalFileStorageBuilder::new()
            .root("/nonexistent/educore-files-missing-root-xyz-12345")
            .build();
        assert!(matches!(missing, Err(FileStorageError::Infrastructure(_))));
    }

    #[tokio::test]
    async fn local_storage_signed_url_is_deterministic() {
        let temp = TempRoot::new("signed-url");
        let storage = LocalFileStorageBuilder::new()
            .root(temp.path())
            .signing_secret(b"test-secret-for-signed-url")
            .build()
            .expect("build should succeed");

        let tenant = test_tenant();
        let reference = test_reference("photos/ada.jpg", b"hello", tenant);

        let options_60s = SignedUrlOptions::new(Duration::from_secs(60), SignedUrlMethod::Get);
        let options_120s = SignedUrlOptions::new(Duration::from_secs(120), SignedUrlMethod::Get);

        // Same path + same expiry → same URL.
        let url1 = storage
            .signed_url(&reference, options_60s.clone())
            .await
            .expect("signed_url 1");
        let url2 = storage
            .signed_url(&reference, options_60s)
            .await
            .expect("signed_url 2");
        assert_eq!(url1, url2, "same (key, expires_in) must yield the same URL");

        // Different expiry → different URL.
        let url3 = storage
            .signed_url(&reference, options_120s)
            .await
            .expect("signed_url 3");
        assert_ne!(
            url1, url3,
            "different expires_in must yield a different URL"
        );

        // The URL embeds the key, the expiry, the method, and the
        // HMAC token.
        assert!(url1.contains("expires_in=60"));
        assert!(url1.contains("method=GET"));
        assert!(url1.contains("token="));
        assert!(url1.starts_with("file://"));
    }

    #[test]
    fn sha256_matches_fips_180_4_test_vector() {
        // FIPS 180-4 Appendix B.1: SHA-256("abc") =
        // ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
        let digest = sha256(b"abc");
        assert_eq!(
            hex_encode(&digest),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn hmac_sha256_matches_rfc_4231_test_case_1() {
        // RFC 4231 §4.2: key = 0x0b * 20, data = "Hi There",
        // expected =
        // b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7
        let key = vec![0x0b; 20];
        let mac = hmac_sha256(&key, b"Hi There");
        assert_eq!(
            hex_encode(&mac),
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
    }

    #[test]
    fn resolve_rejects_path_traversal() {
        let temp = TempRoot::new("path-traversal");
        let storage = LocalFileStorageBuilder::new()
            .root(temp.path())
            .build()
            .expect("build should succeed");

        // A `..` component is rejected.
        let err = storage.resolve("../escape.txt");
        assert!(matches!(err, Err(FileStorageError::InvalidKey(_))));

        // An absolute key is rejected.
        let err = storage.resolve("/etc/passwd");
        assert!(matches!(err, Err(FileStorageError::InvalidKey(_))));

        // A normal key resolves successfully.
        let path = storage.resolve("subdir/file.txt").expect("normal key");
        assert!(path.starts_with(storage.root()));
    }

    #[tokio::test]
    async fn put_get_delete_round_trip_streams_content() {
        let temp = TempRoot::new("round-trip");
        let storage = LocalFileStorageBuilder::new()
            .root(temp.path())
            .key_prefix("rt/")
            .build()
            .expect("build should succeed");

        let tenant = test_tenant();
        let payload = b"the quick brown fox jumps over the lazy dog".to_vec();
        let request = test_put("hello.txt", payload.clone(), tenant.clone());

        let reference = storage.put(request).await.expect("put should succeed");
        assert_eq!(reference.size, u64_from_usize(payload.len()));
        assert_eq!(reference.checksum.as_str(), hex_encode(&sha256(&payload)));

        // `head` returns the metadata.
        let metadata = storage.head(&reference).await.expect("head should succeed");
        assert_eq!(metadata.size, reference.size);
        assert_eq!(metadata.key, reference.key);

        // `exists` returns true.
        let exists = storage
            .exists(&reference)
            .await
            .expect("exists should succeed");
        assert!(exists);

        // `get` streams the content. Drain the channel into a
        // single buffer and compare.
        let mut rx = storage.get(&reference).await.expect("get should succeed");
        let mut received = Vec::new();
        while let Some(chunk) = rx.recv().await {
            match chunk {
                Ok(bytes) => received.extend_from_slice(&bytes),
                Err(e) => panic!("stream error: {e}"),
            }
        }
        assert_eq!(received, payload);

        // `delete` is idempotent.
        storage.delete(&reference).await.expect("delete");
        storage.delete(&reference).await.expect("delete idempotent");

        // `exists` is now false.
        let exists_after = storage
            .exists(&reference)
            .await
            .expect("exists should succeed after delete");
        assert!(!exists_after);
    }
}
