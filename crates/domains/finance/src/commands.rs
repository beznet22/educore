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
use educore_rbac::value_objects::Capability;
use educore_core::tenant::TenantContext;

use crate::value_objects::{
    AccountType, AmountTransferId, BankAccountId, BankMode, BankPaymentSlipId, BankStatementId,
    ChartOfAccountId, Currency, DirectFeesInstallmentAssignId, DirectFeesInstallmentChildPaymentId,
    DirectFeesInstallmentId, DirectFeesReminderId, DirectFeesSettingId, DonorId,
    DueFeesLoginPreventId, ExpenseHeadId, ExpenseId, FeesAssignDiscountId, FeesAssignId,
    FeesCarryForwardId, FeesCarryForwardLogId, FeesCarryForwardSettingId, FeesDiscountId,
    FeesGroupId, FeesInstallmentAssignId, FeesInstallmentCreditId, FeesInstallmentId,
    FeesInvoiceId, FeesInvoiceSettingId, FeesMasterId, FeesPaymentId, FeesPaymentSlipId,
    FeesTypeId, FmFeesGroupId, FmFeesInvoiceChildId, FmFeesInvoiceId, FmFeesInvoiceSettingId,
    FmFeesTransactionChildId, FmFeesTransactionId, FmFeesTypeId, FmFeesWeaverId, GatewayMode,
    IncomeHeadId, IncomeId, InventoryPaymentId, InvoiceSettingId, PaymentGatewaySettingId,
    PaymentMethodId, PaymentMethodKind, PayrollPaymentId, PreventReason, ProductPurchaseId,
    SalaryTemplateId, TransactionId, WalletId, WalletTransactionId, WalletTxType,
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
    pub name: String,
    pub description: Option<String>,
}


impl CreateFeesGroupCommand {
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
    pub name: Option<String>,
    pub description: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,
}


impl UpdateFeesGroupCommand {
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
    pub fees_group_id: FeesGroupId,
    pub class_id: crate::value_objects::ClassId,
    pub amount_minor: i64,
    pub currency: Currency,
    pub due_date: NaiveDate,
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
    pub name: String,
    pub discount_code: String,
    pub amount_minor: i64,
    pub currency: Currency,
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
    pub amount_minor: Option<i64>,
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
    pub student_id: educore_academic::StudentId,
    pub name: String,
    pub amount_minor: i64,
    pub currency: Currency,
    pub due_date: NaiveDate,
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
// -- DirectFeesInstallmentAssign (per-student linkage) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateDirectFeesInstallmentAssignCommand {
    pub tenant: TenantContext,
    pub direct_fees_installment_id: DirectFeesInstallmentId,
    pub student_id: educore_academic::StudentId,
    pub amount_minor: i64,
    pub currency: Currency,
}


impl CreateDirectFeesInstallmentAssignCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceDirectFeesInstallmentCreate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteDirectFeesInstallmentAssignCommand {
    pub tenant: TenantContext,
    pub direct_fees_installment_assign_id: DirectFeesInstallmentAssignId,
}


impl DeleteDirectFeesInstallmentAssignCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceDirectFeesInstallmentDelete]
    }
}
// -- DirectFeesSetting (per-school direct-fees configuration) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateDirectFeesSettingCommand {
    pub tenant: TenantContext,
    pub enabled: bool,
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
    pub direct_fees_installment_id: DirectFeesInstallmentId,
    pub student_id: educore_academic::StudentId,
    pub remind_at: NaiveDate,
    pub note: Option<String>,
}


impl CreateDirectFeesReminderCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesReminderRead]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateDirectFeesReminderCommand {
    pub tenant: TenantContext,
    pub direct_fees_reminder_id: DirectFeesReminderId,
    pub remind_at: Option<NaiveDate>,
    pub note: Option<String>,
}


impl UpdateDirectFeesReminderCommand {
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
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesReminderDelete]
    }
}
// -- PaymentMethod --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreatePaymentMethodCommand {
    pub tenant: TenantContext,
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


impl CreateIncomeCommand {
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
    pub name: Option<String>,
    pub description: Option<String>,
}


impl UpdateExpenseHeadCommand {
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
// -- BankAccount (Update / Delete / Read) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateBankAccountCommand {
    pub tenant: TenantContext,
    pub bank_account_id: BankAccountId,
    pub bank_name: Option<String>,
    pub account_number: Option<String>,
    pub account_type: Option<AccountType>,
    pub currency: Option<Currency>,
    pub opening_balance_minor: Option<i64>,
}


impl UpdateBankAccountCommand {
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
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceDueFeesBlock]
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
// -- AmountTransfer (inter-account cash movement) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateAmountTransferCommand {
    pub tenant: TenantContext,
    pub from_account_id: BankAccountId,
    pub to_account_id: BankAccountId,
    pub amount_minor: i64,
    pub currency: Currency,
    pub transfer_date: NaiveDate,
    pub note: Option<String>,
}


