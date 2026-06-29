//! # educore-files port
//!
//! The file storage port stores and retrieves files (student photos,
//! homework submissions, certificates, ID-card templates, content
//! images, documents). The engine does not own S3, GCS, or local
//! filesystems — the consumer supplies an adapter that implements
//! [`FileStorage`] and returns [`FileReference`] values for the
//! engine to persist on aggregates.
//!
//! This module is the **port-only** surface. Reference
//! implementations (AWS S3, GCS, Azure Blob, local filesystem,
//! offline-mode cache) land in separate microtasks; see
//! `docs/build-plan.md` § "Phase 15" for the split.
//!
//! See `docs/ports/file-storage.md` for the authoritative
//! specification.
//!
//! # Object safety
//!
//! The [`FileStorage`] trait is object-safe: every method takes
//! `&self`, has no generic parameters, and returns
//! `Result<T, FileStorageError>` directly. The compile-time
//! assertion at the bottom of this module pins the object-safety
//! contract.
//!
//! # Deviations from `docs/ports/file-storage.md`
//!
//! The crate's `Cargo.toml` is intentionally minimal (only
//! `core`, `platform`, `events`, `tokio`, `async-trait`), so the
//! port uses **stdlib-only** value representations:
//!
//! - All opaque ID newtypes (`IdempotencyKey`, …) wrap `String`
//!   rather than `uuid::Uuid`. Adapters that need a parsed UUID
//!   parse the inner string at their boundary.
//! - `Url` is represented as `String` (URL-formatted UTF-8).
//!   Adapters that need a `url::Url` parse the inner string at
//!   their boundary.
//! - `bytes::Bytes` is replaced by `Vec<u8>`. The `bytes` crate is
//!   not a direct dependency of `educore-files`; `Vec<u8>` is
//!   functionally equivalent at the port boundary (zero-copy
//!   sharing is a perf optimisation, not a correctness concern).
//! - `futures::Stream` is replaced by
//!   `tokio::sync::mpsc::Receiver<Result<Vec<u8>, std::io::Error>>`.
//!   The `futures` crate is not a direct dependency of
//!   `educore-files`; the mpsc receiver is a concrete, owned,
//!   `Send`-bound streaming primitive that adapters populate via
//!   a `Sender` and the engine consumes via `.recv().await`. This
//!   preserves the streaming semantic from the spec without taking
//!   on the `futures` dependency.
//! - `Duration` (in [`SignedUrlOptions::expires_in`]) is
//!   `std::time::Duration` (stable, stdlib). No deviation here;
//!   this is the spec's type.
//! - `chrono::NaiveDate` is not used by the spec; no deviation.
//! - The port types do not derive `Serialize` or `Deserialize`.
//!   Adapters that need to cross a wire boundary (e.g. an
//!   offline-mode cache file) implement their own wire format.
//!
//! These deviations match the `educore-payment` and `educore-auth`
//! port patterns and are documented here so future ports that gain
//! richer dependencies can adopt the spec's idiomatic types
//! without changing the trait surface.

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

use std::collections::BTreeMap;
use std::fmt;
use std::result::Result as StdResult;
use std::time::Duration;

use async_trait::async_trait;

use educore_core::ids::{SchoolId, UserId};
use educore_core::tenant::TenantContext;
use educore_core::value_objects::Timestamp;

use crate::errors::FileStorageError;

/// Convenience alias for the port's [`StdResult`] type. Adapters
/// return `Result<T, FileStorageError>`.
pub type Result<T> = StdResult<T, FileStorageError>;

// ---------------------------------------------------------------------------
// Newtype identifiers and opaque value objects (String-backed)
// ---------------------------------------------------------------------------

/// A caller-provided idempotency token for upload retries.
///
/// Per `docs/ports/file-storage.md` § "Idempotency": a retry of
/// the same upload with the same `IdempotencyKey` returns the
/// same [`FileReference`] without re-uploading.
///
/// The workspace's `educore-core::ids::IdempotencyKey` wraps a
/// `uuid::Uuid`; the file port uses this local
/// `String`-backed newtype so it does not need to take a direct
/// dependency on the `uuid` crate. Adapters that need the parsed
/// UUID parse the inner string at their boundary (or hash it
/// directly to deduplicate — the spec does not require UUID
/// semantics for the file port).
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct IdempotencyKey(pub String);

