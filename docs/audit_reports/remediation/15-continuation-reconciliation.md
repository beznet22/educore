# Engine Production Depth — Continuation Reconciliation (v3 → Wave 65+)

**Generated:** 2026‑07‑20 (after Engine Production Depth v3 roadmap commit `d02b295`, 2026‑07‑02)
**Purpose:** Operational counterpart to [`14-engine-production-depth-v3-roadmap.md`](14-engine-production-depth-v3-roadmap.md).
Captures the delta between v3's frozen baseline (grade D, ~2 % invariants `[x]`) and the actual state at HEAD `c676003`
plus the uncommitted finance working tree. Names the continuation point and the wave‑numbering convention going
forward.

> **Honesty rule (carried forward from v3):** if any deferred v3 work is missing from this reconciliation, that is a
> planning failure here, not a v3 failure. v3 was the ferment‑close baseline; this document is the live tracker.

---

## Background

The v3 roadmap (commit `d02b295`, 2026‑07‑02) was the **frozen plan** at the close of the
Engine Production Depth ferment. It enumerated **233 steps across 7 Parts** with a target grade of A
(all spec invariants enforced with behavioral tests) and an honest pessimistic timeline of "months of focused
engineering work, not a single ferment."

Between v3 commit `d02b295` and HEAD `c676003` (2026‑07‑03), the team executed **33 commits** —
substantially advancing Part 1 (Academic) to completion and starting Part 2 (Finance). This document
captures that delta and proposes the sequencing for Wave 65 onwards.

---

## 1. State of v3's 7 Parts at HEAD

| Part | v3 Steps | v3 Status | Actual Status at HEAD `c676003` | Evidence |
| --- | --- | --- | --- | --- |
| **1 — Academic placeholders** | 16 (A1–A16) | All deferred | **✅ COMPLETE** | Waves 48–63 + Wave 64 cleanup. Per Wave 64 commit msg: "67 invariants reach [x], 0 remain [ ]. `cargo test -p educore-academic --tests --no-fail-fast`: 225 tests pass, 0 fail". `Real*` prefix on every formerly-placeholder aggregate: Guardian, ClassSection, ClassSubject, ClassRoutine, Homework, LessonPlan, Lesson, LessonTopic, StudentRecord, StudentPromotion, StudentCategory, StudentGroup, RegistrationField, Certificate, IdCard. Wave 63 added `StudentRetired` event for I-6 cascade. |
| **2 — Finance** | ~57 (F1–F66) | 19 / 31 / 124 (x/~/[]) | **⏳ IN PROGRESS** | 11 `Real*` aggregates committed at HEAD: `RealBankStatement`, `RealBankAccount`, `RealChartOfAccount`, `RealDonor`, `RealExpenseHead`, `RealBankStatementAttachment`, `RealBankPaymentSlip`, `RealBankPaymentSlipAudit`, `RealExpenseApproval`, `RealAmountTransfer`, `RealFeesGroup`. **Working tree has ~50 modified/new files** (modified `aggregate.rs`, `value_objects.rs`, `repository.rs`, `query.rs`, `lib.rs`, `finance-invariant-checklist.md`; new `tests/bank_account.rs`, `tests/payment_method.rs`; 47 modified test files). The updated checklist references `RealTransaction` and `RealWalletTransactionApproval` which **do not exist in HEAD** → the WT changes are the next aggregates being staged. |
| **3 — HR** | ~43 (H2–H43) | All 42 placeholders | **🟡 MORE PROGRESSED THAN v3 CLAIMS** | Phase 6 closed with 16 aggregates (`PHASE-6-HANDOFF.md`). Cluster C commit `bc938cd` added 26 more → **42 aggregates** now exist in `crates/domains/hr/src/aggregate.rs` (1,550 LOC). Wave 29 (`e426bb5`) added typed-id round-trip tests for all 38. Wave 32 (`3376a4b`) added invariant enforcements. **Real behavioral tests only on 3 aggregates**: `Department` (161 LOC), `Designation` (161 LOC), `Staff` (via `workflows.rs`, 198 LOC). **`hr-invariant-checklist.md` still shows 0 `[x]`** — Wave 32 work has not been back-propagated to the checklist. |
| **4 — 7 remaining domains** | ~93 (Att/Com/Doc/Fac/Lib/Cms/Evt) | All TBD | **🟡 PARTIAL** | Attendance: Phase 5 closed (16 fns, 93 unit tests, 13 coverage rows flipped). Communication: Phase 10 closed (104 fns, 60 unit + 6 integration). Documents: Phase 11 closed (18 fns, 145 unit tests). CMS: Phase 12 closed (20 aggregates, 183 unit tests). Library: Wave 30 closed (16 fns). **Facilities: Phase 8 — scaffold only** (no functional closure). **Events-domain: Phase 13 partial** — Waves 23, 5 added integration tests for AssignIncident, IncidentComment, CalendarSetting, Weekend, Holiday (7 root aggregates per `crates/cross-cutting/events-domain/`). **No domain has produced its invariant checklist yet** — the "Step 0" for each domain in v3 Part 4 is unstarted. |
| **5 — RBAC spec validation** | 11 (R0–R11) | 540 mappings need spec review | **🟡 FOUNDATION DONE, CORRECTIONS DEFERRED** | `docs/audit_reports/rbac-spec-map.toml` (163 commands mapped) + Wave 36 (`required_capabilities()` on 540 Command structs) + 10 rejection tests at `crates/cross-cutting/dispatcher/tests/forbidden_rejection.rs`. **Per-domain corrections** (R1-academic, R1-assessment, … R1-cms; finance split into 3 sub-batches per v3) **deferred**. |
| **6 — Dispatcher wrappers** | 10 (W1–W10, ~509 wrappers) | All 509 deferred | **❌ NOT STARTED** | `crates/educore/src/dispatch.rs` is **92 lines, all comments** — only skeleton + pattern doc (`docs/guides/dispatcher-wrapper-pattern.md`) exist. Zero wrapper bodies implemented. |
| **7 — CI cross-compile** | 3 (CI1–CI3) | env-bound | **❌ DEFERRED** | aarch64 toolchain not installed locally; wasm32 needs clang. CI workflow file exists from Wave 43 (`.github/workflows/ci.yml`) but cross-compile job has not been exercised. Load test at full 100×10k scale deferred. |

