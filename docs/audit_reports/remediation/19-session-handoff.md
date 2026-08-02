# Session Handoff — Wave 171 (next agent pickup)

**Generated:** 2026-08-02, end of session (commit `f8e6c3e`)
**For:** The next session agent. **Zero prior context assumed.**
**Predecessor:** [`18-session-handoff.md`](18-session-handoff.md) — read FIRST for repository identity, 20 ADRs summary, completed work table (Waves 65–168), remaining work by priority, blockers + assumptions, modified files, unresolved conflicts, production risks, recommended next execution order, exact first commands, commit attribution, engine rules.
**Companion:** [`17-final-reconciliation-audit.md`](17-final-reconciliation-audit.md) — full 12-category gap analysis (architecture, missing features, technical debt, performance, security, testing, portability, API, workflow, adapter, migration, operational), prioritized 10-session remediation roadmap, production deployment checklist.

---

## 0. TL;DR (read this first)

**Where Wave 171 left off:** Head at `f8e6c3e` on `main`, all pushed to `origin/main`. The HR crate's **Staff aggregate is now fully invariant-enforced** (8 of 8 spec invariants `[x]`); 134 HR tests pass (up from 107). The working tree is clean except for `.kimchi/` and harness session HTML.

**What Wave 171 shipped:**

1. **Working-tree reconciliation (commit `13e5fe4`):** committed the 19 uncommitted Wave 169 files (17 retired `docs/handoff/PHASE-X-HANDOFF.md` files + a benign finance `match → matches!` refactor + regenerated `graphify-out/GRAPH_REPORT.md`). The per-phase handoff docs are now archived to git history; the new audit_reports/remediation/17- and 18- docs supersede them.
2. **Spec↔checklist drift documentation (commit `8f32faf`):** added a "Spec Reconciliation (Wave 171)" section to `hr-invariant-checklist.md` recording the drift between `docs/specs/hr/aggregates.md` Staff section (8 numbered invariants with specific semantics) and the checklist's Staff row (8 entries with different numbering/wording). 4 of 8 rows had wording mismatches; 2 spec invariants (#2 UserId binding, #8 leave quotas) were missing from the checklist entirely.
3. **HR Staff I-1 + I-2 + I-3 + I-5 (commit `defff6d`):** 4 invariants flipped with full file:line evidence. Added `validate_joining_date_not_future` helper + `tests/staff.rs` (8 tests). Total HR tests: 107 → 115.
4. **HR Staff I-6 (commit `d7c8eb0`):** the status state machine. Added `StaffStatus::can_transition_to` FSM helper + 5 mutator methods (`suspend`, `reinstate`, `resign`, `terminate`, `retire`) + 4 new event types (`StaffReinstated`, `StaffResigned`, `StaffTerminated`, `StaffRetired`) + 10 behavioral tests. Total HR tests: 115 → 125.
5. **HR Staff I-7 + I-8 (commit `f8e6c3e`):** soft-delete with cross-aggregate reference check + leave-quota guard. Added `StaffReferenceChecker` port trait + `delete_staff` service function + `DeleteStaffCommand` + `Staff::soft_delete` + `Staff::set_leave_quotas` + 9 behavioral tests. Total HR tests: 125 → 134.
6. **Docs + handoff (this commit):** updated `progress-tracker.md` (12 phase-handoff link replacements + HR workspace row update) + this handoff doc.

**What's left:**

1. **HR domain — 41 aggregates still need real implementation.** The HR crate has 42 placeholder-stub aggregates; only Staff (8 invariants) is fully done. **92 `[ ]` invariants remain** across the other 41 aggregates. The next waves should continue the per-aggregate pattern on the highest-count aggregates: PayrollGenerate (6), LeaveRequest (2 remaining: I-3, I-5), Department (3), Designation (3), LeaveDefine (2 remaining), LeaveType (3), PayrollEarnDeduc (2 remaining), SalaryTemplate (4), etc.
2. **Finance domain — ~30 placeholder stubs remain.** Continue the per-aggregate wave pipeline.
3. **2 stale checklist rows** likely need normalization across the other 41 HR aggregate entries (the same spec↔checklist drift pattern likely recurs).
4. **0/509 dispatcher wrappers** — still the #1 production gap; deferred to post-wave sessions.
5. **Cross-compile verification, parity suite population, SDK facade, threat model** — all deferred per Wave 169 scope.

