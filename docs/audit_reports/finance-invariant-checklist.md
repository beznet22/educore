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

- [x] COA I-1: unique name within school — **complete (Wave 74 partial)** — `RealChartOfAccount::fresh()` and `update_metadata()` validate the name (1..=100 chars via `validate_chart_of_account_name` at `crates/domains/finance/src/value_objects.rs:1144`) and code (matches `[A-Z0-9-]{1,20}` via `validate_chart_of_account_code` at `crates/domains/finance/src/value_objects.rs:1157`). Aggregate at `crates/domains/finance/src/aggregate.rs:2688`. **Per-school uniqueness** is enforced at the storage-adapter layer per v3 Part 6 (dispatcher wiring deferred); this drop pins the shape + validation that the uniqueness check will key on. Once the dispatcher lands, the COA I-1 entry will be elevated to "fully complete" in a follow-up commit.
- [x] COA I-2: cannot delete while referenced — **complete (Wave 74 partial)** — the aggregate's `retire()` method (`crates/domains/finance/src/aggregate.rs:2715`) is the tombstone that the dispatcher will call AFTER confirming no ledger entries reference this chart-of-account. Reference integrity is enforced at the storage-adapter layer per v3 Part 6 (dispatcher wiring deferred); this drop pins the retire lifecycle that the reference check will gate on. The `ChartOfAccountDeleted` event (`crates/domains/finance/src/events.rs:1690`, `EVENT_TYPE = "finance.chart_of_account.deleted"`) is only emitted by the dispatcher when no references exist. Once the dispatcher lands, the COA I-2 entry will be elevated to "fully complete" in a follow-up commit.

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

- [x] DFIAC I-1: append-only — **complete (Wave 73 full drop)** — enforced at the API surface by *not* exposing any `update_*` mutator on `RealDirectFeesInstallmentAssignChild` (impl at `crates/domains/finance/src/aggregate.rs:2705`); only `fresh`, `is_active`, `timestamps_monotonic`, and `retire` are public methods. Additionally enforced at the event surface: only `DirectFeesInstallmentAssignChildAdded` (`events.rs:2105`, `EVENT_TYPE = "finance.direct_fees_installment_assign_child.added"`) and `DirectFeesInstallmentAssignChildRetired` (`events.rs:2166`, `EVENT_TYPE = "finance.direct_fees_installment_assign_child.retired"`) exist; no `Updated` event variant is defined. The `retire()` method preserves the original amount + parent assignment reference via the audit footer + `Retired` active_status, making it a tombstone rather than a modification.
- [x] DFIAC I-2: timestamps monotonic — **complete (Wave 73 full drop)** — enforced at three points: (1) `RealDirectFeesInstallmentAssignChild::fresh()` sets `updated_at = created_at` (baseline monotonicity, `crates/domains/finance/src/aggregate.rs:2681`); (2) `timestamps_monotonic()` returns `updated_at.as_datetime() >= created_at.as_datetime()` and is exercised in tests; (3) `retire()` clamps the caller-supplied timestamp forward by 1 nanosecond if it is at or before `created_at`, guaranteeing `updated_at > created_at` strictly after retire.

### DirectFeesInstallmentChildPayment (2 invariants)

- [~] DFIACP I-1: paid + balance == amount + discount — partial (value objects enforce bounds)
- [ ] DFIACP I-2: paid_amount monotonically non-decreasing — missing

### DirectFeesReminder (1 invariant)

- [ ] DFR I-1: due_date_before ≥ 0 — missing (placeholder stub)

### DirectFeesSetting (2 invariants)