impl IdempotencyKey {
    /// Wraps an opaque idempotency-key string.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the inner string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for IdempotencyKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for IdempotencyKey {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for IdempotencyKey {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// A typed, tenant-scoped file key (logical path).
///
/// The engine does not accept raw path strings on the public
/// port surface — every key is wrapped in a [`FileKey`] so the
/// type system catches the "I passed a URL instead of a key"
/// mistake at compile time. The full physical key is
/// `<school_id>/<domain>/<aggregate>/<id>/<logical>`; the
/// adapter is responsible for prefixing the tenant portion.
///
/// Per `docs/ports/file-storage.md` § "Key Namespacing": the
/// adapter namespaces keys by tenant so a consumer that stores
/// multiple schools in one bucket cannot accidentally cross
/// tenant boundaries.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct FileKey(String);

impl FileKey {
    /// Constructs a `FileKey` from a raw logical-path string.
    /// The constructor is infallible at the type level; adapters
    /// that need validation (length, character set, reserved
    /// prefix) perform it inside the [`FileStorage::put`]
    /// implementation and return
    /// [`FileStorageError::InvalidKey`](crate::errors::FileStorageError::InvalidKey)
    /// on a malformed input.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the inner string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FileKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for FileKey {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for FileKey {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl AsRef<str> for FileKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// A MIME content type (e.g. `image/jpeg`, `application/pdf`).
///
/// The engine does not validate the tag against the IANA media
/// registry — adapters that need validation (e.g. "only accept
/// `image/*`") do it inside [`FileStorage::put`] and return
/// [`FileStorageError::UnsupportedContentType`](crate::errors::FileStorageError::UnsupportedContentType)
/// on a rejected MIME type.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct ContentType(pub String);

impl ContentType {
    /// Constructs a `ContentType` from a raw MIME string. The
    /// constructor is infallible at the type level; see
    /// [`ContentType`] for the validation contract.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the inner string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for ContentType {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for ContentType {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// A content-addressable hash recorded at upload time.
///
/// Per `docs/ports/file-storage.md` § "Content-Addressable
/// Hashing": the adapter computes a SHA-256 checksum on upload;
/// the engine verifies the checksum on read. The wire format is
/// the lowercase hex representation of the digest (the spec
/// does not pin the algorithm, but SHA-256 hex is the canonical
/// shape).
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Checksum(pub String);

impl Checksum {
    /// Constructs a `Checksum` from a raw hex string.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the inner string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Checksum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for Checksum {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for Checksum {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

// ---------------------------------------------------------------------------
// Visibility and storage class
// ---------------------------------------------------------------------------

/// The access scope of a file. Locked to
/// `docs/ports/file-storage.md` § "PutRequest".
///
/// Per `docs/ports/file-storage.md` § "Key Namespacing": the
/// adapter is responsible for enforcing visibility — a file
/// uploaded as [`Visibility::Private`] must require a signed
/// URL on every read; a file uploaded as [`Visibility::Public`]
/// is fetchable without authentication (subject to the CDN's
/// public-bucket policy).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Visibility {
    /// Accessible only to the uploader (and consumers that hold a
    /// signed URL).
    #[default]
    Private,

    /// Accessible without authentication (subject to the CDN's
    /// public-bucket policy). The engine never stores
    /// public-access URLs on aggregates — clients resolve them
    /// through a public-site port adapter.
    Public,

    /// Accessible to any authenticated user inside the tenant.
    /// The adapter enforces the tenant boundary via key prefix
    /// (`<school_id>/...`); the engine does not check it again.
    TenantPrivate,
}

impl Visibility {
    /// Returns the canonical snake_case wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Private => "private",
            Self::Public => "public",
            Self::TenantPrivate => "tenant_private",
        }
    }
}

impl fmt::Display for Visibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

/// The storage class of a file. Locked to
/// `docs/ports/file-storage.md` § "StorageClass".
///
/// Per `docs/ports/file-storage.md` § "Lifecycle Rules": the
/// adapter supports lifecycle rules configured per bucket
/// (transition Hot → Cool after N days, Cool → Archive after M
/// days, expire after P days). The adapter picks the class at
/// upload time; transitions are governed by consumer-side
/// configuration.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum StorageClass {
    /// Frequent access. Default for newly uploaded files.
    #[default]
    Hot,

    /// Infrequent access. Lower cost per GB, slightly higher
    /// retrieval cost.
    Cool,

    /// Long-term, slower retrieval. Lowest cost per GB, minutes-
    /// to-hours retrieval latency on cold restores.
    Archive,
}

