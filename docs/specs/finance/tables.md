# Finance Domain — Tables

The finance domain is backed by the following tables from
`migrations/0009_finance.sql`. Each table maps to one or more
aggregates; the `aggregate` column tells you which aggregate owns
the row.

| Table                                     | Aggregate                                | Notes                                        |
| ----------------------------------------- | ---------------------------------------- | -------------------------------------------- |
| `finance_fees_groups`                          | FeesGroup                                | Classic fees group                           | <!-- derive_skip -->
| `finance_fees_types`                           | FeesType                                 | Classic fees type                            | <!-- derive_skip -->
| `finance_fees_masters`                         | FeesMaster                               | Classic (class, type) master                 | <!-- derive_skip -->
| `finance_fees_assigns`                         | FeesAssign                               | Classic per-student assignment               | <!-- derive_skip -->
| `finance_fees_assign_discounts`                | FeesAssignDiscount                       | Classic per-student applied discount         | <!-- derive_skip -->
| `finance_fees_discounts`                       | FeesDiscount                             | Classic discount catalog                     | <!-- derive_skip -->
| `finance_invoice_settings`                           | FeesInvoice                              | Classic invoice numbering                    | <!-- derive_skip -->
| `fees_invoice_settings`                   | FeesInvoiceSetting                       | Classic invoice layout                       | <!-- derive_skip -->
| `invoice_settings`                        | InvoiceSetting                           | Newer invoice layout                         | <!-- derive_skip -->
| `finance_invoices`                        | FmFeesInvoice                            | Newer FM invoice header                      | <!-- derive_skip -->
| `finance_invoice_children`                 | FmFeesInvoiceChild                       | Newer FM invoice line                        | <!-- derive_skip -->
| `finance_fees_transactions`                    | FmFeesTransaction                        | Newer FM payment transaction                 | <!-- derive_skip -->
| `finance_fees_transcation_children`             | FmFeesTransactionChild                   | Newer FM transaction line                    | <!-- derive_skip -->
| `finance_fees_weavers`                         | FmFeesWeaver                             | Newer FM weaver adjustment                   | <!-- derive_skip -->
| `finance_direct_fees_installments`                | DirectFeesInstallment                    | Direct installment plan                      | <!-- derive_skip -->
| `finance_direct_fees_installment_assigns`         | DirectFeesInstallmentAssign              | Direct installment per-student assignment    | <!-- derive_skip -->
| `finance_direct_fees_installment_child_payments`    | DirectFeesInstallmentChildPayment        | Direct installment child payment             | <!-- derive_skip -->
| `direct_fees_reminders`                   | DirectFeesReminder                       | Direct-fees reminder rule                    | <!-- derive_skip -->
| `direct_fees_settings`                    | DirectFeesSetting                        | Direct-fees global setting                   | <!-- derive_skip -->
| `finance_due_fees_login_prevents`                 | DueFeesLoginPrevent                      | Login block for overdue users                | <!-- derive_skip -->
| `fees_carry_forward_logs`                 | FeesCarryForwardLog                      | Carry-forward audit row                      | <!-- derive_skip -->
| `fees_carry_forward_settings`             | FeesCarryForwardSetting                  | Carry-forward configuration                  | <!-- derive_skip -->
| `finance_fees_carry_forwards`                  | FeesCarryForward                         | Per-student carry-forward balance            | <!-- derive_skip -->
| `fees_installment_credits`                | FeesInstallmentCredit                    | Pre-paid installment credit                  | <!-- derive_skip -->
| `finance_fees_payments`                        | FeesPayment                              | Classic fees payment                         | <!-- derive_skip -->
| `finance_fees_installment_assigns`                | FeesInstallmentAssign                    | Classic installment per-student assignment    | <!-- derive_skip -->
| `finance_payment_methods`                     | PaymentMethod                            | Catalog of payment methods                   | <!-- derive_skip -->
| `finance_payment_gateway_settings`             | PaymentGatewaySetting                    | Per-gateway credentials and mode             | <!-- derive_skip -->
| `finance_bank_accounts`                        | BankAccount                              | Bank or cash account                         | <!-- derive_skip -->
| `finance_bank_statements`                      | BankStatement                            | Bank statement entry                         | <!-- derive_skip -->
| `finance_bank_payment_slips`                   | BankPaymentSlip                          | Bank/cheque payment slip                     | <!-- derive_skip -->
| `finance_add_expenses`                         | Expense                                  | Recorded expense                             | <!-- derive_skip -->
| `finance_add_incomes`                          | Income                                   | Recorded income                              | <!-- derive_skip -->
| `finance_expense_heads`                        | ExpenseHead                              | Expense category                             | <!-- derive_skip -->
| `finance_income_heads`                         | IncomeHead                               | Income category                              | <!-- derive_skip -->
| `finance_donors`                               | Donor                                    | Donor profile                                | <!-- derive_skip -->
| `finance_product_purchases`                    | ProductPurchase                          | Vendor product purchase                      | <!-- derive_skip -->
| `finance_inventory_payments`                   | InventoryPayment                         | Inventory payment                            | <!-- derive_skip -->
| `finance_wallet_transactions`                     | WalletTransaction                        | Wallet movement                              | <!-- derive_skip -->
| `finance_transactions`                            | Transaction                              | Double-entry journal line                    | <!-- derive_skip -->
| `finance_payroll_payments`                        | PayrollPayment                           | Payroll payment row                          | <!-- derive_skip -->
| `hr_payroll_generates`                 | PayrollGenerate (HR, finance-reads)      | Monthly payroll run                          | <!-- derive_skip -->
| `hr_payroll_earn_deducs`               | PayrollEarnDeduc (HR, finance-reads)     | Payroll earnings/deductions line             | <!-- derive_skip -->
| `hr_salary_templates`                  | SalaryTemplate                           | Salary grade template                        | <!-- derive_skip -->
| `hr_hourly_rates`                         | HourlyRateRow (HR)                       | Per-grade hourly rate                        | <!-- derive_skip -->
| `hr_leave_deduction_infos`                | LeaveDeductionInfo (HR)                  | Leave-deduction row on payroll               | <!-- derive_skip -->
| `assessment_question_banks`                       | QuestionBank (assessment) / QuestionBankFee (finance) | Question bank item + its fees mapping | <!-- derive_skip -->
| `assessment_question_bank_mu_options`             | QuestionBankMuOption (assessment)        | MCQ option for a question bank item          | <!-- derive_skip -->
| `chart_of_accounts`                       | ChartOfAccount                           | Accounting category                          | <!-- derive_skip -->
| `fees_carry_forwards` (alt name)          | FeesCarryForward (alias)                 | Alias of `finance_fees_carry_forwards`            | <!-- derive_skip -->

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
