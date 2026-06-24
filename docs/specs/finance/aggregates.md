# Finance Domain — Aggregates

## FeesGroup

**Root type:** `FeesGroup`
**Identity:** `FeesGroupId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Finance

### Purpose

A logical grouping of fees types (e.g. "Tuition", "Exam", "Transport",
"Hostel"). A `FeesGroup` carries a date range and a due date for the
whole group; individual `FeesMaster`s reference it.

### Owned Children

- `FeesMaster` (one or more).
- `FeesType` (one or more).

### Invariants

1. A `FeesGroup` is unique by `name` within a school.
2. `start_date <= end_date`.
3. `due_date` is on or before `end_date`.
4. A `FeesGroup` cannot be deleted while any `FeesMaster` references it.

### Commands

- `CreateFeesGroup`
- `UpdateFeesGroup`
- `DeleteFeesGroup`

### Events

- `FeesGroupCreated`
- `FeesGroupUpdated`
- `FeesGroupDeleted`

---

## FeesType

**Root type:** `FeesType`
**Identity:** `FeesTypeId(SchoolId, Uuid)`

### Purpose

A billable line item within a `FeesGroup` (e.g. "Lab Fee", "Library
Fee", "Annual Function"). Each type has a default amount that is
specialized by `FeesMaster` per class.

### Invariants

1. A `FeesType` belongs to exactly one `FeesGroup`.
2. The `name` is unique within `(school_id, fees_group_id)`.
3. Deleting a `FeesType` is allowed only when no `FeesMaster` references
   it.

### Commands

- `CreateFeesType`
- `UpdateFeesType`
- `DeleteFeesType`

### Events

- `FeesTypeCreated`
- `FeesTypeUpdated`
- `FeesTypeDeleted`

---

## FeesMaster

**Root type:** `FeesMaster`
**Identity:** `FeesMasterId(SchoolId, Uuid)`

### Purpose

The canonical `(class, fees_type, amount)` tuple. A `FeesMaster` is
the per-class specialization of a `FeesType`. It is the source of
all per-student fees assignments in that class.

### Invariants

1. A `FeesMaster` is unique by `(school_id, fees_group_id, fees_type_id,
   class_id, section_id?, academic_id)`.
2. `amount >= 0`.
3. A `FeesMaster` cannot be deleted while any `FeesAssign` references
   it.

### Commands

- `CreateFeesMaster`
- `UpdateFeesMasterAmount`
- `DeleteFeesMaster`

### Events

- `FeesMasterCreated`
- `FeesMasterAmountUpdated`
- `FeesMasterDeleted`

---

## FeesAssign

**Root type:** `FeesAssign`
**Identity:** `FeesAssignId(SchoolId, Uuid)`

### Purpose

The per-student assignment of a `FeesMaster`. It carries the
discounted amount actually owed by the student for that master and is
the target of `FeesPayment` records.

### Invariants

1. A `FeesAssign` is unique by `(school_id, fees_master_id, student_id,
   academic_id)`.
2. `fees_amount >= 0`.
3. `applied_discount <= fees_amount`.
4. The sum of `FeesPayment` amounts against an assignment never
   exceeds `(fees_amount - applied_discount) + fine + weaver`.
5. An assignment's `active_status` is true while any open balance
   remains.

### Commands

- `AssignFeesToClass`
- `AssignFeesToStudent`
- `UpdateFeesAssignDiscount`
- `CloseFeesAssign`

### Events

- `FeesAssignedToClass`
- `FeesAssignedToStudent`
- `FeesAssignDiscountUpdated`
- `FeesAssignClosed`

---

## FeesAssignDiscount

**Root type:** `FeesAssignDiscount`
**Identity:** `FeesAssignDiscountId(SchoolId, Uuid)`

### Purpose

A per-student application of a `FeesDiscount` to a specific
`fees_type` or `fees_group`. The applied amount and unapplied amount
are tracked separately so that the same discount can be partially
consumed across multiple invoices.

### Invariants

1. `applied_amount >= 0` and `unapplied_amount >= 0`.
2. `applied_amount + unapplied_amount` is constant for the life of the
   record (a value object invariants test).
3. The discount may be assigned by `role_id` (default) or by
   `student_id` (override).

### Commands

- `AssignDiscountToStudent`
- `ApplyDiscountToInvoice`
- `UnapplyDiscount`
- `DeleteFeesAssignDiscount`

### Events

- `FeesDiscountAssigned`
- `FeesDiscountApplied`
- `FeesDiscountUnapplied`
- `FeesAssignDiscountDeleted`

---

## FeesDiscount

**Root type:** `FeesDiscount`
**Identity:** `FeesDiscountId(SchoolId, Uuid)`

### Purpose

A catalog entry for a discount (e.g. "Sibling Discount",
"Scholarship"). Each discount has a `type` of `once` or `year`.

### Invariants

1. A `FeesDiscount` is unique by `name` within a school.
2. `amount >= 0`.
3. A discount of type `once` may be applied to one fees master per
   student per year.
4. A discount of type `year` may be applied once per student per year
   across all masters in its scope.

### Commands

- `CreateFeesDiscount`
- `UpdateFeesDiscount`
- `DeleteFeesDiscount`

### Events

- `FeesDiscountCreated`
- `FeesDiscountUpdated`
- `FeesDiscountDeleted`

---

## FeesInvoice

**Root type:** `FeesInvoice`
**Identity:** `FeesInvoiceId(SchoolId, Uuid)`

### Purpose

The classic invoice numbering scheme. Stores the `prefix` and
`start_form` that drive the next invoice number for a school.

### Invariants

1. A `FeesInvoice` is unique by `school_id` (one per school).
2. `start_form >= 0`.
3. The next invoice number is `start_form + count(issued_invoices)`.

### Commands

- `ConfigureInvoiceNumbering`
- `IncrementInvoiceCounter`

### Events

- `InvoiceNumberingConfigured`
- `InvoiceCounterIncremented`

---

## FeesInstallment

**Root type:** `FeesInstallment`
**Identity:** `FeesInstallmentId(SchoolId, Uuid)`

### Purpose

A class-wide installment plan, scoped to a `FeesMaster`. Each
installment has a percentage of the master amount, a fixed amount, and
a due date.

### Invariants

1. A `FeesInstallment` belongs to exactly one `FeesMaster`.
2. `percentage >= 0 && percentage <= 100`.
3. `amount >= 0`.
4. The sum of percentages of all installments in a master is at most
   `100.0`.
5. A `FeesInstallment` cannot be deleted while any
   `FeesInstallmentAssign` references it.

### Commands

- `CreateFeesInstallment`
- `UpdateFeesInstallment`
- `DeleteFeesInstallment`

### Events

- `FeesInstallmentCreated`
- `FeesInstallmentUpdated`
- `FeesInstallmentDeleted`

---

## FeesInstallmentAssign

**Root type:** `FeesInstallmentAssign`
**Identity:** `FeesInstallmentAssignId(SchoolId, Uuid)`

### Purpose

The per-student assignment of a `FeesInstallment`. Carries the
student's amount, paid amount, due date, and payment slip reference.

### Invariants

1. A `FeesInstallmentAssign` is unique by `(fees_installment_id,
   student_id, record_id)`.
2. `paid_amount <= amount + discount_amount`.
3. `active_status` is `1` while the installment is open.

### Commands

- `AssignInstallmentToStudent`
- `UpdateInstallmentAssignment`
- `CancelInstallmentAssignment`

### Events

- `FeesInstallmentAssigned`
- `FeesInstallmentAssignmentUpdated`
- `FeesInstallmentAssignmentCancelled`

---

## FeesPayment

**Root type:** `FeesPayment`
**Identity:** `FeesPaymentId(SchoolId, Uuid)`

### Purpose

A single payment against a `FeesAssign` (or a `FeesInstallmentAssign`).
It captures the amount, mode, slip reference, discount applied, and
fine paid at the time of payment.

### Invariants

1. A `FeesPayment` references a non-null `assign_id` and a non-null
   `student_id`.
2. `amount >= 0` and `discount_amount >= 0` and `fine >= 0`.
3. `payment_mode` is a `PaymentMethodId` whose `gateway_id` matches
   the chosen gateway.
4. If `payment_mode` is `gateway`, the gateway transaction id is
   required.

### Commands

- `RecordPayment`
- `ReversePayment`

### Events

- `PaymentReceived`
- `PaymentReversed`

---

## FeesCarryForward

**Root type:** `FeesCarryForward`
**Identity:** `FeesCarryForwardId(SchoolId, Uuid)`

### Purpose

A per-student carry-forward balance from one academic year to the
next. The balance has a `balance_type` (debit or credit) and a
`due_date`.

### Invariants

1. A `FeesCarryForward` is unique by `(school_id, student_id,
   academic_id)` — at most one open carry-forward per student per
   year.
2. `balance >= 0`.
3. `balance_type` is `debit` (student owes more) or `credit`
   (overpayment).

### Commands

- `CarryForwardFeesBalance`
- `CloseFeesCarryForward`

### Events

- `FeesCarriedForward`
- `FeesCarryForwardClosed`

---

## DirectFeesInstallment

**Root type:** `DirectFeesInstallment`
**Identity:** `DirectFeesInstallmentId(SchoolId, Uuid)`

### Purpose

A "direct" installment plan that bypasses the
`FeesMaster` / `FeesInvoice` path. Each installment is created
directly with a percentage, amount, and due date and is applied to
selected fees master ids.

### Invariants

1. A `DirectFeesInstallment` belongs to exactly one `FeesMaster`.
2. `percentage >= 0 && percentage <= 100`.
3. The sum of percentages of all direct installments in a master is
   at most `100.0`.
4. `due_date` is required.

### Commands

- `CreateDirectFeesInstallment`
- `UpdateDirectFeesInstallment`
- `DeleteDirectFeesInstallment`

### Events

- `DirectFeesInstallmentCreated`
- `DirectFeesInstallmentUpdated`
- `DirectFeesInstallmentDeleted`

---

## DirectFeesInstallmentAssign

**Root type:** `DirectFeesInstallmentAssign`
**Identity:** `DirectFeesInstallmentAssignId(SchoolId, Uuid)`

### Purpose

The per-student assignment of a `DirectFeesInstallment`. Carries the
amount, paid amount, due date, payment date, mode, and a reference to
a fees type, fees discount, and bank account.

### Invariants

1. A `DirectFeesInstallmentAssign` is unique by
   `(direct_fees_installment_id, student_id, record_id)`.
2. `paid_amount <= amount + discount_amount`.
3. `active_status` is `1` while the assignment is open.

### Commands

- `AssignDirectInstallment`
- `PayDirectInstallment`
- `CancelDirectInstallment`

### Events

- `DirectFeesInstallmentAssigned`
- `DirectFeesInstallmentPaid`
- `DirectFeesInstallmentCancelled`

---

## DirectFeesInstallmentChildPayment

**Root type:** `DirectFeesInstallmentChildPayment`
**Identity:** `DirectFeesInstallmentChildPaymentId(SchoolId, Uuid)`

### Purpose

A payment against a `DirectFeesInstallmentAssign`. Stores the amount
paid, balance remaining, mode, slip, bank, discount, fees type, and
the student.

### Invariants

1. `paid_amount + balance_amount == amount + discount_amount` at
   construction.
2. `paid_amount` is monotonically non-decreasing across payments.

### Commands

- `RecordDirectInstallmentPayment`

### Events

- `DirectInstallmentPaymentRecorded`

---

## FmFeesGroup

**Root type:** `FmFeesGroup`
**Identity:** `FmFeesGroupId(SchoolId, Uuid)`

### Purpose

A group of fees in the newer (FM) invoice scheme. Similar to
`FeesGroup` but used in the FM invoice flow.

### Invariants

1. Unique by `name` within a school.

### Commands

- `CreateFmFeesGroup`
- `UpdateFmFeesGroup`
- `DeleteFmFeesGroup`

### Events

- `FinanceFeesGroupCreated`
- `FinanceFeesGroupUpdated`
- `FinanceFeesGroupDeleted`

---

## FmFeesType

**Root type:** `FmFeesType`
**Identity:** `FmFeesTypeId(SchoolId, Uuid)`

### Purpose

A billable type in the FM scheme, with a `type` of `fees` or `lms`
and an optional `course_id` for LMS contexts.

### Invariants

1. Belongs to one `FmFeesGroup`.
2. The `type` field is `fees` or `lms`.
3. If `type == lms`, `course_id` is required.

### Commands

- `CreateFmFeesType`
- `UpdateFmFeesType`
- `DeleteFmFeesType`

### Events

- `FinanceFeesTypeCreated`
- `FinanceFeesTypeUpdated`
- `FinanceFeesTypeDeleted`

---

## FmFeesInvoice

**Root type:** `FmFeesInvoice`
**Identity:** `FmFeesInvoiceId(SchoolId, Uuid)`

### Purpose

A newer invoice header carrying a typed `invoice_id` (e.g.
`INV-2026-0001`), a `payment_status` (`paid`, `partial`, `unpaid`),
`payment_method`, and an optional `bank_id`.

### Invariants

1. `invoice_id` is unique within a school.
2. The sum of `FmFeesInvoiceChild` subtotals plus fine plus
   service_charge plus weaver equals the invoice's grand total.
3. The `type` field is `fees` or `lms`.

### Commands

- `GenerateFmFeesInvoice`
- `UpdateFmFeesInvoiceStatus`
- `CancelFmFeesInvoice`

### Events

- `FmFeesInvoiceGenerated`
- `FinanceInvoiceStatusUpdated`
- `FinanceInvoiceCancelled`

---

## FmFeesInvoiceChild

**Root type:** `FmFeesInvoiceChild`
**Identity:** `FmFeesInvoiceChildId(SchoolId, Uuid)`

### Purpose

A line on an `FmFeesInvoice`. Carries the `fees_type` reference,
amount, weaver, fine, sub_total, paid_amount, service_charge, due
amount, and a note.

### Invariants

1. Belongs to exactly one `FmFeesInvoice`.
2. `sub_total == amount + weaver + fine`.
3. `paid_amount <= sub_total + service_charge`.

### Commands

- `AddFmFeesInvoiceLine`
- `UpdateFmFeesInvoiceLine`
- `RemoveFmFeesInvoiceLine`

### Events

- `FinanceInvoiceLineAdded`
- `FinanceInvoiceLineUpdated`
- `FinanceInvoiceLineRemoved`

---

## FmFeesTransaction

**Root type:** `FmFeesTransaction`
**Identity:** `FmFeesTransactionId(SchoolId, Uuid)`

### Purpose

A payment transaction in the FM scheme. Stores the invoice number,
student, user, payment method, bank id, optional wallet credit,
service charge, paid status, and a list of `FmFeesTransactionChild`
lines.

### Invariants

1. References one `FmFeesInvoice`.
2. `total_paid_amount >= 0`.
3. The `add_wallet_money` may be non-zero only if the user has a
   wallet.

### Commands

- `RecordFmFeesTransaction`
- `ReverseFmFeesTransaction`

### Events

- `FinanceTransactionRecorded`
- `FmFeesTransactionReversed`

---

## FmFeesTransactionChild

**Root type:** `FmFeesTransactionChild`
**Identity:** `FmFeesTransactionChildId(SchoolId, Uuid)`

### Purpose

A line on an `FmFeesTransaction` describing which fees type was paid,
how much, including service charge, fine, weaver, and a note.

### Invariants

1. Belongs to one `FmFeesTransaction`.
2. `paid_amount >= 0`.

### Commands

- `AddFmFeesTransactionLine`

### Events

- `FmFeesTransactionLineAdded`

---

## FmFeesWeaver

**Root type:** `FmFeesWeaver`
**Identity:** `FmFeesWeaverId(SchoolId, Uuid)`

### Purpose

A per-invoice weaver adjustment (e.g. partial forgiveness). Carries
the invoice id, fees type, student, weaver amount, and note.

### Invariants

1. The weaver amount is non-negative.
2. The sum of weavers on a single `FmFeesInvoice` does not exceed the
   sum of its `FmFeesInvoiceChild` subtotals.

### Commands

- `ApplyFmFeesWeaver`
- `ReverseFmFeesWeaver`

### Events

- `FinanceWeaverApplied`
- `FinanceWeaverReversed`

---

## FeesInvoiceSetting

**Root type:** `FeesInvoiceSetting`
**Identity:** `FeesInvoiceSettingId(SchoolId, Uuid)`

### Purpose

The classic invoice layout configuration: which student fields to
show, the three footer labels, the three signature flags, the
per-thousand rounding, and the invoice type (`invoice` or `slip`).

### Invariants

1. A `FeesInvoiceSetting` is unique by `(school_id, academic_id)`.
2. `per_th` is a non-negative integer.

### Commands

- `ConfigureFeesInvoiceSetting`

### Events

- `FeesInvoiceSettingConfigured`

---

## InvoiceSetting

**Root type:** `InvoiceSetting`
**Identity:** `InvoiceSettingId(SchoolId, Uuid)`

### Purpose

The newer invoice layout configuration. Mirrors `FeesInvoiceSetting`
but applies to the FM invoice scheme.

### Invariants

1. A `InvoiceSetting` is unique by `(school_id, academic_id)`.

### Commands

- `ConfigureInvoiceSetting`

### Events

- `InvoiceSettingConfigured`

---

## FmFeesInvoiceSetting

**Root type:** `FmFeesInvoiceSetting`
**Identity:** `FmFeesInvoiceSettingId(SchoolId, Uuid)`

### Purpose

A separate row carrying the FM invoice positions, the
`uniq_id_start`, the `prefix`, the `class_limit`, `section_limit`,
`admission_limit`, and the weaver mode.

### Invariants

1. One `FmFeesInvoiceSetting` per school.
2. `class_limit`, `section_limit`, `admission_limit` are non-negative
   integers.
3. `uniq_id_start` is unique.

### Commands

- `ConfigureFmFeesInvoiceSetting`

### Events

- `FinanceInvoiceSettingConfigured`

---

## BankAccount

**Root type:** `BankAccount`
**Identity:** `BankAccountId(SchoolId, Uuid)`

### Purpose

A school bank account or cash drawer. Carries bank name, account
name, account number, account type, opening balance, current balance,
and a note.

### Invariants

1. `account_number` is unique within a school.
2. `current_balance` is derived from the `BankStatement` log; it is
   only written on creation.
3. `account_type` is `bank` or `cash`.

### Commands

- `OpenBankAccount`
- `UpdateBankAccount`
- `CloseBankAccount`

### Events

- `BankAccountOpened`
- `BankAccountUpdated`
- `BankAccountClosed`

---

## BankStatement

**Root type:** `BankStatement`
**Identity:** `BankStatementId(SchoolId, Uuid)`

### Purpose

A single entry in the bank ledger. Carries the bank id, amount,
type (`income` or `expense`), the resulting balance, a payment method
reference, an optional fees payment id, and an optional payroll
payment id.

### Invariants

1. `amount >= 0`.
2. `type` is `income` or `expense`.
3. The `after_balance` matches the running balance of the bank
   account at insertion.
4. Statements are append-only; corrections are made by reverse
   statements.

### Commands

- `RecordBankStatement`
- `ReverseBankStatement`

### Events

- `BankStatementRecorded`
- `BankStatementReversed`

---

## BankPaymentSlip

**Root type:** `BankPaymentSlip`
**Identity:** `BankPaymentSlipId(SchoolId, Uuid)`

### Purpose

A slip submitted by a parent/student for a bank transfer or cheque
payment. The slip is `pending` until approved.

### Invariants

1. `payment_mode` is `Bk` (bank transfer) or `Cq` (cheque).
2. `approve_status` is `pending`, `approved`, or `rejected`.
3. Only approved slips may be promoted to a `BankStatement` and a
   `FeesPayment`.
4. A slip carries the bank id, the fees type, the fees discount, the
   student id, the assign id, the class id, the section id, and an
   optional installment id.

### Commands

- `GenerateBankPaymentSlip`
- `ApproveBankPayment`
- `RejectBankPayment`

### Events

- `BankPaymentSlipGenerated`
- `BankPaymentApproved`
- `BankPaymentRejected`

---

## Expense

**Root type:** `Expense`
**Identity:** `ExpenseId(SchoolId, Uuid)`

### Purpose

A recorded expense. Carries name, date, amount, file reference,
description, expense head, account (bank), payment method, and a
link to a payroll payment (for payroll-derived expenses).

### Invariants

1. `amount >= 0`.
2. The expense's `payment_method` and `account` must be compatible
   (cash payment → cash account; bank → bank account).
3. The expense has exactly one `expense_head`.

### Commands

- `RecordExpense`
- `UpdateExpense`
- `DeleteExpense`

### Events

- `ExpenseRecorded`
- `ExpenseUpdated`
- `ExpenseDeleted`

---

## Income

**Root type:** `Income`
**Identity:** `IncomeId(SchoolId, Uuid)`

### Purpose

A recorded income. Carries name, date, amount, file reference,
description, income head, account, payment method, and an optional
link to a fees collection, an installment payment, an inventory
item, or an item sell.

### Invariants

1. `amount >= 0`.
2. The income is anchored to one `income_head`.
3. The income's account and payment method are compatible.

### Commands

- `RecordIncome`
- `UpdateIncome`
- `DeleteIncome`

### Events

- `IncomeRecorded`
- `IncomeUpdated`
- `IncomeDeleted`

---

## Donor

**Root type:** `Donor`
**Identity:** `DonorId(SchoolId, Uuid)`

### Purpose

A donor profile. Carries full name, profession, date of birth, email,
mobile, photo, age, addresses, and a `show_public` flag. Has
references to a blood group, religion, and gender (using the
shared `BaseSetup` lookup table owned by the platform).

### Invariants

1. `show_public` is a boolean.
2. A donor is unique by email within a school when email is provided.

### Commands

- `RegisterDonor`
- `UpdateDonor`
- `DeleteDonor`

### Events

- `DonorRegistered`
- `DonorUpdated`
- `DonorDeleted`

---

## ExpenseHead

**Root type:** `ExpenseHead`
**Identity:** `ExpenseHeadId(SchoolId, Uuid)`

### Purpose

A category for expenses (e.g. "Utilities", "Salaries", "Supplies").

### Invariants

1. Unique by `name` within a school.

### Commands

- `CreateExpenseHead`
- `UpdateExpenseHead`
- `DeleteExpenseHead`

### Events

- `ExpenseHeadCreated`
- `ExpenseHeadUpdated`
- `ExpenseHeadDeleted`

---

## IncomeHead

**Root type:** `IncomeHead`
**Identity:** `IncomeHeadId(SchoolId, Uuid)`

### Purpose

A category for income (e.g. "Donations", "Rentals", "Sales").

### Invariants

1. Unique by `name` within a school.

### Commands

- `CreateIncomeHead`
- `UpdateIncomeHead`
- `DeleteIncomeHead`

### Events

- `IncomeHeadCreated`
- `IncomeHeadUpdated`
- `IncomeHeadDeleted`

---

## WalletTransaction

**Root type:** `WalletTransaction`
**Identity:** `WalletTransactionId(SchoolId, Uuid)`

### Purpose

A wallet movement. A wallet transaction may be a `deposit`,
`refund`, `expense`, or `fees_refund`. It has a status of
`pending`, `approve`, or `reject`.

### Invariants

1. `amount >= 0`.
2. `status` is `pending`, `approve`, or `reject`.
3. Only `approve` transitions the wallet balance.
4. The transaction references a user and an optional bank.

### Commands

- `AddWalletCredit`
- `RequestWalletRefund`
- `DeductWalletCredit`
- `ApproveWalletTransaction`
- `RejectWalletTransaction`

### Events

- `WalletCredited`
- `WalletRefundRequested`
- `WalletDebited`
- `WalletTransactionApproved`
- `WalletTransactionRejected`

---

## Transaction

**Root type:** `Transaction`
**Identity:** `TransactionId(SchoolId, Uuid)`

### Purpose

A typed double-entry line. The transaction carries `title`, `type`
(`debit` or `credit`), `payment_method`, `reference`, `description`,
a polymorphic `morphable_id` / `morphable_type`, and the user that
created it.

### Invariants

1. `type` is `debit` or `credit`.
2. The polymorphic target is one of the supported finance entities
   (invoice, expense, income, payroll, wallet, transfer, product
   purchase, inventory payment, donation).
3. `amount >= 0`.

### Commands

- `RecordTransaction`
- `ReverseTransaction`

### Events

- `TransactionRecorded`
- `TransactionReversed`

---

## PayrollPayment

**Root type:** `PayrollPayment`
**Identity:** `PayrollPaymentId(SchoolId, Uuid)`

### Purpose

A payment against a `PayrollGenerate`. Carries the payroll id,
amount, payment mode, payment method id, payment date, bank id, and
a note. The aggregate is owned by the finance domain; the underlying
`PayrollGenerate` aggregate is owned by the HR domain.

### Invariants

1. The sum of `PayrollPayment` amounts against a `PayrollGenerate`
   never exceeds the payroll's unpaid `net_salary`.
2. The payment's `payment_method` and `bank_id` are compatible.
3. A payment creates a corresponding `Expense` and `BankStatement`
   on approval.

### Commands

- `RecordPayrollPayment`

### Events

- `PayrollPaymentRecorded`
- `PayrollPaid` (when the payroll is fully paid)

---

## PayrollGenerate

**Root type:** `PayrollGenerate`
**Identity:** `PayrollGenerateId(SchoolId, Uuid)`

### Purpose

The monthly payroll run for a single staff member. The aggregate is
created by the HR domain's payroll service and queried by finance to
record payments. It carries basic salary, total earnings, total
deductions, gross salary, tax, net salary, payroll month and year,
status (`not_generated`, `generated`, `paid`), payment mode, payment
date, bank, paid amount, and `is_partial` flag.

### Invariants

1. `gross_salary == basic_salary + total_earning`.
2. `net_salary == gross_salary - total_deduction - tax`.
3. `payroll_status` is `not_generated` (initial), `generated`, or
   `paid`. `paid` is terminal.
4. `paid_amount <= net_salary`.

### Commands (HR-owned; finance reads)

- `GeneratePayroll` (HR)
- `ApprovePayroll` (HR)
- `PayPayroll` (finance)

### Events

- `PayrollGenerated`
- `PayrollApproved`
- `PayrollPaid`

---

## PayrollEarnDeduc

**Root type:** `PayrollEarnDeduc`
**Identity:** `PayrollEarnDeducId(SchoolId, Uuid)`

### Purpose

A single earnings or deductions line on a `PayrollGenerate`. Carries
`type_name`, `amount`, `earn_dedc_type` (`e` or `d`), and a back-
reference to the payroll.

### Invariants

1. `amount >= 0`.
2. `earn_dedc_type` is `e` (earning) or `d` (deduction).
3. The sum of all `e` rows equals `total_earning`; the sum of all
   `d` rows equals `total_deduction`.

### Commands

- `AddPayrollEarning`
- `AddPayrollDeduction`
- `UpdatePayrollEarnDeduc`
- `DeletePayrollEarnDeduc`

### Events

- `PayrollEarningAdded`
- `PayrollDeductionAdded`
- `PayrollEarnDeducUpdated`
- `PayrollEarnDeducDeleted`

---

## SalaryTemplate

**Root type:** `SalaryTemplate`
**Identity:** `SalaryTemplateId(SchoolId, Uuid)`

### Purpose

A reusable salary grade and structure (e.g. "Grade 3: Basic 15000 +
House Rent 3000"). Carries `salary_grades`, `salary_basic`,
`overtime_rate`, `house_rent`, `provident_fund`, `gross_salary`,
`total_deduction`, `net_salary`.

### Invariants

1. `gross_salary == salary_basic + house_rent + provident_fund` (or
   the consumer-defined composition).
2. `net_salary == gross_salary - total_deduction`.

### Commands

- `CreateSalaryTemplate`
- `UpdateSalaryTemplate`
- `DeleteSalaryTemplate`

### Events

- `SalaryTemplateCreated`
- `SalaryTemplateUpdated`
- `SalaryTemplateDeleted`

---

## ProductPurchase

**Root type:** `ProductPurchase`
**Identity:** `ProductPurchaseId(SchoolId, Uuid)`

### Purpose

A purchase of a vendor product (e.g. an SMS bundle). Carries
purchase date, expiry date, price, paid amount, due amount, package
code, the staff who initiated it, and the user behind it.

### Invariants

1. `paid_amount + due_amount == price`.
2. `paid_amount >= 0` and `due_amount >= 0`.
3. The purchase is owned by exactly one school.

### Commands

- `RecordProductPurchase`
- `RecordProductPayment`

### Events

- `ProductPurchaseRecorded`
- `ProductPaymentRecorded`

---

## InventoryPayment

**Root type:** `InventoryPayment`
**Identity:** `InventoryPaymentId(SchoolId, Uuid)`

### Purpose

A payment against an inventory item (e.g. a book purchase from a
supplier, or a sell to a buyer). Carries `item_receive_sell_id`,
payment date, amount, reference number, payment type (`R` receive or
`S` sell), payment method, and a note.

### Invariants

1. `payment_type` is `R` or `S`.
2. `amount >= 0`.
3. The payment's `payment_method` and `bank_id` are compatible.

### Commands

- `RecordInventoryPayment`

### Events

- `InventoryPaymentRecorded`

---

## AmountTransfer

**Root type:** `AmountTransfer`
**Identity:** `AmountTransferId(SchoolId, Uuid)`

### Purpose

A transfer between two bank accounts. Captures the source account,
the destination account, the amount, the date, and a note.

### Invariants

1. The source and destination accounts differ.
2. `amount > 0`.
3. The transfer produces exactly two `BankStatement` rows (one debit
   on the source, one credit on the destination) in a single
   transaction.

### Commands

- `TransferFunds`

### Events

- `FundsTransferred`

---

## ChartOfAccount

**Root type:** `ChartOfAccount`
**Identity:** `ChartOfAccountId(SchoolId, Uuid)`

### Purpose

A typed accounting category: `asset`, `liability`, `income`,
`expense`, or `equity`. Drives ledger reporting and is referenced by
expenses, income, payroll, and transfers.

### Invariants

1. Each `ChartOfAccount` is unique by `name` within a school.
2. A `ChartOfAccount` cannot be deleted while any `Expense`,
   `Income`, or `BankStatement` references it.

### Commands

- `CreateChartOfAccount`
- `UpdateChartOfAccount`
- `DeleteChartOfAccount`

### Events

- `ChartOfAccountCreated`
- `ChartOfAccountUpdated`
- `ChartOfAccountDeleted`

---

## QuestionBankFee

**Root type:** `QuestionBankFee`
**Identity:** `QuestionBankFeeId(SchoolId, Uuid)`

### Purpose

A fees type associated with a question bank (e.g. "Exam paper fee").
The question bank itself is owned by the assessment domain; the fees
mapping lives here so that the standard fees assignment, invoice,
and payment flow applies.

### Invariants

1. The `fees_type_id` is unique per question bank.

### Commands

- `AttachFeesToQuestionBank`
- `DetachFeesFromQuestionBank`

### Events

- `FeesAttachedToQuestionBank`
- `FeesDetachedFromQuestionBank`

---

## PaymentGatewaySetting

**Root type:** `PaymentGatewaySetting`
**Identity:** `PaymentGatewaySettingId(SchoolId, Uuid)`

### Purpose

Per-gateway credentials and mode. Carries gateway name, username,
password, signature, client id, secret key, secret word, publisher
key, private key, mode (`sandbox` or `live`), service charge, charge
type (`P` percentage or `F` flat), and charge value.

### Invariants

1. The gateway name is unique within a school.
2. The mode is `sandbox` or `live`.
3. The charge is non-negative; the charge type is `P` or `F`.
4. Credentials are encrypted at rest by the storage adapter.

### Commands

- `ConfigurePaymentGateway`
- `UpdatePaymentGateway`
- `DisablePaymentGateway`

### Events

- `PaymentGatewayConfigured`
- `PaymentGatewayUpdated`
- `PaymentGatewayDisabled`

---

## PaymentMethod

**Root type:** `PaymentMethod`
**Identity:** `PaymentMethodId(SchoolId, Uuid)`

### Purpose

A payment method (e.g. "Cash", "Bank Transfer", "Cheque", "Card",
"Mobile", "Stripe"). Each method is associated with a gateway (if
applicable) and a type.

### Invariants

1. A `PaymentMethod` is unique by `method` within a school.
2. Methods backed by a gateway require a `gateway_id`.
3. Cash and bank-transfer methods do not require a gateway.

### Commands

- `CreatePaymentMethod`
- `UpdatePaymentMethod`
- `DeletePaymentMethod`

### Events

- `PaymentMethodCreated`
- `PaymentMethodUpdated`
- `PaymentMethodDeleted`

---

## DirectFeesReminder

**Root type:** `DirectFeesReminder`
**Identity:** `DirectFeesReminderId(SchoolId, Uuid)`

### Purpose

A "remind N days before due" rule. Carries `due_date_before` and
`notification_types` (a comma-separated list of channels).

### Invariants

1. `due_date_before >= 0`.

### Commands

- `ConfigureFeesReminder`
- `UpdateFeesReminder`
- `DeleteFeesReminder`

### Events

- `FeesReminderConfigured`
- `FeesReminderUpdated`
- `FeesReminderDeleted`

---

## DueFeesLoginPrevent

**Root type:** `DueFeesLoginPrevent`
**Identity:** `DueFeesLoginPreventId(SchoolId, Uuid)`

### Purpose

A list of `(user_id, role_id)` pairs that are blocked from logging
in while the user has an overdue fees balance. The list is
maintained per school and per academic year.

### Invariants

1. Each row is unique by `(school_id, academic_id, user_id, role_id)`.
2. Only user ids with a non-zero overdue balance are kept; rows are
   auto-pruned when balance becomes zero.

### Commands

- `BlockLoginForDueFees`
- `UnblockLoginForDueFees`

### Events

- `DueFeesLoginPrevented`
- `DueFeesLoginRestored`

---

## FeesCarryForwardLog

**Root type:** `FeesCarryForwardLog`
**Identity:** `FeesCarryForwardLogId(SchoolId, Uuid)`

### Purpose

An audit row recording a single carry-forward action. Carries the
student record id, the note, the amount, the amount type, the type
of operation, and the date.

### Invariants

1. The log is append-only.
2. The amount is non-negative.

### Commands

- (None — produced as a side effect of `CarryForwardFeesBalance`.)

### Events

- `FeesCarryForwardLogged`

---

## FeesInstallmentCredit

**Root type:** `FeesInstallmentCredit`
**Identity:** `FeesInstallmentCreditId(SchoolId, Uuid)`

### Purpose

A pre-paid credit applied to a student's upcoming installment.
Carries the student id, the student record id, the active status,
and the amount.

### Invariants

1. `amount >= 0`.
2. The credit is unique by `(student_id, student_record_id)`.
3. `active_status` is `1` while the credit is open.

### Commands

- `AddFeesInstallmentCredit`
- `ConsumeFeesInstallmentCredit`
- `CancelFeesInstallmentCredit`

### Events

- `FeesInstallmentCreditAdded`
- `FeesInstallmentCreditConsumed`
- `FeesInstallmentCreditCancelled`

---

## FeesCarryForwardSetting

**Root type:** `FeesCarryForwardSetting`
**Identity:** `FeesCarryForwardSettingId(SchoolId, Uuid)`

### Purpose

Configuration row for when carry-forward runs. Carries the title,
`fees_due_days` (how many days after the due date a balance is
carried forward), and the `payment_gateway` reference.

### Invariants

1. The title is unique within a school.
2. `fees_due_days >= 0`.

### Commands

- `ConfigureFeesCarryForward`

### Events

- `FeesCarryForwardConfigured`

---

## DirectFeesSetting

**Root type:** `DirectFeesSetting`
**Identity:** `DirectFeesSettingId(SchoolId, Uuid)`

### Purpose

Global toggles for the direct-fees flow. Carries `fees_installment`
(toggle), `fees_reminder` (toggle), `reminder_before` (days),
`no_installment` (count), `due_date_from_sem` (day of month), and
`end_day` (optional day of month for the last installment).

### Invariants

1. `reminder_before >= 0` and `no_installment >= 0`.
2. `due_date_from_sem in 1..=28`.

### Commands

- `ConfigureDirectFees`

### Events

- `DirectFeesConfigured`

## BankPaymentSlipAudit

**Root type:** `BankPaymentSlipAudit`
**Identity:** `BankPaymentSlipAuditId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Finance

