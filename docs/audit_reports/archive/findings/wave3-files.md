# Audit findings: educore-files (Phase 15 / adapters)

**Scope:** `crates/adapters/files/` (5 src files: `lib.rs`,
`port.rs`, `errors.rs`, `local.rs`, `s3.rs`, `services.rs`; 1 test
file: `tests/files_integration.rs`),
`docs/ports/file-storage.md`,
`docs/handoff/PHASE-15-HANDOFF.md`,
`docs/code-standards.md`, `AGENTS.md`.

**Backends shipped:** `LocalFileStorage` (filesystem) +
`S3FileStorage` (AWS S3). `Cargo.toml` description advertises
"File storage port, S3-compatible, GCS, local filesystem
adapters." but **no GCS adapter ships**.

**Total findings:** 28

---

### FINDING 1

- **id:** ADAPTER-FILE-001
- **area:** adapters
- **severity:** Critical
- **location:** crates/adapters/files/src/s3.rs:155-160, 241-306
- **description:** `S3FileStorage::physical_key` only prepends
  the consumer-supplied `key_prefix` to the logical `FileKey`;
  it does NOT prefix `request.tenant.school_id`. The
  `PutRequest::tenant` field is destructured at line 243 but
  `tenant.school_id` is never used to compute `physical_key`
  (only `tenant.actor_id` is consumed at line 301 for
  `uploaded_by`). The port contract puts the
  `<school_id>/...` prefix on the adapter — the S3 adapter
  delegates it to the consumer's builder configuration, so a
  consumer who forgets to set a per-tenant `key_prefix` writes
  all schools into a flat namespace and a cross-tenant read is
  a one-line mistake.
- **expected:** `docs/ports/file-storage.md:91-95` — "The
  adapter namespaces keys by tenant. The full key is
  `<school_id>/<domain>/<aggregate>/<id>/<filename>`. The
  adapter is responsible for enforcing this prefix. A consumer
  who stores multiple schools in one bucket cannot
  accidentally cross-tenant access because keys are prefixed."
- **evidence:**
  ```rust
  fn physical_key(&self, key: &FileKey) -> String {
      let mut buf = String::with_capacity(self.key_prefix.len() + key.as_str().len());
      buf.push_str(&self.key_prefix);
      buf.push_str(key.as_str());
      buf
  }
  ```
  with `tenant.school_id` unused across all 8 trait methods:
  `let PutRequest { tenant, key, ... } = request;`
  `let physical_key = self.physical_key(&key);` at
  `crates/adapters/files/src/s3.rs:241-253`.

---

### FINDING 2

- **id:** ADAPTER-FILE-002
- **area:** adapters
- **severity:** Critical
- **location:** crates/adapters/files/src/local.rs:148-152, 343-382
- **description:** `LocalFileStorage` does NOT prefix
  `request.tenant.school_id` onto the resolved filesystem
  path. `LocalFileStorage::put` (line 343) calls
  `self.resolve(request.key.as_str())?` and the `resolve`
  method (line 174) only composes `root / key_prefix / key` —
  neither `tenant` nor `tenant.school_id` appears anywhere on
  the path. The crate's own module-level doc acknowledges this
  at line 145: "Tenant isolation is the caller's responsibility
  — the engine namespaces the key with the `school_id` before
  calling `put`", which is the inverse of what the port
  contract says (the ADAPTER does it).
- **expected:** `docs/ports/file-storage.md:91-95` — "The
  adapter namespaces keys by tenant. The full key is
  `<school_id>/<domain>/<aggregate>/<id>/<filename>`. The
  adapter is responsible for enforcing this prefix."
- **evidence:**
  ```rust
  async fn put(&self, request: PutRequest) -> PortResult<FileReference> {
      let path = self.resolve(request.key.as_str())?;
  ```
  at `crates/adapters/files/src/local.rs:343-344`. All other
  trait methods (`get`, `delete`, `exists`, `head`,
  `signed_url`, `copy`, `move_to`) follow the same pattern;
  no method in `crates/adapters/files/src/local.rs` reads
  `request.tenant.school_id`.

---

### FINDING 3

- **id:** ADAPTER-FILE-003
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/files/src/s3.rs:241-306, 26-30
- **description:** `S3FileStorage::put` does not implement
  idempotency on `PutRequest::idempotency_key`. The field is
  destructured into `_idempotency_key` at line 250 (discarded)
  and the module-level docstring at line 26 explicitly
  acknowledges "`S3FileStorage::put` is **not** idempotent on
  `PutRequest::idempotency_key` at the S3 layer; the adapter
  documents the key on the returned `FileReference` but does
  not consult it before upload." A retry of the same upload
  with the same key will create N S3 objects and N storage
  charges.
