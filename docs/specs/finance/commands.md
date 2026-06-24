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

## Orphaned Items (Cluster D catch-up)

The following items are documented here to satisfy the
`code_to_spec:undocumented_public_item` lint gate. They were
discovered after the main spec was written.

### Approve Bank Slip

```rust
pub struct ApproveBankSlipCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `BankSlip.Approve`
**Effects:** Emits `BankSlipApproveed`.


### Approve Expense

```rust
pub struct ApproveExpenseCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Expense.Approve`
**Effects:** Emits `ExpenseApproveed`.


### Approve Income

```rust
pub struct ApproveIncomeCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Income.Approve`
**Effects:** Emits `IncomeApproveed`.


### Approve Payroll Payment

```rust
pub struct ApprovePayrollPaymentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `PayrollPayment.Approve`
**Effects:** Emits `PayrollPaymentApproveed`.


### Cancel Invoice

```rust
pub struct CancelInvoiceCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Invoice.Cancel`
**Effects:** Emits `InvoiceCanceled`.


### Configure Due Fees Block Setting

```rust
pub struct ConfigureDueFeesBlockSettingCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DueFeesBlockSetting.Configure`
**Effects:** Emits `DueFeesBlockSettingConfigureed`.


### Configure Fees Group

```rust
pub struct ConfigureFeesGroupCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesGroup.Configure`
**Effects:** Emits `FeesGroupConfigureed`.


### Configure Fees Type

```rust
pub struct ConfigureFeesTypeCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesType.Configure`
**Effects:** Emits `FeesTypeConfigureed`.


### Create Amount Transfer

```rust
pub struct CreateAmountTransferCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `AmountTransfer.Create`
**Effects:** Emits `AmountTransferCreateed`.


### Create Direct Fees Installment Assign

```rust
pub struct CreateDirectFeesInstallmentAssignCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DirectFeesInstallmentAssign.Create`
**Effects:** Emits `DirectFeesInstallmentAssignCreateed`.


### Create Direct Fees Installment Child Payment

```rust
pub struct CreateDirectFeesInstallmentChildPaymentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DirectFeesInstallmentChildPayment.Create`
**Effects:** Emits `DirectFeesInstallmentChildPaymentCreateed`.


### Create Direct Fees Installment

```rust
pub struct CreateDirectFeesInstallmentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DirectFeesInstallment.Create`
**Effects:** Emits `DirectFeesInstallmentCreateed`.


### Create Direct Fees Reminder

```rust
pub struct CreateDirectFeesReminderCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DirectFeesReminder.Create`
**Effects:** Emits `DirectFeesReminderCreateed`.


### Create Direct Fees Setting

```rust
pub struct CreateDirectFeesSettingCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DirectFeesSetting.Create`
**Effects:** Emits `DirectFeesSettingCreateed`.


### Create Donor

```rust
pub struct CreateDonorCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Donor.Create`
**Effects:** Emits `DonorCreateed`.


### Create Expense Head

```rust
pub struct CreateExpenseHeadCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ExpenseHead.Create`
**Effects:** Emits `ExpenseHeadCreateed`.


### Create Fees Assign

```rust
pub struct CreateFeesAssignCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesAssign.Create`
**Effects:** Emits `FeesAssignCreateed`.


### Create Fees Assign Discount

```rust
pub struct CreateFeesAssignDiscountCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesAssignDiscount.Create`
**Effects:** Emits `FeesAssignDiscountCreateed`.


### Create Fees Installment Credit

```rust
pub struct CreateFeesInstallmentCreditCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesInstallmentCredit.Create`
**Effects:** Emits `FeesInstallmentCreditCreateed`.


### Create Fees Invoice Setting

```rust
pub struct CreateFeesInvoiceSettingCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesInvoiceSetting.Create`
**Effects:** Emits `FeesInvoiceSettingCreateed`.


### Create Fm Fees Group

```rust
pub struct CreateFmFeesGroupCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesGroup.Create`
**Effects:** Emits `FmFeesGroupCreateed`.


### Create Fm Fees Invoice Child

```rust
pub struct CreateFmFeesInvoiceChildCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesInvoiceChild.Create`
**Effects:** Emits `FmFeesInvoiceChildCreateed`.


### Create Fm Fees Invoice

```rust
pub struct CreateFmFeesInvoiceCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesInvoice.Create`
**Effects:** Emits `FmFeesInvoiceCreateed`.


### Create Fm Fees Invoice Setting

```rust
pub struct CreateFmFeesInvoiceSettingCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesInvoiceSetting.Create`
**Effects:** Emits `FmFeesInvoiceSettingCreateed`.


### Create Fm Fees Transaction Child

```rust
pub struct CreateFmFeesTransactionChildCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesTransactionChild.Create`
**Effects:** Emits `FmFeesTransactionChildCreateed`.


### Create Fm Fees Transaction

```rust
pub struct CreateFmFeesTransactionCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesTransaction.Create`
**Effects:** Emits `FmFeesTransactionCreateed`.


### Create Fm Fees Type

```rust
pub struct CreateFmFeesTypeCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesType.Create`
**Effects:** Emits `FmFeesTypeCreateed`.


### Create Fm Fees Weaver

```rust
pub struct CreateFmFeesWeaverCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesWeaver.Create`
**Effects:** Emits `FmFeesWeaverCreateed`.


### Create Income

```rust
pub struct CreateIncomeCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Income.Create`
**Effects:** Emits `IncomeCreateed`.


### Create Income Head

```rust
pub struct CreateIncomeHeadCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `IncomeHead.Create`
**Effects:** Emits `IncomeHeadCreateed`.


### Create Inventory Payment

```rust
pub struct CreateInventoryPaymentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `InventoryPayment.Create`
**Effects:** Emits `InventoryPaymentCreateed`.


### Create Payment Gateway

```rust
pub struct CreatePaymentGatewayCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `PaymentGateway.Create`
**Effects:** Emits `PaymentGatewayCreateed`.


