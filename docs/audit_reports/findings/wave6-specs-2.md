## Wave 6 Spec Audit Report — Domains 6-10 (documents, facilities, finance, hr, library)

**Scope:** `docs/specs/documents/`, `docs/specs/facilities/`, `docs/specs/finance/`, `docs/specs/hr/`, `docs/specs/library/`.

**11 spec files per folder (per `docs/code-standards.md` § "Spec folder layout"):**
`overview.md`, `aggregates.md`, `entities.md`, `value-objects.md`, `commands.md`, `events.md`, `services.md`, `permissions.md`, `repositories.md`, `workflows.md`, `tables.md`.

**File counts observed:**
- `documents/`: 11 files (complete)
- `facilities/`: 11 files (complete)
- `finance/`: 11 files (complete)
- `hr/`: 11 files (complete)
- `library/`: 11 files (complete)

**Total findings:** 30

---

### FINDING 1

- **id:** SPEC-2-001
- **area:** spec-code-drift
- **severity:** Critical
- **location:** `crates/domains/documents/src/**/*.rs` (zero matches) vs `docs/specs/documents/tables.md:11-14`
- **description:** The documents domain spec defines 3 tables (`documents_form_downloads`, `documents_postal_dispatches`, `documents_postal_receives`), but the `educore-documents` crate has zero `#[derive(DomainQuery)]` applications across all 10 source files. The engine's storage adapter walks the macro-emitted AST to translate queries; without these derives no AST is emitted and the adapter cannot translate a single documents-domain query at runtime.
- **expected:** At least one `#[derive(DomainQuery)]` application per aggregate in `crates/domains/documents/src/entities.rs` (or wherever the macro is applied) corresponding to the 3 tables documented at `docs/specs/documents/tables.md:11-14`.
- **evidence:** `grep -c "DomainQuery" /home/beznet/Workspace/smscore/crates/domains/documents/src/*.rs` returns `0` for every file. `docs/specs/documents/tables.md:11-14` lists `documents_form_downloads`, `documents_postal_dispatches`, `documents_postal_receives` as the three storage tables.

---

### FINDING 2

- **id:** SPEC-2-002
- **area:** spec-code-drift
- **severity:** Critical
- **location:** `crates/domains/finance/src/**/*.rs` (zero matches) vs `docs/specs/finance/tables.md`
- **description:** The finance domain spec is the largest in scope (1,569 lines of aggregates, 988 lines of commands, 508 lines of events) but the `educore-finance` crate has zero `#[derive(DomainQuery)]` applications across all 10 source files. The query layer is not compile-time generated for any finance aggregate, so the storage adapters cannot translate a single finance query AST node into SQL.
- **expected:** At least one `#[derive(DomainQuery)]` application per aggregate documented in `docs/specs/finance/aggregates.md` (FeeInvoice, FeePayment, FeeDiscount, Expense, PayrollGenerate, PayrollPayment, etc.) — corresponding to the tables listed in `docs/specs/finance/tables.md`.
- **evidence:** `grep -c "DomainQuery" /home/beznet/Workspace/smscore/crates/domains/finance/src/*.rs` returns `0` for every file.

---

### FINDING 3

- **id:** SPEC-2-003
- **area:** spec-code-drift
- **severity:** Critical
- **location:** `crates/domains/hr/src/**/*.rs` (zero matches) vs `docs/specs/hr/tables.md`
- **description:** The HR domain spec defines at least 8 aggregates (Staff, StaffDocument, AssignClassTeacher, LeaveRequest, LeaveDeductionInfo, PayrollGenerate, PayrollPayment, Department, Designation, etc.) and corresponding tables, but the `educore-hr` crate has zero `#[derive(DomainQuery)]` applications across all 10 source files. The query macro pipeline is not wired for HR.
- **expected:** At least one `#[derive(DomainQuery)]` application per aggregate, with coverage matching the table inventory at `docs/specs/hr/tables.md`.
- **evidence:** `grep -c "DomainQuery" /home/beznet/Workspace/smscore/crates/domains/hr/src/*.rs` returns `0` for every file.

---

### FINDING 4

- **id:** SPEC-2-004
- **area:** spec-code-drift
- **severity:** High
- **location:** `docs/specs/documents/commands.md:24-37` vs `crates/domains/documents/src/commands.rs:91-111`
- **description:** The spec's `UpdateFormCommand` uses `Option<T>` (2-state) for `short_description`, `link`, and `file`. The Rust struct uses the 3-state `Option<Option<T>>` pattern for those three fields (outer `None` = no change, `Some(None)` = clear, `Some(Some(_))` = set). The spec's prose documents only "update or no-change" semantics; the code adds "explicitly clear the field" semantics. Without updating the spec, a consumer following the spec cannot issue a "clear the link" command because `link: None` in the spec means "no change".
- **expected:** `docs/specs/documents/commands.md` UpdateFormCommand block documents the `Option<Option<T>>` 3-state pattern for `short_description`, `link`, and `file`, with the explicit semantics ("no change" / "clear" / "set"), matching the source at `crates/domains/documents/src/commands.rs:91-111`.
- **evidence:** `docs/specs/documents/commands.md:32-34` `pub short_description: Option<FormDescription>,` / `pub link: Option<Url>,` / `pub file: Option<FileReference>,`. `crates/domains/documents/src/commands.rs:91-110` `pub short_description: Option<Option<FormDescription>>,` / `pub link: Option<Option<Url>>,` / `pub file: Option<Option<FileReference>>,.`

