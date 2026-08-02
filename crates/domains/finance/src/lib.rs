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
        DueFeesLoginPreventRole, Expense, FeesInstallmentCreditSource, FeesInvoice, FeesPayment,
        RealAmountTransfer, RealBankAccount, RealBankPaymentSlip, RealBankPaymentSlipAudit,
        RealBankStatement, RealChartOfAccount, RealDirectFeesInstallment,
        RealDirectFeesInstallmentAssign, RealDirectFeesInstallmentAssignChild,
        RealDirectFeesInstallmentChildPayment, RealDirectFeesReminder, RealDirectFeesSetting,
        RealDonor, RealDueFeesLoginPrevent, RealExpenseApproval, RealExpenseHead, RealFeesAssign,
        RealFeesAssignDiscount, RealFeesCarryForward, RealFeesCarryForwardLog,
        RealFeesCarryForwardSetting, RealFeesDiscount, RealFeesGroup, RealFeesInstallment,
        RealFeesInstallmentAssign, RealFeesInstallmentAssignDiscount, RealFeesInstallmentCredit,
        RealFeesInvoiceSetting, RealFeesMaster, RealFmFeesGroup, RealFmFeesInvoice,
        RealFmFeesInvoiceChild, RealFmFeesInvoiceLineNote, RealFmFeesInvoiceSetting,
        RealFmFeesTransaction, RealFmFeesTransactionChild, RealFmFeesTransactionLineNote,
        RealFmFeesType, RealFmFeesWeaver, RealIncome, RealIncomeApproval, RealIncomeHead,
        RealInventoryPayment, RealInvoiceSetting, RealPaymentGatewaySetting, RealPaymentMethod,
        RealPayrollPayment, RealProductPurchase, RealQuestionBankFee, RealSalaryTemplate,
        RealTransaction, Wallet, WalletTransaction,
    };
    // Reference / child aggregates
    pub use crate::entities::{
        BankStatementAttachment, PayrollPaymentApproval, WalletTransactionApproval,
    };

    pub use crate::commands::{
        ApproveBankPaymentSlipCommand, ApproveExpenseApprovalCommand, ApproveFmFeesInvoiceCommand,
        ApproveFmFeesTransactionCommand, ApproveIncomeApprovalCommand,
        ApprovePayrollPaymentApprovalCommand, ApproveWalletTransactionApprovalCommand,
        BlockLoginForDueFeesCommand, CancelFeesAssignCommand, CancelFeesInstallmentAssignCommand,
        CancelProductPurchaseCommand, CarryForwardFeesBalanceCommand,
        CloseFeesInstallmentAssignCommand, ConfigureFeesGroupCommand, ConfigureFeesTypeCommand,
        ConfigureInvoiceNumberingCommand, ConfigurePaymentGatewayCommand,
        CreateAmountTransferCommand, CreateBankPaymentSlipAuditCommand,
        CreateBankPaymentSlipCommand, CreateBankStatementAttachmentCommand,
        CreateBankStatementCommand, CreateChartOfAccountCommand,
        CreateDirectFeesInstallmentAssignChildCommand, CreateDirectFeesInstallmentAssignCommand,
        CreateDirectFeesInstallmentChildPaymentCommand, CreateDirectFeesInstallmentCommand,
        CreateDirectFeesReminderCommand, CreateDirectFeesSettingCommand, CreateDonorCommand,
        CreateExpenseApprovalCommand, CreateExpenseHeadCommand, CreateFeesAssignCommand,
        CreateFeesAssignDiscountCommand, CreateFeesCarryForwardLogCommand,
        CreateFeesCarryForwardSettingCommand, CreateFeesDiscountCommand, CreateFeesGroupCommand,
        CreateFeesInstallmentAssignCommand, CreateFeesInstallmentAssignDiscountCommand,
        CreateFeesInstallmentCommand, CreateFeesInstallmentCreditCommand,
        CreateFeesInvoiceSettingCommand, CreateFmFeesGroupCommand, CreateFmFeesInvoiceChildCommand,
        CreateFmFeesInvoiceCommand, CreateFmFeesInvoiceLineNoteCommand,
        CreateFmFeesInvoiceSettingCommand, CreateFmFeesTransactionCommand,
        CreateFmFeesTransactionLineNoteCommand, CreateFmFeesTypeCommand, CreateFmFeesWeaverCommand,
        CreateIncomeApprovalCommand, CreateIncomeCommand, CreateIncomeHeadCommand,
        CreateInventoryPaymentCommand, CreateInvoiceSettingCommand, CreatePaymentMethodCommand,
        CreatePayrollPaymentApprovalCommand, CreateProductPurchaseCommand,
        CreateQuestionBankFeeCommand, CreateSalaryTemplateCommand, CreateTransactionCommand,
        CreateWalletCommand, CreateWalletTransactionApprovalCommand, CreditWalletCommand,
        DeductWalletCreditCommand, DeleteBankAccountCommand, DeleteDirectFeesReminderCommand,
        DeleteExpenseHeadCommand, DeleteFeesGroupCommand, DeleteFeesInvoiceSettingCommand,
        OpenBankAccountCommand, PostTransactionCommand, ReadAmountTransferCommand,
        ReadBankAccountCommand, ReadBankPaymentSlipCommand, ReadDirectFeesInstallmentAssignCommand,
        ReadDirectFeesInstallmentChildPaymentCommand, ReadDueFeesBlockCommand,
        ReadFeesAssignCommand, ReadFeesInstallmentAssignCommand,
        ReadFeesInstallmentAssignDiscountCommand, ReadFeesInstallmentCommand,
        ReadFeesInstallmentCreditCommand, ReadFeesInvoiceSettingCommand,
        ReadFmFeesInvoiceChildCommand, ReadFmFeesInvoiceCommand, ReadFmFeesInvoiceSettingCommand,
        ReadFmFeesTransactionCommand, ReadFmFeesTypeCommand, ReadFmFeesWeaverCommand,
        ReadIncomeCommand, ReadInventoryPaymentCommand, ReadProductPurchaseCommand,
        ReadTransactionCommand, RecordExpenseCommand, RecordFeesAssignPaymentCommand,
        RecordPaymentCommand, RecordProductPurchaseReceiptCommand, RejectBankPaymentSlipCommand,
        RejectExpenseApprovalCommand, RejectFmFeesInvoiceCommand, RejectFmFeesTransactionCommand,
        RejectIncomeApprovalCommand, RejectPayrollPaymentApprovalCommand,
        RequestWalletRefundCommand, RetireAmountTransferCommand, RetireBankPaymentSlipCommand,
        RetireBankStatementCommand, RetireDirectFeesInstallmentAssignCommand,
        RetireDirectFeesInstallmentChildPaymentCommand, RetireDirectFeesInstallmentCommand,
        RetireFeesAssignCommand, RetireFeesAssignDiscountCommand,
        RetireFeesInstallmentAssignCommand, RetireFeesInstallmentAssignDiscountCommand,
        RetireFeesInstallmentCommand, RetireFeesInstallmentCreditCommand,
        RetireFmFeesInvoiceChildCommand, RetireFmFeesInvoiceCommand,
        RetireFmFeesInvoiceSettingCommand, RetireFmFeesTransactionCommand, RetireFmFeesTypeCommand,
        RetireFmFeesWeaverCommand, RetireIncomeCommand, RetireInventoryPaymentCommand,
        RetirePaymentMethodCommand, RetireProductPurchaseCommand, RetireTransactionCommand,
        ReverseBankStatementCommand, UnblockLoginForDueFeesCommand, UpdateBankAccountCommand,
        UpdateBankStatementCommand, UpdateDirectFeesReminderCommand, UpdateExpenseHeadCommand,
        UpdateFeesGroupCommand, UpdateFeesInvoiceSettingCommand, UpdateFmFeesInvoiceSettingCommand,
        UpdatePaymentGatewayCommand, FINANCE_AMOUNT_TRANSFER_CREATE_COMMAND_TYPE,
        FINANCE_AMOUNT_TRANSFER_READ_COMMAND_TYPE, FINANCE_AMOUNT_TRANSFER_RETIRE_COMMAND_TYPE,
        FINANCE_DIRECT_FEES_INSTALLMENT_ASSIGN_CREATE_COMMAND_TYPE,
        FINANCE_DIRECT_FEES_INSTALLMENT_ASSIGN_READ_COMMAND_TYPE,
        FINANCE_DIRECT_FEES_INSTALLMENT_ASSIGN_RETIRE_COMMAND_TYPE,
        FINANCE_DIRECT_FEES_INSTALLMENT_CHILD_PAYMENT_CREATE_COMMAND_TYPE,
        FINANCE_DIRECT_FEES_INSTALLMENT_CHILD_PAYMENT_READ_COMMAND_TYPE,
        FINANCE_DIRECT_FEES_INSTALLMENT_CHILD_PAYMENT_RETIRE_COMMAND_TYPE,
        FINANCE_DIRECT_FEES_INSTALLMENT_RETIRE_COMMAND_TYPE, FINANCE_EXPENSE_DELETE_COMMAND_TYPE,
        FINANCE_EXPENSE_RECORD_COMMAND_TYPE, FINANCE_EXPENSE_UPDATE_COMMAND_TYPE,
        FINANCE_FEES_ASSIGN_DISCOUNT_CREATE_COMMAND_TYPE,
        FINANCE_FEES_ASSIGN_DISCOUNT_RETIRE_COMMAND_TYPE, FINANCE_FEES_ASSIGN_RETIRE_COMMAND_TYPE,
        FINANCE_FEES_CARRY_FORWARD_CREATE_COMMAND_TYPE,
        FINANCE_FEES_CARRY_FORWARD_RETIRE_COMMAND_TYPE,
        FINANCE_FEES_INSTALLMENT_ASSIGN_CREATE_COMMAND_TYPE,
        FINANCE_FEES_INSTALLMENT_ASSIGN_DISCOUNT_CREATE_COMMAND_TYPE,
        FINANCE_FEES_INSTALLMENT_ASSIGN_DISCOUNT_READ_COMMAND_TYPE,
        FINANCE_FEES_INSTALLMENT_ASSIGN_DISCOUNT_RETIRE_COMMAND_TYPE,
        FINANCE_FEES_INSTALLMENT_ASSIGN_READ_COMMAND_TYPE,
        FINANCE_FEES_INSTALLMENT_ASSIGN_RETIRE_COMMAND_TYPE,
        FINANCE_FEES_INVOICE_CONFIGURE_COMMAND_TYPE, FINANCE_FEES_MASTER_RETIRE_COMMAND_TYPE,
        FINANCE_FEES_PAYMENT_RECORD_COMMAND_TYPE,
        FINANCE_FM_FEES_INVOICE_CHILD_CREATE_COMMAND_TYPE,
        FINANCE_FM_FEES_INVOICE_CHILD_READ_COMMAND_TYPE,
        FINANCE_FM_FEES_INVOICE_CHILD_RETIRE_COMMAND_TYPE,
        FINANCE_FM_FEES_INVOICE_CREATE_COMMAND_TYPE, FINANCE_FM_FEES_INVOICE_READ_COMMAND_TYPE,
        FINANCE_FM_FEES_INVOICE_RETIRE_COMMAND_TYPE,
        FINANCE_FM_FEES_INVOICE_SETTING_CREATE_COMMAND_TYPE,
        FINANCE_FM_FEES_INVOICE_SETTING_READ_COMMAND_TYPE,
        FINANCE_FM_FEES_INVOICE_SETTING_RETIRE_COMMAND_TYPE,
        FINANCE_FM_FEES_INVOICE_SETTING_UPDATE_COMMAND_TYPE,
        FINANCE_FM_FEES_WEAVER_CREATE_COMMAND_TYPE, FINANCE_FM_FEES_WEAVER_READ_COMMAND_TYPE,
        FINANCE_FM_FEES_WEAVER_RETIRE_COMMAND_TYPE, FINANCE_INCOME_CREATE_COMMAND_TYPE,
        FINANCE_INCOME_READ_COMMAND_TYPE, FINANCE_INCOME_RETIRE_COMMAND_TYPE,
        FINANCE_INVENTORY_PAYMENT_CREATE_COMMAND_TYPE, FINANCE_INVENTORY_PAYMENT_READ_COMMAND_TYPE,
        FINANCE_INVENTORY_PAYMENT_RETIRE_COMMAND_TYPE, FINANCE_PAYMENT_METHOD_CREATE_COMMAND_TYPE,
        FINANCE_PAYMENT_METHOD_READ_COMMAND_TYPE, FINANCE_PAYMENT_METHOD_RETIRE_COMMAND_TYPE,
        FINANCE_PAYROLL_PAYMENT_RECORD_COMMAND_TYPE, FINANCE_PRODUCT_PURCHASE_CREATE_COMMAND_TYPE,
        FINANCE_PRODUCT_PURCHASE_READ_COMMAND_TYPE, FINANCE_PRODUCT_PURCHASE_RETIRE_COMMAND_TYPE,
        FINANCE_TRANSACTION_CREATE_COMMAND_TYPE, FINANCE_TRANSACTION_READ_COMMAND_TYPE,
        FINANCE_TRANSACTION_RETIRE_COMMAND_TYPE, FINANCE_WALLET_CREATE_COMMAND_TYPE,
        FINANCE_WALLET_CREDIT_COMMAND_TYPE, FINANCE_WALLET_DEBIT_COMMAND_TYPE,
        FINANCE_WALLET_REFUND_REQUEST_COMMAND_TYPE,
        FINANCE_WALLET_TRANSACTION_APPROVE_COMMAND_TYPE,
        FINANCE_WALLET_TRANSACTION_REJECT_COMMAND_TYPE,
    };
    pub use crate::entities::WalletTransactionApproval as WalletTransactionApprovalEntity;
    pub use crate::errors::FinanceError;
    pub use crate::events::{
        AmountTransferCreated, AmountTransferRetired, BankAccountCreated, BankAccountRetired,
        BankAccountUpdated, BankPaymentSlipApproved, BankPaymentSlipAuditCreated,
        BankPaymentSlipAuditRetired, BankPaymentSlipCreated, BankPaymentSlipRejected,
        BankPaymentSlipRetired, BankStatementAttachmentCreated, BankStatementAttachmentRetired,
        BankStatementCreated, BankStatementRetired, BankStatementReversed, BankStatementUpdated,
        ChartOfAccountCreated, ChartOfAccountDeleted, ChartOfAccountUpdated,
        DirectFeesInstallmentAssignChildAdded, DirectFeesInstallmentAssignChildRetired,
        DirectFeesInstallmentAssignCreated, DirectFeesInstallmentAssignRetired,
        DirectFeesInstallmentChildPaymentCreated, DirectFeesInstallmentChildPaymentRetired,
        DirectFeesInstallmentCreated, DirectFeesInstallmentRetired, DirectFeesReminderCreated,
        DirectFeesReminderRetired, DirectFeesReminderUpdated, DirectFeesSettingCreated,
        DonorCreated, DueFeesLoginPreventCreated, DueFeesLoginPreventPruned,
        DueFeesLoginPreventRetired, DueFeesLoginPreventUpdated, ExpenseApprovalApproved,
        ExpenseApprovalCreated, ExpenseApprovalRejected, ExpenseHeadCreated, ExpenseHeadRetired,
        ExpenseHeadUpdated, ExpenseRecorded, FeesAssignCancelled, FeesAssignCreated,
        FeesAssignDiscountCreated, FeesAssignDiscountRetired, FeesAssignPaymentRecorded,
        FeesAssignRetired, FeesCarryForwardCreated, FeesCarryForwardLogCreated,
        FeesCarryForwardLogRetired, FeesCarryForwardRetired, FeesCarryForwardSettingCreated,
        FeesCarryForwardSettingRetired, FeesCarryForwardSettingUpdated, FeesDiscountCreated,
        FeesDiscountRetired, FeesDiscountUpdated, FeesGroupCreated, FeesGroupRetired,
        FeesGroupUpdated, FeesInstallmentAssignCancelled, FeesInstallmentAssignClosed,
        FeesInstallmentAssignCreated, FeesInstallmentAssignDiscountCreated,
        FeesInstallmentAssignDiscountRetired, FeesInstallmentAssignRetired, FeesInstallmentCreated,
        FeesInstallmentCreditCreated, FeesInstallmentCreditRetired, FeesInstallmentRetired,
        FeesInvoiceSettingCreated, FeesInvoiceSettingRetired, FeesInvoiceSettingUpdated,
        FeesMasterCreated, FeesMasterRetired, FmFeesGroupCreated, FmFeesInvoiceApproved,
        FmFeesInvoiceChildCreated, FmFeesInvoiceChildRetired, FmFeesInvoiceCreated,
        FmFeesInvoiceLineNoteCreated, FmFeesInvoiceLineNoteRetired, FmFeesInvoiceRejected,
        FmFeesInvoiceRetired, FmFeesInvoiceSettingCreated, FmFeesInvoiceSettingRetired,
        FmFeesInvoiceSettingUpdated, FmFeesTransactionApproved, FmFeesTransactionChildCreated,
        FmFeesTransactionChildRetired, FmFeesTransactionChildUpdated, FmFeesTransactionCreated,
        FmFeesTransactionLineNoteAdded, FmFeesTransactionLineNoteRetired,
        FmFeesTransactionRejected, FmFeesTransactionRetired, FmFeesTypeCreated, FmFeesTypeRetired,
        FmFeesWeaverCreated, FmFeesWeaverRetired, IncomeApprovalApproved, IncomeApprovalCreated,
        IncomeApprovalRejected, IncomeCreated, IncomeHeadCreated, IncomeRetired,
        InventoryPaymentCreated, InventoryPaymentRetired, InvoiceNumberingConfigured,
        InvoiceSettingCreated, PaymentGatewayConfigured, PaymentGatewayDisabled,
        PaymentGatewayUpdated, PaymentMethodCreated, PaymentMethodRetired, PaymentReceived,
        PayrollPaymentApprovalApproved, PayrollPaymentApprovalCreated,
        PayrollPaymentApprovalRejected, PayrollPaymentRecorded, PayrollPaymentRetired,
        ProductPurchaseCancelled, ProductPurchaseCreated, ProductPurchaseReceived,
        ProductPurchaseRetired, QuestionBankFeeCreated, SalaryTemplateCreated,
        SalaryTemplateRetired, SalaryTemplateUpdated, TransactionCreated, TransactionPosted,
        TransactionRetired, WalletCreated, WalletCredited, WalletDebited, WalletRefundRequested,
        WalletTransactionApprovalApproved, WalletTransactionApprovalCreated,
        WalletTransactionApprovalRejected, WalletTransactionApproved, WalletTransactionRejected,
    };
    pub use crate::query::{FeesPaymentQuery, WalletQuery, WalletTransactionQuery};
    pub use crate::repository::{WalletRepository, WalletTransactionRepository};
    pub use crate::services::{
        approve_bank_payment_slip, approve_expense_approval, approve_fm_fees_invoice,
        approve_fm_fees_transaction, approve_income_approval, approve_payroll_payment_approval,
        approve_wallet_transaction, approve_wallet_transaction_approval, cancel_fees_assign,
        cancel_fees_installment_assign, cancel_product_purchase, close_fees_installment_assign,
        configure_invoice_numbering, create_amount_transfer, create_bank_payment_slip,
        create_bank_payment_slip_audit, create_bank_statement, create_bank_statement_attachment,
        create_chart_of_account, create_direct_fees_installment,
        create_direct_fees_installment_assign, create_direct_fees_installment_assign_child,
        create_direct_fees_installment_child_payment, create_direct_fees_reminder,
        create_direct_fees_setting, create_donor, create_due_fees_login_prevent,
        create_expense_approval, create_expense_head, create_fees_assign,
        create_fees_assign_discount, create_fees_carry_forward, create_fees_carry_forward_log,
        create_fees_carry_forward_setting, create_fees_discount, create_fees_group,
        create_fees_installment, create_fees_installment_assign,
        create_fees_installment_assign_discount, create_fees_installment_credit,
        create_fees_invoice_setting, create_fees_master, create_fm_fees_group,
        create_fm_fees_invoice, create_fm_fees_invoice_child, create_fm_fees_invoice_line_note,
        create_fm_fees_invoice_setting, create_fm_fees_transaction,
        create_fm_fees_transaction_child, create_fm_fees_transaction_line_note,
        create_fm_fees_type, create_fm_fees_weaver, create_income, create_income_approval,
        create_income_head, create_inventory_payment, create_invoice_setting,
        create_payment_method, create_payroll_payment_approval, create_product_purchase,
        create_question_bank_fee, create_salary_template, create_transaction, create_wallet,
        create_wallet_transaction_approval, credit_wallet, deduct_wallet_credit, open_bank_account,
        post_transaction, prune_due_fees_login_prevent, read_amount_transfer,
        read_bank_payment_slip, read_direct_fees_installment, read_direct_fees_installment_assign,
        read_direct_fees_installment_child_payment, read_fees_assign, read_fees_assign_discount,
        read_fees_carry_forward, read_fees_installment, read_fees_installment_assign,
        read_fees_installment_assign_discount, read_fees_master, read_fm_fees_invoice,
        read_fm_fees_invoice_child, read_fm_fees_invoice_setting, read_fm_fees_transaction,
        read_fm_fees_type, read_fm_fees_weaver, read_income, read_inventory_payment,
        read_payment_method, read_product_purchase, read_transaction, record_expense,
        record_fees_assign_payment, record_payment, record_product_purchase_receipt,
        reject_bank_payment_slip, reject_expense_approval, reject_fm_fees_invoice,
        reject_fm_fees_transaction, reject_income_approval, reject_payroll_payment_approval,
        reject_wallet_transaction, reject_wallet_transaction_approval, request_wallet_refund,
        retire_amount_transfer, retire_bank_account, retire_bank_payment_slip,
        retire_bank_statement, retire_direct_fees_installment,
        retire_direct_fees_installment_assign, retire_direct_fees_installment_child_payment,
        retire_direct_fees_reminder, retire_due_fees_login_prevent, retire_expense_head,
        retire_fees_assign, retire_fees_assign_discount, retire_fees_carry_forward,
        retire_fees_group, retire_fees_installment, retire_fees_installment_assign,
        retire_fees_installment_assign_discount, retire_fees_installment_credit,
        retire_fees_invoice_setting, retire_fees_master, retire_fm_fees_invoice,
        retire_fm_fees_invoice_child, retire_fm_fees_invoice_setting, retire_fm_fees_transaction,
        retire_fm_fees_type, retire_fm_fees_weaver, retire_income, retire_inventory_payment,
        retire_payment_method, retire_product_purchase, retire_transaction, reverse_bank_statement,
        update_bank_account, update_direct_fees_reminder, update_due_fees_login_prevent,
        update_expense_head, update_fees_group, update_fees_invoice_setting,
        update_fm_fees_invoice_setting, ChargeRequest, PaymentProvider, PaymentProviderPaymentId,
        PaymentProviderStatus, PaymentReceipt, PaymentStatus, RefundReceipt, RefundRequest,
        StubPaymentProvider, WalletService,
    };
    pub use crate::value_objects::{
        validate_bank_account_number, validate_discount_name, validate_donor_name,
        validate_ifsc_code, validate_ledger_name, validate_percentage, AccountType, Amount,
        AmountTransferId, ApprovalStatus, Balance, BalanceType, BankAccountId, BankMode,
        BankPaymentSlipId, ChartOfAccountId, Currency, DirectFeesInstallmentAssignId,
        DirectFeesInstallmentChildPaymentId, DirectFeesInstallmentId, DirectFeesReminderId,
        DirectFeesSettingId, DiscountAmount, DiscountType, DonorId, DueFeesLoginPreventId,
        ExpenseHeadId, ExpenseId, FeeAmount, FeesAssignDiscountId, FeesAssignId,
        FeesCarryForwardId, FeesCarryForwardLogId, FeesCarryForwardSettingId, FeesDiscountId,
        FeesGroupId, FeesInstallmentAssignDiscountId, FeesInstallmentAssignId,
        FeesInstallmentCreditId, FeesInstallmentId, FeesInvoiceId, FeesInvoiceSettingId,
        FeesMasterId, FeesPaymentFineId, FeesPaymentId, FeesPaymentSlipId, FeesPaymentStatus,
        FeesTypeId, FineAmount, FmFeesGroupId, FmFeesInvoiceChildId, FmFeesInvoiceId,
        FmFeesInvoiceSettingId, FmFeesTransactionChildId, FmFeesTransactionId, FmFeesTypeId,
        FmFeesTypeKind, FmFeesWeaverId, FmInvoiceType, GatewayChargeType, GatewayMode,
        IncomeHeadId, IncomeId, InvoiceSettingId, LifecycleStatus, Money, PaymentGatewaySettingId,
        PaymentMethodId, PaymentMethodKind, PaymentMode, PayrollPaymentId, PreventReason,
        ProductPurchaseId, ProductPurchaseLifecycleStatus, QuestionBankFeeId, StatementType,
        TransactionId, TransactionLifecycleStatus, WalletId, WalletTransactionId, WalletTxStatus,
        WalletTxType,
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
