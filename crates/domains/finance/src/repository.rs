//! # Finance repository ports
//!
//! Phase 7 ships 62 `#[async_trait]` repository port traits — one
//! per finance aggregate. Storage adapters (PG/MySQL/SQLite)
//! implement these in Phase 17 (production hardening); the test
//! fixtures in this crate use in-memory implementations matching
//! the Phase 5/6 pattern.
//!
//! Every trait follows the same shape:
//!
//! - `get(&TenantContext, TypedId) -> Result<Option<Aggregate>>`
//! - `list_for_school(SchoolId) -> Result<Vec<Aggregate>>`
//! - `insert(&TenantContext, &Aggregate) -> Result<()>`
//! - `update(&TenantContext, &Aggregate) -> Result<()>`
//!
//! Plus 1-3 per-aggregate lookups (`find_by_*` / `list_for_*`) for
//! the indices the domain services reach for most often.

#![allow(missing_docs)]
#![allow(unused_imports)]
#![allow(dead_code)]

use async_trait::async_trait;

use educore_core::error::Result;
use educore_core::ids::SchoolId;
use educore_core::tenant::TenantContext;

use crate::aggregate::{
    AmountTransfer, BankAccount, BankPaymentSlip, BankPaymentSlipAudit, BankStatement,
    BankStatementAttachment, ChartOfAccount, DirectFeesInstallment, DirectFeesInstallmentAssign,
    DirectFeesInstallmentAssignChild, DirectFeesInstallmentChildPayment, DirectFeesReminder,
    DirectFeesSetting, Donor, DueFeesLoginPrevent, Expense, ExpenseApproval, ExpenseHead,
    FeesAssign, FeesAssignDiscount, FeesCarryForward, FeesCarryForwardLog, FeesCarryForwardSetting,
    FeesDiscount, FeesGroup, FeesInstallment, FeesInstallmentAssign, FeesInstallmentAssignDiscount,
    FeesInstallmentCredit, FeesInvoice, FeesInvoiceSetting, FeesMaster, FeesPayment, FeesType,
    FmFeesGroup, FmFeesInvoice, FmFeesInvoiceChild, FmFeesInvoiceLineNote, FmFeesInvoiceSetting,
    FmFeesTransaction, FmFeesTransactionChild, FmFeesTransactionLineNote, FmFeesType, FmFeesWeaver,
    Income, IncomeApproval, IncomeHead, InventoryPayment, InvoiceSetting, PaymentGatewaySetting,
    PaymentMethod, PayrollEarnDeduc, PayrollGenerate, PayrollPayment, PayrollPaymentApproval,
    ProductPurchase, QuestionBankFee, SalaryTemplate, Transaction, Wallet, WalletTransaction,
    WalletTransactionApproval,
};
use crate::value_objects::{
    AcademicYearId, AmountTransferId, BankAccountId, BankPaymentSlipAuditId, BankPaymentSlipId,
    BankStatementAttachmentId, BankStatementId, ChartOfAccountId, ClassId,
    DirectFeesInstallmentAssignChildId, DirectFeesInstallmentAssignId,
    DirectFeesInstallmentChildPaymentId, DirectFeesInstallmentId, DirectFeesReminderId,
    DirectFeesSettingId, DonorId, DueFeesLoginPreventId, ExpenseApprovalId, ExpenseHeadId,
    ExpenseId, FeesAssignDiscountId, FeesAssignId, FeesCarryForwardId, FeesCarryForwardLogId,
    FeesCarryForwardSettingId, FeesDiscountId, FeesGroupId, FeesInstallmentAssignDiscountId,
    FeesInstallmentAssignId, FeesInstallmentCreditId, FeesInstallmentId, FeesInvoiceId,
    FeesInvoiceSettingId, FeesMasterId, FeesPaymentId, FeesTypeId, FmFeesGroupId,
    FmFeesInvoiceChildId, FmFeesInvoiceId, FmFeesInvoiceLineNoteId, FmFeesInvoiceSettingId,
    FmFeesTransactionChildId, FmFeesTransactionId, FmFeesTransactionLineNoteId, FmFeesTypeId,
    FmFeesWeaverId, GatewayMode, IncomeApprovalId, IncomeHeadId, IncomeId, InventoryPaymentId,
    InvoiceSettingId, PaymentGatewaySettingId, PaymentMethodId, PaymentMethodKind,
    PayrollEarnDeducId, PayrollGenerateId, PayrollPaymentApprovalId, PayrollPaymentId,
    ProductPurchaseId, QuestionBankFeeId, SalaryTemplateId, StaffId, StudentId, SubjectId,
    TransactionId, WalletId, WalletTransactionApprovalId, WalletTransactionId,
};

// =============================================================================
// Headline 1: Wallet
// =============================================================================

#[async_trait]
pub trait WalletRepository: Send + Sync {
    /// Look up a wallet by id.
    async fn get(&self, ctx: &TenantContext, id: WalletId) -> Result<Option<Wallet>>;

    /// Look up a wallet by `(school_id, user_id)` (the canonical
    /// index for "find this user's wallet").
    async fn get_by_user(
        &self,
        school: SchoolId,
        user_id: educore_core::ids::UserId,
    ) -> Result<Option<Wallet>>;

    /// List all wallets in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<Wallet>>;

    /// List all wallets belonging to a user across schools (rare;
    /// used by the due-fees login-prevention scan).
    async fn list_for_user(&self, user_id: educore_core::ids::UserId) -> Result<Vec<Wallet>>;

    /// Insert a new wallet.
    async fn insert(&self, ctx: &TenantContext, w: &Wallet) -> Result<()>;

    /// Update an existing wallet.
    async fn update(&self, ctx: &TenantContext, w: &Wallet) -> Result<()>;
}

// =============================================================================
// Headline 2: WalletTransaction
// =============================================================================

#[async_trait]
pub trait WalletTransactionRepository: Send + Sync {
    /// Look up a wallet transaction by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: WalletTransactionId,
    ) -> Result<Option<WalletTransaction>>;

    /// List all transactions for a wallet, newest first.
    async fn list_for_wallet(&self, wallet_id: WalletId) -> Result<Vec<WalletTransaction>>;

    /// List all approved transactions for a wallet (used by the
    /// `WalletService::balance` cross-check helper).
    async fn list_approved_for_wallet(&self, wallet_id: WalletId)
        -> Result<Vec<WalletTransaction>>;

    /// List all pending transactions in a school (used by the
    /// approval inbox).
    async fn list_pending(&self, school: SchoolId) -> Result<Vec<WalletTransaction>>;

    /// Insert a new wallet transaction.
    async fn insert(&self, ctx: &TenantContext, tx: &WalletTransaction) -> Result<()>;

    /// Update an existing wallet transaction.
    async fn update(&self, ctx: &TenantContext, tx: &WalletTransaction) -> Result<()>;
}

// =============================================================================
// 3: FeesGroup
// =============================================================================

#[async_trait]
pub trait FeesGroupRepository: Send + Sync {
    /// Look up a fees group by id.
    async fn get(&self, ctx: &TenantContext, id: FeesGroupId) -> Result<Option<FeesGroup>>;

    /// List all fees groups in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<FeesGroup>>;

    /// Find a fees group by its (school-scoped) unique name.
    async fn find_by_name(&self, school: SchoolId, name: &str) -> Result<Option<FeesGroup>>;

    /// Insert a new fees group.
    async fn insert(&self, ctx: &TenantContext, agg: &FeesGroup) -> Result<()>;