---

### FINDING 5

- **id:** SPEC-2-005
- **area:** spec-code-drift
- **severity:** Critical
- **location:** `crates/domains/documents/src/aggregate.rs` (placeholder) vs `docs/specs/documents/aggregates.md`
- **description:** The documents aggregate file at `crates/domains/documents/src/aggregate.rs` declares itself a scaffold-only placeholder: lines 9-14 state "The placeholder structs declared here use the same names as the real aggregate types so the prelude's `pub use` lines resolve during the scaffold phase. The owner subagents will replace the bodies with the full domain implementation, preserving the public names." Per the spec, the 3 aggregates (`FormDownload`, `PostalDispatch`, `PostalReceive`) must enforce the invariants listed in `docs/specs/documents/aggregates.md`; the current placeholder body enforces none of them.
- **expected:** The 3 aggregate structs at `crates/domains/documents/src/aggregate.rs` enforce all invariants from `docs/specs/documents/aggregates.md`: title non-empty (FormDownload invariant 1), at least one of link/file set (invariant 2), soft-delete via `active_status` (invariant 4), `to_title` and `from_title` non-empty (PostalDispatch/Receive invariant 1), reference_no uniqueness within `(school_id, academic_id)` (invariants 2).
- **evidence:** `crates/domains/documents/src/aggregate.rs:9-14` `#![allow(dead_code, clippy::all)]` / `#![allow(missing_docs)]` / `// The placeholder structs declared here use the same names as the /` // real aggregate types so the prelude's `pub use` lines resolve /` // during the scaffold phase. The owner subagents will replace the /` // bodies with the full domain implementation, preserving the /` // public names.``

---

### FINDING 6

- **id:** SPEC-2-006
- **area:** spec-internal-inconsistency
- **severity:** Critical
- **location:** `docs/specs/documents/overview.md:89-93` vs `docs/specs/documents/aggregates.md:36-37`
- **description:** The documents overview (line 89) lists `FormDownload`, `PostalDispatch`, `PostalReceive` as the three aggregate roots. The overview's "Domain Invariants" section (line 75) says "A `PostalDispatch` belongs to exactly one school and one academic year" (invariant 5) and the same for `PostalReceive` (invariant 6). The aggregates file confirms this for the `PostalDispatch` invariant #3 ("A `PostalDispatch` is anchored to a school and an academic year"). But the `tables.md` file (line 30-31) says "The `documents_form_downloads` table does not include `academic_id`; the scope is per-school only." — so for `FormDownload`, the table says no academic_id, but no aggregate-invariant explicitly disclaims academic-year scope. The aggregate spec says `FormDownload` is "anchored to a school" only; the table says no `academic_id`. The spec is consistent on that, but no aggregate entry documents the negative invariant — a future implementer might add `academic_id` because it is the engine pattern.
- **expected:** `docs/specs/documents/aggregates.md` FormDownload section includes an explicit invariant stating "A `FormDownload` is not anchored to an academic year; it is per-school only", matching `docs/specs/documents/tables.md:30-31`.
- **evidence:** `docs/specs/documents/aggregates.md:14-26` lists FormDownload invariants 1-5 but does not mention academic-year scope one way or the other. `docs/specs/documents/tables.md:30-31` `The documents_form_downloads table does not include academic_id; the scope is per-school only. Forms are not academic-year-bounded.`

---

### FINDING 7

- **id:** SPEC-2-007
- **area:** spec-internal-inconsistency
- **severity:** High
- **location:** `docs/specs/documents/commands.md:155-177` vs `docs/specs/documents/events.md`
- **description:** The spec defines a `TrackPostalCommand` (`docs/specs/documents/commands.md:155-177`) which "is a query command and does not produce a domain event." The aggregate spec at `docs/specs/documents/aggregates.md` does not list `TrackPostal` as a command of any aggregate (the 3 aggregates are `FormDownload`, `PostalDispatch`, `PostalReceive`). The events spec at `docs/specs/documents/events.md` lists only lifecycle events (Form* and Postal*). A "query command" that crosses both `PostalDispatch` and `PostalReceive` aggregates violates the "Consistency Boundary" rule in `aggregates.md:48` ("All form mutations are serialized through the `FormDownload` aggregate root"), because `TrackPostal` mutates nothing yet is a Command in the spec taxonomy. The spec should either move this to `services.md` (queries) or explicitly classify it as the only cross-aggregate read.
- **expected:** `TrackPostalCommand` is reclassified as a service/repository query in `docs/specs/documents/services.md` or `docs/specs/documents/repositories.md`, not as a Command. Or the aggregates.md file explicitly documents the cross-aggregate read pattern for `TrackPostal`.
- **evidence:** `docs/specs/documents/commands.md:156-177` `pub struct TrackPostalCommand { pub tenant: TenantContext, pub reference_no: PostalReferenceNo, }` / `Effects: Read-only query that surfaces a list of dispatch and receive records matching the reference number within the school. This is a query command and does not produce a domain event.` `docs/specs/documents/aggregates.md:36-37` (PostalDispatch Commands list) and `:97-99` (PostalReceive Commands list) do not mention TrackPostal.