### Create Product Purchase

```rust
pub struct CreateProductPurchaseCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ProductPurchase.Create`
**Effects:** Emits `ProductPurchaseCreateed`.


### Create Transaction

```rust
pub struct CreateTransactionCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Transaction.Create`
**Effects:** Emits `TransactionCreateed`.


### Delete Bank Account

```rust
pub struct DeleteBankAccountCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `BankAccount.Delete`
**Effects:** Emits `BankAccountDeleteed`.


### Delete Direct Fees Installment Assign

```rust
pub struct DeleteDirectFeesInstallmentAssignCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DirectFeesInstallmentAssign.Delete`
**Effects:** Emits `DirectFeesInstallmentAssignDeleteed`.


### Delete Direct Fees Installment

```rust
pub struct DeleteDirectFeesInstallmentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DirectFeesInstallment.Delete`
**Effects:** Emits `DirectFeesInstallmentDeleteed`.


### Delete Direct Fees Reminder

```rust
pub struct DeleteDirectFeesReminderCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DirectFeesReminder.Delete`
**Effects:** Emits `DirectFeesReminderDeleteed`.


### Delete Direct Fees Setting

```rust
pub struct DeleteDirectFeesSettingCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DirectFeesSetting.Delete`
**Effects:** Emits `DirectFeesSettingDeleteed`.


### Delete Expense

```rust
pub struct DeleteExpenseCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Expense.Delete`
**Effects:** Emits `ExpenseDeleteed`.


### Delete Expense Head

```rust
pub struct DeleteExpenseHeadCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ExpenseHead.Delete`
**Effects:** Emits `ExpenseHeadDeleteed`.


### Delete Fees Assign

```rust
pub struct DeleteFeesAssignCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesAssign.Delete`
**Effects:** Emits `FeesAssignDeleteed`.


### Delete Fees Discount

```rust
pub struct DeleteFeesDiscountCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesDiscount.Delete`
**Effects:** Emits `FeesDiscountDeleteed`.


### Delete Fees Group

```rust
pub struct DeleteFeesGroupCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesGroup.Delete`
**Effects:** Emits `FeesGroupDeleteed`.


### Delete Fees Installment

```rust
pub struct DeleteFeesInstallmentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesInstallment.Delete`
**Effects:** Emits `FeesInstallmentDeleteed`.


### Delete Fees Master

```rust
pub struct DeleteFeesMasterCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesMaster.Delete`
**Effects:** Emits `FeesMasterDeleteed`.


### Delete Fees Type

```rust
pub struct DeleteFeesTypeCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesType.Delete`
**Effects:** Emits `FeesTypeDeleteed`.


### Delete Income

```rust
pub struct DeleteIncomeCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Income.Delete`
**Effects:** Emits `IncomeDeleteed`.


### Delete Income Head

```rust
pub struct DeleteIncomeHeadCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `IncomeHead.Delete`
**Effects:** Emits `IncomeHeadDeleteed`.


### Delete Payment Gateway

```rust
pub struct DeletePaymentGatewayCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `PaymentGateway.Delete`
**Effects:** Emits `PaymentGatewayDeleteed`.


### Delete Payment Method

```rust
pub struct DeletePaymentMethodCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `PaymentMethod.Delete`
**Effects:** Emits `PaymentMethodDeleteed`.


### Generate Bank Slip

```rust
pub struct GenerateBankSlipCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `BankSlip.Generate`
**Effects:** Emits `BankSlipGenerateed`.


### Generate Invoice

```rust
pub struct GenerateInvoiceCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Invoice.Generate`
**Effects:** Emits `InvoiceGenerateed`.


### Pay Payroll

```rust
pub struct PayPayrollCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Payroll.Pay`
**Effects:** Emits `PayrollPayed`.


### Pay Payroll Payment

```rust
pub struct PayPayrollPaymentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `PayrollPayment.Pay`
**Effects:** Emits `PayrollPaymentPayed`.


### Read Amount Transfer

```rust
pub struct ReadAmountTransferCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `AmountTransfer.Read`
**Effects:** Emits `AmountTransferReaded`.


### Read Balance Sheet Report

```rust
pub struct ReadBalanceSheetReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `BalanceSheetReport.Read`
**Effects:** Emits `BalanceSheetReportReaded`.


### Read Bank Account

```rust
pub struct ReadBankAccountCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `BankAccount.Read`
**Effects:** Emits `BankAccountReaded`.


### Read Bank Slip

```rust
pub struct ReadBankSlipCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `BankSlip.Read`
**Effects:** Emits `BankSlipReaded`.


### Read Bank Statement

```rust
pub struct ReadBankStatementCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `BankStatement.Read`
**Effects:** Emits `BankStatementReaded`.


### Read Bank Statement Report

```rust
pub struct ReadBankStatementReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `BankStatementReport.Read`
**Effects:** Emits `BankStatementReportReaded`.


### Read Cash Flow Report

