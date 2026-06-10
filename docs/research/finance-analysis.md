# Finance Domain — Business Analysis

## Purpose

The finance domain is the school's monetary spine. It
owns every rupee, dollar, euro, or peso that flows in
or out. It is the most complex domain in the engine:
the most aggregates, the most business rules, the
most integrations, and the most regulatory attention.

This document describes how fees, payments, banking,
expenses, payroll, and the rest of finance work in
real schools, with the edge cases that real schools
hit.

> This is the deepest analysis because finance is the
> domain where misunderstanding the school costs the
> school money — and the school's ability to operate.

## Key Concepts

### Fees Catalog

- **FeesGroup** — a logical grouping of fees
  ("Tuition", "Exam", "Transport", "Hostel").
- **FeesType** — a billable line item inside a group
  ("Tuition Term 1", "Library Fee", "Lab Fee").
- **FeesMaster** — a class+type+amount master record
  defining the amount for a class in an academic
  year.
- **FeesAssign** — a per-student assignment of a
  master record.
- **FeesDiscount** — a discount catalog entry
  ("Sibling Discount 10%", "Scholarship 50%").
- **FeesAssignDiscount** — a per-student applied
  discount.

### Invoicing

- **FeesInvoice** — an invoice header (with prefix
  and starting number).
- **FeesInstallment** — a classic installment plan
  (3 installments of 33% each, etc.).
- **FeesInstallmentAssign** — a per-student
  installment assignment.
- **DirectFeesInstallment** — a custom installment
  plan that bypasses the master/invoice path.
- **DirectFeesInstallmentAssign** — a per-student
  direct installment assignment.

### Payment

- **FeesPayment** — a payment against a fees assign
  or installment.
- **PaymentMethod** — a configured payment method
  (cash, bank, cheque, card, mobile, gateway).
- **PaymentGateway** — a configured online payment
  gateway (Stripe, PayPal, local).

### Banking

- **BankAccount** — a bank account held by the
  school.
- **BankPaymentSlip** — a payment slip record
  (physical slip deposited at the bank).
- **BankStatement** — a bank statement entry
  (debit or credit).

### Expense and Income

- **ExpenseHead** — a category of expense
  ("Salary", "Electricity", "Stationery").
- **Expense** — an expense record.
- **IncomeHead** — a category of income
  ("Tuition", "Donation", "Sale").
- **Income** — an income record.
- **Donor** — a donor's record for donation
  tracking.

### Payroll

- **PayrollPeriod** — a month/year payroll period.
- **SalaryTemplate** — a salary structure
  (basic, allowances, deductions).
- **Payroll** — a per-staff payroll record for a
  period.
- **PayrollPayment** — a payment against a payroll
  (a payroll may be paid in parts).
- **LeaveDeduction** — a per-leave-type deduction
  amount.
- **HourlyRate** — an hourly rate for part-time
  staff.

### Wallet

- **Wallet** — a per-user wallet for prepaid
  credits.
- **WalletTransaction** — a wallet credit or
  debit.

### Carry-Forward

- **FeesCarryForward** — a carry-forward record
  for unpaid balances across the academic year.
- **FeesCarryForwardSetting** — a per-school
  carry-forward configuration.

### Due-Fees Login Prevention

- **DueFeesLoginPrevent** — a per-user record
  blocking portal login when fees are overdue.

## Real-World Scenarios

### Fees Master Setup

At the start of the academic year, the school
admin sets up the fees:

1. Define `FeesGroup`s ("Tuition", "Exam",
   "Transport", "Hostel").
2. Define `FeesType`s for each group ("Tuition
   Term 1", "Tuition Term 2", "Library Fee",
   "Lab Fee", "Exam Fee", "Sports Fee").
3. For each `(FeesType, Class)`, create a
   `FeesMaster` with the amount and the due date.

In real schools, fees are **per-class**. Grade 1
tuition is different from Grade 12 tuition. The
amounts may also vary by academic year (with
annual increases).