**The first 5 commands to run are in §11 below.**

---

## 1. Repository Identity

(Same as `18-session-handoff.md` §1 — read that. The Educore brand, `educore` umbrella, `educore-<name>` packages, `crates/<tier>/<name>/` directories, 5-tier system, 37 packages, and naming rules are unchanged.)

**Changes since Wave 169:**

- 17 `docs/handoff/PHASE-X-HANDOFF.md` files retired (archived to git history). All references in `progress-tracker.md` updated to point at `docs/audit_reports/remediation/18-session-handoff.md` (with a `Phase N archived for git history` annotation).
- `docs/audit_reports/remediation/19-session-handoff.md` added (this doc).

---

## 2. Architectural Decisions Made (Wave 171)

No new ADRs in Wave 171. The decisions made during the wave were all per-aggregate enforcement choices that follow existing patterns:

| Decision | Rationale | Reference |
|---|---|---|
| **Spec is the source of truth, not the checklist** | The checklist wording was a pre-Wave 32 artifact with 4-of-8 rows mismatching the spec. Per AGENTS.md § Spec folder layout + ADR-001 (DDD aggregate invariants are spec-defined), the spec wins. | `docs/audit_reports/hr-invariant-checklist.md` § Spec Reconciliation |
| **Staff I-1 (tenant anchor) is structurally enforced, not runtime-validated** | The `hr_typed_id!` macro makes it impossible to construct a `StaffId` with the wrong school, and `Staff::fresh` derives `school_id: id.school_id()`. Adding a runtime check would be redundant. | `aggregate.rs` line ~92 |
| **Staff I-6 FSM transitions via `StaffStatus::can_transition_to`** | Matches the `LeaveStatus::can_transition_to` pattern from Wave 32 / Phase 6. Refactored from `match` to `matches!` macro per the Wave 168 convention. | `value_objects.rs` StaffStatus impl |
| **Staff I-7 enforces the spec wording (no-hard-delete-while-referenced), not the checklist wording (resign-while-open-payroll)** | The checklist wording doesn't match the spec; the cross-aggregate reference check is the spec-faithful interpretation. The checklist row title is now updated to match. | `services.rs` StaffReferenceChecker port + `delete_staff` service |
| **Soft-delete (`active_status = Retired`) is distinct from FSM `retire()` (`status = Retired`)** | Spec #7 is "cannot hard-delete while references exist"; spec #6 has `retire()` as a terminal FSM transition. These are orthogonal concerns; conflating them would lose audit history. | `aggregate.rs` Staff::soft_delete vs Staff::retire |
| **StaffReferenceChecker takes a `&mut Staff` in `delete_staff`** | Mirrors the `approve_leave(&mut LeaveRequest, ...)` pattern from Phase 6. The dispatcher loads the aggregate, the service mutates it. | `services.rs` delete_staff signature |

---

## 3. Completed Work (Wave 171 — 5 commits)

**Cumulative HR invariant tally:**

- Before Wave 171: 8 `[x]` / 0 `[~]` / 99 `[ ]` / 0 `[N/A]` (Wave 32 baseline — 1 Staff + 2 PayrollGenerate + 3 LeaveRequest + 1 LeaveDefine + 1 HourlyRate)
- After Wave 171: **15 `[x]` / 0 `[~]` / 92 `[ ]` / 0 `[N/A]`**
- **Staff aggregate: 8 of 8 spec invariants `[x]`** (the only HR aggregate fully done)

**Commit range:**

| Commit | Chunk | Description |
|---|---|---|
| `13e5fe4` | 1 | Wave 169 leftovers: retire phase handoffs + finance matches! refactor + graphify regen (19 files: 17 docs deleted + 1 finance refactor + 1 graphify regen) |
| `8f32faf` | 2 | Wave 171 (Chunk 2): document Staff spec↔checklist drift (additive section in `hr-invariant-checklist.md`) |
| `defff6d` | 3 | Wave 171 (Chunk 3): HR Staff I-1 + I-2 + I-3 + I-5 (4 invariants, 8 new tests, 1 new validator helper) |
| `d7c8eb0` | 4 | Wave 171 (Chunk 4): HR Staff I-6 (status FSM, 10 new tests, 4 new events, 5 new mutator methods) |
| `f8e6c3e` | 5 | Wave 171 (Chunk 5): HR Staff I-7 + I-8 (delete_staff + leave quotas, 9 new tests, 1 new port trait) |
| (this) | 6 | Wave 171 (Chunk 6): docs + handoff (12 progress-tracker link fixes, HR workspace row update, this doc) |

