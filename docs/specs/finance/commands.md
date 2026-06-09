# Finance Domain — Commands

Commands describe intent. They are validated, authorized, and
dispatched to the relevant aggregate. Every command produces zero or
more events that are recorded in the event log.

All commands carry a `TenantContext` (school + actor + correlation)
and are rejected if the actor lacks the required capability.

## FeesGroup

### CreateFeesGroup

```rust
pub struct CreateFeesGroupCommand {
    pub tenant: TenantContext,
    pub name: String,
    pub description: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub due_date: NaiveDate,
}
```

**Capability:** `FeesGroup.Create`
**Pre-conditions:** `start_date <= due_date <= end_date`.
**Effects:** Emits `FeesGroupCreated`.

### UpdateFeesGroup / DeleteFeesGroup

Standard update and soft-delete commands. A group cannot be deleted
while referenced by a `FeesMaster`.

**Capabilities:** `FeesGroup.Update`, `FeesGroup.Delete`.

## FeesType

### CreateFeesType

```rust
pub struct CreateFeesTypeCommand {
    pub tenant: TenantContext,
    pub fees_group_id: FeesGroupId,
    pub name: String,
    pub description: Option<String>,
}
```

**Capability:** `FeesType.Create`
**Effects:** Emits `FeesTypeCreated`.

## FeesMaster

### CreateFeesMaster

```rust
pub struct CreateFeesMasterCommand {
    pub tenant: TenantContext,
    pub fees_group_id: FeesGroupId,
    pub fees_type_id: FeesTypeId,
    pub class_id: ClassId,
    pub section_id: Option<SectionId>,
    pub academic_id: AcademicYearId,
    pub amount: FeeAmount,
    pub due_date: Option<NaiveDate>,
}
```

**Capability:** `FeesMaster.Create`
**Effects:** Emits `FeesMasterCreated`. Triggers a
`FeesMasterReassigned` event in the academic domain that may also
re-issue fees assignments for the class.

### UpdateFeesMasterAmount

```rust
pub struct UpdateFeesMasterAmountCommand {
    pub tenant: TenantContext,
    pub fees_master_id: FeesMasterId,
    pub new_amount: FeeAmount,
}
```

**Capability:** `FeesMaster.Update`
**Effects:** Emits `FeesMasterAmountUpdated`. Existing
`FeesAssign`s with open balances are not retroactively changed; new
assignments use the new amount.

## FeesAssign

### AssignFeesToClass

```rust
pub struct AssignFeesToClassCommand {
    pub tenant: TenantContext,
    pub fees_master_id: FeesMasterId,
    pub class_id: ClassId,
    pub section_id: Option<SectionId>,
    pub academic_id: AcademicYearId,
    pub due_date: NaiveDate,
    pub fees_discount_id: Option<FeesDiscountId>,
    pub due_date_from_sem: Option<u8>,
    pub no_installment: Option<u16>,
}
```

**Capability:** `FeesAssign.Create`
**Pre-conditions:** The fees master is active. The class and section
exist. The academic year is current or open.

**Effects:** Creates one `FeesAssign` per active student in the scope,
applies the optional default discount, and emits
`FeesAssignedToClass` (one per student) and one master event
`FeesMasterAssigned` (for traceability).

### AssignFeesToStudent

```rust
pub struct AssignFeesToStudentCommand {
    pub tenant: TenantContext,
    pub fees_master_id: FeesMasterId,
    pub student_id: StudentId,
    pub record_id: StudentRecordId,
    pub fees_discount_id: Option<FeesDiscountId>,
}
```

**Capability:** `FeesAssign.Create`
**Effects:** Emits `FeesAssignedToStudent`.

### UpdateFeesAssignDiscount

```rust
pub struct UpdateFeesAssignDiscountCommand {
    pub tenant: TenantContext,
    pub fees_assign_id: FeesAssignId,
    pub fees_discount_id: FeesDiscountId,
    pub applied_amount: DiscountAmount,
}
```

**Capability:** `FeesAssign.Update`
**Effects:** Emits `FeesAssignDiscountUpdated`.