impl StorageClass {
    /// Returns the canonical snake_case wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Hot => "hot",
            Self::Cool => "cool",
            Self::Archive => "archive",
        }
    }
}

impl fmt::Display for StorageClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

// ---------------------------------------------------------------------------
// Signed URL options
// ---------------------------------------------------------------------------

/// The HTTP method that a signed URL authorizes. Locked to
/// `docs/ports/file-storage.md` § "SignedUrlMethod".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignedUrlMethod {
    /// Authorizes a `GET` (read) request.
    Get,
    /// Authorizes a `PUT` (write) request.
    Put,
}

impl SignedUrlMethod {
    /// Returns the canonical uppercase HTTP verb.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Put => "PUT",
        }
    }
}

impl fmt::Display for SignedUrlMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

/// Options for [`FileStorage::signed_url`]. Locked to
/// `docs/ports/file-storage.md` § "SignedUrlOptions".
///
/// `signed_url` produces a time-limited URL for a private file.
/// The adapter uses the storage provider's signing mechanism
/// (e.g. S3 presigned URLs, GCS signed URLs, local token URLs).
/// The returned URL MUST expire after `expires_in`; the engine
/// surfaces this via [`FileStorageError::PermissionDenied`] when
/// the consumer tries to use an expired URL.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SignedUrlOptions {
    /// How long the signed URL is valid. The adapter rounds
    /// down to the provider's minimum granularity (e.g. S3
    /// presigned URLs are valid down to one second).
    pub expires_in: Duration,

    /// The HTTP method the signed URL authorizes.
    pub method: SignedUrlMethod,

    /// Optional override for the `Content-Disposition` header on
    /// the response (e.g. `"attachment; filename=\"homework.pdf\""`
    /// to force a download prompt). `None` lets the underlying
    /// object set its own disposition.
    pub response_content_disposition: Option<String>,

    /// Optional override for the `Content-Type` header on the
    /// response. `None` uses the object's stored content type.
    pub response_content_type: Option<ContentType>,
}

impl SignedUrlOptions {
    /// Constructs a `SignedUrlOptions` with `expires_in` and
    /// `method`; the two optional overrides default to `None`.
    #[must_use]
    pub fn new(expires_in: Duration, method: SignedUrlMethod) -> Self {
        Self {
            expires_in,
            method,
            response_content_disposition: None,
            response_content_type: None,
        }
    }

    /// Sets the `Content-Disposition` override.
    #[must_use]
    pub fn with_response_content_disposition(mut self, value: impl Into<String>) -> Self {
        self.response_content_disposition = Some(value.into());
        self
    }

    /// Sets the `Content-Type` override.
    #[must_use]
    pub fn with_response_content_type(mut self, value: ContentType) -> Self {
        self.response_content_type = Some(value);
        self
    }
}

// ---------------------------------------------------------------------------
// PutRequest
// ---------------------------------------------------------------------------

/// The input to [`FileStorage::put`]. Locked to
/// `docs/ports/file-storage.md` § "PutRequest".
///
/// `content` is `Vec<u8>` in this crate's deviation (the spec
/// uses `bytes::Bytes`; see the module-level doc).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PutRequest {
    /// The tenant context (school, actor, correlation). The
    /// adapter uses `tenant.school_id` to namespace the key.
    pub tenant: TenantContext,

    /// The logical key, scoped to the tenant. The adapter is
    /// responsible for prefixing the school id.
    pub key: FileKey,

    /// The file content. The engine does not impose an upper
    /// bound; the adapter enforces its own limit and returns
    /// [`FileStorageError::TooLarge`](crate::errors::FileStorageError::TooLarge)
    /// on oversize content.
    pub content: Vec<u8>,

    /// The MIME type. The adapter may reject unknown or
    /// disallowed types with
    /// [`FileStorageError::UnsupportedContentType`](crate::errors::FileStorageError::UnsupportedContentType).
    pub content_type: ContentType,

    /// Adapter-specific metadata (e.g. `uploaded_via`,
    /// `original_filename`, `submission_id`). Adapters may use
    /// any well-known keys; the engine does not inspect this map.
    pub metadata: BTreeMap<String, String>,

    /// The access scope (see [`Visibility`]).
    pub visibility: Visibility,

    /// `true` to overwrite an existing object at the same key;
    /// `false` to return an error if the key is already in use.
    /// The spec leaves the precise error to the adapter; the
    /// engine surfaces a generic
    /// [`FileStorageError::Infrastructure`](crate::errors::FileStorageError::Infrastructure)
    /// for "key exists, overwrite not allowed".
    pub overwrite: bool,

    /// Optional idempotency key. A retry of the same upload
    /// returns the same [`FileReference`] without re-uploading.
    pub idempotency_key: Option<IdempotencyKey>,
}

