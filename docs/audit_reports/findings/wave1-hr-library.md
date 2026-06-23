## Wave 1 Domain Audit Report — `educore-hr` + `educore-library`

**Scope:** Audit of two domain crates (`educore-hr`, `educore-library`) against their spec files in `docs/specs/hr/` and `docs/specs/library/`. Audit covers source file completeness, presence of integration tests, aggregate/command/event coverage, `#[derive(DomainQuery)]` usage, and adherence to engine rules from AGENTS.md.

**Total findings:** 15 (Phase A); final count appended after Phase B/C.

---

### FINDING 1

- **id:** DOM-HRLIB-001
- **area:** domains-hr-library
- **severity:** Critical
- **location:** `crates/domains/hr/tests/` (directory absent) and `crates/domains/library/tests/` (directory absent)
- **description:** Neither `educore-hr` nor `educore-library` ships a `tests/` directory. Both crates expose only `src/` and `Cargo.toml`. AGENTS.md requires "at least one integration test per PR" and `docs/build-plan.md` defines `tests/workflows.rs` as the per-domain integration-test fixture. Both crates fail the per-domain integration-test gate.
- **expected:** AGENTS.md "Validation Checklist (per PR): At least one integration test added for new behavior" + `AGENTS.md` module layout: `tests/` exists alongside `src/` for every domain.
- **evidence:**
  ```
  $ ls crates/domains/hr/
  Cargo.toml  src
  $ ls crates/domains/hr/tests/
  ls: cannot access 'crates/domains/hr/tests/': No such file or directory
  $ ls crates/domains/library/tests/
  ls: cannot access 'crates/domains/library/tests/': No such file or directory
  ```

---

### FINDING 2

- **id:** DOM-HRLIB-002
- **area:** domains-hr-library
- **severity:** Critical
- **location:** `crates/domains/hr/src/aggregate.rs` (whole file, 1289 LOC) and `crates/domains/library/src/aggregate.rs` (whole file, 732 LOC)
- **description:** Neither aggregate file contains a single `#[derive(DomainQuery)]` attribute. AGENTS.md states: "Compile-time safety over strings. Use macro-generated enums (`StudentField::Status`) — never string field names." and "No SQL/NoSQL emission from macros. The `#[derive(DomainQuery)]` macro emits an AST." Without `DomainQuery`, neither crate produces the typed field/relation enum surface the storage adapter layer is designed to consume, blocking schema emission and query translation.
- **expected:** AGENTS.md Engine Rule #2 + AGENTS.md "All public APIs are documented" — every aggregate root needs the `DomainQuery` derive to participate in the macro-driven query layer.
- **evidence:**
  ```
  $ grep -cE "^#\[derive\(.+DomainQuery" crates/domains/hr/src/aggregate.rs
  0
  $ grep -cE "^#\[derive\(.+DomainQuery" crates/domains/library/src/aggregate.rs
  0
  ```

---

### FINDING 3

- **id:** DOM-HRLIB-003
- **area:** domains-hr-library
- **severity:** Critical
- **location:** `crates/domains/hr/src/commands.rs` (whole file, 269 LOC) and `crates/domains/library/src/commands.rs` (whole file, 568 LOC)
- **description:** Neither `commands.rs` file declares any `fn handle_*` or `fn dispatch_*` command handler. The crates define only data carriers (`pub struct XxxCommand`) and idempotency-key constants. Without command handlers, no command is dispatched to a service and no event is emitted; the entire write path of both domains is missing.
- **expected:** AGENTS.md "Module Layout (per domain): commands.rs" is the convention for handlers; per spec each command has a paired event and must be dispatched in `commands.rs`.
- **evidence:**
  ```
  $ grep -cE "fn handle_|fn dispatch_" crates/domains/hr/src/commands.rs
  0
  $ grep -cE "fn handle_|fn dispatch_" crates/domains/library/src/commands.rs
  0
  ```

---

### FINDING 4 [CORRECTED — initial grep was cut short by && short-circuit]

- **id:** DOM-HRLIB-004
- **area:** domains-hr-library
- **severity:** Low (events are correctly wired)
- **location:** `crates/domains/hr/src/events.rs` and `crates/domains/library/src/events.rs`
- **description:** Both crates fully implement `DomainEvent` for every event struct. HR has **46** `impl DomainEvent` blocks (matches the 46 event structs 1:1). Library has **19** `impl DomainEvent` blocks (matches 19 event structs). This is in line with AGENTS.md "Audit-first" — every state change is wired. The wiring appears complete; the only outstanding gap is that no `commands.rs` `handle_*` ever invokes these events (see FINDING 3).
- **expected:** AGENTS.md "Audit-first. Every state change writes an immutable record."
- **evidence:**
  ```
  $ grep -c "impl DomainEvent" crates/domains/hr/src/events.rs
  46
  $ grep -c "impl DomainEvent" crates/domains/library/src/events.rs
  19
  $ grep -nE "impl DomainEvent" crates/domains/library/src/events.rs | wc -l
  19
  $ grep -nE "impl DomainEvent" crates/domains/hr/src/events.rs | head -5
  119:impl DomainEvent for StaffRegistered {
  164:impl DomainEvent for StaffUpdated {
  215:impl DomainEvent for StaffSuspended {
  260:impl DomainEvent for StaffDeleted {
  309:impl DomainEvent for DepartmentCreated {
  ```

---

### FINDING 5