### CloseFeesAssign

```rust
pub struct CloseFeesAssignCommand {
    pub tenant: TenantContext,
    pub fees_assign_id: FeesAssignId,
    pub reason: CloseReason,
}
```

**Capability:** `FeesAssign.Close`
**Effects:** Emits `FeesAssignClosed`. The assignment cannot accept
further payments.

## FeesDiscount

### CreateFeesDiscount

```rust
pub struct CreateFeesDiscountCommand {
    pub tenant: TenantContext,
    pub name: String,
    pub code: DiscountCode,
    pub discount_type: DiscountType,
    pub amount: DiscountAmount,
    pub description: Option<String>,
}
```

**Capability:** `FeesDiscount.Create`
**Effects:** Emits `FeesDiscountCreated`.

## FeesInstallment

### CreateFeesInstallment

```rust
pub struct CreateFeesInstallmentCommand {
    pub tenant: TenantContext,
    pub fees_master_id: FeesMasterId,
    pub title: String,
    pub due_date: NaiveDate,
    pub amount: FeeAmount,
    pub percentage: FeePercentage,
}
```

**Capability:** `FeesInstallment.Create`
**Effects:** Emits `FeesInstallmentCreated`. Triggers the creation
of per-student assignments.

### AssignInstallmentToStudent

```rust
pub struct AssignInstallmentToStudentCommand {
    pub tenant: TenantContext,
    pub fees_installment_id: FeesInstallmentId,
    pub student_id: StudentId,
    pub record_id: StudentRecordId,
    pub fees_discount_id: Option<FeesDiscountId>,
    pub fees_type_id: Option<FeesTypeId>,
    pub amount: FeeAmount,
    pub due_date: NaiveDate,
}
```

**Capability:** `FeesInstallment.Assign`
**Effects:** Emits `FeesInstallmentAssigned`.

## FeesInvoice

### ConfigureInvoiceNumbering

```rust
pub struct ConfigureInvoiceNumberingCommand {
    pub tenant: TenantContext,
    pub prefix: InvoicePrefix,
    pub start_form: InvoiceStartForm,
}
```

**Capability:** `Invoice.Configure`
**Effects:** Emits `InvoiceNumberingConfigured`.

## Payment

### PayInvoice

```rust
pub struct PayInvoiceCommand {
    pub tenant: TenantContext,
    pub fees_assign_id: FeesAssignId,
    pub amount: Amount,
    pub payment_method_id: PaymentMethodId,
    pub bank_id: Option<BankAccountId>,
    pub note: Option<String>,
    pub slip: Option<SlipReference>,
    pub transaction_id: Option<TransactionId>,
    pub discount_month: Option<u8>,
    pub discount_amount: Option<DiscountAmount>,
    pub fine_amount: Option<FineAmount>,
    pub fine_title: Option<String>,
    pub service_charge: Option<ServiceCharge>,
}
```

**Capability:** `Payment.Collect`
**Pre-conditions:** The assignment exists, is not closed, and the
amount does not exceed the open balance. The payment method and bank
are compatible.

**Effects:** Emits `PaymentReceived` and a corresponding
`BankStatementRecorded` (when bank) and a `Transaction` line.

### PayInstallment

```rust
pub struct PayInstallmentCommand {
    pub tenant: TenantContext,
    pub fees_installment_assign_id: FeesInstallmentAssignId,
    pub amount: Amount,
    pub payment_method_id: PaymentMethodId,
    pub bank_id: Option<BankAccountId>,
    pub note: Option<String>,
    pub slip: Option<SlipReference>,
}
```

**Capability:** `Payment.Collect`
**Effects:** Emits `FeesInstallmentPaid` and a `BankStatement`.

### ReversePayment

```rust
pub struct ReversePaymentCommand {
    pub tenant: TenantContext,
    pub payment_id: FeesPaymentId,
    pub reason: String,
}
```

**Capability:** `Payment.Reverse`
**Effects:** Emits `PaymentReversed` and a negative
`BankStatement`.

## Direct Fees Installments

