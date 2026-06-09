# Finance Domain — Repositories

Repositories are ports (Rust traits). Adapters implement them. The
default adapter targets PostgreSQL; an SQLite adapter is provided for
embedded deployments.

All repositories include a `school_id` parameter on every read. The
storage adapter is responsible for tenant isolation.

## FeesGroupRepository

```rust
#[async_trait]
pub trait FeesGroupRepository: Send + Sync {
    async fn get(&self, id: FeesGroupId) -> Result<Option<FeesGroup>>;
    async fn list(&self, school: SchoolId, academic: AcademicYearId) -> Result<Vec<FeesGroup>>;
    async fn find_by_name(&self, school: SchoolId, name: &str) -> Result<Option<FeesGroup>>;
    async fn insert(&self, g: &FeesGroup) -> Result<()>;
    async fn update(&self, g: &FeesGroup) -> Result<()>;
    async fn delete(&self, id: FeesGroupId) -> Result<()>;
}
```

## FeesTypeRepository

```rust
#[async_trait]
pub trait FeesTypeRepository: Send + Sync {
    async fn get(&self, id: FeesTypeId) -> Result<Option<FeesType>>;
    async fn list_for_group(&self, group: FeesGroupId) -> Result<Vec<FeesType>>;
    async fn find_by_name(&self, group: FeesGroupId, name: &str) -> Result<Option<FeesType>>;
    async fn insert(&self, t: &FeesType) -> Result<()>;
    async fn update(&self, t: &FeesType) -> Result<()>;
    async fn delete(&self, id: FeesTypeId) -> Result<()>;
}
```

## FeesMasterRepository

```rust
#[async_trait]
pub trait FeesMasterRepository: Send + Sync {
    async fn get(&self, id: FeesMasterId) -> Result<Option<FeesMaster>>;
    async fn list_for_class(&self, school: SchoolId, class_id: ClassId, section: Option<SectionId>, academic: AcademicYearId) -> Result<Vec<FeesMaster>>;
    async fn list_for_group(&self, group: FeesGroupId) -> Result<Vec<FeesMaster>>;
    async fn insert(&self, m: &FeesMaster) -> Result<()>;
    async fn update(&self, m: &FeesMaster) -> Result<()>;
    async fn delete(&self, id: FeesMasterId) -> Result<()>;
}
```

## FeesAssignRepository

```rust
#[async_trait]
pub trait FeesAssignRepository: Send + Sync {
    async fn get(&self, id: FeesAssignId) -> Result<Option<FeesAssign>>;
    async fn list_for_student(&self, student: StudentId, academic: AcademicYearId) -> Result<Vec<FeesAssign>>;
    async fn list_for_class(&self, school: SchoolId, class_id: ClassId, academic: AcademicYearId) -> Result<Vec<FeesAssign>>;
    async fn list_open(&self, school: SchoolId, academic: AcademicYearId) -> Result<Vec<FeesAssign>>;
    async fn outstanding_balance(&self, id: FeesAssignId) -> Result<Amount>;
    async fn insert(&self, a: &FeesAssign) -> Result<()>;
    async fn update(&self, a: &FeesAssign) -> Result<()>;
    async fn close(&self, id: FeesAssignId) -> Result<()>;
}
```

## FeesAssignDiscountRepository

```rust
#[async_trait]
pub trait FeesAssignDiscountRepository: Send + Sync {
    async fn get(&self, id: FeesAssignDiscountId) -> Result<Option<FeesAssignDiscount>>;
    async fn list_for_student(&self, student: StudentId, academic: AcademicYearId) -> Result<Vec<FeesAssignDiscount>>;
    async fn list_for_discount(&self, discount: FeesDiscountId) -> Result<Vec<FeesAssignDiscount>>;
    async fn insert(&self, d: &FeesAssignDiscount) -> Result<()>;
    async fn update(&self, d: &FeesAssignDiscount) -> Result<()>;
    async fn delete(&self, id: FeesAssignDiscountId) -> Result<()>;
}
```

## FeesDiscountRepository

```rust
#[async_trait]
pub trait FeesDiscountRepository: Send + Sync {
    async fn get(&self, id: FeesDiscountId) -> Result<Option<FeesDiscount>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<FeesDiscount>>;
    async fn find_by_code(&self, school: SchoolId, code: &str) -> Result<Option<FeesDiscount>>;
    async fn insert(&self, d: &FeesDiscount) -> Result<()>;
    async fn update(&self, d: &FeesDiscount) -> Result<()>;
    async fn delete(&self, id: FeesDiscountId) -> Result<()>;
}
```

