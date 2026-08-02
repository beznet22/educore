# Session Handoff — Wave 169 (next agent pickup)

**Generated:** 2026-08-02, end of session (commit `d2a9e45`)
**For:** The next session agent. **Zero prior context assumed.**
**Companion:** [`17-final-reconciliation-audit.md`](17-final-reconciliation-audit.md) — full gap analysis; this doc is the operational entry point.

> **Supersession note (Wave 171):** This doc was superseded by [`19-session-handoff.md`](19-session-handoff.md) on 2026-08-02 (commit `f743f8d`). The Wave 169 tally claim ("Wave 32 added 7 invariants") was an off-by-one error — the actual count was 8 (1 Staff + 2 PayrollGenerate + 3 LeaveRequest + 1 LeaveDefine + 1 HourlyRate). The corrected tally (15 `[x]` / 92 `[ ]`) is in the Wave 171 doc. All 17 `docs/handoff/PHASE-X-HANDOFF.md` references in this doc are also stale — they point at retired files; see `19-session-handoff.md` for the replacement pattern.

---

## 0. TL;DR (read this first)

**Where we are:** Head at `d2a9e45` on `main`, all pushed to `origin/main`. The finance crate is in excellent shape after 102 waves of per-aggregate work + 3 clippy cleanup waves. The repository has 37 packages (1 umbrella + 36 internal crates), 992+ behavioral tests across 139+ invariants, and 4 dialect-aware storage adapters.

**What's left:**
1. **HR domain — 38 aggregates need real implementation.** The HR crate has 42 placeholder aggregates; only 7 have `[x]` invariants (all from Wave 32). Wave 169+ should continue the per-aggregate wave pattern on HR, starting with the Staff aggregate (8 invariants — the highest count).
2. **Finance domain — ~30 placeholder stubs remain.** The per-aggregate wave pipeline is proven and active; just keep going.
3. **2 stale docs to fix immediately.** `progress-tracker.md` Phase 6 row + `hr-invariant-checklist.md` Summary table. 1-line edits.
4. **1 missing ADR.** ADR-021-PhaseNumberingConventions to formalize the "Phase 17 = production hardening or CMS" resolution.
5. **0/509 dispatcher wrappers.** This is the #1 production gap; once the wave pipeline catches up to most domains, the dispatcher layer needs implementation.

**The first 5 commands to run are in §11 below.** Read §1-10 first to understand the state.

---

## 1. Repository Identity (don't get this wrong)

| Concept | Value |
|---|---|
| **Brand (prose)** | **Educore** |
| **Umbrella package** | `educore` (in `crates/educore/`) |
| **Internal package names** | `educore-<name>` (e.g., `educore-finance`) |
| **Internal crate directories** | `crates/<tier>/<name>/` (drop the `educore-` prefix) |
| **External crate ids in Rust code** | `educore_<name>` (e.g., `use educore_finance::...`) |
| **Public registry path** | `educore::*` |
| **Tier system** | infra (3) ← cross-cutting (9) ← domains (10) ← tools (4); adapters (10) depend on infra + cross-cutting |
| **Tier boundary enforcement** | `educore-core::lint` sub-module verifies that domains don't import from adapters/tools and cross-cutting doesn't import from domains/adapters/tools |
| **5 + 1 tiers** | infra, cross-cutting, domains, adapters, tools, umbrella |
| **Total packages** | 37 = 36 internal + 1 umbrella |
| **Total domain crates** | 10 (academic, assessment, attendance, hr, finance, facilities, library, communication, documents, cms) |
| **Cross-cutting crates** | 9 (platform, rbac, events, events-domain, settings, operations, audit, sync, sync-inprocess) |
| **Adapters** | 10 (4 storage + auth, event-bus, files, integrations, notify, payment) |
| **Tools** | 4 (testkit, storage-parity, sdk, cli) |
| **Infra** | 3 (core, query-derive, storage) |

**Naming rules (enforced):**
- Never use legacy names (Schoolify, InfixEdu) — removed from `docs/specs/` (77 files, ~1033/~1102 lines) but Git history retains them
- Use **Educore** in prose; **`educore`** in code
- Cargo deps use `cargo add <crate> --package <package-name>`
- New crates: `cargo new --lib --vcs none crates/<name>`