### ConfigureDirectFeesInstallment

```rust
pub struct ConfigureDirectFeesInstallmentCommand {
    pub tenant: TenantContext,
    pub fees_master_id: FeesMasterId,
    pub title: String,
    pub due_date: NaiveDate,
    pub amount: FeeAmount,
    pub percentage: FeePercentage,
}
```

**Capability:** `DirectFeesInstallment.Create`
**Effects:** Emits `DirectFeesInstallmentCreated`.

### AssignDirectInstallment

```rust
pub struct AssignDirectInstallmentCommand {
    pub tenant: TenantContext,
    pub direct_fees_installment_id: DirectFeesInstallmentId,
    pub student_ids: Vec<StudentId>,
    pub record_ids: Vec<StudentRecordId>,
    pub fees_type_id: Option<FeesTypeId>,
    pub fees_discount_id: Option<FeesDiscountId>,
    pub due_date: NaiveDate,
    pub note: Option<String>,
}
```

**Capability:** `DirectFeesInstallment.Assign`
**Effects:** Emits `DirectFeesInstallmentAssigned` per student.

### PayDirectInstallment

```rust
pub struct PayDirectInstallmentCommand {
    pub tenant: TenantContext,
    pub direct_fees_installment_assign_id: DirectFeesInstallmentAssignId,
    pub amount: Amount,
    pub payment_method_id: PaymentMethodId,
    pub bank_id: Option<BankAccountId>,
    pub slip: Option<SlipReference>,
    pub note: Option<String>,
}
```

**Capability:** `DirectFeesInstallment.Pay`
**Effects:** Emits `DirectFeesInstallmentPaid` and a child payment
event.

## Carry Forward

### CarryForwardFeesBalance

```rust
pub struct CarryForwardFeesBalanceCommand {
    pub tenant: TenantContext,
    pub student_id: StudentId,
    pub academic_id: AcademicYearId,
    pub target_academic_id: AcademicYearId,
    pub notes: Option<String>,
    pub due_date: Option<NaiveDate>,
    pub payment_gateway: Option<String>,
}
```

**Capability:** `FeesCarryForward.Execute`
**Pre-conditions:** The student has an open `FeesCarryForward` or an
outstanding balance in the source year.

**Effects:** Closes the source year balance, creates a
`FeesCarryForward` for the target year, writes a
`FeesCarryForwardLog` row, and emits `FeesCarriedForward`.

### ConfigureFeesCarryForward

```rust
pub struct ConfigureFeesCarryForwardCommand {
    pub tenant: TenantContext,
    pub title: String,
    pub fees_due_days: u16,
    pub payment_gateway: Option<String>,
}
```

**Capability:** `FeesCarryForward.Configure`
**Effects:** Emits `FeesCarryForwardConfigured`.

## Login Prevention

### ConfigureDirectFees

```rust
pub struct ConfigureDirectFeesCommand {
    pub tenant: TenantContext,
    pub fees_installment: bool,
    pub fees_reminder: bool,
    pub reminder_before: u16,
    pub no_installment: u16,
    pub due_date_from_sem: u8,
    pub end_day: Option<u8>,
}
```

**Capability:** `DirectFees.Configure`
**Effects:** Emits `DirectFeesConfigured`.

### ConfigureFeesReminder

```rust
pub struct ConfigureFeesReminderCommand {
    pub tenant: TenantContext,
    pub due_date_before: u16,
    pub notification_types: Vec<NotificationChannel>,
}
```

**Capability:** `FeesReminder.Configure`
**Effects:** Emits `FeesReminderConfigured`.

### BlockLoginForDueFees

```rust
pub struct BlockLoginForDueFeesCommand {
    pub tenant: TenantContext,
    pub user_id: UserId,
    pub role_id: Option<RoleId>,
    pub reason: PreventReason,
}
```

**Capability:** `DueFees.Block`
**Pre-conditions:** The user has a non-zero overdue balance.

**Effects:** Emits `DueFeesLoginPrevented`.

### UnblockLoginForDueFees

