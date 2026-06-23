# Spec Folder Audit Wave 6 — Specs Group 3

**Scope:** `docs/specs/finance/`, `docs/specs/hr/`, `docs/specs/library/`.

**11 spec files per folder (per `docs/code-standards.md` § "Spec folder layout"):**
`overview.md`, `aggregates.md`, `entities.md`, `value-objects.md`, `commands.md`, `events.md`, `services.md`, `permissions.md`, `repositories.md`, `workflows.md`, `tables.md`.

**File counts observed:**
- `finance/`: 11 files (complete)
- `hr/`: 11 files (complete)
- `library/`: 11 files (complete)

---

### FINDING 1

- **id:** SPEC-3-001
- **area:** spec
- **severity:** Critical
- **location:** `docs/specs/hr/overview.md:73`
- **description:** The HR overview uses the legacy `Sm_` brand prefix in prose. Per `AGENTS.md` § "Brand is Educore", legacy brand references are forbidden in new spec prose. The spec narrative refers to `SmAssignClassTeacher` instead of the engine's `AssignClassTeacher` aggregate defined in `docs/specs/hr/aggregates.md:308-337`.
- **expected:** All prose references in the HR spec use the engine's `AssignClassTeacher` aggregate name (no `Sm_` prefix).
- **evidence:** `docs/specs/hr/overview.md:73` `9. A `Staff` cannot be deleted while active` `   `SmAssignClassTeacher`, `LeaveRequest`, or `PayrollGenerate``. The HR aggregates file at `docs/specs/hr/aggregates.md:308` is `## AssignClassTeacher` (no `Sm_` prefix).

---

### FINDING 2

- **id:** SPEC-3-002
- **area:** spec
- **severity:** Critical
- **location:** `docs/specs/finance/entities.md:255`
- **description:** The finance entities file uses the legacy `Sm_` brand prefix in prose. The engine's HR-owned aggregate is `LeaveDeductionInfo` (per `docs/specs/hr/aggregates.md:476-505`); referring to it as `SmLeaveDeductionInfo` violates the no-legacy-brand rule in `AGENTS.md`.
- **expected:** All prose references in the finance entities file use the engine's `LeaveDeductionInfo` name (no `Sm_` prefix).
- **evidence:** `docs/specs/finance/entities.md:255` `**ActiveStatus**. This is the typed projection of the HR-owned` `   `SmLeaveDeductionInfo` row.` The HR aggregates file at `docs/specs/hr/aggregates.md:478` is `**Root type:** `LeaveDeductionInfo``.

---

### FINDING 3

- **id:** SPEC-3-003
- **area:** spec
- **severity:** Critical
- **location:** `docs/specs/hr/events.md:113-127` vs `docs/specs/academic/events.md:212`
- **description:** The `ClassTeacherAssigned` event is defined with two different payload shapes by the HR spec and the academic spec. The HR events file (`hr/events.md:116-122`) defines the event payload with `assign_class_teacher_id, class_id, section_id, staff_id, academic_id`; the academic events file (`academic/events.md:212`) defines it as `ClassTeacherAssigned { class_section_id, staff_id, role }`. Consumers subscribing to the event cannot reconcile the payload, and the cross-domain command composition breaks.
- **expected:** A single canonical `ClassTeacherAssigned` event payload is documented in both specs, or one domain owns the event and the other re-publishes a projection.
- **evidence:** `docs/specs/hr/events.md:116-122` `pub struct ClassTeacherAssigned {` `    pub assign_class_teacher_id: AssignClassTeacherId,` `    pub class_id: ClassId,` `    pub section_id: SectionId,` `    pub staff_id: StaffId,` `    pub academic_id: AcademicYearId,` `}`. `docs/specs/academic/events.md:212` `- `ClassTeacherAssigned { class_section_id, staff_id, role }``.

---

### FINDING 4

- **id:** SPEC-3-004
- **area:** spec
- **severity:** Critical
- **location:** `docs/specs/hr/events.md:127` vs `docs/specs/academic/events.md:213`
- **description:** The `SubjectTeacherAssigned` event is defined with two different payload shapes by the HR spec and the academic spec. The HR events file (`hr/events.md:127`) defines the event payload as `{ class_id, section_id, subject_id, staff_id, academic_id }`; the academic events file (`academic/events.md:213`) defines it as `{ class_section_id, subject_id, staff_id }`. The cross-domain event flow for `HR → academic` cannot reconcile the payload.
- **expected:** A single canonical `SubjectTeacherAssigned` event payload is documented in both specs.
- **evidence:** `docs/specs/hr/events.md:127` `- `SubjectTeacherAssigned { class_id, section_id, subject_id, staff_id, academic_id }``. `docs/specs/academic/events.md:213` `- `SubjectTeacherAssigned { class_section_id, subject_id, staff_id }``.

---

### FINDING 5

- **id:** SPEC-3-005
- **area:** spec
- **severity:** Critical
- **location:** `docs/specs/finance/events.md:468` vs `docs/specs/hr/events.md:242`
- **description:** The terminal `PayrollPaid` event is defined with two different field names by the finance and HR specs, despite being the cross-domain coordination event between the two domains. The finance events file (`finance/events.md:468`) defines the event as `{ payroll_generate_id, paid_amount, payment_date }`; the HR events file (`hr/events.md:242`) defines it as `{ payroll_generate_id, paid_amount, paid_at }`. The HR `MarkPayrollPaid` command (hr/commands.md:494-511) is the HR-side ack of this finance event, but it cannot bind to a single payload shape.
- **expected:** A single canonical `PayrollPaid` event payload is documented in both specs, using one timestamp field name (either `payment_date` or `paid_at`).
- **evidence:** `docs/specs/finance/events.md:468` `- `PayrollPaid { payroll_generate_id, paid_amount, payment_date }``. `docs/specs/hr/events.md:242` `- `PayrollPaid { payroll_generate_id, paid_amount, paid_at }``. The HR-side ack is at `docs/specs/hr/commands.md:511` `**Effects:** Emits `PayrollPaid`.`.

---

### FINDING 6

