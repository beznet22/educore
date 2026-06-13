//! # Finance repository ports
//!
//! Phase 7 ships the 2 `#[async_trait]` repository port traits for
//! the headline 6 aggregates (`Wallet` + `WalletTransaction`). The
//! remaining 42 repository port traits are added in subsequent
//! workstreams. Storage adapters (PG/MySQL/SQLite) implement these
//! in Phase 17 (production hardening); the test fixtures in this
//! crate use in-memory implementations matching the Phase 5/6 pattern.

#![allow(missing_docs)]
#![allow(dead_code)]

use async_trait::async_trait;

use educore_core::error::Result;
use educore_core::ids::SchoolId;
use educore_core::tenant::TenantContext;

use crate::aggregate::{Wallet, WalletTransaction};
use crate::value_objects::{WalletId, WalletTransactionId};

#[async_trait]
pub trait WalletRepository: Send + Sync {
    /// Look up a wallet by id.
    async fn get(&self, ctx: &TenantContext, id: WalletId) -> Result<Option<Wallet>>;

    /// Look up a wallet by `(school_id, user_id)` (the canonical
    /// index for "find this user's wallet").
    async fn get_by_user(
        &self,
        school: SchoolId,
        user_id: educore_core::ids::UserId,
    ) -> Result<Option<Wallet>>;

    /// List all wallets in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<Wallet>>;

    /// List all wallets belonging to a user across schools (rare;
    /// used by the due-fees login-prevention scan).
    async fn list_for_user(&self, user_id: educore_core::ids::UserId) -> Result<Vec<Wallet>>;

    /// Insert a new wallet.
    async fn insert(&self, ctx: &TenantContext, w: &Wallet) -> Result<()>;

    /// Update an existing wallet.
    async fn update(&self, ctx: &TenantContext, w: &Wallet) -> Result<()>;
}

#[async_trait]
pub trait WalletTransactionRepository: Send + Sync {
    /// Look up a wallet transaction by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: WalletTransactionId,
    ) -> Result<Option<WalletTransaction>>;

    /// List all transactions for a wallet, newest first.
    async fn list_for_wallet(&self, wallet_id: WalletId) -> Result<Vec<WalletTransaction>>;

    /// List all approved transactions for a wallet (used by the
    /// `WalletService::balance` cross-check helper).
    async fn list_approved_for_wallet(&self, wallet_id: WalletId)
        -> Result<Vec<WalletTransaction>>;

    /// List all pending transactions in a school (used by the
    /// approval inbox).
    async fn list_pending(&self, school: SchoolId) -> Result<Vec<WalletTransaction>>;

    /// Insert a new wallet transaction.
    async fn insert(&self, ctx: &TenantContext, tx: &WalletTransaction) -> Result<()>;

    /// Update an existing wallet transaction.
    async fn update(&self, ctx: &TenantContext, tx: &WalletTransaction) -> Result<()>;
}
