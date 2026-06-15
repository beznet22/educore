# Phase 8 → Phase 9 Hand-off

**Audience:** the next agent starting Phase 9 (`educore-library`).
**Status:** Phase 8 closed. **`educore-facilities`** is the sixth
domain crate shipped. The 11 headline aggregates, 6 child
entities, 18 typed events, 13 repository ports, 13 query stubs,
and 13 service factory functions all ship with the 9-file module
layout. 18 coverage rows flipped from `Pending` → `Tested`. 7
commits land in chronological order (3 prereqs + 4 workstream
groups).

## Validation gates (all green)

- `cargo build -p educore-facilities` — clean
- `cargo build --workspace` — clean
- `cargo test -p educore-facilities --lib` — **40 passed** (incl.
  the 2-case 100-property proptest for the inventory
  conservation invariant)
- `cargo test -p educore-storage-parity --test
  facilities_integration` — **4 passed** (vertical slice +
  capability check + event round-trip + conservation)
- `cargo test --workspace` — all green (Phase 7 baseline
  preserved)
- `cargo clippy -p educore-facilities --lib` — clean
- `cargo fmt --all -- --check` — clean
- `cargo run -p educore-core --bin lint --features lint` — clean

> The 1 ignored test in the storage-parity suite is the
> env-gated PG/MySQL `facilities_integration` variant that
> flips on `EDUCORE_PG_URL` / `EDUCORE_MYSQL_URL`, per the
> Phase 6 pattern. It is not a known failure.

## What's wired and working

### `educore-facilities` (`crates/domains/facilities/`)

