### FINDING DOMAIN-DOC-001

- **id:** DOMAIN-DOC-001
- **area:** domain-crates
- **severity:** Critical
- **location:** crates/domains/documents/Cargo.toml:22
- **description:** The `educore-documents` crate (domains tier) declares a direct dependency on `educore-event-bus`, which lives in `crates/adapters/event-bus/` (adapters tier). This violates the tier boundary rule that a domains crate must not import from an adapters crate.
- **expected:** AGENTS.md § "Dependency Rules" mandates: "A domain crate may not depend on: Any crate in the adapters tier." The spec `docs/specs/documents/overview.md` § "Dependencies" lists only `educore-core`, `educore-platform`, `educore-rbac`, `educore-events`.
- **evidence:** `crates/domains/documents/Cargo.toml:21-22` reads: `educore-storage = { workspace = true }` followed by `educore-event-bus = { workspace = true }`. The `educore-event-bus` crate is published at `crates/adapters/event-bus/Cargo.toml` per `AGENTS.md` § "Tier System". Concrete use site: `crates/domains/documents/src/services.rs:1304` (`use educore_event_bus::InProcessEventBus;`).

### FINDING DOMAIN-DOC-002

- **id:** DOMAIN-DOC-002
- **area:** domain-crates
- **severity:** Critical
- **location:** crates/domains/documents/src/repository.rs:23-60
- **description:** The `FormDownloadRepository` trait is missing the `delete` method declared in the spec's `repositories.md`. The spec calls for hard-delete capability on the port even though the engine never hard-deletes in production — the spec is the source of truth for the trait surface.
- **expected:** `docs/specs/documents/repositories.md:17` mandates: `async fn delete(&self, id: FormDownloadId) -> Result<()>` on the `FormDownloadRepository` trait.
- **evidence:** `crates/domains/documents/src/repository.rs:23-60` defines `FormDownloadRepository` with methods `get`, `list`, `list_public`, `insert`, `update`, `by_publish_date`, `count`, `page`. There is no `delete` method. `grep "fn delete" crates/domains/documents/src/repository.rs` returns no matches.

### FINDING DOMAIN-DOC-003

- **id:** DOMAIN-DOC-003
- **area:** domain-crates
- **severity:** Critical
- **location:** crates/domains/documents/src/repository.rs:95-140
- **description:** The `PostalDispatchRepository` trait is missing the `delete` method declared in the spec's `repositories.md`.
- **expected:** `docs/specs/documents/repositories.md:33` mandates: `async fn delete(&self, id: PostalDispatchId) -> Result<()>` on the `PostalDispatchRepository` trait.
- **evidence:** `crates/domains/documents/src/repository.rs:95-140` defines `PostalDispatchRepository` with methods `get`, `list`, `insert`, `update`, `find_by_reference`, `between`, `by_academic_year`. No `delete` method.

### FINDING DOMAIN-DOC-004

- **id:** DOMAIN-DOC-004
- **area:** domain-crates
- **severity:** Critical
- **location:** crates/domains/documents/src/repository.rs:176-217
- **description:** The `PostalReceiveRepository` trait is missing the `delete` method declared in the spec's `repositories.md`.
- **expected:** `docs/specs/documents/repositories.md:49` mandates: `async fn delete(&self, id: PostalReceiveId) -> Result<()>` on the `PostalReceiveRepository` trait.
- **evidence:** `crates/domains/documents/src/repository.rs:176-217` defines `PostalReceiveRepository` with methods `get`, `list`, `insert`, `update`, `find_by_reference`, `between`, `by_academic_year`. No `delete` method.

### FINDING DOMAIN-DOC-005

- **id:** DOMAIN-DOC-005
- **area:** domain-crates
- **severity:** High
- **location:** crates/domains/documents/src/repository.rs:53-59
- **description:** The `page` method on `FormDownloadRepository` returns `Vec<FormDownload>` but the spec requires `Page<FormDownload>`. Additionally, no generic `Page<T>` type exists in the engine (`crates/infra/core/src/query.rs:399` defines a non-generic `Page { offset, limit }` struct), so the spec's reference to `Page<FormDownload>` is itself unimplementable without a new engine type.
- **expected:** `docs/specs/documents/repositories.md:20` mandates: `async fn page(&self, school: SchoolId, q: FormDownloadQuery, offset: u32, limit: u32) -> Result<Page<FormDownload>>`.
- **evidence:** `crates/domains/documents/src/repository.rs:53-59` reads: `async fn page(&self, school: SchoolId, q: FormDownloadQuery, offset: u32, limit: u32) -> StorageResult<Vec<FormDownload>>;`. `grep -rn "pub struct Page<" crates` returns no matches. The non-generic `Page` struct lives at `crates/infra/core/src/query.rs:399`.

### FINDING DOMAIN-DOC-006

