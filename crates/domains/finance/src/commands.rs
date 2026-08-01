//! # Finance command structs and command-type constants
//!
//! Phase 7 ships the typed command shapes for the headline 6
//! aggregates (`Wallet`, `WalletTransaction`, `FeesInvoice`,
//! `FeesPayment`, `Expense`, `Refund`) plus the supporting
//! command-type constants the idempotency sub-port reads.
//!
//! This module also ships the typed command shapes and
//! command-type constants for the **full set of finance commands**
//! — every (aggregate × action) pair in
//! `docs/specs/finance/aggregates.md` and the report catalogue in
//! `docs/specs/finance/reports.md` is covered. The idempotency
//! sub-port keys commands by `command_type`; the constants here
//! are the canonical values for that key.

#![allow(missing_docs)]
#![allow(unused_imports)]
#![allow(dead_code)]

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use educore_core::ids::UserId;
use uuid::Uuid;
use educore_rbac::value_objects::Capability;
use educore_academic::{AcademicYearId, StudentId};
use educore_core::tenant::TenantContext;
use educore_core::value_objects::Timestamp;

use crate::value_objects::{
    AccountType, AmountTransferId, BalanceType, BankAccountId, BankMode, BankPaymentSlipId, BankPaymentSlipAuditId, BankStatementId,
    ChartOfAccountId, Currency, DirectFeesInstallmentAssignChildId, DirectFeesInstallmentAssignId,
    DirectFeesInstallmentChildPaymentId,
    DirectFeesInstallmentId, DirectFeesReminderId, DirectFeesSettingId, DonorId,
    DueFeesLoginPreventId, ExpenseApprovalId, ExpenseHeadId, ExpenseId, FeesAssignDiscountId, FeesAssignId,
    FeesCarryForwardId, FeesCarryForwardLogId, FeesCarryForwardSettingId, FeesDiscountId,
    FeesGroupId, FeesInstallmentAssignDiscountId, FeesInstallmentAssignId, FeesInstallmentCreditId, FeesInstallmentId,
    FeesInvoiceId, FeesInvoiceSettingId, FeesMasterId, FeesPaymentId, FeesPaymentSlipId,
    FeesTypeId, FmFeesGroupId, FmFeesInvoiceChildId, FmFeesInvoiceId, FmFeesInvoiceSettingId,
    FmFeesTransactionChildId, FmFeesTransactionId, FmFeesTypeId, FmFeesWeaverId, GatewayMode,
    IncomeApprovalId, IncomeHeadId, IncomeId, InventoryPaymentId, InvoiceSettingId, PaymentGatewaySettingId,
    PaymentMethodId, PaymentMethodKind, PayrollPaymentId, PreventReason, ProductPurchaseId,
    SalaryTemplateId, StatementType, TransactionId, WalletId, WalletTransactionApprovalId, WalletTransactionId,
    DiscountType,
    WalletTxType,
};

// =============================================================================
// Command-type constants (the idempotency sub-port key)
// =============================================================================

// -- Wallet & WalletTransaction (the headline 6) --

pub const FINANCE_WALLET_CREATE_COMMAND_TYPE: &str = "finance.wallet.create";
pub const FINANCE_WALLET_CREDIT_COMMAND_TYPE: &str = "finance.wallet.credit";
pub const FINANCE_WALLET_DEBIT_COMMAND_TYPE: &str = "finance.wallet.debit";
pub const FINANCE_WALLET_READ_COMMAND_TYPE: &str = "finance.wallet.read";
pub const FINANCE_WALLET_REFUND_REQUEST_COMMAND_TYPE: &str = "finance.wallet.refund_request";
pub const FINANCE_WALLET_TRANSACTION_APPROVE_COMMAND_TYPE: &str =
    "finance.wallet_transaction.approve";
pub const FINANCE_WALLET_TRANSACTION_REJECT_COMMAND_TYPE: &str =
    "finance.wallet_transaction.reject";
pub const FINANCE_WALLET_TRANSACTION_READ_COMMAND_TYPE: &str = "finance.wallet_transaction.read";

// -- FeesInvoice & FeesPayment (the headline 6) --

pub const FINANCE_FEES_INVOICE_CONFIGURE_COMMAND_TYPE: &str = "finance.fees_invoice.configure";
pub const FINANCE_FEES_INVOICE_GENERATE_COMMAND_TYPE: &str = "finance.fees_invoice.generate";
pub const FINANCE_FEES_INVOICE_UPDATE_COMMAND_TYPE: &str = "finance.fees_invoice.update";
pub const FINANCE_FEES_INVOICE_CANCEL_COMMAND_TYPE: &str = "finance.fees_invoice.cancel";
pub const FINANCE_FEES_INVOICE_READ_COMMAND_TYPE: &str = "finance.fees_invoice.read";
pub const FINANCE_FEES_PAYMENT_RECORD_COMMAND_TYPE: &str = "finance.fees_payment.record";
pub const FINANCE_FEES_PAYMENT_REVERSE_COMMAND_TYPE: &str = "finance.fees_payment.reverse";
pub const FINANCE_FEES_PAYMENT_REFUND_COMMAND_TYPE: &str = "finance.fees_payment.refund";
pub const FINANCE_FEES_PAYMENT_READ_COMMAND_TYPE: &str = "finance.fees_payment.read";

// -- FeesGroup (the fees catalogue) --

pub const FINANCE_FEES_GROUP_CREATE_COMMAND_TYPE: &str = "finance.fees_group.create";
pub const FINANCE_FEES_GROUP_UPDATE_COMMAND_TYPE: &str = "finance.fees_group.update";
pub const FINANCE_FEES_GROUP_DELETE_COMMAND_TYPE: &str = "finance.fees_group.delete";
pub const FINANCE_FEES_GROUP_READ_COMMAND_TYPE: &str = "finance.fees_group.read";

// -- FeesType (per-group fee line items) --

pub const FINANCE_FEES_TYPE_CREATE_COMMAND_TYPE: &str = "finance.fees_type.create";
pub const FINANCE_FEES_TYPE_UPDATE_COMMAND_TYPE: &str = "finance.fees_type.update";
pub const FINANCE_FEES_TYPE_DELETE_COMMAND_TYPE: &str = "finance.fees_type.delete";
pub const FINANCE_FEES_TYPE_READ_COMMAND_TYPE: &str = "finance.fees_type.read";

// -- FeesMaster (the per-class fee template) --

pub const FINANCE_FEES_MASTER_CREATE_COMMAND_TYPE: &str = "finance.fees_master.create";
pub const FINANCE_FEES_MASTER_UPDATE_COMMAND_TYPE: &str = "finance.fees_master.update";
pub const FINANCE_FEES_MASTER_DELETE_COMMAND_TYPE: &str = "finance.fees_master.delete";
pub const FINANCE_FEES_MASTER_READ_COMMAND_TYPE: &str = "finance.fees_master.read";
pub const FINANCE_FEES_MASTER_RETIRE_COMMAND_TYPE: &str = "finance.fees_master.retire";

// -- FeesDiscount (the discount catalogue) --

pub const FINANCE_FEES_DISCOUNT_CREATE_COMMAND_TYPE: &str = "finance.fees_discount.create";
pub const FINANCE_FEES_DISCOUNT_UPDATE_COMMAND_TYPE: &str = "finance.fees_discount.update";
pub const FINANCE_FEES_DISCOUNT_DELETE_COMMAND_TYPE: &str = "finance.fees_discount.delete";
pub const FINANCE_FEES_DISCOUNT_READ_COMMAND_TYPE: &str = "finance.fees_discount.read";

// -- FeesAssign (per-student fee assignment) --

pub const FINANCE_FEES_ASSIGN_CREATE_COMMAND_TYPE: &str = "finance.fees_assign.create";
pub const FINANCE_FEES_ASSIGN_UPDATE_COMMAND_TYPE: &str = "finance.fees_assign.update";
pub const FINANCE_FEES_ASSIGN_DELETE_COMMAND_TYPE: &str = "finance.fees_assign.delete";

// -- FeesInstallment (split-by-installment plans) --

pub const FINANCE_FEES_INSTALLMENT_CREATE_COMMAND_TYPE: &str = "finance.fees_installment.create";
pub const FINANCE_FEES_INSTALLMENT_UPDATE_COMMAND_TYPE: &str = "finance.fees_installment.update";
pub const FINANCE_FEES_INSTALLMENT_DELETE_COMMAND_TYPE: &str = "finance.fees_installment.delete";

// -- DirectFeesInstallment (ad-hoc installments for a single student) --

pub const FINANCE_DIRECT_FEES_INSTALLMENT_CREATE_COMMAND_TYPE: &str =
    "finance.direct_fees_installment.create";
pub const FINANCE_DIRECT_FEES_INSTALLMENT_UPDATE_COMMAND_TYPE: &str =
    "finance.direct_fees_installment.update";
pub const FINANCE_DIRECT_FEES_INSTALLMENT_DELETE_COMMAND_TYPE: &str =
    "finance.direct_fees_installment.delete";
pub const FINANCE_DIRECT_FEES_INSTALLMENT_READ_COMMAND_TYPE: &str =
    "finance.direct_fees_installment.read";

// -- DirectFeesSetting (per-school direct-fees configuration) --

pub const FINANCE_DIRECT_FEES_SETTING_CREATE_COMMAND_TYPE: &str =
    "finance.direct_fees_setting.create";
pub const FINANCE_DIRECT_FEES_SETTING_UPDATE_COMMAND_TYPE: &str =
    "finance.direct_fees_setting.update";
pub const FINANCE_DIRECT_FEES_SETTING_DELETE_COMMAND_TYPE: &str =
    "finance.direct_fees_setting.delete";

// -- DirectFeesReminder (per-student reminder configuration) --

pub const FINANCE_DIRECT_FEES_REMINDER_CREATE_COMMAND_TYPE: &str =
    "finance.direct_fees_reminder.create";
pub const FINANCE_DIRECT_FEES_REMINDER_UPDATE_COMMAND_TYPE: &str =
    "finance.direct_fees_reminder.update";
pub const FINANCE_DIRECT_FEES_REMINDER_DELETE_COMMAND_TYPE: &str =
    "finance.direct_fees_reminder.delete";

// -- PaymentMethod (cash / bank / cheque / card / mobile) --

pub const FINANCE_PAYMENT_METHOD_CREATE_COMMAND_TYPE: &str = "finance.payment_method.create";
pub const FINANCE_PAYMENT_METHOD_UPDATE_COMMAND_TYPE: &str = "finance.payment_method.update";
pub const FINANCE_PAYMENT_METHOD_DELETE_COMMAND_TYPE: &str = "finance.payment_method.delete";
pub const FINANCE_PAYMENT_METHOD_READ_COMMAND_TYPE: &str = "finance.payment_method.read";

// -- PaymentGateway (Stripe / PayPal / Razorpay settings) --

pub const FINANCE_PAYMENT_GATEWAY_CREATE_COMMAND_TYPE: &str = "finance.payment_gateway.create";
pub const FINANCE_PAYMENT_GATEWAY_UPDATE_COMMAND_TYPE: &str = "finance.payment_gateway.update";
pub const FINANCE_PAYMENT_GATEWAY_DELETE_COMMAND_TYPE: &str = "finance.payment_gateway.delete";

// -- Expense (the expense ledger) --

pub const FINANCE_EXPENSE_RECORD_COMMAND_TYPE: &str = "finance.expense.record";
pub const FINANCE_EXPENSE_UPDATE_COMMAND_TYPE: &str = "finance.expense.update";
pub const FINANCE_EXPENSE_DELETE_COMMAND_TYPE: &str = "finance.expense.delete";
pub const FINANCE_EXPENSE_APPROVE_COMMAND_TYPE: &str = "finance.expense.approve";

// -- Income (the income ledger) --

pub const FINANCE_INCOME_CREATE_COMMAND_TYPE: &str = "finance.income.create";
pub const FINANCE_INCOME_UPDATE_COMMAND_TYPE: &str = "finance.income.update";
pub const FINANCE_INCOME_DELETE_COMMAND_TYPE: &str = "finance.income.delete";
pub const FINANCE_INCOME_APPROVE_COMMAND_TYPE: &str = "finance.income.approve";

// -- ExpenseHead (the expense category catalogue) --

pub const FINANCE_EXPENSE_HEAD_CREATE_COMMAND_TYPE: &str = "finance.expense_head.create";
pub const FINANCE_EXPENSE_HEAD_UPDATE_COMMAND_TYPE: &str = "finance.expense_head.update";
pub const FINANCE_EXPENSE_HEAD_DELETE_COMMAND_TYPE: &str = "finance.expense_head.delete";

// -- IncomeHead (the income category catalogue) --

pub const FINANCE_INCOME_HEAD_CREATE_COMMAND_TYPE: &str = "finance.income_head.create";
pub const FINANCE_INCOME_HEAD_UPDATE_COMMAND_TYPE: &str = "finance.income_head.update";
pub const FINANCE_INCOME_HEAD_DELETE_COMMAND_TYPE: &str = "finance.income_head.delete";

// -- BankAccount (the cash + bank ledger) --

pub const FINANCE_BANK_ACCOUNT_OPEN_COMMAND_TYPE: &str = "finance.bank_account.open";
pub const FINANCE_BANK_ACCOUNT_UPDATE_COMMAND_TYPE: &str = "finance.bank_account.update";
pub const FINANCE_BANK_ACCOUNT_DELETE_COMMAND_TYPE: &str = "finance.bank_account.delete";
pub const FINANCE_BANK_ACCOUNT_READ_COMMAND_TYPE: &str = "finance.bank_account.read";

// -- BankStatement (the per-account transaction log) --

pub const FINANCE_BANK_STATEMENT_READ_COMMAND_TYPE: &str = "finance.bank_statement.read";

// -- BankPaymentSlip (bank transfer / cheque slips) --

pub const FINANCE_BANK_SLIP_GENERATE_COMMAND_TYPE: &str = "finance.bank_slip.generate";
pub const FINANCE_BANK_SLIP_UPDATE_COMMAND_TYPE: &str = "finance.bank_slip.update";
pub const FINANCE_BANK_SLIP_APPROVE_COMMAND_TYPE: &str = "finance.bank_slip.approve";
pub const FINANCE_BANK_SLIP_READ_COMMAND_TYPE: &str = "finance.bank_slip.read";

// -- Payroll (HR-side payroll generation; finance records the payment) --