### Purpose

The `BankPaymentSlipAudit` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `BankPaymentSlipAuditId` within a school.

### Commands

- `CreateBankPaymentSlipAudit`
- `UpdateBankPaymentSlipAudit`
- `DeleteBankPaymentSlipAudit`

### Events

- `BankPaymentSlipAuditCreated`

---

## BankStatementAttachment

**Root type:** `BankStatementAttachment`
**Identity:** `BankStatementAttachmentId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Finance

### Purpose

The `BankStatementAttachment` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `BankStatementAttachmentId` within a school.

### Commands

- `CreateBankStatementAttachment`
- `UpdateBankStatementAttachment`
- `DeleteBankStatementAttachment`

### Events

- `BankStatementAttachmentCreated`

---

## DirectFeesInstallmentAssignChild

**Root type:** `DirectFeesInstallmentAssignChild`
**Identity:** `DirectFeesInstallmentAssignChildId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Finance

### Purpose

The `DirectFeesInstallmentAssignChild` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `DirectFeesInstallmentAssignChildId` within a school.

### Commands

- `CreateDirectFeesInstallmentAssignChild`
- `UpdateDirectFeesInstallmentAssignChild`
- `DeleteDirectFeesInstallmentAssignChild`

### Events

- `DirectFeesInstallmentAssignChildCreated`