```rust
pub struct ReadCashFlowReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `CashFlowReport.Read`
**Effects:** Emits `CashFlowReportReaded`.


### Read Chart Of Account

```rust
pub struct ReadChartOfAccountCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ChartOfAccount.Read`
**Effects:** Emits `ChartOfAccountReaded`.


### Read Class Wise Collection Report

```rust
pub struct ReadClassWiseCollectionReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ClassWiseCollectionReport.Read`
**Effects:** Emits `ClassWiseCollectionReportReaded`.


### Read Daily Collection Report

```rust
pub struct ReadDailyCollectionReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DailyCollectionReport.Read`
**Effects:** Emits `DailyCollectionReportReaded`.


### Read Direct Fees Installment Child Payment

```rust
pub struct ReadDirectFeesInstallmentChildPaymentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DirectFeesInstallmentChildPayment.Read`
**Effects:** Emits `DirectFeesInstallmentChildPaymentReaded`.


### Read Direct Fees Installment

```rust
pub struct ReadDirectFeesInstallmentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DirectFeesInstallment.Read`
**Effects:** Emits `DirectFeesInstallmentReaded`.


### Read Donor

```rust
pub struct ReadDonorCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Donor.Read`
**Effects:** Emits `DonorReaded`.


### Read Due Fees Block

```rust
pub struct ReadDueFeesBlockCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DueFeesBlock.Read`
**Effects:** Emits `DueFeesBlockReaded`.


### Read Due Fees Report

```rust
pub struct ReadDueFeesReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DueFeesReport.Read`
**Effects:** Emits `DueFeesReportReaded`.


### Read Expense Report

```rust
pub struct ReadExpenseReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ExpenseReport.Read`
**Effects:** Emits `ExpenseReportReaded`.


### Read Fees Assign Discount

```rust
pub struct ReadFeesAssignDiscountCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesAssignDiscount.Read`
**Effects:** Emits `FeesAssignDiscountReaded`.


### Read Fees Carry Forward

```rust
pub struct ReadFeesCarryForwardCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesCarryForward.Read`
**Effects:** Emits `FeesCarryForwardReaded`.


### Read Fees Carry Forward Log

```rust
pub struct ReadFeesCarryForwardLogCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesCarryForwardLog.Read`
**Effects:** Emits `FeesCarryForwardLogReaded`.


### Read Fees Collection Report

```rust
pub struct ReadFeesCollectionReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesCollectionReport.Read`
**Effects:** Emits `FeesCollectionReportReaded`.


### Read Fees Discount

```rust
pub struct ReadFeesDiscountCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesDiscount.Read`
**Effects:** Emits `FeesDiscountReaded`.


### Read Fees Discount Report

```rust
pub struct ReadFeesDiscountReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesDiscountReport.Read`
**Effects:** Emits `FeesDiscountReportReaded`.


### Read Fees Group

```rust
pub struct ReadFeesGroupCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesGroup.Read`
**Effects:** Emits `FeesGroupReaded`.


### Read Fees Installment Credit

```rust
pub struct ReadFeesInstallmentCreditCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesInstallmentCredit.Read`
**Effects:** Emits `FeesInstallmentCreditReaded`.


### Read Fees Invoice Setting

```rust
pub struct ReadFeesInvoiceSettingCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesInvoiceSetting.Read`
**Effects:** Emits `FeesInvoiceSettingReaded`.


### Read Fees Master

```rust
pub struct ReadFeesMasterCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesMaster.Read`
**Effects:** Emits `FeesMasterReaded`.


### Read Fees Payment

```rust
pub struct ReadFeesPaymentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesPayment.Read`
**Effects:** Emits `FeesPaymentReaded`.


### Read Fees Payment Slip

```rust
pub struct ReadFeesPaymentSlipCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesPaymentSlip.Read`
**Effects:** Emits `FeesPaymentSlipReaded`.


### Read Fees Type

```rust
pub struct ReadFeesTypeCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesType.Read`
**Effects:** Emits `FeesTypeReaded`.


### Read Fm Fees Group

```rust
pub struct ReadFmFeesGroupCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesGroup.Read`
**Effects:** Emits `FmFeesGroupReaded`.


### Read Fm Fees Invoice Child

```rust
pub struct ReadFmFeesInvoiceChildCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesInvoiceChild.Read`
**Effects:** Emits `FmFeesInvoiceChildReaded`.


### Read Fm Fees Invoice

```rust
pub struct ReadFmFeesInvoiceCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesInvoice.Read`
**Effects:** Emits `FmFeesInvoiceReaded`.


### Read Fm Fees Invoice Setting

```rust
pub struct ReadFmFeesInvoiceSettingCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesInvoiceSetting.Read`
**Effects:** Emits `FmFeesInvoiceSettingReaded`.


### Read Fm Fees Transaction Child

```rust
pub struct ReadFmFeesTransactionChildCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesTransactionChild.Read`
**Effects:** Emits `FmFeesTransactionChildReaded`.


### Read Fm Fees Transaction

```rust
pub struct ReadFmFeesTransactionCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesTransaction.Read`
**Effects:** Emits `FmFeesTransactionReaded`.


### Read Fm Fees Type

```rust
pub struct ReadFmFeesTypeCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesType.Read`
**Effects:** Emits `FmFeesTypeReaded`.


### Read Fm Fees Weaver

```rust
pub struct ReadFmFeesWeaverCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesWeaver.Read`
**Effects:** Emits `FmFeesWeaverReaded`.


### Read Head Wise Expense Report

```rust
pub struct ReadHeadWiseExpenseReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `HeadWiseExpenseReport.Read`
**Effects:** Emits `HeadWiseExpenseReportReaded`.


### Read Head Wise Income Report

```rust
pub struct ReadHeadWiseIncomeReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `HeadWiseIncomeReport.Read`
**Effects:** Emits `HeadWiseIncomeReportReaded`.


### Read Income Report

```rust
pub struct ReadIncomeReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `IncomeReport.Read`
**Effects:** Emits `IncomeReportReaded`.


### Read Inventory Payment

```rust
pub struct ReadInventoryPaymentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `InventoryPayment.Read`
**Effects:** Emits `InventoryPaymentReaded`.


### Read Invoice

```rust
pub struct ReadInvoiceCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Invoice.Read`
**Effects:** Emits `InvoiceReaded`.


### Read Invoice Setting

```rust
pub struct ReadInvoiceSettingCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `InvoiceSetting.Read`
**Effects:** Emits `InvoiceSettingReaded`.


### Read Ledger Report