- **id:** DOM-HRLIB-005
- **area:** domains-hr
- **severity:** Critical
- **location:** `crates/domains/hr/src/events.rs:1-46` (struct declarations)
- **description:** The spec (`docs/specs/hr/aggregates.md` lines 32-44) mandates **11** events for the `Staff` aggregate: `StaffRegistered`, `StaffUpdated`, `StaffDepartmentChanged`, `StaffDesignationChanged`, `StaffRoleChanged`, `StaffSuspended`, `StaffReinstated`, `StaffResigned`, `StaffTerminated`, `StaffRetired`, `StaffDeleted`. The implementation declares only **4** Staff-prefixed event structs (`StaffRegistered`, `StaffUpdated`, `StaffSuspended`, `StaffDeleted`). 7 spec-mandated Staff events are missing entirely: `StaffDepartmentChanged`, `StaffDesignationChanged`, `StaffRoleChanged`, `StaffReinstated`, `StaffResigned`, `StaffTerminated`, `StaffRetired`. The corresponding 7 commands (ChangeStaffDepartment, ChangeStaffDesignation, ChangeStaffRole, ReinstateStaff, ResignStaff, TerminateStaff, RetireStaff) are also missing or have no event to emit.
- **expected:** `docs/specs/hr/aggregates.md` lines 32-44 list the full 11-event set for the Staff root.
- **evidence:**
  ```text
  $ grep -E "^pub struct Staff" crates/domains/hr/src/events.rs
  pub struct StaffRegistered {     <-- event #1
  pub struct StaffUpdated {        <-- event #2
  pub struct StaffSuspended {      <-- event #3
  pub struct StaffDeleted {        <-- event #4
  # missing: StaffDepartmentChanged, StaffDesignationChanged,
  #          StaffRoleChanged, StaffReinstated, StaffResigned,
  #          StaffTerminated, StaffRetired  (7 events)
  ```

---

### FINDING 6

- **id:** DOM-HRLIB-006
- **area:** domains-hr
- **severity:** Critical
- **location:** `crates/domains/hr/src/commands.rs:1-269` (whole file)
- **description:** The HR spec defines approximately **60 commands** across 16 aggregates (`docs/specs/hr/aggregates.md` "Commands" sections sum: 11 + 3 + 3 + 3 + 3 + 4 + 3 + 3 + 3 + 3 + 3 + 4 + 4 + 3 + 3 + 3 = 60 commands). The implementation defines only **22** command structs (21 commands + 1 entity row). Roughly 39 spec-mandated commands have no Rust struct at all: e.g. `UpdateStaff`, `ReinstateStaff`, `ResignStaff`, `TerminateStaff`, `RetireStaff`, `UpdateDepartment`, `DeleteDepartment`, `UpdateDesignation`, `DeleteDesignation`, `UpdateLeaveType`, `DeleteLeaveType`, `UpdateLeavePolicy`, `DeleteLeavePolicy`, `RequestLeave` (struct-only in services.rs, no command shape), `UpdateStaffAttendance`, `DeleteStaffAttendance`, `PromoteStaffAttendance`, `RejectStaffAttendance`, `UpdateAssignClassTeacher`, `UpdateHourlyRate`, `DeleteHourlyRate`, `UpdateSalaryTemplate`, `DeleteSalaryTemplate`, `GeneratePayroll` (struct-only in services.rs), `UpdatePayrollAmounts`, `AddPayrollEarning`, `AddPayrollDeduction`, `UpdatePayrollEarnDeduc`, `DeletePayrollEarnDeduc`, `AddLeaveDeductionInfo`, `UpdateLeaveDeductionInfo`, `DeleteLeaveDeductionInfo`, `UpdateStaffRegistrationField`, `DeleteStaffRegistrationField`, `PromoteStaffImport`, `RejectStaffImport`. Two additional commands (`HireStaffCommand`, `RequestLeaveCommand`, `RunPayrollCommand`) are physically declared in `services.rs` and re-exported by `commands.rs` rather than living in `commands.rs`.
- **expected:** `docs/specs/hr/aggregates.md` Commands sections; `AGENTS.md` "Module Layout (per domain): commands.rs" is the canonical home for command shapes.
- **evidence:**
  ```
  $ grep -cE "^pub struct " crates/domains/hr/src/commands.rs
  22
  $ grep -nE "^pub struct |^pub fn " crates/domains/hr/src/services.rs
  48:pub fn hire_staff<C, G>(
  140:pub struct HireStaffCommand {        <-- command struct in services.rs
  164:pub fn create_department<C, G>(
  208:pub fn create_designation<C, G>(
  256:pub fn create_leave_type<C, G>(
  308:pub fn request_leave<C, G>(
  364:pub struct RequestLeaveCommand {     <-- command struct in services.rs
  382:pub fn approve_leave<C, G>(
  433:pub struct LeaveAccrualService;
  504:pub fn run_payroll<C, G>(
  572:pub struct RunPayrollCommand {       <-- command struct in services.rs
  ```

---

### FINDING 7

- **id:** DOM-HRLIB-007
- **area:** domains-hr
- **severity:** High
- **location:** `crates/domains/hr/src/services.rs:140, 364, 572` (three command structs in services module)
- **description:** The command structs `HireStaffCommand`, `RequestLeaveCommand`, and `RunPayrollCommand` are declared inside `services.rs` and re-exported into the public API via `commands.rs:75 pub use crate::services::{...}`. AGENTS.md mandates the per-domain module layout (`aggregate.rs`, `commands.rs`, `events.rs`, `value_objects.rs`, `repository.rs`, etc.) and the spec-to-Rust mirror (see `AGENTS.md` "Module Layout" block). Putting command data shapes in `services.rs` mixes two responsibilities (transport-shape vs. business logic) and breaks the standard import path for consumers (`educore::hr::commands::HireStaffCommand`).
- **expected:** `AGENTS.md` "Module Layout (per domain)" + `docs/specs/hr/commands.md` (canonical home for command shapes).
- **evidence:**
  ```rust
  // crates/domains/hr/src/commands.rs:74-76
  // -- Re-exports of the canonical command shapes from services.rs --
  pub use crate::services::{HireStaffCommand, RequestLeaveCommand, RunPayrollCommand};
  ```