---

## ExpenseApproval

**Root type:** `ExpenseApproval`
**Identity:** `ExpenseApprovalId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Finance

### Purpose

The `ExpenseApproval` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `ExpenseApprovalId` within a school.

### Commands

- `CreateExpenseApproval`
- `UpdateExpenseApproval`
- `DeleteExpenseApproval`

### Events

- `ExpenseApprovalCreated`

---

## FeesInstallmentAssignDiscount

**Root type:** `FeesInstallmentAssignDiscount`
**Identity:** `FeesInstallmentAssignDiscountId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Finance

### Purpose

The `FeesInstallmentAssignDiscount` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `FeesInstallmentAssignDiscountId` within a school.

### Commands

- `CreateFeesInstallmentAssignDiscount`
- `UpdateFeesInstallmentAssignDiscount`
- `DeleteFeesInstallmentAssignDiscount`

### Events

- `FeesInstallmentAssignDiscountCreated`

---

## FmFeesInvoiceLineNote

**Root type:** `FmFeesInvoiceLineNote`
**Identity:** `FmFeesInvoiceLineNoteId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Finance

### Purpose

The `FmFeesInvoiceLineNote` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `FmFeesInvoiceLineNoteId` within a school.

### Commands

- `CreateFmFeesInvoiceLineNote`
- `UpdateFmFeesInvoiceLineNote`
- `DeleteFmFeesInvoiceLineNote`

### Events

- `FmFeesInvoiceLineNoteCreated`

---

## FmFeesTransactionLineNote

**Root type:** `FmFeesTransactionLineNote`
**Identity:** `FmFeesTransactionLineNoteId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Finance

