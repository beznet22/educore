//! In-memory `FileStorage` adapter.
//!
//! Implements [`FileStorage`] against an in-process
//! `parking_lot::Mutex<HashMap<...>>` so consumer tests can
//! exercise the engine's file-handling code paths without a real
//! object store (S3 / GCS / Azure Blob / local filesystem).
//!
//! # Idempotency
//!
//! `put` is idempotent on [`PutRequest::idempotency_key`]. A
//! retry with the same token returns the original
//! [`FileReference`] without re-uploading; this matches the port
//! contract in `docs/ports/file-storage.md` § "Idempotency".
//!
//! # Checksum
//!
//! The spec requires a content-addressable SHA-256 hex digest.
//! The in-memory adapter uses a length-based hex placeholder
//! (`format!("{:x}", content.len())`) so the testkit does not
//! need to take on a SHA-256 crate dependency. Tests that need a
//! real content-addressable hash should exercise the
//! `LocalFileStorage` or `S3FileStorage` reference
//! implementations instead.

use std::collections::HashMap;

use async_trait::async_trait;
use educore_core::value_objects::Timestamp;
use educore_files::port::{
    Checksum, ContentType, FileKey, FileMetadata, FileReference, FileStorage, FileStream,
    IdempotencyKey, PutRequest, SignedUrlOptions, StorageClass, Visibility,
};
use parking_lot::Mutex;
use tokio::sync::mpsc;

use educore_files::errors::{FileStorageError, InfrastructureError};

/// In-memory `FileStorage` backed by a `HashMap`.
#[derive(Debug, Default)]
pub struct InMemoryFileStorage {
    /// `FileKey` → `(FileReference, content bytes)`. The reference
    /// is stored alongside the bytes so `head`, `exists`, and
    /// `signed_url` can answer without re-deriving metadata.
    store: Mutex<HashMap<FileKey, (FileReference, Vec<u8>)>>,
    /// `IdempotencyKey` → `FileKey`. A retry of `put` with the
    /// same idempotency token returns the stored reference at
    /// the mapped key without re-uploading.
    idempotency_keys: Mutex<HashMap<IdempotencyKey, FileKey>>,
}

impl InMemoryFileStorage {
    /// Constructs a fresh, empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl FileStorage for InMemoryFileStorage {
    async fn put(
        &self,
        request: PutRequest,
    ) -> educore_files::port::Result<FileReference> {
        // Idempotency: if this token has already been used, return
        // the original reference verbatim.
        if let Some(idempotency_key) = request.idempotency_key.as_ref() {
            let idem = self.idempotency_keys.lock();
            if let Some(existing_key) = idem.get(idempotency_key).cloned() {
                drop(idem);
                let store = self.store.lock();
                if let Some((existing_ref, _)) = store.get(&existing_key) {
                    return Ok(existing_ref.clone());
                }
            }
        }

        // Overwrite guard: if the key is already in use and the
        // caller did not opt into overwrite, surface an
        // infrastructure error per the port spec.
        {
            let store = self.store.lock();
            if !request.overwrite && store.contains_key(&request.key) {
                return Err(FileStorageError::Infrastructure(InfrastructureError::new(
                    format!(
                        "key {:?} already exists and overwrite=false",
                        request.key.as_str()
                    ),
                )));
            }
        }

        // Length-based hex placeholder. See the module-level doc
        // for the rationale.
        let checksum = format!("{:x}", request.content.len());
        let now = Timestamp::now();

        let reference = FileReference {
            key: request.key.clone(),
            etag: format!("\"{checksum}\""),
            size: u64::try_from(request.content.len()).unwrap_or(u64::MAX),
            content_type: request.content_type.clone(),
            visibility: request.visibility,
            uploaded_at: now,
            uploaded_by: request.tenant.actor_id,
            tenant: request.tenant.clone(),
            storage_class: StorageClass::Hot,
            checksum: Checksum::new(checksum),
        };

        let mut store = self.store.lock();
        store.insert(request.key.clone(), (reference.clone(), request.content));
        drop(store);

        if let Some(idempotency_key) = request.idempotency_key {
            let mut idem = self.idempotency_keys.lock();
            idem.insert(idempotency_key, request.key);
        }

        Ok(reference)
    }

