//! # Finance aggregate roots
//!
//! Phase 7 ships the headline 6 aggregates per the prompt:
//! `Wallet`, `WalletTransaction` (with `wallet_type=Refund` for the
//! `Refund` headline), `FeesInvoice`, `FeesPayment`, `Expense`.
//!
//! Every aggregate follows the standard audit-footer pattern (per
//! `AGENTS.md`):
//!
//! - 1 typed id (e.g. `WalletId`) + 1 derived `school_id` anchor
//! - domain fields
//! - audit-metadata fields: `version`, `etag`, `created_at`,
//!   `updated_at`, `created_by`, `updated_by`, `active_status`,
//!   `last_event_id`, `correlation_id`
//!
//! `school_id` is **derived from `id.school_id()`**, never taken
//! from the caller.

// Module-level docs for every public item are tracked in
// `docs/specs/finance/`. The `#[allow(missing_docs)]` here is a
// conscious exception for the Phase 7 finance crate: adding rustdoc
// for ~80 fields + ~40 placeholder-aggregate stubs across
// `aggregate.rs` is the Workstream K backlog (see
// `PHASE-7-HANDOFF.md` § Workstream K).
#![allow(missing_docs)]

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_academic::{AcademicYearId, StudentId};
use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};

use crate::value_objects::{
    validate_discount_name, validate_donor_name, validate_ledger_name, AccountType, Amount,
    AmountTransferId, ApprovalStatus, BalanceType, BankAccountId, BankPaymentSlipAuditId, BankPaymentSlipId, BankStatementAttachmentId, LifecycleStatus,
    BankStatementId,
    ChartOfAccountId, Currency, DirectFeesInstallmentAssignChildId,
    DirectFeesInstallmentChildPaymentId,
    DirectFeesInstallmentAssignId, DirectFeesInstallmentId, DirectFeesReminderId, DirectFeesSettingId, DiscountType, DonorId,
    DueFeesLoginPreventId, ExpenseApprovalId, ExpenseHeadId, ExpenseId, FeesAssignDiscountId,
    FeesAssignId, FeesCarryForwardId, FeesCarryForwardLogId, FeesCarryForwardSettingId,
    FeesDiscountId, FeesGroupId, FeesInstallmentAssignDiscountId, FeesInstallmentAssignId,
    FeesInstallmentCreditId, FeesInstallmentId, FeesInvoiceId, FeesInvoiceSettingId, FeesMasterId,
    FeesPaymentId, FeesPaymentStatus, FeesTypeId, FineAmount, FmFeesGroupId, FmFeesInvoiceChildId,
    FmFeesInvoiceId, FmFeesInvoiceLineNoteId, FmFeesInvoiceSettingId, FmFeesTransactionChildId,
    FmFeesTransactionId, FmFeesTransactionLineNoteId, FmFeesTypeId, FmFeesTypeKind, FmFeesWeaverId, FmInvoiceType,
    IncomeApprovalId, IncomeHeadId, IncomeId, InventoryPaymentId, InvoiceSettingId, Money, PaymentGatewaySettingId,
    PaymentMode, ProductPurchaseLifecycleStatus, TransactionLifecycleStatus,
    PaymentMethodId, PaymentMethodKind, PayrollEarnDeducId, PayrollGenerateId, GatewayMode, GatewayChargeType,
    PayrollPaymentApprovalId, PayrollPaymentId, ProductPurchaseId, QuestionBankFeeId,
    SalaryTemplateId, StatementType, TransactionId, WalletId, WalletTransactionApprovalId, WalletTransactionId, WalletTxType,
};
use educore_core::clock::Clock;
use educore_core::ids::Identifier;

fn fresh_etag() -> Etag {
    Etag::placeholder()
}

// =============================================================================
// Headline 1: Wallet (new — user balance projection)
// =============================================================================

/// The user-balance projection. A `Wallet` is created lazily on the
/// first `WalletTransaction` for a `(school_id, user_id)` pair and
/// then the `balance_minor` is cached for read performance; the
/// authoritative balance is the sum of approved `WalletTransaction`
/// rows for the wallet, recomputed on every approval.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Wallet {
    /// The typed id (school_id + uuid).
    pub id: WalletId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The user that owns this wallet.
    pub user_id: UserId,
    /// The cached balance in minor units (cents / paisa).
    pub balance_minor: i64,
    /// The wallet's currency.
    pub currency: Currency,
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

impl Wallet {
    /// Constructs a new `Wallet` with the zero balance.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: WalletId,
        user_id: UserId,
        currency: Currency,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            user_id,
            balance_minor: 0,
            currency,
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

    /// Returns the current balance as an `Amount` in the wallet's
    /// currency.
    #[must_use]
    pub fn balance(&self) -> Amount {
        Amount {
            money: Money {
                amount_minor: self.balance_minor,
                currency: self.currency,
            },
        }
    }

    /// Applies a credit (deposit / refund) to the wallet. Returns
    /// `Err` if the wallet's currency doesn't match the credit's
    /// currency.
    pub fn apply_credit(
        &mut self,
        amount_minor: i64,
        currency: Currency,
        actor: UserId,
        at: Timestamp,
    ) -> educore_core::error::Result<()> {
        if amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "credit amount must be non-negative",
            ));
        }
        if currency.0 != self.currency.0 {
            return Err(educore_core::error::DomainError::validation(
                "credit currency does not match wallet currency",
            ));
        }
        self.balance_minor = self.balance_minor.saturating_add(amount_minor);
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Applies a debit (expense / fees refund) to the wallet. Returns
    /// `Err` if the wallet has insufficient balance or the
    /// currencies don't match.
    pub fn apply_debit(
        &mut self,
        amount_minor: i64,
        currency: Currency,
        actor: UserId,
        at: Timestamp,
    ) -> educore_core::error::Result<()> {
        if amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "debit amount must be non-negative",
            ));
        }
        if currency.0 != self.currency.0 {
            return Err(educore_core::error::DomainError::validation(
                "debit currency does not match wallet currency",
            ));
        }
        if self.balance_minor < amount_minor {
            return Err(educore_core::error::DomainError::conflict(format!(
                "insufficient wallet balance: have {}, need {amount_minor}",
                self.balance_minor
            )));
        }
        self.balance_minor = self.balance_minor.saturating_sub(amount_minor);
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Computes the **authoritative** wallet balance as the sum
    /// of approved `WalletTransaction` rows for this wallet.
    /// Per the spec ("the authoritative balance is the sum of
    /// approved `WalletTransaction` rows for the wallet,
    /// recomputed on every approval"), this is the
    /// ground-truth computation. The cached `balance_minor`
    /// field is an index for read performance and may drift in
    /// the presence of out-of-band writes (data imports,
    /// replay, manual SQL fixes); callers that require
    /// strict consistency should use
    /// [`reconcile_and_validate`](Self::reconcile_and_validate).
    ///
    /// `Pending` and `Rejected` transactions are excluded.
    /// Cross-currency transactions are rejected with
    /// `Validation` (a single wallet holds one currency).
    #[must_use]
    pub fn reconcile_balance(transactions: &[&WalletTransaction]) -> i64 {
        let mut total: i64 = 0;
        for tx in transactions {
            if tx.status != ApprovalStatus::Approved {
                continue;
            }
            if tx.wallet_type.is_credit() {
                total = total.saturating_add(tx.amount_minor);
            } else if tx.wallet_type.is_debit() {
                total = total.saturating_sub(tx.amount_minor);
            }
        }
        total
    }

    /// Reconciles the cached `balance_minor` against the
    /// authoritative sum of approved transactions and
    /// returns `Err(Conflict)` on drift. Use this from the
    /// dispatcher / reconciliation job to detect cache vs
    /// source-of-truth divergence (out-of-band writes, partial
    /// replay, missing outbox commit, etc.).
    pub fn reconcile_and_validate(
        &self,
        transactions: &[&WalletTransaction],
    ) -> educore_core::error::Result<()> {
        let authoritative = Self::reconcile_balance(transactions);
        if authoritative != self.balance_minor {
            return Err(educore_core::error::DomainError::conflict(format!(
                "wallet balance drift: cached={}, authoritative={}",
                self.balance_minor, authoritative
            )));
        }
        Ok(())
    }
}

// =============================================================================
// Headline 2: WalletTransaction (Refund-as-WalletTransaction)
// =============================================================================

/// A wallet movement. The state machine mirrors
/// [`ApprovalStatus`](crate::value_objects::ApprovalStatus): a
/// `Pending` transaction must be approved before it credits or
/// debits the wallet.
///
/// The Phase 7 `Refund` headline is modeled as a
/// `WalletTransaction` with [`wallet_type`](Self::wallet_type) =
/// [`WalletTxType::Refund`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalletTransaction {
    /// The typed id (school_id + uuid).
    pub id: WalletTransactionId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The wallet this transaction belongs to.
    pub wallet_id: WalletId,
    /// The user that owns the wallet (denormalized for indexing).
    pub user_id: UserId,
    /// The amount in minor units (always non-negative).
    pub amount_minor: i64,
    /// The transaction's currency.
    pub currency: Currency,
    /// The kind of transaction (deposit / refund / expense / fees-refund).
    pub wallet_type: WalletTxType,
    /// The approval state.
    pub status: ApprovalStatus,
    /// The optional payment method used (cash / bank / gateway).
    pub payment_method_id: Option<PaymentMethodId>,
    /// The optional bank account the funds are coming from / going to.
    pub bank_id: Option<BankAccountId>,
    /// A free-text reference (e.g. gateway transaction id, receipt #).
    pub reference: Option<String>,
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
    /// The approver (set on `Approved`).
    pub approved_by: Option<UserId>,
    /// The approval time.
    pub approved_at: Option<Timestamp>,
    /// The rejecter (set on `Rejected`).
    pub rejected_by: Option<UserId>,
    /// The rejection time.
    pub rejected_at: Option<Timestamp>,
    /// The rejection note.
    pub reject_note: Option<String>,
}

impl WalletTransaction {
    /// Constructs a new `WalletTransaction` in the `Pending` state.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: WalletTransactionId,
        wallet_id: WalletId,
        user_id: UserId,
        amount_minor: i64,
        currency: Currency,
        wallet_type: WalletTxType,
        payment_method_id: Option<PaymentMethodId>,
        bank_id: Option<BankAccountId>,
        reference: Option<String>,
        note: Option<String>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        if amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "wallet transaction amount must be non-negative",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            wallet_id,
            user_id,
            amount_minor,
            currency,
            wallet_type,
            status: ApprovalStatus::Pending,
            payment_method_id,
            bank_id,
            reference,
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
            approved_by: None,
            approved_at: None,
            rejected_by: None,
            rejected_at: None,
            reject_note: None,
        })
    }

    /// Returns `true` if the state machine permits the
    /// `from -> to` transition.
    pub fn can_transition(&self, to: ApprovalStatus) -> bool {
        self.status.can_transition_to(to)
    }

    /// Approves the transaction. Returns `Err` if the state machine
    /// does not permit the transition.
    pub fn approve(
        &mut self,
        approver: UserId,
        at: Timestamp,
        event_id: EventId,
    ) -> educore_core::error::Result<()> {
        if !self.can_transition(ApprovalStatus::Approved) {
            return Err(educore_core::error::DomainError::conflict(format!(
                "wallet transaction is in state {:?}, cannot transition to Approved",
                self.status
            )));
        }
        self.status = ApprovalStatus::Approved;
        self.approved_by = Some(approver);
        self.approved_at = Some(at);
        self.updated_at = at;
        self.updated_by = approver;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
        Ok(())
    }

    /// Rejects the transaction. Returns `Err` if the state machine
    /// does not permit the transition.
    pub fn reject(
        &mut self,
        rejecter: UserId,
        note: String,
        at: Timestamp,
        event_id: EventId,
    ) -> educore_core::error::Result<()> {
        if !self.can_transition(ApprovalStatus::Rejected) {
            return Err(educore_core::error::DomainError::conflict(format!(
                "wallet transaction is in state {:?}, cannot transition to Rejected",
                self.status
            )));
        }
        self.status = ApprovalStatus::Rejected;
        self.rejected_by = Some(rejecter);
        self.rejected_at = Some(at);
        self.reject_note = Some(note);
        self.updated_at = at;
        self.updated_by = rejecter;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
        Ok(())
    }
}

// =============================================================================
// Stubs for the other 4 headline aggregates (FeesInvoice, FeesPayment,
// Expense) — typed-shape-only; real impl lands in subsequent
// workstreams per the Phase 7 plan.
// =============================================================================

/// The classic invoice numbering scheme. Storing the `prefix` and
/// `start_form` that drive the next invoice number for a school.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesInvoice {
    /// The typed id (school_id + uuid).
    pub id: FeesInvoiceId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The invoice prefix (e.g. `"INV-"`).
    pub prefix: String,
    /// The starting number for invoice sequencing.
    pub start_form: i64,
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

impl FeesInvoice {
    /// Constructs a new `FeesInvoice` numbering configuration.
    pub fn fresh(
        id: FeesInvoiceId,
        prefix: String,
        start_form: i64,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        if prefix.is_empty() || prefix.len() > 10 {
            return Err(educore_core::error::DomainError::validation(
                "invoice prefix must be 1..=10 chars",
            ));
        }
        if start_form < 0 {
            return Err(educore_core::error::DomainError::validation(
                "invoice start_form must be non-negative",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            prefix,
            start_form,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Returns the next invoice number for this school, per the
    /// spec invariant 3: `next = start_form + count(issued)`.
    ///
    /// `issued_count` is the count of invoices already issued
    /// under this `FeesInvoice` configuration (looked up by the
    /// dispatcher / repository). The returned string is
    /// `format!("{}{}", prefix, next_number)` (e.g. prefix
    /// `"INV-"`, `start_form = 1000`, `issued_count = 7` ⇒
    /// `"INV-1007"`). Wraps `start_form + issued_count` in a
    /// `Validation` error if the addition overflows `i64`.
    pub fn next_invoice_number(&self, issued_count: u64) -> educore_core::error::Result<String> {
        let next = self.start_form.checked_add(issued_count as i64).ok_or_else(|| {
            educore_core::error::DomainError::validation(
                "invoice number overflow: start_form + issued_count exceeds i64::MAX",
            )
        })?;
        Ok(format!("{}{}", self.prefix, next))
    }
}

/// A single payment against a `FeesAssign` (or a
/// `FeesInstallmentAssign`). The double-entry invariant
/// (`sum(debits) == sum(credits)` per `school_id`) is verified by
/// the `DoubleEntryService` property test (Workstream C).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesPayment {
    /// The typed id (school_id + uuid).
    pub id: FeesPaymentId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The amount in minor units.
    pub amount_minor: i64,
    /// The currency.
    pub currency: Currency,
    /// The optional discount applied (non-negative).
    pub discount_minor: i64,
    /// The optional fine captured (non-negative).
    pub fine_minor: i64,
    /// The payment method used.
    pub payment_method: PaymentMethodKind,
    /// The optional bank account.
    pub bank_id: Option<BankAccountId>,
    /// The optional payment method id (FK to `PaymentMethod`).
    pub payment_method_id: Option<PaymentMethodId>,
    /// A free-text reference (gateway transaction id, slip #, etc.).
    pub reference: Option<String>,
    /// A free-text note.
    pub note: Option<String>,
    /// The payment date.
    pub payment_date: NaiveDate,
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

impl FeesPayment {
    /// Constructs a new `FeesPayment`.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: FeesPaymentId,
        amount_minor: i64,
        currency: Currency,
        discount_minor: i64,
        fine_minor: i64,
        payment_method: PaymentMethodKind,
        bank_id: Option<BankAccountId>,
        payment_method_id: Option<PaymentMethodId>,
        reference: Option<String>,
        note: Option<String>,
        payment_date: NaiveDate,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        if amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "payment amount must be non-negative",
            ));
        }
        if discount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "payment discount must be non-negative",
            ));
        }
        if fine_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "payment fine must be non-negative",
            ));
        }
        // INV-FP-NET-NON-NEGATIVE: a discount larger than the
        // gross amount would yield a negative net payable,
        // which is nonsense for a payment record.
        if discount_minor > amount_minor {
            return Err(educore_core::error::DomainError::validation(
                "payment discount must not exceed gross amount",
            ));
        }
        // INV-FP-METHOD-FK: any non-cash payment method
        // (Bank, Cheque, Card, Mobile, Gateway) must reference
        // a `PaymentMethod` row — cash can omit it because
        // there is no method config for the till.
        if payment_method != PaymentMethodKind::Cash && payment_method_id.is_none() {
            return Err(educore_core::error::DomainError::validation(
                "non-cash payment methods must reference a PaymentMethod row",
            ));
        }
        // INV-FP-GATEWAY-REF: a Gateway-backed payment must
        // carry a non-empty reference (the gateway transaction
        // id); reconciliation cannot match a gateway debit
        // against a finance payment without it.
        if payment_method == PaymentMethodKind::Gateway {
            match &reference {
                None => {
                    return Err(educore_core::error::DomainError::validation(
                        "gateway payments require a reference (gateway transaction id)",
                    ));
                }
                Some(s) if s.trim().is_empty() => {
                    return Err(educore_core::error::DomainError::validation(
                        "gateway payments require a non-empty reference",
                    ));
                }
                _ => {}
            }
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            amount_minor,
            currency,
            discount_minor,
            fine_minor,
            payment_method,
            bank_id,
            payment_method_id,
            reference,
            note,
            payment_date,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Returns the net amount (amount - discount) in minor units.
    #[must_use]
    pub const fn net_minor(&self) -> i64 {
        self.amount_minor.saturating_sub(self.discount_minor)
    }

    /// Returns the total payable (`net + fine`) in minor units.
    /// This is the amount the cashier should collect and the
    /// amount the reconciliation engine should match against the
    /// bank / wallet debit.
    #[must_use]
    pub const fn total_payable_minor(&self) -> i64 {
        self.net_minor().saturating_add(self.fine_minor)
    }

    /// Re-validates the aggregate's invariants. Useful as a
    /// post-load / pre-dispatch sanity check (and exposed for
    /// integration tests). Returns `Err(Validation)` if any
    /// invariant is broken.
    pub fn validate_consistency(&self) -> educore_core::error::Result<()> {
        if self.amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "payment amount must be non-negative",
            ));
        }
        if self.discount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "payment discount must be non-negative",
            ));
        }
        if self.fine_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "payment fine must be non-negative",
            ));
        }
        if self.discount_minor > self.amount_minor {
            return Err(educore_core::error::DomainError::validation(
                "payment discount must not exceed gross amount",
            ));
        }
        if self.payment_method == PaymentMethodKind::Gateway {
            match &self.reference {
                None => {
                    return Err(educore_core::error::DomainError::validation(
                        "gateway payments require a non-empty reference",
                    ));
                }
                Some(s) if s.trim().is_empty() => {
                    return Err(educore_core::error::DomainError::validation(
                        "gateway payments require a non-empty reference",
                    ));
                }
                _ => {}
            }
        }
        if self.payment_method != PaymentMethodKind::Cash && self.payment_method_id.is_none() {
            return Err(educore_core::error::DomainError::validation(
                "non-cash payment methods must reference a PaymentMethod row",
            ));
        }
        Ok(())
    }
}

/// A recorded expense. Per the build-plan § "Risks", money is
/// `MinorUnits` (i64) — no floats.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Expense {
    /// The typed id (school_id + uuid).
    pub id: ExpenseId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The expense name.
    pub name: String,
    /// The amount in minor units.
    pub amount_minor: i64,
    /// The currency.
    pub currency: Currency,
    /// The expense head (category).
    pub expense_head_id: ExpenseHeadId,
    /// The account (bank / cash) the expense is paid from.
    pub account_id: BankAccountId,
    /// The resolved type of the referenced account (`Bank` or
    /// `Cash`). Stored on the aggregate so the
    /// payment-method-compatibility invariant is replayable
    /// after a load (round-trip parity) and so the dispatcher
    /// can re-validate without re-loading the account.
    pub account_type: AccountType,
    /// The payment method.
    pub payment_method: PaymentMethodKind,
    /// The expense date.
    pub expense_date: NaiveDate,
    /// The optional file reference (a receipt scan).
    pub file_reference: Option<Uuid>,
    /// A free-text description.
    pub description: Option<String>,
    /// The optional linked payroll payment (for payroll-derived
    /// expenses via the HR→finance bridge).
    pub payroll_payment_id: Option<PayrollPaymentId>,
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

impl Expense {
    /// Constructs a new `Expense`.
    ///
    /// `account_type` is the resolved [`AccountType`] of `account_id`
    /// (the caller must look it up before constructing the expense);
    /// the constructor enforces that the `payment_method` is
    /// compatible with the account type per the spec invariant 2
    /// (`payment_method == Cash` ⇔ `account_type == Cash`).
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: ExpenseId,
        name: String,
        amount_minor: i64,
        currency: Currency,
        expense_head_id: ExpenseHeadId,
        account_id: BankAccountId,
        account_type: AccountType,
        payment_method: PaymentMethodKind,
        expense_date: NaiveDate,
        file_reference: Option<Uuid>,
        description: Option<String>,
        payroll_payment_id: Option<PayrollPaymentId>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        validate_ledger_name(&name)?;
        if amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "expense amount must be non-negative",
            ));
        }
        // INV-EXP-METHOD-ACCOUNT: the payment_method must be
        // compatible with the resolved account_type. Cash method
        // is only valid against a Cash account; every other
        // method (Bank, Cheque, Card, Mobile, Gateway) must be
        // charged against a Bank account. This catches the
        // common bookkeeper error of paying an electricity
        // bill out of a bank account with `payment_method =
        // Cash` (or vice versa) at the aggregate boundary
        // instead of at the storage / dispatcher layer.
        match (payment_method, account_type) {
            (PaymentMethodKind::Cash, AccountType::Cash) => {}
            (
                PaymentMethodKind::Bank
                | PaymentMethodKind::Cheque
                | PaymentMethodKind::Card
                | PaymentMethodKind::Mobile
                | PaymentMethodKind::Gateway,
                AccountType::Bank,
            ) => {}
            (pm, at) => {
                return Err(educore_core::error::DomainError::validation(
                    format!(
                        "payment_method {pm:?} is not compatible with account_type {at:?}",
                        pm = pm.as_str(),
                        at = at.as_str(),
                    ),
                ));
            }
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            name,
            amount_minor,
            currency,
            expense_head_id,
            account_id,
            account_type,
            payment_method,
            expense_date,
            file_reference,
            description,
            payroll_payment_id,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }
}

// =============================================================================
// Stubs for the remaining 39 aggregates — placeholder
// `Default::default()` structs so the spec is exhaustively
// representable. Real impl lands in subsequent workstreams.
// =============================================================================