## FeesInvoiceRepository

```rust
#[async_trait]
pub trait FeesInvoiceRepository: Send + Sync {
    async fn get(&self, id: FeesInvoiceId) -> Result<Option<FeesInvoice>>;
    async fn get_for_school(&self, school: SchoolId) -> Result<Option<FeesInvoice>>;
    async fn insert(&self, i: &FeesInvoice) -> Result<()>;
    async fn update(&self, i: &FeesInvoice) -> Result<()>;
}
```

## FeesInstallmentRepository

```rust
#[async_trait]
pub trait FeesInstallmentRepository: Send + Sync {
    async fn get(&self, id: FeesInstallmentId) -> Result<Option<FeesInstallment>>;
    async fn list_for_master(&self, master: FeesMasterId) -> Result<Vec<FeesInstallment>>;
    async fn insert(&self, i: &FeesInstallment) -> Result<()>;
    async fn update(&self, i: &FeesInstallment) -> Result<()>;
    async fn delete(&self, id: FeesInstallmentId) -> Result<()>;
}
```

## FeesInstallmentAssignRepository

```rust
#[async_trait]
pub trait FeesInstallmentAssignRepository: Send + Sync {
    async fn get(&self, id: FeesInstallmentAssignId) -> Result<Option<FeesInstallmentAssign>>;
    async fn list_for_student(&self, student: StudentId, academic: AcademicYearId) -> Result<Vec<FeesInstallmentAssign>>;
    async fn list_for_installment(&self, installment: FeesInstallmentId) -> Result<Vec<FeesInstallmentAssign>>;
    async fn insert(&self, a: &FeesInstallmentAssign) -> Result<()>;
    async fn update(&self, a: &FeesInstallmentAssign) -> Result<()>;
}
```

## FeesPaymentRepository

```rust
#[async_trait]
pub trait FeesPaymentRepository: Send + Sync {
    async fn get(&self, id: FeesPaymentId) -> Result<Option<FeesPayment>>;
    async fn list_for_assign(&self, assign: FeesAssignId) -> Result<Vec<FeesPayment>>;
    async fn list_for_student(&self, student: StudentId, academic: AcademicYearId) -> Result<Vec<FeesPayment>>;
    async fn list_for_date(&self, school: SchoolId, date: NaiveDate) -> Result<Vec<FeesPayment>>;
    async fn list_for_method(&self, school: SchoolId, method: PaymentMethodId) -> Result<Vec<FeesPayment>>;
    async fn insert(&self, p: &FeesPayment) -> Result<()>;
    async fn reverse(&self, id: FeesPaymentId) -> Result<()>;
}
```

## FeesCarryForwardRepository

```rust
#[async_trait]
pub trait FeesCarryForwardRepository: Send + Sync {
    async fn get(&self, id: FeesCarryForwardId) -> Result<Option<FeesCarryForward>>;
    async fn list_for_student(&self, student: StudentId) -> Result<Vec<FeesCarryForward>>;
    async fn list_open(&self, school: SchoolId, academic: AcademicYearId) -> Result<Vec<FeesCarryForward>>;
    async fn insert(&self, c: &FeesCarryForward) -> Result<()>;
    async fn close(&self, id: FeesCarryForwardId) -> Result<()>;
}
```

## FeesCarryForwardLogRepository

```rust
#[async_trait]
pub trait FeesCarryForwardLogRepository: Send + Sync {
    async fn insert(&self, log: &FeesCarryForwardLog) -> Result<()>;
    async fn list_for_student(&self, student_record: StudentRecordId) -> Result<Vec<FeesCarryForwardLog>>;
}
```

## FeesCarryForwardSettingRepository

```rust
#[async_trait]
pub trait FeesCarryForwardSettingRepository: Send + Sync {
    async fn get(&self, id: FeesCarryForwardSettingId) -> Result<Option<FeesCarryForwardSetting>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<FeesCarryForwardSetting>>;
    async fn insert(&self, s: &FeesCarryForwardSetting) -> Result<()>;
    async fn update(&self, s: &FeesCarryForwardSetting) -> Result<()>;
}
```

## DirectFeesInstallmentRepository