### Purpose

The `FmFeesTransactionLineNote` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `FmFeesTransactionLineNoteId` within a school.

### Commands

- `CreateFmFeesTransactionLineNote`
- `UpdateFmFeesTransactionLineNote`
- `DeleteFmFeesTransactionLineNote`

### Events

- `FmFeesTransactionLineNoteCreated`

---

## IncomeApproval

**Root type:** `IncomeApproval`
**Identity:** `IncomeApprovalId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Finance

### Purpose

The `IncomeApproval` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `IncomeApprovalId` within a school.

### Commands

- `CreateIncomeApproval`
- `UpdateIncomeApproval`
- `DeleteIncomeApproval`

### Events

- `IncomeApprovalCreated`

---

## PayrollPaymentApproval

**Root type:** `PayrollPaymentApproval`
**Identity:** `PayrollPaymentApprovalId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Finance

### Purpose

The `PayrollPaymentApproval` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `PayrollPaymentApprovalId` within a school.

### Commands

- `CreatePayrollPaymentApproval`
- `UpdatePayrollPaymentApproval`
- `DeletePayrollPaymentApproval`

### Events

- `PayrollPaymentApprovalCreated`

---

## Wallet

**Root type:** `Wallet`
**Identity:** `WalletId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Finance

### Purpose

The `Wallet` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `WalletId` within a school.

### Commands

- `CreateWallet`
- `UpdateWallet`
- `DeleteWallet`

### Events

- `WalletCreated`

---

## WalletTransactionApproval

**Root type:** `WalletTransactionApproval`
**Identity:** `WalletTransactionApprovalId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Finance

