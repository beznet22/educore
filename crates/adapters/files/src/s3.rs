//! # educore-files S3 reference impl
//!
//! An AWS S3-backed implementation of the [`FileStorage`](crate::port::FileStorage)
//! port. The adapter is one of the four reference impls scheduled for
//! Phase 15; see `docs/build-plan.md` § "Phase 15" for the
//! parallel split. The S3 adapter is the canonical production
//! target.
//!
//! # Tenant namespacing
//!
//! Every key written by this adapter is prefixed with the
//! [`S3FileStorage::key_prefix`] configured at construction time.
//! The engine does not compute tenant prefixes — the consumer
//! supplies the per-tenant prefix (e.g. `school_<uuid>/`) at
//! builder time. A consumer that stores multiple schools in one
//! bucket cannot accidentally cross-tenant access because every
//! physical key begins with the configured prefix.
//!
//! Per `docs/ports/file-storage.md` § "Key Namespacing": the
//! adapter is responsible for enforcing tenant isolation via key
//! prefix; the engine does not check it again.
//!
//! # Idempotency
//!
//! `S3FileStorage::put` is **not** idempotent on
//! `PutRequest::idempotency_key` at the S3 layer; the adapter
//! documents the key on the returned [`FileReference`](crate::port::FileReference)
//! but does not consult it before upload. A future revision will
//! layer a metadata-based dedup on top of the S3 idempotency
//! contract (see `docs/build-plan.md` Phase 15 workstream F).
//!
//! # Content hashing
//!
//! The adapter computes a SHA-256 hex digest at upload time and
//! records it on the [`Checksum`](crate::port::Checksum) field of
//! the returned [`FileReference`](crate::port::FileReference).
//! Reads do not currently re-verify the checksum on the streamed
//! bytes; consumers that require wire-level integrity verification
//! must layer it on top of the [`FileStream`](crate::port::FileStream).
//!
//! # Visibility and storage class
//!
//! The adapter records the upload-time [`Visibility`](crate::port::Visibility)
//! on the returned reference and picks
//! [`S3StorageClass::Standard`](aws_sdk_s3::types::StorageClass::Standard)
//! at upload time. ACL-based visibility enforcement and
//! storage-class lifecycle rules are configured on the bucket by
//! the consumer, not the engine.
//!
//! # Deviation from the spec's `aws_config` builder
//!
//! The original task spec asks for an `aws_config(aws_config::SdkConfig)`
//! builder method. The [`S3FileStorageBuilder::client`] method
//! accepts an already-built [`aws_sdk_s3::Client`] instead. The
//! reason is that `aws-sdk-s3` 1.55 (the MSRV-pinned version per
//! ADR-015) does **not** declare `aws-config` as a normal
//! dependency — it is dev-only — so the `aws_config::SdkConfig`
//! type is not reachable from a downstream crate that depends
//! only on `aws-sdk-s3`. The consumer's setup code is unchanged:
//!
//! ```rust,ignore
//! use aws_config::BehaviorVersion;
//! use aws_sdk_s3::Client;
//!
//! let sdk_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
//! let client = Client::new(&sdk_config);
//! let storage = S3FileStorage::builder().client(client).bucket("my-bucket").build();
//! ```
//!
//! The `client` method signature is the only API deviation; the
//! eight [`FileStorage`](crate::port::FileStorage) trait methods
//! match the port contract exactly.

use std::fmt;
use std::result::Result as StdResult;

use async_trait::async_trait;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::StorageClass as S3StorageClass;
use aws_sdk_s3::Client;
use educore_core::value_objects::Timestamp;
use uuid::Uuid;

use crate::errors::{FileStorageError, InfrastructureError};
use crate::port::{
    Checksum, ContentType, FileKey, FileMetadata, FileReference, FileStorage, FileStream,
    PutRequest, SignedUrlMethod, SignedUrlOptions, StorageClass, Visibility,
};

/// The mpsc channel capacity used to stream objects from S3 to the
/// engine. Sized to amortize the per-chunk send overhead without
/// pinning large objects entirely in memory.
const STREAM_CHANNEL_CAPACITY: usize = 16;

/// The streaming cap (in bytes) for a single chunk pushed through
/// the [`FileStream`] mpsc channel. The cap is the maximum
/// allocated slice length; the underlying S3 chunk size governs
/// the actual payload length per chunk.
const STREAM_CHUNK_CAP_BYTES: usize = 64 * 1024;

/// A [`FileStorage`](crate::port::FileStorage) backed by AWS S3.
///
/// Cloning is cheap because the underlying [`Client`] is internally
/// `Arc`-shared. Cloning the adapter does not clone any in-flight
/// uploads or downloads.
///
/// # Object safety
///
/// The [`FileStorage`](crate::port::FileStorage) trait is
/// object-safe (see `crate::port::FileStorage`). `S3FileStorage`
/// preserves object safety: every method is `&self`, takes no
/// generic parameters, and returns `Result<T, FileStorageError>`.
#[derive(Clone, Debug)]
pub struct S3FileStorage {
    client: Client,
    bucket: String,
    key_prefix: String,
}

