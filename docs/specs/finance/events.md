# Finance Domain — Events

Domain events describe facts that have already happened. They are
immutable, append-only records used for cross-domain integration,
audit, and event sourcing.

All events implement:

```rust
pub trait DomainEvent: Serialize + DeserializeOwned + Send + Sync {
    const TYPE: &'static str;
    fn aggregate_id(&self) -> Uuid;
    fn school_id(&self) -> SchoolId;
    fn occurred_at(&self) -> Timestamp;
}
```

The event envelope wraps the event with metadata:

```rust
pub struct EventEnvelope<E> {
    pub event_id: EventId,
    pub event_type: &'static str,
    pub school_id: SchoolId,
    pub aggregate_id: Uuid,
    pub aggregate_type: &'static str,
    pub actor_id: UserId,
    pub correlation_id: CorrelationId,
    pub causation_id: Option<EventId>,
    pub occurred_at: Timestamp,
    pub payload: E,
}
```

## Fees Catalog

### FeesGroupCreated

```rust
pub struct FeesGroupCreated {
    pub fees_group_id: FeesGroupId,
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub due_date: NaiveDate,
}
```

- `FeesGroupUpdated { fees_group_id, changes }`
- `FeesGroupDeleted { fees_group_id }`

### FeesTypeCreated

```rust
pub struct FeesTypeCreated {
    pub fees_type_id: FeesTypeId,
    pub fees_group_id: FeesGroupId,
    pub name: String,
}
```

- `FeesTypeUpdated { fees_type_id, changes }`
- `FeesTypeDeleted { fees_type_id }`

### FeesMasterCreated

```rust
pub struct FeesMasterCreated {
    pub fees_master_id: FeesMasterId,
    pub fees_group_id: FeesGroupId,
    pub fees_type_id: FeesTypeId,
    pub class_id: ClassId,
    pub section_id: Option<SectionId>,
    pub amount: Amount,
    pub academic_id: AcademicYearId,
}
```

- `FeesMasterAmountUpdated { fees_master_id, from, to }`
- `FeesMasterDeleted { fees_master_id }`

## Assignment

### FeesAssignedToClass

```rust
pub struct FeesAssignedToClass {
    pub fees_master_id: FeesMasterId,
    pub class_id: ClassId,
    pub section_id: Option<SectionId>,
    pub academic_id: AcademicYearId,
    pub due_date: NaiveDate,
    pub fees_assign_ids: Vec<FeesAssignId>,
}
```

**Subscribers:**
- `communication` — schedule a "fees assigned" notice to guardians.
- `attendance` — none; informational only.

### FeesAssignedToStudent

```rust
pub struct FeesAssignedToStudent {
    pub fees_assign_id: FeesAssignId,
    pub fees_master_id: FeesMasterId,
    pub student_id: StudentId,
    pub record_id: StudentRecordId,
    pub fees_amount: Amount,
    pub applied_discount: DiscountAmount,
}
```

- `FeesAssignDiscountUpdated { fees_assign_id, from, to }`
- `FeesAssignClosed { fees_assign_id, reason, final_balance }`

## Discounts

### FeesDiscountAssigned

```rust
pub struct FeesDiscountAssigned {
    pub fees_assign_discount_id: FeesAssignDiscountId,
    pub student_id: StudentId,
    pub record_id: StudentRecordId,
    pub fees_discount_id: FeesDiscountId,
    pub fees_type_id: Option<FeesTypeId>,
    pub fees_group_id: Option<FeesGroupId>,
    pub applied_amount: DiscountAmount,
    pub unapplied_amount: DiscountAmount,
}
```

- `FeesDiscountApplied { fees_assign_discount_id, reference }`
- `FeesDiscountUnapplied { fees_assign_discount_id, amount }`
- `FeesAssignDiscountDeleted { fees_assign_discount_id }`
- `FeesDiscountCreated { fees_discount_id, name, amount, type }`
- `FeesDiscountUpdated { fees_discount_id, changes }`
- `FeesDiscountDeleted { fees_discount_id }`

## Installments

### FeesInstallmentCreated

```rust
pub struct FeesInstallmentCreated {
    pub fees_installment_id: FeesInstallmentId,
    pub fees_master_id: FeesMasterId,
    pub title: String,
    pub due_date: NaiveDate,
    pub amount: Amount,
    pub percentage: FeePercentage,
}
```