```rust
pub struct UnblockLoginForDueFeesCommand {
    pub tenant: TenantContext,
    pub user_id: UserId,
    pub role_id: Option<RoleId>,
}
```

**Capability:** `DueFees.Unblock`
**Effects:** Emits `DueFeesLoginRestored`.

## Bank

### OpenBankAccount

```rust
pub struct OpenBankAccountCommand {
    pub tenant: TenantContext,
    pub bank_name: String,
    pub account_name: String,
    pub account_number: BankAccountNumber,
    pub account_type: AccountType,
    pub opening_balance: Amount,
    pub note: Option<String>,
}
```

**Capability:** `Bank.Open`
**Effects:** Emits `BankAccountOpened` and an initial
`BankStatementRecorded` for the opening balance.

### RecordBankStatement

```rust
pub struct RecordBankStatementCommand {
    pub tenant: TenantContext,
    pub bank_id: BankAccountId,
    pub amount: Amount,
    pub statement_type: StatementType,
    pub payment_method_id: Option<PaymentMethodId>,
    pub details: Option<String>,
    pub payment_date: NaiveDate,
    pub reference_id: Option<ReferenceId>,
}
```

**Capability:** `Bank.Statement.Record`
**Effects:** Emits `BankStatementRecorded` and updates the bank's
`current_balance`.

### GenerateBankPaymentSlip

```rust
pub struct GenerateBankPaymentSlipCommand {
    pub tenant: TenantContext,
    pub bank_id: BankAccountId,
    pub student_id: StudentId,
    pub record_id: StudentRecordId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub fees_type_id: Option<FeesTypeId>,
    pub fees_discount_id: Option<FeesDiscountId>,
    pub amount: Amount,
    pub payment_mode: BankMode,
    pub date: NaiveDate,
    pub note: Option<String>,
    pub slip: Option<SlipReference>,
    pub assign_id: Option<FeesAssignId>,
    pub installment_id: Option<FeesInstallmentAssignId>,
}
```

**Capability:** `BankSlip.Generate`
**Effects:** Emits `BankPaymentSlipGenerated` with `pending` status.

### ApproveBankPayment

```rust
pub struct ApproveBankPaymentCommand {
    pub tenant: TenantContext,
    pub bank_payment_slip_id: BankPaymentSlipId,
    pub note: Option<String>,
}
```

**Capability:** `BankSlip.Approve`
**Pre-conditions:** The slip is `pending`.

**Effects:** Promotes the slip to a `BankStatement` and a
`FeesPayment`, and emits `BankPaymentApproved` and
`PaymentReceived`.

### RejectBankPayment

```rust
pub struct RejectBankPaymentCommand {
    pub tenant: TenantContext,
    pub bank_payment_slip_id: BankPaymentSlipId,
    pub reason: String,
}
```

**Capability:** `BankSlip.Reject`
**Effects:** Emits `BankPaymentRejected`. The slip is preserved for
audit; no funds are applied.

### TransferFunds

```rust
pub struct TransferFundsCommand {
    pub tenant: TenantContext,
    pub from_bank_id: BankAccountId,
    pub to_bank_id: BankAccountId,
    pub amount: Amount,
    pub note: Option<String>,
    pub transfer_date: NaiveDate,
}
```

**Capability:** `Bank.Transfer`
**Pre-conditions:** `from_bank_id != to_bank_id`. The source account
has sufficient balance.

**Effects:** Emits `FundsTransferred` and two `BankStatementRecorded`
events (debit, credit).

## Expense & Income

### RecordExpense

```rust
pub struct RecordExpenseCommand {
    pub tenant: TenantContext,
    pub name: String,
    pub date: NaiveDate,
    pub amount: Amount,
    pub expense_head_id: ExpenseHeadId,
    pub account_id: BankAccountId,
    pub payment_method_id: PaymentMethodId,
    pub description: Option<String>,
    pub file: Option<FileReference>,
    pub item_receive_id: Option<u64>,
    pub inventory_id: Option<u64>,
    pub payroll_payment_id: Option<PayrollPaymentId>,
}
```