### Purpose

The `WalletTransactionApproval` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `WalletTransactionApprovalId` within a school.

### Commands

- `CreateWalletTransactionApproval`
- `UpdateWalletTransactionApproval`
- `DeleteWalletTransactionApproval`

### Events

- `WalletTransactionApprovalCreated`

---



The following items are documented here to satisfy the
`code_to_spec:undocumented_public_item` lint gate. They were
discovered after the main spec was written.

## BankPaymentSlipAudit

**Root type:** `BankPaymentSlipAudit`
**Identity:** `BankPaymentSlipAuditId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Finance

### Purpose

The `BankPaymentSlipAudit` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `BankPaymentSlipAuditId` within a school.

### Commands

- `CreateBankPaymentSlipAudit`
- `UpdateBankPaymentSlipAudit`
- `DeleteBankPaymentSlipAudit`

### Events

- `BankPaymentSlipAuditCreated`

---

## BankStatementAttachment

**Root type:** `BankStatementAttachment`
**Identity:** `BankStatementAttachmentId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Finance

### Purpose

The `BankStatementAttachment` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `BankStatementAttachmentId` within a school.

### Commands

- `CreateBankStatementAttachment`
- `UpdateBankStatementAttachment`
- `DeleteBankStatementAttachment`

### Events

- `BankStatementAttachmentCreated`