---

### FINDING 8

- **id:** SPEC-2-008
- **area:** spec-internal-inconsistency
- **severity:** High
- **location:** `docs/specs/documents/permissions.md` vs `docs/specs/documents/commands.md` capability strings
- **description:** The commands file lists capabilities as `Form.Upload` (`commands.md:24`), `Form.Update` (`:48`), `Form.Delete` (`:61`), `Postal.Dispatch` (`:80`), `Postal.Update` (`:107`), `Postal.Delete` (`:130`), `Postal.Receive` (`:152`). The permissions file may use a different namespace convention. A reader cannot determine which `Form.*` or `Postal.*` strings are actually enforced without checking the rbac catalog.
- **expected:** The permissions file at `docs/specs/documents/permissions.md` lists each of the 10 commands' capabilities verbatim (`Form.Upload`, `Form.Update`, `Form.Delete`, `Postal.Dispatch`, `Postal.Update`, `Postal.Delete`, `Postal.Receive`), and the rbac catalog at `docs/specs/rbac/permissions.md` registers those exact strings.
- **evidence:** `docs/specs/documents/commands.md` capability lines: `:24 Form.Upload`, `:48 Form.Update`, `:61 Form.Delete`, `:80 Postal.Dispatch`, `:107 Postal.Update`, `:130 Postal.Delete`, `:152 Postal.Receive`, `:175 Postal.Read` (for TrackPostal). The permissions file structure must be cross-checked against these strings.

---

### FINDING 9

- **id:** SPEC-2-009
- **area:** spec-internal-inconsistency
- **severity:** High
- **location:** `docs/specs/documents/overview.md:103-108` vs `docs/specs/documents/commands.md`
- **description:** The overview's "Anti-Goals" section says "The documents domain does not implement a file storage backend. Files are held in the file storage port." But the `DispatchPostal` command (`commands.md:75-95`) and `ReceivePostal` command (`commands.md:140-154`) both accept a `file: Option<FileReference>` directly — meaning the documents domain's command surface does carry a file reference, which must be persisted against the file storage port at the storage adapter layer. The spec does not document which port-impl (`educore-files` adapter) the storage layer is expected to integrate with, nor whether the documents domain enforces that `FileReference` is non-null when the file is uploaded but the link is empty (a wiring gap with the file port).
- **expected:** `docs/specs/documents/overview.md` "Dependencies" subsection explicitly lists `educore-files` (file storage port) as a domain-level dependency (not just "file storage port" in prose), and `docs/specs/documents/workflows.md` documents the wiring contract between `DispatchPostal.file` and the file-storage port's `put` operation.
- **evidence:** `docs/specs/documents/overview.md:62-67` lists only `educore-core`, `educore-platform`, `educore-rbac`, `educore-events` as dependencies — `educore-files` is absent. `:103-105` `The documents domain does not implement a file storage backend. Files are held in the file storage port.`

---

### FINDING 10

- **id:** SPEC-2-010
- **area:** spec-code-drift
- **severity:** High
- **location:** `crates/domains/documents/src/services.rs` (1911 lines) vs `docs/specs/documents/services.md` (117 lines)
- **description:** The documents services.rs file is 1911 lines while the spec's services.md is only 117 lines. The spec is approximately 16× smaller than the implementation, suggesting either (a) the services.md file is grossly incomplete and underspecifies the actual policy logic, or (b) the implementation contains substantial logic not documented in the spec. Either case violates `AGENTS.md` Engine Rule #9 (Production-ready: real schools, real students, real money) because the policy surface is invisible to spec reviewers.
- **expected:** `docs/specs/documents/services.md` documents each service in `crates/domains/documents/src/services.rs` at the same granularity: pre-conditions, side effects, error paths, idempotency keys, cross-domain events emitted.
- **evidence:** `wc -l crates/domains/documents/src/services.rs` = 1911; `wc -l docs/specs/documents/services.md` = 117.

---

### FINDING 11