impl CreateAmountTransferCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
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
// -- InvoiceSetting (the school's invoice-numbering config; read) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadInvoiceSettingCommand {
    pub tenant: TenantContext,
    pub invoice_setting_id: InvoiceSettingId,
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
    pub name: String,
}


impl CreateExpenseHeadCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceExpenseHeadCreate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenBankAccountCommand {
    pub tenant: TenantContext,
    pub bank_name: String,
    pub account_number: String,
    pub account_type: crate::value_objects::AccountType,
    pub opening_balance_minor: i64,
    pub currency: Currency,
}


impl OpenBankAccountCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceBankOpen]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockLoginForDueFeesCommand {
    pub tenant: TenantContext,
    pub user_id: UserId,
    pub reason: crate::value_objects::PreventReason,
}


impl BlockLoginForDueFeesCommand {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFeesAssignDiscountCommand {
    pub tenant: TenantContext,
    pub fees_assign_discount_id: FeesAssignDiscountId,
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
// -- DirectFeesInstallmentChildPayment (Phase 7 Workstream F) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateDirectFeesInstallmentChildPaymentCommand {
    pub tenant: TenantContext,
    pub direct_fees_installment_child_payment_id: DirectFeesInstallmentChildPaymentId,
}


impl CreateDirectFeesInstallmentChildPaymentCommand {
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
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceDirectFeesInstallmentPay]
    }
}
// -- FmFeesGroup (Phase 7 Workstream G) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFmFeesGroupCommand {
    pub tenant: TenantContext,
    pub fm_fees_group_id: FmFeesGroupId,
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
// -- FmFeesInvoice (Phase 7 Workstream G) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFmFeesInvoiceCommand {
    pub tenant: TenantContext,
    pub fm_fees_invoice_id: FmFeesInvoiceId,
}


impl CreateFmFeesInvoiceCommand {
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
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesInvoiceRead]
    }
}
// -- FmFeesInvoiceChild (Phase 7 Workstream G) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFmFeesInvoiceChildCommand {
    pub tenant: TenantContext,
    pub fm_fees_invoice_child_id: FmFeesInvoiceChildId,
}


impl CreateFmFeesInvoiceChildCommand {
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
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesInvoiceRead]
    }
}
// -- FmFeesInvoiceSetting (Phase 7 Workstream G) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFmFeesInvoiceSettingCommand {
    pub tenant: TenantContext,
    pub fm_fees_invoice_setting_id: FmFeesInvoiceSettingId,
}


impl CreateFmFeesInvoiceSettingCommand {
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
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesInvoiceRead]
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
// -- FmFeesWeaver (Phase 7 Workstream G) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFmFeesWeaverCommand {
    pub tenant: TenantContext,
    pub fm_fees_weaver_id: FmFeesWeaverId,
}


impl CreateFmFeesWeaverCommand {
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
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
// -- FeesInvoiceSetting (Phase 7 Workstream B) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFeesInvoiceSettingCommand {
    pub tenant: TenantContext,
    pub fees_invoice_setting_id: FeesInvoiceSettingId,
}


impl CreateFeesInvoiceSettingCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesInvoiceGenerate]
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFeesInvoiceSettingCommand {
    pub tenant: TenantContext,
    pub fees_invoice_setting_id: FeesInvoiceSettingId,
}


impl ReadFeesInvoiceSettingCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesInvoiceRead]
    }
}
// -- FeesInstallmentCredit (Phase 7 Workstream F) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFeesInstallmentCreditCommand {
    pub tenant: TenantContext,
    pub fees_installment_credit_id: FeesInstallmentCreditId,
}


impl CreateFeesInstallmentCreditCommand {
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
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceFeesInstallmentRead]
    }
}
// -- Transaction (Phase 7 Workstream C — double-entry journal line) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateTransactionCommand {
    pub tenant: TenantContext,
    pub transaction_id: TransactionId,
}


impl CreateTransactionCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
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
// -- Donor (Phase 7 Workstream D) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateDonorCommand {
    pub tenant: TenantContext,
    pub donor_id: DonorId,
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
// -- ProductPurchase (Phase 7 Workstream L) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateProductPurchaseCommand {
    pub tenant: TenantContext,
    pub product_purchase_id: ProductPurchaseId,
}


impl CreateProductPurchaseCommand {
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
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
    }
}
// -- InventoryPayment (Phase 7 Workstream L) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateInventoryPaymentCommand {
    pub tenant: TenantContext,
    pub inventory_payment_id: InventoryPaymentId,
}


impl CreateInventoryPaymentCommand {
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
}


impl CreateSalaryTemplateCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::FinanceInvoiceRead]
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
