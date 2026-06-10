# Finance Domain Overview

## Purpose

The finance domain owns the school's monetary spine. It tracks every
amount that flows in (fees, donations, sales, payroll recoveries,
inventory income) and every amount that flows out (expenses, payroll,
purchases, refunds). It also owns banking, payment slips, payment
gateways, payroll, wallet credits, carry-forward of unpaid balances,
and the blocking of parent/student logins when balances are overdue.

The domain is the canonical ledger of the school: every figure shown
to a parent, an accountant, a regulator, or a court is built from
aggregates that live here.

## Responsibilities

- Fees master, fees group, fees type, and fees assignment per class
  and per student.
- Discounts, both catalog-wide and per-student.
- Invoice generation, numbering, and configuration.
- Installment plans, including "direct fees" installment schemes that
  bypass the master/invoice path.
- Payment collection across cash, bank, cheque, card, mobile, and
  payment gateway.
- Bank accounts, bank statements, bank payment slips, and bank
  reconciliation.
- Expenses, expense heads, income, income heads, and finance_donors.
- Payroll generation, approval, payment, and per-month earnings and
  deductions.
- Wallet credits, debits, and refunds.
- Carry-forward of unpaid balances across the academic year.
- Blocking portal logins of users with overdue balances.
- Online payment gateway configuration and method binding.
- Invoice, slip, and receipt printing configuration.
- Question-bank-level fees (where exam-question paper purchases are
  charged as fees).
- Inventory payments, product purchases, and amount transfers between
  accounts.

## Boundaries

The finance domain does **not** own:

- Student or guardian identity (see `specs/academic/`).
- Staff identity, leave, attendance, designations (see `specs/hr/`).
- Class routines, subjects, exams (see `specs/academic/` and
  `specs/assessment/`).
- Notification delivery — reminders are produced as events; delivery
  is a port.

The finance domain **does** own all money in the school. Every
transaction (income, expense, payment, refund, transfer) is anchored
to a `SchoolId` and a `BankAccountId` (or to the cash account).

## Dependencies

- `educore-core` — error types, result, identifier trait.
- `educore-platform` — `SchoolId`, `UserId`, `TenantContext`.
- `educore-rbac` — capability checks.
- `educore-events` — domain event publishing.
- `educore-academic` — `StudentId`, `ClassId`, `SectionId`,
  `AcademicYearId`, `StudentRecordId`.
- `educore-hr` — `StaffId` for payroll and staff-attendance related
  expense capture.

## Domain Invariants

1. Every monetary aggregate is anchored to a `SchoolId`.
2. A fees amount is non-negative; a discount amount is non-negative.
3. An invoice's `paid_amount` plus `due_amount` equals its
   `sub_total` plus `fine` plus `service_charge` plus `weaver`.
4. A bank account's `current_balance` is derived from its statements;
   it is never written directly except at account creation.
5. A payroll is generated before it is approved, and approved before
   it is paid. A `PayrollPaid` event is the only terminal transition.
6. A `FeesInstallment` can be paid only if it is `Active`.
7. Carry-forward never overwrites a previous forward for the same
   student in the same academic year; it adds to the existing balance.
8. A wallet transaction is `pending` until approved; only `approve`
   transitions credit the wallet.
9. A bank payment slip is `pending` until approved; only approved
   slips contribute to a bank statement.
10. The "due fees login prevention" rule blocks logins only of the
    user-ids explicitly listed, never of staff.
11. Payroll earnings minus deductions equals `net_salary`; a partial
    payment cannot exceed the unpaid balance.
12. A fees discount assigned to a student cannot exceed the student's
    applicable fees amount.
13. Question-bank fees follow the same invoice and payment path as
    class fees; they are not a separate ledger.
14. An amount transfer between two accounts always produces two bank
    statements (one debit, one credit) in a single transaction.

## Aggregate Roots

