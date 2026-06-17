# Phase 11 Verify Prompt — Documents

> Verify the Phase 11 (`educore-documents`) implementation against
> the spec, build plan, and handoff. Two sections: pre-implementation
> (the prep gate the implementer must pass) and post-implementation
> (the close-out gate the verifier must confirm).

---

## Section A — Pre-Implementation Verification

Before any code is written for Phase 11, the implementer must
confirm the following 10 items. Each is a hard gate; a single
"no" blocks the implementation from proceeding.

### A.1 — Required reading completed

Confirm you have read all 8 documents in the Required Reading list:

- [ ] `docs/handoff/PHASE-10-HANDOFF.md` — the most recent
  domain handoff (8 OQs carry over; OQ #1 spec-faithful, OQ #2
  `NotificationProvider` port, OQ #6 `ChatStatusRecord` rename
  are the most material for Phase 11)
- [ ] `docs/build-plan.md` § "Phase 11" (lines 1264–1353) and
  § "Phase 10 outcome."
- [ ] `docs/specs/documents/` (all 11 spec files)
- [ ] `docs/ports/{event-bus,storage,files}.md`,
  `docs/schemas/{tenancy,audit}-schema.md`
- [ ] `docs/decisions/ADR-013-CrateLayout.md`,
  `ADR-015-ExternalCrates.md`
- [ ] `crates/cross-cutting/events/src/{lib,domain_event,envelope}.rs`,
  `crates/cross-cutting/rbac/src/services.rs`,
  `crates/cross-cutting/audit/src/writer.rs`
- [ ] `crates/domains/communication/src/` (the most recent
  9-file template)
- [ ] `crates/domains/library/src/` (the proptest pattern at
  `services.rs`)
- [ ] `crates/tools/storage-parity/tests/communication_integration.rs`
  (the 6-scenario vertical-slice template)
- [ ] `AGENTS.md`, `docs_guidlines/system.md`,
  `docs_guidlines/execution_guidlines.md`

### A.2 — Workspace state confirmed

- [ ] `cargo build --workspace` is clean (Phase 10 baseline
  preserved)
- [ ] `cargo test --workspace` is green
- [ ] `cargo fmt --all -- --check` is clean
- [ ] `cargo run -p educore-core --bin lint --features lint`
  is clean
- [ ] The 17 closed crates (7 cross-cutting + 8 domain +
  storage-parity + settings) are the foundation
- [ ] `crates/domains/documents/` exists as a scaffold
  (27-line `lib.rs` with `PACKAGE_NAME` + `PACKAGE_VERSION`)

### A.3 — Spec scope understood

- [ ] `docs/specs/documents/aggregates.md` lists **3 root
  aggregates**: `FormDownload`, `PostalDispatch`, `PostalReceive`
- [ ] `FormDownload` owns 2 children: `FormDownloadFile`
  (optional `FileReference`) + `FormDownloadLink` (optional
  `Url`)
- [ ] `PostalDispatch` owns 1 child: `PostalDispatchAttachment`
  (optional)
- [ ] `PostalReceive` owns 1 child: `PostalReceiveAttachment`
  (optional)
- [ ] The `FormDownload` invariant "at least one of `link` or
  `file` set" is documented in the spec
- [ ] The `PostalDispatch.reference_no` and
  `PostalReceive.reference_no` invariants (unique within
  `(school_id, academic_id)`, immutable once set) are documented
  in the spec
- [ ] The soft-delete-only invariant (`active_status` flag,
  never hard delete) is documented for all 3 root aggregates
- [ ] The `show_public` field on `FormDownload` is documented
  (the bridge to Phase 12 CMS bus subscriber)

### A.4 — Carry-forward rules acknowledged

- [ ] **Phase 8 OQ #6 + Phase 10 OQ #3** — do NOT add a
  `educore-finance` dep
- [ ] **Phase 10 OQ #4** — do NOT add a `educore-notify` dep
  (port lands in Phase 15)