**HR test count progression (educore-hr crate only — verified at commit `13e5fe4` via `git worktree`):**

| Checkpoint | HR tests passing | Notes |
|---|---|---|
| Pre-Wave-171-code baseline (commit `13e5fe4`) | **107** | staff.rs didn't exist yet; 36+ placeholder test files |
| After Wave 171 Chunk 1 | 107 | cleanup commit, no code |
| After Wave 171 Chunk 2 | 107 | docs only |
| After Wave 171 Chunk 3 | 115 | +8 tests in `tests/staff.rs` (I-1, I-2, I-3, I-5) |
| After Wave 171 Chunk 4 | 125 | +10 tests (FSM transitions I-6) |
| After Wave 171 Chunk 5 | **134** | +9 tests (I-7 delete + I-8 quota) |
| After Wave 171 Chunk 6 | 134 | docs only |

**Note:** Wave 169's audit-session handoff (`18-session-handoff.md`) cited a workspace-wide count of 553 tests at the `PHASE-6-HANDOFF.md` close-out — that number is correct for the *workspace*, not the educore-hr crate specifically. The educore-hr baseline (107) was verified by checking out `13e5fe4` in a temp worktree and running `cargo test -p educore-hr --tests`.

**New public API surface in Wave 171:**

- `validate_joining_date_not_future(joining_date: NaiveDate) -> Result<()>` — `crates/domains/hr/src/value_objects.rs`
- `validate_non_negative_f32_quota(name: &str, value: f32) -> Result<()>` — same
- `StaffStatus::can_transition_to(self, to: Self) -> bool` — same
- `Staff::suspend(...) -> Result<()>`, `Staff::reinstate(...) -> Result<()>`, `Staff::resign(...) -> Result<()>`, `Staff::terminate(...) -> Result<()>`, `Staff::retire(...) -> Result<()>` — `crates/domains/hr/src/aggregate.rs`
- `Staff::soft_delete(...)`, `Staff::set_leave_quotas(...) -> Result<()>` — same
- `StaffReinstated`, `StaffResigned`, `StaffTerminated`, `StaffRetired` events — `crates/domains/hr/src/events.rs`
- `StaffReferenceChecker` port trait — `crates/domains/hr/src/services.rs`
- `delete_staff(...) -> Result<StaffDeleted>`, `DeleteStaffCommand` — same

---

## 4. Remaining Work (in priority order for Wave 172+)

### 4.1 HR Domain (PRIMARY NEXT FOCUS)

**State after Wave 171:** 42 aggregates scaffolded. **Staff (8/8 invariants `[x]`, 27 tests)** is fully done. **41 other aggregates** have only placeholder-stub implementations.

**The 92 remaining `[ ]` invariants grouped by highest-count aggregates:**

| Aggregate | Invariants `[ ]` | Notes |
|---|---|---|
| **PayrollGenerate** | 5 (I-1, I-3, I-4, I-6 + I-2/I-5 already `[x]`) | Next highest. Status FSM + paid_amount guard + bonus/overtime handling. |
| **LeaveRequest** | 2 (I-3, I-5) | Status FSM + reject-reason required. The 3 already `[x]` invariants (I-1/I-2/I-4) were added in Wave 32. |
| **Department** | 3 | Name unique per school, tenant anchor (structural), cannot-delete-while-staff-assigned (needs port). |
| **Designation** | 3 | Same pattern as Department. |
| **LeaveType** | 3 | Name unique per school, type ∈ {paid, unpaid, partial}, tenant anchor. |
| **LeaveDefine** | 2 (I-1, I-2) | Per-school unique leave type, days_per_year > 0. I-3 already `[x]`. |
| **LeaveDeductionInfo** | 3 | deduction_amount ≥ 0, leave_days ≥ 0, per LeaveDefine. |
| **SalaryTemplate** | 4 | gross_salary == sum, net_salary == gross - deduction, name unique, append-only. |
| **StaffAttendance** | 3 | One-per-day, in_time < out_time, status FSM. |
| **StaffAttendanceImport** | 3 | batch_id valid, per-row date, idempotency. |
| **PayrollEarnDeduc** | 2 (I-1, I-3) | amount ≥ 0, sum invariants. I-2 already covered by type. |
| **Other 31 aggregates** | 1-2 each | Mostly 2-invariant placeholders. |