pub const FINANCE_PAYROLL_GENERATE_COMMAND_TYPE: &str = "finance.payroll.generate";
pub const FINANCE_PAYROLL_APPROVE_COMMAND_TYPE: &str = "finance.payroll.approve";
pub const FINANCE_PAYROLL_PAY_COMMAND_TYPE: &str = "finance.payroll.pay";
pub const FINANCE_PAYROLL_READ_COMMAND_TYPE: &str = "finance.payroll.read";

// -- PayrollPayment (finance-side accounting record for a payroll run) --

pub const FINANCE_PAYROLL_PAYMENT_RECORD_COMMAND_TYPE: &str = "finance.payroll_payment.record";
pub const FINANCE_PAYROLL_PAYMENT_APPROVE_COMMAND_TYPE: &str = "finance.payroll_payment.approve";
pub const FINANCE_PAYROLL_PAYMENT_PAY_COMMAND_TYPE: &str = "finance.payroll_payment.pay";
pub const FINANCE_PAYROLL_PAYMENT_READ_COMMAND_TYPE: &str = "finance.payroll_payment.read";

// -- FeesCarryForward (end-of-year balance roll-over) --

pub const FINANCE_FEES_CARRY_FORWARD_EXECUTE_COMMAND_TYPE: &str =
    "finance.fees_carry_forward.execute";
pub const FINANCE_FEES_CARRY_FORWARD_READ_COMMAND_TYPE: &str = "finance.fees_carry_forward.read";
pub const FINANCE_FEES_CARRY_FORWARD_CONFIGURE_COMMAND_TYPE: &str =
    "finance.fees_carry_forward.configure";

// -- DueFeesLoginPrevent (login prevention for overdue students) --

pub const FINANCE_DUE_FEES_BLOCK_COMMAND_TYPE: &str = "finance.due_fees.block";
pub const FINANCE_DUE_FEES_UNBLOCK_COMMAND_TYPE: &str = "finance.due_fees.unblock";
pub const FINANCE_DUE_FEES_READ_COMMAND_TYPE: &str = "finance.due_fees.read";

// -- Reports (the 22 finance reports) --

pub const FINANCE_REPORT_FEES_COLLECTION_COMMAND_TYPE: &str = "finance.report.fees_collection.read";
pub const FINANCE_REPORT_OUTSTANDING_FEES_COMMAND_TYPE: &str =
    "finance.report.outstanding_fees.read";
pub const FINANCE_REPORT_EXPENSE_COMMAND_TYPE: &str = "finance.report.expense.read";
pub const FINANCE_REPORT_INCOME_COMMAND_TYPE: &str = "finance.report.income.read";
pub const FINANCE_REPORT_BANK_STATEMENT_COMMAND_TYPE: &str = "finance.report.bank_statement.read";
pub const FINANCE_REPORT_WALLET_BALANCE_COMMAND_TYPE: &str = "finance.report.wallet_balance.read";
pub const FINANCE_REPORT_PAYROLL_COMMAND_TYPE: &str = "finance.report.payroll.read";
pub const FINANCE_REPORT_PAYMENT_METHOD_COMMAND_TYPE: &str = "finance.report.payment_method.read";
pub const FINANCE_REPORT_FEES_DISCOUNT_COMMAND_TYPE: &str = "finance.report.fees_discount.read";
pub const FINANCE_REPORT_DUE_FEES_COMMAND_TYPE: &str = "finance.report.due_fees.read";
pub const FINANCE_REPORT_CLASS_WISE_COLLECTION_COMMAND_TYPE: &str =
    "finance.report.class_wise_collection.read";
pub const FINANCE_REPORT_DAILY_COLLECTION_COMMAND_TYPE: &str =
    "finance.report.daily_collection.read";
pub const FINANCE_REPORT_MONTHLY_COLLECTION_COMMAND_TYPE: &str =
    "finance.report.monthly_collection.read";
pub const FINANCE_REPORT_HEAD_WISE_EXPENSE_COMMAND_TYPE: &str =
    "finance.report.head_wise_expense.read";
pub const FINANCE_REPORT_HEAD_WISE_INCOME_COMMAND_TYPE: &str =
    "finance.report.head_wise_income.read";
pub const FINANCE_REPORT_CASH_FLOW_COMMAND_TYPE: &str = "finance.report.cash_flow.read";
pub const FINANCE_REPORT_PROFIT_LOSS_COMMAND_TYPE: &str = "finance.report.profit_loss.read";
pub const FINANCE_REPORT_BALANCE_SHEET_COMMAND_TYPE: &str = "finance.report.balance_sheet.read";
pub const FINANCE_REPORT_TRIAL_BALANCE_COMMAND_TYPE: &str = "finance.report.trial_balance.read";
pub const FINANCE_REPORT_LEDGER_COMMAND_TYPE: &str = "finance.report.ledger.read";
pub const FINANCE_REPORT_RECEIPT_COMMAND_TYPE: &str = "finance.report.receipt.read";
pub const FINANCE_REPORT_REFUND_COMMAND_TYPE: &str = "finance.report.refund.read";

// =============================================================================
// Re-exports of the canonical command shapes from services.rs
// =============================================================================

// =============================================================================
// Re-exports of the canonical command shapes from services.rs
//
// `ConfigureInvoiceNumberingCommand`, `DeductWalletCreditCommand`, and
// `RecordExpenseCommand` are NOT re-exported here because commands.rs now
// owns the canonical `pub struct` definitions (see the Cluster D catch-up
// block at the end of this file). The services.rs copies remain in place
// for the service-function parameter types so `crate::services::X` resolves
// correctly. External callers should import these three via `educore_finance`
// (the umbrella re-export in `lib.rs`).
// =============================================================================

pub use crate::services::{
    CreateWalletCommand, CreditWalletCommand, RecordPaymentCommand, RequestWalletRefundCommand,
};

// =============================================================================
// Command shapes — typed inputs for every (aggregate × action) pair
// =============================================================================

// -- FeesGroup --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFeesGroupCommand {
    pub tenant: TenantContext,
    pub fees_group_id: FeesGroupId,
    pub name: String, // FG I-1 + FG I-2 pinned
    pub description: Option<String>,
}


impl CreateFeesGroupCommand {
    /// The command type discriminator for the dispatcher's audit
    /// log / idempotency table.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_FEES_GROUP_CREATE_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesGroupCreate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateFeesGroupCommand {
    pub tenant: TenantContext,
    pub fees_group_id: FeesGroupId,
    /// Mutable description (NOT name — FG I-1 says name is the
    /// uniqueness anchor and is NOT mutable via update_metadata;
    /// changing the name requires retire + create-new).
    pub description: Option<String>,
}


impl UpdateFeesGroupCommand {
    /// The command type discriminator for the dispatcher's audit
    /// log / idempotency table.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_FEES_GROUP_UPDATE_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesGroupUpdate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteFeesGroupCommand {
    pub tenant: TenantContext,
    pub fees_group_id: FeesGroupId,
}


impl DeleteFeesGroupCommand {
    /// The command type discriminator for the dispatcher's audit
    /// log / idempotency table.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_FEES_GROUP_DELETE_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesGroupDelete]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFeesGroupCommand {
    pub tenant: TenantContext,
    pub fees_group_id: FeesGroupId,
}


impl ReadFeesGroupCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesGroupRead]
    }
}
// -- FeesType --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFeesTypeCommand {
    pub tenant: TenantContext,
    pub fees_group_id: FeesGroupId,
    pub name: String,
    pub description: Option<String>,
    pub amount_minor: i64,
    pub currency: Currency,
}


impl CreateFeesTypeCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesTypeCreate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateFeesTypeCommand {
    pub tenant: TenantContext,
    pub fees_type_id: FeesTypeId,
    pub name: Option<String>,
    pub description: Option<String>,
    pub amount_minor: Option<i64>,
}


impl UpdateFeesTypeCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesTypeUpdate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteFeesTypeCommand {
    pub tenant: TenantContext,
    pub fees_type_id: FeesTypeId,
}


impl DeleteFeesTypeCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesTypeDelete]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFeesTypeCommand {
    pub tenant: TenantContext,
    pub fees_type_id: FeesTypeId,
}


impl ReadFeesTypeCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesTypeRead]
    }
}
// -- FeesMaster --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFeesMasterCommand {
    pub tenant: TenantContext,
    pub fees_master_id: FeesMasterId,
    pub fees_group_id: FeesGroupId,
    pub class_id: crate::value_objects::ClassId,
    pub amount_minor: i64, // FM I-1
    pub currency: Currency,
    pub due_date: NaiveDate,
    pub name: String,
}


impl CreateFeesMasterCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesMasterCreate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateFeesMasterCommand {
    pub tenant: TenantContext,
    pub fees_master_id: FeesMasterId,
    pub amount_minor: Option<i64>,
    pub due_date: Option<NaiveDate>,
}


impl UpdateFeesMasterCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesMasterUpdate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteFeesMasterCommand {
    pub tenant: TenantContext,
    pub fees_master_id: FeesMasterId,
}


impl DeleteFeesMasterCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesMasterDelete]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetireFeesMasterCommand {
    pub tenant: TenantContext,
    pub fees_master_id: FeesMasterId,
}

impl RetireFeesMasterCommand {
    /// The command-type discriminator.
    pub const COMMAND_TYPE: &'static str = FINANCE_FEES_MASTER_RETIRE_COMMAND_TYPE;
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesMasterDelete]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFeesMasterCommand {
    pub tenant: TenantContext,
    pub fees_master_id: FeesMasterId,
}


impl ReadFeesMasterCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesMasterRead]
    }
}
// -- FeesDiscount --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFeesDiscountCommand {
    pub tenant: TenantContext,
    pub fees_discount_id: FeesDiscountId,
    pub fees_master_id: FeesMasterId,
    pub academic_year_id: AcademicYearId,
    pub name: String,
    pub discount_code: String,
    pub discount_type: DiscountType,
    pub description: Option<String>,
}


impl CreateFeesDiscountCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesDiscountCreate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateFeesDiscountCommand {
    pub tenant: TenantContext,
    pub fees_discount_id: FeesDiscountId,
    pub name: Option<String>,
    pub discount_code: Option<String>,
    pub discount_type: Option<DiscountType>,
    pub description: Option<String>,
}


impl UpdateFeesDiscountCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesDiscountUpdate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteFeesDiscountCommand {
    pub tenant: TenantContext,
    pub fees_discount_id: FeesDiscountId,
}


impl DeleteFeesDiscountCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesDiscountDelete]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFeesDiscountCommand {
    pub tenant: TenantContext,
    pub fees_discount_id: FeesDiscountId,
}


impl ReadFeesDiscountCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesDiscountRead]
    }
}
// -- FeesAssign --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFeesAssignCommand {
    pub tenant: TenantContext,
    pub student_id: educore_academic::StudentId,
    pub fees_master_id: FeesMasterId,
    pub amount_minor: i64,
    pub currency: Currency,
    pub due_date: NaiveDate,
}


impl CreateFeesAssignCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesAssignCreate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateFeesAssignCommand {
    pub tenant: TenantContext,
    pub fees_assign_id: FeesAssignId,
    pub amount_minor: Option<i64>,
    pub due_date: Option<NaiveDate>,
}


impl UpdateFeesAssignCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesAssignUpdate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteFeesAssignCommand {
    pub tenant: TenantContext,
    pub fees_assign_id: FeesAssignId,
}


impl DeleteFeesAssignCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesAssignClose]
    }
}
// -- FeesInstallment --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFeesInstallmentCommand {
    pub tenant: TenantContext,
    pub fees_master_id: FeesMasterId,
    pub name: String,
    pub due_date: NaiveDate,
    pub amount_minor: i64,
    pub currency: Currency,
}


impl CreateFeesInstallmentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesInstallmentCreate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateFeesInstallmentCommand {
    pub tenant: TenantContext,
    pub fees_installment_id: FeesInstallmentId,
    pub name: Option<String>,
    pub due_date: Option<NaiveDate>,
    pub amount_minor: Option<i64>,
}


impl UpdateFeesInstallmentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesInstallmentUpdate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteFeesInstallmentCommand {
    pub tenant: TenantContext,
    pub fees_installment_id: FeesInstallmentId,
}


impl DeleteFeesInstallmentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesInstallmentDelete]
    }
}
// -- DirectFeesInstallment --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateDirectFeesInstallmentCommand {
    pub tenant: TenantContext,
    pub direct_fees_installment_id: DirectFeesInstallmentId,
    pub student_id: educore_academic::StudentId,
    pub name: String,
    pub amount_minor: i64,
    pub currency: Currency,
    pub due_date: NaiveDate,
    pub percentage_minor: i64, // DFI I-3
    pub window_start: Option<NaiveDate>, // DFI I-4
    pub window_end: Option<NaiveDate>, // DFI I-4
}


impl CreateDirectFeesInstallmentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceDirectFeesInstallmentCreate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateDirectFeesInstallmentCommand {
    pub tenant: TenantContext,
    pub direct_fees_installment_id: DirectFeesInstallmentId,
    pub name: Option<String>,
    pub amount_minor: Option<i64>,
    pub due_date: Option<NaiveDate>,
}


impl UpdateDirectFeesInstallmentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceDirectFeesInstallmentUpdate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteDirectFeesInstallmentCommand {
    pub tenant: TenantContext,
    pub direct_fees_installment_id: DirectFeesInstallmentId,
}


impl DeleteDirectFeesInstallmentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceDirectFeesInstallmentDelete]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadDirectFeesInstallmentCommand {
    pub tenant: TenantContext,
    pub direct_fees_installment_id: DirectFeesInstallmentId,
}


impl ReadDirectFeesInstallmentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceDirectFeesInstallmentRead]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetireDirectFeesInstallmentCommand {
    pub tenant: TenantContext,
    pub direct_fees_installment_id: DirectFeesInstallmentId,
}

impl RetireDirectFeesInstallmentCommand {
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceDirectFeesInstallmentDelete]
    }
}

/// Stable command type identifier for [`RetireDirectFeesInstallmentCommand`].
/// (FINANCE_DIRECT_FEES_INSTALLMENT_CREATE/UPDATE/DELETE/READ_COMMAND_TYPE
/// are already declared earlier in this file as part of the
/// pre-existing Phase 7 DirectFeesInstallment command set.)
pub const FINANCE_DIRECT_FEES_INSTALLMENT_RETIRE_COMMAND_TYPE: &str =
    "finance.direct_fees_installment.retire";