- **expected:** `docs/ports/file-storage.md:80-82` —
  "`idempotency_key` is used by the adapter to deduplicate
  retry uploads. A retry of the same upload returns the same
  `FileReference` without re-uploading."
- **evidence:** `idempotency_key: _idempotency_key,` at
  `crates/adapters/files/src/s3.rs:250`.

---

### FINDING 4

- **id:** ADAPTER-FILE-004
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/files/src/local.rs:69-73, 343-382
- **description:** `LocalFileStorage::put` does not implement
  idempotency on `PutRequest::idempotency_key`. The module-
  level docstring at line 69 explicitly documents "**No
  idempotency cache.** `PutRequest::idempotency_key` is
  accepted on the wire but not deduplicated — a retry
  re-uploads." The field is set to `None` on the test fixture
  and is never read on the put path.
- **expected:** `docs/ports/file-storage.md:80-82` — see
  Finding 3.
- **evidence:** `crates/adapters/files/src/local.rs:69-73`:
  ```text
  //! 4. **No idempotency cache.** `PutRequest::idempotency_key`
  //!    is accepted on the wire but not deduplicated — a retry
  //!    re-uploads. ...
  ```

---

### FINDING 5

- **id:** ADAPTER-FILE-005
- **area:** adapters
- **severity:** Critical
- **location:** crates/adapters/files/src/s3.rs:241-306,
  crates/adapters/files/src/local.rs:343-382,
  crates/adapters/files/src/errors.rs:73-76
- **description:** Neither `S3FileStorage::put` nor
  `LocalFileStorage::put` enforces an upload size limit. The
  port contract documents `FileStorageError::TooLarge(u64,
  u64)` (errors.rs:73-76) for "content to upload exceeds the
  adapter's configured maximum", but neither adapter ever
  constructs it; `S3FileStorage::put` calls `put_object()`
  with whatever `content.len()` the caller provides (mapped
  via `i64::try_from(content_len).unwrap_or(i64::MAX)` at
  line 266), and `LocalFileStorage::put` calls
  `tokio::fs::write(&path, &request.content)` with no length
  check. A consumer can OOM the process or pin the disk by
  uploading a 10 GB object.
- **expected:** `docs/ports/file-storage.md:75-77` — port
  error variant `TooLarge(u64, u64)` and the engine's port
  contract at `crates/adapters/files/src/port.rs:482-485`:
  "The engine does not impose an upper bound; the adapter
  enforces its own limit and returns
  `FileStorageError::TooLarge` on oversize content."
- **evidence:** No `TooLarge(` constructor call exists
  anywhere in `crates/adapters/files/src/s3.rs` or
  `crates/adapters/files/src/local.rs` (only in
  `crates/adapters/files/src/errors.rs:260` test). The local
  `put` body at `crates/adapters/files/src/local.rs:355-365`
  writes the content directly without any size gate.

---

### FINDING 6

- **id:** ADAPTER-FILE-006
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/files/src/s3.rs:241-306,
  crates/adapters/files/src/local.rs:343-382,
  crates/adapters/files/src/errors.rs:78-81
- **description:** Neither adapter validates
  `PutRequest::content_type` against any allow-list. The port
  documents `FileStorageError::UnsupportedContentType(ContentType)`
  (errors.rs:78-81) for "The adapter does not accept the
  supplied MIME type", but no construction site exists;
  `S3FileStorage::put` forwards the caller-supplied MIME
  string verbatim to S3 (`s3.rs:265`:
  `.content_type(content_type.as_str())`), and the local
  adapter accepts and stores it as-is. A consumer can upload
  `application/x-msdownload` (Windows EXE), `text/html` with
  embedded `<script>` XSS payloads, or any other arbitrary
  MIME and the adapter will not raise an error.
- **expected:** `docs/ports/file-storage.md:148-150` and
  `crates/adapters/files/src/port.rs:488-491`: "The adapter
  may reject unknown or disallowed types with
  `FileStorageError::UnsupportedContentType`."
- **evidence:** No `UnsupportedContentType(` constructor call
  exists anywhere in `crates/adapters/files/src/s3.rs` or
  `crates/adapters/files/src/local.rs` (only in
  `crates/adapters/files/src/errors.rs:265` test). The S3 put
  forwards verbatim at `crates/adapters/files/src/s3.rs:265`.

---

### FINDING 7