    /// Update an existing fees group.
    async fn update(&self, ctx: &TenantContext, agg: &FeesGroup) -> Result<()>;
}

// =============================================================================
// 4: FeesType
// =============================================================================

#[async_trait]
pub trait FeesTypeRepository: Send + Sync {
    /// Look up a fees type by id.
    async fn get(&self, ctx: &TenantContext, id: FeesTypeId) -> Result<Option<FeesType>>;

    /// List all fees types in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<FeesType>>;

    /// List all fees types that belong to a fees group.
    async fn list_for_group(&self, group_id: FeesGroupId) -> Result<Vec<FeesType>>;

    /// Find a fees type by its (school-scoped) unique name.
    async fn find_by_name(&self, school: SchoolId, name: &str) -> Result<Option<FeesType>>;

    /// Insert a new fees type.
    async fn insert(&self, ctx: &TenantContext, agg: &FeesType) -> Result<()>;

    /// Update an existing fees type.
    async fn update(&self, ctx: &TenantContext, agg: &FeesType) -> Result<()>;
}

// =============================================================================
// 5: FeesMaster
// =============================================================================

#[async_trait]
pub trait FeesMasterRepository: Send + Sync {
    /// Look up a fees master by id.
    async fn get(&self, ctx: &TenantContext, id: FeesMasterId) -> Result<Option<FeesMaster>>;

    /// List all fees masters in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<FeesMaster>>;

    /// List all fees masters for a class.
    async fn list_for_class(&self, class_id: ClassId) -> Result<Vec<FeesMaster>>;

    /// Find the fees master for a `(class, academic_year)` pair.
    async fn find_by_class_and_year(
        &self,
        class_id: ClassId,
        academic_year_id: AcademicYearId,
    ) -> Result<Option<FeesMaster>>;

    /// Insert a new fees master.
    async fn insert(&self, ctx: &TenantContext, agg: &FeesMaster) -> Result<()>;

    /// Update an existing fees master.
    async fn update(&self, ctx: &TenantContext, agg: &FeesMaster) -> Result<()>;
}

// =============================================================================
// 6: FeesDiscount
// =============================================================================

#[async_trait]
pub trait FeesDiscountRepository: Send + Sync {
    /// Look up a fees discount by id.
    async fn get(&self, ctx: &TenantContext, id: FeesDiscountId) -> Result<Option<FeesDiscount>>;

    /// List all fees discounts in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<FeesDiscount>>;

    /// Find a fees discount by its (school-scoped) unique name.
    async fn find_by_name(&self, school: SchoolId, name: &str) -> Result<Option<FeesDiscount>>;

    /// Insert a new fees discount.
    async fn insert(&self, ctx: &TenantContext, agg: &FeesDiscount) -> Result<()>;

    /// Update an existing fees discount.
    async fn update(&self, ctx: &TenantContext, agg: &FeesDiscount) -> Result<()>;
}

// =============================================================================
// 7: FeesAssign
// =============================================================================

#[async_trait]
pub trait FeesAssignRepository: Send + Sync {
    /// Look up a fees assignment by id.
    async fn get(&self, ctx: &TenantContext, id: FeesAssignId) -> Result<Option<FeesAssign>>;

    /// List all fees assignments in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<FeesAssign>>;

    /// List all fees assignments for a student.
    async fn list_for_student(&self, student_id: StudentId) -> Result<Vec<FeesAssign>>;

    /// List all fees assignments for a class.
    async fn list_for_class(&self, class_id: ClassId) -> Result<Vec<FeesAssign>>;

    /// Insert a new fees assignment.
    async fn insert(&self, ctx: &TenantContext, agg: &FeesAssign) -> Result<()>;

    /// Update an existing fees assignment.
    async fn update(&self, ctx: &TenantContext, agg: &FeesAssign) -> Result<()>;
}

// =============================================================================
// 8: FeesAssignDiscount
// =============================================================================

#[async_trait]
pub trait FeesAssignDiscountRepository: Send + Sync {
    /// Look up a fees-assign discount by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: FeesAssignDiscountId,
    ) -> Result<Option<FeesAssignDiscount>>;

    /// List all fees-assign discounts in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<FeesAssignDiscount>>;

    /// List all fees-assign discounts attached to a fees assignment.
    async fn list_for_assign(&self, assign_id: FeesAssignId) -> Result<Vec<FeesAssignDiscount>>;

    /// List all fees-assign discounts applied to a student.
    async fn list_for_student(&self, student_id: StudentId) -> Result<Vec<FeesAssignDiscount>>;

    /// Insert a new fees-assign discount.
    async fn insert(&self, ctx: &TenantContext, agg: &FeesAssignDiscount) -> Result<()>;

    /// Update an existing fees-assign discount.
    async fn update(&self, ctx: &TenantContext, agg: &FeesAssignDiscount) -> Result<()>;
}

// =============================================================================
// 9: FeesInvoiceSetting
// =============================================================================

#[async_trait]
pub trait FeesInvoiceSettingRepository: Send + Sync {
    /// Look up a fees invoice setting by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: FeesInvoiceSettingId,
    ) -> Result<Option<FeesInvoiceSetting>>;

    /// List all fees invoice settings in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<FeesInvoiceSetting>>;

    /// Get the (singleton) fees invoice setting for a school.
    async fn get_for_school(&self, school: SchoolId) -> Result<Option<FeesInvoiceSetting>>;

    /// Insert a new fees invoice setting.
    async fn insert(&self, ctx: &TenantContext, agg: &FeesInvoiceSetting) -> Result<()>;

    /// Update an existing fees invoice setting.
    async fn update(&self, ctx: &TenantContext, agg: &FeesInvoiceSetting) -> Result<()>;
}

// =============================================================================
// 10: InvoiceSetting
// =============================================================================

#[async_trait]
pub trait InvoiceSettingRepository: Send + Sync {
    /// Look up an invoice setting by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: InvoiceSettingId,
    ) -> Result<Option<InvoiceSetting>>;

    /// List all invoice settings in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<InvoiceSetting>>;

    /// Get the (singleton) invoice setting for a school.
    async fn get_for_school(&self, school: SchoolId) -> Result<Option<InvoiceSetting>>;

    /// Insert a new invoice setting.
    async fn insert(&self, ctx: &TenantContext, agg: &InvoiceSetting) -> Result<()>;

    /// Update an existing invoice setting.
    async fn update(&self, ctx: &TenantContext, agg: &InvoiceSetting) -> Result<()>;
}

// =============================================================================
// 11: FeesInstallment
// =============================================================================

#[async_trait]
pub trait FeesInstallmentRepository: Send + Sync {
    /// Look up a fees installment plan by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: FeesInstallmentId,
    ) -> Result<Option<FeesInstallment>>;

    /// List all fees installment plans in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<FeesInstallment>>;

    /// List all fees installment plans for an academic year.
    async fn list_for_year(&self, academic_year_id: AcademicYearId)
        -> Result<Vec<FeesInstallment>>;

    /// Find a fees installment plan by its (school-scoped) name.
    async fn find_by_name(&self, school: SchoolId, name: &str) -> Result<Option<FeesInstallment>>;

    /// Insert a new fees installment plan.
    async fn insert(&self, ctx: &TenantContext, agg: &FeesInstallment) -> Result<()>;

    /// Update an existing fees installment plan.
    async fn update(&self, ctx: &TenantContext, agg: &FeesInstallment) -> Result<()>;
}

// =============================================================================
// 12: FeesInstallmentAssign
// =============================================================================