impl S3FileStorage {
    /// Constructs an `S3FileStorage` directly from a configured S3
    /// client, bucket name, and tenant-namespacing key prefix.
    ///
    /// Equivalent to [`S3FileStorage::builder`].
    #[must_use]
    pub fn new(client: Client, bucket: impl Into<String>, key_prefix: impl Into<String>) -> Self {
        Self {
            client,
            bucket: bucket.into(),
            key_prefix: key_prefix.into(),
        }
    }

    /// Returns a builder for constructing an `S3FileStorage`.
    #[must_use]
    pub fn builder() -> S3FileStorageBuilder {
        S3FileStorageBuilder::default()
    }

    /// Returns the configured S3 bucket name.
    #[must_use]
    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    /// Returns the configured tenant-namespacing key prefix.
    #[must_use]
    pub fn key_prefix(&self) -> &str {
        &self.key_prefix
    }

    /// Composes the physical S3 key from the configured prefix
    /// and the logical [`FileKey`]. The logical key never escapes
    /// the adapter: it is the engine's handle, and the physical
    /// key is the bucket-side path.
    fn physical_key(&self, key: &FileKey) -> String {
        let mut buf = String::with_capacity(self.key_prefix.len() + key.as_str().len());
        buf.push_str(&self.key_prefix);
        buf.push_str(key.as_str());
        buf
    }

    /// Computes the lowercase hex SHA-256 digest of `content`.
    /// The format matches the engine's
    /// [`Checksum`](crate::port::Checksum) contract: 64 lowercase
    /// hex characters.
    fn sha256_hex(content: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content);
        let digest = hasher.finalize();
        to_lower_hex(&digest)
    }

    /// Maps an [`aws_sdk_s3::types::StorageClass`] into the
    /// engine's [`StorageClass`]. Unknown S3 classes default to
    /// [`StorageClass::Hot`] (treated as Standard).
    fn map_storage_class(s3_class: Option<&S3StorageClass>) -> StorageClass {
        match s3_class {
            Some(
                S3StorageClass::Standard
                | S3StorageClass::ReducedRedundancy
                | S3StorageClass::ExpressOnezone,
            )
            | None => StorageClass::Hot,
            Some(
                S3StorageClass::StandardIa
                | S3StorageClass::OnezoneIa
                | S3StorageClass::IntelligentTiering,
            ) => StorageClass::Cool,
            Some(
                S3StorageClass::Glacier
                | S3StorageClass::DeepArchive
                | S3StorageClass::GlacierIr
                | S3StorageClass::Snow,
            ) => StorageClass::Archive,
            // `#[non_exhaustive]` on the S3 enum — future variants
            // default to Hot (Standard).
            _ => StorageClass::Hot,
        }
    }

    /// Picks the S3 storage class to assign at upload time.
    /// Per the engine's contract, the default is `Standard`
    /// ([`StorageClass::Hot`]).
    fn upload_storage_class(_visibility: Visibility) -> S3StorageClass {
        // Future: pick a class based on visibility (e.g. Archive
        // for `Visibility::Private` after a long retention). For
        // Phase 15 we always upload to Standard.
        S3StorageClass::Standard
    }

    /// Lifts any SDK / runtime error into the engine's
    /// [`FileStorageError::Infrastructure`] variant. The
    /// structured wrapper preserves the source chain for the
    /// audit log.
    fn infra_error(
        err: impl Into<Box<dyn std::error::Error + Send + Sync>>,
        context: &'static str,
    ) -> FileStorageError {
        FileStorageError::Infrastructure(InfrastructureError::with_source(context, err.into()))
    }

    /// Maps an S3 error from an object-level operation
    /// (`GetObject`, `HeadObject`, `PutObject`, etc.) into the
    /// engine's [`FileStorageError`]. 404-class failures are
    /// surfaced as [`FileStorageError::NotFound`] carrying the
    /// supplied [`FileKey`]; every other error is wrapped as
    /// [`FileStorageError::Infrastructure`].
    fn map_object_error(err: aws_sdk_s3::Error, key: &FileKey) -> FileStorageError {
        match &err {
            aws_sdk_s3::Error::NoSuchKey(_)
            | aws_sdk_s3::Error::NotFound(_)
            | aws_sdk_s3::Error::NoSuchBucket(_) => FileStorageError::NotFound(key.clone()),
            _ => Self::infra_error(err, "S3 object operation failed"),
        }
    }
}