- `FeesInstallmentUpdated { fees_installment_id, changes }`
- `FeesInstallmentDeleted { fees_installment_id }`
- `FeesInstallmentAssigned { fees_installment_assign_id, fees_installment_id, student_id, record_id, amount, due_date }`
- `FeesInstallmentAssignmentUpdated { fees_installment_assign_id, changes }`
- `FeesInstallmentAssignmentCancelled { fees_installment_assign_id, reason }`

## Payment

### PaymentReceived

```rust
pub struct PaymentReceived {
    pub fees_payment_id: FeesPaymentId,
    pub assign_id: FeesAssignId,
    pub student_id: StudentId,
    pub record_id: StudentRecordId,
    pub amount: Amount,
    pub discount_amount: DiscountAmount,
    pub fine_amount: FineAmount,
    pub service_charge: Option<ServiceCharge>,
    pub payment_method_id: PaymentMethodId,
    pub bank_id: Option<BankAccountId>,
    pub slip: Option<SlipReference>,
    pub transaction_id: Option<TransactionId>,
    pub payment_date: NaiveDate,
    pub note: Option<String>,
}
```

**Subscribers:**
- `communication` — send a receipt notification.
- `hr` (if the staff is the payer) — log a wallet credit.
- `assessment` (for exam fees) — unlock the related exam registration.

### PaymentReversed

```rust
pub struct PaymentReversed {
    pub fees_payment_id: FeesPaymentId,
    pub reason: String,
    pub reversal_date: NaiveDate,
}
```

## Invoice

### InvoiceNumberingConfigured

```rust
pub struct InvoiceNumberingConfigured {
    pub fees_invoice_id: FeesInvoiceId,
    pub prefix: InvoicePrefix,
    pub start_form: InvoiceStartForm,
}
```

- `InvoiceCounterIncremented { fees_invoice_id, next_number }`
- `FeesInvoiceSettingConfigured { fees_invoice_setting_id, academic_id, layout }`
- `InvoiceSettingConfigured { invoice_setting_id, academic_id, layout }`
- `FmFeesInvoiceSettingConfigured { fm_fees_invoice_setting_id, layout }`

## FM Invoice Scheme

### FmFeesInvoiceGenerated

```rust
pub struct FmFeesInvoiceGenerated {
    pub fm_fees_invoice_id: FmFeesInvoiceId,
    pub invoice_id: InvoiceNumber,
    pub student_id: StudentId,
    pub record_id: StudentRecordId,
    pub class_id: ClassId,
    pub type_: FmInvoiceType,
    pub create_date: NaiveDate,
    pub due_date: NaiveDate,
    pub total: Amount,
}
```

**Subscribers:**
- `communication` — invoice notification.

- `FmFeesInvoiceStatusUpdated { fm_fees_invoice_id, status }`
- `FmFeesInvoiceCancelled { fm_fees_invoice_id, reason }`
- `FmFeesInvoiceLineAdded { fm_fees_invoice_id, line_id, fees_type, amount }`
- `FmFeesInvoiceLineUpdated { fm_fees_invoice_id, line_id, changes }`
- `FmFeesInvoiceLineRemoved { fm_fees_invoice_id, line_id }`
- `FmFeesTransactionRecorded { fm_fees_transaction_id, fm_fees_invoice_id, payment_method, total_paid_amount, add_wallet_money }`
- `FmFeesTransactionReversed { fm_fees_transaction_id, reason }`
- `FmFeesTransactionLineAdded { fm_fees_transaction_id, line_id, fees_type, paid_amount }`
- `FmFeesWeaverApplied { fm_fees_weaver_id, fm_fees_invoice_id, fees_type, amount, note }`
- `FmFeesWeaverReversed { fm_fees_weaver_id, reason }`
- `FmFeesGroupCreated / Updated / Deleted { fm_fees_group_id, ... }`
- `FmFeesTypeCreated / Updated / Deleted { fm_fees_type_id, ... }`

## Direct Fees

### DirectFeesInstallmentCreated

```rust
pub struct DirectFeesInstallmentCreated {
    pub direct_fees_installment_id: DirectFeesInstallmentId,
    pub fees_master_id: FeesMasterId,
    pub title: String,
    pub due_date: NaiveDate,
    pub amount: Amount,
    pub percentage: FeePercentage,
}
```

- `DirectFeesInstallmentUpdated { direct_fees_installment_id, changes }`
- `DirectFeesInstallmentDeleted { direct_fees_installment_id }`
- `DirectFeesInstallmentAssigned { direct_fees_installment_assign_id, student_id, record_id, amount, due_date }`
- `DirectFeesInstallmentPaid { direct_fees_installment_assign_id, amount, payment_method, balance }`
- `DirectFeesInstallmentCancelled { direct_fees_installment_assign_id, reason }`
- `DirectInstallmentPaymentRecorded { child_payment_id, parent_assign_id, amount, balance, payment_date }`