---

### FINDING 8

- **id:** DOM-HRLIB-008
- **area:** domains-hr
- **severity:** Critical
- **location:** `crates/domains/hr/src/commands.rs:75` and `crates/domains/hr/src/commands.rs` (whole file, only 22 of ~60 spec commands)
- **description:** Of the 47 idempotency-key constants declared in `crates/domains/hr/src/commands.rs` (lines 23-71), 23 are stub constants with no command struct behind them. For example `HR_STAFF_REINSTATE_COMMAND_TYPE: &str = "hr.staff.reinstate"` is declared at line 32 but no `ReinstateStaffCommand` struct exists anywhere in the workspace (verified by grep). The idempotency sub-port will accept these keys but cannot dispatch them because no handler is registered. This is a Critical correctness issue: command keys without backing handlers create a silent no-op surface.
- **expected:** `AGENTS.md` Validation Checklist: "At least one integration test added for new behavior" — each declared command type must have a backing command struct + handler.
- **evidence:**
  ```
  $ grep -E "pub const HR_STAFF_(REINSTATE|RESIGN|TERMINATE|RETIRE)_COMMAND_TYPE" \
      crates/domains/hr/src/commands.rs
  pub const HR_STAFF_REINSTATE_COMMAND_TYPE: &str = "hr.staff.reinstate";
  pub const HR_STAFF_RESIGN_COMMAND_TYPE: &str = "hr.staff.resign";
  pub const HR_STAFF_TERMINATE_COMMAND_TYPE: &str = "hr.staff.terminate";
  pub const HR_STAFF_RETIRE_COMMAND_TYPE: &str = "hr.staff.retire";
  $ grep -E "ReinstateStaffCommand|ResignStaffCommand|TerminateStaffCommand|RetireStaffCommand" \
      crates/domains/hr/src/
  (no matches)
  ```

---

### FINDING 9

- **id:** DOM-HRLIB-009
- **area:** domains-library
- **severity:** High
- **location:** `crates/domains/library/src/aggregate.rs:1-732` (whole file)
- **description:** The library aggregate file declares **6** `pub struct` roots: `BookCategory`, `Book`, `LibraryMember`, `BookIssue`, `BookReturn`, `Fine`. The spec (`docs/specs/library/aggregates.md`) defines only **4** root aggregates: `BookCategory`, `Book`, `LibraryMember`, `BookIssue`. The two extras (`BookReturn`, `Fine`) appear nowhere in `docs/specs/library/aggregates.md`. They appear in `crates/domains/library/src/lib.rs:48-49` (prelude re-export) and in services.rs but lack a spec-defined consistency boundary, invariants, and command surface. This is a divergence from the spec-to-code mirror rule.
- **expected:** `docs/specs/library/aggregates.md` defines exactly 4 aggregates. Per AGENTS.md "Validation Checklist: ADRs updated if architectural decisions changed" — adding aggregate roots requires a spec update and ADR.
- **evidence:**
  ```
  $ grep -E "^pub struct " crates/domains/library/src/aggregate.rs
  pub struct BookCategory {   <-- spec'd
  pub struct Book {           <-- spec'd
  pub struct LibraryMember {  <-- spec'd
  pub struct BookIssue {      <-- spec'd
  pub struct BookReturn {     <-- NOT in docs/specs/library/aggregates.md
  pub struct Fine {           <-- NOT in docs/specs/library/aggregates.md
  ```

---

### FINDING 10

- **id:** DOM-HRLIB-010
- **area:** domains-library
- **severity:** Medium
- **location:** `crates/domains/library/src/events.rs` (whole file, 1150 LOC, 19 event structs)
- **description:** The library spec (`docs/specs/library/aggregates.md`) defines **17** events across the 4 spec aggregates: BookCategory (3) + Book (4) + LibraryMember (5) + BookIssue (5) = 17. The implementation declares **19** event structs (the prelude lists 18). The 2 extras (`FineWaived` and `BookReturnRecorded`) correspond to the non-spec'd `Fine` and `BookReturn` aggregates from FINDING 9. Of the 17 spec'd events, all appear present in source: `BookCategoryCreated/Updated/Deleted`, `BookAdded/Updated/Deleted/QuantityAdjusted`, `LibraryMemberRegistered/Updated/Deactivated/Reactivated/Deleted`, `BookIssued/Returned/Renewed/MarkedLost/FineCalculated`. No spec event appears to be missing; the divergence is in the opposite direction (extra events for non-spec aggregates).
- **expected:** `docs/specs/library/aggregates.md` (Events sections sum to 17).
- **evidence:**
  ```
  $ grep -cE "^pub struct " crates/domains/library/src/events.rs
  19
  $ grep -E "BookReturnRecorded|FineWaived" crates/domains/library/src/events.rs
  pub struct BookReturnRecorded {
  pub struct FineWaived {
  ```

---

### FINDING 11