impl PutRequest {
    /// Returns the active school's id (denormalised off the
    /// tenant for adapter convenience).
    #[must_use]
    pub fn active_school_id(&self) -> SchoolId {
        self.tenant.school_id
    }

    /// Returns the uploading user's id (from the tenant context).
    #[must_use]
    pub fn actor_id(&self) -> UserId {
        self.tenant.actor_id
    }
}

// ---------------------------------------------------------------------------
// FileReference
// ---------------------------------------------------------------------------

/// The output of [`FileStorage::put`] and the durable handle to
/// a stored file. Domain aggregates store [`FileReference`]
/// values; they never store URLs.
///
/// Per `docs/ports/file-storage.md` § "FileReference": the
/// reference carries everything the engine needs to re-derive a
/// signed URL, to copy / move the object, and to verify
/// integrity on read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileReference {
    /// The logical key, scoped to the tenant.
    pub key: FileKey,

    /// The entity tag (content hash) returned by the provider.
    /// The engine compares this against the recomputed hash on
    /// read; mismatch fails the read with
    /// [`FileStorageError::ChecksumMismatch`](crate::errors::FileStorageError::ChecksumMismatch).
    pub etag: String,

    /// The object's size in bytes.
    pub size: u64,

    /// The stored content type.
    pub content_type: ContentType,

    /// The access scope at upload time.
    pub visibility: Visibility,

    /// When the upload was accepted by the adapter.
    pub uploaded_at: Timestamp,

    /// The user who uploaded the file. Denormalised off the
    /// `TenantContext` for downstream audit log convenience.
    pub uploaded_by: UserId,

    /// The tenant context at upload time. Carried so the
    /// adapter can re-validate cross-tenant access without the
    /// engine rebuilding it.
    pub tenant: TenantContext,

    /// The storage class the adapter assigned.
    pub storage_class: StorageClass,

    /// The content-addressable checksum the adapter computed.
    /// The spec pins SHA-256 hex; the field is opaque at the
    /// port boundary so the engine does not parse it.
    pub checksum: Checksum,

    /// The provider-assigned object version id, if the underlying
    /// bucket has versioning enabled (S3 returns a non-`None`
    /// `VersionId` on every write when the bucket is versioned;
    /// GCS and Azure expose equivalent "generation" or
    /// "version id" tokens). `None` means "current version" —
    /// the engine treats the reference as a live handle and
    /// adapters pass no `versionId` / `generation` query
    /// parameter to the provider on `get` / `delete` /
    /// `head`.
    ///
    /// Per `docs/ports/file-storage.md` § "Versioning": when
    /// versioning is enabled, the adapter surfaces the
    /// `VersionId` returned by `PutObject` on the reference and
    /// re-uses it on subsequent `GetObject` / `DeleteObject` /
    /// `HeadObject` calls when the field is set. Callers pin to
    /// a specific past version via
    /// [`FileStorage::pin_version`](FileStorage::pin_version) (the
    /// adapter-specific helper documented alongside each
    /// reference impl).
    ///
    /// **Forward compatibility note:** when/if `Serialize` /
    /// `Deserialize` derives are added to this struct (the port
    /// currently does not derive them per the module-level doc),
    /// this field MUST carry
    /// `#[serde(default, skip_serializing_if = "Option::is_none")]`
    /// so old persisted rows deserialize cleanly and so the wire
    /// format stays compact for non-versioned buckets.
    pub version_id: Option<String>,
}

// ---------------------------------------------------------------------------
// FileMetadata (returned by `head`)
// ---------------------------------------------------------------------------

/// A lightweight summary of a stored object, returned by
/// [`FileStorage::head`]. Carries the metadata fields the engine
/// needs to decide whether to issue a signed URL, surface a
/// duplicate, or stream the body. The body itself is NOT
/// included — call [`FileStorage::get`] to fetch the content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileMetadata {
    /// The logical key.
    pub key: FileKey,

    /// The entity tag (content hash).
    pub etag: String,

    /// The object's size in bytes.
    pub size: u64,

    /// The stored content type.
    pub content_type: ContentType,

    /// When the object was uploaded (or, for versioned objects,
    /// when the current version was created).
    pub uploaded_at: Timestamp,
}