**Capability:** `Expense.Create`
**Effects:** Emits `ExpenseRecorded` and a `BankStatementRecorded`
(debit).

### RecordIncome

```rust
pub struct RecordIncomeCommand {
    pub tenant: TenantContext,
    pub name: String,
    pub date: NaiveDate,
    pub amount: Amount,
    pub income_head_id: IncomeHeadId,
    pub account_id: BankAccountId,
    pub payment_method_id: PaymentMethodId,
    pub description: Option<String>,
    pub file: Option<FileReference>,
    pub item_sell_id: Option<u64>,
    pub fees_collection_id: Option<u64>,
    pub inventory_id: Option<u64>,
    pub installment_payment_id: Option<FeesInstallmentAssignId>,
}
```

**Capability:** `Income.Create`
**Effects:** Emits `IncomeRecorded` and a `BankStatementRecorded`
(credit).

### CreateExpenseHead / UpdateExpenseHead / DeleteExpenseHead

Standard CRUD. The head cannot be deleted while referenced by an
expense.

**Capabilities:** `ExpenseHead.Create`, `ExpenseHead.Update`,
`ExpenseHead.Delete`.

### CreateIncomeHead / UpdateIncomeHead / DeleteIncomeHead

Standard CRUD.

**Capabilities:** `IncomeHead.Create`, `IncomeHead.Update`,
`IncomeHead.Delete`.

### RegisterDonor / UpdateDonor / DeleteDonor

Standard CRUD on `Donor`.

**Capabilities:** `Donor.Create`, `Donor.Update`, `Donor.Delete`.

## Wallet

### AddWalletCredit

```rust
pub struct AddWalletCreditCommand {
    pub tenant: TenantContext,
    pub user_id: UserId,
    pub amount: Amount,
    pub payment_method_id: PaymentMethodId,
    pub bank_id: Option<BankAccountId>,
    pub note: Option<String>,
    pub file: Option<FileReference>,
    pub wallet_type: WalletTxType, // deposit or refund
}
```

**Capability:** `Wallet.Credit`
**Pre-conditions:** A bank statement has been recorded if a bank
account is provided.

**Effects:** Emits `WalletCredited` and a `WalletTransactionApproved`
on auto-approval. On manual approval, a separate command is required.

### ApproveWalletTransaction

```rust
pub struct ApproveWalletTransactionCommand {
    pub tenant: TenantContext,
    pub wallet_transaction_id: WalletTransactionId,
}
```

**Capability:** `Wallet.Approve`
**Effects:** Emits `WalletTransactionApproved`. The wallet balance
is updated.

### RejectWalletTransaction

```rust
pub struct RejectWalletTransactionCommand {
    pub tenant: TenantContext,
    pub wallet_transaction_id: WalletTransactionId,
    pub reject_note: String,
}
```

**Capability:** `Wallet.Reject`
**Effects:** Emits `WalletTransactionRejected`. No wallet movement.

### DeductWalletCredit

```rust
pub struct DeductWalletCreditCommand {
    pub tenant: TenantContext,
    pub user_id: UserId,
    pub amount: Amount,
    pub wallet_type: WalletTxType, // expense or fees_refund
    pub note: Option<String>,
    pub reference_id: Option<ReferenceId>,
}
```

**Capability:** `Wallet.Debit`
**Pre-conditions:** The user has sufficient wallet balance.

**Effects:** Emits `WalletDebited` and `WalletTransactionApproved`.

## Payroll

### GeneratePayroll

```rust
pub struct GeneratePayrollCommand {
    pub tenant: TenantContext,
    pub staff_id: StaffId,
    pub pay_period: PayPeriod,
    pub salary_template_id: Option<SalaryTemplateId>,
    pub earnings: Vec<PayrollEarningLine>,
    pub deductions: Vec<PayrollDeductionLine>,
    pub note: Option<String>,
}
```

**Capability:** `Payroll.Generate`
**Pre-conditions:** The staff has a salary template or hourly rate
configured. The pay period is open.

**Effects:** Emits `PayrollGenerated` on the HR-owned aggregate and
the corresponding `PayrollEarningAdded` and `PayrollDeductionAdded`
events.