#[async_trait]
pub trait FeesInstallmentAssignRepository: Send + Sync {
    /// Look up a fees-installment assignment by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: FeesInstallmentAssignId,
    ) -> Result<Option<FeesInstallmentAssign>>;

    /// List all fees-installment assignments in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<FeesInstallmentAssign>>;

    /// List all fees-installment assignments for a student.
    async fn list_for_student(&self, student_id: StudentId) -> Result<Vec<FeesInstallmentAssign>>;

    /// List all fees-installment assignments for a fees installment plan.
    async fn list_for_installment(
        &self,
        installment_id: FeesInstallmentId,
    ) -> Result<Vec<FeesInstallmentAssign>>;

    /// Insert a new fees-installment assignment.
    async fn insert(&self, ctx: &TenantContext, agg: &FeesInstallmentAssign) -> Result<()>;

    /// Update an existing fees-installment assignment.
    async fn update(&self, ctx: &TenantContext, agg: &FeesInstallmentAssign) -> Result<()>;
}

// =============================================================================
// 13: FeesInstallmentCredit
// =============================================================================

#[async_trait]
pub trait FeesInstallmentCreditRepository: Send + Sync {
    /// Look up a fees-installment credit by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: FeesInstallmentCreditId,
    ) -> Result<Option<FeesInstallmentCredit>>;

    /// List all fees-installment credits in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<FeesInstallmentCredit>>;

    /// List all fees-installment credits for a fees installment plan.
    async fn list_for_installment(
        &self,
        installment_id: FeesInstallmentId,
    ) -> Result<Vec<FeesInstallmentCredit>>;

    /// List all fees-installment credits granted to a student.
    async fn list_for_student(&self, student_id: StudentId) -> Result<Vec<FeesInstallmentCredit>>;

    /// Insert a new fees-installment credit.
    async fn insert(&self, ctx: &TenantContext, agg: &FeesInstallmentCredit) -> Result<()>;

    /// Update an existing fees-installment credit.
    async fn update(&self, ctx: &TenantContext, agg: &FeesInstallmentCredit) -> Result<()>;
}

// =============================================================================
// 14: DirectFeesInstallment
// =============================================================================

#[async_trait]
pub trait DirectFeesInstallmentRepository: Send + Sync {
    /// Look up a direct-fees installment plan by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: DirectFeesInstallmentId,
    ) -> Result<Option<DirectFeesInstallment>>;

    /// List all direct-fees installment plans in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<DirectFeesInstallment>>;

    /// Find a direct-fees installment plan by its (school-scoped) name.
    async fn find_by_name(
        &self,
        school: SchoolId,
        name: &str,
    ) -> Result<Option<DirectFeesInstallment>>;

    /// Insert a new direct-fees installment plan.
    async fn insert(&self, ctx: &TenantContext, agg: &DirectFeesInstallment) -> Result<()>;

    /// Update an existing direct-fees installment plan.
    async fn update(&self, ctx: &TenantContext, agg: &DirectFeesInstallment) -> Result<()>;
}

// =============================================================================
// 15: DirectFeesInstallmentAssign
// =============================================================================

#[async_trait]
pub trait DirectFeesInstallmentAssignRepository: Send + Sync {
    /// Look up a direct-fees installment assignment by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: DirectFeesInstallmentAssignId,
    ) -> Result<Option<DirectFeesInstallmentAssign>>;

    /// List all direct-fees installment assignments in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<DirectFeesInstallmentAssign>>;

    /// List all direct-fees installment assignments for a student.
    async fn list_for_student(
        &self,
        student_id: StudentId,
    ) -> Result<Vec<DirectFeesInstallmentAssign>>;

    /// List all direct-fees installment assignments for a plan.
    async fn list_for_installment(
        &self,
        installment_id: DirectFeesInstallmentId,
    ) -> Result<Vec<DirectFeesInstallmentAssign>>;

    /// Insert a new direct-fees installment assignment.
    async fn insert(&self, ctx: &TenantContext, agg: &DirectFeesInstallmentAssign) -> Result<()>;

    /// Update an existing direct-fees installment assignment.
    async fn update(&self, ctx: &TenantContext, agg: &DirectFeesInstallmentAssign) -> Result<()>;
}

// =============================================================================
// 16: DirectFeesInstallmentChildPayment
// =============================================================================

#[async_trait]
pub trait DirectFeesInstallmentChildPaymentRepository: Send + Sync {
    /// Look up a direct-fees installment child payment by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: DirectFeesInstallmentChildPaymentId,
    ) -> Result<Option<DirectFeesInstallmentChildPayment>>;

    /// List all direct-fees installment child payments in a school.
    async fn list_for_school(
        &self,
        school: SchoolId,
    ) -> Result<Vec<DirectFeesInstallmentChildPayment>>;

    /// List all child payments under a direct-fees installment assignment.
    async fn list_for_assign(
        &self,
        assign_id: DirectFeesInstallmentAssignId,
    ) -> Result<Vec<DirectFeesInstallmentChildPayment>>;

    /// List all direct-fees installment child payments for a student.
    async fn list_for_student(
        &self,
        student_id: StudentId,
    ) -> Result<Vec<DirectFeesInstallmentChildPayment>>;

    /// Insert a new direct-fees installment child payment.
    async fn insert(
        &self,
        ctx: &TenantContext,
        agg: &DirectFeesInstallmentChildPayment,
    ) -> Result<()>;

    /// Update an existing direct-fees installment child payment.
    async fn update(
        &self,
        ctx: &TenantContext,
        agg: &DirectFeesInstallmentChildPayment,
    ) -> Result<()>;
}

// =============================================================================
// 17: DirectFeesSetting
// =============================================================================

#[async_trait]
pub trait DirectFeesSettingRepository: Send + Sync {
    /// Look up a direct-fees setting by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: DirectFeesSettingId,
    ) -> Result<Option<DirectFeesSetting>>;

    /// List all direct-fees settings in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<DirectFeesSetting>>;

    /// Get the (singleton) direct-fees setting for a school.
    async fn get_for_school(&self, school: SchoolId) -> Result<Option<DirectFeesSetting>>;

    /// Insert a new direct-fees setting.
    async fn insert(&self, ctx: &TenantContext, agg: &DirectFeesSetting) -> Result<()>;

    /// Update an existing direct-fees setting.
    async fn update(&self, ctx: &TenantContext, agg: &DirectFeesSetting) -> Result<()>;
}

// =============================================================================
// 18: DirectFeesReminder
// =============================================================================

#[async_trait]
pub trait DirectFeesReminderRepository: Send + Sync {
    /// Look up a direct-fees reminder by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: DirectFeesReminderId,
    ) -> Result<Option<DirectFeesReminder>>;

    /// List all direct-fees reminders in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<DirectFeesReminder>>;

    /// List all direct-fees reminders for a plan.
    async fn list_for_installment(
        &self,
        installment_id: DirectFeesInstallmentId,
    ) -> Result<Vec<DirectFeesReminder>>;

    /// List all pending direct-fees reminders in a school.
    async fn list_pending_for_school(&self, school: SchoolId) -> Result<Vec<DirectFeesReminder>>;

    /// Insert a new direct-fees reminder.
    async fn insert(&self, ctx: &TenantContext, agg: &DirectFeesReminder) -> Result<()>;

    /// Update an existing direct-fees reminder.
    async fn update(&self, ctx: &TenantContext, agg: &DirectFeesReminder) -> Result<()>;
}

// =============================================================================
// 19: FmFeesGroup
// =============================================================================