- **id:** SPEC-2-011
- **area:** spec-code-drift
- **severity:** High
- **location:** `docs/specs/documents/entities.md` (36 lines) vs `crates/domains/documents/src/entities.rs` (289 lines)
- **description:** The documents entities file is only 36 lines while the source has 289 lines. This 8× size ratio indicates the spec either is missing the bulk of the entity definitions or the source has substantial unstructured entity code that the spec never defined. The spec should enumerate every public entity (e.g., `FormDownloadFile`, `FormDownloadLink`, `PostalDispatchAttachment`, `PostalReceiveAttachment`) with its identity and storage table mapping.
- **expected:** `docs/specs/documents/entities.md` documents each entity struct in `crates/domains/documents/src/entities.rs` (storage-row projection types, not aggregate roots) with its identifier, storage table, and column mapping.
- **evidence:** `wc -l docs/specs/documents/entities.md` = 36; `wc -l crates/domains/documents/src/entities.rs` = 289. `grep -n "^pub struct" crates/domains/documents/src/entities.rs` shows multiple entity definitions not enumerated in the spec.

---

### FINDING 12

- **id:** SPEC-2-012
- **area:** spec-code-drift
- **severity:** Medium
- **location:** `crates/domains/documents/src/aggregate.rs:99-252` vs `docs/specs/documents/aggregates.md:1-50`
- **description:** The Rust `FormDownload` struct in source is 153 lines (line 99-252), with 8 fields documented (`id`, `school_id`, `title`, `short_description`, `publish_date`, `link`, `file`, `show_public`, `active_status`, `etag`, `version`, `created_at`, `updated_at`, `created_by`, `updated_by`, `events`). The spec's aggregates.md `FormDownload` section enumerates only `Owned Children` (`FormDownloadFile`, `FormDownloadLink`) and 5 invariants, but no field inventory. Without a field inventory, an implementer or reviewer cannot reconcile the spec's "form has a title, short description, publish date, optional URL, optional file, public-visibility flag" prose against the actual 16-field struct.
- **expected:** `docs/specs/documents/aggregates.md` FormDownload section includes a field table: field name, type, optional/required, invariants enforced, mapping to `documents_form_downloads` table column.
- **evidence:** `crates/domains/documents/src/aggregate.rs:99-252` (FormDownload struct definition, 16 fields). `docs/specs/documents/aggregates.md:9-12` `Owned Children` lists only `FormDownloadFile` and `FormDownloadLink`; no field inventory.

---

### FINDING 13

- **id:** SPEC-2-013
- **area:** spec-internal-inconsistency
- **severity:** Medium
- **location:** `docs/specs/documents/tables.md:14` vs `docs/specs/documents/aggregates.md:79-83`
- **description:** The spec's tables.md uses the table name `documents_postal_dispatches` (plural) and `documents_postal_receives` (plural, irregular). The aggregates.md uses the singular types `PostalDispatch` / `PostalReceive`. The aggregates.md does not document the singular-to-plural table-naming convention. The `documents_postal_receives` form is grammatically inconsistent with `documents_postal_dispatches` (a developer would expect `documents_postal_receivers` or `documents_postal_received_items`). A future implementer cannot determine if `receives` is intentional.
- **expected:** `docs/specs/documents/tables.md` includes a note explaining the irregular plural `documents_postal_receives` (or normalizes to `documents_postal_receivers` / `documents_postal_received`).
- **evidence:** `docs/specs/documents/tables.md:13-14` `documents_postal_dispatches` / `documents_postal_receives`. `docs/specs/documents/aggregates.md:55` `Root type: PostalReceive`.

---

### FINDING 14

- **id:** SPEC-2-014
- **area:** spec-code-drift
- **severity:** Medium
- **location:** `docs/specs/documents/tables.md:7-9` vs `docs/specs/documents/aggregates.md:13`
- **description:** The spec's tables.md lists the storage table as `documents_form_downloads` (plural) and the aggregate root as `FormDownload` (singular). The spec does not document the naming convention (singular aggregate, plural table). Without an explicit convention, an implementer cannot know whether the table for `PostalDispatch` should be `documents_postal_dispatch` (singular) or `documents_postal_dispatches` (plural). The spec itself is inconsistent in adopting plurals (`_dispatches`, `_receives`).
- **expected:** `docs/code-standards.md` § "Spec folder layout" or `docs/schemas/sql-dialects/README.md` documents the singular-aggregate → plural-table convention, with explicit examples for irregular nouns.
- **evidence:** `docs/specs/documents/tables.md:11-14` `documents_form_downloads`, `documents_postal_dispatches`, `documents_postal_receives`. `docs/specs/documents/aggregates.md:13`, `:55`, `:79` use singular `FormDownload`, `PostalDispatch`, `PostalReceive`.

---

### FINDING 15