// ---------------------------------------------------------------------------
// FileStream (returned by `get`)
// ---------------------------------------------------------------------------

/// A streaming download. The engine pulls byte chunks via
/// `.recv().await` until the channel is closed (the adapter
/// drops the sender when the read is complete or fails).
///
/// Per `docs/ports/file-storage.md` § "Streaming Downloads": the
/// engine does not load entire files into memory. Adapters
/// MUST yield chunks promptly and MUST NOT buffer the entire
/// object before sending the first chunk.
///
/// Deviation from the spec: the spec types this as
/// `Pin<Box<dyn futures::Stream<Item = Result<Bytes>> + Send>>`.
/// The `futures` and `bytes` crates are not direct dependencies
/// of `educore-files`; this alias uses
/// `tokio::sync::mpsc::Receiver<Result<Vec<u8>, std::io::Error>>`
/// instead. Adapters construct a channel via
/// `tokio::sync::mpsc::channel(capacity)`, spawn a task that
/// pushes chunks via the [`Sender`](tokio::sync::mpsc::Sender),
/// and return the [`Receiver`](tokio::sync::mpsc::Receiver).
/// The engine pulls chunks via `.recv().await` until the channel
/// returns `None` (sender dropped) or an error.
pub type FileStream = tokio::sync::mpsc::Receiver<StdResult<Vec<u8>, std::io::Error>>;

// ---------------------------------------------------------------------------
// FileStorage trait (8 methods, object-safe)
// ---------------------------------------------------------------------------

/// The file storage port — the engine's sole entry point for
/// storing and retrieving binary objects. Adapters are
/// `Send + Sync` so the engine can dispatch against them from any
/// async runtime.
///
/// # Object safety
///
/// The trait is object-safe: every method takes `&self`, has no
/// generic parameters, and returns `Result<T, FileStorageError>`
/// directly. The compile-time assertion at the bottom of this
/// module pins the object-safety contract.
///
/// # Idempotency
///
/// `put` is idempotent on `PutRequest::idempotency_key`. A
/// retry with the same key returns the original [`FileReference`]
/// without re-uploading. Adapters that cannot guarantee
/// idempotency MUST implement it themselves (the engine does not
/// retry).
///
/// # Tenant isolation
///
/// The adapter is responsible for prefixing the key with the
/// active `school_id`. A consumer that stores multiple schools
/// in one bucket cannot accidentally cross-tenant access because
/// keys are prefixed.
///
/// # Streaming
///
/// `get` returns a [`FileStream`] that yields the file in
/// chunks. The engine does not load entire files into memory.
/// Adapters MUST yield chunks promptly and MUST NOT buffer the
/// entire object before sending the first chunk.
#[async_trait]
pub trait FileStorage: Send + Sync + std::fmt::Debug {
    /// Uploads a new object. Returns a [`FileReference`] that
    /// the engine persists on the aggregate. A retry with the
    /// same `PutRequest::idempotency_key` returns the original
    /// [`FileReference`] without re-uploading.
    async fn put(&self, request: PutRequest) -> Result<FileReference>;

    /// Streams the object's content. The returned [`FileStream`]
    /// yields chunks until the adapter closes the channel; the
    /// engine pulls them via `.recv().await`. The adapter MUST
    /// verify the content hash against `reference.checksum` and
    /// MUST surface a
    /// [`FileStorageError::ChecksumMismatch`](crate::errors::FileStorageError::ChecksumMismatch)
    /// on a mismatch.
    async fn get(&self, reference: &FileReference) -> Result<FileStream>;

    /// Deletes the object. Returns `Ok(())` whether or not the
    /// object existed (delete is idempotent at the engine
    /// boundary).
    async fn delete(&self, reference: &FileReference) -> Result<()>;

    /// Returns `true` if the object exists. Adapters MAY issue a
    /// cheap `HEAD` request internally and avoid a full
    /// metadata round-trip.
    async fn exists(&self, reference: &FileReference) -> Result<bool>;

    /// Returns the object's metadata. The body is NOT fetched;
    /// call [`FileStorage::get`] to retrieve the content.
    async fn head(&self, reference: &FileReference) -> Result<FileMetadata>;

    /// Produces a time-limited URL for the object. The returned
    /// URL is a `String` (URL-formatted UTF-8); the spec uses
    /// `url::Url` but `url` is not a direct dependency of this
    /// crate. Adapters MUST honor `options.expires_in` and MUST
    /// reject requests on objects whose visibility does not
    /// permit the requested method.
    async fn signed_url(
        &self,
        reference: &FileReference,
        options: SignedUrlOptions,
    ) -> Result<String>;

