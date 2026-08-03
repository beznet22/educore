# Stub & Legacy Remediation Report

**Generated:** Wave 198
**Scope:** All 37 crates (`crates/domains/`, `crates/cross-cutting/`, `crates/adapters/`, `crates/tools/`)
**Author:** Antigravity (automated audit)

## Executive Summary

The repository contains significant stub, placeholder, and legacy code that must be
progressively eliminated to reach production-readiness. This report enumerates all
findings, groups them by category, and recommends an implementation order.

| Category | Count | Severity |
|----------|------:|----------|
| Stub aggregates (`id` + `school_id` only) | **133** | High |
| Stub services returning `DomainError::NotSupported("TODO: ...")` | **17** | High |
| Stub test placeholders | **6** | Medium |
| Legacy `Fm*` (Fees Module) prefix references | **1180** | High |
| Spec files with duplicate aggregate headers | **37** (across 2 files) | Medium |
| TODO comments (excluding lint infrastructure) | **~20** | Low |
| `todo!()` / `unimplemented!()` calls in production | **0** | ✅ Clean |
| FIXME / HACK comments | **0** | ✅ Clean |

**Total stubs found:** ~133 + 17 + 6 = **~156**
**Legacy prefix hits:** **1180**
**Spec duplicates:** **37**
**Total remediation scope:** **~1373 items**

## 1. Stub Aggregates (133 total)

A "stub aggregate" is a `pub struct` whose only `pub` fields are `id: XId` and
`school_id: SchoolId` — i.e. the type exists for lint compliance but has no
real behavior.

### Per-crate distribution

| Crate | Stub count | Real-spec aggregates | Spec stubs |
|-------|-----------:|---------------------:|-----------:|
| `educore-finance` | 57 | 0 | 73 (62 unique) |
| `educore-assessment` | 38 | 15 (already in Wave 196 plan) | 44 (29 stubs) |
| `educore-hr` | 26 | 14 (already done in Waves 171-189) | 68 (42 unique) |
| `educore-academic` | 10 | ? | ? |
| `educore-attendance` | 2 | ? | ? |
| `educore-cms` | 0 (42 real, all done in Wave 12) | 0 (per Wave 12 handoff) | ? |

### Why this matters

- These structs compile, pass lint, and satisfy `educore-core::lint`, but
  they cannot model any real business behavior.
- They cannot enforce invariants, emit events, or persist via storage adapters.
- They are dead code in production — they add maintenance burden without
  providing value.

## 2. NotSupported Stubs (17 total)

These are service functions that return `DomainError::NotSupported("TODO: X")`
instead of doing the work.

### Locations

```
crates/domains/assessment/tests/student_take_online_exam.rs:86
crates/domains/assessment/tests/student_take_online_exam.rs:143
crates/domains/assessment/tests/teacher_evaluation.rs:134
crates/domains/assessment/src/services.rs:1180 (comment: "These are TODO stubs")
crates/domains/attendance/src/aggregate.rs:734 (ClassAttendance::verify_invariants)
crates/domains/attendance/src/aggregate.rs:838 (AttendanceBulk::promote_to_student_attendance)
crates/domains/documents/src/aggregate.rs:694 (TODO local alias replacement)
crates/educore/src/subscribers.rs (6 TODO(SDK) markers — subscribers not wired to dispatcher)
```

## 3. Stub Test Placeholders (6 total)

Located in `crates/domains/assessment/tests/student_take_online_exam.rs` and
`teacher_evaluation.rs` — tests that pin `NotSupported` behavior rather than
asserting real behavior. These should be rewritten when the underlying service
functions are implemented.

## 4. Legacy `Fm*` Prefix (1180 hits)

The `Fm` prefix (Fees Module) is a legacy prefix from the Schoolify/InfixEdu
project. Per AGENTS.md § "Project Identity":

> The brand is **Educore**; the package namespace is **`educore`**; internal
> crates publish under **`educore-<name>`**. Use these forms everywhere.
> **No legacy names are permitted in new code, comments, commit messages,
> or documentation.**

The `Fm*` prefix is a legacy name that violates this rule. It is used in
`educore-finance` for an entire sub-module of finance aggregates
(`FmFeesGroup`, `FmFeesType`, `FmFeesInvoice`, `FmFeesInvoiceChild`,
`FmFeesTransaction`, `FmFeesTransactionChild`, `FmFeesWeaver`,
`FmFeesInvoiceSetting`).

### Affected types (sample)

`FmFeesGroup`, `FmFeesType`, `FmFeesInvoice`, `FmFeesInvoiceChild`,
`FmFeesTransaction`, `FmFeesTransactionChild`, `FmFeesWeaver`,
`FmFeesInvoiceSetting`, `FmFeesInvoiceLineNote`, `FmFeesTransactionLineNote`,
`FmInvoiceType`, `FmFeesTypeKind`.