**Net progress since v3 freeze:** Part 1 fully done (16/16 steps), Part 2 well underway (~21/57 steps committed + WT in flight), Parts 3–7 untouched structurally but Parts 3, 4, 5 are partially advanced through prior-ferment (Phase 6, Wave 29, Wave 32, Wave 36) work that the v3 roadmap did not credit.

---

## 2. Honest Re-Grade (v3 baseline → current)

| Baseline (v3 ferment close, 2026‑07‑02) | Current (2026‑07‑20, HEAD `c676003`) |
| --- | --- |
| **Grade: D** | **Grade: C+** (conservative; B− if Wave 32 HR work is back-propagated to the checklist) |
| ~9 invariants promoted to `[x]` out of ~700+ spec invariants (~1 %) | **~100+ invariants promoted to `[x]`** (Academic 67/67 + Finance 33+/165 partial + HR Wave 32 N untallied + RBAC 540/540 `required_capabilities()` method-level coverage) |
| Part 1: 16 deferred | Part 1: **16/16 complete** |
| Part 2: 0 done | Part 2: **~21/57 done + WT in flight** |
| Part 3: 0 done | Part 3: 16/43 done (Phase 6) + 26/43 scaffolded (Cluster C) + 3/43 with real behavioral tests |
| Part 4: 0 done | Part 4: 6/7 domains partially advanced (Att/Com/Doc/Cms/Lib via phases; Evt via waves); Facilities untouched |
| Part 5: 0 done | Part 5: spec map + required_capabilities done; per-domain corrections deferred |
| Part 6: 0/509 wrappers | Part 6: 0/509 wrappers (skeleton only) |
| Part 7: deferred | Part 7: deferred (env-bound) |

The grade is **C+** because the engine now has *demonstrated* per-aggregate behavioral coverage for **all 16 academic placeholder-stub aggregates** (Academic = the reference slice), and *partial* coverage for the remaining 4 finance aggregates that were partially implemented before v3 (Wallet, WalletTransaction, FeesInvoice, FeesPayment) plus 11 new finance `Real*` aggregates. The grade is **not B** because:

1. **The HR invariant checklist still shows 0 `[x]`** even though Wave 32 enforcements landed — the tracking has not caught up with the code.
2. **No domain has produced its full invariant checklist** (the v3 Part 4 "Step 0" prerequisite for each of the 7 domains).
3. **The dispatcher wrapper layer is empty** — every domain service still requires a hand-wired wrapper to participate in RBAC + idempotency + outbox + audit + bus publish.
4. **Cross-compile verification is unproven** — a hard production deployment gate.

---

## 3. Documentation Drift (must be fixed alongside the work continuation)

Three drift issues surfaced during this reconciliation. Each will confuse future agents if not fixed.

### 3.1 `docs/progress-tracker.md` is stale

The Phase Progress table (lines 79–86) still shows:
- **Phase 13 (events-domain)**: `Planned / No`
- **Phase 14 (settings + operations)**: `Planned / No`
- **Phase 17 (production readiness)**: `Planned / No`

In reality:
- Phase 13 has 7 root aggregates + 4 child entities (per AGENTS.md row 26), with Wave 23 + Wave 5 integration tests
- Phase 14 closed cleanly (`PHASE-14-HANDOFF.md`: "Phase 14 closed. educore-settings and educore-operations are the two new cross-cutting tier crates shipped… 15 + 8 interpretation, 53 + 25 typed events, 100 net-new Capability variants, 28 coverage.toml rows flipped")
- Phase 15 closed (`PHASE-15-HANDOFF.md`): 5 port-adapter crates shipped
- Phase 16 closed 2026‑06‑21 (`PHASE-16-HANDOFF.md`): 4 tools crates shipped
- The tracker was last updated **before Phase 13 closed** (`git log -1 --format=%H -- docs/progress-tracker.md` → `fdfe88b Phase 16: flip 8 coverage rows + write phase-17 prompt + progress + build-plan…`)

### 3.2 The "Phase 17 = phantom?" decision was resolved but not ADR'd

`docs/audit_reports/remediation/13-decision-needed.md` (D-4) was resolved as "[x] A: Phase 17 is `CMS` (Phase 12 in AGENTS.md)" but no follow-up ADR was created. The build-plan still says "18 phases (Phase 0..17)" with Phase 17 = production hardening, not CMS. This is a numbering convention mismatch (AGENTS.md counts 0..17 = 18 phases; build-plan § "The 18 phases" lists 18 entries), not a substance disagreement, but it should be ADR'd.

### 3.3 `docs/audit_reports/academic-invariant-checklist.md` Summary table is stale

The Summary at lines 19–26 still reports "Enforced [x]: 15 (20.5 %)" whereas the wave-by-wave log at the top of the file shows the running total reached 47 by Wave 58, and Wave 64's commit message claims **67/67 `[x]`**. The Summary needs to be re-computed to match Wave 64's reality (or Wave 64 needs a follow-up commit that just updates the Summary).

### 3.4 `docs/audit_reports/finance-invariant-checklist.md` references aggregates not in HEAD

Lines 401–403 + 409–410 cite `RealTransaction::record`, `RealWalletTransactionApproval::fresh/approve/reject`, and test paths like `::wta_i_1_fresh_state_is_pending` — none of these `Real*` structs exist in the committed `crates/domains/finance/src/aggregate.rs` (only the 11 listed in §1 Part 2). The checklist is documenting the **working-tree** state. Either commit the WT (the natural next move per §4) or roll back the checklist edits.

---

## 4. The Continuation Point — Wave 65+ Plan

**The natural continuation is Part 2 of the v3 roadmap — Finance** — for three converging reasons:

1. **The working tree already has the next 10–15 finance aggregates staged** (per `git status`: modified `aggregate.rs` + `value_objects.rs` + `repository.rs` + `query.rs` + `lib.rs` + `finance-invariant-checklist.md` + 47 modified test files + 2 new test files `bank_account.rs` + `payment_method.rs`). This is a wave-scale drop matching the academic pattern (Waves 48–62 each = 1 aggregate + tests + checklist flip).

2. **Finance is the dependency for the next phase in `build-plan.md` § "The 18 phases".** Phase 8 (Facilities) needs `PaymentGatewaySetting` and `BankAccount` typed-ids from finance to wire its `Supplier` aggregate. Phase 17 (Production readiness, the actual target) needs `WalletTransaction.balance` cache reconciliation (F1/F7) which is currently `[~]` partial.