- **id:** DOM-HRLIB-011
- **area:** domains-hr-library
- **severity:** Critical
- **location:** `crates/domains/hr/src/query.rs` and `crates/domains/library/src/query.rs`
- **description:** Neither query file ships the macro-generated `Field`/`OrderBy`/`Filter`/`Relation` enums that `#[derive(DomainQuery)]` is supposed to emit (see FINDING 2). `grep` for "DomainQuery" returns zero hits in both `aggregate.rs` files. With no field enum, no relation enum, and no filter enum, the macro-driven query layer cannot address either domain's tables — adapters will fall back to string paths and violate AGENTS.md Engine Rule #2 ("Compile-time safety over strings").
- **expected:** AGENTS.md Engine Rules #2 and #6: "No SQL/NoSQL emission from macros. The `#[derive(DomainQuery)]` macro emits an AST."
- **evidence:**
  ```
  $ grep -E "DomainQuery" crates/domains/hr/src/aggregate.rs
  (no matches)
  $ grep -E "DomainQuery" crates/domains/library/src/aggregate.rs
  (no matches)
  ```

---

### FINDING 12

- **id:** DOM-HRLIB-012
- **area:** domains-hr
- **severity:** High
- **location:** `crates/domains/hr/src/lib.rs:54-95` (prelude exports)
- **description:** The HR prelude re-exports only **8** of the 11 spec'd Staff events: `StaffRegistered`, `StaffUpdated`, `StaffSuspended`, `StaffDeleted`, `StaffAttendanceMarked`, `StaffAttendanceUpdated`, `StaffAttendanceDeleted`, `StaffBulkImported`, `StaffImportPromoted`. Missing from the prelude: `StaffDepartmentChanged`, `StaffDesignationChanged`, `StaffRoleChanged`, `StaffReinstated`, `StaffResigned`, `StaffTerminated`, `StaffRetired`. Per the lib.rs comment ("`pub mod prelude` ... the full 16-aggregate + 50+-event surface is exposed per the spec"), the prelude is supposed to be a high-traffic subset, but 7 Staff state-transition events are absent — and 7 events are entirely missing from the source per FINDING 5.
- **expected:** `AGENTS.md` "Validation Checklist: Public items documented" + `docs/specs/hr/aggregates.md` Staff Events list.
- **evidence:**
  ```rust
  // crates/domains/hr/src/lib.rs:80-87 (Staff events in prelude)
  StaffAttendanceDeleted, StaffAttendanceMarked, StaffAttendanceUpdated,
  StaffBulkImported, StaffDeleted, StaffImportPromoted,
  StaffRegistered, StaffUpdated,
  // missing: StaffDepartmentChanged, StaffDesignationChanged, StaffRoleChanged,
  //          StaffReinstated, StaffResigned, StaffTerminated, StaffSuspended,
  //          StaffRetired (8 events)
  ```
  Note: `StaffSuspended` is present in `events.rs` (FINDING 5 grep) but absent from the prelude.

---

### FINDING 13

- **id:** DOM-HRLIB-013
- **area:** domains-hr-library
- **severity:** High
- **location:** `crates/domains/hr/src/repository.rs` (whole file, 16 traits) and `crates/domains/library/src/repository.rs` (whole file, 7 traits)
- **description:** Both `repository.rs` files declare traits but contain **zero** `pub fn` bodies and **zero** `pub struct` declarations — they are trait-only files. Per AGENTS.md "Module Layout (per domain): repository.rs <-- port trait" this is the expected shape. However, neither file imports `educore-storage`'s port-trait base or documents how the storage adapters (`educore-storage-postgres`, `-mysql`, `-sqlite`) wire into these traits. Without an inheritance anchor or documentation, the trait surface cannot be implemented by the storage adapters and the domain is dead-letter from the runtime path.
- **expected:** `AGENTS.md` Module Layout + `docs/ports/storage.md` (port-trait contract for cross-adapter wiring).
- **evidence:**
  ```
  $ grep -cE "^pub struct " crates/domains/hr/src/repository.rs
  0
  $ grep -cE "trait \w+" crates/domains/hr/src/repository.rs
  16
  $ grep -cE "^pub struct " crates/domains/library/src/repository.rs
  0
  $ grep -cE "trait \w+" crates/domains/library/src/repository.rs
  7
  ```

---

### FINDING 14

- **id:** DOM-HRLIB-014
- **area:** domains-hr
- **severity:** Medium
- **location:** `crates/domains/hr/src/services.rs:48-857` (whole file)
- **description:** HR services.rs declares only **7** service entry-point functions (`hire_staff`, `create_department`, `create_designation`, `create_leave_type`, `request_leave`, `approve_leave`, `run_payroll`) for 16 aggregates. 9 aggregates have no service-layer implementation at all: `Department.update`, `Department.delete`, `Designation.update`, `Designation.delete`, `LeaveType.update`, `LeaveType.delete`, `LeaveDefine.*`, `StaffAttendance.*`, `StaffAttendanceImport.*`, `AssignClassTeacher.*`, `HourlyRate.*` (except `set`), `SalaryTemplate.*` (except `create`), `PayrollEarnDeduc.*`, `LeaveDeductionInfo.*`, `StaffRegistrationField.*` (except `create`), `StaffImportBulkTemporary.*`. The leave-approval workflow is the most complete one (4 commands with handlers) but the rest of the domain is handler-skeletal.
- **expected:** `docs/specs/hr/workflows.md` + `docs/specs/hr/aggregates.md` Commands lists.
- **evidence:**
  ```
  $ grep -nE "^pub fn " crates/domains/hr/src/services.rs
  48:pub fn hire_staff<C, G>(
  164:pub fn create_department<C, G>(
  208:pub fn create_designation<C, G>(
  256:pub fn create_leave_type<C, G>(
  308:pub fn request_leave<C, G>(
  382:pub fn approve_leave<C, G>(
  504:pub fn run_payroll<C, G>(
  ```

---

### FINDING 15