    /// Copies the object to a new key inside the same tenant.
    /// The source `FileReference` is left intact; the returned
    /// `FileReference` describes the destination. The
    /// destination key is supplied as a raw logical-path string
    /// and is re-wrapped as a [`FileKey`] inside the adapter.
    async fn copy(&self, src: &FileReference, dst_key: &str) -> Result<FileReference>;

    /// Atomically renames the object to a new key inside the
    /// same tenant. The source `FileReference` MUST be
    /// considered invalid by the engine after a successful
    /// `move_to`; aggregates that reference the source need
    /// to be updated with the returned `FileReference`. The
    /// destination key is supplied as a raw logical-path string
    /// and is re-wrapped as a [`FileKey`] inside the adapter.
    async fn move_to(&self, src: &FileReference, dst_key: &str) -> Result<FileReference>;
}

// ---------------------------------------------------------------------------
// Compile-time object-safety assertion
// ---------------------------------------------------------------------------

/// Compile-time check that the [`FileStorage`] trait is
/// object-safe. If the trait gains a generic method or a
/// `Self`-typed associated type, the assignment will fail to
/// compile, surfacing the regression immediately.
#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod object_safety {
    use super::*;

    /// The trait object must be a usable type.
    #[allow(dead_code)]
    fn _assert_object_safe(_: Box<dyn FileStorage>) {}

    /// And it must compose with `Arc`, which is the shape the
    /// engine uses internally.
    #[allow(dead_code)]
    fn _assert_arc_object_safe(_: std::sync::Arc<dyn FileStorage>) {}

    #[test]
    fn signed_url_options_builder_sets_overrides() {
        let opts = SignedUrlOptions::new(Duration::from_secs(900), SignedUrlMethod::Get)
            .with_response_content_disposition("attachment; filename=\"x.pdf\"")
            .with_response_content_type(ContentType::new("application/pdf"));
        assert_eq!(opts.expires_in, Duration::from_secs(900));
        assert_eq!(opts.method, SignedUrlMethod::Get);
        assert_eq!(
            opts.response_content_disposition.as_deref(),
            Some("attachment; filename=\"x.pdf\"")
        );
        assert_eq!(
            opts.response_content_type.as_ref().map(ContentType::as_str),
            Some("application/pdf")
        );
    }

    #[test]
    fn signed_url_method_uses_http_verbs() {
        assert_eq!(SignedUrlMethod::Get.as_str(), "GET");
        assert_eq!(SignedUrlMethod::Put.as_str(), "PUT");
        assert_eq!(SignedUrlMethod::Get.to_string(), "GET");
        assert_eq!(SignedUrlMethod::Put.to_string(), "PUT");
    }

    #[test]
    fn visibility_uses_snake_case_wire_string() {
        assert_eq!(Visibility::Private.as_str(), "private");
        assert_eq!(Visibility::Public.as_str(), "public");
        assert_eq!(Visibility::TenantPrivate.as_str(), "tenant_private");
    }

    #[test]
    fn storage_class_uses_snake_case_wire_string() {
        assert_eq!(StorageClass::Hot.as_str(), "hot");
        assert_eq!(StorageClass::Cool.as_str(), "cool");
        assert_eq!(StorageClass::Archive.as_str(), "archive");
    }

    #[test]
    fn file_key_round_trips_string() {
        let key = FileKey::new("students/photos/ada.jpg");
        assert_eq!(key.as_str(), "students/photos/ada.jpg");
        assert_eq!(key.to_string(), "students/photos/ada.jpg");
        let from_str: FileKey = "reports/2026-06.pdf".into();
        assert_eq!(from_str.as_str(), "reports/2026-06.pdf");
    }

    #[test]
    fn content_type_round_trips_string() {
        let ct = ContentType::new("image/png");
        assert_eq!(ct.as_str(), "image/png");
        assert_eq!(ct.to_string(), "image/png");
    }

    #[test]
    fn checksum_round_trips_string() {
        let c = Checksum::new("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        assert!(c.as_str().starts_with("e3b0c44"));
    }

    #[test]
    fn idempotency_key_round_trips_string() {
        let k = IdempotencyKey::new("upload-2026-06-19-001");
        assert_eq!(k.as_str(), "upload-2026-06-19-001");
        assert_eq!(k.to_string(), "upload-2026-06-19-001");
    }
}
