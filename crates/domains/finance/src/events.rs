//! # Finance domain events
//!
//! Every aggregate's state change emits an event implementing
//! [`DomainEvent`](::educore_events::domain_event::DomainEvent).
//! The full set follows the spec at `docs/specs/finance/events.md`.
//!
//! Wire form: `finance.<aggregate>.<verb>` (e.g.
//! `finance.wallet.credited`, `finance.wallet.refund_requested`,
//! `finance.payroll_payment.recorded`).
//!
//! Workstream A ships the 5 headline events for `Wallet` +
//! `WalletTransaction` (incl. the `Refund` headline) +
//! `FeesInvoiceConfigured` (the invoice numbering service) +
//! `ExpenseRecorded` (the expense headline).

#![allow(missing_docs)]
#![allow(unused_imports)]

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::{CorrelationId, EventId, Identifier, SchoolId, UserId};
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;

use crate::value_objects::{
    AccountType, AmountTransferId, ApprovalStatus, BalanceType, BankAccountId, BankPaymentSlipAuditId, BankPaymentSlipId, BankStatementAttachmentId,
    BankStatementId, StatementType,
    ChartOfAccountId, Currency, DirectFeesInstallmentAssignChildId, DirectFeesInstallmentAssignId, DiscountType,
    DirectFeesInstallmentChildPaymentId, DirectFeesInstallmentId, DirectFeesReminderId, DirectFeesSettingId, DonorId,
    DueFeesLoginPreventId, FeesInstallmentCreditId, FeesInvoiceSettingId,
    ExpenseApprovalId, ExpenseHeadId, ExpenseId, FeesAssignDiscountId, FeesAssignId,
    FeesCarryForwardId, FeesCarryForwardLogId, FeesCarryForwardSettingId, FeesDiscountId, FeesGroupId, FeesInstallmentAssignDiscountId, FeesInstallmentAssignId,
    FeesInstallmentId,
    FeesMasterId, FeesPaymentId, FeesTypeId, FmFeesGroupId, FmFeesInvoiceId,
    FmFeesInvoiceLineNoteId, FmFeesInvoiceSettingId, FmFeesTransactionChildId, FmFeesTransactionId,
    FmFeesWeaverId,
    FmFeesTransactionLineNoteId, IncomeApprovalId, IncomeHeadId,
    IncomeId, InventoryPaymentId, InvoiceSettingId, PaymentMethodId, PaymentMethodKind, PayrollGenerateId, ProductPurchaseId, FmFeesInvoiceChildId,
    PayrollPaymentApprovalId, PayrollPaymentId, QuestionBankFeeId, SalaryTemplateId, WalletId,
    WalletTransactionApprovalId, WalletTransactionId, WalletTxType, TransactionId,
};

use educore_academic::{AcademicYearId, ClassId, SectionId, StudentId};

// =============================================================================
// Wallet events
// =============================================================================

