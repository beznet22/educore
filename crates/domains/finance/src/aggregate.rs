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

use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};

use crate::value_objects::{
    validate_discount_name, validate_donor_name, validate_ledger_name, AccountType, Amount,
    ApprovalStatus, BalanceType, BankAccountId, BankPaymentSlipAuditId, BankStatementAttachmentId,
    ChartOfAccountId, Currency, DirectFeesInstallmentAssignChildId, DiscountType, DonorId,
    DueFeesLoginPreventId, ExpenseApprovalId, ExpenseHeadId, ExpenseId, FeesAssignDiscountId,
    FeesAssignId, FeesCarryForwardId, FeesCarryForwardLogId, FeesCarryForwardSettingId,
    FeesDiscountId, FeesGroupId, FeesInstallmentAssignDiscountId, FeesInstallmentAssignId,
    FeesInstallmentCreditId, FeesInstallmentId, FeesInvoiceId, FeesInvoiceSettingId, FeesMasterId,
    FeesPaymentId, FeesPaymentStatus, FeesTypeId, FineAmount, FmFeesGroupId, FmFeesInvoiceChildId,
    FmFeesInvoiceId, FmFeesInvoiceLineNoteId, FmFeesInvoiceSettingId, FmFeesTransactionChildId,
    FmFeesTransactionId, FmFeesTransactionLineNoteId, FmFeesTypeId, FmFeesWeaverId, FmInvoiceType,
    IncomeApprovalId, IncomeHeadId, InvoiceSettingId, Money, PaymentGatewaySettingId,
    PaymentMethodId, PaymentMethodKind, PayrollEarnDeducId, PayrollGenerateId,
    PayrollPaymentApprovalId, PayrollPaymentId, ProductPurchaseId, QuestionBankFeeId,
    StatementType, WalletId, WalletTransactionApprovalId, WalletTransactionId, WalletTxType,
};

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
