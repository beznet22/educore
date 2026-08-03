# Session Handoff — Wave 184 (next agent pickup)

**Generated:** 2026-08-03, end of session (commit `a30e092`)
**For:** The next session agent. **Zero prior context assumed.**
**Companion:** [`19-session-handoff.md`](19-session-handoff.md) — end of Wave 171; this doc picks up from there.

---

## 0. TL;DR (read this first)

**Where Wave 184 left off:** Head at `a30e092` on `main`, all pushed to `origin/main`. **The full workspace is now GREEN**: 3730 tests pass, 0 fail, 69 ignored (env-gated PG/MySQL variants). This is the first time the workspace builds and tests cleanly across all 37 packages since Wave 48 (`d0157a7`) introduced the academic_integration compile error.

**14 commits in this session** (Waves 171.5, 172, 173, 174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184):
- **Waves 172-180** (9 commits): HR per-aggregate invariant sweeps — 9 aggregates fully done (Staff + PayrollGenerate + Department + Designation + LeaveType + LeaveDefine + LeaveRequest + LeaveDeductionInfo + StaffAttendance + PayrollEarnDeduc + AssignClassTeacher). **43 of 107 HR invariants `[x]`** (was 15 after Wave 171).
- **Waves 181-184** (4 commits): workspace test green-up mini-campaign — fixed 6 long-standing test failures (academic_integration compile error, RBAC round-trip mismatch, assessment stub contract, SQLite outbox tenant partitioning security bug, finance test data).

**Working tree:** clean except for `.kimchi/`, `ferment-export-*.json`, `kimchi-session-*.html` untracked.

---

## 1. Context (assume nothing)