### ApprovePayroll

```rust
pub struct ApprovePayrollCommand {
    pub tenant: TenantContext,
    pub payroll_generate_id: PayrollGenerateId,
}
```

**Capability:** `Payroll.Approve`
**Effects:** Emits `PayrollApproved`.

### RecordPayrollPayment

```rust
pub struct RecordPayrollPaymentCommand {
    pub tenant: TenantContext,
    pub payroll_generate_id: PayrollGenerateId,
    pub amount: Amount,
    pub payment_method_id: PaymentMethodId,
    pub bank_id: Option<BankAccountId>,
    pub payment_date: NaiveDate,
    pub note: Option<String>,
}
```

**Capability:** `Payroll.Pay`
**Pre-conditions:** The payroll is `Generated`. The amount does not
exceed the unpaid balance.

**Effects:** Emits `PayrollPaymentRecorded`, a `BankStatement`, an
`Expense` (on the salary expense head), and `PayrollPaid` when the
payroll is fully paid.

## Inventory & Product

### RecordInventoryPayment

```rust
pub struct RecordInventoryPaymentCommand {
    pub tenant: TenantContext,
    pub item_receive_sell_id: u64,
    pub payment_type: PaymentType, // receive or sell
    pub amount: Amount,
    pub payment_date: NaiveDate,
    pub reference_no: Option<String>,
    pub payment_method_id: PaymentMethodId,
    pub notes: Option<String>,
}
```

**Capability:** `Inventory.Payment.Record`
**Effects:** Emits `InventoryPaymentRecorded` and either an
`ExpenseRecorded` or an `IncomeRecorded` (depending on
`payment_type`).

### RecordProductPurchase

```rust
pub struct RecordProductPurchaseCommand {
    pub tenant: TenantContext,
    pub user_id: UserId,
    pub staff_id: Option<StaffId>,
    pub purchase_date: NaiveDate,
    pub expiry_date: NaiveDate,
    pub price: Amount,
    pub package: ProductPackage,
    pub paid_amount: Amount,
    pub due_amount: Amount,
}
```

**Capability:** `Product.Purchase`
**Pre-conditions:** `paid_amount + due_amount == price`.

**Effects:** Emits `ProductPurchaseRecorded`.

### RecordProductPayment

```rust
pub struct RecordProductPaymentCommand {
    pub tenant: TenantContext,
    pub product_purchase_id: ProductPurchaseId,
    pub amount: Amount,
    pub payment_method_id: PaymentMethodId,
    pub bank_id: Option<BankAccountId>,
    pub payment_date: NaiveDate,
    pub reference: Option<String>,
}
```

**Capability:** `Product.Payment`
**Effects:** Emits `ProductPaymentRecorded` and a `BankStatement`.

## Settings

### ConfigureInvoiceSettings

```rust
pub struct ConfigureInvoiceSettingsCommand {
    pub tenant: TenantContext,
    pub academic_id: AcademicYearId,
    pub invoice_type: InvoiceType,
    pub per_th: PerThousand,
    pub prefix: Option<InvoicePrefix>,
    pub show_student_name: bool,
    pub show_student_section: bool,
    pub show_student_class: bool,
    pub show_student_roll: bool,
    pub show_student_group: bool,
    pub show_student_admission_no: bool,
    pub footer_1: String,
    pub footer_2: String,
    pub footer_3: String,
    pub signature_parent: bool,
    pub signature_cashier: bool,
    pub signature_officer: bool,
    pub copy_parent: String,
    pub copy_office: String,
    pub copy_cashier: String,
    pub copyright_msg: Option<String>,
}
```

**Capability:** `Invoice.Setting.Configure`
**Effects:** Emits `InvoiceSettingConfigured` (or
`FeesInvoiceSettingConfigured` for the classic scheme).

### ConfigurePaymentGateway