- **id:** SPEC-3-006
- **area:** spec
- **severity:** High
- **location:** `docs/specs/hr/permissions.md:14` vs `docs/specs/hr/permissions.md:37-39`
- **description:** The HR permissions file's "Naming" section lists `BulkImport.*` as the capability prefix used by the HR domain, but the actual capabilities listed under the `### Staff` section use `Staff.ImportBulk`, `Staff.ImportBulk.Promote`, and `Staff.ImportBulk.Reject`. The "Naming" block therefore misleads readers about the actual namespace the engine enforces, and no `BulkImport.*` capabilities are defined anywhere else in the HR spec.
- **expected:** The HR permissions "Naming" section lists the actual prefix (`Staff.ImportBulk.*`) used by the bulk-import commands, or the bulk-import capabilities are renamed to a `BulkImport.*` namespace consistently.
- **evidence:** `docs/specs/hr/permissions.md:14` ``StaffRegistrationField.*`, `BulkImport.*`,``. `docs/specs/hr/permissions.md:37-39` `- `Staff.ImportBulk``; `- `Staff.ImportBulk.Promote``; `- `Staff.ImportBulk.Reject``. The HR commands file at `docs/specs/hr/commands.md:658,676,693` uses the same `Staff.ImportBulk*` capabilities on the corresponding `ImportStaffBulkCommand`, `PromoteStaffImportCommand`, and `RejectStaffImportCommand`.

---

### FINDING 7

- **id:** SPEC-3-007
- **area:** spec
- **severity:** High
- **location:** `docs/specs/library/permissions.md:103-119` vs `docs/specs/library/events.md` (entire file)
- **description:** The library spec references a `FineWaived` event from three locations (`permissions.md:117`, `services.md:103`, plus the workflow at `workflows.md:128-130` which calls `WaiveBookIssueFine`), but the `FineWaived` event is not defined anywhere in `docs/specs/library/events.md`. The events file's only fine-related event is `FineCalculated` at line 223.
- **expected:** The library events file defines a `FineWaived` event with the same payload semantics implied by `BookIssueFine.Waived` (per `entities.md:50-51`) and the `FineCalculationService::apply_waiver` (per `services.md:80-83`).
- **evidence:** `docs/specs/library/permissions.md:117` `**Effects:** Emits `FineWaived` and updates the`. `docs/specs/library/services.md:103` `history entry and emits a `FineWaived` event.`. `docs/specs/library/events.md` defines `FineCalculated` at line 223 only; no `FineWaived` event is declared.

---

### FINDING 8

- **id:** SPEC-3-008
- **area:** spec
- **severity:** High
- **location:** `docs/specs/library/permissions.md:108-119` and `docs/specs/library/workflows.md:128-130` vs `docs/specs/library/commands.md` (entire file)
- **description:** The library spec references `WaiveBookIssueFineCommand` from three locations (`permissions.md:108-119`, `workflows.md:128-130`, and indirectly via the `BookIssue.WaiveFine` capability in `permissions.md:58,103`), but no `WaiveBookIssueFineCommand` struct is defined in `docs/specs/library/commands.md`. The library `BookIssue` aggregate (`aggregates.md:175`) lists only `IssueBook`, `ReturnBook`, `RenewBook`, `MarkBookLost`, `CalculateFine` as commands — no `WaiveBookIssueFine`.
- **expected:** The library commands file defines `WaiveBookIssueFineCommand` with a payload, pre-conditions, and `Capability: BookIssue.WaiveFine`, and the `BookIssue` aggregate in `aggregates.md` lists it under the aggregate's Commands section.
- **evidence:** `docs/specs/library/permissions.md:108-119` documents the full `pub struct WaiveBookIssueFineCommand { ... }` shape with `book_issue_fine_id: BookIssueFineId, reason: String` and the `BookIssue.WaiveFine` capability. `docs/specs/library/aggregates.md:175` lists the `BookIssue` aggregate's commands as `- `IssueBook``, `- `ReturnBook``, `- `RenewBook``, `- `MarkBookLost``, `- `CalculateFine``. `docs/specs/library/commands.md` has no `## WaiveBookIssueFine` or `### WaiveBookIssueFine` section.

---

### FINDING 9

- **id:** SPEC-3-009
- **area:** spec
- **severity:** High
- **location:** `docs/specs/library/commands.md:82,155` vs `docs/specs/library/value-objects.md` (entire file)
- **description:** The library commands reference `BookPatch` (`commands.md:82`, referenced in `commands.md:86`) and `LibraryMemberPatch` (`commands.md:155`, referenced in `commands.md:159`) but neither value object is defined in `docs/specs/library/value-objects.md`. The value-objects file's "Bibliographic", "Members", and "Issues" sections do not declare either patch type.
- **expected:** `BookPatch` and `LibraryMemberPatch` are declared in `library/value-objects.md` (or referenced from a shared patch crate) with the documented mutable fields (`book_title, publisher_name, author_name, rack_number, book_price, post_date, details, book_category_id, book_subject_id` for `BookPatch`; `member_ud_id, note` for `LibraryMemberPatch`).
- **evidence:** `docs/specs/library/commands.md:82` `    pub patch: BookPatch,`. `docs/specs/library/commands.md:86` `   `BookPatch` carries the mutable fields: `book_title`,`. `docs/specs/library/commands.md:155` `    pub patch: LibraryMemberPatch,`. `docs/specs/library/value-objects.md` lists value objects under "Bibliographic", "Members", "Issues", "Money & Quantities", "Status Enums", "Identity & Contact" — no `BookPatch` or `LibraryMemberPatch` row.

---

### FINDING 10

- **id:** SPEC-3-010
- **area:** spec
- **severity:** High
- **location:** `docs/specs/finance/commands.md:151,474,687,914` and `docs/specs/finance/services.md:32-44,137-143` vs `docs/specs/finance/value-objects.md` (entire file)
- **description:** The finance spec references seven value-object types that are not declared in `docs/specs/finance/value-objects.md`: `CloseReason` (commands.md:151), `ReferenceId` (commands.md:474,687), `ChartAccountType` (commands.md:914), `FmFeesInvoiceDraft` (services.md:32,39,41,42,43,44), `ReconciliationMatch` (services.md:137,143), `ReconciliationReport` (services.md:138). Consumers implementing the spec cannot construct these types because no definition (constraints, fields) is given.
- **expected:** Each of the seven types is declared in `finance/value-objects.md` with its constraints or enum variants, or a documented dependency points to a cross-domain value object.
- **evidence:** `docs/specs/finance/commands.md:151` `    pub reason: CloseReason,`. `docs/specs/finance/commands.md:474` `    pub reference_id: Option<ReferenceId>,`. `docs/specs/finance/commands.md:687` `    pub reference_id: Option<ReferenceId>,`. `docs/specs/finance/commands.md:914` `    pub account_type: ChartAccountType, // asset, liability, income, expense, equity`. `docs/specs/finance/services.md:32` `    ) -> Vec<FmFeesInvoiceDraft> { ... }`. `docs/specs/finance/services.md:137` `    pub fn match_statement(stmt: &BankStatement, payments: &[FeesPayment], slips: &[BankPaymentSlip]) -> ReconciliationMatch { ... }`. `docs/specs/finance/services.md:138` `    pub fn build_reconciliation_report(school: SchoolId, from: NaiveDate, to: NaiveDate) -> ReconciliationReport { ... }`. None of these types appear in `finance/value-objects.md`'s "Identifiers", "Money", "Percentages & Rounding", "Invoice & Receipt", "Payment Status", "Payment Method", "Bank", "Discount", "Payroll", "Carry Forward", "Reminder", "Login Prevention", "Installment Credit", "Question Bank Fees", "Status Enums", "Donor", "Inventory", "Bank Statement", or "School Identity Bindings" tables.