- **id:** DOMAIN-DOC-006
- **area:** domain-crates
- **severity:** High
- **location:** crates/domains/documents/src/services.rs:847-887
- **description:** The `track_postal_service` always returns `dispatch: None` in the `PostalPair` result — it never queries the dispatch repository. The spec mandates returning matched dispatch + receive records; the workflow `## Postal Tracking Workflow` step 2 says "The system returns the list of matching dispatch and receive records within the school."
- **expected:** `docs/specs/documents/workflows.md:69-70` mandates step 2 of the Postal Tracking Workflow: "The system returns the list of matching dispatch and receive records within the school."
- **evidence:** `crates/domains/documents/src/services.rs:856-887` `track_postal_service` body contains `let _ = dispatch_repo;` (line 868) and then constructs `let pair = PostalPair { dispatch: None, receive: receives.into_iter().next(), };` (lines 872-875). The comment at line 850-854 explicitly states: "Until then the dispatch side is always `None`."

### FINDING DOMAIN-DOC-007

- **id:** DOMAIN-DOC-007
- **area:** domain-crates
- **severity:** High
- **location:** crates/domains/documents/src/services.rs:1-1911
- **description:** The four `Specification` structs mandated by the spec are not implemented: `PublicForms`, `ActiveForms`, `DispatchesInDateRange`, `ReceivesInDateRange`. None of the corresponding trait `Specification<T>` exists in the engine either.
- **expected:** `docs/specs/documents/services.md:39-93` mandates four `Specification<T>` impls: `PublicForms`, `ActiveForms`, `DispatchesInDateRange`, `ReceivesInDateRange`.
- **evidence:** `grep -rn "PublicForms\|ActiveForms\|DispatchesInDateRange\|ReceivesInDateRange\|trait Specification" crates` returns zero matches. No `Specification<T>` trait exists in `crates/`.

### FINDING DOMAIN-DOC-008

- **id:** DOMAIN-DOC-008
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/documents/src/services.rs:1-1911
- **description:** The `DocumentsCoordinator` mandated by the spec is not implemented in this crate. The spec places the coordinator "in the engine facade", which the documents crate itself does not host; no equivalent type exists in `crates/educore/` either.
- **expected:** `docs/specs/documents/services.md:96-114` mandates `pub struct DocumentsCoordinator<'a>` with `pub async fn upload_form(&self, cmd: UploadFormCommand) -> Result<FormDownload, DomainError>`.
- **evidence:** `grep -rn "DocumentsCoordinator" crates` returns zero matches.

### FINDING DOMAIN-DOC-009

- **id:** DOMAIN-DOC-009
- **area:** domain-crates
- **severity:** High
- **location:** crates/domains/documents/src/services.rs:145,187,257,487,531,604,700,743,815,867
- **description:** Capability gating uses the `FormDownload{Upload,Update,Delete}` / `PostalDispatch{Create,Update,Delete}` / `PostalReceive{Create,Update,Delete}` / `PostalRead` naming. The spec mandates the `<Domain>.<Aggregate>.<Action>` form: `Form.Upload`, `Form.Update`, `Form.Delete`, `Postal.Dispatch`, `Postal.Receive`, `Postal.Update`, `Postal.Delete`, `Postal.Read`.
- **expected:** `docs/specs/documents/permissions.md:7-32` mandates `<Domain>.<Aggregate>.<Action>` (e.g. `Form.Upload`, `Postal.Dispatch`). `docs/specs/documents/commands.md:24,43,56,75,95,109,128,148,162,174` uses those exact strings in `**Capability:**` markers.
- **evidence:** `crates/domains/documents/src/services.rs:145` reads `Capability::FormDownloadUpload` (vs spec `Form.Upload`). Line 487 reads `Capability::PostalDispatchCreate` (vs spec `Postal.Dispatch`). Line 867 reads `Capability::PostalRead` (matches spec).

### FINDING DOMAIN-DOC-010

- **id:** DOMAIN-DOC-010
- **area:** domain-crates
- **severity:** High
- **location:** crates/domains/documents/src/services.rs:1-1911
- **description:** The `Form.Read` capability is declared in `educore-rbac` (`FormDownloadRead`, line 720 of `crates/cross-cutting/rbac/src/value_objects.rs`) but is never checked by any service factory in `crates/domains/documents/src/services.rs`. Staff read access has no enforcement gate in the documents crate.
- **expected:** `docs/specs/documents/permissions.md:23` mandates `Form.Read` capability; `docs/specs/documents/permissions.md:50-63` says "Capabilities are checked at the command boundary."
- **evidence:** `grep -n "FormDownloadRead\|Form\.Read" crates/domains/documents/src/services.rs` returns no matches. The capability exists in `crates/cross-cutting/rbac/src/value_objects.rs:720` (`FormDownloadRead`).

### FINDING DOMAIN-DOC-011