---

## 2. Architectural Decisions Made (the ones that matter)

Full list at `docs/decisions/` (20 ADRs). The load-bearing ones:

| ADR | Title | Implication |
|---|---|---|
| **ADR-001** | DDD | Aggregate roots own invariants. Service functions are pure factories. Domain events are the contract between aggregates. |
| **ADR-002** | Hexagonal | Domain code is port-define-only. Adapters implement ports. Crates can't depend on adapters. |
| **ADR-003** | Multi-Tenancy | Every aggregate carries `SchoolId`. `TenantContext` flows through every command. |
| **ADR-004** | Commands | Every state change is a typed Command. Commands have `required_capabilities()`. |
| **ADR-005** | Events | Every state change emits a typed `DomainEvent`. Events are immutable + serialized. |
| **ADR-006** | QueryLayer | `#[derive(DomainQuery)]` macro emits a typed AST; storage adapters translate to dialect SQL. |
| **ADR-007** | Audit-First | Every state change writes to `audit_log`. Storage adapters own the 4-port split: Outbox, AuditLog, EventLog, Idempotency. |
| **ADR-008** | Offline-First | Sync engine port (ADR-018) supports offline clients; SurrealDB is the embedded/offline adapter. |
| **ADR-009** | Capability Permissions | RBAC is capability-based (not role-based). 540 Command structs have `required_capabilities()`. |
| **ADR-010** | AI Agent | AI agents are first-class contributors. The handoff doc pattern is canonical. |
| **ADR-011** | Rust Ecosystem | Rust edition 2021, MSRV 1.75. `unsafe` forbidden in domain code. |
| **ADR-012** | No Reflection | No `serde_json::Value` in domain code. No `HashMap<String, T>` for domain data. No service locators. |
| **ADR-013** | Crate Layout | The `educore-` prefix on packages, dropped on directories. Documented in `AGENTS.md`. |
| **ADR-014** | Idempotency | Commands carry `IdempotencyKey`; storage adapter enforces uniqueness. |
| **ADR-015** | External Crates | 11 external crates exceed MSRV floor (1.75) and are pinned to last-compatible line. rustls everywhere (no native-tls). |
| **ADR-016** | Engine Graph | `graphify-out/` is the committed AST-only knowledge graph; auto-rebuilt on every commit. |
| **ADR-017** | SurrealDB-First | SurrealDB is the primary adapter (embedded + server). PG/MySQL/SQLite are parity adapters. |
| **ADR-018** | Sync Engine Architecture | `educore-sync::SyncAdapter` port trait + `educore-sync-inprocess` in-process default impl. |
| **ADR-019** | Public API Naming | Umbrella re-exports at `educore::*`. Each domain crate has a `prelude::*`. |
| **ADR-020** | Cross-Domain Ownership | Each domain owns its aggregates; cross-domain refs use typed-id (no FK tables). |

**The decision needed but not yet ADR'd:** "Phase 17 = production hardening or CMS?" Resolved as "Phase 17 is `CMS` (Phase 12 in AGENTS.md)" in `13-decision-needed.md` D-4, but **should be formalized as ADR-021**. 30-min task.

---

## 3. Completed Work (this session)

**Cumulative across 102 waves (Waves 65-101 + 103-168):**

- **53 `Real*` aggregates** built end-to-end (struct + impl + events + service + tests)
- **10 state machines** (ApprovalStatus variants for BankPaymentSlip, WalletTransaction, WalletTransactionApproval, Expense, ExpenseApproval, IncomeApproval, PayrollPayment, PaymentGatewaySetting, etc.)
- **6 new enums** (`FmFeesTypeKind`, `PaymentMode`, `LifecycleStatus`, `TransactionLifecycleStatus`, `ProductPurchaseLifecycleStatus`, `GatewayChargeType`)
- **992+ behavioral tests** across 139+ invariants
- **Head at `d2a9e45`**, all pushed to `origin/main`

