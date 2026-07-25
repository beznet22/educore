//! # Finance child entities
//!
//! Phase 7 ships the 5 child entities from
//! `docs/specs/finance/entities.md`. Each is owned by an aggregate
//! root and persisted as a child row (loaded through the aggregate
//! repository).

#![allow(missing_docs)]
#![allow(unused_imports)]
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};

use crate::value_objects::{
    AmountTransferId, BalanceType, BankStatementId, Currency, FeesPaymentId, PayrollPaymentId,
    WalletTransactionId,
};

fn fresh_etag() -> Etag {
    Etag::placeholder()
}

// =============================================================================
// WalletTransactionApproval (headline 6 — Wallet + Refund)
// =============================================================================

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
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Returns `true` if the approval row is in the pending state
    /// (no approval, no rejection recorded yet).
    #[must_use]
    pub const fn is_pending(&self) -> bool {
        self.approved_at.is_none() && self.rejected_at.is_none()
    }

    /// Returns `true` if the approval row is currently active (not
    /// retired via tombstone).
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Returns `true` if the approval row has been approved.
    #[must_use]
    pub const fn is_approved(&self) -> bool {
        self.approved_at.is_some()
    }

    /// Returns `true` if the approval row has been rejected.
    #[must_use]
    pub const fn is_rejected(&self) -> bool {
        self.rejected_at.is_some()
    }

    /// Transitions the approval row from `pending` to `approved`.
    /// Per WTA I-1, this is a state-machine transition: cannot approve
    /// a row that is already approved or rejected. Sets `approver_id`,
    /// `approved_at`, bumps version, advances `updated_at`, sets
    /// `updated_by`. The caller (the service layer) mints the
    /// `event_id` and sets `last_event_id` after construction.
    pub fn approve(
        &mut self,
        approver: UserId,
        at: Timestamp,
        event_id: EventId,
    ) -> educore_core::error::Result<()> {
        if self.is_approved() {
            return Err(educore_core::error::DomainError::conflict(
                "WalletTransactionApproval is already approved",
            ));
        }
        if self.is_rejected() {
            return Err(educore_core::error::DomainError::conflict(
                "WalletTransactionApproval is already rejected; cannot approve",
            ));
        }
        self.approver_id = Some(approver);
        self.approved_at = Some(at);
        self.updated_at = at;
        self.updated_by = approver;
        self.last_event_id = Some(event_id);
        self.version = self.version.next();
        Ok(())
    }

    /// Transitions the approval row from `pending` to `rejected`.
    /// Per WTA I-1, this is a state-machine transition: cannot reject
    /// a row that is already approved or rejected. Sets `rejecter_id`,
    /// `rejected_at`, `reject_note`, bumps version, advances
    /// `updated_at`, sets `updated_by`. Per WTA I-2, the reject note is
    /// required and validated (1..=500 chars after trim) by the
    /// caller via `validate_reject_note` before this method is called.
    pub fn reject(
        &mut self,
        rejecter: UserId,
        note: String,
        at: Timestamp,
        event_id: EventId,
    ) -> educore_core::error::Result<()> {
        if self.is_approved() {
            return Err(educore_core::error::DomainError::conflict(
                "WalletTransactionApproval is already approved; cannot reject",
            ));
        }
        if self.is_rejected() {
            return Err(educore_core::error::DomainError::conflict(
                "WalletTransactionApproval is already rejected",
            ));
        }
        self.rejecter_id = Some(rejecter);
        self.rejected_at = Some(at);
        self.reject_note = Some(note);
        self.updated_at = at;
        self.updated_by = rejecter;
        self.last_event_id = Some(event_id);
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// FeesPaymentSlip (owned by FeesPayment)
// =============================================================================

/// A scanned slip attached to a `FeesPayment`. The slip lives in
/// the file storage port (Phase 15); this row holds the
/// `FileReference` + a free-text note.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesPaymentSlip {
    /// The owning school (derived from `fees_payment_id`).
    pub school_id: SchoolId,
    /// The parent payment.
    pub fees_payment_id: FeesPaymentId,
    /// The file reference (Phase 15 file storage port).
    pub slip_reference: Uuid,
    /// A free-text note.
    pub note: Option<String>,
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

impl FeesPaymentSlip {
    /// Constructs a new `FeesPaymentSlip` row.
    pub fn fresh(
        fees_payment_id: FeesPaymentId,
        slip_reference: Uuid,
        note: Option<String>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: fees_payment_id.school_id(),
            fees_payment_id,
            slip_reference,
            note,
            version: Version::initial(),
            etag: fresh_etag(),
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

// =============================================================================
// PayrollPaymentApproval (owned by PayrollPayment — finance side)
// =============================================================================

/// The approval state of a finance-side `PayrollPayment`. Recorded
/// when the accountant approves / rejects the bridge-emitted
/// payment (the HR→finance bridge creates the row in
/// `Pending`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayrollPaymentApproval {
    /// The owning school (derived from `payroll_payment_id`).
    pub school_id: SchoolId,
    /// The parent payment.
    pub payroll_payment_id: PayrollPaymentId,
    /// The approver.
    pub approver_id: Option<UserId>,
    /// The approval time.
    pub approved_at: Option<Timestamp>,
    /// The rejecter.
    pub rejecter_id: Option<UserId>,
    /// The rejection time.
    pub rejected_at: Option<Timestamp>,
    /// The rejection reason.
    pub rejection_reason: Option<String>,
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

impl PayrollPaymentApproval {
    /// Constructs a new `PayrollPaymentApproval` row in the
    /// initial state.
    pub fn fresh(
        payroll_payment_id: PayrollPaymentId,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: payroll_payment_id.school_id(),
            payroll_payment_id,
            approver_id: None,
            approved_at: None,
            rejecter_id: None,
            rejected_at: None,
            rejection_reason: None,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Returns `true` if the approval is in the Pending state
    /// (PPA I-1). Pending means neither approved_at nor rejected_at
    /// has been set.
    #[must_use]
    pub const fn is_pending(&self) -> bool {
        self.approved_at.is_none() && self.rejected_at.is_none()
    }

    /// Returns `true` if the approval is in the Approved state
    /// (PPA I-1). Approved means approved_at has been set.
    #[must_use]
    pub const fn is_approved(&self) -> bool {
        self.approved_at.is_some()
    }

    /// Returns `true` if the approval is in the Rejected state
    /// (PPA I-1). Rejected means rejected_at has been set.
    #[must_use]
    pub const fn is_rejected(&self) -> bool {
        self.rejected_at.is_some()
    }

    /// Returns `true` if the approval is still pending (i.e. has
    /// not been decided). Equivalent to `is_pending()` — kept as a
    /// separate method to match the Wave 76 / Wave 79 / Wave 80
    /// naming convention.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.is_pending()
    }

    /// Transitions the approval from Pending to Approved (PPA I-1).
    /// Returns `DomainError::conflict` if the approval is already
    /// in a terminal state (already approved or already rejected).
    /// Stamps `approver_id` + `approved_at` on the aggregate
    /// (PPA I-2). Bumps version, advances `updated_at`, sets
    /// `updated_by`.
    pub fn approve(
        &mut self,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        // PPA I-1: only Pending can transition.
        if !self.is_pending() {
            return Err(educore_core::error::DomainError::conflict(
                "PayrollPaymentApproval is not pending; cannot approve",
            ));
        }
        self.approver_id = Some(actor); // PPA I-2
        self.approved_at = Some(at); // PPA I-2
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Transitions the approval from Pending to Rejected (PPA I-1).
    /// Returns `DomainError::conflict` if the approval is already
    /// in a terminal state. Stamps `rejecter_id` + `rejected_at` +
    /// `rejection_reason` on the aggregate (PPA I-2). Bumps version,
    /// advances `updated_at`, sets `updated_by`.
    pub fn reject(
        &mut self,
        reason: Option<String>,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        // PPA I-1: only Pending can transition.
        if !self.is_pending() {
            return Err(educore_core::error::DomainError::conflict(
                "PayrollPaymentApproval is not pending; cannot reject",
            ));
        }
        self.rejecter_id = Some(actor); // PPA I-2
        self.rejected_at = Some(at); // PPA I-2
        self.rejection_reason = reason
            .map(|r| r.trim().to_owned())
            .filter(|r| !r.is_empty()); // PPA I-2
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// AmountTransferLeg (owned by AmountTransfer)
// =============================================================================

/// One leg of a two-leg `AmountTransfer`. Each transfer produces
/// exactly two `BankStatement` rows (one debit on the source, one
/// credit on the destination) and two `AmountTransferLeg` rows in
/// a single transaction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AmountTransferLeg {
    /// The owning school (derived from `amount_transfer_id`).
    pub school_id: SchoolId,
    /// The parent transfer.
    pub amount_transfer_id: AmountTransferId,
    /// The bank account for this leg.
    pub bank_id: crate::value_objects::BankAccountId,
    /// The direction (debit on the source or credit on the
    /// destination).
    pub direction: BalanceType,
    /// The leg amount in minor units.
    pub amount_minor: i64,
    /// The currency.
    pub currency: Currency,
    /// The resulting `BankStatement` row (set on completion).
    pub bank_statement_id: Option<BankStatementId>,
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

impl AmountTransferLeg {
    /// Constructs a new `AmountTransferLeg` row.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        amount_transfer_id: AmountTransferId,
        bank_id: crate::value_objects::BankAccountId,
        direction: BalanceType,
        amount_minor: i64,
        currency: Currency,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: amount_transfer_id.school_id(),
            amount_transfer_id,
            bank_id,
            direction,
            amount_minor,
            currency,
            bank_statement_id: None,
            version: Version::initial(),
            etag: fresh_etag(),
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

// =============================================================================
// BankStatementAttachment (owned by BankStatement)
// =============================================================================

/// A receipt or file attached to a `BankStatement`. The file lives
/// in the file storage port (Phase 15); this row holds the
/// `FileReference` + the upload time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BankStatementAttachment {
    /// The owning school (derived from `bank_statement_id`).
    pub school_id: SchoolId,
    /// The parent statement.
    pub bank_statement_id: BankStatementId,
    /// The file reference.
    pub file_reference: Uuid,
    /// The upload time.
    pub uploaded_at: Timestamp,
    /// The uploader.
    pub uploaded_by: UserId,
    /// A free-text description.
    pub description: Option<String>,
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

impl BankStatementAttachment {
    /// Constructs a new `BankStatementAttachment` row.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        bank_statement_id: BankStatementId,
        file_reference: Uuid,
        uploaded_at: Timestamp,
        uploaded_by: UserId,
        description: Option<String>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: bank_statement_id.school_id(),
            bank_statement_id,
            file_reference,
            uploaded_at,
            uploaded_by,
            description,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Returns `true` if the attachment is currently active (not retired).
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Soft-deletes the attachment by flipping `active_status` to
    /// `Retired`. This is a **tombstone**, NOT a content edit —
    /// the original `bank_statement_id` + `file_reference` +
    /// `uploaded_at` + `uploaded_by` + `description` are preserved in
    /// the audit footer for legal-record retention. BSA I-1
    /// (attachment ref valid) is upheld because `retire()` does NOT
    /// mutate the file_reference; BSA I-2 (orphan after
    /// BankStatement delete) is upheld because the
    /// `bank_statement_id` reference is preserved in the audit
    /// footer even after retire.
    pub fn retire(
        &mut self,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "BankStatementAttachment is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
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
    use educore_core::ids::Identifier;

    #[test]
    fn wallet_transaction_approval_starts_unapproved() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let user = g.next_user_id();
        let id = WalletTransactionId::new(school, g.next_uuid());
        let row = WalletTransactionApproval::fresh(
            id,
            user,
            Timestamp::now(),
            CorrelationId(g.next_uuid()),
        );
        assert!(row.approver_id.is_none());
        assert!(row.rejecter_id.is_none());
    }

    #[test]
    fn fees_payment_slip_carries_file_reference() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let user = g.next_user_id();
        let id = FeesPaymentId::new(school, g.next_uuid());
        let row = FeesPaymentSlip::fresh(
            id,
            g.next_uuid(),
            Some("test slip".to_owned()),
            user,
            Timestamp::now(),
            CorrelationId(g.next_uuid()),
        );
        assert_eq!(row.fees_payment_id, id);
        assert_eq!(row.school_id, school);
    }

    #[test]
    fn payroll_payment_approval_starts_unapproved() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let user = g.next_user_id();
        let id = PayrollPaymentId::new(school, g.next_uuid());
        let row =
            PayrollPaymentApproval::fresh(id, user, Timestamp::now(), CorrelationId(g.next_uuid()));
        assert!(row.approver_id.is_none());
        assert!(row.rejection_reason.is_none());
    }

    #[test]
    fn amount_transfer_leg_carries_direction() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let user = g.next_user_id();
        let id = AmountTransferId::new(school, g.next_uuid());
        let acct = crate::value_objects::BankAccountId::new(school, g.next_uuid());
        let row = AmountTransferLeg::fresh(
            id,
            acct,
            BalanceType::Debit,
            5000,
            Currency::INR,
            user,
            Timestamp::now(),
            CorrelationId(g.next_uuid()),
        );
        assert_eq!(row.amount_minor, 5000);
        assert!(row.bank_statement_id.is_none());
    }

    #[test]
    fn bank_statement_attachment_carries_uploader() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let user = g.next_user_id();
        let id = BankStatementId::new(school, g.next_uuid());
        let row = BankStatementAttachment::fresh(
            id,
            g.next_uuid(),
            Timestamp::now(),
            user,
            Some("receipt".to_owned()),
            user,
            Timestamp::now(),
            CorrelationId(g.next_uuid()),
        );
        assert_eq!(row.uploaded_by, user);
        assert!(row.bank_statement_id == id);
    }
}