```rust
pub struct ReadLedgerReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `LedgerReport.Read`
**Effects:** Emits `LedgerReportReaded`.


### Read Monthly Collection Report

```rust
pub struct ReadMonthlyCollectionReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `MonthlyCollectionReport.Read`
**Effects:** Emits `MonthlyCollectionReportReaded`.


### Read Outstanding Fees Report

```rust
pub struct ReadOutstandingFeesReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `OutstandingFeesReport.Read`
**Effects:** Emits `OutstandingFeesReportReaded`.


### Read Payment Method

```rust
pub struct ReadPaymentMethodCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `PaymentMethod.Read`
**Effects:** Emits `PaymentMethodReaded`.


### Read Payment Method Report

```rust
pub struct ReadPaymentMethodReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `PaymentMethodReport.Read`
**Effects:** Emits `PaymentMethodReportReaded`.


### Read Payroll

```rust
pub struct ReadPayrollCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Payroll.Read`
**Effects:** Emits `PayrollReaded`.


### Read Payroll Payment

```rust
pub struct ReadPayrollPaymentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `PayrollPayment.Read`
**Effects:** Emits `PayrollPaymentReaded`.


### Read Payroll Report

```rust
pub struct ReadPayrollReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `PayrollReport.Read`
**Effects:** Emits `PayrollReportReaded`.


### Read Product Purchase

```rust
pub struct ReadProductPurchaseCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ProductPurchase.Read`
**Effects:** Emits `ProductPurchaseReaded`.


### Read Profit Loss Report

```rust
pub struct ReadProfitLossReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ProfitLossReport.Read`
**Effects:** Emits `ProfitLossReportReaded`.


### Read Receipt Report

```rust
pub struct ReadReceiptReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ReceiptReport.Read`
**Effects:** Emits `ReceiptReportReaded`.


### Read Refund Report

```rust
pub struct ReadRefundReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `RefundReport.Read`
**Effects:** Emits `RefundReportReaded`.


### Read Transaction

```rust
pub struct ReadTransactionCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Transaction.Read`
**Effects:** Emits `TransactionReaded`.


### Read Trial Balance Report

```rust
pub struct ReadTrialBalanceReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `TrialBalanceReport.Read`
**Effects:** Emits `TrialBalanceReportReaded`.


### Read Wallet Balance Report

```rust
pub struct ReadWalletBalanceReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `WalletBalanceReport.Read`
**Effects:** Emits `WalletBalanceReportReaded`.


### Read Wallet

```rust
pub struct ReadWalletCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Wallet.Read`
**Effects:** Emits `WalletReaded`.


### Read Wallet Transaction

```rust
pub struct ReadWalletTransactionCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `WalletTransaction.Read`
**Effects:** Emits `WalletTransactionReaded`.


### Refund Payment

```rust
pub struct RefundPaymentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Payment.Refund`
**Effects:** Emits `PaymentRefunded`.


### Update Bank Account

```rust
pub struct UpdateBankAccountCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `BankAccount.Update`
**Effects:** Emits `BankAccountUpdateed`.


### Update Bank Slip

```rust
pub struct UpdateBankSlipCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `BankSlip.Update`
**Effects:** Emits `BankSlipUpdateed`.


### Update Direct Fees Installment

```rust
pub struct UpdateDirectFeesInstallmentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DirectFeesInstallment.Update`
**Effects:** Emits `DirectFeesInstallmentUpdateed`.


### Update Direct Fees Reminder

```rust
pub struct UpdateDirectFeesReminderCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DirectFeesReminder.Update`
**Effects:** Emits `DirectFeesReminderUpdateed`.


### Update Direct Fees Setting

```rust
pub struct UpdateDirectFeesSettingCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DirectFeesSetting.Update`
**Effects:** Emits `DirectFeesSettingUpdateed`.


### Update Expense

```rust
pub struct UpdateExpenseCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Expense.Update`
**Effects:** Emits `ExpenseUpdateed`.


### Update Expense Head

```rust
pub struct UpdateExpenseHeadCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ExpenseHead.Update`
**Effects:** Emits `ExpenseHeadUpdateed`.


### Update Fees Assign

```rust
pub struct UpdateFeesAssignCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesAssign.Update`
**Effects:** Emits `FeesAssignUpdateed`.


### Update Fees Discount

```rust
pub struct UpdateFeesDiscountCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesDiscount.Update`
**Effects:** Emits `FeesDiscountUpdateed`.


### Update Fees Group

```rust
pub struct UpdateFeesGroupCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesGroup.Update`
**Effects:** Emits `FeesGroupUpdateed`.


### Update Fees Installment

```rust
pub struct UpdateFeesInstallmentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesInstallment.Update`
**Effects:** Emits `FeesInstallmentUpdateed`.


### Update Fees Master

```rust
pub struct UpdateFeesMasterCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesMaster.Update`
**Effects:** Emits `FeesMasterUpdateed`.


### Update Fees Type

```rust
pub struct UpdateFeesTypeCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesType.Update`
**Effects:** Emits `FeesTypeUpdateed`.


### Update Income

```rust
pub struct UpdateIncomeCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Income.Update`
**Effects:** Emits `IncomeUpdateed`.


### Update Income Head

```rust
pub struct UpdateIncomeHeadCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `IncomeHead.Update`
**Effects:** Emits `IncomeHeadUpdateed`.


### Update Invoice

```rust
pub struct UpdateInvoiceCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Invoice.Update`
**Effects:** Emits `InvoiceUpdateed`.


### Update Payment Gateway

```rust
pub struct UpdatePaymentGatewayCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `PaymentGateway.Update`
**Effects:** Emits `PaymentGatewayUpdateed`.


### Update Payment Method

```rust
pub struct UpdatePaymentMethodCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `PaymentMethod.Update`
**Effects:** Emits `PaymentMethodUpdateed`.



