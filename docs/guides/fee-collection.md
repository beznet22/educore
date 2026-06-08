# Fee Collection Guide

## Goal

Implement a complete fee collection flow: configure a fees master,
assign fees to a class, generate invoices, collect payments, and
produce a collection report.

## Concepts

- **FeesGroup**: a logical grouping of fees (e.g. "Tuition",
  "Transport", "Exam").
- **FeesType**: a specific fee within a group (e.g. "Tuition Q1",
  "Tuition Q2").
- **FeesMaster**: a bundle of fees types for a class-section in a
  term.
- **FeesAssign**: a per-student assignment that creates invoiceable
  fees.
- **FeesInvoice**: a bill for a student.
- **FeesInstallment**: a scheduled payment within an invoice.
- **FeesPayment**: a recorded payment against an invoice.
- **FeesDiscount**: a discount applied to a student.
- **FeesCarryForward**: a balance carried from a previous term.

## Workflow

```text
1. Configure FeesGroups and FeesTypes.
2. Build a FeesMaster for a class-section in a term.
3. Assign the FeesMaster to students in the class-section.
4. The system generates FeesInvoices per student.
5. The system schedules FeesInstallments per invoice.
6. The system sends due-date reminders.
7. The student or cashier records a FeesPayment.
8. The system updates the invoice and emits PaymentReceived.
9. The system produces a collection report.
10. At term end, carry forward unpaid balances.
```

## Setup

```rust
// 1. Group
engine.fees().create_group(CreateFeesGroupCommand {
    tenant,
    name: "Tuition".into(),
    description: Some("Quarterly tuition".into()),
}).await?;

// 2. Type
let tuition_q1 = engine.fees().create_type(CreateFeesTypeCommand {
    tenant,
    group_id,
    name: "Tuition Q1".into(),
    amount: Money::new(dec!(500.00), CurrencyCode::USD),
}).await?;

// 3. Master
let master = engine.fees().create_master(CreateFeesMasterCommand {
    tenant,
    name: "Tuition Q1 2026".into(),
    class_id,
    section_id,
    academic_year_id,
    fees_type_ids: vec![tuition_q1],
    due_date: NaiveDate::from_ymd_opt(2026, 4, 15).unwrap(),
}).await?;

// 4. Assign to students
engine.fees().assign_to_class(AssignFeesToClassCommand {
    tenant,
    fees_master_id: master.id,
    class_id,
    section_id,
}).await?;
```

## Invoice Generation

When fees are assigned, the system auto-generates invoices:

```rust
let invoices = engine.fees().generate_invoices(GenerateInvoicesCommand {
    tenant,
    fees_assign_id,
    due_date: NaiveDate::from_ymd_opt(2026, 4, 15).unwrap(),
}).await?;
```

Each invoice has a unique `invoice_number` and may have one or more
installments.

## Discount

A student may be eligible for a discount (sibling, scholarship,
staff child):

```rust
engine.fees().assign_discount(AssignDiscountCommand {
    tenant,
    student_id,
    fees_assign_id,
    discount_id: sibling_discount_id,
    amount: Money::new(dec!(50.00), CurrencyCode::USD),
}).await?;
```

The discount is applied to the invoice on generation.

## Payment

```rust
engine.fees().record_payment(RecordPaymentCommand {
    tenant,
    invoice_id,
    amount: Money::new(dec!(450.00), CurrencyCode::USD),
    method: PaymentMethodKind::Cash,
    received_by: cashier_id,
    bank_id: None,
    note: None,
}).await?;
```

The engine emits `PaymentReceived` and updates the invoice balance.

## Installments

A school may split an invoice into installments:

```rust
engine.fees().configure_installments(ConfigureInstallmentsCommand {
    tenant,
    invoice_id,
    installments: vec![
        Installment { due_date: ..., amount: ... },
        Installment { due_date: ..., amount: ... },
    ],
}).await?;
```

The system sends a reminder N days before each installment due date
(configured via `direct_fees_reminders`).

## Carry Forward

At the end of a term, unpaid balances are carried forward:

```rust
engine.fees().carry_forward(CarryForwardCommand {
    tenant,
    from_academic_year_id,
    to_academic_year_id,
    log_note: "End of term 2025-2026".into(),
}).await?;
```

The system creates a `FeesCarryForwardLog` entry per student and
emits `FeesCarriedForward` events.

## Due Fees Login Prevention

A school may configure that students with overdue fees cannot log in
to the parent portal:

```rust
engine.fees().block_login_for_due(BlockLoginForDueCommand {
    tenant,
    user_id,
    role_id: parent_role_id,
    school_id: tenant.school_id,
}).await?;
```

The engine emits `DueFeesLoginPrevented`.

## Reports

- **Collection report**: by date range, by class, by fees type.
- **Outstanding report**: invoices with balance > 0.
- **Discount report**: total discount applied per category.
- **Cashier report**: payments collected by cashier.
- **Carry-forward report**: balances moved from one term to the next.

## Edge Cases

- **Partial payment**: recorded as a partial payment; invoice
  remains open.
- **Overpayment**: recorded; engine emits `PaymentOverreceived` and
  the consumer may issue a refund or a credit.
- **Refund**: requires `FinanceRefund` capability; the engine
  records a refund against the original payment.
- **Bank slip**: see `payments.md`.

## Audit

Every fees operation is audited with full before/after snapshots.
Auditors can reconstruct every change to every invoice.

## Worked Example

A school admin configures fees for the new term and collects the
first batch of payments:

```rust
let engine = ...;  // configured
let tenant = ...;  // school admin

// Setup
let group = engine.fees().create_group(...).await?;
let types = vec![
    engine.fees().create_type(...).await?,
    engine.fees().create_type(...).await?,
];
let master = engine.fees().create_master(...).await?;

// Generate
engine.fees().assign_to_class(...).await?;
engine.fees().generate_invoices(...).await?;

// Collect
for payment in payments {
    engine.fees().record_payment(payment).await?;
}

// Report
let report = engine.fees().report_collection(CollectionReportQuery {
    tenant,
    from: ...,
    to: ...,
    class_id: Some(class_id),
}).await?;
```

The engine enforces: unique admission number, no duplicate fees
master for a (class, term), invoice numbering, balance integrity,
discount limits, and refund caps.