- **id:** DOMAIN-DOC-011
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/cross-cutting/rbac/src/value_objects.rs:1-6206
- **description:** The `Form.Read.Public` capability mandated by the spec is not present in the `Capability` enum. The spec also notes it is "granted to anonymous visitors on the public site", which is the only capability the `Public` role holds.
- **expected:** `docs/specs/documents/permissions.md:24` mandates `Form.Read.Public (granted to anonymous visitors on the public site)`.
- **evidence:** `grep -n "FormReadPublic\|FormRead\.Public\|Form\.Read\.Public" crates/cross-cutting/rbac/src/` returns zero matches. The full set of `Documents.*` capabilities in `crates/cross-cutting/rbac/src/value_objects.rs:698-734` does not include `FormReadPublic`.

### FINDING DOMAIN-DOC-012

- **id:** DOMAIN-DOC-012
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/cross-cutting/rbac/src/value_objects.rs:1-6206
- **description:** The `Document.Read` cross-cutting capability mandated by the spec is not present in the `Capability` enum. The spec names it as the cross-cutting read capability shared across the documents domain.
- **expected:** `docs/specs/documents/permissions.md:16` mandates `Document.Read` (under "### Document (Cross-Cutting)").
- **evidence:** `grep -n "DocumentRead\b\|Document\.Read" crates/cross-cutting/rbac/src/` returns no matches. The only `Document`-prefixed capabilities in rbac are `HrStaffDocumentUpload` and `HrStaffDocumentDownload` (HR-domain).

### FINDING DOMAIN-DOC-013

- **id:** DOMAIN-DOC-013
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/documents/src/services.rs:855-887
- **description:** The signature `track_postal_service<DRepo, RRepo>(cmd: TrackPostalCommand, dispatch_repo: Arc<DRepo>, receive_repo: Arc<RRepo>, audit: Arc<AuditWriter>, cap: &dyn CapabilityCheck)` carries an `#[allow(unused_variables, clippy::too_many_arguments)]` and explicitly discards `dispatch_repo` (`let _ = dispatch_repo;`). The unused parameter is a code smell tied to finding DOMAIN-DOC-006.
- **expected:** The spec workflow (`docs/specs/documents/workflows.md:64-75`) says the system should query both repos and return matched records.
- **evidence:** `crates/domains/documents/src/services.rs:855` `#[allow(unused_variables, clippy::too_many_arguments)]` on `track_postal_service`. Line 868: `let _ = dispatch_repo;`.

### FINDING DOMAIN-DOC-014

- **id:** DOMAIN-DOC-014
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/documents/src/aggregate.rs:693-702
- **description:** Non-test production code in `aggregate.rs` contains a `// TODO(phase-11/1C)` comment marking a known incomplete migration to `educore-academic::value_objects::AcademicYearId`. AGENTS.md § "Agent Instructions" anti-pattern list includes `// TODO:` in non-test code.
- **expected:** AGENTS.md § "Anti-patterns" forbids `// TODO:` in non-test code; the spec `docs/specs/documents/aggregates.md:54` already establishes `PostalDispatch` belongs to a school and academic year, expecting the proper typed id.
- **evidence:** `crates/domains/documents/src/aggregate.rs:694-702` contains the comment block `// TODO(phase-11/1C): replace this local alias with` followed by `pub type AcademicYearId = Uuid;`. Outside of `#[cfg(test)]`.

### FINDING DOMAIN-DOC-015

- **id:** DOMAIN-DOC-015
- **area:** domain-crates
- **severity:** Low
- **location:** crates/domains/documents/src/lib.rs:195-197
- **description:** `lib.rs` uses `unreachable!()` inside `#[test]`-marked functions (lines 195-197). While `unreachable!()` is allowed by lint configuration in `#[cfg(test)]` blocks, it is technically a `panic!` form and is on the audit checklist. Test code is exempted from the AGENTS.md anti-pattern ban; this is logged for completeness.
- **expected:** AGENTS.md § "Type Safety" bans `panic!` in production paths (test code exempt).
- **evidence:** `crates/domains/documents/src/lib.rs:195-197`: `let _: fn() -> FormDownloadQuery = || unreachable!();` and analogous lines for the other two query types. Each is in a `#[test] fn prelude_query_structs_resolve()` function.

### FINDING DOMAIN-DOC-016

- **id:** DOMAIN-DOC-016
- **area:** domain-crates
- **severity:** Low
- **location:** crates/domains/documents/src/{aggregate,commands,entities,events,query,repository,services,value_objects}.rs
- **description:** Every domain source file carries `#![allow(missing_docs)]` at module scope, which neuters the crate-level `#![deny(missing_docs)]` set in `lib.rs:8`. While individual files still declare doc comments, the explicit `allow` defeats the engine rule.
- **expected:** AGENTS.md § "Engine Rules" mandates `#![deny(missing_docs)]` and "All public APIs are documented with rustdoc." The crate-level deny should be the enforcement mechanism.
- **evidence:** `crates/domains/documents/src/lib.rs:8` has `#![deny(missing_docs)]`. Every other source file has `#![allow(missing_docs)]`: `aggregate.rs:17`, `commands.rs:4`, `entities.rs:38`, `events.rs:4`, `query.rs:4`, `repository.rs:4`, `services.rs:4`, `value_objects.rs:20`.