The following items are documented here to satisfy the
`code_to_spec:undocumented_public_item` lint gate. They were
discovered after the main spec was written.

### Approve Bank Slip

```rust
pub struct ApproveBankSlipCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `BankSlip.Approve`
**Effects:** Emits `BankSlipApproveed`.


### Approve Expense

```rust
pub struct ApproveExpenseCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Expense.Approve`
**Effects:** Emits `ExpenseApproveed`.


### Approve Income

```rust
pub struct ApproveIncomeCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Income.Approve`
**Effects:** Emits `IncomeApproveed`.


### Approve Payroll Payment

```rust
pub struct ApprovePayrollPaymentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `PayrollPayment.Approve`
**Effects:** Emits `PayrollPaymentApproveed`.


### Cancel Invoice

```rust
pub struct CancelInvoiceCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Invoice.Cancel`
**Effects:** Emits `InvoiceCanceled`.


### Configure Due Fees Block Setting

```rust
pub struct ConfigureDueFeesBlockSettingCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DueFeesBlockSetting.Configure`
**Effects:** Emits `DueFeesBlockSettingConfigureed`.


### Configure Fees Group

```rust
pub struct ConfigureFeesGroupCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesGroup.Configure`
**Effects:** Emits `FeesGroupConfigureed`.


### Configure Fees Type

```rust
pub struct ConfigureFeesTypeCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesType.Configure`
**Effects:** Emits `FeesTypeConfigureed`.


### Create Amount Transfer

```rust
pub struct CreateAmountTransferCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `AmountTransfer.Create`
**Effects:** Emits `AmountTransferCreateed`.


### Create Direct Fees Installment Assign

```rust
pub struct CreateDirectFeesInstallmentAssignCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DirectFeesInstallmentAssign.Create`
**Effects:** Emits `DirectFeesInstallmentAssignCreateed`.


### Create Direct Fees Installment Child Payment

```rust
pub struct CreateDirectFeesInstallmentChildPaymentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DirectFeesInstallmentChildPayment.Create`
**Effects:** Emits `DirectFeesInstallmentChildPaymentCreateed`.


### Create Direct Fees Installment

```rust
pub struct CreateDirectFeesInstallmentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DirectFeesInstallment.Create`
**Effects:** Emits `DirectFeesInstallmentCreateed`.


### Create Direct Fees Reminder

```rust
pub struct CreateDirectFeesReminderCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DirectFeesReminder.Create`
**Effects:** Emits `DirectFeesReminderCreateed`.


### Create Direct Fees Setting

```rust
pub struct CreateDirectFeesSettingCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DirectFeesSetting.Create`
**Effects:** Emits `DirectFeesSettingCreateed`.


### Create Donor

```rust
pub struct CreateDonorCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Donor.Create`
**Effects:** Emits `DonorCreateed`.


### Create Expense Head

```rust
pub struct CreateExpenseHeadCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ExpenseHead.Create`
**Effects:** Emits `ExpenseHeadCreateed`.


### Create Fees Assign

```rust
pub struct CreateFeesAssignCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesAssign.Create`
**Effects:** Emits `FeesAssignCreateed`.


### Create Fees Assign Discount

```rust
pub struct CreateFeesAssignDiscountCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesAssignDiscount.Create`
**Effects:** Emits `FeesAssignDiscountCreateed`.


### Create Fees Installment Credit

```rust
pub struct CreateFeesInstallmentCreditCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesInstallmentCredit.Create`
**Effects:** Emits `FeesInstallmentCreditCreateed`.


### Create Fees Invoice Setting

```rust
pub struct CreateFeesInvoiceSettingCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesInvoiceSetting.Create`
**Effects:** Emits `FeesInvoiceSettingCreateed`.


### Create Fm Fees Group

```rust
pub struct CreateFmFeesGroupCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesGroup.Create`
**Effects:** Emits `FmFeesGroupCreateed`.


### Create Fm Fees Invoice Child

```rust
pub struct CreateFmFeesInvoiceChildCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesInvoiceChild.Create`
**Effects:** Emits `FmFeesInvoiceChildCreateed`.


### Create Fm Fees Invoice

```rust
pub struct CreateFmFeesInvoiceCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesInvoice.Create`
**Effects:** Emits `FmFeesInvoiceCreateed`.


### Create Fm Fees Invoice Setting

```rust
pub struct CreateFmFeesInvoiceSettingCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesInvoiceSetting.Create`
**Effects:** Emits `FmFeesInvoiceSettingCreateed`.


### Create Fm Fees Transaction Child

```rust
pub struct CreateFmFeesTransactionChildCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesTransactionChild.Create`
**Effects:** Emits `FmFeesTransactionChildCreateed`.


### Create Fm Fees Transaction

```rust
pub struct CreateFmFeesTransactionCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesTransaction.Create`
**Effects:** Emits `FmFeesTransactionCreateed`.


### Create Fm Fees Type

```rust
pub struct CreateFmFeesTypeCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesType.Create`
**Effects:** Emits `FmFeesTypeCreateed`.


### Create Fm Fees Weaver

```rust
pub struct CreateFmFeesWeaverCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesWeaver.Create`
**Effects:** Emits `FmFeesWeaverCreateed`.


### Create Income

```rust
pub struct CreateIncomeCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Income.Create`
**Effects:** Emits `IncomeCreateed`.


### Create Income Head

```rust
pub struct CreateIncomeHeadCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `IncomeHead.Create`
**Effects:** Emits `IncomeHeadCreateed`.


### Create Inventory Payment

```rust
pub struct CreateInventoryPaymentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `InventoryPayment.Create`
**Effects:** Emits `InventoryPaymentCreateed`.


### Create Payment Gateway

```rust
pub struct CreatePaymentGatewayCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `PaymentGateway.Create`
**Effects:** Emits `PaymentGatewayCreateed`.


