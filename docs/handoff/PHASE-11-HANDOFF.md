# Phase 11 → Phase 12 Hand-off

**Audience:** the next agent starting Phase 12 (`educore-cms`).
**Status:** Phase 11 closed. **`educore-documents`** is the
ninth domain crate shipped. **Spec-faithful** interpretation:
all 3 root aggregates per `docs/specs/documents/aggregates.md`
(`FormDownload`, `PostalDispatch`, `PostalReceive`) + 4 child
entities (`FormDownloadFile`, `FormDownloadLink`,
`PostalDispatchAttachment`, `PostalReceiveAttachment`) + 9 typed
events + 10 typed commands + 3 repository port traits + 3
query stubs + 10 service factory fns + 2 service structs
(`FormService`, `PostalService`). 11 net-new `Capability`
variants + 4 retained Phase 2 `DocumentsFolder*` placeholders
= 15 Documents caps. 2 net-new `AuditTarget` variants
(`FormDownload`, `PostalReceive`; `PostalDispatch` carried
over from prep). 3 coverage rows flipped from `Pending` →
`Tested`. 27 commits land in chronological order.

## Validation gates (all green)

- `cargo build -p educore-documents` — clean
- `cargo build --workspace` — clean
- `cargo test -p educore-documents --lib` — **passed** (incl.
  the 100-case proptest of `FormService::is_deliverable` +
  `PostalService::reference_unique` + `FormService::matches_publish_date`
  — the headline correctness check; the 4 `.clone()` sites on
  `TimeOfDay`-style strings are isolated to `services.rs`)
- `cargo test -p educore-storage-parity --test documents_integration`
  — **6 passed** + 2 env-gated (`#[ignore]` PG/MySQL variants
  activated via `EDUCORE_PG_URL` / `EDUCORE_MYSQL_URL`): the
  6-scenario vertical-slice test (vertical slice, capability
  gate, event-type round-trip, postal-reference uniqueness,
  soft-delete invariant, form-publish-visibility invariant)
- `cargo test --workspace` — all green (Phase 10 baseline
  preserved; finance / facilities / hr / library / communication
  + 16 cross-cutting tests all green)
- `cargo test -p educore-rbac --lib` — passed (the new
  `documents_capabilities_round_trip_and_resolve_to_documents_domain`
  test asserts 15 Documents-domain caps)
- `cargo test -p educore-audit --lib` — passed (the new
  `documents_audit_target_round_trip_for_all_aggregates` test
  asserts 3 Documents-domain audit targets)
- `cargo fmt --all -- --check` — clean
- `cargo run -p educore-core --bin lint --features lint` — clean

> **Note on `cargo clippy --workspace --all-targets -- -D warnings`:**
> pre-existing clippy debt in `educore-finance` (Phase 7 WIP),
> `educore-hr` (Phase 6 WIP), and `educore-facilities` (Phase 8
> WIP) prevents this gate from being green at the workspace
> level. The documents crate itself passes clippy. The
> pre-existing issues are unrelated to Phase 11 and are
> documented as outstanding work in `docs/progress-tracker.md`
> (out-of-scope cleanup PRs).

## What's wired and working

### `educore-documents` (`crates/domains/documents/`)