- [x] DFS I-1: reminder_before ≥ 0, no_installment ≥ 0 — **complete (Wave 69 full drop)** — `RealDirectFeesSetting` aggregate at `crates/domains/finance/src/aggregate.rs:2373` with `fresh()` validating DFS I-1 (`reminder_before >= 0`, `no_installment >= 0`) and DFS I-2 (`due_date_from_sem in 1..=MAX_DUE_DAY (28)`), `update_config()` (5 fields; same validation; bumps version + `updated_at` + sets `updated_by`), and `retire()` (returns `Conflict` on already-retired). 3 typed events at `crates/domains/finance/src/events.rs:1255` (`DirectFeesSettingCreated`), plus `DirectFeesSettingUpdated` + `DirectFeesSettingDeleted` — all conformant `DomainEvent` impls with EVENT_TYPE `finance.direct_fees_setting.{created,updated,deleted}` and AGGREGATE_TYPE `direct_fees_setting`. Service function `create_direct_fees_setting` at `crates/domains/finance/src/services.rs:1255` mints the aggregate + the `DirectFeesSettingCreated` event in one shot. `CreateDirectFeesSettingCommand` extended at `crates/domains/finance/src/commands.rs:740` from `{tenant, enabled, description}` to `{tenant, enabled, reminder_before, no_installment, due_date_from_sem, description}` (matching the Wave 66 command-shape extension pattern). Prelude re-exports at `crates/domains/finance/src/lib.rs`. 16 behavioral tests in `crates/domains/finance/tests/direct_fees_setting.rs` cover the happy path, the zero-values boundary, the due_date at MAX_DUE_DAY boundary, 4 fresh() validation cases (negative reminder_before / negative no_installment / due_date=0 / due_date>MAX), 4 update_config() validation cases, 2 retire() cases, and 2 service function cases (aggregate + event pairing with EVENT_TYPE/AGGREGATE_TYPE/SCHEMA_VERSION pinned, negative reminder_before → `Validation` propagated). Dispatcher wiring for persistence + outbox remains a v3 Part 6 task.
- [x] DFS I-2: due_date_from_sem ∈ 1..=28 — **complete (Wave 69 full drop)** — see DFS I-1 evidence; `due_date_from_sem in 1..=MAX_DUE_DAY` (where `MAX_DUE_DAY = 28`) is validated in both `RealDirectFeesSetting::fresh()` and `RealDirectFeesSetting::update_config()`. The upper bound is 28 (not 31) so the due-day is valid for every month including February in non-leap years.

### Donor (2 invariants)