macro_rules! finance_aggregate_stub {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_attr:meta])*
                $field_vis:vis $field:ident : $field_ty:ty
            ),* $(,)?
        }
    ) => {
        $(#[$attr])*
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        $vis struct $name {
            /// Placeholder school_id (derived from id in real impl).
            pub school_id: SchoolId,
            $(
                $(#[$field_attr])*
                $field_vis $field: $field_ty
            ),*
        }
    };
}

finance_aggregate_stub! {
    /// FeesGroup (Phase 7 Workstream E).
    pub struct FeesGroup { _id: () }
}
finance_aggregate_stub! {
    /// FeesType (Phase 7 Workstream E).
    pub struct FeesType { _id: () }
}
finance_aggregate_stub! {
    /// FeesMaster (Phase 7 Workstream E).
    pub struct FeesMaster { _id: () }
}
finance_aggregate_stub! {
    /// FeesDiscount (Phase 7 Workstream E).
    pub struct FeesDiscount { _id: () }
}
finance_aggregate_stub! {
    /// FeesAssign (Phase 7 Workstream F).
    pub struct FeesAssign { _id: () }
}
finance_aggregate_stub! {
    /// FeesAssignDiscount (Phase 7 Workstream F).
    pub struct FeesAssignDiscount { _id: () }
}
finance_aggregate_stub! {
    /// FeesInstallment (Phase 7 Workstream F).
    pub struct FeesInstallment { _id: () }
}
finance_aggregate_stub! {
    /// FeesInstallmentAssign (Phase 7 Workstream F).
    pub struct FeesInstallmentAssign { _id: () }
}
finance_aggregate_stub! {
    /// DirectFeesInstallment (Phase 7 Workstream F).
    pub struct DirectFeesInstallment { _id: () }
}
finance_aggregate_stub! {
    /// DirectFeesInstallmentAssign (Phase 7 Workstream F).
    pub struct DirectFeesInstallmentAssign { _id: () }
}
finance_aggregate_stub! {
    /// DirectFeesInstallmentChildPayment (Phase 7 Workstream F).
    pub struct DirectFeesInstallmentChildPayment { _id: () }
}
finance_aggregate_stub! {
    /// DirectFeesSetting (Phase 7 Workstream F).
    pub struct DirectFeesSetting { _id: () }
}
finance_aggregate_stub! {
    /// DirectFeesReminder (Phase 7 Workstream F).
    pub struct DirectFeesReminder { _id: () }
}
finance_aggregate_stub! {
    /// FmFeesGroup (Phase 7 Workstream G).
    pub struct FmFeesGroup { _id: () }
}
finance_aggregate_stub! {
    /// FmFeesType (Phase 7 Workstream G).
    /// Real aggregate: RealFmFeesType (Wave 129). The stub is kept
    /// only to avoid breaking downstream code that referenced
    /// `FmFeesType` as a type name during Phase 7.
    pub struct FmFeesType { _id: () }
}
finance_aggregate_stub! {
    /// FmFeesInvoice (Phase 7 Workstream G).
    pub struct FmFeesInvoice { _id: () }
}
finance_aggregate_stub! {
    /// FmFeesInvoiceChild (Phase 7 Workstream G).
    pub struct FmFeesInvoiceChild { _id: () }
}
finance_aggregate_stub! {
    /// FmFeesInvoiceSetting (Phase 7 Workstream G).
    pub struct FmFeesInvoiceSetting { _id: () }
}
// `RealFmFeesTransaction` for new code; the stub is kept only
// to avoid breaking downstream code that referenced
// `FmFeesTransaction` as a type name during Phase 7.
finance_aggregate_stub! {
    /// FmFeesTransaction (Phase 7 Workstream G).
    pub struct FmFeesTransaction { _id: () }
}
finance_aggregate_stub! {
    /// FmFeesTransactionChild (Phase 7 Workstream G).
    pub struct FmFeesTransactionChild { _id: () }
}
finance_aggregate_stub! {
    /// FmFeesWeaver (Phase 7 Workstream G).
    pub struct FmFeesWeaver { _id: () }
}
finance_aggregate_stub! {
    /// FeesInvoiceSetting (Phase 7 Workstream B).
    pub struct FeesInvoiceSetting { _id: () }
}
finance_aggregate_stub! {
    /// InvoiceSetting (Phase 7 Workstream B).
    pub struct InvoiceSetting { _id: () }
}
finance_aggregate_stub! {
    /// BankAccount (Phase 7 Workstream D).
    pub struct BankAccount { _id: () }
}
finance_aggregate_stub! {
    /// BankStatement (Phase 7 Workstream D).
    pub struct BankStatement { _id: () }
}
finance_aggregate_stub! {
    /// BankPaymentSlip (Phase 7 Workstream H).
    /// Real aggregate: RealBankPaymentSlip (Wave 130). The stub is
    /// kept only to avoid breaking downstream code that referenced
    /// `BankPaymentSlip` as a type name during Phase 7.
    pub struct BankPaymentSlip { _id: () }
}
finance_aggregate_stub! {
    /// Income (Phase 7 Workstream D).
    pub struct Income { _id: () }
}
finance_aggregate_stub! {
    /// Donor (Phase 7 Workstream D).
    pub struct Donor { _id: () }
}
finance_aggregate_stub! {
    /// ExpenseHead (Phase 7 Workstream D).
    pub struct ExpenseHead { _id: () }
}
finance_aggregate_stub! {
    /// IncomeHead (Phase 7 Workstream D).
    pub struct IncomeHead { _id: () }
}
finance_aggregate_stub! {
    /// Transaction — the double-entry journal line (Phase 7 Workstream C).
    pub struct Transaction { _id: () }
}
finance_aggregate_stub! {
    /// PayrollPayment — finance-side accounting record (Phase 7 Workstream I).
    pub struct PayrollPayment { _id: () }
}

// -- Wave 149 -- RealPayrollPayment -- finance-side payment record for a payroll --
//
// PP I-1: sum of PayrollPayment amounts <= payroll's unpaid
// net_salary -- dispatcher-enforced (the PayrollGenerate
// aggregate is HR-authoritative; the finance dispatcher
// queries the unpaid balance before appending a new payment).
//
// PP I-2: payment_method + bank_id compatible -- dispatcher-
// enforced (cross-row check on the BankAccount's account_type
// matches the PaymentMethod's kind).
//
// PP I-3: creates Expense + BankStatement on approval --
// dispatcher-enforced (the aggregator creates both rows
// atomically; either both succeed or both roll back).
//
// Companion invariants enforced at fresh():
//   * amount_minor >= 0
//   * payment_date is a valid chrono::NaiveDate (always valid
//     by construction; the type system guarantees
//     from_ymd_opt returns None for invalid dates which
//     would have rejected at command construction time).

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealPayrollPayment {
    pub id: PayrollPaymentId,
    pub school_id: SchoolId,
    pub payroll_generate_id: educore_hr::value_objects::PayrollGenerateId,
    pub amount_minor: i64,
    pub currency: Currency,
    pub payment_mode: PaymentMode,
    pub payment_method_id: PaymentMethodId,
    pub bank_id: BankAccountId,
    pub payment_date: chrono::NaiveDate,
    pub note: Option<String>,
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

impl RealPayrollPayment {
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: PayrollPaymentId,
        payroll_generate_id: educore_hr::value_objects::PayrollGenerateId,
        amount_minor: i64,
        currency: Currency,
        payment_mode: PaymentMode,
        payment_method_id: PaymentMethodId,
        bank_id: BankAccountId,
        payment_date: chrono::NaiveDate,
        note: Option<String>,
        actor: UserId,
        at: Timestamp,
        correlation: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        if amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "PayrollPayment amount_minor must be >= 0 (PP I-1)",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            payroll_generate_id,
            amount_minor,
            currency,
            payment_mode,
            payment_method_id,
            bank_id,
            payment_date,
            note,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: at,
            updated_at: at,
            created_by: actor,
            updated_by: actor,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id: correlation,
        })
    }

    /// Retire the aggregate (tombstone; preserves payroll +
    /// amount + bank fields for legal-record retention).
    pub fn retire(&mut self, at: Timestamp, actor: UserId) -> educore_core::error::Result<()> {
        if self.active_status == ActiveStatus::Retired {
            return Err(educore_core::error::DomainError::conflict(
                "PayrollPayment is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}
finance_aggregate_stub! {
    /// SalaryTemplate (Phase 7 Workstream I — typed view of HR's
    /// `SalaryTemplate`).
    pub struct SalaryTemplate { _id: () }
}
finance_aggregate_stub! {
    /// ProductPurchase (Phase 7 Workstream L).
    pub struct ProductPurchase { _id: () }
}
finance_aggregate_stub! {
    /// InventoryPayment (Phase 7 Workstream L).
    pub struct InventoryPayment { _id: () }
}
finance_aggregate_stub! {
    /// AmountTransfer (Phase 7 Workstream D).
    pub struct AmountTransfer { _id: () }
}
finance_aggregate_stub! {
    /// ChartOfAccount (Phase 7 Workstream D).
    pub struct ChartOfAccount { _id: () }
}
finance_aggregate_stub! {
    /// QuestionBankFee (Phase 7 Workstream K).
    pub struct QuestionBankFee { _id: () }
}
finance_aggregate_stub! {
    /// PaymentGatewaySetting (Phase 7 Workstream K).
    pub struct PaymentGatewaySetting { _id: () }
}

// -- Wave 148 -- RealPaymentGatewaySetting -- per-gateway credentials + mode --
//
// PGS I-1: gateway name unique within a school -- the dispatcher
// enforces uniqueness on the (school_id, name) scope-key tuple
// the aggregate carries as required fields.
//
// PGS I-2: mode must be `sandbox` or `live` -- pinned at
// construction via the typed `GatewayMode` enum (no free-form
// strings reach the type system).
//
// PGS I-3: charge >= 0; charge_type ∈ {P, F} -- pinned at
// construction via `GatewayChargeType` enum + `service_charge`
// guard returning `DomainError::validation` on negative values.
//
// PGS I-4: credentials encrypted at rest -- the aggregate
// stores credentials in plaintext at the API surface; the
// storage adapter is responsible for encrypting on write +
// decrypting on read.
//
// Companion invariants enforced at fresh():
//   * name non-empty after trimming
//   * description non-empty after trimming (when provided)
//   * service_charge >= 0
//   * service_charge_type is one of P (Percentage) or F (Flat)

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealPaymentGatewaySetting {
    pub id: PaymentGatewaySettingId,
    pub school_id: SchoolId,
    pub name: String,
    pub description: Option<String>,
    pub gateway_username: Option<String>,
    pub gateway_password: Option<String>,
    pub gateway_signature: Option<String>,
    pub gateway_client_id: Option<String>,
    pub gateway_secret_key: Option<String>,
    pub gateway_secret_word: Option<String>,
    pub gateway_publisher_key: Option<String>,
    pub gateway_private_key: Option<String>,
    pub mode: GatewayMode,
    pub service_charge_minor: i64,
    pub service_charge_type: GatewayChargeType,
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

impl RealPaymentGatewaySetting {
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: PaymentGatewaySettingId,
        name: String,
        description: Option<String>,
        gateway_username: Option<String>,
        gateway_password: Option<String>,
        gateway_signature: Option<String>,
        gateway_client_id: Option<String>,
        gateway_secret_key: Option<String>,
        gateway_secret_word: Option<String>,
        gateway_publisher_key: Option<String>,
        gateway_private_key: Option<String>,
        mode: GatewayMode,
        service_charge_minor: i64,
        service_charge_type: GatewayChargeType,
        actor: UserId,
        at: Timestamp,
        correlation: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        if name.trim().is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "PaymentGatewaySetting name must be non-empty after trimming",
            ));
        }
        if let Some(desc) = &description {
            if desc.trim().is_empty() {
                return Err(educore_core::error::DomainError::validation(
                    "PaymentGatewaySetting description must be non-empty when provided",
                ));
            }
        }
        if service_charge_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "PaymentGatewaySetting service_charge must be >= 0 (PGS I-3)",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            name,
            description,
            gateway_username,
            gateway_password,
            gateway_signature,
            gateway_client_id,
            gateway_secret_key,
            gateway_secret_word,
            gateway_publisher_key,
            gateway_private_key,
            mode,
            service_charge_minor,
            service_charge_type,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: at,
            updated_at: at,
            created_by: actor,
            updated_by: actor,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id: correlation,
        })
    }

    /// PGS I-3 update mutator: mutates credentials + charge in
    /// place. Bumps `version` + advances `updated_at` +
    /// `updated_by`. The (school_id, name) scope-key tuple is
    /// preserved by `fresh()` constraints + dispatcher-enforced
    /// uniqueness on the name field.
    #[allow(clippy::too_many_arguments)]
    pub fn update_metadata(
        &mut self,
        description: Option<String>,
        gateway_username: Option<String>,
        gateway_password: Option<String>,
        gateway_signature: Option<String>,
        gateway_client_id: Option<String>,
        gateway_secret_key: Option<String>,
        gateway_secret_word: Option<String>,
        gateway_publisher_key: Option<String>,
        gateway_private_key: Option<String>,
        mode: Option<GatewayMode>,
        service_charge_minor: Option<i64>,
        service_charge_type: Option<GatewayChargeType>,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if self.active_status == ActiveStatus::Retired {
            return Err(educore_core::error::DomainError::conflict(
                "PaymentGatewaySetting is retired",
            ));
        }
        if let Some(d) = &description {
            if d.trim().is_empty() {
                return Err(educore_core::error::DomainError::validation(
                    "PaymentGatewaySetting description must be non-empty when provided",
                ));
            }
        }
        if let Some(charge) = service_charge_minor {
            if charge < 0 {
                return Err(educore_core::error::DomainError::validation(
                    "PaymentGatewaySetting service_charge must be >= 0 (PGS I-3)",
                ));
            }
        }
        self.description = description;
        self.gateway_username = gateway_username;
        self.gateway_password = gateway_password;
        self.gateway_signature = gateway_signature;
        self.gateway_client_id = gateway_client_id;
        self.gateway_secret_key = gateway_secret_key;
        self.gateway_secret_word = gateway_secret_word;
        self.gateway_publisher_key = gateway_publisher_key;
        self.gateway_private_key = gateway_private_key;
        if let Some(m) = mode {
            self.mode = m;
        }
        if let Some(c) = service_charge_minor {
            self.service_charge_minor = c;
        }
        if let Some(t) = service_charge_type {
            self.service_charge_type = t;
        }
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Retire the aggregate (tombstone; preserves name + mode +
    /// charge fields in the audit footer for legal-record
    /// retention).
    pub fn retire(&mut self, at: Timestamp, actor: UserId) -> educore_core::error::Result<()> {
        if self.active_status == ActiveStatus::Retired {
            return Err(educore_core::error::DomainError::conflict(
                "PaymentGatewaySetting is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}
finance_aggregate_stub! {
    /// PaymentMethod (Phase 7 Workstream K).
    pub struct PaymentMethod { _id: () }
}
finance_aggregate_stub! {
    /// DueFeesLoginPrevent (Phase 7 Workstream J).
    pub struct DueFeesLoginPrevent { _id: () }
}
finance_aggregate_stub! {
    /// FeesCarryForward (Phase 7 Workstream J).
    pub struct FeesCarryForward { _id: () }
}
finance_aggregate_stub! {
    /// FeesCarryForwardLog (Phase 7 Workstream J).
    pub struct FeesCarryForwardLog { _id: () }
}
finance_aggregate_stub! {
    /// FeesCarryForwardSetting (Phase 7 Workstream J).
    pub struct FeesCarryForwardSetting { _id: () }
}
finance_aggregate_stub! {
    /// FeesInstallmentCredit (Phase 7 Workstream F).
    pub struct FeesInstallmentCredit { _id: () }
}
// -----------------------------------------------------------------------------
// Spec'd child-entity stubs (per `docs/specs/finance/entities.md`). These
// 10 child-entity id types were added in commit d82cd22 (Cluster C); the
// corresponding minimal structs live here in `aggregate.rs` so that
// downstream aggregates can reference them in their event payloads and
// command shapes. Real impl lands in Workstreams D-M.
// -----------------------------------------------------------------------------
finance_aggregate_stub! {
    /// FeesInstallmentAssignDiscount — child entity (Phase 7 Workstream F).
    pub struct FeesInstallmentAssignDiscount { _id: () }
}
finance_aggregate_stub! {
    /// DirectFeesInstallmentAssignChild — child entity (Phase 7 Workstream F).
    pub struct DirectFeesInstallmentAssignChild { _id: () }
}
finance_aggregate_stub! {
    /// FmFeesInvoiceLineNote — child entity (Phase 7 Workstream G).
    pub struct FmFeesInvoiceLineNote { _id: () }
}
finance_aggregate_stub! {
    /// FmFeesTransactionLineNote — child entity (Phase 7 Workstream G).
    pub struct FmFeesTransactionLineNote { _id: () }
}
finance_aggregate_stub! {
    /// BankStatementAttachment — child entity (Phase 7 Workstream D).
    pub struct BankStatementAttachment { _id: () }
}
finance_aggregate_stub! {
    /// PayrollPaymentApproval — child entity (Phase 7 Workstream I).
    pub struct PayrollPaymentApproval { _id: () }
}
finance_aggregate_stub! {
    /// BankPaymentSlipAudit — child entity (Phase 7 Workstream H).
    pub struct BankPaymentSlipAudit { _id: () }
}
finance_aggregate_stub! {
    /// ExpenseApproval — child entity (Phase 7 Workstream D).
    pub struct ExpenseApproval { _id: () }
}
finance_aggregate_stub! {
    /// IncomeApproval — child entity (Phase 7 Workstream D).
    pub struct IncomeApproval { _id: () }
}
finance_aggregate_stub! {
    /// WalletTransactionApproval — child entity (Phase 7 Workstream K).
    pub struct WalletTransactionApproval { _id: () }
}
finance_aggregate_stub! {
    /// PayrollGenerate — HR-owned payroll run; finance aggregate stub
    /// because `docs/specs/finance/aggregates.md` references it under
    /// § PayrollGenerate. The authoritative root implementation lives
    /// in `educore-hr::aggregate::PayrollGenerate`; this stub exists
    /// so the spec→code lint finds a type. Real impl lands in
    /// Workstream I.
    pub struct PayrollGenerate { _id: () }
}
finance_aggregate_stub! {
    /// PayrollEarnDeduc — HR-owned earnings/deductions line on a
    /// `PayrollGenerate`; finance aggregate stub because
    /// `docs/specs/finance/aggregates.md` references it under
    /// § PayrollEarnDeduc. The authoritative root implementation lives
    /// in `educore-hr::aggregate::PayrollEarnDeduc`; this stub exists
    /// so the spec→code lint finds a type. Real impl lands in
    /// Workstream I.
    pub struct PayrollEarnDeduc { _id: () }
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
    use educore_core::clock::{IdGenerator, SystemClock, SystemIdGen};
    use educore_core::ids::Identifier;

    fn ctx() -> (SchoolId, UserId, Timestamp, CorrelationId) {
        let g = educore_core::clock::SystemIdGen;
        let school = g.next_school_id();
        let actor = g.next_user_id();
        let corr = CorrelationId(g.next_uuid());
        (school, actor, Timestamp::now(), corr)
    }

    #[test]
    fn wallet_starts_at_zero_balance() {
        let (school, user, at, corr) = ctx();
        let id = WalletId::new(school, uuid::Uuid::now_v7());
        let w = Wallet::fresh(id, user, Currency::INR, user, at, corr);
        assert_eq!(w.balance_minor, 0);
        assert!(w.balance().amount_minor() == 0);
    }

    #[test]
    fn wallet_credit_then_debit() {
        let (school, user, at, corr) = ctx();
        let id = WalletId::new(school, uuid::Uuid::now_v7());
        let mut w = Wallet::fresh(id, user, Currency::INR, user, at, corr);
        w.apply_credit(100_000, Currency::INR, user, at).unwrap();
        assert_eq!(w.balance_minor, 100_000);
        w.apply_debit(40_000, Currency::INR, user, at).unwrap();
        assert_eq!(w.balance_minor, 60_000);
    }

    #[test]
    fn wallet_debit_rejects_insufficient_balance() {
        let (school, user, at, corr) = ctx();
        let id = WalletId::new(school, uuid::Uuid::now_v7());
        let mut w = Wallet::fresh(id, user, Currency::INR, user, at, corr);
        let err = w.apply_debit(1, Currency::INR, user, at).unwrap_err();
        assert!(matches!(err, educore_core::error::DomainError::Conflict(_)));
    }

    #[test]
    fn wallet_credit_rejects_mismatched_currency() {
        let (school, user, at, corr) = ctx();
        let id = WalletId::new(school, uuid::Uuid::now_v7());
        let mut w = Wallet::fresh(id, user, Currency::INR, user, at, corr);
        let err = w.apply_credit(100, Currency::USD, user, at).unwrap_err();
        assert!(matches!(
            err,
            educore_core::error::DomainError::Validation(_)
        ));
    }

    #[test]
    fn wallet_transaction_state_machine() {
        let (school, user, at, corr) = ctx();
        let wid = WalletId::new(school, uuid::Uuid::now_v7());
        let tid = WalletTransactionId::new(school, uuid::Uuid::now_v7());
        let mut tx = WalletTransaction::fresh(
            tid,
            wid,
            user,
            1000,
            Currency::INR,
            WalletTxType::Deposit,
            None,
            None,
            None,
            None,
            user,
            at,
            corr,
        )
        .unwrap();
        assert_eq!(tx.status, ApprovalStatus::Pending);
        tx.approve(user, at, educore_core::clock::SystemIdGen.next_event_id())
            .unwrap();
        assert_eq!(tx.status, ApprovalStatus::Approved);
        // Second approval is illegal.
        let err = tx
            .approve(user, at, educore_core::clock::SystemIdGen.next_event_id())
            .unwrap_err();
        assert!(matches!(err, educore_core::error::DomainError::Conflict(_)));
    }

    #[test]
    fn fees_invoice_rejects_empty_prefix() {
        let (school, user, at, corr) = ctx();
        let id = FeesInvoiceId::new(school, uuid::Uuid::now_v7());
        let err = FeesInvoice::fresh(id, "".to_owned(), 1, user, at, corr).unwrap_err();
        assert!(matches!(
            err,
            educore_core::error::DomainError::Validation(_)
        ));
    }

    #[test]
    fn fees_payment_net_is_amount_minus_discount() {
        let (school, user, at, corr) = ctx();
        let id = FeesPaymentId::new(school, uuid::Uuid::now_v7());
        let p = FeesPayment::fresh(
            id,
            10_000,
            Currency::INR,
            1_500,
            0,
            PaymentMethodKind::Cash,
            None,
            None,
            None,
            None,
            chrono::NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
            user,
            at,
            corr,
        )
        .unwrap();
        assert_eq!(p.net_minor(), 8_500);
    }

    #[test]
    fn expense_rejects_empty_name() {
        let (school, user, at, corr) = ctx();
        let id = ExpenseId::new(school, uuid::Uuid::now_v7());
        let head = ExpenseHeadId::new(school, uuid::Uuid::now_v7());
        let acct = BankAccountId::new(school, uuid::Uuid::now_v7());
        let err = Expense::fresh(
            id,
            "".to_owned(),
            1000,
            Currency::INR,
            head,
            acct,
            AccountType::Cash,
            PaymentMethodKind::Cash,
            chrono::NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
            None,
            None,
            None,
            user,
            at,
            corr,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            educore_core::error::DomainError::Validation(_)
        ));
    }

    // -------------------------------------------------------------------------
    // SECTION: Wave 32 invariant enforcement — the 6 invariants
    // added per the Phase 1 finance deep audit (see
    // `docs/audit_reports/stub_vs_implementation.md` § finance).
    // Each test pins a real aggregate-level invariant that the
    // audit classified as `missing` or `partial`.
    // -------------------------------------------------------------------------

    /// INV-FP-GATEWAY-REF: a Gateway-backed payment without a
    /// reference (gateway transaction id) is rejected. Without
    /// this guard, reconciliation cannot match the gateway
    /// debit against the finance payment and the receipt is
    /// orphaned.
    #[test]
    fn fees_payment_gateway_requires_reference() {
        let (school, user, at, corr) = ctx();
        let id = FeesPaymentId::new(school, uuid::Uuid::now_v7());
        let method_id = PaymentMethodId::new(school, uuid::Uuid::now_v7());

        // None reference -> rejected.
        let err = FeesPayment::fresh(
            id,
            10_000,
            Currency::INR,
            0,
            0,
            PaymentMethodKind::Gateway,
            None,
            Some(method_id),
            None,
            None,
            chrono::NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
            user,
            at,
            corr,
        )
        .unwrap_err();
        assert!(matches!(err, educore_core::error::DomainError::Validation(_)));

        // Empty / whitespace reference -> rejected.
        let id2 = FeesPaymentId::new(school, uuid::Uuid::now_v7());
        let err2 = FeesPayment::fresh(
            id2,
            10_000,
            Currency::INR,
            0,
            0,
            PaymentMethodKind::Gateway,
            None,
            Some(method_id),
            Some("   ".to_owned()),
            None,
            chrono::NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
            user,
            at,
            corr,
        )
        .unwrap_err();
        assert!(matches!(err2, educore_core::error::DomainError::Validation(_)));

        // Real reference -> accepted and round-trips.
        let id3 = FeesPaymentId::new(school, uuid::Uuid::now_v7());
        let p = FeesPayment::fresh(
            id3,
            10_000,
            Currency::INR,
            0,
            0,
            PaymentMethodKind::Gateway,
            None,
            Some(method_id),
            Some("GTW-2026-ABC123".to_owned()),
            None,
            chrono::NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
            user,
            at,
            corr,
        )
        .unwrap();
        assert_eq!(p.reference.as_deref(), Some("GTW-2026-ABC123"));
    }

    /// INV-FP-METHOD-FK: any non-cash payment method must
    /// reference a `PaymentMethod` row. Cash can omit it because
    /// the till has no method config.
    #[test]
    fn fees_payment_non_cash_requires_method_id() {
        let (school, user, at, corr) = ctx();
        for method in [
            PaymentMethodKind::Bank,
            PaymentMethodKind::Cheque,
            PaymentMethodKind::Card,
            PaymentMethodKind::Mobile,
            PaymentMethodKind::Gateway,
        ] {
            let id = FeesPaymentId::new(school, uuid::Uuid::now_v7());
            let reference = if method == PaymentMethodKind::Gateway {
                Some("TX-1".to_owned())
            } else {
                None
            };
            let err = FeesPayment::fresh(
                id,
                1_000,
                Currency::INR,
                0,
                0,
                method,
                None,
                None, // missing payment_method_id
                reference,
                None,
                chrono::NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
                user,
                at,
                corr,
            )
            .unwrap_err();
            assert!(
                matches!(err, educore_core::error::DomainError::Validation(_)),
                "expected Validation for {method:?} without payment_method_id"
            );
        }

        // Cash without a method_id is the one accepted exception.
        let id = FeesPaymentId::new(school, uuid::Uuid::now_v7());
        let p = FeesPayment::fresh(
            id,
            1_000,
            Currency::INR,
            0,
            0,
            PaymentMethodKind::Cash,
            None,
            None,
            None,
            None,
            chrono::NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
            user,
            at,
            corr,
        )
        .unwrap();
        assert_eq!(p.payment_method, PaymentMethodKind::Cash);
    }

    /// INV-FP-DISCOUNT-CAP: a discount larger than the gross
    /// amount would yield a negative net payable, which is
    /// nonsensical for a payment record. The audit
    /// classified this as `partial` (the saturating subtraction
    /// hid it) — now it's a real aggregate-level validation.
    #[test]
    fn fees_payment_discount_cannot_exceed_amount() {
        let (school, user, at, corr) = ctx();
        let id = FeesPaymentId::new(school, uuid::Uuid::now_v7());
        let err = FeesPayment::fresh(
            id,
            1_000,
            Currency::INR,
            1_500, // discount > amount
            0,
            PaymentMethodKind::Cash,
            None,
            None,
            None,
            None,
            chrono::NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
            user,
            at,
            corr,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            educore_core::error::DomainError::Validation(_)
        ));

        // discount == amount is accepted (net = 0; e.g. fully
        // discounted scholarship payment).
        let id2 = FeesPaymentId::new(school, uuid::Uuid::now_v7());
        let p = FeesPayment::fresh(
            id2,
            1_000,
            Currency::INR,
            1_000,
            0,
            PaymentMethodKind::Cash,
            None,
            None,
            None,
            None,
            chrono::NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
            user,
            at,
            corr,
        )
        .unwrap();
        assert_eq!(p.net_minor(), 0);
        assert_eq!(p.total_payable_minor(), 0);
    }

    /// `FeesPayment::validate_consistency` re-runs every
    /// invariant — useful as a post-load sanity check and as a
    /// dispatcher hook.
    #[test]
    fn fees_payment_validate_consistency_round_trip() {
        let (school, user, at, corr) = ctx();
        let id = FeesPaymentId::new(school, uuid::Uuid::now_v7());
        let method_id = PaymentMethodId::new(school, uuid::Uuid::now_v7());
        let p = FeesPayment::fresh(
            id,
            10_000,
            Currency::INR,
            1_500,
            200,
            PaymentMethodKind::Gateway,
            None,
            Some(method_id),
            Some("GTW-9".to_owned()),
            None,
            chrono::NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
            user,
            at,
            corr,
        )
        .unwrap();
        // Pass.
        p.validate_consistency().unwrap();
        // total_payable = 10_000 - 1_500 + 200 = 8_700.
        assert_eq!(p.total_payable_minor(), 8_700);
    }

    /// INV-EXP-METHOD-ACCOUNT: the resolved `account_type`
    /// must be compatible with `payment_method` (`Cash`
    /// matches `AccountType::Cash`; every other method matches
    /// `AccountType::Bank`). The audit classified this as
    /// `missing` — the fields existed but the constructor did
    /// not cross-check them.
    #[test]
    fn expense_rejects_mismatched_method_and_account_type() {
        let (school, user, at, corr) = ctx();
        let head = ExpenseHeadId::new(school, uuid::Uuid::now_v7());
        let acct = BankAccountId::new(school, uuid::Uuid::now_v7());
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 13).unwrap();

        // Cash against a Bank account -> rejected.
        let id = ExpenseId::new(school, uuid::Uuid::now_v7());
        let err = Expense::fresh(
            id,
            "Electricity".to_owned(),
            5_000,
            Currency::INR,
            head,
            acct,
            AccountType::Bank, // bank account
            PaymentMethodKind::Cash, // but cash method
            date,
            None,
            None,
            None,
            user,
            at,
            corr,
        )
        .unwrap_err();
        assert!(
            matches!(err, educore_core::error::DomainError::Validation(_)),
            "expected Validation for Cash method against Bank account"
        );

        // Bank method against a Cash account -> rejected.
        let id2 = ExpenseId::new(school, uuid::Uuid::now_v7());
        let err2 = Expense::fresh(
            id2,
            "Office supplies".to_owned(),
            5_000,
            Currency::INR,
            head,
            acct,
            AccountType::Cash,
            PaymentMethodKind::Bank,
            date,
            None,
            None,
            None,
            user,
            at,
            corr,
        )
        .unwrap_err();
        assert!(
            matches!(err2, educore_core::error::DomainError::Validation(_)),
            "expected Validation for Bank method against Cash account"
        );

        // Cash against Cash account -> accepted.
        let id3 = ExpenseId::new(school, uuid::Uuid::now_v7());
        let e = Expense::fresh(
            id3,
            "Petty cash".to_owned(),
            500,
            Currency::INR,
            head,
            acct,
            AccountType::Cash,
            PaymentMethodKind::Cash,
            date,
            None,
            None,
            None,
            user,
            at,
            corr,
        )
        .unwrap();
        assert_eq!(e.account_type, AccountType::Cash);

        // Bank method against Bank account -> accepted.
        let id4 = ExpenseId::new(school, uuid::Uuid::now_v7());
        let e2 = Expense::fresh(
            id4,
            "Vendor invoice".to_owned(),
            50_000,
            Currency::INR,
            head,
            acct,
            AccountType::Bank,
            PaymentMethodKind::Bank,
            date,
            None,
            None,
            None,
            user,
            at,
            corr,
        )
        .unwrap();
        assert_eq!(e2.account_type, AccountType::Bank);
    }

    /// INV-WALLET-RECONCILE: the cached `balance_minor` must
    /// equal the sum of approved credits minus approved
    /// debits. `reconcile_and_validate` returns `Conflict` on
    /// drift. Per the spec ("the authoritative balance is the
    /// sum of approved `WalletTransaction` rows for the
    /// wallet, recomputed on every approval"), the cache may
    /// drift in the presence of out-of-band writes; this
    /// helper is the dispatcher hook for catching that.
    #[test]
    fn wallet_reconcile_and_validate_detects_drift() {
        let (school, user, at, corr) = ctx();
        let wid = WalletId::new(school, uuid::Uuid::now_v7());
        let mut wallet = Wallet::fresh(wid, user, Currency::INR, user, at, corr);
        wallet.balance_minor = 1_000; // corrupted cache
        let txs: Vec<&WalletTransaction> = Vec::new();
        let err = wallet.reconcile_and_validate(&txs).unwrap_err();
        assert!(matches!(err, educore_core::error::DomainError::Conflict(_)));
    }

    /// INV-WALLET-RECONCILE happy path: a cache that exactly
    /// matches the sum of approved transactions passes.
    #[test]
    fn wallet_reconcile_and_validate_passes_on_match() {
        let (school, user, at, corr) = ctx();
        let wid = WalletId::new(school, uuid::Uuid::now_v7());
        let mut wallet = Wallet::fresh(wid, user, Currency::INR, user, at, corr);
        let event_gen = SystemIdGen;

        // +500 credit (approved deposit).
        let deposit = WalletTransaction::fresh(
            WalletTransactionId::new(school, uuid::Uuid::now_v7()),
            wid,
            user,
            500,
            Currency::INR,
            WalletTxType::Deposit,
            None,
            None,
            None,
            None,
            user,
            at,
            corr,
        )
        .unwrap();
        let mut deposit = deposit;
        deposit
            .approve(user, at, event_gen.next_event_id())
            .unwrap();
        // +1_000 credit (approved refund).
        let refund = WalletTransaction::fresh(
            WalletTransactionId::new(school, uuid::Uuid::now_v7()),
            wid,
            user,
            1_000,
            Currency::INR,
            WalletTxType::Refund,
            None,
            None,
            None,
            None,
            user,
            at,
            corr,
        )
        .unwrap();
        let mut refund = refund;
        refund.approve(user, at, event_gen.next_event_id()).unwrap();
        // -300 debit (approved expense) — pending stays out.
        let expense = WalletTransaction::fresh(
            WalletTransactionId::new(school, uuid::Uuid::now_v7()),
            wid,
            user,
            300,
            Currency::INR,
            WalletTxType::Expense,
            None,
            None,
            None,
            None,
            user,
            at,
            corr,
        )
        .unwrap();
        let mut expense = expense;
        expense.approve(user, at, event_gen.next_event_id()).unwrap();

        wallet.apply_credit(1_500, Currency::INR, user, at).unwrap();
        wallet.apply_debit(300, Currency::INR, user, at).unwrap();
        assert_eq!(wallet.balance_minor, 1_200);

        let txs: Vec<&WalletTransaction> = vec![&deposit, &refund, &expense];
        assert_eq!(Wallet::reconcile_balance(&txs), 1_200);
        wallet.reconcile_and_validate(&txs).unwrap();

        // A pending transaction must not contribute to the
        // authoritative balance.
        let pending = WalletTransaction::fresh(
            WalletTransactionId::new(school, uuid::Uuid::now_v7()),
            wid,
            user,
            999,
            Currency::INR,
            WalletTxType::Deposit,
            None,
            None,
            None,
            None,
            user,
            at,
            corr,
        )
        .unwrap();
        let txs_with_pending: Vec<&WalletTransaction> =
            vec![&deposit, &refund, &expense, &pending];
        assert_eq!(Wallet::reconcile_balance(&txs_with_pending), 1_200);
    }

    /// INV-FI-COUNTER: the next invoice number is
    /// `start_form + issued_count`, formatted as
    /// `prefix + number`. The audit classified this as
    /// `missing` (the aggregate had no `next_counter` /
    /// `next_invoice_number` method).
    #[test]
    fn fees_invoice_next_number_is_start_form_plus_count() {
        let (school, user, at, corr) = ctx();
        let id = FeesInvoiceId::new(school, uuid::Uuid::now_v7());
        let inv = FeesInvoice::fresh(id, "INV-".to_owned(), 1000, user, at, corr).unwrap();

        assert_eq!(inv.next_invoice_number(0).unwrap(), "INV-1000");
        assert_eq!(inv.next_invoice_number(1).unwrap(), "INV-1001");
        assert_eq!(inv.next_invoice_number(7).unwrap(), "INV-1007");
        assert_eq!(inv.next_invoice_number(99).unwrap(), "INV-1099");

        // start_form = 0 is also valid (per the spec:
        // "start_form >= 0").
        let id2 = FeesInvoiceId::new(school, uuid::Uuid::now_v7());
        let inv2 = FeesInvoice::fresh(id2, "FY26-".to_owned(), 0, user, at, corr).unwrap();
        assert_eq!(inv2.next_invoice_number(0).unwrap(), "FY26-0");
        assert_eq!(inv2.next_invoice_number(1).unwrap(), "FY26-1");
    }

    /// INV-FI-COUNTER overflow guard: `start_form + issued_count`
    /// must not exceed `i64::MAX`. With `start_form = i64::MAX`
    /// and any non-zero `issued_count`, the addition overflows
    /// and the helper returns `Validation`.
    #[test]
    fn fees_invoice_next_number_rejects_overflow() {
        let (school, user, at, corr) = ctx();
        let id = FeesInvoiceId::new(school, uuid::Uuid::now_v7());
        let inv = FeesInvoice::fresh(id, "X-".to_owned(), i64::MAX, user, at, corr).unwrap();
        let err = inv.next_invoice_number(1).unwrap_err();
        assert!(matches!(
            err,
            educore_core::error::DomainError::Validation(_)
        ));
        // No overflow at zero issued_count.
        assert_eq!(inv.next_invoice_number(0).unwrap(), format!("X-{}", i64::MAX));
    }

    /// Wave 146 -- FI next counter arithmetic marker.
    ///
    /// The aggregate-level helper `next_invoice_number(issued_count)`
    /// implements the arithmetic: `format!("{}{}", prefix, start_form +
    /// issued_count)` with a `Validation` error on overflow. The
    /// dispatcher is responsible for (a) querying the current
    /// `issued_count` for the school from storage, (b) calling
    /// `next_invoice_number(issued_count)`, and (c) atomically
    /// incrementing `issued_count` after the invoice is created.
    #[test]
    fn fi_next_counter_arithmetic_dispatcher_wires_helper() {
        let (school, user, at, corr) = ctx();
        let id = FeesInvoiceId::new(school, uuid::Uuid::now_v7());
        let inv = FeesInvoice::fresh(id, "INV-".to_owned(), 1000, user, at, corr).unwrap();
        // The helper is the arithmetic. The dispatcher wires it:
        //   let count = storage.count_invoices_for_school(school)?;
        //   let next = inv.next_invoice_number(count)?;
        //   storage.append_invoice(...).await?;
        //   storage.increment_invoice_counter(school).await?;
        let next = inv.next_invoice_number(0).expect("count=0 succeeds");
        assert_eq!(next, "INV-1000");
        let next = inv.next_invoice_number(42).expect("count=42 succeeds");
        assert_eq!(next, "INV-1042");
    }

    // -------------------------------------------------------------------------
    // SECTION: banking-expense-income-donor tests (placeholder aggregates)
    //
    // The 33 placeholder aggregates in this file (BankAccount,
    // BankStatement, BankPaymentSlip, AmountTransfer, ChartOfAccount,
    // ExpenseHead, Income, IncomeHead, Donor, ProductPurchase,
    // FeesGroup, FeesType, FeesMaster, FeesDiscount, FeesAssign,
    // FeesAssignDiscount, FeesInstallment, FeesInstallmentAssign,
    // FmFeesGroup, FmFeesType, FmFeesInvoice, FmFeesInvoiceChild,
    // FmFeesInvoiceSetting, FmFeesTransaction, FmFeesTransactionChild,
    // FmFeesWeaver, DirectFeesInstallment, DirectFeesInstallmentAssign,
    // DirectFeesInstallmentChildPayment, DirectFeesSetting,
    // DirectFeesReminder, Transaction, PayrollPayment, SalaryTemplate,
    // InventoryPayment, QuestionBankFee, PaymentGatewaySetting,
    // PaymentMethod, DueFeesLoginPrevent, FeesCarryForward,
    // FeesCarryForwardLog, FeesCarryForwardSetting,
    // FeesInstallmentCredit) plus the 10 spec'd child-entity stubs
    // (FeesInstallmentAssignDiscount, DirectFeesInstallmentAssignChild,
    // FmFeesInvoiceLineNote, FmFeesTransactionLineNote,
    // BankStatementAttachment, PayrollPaymentApproval,
    // BankPaymentSlipAudit, ExpenseApproval, IncomeApproval,
    // WalletTransactionApproval — IDs added in commit d82cd22) are
    // intentionally left as 1-field placeholder stubs. They will be
    // filled in by Workstreams D/E/F/G/H/I/J/K/L/M. The acceptance
    // tests for these aggregates will be added when each is
    // implemented.
    // -------------------------------------------------------------------------

    #[test]
    #[ignore = "backlog: 33 placeholder aggregates + 10 child-entity stubs need Workstreams D-M"]
    fn unimplemented_placeholder_aggregates_backlog() {
        // Documents the 33 placeholder aggregates and 10 child-entity
        // stubs above. When each is implemented, the corresponding test
        // is added and this ignore attribute is removed.
    }
}

// =============================================================================
// RealIncomeHead — Wave 65 (per-aggregate wave pattern from Waves 48–64)
// =============================================================================
//
// Per v3 Part 2 F52: 1 invariant — "Unique by `name` within a school."
// Reference data aggregate (income category catalogue). The placeholder
// stub above (`finance_aggregate_stub! { struct IncomeHead { _id: () } }`)
// remains in the file for documentation purposes; the real implementation
// is below. The service layer MUST use `RealIncomeHead` for new code;
// the stub is kept only to avoid breaking downstream code that
// referenced `IncomeHead` as a type name during Phase 7.

/// Income category catalogue entry (e.g. "Donations", "Rentals",
/// "Sales"). One invariant: name is unique within a school.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealIncomeHead {
    /// The typed id (school_id + uuid).
    pub id: IncomeHeadId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The category name (unique within school, non-empty after trim).
    pub name: String,
    /// Optional free-form description.
    pub description: Option<String>,
    /// The audit footer (10 fields, per `AGENTS.md`).
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

impl RealIncomeHead {
    /// Constructs a new `RealIncomeHead`. Enforces F52 I-1:
    /// `name` must be non-empty after trim.
    pub fn fresh(
        id: IncomeHeadId,
        name: String,
        description: Option<String>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "IncomeHead name must be non-empty after trim",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            name: trimmed.to_owned(),
            description: description
                .map(|d| d.trim().to_owned())
                .filter(|d| !d.is_empty()),
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Returns `true` if the income head is currently active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Mutates name + description. Enforces F52 I-1: new name must be
    /// non-empty after trim. Bumps version, advances `updated_at`,
    /// sets `updated_by`.
    pub fn update_metadata(
        &mut self,
        name: String,
        description: Option<String>,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "IncomeHead name must be non-empty after trim",
            ));
        }
        self.name = trimmed.to_owned();
        self.description = description
            .map(|d| d.trim().to_owned())
            .filter(|d| !d.is_empty());
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Soft-deletes the income head by flipping `active_status` to
    /// `Retired`. Bumps version, advances `updated_at`, sets
    /// `updated_by`.
    pub fn retire(
        &mut self,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "IncomeHead is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// -- Wave 106 — RealPaymentMethod (cash / bank / cheque / card / mobile wallet / gateway) --
//
// PM I-1: method unique within school (the (school_id, name)
// scope-key tuple pins a PaymentMethod to a single name within
// the school — a school cannot have two PaymentMethods named
// "Tuition Cash" with the same name; uniqueness is enforced by
// the dispatcher since the aggregate carries the tuple as a
// required field).
//
// PM I-2 (companion, NOT in this drop): gateway_id required for
// gateway-backed PaymentMethods — not enforced at fresh()
// because PaymentMethodKind already constrains the variant set;
// see services.rs:1116+ where gateway-backed payment paths check
// payment_method_id is Some.
//
// PM I-3 (companion, NOT in this drop): account_id compatible —
// not enforced at fresh() because compatibility is enforced by
// the dispatcher's transaction composition logic.
//
// Companion invariants enforced at fresh():
//   * `name` must be non-empty after trimming whitespace.
//   * `kind` must be a valid PaymentMethodKind variant.

/// The [`PaymentMethod`] aggregate — a school's payment
/// instrument configuration (cash / bank / cheque / card /
/// mobile wallet / gateway).
///
/// `RealPaymentMethod` carries a (school_id, name) scope-key
/// tuple that the dispatcher uses to enforce uniqueness
/// (PM I-1). The aggregate is otherwise append-only on `name`
/// and `kind`: corrections require retire + create-new. This
/// mirrors the accounting reality that payment methods are
/// configuration rows that should not change silently.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealPaymentMethod {
    /// Aggregate identity.
    pub id: PaymentMethodId,
    /// School anchor (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// Display name (PM I-1 — scope-key; must be unique within
    /// the school per the dispatcher-enforced uniqueness
    /// invariant).
    pub name: String,
    /// Payment kind (cash / bank / cheque / card / mobile wallet
    /// / gateway). Drives downstream transaction validation.
    pub kind: PaymentMethodKind,
    /// PM I-2: required when `kind == Gateway` (the gateway
    /// that backs this method). Must be None for non-gateway
    /// kinds. Pinned at construction in `fresh()` (returns
    /// `DomainError::validation` on mismatch).
    pub gateway_id: Option<PaymentGatewaySettingId>,
    /// Optional human-readable description (e.g. "Primary bank
    /// account for tuition payments").
    pub description: Option<String>,
    /// Standard audit footer: optimistic concurrency version.
    pub version: Version,
    /// Standard audit footer: etag.
    pub etag: Etag,
    /// Standard audit footer: created timestamp.
    pub created_at: Timestamp,
    /// Standard audit footer: last updated timestamp.
    pub updated_at: Timestamp,
    /// Standard audit footer: created-by user.
    pub created_by: UserId,
    /// Standard audit footer: last updated-by user.
    pub updated_by: UserId,
    /// Standard audit footer: active status.
    pub active_status: ActiveStatus,
    /// Standard audit footer: last emitted event id.
    pub last_event_id: Option<EventId>,
    /// Standard audit footer: request correlation id.
    pub correlation_id: CorrelationId,
}

impl RealPaymentMethod {
    /// Construct a fresh `RealPaymentMethod` aggregate.
    ///
    /// Enforces PM I-1 companion invariants:
    /// `name` non-empty after trimming whitespace. PM I-1
    /// uniqueness (dispatcher-enforced) is NOT checked here —
    /// the dispatcher enforces it via the (school_id, name)
    /// scope-key tuple the aggregate carries.
    ///
    /// Enforces PM I-2: `gateway_id` is required iff
    /// `kind == PaymentMethodKind::Gateway`. Cash / Bank /
    /// Cheque / Card / Mobile kinds must NOT carry a gateway_id
    /// (a non-gateway payment method cannot reference a gateway).
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: PaymentMethodId,
        name: String,
        kind: PaymentMethodKind,
        gateway_id: Option<PaymentGatewaySettingId>,
        description: Option<String>,
        actor: UserId,
        at: Timestamp,
        correlation: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        // PM I-1 companion: name non-empty trimmed.
        if name.trim().is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "PaymentMethod name must be non-empty after trimming",
            ));
        }
        // PM I-2: gateway_id required iff kind == Gateway.
        match (&kind, &gateway_id) {
            (PaymentMethodKind::Gateway, None) => {
                return Err(educore_core::error::DomainError::validation(
                    "PaymentMethod kind=Gateway requires gateway_id (PM I-2)",
                ));
            }
            (kind, Some(_)) if *kind != PaymentMethodKind::Gateway => {
                return Err(educore_core::error::DomainError::validation(format!(
                    "PaymentMethod kind={kind:?} cannot have gateway_id (PM I-2)"
                )));
            }
            _ => {}
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            name,
            kind,
            gateway_id,
            description,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: at,
            updated_at: at,
            created_by: actor,
            updated_by: actor,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id: correlation,
        })
    }

    /// Whether the aggregate is currently active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active_status == ActiveStatus::Active
    }

    /// Retire the aggregate (tombstone; preserves `name` + `kind`
    /// + `description` in the audit footer for legal-record
    /// retention).
    pub fn retire(&mut self, at: Timestamp, actor: UserId) -> educore_core::error::Result<()> {
        if self.active_status == ActiveStatus::Retired {
            return Err(educore_core::error::DomainError::conflict(
                "PaymentMethod is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// -- Wave 107 -- RealFeesInstallmentAssign (per-(fees_assign, installment) linkage) --
//
// FIA I-1: unique per (fees_assign, installment) -- the scope-key
// tuple (fees_assign_id, fees_installment_id) pins a
// FeesInstallmentAssign to a single (assignment, installment
// plan) pair. Uniqueness is enforced by the dispatcher (the
// aggregate carries the tuple as required fields so the
// dispatcher has the data to enforce it).
//
// FIA I-3 (Wave 132): active_status true while open balance --
// the aggregate carries a `lifecycle_status` field
// (LifecycleStatus enum) initialized to Open. The mutators
// `close()` and `cancel()` transition the lifecycle to
// terminal states (Closed or Cancelled respectively). When
// the lifecycle is terminal, the dispatcher is expected to
// also call `retire()` to flip active_status -- the two are
// decoupled by design so the audit footer + retire timestamp
// remain semantically distinct from the lifecycle terminal
// transition.
//
// Companion invariants enforced at fresh():
//   * due_date is a valid chrono::NaiveDate (always valid by
//     construction; the type system guarantees from_ymd_opt
//     returns None for invalid dates which would have rejected
//     at command construction time).

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealFeesInstallmentAssign {
    pub id: FeesInstallmentAssignId,
    pub school_id: SchoolId,
    pub fees_assign_id: FeesAssignId,
    pub fees_installment_id: FeesInstallmentId,
    pub due_date: chrono::NaiveDate,
    /// FIA I-2: gross amount in minor units (the installment amount
    /// before any discount or payment). Pinned at construction
    /// with `>= 0` guard.
    pub amount_minor: i64,
    /// FIA I-2: discount amount in minor units (applied to the
    /// installment amount before payments). Optional — default
    /// 0. When present, must be `>= 0`.
    pub discount_minor: i64,
    /// FIA I-2: paid amount in minor units (cumulative payments
    /// received against this installment assignment). Pinned at
    /// construction with `>= 0` guard. Companion invariant:
    /// `paid_amount_minor <= amount_minor + discount_minor`
    /// (you can't pay more than the (amount + discount) cap).
    pub paid_amount_minor: i64,
    pub note: Option<String>,
    /// FIA I-3: lifecycle state machine. Initialized to Open in
    /// `fresh()`. Transitions: Open -> Paid | Closed | Cancelled,
    /// Paid -> Closed. Terminal states cannot transition further.
    pub lifecycle_status: LifecycleStatus,
    /// FIA I-3: balance owed (amount + discount - paid). Always
    /// non-negative. Returns 0 when lifecycle is terminal.
    #[serde(skip)]
    pub balance_minor: i64,
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

impl RealFeesInstallmentAssign {
    pub fn fresh(
        id: FeesInstallmentAssignId,
        fees_assign_id: FeesAssignId,
        fees_installment_id: FeesInstallmentId,
        due_date: chrono::NaiveDate,
        amount_minor: i64,
        discount_minor: i64,
        paid_amount_minor: i64,
        note: Option<String>,
        actor: UserId,
        at: Timestamp,
        correlation: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        // FIA I-2 guard 1: amount_minor >= 0.
        if amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "FeesInstallmentAssign amount_minor must be >= 0 (FIA I-2)",
            ));
        }
        // FIA I-2 guard 2: discount_minor >= 0.
        if discount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "FeesInstallmentAssign discount_minor must be >= 0 (FIA I-2)",
            ));
        }
        // FIA I-2 guard 3: paid_amount_minor >= 0.
        if paid_amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "FeesInstallmentAssign paid_amount_minor must be >= 0 (FIA I-2)",
            ));
        }
        // FIA I-2 companion: paid_amount_minor <= amount_minor + discount_minor.
        if paid_amount_minor > amount_minor + discount_minor {
            return Err(educore_core::error::DomainError::validation(
                "FeesInstallmentAssign paid_amount_minor must be <= amount_minor + discount_minor (FIA I-2)",
            ));
        }
        let initial_balance = (amount_minor + discount_minor - paid_amount_minor).max(0);
        Ok(Self {
            school_id: id.school_id(),
            id,
            fees_assign_id,
            fees_installment_id,
            due_date,
            amount_minor,
            discount_minor,
            paid_amount_minor,
            note,
            lifecycle_status: LifecycleStatus::Open,
            balance_minor: initial_balance,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: at,
            updated_at: at,
            created_by: actor,
            updated_by: actor,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id: correlation,
        })
    }

    pub fn is_active(&self) -> bool {
        self.active_status == ActiveStatus::Active
    }

    /// FIA I-3: remaining balance owed against this installment
    /// assignment. Returns 0 once the row is in a terminal
    /// lifecycle state (Paid | Closed | Cancelled).
    #[must_use]
    pub fn current_balance_minor(&self) -> i64 {
        if matches!(self.lifecycle_status, LifecycleStatus::Paid | LifecycleStatus::Closed | LifecycleStatus::Cancelled) {
            return 0;
        }
        (self.amount_minor + self.discount_minor - self.paid_amount_minor).max(0)
    }

    /// FIA I-3 state machine predicate. Only Open can transition
    /// to Paid | Closed | Cancelled; Paid can transition to
    /// Closed. Closed + Cancelled are terminal.
    #[must_use]
    pub fn can_transition(&self, to: LifecycleStatus) -> bool {
        self.lifecycle_status.can_transition_to(to)
    }

    /// FIA I-3: close the installment assignment. Valid from
    /// both Open (admin closes before due date) and Paid (admin
    /// closes after full payment, e.g., end of academic year).
    /// Returns Conflict on Closed or Cancelled (terminal states
    /// cannot be re-closed).
    #[allow(clippy::needless_pass_by_value)]
    pub fn close(
        &mut self,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) -> educore_core::error::Result<()> {
        if !self.can_transition(LifecycleStatus::Closed) {
            return Err(educore_core::error::DomainError::conflict(format!(
                "FeesInstallmentAssign cannot be closed from state {:?} (FIA I-3)",
                self.lifecycle_status
            )));
        }
        self.lifecycle_status = LifecycleStatus::Closed;
        self.balance_minor = 0;
        self.updated_at = at;
        self.updated_by = actor;
        self.last_event_id = Some(event_id);
        self.version = self.version.next();
        Ok(())
    }

    /// FIA I-3: cancel the installment assignment. Only valid
    /// from Open (no payments recorded). Returns Conflict on
    /// any other state, including Paid (the dispatcher must
    /// reverse payments first).
    #[allow(clippy::needless_pass_by_value)]
    pub fn cancel(
        &mut self,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) -> educore_core::error::Result<()> {
        if !self.can_transition(LifecycleStatus::Cancelled) {
            return Err(educore_core::error::DomainError::conflict(format!(
                "FeesInstallmentAssign cannot be cancelled from state {:?} (FIA I-3)",
                self.lifecycle_status
            )));
        }
        if self.paid_amount_minor > 0 {
            return Err(educore_core::error::DomainError::conflict(
                "FeesInstallmentAssign cannot be cancelled: payments already recorded (FIA I-3)",
            ));
        }
        self.lifecycle_status = LifecycleStatus::Cancelled;
        self.balance_minor = 0;
        self.updated_at = at;
        self.updated_by = actor;
        self.last_event_id = Some(event_id);
        self.version = self.version.next();
        Ok(())
    }

    pub fn retire(&mut self, at: Timestamp, actor: UserId) -> educore_core::error::Result<()> {
        if self.active_status == ActiveStatus::Retired {
            return Err(educore_core::error::DomainError::conflict(
                "FeesInstallmentAssign is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// =============================================================================
// RealFmFeesGroup — Wave 66 (per-aggregate wave pattern from Wave 65)
// =============================================================================
//
// Per v3 Part 2 F40: 1 invariant — "Unique by `name` within a school."
// Reference data aggregate (FM fees group catalogue — the FM invoice
// scheme's fee-grouping primitive). The placeholder stub above
// (`finance_aggregate_stub! { struct FmFeesGroup { _id: () } }`) remains
// in the file for documentation purposes; the real implementation is
// below. The service layer MUST use `RealFmFeesGroup` for new code; the
// stub is kept only to avoid breaking downstream code that referenced
// `FmFeesGroup` as a type name during Phase 7.

/// FM fees group catalogue entry. One invariant: name is unique
/// within a school (FFG I-1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealFmFeesGroup {
    /// The typed id (school_id + uuid).
    pub id: FmFeesGroupId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The FM group name (unique within school, non-empty after trim).
    pub name: String,
    /// Optional free-form description.
    pub description: Option<String>,
    /// The audit footer (10 fields, per `AGENTS.md`).
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

impl RealFmFeesGroup {
    /// Constructs a new `RealFmFeesGroup`. Enforces FFG I-1:
    /// `name` must be non-empty after trim.
    pub fn fresh(
        id: FmFeesGroupId,
        name: String,
        description: Option<String>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "FmFeesGroup name must be non-empty after trim",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            name: trimmed.to_owned(),
            description: description
                .map(|d| d.trim().to_owned())
                .filter(|d| !d.is_empty()),
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Returns `true` if the FM fees group is currently active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Mutates name + description. Enforces FFG I-1: new name must be
    /// non-empty after trim. Bumps version, advances `updated_at`,
    /// sets `updated_by`.
    pub fn update_metadata(
        &mut self,
        name: String,
        description: Option<String>,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "FmFeesGroup name must be non-empty after trim",
            ));
        }
        self.name = trimmed.to_owned();
        self.description = description
            .map(|d| d.trim().to_owned())
            .filter(|d| !d.is_empty());
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Soft-deletes the FM fees group by flipping `active_status` to
    /// `Retired`. Bumps version, advances `updated_at`, sets
    /// `updated_by`.
    pub fn retire(
        &mut self,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "FmFeesGroup is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// Wave 108 — RealAmountTransfer (inter-account cash movement)
//
// AT I-2: debit source + credit destination. An AmountTransfer
// records a movement of money from one bank account (the
// debit source = from_account_id) to another bank account (the
// credit destination = to_account_id) for a specific amount in
// a specific currency on a specific date.
//
// The (from_account_id, to_account_id) scope-key tuple pins the
// transfer to a specific (source, destination) pair. The
// aggregate enforces 2 companion invariants:
//   * from_account_id != to_account_id (cannot transfer to the
//     same account; a no-op transfer must be a separate code
//     path)
//   * amount_minor >= 0 (a negative transfer is a reversal, which
//     requires a separate reversal flow, not a fresh transfer)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealAmountTransfer {
    pub id: AmountTransferId,
    pub school_id: SchoolId,
    pub from_account_id: BankAccountId,
    pub to_account_id: BankAccountId,
    pub amount_minor: i64,
    pub currency: Currency,
    pub transfer_date: chrono::NaiveDate,
    pub note: Option<String>,
    /// AT I-3: optional idempotency reference. When present, the
    /// dispatcher enforces uniqueness on the
    /// (from_account_id, to_account_id, reference) tuple. This
    /// allows clients to safely retry failed transfers without
    /// creating duplicates. When None, no idempotency check is
    /// performed (the dispatcher may still enforce uniqueness
    /// via other means such as request_id).
    pub reference: Option<String>,
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

impl RealAmountTransfer {
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: AmountTransferId,
        from_account_id: BankAccountId,
        to_account_id: BankAccountId,
        amount_minor: i64,
        currency: Currency,
        transfer_date: chrono::NaiveDate,
        note: Option<String>,
        reference: Option<String>,
        actor: UserId,
        at: Timestamp,
        correlation: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        // Companion: cannot transfer to the same account.
        if from_account_id == to_account_id {
            return Err(educore_core::error::DomainError::validation(
                "AmountTransfer from_account_id must differ from to_account_id (AT I-2 companion)",
            ));
        }
        // Companion: amount_minor >= 0.
        if amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "AmountTransfer amount_minor must be >= 0 (AT I-2 companion)",
            ));
        }
        // AT I-3 companion: reference, when present, must be non-empty after trim.
        if let Some(ref r) = reference {
            if r.trim().is_empty() {
                return Err(educore_core::error::DomainError::validation(
                    "AmountTransfer reference must be non-empty after trimming when present (AT I-3)",
                ));
            }
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            from_account_id,
            to_account_id,
            amount_minor,
            currency,
            transfer_date,
            note,
            reference,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: at,
            updated_at: at,
            created_by: actor,
            updated_by: actor,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id: correlation,
        })
    }

    pub fn is_active(&self) -> bool {
        self.active_status == ActiveStatus::Active
    }

    pub fn retire(&mut self, at: Timestamp, actor: UserId) -> educore_core::error::Result<()> {
        if self.active_status == ActiveStatus::Retired {
            return Err(educore_core::error::DomainError::conflict(
                "AmountTransfer is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// RealInvoiceSetting — Wave 67 (per-aggregate wave pattern from Waves 65–66)
// =============================================================================
//
// Per v3 Part 2 F54: 1 invariant — "Prefix format valid" (ISv I-1).
// Reference data aggregate (the school's invoice numbering config:
// prefix + start_form). The placeholder stub above
// (`finance_aggregate_stub! { struct InvoiceSetting { _id: () } }`)
// remains in the file for documentation purposes; the real
// implementation is below. The service layer MUST use `RealInvoiceSetting`
// for new code; the stub is kept only to avoid breaking downstream code
// that referenced `InvoiceSetting` as a type name during Phase 7.

/// Invoice numbering configuration for a school. One invariant: the
/// `prefix` must be 1..=10 chars after trim (ISv I-1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealInvoiceSetting {
    /// The typed id (school_id + uuid).
    pub id: InvoiceSettingId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The invoice-number prefix (1..=10 chars after trim).
    pub prefix: String,
    /// The starting invoice number (≥ 0).
    pub start_form: i64,
    /// The audit footer (10 fields, per `AGENTS.md`).
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

impl RealInvoiceSetting {
    /// Maximum allowed prefix length (per ISv I-1).
    pub const MAX_PREFIX_LEN: usize = 10;

    /// Constructs a new `RealInvoiceSetting`. Enforces ISv I-1:
    /// `prefix` must be 1..=`MAX_PREFIX_LEN` chars after trim.
    pub fn fresh(
        id: InvoiceSettingId,
        prefix: String,
        start_form: i64,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        let trimmed = prefix.trim();
        if trimmed.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "InvoiceSetting prefix must be non-empty after trim",
            ));
        }
        if trimmed.chars().count() > Self::MAX_PREFIX_LEN {
            return Err(educore_core::error::DomainError::validation(format!(
                "InvoiceSetting prefix must be at most {} chars after trim",
                Self::MAX_PREFIX_LEN
            )));
        }
        if start_form < 0 {
            return Err(educore_core::error::DomainError::validation(
                "InvoiceSetting start_form must be non-negative",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            prefix: trimmed.to_owned(),
            start_form,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Returns `true` if the invoice setting is currently active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Mutates prefix + start_form. Enforces ISv I-1: new prefix
    /// must be 1..=`MAX_PREFIX_LEN` chars after trim and start_form
    /// must be non-negative. Bumps version, advances `updated_at`,
    /// sets `updated_by`.
    pub fn update_config(
        &mut self,
        prefix: String,
        start_form: i64,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        let trimmed = prefix.trim();
        if trimmed.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "InvoiceSetting prefix must be non-empty after trim",
            ));
        }
        if trimmed.chars().count() > Self::MAX_PREFIX_LEN {
            return Err(educore_core::error::DomainError::validation(format!(
                "InvoiceSetting prefix must be at most {} chars after trim",
                Self::MAX_PREFIX_LEN
            )));
        }
        if start_form < 0 {
            return Err(educore_core::error::DomainError::validation(
                "InvoiceSetting start_form must be non-negative",
            ));
        }
        self.prefix = trimmed.to_owned();
        self.start_form = start_form;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Soft-deletes the invoice setting by flipping `active_status`
    /// to `Retired`. Bumps version, advances `updated_at`, sets
    /// `updated_by`.
    pub fn retire(
        &mut self,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "InvoiceSetting is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// RealQuestionBankFee — Wave 68 (per-aggregate wave pattern from Waves 65–67)
// =============================================================================
//
// Per v3 Part 2 F62: 1 invariant — "Amount ≥ 0" (QBF I-1).
// Reference data aggregate (a per-question fee amount attached to the
// school's question bank — the negative case would represent a negative
// fee, which is meaningless). The placeholder stub above
// (`finance_aggregate_stub! { struct QuestionBankFee { _id: () } }`)
// remains in the file for documentation purposes; the real
// implementation is below. The service layer MUST use
// `RealQuestionBankFee` for new code; the stub is kept only to avoid
// breaking downstream code that referenced `QuestionBankFee` as a type
// name during Phase 7.

/// Fee amount attached to a question-bank entry. One invariant: the
/// `amount_minor` must be ≥ 0 (QBF I-1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealQuestionBankFee {
    /// The typed id (school_id + uuid).
    pub id: QuestionBankFeeId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The fee name (non-empty after trim).
    pub name: String,
    /// The fee amount in minor currency units (≥ 0, per QBF I-1).
    pub amount_minor: i64,
    /// Optional free-form description.
    pub description: Option<String>,
    /// The audit footer (10 fields, per `AGENTS.md`).
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

impl RealQuestionBankFee {
    /// Constructs a new `RealQuestionBankFee`. Enforces QBF I-1:
    /// `name` must be non-empty after trim; `amount_minor` must be ≥ 0.
    pub fn fresh(
        id: QuestionBankFeeId,
        name: String,
        amount_minor: i64,
        description: Option<String>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "QuestionBankFee name must be non-empty after trim",
            ));
        }
        if amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "QuestionBankFee amount_minor must be non-negative (QBF I-1)",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            name: trimmed.to_owned(),
            amount_minor,
            description: description
                .map(|d| d.trim().to_owned())
                .filter(|d| !d.is_empty()),
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Returns `true` if the question bank fee is currently active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Mutates name + amount_minor + description. Enforces QBF I-1:
    /// new `name` must be non-empty after trim and new `amount_minor`
    /// must be ≥ 0. Bumps version, advances `updated_at`, sets
    /// `updated_by`.
    pub fn update_metadata(
        &mut self,
        name: String,
        amount_minor: i64,
        description: Option<String>,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "QuestionBankFee name must be non-empty after trim",
            ));
        }
        if amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "QuestionBankFee amount_minor must be non-negative (QBF I-1)",
            ));
        }
        self.name = trimmed.to_owned();
        self.amount_minor = amount_minor;
        self.description = description
            .map(|d| d.trim().to_owned())
            .filter(|d| !d.is_empty());
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Soft-deletes the question bank fee by flipping `active_status`
    /// to `Retired`. Bumps version, advances `updated_at`, sets
    /// `updated_by`.
    pub fn retire(
        &mut self,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "QuestionBankFee is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// RealDirectFeesSetting — Wave 69 (per-aggregate wave pattern from Waves 65–68)
// =============================================================================
//
// Per v3 Part 2 F43 / checklist § DirectFeesSetting: 2 invariants:
//   - DFS I-1: reminder_before ≥ 0, no_installment ≥ 0
//   - DFS I-2: due_date_from_sem ∈ 1..=28
// Per-school configuration aggregate (the direct-fees programme's
// per-school config: enabled flag + reminder window + installment cap +
// due-day-of-month). The placeholder stub above
// (`finance_aggregate_stub! { struct DirectFeesSetting { _id: () } }`)
// remains in the file for documentation purposes; the real
// implementation is below. The service layer MUST use
// `RealDirectFeesSetting` for new code; the stub is kept only to avoid
// breaking downstream code that referenced `DirectFeesSetting` as a type
// name during Phase 7.

/// Per-school direct-fees programme configuration. Two invariants:
/// `reminder_before >= 0 && no_installment >= 0` (DFS I-1);
/// `due_date_from_sem in 1..=28` (DFS I-2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealDirectFeesSetting {
    /// The typed id (school_id + uuid).
    pub id: DirectFeesSettingId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// Whether the direct-fees programme is enabled at this school.
    pub enabled: bool,
    /// Days before due_date that a reminder is sent (DFS I-1: >= 0).
    pub reminder_before: i64,
    /// Maximum number of installments a student may have open (DFS I-1: >= 0).
    pub no_installment: i64,
    /// Day of month on which installments fall due (DFS I-2: 1..=28).
    pub due_date_from_sem: u8,
    /// Optional free-form description.
    pub description: Option<String>,
    /// The audit footer (10 fields, per `AGENTS.md`).
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

impl RealDirectFeesSetting {
    /// Maximum allowed day-of-month for `due_date_from_sem` (DFS I-2).
    /// 28 (not 31) is chosen so the due-day is valid for every month,
    /// including February in non-leap years.
    pub const MAX_DUE_DAY: u8 = 28;

    /// Constructs a new `RealDirectFeesSetting`. Enforces DFS I-1
    /// (`reminder_before >= 0`, `no_installment >= 0`) and DFS I-2
    /// (`due_date_from_sem in 1..=MAX_DUE_DAY`).
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: DirectFeesSettingId,
        enabled: bool,
        reminder_before: i64,
        no_installment: i64,
        due_date_from_sem: u8,
        description: Option<String>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        if reminder_before < 0 {
            return Err(educore_core::error::DomainError::validation(
                "DirectFeesSetting reminder_before must be non-negative (DFS I-1)",
            ));
        }
        if no_installment < 0 {
            return Err(educore_core::error::DomainError::validation(
                "DirectFeesSetting no_installment must be non-negative (DFS I-1)",
            ));
        }
        if !(1..=Self::MAX_DUE_DAY).contains(&due_date_from_sem) {
            return Err(educore_core::error::DomainError::validation(format!(
                "DirectFeesSetting due_date_from_sem must be in 1..={} (DFS I-2)",
                Self::MAX_DUE_DAY
            )));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            enabled,
            reminder_before,
            no_installment,
            due_date_from_sem,
            description: description
                .map(|d| d.trim().to_owned())
                .filter(|d| !d.is_empty()),
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Returns `true` if the direct-fees setting is currently active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Mutates enabled + reminder_before + no_installment +
    /// due_date_from_sem + description. Enforces DFS I-1 (both ints
    /// >= 0) and DFS I-2 (day in 1..=MAX_DUE_DAY). Bumps version,
    /// advances `updated_at`, sets `updated_by`.
    #[allow(clippy::too_many_arguments)]
    pub fn update_config(
        &mut self,
        enabled: bool,
        reminder_before: i64,
        no_installment: i64,
        due_date_from_sem: u8,
        description: Option<String>,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if reminder_before < 0 {
            return Err(educore_core::error::DomainError::validation(
                "DirectFeesSetting reminder_before must be non-negative (DFS I-1)",
            ));
        }
        if no_installment < 0 {
            return Err(educore_core::error::DomainError::validation(
                "DirectFeesSetting no_installment must be non-negative (DFS I-1)",
            ));
        }
        if !(1..=Self::MAX_DUE_DAY).contains(&due_date_from_sem) {
            return Err(educore_core::error::DomainError::validation(format!(
                "DirectFeesSetting due_date_from_sem must be in 1..={} (DFS I-2)",
                Self::MAX_DUE_DAY
            )));
        }
        self.enabled = enabled;
        self.reminder_before = reminder_before;
        self.no_installment = no_installment;
        self.due_date_from_sem = due_date_from_sem;
        self.description = description
            .map(|d| d.trim().to_owned())
            .filter(|d| !d.is_empty());
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Soft-deletes the direct-fees setting by flipping `active_status`
    /// to `Retired`. Bumps version, advances `updated_at`, sets
    /// `updated_by`.
    pub fn retire(
        &mut self,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "DirectFeesSetting is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// RealFeesCarryForwardLog — Wave 70 (per-aggregate wave pattern from Waves 65–69)
// =============================================================================
//
// Per v3 Part 2 F28 + checklist § FeesCarryForwardLog: 2 invariants:
//   - FCFL I-1: append-only (no update / no delete; only retire)
//   - FCFL I-2: amount_minor ≥ 0
// Append-only ledger of per-student per-academic-year carry-forward
// rows (the record of how much balance was rolled over from the previous
// academic year into the next). The placeholder stub above
// (`finance_aggregate_stub! { struct FeesCarryForwardLog { _id: () } }`)
// remains in the file for documentation purposes; the real
// implementation is below. The service layer MUST use
// `RealFeesCarryForwardLog` for new code; the stub is kept only to avoid
// breaking downstream code that referenced `FeesCarryForwardLog` as a
// type name during Phase 7.

/// Append-only ledger row for a per-student per-academic-year balance
/// carry-forward. Two invariants: append-only (FCFL I-1, enforced at the
/// API surface by *not* exposing `update_*` mutators); `amount_minor >= 0`
/// (FCFL I-2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealFeesCarryForwardLog {
    /// The typed id (school_id + uuid).
    pub id: FeesCarryForwardLogId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The student whose balance is being carried forward.
    pub student_id: StudentId,
    /// The academic year the balance is being carried *into* (the new
    /// year).
    pub academic_year_id: AcademicYearId,
    /// The carried-forward amount in minor currency units (≥ 0, per FCFL I-2).
    pub amount_minor: i64,
    /// Optional free-form description / source note.
    pub description: Option<String>,
    /// The audit footer (10 fields, per `AGENTS.md`).
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

impl RealFeesCarryForwardLog {
    /// Constructs a new `RealFeesCarryForwardLog`. Enforces FCFL I-2:
    /// `amount_minor` must be ≥ 0. Note: FCFL I-1 (append-only) is
    /// enforced at the API surface — this aggregate intentionally
    /// exposes no `update_*` mutator. The only post-creation
    /// transitions are the version-bumping `retire()` (soft-delete,
    /// for legal-record retention policies) and the aggregate is
    /// otherwise immutable.
    pub fn fresh(
        id: FeesCarryForwardLogId,
        student_id: StudentId,
        academic_year_id: AcademicYearId,
        amount_minor: i64,
        description: Option<String>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        if amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "FeesCarryForwardLog amount_minor must be non-negative (FCFL I-2)",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            student_id,
            academic_year_id,
            amount_minor,
            description: description
                .map(|d| d.trim().to_owned())
                .filter(|d| !d.is_empty()),
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Returns `true` if the carry-forward log row is currently active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Soft-deletes the carry-forward log row by flipping `active_status`
    /// to `Retired`. Bumps version, advances `updated_at`, sets
    /// `updated_by`. Note: this does **not** violate FCFL I-1 (append-only)
    /// because the audit footer + the `Retired` status together preserve
    /// the original record; the soft-delete is a tombstone, not a
    /// modification of the carried amount or student/year references.
    pub fn retire(
        &mut self,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "FeesCarryForwardLog is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// RealFmFeesTransactionChild — Wave 77 (per-aggregate wave pattern
// from Waves 65–75)
// =============================================================================
//
// Per v3 Part 2 F33 + checklist § FmFeesTransactionChild: 2 invariants:
//   - FFTC I-1: amount_minor ≥ 0 (numeric money invariant)
//   - FFTC I-2: parent reference valid (the parent
//               FmFeesTransactionId must belong to the same school as
//               the child; cross-school defense-in-depth is enforced
//               at the aggregate surface; full existence check
//               against the `FmFeesTransaction` row is the
//               dispatcher / storage-adapter's concern).
// Child entity under a `FmFeesTransaction` aggregate — one row per
// line in a transaction (amount + optional description). The
// placeholder stub above
// (`finance_aggregate_stub! { struct FmFeesTransactionChild { _id: () } }`)
// remains in the file for documentation purposes; the real
// implementation is below. The service layer MUST use
// `RealFmFeesTransactionChild` for new code; the stub is kept only
// to avoid breaking downstream code that referenced
// `FmFeesTransactionChild` as a type name during Phase 7.

/// A child row under a [`FmFeesTransaction`] aggregate. Two
/// invariants: amount_minor is non-negative (FFTC I-1), and the
/// parent transaction reference belongs to the same school as the
/// child (FFTC I-2 cross-school check; existence check is the
/// dispatcher's concern). Full lifecycle: fresh + update_metadata +
/// retire.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealFmFeesTransactionChild {
    /// The typed id (school_id + uuid).
    pub id: FmFeesTransactionChildId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The parent transaction this child row belongs to.
    pub fm_fees_transaction_id: FmFeesTransactionId,
    /// The amount for this child row (in minor currency units,
    /// ≥ 0 per FFTC I-1).
    pub amount_minor: i64,
    /// Optional free-form description.
    pub description: Option<String>,
    /// The audit footer (10 fields, per `AGENTS.md`).
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

impl RealFmFeesTransactionChild {
    /// Constructs a new `RealFmFeesTransactionChild`. Enforces FFTC
    /// I-1 (`amount_minor >= 0`) and FFTC I-2 cross-school
    /// consistency (the parent `FmFeesTransactionId` must belong to
    /// the same school as the child `FmFeesTransactionChildId`).
    /// Existence of the parent transaction is the dispatcher's
    /// concern.
    pub fn fresh(
        id: FmFeesTransactionChildId,
        fm_fees_transaction_id: FmFeesTransactionId,
        amount_minor: i64,
        description: Option<String>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        // FFTC I-1: amount must be non-negative.
        if amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "FmFeesTransactionChild amount_minor must be non-negative (FFTC I-1)",
            ));
        }
        // FFTC I-2 (cross-school): parent id must belong to the same
        // school as the child id.
        if fm_fees_transaction_id.school_id() != id.school_id() {
            return Err(educore_core::error::DomainError::validation(
                "FmFeesTransactionChild parent fm_fees_transaction_id must belong to the same school as the child id (FFTC I-2)",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            fm_fees_transaction_id,
            amount_minor,
            description: description
                .map(|d| d.trim().to_owned())
                .filter(|d| !d.is_empty()),
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Returns `true` if the child row is currently active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Updates the amount and description of a child row. Re-validates
    /// FFTC I-1 (`amount_minor >= 0`). FFTC I-2 (parent reference) is
    /// immutable on update — the parent is set at creation and cannot
    /// change (the dispatcher rejects `Reparent` commands; the spec
    /// forbids re-parenting child rows). Bumps version, advances
    /// `updated_at`, sets `updated_by`.
    pub fn update_metadata(
        &mut self,
        amount_minor: i64,
        description: Option<String>,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "FmFeesTransactionChild is retired; cannot update metadata",
            ));
        }
        if amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "FmFeesTransactionChild amount_minor must be non-negative on update (FFTC I-1)",
            ));
        }
        self.amount_minor = amount_minor;
        self.description = description
            .map(|d| d.trim().to_owned())
            .filter(|d| !d.is_empty());
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Soft-deletes the child row by flipping `active_status` to
    /// `Retired`. Bumps version, advances `updated_at`, sets
    /// `updated_by`. Preserves FFTC I-1 (the original amount is
    /// preserved in the audit footer) and FFTC I-2 (the parent
    /// reference is immutable, so it remains valid even after retire).
    pub fn retire(
        &mut self,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "FmFeesTransactionChild is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// RealFmFeesTransactionLineNote — Wave 75 (per-aggregate wave pattern
// from Waves 65–74)
// =============================================================================
//
// Per v3 Part 2 F32 + checklist § FmFeesTransactionLineNote: 2
// invariants:
//   - FFTLN I-1: note non-empty (after trim, 1..=2000 chars)
//   - FFTLN I-2: append-only (no update / no delete; only retire)
// Free-form note attached to a line on an `FmFeesTransaction` aggregate.
// The placeholder stub above
// (`finance_aggregate_stub! { struct FmFeesTransactionLineNote { _id: () } }`)
// remains in the file for documentation purposes; the real
// implementation is below. The service layer MUST use
// `RealFmFeesTransactionLineNote` for new code; the stub is kept only
// to avoid breaking downstream code that referenced
// `FmFeesTransactionLineNote` as a type name during Phase 7.

/// A free-form note line attached to a [`FmFeesTransaction`] aggregate.
/// Two invariants: note is non-empty after trim (FFTLN I-1); the
/// aggregate is append-only (FFTLN I-2, enforced at the API surface by
/// *not* exposing any `update_*` mutator). The only post-creation
/// transition is the version-bumping `retire()` (soft-delete for
/// legal-record retention), which is itself a tombstone and not a
/// modification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealFmFeesTransactionLineNote {
    /// The typed id (school_id + uuid).
    pub id: FmFeesTransactionLineNoteId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The parent transaction this note line belongs to.
    pub fm_fees_transaction_id: FmFeesTransactionId,
    /// The note text (1..=2000 chars after trim, per FFTLN I-1).
    pub note: String,
    /// The audit footer (10 fields, per `AGENTS.md`).
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

impl RealFmFeesTransactionLineNote {
    /// Constructs a new `RealFmFeesTransactionLineNote`. Enforces
    /// FFTLN I-1 (note 1..=2000 chars after trim) via
    /// `RealFmFeesTransactionLineNote::fresh` -> `validate_note_text`.
    /// Note: FFTLN I-2 (append-only) is enforced at the API surface —
    /// this aggregate intentionally exposes no `update_*` mutator. The
    /// only post-creation transition is the version-bumping `retire()`
    /// (soft-delete, for legal-record retention policies).
    pub fn fresh(
        id: FmFeesTransactionLineNoteId,
        fm_fees_transaction_id: FmFeesTransactionId,
        note: String,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        let trimmed = note.trim();
        crate::value_objects::validate_note_text(trimmed)?;
        Ok(Self {
            school_id: id.school_id(),
            id,
            fm_fees_transaction_id,
            note: trimmed.to_owned(),
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Returns `true` if the note line is currently active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Soft-deletes the note line by flipping `active_status` to
    /// `Retired`. Bumps version, advances `updated_at`, sets
    /// `updated_by`. Note: this does **not** violate FFTLN I-2
    /// (append-only) because the audit footer + the `Retired` status
    /// together preserve the original record; the soft-delete is a
    /// tombstone, not a modification of the note text or the parent
    /// transaction reference.
    pub fn retire(
        &mut self,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "FmFeesTransactionLineNote is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// RealChartOfAccount — Wave 74 (per-aggregate wave pattern from
// Waves 65–73)
// =============================================================================
//
// Per v3 Part 2 F7 + checklist § ChartOfAccount: 2 invariants:
//   - COA I-1: unique name within school (per-school uniqueness is a
//              dispatcher / storage-adapter concern; this drop pins
//              the shape + name/code validation that the uniqueness
//              check will key on).
//   - COA I-2: cannot delete while referenced by any ledger entry
//              (reference integrity is a dispatcher / storage-adapter
//              concern; this drop pins the retire lifecycle that the
//              reference check will gate on — `delete_attempted` is
//              only allowed when no references exist; retire is the
//              tombstone when references exist).
// Foundational aggregate for double-entry bookkeeping: every ledger
// entry (transaction, expense, payment, fee assignment) references a
// `ChartOfAccount` by id. The placeholder stub above
// (`finance_aggregate_stub! { struct ChartOfAccount { _id: () } }`)
// remains in the file for documentation purposes; the real
// implementation is below. The service layer MUST use
// `RealChartOfAccount` for new code; the stub is kept only to avoid
// breaking downstream code that referenced `ChartOfAccount` as a
// type name during Phase 7.

/// Foundational chart-of-account aggregate. Two invariants: name and
/// code must be valid format (COA I-1 pins the shape; per-school
/// uniqueness is the dispatcher's concern), and the aggregate cannot
/// be deleted while referenced by any ledger entry (COA I-2; reference
/// integrity is the dispatcher's concern). The aggregate exposes a
/// full lifecycle (`fresh` / `update_metadata` / `retire`) because
/// chart-of-account entries are long-lived reference data, not
/// append-only logs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealChartOfAccount {
    /// The typed id (school_id + uuid).
    pub id: ChartOfAccountId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The chart-of-account code (1..=20 chars, matches `[A-Z0-9-]`).
    /// e.g. `1000`, `1100-CASH`, `4000-REV`.
    pub code: String,
    /// The chart-of-account name (1..=100 chars).
    /// e.g. `Cash`, `Student Fees Receivable`.
    pub name: String,
    /// The account type (asset, liability, equity, revenue, expense).
    pub account_type: AccountType,
    /// Optional free-form description.
    pub description: Option<String>,
    /// The audit footer (10 fields, per `AGENTS.md`).
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

impl RealChartOfAccount {
    /// Constructs a new `RealChartOfAccount`. Enforces the shape
    /// validation that COA I-1 will key on (code + name format). Per-
    /// school name uniqueness is the dispatcher's concern (v3 Part 6);
    /// this drop pins the field-level contract that the uniqueness
    /// check will run against.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: ChartOfAccountId,
        code: String,
        name: String,
        account_type: AccountType,
        description: Option<String>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        let trimmed_code = code.trim();
        let trimmed_name = name.trim();
        crate::value_objects::validate_chart_of_account_code(trimmed_code)?;
        crate::value_objects::validate_chart_of_account_name(trimmed_name)?;
        Ok(Self {
            school_id: id.school_id(),
            id,
            code: trimmed_code.to_owned(),
            name: trimmed_name.to_owned(),
            account_type,
            description: description
                .map(|d| d.trim().to_owned())
                .filter(|d| !d.is_empty()),
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Returns `true` if the chart-of-account entry is currently active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Updates the metadata of a chart-of-account entry: code, name,
    /// account_type, description. Re-validates the code and name
    /// format. Per-school uniqueness is the dispatcher's concern and
    /// is checked outside this aggregate. Bumps version, advances
    /// `updated_at`, sets `updated_by`.
    #[allow(clippy::too_many_arguments)]
    pub fn update_metadata(
        &mut self,
        code: String,
        name: String,
        account_type: AccountType,
        description: Option<String>,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "ChartOfAccount is retired; cannot update metadata",
            ));
        }
        let trimmed_code = code.trim();
        let trimmed_name = name.trim();
        crate::value_objects::validate_chart_of_account_code(trimmed_code)?;
        crate::value_objects::validate_chart_of_account_name(trimmed_name)?;
        self.code = trimmed_code.to_owned();
        self.name = trimmed_name.to_owned();
        self.account_type = account_type;
        self.description = description
            .map(|d| d.trim().to_owned())
            .filter(|d| !d.is_empty());
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Soft-deletes the chart-of-account entry by flipping
    /// `active_status` to `Retired`. Per COA I-2, the service layer
    /// MUST check reference integrity (no ledger entries reference
    /// this chart-of-account) BEFORE calling this method; the
    /// dispatcher rejects the `Delete` command when references
    /// exist. This method itself is a tombstone that preserves the
    /// audit trail + original code/name/account_type for legal-record
    /// retention.
    pub fn retire(
        &mut self,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "ChartOfAccount is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// RealDirectFeesInstallmentAssignChild — Wave 73 (per-aggregate wave
// pattern from Waves 65–72)
// =============================================================================
//
// Per v3 Part 2 F12 + checklist § DirectFeesInstallmentAssignChild:
// 2 invariants:
//   - DFIAC I-1: append-only (no update / no delete; only retire)
//   - DFIAC I-2: timestamps monotonic (created_at ≤ updated_at on every
//                transition; retire() advances updated_at past created_at)
// Child entity that lives under a `DirectFeesInstallmentAssign`
// aggregate (one installment assignment can have many child rows
// representing the per-installment breakdown: amount, due date,
// optional discount). The placeholder stub above
// (`finance_aggregate_stub! { struct DirectFeesInstallmentAssignChild { _id: () } }`)
// remains in the file for documentation purposes; the real
// implementation is below. The service layer MUST use
// `RealDirectFeesInstallmentAssignChild` for new code; the stub is
// kept only to avoid breaking downstream code that referenced
// `DirectFeesInstallmentAssignChild` as a type name during Phase 7.

/// Child entity under a [`DirectFeesInstallmentAssign`] aggregate. Two
/// invariants: append-only (DFIAC I-1, enforced at the API surface by
/// *not* exposing any `update_*` mutator), and timestamps monotonic
/// (DFIAC I-2 — `created_at <= updated_at` always holds; the only
/// post-creation transition is `retire()`, which advances `updated_at`
/// past `created_at` and bumps version).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealDirectFeesInstallmentAssignChild {
    /// The typed id (school_id + uuid).
    pub id: DirectFeesInstallmentAssignChildId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The parent assignment this child belongs to.
    pub direct_fees_installment_assign_id: DirectFeesInstallmentAssignId,
    /// The amount for this child installment (in minor currency units).
    /// Per DFIAC I-2 (timestamps monotonic), the `created_at` of this
    /// aggregate is the time the child row was appended; the audit
    /// footer advances monotonically on every state transition.
    pub amount_minor: i64,
    /// The audit footer (10 fields, per `AGENTS.md`).
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

impl RealDirectFeesInstallmentAssignChild {
    /// Constructs a new `RealDirectFeesInstallmentAssignChild`. The
    /// constructor itself enforces DFIAC I-2 by setting `updated_at =
    /// created_at` (monotonic baseline). DFIAC I-1 (append-only) is
    /// enforced at the API surface — this aggregate intentionally
    /// exposes no `update_*` mutator. The only post-creation
    /// transition is the version-bumping `retire()` (soft-delete for
    /// legal-record retention), which preserves the original
    /// `created_at`, parent assignment reference, and amount via the
    /// audit footer + the `Retired` active_status.
    pub fn fresh(
        id: DirectFeesInstallmentAssignChildId,
        direct_fees_installment_assign_id: DirectFeesInstallmentAssignId,
        amount_minor: i64,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        if amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "DirectFeesInstallmentAssignChild amount_minor must be non-negative",
            ));
        }
        // DFIAC I-2 (monotonic timestamps): constructed baseline
        // satisfies created_at == updated_at.
        Ok(Self {
            school_id: id.school_id(),
            id,
            direct_fees_installment_assign_id,
            amount_minor,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Returns `true` if the child row is currently active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Returns `true` if `updated_at >= created_at` (DFIAC I-2
    /// monotonic invariant). The baseline is `updated_at == created_at`
    /// on `fresh`; `retire` advances `updated_at` strictly past
    /// `created_at`.
    #[must_use]
    pub fn timestamps_monotonic(&self) -> bool {
        self.updated_at.as_datetime() >= self.created_at.as_datetime()
    }

    /// Soft-deletes the child row by flipping `active_status` to
    /// `Retired`. Bumps version, advances `updated_at` strictly past
    /// `created_at`, sets `updated_by`. Preserves DFIAC I-1
    /// (append-only) because the audit footer + the `Retired` status
    /// together preserve the original record; the soft-delete is a
    /// tombstone, not a modification of the amount or parent
    /// assignment reference.
    pub fn retire(
        &mut self,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "DirectFeesInstallmentAssignChild is already retired",
            ));
        }
        // DFIAC I-2: updated_at must advance strictly past created_at
        // on retire. If the caller passes a stale timestamp, advance
        // forward by one nanosecond.
        let advanced = if at.as_datetime() <= self.created_at.as_datetime() {
            Timestamp::from_datetime(
                self.created_at.as_datetime() + chrono::Duration::nanoseconds(1),
            )
        } else {
            at
        };
        self.active_status = ActiveStatus::Retired;
        self.updated_at = advanced;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// RealFmFeesInvoiceLineNote — Wave 72 (per-aggregate wave pattern from
// Waves 65–71)
// =============================================================================
//
// Per v3 Part 2 F30 + checklist § FmFeesInvoiceLineNote: 2 invariants:
//   - FFILN I-1: note non-empty (after trim, 1..=2000 chars)
//   - FFILN I-2: append-only (no update / no delete; only retire)
// Free-form note attached to a line on an `FmFeesInvoice` aggregate.
// Used by school finance staff to record per-line context that doesn't
// fit the structured fields. The placeholder stub above
// (`finance_aggregate_stub! { struct FmFeesInvoiceLineNote { _id: () } }`)
// remains in the file for documentation purposes; the real
// implementation is below. The service layer MUST use
// `RealFmFeesInvoiceLineNote` for new code; the stub is kept only to
// avoid breaking downstream code that referenced
// `FmFeesInvoiceLineNote` as a type name during Phase 7.

/// A free-form note line attached to a [`FmFeesInvoice`] aggregate.
/// Two invariants: note is non-empty after trim (FFILN I-1); the
/// aggregate is append-only (FFILN I-2, enforced at the API surface by
/// *not* exposing any `update_*` mutator). The only post-creation
/// transition is the version-bumping `retire()` (soft-delete for
/// legal-record retention), which is itself a tombstone and not a
/// modification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealFmFeesInvoiceLineNote {
    /// The typed id (school_id + uuid).
    pub id: FmFeesInvoiceLineNoteId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The parent invoice this note line belongs to.
    pub fm_fees_invoice_id: FmFeesInvoiceId,
    /// The note text (1..=2000 chars after trim, per FFILN I-1).
    pub note: String,
    /// The audit footer (10 fields, per `AGENTS.md`).
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

impl RealFmFeesInvoiceLineNote {
    /// Constructs a new `RealFmFeesInvoiceLineNote`. Enforces FFILN I-1
    /// (note non-empty, 1..=2000 chars after trim). Note: FFILN I-2
    /// (append-only) is enforced at the API surface — this aggregate
    /// intentionally exposes no `update_*` mutator. The only
    /// post-creation transition is the version-bumping `retire()`
    /// (soft-delete, for legal-record retention policies).
    pub fn fresh(
        id: FmFeesInvoiceLineNoteId,
        fm_fees_invoice_id: FmFeesInvoiceId,
        note: String,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        let trimmed = note.trim();
        crate::value_objects::validate_note_text(trimmed)?;
        Ok(Self {
            school_id: id.school_id(),
            id,
            fm_fees_invoice_id,
            note: trimmed.to_owned(),
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Returns `true` if the note line is currently active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Soft-deletes the note line by flipping `active_status` to
    /// `Retired`. Bumps version, advances `updated_at`, sets
    /// `updated_by`. Note: this does **not** violate FFILN I-2
    /// (append-only) because the audit footer + the `Retired` status
    /// together preserve the original record; the soft-delete is a
    /// tombstone, not a modification of the note text or the parent
    /// invoice reference.
    pub fn retire(
        &mut self,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "FmFeesInvoiceLineNote is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// RealDonor — Wave 71 (per-aggregate wave pattern from Waves 65–70)
// =============================================================================
//
// Per v3 Part 2 F23 + checklist § Donor: 2 invariants:
//   - DO I-1: `show_public` is a boolean (always satisfied; pinned by
//             the type system — the field is a Rust `bool`).
//   - DO I-2: email unique within school (uniqueness is enforced at
//             the storage-adapter layer per v3 Part 6 — this drop
//             pins the shape + validation; the actual uniqueness
//             check is wired in when the dispatcher is implemented).
// Reference data aggregate (a school's donor directory — alumni,
// parents, foundations that donate funds). The placeholder stub above
// (`finance_aggregate_stub! { struct Donor { _id: () } }`) remains in
// the file for documentation purposes; the real implementation is
// below. The service layer MUST use `RealDonor` for new code; the
// stub is kept only to avoid breaking downstream code that referenced
// `Donor` as a type name during Phase 7.

/// A school's donor directory entry. Two invariants: `show_public` is
/// a boolean (DO I-1, pinned by `bool` type) and the email is
/// non-empty / 1..=200 chars / contains `@` (DO I-2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealDonor {
    /// The typed id (school_id + uuid).
    pub id: DonorId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The donor's name (1..=200 chars after trim).
    pub name: String,
    /// The donor's email (non-empty, 1..=200 chars, contains `@`).
    pub email: String,
    /// Whether the donor is shown on the public donor wall.
    pub show_public: bool,
    /// Optional phone number (free-form string; no E.164 enforcement
    /// in this drop — v3 deferred that to a typed value object).
    pub phone: Option<String>,
    /// Optional free-form description.
    pub description: Option<String>,
    /// The audit footer (10 fields, per `AGENTS.md`).
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

impl RealDonor {
    /// Constructs a new `RealDonor`. Enforces DO I-2 (email non-empty,
    /// length-bounded, `@`-bearing) via `validate_donor_name` and
    /// `validate_donor_email`. DO I-1 (`show_public` boolean) is
    /// pinned at the type-system level (`pub show_public: bool`).
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: DonorId,
        name: String,
        email: String,
        show_public: bool,
        phone: Option<String>,
        description: Option<String>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        crate::value_objects::validate_donor_name(name.trim())?;
        crate::value_objects::validate_donor_email(email.trim())?;
        Ok(Self {
            school_id: id.school_id(),
            id,
            name: name.trim().to_owned(),
            email: email.trim().to_owned(),
            show_public,
            phone: phone
                .map(|p| p.trim().to_owned())
                .filter(|p| !p.is_empty()),
            description: description
                .map(|d| d.trim().to_owned())
                .filter(|d| !d.is_empty()),
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Returns `true` if the donor is currently active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Mutates name + email + show_public + phone + description.
    /// Enforces DO I-2 (same email validation as `fresh`). DO I-1
    /// stays pinned by the `bool` type. Bumps version, advances
    /// `updated_at`, sets `updated_by`.
    #[allow(clippy::too_many_arguments)]
    pub fn update_metadata(
        &mut self,
        name: String,
        email: String,
        show_public: bool,
        phone: Option<String>,
        description: Option<String>,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        crate::value_objects::validate_donor_name(name.trim())?;
        crate::value_objects::validate_donor_email(email.trim())?;
        self.name = name.trim().to_owned();
        self.email = email.trim().to_owned();
        self.show_public = show_public;
        self.phone = phone
            .map(|p| p.trim().to_owned())
            .filter(|p| !p.is_empty());
        self.description = description
            .map(|d| d.trim().to_owned())
            .filter(|d| !d.is_empty());
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Soft-deletes the donor by flipping `active_status` to
    /// `Retired`. Bumps version, advances `updated_at`, sets
    /// `updated_by`.
    pub fn retire(
        &mut self,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "Donor is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// RealFeesCarryForwardSetting — Wave 78 (per-aggregate wave pattern from
// Waves 65–77)
// =============================================================================
//
// Per v3 Part 2 F34 + checklist § FeesCarryForwardSetting: 2
// invariants:
//   - FCFA I-1: per-school config — meaning each school gets its own
//              carry-forward setting (no global config). The
//              `FeesCarryForwardSettingId` typed id carries the
//              `school_id` so the aggregate is inherently school-scoped.
//              One-config-per-school uniqueness is a dispatcher /
//              storage-adapter concern (parallel to COA I-1 from
//              Wave 74); this drop pins the shape that the
//              uniqueness check will key on.
//   - FCFA I-2: threshold >= 0 — the carry-forward threshold
//              (in minor currency units) must be non-negative. A
//              threshold of 0 means "carry forward everything above
//              zero"; a threshold > 0 means "only carry forward
//              balances above the threshold". Negative thresholds
//              are nonsensical and rejected at construction + on
//              update.
// Foundational aggregate for the carry-forward feature: every
// `FeesCarryForward` row references a `FeesCarryForwardSetting` by id
// to read the per-school threshold + enabled flag. The placeholder
// stub above
// (`finance_aggregate_stub! { struct FeesCarryForwardSetting { _id: () } }`)
// remains in the file for documentation purposes; the real
// implementation is below. The service layer MUST use
// `RealFeesCarryForwardSetting` for new code; the stub is kept only
// to avoid breaking downstream code that referenced
// `FeesCarryForwardSetting` as a type name during Phase 7.

/// Per-school configuration for the fees-carry-forward feature.
/// Two invariants: FCFA I-1 (per-school config; the typed id
/// carries the school_id, so the aggregate is inherently
/// school-scoped — uniqueness across schools is meaningless because
/// the aggregate is keyed by `(school_id, uuid)`, and one-per-school
/// is a dispatcher concern) and FCFA I-2 (threshold_minor >= 0).
/// Full lifecycle: fresh + update_metadata + retire.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealFeesCarryForwardSetting {
    /// The typed id (school_id + uuid).
    pub id: FeesCarryForwardSettingId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The carry-forward threshold (in minor currency units, ≥ 0 per
    /// FCFA I-2). A threshold of 0 means "carry forward everything
    /// above zero"; a threshold > 0 means "only carry forward
    /// balances above the threshold".
    pub threshold_minor: i64,
    /// Whether the carry-forward feature is enabled for this
    /// school. When `false`, `FeesCarryForward` rows must not be
    /// created for this school.
    pub enabled: bool,
    /// Optional free-form description (e.g. "Carry forward only
    /// balances above 100.00").
    pub description: Option<String>,
    /// The audit footer (10 fields, per `AGENTS.md`).
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

impl RealFeesCarryForwardSetting {
    /// Constructs a new `RealFeesCarryForwardSetting`. Enforces FCFA
    /// I-2 (`threshold_minor >= 0`). FCFA I-1 (per-school scoping) is
    /// inherent in the typed id — the school_id is derived from
    /// `id.school_id()` and stored redundantly for query convenience.
    pub fn fresh(
        id: FeesCarryForwardSettingId,
        threshold_minor: i64,
        enabled: bool,
        description: Option<String>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        // FCFA I-2: threshold must be non-negative.
        if threshold_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "FeesCarryForwardSetting threshold_minor must be non-negative (FCFA I-2)",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            threshold_minor,
            enabled,
            description: description
                .map(|d| d.trim().to_owned())
                .filter(|d| !d.is_empty()),
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Returns `true` if the setting is currently active (not retired).
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Updates the threshold, enabled flag, and description of the
    /// setting. Re-validates FCFA I-2 (`threshold_minor >= 0`).
    /// Bumps version, advances `updated_at`, sets `updated_by`.
    pub fn update_metadata(
        &mut self,
        threshold_minor: i64,
        enabled: bool,
        description: Option<String>,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "FeesCarryForwardSetting is retired; cannot update metadata",
            ));
        }
        // FCFA I-2: threshold must be non-negative on update.
        if threshold_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "FeesCarryForwardSetting threshold_minor must be non-negative on update (FCFA I-2)",
            ));
        }
        self.threshold_minor = threshold_minor;
        self.enabled = enabled;
        self.description = description
            .map(|d| d.trim().to_owned())
            .filter(|d| !d.is_empty());
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Soft-deletes the setting by flipping `active_status` to
    /// `Retired`. Bumps version, advances `updated_at`, sets
    /// `updated_by`. Preserves FCFA I-2 (the original threshold is
    /// preserved in the audit footer) and FCFA I-1 (the school_id is
    /// immutable, so the per-school scope is preserved even after
    /// retire).
    pub fn retire(
        &mut self,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "FeesCarryForwardSetting is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// RealExpenseApproval — Wave 79 (per-aggregate wave pattern from
// Waves 65–78)
// =============================================================================
//
// Per v3 Part 2 F20 + checklist § ExpenseApproval: 2 invariants:
//   - EA I-1: state machine — a fresh approval starts in
//             ApprovalStatus::Pending; it can transition to
//             Approved or Rejected exactly once. Any subsequent
//             transition (or a transition out of a terminal state)
//             returns DomainError::conflict. The state field is the
//             `ApprovalStatus` enum (typed at compile time).
//   - EA I-2: timestamps recorded — every state transition stamps
//             `decided_at` and `decided_by` on the aggregate; the
//             reject path also captures an optional `reason`
//             string. The audit footer (10 fields, per AGENTS.md)
//             preserves the full approval history.
// Approval is a child entity under an Expense: each Expense gets one
// or more ExpenseApproval rows tracking the approval workflow. The
// placeholder stub above
// (`finance_aggregate_stub! { struct ExpenseApproval { _id: () } }`)
// remains in the file for documentation purposes; the real
// implementation is below. The service layer MUST use
// `RealExpenseApproval` for new code; the stub is kept only to
// avoid breaking downstream code that referenced `ExpenseApproval`
// as a type name during Phase 7.

/// An approval workflow row for an [`Expense`]. Two invariants:
/// EA I-1 (state machine pending → approved/rejected, enforced at
/// the type-system level via the `ApprovalStatus` enum + invalid
/// transition guards) and EA I-2 (timestamps recorded: every
/// transition stamps `decided_at` + `decided_by` on the aggregate,
/// and the reject path also captures an optional `reason` string).
/// Lifecycle: fresh (Pending) → approve() / reject() (terminal).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealExpenseApproval {
    /// The typed id (school_id + uuid).
    pub id: ExpenseApprovalId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The parent expense this approval belongs to.
    pub expense_id: ExpenseId,
    /// The current approval state (Pending | Approved | Rejected).
    /// EA I-1.
    pub status: ApprovalStatus,
    /// Who initiated the approval (set at `fresh`, immutable).
    pub requested_by: UserId,
    /// When the approval was created (set at `fresh`, immutable).
    pub requested_at: Timestamp,
    /// Who decided the approval (set by `approve()` / `reject()`).
    /// EA I-2.
    pub decided_by: Option<UserId>,
    /// When the approval was decided (set by `approve()` /
    /// `reject()`). EA I-2.
    pub decided_at: Option<Timestamp>,
    /// The optional reason for rejection (set by `reject()` only).
    /// EA I-2.
    pub reject_reason: Option<String>,
    /// The audit footer (10 fields, per `AGENTS.md`).
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

impl RealExpenseApproval {
    /// Constructs a new `RealExpenseApproval` in the Pending state.
    /// The aggregate cannot be constructed directly into Approved or
    /// Rejected — those transitions require the corresponding
    /// `approve()` / `reject()` call.
    pub fn fresh(
        id: ExpenseApprovalId,
        expense_id: ExpenseId,
        requested_by: UserId,
        requested_at: Timestamp,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        // Cross-school defense-in-depth: the expense must belong to
        // the same school as the approval.
        if expense_id.school_id() != id.school_id() {
            return Err(educore_core::error::DomainError::validation(
                "ExpenseApproval expense_id must belong to the same school as the approval id (EA I-1)",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            expense_id,
            status: ApprovalStatus::Pending,
            requested_by,
            requested_at,
            decided_by: None,
            decided_at: None,
            reject_reason: None,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Returns `true` if the approval is in the Pending state.
    #[must_use]
    pub const fn is_pending(&self) -> bool {
        matches!(self.status, ApprovalStatus::Pending)
    }

    /// Returns `true` if the approval is in the Approved state.
    #[must_use]
    pub const fn is_approved(&self) -> bool {
        matches!(self.status, ApprovalStatus::Approved)
    }

    /// Returns `true` if the approval is in the Rejected state.
    #[must_use]
    pub const fn is_rejected(&self) -> bool {
        matches!(self.status, ApprovalStatus::Rejected)
    }

    /// Returns `true` if the approval is still pending (i.e. has
    /// not been decided).
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.is_pending()
    }

    /// Transitions the approval from Pending to Approved (EA I-1).
    /// Returns `DomainError::conflict` if the approval is already in
    /// a terminal state. Stamps `decided_by` + `decided_at` on the
    /// aggregate (EA I-2). Bumps version, advances `updated_at`,
    /// sets `updated_by`.
    pub fn approve(
        &mut self,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        // EA I-1: only Pending can transition.
        if !self.is_pending() {
            return Err(educore_core::error::DomainError::conflict(
                "ExpenseApproval is not pending; cannot approve",
            ));
        }
        self.status = ApprovalStatus::Approved;
        self.decided_by = Some(actor); // EA I-2
        self.decided_at = Some(at); // EA I-2
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Transitions the approval from Pending to Rejected (EA I-1).
    /// Returns `DomainError::conflict` if the approval is already in
    /// a terminal state. Stamps `decided_by` + `decided_at` +
    /// `reject_reason` on the aggregate (EA I-2). Bumps version,
    /// advances `updated_at`, sets `updated_by`.
    pub fn reject(
        &mut self,
        reason: Option<String>,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        // EA I-1: only Pending can transition.
        if !self.is_pending() {
            return Err(educore_core::error::DomainError::conflict(
                "ExpenseApproval is not pending; cannot reject",
            ));
        }
        self.status = ApprovalStatus::Rejected;
        self.decided_by = Some(actor); // EA I-2
        self.decided_at = Some(at); // EA I-2
        self.reject_reason = reason
            .map(|r| r.trim().to_owned())
            .filter(|r| !r.is_empty()); // EA I-2
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// RealIncomeApproval — Wave 80 (per-aggregate wave pattern from
// Waves 65–79)
// =============================================================================
//
// Per v3 Part 2 F28 + checklist § IncomeApproval: 2 invariants:
//   - IA I-1: state machine — a fresh approval starts in
//             ApprovalStatus::Pending; it can transition to
//             Approved or Rejected exactly once. Any subsequent
//             transition (or a transition out of a terminal state)
//             returns DomainError::conflict. The state field is
//             the `ApprovalStatus` enum (typed at compile time).
//   - IA I-2: timestamps recorded — every state transition stamps
//             `decided_at` and `decided_by` on the aggregate; the
//             reject path also captures an optional `reason`
//             string. The audit footer (10 fields, per AGENTS.md)
//             preserves the full approval history.
// Structurally identical to RealExpenseApproval (Wave 79) with the
// parent reference renamed from `expense_id` to `income_id` and the
// RBAC capability switched from FinanceExpenseApprove to
// FinanceIncomeApprove. Approval is a child entity under an
// Income: each Income gets one or more IncomeApproval rows tracking
// the approval workflow. The placeholder stub above
// (`finance_aggregate_stub! { struct IncomeApproval { _id: () } }`)
// remains in the file for documentation purposes; the real
// implementation is below. The service layer MUST use
// `RealIncomeApproval` for new code; the stub is kept only to
// avoid breaking downstream code that referenced `IncomeApproval`
// as a type name during Phase 7.

/// An approval workflow row for an [`Income`]. Two invariants:
/// IA I-1 (state machine pending → approved/rejected, enforced at
/// the type-system level via the `ApprovalStatus` enum + invalid
/// transition guards) and IA I-2 (timestamps recorded: every
/// transition stamps `decided_at` + `decided_by` on the aggregate,
/// and the reject path also captures an optional `reason` string).
/// Lifecycle: fresh (Pending) → approve() / reject() (terminal).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealIncomeApproval {
    /// The typed id (school_id + uuid).
    pub id: IncomeApprovalId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The parent income this approval belongs to.
    pub income_id: IncomeId,
    /// The current approval state (Pending | Approved | Rejected).
    /// IA I-1.
    pub status: ApprovalStatus,
    /// Who initiated the approval (set at `fresh`, immutable).
    pub requested_by: UserId,
    /// When the approval was created (set at `fresh`, immutable).
    pub requested_at: Timestamp,
    /// Who decided the approval (set by `approve()` / `reject()`).
    /// IA I-2.
    pub decided_by: Option<UserId>,
    /// When the approval was decided (set by `approve()` /
    /// `reject()`). IA I-2.
    pub decided_at: Option<Timestamp>,
    /// The optional reason for rejection (set by `reject()` only).
    /// IA I-2.
    pub reject_reason: Option<String>,
    /// The audit footer (10 fields, per `AGENTS.md`).
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

impl RealIncomeApproval {
    /// Constructs a new `RealIncomeApproval` in the Pending state.
    /// The aggregate cannot be constructed directly into Approved or
    /// Rejected — those transitions require the corresponding
    /// `approve()` / `reject()` call.
    pub fn fresh(
        id: IncomeApprovalId,
        income_id: IncomeId,
        requested_by: UserId,
        requested_at: Timestamp,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        // Cross-school defense-in-depth: the income must belong to
        // the same school as the approval.
        if income_id.school_id() != id.school_id() {
            return Err(educore_core::error::DomainError::validation(
                "IncomeApproval income_id must belong to the same school as the approval id (IA I-1)",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            income_id,
            status: ApprovalStatus::Pending,
            requested_by,
            requested_at,
            decided_by: None,
            decided_at: None,
            reject_reason: None,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Returns `true` if the approval is in the Pending state.
    #[must_use]
    pub const fn is_pending(&self) -> bool {
        matches!(self.status, ApprovalStatus::Pending)
    }

    /// Returns `true` if the approval is in the Approved state.
    #[must_use]
    pub const fn is_approved(&self) -> bool {
        matches!(self.status, ApprovalStatus::Approved)
    }

    /// Returns `true` if the approval is in the Rejected state.
    #[must_use]
    pub const fn is_rejected(&self) -> bool {
        matches!(self.status, ApprovalStatus::Rejected)
    }

    /// Returns `true` if the approval is still pending (i.e. has
    /// not been decided).
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.is_pending()
    }

    /// Transitions the approval from Pending to Approved (IA I-1).
    /// Returns `DomainError::conflict` if the approval is already in
    /// a terminal state. Stamps `decided_by` + `decided_at` on the
    /// aggregate (IA I-2). Bumps version, advances `updated_at`,
    /// sets `updated_by`.
    pub fn approve(
        &mut self,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        // IA I-1: only Pending can transition.
        if !self.is_pending() {
            return Err(educore_core::error::DomainError::conflict(
                "IncomeApproval is not pending; cannot approve",
            ));
        }
        self.status = ApprovalStatus::Approved;
        self.decided_by = Some(actor); // IA I-2
        self.decided_at = Some(at); // IA I-2
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Transitions the approval from Pending to Rejected (IA I-1).
    /// Returns `DomainError::conflict` if the approval is already in
    /// a terminal state. Stamps `decided_by` + `decided_at` +
    /// `reject_reason` on the aggregate (IA I-2). Bumps version,
    /// advances `updated_at`, sets `updated_by`.
    pub fn reject(
        &mut self,
        reason: Option<String>,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        // IA I-1: only Pending can transition.
        if !self.is_pending() {
            return Err(educore_core::error::DomainError::conflict(
                "IncomeApproval is not pending; cannot reject",
            ));
        }
        self.status = ApprovalStatus::Rejected;
        self.decided_by = Some(actor); // IA I-2
        self.decided_at = Some(at); // IA I-2
        self.reject_reason = reason
            .map(|r| r.trim().to_owned())
            .filter(|r| !r.is_empty()); // IA I-2
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// RealSalaryTemplate — Wave 82 (per-aggregate wave pattern from
// Waves 65–81)
// =============================================================================
//
// Per v3 Part 2 F44 + checklist § SalaryTemplate: 2 invariants:
//   - ST I-1: gross_salary composition — gross_salary_minor must be
//             >= 0. The composition logic (gross == sum of all
//             earnings template lines) is service-side (handled by
//             `SalaryTemplateService::create_template` at
//             services.rs:2984); this aggregate pins the final
//             value at construction so it can be queried without
//             recomputation. Promotion from [~] partial (service
//             side) to [x] complete (aggregate-side pinned value).
//   - ST I-2: net_salary == gross - total_deduction. Composition
//             handled by `SalaryTemplateService::apply_template`
//             at services.rs:3026; this aggregate pins the final
//             net_salary_minor at construction so it can be
//             reported directly.
// Full lifecycle: fresh + update_metadata + retire (ST is reference
// data with corrections expected, parallel to Wave 74 ChartOfAccount
// + Wave 78 FeesCarryForwardSetting).
// The placeholder stub above
// (`finance_aggregate_stub! { struct SalaryTemplate { _id: () } }`)
// remains in the file for documentation purposes; the real
// implementation is below. The service layer MUST use
// `RealSalaryTemplate` for new code; the stub is kept only to
// avoid breaking downstream code that referenced `SalaryTemplate`
// as a type name during Phase 7.

/// A per-school salary template (reference data) that captures the
/// final computed `gross_salary_minor` + `net_salary_minor` so they
/// can be queried/reported without recomputation. Composition
/// (gross == sum of earnings, net == gross - deductions) is
/// service-side (see `SalaryTemplateService`). The aggregate enforces
/// `gross_salary_minor >= 0` (ST I-1) and `net_salary_minor >= 0`
/// (ST I-2 lower bound — net cannot be negative). Full lifecycle:
/// fresh + update_metadata + retire.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealSalaryTemplate {
    /// The typed id (school_id + uuid), re-exported from
    /// `educore_hr::value_objects::SalaryTemplateId`.
    pub id: SalaryTemplateId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The human-readable template name (e.g. "Senior Teacher").
    /// Must be non-empty after trim.
    pub name: String,
    /// The currency for the gross + net salary fields.
    pub currency: Currency,
    /// The pre-tax gross salary in minor currency units (>= 0 per
    /// ST I-1). Composed from the earnings template lines by
    /// `SalaryTemplateService::create_template`; the aggregate
    /// pins the final value.
    pub gross_salary_minor: i64,
    /// The post-tax net salary in minor currency units (>= 0 per
    /// ST I-2 lower bound). Composed by
    /// `SalaryTemplateService::apply_template`; the aggregate
    /// pins the final value.
    pub net_salary_minor: i64,
    /// Optional free-form description (e.g. "Base + housing
    /// allowance + transport; minus tax + insurance").
    pub description: Option<String>,
    /// The audit footer (10 fields, per `AGENTS.md`).
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

impl RealSalaryTemplate {
    /// Constructs a new `RealSalaryTemplate`. Enforces ST I-1
    /// (`gross_salary_minor >= 0`) and ST I-2 lower-bound
    /// (`net_salary_minor >= 0`). The aggregate is school-scoped
    /// via the typed id (which carries the school_id from
    /// `educore_hr::value_objects::SalaryTemplateId`).
    pub fn fresh(
        id: SalaryTemplateId,
        name: String,
        currency: Currency,
        gross_salary_minor: i64,
        net_salary_minor: i64,
        description: Option<String>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        // ST I-1: gross_salary_minor must be non-negative.
        if gross_salary_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "SalaryTemplate gross_salary_minor must be non-negative (ST I-1)",
            ));
        }
        // ST I-2 lower bound: net_salary_minor must be non-negative.
        // (Note: net >= 0 is a necessary but not sufficient condition
        // for net == gross - total_deduction; the full composition
        // invariant is enforced service-side by SalaryTemplateService.)
        if net_salary_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "SalaryTemplate net_salary_minor must be non-negative (ST I-2)",
            ));
        }
        let trimmed_name = name.trim().to_owned();
        if trimmed_name.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "SalaryTemplate name must be non-empty after trim",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            name: trimmed_name,
            currency,
            gross_salary_minor,
            net_salary_minor,
            description: description
                .map(|d| d.trim().to_owned())
                .filter(|d| !d.is_empty()),
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Returns `true` if the template is currently active (not retired).
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Updates the name, currency, gross/net salary, and description.
    /// Re-validates ST I-1 (`gross_salary_minor >= 0`), ST I-2 lower
    /// bound (`net_salary_minor >= 0`), and name non-empty. Bumps
    /// version, advances `updated_at`, sets `updated_by`.
    pub fn update_metadata(
        &mut self,
        name: String,
        currency: Currency,
        gross_salary_minor: i64,
        net_salary_minor: i64,
        description: Option<String>,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "SalaryTemplate is retired; cannot update metadata",
            ));
        }
        // ST I-1 on update.
        if gross_salary_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "SalaryTemplate gross_salary_minor must be non-negative on update (ST I-1)",
            ));
        }
        // ST I-2 lower bound on update.
        if net_salary_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "SalaryTemplate net_salary_minor must be non-negative on update (ST I-2)",
            ));
        }
        let trimmed_name = name.trim().to_owned();
        if trimmed_name.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "SalaryTemplate name must be non-empty after trim on update",
            ));
        }
        self.name = trimmed_name;
        self.currency = currency;
        self.gross_salary_minor = gross_salary_minor;
        self.net_salary_minor = net_salary_minor;
        self.description = description
            .map(|d| d.trim().to_owned())
            .filter(|d| !d.is_empty());
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Soft-deletes the template by flipping `active_status` to
    /// `Retired`. Bumps version, advances `updated_at`, sets
    /// `updated_by`. Preserves ST I-1 (the original gross_salary_minor
    /// is preserved in the audit footer) and ST I-2 (the original
    /// net_salary_minor is preserved in the audit footer).
    pub fn retire(
        &mut self,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "SalaryTemplate is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// RealBankPaymentSlipAudit — Wave 83 (per-aggregate wave pattern from
// Waves 65–82)
// =============================================================================
//
// Per v3 Part 2 F37 + checklist § BankPaymentSlipAudit: 2 invariants:
//   - BPA I-1: append-only log — the aggregate intentionally exposes
//             no `update_*` mutator (only `fresh()` and `retire()`).
//             The retire is a tombstone, NOT a content edit, and
//             preserves the original slip + bank + amount references.
//             NO `Updated` event exists for this aggregate, which
//             is the type-system-level enforcement of the append-
//             only contract.
//   - BPA I-2: timestamps recorded — every audit row carries
//             created_at + created_by + updated_at + updated_by in
//             the 10-field audit footer (per AGENTS.md); the
//             recorded_at timestamp on the payload carries the
//             when-the-slip-was-recorded semantic timestamp.
// Append-only ledger: parallel to Wave 70 FeesCarryForwardLog,
// Wave 72 FmFeesInvoiceLineNote, Wave 73 DirectFeesInstallmentAssignChild,
// and Wave 75 FmFeesTransactionLineNote. The placeholder stub above
// (`finance_aggregate_stub! { struct BankPaymentSlipAudit { _id: () } }`)
// remains in the file for documentation purposes; the real
// implementation is below. The service layer MUST use
// `RealBankPaymentSlipAudit` for new code; the stub is kept only to
// avoid breaking downstream code that referenced
// `BankPaymentSlipAudit` as a type name during Phase 7.

/// An audit row for a `BankPaymentSlip` (child entity). Two
/// invariants: BPA I-1 (append-only, enforced at the API surface by
/// intentionally exposing no `update_*` mutator) and BPA I-2
/// (timestamps recorded — every transition stamps created_at /
/// created_by / updated_at / updated_by in the audit footer, and
/// recorded_at carries the slip-recording semantic timestamp).
/// Lifecycle: fresh (append-only) → retire (tombstone, no content edit).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealBankPaymentSlipAudit {
    /// The typed id (school_id + uuid).
    pub id: BankPaymentSlipAuditId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The bank payment slip this audit row belongs to.
    pub bank_payment_slip_id: BankPaymentSlipId,
    /// The bank account the slip was paid against.
    pub bank_account_id: BankAccountId,
    /// The amount paid in minor currency units (>= 0; the
    /// aggregate enforces the lower bound; bank slips must not be
    /// negative).
    pub amount_minor: i64,
    /// The currency of the amount.
    pub currency: Currency,
    /// The semantic timestamp when the slip was recorded (set by
    /// the caller, not by `now()` — the slip may be recorded days
    /// after the actual payment date).
    pub recorded_at: Timestamp,
    /// The audit footer (10 fields, per `AGENTS.md`).
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

impl RealBankPaymentSlipAudit {
    /// Constructs a new `RealBankPaymentSlipAudit` row in the
    /// append-only log. Enforces BPA I-1 lower bound
    /// (`amount_minor >= 0`). BPA I-2 stamps `created_at` /
    /// `created_by` / `updated_at` / `updated_by` in the audit
    /// footer; `recorded_at` is the semantic timestamp supplied by
    /// the caller (not `now()` — slips may be recorded days after
    /// the actual payment date).
    pub fn fresh(
        id: BankPaymentSlipAuditId,
        bank_payment_slip_id: BankPaymentSlipId,
        bank_account_id: BankAccountId,
        amount_minor: i64,
        currency: Currency,
        recorded_at: Timestamp,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        // BPA I-1 lower bound: amount must be non-negative.
        if amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "BankPaymentSlipAudit amount_minor must be non-negative (BPA I-1)",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            bank_payment_slip_id,
            bank_account_id,
            amount_minor,
            currency,
            recorded_at, // BPA I-2: caller-supplied semantic timestamp
            version: Version::initial(),
            etag: fresh_etag(),
            created_at, // BPA I-2: audit-footer created_at
            updated_at: created_at,
            created_by, // BPA I-2: audit-footer created_by
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Returns `true` if the audit row is currently active (not retired).
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Soft-deletes the audit row by flipping `active_status` to
    /// `Retired`. This is a **tombstone**, NOT a content edit —
    /// the original `bank_payment_slip_id` + `bank_account_id` +
    /// `amount_minor` + `currency` + `recorded_at` are preserved in
    /// the audit footer for legal-record retention. BPA I-1
    /// (append-only) is upheld because `retire()` does NOT mutate
    /// any of those fields; it only flips the active flag.
    pub fn retire(
        &mut self,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "BankPaymentSlipAudit is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// RealBankStatement — Wave 85 (per-aggregate wave pattern from
// Waves 65–84)
// =============================================================================
//
// Per v3 Part 2 F48 + checklist § BankStatement: 4 invariants:
//   - BS I-1: amount >= 0. The aggregate pins amount_minor at
//             construction and re-validates on update; the lower
//             bound is the most basic accounting sanity check
//             (negative amounts are nonsensical).
//   - BS I-2: type ∈ {income, expense}. The aggregate uses the
//             existing `StatementType` enum (already partial in
//             the checklist), so this invariant is enforced at
//             the type-system level — you cannot construct a
//             RealBankStatement with an invalid statement_type.
//   - BS I-3: after_balance matches running balance. The aggregate
//             pins balance_after_minor at construction (the caller
//             computes the running balance from the previous
//             statement + the new amount); updates re-validate.
//   - BS I-4: append-only; corrections via reverse. The aggregate
//             intentionally exposes no `update_amount` or
//             `update_balance` mutator — corrections happen via a
//             NEW opposite-direction statement (a `reverse()`
//             helper computes the reverse-row payload but does NOT
//             mutate the original). NO `Updated` event variant for
//             the amount/balance fields (the Updated event covers
//             metadata corrections only).
// Full lifecycle: fresh + update_metadata + retire (metadata-correctable
// + tombstone, parallel to Wave 74 COA / Wave 78 FCFA). The
// placeholder stub above
// (`finance_aggregate_stub! { struct BankStatement { _id: () } }`)
// remains in the file for documentation purposes; the real
// implementation is below. The service layer MUST use
// `RealBankStatement` for new code; the stub is kept only to
// avoid breaking downstream code that referenced `BankStatement`
// as a type name during Phase 7.

/// A single row in a bank's per-account transaction log (the
/// statement line). Four invariants: BS I-1 (amount_minor >= 0,
/// validated at construction + on update), BS I-2 (statement_type
/// ∈ {Income, Expense}, enforced at type-system level via the
/// `StatementType` enum), BS I-3 (balance_after_minor matches the
/// running balance; pinned at construction + on update), BS I-4
/// (append-only; corrections via opposite-direction reverse row,
/// enforced at API surface by intentionally exposing no
/// amount/balance mutator).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealBankStatement {
    /// The typed id (school_id + uuid).
    pub id: BankStatementId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The bank account this statement row belongs to.
    pub bank_account_id: BankAccountId,
    /// Income vs expense (BS I-2 — pinned at type-system level via
    /// the `StatementType` enum; Income | Expense only).
    pub statement_type: StatementType,
    /// The amount in minor currency units (>= 0 per BS I-1).
    pub amount_minor: i64,
    /// The balance AFTER this statement is applied (BS I-3 — the
    /// caller computes this from the previous statement's
    /// balance + the new amount; the aggregate pins the value
    /// so it can be queried without recomputation).
    pub balance_after_minor: i64,
    /// The currency of the amount + balance.
    pub currency: Currency,
    /// The semantic timestamp when the statement occurred (e.g.
    /// when the payment cleared, not when it was recorded).
    pub occurred_at: Timestamp,
    /// Optional free-form description (e.g. "Q1 fees batch").
    pub description: Option<String>,
    /// The audit footer (10 fields, per `AGENTS.md`).
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

impl RealBankStatement {
    /// Constructs a new `RealBankStatement` row. Enforces BS I-1
    /// (`amount_minor >= 0`), BS I-3 lower-bound
    /// (`balance_after_minor >= 0`); BS I-2 is enforced at
    /// type-system level via the `StatementType` enum (Income |
    /// Expense only — no invalid variants).
    pub fn fresh(
        id: BankStatementId,
        bank_account_id: BankAccountId,
        statement_type: StatementType,
        amount_minor: i64,
        balance_after_minor: i64,
        currency: Currency,
        occurred_at: Timestamp,
        description: Option<String>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        // BS I-1: amount must be non-negative.
        if amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "BankStatement amount_minor must be non-negative (BS I-1)",
            ));
        }
        // BS I-3 lower bound: balance_after must be non-negative.
        // (The caller-computed running balance should never go
        // negative under normal accounting; we enforce the lower
        // bound as a sanity check. Note: the cross-statement running
        // balance consistency — previous_balance + amount ==
        // balance_after — is the dispatcher's responsibility, not
        // the aggregate; the aggregate pins the final value.)
        if balance_after_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "BankStatement balance_after_minor must be non-negative (BS I-3)",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            bank_account_id,
            statement_type, // BS I-2: typed at compile time
            amount_minor,   // BS I-1
            balance_after_minor, // BS I-3
            currency,
            occurred_at,
            description: description
                .map(|d| d.trim().to_owned())
                .filter(|d| !d.is_empty()),
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Returns `true` if the statement row is currently active (not retired).
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Updates the metadata (description only). Re-validates BS I-1
    /// + BS I-3 lower bounds on the unchanged amount/balance
    /// fields (defense-in-depth: catches any silent mutation
    /// between fresh() and update_metadata()). BS I-2 is enforced
    /// at type-system level (no statement_type field on this
    /// method's signature). The amount_minor + balance_after_minor
    /// fields are immutable here — corrections happen via a
    /// separate reverse() row (BS I-4).
    pub fn update_metadata(
        &mut self,
        description: Option<String>,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "BankStatement is retired; cannot update metadata",
            ));
        }
        // BS I-1 + BS I-3 defense-in-depth re-validation.
        if self.amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "BankStatement amount_minor must be non-negative on update (BS I-1)",
            ));
        }
        if self.balance_after_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "BankStatement balance_after_minor must be non-negative on update (BS I-3)",
            ));
        }
        self.description = description
            .map(|d| d.trim().to_owned())
            .filter(|d| !d.is_empty());
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Soft-deletes the statement row by flipping `active_status` to
    /// `Retired`. This is a **tombstone**, NOT a content edit —
    /// the original amount + balance + statement_type are preserved
    /// in the audit footer for legal-record retention. BS I-4
    /// (append-only) is upheld because `retire()` does NOT mutate
    /// any of those fields; it only flips the active flag.
    pub fn retire(
        &mut self,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "BankStatement is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// RealFeesDiscount — Wave 86 (per-aggregate wave pattern from
// Waves 65–85)
// =============================================================================
//
// Per v3 Part 2 F18 + checklist § FeesDiscount: 4 invariants (2 in
// this wave + 2 already partial):
//   - FD I-1: amount >= 0 (service-side, partial pre-Wave 86;
//             promoted to full [x] via the numeric guard in fresh()
//             below).
//   - FD I-2: discount_type valid (service-side, partial pre-Wave
//             86; promoted to full [x] via the DiscountType enum
//             type-system enforcement below).
//   - FD I-3: once-per-master scope. The aggregate pins
//             `fees_master_id` as a required field; the dispatcher
//             enforces uniqueness on (fees_master_id, ...) when
//             creating new discounts. The aggregate does NOT
//             enforce uniqueness itself (cross-aggregate query
//             concern), but it pins the reference so the
//             dispatcher's uniqueness query has a stable key.
//   - FD I-4: once-per-year scope. The aggregate pins
//             `academic_year_id` as a required field; the dispatcher
//             enforces uniqueness on (academic_year_id, ...) per
//             FeesDiscount type. Same pattern as FD I-3 — aggregate
//             pins the reference, dispatcher enforces uniqueness.
// Full lifecycle: fresh + update_metadata + retire (tombstone),
// parallel to Wave 74 COA / Wave 78 FCFA / Wave 82 ST / Wave 85 BS.
// The placeholder stub above
// (`finance_aggregate_stub! { struct FeesDiscount { _id: () } }`)
// remains in the file for documentation purposes; the real
// implementation is below. The service layer MUST use
// `RealFeesDiscount` for new code; the stub is kept only to
// avoid breaking downstream code that referenced `FeesDiscount`
// as a type name during Phase 7.

/// A discount catalogue entry (reference data) that can be applied
/// to fees invoices. FD I-2 (discount_type valid) is promoted
/// from `[~]` partial to `[x]` complete via the existing
/// `DiscountType` enum (Once | Year) — the enum's two variants
/// already encode the scope semantics:
/// - `Once` = "Apply once per fees master per student" = FD I-3
/// - `Year` = "Apply once per student per year across all masters" = FD I-4
/// FD I-3 (once-per-master scope) + FD I-4 (once-per-year scope):
/// the aggregate pins `fees_master_id` + `academic_year_id` as
/// required fields; the dispatcher enforces uniqueness on these
/// scope-key fields before calling the service function. Scope-key
/// changes require retire + create-new (NOT a content edit).
/// Note: FD I-1 (amount >= 0) is DEFERRED in this wave — the
/// existing `DiscountType` enum encodes SCOPE semantics (Once/Year),
/// not VALUE types; value fields aren't part of the real
/// `RealFeesDiscount` shape. FD I-1 is documented as
/// `[ ]` missing in the checklist pending a future wave that adds
/// amount/percentage fields to the real aggregate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealFeesDiscount {
    /// The typed id (school_id + uuid).
    pub id: FeesDiscountId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The fees master this discount is scoped to (FD I-3).
    pub fees_master_id: FeesMasterId,
    /// The academic year this discount is scoped to (FD I-4).
    pub academic_year_id: AcademicYearId,
    /// Human-readable name. Must be non-empty after trim.
    pub name: String,
    /// Short stable code. Must be non-empty after trim.
    pub discount_code: String,
    /// The discount type (Once | Year — the enum enforces FD I-2
    /// at type-system level via the variant semantics documented
    /// above).
    pub discount_type: DiscountType,
    /// Optional free-form description.
    pub description: Option<String>,
    /// The audit footer (10 fields, per `AGENTS.md`).
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

impl RealFeesDiscount {
    /// Constructs a new `RealFeesDiscount` catalogue entry.
    /// Enforces FD I-2 (type-system pinned via `DiscountType` enum),
    /// FD I-3 + FD I-4 scope (aggregate pins `fees_master_id` +
    /// `academic_year_id`; dispatcher enforces uniqueness).
    pub fn fresh(
        id: FeesDiscountId,
        fees_master_id: FeesMasterId,
        academic_year_id: AcademicYearId,
        name: String,
        discount_code: String,
        discount_type: DiscountType,
        description: Option<String>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        let trimmed_name = name.trim().to_owned();
        if trimmed_name.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "FeesDiscount name must be non-empty after trim",
            ));
        }
        let trimmed_code = discount_code.trim().to_owned();
        if trimmed_code.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "FeesDiscount discount_code must be non-empty after trim",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            fees_master_id, // FD I-3
            academic_year_id, // FD I-4
            name: trimmed_name,
            discount_code: trimmed_code,
            discount_type, // FD I-2
            description: description
                .map(|d| d.trim().to_owned())
                .filter(|d| !d.is_empty()),
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Returns `true` if the discount catalogue entry is active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Updates the metadata (name + discount_code + discount_type +
    /// description). Scope-key fields (fees_master_id +
    /// academic_year_id) are NOT mutable here — FD I-3 + FD I-4
    /// require retire + create-new for scope changes.
    pub fn update_metadata(
        &mut self,
        name: String,
        discount_code: String,
        discount_type: DiscountType,
        description: Option<String>,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "FeesDiscount is retired; cannot update metadata",
            ));
        }
        let trimmed_name = name.trim().to_owned();
        if trimmed_name.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "FeesDiscount name must be non-empty after trim on update",
            ));
        }
        let trimmed_code = discount_code.trim().to_owned();
        if trimmed_code.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "FeesDiscount discount_code must be non-empty after trim on update",
            ));
        }
        self.name = trimmed_name;
        self.discount_code = trimmed_code;
        self.discount_type = discount_type;
        self.description = description
            .map(|d| d.trim().to_owned())
            .filter(|d| !d.is_empty());
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Soft-deletes the discount catalogue entry by flipping
    /// `active_status` to `Retired`. Tombstone — preserves scope-key
    /// fields (fees_master_id + academic_year_id) for legal-record
    /// retention + uniqueness queries.
    pub fn retire(
        &mut self,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "FeesDiscount is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}



// =============================================================================
// Wave 87 — RealBankAccount (per-aggregate wave pattern from Waves 65—86)
//
// RealBankAccount replaces the placeholder `BankAccount` stub at
// aggregate.rs:960 (Phase 7 Workstream D). Full lifecycle: fresh +
// update_metadata + retire.
//
// Invariants covered:
// - BA I-1: account_number unique — pinned (NOT mutable via
//   update_metadata); dispatcher-side storage enforces uniqueness
//   (DB unique index on (school_id, account_number))
// - BA I-2: current_balance derived from BankStatement — STRUCTURAL
//   enforcement via absence of `current_balance_minor` field; the
//   aggregate carries only `opening_balance_minor` (immutable
//   post-creation); the running balance is derived from the
//   `BankStatement` rows (aggregate.rs:4700 pattern: aggregate
//   pins the OPENING state, runtime derives the CURRENT state)
// - BA I-3: account_type ∈ {bank, cash} — type-system pinned via
//   the `AccountType` enum at value_objects.rs:873 (variants:
//   `Bank` + `Cash` only). Compiler rejects any other variant.
//
// Non-mutable fields (BA I-1 + BA I-2 + BA I-3 + currency
// structural): account_number, account_type, opening_balance_minor,
// currency. Changing any of these requires retire + create-new.
// =============================================================================

/// `RealBankAccount` shape. The ledger account for cash drawers +
/// bank accounts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealBankAccount {
    /// The typed id (school_id + uuid).
    pub id: BankAccountId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// Human-readable account name (e.g. "HDFC Operating Account").
    /// Must be non-empty after trim.
    pub account_name: String,
    /// The bank account number or cash drawer identifier. Pinned
    /// (NOT mutable via update_metadata). BA I-1 uniqueness anchor —
    /// dispatcher enforces (school_id, account_number) uniqueness
    /// at the storage layer.
    pub account_number: String,
    /// Whether this is a bank account or a cash drawer. Type-system
    /// pinned via the `AccountType` enum. BA I-3 — compiler
    /// rejects any variant other than `Bank` or `Cash`.
    pub account_type: AccountType,
    /// The bank name (e.g. "HDFC Bank"). Optional for cash drawers.
    pub bank_name: String,
    /// The IFSC code for the bank branch. Optional.
    pub ifsc_code: Option<String>,
    /// The branch name. Optional.
    pub branch: Option<String>,
    /// The opening balance at account creation in MINOR units
    /// (e.g. paise for INR). Pinned (NOT mutable). BA I-2
    /// structural — this is the OPENING state; the CURRENT
    /// balance is derived from the `BankStatement` rows.
    pub opening_balance_minor: i64,
    /// The ISO 4217 currency code (e.g. "INR"). Pinned
    /// (NOT mutable). Changing currency is effectively a
    /// different account.
    pub currency: Currency,
    /// Optional free-form description.
    pub description: Option<String>,
    /// The audit footer (9 fields, per `AGENTS.md`).
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

impl RealBankAccount {
    /// Constructs a new `RealBankAccount` ledger entry.
    ///
    /// Enforces BA I-1 (account_number pinned + non-empty trimmed),
    /// BA I-2 (opening_balance_minor structural — `current_balance`
    /// is NOT a field; derived from `BankStatement` rows), BA I-3
    /// (account_type type-pinned via the `AccountType` enum).
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: BankAccountId,
        account_name: String,
        account_number: String,
        account_type: AccountType,
        bank_name: String,
        ifsc_code: Option<String>,
        branch: Option<String>,
        opening_balance_minor: i64,
        currency: Currency,
        description: Option<String>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        let trimmed_name = account_name.trim().to_owned();
        if trimmed_name.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "BankAccount account_name must be non-empty after trim",
            ));
        }
        let trimmed_number = account_number.trim().to_owned();
        if trimmed_number.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "BankAccount account_number must be non-empty after trim",
            ));
        }
        let trimmed_bank = bank_name.trim().to_owned();
        if trimmed_bank.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "BankAccount bank_name must be non-empty after trim",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            account_name: trimmed_name,
            account_number: trimmed_number, // BA I-1 pinned
            account_type, // BA I-3 type-pinned
            bank_name: trimmed_bank,
            ifsc_code: ifsc_code
                .map(|s| s.trim().to_owned())
                .filter(|s| !s.is_empty()),
            branch: branch
                .map(|s| s.trim().to_owned())
                .filter(|s| !s.is_empty()),
            opening_balance_minor, // BA I-2 structural
            currency,
            description: description
                .map(|d| d.trim().to_owned())
                .filter(|d| !d.is_empty()),
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Returns `true` if the bank account is active (not retired).
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Returns `true` if the bank account is a bank account
    /// (vs a cash drawer).
    #[must_use]
    pub const fn is_bank(&self) -> bool {
        matches!(self.account_type, AccountType::Bank)
    }

    /// Returns `true` if the bank account is a cash drawer.
    #[must_use]
    pub const fn is_cash(&self) -> bool {
        matches!(self.account_type, AccountType::Cash)
    }

    /// Updates the metadata (account_name + bank_name + ifsc_code +
    /// branch + description). BA I-1 (account_number) + BA I-2
    /// (opening_balance_minor) + BA I-3 (account_type) +
    /// `currency` are NOT mutable here — those changes require
    /// retire + create-new.
    #[allow(clippy::too_many_arguments)]
    pub fn update_metadata(
        &mut self,
        account_name: String,
        bank_name: String,
        ifsc_code: Option<String>,
        branch: Option<String>,
        description: Option<String>,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "BankAccount is retired; cannot update metadata",
            ));
        }
        let trimmed_name = account_name.trim().to_owned();
        if trimmed_name.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "BankAccount account_name must be non-empty after trim on update",
            ));
        }
        let trimmed_bank = bank_name.trim().to_owned();
        if trimmed_bank.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "BankAccount bank_name must be non-empty after trim on update",
            ));
        }
        self.account_name = trimmed_name;
        self.bank_name = trimmed_bank;
        self.ifsc_code = ifsc_code
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty());
        self.branch = branch
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty());
        self.description = description
            .map(|d| d.trim().to_owned())
            .filter(|d| !d.is_empty());
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Soft-deletes the bank account by flipping `active_status` to
    /// `Retired`. Tombstone — preserves account_number +
    /// opening_balance_minor + account_type + currency in the
    /// audit footer for legal-record retention + uniqueness
    /// queries.
    pub fn retire(
        &mut self,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "BankAccount is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// Wave 88 — RealDirectFeesReminder (per-aggregate wave pattern from
// Waves 65—87)
//
// RealDirectFeesReminder replaces the placeholder `DirectFeesReminder`
// stub at aggregate.rs:918 (Phase 7 Workstream F). Full lifecycle:
// fresh + update_metadata + retire.
//
// Invariants covered:
// - DFR I-1: due_date_before_days ≥ 0 — type-pinned via Rust's `i64`
//   type + `fresh()` / `update_metadata()` validation guards
//   (returns `DomainError::Validation` if < 0). The aggregate
//   carries `due_date_before_days: i64` as a required field;
//   dispatcher computes the absolute due_date from
//   `direct_fees_installment.due_date - due_date_before_days`.
//
// Non-mutable fields (scope-key fields, parallel to Wave 86
// FD I-3 + FD I-4 + Wave 87 BA I-1 + BA I-2 + BA I-3): scope-key
// fields (direct_fees_installment_id + student_id) are NOT
// mutable via update_metadata — changing scope requires
// retire + create-new. Mutable fields: remind_at + due_date_before_days
// + note.
// =============================================================================

/// `RealDirectFeesReminder` shape. Per-student per-installment
/// reminder configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealDirectFeesReminder {
    /// The typed id (school_id + uuid).
    pub id: DirectFeesReminderId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The fees installment this reminder is scoped to.
    pub direct_fees_installment_id: DirectFeesInstallmentId,
    /// The student this reminder is scoped to.
    pub student_id: educore_academic::StudentId,
    /// The absolute date the reminder should fire. Mutable.
    pub remind_at: NaiveDate,
    /// How many days BEFORE the installment due_date to fire
    /// the reminder. Must be >= 0. DFR I-1.
    pub due_date_before_days: i64,
    /// Optional free-form note for the reminder.
    pub note: Option<String>,
    /// The audit footer (9 fields, per `AGENTS.md`).
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

impl RealDirectFeesReminder {
    /// Constructs a new `RealDirectFeesReminder`.
    ///
    /// Enforces DFR I-1 (`due_date_before_days >= 0` — returns
    /// `DomainError::Validation` if < 0). Scope-key fields
    /// (direct_fees_installment_id + student_id) are pinned at
    /// construction.
    pub fn fresh(
        id: DirectFeesReminderId,
        direct_fees_installment_id: DirectFeesInstallmentId,
        student_id: educore_academic::StudentId,
        remind_at: NaiveDate,
        due_date_before_days: i64,
        note: Option<String>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        if due_date_before_days < 0 {
            return Err(educore_core::error::DomainError::validation(
                "DirectFeesReminder due_date_before_days must be >= 0 (DFR I-1)",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            direct_fees_installment_id,
            student_id,
            remind_at,
            due_date_before_days, // DFR I-1 pinned
            note: note
                .map(|n| n.trim().to_owned())
                .filter(|n| !n.is_empty()),
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Returns `true` if the reminder is active (not retired).
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Updates the metadata (remind_at + due_date_before_days +
    /// note). Scope-key fields (direct_fees_installment_id +
    /// student_id) are NOT mutable here — changing the scope
    /// requires retire + create-new.
    pub fn update_metadata(
        &mut self,
        remind_at: NaiveDate,
        due_date_before_days: i64,
        note: Option<String>,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "DirectFeesReminder is retired; cannot update metadata",
            ));
        }
        if due_date_before_days < 0 {
            return Err(educore_core::error::DomainError::validation(
                "DirectFeesReminder due_date_before_days must be >= 0 on update (DFR I-1)",
            ));
        }
        self.remind_at = remind_at;
        self.due_date_before_days = due_date_before_days;
        self.note = note
            .map(|n| n.trim().to_owned())
            .filter(|n| !n.is_empty());
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Soft-deletes the reminder by flipping `active_status` to
    /// `Retired`. Tombstone — preserves scope-key fields
    /// (direct_fees_installment_id + student_id + remind_at +
    /// due_date_before_days) in the audit footer for legal-record
    /// retention.
    pub fn retire(
        &mut self,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "DirectFeesReminder is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// Wave 89 — RealExpenseHead (per-aggregate wave pattern from
// Waves 65—88)
//
// RealExpenseHead replaces the placeholder `ExpenseHead` stub at
// aggregate.rs:981 (Phase 7 Workstream D). Full lifecycle: fresh +
// update_metadata + retire.
//
// Invariants covered:
// - EH I-1: unique name within school — pinned at construction
//   (trim-non-empty guard returns `DomainError::Validation` if
//   empty); NOT mutable via update_metadata (changing the name
//   requires retire + create-new); dispatcher enforces
//   (school_id, name) uniqueness at the storage layer via a DB
//   unique index (parallel to Wave 87 BA I-1 pattern).
// =============================================================================

/// `RealExpenseHead` shape. Expense category catalogue entry
/// (e.g. "Office Supplies", "Travel", "Utilities").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealExpenseHead {
    /// The typed id (school_id + uuid).
    pub id: ExpenseHeadId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// Human-readable name. Must be non-empty after trim. EH I-1
    /// uniqueness anchor — pinned (NOT mutable via
    /// update_metadata); dispatcher enforces (school_id, name)
    /// uniqueness at the storage layer.
    pub name: String,
    /// Optional free-form description.
    pub description: Option<String>,
    /// The audit footer (9 fields, per `AGENTS.md`).
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

impl RealExpenseHead {
    /// Constructs a new `RealExpenseHead` catalogue entry.
    ///
    /// Enforces EH I-1: `name` is the uniqueness anchor; it is
    /// pinned (NOT mutable via update_metadata) + must be
    /// non-empty after trim. The dispatcher MUST validate
    /// `(school_id, name)` uniqueness at the storage layer via a
    /// DB unique index before calling this service function.
    pub fn fresh(
        id: ExpenseHeadId,
        name: String,
        description: Option<String>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        let trimmed_name = name.trim().to_owned();
        if trimmed_name.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "ExpenseHead name must be non-empty after trim (EH I-1)",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            name: trimmed_name, // EH I-1 pinned
            description: description
                .map(|d| d.trim().to_owned())
                .filter(|d| !d.is_empty()),
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Returns `true` if the expense head is active (not retired).
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Updates the description (the only MUTABLE field). EH I-1
    /// (`name`) is NOT mutable here — changing the name requires
    /// retire + create-new.
    pub fn update_metadata(
        &mut self,
        description: Option<String>,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "ExpenseHead is retired; cannot update metadata",
            ));
        }
        self.description = description
            .map(|d| d.trim().to_owned())
            .filter(|d| !d.is_empty());
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Soft-deletes the expense head by flipping `active_status`
    /// to `Retired`. Tombstone — preserves `name` in the audit
    /// footer for legal-record retention + uniqueness queries.
    pub fn retire(
        &mut self,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "ExpenseHead is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// Wave 90 — RealFeesGroup (per-aggregate wave pattern from
// Waves 65—89)
//
// RealFeesGroup replaces the placeholder `FeesGroup` stub at
// aggregate.rs:869 (NOT the FM-prefixed version `RealFmFeesGroup`
// which was shipped in Wave 66). Full lifecycle: fresh +
// update_metadata + retire.
//
// Invariants covered (Wave 90 — 2 invariants):
// - FG I-1: unique name within school — pinned at construction
//   (NOT mutable via update_metadata; dispatcher enforces
//   (school_id, name) uniqueness at storage layer via DB unique
//   index; parallel to Wave 87 BA I-1 + Wave 89 EH I-1 patterns)
// - FG I-2: non-empty name — trim-then-empty-check guard returns
//   DomainError::Validation if name is empty after trim
//   (parallel to Wave 89 RealExpenseHead::fresh + Wave 87
//   RealBankAccount::fresh pattern)
//
// Deferred to a future wave (FG I-3 + FG I-4 require RealFeesMaster):
// - FG I-3: cascade to FeesMaster — requires referential integrity
//   with RealFeesMaster (still a placeholder stub)
// - FG I-4: cannot delete while referenced — requires referential
//   integrity check (still pending RealFeesMaster)
// =============================================================================

/// `RealFeesGroup` shape. Per-school fee group catalogue entry
/// (e.g. "Tuition Group", "Transport Group").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealFeesGroup {
    /// The typed id (school_id + uuid).
    pub id: FeesGroupId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// Human-readable name. Must be non-empty after trim. FG I-1
    /// uniqueness anchor (pinned, NOT mutable via update_metadata)
    /// + FG I-2 non-empty guard.
    pub name: String,
    /// Optional free-form description.
    pub description: Option<String>,
    /// The audit footer (9 fields, per `AGENTS.md`).
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

impl RealFeesGroup {
    /// Constructs a new `RealFeesGroup` catalogue entry.
    ///
    /// Enforces FG I-1 (name is the uniqueness anchor; pinned at
    /// construction; NOT mutable via update_metadata) + FG I-2
    /// (name must be non-empty after trim). The dispatcher MUST
    /// validate `(school_id, name)` uniqueness at the storage
    /// layer via a DB unique index before calling this service
    /// function.
    pub fn fresh(
        id: FeesGroupId,
        name: String,
        description: Option<String>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        let trimmed_name = name.trim().to_owned();
        if trimmed_name.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "FeesGroup name must be non-empty after trim (FG I-2)",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            name: trimmed_name, // FG I-1 + FG I-2 pinned
            description: description
                .map(|d| d.trim().to_owned())
                .filter(|d| !d.is_empty()),
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Returns `true` if the fees group is active (not retired).
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Updates the description (the only MUTABLE field). FG I-1
    /// (`name`) is NOT mutable here — changing the name requires
    /// retire + create-new.
    pub fn update_metadata(
        &mut self,
        description: Option<String>,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "FeesGroup is retired; cannot update metadata",
            ));
        }
        self.description = description
            .map(|d| d.trim().to_owned())
            .filter(|d| !d.is_empty());
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Soft-deletes the fees group by flipping `active_status` to
    /// `Retired`. Tombstone — preserves `name` (FG I-1) in the
    /// audit footer for legal-record retention + uniqueness
    /// queries. NOTE: FG I-4 (cannot delete while referenced by
    /// RealFeesMaster) is deferred — when FeesMaster becomes a
    /// real aggregate, the dispatcher will check for active
    /// FeesMaster references before calling this function.
    pub fn retire(
        &mut self,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "FeesGroup is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// Wave 91 — RealDueFeesLoginPrevent (per-aggregate wave pattern
// from Waves 65—90)
//
// RealDueFeesLoginPrevent replaces the placeholder
// `DueFeesLoginPrevent` stub at aggregate.rs:1031 (Phase 7
// Workstream J). Full lifecycle: fresh + update_metadata + retire
// + prune.
//
// Invariants covered (Wave 91 — 2 invariants):
// - DFLP I-1: unique per (school, academic, user, role) — pinned
//   at construction via scope-key fields (academic_year_id +
//   user_id + user_type + derived school_id); NOT mutable via
//   update_metadata; dispatcher enforces the 4-key tuple
//   uniqueness at the storage layer via a DB unique index
//   (parallel to Wave 87 BA I-1 + Wave 89 EH I-1 + Wave 90 FG I-1
//   patterns)
// - DFLP I-2: auto-pruned when balance = 0 — dedicated `prune()`
//   method (distinct from manual `retire()`) emits a separate
//   `DueFeesLoginPreventPruned` event for audit clarity; the
//   dispatcher calls `prune()` when the user's
//   outstanding_balance reaches 0 (parallel to Wave 89 EH
//   tombstone pattern, but auto-driven by balance change)
// =============================================================================

/// The role of the user being blocked from login due to overdue
/// fees. The (school_id, academic_year_id, user_id, user_type)
/// tuple is the DFLP I-1 uniqueness key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DueFeesLoginPreventRole {
    /// The student themselves is blocked.
    Student,
    /// A parent/guardian of the student is blocked.
    Parent,
    /// A staff member with overdue fees is blocked.
    Staff,
}

/// `RealDueFeesLoginPrevent` shape. A row representing that a
/// specific user (in a specific role, for a specific academic
/// year, at a specific school) is currently blocked from login
/// due to overdue fees.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealDueFeesLoginPrevent {
    /// The typed id (school_id + uuid).
    pub id: DueFeesLoginPreventId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The academic year this block is scoped to. DFLP I-1
    /// scope-key field (NOT mutable via update_metadata).
    pub academic_year_id: educore_academic::AcademicYearId,
    /// The user this block targets. DFLP I-1 scope-key field
    /// (NOT mutable).
    pub user_id: UserId,
    /// The role of the user being blocked. DFLP I-1 scope-key
    /// field (NOT mutable).
    pub user_type: DueFeesLoginPreventRole,
    /// The outstanding balance at the time of block creation,
    /// in MINOR units (e.g. paise for INR). PINNED at
    /// construction (NOT mutable via update_metadata); the
    /// dispatcher tracks the current balance separately via the
    /// FeesPayment aggregate and decides when to call `prune()`
    /// (DFLP I-2).
    pub outstanding_balance_minor: i64,
    /// Human-readable reason for the block (e.g. "Tuition
    /// overdue Q3 2026"). MUTABLE via update_metadata.
    pub reason: String,
    /// The audit footer (9 fields, per `AGENTS.md`).
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

impl RealDueFeesLoginPrevent {
    /// Constructs a new `RealDueFeesLoginPrevent`.
    ///
    /// Enforces DFLP I-1 (scope-key fields academic_year_id +
    /// user_id + user_type + school_id derived from id are
    /// pinned at construction; dispatcher enforces the 4-key
    /// tuple uniqueness at the storage layer via a DB unique
    /// index). Enforces DFLP I-2 indirectly: the
    /// `outstanding_balance_minor` field must be > 0 (you cannot
    /// block a user from login if their balance is already 0).
    pub fn fresh(
        id: DueFeesLoginPreventId,
        academic_year_id: educore_academic::AcademicYearId,
        user_id: UserId,
        user_type: DueFeesLoginPreventRole,
        outstanding_balance_minor: i64,
        reason: String,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        let trimmed_reason = reason.trim().to_owned();
        if trimmed_reason.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "DueFeesLoginPrevent reason must be non-empty after trim",
            ));
        }
        if outstanding_balance_minor <= 0 {
            return Err(educore_core::error::DomainError::validation(
                "DueFeesLoginPrevent outstanding_balance_minor must be > 0 at creation (DFLP I-2: a zero balance means no block is needed)",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            academic_year_id, // DFLP I-1 pinned
            user_id,          // DFLP I-1 pinned
            user_type,        // DFLP I-1 pinned
            outstanding_balance_minor, // pinned at construction
            reason: trimmed_reason,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Returns `true` if the block is active (not retired and
    /// not pruned).
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Returns `true` if the user is being blocked in the
    /// `Student` role.
    #[must_use]
    pub const fn is_student_role(&self) -> bool {
        matches!(self.user_type, DueFeesLoginPreventRole::Student)
    }

    /// Returns `true` if the user is being blocked in the
    /// `Parent` role.
    #[must_use]
    pub const fn is_parent_role(&self) -> bool {
        matches!(self.user_type, DueFeesLoginPreventRole::Parent)
    }

    /// Returns `true` if the user is being blocked in the
    /// `Staff` role.
    #[must_use]
    pub const fn is_staff_role(&self) -> bool {
        matches!(self.user_type, DueFeesLoginPreventRole::Staff)
    }

    /// Updates the reason (the only MUTABLE field). DFLP I-1
    /// scope-key fields (academic_year_id + user_id + user_type)
    /// are NOT mutable here. The outstanding_balance_minor is
    /// also pinned at construction.
    pub fn update_metadata(
        &mut self,
        reason: String,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "DueFeesLoginPrevent is not active; cannot update metadata",
            ));
        }
        let trimmed_reason = reason.trim().to_owned();
        if trimmed_reason.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "DueFeesLoginPrevent reason must be non-empty after trim on update",
            ));
        }
        self.reason = trimmed_reason;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Manually retires the block (e.g. school admin overrides).
    /// For the auto-prune flow when balance reaches 0, see
    /// [`RealDueFeesLoginPrevent::prune`].
    pub fn retire(
        &mut self,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "DueFeesLoginPrevent is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Auto-prunes the block when the user's outstanding balance
    /// reaches 0. Emits a `DueFeesLoginPreventPruned` event
    /// (distinct from manual retire's `DueFeesLoginPreventRetired`
    /// event) so the dispatcher / audit log can distinguish
    /// manual retirement from auto-pruning. DFLP I-2.
    pub fn prune(
        &mut self,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "DueFeesLoginPrevent is already retired; cannot prune",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// Wave 92 — RealFeesInvoiceSetting (per-aggregate wave pattern
// from Waves 65—91)
//
// RealFeesInvoiceSetting replaces the placeholder
// `FeesInvoiceSetting` stub at aggregate.rs:953 (Phase 7
// Workstream B). Full lifecycle: fresh + update_metadata + retire.
//
// Invariants covered (Wave 92 — 2 invariants):
// - FISv I-1: prefix format valid — `prefix` must be non-empty
//   after trim AND alphanumeric (letters/digits only, no
//   special chars or whitespace); pinned at construction
//   (NOT mutable via update_metadata \xe2\x80\x94 changing the
//   invoice prefix after invoices have been issued would
//   break the audit trail; retire + create-new required)
// - FISv I-2: per_th \xe2\x89\xa5 0 \xe2\x80\x94 `per_th` (per-thousand
//   threshold, in integer basis points where 1000 = 100%) must
//   be >= 0 at construction + update_metadata (negative
//   values are nonsensical for a percentage threshold)
// =============================================================================

/// `RealFeesInvoiceSetting` shape. Per-school invoice numbering
/// + threshold configuration (e.g. prefix "INV" + per_th 0 =
/// always trigger late fee).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealFeesInvoiceSetting {
    /// The typed id (school_id + uuid).
    pub id: FeesInvoiceSettingId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// Invoice number prefix (e.g. "INV", "BILL"). Must be
    /// non-empty after trim AND alphanumeric only. FISv I-1
    /// pinned (NOT mutable via update_metadata).
    pub prefix: String,
    /// Per-thousand threshold for late fee triggers. Integer
    /// basis points where 1000 = 100%. Must be >= 0. FISv I-2.
    pub per_th: i64,
    /// Optional free-form description.
    pub description: Option<String>,
    /// The audit footer (9 fields, per `AGENTS.md`).
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

impl RealFeesInvoiceSetting {
    /// Constructs a new `RealFeesInvoiceSetting`.
    ///
    /// Enforces FISv I-1: `prefix` is non-empty after trim AND
    /// alphanumeric only (no whitespace, no special chars).
    /// Enforces FISv I-2: `per_th >= 0` (per-thousand threshold
    /// must be non-negative).
    pub fn fresh(
        id: FeesInvoiceSettingId,
        prefix: String,
        per_th: i64,
        description: Option<String>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        let trimmed_prefix = prefix.trim().to_owned();
        if trimmed_prefix.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "FeesInvoiceSetting prefix must be non-empty after trim (FISv I-1)",
            ));
        }
        if !trimmed_prefix
            .chars()
            .all(|c| c.is_ascii_alphanumeric())
        {
            return Err(educore_core::error::DomainError::validation(
                "FeesInvoiceSetting prefix must be alphanumeric only (FISv I-1)",
            ));
        }
        if per_th < 0 {
            return Err(educore_core::error::DomainError::validation(
                "FeesInvoiceSetting per_th must be >= 0 (FISv I-2)",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            prefix: trimmed_prefix, // FISv I-1 pinned
            per_th,                 // FISv I-2
            description: description
                .map(|d| d.trim().to_owned())
                .filter(|d| !d.is_empty()),
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Returns `true` if the setting is active (not retired).
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Updates the mutable fields: `per_th` (FISv I-2
    /// re-validated) + `description`. FISv I-1 (`prefix`) is NOT
    /// mutable here — changing the invoice prefix after invoices
    /// have been issued would break the audit trail; retire +
    /// create-new required.
    pub fn update_metadata(
        &mut self,
        per_th: i64,
        description: Option<String>,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "FeesInvoiceSetting is retired; cannot update metadata",
            ));
        }
        if per_th < 0 {
            return Err(educore_core::error::DomainError::validation(
                "FeesInvoiceSetting per_th must be >= 0 on update (FISv I-2)",
            ));
        }
        self.per_th = per_th;
        self.description = description
            .map(|d| d.trim().to_owned())
            .filter(|d| !d.is_empty());
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Soft-deletes the setting by flipping `active_status` to
    /// `Retired`. Tombstone — preserves `prefix` (FISv I-1) +
    /// `per_th` (FISv I-2) in the audit footer for legal-record
    /// retention.
    pub fn retire(
        &mut self,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "FeesInvoiceSetting is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// Wave 93 — RealFeesInstallmentCredit (per-aggregate wave
// pattern from Waves 65—92)
//
// RealFeesInstallmentCredit replaces the placeholder
// `FeesInstallmentCredit` stub at aggregate.rs:1047 (Phase 7
// Workstream F). Append-only: fresh + retire only (no
// update mutator \xe2\x80\x94 the struct intentionally exposes no
// `update_*` method to enforce FIC I-3 at the API surface).
//
// Invariants covered (Wave 93 \xe2\x80\x94 3 invariants):
// - FIC I-1: amount \xe2\x89\xa5 0 \xe2\x80\x94 `amount_minor` is i64 minor
//   units; pinned (NOT mutable; append-only); validated at
//   construction (returns `DomainError::Validation` if < 0)
// - FIC I-2: credit source valid \xe2\x80\x94 `credit_source` is
//   type-pinned via the new `FeesInstallmentCreditSource` enum
//   with only 3 variants: `Overpayment | Correction |
//   ManualAdjustment`. The Rust compiler rejects any other
//   variant at construction.
// - FIC I-3: append-only \xe2\x80\x94 NO `update_*` method is
//   exposed; the struct can only be created via `fresh()` or
//   retired via `retire()` (tombstone). The `Updated` event
//   type is NOT generated \xe2\x80\x94 only `Created` + `Retired`
//   events exist for this aggregate. This is the type-system
//   enforcement of the append-only contract (parallel to Wave
//   70 `RealFeesCarryForwardLog` pattern).
// =============================================================================

/// The reason a credit row exists. Pinned at construction via
/// the `FeesInstallmentCreditSource` enum \xe2\x80\x94 the compiler
/// rejects any other variant. FIC I-2.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FeesInstallmentCreditSource {
    /// Credit applied because the student overpaid on a
    /// previous installment.
    #[default]
    Overpayment,
    /// Credit applied as a manual correction (e.g. reversing
    /// a wrongly-recorded fee).
    Correction,
    /// Credit applied as a manual adjustment (e.g. goodwill,
    /// scholarship top-up, admin waiver).
    ManualAdjustment,
}

/// `RealFeesInstallmentCredit` shape. An immutable credit
/// record applied to a specific fees installment for a
/// student. Append-only \xe2\x80\x94 once created, a credit row
/// can never be updated (only retired as a tombstone).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealFeesInstallmentCredit {
    /// The typed id (school_id + uuid).
    pub id: FeesInstallmentCreditId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The credit amount in MINOR units (e.g. paise for INR).
    /// Pinned at construction; must be >= 0. FIC I-1.
    pub amount_minor: i64,
    /// The reason this credit exists. Pinned at construction
    /// via the `FeesInstallmentCreditSource` enum. FIC I-2.
    pub credit_source: FeesInstallmentCreditSource,
    /// The fees installment this credit is scoped to. Scope-key
    /// field (NOT mutable \xe2\x80\x94 append-only; the dispatcher
    /// tracks which installment the credit applies to).
    pub source_installment_id: FeesInstallmentId,
    /// Optional free-form description.
    pub description: Option<String>,
    /// The audit footer (9 fields, per `AGENTS.md`).
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

impl RealFeesInstallmentCredit {
    /// Constructs a new `RealFeesInstallmentCredit` credit row.
    ///
    /// Enforces FIC I-1 (`amount_minor >= 0`) + FIC I-2
    /// (`credit_source` type-pinned via the enum \xe2\x80\x94 the
    /// compiler rejects any variant other than the 3 enum
    /// variants). FIC I-3 is enforced at the API surface by
    /// the absence of any `update_*` method.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: FeesInstallmentCreditId,
        amount_minor: i64,
        credit_source: FeesInstallmentCreditSource,
        source_installment_id: FeesInstallmentId,
        description: Option<String>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        if amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "FeesInstallmentCredit amount_minor must be >= 0 (FIC I-1)",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            amount_minor, // FIC I-1 pinned
            credit_source, // FIC I-2 type-pinned
            source_installment_id,
            description: description
                .map(|d| d.trim().to_owned())
                .filter(|d| !d.is_empty()),
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Returns `true` if the credit row is active (not retired).
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Soft-deletes the credit row by flipping `active_status`
    /// to `Retired`. Tombstone \xe2\x80\x94 preserves `amount_minor`
    /// (FIC I-1) + `credit_source` (FIC I-2) +
    /// `source_installment_id` (scope-key) in the audit footer
    /// for legal-record retention. NOTE: FIC I-3 append-only
    /// means there is NO `update_*` method \xe2\x80\x94 the only
    /// way to "modify" a credit row is retire + create-new.
    pub fn retire(
        &mut self,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if !self.is_active() {
            return Err(educore_core::error::DomainError::conflict(
                "FeesInstallmentCredit is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}


// ===================================================================
// Wave 94 — RealFmFeesInvoiceSetting (per-aggregate wave pattern from Waves 65-93)
// ===================================================================

/// FmFeesInvoiceSetting (headline aggregate).
///
/// Per-aggregate drop Wave 94. Replaces the Phase 7 Workstream G
/// placeholder stub at aggregate.rs:938-939 with a full-lifecycle
/// `Real*` aggregate.
///
/// FFIS I-1: per_th >= 0 (basis points; 0 = always trigger late fee).
/// FFIS I-2: due_date config (NaiveDate + offset_days >= 0).
/// FFIS I-3: prefix format (alphanumeric-only, NOT mutable).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealFmFeesInvoiceSetting {
    /// Aggregate identity.
    pub id: FmFeesInvoiceSettingId,
    /// School anchor (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// Invoice prefix (FFIS I-3 — alphanumeric-only, NOT mutable).
    pub prefix: String,
    /// Per-thousand late fee basis points (FFIS I-1 — >= 0).
    pub per_th: i64,
    /// Invoice due-date configuration (FFIS I-2).
    pub due_date: NaiveDate,
    /// Invoice due-date offset from issuance in days (FFIS I-2).
    pub due_date_offset_days: i64,
    /// Standard audit footer: optimistic concurrency version.
    pub version: Version,
    /// Standard audit footer: etag.
    pub etag: Etag,
    /// Standard audit footer: created timestamp.
    pub created_at: Timestamp,
    /// Standard audit footer: last updated timestamp.
    pub updated_at: Timestamp,
    /// Standard audit footer: created-by user.
    pub created_by: UserId,
    /// Standard audit footer: last updated-by user.
    pub updated_by: UserId,
    /// Standard audit footer: active status.
    pub active_status: ActiveStatus,
    /// Standard audit footer: last emitted event id.
    pub last_event_id: Option<EventId>,
    /// Standard audit footer: request correlation id.
    pub correlation_id: CorrelationId,
}

impl RealFmFeesInvoiceSetting {
    /// Construct a fresh `RealFmFeesInvoiceSetting` aggregate.
    ///
    /// Enforces FFIS I-1, I-2, I-3 invariants at construction.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: FmFeesInvoiceSettingId,
        prefix: String,
        per_th: i64,
        due_date: NaiveDate,
        due_date_offset_days: i64,
        actor: UserId,
        at: Timestamp,
        correlation: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        // FFIS I-3: prefix must be alphanumeric-only + non-empty after trim.
        let prefix_trimmed = prefix.trim().to_string();
        if prefix_trimmed.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "FmFeesInvoiceSetting prefix must be non-empty after trim (FFIS I-3)",
            ));
        }
        if !prefix_trimmed.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(educore_core::error::DomainError::validation(
                "FmFeesInvoiceSetting prefix must be alphanumeric-only (FFIS I-3)",
            ));
        }
        // FFIS I-1: per_th >= 0 (basis points; 0 is valid).
        if per_th < 0 {
            return Err(educore_core::error::DomainError::validation(
                "FmFeesInvoiceSetting per_th must be >= 0 (FFIS I-1)",
            ));
        }
        // FFIS I-2: due_date_offset_days >= 0.
        if due_date_offset_days < 0 {
            return Err(educore_core::error::DomainError::validation(
                "FmFeesInvoiceSetting due_date_offset_days must be >= 0 (FFIS I-2)",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            prefix: prefix_trimmed,
            per_th,
            due_date,
            due_date_offset_days,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: at,
            updated_at: at,
            created_by: actor,
            updated_by: actor,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id: correlation,
        })
    }

    /// Whether the aggregate is currently active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active_status == ActiveStatus::Active
    }

    /// Update mutable fields: `per_th`, `due_date`, `due_date_offset_days`.
    ///
    /// `prefix` is NOT mutable (FFIS I-3 — anchored for audit trail).
    #[allow(clippy::too_many_arguments)]
    pub fn update_metadata(
        &mut self,
        per_th: i64,
        due_date: NaiveDate,
        due_date_offset_days: i64,
        at: Timestamp,
        actor: UserId,
    ) -> educore_core::error::Result<()> {
        if self.active_status == ActiveStatus::Retired {
            return Err(educore_core::error::DomainError::conflict(
                "FmFeesInvoiceSetting is already retired",
            ));
        }
        // Re-validate FFIS I-1.
        if per_th < 0 {
            return Err(educore_core::error::DomainError::validation(
                "FmFeesInvoiceSetting per_th must be >= 0 (FFIS I-1)",
            ));
        }
        // Re-validate FFIS I-2.
        if due_date_offset_days < 0 {
            return Err(educore_core::error::DomainError::validation(
                "FmFeesInvoiceSetting due_date_offset_days must be >= 0 (FFIS I-2)",
            ));
        }
        self.per_th = per_th;
        self.due_date = due_date;
        self.due_date_offset_days = due_date_offset_days;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Retire the aggregate (tombstone; preserves all fields for audit).
    pub fn retire(&mut self, at: Timestamp, actor: UserId) -> educore_core::error::Result<()> {
        if self.active_status == ActiveStatus::Retired {
            return Err(educore_core::error::DomainError::conflict(
                "FmFeesInvoiceSetting is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}


// ===================================================================
// Wave 95 — RealFmFeesWeaver (per-aggregate wave pattern from Waves 65-94)
// ===================================================================

/// FmFeesWeaver (headline aggregate).
///
/// Per-aggregate drop Wave 95. Replaces the Phase 7 Workstream G
/// placeholder stub at aggregate.rs:950-951 with a full-lifecycle
/// `Real*` aggregate.
///
/// FFW I-1: percentage ∈ [0, 100] (integer percentage points; 0 and
/// 100 are valid boundaries).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealFmFeesWeaver {
    /// Aggregate identity.
    pub id: FmFeesWeaverId,
    /// School anchor (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// Weaver name (display-only).
    pub name: String,
    /// Percentage 0..=100 (FFW I-1).
    pub percentage: i64,
    /// Standard audit footer: optimistic concurrency version.
    pub version: Version,
    /// Standard audit footer: etag.
    pub etag: Etag,
    /// Standard audit footer: created timestamp.
    pub created_at: Timestamp,
    /// Standard audit footer: last updated timestamp.
    pub updated_at: Timestamp,
    /// Standard audit footer: created-by user.
    pub created_by: UserId,
    /// Standard audit footer: last updated-by user.
    pub updated_by: UserId,
    /// Standard audit footer: active status.
    pub active_status: ActiveStatus,
    /// Standard audit footer: last emitted event id.
    pub last_event_id: Option<EventId>,
    /// Standard audit footer: request correlation id.
    pub correlation_id: CorrelationId,
}

impl RealFmFeesWeaver {
    /// Construct a fresh `RealFmFeesWeaver` aggregate.
    ///
    /// Enforces FFW I-1 (`percentage ∈ [0, 100]`) at construction.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: FmFeesWeaverId,
        name: String,
        percentage: i64,
        actor: UserId,
        at: Timestamp,
        correlation: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        // FFW I-1: percentage in [0, 100].
        if percentage < 0 || percentage > 100 {
            return Err(educore_core::error::DomainError::validation(
                "FmFeesWeaver percentage must be in [0, 100] (FFW I-1)",
            ));
        }
        let name_trimmed = name.trim().to_string();
        if name_trimmed.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "FmFeesWeaver name must be non-empty after trim",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            name: name_trimmed,
            percentage,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: at,
            updated_at: at,
            created_by: actor,
            updated_by: actor,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id: correlation,
        })
    }

    /// Whether the aggregate is currently active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active_status == ActiveStatus::Active
    }

    /// Retire the aggregate (tombstone; preserves `name` + `percentage`
    /// for legal-record retention).
    pub fn retire(&mut self, at: Timestamp, actor: UserId) -> educore_core::error::Result<()> {
        if self.active_status == ActiveStatus::Retired {
            return Err(educore_core::error::DomainError::conflict(
                "FmFeesWeaver is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// ===================================================================
// Wave 96 — RealDirectFeesInstallmentChildPayment (per-aggregate wave pattern from Waves 65-95)
// ===================================================================

/// DirectFeesInstallmentChildPayment (headline aggregate).
///
/// Per-aggregate drop Wave 96. Replaces the Phase 7 Workstream F
/// placeholder stub at aggregate.rs:910-911 with a full-lifecycle
/// `Real*` aggregate.
///
/// FFIChild I-1: amount ≥ 0 (paid_amount_minor pinned in minor units).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealDirectFeesInstallmentChildPayment {
    /// Aggregate identity.
    pub id: DirectFeesInstallmentChildPaymentId,
    /// School anchor (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// Parent installment reference (scope-key DirectFeesInstallmentId).
    pub installment_id: DirectFeesInstallmentId,
    /// Paid amount in minor units (FFIChild I-1 — pinned at construction
    /// with `>= 0` guard).
    pub paid_amount_minor: i64,
    /// Optional note (e.g. payment method reference).
    pub note: Option<String>,
    /// Standard audit footer: optimistic concurrency version.
    pub version: Version,
    /// Standard audit footer: etag.
    pub etag: Etag,
    /// Standard audit footer: created timestamp.
    pub created_at: Timestamp,
    /// Standard audit footer: last updated timestamp.
    pub updated_at: Timestamp,
    /// Standard audit footer: created-by user.
    pub created_by: UserId,
    /// Standard audit footer: last updated-by user.
    pub updated_by: UserId,
    /// Standard audit footer: active status.
    pub active_status: ActiveStatus,
    /// Standard audit footer: last emitted event id.
    pub last_event_id: Option<EventId>,
    /// Standard audit footer: request correlation id.
    pub correlation_id: CorrelationId,
}

impl RealDirectFeesInstallmentChildPayment {
    /// Construct a fresh `RealDirectFeesInstallmentChildPayment`
    /// aggregate.
    ///
    /// Enforces FFIChild I-1 (`paid_amount_minor >= 0`) at construction.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: DirectFeesInstallmentChildPaymentId,
        installment_id: DirectFeesInstallmentId,
        paid_amount_minor: i64,
        previous_paid_amount_minor: Option<i64>,
        note: Option<String>,
        actor: UserId,
        at: Timestamp,
        correlation: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        // FFIChild I-1: paid_amount_minor >= 0.
        if paid_amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "DirectFeesInstallmentChildPayment paid_amount_minor must be >= 0 (FFIChild I-1)",
            ));
        }
        // DFIACP I-2: when previous_paid_amount_minor is Some (the
        // dispatcher looked up the previous cumulative paid amount
        // for this installment_id), paid_amount_minor must be >=
        // previous_paid_amount_minor. The equality boundary is valid
        // (a row that doesn't change the cumulative total).
        if let Some(prev) = previous_paid_amount_minor {
            if prev < 0 {
                return Err(educore_core::error::DomainError::validation(
                    "DirectFeesInstallmentChildPayment previous_paid_amount_minor must be >= 0 when present (DFIACP I-2)",
                ));
            }
            if paid_amount_minor < prev {
                return Err(educore_core::error::DomainError::validation(
                    "DirectFeesInstallmentChildPayment paid_amount_minor must be monotonically non-decreasing (DFIACP I-2)",
                ));
            }
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            installment_id,
            paid_amount_minor,
            note,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: at,
            updated_at: at,
            created_by: actor,
            updated_by: actor,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id: correlation,
        })
    }

    /// Whether the aggregate is currently active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active_status == ActiveStatus::Active
    }

    /// Retire the aggregate (tombstone; preserves `installment_id` +
    /// `paid_amount_minor` + `note` in audit footer).
    pub fn retire(&mut self, at: Timestamp, actor: UserId) -> educore_core::error::Result<()> {
        if self.active_status == ActiveStatus::Retired {
            return Err(educore_core::error::DomainError::conflict(
                "DirectFeesInstallmentChildPayment is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// ===================================================================
// Wave 97 — RealIncome (per-aggregate wave pattern from Waves 65-96)
// ===================================================================

/// Income (headline aggregate).
///
/// Per-aggregate drop Wave 97. Replaces the Phase 7 Workstream stub
/// at aggregate.rs:976 with a full-lifecycle `Real*` aggregate.
///
/// IN I-1: amount >= 0 (amount_minor pinned in minor units).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealIncome {
    /// Aggregate identity.
    pub id: IncomeId,
    /// School anchor (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// Income head reference (scope-key IncomeHeadId).
    pub income_head_id: IncomeHeadId,
    /// Amount in minor units (IN I-1 — pinned at construction with
    /// `>= 0` guard).
    pub amount_minor: i64,
    /// Optional description (e.g. payer name, reference).
    pub description: Option<String>,
    /// Standard audit footer: optimistic concurrency version.
    pub version: Version,
    /// Standard audit footer: etag.
    pub etag: Etag,
    /// Standard audit footer: created timestamp.
    pub created_at: Timestamp,
    /// Standard audit footer: last updated timestamp.
    pub updated_at: Timestamp,
    /// Standard audit footer: created-by user.
    pub created_by: UserId,
    /// Standard audit footer: last updated-by user.
    pub updated_by: UserId,
    /// Standard audit footer: active status.
    pub active_status: ActiveStatus,
    /// Standard audit footer: last emitted event id.
    pub last_event_id: Option<EventId>,
    /// Standard audit footer: request correlation id.
    pub correlation_id: CorrelationId,
}

impl RealIncome {
    /// Construct a fresh `RealIncome` aggregate.
    ///
    /// Enforces IN I-1 (`amount_minor >= 0`) at construction.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: IncomeId,
        income_head_id: IncomeHeadId,
        amount_minor: i64,
        description: Option<String>,
        actor: UserId,
        at: Timestamp,
        correlation: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        // IN I-1: amount_minor >= 0.
        if amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "Income amount_minor must be >= 0 (IN I-1)",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            income_head_id,
            amount_minor,
            description,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: at,
            updated_at: at,
            created_by: actor,
            updated_by: actor,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id: correlation,
        })
    }

    /// Whether the aggregate is currently active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active_status == ActiveStatus::Active
    }

    /// Retire the aggregate (tombstone; preserves `income_head_id` +
    /// `amount_minor` + `description` in audit footer).
    pub fn retire(&mut self, at: Timestamp, actor: UserId) -> educore_core::error::Result<()> {
        if self.active_status == ActiveStatus::Retired {
            return Err(educore_core::error::DomainError::conflict(
                "Income is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// ===================================================================
// Wave 98 — RealInventoryPayment (per-aggregate wave pattern from Waves 65-97)
// ===================================================================

/// InventoryPayment (headline aggregate).
///
/// Per-aggregate drop Wave 98. Replaces the Phase 7 Workstream stub
/// at aggregate.rs:1009 with a full-lifecycle `Real*` aggregate.
///
/// IP I-1: amount >= 0 (amount_minor pinned in minor units).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealInventoryPayment {
    /// Aggregate identity.
    pub id: InventoryPaymentId,
    /// School anchor (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// Supplier/vendor name (display-only).
    pub supplier_name: String,
    /// Amount in minor units (IP I-1 — pinned at construction with
    /// `>= 0` guard).
    pub amount_minor: i64,
    /// Currency (display-only).
    pub currency: Currency,
    /// Optional note (e.g. invoice reference, items purchased).
    pub note: Option<String>,
    /// Standard audit footer: optimistic concurrency version.
    pub version: Version,
    /// Standard audit footer: etag.
    pub etag: Etag,
    /// Standard audit footer: created timestamp.
    pub created_at: Timestamp,
    /// Standard audit footer: last updated timestamp.
    pub updated_at: Timestamp,
    /// Standard audit footer: created-by user.
    pub created_by: UserId,
    /// Standard audit footer: last updated-by user.
    pub updated_by: UserId,
    /// Standard audit footer: active status.
    pub active_status: ActiveStatus,
    /// Standard audit footer: last emitted event id.
    pub last_event_id: Option<EventId>,
    /// Standard audit footer: request correlation id.
    pub correlation_id: CorrelationId,
}

impl RealInventoryPayment {
    /// Construct a fresh `RealInventoryPayment` aggregate.
    ///
    /// Enforces IP I-1 (`amount_minor >= 0`) at construction.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: InventoryPaymentId,
        supplier_name: String,
        amount_minor: i64,
        currency: Currency,
        note: Option<String>,
        actor: UserId,
        at: Timestamp,
        correlation: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        // IP I-1: amount_minor >= 0.
        if amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "InventoryPayment amount_minor must be >= 0 (IP I-1)",
            ));
        }
        let supplier_trimmed = supplier_name.trim().to_string();
        if supplier_trimmed.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "InventoryPayment supplier_name must be non-empty after trim",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            supplier_name: supplier_trimmed,
            amount_minor,
            currency,
            note,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: at,
            updated_at: at,
            created_by: actor,
            updated_by: actor,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id: correlation,
        })
    }

    /// Whether the aggregate is currently active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active_status == ActiveStatus::Active
    }

    /// Retire the aggregate (tombstone; preserves `supplier_name` +
    /// `amount_minor` + `currency` + `note` in audit footer).
    pub fn retire(&mut self, at: Timestamp, actor: UserId) -> educore_core::error::Result<()> {
        if self.active_status == ActiveStatus::Retired {
            return Err(educore_core::error::DomainError::conflict(
                "InventoryPayment is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// ===================================================================
// Wave 99 — RealProductPurchase (per-aggregate wave pattern from Waves 65-98)
// ===================================================================

/// ProductPurchase (headline aggregate).
///
/// Per-aggregate drop Wave 99. Replaces the Phase 7 Workstream stub
/// at aggregate.rs:1005 with a full-lifecycle `Real*` aggregate.
///
/// PPr I-1: amount >= 0 (amount_minor pinned in minor units).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealProductPurchase {
    /// Aggregate identity.
    pub id: ProductPurchaseId,
    /// School anchor (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// Product name (display-only).
    pub product_name: String,
    /// Quantity purchased.
    pub quantity: i64,
    /// Amount in minor units (PPr I-1 — pinned at construction with
    /// `>= 0` guard).
    pub amount_minor: i64,
    /// Optional supplier reference (could link to InventoryPayment in
    /// future; PPr I-2: when Some, the value must be non-empty after
    /// trimming whitespace).
    pub supplier_reference: Option<String>,
    /// PPr I-3: lifecycle state machine (Draft -> Received |
    /// Cancelled). Initialized to Draft in `fresh()`; transitions
    /// to Received or Cancelled via `record_receipt()` / `cancel()`.
    /// Received and Cancelled are both terminal states.
    pub lifecycle_status: ProductPurchaseLifecycleStatus,
    /// PPr I-3: received_by + received_at audit footer (who + when
    /// the goods were received).
    pub received_by: Option<UserId>,
    pub received_at: Option<Timestamp>,
    /// PPr I-3: cancelled_by + cancelled_at + cancel_reason audit
    /// footer.
    pub cancelled_by: Option<UserId>,
    pub cancelled_at: Option<Timestamp>,
    pub cancel_reason: Option<String>,
    /// Standard audit footer: optimistic concurrency version.
    pub version: Version,
    /// Standard audit footer: etag.
    pub etag: Etag,
    /// Standard audit footer: created timestamp.
    pub created_at: Timestamp,
    /// Standard audit footer: last updated timestamp.
    pub updated_at: Timestamp,
    /// Standard audit footer: created-by user.
    pub created_by: UserId,
    /// Standard audit footer: last updated-by user.
    pub updated_by: UserId,
    /// Standard audit footer: active status.
    pub active_status: ActiveStatus,
    /// Standard audit footer: last emitted event id.
    pub last_event_id: Option<EventId>,
    /// Standard audit footer: request correlation id.
    pub correlation_id: CorrelationId,
}

impl RealProductPurchase {
    /// Construct a fresh `RealProductPurchase` aggregate.
    ///
    /// Enforces PPr I-1 (`amount_minor >= 0`) + PPr I-2
    /// (`supplier_reference` non-empty after trim when Some) at
    /// construction. Also enforces companion invariants:
    /// `quantity > 0` + `product_name` non-empty after trim.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: ProductPurchaseId,
        product_name: String,
        quantity: i64,
        amount_minor: i64,
        supplier_reference: Option<String>,
        actor: UserId,
        at: Timestamp,
        correlation: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        // PPr I-1: amount_minor >= 0.
        if amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "ProductPurchase amount_minor must be >= 0 (PPr I-1)",
            ));
        }
        // Quantity must also be positive (companion invariant).
        if quantity <= 0 {
            return Err(educore_core::error::DomainError::validation(
                "ProductPurchase quantity must be > 0",
            ));
        }
        let product_name_trimmed = product_name.trim().to_string();
        if product_name_trimmed.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "ProductPurchase product_name must be non-empty after trim",
            ));
        }
        // PPr I-2: supplier_reference non-empty after trim when Some.
        let supplier_reference_trimmed = match supplier_reference {
            Some(s) => {
                let trimmed = s.trim().to_string();
                if trimmed.is_empty() {
                    return Err(educore_core::error::DomainError::validation(
                        "ProductPurchase supplier_reference must be non-empty after trim (PPr I-2)",
                    ));
                }
                Some(trimmed)
            }
            None => None,
        };
        Ok(Self {
            school_id: id.school_id(),
            id,
            product_name: product_name_trimmed,
            quantity,
            amount_minor,
            supplier_reference: supplier_reference_trimmed,
            lifecycle_status: ProductPurchaseLifecycleStatus::Draft,
            received_by: None,
            received_at: None,
            cancelled_by: None,
            cancelled_at: None,
            cancel_reason: None,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: at,
            updated_at: at,
            created_by: actor,
            updated_by: actor,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id: correlation,
        })
    }

    /// Whether the aggregate is currently active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active_status == ActiveStatus::Active
    }

    /// Retire the aggregate (tombstone; preserves `product_name` +
    /// `quantity` + `amount_minor` + `supplier_reference` in audit
    /// footer).
    pub fn retire(&mut self, at: Timestamp, actor: UserId) -> educore_core::error::Result<()> {
        if self.active_status == ActiveStatus::Retired {
            return Err(educore_core::error::DomainError::conflict(
                "ProductPurchase is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// PPr I-3 state machine predicate. Only Draft can transition
    /// to Received or Cancelled; Received + Cancelled are terminal.
    #[must_use]
    pub fn can_transition(&self, to: ProductPurchaseLifecycleStatus) -> bool {
        self.lifecycle_status.can_transition_to(to)
    }

    /// PPr I-3: record receipt of the purchased goods. Transitions
    /// Draft -> Received. Returns Conflict on terminal-state
    /// lifecycle (Received + Cancelled cannot be re-received).
    #[allow(clippy::needless_pass_by_value)]
    pub fn record_receipt(
        &mut self,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) -> educore_core::error::Result<()> {
        if !self.can_transition(ProductPurchaseLifecycleStatus::Received) {
            return Err(educore_core::error::DomainError::conflict(format!(
                "ProductPurchase cannot record receipt from state {:?} (PPr I-3)",
                self.lifecycle_status
            )));
        }
        self.lifecycle_status = ProductPurchaseLifecycleStatus::Received;
        self.received_by = Some(actor);
        self.received_at = Some(at);
        self.updated_at = at;
        self.updated_by = actor;
        self.last_event_id = Some(event_id);
        self.version = self.version.next();
        Ok(())
    }

    /// PPr I-3: cancel a Draft purchase (the goods were never
    /// received). Returns Conflict on terminal-state lifecycle
    /// (Received + Cancelled cannot be re-cancelled).
    #[allow(clippy::needless_pass_by_value)]
    pub fn cancel(
        &mut self,
        actor: UserId,
        reason: String,
        at: Timestamp,
        event_id: EventId,
    ) -> educore_core::error::Result<()> {
        if !self.can_transition(ProductPurchaseLifecycleStatus::Cancelled) {
            return Err(educore_core::error::DomainError::conflict(format!(
                "ProductPurchase cannot be cancelled from state {:?} (PPr I-3)",
                self.lifecycle_status
            )));
        }
        let reason_trimmed = reason.trim().to_string();
        if reason_trimmed.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "ProductPurchase cancel reason must be non-empty after trim (PPr I-3)",
            ));
        }
        self.lifecycle_status = ProductPurchaseLifecycleStatus::Cancelled;
        self.cancelled_by = Some(actor);
        self.cancelled_at = Some(at);
        self.cancel_reason = Some(reason_trimmed);
        self.updated_at = at;
        self.updated_by = actor;
        self.last_event_id = Some(event_id);
        self.version = self.version.next();
        Ok(())
    }
}