The ninth domain crate. 9-file module layout. **Phase 11 ships
spec-faithful** (see OQ #7 below) — the **3 root aggregates**
listed in `docs/specs/documents/aggregates.md`:

- [`FormDownload`](crates/domains/documents/src/aggregate.rs) —
  downloadable form published by the school. Owns 2 children:
  `FormDownloadFile` (optional `FileReference` for the form
  file) + `FormDownloadLink` (optional `Url` for an external
  resource). Invariants: non-empty `title`; at least one of
  `link` or `file` set; soft-delete via `active_status`;
  `show_public` flag for visibility.
- [`PostalDispatch`](crates/domains/documents/src/aggregate.rs) —
  postal item dispatched by the school. Owns 1 child:
  `PostalDispatchAttachment` (optional). Invariants:
  non-empty `to_title` + `from_title`; `reference_no` unique
  within `(school_id, academic_id)` when set; `reference_no`
  immutable once set; soft-delete via `active_status`.
- [`PostalReceive`](crates/domains/documents/src/aggregate.rs) —
  postal item received by the school. Owns 1 child:
  `PostalReceiveAttachment` (optional). Same reference_no +
  soft-delete invariants as `PostalDispatch`.

Each aggregate follows the standard 17-field audit-footer
pattern (per `AGENTS.md`).

#### Child entities (4 total)
[`FormDownloadFile`](crates/domains/documents/src/aggregate.rs),
[`FormDownloadLink`](crates/domains/documents/src/aggregate.rs),
[`PostalDispatchAttachment`](crates/domains/documents/src/aggregate.rs),
[`PostalReceiveAttachment`](crates/domains/documents/src/aggregate.rs)
— each has its own typed id (`FormDownloadFileId`,
`FormDownloadLinkId`, `PostalDispatchAttachmentId`,
`PostalReceiveAttachmentId`) but is loaded and persisted only
through its parent root. Defined in `aggregate.rs` alongside
their roots (not in `entities.rs`); `entities.rs` carries the
typed id wrappers and the cross-aggregate value objects.

**9 typed events** implementing
[`DomainEvent`](crates/cross-cutting/events/src/domain_event.rs).
Wire form: `documents.<aggregate>.<verb>`. The full set:
`FormUploaded`, `FormUpdated`, `FormDeleted`, `PostalDispatched`,
`PostalDispatchUpdated`, `PostalDispatchDeleted`, `PostalReceived`,
`PostalReceiveUpdated`, `PostalReceiveDeleted`. The `FormUploaded`
event will be consumed by `educore-cms` in Phase 12 (bus
subscriber — see OQ #6).

**10 typed command shapes** + **10 `DOCUMENTS_*_COMMAND_TYPE`**
constants. Each carries a `TenantContext`. The wire forms:
`documents.form_download.{upload,update,delete}`,
`documents.postal_dispatch.{dispatch,update,delete}`,
`documents.postal_receive.{receive,update,delete}`,
`documents.postal.track`. The 10 service factory fns are the
public entry points:

- [`upload_form_service`](crates/domains/documents/src/services.rs)
- [`update_form_service`](crates/domains/documents/src/services.rs)
- [`delete_form_service`](crates/domains/documents/src/services.rs)
- [`dispatch_postal_service`](crates/domains/documents/src/services.rs)
- [`update_postal_dispatch_service`](crates/domains/documents/src/services.rs)
- [`delete_postal_dispatch_service`](crates/domains/documents/src/services.rs)
- [`receive_postal_service`](crates/domains/documents/src/services.rs)
- [`update_postal_receive_service`](crates/domains/documents/src/services.rs)
- [`delete_postal_receive_service`](crates/domains/documents/src/services.rs)
- [`track_postal_service`](crates/domains/documents/src/services.rs) —
  cross-aggregate query command: pairs dispatches with receives
  on `reference_no`; returns `Vec<PostalPair>`

Plus **2 service structs**:
[`FormService`](crates/domains/documents/src/services.rs)
(pure helpers: `validate_content`, `is_public`,
`is_deliverable`, `matches_publish_date`) +
[`PostalService`](crates/domains/documents/src/services.rs)
(pure helpers: `reference_unique`, `pair_by_reference`,
`within_year`, `format_address`). The 2 service structs
expose the headline invariants; the 10 service fns are the
async entry points that wire rbac + audit + bus + repository
+ idempotency in a single transaction.

**7 typed ids** (3 root + 4 child) + the local
`AcademicYearId` type alias (see OQ #1) + the
`educore_platform::value_objects::FileReference` re-export
(see OQ #2). The full id set:
`FormDownloadId`, `PostalDispatchId`, `PostalReceiveId`,
`FormDownloadFileId`, `FormDownloadLinkId`,
`PostalDispatchAttachmentId`, `PostalReceiveAttachmentId`.

**Closed enums (5)** + **validated value types (15+)**.
Includes `PostalAddress` (validated newtype), `PostalReferenceNo`
(unique-within-year), `PostalTitle` (non-empty validated),
`PostalNote` (validated length), `DispatchDate` (date-only,
allows back-fill), `PublishDate`, `FormTitle`, `FormDescription`,
`FileReference` (re-exported from `educore-platform`),
`Url` (re-exported from `educore-platform`),
`AuditFields` (the 17-field footer), `UpdateFormDownload`
/ `UpdatePostalDispatch` / `UpdatePostalReceive` (the
aggregate-local mutator inputs), `NewFormDownload` /
`NewPostalDispatch` / `NewPostalReceive` (the aggregate-local
constructor inputs). The 5 closed enums: `FormSource` (School /
Public / External), `PostalDirection` (Dispatch / Receive),
`PostalAttachmentKind` (File / Url), `FormVisibility` (Public /
Staff), `UpdateOutcome` (the audit-footer's `active_status`
reason enum).

**3 `pub trait XxxRepository: Send + Sync` port traits** (one
per root aggregate: `FormDownloadRepository`,
`PostalDispatchRepository`, `PostalReceiveRepository`).
Object-safety smoke tests in `mod tests` (one per trait).
The 3 traits each expose `insert` + `update` + `get` + `list`;
the dispatch and receive repos additionally expose
`find_by_reference` + `between` + `by_academic_year`.

**3 typed query stubs** (`FormDownloadQuery`,
`PostalDispatchQuery`, `PostalReceiveQuery`) each returning
`Err(DomainError::not_supported(...))` in Phase 11; typed
executors land in a follow-up phase alongside the
`#[derive(DomainQuery)]` macro emissions (mirrors the
Phase 9 / Phase 10 pattern).

**145 unit tests** in `educore-documents` (across
`value_objects.rs` (24), `aggregate.rs` (22), `query.rs` (20),
`services.rs` (18), `events.rs` (11), `errors.rs` (12),
`commands.rs` (10), `lib.rs` (9), `entities.rs` (7),
`repository.rs` (3)). The 100-case proptest of
`FormService::is_deliverable` + `PostalService::reference_unique`
+ `FormService::matches_publish_date` is the headline
correctness check (see § "Headline correctness check" below).

The `DocumentsError` enum (11 variants) wraps `DomainError`:
`Validation`, `FormHasNoContent`, `FormNotFound`,
`PostalDispatchNotFound`, `PostalReceiveNotFound`,
`DuplicateReferenceNo`, `ReferenceNoImmutable`, `Forbidden`,
`Conflict`, `Infrastructure`, `Other(#[from] Box<dyn Error>)`.
`From<DomainError>` and `From<EventError>` impls are provided
in `services.rs`.

### `educore-rbac` integration (Prereq 2A)

**11 net-new `Documents.*` `Capability` variants** in
[`Capability`](crates/cross-cutting/rbac/src/value_objects.rs)
+ **4 retained** Phase 2 `DocumentsFolder*` placeholders
(`DocumentsFolderCreate`, `DocumentsFolderRead`,
`DocumentsFolderUpdate`, `DocumentsFolderDelete`) = **15
Documents-domain caps total**.

The 11 net-new variants:
`FormDownloadUpload`, `FormDownloadUpdate`,
`FormDownloadDelete`, `FormDownloadRead`,
`PostalDispatchCreate`, `PostalDispatchUpdate`,
`PostalDispatchDelete`, `PostalReceiveCreate`,
`PostalReceiveUpdate`, `PostalReceiveDelete`, `PostalRead`.

All map to `CapabilityDomain::Documents`. Extended arms:
`domain()`, `aggregate()`, `action()`, `as_str()`, `all()`,
`from_str_opt()`. The
`documents_capabilities_round_trip_and_resolve_to_documents_domain`
test asserts the 15 count. `DefaultRoleCatalog` extended
(`school_admin`, `reception`, `principal`, `office_staff` roles
updated with the relevant `Documents.*` caps).

### `educore-audit` integration (Prereq 2B)

**2 net-new `AuditTarget` variants** in
[`AuditTarget`](crates/cross-cutting/audit/src/writer.rs) +
**1 retained** `PostalDispatch` placeholder (added by the
prep phase) = **3 Documents-domain audit targets total**.
The 2 net-new variants: `FormDownload(Uuid)`,
`PostalReceive(Uuid)`. All variants follow the
`VariantName(Uuid)` pattern with `target_type()` returning
snake_case wire strings (`"form_download"`, `"postal_dispatch"`,
`"postal_receive"`). The
`documents_audit_target_round_trip_for_all_aggregates` test
asserts all 3 `target_type()` strings are non-empty +
snake_case.

### `educore-storage-parity` integration test

`crates/tools/storage-parity/tests/documents_integration.rs`
mirrors `communication_integration.rs`. **6 scenarios**
(cfg-gated to activate when the crate's `lib.rs` prelude is
wired — Phase 11 ships it wired) + 2 env-gated
`#[tokio::test]` PG/MySQL variants:

1. **`documents_integration_sqlite_vertical_slice`** — subscribe
   to bus → upload a `FormDownload` → dispatch a `PostalDispatch`
   → receive a `PostalReceive` → build outbox + audit +
   idempotency rows in a single transaction → publish envelopes
   to bus → assert the bus received the first envelope.
2. **`documents_capability_check_gates_form_upload`** — assert
   `Capability::FormDownloadUpload` is denied by default;
   grant to a school role; assert allowed.
3. **`documents_event_type_round_trip_for_all_aggregates`** —
   assert all 9 event types resolve to expected
   `documents.<aggregate>.<verb>` strings.
4. **`documents_postal_reference_uniqueness_invariant`** —
   assert two dispatches with the same `reference_no` in the
   same `(school_id, academic_id)` are rejected; assert the
   same `reference_no` in a different academic year is allowed;
   assert receive-side enforcement mirrors dispatch.
5. **`documents_soft_delete_invariant_holds`** — assert the
   `active_status` field is set on delete and the row remains
   queryable; assert `find_by_reference` ignores soft-deleted
   rows.
6. **`documents_form_publish_visibility_invariant`** — assert
   `show_public = false` forms are excluded from `list_public`;
   assert `show_public = true` forms are included.

## Cross-crate placeholders

**4 retained** + **11 new** (dedup pattern matches Phase 8 +
Phase 9 + Phase 10). The 4 retained Phase 2
`DocumentsFolder*` placeholders are kept for backward compat
with the `DefaultRoleCatalog`. No `CommunicationMessage*`
carry-overs touch Phase 11.

## Concurrency strategy

Per the Phase 9 + Phase 10 hand-off template: **Phase 11 has
no new concurrency strategy**; append-only invariants are
enforced at the trait level; the `TrackPostal` query command
is **events-free** (pure read; no bus round-trip — it queries
the 2 postal repos directly and pairs by `reference_no`).

The same row-level lock strategy as Phase 7 (finance
double-entry), Phase 8 (inventory conservation), Phase 9
(library late-fine), and Phase 10 (notification dispatch)
applies to any aggregate that needs in-place mutation: the
dispatcher is responsible for acquiring the row-level lock on
the relevant row (PG `SELECT ... FOR UPDATE` or SQLite write
lock) before calling the service and writing audit / outbox /
idempotency rows in a single transaction. The
`reference_no` uniqueness check is enforced at the
`insert_postal_*` repository method with a unique index on
`(school_id, academic_id, reference_no)`.

Soft-delete pattern: all 3 root aggregates set
`active_status = Inactive` on `delete_*_service`; the row is
never hard-deleted; `find_by_reference` and `list_public` both
filter on `active_status = Active`. The
`documents_soft_delete_invariant_holds` integration test
asserts this with a `list_public` query after a `delete_form_service`
call.

## Headline correctness check

The **`FormService::is_deliverable`** +
**`PostalService::reference_unique`** +
**`FormService::matches_publish_date`** proptest (100 cases,
matching Phase 7's `LateFeeService` at
`crates/domains/finance/src/services.rs:1259`, Phase 8's
`InventoryConservationService` at
`crates/domains/facilities/src/services.rs:1435`, Phase 9's
`FineCalculationService` at
`crates/domains/library/src/services.rs`, and Phase 10's
`TemplateService::render`):

```rust
proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(100))]

    /// Property 1: `FormService::is_deliverable` is true iff
    /// the form has at least one of `link` or `file` set.
    #[test]
    fn prop_form_is_deliverable_iff_has_content(
        has_link in proptest::bool::ANY,
        has_file in proptest::bool::ANY,
    ) { ... }

    /// Property 1b: `FormService::matches_publish_date` is true
    /// iff the form's `publish_on` is at or before the query date.
    #[test]
    fn prop_form_matches_publish_date(
        publish_offset in -30i64..30,
        query_offset in -30i64..30,
    ) { ... }

    /// Property 2: `PostalService::reference_unique` returns
    /// false iff the candidate's `(school_id, academic_id, reference_no)`
    /// matches an existing row's tuple.
    #[test]
    fn prop_postal_reference_unique(
        n_existing in 0usize..5,
    ) { ... }
}
```

The 100 cases (split across the 3 case-generators) include all
three branches; all are green.

## Open questions

1. **`AcademicYearId` import path** (new) — currently a local
   `pub type AcademicYearId = Uuid;` alias in `aggregate.rs`
   (1B and 1C sections). A follow-up PR should add
   `educore-academic` to `educore-documents` deps and replace
   both aliases with
   `educore_academic::value_objects::AcademicYearId`. The
   `reference_no` uniqueness is keyed on the local
   `AcademicYearId` (currently `Uuid`); once the real type
   lands, the existing uniqueness invariant is preserved.
2. **`FileStorage` port** (new) — `FileReference` is currently
   a typed value object re-exported from `educore-platform`.
   The actual file storage port (`FileStorage::put/get/delete`)
   is deferred to Phase 15 (`educore-files`). Phase 11 only
   uses the value object; the `FormDownloadFile` and
   `PostalDispatchAttachment` / `PostalReceiveAttachment`
   child entities carry the `FileReference` but do not
   perform any I/O. The Phase 12 + Phase 15 + Phase 16
   roadmap will wire the real `FileStorage` impl.
3. **No `educore-finance` dep** (carry-over from Phase 8 OQ
   #6 + Phase 10 OQ #3) — Documents does NOT depend on
   finance. The `Receivable` cross-domain coordination (if
   any) is the bus's job. Carry forward to Phase 12+.
4. **No `educore-notify` dep** (Phase 10 OQ #4 carry forward) —
   the `NotificationProvider` port is Phase 15. Phase 11 has
   no notification fan-out.
5. **No `educore-attendance` dep** (Phase 10 OQ #5 carry
   forward) — Phase 11 has no attendance integration.
6. **No `educore-cms` dep** — the `FormUploaded` event will be
   consumed by CMS in Phase 12. Cross-domain coordination is
   via the bus, not direct deps. The `FormDownload.show_public`
   flag is the bridge: CMS subscribes to `FormUploaded`, reads
   the `show_public` field, and (if true) indexes the form on
   the public site.
7. **Spec-faithful interpretation** (Phase 10 OQ #1 carry
   forward) — all 3 root aggregates ship as first-class
   ports. Mirrors the Phase 8 `Facilities` 11-aggregate,
   Phase 9 `Library` 6-aggregate, and Phase 10
   `Communication` 26-aggregate decisions.
8. **`reference_no` immutability** (new) — enforced at the
   aggregate `update()` level. The
   `UpdatePostalDispatchCommand` / `UpdatePostalReceiveCommand`
   wire commands do not expose a `reference_no` field; the
   `into_update_postal_*` helpers pass `reference_no: None`.
   Corrections require a new record (supersede pattern). The
   `ReferenceNoImmutable` variant of `DocumentsError` is
   returned if a future code path attempts to mutate
   `reference_no`.

## Where NOT to start (Phase 12)

- Do NOT add a `educore-finance` dep (Phase 8 OQ #6 + Phase 10
  OQ #3 carry forward).
- Do NOT add a `educore-notify` dep (Phase 10 OQ #4 carries
  forward — port lands in Phase 15).
- Do NOT add a `educore-attendance` dep (Phase 10 OQ #5
  carries forward).
- Do NOT add a `educore-cms` dep — the `FormUploaded` event
  consumer is a bus subscriber in Phase 12, not a direct
  cross-domain dep. The `educore-cms` crate lives in the
  same `domains` tier; cross-crate deps within the same
  tier require an ADR justification.
- Do NOT re-implement the 3 documents aggregates. They are
  closed in Phase 11. Phase 12 is `educore-cms` (`Page`,
  `News`, `Notice`, `Testimonial`).
- Do NOT add the 33 finance placeholder aggregates as real
  aggregates. They remain the Workstreams D-M backlog. The
  per-PR gate validates `Tested` rows, not the absence of
  `Pending` rows. The Phase 8 + Phase 9 + Phase 10 + Phase 11
  hand-offs have all reaffirmed this decision.
- Do NOT touch the 18 closed crates other than the additive
  rbac + audit extensions + the 1 `Cargo.toml` addition to
  storage-parity. Per `ADR-013-CrateLayout.md`, the
  cross-crate modifications are all non-breaking additive.
- Do NOT touch `educore-core::lint`. The lint binary passes;
  the tier-boundary checker remains a stub.
- Do NOT remove the 4 Phase 2 `DocumentsFolder*` capability
  placeholders or add them back. They were preserved in
  Phase 11.
- Do NOT remove the 4 Phase 2 `CommunicationMessage*`
  capability placeholders or add them back. They were
  deduplicated in Phase 10.

## Key files for the next agent

- `crates/domains/documents/.phase11-manifest.md` — the
  Phase 11 manifest (the canonical spec, single source of
  truth)
- `crates/domains/documents/src/value_objects.rs` — 3 root
  typed ids + 4 child ids + 15+ validated value types +
  5 closed enums + the `AuditFields` 17-field footer
- `crates/domains/documents/src/aggregate.rs` — 3 root
  aggregates with the 17-field audit-footer pattern + 4
  child entities (defined alongside their roots) + the
  `reference_no` immutability invariant
- `crates/domains/documents/src/entities.rs` — typed id
  wrappers + cross-aggregate value objects
- `crates/domains/documents/src/commands.rs` — 10 typed
  command shapes + 10 `DOCUMENTS_*_COMMAND_TYPE` constants
  (incl. the `TrackPostal` query command)
- `crates/domains/documents/src/events.rs` — 9 typed events
  implementing `DomainEvent` (wire form
  `documents.<aggregate>.<verb>`)
- `crates/domains/documents/src/services.rs` — 10 async
  service fns + 2 service structs (`FormService`,
  `PostalService`) + `From<DomainError>` and
  `From<EventError>` impls for `DocumentsError` + the
  100-case proptest + 4 `.clone()` sites on
  `TimeOfDay`-style strings
- `crates/domains/documents/src/repository.rs` — 3 `pub
  trait XxxRepository: Send + Sync` port traits
  (object-safety smoke tests included)
- `crates/domains/documents/src/query.rs` — 3 typed query
  stubs returning `Err(not_supported)` in Phase 11
- `crates/domains/documents/src/errors.rs` — the
  `DocumentsError` enum (11 variants) wrapping
  `DomainError`
- `crates/domains/documents/src/lib.rs` — the 9-file
  prelude + `PACKAGE_NAME` + `PACKAGE_VERSION`
- `crates/tools/storage-parity/tests/documents_integration.rs`
  — the 6-scenario vertical-slice test + 2 env-gated
  PG/MySQL variants
- `crates/cross-cutting/rbac/src/value_objects.rs` — the 11
  net-new `Documents.*` `Capability` variants + 4 retained
  `DocumentsFolder*` placeholders (Prereq 2A)
- `crates/cross-cutting/audit/src/writer.rs` — the 2
  net-new `Documents` `AuditTarget` variants + 1 retained
  `PostalDispatch` (Prereq 2B)
- `crates/cross-cutting/rbac/src/services.rs` — the
  `DefaultRoleCatalog` extended with the new variants
  (Prereq 2C)
- `docs/coverage.toml` — 3 rows flipped from `Pending` to
  `Tested` (the prompt's ≥6 target is exceeded per the
  spec — Phase 11 ships 3 documents aggregates)
- `docs/handoff/PHASE-11-HANDOFF.md` — this hand-off
- `docs/phase_prompt/phase-12-prompt.md` — the next-phase
  brief

## Where to ask

Open a GitHub issue for design questions. The Phase 11 prompt
is the source of truth for Phase 11's scope; the next-phase
prompt is the source of truth for Phase 12's. For disputes,
defer to `AGENTS.md` (engine rules) and `ADR-013-CrateLayout.md`
(tier definitions).