- **id:** ADAPTER-FILE-007
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/files/src/s3.rs:33-39, 308-359,
  crates/adapters/files/src/local.rs:384-421,
  crates/adapters/files/src/port.rs:681-688
- **description:** Neither `get` implementation re-verifies
  the SHA-256 checksum of the streamed bytes against
  `reference.checksum`. The port contract at port.rs:681-688
  says "The adapter MUST verify the content hash against
  `reference.checksum` and MUST surface a
  `FileStorageError::ChecksumMismatch` on a mismatch." The S3
  module doc at line 37 acknowledges the gap: "Reads do not
  currently re-verify the checksum on the streamed bytes;
  consumers that require wire-level integrity verification
  must layer it on top of the `FileStream`." The local
  adapter's `get` (line 384) opens the file and pushes raw
  4 KB chunks through the channel without computing any hash.
  The `ChecksumMismatch` variant (errors.rs:67-71) is never
  constructed.
- **expected:** `crates/adapters/files/src/port.rs:681-688` and
  `docs/ports/file-storage.md:84-87` — "The adapter computes a
  SHA-256 checksum on upload. The engine verifies the checksum
  on read. Mismatches fail the read."
- **evidence:** `crates/adapters/files/src/s3.rs:33-39`
  (module doc acknowledging the gap), and no
  `ChecksumMismatch` constructor call in any non-test source.

---

### FINDING 8

- **id:** ADAPTER-FILE-008
- **area:** adapters
- **severity:** Critical
- **location:** crates/adapters/files/src/s3.rs (whole),
  crates/adapters/files/src/local.rs (whole),
  docs/ports/file-storage.md:209-213
- **description:** Neither `S3FileStorage` nor
  `LocalFileStorage` records an audit event for `put`, `get`,
  `delete`, or `signed_url`. The port contract at
  file-storage.md:209-213 mandates: "Every put, get, delete,
  and signed-URL generation is recorded in the audit log. The
  log includes the key, the actor, and the size. File content
  is never logged." No `audit_log`, `audit_event`,
  `record_audit`, `write_audit`, `tracing::*`, or `log::*`
  call exists in either adapter. The 5 phase-15 handoff
  "Headline numbers" claim "1 net-new `AuditTarget` variant:
  `FileReference`" in `educore-audit`, but no code in the file
  adapter ever emits an event with that variant.
- **expected:** `docs/ports/file-storage.md:209-213` — "Every
  put, get, delete, and signed-URL generation is recorded in
  the audit log. The log includes the key, the actor, and the
  size. File content is never logged."
- **evidence:** Searching `crates/adapters/files/src/` for
  `audit_log|put_audit|record_audit|write_audit|tracing::|log::`
  returns zero matches.

---

### FINDING 9

- **id:** ADAPTER-FILE-009
- **area:** adapters
- **severity:** Critical
- **location:** crates/adapters/files/src/local.rs:214-219,
  473-502
- **description:** `LocalFileStorage::sign` includes
  `expires_in` (a relative duration, e.g. 60) in the HMAC
  input rather than an absolute `expires_at` timestamp, and
  there is no `Timestamp::now()` comparison anywhere on the
  local signed-URL read path. The module doc at line 53-56
  explicitly states: "The current wall clock is intentionally
  NOT part of the token, so URLs are reproducible across
  processes and replays. A production adapter that needs hard
  expiry should embed an absolute timestamp and verify it at
  fetch time." A URL minted today with `expires_in =
  Duration::from_secs(60)` validates identically in 2030
  because the signature input does not depend on absolute
  time. This is the inverse of the
  `services::SignedUrlService::verify` contract
  (services.rs:187-195), which DOES check `Timestamp::now() <
  expires_at`.
- **expected:** `docs/ports/file-storage.md:99-101` —
  "`signed_url` produces a time-limited URL for a private
  file." and `crates/adapters/files/src/port.rs:407-413`:
  "The returned URL MUST expire after `expires_in`."
- **evidence:**
  ```rust
  fn sign(&self, key: &str, expires_in_secs: u64) -> String {
      let message = format!("{key}|{expires_in_secs}");
      let mac = hmac_sha256(&self.signing_secret, message.as_bytes());
      hex_encode(&mac)
  }
  ```
  at `crates/adapters/files/src/local.rs:215-219`. No
  `SystemTime::now()` or `Timestamp::now()` call appears in
  the local signed_url codepath.

---

### FINDING 10

- **id:** ADAPTER-FILE-010
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/files/src/local.rs:131-135,
  258-296, 301-324
- **description:** `LocalFileStorageBuilder::new()` seeds
  `signing_secret` from a well-known compile-time constant
  `DEFAULT_SIGNING_SECRET =
  b"educore-local-file-storage-default-signing-secret-do-not-use-in-prod"`.
  `build()` (line 301) does NOT compare the configured secret
  against this constant; it accepts the default silently with
  no log, panic, or `Result::Err`. A consumer that forgets to
  call `.signing_secret(...)` ships with a publicly-readable
  HMAC key — every signed URL can be minted by anyone who has
  read the crate's source.
- **expected:** Engine rule: `docs/code-standards.md` § "Type
  Safety" — secrets must not silently fall through to a
  default; `AGENTS.md` § "Code Standards" — "Production-ready.
  Real schools, real students, real money."
- **evidence:**
  ```rust
  const DEFAULT_SIGNING_SECRET: &[u8] =
      b"educore-local-file-storage-default-signing-secret-do-not-use-in-prod";
  ```
  at `crates/adapters/files/src/local.rs:134-135`. The
  builder's `build()` (line 301-324) does not compare
  `self.signing_secret` against `DEFAULT_SIGNING_SECRET`
  before constructing `LocalFileStorage`.

---

### FINDING 11

- **id:** ADAPTER-FILE-011
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/files/src/local.rs:473-502,
  46-47
- **description:** `LocalFileStorage::signed_url` emits a
  `file://` URL pointing to a local filesystem path. There
  is no fetch endpoint, no middleware that re-validates the
  token, and no client surface that consumes a `file://`
  URL. Anyone with shell access to the host can `cat` the
  file directly, bypassing the HMAC token entirely. The
  signed URL provides no security boundary — it is a stub.
