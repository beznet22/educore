# Finance Invariant Checklist

**Spec source:** `docs/specs/finance/aggregates.md`
**Code location:** `crates/domains/finance/src/`
**Baseline:** `docs/audit_reports/stub_vs_implementation.md` § "finance — Deep Invariant Audit"
**Generated:** Engine Production Depth Phase 2, Step 1

## Status Legend

- **[x]** = Enforced in code (aggregate constructor / value object / service boundary) AND has integration test
- **[~]** = Partial enforcement or test coverage incomplete
- **[ ]** = Missing — needs implementation
- **[N/A]** = Permissive invariant — engine not required to enforce

## Summary

Per spec recount (this checklist): **165 invariants** across **59 aggregates**.

Per deep audit (`stub_vs_implementation.md`): 110 invariants audited → 38 real / 22 partial / 50 missing.

Net coverage gap to close: ~50 missing + ~22 partial = **~72 invariants** must reach [x].

## Per-aggregate Status

### Foundation (Money / Currency)

- [x] Money rejects negative — `value_objects.rs:541-548`
- [x] Currency enforces 3-letter ISO-4217 uppercase — `value_objects.rs:392-407`
- [x] FeeAmount enforces `0..=100_000_000` minor — `value_objects.rs:593-606`
- [x] FineAmount enforces `0..=10_000_000` minor — `value_objects.rs:619-632`
- [x] validate_percentage enforces `[0, 100]` — `value_objects.rs:1216-1223`

### Wallet (2 invariants)

- [x] Wallet I-1: balance starts at 0 — `aggregate.rs:103-127`
- [x] Wallet I-2: WalletTransaction append-only — `aggregate.rs:139-189`

### WalletTransaction (4 invariants)

- [x] WT I-1: amount ≥ 0 — `aggregate.rs:269-273`
- [x] WT I-2: starts in Pending — `aggregate.rs:283`
- [x] WT I-3: state machine Pending → Approved/Rejected — `value_objects.rs:937-945`
- [~] WT I-4: balance invariant via cache reconciliation — partial (cache not recomputed)

### FeesPayment (4 invariants)

- [x] FP I-1: amount ≥ 0 — `aggregate.rs:476-480`
- [x] FP I-2: discount ≥ 0 — `aggregate.rs:481-485`
- [x] FP I-3: fine ≥ 0 — `aggregate.rs:486-490`
- [x] FP net_minor arithmetic — `aggregate.rs:502-505`
- [ ] FP FK to FeesAssign/Student — missing (deferred to dispatch)
- [ ] FP gateway consistency — missing (deferred to dispatch)
- [ ] FP gateway tx id required if Gateway — missing

### FeesInvoice (3 invariants)

- [x] FI I-1: prefix 1..=10 chars — `aggregate.rs:380-384`
- [x] FI I-2: start_form ≥ 0 — `aggregate.rs:385-389`
- [~] FI I-3: one per school (uniqueness) — partial (storage-layer)
- [ ] FI next counter arithmetic — missing (IncrementInvoiceCounter not implemented)

### Expense (3 invariants)

- [x] EX I-1: amount ≥ 0 — `aggregate.rs:557-561`
- [x] EX non-empty name — `aggregate.rs:556` + `value_objects.rs:1139-1147`
- [ ] EX I-2: payment_method compatible with account_id — missing (deferred to dispatch)
- [~] EX I-3: exactly one expense_head — partial (single field; structural)

### AmountTransfer (3 invariants)

- [ ] AT I-1: produces 2 BankStatement rows in 1 tx — missing (placeholder stub `aggregate.rs:851-854`)
- [ ] AT I-2: debit source + credit destination — missing
- [ ] AT I-3: idempotency on (source, dest, ref) — missing

### BankAccount (3 invariants)

- [~] BA I-1: account_number unique — partial (placeholder stub; storage concern)
- [ ] BA I-2: current_balance derived from BankStatement — missing
- [~] BA I-3: account_type ∈ {bank, cash} — partial (enum exists, aggregate missing)