- [ ] **Phase 10 OQ #5** — do NOT add a `educore-attendance` dep
- [ ] **Phase 11 OQ #1** — `AcademicYearId` is a local `pub
  type` alias in `aggregate.rs`; do NOT add `educore-academic`
  to `educore-documents` deps in Phase 11
- [ ] **Phase 11 OQ #2** — `FileReference` is a value object
  re-exported from `educore-platform`; do NOT implement the
  `FileStorage` port in Phase 11 (deferred to Phase 15)
- [ ] **Phase 11 OQ #6** — do NOT add a `educore-cms` dep; the
  `FormUploaded` event will be consumed by CMS in Phase 12 via
  bus subscriber
- [ ] **Phase 11 OQ #7** — all 3 root aggregates ship as
  first-class ports (spec-faithful interpretation)
- [ ] **Phase 11 OQ #8** — `reference_no` immutability is
  enforced at the aggregate `update()` level; corrections
  require a new record (supersede pattern)

### A.5 — Coverage matrix read

- [ ] `docs/coverage.toml` has 3 rows for Phase 11
  (`documents_form_downloads_aggregate`,
  `documents_postal_dispatches_aggregate`,
  `documents_postal_receives_aggregate`), all `status = "Pending"`
- [ ] The 3 row ids map 1:1 to the 3 root aggregates
- [ ] No additional coverage rows are needed for Phase 11 (the
  spec has 3 root aggregates; the build-plan's ≥6 target is
  satisfied by 3 spec-faithful rows)

### A.6 — Cross-crate placeholder policy understood

- [ ] The 4 Phase 2 `DocumentsFolder*` capability placeholders
  (`DocumentsFolderCreate`, `DocumentsFolderRead`,
  `DocumentsFolderUpdate`, `DocumentsFolderDelete`) are
  retained for backward compat with `DefaultRoleCatalog`
- [ ] The 1 Phase 2 `PostalDispatch` audit target placeholder
  is retained (no duplication; the net-new Phase 11
  `PostalDispatch(Uuid)` variant matches the placeholder
  shape)
- [ ] The 4 Phase 2 `CommunicationMessage*` capability
  placeholders are NOT touched in Phase 11 (deduplicated in
  Phase 10)

### A.7 — Workstream plan drafted

- [ ] Workstream A = `FormDownload`
  (upload/update/delete + the `FileStorage` port boundary)
- [ ] Workstream B = `PostalDispatch`
  (dispatch/update/delete + `reference_no` uniqueness)
- [ ] Workstream C = `PostalReceive`
  (receive/update/delete + `reference_no` uniqueness)
- [ ] Workstream D = reconcile cross-crate placeholders +
  integration test + coverage flips + handoff docs
- [ ] The `TrackPostal` query command is included in Workstream C
  (cross-aggregate query; pairs dispatches with receives on
  `reference_no`; returns `Vec<PostalPair>`)

### A.8 — Commit plan drafted

- [ ] 5 prep-phase commits (deps + value objects + child
  entities + `AuditTarget` + rbac caps + storage-parity dep)
- [ ] 3 form workstream commits (root + children, events,
  commands)
- [ ] 3 dispatch workstream commits (root + child, events,
  commands)
- [ ] 3 receive workstream commits (root + child, events,
  commands + `TrackPostal`)
- [ ] 3 port wiring commits (`FormDownloadRepository` +
  `PostalDispatchRepository` + `PostalReceiveRepository`)
- [ ] 3 service wiring commits
  (`FormService` + `PostalDispatch` service + `PostalReceive`
  service)
- [ ] 3 query-stub commits
  (`FormDownloadQuery` + `PostalDispatchQuery` +
  `PostalReceiveQuery`)
- [ ] 2 fix-up commits (prep test errors + rustdoc on
  `DocumentsError` variants)
- [ ] 2 test commits (inline unit tests + proptest)
- [ ] 1 storage-parity integration test commit
- [ ] **Total: 30 commits** (5 + 3 + 3 + 3 + 3 + 3 + 3 + 2 + 2
  + 1 = 30)

### A.9 — Test plan drafted

- [ ] Unit tests drafted per module:
  `value_objects.rs` (24), `aggregate.rs` (22), `query.rs`
  (20), `services.rs` (18), `events.rs` (11), `errors.rs`
  (12), `commands.rs` (10), `lib.rs` (9), `entities.rs` (7),
  `repository.rs` (3) = **145 unit tests total**
- [ ] 100-case proptest drafted for the 3 headline invariants:
  `FormService::is_deliverable` (at least one of `link` or
  `file` set) +
  `PostalService::reference_unique` (no duplicate
  `(school_id, academic_id, reference_no)`) +
  `FormService::matches_publish_date` (publish date ≤ query
  date)
- [ ] 6-scenario storage-parity integration test drafted (mirrors
  `communication_integration.rs`):
  1. vertical slice (subscribe to bus → upload form → dispatch
     postal → receive postal → assert bus received the first
     envelope)
  2. capability check gates form upload
  3. event type round-trip for all 9 events
  4. postal reference uniqueness invariant
  5. soft-delete invariant
  6. form publish visibility invariant
- [ ] 2 env-gated `#[tokio::test]` PG/MySQL variants planned
  (activated via `EDUCORE_PG_URL` / `EDUCORE_MYSQL_URL`)