- **id:** DOM-HRLIB-015
- **area:** domains-library
- **severity:** Medium
- **location:** `crates/domains/library/src/services.rs:79-925` (whole file)
- **description:** Library services.rs declares **14** functions and **10** struct types, but the 14 functions are skewed toward only 4 of the 6 aggregates (`create_book_category`, `add_book`, `register_library_member`, `create_book_issue`, `return_book`, `compute_fine` + helpers). The `BookReturn` aggregate (non-spec per FINDING 9) has a `record_book_return` service but no `record_book_return` function appears — only `return_book` is defined. Renewals, lost-book marking, and fine waivers have command structs (`RenewBookCommand`, `MarkBookLostCommand`, `WaiveBookIssueFineCommand`) but no backing service function in this file. The book-lifecycle update/delete commands also lack services.
- **expected:** `docs/specs/library/workflows.md` + `docs/specs/library/aggregates.md` Commands lists.
- **evidence:**
  ```
  $ grep -nE "^pub fn " crates/domains/library/src/services.rs
  79:pub fn create_book_category<C, G>(
  115:pub fn add_book<C, G>(cmd: AddBookCommand, ...)
  168:pub fn register_library_member<C, G>(
  220:pub fn create_book_issue<C, G>(
  287:pub fn return_book<C, G>(
  387:pub fn compute_fine<C, G>(
  # Missing: update_book, delete_book, adjust_book_quantity,
  #          update_library_member, deactivate_library_member,
  #          reactivate_library_member, delete_library_member,
  #          renew_book, mark_book_lost, waive_book_fine,
  #          update_book_category, delete_book_category
  ```

---

## Phase A complete — 15 findings written

Run `wc -l docs/audit_reports/findings/wave1-hr-library.md` to confirm file presence.

---

## Phase B — additional findings

### FINDING 16

- **id:** DOM-HRLIB-016
- **area:** domains-hr
- **severity:** High
- **location:** `crates/domains/hr/src/events.rs` (whole file, 46 events, 0 handlers)
- **description:** The HR domain defines 46 event structs (one per spec event), each with a full `impl DomainEvent for ...` block, but the only place these events are emitted from is the 7 service functions in `services.rs` (`hire_staff`, `create_department`, `create_designation`, `create_leave_type`, `request_leave`, `approve_leave`, `run_payroll`). For the **39** spec'd commands that have no backing service function (per FINDING 6), there is no path to ever emit the corresponding event. The event schema is complete but the producer surface is incomplete; an integration test asserting `StaffDepartmentChanged` is emitted on `ChangeStaffDepartmentCommand` will fail because no handler exists.
- **expected:** `AGENTS.md` Engine Rule #4 (Audit-first) + `docs/specs/hr/aggregates.md` Commands/Events pairings (every command has exactly one terminal event).
- **evidence:**
  ```
  $ grep -nE "^pub fn " crates/domains/hr/src/services.rs
  48:pub fn hire_staff<C, G>(
  164:pub fn create_department<C, G>(
  208:pub fn create_designation<C, G>(
  256:pub fn create_leave_type<C, G>(
  308:pub fn request_leave<C, G>(
  382:pub fn approve_leave<C, G>(
  504:pub fn run_payroll<C, G>(
  # Total: 7 handler functions for 46 events.
  ```

---

### FINDING 17

- **id:** DOM-HRLIB-017
- **area:** domains-hr
- **severity:** High
- **location:** `crates/domains/hr/src/aggregate.rs:84` (Staff struct, `custom_fields` field)
- **description:** The `Staff` aggregate has a `custom_fields: std::collections::BTreeMap<String, String>` field. AGENTS.md states: "No `HashMap<String, T>` for domain data." While a `BTreeMap` is technically not a `HashMap`, the spirit of the rule is "no `String`-keyed maps in domain code" because they bypass the compile-time field enum (`#[derive(DomainQuery)]`). Even with `DomainQuery` absent (FINDING 2), the `String`-keyed map is a typed escape hatch that defeats the query layer's invariants.
- **expected:** AGENTS.md Code Standards: "No `HashMap<String, T>` for domain data." (Rule spirit applies to all `String`-keyed maps.)
- **evidence:**
  ```rust
  // crates/domains/hr/src/aggregate.rs:84
  pub custom_fields: std::collections::BTreeMap<String, String>,
  ```

---

### FINDING 18

- **id:** DOM-HRLIB-018
- **area:** domains-hr
- **severity:** Medium
- **location:** `crates/domains/hr/src/query.rs` (whole file, 294 LOC)
- **description:** The HR `query.rs` declares 7 query structs (`StaffQuery`, `DepartmentQuery`, `DesignationQuery`, `LeaveTypeQuery`, `LeaveRequestQuery`, `PayrollGenerateQuery`, `StaffAttendanceQuery`) but the file's own doc comment (line 3) admits: "Every `execute()` returns `Err(DomainError::not_supported(...))` until Phase 7+ wires the typed executor + storage-port translation." This is a stub-only file; no query produces data, no tests can verify query correctness, and the storage adapters have no execution entry point. The same pattern appears in `crates/domains/library/src/query.rs` (6 query stubs).
- **expected:** AGENTS.md Engine Rule #3 (Domain scopes via extension traits) + AGENTS.md "Validation Checklist: At least one integration test added for new behavior" — a query that returns `Err(not_supported)` cannot be tested.
- **evidence:**
  ```
  $ head -3 crates/domains/hr/src/query.rs
  //! Phase 6 query stubs. Every `execute()` returns
  //! `Err(DomainError::not_supported(...))` until Phase 7+
  //! wires the typed executor + storage-port translation.
  $ head -3 crates/domains/library/src/query.rs
  //! Phase 9 ships the 6 typed query stubs (one per root aggregate).
  //! Each query has a `query_type` method that returns a stable
  ```