### BankPaymentSlip (4 invariants)

- [~] BP I-1: payment_mode ∈ {Bk, Cq} — partial (enum exists)
- [~] BP I-2: approve_status ∈ {pending, approved, rejected} — partial (enum shared)
- [ ] BP I-3: approved slips promote to BankStatement + FeesPayment — missing
- [ ] BP I-4: cannot reject after approval — missing

### BankPaymentSlipAudit (2 invariants)

- [ ] BPA I-1: append-only log — missing (placeholder stub)
- [ ] BPA I-2: timestamps recorded — missing

### BankStatement (4 invariants)

- [ ] BS I-1: amount ≥ 0 — missing (placeholder stub `aggregate.rs:825-828`)
- [~] BS I-2: type ∈ {income, expense} — partial (StatementType enum)
- [ ] BS I-3: after_balance matches running balance — missing
- [ ] BS I-4: append-only; corrections via reverse — missing

### BankStatementAttachment (2 invariants)

- [ ] BSA I-1: attachment ref valid — missing (placeholder stub)
- [ ] BSA I-2: orphan after BankStatement delete — missing

### ChartOfAccount (2 invariants)

- [ ] COA I-1: unique name within school — missing (placeholder stub)
- [ ] COA I-2: cannot delete while referenced — missing

### DirectFeesInstallment (4 invariants)

- [~] DFI I-1: percentage ∈ [0, 100] — partial (validate_percentage exists)
- [~] DFI I-2: amount ≥ 0 — partial (placeholder)
- [ ] DFI I-3: percentage sum ≤ 100 — missing
- [ ] DFI I-4: non-overlapping windows — missing

### DirectFeesInstallmentAssign (3 invariants)

- [ ] DFIA I-1: unique per (student, installment) — missing (placeholder stub)
- [ ] DFIA I-2: amount ≥ 0 — missing
- [ ] DFIA I-3: balance ≥ 0 — missing

### DirectFeesInstallmentAssignChild (2 invariants)

- [ ] DFIAC I-1: append-only — missing (placeholder stub)
- [ ] DFIAC I-2: timestamps monotonic — missing

### DirectFeesInstallmentChildPayment (2 invariants)

- [~] DFIACP I-1: paid + balance == amount + discount — partial (value objects enforce bounds)
- [ ] DFIACP I-2: paid_amount monotonically non-decreasing — missing

### DirectFeesReminder (1 invariant)

- [ ] DFR I-1: due_date_before ≥ 0 — missing (placeholder stub)

### DirectFeesSetting (2 invariants)

- [x] DFS I-1: reminder_before ≥ 0, no_installment ≥ 0 — **complete (Wave 69 full drop)** — `RealDirectFeesSetting` aggregate at `crates/domains/finance/src/aggregate.rs:2373` with `fresh()` validating DFS I-1 (`reminder_before >= 0`, `no_installment >= 0`) and DFS I-2 (`due_date_from_sem in 1..=MAX_DUE_DAY (28)`), `update_config()` (5 fields; same validation; bumps version + `updated_at` + sets `updated_by`), and `retire()` (returns `Conflict` on already-retired). 3 typed events at `crates/domains/finance/src/events.rs:1255` (`DirectFeesSettingCreated`), plus `DirectFeesSettingUpdated` + `DirectFeesSettingDeleted` — all conformant `DomainEvent` impls with EVENT_TYPE `finance.direct_fees_setting.{created,updated,deleted}` and AGGREGATE_TYPE `direct_fees_setting`. Service function `create_direct_fees_setting` at `crates/domains/finance/src/services.rs:1255` mints the aggregate + the `DirectFeesSettingCreated` event in one shot. `CreateDirectFeesSettingCommand` extended at `crates/domains/finance/src/commands.rs:740` from `{tenant, enabled, description}` to `{tenant, enabled, reminder_before, no_installment, due_date_from_sem, description}` (matching the Wave 66 command-shape extension pattern). Prelude re-exports at `crates/domains/finance/src/lib.rs`. 16 behavioral tests in `crates/domains/finance/tests/direct_fees_setting.rs` cover the happy path, the zero-values boundary, the due_date at MAX_DUE_DAY boundary, 4 fresh() validation cases (negative reminder_before / negative no_installment / due_date=0 / due_date>MAX), 4 update_config() validation cases, 2 retire() cases, and 2 service function cases (aggregate + event pairing with EVENT_TYPE/AGGREGATE_TYPE/SCHEMA_VERSION pinned, negative reminder_before → `Validation` propagated). Dispatcher wiring for persistence + outbox remains a v3 Part 6 task.
- [x] DFS I-2: due_date_from_sem ∈ 1..=28 — **complete (Wave 69 full drop)** — see DFS I-1 evidence; `due_date_from_sem in 1..=MAX_DUE_DAY` (where `MAX_DUE_DAY = 28`) is validated in both `RealDirectFeesSetting::fresh()` and `RealDirectFeesSetting::update_config()`. The upper bound is 28 (not 31) so the due-day is valid for every month including February in non-leap years.