### A.10 — Dependencies planned

- [ ] `crates/domains/documents/Cargo.toml` declares the
  standard deps: `educore-core`, `educore-platform`,
  `educore-rbac`, `educore-events`, `educore-events-domain`,
  `educore-storage`, `educore-event-bus`, `educore-audit`,
  `async-trait`, `chrono`, `proptest`, `serde`, `serde_json`,
  `thiserror`, `uuid`, `bytes`
- [ ] No `educore-finance` dep (per carry-forward rule)
- [ ] No `educore-notify` dep (per carry-forward rule)
- [ ] No `educore-attendance` dep (per carry-forward rule)
- [ ] No `educore-cms` dep (per carry-forward rule)
- [ ] No `educore-academic` dep (per Phase 11 OQ #1;
  `AcademicYearId` is a local `pub type` alias)

---

## Section B — Post-Implementation Verification

After Phase 11 closes, the verifier must confirm the following
10 items. Each maps to an exit criterion or a handoff claim.
A single "no" blocks the phase close-out.

### B.1 — `educore-documents` crate built

- [ ] `cargo build -p educore-documents` is clean
- [ ] `cargo build --workspace` is clean (Phase 10 baseline
  preserved)
- [ ] `cargo clippy -p educore-documents --all-targets -- -D
  warnings` is clean (the documents crate itself; workspace-
  level clippy debt is documented in `docs/progress-tracker.md`)
- [ ] `cargo fmt --all -- --check` is clean
- [ ] `cargo run -p educore-core --bin lint --features lint` is
  clean
- [ ] The 9-file module layout is preserved
  (`lib.rs`, `aggregate.rs`, `entities.rs`, `value_objects.rs`,
  `commands.rs`, `events.rs`, `services.rs`, `repository.rs`,
  `query.rs`, `errors.rs`)

### B.2 — 3 root aggregates + 4 child entities wired

- [ ] `FormDownload` aggregate has non-empty `title` invariant
- [ ] `FormDownload` has the "at least one of `link` or `file`
  set" invariant
- [ ] `FormDownload` has soft-delete via `active_status`
- [ ] `FormDownload.show_public` field is wired (the bridge to
  Phase 12 CMS bus subscriber)
- [ ] `FormDownloadFile` child (optional `FileReference`) is
  defined alongside its root in `aggregate.rs`
- [ ] `FormDownloadLink` child (optional `Url`) is defined
  alongside its root in `aggregate.rs`
- [ ] `PostalDispatch` aggregate has non-empty `to_title` +
  `from_title` invariants
- [ ] `PostalDispatch.reference_no` is unique within
  `(school_id, academic_id)` when set
- [ ] `PostalDispatch.reference_no` is immutable once set
  (enforced at the aggregate `update()` level; the
  `ReferenceNoImmutable` variant of `DocumentsError` is
  returned if a future code path attempts to mutate
  `reference_no`)
- [ ] `PostalDispatch` has soft-delete via `active_status`
- [ ] `PostalDispatchAttachment` child (optional) is defined
  alongside its root in `aggregate.rs`
- [ ] `PostalReceive` aggregate mirrors `PostalDispatch`
  invariants
- [ ] `PostalReceiveAttachment` child (optional) is defined
  alongside its root in `aggregate.rs`
- [ ] All 7 typed ids are wired: `FormDownloadId`,
  `PostalDispatchId`, `PostalReceiveId`, `FormDownloadFileId`,
  `FormDownloadLinkId`, `PostalDispatchAttachmentId`,
  `PostalReceiveAttachmentId`

