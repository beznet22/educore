# Finance Domain — Workflows

Workflows orchestrate commands, queries, and policies to fulfill a
business goal. They are documented as ordered, conditional steps.

## Fees Master Setup

```text
1. SchoolAdmin creates FeesGroups (Tuition, Exam, Transport).
2. SchoolAdmin creates FeesTypes inside each group (e.g. Tuition →
   Monthly Tuition, Admission Tuition, Late Fee).
3. SchoolAdmin creates FeesMaster per (class, section, fees_type) for
   the current academic year, with the per-class amount and due date.
4. Optional: SchoolAdmin creates FeesDiscounts (sibling, scholarship,
   staff-child) with discount type (once or year) and amount.
5. Optional: SchoolAdmin configures the InvoiceSetting layout.
6. Optional: SchoolAdmin configures the invoice numbering
   (prefix, start_form).
7. Optional: SchoolAdmin configures DirectFeesSetting and
   DirectFeesReminder.
8. Optional: SchoolAdmin configures FeesCarryForwardSetting
   (fees_due_days, payment_gateway).
9. Optional: SchoolAdmin configures payment gateways and payment
   methods.
```

**Pre-conditions:**
- The current academic year is open.
- The school has at least one bank account opened.

**Failure paths:**
- Duplicate fees group name → `ValidationError::UniqueViolation`.
- Amount out of range → `ValidationError::OutOfRange`.
- Missing payment method for the chosen gateway →
  `ValidationError::Inconsistent`.

## Fees Assignment

```text
1. SchoolAdmin triggers AssignFeesToClass with a FeesMaster and a
   class/section scope.
2. The system creates one FeesAssign per active Student in the scope.
3. The system applies the default FeesDiscount if one is configured
   on the master.
4. The system creates FeesInstallment rows (if any) and corresponding
   per-student FeesInstallmentAssign rows.
5. Communication receives FeesAssignedToClass and schedules a notice
   to each guardian.
```

**Edge cases:**
- A student with a `Suspended` status still receives an assignment.
- A student with a `Withdrawn` status in the source year is skipped.
- A student who already has a `FeesAssign` for the same master and
  year is skipped (idempotent).

## Invoice Generation (Per Class)

```text
1. SchoolAdmin triggers GenerateFmFeesInvoice for a class and a
   due date.
2. The system computes each student's total from open
   FeesAssign and FeesInstallmentAssign records.
3. The system creates an FmFeesInvoice per student with one
   FmFeesInvoiceChild per assigned type.
4. The system applies FmFeesWeaver (if any) and
   FeesInstallmentCredit (if any) per line.
5. The system emits FmFeesInvoiceGenerated and sends a notification.
```

## Invoice Generation (Per Student)

```text
1. SchoolAdmin selects a student and a date range.
2. The system opens all active FeesAssign and
   FeesInstallmentAssign records and builds a draft invoice.
3. SchoolAdmin may add an FmFeesWeaver adjustment per line.
4. SchoolAdmin confirms; the system persists the invoice.
```

## Invoice Generation (Per Term)

```text
1. SchoolAdmin defines the term (start, end) and a due date.
2. The system creates one invoice per student in the class with
   amounts due in that term.
3. Direct fees installments (if any) are split across the term.
```

## Payment Collection (Cash)

```text
1. Cashier opens a student's invoice.
2. Cashier selects "Cash" as the payment method.
3. Cashier enters the amount and any fine, discount, or weaver.
4. System records the payment (PaymentReceived), updates the bank
   account's cash balance via a BankStatement, and prints a receipt.
5. Communication sends a receipt notification.
6. Wallet, if any, is optionally credited with change.
```

## Payment Collection (Bank Slip)

```text
1. Parent submits a slip through the portal, calling
   GenerateBankPaymentSlip with the scanned image.
2. The slip is in `pending` status.
3. Accountant reviews the slip and calls ApproveBankPayment (or
   RejectBankPayment).
4. On approval, the system creates a BankStatement (debit on the bank
   account), creates a FeesPayment, and applies the payment to the
   invoice. Communication sends a receipt notification.
```