// -- DirectFeesInstallmentAssign (per-student linkage) --


// -- DirectFeesInstallmentAssign (Wave 103 — RealDirectFeesInstallmentAssign) --

/// COMMAND_TYPE discriminator for `CreateDirectFeesInstallmentAssignCommand`.
pub const FINANCE_DIRECT_FEES_INSTALLMENT_ASSIGN_CREATE_COMMAND_TYPE: &str =
    "finance.direct_fees_installment_assign.create";

/// COMMAND_TYPE discriminator for `ReadDirectFeesInstallmentAssignCommand`.
pub const FINANCE_DIRECT_FEES_INSTALLMENT_ASSIGN_READ_COMMAND_TYPE: &str =
    "finance.direct_fees_installment_assign.read";

/// COMMAND_TYPE discriminator for `RetireDirectFeesInstallmentAssignCommand`.
pub const FINANCE_DIRECT_FEES_INSTALLMENT_ASSIGN_RETIRE_COMMAND_TYPE: &str =
    "finance.direct_fees_installment_assign.retire";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateDirectFeesInstallmentAssignCommand {
    pub tenant: TenantContext,
    pub direct_fees_installment_assign_id: DirectFeesInstallmentAssignId,
    pub student_id: StudentId,
    pub installment_id: DirectFeesInstallmentId,
    pub amount_minor: i64, // DFIA I-2
    pub balance_minor: i64, // DFIA I-3
}

impl CreateDirectFeesInstallmentAssignCommand {
    /// The command-type discriminator.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_DIRECT_FEES_INSTALLMENT_ASSIGN_CREATE_COMMAND_TYPE;
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceDirectFeesInstallmentCreate]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadDirectFeesInstallmentAssignCommand {
    pub tenant: TenantContext,
    pub direct_fees_installment_assign_id: DirectFeesInstallmentAssignId,
}

impl ReadDirectFeesInstallmentAssignCommand {
    /// The command-type discriminator.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_DIRECT_FEES_INSTALLMENT_ASSIGN_READ_COMMAND_TYPE;
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceDirectFeesInstallmentCreate]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetireDirectFeesInstallmentAssignCommand {
    pub tenant: TenantContext,
    pub direct_fees_installment_assign_id: DirectFeesInstallmentAssignId,
}

impl RetireDirectFeesInstallmentAssignCommand {
    /// The command-type discriminator.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_DIRECT_FEES_INSTALLMENT_ASSIGN_RETIRE_COMMAND_TYPE;
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceDirectFeesInstallmentDelete]
    }
}

/// Command: append a new child row under an existing
/// [`DirectFeesInstallmentAssign`] aggregate (per v3 Part 2 F12 +
/// checklist § DirectFeesInstallmentAssignChild). The child row
/// represents one installment in the per-installment breakdown (amount
/// + due date). DFIAC I-1 (append-only) and DFIAC I-2 (timestamps
/// monotonic) are enforced by the service function
/// `create_direct_fees_installment_assign_child` in `services.rs`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateDirectFeesInstallmentAssignChildCommand {
    pub tenant: TenantContext,
    pub direct_fees_installment_assign_id: DirectFeesInstallmentAssignId,
    pub amount_minor: i64,
}

impl CreateDirectFeesInstallmentAssignChildCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceDirectFeesInstallmentAssign]
    }
}
// -- DirectFeesSetting (per-school direct-fees configuration) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateDirectFeesSettingCommand {
    pub tenant: TenantContext,
    pub enabled: bool,
    pub reminder_before: i64,
    pub no_installment: i64,
    pub due_date_from_sem: u8,
    pub description: Option<String>,
}


impl CreateDirectFeesSettingCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateDirectFeesSettingCommand {
    pub tenant: TenantContext,
    pub direct_fees_setting_id: DirectFeesSettingId,
    pub enabled: Option<bool>,
    pub description: Option<String>,
}


impl UpdateDirectFeesSettingCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteDirectFeesSettingCommand {
    pub tenant: TenantContext,
    pub direct_fees_setting_id: DirectFeesSettingId,
}


impl DeleteDirectFeesSettingCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
// -- DirectFeesReminder --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateDirectFeesReminderCommand {
    pub tenant: TenantContext,
    pub direct_fees_reminder_id: DirectFeesReminderId,
    pub direct_fees_installment_id: DirectFeesInstallmentId,
    pub student_id: educore_academic::StudentId,
    pub remind_at: NaiveDate,
    /// How many days BEFORE the installment due_date to fire
    /// the reminder. Must be >= 0. DFR I-1.
    pub due_date_before_days: i64,
    pub note: Option<String>,
}


impl CreateDirectFeesReminderCommand {
    /// The command type discriminator for the dispatcher's audit
    /// log / idempotency table.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_DIRECT_FEES_REMINDER_CREATE_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesReminderConfigure]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateDirectFeesReminderCommand {
    pub tenant: TenantContext,
    pub direct_fees_reminder_id: DirectFeesReminderId,
    pub remind_at: Option<NaiveDate>,
    /// Optional override for the days-before-due field. DFR I-1
    /// (>= 0) is validated in the service function.
    pub due_date_before_days: Option<i64>,
    pub note: Option<String>,
}


impl UpdateDirectFeesReminderCommand {
    /// The command type discriminator for the dispatcher's audit
    /// log / idempotency table.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_DIRECT_FEES_REMINDER_UPDATE_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesReminderUpdate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteDirectFeesReminderCommand {
    pub tenant: TenantContext,
    pub direct_fees_reminder_id: DirectFeesReminderId,
}


impl DeleteDirectFeesReminderCommand {
    /// The command type discriminator for the dispatcher's audit
    /// log / idempotency table.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_DIRECT_FEES_REMINDER_DELETE_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesReminderDelete]
    }
}
// -- PaymentMethod --

/// Stable command type identifier for [`RetirePaymentMethodCommand`].
/// (FINANCE_PAYMENT_METHOD_CREATE/UPDATE/DELETE/READ_COMMAND_TYPE
/// are already declared earlier in this file as part of the
/// pre-existing Phase 7 PaymentMethod command set.)
pub const FINANCE_PAYMENT_METHOD_RETIRE_COMMAND_TYPE: &str = "finance.payment_method.retire";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreatePaymentMethodCommand {
    pub tenant: TenantContext,
    pub payment_method_id: PaymentMethodId,
    pub name: String,
    pub kind: PaymentMethodKind,
    pub description: Option<String>,
}


impl CreatePaymentMethodCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinancePaymentMethodCreate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetirePaymentMethodCommand {
    pub tenant: TenantContext,
    pub payment_method_id: PaymentMethodId,
}


impl RetirePaymentMethodCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinancePaymentMethodCreate]
    }
}


// -- Wave 107 -- FeesInstallmentAssign (per-(fees_assign, installment) linkage) --

pub const FINANCE_FEES_INSTALLMENT_ASSIGN_CREATE_COMMAND_TYPE: &str = "finance.fees_installment_assign.create";
pub const FINANCE_FEES_INSTALLMENT_ASSIGN_READ_COMMAND_TYPE: &str = "finance.fees_installment_assign.read";
pub const FINANCE_FEES_INSTALLMENT_ASSIGN_RETIRE_COMMAND_TYPE: &str = "finance.fees_installment_assign.retire";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFeesInstallmentAssignCommand {
    pub tenant: TenantContext,
    pub fees_installment_assign_id: FeesInstallmentAssignId,
    pub fees_assign_id: FeesAssignId,
    pub fees_installment_id: FeesInstallmentId,
    pub due_date: chrono::NaiveDate,
    pub amount_minor: i64,
    pub discount_minor: i64,
    pub paid_amount_minor: i64,
    pub note: Option<String>,
}

impl CreateFeesInstallmentAssignCommand {
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesAssignCreate]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFeesInstallmentAssignCommand {
    pub tenant: TenantContext,
    pub fees_installment_assign_id: FeesInstallmentAssignId,
}

impl ReadFeesInstallmentAssignCommand {
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesAssignRead]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetireFeesInstallmentAssignCommand {
    pub tenant: TenantContext,
    pub fees_installment_assign_id: FeesInstallmentAssignId,
}

impl RetireFeesInstallmentAssignCommand {
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesAssignCreate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdatePaymentMethodCommand {
    pub tenant: TenantContext,
    pub payment_method_id: PaymentMethodId,
    pub name: Option<String>,
    pub description: Option<String>,
}


impl UpdatePaymentMethodCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinancePaymentMethodUpdate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeletePaymentMethodCommand {
    pub tenant: TenantContext,
    pub payment_method_id: PaymentMethodId,
}


impl DeletePaymentMethodCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinancePaymentMethodDelete]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadPaymentMethodCommand {
    pub tenant: TenantContext,
    pub payment_method_id: PaymentMethodId,
}


impl ReadPaymentMethodCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinancePaymentMethodRead]
    }
}
// -- PaymentGateway --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreatePaymentGatewayCommand {
    pub tenant: TenantContext,
    pub name: String,
    pub api_key: String,
    pub api_secret: String,
    pub mode: GatewayMode,
    pub description: Option<String>,
}


impl CreatePaymentGatewayCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinancePaymentGatewayConfigure]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdatePaymentGatewayCommand {
    pub tenant: TenantContext,
    pub payment_gateway_setting_id: PaymentGatewaySettingId,
    pub name: Option<String>,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub mode: Option<GatewayMode>,
    pub description: Option<String>,
}


impl UpdatePaymentGatewayCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinancePaymentGatewayUpdate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeletePaymentGatewayCommand {
    pub tenant: TenantContext,
    pub payment_gateway_setting_id: PaymentGatewaySettingId,
}


impl DeletePaymentGatewayCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinancePaymentGatewayDisable]
    }
}
// -- FeesInvoice (Generate / Update / Cancel / Read) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerateInvoiceCommand {
    pub tenant: TenantContext,
    pub student_id: educore_academic::StudentId,
    pub academic_year_id: educore_academic::AcademicYearId,
    pub due_date: NaiveDate,
    pub amount_minor: i64,
    pub currency: Currency,
}


impl GenerateInvoiceCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateInvoiceCommand {
    pub tenant: TenantContext,
    pub fees_invoice_id: FeesInvoiceId,
    pub due_date: Option<NaiveDate>,
    pub amount_minor: Option<i64>,
    pub note: Option<String>,
}


impl UpdateInvoiceCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceUpdate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CancelInvoiceCommand {
    pub tenant: TenantContext,
    pub fees_invoice_id: FeesInvoiceId,
    pub reason: String,
}


impl CancelInvoiceCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadInvoiceCommand {
    pub tenant: TenantContext,
    pub fees_invoice_id: FeesInvoiceId,
}


impl ReadInvoiceCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
// -- FeesPayment (Reverse / Refund / Read) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReversePaymentCommand {
    pub tenant: TenantContext,
    pub fees_payment_id: FeesPaymentId,
    pub reason: String,
}


impl ReversePaymentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinancePaymentReverse]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RefundPaymentCommand {
    pub tenant: TenantContext,
    pub fees_payment_id: FeesPaymentId,
    pub amount_minor: i64,
    pub currency: Currency,
    pub reason: String,
}


impl RefundPaymentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinancePaymentRefund]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFeesPaymentCommand {
    pub tenant: TenantContext,
    pub fees_payment_id: FeesPaymentId,
}


impl ReadFeesPaymentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinancePaymentRead]
    }
}
// -- Expense (Update / Delete / Approve) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateExpenseCommand {
    pub tenant: TenantContext,
    pub expense_id: ExpenseId,
    pub name: Option<String>,
    pub amount_minor: Option<i64>,
    pub expense_head_id: Option<ExpenseHeadId>,
    pub account_id: Option<BankAccountId>,
    pub expense_date: Option<NaiveDate>,
    pub description: Option<String>,
}


impl UpdateExpenseCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceExpenseUpdate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteExpenseCommand {
    pub tenant: TenantContext,
    pub expense_id: ExpenseId,
}


impl DeleteExpenseCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceExpenseDelete]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApproveExpenseCommand {
    pub tenant: TenantContext,
    pub expense_id: ExpenseId,
    pub approver_user_id: UserId,
    pub note: Option<String>,
}


impl ApproveExpenseCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceExpenseApprove]
    }
}
// -- Income (Create / Update / Delete / Approve) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateIncomeCommand {
    pub tenant: TenantContext,
    pub name: String,
    pub amount_minor: i64,
    pub currency: Currency,
    pub income_head_id: IncomeHeadId,
    pub account_id: BankAccountId,
    pub income_date: NaiveDate,
    pub description: Option<String>,
    pub donor_id: Option<DonorId>,
}

/// COMMAND_TYPE discriminator for `ReadIncomeCommand`.
pub const FINANCE_INCOME_READ_COMMAND_TYPE: &str = "finance.income.read";

/// COMMAND_TYPE discriminator for `RetireIncomeCommand`.
pub const FINANCE_INCOME_RETIRE_COMMAND_TYPE: &str = "finance.income.retire";

impl CreateIncomeCommand {
    /// The command-type discriminator.
    pub const COMMAND_TYPE: &'static str = FINANCE_INCOME_CREATE_COMMAND_TYPE;
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceIncomeCreate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateIncomeCommand {
    pub tenant: TenantContext,
    pub income_id: IncomeId,
    pub name: Option<String>,
    pub amount_minor: Option<i64>,
    pub income_head_id: Option<IncomeHeadId>,
    pub account_id: Option<BankAccountId>,
    pub income_date: Option<NaiveDate>,
    pub description: Option<String>,
}


impl UpdateIncomeCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceIncomeUpdate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteIncomeCommand {
    pub tenant: TenantContext,
    pub income_id: IncomeId,
}


impl DeleteIncomeCommand {
    /// The command-type discriminator.
    pub const COMMAND_TYPE: &'static str = FINANCE_INCOME_RETIRE_COMMAND_TYPE;
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceIncomeDelete]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadIncomeCommand {
    pub tenant: TenantContext,
    pub income_id: IncomeId,
}

impl ReadIncomeCommand {
    /// The command-type discriminator.
    pub const COMMAND_TYPE: &'static str = FINANCE_INCOME_READ_COMMAND_TYPE;
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceIncomeRead]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetireIncomeCommand {
    pub tenant: TenantContext,
    pub income_id: IncomeId,
}

impl RetireIncomeCommand {
    /// The command-type discriminator.
    pub const COMMAND_TYPE: &'static str = FINANCE_INCOME_RETIRE_COMMAND_TYPE;
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceIncomeDelete]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApproveIncomeCommand {
    pub tenant: TenantContext,
    pub income_id: IncomeId,
    pub approver_user_id: UserId,
    pub note: Option<String>,
}


impl ApproveIncomeCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceIncomeApprove]
    }
}
// -- ExpenseHead (Update / Delete) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateExpenseHeadCommand {
    pub tenant: TenantContext,
    pub expense_head_id: ExpenseHeadId,
    /// Mutable description (NOT name — EH I-1 says name is the
    /// uniqueness anchor and is NOT mutable via update_metadata;
    /// changing the name requires retire + create-new).
    pub description: Option<String>,
}