### B.3 — 9 typed events + 10 typed commands wired

- [ ] 9 typed events implement `DomainEvent` with wire form
  `documents.<aggregate>.<verb>`:
  `FormUploaded`, `FormUpdated`, `FormDeleted`,
  `PostalDispatched`, `PostalDispatchUpdated`,
  `PostalDispatchDeleted`, `PostalReceived`,
  `PostalReceiveUpdated`, `PostalReceiveDeleted`
- [ ] 10 typed command shapes + 10 `DOCUMENTS_*_COMMAND_TYPE`
  constants
- [ ] The 10 wire forms are:
  `documents.form_download.{upload,update,delete}`,
  `documents.postal_dispatch.{dispatch,update,delete}`,
  `documents.postal_receive.{receive,update,delete}`,
  `documents.postal.track`
- [ ] Every command carries a `TenantContext`
- [ ] The `TrackPostal` query command is events-free (pure
  read; queries the 2 postal repos directly and pairs by
  `reference_no`; returns `Vec<PostalPair>`)

### B.4 — 3 repository port traits wired

- [ ] `pub trait FormDownloadRepository: Send + Sync` with
  `insert` + `update` + `get` + `list`
- [ ] `pub trait PostalDispatchRepository: Send + Sync` with
  `insert` + `update` + `get` + `list` + `find_by_reference` +
  `between` + `by_academic_year`
- [ ] `pub trait PostalReceiveRepository: Send + Sync` with
  `insert` + `update` + `get` + `list` + `find_by_reference` +
  `between` + `by_academic_year`
- [ ] Object-safety smoke tests in `mod tests` (one per trait;
  `let _: Box<dyn XxxRepository>;` compiles)
- [ ] The `reference_no` uniqueness check is enforced at the
  `insert_postal_*` repository method with a unique index on
  `(school_id, academic_id, reference_no)`

### B.5 — 3 query stubs wired

- [ ] `FormDownloadQuery` returns
  `Err(DomainError::not_supported(...))` in Phase 11
- [ ] `PostalDispatchQuery` returns
  `Err(DomainError::not_supported(...))` in Phase 11
- [ ] `PostalReceiveQuery` returns
  `Err(DomainError::not_supported(...))` in Phase 11
- [ ] Typed executors land in a follow-up phase alongside the
  `#[derive(DomainQuery)]` macro emissions (mirrors the Phase 9
  / Phase 10 pattern)

### B.6 — 10 service factory fns + 2 service structs wired

- [ ] 10 async service factory fns are the public entry points:
  `upload_form_service`, `update_form_service`,
  `delete_form_service`, `dispatch_postal_service`,
  `update_postal_dispatch_service`,
  `delete_postal_dispatch_service`, `receive_postal_service`,
  `update_postal_receive_service`,
  `delete_postal_receive_service`, `track_postal_service`
- [ ] Each service fn wires rbac + audit + bus + repository +
  idempotency in a single transaction
- [ ] `FormService` struct exposes pure helpers:
  `validate_content`, `is_public`, `is_deliverable`,
  `matches_publish_date`
- [ ] `PostalService` struct exposes pure helpers:
  `reference_unique`, `pair_by_reference`, `within_year`,
  `format_address`
- [ ] The `DocumentsError` enum has 11 variants:
  `Validation`, `FormHasNoContent`, `FormNotFound`,
  `PostalDispatchNotFound`, `PostalReceiveNotFound`,
  `DuplicateReferenceNo`, `ReferenceNoImmutable`, `Forbidden`,
  `Conflict`, `Infrastructure`, `Other(#[from] Box<dyn Error>)`
- [ ] `From<DomainError>` and `From<EventError>` impls for
  `DocumentsError` are provided in `services.rs`

### B.7 — 145 unit tests + 100-case proptest green

- [ ] `cargo test -p educore-documents --lib` passes all
  **145 unit tests** (24 in `value_objects.rs`, 22 in
  `aggregate.rs`, 20 in `query.rs`, 18 in `services.rs`, 11 in
  `events.rs`, 12 in `errors.rs`, 10 in `commands.rs`, 9 in
  `lib.rs`, 7 in `entities.rs`, 3 in `repository.rs`)
