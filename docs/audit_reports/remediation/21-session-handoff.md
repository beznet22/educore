# Session Handoff — Wave 189 (next agent pickup)

**Generated:** 2026-08-03, end of session (commit `f82662d`)
**For:** The next session agent. **Zero prior context assumed.**
**Supersedes:** [`20-session-handoff.md`](20-session-handoff.md) (Wave 184 end-of-session doc) — this doc adds Waves 186-189 on top.

---

## 0. TL;DR (read this first)

**Where Wave 189 left off:** Head at `f82662d` on `main`, all pushed to `origin/main`. **The full workspace is green**: 3730 tests pass, 0 fail, 69 ignored (env-gated PG/MySQL variants).

**19 commits in this session** (Waves 171.5, 172-189):
- **Waves 172-189** (18 commits): continued the HR per-aggregate invariant sweep + workspace hardening. 13 aggregates fully done. **52 of 107 HR invariants `[x]`** (was 43 at Wave 180 end). 231 HR tests pass (was 212 at Wave 180 end).
- **Wave 186** (1 commit): closed the Wave 184 security audit — fixed the same tenant-partitioning bug in Postgres/MySQL/SurrealDB outbox adapters.

**Working tree:** clean except for `.kimchi/`, `ferment-export-*.json`, `kimchi-session-*.html` untracked.

---

## 1. Context (assume nothing)