## Payment Collection (Gateway)

```text
1. Parent selects a gateway-backed method on the portal.
2. The portal issues PayInvoice with the gateway's transaction id
   and slip.
3. The engine persists the payment with the gateway method.
4. The engine emits PaymentReceived and a BankStatement (debit on
   the gateway-clearing account).
5. The portal polls or receives a webhook from the gateway for
   confirmation; on failure, the engine issues a reversal.
```

## Direct Fees Installments

```text
1. SchoolAdmin configures DirectFeesSetting
   (no_installment, due_date_from_sem, end_day).
2. SchoolAdmin creates DirectFeesInstallment rows per FeesMaster
   with percentage and due_date.
3. The system creates DirectFeesInstallmentAssign rows for students
   in scope.
4. SchoolAdmin (or student) may pay an installment via
   PayDirectInstallment.
5. The system emits DirectInstallmentPaymentRecorded and updates
   the bank statement and the parent invoice.
6. The reminder job (consuming DirectFeesReminder) dispatches
   notifications `N` days before each due date.
```

## Carry Forward

```text
1. At the configured FeesCarryForwardSetting trigger date, the
   system scans for students with open balances.
2. The system emits CarryForwardFeesBalance per student to move
   the balance to the target academic year.
3. A FeesCarryForwardLog row is appended for audit.
4. Communication receives FeesCarriedForward and notifies the
   guardian.
5. If the new year's FeesAssigns already exist, the carried-forward
   balance is added to the student's running balance.
```

## Due Fees Login Prevention

```text
1. The system periodically scans for users (parents, students) with
   a non-zero overdue balance.
2. For each such user, the system emits BlockLoginForDueFees.
3. RBAC subscribes and blocks the user at the authentication port
   for the role.
4. When the user pays in full, the system emits
   UnblockLoginForDueFees and the user is restored.
5. Staff users are never blocked.
```

## Expense Recording

```text
1. Accountant records an expense via RecordExpense with the head,
   amount, account, payment method, and an optional file.
2. The system creates a BankStatement (debit) on the chosen account
   and an Expense row.
3. If the expense is for a payroll, PayrollPaymentRecorded subscribes
   and an Expense is automatically created.
4. The system emits ExpenseRecorded and updates the
   ChartOfAccount balance.
```

## Income Recording

```text
1. Accountant records an income via RecordIncome with the head,
   amount, account, payment method, and an optional file.
2. The system creates a BankStatement (credit) on the chosen account
   and an Income row.
3. The system emits IncomeRecorded.
4. If the income is for a fees payment, the system auto-creates
   the Income row from PaymentReceived and links it.
5. If the income is for an installment payment, the system links it
   to the installment.
```

## Donor Lifecycle

```text
1. SchoolAdmin registers a donor with name, email, mobile,
   addresses, photo, and a `show_public` flag.
2. The donor appears in the public donor list if `show_public`.
3. SchoolAdmin can update the donor at any time.
4. Donations are recorded as Income with the
   "Donations" income head and a reference to the donor.
```

## Bank Reconciliation

```text
1. Accountant imports the bank statement file (CSV/MT940) via the
   ingestion port.
2. The system creates a BankStatement for each row.
3. The system matches each BankStatement against outstanding
   FeesPayments, BankPaymentSlips, Expenses, and Incomes.
4. Unmatched statements surface in a reconciliation report.
5. The accountant creates a manual adjustment via
   RecordBankStatement if needed.
```

## Payroll Generation

```text
1. HR runs a scheduled job at month-end.
2. The job reads the staff's salary template, hourly rate,
   attendance, and leave records.
3. The job builds the PayrollGenerate with all earnings and
   deductions.
4. The job emits PayrollGenerated and PayrollEarningAdded /
   PayrollDeductionAdded.
5. The school admin reviews the payroll register and triggers
   ApprovePayroll.
```

## Payroll Approval