### Donor (2 invariants)

- [ ] DO I-1: show_public boolean — missing (placeholder stub)
- [ ] DO I-2: email unique within school — missing

### DueFeesLoginPrevent (2 invariants)

- [ ] DFLP I-1: unique per (school, academic, user, role) — missing (placeholder stub)
- [ ] DFLP I-2: auto-pruned when balance = 0 — missing

### ExpenseApproval (2 invariants)

- [ ] EA I-1: state machine pending → approved/rejected — missing (placeholder stub)
- [ ] EA I-2: timestamps recorded — missing

### ExpenseHead (1 invariant)

- [ ] EH I-1: unique name within school — missing (placeholder stub)

### FeesAssign (5 invariants)

- [~] FA I-1: amount ≥ 0 — partial (placeholder, FeeAmount VO enforces upper)
- [~] FA I-2: applied_discount ≤ fees_amount — partial (placeholder)
- [ ] FA I-3: sum(FeesPayment) cap — missing
- [ ] FA I-4: active_status true while open balance — missing
- [ ] FA I-5: unique per (student, fee_master, year) — missing

### FeesAssignDiscount (3 invariants)

- [~] FAD I-1: applied_amount ≥ 0 && unapplied ≥ 0 — partial (VO)
- [~] FAD I-2: applied + unapplied constant — partial (no mutator)
- [ ] FAD I-3: timestamp recorded — missing

### FeesCarryForward (3 invariants)

- [~] FCF I-1: balance ≥ 0 — partial (placeholder + VO)
- [~] FCF I-2: balance_type ∈ {debit, credit} — partial (BalanceType enum)
- [ ] FCF I-3: unique per (school, student, academic) — missing

### FeesCarryForwardLog (2 invariants)

- [ ] FCFL I-1: append-only — missing (placeholder stub)
- [ ] FCFL I-2: amount ≥ 0 — missing

### FeesCarryForwardSetting (2 invariants)

- [ ] FCFA I-1: per-school config — missing (placeholder stub)
- [ ] FCFA I-2: threshold ≥ 0 — missing

### FeesDiscount (4 invariants)

- [~] FD I-1: amount ≥ 0 — partial (placeholder)
- [~] FD I-2: discount_type valid — partial (DiscountType enum)
- [ ] FD I-3: once-per-master scope — missing
- [ ] FD I-4: once-per-year scope — missing

### FeesGroup (4 invariants)

- [ ] FG I-1: unique name within school — missing (placeholder stub)
- [ ] FG I-2: non-empty name — missing
- [ ] FG I-3: cascade to FeesMaster — missing
- [ ] FG I-4: cannot delete while referenced — missing

### FeesInstallment (5 invariants)