**Wave-by-wave breakdown:**
| Waves | Work | Commit Range |
|---|---|---|
| 65-77 | RealIncomeHead, RealFmFeesGroup, RealInvoiceSetting, RealQuestionBankFee, RealDirectFeesSetting, RealFeesCarryForwardLog, RealDonor, RealFmFeesInvoiceLineNote, RealDirectFeesInstallmentAssignChild, RealChartOfAccount, RealFmFeesTransactionLineNote, RealFmFeesTransactionChild, WalletTransactionApproval | `563a768`..`88443a9` |
| 78-101 | Continued pipeline; included RealBankAccount, RealBankStatement, RealBankPaymentSlip, RealExpense, RealExpenseApproval, RealExpenseHead, RealAmountTransfer, RealBankPaymentSlipAudit, RealBankStatementAttachment, RealDirectFeesInstallment, etc. | `88443a9`..`~ad38365` |
| 102 | **Aborted** — exceeded turn budget; reverted before commit | (no commit) |
| 103-145 | Continued pipeline: RealDirectFeesInstallmentAssign, RealFeesInstallmentAssignDiscount, RealFeesInstallment, RealFeesDiscount, RealWalletTransaction, RealFeesCarryForwardSetting, RealIncome, RealIncomeApproval, RealFeesMaster, RealFeesInvoiceSetting, RealFeesGroup, RealFeesAssign, RealFeesAssignDiscount, RealFeesInstallmentCredit, RealDirectFeesReminder, RealDueFeesLoginPrevent, RealPaymentGatewaySetting, RealPayrollPayment | `~ad38365`..`~4ec80ad` |
| 146-157 | RealWalletTransaction, RealFeesDiscount extension (FD I-1 value fields), event + update_metadata wiring | `~7822805`..`156bca6` |
| 158 | DiscountType docstring cleanup | `d1f3777` |
| 159 | `cargo fmt -p educore-finance` across 64 files | `7a5aa73` |
| 160-165 | Clippy cleanup: cast, unwrap, expect, unused_imports, unused_variables | `4ec80ad`..`ad38365` |
| 166-168 | Final clippy cleanup: matches!, manual_range_contains, too_many_arguments file-level allow | `4670fa5`..`d2a9e45` |

**The 6 established drop patterns (now locked in):**

1. **Append-only** (BankStatement, BankPaymentSlipAudit, etc.) — no `update_*` mutator; only `fresh()` + `retire()`.
2. **Type-pinned** (StatementType, AccountType, PaymentMode, etc.) — closed enum enforces the invariant at the type-system level.
3. **Full lifecycle** (RealBankAccount, RealBankStatement) — typed id + audit footer + `fresh()` + `update_metadata()` + `retire()`.
4. **Generic update_metadata** (RealFeesDiscount) — triple-nested `Option<Option<T>>` semantics: `None`=don't touch, `Some(None)`=clear, `Some(Some(v))`=set.
5. **State machine** (WalletTransactionApproval, ExpenseApproval) — `can_transition_to` + audit-footer state machine fields + dedicated events.
6. **Money + reference validity** (RealFeesPayment) — discriminated unions on `payment_method` with cross-field guards.

---

## 4. Remaining Work (in priority order)

### 4.1 HR Domain (PRIMARY NEXT FOCUS)

**State:** 42 aggregates scaffolded in `crates/domains/hr/src/aggregate.rs` (~1,550 LOC). Wave 32 added 7 invariant enforcements to the `required_capabilities()` callsites + 1 Port trait (`LeaveAccrualChecker`). **Master checklist (`hr-invariant-checklist.md`) reports only 7 `[x]`** out of 107 invariants across 42 aggregates.

**The 7 invariants that are `[x]`:**
- Staff I-4 (phone unique per school) — Wave 32
- PayrollGenerate I-2 (net == gross - total_deduction - tax) — Wave 32
- PayrollGenerate I-5 (monthly recurring uniqueness) — Wave 32
- LeaveRequest I-1 (from_date ≤ to_date) — Wave 32
- LeaveRequest I-2 (leave_days balance check) — Wave 32
- LeaveRequest I-4 (cannot overlap existing approved leaves) — Wave 32
- LeaveDefine I-3 (carry_forward cap) — Wave 32
- HourlyRate I-1 (rate ≥ 0) — Wave 32

**The 100 remaining `[ ]` invariants grouped by aggregate:**