```rust
#[async_trait]
pub trait DirectFeesInstallmentRepository: Send + Sync {
    async fn get(&self, id: DirectFeesInstallmentId) -> Result<Option<DirectFeesInstallment>>;
    async fn list_for_master(&self, master: FeesMasterId) -> Result<Vec<DirectFeesInstallment>>;
    async fn list_due_on(&self, school: SchoolId, date: NaiveDate) -> Result<Vec<DirectFeesInstallment>>;
    async fn insert(&self, i: &DirectFeesInstallment) -> Result<()>;
    async fn update(&self, i: &DirectFeesInstallment) -> Result<()>;
    async fn delete(&self, id: DirectFeesInstallmentId) -> Result<()>;
}
```

## DirectFeesInstallmentAssignRepository

```rust
#[async_trait]
pub trait DirectFeesInstallmentAssignRepository: Send + Sync {
    async fn get(&self, id: DirectFeesInstallmentAssignId) -> Result<Option<DirectFeesInstallmentAssign>>;
    async fn list_for_installment(&self, installment: DirectFeesInstallmentId) -> Result<Vec<DirectFeesInstallmentAssign>>;
    async fn list_for_student(&self, student: StudentId) -> Result<Vec<DirectFeesInstallmentAssign>>;
    async fn list_active(&self, school: SchoolId) -> Result<Vec<DirectFeesInstallmentAssign>>;
    async fn insert(&self, a: &DirectFeesInstallmentAssign) -> Result<()>;
    async fn update(&self, a: &DirectFeesInstallmentAssign) -> Result<()>;
    async fn cancel(&self, id: DirectFeesInstallmentAssignId) -> Result<()>;
}
```

## DirectFeesInstallmentChildPaymentRepository

```rust
#[async_trait]
pub trait DirectFeesInstallmentChildPaymentRepository: Send + Sync {
    async fn get(&self, id: DirectFeesInstallmentChildPaymentId) -> Result<Option<DirectFeesInstallmentChildPayment>>;
    async fn list_for_assign(&self, assign: DirectFeesInstallmentAssignId) -> Result<Vec<DirectFeesInstallmentChildPayment>>;
    async fn insert(&self, p: &DirectFeesInstallmentChildPayment) -> Result<()>;
}
```

## FmFeesGroupRepository

```rust
#[async_trait]
pub trait FmFeesGroupRepository: Send + Sync {
    async fn get(&self, id: FmFeesGroupId) -> Result<Option<FmFeesGroup>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<FmFeesGroup>>;
    async fn find_by_name(&self, school: SchoolId, name: &str) -> Result<Option<FmFeesGroup>>;
    async fn insert(&self, g: &FmFeesGroup) -> Result<()>;
    async fn update(&self, g: &FmFeesGroup) -> Result<()>;
    async fn delete(&self, id: FmFeesGroupId) -> Result<()>;
}
```

## FmFeesTypeRepository

```rust
#[async_trait]
pub trait FmFeesTypeRepository: Send + Sync {
    async fn get(&self, id: FmFeesTypeId) -> Result<Option<FmFeesType>>;
    async fn list_for_group(&self, group: FmFeesGroupId) -> Result<Vec<FmFeesType>>;
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<FmFeesType>>;
    async fn insert(&self, t: &FmFeesType) -> Result<()>;
    async fn update(&self, t: &FmFeesType) -> Result<()>;
    async fn delete(&self, id: FmFeesTypeId) -> Result<()>;
}
```

## FmFeesInvoiceRepository

```rust
#[async_trait]
pub trait FmFeesInvoiceRepository: Send + Sync {
    async fn get(&self, id: FmFeesInvoiceId) -> Result<Option<FmFeesInvoice>>;
    async fn get_by_invoice_number(&self, school: SchoolId, number: &str) -> Result<Option<FmFeesInvoice>>;
    async fn list_for_student(&self, student: StudentId, academic: AcademicYearId) -> Result<Vec<FmFeesInvoice>>;
    async fn list_for_class(&self, class: ClassId, academic: AcademicYearId) -> Result<Vec<FmFeesInvoice>>;
    async fn list_outstanding(&self, school: SchoolId, as_of: NaiveDate) -> Result<Vec<FmFeesInvoice>>;
    async fn insert(&self, i: &FmFeesInvoice) -> Result<()>;
    async fn update(&self, i: &FmFeesInvoice) -> Result<()>;
}
```

## FmFeesInvoiceChildRepository

