# Finance Domain — Permissions

Permissions are capability strings. They are not roles. The RBAC
domain maps capabilities to roles.

## Naming

```text
<Domain>.<Aggregate>.<Action>
```

The finance domain uses `Fees.*`, `Invoice.*`, `Payment.*`,
`Expense.*`, `Income.*`, `Bank.*`, `Payroll.*`, `Wallet.*`,
`Discount.*`, `Donor.*`, `DirectFees.*`, `FeesReminder.*`,
`FeesCarryForward.*`, `DueFees.*`, `Report.*`, plus a few shared
catalog capabilities.

## Capabilities

### Fees Group / Type / Master

- `FeesGroup.Create`
- `FeesGroup.Update`
- `FeesGroup.Delete`
- `FeesGroup.Read`
- `FeesType.Create`
- `FeesType.Update`
- `FeesType.Delete`
- `FeesType.Read`
- `FeesMaster.Create`
- `FeesMaster.Update`
- `FeesMaster.Delete`
- `FeesMaster.Read`

### Fees Assignment

- `FeesAssign.Create`
- `FeesAssign.Update`
- `FeesAssign.Close`
- `FeesAssign.Read`
- `FeesAssign.Discount.Update`
- `FeesInstallment.Create`
- `FeesInstallment.Update`
- `FeesInstallment.Delete`
- `FeesInstallment.Assign`
- `FeesInstallment.Read`
- `DirectFeesInstallment.Create`
- `DirectFeesInstallment.Update`
- `DirectFeesInstallment.Delete`
- `DirectFeesInstallment.Assign`
- `DirectFeesInstallment.Pay`
- `DirectFeesInstallment.Read`
- `DirectFees.Configure`

### Fees Discount

- `FeesDiscount.Create`
- `FeesDiscount.Update`
- `FeesDiscount.Delete`
- `FeesDiscount.Read`
- `FeesDiscount.Assign`

### Invoice

- `Invoice.Generate`
- `Invoice.Configure`
- `Invoice.Read`
- `Invoice.Cancel`
- `Invoice.Setting.Configure`
- `Invoice.Print`

### Payment

- `Payment.Collect`
- `Payment.Reverse`
- `Payment.Read`
- `PaymentMethod.Create`
- `PaymentMethod.Update`
- `PaymentMethod.Delete`
- `PaymentMethod.Read`
- `PaymentGateway.Configure`
- `PaymentGateway.Update`
- `PaymentGateway.Disable`
- `PaymentGateway.Read`

### Expense

- `Expense.Create`
- `Expense.Update`
- `Expense.Delete`
- `Expense.Read`
- `Expense.Approve`
- `ExpenseHead.Create`
- `ExpenseHead.Update`
- `ExpenseHead.Delete`

### Income

- `Income.Create`
- `Income.Update`
- `Income.Delete`
- `Income.Read`
- `Income.Approve`
- `IncomeHead.Create`
- `IncomeHead.Update`
- `IncomeHead.Delete`
- `Donor.Create`
- `Donor.Update`
- `Donor.Delete`
- `Donor.Read`

### Bank

- `Bank.Open`
- `Bank.Update`
- `Bank.Close`
- `Bank.Read`
- `Bank.Statement.Record`
- `Bank.Statement.Reverse`
- `Bank.Transfer`
- `BankSlip.Generate`
- `BankSlip.Approve`
- `BankSlip.Reject`
- `BankSlip.Read`

### Payroll

- `Payroll.Generate`
- `Payroll.Approve`
- `Payroll.Pay`
- `Payroll.Read`
- `PayrollPayment.Read`
- `SalaryTemplate.Create`
- `SalaryTemplate.Update`
- `SalaryTemplate.Delete`
- `SalaryTemplate.Read`
- `HourlyRate.Set`
- `HourlyRate.Read`

### Wallet

- `Wallet.Credit`
- `Wallet.Debit`
- `Wallet.Approve`
- `Wallet.Reject`
- `Wallet.Read`

### Carry Forward

- `FeesCarryForward.Execute`
- `FeesCarryForward.Configure`
- `FeesCarryForward.Read`

### Due Fees

- `DueFees.Block`
- `DueFees.Unblock`
- `DueFees.Read`
- `FeesReminder.Configure`
- `FeesReminder.Update`
- `FeesReminder.Delete`
- `FeesReminder.Read`

### Inventory & Product

- `Inventory.Payment.Record`
- `Inventory.Read`
- `Product.Purchase`
- `Product.Payment`
- `Product.Read`

### Chart of Account

- `ChartOfAccount.Create`
- `ChartOfAccount.Update`
- `ChartOfAccount.Delete`
- `ChartOfAccount.Read`

### Fees Installment Credit

- `FeesInstallmentCredit.Add`
- `FeesInstallmentCredit.Consume`
- `FeesInstallmentCredit.Cancel`
- `FeesInstallmentCredit.Read`

### Question Bank Fees

- `QuestionBank.Fee.Attach`
- `QuestionBank.Fee.Detach`
- `QuestionBank.Fee.Read`

### Reports

- `Report.FeesCollected`
- `Report.FeesOutstanding`
- `Report.FeesByClass`
- `Report.FeesByGroup`
- `Report.DiscountUsage`
- `Report.DailyCollection`
- `Report.MonthlyCollection`
- `Report.YearlyCollection`
- `Report.BankReconciliation`
- `Report.Expense`
- `Report.Income`
- `Report.IncomeVsExpense`
- `Report.ProfitLoss`
- `Report.BalanceSheet`
- `Report.CashFlow`
- `Report.PayrollRegister`
- `Report.PayrollByStaff`
- `Report.PayrollTax`
- `Report.WalletLedger`
- `Report.CarryForward`
- `Report.Ledger`
- `Report.AuditTrail`
- `Report.Finance.Read` (umbrella)

## Default Role Mapping

The platform's default role catalog binds the following:

| Role          | Capabilities (highlights)                                                     |
| ------------- | ----------------------------------------------------------------------------- |
| SuperAdmin    | All                                                                           |
| SchoolAdmin   | All within the school                                                         |
| Accountant    | Fees*, Invoice*, Payment*, Expense*, Income*, Bank*, Wallet*, Report.Finance.*|
| Cashier       | Payment.Collect, BankSlip.Generate, BankSlip.Read                             |
| Teacher       | FeesAssign.Read, Invoice.Read, Payment.Read, Payroll.Read                     |
| Student       | Invoice.Read (own), Payment.Read (own), Wallet.Read (own)                     |
| Parent        | Invoice.Read (linked), Payment.Read (linked), Wallet.Read (linked)            |
| Donor         | Donor.Read (self)                                                             |
| HR            | Payroll.Read, PayrollPayment.Read, Bank.Read                                  |

The default mapping is a starting point and is configurable per
school.

## Authorization Pattern

Capabilities are checked at the command boundary. The engine never
trusts the caller to assert their own role.

```rust
if !engine.rbac().has(actor_id, Capability::PaymentCollect).await? {
    return Err(DomainError::forbidden("missing capability"));
}
```

Some commands have a secondary ownership check: a student paying an
invoice is only allowed to pay their own invoice; an approver cannot
approve a slip they themselves generated (segregation of duties).

## Read vs Write

Read capabilities are explicit. `Invoice.Read` does not imply
`Invoice.Generate`. A consumer may grant only read-only access to a
parent or auditor.

## Tenant Isolation

Every capability check is paired with a tenant check. The actor must
be authenticated to the school that owns the target aggregate. There
is no cross-tenant capability elevation. Cross-tenant operations
(e.g. moving funds between two schools in a SaaS) require explicit
per-tenant grants.
