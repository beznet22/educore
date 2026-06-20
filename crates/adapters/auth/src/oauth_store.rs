//! # In-memory OAuth store reference implementation
//!
//! Reference adapter implementing the four port-driven repository
//! traits declared in
//! [`educore_operations::repository`](educore_operations::repository):
//!
//! - [`OAuthAccessTokenRepository`] — `oauth_access_tokens` table.
//! - [`OAuthClientRepository`] — `oauth_clients` table.
//! - [`PasswordResetRepository`] — `password_resets` table.
//! - [`MigrationRepository`] — `migrations` table.
//!
//! These four traits are **port-driven**: the operations domain
//! documents them for completeness, but the engine does not own
//! their lifecycle. Consumer auth / migration adapters implement
//! them; this file is the in-memory reference implementation used
//! by tests and the worked example. The SQL adapters (Postgres,
//! MySQL, SQLite) ship separate implementations that translate the
//! same trait surface into dialect-specific DML.
//!
//! The store holds four `Arc<Mutex<HashMap<...>>>` collections
//! behind a single [`InMemoryOAuthStore`] value. The `Mutex` is a
//! `std::sync::Mutex` rather than `tokio::sync::Mutex` because the
//! critical sections are O(1)-or-O(n)-over-a-small-map and never
//! await; the lock is held for microseconds. The store is therefore
//! `Send + Sync` and can be wrapped in `Arc<dyn Trait>` for object
//! dispatch.
//!
//! ## Audit integration
//!
//! Every state-changing method maps its operation to a typed
//! [`educore_audit::AuditAction`] via the private
//! [`audit_action_for_op`] helper. The mapping is exposed as a
//! free function so future subagent work (the command handler
//! that wires the full auth flow) can pass the returned action to
//! [`educore_audit::AuditWriter::write`] without re-deriving the
//! verb. The reference store itself does not write audit rows —
//! audit emission belongs to the command handler, not the
//! repository port.

#![allow(clippy::expect_used)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use async_trait::async_trait;
use uuid::Uuid;

use educore_audit::prelude::AuditAction;
use educore_core::error::Result as StorageResult;
use educore_core::value_objects::Timestamp;
use educore_operations::repository::{
    Migration, MigrationRepository, OAuthAccessToken, OAuthAccessTokenRepository, OAuthClient,
    OAuthClientRepository, PasswordReset, PasswordResetRepository,
};

/// In-memory implementation of the four port-driven OAuth-related
/// repository traits declared in
/// [`educore_operations::repository`](educore_operations::repository).
///
/// Construct one per process and share via `Arc<dyn Trait>` for
/// object dispatch (each of the four traits is object-safe; the
/// `_assert_*_object_safe` helpers in
/// `educore_operations::repository` prove it).
#[derive(Debug, Default)]
pub struct InMemoryOAuthStore {
    access_tokens: Arc<Mutex<HashMap<String, OAuthAccessToken>>>,
    oauth_clients: Arc<Mutex<HashMap<String, OAuthClient>>>,
    password_resets: Arc<Mutex<HashMap<String, PasswordReset>>>,
    migrations: Arc<Mutex<HashMap<String, Migration>>>,
}

impl InMemoryOAuthStore {
    /// Constructs a new, empty `InMemoryOAuthStore`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Acquires the access-tokens mutex, recovering from poisoning.
    fn lock_tokens(&self) -> std::sync::MutexGuard<'_, HashMap<String, OAuthAccessToken>> {
        self.access_tokens
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    /// Acquires the oauth-clients mutex, recovering from poisoning.
    fn lock_clients(&self) -> std::sync::MutexGuard<'_, HashMap<String, OAuthClient>> {
        self.oauth_clients
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    /// Acquires the password-resets mutex, recovering from poisoning.
    fn lock_resets(&self) -> std::sync::MutexGuard<'_, HashMap<String, PasswordReset>> {
        self.password_resets
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    /// Acquires the migrations mutex, recovering from poisoning.
    fn lock_migrations(&self) -> std::sync::MutexGuard<'_, HashMap<String, Migration>> {
        self.migrations
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

/// Maps a port-driven operation to the canonical
/// [`educore_audit::AuditAction`] verb the command handler should
/// pass to [`educore_audit::AuditWriter::write`].
fn audit_action_for_op(op: &str) -> AuditAction {
    match op {
        "oauth_access_token.insert"
        | "oauth_client.insert"
        | "password_reset.insert"
        | "migration.insert" => AuditAction::Create,
        "oauth_access_token.revoke" | "oauth_client.revoke" | "password_reset.delete" => {
            AuditAction::Delete
        }
        "oauth_access_token.purge" | "password_reset.purge" => AuditAction::Other(op.to_owned()),
        _ => AuditAction::Other(op.to_owned()),
    }
}

#[async_trait]
impl OAuthAccessTokenRepository for InMemoryOAuthStore {
    async fn get(&self, id: &str) -> StorageResult<Option<OAuthAccessToken>> {
        Ok(self.lock_tokens().get(id).cloned())
    }