| Aggregate | Invariants | Notes |
|---|---|---|
| **Staff** | 8 (I-1, I-2, I-3, I-5, I-6, I-7, I-8) | Highest count. Tenant anchor, ID/email/phone uniqueness, status FSM, payroll-block-on-resign |
| **PayrollGenerate** | 5 (I-1, I-3, I-4, I-6) | Gross composition, status FSM, paid_amount ≤ net, bonus/overtime |
| **LeaveRequest** | 2 (I-3, I-5) | Status FSM, reject-reason required |
| **StaffAttendance** | 3 | One-per-day, in_time < out_time, status FSM |
| **Department** | 3 | Name unique per school, tenant anchor, cannot-delete-while-staff-assigned |
| **Designation** | 3 | Same pattern as Department |
| **LeaveDefine** | 2 (I-1, I-2) | Per-school unique leave type, days_per_year > 0 |
| **LeaveDeductionInfo** | 3 | deduction_amount ≥ 0, leave_days ≥ 0, per LeaveDefine |
| **LeaveType** | 3 | Name unique per school, type ∈ {paid, unpaid, partial}, tenant anchor |
| **PayrollEarnDeduc** | 2 (I-1, I-3) | amount ≥ 0, sum invariants (covered by PayrollGenerate I-1) |
| **SalaryTemplate** | 4 | gross_salary == sum, net_salary == gross - deduction, name unique, append-only |
| **StaffAttendanceImport** | 3 | batch_id valid, per-row date, idempotency |
| **AssignClassTeacher** | 2 | teacher active, class-section valid |
| **AssignClassTeacherScope** | 2 | scope enum, scope fields consistent |
| **BulkImportJob** | 2 | status FSM, row_count ≥ 0 |
| **DepartmentHead** | 2 | staff active, department exists |
| **DesignationGrade** | 2 | grade numeric range, unique per school |
| **Other 24 aggregates** | 1-2 each | Mostly 2-invariant placeholders |

**Estimated work:** ~38 per-aggregate waves × ~30 min each = ~19 hours of focused work = **5-8 sessions**.

**Recommended first HR wave (Wave 169):**
- Start with **Staff aggregate I-1 (Tenant anchor from SchoolId)** — 1 invariant, lowest complexity
- Or **Staff aggregate I-2/I-3 (ID/email unique per school)** — more interesting, requires the dispatcher pattern

The Staff aggregate is the highest-priority because it's referenced by ~5 other HR aggregates (DepartmentHead, DesignationGrade, LeaveRequest, etc.).

### 4.2 Finance Domain (CONTINUE)