impl UpdateExpenseHeadCommand {
    /// The command type discriminator for the dispatcher's audit
    /// log / idempotency table.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_EXPENSE_HEAD_UPDATE_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceExpenseHeadUpdate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteExpenseHeadCommand {
    pub tenant: TenantContext,
    pub expense_head_id: ExpenseHeadId,
}


impl DeleteExpenseHeadCommand {
    /// The command type discriminator for the dispatcher's audit
    /// log / idempotency table.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_EXPENSE_HEAD_DELETE_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceExpenseHeadDelete]
    }
}
// -- IncomeHead (Create / Update / Delete) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateIncomeHeadCommand {
    pub tenant: TenantContext,
    pub name: String,
    pub description: Option<String>,
}


impl CreateIncomeHeadCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceIncomeHeadCreate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateIncomeHeadCommand {
    pub tenant: TenantContext,
    pub income_head_id: IncomeHeadId,
    pub name: Option<String>,
    pub description: Option<String>,
}


impl UpdateIncomeHeadCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceIncomeHeadUpdate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteIncomeHeadCommand {
    pub tenant: TenantContext,
    pub income_head_id: IncomeHeadId,
}


impl DeleteIncomeHeadCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceIncomeHeadDelete]
    }
}
// -- BankAccount (Open / Update / Delete / Read) --

/// Command: open (create) a new `RealBankAccount` ledger entry.
///
/// Carries the immutable fields (BA I-1 account_number + BA I-2
/// opening_balance_minor + BA I-3 account_type + currency) + the
/// mutable metadata (account_name + bank_name + ifsc_code + branch
/// + description).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenBankAccountCommand {
    pub tenant: TenantContext,
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
}


impl OpenBankAccountCommand {
    /// The command type discriminator for the dispatcher's audit
    /// log / idempotency table.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_BANK_ACCOUNT_OPEN_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceBankOpen]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateBankAccountCommand {
    pub tenant: TenantContext,
    pub bank_account_id: BankAccountId,
    pub account_name: String,
    pub bank_name: String,
    pub ifsc_code: Option<String>,
    pub branch: Option<String>,
    pub description: Option<String>,
}


impl UpdateBankAccountCommand {
    /// The command type discriminator for the dispatcher's audit
    /// log / idempotency table.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_BANK_ACCOUNT_UPDATE_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceBankUpdate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteBankAccountCommand {
    pub tenant: TenantContext,
    pub bank_account_id: BankAccountId,
}


impl DeleteBankAccountCommand {
    /// The command type discriminator for the dispatcher's audit
    /// log / idempotency table.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_BANK_ACCOUNT_DELETE_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceBankClose]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadBankAccountCommand {
    pub tenant: TenantContext,
    pub bank_account_id: BankAccountId,
}


impl ReadBankAccountCommand {
    /// The command type discriminator for the dispatcher's audit
    /// log / idempotency table.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_BANK_ACCOUNT_READ_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceBankRead]
    }
}
// -- BankStatement (Read) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadBankStatementCommand {
    pub tenant: TenantContext,
    pub bank_statement_id: BankStatementId,
}


impl ReadBankStatementCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
// -- BankPaymentSlip (Generate / Update / Approve / Read) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerateBankSlipCommand {
    pub tenant: TenantContext,
    pub bank_account_id: BankAccountId,
    pub amount_minor: i64,
    pub currency: Currency,
    pub mode: BankMode,
    pub slip_date: NaiveDate,
    pub note: Option<String>,
    pub payee_name: Option<String>,
}


impl GenerateBankSlipCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceBankSlipGenerate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateBankSlipCommand {
    pub tenant: TenantContext,
    pub bank_payment_slip_id: BankPaymentSlipId,
    pub amount_minor: Option<i64>,
    pub slip_date: Option<NaiveDate>,
    pub note: Option<String>,
    pub payee_name: Option<String>,
}


impl UpdateBankSlipCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceBankSlipRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApproveBankSlipCommand {
    pub tenant: TenantContext,
    pub bank_payment_slip_id: BankPaymentSlipId,
    pub approver_user_id: UserId,
    pub note: Option<String>,
}


impl ApproveBankSlipCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceBankSlipApprove]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadBankSlipCommand {
    pub tenant: TenantContext,
    pub bank_payment_slip_id: BankPaymentSlipId,
}


impl ReadBankSlipCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceBankSlipRead]
    }
}
// -- Payroll (Generate / Approve / Pay / Read) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneratePayrollCommand {
    pub tenant: TenantContext,
    pub payroll_generate_id: educore_hr::value_objects::PayrollGenerateId,
    pub note: Option<String>,
}


impl GeneratePayrollCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApprovePayrollCommand {
    pub tenant: TenantContext,
    pub payroll_generate_id: educore_hr::value_objects::PayrollGenerateId,
    pub approver_user_id: UserId,
    pub note: Option<String>,
}


impl ApprovePayrollCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayPayrollCommand {
    pub tenant: TenantContext,
    pub payroll_payment_id: PayrollPaymentId,
    pub account_id: BankAccountId,
    pub amount_minor: i64,
    pub currency: Currency,
    pub payment_date: NaiveDate,
}


impl PayPayrollCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadPayrollCommand {
    pub tenant: TenantContext,
    pub payroll_generate_id: educore_hr::value_objects::PayrollGenerateId,
}


impl ReadPayrollCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
// -- PayrollPayment (Approve / Pay / Read) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApprovePayrollPaymentCommand {
    pub tenant: TenantContext,
    pub payroll_payment_id: PayrollPaymentId,
    pub approver_user_id: UserId,
    pub note: Option<String>,
}


impl ApprovePayrollPaymentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinancePaymentRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayPayrollPaymentCommand {
    pub tenant: TenantContext,
    pub payroll_payment_id: PayrollPaymentId,
    pub account_id: BankAccountId,
    pub amount_minor: i64,
    pub currency: Currency,
    pub payment_date: NaiveDate,
}


impl PayPayrollPaymentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinancePaymentRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadPayrollPaymentCommand {
    pub tenant: TenantContext,
    pub payroll_payment_id: PayrollPaymentId,
}


impl ReadPayrollPaymentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinancePaymentRead]
    }
}
// -- Wallet (Read) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadWalletCommand {
    pub tenant: TenantContext,
    pub wallet_id: WalletId,
}


impl ReadWalletCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceWalletRead]
    }
}
// -- WalletTransaction (Approve / Reject / Read) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApproveWalletTransactionCommand {
    pub tenant: TenantContext,
    pub wallet_transaction_id: WalletTransactionId,
    pub approver_user_id: UserId,
}


impl ApproveWalletTransactionCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceWalletApprove]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RejectWalletTransactionCommand {
    pub tenant: TenantContext,
    pub wallet_transaction_id: WalletTransactionId,
    pub rejecter_user_id: UserId,
    pub reason: String,
}


impl RejectWalletTransactionCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceWalletReject]
    }
}

/// Command: create a new `WalletTransactionApproval` child row in
/// the initial pending state. Dispatched by the
/// `create_wallet_transaction_approval` service function in
/// `services.rs`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateWalletTransactionApprovalCommand {
    pub tenant: TenantContext,
    pub wallet_transaction_approval_id: WalletTransactionApprovalId,
    pub wallet_transaction_id: WalletTransactionId,
}

impl CreateWalletTransactionApprovalCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceWalletApprove]
    }
}

/// Command: transition a `WalletTransactionApproval` row from
/// `pending` to `approved`. Dispatched by the
/// `approve_wallet_transaction_approval` service function in
/// `services.rs`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApproveWalletTransactionApprovalCommand {
    pub tenant: TenantContext,
    pub wallet_transaction_approval_id: WalletTransactionApprovalId,
    pub approver_user_id: UserId,
}

impl ApproveWalletTransactionApprovalCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceWalletApprove]
    }
}

/// Command: transition a `WalletTransactionApproval` row from
/// `pending` to `rejected` with a required reason note (per WTA I-2).
/// Dispatched by the `reject_wallet_transaction_approval` service
/// function in `services.rs`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RejectWalletTransactionApprovalCommand {
    pub tenant: TenantContext,
    pub wallet_transaction_approval_id: WalletTransactionApprovalId,
    pub rejecter_user_id: UserId,
    pub reason: String,
}

impl RejectWalletTransactionApprovalCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceWalletReject]
    }
}

// -----------------------------------------------------------------------------
// ExpenseApproval commands (Wave 79 — per-aggregate wave pattern from
// Waves 65–78)
// -----------------------------------------------------------------------------
//
// Per v3 Part 2 F20 + checklist § ExpenseApproval: 2 invariants:
//   - EA I-1: state machine pending → approved/rejected.
//   - EA I-2: timestamps recorded (every transition stamps
//             decided_by + decided_at; reject also captures reason).
//
// Three commands: Create (enter Pending), Approve (Pending→Approved),
// Reject (Pending→Rejected with optional reason).

/// Command: create a new `ExpenseApproval` row in the Pending state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateExpenseApprovalCommand {
    pub tenant: TenantContext,
    pub expense_approval_id: ExpenseApprovalId,
    pub expense_id: ExpenseId,
    pub requested_by: UserId,
}

impl CreateExpenseApprovalCommand {
    #[must_use]
    pub const fn required_capabilities() -> &'static [Capability] {
        // Fm-prefix / EA-prefix RBAC variants don't exist yet; fall
        // back to the closest existing capability (Wave 72/75/77
        // precedent). To be revisited in a future RBAC revision (v3
        // Part 6).
        &[Capability::FinanceExpenseApprove]
    }
}

/// Command: transition an `ExpenseApproval` from Pending to Approved.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApproveExpenseApprovalCommand {
    pub tenant: TenantContext,
    pub expense_approval_id: ExpenseApprovalId,
}

impl ApproveExpenseApprovalCommand {
    #[must_use]
    pub const fn required_capabilities() -> &'static [Capability] {
        &[Capability::FinanceExpenseApprove]
    }
}

/// Command: transition an `ExpenseApproval` from Pending to Rejected
/// with an optional reason string.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RejectExpenseApprovalCommand {
    pub tenant: TenantContext,
    pub expense_approval_id: ExpenseApprovalId,
    pub reason: Option<String>,
}

impl RejectExpenseApprovalCommand {
    #[must_use]
    pub const fn required_capabilities() -> &'static [Capability] {
        &[Capability::FinanceExpenseApprove]
    }
}

// -----------------------------------------------------------------------------
// IncomeApproval commands (Wave 80 — per-aggregate wave pattern from
// Waves 65–79)
// -----------------------------------------------------------------------------
//
// Per v3 Part 2 F28 + checklist § IncomeApproval: 2 invariants:
//   - IA I-1: state machine pending → approved/rejected.
//   - IA I-2: timestamps recorded (every transition stamps
//             decided_by + decided_at; reject also captures reason).
//
// Structurally identical to the Wave 79 ExpenseApproval commands
// with the parent reference renamed from `expense_id` to
// `income_id` and the RBAC capability switched from
// FinanceExpenseApprove to FinanceIncomeApprove (existing variant
// at `crates/cross-cutting/rbac/src/value_objects.rs:345`).
//
// Three commands: Create (enter Pending), Approve (Pending→Approved),
// Reject (Pending→Rejected with optional reason).

/// Command: create a new `IncomeApproval` row in the Pending state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateIncomeApprovalCommand {
    pub tenant: TenantContext,
    pub income_approval_id: IncomeApprovalId,
    pub income_id: IncomeId,
    pub requested_by: UserId,
}

impl CreateIncomeApprovalCommand {
    #[must_use]
    pub const fn required_capabilities() -> &'static [Capability] {
        &[Capability::FinanceIncomeApprove]
    }
}

/// Command: transition an `IncomeApproval` from Pending to Approved.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApproveIncomeApprovalCommand {
    pub tenant: TenantContext,
    pub income_approval_id: IncomeApprovalId,
}

impl ApproveIncomeApprovalCommand {
    #[must_use]
    pub const fn required_capabilities() -> &'static [Capability] {
        &[Capability::FinanceIncomeApprove]
    }
}

/// Command: transition an `IncomeApproval` from Pending to Rejected
/// with an optional reason string.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RejectIncomeApprovalCommand {
    pub tenant: TenantContext,
    pub income_approval_id: IncomeApprovalId,
    pub reason: Option<String>,
}

impl RejectIncomeApprovalCommand {
    #[must_use]
    pub const fn required_capabilities() -> &'static [Capability] {
        &[Capability::FinanceIncomeApprove]
    }
}

// -----------------------------------------------------------------------------
// PayrollPaymentApproval commands (Wave 81 — per-aggregate wave pattern
// from Waves 65–80)
// -----------------------------------------------------------------------------
//
// Per v3 Part 2 F44 + checklist § PayrollPaymentApproval: 2 invariants:
//   - PPA I-1: state machine pending → approved/rejected.
//   - PPA I-2: timestamps recorded (every transition stamps
//             approver_id + approved_at; reject also captures
//             rejecter_id + rejected_at + rejection_reason).
//
// Structurally identical to the Wave 79 ExpenseApproval and
// Wave 80 IncomeApproval commands, but the
// PayrollPaymentApproval aggregate does NOT have its own id field
// (parent payroll_payment_id is de-facto identity) so the commands
// reference payroll_payment_id directly.
//
// Three commands: Create (enter Pending), Approve (Pending→Approved),
// Reject (Pending→Rejected with optional reason). RBAC fallback:
// Capability::FinancePayrollPaymentApprove does NOT exist; use the
// closest existing variant (FinancePayrollPaymentRecord). To be
// revisited in a future RBAC revision (v3 Part 6).