### Fees Assignment

When a student is admitted (academic domain emits
`StudentAdmitted`), the finance domain:

1. Looks up the `FeesMaster` records for the
   student's class.
2. Creates a `FeesAssign` for each master record,
   applying any applicable `FeesDiscount`s
   (sibling, scholarship, staff child, etc.).
3. Computes the net amount (master - discount).

The engine emits `FeesAssigned` events; the
parent portal shows the assigned fees.

### Fees Discount

A school has multiple discount types:
- **Sibling discount** — 10% for the second
  child, 15% for the third, 20% for the fourth.
- **Scholarship** — based on merit, varies
  (25%, 50%, 100%).
- **Staff child** — 50% for children of staff.
- **Financial aid** — case-by-case.

The discount may be applied:
- **Automatically** — sibling discount is
  computed by the engine from the family's
  enrollment.
- **Manually** — scholarship and financial aid
  are applied by the school's admin.

The engine's `FeesDiscount` aggregate captures
the catalog; the `FeesAssignDiscount` captures
the per-student application.

### Invoice Generation

The school generates invoices for fees:

1. The admin clicks "Generate Invoices" for a
   term.
2. The engine creates one invoice per student
   with all the fees assigned for the term.
3. The invoice has a unique number (per the
   school's invoice prefix and starting number).
4. The invoice is in the parent's portal; the
   parent is notified.

In real schools, invoice generation is:
- **Batch** — by term or by class.
- **All-or-nothing** — a single validation
  failure aborts the batch.
- **Idempotent** — re-running the same batch
  produces no duplicates (the engine checks
  the invoice number range).

### Installment Plan

A school offers a 3-installment payment plan for
tuition:

1. The admin defines the installment schedule
   (33% / 33% / 34% with due dates).
2. The engine creates a `FeesInstallment` for
   each student.
3. Each installment has a due date; the parent
   sees the schedule in the portal.
4. A reminder is sent N days before the due date
   (per the school's policy).

In real schools, installment plans are:
- **Per-fee** — tuition may have installments;
  exam fees do not.
- **Opt-in** — the parent chooses to pay in
  installments.
- **With discount or surcharge** — some schools
  offer a small discount for full payment; some
  charge a small surcharge for installments.

### Direct Fees Installment

Some schools use a simpler "direct fees" model
that bypasses the master/invoice path. The parent
pays a custom amount against a custom due date.
The engine's `DirectFeesInstallment` aggregate
captures this; the `DirectFeesInstallmentAssign`
is the per-student assignment.

This is common in schools that:
- Want a simpler parent experience (one
  payment, one receipt).
- Have irregular fee schedules (e.g. per
  activity).
- Have a single "total fees" rather than
  line items.

### Payment Collection

A parent pays fees. The payment can be:

- **Cash** — paid at the school office. The
  cashier records the payment.
- **Cheque** — paid at the school office or
  mailed. The cashier records the cheque
  number and the deposit date.
- **Bank transfer** — the parent transfers to
  the school's bank. The cashier records the
  transfer reference.
- **Card** — paid at the school office via a
  card terminal.
- **Online gateway** — paid via the parent
  portal via Stripe / PayPal / etc. The
  gateway calls the engine's webhook.
- **Mobile wallet** — paid via a mobile
  wallet.

The engine's `FeesPayment` aggregate captures
every payment with the amount, the mode, the
date, the reference, the bank account, and the
collector.

### Payment Reversal

A payment was recorded in error. The cashier
reverses it. The engine's `PaymentReversed` event
captures the reversal. The original payment's
status becomes `Reversed`. The fees balance
restores.

In real schools, reversals are **audited**:
- The cashier provides a reason.
- The principal approves (if the reversal is
  large).
- The audit log captures the reversal.

### Bank Account Management

The school has one or more bank accounts:

1. The admin creates a `BankAccount` with the
   account name, number, bank name, and
   branch.
2. The current balance is computed from the
   `BankStatement` records; it is not written
   directly.
3. The admin records a `BankStatement` entry
   for each deposit and each withdrawal.

In real schools, bank accounts are:
- **Multiple** — the school has a tuition
  account, a salary account, a general
  account.
- **Per-purpose** — the tuition account
  receives only fees; the salary account
  pays only salaries. This is a regulatory
  requirement in some jurisdictions.
- **Reconciled monthly** — the admin matches
  the bank statement to the engine's
  records.

### Bank Reconciliation

At the end of the month, the admin reconciles
the bank statement:

1. The admin imports the bank statement (CSV
   from the bank's online portal).
2. The engine matches each statement entry to
   the engine's records (payments, expenses,
   transfers).
3. Unmatched entries are flagged for review.
4. The admin resolves the unmatched entries
   (e.g. a bank charge the school did not
   record).

The engine's `BankReconciliation` aggregate
captures the reconciliation. The `BankStatement`
entries have a `reconciled = true` flag after
matching.

### Bank Payment Slip

A school accepts bank payment slips. The parent
deposits cash at the bank with a payment slip
issued by the school. The school records the
slip:

1. The admin creates a `BankPaymentSlip` with
   the slip number, the bank, the amount, the
   student, and the deposit date.
2. The slip status is `Pending`.
3. The admin reviews the bank statement for
   the deposit; the slip status becomes
   `Approved`.
4. The `FeesPayment` is recorded against the
   slip.

### Expense Recording

The school incurs expenses:
- Salaries (covered by payroll).
- Utilities (electricity, water, internet).
- Supplies (stationery, cleaning).
- Maintenance (repairs, renovations).
- Vendor bills (catering, transport).

The accountant records an expense:

1. Selects an `ExpenseHead` (e.g. "Electricity").
2. Enters the amount, the date, the vendor
   (optional), the description, and the bank
   account.
3. The expense is approved (per the school's
   approval workflow).
4. The expense is recorded; the bank balance
   decreases.

In real schools, expense approval is:
- **Threshold-based** — expenses under a
  threshold are auto-approved; larger ones
  require principal approval.
- **Category-based** — some categories (e.g.
  "Salary") always require approval.
- **Multi-step** — large expenses may require
  multiple approvals.

### Income Recording

The school receives income that is not fees:
- Donations.
- Sale of old furniture.
- Rent from a facility.
- Investment income.
- Grant from a government body.

The accountant records the income with the
`IncomeHead`, the amount, the date, the source
(individual or organization), and the bank
account. The `Donor` aggregate captures the
donor's details for donation receipts.

### Payroll Generation

At the end of the month, the HR / finance team
generates payroll:

1. The payroll period is closed (e.g. "October
   2026").
2. The engine computes the payroll for each
   active staff member:
   - Gross salary (from the salary template).
   - Overtime (from the staff attendance).
   - Allowances (per the salary template).
   - Deductions (PF, tax, leave, advance).
   - Net salary.
3. The payroll is generated (status `Generated`).
4. The principal reviews and approves (status
   `Approved`).
5. The accountant pays (status `Paid`); the
   bank statement is recorded.

In real schools, payroll generation is:
- **Per-staff** — each staff has their own
  payroll record.
- **Configurable** — the salary template,
  deductions, and overtime rules are per
  school.
- **Reviewed before payment** — the principal
  reviews the generated payroll before
  approval.
- **Multi-step** — generation, approval,
  payment are distinct steps; the engine
  enforces the sequence.

### Payroll Payment

A payroll is paid:

1. The accountant initiates a bank transfer to
   the staff's bank account.
2. The transfer reference is recorded in the
   engine.
3. The `PayrollPaid` event is emitted.
4. The staff's payslip is generated and
   delivered (via the communication domain).
5. The HR domain updates the staff's payment
   history.

A payroll may be paid in parts (e.g. an
advance + a final payment). The engine's
`PayrollPayment` captures each payment.

### Wallet

A school may offer a prepaid wallet for parents:

1. The parent tops up the wallet (via online
   gateway or bank transfer).
2. The wallet balance is recorded in the
   engine.
3. The parent uses the wallet to pay fees.
4. The wallet balance decreases.

The wallet is useful for:
- Multiple children in the same family.
- Pre-paid tuition.
- Refunds (the school credits the wallet).

### Carry-Forward of Unpaid Balances

At the end of the term, a student has an unpaid
balance. The school carries it forward to the
next term:

1. The school configures the carry-forward
   policy (e.g. "carry forward all unpaid
   balances; add 5% interest").
2. The engine creates a `FeesCarryForward`
   record for each student with a balance.
3. The carry-forward is added to the next
   term's invoice.
4. The parent is notified.

The engine's `FeesCarryForward` is **additive**:
it adds to the existing balance, never
overwrites.

### Due-Fees Login Prevention

A school may block parent / student portal
login if fees are overdue:

1. The school configures the policy
   (e.g. "block login if balance > ₹5,000 and
   overdue by 30+ days").
2. The engine evaluates the policy at login
   time.
3. If the policy matches, the login is
   blocked; the user is redirected to a
   payment page.
4. The user is unblocked once the balance is
   cleared.

The engine's `DueFeesLoginPrevent` aggregate
captures the policy evaluation.

### Fees Reminder

The school sends reminders for upcoming due
dates:

1. The school configures the reminder
   schedule (e.g. "remind 7 days, 3 days, and
   1 day before the due date").
2. The engine's reminder worker scans
   upcoming due dates.
3. The worker sends a notification to the
   parent (SMS, email, push).
4. The reminder is recorded; the parent sees
   the history in the portal.

## Business Rules

1. Every monetary aggregate is anchored to a
   `SchoolId`.
2. A fees amount is non-negative; a discount
   amount is non-negative.
3. An invoice's `paid_amount` plus `due_amount`
   equals its `sub_total` plus `fine` plus
   `service_charge` plus any adjustments.
4. A bank account's `current_balance` is
   derived from its statements; it is never
   written directly except at account creation.
5. A payroll is generated before it is approved,
   and approved before it is paid. A
   `PayrollPaid` event is the only terminal
   transition.
6. A `FeesInstallment` can be paid only if it
   is `Active`.
7. Carry-forward never overwrites a previous
   forward for the same student in the same
   academic year; it adds to the existing
   balance.
8. A wallet transaction is `Pending` until
   approved; only `Approve` transitions credit
   the wallet.
9. A bank payment slip is `Pending` until
   approved; only approved slips contribute to
   a bank statement.
10. The "due fees login prevention" rule blocks
    logins only of the user-ids explicitly
    listed, never of staff.
11. Payroll earnings minus deductions equals
    `net_salary`; a partial payment cannot
    exceed the unpaid balance.
12. A fees discount assigned to a student
    cannot exceed the student's applicable fees
    amount.
13. Question-bank fees follow the same invoice
    and payment path as class fees; they are
    not a separate ledger.
14. An amount transfer between two accounts
    always produces two bank statements (one
    debit, one credit) in a single transaction.
15. A payment is **idempotent on the gateway
    transaction id**. A duplicate webhook from
    the gateway is a no-op.
16. A payment's date is the date the funds were
    received, not the date the cashier recorded
    it (with a configurable grace period).
17. A refund is a separate transaction; it
    creates a `Refund` record linked to the
    original payment.
18. Expense approval is threshold-based; the
    thresholds are per-school configuration.

## Edge Cases

### Partial Payment

A parent pays half the fees. The engine records
the payment; the balance is updated. The parent
receives a receipt for the partial payment. The
student's status remains `Active` (assuming
the school allows partial payments).

### Over-Payment

A parent pays more than the balance. The engine
records the payment; the over-amount is held as
credit. The parent's portal shows the credit.
The next invoice applies the credit.

### Payment Failure (Online Gateway)

A parent pays via Stripe. The payment fails at
the gateway. The gateway sends a webhook to
the engine. The engine's `PaymentFailed` event
is emitted. The parent is notified. The
invoice's `paid_amount` is unchanged.

### Payment Reversal Mid-Term

A payment was recorded in error. The cashier
reverses it. The balance restores. The parent
is notified of the reversal.

### Bank Charge Without Matching Payment

The bank charges a fee. The school's record
shows the charge; no matching expense exists.
The admin records an expense for the bank
charge.

### Cheque Bounce

A parent pays by cheque. The cheque bounces. The
cashier records the bounce. The original
payment's status becomes `Bounced`. The
parent's balance is restored; a bounce fee may
be added (per the school's policy).

### Staff Resignation Mid-Month

A staff resigns on October 15. The October
payroll is generated for the partial month
(15 days). The engine's `Payroll` computes the
pro-rata salary.

### Staff on Unpaid Leave

A staff is on unpaid leave for the entire
month. The October payroll is zero (no
earnings, no deductions). The engine records a
zero-payroll for the month.

### Carry-Forward with Multiple Terms

A student has an unpaid balance from Term 1.
The school carries it forward. In Term 2, the
student is also assigned new fees. The Term 2
invoice includes the carried-forward balance
plus the new fees.

### Multi-Currency

A school accepts payment in multiple
currencies (e.g. local + USD for international
students). The engine's `FeesPayment` carries a
`currency` field; the bank account has a
`currency` field. Conversion is at the bank's
rate on the payment date.

### Fees Refund

A student withdraws mid-term. The school
refunds the unused portion. The engine
records a `Refund` (a negative payment) linked
to the original payment. The bank statement
reflects the refund.

### Wallet Used for Partial Payment

A parent's wallet has ₹1,000. The fees balance
is ₹5,000. The parent pays ₹1,000 from the
wallet and ₹4,000 via bank. The engine records
two `FeesPayment` records (one wallet, one
bank) against the same fees assign.

### Bank Account Frozen

A school's bank account is frozen by a court
order. The admin records the freeze. The
engine rejects payments to the frozen account.
The admin opens a new bank account; the
payments redirect.

### Audit Reconciliation

A regulator audits the school. The auditor
asks for a reconciliation between the engine's
ledger and the bank statement. The engine's
`Report.Finance.Reconciliation` command
produces the reconciliation. The audit log
captures the report generation.

## Notes for Educore Implementation

- The **finance** crate is the most complex
  domain. It has the most aggregates, the most
  business rules, and the most cross-domain
  integrations.
- The finance domain is **eventually
  consistent** with academic (fees
  assignment), HR (payroll), and library
  (membership). The cross-domain coordination
  is event-driven.
- The finance domain is the most
  **integration-heavy**. The consumer's
  payment gateway, bank reconciliation, and
  tax-reporting integrations are port-driven.
- The finance domain is the most
  **performance-sensitive**. A school
  generates 1,000+ invoices at the start of a
  term and processes 500+ payments per day.
  The engine's command pipeline must be
  fast.
- The finance domain's **idempotency** is
  critical. The engine deduplicates on
  `(idempotency_key)` and on the gateway's
  transaction id.
- The finance domain's **audit** is the most
  rigorous. Every monetary change is recorded
  with the actor, the IP, the before/after.
  Regulators can reconstruct any historical
  state from the audit log.
- The finance domain's **reports** are the
  most numerous. Daily collection, monthly
  expense, bank reconciliation, payroll
  summary, donor receipt, tax filing. The
  engine's `Report.Generate` command produces
  capability-gated reports.
- The finance domain's **bulk operations**
  are common. Generate 1,000 invoices,
  generate 200 payrolls. The engine's bulk
  command is all-or-nothing.
- The finance domain's **multi-currency** is
  a per-school configuration. The engine
  reads the school's default currency;
  payments in other currencies are recorded
  with the conversion rate at the payment
  date.
- The finance domain's **fiscal year** may
  differ from the academic year. The engine
  supports both calendars.