- **expected:** `docs/ports/file-storage.md:99-101` — "The
  adapter uses the storage provider's signing mechanism (e.g.
  S3 presigned URLs, GCS signed URLs, local token URLs)." An
  `https://` URL backed by a fetch endpoint that verifies the
  token is implied by "signed URL".
- **evidence:**
  ```rust
  let mut url = format!(
      "file://{}?expires_in={expires_in}&method={method}&token={token}",
      path.display(),
  );
  ```
  at `crates/adapters/files/src/local.rs:489-491`.

---

### FINDING 12

- **id:** ADAPTER-FILE-012
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/files/src/s3.rs:448-499
- **description:** `S3FileStorage::signed_url` does not
  consult `reference.visibility` at all. It will mint a
  presigned URL for any `Visibility::Private` file
  regardless of who is asking. Contrast the local adapter
  (local.rs:481) which short-circuits
  `Visibility::Public && method==Get` to skip the token. The
  S3 implementation also does not accept a per-call
  `actor_id`/`tenant` parameter (the trait signature is
  fixed at port.rs:710-714), so the adapter has no way to
  check whether the caller is authorised for the requested
  URL — it relies entirely on S3's signing model.
- **expected:** `crates/adapters/files/src/port.rs:708-709`:
  "MUST reject requests on objects whose visibility does not
  permit the requested method."
- **evidence:** `crates/adapters/files/src/s3.rs:448-499`
  reads `reference` only for `reference.key` (line 453);
  `visibility`, `tenant`, `uploaded_by` are never inspected.

---

### FINDING 13

- **id:** ADAPTER-FILE-013
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/files/src/s3.rs:570-578,
  crates/adapters/files/src/port.rs:723-724
- **description:** `S3FileStorage::move_to` is implemented
  as `self.copy(src, dst_key).await?; self.delete(src).await?;`
  rather than S3's atomic `POST /<bucket>/<dst>?x-id=CopySource`
  followed by `DELETE /<bucket>/<src>` in a single request,
  or S3 Multi-Object Delete. If the copy succeeds and the
  delete fails (network blip, throttling, IAM revocation
  mid-call), the source object persists and the engine has
  returned a `FileReference` for the destination that points
  at content still also living at the source path — orphan
  files plus duplicate storage charges. The port doc
  explicitly says `move_to` "**Atomically** renames the
  object".
- **expected:** `crates/adapters/files/src/port.rs:723-724` —
  "Atomically renames the object to a new key inside the same
  tenant."
- **evidence:**
  ```rust
  async fn move_to(
      &self,
      src: &FileReference,
      dst_key: &str,
  ) -> StdResult<FileReference, FileStorageError> {
      let dst = self.copy(src, dst_key).await?;
      self.delete(src).await?;
      Ok(dst)
  }
  ```
  at `crates/adapters/files/src/s3.rs:570-578`.

---

### FINDING 14

- **id:** ADAPTER-FILE-014
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/files/src/local.rs:174-212,
  384-421, 423-435