```rust
#[async_trait]
pub trait FmFeesInvoiceChildRepository: Send + Sync {
    async fn list_for_invoice(&self, invoice: FmFeesInvoiceId) -> Result<Vec<FmFeesInvoiceChild>>;
    async fn insert(&self, c: &FmFeesInvoiceChild) -> Result<()>;
    async fn update(&self, c: &FmFeesInvoiceChild) -> Result<()>;
    async fn remove(&self, id: FmFeesInvoiceChildId) -> Result<()>;
}
```

## FmFeesTransactionRepository

```rust
#[async_trait]
pub trait FmFeesTransactionRepository: Send + Sync {
    async fn get(&self, id: FmFeesTransactionId) -> Result<Option<FmFeesTransaction>>;
    async fn list_for_invoice(&self, invoice: FmFeesInvoiceId) -> Result<Vec<FmFeesTransaction>>;
    async fn list_for_student(&self, student: StudentId, academic: AcademicYearId) -> Result<Vec<FmFeesTransaction>>;
    async fn insert(&self, t: &FmFeesTransaction) -> Result<()>;
    async fn reverse(&self, id: FmFeesTransactionId) -> Result<()>;
}
```

## FmFeesTransactionChildRepository

```rust
#[async_trait]
pub trait FmFeesTransactionChildRepository: Send + Sync {
    async fn list_for_transaction(&self, tx: FmFeesTransactionId) -> Result<Vec<FmFeesTransactionChild>>;
    async fn insert(&self, c: &FmFeesTransactionChild) -> Result<()>;
}
```

## FmFeesWeaverRepository

```rust
#[async_trait]
pub trait FmFeesWeaverRepository: Send + Sync {
    async fn list_for_invoice(&self, invoice: FmFeesInvoiceId) -> Result<Vec<FmFeesWeaver>>;
    async fn list_for_student(&self, student: StudentId) -> Result<Vec<FmFeesWeaver>>;
    async fn insert(&self, w: &FmFeesWeaver) -> Result<()>;
    async fn reverse(&self, id: FmFeesWeaverId) -> Result<()>;
}
```

## FeesInvoiceSettingRepository

```rust
#[async_trait]
pub trait FeesInvoiceSettingRepository: Send + Sync {
    async fn get(&self, id: FeesInvoiceSettingId) -> Result<Option<FeesInvoiceSetting>>;
    async fn get_for_school(&self, school: SchoolId, academic: AcademicYearId) -> Result<Option<FeesInvoiceSetting>>;
    async fn insert(&self, s: &FeesInvoiceSetting) -> Result<()>;
    async fn update(&self, s: &FeesInvoiceSetting) -> Result<()>;
}
```

## InvoiceSettingRepository

```rust
#[async_trait]
pub trait InvoiceSettingRepository: Send + Sync {
    async fn get(&self, id: InvoiceSettingId) -> Result<Option<InvoiceSetting>>;
    async fn get_for_school(&self, school: SchoolId, academic: AcademicYearId) -> Result<Option<InvoiceSetting>>;
    async fn insert(&self, s: &InvoiceSetting) -> Result<()>;
    async fn update(&self, s: &InvoiceSetting) -> Result<()>;
}
```

## FmFeesInvoiceSettingRepository

```rust
#[async_trait]
pub trait FmFeesInvoiceSettingRepository: Send + Sync {
    async fn get(&self, school: SchoolId) -> Result<Option<FmFeesInvoiceSetting>>;
    async fn insert(&self, s: &FmFeesInvoiceSetting) -> Result<()>;
    async fn update(&self, s: &FmFeesInvoiceSetting) -> Result<()>;
}
```

## BankAccountRepository

```rust
#[async_trait]
pub trait BankAccountRepository: Send + Sync {
    async fn get(&self, id: BankAccountId) -> Result<Option<BankAccount>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<BankAccount>>;
    async fn find_by_number(&self, school: SchoolId, number: &str) -> Result<Option<BankAccount>>;
    async fn insert(&self, a: &BankAccount) -> Result<()>;
    async fn update(&self, a: &BankAccount) -> Result<()>;
    async fn close(&self, id: BankAccountId) -> Result<()>;
}
```

## BankStatementRepository