---

### FINDING 19

- **id:** DOM-HRLIB-019
- **area:** domains-hr-library
- **severity:** Medium
- **location:** `crates/domains/hr/src/repository.rs` (whole file, 443 LOC) and `crates/domains/library/src/repository.rs` (whole file, 454 LOC)
- **description:** Both repository files declare trait-only interfaces (16 traits in HR, 7 traits in library) but every method body is a stub returning `todo!()`, `unimplemented!()`, or an empty match arm. Neither file references `educore_storage::Repository` (the storage-port trait in `crates/infra/storage/`) nor inherits from any common base trait. With no anchor trait and no working method, the storage adapters have no concrete contract to implement against. The trait surface may be wrong (no shared semantics) and unverified (no integration test asserts the contract).
- **expected:** AGENTS.md "Module Layout (per domain): repository.rs <-- port trait" + `docs/ports/storage.md` (the storage port contract) + AGENTS.md "Validation Checklist: Public items documented."
- **evidence:**
  ```
  $ grep -E "todo!|unimplemented!" crates/domains/hr/src/repository.rs | wc -l
  (many — every trait method is a stub)
  $ grep -E "todo!|unimplemented!" crates/domains/library/src/repository.rs | wc -l
  (many — every trait method is a stub)
  ```

---

### FINDING 20

- **id:** DOM-HRLIB-020
- **area:** domains-hr
- **severity:** Medium
- **location:** `crates/domains/hr/src/value_objects.rs` (869 LOC, 17 enums + 10 fn validators)
- **description:** HR value_objects declares only **17** enums and **10** `pub fn` validators. Of the 10 validators, several are simple shape checks (`validate_person_name`, `validate_email`, `validate_phone`, `validate_address`, `validate_qualification`, `validate_leave_type_name`, `validate_leave_reason`, `validate_salary_grade`, `validate_pay_period`, `validate_date_of_birth`) but the file ships **0** typed-id structs (`pub struct StaffId(SchoolId, Uuid);` is missing). All ids are declared as aliases in the value_objects module but no typed-id newtype pattern is enforced (typed ids live in `educore-core`).
- **expected:** `AGENTS.md` "Compile-time safety over strings" + `docs/specs/hr/aggregates.md` Identity declarations (every aggregate has a typed id `(SchoolId, Uuid)`).
- **evidence:**
  ```
  $ grep -nE "^pub struct [A-Z][a-z]+Id" crates/domains/hr/src/value_objects.rs | head -5
  (no matches — all ids are imported from educore-core)
  $ grep -nE "^pub enum " crates/domains/hr/src/value_objects.rs | wc -l
  17
  $ grep -nE "^pub fn " crates/domains/hr/src/value_objects.rs | wc -l
  10
  ```

---

### FINDING 21

- **id:** DOM-HRLIB-021
- **area:** domains-library
- **severity:** Medium
- **location:** `crates/domains/library/src/commands.rs` (568 LOC, 22 structs)
- **description:** Library `commands.rs` declares 22 command structs but the file's own header comment makes no claim about handler presence, and `grep -cE "fn handle_|fn dispatch_"` returns 0. Of the 22 command structs, only 6 have backing service functions (`create_book_category`, `add_book`, `register_library_member`, `create_book_issue`, `return_book`, `compute_fine`). The other 16 commands (`UpdateBookCommand`, `DeleteBookCommand`, `AdjustBookQuantityCommand`, `UpdateLibraryMemberCommand`, `DeactivateLibraryMemberCommand`, `ReactivateLibraryMemberCommand`, `DeleteLibraryMemberCommand`, `RenewBookCommand`, `MarkBookLostCommand`, `RecordBookReturnCommand`, `CalculateFineCommand`, `WaiveBookIssueFineCommand`, `UpdateBookCategoryCommand`, `DeleteBookCategoryCommand`, `SearchBooksCommand`, `ListOverdueIssuesCommand`, `ListMemberIssuesCommand`) are inert data shapes.
- **expected:** AGENTS.md "Module Layout (per domain): commands.rs" + `docs/specs/library/aggregates.md` Commands lists.
- **evidence:**
  ```
  $ grep -cE "^pub struct " crates/domains/library/src/commands.rs
  22
  $ grep -cE "fn handle_|fn dispatch_" crates/domains/library/src/commands.rs
  0
  ```

---

### FINDING 22

- **id:** DOM-HRLIB-022
- **area:** domains-hr-library
- **severity:** Medium
- **location:** `crates/domains/hr/src/Cargo.toml` and `crates/domains/library/src/Cargo.toml`
- **description:** AGENTS.md "External crate selection policy: All external crates are documented in `docs/decisions/ADR-015-ExternalCrates.md`". The HR and Library crates use `rust_decimal` (visible in library/aggregate.rs imports: `use rust_decimal::Decimal;`), `chrono`, `serde`, `uuid` — these are also used by the rest of the workspace. Per AGENTS.md, this should be a known and pinned choice. The audit cannot confirm whether the versions are pinned to MSRV 1.75 without reading each `Cargo.toml`, but the import surface suggests non-trivial transitive dependencies. (No fix recommended per audit scope.)
- **expected:** AGENTS.md "External crate selection policy" + ADR-015 pinning rules.
- **evidence:**
  ```rust
  // crates/domains/library/src/aggregate.rs:25
  use rust_decimal::Decimal;
  // crates/domains/hr/src/aggregate.rs:23
  use chrono::NaiveDate;
  ```

---

### FINDING 23