### FINDING DOMAIN-DOC-017

- **id:** DOMAIN-DOC-017
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/documents/src/events.rs:99, 315, 524
- **description:** The `changes` field on `FormUpdated`, `PostalDispatchUpdated`, `PostalReceiveUpdated` is typed `Vec<String>` in code but the spec mandates `Vec<&'static str>`. Per the engine rule, `Vec<&'static str>` requires the producer to pass string literals only; `Vec<String>` allows arbitrary owned data.
- **expected:** `docs/specs/documents/events.md:46,68,96` mandates `pub changes: Vec<&'static str>` for `FormUpdated`, `PostalDispatchUpdated`, `PostalReceiveUpdated`.
- **evidence:** `crates/domains/documents/src/events.rs:99` (`FormUpdated.changes: Vec<String>`), line 315 (`PostalDispatchUpdated.changes: Vec<String>`), line 524 (`PostalReceiveUpdated.changes: Vec<String>`). Code constructs the vector with `.to_owned()` at lines 813, 869, 936, 1243-1252, etc.

### FINDING DOMAIN-DOC-018

- **id:** DOMAIN-DOC-018
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/documents/src/events.rs:1-1028
- **description:** The spec's `DomainEvent` trait uses `const TYPE` as the event-type identifier and does not include `SCHEMA_VERSION` or `AGGREGATE_TYPE`. The code uses the engine's actual `DomainEvent` trait which exposes `EVENT_TYPE`, `SCHEMA_VERSION`, and `AGGREGATE_TYPE`. The event-payload structs in `events.rs` therefore include envelope metadata (`school_id`, `event_id`, `correlation_id`, `occurred_at`) that the spec attributes to the outer `EventEnvelope<E>`, not to the payload struct.
- **expected:** `docs/specs/documents/events.md:9-16` mandates `pub trait DomainEvent { const TYPE: &'static str; fn aggregate_id(&self) -> Uuid; fn school_id(&self) -> SchoolId; fn occurred_at(&self) -> Timestamp; }`. The spec payload structs `FormUploaded`, `FormUpdated`, `FormDeleted`, etc. list only the domain payload fields.
- **evidence:** `crates/domains/documents/src/events.rs:72-88` (`impl DomainEvent for FormUploaded` with `const EVENT_TYPE`, `const SCHEMA_VERSION: u32 = 1`, `const AGGREGATE_TYPE: &'static str = "form_download"`). Lines 24-43 show `FormUploaded` carries `school_id`, `event_id`, `correlation_id`, `occurred_at` in addition to the spec's five payload fields.

### FINDING DOMAIN-DOC-019

- **id:** DOMAIN-DOC-019
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/documents/src/value_objects.rs:1-1110
- **description:** The `Validate` trait mandated by the spec for value objects is not implemented anywhere in the engine. The spec mandates `pub trait Validate { fn validate(&self) -> Result<(), ValueError>; }` on every value object.
- **expected:** `docs/specs/documents/value-objects.md:73-77` mandates: `pub trait Validate { fn validate(&self) -> Result<(), ValueError>; }`.
- **evidence:** `grep -rn "trait Validate" crates` returns zero matches. Value objects in `value_objects.rs` validate at construction time inside `::new` constructors but never expose the trait.

### FINDING DOMAIN-DOC-020

- **id:** DOMAIN-DOC-020
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/documents/src/services.rs:489,702
- **description:** `dispatch_postal_service` and `receive_postal_service` mint the typed aggregate id internally (`PostalDispatchId::new(tenant.school_id, Uuid::now_v7())`) instead of accepting it from the dispatcher like `into_new_postal_dispatch` and `into_new_postal_receive` already do (which take an `id: PostalDispatchId` parameter). The two patterns are inconsistent.
- **expected:** `docs/specs/documents/commands.md:73,118` and `crates/domains/documents/src/commands.rs:221,393` `into_new_postal_dispatch(self, id: PostalDispatchId, academic_id: AcademicYearId)` and `into_new_postal_receive(self, id: PostalReceiveId, academic_id: AcademicYearId)` show the id is supplied by the dispatcher.
- **evidence:** `crates/domains/documents/src/services.rs:489` `let id = PostalDispatchId::new(tenant.school_id, Uuid::now_v7());` and `services.rs:702` `let id = PostalReceiveId::new(tenant.school_id, Uuid::now_v7());`.

### FINDING DOMAIN-DOC-021