- **description:** `LocalFileStorage::resolve` performs only
  **lexical** path validation; it never `canonicalize`s the
  destination against `self.root`. The module-level doc at
  lines 193-198 acknowledges the gap: "We can't `canonicalize`
  the full path because the file may not exist yet (for
  `put`); the lexical checks above are the authoritative guard
  against `..` escapes." If an attacker can place a symlink at
  `root/key_prefix/<legit_key>` pointing to `/etc/passwd` or
  `/home/teacher/.ssh/id_rsa`, the `get`/`exists`/`head`/`copy`
  paths will follow the symlink and read or copy arbitrary host
  content. The local adapter is unsuitable for any
  multi-tenant deployment that does not already operate inside
  a hardened namespace.
- **expected:** Defensive practice; the port contract at
  `docs/ports/file-storage.md:48-50` says "the consumer's
  adapter enforces a safe key namespace", which implies
  filesystem symlinks should not bypass the boundary.
- **evidence:** No `symlink_metadata`, `canonicalize`, or
  `read_link` call exists anywhere in
  `crates/adapters/files/src/local.rs` (the only matches for
  `canonicalize` at lines 193 and 604 are inside
  doc-comments).

---

### FINDING 15

- **id:** ADAPTER-FILE-015
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/files/src/port.rs:634,
  117-124, 408-413
- **description:** `FileStream` is typed as
  `tokio::sync::mpsc::Receiver<StdResult<Vec<u8>, std::io::Error>>`
  (port.rs:634), so streaming errors surface as
  `std::io::Error` rather than `FileStorageError`. The spec
  types the stream as
  `Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>`
  (`docs/ports/file-storage.md:122-124`) — i.e. the port's own
  `Result<T, FileStorageError>`. A consumer of the trait
  cannot match against `FileStorageError::NotFound` /
  `ChecksumMismatch` on the streaming path because the error
  variant has been flattened to `io::Error`. The local adapter
  surfaces file-open failures as `io::Error` (local.rs:389-397),
  not as `FileStorageError::NotFound` on the stream itself.
- **expected:** `docs/ports/file-storage.md:122-124` — stream
  items wrapped in the port's `Result` type.
- **evidence:**
  ```rust
  pub type FileStream = tokio::sync::mpsc::Receiver<StdResult<Vec<u8>, std::io::Error>>;
  ```
  at `crates/adapters/files/src/port.rs:634`.

---

### FINDING 16

- **id:** ADAPTER-FILE-016
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/files/src/port.rs:63-64,
  crates/adapters/files/src/lib.rs:41
- **description:** `port.rs` sets `#![allow(missing_docs)]` at
  module level (line 64), which silently overrides the
  crate-root `#![deny(missing_docs)]` declared in `lib.rs:41`.
  The `port.rs` module is the largest public surface in the
  crate (the `FileStorage` trait, `PutRequest`, `FileReference`,
  `FileMetadata`, `Visibility`, `StorageClass`,
  `SignedUrlMethod`, `SignedUrlOptions`, `FileKey`,
  `ContentType`, `Checksum`, `IdempotencyKey`, `FileStream`);
  the engine rule requires every public item to carry rustdoc.
  With the allow, a future contributor can remove a
  doc-comment from any of these types without the deny
  firing.
- **expected:** `AGENTS.md` § "Code Standards" — "All public
  APIs are documented with rustdoc; `#![deny(missing_docs)]`."
- **evidence:**
  ```rust
  #![allow(dead_code, clippy::all)]
  #![allow(missing_docs)]
  ```
  at `crates/adapters/files/src/port.rs:63-64`, preceded by
  the crate-root `#![deny(missing_docs)]` at
  `crates/adapters/files/src/lib.rs:41`.

---

### FINDING 17

- **id:** ADAPTER-FILE-017
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/files/tests/files_integration.rs,
  docs/ports/file-storage.md:193-202
- **description:** The integration test file
  (`crates/adapters/files/tests/files_integration.rs`) ships
  5 sync tests + 2 env-gated tests. The env-gated tests
  (`files_integration_async_s3_put_mock` at line 159 and
  `files_integration_async_local_put_mock` at line 168) call
  `.build()` on the respective builders and discard the
  result — they do not exercise `put`, `get`, `delete`,
  `exists`, `head`, `signed_url`, `copy`, or `move_to`
  against any real or fake backend. The port contract lists
  7 categories of required integration tests; only 5
  (SHA-256, ETag, key namespace round-trip, visibility
  classification, signed URL build+verify) are present,
  missing:
  - "Integration tests of signed URL generation and
    **expiration**" (no expiry assertion exists)
  - "A test of **cross-tenant denial**" (no actor/school
    check)
  - "A test of **checksum mismatch**" (no read-side
    recompute)
  - "A test of **content type validation**" (no allow-list
    test)
  - "A test of **large file streaming**" (no multi-MB
    stream)
  - "A test of **idempotent retry**" (no second-put
    assertion)