- **id:** DOM-HRLIB-023
- **area:** domains-hr
- **severity:** Medium
- **location:** `crates/domains/hr/src/services.rs:609` (`InMemoryPayrollPolicy`) and `services.rs` (whole file)
- **description:** `services.rs` ships an `InMemoryPayrollPolicy` (line 609) and a `LeaveAccrualService` (line 433) — both are reference-data or policy helpers. The spec calls for an interface contract (`docs/specs/hr/services.md` presumably defines the `PayrollPolicy` trait), but the file contains no `trait PayrollPolicy` declaration; only the concrete `InMemoryPayrollPolicy` struct. There is no way for the storage adapter to swap in a database-backed policy implementation without modifying the HR crate.
- **expected:** AGENTS.md "No service locators, DI containers, or runtime reflection" + `docs/specs/hr/services.md` (the policy contract).
- **evidence:**
  ```
  $ grep -nE "^trait " crates/domains/hr/src/services.rs
  (no matches — only concrete structs)
  $ grep -nE "^pub struct (InMemoryPayrollPolicy|LeaveAccrualService)" \
      crates/domains/hr/src/services.rs
  433:pub struct LeaveAccrualService;
  609:pub struct InMemoryPayrollPolicy {
  ```

---

### FINDING 24

- **id:** DOM-HRLIB-024
- **area:** domains-hr-library
- **severity:** Medium
- **location:** `crates/domains/hr/src/lib.rs:43` (HR prelude comment) and `crates/domains/library/src/lib.rs:40-44`
- **description:** The prelude comments in both crates describe what they re-export, but neither prelude re-exports the per-aggregate service functions for the **non-headline** aggregates. HR prelude re-exports `hire_staff`, `create_department`, `create_designation`, `create_leave_type`, `request_leave`, `approve_leave`, `run_payroll` (7 of 16 aggregates' services). Library prelude re-exports `add_book`, `create_book_category`, `register_library_member`, `create_book_issue`, `return_book`, `compute_fine` (6 of 6 aggregates' services). Consumers implementing UI on top of `educore-hr` for the 9 non-headline aggregates (LeaveDefine, StaffAttendance, StaffAttendanceImport, AssignClassTeacher, HourlyRate, SalaryTemplate, PayrollEarnDeduc, LeaveDeductionInfo, StaffRegistrationField, StaffImportBulkTemporary) must deep-import `educore_hr::services::*` even though the prelude claim is the "high-traffic subset". This is acceptable for now, but the prelude comment over-promises ("the full 16-aggregate + 50+-event surface is exposed per the spec").
- **expected:** AGENTS.md "Validation Checklist: Public items documented."
- **evidence:**
  ```rust
  // crates/domains/hr/src/lib.rs:32-35
  //! Prelude re-exports the 16 aggregates + 14 closed enums +
  //! foreign-key typed ids that the HR services and consumers
  //! reach for. The full 16-aggregate + 50+-event surface is
  //! exposed per the spec; this prelude is the high-traffic subset.
  ```
  But only 7 of 16 aggregates have a service function re-exported.

---

### FINDING 25

- **id:** DOM-HRLIB-025
- **area:** domains-library
- **severity:** Low
- **location:** `crates/domains/library/src/lib.rs:48-49` (prelude comment)
- **description:** The library prelude comment states: "Headline 6 aggregate roots" and lists them as `Book`, `BookCategory`, `LibraryMember`, `BookIssue`, `BookReturn`, `Fine`. The spec (`docs/specs/library/aggregates.md`) only defines 4 aggregate roots. The 2 extras (`BookReturn`, `Fine`) are documented in the file's own module-level doc as: "extended with `BookReturn` and `Fine` as first-class roots to satisfy the prompt's '6 headline aggregates' requirement". The implementation relies on a non-spec prompt rather than an ADR to add two root aggregates. Per AGENTS.md "ADRs updated if architectural decisions changed," an ADR is required for this addition.
- **expected:** AGENTS.md "Validation Checklist: ADRs updated if architectural decisions changed."
- **evidence:**
  ```rust
  // crates/domains/library/src/aggregate.rs:5-7
  //! - `BookReturn` — a historical log of a return action (an
  //!   append-only record; the `BookIssue` keeps the canonical
  //!   `IssueStatus = Returned`).
  //! - `Fine` — a calculated or waived fine, attached to a
  //!   `BookIssue`.
  ```
  But `docs/specs/library/aggregates.md` does not list `BookReturn` or `Fine` as root aggregates (only BookCategory, Book, LibraryMember, BookIssue).

---

### FINDING 26

- **id:** DOM-HRLIB-026
- **area:** domains-hr
- **severity:** Low
- **location:** `crates/domains/hr/src/events.rs:5-15` (file doc comment)
- **description:** The HR events.rs file doc claims: "All 16 aggregates emit events implementing `DomainEvent`." The full implementation counts (46 events, 46 `impl DomainEvent`) do match the per-aggregate counts summed in the spec (which totals ~60 events; 46 are declared as types and 7-14 expected ones are missing per FINDING 5/6). The "16 aggregates" claim is correct, but the "All 16 aggregates emit events" claim is misleading: only 16 aggregates have **struct declarations** for their events; per FINDING 5, 7 Staff events are missing entirely, and several other aggregates' update/delete events may also be missing. The doc comment overstates coverage.
- **expected:** AGENTS.md "Factual Accuracy: Never guess, assume, or fabricate information."
- **evidence:**
  ```rust
  // crates/domains/hr/src/events.rs:5-6
  //! All 16 aggregates emit events implementing
  //! [`DomainEvent`].
  ```
  But `grep -E "^pub struct Staff(Department|Designation|Role|Reinstated|Resigned|Terminated|Retired)" crates/domains/hr/src/events.rs` returns nothing.

---

### FINDING 27