- **id:** DOMAIN-DOC-021
- **area:** docs-vs-code
- **severity:** High
- **location:** docs/handoff/PHASE-11-HANDOFF.md:177-184
- **description:** Phase 11 hand-off claims 145 unit tests with a specific per-file breakdown. The breakdown is incorrect: `services.rs` has 27 tests (24 `#[test]` + 3 `#[tokio::test]` matching service-factory cases plus additional) per `grep -c "#\[test\]\|#\[tokio::test\]"` of `crates/domains/documents/src/services.rs`, not the 18 stated.
- **expected:** AGENTS.md § "Engine Rules" require honest accounting.
- **evidence:** `docs/handoff/PHASE-11-HANDOFF.md:177-184` claims "services.rs (18)". Actual count is 27 in `crates/domains/documents/src/services.rs` (24 sync tests + 3 of 9 async tests, all visible via `grep -c "#\[test\]\|#\[tokio::test\]"`). The grand total 145 still matches because other counts differ by an inverse amount.

### FINDING DOMAIN-DOC-022

- **id:** DOMAIN-DOC-022
- **area:** docs-vs-code
- **severity:** High
- **location:** docs/handoff/PHASE-11-HANDOFF.md:156-160
- **description:** Phase 11 hand-off claims 5 closed enums (`FormSource`, `PostalDirection`, `PostalAttachmentKind`, `FormVisibility`, `UpdateOutcome`) plus an `AuditFields` 17-field footer struct. None of these named items exist in the code.
- **expected:** AGENTS.md § "Engine Rules" require honest accounting of what was actually shipped.
- **evidence:** `docs/handoff/PHASE-11-HANDOFF.md:156-160` lists `FormSource`, `PostalDirection`, `PostalAttachmentKind`, `FormVisibility`, `UpdateOutcome`, and `AuditFields`. `grep -rn "FormSource\|PostalDirection\|PostalAttachmentKind\|FormVisibility\|UpdateOutcome\|AuditFields" crates/domains/documents/src/` returns zero matches. The actual code has only 2 closed enums: `DocumentType` (`value_objects.rs:806`) and `DocumentVisibility` (`value_objects.rs:840`).

### FINDING DOMAIN-DOC-023

- **id:** DOMAIN-DOC-023
- **area:** docs-vs-code
- **severity:** Medium
- **location:** docs/handoff/PHASE-11-HANDOFF.md:149-150
- **description:** Phase 11 hand-off describes `FileReference` as `the educore_platform::value_objects::FileReference re-export (see OQ #2)`. The code locally defines `FileReference` at `crates/domains/documents/src/value_objects.rs:683` rather than re-exporting it from `educore-platform`.
- **expected:** Hand-off documentation should describe what the code actually does.
- **evidence:** `docs/handoff/PHASE-11-HANDOFF.md:149-150` claim `FileReference (re-exported from educore-platform)`. `crates/domains/documents/src/value_objects.rs:683` defines `pub struct FileReference(String);`. `crates/cross-cutting/platform/src/value_objects.rs` does not contain `FileReference` (`grep "FileReference" crates/cross-cutting/platform/src/value_objects.rs` returns no matches).

### FINDING DOMAIN-DOC-024

- **id:** DOMAIN-DOC-024
- **area:** docs-vs-code
- **severity:** Medium
- **location:** docs/handoff/PHASE-11-HANDOFF.md:151
- **description:** Phase 11 hand-off describes `Url` as `re-exported from educore-platform`. The code locally defines `Url` at `crates/domains/documents/src/value_objects.rs:628` rather than re-exporting it from `educore-platform`.
- **expected:** Hand-off documentation should describe what the code actually does.
- **evidence:** `docs/handoff/PHASE-11-HANDOFF.md:151` claims `Url (re-exported from educore-platform)`. `crates/domains/documents/src/value_objects.rs:628` defines `pub struct Url(String);`. `grep "pub struct Url" crates/cross-cutting/platform/src/value_objects.rs` returns no matches.

### FINDING DOMAIN-DOC-025

- **id:** DOMAIN-DOC-025
- **area:** docs-vs-code
- **severity:** Medium
- **location:** docs/handoff/PHASE-11-HANDOFF.md:170-175
- **description:** Phase 11 hand-off describes the 3 typed query stubs as "returning `Err(DomainError::not_supported(...))` in Phase 11" (matching the Phase 9 / Phase 10 pattern). The actual `FormDownloadQuery`, `PostalDispatchQuery`, `PostalReceiveQuery` are typed builders with `Default`, `new`, and `with_*` methods that never return errors — there is no `not_supported` arm anywhere.
- **expected:** Hand-off documentation should match the implementation.
- **evidence:** `docs/handoff/PHASE-11-HANDOFF.md:170-175` claims the queries "return `Err(DomainError::not_supported(...))` in Phase 11". `crates/domains/documents/src/query.rs` contains only `Default + new + with_*` builders; `grep "not_supported" crates/domains/documents/src/query.rs` returns no matches.

### FINDING DOMAIN-DOC-026

- **id:** DOMAIN-DOC-026
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/documents/ (no tests directory)
- **description:** The crate has no `crates/domains/documents/tests/` directory. AGENTS.md § "Validation Checklist" requires "At least one integration test added for new behavior" per PR. The 6-scenario integration test for documents lives at `crates/tools/storage-parity/tests/documents_integration.rs`, but the crate itself does not host any tests/ folder.
- **expected:** The standard Rust crate layout per `AGENTS.md` § "Module Layout" is: `crates/domains/<domain>/tests/` for integration tests.
- **evidence:** `find crates/domains/documents -type d` returns only `crates/domains/documents` and `crates/domains/documents/src`. No `tests/` directory.