---

### FINDING 11

- **id:** SPEC-3-011
- **area:** spec
- **severity:** High
- **location:** `docs/specs/finance/services.md:43` vs `docs/specs/finance/value-objects.md:250`
- **description:** `InvoiceNumberingService::number_invoice` (services.md:43) takes a `school: &School` parameter, but `School` is not a defined value object. The only school-typed binding declared in the finance value-objects file is `SchoolId` (line 250, "From `educore-platform`"). The signature mixes the wrong type, so the service is inconsistent with the rest of the spec which uses `SchoolId`.
- **expected:** `number_invoice` takes `school: &SchoolContext` or `school_id: SchoolId`, matching the rest of the finance spec's school typing.
- **evidence:** `docs/specs/finance/services.md:43` `    pub fn number_invoice(school: &School, draft: &FmFeesInvoiceDraft) -> InvoiceNumber { ... }`. `docs/specs/finance/value-objects.md:250` `| `SchoolId`            | From `educore-platform`                                     |`. The rest of the finance services file uses `SchoolId`: `services.md:138` `pub fn build_reconciliation_report(school: SchoolId, from: NaiveDate, to: NaiveDate) -> ReconciliationReport`.

---

### FINDING 12

- **id:** SPEC-3-012
- **area:** spec
- **severity:** High
- **location:** `docs/specs/hr/commands.md:82,443-444,654,397` and `docs/specs/hr/services.md:14,93-95,145-146` vs `docs/specs/hr/value-objects.md` (entire file)
- **description:** The HR spec references five value-object types that are not declared in `docs/specs/hr/value-objects.md`: `StaffProfilePatch` (commands.md:82, services.md:14), `PayrollEarningLine` and `PayrollDeductionLine` (commands.md:443,444), `StaffImportRow` (commands.md:654, services.md:145,146), `StaffAttendanceImportRow` (commands.md:397, services.md:93-95). The value-objects file's tables ("Identifiers", "Names & Identity", "Salary & Money", "Leave", "Attendance", "Dates", "Status Enums", "School Identity Bindings") declare no such patch or row types.
- **expected:** Each of the five types is declared in `hr/value-objects.md` with its fields (or referenced from a cross-crate value-object catalog) so the HR command/service signatures can be satisfied.
- **evidence:** `docs/specs/hr/commands.md:82` `    pub patch: StaffProfilePatch,`. `docs/specs/hr/commands.md:443-444` `    pub earnings: Vec<PayrollEarningLine>,` `    pub deductions: Vec<PayrollDeductionLine>,`. `docs/specs/hr/commands.md:397` `    pub rows: Vec<StaffAttendanceImportRow>,`. `docs/specs/hr/commands.md:654` `    pub rows: Vec<StaffImportRow>,`. `docs/specs/hr/services.md:14` `    pub fn apply_patch(staff: &mut Staff, patch: StaffProfilePatch) -> Result<(), ValidationError> { ... }`. `docs/specs/hr/services.md:93-95` `    pub fn parse_csv(rows: Vec<Vec<String>>) -> Vec<StaffAttendanceImportRow> { ... }` `    pub fn validate(row: &StaffAttendanceImportRow) -> Result<(), ValidationError> { ... }` `    pub fn dedupe(rows: Vec<StaffAttendanceImportRow>) -> Vec<StaffAttendanceImportRow> { ... }`. `docs/specs/hr/services.md:145-146` `    pub fn validate_row(row: &StaffImportRow) -> Result<(), ValidationError> { ... }` `    pub fn normalize(row: &StaffImportRow) -> StaffImportRow { ... }`. None of `StaffProfilePatch`, `PayrollEarningLine`, `PayrollDeductionLine`, `StaffImportRow`, `StaffAttendanceImportRow` appear in `hr/value-objects.md`.

---

### FINDING 13

- **id:** SPEC-3-013
- **area:** spec
- **severity:** High
- **location:** `docs/specs/finance/overview.md:120,150` vs `docs/specs/finance/aggregates.md:671-719`
- **description:** The finance overview's "Aggregate Roots" table lists `FeesInvoiceSetting` twice (lines 120 and 150). The aggregates file at `finance/aggregates.md:671-696` defines a single `FeesInvoiceSetting` aggregate; the duplicate row in the overview (line 150, with the annotation `(above, listed again)`) is a documentation bug that inflates the aggregate count and risks double-listing in any cross-spec consistency check.
- **expected:** The "Aggregate Roots" table in `finance/overview.md` lists each of the 51 unique aggregates exactly once.
- **evidence:** `docs/specs/finance/overview.md:120` `| FeesInvoiceSetting              | `FeesInvoiceSetting`      | Classic invoice layout settings                      |`. `docs/specs/finance/overview.md:150` `| FeesInvoiceSetting              | `FeesInvoiceSetting`      | (above, listed again)                                |`. The aggregates file at `docs/specs/finance/aggregates.md:671-696` has only one `## FeesInvoiceSetting` heading; the duplicate row at line 150 contradicts the actual aggregate count of 51 unique roots in `aggregates.md` (verified by `grep -c '^## ' finance/aggregates.md` = 51).

---

### FINDING 14