```rust
pub struct ConfigurePaymentGatewayCommand {
    pub tenant: TenantContext,
    pub gateway_name: GatewayName,
    pub gateway_username: Option<String>,
    pub gateway_password: Secret<String>,
    pub gateway_signature: Option<String>,
    pub gateway_client_id: Option<String>,
    pub gateway_secret_key: Secret<Option<String>>,
    pub gateway_secret_word: Secret<Option<String>>,
    pub gateway_publisher_key: Option<String>,
    pub gateway_private_key: Secret<Option<String>>,
    pub gateway_mode: GatewayMode,
    pub bank_details: Option<String>,
    pub cheque_details: Option<String>,
    pub service_charge: bool,
    pub charge_type: Option<ServiceChargeType>,
    pub charge: Option<ServiceCharge>,
}
```

**Capability:** `PaymentGateway.Configure`
**Effects:** Emits `PaymentGatewayConfigured`. Credentials are
encrypted by the storage adapter.

### CreatePaymentMethod / UpdatePaymentMethod / DeletePaymentMethod

```rust
pub struct CreatePaymentMethodCommand {
    pub tenant: TenantContext,
    pub method: String,
    pub method_type: PaymentMethodKind,
    pub gateway_id: Option<PaymentGatewaySettingId>,
}
```

**Capabilities:** `PaymentMethod.Create`, `PaymentMethod.Update`,
`PaymentMethod.Delete`.

## Question Bank Fees

### AttachFeesToQuestionBank

```rust
pub struct AttachFeesToQuestionBankCommand {
    pub tenant: TenantContext,
    pub question_bank_id: QuestionBankId,
    pub fees_type_id: FeesTypeId,
    pub class_id: Option<ClassId>,
    pub section_id: Option<SectionId>,
}
```

**Capability:** `QuestionBank.Fee.Attach`
**Effects:** Emits `FeesAttachedToQuestionBank`.

## Chart of Account

### CreateChartOfAccount / UpdateChartOfAccount / DeleteChartOfAccount

```rust
pub struct CreateChartOfAccountCommand {
    pub tenant: TenantContext,
    pub name: String,
    pub account_type: ChartAccountType, // asset, liability, income, expense, equity
    pub parent_id: Option<ChartOfAccountId>,
}
```

**Capabilities:** `ChartOfAccount.Create`, `ChartOfAccount.Update`,
`ChartOfAccount.Delete`.

## Salary Template

### CreateSalaryTemplate

```rust
pub struct CreateSalaryTemplateCommand {
    pub tenant: TenantContext,
    pub salary_grades: String,
    pub salary_basic: String,
    pub overtime_rate: String,
    pub house_rent: Amount,
    pub provident_fund: Amount,
    pub gross_salary: Amount,
    pub total_deduction: Amount,
    pub net_salary: Amount,
}
```

**Capability:** `SalaryTemplate.Create`
**Effects:** Emits `SalaryTemplateCreated`.

## Hourly Rate (consulted by finance for payroll)

### SetHourlyRate

```rust
pub struct SetHourlyRateCommand {
    pub tenant: TenantContext,
    pub grade: String,
    pub rate: HourlyRate,
}
```

**Capability:** `HourlyRate.Set`
**Effects:** Emits `HourlyRateSet` (consumed by HR domain; finance
reads it during payroll generation).

## Installment Credit

### AddFeesInstallmentCredit

```rust
pub struct AddFeesInstallmentCreditCommand {
    pub tenant: TenantContext,
    pub student_id: StudentId,
    pub record_id: StudentRecordId,
    pub amount: Amount,
}
```

**Capability:** `FeesInstallmentCredit.Add`
**Effects:** Emits `FeesInstallmentCreditAdded`.

### ConsumeFeesInstallmentCredit

```rust
pub struct ConsumeFeesInstallmentCreditCommand {
    pub tenant: TenantContext,
    pub fees_installment_credit_id: FeesInstallmentCreditId,
    pub fees_installment_assign_id: FeesInstallmentAssignId,
    pub applied_amount: Amount,
}
```

**Capability:** `FeesInstallmentCredit.Consume`
**Effects:** Emits `FeesInstallmentCreditConsumed` and an
`FinanceWeaverApplied`-equivalent adjustment.