/// Command: create a new `PayrollPaymentApproval` row in the Pending
/// state. The PayrollPayment aggregate's create flow typically mints
/// the approval inline (see entities.rs:499); this command is for
/// the dispatcher-level explicit create path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreatePayrollPaymentApprovalCommand {
    pub tenant: TenantContext,
    pub payroll_payment_id: PayrollPaymentId,
}

impl CreatePayrollPaymentApprovalCommand {
    #[must_use]
    pub const fn required_capabilities() -> &'static [Capability] {
        // Wave 72/75/77/78/81 Fm-prefix / approval-prefix RBAC
        // fallback: FinancePayrollPaymentApprove does not exist;
        // closest semantic match is FinancePayrollPaymentRecord.
        &[Capability::FinancePayrollPaymentRecord]
    }
}

/// Command: transition a `PayrollPaymentApproval` from Pending to
/// Approved.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApprovePayrollPaymentApprovalCommand {
    pub tenant: TenantContext,
    pub payroll_payment_id: PayrollPaymentId,
}

impl ApprovePayrollPaymentApprovalCommand {
    #[must_use]
    pub const fn required_capabilities() -> &'static [Capability] {
        // Same fallback as Create.
        &[Capability::FinancePayrollPaymentRecord]
    }
}

/// Command: transition a `PayrollPaymentApproval` from Pending to
/// Rejected with an optional reason string.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RejectPayrollPaymentApprovalCommand {
    pub tenant: TenantContext,
    pub payroll_payment_id: PayrollPaymentId,
    pub reason: Option<String>,
}

impl RejectPayrollPaymentApprovalCommand {
    #[must_use]
    pub const fn required_capabilities() -> &'static [Capability] {
        // Same fallback as Create.
        &[Capability::FinancePayrollPaymentRecord]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadWalletTransactionCommand {
    pub tenant: TenantContext,
    pub wallet_transaction_id: WalletTransactionId,
}


impl ReadWalletTransactionCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceWalletRead]
    }
}
// -- FeesCarryForward (Read / Configure) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFeesCarryForwardCommand {
    pub tenant: TenantContext,
    pub fees_carry_forward_id: FeesCarryForwardId,
}


impl ReadFeesCarryForwardCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesCarryForwardRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigureFeesCarryForwardCommand {
    pub tenant: TenantContext,
    pub fees_carry_forward_setting_id: FeesCarryForwardSettingId,
    pub enabled: bool,
    pub description: Option<String>,
}


impl ConfigureFeesCarryForwardCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesCarryForwardConfigure]
    }
}


// -- Wave 113 -- FeesCarryForward (greenfield commands) --

pub const FINANCE_FEES_CARRY_FORWARD_CREATE_COMMAND_TYPE: &str = "finance.fees_carry_forward.create";
pub const FINANCE_FEES_CARRY_FORWARD_RETIRE_COMMAND_TYPE: &str = "finance.fees_carry_forward.retire";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFeesCarryForwardCommand {
    pub tenant: TenantContext,
    pub fees_carry_forward_id: FeesCarryForwardId,
    pub student_id: educore_academic::StudentId,
    pub academic_year_id: educore_academic::AcademicYearId,
    pub balance_minor: i64,
    pub balance_type: BalanceType,
    pub currency: Currency,
}

impl CreateFeesCarryForwardCommand {
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesCarryForwardConfigure]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetireFeesCarryForwardCommand {
    pub tenant: TenantContext,
    pub fees_carry_forward_id: FeesCarryForwardId,
}

impl RetireFeesCarryForwardCommand {
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesCarryForwardConfigure]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFeesCarryForwardLogCommand {
    pub tenant: TenantContext,
    pub student_id: StudentId,
    pub academic_year_id: AcademicYearId,
    pub amount_minor: i64,
    pub description: Option<String>,
}

/// Command: create a new per-school `FeesCarryForwardSetting` (per
/// v3 Part 2 F34 + checklist § FeesCarryForwardSetting). Dispatched
/// by the `create_fees_carry_forward_setting` service function in
/// `services.rs`. Enforces FCFA I-1 (per-school scoping via the
/// typed id) and FCFA I-2 (`threshold_minor >= 0`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFeesCarryForwardSettingCommand {
    pub tenant: TenantContext,
    pub fees_carry_forward_setting_id: FeesCarryForwardSettingId,
    pub threshold_minor: i64,
    pub enabled: bool,
    pub description: Option<String>,
}

impl CreateFeesCarryForwardSettingCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub const fn required_capabilities() -> &'static [Capability] {
        // Fm-prefix RBAC variants don't exist yet; fall back to the
        // closest existing capability. See Wave 72 / Wave 75 lessons
        // for the Fm-prefix RBAC variant gap. To be revisited in a
        // future RBAC revision (v3 Part 6).
        &[Capability::FinanceFeesCarryForwardConfigure]
    }
}

/// Command: append a new free-form note to an [`FmFeesInvoice`] line
/// (per v3 Part 2 F30 + checklist § FmFeesInvoiceLineNote). Dispatched
/// by the `create_fm_fees_invoice_line_note` service function in
/// `services.rs`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFmFeesInvoiceLineNoteCommand {
    pub tenant: TenantContext,
    pub fm_fees_invoice_id: FmFeesInvoiceId,
    pub note: String,
}

impl CreateFmFeesInvoiceLineNoteCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesInvoiceConfigure]
    }
}

/// Command: append a new free-form note to an [`FmFeesTransaction`]
/// line (per v3 Part 2 F32 + checklist § FmFeesTransactionLineNote).
/// Dispatched by the `create_fm_fees_transaction_line_note` service
/// function in `services.rs`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFmFeesTransactionLineNoteCommand {
    pub tenant: TenantContext,
    pub fm_fees_transaction_id: FmFeesTransactionId,
    pub note: String,
}

impl CreateFmFeesTransactionLineNoteCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        // Falls back to FinanceFeesInvoiceConfigure since the RBAC
        // capability enum does not yet have a Fm-prefix variant
        // (per Wave 72 FFILN precedent). When Fm-prefix variants land
        // in a future RBAC revision, this will be updated to
        // FinanceFmFeesTransactionConfigure.
        vec![Capability::FinanceFeesInvoiceConfigure]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFeesCarryForwardLogCommand {
    pub tenant: TenantContext,
    pub fees_carry_forward_log_id: FeesCarryForwardLogId,
}


impl ReadFeesCarryForwardLogCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesCarryForwardRead]
    }
}
// -- DueFeesLoginPrevent (Unblock / Read) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnblockLoginForDueFeesCommand {
    pub tenant: TenantContext,
    pub due_fees_login_prevent_id: DueFeesLoginPreventId,
    pub reason: String,
}


impl UnblockLoginForDueFeesCommand {
    /// The command type discriminator for the dispatcher's audit
    /// log / idempotency table.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_DUE_FEES_UNBLOCK_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceDueFeesUnblock]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadDueFeesBlockCommand {
    pub tenant: TenantContext,
    pub due_fees_login_prevent_id: DueFeesLoginPreventId,
}


impl ReadDueFeesBlockCommand {
    /// The command type discriminator for the dispatcher's audit
    /// log / idempotency table.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_DUE_FEES_READ_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceDueFeesRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigureDueFeesBlockSettingCommand {
    pub tenant: TenantContext,
    pub days_overdue_threshold: i64,
    pub prevent_reason: PreventReason,
}


impl ConfigureDueFeesBlockSettingCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceDueFeesBlock]
    }
}
// -- Wave 108 -- AmountTransfer (inter-account cash movement) --

pub const FINANCE_AMOUNT_TRANSFER_CREATE_COMMAND_TYPE: &str = "finance.amount_transfer.create";
pub const FINANCE_AMOUNT_TRANSFER_READ_COMMAND_TYPE: &str = "finance.amount_transfer.read";
pub const FINANCE_AMOUNT_TRANSFER_RETIRE_COMMAND_TYPE: &str = "finance.amount_transfer.retire";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateAmountTransferCommand {
    pub tenant: TenantContext,
    pub amount_transfer_id: AmountTransferId,
    pub from_account_id: BankAccountId,
    pub to_account_id: BankAccountId,
    pub amount_minor: i64,
    pub currency: Currency,
    pub transfer_date: NaiveDate,
    pub note: Option<String>,
    /// AT I-3: optional idempotency reference. The dispatcher
    /// enforces uniqueness on the
    /// (from_account_id, to_account_id, reference) tuple.
    pub reference: Option<String>,
}


impl CreateAmountTransferCommand {
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceBankTransfer]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetireAmountTransferCommand {
    pub tenant: TenantContext,
    pub amount_transfer_id: AmountTransferId,
}

impl RetireAmountTransferCommand {
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceBankTransfer]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadAmountTransferCommand {
    pub tenant: TenantContext,
    pub amount_transfer_id: AmountTransferId,
}


impl ReadAmountTransferCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
// -- ChartOfAccount (read) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadChartOfAccountCommand {
    pub tenant: TenantContext,
    pub chart_of_account_id: ChartOfAccountId,
}


impl ReadChartOfAccountCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceChartOfAccountRead]
    }
}
// -- InvoiceSetting (the school's invoice-numbering config) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateInvoiceSettingCommand {
    pub tenant: TenantContext,
    pub prefix: String,
    pub start_form: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadInvoiceSettingCommand {
    pub tenant: TenantContext,
    pub invoice_setting_id: InvoiceSettingId,
}

// -- QuestionBankFee (the per-question fee amount; Wave 68) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateQuestionBankFeeCommand {
    pub tenant: TenantContext,
    pub name: String,
    pub amount_minor: i64,
    pub description: Option<String>,
}


impl ReadInvoiceSettingCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
// -- FeesPaymentSlip (per-payment printable slip) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFeesPaymentSlipCommand {
    pub tenant: TenantContext,
    pub fees_payment_slip_id: FeesPaymentSlipId,
}


impl ReadFeesPaymentSlipCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinancePaymentRead]
    }
}
// =============================================================================
// Reports — the 22 finance reports. Each is a read-only command with the
// tenant anchor, a date range, and an optional class scope.
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFeesCollectionReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub class_id: Option<crate::value_objects::ClassId>,
}


impl ReadFeesCollectionReportCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceReportRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadOutstandingFeesReportCommand {
    pub tenant: TenantContext,
    pub as_of: NaiveDate,
    pub class_id: Option<crate::value_objects::ClassId>,
}


impl ReadOutstandingFeesReportCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceReportRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadExpenseReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub expense_head_id: Option<ExpenseHeadId>,
}


impl ReadExpenseReportCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceExpenseRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadIncomeReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub income_head_id: Option<IncomeHeadId>,
}


impl ReadIncomeReportCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceIncomeRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadBankStatementReportCommand {
    pub tenant: TenantContext,
    pub bank_account_id: BankAccountId,
    pub from: NaiveDate,
    pub to: NaiveDate,
}


impl ReadBankStatementReportCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceReportRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadWalletBalanceReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
}


impl ReadWalletBalanceReportCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceWalletRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadPayrollReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub staff_id: Option<educore_hr::value_objects::StaffId>,
}


impl ReadPayrollReportCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceReportRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadPaymentMethodReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub payment_method_id: Option<PaymentMethodId>,
}


impl ReadPaymentMethodReportCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinancePaymentMethodRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFeesDiscountReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub fees_discount_id: Option<FeesDiscountId>,
}


impl ReadFeesDiscountReportCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesDiscountRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadDueFeesReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub class_id: Option<crate::value_objects::ClassId>,
}


impl ReadDueFeesReportCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceDueFeesRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadClassWiseCollectionReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub class_id: crate::value_objects::ClassId,
}


impl ReadClassWiseCollectionReportCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceReportRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadDailyCollectionReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
}


impl ReadDailyCollectionReportCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceReportRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadMonthlyCollectionReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
}


impl ReadMonthlyCollectionReportCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceReportRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadHeadWiseExpenseReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub expense_head_id: ExpenseHeadId,
}


impl ReadHeadWiseExpenseReportCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceReportRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadHeadWiseIncomeReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub income_head_id: IncomeHeadId,
}


impl ReadHeadWiseIncomeReportCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceReportRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadCashFlowReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
}


impl ReadCashFlowReportCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceReportRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadProfitLossReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
}


impl ReadProfitLossReportCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceReportRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadBalanceSheetReportCommand {
    pub tenant: TenantContext,
    pub as_of: NaiveDate,
}


impl ReadBalanceSheetReportCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceReportRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadTrialBalanceReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
}


impl ReadTrialBalanceReportCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceReportRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadLedgerReportCommand {
    pub tenant: TenantContext,
    pub chart_of_account_id: ChartOfAccountId,
    pub from: NaiveDate,
    pub to: NaiveDate,
}


impl ReadLedgerReportCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceReportRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadReceiptReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub fees_payment_id: Option<FeesPaymentId>,
}


impl ReadReceiptReportCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceReportRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadRefundReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
}


impl ReadRefundReportCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceReportRead]
    }
}
// =============================================================================
// Standalone command shapes (kept for backward compatibility with the
// pre-expansion callers; the equivalent Create/Open/Block/Execute shapes
// above are the canonical Phase 7 command types.)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigureFeesGroupCommand {
    pub tenant: TenantContext,
    pub name: String,
    pub description: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub due_date: NaiveDate,
}


impl ConfigureFeesGroupCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesGroupRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigureFeesTypeCommand {
    pub tenant: TenantContext,
    pub fees_group_id: crate::value_objects::FeesGroupId,
    pub name: String,
    pub description: Option<String>,
}


impl ConfigureFeesTypeCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesTypeRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateExpenseHeadCommand {
    pub tenant: TenantContext,
    pub expense_head_id: ExpenseHeadId,
    pub name: String, // EH I-1 pinned
    pub description: Option<String>,
}


impl CreateExpenseHeadCommand {
    /// The command type discriminator for the dispatcher's audit
    /// log / idempotency table.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_EXPENSE_HEAD_CREATE_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceExpenseHeadCreate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockLoginForDueFeesCommand {
    pub tenant: TenantContext,
    pub due_fees_login_prevent_id: DueFeesLoginPreventId,
    pub academic_year_id: educore_academic::AcademicYearId,
    pub user_id: UserId,
    pub user_type: crate::aggregate::DueFeesLoginPreventRole,
    pub outstanding_balance_minor: i64,
    pub reason: String,
}