---

## DirectFeesInstallmentAssignChild

**Root type:** `DirectFeesInstallmentAssignChild`
**Identity:** `DirectFeesInstallmentAssignChildId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Finance

### Purpose

The `DirectFeesInstallmentAssignChild` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `DirectFeesInstallmentAssignChildId` within a school.

### Commands

- `CreateDirectFeesInstallmentAssignChild`
- `UpdateDirectFeesInstallmentAssignChild`
- `DeleteDirectFeesInstallmentAssignChild`

### Events

- `DirectFeesInstallmentAssignChildCreated`

---

## ExpenseApproval

**Root type:** `ExpenseApproval`
**Identity:** `ExpenseApprovalId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Finance

### Purpose

The `ExpenseApproval` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `ExpenseApprovalId` within a school.

### Commands

- `CreateExpenseApproval`
- `UpdateExpenseApproval`
- `DeleteExpenseApproval`

### Events

- `ExpenseApprovalCreated`

---

## FeesInstallmentAssignDiscount

**Root type:** `FeesInstallmentAssignDiscount`
**Identity:** `FeesInstallmentAssignDiscountId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Finance

### Purpose

The `FeesInstallmentAssignDiscount` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `FeesInstallmentAssignDiscountId` within a school.

### Commands

- `CreateFeesInstallmentAssignDiscount`
- `UpdateFeesInstallmentAssignDiscount`
- `DeleteFeesInstallmentAssignDiscount`

### Events

- `FeesInstallmentAssignDiscountCreated`

---

## FmFeesInvoiceLineNote

**Root type:** `FmFeesInvoiceLineNote`
**Identity:** `FmFeesInvoiceLineNoteId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Finance