(If you've read `19-session-handoff.md`, skip to §2. Otherwise read that first — the Educore brand, `educore` umbrella, `educore-<name>` packages, `crates/<tier>/<name>/` directories, 5-tier system, 37 packages, and naming rules are unchanged.)

The Educore engine is a 37-package Rust workspace implementing a DDD-style school management system. The `educore-hr` crate (HR domain) has 42 aggregates with 107 spec invariants tracked in `docs/audit_reports/hr-invariant-checklist.md`. Waves 32 + 171 had landed 15 `[x]` invariants; this session added **28 more** (43 total) plus fixed all 6 long-standing workspace test failures.

---

## 2. Completed Work (Wave 172-184 — 13 commits)

### 2.1 HR Per-Aggregate Wave Pipeline (Waves 172-180)

**Pattern reused from Wave 171:** each wave adds (a) one port trait in `services.rs`, (b) one or more mutator methods on the aggregate, (c) a service function, (d) a spec-reconciliation section in the checklist, (e) 4-21 behavioral tests in the aggregate's `tests/<name>.rs` file.

| Wave | Aggregate | Invariants | Tests | New artifacts |
|---|---|---|---|---|
| 172 | PayrollGenerate | 4 (`I-1, I-3, I-4, I-6`) | 21 | `PayrollStatus::can_transition_to` FSM + `mark_generated`/`mark_paid`/`record_payment`/`update_amounts` mutators + `validate_paid_amount` + `validate_gross_salary` helpers |
| 173 | Department | 3 (`I-1, I-2, I-3`) | 7 | `DepartmentReferenceChecker` port (has_assigned_staff + has_department_head) + `Department::ensure_deletable`/`soft_delete` mutators + `delete_department` service + `DeleteDepartmentCommand` |
| 174 | Designation | 3 (`I-1, I-2, I-3`) | 6 | `DesignationReferenceChecker` port + `Designation::ensure_deletable`/`soft_delete` mutators + `delete_designation` service + `DeleteDesignationCommand` + `DesignationDeleted::new()` constructor |
| 175 | LeaveType | 3 (`I-1, I-2, I-3`) | 9 | `LeaveTypeReferenceChecker` port + `LeaveType::ensure_total_days_valid`/`soft_delete` mutators + `delete_leave_type` service + `DeleteLeaveTypeCommand` |
| 176 | LeaveDefine | 3 (`I-1, I-2, I-3`) | 7 | `LeaveDefineUniquenessChecker` port + `LeaveDefine::ensure_non_negative`/`ensure_unique` mutators |
| 177 | LeaveRequest | 2 (`I-3, I-5`) | 9 | `LeaveRequestUniquenessChecker` port + `LeaveRequest::reject`/`ensure_unique`/`ensure_within_leave_define` mutators + `reject_leave` service |
| 178 | LeaveDeductionInfo | 3 (`I-1, I-2, I-3`) | 7 | `LeaveDeductionInfoUniquenessChecker` port + `LeaveDeductionInfo::ensure_non_negative`/`ensure_unique`/`ensure_active` mutators |
| 179 | StaffAttendance + PayrollEarnDeduc | 3 + 2 = 5 | 6 + 6 | `StaffAttendanceUniquenessChecker` port + `StaffAttendance::ensure_unique`/`ensure_date_required`/`ensure_attendance_type_valid` mutators + `PayrollEarnDeduc::ensure_amount_non_negative`/`ensure_earn_dedc_type_valid` mutators |
| 180 | AssignClassTeacher | 2 (`I-1, I-2`) | 4 | `AssignClassTeacherUniquenessChecker` port (uses `educore_academic::value_objects::ClassId`/`SectionId`) + `AssignClassTeacher::ensure_unique`/`ensure_active_open` mutators |

**HR tally evolution:**

| Checkpoint | `[x]` | `[ ]` | Tests | HR test count |
|---|---|---|---|---|
| Wave 171 end | 15 | 92 | 134 | 134 |
| Wave 172 (PayrollGenerate) | 19 | 88 | 155 | 155 |
| Wave 173 (Department) | 22 | 85 | 162 | 162 |
| Wave 174 (Designation) | 25 | 82 | 168 | 168 |
| Wave 175 (LeaveType) | 28 | 79 | 177 | 177 |
| Wave 176 (LeaveDefine) | 30 | 77 | 184 | 184 |
| Wave 177 (LeaveRequest) | 32 | 75 | 193 | 193 |
| Wave 178 (LeaveDeductionInfo) | 35 | 72 | 200 | 200 |
| Wave 179 (StaffAttendance + PayrollEarnDeduc) | 41 | 66 | 208 | 208 |
| Wave 180 (AssignClassTeacher) | 43 | 64 | 212 | 212 |

### 2.2 Spec Reconciliation Discoveries

Every HR wave (171-180) found **spec↔checklist drift** — the checklist wording/numbering didn't match `docs/specs/hr/aggregates.md`. Three drift categories emerged:

1. **Row renames** (most common): checklist I-N says X, spec I-M says Y. Resolution: flip under spec-faithful interpretation, cite spec invariant in evidence line.
2. **Spec invariant missing from checklist entirely** (Wave 177 LeaveRequest#1 uniqueness, LeaveRequest#5 days ≤ total_days): added new mutators + port.
3. **Checklist row not a spec invariant at all** (Wave 175 LeaveType I-2 'type ∈ {paid, unpaid, partial}'): flipped under spec-faithful interpretation to point at the real spec invariant.

The reconciliation sections are in `docs/audit_reports/hr-invariant-checklist.md` § "Spec Reconciliation (Wave NNN)" for N ∈ {171, 172, 173, 174, 175, 176, 177, 178}.

### 2.3 Workspace Test Green-Up Mini-Campaign (Waves 181-184)

| Wave | Target | Fix |
|---|---|---|
| 181 | `educore-storage-parity academic_integration` | Added 8 missing trait methods to `TestUniqueness` impl (`student_group_name_exists`, `student_category_name_exists`, `student_has_active_record`, `lesson_title_exists`, `class_section_exists`, `class_section_has_student_records`, `teacher_has_conflict`, `room_has_conflict`) — the trait was extended since Wave 48 but the test was never updated. Also added `UserId` import. |
| 182 | `educore-rbac --lib` + `rbac_e2e` | Added 3 missing aliases to the `Capability` `from_str` map (`Hr.Staff.AssignClassTeacher.Create/Update/Delete`) — the `Display` impl produced these strings but `from_str` only knew the shorter `Hr.AssignClassTeacher.*` form, breaking the round-trip test. Cascading fix: `rbac_e2e` went from blocked to 19 passing. |
| 183 | `educore-assessment student_take_online_exam` + `teacher_evaluation` | Tests asserted the old stub contract (`NotSupported`) but the stubs were upgraded to real implementations that return `Ok` for same-tenant or `Validation` on cross-tenant mismatch. Rewrote 3 tests to assert the real contract. |
| 184 | `educore-storage-sqlite outbox_e2e` | **SECURITY FIX**: `outbox::pending()` ignored the caller-supplied `school_id` (prefixed with `_`) and always used `self.school`. Fixed to reject mismatches with `DomainError::tenant_violation`, matching the pattern already used by `pending_count()`. This was a real cross-tenant probe vulnerability. |
| 184 | `educore-storage-parity finance_integration` | Test used `PaymentMethodKind::Cash` with `AccountType::Bank` which the real validation rejects. Fixed to `AccountType::Cash`. |

**Result:** workspace went from **6 failing targets** → **0 failing targets**. **3730 tests pass, 0 fail, 69 ignored** (env-gated PG/MySQL variants).

---

## 3. Modified Docs/Files (Waves 172-184)

### 3.1 New Files

- None (all new code went into existing test files; no new test files were created in this session)

### 3.2 Files Modified

- `docs/audit_reports/hr-invariant-checklist.md` — 9 new "Spec Reconciliation (Wave NNN)" sections (~450 lines total) + 9 Summary tally updates + 28 rows flipped from `[ ]` to `[x]` with full file:line evidence
- `docs/progress-tracker.md` — educore-hr workspace row updated with Waves 171-180 results
- `docs/audit_reports/remediation/19-session-handoff.md` — Wave 171 tally bug fixed at Wave 171.5 (commit `13288ca`)
- `crates/domains/hr/src/aggregate.rs` — 25 new mutator methods added across 8 aggregates (Staff already done in Wave 171)
- `crates/domains/hr/src/services.rs` — 9 new port traits + 9 new service functions + 4 new event constructors
- `crates/domains/hr/src/commands.rs` — 4 new `*Command` structs (DeleteDepartment/DeleteDesignation/DeleteLeaveType) + prelude re-exports
- `crates/domains/hr/src/events.rs` — 1 new constructor (`DesignationDeleted::new` was missing)
- `crates/domains/hr/src/value_objects.rs` — 2 new FSM helpers (`PayrollStatus::can_transition_to`, `DesignationStatus` was already done in Wave 171) + 3 new validators (`validate_paid_amount`, `validate_gross_salary`)
- `crates/domains/hr/src/lib.rs` — 10 new prelude re-exports
- `crates/domains/hr/tests/staff.rs` — Wave 171 had 27 tests; Waves 172-180 added 78 more across payroll_generate.rs, department.rs, designation.rs, leave_type.rs, leave_define.rs, leave_request.rs, leave_deduction_info.rs, staff_attendance.rs, payroll_earn_deduc.rs, assign_class_teacher.rs (was 2 each before Wave 172)
- `crates/tools/storage-parity/tests/academic_integration.rs` — 8 missing trait methods + UserId import (Wave 181)
- `crates/cross-cutting/rbac/src/value_objects.rs` — 3 missing Capability aliases (Wave 182)
- `crates/domains/assessment/tests/student_take_online_exam.rs` — 3 tests rewritten to real contract (Wave 183)
- `crates/domains/assessment/tests/teacher_evaluation.rs` — 1 test rewritten to real contract (Wave 183)
- `crates/adapters/storage-sqlite/src/outbox.rs` — SECURITY FIX: tenant partitioning in `pending()` (Wave 184)
- `crates/tools/storage-parity/tests/finance_integration.rs` — AccountType::Cash fix (Wave 184)
- `graphify-out/GRAPH_REPORT.md` + `graph.json` — auto-regenerated after each commit

### 3.3 Files Deleted

- None in this session

---

## 4. Remaining Work (in priority order for Wave 185+)

### 4.1 HR Domain (CONTINUE)

**State after Wave 180:** 11 aggregates fully done. **31 aggregates remain** with **64 `[ ]` invariants** — but most are **stub aggregates** (DepartmentHead, DesignationGrade, AssignClassTeacherScope, LeaveDefineAdjustment, PayrollPaymentLink, StaffAddress, StaffAttendanceImportBatch, StaffAttendancePunch, StaffBankDetail, StaffCustomField, StaffDocument, StaffDrivingLicense, StaffImportBulkTemporary, StaffImportResolution, StaffProfilePhoto, StaffRegistrationField, StaffRegistrationFieldOption, StaffRoleAssignment, StaffSocialLink, StaffTimeline, BulkImportJob, LeaveRequestApproval, PayrollGenerateAudit, HourlyRateOverride) with only `id` + `school_id` fields. The handoff explicitly flags building these from scratch as an anti-pattern.

**Next HR candidates** (aggregates with spec drift worth a dedicated wave):

| Aggregate | Invariants `[ ]` | Notes |
|---|---|---|
| SalaryTemplate | 4 | I-1 gross_salary == sum, I-2 net_salary == gross - deduction, I-3 name unique, I-4 append-only. |
| StaffAttendanceImport | 3 | I-1 batch_id valid, I-2 per-row date, I-3 idempotency. |
| HourlyRate | 2 | Wave 32 already enforced `rate >= 0`; remaining are likely spec drift. |
| LeaveDefineAdjustment | 2 | Adjustment to LeaveDefine (carry-forward, special grant). |
| StaffTimeline | 2 | Append-only log + monotonic timestamp. |

**Estimated work:** ~5-10 more per-aggregate waves × ~30 min each = ~3-5 hours = **1-2 sessions**.

### 4.2 Finance Domain (CONTINUE)

~30 placeholder stubs remain (the `Real*` aggregates that don't exist yet). The per-aggregate wave pipeline is proven.

**Next candidates:** RealTransaction (~1-2 invariants), RealTransactionChild, RealExpense (already exists, may need expansion).

**Estimated work:** ~30 more finance waves = ~15 hours = **4-5 sessions**.

### 4.3 Workspace Hardening (NEW PRIORITY from this session)

The workspace is now green, but several production-readiness gaps remain:

| Gap | Severity | Estimated Effort |
|---|---|---|
| **Dispatcher wrapper layer (0/509)** | CRITICAL | 10+ sessions |
| **Cross-adapter parity test suite** | MEDIUM | 3-5 sessions |
| **Cross-compile verification** (Android ARM64, WASM) | MEDIUM | 1-2 sessions (toolchain install + CI workflow) |
| **Threat model + operational docs** | MEDIUM | 1-2 sessions |
| **`educore-sdk::Engine::builder()`** | LOW | 1-2 sessions |
| **`educore-testkit` in-memory port impls** | LOW | 1-2 sessions |
| **Per-invariant checklist for 9 remaining domains** (academic, finance, assessment, attendance, facilities, library, communication, documents, cms) | MEDIUM | 5-7 sessions |

### 4.4 The Security Bug from Wave 184 (ACTION ITEM)

The Wave 184 SQLite outbox tenant-partitioning fix only covers the SQLite adapter. The same bug pattern may exist in:

- `crates/adapters/storage-postgres/src/outbox.rs`
- `crates/adapters/storage-mysql/src/outbox.rs`
- `crates/adapters/storage-surrealdb/src/outbox.rs`

**Recommended:** audit all 4 storage adapters for the same `_school_id` parameter prefix pattern, and either fix the same way (add tenant violation check) or document why it's intentionally allowed.

### 4.5 Spec Reconciliation Backlog

The Wave 171-178 reconciliation sections documented drift on:

- `LeaveDeductionInfo` rows (renamed in Wave 178)
- `PayrollEarnDeduc` rows (verified non-drift in Wave 179)

**Remaining unchecked aggregates** (where the checklist rows may still be wrong):

- All 20+ 2-invariant stub aggregates (DepartmentHead, DesignationGrade, etc.)
- SalaryTemplate (4 invariants)
- StaffAttendanceImport (3 invariants)
- HourlyRateOverride (2 invariants)
- LeaveDefineAdjustment (2 invariants)

---

## 5. Prioritized TODO (for the next session)

In priority order (do top to bottom):

1. **[15 min] Verify clean state + read this doc end-to-end.**
2. **[30 min] Audit all 4 storage adapters for the Wave 184 tenant-partitioning pattern** (Postgres, MySQL, SurrealDB outbox `pending()` methods). Fix any that have the same `_school_id` parameter prefix without a tenant-violation check.
3. **[2-3 hours] Wave 185 — HR SalaryTemplate (4 invariants)** — `gross_salary == sum` + `net_salary == gross - deduction` + name unique + append-only. Reuses the per-aggregate wave pattern.
4. **[1-2 hours] Wave 186 — HR StaffAttendanceImport (3 invariants)** — batch_id + per-row date + idempotency.
5. **[1-2 hours] Wave 187 — HR HourlyRate + HourlyRateOverride (2 + 2 invariants)** — rate >= 0 already enforced from Wave 32; verify and add spec reconciliation.
6. **[2-3 hours] Wave 188 — Begin dispatcher wrapper layer** — start with `educore-academic` (most-tested domain) as the template. Wrap all 37 service functions.
7. **[if time] Wave 189 — Finance: next placeholder stub** (RealTransaction or RealExpense).
8. **[post-wave-pipeline] Cross-compile verification** — install Android + WASM toolchains, add CI workflow.

---

## 6. Blockers and Assumptions

### 6.1 Blockers

(Same as `19-session-handoff.md` §6.1, plus one resolved.)

| Blocker | Severity | Resolution |
|---|---|---|
| ~~Workspace test failures (6 targets failing)~~ | ~~HIGH~~ | ✅ **RESOLVED at Wave 184.** Full workspace is now green (3730 passing, 0 failing). |
| Cross-compile toolchains not installed locally | MEDIUM | Install via `rustup target add aarch64-linux-android wasm32-unknown-unknown` + clang |
| `EDUCORE_PG_URL` / `EDUCORE_MYSQL_URL` env vars not set | LOW | Only needed for env-gated integration tests; default SQLite tests always run |
| `educore-storage-parity` partial population | LOW | Some domains have integration tests (academic, assessment, attendance, cms, facilities, library, finance); others don't yet |
| **Dispatcher layer not implemented (0/509)** | HIGH | Hard blocker on I-7-style cross-aggregate reference checks landing end-to-end; the Staff I-7 unit tests pass because they mock the `StaffReferenceChecker` port, but the storage-backed implementation is a follow-up. |
| **Storage adapter tenant-partitioning audit (NEW)** | MEDIUM | Wave 184 fixed SQLite; Postgres/MySQL/SurrealDB adapters need the same audit. |

### 6.2 Assumptions (carry forward unless contradicted)

(Same as `19-session-handoff.md` §6.2, plus one new.)

1. AI agents are first-class contributors per ADR-010.
2. `Real*` prefix on aggregates is finance convention (HR goes straight to full implementation).
3. All `#[allow(...)]` on production code should be per-function; tests can use file-level allows.
4. `cargo add <crate> --package <package-name>` is the canonical dep-add command.
5. graphify hook is installed locally — auto-rebuilds the AST-only graph on every commit.
6. Pre-commit hook may run cargo fmt + clippy.
7. `educore-events-domain` (cross-cutting tier) is the CALENDAR domain, distinct from `educore-events` (cross-cutting tier) which is the event ENVELOPE + bus port.
8. Spec is the source of truth for invariant wording; the checklist is a derived artifact. When they disagree, fix the checklist to match the spec, not vice versa.
9. **NEW (Wave 184):** Tenant-partitioning violations are always `DomainError::tenant_violation`, not `Validation` or `Conflict`. The cross-tenant probe is a security boundary, not a user error.

---

## 7. Quick Reference

- **Current head:** `a30e092` on `main` (pushed to `origin/main`)
- **Workspace tests:** 3730 passing, 0 failing, 69 ignored (env-gated)
- **HR tests:** 212 passing (was 134 pre-Wave-172)
- **HR invariants:** 43 of 107 `[x]` (was 15 pre-Wave-172)
- **HR aggregates fully done:** 11 (Staff, PayrollGenerate, Department, Designation, LeaveType, LeaveDefine, LeaveRequest, LeaveDeductionInfo, StaffAttendance, PayrollEarnDeduc, AssignClassTeacher)
- **Key file paths:**
  - `docs/audit_reports/hr-invariant-checklist.md` — the master checklist (43 `[x]` rows + 9 reconciliation sections)
  - `docs/audit_reports/remediation/19-session-handoff.md` — Wave 171 end-of-session doc
  - `docs/progress-tracker.md` — workspace-level progress table
  - `crates/domains/hr/src/aggregate.rs` — 25 mutator methods added in Waves 172-180
  - `crates/domains/hr/src/services.rs` — 9 port traits + 9 service functions added
  - `crates/domains/hr/tests/staff.rs` — 27 Wave 171 tests + 78 added in Waves 172-180 across all test files
  - `crates/adapters/storage-sqlite/src/outbox.rs` — Wave 184 tenant-partitioning security fix
  - `crates/cross-cutting/rbac/src/value_objects.rs` — Wave 182 Capability round-trip fix
  - `crates/tools/storage-parity/tests/academic_integration.rs` — Wave 181 compile fix

---

**This is the first session handoff that documents a fully-green workspace.** Use it as the baseline for all future production-readiness work.