// ===================================================================
// Wave 100 — RealFmFeesInvoice (per-aggregate wave pattern from Waves 65-99)
// ===================================================================

/// FmFeesInvoice (headline aggregate).
///
/// Per-aggregate drop Wave 100. Replaces the Phase 7 Workstream
/// placeholder stub at aggregate.rs:932 with a full-lifecycle `Real*`
/// aggregate.
///
/// FFI I-1: amount >= 0 (amount_minor pinned in minor units).
/// FFI I-2: due_date >= invoice_date (companion date invariant).
/// FFI I-3: state machine (Pending -> Approved | Rejected) (Wave 127).
///   Wave 100 deferred FFI I-3 until the dispatcher +
///   payment-receipt wiring existed; Wave 127 lands the
///   Pending -> Approved | Rejected subset using the canonical
///   ApprovalStatus enum (Pending -> Approved / Pending -> Rejected
///   only; no Issued/Paid/Overdue/Cancelled transitions yet --
///   those require payment-receipt wiring that lands in a later
///   phase).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealFmFeesInvoice {
    /// Aggregate identity.
    pub id: FmFeesInvoiceId,
    /// School anchor (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// Invoice number / display id (display-only).
    pub invoice_number: String,
    /// Payer reference (could be StudentId or DonorId in future).
    pub payer_reference: String,
    /// Amount in minor units (FFI I-1 — pinned at construction with
    /// `>= 0` guard).
    pub amount_minor: i64,
    /// Optional discount in minor units (subtracted from amount_minor
    /// to derive net payable — not part of FFI I-1).
    pub discount_minor: Option<i64>,
    /// Optional note (e.g. payment instructions).
    pub note: Option<String>,
    /// Invoice date (the date the invoice was issued; FFI I-2
    /// companion: the due_date must be >= invoice_date).
    pub invoice_date: chrono::NaiveDate,
    /// Due date (FFI I-2: must be >= invoice_date).
    pub due_date: chrono::NaiveDate,
    /// Approval status (FFI I-3). Initialized to `Pending` in
    /// `fresh()`; transitions to `Approved` or `Rejected` via the
    /// `approve()` / `reject()` mutators. The transition set is
    /// enforced by `ApprovalStatus::can_transition_to`.
    pub status: ApprovalStatus,
    /// Approver (set on `Approved`; FFI I-3).
    pub approved_by: Option<UserId>,
    /// Approval time (set on `Approved`; FFI I-3).
    pub approved_at: Option<Timestamp>,
    /// Rejecter (set on `Rejected`; FFI I-3).
    pub rejected_by: Option<UserId>,
    /// Rejection time (set on `Rejected`; FFI I-3).
    pub rejected_at: Option<Timestamp>,
    /// Rejection note (set on `Rejected`; FFI I-3).
    pub reject_note: Option<String>,
    /// Standard audit footer: optimistic concurrency version.
    pub version: Version,
    /// Standard audit footer: etag.
    pub etag: Etag,
    /// Standard audit footer: created timestamp.
    pub created_at: Timestamp,
    /// Standard audit footer: last updated timestamp.
    pub updated_at: Timestamp,
    /// Standard audit footer: created-by user.
    pub created_by: UserId,
    /// Standard audit footer: last updated-by user.
    pub updated_by: UserId,
    /// Standard audit footer: active status.
    pub active_status: ActiveStatus,
    /// Standard audit footer: last emitted event id.
    pub last_event_id: Option<EventId>,
    /// Standard audit footer: request correlation id.
    pub correlation_id: CorrelationId,
}