- **expected:** `docs/ports/file-storage.md:193-202` — the 7
  test categories above.
- **evidence:** `crates/adapters/files/tests/files_integration.rs:157-172`
  contains only builder-construction bodies (e.g. line 160-163
  builds an `S3FileStorage` and binds it to `_storage`, never
  calling `put`).

---

### FINDING 18

- **id:** ADAPTER-FILE-018
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/files/Cargo.toml:8,
  crates/adapters/files/src/lib.rs:3
- **description:** `Cargo.toml` description claims "File
  storage port, **S3-compatible, GCS**, local filesystem
  adapters" and the lib.rs docstring mirrors it: "File
  storage port, S3-compatible, **GCS**, local filesystem
  adapters." No GCS module, no GCS builder, no GCS client
  dep (`grep -rn 'gcp\|google.cloud\|gcs'` on
  `crates/adapters/files/` returns zero source hits; only
  doc references to GCS as a "future" alternative). The
  crate description advertises a feature that does not exist;
  downstream consumers reading the manifest will believe GCS
  is supported.
- **expected:** Accurate crate metadata; engine rule
  `AGENTS.md` § "Naming Convention" requires exact advertised
  surface area.
- **evidence:** `crates/adapters/files/Cargo.toml:8` —
  `description = "File storage port, S3-compatible, GCS, local
  filesystem adapters."` Only `pub mod s3;` (s3.rs) and
  `pub mod local;` (local.rs) exist.

---

### FINDING 19

- **id:** ADAPTER-FILE-019
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/files/src/local.rs:78-81,
  343-382
- **description:** `LocalFileStorage::put` ignores
  `PutRequest::overwrite` and always writes through. The
  module doc at lines 78-81 acknowledges the gap:
  "**`overwrite = false` is not enforced.** The local adapter
  always overwrites; the spec leaves the precise error to the
  adapter and a real S3 adapter would surface a
  `PreconditionFailed`." A consumer that uploads a student
  photo with `overwrite = false` to
  `students/photos/ada.jpg` will silently replace an existing
  photo, breaking the "preserve old version" contract implied
  by the `PutRequest::overwrite` field.
- **expected:** `crates/adapters/files/src/port.rs:501-507` —
  "`true` to overwrite an existing object at the same key;
  `false` to return an error if the key is already in use."
- **evidence:** `crates/adapters/files/src/local.rs:343-365`
  calls `tokio::fs::write(&path, &request.content)` without
  inspecting `request.overwrite` or calling `exists` first.

---

### FINDING 20

- **id:** ADAPTER-FILE-020
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/files/src/s3.rs:297
- **description:** `S3FileStorage::put` performs
  `content_len as u64` to coerce `usize` into
  `FileReference.size` (`u64`). On a 32-bit platform where
  `usize == u32`, this is an `as`-cast that the engine rule
  forbids (lossy on byte counts above `u32::MAX` only in
  theory, but the engine code standard forbids ALL `as` on
  numerics — `TryFrom`/`TryInto` is required). The S3
  `content_length` setter already does the correct conversion
  on line 266 via
  `i64::try_from(content_len).unwrap_or(i64::MAX)` — the
  returned-reference field should match.
- **expected:** `AGENTS.md` § "Type Safety" — "No `as` casts
  that truncate or lose data. Use `TryFrom` / `TryInto` with
  proper error handling."
- **evidence:** `size: content_len as u64,` at
  `crates/adapters/files/src/s3.rs:297`.

---

### FINDING 21

- **id:** ADAPTER-FILE-021
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/files/src/port.rs:710-714,
  docs/ports/file-storage.md:103-115
- **description:** `FileStorage::signed_url` returns
  `Result<String>` (port.rs:714) instead of
  `Result<url::Url>`. The port contract types the return as
  `Result<Url>` (`docs/ports/file-storage.md:20`). Adapters
  must hand-roll URL string assembly (local.rs:489-501;
  s3.rs:498), and downstream consumers cannot use the
  standard `Url` API (`url::Url::parse`) to parse or join
  against the returned URL. The crate's deviation note at
  port.rs:33-37 acknowledges this but the port surface itself
  was the spec target.
