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
    BankAccountId, BankPaymentSlipAuditId, BankPaymentSlipId, BankStatementAttachmentId, Currency,
    DirectFeesInstallmentAssignChildId, DirectFeesInstallmentId, DueFeesLoginPreventId,
    ExpenseApprovalId, ExpenseHeadId, ExpenseId, FeesAssignDiscountId, FeesAssignId,
    FeesCarryForwardId, FeesGroupId, FeesInstallmentAssignDiscountId, FeesInstallmentId,
    FeesMasterId, FeesPaymentId, FeesTypeId, FmFeesInvoiceId, FmFeesInvoiceLineNoteId,
    FmFeesTransactionLineNoteId, IncomeApprovalId, IncomeHeadId, IncomeId, PaymentMethodId,
    PaymentMethodKind, PayrollGenerateId, PayrollPaymentApprovalId, PayrollPaymentId, WalletId,
    WalletTransactionApprovalId, WalletTransactionId, WalletTxType,
};

use educore_academic::{ClassId, SectionId};

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

/// Emitted when a `DirectFeesInstallmentAssignChild` child entity is added.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectFeesInstallmentAssignChildAdded {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub aggregate_id: DirectFeesInstallmentAssignChildId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DirectFeesInstallmentAssignChildAdded {
    pub fn new(
        event_id: EventId,
        school_id: SchoolId,
        aggregate_id: DirectFeesInstallmentAssignChildId,
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

impl DomainEvent for DirectFeesInstallmentAssignChildAdded {
    const EVENT_TYPE: &'static str = "finance.direct_fees_installment_assign_child.added";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "direct_fees_installment_assign_child";
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

/// Emitted when an `FmFeesTransactionLineNote` child entity is added.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FmFeesTransactionLineNoteAdded {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub aggregate_id: FmFeesTransactionLineNoteId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FmFeesTransactionLineNoteAdded {
    pub fn new(
        event_id: EventId,
        school_id: SchoolId,
        aggregate_id: FmFeesTransactionLineNoteId,
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

impl DomainEvent for FmFeesTransactionLineNoteAdded {
    const EVENT_TYPE: &'static str = "finance.fm_fees_transaction_line_note.added";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fm_fees_transaction_line_note";
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

/// Emitted when a `WalletTransactionApproval` child entity is recorded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalletTransactionApprovalRecorded {
    pub event_id: EventId,
    pub school_id: SchoolId,
    pub aggregate_id: WalletTransactionApprovalId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl WalletTransactionApprovalRecorded {
    pub fn new(
        event_id: EventId,
        school_id: SchoolId,
        aggregate_id: WalletTransactionApprovalId,
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

impl DomainEvent for WalletTransactionApprovalRecorded {
    const EVENT_TYPE: &'static str = "finance.wallet_transaction_approval.recorded";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "wallet_transaction_approval";
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