### FINDING DOMAIN-DOC-027

- **id:** DOMAIN-DOC-027
- **area:** domain-crates
- **severity:** Low
- **location:** crates/domains/documents/src/services.rs:126,461,680
- **description:** The `snapshot`, `snapshot_dispatch`, `snapshot_receive` helpers in `services.rs` use `serde_json::to_vec(...).unwrap_or_default()` in non-test production code. This is a fall-back-to-empty pattern that hides serialization errors from the audit row pipeline. AGENTS.md § "Type Safety" expects all fallible APIs to return `Result`; using `unwrap_or_default()` here is a silent-failure pattern in the audit path.
- **expected:** AGENTS.md § "Type Safety": "All fallible APIs return `Result` for fallible operations. Use `anyhow::Result` as the default surface." A snapshot helper should propagate the JSON error or be wrapped in a fallible signature.
- **evidence:** `crates/domains/documents/src/services.rs:126` `Bytes::from(serde_json::to_vec(form).unwrap_or_default())`. Line 461: `Bytes::from(serde_json::to_vec(dispatch).unwrap_or_default())`. Line 680: `Bytes::from(serde_json::to_vec(receive).unwrap_or_default())`. All outside `#[cfg(test)]`.

### FINDING DOMAIN-DOC-028

- **id:** DOMAIN-DOC-028
- **area:** domain-crates
- **severity:** Low
- **location:** crates/domains/documents/src/services.rs:1400
- **description:** The test helper `FactoryTestFormRepo::count` uses an `as u64` cast: `Ok(self.rows.lock().unwrap().len() as u64)`. The cast is inside `#[cfg(test)]`, so it is exempt from the AGENTS.md ban on `as` on numerics in production paths. Documented for completeness.
- **expected:** AGENTS.md § "Type Safety": "Numeric conversions use `TryFrom`/`TryInto`; `as` on numerics is forbidden" in production paths. Test code is exempt.
- **evidence:** `crates/domains/documents/src/services.rs:1400` `Ok(self.rows.lock().unwrap().len() as u64)`. Inside `mod tests` block (line 902).

### FINDING DOMAIN-DOC-029

- **id:** DOMAIN-DOC-029
- **area:** docs-vs-code
- **severity:** High
- **location:** docs/specs/documents/tables.md:7-11
- **description:** `tables.md` declares 3 tables for the documents domain. None of them is referenced by a `#[derive(DomainQuery)]` struct in `crates/domains/documents/src/aggregate.rs` (or anywhere else in the crate). The crate uses manual typed `*Query` builders instead of the macro-generated AST mandated by AGENTS.md § "Engine Rules" (rule 6) and rule 2 ("Compile-time safety over strings").
- **expected:** AGENTS.md § "Engine Rules" rule 6: "No SQL/NoSQL emission from macros. The `#[derive(DomainQuery)]` macro emits an AST; storage adapters translate the AST." Each `tables.md` row should have a corresponding macro-emitted typed AST struct in `entities.rs` (per `docs/build-plan.md` § "The No-Gaps Gates").
- **evidence:** `docs/specs/documents/tables.md:7-11` lists `documents_form_downloads`, `documents_postal_dispatches`, `documents_postal_receives`. `grep -rn "#\[derive(DomainQuery)\]" crates/domains/documents/` returns zero matches. Manual `FormDownloadQuery`, `PostalDispatchQuery`, `PostalReceiveQuery` builders live at `crates/domains/documents/src/query.rs` instead.

### FINDING DOMAIN-DOC-030

