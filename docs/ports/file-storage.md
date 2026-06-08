# File Storage Port

## Purpose

The file storage port stores and retrieves files (student photos,
homework submissions, certificates, ID card templates, content
images, documents). The engine does not own S3, GCS, or local
filesystems. The consumer supplies an adapter.

## Trait: `FileStorage`

```rust
#[async_trait]
pub trait FileStorage: Send + Sync + std::fmt::Debug {
    async fn put(&self, request: PutRequest) -> Result<FileReference>;
    async fn get(&self, reference: &FileReference) -> Result<FileStream>;
    async fn delete(&self, reference: &FileReference) -> Result<()>;
    async fn exists(&self, reference: &FileReference) -> Result<bool>;
    async fn head(&self, reference: &FileReference) -> Result<FileMetadata>;
    async fn signed_url(&self, reference: &FileReference, options: SignedUrlOptions) -> Result<Url>;
    async fn copy(&self, src: &FileReference, dst_key: &str) -> Result<FileReference>;
    async fn move_to(&self, src: &FileReference, dst_key: &str) -> Result<FileReference>;
}
```

The trait is object-safe.

## PutRequest

```rust
pub struct PutRequest {
    pub tenant: TenantContext,
    pub key: FileKey,                  // logical path, scoped to tenant
    pub content: Bytes,
    pub content_type: ContentType,
    pub metadata: BTreeMap<String, String>,
    pub visibility: Visibility,
    pub overwrite: bool,
    pub idempotency_key: Option<IdempotencyKey>,
}

pub enum Visibility {
    Private,
    Public,
    TenantPrivate,                     // accessible to any user in the tenant
}
```

`FileKey` is a typed string. The engine does not accept raw path
strings; the consumer's adapter enforces a safe key namespace.

## FileReference

```rust
pub struct FileReference {
    pub key: FileKey,
    pub etag: String,
    pub size: u64,
    pub content_type: ContentType,
    pub visibility: Visibility,
    pub uploaded_at: Timestamp,
    pub uploaded_by: UserId,
    pub tenant: TenantContext,
    pub storage_class: StorageClass,
    pub checksum: Checksum,
}

pub enum StorageClass {
    Hot,        // frequent access
    Cool,       // infrequent access
    Archive,    // long-term, slower retrieval
}
```

The `FileReference` is what domain aggregates store. They do not store
URLs.

## Idempotency

`idempotency_key` is used by the adapter to deduplicate retry uploads.
A retry of the same upload returns the same `FileReference` without
re-uploading.

## Content-Addressable Hashing

The adapter computes a SHA-256 checksum on upload. The engine
verifies the checksum on read. Mismatches fail the read.

## Key Namespacing

The adapter namespaces keys by tenant. The full key is
`<school_id>/<domain>/<aggregate>/<id>/<filename>`. The adapter is
responsible for enforcing this prefix. A consumer who stores multiple
schools in one bucket cannot accidentally cross-tenant access because
keys are prefixed.

## Signed URLs

`signed_url` produces a time-limited URL for a private file. The
adapter uses the storage provider's signing mechanism (e.g. S3
presigned URLs, GCS signed URLs, local token URLs).

```rust
pub struct SignedUrlOptions {
    pub expires_in: Duration,
    pub method: SignedUrlMethod,
    pub response_content_disposition: Option<String>,
    pub response_content_type: Option<ContentType>,
}

pub enum SignedUrlMethod {
    Get,
    Put,
}
```

## Streaming Downloads

`get` returns a `FileStream` that yields the file in chunks. The
engine does not load entire files into memory.

```rust
pub type FileStream = Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>;
```

## Versioning

If the underlying provider supports versioning (S3 does), the adapter
enables it. Older versions are retained for a configurable period.

## Lifecycle Rules

The adapter supports lifecycle rules configured per bucket:

- Transition from Hot to Cool after N days.
- Transition from Cool to Archive after M days.
- Expire (delete) after P days.

Rules are configured by the consumer, not the engine.

## Error Type

```rust
#[derive(Debug, thiserror::Error)]
pub enum FileStorageError {
    #[error("file not found: {0}")] NotFound(FileKey),
    #[error("permission denied")] PermissionDenied,
    #[error("checksum mismatch")] ChecksumMismatch,
    #[error("content too large: {0} bytes, max {1}")] TooLarge(u64, u64),
    #[error("unsupported content type: {0}")] UnsupportedContentType(ContentType),
    #[error("key invalid: {0}")] InvalidKey(String),
    #[error("storage class not available: {0}")] StorageClassUnavailable(String),
    #[error("infrastructure error: {0}")] Infrastructure(#[source] Box<dyn std::error::Error + Send + Sync>),
}
```

## Worked Example

```rust
let photo_ref = engine.files().put(PutRequest {
    tenant,
    key: FileKey::new("students/photos/ada.jpg")?,
    content: photo_bytes,
    content_type: ContentType::Jpeg,
    metadata: btreemap! { "uploaded_via".into() => "admission_form".into() },
    visibility: Visibility::TenantPrivate,
    overwrite: false,
    idempotency_key: None,
}).await?;

let student = engine.students().admit(AdmitStudentCommand {
    tenant,
    photo: Some(photo_ref),
    ...
}).await?;
```

A teacher downloads a homework submission:

```rust
let url = engine.files().signed_url(&submission.file_ref, SignedUrlOptions {
    expires_in: Duration::from_minutes(15),
    method: SignedUrlMethod::Get,
    response_content_disposition: Some("attachment; filename=\"homework.pdf\"".into()),
    response_content_type: None,
}).await?;
```

## Object Safety

`FileStorage` is object-safe.

## Testing

- Unit tests of put, get, delete, exists, head.
- Integration tests of signed URL generation and expiration.
- A test of cross-tenant denial.
- A test of checksum mismatch.
- A test of content type validation.
- A test of large file streaming.
- A test of idempotent retry.

## Offline Mode

In offline mode, the consumer's adapter stores files in a local
directory. Files are synchronized to the central store on reconnect.
The `FileReference` carries a local URI during the offline period.

## Audit

Every put, get, delete, and signed-URL generation is recorded in the
audit log. The log includes the key, the actor, and the size. File
content is never logged.