- [ ] The 100-case proptest of
  `FormService::is_deliverable` +
  `PostalService::reference_unique` +
  `FormService::matches_publish_date` is green
- [ ] The proptest config is
  `proptest_config(proptest::test_runner::Config::with_cases(100))`
- [ ] The 4 `.clone()` sites on `TimeOfDay`-style strings are
  isolated to `services.rs`

### B.8 — 6-scenario storage-parity integration test green

- [ ] `cargo test -p educore-storage-parity --test
  documents_integration` passes 6 scenarios:
  1. `documents_integration_sqlite_vertical_slice`
  2. `documents_capability_check_gates_form_upload`
  3. `documents_event_type_round_trip_for_all_aggregates`
  4. `documents_postal_reference_uniqueness_invariant`
  5. `documents_soft_delete_invariant_holds`
  6. `documents_form_publish_visibility_invariant`
- [ ] 2 env-gated `#[tokio::test]` PG/MySQL variants are
  activated via `EDUCORE_PG_URL` / `EDUCORE_MYSQL_URL`
- [ ] The integration test mirrors `communication_integration.rs`

### B.9 — Cross-crate integration green

- [ ] `cargo test -p educore-rbac --lib` passes
  (the new
  `documents_capabilities_round_trip_and_resolve_to_documents_domain`
  test asserts 15 Documents-domain caps)
- [ ] `cargo test -p educore-audit --lib` passes
  (the new
  `documents_audit_target_round_trip_for_all_aggregates` test
  asserts 3 Documents-domain audit targets)
- [ ] 11 net-new `Capability` variants in
  `crates/cross-cutting/rbac/src/value_objects.rs`:
  `FormDownloadUpload`, `FormDownloadUpdate`,
  `FormDownloadDelete`, `FormDownloadRead`,
  `PostalDispatchCreate`, `PostalDispatchUpdate`,
  `PostalDispatchDelete`, `PostalReceiveCreate`,
  `PostalReceiveUpdate`, `PostalReceiveDelete`, `PostalRead`
- [ ] 4 retained Phase 2 `DocumentsFolder*` placeholders:
  `DocumentsFolderCreate`, `DocumentsFolderRead`,
  `DocumentsFolderUpdate`, `DocumentsFolderDelete`
- [ ] 2 net-new `AuditTarget` variants in
  `crates/cross-cutting/audit/src/writer.rs`:
  `FormDownload(Uuid)`, `PostalReceive(Uuid)`
- [ ] 1 retained `PostalDispatch(Uuid)` audit target
  placeholder (added by the prep phase)
- [ ] `DefaultRoleCatalog` extended (`school_admin`,
  `reception`, `principal`, `office_staff` roles updated with
  the relevant `Documents.*` caps)

### B.10 — Coverage matrix + handoff docs green

- [ ] 3 coverage rows flipped from `Pending` → `Tested` in
  `docs/coverage.toml`:
  - `documents_form_downloads_aggregate`
  - `documents_postal_dispatches_aggregate`
  - `documents_postal_receives_aggregate`
- [ ] `docs/handoff/PHASE-11-HANDOFF.md` is written (status,
  what's wired, validation gates, open questions, where NOT
  to start, key files, where to ask)
- [ ] `docs/phase_prompt/phase-12-prompt.md` is written (≤50
  lines, with mandatory Required Reading section)
- [ ] `docs/progress-tracker.md` is updated (workspace status
  row, phase progress row, coverage matrix summary bucket)
- [ ] `docs/build-plan.md` § "Phase 11 outcome." subsection
  is added (between `**Risks.**` and the trailing `---`)

---

## Per-Phase Specifics — Phase 11

These items are Phase 11 specific and are not covered by the
generic template above. The verifier must confirm each.

### P.11.1 — `reference_no` immutability invariant

The `reference_no` immutability is the headline invariant of
the postal aggregates. The verifier must confirm:

- [ ] `UpdatePostalDispatchCommand` and
  `UpdatePostalReceiveCommand` wire commands do NOT expose a
  `reference_no` field
- [ ] The `into_update_postal_*` helpers pass
  `reference_no: None`
- [ ] The `ReferenceNoImmutable` variant of `DocumentsError`
  is defined and returned if a future code path attempts to
  mutate `reference_no`