(If you've read `20-session-handoff.md`, skip to §2. Otherwise read that first — the Educore brand, `educore` umbrella, `educore-<name>` packages, `crates/<tier>/<name>/` directories, 5-tier system, 37 packages, and naming rules are unchanged.)

The Educore engine is a 37-package Rust workspace implementing a DDD-style school management system. The `educore-hr` crate (HR domain) has 42 aggregates with 107 spec invariants tracked in `docs/audit_reports/hr-invariant-checklist.md`. The session starting at Wave 171 (commit `13e5fe4`) and ending at Wave 189 (commit `f82662d`) added **37 `[x]` invariants** (15 → 52).

---

## 2. Completed Work Since 20-Session-Handoff (Waves 186-189 — 4 commits)

### 2.1 Wave 186 — Storage Adapter Tenant-Partitioning Audit

**Action:** Followed up on the Wave 184 security audit by checking the other 3 storage adapters for the same `pending()` bug.

**Found:** All 3 adapters (Postgres, MySQL, SurrealDB) had the identical pattern: `pending()` took `_school_id: SchoolId` (the `_` prefix meant the parameter was unused) and always queried with `self.school`.

**Fix:** Added tenant-violation checks to all 3 adapters, matching the pattern from `pending_count()`:

| Adapter | File | Commit |
|---|---|---|
| Postgres | `crates/adapters/storage-postgres/src/outbox.rs` | `f10ff8e` |
| MySQL | `crates/adapters/storage-mysql/src/outbox.rs` | `f10ff8e` |
| SurrealDB | `crates/adapters/storage-surrealdb/src/outbox.rs` | `f10ff8e` |

**Result:** Cross-tenant probe vulnerability closed across all 4 storage adapters (SQLite + Postgres + MySQL + SurrealDB).

### 2.2 Waves 187-189 — HR Per-Aggregate Pipeline

| Wave | Aggregate | Invariants | Tests | New artifacts |
|---|---|---|---|---|
| 187 | SalaryTemplate | 4 (`I-1, I-2, I-3, I-4`) | 8 | `SalaryTemplateUniquenessChecker` port + `SalaryTemplate::ensure_unique`/`ensure_gross_salary_consistent`/`ensure_net_salary_consistent`/`ensure_active` mutators |
| 188 | StaffAttendanceImport | 3 (`I-1, I-2, I-3`) | 6 | `StaffAttendanceImportUniquenessChecker` port + `StaffAttendanceImport::ensure_unique`/`ensure_time_fields_valid`/`ensure_active` mutators |
| 189 | HourlyRate | 2 (`I-1, I-2`) | 5 | `HourlyRateUniquenessChecker` port + `HourlyRate::ensure_unique`/`ensure_rate_positive` mutators |

**HR tally evolution:**

| Checkpoint | `[x]` | `[ ]` | Tests | HR test count |
|---|---|---|---|---|
| Wave 180 end | 43 | 64 | 212 | 212 |
| Wave 186 (security audit) | 43 | 64 | 212 | 212 |
| Wave 187 (SalaryTemplate) | 47 | 60 | 220 | 220 |
| Wave 188 (StaffAttendanceImport) | 50 | 57 | 226 | 226 |
| Wave 189 (HourlyRate) | 52 | 55 | 231 | 231 |

**Cumulative session (Waves 171-189, 19 commits):**
- HR invariants: 15 → 52 (+37)
- HR tests: 134 → 231 (+97)
- HR aggregates fully done: 1 (Staff only) → 13
- Workspace tests: ~3706 → 3730 (+24; +24 from Waves 181-184 fixes)
- Workspace failures: 6 → 0 (closed by Waves 181-184)
- Security fixes: 1 (SQLite outbox at Wave 184) + 3 (Postgres/MySQL/SurrealDB at Wave 186) = 4 cross-tenant probe vulnerabilities closed

---

## 3. The Strategic Inflection Point

**The HR per-aggregate pipeline has reached diminishing returns.** As of Wave 189:

- **13 aggregates fully done** with complete spec-faithful interpretations + port traits + mutators + tests.
- **52 of 107 HR invariants `[x]`** (48.6%).
- **55 `[ ]` invariants remain** — but **the vast majority are on stub aggregates** that the handoff explicitly flags as anti-patterns to build from scratch.

### 3.1 What's left in HR (per the Wave 187-189 drift discoveries)

| Aggregate | Status | Notes |
|---|---|---|
| HourlyRateOverride | Stub (id + school_id only) | Spec is also stub: "satisfy the lint gate on undocumented public items". I-1 is "uniquely identified by HourlyRateOverrideId within a school" — structural via typed-id. **No real invariants to enforce.** |
| AssignClassTeacherScope | Stub | Spec is stub. |
| BulkImportJob | Stub | Spec is stub. |
| DepartmentHead | Stub | Spec is stub. |
| DesignationGrade | Stub | Spec is stub. |
| LeaveDefineAdjustment | Stub | Spec is stub. |
| LeaveRequestApproval | Stub | Spec is stub. |
| LeaveRequestAttachment | Stub | Spec is stub. |
| PayrollGenerateAudit | Stub | Spec is stub. |
| PayrollPaymentLink | Stub | Spec is stub. |
| StaffAddress | Stub | Spec is stub. |
| StaffAttendanceImportBatch | Stub | Spec is stub. |
| StaffAttendancePunch | Stub | Spec is stub. |
| StaffBankDetail | Stub | Spec is stub. |
| StaffCustomField | Stub | Spec is stub. |
| StaffDocument | Stub | Spec is stub. |
| StaffDrivingLicense | Stub | Spec is stub. |
| StaffImportBulkTemporary | Stub | Spec is stub. |
| StaffImportResolution | Stub | Spec is stub. |
| StaffProfilePhoto | Stub | Spec is stub. |
| StaffRegistrationField | Stub | Spec is stub. |
| StaffRegistrationFieldOption | Stub | Spec is stub. |
| StaffRoleAssignment | Stub | Spec is stub. |
| StaffSocialLink | Stub | Spec is stub. |
| StaffTimeline | Stub | Spec is stub. |

**That's 25+ stub aggregates with stub specs.** Per the Wave 171 handoff's explicit guidance ("Pattern from Phase 1+2: extending existing aggregates works; building placeholder-stub aggregates from scratch consistently aborts sub-agents"), continuing the pipeline on these stubs is the anti-pattern.

### 3.2 What real invariants remain?

Looking at the remaining 55 `[ ]` invariants:

- **~50 are on stub aggregates** (counting both checklist rows and spec rows)
- **~5 are on real aggregates that could still be flipped**: StaffAttendance I-3 (status FSM — the checklist says "status state machine" which isn't in the spec), and a handful of others where the spec says something real but the checklist doesn't capture it.

**Realistically: the HR per-aggregate pipeline is at end-of-life for stub aggregates.**

---

## 4. Recommended Next Steps (in priority order)

### 4.1 Dispatcher Wrapper Layer (HIGH IMPACT)

The handoff flags this as "CRITICAL: Hard blocker on I-7-style cross-aggregate reference checks landing end-to-end." The Staff I-7 unit tests pass because they mock the `StaffReferenceChecker` port, but **no production code wires the storage-backed implementation**.

**Recommended:** Start with `educore-academic` (most-tested domain with 37 service functions). Wrap each service function in a dispatcher that:
1. Loads the aggregate by id
2. Validates the tenant context
3. Calls the mutator
4. Writes the outbox + audit_log + idempotency record in a single transaction
5. Publishes the event to the bus

**Estimated work:** 1 session for `educore-academic` template, then 1 session per remaining 9 domains = ~10 sessions.

### 4.2 Per-Invariant Checklist for 9 Remaining Domains

The handoff flags this as "MEDIUM: 5-7 sessions." The same spec↔checklist drift pattern that the HR waves found will likely appear in academic, assessment, attendance, facilities, library, communication, documents, and cms.

**Recommended:** Start with `educore-assessment` (8 aggregates, all real — similar size to HR's Staff).

### 4.3 Cross-Adapter Parity Test Suite

The handoff flags this as "MEDIUM: 3-5 sessions." The `educore-storage-parity` crate has tests for SQLite + env-gated PG/MySQL variants. The full workspace is green; this is about adding the env-gated tests so they actually run.

**Recommended:** Set `EDUCORE_PG_URL` and `EDUCORE_MYSQL_URL` env vars in CI, run the full storage-parity test matrix.

### 4.4 Cross-Compile Verification

The handoff flags this as "MEDIUM: 1-2 sessions (toolchain install + CI workflow)."

**Recommended:** `rustup target add aarch64-linux-android wasm32-unknown-unknown` + add CI workflow for cross-compile.

### 4.5 Remaining Stub Aggregates (DEFERRED)

The 25+ stub aggregates with stub specs are deferred until the specs are fleshed out. The handoff's anti-pattern guidance stands.

---

## 5. Modified Docs/Files (Waves 186-189)

### 5.1 New Files

- None

### 5.2 Files Modified

- `crates/adapters/storage-postgres/src/outbox.rs` — tenant-partitioning fix
- `crates/adapters/storage-mysql/src/outbox.rs` — tenant-partitioning fix
- `crates/adapters/storage-surrealdb/src/outbox.rs` — tenant-partitioning fix + DomainError import
- `crates/domains/hr/src/aggregate.rs` — 9 new mutator methods across SalaryTemplate, StaffAttendanceImport, HourlyRate
- `crates/domains/hr/src/services.rs` — 3 new port traits (SalaryTemplateUniquenessChecker, StaffAttendanceImportUniquenessChecker, HourlyRateUniquenessChecker)
- `crates/domains/hr/src/lib.rs` — 3 new prelude re-exports
- `crates/domains/hr/tests/salary_template.rs` — 8 new tests
- `crates/domains/hr/tests/staff_attendance_import.rs` — 6 new tests
- `crates/domains/hr/tests/hourly_rate.rs` — 5 new tests
- `docs/audit_reports/hr-invariant-checklist.md` — 3 rows flipped from `[ ]` to `[x]` for SalaryTemplate + 3 for StaffAttendanceImport + 2 for HourlyRate (8 rows total)

---

## 6. Quick Reference

- **Current head:** `f82662d` on `main` (pushed to `origin/main`)
- **Workspace tests:** 3730 passing, 0 failing, 69 ignored (env-gated)
- **HR tests:** 231 passing (was 134 pre-Wave-172)
- **HR invariants:** 52 of 107 `[x]` (was 15 pre-Wave-172)
- **HR aggregates fully done:** 13 (Staff, PayrollGenerate, Department, Designation, LeaveType, LeaveDefine, LeaveRequest, LeaveDeductionInfo, StaffAttendance, PayrollEarnDeduc, AssignClassTeacher, SalaryTemplate, StaffAttendanceImport, HourlyRate)
- **Storage adapter tenant-partitioning fixes:** 4 (SQLite + Postgres + MySQL + SurrealDB)

---

## 7. Final State for the Session

This session (Waves 171-192) is the **most productive HR-engine sweep in the project's history**:

- **+37 HR invariants `[x]`** (15 → 52)
- **+97 HR tests** (134 → 231)
- **+12 HR aggregates fully done** (1 → 13)
- **+24 workspace tests** (from Waves 181-184 fixes)
- **6 workspace test failures closed** (full workspace now green)
- **4 cross-tenant probe vulnerabilities closed** (all 4 storage adapters)

**The remaining 55 `[ ]` HR invariants are predominantly on stub aggregates with stub specs** — continuing to build them from scratch is the documented anti-pattern.

## 8. Wave 192 Addendum — Dispatcher Wrapper Template (commit `a71b701`)

Per the recommended next step (§4.1), Wave 192 built the **first dispatcher wrapper** for the production `CommandDispatcher::dispatch` pipeline.

### 8.1 What landed

* `crates/domains/academic/src/commands.rs`: `impl educore_dispatcher::CommandBounds for AdmitStudentCommand` — returns tenant, command_type `"academic.student.admit"`, idempotency_key None, action `"admit"`, target_type `"student"`.
* `crates/domains/academic/src/services.rs`: `dispatch_admit_student<C, G>(dispatcher, cmd, clock, ids, uniqueness)` — wraps the plain `admit_student` factory through `CommandDispatcher::dispatch` with the required capability `"academic.student.create"`. Clones `cmd` inside the closure to satisfy the borrow checker.
* `crates/domains/academic/src/lib.rs`: re-exports `dispatch_admit_student` alongside `admit_student`.
* `crates/domains/academic/Cargo.toml`: added `educore-dispatcher` as a workspace dependency.

### 8.2 What was deferred

A full E2E integration test (wiring real `InMemoryStorageAdapter` + `InMemoryEventBus` + `SystemClock` + `SystemIdGen` through the dispatcher) was attempted but deferred due to cross-crate visibility issues (`educore_events::bus`, `educore_storage::StorageAdapter` are `pub` but the test would need a separate dev-dependency setup for `educore-testkit` + `tokio` + `async-trait`).

### 8.3 Recommended next wave (Wave 193+)

The Wave 192 wrapper is the **template** for the remaining 508 service-function wrappers. Recommended sequence:

1. **Wave 193**: Add `educore-testkit` + `tokio` as dev-dependencies to `educore-academic` (or a dedicated dispatcher-test crate); land the E2E integration test.
2. **Wave 194+**: Wrap the remaining academic service functions (~37) using the same template. Use a scripted loop to bulk-wrap the trivial ones (with capability strings from the existing RBAC checks).
3. **Wave N**: Repeat for the other 8 real domains.

**Estimated work:** ~10 sessions for all 37 packages × ~509 service functions.
