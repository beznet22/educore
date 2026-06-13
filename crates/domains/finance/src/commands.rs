//! # Finance command structs and command-type constants
//!
//! Phase 7 ships the typed command shapes for the headline 6
//! aggregates (`Wallet`, `WalletTransaction`, `FeesInvoice`,
//! `FeesPayment`, `Expense`, `Refund`) plus the supporting
//! command-type constants the idempotency sub-port reads.

#![allow(missing_docs)]
#![allow(dead_code)]

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use educore_core::ids::UserId;
use educore_core::tenant::TenantContext;

use crate::value_objects::{BankAccountId, Currency, ExpenseHeadId, WalletId, WalletTxType};

// -- Command-type constants (the idempotency sub-port key) --

pub const FINANCE_WALLET_CREATE_COMMAND_TYPE: &str = "finance.wallet.create";
pub const FINANCE_WALLET_CREDIT_COMMAND_TYPE: &str = "finance.wallet.credit";
pub const FINANCE_WALLET_DEBIT_COMMAND_TYPE: &str = "finance.wallet.debit";
pub const FINANCE_WALLET_REFUND_REQUEST_COMMAND_TYPE: &str = "finance.wallet.refund_request";
pub const FINANCE_WALLET_TRANSACTION_APPROVE_COMMAND_TYPE: &str =
    "finance.wallet_transaction.approve";
pub const FINANCE_WALLET_TRANSACTION_REJECT_COMMAND_TYPE: &str =
    "finance.wallet_transaction.reject";

pub const FINANCE_FEES_INVOICE_CONFIGURE_COMMAND_TYPE: &str = "finance.fees_invoice.configure";
pub const FINANCE_FEES_PAYMENT_RECORD_COMMAND_TYPE: &str = "finance.fees_payment.record";

pub const FINANCE_EXPENSE_RECORD_COMMAND_TYPE: &str = "finance.expense.record";
pub const FINANCE_EXPENSE_UPDATE_COMMAND_TYPE: &str = "finance.expense.update";
pub const FINANCE_EXPENSE_DELETE_COMMAND_TYPE: &str = "finance.expense.delete";

pub const FINANCE_PAYROLL_PAYMENT_RECORD_COMMAND_TYPE: &str = "finance.payroll_payment.record";

// -- Re-exports of the canonical command shapes from services.rs --

pub use crate::services::{
    ConfigureInvoiceNumberingCommand, CreateWalletCommand, CreditWalletCommand,
    DeductWalletCreditCommand, RecordExpenseCommand, RecordPaymentCommand,
    RequestWalletRefundCommand,
};

// -- A few additional command shapes not yet in services.rs --

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