## 5. Spec File Duplicates (37 total)

`docs/specs/hr/aggregates.md` has the section for 26 stub aggregates duplicated
(half the file is a copy). `docs/specs/finance/aggregates.md` has 11 duplicates.
This was caused by a copy-paste error during spec cleanup.

## 6. TODO Comments (~20, excluding lint infrastructure)

Excluding the `educore-core::lint` module (which intentionally mentions
`todo!`/`unimplemented!` as patterns to detect), the remaining TODO comments
are concentrated in:

- `crates/domains/attendance/src/aggregate.rs` (2 — ClassAttendance invariant check, AttendanceBulk promote logic)
- `crates/domains/documents/src/aggregate.rs` (1 — local alias replacement)
- `crates/educore/src/subscribers.rs` (6 — TODO(SDK): dispatch … when subscribers are wired)
- `crates/adapters/storage-mysql/src/schema.rs` (5 — MySQL RLS skipped)
- `crates/adapters/storage-sqlite/src/schema.rs` (3 — SQLite RLS skipped)

The MySQL/SQLite RLS TODOs are architectural scope decisions (RLS is a
PostgreSQL/SurrealDB feature; the SQLite/MySQL adapters explicitly skip it),
so these are intentional and tracked.

## 7. Cleanup Status

| Metric | Status |
|--------|--------|
| `todo!()` in production | ✅ Zero |
| `unimplemented!()` in production | ✅ Zero |
| `panic!("TODO")` in production | ✅ Zero |
| `FIXME` comments | ✅ Zero |
| `Hack` comments | ✅ Zero |
| `unwrap`/`expect` in production | ✅ Zero (enforced by `educore-core::lint`) |
| `as` casts that truncate | ✅ Zero (enforced by `educore-core::lint`) |
| `serde_json::Value` in domain code | ✅ Zero (enforced by `educore-core::lint`) |

The lint infrastructure is solid. The remaining work is **stub aggregate
implementation + legacy prefix removal + spec cleanup**.

## 8. Recommended Implementation Order

The order is chosen to maximize production-readiness impact per session while
minimizing risk.

### Phase A: Foundation cleanup (1-2 sessions)

1. **Remove spec duplicates** — de-duplicate `docs/specs/hr/aggregates.md` and
   `docs/specs/finance/aggregates.md`. Mechanical change, 0 risk.
2. **Remove intentional TODO comments** — `crates/educore/src/subscribers.rs`
   TODO(SDK) markers, `crates/domains/documents/src/aggregate.rs` TODO marker.
   These are tracked as known gaps and should be either wired or removed.

### Phase B: Legacy prefix elimination (1-2 sessions)

3. **Rename `Fm*` → `*` in `educore-finance`** — 12 types, ~1180 references.
   Use `lsp_rename` for atomic rename across the workspace. This is a
   one-shot mechanical change. Replace `FmFeesGroup` → `FeesGroup`,
   `FmFeesType` → `FeesType`, etc.

### Phase C: Stub aggregate elimination (5-10 sessions)

4. **Implement real aggregates per HR template** — for each crate with stubs:
   - Start with aggregates that have real-spec text (see Wave 196 assessment
     plan).
   - Add mutator methods, port traits, service functions, behavioral tests.
   - Mirror the Wave 171-189 HR sweep pattern.

### Phase D: NotSupported stub elimination (1-2 sessions)

5. **Wire subscribers through dispatcher** — `crates/educore/src/subscribers.rs`
   has 6 TODO(SDK) markers for `dispatch X when Y happens`. Wire them through
   the new dispatcher (Wave 192+ wrapper layer).
6. **Implement assessment NotSupported stubs** — `student_take_online_exam`,
   `teacher_evaluation`, `ClassAttendance::verify_invariants`,
   `AttendanceBulk::promote_to_student_attendance`.

### Phase E: Mass dispatcher wrapper creation (1-2 sessions)

7. **Wrap all 509 service functions** through `CommandDispatcher::dispatch`
   using the Wave 192 template + the dispatcher-gen tool.

## 9. Total Estimated Sessions

| Phase | Sessions |
|-------|---------:|
| A: Foundation cleanup | 1-2 |
| B: Legacy prefix elimination | 1-2 |
| C: Stub aggregate elimination | 5-10 |
| D: NotSupported stub elimination | 1-2 |
| E: Dispatcher wrapper creation | 1-2 |
| **Total** | **9-18 sessions** |

## 10. Final Goals

- **Zero undocumented stubs.**
- **Zero placeholder business logic.**
- **Zero obsolete implementations.**
- **One production-quality implementation for every feature.**

---

Co-Authored-By: Antigravity <antigravity@google.com>