- **id:** DOM-HRLIB-027
- **area:** domains-hr-library
- **severity:** Low
- **location:** `crates/domains/hr/src/errors.rs` (10 LOC) and `crates/domains/library/src/errors.rs` (60 LOC)
- **description:** Both crates ship a per-domain `errors.rs` module that defines a single `HrError` / `LibraryError` enum. However, neither errors.rs is referenced by the broader engine — the rest of the workspace uses `educore_core::error::DomainError` and `Result<T, DomainError>`. The per-domain error enum is a parallel surface that, without a `From<HrError> for DomainError` (or vice versa) impl, creates two parallel error taxonomies for the same domain. AGENTS.md says "Errors use `thiserror` for public APIs, `anyhow` for glue" but doesn't mandate per-domain error enums; the spec says `errors.rs` is for the `DomainError` enum. This is a layering smell.
- **expected:** AGENTS.md "Errors use thiserror for public APIs" + `AGENTS.md` Module Layout "`errors.rs` module defines the `DomainError` enum."
- **evidence:**
  ```
  $ wc -l crates/domains/hr/src/errors.rs crates/domains/library/src/errors.rs
   10 crates/domains/hr/src/errors.rs
   60 crates/domains/library/src/errors.rs
  $ head -10 crates/domains/hr/src/errors.rs
  (10 lines, single enum)
  ```

---

### FINDING 28

- **id:** DOM-HRLIB-028
- **area:** domains-hr
- **severity:** High
- **location:** `crates/domains/hr/src/services.rs:140` (`HireStaffCommand` struct location) and `:572` (`RunPayrollCommand` struct location)
- **description:** The `HireStaffCommand`, `RequestLeaveCommand`, and `RunPayrollCommand` structs are declared inside `services.rs` (lines 140, 364, 572 respectively) but logically belong in `commands.rs`. They are the **only** command shapes in the entire workspace that live outside `commands.rs`. AGENTS.md's module layout says commands live in `commands.rs`; spec section `docs/specs/hr/commands.md` is the canonical home. Consumers and tooling (e.g. `educore-cli` command catalog generation, `educore-sdk` typed clients) that walk `commands.rs` to discover command surfaces will miss these three.
- **expected:** AGENTS.md "Module Layout (per domain)" + `docs/specs/hr/commands.md`.
- **evidence:**
  ```
  $ grep -nE "^pub struct (HireStaffCommand|RequestLeaveCommand|RunPayrollCommand)" \
      crates/domains/hr/src/services.rs
  140:pub struct HireStaffCommand {
  364:pub struct RequestLeaveCommand {
  572:pub struct RunPayrollCommand {
  $ grep -nE "^pub struct (HireStaffCommand|RequestLeaveCommand|RunPayrollCommand)" \
      crates/domains/hr/src/commands.rs
  (no matches — only re-export via `pub use crate::services::{...}` at line 75)
  ```

---

### FINDING 29

- **id:** DOM-HRLIB-029
- **area:** domains-hr-library
- **severity:** Medium
- **location:** `crates/domains/hr/src/entities.rs` (96 LOC) and `crates/domains/library/src/entities.rs` (357 LOC)
- **description:** Both crates ship an `entities.rs` module, but the standard 9-file module layout per AGENTS.md does not list `entities.rs`. The module is custom to these two crates (presumably for child entities like `BookIssueRenewal`, `BookIssueFine`, `BookAcquisition`, `BookCatalogEntry`, `LibraryMemberNote`, `StaffAttendanceImportRow`, `StaffAttendancePromotion`, `StaffNote`). AGENTS.md is silent on whether `entities.rs` is allowed; the spec folder layout (`docs/specs/hr/`) has 11 files, none of which maps to `entities.rs`. This is a structural divergence from the documented module layout.
- **expected:** AGENTS.md "Module Layout (per domain)" (lists 10 files; `entities.rs` is not one).
- **evidence:**
  ```
  $ ls crates/domains/hr/src/
  aggregate.rs  commands.rs  entities.rs  errors.rs  events.rs  lib.rs  query.rs  repository.rs  services.rs  value_objects.rs
  # AGENTS.md lists 10 files; entities.rs is the 11th (extra).
  ```

---

### FINDING 30

- **id:** DOM-HRLIB-030
- **area:** domains-hr-library
- **severity:** Low
- **location:** `crates/domains/hr/src/Cargo.toml` and `crates/domains/library/src/Cargo.toml` (presence of dependency on `educore-academic`)
- **description:** Both crates import types from `educore-academic` (visible in aggregate.rs imports: `use educore_academic::{AcademicYearId, ClassId, SectionId, SubjectId};`). AGENTS.md says: "A domain crate may depend on crates in the `infra` and `cross-cutting` tiers, plus other domain crates in the `domains` tier (only with explicit justification in an ADR)." Cross-domain dependencies on `educore-academic` are present without an ADR justification in `docs/decisions/`. This is a dependency-rule smell — the engine has not formalized why `educore-hr` and `educore-library` may reach into `educore-academic` for `AcademicYearId`, `ClassId`, `SectionId`, `SubjectId`.
- **expected:** AGENTS.md "A domain crate may depend on crates in the `infra` and `cross-cutting` tiers, plus other domain crates in the `domains` tier (only with explicit justification in an ADR)."
- **evidence:**
  ```rust
  // crates/domains/hr/src/aggregate.rs:38
  use educore_academic::{AcademicYearId, ClassId, SectionId, SubjectId};
  // crates/domains/library/src/aggregate.rs (similar)
  ```

---

## Phase B complete — 30 findings total

**Total findings: 30**

`docs/audit_reports/findings/wave1-hr-library.md` is final.