#[async_trait]
pub trait FmFeesGroupRepository: Send + Sync {
    /// Look up an FM fees group by id.
    async fn get(&self, ctx: &TenantContext, id: FmFeesGroupId) -> Result<Option<FmFeesGroup>>;

    /// List all FM fees groups in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<FmFeesGroup>>;

    /// Find an FM fees group by its (school-scoped) unique name.
    async fn find_by_name(&self, school: SchoolId, name: &str) -> Result<Option<FmFeesGroup>>;

    /// Insert a new FM fees group.
    async fn insert(&self, ctx: &TenantContext, agg: &FmFeesGroup) -> Result<()>;

    /// Update an existing FM fees group.
    async fn update(&self, ctx: &TenantContext, agg: &FmFeesGroup) -> Result<()>;
}

// =============================================================================
// 20: FmFeesType
// =============================================================================

#[async_trait]
pub trait FmFeesTypeRepository: Send + Sync {
    /// Look up an FM fees type by id.
    async fn get(&self, ctx: &TenantContext, id: FmFeesTypeId) -> Result<Option<FmFeesType>>;

    /// List all FM fees types in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<FmFeesType>>;

    /// List all FM fees types that belong to an FM fees group.
    async fn list_for_group(&self, group_id: FmFeesGroupId) -> Result<Vec<FmFeesType>>;

    /// Insert a new FM fees type.
    async fn insert(&self, ctx: &TenantContext, agg: &FmFeesType) -> Result<()>;

    /// Update an existing FM fees type.
    async fn update(&self, ctx: &TenantContext, agg: &FmFeesType) -> Result<()>;
}

// =============================================================================
// 21: FmFeesInvoice
// =============================================================================

#[async_trait]
pub trait FmFeesInvoiceRepository: Send + Sync {
    /// Look up an FM fees invoice by id.
    async fn get(&self, ctx: &TenantContext, id: FmFeesInvoiceId) -> Result<Option<FmFeesInvoice>>;

    /// List all FM fees invoices in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<FmFeesInvoice>>;

    /// List all FM fees invoices for a student.
    async fn list_for_student(&self, student_id: StudentId) -> Result<Vec<FmFeesInvoice>>;

    /// List all FM fees invoices for an academic year.
    async fn list_for_year(&self, academic_year_id: AcademicYearId) -> Result<Vec<FmFeesInvoice>>;

    /// Insert a new FM fees invoice.
    async fn insert(&self, ctx: &TenantContext, agg: &FmFeesInvoice) -> Result<()>;

    /// Update an existing FM fees invoice.
    async fn update(&self, ctx: &TenantContext, agg: &FmFeesInvoice) -> Result<()>;
}

// =============================================================================
// 22: FmFeesInvoiceChild
// =============================================================================

#[async_trait]
pub trait FmFeesInvoiceChildRepository: Send + Sync {
    /// Look up an FM fees invoice child line by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: FmFeesInvoiceChildId,
    ) -> Result<Option<FmFeesInvoiceChild>>;

    /// List all FM fees invoice child lines in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<FmFeesInvoiceChild>>;

    /// List all FM fees invoice child lines under a parent invoice.
    async fn list_for_invoice(
        &self,
        invoice_id: FmFeesInvoiceId,
    ) -> Result<Vec<FmFeesInvoiceChild>>;

    /// Insert a new FM fees invoice child line.
    async fn insert(&self, ctx: &TenantContext, agg: &FmFeesInvoiceChild) -> Result<()>;

    /// Update an existing FM fees invoice child line.
    async fn update(&self, ctx: &TenantContext, agg: &FmFeesInvoiceChild) -> Result<()>;
}

// =============================================================================
// 23: FmFeesInvoiceSetting
// =============================================================================

#[async_trait]
pub trait FmFeesInvoiceSettingRepository: Send + Sync {
    /// Look up an FM fees invoice setting by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: FmFeesInvoiceSettingId,
    ) -> Result<Option<FmFeesInvoiceSetting>>;

    /// List all FM fees invoice settings in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<FmFeesInvoiceSetting>>;

    /// Get the (singleton) FM fees invoice setting for a school.
    async fn get_for_school(&self, school: SchoolId) -> Result<Option<FmFeesInvoiceSetting>>;

    /// Insert a new FM fees invoice setting.
    async fn insert(&self, ctx: &TenantContext, agg: &FmFeesInvoiceSetting) -> Result<()>;

    /// Update an existing FM fees invoice setting.
    async fn update(&self, ctx: &TenantContext, agg: &FmFeesInvoiceSetting) -> Result<()>;
}

// =============================================================================
// 24: FmFeesTransaction
// =============================================================================

#[async_trait]
pub trait FmFeesTransactionRepository: Send + Sync {
    /// Look up an FM fees transaction by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: FmFeesTransactionId,
    ) -> Result<Option<FmFeesTransaction>>;

    /// List all FM fees transactions in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<FmFeesTransaction>>;

    /// List all FM fees transactions for a student.
    async fn list_for_student(&self, student_id: StudentId) -> Result<Vec<FmFeesTransaction>>;

    /// List all FM fees transactions for an FM fees invoice.
    async fn list_for_invoice(&self, invoice_id: FmFeesInvoiceId)
        -> Result<Vec<FmFeesTransaction>>;

    /// Insert a new FM fees transaction.
    async fn insert(&self, ctx: &TenantContext, agg: &FmFeesTransaction) -> Result<()>;

    /// Update an existing FM fees transaction.
    async fn update(&self, ctx: &TenantContext, agg: &FmFeesTransaction) -> Result<()>;
}

// =============================================================================
// 25: FmFeesTransactionChild
// =============================================================================

#[async_trait]
pub trait FmFeesTransactionChildRepository: Send + Sync {
    /// Look up an FM fees transaction child by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: FmFeesTransactionChildId,
    ) -> Result<Option<FmFeesTransactionChild>>;

    /// List all FM fees transaction children in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<FmFeesTransactionChild>>;

    /// List all FM fees transaction children under a parent transaction.
    async fn list_for_transaction(
        &self,
        transaction_id: FmFeesTransactionId,
    ) -> Result<Vec<FmFeesTransactionChild>>;

    /// Insert a new FM fees transaction child.
    async fn insert(&self, ctx: &TenantContext, agg: &FmFeesTransactionChild) -> Result<()>;

    /// Update an existing FM fees transaction child.
    async fn update(&self, ctx: &TenantContext, agg: &FmFeesTransactionChild) -> Result<()>;
}

// =============================================================================
// 26: FmFeesWeaver
// =============================================================================

#[async_trait]
pub trait FmFeesWeaverRepository: Send + Sync {
    /// Look up an FM fees weaver by id.
    async fn get(&self, ctx: &TenantContext, id: FmFeesWeaverId) -> Result<Option<FmFeesWeaver>>;

    /// List all FM fees weavers in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<FmFeesWeaver>>;

    /// List all FM fees weavers attached to an FM fees invoice.
    async fn list_for_invoice(&self, invoice_id: FmFeesInvoiceId) -> Result<Vec<FmFeesWeaver>>;

    /// List all FM fees weavers attached to a class.
    async fn list_for_class(&self, class_id: ClassId) -> Result<Vec<FmFeesWeaver>>;

    /// Insert a new FM fees weaver.
    async fn insert(&self, ctx: &TenantContext, agg: &FmFeesWeaver) -> Result<()>;

    /// Update an existing FM fees weaver.
    async fn update(&self, ctx: &TenantContext, agg: &FmFeesWeaver) -> Result<()>;
}

// =============================================================================
// 27: BankAccount
// =============================================================================