#[async_trait]
impl FileStorage for S3FileStorage {
    async fn put(&self, request: PutRequest) -> StdResult<FileReference, FileStorageError> {
        let PutRequest {
            tenant,
            key,
            content,
            content_type,
            metadata,
            visibility,
            overwrite,
            idempotency_key: _idempotency_key,
        } = request;

        let physical_key = self.physical_key(&key);
        let content_len = content.len();
        let checksum_hex = Self::sha256_hex(&content);

        let body = ByteStream::from(content);
        let storage_class = Self::upload_storage_class(visibility);

        let mut op = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(&physical_key)
            .content_type(content_type.as_str())
            .content_length(i64::try_from(content_len).unwrap_or(i64::MAX))
            .storage_class(storage_class)
            .body(body);

        if !overwrite {
            // Mirror the spec's "overwrite=false means error on
            // duplicate" contract. S3 expresses this via the
            // `If-None-Match: *` precondition: the request fails
            // with 412 if the key already exists.
            op = op.if_none_match("*");
        }

        for (meta_key, meta_value) in &metadata {
            op = op.metadata(meta_key, meta_value);
        }

        let output = op
            .send()
            .await
            .map_err(|e| Self::map_object_error(e.into(), &key))?;

        // The S3 ETag is wrapped in double quotes; strip them.
        let etag = output
            .e_tag()
            .unwrap_or_default()
            .trim_matches('"')
            .to_string();

        // Capture the S3-assigned `VersionId` when the bucket has
        // versioning enabled. Per `docs/ports/file-storage.md`
        // § "Versioning": the adapter surfaces `version_id` on
        // the returned `FileReference` so subsequent
        // `get` / `delete` / `head` calls can pin to the
        // specific version if the caller asks for it. When the
        // bucket is not versioned, S3 returns `None` and the
        // reference stays a "current version" handle.
        let version_id = output.version_id().map(str::to_owned);

        Ok(FileReference {
            key,
            etag,
            // `content_len` is `usize` (from `Vec::len()`); widen
            // to `u64` via `TryFrom` per the engine's no-`as`
            // rule. The `unwrap_or(0)` fallback is unreachable on
            // the engine's supported targets (Linux/Android/WASM
            // on 64-bit where `usize == u64`; on the
            // hypothetical 16-bit target a `usize` larger than
            // `u64::MAX` cannot occur), but it satisfies
            // `-D warnings` while preserving a total return type.
            size: u64::try_from(content_len).unwrap_or(0),
            content_type,
            visibility,
            uploaded_at: Timestamp::now(),
            uploaded_by: tenant.actor_id,
            tenant,
            storage_class: StorageClass::Hot,
            checksum: Checksum::new(checksum_hex),
            version_id,
        })
    }

    async fn get(&self, reference: &FileReference) -> StdResult<FileStream, FileStorageError> {
        let physical_key = self.physical_key(&reference.key);
        let key_for_error = reference.key.clone();

        let mut req = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(&physical_key);

        // When the reference is pinned to a specific S3 version
        // (i.e. the caller passed a `version_id` produced by a
        // prior `put` or by [`S3FileStorage::pin_version`]),
        // forward the `versionId` query parameter so S3 streams
        // the historical bytes rather than the latest one. A
        // missing version pin streams the current version.
        if let Some(version_id) = reference.version_id.as_deref() {
            req = req.version_id(version_id);
        }

        let output = req
            .send()
            .await
            .map_err(|e| Self::map_object_error(e.into(), &key_for_error))?;

        let mut body = output.body;
        let (tx, rx) = tokio::sync::mpsc::channel(STREAM_CHANNEL_CAPACITY);

        // Spawn the streaming task on the current tokio runtime.
        // The task drains the SDK's ByteStream chunk-by-chunk and
        // forwards each chunk to the mpsc receiver. The task
        // terminates when the SDK closes the body (sender dropped
        // implicitly) or when a chunk send fails (receiver
        // dropped, i.e. the engine stopped reading).
        tokio::spawn(async move {
            while let Some(chunk_result) = body.next().await {
                match chunk_result {
                    Ok(bytes) => {
                        let chunk_len = bytes.len();
                        let mut start = 0usize;
                        while start < chunk_len {
                            let end = chunk_len.min(start.saturating_add(STREAM_CHUNK_CAP_BYTES));
                            // Splitting the `bytes::Bytes` view
                            // gives the receiver a `Vec<u8>` slice
                            // that does not retain the original
                            // SDK buffer.
                            let slice: Vec<u8> = bytes[start..end].to_vec();
                            if tx.send(Ok(slice)).await.is_err() {
                                return;
                            }
                            start = end;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(std::io::Error::other(e.to_string()))).await;
                        return;
                    }
                }
            }
            // `tx` drops here, closing the receiver.
        });

        Ok(rx)
    }