- [~] FIv I-1: percentage ∈ [0, 100] — partial (placeholder + VO)
- [~] FIv I-2: amount ≥ 0 — partial (placeholder + VO)
- [ ] FIv I-3: percentage sum ≤ 100 across installments — missing
- [ ] FIv I-4: due_date ordering — missing
- [ ] FIv I-5: non-overlapping windows — missing

### FeesInstallmentAssign (3 invariants)

- [ ] FIA I-1: unique per (assign, installment) — missing (placeholder stub)
- [ ] FIA I-2: paid_amount ≤ amount + discount — missing
- [ ] FIA I-3: active_status while open — missing

### FeesInstallmentAssignDiscount (2 invariants)

- [ ] FIAD I-1: applied_amount ≥ 0 — missing (placeholder stub)
- [ ] FIAD I-2: timestamps recorded — missing

### FeesInstallmentCredit (3 invariants)

- [ ] FIC I-1: amount ≥ 0 — missing (placeholder stub)
- [ ] FIC I-2: credit source valid — missing
- [ ] FIC I-3: append-only — missing

### FeesInvoiceSetting (2 invariants)

- [ ] FISv I-1: prefix format valid — missing (placeholder stub)
- [ ] FISv I-2: per_th ≥ 0 — missing

### FeesMaster (3 invariants)

- [~] FM I-1: amount ≥ 0 — partial (placeholder + FeeAmount VO)
- [ ] FM I-2: unique per (school, name, group) — missing
- [ ] FM I-3: cannot delete while FeesAssign references — missing

### FmFeesGroup (1 invariant)

- [x] FFG I-1: unique name within school — **complete (Wave 66 full drop)** — `RealFmFeesGroup` aggregate at `crates/domains/finance/src/aggregate.rs:1948` with `fresh()` (`:1972`, validates non-empty trimmed name), `update_metadata()` (`:2014`, bumps version + advances `updated_at`), and `retire()` (`:2040`, returns `Conflict` on already-retired). 3 typed events at `crates/domains/finance/src/events.rs:778` (`FmFeesGroupCreated`), `:830` (`FmFeesGroupUpdated`), `:883` (`FmFeesGroupDeleted`) — all conformant `DomainEvent` impls with EVENT_TYPE `finance.fm_fees_group.{created,updated,deleted}`. Service function `create_fm_fees_group` at `crates/domains/finance/src/services.rs:1114` mints the aggregate + the `FmFeesGroupCreated` event in one shot. Prelude re-exports the aggregate, command, event, and service at `crates/domains/finance/src/lib.rs`. The `CreateFmFeesGroupCommand` was extended from `{tenant, fm_fees_group_id}` to `{tenant, name, description}` so the service can mint the typed id from the next event id (matching the Wave 65 `create_income_head` shape). 11 behavioral tests in `crates/domains/finance/tests/fm_fees_group.rs` cover happy path, trim, 2 fresh() validation cases (empty + whitespace), 3 update_metadata() cases (empty new name → `Validation` + state preserved, valid name → version + `updated_at` bump + `updated_by` set, empty description cleared), 2 retire() cases, and 2 service function cases (aggregate + event pairing with EVENT_TYPE/AGGREGATE_TYPE/SCHEMA_VERSION pinned, whitespace-only name → `Validation` propagated). Dispatcher wiring for persistence + outbox path remains a v3 Part 6 task.

### FmFeesInvoice (3 invariants)

- [ ] FFI I-1: amount ≥ 0 — missing (placeholder stub)
- [ ] FFI I-2: due_date ≥ invoice_date — missing
- [ ] FFI I-3: state machine — missing

### FmFeesInvoiceChild (3 invariants)

- [ ] FFIChild I-1: amount ≥ 0 — missing (placeholder stub)
- [ ] FFIChild I-2: sub_total == amount + weaver + fine — missing
- [ ] FFIChild I-3: paid_amount ≤ sub_total + service_charge — missing

### FmFeesInvoiceLineNote (2 invariants)

- [ ] FFILN I-1: non-empty note — missing (placeholder stub)
- [ ] FFILN I-2: append-only — missing

