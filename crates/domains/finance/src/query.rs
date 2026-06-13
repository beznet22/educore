//! # Finance query stubs
//!
//! Phase 7 query stubs for the headline 6 aggregates. Every
//! `execute()` returns `Err(DomainError::not_supported(...))`
//! until Phase 17 wires the typed executor + storage-port
//! translation.

#![allow(missing_docs)]
#![allow(dead_code)]

use educore_core::error::{DomainError, Result};
use educore_core::ids::SchoolId;
use educore_core::tenant::TenantContext;

use crate::aggregate::{FeesPayment, Wallet, WalletTransaction};
use crate::value_objects::{ApprovalStatus, WalletId, WalletTransactionId, WalletTxType};

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WalletQuery {
    pub user_id: Option<educore_core::ids::UserId>,
    pub min_balance_minor: Option<i64>,
    pub max_balance_minor: Option<i64>,
    pub offset: u32,
    pub limit: u32,
}

impl WalletQuery {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            user_id: None,
            min_balance_minor: None,
            max_balance_minor: None,
            offset: 0,
            limit: 50,
        }
    }

    /// Phase 7 stub.
    pub async fn execute(&self, _ctx: &TenantContext) -> Result<Vec<Wallet>> {
        Err(DomainError::not_supported(
            "WalletQuery::execute is a Phase 7 stub",
        ))
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WalletTransactionQuery {
    pub wallet_id: Option<WalletId>,
    pub user_id: Option<educore_core::ids::UserId>,
    pub wallet_type: Option<WalletTxType>,
    pub status: Option<ApprovalStatus>,
    pub from: Option<chrono::NaiveDate>,
    pub to: Option<chrono::NaiveDate>,
    pub offset: u32,
    pub limit: u32,
}

impl WalletTransactionQuery {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            wallet_id: None,
            user_id: None,
            wallet_type: None,
            status: None,
            from: None,
            to: None,
            offset: 0,
            limit: 50,
        }
    }

    /// Phase 7 stub.
    pub async fn execute(&self, _ctx: &TenantContext) -> Result<Vec<WalletTransaction>> {
        Err(DomainError::not_supported(
            "WalletTransactionQuery::execute is a Phase 7 stub",
        ))
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FeesPaymentQuery {
    pub student_id: Option<educore_academic::StudentId>,
    pub from: Option<chrono::NaiveDate>,
    pub to: Option<chrono::NaiveDate>,
    pub offset: u32,
    pub limit: u32,
}

impl FeesPaymentQuery {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            student_id: None,
            from: None,
            to: None,
            offset: 0,
            limit: 50,
        }
    }

    /// Phase 7 stub.
    pub async fn execute(&self, _ctx: &TenantContext) -> Result<Vec<FeesPayment>> {
        Err(DomainError::not_supported(
            "FeesPaymentQuery::execute is a Phase 7 stub",
        ))
    }
}
