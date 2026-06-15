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
use educore_core::tenant::TenantContext;

use crate::value_objects::{
    AccountType, AmountTransferId, BankAccountId, BankMode, BankPaymentSlipId, BankStatementId,
    ChartOfAccountId, Currency, DirectFeesInstallmentAssignId, DirectFeesInstallmentId,
    DirectFeesReminderId, DirectFeesSettingId, DonorId, DueFeesLoginPreventId, ExpenseHeadId,
    ExpenseId, FeesAssignId, FeesCarryForwardId, FeesCarryForwardLogId, FeesCarryForwardSettingId,
    FeesDiscountId, FeesGroupId, FeesInstallmentId, FeesInvoiceId, FeesMasterId, FeesPaymentId,
    FeesPaymentSlipId, FeesTypeId, GatewayMode, IncomeHeadId, IncomeId, InvoiceSettingId,
    PaymentGatewaySettingId, PaymentMethodId, PaymentMethodKind, PayrollPaymentId, PreventReason,
    WalletId, WalletTransactionId, WalletTxType,
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

pub use crate::services::{
    ConfigureInvoiceNumberingCommand, CreateWalletCommand, CreditWalletCommand,
    DeductWalletCreditCommand, RecordExpenseCommand, RecordPaymentCommand,
    RequestWalletRefundCommand,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteFeesGroupCommand {
    pub tenant: TenantContext,
    pub fees_group_id: FeesGroupId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFeesGroupCommand {
    pub tenant: TenantContext,
    pub fees_group_id: FeesGroupId,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateFeesTypeCommand {
    pub tenant: TenantContext,
    pub fees_type_id: FeesTypeId,
    pub name: Option<String>,
    pub description: Option<String>,
    pub amount_minor: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteFeesTypeCommand {
    pub tenant: TenantContext,
    pub fees_type_id: FeesTypeId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFeesTypeCommand {
    pub tenant: TenantContext,
    pub fees_type_id: FeesTypeId,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateFeesMasterCommand {
    pub tenant: TenantContext,
    pub fees_master_id: FeesMasterId,
    pub amount_minor: Option<i64>,
    pub due_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteFeesMasterCommand {
    pub tenant: TenantContext,
    pub fees_master_id: FeesMasterId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFeesMasterCommand {
    pub tenant: TenantContext,
    pub fees_master_id: FeesMasterId,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateFeesDiscountCommand {
    pub tenant: TenantContext,
    pub fees_discount_id: FeesDiscountId,
    pub name: Option<String>,
    pub amount_minor: Option<i64>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteFeesDiscountCommand {
    pub tenant: TenantContext,
    pub fees_discount_id: FeesDiscountId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFeesDiscountCommand {
    pub tenant: TenantContext,
    pub fees_discount_id: FeesDiscountId,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateFeesAssignCommand {
    pub tenant: TenantContext,
    pub fees_assign_id: FeesAssignId,
    pub amount_minor: Option<i64>,
    pub due_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteFeesAssignCommand {
    pub tenant: TenantContext,
    pub fees_assign_id: FeesAssignId,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateFeesInstallmentCommand {
    pub tenant: TenantContext,
    pub fees_installment_id: FeesInstallmentId,
    pub name: Option<String>,
    pub due_date: Option<NaiveDate>,
    pub amount_minor: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteFeesInstallmentCommand {
    pub tenant: TenantContext,
    pub fees_installment_id: FeesInstallmentId,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateDirectFeesInstallmentCommand {
    pub tenant: TenantContext,
    pub direct_fees_installment_id: DirectFeesInstallmentId,
    pub name: Option<String>,
    pub amount_minor: Option<i64>,
    pub due_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteDirectFeesInstallmentCommand {
    pub tenant: TenantContext,
    pub direct_fees_installment_id: DirectFeesInstallmentId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadDirectFeesInstallmentCommand {
    pub tenant: TenantContext,
    pub direct_fees_installment_id: DirectFeesInstallmentId,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteDirectFeesInstallmentAssignCommand {
    pub tenant: TenantContext,
    pub direct_fees_installment_assign_id: DirectFeesInstallmentAssignId,
}

// -- DirectFeesSetting (per-school direct-fees configuration) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateDirectFeesSettingCommand {
    pub tenant: TenantContext,
    pub enabled: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateDirectFeesSettingCommand {
    pub tenant: TenantContext,
    pub direct_fees_setting_id: DirectFeesSettingId,
    pub enabled: Option<bool>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteDirectFeesSettingCommand {
    pub tenant: TenantContext,
    pub direct_fees_setting_id: DirectFeesSettingId,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateDirectFeesReminderCommand {
    pub tenant: TenantContext,
    pub direct_fees_reminder_id: DirectFeesReminderId,
    pub remind_at: Option<NaiveDate>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteDirectFeesReminderCommand {
    pub tenant: TenantContext,
    pub direct_fees_reminder_id: DirectFeesReminderId,
}

// -- PaymentMethod --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreatePaymentMethodCommand {
    pub tenant: TenantContext,
    pub name: String,
    pub kind: PaymentMethodKind,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdatePaymentMethodCommand {
    pub tenant: TenantContext,
    pub payment_method_id: PaymentMethodId,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeletePaymentMethodCommand {
    pub tenant: TenantContext,
    pub payment_method_id: PaymentMethodId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadPaymentMethodCommand {
    pub tenant: TenantContext,
    pub payment_method_id: PaymentMethodId,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeletePaymentGatewayCommand {
    pub tenant: TenantContext,
    pub payment_gateway_setting_id: PaymentGatewaySettingId,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateInvoiceCommand {
    pub tenant: TenantContext,
    pub fees_invoice_id: FeesInvoiceId,
    pub due_date: Option<NaiveDate>,
    pub amount_minor: Option<i64>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CancelInvoiceCommand {
    pub tenant: TenantContext,
    pub fees_invoice_id: FeesInvoiceId,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadInvoiceCommand {
    pub tenant: TenantContext,
    pub fees_invoice_id: FeesInvoiceId,
}

// -- FeesPayment (Reverse / Refund / Read) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReversePaymentCommand {
    pub tenant: TenantContext,
    pub fees_payment_id: FeesPaymentId,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RefundPaymentCommand {
    pub tenant: TenantContext,
    pub fees_payment_id: FeesPaymentId,
    pub amount_minor: i64,
    pub currency: Currency,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFeesPaymentCommand {
    pub tenant: TenantContext,
    pub fees_payment_id: FeesPaymentId,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteExpenseCommand {
    pub tenant: TenantContext,
    pub expense_id: ExpenseId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApproveExpenseCommand {
    pub tenant: TenantContext,
    pub expense_id: ExpenseId,
    pub approver_user_id: UserId,
    pub note: Option<String>,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteIncomeCommand {
    pub tenant: TenantContext,
    pub income_id: IncomeId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApproveIncomeCommand {
    pub tenant: TenantContext,
    pub income_id: IncomeId,
    pub approver_user_id: UserId,
    pub note: Option<String>,
}

// -- ExpenseHead (Update / Delete) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateExpenseHeadCommand {
    pub tenant: TenantContext,
    pub expense_head_id: ExpenseHeadId,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteExpenseHeadCommand {
    pub tenant: TenantContext,
    pub expense_head_id: ExpenseHeadId,
}

// -- IncomeHead (Create / Update / Delete) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateIncomeHeadCommand {
    pub tenant: TenantContext,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateIncomeHeadCommand {
    pub tenant: TenantContext,
    pub income_head_id: IncomeHeadId,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteIncomeHeadCommand {
    pub tenant: TenantContext,
    pub income_head_id: IncomeHeadId,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteBankAccountCommand {
    pub tenant: TenantContext,
    pub bank_account_id: BankAccountId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadBankAccountCommand {
    pub tenant: TenantContext,
    pub bank_account_id: BankAccountId,
}

// -- BankStatement (Read) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadBankStatementCommand {
    pub tenant: TenantContext,
    pub bank_statement_id: BankStatementId,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateBankSlipCommand {
    pub tenant: TenantContext,
    pub bank_payment_slip_id: BankPaymentSlipId,
    pub amount_minor: Option<i64>,
    pub slip_date: Option<NaiveDate>,
    pub note: Option<String>,
    pub payee_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApproveBankSlipCommand {
    pub tenant: TenantContext,
    pub bank_payment_slip_id: BankPaymentSlipId,
    pub approver_user_id: UserId,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadBankSlipCommand {
    pub tenant: TenantContext,
    pub bank_payment_slip_id: BankPaymentSlipId,
}

// -- Payroll (Generate / Approve / Pay / Read) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneratePayrollCommand {
    pub tenant: TenantContext,
    pub payroll_generate_id: educore_hr::value_objects::PayrollGenerateId,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApprovePayrollCommand {
    pub tenant: TenantContext,
    pub payroll_generate_id: educore_hr::value_objects::PayrollGenerateId,
    pub approver_user_id: UserId,
    pub note: Option<String>,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadPayrollCommand {
    pub tenant: TenantContext,
    pub payroll_generate_id: educore_hr::value_objects::PayrollGenerateId,
}

// -- PayrollPayment (Approve / Pay / Read) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApprovePayrollPaymentCommand {
    pub tenant: TenantContext,
    pub payroll_payment_id: PayrollPaymentId,
    pub approver_user_id: UserId,
    pub note: Option<String>,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadPayrollPaymentCommand {
    pub tenant: TenantContext,
    pub payroll_payment_id: PayrollPaymentId,
}

// -- Wallet (Read) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadWalletCommand {
    pub tenant: TenantContext,
    pub wallet_id: WalletId,
}

// -- WalletTransaction (Approve / Reject / Read) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApproveWalletTransactionCommand {
    pub tenant: TenantContext,
    pub wallet_transaction_id: WalletTransactionId,
    pub approver_user_id: UserId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RejectWalletTransactionCommand {
    pub tenant: TenantContext,
    pub wallet_transaction_id: WalletTransactionId,
    pub rejecter_user_id: UserId,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadWalletTransactionCommand {
    pub tenant: TenantContext,
    pub wallet_transaction_id: WalletTransactionId,
}

// -- FeesCarryForward (Read / Configure) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFeesCarryForwardCommand {
    pub tenant: TenantContext,
    pub fees_carry_forward_id: FeesCarryForwardId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigureFeesCarryForwardCommand {
    pub tenant: TenantContext,
    pub fees_carry_forward_setting_id: FeesCarryForwardSettingId,
    pub enabled: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFeesCarryForwardLogCommand {
    pub tenant: TenantContext,
    pub fees_carry_forward_log_id: FeesCarryForwardLogId,
}

// -- DueFeesLoginPrevent (Unblock / Read) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnblockLoginForDueFeesCommand {
    pub tenant: TenantContext,
    pub due_fees_login_prevent_id: DueFeesLoginPreventId,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadDueFeesBlockCommand {
    pub tenant: TenantContext,
    pub due_fees_login_prevent_id: DueFeesLoginPreventId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigureDueFeesBlockSettingCommand {
    pub tenant: TenantContext,
    pub days_overdue_threshold: i64,
    pub prevent_reason: PreventReason,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadAmountTransferCommand {
    pub tenant: TenantContext,
    pub amount_transfer_id: AmountTransferId,
}

// -- ChartOfAccount (read) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadChartOfAccountCommand {
    pub tenant: TenantContext,
    pub chart_of_account_id: ChartOfAccountId,
}

// -- InvoiceSetting (the school's invoice-numbering config; read) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadInvoiceSettingCommand {
    pub tenant: TenantContext,
    pub invoice_setting_id: InvoiceSettingId,
}

// -- FeesPaymentSlip (per-payment printable slip) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFeesPaymentSlipCommand {
    pub tenant: TenantContext,
    pub fees_payment_slip_id: FeesPaymentSlipId,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadOutstandingFeesReportCommand {
    pub tenant: TenantContext,
    pub as_of: NaiveDate,
    pub class_id: Option<crate::value_objects::ClassId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadExpenseReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub expense_head_id: Option<ExpenseHeadId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadIncomeReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub income_head_id: Option<IncomeHeadId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadBankStatementReportCommand {
    pub tenant: TenantContext,
    pub bank_account_id: BankAccountId,
    pub from: NaiveDate,
    pub to: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadWalletBalanceReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadPayrollReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub staff_id: Option<educore_hr::value_objects::StaffId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadPaymentMethodReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub payment_method_id: Option<PaymentMethodId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadFeesDiscountReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub fees_discount_id: Option<FeesDiscountId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadDueFeesReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub class_id: Option<crate::value_objects::ClassId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadClassWiseCollectionReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub class_id: crate::value_objects::ClassId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadDailyCollectionReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadMonthlyCollectionReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadHeadWiseExpenseReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub expense_head_id: ExpenseHeadId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadHeadWiseIncomeReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub income_head_id: IncomeHeadId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadCashFlowReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadProfitLossReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadBalanceSheetReportCommand {
    pub tenant: TenantContext,
    pub as_of: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadTrialBalanceReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadLedgerReportCommand {
    pub tenant: TenantContext,
    pub chart_of_account_id: ChartOfAccountId,
    pub from: NaiveDate,
    pub to: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadReceiptReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub fees_payment_id: Option<FeesPaymentId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadRefundReportCommand {
    pub tenant: TenantContext,
    pub from: NaiveDate,
    pub to: NaiveDate,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigureFeesTypeCommand {
    pub tenant: TenantContext,
    pub fees_group_id: crate::value_objects::FeesGroupId,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateExpenseHeadCommand {
    pub tenant: TenantContext,
    pub name: String,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockLoginForDueFeesCommand {
    pub tenant: TenantContext,
    pub user_id: UserId,
    pub reason: crate::value_objects::PreventReason,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CarryForwardFeesBalanceCommand {
    pub tenant: TenantContext,
    pub student_id: educore_academic::StudentId,
    pub from: educore_academic::AcademicYearId,
    pub to: educore_academic::AcademicYearId,
}