### Create Product Purchase

```rust
pub struct CreateProductPurchaseCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ProductPurchase.Create`
**Effects:** Emits `ProductPurchaseCreateed`.


### Create Transaction

```rust
pub struct CreateTransactionCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Transaction.Create`
**Effects:** Emits `TransactionCreateed`.


### Delete Bank Account

```rust
pub struct DeleteBankAccountCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `BankAccount.Delete`
**Effects:** Emits `BankAccountDeleteed`.


### Delete Direct Fees Installment Assign

```rust
pub struct DeleteDirectFeesInstallmentAssignCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DirectFeesInstallmentAssign.Delete`
**Effects:** Emits `DirectFeesInstallmentAssignDeleteed`.


### Delete Direct Fees Installment

```rust
pub struct DeleteDirectFeesInstallmentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DirectFeesInstallment.Delete`
**Effects:** Emits `DirectFeesInstallmentDeleteed`.


### Delete Direct Fees Reminder

```rust
pub struct DeleteDirectFeesReminderCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DirectFeesReminder.Delete`
**Effects:** Emits `DirectFeesReminderDeleteed`.


### Delete Direct Fees Setting

```rust
pub struct DeleteDirectFeesSettingCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DirectFeesSetting.Delete`
**Effects:** Emits `DirectFeesSettingDeleteed`.


### Delete Expense

```rust
pub struct DeleteExpenseCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Expense.Delete`
**Effects:** Emits `ExpenseDeleteed`.


### Delete Expense Head

```rust
pub struct DeleteExpenseHeadCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ExpenseHead.Delete`
**Effects:** Emits `ExpenseHeadDeleteed`.


### Delete Fees Assign

```rust
pub struct DeleteFeesAssignCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesAssign.Delete`
**Effects:** Emits `FeesAssignDeleteed`.


### Delete Fees Discount

```rust
pub struct DeleteFeesDiscountCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesDiscount.Delete`
**Effects:** Emits `FeesDiscountDeleteed`.


### Delete Fees Group

```rust
pub struct DeleteFeesGroupCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesGroup.Delete`
**Effects:** Emits `FeesGroupDeleteed`.


### Delete Fees Installment

```rust
pub struct DeleteFeesInstallmentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesInstallment.Delete`
**Effects:** Emits `FeesInstallmentDeleteed`.


### Delete Fees Master

```rust
pub struct DeleteFeesMasterCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesMaster.Delete`
**Effects:** Emits `FeesMasterDeleteed`.


### Delete Fees Type

```rust
pub struct DeleteFeesTypeCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesType.Delete`
**Effects:** Emits `FeesTypeDeleteed`.


### Delete Income

```rust
pub struct DeleteIncomeCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Income.Delete`
**Effects:** Emits `IncomeDeleteed`.


### Delete Income Head

```rust
pub struct DeleteIncomeHeadCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `IncomeHead.Delete`
**Effects:** Emits `IncomeHeadDeleteed`.


### Delete Payment Gateway

```rust
pub struct DeletePaymentGatewayCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `PaymentGateway.Delete`
**Effects:** Emits `PaymentGatewayDeleteed`.


### Delete Payment Method

```rust
pub struct DeletePaymentMethodCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `PaymentMethod.Delete`
**Effects:** Emits `PaymentMethodDeleteed`.


### Generate Bank Slip

```rust
pub struct GenerateBankSlipCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `BankSlip.Generate`
**Effects:** Emits `BankSlipGenerateed`.


### Generate Invoice

```rust
pub struct GenerateInvoiceCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Invoice.Generate`
**Effects:** Emits `InvoiceGenerateed`.


### Pay Payroll

```rust
pub struct PayPayrollCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Payroll.Pay`
**Effects:** Emits `PayrollPayed`.


### Pay Payroll Payment

```rust
pub struct PayPayrollPaymentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `PayrollPayment.Pay`
**Effects:** Emits `PayrollPaymentPayed`.


### Read Amount Transfer

```rust
pub struct ReadAmountTransferCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `AmountTransfer.Read`
**Effects:** Emits `AmountTransferReaded`.


### Read Balance Sheet Report

```rust
pub struct ReadBalanceSheetReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `BalanceSheetReport.Read`
**Effects:** Emits `BalanceSheetReportReaded`.


### Read Bank Account

```rust
pub struct ReadBankAccountCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `BankAccount.Read`
**Effects:** Emits `BankAccountReaded`.


### Read Bank Slip

```rust
pub struct ReadBankSlipCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `BankSlip.Read`
**Effects:** Emits `BankSlipReaded`.


### Read Bank Statement

```rust
pub struct ReadBankStatementCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `BankStatement.Read`
**Effects:** Emits `BankStatementReaded`.


### Read Bank Statement Report

```rust
pub struct ReadBankStatementReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `BankStatementReport.Read`
**Effects:** Emits `BankStatementReportReaded`.


### Read Cash Flow Report