- **expected:** `docs/ports/file-storage.md:20` — `async fn
  signed_url(...) -> Result<Url>`.
- **evidence:**
  ```rust
  async fn signed_url(
      &self,
      reference: &FileReference,
      options: SignedUrlOptions,
  ) -> Result<String>;
  ```
  at `crates/adapters/files/src/port.rs:710-714`.

---

### FINDING 22

- **id:** ADAPTER-FILE-022
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/files/src/local.rs:174-212,
  343-382, docs/ports/file-storage.md:126-129
- **description:** `LocalFileStorage::resolve` does not
  reject empty keys or keys containing null bytes; both
  surface as filesystem errors (`NotADirectory`,
  `InvalidInput`) wrapped in
  `FileStorageError::Infrastructure`, not the more specific
  `FileStorageError::InvalidKey` that the spec reserves for
  malformed inputs. S3 has no such guard either — empty keys
  become empty object names, null bytes are forwarded verbatim
  to AWS (which rejects them with a 400 wrapped as
  `Infrastructure`). The port contract's `InvalidKey(String)`
  variant implies a key-validation step neither adapter
  performs.
- **expected:** `docs/ports/file-storage.md:151` and
  `crates/adapters/files/src/port.rs:152-158` — "adapters
  that need validation (length, character set, reserved
  prefix) perform it inside the `FileStorage::put`
  implementation and return `FileStorageError::InvalidKey` on
  a malformed input."
- **evidence:** No length / null-byte / character-set check
  exists in `crates/adapters/files/src/local.rs:174-212` or
  `crates/adapters/files/src/s3.rs:241-306`.

---

### FINDING 23

- **id:** ADAPTER-FILE-023
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/files/src/s3.rs:308-359,
  crates/adapters/files/src/local.rs:384-421,
  docs/ports/file-storage.md:286-311
- **description:** Neither `S3FileStorage::get` nor
  `LocalFileStorage::get` enforces `reference.visibility`. A
  `Visibility::Private` file uploaded at school A is fetchable
  by anyone who holds a valid `FileReference` for it,
  regardless of the requesting user's role or school. The
  port contract at file-storage.md:42-47 defines
  `Visibility::Private` / `Public` / `TenantPrivate`, and
  the port trait doc at port.rs:286-294 requires "a file
  uploaded as `Visibility::Private` must require a signed URL
  on every read." The `Visibility::Private` variant in
  `FileStorageError::PermissionDenied` (errors.rs:62-65) is
  documented for "cross-tenant attempts, expired signed URLs,
  and unauthorised `Visibility::Private` reads" but is never
  constructed anywhere in either adapter.
- **expected:** `crates/adapters/files/src/port.rs:708-709`:
  "MUST reject requests on objects whose visibility does not
  permit the requested method."
- **evidence:** No `PermissionDenied` constructor call exists
  anywhere in `crates/adapters/files/src/s3.rs` or
  `crates/adapters/files/src/local.rs` (only in
  `crates/adapters/files/src/errors.rs:252` test).

---

### FINDING 24

- **id:** ADAPTER-FILE-024
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/files/src/s3.rs:448-499,
  crates/adapters/files/src/port.rs:710-714
- **description:** `FileStorage::signed_url` has no
  per-caller / per-actor parameter; the trait signature is
  `(&self, reference, options) -> Result<String>`. The S3
  adapter therefore has no way to enforce that the actor
  requesting the URL is a member of
  `reference.tenant.school_id`, has the `FilesSignedUrl`
  capability, or even that the requester is different from
  the uploader. Any code that holds a `FileReference`
  (e.g. a student viewing their own report card) can mint
  an admin-grade PUT presigned URL and overwrite the source
  object. Visibility is the only gate and Finding 23
  documents that it is not enforced on reads either.