```rust
#[async_trait]
pub trait BankStatementRepository: Send + Sync {
    async fn get(&self, id: BankStatementId) -> Result<Option<BankStatement>>;
    async fn list_for_account(&self, account: BankAccountId, from: NaiveDate, to: NaiveDate) -> Result<Vec<BankStatement>>;
    async fn list_for_school(&self, school: SchoolId, from: NaiveDate, to: NaiveDate) -> Result<Vec<BankStatement>>;
    async fn balance_after(&self, account: BankAccountId) -> Result<Amount>;
    async fn insert(&self, s: &BankStatement) -> Result<()>;
    async fn reverse(&self, id: BankStatementId) -> Result<()>;
}
```

## BankPaymentSlipRepository

```rust
#[async_trait]
pub trait BankPaymentSlipRepository: Send + Sync {
    async fn get(&self, id: BankPaymentSlipId) -> Result<Option<BankPaymentSlip>>;
    async fn list_pending(&self, school: SchoolId) -> Result<Vec<BankPaymentSlip>>;
    async fn list_for_student(&self, student: StudentId) -> Result<Vec<BankPaymentSlip>>;
    async fn find_by_slip_hash(&self, school: SchoolId, slip_hash: &str) -> Result<Option<BankPaymentSlip>>;
    async fn insert(&self, s: &BankPaymentSlip) -> Result<()>;
    async fn update(&self, s: &BankPaymentSlip) -> Result<()>;
}
```

## ExpenseRepository

```rust
#[async_trait]
pub trait ExpenseRepository: Send + Sync {
    async fn get(&self, id: ExpenseId) -> Result<Option<Expense>>;
    async fn list(&self, school: SchoolId, from: NaiveDate, to: NaiveDate) -> Result<Vec<Expense>>;
    async fn list_for_head(&self, head: ExpenseHeadId, from: NaiveDate, to: NaiveDate) -> Result<Vec<Expense>>;
    async fn list_for_account(&self, account: BankAccountId) -> Result<Vec<Expense>>;
    async fn insert(&self, e: &Expense) -> Result<()>;
    async fn update(&self, e: &Expense) -> Result<()>;
    async fn delete(&self, id: ExpenseId) -> Result<()>;
}
```

## IncomeRepository

```rust
#[async_trait]
pub trait IncomeRepository: Send + Sync {
    async fn get(&self, id: IncomeId) -> Result<Option<Income>>;
    async fn list(&self, school: SchoolId, from: NaiveDate, to: NaiveDate) -> Result<Vec<Income>>;
    async fn list_for_head(&self, head: IncomeHeadId, from: NaiveDate, to: NaiveDate) -> Result<Vec<Income>>;
    async fn list_for_account(&self, account: BankAccountId) -> Result<Vec<Income>>;
    async fn insert(&self, i: &Income) -> Result<()>;
    async fn update(&self, i: &Income) -> Result<()>;
    async fn delete(&self, id: IncomeId) -> Result<()>;
}
```

## DonorRepository

```rust
#[async_trait]
pub trait DonorRepository: Send + Sync {
    async fn get(&self, id: DonorId) -> Result<Option<Donor>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<Donor>>;
    async fn list_public(&self, school: SchoolId) -> Result<Vec<Donor>>;
    async fn find_by_email(&self, school: SchoolId, email: &str) -> Result<Option<Donor>>;
    async fn insert(&self, d: &Donor) -> Result<()>;
    async fn update(&self, d: &Donor) -> Result<()>;
    async fn delete(&self, id: DonorId) -> Result<()>;
}
```

## ExpenseHeadRepository / IncomeHeadRepository / ChartOfAccountRepository

```rust
#[async_trait]
pub trait ExpenseHeadRepository: Send + Sync {
    async fn get(&self, id: ExpenseHeadId) -> Result<Option<ExpenseHead>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<ExpenseHead>>;
    async fn insert(&self, h: &ExpenseHead) -> Result<()>;
    async fn update(&self, h: &ExpenseHead) -> Result<()>;
    async fn delete(&self, id: ExpenseHeadId) -> Result<()>;
}

#[async_trait]
pub trait IncomeHeadRepository: Send + Sync {
    async fn get(&self, id: IncomeHeadId) -> Result<Option<IncomeHead>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<IncomeHead>>;
    async fn insert(&self, h: &IncomeHead) -> Result<()>;
    async fn update(&self, h: &IncomeHead) -> Result<()>;
    async fn delete(&self, id: IncomeHeadId) -> Result<()>;
}

#[async_trait]
pub trait ChartOfAccountRepository: Send + Sync {
    async fn get(&self, id: ChartOfAccountId) -> Result<Option<ChartOfAccount>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<ChartOfAccount>>;
    async fn insert(&self, a: &ChartOfAccount) -> Result<()>;
    async fn update(&self, a: &ChartOfAccount) -> Result<()>;
    async fn delete(&self, id: ChartOfAccountId) -> Result<()>;
}
```