| Aggregate                       | Root Type                 | Purpose                                              |
| ------------------------------- | ------------------------- | ---------------------------------------------------- |
| FeesGroup                       | `FeesGroup`               | Logical grouping of fees (e.g. "Tuition", "Exam")    |
| FeesType                        | `FeesType`                | A billable line item inside a group                  |
| FeesMaster                      | `FeesMaster`              | Class+type+amount master record                      |
| FeesAssign                      | `FeesAssign`              | Per-student assignment of a master record            |
| FeesAssignDiscount              | `FeesAssignDiscount`      | Per-student applied discount                         |
| FeesDiscount                    | `FeesDiscount`            | Discount catalog entry                               |
| FeesInvoice                     | `FeesInvoice`             | The classic invoice header (prefix + start_form)     |
| FeesInstallment                 | `FeesInstallment`         | Classic installment plan                             |
| FeesInstallmentAssign           | `FeesInstallmentAssign`   | Per-student installment assignment                   |
| FeesPayment                     | `FeesPayment`             | A payment against a fees assign/installment          |
| FeesCarryForward                | `FeesCarryForward`        | Carry-forwarded balance per student                  |
| DirectFeesInstallment           | `DirectFeesInstallment`   | "Direct" installment plan (custom percent and due)   |
| DirectFeesInstallmentAssign     | `DirectFeesInstallmentAssign` | Per-student direct installment assignment        |
| DirectFeesInstallmentChildPayment | `DirectFeesInstallmentChildPayment` | Payment against a direct installment      |
| FmFeesGroup                     | `FmFeesGroup`             | Newer invoice-scheme group                           |
| FmFeesType                      | `FmFeesType`              | Newer invoice-scheme type                            |
| FmFeesInvoice                   | `FmFeesInvoice`           | Newer invoice header                                 |
| FmFeesInvoiceChild              | `FmFeesInvoiceChild`      | Newer invoice line                                   |
| FmFeesTransaction               | `FmFeesTransaction`       | Newer payment transaction                            |
| FmFeesTransactionChild          | `FmFeesTransactionChild`  | Newer transaction line                               |
| FmFeesWeaver                    | `FmFeesWeaver`            | Per-invoice weaver/fine adjustment                   |
| FeesInvoiceSetting              | `FeesInvoiceSetting`      | Classic invoice layout settings                      |
| InvoiceSetting                  | `InvoiceSetting`          | Newer invoice layout settings                        |
| FmFeesInvoiceSetting           | `FmFeesInvoiceSetting`    | Newer invoice numbering and positions                |
| BankAccount                     | `BankAccount`             | A school bank or cash account                        |
| BankStatement                   | `BankStatement`           | One entry in the bank ledger                         |
| BankPaymentSlip                 | `BankPaymentSlip`         | A slip submitted for a bank/cheque payment           |
| Expense                         | `Expense`                 | A recorded expense                                   |
| Income                          | `Income`                  | A recorded income                                    |
| Donor                           | `Donor`                   | A donor profile                                      |
| ExpenseHead                     | `ExpenseHead`             | A category for expenses                              |
| IncomeHead                      | `IncomeHead`              | A category for income                                |
| WalletTransaction               | `WalletTransaction`       | A wallet credit, debit, refund, or expense           |
| Transaction                     | `Transaction`             | The double-entry journal line                        |
| PayrollPayment                  | `PayrollPayment`          | A payment against a payroll                          |
| PayrollGenerate                 | `PayrollGenerate`         | The monthly payroll run (typed here, persisted by HR)|
| PayrollEarnDeduc                | `PayrollEarnDeduc`        | A single earnings or deductions line on a payroll    |
| SalaryTemplate                  | `SalaryTemplate`          | A reusable salary grade and structure (typed here)    |
| ProductPurchase                 | `ProductPurchase`         | A purchase of a product (e.g. an SMS bundle)         |
| InventoryPayment                | `InventoryPayment`        | A payment against an inventory item                  |
| AmountTransfer                  | `AmountTransfer`          | A transfer between two bank accounts                 |
| ChartOfAccount                  | `ChartOfAccount`          | An accounting category (asset/liability/expense/...) |
| QuestionBankFee                 | `QuestionBankFee`         | Fees configured on a question bank (e.g. paper fee)  |
| PaymentGatewaySetting           | `PaymentGatewaySetting`   | Per-gateway credentials and mode                     |
| PaymentMethod                   | `PaymentMethod`           | Catalog of accepted payment methods                  |
| DirectFeesReminder              | `DirectFeesReminder`      | A "remind N days before due" rule                    |
| DueFeesLoginPrevent             | `DueFeesLoginPrevent`     | The list of user-ids blocked from login for dues     |
| FeesCarryForwardLog             | `FeesCarryForwardLog`     | Audit row for a single carry-forward                 |
| FeesInstallmentCredit           | `FeesInstallmentCredit`   | A pre-paid credit applied to an installment          |
| FeesCarryForwardSetting         | `FeesCarryForwardSetting` | When and how carry-forward is triggered              |
| DirectFeesSetting               | `DirectFeesSetting`       | Global toggles for direct fees and reminders         |
| FeesInvoiceSetting              | `FeesInvoiceSetting`      | (above, listed again)                                |

Each aggregate is documented in detail under `docs/specs/finance/aggregates.md`.

## Cross-Domain Impact

When a `Student` is admitted, the academic domain emits
`StudentAdmitted`. Finance subscribes and:

- Creates a `FeesAssign` per active `FeesMaster` matching the student's
  class and section.
- Subscribes to `StudentCategoryChanged` to recompute discounts.

When a `Student` is promoted, the academic domain emits
`StudentPromoted`. Finance subscribes and:

- Closes or carries-forward the prior-year balance.
- Re-assigns masters for the new year.

When a `Student` is withdrawn, the academic domain emits
`StudentWithdrawn`. Finance subscribes and:

- Marks the student's open assignments as `Closed`.
- Optionally triggers a refund workflow.

When a `Staff` is registered, the HR domain emits `StaffRegistered`.
Finance subscribes and:

- Reads the staff's salary template and hourly rate to prepare
  payroll.

## Consumers

- Web admin UI (accountant, fees manager).
- Parent/student portal (view invoices, pay, view receipts).
- Mobile parent app (push reminders, gateway payment).
- Mobile student app (view balances, pay by wallet).
- Desktop cashier tool (slip printing, cash collection).
- Bank reconciliation automation.
- AI agent (collect fees, generate payroll, generate invoices).

## Anti-Goals

- The finance domain does not connect to any payment gateway. Gateway
  integration is a port.
- The finance domain does not print, render, or PDF invoices.
  Rendering is a port.
- The finance domain does not decide academic policy (e.g. "what is
  a fees group"). The consumer defines the catalog.
- The finance domain does not maintain its own authentication or
  authorization; it queries the RBAC port for every command.