**Estimated work:** ~41 per-aggregate waves × ~30 min each = ~20 hours = **5-7 sessions**.

**Recommended first HR wave (Wave 172):** PayrollGenerate I-3 (status FSM) — reuses the `StaffStatus::can_transition_to` pattern from Wave 171 Chunk 4, and the FSM is the smallest entry point.

### 4.2 Finance Domain (CONTINUE)

~30 placeholder stubs remain (the `Real*` aggregates that don't exist yet). The per-aggregate wave pipeline is proven.

**Next candidates:** RealTransaction (~1-2 invariants), RealTransactionChild, RealExpense (already exists, may need expansion).

**Estimated work:** ~30 more finance waves = ~15 hours = **4-5 sessions**.

### 4.3 Spec↔checklist Reconciliation Carry-Forward

The Wave 171 Chunk 2 "Spec Reconciliation" section flagged that **the same drift pattern likely affects the other 41 HR aggregates** (Department, Designation, LeaveType, etc.). Before flipping those rows to `[x]`, audit them against `docs/specs/hr/aggregates.md` for matching wording. This is a one-time pass per aggregate, ~5 min each.

**Estimated work:** ~41 audits × ~5 min each = ~3.5 hours = **1 dedicated session**.

### 4.4 The Big Production Gaps (post-wave-pipeline)

(Same as `18-session-handoff.md` §4.4 — no change.)

| Gap | Severity | Estimated Effort |
|---|---|---|
| **Dispatcher wrapper layer (0/509)** | CRITICAL | 10+ sessions |
| **Per-invariant checklist for 9 remaining domains** | MEDIUM | 5-7 sessions |
| **Cross-adapter parity test suite** | MEDIUM | 3-5 sessions |
| **Cross-compile verification** | MEDIUM | 1-2 sessions (toolchain install + CI workflow) |
| **Threat model + operational docs** | MEDIUM | 1-2 sessions |
| **`educore-sdk::Engine::builder()`** | LOW | 1-2 sessions |
| **`educore-testkit` in-memory port impls** | LOW | 1-2 sessions |

---

## 5. Prioritized TODO (for the next session)

In priority order (do top to bottom):

1. **[15 min] Verify clean state + read this doc end-to-end.**
2. **[30 min] Spec↔checklist reconciliation pass on HR Department + Designation + LeaveType + LeaveDefine + LeaveRequest + LeaveDeductionInfo + SalaryTemplate + StaffAttendance + PayrollEarnDeduc + PayrollGenerate** (~10 aggregates). For each, audit the checklist row wording against `docs/specs/hr/aggregates.md` and document drift in a new "Spec Reconciliation (Wave 172)" section in `hr-invariant-checklist.md`.
3. **[2-3 hours] Wave 172 — HR PayrollGenerate I-3 (status FSM)** — smallest entry point, reuses the Staff FSM pattern from Wave 171 Chunk 4. Add `PayrollStatus::can_transition_to` enrichment (already has `is_paid` from Wave 32) + `PayrollGenerate::mark_paid`/`mark_partial` mutators + 1-2 events.
4. **[2-3 hours] Wave 173 — HR Department (3 invariants)** — `Department::fresh` Result-returning variant + `DepartmentReferenceChecker` port + `delete_department` service + 6-8 tests.
5. **[2-3 hours] Wave 174 — HR Designation (3 invariants)** — same pattern as Department.
6. **[continue HR waves for the remaining 38 aggregates]**
7. **[if time] Wave N — Finance: next placeholder stub** (parallel HR work).
8. **[post-wave-pipeline] Dispatcher wrapper implementation** — start with `educore-academic` (most-tested domain) as the template.

---

## 6. Blockers and Assumptions

### 6.1 Blockers

(Same as `18-session-handoff.md` §6.1 — no change.)

| Blocker | Severity | Resolution |
|---|---|---|
| Cross-compile toolchains not installed locally | MEDIUM | Install via `rustup target add aarch64-linux-android wasm32-unknown-unknown` + clang |
| `EDUCORE_PG_URL` / `EDUCORE_MYSQL_URL` env vars not set | LOW | Only needed for env-gated integration tests; default SQLite tests always run |
| `educore-storage-parity` not populated | MEDIUM | This is work to do, not a blocker |
| **Dispatcher layer not implemented (0/509)** | HIGH | Hard blocker on I-7-style cross-aggregate reference checks landing end-to-end; the Staff I-7 unit tests pass because they mock the `StaffReferenceChecker` port, but the storage-backed implementation is a follow-up. |

### 6.2 Assumptions (carry forward unless contradicted)

(Same as `18-session-handoff.md` §6.2 — no change.)

1. AI agents are first-class contributors per ADR-010.
2. `Real*` prefix on aggregates is finance convention (HR goes straight to full implementation).
3. All `#[allow(...)]` on production code should be per-function; tests can use file-level allows.
4. `cargo add <crate> --package <package-name>` is the canonical dep-add command.
5. graphify hook is installed locally — auto-rebuilds the AST-only graph on every commit.
6. Pre-commit hook may run cargo fmt + clippy.
7. `educore-events-domain` (cross-cutting tier) is the CALENDAR domain, distinct from `educore-events` (cross-cutting tier) which is the event ENVELOPE + bus port.
8. **Wave 171 added:** Spec is the source of truth for invariant wording; the checklist is a derived artifact. When they disagree, fix the checklist to match the spec, not vice versa.

---

## 7. Modified Docs/Files (Wave 171)

### 7.1 Files Created

- `docs/audit_reports/remediation/19-session-handoff.md` — this doc
- `crates/domains/hr/tests/staff.rs` — 27 new behavioral tests for the Staff aggregate (was missing entirely pre-Wave 171)

### 7.2 Files Modified

- `docs/audit_reports/hr-invariant-checklist.md` — added "Spec Reconciliation (Wave 171)" section (~40 lines) + flipped 7 Staff rows from `[ ]` to `[x]` with full file:line evidence + updated Summary tally from 8 to 15 `[x]`
- `docs/progress-tracker.md` — 12 `docs/handoff/PHASE-X-HANDOFF.md` references replaced with `docs/audit_reports/remediation/18-session-handoff.md` + updated the `educore-hr` workspace row to `Yes/Yes/Yes` with full evidence
- `crates/domains/hr/src/value_objects.rs` — added `StaffStatus::can_transition_to` FSM helper, `validate_joining_date_not_future`, `validate_non_negative_f32_quota`
- `crates/domains/hr/src/aggregate.rs` — added 7 new mutator methods on `Staff` (`suspend`, `reinstate`, `resign`, `terminate`, `retire`, `soft_delete`, `set_leave_quotas`) + extended `Staff::fresh` doc comment with spec invariants
- `crates/domains/hr/src/events.rs` — added 4 new event types (`StaffReinstated`, `StaffResigned`, `StaffTerminated`, `StaffRetired`)
- `crates/domains/hr/src/services.rs` — added `StaffReferenceChecker` port + `delete_staff` service + `DeleteStaffCommand` + reordered `StaffId` import
- `crates/domains/finance/src/value_objects.rs` — `ApprovalStatus::can_transition_to` refactor (Wave 169 leftover)
- `graphify-out/GRAPH_REPORT.md` + `graph.json` — auto-regenerated after each commit

### 7.3 Files Deleted

- `docs/handoff/PHASE-0-HANDOFF.md` through `docs/handoff/PHASE-16-HANDOFF.md` (17 files, ~7.5K LOC total) — superseded by `docs/audit_reports/remediation/17-` and `18-` docs

---

## 8. Unresolved Spec ↔ Code Conflicts

(Same as `18-session-handoff.md` §8 with Wave 171 additions.)

| Conflict | Resolution | Status |
|---|---|---|
| Phase 17 numbering (production hardening vs CMS) | Resolved as "Phase 17 = CMS (Phase 12 in AGENTS.md)" | ✅ ADR-021 exists |
| `educore-events` vs `educore-events-domain` naming | Resolved (envelope + bus port vs calendar domain) | ✅ Documented in AGENTS.md |
| `finance_aggregate_stub!` macro generates placeholders | Resolved (placeholders are documentation markers) | ✅ Documented in Wave 65 |
| `educore-storage-parity` listed at both Phase 0 + Phase 16 | Resolved | ✅ Documented |
| `educore-events-domain` listed at both Phase 2 + Phase 13 | Resolved | ✅ Documented |
| `educore-storage-parity` 0/509 wrappers vs production deployment | UNRESOLVED | Open work |
| `educore-finance` clippy 55 doc list indentations + 27 unreachable patterns | Deferred (cosmetic) | Out-of-scope |
| RBAC `required_capabilities()` 540 method-level declarations but per-domain correctness audit deferred | Deferred (v3 Part 5 R1-R10) | Open work |
| **HR Staff checklist wording vs spec wording (4 of 8 rows mismatched)** | Resolved by Wave 171 spec↔checklist reconciliation section + checklist row labels updated | ✅ This session |
| **HR other 41 aggregates' checklist wording** | Likely drifted the same way Staff did; carry-forward audit needed before flipping | 🆕 Open |
| **Dispatcher layer not wired for `StaffReferenceChecker`** | Unit tests pass with mocked port; production wiring is a post-wave follow-up | 🆕 Open |
| **Soft-delete FSM status coupling** | `Staff::soft_delete` flips `active_status` but leaves FSM `status` untouched. Some consumers may expect both to flip together. A future spec clarification is needed if this causes issues. | 🆕 Open |

---

## 9. Production Risks

(Same as `18-session-handoff.md` §9 — no change. The 0/509 dispatcher wrappers remain the #1 critical risk.)

| Risk | Severity | Mitigation |
|---|---|---|
| Deploying without dispatcher wrappers means RBAC + idempotency + outbox + audit + bus-publish are NOT enforced | **CRITICAL** | Do not deploy until 509 wrappers are implemented. The codebase is B-, not production. |
| HR domain is partially scaffolded; production schools need Staff + PayrollGenerate + LeaveRequest | **HIGH** | Staff is now done (Wave 171). PayrollGenerate + LeaveRequest are next waves. |
| Cross-compile to Android/WASM unverified | MEDIUM | Install toolchains + exercise CI job before mobile/WASM clients |
| No threat model + no pentest | MEDIUM | Engage security review before handling real PII |
| No operational runbook + no SLO/SLI | MEDIUM | Document before on-call rotation starts |
| 0/509 wrappers means every consumer hand-wires RBAC + idempotency + outbox + audit + bus-publish | **CRITICAL** | Same as first row |
| `educore-files::S3FileStorage` + `educore-payment::StripeProvider` not exercised in CI | MEDIUM | Add CI workflows with test-mode credentials |
| **StaffReferenceChecker port has no storage-backed implementation** (Wave 171) | MEDIUM | Tests pass with mocked port; follow-up session should wire the storage adapter to query `AssignClassTeacher`, `LeaveRequest`, `PayrollGenerate` tables |

---

## 10. Recommended Next Execution Order

### Session (Wave 172) — recommended next session

**Total time: 2-4 hours**

1. **[15 min] Verify clean state + read this doc end-to-end**
2. **[30 min] Spec↔checklist reconciliation pass on ~10 HR aggregates** (Department, Designation, LeaveType, LeaveDefine, LeaveRequest, LeaveDeductionInfo, SalaryTemplate, StaffAttendance, PayrollEarnDeduc, PayrollGenerate). For each, audit checklist row wording against spec, document drift in a new "Spec Reconciliation (Wave 172)" section in `hr-invariant-checklist.md`.
3. **[2-3 hours] Wave 172 — HR PayrollGenerate I-3 (status FSM)** — smallest entry point, reuses the Staff FSM pattern from Wave 171 Chunk 4. Add `PayrollGenerate::mark_paid`/`mark_partial` mutator + 1-2 new events + 4-6 tests.

### Session 2+ (Wave 173+) — continuation pattern

Continue the HR per-aggregate wave pipeline:

- Wave 173: HR Department (3 invariants)
- Wave 174: HR Designation (3 invariants)
- Wave 175: HR LeaveDefine (2 remaining invariants)
- Wave 176: HR LeaveType (3 invariants)
- Wave 177: HR LeaveRequest (2 remaining invariants — I-3 FSM, I-5 reject-reason required)
- Wave 178+: HR PayrollEarnDeduc, SalaryTemplate, StaffAttendance, etc.
- Interleave with Finance remaining placeholder stubs.

### Session 20+ (post-wave-pipeline)

Start the dispatcher wrapper implementation. Begin with `educore-academic` (the most-tested domain) — implement wrappers for all 37 service functions there (~1 session). Then move to other domains.

---

## 11. Exact Commands the Next Agent Should Run First

```bash
# 0. Verify clean state
cd /home/beznet/Workspace/smscore
git log --oneline -5
# Expected: f8e6c3e Wave 171 (Chunk 5): HR Staff I-7 + I-8 (delete_staff + leave quotas)
#           d7c8eb0 Wave 171 (Chunk 4): HR Staff I-6 (status state machine)
#           defff6d Wave 171 (Chunk 3): HR Staff I-1 + I-2 + I-3 + I-5
#           8f32faf Wave 171 (Chunk 2): document Staff spec↔checklist drift
#           13e5fe4 Wave 169 leftovers: retire phase handoffs + finance matches! refactor + graphify regen
git status
# Expected: clean (only .kimchi/ and session HTML untracked)
git branch -v
# Expected: * main ... origin/main

# 1. Verify the build is clean
cargo check -p educore-hr --tests 2>&1 | tail -3
# Expected: "Finished `dev` profile [unoptimized + debuginfo] target(s)" -- 0 errors
cargo test -p educore-hr --tests --no-fail-fast 2>&1 | tail -3
# Expected: 134+ tests passing (the Wave 171 staff.rs adds 27 to the pre-Wave 171 baseline of 107)

# 2. Read this doc's predecessor
cat docs/audit_reports/remediation/18-session-handoff.md | head -100
# The 18-session-handoff.md covers everything Wave 171 built on; read it to understand the
# audit session that preceded Wave 171.

# 3. Read the HR PayrollGenerate spec to plan Wave 172
cat docs/specs/hr/aggregates.md | sed -n '/^## PayrollGenerate/,/^## /p' | head -100
# This is the spec for what you'll build.

# 4. Read the existing PayrollGenerate aggregate
grep -n "pub struct PayrollGenerate\|impl PayrollGenerate" crates/domains/hr/src/aggregate.rs | head -5
head -200 crates/domains/hr/src/aggregate.rs
# Find the PayrollGenerate struct + impl block.

# 5. Read the value_objects.rs for HR helper functions (especially PayrollStatus)
grep -n "PayrollStatus\|fn is_paid" crates/domains/hr/src/value_objects.rs | head -10

# 6. After completing Wave 172 (PayrollGenerate I-3), run:
cargo test -p educore-hr --test payroll_generate --no-fail-fast
# Expected: all tests green (including the new ones)
cargo fmt -p educore-hr

# 7. Commit + push
git add crates/domains/hr/src/aggregate.rs \
        crates/domains/hr/src/events.rs \
        crates/domains/hr/src/services.rs \
        crates/domains/hr/src/value_objects.rs \
        crates/domains/hr/tests/payroll_generate.rs \
        docs/audit_reports/hr-invariant-checklist.md \
        graphify-out/GRAPH_REPORT.md \
        graphify-out/graph.json
git -c user.name="Educore Dev" -c user.email="dev@educore.local" commit -m "Wave 172: HR PayrollGenerate I-3 (status FSM) + spec reconciliation pass on 10 aggregates"
git push origin main
```

---

## 12. The Per-Aggregate Wave Template (proven across 103 waves)

(Same as `18-session-handoff.md` §12 — proven across 102 finance waves + 5 HR waves (Wave 171 Chunks 3-5).)

When starting a new per-aggregate wave (e.g., HR PayrollGenerate I-3), follow this template. Total time per wave: 15-45 minutes.

**Steps:**

1. Read the spec — `cat docs/specs/<domain>/aggregates.md | sed -n '/^## <Aggregate>/,/^## /p'`
2. Read the existing aggregate — `head -<N> crates/domains/<domain>/src/aggregate.rs`
3. Audit the checklist row against the spec (Wave 172+ agents must do this first)
4. Add the validation helper to `crates/domains/<domain>/src/value_objects.rs` (if not already present)
5. Update `Real<Aggregate>::fresh()` to enforce the invariant — return `DomainError::Validation` on failure
6. Update `Real<Aggregate>::update_*()` methods to re-validate the invariant (defense-in-depth)
7. Extend `Create<Aggregate>Command` in `crates/domains/<domain>/src/commands.rs` if new fields needed
8. Extend the `Created` event in `crates/domains/<domain>/src/events.rs` if new fields needed
9. Update the service function in `crates/domains/<domain>/src/services.rs` to pass new fields
10. Add re-exports to `crates/domains/<domain>/src/lib.rs::prelude` (single-shot edit)
11. Write behavioral tests in `crates/domains/<domain>/tests/<aggregate>.rs`
12. Add `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::dbg_macro, missing_docs)]` at the top of the test file
13. Run `cargo check -p <package> --tests && cargo test -p <package> --tests --no-fail-fast`
14. Flip the checklist entry in `docs/audit_reports/<domain>-invariant-checklist.md` from `[ ]` to `[x]` with full file:line evidence
15. Commit + push with the Co-Authored-By trailer

**Test fixture pattern (per AGENTS.md):**
```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::dbg_macro, missing_docs)]

use educore_core::clock::SystemIdGen;
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

---

## 13. Git Commit Attribution

(Same as `18-session-handoff.md` §13 — unchanged.)

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

(Same as `18-session-handoff.md` §14 — unchanged.)

Per AGENTS.md § Engine Rules + § Type Safety:

1. **No `unwrap()` or `expect()` in production paths** — use `?` or document the invariant
2. **No `#[allow(dead_code)]`** or `_var` prefixes to silence the compiler — delete unused code or open a follow-up issue
3. **No `as` casts** that truncate or lose data — use `TryFrom`/`TryInto`
4. **No `serde_json::Value`** in domain code — use typed wrappers
5. **No `HashMap<String, T>`** for domain data — use typed structs
6. **No service locators**, DI containers, runtime reflection
7. **No `unsafe`** in domain code (`#![forbid(unsafe_code)]`)
8. **No `native-tls`** — only `rustls`
9. **No `tokio`** directly in domain code — only through `educore-core` re-exports
10. **No glob imports** in domain code

---

## 15. See Also (canonical references)

- `AGENTS.md` — the engine operating contract (READ FIRST)
- `docs/audit_reports/remediation/18-session-handoff.md` — the Wave 169 audit session that preceded Wave 171 (read SECOND)
- `docs/audit_reports/remediation/17-final-reconciliation-audit.md` — the comprehensive 12-category gap analysis (read THIRD)
- `docs/audit_reports/remediation/15-continuation-reconciliation.md` — v3 → Wave 65+ reconciliation
- `docs/audit_reports/remediation/14-engine-production-depth-v3-roadmap.md` — v3 233-step plan
- `docs/progress-tracker.md` — per-crate implementation status (Wave 171 updated the HR row + 12 phase-handoff links)
- `docs/build-plan.md` — the 18 phases
- `docs/architecture.md` — the system map
- `docs/code-standards.md` — engineering rules
- `docs/specs/hr/aggregates.md` — the HR per-aggregate spec (source of truth for invariants)
- `docs/audit_reports/hr-invariant-checklist.md` — the HR invariant checklist (Wave 171 added the Spec Reconciliation section + flipped 7 Staff rows to `[x]`)
- `docs/audit_reports/finance-invariant-checklist.md` — finance per-invariant status
- `docs/audit_reports/academic-invariant-checklist.md` — academic per-invariant status (reference slice)
- `docs/decisions/` — 21 ADRs (ADR-021-PhaseNumberingConventions added by Wave 169)
- `graphify-out/GRAPH_REPORT.md` — engine knowledge graph (god nodes + community structure)
- `crates/domains/hr/src/aggregate.rs` — Staff aggregate + 7 new mutator methods
- `crates/domains/hr/src/events.rs` — 4 new Staff* events (Reinstated, Resigned, Terminated, Retired)
- `crates/domains/hr/src/services.rs` — `hire_staff` (Wave 32) + `delete_staff` (Wave 171) + `StaffReferenceChecker` port (Wave 171)
- `crates/domains/hr/src/value_objects.rs` — `StaffStatus::can_transition_to` (Wave 171) + `validate_joining_date_not_future` (Wave 171) + `validate_non_negative_f32_quota` (Wave 171)
- `crates/domains/hr/tests/staff.rs` — 27 new behavioral tests (Wave 171)

---

**The next agent has everything they need. Start by reading this doc + the Wave 169 handoff, then begin Wave 172 with the spec↔checklist reconciliation pass + HR PayrollGenerate I-3. Good luck.**