## WalletTransactionRepository

```rust
#[async_trait]
pub trait WalletTransactionRepository: Send + Sync {
    async fn get(&self, id: WalletTransactionId) -> Result<Option<WalletTransaction>>;
    async fn list_for_user(&self, user: UserId) -> Result<Vec<WalletTransaction>>;
    async fn list_pending(&self, school: SchoolId) -> Result<Vec<WalletTransaction>>;
    async fn balance(&self, user: UserId) -> Result<Amount>;
    async fn insert(&self, t: &WalletTransaction) -> Result<()>;
    async fn update(&self, t: &WalletTransaction) -> Result<()>;
}
```

## TransactionRepository

```rust
#[async_trait]
pub trait TransactionRepository: Send + Sync {
    async fn get(&self, id: TransactionId) -> Result<Option<Transaction>>;
    async fn list_for_morph(&self, morph_type: &str, morph_id: u64) -> Result<Vec<Transaction>>;
    async fn list_for_user(&self, user: UserId) -> Result<Vec<Transaction>>;
    async fn insert(&self, t: &Transaction) -> Result<()>;
    async fn reverse(&self, id: TransactionId) -> Result<()>;
}
```

## PayrollPaymentRepository

```rust
#[async_trait]
pub trait PayrollPaymentRepository: Send + Sync {
    async fn get(&self, id: PayrollPaymentId) -> Result<Option<PayrollPayment>>;
    async fn list_for_payroll(&self, payroll: PayrollGenerateId) -> Result<Vec<PayrollPayment>>;
    async fn list_for_staff(&self, staff: StaffId) -> Result<Vec<PayrollPayment>>;
    async fn insert(&self, p: &PayrollPayment) -> Result<()>;
}
```

## PayrollEarnDeducRepository

```rust
#[async_trait]
pub trait PayrollEarnDeducRepository: Send + Sync {
    async fn get(&self, id: PayrollEarnDeducId) -> Result<Option<PayrollEarnDeduc>>;
    async fn list_for_payroll(&self, payroll: PayrollGenerateId) -> Result<Vec<PayrollEarnDeduc>>;
    async fn insert(&self, e: &PayrollEarnDeduc) -> Result<()>;
    async fn update(&self, e: &PayrollEarnDeduc) -> Result<()>;
    async fn delete(&self, id: PayrollEarnDeducId) -> Result<()>;
}
```

## SalaryTemplateRepository

```rust
#[async_trait]
pub trait SalaryTemplateRepository: Send + Sync {
    async fn get(&self, id: SalaryTemplateId) -> Result<Option<SalaryTemplate>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<SalaryTemplate>>;
    async fn find_by_grade(&self, school: SchoolId, grade: &str) -> Result<Option<SalaryTemplate>>;
    async fn insert(&self, t: &SalaryTemplate) -> Result<()>;
    async fn update(&self, t: &SalaryTemplate) -> Result<()>;
    async fn delete(&self, id: SalaryTemplateId) -> Result<()>;
}
```

## ProductPurchaseRepository

```rust
#[async_trait]
pub trait ProductPurchaseRepository: Send + Sync {
    async fn get(&self, id: ProductPurchaseId) -> Result<Option<ProductPurchase>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<ProductPurchase>>;
    async fn list_for_user(&self, user: UserId) -> Result<Vec<ProductPurchase>>;
    async fn list_active(&self, school: SchoolId, as_of: NaiveDate) -> Result<Vec<ProductPurchase>>;
    async fn insert(&self, p: &ProductPurchase) -> Result<()>;
    async fn update(&self, p: &ProductPurchase) -> Result<()>;
}
```

## InventoryPaymentRepository

```rust
#[async_trait]
pub trait InventoryPaymentRepository: Send + Sync {
    async fn get(&self, id: InventoryPaymentId) -> Result<Option<InventoryPayment>>;
    async fn list_for_item(&self, item_id: u64) -> Result<Vec<InventoryPayment>>;
    async fn list_for_school(&self, school: SchoolId, from: NaiveDate, to: NaiveDate) -> Result<Vec<InventoryPayment>>;
    async fn insert(&self, p: &InventoryPayment) -> Result<()>;
    async fn update(&self, p: &InventoryPayment) -> Result<()>;
}
```