    async fn get(&self, reference: &FileReference) -> educore_files::port::Result<FileStream> {
        let bytes = {
            let store = self.store.lock();
            match store.get(&reference.key) {
                Some((_, bytes)) => bytes.clone(),
                None => return Err(FileStorageError::NotFound(reference.key.clone())),
            }
        };

        let (tx, rx) = mpsc::channel::<std::result::Result<Vec<u8>, std::io::Error>>(8);
        tokio::spawn(async move {
            // Push the bytes in one chunk. The receiver
            // observes `Some(Ok(bytes))` and then channel close
            // (the sender drops at end of scope).
            let _ = tx.send(Ok(bytes)).await;
        });
        Ok(rx)
    }

    async fn delete(&self, reference: &FileReference) -> educore_files::port::Result<()> {
        self.store.lock().remove(&reference.key);
        Ok(())
    }

    async fn exists(&self, reference: &FileReference) -> educore_files::port::Result<bool> {
        Ok(self.store.lock().contains_key(&reference.key))
    }

    async fn head(&self, reference: &FileReference) -> educore_files::port::Result<FileMetadata> {
        let store = self.store.lock();
        match store.get(&reference.key) {
            Some((stored_ref, _)) => Ok(FileMetadata {
                key: stored_ref.key.clone(),
                etag: stored_ref.etag.clone(),
                size: stored_ref.size,
                content_type: stored_ref.content_type.clone(),
                uploaded_at: stored_ref.uploaded_at,
            }),
            None => Err(FileStorageError::NotFound(reference.key.clone())),
        }
    }

    async fn signed_url(
        &self,
        reference: &FileReference,
        options: SignedUrlOptions,
    ) -> educore_files::port::Result<String> {
        Ok(format!(
            "in-memory://signed/{}/{}?expires_in={}",
            reference.key.as_str(),
            options.method.as_str(),
            options.expires_in.as_secs()
        ))
    }

    async fn copy(
        &self,
        src: &FileReference,
        dst_key: &str,
    ) -> educore_files::port::Result<FileReference> {
        let bytes = {
            let store = self.store.lock();
            match store.get(&src.key) {
                Some((_, bytes)) => bytes.clone(),
                None => return Err(FileStorageError::NotFound(src.key.clone())),
            }
        };

        let new_key = FileKey::new(dst_key);
        let new_ref = FileReference {
            key: new_key.clone(),
            etag: src.etag.clone(),
            size: src.size,
            content_type: src.content_type.clone(),
            visibility: src.visibility,
            uploaded_at: Timestamp::now(),
            uploaded_by: src.tenant.actor_id,
            tenant: src.tenant.clone(),
            storage_class: src.storage_class,
            checksum: src.checksum.clone(),
        };

        self.store
            .lock()
            .insert(new_key, (new_ref.clone(), bytes));
        Ok(new_ref)
    }

