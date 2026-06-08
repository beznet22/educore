# Payroll Calculation Guide

## Goal

Compute monthly payroll for every staff member using configurable
salary templates, earnings/deductions, leave deductions, and
overtime.

## Concepts

- **Staff**: an employee (teacher, accountant, principal, etc.).
- **SalaryTemplate**: a bundle of earnings and deductions for a
  staff role.
- **PayrollEarnDeduc**: a single earning or deduction line.
- **HourlyRate**: an alternative compensation model for part-time
  staff.
- **LeaveDeductionInfo**: a per-leave-type deduction rule.
- **PayrollGenerate**: a monthly payroll run.
- **PayrollPayment**: a payment against a payroll run.

## Workflow

```text
1. HR configures SalaryTemplates per role.
2. HR assigns a template to each staff member.
3. For part-time staff, HR sets an HourlyRate.
4. For leave deductions, HR configures LeaveDeductionInfo per
   leave type.
5. At month end, HR triggers PayrollGeneration.
6. The engine computes each staff member's:
   - Base salary (from template)
   - Allowances (from template)
   - Deductions (from template)
   - Leave deductions (based on approved leave)
   - Overtime (if applicable)
   - Net pay
7. The engine emits PayrollGenerated.
8. The principal/HR reviews and approves (PayrollApproved).
9. Finance records payment (PayrollPaid).
```

## Salary Template

```rust
pub struct SalaryTemplate {
    pub template_id: SalaryTemplateId,
    pub name: String,                          // "Teacher Grade A"
    pub grade_type: GradeType,                 // "Grade", "Hourly"
    pub center_school_id: SchoolId,
    pub earnings: Vec<EarnDeducLine>,          // basic + allowances
    pub deductions: Vec<EarnDeducLine>,        // tax + insurance
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
}

pub struct EarnDeducLine {
    pub name: String,                          // "Basic", "HRA", "Tax"
    pub amount_type: AmountType,               // Fixed, PercentageOfBasic
    pub amount: Decimal,
    pub is_taxable: bool,
}

pub enum AmountType {
    Fixed,
    PercentageOfBasic,
}
```

## Staff Assignment

```rust
engine.hr().assign_salary_template(AssignSalaryTemplateCommand {
    tenant,
    staff_id,
    template_id,
    effective_from: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
}).await?;
```

## Hourly Rate

```rust
engine.hr().set_hourly_rate(SetHourlyRateCommand {
    tenant,
    staff_id,
    rate_per_hour: Money::new(dec!(25.00), CurrencyCode::USD),
    effective_from: ...,
}).await?;
```

For hourly staff, the engine computes payroll as
`rate_per_hour * hours_worked` (where hours are tracked via staff
attendance).

## Leave Deduction

```rust
pub struct LeaveDeductionInfo {
    pub leave_type_id: LeaveTypeId,
    pub deduction_type: DeductionType,         // Fixed, PerDay, Unpaid
    pub amount: Decimal,
}

pub enum DeductionType {
    FixedAmount,
    PerDayOfLeave,
    UnpaidLeave,    // leave is unpaid, no separate deduction
}
```

When an employee takes unpaid leave, the engine deducts
`(basic_salary / working_days) * leave_days` from the payroll.

## Payroll Generation

```rust
engine.hr().generate_payroll(GeneratePayrollCommand {
    tenant,
    period: PayPeriod { year: 2026, month: 6 },
    staff_ids: None,                           // all staff
    generated_by: hr_id,
}).await?;
```

The engine computes per staff:

```text
gross = basic + sum(allowances)
leave_deduction = compute_leave_deduction(staff, period)
deductions = sum(template_deductions) + leave_deduction
net = gross - deductions
```

The engine creates a `PayrollGenerate` aggregate per staff and emits
`PayrollGenerated`.

## Approval

```rust
engine.hr().approve_payroll(ApprovePayrollCommand {
    tenant,
    payroll_run_id,
    approved_by: principal_id,
}).await?;
```

The engine emits `PayrollApproved`. The payroll is now read-only.

## Payment

```rust
engine.finance().record_payroll_payment(RecordPayrollPaymentCommand {
    tenant,
    payroll_id,
    amount: ...,
    method: PaymentMethodKind::Bank,
    bank_id: ...,
    reference: ...,
}).await?;
```

The engine emits `PayrollPaid`.

## Edge Cases

- **New staff mid-month**: prorated salary.
- **Resigned staff**: final settlement including unused leave, prorated
  salary, and gratuity.
- **Overlapping leave and holidays**: leave is not deducted for
  holidays.
- **Negative net pay**: rare, but allowed. The engine flags it for
  HR review.
- **Currency**: each school has one currency. Cross-currency
  payroll is not supported in v1.

## Reports

- **Payroll register**: all staff, all earnings/deductions, all
  net amounts, for a period.
- **Bank advice**: per-bank, list of staff to credit, amounts.
- **Tax report**: total tax deducted per period.
- **Leave deduction report**: leave taken and deductions applied.

## Worked Example

A school runs June 2026 payroll:

```rust
// 1. Generate
let run = engine.hr().generate_payroll(GeneratePayrollCommand {
    tenant,
    period: PayPeriod { year: 2026, month: 6 },
    staff_ids: None,
    generated_by: hr_id,
}).await?;

// 2. Review (read-only report)
let report = engine.hr().payroll_register(PayrollRegisterQuery {
    tenant,
    period: ...,
}).await?;

// 3. Approve
engine.hr().approve_payroll(ApprovePayrollCommand {
    tenant,
    payroll_run_id: run.id,
    approved_by: principal_id,
}).await?;

// 4. Pay
for staff in run.payslips {
    engine.finance().record_payroll_payment(RecordPayrollPaymentCommand {
        tenant,
        payroll_id: staff.payroll_id,
        amount: staff.net,
        method: PaymentMethodKind::Bank,
        bank_id: ...,
        reference: ...,
    }).await?;
}
```

## Audit

Every payroll generation, approval, and payment is audited with full
calculations. Auditors can reconstruct the payslip for any period.