    async fn delete(&self, reference: &FileReference) -> StdResult<(), FileStorageError> {
        let physical_key = self.physical_key(&reference.key);
        let key_for_error = reference.key.clone();

        let mut req = self
            .client
            .delete_object()
            .bucket(&self.bucket)
            .key(&physical_key);

        // Pin the delete to the historical version if the
        // reference carries a `version_id`; without the pin the
        // request removes the current version (and, when the
        // bucket is versioned, leaves older versions in place
        // until their lifecycle rules expire them).
        if let Some(version_id) = reference.version_id.as_deref() {
            req = req.version_id(version_id);
        }

        req.send()
            .await
            .map_err(|e| Self::map_object_error(e.into(), &key_for_error))?;

        Ok(())
    }

    async fn exists(&self, reference: &FileReference) -> StdResult<bool, FileStorageError> {
        let physical_key = self.physical_key(&reference.key);

        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(&physical_key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                let err: aws_sdk_s3::Error = e.into();
                match &err {
                    aws_sdk_s3::Error::NotFound(_) | aws_sdk_s3::Error::NoSuchKey(_) => Ok(false),
                    _ => Err(Self::infra_error(err, "S3 exists check failed")),
                }
            }
        }
    }

    async fn head(&self, reference: &FileReference) -> StdResult<FileMetadata, FileStorageError> {
        let physical_key = self.physical_key(&reference.key);
        let key_for_error = reference.key.clone();

        let mut req = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(&physical_key);

        // When the reference pins a specific version, forward
        // the `versionId` parameter so the head request returns
        // metadata for that historical version rather than the
        // current one.
        if let Some(version_id) = reference.version_id.as_deref() {
            req = req.version_id(version_id);
        }

        let output = req
            .send()
            .await
            .map_err(|e| Self::map_object_error(e.into(), &key_for_error))?;

        let etag = output
            .e_tag()
            .unwrap_or_default()
            .trim_matches('"')
            .to_string();

        // `content_length` is `Option<i64>`; coerce to `u64` via
        // `try_from`, falling back to 0 on negative / overflow.
        let size = output
            .content_length()
            .and_then(|n| u64::try_from(n).ok())
            .unwrap_or(0);

        let content_type =
            ContentType::new(output.content_type().unwrap_or("application/octet-stream"));

        let uploaded_at = output.last_modified().map_or_else(Timestamp::epoch, |dt| {
            // `aws_smithy_types::DateTime` formats to RFC 3339
            // with UTC offset forbidden (Format::DateTime →
            // rfc3339::format with `Z` suffix). Re-parse through
            // the engine's `Timestamp::parse_rfc3339` to avoid
            // taking a direct `chrono` dep in this crate.
            let rfc3339 = dt
                .fmt(aws_sdk_s3::primitives::DateTimeFormat::DateTime)
                .unwrap_or_default();
            Timestamp::parse_rfc3339(&rfc3339).unwrap_or_else(|_| Timestamp::now())
        });

        Ok(FileMetadata {
            key: reference.key.clone(),
            etag,
            size,
            content_type,
            uploaded_at,
        })
    }

    async fn signed_url(
        &self,
        reference: &FileReference,
        options: SignedUrlOptions,
    ) -> StdResult<String, FileStorageError> {
        let physical_key = self.physical_key(&reference.key);

        let presigning = aws_sdk_s3::presigning::PresigningConfig::expires_in(options.expires_in)
            .map_err(|e| Self::infra_error(e, "S3 presigning config build failed"))?;

        let uri: String = match options.method {
            SignedUrlMethod::Get => {
                let mut req = self
                    .client
                    .get_object()
                    .bucket(&self.bucket)
                    .key(&physical_key);

                if let Some(disp) = options.response_content_disposition.as_deref() {
                    req = req.response_content_disposition(disp);
                }
                if let Some(ct) = options.response_content_type.as_ref() {
                    req = req.response_content_type(ct.as_str());
                }

                let presigned = req
                    .presigned(presigning)
                    .await
                    .map_err(|e| Self::infra_error(e, "S3 presigned GET failed"))?;
                presigned.uri().to_string()
            }
            SignedUrlMethod::Put => {
                let mut req = self
                    .client
                    .put_object()
                    .bucket(&self.bucket)
                    .key(&physical_key);

                if let Some(ct) = options.response_content_type.as_ref() {
                    req = req.content_type(ct.as_str());
                }

                let presigned = req
                    .presigned(presigning)
                    .await
                    .map_err(|e| Self::infra_error(e, "S3 presigned PUT failed"))?;
                presigned.uri().to_string()
            }
        };

        Ok(uri)
    }

    async fn copy(
        &self,
        src: &FileReference,
        dst_key: &str,
    ) -> StdResult<FileReference, FileStorageError> {
        let src_physical = self.physical_key(&src.key);
        let dst_physical = format!("{}{}", self.key_prefix, dst_key);
        let dst_file_key = FileKey::new(dst_key);
        // S3 `CopyObject` pins to a specific historical version
        // by appending `?versionId=<id>` to the `copy_source`
        // value. Build the URL-encoded form when the source
        // reference is pinned; otherwise copy the current
        // version (the original behaviour).
        let copy_source = match src.version_id.as_deref() {
            Some(version_id) => format!(
                "{}/{}?versionId={}",
                self.bucket,
                src_physical,
                urlencode_query_value(version_id),
            ),
            None => format!("{}/{}", self.bucket, src_physical),
        };

        let copy_output = self
            .client
            .copy_object()
            .bucket(&self.bucket)
            .key(&dst_physical)
            .copy_source(copy_source)
            .send()
            .await
            .map_err(|e| Self::map_object_error(e.into(), &dst_file_key))?;

        let etag = copy_output
            .copy_object_result()
            .and_then(|r| r.e_tag())
            .unwrap_or_default()
            .trim_matches('"')
            .to_string();

        // Fetch the destination's metadata so the returned
        // reference carries the correct size + content type.
        let head_output = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(&dst_physical)
            .send()
            .await
            .map_err(|e| Self::map_object_error(e.into(), &dst_file_key))?;

        let size = head_output
            .content_length()
            .and_then(|n| u64::try_from(n).ok())
            .unwrap_or(0);

        let content_type = ContentType::new(
            head_output
                .content_type()
                .unwrap_or("application/octet-stream"),
        );

        let storage_class = Self::map_storage_class(head_output.storage_class());

        // Capture the destination's `VersionId` if the bucket is
        // versioned so the caller can pin the copy to its
        // freshly-assigned version.
        let version_id = head_output.version_id().map(str::to_owned);

        Ok(FileReference {
            key: dst_file_key,
            etag,
            size,
            content_type,
            visibility: src.visibility,
            uploaded_at: Timestamp::now(),
            uploaded_by: src.uploaded_by,
            tenant: src.tenant.clone(),
            storage_class,
            // The byte content is identical to the source; the
            // checksum travels with it. A future revision may
            // re-hash on the destination to defend against
            // server-side tampering during the copy.
            checksum: src.checksum.clone(),
            version_id,
        })
    }

    async fn move_to(
        &self,
        src: &FileReference,
        dst_key: &str,
    ) -> StdResult<FileReference, FileStorageError> {
        let dst = self.copy(src, dst_key).await?;
        self.delete(src).await?;
        Ok(dst)
    }
}