- [ ] Corrections require a new record (supersede pattern)

### P.11.2 — `FormDownload` "at least one of `link` or `file`" invariant

- [ ] The `is_deliverable` helper returns `false` when both
  `link` and `file` are `None`
- [ ] The `is_deliverable` helper returns `true` when at least
  one of `link` or `file` is set
- [ ] The 100-case proptest covers all 4 combinations of
  `(has_link, has_file)` ∈ `{true, false}²`
- [ ] The `FormHasNoContent` variant of `DocumentsError` is
  returned if a service fn attempts to persist a form with
  neither `link` nor `file`

### P.11.3 — `FormDownload.show_public` field

- [ ] The `show_public` field is wired on the `FormDownload`
  aggregate
- [ ] `list_public` filters on `show_public = true`
- [ ] The `documents_form_publish_visibility_invariant`
  integration test asserts:
  - `show_public = false` forms are excluded from `list_public`
  - `show_public = true` forms are included

### P.11.4 — `TrackPostal` query command

- [ ] `track_postal_service` is the only non-mutation command
- [ ] It pairs dispatches with receives on `reference_no`
- [ ] It returns `Vec<PostalPair>`
- [ ] It is events-free (pure read; no bus round-trip)
- [ ] It queries the 2 postal repos directly

### P.11.5 — `FileReference` value object re-export

- [ ] `educore_platform::value_objects::FileReference` is
  re-exported from `educore-documents`
- [ ] The `FormDownloadFile` and `PostalDispatchAttachment` /
  `PostalReceiveAttachment` child entities carry the
  `FileReference` but do not perform any I/O
- [ ] The `FileStorage` port (`FileStorage::put/get/delete`)
  is NOT implemented in Phase 11 (deferred to Phase 15)

### P.11.6 — Cross-cutting OQ: `documents.form_download.uploaded` event publication

- [ ] The `FormUploaded` event is published to the event bus
  on `upload_form_service`
- [ ] The `documents_form_download_uploaded` event type
  string is consistent with the wire form
  `documents.<aggregate>.<verb>`