```text
1. SchoolAdmin reviews the payroll register.
2. SchoolAdmin triggers ApprovePayroll per row.
3. The system emits PayrollApproved.
4. The system locks the earnings and deductions; further changes
   require a reversal.
```

## Payroll Disbursement

```text
1. Accountant triggers RecordPayrollPayment with the amount,
   payment method, bank account, and date.
2. The system records the payment, creates a BankStatement, and
   creates an Expense on the salary expense head.
3. When the cumulative paid amount equals net_salary, the system
   emits PayrollPaid and the payroll is closed.
4. The system emits a payment receipt.
5. The LeaveDeductionInfo line (if any) is included in the
   PayrollEarnDeduc and reduces the net salary.
```

## Hourly Rate Management

```text
1. HR configures a per-grade hourly rate via SetHourlyRate.
2. The system stores the rate for use during payroll generation.
3. Hourly-based staff have their gross salary computed as
   (hours worked) * (rate) plus any fixed components.
4. The rate is read by the payroll service but not mutated by it.
```

## Salary Template

```text
1. SchoolAdmin defines a SalaryTemplate (grade, basic, house rent,
   provident fund, gross, total deduction, net).
2. The system validates that gross equals basic + house rent +
   provident fund and net equals gross - total deduction.
3. The template is assigned to a staff by HR.
4. When PayrollGenerate is created, the template's components are
   pre-filled; the payroll service can override per-staff.
```

## Wallet Credit

```text
1. SchoolAdmin (or system) issues AddWalletCredit for a user.
2. The system creates a WalletTransaction in `pending` state and a
   matching BankStatement.
3. Approver reviews and triggers ApproveWalletTransaction.
4. The system credits the wallet and emits WalletTransactionApproved.
5. The user may use the wallet to pay an invoice via a gateway-
   backed method that supports wallet deduction.
```

## Wallet Refund

```text
1. SchoolAdmin triggers AddWalletCredit with wallet_type `refund`.
2. The system creates a pending WalletTransaction.
3. Approver approves; the wallet is credited.
4. Communication notifies the user of the refund.
```

## Wallet Debit (Expense / Fees Refund)

```text
1. SchoolAdmin triggers DeductWalletCredit with a wallet_type of
   `expense` or `fees_refund` and an optional reference id.
2. The system validates the user has sufficient balance.
3. The system creates a WalletTransaction in `pending` state.
4. Approver approves; the wallet is debited and a corresponding
   Expense or FeesRefund is created.
5. The system emits WalletDebited.
```

## Idempotency

- `PayInvoice` is idempotent on `(fees_assign_id, transaction_id)`.
  A duplicate payment with the same transaction id is a no-op
  success.
- `RecordPayrollPayment` is idempotent on
  `(payroll_generate_id, payment_date, amount, reference)`.
- `BankPaymentSlip` is unique by `(student_id, assign_id, slip_hash)`.
- `CarryForwardFeesBalance` is idempotent on
  `(student_id, from_academic_id, to_academic_id)`; re-issuing
  returns the existing record.
- `BlockLoginForDueFees` is idempotent on `(user_id, role_id)`.
- `AddWalletCredit` is idempotent on
  `(user_id, amount, payment_method, reference)`.
- `TransferFunds` is idempotent on
  `(from_bank_id, to_bank_id, amount, transfer_date, note)`.

## Cross-Workflow Order

The finance domain observes the following order to keep state
coherent:

1. `StudentAdmitted` (academic) → `FeesAssign` is created.
2. `StudentPromoted` (academic) → prior balance is closed, new
   `FeesAssign` is created in the new year, carry-forward is
   applied.
3. `StudentWithdrawn` (academic) → open `FeesAssign` is closed;
   unpaid balance becomes a carry-forward or a refund, per policy.
4. `StaffRegistered` (HR) → `SalaryTemplate` is bound to the staff;
   payroll becomes available.
5. `LeaveApproved` (HR) → on payroll generation, the extra-leave
   deduction is computed.

The finance domain never calls the academic or HR domains
directly. It only subscribes to their events and reacts through
its own commands.