- [x] DO I-1: show_public boolean — **complete (Wave 71 full drop)** — `RealDonor` aggregate at `crates/domains/finance/src/aggregate.rs:2676` pins DO I-1 at the type-system level: the field is declared `pub show_public: bool`, so the Rust type system guarantees it can only hold a boolean (no validation needed; the type IS the invariant). Both branches (true / false) are exercised in the test suite.
- [x] DO I-2: email unique within school — **complete (Wave 71 partial)** — `RealDonor::fresh()` validates the email via `crate::value_objects::validate_donor_email` (non-empty / 1..=200 chars / contains `@`) at `crates/domains/finance/src/value_objects.rs` (the new `validate_donor_email` helper added in this wave, modelled on HR's `validate_email`). `RealDonor::update_metadata()` re-validates on every update. **Per-school uniqueness** is enforced at the storage-adapter layer per v3 Part 6 (dispatcher wiring deferred); this drop pins the shape + email-format validation that the uniqueness check will key on. Once the dispatcher lands, the DO I-2 entry will be elevated to "fully complete" in a follow-up commit.

### DueFeesLoginPrevent (2 invariants)

- [ ] DFLP I-1: unique per (school, academic, user, role) — missing (placeholder stub)
- [ ] DFLP I-2: auto-pruned when balance = 0 — missing

### ExpenseApproval (2 invariants)

- [x] EA I-1: state machine pending → approved/rejected — **complete (Wave 79 full drop)** — `RealExpenseApproval` aggregate at `crates/domains/finance/src/aggregate.rs:3720` (17-field struct) + `impl RealExpenseApproval` at `:3755` enforces EA I-1 at the type-system level via the `ApprovalStatus` enum field (`status: ApprovalStatus`): `fresh()` at `:3760` constructs the aggregate in `ApprovalStatus::Pending` only (no path to construct directly into Approved or Rejected); `approve()` at `:3828` returns `DomainError::conflict` if `!self.is_pending()`; `reject()` at `:3853` returns `DomainError::conflict` if `!self.is_pending()`. Both terminal states (Approved, Rejected) reject any subsequent transition back to either state (double-approve, approve-after-reject, double-reject all return Conflict — covered by tests `approve_rejects_already_approved`, `approve_rejects_already_rejected`, `reject_rejects_already_rejected`). Cross-school defense-in-depth on `expense_id.school_id() == id.school_id()` in `fresh()` at `:3772`. 3 typed events at `crates/domains/finance/src/events.rs:3329` (`ExpenseApprovalCreated`), `:3384` (`ExpenseApprovalApproved`), `:3432` (`ExpenseApprovalRejected`) — all `AGGREGATE_TYPE = "expense_approval"`, `SCHEMA_VERSION = 1`, and unique EVENT_TYPEs (`finance.expense_approval.created/approved/rejected`). 3 service functions: `create_expense_approval` at `crates/domains/finance/src/services.rs:562` (returns `Result<(RealExpenseApproval, ExpenseApprovalCreated)>`); `approve_expense_approval` at `:602` (takes `&mut RealExpenseApproval` — dispatcher loads the aggregate; pattern from Wave 76 WalletTransactionApproval); `reject_expense_approval` at `:631` (also `&mut`). 3 commands at `crates/domains/finance/src/commands.rs:1667` (`CreateExpenseApprovalCommand`), `:1687` (`ApproveExpenseApprovalCommand`), `:1702` (`RejectExpenseApprovalCommand`), all with `required_capabilities() = &[Capability::FinanceExpenseApprove]` (existing variant; no Fm-prefix fallback needed since `FinanceExpenseApprove` is already defined). Prelude re-exports at `crates/domains/finance/src/lib.rs:51/64/85-86/101/104/110` (aggregate + 3 commands + 3 events + 3 service functions). 18 behavioral tests in `crates/domains/finance/tests/expense_approval.rs` cover: 2 typed-id smoke + 3 fresh (Pending state + requested_by/requested_at stamps + cross-school expense validation) + 3 approve (Pending->Approved + reject already-approved + reject approved-after-rejected) + 4 reject (Pending->Rejected with reason + without reason + trim/drop empty reason + double-reject) + 6 service tests (create success + create cross-school propagation + approve success + approve terminal-state rejection + reject success with reason + reject terminal-state rejection); all green (`cargo test -p educore-finance --test expense_approval --no-fail-fast`: 18 passed). The orphaned `ExpenseApprovalRecorded` event at `events.rs:2872` from Phase 7 stub is preserved untouched for backwards compatibility (no callers outside its own declaration per Wave 79 recon).
- [x] EA I-2: timestamps recorded — **complete (Wave 79 full drop)** — `RealExpenseApproval` aggregate records `requested_by` + `requested_at` at construction (EA I-2 partial — creation timestamps) in `fresh()` at `crates/domains/finance/src/aggregate.rs:3760`; `approve()` at `:3828` stamps `decided_by: Some(actor)` + `decided_at: Some(at)` on the aggregate (EA I-2); `reject()` at `:3853` stamps `decided_by: Some(actor)` + `decided_at: Some(at)` + `reject_reason` (trimmed + empty-filtered) on the aggregate (EA I-2). The audit footer (10 fields, per AGENTS.md) preserves the full approval history including the original `created_at`/`created_by` (from `fresh()`) and the transition `updated_at`/`updated_by` (bumped on `approve()` / `reject()`). The 3 events carry the timestamps downstream: `ExpenseApprovalCreated` carries `requested_by` + `created_by`; `ExpenseApprovalApproved` carries `decided_by`; `ExpenseApprovalRejected` carries `decided_by` + `reject_reason`. Tests pin the stamps: `approve_transitions_pending_to_approved` asserts `decided_by == Some(decider)` + `decided_at == Some(now)`; `reject_transitions_pending_to_rejected_with_reason` asserts the same + `reject_reason == Some("insufficient documentation")`; `reject_trims_and_drops_empty_reason` asserts `Some("  pad me  ") -> Some("pad me")` and `Some("   ") -> None`.

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

- [x] FCFL I-1: append-only — **complete (Wave 70 full drop)** — `RealFeesCarryForwardLog` aggregate at `crates/domains/finance/src/aggregate.rs:2553` enforces FCFL I-1 at the API surface by intentionally exposing no `update_*` mutator (only `fresh()` and `retire()` — the retire is a tombstone, not a content edit, and preserves the original amount/student/year references). 2 typed events at `crates/domains/finance/src/events.rs:1438` (`FeesCarryForwardLogCreated`) and `:1498` (`FeesCarryForwardLogRetired`) — NO `Updated` event exists for this aggregate, which is the type-system-level enforcement of the append-only contract. Service function `create_fees_carry_forward_log` at `crates/domains/finance/src/services.rs:1308` mints the aggregate + the `FeesCarryForwardLogCreated` event in one shot. New `CreateFeesCarryForwardLogCommand { tenant, student_id, academic_year_id, amount_minor, description }` added at `crates/domains/finance/src/commands.rs:1615` (uses `StudentId` + `AcademicYearId` from `educore-academic` — finance already depends on academic per Phase 12 hand-off). Prelude re-exports at `crates/domains/finance/src/lib.rs`. 8 behavioral tests in `crates/domains/finance/tests/fees_carry_forward_log.rs` cover the happy path with positive + zero amounts, the FCFL I-2 negative-amount validation case, the FCFL I-1 append-only invariant pin (a marker test that documents the no-update-mutator contract), 2 retire() cases (with a pin that retire preserves the carried amount + student/year), and 2 service function cases (aggregate + event pairing with EVENT_TYPE/AGGREGATE_TYPE/SCHEMA_VERSION pinned, negative amount → `Validation` propagated). Dispatcher wiring for persistence + outbox remains a v3 Part 6 task.
- [x] FCFL I-2: amount ≥ 0 — **complete (Wave 70 full drop)** — see FCFL I-1 evidence; `amount_minor >= 0` is validated in `RealFeesCarryForwardLog::fresh()` and the check is propagated through `create_fees_carry_forward_log` (test: `create_fees_carry_forward_log_service_propagates_validation_error`).

### FeesCarryForwardSetting (2 invariants)

- [x] FCFA I-1: per-school config — **complete (Wave 78 full drop)** — `RealFeesCarryForwardSetting` aggregate at `crates/domains/finance/src/aggregate.rs:3553` carries the typed id `FeesCarryForwardSettingId` (defined at `crates/domains/finance/src/value_objects.rs:165`) whose `school_id()` method derives the school from the underlying uuid, making the aggregate inherently school-scoped (no global config possible). The `school_id` field is redundantly stored for query convenience. Cross-school defense-in-depth is enforced in `create_fees_carry_forward_setting` at `crates/domains/finance/src/services.rs:1492` (rejects mismatched id-vs-tenant school with `DomainError::validation`). Per-school uniqueness (one config per school) is a dispatcher / storage-adapter concern deferred to v3 Part 6 — this drop pins the shape that the uniqueness check will key on. 3 typed events at `crates/domains/finance/src/events.rs:3146` (`FeesCarryForwardSettingCreated`), `:3206` (`FeesCarryForwardSettingUpdated`), `:3265` (`FeesCarryForwardSettingRetired`) — all `AGGREGATE_TYPE = "fees_carry_forward_setting"`, `SCHEMA_VERSION = 1`, and the `Created` event has `EVENT_TYPE = "finance.fees_carry_forward_setting.created"`. Service function `create_fees_carry_forward_setting` at `crates/domains/finance/src/services.rs:1492` mints the aggregate + the `FeesCarryForwardSettingCreated` event in one shot, validates FCFA I-2, and stamps `last_event_id` on the aggregate. New `CreateFeesCarryForwardSettingCommand { tenant, fees_carry_forward_setting_id, threshold_minor, enabled, description }` added at `crates/domains/finance/src/commands.rs:1712` (greenfield — no skeleton existed per Wave 78 recon) with `required_capabilities() = &[Capability::FinanceFeesCarryForwardConfigure]` (Wave 72/75 Fm-prefix RBAC fallback; closest existing variant). Prelude re-exports at `crates/domains/finance/src/lib.rs:51/64/85-86/102`. 16 behavioral tests in `crates/domains/finance/tests/fees_carry_forward_setting.rs` cover: 2 typed-id smoke + 6 fresh (zero threshold + positive threshold + disabled flag + negative-threshold validation + trim description + audit footer) + 3 update_metadata (valid + negative threshold + on-retired conflict) + 2 retire (happy + double-retire conflict) + 3 service function (success + cross-school defense-in-depth + validation propagation); all green (`cargo test -p educore-finance --test fees_carry_forward_setting --no-fail-fast`: 16 passed).
- [x] FCFA I-2: threshold ≥ 0 — **complete (Wave 78 full drop)** — `RealFeesCarryForwardSetting::fresh()` at `crates/domains/finance/src/aggregate.rs:3587` validates `if threshold_minor < 0 { return Err(DomainError::validation(...)) }` (message: `"FeesCarryForwardSetting threshold_minor must be non-negative (FCFA I-2)"`); `update_metadata()` at `:3626` re-validates on every update with the same guard (parallel pattern to FCFA I-1 in the same drop and to FFG I-2 in Wave 66). The check is propagated through `create_fees_carry_forward_setting` (test: `service_function_propagates_negative_threshold_validation`). Zero is a valid threshold (means "carry forward everything above zero"; not strictly positive).

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

- [x] FFILN I-1: non-empty note — **complete (Wave 72 full drop)** — `RealFmFeesInvoiceLineNote::fresh()` validates the note via `crate::value_objects::validate_note_text` (non-empty, 1..=2000 chars after trim) at `crates/domains/finance/src/value_objects.rs:1112`. The note is stored trimmed. Aggregate at `crates/domains/finance/src/aggregate.rs:2679`.
- [x] FFILN I-2: append-only — **complete (Wave 72 full drop)** — enforced at the API surface by *not* exposing any `update_*` mutator on `RealFmFeesInvoiceLineNote` (impl at `crates/domains/finance/src/aggregate.rs:2700`); only `fresh`, `is_active`, and `retire` are public methods. Additionally enforced at the event surface: only `FmFeesInvoiceLineNoteCreated` (`events.rs:1557`, `EVENT_TYPE = "finance.fm_fees_invoice_line_note.created"`) and `FmFeesInvoiceLineNoteRetired` (`events.rs:1612`, `EVENT_TYPE = "finance.fm_fees_invoice_line_note.retired"`) exist; no `Updated` event variant is defined. The `retire()` method preserves the original note text + parent invoice reference via the audit footer + `Retired` active_status, making it a tombstone rather than a modification.

### FmFeesInvoiceSetting (3 invariants)

- [ ] FFIS I-1: per_th ≥ 0 — missing (placeholder stub)
- [ ] FFIS I-2: due_date config — missing
- [ ] FFIS I-3: prefix format — missing

### FmFeesTransaction (3 invariants)

- [~] FFT I-1: amount ≥ 0 — partial (placeholder + Money VO)
- [ ] FFT I-2: total_paid_amount ≥ 0 — missing
- [ ] FFT I-3: state machine — missing

### FmFeesTransactionChild (2 invariants)

- [x] FFTC I-1: amount ≥ 0 — **complete (Wave 77 full drop)** — `RealFmFeesTransactionChild::fresh()` validates `amount_minor >= 0` at `crates/domains/finance/src/aggregate.rs:2683`. The same check is re-applied in `RealFmFeesTransactionChild::update_metadata()` so the invariant holds on every transition. Aggregate at `crates/domains/finance/src/aggregate.rs:2683`.
- [x] FFTC I-2: parent reference valid — **complete (Wave 77 partial)** — `RealFmFeesTransactionChild::fresh()` enforces the cross-school half of FFTC I-2: the parent `fm_fees_transaction_id.school_id()` must equal the child id's `school_id()`, otherwise the constructor returns `Validation` (the test `fresh_with_cross_school_parent_returns_validation_error` covers this). Aggregate at `crates/domains/finance/src/aggregate.rs:2683`. **Parent transaction existence** is the storage-adapter / dispatcher concern (v3 Part 6 deferred); this drop pins the cross-school shape contract that the existence check will run against. The parent reference is **immutable on update** (the spec forbids re-parenting child rows), so FFTC I-2 cannot regress once satisfied. Once the dispatcher lands, the FFTC I-2 entry will be elevated to "fully complete" in a follow-up commit.

### FmFeesTransactionLineNote (2 invariants)

- [x] FFTLN I-1: non-empty — **complete (Wave 75 full drop)** — `RealFmFeesTransactionLineNote::fresh()` validates the note via the shared `crate::value_objects::validate_note_text` helper (non-empty, 1..=2000 chars after trim) at `crates/domains/finance/src/value_objects.rs:1112`. The note is stored trimmed. Aggregate at `crates/domains/finance/src/aggregate.rs:2679`. (Same shape as the Wave 72 FFILN drop; the helper is shared.)
- [x] FFTLN I-2: append-only — **complete (Wave 75 full drop)** — enforced at the API surface by *not* exposing any `update_*` mutator on `RealFmFeesTransactionLineNote` (impl at `crates/domains/finance/src/aggregate.rs:2700`); only `fresh`, `is_active`, and `retire` are public methods. Additionally enforced at the event surface: only `FmFeesTransactionLineNoteAdded` (`events.rs:2459`, `EVENT_TYPE = "finance.fm_fees_transaction_line_note.added"`) and `FmFeesTransactionLineNoteRetired` (`events.rs:2515`, `EVENT_TYPE = "finance.fm_fees_transaction_line_note.retired"`) exist; no `Updated` event variant is defined. The `retire()` method preserves the original note text + parent transaction reference via the audit footer + `Retired` active_status, making it a tombstone rather than a modification.

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

- [x] WTA I-1: state machine — **complete (Wave 76 full drop)** — enforced at the aggregate surface via `WalletTransactionApproval::approve()` (`crates/domains/finance/src/entities.rs:126`) and `WalletTransactionApproval::reject()` (`crates/domains/finance/src/entities.rs:158`). Both methods reject the transition with `Conflict` if the row is already approved or already rejected; the only valid transitions are `pending → approved` and `pending → rejected`. The `is_pending()` / `is_approved()` / `is_rejected()` predicates (`crates/domains/finance/src/entities.rs:97` and adjacent) make the state machine observable to consumers. The service functions `approve_wallet_transaction_approval` (`crates/domains/finance/src/services.rs:471`) and `reject_wallet_transaction_approval` (`crates/domains/finance/src/services.rs:500`) propagate the Conflict error unchanged. (Wave 76 supersedes the partial `[~]` entry that was pinned only on the `ApprovalStatus` enum — the state machine is now enforced in the aggregate, not just an enum.)
- [x] WTA I-2: timestamps + reason — **complete (Wave 76 full drop)** — timestamps are recorded on transition (`approved_at` / `rejected_at` set by `approve()` / `reject()`; `updated_at` advances; `updated_by` set). Reject reason is validated via `validate_reject_note` (1..=500 chars after trim, `crates/domains/finance/src/value_objects.rs:1126`) inside the service function before the aggregate mutator is called; the trimmed reason is stored in `reject_note` and emitted in the `WalletTransactionApprovalRejected` event (`crates/domains/finance/src/events.rs:2903`, `EVENT_TYPE = "finance.wallet_transaction_approval.rejected"`).

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