- [ ] The `FormUploaded` event will be consumed by
  `educore-cms` in Phase 12 (bus subscriber — see Phase 12
  prompt's Required Reading)
- [ ] The `show_public` field is the bridge: CMS subscribes to
  `FormUploaded`, reads `show_public`, and (if true) indexes
  the form on the public site

### P.11.7 — `educore-documents` `Cargo.toml` deps

- [ ] `educore-core` (foundation)
- [ ] `educore-platform` (value objects incl. `FileReference`,
  `Url`)
- [ ] `educore-rbac` (capability check)
- [ ] `educore-events` (envelope)
- [ ] `educore-events-domain` (calendar — for date types)
- [ ] `educore-storage` (port trait)
- [ ] `educore-event-bus` (bus port)
- [ ] `educore-audit` (audit log)
- [ ] `async-trait` (async fns in traits)
- [ ] `chrono` (date types)
- [ ] `proptest` (property tests)
- [ ] `serde` + `serde_json` (serialization)
- [ ] `thiserror` (error types)
- [ ] `uuid` (typed ids)
- [ ] `bytes` (file references)

### P.11.8 — Workspace stability

- [ ] `cargo test --workspace` is green (Phase 10 baseline
  preserved; finance / facilities / hr / library /
  communication + 16 cross-cutting tests all green)
- [ ] No new clippy warnings in `educore-documents`
- [ ] Pre-existing clippy debt in `educore-finance` (Phase 7
  WIP), `educore-hr` (Phase 6 WIP), and `educore-facilities`
  (Phase 8 WIP) is documented as outstanding work in
  `docs/progress-tracker.md`

---

## Per-Phase Preamble — Phase 11 (Documents)

**Phase title:** Documents (most recent closed phase)

**Status:** Implemented (per `docs/handoff/PHASE-11-HANDOFF.md`)

**Build-plan section:** `docs/build-plan.md` lines 1264–1353

**Spec:** `docs/specs/documents/` (11 files)

**Spec aggregate count:** 3 root aggregates per
`docs/specs/documents/aggregates.md` (`FormDownload`,
`PostalDispatch`, `PostalReceive`)

**Handoff:** `docs/handoff/PHASE-11-HANDOFF.md`

**Implementation crate:** `crates/domains/documents/`
(`educore-documents`)

**Coverage rows in `docs/coverage.toml` for `phase = 11` or
`crate = "educore-documents"`:**
- `documents_form_downloads_aggregate` — `status = "Tested"`
  (spec: `docs/specs/documents/aggregates.md#formdownload`)
- `documents_postal_dispatches_aggregate` — `status = "Tested"`
  (spec: `docs/specs/documents/aggregates.md#postaldispatch`)
- `documents_postal_receives_aggregate` — `status = "Tested"`
  (spec: `docs/specs/documents/aggregates.md#postalreceive`)

**Known carry-forward rules relevant to this phase:**
- **Phase 8 OQ #6: "no `educore-finance` dep"** — Phase 11 is
  a consumer (per Subagent 3's report; Phase 11 handoff's
  "Where NOT to start" includes this rule).
- **Phase 10 OQ #3: "no `educore-finance` dep carries forward"**
  — same.
- **Phase 10 OQ #4: "no `educore-notify` dep"** — Phase 11 has
  no notification fan-out; the rule carries.
- **Phase 10 OQ #5: "no `educore-attendance` dep"** — Phase 11
  has no attendance integration.
- **Phase 11 OQ #1: "AcademicYearId import path"** — currently
  a local `pub type AcademicYearId = Uuid;` alias; a follow-up
  PR should add `educore-academic` to `educore-documents` deps.
- **Phase 11 OQ #2: "FileStorage port is Phase 15"** — Phase 11
  uses the `FileReference` value object only.
- **Phase 11 OQ #6: "no `educore-cms` dep — bus subscriber only"**
  — the `FormUploaded` event will be consumed by CMS in Phase 12.
- **Phase 11 OQ #7: "spec-faithful interpretation"** — all 3
  root aggregates ship as first-class ports.
- **Phase 11 OQ #8: "reference_no immutability"** — enforced at
  the aggregate `update()` level.
- 11 net-new `Capability` variants (Documents group)
- 2 net-new `AuditTarget` variants (`FormDownload`,
  `PostalReceive`)

**Pre-implementation gaps found in PRE-CHECK-PHASES-13-17.md
(if any for this phase):** N/A.

**Known secondary-doc gaps (from the Subagent 2 survey of
Phase 11 close-out):**
- None specific to Phase 11 (the verify prompt's Section B.3
  was the trigger for fixing the phase-12-prompt disparities;
  the documents prompt is now consistent with the spec).

**Specific verification focus:**
- The 3 root aggregates vs the 9 events vs the 10 commands
  (per handoff)
- The 3 coverage Tested rows (must match the 3 aggregates)
- The `reference_no` immutability invariant (Phase 11 OQ #8)
- The `FormDownload` invariant: at least one of `link` or
  `file` MUST be set
- The `FormDownload.show_public` field (the bridge to Phase 12
  CMS bus subscriber)
- The `TrackPostal` query command (the only non-mutation
  command)
- The 145 unit tests
- The 6-scenario storage-parity integration test (per handoff)
- The 4 child entities: `FormDownloadFile`, `FormDownloadLink`,
  `PostalDispatchAttachment`, `PostalReceiveAttachment`
- The 30 commits (5 prep + 3 form + 3 dispatch + 3 receive +
  3 port + 3 service + 3 query + 2 fix + 2 test + 1
  integration = 30)
- The 100-case proptest:
  `FormService::is_deliverable` +
  `PostalService::reference_unique` +
  `FormService::matches_publish_date`
- The `educore-documents` `Cargo.toml` deps: `educore-core`,
  `educore-platform`, `educore-rbac`, `educore-events`,
  `educore-events-domain`, `educore-storage`, `educore-event-bus`,
  `educore-audit`, `async-trait`, `chrono`, `proptest`, `serde`,
  `serde_json`, `thiserror`, `uuid`, `bytes`
- The 4 `DocumentsFolder*` Phase 2 placeholders (kept; dedup
  pattern)
- The 1 `PostalDispatch` audit target (Phase 2 placeholder;
  kept)
- The cross-cutting OQ about
  `documents.form_download.uploaded` event publication (per
  Phase 12 prompt's Required Reading)