#[async_trait]
pub trait BankAccountRepository: Send + Sync {
    /// Look up a bank account by id.
    async fn get(&self, ctx: &TenantContext, id: BankAccountId) -> Result<Option<BankAccount>>;

    /// List all bank accounts in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<BankAccount>>;

    /// Find a bank account by its (school-scoped) account number.
    async fn find_by_account_number(
        &self,
        school: SchoolId,
        account_number: &str,
    ) -> Result<Option<BankAccount>>;

    /// List all active bank accounts in a school.
    async fn list_active(&self, school: SchoolId) -> Result<Vec<BankAccount>>;

    /// Insert a new bank account.
    async fn insert(&self, ctx: &TenantContext, agg: &BankAccount) -> Result<()>;

    /// Update an existing bank account.
    async fn update(&self, ctx: &TenantContext, agg: &BankAccount) -> Result<()>;
}

// =============================================================================
// 28: BankStatement
// =============================================================================

#[async_trait]
pub trait BankStatementRepository: Send + Sync {
    /// Look up a bank statement by id.
    async fn get(&self, ctx: &TenantContext, id: BankStatementId) -> Result<Option<BankStatement>>;

    /// List all bank statements in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<BankStatement>>;

    /// List all bank statements for a bank account.
    async fn list_for_account(&self, account_id: BankAccountId) -> Result<Vec<BankStatement>>;

    /// List all pending (unreconciled) bank statements for an account.
    async fn list_pending_for_account(
        &self,
        account_id: BankAccountId,
    ) -> Result<Vec<BankStatement>>;

    /// Insert a new bank statement.
    async fn insert(&self, ctx: &TenantContext, agg: &BankStatement) -> Result<()>;

    /// Update an existing bank statement.
    async fn update(&self, ctx: &TenantContext, agg: &BankStatement) -> Result<()>;
}

// =============================================================================
// 29: BankPaymentSlip
// =============================================================================

#[async_trait]
pub trait BankPaymentSlipRepository: Send + Sync {
    /// Look up a bank payment slip by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: BankPaymentSlipId,
    ) -> Result<Option<BankPaymentSlip>>;

    /// List all bank payment slips in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<BankPaymentSlip>>;

    /// List all pending bank payment slips in a school.
    async fn list_pending_for_school(&self, school: SchoolId) -> Result<Vec<BankPaymentSlip>>;

    /// Find a bank payment slip by its (school-scoped) reference.
    async fn find_by_reference(
        &self,
        school: SchoolId,
        reference: &str,
    ) -> Result<Option<BankPaymentSlip>>;

    /// Insert a new bank payment slip.
    async fn insert(&self, ctx: &TenantContext, agg: &BankPaymentSlip) -> Result<()>;

    /// Update an existing bank payment slip.
    async fn update(&self, ctx: &TenantContext, agg: &BankPaymentSlip) -> Result<()>;
}

// =============================================================================
// 30: Expense
// =============================================================================

#[async_trait]
pub trait ExpenseRepository: Send + Sync {
    /// Look up an expense by id.
    async fn get(&self, ctx: &TenantContext, id: ExpenseId) -> Result<Option<Expense>>;

    /// List all expenses in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<Expense>>;

    /// List all expenses under an expense head.
    async fn list_for_head(&self, head_id: ExpenseHeadId) -> Result<Vec<Expense>>;

    /// List all expenses paid from a bank account.
    async fn list_for_account(&self, account_id: BankAccountId) -> Result<Vec<Expense>>;

    /// Insert a new expense.
    async fn insert(&self, ctx: &TenantContext, agg: &Expense) -> Result<()>;

    /// Update an existing expense.
    async fn update(&self, ctx: &TenantContext, agg: &Expense) -> Result<()>;
}

// =============================================================================
// 31: ExpenseHead
// =============================================================================

#[async_trait]
pub trait ExpenseHeadRepository: Send + Sync {
    /// Look up an expense head by id.
    async fn get(&self, ctx: &TenantContext, id: ExpenseHeadId) -> Result<Option<ExpenseHead>>;

    /// List all expense heads in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<ExpenseHead>>;

    /// Find an expense head by its (school-scoped) unique name.
    async fn find_by_name(&self, school: SchoolId, name: &str) -> Result<Option<ExpenseHead>>;

    /// Insert a new expense head.
    async fn insert(&self, ctx: &TenantContext, agg: &ExpenseHead) -> Result<()>;

    /// Update an existing expense head.
    async fn update(&self, ctx: &TenantContext, agg: &ExpenseHead) -> Result<()>;
}

// =============================================================================
// 32: Income
// =============================================================================

#[async_trait]
pub trait IncomeRepository: Send + Sync {
    /// Look up an income by id.
    async fn get(&self, ctx: &TenantContext, id: IncomeId) -> Result<Option<Income>>;

    /// List all incomes in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<Income>>;

    /// List all incomes under an income head.
    async fn list_for_head(&self, head_id: IncomeHeadId) -> Result<Vec<Income>>;

    /// List all incomes received into a bank account.
    async fn list_for_account(&self, account_id: BankAccountId) -> Result<Vec<Income>>;

    /// Insert a new income.
    async fn insert(&self, ctx: &TenantContext, agg: &Income) -> Result<()>;

    /// Update an existing income.
    async fn update(&self, ctx: &TenantContext, agg: &Income) -> Result<()>;
}

// =============================================================================
// 33: IncomeHead
// =============================================================================

#[async_trait]
pub trait IncomeHeadRepository: Send + Sync {
    /// Look up an income head by id.
    async fn get(&self, ctx: &TenantContext, id: IncomeHeadId) -> Result<Option<IncomeHead>>;

    /// List all income heads in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<IncomeHead>>;

    /// Find an income head by its (school-scoped) unique name.
    async fn find_by_name(&self, school: SchoolId, name: &str) -> Result<Option<IncomeHead>>;

    /// Insert a new income head.
    async fn insert(&self, ctx: &TenantContext, agg: &IncomeHead) -> Result<()>;

    /// Update an existing income head.
    async fn update(&self, ctx: &TenantContext, agg: &IncomeHead) -> Result<()>;
}

// =============================================================================
// 34: Donor
// =============================================================================

#[async_trait]
pub trait DonorRepository: Send + Sync {
    /// Look up a donor by id.
    async fn get(&self, ctx: &TenantContext, id: DonorId) -> Result<Option<Donor>>;

    /// List all donors in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<Donor>>;

    /// Find a donor by its (school-scoped) unique name.
    async fn find_by_name(&self, school: SchoolId, name: &str) -> Result<Option<Donor>>;

    /// Insert a new donor.
    async fn insert(&self, ctx: &TenantContext, agg: &Donor) -> Result<()>;

    /// Update an existing donor.
    async fn update(&self, ctx: &TenantContext, agg: &Donor) -> Result<()>;
}

// =============================================================================
// 35: ChartOfAccount
// =============================================================================

#[async_trait]
pub trait ChartOfAccountRepository: Send + Sync {
    /// Look up a chart-of-account entry by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: ChartOfAccountId,
    ) -> Result<Option<ChartOfAccount>>;

    /// List all chart-of-account entries in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<ChartOfAccount>>;

    /// Find a chart-of-account entry by its (school-scoped) code.
    async fn find_by_code(&self, school: SchoolId, code: &str) -> Result<Option<ChartOfAccount>>;

    /// List all active chart-of-account entries in a school.
    async fn list_active(&self, school: SchoolId) -> Result<Vec<ChartOfAccount>>;

    /// Insert a new chart-of-account entry.
    async fn insert(&self, ctx: &TenantContext, agg: &ChartOfAccount) -> Result<()>;

    /// Update an existing chart-of-account entry.
    async fn update(&self, ctx: &TenantContext, agg: &ChartOfAccount) -> Result<()>;
}