    async fn list_for_user(&self, user_id: Uuid) -> StorageResult<Vec<OAuthAccessToken>> {
        Ok(self
            .lock_tokens()
            .values()
            .filter(|t| t.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn insert(&self, t: &OAuthAccessToken) -> StorageResult<()> {
        let _action = audit_action_for_op("oauth_access_token.insert");
        self.lock_tokens().insert(t.id.clone(), t.clone());
        Ok(())
    }

    async fn revoke(&self, id: &str) -> StorageResult<()> {
        let _action = audit_action_for_op("oauth_access_token.revoke");
        if let Some(t) = self.lock_tokens().get_mut(id) {
            t.revoked = true;
        }
        Ok(())
    }

    async fn purge_expired(&self, before: Timestamp) -> StorageResult<u64> {
        let _action = audit_action_for_op("oauth_access_token.purge");
        let mut guard = self.lock_tokens();
        let initial = guard.len();
        guard.retain(|_, t| t.expires_at.map_or(true, |exp| exp >= before));
        Ok(u64::try_from(initial - guard.len()).unwrap_or(0))
    }
}

#[async_trait]
impl OAuthClientRepository for InMemoryOAuthStore {
    async fn get(&self, id: &str) -> StorageResult<Option<OAuthClient>> {
        Ok(self.lock_clients().get(id).cloned())
    }

    async fn list(&self) -> StorageResult<Vec<OAuthClient>> {
        Ok(self.lock_clients().values().cloned().collect())
    }

    async fn insert(&self, c: &OAuthClient) -> StorageResult<()> {
        let _action = audit_action_for_op("oauth_client.insert");
        self.lock_clients().insert(c.id.clone(), c.clone());
        Ok(())
    }

    async fn revoke(&self, id: &str) -> StorageResult<()> {
        let _action = audit_action_for_op("oauth_client.revoke");
        if let Some(c) = self.lock_clients().get_mut(id) {
            c.revoked = true;
        }
        Ok(())
    }
}

#[async_trait]
impl PasswordResetRepository for InMemoryOAuthStore {
    async fn get_by_email(&self, email: &str) -> StorageResult<Option<PasswordReset>> {
        Ok(self.lock_resets().get(email).cloned())
    }

    async fn insert(&self, r: &PasswordReset) -> StorageResult<()> {
        let _action = audit_action_for_op("password_reset.insert");
        self.lock_resets().insert(r.email.clone(), r.clone());
        Ok(())
    }

    async fn delete(&self, email: &str) -> StorageResult<()> {
        let _action = audit_action_for_op("password_reset.delete");
        self.lock_resets().remove(email);
        Ok(())
    }

    async fn purge_older_than(&self, before: Timestamp) -> StorageResult<u64> {
        let _action = audit_action_for_op("password_reset.purge");
        let mut guard = self.lock_resets();
        let initial = guard.len();
        guard.retain(|_, r| r.expires_at.map_or(true, |exp| exp >= before));
        Ok(u64::try_from(initial - guard.len()).unwrap_or(0))
    }
}

#[async_trait]
impl MigrationRepository for InMemoryOAuthStore {
    async fn list(&self) -> StorageResult<Vec<Migration>> {
        Ok(self.lock_migrations().values().cloned().collect())
    }

    async fn get_by_name(&self, name: &str) -> StorageResult<Option<Migration>> {
        Ok(self.lock_migrations().get(name).cloned())
    }

    async fn insert(&self, m: &Migration, _batch: i32) -> StorageResult<()> {
        let _action = audit_action_for_op("migration.insert");
        self.lock_migrations()
            .insert(m.migration.clone(), m.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_token(id: &str, user: Uuid) -> OAuthAccessToken {
        OAuthAccessToken {
            id: id.to_owned(),
            user_id: user,
            client_id: "client-1".to_owned(),
            scopes: "read".to_owned(),
            revoked: false,
            expires_at: None,
            created_at: Timestamp::now(),
        }
    }

    fn sample_client(id: &str) -> OAuthClient {
        OAuthClient {
            id: id.to_owned(),
            name: format!("client-{id}"),
            secret_hash: "hash".to_owned(),
            redirect_uri: "https://example.com/callback".to_owned(),
            provider: None,
            revoked: false,
            created_at: Timestamp::now(),
        }
    }

    fn sample_reset(email: &str) -> PasswordReset {
        PasswordReset {
            email: email.to_owned(),
            token_hash: "h".to_owned(),
            created_at: Timestamp::now(),
            expires_at: None,
        }
    }

    #[tokio::test]
    async fn test_oauth_access_token_insert_and_get() {
        let store = InMemoryOAuthStore::new();
        let user = Uuid::new_v4();
        let t = sample_token("t1", user);

        OAuthAccessTokenRepository::insert(&store, &t)
            .await
            .unwrap();

        let got = OAuthAccessTokenRepository::get(&store, "t1").await.unwrap();
        assert_eq!(got, Some(t));

        let for_user = OAuthAccessTokenRepository::list_for_user(&store, user)
            .await
            .unwrap();
        assert_eq!(for_user.len(), 1);

        let other = OAuthAccessTokenRepository::list_for_user(&store, Uuid::new_v4())
            .await
            .unwrap();
        assert!(other.is_empty());
    }

    #[tokio::test]
    async fn test_oauth_client_list_and_revoke() {
        let store = InMemoryOAuthStore::new();
        let c1 = sample_client("c1");
        let c2 = sample_client("c2");

        OAuthClientRepository::insert(&store, &c1).await.unwrap();
        OAuthClientRepository::insert(&store, &c2).await.unwrap();

        let listed = OAuthClientRepository::list(&store).await.unwrap();
        assert_eq!(listed.len(), 2);

        OAuthClientRepository::revoke(&store, "c1").await.unwrap();
        let after = OAuthClientRepository::get(&store, "c1").await.unwrap().unwrap();
        assert!(after.revoked);
        let other = OAuthClientRepository::get(&store, "c2").await.unwrap().unwrap();
        assert!(!other.revoked);
    }

    #[tokio::test]
    async fn test_password_reset_insert_and_delete() {
        let store = InMemoryOAuthStore::new();
        let r = sample_reset("a@b.com");

        PasswordResetRepository::insert(&store, &r).await.unwrap();
        let got = PasswordResetRepository::get_by_email(&store, "a@b.com")
            .await
            .unwrap();
        assert_eq!(got, Some(r));

        PasswordResetRepository::delete(&store, "a@b.com").await.unwrap();
        let after = PasswordResetRepository::get_by_email(&store, "a@b.com")
            .await
            .unwrap();
        assert!(after.is_none());
    }

    #[tokio::test]
    async fn test_migration_insert_and_list() {
        let store = InMemoryOAuthStore::new();
        let m1 = Migration {
            migration: "001_init".to_owned(),
            batch: 1,
        };
        let m2 = Migration {
            migration: "002_index".to_owned(),
            batch: 1,
        };

        MigrationRepository::insert(&store, &m1, 1).await.unwrap();
        MigrationRepository::insert(&store, &m2, 1).await.unwrap();

        let got = MigrationRepository::get_by_name(&store, "001_init")
            .await
            .unwrap();
        assert_eq!(got, Some(m1));

        let list = MigrationRepository::list(&store).await.unwrap();
        assert_eq!(list.len(), 2);

        let unknown = MigrationRepository::get_by_name(&store, "999_missing")
            .await
            .unwrap();
        assert!(unknown.is_none());
    }
}