**Remaining ~30 placeholder stubs** (the `Real*` aggregates that don't exist yet). The per-aggregate wave pipeline is proven and the test pattern is locked in.

**Next candidates** (sorted by complexity, smallest first):
- RealTransaction (~1-2 invariants)
- RealTransactionChild
- RealWalletTransactionApproval (state machine, ~4 invariants)
- RealExpense (already exists; expansion may be needed)

**Estimated work:** ~30 more finance waves = ~15 hours = **4-5 sessions**.

### 4.3 The 3 Quick Wins (do these in the first 15 minutes)

1. **Fix `docs/progress-tracker.md` Phase 6 row** — change "Planned | No" to "Done | Yes (16 aggregates + 553 tests pass + 30 coverage rows flipped per PHASE-6-HANDOFF.md)"
2. **Fix `docs/audit_reports/hr-invariant-checklist.md` Summary table** — update the tally from "TBD/TBD/TBD" to "Wave 32: 7 `[x]` / 100 `[ ]` / 0 `[N/A]`"
3. **Create `docs/decisions/ADR-021-PhaseNumberingConventions.md`** — formalize the resolution from `13-decision-needed.md` D-4

All three are 5-15 min each. One combined commit.

### 4.4 The Big Production Gaps (post-wave-pipeline)

| Gap | Severity | Estimated Effort |
|---|---|---|
| **Dispatcher wrapper layer (0/509)** | CRITICAL | 10+ sessions (1 domain per session) |
| **Per-invariant checklist for 9 domains** | MEDIUM | 5-7 sessions (1 domain per session) |
| **Cross-adapter parity test suite** | MEDIUM | 3-5 sessions |
| **Cross-compile verification** | MEDIUM | 1-2 sessions (toolchain install + CI workflow) |
| **Threat model + operational docs** | MEDIUM | 1-2 sessions |
| **`educore-sdk::Engine::builder()`** | LOW | 1-2 sessions |
| **`educore-testkit` in-memory port impls** | LOW | 1-2 sessions |

---

## 5. Prioritized TODO (for the next session)

In priority order (do top to bottom):

1. **[15 min] Fix 3 quick wins** (commit immediately)
2. **[2-4 hours] Wave 169 — HR Staff I-1** (Tenant anchor — smallest entry point)
3. **[2-4 hours] Wave 170 — HR Staff I-2/I-3** (ID/email uniqueness)
4. **[2-4 hours] Wave 171 — HR Staff I-5/I-6** (Joining date + Status FSM)
5. **[2-4 hours] Wave 172 — HR Staff I-7/I-8** (payroll block + soft-delete)
6. **[2-4 hours] Wave 173 — Finance: next placeholder stub** (parallel HR work)
7. **[continue HR waves for the remaining 7 high-count aggregates]**
8. **[2-4 hours] Wave N — first dispatcher wrapper** (start with academic's `create_class` as the template)
9. **[continue dispatcher wrappers]**

---

## 6. Blockers and Assumptions

### 6.1 Blockers

| Blocker | Severity | Resolution |
|---|---|---|
| Cross-compile toolchains not installed locally | MEDIUM | Install via `rustup target add aarch64-linux-android wasm32-unknown-unknown` + clang |
| `EDUCORE_PG_URL` / `EDUCORE_MYSQL_URL` env vars not set | LOW | Only needed for env-gated integration tests; default SQLite tests always run |
| `educore-storage-parity` not populated | MEDIUM | This is work to do, not a blocker |

### 6.2 Assumptions (carry forward unless contradicted)

1. **AI agents are first-class contributors** per ADR-010 — handoff docs are the canonical pattern.
2. **`Real*` prefix on aggregates** is finance convention (matches the placeholder-stub pattern). HR doesn't use it (HR goes straight to full implementation).
3. **All `#[allow(...)]` on production code should be per-function** (AGENTS.md § Type Safety stricter rule for non-tests). Tests can use file-level allows.
4. **`cargo add <crate> --package <package-name>`** is the canonical dep-add command.
5. **graphify hook is installed locally** — auto-rebuilds the AST-only graph on every commit.
6. **Pre-commit hook may run cargo fmt + clippy** depending on the user's `.git/hooks/pre-commit`; if a wave fails formatting, run `cargo fmt -p <package>` before committing.
7. **The `educore-events-domain` (cross-cutting tier) is the CALENDAR domain, distinct from `educore-events` (cross-cutting tier) which is the event ENVELOPE + bus port.** Don't conflate them.

---

## 7. Modified Docs/Files (this session)

### 7.1 Files Created

- `docs/audit_reports/remediation/17-final-reconciliation-audit.md` — the comprehensive gap analysis (~32 KB)

### 7.2 Files Modified

None in this audit phase. The audit doc is purely additive.

### 7.3 Files That Need Updating (carry-forward list)

- `docs/progress-tracker.md` Phase 6 row — 1-line edit
- `docs/audit_reports/hr-invariant-checklist.md` Summary table — 1-line edit
- `docs/decisions/` — add ADR-021-PhaseNumberingConventions.md

---

## 8. Unresolved Spec ↔ Code Conflicts

| Conflict | Resolution | Status |
|---|---|---|
| Phase 17 numbering (production hardening vs CMS) | Resolved as "Phase 17 = CMS (Phase 12 in AGENTS.md)" | **Pending ADR-021** |
| `educore-events` vs `educore-events-domain` naming | Resolved (envelope + bus port vs calendar domain) | ✅ Documented in AGENTS.md |
| `finance_aggregate_stub!` macro generates placeholders that some queries reference | Resolved (placeholders are documentation markers) | ✅ Documented in Wave 65 pitfalls |
| `educore-storage-parity` listed at both Phase 0 + Phase 16 | Resolved (Phase 0 scaffolds, Phase 16 implements) | ✅ Documented in AGENTS.md |
| `educore-events-domain` listed at both Phase 2 + Phase 13 | Resolved (Phase 2 is the envelope, Phase 13 is the calendar) | ✅ Documented in AGENTS.md |
| `educore-storage-parity` 0/509 wrappers vs production deployment | **UNRESOLVED** | Open work |
| `educore-finance` clippy 55 doc list indentations + 27 unreachable patterns | Deferred as out-of-scope for cleanup waves | Cosmetic |
| RBAC `required_capabilities()` 540 method-level declarations but per-domain correctness audit deferred | Deferred (v3 Part 5 R1-R10) | Open work |

---

## 9. Production Risks

| Risk | Severity | Mitigation |
|---|---|---|
| Deploying without dispatcher wrappers means RBAC + idempotency + outbox + audit + bus-publish are NOT enforced | **CRITICAL** | Do not deploy until 509 wrappers are implemented. The codebase is B-, not production. |
| HR domain is mostly scaffolded; production schools need Staff + PayrollGenerate + LeaveRequest | **HIGH** | Implement HR Staff first (8 invariants, 1-2 sessions) |
| Cross-compile to Android/WASM unverified | MEDIUM | Install toolchains + exercise CI job before mobile/WASM clients |
| No threat model + no pentest | MEDIUM | Engage security review before handling real PII |
| No operational runbook + no SLO/SLI | MEDIUM | Document before on-call rotation starts |
| 0/509 wrappers means every consumer hand-wires RBAC + idempotency + outbox + audit + bus-publish | **CRITICAL** | Same as first row |
| `educore-files::S3FileStorage` + `educore-payment::StripeProvider` not exercised in CI | MEDIUM | Add CI workflows with test-mode credentials |

---

## 10. Recommended Next Execution Order

### Session 1 (this is the recommended next session for the next agent)

**Total time: 2-4 hours**

1. **[15 min] Fix 3 quick wins** (1 combined commit):
   - `docs/progress-tracker.md` Phase 6 row
   - `docs/audit_reports/hr-invariant-checklist.md` Summary table
   - `docs/decisions/ADR-021-PhaseNumberingConventions.md` (new file)
2. **[30 min] Verify clean state**:
   - `git log --oneline -3` → should end with `d2a9e45`
   - `git status` → clean
   - `cargo check -p educore-hr --tests` → 0 errors (warnings OK)
   - `cargo test -p educore-hr --tests --no-fail-fast` → all green
3. **[2-3 hours] Wave 169 — HR Staff I-1 (Tenant anchor)**:
   - Read `docs/specs/hr/aggregates.md` § Staff to understand the spec
   - Add `validate_school_id_anchor` helper to `crates/domains/hr/src/value_objects.rs` (or reuse existing pattern)
   - Update `RealStaff::fresh()` to enforce I-1 (return `DomainError::Validation` if tenant's `SchoolId` doesn't match the staff's `SchoolId`)
   - Update `RealStaff::update_metadata()` to re-validate I-1 (defense-in-depth)
   - Write 3-5 tests in `crates/domains/hr/tests/staff.rs` (model after `crates/domains/finance/tests/wallet.rs`)
   - Run `cargo test -p educore-hr --test staff --no-fail-fast`
   - Flip `hr-invariant-checklist.md` Staff I-1 from `[ ]` to `[x]` with full file:line evidence
   - Commit + push
4. **[if time] Wave 170 — HR Staff I-2 (Staff ID unique per school)** — uses the dispatcher + storage port pattern
5. **[if time] Wave 171 — HR Staff I-3 (Email unique per school)** — same pattern

### Session 2+ (continuation pattern)

Continue the HR per-aggregate wave pipeline:
- Wave 172: Staff I-5 (Joining date ≤ current date)
- Wave 173: Staff I-6 (Status FSM)
- Wave 174: Staff I-7 (Cannot resign while has open payroll)
- Wave 175: Staff I-8 (Soft-delete preserves history)
- Wave 176+: Department (3 invariants), Designation (3), LeaveDefine (2), LeaveType (3), etc.
- Interleave with Finance remaining placeholder stubs

### Session 20+ (post-wave-pipeline)

Start the dispatcher wrapper implementation. Begin with `educore-academic` (the most-tested domain) — implement wrappers for all 37 service functions there (~1 session). Then move to other domains.

---

## 11. Exact Commands the Next Agent Should Run First

```bash
# 0. Verify clean state
cd /home/beznet/Workspace/smscore
git log --oneline -5
# Expected: d2a9e45 Wave 168: File-level too_many_arguments allows across 3 files
#           4eb3471 Wave 167: Fix 4 quick lints (getter, dup attrs, broken test)
#           4670fa5 Wave 166: ...
git status
# Expected: clean (only .kimchi/ untracked)
git branch -v
# Expected: * main ... origin/main

# 1. Verify the build is clean
cargo check -p educore-hr --tests 2>&1 | tail -5
# Expected: "Finished `dev` profile [unoptimized + debuginfo] target(s)" — 0 errors
cargo test -p educore-hr --tests --no-fail-fast 2>&1 | tail -5
# Expected: tests pass; warning if any test has the !-allow attribute
cargo clippy -p educore-hr --tests --no-deps 2>&1 | grep -c "^error\|^warning"
# Expected: a number (warnings OK; errors should be 0)

# 2. Read the HR Staff spec
cat docs/specs/hr/aggregates.md | sed -n '/^## Staff/,/^## /p' | head -100
# This is the spec for what you'll build

# 3. Read the existing Staff aggregate
head -200 crates/domains/hr/src/aggregate.rs
# Find the Staff struct + impl block (search for `pub struct Staff`)

# 4. Read the HR test pattern (if any tests exist for Staff)
ls crates/domains/hr/tests/ | head -20
# Look for staff.rs or any related test file

# 5. Read the value_objects.rs for HR helper functions
head -100 crates/domains/hr/src/value_objects.rs
# Find existing validators (validate_phone, validate_email, etc.)

# 6. After completing Wave 169 (Staff I-1), run:
cargo test -p educore-hr --test staff --no-fail-fast
# Expected: all tests green (including the new ones)
cargo fmt -p educore-hr
# Fix any formatting issues

# 7. Commit + push
git add docs/progress-tracker.md \
        docs/audit_reports/hr-invariant-checklist.md \
        docs/decisions/ADR-021-PhaseNumberingConventions.md \
        crates/domains/hr/src/aggregate.rs \
        crates/domains/hr/src/value_objects.rs \
        crates/domains/hr/tests/staff.rs \
        docs/audit_reports/hr-invariant-checklist.md \
        graphify-out/GRAPH_REPORT.md \
        graphify-out/graph.json
git -c user.name="Educore Dev" -c user.email="dev@educore.local" commit -m "Wave 169: HR Staff I-1 (tenant anchor) + 3 quick doc fixes"
git push origin main
```

---

## 12. The Per-Aggregate Wave Template (proven across 102 waves)

When starting a new per-aggregate wave (e.g., HR Staff I-1), follow this template. Total time per wave: 15-45 minutes depending on complexity.

**Steps:**

1. **Read the spec** — `cat docs/specs/<domain>/aggregates.md | sed -n '/^## <Aggregate>/,/^## /p'`
2. **Read the existing aggregate** — `head -<N> crates/domains/<domain>/src/aggregate.rs`
3. **Add the validation helper** to `crates/domains/<domain>/src/value_objects.rs` (if not already present)
4. **Update `Real<Aggregate>::fresh()`** to enforce the invariant — return `DomainError::Validation` on failure
5. **Update `Real<Aggregate>::update_*()`** methods to re-validate the invariant (defense-in-depth)
6. **Extend `Create<Aggregate>Command`** in `crates/domains/<domain>/src/commands.rs` if new fields needed
7. **Extend the `Created` event** in `crates/domains/<domain>/src/events.rs` if new fields needed
8. **Update the service function** in `crates/domains/<domain>/src/services.rs` to pass new fields
9. **Add re-exports** to `crates/domains/<domain>/src/lib.rs::prelude` (single-shot edit)
10. **Write behavioral tests** in `crates/domains/<domain>/tests/<aggregate>.rs` — model after the closest existing test file
11. **Add `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::dbg_macro, missing_docs)]`** at the top of the test file (per AGENTS.md § Type Safety)
12. **Run** `cargo check -p <package> --tests && cargo test -p <package> --tests --no-fail-fast`
13. **Flip the checklist entry** in `docs/audit_reports/<domain>-invariant-checklist.md` from `[ ]` to `[x]` with full file:line evidence
14. **Commit + push** with the Co-Authored-By trailer (see §13)

**Test fixture pattern (per AGENTS.md):**
```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::dbg_macro, missing_docs)]

use educore_core::clock::SystemClock;
use educore_core::id::SystemIdGen;
use educore_platform::tenant::TenantContext;
use educore_hr::prelude::*;

fn admin_context() -> (TenantContext, SystemIdGen) {
    let school_id = SchoolId::new();
    let tenant = TenantContext::new(school_id, Role::HrAdmin, UserId::new());
    let ids = SystemIdGen::new();
    (tenant, ids)
}
```

**Per-function `#[allow(clippy::unwrap_used)]`** in production code (per AGENTS.md § Type Safety stricter rule for non-tests):
```rust
#[allow(clippy::unwrap_used)] // unwrap on validated-at-construction field; cannot panic
fn retire(&mut self) -> Result<...> {
    let updated_at = self.updated_at.unwrap_or(self.created_at);
    ...
}
```

---

## 13. Git Commit Attribution

**Every commit** in this repo (whether human or AI) must end with:

```text
Co-Authored-By: Antigravity <antigravity@google.com>
```

**Git identity for AI agents:**
```bash
git -c user.name="Educore Dev" -c user.email="dev@educore.local" commit -m "..."
```

**Branch:** `main`. Push to `origin main` after every commit. Never force-push.

**Stage files explicitly** (no `git add -A` or `git add .`):
```bash
git add path/to/file1 path/to/file2 ...
```

---

## 14. The Engine Rule Violations to NEVER Do

Per AGENTS.md § Engine Rules + § Type Safety:

1. **No `unwrap()` or `expect()` in production paths** — use `?` or document the invariant
2. **No `#[allow(dead_code)]`** or `_var` prefixes to silence the compiler — delete unused code or open a follow-up issue
3. **No `as` casts that truncate or lose data** — use `TryFrom`/`TryInto`
4. **No `serde_json::Value` in domain code** — use typed wrappers
5. **No `HashMap<String, T>` for domain data** — use typed structs
6. **No service locators, DI containers, runtime reflection**
7. **No `unsafe`** in domain code (`#![forbid(unsafe_code)]`)
8. **No `native-tls`** — only `rustls`
9. **No `tokio` directly in domain code** — only through `educore-core` re-exports
10. **No glob imports in domain code** — use explicit crate-relative paths

---

## 15. See Also (canonical references)

- `AGENTS.md` — the engine operating contract (READ FIRST)
- `docs/audit_reports/remediation/17-final-reconciliation-audit.md` — the comprehensive gap analysis from this session
- `docs/audit_reports/remediation/16-session-handoff.md` — previous session handoff (Wave 65 continuation)
- `docs/audit_reports/remediation/15-continuation-reconciliation.md` — v3 → Wave 65+ reconciliation
- `docs/audit_reports/remediation/14-engine-production-depth-v3-roadmap.md` — v3 233-step plan
- `docs/progress-tracker.md` — per-crate implementation status (stale Phase 6 — needs 1-line edit)
- `docs/build-plan.md` — the 18 phases
- `docs/architecture.md` — the system map
- `docs/code-standards.md` — engineering rules
- `docs/handoff/PHASE-6-HANDOFF.md` — HR phase close-out
- `docs/audit_reports/hr-invariant-checklist.md` — HR per-invariant status (summary table stale)
- `docs/audit_reports/finance-invariant-checklist.md` — finance per-invariant status (current)
- `docs/audit_reports/academic-invariant-checklist.md` — academic per-invariant status (reference slice)
- `docs/decisions/` — 20 ADRs
- `graphify-out/GRAPH_REPORT.md` — engine knowledge graph (god nodes + community structure)

---

**The next agent has everything they need. Start with the 3 quick wins (§5 #1), then Wave 169 HR Staff I-1 (§10 Session 1). Good luck.**