// =============================================================================
// 36: PayrollPayment
// =============================================================================

#[async_trait]
pub trait PayrollPaymentRepository: Send + Sync {
    /// Look up a payroll payment by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: PayrollPaymentId,
    ) -> Result<Option<PayrollPayment>>;

    /// List all payroll payments in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<PayrollPayment>>;

    /// List all payroll payments for a staff member.
    async fn list_for_staff(&self, staff_id: StaffId) -> Result<Vec<PayrollPayment>>;

    /// List all payroll payments for an academic year.
    async fn list_for_year(&self, academic_year_id: AcademicYearId) -> Result<Vec<PayrollPayment>>;

    /// Insert a new payroll payment.
    async fn insert(&self, ctx: &TenantContext, agg: &PayrollPayment) -> Result<()>;

    /// Update an existing payroll payment.
    async fn update(&self, ctx: &TenantContext, agg: &PayrollPayment) -> Result<()>;
}

// =============================================================================
// 37: ProductPurchase
// =============================================================================

#[async_trait]
pub trait ProductPurchaseRepository: Send + Sync {
    /// Look up a product purchase by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: ProductPurchaseId,
    ) -> Result<Option<ProductPurchase>>;

    /// List all product purchases in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<ProductPurchase>>;

    /// List all product purchases from a given supplier.
    async fn list_for_supplier(
        &self,
        school: SchoolId,
        supplier: &str,
    ) -> Result<Vec<ProductPurchase>>;

    /// Insert a new product purchase.
    async fn insert(&self, ctx: &TenantContext, agg: &ProductPurchase) -> Result<()>;

    /// Update an existing product purchase.
    async fn update(&self, ctx: &TenantContext, agg: &ProductPurchase) -> Result<()>;
}

// =============================================================================
// 38: InventoryPayment
// =============================================================================

#[async_trait]
pub trait InventoryPaymentRepository: Send + Sync {
    /// Look up an inventory payment by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: InventoryPaymentId,
    ) -> Result<Option<InventoryPayment>>;

    /// List all inventory payments in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<InventoryPayment>>;

    /// List all inventory payments for a product purchase.
    async fn list_for_purchase(
        &self,
        purchase_id: ProductPurchaseId,
    ) -> Result<Vec<InventoryPayment>>;

    /// Insert a new inventory payment.
    async fn insert(&self, ctx: &TenantContext, agg: &InventoryPayment) -> Result<()>;

    /// Update an existing inventory payment.
    async fn update(&self, ctx: &TenantContext, agg: &InventoryPayment) -> Result<()>;
}

// =============================================================================
// 39: AmountTransfer
// =============================================================================

#[async_trait]
pub trait AmountTransferRepository: Send + Sync {
    /// Look up an amount transfer by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: AmountTransferId,
    ) -> Result<Option<AmountTransfer>>;

    /// List all amount transfers in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<AmountTransfer>>;

    /// List all amount transfers involving a bank account (in or out).
    async fn list_for_account(&self, account_id: BankAccountId) -> Result<Vec<AmountTransfer>>;

    /// Insert a new amount transfer.
    async fn insert(&self, ctx: &TenantContext, agg: &AmountTransfer) -> Result<()>;

    /// Update an existing amount transfer.
    async fn update(&self, ctx: &TenantContext, agg: &AmountTransfer) -> Result<()>;
}

// =============================================================================
// 40: QuestionBankFee
// =============================================================================

#[async_trait]
pub trait QuestionBankFeeRepository: Send + Sync {
    /// Look up a question-bank fee by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: QuestionBankFeeId,
    ) -> Result<Option<QuestionBankFee>>;

    /// List all question-bank fees in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<QuestionBankFee>>;

    /// List all question-bank fees for a class.
    async fn list_for_class(&self, class_id: ClassId) -> Result<Vec<QuestionBankFee>>;

    /// List all question-bank fees for a subject.
    async fn list_for_subject(&self, subject_id: SubjectId) -> Result<Vec<QuestionBankFee>>;

    /// Insert a new question-bank fee.
    async fn insert(&self, ctx: &TenantContext, agg: &QuestionBankFee) -> Result<()>;

    /// Update an existing question-bank fee.
    async fn update(&self, ctx: &TenantContext, agg: &QuestionBankFee) -> Result<()>;
}

// =============================================================================
// 41: PaymentGatewaySetting
// =============================================================================

#[async_trait]
pub trait PaymentGatewaySettingRepository: Send + Sync {
    /// Look up a payment-gateway setting by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: PaymentGatewaySettingId,
    ) -> Result<Option<PaymentGatewaySetting>>;

    /// List all payment-gateway settings in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<PaymentGatewaySetting>>;

    /// Find a payment-gateway setting by its (school-scoped) mode.
    async fn find_by_mode(
        &self,
        school: SchoolId,
        mode: GatewayMode,
    ) -> Result<Option<PaymentGatewaySetting>>;

    /// Insert a new payment-gateway setting.
    async fn insert(&self, ctx: &TenantContext, agg: &PaymentGatewaySetting) -> Result<()>;

    /// Update an existing payment-gateway setting.
    async fn update(&self, ctx: &TenantContext, agg: &PaymentGatewaySetting) -> Result<()>;
}

// =============================================================================
// 42: PaymentMethod
// =============================================================================

#[async_trait]
pub trait PaymentMethodRepository: Send + Sync {
    /// Look up a payment method by id.
    async fn get(&self, ctx: &TenantContext, id: PaymentMethodId) -> Result<Option<PaymentMethod>>;

    /// List all payment methods in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<PaymentMethod>>;

    /// Find a payment method by its (school-scoped) kind.
    async fn find_by_kind(
        &self,
        school: SchoolId,
        kind: PaymentMethodKind,
    ) -> Result<Option<PaymentMethod>>;

    /// List all active payment methods in a school.
    async fn list_active(&self, school: SchoolId) -> Result<Vec<PaymentMethod>>;

    /// Insert a new payment method.
    async fn insert(&self, ctx: &TenantContext, agg: &PaymentMethod) -> Result<()>;

    /// Update an existing payment method.
    async fn update(&self, ctx: &TenantContext, agg: &PaymentMethod) -> Result<()>;
}

// =============================================================================
// 43: DueFeesLoginPrevent
// =============================================================================

#[async_trait]
pub trait DueFeesLoginPreventRepository: Send + Sync {
    /// Look up a due-fees login-prevent record by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: DueFeesLoginPreventId,
    ) -> Result<Option<DueFeesLoginPrevent>>;

    /// List all due-fees login-prevent records in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<DueFeesLoginPrevent>>;

    /// List all due-fees login-prevent records for a student.
    async fn list_for_student(&self, student_id: StudentId) -> Result<Vec<DueFeesLoginPrevent>>;

    /// List all active due-fees login-prevent records in a school.
    async fn list_active_for_school(&self, school: SchoolId) -> Result<Vec<DueFeesLoginPrevent>>;

    /// Insert a new due-fees login-prevent record.
    async fn insert(&self, ctx: &TenantContext, agg: &DueFeesLoginPrevent) -> Result<()>;

    /// Update an existing due-fees login-prevent record.
    async fn update(&self, ctx: &TenantContext, agg: &DueFeesLoginPrevent) -> Result<()>;
}