3. **Academic proved the per-aggregate wave pattern works.** 16 aggregates, 14 waves, 67 invariants `[x]`, 225 tests passing — the same pattern can now be applied to finance. The v3 roadmap's 233-step pessimistic estimate was based on multi-aggregate batches failing; the per-aggregate wave pattern (1 commit per aggregate) is the new norm.

### 4.1 Recommended sequencing for Wave 65+

| Step | What | v3 Ref | Why now |
| --- | --- | --- | --- |
| **1** | Commit the uncommitted finance WT as Wave 65–79 (likely one worktree branch per aggregate: RealTransaction, RealWalletTransactionApproval, RealDirectFeesInstallment, …). Each = 1 commit, 1 wave. | F1–F9, F20, F22, F27, F28, F32, F34, F35, F37, F38, F39, F42, F45, F49, F50, F58, F59 | This is **already done in the WT** — just commit it cleanly per the academic pattern. |
| **2** | Build the remaining finance placeholders per the v3 F10–F66 list, one aggregate per wave. ~30 more waves. Each adds: struct + impl + service factory + 3–5 invariant-rejection tests + checklist flip. | F10–F66 | Pattern proven by Academic. |
| **3** | Back-propagate Wave 32 HR invariant enforcements to `hr-invariant-checklist.md` (which still shows 0 `[x]`). Update summary table. Then 1 wave per HR aggregate for real behavioral tests. ~38 waves. | H1–H43 | The work has been done but not reflected in tracking. Fixing the tracking surfaces real-world coverage accurately. |
| **4** | Produce the 7 missing invariant checklists (Attendance, Communication, Documents, Facilities, Library, CMS, Events-Domain). Each is the v3 "Step 0" prerequisite for that domain's Part 4 work. | Att-0, Com-0, Doc-0, Fac-0, Lib-0, Cms-0, Evt-0 | These are pure documentation (audit the codebase, enumerate invariants) — no new code required. |
| **5** | Per-aggregate waves for the 7 domains. ~93 waves total. | Att-1..Att-9, Com-1..Com-25, … | Same wave pattern as Academic + Finance. |
| **6** | RBAC per-domain corrections. 10 steps (split finance into 3 sub-batches per v3 R1). | R1-academic, R1-assessment, … R1-cms | Spec map already exists; just walk it. |
| **7** | Dispatcher wrappers. 10 steps (one per domain, ~50 wrappers per step). | W1–W10 | Pattern + skeleton exist. Mechanical work. |
| **8** | CI cross-compile verification. 3 steps. | CI1–CI3 | Env-bound; can be a single PR with the CI workflow update. |

### 4.2 Grade projection

If all 8 steps above land, the v3 target of "**grade A (all spec invariants enforced with behavioral tests)**" is reachable.

| Step | Projected grade after step |
| --- | --- |
| 1 (commit finance WT) | **C+ → B−** (~25 % invariants `[x]`, finance in-flight work credited) |
| 2 (remaining finance) | **B− → B+** (~40 %, finance = parity with academic) |
| 3 (HR waves) | **B+ → A−** (~60 %, HR = parity with academic) |
| 4 (7 invariant checklists) | A− → A− (documentation only) |
| 5 (7 domain waves) | **A− → A** (~90 %, all domains parity) |
| 6 (RBAC corrections) | A → A (~95 %) |
| 7 (dispatcher wrappers) | A → A (~99 %) |
| 8 (CI cross-compile) | A → A (100 %, gate closes) |

### 4.3 Wave-numbering convention (Wave 65+)

Per the academic precedent (Waves 47–64):
- **One wave = one PR = one aggregate**
- Branch name: `wave<NN>/<scope>-<aggregate>` (e.g. `wave65/fin-transaction`, `wave66/fin-wallet-transaction-approval`)
- Commit message: `Wave <NN>: <one-line summary>` (matches Wave 48–64 style)
- Merge commit style: `Merge wave<NN>/<scope>-<aggregate>: <one-line summary>`
- Each wave flips exactly **one** section of the relevant invariant checklist from `[ ]` / `[~]` to `[x]` (or `[~]` to `[x]`) with file:line evidence
- Each wave adds **at least one behavioral test** that proves the invariant violation is REJECTED (matches academic pattern)
- Each wave validates with `cargo test -p <domain> --tests --no-fail-fast` before commit