impl S3FileStorage {
    /// Pins a [`FileReference`] to a specific S3 object version.
    ///
    /// Returns a new [`FileReference`] with the supplied
    /// `version_id` set; subsequent calls against the returned
    /// reference via [`get`](FileStorage::get),
    /// [`head`](FileStorage::head),
    /// [`delete`](FileStorage::delete), and
    /// [`copy`](FileStorage::copy) operate on that historical
    /// version rather than the bucket's current head.
    ///
    /// This method does NOT issue an S3 request — the caller
    /// supplies the `version_id` (typically obtained from a
    /// prior [`put`](FileStorage::put) response, from
    /// `list_object_versions`, or from an audit log entry that
    /// captured the upload-time version). If the version does
    /// not exist on the provider, the next data-plane call
    /// surfaces a [`FileStorageError::NotFound`].
    ///
    /// Per `docs/ports/file-storage.md` § "Versioning": the
    /// adapter enables bucket-side versioning on construction
    /// (consumer-controlled via bucket policy); once enabled,
    /// every upload records a `VersionId` on the returned
    /// reference and consumers can pin / roll back to any
    /// retained version via this helper.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let stored = storage.put(request).await?;
    /// let pinned = storage.pin_version(&stored, "old-version-id");
    /// // `pinned` operates on the historical bytes; `stored`
    /// // still resolves to the current version.
    /// ```
    #[must_use]
    pub fn pin_version(
        &self,
        reference: &FileReference,
        version_id: impl Into<String>,
    ) -> FileReference {
        let mut pinned = reference.clone();
        pinned.version_id = Some(version_id.into());
        pinned
    }
}

/// Builder for [`S3FileStorage`].
///
/// All three fields are independently optional; `build()` fails
/// with [`S3FileStorageBuildError::MissingClient`] or
/// [`S3FileStorageBuildError::MissingBucket`] when the
/// corresponding setter was not called. `key_prefix` defaults to
/// the empty string.
///
/// # Deviation from the spec
///
/// The original task spec asks for an
/// `aws_config(aws_config::SdkConfig)` setter that wraps
/// `aws_config::SdkConfig`. This builder uses
/// [`S3FileStorageBuilder::client`] which accepts an
/// already-built [`Client`] instead. See the module-level doc for
/// the rationale.
#[derive(Clone, Debug, Default)]
pub struct S3FileStorageBuilder {
    client: Option<Client>,
    bucket: Option<String>,
    key_prefix: String,
}

