//! # Finance child entities
//!
//! Phase 7 ships the [`WalletTransactionApproval`] child entity
//! for the headline `Wallet` / `WalletTransaction` aggregates. The
//! remaining 3+ child entities from the spec
//! (`FeesPaymentSlip`, `PayrollPaymentApproval`,
//! `AmountTransferLeg`, `BankStatementAttachment`, etc.) land in
//! subsequent workstreams.

#![allow(missing_docs)]
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};

use crate::value_objects::WalletTransactionId;

/// The approval state of a wallet transaction. Mirrors the
/// HR `StaffNote` / `StaffAttendancePromotion` child-entity pattern
/// at `crates/domains/hr/src/entities.rs`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalletTransactionApproval {
    /// The owning school (derived from `wallet_transaction_id`).
    pub school_id: SchoolId,
    /// The transaction this approval row belongs to.
    pub wallet_transaction_id: WalletTransactionId,
    /// The approver (set on `Approved`).
    pub approver_id: Option<UserId>,
    /// The approval time.
    pub approved_at: Option<Timestamp>,
    /// The rejecter (set on `Rejected`).
    pub rejecter_id: Option<UserId>,
    /// The rejection time.
    pub rejected_at: Option<Timestamp>,
    /// The rejection note.
    pub reject_note: Option<String>,
    /// An optional file reference (audit receipt).
    pub file_reference: Option<Uuid>,
    /// The audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl WalletTransactionApproval {
    /// Constructs a new `WalletTransactionApproval` row in the
    /// initial state (no approval, no rejection).
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        wallet_transaction_id: WalletTransactionId,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: wallet_transaction_id.school_id(),
            wallet_transaction_id,
            approver_id: None,
            approved_at: None,
            rejecter_id: None,
            rejected_at: None,
            reject_note: None,
            file_reference: None,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}