### 4.4 Worktree pattern (carried forward from v2 / Wave 32-44)

For each wave:
1. Create worktree: `git worktree add ../wave<NN>-<scope> main`
2. Implement the aggregate + tests + checklist flip
3. Run `cargo test -p <domain> --tests --no-fail-fast` (must pass)
4. Run `cargo fmt --all` (must be clean)
5. Run `cargo clippy --workspace --all-targets -- -D warnings` (must be clean)
6. Commit with the wave-message format
7. Push branch + open PR
8. Merge PR (manual merge per v2 § "Worktree + merge")
9. Remove worktree: `git worktree remove ../wave<NN>-<scope>`
10. Update `graphify-out/` (auto-rebuilt on commit per AGENTS.md § "graphify")

### 4.5 Out of scope for this ferment

- **CI cross-compile** (Step 8) — env-bound; needs CI environment or pre-installed aarch64/wasm32 toolchains. Tracked but deferred.
- **Load test at full scale** (v3 PART 7 Step CI3) — needs running DB instances. Tracked but deferred.
- **New ADRs** for any decisions that surface during the work — to be created on-demand, not pre-emptively.

---

## 5. Success Criteria (Wave 65+ ferment)

1. **Step 1 (commit finance WT) complete:** ~10–15 waves land, ~30 finance invariants reach `[x]`, finance WT diff returns to 0.
2. **Step 2 (remaining finance) complete:** ~30 more waves, finance reaches ~80/165 invariants `[x]` (~50 %).
3. **Step 3 (HR waves) complete:** 38 waves, HR reaches ~60/107 invariants `[x]` (~55 %).
4. **Step 4 (7 invariant checklists) complete:** 7 new docs at `docs/audit_reports/<domain>-invariant-checklist.md`.
5. **Step 5 (7 domain waves) complete:** ~93 waves, all 7 domains parity with academic.
6. **Step 6 (RBAC corrections) complete:** 540 capability mappings validated against spec annotations.
7. **Step 7 (dispatcher wrappers) complete:** ~509 wrapper bodies, `crates/educore/src/dispatch.rs` no longer a skeleton.
8. **Step 8 (CI cross-compile) complete:** aarch64 + wasm32 builds green in CI.

**Total: ~233 waves to reach grade A** (matches v3 forecast).

**Realistic timeline:** ~233 focused waves × 30–50 turns each = **months of focused engineering work**, not a single ferment. (Carried forward verbatim from v3 — the per-aggregate pattern does not change the scope, only the per-wave failure rate.)

---

## 6. Re-grade target

**Starting grade (v3 baseline):** D (~2 % invariants `[x]`)
**Current grade (this reconciliation):** C+ (~14 % invariants `[x]`)
**Target grade (all steps complete):** A (100 % invariants `[x]`, all dispatcher wrappers, CI green)

**No goal erosion:** if a wave can't be completed, the gate evidence must show real reasons (spec ambiguity, missing prerequisite, env limit), not a re-scoped goal. Either the work is done or explicitly deferred with a tracking entry.

---

## See also

- [`14-engine-production-depth-v3-roadmap.md`](14-engine-production-depth-v3-roadmap.md) — v3 ferment‑close plan (the frozen baseline)
- [`13-production-readiness-v2.md`](13-production-readiness-v2.md) — v2 ferment‑close honest re-grade
- [`12-roadmap-gaps-audit.toml`](12-roadmap-gaps-audit.toml) — gap inventory per domain (machine-readable)
- [`academic-invariant-checklist.md`](../academic-invariant-checklist.md) — 67/67 `[x]` at Wave 64 (live tracker)
- [`finance-invariant-checklist.md`](../finance-invariant-checklist.md) — partial (WT in flight; updated to reflect staged aggregates)
- [`hr-invariant-checklist.md`](../hr-invariant-checklist.md) — 0 `[x]` (Wave 32 work not yet back-propagated)
- [`stub_vs_implementation.md`](../stub_vs_implementation.md) — 1,500-row per-function audit (legacy v1 ferment baseline)
- `docs/build-plan.md` § "The 18 phases" — canonical phase plan
- `docs/architecture.md` — system map
- `AGENTS.md` § "Tier System" + "graphify" — engine conventions