- **id:** SPEC-3-014
- **area:** spec
- **severity:** High
- **location:** `docs/specs/finance/tables.md:91`
- **description:** The finance tables file's closing note claims "Total finance tables: 52 (one per aggregate; see Coverage Matrix in build-plan.md)". The actual table inventory in `finance/tables.md` lists only 39 distinct `hr_*` and `finance_*` tables (verified by `grep -cE '^\| `(hr_|finance_)[a-z_0-9]+`' finance/tables.md` = 39). The aggregate count in `finance/aggregates.md` is 51 unique aggregates. Both 39 and 51 contradict the "52 tables" claim.
- **expected:** The closing note either matches the actual count or is removed if no precise per-aggregate table mapping exists.
- **evidence:** `docs/specs/finance/tables.md:91` `**Total finance tables: 52 (one per aggregate; see Coverage Matrix in build-plan.md).**`. `docs/specs/finance/tables.md:10-58` lists 49 table rows total (mix of `finance_*` and `hr_*` engine tables plus cross-domain `assessment_*` and `chart_of_accounts` rows); only 39 of them have an `hr_` or `finance_` engine-owned prefix. The aggregate file at `docs/specs/finance/aggregates.md` has 51 unique `## Aggregate` headings.

---

### FINDING 15

- **id:** SPEC-3-015
- **area:** spec
- **severity:** High
- **location:** `docs/specs/finance/tables.md:22` vs `docs/specs/finance/aggregates.md:618-640`
- **description:** The finance tables file lists the child-transaction table as `finance_fees_transcation_children` (line 22, with the word "transcation" instead of "transaction"). The corresponding aggregate at `finance/aggregates.md:618-640` is `FmFeesTransactionChild` (correct spelling). The migration's table name must match the aggregate's underlying storage name.
- **expected:** The table name in `finance/tables.md` matches the aggregate spelling: `finance_fees_transaction_children` (or whatever the engine's canonical name is).
- **evidence:** `docs/specs/finance/tables.md:22` `| `finance_fees_transcation_children`             | FmFeesTransactionChild                   | Newer FM transaction line                    |`. `docs/specs/finance/aggregates.md:619-620` `## FmFeesTransactionChild` `**Identity:** `FmFeesTransactionChildId(SchoolId, Uuid)``. `docs/specs/finance/aggregates.md:630` `2. Belongs to one `FmFeesTransaction`.` — confirming the canonical name is `transaction`, not `transcation`.

---

### FINDING 16

- **id:** SPEC-3-016
- **area:** spec
- **severity:** High
- **location:** `docs/specs/hr/repositories.md:272,275` vs `docs/specs/hr/aggregates.md:242-272`
- **description:** The HR repositories file's recommended PostgreSQL indexes contain typos in two index names: line 272 names the index `ux_hr_staff_attendences_school_id_staff_date` (with `attendences` instead of `attendances`), and line 275 references the column `attendence_date` (also missing one `a`). The corresponding HR aggregate at `aggregates.md:255` and the column at `tables.md:16` are spelled `StaffAttendance` and `attendance_date`. Migrations generated from these index names will not match the engine's actual storage column.
- **expected:** The index names and column references in `hr/repositories.md` use the canonical spelling: `attendance` (with two `a`s between `tend` and `nce`).
- **evidence:** `docs/specs/hr/repositories.md:272` `CREATE UNIQUE INDEX ux_hr_staff_attendences_school_id_staff_date ON hr_staff_attendances (school_id, staff_id, attendance_date);`. `docs/specs/hr/repositories.md:275` `CREATE INDEX ix_attendance_staff_attendance_imports_school_id_date ON attendance_staff_attendance_imports (school_id, attendence_date);`. `docs/specs/hr/aggregates.md:245` `**Root type:** `StaffAttendance``. `docs/specs/hr/aggregates.md:258` `   `attendance_date` is required.`. `docs/specs/hr/tables.md:16` `| `hr_staff_attendances`             | StaffAttendance           | A daily attendance row               |`.

---

### FINDING 17

- **id:** SPEC-3-017
- **area:** spec
- **severity:** High
- **location:** `docs/specs/finance/permissions.md:119` vs `docs/specs/finance/commands.md` (entire file)
- **description:** The finance permissions file advertises the `Bank.Statement.Reverse` capability (line 119), but the corresponding `ReverseBankStatementCommand` struct is not defined in `docs/specs/finance/commands.md`. The capability exists only as a referenced capability without a command implementation. The `BankStatement` aggregate at `finance/aggregates.md:802-806` lists the command name, but no struct is provided.
- **expected:** Either the `Bank.Statement.Reverse` capability is removed from `permissions.md`, or `ReverseBankStatementCommand` is defined in `commands.md` with a payload, pre-conditions, and the `Bank.Statement.Reverse` capability tag (matching the `RecordBankStatementCommand` shape at `commands.md:465-481`).
- **evidence:** `docs/specs/finance/permissions.md:119` `- `Bank.Statement.Reverse``. `docs/specs/finance/aggregates.md:805` `- `ReverseBankStatement``. `docs/specs/finance/commands.md` defines `RecordBankStatement` (line 463) but no `ReverseBankStatement` or `ReverseBankStatementCommand` section.

---

### FINDING 18

- **id:** SPEC-3-018
- **area:** spec
- **severity:** High
- **location:** `docs/specs/finance/permissions.md:189` vs `docs/specs/finance/commands.md` (entire file)
- **description:** The finance permissions file advertises the `QuestionBank.Fee.Detach` capability (line 189), and the `QuestionBankFee` aggregate at `finance/aggregates.md:1333` lists `DetachFeesFromQuestionBank` as a command, but the corresponding `DetachFeesFromQuestionBankCommand` struct is not defined in `docs/specs/finance/commands.md`. The mirror command `AttachFeesToQuestionBankCommand` is defined at `commands.md:891-905`, so the asymmetry between attach and detach implementations is also a gap.
- **expected:** `DetachFeesFromQuestionBankCommand` is defined in `commands.md` symmetric to `AttachFeesToQuestionBankCommand`, with the `QuestionBank.Fee.Detach` capability and the `FeesDetachedFromQuestionBank` event effect.
- **evidence:** `docs/specs/finance/permissions.md:189` `- `QuestionBank.Fee.Detach``. `docs/specs/finance/aggregates.md:1333` `- `DetachFeesFromQuestionBank``. `docs/specs/finance/commands.md` defines `AttachFeesToQuestionBankCommand` at lines 891-905 but no `DetachFeesFromQuestionBank` section.

---

### FINDING 19

- **id:** SPEC-3-019
- **area:** spec
- **severity:** High
- **location:** `docs/specs/finance/commands.md` (FM section absent) vs `docs/specs/finance/aggregates.md:524-745`
- **description:** The finance commands file does not document the 11 FM-invoice-scheme commands listed in `finance/aggregates.md`: `GenerateFmFeesInvoice`, `UpdateFmFeesInvoiceStatus`, `CancelFmFeesInvoice`, `AddFmFeesInvoiceLine`, `UpdateFmFeesInvoiceLine`, `RemoveFmFeesInvoiceLine`, `RecordFmFeesTransaction`, `ReverseFmFeesTransaction`, `AddFmFeesTransactionLine`, `ApplyFmFeesWeaver`, `ReverseFmFeesWeaver` (plus the FM group/type/settings CRUD commands `CreateFmFeesGroup`, `UpdateFmFeesGroup`, `DeleteFmFeesGroup`, `CreateFmFeesType`, `UpdateFmFeesType`, `DeleteFmFeesType`, `ConfigureFmFeesInvoiceSetting`, `ConfigureFeesInvoiceSetting`, `ConfigureInvoiceSetting`). These aggregates have full event, repository, and service coverage but no command struct definitions, breaking the spec's completeness.
- **expected:** Each FM-scheme aggregate in `finance/aggregates.md` has a matching command struct in `finance/commands.md` with a payload, pre-conditions, capability, and effects.
- **evidence:** `docs/specs/finance/aggregates.md:544-546` lists `GenerateFmFeesInvoice`, `UpdateFmFeesInvoiceStatus`, `CancelFmFeesInvoice` under the `FmFeesInvoice` aggregate. `docs/specs/finance/aggregates.md:575-577` lists `AddFmFeesInvoiceLine`, `UpdateFmFeesInvoiceLine`, `RemoveFmFeesInvoiceLine` under `FmFeesInvoiceChild`. `docs/specs/finance/aggregates.md:608-609` lists `RecordFmFeesTransaction`, `ReverseFmFeesTransaction` under `FmFeesTransaction`. `docs/specs/finance/aggregates.md:635` lists `AddFmFeesTransactionLine` under `FmFeesTransactionChild`. `docs/specs/finance/aggregates.md:661-662` lists `ApplyFmFeesWeaver`, `ReverseFmFeesWeaver` under `FmFeesWeaver`. `docs/specs/finance/aggregates.md:482-484,512-514` list FM group/type CRUD commands. `docs/specs/finance/commands.md` contains no `### GenerateFmFeesInvoice`, `### AddFmFeesInvoiceLine`, `### RecordFmFeesTransaction`, etc. sections. The only FM-adjacent section in commands.md is `### ConfigureInvoiceSettings` (line 816), which emits `InvoiceSettingConfigured` or `FeesInvoiceSettingConfigured` (commands.md:845-846) — covering only the settings aggregates.

---

### FINDING 20

- **id:** SPEC-3-020
- **area:** spec
- **severity:** High
- **location:** `docs/specs/finance/commands.md` (multiple aggregates) vs `docs/specs/finance/aggregates.md` (CRUD commands)
- **description:** Beyond the FM scheme (covered by SPEC-3-019), the finance commands file omits update and delete variants for many aggregates. The aggregates file lists `UpdateFeesType`/`DeleteFeesType` (line 63-64), `UpdateFeesDiscount`/`DeleteFeesDiscount` (line 203-204), `UpdateFeesInstallment`/`DeleteFeesInstallment` (line 266-267), `UpdateDirectFeesInstallment`/`DeleteDirectFeesInstallment` (line 396-397), `UpdateBankAccount`/`CloseBankAccount` (line 770-771), `UpdateExpense`/`DeleteExpense` (line 869-870), `UpdateIncome`/`DeleteIncome` (line 901-902), `UpdateChartOfAccount`/`DeleteChartOfAccount` (line 1303-1304), `UpdatePaymentMethod`/`DeletePaymentMethod` (line 1395-1396), `UpdateFeesReminder`/`DeleteFeesReminder` (line 1423-1424), `UpdateSalaryTemplate`/`DeleteSalaryTemplate` (line 1187-1188), `UpdateHourlyRate`/`DeleteHourlyRate` (line 357-358), `UpdatePayrollEarnDeduc`/`DeletePayrollEarnDeduc` (line 1154-1155), and others — but the commands file provides struct definitions only for a subset (`CreateExpenseHead / UpdateExpenseHead / DeleteExpenseHead` at line 607 is abbreviated and lacks struct bodies; `RegisterDonor / UpdateDonor / DeleteDonor` at line 622 is also abbreviated).
- **expected:** Each update/delete command listed in `aggregates.md` has a struct definition in `commands.md`, even if abbreviated, with a payload type, capability, and effects line.
- **evidence:** `docs/specs/finance/aggregates.md:63-64` `- `UpdateFeesType``; `- `DeleteFeesType``. `docs/specs/finance/commands.md:38-51` defines only `### CreateFeesType` (line 38) and no `### UpdateFeesType` or `### DeleteFeesType` sections. Same pattern for `FeesDiscount`, `FeesInstallment`, `DirectFeesInstallment`, `BankAccount`, `Expense`, `Income`, `Donor` (abbreviated), `ChartOfAccount` (abbreviated), `PaymentMethod` (abbreviated), `DirectFeesReminder`, `SalaryTemplate`, `HourlyRate`, `PayrollEarnDeduc`.

---

### FINDING 21

- **id:** SPEC-3-021
- **area:** spec
- **severity:** High
- **location:** `docs/specs/library/events.md:113` and `docs/specs/library/commands.md:133,211` vs `docs/specs/library/tables.md:34,57,69,85`
- **description:** The library events and commands use the field name `academic_year_id`, but the library tables file uses the column name `academic_id` in the field-mapping tables. The mismatch is internal to the library spec; for example `LibraryMemberRegistered` (`events.md:113`) carries `pub academic_year_id: AcademicYearId`, but the storage column at `tables.md:69` is `academic_id`. The HR spec (`hr/tables.md`) consistently uses `academic_id`, so consumers cannot rely on a single naming convention.
- **expected:** The library spec consistently uses one column name (either `academic_id` or `academic_year_id`) across events, commands, and the tables file.
- **evidence:** `docs/specs/library/events.md:113` `    pub academic_year_id: AcademicYearId,`. `docs/specs/library/commands.md:133,211` `    pub academic_year_id: AcademicYearId,`. `docs/specs/library/tables.md:34` `| `academic_id`     | `u64`               | `AcademicYearId`                     |`. `docs/specs/library/tables.md:57` `| `academic_id`       | `u64`               | `AcademicYearId`                     |`. `docs/specs/library/tables.md:69` `| `academic_id`         | `u64`               | `AcademicYearId`                     |`. `docs/specs/library/tables.md:85` `| `academic_id`       | `u64`               | `AcademicYearId`                     |`. HR tables at `docs/specs/hr/tables.md:35` use `academic_id` uniformly.

---

### FINDING 22

- **id:** SPEC-3-022
- **area:** spec
- **severity:** Medium
- **location:** `docs/specs/library/aggregates.md:99-101` vs `docs/specs/library/value-objects.md:56-58`
- **description:** The library aggregates file uses the term `StudentStaffId` (line 99) for the polymorphic member reference, but the library value-objects file uses the term `MemberId` for the same concept (line 56: `enum `Student(StudentId)` or `Staff(StaffId)`). The aggregates file's later description (line 106) does acknowledge the storage column `student_staff_id`, but the domain-level term is inconsistent between the two files.
- **expected:** The library spec uses one domain-level term (`MemberId` is the value-object name; the aggregates file should refer to it consistently) and one storage column name (`student_staff_id`).
- **evidence:** `docs/specs/library/aggregates.md:99-101` `A registered borrower. May be a student or a staff member. Each` `member has a `MemberType` (from the role catalog) and a` `   `StudentStaffId` (the underlying user id from the platform).`. `docs/specs/library/value-objects.md:56` `| `MemberId`        | enum `Student(StudentId)` or `Staff(StaffId)`            |`. `docs/specs/library/value-objects.md:67` `| `student_staff_id`  | `u64`               | `MemberId` (StudentId or StaffId)    |`. The aggregates file uses `StudentStaffId` at line 99 (no entry in value-objects.md under that name) and `student_staff_id` at line 106.

---

### FINDING 23

- **id:** SPEC-3-023
- **area:** spec
- **severity:** Medium
- **location:** `docs/specs/hr/permissions.md:40-41` vs `docs/specs/hr/commands.md` (entire file) and `docs/specs/hr/aggregates.md:42-69`
- **description:** The HR permissions file lists `Staff.Document.Upload` and `Staff.Document.Download` capabilities (lines 40-41) but the HR commands file contains no `UploadStaffDocumentCommand` or `DownloadStaffDocumentCommand` struct. The `Staff` aggregate at `aggregates.md:42-69` lists only `RegisterStaff`, `UpdateStaff`, role-change, status-change, and `DeleteStaff` commands. The `StaffDocument` entity at `entities.md:42-50` is documented as a sub-entity but has no command surface.
- **expected:** Either the `Staff.Document.Upload` and `Staff.Document.Download` capabilities are removed from `permissions.md`, or matching `UploadStaffDocumentCommand` and `DownloadStaffDocumentCommand` structs are added to `commands.md` (with the `StaffDocument` aggregate or the `Staff` aggregate as the owner).
- **evidence:** `docs/specs/hr/permissions.md:40-41` `- `Staff.Document.Upload``; `- `Staff.Document.Download``. `docs/specs/hr/aggregates.md:42-54` lists the `Staff` aggregate's commands (no upload/download). `docs/specs/hr/commands.md` has no `### UploadStaffDocument` or `### DownloadStaffDocument` section. The `StaffDocument` entity is defined at `docs/specs/hr/entities.md:42-50` but is not an aggregate, so a command must attach to an aggregate root.

---

### FINDING 24

- **id:** SPEC-3-024
- **area:** spec
- **severity:** Medium
- **location:** `docs/specs/hr/permissions.md:34-36` vs `docs/specs/hr/commands.md:209-239`
- **description:** The HR permissions file uses three sub-namespaced capabilities (`Staff.AssignClassTeacher.Create`, `Staff.AssignClassTeacher.Update`, `Staff.AssignClassTeacher.Delete` at lines 34-36), but the corresponding command in `commands.md` is a single `AssignClassTeacherCommand` (line 210) and a single `UpdateAssignClassTeacherCommand` (line 229); there is no `DeleteAssignClassTeacherCommand` struct in `commands.md` even though the `AssignClassTeacher` aggregate at `aggregates.md:329` lists `DeleteAssignClassTeacher` as a command. The capability namespace implies three discrete commands but the commands file merges them or omits them.
- **expected:** The HR permissions file lists either the merged `Staff.AssignClassTeacher` capability (matching the actual commands) or the three sub-namespaced capabilities match three distinct command structs (Create/Update/Delete).
- **evidence:** `docs/specs/hr/permissions.md:34-36` `- `Staff.AssignClassTeacher.Create``; `- `Staff.AssignClassTeacher.Update``; `- `Staff.AssignClassTeacher.Delete``. `docs/specs/hr/commands.md:209-239` defines `AssignClassTeacherCommand` (line 210) and `UpdateAssignClassTeacherCommand` (line 229) only. `docs/specs/hr/aggregates.md:329` `- `DeleteAssignClassTeacher`` has no matching struct in `commands.md`.

---

### FINDING 25

- **id:** SPEC-3-025
- **area:** spec
- **severity:** Medium
- **location:** `docs/specs/finance/value-objects.md:73-89` vs `docs/specs/finance/aggregates.md:749-811` (Bank section)
- **description:** The finance value-objects file declares `BankAccountNumber` as `6..34 chars, alphanumeric` (line 142), `IfscCode` as `11 chars, format `[A-Z]{4}0[A-Z0-9]{6}`` (line 143), and `ChequeNumber` as `6 digits` (line 144), but these value objects are not declared on the `OpenBankAccountCommand` (commands.md:445-461) or `BankPaymentSlip`-related commands (commands.md:482-537). The `BankAccount` aggregate at `aggregates.md:755-758` lists only `bank_name`, `account_name`, `account_number`, `account_type`, `opening_balance`, and `note` as fields, with no reference to `IfscCode` or `ChequeNumber`. The IFSC and cheque-number constraints therefore cannot be enforced on `BankPaymentSlip` (aggregates.md:825: only `payment_mode` is enumerated as `Bk` or `Cq`).
- **expected:** Either `IfscCode` and `ChequeNumber` are used in the bank commands/aggregates (with validation), or they are removed from `value-objects.md` if they are not first-class concerns of the finance domain.
- **evidence:** `docs/specs/finance/value-objects.md:142-144` `| `BankAccountNumber`  | 6..34 chars, alphanumeric                                         |` `| `IfscCode`           | 11 chars, format `[A-Z]{4}0[A-Z0-9]{6}``                           |` `| `ChequeNumber`       | 6 digits                                                          |`. `docs/specs/finance/commands.md:445-461` `pub struct OpenBankAccountCommand { ... pub bank_name: String, ... pub account_number: BankAccountNumber, pub account_type: AccountType, ... }` (no `IfscCode`, no `ChequeNumber`). `docs/specs/finance/aggregates.md:825-826` `1. `payment_mode` is `Bk` (bank transfer) or `Cq` (cheque).` (no `ChequeNumber` validation).

---

### FINDING 26

- **id:** SPEC-3-026
- **area:** spec
- **severity:** Medium
- **location:** `docs/specs/finance/aggregates.md:1140-1147` vs `docs/specs/hr/aggregates.md:451-456`
- **description:** The `PayrollEarnDeduc` aggregate is declared in both the finance aggregates file (lines 1132-1163, "typed here") and the HR aggregates file (lines 443-473) with conflicting ownership semantics. The finance aggregates file declares it as a finance-owned aggregate (`**Root type:** `PayrollEarnDeduc`` at line 1134), while the HR aggregates file declares it as an HR-owned aggregate (`**Root type:** `PayrollEarnDeduc`` at line 445). The finance overview at line 135 says it is "typed here" (in finance), but the HR overview at line 42 says the cross-domain bridge "happens through the `PayrollGenerate` and `PayrollEarnDeduc` aggregates (HR-owned writes; finance reads and pays)". The ownership is contradictory.
- **expected:** A single canonical owner for `PayrollEarnDeduc` is declared in both spec files. Per the cross-domain narrative (HR-owned writes, finance reads), the HR aggregates file should be the canonical owner, and the finance aggregates file should cross-reference it as a read-only view.
- **evidence:** `docs/specs/finance/aggregates.md:1132-1138` `## PayrollEarnDeduc` `**Root type:** `PayrollEarnDeduc`` `**Identity:** `PayrollEarnDeducId(SchoolId, Uuid)``. `docs/specs/finance/aggregates.md:1153-1156` `- `AddPayrollEarning``; `- `AddPayrollDeduction``; `- `UpdatePayrollEarnDeduc``; `- `DeletePayrollEarnDeduc``. `docs/specs/hr/aggregates.md:443-472` `## PayrollEarnDeduc` `**Root type:** `PayrollEarnDeduc`` with the same commands (`AddPayrollEarning`, etc.) and events (`PayrollEarningAdded`, etc.). `docs/specs/hr/overview.md:42-43` `finance happens through the `PayrollGenerate` and `PayrollEarnDeduc`` `aggregates (HR-owned writes; finance reads and pays).`.

---

### FINDING 27

- **id:** SPEC-3-027
- **area:** spec
- **severity:** Medium
- **location:** `docs/specs/finance/commands.md:698-718` vs `docs/specs/hr/commands.md:436-456`
- **description:** The `GeneratePayrollCommand` struct is defined in both the finance commands file (lines 698-718, marked `(HR)` in comments at aggregates.md:1118-1123) and the HR commands file (lines 436-456). Both definitions declare identical fields (`tenant`, `staff_id`, `pay_period`, `salary_template_id`, `earnings`, `deductions`, `note`, `bank_id`, `payment_mode`) but differ in ordering and in field names (`bank_id: Option<BankAccountId>` vs the same in both, but the HR version adds no extra fields). The duplication means consumers must reconcile which is the canonical struct.
- **expected:** `GeneratePayrollCommand` is defined once (in the HR commands file as the HR-owned write command, per the cross-domain narrative) and the finance commands file references it as the source-of-truth HR-side command (or defines a finance-side `PayPayrollCommand` for the disbursement only).
- **evidence:** `docs/specs/finance/commands.md:700-709` `pub struct GeneratePayrollCommand {` `    pub tenant: TenantContext,` `    pub staff_id: StaffId,` `    pub pay_period: PayPeriod,` `    pub salary_template_id: Option<SalaryTemplateId>,` `    pub earnings: Vec<PayrollEarningLine>,` `    pub deductions: Vec<PayrollDeductionLine>,` `    pub note: Option<String>,` `    pub bank_id: Option<BankAccountId>,` `    pub payment_mode: Option<PaymentMethodId>,` `}`. `docs/specs/hr/commands.md:439-448` `pub struct GeneratePayrollCommand {` `    pub tenant: TenantContext,` `    pub staff_id: StaffId,` `    pub pay_period: PayPeriod,` `    pub salary_template_id: Option<SalaryTemplateId>,` `    pub earnings: Vec<PayrollEarningLine>,` `    pub deductions: Vec<PayrollDeductionLine>,` `    pub note: Option<String>,` `    pub bank_id: Option<BankAccountId>,` `    pub payment_mode: Option<PaymentMethodId>,` `}`. Identical struct shapes in both spec files.

---

### FINDING 28

- **id:** SPEC-3-028
- **area:** spec
- **severity:** Medium
- **location:** `docs/specs/finance/aggregates.md:1296-1300` vs `docs/specs/finance/value-objects.md:215`
- **description:** The `ChartOfAccount` aggregate at `finance/aggregates.md:1294` declares `account_type` as a value with five variants (`asset, liability, income, expense, equity` per the comment at `commands.md:914`), but the finance value-objects file does not declare a `ChartAccountType` enum (verified by `grep -n "ChartAccountType" finance/value-objects.md` returning no matches). The only related enum at `value-objects.md:215` is `AccountDirection` (`Debit`, `Credit`). Without a declared `ChartAccountType`, the engine cannot validate the `account_type` field on `ChartOfAccount` creation.
- **expected:** `ChartAccountType` is declared in `finance/value-objects.md` with its five variants and constraints (e.g. distinctness, naming rules), or the value-object comment in the command is updated to reference the existing `AccountDirection`.
- **evidence:** `docs/specs/finance/commands.md:914` `    pub account_type: ChartAccountType, // asset, liability, income, expense, equity`. `docs/specs/finance/aggregates.md:1296-1300` `1. Each `ChartOfAccount` is unique by `name` within a school.` `2. A `ChartOfAccount` cannot be deleted while any `Expense`,` `   `Income`, or `BankStatement` references it.`. `docs/specs/finance/value-objects.md:215` `| `AccountDirection`   | `Debit`, `Credit`                                                 |` — only `AccountDirection`, no `ChartAccountType`.

---

### FINDING 29

- **id:** SPEC-3-029
- **area:** spec
- **severity:** Low
- **location:** `docs/specs/finance/events.md:229-249` vs `docs/specs/finance/aggregates.md:524-553`
- **description:** The FM-invoice-scheme events file (`finance/events.md:238-249`) lists 14 FM events (`FinanceInvoiceStatusUpdated`, `FinanceInvoiceCancelled`, `FinanceInvoiceLineAdded`, `FinanceInvoiceLineUpdated`, `FinanceInvoiceLineRemoved`, `FinanceTransactionRecorded`, `FmFeesTransactionReversed`, `FmFeesTransactionLineAdded`, `FinanceWeaverApplied`, `FinanceWeaverReversed`, `FinanceFeesGroupCreated`/`Updated`/`Deleted`, `FinanceFeesTypeCreated`/`Updated`/`Deleted`), but the corresponding FM aggregates (`FmFeesInvoice`, `FmFeesInvoiceChild`, `FmFeesTransaction`, `FmFeesTransactionChild`, `FmFeesWeaver`, `FmFeesGroup`, `FmFeesType`) at `aggregates.md:524-521` list the event names with a different prefix (`FmFeesInvoiceGenerated` vs the listed `FmFeesInvoiceGenerated`; the events file uses `FinanceInvoice*` and `FinanceTransaction*` prefixes, while the aggregates file uses `FmFees*`). The same logical event is documented under two prefixes.
- **expected:** The FM-scheme events use one prefix consistently — either `Fm*` (matching the aggregate naming) or `Finance*` (matching the events-file shorthand).
- **evidence:** `docs/specs/finance/events.md:238-244` `- `FinanceInvoiceStatusUpdated { finance_invoice_id, status }``; `- `FinanceInvoiceCancelled { finance_invoice_id, reason }``; `- `FinanceInvoiceLineAdded { finance_invoice_id, line_id, fees_type, amount }``; `- `FinanceInvoiceLineUpdated { finance_invoice_id, line_id, changes }``; `- `FinanceInvoiceLineRemoved { finance_invoice_id, line_id }``; `- `FinanceTransactionRecorded { finance_transaction_id, finance_invoice_id, payment_method, total_paid_amount, add_wallet_money }``; `- `FmFeesTransactionReversed { finance_transaction_id, reason }``. `docs/specs/finance/aggregates.md:550-552` lists `FmFeesInvoiceGenerated`, `FinanceInvoiceStatusUpdated`, `FinanceInvoiceCancelled` under the `FmFeesInvoice` aggregate — mixing both prefixes within a single aggregate.

---

### FINDING 30

- **id:** SPEC-3-030
- **area:** spec
- **severity:** Low
- **location:** `docs/specs/library/overview.md:38` vs `docs/specs/library/value-objects.md:34-35`
- **description:** The library overview states the domain depends on `StudentId`, `StaffId`, `RoleId`, `AcademicYearId` from the academic/HR/RBAC domains, but the overview does not list the engine-internal identifier types `BookId`, `BookCategoryId`, `LibraryMemberId`, `BookIssueId` that the spec promises the library domain exposes to consumers (per the same overview at lines 39-41). Consumers of the library SDK cannot determine the canonical exported identifier names without cross-referencing `value-objects.md:14-23`.
- **expected:** The library overview's "Dependencies" section lists both the inbound cross-domain identifiers AND the library's own identifier types that are exposed to consumers.
- **evidence:** `docs/specs/library/overview.md:36-41` `The library domain **does** depend on identifier types defined by` `the academic and human-resource domains: `StudentId`, `StaffId`,`` `   `RoleId`, `AcademicYearId`. It exposes its own identifier types to` `   consumers: `BookId`, `BookCategoryId`, `LibraryMemberId`,`` `   `BookIssueId`.`. `docs/specs/library/overview.md:42-54` (Dependencies section) lists `educore-core`, `educore-platform`, `educore-rbac`, `educore-events`, `educore-academic`, `educore-hr`, `educore-finance`, but no `BookId`/`BookCategoryId`/`LibraryMemberId`/`BookIssueId` reference. `docs/specs/library/value-objects.md:14-23` lists these four identifiers in the "Identifiers" table.

---

### FINDING 31

- **id:** SPEC-3-031
- **area:** spec
- **severity:** Low
- **location:** `docs/specs/hr/overview.md:103-135` vs `docs/specs/hr/commands.md:696-715`
- **description:** The HR overview's "Cross-Domain Impact" section lists `StaffUnregistered` as the event emitted when a staff leaves (line 129-135), and the HR aggregates file lists the events `StaffResigned`, `StaffTerminated`, `StaffRetired` (aggregates.md:65-67) as the matching terminal transitions. The HR commands file at lines 696-715 defines `AssignSubjectTeacherCommand` but does not define a corresponding `UnregisterStaffCommand` or `WithdrawStaffCommand` — only the softer `ResignStaff`, `TerminateStaff`, and `RetireStaff` commands at `commands.md:132-150`. The overview's `StaffUnregistered` event has no producing command.
- **expected:** Either the `StaffUnregistered` event is replaced with the existing terminal events (`StaffResigned` / `StaffTerminated` / `StaffRetired`), or a new `UnregisterStaffCommand` is added to `commands.md` that emits `StaffUnregistered`.
- **evidence:** `docs/specs/hr/overview.md:128-135` `When a `Staff` is unregistered, the HR domain emits` `   `StaffUnregistered`. The following domains may subscribe:` `- `academic` — release any class teacher or subject teacher` `   assignment.` `- `finance` — close the staff's payroll.` `- `rbac` — revoke the staff's role.`. `docs/specs/hr/aggregates.md:65-67` lists `StaffResigned`, `StaffTerminated`, `StaffRetired` as terminal events. `docs/specs/hr/events.md:93-97` documents `StaffReinstated`, `StaffResigned`, `StaffTerminated`, `StaffRetired`, `StaffDeleted` — but no `StaffUnregistered`. The HR overview therefore references a non-existent event.

---

### END FINDINGS
Total Findings: 31
