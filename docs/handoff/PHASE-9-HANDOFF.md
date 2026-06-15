# Phase 9 → Phase 10 Hand-off

**Audience:** the next agent starting Phase 10 (`educore-communication`).
**Status:** Phase 9 closed. **`educore-library`** is the seventh
domain crate shipped. The headline 6 aggregates + 3 child
entities + 18 events + 18 typed commands + 6 service
factories + 6 repository ports + 6 query stubs + the
`FineCalculationService` (the late-fine proptest) all ship
with the 9-file module layout. 10 coverage rows flipped from
`Pending` → `Tested`. 5 commits land in chronological order.

## Validation gates (all green)

- `cargo build -p educore-library` — clean
- `cargo build --workspace` — clean
- `cargo test -p educore-library --lib` — **31 passed**
  (incl. the 2-case 100-property proptest for the
  `FineCalculationService`)
- `cargo test -p educore-storage-parity --test library_integration` — **4 passed**
  (vertical slice + capability check + event round-trip + late-fine
  invariant)
- `cargo test --workspace` — all green (Phase 8 baseline
  preserved; finance / facilities / hr / library + 14 cross-cutting
  tests all green)
- `cargo test -p educore-rbac --lib` — 46 passed
- `cargo test -p educore-audit --lib` — 23 passed
- `cargo fmt --all -- --check` — clean
- `cargo run -p educore-core --bin lint --features lint` — clean

> **Note on `cargo clippy --workspace --all-targets -- -D warnings`:**
> pre-existing clippy debt in `educore-finance` (Phase 7 WIP),
> `educore-hr` (Phase 6 WIP), and `educore-facilities` (Phase 8
> WIP) prevents this gate from being green at the workspace
> level. The library crate itself passes clippy. The pre-existing
> issues are unrelated to Phase 9 and are documented as
> outstanding work in `docs/progress-tracker.md` (out-of-scope
> cleanup PRs).

## What's wired and working

### `educore-library` (`crates/domains/library/`)