```rust
pub struct ReadCashFlowReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `CashFlowReport.Read`
**Effects:** Emits `CashFlowReportReaded`.


### Read Chart Of Account

```rust
pub struct ReadChartOfAccountCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ChartOfAccount.Read`
**Effects:** Emits `ChartOfAccountReaded`.


### Read Class Wise Collection Report

```rust
pub struct ReadClassWiseCollectionReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ClassWiseCollectionReport.Read`
**Effects:** Emits `ClassWiseCollectionReportReaded`.


### Read Daily Collection Report

```rust
pub struct ReadDailyCollectionReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DailyCollectionReport.Read`
**Effects:** Emits `DailyCollectionReportReaded`.


### Read Direct Fees Installment Child Payment

```rust
pub struct ReadDirectFeesInstallmentChildPaymentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DirectFeesInstallmentChildPayment.Read`
**Effects:** Emits `DirectFeesInstallmentChildPaymentReaded`.


### Read Direct Fees Installment

```rust
pub struct ReadDirectFeesInstallmentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DirectFeesInstallment.Read`
**Effects:** Emits `DirectFeesInstallmentReaded`.


### Read Donor

```rust
pub struct ReadDonorCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Donor.Read`
**Effects:** Emits `DonorReaded`.


### Read Due Fees Block

```rust
pub struct ReadDueFeesBlockCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DueFeesBlock.Read`
**Effects:** Emits `DueFeesBlockReaded`.


### Read Due Fees Report

```rust
pub struct ReadDueFeesReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DueFeesReport.Read`
**Effects:** Emits `DueFeesReportReaded`.


### Read Expense Report

```rust
pub struct ReadExpenseReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ExpenseReport.Read`
**Effects:** Emits `ExpenseReportReaded`.


### Read Fees Assign Discount

```rust
pub struct ReadFeesAssignDiscountCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesAssignDiscount.Read`
**Effects:** Emits `FeesAssignDiscountReaded`.


### Read Fees Carry Forward

```rust
pub struct ReadFeesCarryForwardCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesCarryForward.Read`
**Effects:** Emits `FeesCarryForwardReaded`.


### Read Fees Carry Forward Log

```rust
pub struct ReadFeesCarryForwardLogCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesCarryForwardLog.Read`
**Effects:** Emits `FeesCarryForwardLogReaded`.


### Read Fees Collection Report

```rust
pub struct ReadFeesCollectionReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesCollectionReport.Read`
**Effects:** Emits `FeesCollectionReportReaded`.


### Read Fees Discount

```rust
pub struct ReadFeesDiscountCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesDiscount.Read`
**Effects:** Emits `FeesDiscountReaded`.


### Read Fees Discount Report

```rust
pub struct ReadFeesDiscountReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesDiscountReport.Read`
**Effects:** Emits `FeesDiscountReportReaded`.


### Read Fees Group

```rust
pub struct ReadFeesGroupCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesGroup.Read`
**Effects:** Emits `FeesGroupReaded`.


### Read Fees Installment Credit

```rust
pub struct ReadFeesInstallmentCreditCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesInstallmentCredit.Read`
**Effects:** Emits `FeesInstallmentCreditReaded`.


### Read Fees Invoice Setting

```rust
pub struct ReadFeesInvoiceSettingCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesInvoiceSetting.Read`
**Effects:** Emits `FeesInvoiceSettingReaded`.


### Read Fees Master

```rust
pub struct ReadFeesMasterCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesMaster.Read`
**Effects:** Emits `FeesMasterReaded`.


### Read Fees Payment

```rust
pub struct ReadFeesPaymentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesPayment.Read`
**Effects:** Emits `FeesPaymentReaded`.


### Read Fees Payment Slip

```rust
pub struct ReadFeesPaymentSlipCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesPaymentSlip.Read`
**Effects:** Emits `FeesPaymentSlipReaded`.


### Read Fees Type

```rust
pub struct ReadFeesTypeCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesType.Read`
**Effects:** Emits `FeesTypeReaded`.


### Read Fm Fees Group

```rust
pub struct ReadFmFeesGroupCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesGroup.Read`
**Effects:** Emits `FmFeesGroupReaded`.


### Read Fm Fees Invoice Child

```rust
pub struct ReadFmFeesInvoiceChildCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesInvoiceChild.Read`
**Effects:** Emits `FmFeesInvoiceChildReaded`.


### Read Fm Fees Invoice

```rust
pub struct ReadFmFeesInvoiceCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesInvoice.Read`
**Effects:** Emits `FmFeesInvoiceReaded`.


### Read Fm Fees Invoice Setting

```rust
pub struct ReadFmFeesInvoiceSettingCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesInvoiceSetting.Read`
**Effects:** Emits `FmFeesInvoiceSettingReaded`.


### Read Fm Fees Transaction Child

```rust
pub struct ReadFmFeesTransactionChildCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesTransactionChild.Read`
**Effects:** Emits `FmFeesTransactionChildReaded`.


### Read Fm Fees Transaction

```rust
pub struct ReadFmFeesTransactionCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesTransaction.Read`
**Effects:** Emits `FmFeesTransactionReaded`.


### Read Fm Fees Type

```rust
pub struct ReadFmFeesTypeCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesType.Read`
**Effects:** Emits `FmFeesTypeReaded`.


### Read Fm Fees Weaver

```rust
pub struct ReadFmFeesWeaverCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FmFeesWeaver.Read`
**Effects:** Emits `FmFeesWeaverReaded`.


### Read Head Wise Expense Report

```rust
pub struct ReadHeadWiseExpenseReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `HeadWiseExpenseReport.Read`
**Effects:** Emits `HeadWiseExpenseReportReaded`.


### Read Head Wise Income Report

```rust
pub struct ReadHeadWiseIncomeReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `HeadWiseIncomeReport.Read`
**Effects:** Emits `HeadWiseIncomeReportReaded`.


### Read Income Report

```rust
pub struct ReadIncomeReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `IncomeReport.Read`
**Effects:** Emits `IncomeReportReaded`.


### Read Inventory Payment

```rust
pub struct ReadInventoryPaymentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `InventoryPayment.Read`
**Effects:** Emits `InventoryPaymentReaded`.


### Read Invoice

```rust
pub struct ReadInvoiceCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Invoice.Read`
**Effects:** Emits `InvoiceReaded`.


### Read Invoice Setting

```rust
pub struct ReadInvoiceSettingCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `InvoiceSetting.Read`
**Effects:** Emits `InvoiceSettingReaded`.


### Read Ledger Report