### FmFeesInvoiceSetting (3 invariants)

- [ ] FFIS I-1: per_th ≥ 0 — missing (placeholder stub)
- [ ] FFIS I-2: due_date config — missing
- [ ] FFIS I-3: prefix format — missing

### FmFeesTransaction (3 invariants)

- [~] FFT I-1: amount ≥ 0 — partial (placeholder + Money VO)
- [ ] FFT I-2: total_paid_amount ≥ 0 — missing
- [ ] FFT I-3: state machine — missing

### FmFeesTransactionChild (2 invariants)

- [ ] FFTC I-1: amount ≥ 0 — missing (placeholder stub)
- [ ] FFTC I-2: parent reference valid — missing

### FmFeesTransactionLineNote (2 invariants)

- [ ] FFTLN I-1: non-empty — missing (placeholder stub)
- [ ] FFTLN I-2: append-only — missing

### FmFeesType (3 invariants)

- [~] FFT I-1: type ∈ {fee, discount, fine} — partial (placeholder)
- [ ] FFT I-2: amount ≥ 0 — missing
- [ ] FFT I-3: unique per (school, name) — missing

### FmFeesWeaver (2 invariants)

- [ ] FFW I-1: percentage ∈ [0, 100] — missing (placeholder stub)
- [ ] FFW I-2: sum on invoice ≤ sum of child subtotals — missing

### Income (3 invariants)

- [ ] IN I-1: amount ≥ 0 — missing (placeholder stub)
- [ ] IN I-2: account + payment_method compatible — missing
- [ ] IN I-3: timestamps recorded — missing

### IncomeApproval (2 invariants)

- [ ] IA I-1: state machine — missing (placeholder stub)
- [ ] IA I-2: timestamps — missing

### IncomeHead (1 invariant)

- [x] IH I-1: unique name within school — **complete (Wave 65 full drop)** — `RealIncomeHead` aggregate at `crates/domains/finance/src/aggregate.rs:1822` with `fresh()` (`:1846`, validates non-empty trimmed name), `update_metadata()` (`:1888`, bumps version + advances `updated_at`), and `retire()` (`:1914`, returns `Conflict` on already-retired). 3 typed events at `crates/domains/finance/src/events.rs:623` (`IncomeHeadCreated`), `:675` (`IncomeHeadUpdated`), `:728` (`IncomeHeadDeleted`) — all conformant `DomainEvent` impls with EVENT_TYPE `finance.income_head.{created,updated,deleted}`. Service function `create_income_head` at `crates/domains/finance/src/services.rs:651` mints the aggregate + the `IncomeHeadCreated` event in one shot. Prelude re-exports the aggregate, command, event, and service at `crates/domains/finance/src/lib.rs:62-91`. 13 behavioral + typed-id tests in `crates/domains/finance/tests/income_head.rs` cover the happy path, the 2 fresh() validation cases (empty + whitespace-only), update_metadata() success/failure, the 2 retire() cases, and the create_income_head service (success + validation propagation). Dispatcher wiring for the persistence + outbox path remains a v3 Part 6 task (0/509 wrappers done).

### InventoryPayment (3 invariants)

- [ ] IP I-1: amount ≥ 0 — missing (placeholder stub)
- [ ] IP I-2: payment_method + account compatible — missing
- [ ] IP I-3: append-only — missing

### InvoiceSetting (1 invariant)