impl BlockLoginForDueFeesCommand {
    /// The command type discriminator for the dispatcher's audit
    /// log / idempotency table.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_DUE_FEES_BLOCK_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceDueFeesBlock]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CarryForwardFeesBalanceCommand {
    pub tenant: TenantContext,
    pub student_id: educore_academic::StudentId,
    pub from: educore_academic::AcademicYearId,
    pub to: educore_academic::AcademicYearId,
}


impl CarryForwardFeesBalanceCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
// =============================================================================
// Minimal command stubs — new aggregates added in commit 429f74f
// (Cluster C: finance aggregate gap-fill). These stubs carry the
// school_id anchor (`tenant`) and the typed id only; real field
// shapes land in subsequent Phase 7 workstreams (B–L). The
// idempotency sub-port keys each command by `command_type`; the
// matching constants above are the canonical values.
// =============================================================================

// -- FeesAssignDiscount (Phase 7 Workstream F) --

/// Stable command type identifier for [`CreateFeesAssignDiscountCommand`].
pub const FINANCE_FEES_ASSIGN_DISCOUNT_CREATE_COMMAND_TYPE: &str = "finance.fees_assign_discount.create";
/// Stable command type identifier for [`RetireFeesAssignDiscountCommand`].
pub const FINANCE_FEES_ASSIGN_DISCOUNT_RETIRE_COMMAND_TYPE: &str = "finance.fees_assign_discount.retire";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFeesAssignDiscountCommand {
    pub tenant: TenantContext,
    pub fees_assign_discount_id: FeesAssignDiscountId,
    pub fees_assign_id: FeesAssignId,
    pub discount_id: FeesDiscountId,
    pub applied_amount_minor: i64, // FAD I-1
    pub unapplied_amount_minor: i64, // FAD I-1
    pub currency: Currency,
    pub note: Option<String>,
}


impl CreateFeesAssignDiscountCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesAssignCreate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFeesAssignDiscountCommand {
    pub tenant: TenantContext,
    pub fees_assign_discount_id: FeesAssignDiscountId,
}


impl ReadFeesAssignDiscountCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesAssignRead]
    }
}
// -- DirectFeesInstallmentChildPayment (Wave 96 — RealDirectFeesInstallmentChildPayment) --

/// COMMAND_TYPE discriminator for `CreateDirectFeesInstallmentChildPaymentCommand`.
pub const FINANCE_DIRECT_FEES_INSTALLMENT_CHILD_PAYMENT_CREATE_COMMAND_TYPE: &str =
    "finance.direct_fees_installment_child_payment.create";

/// COMMAND_TYPE discriminator for `ReadDirectFeesInstallmentChildPaymentCommand`.
pub const FINANCE_DIRECT_FEES_INSTALLMENT_CHILD_PAYMENT_READ_COMMAND_TYPE: &str =
    "finance.direct_fees_installment_child_payment.read";

/// COMMAND_TYPE discriminator for `RetireDirectFeesInstallmentChildPaymentCommand`.
pub const FINANCE_DIRECT_FEES_INSTALLMENT_CHILD_PAYMENT_RETIRE_COMMAND_TYPE: &str =
    "finance.direct_fees_installment_child_payment.retire";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateDirectFeesInstallmentChildPaymentCommand {
    pub tenant: TenantContext,
    pub direct_fees_installment_child_payment_id: DirectFeesInstallmentChildPaymentId,
    pub installment_id: DirectFeesInstallmentId,
    pub paid_amount_minor: i64, // FFIChild I-1
    pub note: Option<String>,
}

impl CreateDirectFeesInstallmentChildPaymentCommand {
    /// The command-type discriminator.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_DIRECT_FEES_INSTALLMENT_CHILD_PAYMENT_CREATE_COMMAND_TYPE;
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceDirectFeesInstallmentCreate]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadDirectFeesInstallmentChildPaymentCommand {
    pub tenant: TenantContext,
    pub direct_fees_installment_child_payment_id: DirectFeesInstallmentChildPaymentId,
}

impl ReadDirectFeesInstallmentChildPaymentCommand {
    /// The command-type discriminator.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_DIRECT_FEES_INSTALLMENT_CHILD_PAYMENT_READ_COMMAND_TYPE;
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceDirectFeesInstallmentPay]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetireDirectFeesInstallmentChildPaymentCommand {
    pub tenant: TenantContext,
    pub direct_fees_installment_child_payment_id: DirectFeesInstallmentChildPaymentId,
}

impl RetireDirectFeesInstallmentChildPaymentCommand {
    /// The command-type discriminator.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_DIRECT_FEES_INSTALLMENT_CHILD_PAYMENT_RETIRE_COMMAND_TYPE;
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceDirectFeesInstallmentDelete]
    }
}
// -- FmFeesGroup (Phase 7 Workstream G) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFmFeesGroupCommand {
    pub tenant: TenantContext,
    pub name: String,
    pub description: Option<String>,
}


impl CreateFmFeesGroupCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesGroupCreate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFmFeesGroupCommand {
    pub tenant: TenantContext,
    pub fm_fees_group_id: FmFeesGroupId,
}


impl ReadFmFeesGroupCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesGroupRead]
    }
}
// -- FmFeesType (Phase 7 Workstream G) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFmFeesTypeCommand {
    pub tenant: TenantContext,
    pub fm_fees_type_id: FmFeesTypeId,
}


impl CreateFmFeesTypeCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesTypeCreate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFmFeesTypeCommand {
    pub tenant: TenantContext,
    pub fm_fees_type_id: FmFeesTypeId,
}


impl ReadFmFeesTypeCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesTypeRead]
    }
}
// -- FmFeesInvoice (Wave 100 — RealFmFeesInvoice) --

/// COMMAND_TYPE discriminator for `CreateFmFeesInvoiceCommand`.
pub const FINANCE_FM_FEES_INVOICE_CREATE_COMMAND_TYPE: &str =
    "finance.fm_fees_invoice.create";

/// COMMAND_TYPE discriminator for `ReadFmFeesInvoiceCommand`.
pub const FINANCE_FM_FEES_INVOICE_READ_COMMAND_TYPE: &str =
    "finance.fm_fees_invoice.read";

/// COMMAND_TYPE discriminator for `RetireFmFeesInvoiceCommand`.
pub const FINANCE_FM_FEES_INVOICE_RETIRE_COMMAND_TYPE: &str =
    "finance.fm_fees_invoice.retire";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFmFeesInvoiceCommand {
    pub tenant: TenantContext,
    pub fm_fees_invoice_id: FmFeesInvoiceId,
    pub invoice_number: String,
    pub payer_reference: String,
    pub amount_minor: i64, // FFI I-1
    pub discount_minor: Option<i64>,
    pub note: Option<String>,
    pub invoice_date: NaiveDate,
    pub due_date: NaiveDate,
}

impl CreateFmFeesInvoiceCommand {
    /// The command-type discriminator.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_FM_FEES_INVOICE_CREATE_COMMAND_TYPE;
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesInvoiceGenerate]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFmFeesInvoiceCommand {
    pub tenant: TenantContext,
    pub fm_fees_invoice_id: FmFeesInvoiceId,
}

impl ReadFmFeesInvoiceCommand {
    /// The command-type discriminator.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_FM_FEES_INVOICE_READ_COMMAND_TYPE;
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesInvoiceRead]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetireFmFeesInvoiceCommand {
    pub tenant: TenantContext,
    pub fm_fees_invoice_id: FmFeesInvoiceId,
}

impl RetireFmFeesInvoiceCommand {
    /// The command-type discriminator.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_FM_FEES_INVOICE_RETIRE_COMMAND_TYPE;
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesInvoiceConfigure]
    }
}
// -- FmFeesInvoiceChild (Wave 101 — RealFmFeesInvoiceChild) --

/// COMMAND_TYPE discriminator for `CreateFmFeesInvoiceChildCommand`.
pub const FINANCE_FM_FEES_INVOICE_CHILD_CREATE_COMMAND_TYPE: &str =
    "finance.fm_fees_invoice_child.create";

/// COMMAND_TYPE discriminator for `ReadFmFeesInvoiceChildCommand`.
pub const FINANCE_FM_FEES_INVOICE_CHILD_READ_COMMAND_TYPE: &str =
    "finance.fm_fees_invoice_child.read";

/// COMMAND_TYPE discriminator for `RetireFmFeesInvoiceChildCommand`.
pub const FINANCE_FM_FEES_INVOICE_CHILD_RETIRE_COMMAND_TYPE: &str =
    "finance.fm_fees_invoice_child.retire";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFmFeesInvoiceChildCommand {
    pub tenant: TenantContext,
    pub fm_fees_invoice_child_id: FmFeesInvoiceChildId,
    pub invoice_id: FmFeesInvoiceId,
    pub description: String,
    pub amount_minor: i64, // FFIChild I-1
}

impl CreateFmFeesInvoiceChildCommand {
    /// The command-type discriminator.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_FM_FEES_INVOICE_CHILD_CREATE_COMMAND_TYPE;
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesInvoiceGenerate]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFmFeesInvoiceChildCommand {
    pub tenant: TenantContext,
    pub fm_fees_invoice_child_id: FmFeesInvoiceChildId,
}

impl ReadFmFeesInvoiceChildCommand {
    /// The command-type discriminator.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_FM_FEES_INVOICE_CHILD_READ_COMMAND_TYPE;
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesInvoiceRead]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetireFmFeesInvoiceChildCommand {
    pub tenant: TenantContext,
    pub fm_fees_invoice_child_id: FmFeesInvoiceChildId,
}

impl RetireFmFeesInvoiceChildCommand {
    /// The command-type discriminator.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_FM_FEES_INVOICE_CHILD_RETIRE_COMMAND_TYPE;
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesInvoiceConfigure]
    }
}
// -- FmFeesInvoiceSetting (Wave 94 — RealFmFeesInvoiceSetting) --

/// COMMAND_TYPE discriminator for `CreateFmFeesInvoiceSettingCommand`.
pub const FINANCE_FM_FEES_INVOICE_SETTING_CREATE_COMMAND_TYPE: &str =
    "finance.fm_fees_invoice_setting.create";

/// COMMAND_TYPE discriminator for `ReadFmFeesInvoiceSettingCommand`.
pub const FINANCE_FM_FEES_INVOICE_SETTING_READ_COMMAND_TYPE: &str =
    "finance.fm_fees_invoice_setting.read";

/// COMMAND_TYPE discriminator for `UpdateFmFeesInvoiceSettingCommand`.
pub const FINANCE_FM_FEES_INVOICE_SETTING_UPDATE_COMMAND_TYPE: &str =
    "finance.fm_fees_invoice_setting.update";

/// COMMAND_TYPE discriminator for `RetireFmFeesInvoiceSettingCommand`.
pub const FINANCE_FM_FEES_INVOICE_SETTING_RETIRE_COMMAND_TYPE: &str =
    "finance.fm_fees_invoice_setting.retire";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFmFeesInvoiceSettingCommand {
    pub tenant: TenantContext,
    pub fm_fees_invoice_setting_id: FmFeesInvoiceSettingId,
    pub prefix: String, // FFIS I-3
    pub per_th: i64, // FFIS I-1
    pub due_date: NaiveDate, // FFIS I-2
    pub due_date_offset_days: i64, // FFIS I-2
}

impl CreateFmFeesInvoiceSettingCommand {
    /// The command-type discriminator.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_FM_FEES_INVOICE_SETTING_CREATE_COMMAND_TYPE;
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesInvoiceGenerate]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFmFeesInvoiceSettingCommand {
    pub tenant: TenantContext,
    pub fm_fees_invoice_setting_id: FmFeesInvoiceSettingId,
}

impl ReadFmFeesInvoiceSettingCommand {
    /// The command-type discriminator.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_FM_FEES_INVOICE_SETTING_READ_COMMAND_TYPE;
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesInvoiceRead]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateFmFeesInvoiceSettingCommand {
    pub tenant: TenantContext,
    pub fm_fees_invoice_setting_id: FmFeesInvoiceSettingId,
    pub per_th: i64, // FFIS I-1 (mutable)
    pub due_date: NaiveDate, // FFIS I-2 (mutable)
    pub due_date_offset_days: i64, // FFIS I-2 (mutable)
}

impl UpdateFmFeesInvoiceSettingCommand {
    /// The command-type discriminator.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_FM_FEES_INVOICE_SETTING_UPDATE_COMMAND_TYPE;
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesInvoiceConfigure]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetireFmFeesInvoiceSettingCommand {
    pub tenant: TenantContext,
    pub fm_fees_invoice_setting_id: FmFeesInvoiceSettingId,
}

impl RetireFmFeesInvoiceSettingCommand {
    /// The command-type discriminator.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_FM_FEES_INVOICE_SETTING_RETIRE_COMMAND_TYPE;
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesInvoiceConfigure]
    }
}
// -- FmFeesTransaction (Phase 7 Workstream G) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFmFeesTransactionCommand {
    pub tenant: TenantContext,
    pub fm_fees_transaction_id: FmFeesTransactionId,
}


impl CreateFmFeesTransactionCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFmFeesTransactionCommand {
    pub tenant: TenantContext,
    pub fm_fees_transaction_id: FmFeesTransactionId,
}


impl ReadFmFeesTransactionCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
// -- FmFeesTransactionChild (Phase 7 Workstream G) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFmFeesTransactionChildCommand {
    pub tenant: TenantContext,
    pub fm_fees_transaction_child_id: FmFeesTransactionChildId,
    pub fm_fees_transaction_id: FmFeesTransactionId,
    pub amount_minor: i64,
    pub description: Option<String>,
}


impl CreateFmFeesTransactionChildCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFmFeesTransactionChildCommand {
    pub tenant: TenantContext,
    pub fm_fees_transaction_child_id: FmFeesTransactionChildId,
}


impl ReadFmFeesTransactionChildCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
// -- FmFeesWeaver (Wave 95 — RealFmFeesWeaver) --

/// COMMAND_TYPE discriminator for `CreateFmFeesWeaverCommand`.
pub const FINANCE_FM_FEES_WEAVER_CREATE_COMMAND_TYPE: &str =
    "finance.fm_fees_weaver.create";

