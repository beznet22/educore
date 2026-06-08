# Finance Domain — Services

Domain services encapsulate business logic that does not fit cleanly
in a single aggregate. They are stateless, sync, and pure (no I/O).

## FeesMasterService

```rust
pub struct FeesMasterService;

impl FeesMasterService {
    pub fn validate_amount(amount: FeeAmount) -> Result<(), ValidationError> { ... }
    pub fn build_master(cmd: CreateFeesMasterCommand) -> Result<FeesMaster, ValidationError> { ... }
    pub fn can_delete(master: &FeesMaster, assignments: &[FeesAssign]) -> bool { ... }
    pub fn effective_amount(master: &FeesMaster, discount: Option<&FeesDiscount>) -> Amount { ... }
}
```

`effective_amount` returns the post-discount amount a student will
owe for the master.

## InvoiceGenerationService

```rust
pub struct InvoiceGenerationService;

impl InvoiceGenerationService {
    pub fn build_class_invoice(
        class_id: ClassId,
        section_id: Option<SectionId>,
        academic_id: AcademicYearId,
    ) -> Vec<FmFeesInvoiceDraft> { ... }

    pub fn build_student_invoice(
        student_id: StudentId,
        record_id: StudentRecordId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> FmFeesInvoiceDraft { ... }

    pub fn apply_weavers(draft: &mut FmFeesInvoiceDraft, weavers: &[FmFeesWeaver]) { ... }
    pub fn apply_credits(draft: &mut FmFeesInvoiceDraft, credits: &[FeesInstallmentCredit]) { ... }
    pub fn number_invoice(school: &School, draft: &FmFeesInvoiceDraft) -> InvoiceNumber { ... }
    pub fn persist(draft: FmFeesInvoiceDraft) -> Result<FmFeesInvoice, ValidationError> { ... }
}
```

The `number_invoice` step consults the school's `FmFeesInvoiceSetting`
and the running counter to produce a unique invoice number.

## PaymentService

```rust
pub struct PaymentService;

impl PaymentService {
    pub fn validate(method: &PaymentMethod, bank: Option<&BankAccount>, amount: Amount) -> Result<(), ValidationError> { ... }
    pub fn build_payment(cmd: PayInvoiceCommand, assign: &FeesAssign) -> Result<FeesPayment, ValidationError> { ... }
    pub fn apply_to_assign(payment: &FeesPayment, assign: &mut FeesAssign) -> Result<Amount, ValidationError> { ... }
    pub fn build_bank_statement(payment: &FeesPayment, account: &BankAccount) -> BankStatement { ... }
    pub fn reverse(payment: &FeesPayment, reason: String) -> (FeesPayment, BankStatement) { ... }
}
```

The service enforces that the cumulative payment never exceeds the
open balance.

## InstallmentService

```rust
pub struct InstallmentService;

impl InstallmentService {
    pub fn validate_percentages(installments: &[DirectFeesInstallment]) -> Result<(), ValidationError> { ... }
    pub fn build_assignments(installment: &DirectFeesInstallment, students: &[StudentId]) -> Vec<DirectFeesInstallmentAssign> { ... }
    pub fn build_payments(assign: &DirectFeesInstallmentAssign, cmd: &PayDirectInstallmentCommand) -> Vec<DirectFeesInstallmentChildPayment> { ... }
    pub fn next_due_date(master: &FeesMaster, settings: &DirectFeesSetting) -> NaiveDate { ... }
}
```

## CarryForwardService

```rust
pub struct CarryForwardService;

impl CarryForwardService {
    pub fn compute_balance(student: &StudentId, from: AcademicYearId) -> CarryForwardAmount { ... }
    pub fn build_carry_forward(student: StudentId, from: AcademicYearId, to: AcademicYearId) -> FeesCarryForward { ... }
    pub fn append_log(carry: &FeesCarryForward) -> FeesCarryForwardLog { ... }
    pub fn apply_to_assigns(carry: &FeesCarryForward, assigns: &mut [FeesAssign]) { ... }
    pub fn should_carry_forward(balance: CarryForwardAmount, settings: &FeesCarryForwardSetting) -> bool { ... }
}
```

