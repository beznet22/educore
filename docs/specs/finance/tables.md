# Finance Domain — Tables

The finance domain is backed by the following tables from
`migrations/0009_finance.sql`. Each table maps to one or more
aggregates; the `aggregate` column tells you which aggregate owns
the row.

| Table                                     | Aggregate                                | Notes                                        |
| ----------------------------------------- | ---------------------------------------- | -------------------------------------------- |
| `sm_fees_groups`                          | FeesGroup                                | Classic fees group                           |
| `sm_fees_types`                           | FeesType                                 | Classic fees type                            |
| `sm_fees_masters`                         | FeesMaster                               | Classic (class, type) master                 |
| `sm_fees_assigns`                         | FeesAssign                               | Classic per-student assignment               |
| `sm_fees_assign_discounts`                | FeesAssignDiscount                       | Classic per-student applied discount         |
| `sm_fees_discounts`                       | FeesDiscount                             | Classic discount catalog                     |
| `fees_invoices`                           | FeesInvoice                              | Classic invoice numbering                    |
| `fees_invoice_settings`                   | FeesInvoiceSetting                       | Classic invoice layout                       |
| `invoice_settings`                        | InvoiceSetting                           | Newer invoice layout                         |
| `fm_fees_invoice_settings`                | FmFeesInvoiceSetting                     | Newer FM invoice numbering and positions     |
| `fm_fees_groups`                          | FmFeesGroup                              | Newer FM fees group                          |
| `fm_fees_types`                           | FmFeesType                               | Newer FM fees type                           |
| `fm_fees_invoices`                        | FmFeesInvoice                            | Newer FM invoice header                      |
| `fm_fees_invoice_chields`                 | FmFeesInvoiceChild                       | Newer FM invoice line                        |
| `fm_fees_transactions`                    | FmFeesTransaction                        | Newer FM payment transaction                 |
| `fm_fees_transaction_chields`             | FmFeesTransactionChild                   | Newer FM transaction line                    |
| `fm_fees_weavers`                         | FmFeesWeaver                             | Newer FM weaver adjustment                   |
| `direct_fees_installments`                | DirectFeesInstallment                    | Direct installment plan                      |
| `direct_fees_installment_assigns`         | DirectFeesInstallmentAssign              | Direct installment per-student assignment    |
| `dire_fees_installment_child_payments`    | DirectFeesInstallmentChildPayment        | Direct installment child payment             |
| `direct_fees_reminders`                   | DirectFeesReminder                       | Direct-fees reminder rule                    |
| `direct_fees_settings`                    | DirectFeesSetting                        | Direct-fees global setting                   |
| `due_fees_login_prevents`                 | DueFeesLoginPrevent                      | Login block for overdue users                |
| `fees_carry_forward_logs`                 | FeesCarryForwardLog                      | Carry-forward audit row                      |
| `fees_carry_forward_settings`             | FeesCarryForwardSetting                  | Carry-forward configuration                  |
| `sm_fees_carry_forwards`                  | FeesCarryForward                         | Per-student carry-forward balance            |
| `fees_installment_credits`                | FeesInstallmentCredit                    | Pre-paid installment credit                  |
| `sm_fees_payments`                        | FeesPayment                              | Classic fees payment                         |
| `fees_installment_assigns`                | FeesInstallmentAssign                    | Classic installment per-student assignment    |
| `sm_payment_methhods`                     | PaymentMethod                            | Catalog of payment methods                   |
| `sm_payment_gateway_settings`             | PaymentGatewaySetting                    | Per-gateway credentials and mode             |
| `sm_bank_accounts`                        | BankAccount                              | Bank or cash account                         |
| `sm_bank_statements`                      | BankStatement                            | Bank statement entry                         |
| `sm_bank_payment_slips`                   | BankPaymentSlip                          | Bank/cheque payment slip                     |
| `sm_add_expenses`                         | Expense                                  | Recorded expense                             |
| `sm_add_incomes`                          | Income                                   | Recorded income                              |
| `sm_expense_heads`                        | ExpenseHead                              | Expense category                             |
| `sm_income_heads`                         | IncomeHead                               | Income category                              |
| `sm_donors`                               | Donor                                    | Donor profile                                |
| `sm_product_purchases`                    | ProductPurchase                          | Vendor product purchase                      |
| `sm_inventory_payments`                   | InventoryPayment                         | Inventory payment                            |
| `wallet_transactions`                     | WalletTransaction                        | Wallet movement                              |
| `transcations`                            | Transaction                              | Double-entry journal line                    |
| `payroll_payments`                        | PayrollPayment                           | Payroll payment row                          |
| `sm_hr_payroll_generates`                 | PayrollGenerate (HR, finance-reads)      | Monthly payroll run                          |
| `sm_hr_payroll_earn_deducs`               | PayrollEarnDeduc (HR, finance-reads)     | Payroll earnings/deductions line             |
| `sm_hr_salary_templates`                  | SalaryTemplate                           | Salary grade template                        |
| `sm_hourly_rates`                         | HourlyRateRow (HR)                       | Per-grade hourly rate                        |
| `sm_leave_deduction_infos`                | LeaveDeductionInfo (HR)                  | Leave-deduction row on payroll               |
| `sm_question_banks`                       | QuestionBank (assessment) / QuestionBankFee (finance) | Question bank item + its fees mapping |
| `sm_question_bank_mu_options`             | QuestionBankMuOption (assessment)        | MCQ option for a question bank item          |
| `chart_of_accounts`                       | ChartOfAccount                           | Accounting category                          |
| `fees_carry_forwards` (alt name)          | FeesCarryForward (alias)                 | Alias of `sm_fees_carry_forwards`            |

## Notes

- Every table includes `school_id` for multi-tenant isolation. The
  `school_id` is `NOT NULL DEFAULT 1` for the bootstrap school.
- Every table includes `created_at`, `updated_at`, `created_by`,
  `updated_by`, `active_status` (where applicable). These are
  managed by the engine's storage adapter.
- `academic_id` references `sm_academic_years` (the per-year scope).
- The classic scheme (tables prefixed `sm_fees_*`) and the newer FM
  scheme (tables prefixed `fm_fees_*` and `fees_invoice*`) are
  maintained side-by-side. New consumers may use either; the engine
  treats them as parallel catalogs.
- Question-bank fees live in `sm_question_banks` (which carries the
  assessment fields) but the fees mapping (`fees_type_id`, etc.) is
  consumed by the finance domain.
- The HR-owned payroll tables (`sm_hr_payroll_generates`,
  `sm_hr_payroll_earn_deducs`) are typed as HR aggregates but are
  read and paid by finance. The corresponding Rust types live in
  the finance domain's value-object catalog with HR provenance
  recorded.
- The `transcations` table is a double-entry journal that mirrors
  bank statements and supports polymorphic references
  (`morphable_type` / `morphable_id`).
- The `due_fees_login_prevents` table is the canonical login-block
  list and is the only source of truth for the RBAC port's
  login-time check.