## AmountTransferRepository

```rust
#[async_trait]
pub trait AmountTransferRepository: Send + Sync {
    async fn get(&self, id: AmountTransferId) -> Result<Option<AmountTransfer>>;
    async fn list(&self, school: SchoolId, from: NaiveDate, to: NaiveDate) -> Result<Vec<AmountTransfer>>;
    async fn insert(&self, t: &AmountTransfer) -> Result<()>;
}
```

## PaymentGatewaySettingRepository

```rust
#[async_trait]
pub trait PaymentGatewaySettingRepository: Send + Sync {
    async fn get(&self, id: PaymentGatewaySettingId) -> Result<Option<PaymentGatewaySetting>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<PaymentGatewaySetting>>;
    async fn find_by_name(&self, school: SchoolId, name: &str) -> Result<Option<PaymentGatewaySetting>>;
    async fn insert(&self, g: &PaymentGatewaySetting) -> Result<()>;
    async fn update(&self, g: &PaymentGatewaySetting) -> Result<()>;
    async fn disable(&self, id: PaymentGatewaySettingId) -> Result<()>;
}
```

## PaymentMethodRepository

```rust
#[async_trait]
pub trait PaymentMethodRepository: Send + Sync {
    async fn get(&self, id: PaymentMethodId) -> Result<Option<PaymentMethod>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<PaymentMethod>>;
    async fn list_active(&self, school: SchoolId) -> Result<Vec<PaymentMethod>>;
    async fn insert(&self, m: &PaymentMethod) -> Result<()>;
    async fn update(&self, m: &PaymentMethod) -> Result<()>;
    async fn delete(&self, id: PaymentMethodId) -> Result<()>;
}
```

## DirectFeesReminderRepository

```rust
#[async_trait]
pub trait DirectFeesReminderRepository: Send + Sync {
    async fn get(&self, id: DirectFeesReminderId) -> Result<Option<DirectFeesReminder>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<DirectFeesReminder>>;
    async fn insert(&self, r: &DirectFeesReminder) -> Result<()>;
    async fn update(&self, r: &DirectFeesReminder) -> Result<()>;
    async fn delete(&self, id: DirectFeesReminderId) -> Result<()>;
}
```

## DirectFeesSettingRepository

```rust
#[async_trait]
pub trait DirectFeesSettingRepository: Send + Sync {
    async fn get(&self, school: SchoolId) -> Result<Option<DirectFeesSetting>>;
    async fn insert(&self, s: &DirectFeesSetting) -> Result<()>;
    async fn update(&self, s: &DirectFeesSetting) -> Result<()>;
}
```

## DueFeesLoginPreventRepository

```rust
#[async_trait]
pub trait DueFeesLoginPreventRepository: Send + Sync {
    async fn list(&self, school: SchoolId, academic: AcademicYearId) -> Result<Vec<DueFeesLoginPrevent>>;
    async fn find(&self, school: SchoolId, user: UserId, role: Option<RoleId>) -> Result<Option<DueFeesLoginPrevent>>;
    async fn insert(&self, d: &DueFeesLoginPrevent) -> Result<()>;
    async fn delete(&self, id: DueFeesLoginPreventId) -> Result<()>;
    async fn list_users_with_overdue(&self, school: SchoolId) -> Result<Vec<UserId>>;
}
```

## FeesInstallmentCreditRepository

```rust
#[async_trait]
pub trait FeesInstallmentCreditRepository: Send + Sync {
    async fn get(&self, id: FeesInstallmentCreditId) -> Result<Option<FeesInstallmentCredit>>;
    async fn list_for_student(&self, student: StudentId) -> Result<Vec<FeesInstallmentCredit>>;
    async fn balance(&self, student: StudentId) -> Result<Amount>;
    async fn insert(&self, c: &FeesInstallmentCredit) -> Result<()>;
    async fn consume(&self, id: FeesInstallmentCreditId, amount: Amount) -> Result<()>;
    async fn cancel(&self, id: FeesInstallmentCreditId) -> Result<()>;
}
```

## Indexes (recommended)

The default PostgreSQL adapter documents the following indexes;
consumers should declare them in their migrations:

```sql
-- Fees catalog
CREATE INDEX ix_fees_groups_school_id_academic ON finance_fees_groups (school_id, academic_id);
CREATE UNIQUE INDEX ux_fees_groups_school_id_name ON finance_fees_groups (school_id, name);
CREATE INDEX ix_fees_types_school_id_group ON finance_fees_types (school_id, fees_group_id);
CREATE UNIQUE INDEX ux_fees_masters_school_id_group_type_class_section_year
  ON finance_fees_masters (school_id, fees_group_id, fees_type_id, class_id, section_id, academic_id);
CREATE UNIQUE INDEX ux_fees_assigns_school_id_master_student_year
  ON fees_assigns (school_id, fees_master_id, student_id, academic_id);
CREATE INDEX ix_fees_assigns_school_id_class ON fees_assigns (school_id, class_id, section_id, academic_id);
CREATE INDEX ix_fees_payments_school_id_date ON finance_payments (school_id, payment_date);
CREATE INDEX ix_fees_payments_school_id_method ON finance_payments (school_id, payment_method_id);
CREATE INDEX ix_fees_payments_school_id_student ON finance_payments (school_id, student_id, academic_id);
CREATE INDEX ix_fees_installment_assigns_school_id_student ON finance_fees_installment_assigns (school_id, student_id);
CREATE INDEX ix_direct_fees_installment_assigns_school_id_student ON finance_direct_fees_installment_assigns (school_id, student_id);
CREATE INDEX ix_fees_carry_forwards_school_id_student ON fees_carry_forwards (school_id, student_id);
CREATE UNIQUE INDEX ux_fees_carry_forwards_school_id_student_year ON fees_carry_forwards (school_id, student_id, academic_id);
CREATE UNIQUE INDEX ux_fees_installment_credits_school_id_student_record ON fees_installment_credits (school_id, student_id, student_record_id);
-- FM invoice
CREATE INDEX ix_finance_invoices_school_id_student ON finance_invoices (school_id, student_id, academic_id);
CREATE UNIQUE INDEX ux_finance_fees_invoices_school_id_invoice_number ON finance_invoices (school_id, invoice_id);
CREATE INDEX ix_finance_invoices_school_id_class ON finance_invoices (school_id, class_id, academic_id);
CREATE INDEX ix_finance_fees_invoice_chields_invoice ON finance_invoice_lines (fees_invoice_id);
CREATE INDEX ix_finance_fees_transactions_school_id_invoice ON finance_transactions (school_id, fees_invoice_id);
-- Bank
CREATE UNIQUE INDEX ux_finance_bank_accounts_school_id_account_number ON finance_bank_accounts (school_id, account_number);
CREATE INDEX ix_finance_bank_statements_school_id_bank_date ON finance_bank_statements (school_id, bank_id, payment_date);
CREATE INDEX ix_finance_bank_payment_slips_school_id_status ON finance_bank_payment_slips (school_id, approve_status);
CREATE INDEX ix_finance_bank_payment_slips_school_id_student ON finance_bank_payment_slips (school_id, student_id);
-- Expense & income
CREATE INDEX ix_finance_add_expenses_school_id_date ON finance_add_expenses (school_id, date);
CREATE INDEX ix_finance_add_expenses_school_id_head ON finance_add_expenses (school_id, expense_head_id);
CREATE INDEX ix_finance_add_incomes_school_id_date ON finance_add_incomes (school_id, date);
CREATE INDEX ix_finance_add_incomes_school_id_head ON finance_add_incomes (school_id, income_head_id);
-- Wallet
CREATE INDEX ix_wallet_transactions_school_id_user ON wallet_transactions (school_id, user_id);
CREATE INDEX ix_wallet_transactions_school_id_status ON wallet_transactions (school_id, status);
-- Payroll
CREATE INDEX ix_payroll_payments_school_id_payroll ON payroll_payments (hr_payroll_generate_id);
CREATE INDEX ix_hr_hr_payroll_earn_deducs_school_id_payroll ON hr_payroll_earn_deducs (school_id, payroll_generate_id);
-- Due fees
CREATE INDEX ix_finance_due_fees_login_prevents_school_id_user ON due_fees_login_prevents (school_id, user_id, role_id);
-- Carry forward
CREATE INDEX ix_fees_carry_forward_logs_school_id_student ON fees_carry_forward_logs (school_id, student_record_id);
```

The `school_id` predicate is mandatory for tenant isolation.
