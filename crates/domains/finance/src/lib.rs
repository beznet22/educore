//! # educore-finance
//!
//! Fees (group, type, master, assign, discount, invoice, payment),
//! banking (account, statement, slip), expenses, income, wallet,
//! payroll accounting, carry-forward, late-fee computation, and
//! the HR→finance payroll bridge.
//!
//! This crate is a member of the Educore workspace. See
//! `docs/architecture.md` and the domain spec in
//! `docs/specs/finance/` for behavioral details.

#![forbid(unsafe_code)]
#![allow(unused_imports)]
#![deny(missing_docs)]

/// Package name constant. Re-exported so consumers can assert they
/// are using the right crate version at compile time.
pub const PACKAGE_NAME: &str = "educore-finance";

/// Package version at compile time.
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod value_objects;

mod aggregate;
pub mod commands;
mod entities;
mod errors;
pub mod events;
pub mod query;
mod repository;
pub mod services;

// Prelude: re-export the engine-wide types the finance services reach for.
#[allow(missing_docs)]
pub mod prelude {
    pub use chrono::NaiveDate;
    pub use educore_core::clock::{Clock, IdGenerator, SystemClock, SystemIdGen};
    pub use educore_core::error::{DomainError, Result};
    pub use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
    pub use educore_core::tenant::TenantContext;
    pub use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};
    pub use educore_events::domain_event::DomainEvent;
    pub use educore_events::envelope::EventEnvelope;
    pub use educore_rbac::value_objects::Capability;

    // Headline 6 aggregates
    pub use crate::aggregate::{
        Expense, FeesInvoice, FeesPayment, RealChartOfAccount,
        RealDirectFeesInstallmentAssignChild, RealDirectFeesSetting, RealDonor,
        RealExpenseApproval, RealFeesCarryForwardLog, RealFeesCarryForwardSetting,
        RealFmFeesGroup, RealFmFeesInvoiceLineNote, RealFmFeesTransactionChild,
        RealFmFeesTransactionLineNote, RealIncomeApproval, RealIncomeHead, RealInvoiceSetting,
        RealQuestionBankFee, Wallet, WalletTransaction,
    };
    // Reference / child aggregates
    pub use crate::entities::WalletTransactionApproval;

    pub use crate::commands::{
        ApproveExpenseApprovalCommand, ApproveIncomeApprovalCommand,
        ApproveWalletTransactionApprovalCommand,
        BlockLoginForDueFeesCommand,
        CarryForwardFeesBalanceCommand, ConfigureFeesGroupCommand, ConfigureFeesTypeCommand,
        ConfigureInvoiceNumberingCommand, CreateChartOfAccountCommand,
        CreateDirectFeesInstallmentAssignChildCommand, CreateDirectFeesSettingCommand,
        CreateDonorCommand, CreateExpenseApprovalCommand, CreateExpenseHeadCommand,
        CreateFeesCarryForwardLogCommand,
        CreateFeesCarryForwardSettingCommand,
        CreateIncomeApprovalCommand,
        CreateFmFeesGroupCommand, CreateFmFeesInvoiceLineNoteCommand,
        CreateFmFeesTransactionLineNoteCommand, CreateIncomeHeadCommand,
        CreateInvoiceSettingCommand, CreateQuestionBankFeeCommand, CreateWalletCommand,
        CreateWalletTransactionApprovalCommand, CreditWalletCommand,
        DeductWalletCreditCommand, OpenBankAccountCommand,
        RecordExpenseCommand, RecordPaymentCommand, RejectExpenseApprovalCommand,
        RejectIncomeApprovalCommand,
        RequestWalletRefundCommand,
        FINANCE_EXPENSE_DELETE_COMMAND_TYPE, FINANCE_EXPENSE_RECORD_COMMAND_TYPE,
        FINANCE_EXPENSE_UPDATE_COMMAND_TYPE, FINANCE_FEES_INVOICE_CONFIGURE_COMMAND_TYPE,
        FINANCE_FEES_PAYMENT_RECORD_COMMAND_TYPE, FINANCE_PAYROLL_PAYMENT_RECORD_COMMAND_TYPE,
        FINANCE_WALLET_CREATE_COMMAND_TYPE, FINANCE_WALLET_CREDIT_COMMAND_TYPE,
        FINANCE_WALLET_DEBIT_COMMAND_TYPE, FINANCE_WALLET_REFUND_REQUEST_COMMAND_TYPE,
        FINANCE_WALLET_TRANSACTION_APPROVE_COMMAND_TYPE,
        FINANCE_WALLET_TRANSACTION_REJECT_COMMAND_TYPE,
    };
    pub use crate::entities::WalletTransactionApproval as WalletTransactionApprovalEntity;
    pub use crate::errors::FinanceError;
    pub use crate::events::{
        ChartOfAccountCreated, ChartOfAccountDeleted, ChartOfAccountUpdated,
        DirectFeesInstallmentAssignChildAdded, DirectFeesInstallmentAssignChildRetired,
        DirectFeesSettingCreated, DonorCreated, ExpenseApprovalApproved, ExpenseApprovalCreated,
        ExpenseApprovalRejected, ExpenseRecorded, FeesCarryForwardLogCreated,
        FeesCarryForwardLogRetired, FeesCarryForwardSettingCreated,
        IncomeApprovalApproved, IncomeApprovalCreated, IncomeApprovalRejected,
        FeesCarryForwardSettingRetired, FeesCarryForwardSettingUpdated, FmFeesGroupCreated, FmFeesInvoiceLineNoteCreated,
        FmFeesInvoiceLineNoteRetired, FmFeesTransactionChildCreated,
        FmFeesTransactionChildRetired, FmFeesTransactionChildUpdated,
        FmFeesTransactionLineNoteAdded, FmFeesTransactionLineNoteRetired, IncomeHeadCreated,
        InvoiceNumberingConfigured, InvoiceSettingCreated, PaymentReceived,
        PayrollPaymentRecorded, QuestionBankFeeCreated, WalletCreated, WalletCredited,
        WalletDebited, WalletRefundRequested, WalletTransactionApprovalApproved,
        WalletTransactionApprovalCreated, WalletTransactionApprovalRejected,
        WalletTransactionApproved, WalletTransactionRejected,
    };
    pub use crate::query::{FeesPaymentQuery, WalletQuery, WalletTransactionQuery};
    pub use crate::repository::{WalletRepository, WalletTransactionRepository};
    pub use crate::services::{
        approve_expense_approval, approve_income_approval, approve_wallet_transaction, approve_wallet_transaction_approval,
        configure_invoice_numbering, create_chart_of_account,
        create_direct_fees_installment_assign_child, create_direct_fees_setting,
        create_donor, create_expense_approval, create_fees_carry_forward_log, create_fees_carry_forward_setting,
        create_fm_fees_group, create_income_approval,
        create_fm_fees_invoice_line_note, create_fm_fees_transaction_child,
        create_fm_fees_transaction_line_note,
        create_income_head, create_invoice_setting, create_question_bank_fee, create_wallet,
        create_wallet_transaction_approval, credit_wallet, deduct_wallet_credit,
        record_expense, record_payment, reject_expense_approval, reject_income_approval,
        reject_wallet_transaction,
        reject_wallet_transaction_approval,
        request_wallet_refund, ChargeRequest, PaymentProvider, PaymentProviderPaymentId,
        PaymentProviderStatus, PaymentReceipt, PaymentStatus, RefundReceipt, RefundRequest,
        StubPaymentProvider, WalletService,
    };
    pub use crate::value_objects::{
        validate_bank_account_number, validate_discount_name, validate_donor_name,
        validate_ifsc_code, validate_ledger_name, validate_percentage, AccountType, Amount,
        ApprovalStatus, Balance, BalanceType, BankAccountId, BankMode, BankPaymentSlipId,
        ChartOfAccountId, Currency, DirectFeesInstallmentAssignId,
        DirectFeesInstallmentChildPaymentId, DirectFeesInstallmentId, DirectFeesReminderId,
        DirectFeesSettingId, DiscountAmount, DiscountType, DonorId, DueFeesLoginPreventId,
        ExpenseHeadId, ExpenseId, FeeAmount, FeesAssignDiscountId, FeesAssignId,
        FeesCarryForwardId, FeesCarryForwardLogId, FeesCarryForwardSettingId, FeesDiscountId,
        FeesGroupId, FeesInstallmentAssignId, FeesInstallmentCreditId, FeesInstallmentId,
        FeesInvoiceId, FeesInvoiceSettingId, FeesMasterId, FeesPaymentFineId, FeesPaymentId,
        FeesPaymentSlipId, FeesPaymentStatus, FeesTypeId, FineAmount, FmFeesGroupId,
        FmFeesInvoiceChildId, FmFeesInvoiceId, FmFeesInvoiceSettingId, FmFeesTransactionChildId,
        FmFeesTransactionId, FmFeesTypeId, FmFeesWeaverId, FmInvoiceType, GatewayMode,
        IncomeHeadId, IncomeId, InvoiceSettingId, Money, PaymentGatewaySettingId, PaymentMethodId,
        PaymentMethodKind, PayrollPaymentId, PreventReason, ProductPurchaseId, QuestionBankFeeId,
        StatementType, WalletId, WalletTransactionId, WalletTxStatus, WalletTxType,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn package_metadata_is_set() {
        assert_eq!(PACKAGE_NAME, "educore-finance");
        assert!(!PACKAGE_VERSION.is_empty());
    }
}
