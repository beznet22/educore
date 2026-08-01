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
pub mod entities;
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
        RealExpenseApproval, RealFeesCarryForwardLog, RealFeesCarryForwardSetting, RealFeesCarryForward, RealFeesMaster,
        RealFmFeesGroup, RealFmFeesInvoiceLineNote, RealFmFeesTransactionChild,
        RealFmFeesTransactionLineNote, RealIncomeApproval, RealIncomeHead, RealInvoiceSetting,
        RealBankPaymentSlipAudit, RealBankStatement, RealBankAccount, RealQuestionBankFee, RealSalaryTemplate, RealFeesDiscount, RealDirectFeesReminder, RealExpenseHead, RealFeesGroup, RealDueFeesLoginPrevent, DueFeesLoginPreventRole, RealFeesInvoiceSetting, RealFeesInstallmentCredit, FeesInstallmentCreditSource, RealFmFeesInvoiceSetting, RealFmFeesWeaver, RealDirectFeesInstallmentChildPayment, RealIncome, RealInventoryPayment, RealProductPurchase, RealFmFeesInvoice, RealFmFeesInvoiceChild, RealDirectFeesInstallmentAssign, RealTransaction, RealFeesInstallmentAssignDiscount, RealPaymentMethod, RealFeesInstallmentAssign, RealAmountTransfer, RealDirectFeesInstallment, RealFeesAssignDiscount, RealFeesAssign, RealFmFeesTransaction, RealFeesInstallment, RealFmFeesType, RealBankPaymentSlip, Wallet, WalletTransaction,
    };
    // Reference / child aggregates
    pub use crate::entities::{
        BankStatementAttachment, PayrollPaymentApproval, WalletTransactionApproval,
    };

    pub use crate::commands::{
        ApproveExpenseApprovalCommand, ApproveIncomeApprovalCommand,
        ApprovePayrollPaymentApprovalCommand,
        ApproveWalletTransactionApprovalCommand,
        BlockLoginForDueFeesCommand,
        UpdateBankStatementCommand, ReverseBankStatementCommand, RetireBankStatementCommand,
        CarryForwardFeesBalanceCommand, ConfigureFeesGroupCommand, ConfigureFeesTypeCommand,
        CreateBankPaymentSlipAuditCommand,
    CreateBankStatementAttachmentCommand,
    CreateBankStatementCommand,
    CreateFeesDiscountCommand,
        ConfigureInvoiceNumberingCommand, CreateChartOfAccountCommand,
        CreateSalaryTemplateCommand,
        CreateDirectFeesInstallmentAssignChildCommand, CreateDirectFeesSettingCommand,
        CreateDonorCommand, CreateExpenseApprovalCommand, CreateExpenseHeadCommand,
        CreateFeesCarryForwardLogCommand,
        CreateFeesCarryForwardSettingCommand,
        CreateIncomeApprovalCommand,
        CreatePayrollPaymentApprovalCommand,
        CreateFmFeesGroupCommand, CreateFmFeesInvoiceLineNoteCommand,
        CreateFmFeesTransactionLineNoteCommand, CreateFmFeesTransactionCommand,
        CreateFmFeesTypeCommand, ReadFmFeesTypeCommand, RetireFmFeesTypeCommand,
        RecordFeesAssignPaymentCommand, CancelFeesAssignCommand,
        CreateBankPaymentSlipCommand, ReadBankPaymentSlipCommand, RetireBankPaymentSlipCommand,
        ApproveBankPaymentSlipCommand, RejectBankPaymentSlipCommand,
        CreateFeesInstallmentCommand, ReadFeesInstallmentCommand, RetireFeesInstallmentCommand,
        ApproveFmFeesTransactionCommand, RejectFmFeesTransactionCommand,
        ApproveFmFeesInvoiceCommand, RejectFmFeesInvoiceCommand,
        ReadFmFeesTransactionCommand, RetireFmFeesTransactionCommand,
        CreateIncomeHeadCommand,
        CreateInvoiceSettingCommand, CreateQuestionBankFeeCommand, CreateWalletCommand,
        CreateWalletTransactionApprovalCommand, CreditWalletCommand,
        DeductWalletCreditCommand, OpenBankAccountCommand,
        UpdateBankAccountCommand, DeleteBankAccountCommand, ReadBankAccountCommand,
        CreateDirectFeesReminderCommand, UpdateDirectFeesReminderCommand, DeleteDirectFeesReminderCommand,
        UpdateExpenseHeadCommand, DeleteExpenseHeadCommand,
        CreateFeesGroupCommand, UpdateFeesGroupCommand, DeleteFeesGroupCommand,
        UnblockLoginForDueFeesCommand, ReadDueFeesBlockCommand,
        CreateFeesInvoiceSettingCommand, ReadFeesInvoiceSettingCommand, UpdateFeesInvoiceSettingCommand, DeleteFeesInvoiceSettingCommand,
        CreateFeesInstallmentCreditCommand, ReadFeesInstallmentCreditCommand, RetireFeesInstallmentCreditCommand,
        CreateFmFeesInvoiceSettingCommand, ReadFmFeesInvoiceSettingCommand, UpdateFmFeesInvoiceSettingCommand, RetireFmFeesInvoiceSettingCommand,
        CreateFmFeesWeaverCommand, ReadFmFeesWeaverCommand, RetireFmFeesWeaverCommand,
        CreateDirectFeesInstallmentChildPaymentCommand, ReadDirectFeesInstallmentChildPaymentCommand, RetireDirectFeesInstallmentChildPaymentCommand,
        CreateIncomeCommand, ReadIncomeCommand, RetireIncomeCommand,
        CreateInventoryPaymentCommand, ReadInventoryPaymentCommand, RetireInventoryPaymentCommand,
        CreateProductPurchaseCommand, ReadProductPurchaseCommand, RetireProductPurchaseCommand,
        CreateFmFeesInvoiceCommand, ReadFmFeesInvoiceCommand, RetireFmFeesInvoiceCommand,
        CreateFmFeesInvoiceChildCommand, ReadFmFeesInvoiceChildCommand, RetireFmFeesInvoiceChildCommand,
        CreateDirectFeesInstallmentAssignCommand, ReadDirectFeesInstallmentAssignCommand, RetireDirectFeesInstallmentAssignCommand,
        RecordExpenseCommand, RecordPaymentCommand, RejectExpenseApprovalCommand,
        RejectIncomeApprovalCommand, RejectPayrollPaymentApprovalCommand,
        RequestWalletRefundCommand,
        FINANCE_EXPENSE_DELETE_COMMAND_TYPE, FINANCE_EXPENSE_RECORD_COMMAND_TYPE,
        FINANCE_EXPENSE_UPDATE_COMMAND_TYPE, FINANCE_FEES_INVOICE_CONFIGURE_COMMAND_TYPE,
        FINANCE_FEES_PAYMENT_RECORD_COMMAND_TYPE, FINANCE_PAYROLL_PAYMENT_RECORD_COMMAND_TYPE,
        FINANCE_WALLET_CREATE_COMMAND_TYPE, FINANCE_WALLET_CREDIT_COMMAND_TYPE,
        FINANCE_WALLET_DEBIT_COMMAND_TYPE, FINANCE_WALLET_REFUND_REQUEST_COMMAND_TYPE,
        FINANCE_WALLET_TRANSACTION_APPROVE_COMMAND_TYPE,
        FINANCE_WALLET_TRANSACTION_REJECT_COMMAND_TYPE,
        FINANCE_FM_FEES_INVOICE_SETTING_CREATE_COMMAND_TYPE,
        FINANCE_FM_FEES_INVOICE_SETTING_READ_COMMAND_TYPE,
        FINANCE_FM_FEES_INVOICE_SETTING_UPDATE_COMMAND_TYPE,
        FINANCE_FM_FEES_INVOICE_SETTING_RETIRE_COMMAND_TYPE,
        FINANCE_FM_FEES_WEAVER_CREATE_COMMAND_TYPE,
        FINANCE_FM_FEES_WEAVER_READ_COMMAND_TYPE,
        FINANCE_FM_FEES_WEAVER_RETIRE_COMMAND_TYPE,
        FINANCE_DIRECT_FEES_INSTALLMENT_CHILD_PAYMENT_CREATE_COMMAND_TYPE,
        FINANCE_DIRECT_FEES_INSTALLMENT_CHILD_PAYMENT_READ_COMMAND_TYPE,
        FINANCE_DIRECT_FEES_INSTALLMENT_CHILD_PAYMENT_RETIRE_COMMAND_TYPE,
        FINANCE_INCOME_CREATE_COMMAND_TYPE,
        FINANCE_INCOME_READ_COMMAND_TYPE,
        FINANCE_INCOME_RETIRE_COMMAND_TYPE,
        FINANCE_INVENTORY_PAYMENT_CREATE_COMMAND_TYPE,
        FINANCE_INVENTORY_PAYMENT_READ_COMMAND_TYPE,
        FINANCE_INVENTORY_PAYMENT_RETIRE_COMMAND_TYPE,
        FINANCE_PRODUCT_PURCHASE_CREATE_COMMAND_TYPE,
        FINANCE_PRODUCT_PURCHASE_READ_COMMAND_TYPE,
        FINANCE_PRODUCT_PURCHASE_RETIRE_COMMAND_TYPE,
        FINANCE_FM_FEES_INVOICE_CREATE_COMMAND_TYPE,
        FINANCE_FM_FEES_INVOICE_READ_COMMAND_TYPE,
        FINANCE_FM_FEES_INVOICE_RETIRE_COMMAND_TYPE,
        FINANCE_FEES_CARRY_FORWARD_CREATE_COMMAND_TYPE,
        FINANCE_FEES_CARRY_FORWARD_RETIRE_COMMAND_TYPE,
        FINANCE_FEES_MASTER_RETIRE_COMMAND_TYPE,
        FINANCE_FM_FEES_INVOICE_CHILD_CREATE_COMMAND_TYPE,
        FINANCE_FM_FEES_INVOICE_CHILD_READ_COMMAND_TYPE,
        FINANCE_FM_FEES_INVOICE_CHILD_RETIRE_COMMAND_TYPE,
        FINANCE_DIRECT_FEES_INSTALLMENT_ASSIGN_CREATE_COMMAND_TYPE,
        FINANCE_DIRECT_FEES_INSTALLMENT_ASSIGN_READ_COMMAND_TYPE,
        FINANCE_DIRECT_FEES_INSTALLMENT_ASSIGN_RETIRE_COMMAND_TYPE,
        CreateTransactionCommand, ReadTransactionCommand, RetireTransactionCommand,
        FINANCE_TRANSACTION_CREATE_COMMAND_TYPE,
        FINANCE_TRANSACTION_READ_COMMAND_TYPE,
        FINANCE_TRANSACTION_RETIRE_COMMAND_TYPE,
        CreateFeesInstallmentAssignDiscountCommand, ReadFeesInstallmentAssignDiscountCommand, RetireFeesInstallmentAssignDiscountCommand,
        CreatePaymentMethodCommand, RetirePaymentMethodCommand, CreateFeesInstallmentAssignCommand, ReadFeesInstallmentAssignCommand, RetireFeesInstallmentAssignCommand, CreateAmountTransferCommand, ReadAmountTransferCommand, RetireAmountTransferCommand,
        CreateDirectFeesInstallmentCommand, RetireDirectFeesInstallmentCommand, CreateFeesAssignDiscountCommand, RetireFeesAssignDiscountCommand, CreateFeesAssignCommand, ReadFeesAssignCommand, RetireFeesAssignCommand,
        FINANCE_FEES_INSTALLMENT_ASSIGN_DISCOUNT_CREATE_COMMAND_TYPE,
        FINANCE_FEES_INSTALLMENT_ASSIGN_DISCOUNT_READ_COMMAND_TYPE,
        FINANCE_FEES_INSTALLMENT_ASSIGN_DISCOUNT_RETIRE_COMMAND_TYPE,
        FINANCE_PAYMENT_METHOD_CREATE_COMMAND_TYPE,
        FINANCE_PAYMENT_METHOD_READ_COMMAND_TYPE,
        FINANCE_PAYMENT_METHOD_RETIRE_COMMAND_TYPE,
        FINANCE_FEES_INSTALLMENT_ASSIGN_CREATE_COMMAND_TYPE,
        FINANCE_FEES_INSTALLMENT_ASSIGN_READ_COMMAND_TYPE,
        FINANCE_FEES_INSTALLMENT_ASSIGN_RETIRE_COMMAND_TYPE,
        FINANCE_AMOUNT_TRANSFER_CREATE_COMMAND_TYPE,
        FINANCE_AMOUNT_TRANSFER_READ_COMMAND_TYPE,
        FINANCE_AMOUNT_TRANSFER_RETIRE_COMMAND_TYPE,
        FINANCE_DIRECT_FEES_INSTALLMENT_RETIRE_COMMAND_TYPE, FINANCE_FEES_ASSIGN_DISCOUNT_CREATE_COMMAND_TYPE, FINANCE_FEES_ASSIGN_DISCOUNT_RETIRE_COMMAND_TYPE, FINANCE_FEES_ASSIGN_RETIRE_COMMAND_TYPE,
    };
    pub use crate::entities::WalletTransactionApproval as WalletTransactionApprovalEntity;
    pub use crate::errors::FinanceError;
    pub use crate::events::{
        ChartOfAccountCreated, ChartOfAccountDeleted, ChartOfAccountUpdated,
        DirectFeesInstallmentAssignChildAdded, DirectFeesInstallmentAssignChildRetired,
        DirectFeesSettingCreated, DonorCreated, ExpenseApprovalApproved, ExpenseApprovalCreated,
        ExpenseApprovalRejected, ExpenseRecorded, FeesCarryForwardLogCreated,
        FeesCarryForwardLogRetired, FeesCarryForwardSettingCreated, FeesMasterCreated, FeesMasterRetired, FeesCarryForwardCreated, FeesCarryForwardRetired,
        IncomeApprovalApproved, IncomeApprovalCreated, IncomeApprovalRejected,
        PayrollPaymentApprovalApproved, PayrollPaymentApprovalCreated,
        PayrollPaymentApprovalRejected,
        SalaryTemplateCreated, SalaryTemplateRetired, SalaryTemplateUpdated,
        BankPaymentSlipAuditCreated, BankPaymentSlipAuditRetired,
        BankStatementAttachmentCreated, BankStatementAttachmentRetired,
        BankStatementCreated, BankStatementReversed, BankStatementRetired, BankStatementUpdated,
        FeesDiscountCreated, FeesDiscountRetired, FeesDiscountUpdated,
        BankAccountCreated, BankAccountUpdated, BankAccountRetired,
        DirectFeesReminderCreated, DirectFeesReminderUpdated, DirectFeesReminderRetired,
        ExpenseHeadCreated, ExpenseHeadUpdated, ExpenseHeadRetired,
        FeesGroupCreated, FeesGroupUpdated, FeesGroupRetired,
        DueFeesLoginPreventCreated, DueFeesLoginPreventUpdated, DueFeesLoginPreventRetired, DueFeesLoginPreventPruned,
        FeesInvoiceSettingCreated, FeesInvoiceSettingUpdated, FeesInvoiceSettingRetired,
        FeesInstallmentCreditCreated, FeesInstallmentCreditRetired,
        FmFeesInvoiceSettingCreated, FmFeesInvoiceSettingUpdated, FmFeesInvoiceSettingRetired,
        FmFeesWeaverCreated, FmFeesWeaverRetired,
        DirectFeesInstallmentChildPaymentCreated, DirectFeesInstallmentChildPaymentRetired,
        IncomeCreated, IncomeRetired,
        InventoryPaymentCreated, InventoryPaymentRetired,
        ProductPurchaseCreated, ProductPurchaseRetired,
        FmFeesInvoiceCreated, FmFeesInvoiceRetired,
        FmFeesInvoiceChildCreated, FmFeesInvoiceChildRetired,
        DirectFeesInstallmentAssignCreated, DirectFeesInstallmentAssignRetired,
        TransactionCreated, TransactionRetired,
        FeesInstallmentAssignDiscountCreated, FeesInstallmentAssignDiscountRetired,
        PaymentMethodCreated, PaymentMethodRetired,
        FeesInstallmentAssignCreated, FeesInstallmentAssignRetired,
        AmountTransferCreated, AmountTransferRetired,
        DirectFeesInstallmentCreated, DirectFeesInstallmentRetired, FeesAssignDiscountCreated, FeesAssignDiscountRetired, FeesAssignCreated, FeesAssignRetired,
        FeesCarryForwardSettingRetired, FeesCarryForwardSettingUpdated, FmFeesGroupCreated, FmFeesInvoiceLineNoteCreated,
        FmFeesInvoiceLineNoteRetired, FmFeesTransactionChildCreated,
        FmFeesTransactionChildRetired, FmFeesTransactionChildUpdated,
        FmFeesTransactionLineNoteAdded, FmFeesTransactionLineNoteRetired, IncomeHeadCreated,
        FmFeesTransactionCreated, FmFeesTransactionRetired,
        FeesInstallmentCreated, FeesInstallmentRetired,
        FmFeesTransactionApproved, FmFeesTransactionRejected,
        FmFeesInvoiceApproved, FmFeesInvoiceRejected,
        FmFeesTypeCreated, FmFeesTypeRetired,
        BankPaymentSlipCreated, BankPaymentSlipApproved, BankPaymentSlipRejected, BankPaymentSlipRetired,
        FeesAssignPaymentRecorded, FeesAssignCancelled,
        InvoiceNumberingConfigured, InvoiceSettingCreated, PaymentReceived,
        PayrollPaymentRecorded, QuestionBankFeeCreated, WalletCreated, WalletCredited,
        WalletDebited, WalletRefundRequested, WalletTransactionApprovalApproved,
        WalletTransactionApprovalCreated, WalletTransactionApprovalRejected,
        WalletTransactionApproved, WalletTransactionRejected,
    };
    pub use crate::query::{FeesPaymentQuery, WalletQuery, WalletTransactionQuery};
    pub use crate::repository::{WalletRepository, WalletTransactionRepository};
    pub use crate::services::{
        approve_expense_approval, approve_income_approval, approve_payroll_payment_approval,
        approve_wallet_transaction, approve_wallet_transaction_approval,
        configure_invoice_numbering, create_chart_of_account,
        create_direct_fees_installment_assign_child, create_direct_fees_setting,
        create_bank_payment_slip_audit, create_bank_statement, create_bank_statement_attachment, create_donor, create_expense_approval, create_fees_carry_forward_log, create_fees_carry_forward_setting, create_fees_discount,
        create_fm_fees_group, create_income_approval, create_payroll_payment_approval,
        create_salary_template,
        create_fm_fees_invoice_line_note, create_fm_fees_transaction_child,
        create_fm_fees_transaction, read_fm_fees_transaction, retire_fm_fees_transaction,
        create_fees_installment, read_fees_installment, retire_fees_installment,
        approve_fm_fees_transaction, reject_fm_fees_transaction,
        approve_fm_fees_invoice, reject_fm_fees_invoice,
        create_fm_fees_type, read_fm_fees_type, retire_fm_fees_type,
        record_fees_assign_payment, cancel_fees_assign,
        create_bank_payment_slip, read_bank_payment_slip, retire_bank_payment_slip,
        approve_bank_payment_slip, reject_bank_payment_slip,
        create_fm_fees_transaction_line_note,
        create_income_head, create_invoice_setting, create_question_bank_fee, create_wallet,
        create_wallet_transaction_approval, credit_wallet, deduct_wallet_credit,
        record_expense, record_payment, reject_expense_approval, reject_income_approval,
        reject_payroll_payment_approval,
        reject_wallet_transaction,
        reject_wallet_transaction_approval,
        retire_bank_statement, reverse_bank_statement,
        open_bank_account, update_bank_account, retire_bank_account,
        create_direct_fees_reminder, update_direct_fees_reminder, retire_direct_fees_reminder,
        create_expense_head, update_expense_head, retire_expense_head,
        create_fees_group, update_fees_group, retire_fees_group,
        create_due_fees_login_prevent, update_due_fees_login_prevent, retire_due_fees_login_prevent, prune_due_fees_login_prevent,
        create_fees_invoice_setting, update_fees_invoice_setting, retire_fees_invoice_setting,
        create_fees_installment_credit, retire_fees_installment_credit,
        create_fm_fees_invoice_setting, read_fm_fees_invoice_setting, update_fm_fees_invoice_setting, retire_fm_fees_invoice_setting,
        create_fm_fees_weaver, read_fm_fees_weaver, retire_fm_fees_weaver,
        create_direct_fees_installment_child_payment, read_direct_fees_installment_child_payment, retire_direct_fees_installment_child_payment,
        create_income, read_income, retire_income,
        create_inventory_payment, read_inventory_payment, retire_inventory_payment,
        create_product_purchase, read_product_purchase, retire_product_purchase,
        create_fm_fees_invoice, read_fm_fees_invoice, retire_fm_fees_invoice, create_fees_carry_forward, read_fees_carry_forward, retire_fees_carry_forward, create_fees_master, read_fees_master, retire_fees_master,
        create_fm_fees_invoice_child, read_fm_fees_invoice_child, retire_fm_fees_invoice_child,
        create_direct_fees_installment_assign, read_direct_fees_installment_assign, retire_direct_fees_installment_assign,
        create_transaction, read_transaction, retire_transaction,
        create_fees_installment_assign_discount, read_fees_installment_assign_discount, retire_fees_installment_assign_discount,
        create_payment_method, read_payment_method, retire_payment_method,
        create_fees_installment_assign, read_fees_installment_assign, retire_fees_installment_assign,
        create_amount_transfer, read_amount_transfer, retire_amount_transfer,
        create_direct_fees_installment, read_direct_fees_installment, retire_direct_fees_installment, create_fees_assign_discount, read_fees_assign_discount, retire_fees_assign_discount, create_fees_assign, read_fees_assign, retire_fees_assign,
        request_wallet_refund, ChargeRequest, PaymentProvider, PaymentProviderPaymentId,
        PaymentProviderStatus, PaymentReceipt, PaymentStatus, RefundReceipt, RefundRequest,
        StubPaymentProvider, WalletService,
    };
    pub use crate::value_objects::{
        validate_bank_account_number, validate_discount_name, validate_donor_name,
        validate_ifsc_code, validate_ledger_name, validate_percentage, AccountType, Amount, AmountTransferId,
        ApprovalStatus, Balance, BalanceType, BankAccountId, BankMode, BankPaymentSlipId,
        ChartOfAccountId, Currency, DirectFeesInstallmentAssignId,
        DirectFeesInstallmentChildPaymentId, DirectFeesInstallmentId, DirectFeesReminderId,
        DirectFeesSettingId, DiscountAmount, DiscountType, DonorId, DueFeesLoginPreventId,
        ExpenseHeadId, ExpenseId, FeeAmount, FeesAssignDiscountId, FeesAssignId,
        FeesCarryForwardId, FeesCarryForwardLogId, FeesCarryForwardSettingId, FeesDiscountId,
        FeesGroupId, FeesInstallmentAssignDiscountId, FeesInstallmentAssignId, FeesInstallmentCreditId, FeesInstallmentId,
        FeesInvoiceId, FeesInvoiceSettingId, FeesMasterId, FeesPaymentFineId, FeesPaymentId,
        FeesPaymentSlipId, FeesPaymentStatus, FeesTypeId, FineAmount, FmFeesGroupId,
        FmFeesInvoiceChildId, FmFeesInvoiceId, FmFeesInvoiceSettingId, FmFeesTransactionChildId,
        FmFeesTransactionId, FmFeesTypeId, FmFeesTypeKind, FmFeesWeaverId, FmInvoiceType, GatewayMode,
        LifecycleStatus, PaymentMode,
        IncomeHeadId, IncomeId, InvoiceSettingId, Money, PaymentGatewaySettingId, PaymentMethodId,
        PaymentMethodKind, PayrollPaymentId, PreventReason, ProductPurchaseId, QuestionBankFeeId,
        StatementType, TransactionId, WalletId, WalletTransactionId, WalletTxStatus, WalletTxType,
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