/// Emitted when a new `Wallet` is created (lazy on first
/// `WalletTransaction`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalletCreated {
    pub wallet_id: WalletId,
    pub user_id: UserId,
    pub currency: Currency,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl WalletCreated {
    pub fn new(
        wallet_id: WalletId,
        user_id: UserId,
        currency: Currency,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            wallet_id,
            user_id,
            currency,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for WalletCreated {
    const EVENT_TYPE: &'static str = "finance.wallet.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "wallet";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.wallet_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.wallet_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a wallet is credited (deposit / refund).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalletCredited {
    pub wallet_id: WalletId,
    pub wallet_transaction_id: WalletTransactionId,
    pub user_id: UserId,
    pub amount_minor: i64,
    pub currency: Currency,
    pub wallet_type: WalletTxType,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl WalletCredited {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        wallet_id: WalletId,
        wallet_transaction_id: WalletTransactionId,
        user_id: UserId,
        amount_minor: i64,
        currency: Currency,
        wallet_type: WalletTxType,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            wallet_id,
            wallet_transaction_id,
            user_id,
            amount_minor,
            currency,
            wallet_type,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for WalletCredited {
    const EVENT_TYPE: &'static str = "finance.wallet.credited";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "wallet";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.wallet_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.wallet_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a wallet is debited (expense / fees refund).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalletDebited {
    pub wallet_id: WalletId,
    pub wallet_transaction_id: WalletTransactionId,
    pub user_id: UserId,
    pub amount_minor: i64,
    pub currency: Currency,
    pub wallet_type: WalletTxType,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl WalletDebited {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        wallet_id: WalletId,
        wallet_transaction_id: WalletTransactionId,
        user_id: UserId,
        amount_minor: i64,
        currency: Currency,
        wallet_type: WalletTxType,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            wallet_id,
            wallet_transaction_id,
            user_id,
            amount_minor,
            currency,
            wallet_type,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for WalletDebited {
    const EVENT_TYPE: &'static str = "finance.wallet.debited";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "wallet";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.wallet_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.wallet_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a wallet refund is requested. The transaction is
/// in `Pending` state and must be approved to credit the wallet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalletRefundRequested {
    pub wallet_transaction_id: WalletTransactionId,
    pub wallet_id: WalletId,
    pub user_id: UserId,
    pub amount_minor: i64,
    pub currency: Currency,
    pub reason: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl WalletRefundRequested {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        wallet_transaction_id: WalletTransactionId,
        wallet_id: WalletId,
        user_id: UserId,
        amount_minor: i64,
        currency: Currency,
        reason: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            wallet_transaction_id,
            wallet_id,
            user_id,
            amount_minor,
            currency,
            reason,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for WalletRefundRequested {
    const EVENT_TYPE: &'static str = "finance.wallet.refund_requested";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "wallet";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.wallet_transaction_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.wallet_transaction_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a wallet transaction transitions to `Approved`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalletTransactionApproved {
    pub wallet_transaction_id: WalletTransactionId,
    pub wallet_id: WalletId,
    pub approver_id: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl WalletTransactionApproved {
    pub fn new(
        wallet_transaction_id: WalletTransactionId,
        wallet_id: WalletId,
        approver_id: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            wallet_transaction_id,
            wallet_id,
            approver_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for WalletTransactionApproved {
    const EVENT_TYPE: &'static str = "finance.wallet_transaction.approved";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "wallet_transaction";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.wallet_transaction_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.wallet_transaction_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a wallet transaction transitions to `Rejected`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalletTransactionRejected {
    pub wallet_transaction_id: WalletTransactionId,
    pub wallet_id: WalletId,
    pub rejecter_id: UserId,
    pub reject_note: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl WalletTransactionRejected {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        wallet_transaction_id: WalletTransactionId,
        wallet_id: WalletId,
        rejecter_id: UserId,
        reject_note: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            wallet_transaction_id,
            wallet_id,
            rejecter_id,
            reject_note,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for WalletTransactionRejected {
    const EVENT_TYPE: &'static str = "finance.wallet_transaction.rejected";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "wallet_transaction";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.wallet_transaction_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.wallet_transaction_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// Invoice + payment + expense + payroll events (headline 5 + 6)
// =============================================================================

/// Emitted when the school's invoice numbering is configured.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvoiceNumberingConfigured {
    pub fees_invoice_id: crate::value_objects::FeesInvoiceId,
    pub prefix: String,
    pub start_form: i64,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl InvoiceNumberingConfigured {
    pub fn new(
        fees_invoice_id: crate::value_objects::FeesInvoiceId,
        prefix: String,
        start_form: i64,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fees_invoice_id,
            prefix,
            start_form,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for InvoiceNumberingConfigured {
    const EVENT_TYPE: &'static str = "finance.fees_invoice.configured";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fees_invoice";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fees_invoice_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fees_invoice_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `PaymentReceived` event fires (per the spec).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaymentReceived {
    pub fees_payment_id: crate::value_objects::FeesPaymentId,
    pub amount_minor: i64,
    pub currency: Currency,
    pub discount_minor: i64,
    pub fine_minor: i64,
    pub payment_method: PaymentMethodKind,
    pub bank_id: Option<BankAccountId>,
    pub payment_date: NaiveDate,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl PaymentReceived {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        fees_payment_id: crate::value_objects::FeesPaymentId,
        amount_minor: i64,
        currency: Currency,
        discount_minor: i64,
        fine_minor: i64,
        payment_method: PaymentMethodKind,
        bank_id: Option<BankAccountId>,
        payment_date: NaiveDate,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fees_payment_id,
            amount_minor,
            currency,
            discount_minor,
            fine_minor,
            payment_method,
            bank_id,
            payment_date,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for PaymentReceived {
    const EVENT_TYPE: &'static str = "finance.fees_payment.recorded";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fees_payment";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fees_payment_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fees_payment_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when an `Expense` is recorded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpenseRecorded {
    pub expense_id: crate::value_objects::ExpenseId,
    pub name: String,
    pub amount_minor: i64,
    pub currency: Currency,
    pub expense_head_id: ExpenseHeadId,
    pub account_id: BankAccountId,
    pub payment_method: PaymentMethodKind,
    pub expense_date: NaiveDate,
    pub payroll_payment_id: Option<PayrollPaymentId>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ExpenseRecorded {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        expense_id: crate::value_objects::ExpenseId,
        name: String,
        amount_minor: i64,
        currency: Currency,
        expense_head_id: ExpenseHeadId,
        account_id: BankAccountId,
        payment_method: PaymentMethodKind,
        expense_date: NaiveDate,
        payroll_payment_id: Option<PayrollPaymentId>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            expense_id,
            name,
            amount_minor,
            currency,
            expense_head_id,
            account_id,
            payment_method,
            expense_date,
            payroll_payment_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ExpenseRecorded {
    const EVENT_TYPE: &'static str = "finance.expense.recorded";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "expense";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.expense_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.expense_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a finance-side `PayrollPayment` is recorded (the
/// HR→finance bridge).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayrollPaymentRecorded {
    pub payroll_payment_id: PayrollPaymentId,
    pub amount_minor: i64,
    pub currency: Currency,
    pub payment_method: PaymentMethodKind,
    pub bank_id: Option<BankAccountId>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl PayrollPaymentRecorded {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        payroll_payment_id: PayrollPaymentId,
        amount_minor: i64,
        currency: Currency,
        payment_method: PaymentMethodKind,
        bank_id: Option<BankAccountId>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            payroll_payment_id,
            amount_minor,
            currency,
            payment_method,
            bank_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for PayrollPaymentRecorded {
    const EVENT_TYPE: &'static str = "finance.payroll_payment.recorded";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "payroll_payment";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.payroll_payment_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.payroll_payment_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// IncomeHead events (Wave 65 — RealIncomeHead headline events)
// =============================================================================

/// Emitted when a new `RealIncomeHead` is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncomeHeadCreated {
    pub income_head_id: IncomeHeadId,
    pub name: String,
    pub description: Option<String>,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl IncomeHeadCreated {
    pub fn new(
        income_head_id: IncomeHeadId,
        name: String,
        description: Option<String>,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            income_head_id,
            name,
            description,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for IncomeHeadCreated {
    const EVENT_TYPE: &'static str = "finance.income_head.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "income_head";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.income_head_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.income_head_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealIncomeHead` is updated (name/description change).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncomeHeadUpdated {
    pub income_head_id: IncomeHeadId,
    pub name: String,
    pub description: Option<String>,
    pub updated_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl IncomeHeadUpdated {
    pub fn new(
        income_head_id: IncomeHeadId,
        name: String,
        description: Option<String>,
        updated_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            income_head_id,
            name,
            description,
            updated_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for IncomeHeadUpdated {
    const EVENT_TYPE: &'static str = "finance.income_head.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "income_head";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.income_head_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.income_head_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealIncomeHead` is retired (soft-deleted via
/// `RealIncomeHead::retire`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncomeHeadDeleted {
    pub income_head_id: IncomeHeadId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl IncomeHeadDeleted {
    pub fn new(
        income_head_id: IncomeHeadId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            income_head_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for IncomeHeadDeleted {
    const EVENT_TYPE: &'static str = "finance.income_head.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "income_head";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.income_head_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.income_head_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// FmFeesGroup events (Wave 66 — RealFmFeesGroup headline events)
// =============================================================================

/// Emitted when a new `RealFmFeesGroup` is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FmFeesGroupCreated {
    pub fm_fees_group_id: FmFeesGroupId,
    pub name: String,
    pub description: Option<String>,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FmFeesGroupCreated {
    pub fn new(
        fm_fees_group_id: FmFeesGroupId,
        name: String,
        description: Option<String>,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fm_fees_group_id,
            name,
            description,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FmFeesGroupCreated {
    const EVENT_TYPE: &'static str = "finance.fm_fees_group.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fm_fees_group";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fm_fees_group_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fm_fees_group_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealFmFeesGroup` is updated (name/description change).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FmFeesGroupUpdated {
    pub fm_fees_group_id: FmFeesGroupId,
    pub name: String,
    pub description: Option<String>,
    pub updated_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FmFeesGroupUpdated {
    pub fn new(
        fm_fees_group_id: FmFeesGroupId,
        name: String,
        description: Option<String>,
        updated_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fm_fees_group_id,
            name,
            description,
            updated_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FmFeesGroupUpdated {
    const EVENT_TYPE: &'static str = "finance.fm_fees_group.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fm_fees_group";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fm_fees_group_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fm_fees_group_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealFmFeesGroup` is retired (soft-deleted via
/// `RealFmFeesGroup::retire`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FmFeesGroupDeleted {
    pub fm_fees_group_id: FmFeesGroupId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FmFeesGroupDeleted {
    pub fn new(
        fm_fees_group_id: FmFeesGroupId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fm_fees_group_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FmFeesGroupDeleted {
    const EVENT_TYPE: &'static str = "finance.fm_fees_group.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fm_fees_group";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fm_fees_group_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fm_fees_group_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// InvoiceSetting events (Wave 67 — RealInvoiceSetting headline events)
// =============================================================================

/// Emitted when a new `RealInvoiceSetting` is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvoiceSettingCreated {
    pub invoice_setting_id: InvoiceSettingId,
    pub prefix: String,
    pub start_form: i64,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl InvoiceSettingCreated {
    pub fn new(
        invoice_setting_id: InvoiceSettingId,
        prefix: String,
        start_form: i64,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            invoice_setting_id,
            prefix,
            start_form,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for InvoiceSettingCreated {
    const EVENT_TYPE: &'static str = "finance.invoice_setting.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "invoice_setting";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.invoice_setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.invoice_setting_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealInvoiceSetting` is updated (prefix / start_form
/// change).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvoiceSettingUpdated {
    pub invoice_setting_id: InvoiceSettingId,
    pub prefix: String,
    pub start_form: i64,
    pub updated_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl InvoiceSettingUpdated {
    pub fn new(
        invoice_setting_id: InvoiceSettingId,
        prefix: String,
        start_form: i64,
        updated_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            invoice_setting_id,
            prefix,
            start_form,
            updated_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for InvoiceSettingUpdated {
    const EVENT_TYPE: &'static str = "finance.invoice_setting.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "invoice_setting";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.invoice_setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.invoice_setting_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealInvoiceSetting` is retired (soft-deleted via
/// `RealInvoiceSetting::retire`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvoiceSettingDeleted {
    pub invoice_setting_id: InvoiceSettingId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl InvoiceSettingDeleted {
    pub fn new(
        invoice_setting_id: InvoiceSettingId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            invoice_setting_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for InvoiceSettingDeleted {
    const EVENT_TYPE: &'static str = "finance.invoice_setting.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "invoice_setting";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.invoice_setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.invoice_setting_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// QuestionBankFee events (Wave 68 — RealQuestionBankFee headline events)
// =============================================================================

/// Emitted when a new `RealQuestionBankFee` is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuestionBankFeeCreated {
    pub question_bank_fee_id: QuestionBankFeeId,
    pub name: String,
    pub amount_minor: i64,
    pub description: Option<String>,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl QuestionBankFeeCreated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        question_bank_fee_id: QuestionBankFeeId,
        name: String,
        amount_minor: i64,
        description: Option<String>,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            question_bank_fee_id,
            name,
            amount_minor,
            description,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for QuestionBankFeeCreated {
    const EVENT_TYPE: &'static str = "finance.question_bank_fee.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "question_bank_fee";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.question_bank_fee_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.question_bank_fee_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealQuestionBankFee` is updated (name/amount_minor/
/// description change).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuestionBankFeeUpdated {
    pub question_bank_fee_id: QuestionBankFeeId,
    pub name: String,
    pub amount_minor: i64,
    pub description: Option<String>,
    pub updated_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl QuestionBankFeeUpdated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        question_bank_fee_id: QuestionBankFeeId,
        name: String,
        amount_minor: i64,
        description: Option<String>,
        updated_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            question_bank_fee_id,
            name,
            amount_minor,
            description,
            updated_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for QuestionBankFeeUpdated {
    const EVENT_TYPE: &'static str = "finance.question_bank_fee.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "question_bank_fee";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.question_bank_fee_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.question_bank_fee_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealQuestionBankFee` is retired (soft-deleted via
/// `RealQuestionBankFee::retire`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuestionBankFeeDeleted {
    pub question_bank_fee_id: QuestionBankFeeId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl QuestionBankFeeDeleted {
    pub fn new(
        question_bank_fee_id: QuestionBankFeeId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            question_bank_fee_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for QuestionBankFeeDeleted {
    const EVENT_TYPE: &'static str = "finance.question_bank_fee.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "question_bank_fee";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.question_bank_fee_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.question_bank_fee_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// DirectFeesSetting events (Wave 69 — RealDirectFeesSetting headline events)
// =============================================================================

/// Emitted when a new `RealDirectFeesSetting` is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectFeesSettingCreated {
    pub direct_fees_setting_id: DirectFeesSettingId,
    pub enabled: bool,
    pub reminder_before: i64,
    pub no_installment: i64,
    pub due_date_from_sem: u8,
    pub description: Option<String>,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DirectFeesSettingCreated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        direct_fees_setting_id: DirectFeesSettingId,
        enabled: bool,
        reminder_before: i64,
        no_installment: i64,
        due_date_from_sem: u8,
        description: Option<String>,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            direct_fees_setting_id,
            enabled,
            reminder_before,
            no_installment,
            due_date_from_sem,
            description,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DirectFeesSettingCreated {
    const EVENT_TYPE: &'static str = "finance.direct_fees_setting.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "direct_fees_setting";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.direct_fees_setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.direct_fees_setting_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealDirectFeesSetting` is updated (any of enabled /
/// reminder_before / no_installment / due_date_from_sem / description
/// changes).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectFeesSettingUpdated {
    pub direct_fees_setting_id: DirectFeesSettingId,
    pub enabled: bool,
    pub reminder_before: i64,
    pub no_installment: i64,
    pub due_date_from_sem: u8,
    pub description: Option<String>,
    pub updated_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DirectFeesSettingUpdated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        direct_fees_setting_id: DirectFeesSettingId,
        enabled: bool,
        reminder_before: i64,
        no_installment: i64,
        due_date_from_sem: u8,
        description: Option<String>,
        updated_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            direct_fees_setting_id,
            enabled,
            reminder_before,
            no_installment,
            due_date_from_sem,
            description,
            updated_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DirectFeesSettingUpdated {
    const EVENT_TYPE: &'static str = "finance.direct_fees_setting.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "direct_fees_setting";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.direct_fees_setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.direct_fees_setting_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealDirectFeesSetting` is retired (soft-deleted via
/// `RealDirectFeesSetting::retire`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectFeesSettingDeleted {
    pub direct_fees_setting_id: DirectFeesSettingId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DirectFeesSettingDeleted {
    pub fn new(
        direct_fees_setting_id: DirectFeesSettingId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            direct_fees_setting_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DirectFeesSettingDeleted {
    const EVENT_TYPE: &'static str = "finance.direct_fees_setting.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "direct_fees_setting";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.direct_fees_setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.direct_fees_setting_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// FeesCarryForwardLog events (Wave 70 — RealFeesCarryForwardLog headline events)
// =============================================================================
//
// FCFL I-1 (append-only) is enforced at the API surface by NOT emitting
// an `Updated` event for this aggregate. The only two transitions are
// create (always present) and retire (soft-tombstone, never a
// modification of the original record).

/// Emitted when a new `RealFeesCarryForwardLog` row is appended.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesCarryForwardLogCreated {
    pub fees_carry_forward_log_id: FeesCarryForwardLogId,
    pub student_id: StudentId,
    pub academic_year_id: AcademicYearId,
    pub amount_minor: i64,
    pub description: Option<String>,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FeesCarryForwardLogCreated {
    pub fn new(
        fees_carry_forward_log_id: FeesCarryForwardLogId,
        student_id: StudentId,
        academic_year_id: AcademicYearId,
        amount_minor: i64,
        description: Option<String>,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fees_carry_forward_log_id,
            student_id,
            academic_year_id,
            amount_minor,
            description,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FeesCarryForwardLogCreated {
    const EVENT_TYPE: &'static str = "finance.fees_carry_forward_log.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fees_carry_forward_log";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fees_carry_forward_log_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fees_carry_forward_log_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealFeesCarryForwardLog` row is retired (soft-deleted
/// via `RealFeesCarryForwardLog::retire`). Note: this does NOT violate
/// FCFL I-1 (append-only) — the tombstone preserves the original record
/// in the audit footer + the `Retired` active_status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesCarryForwardLogRetired {
    pub fees_carry_forward_log_id: FeesCarryForwardLogId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FeesCarryForwardLogRetired {
    pub fn new(
        fees_carry_forward_log_id: FeesCarryForwardLogId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fees_carry_forward_log_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FeesCarryForwardLogRetired {
    const EVENT_TYPE: &'static str = "finance.fees_carry_forward_log.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fees_carry_forward_log";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fees_carry_forward_log_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fees_carry_forward_log_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// ChartOfAccount events (Wave 74 — RealChartOfAccount headline events)
// =============================================================================
//
// Per v3 Part 2 F7 + checklist § ChartOfAccount: 2 invariants:
//   - COA I-1: unique name within school (shape validated here;
//              per-school uniqueness is the dispatcher's concern)
//   - COA I-2: cannot delete while referenced (reference integrity
//              is the dispatcher's concern; the `ChartOfAccountDeleted`
//              event is only emitted by the dispatcher when no
//              references exist; the aggregate's `retire()` method
//              emits `ChartOfAccountDeleted` after the dispatcher
//              confirms reference integrity)

/// Emitted when a new `RealChartOfAccount` is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChartOfAccountCreated {
    pub chart_of_account_id: ChartOfAccountId,
    pub code: String,
    pub name: String,
    pub account_type: AccountType,
    pub description: Option<String>,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ChartOfAccountCreated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        chart_of_account_id: ChartOfAccountId,
        code: String,
        name: String,
        account_type: AccountType,
        description: Option<String>,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            chart_of_account_id,
            code,
            name,
            account_type,
            description,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ChartOfAccountCreated {
    const EVENT_TYPE: &'static str = "finance.chart_of_account.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "chart_of_account";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.chart_of_account_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.chart_of_account_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealChartOfAccount`'s metadata (code / name /
/// account_type / description) is updated via
/// `RealChartOfAccount::update_metadata`. Per-school name uniqueness
/// is the dispatcher's concern and is enforced outside the aggregate
/// (this event is only emitted when the dispatcher confirms the new
/// name does not collide with any other `RealChartOfAccount` in the
/// same school).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChartOfAccountUpdated {
    pub chart_of_account_id: ChartOfAccountId,
    pub code: String,
    pub name: String,
    pub account_type: AccountType,
    pub description: Option<String>,
    pub updated_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ChartOfAccountUpdated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        chart_of_account_id: ChartOfAccountId,
        code: String,
        name: String,
        account_type: AccountType,
        description: Option<String>,
        updated_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            chart_of_account_id,
            code,
            name,
            account_type,
            description,
            updated_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ChartOfAccountUpdated {
    const EVENT_TYPE: &'static str = "finance.chart_of_account.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "chart_of_account";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.chart_of_account_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.chart_of_account_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealChartOfAccount` is retired (soft-deleted via
/// `RealChartOfAccount::retire`). Per COA I-2, the service layer MUST
/// check reference integrity (no ledger entries reference this
/// chart-of-account) BEFORE calling `retire`; the dispatcher rejects
/// the `Delete` command when references exist. This event marks a
/// tombstone for legal-record retention; the original code/name/
/// account_type are preserved in the audit footer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChartOfAccountDeleted {
    pub chart_of_account_id: ChartOfAccountId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ChartOfAccountDeleted {
    pub fn new(
        chart_of_account_id: ChartOfAccountId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            chart_of_account_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ChartOfAccountDeleted {
    const EVENT_TYPE: &'static str = "finance.chart_of_account.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "chart_of_account";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.chart_of_account_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.chart_of_account_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// FmFeesInvoiceLineNote events (Wave 72 — RealFmFeesInvoiceLineNote headline events)
// =============================================================================
//
// Per v3 Part 2 F30 + checklist § FmFeesInvoiceLineNote: 2 invariants:
//   - FFILN I-1: note non-empty (validated in RealFmFeesInvoiceLineNote::fresh)
//   - FFILN I-2: append-only (enforced at the API surface by NOT emitting
//                an `Updated` event for this aggregate). The only two
//                transitions are create (always present) and retire
//                (soft-tombstone, never a modification of the original
//                record).

/// Emitted when a new `RealFmFeesInvoiceLineNote` row is appended.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FmFeesInvoiceLineNoteCreated {
    pub fm_fees_invoice_line_note_id: FmFeesInvoiceLineNoteId,
    pub fm_fees_invoice_id: FmFeesInvoiceId,
    pub note: String,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FmFeesInvoiceLineNoteCreated {
    pub fn new(
        fm_fees_invoice_line_note_id: FmFeesInvoiceLineNoteId,
        fm_fees_invoice_id: FmFeesInvoiceId,
        note: String,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fm_fees_invoice_line_note_id,
            fm_fees_invoice_id,
            note,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FmFeesInvoiceLineNoteCreated {
    const EVENT_TYPE: &'static str = "finance.fm_fees_invoice_line_note.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fm_fees_invoice_line_note";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fm_fees_invoice_line_note_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fm_fees_invoice_line_note_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealFmFeesInvoiceLineNote` row is retired
/// (soft-deleted via `RealFmFeesInvoiceLineNote::retire`). Note: this
/// does NOT violate FFILN I-2 (append-only) — the tombstone preserves
/// the original record in the audit footer + the `Retired` active_status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FmFeesInvoiceLineNoteRetired {
    pub fm_fees_invoice_line_note_id: FmFeesInvoiceLineNoteId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FmFeesInvoiceLineNoteRetired {
    pub fn new(
        fm_fees_invoice_line_note_id: FmFeesInvoiceLineNoteId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fm_fees_invoice_line_note_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FmFeesInvoiceLineNoteRetired {
    const EVENT_TYPE: &'static str = "finance.fm_fees_invoice_line_note.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fm_fees_invoice_line_note";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fm_fees_invoice_line_note_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fm_fees_invoice_line_note_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// Donor events (Wave 71 — RealDonor headline events)
// =============================================================================

/// Emitted when a new `RealDonor` is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DonorCreated {
    pub donor_id: DonorId,
    pub name: String,
    pub email: String,
    pub show_public: bool,
    pub phone: Option<String>,
    pub description: Option<String>,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DonorCreated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        donor_id: DonorId,
        name: String,
        email: String,
        show_public: bool,
        phone: Option<String>,
        description: Option<String>,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            donor_id,
            name,
            email,
            show_public,
            phone,
            description,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DonorCreated {
    const EVENT_TYPE: &'static str = "finance.donor.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "donor";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.donor_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.donor_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealDonor` is updated (name / email / show_public /
/// phone / description change).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DonorUpdated {
    pub donor_id: DonorId,
    pub name: String,
    pub email: String,
    pub show_public: bool,
    pub phone: Option<String>,
    pub description: Option<String>,
    pub updated_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DonorUpdated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        donor_id: DonorId,
        name: String,
        email: String,
        show_public: bool,
        phone: Option<String>,
        description: Option<String>,
        updated_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            donor_id,
            name,
            email,
            show_public,
            phone,
            description,
            updated_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DonorUpdated {
    const EVENT_TYPE: &'static str = "finance.donor.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "donor";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.donor_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.donor_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealDonor` is retired (soft-deleted via
/// `RealDonor::retire`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DonorDeleted {
    pub donor_id: DonorId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DonorDeleted {
    pub fn new(
        donor_id: DonorId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            donor_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DonorDeleted {
    const EVENT_TYPE: &'static str = "finance.donor.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "donor";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.donor_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.donor_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// Aggregate headline event stubs (Cluster D final 20%).
// Each stub carries only `event_id`, `school_id`, `aggregate_id`,
// `correlation_id`, and `occurred_at`. Real payload fields land with the
// workstream that fills in the corresponding aggregate. The lint in
// `educore-core::lint::spec_to_code` requires that every event declared
// in `docs/specs/finance/events.md` has a `pub struct` of the same name
// in this file; the macro below generates the minimal conformant shape.
// =============================================================================

/// Generates a stub `DomainEvent` for a finance aggregate headline.
/// Mirrors the hand-written child-entity stubs below but condensed into
/// a single macro invocation per event.
macro_rules! finance_event_stub {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident;
        event_type: $event_type:expr,
        aggregate_type: $aggregate_type:expr,
        aggregate_id: $agg_id:ty $(,)?
    ) => {
        $(#[$attr])*
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        $vis struct $name {
            pub event_id: EventId,
            pub school_id: SchoolId,
            pub aggregate_id: $agg_id,
            pub correlation_id: CorrelationId,
            pub occurred_at: Timestamp,
        }

        impl $name {
            pub fn new(
                event_id: EventId,
                school_id: SchoolId,
                aggregate_id: $agg_id,
                correlation_id: CorrelationId,
                occurred_at: Timestamp,
            ) -> Self {
                Self {
                    event_id,
                    school_id,
                    aggregate_id,
                    correlation_id,
                    occurred_at,
                }
            }
        }

        impl DomainEvent for $name {
            const EVENT_TYPE: &'static str = $event_type;
            const SCHEMA_VERSION: u32 = 1;
            const AGGREGATE_TYPE: &'static str = $aggregate_type;
            fn event_id(&self) -> EventId {
                self.event_id
            }
            fn aggregate_id(&self) -> Uuid {
                self.aggregate_id.as_uuid()
            }
            fn school_id(&self) -> SchoolId {
                self.school_id
            }
            fn occurred_at(&self) -> Timestamp {
                self.occurred_at
            }
        }
    };
}

finance_event_stub! {
    /// Emitted when a new `FeesType` aggregate is created.
    pub struct FeesTypeCreated;
    event_type: "finance.fees_type.created",
    aggregate_type: "fees_type",
    aggregate_id: FeesTypeId,
}

// -- Wave 114 -- FeesMaster (Phase 7 finance_event_stub! for
// FeesMasterCreated deleted and replaced with a full struct below
// -- see aggregate.rs:FeesMaster).

// FM I-2: unique per (school, name, group). Both
// FeesMasterCreated (carrying the scope-key tuple +
// amount_minor + currency + due_date downstream) and
// FeesMasterRetired (tombstone preserving the scope-key tuple
// for legal-record retention) are emitted by
// create_fees_master / retire_fees_master service functions.

finance_event_stub! {
    /// Emitted when a `FeesMaster` is assigned to a class (or class+section).
    pub struct FeesAssignedToClass;
    event_type: "finance.fees_master.assigned_to_class",
    aggregate_type: "fees_master",
    aggregate_id: FeesMasterId,
}

finance_event_stub! {
    /// Emitted when a `FeesAssign` is created for a student.
    pub struct FeesAssignedToStudent;
    event_type: "finance.fees_assign.assigned_to_student",
    aggregate_type: "fees_assign",
    aggregate_id: FeesAssignId,
}

// -- Wave 118 -- FeesAssignDiscount (Phase 7 finance_event_stub! for
// FeesDiscountAssigned deleted and replaced with a full struct
// below -- see aggregate.rs:RealFeesAssignDiscount).
//
// FAD I-3: timestamp recorded. Both FeesAssignDiscountCreated
// (carrying the scope-key + applied_amount_minor +
// unapplied_amount_minor + currency + note + the event-level
// occurred_at timestamp downstream) and FeesAssignDiscountRetired
// (tombstone preserving the scope-key for legal-record retention
// + the event-level occurred_at timestamp) are emitted by
// create_fees_assign_discount / retire_fees_assign_discount
// service functions.

finance_event_stub! {
    /// Emitted when a `FeesInstallment` is created for a `FeesMaster`.
    pub struct FeesInstallmentCreated;
    event_type: "finance.fees_installment.created",
    aggregate_type: "fees_installment",
    aggregate_id: FeesInstallmentId,
}

finance_event_stub! {
    /// Emitted when a `FeesPayment` is reversed (e.g. duplicate / wrong
    /// payer / bank chargeback).
    pub struct PaymentReversed;
    event_type: "finance.fees_payment.reversed",
    aggregate_type: "fees_payment",
    aggregate_id: FeesPaymentId,
}

finance_event_stub! {
    /// Emitted when an `FmFeesInvoice` is generated (FM invoice scheme).
    pub struct FmFeesInvoiceGenerated;
    event_type: "finance.fm_fees_invoice.generated",
    aggregate_type: "fm_fees_invoice",
    aggregate_id: FmFeesInvoiceId,
}

// -- Wave 111 -- DirectFeesInstallment (Wave 90 lesson: pre-existing
// finance_event_stub! for DirectFeesInstallmentCreated was deleted and
// replaced with a full struct below -- see aggregate.rs:DirectFeesInstallment).
//
// DFI I-2: amount >= 0. Both DirectFeesInstallmentCreated (carrying
// the amount_minor + due_date + name downstream) and
// DirectFeesInstallmentRetired (tombstone preserving the
// identity for legal-record retention) are emitted by
// create_direct_fees_installment / retire_direct_fees_installment
// service functions.

finance_event_stub! {
    /// Emitted when a student's balance is carried forward between
    /// academic years.
    pub struct FeesCarriedForward;
    event_type: "finance.fees_carry_forward.carried",
    aggregate_type: "fees_carry_forward",
    aggregate_id: FeesCarryForwardId,
}

finance_event_stub! {
    /// Emitted when a user login is blocked due to overdue fees.
    /// `rbac` subscribes to enforce the block at the auth port.
    pub struct DueFeesLoginPrevented;
    event_type: "finance.due_fees_login_prevent.prevented",
    aggregate_type: "due_fees_login_prevent",
    aggregate_id: DueFeesLoginPreventId,
}

finance_event_stub! {
    /// Emitted when a `BankAccount` is opened.
    pub struct BankAccountOpened;
    event_type: "finance.bank_account.opened",
    aggregate_type: "bank_account",
    aggregate_id: BankAccountId,
}

finance_event_stub! {
    /// Emitted when a `BankPaymentSlip` is generated.
    pub struct BankPaymentSlipGenerated;
    event_type: "finance.bank_payment_slip.generated",
    aggregate_type: "bank_payment_slip",
    aggregate_id: BankPaymentSlipId,
}

finance_event_stub! {
    /// Emitted when a `BankPaymentSlip` is approved (the bank
    /// confirmed the deposit).
    pub struct BankPaymentApproved;
    event_type: "finance.bank_payment_slip.approved",
    aggregate_type: "bank_payment_slip",
    aggregate_id: BankPaymentSlipId,
}

finance_event_stub! {
    /// Emitted when an `Income` row is recorded.
    pub struct IncomeRecorded;
    event_type: "finance.income.recorded",
    aggregate_type: "income",
    aggregate_id: IncomeId,
}

finance_event_stub! {
    /// Emitted when a `PayrollGenerate` (HR-side payroll run) is
    /// generated; finance-side consumes it to record the `Expense`.
    pub struct PayrollGenerated;
    event_type: "finance.payroll_generate.generated",
    aggregate_type: "payroll_generate",
    aggregate_id: PayrollGenerateId,
}

// =============================================================================
// Spec'd child-entity event stubs
// (Phase 7 Workstreams D-M; ids added in commit d82cd22, structs in 429f74f).
// Each stub carries only `event_id`, `school_id`, `aggregate_id`,
// `correlation_id`, and `occurred_at`. Real payload fields land with the
// workstream that fills in the corresponding aggregate.
// =============================================================================

/// Emitted when a `FeesInstallmentAssignDiscount` child entity is added.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesInstallmentAssignDiscountAdded {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub aggregate_id: FeesInstallmentAssignDiscountId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FeesInstallmentAssignDiscountAdded {
    pub fn new(
        event_id: EventId,
        school_id: SchoolId,
        aggregate_id: FeesInstallmentAssignDiscountId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            event_id,
            school_id,
            aggregate_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FeesInstallmentAssignDiscountAdded {
    const EVENT_TYPE: &'static str = "finance.fees_installment_assign_discount.added";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fees_installment_assign_discount";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.aggregate_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a new `RealDirectFeesInstallmentAssignChild` row is
/// appended. Per v3 Part 2 F12 + checklist § DFIAC: DFIAC I-1
/// (append-only) is enforced at the API surface by NOT emitting an
/// `Updated` event for this aggregate. The only two transitions are
/// create (always present) and retire (soft-tombstone, never a
/// modification of the original record). DFIAC I-2 (timestamps
/// monotonic) is enforced in the aggregate via the `fresh` and
/// `retire` mutators and pinned here by including both timestamps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectFeesInstallmentAssignChildAdded {
    pub direct_fees_installment_assign_child_id: DirectFeesInstallmentAssignChildId,
    pub direct_fees_installment_assign_id: DirectFeesInstallmentAssignId,
    pub amount_minor: i64,
    pub created_at: Timestamp,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DirectFeesInstallmentAssignChildAdded {
    pub fn new(
        direct_fees_installment_assign_child_id: DirectFeesInstallmentAssignChildId,
        direct_fees_installment_assign_id: DirectFeesInstallmentAssignId,
        amount_minor: i64,
        created_at: Timestamp,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            direct_fees_installment_assign_child_id,
            direct_fees_installment_assign_id,
            amount_minor,
            created_at,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DirectFeesInstallmentAssignChildAdded {
    const EVENT_TYPE: &'static str = "finance.direct_fees_installment_assign_child.added";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "direct_fees_installment_assign_child";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.direct_fees_installment_assign_child_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.direct_fees_installment_assign_child_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealDirectFeesInstallmentAssignChild` row is
/// retired (soft-deleted via `RealDirectFeesInstallmentAssignChild::retire`).
/// Note: this does NOT violate DFIAC I-1 (append-only) — the tombstone
/// preserves the original record in the audit footer + the `Retired`
/// active_status. DFIAC I-2 (timestamps monotonic) is preserved
/// because `retire` always advances `updated_at` strictly past
/// `created_at`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectFeesInstallmentAssignChildRetired {
    pub direct_fees_installment_assign_child_id: DirectFeesInstallmentAssignChildId,
    pub retired_at: Timestamp,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DirectFeesInstallmentAssignChildRetired {
    pub fn new(
        direct_fees_installment_assign_child_id: DirectFeesInstallmentAssignChildId,
        retired_at: Timestamp,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            direct_fees_installment_assign_child_id,
            retired_at,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DirectFeesInstallmentAssignChildRetired {
    const EVENT_TYPE: &'static str = "finance.direct_fees_installment_assign_child.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "direct_fees_installment_assign_child";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.direct_fees_installment_assign_child_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.direct_fees_installment_assign_child_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when an `FmFeesInvoiceLineNote` child entity is added.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FmFeesInvoiceLineNoteAdded {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub aggregate_id: FmFeesInvoiceLineNoteId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FmFeesInvoiceLineNoteAdded {
    pub fn new(
        event_id: EventId,
        school_id: SchoolId,
        aggregate_id: FmFeesInvoiceLineNoteId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            event_id,
            school_id,
            aggregate_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FmFeesInvoiceLineNoteAdded {
    const EVENT_TYPE: &'static str = "finance.fm_fees_invoice_line_note.added";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fm_fees_invoice_line_note";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.aggregate_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a new `RealFmFeesTransactionLineNote` row is appended.
/// Per v3 Part 2 F32 + checklist § FmFeesTransactionLineNote: FFTLN
/// I-1 (note 1..=2000 chars after trim, validated in
/// `RealFmFeesTransactionLineNote::fresh`) and FFTLN I-2 (append-only,
/// enforced at the API surface by NOT emitting an `Updated` event for
/// this aggregate). The only two transitions are create (always
/// present) and retire (soft-tombstone, never a modification of the
/// original record).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FmFeesTransactionLineNoteAdded {
    pub fm_fees_transaction_line_note_id: FmFeesTransactionLineNoteId,
    pub fm_fees_transaction_id: FmFeesTransactionId,
    pub note: String,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FmFeesTransactionLineNoteAdded {
    pub fn new(
        fm_fees_transaction_line_note_id: FmFeesTransactionLineNoteId,
        fm_fees_transaction_id: FmFeesTransactionId,
        note: String,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fm_fees_transaction_line_note_id,
            fm_fees_transaction_id,
            note,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FmFeesTransactionLineNoteAdded {
    const EVENT_TYPE: &'static str = "finance.fm_fees_transaction_line_note.added";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fm_fees_transaction_line_note";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fm_fees_transaction_line_note_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fm_fees_transaction_line_note_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealFmFeesTransactionLineNote` row is retired
/// (soft-deleted via `RealFmFeesTransactionLineNote::retire`). Note:
/// this does NOT violate FFTLN I-2 (append-only) — the tombstone
/// preserves the original record in the audit footer + the `Retired`
/// active_status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FmFeesTransactionLineNoteRetired {
    pub fm_fees_transaction_line_note_id: FmFeesTransactionLineNoteId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FmFeesTransactionLineNoteRetired {
    pub fn new(
        fm_fees_transaction_line_note_id: FmFeesTransactionLineNoteId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fm_fees_transaction_line_note_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FmFeesTransactionLineNoteRetired {
    const EVENT_TYPE: &'static str = "finance.fm_fees_transaction_line_note.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fm_fees_transaction_line_note";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fm_fees_transaction_line_note_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fm_fees_transaction_line_note_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// FmFeesTransactionChild events (Wave 77 — RealFmFeesTransactionChild
// headline events)
// =============================================================================
//
// Per v3 Part 2 F33 + checklist § FmFeesTransactionChild: 2 invariants:
//   - FFTC I-1: amount_minor ≥ 0 (validated in
//               `RealFmFeesTransactionChild::fresh` and
//               `update_metadata`).
//   - FFTC I-2: parent reference valid (cross-school consistency
//               enforced in `fresh`; parent existence is the
//               dispatcher's concern).

/// Emitted when a new `RealFmFeesTransactionChild` row is appended.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FmFeesTransactionChildCreated {
    pub fm_fees_transaction_child_id: FmFeesTransactionChildId,
    pub fm_fees_transaction_id: FmFeesTransactionId,
    pub amount_minor: i64,
    pub description: Option<String>,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FmFeesTransactionChildCreated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        fm_fees_transaction_child_id: FmFeesTransactionChildId,
        fm_fees_transaction_id: FmFeesTransactionId,
        amount_minor: i64,
        description: Option<String>,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fm_fees_transaction_child_id,
            fm_fees_transaction_id,
            amount_minor,
            description,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FmFeesTransactionChildCreated {
    const EVENT_TYPE: &'static str = "finance.fm_fees_transaction_child.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fm_fees_transaction_child";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fm_fees_transaction_child_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fm_fees_transaction_child_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealFmFeesTransactionChild`'s amount / description
/// is updated via `RealFmFeesTransactionChild::update_metadata`. The
/// parent `fm_fees_transaction_id` is immutable on update (FFTC I-2;
/// the spec forbids re-parenting).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FmFeesTransactionChildUpdated {
    pub fm_fees_transaction_child_id: FmFeesTransactionChildId,
    pub amount_minor: i64,
    pub description: Option<String>,
    pub updated_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FmFeesTransactionChildUpdated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        fm_fees_transaction_child_id: FmFeesTransactionChildId,
        amount_minor: i64,
        description: Option<String>,
        updated_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fm_fees_transaction_child_id,
            amount_minor,
            description,
            updated_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FmFeesTransactionChildUpdated {
    const EVENT_TYPE: &'static str = "finance.fm_fees_transaction_child.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fm_fees_transaction_child";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fm_fees_transaction_child_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fm_fees_transaction_child_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealFmFeesTransactionChild` row is retired
/// (soft-deleted via `RealFmFeesTransactionChild::retire`). The
/// original amount + parent reference are preserved in the audit
/// footer for legal-record retention.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FmFeesTransactionChildRetired {
    pub fm_fees_transaction_child_id: FmFeesTransactionChildId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FmFeesTransactionChildRetired {
    pub fn new(
        fm_fees_transaction_child_id: FmFeesTransactionChildId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fm_fees_transaction_child_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FmFeesTransactionChildRetired {
    const EVENT_TYPE: &'static str = "finance.fm_fees_transaction_child.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fm_fees_transaction_child";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fm_fees_transaction_child_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fm_fees_transaction_child_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `BankStatementAttachment` child entity is attached.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BankStatementAttachmentAttached {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub aggregate_id: BankStatementAttachmentId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BankStatementAttachmentAttached {
    pub fn new(
        event_id: EventId,
        school_id: SchoolId,
        aggregate_id: BankStatementAttachmentId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            event_id,
            school_id,
            aggregate_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BankStatementAttachmentAttached {
    const EVENT_TYPE: &'static str = "finance.bank_statement_attachment.attached";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "bank_statement_attachment";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.aggregate_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// BankStatementAttachment events — Wave 84 (per-aggregate wave pattern from
// Waves 65–83)
// =============================================================================
//
// Per v3 Part 2 F47 + checklist § BankStatementAttachment: 2 invariants:
//   - BSA I-1: attachment ref valid — the file_reference Uuid must
//             point to an existing file in the file storage port
//             (dispatcher responsibility, not aggregate).
//   - BSA I-2: orphan after BankStatement delete — the
//             bank_statement_id reference is preserved in the audit
//             footer even after retire; cascade-delete handled by
//             the dispatcher.
// Append-only event family — parallel to Wave 81
// PayrollPaymentApproval events + Wave 83 BankPaymentSlipAudit
// events. Since the BankStatementAttachment struct (entities.rs)
// does NOT have its own id field (parent bank_statement_id is
// de-facto identity + file_reference Uuid serves as a secondary
// identifier), the events use bank_statement_id.as_uuid() as the
// aggregate_id in their DomainEvent impl.
//
// Two headline events: Created (initial attach), Retired (tombstone).

/// Emitted when a new `BankStatementAttachment` row is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BankStatementAttachmentCreated {
    pub bank_statement_id: BankStatementId,
    pub file_reference: Uuid,
    pub uploaded_at: Timestamp,
    pub uploaded_by: UserId,
    pub description: Option<String>,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BankStatementAttachmentCreated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bank_statement_id: BankStatementId,
        file_reference: Uuid,
        uploaded_at: Timestamp,
        uploaded_by: UserId,
        description: Option<String>,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            bank_statement_id,
            file_reference,
            uploaded_at,
            uploaded_by,
            description,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BankStatementAttachmentCreated {
    const EVENT_TYPE: &'static str = "finance.bank_statement_attachment.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "bank_statement_attachment";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        // The BankStatementAttachment struct does not have its own
        // id field; bank_statement_id serves as the de-facto
        // aggregate identifier.
        self.bank_statement_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.bank_statement_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `BankStatementAttachment` row is retired
/// (soft-deleted via `BankStatementAttachment::retire`). The
/// original `bank_statement_id` + `file_reference` + `uploaded_at`
/// + `uploaded_by` + `description` are preserved in the audit
/// footer for legal-record retention. BSA I-1 (attachment ref
/// valid) is upheld because retire does NOT mutate the
/// file_reference; BSA I-2 (orphan after BankStatement delete) is
/// upheld because the `bank_statement_id` reference is preserved
/// in the audit footer even after retire.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BankStatementAttachmentRetired {
    pub bank_statement_id: BankStatementId,
    pub file_reference: Uuid,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BankStatementAttachmentRetired {
    pub fn new(
        bank_statement_id: BankStatementId,
        file_reference: Uuid,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            bank_statement_id,
            file_reference,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BankStatementAttachmentRetired {
    const EVENT_TYPE: &'static str = "finance.bank_statement_attachment.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "bank_statement_attachment";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.bank_statement_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.bank_statement_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `PayrollPaymentApproval` child entity is recorded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayrollPaymentApprovalRecorded {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub aggregate_id: PayrollPaymentApprovalId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl PayrollPaymentApprovalRecorded {
    pub fn new(
        event_id: EventId,
        school_id: SchoolId,
        aggregate_id: PayrollPaymentApprovalId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            event_id,
            school_id,
            aggregate_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for PayrollPaymentApprovalRecorded {
    const EVENT_TYPE: &'static str = "finance.payroll_payment_approval.recorded";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "payroll_payment_approval";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.aggregate_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `BankPaymentSlipAudit` child entity is recorded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BankPaymentSlipAuditRecorded {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub aggregate_id: BankPaymentSlipAuditId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BankPaymentSlipAuditRecorded {
    pub fn new(
        event_id: EventId,
        school_id: SchoolId,
        aggregate_id: BankPaymentSlipAuditId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            event_id,
            school_id,
            aggregate_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BankPaymentSlipAuditRecorded {
    const EVENT_TYPE: &'static str = "finance.bank_payment_slip_audit.recorded";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "bank_payment_slip_audit";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.aggregate_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when an `ExpenseApproval` child entity is recorded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpenseApprovalRecorded {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub aggregate_id: ExpenseApprovalId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ExpenseApprovalRecorded {
    pub fn new(
        event_id: EventId,
        school_id: SchoolId,
        aggregate_id: ExpenseApprovalId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            event_id,
            school_id,
            aggregate_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ExpenseApprovalRecorded {
    const EVENT_TYPE: &'static str = "finance.expense_approval.recorded";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "expense_approval";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.aggregate_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when an `IncomeApproval` child entity is recorded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncomeApprovalRecorded {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub aggregate_id: IncomeApprovalId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl IncomeApprovalRecorded {
    pub fn new(
        event_id: EventId,
        school_id: SchoolId,
        aggregate_id: IncomeApprovalId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            event_id,
            school_id,
            aggregate_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for IncomeApprovalRecorded {
    const EVENT_TYPE: &'static str = "finance.income_approval.recorded";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "income_approval";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.aggregate_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a new `WalletTransactionApproval` child row is created
/// (initial pending state). Per v3 Part 2 + checklist § WTA: WTA I-1
/// (state machine pending → approved/rejected) is enforced in the
/// aggregate via `approve()` / `reject()`; WTA I-2 (timestamps +
/// reason) is enforced via the `approved_at` / `rejected_at` /
/// `reject_note` fields on the aggregate and the corresponding event
/// payloads.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalletTransactionApprovalCreated {
    pub wallet_transaction_approval_id: WalletTransactionApprovalId,
    pub wallet_transaction_id: WalletTransactionId,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl WalletTransactionApprovalCreated {
    pub fn new(
        wallet_transaction_approval_id: WalletTransactionApprovalId,
        wallet_transaction_id: WalletTransactionId,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            wallet_transaction_approval_id,
            wallet_transaction_id,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for WalletTransactionApprovalCreated {
    const EVENT_TYPE: &'static str = "finance.wallet_transaction_approval.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "wallet_transaction_approval";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.wallet_transaction_approval_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.wallet_transaction_approval_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `WalletTransactionApproval` row transitions from
/// `pending` to `approved` via `WalletTransactionApproval::approve()`.
/// Per WTA I-1, this is the first terminal state of the state machine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalletTransactionApprovalApproved {
    pub wallet_transaction_approval_id: WalletTransactionApprovalId,
    pub wallet_transaction_id: WalletTransactionId,
    pub approver_id: UserId,
    pub approved_at: Timestamp,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl WalletTransactionApprovalApproved {
    pub fn new(
        wallet_transaction_approval_id: WalletTransactionApprovalId,
        wallet_transaction_id: WalletTransactionId,
        approver_id: UserId,
        approved_at: Timestamp,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            wallet_transaction_approval_id,
            wallet_transaction_id,
            approver_id,
            approved_at,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for WalletTransactionApprovalApproved {
    const EVENT_TYPE: &'static str = "finance.wallet_transaction_approval.approved";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "wallet_transaction_approval";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.wallet_transaction_approval_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.wallet_transaction_approval_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `WalletTransactionApproval` row transitions from
/// `pending` to `rejected` via `WalletTransactionApproval::reject()`.
/// Per WTA I-1, this is the second terminal state of the state machine.
/// Per WTA I-2, the `reject_note` (reason) is required and recorded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalletTransactionApprovalRejected {
    pub wallet_transaction_approval_id: WalletTransactionApprovalId,
    pub wallet_transaction_id: WalletTransactionId,
    pub rejecter_id: UserId,
    pub rejected_at: Timestamp,
    pub reject_note: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl WalletTransactionApprovalRejected {
    pub fn new(
        wallet_transaction_approval_id: WalletTransactionApprovalId,
        wallet_transaction_id: WalletTransactionId,
        rejecter_id: UserId,
        rejected_at: Timestamp,
        reject_note: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            wallet_transaction_approval_id,
            wallet_transaction_id,
            rejecter_id,
            rejected_at,
            reject_note,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for WalletTransactionApprovalRejected {
    const EVENT_TYPE: &'static str = "finance.wallet_transaction_approval.rejected";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "wallet_transaction_approval";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.wallet_transaction_approval_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.wallet_transaction_approval_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// FeesCarryForwardSetting events — Wave 78 (per-aggregate wave pattern
// from Waves 65–77)
// =============================================================================
//
// Per v3 Part 2 F34 + checklist § FeesCarryForwardSetting: 2
// invariants:
//   - FCFA I-1: per-school config (the typed id carries the
//              school_id; uniqueness is a dispatcher concern).
//   - FCFA I-2: threshold_minor >= 0.
// Full lifecycle (Created + Updated + Retired); the setting is
// reference data so updates are expected (not append-only).

/// Emitted when a new `RealFeesCarryForwardSetting` row is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesCarryForwardSettingCreated {
    pub fees_carry_forward_setting_id: FeesCarryForwardSettingId,
    pub threshold_minor: i64,
    pub enabled: bool,
    pub description: Option<String>,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FeesCarryForwardSettingCreated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        fees_carry_forward_setting_id: FeesCarryForwardSettingId,
        threshold_minor: i64,
        enabled: bool,
        description: Option<String>,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fees_carry_forward_setting_id,
            threshold_minor,
            enabled,
            description,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FeesCarryForwardSettingCreated {
    const EVENT_TYPE: &'static str = "finance.fees_carry_forward_setting.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fees_carry_forward_setting";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fees_carry_forward_setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fees_carry_forward_setting_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealFeesCarryForwardSetting`'s threshold / enabled
/// flag / description is updated via
/// `RealFeesCarryForwardSetting::update_metadata`. The school_id is
/// immutable on update (FCFA I-1 per-school scoping; the typed id
/// carries it so it cannot change).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesCarryForwardSettingUpdated {
    pub fees_carry_forward_setting_id: FeesCarryForwardSettingId,
    pub threshold_minor: i64,
    pub enabled: bool,
    pub description: Option<String>,
    pub updated_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FeesCarryForwardSettingUpdated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        fees_carry_forward_setting_id: FeesCarryForwardSettingId,
        threshold_minor: i64,
        enabled: bool,
        description: Option<String>,
        updated_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fees_carry_forward_setting_id,
            threshold_minor,
            enabled,
            description,
            updated_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FeesCarryForwardSettingUpdated {
    const EVENT_TYPE: &'static str = "finance.fees_carry_forward_setting.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fees_carry_forward_setting";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fees_carry_forward_setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fees_carry_forward_setting_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealFeesCarryForwardSetting` row is retired
/// (soft-deleted via `RealFeesCarryForwardSetting::retire`). The
/// original threshold + enabled flag are preserved in the audit
/// footer for legal-record retention.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesCarryForwardSettingRetired {
    pub fees_carry_forward_setting_id: FeesCarryForwardSettingId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FeesCarryForwardSettingRetired {
    pub fn new(
        fees_carry_forward_setting_id: FeesCarryForwardSettingId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fees_carry_forward_setting_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FeesCarryForwardSettingRetired {
    const EVENT_TYPE: &'static str = "finance.fees_carry_forward_setting.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fees_carry_forward_setting";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fees_carry_forward_setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fees_carry_forward_setting_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// ExpenseApproval events — Wave 79 (per-aggregate wave pattern from
// Waves 65–78)
// =============================================================================
//
// Per v3 Part 2 F20 + checklist § ExpenseApproval: 2 invariants:
//   - EA I-1: state machine pending → approved/rejected (enforced
//             at the type-system level via the ApprovalStatus enum
//             + invalid transition guards; invalid transitions
//             emit DomainError::conflict).
//   - EA I-2: timestamps recorded (every state transition stamps
//             decided_by + decided_at on the aggregate; the reject
//             path also captures an optional reason string).
// Three headline events: Created (when the aggregate enters
// Pending), Approved (Pending → Approved transition), and Rejected
// (Pending → Rejected transition with optional reason).

/// Emitted when a new `RealExpenseApproval` row is created in the
/// Pending state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpenseApprovalCreated {
    pub expense_approval_id: ExpenseApprovalId,
    pub expense_id: ExpenseId,
    pub requested_by: UserId,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ExpenseApprovalCreated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        expense_approval_id: ExpenseApprovalId,
        expense_id: ExpenseId,
        requested_by: UserId,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            expense_approval_id,
            expense_id,
            requested_by,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ExpenseApprovalCreated {
    const EVENT_TYPE: &'static str = "finance.expense_approval.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "expense_approval";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.expense_approval_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.expense_approval_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealExpenseApproval` transitions from Pending to
/// Approved (EA I-1). Stamps `decided_by` + `decided_at` on the
/// aggregate (EA I-2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpenseApprovalApproved {
    pub expense_approval_id: ExpenseApprovalId,
    pub decided_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ExpenseApprovalApproved {
    pub fn new(
        expense_approval_id: ExpenseApprovalId,
        decided_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            expense_approval_id,
            decided_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ExpenseApprovalApproved {
    const EVENT_TYPE: &'static str = "finance.expense_approval.approved";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "expense_approval";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.expense_approval_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.expense_approval_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealExpenseApproval` transitions from Pending to
/// Rejected (EA I-1). Stamps `decided_by` + `decided_at` +
/// `reject_reason` on the aggregate (EA I-2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpenseApprovalRejected {
    pub expense_approval_id: ExpenseApprovalId,
    pub decided_by: UserId,
    pub reject_reason: Option<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ExpenseApprovalRejected {
    pub fn new(
        expense_approval_id: ExpenseApprovalId,
        decided_by: UserId,
        reject_reason: Option<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            expense_approval_id,
            decided_by,
            reject_reason,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ExpenseApprovalRejected {
    const EVENT_TYPE: &'static str = "finance.expense_approval.rejected";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "expense_approval";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.expense_approval_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.expense_approval_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// IncomeApproval events — Wave 80 (per-aggregate wave pattern from
// Waves 65–79)
// =============================================================================
//
// Per v3 Part 2 F28 + checklist § IncomeApproval: 2 invariants:
//   - IA I-1: state machine pending → approved/rejected (enforced
//             at the type-system level via the ApprovalStatus enum
//             + invalid transition guards; invalid transitions
//             emit DomainError::conflict).
//   - IA I-2: timestamps recorded (every state transition stamps
//             decided_by + decided_at on the aggregate; the reject
//             path also captures an optional reason string).
// Structurally identical to the Wave 79 ExpenseApproval event
// family, with the parent reference renamed from expense_id to
// income_id. Three headline events: Created (Pending entry),
// Approved (Pending → Approved), Rejected (Pending → Rejected with
// optional reason).

/// Emitted when a new `RealIncomeApproval` row is created in the
/// Pending state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncomeApprovalCreated {
    pub income_approval_id: IncomeApprovalId,
    pub income_id: IncomeId,
    pub requested_by: UserId,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl IncomeApprovalCreated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        income_approval_id: IncomeApprovalId,
        income_id: IncomeId,
        requested_by: UserId,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            income_approval_id,
            income_id,
            requested_by,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for IncomeApprovalCreated {
    const EVENT_TYPE: &'static str = "finance.income_approval.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "income_approval";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.income_approval_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.income_approval_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealIncomeApproval` transitions from Pending to
/// Approved (IA I-1). Stamps `decided_by` + `decided_at` on the
/// aggregate (IA I-2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncomeApprovalApproved {
    pub income_approval_id: IncomeApprovalId,
    pub decided_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl IncomeApprovalApproved {
    pub fn new(
        income_approval_id: IncomeApprovalId,
        decided_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            income_approval_id,
            decided_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for IncomeApprovalApproved {
    const EVENT_TYPE: &'static str = "finance.income_approval.approved";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "income_approval";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.income_approval_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.income_approval_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealIncomeApproval` transitions from Pending to
/// Rejected (IA I-1). Stamps `decided_by` + `decided_at` +
/// `reject_reason` on the aggregate (IA I-2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncomeApprovalRejected {
    pub income_approval_id: IncomeApprovalId,
    pub decided_by: UserId,
    pub reject_reason: Option<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl IncomeApprovalRejected {
    pub fn new(
        income_approval_id: IncomeApprovalId,
        decided_by: UserId,
        reject_reason: Option<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            income_approval_id,
            decided_by,
            reject_reason,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for IncomeApprovalRejected {
    const EVENT_TYPE: &'static str = "finance.income_approval.rejected";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "income_approval";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.income_approval_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.income_approval_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// PayrollPaymentApproval events — Wave 81 (per-aggregate wave pattern
// from Waves 65–80)
// =============================================================================
//
// Per v3 Part 2 F44 + checklist § PayrollPaymentApproval: 2 invariants:
//   - PPA I-1: state machine pending → approved/rejected (enforced
//             at the type-system level via the existing approved_at
//             + rejected_at timestamp fields — derived state, no
//             explicit ApprovalStatus enum field; invalid
//             transitions emit DomainError::conflict).
//   - PPA I-2: timestamps recorded (every state transition stamps
//             approver_id + approved_at on the aggregate; reject
//             also captures rejecter_id + rejected_at + reason).
// Structurally identical to the Wave 79 ExpenseApproval and
// Wave 80 IncomeApproval event families, but since the
// PayrollPaymentApproval struct (entities.rs) does NOT have its own
// id field (parent_id is de-facto identity), the events use
// payroll_payment_id as the aggregate identifier.
//
// Three headline events: Created (Pending entry), Approved (Pending
// → Approved), Rejected (Pending → Rejected with optional reason).

/// Emitted when a new `PayrollPaymentApproval` row is created in
/// the Pending state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayrollPaymentApprovalCreated {
    pub payroll_payment_id: PayrollPaymentId,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl PayrollPaymentApprovalCreated {
    pub fn new(
        payroll_payment_id: PayrollPaymentId,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            payroll_payment_id,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for PayrollPaymentApprovalCreated {
    const EVENT_TYPE: &'static str = "finance.payroll_payment_approval.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "payroll_payment_approval";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        // The PayrollPaymentApproval struct does not have its own
        // id field; payroll_payment_id serves as the de-facto
        // aggregate identifier.
        self.payroll_payment_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.payroll_payment_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `PayrollPaymentApproval` transitions from Pending
/// to Approved (PPA I-1). Stamps `approver_id` + `approved_at` on
/// the aggregate (PPA I-2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayrollPaymentApprovalApproved {
    pub payroll_payment_id: PayrollPaymentId,
    pub approver_id: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl PayrollPaymentApprovalApproved {
    pub fn new(
        payroll_payment_id: PayrollPaymentId,
        approver_id: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            payroll_payment_id,
            approver_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for PayrollPaymentApprovalApproved {
    const EVENT_TYPE: &'static str = "finance.payroll_payment_approval.approved";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "payroll_payment_approval";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.payroll_payment_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.payroll_payment_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `PayrollPaymentApproval` transitions from Pending
/// to Rejected (PPA I-1). Stamps `rejecter_id` + `rejected_at` +
/// `rejection_reason` on the aggregate (PPA I-2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayrollPaymentApprovalRejected {
    pub payroll_payment_id: PayrollPaymentId,
    pub rejecter_id: UserId,
    pub rejection_reason: Option<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl PayrollPaymentApprovalRejected {
    pub fn new(
        payroll_payment_id: PayrollPaymentId,
        rejecter_id: UserId,
        rejection_reason: Option<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            payroll_payment_id,
            rejecter_id,
            rejection_reason,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for PayrollPaymentApprovalRejected {
    const EVENT_TYPE: &'static str = "finance.payroll_payment_approval.rejected";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "payroll_payment_approval";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.payroll_payment_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.payroll_payment_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// SalaryTemplate events — Wave 82 (per-aggregate wave pattern from
// Waves 65–81)
// =============================================================================
//
// Per v3 Part 2 F44 + checklist § SalaryTemplate: 2 invariants:
//   - ST I-1: gross_salary composition (gross_salary_minor >= 0
//             pinned at construction; service-side composition
//             via SalaryTemplateService::create_template).
//   - ST I-2: net_salary == gross - total_deduction
//             (net_salary_minor >= 0 lower bound pinned at
//             construction; service-side composition via
//             SalaryTemplateService::apply_template).
// Full lifecycle (Created / Updated / Retired) — reference data
// with corrections expected, parallel to Wave 74 ChartOfAccount
// and Wave 78 FeesCarryForwardSetting event families.
//
// Note: SalaryTemplateId is re-exported from educore_hr (cross-
// crate dep, parallel to Wave 71 Donor pattern).

/// Emitted when a new `RealSalaryTemplate` row is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SalaryTemplateCreated {
    pub salary_template_id: SalaryTemplateId,
    pub name: String,
    pub currency: Currency,
    pub gross_salary_minor: i64,
    pub net_salary_minor: i64,
    pub description: Option<String>,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SalaryTemplateCreated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        salary_template_id: SalaryTemplateId,
        name: String,
        currency: Currency,
        gross_salary_minor: i64,
        net_salary_minor: i64,
        description: Option<String>,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            salary_template_id,
            name,
            currency,
            gross_salary_minor,
            net_salary_minor,
            description,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SalaryTemplateCreated {
    const EVENT_TYPE: &'static str = "finance.salary_template.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "salary_template";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.salary_template_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.salary_template_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealSalaryTemplate`'s metadata is updated via
/// `RealSalaryTemplate::update_metadata`. Re-validates ST I-1 +
/// ST I-2 lower bound at the aggregate surface.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SalaryTemplateUpdated {
    pub salary_template_id: SalaryTemplateId,
    pub name: String,
    pub currency: Currency,
    pub gross_salary_minor: i64,
    pub net_salary_minor: i64,
    pub description: Option<String>,
    pub updated_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SalaryTemplateUpdated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        salary_template_id: SalaryTemplateId,
        name: String,
        currency: Currency,
        gross_salary_minor: i64,
        net_salary_minor: i64,
        description: Option<String>,
        updated_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            salary_template_id,
            name,
            currency,
            gross_salary_minor,
            net_salary_minor,
            description,
            updated_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SalaryTemplateUpdated {
    const EVENT_TYPE: &'static str = "finance.salary_template.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "salary_template";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.salary_template_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.salary_template_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealSalaryTemplate` row is retired (soft-deleted
/// via `RealSalaryTemplate::retire`). The original gross_salary_minor
/// + net_salary_minor are preserved in the audit footer for legal-
/// record retention.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SalaryTemplateRetired {
    pub salary_template_id: SalaryTemplateId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SalaryTemplateRetired {
    pub fn new(
        salary_template_id: SalaryTemplateId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            salary_template_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SalaryTemplateRetired {
    const EVENT_TYPE: &'static str = "finance.salary_template.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "salary_template";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.salary_template_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.salary_template_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// BankPaymentSlipAudit events — Wave 83 (per-aggregate wave pattern from
// Waves 65–82)
// =============================================================================
//
// Per v3 Part 2 F37 + checklist § BankPaymentSlipAudit: 2 invariants:
//   - BPA I-1: append-only log (enforced at the API surface by
//             intentionally exposing no `update_*` mutator on the
//             aggregate; NO `Updated` event variant exists, which
//             is the type-system-level enforcement).
//   - BPA I-2: timestamps recorded (every audit row carries
//             created_at + created_by + updated_at + updated_by in
//             the 10-field audit footer; the `recorded_at` payload
//             field carries the slip-recording semantic timestamp).
// Append-only event family — parallel to Wave 70
// FeesCarryForwardLog events (Created + Retired; no Updated).
// Two headline events: Created (initial append), Retired
// (tombstone — preserves original slip + bank + amount).

/// Emitted when a new `RealBankPaymentSlipAudit` row is appended.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BankPaymentSlipAuditCreated {
    pub bank_payment_slip_audit_id: BankPaymentSlipAuditId,
    pub bank_payment_slip_id: BankPaymentSlipId,
    pub bank_account_id: BankAccountId,
    pub amount_minor: i64,
    pub currency: Currency,
    pub recorded_at: Timestamp,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BankPaymentSlipAuditCreated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bank_payment_slip_audit_id: BankPaymentSlipAuditId,
        bank_payment_slip_id: BankPaymentSlipId,
        bank_account_id: BankAccountId,
        amount_minor: i64,
        currency: Currency,
        recorded_at: Timestamp,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            bank_payment_slip_audit_id,
            bank_payment_slip_id,
            bank_account_id,
            amount_minor,
            currency,
            recorded_at,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BankPaymentSlipAuditCreated {
    const EVENT_TYPE: &'static str = "finance.bank_payment_slip_audit.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "bank_payment_slip_audit";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.bank_payment_slip_audit_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.bank_payment_slip_audit_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealBankPaymentSlipAudit` row is retired
/// (soft-deleted via `RealBankPaymentSlipAudit::retire`). The
/// original `bank_payment_slip_id` + `bank_account_id` +
/// `amount_minor` + `currency` + `recorded_at` are preserved in
/// the audit footer for legal-record retention. BPA I-1
/// (append-only) is upheld because retire is a tombstone, NOT a
/// content edit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BankPaymentSlipAuditRetired {
    pub bank_payment_slip_audit_id: BankPaymentSlipAuditId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BankPaymentSlipAuditRetired {
    pub fn new(
        bank_payment_slip_audit_id: BankPaymentSlipAuditId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            bank_payment_slip_audit_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BankPaymentSlipAuditRetired {
    const EVENT_TYPE: &'static str = "finance.bank_payment_slip_audit.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "bank_payment_slip_audit";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.bank_payment_slip_audit_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.bank_payment_slip_audit_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// BankStatement events — Wave 85 (per-aggregate wave pattern from
// Waves 65–84)
// =============================================================================
//
// Per v3 Part 2 F48 + checklist § BankStatement: 4 invariants:
//   - BS I-1: amount >= 0 (validated at construction + on update).
//   - BS I-2: type ∈ {income, expense} (enforced at type-system
//             level via the StatementType enum; Income | Expense
//             only — no invalid variants).
//   - BS I-3: after_balance matches running balance (the aggregate
//             pins balance_after_minor at construction + on update;
//             the cross-statement running balance consistency is
//             the dispatcher's responsibility).
//   - BS I-4: append-only; corrections via reverse. The aggregate
//             intentionally exposes no amount/balance mutator;
//             corrections happen via a new opposite-direction row
//             (the `Reversed` event marks the original as
//             corrected-by-reverse-row, NOT a content mutation).
// Full lifecycle event family — 4 events: Created (initial append),
// Updated (metadata correction only — description; amount/balance
// are immutable, BS I-4), Reversed (BS I-4: marks the original
// statement as corrected by a new opposite-direction row), Retired
// (tombstone — preserves original amount/balance/type).

/// Emitted when a new `RealBankStatement` row is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BankStatementCreated {
    pub bank_statement_id: BankStatementId,
    pub bank_account_id: BankAccountId,
    pub statement_type: StatementType,
    pub amount_minor: i64,
    pub balance_after_minor: i64,
    pub currency: Currency,
    pub occurred_at: Timestamp,
    pub description: Option<String>,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at_event: Timestamp,
}

impl BankStatementCreated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bank_statement_id: BankStatementId,
        bank_account_id: BankAccountId,
        statement_type: StatementType,
        amount_minor: i64,
        balance_after_minor: i64,
        currency: Currency,
        occurred_at: Timestamp,
        description: Option<String>,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at_event: Timestamp,
    ) -> Self {
        Self {
            bank_statement_id,
            bank_account_id,
            statement_type,
            amount_minor,
            balance_after_minor,
            currency,
            occurred_at,
            description,
            created_by,
            event_id,
            correlation_id,
            occurred_at_event,
        }
    }
}

impl DomainEvent for BankStatementCreated {
    const EVENT_TYPE: &'static str = "finance.bank_statement.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "bank_statement";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.bank_statement_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.bank_statement_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at_event
    }
}

/// Emitted when a `RealBankStatement`'s metadata is updated via
/// `RealBankStatement::update_metadata`. Note: only `description`
/// is mutable here; the amount_minor + balance_after_minor +
/// statement_type fields are immutable (BS I-4 append-only).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BankStatementUpdated {
    pub bank_statement_id: BankStatementId,
    pub description: Option<String>,
    pub updated_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BankStatementUpdated {
    pub fn new(
        bank_statement_id: BankStatementId,
        description: Option<String>,
        updated_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            bank_statement_id,
            description,
            updated_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BankStatementUpdated {
    const EVENT_TYPE: &'static str = "finance.bank_statement.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "bank_statement";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.bank_statement_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.bank_statement_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealBankStatement` is marked as corrected via a
/// new opposite-direction row (BS I-4 append-only enforcement).
/// The original amount_minor + balance_after_minor + statement_type
/// are preserved in the audit footer; the correction happens via a
/// separate reverse-direction row (not emitted by this event — the
/// dispatcher is responsible for creating that new row).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BankStatementReversed {
    pub bank_statement_id: BankStatementId,
    pub reverse_row_id: BankStatementId,
    pub reversed_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BankStatementReversed {
    pub fn new(
        bank_statement_id: BankStatementId,
        reverse_row_id: BankStatementId,
        reversed_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            bank_statement_id,
            reverse_row_id,
            reversed_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BankStatementReversed {
    const EVENT_TYPE: &'static str = "finance.bank_statement.reversed";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "bank_statement";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.bank_statement_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.bank_statement_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealBankStatement` row is retired (soft-deleted
/// via `RealBankStatement::retire`). The original amount + balance
/// + statement_type are preserved in the audit footer for
/// legal-record retention. BS I-4 (append-only) is upheld because
/// retire is a tombstone, NOT a content edit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BankStatementRetired {
    pub bank_statement_id: BankStatementId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BankStatementRetired {
    pub fn new(
        bank_statement_id: BankStatementId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            bank_statement_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BankStatementRetired {
    const EVENT_TYPE: &'static str = "finance.bank_statement.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "bank_statement";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.bank_statement_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.bank_statement_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// FeesDiscount events — Wave 86 (per-aggregate wave pattern from
// Waves 65–85)
// =============================================================================
//
// Per v3 Part 2 F18 + checklist § FeesDiscount: 4 invariants (2 in
// this wave + 2 promoted from [~] partial):
//   - FD I-1: amount >= 0 (promoted from [~] partial to [x]
//             complete via numeric guard in RealFeesDiscount::fresh()).
//   - FD I-2: discount_type valid (promoted from [~] partial to
//             [x] complete via DiscountType enum type-system
//             enforcement in RealFeesDiscount::fresh()).
//   - FD I-3: once-per-master scope. RealFeesDiscount pins
//             `fees_master_id` as a required field; the dispatcher
//             enforces uniqueness on the (fees_master_id, ...) key
//             when creating new discounts.
//   - FD I-4: once-per-year scope. RealFeesDiscount pins
//             `academic_year_id` as a required field; the dispatcher
//             enforces uniqueness on the (academic_year_id, ...)
//             key per discount type.
// Full lifecycle event family — 3 events: Created (initial),
// Updated (metadata correction; scope-key fields NOT mutable
// without retire + create-new), Retired (tombstone — preserves
// scope-key fields for legal-record retention + uniqueness queries).

/// Emitted when a new `RealFeesDiscount` catalogue entry is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesDiscountCreated {
    pub fees_discount_id: FeesDiscountId,
    pub fees_master_id: FeesMasterId,
    pub academic_year_id: AcademicYearId,
    pub name: String,
    pub discount_code: String,
    pub discount_type: DiscountType,
    pub description: Option<String>,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FeesDiscountCreated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        fees_discount_id: FeesDiscountId,
        fees_master_id: FeesMasterId,
        academic_year_id: AcademicYearId,
        name: String,
        discount_code: String,
        discount_type: DiscountType,
        description: Option<String>,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fees_discount_id,
            fees_master_id, // FD I-3
            academic_year_id, // FD I-4
            name,
            discount_code,
            discount_type, // FD I-2
            description,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FeesDiscountCreated {
    const EVENT_TYPE: &'static str = "finance.fees_discount.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fees_discount";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fees_discount_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fees_discount_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealFeesDiscount`'s metadata is updated via
/// `RealFeesDiscount::update_metadata`. Note: scope-key fields
/// (fees_master_id + academic_year_id) are NOT mutable here —
/// FD I-3 + FD I-4 require retire + create-new for scope changes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesDiscountUpdated {
    pub fees_discount_id: FeesDiscountId,
    pub name: String,
    pub discount_code: String,
    pub discount_type: DiscountType,
    pub description: Option<String>,
    pub updated_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FeesDiscountUpdated {
    pub fn new(
        fees_discount_id: FeesDiscountId,
        name: String,
        discount_code: String,
        discount_type: DiscountType,
        description: Option<String>,
        updated_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fees_discount_id,
            name,
            discount_code,
            discount_type,
            description,
            updated_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FeesDiscountUpdated {
    const EVENT_TYPE: &'static str = "finance.fees_discount.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fees_discount";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fees_discount_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fees_discount_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealFeesDiscount` catalogue entry is retired
/// (soft-deleted via `RealFeesDiscount::retire`). The original
/// fees_master_id + academic_year_id + name + discount_code +
/// amount + type are preserved in the audit footer for
/// legal-record retention. FD I-3 + FD I-4 scope-key fields are
/// preserved (the (fees_master_id, academic_year_id) pair remains
/// valid for uniqueness queries even after retire).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesDiscountRetired {
    pub fees_discount_id: FeesDiscountId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FeesDiscountRetired {
    pub fn new(
        fees_discount_id: FeesDiscountId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fees_discount_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FeesDiscountRetired {
    const EVENT_TYPE: &'static str = "finance.fees_discount.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fees_discount";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fees_discount_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fees_discount_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealBankAccount` ledger entry is created via
/// `RealBankAccount::fresh`. Carries the full payload including the
/// immutable fields (BA I-1 account_number + BA I-2
/// opening_balance_minor + BA I-3 account_type + currency).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BankAccountCreated {
    pub bank_account_id: BankAccountId,
    pub account_name: String,
    pub account_number: String, // BA I-1 pinned
    pub account_type: AccountType, // BA I-3 type-pinned
    pub bank_name: String,
    pub ifsc_code: Option<String>,
    pub branch: Option<String>,
    pub opening_balance_minor: i64, // BA I-2 structural
    pub currency: Currency,
    pub description: Option<String>,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BankAccountCreated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bank_account_id: BankAccountId,
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
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            bank_account_id,
            account_name,
            account_number,
            account_type,
            bank_name,
            ifsc_code,
            branch,
            opening_balance_minor,
            currency,
            description,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BankAccountCreated {
    const EVENT_TYPE: &'static str = "finance.bank_account.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "bank_account";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.bank_account_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.bank_account_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealBankAccount` ledger entry's mutable metadata
/// is updated via `RealBankAccount::update_metadata`. Carries only
/// the MUTABLE fields (account_name + bank_name + ifsc_code +
/// branch + description). The immutable fields (BA I-1
/// account_number + BA I-2 opening_balance_minor + BA I-3
/// account_type + currency) are NOT carried here — they are
/// preserved in the audit footer of the aggregate itself.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BankAccountUpdated {
    pub bank_account_id: BankAccountId,
    pub account_name: String,
    pub bank_name: String,
    pub ifsc_code: Option<String>,
    pub branch: Option<String>,
    pub description: Option<String>,
    pub updated_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BankAccountUpdated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bank_account_id: BankAccountId,
        account_name: String,
        bank_name: String,
        ifsc_code: Option<String>,
        branch: Option<String>,
        description: Option<String>,
        updated_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            bank_account_id,
            account_name,
            bank_name,
            ifsc_code,
            branch,
            description,
            updated_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BankAccountUpdated {
    const EVENT_TYPE: &'static str = "finance.bank_account.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "bank_account";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.bank_account_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.bank_account_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealBankAccount` ledger entry is retired
/// (soft-deleted via `RealBankAccount::retire`). The original
/// account_number + opening_balance_minor + account_type +
/// currency are preserved in the audit footer for legal-record
/// retention. BA I-1 (account_number) + BA I-2
/// (opening_balance_minor) + BA I-3 (account_type) + currency are
/// preserved (the (school_id, account_number) pair remains valid
/// for uniqueness queries even after retire).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BankAccountRetired {
    pub bank_account_id: BankAccountId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BankAccountRetired {
    pub fn new(
        bank_account_id: BankAccountId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            bank_account_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BankAccountRetired {
    const EVENT_TYPE: &'static str = "finance.bank_account.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "bank_account";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.bank_account_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.bank_account_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealDirectFeesReminder` is created via
/// `RealDirectFeesReminder::fresh`. Carries the scope-key fields
/// (direct_fees_installment_id + student_id) + the mutable
/// metadata (remind_at + due_date_before_days DFR I-1 + note).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectFeesReminderCreated {
    pub direct_fees_reminder_id: DirectFeesReminderId,
    pub direct_fees_installment_id: DirectFeesInstallmentId,
    pub student_id: educore_academic::StudentId,
    pub remind_at: NaiveDate,
    pub due_date_before_days: i64, // DFR I-1
    pub note: Option<String>,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DirectFeesReminderCreated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        direct_fees_reminder_id: DirectFeesReminderId,
        direct_fees_installment_id: DirectFeesInstallmentId,
        student_id: educore_academic::StudentId,
        remind_at: NaiveDate,
        due_date_before_days: i64,
        note: Option<String>,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            direct_fees_reminder_id,
            direct_fees_installment_id,
            student_id,
            remind_at,
            due_date_before_days,
            note,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DirectFeesReminderCreated {
    const EVENT_TYPE: &'static str = "finance.direct_fees_reminder.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "direct_fees_reminder";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.direct_fees_reminder_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.direct_fees_reminder_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealDirectFeesReminder`'s mutable metadata is
/// updated via `RealDirectFeesReminder::update_metadata`. Carries
/// only the MUTABLE fields (remind_at + due_date_before_days +
/// note). The scope-key fields (direct_fees_installment_id +
/// student_id) are NOT carried here — they are preserved in the
/// audit footer of the aggregate itself.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectFeesReminderUpdated {
    pub direct_fees_reminder_id: DirectFeesReminderId,
    pub remind_at: NaiveDate,
    pub due_date_before_days: i64, // DFR I-1
    pub note: Option<String>,
    pub updated_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DirectFeesReminderUpdated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        direct_fees_reminder_id: DirectFeesReminderId,
        remind_at: NaiveDate,
        due_date_before_days: i64,
        note: Option<String>,
        updated_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            direct_fees_reminder_id,
            remind_at,
            due_date_before_days,
            note,
            updated_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DirectFeesReminderUpdated {
    const EVENT_TYPE: &'static str = "finance.direct_fees_reminder.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "direct_fees_reminder";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.direct_fees_reminder_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.direct_fees_reminder_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealDirectFeesReminder` is retired
/// (soft-deleted via `RealDirectFeesReminder::retire`). The
/// original scope-key fields (direct_fees_installment_id +
/// student_id + remind_at + due_date_before_days) are preserved
/// in the audit footer for legal-record retention.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectFeesReminderRetired {
    pub direct_fees_reminder_id: DirectFeesReminderId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DirectFeesReminderRetired {
    pub fn new(
        direct_fees_reminder_id: DirectFeesReminderId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            direct_fees_reminder_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DirectFeesReminderRetired {
    const EVENT_TYPE: &'static str = "finance.direct_fees_reminder.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "direct_fees_reminder";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.direct_fees_reminder_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.direct_fees_reminder_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealExpenseHead` catalogue entry is created
/// via `RealExpenseHead::fresh`. Carries `name` (EH I-1
/// uniqueness anchor — pinned at construction) + the mutable
/// `description`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpenseHeadCreated {
    pub expense_head_id: ExpenseHeadId,
    pub name: String, // EH I-1 pinned
    pub description: Option<String>,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ExpenseHeadCreated {
    pub fn new(
        expense_head_id: ExpenseHeadId,
        name: String,
        description: Option<String>,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            expense_head_id,
            name,
            description,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ExpenseHeadCreated {
    const EVENT_TYPE: &'static str = "finance.expense_head.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "expense_head";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.expense_head_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.expense_head_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealExpenseHead`'s mutable metadata is updated
/// via `RealExpenseHead::update_metadata`. Carries only the
/// MUTABLE `description`. EH I-1 (`name`) is NOT carried here —
/// it is preserved in the audit footer of the aggregate itself.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpenseHeadUpdated {
    pub expense_head_id: ExpenseHeadId,
    pub description: Option<String>,
    pub updated_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ExpenseHeadUpdated {
    pub fn new(
        expense_head_id: ExpenseHeadId,
        description: Option<String>,
        updated_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            expense_head_id,
            description,
            updated_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ExpenseHeadUpdated {
    const EVENT_TYPE: &'static str = "finance.expense_head.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "expense_head";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.expense_head_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.expense_head_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealExpenseHead` is retired (soft-deleted via
/// `RealExpenseHead::retire`). The original `name` (EH I-1) is
/// preserved in the audit footer for legal-record retention +
/// uniqueness queries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpenseHeadRetired {
    pub expense_head_id: ExpenseHeadId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ExpenseHeadRetired {
    pub fn new(
        expense_head_id: ExpenseHeadId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            expense_head_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ExpenseHeadRetired {
    const EVENT_TYPE: &'static str = "finance.expense_head.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "expense_head";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.expense_head_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.expense_head_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealFeesGroup` catalogue entry is created via
/// `RealFeesGroup::fresh`. Carries `name` (FG I-1 uniqueness
/// anchor + FG I-2 non-empty trim guard — both pinned at
/// construction) + the mutable `description`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesGroupCreated {
    pub fees_group_id: FeesGroupId,
    pub name: String, // FG I-1 + FG I-2 pinned
    pub description: Option<String>,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FeesGroupCreated {
    pub fn new(
        fees_group_id: FeesGroupId,
        name: String,
        description: Option<String>,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fees_group_id,
            name,
            description,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FeesGroupCreated {
    const EVENT_TYPE: &'static str = "finance.fees_group.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fees_group";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fees_group_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fees_group_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealFeesGroup`'s mutable metadata is updated
/// via `RealFeesGroup::update_metadata`. Carries only the
/// MUTABLE `description`. FG I-1 (`name`) is NOT carried here —
/// it is preserved in the audit footer of the aggregate itself.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesGroupUpdated {
    pub fees_group_id: FeesGroupId,
    pub description: Option<String>,
    pub updated_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FeesGroupUpdated {
    pub fn new(
        fees_group_id: FeesGroupId,
        description: Option<String>,
        updated_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fees_group_id,
            description,
            updated_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FeesGroupUpdated {
    const EVENT_TYPE: &'static str = "finance.fees_group.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fees_group";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fees_group_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fees_group_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealFeesGroup` is retired (soft-deleted via
/// `RealFeesGroup::retire`). The original `name` (FG I-1) is
/// preserved in the audit footer for legal-record retention +
/// uniqueness queries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesGroupRetired {
    pub fees_group_id: FeesGroupId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FeesGroupRetired {
    pub fn new(
        fees_group_id: FeesGroupId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fees_group_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FeesGroupRetired {
    const EVENT_TYPE: &'static str = "finance.fees_group.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fees_group";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fees_group_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fees_group_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealDueFeesLoginPrevent` block is created via
/// `RealDueFeesLoginPrevent::fresh`. Carries the DFLP I-1
/// scope-key fields (academic_year_id + user_id + user_type) +
/// the pinned `outstanding_balance_minor` + the mutable
/// `reason`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DueFeesLoginPreventCreated {
    pub due_fees_login_prevent_id: DueFeesLoginPreventId,
    pub academic_year_id: educore_academic::AcademicYearId,
    pub user_id: UserId,
    pub user_type: crate::aggregate::DueFeesLoginPreventRole,
    pub outstanding_balance_minor: i64,
    pub reason: String,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DueFeesLoginPreventCreated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        due_fees_login_prevent_id: DueFeesLoginPreventId,
        academic_year_id: educore_academic::AcademicYearId,
        user_id: UserId,
        user_type: crate::aggregate::DueFeesLoginPreventRole,
        outstanding_balance_minor: i64,
        reason: String,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            due_fees_login_prevent_id,
            academic_year_id,
            user_id,
            user_type,
            outstanding_balance_minor,
            reason,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DueFeesLoginPreventCreated {
    const EVENT_TYPE: &'static str = "finance.due_fees_login_prevent.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "due_fees_login_prevent";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.due_fees_login_prevent_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.due_fees_login_prevent_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealDueFeesLoginPrevent`'s mutable `reason`
/// is updated via `RealDueFeesLoginPrevent::update_metadata`.
/// Carries only the MUTABLE `reason` field. DFLP I-1 scope-key
/// fields (academic_year_id + user_id + user_type) +
/// `outstanding_balance_minor` (pinned at construction) are NOT
/// carried here — preserved in the aggregate audit footer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DueFeesLoginPreventUpdated {
    pub due_fees_login_prevent_id: DueFeesLoginPreventId,
    pub reason: String,
    pub updated_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DueFeesLoginPreventUpdated {
    pub fn new(
        due_fees_login_prevent_id: DueFeesLoginPreventId,
        reason: String,
        updated_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            due_fees_login_prevent_id,
            reason,
            updated_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DueFeesLoginPreventUpdated {
    const EVENT_TYPE: &'static str = "finance.due_fees_login_prevent.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "due_fees_login_prevent";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.due_fees_login_prevent_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.due_fees_login_prevent_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealDueFeesLoginPrevent` is MANUALLY retired
/// (e.g. school admin overrides) via
/// `RealDueFeesLoginPrevent::retire`. The original DFLP I-1
/// scope-key fields are preserved in the audit footer for
/// legal-record retention. For AUTO-prune when balance reaches 0
/// (DFLP I-2), see [`DueFeesLoginPreventPruned`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DueFeesLoginPreventRetired {
    pub due_fees_login_prevent_id: DueFeesLoginPreventId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DueFeesLoginPreventRetired {
    pub fn new(
        due_fees_login_prevent_id: DueFeesLoginPreventId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            due_fees_login_prevent_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DueFeesLoginPreventRetired {
    const EVENT_TYPE: &'static str = "finance.due_fees_login_prevent.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "due_fees_login_prevent";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.due_fees_login_prevent_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.due_fees_login_prevent_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealDueFeesLoginPrevent` is AUTO-pruned via
/// `RealDueFeesLoginPrevent::prune` because the user's
/// outstanding balance reached 0 (DFLP I-2). Distinct event
/// type from manual [`DueFeesLoginPreventRetired`] so the
/// dispatcher / audit log can distinguish manual retirement
/// from auto-pruning driven by balance change.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DueFeesLoginPreventPruned {
    pub due_fees_login_prevent_id: DueFeesLoginPreventId,
    pub pruned_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DueFeesLoginPreventPruned {
    pub fn new(
        due_fees_login_prevent_id: DueFeesLoginPreventId,
        pruned_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            due_fees_login_prevent_id,
            pruned_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DueFeesLoginPreventPruned {
    const EVENT_TYPE: &'static str = "finance.due_fees_login_prevent.pruned";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "due_fees_login_prevent";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.due_fees_login_prevent_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.due_fees_login_prevent_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealFeesInvoiceSetting` is created via
/// `RealFeesInvoiceSetting::fresh`. Carries `prefix` (FISv I-1
/// pinned — non-empty trimmed + alphanumeric only) +
/// `per_th` (FISv I-2 mutable) + the mutable `description`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesInvoiceSettingCreated {
    pub fees_invoice_setting_id: FeesInvoiceSettingId,
    pub prefix: String, // FISv I-1 pinned
    pub per_th: i64, // FISv I-2
    pub description: Option<String>,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FeesInvoiceSettingCreated {
    pub fn new(
        fees_invoice_setting_id: FeesInvoiceSettingId,
        prefix: String,
        per_th: i64,
        description: Option<String>,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fees_invoice_setting_id,
            prefix,
            per_th,
            description,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FeesInvoiceSettingCreated {
    const EVENT_TYPE: &'static str = "finance.fees_invoice_setting.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fees_invoice_setting";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fees_invoice_setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fees_invoice_setting_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealFeesInvoiceSetting`'s mutable metadata is
/// updated via `RealFeesInvoiceSetting::update_metadata`. Carries
/// only the MUTABLE fields (`per_th` + `description`). FISv I-1
/// (`prefix`) is NOT carried here — preserved in the aggregate
/// audit footer (changing the invoice prefix after invoices have
/// been issued would break the audit trail).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesInvoiceSettingUpdated {
    pub fees_invoice_setting_id: FeesInvoiceSettingId,
    pub per_th: i64, // FISv I-2 (mutable)
    pub description: Option<String>,
    pub updated_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FeesInvoiceSettingUpdated {
    pub fn new(
        fees_invoice_setting_id: FeesInvoiceSettingId,
        per_th: i64,
        description: Option<String>,
        updated_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fees_invoice_setting_id,
            per_th,
            description,
            updated_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FeesInvoiceSettingUpdated {
    const EVENT_TYPE: &'static str = "finance.fees_invoice_setting.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fees_invoice_setting";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fees_invoice_setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fees_invoice_setting_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealFeesInvoiceSetting` is retired
/// (soft-deleted via `RealFeesInvoiceSetting::retire`). The
/// original `prefix` (FISv I-1) + `per_th` (FISv I-2) are
/// preserved in the audit footer for legal-record retention.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesInvoiceSettingRetired {
    pub fees_invoice_setting_id: FeesInvoiceSettingId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FeesInvoiceSettingRetired {
    pub fn new(
        fees_invoice_setting_id: FeesInvoiceSettingId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fees_invoice_setting_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FeesInvoiceSettingRetired {
    const EVENT_TYPE: &'static str = "finance.fees_invoice_setting.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fees_invoice_setting";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fees_invoice_setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fees_invoice_setting_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealFeesInstallmentCredit` credit row is
/// created via `RealFeesInstallmentCredit::fresh`. Carries the
/// FIC I-1 `amount_minor` (pinned at construction) + FIC I-2
/// `credit_source` (type-pinned via the enum) + the scope-key
/// `source_installment_id`. NOTE: NO `Updated` event exists for
/// this aggregate — the type-system-level enforcement of the
/// FIC I-3 append-only contract (parallel to Wave 70
/// `RealFeesCarryForwardLog` pattern).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesInstallmentCreditCreated {
    pub fees_installment_credit_id: FeesInstallmentCreditId,
    pub amount_minor: i64, // FIC I-1 pinned
    pub credit_source: crate::aggregate::FeesInstallmentCreditSource, // FIC I-2 type-pinned
    pub source_installment_id: FeesInstallmentId,
    pub description: Option<String>,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FeesInstallmentCreditCreated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        fees_installment_credit_id: FeesInstallmentCreditId,
        amount_minor: i64,
        credit_source: crate::aggregate::FeesInstallmentCreditSource,
        source_installment_id: FeesInstallmentId,
        description: Option<String>,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fees_installment_credit_id,
            amount_minor,
            credit_source,
            source_installment_id,
            description,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FeesInstallmentCreditCreated {
    const EVENT_TYPE: &'static str = "finance.fees_installment_credit.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fees_installment_credit";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fees_installment_credit_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fees_installment_credit_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealFeesInstallmentCredit` credit row is
/// retired (soft-deleted via `RealFeesInstallmentCredit::retire`).
/// The original `amount_minor` (FIC I-1) + `credit_source` (FIC
/// I-2) + `source_installment_id` (scope-key) are preserved in
/// the audit footer for legal-record retention.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesInstallmentCreditRetired {
    pub fees_installment_credit_id: FeesInstallmentCreditId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FeesInstallmentCreditRetired {
    pub fn new(
        fees_installment_credit_id: FeesInstallmentCreditId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fees_installment_credit_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FeesInstallmentCreditRetired {
    const EVENT_TYPE: &'static str = "finance.fees_installment_credit.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fees_installment_credit";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fees_installment_credit_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fees_installment_credit_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}



// ===================================================================
// Wave 94 — RealFmFeesInvoiceSetting events (per-aggregate wave pattern from Waves 65-93)
// ===================================================================

/// Emitted when a `RealFmFeesInvoiceSetting` is created via
/// `RealFmFeesInvoiceSetting::fresh`. Carries `prefix` (FFIS I-3
/// pinned — non-empty trimmed + alphanumeric only) +
/// `per_th` (FFIS I-1 mutable, >= 0) + `due_date` +
/// `due_date_offset_days` (FFIS I-2 mutable, >= 0).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FmFeesInvoiceSettingCreated {
    pub fm_fees_invoice_setting_id: FmFeesInvoiceSettingId,
    pub prefix: String, // FFIS I-3 pinned
    pub per_th: i64, // FFIS I-1
    pub due_date: NaiveDate, // FFIS I-2
    pub due_date_offset_days: i64, // FFIS I-2
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FmFeesInvoiceSettingCreated {
    pub fn new(
        fm_fees_invoice_setting_id: FmFeesInvoiceSettingId,
        prefix: String,
        per_th: i64,
        due_date: NaiveDate,
        due_date_offset_days: i64,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fm_fees_invoice_setting_id,
            prefix,
            per_th,
            due_date,
            due_date_offset_days,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FmFeesInvoiceSettingCreated {
    const EVENT_TYPE: &'static str = "finance.fm_fees_invoice_setting.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fm_fees_invoice_setting";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fm_fees_invoice_setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fm_fees_invoice_setting_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealFmFeesInvoiceSetting`'s mutable metadata is
/// updated via `RealFmFeesInvoiceSetting::update_metadata`. Carries
/// only the MUTABLE fields (`per_th` + `due_date` +
/// `due_date_offset_days`). FFIS I-3 (`prefix`) is NOT carried here
/// — preserved in the aggregate audit footer (changing the invoice
/// prefix after invoices have been issued would break the audit
/// trail).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FmFeesInvoiceSettingUpdated {
    pub fm_fees_invoice_setting_id: FmFeesInvoiceSettingId,
    pub per_th: i64, // FFIS I-1 (mutable)
    pub due_date: NaiveDate, // FFIS I-2 (mutable)
    pub due_date_offset_days: i64, // FFIS I-2 (mutable)
    pub updated_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FmFeesInvoiceSettingUpdated {
    pub fn new(
        fm_fees_invoice_setting_id: FmFeesInvoiceSettingId,
        per_th: i64,
        due_date: NaiveDate,
        due_date_offset_days: i64,
        updated_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fm_fees_invoice_setting_id,
            per_th,
            due_date,
            due_date_offset_days,
            updated_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FmFeesInvoiceSettingUpdated {
    const EVENT_TYPE: &'static str = "finance.fm_fees_invoice_setting.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fm_fees_invoice_setting";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fm_fees_invoice_setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fm_fees_invoice_setting_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealFmFeesInvoiceSetting` is retired
/// (soft-deleted via `RealFmFeesInvoiceSetting::retire`). The
/// original `prefix` (FFIS I-3) + `per_th` (FFIS I-1) +
/// `due_date` + `due_date_offset_days` (FFIS I-2) are preserved
/// in the audit footer for legal-record retention.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FmFeesInvoiceSettingRetired {
    pub fm_fees_invoice_setting_id: FmFeesInvoiceSettingId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FmFeesInvoiceSettingRetired {
    pub fn new(
        fm_fees_invoice_setting_id: FmFeesInvoiceSettingId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fm_fees_invoice_setting_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FmFeesInvoiceSettingRetired {
    const EVENT_TYPE: &'static str = "finance.fm_fees_invoice_setting.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fm_fees_invoice_setting";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fm_fees_invoice_setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fm_fees_invoice_setting_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}



// ===================================================================
// Wave 95 — RealFmFeesWeaver events (per-aggregate wave pattern from Waves 65-94)
// ===================================================================

/// Emitted when a `RealFmFeesWeaver` is created via
/// `RealFmFeesWeaver::fresh`. Carries `name` + `percentage`
/// (FFW I-1: `percentage` ∈ [0, 100]).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FmFeesWeaverCreated {
    pub fm_fees_weaver_id: FmFeesWeaverId,
    pub name: String,
    pub percentage: i64, // FFW I-1
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FmFeesWeaverCreated {
    pub fn new(
        fm_fees_weaver_id: FmFeesWeaverId,
        name: String,
        percentage: i64,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fm_fees_weaver_id,
            name,
            percentage,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FmFeesWeaverCreated {
    const EVENT_TYPE: &'static str = "finance.fm_fees_weaver.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fm_fees_weaver";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fm_fees_weaver_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fm_fees_weaver_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealFmFeesWeaver` is retired (soft-deleted via
/// `RealFmFeesWeaver::retire`). The original `name` + `percentage`
/// (FFW I-1) are preserved in the audit footer for legal-record
/// retention.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FmFeesWeaverRetired {
    pub fm_fees_weaver_id: FmFeesWeaverId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FmFeesWeaverRetired {
    pub fn new(
        fm_fees_weaver_id: FmFeesWeaverId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fm_fees_weaver_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FmFeesWeaverRetired {
    const EVENT_TYPE: &'static str = "finance.fm_fees_weaver.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fm_fees_weaver";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fm_fees_weaver_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fm_fees_weaver_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// ===================================================================
// Wave 96 — RealDirectFeesInstallmentChildPayment events (per-aggregate wave pattern from Waves 65-95)
// ===================================================================

/// Emitted when a `RealDirectFeesInstallmentChildPayment` is created
/// via `RealDirectFeesInstallmentChildPayment::fresh`. Carries
/// `installment_id` (scope-key DirectFeesInstallmentId) +
/// `paid_amount_minor` (FFIChild I-1 — >= 0) + `note`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectFeesInstallmentChildPaymentCreated {
    pub direct_fees_installment_child_payment_id: DirectFeesInstallmentChildPaymentId,
    pub installment_id: DirectFeesInstallmentId,
    pub paid_amount_minor: i64, // FFIChild I-1
    pub note: Option<String>,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DirectFeesInstallmentChildPaymentCreated {
    pub fn new(
        direct_fees_installment_child_payment_id: DirectFeesInstallmentChildPaymentId,
        installment_id: DirectFeesInstallmentId,
        paid_amount_minor: i64,
        note: Option<String>,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            direct_fees_installment_child_payment_id,
            installment_id,
            paid_amount_minor,
            note,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DirectFeesInstallmentChildPaymentCreated {
    const EVENT_TYPE: &'static str =
        "finance.direct_fees_installment_child_payment.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "direct_fees_installment_child_payment";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.direct_fees_installment_child_payment_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.direct_fees_installment_child_payment_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealDirectFeesInstallmentChildPayment` is retired
/// (soft-deleted via `RealDirectFeesInstallmentChildPayment::retire`).
/// The original `installment_id` + `paid_amount_minor` (FFIChild I-1)
/// + `note` are preserved in the audit footer for legal-record
/// retention.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectFeesInstallmentChildPaymentRetired {
    pub direct_fees_installment_child_payment_id: DirectFeesInstallmentChildPaymentId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DirectFeesInstallmentChildPaymentRetired {
    pub fn new(
        direct_fees_installment_child_payment_id: DirectFeesInstallmentChildPaymentId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            direct_fees_installment_child_payment_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DirectFeesInstallmentChildPaymentRetired {
    const EVENT_TYPE: &'static str =
        "finance.direct_fees_installment_child_payment.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "direct_fees_installment_child_payment";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.direct_fees_installment_child_payment_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.direct_fees_installment_child_payment_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// ===================================================================
// Wave 97 — RealIncome events (per-aggregate wave pattern from Waves 65-96)
// ===================================================================

/// Emitted when a `RealIncome` is created via `RealIncome::fresh`.
/// Carries `income_head_id` (scope-key IncomeHeadId) +
/// `amount_minor` (IN I-1 — >= 0) + `description`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncomeCreated {
    pub income_id: IncomeId,
    pub income_head_id: IncomeHeadId,
    pub amount_minor: i64, // IN I-1
    pub description: Option<String>,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl IncomeCreated {
    pub fn new(
        income_id: IncomeId,
        income_head_id: IncomeHeadId,
        amount_minor: i64,
        description: Option<String>,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            income_id,
            income_head_id,
            amount_minor,
            description,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for IncomeCreated {
    const EVENT_TYPE: &'static str = "finance.income.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "income";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.income_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.income_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealIncome` is retired (soft-deleted via
/// `RealIncome::retire`). The original `income_head_id` +
/// `amount_minor` (IN I-1) + `description` are preserved in the
/// audit footer for legal-record retention.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncomeRetired {
    pub income_id: IncomeId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl IncomeRetired {
    pub fn new(
        income_id: IncomeId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            income_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for IncomeRetired {
    const EVENT_TYPE: &'static str = "finance.income.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "income";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.income_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.income_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// ===================================================================
// Wave 98 — RealInventoryPayment events (per-aggregate wave pattern from Waves 65-97)
// ===================================================================

/// Emitted when a `RealInventoryPayment` is created via
/// `RealInventoryPayment::fresh`. Carries `supplier_name` +
/// `amount_minor` (IP I-1 — >= 0) + `currency` + optional `note`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InventoryPaymentCreated {
    pub inventory_payment_id: InventoryPaymentId,
    pub supplier_name: String,
    pub amount_minor: i64, // IP I-1
    pub currency: Currency,
    pub note: Option<String>,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl InventoryPaymentCreated {
    pub fn new(
        inventory_payment_id: InventoryPaymentId,
        supplier_name: String,
        amount_minor: i64,
        currency: Currency,
        note: Option<String>,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            inventory_payment_id,
            supplier_name,
            amount_minor,
            currency,
            note,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for InventoryPaymentCreated {
    const EVENT_TYPE: &'static str = "finance.inventory_payment.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "inventory_payment";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.inventory_payment_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.inventory_payment_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealInventoryPayment` is retired (soft-deleted via
/// `RealInventoryPayment::retire`). The original `supplier_name` +
/// `amount_minor` (IP I-1) + `currency` + `note` are preserved in the
/// audit footer for legal-record retention.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InventoryPaymentRetired {
    pub inventory_payment_id: InventoryPaymentId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl InventoryPaymentRetired {
    pub fn new(
        inventory_payment_id: InventoryPaymentId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            inventory_payment_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for InventoryPaymentRetired {
    const EVENT_TYPE: &'static str = "finance.inventory_payment.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "inventory_payment";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.inventory_payment_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.inventory_payment_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// ===================================================================
// Wave 99 — RealProductPurchase events (per-aggregate wave pattern from Waves 65-98)
// ===================================================================

/// Emitted when a `RealProductPurchase` is created via
/// `RealProductPurchase::fresh`. Carries `product_name` + `quantity` +
/// `amount_minor` (PPr I-1 — >= 0) + optional `supplier_reference`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductPurchaseCreated {
    pub product_purchase_id: ProductPurchaseId,
    pub product_name: String,
    pub quantity: i64,
    pub amount_minor: i64, // PPr I-1
    pub supplier_reference: Option<String>,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ProductPurchaseCreated {
    pub fn new(
        product_purchase_id: ProductPurchaseId,
        product_name: String,
        quantity: i64,
        amount_minor: i64,
        supplier_reference: Option<String>,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            product_purchase_id,
            product_name,
            quantity,
            amount_minor,
            supplier_reference,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ProductPurchaseCreated {
    const EVENT_TYPE: &'static str = "finance.product_purchase.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "product_purchase";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.product_purchase_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.product_purchase_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealProductPurchase` is retired (soft-deleted via
/// `RealProductPurchase::retire`). The original `product_name` +
/// `quantity` + `amount_minor` (PPr I-1) + `supplier_reference` are
/// preserved in the audit footer for legal-record retention.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductPurchaseRetired {
    pub product_purchase_id: ProductPurchaseId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ProductPurchaseRetired {
    pub fn new(
        product_purchase_id: ProductPurchaseId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            product_purchase_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ProductPurchaseRetired {
    const EVENT_TYPE: &'static str = "finance.product_purchase.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "product_purchase";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.product_purchase_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.product_purchase_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// ===================================================================
// Wave 100 — RealFmFeesInvoice events (per-aggregate wave pattern from Waves 65-99)
// ===================================================================

/// Emitted when a `RealFmFeesInvoice` is created via
/// `RealFmFeesInvoice::fresh`. Carries `invoice_number` +
/// `payer_reference` + `amount_minor` (FFI I-1 — >= 0) +
/// optional `discount_minor` + optional `note`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FmFeesInvoiceCreated {
    pub fm_fees_invoice_id: FmFeesInvoiceId,
    pub invoice_number: String,
    pub payer_reference: String,
    pub amount_minor: i64, // FFI I-1
    pub discount_minor: Option<i64>,
    pub note: Option<String>,
    pub invoice_date: NaiveDate,
    pub due_date: NaiveDate,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FmFeesInvoiceCreated {
    pub fn new(
        fm_fees_invoice_id: FmFeesInvoiceId,
        invoice_number: String,
        payer_reference: String,
        amount_minor: i64,
        discount_minor: Option<i64>,
        note: Option<String>,
        invoice_date: NaiveDate,
        due_date: NaiveDate,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fm_fees_invoice_id,
            invoice_number,
            payer_reference,
            amount_minor,
            discount_minor,
            note,
            invoice_date,
            due_date,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FmFeesInvoiceCreated {
    const EVENT_TYPE: &'static str = "finance.fm_fees_invoice.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fm_fees_invoice";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fm_fees_invoice_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fm_fees_invoice_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealFmFeesInvoice` is retired (soft-deleted via
/// `RealFmFeesInvoice::retire`). The original `invoice_number` +
/// `payer_reference` + `amount_minor` (FFI I-1) + `discount_minor` +
/// `note` are preserved in the audit footer for legal-record retention.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FmFeesInvoiceRetired {
    pub fm_fees_invoice_id: FmFeesInvoiceId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FmFeesInvoiceRetired {
    pub fn new(
        fm_fees_invoice_id: FmFeesInvoiceId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fm_fees_invoice_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FmFeesInvoiceRetired {
    const EVENT_TYPE: &'static str = "finance.fm_fees_invoice.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fm_fees_invoice";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fm_fees_invoice_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fm_fees_invoice_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// ===================================================================
// Wave 101 — RealFmFeesInvoiceChild events (per-aggregate wave pattern from Waves 65-100)
// ===================================================================

/// Emitted when a `RealFmFeesInvoiceChild` is created via
/// `RealFmFeesInvoiceChild::fresh`. Carries `invoice_id`
/// (scope-key FmFeesInvoiceId) + `description` + `amount_minor`
/// (FFIChild I-1 — >= 0).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FmFeesInvoiceChildCreated {
    pub fm_fees_invoice_child_id: FmFeesInvoiceChildId,
    pub invoice_id: FmFeesInvoiceId,
    pub description: String,
    pub amount_minor: i64, // FFIChild I-1
    pub sub_total_minor: i64, // FFIChild I-2
    pub weaver_minor: i64, // FFIChild I-2
    pub fine_minor: i64, // FFIChild I-2
    pub paid_amount_minor: i64, // FFIChild I-3
    pub service_charge_minor: i64, // FFIChild I-3
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FmFeesInvoiceChildCreated {
    pub fn new(
        fm_fees_invoice_child_id: FmFeesInvoiceChildId,
        invoice_id: FmFeesInvoiceId,
        description: String,
        amount_minor: i64,
        sub_total_minor: i64,
        weaver_minor: i64,
        fine_minor: i64,
        paid_amount_minor: i64,
        service_charge_minor: i64,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fm_fees_invoice_child_id,
            invoice_id,
            description,
            amount_minor,
            sub_total_minor,
            weaver_minor,
            fine_minor,
            paid_amount_minor,
            service_charge_minor,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FmFeesInvoiceChildCreated {
    const EVENT_TYPE: &'static str = "finance.fm_fees_invoice_child.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fm_fees_invoice_child";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fm_fees_invoice_child_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fm_fees_invoice_child_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealFmFeesInvoiceChild` is retired (soft-deleted via
/// `RealFmFeesInvoiceChild::retire`). The original `invoice_id` +
/// `description` + `amount_minor` (FFIChild I-1) are preserved in the
/// audit footer for legal-record retention.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FmFeesInvoiceChildRetired {
    pub fm_fees_invoice_child_id: FmFeesInvoiceChildId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FmFeesInvoiceChildRetired {
    pub fn new(
        fm_fees_invoice_child_id: FmFeesInvoiceChildId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fm_fees_invoice_child_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FmFeesInvoiceChildRetired {
    const EVENT_TYPE: &'static str = "finance.fm_fees_invoice_child.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fm_fees_invoice_child";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fm_fees_invoice_child_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fm_fees_invoice_child_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// ===================================================================
// Wave 103 — RealDirectFeesInstallmentAssign events (per-aggregate wave pattern from Waves 65-101)
// ===================================================================

/// Emitted when a `RealDirectFeesInstallmentAssign` is created via
/// `RealDirectFeesInstallmentAssign::fresh`. Carries the scope-key
/// tuple `(student_id, installment_id)` (DFIA I-1) + `amount_minor`
/// (DFIA I-2) + `balance_minor` (DFIA I-3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectFeesInstallmentAssignCreated {
    pub direct_fees_installment_assign_id: DirectFeesInstallmentAssignId,
    pub student_id: StudentId,
    pub installment_id: DirectFeesInstallmentId,
    pub amount_minor: i64,   // DFIA I-2
    pub balance_minor: i64,  // DFIA I-3
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DirectFeesInstallmentAssignCreated {
    pub fn new(
        direct_fees_installment_assign_id: DirectFeesInstallmentAssignId,
        student_id: StudentId,
        installment_id: DirectFeesInstallmentId,
        amount_minor: i64,
        balance_minor: i64,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            direct_fees_installment_assign_id,
            student_id,
            installment_id,
            amount_minor,
            balance_minor,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DirectFeesInstallmentAssignCreated {
    const EVENT_TYPE: &'static str = "finance.direct_fees_installment_assign.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "direct_fees_installment_assign";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.direct_fees_installment_assign_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.direct_fees_installment_assign_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealDirectFeesInstallmentAssign` is retired
/// (soft-deleted via `RealDirectFeesInstallmentAssign::retire`). The
/// original scope-key tuple + `amount_minor` (DFIA I-2) +
/// `balance_minor` (DFIA I-3) are preserved in the audit footer for
/// legal-record retention.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectFeesInstallmentAssignRetired {
    pub direct_fees_installment_assign_id: DirectFeesInstallmentAssignId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DirectFeesInstallmentAssignRetired {
    pub fn new(
        direct_fees_installment_assign_id: DirectFeesInstallmentAssignId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            direct_fees_installment_assign_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DirectFeesInstallmentAssignRetired {
    const EVENT_TYPE: &'static str = "finance.direct_fees_installment_assign.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "direct_fees_installment_assign";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.direct_fees_installment_assign_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.direct_fees_installment_assign_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}


// -- Wave 104 — Transaction (double-entry journal line) --
//
// TR I-1: the sum of debit lines equals the sum of credit lines
// (the double-entry balancing invariant). Both `TransactionCreated`
// (which carries the totals + description + reference + currency
// downstream) and `TransactionRetired` (which is a tombstone
// preserving the totals + description for legal-record retention)
// are emitted by `create_transaction` / `retire_transaction`
// service functions.

/// Emitted when a `Transaction` is first created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionCreated {
    /// Aggregate identity.
    pub transaction_id: TransactionId,
    /// Transaction date (the date the transaction is recorded).
    pub transaction_date: NaiveDate,
    /// Human-readable description (TR I-1 companion: non-empty
    /// after trimming whitespace).
    pub description: String,
    /// Optional external reference.
    pub reference: Option<String>,
    /// Sum of debit lines in minor units (TR I-1: pinned at
    /// construction with `>= 0` guard; companion invariant
    /// `total_debits_minor == total_credits_minor`).
    pub total_debits_minor: i64,
    /// Sum of credit lines in minor units (TR I-1: pinned at
    /// construction with `>= 0` guard; companion invariant
    /// `total_debits_minor == total_credits_minor`).
    pub total_credits_minor: i64,
    /// Currency the totals are denominated in.
    pub currency: Currency,
    /// Created-by user.
    pub created_by: UserId,
    /// Standard event footer: event id.
    pub event_id: EventId,
    /// Standard event footer: correlation id.
    pub correlation_id: CorrelationId,
    /// Standard event footer: occurred-at timestamp.
    pub occurred_at: Timestamp,
}

impl TransactionCreated {
    /// Construct a new `TransactionCreated` event.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        transaction_id: TransactionId,
        transaction_date: NaiveDate,
        description: String,
        reference: Option<String>,
        total_debits_minor: i64,
        total_credits_minor: i64,
        currency: Currency,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            transaction_id,
            transaction_date,
            description,
            reference,
            total_debits_minor,
            total_credits_minor,
            currency,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for TransactionCreated {
    const EVENT_TYPE: &'static str = "finance.transaction.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "transaction";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.transaction_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.transaction_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Transaction` is retired (tombstone; preserves
/// `transaction_date` + `description` + `reference` +
/// `total_debits_minor` + `total_credits_minor` + `currency` for
/// legal-record retention). The transaction totals are NOT
/// carried on the event (they are preserved in the aggregate's
/// audit footer).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionRetired {
    /// Aggregate identity.
    pub transaction_id: TransactionId,
    /// Retired-by user.
    pub retired_by: UserId,
    /// Standard event footer: event id.
    pub event_id: EventId,
    /// Standard event footer: correlation id.
    pub correlation_id: CorrelationId,
    /// Standard event footer: occurred-at timestamp.
    pub occurred_at: Timestamp,
}

impl TransactionRetired {
    /// Construct a new `TransactionRetired` event.
    pub fn new(
        transaction_id: TransactionId,
        retired_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            transaction_id,
            retired_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for TransactionRetired {
    const EVENT_TYPE: &'static str = "finance.transaction.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "transaction";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.transaction_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.transaction_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}


// -- Wave 105 — FeesInstallmentAssignDiscount (child discount on an installment assign) --
//
// FIAD I-1: applied_amount >= 0. Both
// `FeesInstallmentAssignDiscountCreated` (which carries the
// applied_amount_minor + discount_id + fees_installment_assign_id
// + currency + note downstream) and
// `FeesInstallmentAssignDiscountRetired` (which is a tombstone
// preserving the FK references for legal-record retention) are
// emitted by `create_fees_installment_assign_discount` /
// `retire_fees_installment_assign_discount` service functions.
//
// NOTE: `FeesInstallmentAssignDiscountAdded` already exists at
// events.rs:2236+ (Phase 7 Workstream F; pre-Wave 105). It has a
// different EVENT_TYPE (`finance.fees_installment_assign_discount.added`)
// and is emitted by a different code path. The Wave 105 events
// use `.created` / `.retired` to align with the per-aggregate
// pattern locked in across Waves 65-104 (and to leave room for a
// future `.updated` event in the append-only-on-amount model).

/// Emitted when a `FeesInstallmentAssignDiscount` is first created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesInstallmentAssignDiscountCreated {
    /// Aggregate identity.
    pub fees_installment_assign_discount_id: FeesInstallmentAssignDiscountId,
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
    /// Currency the applied amount is denominated in.
    pub currency: Currency,
    /// Optional human-readable note explaining the discount.
    pub note: Option<String>,
    /// Created-by user.
    pub created_by: UserId,
    /// Standard event footer: event id.
    pub event_id: EventId,
    /// Standard event footer: correlation id.
    pub correlation_id: CorrelationId,
    /// Standard event footer: occurred-at timestamp.
    pub occurred_at: Timestamp,
}

impl FeesInstallmentAssignDiscountCreated {
    /// Construct a new `FeesInstallmentAssignDiscountCreated` event.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        fees_installment_assign_discount_id: FeesInstallmentAssignDiscountId,
        discount_id: FeesDiscountId,
        fees_installment_assign_id: FeesInstallmentAssignId,
        applied_amount_minor: i64,
        currency: Currency,
        note: Option<String>,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fees_installment_assign_discount_id,
            discount_id,
            fees_installment_assign_id,
            applied_amount_minor,
            currency,
            note,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FeesInstallmentAssignDiscountCreated {
    const EVENT_TYPE: &'static str = "finance.fees_installment_assign_discount.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fees_installment_assign_discount";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fees_installment_assign_discount_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fees_installment_assign_discount_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `FeesInstallmentAssignDiscount` is retired
/// (tombstone; preserves `discount_id` +
/// `fees_installment_assign_id` for legal-record retention).
/// The applied_amount_minor + currency + note are NOT carried on
/// the event (they are preserved in the aggregate's audit
/// footer).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesInstallmentAssignDiscountRetired {
    /// Aggregate identity.
    pub fees_installment_assign_discount_id: FeesInstallmentAssignDiscountId,
    /// Retired-by user.
    pub retired_by: UserId,
    /// Standard event footer: event id.
    pub event_id: EventId,
    /// Standard event footer: correlation id.
    pub correlation_id: CorrelationId,
    /// Standard event footer: occurred-at timestamp.
    pub occurred_at: Timestamp,
}

impl FeesInstallmentAssignDiscountRetired {
    /// Construct a new `FeesInstallmentAssignDiscountRetired` event.
    pub fn new(
        fees_installment_assign_discount_id: FeesInstallmentAssignDiscountId,
        retired_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fees_installment_assign_discount_id,
            retired_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FeesInstallmentAssignDiscountRetired {
    const EVENT_TYPE: &'static str = "finance.fees_installment_assign_discount.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fees_installment_assign_discount";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fees_installment_assign_discount_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fees_installment_assign_discount_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}


// -- Wave 106 — PaymentMethod (cash / bank / cheque / card / mobile wallet / gateway) --
//
// PM I-1: method unique within school. Both `PaymentMethodCreated`
// (which carries the name + kind + description downstream) and
// `PaymentMethodRetired` (which is a tombstone preserving the
// (school_id, name) scope-key tuple for legal-record retention)
// are emitted by `create_payment_method` / `retire_payment_method`
// service functions.

/// Emitted when a `PaymentMethod` is first created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaymentMethodCreated {
    /// Aggregate identity.
    pub payment_method_id: PaymentMethodId,
    /// Display name (PM I-1 — scope-key; dispatcher-enforced
    /// uniqueness via (school_id, name) tuple).
    pub name: String,
    /// Payment kind (cash / bank / cheque / card / mobile wallet
    /// / gateway).
    pub kind: PaymentMethodKind,
    /// Optional human-readable description.
    pub description: Option<String>,
    /// Created-by user.
    pub created_by: UserId,
    /// Standard event footer: event id.
    pub event_id: EventId,
    /// Standard event footer: correlation id.
    pub correlation_id: CorrelationId,
    /// Standard event footer: occurred-at timestamp.
    pub occurred_at: Timestamp,
}

impl PaymentMethodCreated {
    /// Construct a new `PaymentMethodCreated` event.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        payment_method_id: PaymentMethodId,
        name: String,
        kind: PaymentMethodKind,
        description: Option<String>,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            payment_method_id,
            name,
            kind,
            description,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for PaymentMethodCreated {
    const EVENT_TYPE: &'static str = "finance.payment_method.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "payment_method";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.payment_method_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.payment_method_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `PaymentMethod` is retired (tombstone;
/// preserves `name` + `kind` in audit footer for legal-record
/// retention). The `name` is NOT carried on the event (it is
/// preserved in the aggregate's audit footer + the
/// `(school_id, name)` scope-key tuple survives via the
/// `payment_method_id.school_id()` accessor).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaymentMethodRetired {
    /// Aggregate identity.
    pub payment_method_id: PaymentMethodId,
    /// Retired-by user.
    pub retired_by: UserId,
    /// Standard event footer: event id.
    pub event_id: EventId,
    /// Standard event footer: correlation id.
    pub correlation_id: CorrelationId,
    /// Standard event footer: occurred-at timestamp.
    pub occurred_at: Timestamp,
}

impl PaymentMethodRetired {
    /// Construct a new `PaymentMethodRetired` event.
    pub fn new(
        payment_method_id: PaymentMethodId,
        retired_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            payment_method_id,
            retired_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for PaymentMethodRetired {
    const EVENT_TYPE: &'static str = "finance.payment_method.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "payment_method";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.payment_method_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.payment_method_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}


// -- Wave 107 -- FeesInstallmentAssign (per-(fees_assign, installment) linkage) --
//
// FIA I-1: unique per (fees_assign, installment). Both
// FeesInstallmentAssignCreated (carrying fees_assign_id +
// fees_installment_id + due_date + note downstream) and
// FeesInstallmentAssignRetired (tombstone preserving the scope-key
// tuple for legal-record retention) are emitted by
// create_fees_installment_assign / retire_fees_installment_assign
// service functions.

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesInstallmentAssignCreated {
    pub fees_installment_assign_id: FeesInstallmentAssignId,
    pub fees_assign_id: FeesAssignId,
    pub fees_installment_id: FeesInstallmentId,
    pub due_date: chrono::NaiveDate,
    pub amount_minor: i64,
    pub discount_minor: i64,
    pub paid_amount_minor: i64,
    pub note: Option<String>,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FeesInstallmentAssignCreated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        fees_installment_assign_id: FeesInstallmentAssignId,
        fees_assign_id: FeesAssignId,
        fees_installment_id: FeesInstallmentId,
        due_date: chrono::NaiveDate,
        amount_minor: i64,
        discount_minor: i64,
        paid_amount_minor: i64,
        note: Option<String>,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fees_installment_assign_id,
            fees_assign_id,
            fees_installment_id,
            due_date,
            amount_minor,
            discount_minor,
            paid_amount_minor,
            note,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FeesInstallmentAssignCreated {
    const EVENT_TYPE: &'static str = "finance.fees_installment_assign.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fees_installment_assign";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fees_installment_assign_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fees_installment_assign_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesInstallmentAssignRetired {
    pub fees_installment_assign_id: FeesInstallmentAssignId,
    pub retired_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FeesInstallmentAssignRetired {
    pub fn new(
        fees_installment_assign_id: FeesInstallmentAssignId,
        retired_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fees_installment_assign_id,
            retired_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FeesInstallmentAssignRetired {
    const EVENT_TYPE: &'static str = "finance.fees_installment_assign.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fees_installment_assign";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fees_installment_assign_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fees_installment_assign_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}


// -- Wave 108 -- AmountTransfer (inter-account cash movement) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AmountTransferCreated {
    pub amount_transfer_id: AmountTransferId,
    pub from_account_id: BankAccountId,
    pub to_account_id: BankAccountId,
    pub amount_minor: i64,
    pub currency: Currency,
    pub transfer_date: chrono::NaiveDate,
    pub note: Option<String>,
    /// AT I-3: optional idempotency reference carried on the
    /// event for downstream consumers (audit trail + idempotency
    /// check reconciliation).
    pub reference: Option<String>,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl AmountTransferCreated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        amount_transfer_id: AmountTransferId,
        from_account_id: BankAccountId,
        to_account_id: BankAccountId,
        amount_minor: i64,
        currency: Currency,
        transfer_date: chrono::NaiveDate,
        note: Option<String>,
        reference: Option<String>,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            amount_transfer_id,
            from_account_id,
            to_account_id,
            amount_minor,
            currency,
            transfer_date,
            note,
            reference,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for AmountTransferCreated {
    const EVENT_TYPE: &'static str = "finance.amount_transfer.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "amount_transfer";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.amount_transfer_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.amount_transfer_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AmountTransferRetired {
    pub amount_transfer_id: AmountTransferId,
    pub retired_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl AmountTransferRetired {
    pub fn new(
        amount_transfer_id: AmountTransferId,
        retired_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            amount_transfer_id,
            retired_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for AmountTransferRetired {
    const EVENT_TYPE: &'static str = "finance.amount_transfer.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "amount_transfer";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.amount_transfer_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.amount_transfer_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}



// -- Wave 111 -- DirectFeesInstallment (Phase 7 stub replaced) --
//
// DFI I-2: amount >= 0. Both DirectFeesInstallmentCreated (carrying
// the amount_minor + due_date + name downstream) and
// DirectFeesInstallmentRetired (tombstone preserving the identity
// for legal-record retention) are emitted by
// create_direct_fees_installment / retire_direct_fees_installment
// service functions.
//
// NOTE: the pre-existing finance_event_stub! macro at events.rs:2160
// was deleted and replaced with this comment marker + the full
// struct definitions below.

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectFeesInstallmentCreated {
    pub direct_fees_installment_id: DirectFeesInstallmentId,
    pub student_id: educore_academic::StudentId,
    pub name: String,
    pub amount_minor: i64,
    pub currency: Currency,
    pub due_date: NaiveDate,
    pub percentage_minor: i64, // DFI I-3
    pub window_start: Option<NaiveDate>, // DFI I-4
    pub window_end: Option<NaiveDate>, // DFI I-4
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DirectFeesInstallmentCreated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        direct_fees_installment_id: DirectFeesInstallmentId,
        student_id: educore_academic::StudentId,
        name: String,
        amount_minor: i64,
        currency: Currency,
        due_date: NaiveDate,
        percentage_minor: i64,
        window_start: Option<NaiveDate>,
        window_end: Option<NaiveDate>,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            direct_fees_installment_id,
            student_id,
            name,
            amount_minor,
            currency,
            due_date,
            percentage_minor,
            window_start,
            window_end,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DirectFeesInstallmentCreated {
    const EVENT_TYPE: &'static str = "finance.direct_fees_installment.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "direct_fees_installment";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.direct_fees_installment_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.direct_fees_installment_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectFeesInstallmentRetired {
    pub direct_fees_installment_id: DirectFeesInstallmentId,
    pub retired_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DirectFeesInstallmentRetired {
    pub fn new(
        direct_fees_installment_id: DirectFeesInstallmentId,
        retired_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            direct_fees_installment_id,
            retired_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DirectFeesInstallmentRetired {
    const EVENT_TYPE: &'static str = "finance.direct_fees_installment.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "direct_fees_installment";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.direct_fees_installment_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.direct_fees_installment_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}


// -- Wave 113 -- FeesCarryForward (end-of-year balance roll-over) --
//
// FCF I-3: unique per (school, student, academic). Both
// FeesCarryForwardCreated (carrying the scope-key tuple +
// balance_minor + balance_type + currency downstream) and
// FeesCarryForwardRetired (tombstone preserving the scope-key
// tuple for legal-record retention) are emitted by
// create_fees_carry_forward / retire_fees_carry_forward
// service functions.

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesCarryForwardCreated {
    pub fees_carry_forward_id: FeesCarryForwardId,
    pub student_id: educore_academic::StudentId,
    pub academic_year_id: educore_academic::AcademicYearId,
    pub balance_minor: i64,
    pub balance_type: BalanceType,
    pub currency: Currency,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FeesCarryForwardCreated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        fees_carry_forward_id: FeesCarryForwardId,
        student_id: educore_academic::StudentId,
        academic_year_id: educore_academic::AcademicYearId,
        balance_minor: i64,
        balance_type: BalanceType,
        currency: Currency,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fees_carry_forward_id,
            student_id,
            academic_year_id,
            balance_minor,
            balance_type,
            currency,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FeesCarryForwardCreated {
    const EVENT_TYPE: &'static str = "finance.fees_carry_forward.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fees_carry_forward";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fees_carry_forward_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fees_carry_forward_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesCarryForwardRetired {
    pub fees_carry_forward_id: FeesCarryForwardId,
    pub retired_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FeesCarryForwardRetired {
    pub fn new(
        fees_carry_forward_id: FeesCarryForwardId,
        retired_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fees_carry_forward_id,
            retired_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FeesCarryForwardRetired {
    const EVENT_TYPE: &'static str = "finance.fees_carry_forward.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fees_carry_forward";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fees_carry_forward_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fees_carry_forward_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}


// -- Wave 114 -- FeesMaster (Phase 7 finance_event_stub! for
// FeesMasterCreated deleted at events.rs:2104 and replaced with
// the full struct definitions below).
//
// FM I-2: unique per (school, name, group). Both
// FeesMasterCreated (carrying the scope-key tuple + amount_minor
// + currency + due_date downstream) and FeesMasterRetired
// (tombstone preserving the scope-key tuple for legal-record
// retention) are emitted by create_fees_master /
// retire_fees_master service functions.

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesMasterCreated {
    pub fees_master_id: FeesMasterId,
    pub name: String,
    pub fees_group_id: FeesGroupId,
    pub class_id: crate::value_objects::ClassId,
    pub amount_minor: i64,
    pub currency: Currency,
    pub due_date: NaiveDate,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FeesMasterCreated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        fees_master_id: FeesMasterId,
        name: String,
        fees_group_id: FeesGroupId,
        class_id: crate::value_objects::ClassId,
        amount_minor: i64,
        currency: Currency,
        due_date: NaiveDate,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fees_master_id,
            name,
            fees_group_id,
            class_id,
            amount_minor,
            currency,
            due_date,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FeesMasterCreated {
    const EVENT_TYPE: &'static str = "finance.fees_master.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fees_master";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fees_master_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fees_master_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesMasterRetired {
    pub fees_master_id: FeesMasterId,
    pub retired_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FeesMasterRetired {
    pub fn new(
        fees_master_id: FeesMasterId,
        retired_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fees_master_id,
            retired_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FeesMasterRetired {
    const EVENT_TYPE: &'static str = "finance.fees_master.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fees_master";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fees_master_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fees_master_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}


// -- Wave 118 -- FeesAssignDiscount (Phase 7 stub replaced) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesAssignDiscountCreated {
    pub fees_assign_discount_id: FeesAssignDiscountId,
    pub fees_assign_id: FeesAssignId,
    pub discount_id: FeesDiscountId,
    pub applied_amount_minor: i64,
    pub unapplied_amount_minor: i64,
    pub currency: Currency,
    pub note: Option<String>,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FeesAssignDiscountCreated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        fees_assign_discount_id: FeesAssignDiscountId,
        fees_assign_id: FeesAssignId,
        discount_id: FeesDiscountId,
        applied_amount_minor: i64,
        unapplied_amount_minor: i64,
        currency: Currency,
        note: Option<String>,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fees_assign_discount_id,
            fees_assign_id,
            discount_id,
            applied_amount_minor,
            unapplied_amount_minor,
            currency,
            note,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FeesAssignDiscountCreated {
    const EVENT_TYPE: &'static str = "finance.fees_assign_discount.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fees_assign_discount";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fees_assign_discount_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fees_assign_discount_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesAssignDiscountRetired {
    pub fees_assign_discount_id: FeesAssignDiscountId,
    pub retired_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FeesAssignDiscountRetired {
    pub fn new(
        fees_assign_discount_id: FeesAssignDiscountId,
        retired_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fees_assign_discount_id,
            retired_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FeesAssignDiscountRetired {
    const EVENT_TYPE: &'static str = "finance.fees_assign_discount.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fees_assign_discount";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fees_assign_discount_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fees_assign_discount_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}


// -- Wave 119 -- FeesAssign (per-(student, fee_master, year) linkage) --
//
// FA I-5: unique per (student, fee_master, year). Both
// FeesAssignCreated (carrying the scope-key tuple +
// amount_minor + currency + due_date downstream) and
// FeesAssignRetired (tombstone preserving the scope-key
// tuple for legal-record retention) are emitted by
// create_fees_assign / retire_fees_assign service functions.

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesAssignCreated {
    pub fees_assign_id: FeesAssignId,
    pub student_id: educore_academic::StudentId,
    pub fees_master_id: FeesMasterId,
    pub academic_year_id: educore_academic::AcademicYearId,
    pub amount_minor: i64,
    pub currency: Currency,
    pub due_date: NaiveDate,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FeesAssignCreated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        fees_assign_id: FeesAssignId,
        student_id: educore_academic::StudentId,
        fees_master_id: FeesMasterId,
        academic_year_id: educore_academic::AcademicYearId,
        amount_minor: i64,
        currency: Currency,
        due_date: NaiveDate,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fees_assign_id,
            student_id,
            fees_master_id,
            academic_year_id,
            amount_minor,
            currency,
            due_date,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FeesAssignCreated {
    const EVENT_TYPE: &'static str = "finance.fees_assign.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fees_assign";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fees_assign_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fees_assign_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesAssignRetired {
    pub fees_assign_id: FeesAssignId,
    pub retired_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FeesAssignRetired {
    pub fn new(
        fees_assign_id: FeesAssignId,
        retired_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fees_assign_id,
            retired_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FeesAssignRetired {
    const EVENT_TYPE: &'static str = "finance.fees_assign.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fees_assign";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fees_assign_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fees_assign_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// RealFmFeesTransaction events — Wave 124 (per-aggregate wave pattern
// from Waves 65–123)
// =============================================================================
//
// Per v3 Part 2 F32 + checklist § FmFeesTransaction: 1 invariant
// dropped in Wave 124:
//   - FFT I-2: total_paid_amount_minor ≥ 0 (carried on Created;
//     append-only on the parent aggregate so the invariant holds
//     at every event emission)

/// Emitted when a new `RealFmFeesTransaction` row is appended.
/// Carries the FFT I-2 invariant value (`total_paid_amount_minor >= 0`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FmFeesTransactionCreated {
    pub fm_fees_transaction_id: FmFeesTransactionId,
    pub total_paid_amount_minor: i64,
    pub transaction_date: chrono::NaiveDate,
    pub description: Option<String>,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FmFeesTransactionCreated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        fm_fees_transaction_id: FmFeesTransactionId,
        total_paid_amount_minor: i64,
        transaction_date: chrono::NaiveDate,
        description: Option<String>,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fm_fees_transaction_id,
            total_paid_amount_minor,
            transaction_date,
            description,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FmFeesTransactionCreated {
    const EVENT_TYPE: &'static str = "finance.fm_fees_transaction.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fm_fees_transaction";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fm_fees_transaction_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fm_fees_transaction_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealFmFeesTransaction` row is retired (tombstone).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FmFeesTransactionRetired {
    pub fm_fees_transaction_id: FmFeesTransactionId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FmFeesTransactionRetired {
    pub fn new(
        fm_fees_transaction_id: FmFeesTransactionId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fm_fees_transaction_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FmFeesTransactionRetired {
    const EVENT_TYPE: &'static str = "finance.fm_fees_transaction.retired";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fm_fees_transaction";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fm_fees_transaction_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fm_fees_transaction_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// RealFmFeesTransaction state-machine events — Wave 125 (FFT I-3)
// =============================================================================

/// Emitted when a `RealFmFeesTransaction` transitions from
/// `Pending` to `Approved`. Carries the approver, the approval
/// timestamp, and the canonical `status` for downstream indexing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FmFeesTransactionApproved {
    pub fm_fees_transaction_id: FmFeesTransactionId,
    pub approved_by: UserId,
    pub status: ApprovalStatus,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FmFeesTransactionApproved {
    pub fn new(
        fm_fees_transaction_id: FmFeesTransactionId,
        approved_by: UserId,
        status: ApprovalStatus,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fm_fees_transaction_id,
            approved_by,
            status,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FmFeesTransactionApproved {
    const EVENT_TYPE: &'static str = "finance.fm_fees_transaction.approved";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fm_fees_transaction";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fm_fees_transaction_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fm_fees_transaction_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `RealFmFeesTransaction` transitions from
/// `Pending` to `Rejected`. Carries the rejecter, the rejection
/// timestamp, the rejection note, and the canonical `status`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FmFeesTransactionRejected {
    pub fm_fees_transaction_id: FmFeesTransactionId,
    pub rejected_by: UserId,
    pub status: ApprovalStatus,
    pub reject_note: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FmFeesTransactionRejected {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        fm_fees_transaction_id: FmFeesTransactionId,
        rejected_by: UserId,
        status: ApprovalStatus,
        reject_note: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fm_fees_transaction_id,
            rejected_by,
            status,
            reject_note,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FmFeesTransactionRejected {
    const EVENT_TYPE: &'static str = "finance.fm_fees_transaction.rejected";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fm_fees_transaction";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fm_fees_transaction_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fm_fees_transaction_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[cfg(test)]
#[allow(t)]
#[allow(t)]
#[allow(est)]
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
    fn wallet_credited_event_type_is_finance_wallet_credited() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let user = g.next_user_id();
        let wid = WalletId::new(school, g.next_uuid());
        let txid = WalletTransactionId::new(school, g.next_uuid());
        let ev = WalletCredited::new(
            wid,
            txid,
            user,
            1000,
            Currency::INR,
            WalletTxType::Deposit,
            g.next_event_id(),
            CorrelationId(g.next_uuid()),
            Timestamp::now(),
        );
        assert_eq!(
            <WalletCredited as DomainEvent>::EVENT_TYPE,
            "finance.wallet.credited"
        );
        assert_eq!(ev.aggregate_id(), wid.as_uuid());
        assert_eq!(ev.school_id(), school);
    }

    #[test]
    fn wallet_refund_requested_event_type_is_finance_wallet_refund_requested() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let user = g.next_user_id();
        let wid = WalletId::new(school, g.next_uuid());
        let txid = WalletTransactionId::new(school, g.next_uuid());
        let ev = WalletRefundRequested::new(
            txid,
            wid,
            user,
            500,
            Currency::INR,
            "test refund".to_owned(),
            g.next_event_id(),
            CorrelationId(g.next_uuid()),
            Timestamp::now(),
        );
        assert_eq!(
            <WalletRefundRequested as DomainEvent>::EVENT_TYPE,
            "finance.wallet.refund_requested"
        );
        assert_eq!(ev.aggregate_id(), txid.as_uuid());
    }

    #[test]
    fn expense_recorded_event_type() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let id = crate::value_objects::ExpenseId::new(school, g.next_uuid());
        let head = ExpenseHeadId::new(school, g.next_uuid());
        let acct = BankAccountId::new(school, g.next_uuid());
        let ev = ExpenseRecorded::new(
            id,
            "Office supplies".to_owned(),
            5000,
            Currency::INR,
            head,
            acct,
            PaymentMethodKind::Cash,
            chrono::NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
            None,
            g.next_event_id(),
            CorrelationId(g.next_uuid()),
            Timestamp::now(),
        );
        assert_eq!(
            <ExpenseRecorded as DomainEvent>::EVENT_TYPE,
            "finance.expense.recorded"
        );
        assert_eq!(<ExpenseRecorded as DomainEvent>::AGGREGATE_TYPE, "expense");
    }
}