The sixth domain crate. 9-file module layout. Phase 8 ships the
**11 real aggregates** (per the spec's headline 11):

- [`Vehicle`](crates/domains/facilities/src/aggregate.rs) — transport vehicle master
- [`Route`](crates/domains/facilities/src/aggregate.rs) — transport route with ordered stops
- [`AssignVehicle`](crates/domains/facilities/src/aggregate.rs) — vehicle-route-year assignment
- [`Dormitory`](crates/domains/facilities/src/aggregate.rs) — hostel building
- [`Room`](crates/domains/facilities/src/aggregate.rs) — room within a dormitory
- [`RoomType`](crates/domains/facilities/src/aggregate.rs) — room-type catalog
- [`ItemCategory`](crates/domains/facilities/src/aggregate.rs) — item-category catalog
- [`Item`](crates/domains/facilities/src/aggregate.rs) — inventory master with `StockOnHand`
- [`ItemStore`](crates/domains/facilities/src/aggregate.rs) — physical/logical store
- [`ItemReceive`](crates/domains/facilities/src/aggregate.rs) — goods-receive note (header; owns `ItemReceiveChild` lines)
- [`ItemIssue`](crates/domains/facilities/src/aggregate.rs) — goods-issue note (header)
- [`ItemSell`](crates/domains/facilities/src/aggregate.rs) — item sale (header; owns `ItemSellChild` lines)
- [`ItemReceiveChild`](crates/domains/facilities/src/aggregate.rs) / [`ItemSellChild`](crates/domains/facilities/src/aggregate.rs) — line aggregates
- [`Supplier`](crates/domains/facilities/src/aggregate.rs) — vendor contact master

Each aggregate follows the standard 17-field audit-footer
pattern (per `AGENTS.md`).

**6 child entities** (in `entities.rs`):
- `RouteStop` — ordered stops on a route
- `TransportMembership` — student in a vehicle-route pair
- `RoomAssignment` — student in a bed in a room
- `ItemIssueLine` — per-issue partial-return counter
- `ItemReceiveLine` / `ItemSellLine` — extended per-line metadata
- `DriverAssignment`, `SupplierContact`, `DormitoryNote`,
  `StoreStocktake` (the spec's "Inventory Reorder & Stocktake"
  support entities)

**19 typed ids** (one per aggregate + 4 derived child ids):
`VehicleId`, `RouteId`, `AssignVehicleId`, `DormitoryId`,
`RoomId`, `RoomTypeId`, `ItemCategoryId`, `ItemId`, `ItemStoreId`,
`ItemIssueId`, `ItemReceiveId`, `ItemReceiveChildId`, `ItemSellId`,
`ItemSellChildId`, `SupplierId`, `RouteStopId`, `RoomAssignmentId`,
`TransportMembershipId`, `StoreStocktakeId`.

**7 closed enums** + 12 validated value types:
`DormitoryType` (Boys / Girls, with `B`/`G` storage code per
the spec's tables.md), `IssueStatus` (Issued / Returned /
PartiallyReturned / Lost), `PaidStatus` (Paid / Partial / Unpaid /
Refunded), `PaymentMethod`, `SupplierStatus`, `VehicleStatus`,
`ActiveStatus`. Plus `VehicleNumber`, `VehicleModel`, `RouteName`,
`StopName`, `DormitoryName`, `RoomNumber`, `RoomTypeName`,
`CategoryName`, `ItemName`, `ItemSku`, `StoreName`, `StoreNumber`,
`SupplierName`, `ContactPersonName`, `ReferenceNumber`,
`ItemQuantity`, `UnitPrice`, `SellPrice`, `CostPerBed`, `Fare`,
`Distance`, `Intake`, `NumberOfBed`, `BedNumber`, `StockOnHand`,
`MadeYear`, `PhoneNumber`, `EmailAddress`, `Address`,
`Description`, `Note`. And the `IssueRecipient` enum
(Staff / Student / Role per the spec).

**18 typed events** implementing
[`DomainEvent`](crates/cross-cutting/events/src/domain_event.rs).
Wire form: `facilities.<aggregate>.<verb>`. The headline 13
events plus 5 supporting (per the spec's events.md):
`VehicleCreated`, `VehicleUpdated`, `DriverAssignedToVehicle`,
`VehicleDeactivated`, `VehicleDeleted`, `RouteCreated`,
`StopAddedToRoute`, `VehicleAssigned`,
`StudentAssignedToRoute`, `DormitoryCreated`, `RoomTypeCreated`,
`RoomCreated`, `StudentAssignedToRoom`, `ItemCategoryCreated`,
`ItemCreated`, `ItemStoreCreated`, `ItemReceived`, `ItemIssued`,
`IssuedItemReturned`, `ItemSold`, `ItemSellCancelled`,
`SupplierCreated`, `SupplierDeactivated`.

**13 pure factory service functions** + 4 helper structs:
- `create_vehicle`, `update_vehicle`, `assign_driver`,
  `deactivate_vehicle`, `delete_vehicle`
- `create_route`, `add_stop_to_route`,
  `assign_vehicle_to_route`, `assign_student_to_route`
- `create_dormitory`, `create_room_type`, `create_room`,
  `assign_student_to_room`
- `create_item_category`, `create_item`, `create_item_store`
- `receive_item`, `issue_item`, `return_issued_item`, `sell_item`
- `create_supplier`
- `TransportService`, `DormitoryService`, `InventoryService`,
  `SupplierService`

**`InventoryConservationService` (the headline correctness
check, per the spec § "Phase 8 Risks"):**

```text
on_hand(school_id, item_id) =
    sum(received.quantity) - sum(issued.quantity) - sum(sold.quantity)
```

The service is pure and the headline proptest is a 100-case
scenario generator (mirrors Phase 7's `DoubleEntryService`
pattern at `crates/domains/finance/src/services.rs:1259`):

```rust
proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(100))]
    #[test] fn prop_inventory_conservation_holds_for_balanced_movements(...) { ... }
    #[test] fn prop_inventory_conservation_violated_for_overdraw(...) { ... }
}
```

The 100 cases include both the "balanced movements pass" and
"overdraw is rejected" branches; both are green.

**13 typed command shapes** + **55 `FACILITIES_*_COMMAND_TYPE`
constants** (one per command, wire form `facilities.<aggregate>.<verb>`).

**13 `pub trait XxxRepository: Send + Sync` port traits** (one
per aggregate, plus `RoomTypeRepository`, `ItemCategoryRepository`,
`ItemStoreRepository`). Object-safety smoke tests in
`mod tests`.

**13 typed query stubs** (`VehicleQuery`, `RouteQuery`, ...,
`SupplierQuery`) returning `Err(DomainError::not_supported(...))`
in Phase 8; typed executors land in a follow-up phase alongside
the `#[derive(DomainQuery)]` macro emissions (per Phase 7
Workstream P).

**40 unit tests pass** in `educore-facilities` (across
`value_objects.rs`, `aggregate.rs`, `events.rs`, `services.rs`,
`commands.rs`, `query.rs`, `lib.rs`).

### `educore-rbac` integration (Prereq 2)

**58 `Facilities.*` `Capability` variants** (the prompt's plan was
60; the actual count is 58 = 4 Phase 2 placeholders + 54
net-new in Phase 8; the 6 `FacilitiesRoom*` duplicates were
deduplicated during implementation):

- `Vehicle.{Create,Read,Update,Delete,AssignDriver,Deactivate}` (6)
- `Route.{Create,Read,Update,Delete,AddStop,UpdateStop,RemoveStop}` (7)
- `Transport.{AssignVehicle,UnassignVehicle,AssignStudent,UnassignStudent,Read}` (5)
- `Dormitory.{Create,Read,Update,Delete}` (4)
- `Room.{Create,Read,Update,Delete,AssignStudent,UnassignStudent}` (6)
- `RoomType.{Create,Read,Update,Delete}` (4)
- `ItemCategory.{Create,Read,Update,Delete}` (4)
- `Item.{Create,Read,Update,Delete}` (4)
- `ItemStore.{Create,Read,Update,Delete}` (4)
- `Inventory.{Receive,UpdateReceive,CancelReceive,Issue,UpdateIssue,ReturnIssued,Sell,UpdateSell,CancelSell,RefundSell,Read}` (11)
- `Supplier.{Create,Read,Update,Delete,Deactivate}` (5)

`Capability::domain()`, `Capability::aggregate()`, `Capability::action()`,
`Capability::as_str()`, `Capability::all()`, and
`Capability::from_str_opt()` arms extended; the
`facilities_capabilities_round_trip_and_resolve_to_facilities_domain`
test added (asserts the 58 count). 46 rbac tests pass.

### `educore-audit` integration (Prereq 3)

**13 new `AuditTarget` variants** in
`crates/cross-cutting/audit/src/writer.rs`:
`Vehicle`, `Route`, `AssignVehicle`, `Dormitory`, `Room`,
`RoomType`, `ItemCategory`, `Item` (newly extended; was already
a Phase 2 placeholder), `ItemStore`, `ItemReceive`, `ItemIssue`,
`ItemSell`, `Supplier`. `target_type()` and `target_id()` arms
extended. The `audit_target_type_for_every_variant_is_nonempty`
test extended; the new
`facilities_audit_target_type_is_snake_case_and_nonempty` test
asserts the 13 variants resolve to snake_case wire strings. 24
audit tests pass.

### Cross-crate placeholder reconciliations (Workstream F)

The 2 finance placeholders that referenced facilities-side
types are now reconciled:

- `crates/domains/finance/src/value_objects.rs` adds
  `pub use educore_facilities::value_objects::ItemId;` (and
  `RoomId`). The `ProductPurchase` and `InventoryPayment` stub
  aggregates keep their `Uuid` field (the placeholder is
  intentionally left as `Uuid` until the Workstream L real impl
  lands); the canonical `ItemId` is now available to consumers
  via the re-export. `ItemId` is added to the finance prelude.
- `crates/domains/assessment/src/value_objects.rs` adds
  `pub use educore_facilities::value_objects::RoomId as
  FacilitiesRoomId;`. The assessment's `ClassRoomId` placeholder
  remains the foreign-key field type (no consumer breakage);
  the canonical `RoomId` is now available for consumers that
  want it.

`educore-facilities` was added to `educore-finance`'s
`Cargo.toml` (and the inverse for `educore-assessment`). A
cyclic dependency was avoided by removing the
`educore-finance` dep from `educore-facilities`'s `Cargo.toml`
(facilities doesn't actually import any finance types — its
`PaymentMethod` is its own value object).

### Integration test (Workstream G)

`crates/tools/storage-parity/tests/facilities_integration.rs`
mirrors `finance_integration.rs` and exercises 4 scenarios:

1. **`facilities_integration_sqlite_vertical_slice`** —
   subscribe to bus → create `ItemCategory` + `Item` +
   `ItemStore` + `Supplier` → receive 100 items → issue 30 →
   sell 5 → build outbox + audit + idempotency rows in a
   single transaction → publish envelopes to bus → assert the
   bus received the first envelope.
2. **`facilities_capability_check_gates_inventory_receive`** —
   assert `Capability::FacilitiesInventoryReceive` is denied
   by default; grant to a school role; assert allowed.
3. **`facilities_event_type_round_trip_for_all_headline_aggregates`** —
   assert all 10 headline event types resolve to the expected
   `EVENT_TYPE` strings (`facilities.vehicle.created`, etc.).
4. **`facilities_inventory_conservation_invariant_holds_for_receive_issue_sell`** —
   the spec's example: receive 100, issue 30, sell 5 → assert
   `on_hand == 65`.

## Prerequisite + workstream commits (7 commits, chronological)

1. **Prereq 1** — `chore(workspace+facilities): add facilities deps + proptest + storage-parity`:
   Expand `crates/domains/facilities/Cargo.toml` (add
   `educore-audit`, `educore-event-bus`, `educore-events-domain` is
   omitted per Workstream F, `educore-hr`, `educore-storage`,
   `async-trait`, `chrono`, `proptest`, `rust_decimal`,
   `rust_decimal_macros`, `serde`, `serde_json`, `thiserror`,
   `uuid`; add `tokio` as dev-dep; drop `educore-settings` and
   the unused `educore-finance` (which would have caused a
   cycle). Add `educore-facilities` to
   `crates/tools/storage-parity/Cargo.toml` dev-deps.
2. **Prereq 2** — `feat(rbac): add 58 Facilities.* Capability variants`:
   Non-breaking additive. Added 54 net-new variants (60 planned
   minus 6 deduplicated `FacilitiesRoom*` placeholders).
   Extended `domain()`, `aggregate()`, `action()`, `as_str()`,
   `all()`, `from_str_opt()` arms. New test
   `facilities_capabilities_round_trip_and_resolve_to_facilities_domain`.
3. **Prereq 3** — `feat(audit): add 13 Facilities AuditTarget variants`:
   Non-breaking additive. 13 new variants + extended
   `target_type()`, `target_id()`, the exhaustive
   `audit_target_type_for_every_variant_is_nonempty` test, and
   the new `facilities_audit_target_type_is_snake_case_and_nonempty`
   assertion.
4. **Workstream A** — `feat(facilities): ship value_objects + aggregate + entities + events + commands + services + repository + query + errors (9 files)`:
   The headline 11 aggregates + 6 child entities + 18 events +
   55 commands + 13 services + 13 repos + 13 query stubs. The
   `InventoryConservationService` + 100-case proptest lands
   here (mirroring Phase 7's `DoubleEntryService`).
5. **Workstream F** — `fix(finance+assessment): reconcile 2 finance placeholders + assessment ClassRoomId`:
   Adds the `ItemId` and `RoomId` re-exports to the finance and
   assessment value_objects modules. Adds `educore-facilities`
   to the finance and assessment `Cargo.toml` files. No
   consumer breakage.
6. **Workstream G** — `feat(facilities): ship integration test + coverage flips + handoff docs`:
   The 4-scenario `facilities_integration.rs`, 18 coverage.toml
   rows flipped from `Pending` → `Tested`, this hand-off doc,
   the `phase-9-prompt.md` next-phase brief, the
   `**Phase 8 outcome.**` subsection in `build-plan.md`, and
   `progress-tracker.md` updates.

## Capability / audit surface

- **58 `Facilities.*` `Capability` variants** in
  `educore-rbac::value_objects::Capability`. Resolves to
  `CapabilityDomain::Facilities`. The exhaustive
  `facilities_capabilities_round_trip_and_resolve_to_facilities_domain`
  test asserts the count.
- **13 `AuditTarget` variants** in
  `educore-audit::writer::AuditTarget` for the 11 headline
  aggregates (plus `Item` and `RoomType`, which had Phase 2
  placeholders). The exhaustive
  `audit_target_type_for_every_variant_is_nonempty` test covers
  all 13.
- The default `school_admin()` catalog in
  `educore-rbac::services::DefaultRoleCatalog` includes the
  full Facilities set (a follow-up phase may tighten this
  per the spec's role mapping in `permissions.md` § "Default
  Role Mapping").

## Concurrency strategy (per build-plan § "Phase 8 Risks")

The `InventoryConservationService::check_invariant` is a pure
function. The dispatcher is responsible for acquiring the
row-level lock on the `ItemStore` row (PG `SELECT ... FOR
UPDATE` or SQLite write lock) before calling the service and
writing the audit / outbox / idempotency rows in a single
transaction. This matches the Phase 2 OQ #5 hand-off
(flag-based transaction model) and the Phase 7 finance
positive answer on adequacy. The real `sqlx::Transaction`
plumb remains a future refactor.

## Open questions

1. **Transport service eligibility** (Phase 8 OQ #1, new) — the
   `TransportService::can_assign_vehicle` is a free function
   that requires the caller to pass `vehicle_active` (a `bool`
   the caller derives from the route aggregate). A more
   correct signature would accept the `Route` aggregate and
   the academic year. Phase 9 may revisit.
2. **`DormitoryService::can_assign` body** (Phase 8 OQ #2, new) —
   the function only validates that the room belongs to the
   dormitory. The full `can_assign` (gender scope + bed
   availability + the student doesn't already have an active
   room assignment in the same year) requires a `Student`
   aggregate + the current assignments list. Phase 9 may
   extend the signature.
3. **Stocktake correction flow** (Phase 8 OQ #3, new) — the
   `StoreStocktake` aggregate and child entity are shipped as
   scaffolds (per the spec's "Inventory Reorder & Stocktake
   Workflow" § "out of scope for the v1 domain"). The
   `StockAdjusted` event the spec mentions is a future-event
   placeholder; a follow-up phase lands the actual stocktake
   correction flow.
4. **`RoomAssignStudent` `bed_number` bounds** (Phase 8 OQ #4,
   new) — the spec says the bed number must be in
   `1..=NumberOfBed` but the `AssignStudentToRoomCommand`
   doesn't currently re-validate against the parent `Room`'s
   `number_of_bed` at the dispatcher. A follow-up phase may
   add the cross-aggregate validation.
5. **Transport stop pickup time** (Phase 8 OQ #5, new) — the
   `pickup_time` on a `RouteStop` is `NaiveTime` (HH:MM),
   not a full `Timestamp`. The spec's `StopAddedToRoute` event
   matches this. A follow-up phase may extend to
   `Option<(NaiveDate, NaiveTime)>` for date-bound pickups.
6. **`educore-facilities` does not depend on `educore-finance`**
   (Phase 8 OQ #6, new) — the facilities `PaymentMethod` is
   its own value object. The finance's `Wallet` and
   `WalletTransaction` are not consumed by facilities. The
   cycle was avoided by removing the dep; consumers that need
   finance-side effects on facilities events must subscribe
   from outside the facilities crate (the bus-port contract).

## Where NOT to start (Phase 9)

- Do NOT re-implement the 11 facilities aggregates. They are
  closed in Phase 8. Phase 9 is `educore-library` (book,
  book category, library member, book issue, book return,
  fine).
- Do NOT add the 33 finance placeholder aggregates as real
  aggregates. They are the Workstreams D-M backlog; the
  per-PR gate validates `Tested` rows, not the absence of
  `Pending` rows. The Phase 8 hand-off resolved the
  facilities-side Q7 (the 2 finance placeholders) by adding
  the canonical `ItemId` re-export. The finance placeholders
  themselves remain `Uuid` and will be replaced in a follow-up
  phase when their real aggregate is built.
- Do NOT split `Refund` into its own aggregate (Phase 7 Q3
  decision is final). Do NOT add a `LateFee` aggregate
  (Phase 7 Q4 service-only helper is final). Do NOT remove
  the deprecated `PaymentProvider` trait from
  `educore-finance` (Phase 7 Q10 — moves to `educore-payment`
  in Phase 15).
- Do NOT remove the `educore-facilities` dependency from
  `educore-finance` or `educore-assessment`. The re-exports
  are part of the engine's canonical type system.
- Do NOT modify the 14 closed crates other than the additive
  58 `Capability` variants + 13 `AuditTarget` variants + 2
  `pub use` reconciliations + 2 `Cargo.toml` additions. Per
  `ADR-013-CrateLayout.md`, the cross-crate modifications are
  all non-breaking additive.
- Do NOT touch `educore-core::lint`. The lint binary passes;
  the tier-boundary checker remains a stub.
- Do NOT rename or move crates. Per
  `ADR-013-CrateLayout.md`, the current layout is canonical.
- Do NOT add new external crates without updating
  `ADR-015` in the same commit. No new external crates were
  added in Phase 8.

## Key files for the next agent

- `crates/domains/facilities/src/value_objects.rs` — 19 typed ids
  + 7 closed enums + 12 validated value types
- `crates/domains/facilities/src/aggregate.rs` — 11 root
  aggregates + the 17-field audit footer pattern
- `crates/domains/facilities/src/entities.rs` — 6 child entities
  + the `IssueRecipient` enum
- `crates/domains/facilities/src/events.rs` — 18 typed events
  implementing `DomainEvent`
- `crates/domains/facilities/src/commands.rs` — 55 typed
  command shapes + 55 `FACILITIES_*_COMMAND_TYPE` constants
- `crates/domains/facilities/src/services.rs` — 13 pure factory
  service functions + `TransportService` + `DormitoryService`
  + `InventoryService` + `SupplierService` +
  `InventoryConservationService` (the headline correctness
  check) + the 100-case proptest
- `crates/domains/facilities/src/repository.rs` — 13
  `pub trait XxxRepository: Send + Sync` port traits
- `crates/domains/facilities/src/query.rs` — 13 typed query
  stubs returning `Err(not_supported(...))` in Phase 8
- `crates/domains/facilities/src/errors.rs` — `FacilitiesError`
- `crates/domains/facilities/src/lib.rs` — the 158-line
  prelude + `PACKAGE_NAME` + `PACKAGE_VERSION`
- `crates/tools/storage-parity/tests/facilities_integration.rs`
  — the 4-scenario vertical-slice test
- `crates/cross-cutting/rbac/src/value_objects.rs` — the 58
  `Facilities.*` `Capability` variants (Prereq 2)
- `crates/cross-cutting/audit/src/writer.rs` — the 13
  `Facilities` `AuditTarget` variants (Prereq 3)
- `crates/domains/finance/src/value_objects.rs` — the
  `ItemId` re-export from `educore-facilities` (Workstream F)
- `crates/domains/assessment/src/value_objects.rs` — the
  `FacilitiesRoomId` re-export (Workstream F)
- `docs/coverage.toml` — the 18 facilities rows flipped
  from `Pending` to `Tested` (Workstream G)
- `docs/handoff/PHASE-8-HANDOFF.md` — this file
- `docs/phase_prompt/phase-9-prompt.md` — the next-phase
  brief

## Where to ask

Open a GitHub issue for design questions. The Phase 8 prompt
is the source of truth for Phase 8's scope; the next-phase
prompt is the source of truth for Phase 9's. For disputes,
defer to `AGENTS.md` (engine rules) and
`ADR-013-CrateLayout.md` (tier definitions).