## Carry Forward

### FeesCarriedForward

```rust
pub struct FeesCarriedForward {
    pub fees_carry_forward_id: FeesCarryForwardId,
    pub student_id: StudentId,
    pub from_academic_id: AcademicYearId,
    pub to_academic_id: AcademicYearId,
    pub balance: Amount,
    pub balance_type: BalanceType,
    pub due_date: Option<NaiveDate>,
    pub notes: Option<String>,
}
```

**Subscribers:**
- `communication` — "balance carried forward" notice.
- `attendance` — none; financial event only.

- `FeesCarryForwardClosed { fees_carry_forward_id, closed_at }`
- `FeesCarryForwardLogged { fees_carry_forward_log_id, student_record_id, amount, type }`
- `FeesCarryForwardConfigured { fees_carry_forward_setting_id, fees_due_days }`

## Login Prevention

### DueFeesLoginPrevented

```rust
pub struct DueFeesLoginPrevented {
    pub due_fees_login_prevent_id: DueFeesLoginPreventId,
    pub user_id: UserId,
    pub role_id: Option<RoleId>,
    pub reason: PreventReason,
}
```

**Subscribers:**
- `rbac` — block login at the authentication port.

- `DueFeesLoginRestored { due_fees_login_prevent_id, user_id }`
- `DirectFeesConfigured { direct_fees_setting_id }`
- `FeesReminderConfigured / Updated / Deleted { direct_fees_reminder_id, due_date_before, notification_types }`

## Bank

### BankAccountOpened

```rust
pub struct BankAccountOpened {
    pub bank_account_id: BankAccountId,
    pub bank_name: String,
    pub account_number: BankAccountNumber,
    pub account_type: AccountType,
    pub opening_balance: Amount,
}
```

- `BankAccountUpdated { bank_account_id, changes }`
- `BankAccountClosed { bank_account_id }`
- `BankStatementRecorded { bank_statement_id, bank_id, amount, statement_type, after_balance, payment_method, reference_id }`
- `BankStatementReversed { bank_statement_id, reason }`
- `FundsTransferred { amount_transfer_id, from_bank_id, to_bank_id, amount }`

### BankPaymentSlipGenerated

```rust
pub struct BankPaymentSlipGenerated {
    pub bank_payment_slip_id: BankPaymentSlipId,
    pub student_id: StudentId,
    pub assign_id: Option<FeesAssignId>,
    pub installment_id: Option<FeesInstallmentAssignId>,
    pub amount: Amount,
    pub payment_mode: BankMode,
    pub date: NaiveDate,
    pub slip: Option<SlipReference>,
}
```

### BankPaymentApproved

```rust
pub struct BankPaymentApproved {
    pub bank_payment_slip_id: BankPaymentSlipId,
    pub fees_payment_id: FeesPaymentId,
    pub bank_statement_id: BankStatementId,
}
```

**Subscribers:**
- `communication` — receipt notification to the guardian.

- `BankPaymentRejected { bank_payment_slip_id, reason }`

## Expense & Income

### ExpenseRecorded

```rust
pub struct ExpenseRecorded {
    pub expense_id: ExpenseId,
    pub name: String,
    pub amount: Amount,
    pub expense_head_id: ExpenseHeadId,
    pub account_id: BankAccountId,
    pub payment_method_id: PaymentMethodId,
    pub date: NaiveDate,
    pub file: Option<FileReference>,
    pub description: Option<String>,
    pub item_receive_id: Option<u64>,
    pub inventory_id: Option<u64>,
    pub payroll_payment_id: Option<PayrollPaymentId>,
}
```

- `ExpenseUpdated { expense_id, changes }`
- `ExpenseDeleted { expense_id, reason }`
- `ExpenseHeadCreated / Updated / Deleted { expense_head_id, ... }`

### IncomeRecorded

```rust
pub struct IncomeRecorded {
    pub income_id: IncomeId,
    pub name: String,
    pub amount: Amount,
    pub income_head_id: IncomeHeadId,
    pub account_id: BankAccountId,
    pub payment_method_id: PaymentMethodId,
    pub date: NaiveDate,
    pub file: Option<FileReference>,
    pub description: Option<String>,
    pub item_sell_id: Option<u64>,
    pub fees_collection_id: Option<u64>,
    pub inventory_id: Option<u64>,
    pub installment_payment_id: Option<FeesInstallmentAssignId>,
}
```