/// COMMAND_TYPE discriminator for `ReadFmFeesWeaverCommand`.
pub const FINANCE_FM_FEES_WEAVER_READ_COMMAND_TYPE: &str =
    "finance.fm_fees_weaver.read";

/// COMMAND_TYPE discriminator for `RetireFmFeesWeaverCommand`.
pub const FINANCE_FM_FEES_WEAVER_RETIRE_COMMAND_TYPE: &str =
    "finance.fm_fees_weaver.retire";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFmFeesWeaverCommand {
    pub tenant: TenantContext,
    pub fm_fees_weaver_id: FmFeesWeaverId,
    pub name: String,
    pub percentage: i64, // FFW I-1: must be in [0, 100]
}

impl CreateFmFeesWeaverCommand {
    /// The command-type discriminator.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_FM_FEES_WEAVER_CREATE_COMMAND_TYPE;
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFmFeesWeaverCommand {
    pub tenant: TenantContext,
    pub fm_fees_weaver_id: FmFeesWeaverId,
}

impl ReadFmFeesWeaverCommand {
    /// The command-type discriminator.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_FM_FEES_WEAVER_READ_COMMAND_TYPE;
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetireFmFeesWeaverCommand {
    pub tenant: TenantContext,
    pub fm_fees_weaver_id: FmFeesWeaverId,
}

impl RetireFmFeesWeaverCommand {
    /// The command-type discriminator.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_FM_FEES_WEAVER_RETIRE_COMMAND_TYPE;
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesInvoiceConfigure]
    }
}
// -- FeesInvoiceSetting (Phase 7 Workstream B) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFeesInvoiceSettingCommand {
    pub tenant: TenantContext,
    pub fees_invoice_setting_id: FeesInvoiceSettingId,
    pub prefix: String, // FISv I-1 pinned
    pub per_th: i64, // FISv I-2
    pub description: Option<String>,
}


impl CreateFeesInvoiceSettingCommand {
    /// The command type discriminator for the dispatcher's audit
    /// log / idempotency table.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_FEES_INVOICE_CONFIGURE_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesInvoiceConfigure]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFeesInvoiceSettingCommand {
    pub tenant: TenantContext,
    pub fees_invoice_setting_id: FeesInvoiceSettingId,
}


impl ReadFeesInvoiceSettingCommand {
    /// The command type discriminator for the dispatcher's audit
    /// log / idempotency table.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_FEES_INVOICE_READ_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesInvoiceRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateFeesInvoiceSettingCommand {
    pub tenant: TenantContext,
    pub fees_invoice_setting_id: FeesInvoiceSettingId,
    /// Mutable per-thousand threshold (FISv I-2). NOT prefix
    /// (FISv I-1 says prefix is pinned — changing the invoice
    /// prefix after invoices have been issued would break the
    /// audit trail; retire + create-new required).
    pub per_th: i64,
    pub description: Option<String>,
}


impl UpdateFeesInvoiceSettingCommand {
    /// The command type discriminator for the dispatcher's audit
    /// log / idempotency table.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_FEES_INVOICE_UPDATE_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesInvoiceUpdate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteFeesInvoiceSettingCommand {
    pub tenant: TenantContext,
    pub fees_invoice_setting_id: FeesInvoiceSettingId,
}


impl DeleteFeesInvoiceSettingCommand {
    /// The command type discriminator for the dispatcher's audit
    /// log / idempotency table.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_FEES_INVOICE_CANCEL_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesInvoiceCancel]
    }
}
// -- FeesInstallmentCredit (Phase 7 Workstream F) --

pub const FINANCE_FEES_INSTALLMENT_CREDIT_CREATE_COMMAND_TYPE: &str =
    "finance.fees_installment_credit.create";
pub const FINANCE_FEES_INSTALLMENT_CREDIT_READ_COMMAND_TYPE: &str =
    "finance.fees_installment_credit.read";
pub const FINANCE_FEES_INSTALLMENT_CREDIT_RETIRE_COMMAND_TYPE: &str =
    "finance.fees_installment_credit.retire";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFeesInstallmentCreditCommand {
    pub tenant: TenantContext,
    pub fees_installment_credit_id: FeesInstallmentCreditId,
    pub amount_minor: i64, // FIC I-1 pinned
    pub credit_source: crate::aggregate::FeesInstallmentCreditSource, // FIC I-2 type-pinned
    pub source_installment_id: FeesInstallmentId,
    pub description: Option<String>,
}


impl CreateFeesInstallmentCreditCommand {
    /// The command type discriminator for the dispatcher's audit
    /// log / idempotency table.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_FEES_INSTALLMENT_CREDIT_CREATE_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesInstallmentCreate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFeesInstallmentCreditCommand {
    pub tenant: TenantContext,
    pub fees_installment_credit_id: FeesInstallmentCreditId,
}


impl ReadFeesInstallmentCreditCommand {
    /// The command type discriminator for the dispatcher's audit
    /// log / idempotency table.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_FEES_INSTALLMENT_CREDIT_READ_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesInstallmentRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetireFeesInstallmentCreditCommand {
    pub tenant: TenantContext,
    pub fees_installment_credit_id: FeesInstallmentCreditId,
}


impl RetireFeesInstallmentCreditCommand {
    /// The command type discriminator for the dispatcher's audit
    /// log / idempotency table.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_FEES_INSTALLMENT_CREDIT_RETIRE_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesInstallmentCreate]
    }
}
// -- Wave 104 — Transaction (Phase 7 Workstream C — double-entry journal line) --
//
// TR I-1: the sum of debit lines equals the sum of credit lines.
// The Create command carries `transaction_date` +
// `description` (TR I-1 companion: non-empty trimmed) +
// `reference` (optional) + `total_debits_minor` (TR I-1 guard 1:
// pinned at construction with `>= 0` guard) +
// `total_credits_minor` (TR I-1 guard 2: pinned at construction
// with `>= 0` guard; companion invariant `total_debits_minor ==
// total_credits_minor`) + `currency` (companion invariant).

/// Stable command type identifier for [`CreateTransactionCommand`].
pub const FINANCE_TRANSACTION_CREATE_COMMAND_TYPE: &str = "finance.transaction.create";
/// Stable command type identifier for [`ReadTransactionCommand`].
pub const FINANCE_TRANSACTION_READ_COMMAND_TYPE: &str = "finance.transaction.read";
/// Stable command type identifier for [`RetireTransactionCommand`].
pub const FINANCE_TRANSACTION_RETIRE_COMMAND_TYPE: &str = "finance.transaction.retire";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateTransactionCommand {
    pub tenant: TenantContext,
    pub transaction_id: TransactionId,
    pub transaction_date: chrono::NaiveDate,
    pub description: String,
    pub reference: Option<String>,
    pub total_debits_minor: i64,
    pub total_credits_minor: i64,
    pub currency: Currency,
}


impl CreateTransactionCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceCreate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadTransactionCommand {
    pub tenant: TenantContext,
    pub transaction_id: TransactionId,
}


impl ReadTransactionCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetireTransactionCommand {
    pub tenant: TenantContext,
    pub transaction_id: TransactionId,
}


impl RetireTransactionCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceCreate]
    }
}

// -- Wave 105 — FeesInstallmentAssignDiscount (child discount on an installment assign) --
//
// FIAD I-1: applied_amount >= 0. The Create command carries
// `applied_amount_minor` (FIAD I-1 guard: pinned at construction
// with `>= 0` guard) + `discount_id` + `fees_installment_assign_id`
// (companion invariants: FK references) + `currency` (companion
// invariant: required) + `note` (optional).

/// Stable command type identifier for [`CreateFeesInstallmentAssignDiscountCommand`].
pub const FINANCE_FEES_INSTALLMENT_ASSIGN_DISCOUNT_CREATE_COMMAND_TYPE: &str =
    "finance.fees_installment_assign_discount.create";
/// Stable command type identifier for [`ReadFeesInstallmentAssignDiscountCommand`].
pub const FINANCE_FEES_INSTALLMENT_ASSIGN_DISCOUNT_READ_COMMAND_TYPE: &str =
    "finance.fees_installment_assign_discount.read";
/// Stable command type identifier for [`RetireFeesInstallmentAssignDiscountCommand`].
pub const FINANCE_FEES_INSTALLMENT_ASSIGN_DISCOUNT_RETIRE_COMMAND_TYPE: &str =
    "finance.fees_installment_assign_discount.retire";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFeesInstallmentAssignDiscountCommand {
    pub tenant: TenantContext,
    pub fees_installment_assign_discount_id: FeesInstallmentAssignDiscountId,
    pub discount_id: FeesDiscountId,
    pub fees_installment_assign_id: FeesInstallmentAssignId,
    pub applied_amount_minor: i64,
    pub currency: Currency,
    pub note: Option<String>,
}


impl CreateFeesInstallmentAssignDiscountCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesDiscountCreate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFeesInstallmentAssignDiscountCommand {
    pub tenant: TenantContext,
    pub fees_installment_assign_discount_id: FeesInstallmentAssignDiscountId,
}


impl ReadFeesInstallmentAssignDiscountCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesDiscountRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetireFeesInstallmentAssignDiscountCommand {
    pub tenant: TenantContext,
    pub fees_installment_assign_discount_id: FeesInstallmentAssignDiscountId,
}


impl RetireFeesInstallmentAssignDiscountCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesDiscountCreate]
    }
}
// -- Donor (Phase 7 Workstream D) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateDonorCommand {
    pub tenant: TenantContext,
    pub name: String,
    pub email: String,
    pub show_public: bool,
    pub phone: Option<String>,
    pub description: Option<String>,
}


impl CreateDonorCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadDonorCommand {
    pub tenant: TenantContext,
    pub donor_id: DonorId,
}


impl ReadDonorCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
// -- ProductPurchase (Wave 99 — RealProductPurchase) --

/// COMMAND_TYPE discriminator for `CreateProductPurchaseCommand`.
pub const FINANCE_PRODUCT_PURCHASE_CREATE_COMMAND_TYPE: &str =
    "finance.product_purchase.create";

/// COMMAND_TYPE discriminator for `ReadProductPurchaseCommand`.
pub const FINANCE_PRODUCT_PURCHASE_READ_COMMAND_TYPE: &str =
    "finance.product_purchase.read";

/// COMMAND_TYPE discriminator for `RetireProductPurchaseCommand`.
pub const FINANCE_PRODUCT_PURCHASE_RETIRE_COMMAND_TYPE: &str =
    "finance.product_purchase.retire";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateProductPurchaseCommand {
    pub tenant: TenantContext,
    pub product_purchase_id: ProductPurchaseId,
    pub product_name: String,
    pub quantity: i64,
    pub amount_minor: i64, // PPr I-1
    pub supplier_reference: Option<String>,
}

impl CreateProductPurchaseCommand {
    /// The command-type discriminator.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_PRODUCT_PURCHASE_CREATE_COMMAND_TYPE;
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadProductPurchaseCommand {
    pub tenant: TenantContext,
    pub product_purchase_id: ProductPurchaseId,
}

impl ReadProductPurchaseCommand {
    /// The command-type discriminator.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_PRODUCT_PURCHASE_READ_COMMAND_TYPE;
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetireProductPurchaseCommand {
    pub tenant: TenantContext,
    pub product_purchase_id: ProductPurchaseId,
}

impl RetireProductPurchaseCommand {
    /// The command-type discriminator.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_PRODUCT_PURCHASE_RETIRE_COMMAND_TYPE;
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
// -- InventoryPayment (Wave 98 — RealInventoryPayment) --

/// COMMAND_TYPE discriminator for `CreateInventoryPaymentCommand`.
pub const FINANCE_INVENTORY_PAYMENT_CREATE_COMMAND_TYPE: &str =
    "finance.inventory_payment.create";

/// COMMAND_TYPE discriminator for `ReadInventoryPaymentCommand`.
pub const FINANCE_INVENTORY_PAYMENT_READ_COMMAND_TYPE: &str =
    "finance.inventory_payment.read";

/// COMMAND_TYPE discriminator for `RetireInventoryPaymentCommand`.
pub const FINANCE_INVENTORY_PAYMENT_RETIRE_COMMAND_TYPE: &str =
    "finance.inventory_payment.retire";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateInventoryPaymentCommand {
    pub tenant: TenantContext,
    pub inventory_payment_id: InventoryPaymentId,
    pub supplier_name: String,
    pub amount_minor: i64, // IP I-1
    pub currency: Currency,
    pub note: Option<String>,
}

impl CreateInventoryPaymentCommand {
    /// The command-type discriminator.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_INVENTORY_PAYMENT_CREATE_COMMAND_TYPE;
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinancePaymentRead]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadInventoryPaymentCommand {
    pub tenant: TenantContext,
    pub inventory_payment_id: InventoryPaymentId,
}

impl ReadInventoryPaymentCommand {
    /// The command-type discriminator.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_INVENTORY_PAYMENT_READ_COMMAND_TYPE;
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinancePaymentRead]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetireInventoryPaymentCommand {
    pub tenant: TenantContext,
    pub inventory_payment_id: InventoryPaymentId,
}

impl RetireInventoryPaymentCommand {
    /// The command-type discriminator.
    pub const COMMAND_TYPE: &'static str =
        FINANCE_INVENTORY_PAYMENT_RETIRE_COMMAND_TYPE;
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinancePaymentRead]
    }
}
// =============================================================================
// Cluster D — 35 missing finance commands (minimal typed shapes).
//
// These commands were declared in `docs/specs/finance/commands.md` but had
// no matching `*Command` struct. Each struct carries `tenant: TenantContext`
// plus the aggregate identifier the command operates on. Full field shapes
// (amounts, dates, method ids, etc.) are filled in by the per-aggregate
// workstream that owns the action.
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateFeesMasterAmountCommand {
    pub tenant: TenantContext,
    pub fees_master_id: FeesMasterId,
}


impl UpdateFeesMasterAmountCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesMasterUpdate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssignFeesToClassCommand {
    pub tenant: TenantContext,
    pub fees_assign_id: FeesAssignId,
}


impl AssignFeesToClassCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssignFeesToStudentCommand {
    pub tenant: TenantContext,
    pub fees_assign_id: FeesAssignId,
}


impl AssignFeesToStudentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateFeesAssignDiscountCommand {
    pub tenant: TenantContext,
    pub fees_assign_discount_id: FeesAssignDiscountId,
}


impl UpdateFeesAssignDiscountCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesAssignUpdate]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetireFeesAssignDiscountCommand {
    pub tenant: TenantContext,
    pub fees_assign_discount_id: FeesAssignDiscountId,
}