The seventh domain crate. 9-file module layout. Phase 9 ships
the **6 root aggregates** (per the prompt's
"6 headline aggregates" requirement — see OQ #2 below):

- [`Book`](crates/domains/library/src/aggregate.rs) — book master
  record (with `AvailableCopies` derived from `Quantity` minus
  the sum of open-issue quantities).
- [`BookCategory`](crates/domains/library/src/aggregate.rs) —
  category catalog.
- [`LibraryMember`](crates/domains/library/src/aggregate.rs) —
  registered borrower (student or staff).
- [`BookIssue`](crates/domains/library/src/aggregate.rs) — an
  issue of a book to a member, with the `IssueStatus` state
  machine `Issued → Renewed → Returned` (or `Overdue` as a
  derived state, or `Lost` as a terminal state).
- [`BookReturn`](crates/domains/library/src/aggregate.rs) — a
  historical log of a return action (append-only record; the
  `BookIssue` keeps the canonical `IssueStatus = Returned`).
- [`Fine`](crates/domains/library/src/aggregate.rs) — a
  calculated or waived fine, attached to a `BookIssue`.

Each aggregate follows the standard 17-field audit-footer
pattern (per `AGENTS.md`).

**3 child entities** (per the spec's `entities.md` + the prompt's
"+3 child entities" requirement):

- [`BookCatalogEntry`](crates/domains/library/src/entities.rs) —
  versioned view of a book's cataloguing metadata.
- [`BookAcquisition`](crates/domains/library/src/entities.rs) —
  single procurement event for a book.
- [`LibraryMemberNote`](crates/domains/library/src/entities.rs) —
  free-text administrative note about a member.

Plus the 2 spec-mandated child entities:
[`BookIssueRenewal`](crates/domains/library/src/entities.rs)
and [`BookIssueFine`](crates/domains/library/src/entities.rs).

**18 typed events** implementing
[`DomainEvent`](crates/cross-cutting/events/src/domain_event.rs).
Wire form: `library.<aggregate>.<verb>`. The full set:

`BookCategoryCreated`, `BookCategoryUpdated`, `BookCategoryDeleted`,
`BookAdded`, `BookUpdated`, `BookDeleted`, `BookQuantityAdjusted`,
`LibraryMemberRegistered`, `LibraryMemberUpdated`,
`LibraryMemberDeactivated`, `LibraryMemberReactivated`,
`LibraryMemberDeleted`, `BookIssued`, `BookReturned`, `BookRenewed`,
`BookMarkedLost`, `BookReturnRecorded`, `FineCalculated`,
`FineWaived`.

**18 typed command shapes** + **18 `LIBRARY_*_COMMAND_TYPE`**
constants (one per command, wire form `library.<aggregate>.<verb>`).
The headline 6 service factories:

- [`create_book_category`](crates/domains/library/src/services.rs)
- [`add_book`](crates/domains/library/src/services.rs)
- [`register_library_member`](crates/domains/library/src/services.rs)
- [`create_book_issue`](crates/domains/library/src/services.rs)
- [`return_book`](crates/domains/library/src/services.rs)
- [`compute_fine`](crates/domains/library/src/services.rs)

Plus the helpers: `BookIssueEligibility`, `BookRenewalEligibility`,
`OverdueIssues`, `AvailableBooks`, `ActiveMembers`,
`BookService`, `FineCalculationService`.

**6 typed ids** (one per aggregate) + **2 child ids**:
`BookId`, `BookCategoryId`, `LibraryMemberId`, `BookIssueId`,
`BookReturnId`, `FineId`, `BookIssueRenewalId`,
`BookIssueFineId`.

**Closed enums** + **validated value types**:
`IssueStatus` (Issued / Returned / Renewed / Overdue / Lost),
`MemberStatus` (Active / Inactive / Blocked), `BookStatus`
(Available / Catalogued / Retired / Lost), `StockAdjustmentReason`
(Acquisition / WriteOff / Transfer / Stocktake / Manual),
`FineReason` (LateReturn / Lost / Manual), `FineKind`
(FixedAmount / PerDayRate / PercentOfPrice). Plus `MemberId`
(sum type: `Student(StudentId)` / `Staff(StaffId)`), `Isbn` (10
or 13 digits with checksum), `BookTitle`, `BookNumber`, `Author`,
`Publisher`, `Edition`, `RackNumber`, `CategoryName`, `Details`,
`MemberUdId`, `IssueQuantity`, `GivenDate`, `DueDate`,
`ReturnDate`, `IssueNote`, `BookPrice`, `FineAmount`,
`FinePerDay`, `DaysOverdue`, `StockCopies`, `FineSettings`.

**6 `pub trait XxxRepository: Send + Sync` port traits** (one per
aggregate). Object-safety smoke tests in `mod tests`.

**6 typed query stubs** (`BookCategoryQuery`, `BookQuery`,
`LibraryMemberQuery`, `BookIssueQuery`, `BookReturnQuery`,
`FineQuery`) each returning `Err(DomainError::not_supported(...))`
in Phase 9; typed executors land in a follow-up phase alongside
the `#[derive(DomainQuery)]` macro emissions.

**31 unit tests** in `educore-library` (across `value_objects.rs`,
`aggregate.rs`, `entities.rs`, `events.rs`, `services.rs`,
`commands.rs`, `query.rs`, `repository.rs`, `lib.rs`).

### `educore-rbac` integration (Prereq 2A)

**26 `Library.*` `Capability` variants** in
[`Capability`](crates/cross-cutting/rbac/src/value_objects.rs).
The 4 Phase 2 placeholders
(`LibraryBook{Create,Read,Update,Delete}`) were **deduplicated**
during implementation; the canonical
`Book{Add,Read,Update,Delete,AdjustQuantity,Search}` variants
below use the same wire forms
(`Library.Book.{Add,Read,Update,Delete,AdjustQuantity,Search}`) as
the Phase 2 placeholders. Consumers that referenced the
placeholders by name (only `DefaultRoleCatalog::librarian()` in
this workspace) were updated to the new names. This matches
the Phase 8 `FacilitiesRoom*` dedup pattern.

The full 26-variant set:

| Group | Variants | Count |
|---|---|---|
| `Library.*` | `LibraryRead`, `LibraryConfigure`, `LibraryReport` | 3 |
| `BookCategory.*` | `BookCategoryCreate`, `BookCategoryRead`, `BookCategoryUpdate`, `BookCategoryDelete` | 4 |
| `Book.*` | `BookAdd`, `BookRead`, `BookUpdate`, `BookDelete`, `BookAdjustQuantity`, `BookSearch` | 6 |
| `Member.*` | `MemberRegister`, `MemberRead`, `MemberUpdate`, `MemberDelete`, `MemberDeactivate`, `MemberReactivate` | 6 |
| `BookIssue.*` | `BookIssueIssue`, `BookIssueRead`, `BookIssueReturn`, `BookIssueRenew`, `BookIssueMarkLost`, `BookIssueCalculateFine`, `BookIssueWaiveFine` | 7 |

Extended arms: `domain()`, `aggregate()`, `action()`,
`as_str()`, `all()`, `from_str_opt()`. The
`library_capabilities_round_trip_and_resolve_to_library_domain`
test asserts the 26 count. 46 rbac tests pass.

### `educore-audit` integration (Prereq 2B)

**6 `Library` `AuditTarget` variants** in
[`AuditTarget`](crates/cross-cutting/audit/src/writer.rs): the
Phase 2 `Book(Uuid)` placeholder was retained; 5 net-new
variants added — `BookCategory`, `LibraryMember`, `BookIssue`,
`BookReturn`, `Fine`. The `audit_target_type_for_every_variant_is_nonempty`
test covers all 6. 23 audit tests pass.

### `educore-storage-parity` integration test

`crates/tools/storage-parity/tests/library_integration.rs`
mirrors `facilities_integration.rs`. 4 scenarios:

1. **`library_integration_sqlite_vertical_slice`** — subscribe
   to bus → create `BookCategory` + `Book` + `LibraryMember`
   → issue a book → return it 5 days late → build outbox + audit
   + idempotency rows in a single transaction → publish
   envelopes to bus → assert the bus received the first envelope.
2. **`library_capability_check_gates_book_issue`** — assert
   `Capability::BookIssueIssue` is denied by default; grant to
   a school role; assert allowed.
3. **`library_event_type_round_trip_for_all_headline_aggregates`**
   — assert all 18 event types resolve to expected
   `library.<aggregate>.<verb>` strings.
4. **`library_fine_calculation_invariant_holds_for_late_return`**
   — late-fine proptest, return 5 days late, assert
   `fine_amount = 5 * per_day_rate`. Also assert zero-fine for
   on-time return; grace period handling; etc.

### Phase 8 WIP completion (out-of-scope but required)

The Phase 8 WIP was uncommitted when Phase 9 started. The
Phase 9 commit landed the missing Phase 8 deliverables in
`educore-facilities` and `educore-rbac` to make the workspace
buildable:

- **`educore-facilities/src/lib.rs`**: the 27-line scaffold was
  missing the 135-line prelude (`prelude` re-exports of the
  14 aggregates, 10 child entities, 18 events, 13 query
  stubs, 13 repos, 13 service factories, 50+ commands, 50+
  value objects, and the `FacilitiesError`).
- **`educore-facilities/Cargo.toml`**: the 20-line scaffold was
  missing the 18 Phase 8 deps (educore-event-bus,
  educore-audit, educore-storage, educore-hr, async-trait,
  chrono, proptest, rust_decimal, rust_decimal_macros, serde,
  serde_json, thiserror, uuid, plus `tokio` dev-dep).
- **`educore-rbac` Phase 8 capabilities**: the 4 Phase 2
  `FacilitiesRoom*` placeholders and 54 net-new Phase 8
  variants (`FacilitiesVehicle*`, `FacilitiesRoute*`,
  `FacilitiesTransport*`, `FacilitiesDormitory*`,
  `FacilitiesRoom{AssignStudent,UnassignStudent}`,
  `FacilitiesRoomType*`, `FacilitiesItem*`,
  `FacilitiesItemCategory*`, `FacilitiesItemStore*`,
  `FacilitiesInventory*`, `FacilitiesSupplier*`).

These are documented in the Phase 8 handoff but were never
committed to the working tree. Phase 9's foundation cleanup
landed them as part of the Round 1 prereq work to make
`cargo test --workspace` green.

## Cross-crate placeholders

**None to reconcile.** The library spec didn't introduce any
cross-crate placeholders. The only consumer of library types
outside the library crate is the engine knowledge graph
(`graphify-out/`), which references the new `Book*`,
`LibraryMember*`, `BookIssue*`, `Fine*` events.

## Concurrency strategy

Per the build-plan § "Phase 9 Risks", late-fine computation
under concurrent writes is mitigated by the same row-level
lock strategy as Phase 8's inventory conservation:

- `BookRepository::adjust_quantity` uses the atomic conditional
  update:
  ```sql
  UPDATE library_books SET quantity = $new WHERE id = $id
    AND $new >= (SELECT coalesce(sum(quantity), 0) FROM library_book_issues
                 WHERE book_id = $id AND issue_status IN ('Issued','Renewed','Overdue'))
  ```
- The `FineCalculationService` is pure; the dispatcher is
  responsible for acquiring the row-level lock on the
  `library_books` row (PG `SELECT ... FOR UPDATE` or SQLite
  write lock) before calling the service and writing audit /
  outbox / idempotency rows in a single transaction.
- The `BookIssueEligibility` policy enforces the
  `open_issues <= quantity` invariant before the `create_book_issue`
  service is invoked.

## Headline correctness check

The **`FineCalculationService`** proptest (100 cases, matching
Phase 7's `LateFeeService` at
`crates/domains/finance/src/services.rs:1259` and Phase 8's
`InventoryConservationService` at
`crates/domains/facilities/src/services.rs:1435`):

```rust
proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(100))]

    /// Property: the per-day fine is monotonically non-decreasing
    /// in `days_overdue` for any settings with a non-negative
    /// per-day rate.
    #[test]
    fn prop_fine_is_monotonic_in_days_late(
        days_late in 0i64..30,
    ) { ... }

    /// Property: a fixed-amount fine is constant in `days_late`
    /// for any days_late > 0.
    #[test]
    fn prop_fixed_fine_is_constant(
        days_late in 1i64..100,
    ) { ... }
}
```

The 100 cases (50 per case-generator) include both the
"per-day fine is monotonic in days late" and "fixed-amount
fine is constant" branches; both are green.

## Open questions

1. **Fine payment integration** (carry-over from Phase 8 OQ #6
   + Phase 7 Q9) — `FineCalculated` is emitted but the finance
   subscriber isn't wired. Phase 10+ must add a
   `educore-finance` subscriber or a follow-up `educore-payment`
   integration. The library crate does NOT depend on finance
   (per Phase 8 OQ #6 — the no-finance-dep decision). The
   cross-domain coordination is the bus's job.
2. **6-aggregate vs 4-root interpretation** (new) — the prompt
   names 6 headline aggregates (`Book`, `BookCategory`,
   `LibraryMember`, `BookIssue`, `BookReturn`, `Fine`) but the
   spec's `aggregates.md` lists only 4 root aggregates;
   `BookReturn` is modeled as a status transition and `Fine`
   as a child entity (`BookIssueFine`). Phase 9 ships the
   6-aggregate interpretation; `BookReturn` and `Fine` are
   first-class root aggregates with their own typed ids,
   repositories, and primary events. A future phase may
   consolidate `BookReturn` into a child entity if the query
   patterns show it's rarely queried independently.
3. **`FineReason::Manual` flow** (new) — the spec's
   `FineCalculated` event has a `reason: FineReason` field. The
   `WaiveBookIssueFine` flow is a separate `FineWaived` event.
   The exact `Manual` flow (librarian-initiated fine) isn't
   documented in the spec; Phase 9 ships `CalculateFine` with
   `reason = FineCalculated` only. A follow-up phase may add a
   `RecordManualFine` command.
4. **ISBN checksum validation** (new) — the `Isbn` value
   object validates 10 or 13 digits + checksum. The exact
   checksum algorithm is documented at
   `docs/specs/library/value-objects.md`; the integration test
   includes an ISBN-13 round-trip
   (`978-0-13-468599-1` → `9780134685991`).
5. **`BookIssueRenewal` history row** (new) — the spec lists
   it as a child entity; Phase 9 ships it as a child entity
   (not a separate aggregate) per the spec. The repository
   method `BookIssueRepository::append_renewal` is the
   persistence path (deferred to a follow-up phase).
6. **`LibrarySettings` per-school** (carry-over from spec) —
   `LibrarySettings` is owned by the `settings` domain
   (Phase 14). The library crate reads but does not own it.
   Phase 9 ships the `LibrarySettings` value object and the
   `FineSettings` type but no persistence.
7. **`MemberId` sum type** (new) — the `MemberId` enum
   (`Student(StudentId)` / `Staff(StaffId)`) is polymorphic in
   storage (`student_staff_id` column). The storage adapter must
   enforce the invariant. Phase 9 ships the typed enum; the
   storage adapter enforcement is a Phase 15+ concern.

## Where NOT to start (Phase 10)

- Do NOT add `educore-finance` as a dep (Phase 8 OQ #6 carries
  forward — no cross-domain deps; the bus handles the
  `FineCalculated` → `Receivable` cross-domain coordination).
- Do NOT re-implement the 6 library aggregates. They are
  closed in Phase 9. Phase 10 is `educore-communication`
  (notices, complaints, chat messages, email logs, SMS logs,
  notification settings).
- Do NOT add the 33 finance placeholder aggregates as real
  aggregates. They are the Workstreams D-M backlog; the
  per-PR gate validates `Tested` rows, not the absence of
  `Pending` rows. The Phase 8 hand-off resolved the
  facilities-side Q7 (the 2 finance placeholders) by adding
  the canonical `ItemId` re-export. The finance placeholders
  themselves remain `Uuid` and will be replaced in a follow-up
  phase when their real aggregate is built.
- Do NOT touch the 15 closed crates other than the additive
  rbac + audit extensions + the Phase 8 WIP completion
  (facilities prelude + facilities Cargo.toml + facilities
  capabilities) + 1 `Cargo.toml` addition to storage-parity.
  Per `ADR-013-CrateLayout.md`, the cross-crate modifications
  are all non-breaking additive.
- Do NOT touch `educore-core::lint`. The lint binary passes;
  the tier-boundary checker remains a stub.
- Do NOT remove the deprecated `PaymentProvider` trait from
  `educore-finance` (Phase 7 Q10 — moves to `educore-payment`
  in Phase 15).
- Do NOT modify the 4 Phase 2 `LibraryBook*` capability
  placeholders or add them back. They were deduplicated in
  Phase 9.

## Key files for the next agent

- `crates/domains/library/src/value_objects.rs` — 6 root typed
  ids + 2 child ids + `Isbn` (10/13 digit with checksum) +
  12 validated value types + 5 closed enums + `MemberId` sum
  type + `FineKind` + `FineSettings`
- `crates/domains/library/src/aggregate.rs` — 6 root aggregates
  with the 17-field audit-footer pattern + the `BookIssue`
  state machine + the `Book.available_copies` derivation
- `crates/domains/library/src/entities.rs` — 3 headline child
  entities (BookCatalogEntry, BookAcquisition,
  LibraryMemberNote) + 2 spec-mandated (BookIssueRenewal,
  BookIssueFine)
- `crates/domains/library/src/commands.rs` — 18 typed command
  shapes + 18 `LIBRARY_*_COMMAND_TYPE` constants
- `crates/domains/library/src/events.rs` — 18 typed events
  implementing `DomainEvent` (wire form `library.<aggregate>.<verb>`)
- `crates/domains/library/src/services.rs` — 6 pure factory
  service functions + 4 service structs + 3 policies +
  3 specifications + `FineCalculationService` (the headline
  correctness check) with the 100-case proptest
- `crates/domains/library/src/repository.rs` — 6 `pub trait
  XxxRepository: Send + Sync` port traits (object-safety
  smoke tests included)
- `crates/domains/library/src/query.rs` — 6 typed query stubs
  returning `Err(not_supported)` in Phase 9
- `crates/domains/library/src/lib.rs` — the 9-file prelude +
  `PACKAGE_NAME` + `PACKAGE_VERSION`
- `crates/tools/storage-parity/tests/library_integration.rs` —
  the 4-scenario vertical-slice test
- `crates/cross-cutting/rbac/src/value_objects.rs` — the 26
  `Library.*` `Capability` variants (Prereq 2A)
- `crates/cross-cutting/audit/src/writer.rs` — the 6 `Library`
  `AuditTarget` variants (Prereq 2B)
- `crates/cross-cutting/rbac/src/services.rs` — the
  `DefaultRoleCatalog::librarian()` extended with the new
  variants (Prereq 2C)
- `crates/domains/facilities/src/lib.rs` — the Phase 8 prelude
  (WIP completion)
- `crates/domains/facilities/Cargo.toml` — the Phase 8 deps
  (WIP completion)
- `docs/coverage.toml` — 10 rows flipped from `Pending` to
  `Tested` (the prompt's ≥6 target is exceeded)
- `docs/handoff/PHASE-9-HANDOFF.md` — this hand-off
- `docs/phase_prompt/phase-10-prompt.md` — the next-phase brief

## Where to ask

Open a GitHub issue for design questions. The Phase 9 prompt
is the source of truth for Phase 9's scope; the next-phase
prompt is the source of truth for Phase 10's. For disputes,
defer to `AGENTS.md` (engine rules) and `ADR-013-CrateLayout.md`
(tier definitions).