- **id:** DOMAIN-DOC-030
- **area:** docs-vs-code
- **severity:** Low
- **location:** docs/specs/documents/tables.md:18-28
- **description:** `tables.md` notes every school-scoped table includes `school_id` (`NOT NULL DEFAULT 1` for the bootstrap school). The aggregate structs in `aggregate.rs` derive `school_id` from `id.school_id()` (e.g. `aggregate.rs:148` `school_id: cmd.id.school_id()`) rather than reading it from the row. While this is internally consistent, the `school_id` is also stored as a typed-field on the aggregate which means persistence must round-trip it; the spec note about `DEFAULT 1` is not preserved in any DB schema emitted by the crate (since there is no DDL emission).
- **expected:** The spec note that school_id has `DEFAULT 1` is a DB-side invariant. There is no DDL emission in the crate (per AGENTS.md runtime DDL is the adapter's job), so this note is informational.
- **evidence:** `docs/specs/documents/tables.md:15-18` "the `school_id` is `NOT NULL DEFAULT 1` for the bootstrap school". `crates/domains/documents/src/aggregate.rs:148` `school_id: cmd.id.school_id()`. No `migrations/engine/00xx_*_documents_*.sql` exists (`ls migrations/engine/`).

### FINDING DOMAIN-DOC-031

- **id:** DOMAIN-DOC-031
- **area:** docs-vs-code
- **severity:** Medium
- **location:** docs/specs/documents/value-objects.md:46-54
- **description:** `value-objects.md` lists `AcademicYearId` "From `educore-academic`" as one of the value-object rows. The documents crate uses a local `pub type AcademicYearId = Uuid;` alias instead (declared at `crates/domains/documents/src/aggregate.rs:702`). The spec/hand-off explicitly anticipate this (`PHASE-11-HANDOFF.md:349-357` OQ #1) but it remains a documented spec/code drift.
- **expected:** `docs/specs/documents/value-objects.md:52` row "AcademicYearId | From educore-academic".
- **evidence:** `docs/specs/documents/value-objects.md:52`. `crates/domains/documents/src/aggregate.rs:702` `pub type AcademicYearId = Uuid;` with comment block at lines 694-701 explaining the deviation.

### FINDING DOMAIN-DOC-032

- **id:** DOMAIN-DOC-032
- **area:** docs-vs-code
- **severity:** Low
- **location:** docs/build-plan.md:1356-1359
- **description:** Build-plan § Phase 11 outcome states: "~915 tests pass workspace-wide (was ~770 at Phase 10 close-out; +145 net new in Phase 11: 145 unit tests in `educore-documents` + 6 integration scenarios + 1 rbac 15-cap test + 1 audit 3-variant test + test fixups)." The 6-scenario integration test exists at `crates/tools/storage-parity/tests/documents_integration.rs` but the per-file test breakdown (services.rs: 18) is incorrect — the actual count is 27 (see DOMAIN-DOC-021).
- **expected:** Build-plan should accurately reflect test counts.
- **evidence:** `docs/build-plan.md:1356-1359`. `grep -c "#\[test\]\|#\[tokio::test\]" crates/domains/documents/src/services.rs` returns 27.

### FINDING DOMAIN-DOC-033

- **id:** DOMAIN-DOC-033
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/documents/src/aggregate.rs:694
- **description:** The local `pub type AcademicYearId = Uuid;` alias in `aggregate.rs` collapses the typed wrapper `Id<AcademicYear>` defined in `educore-academic::value_objects.rs` down to a bare `Uuid`. This means the documents crate's `(school_id, academic_id, reference_no)` uniqueness invariant can be silently violated if a caller passes a Uuid belonging to a different aggregate (e.g. an `EventId` Uuid mistakenly typed as `AcademicYearId`).
- **expected:** AGENTS.md § "Engine Rules" rule 2: "Compile-time safety over strings. Use macro-generated enums (`StudentField::Status`) — never string field names." The typed-id pattern is the engine's defense against cross-aggregate Uuid confusion.
- **evidence:** `crates/domains/documents/src/aggregate.rs:702` `pub type AcademicYearId = Uuid;`. The same alias is referenced in `crates/domains/documents/src/services.rs:314` `use crate::aggregate::{AcademicYearId, NewPostalDispatch, ...}`. The real `AcademicYearId` from `educore-academic` is a typed `Id<AcademicYear>` wrapper per the comment at `aggregate.rs:700-701`.

### FINDING DOMAIN-DOC-034

- **id:** DOMAIN-DOC-034
- **area:** domain-crates
- **severity:** Low
- **location:** crates/domains/documents/src/commands.rs:3
- **description:** `commands.rs` has `#![allow(dead_code, clippy::all)]` at module scope, suppressing the `dead_code` lint for the entire module. AGENTS.md § "Type Safety" prohibits `#[allow(dead_code)]` or `_var` prefixes; while the ban is targeted at unused-variable prefixes, the module-wide `dead_code` allow effectively silences dead-code detection across the entire file.
- **expected:** AGENTS.md § "Type Safety": "No `#[allow(dead_code)]` or `_var` prefixes to silence the compiler. Delete unused code, wire it in, or open a follow-up issue."
- **evidence:** `crates/domains/documents/src/commands.rs:3` `#![allow(dead_code, clippy::all)]`. The same pattern is in `events.rs:3`, `aggregate.rs:16`, `query.rs:3`, `repository.rs:3`, `services.rs:3`, `value_objects.rs:20-21`.

### FINDING DOMAIN-DOC-035

- **id:** DOMAIN-DOC-035
- **area:** docs-vs-code
- **severity:** Low
- **location:** docs/specs/documents/commands.md:165-177
- **description:** The spec mandates `pub struct TrackPostalCommand` with capability `Postal.Read`. The code uses `Capability::PostalRead` (line 867 of services.rs) which matches the spec wording. No drift here; logged as a positive confirmation.
- **expected:** `docs/specs/documents/commands.md:174` `**Capability:** \`Postal.Read\``.
- **evidence:** `crates/domains/documents/src/services.rs:867` `require_capability(cap, &cmd.tenant, Capability::PostalRead).await?;`. This is the only capability name in the codebase that matches the spec's `<Domain>.<Aggregate>.<Action>` form verbatim.

### FINDING DOMAIN-DOC-036

- **id:** DOMAIN-DOC-036
- **area:** docs-vs-code
- **severity:** Low
- **location:** crates/domains/documents/src/aggregate.rs:472-477, 786-791
- **description:** The aggregate docs reference "the Postal Dispatch Tracking workflow, step 3" and "the Postal Receive Tracking workflow, step 3" as the source for the `reference_no` immutability invariant. The workflows file `docs/specs/documents/workflows.md` does indeed describe those workflows at lines 32-46 (`## Postal Dispatch Tracking`) and 48-62 (`## Postal Receive Tracking`), and step 3 of each says "Reception updates the dispatch/receive (`UpdatePostalDispatch`/`UpdatePostalReceive`) when the address or note changes. The reference number is immutable." The references resolve correctly.
- **expected:** The cross-references should match the spec text exactly.
- **evidence:** `crates/domains/documents/src/aggregate.rs:474` reads "immutable once set (per the Postal Dispatch Tracking workflow, step 3)". `crates/domains/documents/src/aggregate.rs:788` reads "immutable once set (per the Postal Receive Tracking workflow, step 3)". `docs/specs/documents/workflows.md:40-41` is step 3 of Postal Dispatch Tracking; `docs/specs/documents/workflows.md:56-57` is step 3 of Postal Receive Tracking. No drift; cross-references are accurate.

### FINDING DOMAIN-DOC-037

- **id:** DOMAIN-DOC-037
- **area:** domain-crates
- **severity:** High
- **location:** crates/domains/documents/src/services.rs:847-887
- **description:** The `track_postal_service` audit row uses `AuditTarget::Other("postal_track".to_owned(), Uuid::now_v7())` (line 881) which invents a fresh Uuid for the audit target rather than tying the audit row to either the dispatch or the receive. This makes the audit row un-joinable to the underlying aggregates.
- **expected:** The spec workflow `## Postal Tracking Workflow` step 4 says "The system emits no domain event for the read; the read is logged in the audit sink." The audit row should be tied to a stable target id (the dispatch or receive id) for forensic queries.
- **evidence:** `crates/domains/documents/src/services.rs:881` `AuditTarget::Other("postal_track".to_owned(), Uuid::now_v7())`. A `Uuid::now_v7()` is minted solely for the audit row.

### FINDING DOMAIN-DOC-038

- **id:** DOMAIN-DOC-038
- **area:** domain-crates
- **severity:** Low
- **location:** crates/domains/documents/src/services.rs:487-489
- **description:** The capability names `Capability::PostalDispatchCreate` and `Capability::PostalReceiveCreate` use the verb `Create`, but the spec uses `Dispatch` and `Receive` respectively. The verbs `Create` and `Dispatch` are not synonymous in this domain: dispatching means recording a sent letter, while creating means a generic insert. The drift implies a weaker action than what the spec describes.
- **expected:** `docs/specs/documents/permissions.md:28` mandates `Postal.Dispatch`. `docs/specs/documents/commands.md:75` `**Capability:** \`Postal.Dispatch\``.
- **evidence:** `crates/domains/documents/src/services.rs:487` `require_capability(cap, &cmd.tenant, Capability::PostalDispatchCreate).await?;` vs `docs/specs/documents/commands.md:75` which says the capability is `Postal.Dispatch`. The rbac enum at `crates/cross-cutting/rbac/src/value_objects.rs:722` defines `PostalDispatchCreate`, not `Postal.Dispatch`.

### FINDING DOMAIN-DOC-039

- **id:** DOMAIN-DOC-039
- **area:** docs-vs-code
- **severity:** High
- **location:** docs/handoff/PHASE-11-HANDOFF.md:1-5
- **description:** Phase 11 is documented as "Closed 2026-06-16" in both `docs/build-plan.md:1294` and `docs/handoff/PHASE-11-HANDOFF.md:4`. The close-out claim implies the spec is satisfied; however, several spec items remain unimplemented (missing `delete` methods on repository traits, missing `Specifications`, missing `DocumentsCoordinator`, partial `track_postal_service` implementation, missing capabilities `Form.Read.Public` / `Document.Read`, capability naming drift, missing `Validate` trait). These gaps indicate the close-out declaration does not match the implementation state.
- **expected:** AGENTS.md § "Validation Checklist" requires all gates to pass before close. The Phase 11 close-out narrative does not acknowledge the gaps enumerated in findings DOMAIN-DOC-002 through DOMAIN-DOC-038.
- **evidence:** `docs/handoff/PHASE-11-HANDOFF.md:4` "Status: Phase 11 closed." `docs/build-plan.md:1294` "Phase 11 outcome. Closed 2026-06-16." The unimplemented items enumerated in the other findings in this report contradict the "closed" assertion.

### END FINDINGS

Total findings: 39