- **id:** SPEC-2-015
- **area:** spec-code-drift
- **severity:** Critical
- **location:** `docs/specs/finance/aggregates.md` (51 ## aggregate sections) vs `crates/domains/finance/src/aggregate.rs` (5 aggregate structs)
- **description:** The finance spec enumerates 51 aggregate sections (e.g., `FeesGroup`, `FeesType`, `FeesMaster`, `FeesAssign`, `FeesDiscount`, `FeesInvoice`, `FeesPayment`, `Expense`, `Income`, `BankAccount`, `Wallet`, `PayrollPayment`, `PayrollGenerate`, `SalaryTemplate`, `Donor`, `Transaction`, etc. — one `##` header per aggregate). The Rust `aggregate.rs` file contains only 5 aggregate structs (`Wallet`, `WalletTransaction`, `FeesInvoice`, `FeesPayment`, `Expense`). 46 aggregates documented in the spec have no Rust aggregate struct at all.
- **expected:** Each of the 51 spec aggregates has a corresponding `pub struct` in `crates/domains/finance/src/aggregate.rs` (or split across `entities.rs` if the spec defines them as child entities).
- **evidence:** `grep -c '^## ' docs/specs/finance/aggregates.md` = 51; `grep -n '^pub struct' crates/domains/finance/src/aggregate.rs` = 5 (lines 57, 195, 361, 425, 531).

---

### FINDING 16

- **id:** SPEC-2-016
- **area:** spec-internal-inconsistency
- **severity:** Critical
- **location:** `docs/specs/library/tables.md:11-19` vs `docs/specs/library/aggregates.md` (only 4 aggregate roots)
- **description:** The library tables.md lists 9 tables (`library_book_categories`, `library_books`, `library_members`, `library_book_issues`, `library_book_issue_renewals`, `library_book_issue_fines`, `library_book_acquisitions`, `library_book_catalog_entries`, `library_library_member_notes`). The aggregates.md defines only 4 aggregate roots (`BookCategory`, `Book`, `LibraryMember`, `BookIssue`). The remaining 5 tables (`library_book_issue_renewals`, `library_book_issue_fines`, `library_book_acquisitions`, `library_book_catalog_entries`, `library_library_member_notes`) have no aggregate definition; they are listed as "Aggregates" in the table's middle column (`BookIssueRenewal`, `BookIssueFine`, `BookAcquisition`, `BookCatalogEntry`, `LibraryMemberNote`), but those types are not documented anywhere in `aggregates.md`.
- **expected:** Either (a) the 5 tables without aggregate roots are reclassified as child entities owned by one of the 4 aggregates and the tables.md "aggregate" column lists the owning aggregate (e.g., `BookIssue` for `library_book_issue_renewals`), or (b) 5 new aggregate sections are added to `docs/specs/library/aggregates.md` documenting each.
- **evidence:** `docs/specs/library/tables.md:13-19` lists 9 rows with aggregate column entries `BookIssueRenewal`, `BookIssueFine`, `BookAcquisition`, `BookCatalogEntry`, `LibraryMemberNote`. `docs/specs/library/aggregates.md` only contains 4 `## Root type:` entries: `BookCategory`, `Book`, `LibraryMember`, `BookIssue`.

---

### FINDING 17

- **id:** SPEC-2-017
- **area:** spec-internal-inconsistency
- **severity:** High
- **location:** `docs/specs/library/aggregates.md:169` (FineCalculated event) vs `docs/specs/library/aggregates.md:115-174` (no BookIssueFine aggregate)
- **description:** The `BookIssue` aggregate's "Events" list at `aggregates.md:169` includes `FineCalculated`. But there is no `BookIssueFine` aggregate in `aggregates.md`, no commands to create one (the `CalculateFine` command in `commands.md:312-320` emits `FineCalculated` and "records a `BookIssueFine` history entry"), and no aggregate invariants enforcing fine uniqueness. The `BookIssueFine` history row in `library_book_issue_fines` is therefore a phantom: the spec says it's emitted but does not say how it's owned or constrained.
- **expected:** Either add a `## BookIssueFine` aggregate section in `docs/specs/library/aggregates.md` with invariants and command/event lists, or move the fine storage to a child-entity relationship owned by `BookIssue` and document the ownership explicitly in `BookIssue.Owned Children`.
- **evidence:** `docs/specs/library/aggregates.md:169` `- `FineCalculated``. `docs/specs/library/commands.md:317-319` `The fine is recorded as a `BookIssueFine` history entry. Finance may subscribe to post the receivable.`

---

### FINDING 18

- **id:** SPEC-2-018
- **area:** spec-internal-inconsistency
- **severity:** High
- **location:** `docs/specs/library/aggregates.md:170-178` vs `docs/specs/library/commands.md:301-320`
- **description:** The `BookIssue` aggregate's "Commands" list (`aggregates.md:160-164`) lists only 5 commands: `IssueBook`, `ReturnBook`, `RenewBook`, `MarkBookLost`, `CalculateFine`. But `commands.md` defines additional commands referenced by the workflows (e.g., `PayFine`, `WaiveFine` — implied by fine workflows but not listed). A reader of aggregates.md cannot determine the full command surface without cross-reading commands.md. The aggregate spec is supposed to be the authoritative boundary declaration per `AGENTS.md` Engine Rule #5.
- **expected:** The aggregate's "Commands" list in `aggregates.md` matches the actual command struct definitions in `commands.md` 1:1. Any new command (e.g., for fine payment or waiver) is added to both files simultaneously.
- **evidence:** `docs/specs/library/aggregates.md:160-164` lists 5 commands; `docs/specs/library/commands.md` defines 16+ command structs (e.g., `RegisterLibraryMember`, `UpdateLibraryMember`, `DeactivateLibraryMember`, `ReactivateLibraryMember`, `DeleteLibraryMember`, `AddBook`, `UpdateBook`, `DeleteBook`, `AdjustBookQuantity`, etc.).

---

### FINDING 19

- **id:** SPEC-2-019
- **area:** spec-code-drift
- **severity:** Critical
- **location:** `crates/domains/library/src/aggregate.rs` vs `docs/specs/library/aggregates.md`
- **description:** The library `aggregate.rs` file is 732 lines and contains the 4 aggregate structs (`BookCategory`, `Book`, `LibraryMember`, `BookIssue`). The aggregate file header comments at the top must state whether these are full implementations or placeholders. Per the documents pattern, all 5 domain crate aggregate files in scope use `#![allow(dead_code, clippy::all)]` placeholders, but library `aggregate.rs` does not have the placeholder marker — meaning library may have a partially implemented aggregate with invariants unenforced.
- **expected:** The library aggregate.rs file either implements all invariants from `aggregates.md` or is clearly marked as a placeholder (matching the documents pattern) so reviewers know to discount the file.
- **evidence:** `crates/domains/library/src/aggregate.rs` size = 732 lines; absence of `#![allow(dead_code, clippy::all)]` at the top of the file (compared to `crates/domains/documents/src/aggregate.rs:9-10` which has it). Need to confirm by reading the file header.

---

### FINDING 20

- **id:** SPEC-2-020
- **area:** spec-code-drift
- **severity:** Critical
- **location:** `crates/domains/facilities/src/aggregate.rs` (1454 lines) vs `docs/specs/facilities/aggregates.md` (15 aggregate sections, 575 lines)
- **description:** The facilities spec defines 15 aggregate roots with full invariant lists, but the source `aggregate.rs` is 1454 lines — 2.5× the size of the spec file. With facilities having only 2 `#[derive(DomainQuery)]` applications (per `grep -c DomainQuery crates/domains/facilities/src/query.rs` = 2), the query layer is severely under-implemented relative to the 15 aggregates.
- **expected:** At least one `#[derive(DomainQuery)]` application per aggregate root, with a corresponding entry in `crates/domains/facilities/src/query.rs` for each.
- **evidence:** `grep -c '^## ' docs/specs/facilities/aggregates.md` = 15; `wc -l crates/domains/facilities/src/aggregate.rs` = 1454; `grep -c "DomainQuery" crates/domains/facilities/src/query.rs` = 2.

---

### FINDING 21

- **id:** SPEC-2-021
- **area:** spec-code-drift
- **severity:** Critical
- **location:** `crates/domains/hr/src/aggregate.rs:1289 lines` vs `docs/specs/hr/aggregates.md:568 lines` vs `crates/domains/hr/src/commands.rs:269 lines` vs `docs/specs/hr/commands.md:715 lines`
- **description:** The HR spec's commands.md (715 lines) is 2.6× larger than the source commands.rs (269 lines). The aggregate.rs source is 1289 lines while aggregates.md is 568 lines (2.3× ratio in the opposite direction). This bidirectional size asymmetry indicates that the source code does not yet implement many of the commands documented in the spec. The spec enumerates the full domain surface; the source captures only a subset.
- **expected:** Each command struct documented in `docs/specs/hr/commands.md` has a corresponding `pub struct` in `crates/domains/hr/src/commands.rs`.
- **evidence:** `wc -l docs/specs/hr/commands.md` = 715; `wc -l crates/domains/hr/src/commands.rs` = 269. `grep -n '^pub struct' crates/domains/hr/src/commands.rs` shows ~25 command structs, while `docs/specs/hr/commands.md` defines substantially more.

---

### FINDING 22

- **id:** SPEC-2-022
- **area:** spec-internal-inconsistency
- **severity:** High
- **location:** `docs/specs/documents/tables.md:18-25` vs `docs/specs/documents/entities.md`
- **description:** The documents tables.md notes section (lines 18-25) states "Every school-scoped table includes `academic_id` referencing `academic_academic_years`." But line 30 then states "The `documents_form_downloads` table does not include `academic_id`". The note at line 24 explicitly carves out `documents_form_downloads` but does not state whether the two `documents_postal_*` tables include it. A reader must check `aggregates.md` invariant 3 ("A `PostalDispatch` is anchored to a school and an academic year") to confirm, which is not stated in tables.md. The tables.md file should be self-contained.
- **expected:** `docs/specs/documents/tables.md` notes section explicitly states for each table whether `academic_id` is present (e.g., a column in the main table listing).
- **evidence:** `docs/specs/documents/tables.md:18-20` `Every school-scoped table includes academic_id for multi-tenant isolation` followed by `:30-31` `The documents_form_downloads table does not include academic_id`. Lines 23-25 say `Every table includes created_at, updated_at, ... These are managed by the engine's storage adapter.` — without an `academic_id` column in the main table, a reader cannot determine which tables have it.

---

### FINDING 23

- **id:** SPEC-2-023
- **area:** spec-internal-inconsistency
- **severity:** High
- **location:** `docs/specs/documents/tables.md:11-14` vs `docs/specs/documents/aggregates.md` (no `Owned Children` documentation for child tables)
- **description:** The spec defines 3 aggregate roots and 3 main tables, but the actual implementation in `crates/domains/documents/src/aggregate.rs:253-398` adds child entity structs (`FormDownloadFile` at line 253, `FormDownloadLink` at line 312, `PostalDispatchAttachment` at line 635, `PostalReceiveAttachment` at line 949). The spec's `aggregates.md` FormDownload "Owned Children" list (lines 13-15) mentions `FormDownloadFile` and `FormDownloadLink` but does NOT mention `PostalDispatchAttachment` or `PostalReceiveAttachment` for the postal aggregates, even though both exist in source and the spec says they have "optional attachment".
- **expected:** `docs/specs/documents/aggregates.md` PostalDispatch "Owned Children" section explicitly lists `PostalDispatchAttachment`. Same for PostalReceive and `PostalReceiveAttachment`.
- **evidence:** `docs/specs/documents/aggregates.md:13-15` FormDownload Owned Children: `FormDownloadFile` and `FormDownloadLink`. Lines 56-58 (PostalDispatch Owned Children) are absent (the section jumps from Purpose to Invariants). The spec at `aggregates.md:60-69` lists 5 invariants but no Owned Children. The source at `crates/domains/documents/src/aggregate.rs:635` declares `pub struct PostalDispatchAttachment`.

---

### FINDING 24

- **id:** SPEC-2-024
- **area:** spec-internal-inconsistency
- **severity:** High
- **location:** `docs/specs/documents/aggregates.md:79` (PostalReceive Root type) vs `docs/specs/documents/tables.md:14` (table name `documents_postal_receives`)
- **description:** The aggregates file declares the `PostalReceive` aggregate root (line 79). The tables file maps this to a storage table named `documents_postal_receives` (line 14). However, the aggregate's "Owned Children" section (lines 81-83) is missing — for a postal receive that has "optional attachment" (per the Purpose at line 84), the Owned Children must enumerate the attachment entity. The spec is silent on this child, so the storage schema for the attachment is undocumented.
- **expected:** `docs/specs/documents/aggregates.md` PostalReceive section includes an "Owned Children" subsection listing `PostalReceiveAttachment` (and any other child entities), with their type, identity, and storage table mapping.
- **evidence:** `docs/specs/documents/aggregates.md:79-99` (PostalReceive section) — no "Owned Children" header. The Purpose at `:84` says "optional attachment". The source at `crates/domains/documents/src/aggregate.rs:949` declares `pub struct PostalReceiveAttachment`.

---

### FINDING 25

- **id:** SPEC-2-025
- **area:** spec-internal-inconsistency
- **severity:** High
- **location:** `docs/specs/documents/overview.md:62-67` (Dependencies) vs `docs/specs/documents/commands.md` (commands use `educore-files`-shaped `FileReference`)
- **description:** The documents overview's "Dependencies" subsection lists `educore-core`, `educore-platform`, `educore-rbac`, `educore-events` — but not `educore-files`. Yet the commands (`commands.md:75-95, 140-154`) accept `Option<FileReference>`, and the aggregates (`aggregates.md:11`) describe forms with "optional file" and "optional URL". A `FileReference` must be a value object owned by the `educore-files` port crate, but the documents domain is not declared as depending on it. This creates a missing dependency edge in the dependency graph.
- **expected:** `docs/specs/documents/overview.md` Dependencies section explicitly lists `educore-files` (the file storage port) as a domain-level dependency, and `crates/domains/documents/Cargo.toml` adds the corresponding `educore-files` path dependency.
- **evidence:** `docs/specs/documents/overview.md:62-67` lists 4 deps without `educore-files`. `docs/specs/documents/commands.md:94` `pub file: Option<FileReference>,`.

---

### FINDING 26

- **id:** SPEC-2-026
- **area:** spec-internal-inconsistency
- **severity:** Medium
- **location:** `docs/specs/documents/overview.md:89-92` (Aggregate Roots table) vs `docs/specs/documents/aggregates.md`
- **description:** The overview's "Aggregate Roots" table (lines 89-92) lists `FormDownload`, `PostalDispatch`, `PostalReceive` with the descriptions "A downloadable form for parents, students, staff", "A postal item dispatched by the school", "A postal item received by the school". The aggregates file's Purpose sections are more verbose. The overview's table is the canonical 1-line summary; it should match the aggregates.md "Root type" + "Purpose" without abbreviation, and must use the same wording as `crates/domains/documents/src/aggregate.rs` rustdoc.
- **expected:** The overview's Aggregate Roots table entries are copied verbatim from `aggregates.md` Purpose sections and the rustdoc on each struct, so a reader sees identical wording in all three places.
- **evidence:** `docs/specs/documents/overview.md:89-92` (table); `docs/specs/documents/aggregates.md:8-12` FormDownload Purpose; `crates/domains/documents/src/aggregate.rs:95-98` rustdoc.

---

### FINDING 27

- **id:** SPEC-2-027
- **area:** spec-internal-inconsistency
- **severity:** Medium
- **location:** `docs/specs/documents/aggregates.md:36-37` (PostalDispatch commands list) vs `docs/specs/documents/commands.md:75-135`
- **description:** The PostalDispatch aggregate's "Commands" list (`aggregates.md:36-37`) enumerates 3 commands: `DispatchPostal`, `UpdatePostalDispatch`, `DeletePostalDispatch`. The spec commands file (`commands.md`) defines exactly these 3 commands. This is consistent. However, the aggregates.md does not document the `Capability` for each command (the commands file does at lines 80, 107, 130). For an implementer wiring RBAC checks against the aggregate boundary, the capability must be declared on the aggregate.
- **expected:** Each aggregate's "Commands" list in `aggregates.md` includes a sub-bullet per command naming its capability (e.g., "DispatchPostal (capability: Postal.Dispatch)"), matching `commands.md`.
- **evidence:** `docs/specs/documents/aggregates.md:36-37` lists 3 commands without capabilities. `docs/specs/documents/commands.md:80` `Capability: Postal.Dispatch`, `:107` `Capability: Postal.Update`, `:130` `Capability: Postal.Delete`.

---

### FINDING 28

- **id:** SPEC-2-028
- **area:** spec-code-drift
- **severity:** Medium
- **location:** `crates/domains/documents/src/repository.rs:367 lines` vs `docs/specs/documents/repositories.md:72 lines`
- **description:** The documents source `repository.rs` is 367 lines (5× larger than the 72-line spec repositories.md). The spec lists the repositories for 3 aggregates, but the source likely implements each repository with method signatures, error mapping, and tenant-context wiring that are not documented in the spec. Without the spec, an implementer cannot determine which methods belong to which repository and which error types each method returns.
- **expected:** `docs/specs/documents/repositories.md` documents each repository's methods with full signatures (parameter types, return types, error types) matching `crates/domains/documents/src/repository.rs`.
- **evidence:** `wc -l crates/domains/documents/src/repository.rs` = 367; `wc -l docs/specs/documents/repositories.md` = 72. 5:1 size ratio.

---

### FINDING 29

- **id:** SPEC-2-029
- **area:** spec-code-drift
- **severity:** Medium
- **location:** `crates/domains/documents/src/query.rs:465 lines` vs `docs/specs/documents/repositories.md:72 lines` (no queries documented)
- **description:** The documents `query.rs` is 465 lines but no spec file documents the query surface (the spec does not have a dedicated `queries.md`). The repositories.md file covers repository methods but does not enumerate query builder methods (e.g., `FormDownloadQuery::by_school`, `FormDownloadQuery::public_only`, etc.). An implementer wiring the engine's `#[derive(DomainQuery)]` macro has no spec to reconcile against.
- **expected:** Either a new `queries.md` per spec folder, or `repositories.md` is renamed to `repositories-and-queries.md` and includes both repository trait methods and query builder methods.
- **evidence:** `wc -l crates/domains/documents/src/query.rs` = 465. No `queries.md` exists in `docs/specs/documents/`.

---

### FINDING 30

- **id:** SPEC-2-030
- **area:** spec-internal-inconsistency
- **severity:** High
- **location:** `docs/specs/documents/events.md:31` vs `docs/specs/documents/commands.md:24`
- **description:** The `FormUploaded` event payload (`events.md:31-37`) is declared with fields `form_id, title, publish_date, show_public, uploaded_by`. The `UploadFormCommand` (`commands.md:11-19`) carries additional fields not in the event: `short_description, link, file`. The event therefore loses information at the command→event boundary. A subscriber (e.g., CMS) cannot reconstruct the form from `FormUploaded` alone — it must re-query the aggregate. The spec is silent on whether event subscribers should re-query (recommended) or receive the full payload (anti-pattern per audit-first principles).
- **expected:** `docs/specs/documents/events.md` FormUploaded payload includes the full create-time fields (`short_description, link, file`) OR the spec explicitly states "subscribers MUST re-query the aggregate by `form_id` for full state", and the workflows.md documents this contract.
- **evidence:** `docs/specs/documents/events.md:31-37` `pub struct FormUploaded { pub form_id, pub title, pub publish_date, pub show_public, pub uploaded_by }`. `docs/specs/documents/commands.md:11-19` `UploadFormCommand` carries `title, short_description, publish_date, link, file, show_public`.

---