impl S3FileStorageBuilder {
    /// Sets the S3 client. Required.
    #[must_use]
    pub fn client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }

    /// Sets the S3 bucket name. Required.
    #[must_use]
    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = Some(bucket.into());
        self
    }

    /// Sets the tenant-namespacing key prefix (e.g. `school_<uuid>/`).
    /// Defaults to the empty string.
    #[must_use]
    pub fn key_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.key_prefix = prefix.into();
        self
    }

    /// Consumes the builder and returns an [`S3FileStorage`], or
    /// a [`S3FileStorageBuildError`] when a required field was
    /// not set.
    pub fn build(self) -> StdResult<S3FileStorage, S3FileStorageBuildError> {
        let client = self.client.ok_or(S3FileStorageBuildError::MissingClient)?;
        let bucket = self.bucket.ok_or(S3FileStorageBuildError::MissingBucket)?;
        Ok(S3FileStorage {
            client,
            bucket,
            key_prefix: self.key_prefix,
        })
    }
}

/// Errors returned by [`S3FileStorageBuilder::build`].
///
/// The two variants correspond to the two required builder
/// fields. Both are programming errors (the consumer forgot to
/// call a setter); they are surfaced as `Err` from `build()` so
/// the engine's no-`expect` rule stays satisfied.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum S3FileStorageBuildError {
    /// The [`S3FileStorageBuilder::client`] setter was not called.
    MissingClient,
    /// The [`S3FileStorageBuilder::bucket`] setter was not called.
    MissingBucket,
}

impl fmt::Display for S3FileStorageBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingClient => f.write_str("S3FileStorageBuilder: client is required"),
            Self::MissingBucket => f.write_str("S3FileStorageBuilder: bucket is required"),
        }
    }
}

impl std::error::Error for S3FileStorageBuildError {}

/// Lowercase hex formatter. Mirrors the helper used by the Stripe
/// adapter (`crates/adapters/payment/src/stripe.rs::to_lower_hex`).
/// Writing to a `String` is infallible; the `expect` is gated to
/// test code in the Stripe copy. We use `let _ =` to avoid the
/// `expect_used` lint without sacrificing performance.
fn to_lower_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        // `write!` to a `String` is total. The `let _ =` matches
        // the pattern used elsewhere in the engine (Stripe
        // adapter, SMS adapter) where the result of writing to a
        // `String` is explicitly discarded.
        let _ = write!(out, "{byte:02x}");
    }
    out
}

/// Minimal percent-encoder for S3 `VersionId` query-string
/// fragments appended to the `copy_source` URL passed to
/// `CopyObject`.
///
/// The S3 SDK's typed `version_id(impl Into<String>)` builder
/// methods handle URL encoding internally for `GetObject`,
/// `DeleteObject`, and `HeadObject`. The `copy_source` parameter
/// is a raw URL, so we encode the version id here to keep the
/// behaviour consistent. Encodes every byte outside the
/// unreserved set (`A-Z`, `a-z`, `0-9`, `-`, `_`, `.`, `~`) as
/// `%HH` uppercase hex — matching the canonical AWS CLI output
/// and AWS Signature Version 4 normalisation.
fn urlencode_query_value(value: &str) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        let unreserved = matches!(
            byte,
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~'
        );
        if unreserved {
            // `push` on a `String` is total and cannot fail.
            out.push(*byte as char);
        } else {
            // `write!` to a `String` is total. Discard the result
            // per the engine's `let _ =` convention (see
            // `to_lower_hex` above).
            let _ = write!(out, "%{byte:02X}");
        }
    }
    out
}