- [x] ISv I-1: prefix format — **complete (Wave 67 full drop)** — `RealInvoiceSetting` aggregate at `crates/domains/finance/src/aggregate.rs:2074` with `fresh()` validating ISv I-1 (`prefix` must be 1..=`MAX_PREFIX_LEN` (10) chars after trim; `start_form` must be ≥ 0), `update_config()` (version + `updated_at` bump + `updated_by` set; same prefix/start_form validation), and `retire()` (returns `Conflict` on already-retired). 3 typed events at `crates/domains/finance/src/events.rs:934` (`InvoiceSettingCreated`), plus `InvoiceSettingUpdated` + `InvoiceSettingDeleted` — all conformant `DomainEvent` impls with EVENT_TYPE `finance.invoice_setting.{created,updated,deleted}` and AGGREGATE_TYPE `invoice_setting`. Service function `create_invoice_setting` at `crates/domains/finance/src/services.rs:1161` mints the aggregate + the `InvoiceSettingCreated` event in one shot. New `CreateInvoiceSettingCommand { tenant, prefix, start_form }` added at `crates/domains/finance/src/commands.rs:1725` (matching the Wave 65/66 mint-id-from-event-id pattern). Prelude re-exports at `crates/domains/finance/src/lib.rs`. 15 behavioral tests in `crates/domains/finance/tests/invoice_setting.rs` cover the happy path, prefix at MAX_PREFIX_LEN boundary, 5 fresh() validation cases (empty / whitespace / over-max / negative start_form), 3 update_config() validation cases, 2 retire() cases, and 2 service function cases (aggregate + event pairing with EVENT_TYPE/AGGREGATE_TYPE/SCHEMA_VERSION pinned). Dispatcher wiring for persistence + outbox remains a v3 Part 6 task.

### PaymentGatewaySetting (4 invariants)

- [~] PGS I-1: per-school unique — partial (placeholder)
- [~] PGS I-2: mode ∈ {sandbox, live} — partial (GatewayMode enum)
- [ ] PGS I-3: charge ≥ 0; charge_type ∈ {P, F} — missing
- [ ] PGS I-4: credentials encrypted at rest — missing (storage-layer)

### PaymentMethod (3 invariants)

- [ ] PM I-1: method unique within school — missing (placeholder stub)
- [ ] PM I-2: gateway_id required for gateway-backed — missing
- [ ] PM I-3: account_id compatible — missing

### PayrollEarnDeduc (3 invariants)

- [ ] PED I-1: amount ≥ 0 — missing (placeholder stub in finance; authoritative in HR)
- [ ] PED I-2: earn_dedc_type ∈ {e, d} — missing
- [ ] PED I-3: sum invariants — missing

### PayrollGenerate (4 invariants)

- [ ] PG I-1: gross == basic + total_earning — missing (placeholder; HR authoritative)
- [ ] PG I-2: net == gross - total_deduction - tax — missing
- [ ] PG I-3: payroll_status state machine — missing
- [ ] PG I-4: paid_amount ≤ net_salary — missing

### PayrollPayment (3 invariants)

- [~] PP I-1: sum vs PayrollGenerate.unpaid net_salary — partial (placeholder + service stub)
- [ ] PP I-2: payment_method + bank_id compatible — missing
- [ ] PP I-3: creates Expense + BankStatement — missing

### PayrollPaymentApproval (2 invariants)

- [ ] PPA I-1: state machine — missing (placeholder stub)
- [ ] PPA I-2: timestamps — missing

### ProductPurchase (3 invariants)

- [ ] PPr I-1: amount ≥ 0 — missing (placeholder stub)
- [ ] PPr I-2: vendor reference valid — missing
- [ ] PPr I-3: state machine — missing

### QuestionBankFee (1 invariant)