## PayrollCalculationService

```rust
pub struct PayrollCalculationService;

impl PayrollCalculationService {
    pub fn build_from_template(
        staff: &Staff,
        template: &SalaryTemplate,
        period: PayPeriod,
    ) -> PayrollGenerate { ... }

    pub fn build_from_hourly(
        staff: &Staff,
        rate: &HourlyRate,
        hours: f32,
        period: PayPeriod,
    ) -> PayrollGenerate { ... }

    pub fn apply_leave_deduction(
        payroll: &mut PayrollGenerate,
        leave_info: &LeaveDeductionInfo,
    ) -> Result<(), ValidationError> { ... }

    pub fn total_earning(payroll: &PayrollGenerate) -> TotalEarning { ... }
    pub fn total_deduction(payroll: &PayrollGenerate) -> TotalDeduction { ... }
    pub fn gross(payroll: &PayrollGenerate) -> GrossSalary { ... }
    pub fn net(payroll: &PayrollGenerate) -> NetSalary { ... }
    pub fn remaining_unpaid(payroll: &PayrollGenerate, payments: &[PayrollPayment]) -> NetSalary { ... }
}
```

The service consumes the `SalaryTemplate` aggregate and a
`LeaveDeductionInfo` value to produce a `PayrollGenerate`. It also
computes cumulative paid and remaining unpaid balances.

## BankReconciliationService

```rust
pub struct BankReconciliationService;

impl BankReconciliationService {
    pub fn match_statement(stmt: &BankStatement, payments: &[FeesPayment], slips: &[BankPaymentSlip]) -> ReconciliationMatch { ... }
    pub fn build_reconciliation_report(school: SchoolId, from: NaiveDate, to: NaiveDate) -> ReconciliationReport { ... }
    pub fn unmatched_statements(school: SchoolId) -> Vec<BankStatement> { ... }
}
```

A `ReconciliationMatch` is one of `Matched { payment_id }`,
`Matched { slip_id }`, `Unmatched { reason }`.

## DiscountService

```rust
pub struct DiscountService;

impl DiscountService {
    pub fn eligibility(student: &Student, discount: &FeesDiscount) -> bool { ... }
    pub fn compute_discount(discount: &FeesDiscount, fees_amount: Amount) -> DiscountAmount { ... }
    pub fn apply(assign: &mut FeesAssign, discount: &FeesDiscount) -> Result<DiscountAmount, ValidationError> { ... }
    pub fn can_assign_once(student: &Student, discount: &FeesDiscount, year: AcademicYearId) -> bool { ... }
}
```

## WalletService

```rust
pub struct WalletService;

impl WalletService {
    pub fn balance(user: &UserId, transactions: &[WalletTransaction]) -> Amount { ... }
    pub fn validate_debit(user: &UserId, amount: Amount, transactions: &[WalletTransaction]) -> Result<(), ValidationError> { ... }
    pub fn approve(tx: &mut WalletTransaction) { ... }
    pub fn reject(tx: &mut WalletTransaction, note: String) { ... }
}
```

## InvoiceNumberingService

```rust
pub struct InvoiceNumberingService;

impl InvoiceNumberingService {
    pub fn next_number(setting: &FmFeesInvoiceSetting) -> InvoiceNumber { ... }
    pub fn build(prefix: &InvoicePrefix, seq: u64) -> InvoiceNumber { ... }
    pub fn validate_class_limit(setting: &FmFeesInvoiceSetting, class_id: ClassId) -> Result<(), ValidationError> { ... }
    pub fn validate_section_limit(setting: &FmFeesInvoiceSetting, section_id: SectionId) -> Result<(), ValidationError> { ... }
    pub fn validate_admission_limit(setting: &FmFeesInvoiceSetting, admission_no: &AdmissionNumber) -> Result<(), ValidationError> { ... }
}
```