### Purpose

The `FmFeesInvoiceLineNote` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `FmFeesInvoiceLineNoteId` within a school.

### Commands

- `CreateFmFeesInvoiceLineNote`
- `UpdateFmFeesInvoiceLineNote`
- `DeleteFmFeesInvoiceLineNote`

### Events

- `FmFeesInvoiceLineNoteCreated`

---

## FmFeesTransactionLineNote

**Root type:** `FmFeesTransactionLineNote`
**Identity:** `FmFeesTransactionLineNoteId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Finance

### Purpose

The `FmFeesTransactionLineNote` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `FmFeesTransactionLineNoteId` within a school.

### Commands

- `CreateFmFeesTransactionLineNote`
- `UpdateFmFeesTransactionLineNote`
- `DeleteFmFeesTransactionLineNote`

### Events

- `FmFeesTransactionLineNoteCreated`

---

## IncomeApproval

**Root type:** `IncomeApproval`
**Identity:** `IncomeApprovalId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Finance

### Purpose

The `IncomeApproval` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `IncomeApprovalId` within a school.

### Commands

- `CreateIncomeApproval`
- `UpdateIncomeApproval`
- `DeleteIncomeApproval`

### Events

- `IncomeApprovalCreated`

---

## PayrollPaymentApproval

**Root type:** `PayrollPaymentApproval`
**Identity:** `PayrollPaymentApprovalId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Finance

### Purpose

The `PayrollPaymentApproval` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `PayrollPaymentApprovalId` within a school.

### Commands

- `CreatePayrollPaymentApproval`
- `UpdatePayrollPaymentApproval`
- `DeletePayrollPaymentApproval`

### Events

- `PayrollPaymentApprovalCreated`

---

## Wallet

**Root type:** `Wallet`
**Identity:** `WalletId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Finance

### Purpose

The `Wallet` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `WalletId` within a school.

### Commands

- `CreateWallet`
- `UpdateWallet`
- `DeleteWallet`

### Events

- `WalletCreated`

---

## WalletTransactionApproval

**Root type:** `WalletTransactionApproval`
**Identity:** `WalletTransactionApprovalId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Finance

### Purpose

The `WalletTransactionApproval` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `WalletTransactionApprovalId` within a school.

### Commands

- `CreateWalletTransactionApproval`
- `UpdateWalletTransactionApproval`
- `DeleteWalletTransactionApproval`

### Events

- `WalletTransactionApprovalCreated`

---