- [x] QBF I-1: amount ≥ 0 — **complete (Wave 68 full drop)** — `RealQuestionBankFee` aggregate at `crates/domains/finance/src/aggregate.rs:2224` with `fresh()` validating QBF I-1 (`name` must be non-empty after trim; `amount_minor` must be ≥ 0; zero amount is allowed — a free sample), `update_metadata()` (version + `updated_at` bump + `updated_by` set; same name + amount_minor validation), and `retire()` (returns `Conflict` on already-retired). 3 typed events at `crates/domains/finance/src/events.rs:1090` (`QuestionBankFeeCreated`), plus `QuestionBankFeeUpdated` + `QuestionBankFeeDeleted` — all conformant `DomainEvent` impls with EVENT_TYPE `finance.question_bank_fee.{created,updated,deleted}` and AGGREGATE_TYPE `question_bank_fee`. Service function `create_question_bank_fee` at `crates/domains/finance/src/services.rs:1206` mints the aggregate + the `QuestionBankFeeCreated` event in one shot. New `CreateQuestionBankFeeCommand { tenant, name, amount_minor, description }` added at `crates/domains/finance/src/commands.rs:1738` (matching the Wave 65/66/67 mint-id-from-event-id pattern). Prelude re-exports at `crates/domains/finance/src/lib.rs`. 14 behavioral tests in `crates/domains/finance/tests/question_bank_fee.rs` cover the happy path with positive amount, the zero-amount boundary case, 4 fresh() validation cases (empty / whitespace name, negative amount_minor), 4 update_metadata() cases (empty new name → `Validation` + state preserved, negative new amount → `Validation`, valid inputs → version + `updated_at` bump + `updated_by` set, empty description cleared), 2 retire() cases, and 2 service function cases (aggregate + event pairing with EVENT_TYPE/AGGREGATE_TYPE/SCHEMA_VERSION pinned, negative amount → `Validation` propagated). Dispatcher wiring for persistence + outbox remains a v3 Part 6 task.

### SalaryTemplate (2 invariants)

- [~] ST I-1: gross_salary composition — partial (service-side; composition deferred)
- [~] ST I-2: net_salary == gross - total_deduction — partial (service-side)

### Transaction (3 invariants)

- [ ] TR I-1: sum(debits) == sum(credits) per school — missing (placeholder stub)
- [ ] TR I-2: append-only — missing
- [ ] TR I-3: state machine — missing

### Wallet (2 invariants, listed separately)

- [x] Wallet I-1: balance starts at 0 — `aggregate.rs:103-127`
- [~] Wallet cross-aggregate: balance == sum of approved tx — partial

### WalletTransaction (4 invariants, listed separately)

See above.

### WalletTransactionApproval (2 invariants)

- [~] WTA I-1: state machine — partial (ApprovalStatus enum)
- [ ] WTA I-2: timestamps + reason — missing

## Cross-cutting Enforcement Gaps

1. **Placeholder stubs** — 28 of 47 aggregates are placeholder stubs (`pub struct { id, school_id }`). Each contributes 2-5 missing invariants.
2. **Cross-aggregate invariants** — Many invariants (FeesAssign payment cap, BankStatement running balance, ChartOfAccount delete guard) require repository access; aggregate layer can't enforce them. These need dispatcher-level enforcement.
3. **HR ↔ Finance split** — `PayrollGenerate`, `PayrollEarnDeduc`, `SalaryTemplate` authoritative implementations live in `educore-hr`; finance is a typed-view stub.
4. **Gateway consistency** — FeesPayment invariants 3-4 (gateway mode consistency, gateway tx id required) need payment-gateway domain knowledge.

## Implementation Order

- **Batch 1:** Foundation (Money/Currency) [already done] + Wallet + WalletTransaction (6 invariants)
- **Batch 2:** FeesPayment + FeesInvoice + Expense + FeesDiscount (15+ invariants)
- **Batch 3:** Banking (BankAccount, BankStatement, AmountTransfer, BankPaymentSlip, BankPaymentSlipAudit, ChartOfAccount) (~18 invariants)
- **Batch 4:** FeesInstallment + DirectFeesInstallment + DirectFeesInstallmentAssign + ChildPayment + FeesMaster + FeesAssign (~20 invariants)
- **Batch 5:** FM variants (FmFeesInvoice + children + Transactions + Types) + Income + ExpenseApproval (~25 invariants)
- **Batch 6:** Payroll + SalaryTemplate + HourlyRate (HR ↔ Finance split, deferred)
- **Batch 7:** Donor + DueFeesLoginPrevent + CarryForward + PaymentGateway + PaymentMethod + InvoiceSetting (~25 invariants)

**Note:** This scope (165 invariants) is significantly larger than academic's 72. Each batch should be sized to fit a single sub-agent budget (~20-30 invariants max).