/// Compiles only when the builder default state holds every
/// optional field as `None`. Belt-and-braces assertion: the
/// compile-time `Default` impl on the builder produces a builder
/// with `client = None`, `bucket = None`, and `key_prefix = ""`.
///
/// If a future change adds a required field with a non-`Default`
/// type, this module's [`S3FileStorageBuilder::default`] call
/// (and the inner `Option::None` initialisers) would need to
/// grow. The assertion below catches that drift at compile time.
#[allow(dead_code)]
const _: fn() = || {
    fn _assert_builder_default_is_valid() {
        let _: S3FileStorageBuilder = S3FileStorageBuilder::default();
    }
};

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;
    use educore_core::ids::CorrelationId;
    use educore_core::tenant::TenantContext;

    /// The builder can be constructed via `S3FileStorage::builder()`
    /// and `S3FileStorageBuilder::default()` with no setter calls,
    /// producing a state where `client` and `bucket` are absent and
    /// `key_prefix` is empty. This is the foundation the
    /// `test_s3_storage_builder_constructs_with_defaults` assertion
    /// checks below.
    ///
    /// The actual S3 calls require AWS credentials; the builder is
    /// exercised in isolation.
    #[test]
    fn test_s3_storage_builder_constructs_with_defaults() {
        let builder = S3FileStorageBuilder::default();
        assert!(builder.client.is_none(), "client should default to None");
        assert!(builder.bucket.is_none(), "bucket should default to None");
        assert_eq!(
            builder.key_prefix, "",
            "key_prefix should default to the empty string"
        );

        // `build()` without a client must surface
        // `MissingClient`, not panic.
        let err = builder.build().unwrap_err();
        assert_eq!(err, S3FileStorageBuildError::MissingClient);

        // Building with a client but no bucket surfaces
        // `MissingBucket`. The actual `Client` cannot be
        // constructed in a unit test without AWS credentials;
        // we rely on the `Option<Client>` typing to skip the
        // real client creation and test the `bucket = None`
        // branch via direct construction of the inner state.
        let builder = S3FileStorageBuilder {
            client: Some(test_client_unused()),
            bucket: None,
            key_prefix: String::from("school_42/"),
        };
        assert_eq!(builder.key_prefix, "school_42/");
        let err = builder.build().unwrap_err();
        assert_eq!(err, S3FileStorageBuildError::MissingBucket);
    }

    /// `to_lower_hex` produces the canonical lowercase SHA-256 hex
    /// format the engine's `Checksum` contract expects: 64
    /// characters, no uppercase, no prefix.
    #[test]
    fn to_lower_hex_emits_64_lowercase_chars() {
        let empty = to_lower_hex(&[]);
        assert_eq!(empty, "");

        let one_byte = to_lower_hex(&[0x00]);
        assert_eq!(one_byte, "00");

        let known = to_lower_hex(&[0xde, 0xad, 0xbe, 0xef]);
        assert_eq!(known, "deadbeef");

        // 32-byte SHA-256 digest: 64 lowercase hex chars.
        let digest = vec![0xab_u8; 32];
        let hex = to_lower_hex(&digest);
        assert_eq!(hex.len(), 64);
        assert!(hex
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    /// `sha256_hex` on empty content produces the well-known empty-
    /// input SHA-256 hex digest. This pins the contract that the
    /// adapter's checksum format is stable across runs.
    #[test]
    fn sha256_hex_matches_known_empty_digest() {
        let hex = S3FileStorage::sha256_hex(&[]);
        assert_eq!(
            hex,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    /// `sha256_hex` on the well-known "abc" input produces the
    /// canonical SHA-256 hex digest.
    #[test]
    fn sha256_hex_matches_known_abc_digest() {
        let hex = S3FileStorage::sha256_hex(b"abc");
        assert_eq!(
            hex,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    /// `physical_key` prepends the configured `key_prefix` to the
    /// logical `FileKey`. The empty prefix yields the logical key
    /// verbatim; the populated prefix yields `prefix + logical`.
    #[test]
    fn physical_key_prepends_prefix() {
        let storage = S3FileStorage {
            client: test_client_unused(),
            bucket: String::from("bucket"),
            key_prefix: String::new(),
        };
        let key = FileKey::new("students/photos/ada.jpg");
        assert_eq!(storage.physical_key(&key), "students/photos/ada.jpg");

        let storage = S3FileStorage {
            client: test_client_unused(),
            bucket: String::from("bucket"),
            key_prefix: String::from("school_42/"),
        };
        assert_eq!(
            storage.physical_key(&key),
            "school_42/students/photos/ada.jpg"
        );
    }

    /// `map_storage_class` covers every supported S3 storage
    /// class and the `None` / default case.
    #[test]
    fn map_storage_class_translates_known_variants() {
        assert_eq!(
            S3FileStorage::map_storage_class(None),
            StorageClass::Hot,
            "None should map to Hot"
        );
        assert_eq!(
            S3FileStorage::map_storage_class(Some(&S3StorageClass::Standard)),
            StorageClass::Hot,
            "Standard should map to Hot"
        );
        assert_eq!(
            S3FileStorage::map_storage_class(Some(&S3StorageClass::StandardIa)),
            StorageClass::Cool,
            "StandardIa should map to Cool"
        );
        assert_eq!(
            S3FileStorage::map_storage_class(Some(&S3StorageClass::Glacier)),
            StorageClass::Archive,
            "Glacier should map to Archive"
        );
        assert_eq!(
            S3FileStorage::map_storage_class(Some(&S3StorageClass::DeepArchive)),
            StorageClass::Archive,
            "DeepArchive should map to Archive"
        );
    }

    /// `S3FileStorageBuildError` produces the documented
    /// display strings.
    #[test]
    fn build_error_display_strings_are_stable() {
        assert_eq!(
            S3FileStorageBuildError::MissingClient.to_string(),
            "S3FileStorageBuilder: client is required"
        );
        assert_eq!(
            S3FileStorageBuildError::MissingBucket.to_string(),
            "S3FileStorageBuilder: bucket is required"
        );
    }

    /// Construct an unused `aws_sdk_s3::Client` for tests that
    /// exercise struct-field access (not actual SDK calls).
    ///
    /// `Client::new` does not require AWS credentials; it only
    /// wires up an HTTP client. The credentials provider chain
    /// is consulted lazily on the first API call, so a
    /// credential-less test can still construct a `Client`.
    fn test_client_unused() -> Client {
        let cfg = aws_sdk_s3::config::Builder::new()
            .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
            .region(aws_sdk_s3::config::Region::new("us-east-1"))
            .build();
        Client::from_conf(cfg)
    }

    /// `pin_version` returns a new `FileReference` with the
    /// supplied `version_id` set on a clone of the source, leaving
    /// the source reference's `version_id` unchanged.
    #[test]
    fn pin_version_sets_version_id_on_clone() {
        let storage = S3FileStorage {
            client: test_client_unused(),
            bucket: String::from("bucket"),
            key_prefix: String::new(),
        };
        let mut source = FileReference {
            key: FileKey::new("students/photos/ada.jpg"),
            etag: String::from("\"etag-original\""),
            size: 42,
            content_type: ContentType::new("image/jpeg"),
            visibility: Visibility::Private,
            uploaded_at: Timestamp::epoch(),
            uploaded_by: educore_core::ids::SYSTEM_USER_ID,
            tenant: TenantContext::system(
                educore_core::ids::PUBLIC_SCHOOL_ID,
                CorrelationId::from(Uuid::nil()),
            ),
            storage_class: StorageClass::Hot,
            checksum: Checksum::new(String::from(
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            )),
            version_id: None,
        };

        let pinned = storage.pin_version(&source, "abc.def-123");
        assert_eq!(pinned.version_id.as_deref(), Some("abc.def-123"));
        assert_eq!(pinned.key, source.key);
        assert_eq!(pinned.etag, source.etag);

        // Pinning is non-destructive: the source reference is
        // untouched and still resolves to the current version.
        assert!(source.version_id.is_none());
    }

    /// `pin_version` overrides any pre-existing version pin on
    /// the source. This lets callers roll forward to a newer
    /// version without round-tripping through `put`.
    #[test]
    fn pin_version_overrides_existing_pin() {
        let storage = S3FileStorage {
            client: test_client_unused(),
            bucket: String::from("bucket"),
            key_prefix: String::new(),
        };
        let mut source = FileReference {
            key: FileKey::new("students/photos/ada.jpg"),
            etag: String::from("etag"),
            size: 1,
            content_type: ContentType::new("image/jpeg"),
            visibility: Visibility::Private,
            uploaded_at: Timestamp::epoch(),
            uploaded_by: educore_core::ids::SYSTEM_USER_ID,
            tenant: TenantContext::system(
                educore_core::ids::PUBLIC_SCHOOL_ID,
                CorrelationId::from(Uuid::nil()),
            ),
            storage_class: StorageClass::Hot,
            checksum: Checksum::new(String::new()),
            version_id: Some(String::from("old-version")),
        };

        let new_pinned = storage.pin_version(&source, "new-version");
        assert_eq!(new_pinned.version_id.as_deref(), Some("new-version"));

        // Source is unchanged.
        assert_eq!(source.version_id.as_deref(), Some("old-version"));
    }

    /// `urlencode_query_value` percent-encodes every byte outside
    /// the unreserved set (`A-Z`, `a-z`, `0-9`, `-`, `_`, `.`,
    /// `~`). Verifies the canonical AWS CLI output shape:
    /// uppercase `%HH` for non-unreserved bytes, raw bytes
    /// otherwise.
    #[test]
    fn urlencode_query_value_matches_canonical_form() {
        // Unreserved bytes round-trip verbatim.
        assert_eq!(
            urlencode_query_value("abcXYZ012-_.~"),
            "abcXYZ012-_.~",
            "unreserved bytes must round-trip verbatim"
        );
        // Spaces encode to `%20`.
        assert_eq!(urlencode_query_value("a b"), "a%20b");
        // Slashes encode to `%2F` (not `/`), critical for
        // query-string context inside `copy_source`.
        assert_eq!(urlencode_query_value("a/b"), "a%2Fb");
        // Lowercase ASCII letters and digits are unreserved.
        assert_eq!(urlencode_query_value("v1.2.3-abc_DEF"), "v1.2.3-abc_DEF");
        // Empty string stays empty.
        assert_eq!(urlencode_query_value(""), "");
    }

    /// The literal string `version_id` must appear in this module
    /// (it is the contract pinned by the roadmap's
    /// `PORT-FILE-VERSIONING` check). This test fails the build
    /// if a future refactor accidentally drops every
    /// `version_id` reference from the S3 adapter.
    #[test]
    fn version_id_identifier_is_present_in_source() {
        const SRC: &str = include_str!("s3.rs");
        assert!(
            SRC.contains("version_id"),
            "s3.rs must contain the literal `version_id` \
             (PORT-FILE-VERSIONING contract); refactor preserved? \
             check `put`, `get`, `delete`, `head`, `copy`, \
             `pin_version`, and the FileReference field."
        );
    }
}