// =============================================================================
// 44: FeesCarryForward
// =============================================================================

#[async_trait]
pub trait FeesCarryForwardRepository: Send + Sync {
    /// Look up a fees carry-forward record by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: FeesCarryForwardId,
    ) -> Result<Option<FeesCarryForward>>;

    /// List all fees carry-forward records in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<FeesCarryForward>>;

    /// List all fees carry-forward records for a student.
    async fn list_for_student(&self, student_id: StudentId) -> Result<Vec<FeesCarryForward>>;

    /// List all fees carry-forward records for an academic year.
    async fn list_for_year(
        &self,
        academic_year_id: AcademicYearId,
    ) -> Result<Vec<FeesCarryForward>>;

    /// Insert a new fees carry-forward record.
    async fn insert(&self, ctx: &TenantContext, agg: &FeesCarryForward) -> Result<()>;

    /// Update an existing fees carry-forward record.
    async fn update(&self, ctx: &TenantContext, agg: &FeesCarryForward) -> Result<()>;
}

// =============================================================================
// 45: BankPaymentSlipAudit
// =============================================================================

#[async_trait]
pub trait BankPaymentSlipAuditRepository: Send + Sync {
    /// Look up a bank-payment-slip audit row by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: BankPaymentSlipAuditId,
    ) -> Result<Option<BankPaymentSlipAudit>>;

    /// List all bank-payment-slip audit rows in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<BankPaymentSlipAudit>>;

    /// Insert a new bank-payment-slip audit row.
    async fn insert(&self, ctx: &TenantContext, agg: &BankPaymentSlipAudit) -> Result<()>;

    /// Update an existing bank-payment-slip audit row.
    async fn update(&self, ctx: &TenantContext, agg: &BankPaymentSlipAudit) -> Result<()>;
}

// =============================================================================
// 46: BankStatementAttachment
// =============================================================================

#[async_trait]
pub trait BankStatementAttachmentRepository: Send + Sync {
    /// Look up a bank-statement attachment by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: BankStatementAttachmentId,
    ) -> Result<Option<BankStatementAttachment>>;

    /// List all bank-statement attachments in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<BankStatementAttachment>>;

    /// Insert a new bank-statement attachment.
    async fn insert(&self, ctx: &TenantContext, agg: &BankStatementAttachment) -> Result<()>;

    /// Update an existing bank-statement attachment.
    async fn update(&self, ctx: &TenantContext, agg: &BankStatementAttachment) -> Result<()>;
}

// =============================================================================
// 47: DirectFeesInstallmentAssignChild
// =============================================================================

#[async_trait]
pub trait DirectFeesInstallmentAssignChildRepository: Send + Sync {
    /// Look up a direct-fees installment assignment child by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: DirectFeesInstallmentAssignChildId,
    ) -> Result<Option<DirectFeesInstallmentAssignChild>>;

    /// List all direct-fees installment assignment children in a school.
    async fn list_for_school(
        &self,
        school: SchoolId,
    ) -> Result<Vec<DirectFeesInstallmentAssignChild>>;

    /// Insert a new direct-fees installment assignment child.
    async fn insert(
        &self,
        ctx: &TenantContext,
        agg: &DirectFeesInstallmentAssignChild,
    ) -> Result<()>;

    /// Update an existing direct-fees installment assignment child.
    async fn update(
        &self,
        ctx: &TenantContext,
        agg: &DirectFeesInstallmentAssignChild,
    ) -> Result<()>;
}

// =============================================================================
// 48: ExpenseApproval
// =============================================================================

#[async_trait]
pub trait ExpenseApprovalRepository: Send + Sync {
    /// Look up an expense-approval row by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: ExpenseApprovalId,
    ) -> Result<Option<ExpenseApproval>>;

    /// List all expense-approval rows in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<ExpenseApproval>>;

    /// Insert a new expense-approval row.
    async fn insert(&self, ctx: &TenantContext, agg: &ExpenseApproval) -> Result<()>;

    /// Update an existing expense-approval row.
    async fn update(&self, ctx: &TenantContext, agg: &ExpenseApproval) -> Result<()>;
}

// =============================================================================
// 49: FeesCarryForwardLog
// =============================================================================

#[async_trait]
pub trait FeesCarryForwardLogRepository: Send + Sync {
    /// Look up a fees carry-forward log row by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: FeesCarryForwardLogId,
    ) -> Result<Option<FeesCarryForwardLog>>;

    /// List all fees carry-forward log rows in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<FeesCarryForwardLog>>;

    /// Insert a new fees carry-forward log row.
    async fn insert(&self, ctx: &TenantContext, agg: &FeesCarryForwardLog) -> Result<()>;

    /// Update an existing fees carry-forward log row.
    async fn update(&self, ctx: &TenantContext, agg: &FeesCarryForwardLog) -> Result<()>;
}

// =============================================================================
// 50: FeesCarryForwardSetting
// =============================================================================

#[async_trait]
pub trait FeesCarryForwardSettingRepository: Send + Sync {
    /// Look up a fees carry-forward setting by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: FeesCarryForwardSettingId,
    ) -> Result<Option<FeesCarryForwardSetting>>;

    /// List all fees carry-forward settings in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<FeesCarryForwardSetting>>;

    /// Get the (singleton) fees carry-forward setting for a school.
    async fn get_for_school(&self, school: SchoolId) -> Result<Option<FeesCarryForwardSetting>>;

    /// Insert a new fees carry-forward setting.
    async fn insert(&self, ctx: &TenantContext, agg: &FeesCarryForwardSetting) -> Result<()>;

    /// Update an existing fees carry-forward setting.
    async fn update(&self, ctx: &TenantContext, agg: &FeesCarryForwardSetting) -> Result<()>;
}

// =============================================================================
// 51: FeesInstallmentAssignDiscount
// =============================================================================

#[async_trait]
pub trait FeesInstallmentAssignDiscountRepository: Send + Sync {
    /// Look up a fees-installment-assign discount by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: FeesInstallmentAssignDiscountId,
    ) -> Result<Option<FeesInstallmentAssignDiscount>>;

    /// List all fees-installment-assign discounts in a school.
    async fn list_for_school(&self, school: SchoolId)
        -> Result<Vec<FeesInstallmentAssignDiscount>>;

    /// Insert a new fees-installment-assign discount.
    async fn insert(&self, ctx: &TenantContext, agg: &FeesInstallmentAssignDiscount) -> Result<()>;

    /// Update an existing fees-installment-assign discount.
    async fn update(&self, ctx: &TenantContext, agg: &FeesInstallmentAssignDiscount) -> Result<()>;
}

// =============================================================================
// 52: FeesInvoice
// =============================================================================

#[async_trait]
pub trait FeesInvoiceRepository: Send + Sync {
    /// Look up a fees invoice by id.
    async fn get(&self, ctx: &TenantContext, id: FeesInvoiceId) -> Result<Option<FeesInvoice>>;

    /// List all fees invoices in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<FeesInvoice>>;

    /// List all fees invoices for a student.
    async fn list_for_student(&self, student_id: StudentId) -> Result<Vec<FeesInvoice>>;

    /// Insert a new fees invoice.
    async fn insert(&self, ctx: &TenantContext, agg: &FeesInvoice) -> Result<()>;

    /// Update an existing fees invoice.
    async fn update(&self, ctx: &TenantContext, agg: &FeesInvoice) -> Result<()>;
}

// =============================================================================
// 53: FeesPayment
// =============================================================================