```rust
pub struct ReadLedgerReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `LedgerReport.Read`
**Effects:** Emits `LedgerReportReaded`.


### Read Monthly Collection Report

```rust
pub struct ReadMonthlyCollectionReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `MonthlyCollectionReport.Read`
**Effects:** Emits `MonthlyCollectionReportReaded`.


### Read Outstanding Fees Report

```rust
pub struct ReadOutstandingFeesReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `OutstandingFeesReport.Read`
**Effects:** Emits `OutstandingFeesReportReaded`.


### Read Payment Method

```rust
pub struct ReadPaymentMethodCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `PaymentMethod.Read`
**Effects:** Emits `PaymentMethodReaded`.


### Read Payment Method Report

```rust
pub struct ReadPaymentMethodReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `PaymentMethodReport.Read`
**Effects:** Emits `PaymentMethodReportReaded`.


### Read Payroll

```rust
pub struct ReadPayrollCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Payroll.Read`
**Effects:** Emits `PayrollReaded`.


### Read Payroll Payment

```rust
pub struct ReadPayrollPaymentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `PayrollPayment.Read`
**Effects:** Emits `PayrollPaymentReaded`.


### Read Payroll Report

```rust
pub struct ReadPayrollReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `PayrollReport.Read`
**Effects:** Emits `PayrollReportReaded`.


### Read Product Purchase

```rust
pub struct ReadProductPurchaseCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ProductPurchase.Read`
**Effects:** Emits `ProductPurchaseReaded`.


### Read Profit Loss Report

```rust
pub struct ReadProfitLossReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ProfitLossReport.Read`
**Effects:** Emits `ProfitLossReportReaded`.


### Read Receipt Report

```rust
pub struct ReadReceiptReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ReceiptReport.Read`
**Effects:** Emits `ReceiptReportReaded`.


### Read Refund Report

```rust
pub struct ReadRefundReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `RefundReport.Read`
**Effects:** Emits `RefundReportReaded`.


### Read Transaction

```rust
pub struct ReadTransactionCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Transaction.Read`
**Effects:** Emits `TransactionReaded`.


### Read Trial Balance Report

```rust
pub struct ReadTrialBalanceReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `TrialBalanceReport.Read`
**Effects:** Emits `TrialBalanceReportReaded`.


### Read Wallet Balance Report

```rust
pub struct ReadWalletBalanceReportCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `WalletBalanceReport.Read`
**Effects:** Emits `WalletBalanceReportReaded`.


### Read Wallet

```rust
pub struct ReadWalletCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Wallet.Read`
**Effects:** Emits `WalletReaded`.


### Read Wallet Transaction

```rust
pub struct ReadWalletTransactionCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `WalletTransaction.Read`
**Effects:** Emits `WalletTransactionReaded`.


### Refund Payment

```rust
pub struct RefundPaymentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Payment.Refund`
**Effects:** Emits `PaymentRefunded`.


### Update Bank Account

```rust
pub struct UpdateBankAccountCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `BankAccount.Update`
**Effects:** Emits `BankAccountUpdateed`.


### Update Bank Slip

```rust
pub struct UpdateBankSlipCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `BankSlip.Update`
**Effects:** Emits `BankSlipUpdateed`.


### Update Direct Fees Installment

```rust
pub struct UpdateDirectFeesInstallmentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DirectFeesInstallment.Update`
**Effects:** Emits `DirectFeesInstallmentUpdateed`.


### Update Direct Fees Reminder

```rust
pub struct UpdateDirectFeesReminderCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DirectFeesReminder.Update`
**Effects:** Emits `DirectFeesReminderUpdateed`.


### Update Direct Fees Setting

```rust
pub struct UpdateDirectFeesSettingCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DirectFeesSetting.Update`
**Effects:** Emits `DirectFeesSettingUpdateed`.


### Update Expense

```rust
pub struct UpdateExpenseCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Expense.Update`
**Effects:** Emits `ExpenseUpdateed`.


### Update Expense Head

```rust
pub struct UpdateExpenseHeadCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ExpenseHead.Update`
**Effects:** Emits `ExpenseHeadUpdateed`.


### Update Fees Assign

```rust
pub struct UpdateFeesAssignCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesAssign.Update`
**Effects:** Emits `FeesAssignUpdateed`.


### Update Fees Discount

```rust
pub struct UpdateFeesDiscountCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesDiscount.Update`
**Effects:** Emits `FeesDiscountUpdateed`.


### Update Fees Group

```rust
pub struct UpdateFeesGroupCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesGroup.Update`
**Effects:** Emits `FeesGroupUpdateed`.


### Update Fees Installment

```rust
pub struct UpdateFeesInstallmentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesInstallment.Update`
**Effects:** Emits `FeesInstallmentUpdateed`.


### Update Fees Master

```rust
pub struct UpdateFeesMasterCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesMaster.Update`
**Effects:** Emits `FeesMasterUpdateed`.


### Update Fees Type

```rust
pub struct UpdateFeesTypeCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `FeesType.Update`
**Effects:** Emits `FeesTypeUpdateed`.


### Update Income

```rust
pub struct UpdateIncomeCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Income.Update`
**Effects:** Emits `IncomeUpdateed`.


### Update Income Head

```rust
pub struct UpdateIncomeHeadCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `IncomeHead.Update`
**Effects:** Emits `IncomeHeadUpdateed`.


### Update Invoice

```rust
pub struct UpdateInvoiceCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `Invoice.Update`
**Effects:** Emits `InvoiceUpdateed`.


### Update Payment Gateway

```rust
pub struct UpdatePaymentGatewayCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `PaymentGateway.Update`
**Effects:** Emits `PaymentGatewayUpdateed`.


### Update Payment Method

```rust
pub struct UpdatePaymentMethodCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `PaymentMethod.Update`
**Effects:** Emits `PaymentMethodUpdateed`.