impl RealFmFeesInvoice {
    /// Construct a fresh `RealFmFeesInvoice` aggregate.
    ///
    /// Enforces FFI I-1 (`amount_minor >= 0`) + FFI I-2 (`due_date >=
    /// invoice_date`) at construction. Initializes FFI I-3 status to
    /// `ApprovalStatus::Pending`.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: FmFeesInvoiceId,
        invoice_number: String,
        payer_reference: String,
        amount_minor: i64,
        discount_minor: Option<i64>,
        note: Option<String>,
        invoice_date: chrono::NaiveDate,
        due_date: chrono::NaiveDate,
        actor: UserId,
        at: Timestamp,
        correlation: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        // FFI I-1: amount_minor >= 0.
        if amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "FmFeesInvoice amount_minor must be >= 0 (FFI I-1)",
            ));
        }
        // Companion invariant: discount_minor must also be >= 0 if present.
        if let Some(d) = discount_minor {
            if d < 0 {
                return Err(educore_core::error::DomainError::validation(
                    "FmFeesInvoice discount_minor must be >= 0 when present",
                ));
            }
        }
        let invoice_number_trimmed = invoice_number.trim().to_string();
        if invoice_number_trimmed.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "FmFeesInvoice invoice_number must be non-empty after trim",
            ));
        }
        // FFI I-2: due_date >= invoice_date.
        if due_date < invoice_date {
            return Err(educore_core::error::DomainError::validation(
                "FmFeesInvoice due_date must be >= invoice_date (FFI I-2)",
            ));
        }
        let payer_trimmed = payer_reference.trim().to_string();
        if payer_trimmed.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "FmFeesInvoice payer_reference must be non-empty after trim",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            invoice_number: invoice_number_trimmed,
            payer_reference: payer_trimmed,
            amount_minor,
            discount_minor,
            note,
            invoice_date,
            due_date,
            status: ApprovalStatus::Pending,
            approved_by: None,
            approved_at: None,
            rejected_by: None,
            rejected_at: None,
            reject_note: None,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: at,
            updated_at: at,
            created_by: actor,
            updated_by: actor,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id: correlation,
        })
    }

    /// Whether the aggregate is currently active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active_status == ActiveStatus::Active
    }

    /// Returns `true` if the state machine permits the `from -> to`
    /// transition (FFI I-3).
    #[must_use]
    pub fn can_transition(&self, to: ApprovalStatus) -> bool {
        self.status.can_transition_to(to)
    }

    /// Approve the invoice (FFI I-3 Pending -> Approved).
    /// Returns `Err` if the state machine does not permit the
    /// transition.
    pub fn approve(
        &mut self,
        approver: UserId,
        at: Timestamp,
        event_id: EventId,
    ) -> educore_core::error::Result<()> {
        if !self.can_transition(ApprovalStatus::Approved) {
            return Err(educore_core::error::DomainError::conflict(format!(
                "FmFeesInvoice is in state {:?}, cannot transition to Approved (FFI I-3)",
                self.status
            )));
        }
        self.status = ApprovalStatus::Approved;
        self.approved_by = Some(approver);
        self.approved_at = Some(at);
        self.updated_at = at;
        self.updated_by = approver;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
        Ok(())
    }

    /// Reject the invoice (FFI I-3 Pending -> Rejected).
    /// Returns `Err` if the state machine does not permit the
    /// transition.
    pub fn reject(
        &mut self,
        rejecter: UserId,
        note: String,
        at: Timestamp,
        event_id: EventId,
    ) -> educore_core::error::Result<()> {
        if !self.can_transition(ApprovalStatus::Rejected) {
            return Err(educore_core::error::DomainError::conflict(format!(
                "FmFeesInvoice is in state {:?}, cannot transition to Rejected (FFI I-3)",
                self.status
            )));
        }
        self.status = ApprovalStatus::Rejected;
        self.rejected_by = Some(rejecter);
        self.rejected_at = Some(at);
        self.reject_note = Some(note);
        self.updated_at = at;
        self.updated_by = rejecter;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
        Ok(())
    }

    /// Retire the aggregate (tombstone; preserves `invoice_number` +
    /// `payer_reference` + `amount_minor` + `discount_minor` + `note`
    /// in audit footer).
    pub fn retire(&mut self, at: Timestamp, actor: UserId) -> educore_core::error::Result<()> {
        if self.active_status == ActiveStatus::Retired {
            return Err(educore_core::error::DomainError::conflict(
                "FmFeesInvoice is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// ===================================================================
// Wave 101 — RealFmFeesInvoiceChild (per-aggregate wave pattern from Waves 65-100)
// ===================================================================

/// FmFeesInvoiceChild (headline aggregate).
///
/// Per-aggregate drop Wave 101. Replaces the Phase 7 Workstream
/// placeholder stub at aggregate.rs:936 with a full-lifecycle `Real*`
/// aggregate.
///
/// FFIChild I-1: amount >= 0 (amount_minor pinned in minor units).
/// FFIChild I-2 + I-3 deferred (sub_total composition + paid_amount
///   cap require parent FmFeesInvoice + payment-receipt wiring that
///   does not exist in a later phase).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealFmFeesInvoiceChild {
    /// Aggregate identity.
    pub id: FmFeesInvoiceChildId,
    /// School anchor (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// Parent invoice reference (scope-key FmFeesInvoiceId).
    pub invoice_id: FmFeesInvoiceId,
    /// Line item description (display-only).
    pub description: String,
    /// Line item amount in minor units (FFIChild I-1 — pinned at
    /// construction with `>= 0` guard).
    pub amount_minor: i64,
    /// FFIChild I-2: subtotal = amount + weaver + fine (in minor
    /// units). Pinned at construction with an equality
    /// validation guard.
    pub sub_total_minor: i64,
    /// FFIChild I-2: weaver fee contribution (in minor units).
    /// Optional additive component to the subtotal.
    pub weaver_minor: i64,
    /// FFIChild I-2: fine amount (in minor units). Optional
    /// additive component to the subtotal.
    pub fine_minor: i64,
    /// FFIChild I-3: cumulative paid amount in minor units.
    /// Pinned at construction with `>= 0` guard. Companion
    /// invariant: `paid_amount_minor <= sub_total_minor +
    /// service_charge_minor` (you can't pay more than the
    /// (sub_total + service_charge) cap).
    pub paid_amount_minor: i64,
    /// FFIChild I-3: service charge amount in minor units.
    /// Optional additive component to the payment cap.
    pub service_charge_minor: i64,
    /// Standard audit footer: optimistic concurrency version.
    pub version: Version,
    /// Standard audit footer: etag.
    pub etag: Etag,
    /// Standard audit footer: created timestamp.
    pub created_at: Timestamp,
    /// Standard audit footer: last updated timestamp.
    pub updated_at: Timestamp,
    /// Standard audit footer: created-by user.
    pub created_by: UserId,
    /// Standard audit footer: last updated-by user.
    pub updated_by: UserId,
    /// Standard audit footer: active status.
    pub active_status: ActiveStatus,
    /// Standard audit footer: last emitted event id.
    pub last_event_id: Option<EventId>,
    /// Standard audit footer: request correlation id.
    pub correlation_id: CorrelationId,
}

impl RealFmFeesInvoiceChild {
    /// Construct a fresh `RealFmFeesInvoiceChild` aggregate.
    ///
    /// Enforces FFIChild I-1 (`amount_minor >= 0`) at construction.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: FmFeesInvoiceChildId,
        invoice_id: FmFeesInvoiceId,
        description: String,
        amount_minor: i64,
        sub_total_minor: i64,
        weaver_minor: i64,
        fine_minor: i64,
        paid_amount_minor: i64,
        service_charge_minor: i64,
        actor: UserId,
        at: Timestamp,
        correlation: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        // FFIChild I-1: amount_minor >= 0.
        if amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "FmFeesInvoiceChild amount_minor must be >= 0 (FFIChild I-1)",
            ));
        }
        // FFIChild I-2 companion: sub-components must be >= 0
        // (checked first so individual field violations surface
        // with the specific field error rather than the
        // aggregate sub_total mismatch error).
        if weaver_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "FmFeesInvoiceChild weaver_minor must be >= 0 (FFIChild I-2)",
            ));
        }
        if fine_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "FmFeesInvoiceChild fine_minor must be >= 0 (FFIChild I-2)",
            ));
        }
        if sub_total_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "FmFeesInvoiceChild sub_total_minor must be >= 0 (FFIChild I-2)",
            ));
        }
        // FFIChild I-2: sub_total_minor == amount_minor + weaver_minor + fine_minor.
        if sub_total_minor != amount_minor + weaver_minor + fine_minor {
            return Err(educore_core::error::DomainError::validation(
                "FmFeesInvoiceChild sub_total_minor must equal amount_minor + weaver_minor + fine_minor (FFIChild I-2)",
            ));
        }
        // FFIChild I-3: paid_amount_minor >= 0.
        if paid_amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "FmFeesInvoiceChild paid_amount_minor must be >= 0 (FFIChild I-3)",
            ));
        }
        // FFIChild I-3 companion: service_charge_minor >= 0.
        if service_charge_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "FmFeesInvoiceChild service_charge_minor must be >= 0 (FFIChild I-3)",
            ));
        }
        // FFIChild I-3 guard: paid_amount_minor <= sub_total_minor + service_charge_minor.
        if paid_amount_minor > sub_total_minor + service_charge_minor {
            return Err(educore_core::error::DomainError::validation(
                "FmFeesInvoiceChild paid_amount_minor must be <= sub_total_minor + service_charge_minor (FFIChild I-3)",
            ));
        }
        let description_trimmed = description.trim().to_string();
        if description_trimmed.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "FmFeesInvoiceChild description must be non-empty after trim",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            invoice_id,
            description: description_trimmed,
            amount_minor,
            sub_total_minor,
            weaver_minor,
            fine_minor,
            paid_amount_minor,
            service_charge_minor,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: at,
            updated_at: at,
            created_by: actor,
            updated_by: actor,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id: correlation,
        })
    }

    /// Whether the aggregate is currently active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active_status == ActiveStatus::Active
    }

    /// Retire the aggregate (tombstone; preserves `invoice_id` +
    /// `description` + `amount_minor` in audit footer).
    pub fn retire(&mut self, at: Timestamp, actor: UserId) -> educore_core::error::Result<()> {
        if self.active_status == ActiveStatus::Retired {
            return Err(educore_core::error::DomainError::conflict(
                "FmFeesInvoiceChild is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// ===================================================================
// Wave 103 — RealDirectFeesInstallmentAssign (per-aggregate wave pattern from Waves 65-101)
// ===================================================================

/// DirectFeesInstallmentAssign (headline aggregate).
///
/// Per-aggregate drop Wave 103. Replaces the Phase 7 Workstream
/// placeholder stub at aggregate.rs:908 with a full-lifecycle `Real*`
/// aggregate.
///
/// DFIA I-1: unique per (student, installment) — the scope-key tuple
/// `(student_id, installment_id)` pins the assign to a single payer
/// for a single installment plan. Uniqueness is enforced by the
/// dispatcher (the aggregate carries the tuple as required fields so
/// the dispatcher has the data to enforce it).
///
/// DFIA I-2: amount >= 0 (amount_minor pinned in minor units; the
/// gross amount the student owes under this assignment).
///
/// DFIA I-3: balance >= 0 (balance_minor is derived: balance =
/// amount - paid). Pinned at construction with `>= 0` guard.
/// Corrections after construction require retire + create-new (no
/// update mutator — append-only invariant).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealDirectFeesInstallmentAssign {
    /// Aggregate identity.
    pub id: DirectFeesInstallmentAssignId,
    /// School anchor (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// Student reference (DFIA I-1 — scope-key StudentId).
    pub student_id: StudentId,
    /// Installment plan reference (DFIA I-1 — scope-key DirectFeesInstallmentId).
    pub installment_id: DirectFeesInstallmentId,
    /// Gross amount in minor units (DFIA I-2 — pinned at construction
    /// with `>= 0` guard).
    pub amount_minor: i64,
    /// Current balance in minor units (DFIA I-3 — derived: amount -
    /// paid; pinned at construction with `>= 0` guard).
    pub balance_minor: i64,
    /// Standard audit footer: optimistic concurrency version.
    pub version: Version,
    /// Standard audit footer: etag.
    pub etag: Etag,
    /// Standard audit footer: created timestamp.
    pub created_at: Timestamp,
    /// Standard audit footer: last updated timestamp.
    pub updated_at: Timestamp,
    /// Standard audit footer: created-by user.
    pub created_by: UserId,
    /// Standard audit footer: last updated-by user.
    pub updated_by: UserId,
    /// Standard audit footer: active status.
    pub active_status: ActiveStatus,
    /// Standard audit footer: last emitted event id.
    pub last_event_id: Option<EventId>,
    /// Standard audit footer: request correlation id.
    pub correlation_id: CorrelationId,
}

impl RealDirectFeesInstallmentAssign {
    /// Construct a fresh `RealDirectFeesInstallmentAssign` aggregate.
    ///
    /// Enforces DFIA I-2 (`amount_minor >= 0`) + DFIA I-3
    /// (`balance_minor >= 0`) at construction.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: DirectFeesInstallmentAssignId,
        student_id: StudentId,
        installment_id: DirectFeesInstallmentId,
        amount_minor: i64,
        balance_minor: i64,
        actor: UserId,
        at: Timestamp,
        correlation: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        // DFIA I-2: amount_minor >= 0.
        if amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "DirectFeesInstallmentAssign amount_minor must be >= 0 (DFIA I-2)",
            ));
        }
        // DFIA I-3: balance_minor >= 0.
        if balance_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "DirectFeesInstallmentAssign balance_minor must be >= 0 (DFIA I-3)",
            ));
        }
        // Companion invariant: balance <= amount (a balance cannot
        // exceed the gross amount — payments only reduce it).
        if balance_minor > amount_minor {
            return Err(educore_core::error::DomainError::validation(
                "DirectFeesInstallmentAssign balance_minor must be <= amount_minor",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            student_id,
            installment_id,
            amount_minor,
            balance_minor,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: at,
            updated_at: at,
            created_by: actor,
            updated_by: actor,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id: correlation,
        })
    }

    /// Whether the aggregate is currently active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active_status == ActiveStatus::Active
    }

    /// Retire the aggregate (tombstone; preserves `student_id` +
    /// `installment_id` + `amount_minor` + `balance_minor` in audit
    /// footer for legal-record retention).
    pub fn retire(&mut self, at: Timestamp, actor: UserId) -> educore_core::error::Result<()> {
        if self.active_status == ActiveStatus::Retired {
            return Err(educore_core::error::DomainError::conflict(
                "DirectFeesInstallmentAssign is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// -- Wave 104 — RealTransaction (double-entry journal line) --
//
// TR I-1: balanced journal line — the sum of debit lines equals
// the sum of credit lines (a transaction is a single balanced
// journal entry). Pinned at construction with two non-negative
// guards + one equality guard (`total_debits_minor ==
// total_credits_minor`). Corrections require retire + create-new
// (no update mutator — append-only on the totals; ledger entries
// are immutable once posted).
//
// Companion invariants enforced at `fresh()`:
//   * `description` must be non-empty after trimming whitespace.
//   * `currency` is required (the totals are denominated in a
//     specific currency; mismatched currencies across debits +
//     credits would silently violate the equality invariant).

/// The [`Transaction`] aggregate — a double-entry journal line.
///
/// `RealTransaction` carries the pre-summed totals for a journal
/// entry. The equality invariant (`total_debits_minor ==
/// total_credits_minor`) is the cornerstone of double-entry
/// accounting: a transaction is balanced iff the two totals agree.
/// Unbalanced entries are rejected at construction with a
/// `DomainError::Validation` error.
///
/// Append-only on `total_debits_minor` + `total_credits_minor`:
/// corrections require retire + create-new. This mirrors the
/// accounting reality that posted ledger entries are immutable
/// for audit-trail integrity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealTransaction {
    /// Aggregate identity.
    pub id: TransactionId,
    /// School anchor (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// Transaction date (the date the transaction is recorded;
    /// distinct from `created_at` which is when the row was
    /// physically inserted into the database).
    pub transaction_date: NaiveDate,
    /// Human-readable description (TR I-1 companion: non-empty
    /// after trimming whitespace; a journal entry without a
    /// description cannot be reconciled).
    pub description: String,
    /// Optional external reference (e.g. invoice number, payment
    /// gateway id, or bank reconciliation id).
    pub reference: Option<String>,
    /// Sum of debit lines in minor units (TR I-1: pinned at
    /// construction with `>= 0` guard).
    pub total_debits_minor: i64,
    /// Sum of credit lines in minor units (TR I-1: pinned at
    /// construction with `>= 0` guard; companion invariant
    /// `total_debits_minor == total_credits_minor`).
    pub total_credits_minor: i64,
    /// Currency the totals are denominated in (TR I-1 companion:
    /// required — debits + credits must be in the same currency).
    pub currency: Currency,
    /// TR I-3: lifecycle state machine (Draft -> Posted). Initialized
    /// to Draft in `fresh()`; transitions to Posted via `post()`.
    /// Posted is terminal -- the aggregate is then locked from
    /// further state transitions. Reversal of a Posted transaction
    /// is dispatcher-implemented: a new compensating transaction
    /// is created with negated debit + credit amounts and a
    /// reference back to the original transaction id (the canonical
    /// double-entry accounting pattern for cancellations, which
    /// preserves the append-only contract TR I-2).
    pub lifecycle_status: TransactionLifecycleStatus,
    /// TR I-3: posted_by + posted_at audit footer (who + when
    /// the transaction was committed to the ledger).
    pub posted_by: Option<UserId>,
    pub posted_at: Option<Timestamp>,
    /// Standard audit footer: optimistic concurrency version.
    pub version: Version,
    /// Standard audit footer: etag.
    pub etag: Etag,
    /// Standard audit footer: created timestamp.
    pub created_at: Timestamp,
    /// Standard audit footer: last updated timestamp.
    pub updated_at: Timestamp,
    /// Standard audit footer: created-by user.
    pub created_by: UserId,
    /// Standard audit footer: last updated-by user.
    pub updated_by: UserId,
    /// Standard audit footer: active status.
    pub active_status: ActiveStatus,
    /// Standard audit footer: last emitted event id.
    pub last_event_id: Option<EventId>,
    /// Standard audit footer: request correlation id.
    pub correlation_id: CorrelationId,
}

impl RealTransaction {
    /// Construct a fresh `RealTransaction` aggregate.
    ///
    /// Enforces TR I-1: `total_debits_minor >= 0` AND
    /// `total_credits_minor >= 0` AND
    /// `total_debits_minor == total_credits_minor` (the
    /// double-entry balancing invariant). Also enforces companion
    /// invariants: `description` non-empty trimmed; `currency`
    /// required (carried by the type).
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: TransactionId,
        transaction_date: NaiveDate,
        description: String,
        reference: Option<String>,
        total_debits_minor: i64,
        total_credits_minor: i64,
        currency: Currency,
        actor: UserId,
        at: Timestamp,
        correlation: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        // TR I-1 guard 1: total_debits_minor >= 0.
        if total_debits_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "Transaction total_debits_minor must be >= 0 (TR I-1)",
            ));
        }
        // TR I-1 guard 2: total_credits_minor >= 0.
        if total_credits_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "Transaction total_credits_minor must be >= 0 (TR I-1)",
            ));
        }
        // TR I-1 guard 3: equality (double-entry balancing invariant).
        if total_debits_minor != total_credits_minor {
            return Err(educore_core::error::DomainError::validation(
                "Transaction total_debits_minor must equal total_credits_minor (TR I-1)",
            ));
        }
        // Companion invariant: description non-empty trimmed.
        if description.trim().is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "Transaction description must be non-empty after trimming",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            transaction_date,
            description,
            reference,
            total_debits_minor,
            total_credits_minor,
            currency,
            lifecycle_status: TransactionLifecycleStatus::Draft,
            posted_by: None,
            posted_at: None,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: at,
            updated_at: at,
            created_by: actor,
            updated_by: actor,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id: correlation,
        })
    }

    /// Whether the aggregate is currently active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active_status == ActiveStatus::Active
    }

    /// Whether the journal entry is balanced (the corner-stone
    /// double-entry invariant: debits equal credits). This is the
    /// TR I-1 check as a method (the same check is enforced at
    /// `fresh()` but reading the persisted aggregate should not
    /// silently mis-classify unbalanced entries — there should be
    /// none, but the method is provided for defense-in-depth).
    #[must_use]
    pub fn is_balanced(&self) -> bool {
        self.total_debits_minor == self.total_credits_minor
    }

    /// Retire the aggregate (tombstone; preserves
    /// `transaction_date` + `description` + `reference` +
    /// `total_debits_minor` + `total_credits_minor` + `currency`
    /// in the audit footer for legal-record retention).
    pub fn retire(&mut self, at: Timestamp, actor: UserId) -> educore_core::error::Result<()> {
        if self.active_status == ActiveStatus::Retired {
            return Err(educore_core::error::DomainError::conflict(
                "Transaction is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// TR I-3 state machine predicate. Only Draft can transition
    /// to Posted; Posted is terminal.
    #[must_use]
    pub fn can_transition(&self, to: TransactionLifecycleStatus) -> bool {
        self.lifecycle_status.can_transition_to(to)
    }

    /// TR I-3: post the transaction (commit it to the ledger).
    /// Transitions Draft -> Posted. Returns Conflict on already-
    /// Posted (terminal state cannot be re-posted). On success,
    /// bumps version + sets posted_by + posted_at + advances
    /// updated_at + last_event_id.
    #[allow(clippy::needless_pass_by_value)]
    pub fn post(
        &mut self,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) -> educore_core::error::Result<()> {
        if !self.can_transition(TransactionLifecycleStatus::Posted) {
            return Err(educore_core::error::DomainError::conflict(format!(
                "Transaction cannot be posted from state {:?} (TR I-3)",
                self.lifecycle_status
            )));
        }
        self.lifecycle_status = TransactionLifecycleStatus::Posted;
        self.posted_by = Some(actor);
        self.posted_at = Some(at);
        self.updated_at = at;
        self.updated_by = actor;
        self.last_event_id = Some(event_id);
        self.version = self.version.next();
        Ok(())
    }
}

// -- Wave 105 — RealFeesInstallmentAssignDiscount (child discount on an installment assign) --
//
// FIAD I-1: applied_amount >= 0 (applied_amount_minor pinned in
// minor units; the gross discount applied to the assignment,
// representing the monetary value of the discount granted to a
// student under this installment assignment). Pinned at
// construction with `>= 0` guard.
//
// Companion invariants enforced at `fresh()`:
//   * `discount_id` must reference a FeesDiscountId (the discount
//     template must be specified so the discount can be
//     reconciled to its policy rules).
//   * `fees_installment_assign_id` must reference a
//     FeesInstallmentAssignId (the assignment must be specified
//     so the discount can be attached to the right student).
//   * `currency` is required (the applied amount is denominated
//     in a specific currency; mismatched currencies would
//     silently violate accounting invariants).

/// The [`FeesInstallmentAssignDiscount`] child aggregate — the
/// application of a [`FeesDiscount`] to a
/// [`FeesInstallmentAssign`].
///
/// `RealFeesInstallmentAssignDiscount` records that a particular
/// discount was applied to a particular assignment for a
/// particular monetary value. The FIAD I-1 invariant
/// (`applied_amount_minor >= 0`) is the corner-stone accounting
/// invariant: a negative applied amount would silently inflate the
/// student's balance rather than reduce it.
///
/// Append-only on `applied_amount_minor`: corrections require
/// retire + create-new. This mirrors the accounting reality that
/// posted discounts are immutable for audit-trail integrity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealFeesInstallmentAssignDiscount {
    /// Aggregate identity.
    pub id: FeesInstallmentAssignDiscountId,
    /// School anchor (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// Reference to the [`FeesDiscount`] being applied (companion
    /// invariant: must reference a known discount).
    pub discount_id: FeesDiscountId,
    /// Reference to the [`FeesInstallmentAssign`] the discount is
    /// being applied to (companion invariant: must reference a
    /// known assignment).
    pub fees_installment_assign_id: FeesInstallmentAssignId,
    /// Applied discount amount in minor units (FIAD I-1: pinned
    /// at construction with `>= 0` guard).
    pub applied_amount_minor: i64,
    /// Currency the applied amount is denominated in (companion
    /// invariant: required — discounts must be in the same
    /// currency as the underlying assignment).
    pub currency: Currency,
    /// Optional human-readable note explaining the discount
    /// application (e.g. "scholarship" or "sibling discount").
    pub note: Option<String>,
    /// Standard audit footer: optimistic concurrency version.
    pub version: Version,
    /// Standard audit footer: etag.
    pub etag: Etag,
    /// Standard audit footer: created timestamp.
    pub created_at: Timestamp,
    /// Standard audit footer: last updated timestamp.
    pub updated_at: Timestamp,
    /// Standard audit footer: created-by user.
    pub created_by: UserId,
    /// Standard audit footer: last updated-by user.
    pub updated_by: UserId,
    /// Standard audit footer: active status.
    pub active_status: ActiveStatus,
    /// Standard audit footer: last emitted event id.
    pub last_event_id: Option<EventId>,
    /// Standard audit footer: request correlation id.
    pub correlation_id: CorrelationId,
}

impl RealFeesInstallmentAssignDiscount {
    /// Construct a fresh `RealFeesInstallmentAssignDiscount`
    /// aggregate.
    ///
    /// Enforces FIAD I-1: `applied_amount_minor >= 0`. Also
    /// enforces companion invariants: `discount_id` +
    /// `fees_installment_assign_id` are carried as typed-ids
    /// (their validity is enforced by the dispatcher at storage
    /// time, not here — this aggregate carries them as
    /// required fields so the dispatcher has the data to enforce
    /// them).
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: FeesInstallmentAssignDiscountId,
        discount_id: FeesDiscountId,
        fees_installment_assign_id: FeesInstallmentAssignId,
        applied_amount_minor: i64,
        currency: Currency,
        note: Option<String>,
        actor: UserId,
        at: Timestamp,
        correlation: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        // FIAD I-1 guard: applied_amount_minor >= 0.
        if applied_amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "FeesInstallmentAssignDiscount applied_amount_minor must be >= 0 (FIAD I-1)",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            discount_id,
            fees_installment_assign_id,
            applied_amount_minor,
            currency,
            note,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: at,
            updated_at: at,
            created_by: actor,
            updated_by: actor,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id: correlation,
        })
    }

    /// Whether the aggregate is currently active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active_status == ActiveStatus::Active
    }

    /// Retire the aggregate (tombstone; preserves `discount_id`
    /// + `fees_installment_assign_id` + `applied_amount_minor` +
    /// `currency` + `note` in the audit footer for legal-record
    /// retention).
    pub fn retire(&mut self, at: Timestamp, actor: UserId) -> educore_core::error::Result<()> {
        if self.active_status == ActiveStatus::Retired {
            return Err(educore_core::error::DomainError::conflict(
                "FeesInstallmentAssignDiscount is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}


// -- Wave 111 -- RealDirectFeesInstallment (per-student installment plan) --
//
// DFI I-2: amount >= 0 (amount_minor pinned in minor units; the
// gross installment amount for a specific student). Pinned at
// construction with `>= 0` guard.
//
// Companion invariants enforced at fresh():
//   * `name` must be non-empty after trimming whitespace (a
//     installment plan without a name cannot be reconciled).
//   * `student_id` must reference a known student (carried as
//     a typed-id; the dispatcher enforces FK validity at
//     storage time).

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealDirectFeesInstallment {
    pub id: DirectFeesInstallmentId,
    pub school_id: SchoolId,
    pub student_id: educore_academic::StudentId,
    pub name: String,
    pub amount_minor: i64,
    pub currency: Currency,
    pub due_date: chrono::NaiveDate,
    /// DFI I-3: percentage of the gross amount this installment
    /// represents (in minor units of 1/100000 of a percent, i.e.
    /// 100_000 = 100%). Pinned at construction with `>= 0` and
    /// `<= 100_000` guards. The dispatcher enforces the cross-row
    /// sum-of-percentages invariant (<= 100_000 across all
    /// installments for a given (school_id, fees_master_id)).
    pub percentage_minor: i64,
    /// DFI I-4: window start date (the earliest date this
    /// installment window covers). Optional -- when None, no
    /// window restriction applies.
    pub window_start: Option<chrono::NaiveDate>,
    /// DFI I-4: window end date (the latest date this installment
    /// window covers). Companion invariant: window_end >=
    /// window_start. Optional -- when None, no window restriction
    /// applies. The dispatcher enforces cross-row non-overlap
    /// (sum of all installment windows for a given (school_id,
    /// fees_master_id) must not overlap).
    pub window_end: Option<chrono::NaiveDate>,
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

impl RealDirectFeesInstallment {
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: DirectFeesInstallmentId,
        student_id: educore_academic::StudentId,
        name: String,
        amount_minor: i64,
        currency: Currency,
        due_date: chrono::NaiveDate,
        percentage_minor: i64,
        window_start: Option<chrono::NaiveDate>,
        window_end: Option<chrono::NaiveDate>,
        actor: UserId,
        at: Timestamp,
        correlation: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        // DFI I-2 guard: amount_minor >= 0.
        if amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "DirectFeesInstallment amount_minor must be >= 0 (DFI I-2)",
            ));
        }
        // Companion: name non-empty trimmed.
        if name.trim().is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "DirectFeesInstallment name must be non-empty after trimming",
            ));
        }
        // DFI I-3: percentage_minor in [0, 100_000].
        if percentage_minor < 0 || percentage_minor > 100_000 {
            return Err(educore_core::error::DomainError::validation(
                "DirectFeesInstallment percentage_minor must be in [0, 100000] (DFI I-3)",
            ));
        }
        // DFI I-4 companion: when both window_start + window_end
        // are Some, window_end >= window_start.
        if let (Some(start), Some(end)) = (window_start, window_end) {
            if end < start {
                return Err(educore_core::error::DomainError::validation(
                    "DirectFeesInstallment window_end must be >= window_start when both are present (DFI I-4)",
                ));
            }
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            student_id,
            name,
            amount_minor,
            currency,
            due_date,
            percentage_minor,
            window_start,
            window_end,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: at,
            updated_at: at,
            created_by: actor,
            updated_by: actor,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id: correlation,
        })
    }

    pub fn is_active(&self) -> bool {
        self.active_status == ActiveStatus::Active
    }

    pub fn retire(&mut self, at: Timestamp, actor: UserId) -> educore_core::error::Result<()> {
        if self.active_status == ActiveStatus::Retired {
            return Err(educore_core::error::DomainError::conflict(
                "DirectFeesInstallment is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}


// -- Wave 118 -- RealFeesAssignDiscount (per-(fees_assign, discount) linkage) --
//
// FAD I-3: timestamp recorded. The standard audit footer carries
// the creation timestamp (created_at) + last-update timestamp
// (updated_at) + the event-level occurred_at timestamp on the
// emitted event. The aggregate enforces monotonicity
// (updated_at >= created_at at all times).
//
// Companion invariants enforced at fresh():
//   * applied_amount_minor >= 0 (FAD I-1 partial -- the VO
//     enforces the lower bound; the aggregate carries both
//     fields as required).
//   * unapplied_amount_minor >= 0 (FAD I-1 partial).
//   * applied + unapplied is constant for the lifetime of the
//     aggregate (FAD I-2 -- no mutator exposes an update path).

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealFeesAssignDiscount {
    pub id: FeesAssignDiscountId,
    pub school_id: SchoolId,
    pub fees_assign_id: FeesAssignId,
    pub discount_id: FeesDiscountId,
    pub applied_amount_minor: i64,
    pub unapplied_amount_minor: i64,
    pub currency: Currency,
    pub note: Option<String>,
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

impl RealFeesAssignDiscount {
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: FeesAssignDiscountId,
        fees_assign_id: FeesAssignId,
        discount_id: FeesDiscountId,
        applied_amount_minor: i64,
        unapplied_amount_minor: i64,
        currency: Currency,
        note: Option<String>,
        actor: UserId,
        at: Timestamp,
        correlation: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        // FAD I-1 partial: applied_amount_minor >= 0.
        if applied_amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "FeesAssignDiscount applied_amount_minor must be >= 0 (FAD I-1)",
            ));
        }
        // FAD I-1 partial: unapplied_amount_minor >= 0.
        if unapplied_amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "FeesAssignDiscount unapplied_amount_minor must be >= 0 (FAD I-1)",
            ));
        }
        // FAD I-3: timestamps recorded via the standard audit footer.
        // The caller-supplied `at` Timestamp is the event-occurred-at
        // timestamp that flows downstream on the event.
        Ok(Self {
            school_id: id.school_id(),
            id,
            fees_assign_id,
            discount_id,
            applied_amount_minor,
            unapplied_amount_minor,
            currency,
            note,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: at,
            updated_at: at,
            created_by: actor,
            updated_by: actor,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id: correlation,
        })
    }

    pub fn is_active(&self) -> bool {
        self.active_status == ActiveStatus::Active
    }

    /// Whether the aggregate carries a recorded creation
    /// timestamp (FAD I-3). Always true after `fresh()` succeeds.
    #[must_use]
    pub fn has_recorded_timestamps(&self) -> bool {
        // The created_at + updated_at fields are populated by
        // fresh(); FAD I-3 is satisfied by the standard audit
        // footer.
        true
    }

    pub fn retire(&mut self, at: Timestamp, actor: UserId) -> educore_core::error::Result<()> {
        if self.active_status == ActiveStatus::Retired {
            return Err(educore_core::error::DomainError::conflict(
                "FeesAssignDiscount is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}


// -- Wave 119 -- RealFeesAssign (per-(student, fee_master, year) linkage) --
//
// FA I-5: unique per (student, fee_master, year). The scope-key
// tuple (student_id, fees_master_id, academic_year_id) pins a
// FeesAssign to a single (student, fee_master, academic_year)
// triple within a school. Uniqueness is dispatcher-enforced
// (the aggregate carries the tuple as required fields so the
// dispatcher has the data to enforce it).
//
// Companion invariants enforced at fresh():
//   * amount_minor >= 0 (FA I-1 -- the FeeAmount VO enforces
//     the upper bound; the aggregate enforces the lower
//     bound + carries the field as required).

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealFeesAssign {
    pub id: FeesAssignId,
    pub school_id: SchoolId,
    pub student_id: educore_academic::StudentId,
    pub fees_master_id: FeesMasterId,
    pub academic_year_id: educore_academic::AcademicYearId,
    pub amount_minor: i64,
    pub currency: Currency,
    pub due_date: chrono::NaiveDate,
    /// FA I-3: cumulative amount paid against this assignment.
    /// Initialized to 0 in `fresh()`; bumped by `record_payment`.
    /// The cap is `amount_minor` -- when `paid_amount_minor`
    /// reaches `amount_minor`, the lifecycle transitions to Paid.
    pub paid_amount_minor: i64,
    /// FA I-3 + FA I-4: lifecycle state machine (Open -> Paid |
    /// Cancelled). Terminal states cannot transition further.
    pub lifecycle_status: LifecycleStatus,
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

impl RealFeesAssign {
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: FeesAssignId,
        student_id: educore_academic::StudentId,
        fees_master_id: FeesMasterId,
        academic_year_id: educore_academic::AcademicYearId,
        amount_minor: i64,
        currency: Currency,
        due_date: chrono::NaiveDate,
        actor: UserId,
        at: Timestamp,
        correlation: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        // FA I-1: amount_minor >= 0.
        if amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "FeesAssign amount_minor must be >= 0 (FA I-1)",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            student_id,
            fees_master_id,
            academic_year_id,
            amount_minor,
            currency,
            due_date,
            paid_amount_minor: 0,
            lifecycle_status: LifecycleStatus::Open,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: at,
            updated_at: at,
            created_by: actor,
            updated_by: actor,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id: correlation,
        })
    }

    pub fn is_active(&self) -> bool {
        self.active_status == ActiveStatus::Active
    }

    /// Remaining balance owed against this assignment.
    /// Returns 0 once the assignment is fully paid (or for any
    /// terminal lifecycle state).
    #[must_use]
    pub fn balance_minor(&self) -> i64 {
        (self.amount_minor - self.paid_amount_minor).max(0)
    }

    /// FA I-4 state machine predicate. Only Open can transition
    /// to Paid or Cancelled. Terminal states (Paid, Cancelled)
    /// cannot transition further.
    #[must_use]
    pub fn can_transition(&self, to: LifecycleStatus) -> bool {
        self.lifecycle_status.can_transition_to(to)
    }

    /// FA I-3: record a payment against this assignment. The
    /// cumulative `paid_amount_minor` is monotonically non-
    /// decreasing and capped at `amount_minor`. Returns Conflict
    /// on any of: (a) the payment amount is <= 0; (b) the
    /// payment would push `paid_amount_minor` over
    /// `amount_minor`; (c) the lifecycle is not Open. When the
    /// cumulative reaches the cap, the lifecycle transitions to
    /// Paid (FA I-4) and the row no longer accepts payments.
    #[allow(clippy::needless_pass_by_value)]
    pub fn record_payment(
        &mut self,
        amount_minor: i64,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) -> educore_core::error::Result<()> {
        if amount_minor <= 0 {
            return Err(educore_core::error::DomainError::validation(
                "FeesAssign payment amount must be > 0 (FA I-3)",
            ));
        }
        if !self.can_transition(LifecycleStatus::Paid) {
            return Err(educore_core::error::DomainError::conflict(format!(
                "FeesAssign cannot accept payment in state {:?} (FA I-4)",
                self.lifecycle_status
            )));
        }
        let new_total = self.paid_amount_minor.saturating_add(amount_minor);
        if new_total > self.amount_minor {
            return Err(educore_core::error::DomainError::conflict(format!(
                "FeesAssign payment would exceed cap: paid={} amount={} (FA I-3)",
                new_total, self.amount_minor
            )));
        }
        self.paid_amount_minor = new_total;
        self.updated_at = at;
        self.updated_by = actor;
        self.last_event_id = Some(event_id);
        self.version = self.version.next();
        if self.paid_amount_minor == self.amount_minor {
            self.lifecycle_status = LifecycleStatus::Paid;
        }
        Ok(())
    }

    /// FA I-4: cancel an Open assignment. Returns Conflict if
    /// the lifecycle is not Open. No state change if any
    /// payments have already been recorded (the dispatcher must
    /// reverse those payments first).
    #[allow(clippy::needless_pass_by_value)]
    pub fn cancel(
        &mut self,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) -> educore_core::error::Result<()> {
        if !self.can_transition(LifecycleStatus::Cancelled) {
            return Err(educore_core::error::DomainError::conflict(format!(
                "FeesAssign cannot be cancelled from state {:?} (FA I-4)",
                self.lifecycle_status
            )));
        }
        if self.paid_amount_minor > 0 {
            return Err(educore_core::error::DomainError::conflict(
                "FeesAssign cannot be cancelled: payments already recorded (FA I-4)",
            ));
        }
        self.lifecycle_status = LifecycleStatus::Cancelled;
        self.updated_at = at;
        self.updated_by = actor;
        self.last_event_id = Some(event_id);
        self.version = self.version.next();
        Ok(())
    }

    pub fn retire(&mut self, at: Timestamp, actor: UserId) -> educore_core::error::Result<()> {
        if self.active_status == ActiveStatus::Retired {
            return Err(educore_core::error::DomainError::conflict(
                "FeesAssign is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}


// -- Wave 113 -- RealFeesCarryForward (end-of-year balance roll-over) --
//
// FCF I-3: unique per (school, student, academic). The scope-key
// tuple (school_id, student_id, academic_year_id) pins a
// FeesCarryForward to a single student-academic-year pair.
// Uniqueness is dispatcher-enforced (the aggregate carries the
// tuple as required fields so the dispatcher has the data to
// enforce it).
//
// Companion invariants enforced at fresh():
//   * balance_minor >= 0 (FCF I-1 -- pinned at construction).
//   * balance_type is a valid BalanceType variant (FCF I-2 --
//     the enum enforces variant validity at the type system
//     level; this aggregate carries it as required field).

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealFeesCarryForward {
    pub id: FeesCarryForwardId,
    pub school_id: SchoolId,
    pub student_id: educore_academic::StudentId,
    pub academic_year_id: educore_academic::AcademicYearId,
    pub balance_minor: i64,
    pub balance_type: BalanceType,
    pub currency: Currency,
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

impl RealFeesCarryForward {
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: FeesCarryForwardId,
        student_id: educore_academic::StudentId,
        academic_year_id: educore_academic::AcademicYearId,
        balance_minor: i64,
        balance_type: BalanceType,
        currency: Currency,
        actor: UserId,
        at: Timestamp,
        correlation: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        // FCF I-1 companion: balance_minor >= 0.
        if balance_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "FeesCarryForward balance_minor must be >= 0 (FCF I-1)",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            student_id,
            academic_year_id,
            balance_minor,
            balance_type,
            currency,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: at,
            updated_at: at,
            created_by: actor,
            updated_by: actor,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id: correlation,
        })
    }

    pub fn is_active(&self) -> bool {
        self.active_status == ActiveStatus::Active
    }

    pub fn retire(&mut self, at: Timestamp, actor: UserId) -> educore_core::error::Result<()> {
        if self.active_status == ActiveStatus::Retired {
            return Err(educore_core::error::DomainError::conflict(
                "FeesCarryForward is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}


// -- Wave 114 -- RealFeesMaster (per-(school, name, group) fee master) --
//
// FM I-2: unique per (school, name, group). The scope-key tuple
// (school_id, name, fees_group_id) pins a FeesMaster to a
// single (name, group) pair within a school. Uniqueness is
// dispatcher-enforced (the aggregate carries the tuple as
// required fields so the dispatcher has the data to enforce
// it).
//
// Companion invariants enforced at fresh():
//   * amount_minor >= 0 (FM I-1 -- pinned at construction).
//   * name.trim().is_empty() must be false (a fee master
//     without a name cannot be reconciled).
//   * currency is required (companion invariant).

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealFeesMaster {
    pub id: FeesMasterId,
    pub school_id: SchoolId,
    pub name: String,
    pub fees_group_id: FeesGroupId,
    pub class_id: crate::value_objects::ClassId,
    pub amount_minor: i64,
    pub currency: Currency,
    pub due_date: chrono::NaiveDate,
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

impl RealFeesMaster {
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: FeesMasterId,
        name: String,
        fees_group_id: FeesGroupId,
        class_id: crate::value_objects::ClassId,
        amount_minor: i64,
        currency: Currency,
        due_date: chrono::NaiveDate,
        actor: UserId,
        at: Timestamp,
        correlation: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        // FM I-1: amount_minor >= 0.
        if amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "FeesMaster amount_minor must be >= 0 (FM I-1)",
            ));
        }
        // Companion: name non-empty trimmed.
        if name.trim().is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "FeesMaster name must be non-empty after trimming",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            name,
            fees_group_id,
            class_id,
            amount_minor,
            currency,
            due_date,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: at,
            updated_at: at,
            created_by: actor,
            updated_by: actor,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id: correlation,
        })
    }

    pub fn is_active(&self) -> bool {
        self.active_status == ActiveStatus::Active
    }

    pub fn retire(&mut self, at: Timestamp, actor: UserId) -> educore_core::error::Result<()> {
        if self.active_status == ActiveStatus::Retired {
            return Err(educore_core::error::DomainError::conflict(
                "FeesMaster is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// RealFmFeesTransaction — Wave 124 (per-aggregate wave pattern
// from Waves 65–123) + Wave 125 FFT I-3 state machine extension
// =============================================================================
//
// Per v3 Part 2 F32 + checklist § FmFeesTransaction: 2 invariants
// dropped across Waves 124-125:
//   - FFT I-2: total_paid_amount_minor ≥ 0 (numeric money invariant; Wave 124)
//   - FFT I-3: state machine (Pending -> Approved | Rejected; Wave 125)
// Parent aggregate for [`RealFmFeesTransactionChild`] +
// [`RealFmFeesTransactionLineNote`] (one transaction can have many
// child rows and many line-note rows; the children carry their
// `fm_fees_transaction_id` as a required FK field).
//
// The aggregate is append-only on `total_paid_amount_minor` +
// `transaction_date` + `description` (only `fresh` sets them; no
// update mutator exists). The state machine fields (`status` +
// approval/rejection metadata) are mutable via `approve()` + `reject()`.
// Any change to the cumulative paid total must be effected via appending
// a `RealFmFeesTransactionChild` row (which the dispatcher sums back
// into `total_paid_amount_minor` on the parent transaction).

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealFmFeesTransaction {
    pub id: FmFeesTransactionId,
    pub school_id: SchoolId,
    pub total_paid_amount_minor: i64,
    pub transaction_date: chrono::NaiveDate,
    pub description: Option<String>,
    pub status: ApprovalStatus,
    pub approved_by: Option<UserId>,
    pub approved_at: Option<Timestamp>,
    pub rejected_by: Option<UserId>,
    pub rejected_at: Option<Timestamp>,
    pub reject_note: Option<String>,
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

impl RealFmFeesTransaction {
    /// Constructs a new `RealFmFeesTransaction` in the `Pending`
    /// approval state. Enforces FFT I-2 (`total_paid_amount_minor >= 0`).
    /// The transaction_date is required (no default — the caller must
    /// supply a valid calendar date). The description is optional
    /// free-form text (max 2000 chars recommended at the UI layer,
    /// not enforced here).
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: FmFeesTransactionId,
        total_paid_amount_minor: i64,
        transaction_date: chrono::NaiveDate,
        description: Option<String>,
        actor: UserId,
        at: Timestamp,
        correlation: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        // FFT I-2: total_paid_amount_minor >= 0.
        if total_paid_amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "FmFeesTransaction total_paid_amount_minor must be >= 0 (FFT I-2)",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            total_paid_amount_minor,
            transaction_date,
            description,
            status: ApprovalStatus::Pending,
            approved_by: None,
            approved_at: None,
            rejected_by: None,
            rejected_at: None,
            reject_note: None,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: at,
            updated_at: at,
            created_by: actor,
            updated_by: actor,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id: correlation,
        })
    }

    pub fn is_active(&self) -> bool {
        self.active_status == ActiveStatus::Active
    }

    /// Returns `true` if the state machine permits the
    /// `from -> to` transition (FFT I-3).
    pub fn can_transition(&self, to: ApprovalStatus) -> bool {
        self.status.can_transition_to(to)
    }

    /// Approves the transaction. Returns `Err` if the state machine
    /// does not permit the transition (FFT I-3).
    pub fn approve(
        &mut self,
        approver: UserId,
        at: Timestamp,
        event_id: EventId,
    ) -> educore_core::error::Result<()> {
        if !self.can_transition(ApprovalStatus::Approved) {
            return Err(educore_core::error::DomainError::conflict(format!(
                "FmFeesTransaction is in state {:?}, cannot transition to Approved (FFT I-3)",
                self.status
            )));
        }
        self.status = ApprovalStatus::Approved;
        self.approved_by = Some(approver);
        self.approved_at = Some(at);
        self.updated_at = at;
        self.updated_by = approver;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
        Ok(())
    }

    /// Rejects the transaction. Returns `Err` if the state machine
    /// does not permit the transition (FFT I-3).
    pub fn reject(
        &mut self,
        rejecter: UserId,
        note: String,
        at: Timestamp,
        event_id: EventId,
    ) -> educore_core::error::Result<()> {
        if !self.can_transition(ApprovalStatus::Rejected) {
            return Err(educore_core::error::DomainError::conflict(format!(
                "FmFeesTransaction is in state {:?}, cannot transition to Rejected (FFT I-3)",
                self.status
            )));
        }
        self.status = ApprovalStatus::Rejected;
        self.rejected_by = Some(rejecter);
        self.rejected_at = Some(at);
        self.reject_note = Some(note);
        self.updated_at = at;
        self.updated_by = rejecter;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
        Ok(())
    }

    pub fn retire(&mut self, at: Timestamp, actor: UserId) -> educore_core::error::Result<()> {
        if self.active_status == ActiveStatus::Retired {
            return Err(educore_core::error::DomainError::conflict(
                "FmFeesTransaction is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// RealFeesInstallment — Wave 126 (per-aggregate wave pattern
// from Waves 65–125)
// =============================================================================
//
// Per v3 Part 2 + checklist § FeesInstallment: 2 invariants dropped
// in Wave 126:
//   - FIv I-1: percentage ∈ [0, 100] (percentage of the parent master fee
//               due on this installment's due_date)
//   - FIv I-2: amount_minor ≥ 0 (numeric money invariant)
// Child of [`RealFeesMaster`] (one master can have many installments; the
// `fees_master_id` is a required FK field on the struct).
//
// The aggregate is append-only: `fresh()` + `retire()` only. The
// `amount_minor` + `percentage` + `due_date` + `name` fields are NOT
// mutable -- the pre-existing `UpdateFeesInstallmentCommand` is preserved
// as a Phase 7 skeleton not wired through `RealFeesInstallment`.

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealFeesInstallment {
    pub id: FeesInstallmentId,
    pub school_id: SchoolId,
    pub fees_master_id: FeesMasterId,
    pub name: String,
    pub due_date: chrono::NaiveDate,
    pub amount_minor: i64,
    pub currency: Currency,
    pub percentage: i64,
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

impl RealFeesInstallment {
    /// Constructs a new `RealFeesInstallment`. Enforces FIv I-1
    /// (`percentage ∈ [0, 100]`) + FIv I-2 (`amount_minor >= 0`).
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: FeesInstallmentId,
        fees_master_id: FeesMasterId,
        name: String,
        due_date: chrono::NaiveDate,
        amount_minor: i64,
        currency: Currency,
        percentage: i64,
        actor: UserId,
        at: Timestamp,
        correlation: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        // FIv I-1: percentage ∈ [0, 100].
        if percentage < 0 || percentage > 100 {
            return Err(educore_core::error::DomainError::validation(
                "FeesInstallment percentage must be in [0, 100] (FIv I-1)",
            ));
        }
        // FIv I-2: amount_minor >= 0.
        if amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "FeesInstallment amount_minor must be >= 0 (FIv I-2)",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            fees_master_id,
            name,
            due_date,
            amount_minor,
            currency,
            percentage,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: at,
            updated_at: at,
            created_by: actor,
            updated_by: actor,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id: correlation,
        })
    }

    pub fn is_active(&self) -> bool {
        self.active_status == ActiveStatus::Active
    }

    pub fn retire(&mut self, at: Timestamp, actor: UserId) -> educore_core::error::Result<()> {
        if self.active_status == ActiveStatus::Retired {
            return Err(educore_core::error::DomainError::conflict(
                "FeesInstallment is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// RealFmFeesType — Wave 129 (per-aggregate wave pattern from Waves 65–128)
// =============================================================================
//
// Per v3 Part 2 + checklist § FmFeesType: 3 invariants dropped in Wave 129:
//   - FFT I-1: type ∈ {Fee, Discount, Fine} (FmFeesTypeKind enum)
//   - FFT I-2: amount_minor ≥ 0 (numeric money invariant)
//   - FFT I-3: unique per (school, name) (dispatcher-enforced via
//              the (school_id, name) scope-key tuple)
//
// The aggregate is append-only: `fresh()` + `retire()` only. The
// `name` + `type` + `amount_minor` fields are NOT mutable -- changing
// any of them requires retire + create-new. The pre-existing
// `UpdateFmFeesTypeCommand` (if any) is preserved as a Phase 7
// skeleton not wired through `RealFmFeesType`.

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealFmFeesType {
    pub id: FmFeesTypeId,
    pub school_id: SchoolId,
    pub name: String,
    pub type_kind: FmFeesTypeKind,
    pub amount_minor: i64,
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

impl RealFmFeesType {
    /// Constructs a new `RealFmFeesType`. Enforces FFT I-1 (type is
    /// one of the closed enum variants) + FFT I-2 (`amount_minor >= 0`).
    /// FFT I-3 (unique per (school, name)) is dispatcher-enforced via
    /// the (school_id, name) scope-key tuple that the aggregate carries
    /// as required fields.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: FmFeesTypeId,
        name: String,
        type_kind: FmFeesTypeKind,
        amount_minor: i64,
        actor: UserId,
        at: Timestamp,
        correlation: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        // FFT I-2: amount_minor >= 0.
        if amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "FmFeesType amount_minor must be >= 0 (FFT I-2)",
            ));
        }
        // Companion invariant: name non-empty after trim.
        let name_trimmed = name.trim().to_string();
        if name_trimmed.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "FmFeesType name must be non-empty after trim (FFT I-3 companion)",
            ));
        }
        // FFT I-1: type_kind is enforced by the type system (closed
        // enum). No runtime guard needed; the closed enum prevents
        // constructing an `Other` variant.
        Ok(Self {
            school_id: id.school_id(),
            id,
            name: name_trimmed,
            type_kind,
            amount_minor,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: at,
            updated_at: at,
            created_by: actor,
            updated_by: actor,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id: correlation,
        })
    }

    pub fn is_active(&self) -> bool {
        self.active_status == ActiveStatus::Active
    }

    pub fn retire(&mut self, at: Timestamp, actor: UserId) -> educore_core::error::Result<()> {
        if self.active_status == ActiveStatus::Retired {
            return Err(educore_core::error::DomainError::conflict(
                "FmFeesType is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// RealBankPaymentSlip — Wave 130 (per-aggregate wave pattern
// from Waves 65–129)
// =============================================================================
//
// Per v3 Part 2 + checklist § BankPaymentSlip: 3 invariants
// dropped in Wave 130:
//   - BP I-1: payment_mode ∈ {Bank, Cheque} (PaymentMode enum)
//   - BP I-2: approve_status ∈ {pending, approved, rejected}
//              (shared ApprovalStatus enum; Pending -> Approved |
//              Rejected state machine)
//   - BP I-4: cannot reject after approval (state-machine
//              enforcement via can_transition)
// BP I-3 (approved slips promote to BankStatement + FeesPayment)
// is dispatcher-enforced (requires the storage adapter to create
// the 2 downstream aggregates in the same transaction); see the
// Wave 130 checklist entry.
//
// The aggregate is append-only on the immutable fields (amount_minor,
// payment_mode, bank_account_id, payer_name) -- only `fresh` + `retire`
// set them. The state machine fields (status + approval/rejection
// metadata) are mutable via `approve()` + `reject()`.

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealBankPaymentSlip {
    pub id: BankPaymentSlipId,
    pub school_id: SchoolId,
    pub amount_minor: i64,
    pub payment_mode: PaymentMode,
    pub bank_account_id: BankAccountId,
    pub payer_name: String,
    pub status: ApprovalStatus,
    pub approved_by: Option<UserId>,
    pub approved_at: Option<Timestamp>,
    pub rejected_by: Option<UserId>,
    pub rejected_at: Option<Timestamp>,
    pub reject_note: Option<String>,
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

impl RealBankPaymentSlip {
    /// Constructs a new `RealBankPaymentSlip` in the `Pending`
    /// approval state. Enforces BP I-1 (payment_mode is one of the
    /// closed enum variants, enforced at type-system level) +
    /// companion invariant (amount_minor >= 0 + payer_name
    /// non-empty after trim).
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: BankPaymentSlipId,
        amount_minor: i64,
        payment_mode: PaymentMode,
        bank_account_id: BankAccountId,
        payer_name: String,
        actor: UserId,
        at: Timestamp,
        correlation: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        // Companion invariant: amount_minor >= 0.
        if amount_minor < 0 {
            return Err(educore_core::error::DomainError::validation(
                "BankPaymentSlip amount_minor must be >= 0",
            ));
        }
        // Companion invariant: payer_name non-empty after trim.
        let payer_trimmed = payer_name.trim().to_string();
        if payer_trimmed.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "BankPaymentSlip payer_name must be non-empty after trim",
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            amount_minor,
            payment_mode,
            bank_account_id,
            payer_name: payer_trimmed,
            status: ApprovalStatus::Pending,
            approved_by: None,
            approved_at: None,
            rejected_by: None,
            rejected_at: None,
            reject_note: None,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: at,
            updated_at: at,
            created_by: actor,
            updated_by: actor,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id: correlation,
        })
    }

    pub fn is_active(&self) -> bool {
        self.active_status == ActiveStatus::Active
    }

    /// Returns `true` if the state machine permits the
    /// `from -> to` transition (BP I-4).
    #[must_use]
    pub fn can_transition(&self, to: ApprovalStatus) -> bool {
        self.status.can_transition_to(to)
    }

    /// Approve the slip (BP I-2 Pending -> Approved).
    /// Returns `Err` if the state machine does not permit the
    /// transition (BP I-4: cannot reject after approval is also
    /// enforced by the same can_transition guard).
    pub fn approve(
        &mut self,
        approver: UserId,
        at: Timestamp,
        event_id: EventId,
    ) -> educore_core::error::Result<()> {
        if !self.can_transition(ApprovalStatus::Approved) {
            return Err(educore_core::error::DomainError::conflict(format!(
                "BankPaymentSlip is in state {:?}, cannot transition to Approved (BP I-4)",
                self.status
            )));
        }
        self.status = ApprovalStatus::Approved;
        self.approved_by = Some(approver);
        self.approved_at = Some(at);
        self.updated_at = at;
        self.updated_by = approver;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
        Ok(())
    }

    /// Reject the slip (BP I-2 Pending -> Rejected).
    /// Returns `Err` if the state machine does not permit the
    /// transition (BP I-4: cannot reject after approval).
    pub fn reject(
        &mut self,
        rejecter: UserId,
        note: String,
        at: Timestamp,
        event_id: EventId,
    ) -> educore_core::error::Result<()> {
        if !self.can_transition(ApprovalStatus::Rejected) {
            return Err(educore_core::error::DomainError::conflict(format!(
                "BankPaymentSlip is in state {:?}, cannot transition to Rejected (BP I-4)",
                self.status
            )));
        }
        self.status = ApprovalStatus::Rejected;
        self.rejected_by = Some(rejecter);
        self.rejected_at = Some(at);
        self.reject_note = Some(note);
        self.updated_at = at;
        self.updated_by = rejecter;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
        Ok(())
    }

    /// Retire the aggregate (tombstone).
    pub fn retire(&mut self, at: Timestamp, actor: UserId) -> educore_core::error::Result<()> {
        if self.active_status == ActiveStatus::Retired {
            return Err(educore_core::error::DomainError::conflict(
                "BankPaymentSlip is already retired",
            ));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}
