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
    ApprovalStatus, BalanceType, BankAccountId, ChartOfAccountId, Currency, DiscountType, DonorId,
    DueFeesLoginPreventId, ExpenseHeadId, ExpenseId, FeesAssignDiscountId, FeesAssignId,
    FeesCarryForwardId, FeesCarryForwardLogId, FeesCarryForwardSettingId, FeesDiscountId,
    FeesGroupId, FeesInstallmentAssignId, FeesInstallmentCreditId, FeesInstallmentId,
    FeesInvoiceId, FeesInvoiceSettingId, FeesMasterId, FeesPaymentId, FeesPaymentStatus,
    FeesTypeId, FineAmount, FmFeesGroupId, FmFeesInvoiceChildId, FmFeesInvoiceId,
    FmFeesInvoiceSettingId, FmFeesTransactionChildId, FmFeesTransactionId, FmFeesTypeId,
    FmFeesWeaverId, FmInvoiceType, IncomeHeadId, InvoiceSettingId, Money, PaymentGatewaySettingId,
    PaymentMethodId, PaymentMethodKind, PayrollPaymentId, ProductPurchaseId, QuestionBankFeeId,
    StatementType, WalletId, WalletTransactionId, WalletTxType,
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
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: ExpenseId,
        name: String,
        amount_minor: i64,
        currency: Currency,
        expense_head_id: ExpenseHeadId,
        account_id: BankAccountId,
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
        Ok(Self {
            school_id: id.school_id(),
            id,
            name,
            amount_minor,
            currency,
            expense_head_id,
            account_id,
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
    // FeesInstallmentCredit) are intentionally left as 1-field
    // placeholder stubs. They will be filled in by Workstreams
    // D/E/F/G/H/I/J/K/L/M. The acceptance tests for these
    // aggregates will be added when each is implemented.
    // -------------------------------------------------------------------------

    #[test]
    #[ignore = "backlog: 33 placeholder aggregates need Workstreams D-M"]
    fn unimplemented_placeholder_aggregates_backlog() {
        // Documents the 33 placeholder aggregates above. When
        // each is implemented, the corresponding test is added
        // and this ignore attribute is removed.
    }
}
