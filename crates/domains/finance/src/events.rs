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
    AccountType, BankAccountId, BankPaymentSlipAuditId, BankPaymentSlipId, BankStatementAttachmentId,
    BankStatementId, StatementType,
    ChartOfAccountId, Currency, DirectFeesInstallmentAssignChildId, DirectFeesInstallmentAssignId, DiscountType,
    DirectFeesInstallmentId, DirectFeesSettingId, DonorId,
    DueFeesLoginPreventId,
    ExpenseApprovalId, ExpenseHeadId, ExpenseId, FeesAssignDiscountId, FeesAssignId,
    FeesCarryForwardId, FeesCarryForwardLogId, FeesCarryForwardSettingId, FeesDiscountId, FeesGroupId, FeesInstallmentAssignDiscountId,
    FeesInstallmentId,
    FeesMasterId, FeesPaymentId, FeesTypeId, FmFeesGroupId, FmFeesInvoiceId,
    FmFeesInvoiceLineNoteId, FmFeesTransactionChildId, FmFeesTransactionId,
    FmFeesTransactionLineNoteId, IncomeApprovalId, IncomeHeadId,
    IncomeId, InvoiceSettingId, PaymentMethodId, PaymentMethodKind, PayrollGenerateId,
    PayrollPaymentApprovalId, PayrollPaymentId, QuestionBankFeeId, SalaryTemplateId, WalletId,
    WalletTransactionApprovalId, WalletTransactionId, WalletTxType,
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
    /// Emitted when a new `FeesGroup` aggregate is created.
    pub struct FeesGroupCreated;
    event_type: "finance.fees_group.created",
    aggregate_type: "fees_group",
    aggregate_id: FeesGroupId,
}

finance_event_stub! {
    /// Emitted when a new `FeesType` aggregate is created.
    pub struct FeesTypeCreated;
    event_type: "finance.fees_type.created",
    aggregate_type: "fees_type",
    aggregate_id: FeesTypeId,
}

finance_event_stub! {
    /// Emitted when a new `FeesMaster` aggregate is created.
    pub struct FeesMasterCreated;
    event_type: "finance.fees_master.created",
    aggregate_type: "fees_master",
    aggregate_id: FeesMasterId,
}

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

finance_event_stub! {
    /// Emitted when a `FeesAssignDiscount` row is assigned to a student.
    pub struct FeesDiscountAssigned;
    event_type: "finance.fees_assign_discount.assigned",
    aggregate_type: "fees_assign_discount",
    aggregate_id: FeesAssignDiscountId,
}

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

finance_event_stub! {
    /// Emitted when a `DirectFeesInstallment` is created.
    pub struct DirectFeesInstallmentCreated;
    event_type: "finance.direct_fees_installment.created",
    aggregate_type: "direct_fees_installment",
    aggregate_id: DirectFeesInstallmentId,
}

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
