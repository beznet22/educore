# Finance Domain — Tables

The finance domain is backed by the following tables from
`migrations/0009_finance.sql`. Each table maps to one or more
aggregates; the `aggregate` column tells you which aggregate owns
the row.

| Table                                     | Aggregate                                | Notes                                        |
| ----------------------------------------- | ---------------------------------------- | -------------------------------------------- |
| `finance_fees_groups`                          | FeesGroup                                | Classic fees group                           |
| `finance_fees_types`                           | FeesType                                 | Classic fees type                            |
| `finance_fees_masters`                         | FeesMaster                               | Classic (class, type) master                 |
| `finance_fees_assigns`                         | FeesAssign                               | Classic per-student assignment               |
| `finance_fees_assign_discounts`                | FeesAssignDiscount                       | Classic per-student applied discount         |
| `finance_fees_discounts`                       | FeesDiscount                             | Classic discount catalog                     |
| `finance_invoice_settings`                           | FeesInvoice                              | Classic invoice numbering                    |
| `fees_invoice_settings`                   | FeesInvoiceSetting                       | Classic invoice layout                       |
| `invoice_settings`                        | InvoiceSetting                           | Newer invoice layout                         |
| `finance_invoices`                        | FmFeesInvoice                            | Newer FM invoice header                      |
| `finance_invoice_children`                 | FmFeesInvoiceChild                       | Newer FM invoice line                        |
| `finance_fees_transactions`                    | FmFeesTransaction                        | Newer FM payment transaction                 |
| `finance_fees_transcation_children`             | FmFeesTransactionChild                   | Newer FM transaction line                    |
| `finance_fees_weavers`                         | FmFeesWeaver                             | Newer FM weaver adjustment                   |
| `finance_direct_fees_installments`                | DirectFeesInstallment                    | Direct installment plan                      |
| `finance_direct_fees_installment_assigns`         | DirectFeesInstallmentAssign              | Direct installment per-student assignment    |
| `finance_direct_fees_installment_child_payments`    | DirectFeesInstallmentChildPayment        | Direct installment child payment             |
| `direct_fees_reminders`                   | DirectFeesReminder                       | Direct-fees reminder rule                    |
| `direct_fees_settings`                    | DirectFeesSetting                        | Direct-fees global setting                   |
| `finance_due_fees_login_prevents`                 | DueFeesLoginPrevent                      | Login block for overdue users                |
| `fees_carry_forward_logs`                 | FeesCarryForwardLog                      | Carry-forward audit row                      |
| `fees_carry_forward_settings`             | FeesCarryForwardSetting                  | Carry-forward configuration                  |
| `finance_fees_carry_forwards`                  | FeesCarryForward                         | Per-student carry-forward balance            |
| `fees_installment_credits`                | FeesInstallmentCredit                    | Pre-paid installment credit                  |
| `finance_fees_payments`                        | FeesPayment                              | Classic fees payment                         |
| `finance_fees_installment_assigns`                | FeesInstallmentAssign                    | Classic installment per-student assignment    |
| `finance_payment_methods`                     | PaymentMethod                            | Catalog of payment methods                   |
| `finance_payment_gateway_settings`             | PaymentGatewaySetting                    | Per-gateway credentials and mode             |
| `finance_bank_accounts`                        | BankAccount                              | Bank or cash account                         |
| `finance_bank_statements`                      | BankStatement                            | Bank statement entry                         |
| `finance_bank_payment_slips`                   | BankPaymentSlip                          | Bank/cheque payment slip                     |
| `finance_add_expenses`                         | Expense                                  | Recorded expense                             |
| `finance_add_incomes`                          | Income                                   | Recorded income                              |
| `finance_expense_heads`                        | ExpenseHead                              | Expense category                             |
| `finance_income_heads`                         | IncomeHead                               | Income category                              |
| `finance_donors`                               | Donor                                    | Donor profile                                |
| `finance_product_purchases`                    | ProductPurchase                          | Vendor product purchase                      |
| `finance_inventory_payments`                   | InventoryPayment                         | Inventory payment                            |
| `finance_wallet_transactions`                     | WalletTransaction                        | Wallet movement                              |
| `finance_transactions`                            | Transaction                              | Double-entry journal line                    |
| `finance_payroll_payments`                        | PayrollPayment                           | Payroll payment row                          |
| `hr_payroll_generates`                 | PayrollGenerate (HR, finance-reads)      | Monthly payroll run                          |
| `hr_payroll_earn_deducs`               | PayrollEarnDeduc (HR, finance-reads)     | Payroll earnings/deductions line             |
| `hr_salary_templates`                  | SalaryTemplate                           | Salary grade template                        |
| `hr_hourly_rates`                         | HourlyRateRow (HR)                       | Per-grade hourly rate                        |
| `hr_leave_deduction_infos`                | LeaveDeductionInfo (HR)                  | Leave-deduction row on payroll               |
| `assessment_question_banks`                       | QuestionBank (assessment) / QuestionBankFee (finance) | Question bank item + its fees mapping |
| `assessment_question_bank_mu_options`             | QuestionBankMuOption (assessment)        | MCQ option for a question bank item          |
| `chart_of_accounts`                       | ChartOfAccount                           | Accounting category                          |
| `fees_carry_forwards` (alt name)          | FeesCarryForward (alias)                 | Alias of `finance_fees_carry_forwards`            |

## Notes

- Every table includes `school_id` for multi-tenant isolation. The
  `school_id` is `NOT NULL DEFAULT 1` for the bootstrap school.
- Every table includes `created_at`, `updated_at`, `created_by`,
  `updated_by`, `active_status` (where applicable). These are
  managed by the engine's storage adapter.
- `academic_id` references `academic_academic_years` (the per-year scope).
- The finance domain has **one canonical table per aggregate**. The
  legacy Schoolify/InfixEdu project maintained two parallel schemes —
  the 'classic' (`sm_fees_*` prefix) and the 'FM' (`fm_fees_*` prefix)
  — to support gradual migration. The engine collapses these into
  single canonical tables named `<domain>_<aggregate>`. The
  legacy→engine mapping is in
  `docs/schemas/data-migration/03-domain-renames.md`.
- Question-bank fees live in `assessment_question_banks` (which carries the
  assessment fields) but the fees mapping (`fees_type_id`, etc.) is
  consumed by the finance domain.
- The HR-owned payroll tables (`hr_payroll_generates`,
  `hr_payroll_earn_deducs`) are typed as HR aggregates but are
  read and paid by finance. The corresponding Rust types live in
  the finance domain's value-object catalog with HR provenance
  recorded.
- The `finance_transactions` table is a double-entry journal that mirrors
  bank statements and supports polymorphic references
  (`morphable_type` / `morphable_id`).
- The `finance_due_fees_login_prevents` table is the canonical login-block
  list and is the only source of truth for the RBAC port's
  login-time check.

**Total finance tables: 52 (one per aggregate; see Coverage Matrix in build-plan.md).**