    async fn move_to(
        &self,
        src: &FileReference,
        dst_key: &str,
    ) -> educore_files::port::Result<FileReference> {
        let new_ref = self.copy(src, dst_key).await?;
        self.delete(src).await?;
        Ok(new_ref)
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
    use educore_core::clock::{IdGenerator, SystemIdGen};
    use educore_core::tenant::{TenantContext, UserType};
    use educore_files::port::SignedUrlMethod;
    use std::time::Duration;

    fn ctx() -> TenantContext {
        let g = SystemIdGen;
        TenantContext::for_user(
            g.next_school_id(),
            g.next_user_id(),
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        )
    }

    fn put_request(key: &str, content: Vec<u8>) -> PutRequest {
        PutRequest {
            tenant: ctx(),
            key: FileKey::new(key),
            content,
            content_type: ContentType::new("application/octet-stream"),
            metadata: std::collections::BTreeMap::new(),
            visibility: Visibility::Private,
            overwrite: true,
            idempotency_key: None,
        }
    }

    async fn drain(rx: FileStream) -> Vec<std::result::Result<Vec<u8>, std::io::Error>> {
        let mut stream = rx;
        let mut out = Vec::new();
        while let Some(chunk) = stream.recv().await {
            out.push(chunk);
        }
        out
    }

    #[tokio::test]
    async fn put_then_get_returns_same_content() {
        let store = InMemoryFileStorage::new();
        let payload = b"hello world".to_vec();
        let reference = store.put(put_request("docs/hello.txt", payload.clone())).await.unwrap();

        let chunks = drain(store.get(&reference).await.unwrap()).await;
        let flat: Vec<u8> = chunks
            .into_iter()
            .map(|c| c.expect("chunk should be Ok"))
            .flatten()
            .collect();
        assert_eq!(flat, payload);
        assert_eq!(reference.size, u64::try_from(payload.len()).unwrap());
    }

    #[tokio::test]
    async fn put_with_idempotency_key_returns_same_reference() {
        let store = InMemoryFileStorage::new();
        let mut request = put_request("uploads/a.bin", b"alpha".to_vec());
        request.idempotency_key = Some(IdempotencyKey::new("upload-token-1"));

        let first = store.put(request.clone()).await.unwrap();
        let second = store.put(request).await.unwrap();

        assert_eq!(first, second);
        assert_eq!(first.size, 5);
    }

    #[tokio::test]
    async fn put_overwrite_true_replaces_existing() {
        let store = InMemoryFileStorage::new();
        let first = store
            .put(put_request("photos/p.png", b"first".to_vec()))
            .await
            .unwrap();
        let second = store
            .put(put_request("photos/p.png", b"second-content".to_vec()))
            .await
            .unwrap();
        assert_eq!(second.size, 14);
        assert_ne!(first.size, second.size);
        assert_ne!(first.etag, second.etag);
        // The new content is what comes back on read.
        let chunks = drain(store.get(&second).await.unwrap()).await;
        let flat: Vec<u8> = chunks
            .into_iter()
            .map(|c| c.expect("chunk should be Ok"))
            .flatten()
            .collect();
        assert_eq!(flat, b"second-content".to_vec());
    }

    #[tokio::test]
    async fn put_overwrite_false_returns_error_for_existing_key() {
        let store = InMemoryFileStorage::new();
        store
            .put(put_request("locked.bin", b"x".to_vec()))
            .await
            .unwrap();
        let mut request = put_request("locked.bin", b"y".to_vec());
        request.overwrite = false;
        let err = store.put(request).await.expect_err("must error");
        assert!(matches!(err, FileStorageError::Infrastructure(_)));
    }

    #[tokio::test]
    async fn delete_removes_file() {
        let store = InMemoryFileStorage::new();
        let reference = store
            .put(put_request("tmp/x.dat", b"to-be-deleted".to_vec()))
            .await
            .unwrap();

        store.delete(&reference).await.unwrap();
        let chunks = store.get(&reference).await;
        assert!(matches!(chunks, Err(FileStorageError::NotFound(_))));
    }

    #[tokio::test]
    async fn exists_returns_true_after_put_false_after_delete() {
        let store = InMemoryFileStorage::new();
        let reference = store
            .put(put_request("k.txt", b"k".to_vec()))
            .await
            .unwrap();
        assert!(store.exists(&reference).await.unwrap());
        store.delete(&reference).await.unwrap();
        assert!(!store.exists(&reference).await.unwrap());
    }

    #[tokio::test]
    async fn head_returns_metadata_for_existing_file() {
        let store = InMemoryFileStorage::new();
        let reference = store
            .put(put_request("meta/m.bin", b"payload".to_vec()))
            .await
            .unwrap();

        let meta = store.head(&reference).await.unwrap();
        assert_eq!(meta.key, reference.key);
        assert_eq!(meta.etag, reference.etag);
        assert_eq!(meta.size, reference.size);
        assert_eq!(meta.content_type, reference.content_type);
        assert_eq!(meta.uploaded_at, reference.uploaded_at);
    }

    #[tokio::test]
    async fn head_returns_not_found_for_missing_key() {
        let store = InMemoryFileStorage::new();
        let ghost = FileReference {
            key: FileKey::new("never/uploaded.bin"),
            etag: "\"0\"".to_owned(),
            size: 0,
            content_type: ContentType::new("application/octet-stream"),
            visibility: Visibility::Private,
            uploaded_at: Timestamp::now(),
            uploaded_by: SystemIdGen.next_user_id(),
            tenant: ctx(),
            storage_class: StorageClass::Hot,
            checksum: Checksum::new("0"),
        };
        let err = store.head(&ghost).await.expect_err("must error");
        assert!(matches!(err, FileStorageError::NotFound(_)));
    }

    #[tokio::test]
    async fn signed_url_contains_key_and_method() {
        let store = InMemoryFileStorage::new();
        let reference = store
            .put(put_request("photos/ada.jpg", b"jpg-bytes".to_vec()))
            .await
            .unwrap();

        let opts = SignedUrlOptions::new(Duration::from_secs(900), SignedUrlMethod::Get);
        let url = store.signed_url(&reference, opts).await.unwrap();
        assert!(url.contains(reference.key.as_str()), "url = {url}");
        assert!(url.contains("GET"), "url = {url}");
        assert!(url.contains("expires_in=900"), "url = {url}");

        let put_opts = SignedUrlOptions::new(Duration::from_secs(60), SignedUrlMethod::Put);
        let put_url = store.signed_url(&reference, put_opts).await.unwrap();
        assert!(put_url.contains("PUT"), "put_url = {put_url}");
        assert!(put_url.contains("expires_in=60"), "put_url = {put_url}");
    }

    #[tokio::test]
    async fn copy_then_original_still_exists() {
        let store = InMemoryFileStorage::new();
        let reference = store
            .put(put_request("orig.txt", b"keep-me".to_vec()))
            .await
            .unwrap();

        let copy_ref = store
            .copy(&reference, "dest/copied.txt")
            .await
            .unwrap();

        assert_eq!(copy_ref.key.as_str(), "dest/copied.txt");
        assert_eq!(copy_ref.size, reference.size);
        assert_eq!(copy_ref.checksum, reference.checksum);

        // Original still readable.
        assert!(store.exists(&reference).await.unwrap());
        let orig_chunks = drain(store.get(&reference).await.unwrap()).await;
        let orig_flat: Vec<u8> = orig_chunks
            .into_iter()
            .map(|c| c.expect("chunk should be Ok"))
            .flatten()
            .collect();
        assert_eq!(orig_flat, b"keep-me".to_vec());

        // Copy is also readable.
        let copy_chunks = drain(store.get(&copy_ref).await.unwrap()).await;
        let copy_flat: Vec<u8> = copy_chunks
            .into_iter()
            .map(|c| c.expect("chunk should be Ok"))
            .flatten()
            .collect();
        assert_eq!(copy_flat, b"keep-me".to_vec());
    }

    #[tokio::test]
    async fn move_to_then_original_does_not_exist() {
        let store = InMemoryFileStorage::new();
        let reference = store
            .put(put_request("to-move.bin", b"moving".to_vec()))
            .await
            .unwrap();

        let moved_ref = store
            .move_to(&reference, "archive/moved.bin")
            .await
            .unwrap();

        assert_eq!(moved_ref.key.as_str(), "archive/moved.bin");
        assert!(!store.exists(&reference).await.unwrap());
        assert!(store.exists(&moved_ref).await.unwrap());

        // Content moved intact.
        let moved_chunks = drain(store.get(&moved_ref).await.unwrap()).await;
        let moved_flat: Vec<u8> = moved_chunks
            .into_iter()
            .map(|c| c.expect("chunk should be Ok"))
            .flatten()
            .collect();
        assert_eq!(moved_flat, b"moving".to_vec());
    }
}