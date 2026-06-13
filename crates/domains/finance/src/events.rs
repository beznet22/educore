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

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::{CorrelationId, EventId, Identifier, SchoolId, UserId};
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;

use crate::value_objects::{
    BankAccountId, Currency, ExpenseHeadId, ExpenseId, PaymentMethodId, PaymentMethodKind,
    PayrollPaymentId, WalletId, WalletTransactionId, WalletTxType,
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