- `IncomeUpdated { income_id, changes }`
- `IncomeDeleted { income_id, reason }`
- `IncomeHeadCreated / Updated / Deleted { income_head_id, ... }`

## Donors

- `DonorRegistered { donor_id, name, email, mobile }`
- `DonorUpdated { donor_id, changes }`
- `DonorDeleted { donor_id }`

## Wallet

### WalletCredited

```rust
pub struct WalletCredited {
    pub wallet_transaction_id: WalletTransactionId,
    pub user_id: UserId,
    pub amount: Amount,
    pub wallet_type: WalletTxType,
    pub payment_method_id: Option<PaymentMethodId>,
    pub bank_id: Option<BankAccountId>,
}
```

- `WalletRefundRequested { wallet_transaction_id, user_id, amount, reason }`
- `WalletDebited { wallet_transaction_id, user_id, amount, wallet_type, reference_id }`
- `WalletTransactionApproved { wallet_transaction_id, approver_id }`
- `WalletTransactionRejected { wallet_transaction_id, reject_note, rejecter_id }`

## Transaction

- `TransactionRecorded { transaction_id, title, type, amount, morphable_type, morphable_id, payment_method }`
- `TransactionReversed { transaction_id, reason }`

## Payroll

### PayrollGenerated

```rust
pub struct PayrollGenerated {
    pub payroll_generate_id: PayrollGenerateId,
    pub staff_id: StaffId,
    pub pay_period: PayPeriod,
    pub basic_salary: BasicSalary,
    pub gross_salary: GrossSalary,
    pub total_earning: TotalEarning,
    pub total_deduction: TotalDeduction,
    pub tax: Tax,
    pub net_salary: NetSalary,
    pub bank_id: Option<BankAccountId>,
}
```

- `PayrollApproved { payroll_generate_id, approver_id, approved_at }`
- `PayrollPaid { payroll_generate_id, paid_amount, payment_date }`
- `PayrollPaymentRecorded { payroll_payment_id, payroll_generate_id, amount, payment_method, bank_id, payment_date, note }`
- `PayrollEarningAdded { payroll_earn_deduc_id, payroll_generate_id, type_name, amount }`
- `PayrollDeductionAdded { payroll_earn_deduc_id, payroll_generate_id, type_name, amount }`
- `PayrollEarnDeducUpdated { payroll_earn_deduc_id, changes }`
- `PayrollEarnDeducDeleted { payroll_earn_deduc_id }`
- `SalaryTemplateCreated { salary_template_id, grade }`
- `SalaryTemplateUpdated { salary_template_id, changes }`
- `SalaryTemplateDeleted { salary_template_id }`
- `HourlyRateSet { grade, rate }`

## Inventory & Product

- `InventoryPaymentRecorded { inventory_payment_id, amount, payment_type, payment_method, bank_id, reference_no }`
- `ProductPurchaseRecorded { product_purchase_id, user_id, staff_id, price, paid, due, package }`
- `ProductPaymentRecorded { product_purchase_id, amount, payment_method, bank_id, payment_date }`

## Amount Transfer

- `FundsTransferred { amount_transfer_id, from_bank_id, to_bank_id, amount, transfer_date, note }`

## Settings

- `InvoiceSettingConfigured { invoice_setting_id, academic_id }`
- `FeesInvoiceSettingConfigured { fees_invoice_setting_id, academic_id }`
- `FmFeesInvoiceSettingConfigured { fm_fees_invoice_setting_id, layout }`
- `PaymentGatewayConfigured { payment_gateway_setting_id, gateway_name, gateway_mode }`
- `PaymentGatewayUpdated { payment_gateway_setting_id, changes }`
- `PaymentGatewayDisabled { payment_gateway_setting_id, disabled_at }`
- `PaymentMethodCreated / Updated / Deleted { payment_method_id, method, type }`
- `FeesAttachedToQuestionBank { question_bank_fee_id, question_bank_id, fees_type_id }`
- `FeesDetachedFromQuestionBank { question_bank_fee_id }`
- `ChartOfAccountCreated / Updated / Deleted { chart_of_account_id, name, account_type }`
- `DirectFeesConfigured { direct_fees_setting_id }`
- `FeesReminderConfigured / Updated / Deleted { direct_fees_reminder_id, due_date_before, notification_types }`

## Installment Credit

- `FeesInstallmentCreditAdded { fees_installment_credit_id, student_id, amount }`
- `FeesInstallmentCreditConsumed { fees_installment_credit_id, installment_assign_id, applied_amount }`
- `FeesInstallmentCreditCancelled { fees_installment_credit_id, reason }`