## ReminderDispatchService

```rust
pub struct ReminderDispatchService;

impl ReminderDispatchService {
    pub fn dispatch(reminder: &DirectFeesReminder, due_date: NaiveDate) -> ReminderDispatchPlan { ... }
    pub fn recipients(reminder: &DirectFeesReminder, installment: &DirectFeesInstallment) -> Vec<UserId> { ... }
}
```

A `ReminderDispatchPlan` is a typed list of channel-target-message
triples ready for the notification port.

## BankSlipService

```rust
pub struct BankSlipService;

impl BankSlipService {
    pub fn validate_slip_hash(slip: &SlipReference, existing: &[BankPaymentSlip]) -> Result<(), ValidationError> { ... }
    pub fn approve(slip: &mut BankPaymentSlip) -> (FeesPayment, BankStatement) { ... }
    pub fn reject(slip: &mut BankPaymentSlip, reason: String) { ... }
}
```

## AccountClosingService

```rust
pub struct AccountClosingService;

impl AccountClosingService {
    pub fn can_close(account: &BankAccount, open_statements: &[BankStatement]) -> bool { ... }
    pub fn close(account: &mut BankAccount) { ... }
}
```

## ChartOfAccountService

```rust
pub struct ChartOfAccountService;

impl ChartOfAccountService {
    pub fn normal_balance(account: &ChartOfAccount) -> AccountDirection { ... }
    pub fn post(account: &mut ChartOfAccount, direction: AccountDirection, amount: Amount) { ... }
    pub fn balance_at(account: &ChartOfAccount, as_of: NaiveDate) -> Amount { ... }
}
```

## Policy: FeesAssignmentEligibility

```rust
pub struct FeesAssignmentEligibility;

impl Policy<AssignFeesToStudentCommand> for FeesAssignmentEligibility {
    type Outcome = Eligible | NotEligible { reason: &'static str };
    fn check(&self, ctx: &Context, cmd: &AssignFeesToStudentCommand) -> Outcome { ... }
}
```

The policy rejects assignment for withdrawn or graduated students.

## Policy: PaymentReversalAllowed

```rust
pub struct PaymentReversalAllowed;

impl Policy<ReversePaymentCommand> for PaymentReversalAllowed {
    type Outcome = Allowed | NotAllowed { reason: &'static str };
    fn check(&self, ctx: &Context, cmd: &ReversePaymentCommand) -> Outcome { ... }
}
```

The policy disallows reversal after the bank reconciliation window
has closed, configurable per school.

## Specification: HasOverdueFees

Used by the due-fees login prevention scan.

```rust
pub struct HasOverdueFees;

impl Specification<User> for HasOverdueFees {
    fn is_satisfied_by(&self, u: &User) -> bool { ... }
}
```

## Specification: FeesAssignableToClass

```rust
pub struct FeesAssignableToClass;

impl Specification<ClassSection> for FeesAssignableToClass {
    fn is_satisfied_by(&self, cs: &ClassSection) -> bool { ... }
}
```

## Specification: PayrollApprovable

```rust
pub struct PayrollApprovable;

impl Specification<PayrollGenerate> for PayrollApprovable {
    fn is_satisfied_by(&self, p: &PayrollGenerate) -> bool { ... }
}
```

## Cross-Domain Coordinator

A thin coordinator lives in the engine facade and orchestrates
multi-domain flows (e.g. admit + fees assignment). It is **not** a
service; it composes command calls:

```rust
pub struct FinanceCoordinator<'a> {
    engine: &'a Engine,
}

impl<'a> FinanceCoordinator<'a> {
    pub async fn pay_invoice(&self, cmd: PayInvoiceCommand) -> Result<FeesPayment, DomainError> {
        let payment = self.engine.finance().pay(cmd).await?;
        Ok(payment)
    }
}
```

Domain services are pure. Cross-domain coordination happens through
events and command composition, never through service-to-service
calls.