#[async_trait]
pub trait FeesPaymentRepository: Send + Sync {
    /// Look up a fees payment by id.
    async fn get(&self, ctx: &TenantContext, id: FeesPaymentId) -> Result<Option<FeesPayment>>;

    /// List all fees payments in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<FeesPayment>>;

    /// List all fees payments for a student.
    async fn list_for_student(&self, student_id: StudentId) -> Result<Vec<FeesPayment>>;

    /// Insert a new fees payment.
    async fn insert(&self, ctx: &TenantContext, agg: &FeesPayment) -> Result<()>;

    /// Update an existing fees payment.
    async fn update(&self, ctx: &TenantContext, agg: &FeesPayment) -> Result<()>;
}

// =============================================================================
// 54: FmFeesInvoiceLineNote
// =============================================================================

#[async_trait]
pub trait FmFeesInvoiceLineNoteRepository: Send + Sync {
    /// Look up an FM fees invoice line note by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: FmFeesInvoiceLineNoteId,
    ) -> Result<Option<FmFeesInvoiceLineNote>>;

    /// List all FM fees invoice line notes in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<FmFeesInvoiceLineNote>>;

    /// Insert a new FM fees invoice line note.
    async fn insert(&self, ctx: &TenantContext, agg: &FmFeesInvoiceLineNote) -> Result<()>;

    /// Update an existing FM fees invoice line note.
    async fn update(&self, ctx: &TenantContext, agg: &FmFeesInvoiceLineNote) -> Result<()>;
}

// =============================================================================
// 55: FmFeesTransactionLineNote
// =============================================================================

#[async_trait]
pub trait FmFeesTransactionLineNoteRepository: Send + Sync {
    /// Look up an FM fees transaction line note by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: FmFeesTransactionLineNoteId,
    ) -> Result<Option<FmFeesTransactionLineNote>>;

    /// List all FM fees transaction line notes in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<FmFeesTransactionLineNote>>;

    /// Insert a new FM fees transaction line note.
    async fn insert(&self, ctx: &TenantContext, agg: &FmFeesTransactionLineNote) -> Result<()>;

    /// Update an existing FM fees transaction line note.
    async fn update(&self, ctx: &TenantContext, agg: &FmFeesTransactionLineNote) -> Result<()>;
}

// =============================================================================
// 56: IncomeApproval
// =============================================================================

#[async_trait]
pub trait IncomeApprovalRepository: Send + Sync {
    /// Look up an income-approval row by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: IncomeApprovalId,
    ) -> Result<Option<IncomeApproval>>;

    /// List all income-approval rows in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<IncomeApproval>>;

    /// Insert a new income-approval row.
    async fn insert(&self, ctx: &TenantContext, agg: &IncomeApproval) -> Result<()>;

    /// Update an existing income-approval row.
    async fn update(&self, ctx: &TenantContext, agg: &IncomeApproval) -> Result<()>;
}

// =============================================================================
// 57: PayrollEarnDeduc
// =============================================================================

#[async_trait]
pub trait PayrollEarnDeducRepository: Send + Sync {
    /// Look up a payroll earnings/deduction line by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: PayrollEarnDeducId,
    ) -> Result<Option<PayrollEarnDeduc>>;

    /// List all payroll earnings/deduction lines in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<PayrollEarnDeduc>>;

    /// List all payroll earnings/deduction lines for a payroll run.
    async fn list_for_payroll(
        &self,
        payroll_id: PayrollGenerateId,
    ) -> Result<Vec<PayrollEarnDeduc>>;

    /// Insert a new payroll earnings/deduction line.
    async fn insert(&self, ctx: &TenantContext, agg: &PayrollEarnDeduc) -> Result<()>;

    /// Update an existing payroll earnings/deduction line.
    async fn update(&self, ctx: &TenantContext, agg: &PayrollEarnDeduc) -> Result<()>;
}

// =============================================================================
// 58: PayrollGenerate
// =============================================================================

#[async_trait]
pub trait PayrollGenerateRepository: Send + Sync {
    /// Look up a payroll generate (run) by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: PayrollGenerateId,
    ) -> Result<Option<PayrollGenerate>>;

    /// List all payroll generate runs in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<PayrollGenerate>>;

    /// Insert a new payroll generate run.
    async fn insert(&self, ctx: &TenantContext, agg: &PayrollGenerate) -> Result<()>;

    /// Update an existing payroll generate run.
    async fn update(&self, ctx: &TenantContext, agg: &PayrollGenerate) -> Result<()>;
}

// =============================================================================
// 59: PayrollPaymentApproval
// =============================================================================

#[async_trait]
pub trait PayrollPaymentApprovalRepository: Send + Sync {
    /// Look up a payroll-payment approval row by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: PayrollPaymentApprovalId,
    ) -> Result<Option<PayrollPaymentApproval>>;

    /// List all payroll-payment approval rows in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<PayrollPaymentApproval>>;

    /// Insert a new payroll-payment approval row.
    async fn insert(&self, ctx: &TenantContext, agg: &PayrollPaymentApproval) -> Result<()>;

    /// Update an existing payroll-payment approval row.
    async fn update(&self, ctx: &TenantContext, agg: &PayrollPaymentApproval) -> Result<()>;
}

// =============================================================================
// 60: SalaryTemplate
// =============================================================================

#[async_trait]
pub trait SalaryTemplateRepository: Send + Sync {
    /// Look up a salary template by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: SalaryTemplateId,
    ) -> Result<Option<SalaryTemplate>>;

    /// List all salary templates in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<SalaryTemplate>>;

    /// List all salary templates for a staff member.
    async fn list_for_staff(&self, staff_id: StaffId) -> Result<Vec<SalaryTemplate>>;

    /// Insert a new salary template.
    async fn insert(&self, ctx: &TenantContext, agg: &SalaryTemplate) -> Result<()>;

    /// Update an existing salary template.
    async fn update(&self, ctx: &TenantContext, agg: &SalaryTemplate) -> Result<()>;
}

// =============================================================================
// 61: Transaction (double-entry journal line)
// =============================================================================

#[async_trait]
pub trait TransactionRepository: Send + Sync {
    /// Look up a journal transaction by id.
    async fn get(&self, ctx: &TenantContext, id: TransactionId) -> Result<Option<Transaction>>;

    /// List all journal transactions in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<Transaction>>;

    /// Insert a new journal transaction.
    async fn insert(&self, ctx: &TenantContext, agg: &Transaction) -> Result<()>;

    /// Update an existing journal transaction.
    async fn update(&self, ctx: &TenantContext, agg: &Transaction) -> Result<()>;
}

// =============================================================================
// 62: WalletTransactionApproval
// =============================================================================

#[async_trait]
pub trait WalletTransactionApprovalRepository: Send + Sync {
    /// Look up a wallet-transaction approval row by id.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: WalletTransactionApprovalId,
    ) -> Result<Option<WalletTransactionApproval>>;

    /// List all wallet-transaction approval rows in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<WalletTransactionApproval>>;

    /// List all wallet-transaction approval rows for a wallet transaction.
    async fn list_for_wallet_transaction(
        &self,
        wallet_transaction_id: WalletTransactionId,
    ) -> Result<Vec<WalletTransactionApproval>>;

    /// Insert a new wallet-transaction approval row.
    async fn insert(&self, ctx: &TenantContext, agg: &WalletTransactionApproval) -> Result<()>;

    /// Update an existing wallet-transaction approval row.
    async fn update(&self, ctx: &TenantContext, agg: &WalletTransactionApproval) -> Result<()>;
}