- **expected:** Per the engine's RBAC contract
  (`AGENTS.md` § "Multi-tenant by default. Every aggregate
  has a `SchoolId`") and the port's
  `Capability::FilesSignedUrl` capability
  (PHASE-15-HANDOFF.md:191-193), a `signed_url` call should
  accept or be paired with an actor context that the adapter
  can authorise against.
- **evidence:** `crates/adapters/files/src/port.rs:710-714` —
  the trait signature carries no `actor` or `tenant_context`
  parameter, and `s3.rs:448-499` only reads `reference.key`
  from the supplied `reference`.

---

### FINDING 25

- **id:** ADAPTER-FILE-025
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/files/src/s3.rs:84-99, 308-359
- **description:** `S3FileStorage::get` issues no `If-Match`
  or `If-None-Match` precondition against `reference.etag`.
  If a racing upload overwrites the object between the
  consumer's fetch and the consumer's checksum check (which
  Finding 7 documents is not done by the adapter), the
  engine will consume bytes that do not match the
  `FileReference` it holds. Combined with Finding 7 (no
  read-side checksum recompute), the S3 adapter cannot
  detect an in-flight overwrite at all.
- **expected:** RFC 7232 / S3 `If-Match` precondition; an
  upload path that captures `reference.etag` should be
  paired with a download path that requires it.
- **evidence:** `crates/adapters/files/src/s3.rs:312-319`
  builds the `get_object` request with only `.bucket(...)`
  and `.key(...)`; no `.if_match(...)` or equivalent call
  exists.

---

### FINDING 26

- **id:** ADAPTER-FILE-026
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/files/src/s3.rs:501-568,
  docs/ports/file-storage.md:126-129
- **description:** S3 versioning is not enabled and not
  exercised in `S3FileStorage`. The port contract at
  `docs/ports/file-storage.md:126-129` requires: "If the
  underlying provider supports versioning (S3 does), the
  adapter enables it. Older versions are retained for a
  configurable period." `S3FileStorage::copy`
  (s3.rs:501-568) overwrites the destination with
  `copy_object` semantics, not a versioned-copy, and
  `S3FileStorage::put` (line 270-276) uses
  `If-None-Match: *` rather than enabling bucket versioning.
  The `VersioningConfiguration` S3 API is never invoked.
- **expected:** `docs/ports/file-storage.md:126-129` — "If
  the underlying provider supports versioning (S3 does), the
  adapter enables it."
- **evidence:** No `versioning`, `VersioningConfiguration`,
  `enable_versioning`, or `version_id` reference exists in
  `crates/adapters/files/src/s3.rs`.

---

### FINDING 27

- **id:** ADAPTER-FILE-027
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/files/src/local.rs:400-418,
  crates/adapters/files/src/s3.rs:330-356
- **description:** Both `get` implementations spawn a
  `tokio::spawn(async move { ... })` to drain the upstream
  into the mpsc channel, but neither captures the
  `JoinHandle` nor surfaces a panic from the spawned task.
  If the task panics (e.g. a `tokio::fs::File` invariant
  violation, an S3 SDK mid-stream decode error that the
  worker surfaces as a panic), the channel closes silently
  with `None` and the engine sees a truncated file as a
  clean EOF. There is no way for the engine's audit or
  observability layer to distinguish "successful end of
  stream" from "spawned task panicked".
- **expected:** Engine rule `AGENTS.md` § "Type Safety" —
  "No `unwrap`/`expect`/`panic` in production paths", and
  the port contract at
  `crates/adapters/files/src/port.rs:614-621` ("Adapters MUST
  yield chunks promptly and MUST NOT buffer the entire object
  before sending the first chunk") — both presume a sound
  task lifetime.
- **evidence:** `crates/adapters/files/src/local.rs:401-418`
  spawns the task without storing the handle, and
  `crates/adapters/files/src/s3.rs:330-356` does the same.

---

### FINDING 28

- **id:** ADAPTER-FILE-028
- **area:** adapters
- **severity:** Low
- **location:** crates/adapters/files/src/local.rs:301-324,
  crates/adapters/files/src/local.rs:258-296
- **description:** `LocalFileStorageBuilder::build()` does
  not validate that `key_prefix` is a safe lexical string. A
  consumer that sets `.key_prefix("/etc/")` or
  `.key_prefix("../escape/")` causes
  `LocalFileStorage::resolve` to compose `root + "/etc/" +
  key` or `root + "../escape/" + key`; the post-normalisation
  prefix check at line 203-209 rejects the second case
  (because the normalised path does not start with the
  normalised root) but accepts the first case (because
  `/etc/` is a child of root if root is `/`). The adapter
  offers no documented whitelist for `key_prefix` and no
  `must_use` warning.
- **expected:** Defensive builder validation; the port
  contract at `crates/adapters/files/src/port.rs:152-158`
  ("adapters that need validation (length, character set,
  reserved prefix) perform it inside the `FileStorage::put`
  implementation") implies the builder's `key_prefix` setter
  should reject obviously-unsafe values.
- **evidence:** `crates/adapters/files/src/local.rs:301-324`
  (`build`) does not call `validate` on `self.key_prefix`;
  the only path-safety check happens per-call inside
  `resolve` (line 174-212) and only on the **key**, not on
  the `key_prefix` itself.

### END FINDINGS