impl RetireFeesAssignDiscountCommand {
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesAssignCreate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CloseFeesAssignCommand {
    pub tenant: TenantContext,
    pub fees_assign_id: FeesAssignId,
}


impl CloseFeesAssignCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesAssignClose]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssignInstallmentToStudentCommand {
    pub tenant: TenantContext,
    pub fees_installment_assign_id: FeesInstallmentAssignId,
}


impl AssignInstallmentToStudentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigureInvoiceNumberingCommand {
    pub tenant: TenantContext,
    pub fees_invoice_setting_id: FeesInvoiceSettingId,
}


impl ConfigureInvoiceNumberingCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayInvoiceCommand {
    pub tenant: TenantContext,
    pub fees_assign_id: FeesAssignId,
}


impl PayInvoiceCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayInstallmentCommand {
    pub tenant: TenantContext,
    pub fees_installment_assign_id: FeesInstallmentAssignId,
}


impl PayInstallmentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigureDirectFeesInstallmentCommand {
    pub tenant: TenantContext,
    pub direct_fees_installment_id: DirectFeesInstallmentId,
}


impl ConfigureDirectFeesInstallmentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceDirectFeesInstallmentRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssignDirectInstallmentCommand {
    pub tenant: TenantContext,
    pub direct_fees_installment_assign_id: DirectFeesInstallmentAssignId,
}


impl AssignDirectInstallmentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayDirectInstallmentCommand {
    pub tenant: TenantContext,
    pub direct_fees_installment_child_payment_id: DirectFeesInstallmentChildPaymentId,
}


impl PayDirectInstallmentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigureDirectFeesCommand {
    pub tenant: TenantContext,
    pub direct_fees_setting_id: DirectFeesSettingId,
}


impl ConfigureDirectFeesCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigureFeesReminderCommand {
    pub tenant: TenantContext,
    pub direct_fees_reminder_id: DirectFeesReminderId,
}


impl ConfigureFeesReminderCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesReminderConfigure]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordBankStatementCommand {
    pub tenant: TenantContext,
    pub bank_statement_id: BankStatementId,
}


impl RecordBankStatementCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceBankStatementRecord]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerateBankPaymentSlipCommand {
    pub tenant: TenantContext,
    pub bank_payment_slip_id: BankPaymentSlipId,
}


impl GenerateBankPaymentSlipCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinancePaymentRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApproveBankPaymentCommand {
    pub tenant: TenantContext,
    pub bank_payment_slip_id: BankPaymentSlipId,
}


impl ApproveBankPaymentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinancePaymentRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RejectBankPaymentCommand {
    pub tenant: TenantContext,
    pub bank_payment_slip_id: BankPaymentSlipId,
}


impl RejectBankPaymentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinancePaymentRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferFundsCommand {
    pub tenant: TenantContext,
    pub amount_transfer_id: AmountTransferId,
}


impl TransferFundsCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordExpenseCommand {
    pub tenant: TenantContext,
    pub expense_id: ExpenseId,
}


impl RecordExpenseCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceExpenseRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordIncomeCommand {
    pub tenant: TenantContext,
    pub income_id: IncomeId,
}


impl RecordIncomeCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceIncomeRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AddWalletCreditCommand {
    pub tenant: TenantContext,
    pub wallet_transaction_id: WalletTransactionId,
}


impl AddWalletCreditCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceWalletCredit]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeductWalletCreditCommand {
    pub tenant: TenantContext,
    pub wallet_transaction_id: WalletTransactionId,
}


impl DeductWalletCreditCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceWalletCredit]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordPayrollPaymentCommand {
    pub tenant: TenantContext,
    pub payroll_payment_id: PayrollPaymentId,
}


impl RecordPayrollPaymentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinancePaymentRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordInventoryPaymentCommand {
    pub tenant: TenantContext,
    pub inventory_payment_id: InventoryPaymentId,
}


impl RecordInventoryPaymentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinancePaymentRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordProductPurchaseCommand {
    pub tenant: TenantContext,
    pub product_purchase_id: ProductPurchaseId,
}


impl RecordProductPurchaseCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordProductPaymentCommand {
    pub tenant: TenantContext,
    pub product_purchase_id: ProductPurchaseId,
}


impl RecordProductPaymentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinancePaymentRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigureInvoiceSettingsCommand {
    pub tenant: TenantContext,
    pub invoice_setting_id: InvoiceSettingId,
}


impl ConfigureInvoiceSettingsCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigurePaymentGatewayCommand {
    pub tenant: TenantContext,
    pub payment_gateway_setting_id: PaymentGatewaySettingId,
}


impl ConfigurePaymentGatewayCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinancePaymentGatewayConfigure]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttachFeesToQuestionBankCommand {
    pub tenant: TenantContext,
    pub fm_fees_weaver_id: FmFeesWeaverId,
}


impl AttachFeesToQuestionBankCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceBankRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateChartOfAccountCommand {
    pub tenant: TenantContext,
    pub chart_of_account_id: ChartOfAccountId,
    pub code: String,
    pub name: String,
    pub account_type: crate::value_objects::AccountType,
    pub description: Option<String>,
}


impl CreateChartOfAccountCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceChartOfAccountCreate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateSalaryTemplateCommand {
    pub tenant: TenantContext,
    pub salary_template_id: SalaryTemplateId,
    pub name: String,
    pub currency: Currency,
    pub gross_salary_minor: i64,
    pub net_salary_minor: i64,
    pub description: Option<String>,
}


impl CreateSalaryTemplateCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        // Wave 82 RBAC: FinancePayrollPaymentRecord (closest existing
        // variant — FinancePayrollPaymentApprove does not exist;
        // same fallback used in Wave 81 PayrollPaymentApproval
        // commands).
        vec![Capability::FinancePayrollPaymentRecord]
    }
}

// -----------------------------------------------------------------------------
// BankPaymentSlipAudit commands (Wave 83 — per-aggregate wave pattern from
// Waves 65–82)
// -----------------------------------------------------------------------------
//
// Per v3 Part 2 F37 + checklist § BankPaymentSlipAudit: 2 invariants:
//   - BPA I-1: append-only log (enforced at the API surface by
//             intentionally exposing no `update_*` mutator on the
//             aggregate).
//   - BPA I-2: timestamps recorded (audit footer stamps; recorded_at
//             payload field carries the slip-recording semantic
//             timestamp).
//
// Append-only event family — parallel to Wave 70
// FeesCarryForwardLog. GREENFIELD command (no skeleton existed per
// Wave 83 recon). RBAC fallback: Capability::FinanceBankSlipAudit
// does not exist; use the closest existing variant
// (FinanceBankSlipGenerate, parallel to Wave 72/75/77/78/80/81/82
// fallback chain).

/// Command: append a new `BankPaymentSlipAudit` row to the
/// append-only log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateBankPaymentSlipAuditCommand {
    pub tenant: TenantContext,
    pub bank_payment_slip_audit_id: BankPaymentSlipAuditId,
    pub bank_payment_slip_id: BankPaymentSlipId,
    pub bank_account_id: BankAccountId,
    pub amount_minor: i64,
    pub currency: Currency,
    pub recorded_at: Timestamp,
    pub description: Option<String>,
}

impl CreateBankPaymentSlipAuditCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub const fn required_capabilities() -> &'static [Capability] {
        // Wave 83 RBAC: FinanceBankSlipAudit does not exist; use
        // FinanceBankSlipGenerate (closest existing variant for the
        // bank-slip generation flow). Parallel to Wave 72/75/77/78/
        // 80/81/82 fallback chain.
        &[Capability::FinanceBankSlipGenerate]
    }
}

// -----------------------------------------------------------------------------
// BankStatementAttachment commands (Wave 84 — per-aggregate wave pattern from
// Waves 65–83)
// -----------------------------------------------------------------------------
//
// Per v3 Part 2 F47 + checklist § BankStatementAttachment: 2 invariants:
//   - BSA I-1: attachment ref valid (file_reference must point to
//             an existing file in the file storage port; dispatcher
//             responsibility, not aggregate).
//   - BSA I-2: orphan after BankStatement delete (bank_statement_id
//             reference is preserved in the audit footer even after
//             retire; cascade-delete handled by dispatcher).
//
// Append-only event family — parallel to Wave 81 PayrollPaymentApproval
// commands + Wave 83 BankPaymentSlipAudit commands. GREENFIELD command
// (no skeleton existed per Wave 84 recon). The BankStatementAttachment
// struct (entities.rs) does NOT have its own id field (parent
// bank_statement_id is de-facto identity + file_reference Uuid serves
// as a secondary identifier), so the command references
// bank_statement_id directly. RBAC fallback:
// Capability::FinanceBankStatementAttachment does not exist; use the
// closest existing variant (FinanceBankStatementRecord, parallel to
// Wave 72/75/77/78/80/81/82/83 fallback chain).

/// Command: create a new `BankStatementAttachment` row. The
/// dispatcher validates BSA I-1 (file_reference exists at the file
/// storage port) before calling the service function.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateBankStatementAttachmentCommand {
    pub tenant: TenantContext,
    pub bank_statement_id: BankStatementId,
    pub file_reference: Uuid,
    pub uploaded_at: Timestamp,
    pub uploaded_by: UserId,
    pub description: Option<String>,
}

impl CreateBankStatementAttachmentCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub const fn required_capabilities() -> &'static [Capability] {
        // Wave 84 RBAC: FinanceBankStatementAttachment does not exist;
        // use FinanceBankStatementRecord (closest existing variant for
        // the bank-statement record flow). Parallel to Wave 72/75/77/78/
        // 80/81/82/83 fallback chain.
        &[Capability::FinanceBankStatementRecord]
    }
}

// -----------------------------------------------------------------------------
// BankStatement commands (Wave 85 — per-aggregate wave pattern from
// Waves 65–84)
// -----------------------------------------------------------------------------
//
// Per v3 Part 2 F48 + checklist § BankStatement: 4 invariants:
//   - BS I-1: amount >= 0 (validated at construction + on update).
//   - BS I-2: type ∈ {income, expense} (enforced at type-system
//             level via the StatementType enum).
//   - BS I-3: after_balance matches running balance (the aggregate
//             pins balance_after_minor at construction + on update;
//             cross-statement consistency is the dispatcher's
//             responsibility).
//   - BS I-4: append-only; corrections via reverse. The Update
//             command only allows metadata changes (description);
//             amount/balance corrections happen via the Reverse
//             command (which creates a new opposite-direction row).
//
// Full lifecycle event family — 4 commands: Create (enter the log),
// Update (description only — BS I-4 immutable amount/balance),
// Reverse (BS I-4: marks the original as corrected by a new
// opposite-direction row), Retire (tombstone).
//
// GREENFIELD drop (no skeleton existed per Wave 85 recon). RBAC:
// FinanceBankStatementReverse exists at rbac/value_objects.rs:366
// (BS I-4 explicit capability — use for the Reverse command);
// FinanceBankStatementRecord is the closest existing variant for
// Create/Update (bank-statement record flow); FinanceBankStatementRecord
// also covers Retire (tombstone).

/// Command: create a new `BankStatement` row in the per-account log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateBankStatementCommand {
    pub tenant: TenantContext,
    pub bank_statement_id: BankStatementId,
    pub bank_account_id: BankAccountId,
    pub statement_type: StatementType,
    pub amount_minor: i64,
    pub balance_after_minor: i64,
    pub currency: Currency,
    pub occurred_at: Timestamp,
    pub description: Option<String>,
}

impl CreateBankStatementCommand {
    #[must_use]
    pub const fn required_capabilities() -> &'static [Capability] {
        // Wave 85 RBAC: FinanceBankStatementCreate does not exist;
        // use FinanceBankStatementRecord (closest existing variant
        // for the bank-statement record flow).
        &[Capability::FinanceBankStatementRecord]
    }
}

/// Command: update the metadata of an existing `BankStatement` row.
/// Only the `description` field is mutable here; amount_minor +
/// balance_after_minor + statement_type are immutable (BS I-4
/// append-only enforcement). Corrections to amount/balance happen
/// via the `ReverseBankStatementCommand`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateBankStatementCommand {
    pub tenant: TenantContext,
    pub bank_statement_id: BankStatementId,
    pub description: Option<String>,
}

impl UpdateBankStatementCommand {
    #[must_use]
    pub const fn required_capabilities() -> &'static [Capability] {
        &[Capability::FinanceBankStatementRecord]
    }
}

/// Command: mark a `BankStatement` as corrected via a new
/// opposite-direction row (BS I-4 append-only enforcement). The
/// dispatcher is responsible for creating the new reverse row
/// (which carries the inverse amount + type). This command only
/// emits the `BankStatementReversed` event marking the original
/// statement as corrected.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReverseBankStatementCommand {
    pub tenant: TenantContext,
    pub bank_statement_id: BankStatementId,
    pub reverse_row_id: BankStatementId,
}

impl ReverseBankStatementCommand {
    #[must_use]
    pub const fn required_capabilities() -> &'static [Capability] {
        // FinanceBankStatementReverse EXISTS at
        // rbac/value_objects.rs:366 — use it directly (BS I-4 explicit
        // capability, no fallback needed).
        &[Capability::FinanceBankStatementReverse]
    }
}

/// Command: soft-delete a `BankStatement` row by flipping
/// `active_status` to `Retired`. This is a tombstone, NOT a content
/// edit — the original amount + balance + statement_type are
/// preserved in the audit footer for legal-record retention.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetireBankStatementCommand {
    pub tenant: TenantContext,
    pub bank_statement_id: BankStatementId,
}

impl RetireBankStatementCommand {
    #[must_use]
    pub const fn required_capabilities() -> &'static [Capability] {
        &[Capability::FinanceBankStatementRecord]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetHourlyRateCommand {
    pub tenant: TenantContext,
}


impl SetHourlyRateCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AddFeesInstallmentCreditCommand {
    pub tenant: TenantContext,
    pub fees_installment_credit_id: FeesInstallmentCreditId,
}


impl AddFeesInstallmentCreditCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesInstallmentRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsumeFeesInstallmentCreditCommand {
    pub tenant: TenantContext,
    pub fees_installment_credit_id: FeesInstallmentCreditId,
}
