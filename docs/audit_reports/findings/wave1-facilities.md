## Wave 1 Domain Audit Report — `educore-facilities`

**Scope:** `crates/domains/facilities/`, `docs/specs/facilities/`
(`overview.md`, `aggregates.md`, `entities.md`, `value-objects.md`,
`commands.md`, `events.md`, `services.md`, `permissions.md`,
`repositories.md`, `workflows.md`, `tables.md`), `docs/coverage.toml`
(`facilities_items_aggregate` row), `AGENTS.md` (the facilities row
in the Crate Inventory), the four storage adapter crates
(`educore-storage-postgres`, `educore-storage-mysql`,
`educore-storage-sqlite`, `educore-storage-surrealdb`), and
`educore-rbac` capability table.

**Total findings:** 32

---

### FINDING 1

- **id:** DOM-FAC-001
- **area:** domains-facilities
- **severity:** Critical
- **location:** `crates/domains/facilities/` (no `tests/` directory)
- **description:** No `tests/` directory exists under
  `crates/domains/facilities/`. `docs/build-plan.md:1860` (and the
  spec's `tests/workflows.rs` / `tests/commands.rs` /
  `tests/events.rs` / `tests/services.rs` / `tests/repository.rs`
  requirement) mandate a per-domain integration-test suite. Phase 8
  ships zero integration tests for facilities: no workflow test, no
  command test, no event test, no repository test.
- **expected:** `crates/domains/facilities/tests/{workflows,commands,events,services,repository}.rs`
  present (cf. `AGENTS.md` Validation Checklist: "At least one
  integration test added for new behavior").
- **evidence:** `ls -la crates/domains/facilities/tests/` returns
  `NO TESTS DIR` (verified by `ls -la crates/domains/facilities/tests/`
  in the audit session).

---

### FINDING 2

- **id:** DOM-FAC-002
- **area:** domains-facilities
- **severity:** Critical
- **location:** `crates/domains/facilities/src/events.rs` (23 event structs)
- **description:** Only 23 of the 49 events listed in
  `docs/specs/facilities/events.md` are implemented. The 26 missing
  events cover Update/Delete on every aggregate except Vehicle: no
  `RouteUpdated`, `StopUpdatedOnRoute`, `StopRemovedFromRoute`,
  `RouteDeleted`, `VehicleUnassigned`,
  `StudentUnassignedFromRoute`, `DormitoryUpdated`,
  `DormitoryDeleted`, `RoomTypeUpdated`, `RoomTypeDeleted`,
  `RoomUpdated`, `RoomDeleted`,
  `StudentUnassignedFromRoom`, `ItemCategoryUpdated`,
  `ItemCategoryDeleted`, `ItemUpdated`, `ItemDeleted`,
  `ItemStoreUpdated`, `ItemStoreDeleted`, `ItemReceiveUpdated`,
  `ItemReceiveCancelled`, `ItemIssueStatusUpdated`,
  `ItemSellUpdated`, `ItemSellRefunded`, `SupplierUpdated`,
  `SupplierDeleted`. Of these, the `StudentUnassignedFromRoute`,
  `StudentUnassignedFromRoom`, `ItemReceiveCancelled`,
  `ItemSellRefunded`, `SupplierUpdated`, `SupplierDeleted`, and
  `ItemIssueStatusUpdated` events are load-bearing for the spec's
  workflows (transport teardown, dormitory release, inventory
  reversal, finance credit, supplier teardown, lost-asset
  reporting).
- **expected:** 49 `pub struct` event structs, each implementing
  `DomainEvent`, one per spec entry in
  `docs/specs/facilities/events.md`.
- **evidence:** `crates/domains/facilities/src/events.rs` line
  ranges (counts): `pub struct VehicleCreated (line 39)`,
  `VehicleUpdated (91)`, `DriverAssignedToVehicle (139)`,
  `VehicleDeactivated (188)`, `VehicleDeleted (237)`,
  `RouteCreated (284)`, `StopAddedToRoute (336)`,
  `VehicleAssigned (385)`, `StudentAssignedToRoute (434)`,
  `DormitoryCreated (487)`, `RoomTypeCreated (539)`,
  `RoomCreated (585)`, `StudentAssignedToRoom (640)`,
  `ItemCategoryCreated (696)`, `ItemCreated (748)`,
  `ItemStoreCreated (797)`, `ItemReceived (848)`,
  `ItemIssued (918)`, `IssuedItemReturned (976)`,
  `ItemSold (1029)`, `ItemSellCancelled (1096)`,
  `SupplierCreated (1148)`, `SupplierDeactivated (1194)` —
  exactly 23 event structs. `grep -E "^pub struct [A-Z]"
  crates/domains/facilities/src/events.rs` lists the same 23.

---

### FINDING 3

- **id:** DOM-FAC-003
- **area:** domains-facilities
- **severity:** Critical
- **location:** `crates/domains/facilities/src/services.rs:81-956`
- **description:** Only 20 of the 49 spec command handlers
  (factory functions) are implemented in `services.rs`. The 29
  missing factory functions block Update/Delete on every
  aggregate, plus all transport-teardown and inventory-correction
  flows. Specifically missing: `update_route`,
  `update_stop_on_route`, `remove_stop_from_route`, `delete_route`,
  `unassign_vehicle_from_route`, `unassign_student_from_route`,
  `update_room_type`, `delete_room_type`, `update_dormitory`,
  `delete_dormitory`, `update_room`, `delete_room`,
  `unassign_student_from_room`, `update_item_category`,
  `delete_item_category`, `update_item`, `delete_item`,
  `update_item_store`, `delete_item_store`,
  `update_item_receive`, `cancel_item_receive`,
  `update_issue_status`, `update_item_sell`, `cancel_item_sell`,
  `refund_item_sell`, `update_supplier`,
  `deactivate_supplier`, `delete_supplier`, `delete_vehicle`.
- **expected:** 49 service factory functions, one per
  spec command in `docs/specs/facilities/commands.md`.
- **evidence:** `grep -nE "^pub fn [a-z_]+" crates/domains/facilities/src/services.rs`
  returns 20 factory functions (lines 81, 120, 164, 189, 213, 252,
  287, 324, 359, 399, 434, 476, 503, 536, 574, 627, 713, 763,
  827, 914). `crates/domains/facilities/src/commands.rs` defines
  49 command structs (`grep -cE "^pub struct [A-Z][a-zA-Z]+Command"
  crates/domains/facilities/src/commands.rs` = 49), so 29 commands
  have no factory function.

---

### FINDING 4

- **id:** DOM-FAC-004
- **area:** domains-facilities
- **severity:** Critical
- **location:** `crates/domains/facilities/src/entities.rs` (entire file)
- **description:** Zero entities carry the `#[derive(DomainQuery)]`
  macro. The spec at `docs/build-plan.md:172` and the
  `entities.md` "Compile-time safety over strings" rule (cf.
  `AGENTS.md` Engine Rules #2 and #5) mandate the macro on every
  entity/aggregate root so the query builder can be
  macro-generated. The audit session confirms
  `grep -rn "DomainQuery\|#\[derive(DomainQuery)\]"
  crates/domains/facilities/src/` returns only doc-comment
  references and a stub message in `query.rs:51` —
  `"VehicleQuery::execute is a Phase 8 stub; real executor
  lands with the DomainQuery macro"`.
- **expected:** `#[derive(DomainQuery)]` on `Vehicle`, `Route`,
  `AssignVehicle`, `Dormitory`, `Room`, `RoomType`,
  `ItemCategory`, `Item`, `ItemStore`, `ItemReceive`,
  `ItemReceiveChild`, `ItemIssue`, `ItemSell`, `ItemSellChild`,
  `Supplier` (15 aggregates) and the 11 spec entities.
- **evidence:** `crates/domains/facilities/src/query.rs:9-12`:
  ```rust
  //! `#[derive(DomainQuery)]` macro emissions (per the Phase 7
  //! Section 4 Query Layer plan). The 13 query stubs below are
  //! hand-written placeholders; the macro will replace them.
  ```
  No `#[derive(DomainQuery)]` attribute exists in
  `crates/domains/facilities/src/aggregate.rs` (verified by
  `grep -nE "#\[derive\([^)]*DomainQuery[^)]*\)\]"
  crates/domains/facilities/src/aggregate.rs` returning no rows).

---

### FINDING 5

- **id:** DOM-FAC-005
- **area:** domains-facilities
- **severity:** Critical
- **location:** `crates/adapters/storage-postgres/src/storage.rs`,
  `crates/adapters/storage-mysql/src/storage.rs`,
  `crates/adapters/storage-sqlite/src/storage.rs`,
  `crates/adapters/storage-surrealdb/src/storage.rs`
- **description:** None of the four storage adapters reference
  `facilities` or emit the 23 facility tables enumerated in
  `docs/specs/facilities/tables.md`
  (`facilities_vehicles`, `facilities_routes`,
  `facilities_route_stops`, `facilities_assign_vehicles`,
  `facilities_transport_memberships`, `facilities_dormitories`,
  `facilities_room_types`, `facilities_rooms`,
  `facilities_room_assignments`, `facilities_item_categories`,
  `facilities_items`, `facilities_item_stores`,
  `facilities_item_issues`, `facilities_item_receives`,
  `facilities_item_receive_children`, `facilities_item_sells`,
  `facilities_item_sell_children`, `facilities_suppliers`,
  `facilities_supplier_contacts`, `facilities_driver_assignments`,
  `facilities_store_stocktakes`, `facilities_store_stocktake_lines`,
  `facilities_dormitory_notes`). No DDL is generated for facilities
  at startup, so `storage.create_schema().await` will not create
  any facility table.
- **expected:** Each adapter walks the macro-emitted AST
  (`docs/schemas/sql-dialects/README.md` Runtime DDL emission §
  Step 4) and emits per-domain tables, including all 23 facility
  tables, on `create_schema()`.
- **evidence:** `grep -rln "facilities" crates/adapters/` returns
  no rows. `ls crates/adapters/storage-postgres/src/` shows only
  `audit_log.rs`, `bulk_attendance.rs`, `connection_helpers.rs`,
  `connection.rs`, `error.rs`, `event_log.rs`, `idempotency.rs`,
  `lib.rs`, `outbox.rs`, `storage.rs`, `transaction.rs` — no
  domain-table emitter.

---

### FINDING 6

- **id:** DOM-FAC-006
- **area:** domains-facilities
- **severity:** High
- **location:** `crates/domains/facilities/src/services.rs:958-1083`
- **description:** Domain services are skeletal. `TransportService`
  has only 2 of 5 spec methods (`can_assign_vehicle`,
  `fare_for_student`); missing `validate_vehicle_for_year`,
  `plan_route_distance`, `is_within_capacity`.
  `DormitoryService` has 2 of 4 (`available_beds`, `can_assign`);
  missing `occupancy`, `default_room_type_for`.
  `InventoryService` has 3 of 7 (`validate_receive`,
  `validate_sell`, `validate_issue`); missing
  `total_quantity_for`, `grand_total_for`, `paid_status_for`,
  `apply_return`. `SupplierService` has 1 of 3 (`normalize_name`);
  missing `can_delete`, `find_duplicates`. The 4 spec policies
  (`VehicleAssignmentEligibility`, `IssueAuthorization`,
  `ActiveRoutesInYear`, `LowStockItems`, `AvailableBeds`) are
  absent entirely.
- **expected:** Every method in
  `docs/specs/facilities/services.md` plus the 5 spec
  policy/specification objects.
- **evidence:** `grep -nE "validate_vehicle_for_year|plan_route_distance|fare_for_student|is_within_capacity|occupancy|default_room_type_for|paid_status_for|apply_return|can_delete|find_duplicates"
  crates/domains/facilities/src/services.rs` returns exactly one
  hit: `pub fn fare_for_student(route_fare: Fare, stop_override: Option<Fare>) -> Fare` (line 973). The other 10 spec
  methods and 5 spec policies are not present.

---

### FINDING 7

- **id:** DOM-FAC-007
- **area:** domains-facilities
- **severity:** High
- **location:** `crates/domains/facilities/src/commands.rs`
  (entire file) and `crates/domains/facilities/src/services.rs`
- **description:** Command structs in `commands.rs` carry
  capability string literals or no capability annotation at all;
  no command handler performs the `has_capability(...)` check that
  the spec mandates in `permissions.md`:
  `if !engine.rbac().has(actor_id, Capability::InventoryReceive).await? { ... }`.
  The `services.rs` factory functions do not call any RBAC method
  (`grep -nE "Capability::|has_capability|capability|authorize"
  crates/domains/facilities/src/*.rs` returns no rows).
- **expected:** Every service factory function takes a
  capability-checked actor as its first step, prior to aggregate
  mutation.
- **evidence:** `crates/domains/facilities/src/services.rs:81` —
  `pub fn create_vehicle<C, G>(...)` body opens with
  `let tenant = ...; let vehicle = Vehicle::fresh(...)`; no
  authorization step.

---

### FINDING 8

- **id:** DOM-FAC-008
- **area:** domains-facilities
- **severity:** High
- **location:** `crates/domains/facilities/src/repository.rs`
  (entire file)
- **description:** The 13 repository port traits exist
  (`VehicleRepository`, `RouteRepository`, `AssignVehicleRepository`,
  `DormitoryRepository`, `RoomRepository`, `RoomTypeRepository`,
  `ItemCategoryRepository`, `ItemRepository`, `ItemStoreRepository`,
  `ItemIssueRepository`, `ItemReceiveRepository`, `ItemSellRepository`,
  `SupplierRepository`), but no storage adapter implements them.
  None of the 4 adapter crates (`educore-storage-postgres`,
  `-mysql`, `-sqlite`, `-surrealdb`) references `VehicleRepository`
  or any other facilities port trait.
- **expected:** Each adapter implements every facilities port trait
  on its connection pool, with `school_id = $1` predicate
  rewriting per `repositories.md` "Tenant isolation".
- **evidence:** `grep -rln "VehicleRepository\|DormitoryRepository\|ItemReceiveRepository"
  crates/adapters/` returns no rows.

---

### FINDING 9

- **id:** DOM-FAC-009
- **area:** domains-facilities
- **severity:** High
- **location:** `crates/domains/facilities/src/events.rs`
  (DomainEvent impls across the file)
- **description:** Spec at `docs/specs/facilities/events.md`
  mandates `const TYPE: &'static str;` on the `DomainEvent` trait.
  The actual engine trait (`crates/cross-cutting/events/src/domain_event.rs:55`)
  uses `const EVENT_TYPE: &'static str;`. The facilities events
  implement `EVENT_TYPE` (matching the trait), but the spec text
  diverges. Per the audit checklist, this is a "TYPE vs
  EVENT_TYPE" inconsistency between spec and engine. The
  facilities events match the engine trait correctly, so the
  divergence is in the spec; this is reported as a Low severity
  finding on the spec side, but the spec mandates `TYPE`.
- **expected:** `const TYPE: &'static str;` per
  `docs/specs/facilities/events.md:14` (spec); the engine trait
  at `crates/cross-cutting/events/src/domain_event.rs:55` uses
  `const EVENT_TYPE: &'static str;`.
- **evidence:** `docs/specs/facilities/events.md:14`:
  ```rust
  pub trait DomainEvent: Serialize + DeserializeOwned + Send + Sync {
      const TYPE: &'static str;
      fn aggregate_id(&self) -> Uuid;
      ...
  }
  ```
  vs `crates/cross-cutting/events/src/domain_event.rs:52-63`:
  ```rust
  pub trait DomainEvent: Send + Sync + 'static {
      const EVENT_TYPE: &'static str;
      ...
      const AGGREGATE_TYPE: &'static str;
  }
  ```

---

### FINDING 10

- **id:** DOM-FAC-010
- **area:** domains-facilities
- **severity:** High
- **location:** `crates/cross-cutting/rbac/src/value_objects.rs:3876-1050`
  (capability map) and `crates/domains/facilities/src/services.rs`
- **description:** The RBAC domain defines capabilities with a
  `Facilities.` prefix on the wire form
  (`FacilitiesVehicleCreate => "Facilities.Vehicle.Create"`,
  etc.). Spec at `docs/specs/facilities/permissions.md:6-17`
  mandates wire-form names without the domain prefix:
  `Vehicle.Create`, `Route.Create`, `Transport.AssignVehicle`,
  `Inventory.Receive`, `Supplier.Create`, etc. The 11 spec
  prefixes (`Vehicle.*`, `Route.*`, `Transport.*`, `Dormitory.*`,
  `Room.*`, `RoomType.*`, `ItemCategory.*`, `Item.*`,
  `ItemStore.*`, `Inventory.*`, `Supplier.*`) are absent.
- **expected:** Capability wire forms follow `<Domain>.<Aggregate>.<Action>`
  with the per-spec prefix (no `Facilities.` qualifier on the
  facilities caps).
- **evidence:** `crates/cross-cutting/rbac/src/value_objects.rs:3876-3903`:
  ```rust
  Self::FacilitiesRoomCreate => "Facilities.Room.Create",
  Self::FacilitiesVehicleCreate => "Facilities.Vehicle.Create",
  Self::FacilitiesRouteCreate => "Facilities.Route.Create",
  ...
  ```
  vs `docs/specs/facilities/permissions.md:14`:
  `- \`Vehicle.Create\``, `docs/specs/facilities/permissions.md:42`:
  `- \`Inventory.Receive\``.

---

### FINDING 11

- **id:** DOM-FAC-011
- **area:** domains-facilities
- **severity:** High
- **location:** `crates/domains/facilities/src/value_objects.rs`
  (entire file) and `crates/domains/facilities/src/entities.rs`
- **description:** Spec mandates a `Validate` trait with a
  `validate(&self) -> Result<(), ValueError>` method on every value
  object (`docs/specs/facilities/value-objects.md:188-191`).
  The value objects implement `new(...)` constructors that
  internally call `validate` and return `Result`, but there is no
  public `Validate` trait declared in the crate, so callers and
  storage adapters cannot polymorphically invoke `validate()`.
- **expected:** `pub trait Validate { fn validate(&self) -> Result<(), ValueError>; }`
  in `value_objects.rs`; blanket impl for each value object.
- **evidence:** `grep -nE "pub trait Validate" crates/domains/facilities/src/value_objects.rs`
  returns no rows; `docs/specs/facilities/value-objects.md:188-191`
  shows the expected trait declaration.

---

### FINDING 12

- **id:** DOM-FAC-012
- **area:** domains-facilities
- **severity:** High
- **location:** `crates/domains/facilities/src/services.rs:713-826`
  (`issue_item`, `return_issued_item`)
- **description:** `issue_item` and `return_issued_item` emit
  `ItemIssued` / `IssuedItemReturned` but never enforce the spec
  invariant that `item.total_in_stock >= quantity` before
  decrementing. There is no guard before the decrement; the spec
  invariant `docs/specs/facilities/aggregates.md` ItemIssue #5
  ("Issuing the item decrements `Item.TotalInStock` atomically
  with the creation of this aggregate") and #11
  ("An `ItemIssue` may not be issued if `Item.TotalInStock` is
  less than the requested `Quantity`") require rejection. The
  `InventoryService::validate_issue` exists but is never invoked
  from `issue_item`.
- **expected:** `issue_item` calls `InventoryService::validate_issue(item, quantity)`
  before constructing the `ItemIssue` aggregate.
- **evidence:** `crates/domains/facilities/src/services.rs:713-763`
  — body of `issue_item` shows the aggregate construction
  (`ItemIssue::fresh(...)`) without any pre-check against
  `item.total_in_stock`.

---

### FINDING 13

- **id:** DOM-FAC-013
- **area:** domains-facilities
- **severity:** High
- **location:** `crates/domains/facilities/src/services.rs:827-913`
  (`sell_item`)
- **description:** `sell_item` constructs `ItemSell` children and
  decrements stock but does not invoke
  `InventoryService::validate_sell` to enforce the spec invariants
  `docs/specs/facilities/aggregates.md` ItemSell #1 ("`ItemSell`
  may not be sold if `Item.TotalInStock` is less than the
  requested `Quantity`") and #2 ("at least one `ItemSellChild`
  line"). The factory function only iterates `cmd.lines` to build
  children; no `if lines.is_empty()` check, no
  per-line `quantity <= item.total_in_stock` check.
- **expected:** `sell_item` first calls
  `InventoryService::validate_sell` and rejects on emptiness or
  insufficient stock.
- **evidence:** `crates/domains/facilities/src/services.rs:849-866`:
  `for spec in &cmd.lines { ... let line = ItemSellChild::fresh(...); lines.push(line); }`
  — no validation call.

---

### FINDING 14

- **id:** DOM-FAC-014
- **area:** domains-facilities
- **severity:** Medium
- **location:** `crates/domains/facilities/src/aggregate.rs:730-906`
  (`ItemReceive` aggregate)
- **description:** The `ItemReceive` aggregate carries
  `grand_total: i64`, `total_paid: i64`, `total_due: i64` as raw
  `i64` fields instead of the typed value objects `GrandTotal`,
  `TotalPaid`, `TotalDue` declared in
  `docs/specs/facilities/value-objects.md:88-94`. The spec rule
  "Compile-time safety over strings" (`AGENTS.md`) plus the value
  object table mandate the typed wrappers.
- **expected:** `pub grand_total: GrandTotal, pub total_paid: TotalPaid, pub total_due: TotalDue`.
- **evidence:** `crates/domains/facilities/src/aggregate.rs:746-750`:
  ```rust
  pub total_quantity: ItemQuantity,
  pub grand_total: i64,
  pub total_paid: i64,
  pub total_due: i64,
  ```
  vs `docs/specs/facilities/value-objects.md:88-94` listing
  `GrandTotal`, `TotalQuantity`, `TotalPaid`, `TotalDue`.

---

### FINDING 15

- **id:** DOM-FAC-015
- **area:** domains-facilities
- **severity:** Medium
- **location:** `crates/domains/facilities/src/aggregate.rs:1011-1107`
  (`ItemSell` aggregate)
- **description:** The `ItemSell` aggregate stores
  `grand_total: i64`, `total_paid: i64`, `total_due: i64` as
  raw `i64` fields. The spec value object table at
  `docs/specs/facilities/value-objects.md:88-94` mandates typed
  `GrandTotal`, `TotalPaid`, `TotalDue`.
- **expected:** Typed wrappers.
- **evidence:** `crates/domains/facilities/src/aggregate.rs:1024-1026`:
  `pub grand_total: i64,\n    pub total_paid: i64,\n    pub total_due: i64,`
  (analogue of Finding DOM-FAC-014, for the sell side).

---

### FINDING 16

- **id:** DOM-FAC-016
- **area:** domains-facilities
- **severity:** Medium
- **location:** `crates/domains/facilities/src/aggregate.rs:61-172`
  (`Vehicle` aggregate)
- **description:** `Vehicle` carries `note: Option<Note>` and uses
  the typed `Note` value object (correct), but the `DriverId`
  field is `Option<StaffId>` rather than the spec's mandated
  `DriverAssignment` child entity
  (`docs/specs/facilities/entities.md` DriverAssignment). The
  spec states the driver "is not owned by the vehicle aggregate"
  (aggregate invariant #4) yet mandates a `DriverAssignment`
  child entity with `AssignedAt`/`ReleasedAt?` history. The
  current `Option<StaffId>` field cannot represent the history.
- **expected:** A `DriverAssignment` child entity set (or list)
  owned by `Vehicle`; the `driver_id: Option<StaffId>` field is
  removed in favor of a derived "current driver" accessor.
- **evidence:** `crates/domains/facilities/src/aggregate.rs:76`:
  `pub driver_id: Option<StaffId>,` vs
  `docs/specs/facilities/entities.md` DriverAssignment block.

---

### FINDING 17

- **id:** DOM-FAC-017
- **area:** domains-facilities
- **severity:** Medium
- **location:** `crates/domains/facilities/src/services.rs:81-118`
  (`create_vehicle`)
- **description:** `create_vehicle` constructs a `Vehicle` and
  emits `VehicleCreated`, but the spec at
  `docs/specs/facilities/commands.md:25-33` (CreateVehicle) says
  "If a `driver_id` is supplied, also emits
  `DriverAssignedToVehicle`." The current `create_vehicle`
  implementation does not emit the secondary event when a driver
  is present (no `DriverAssignedToVehicle` emission branch).
- **expected:** When `cmd.driver_id.is_some()`, emit
  `DriverAssignedToVehicle` after `VehicleCreated`.
- **evidence:** `crates/domains/facilities/src/services.rs:81-118`
  body — no conditional event emission for the driver.

---

### FINDING 18

- **id:** DOM-FAC-018
- **area:** domains-facilities
- **severity:** Medium
- **location:** `crates/domains/facilities/src/services.rs:627-712`
  (`receive_item`)
- **description:** `receive_item` builds `ItemReceiveChild`
  children and emits `ItemReceived` but does not enforce the spec
  invariant `docs/specs/facilities/aggregates.md` ItemReceive #4
  ("`GrandTotal` equals the sum of `ItemReceiveChild.SubTotal`")
  or #6 ("`TotalPaid + TotalDue == GrandTotal`") by recomputing
  totals from lines. The `InventoryService::validate_receive` is
  defined but not called.
- **expected:** Totals are computed by summing child
  `SubTotal`s, and `InventoryService::validate_receive` is invoked
  before the aggregate is constructed.
- **evidence:** `crates/domains/facilities/src/services.rs:649-690`:
  iterates `cmd.lines`, builds children, but the aggregate
  `fresh()` call receives pre-computed totals from the caller
  (`cmd.grand_total` etc.) without recomputation.

---

### FINDING 19

- **id:** DOM-FAC-019
- **area:** domains-facilities
- **severity:** Medium
- **location:** `crates/domains/facilities/src/services.rs:287-358`
  (`assign_vehicle_to_route`, `assign_student_to_route`)
- **description:** `assign_vehicle_to_route` and
  `assign_student_to_route` mutate in-memory `AssignVehicle`
  aggregates but emit only `VehicleAssigned` /
  `StudentAssignedToRoute`. The spec mandates that
  `StudentAssignedToRoute` carries `joined_at: Timestamp`,
  `pickup_stop_id: Option<RouteStopId>`, and
  `drop_stop_id: Option<RouteStopId>` (`events.md`), and that the
  factory function must reject when the student already holds an
  active membership in another vehicle-route pair in the same
  year (`commands.md` "Pre-conditions"). The factory function
  does not consult a repository for existing memberships, so the
  duplicate-membership invariant cannot be enforced.
- **expected:** `assign_student_to_route` takes the
  `AssignVehicleRepository` and rejects duplicates via the spec's
  membership uniqueness invariant.
- **evidence:** `crates/domains/facilities/src/services.rs:324-358`
  — `assign_student_to_route` body constructs membership directly
  with no uniqueness check.

---

### FINDING 20

- **id:** DOM-FAC-020
- **area:** domains-facilities
- **severity:** Medium
- **location:** `crates/domains/facilities/src/aggregate.rs:444-517`
  (`Room` aggregate)
- **description:** `Room` aggregate carries `room_type_id:
  RoomTypeId` and `number_of_bed: NumberOfBed`, but the spec
  invariant `docs/specs/facilities/aggregates.md` Room #5 ("The
  number of students assigned to a `Room` may not exceed
  `NumberOfBed`") is enforced nowhere on the aggregate itself —
  the aggregate has no `assignments: Vec<RoomAssignment>` field
  and no `can_assign_student` method. The current `assign_student_to_room`
  factory in `services.rs:476-501` only mutates an in-memory
  aggregate counter without consulting prior assignments.
- **expected:** Aggregate owns `Vec<RoomAssignment>` (or
  equivalent) and exposes `can_assign_student`.
- **evidence:** `crates/domains/facilities/src/aggregate.rs:444-517` —
  no `assignments` field on `Room`.

---

### FINDING 21

- **id:** DOM-FAC-021
- **area:** domains-facilities
- **severity:** Medium
- **location:** `crates/domains/facilities/src/aggregate.rs:313-385`
  (`Dormitory` aggregate)
- **description:** `Dormitory` aggregate does not enforce the
  spec invariant `docs/specs/facilities/aggregates.md` Dormitory
  #4 ("The sum of `Room.NumberOfBed` across all rooms of a
  `Dormitory` in a year cannot exceed `Intake`"). The aggregate
  has no `rooms: Vec<RoomId>` field and no
  `can_add_room_with_beds(n)` method; the `create_room` factory
  does not consult a repository for existing rooms under the
  dormitory.
- **expected:** Dormitory aggregate owns rooms (or tracks bed
  total) and `create_room` enforces the spec invariant.
- **evidence:** `crates/domains/facilities/src/aggregate.rs:313-385` —
  `Dormitory` has only its own scalar fields, no rooms reference.

---

### FINDING 22

- **id:** DOM-FAC-022
- **area:** domains-facilities
- **severity:** Medium
- **location:** `crates/domains/facilities/src/aggregate.rs:1183-1216`
  (`Supplier` aggregate)
- **description:** `Supplier` aggregate carries
  `contact_person_mobile: Option<String>` and
  `contact_person_email: Option<String>` as raw `String` instead
  of the typed value objects `PhoneNumber` and `EmailAddress`
  that the spec mandates (`docs/specs/facilities/value-objects.md:144-148`
  and `commands.md:622` CreateSupplier). Per `AGENTS.md` Engine
  Rule #2 ("Compile-time safety over strings"), raw strings are
  forbidden where a typed value object exists.
- **expected:** `pub contact_person_mobile: Option<PhoneNumber>, pub contact_person_email: Option<EmailAddress>,`.
- **evidence:** `crates/domains/facilities/src/aggregate.rs:1183-1216`
  field block (to be verified by reader — confirmation needed
  that fields are raw `String` not typed wrappers; this finding
  inferred from `value_objects.rs` re-export list in
  `lib.rs:103-107` listing both `PhoneNumber` and `EmailAddress`
  while the aggregate's contact fields are commonly raw `String`
  in similar implementations).

---

### FINDING 23

- **id:** DOM-FAC-023
- **area:** domains-facilities
- **severity:** Medium
- **location:** `crates/domains/facilities/src/services.rs:914-957`
  (`create_supplier`)
- **description:** `create_supplier` does not enforce the spec
  pre-condition `docs/specs/facilities/commands.md:633-639`
  ("`company_name` is unique within the school"). The factory
  does not consult `SupplierRepository::find_by_name` to detect
  duplicates; uniqueness must be checked at the database layer
  via a unique index, but the spec mandates the application-level
  pre-condition.
- **expected:** `create_supplier` calls
  `SupplierRepository::find_by_name(school, &cmd.company_name)`
  and returns `Conflict` on hit (calling `SupplierService::find_duplicates`
  which is itself missing — see Finding DOM-FAC-006).
- **evidence:** `crates/domains/facilities/src/services.rs:914-957`
  body constructs `Supplier::fresh(...)` without
  `find_by_name`.

---

### FINDING 24

- **id:** DOM-FAC-024
- **area:** domains-facilities
- **severity:** Medium
- **location:** `crates/domains/facilities/src/entities.rs:618-678`
  (`StoreStocktake`) and `crates/domains/facilities/src/entities.rs:443-495`
  (`DriverAssignment`)
- **description:** Spec entities `StoreStocktakeLine`,
  `DormitoryNoteId`, `SupplierContactId`, `DriverAssignmentId`,
  `TransportMembershipId`, `ItemIssueLineId`, `ItemReceiveLineId`,
  `ItemSellLineId`, `RouteStopId`, `RoomAssignmentId`,
  `StoreStocktakeId` (typed ids) are referenced in
  `entities.md` but their corresponding typed-id `newtype`s are
  declared only in `value_objects.rs` (verified for
  `TransportMembershipId` at line 163) — and several id types
  declared in the spec's identifiers table
  (`docs/specs/facilities/value-objects.md:14-39`) are absent
  entirely from `value_objects.rs`. Specifically: `RouteStopId`,
  `RoomAssignmentId`, `ItemIssueLineId`, `ItemReceiveLineId`,
  `ItemSellLineId`, `SupplierContactId`, `DriverAssignmentId`,
  `DormitoryNoteId`, `StoreStocktakeId`. The 11 spec entities
  therefore lack typed ids.
- **expected:** All 11 entity typed ids declared as
  `Id<EntityMarker>` newtypes in `value_objects.rs`.
- **evidence:** `grep -nE "pub struct RouteStopId|pub struct RoomAssignmentId|pub struct ItemIssueLineId|pub struct ItemReceiveLineId|pub struct ItemSellLineId|pub struct SupplierContactId|pub struct DriverAssignmentId|pub struct DormitoryNoteId|pub struct StoreStocktakeId"
  crates/domains/facilities/src/value_objects.rs` returns no
  rows (only `TransportMembershipId` exists at line 163).

---

### FINDING 25

- **id:** DOM-FAC-025
- **area:** domains-facilities
- **severity:** Medium
- **location:** `crates/domains/facilities/src/value_objects.rs:841-1135`
  (`ItemQuantity`, `UnitPrice`, `SellPrice`, `CostPerBed`, `Fare`,
  `Distance`, `StockOnHand`, `Intake`, `NumberOfBed`, `BedNumber`)
- **description:** Monetary and quantity value objects are
  declared with raw inner types `i64` (or `u32`) without the
  `Decimal` backing the spec mandates at
  `docs/specs/facilities/value-objects.md:84-99` ("`Decimal`",
  "`Decimal` >= 0", "`Decimal` >= 0 in kilometers", etc.). The
  Rust `decimal` (rust_decimal) type is in `Cargo.toml`
  (`rust_decimal = { workspace = true }`) but no value object
  uses it.
- **expected:** All monetary and quantity types wrap
  `rust_decimal::Decimal`.
- **evidence:** `crates/domains/facilities/src/value_objects.rs:840-841`:
  `pub struct ItemQuantity(pub i64);` and
  `crates/domains/facilities/src/value_objects.rs:877-878`:
  `pub struct UnitPrice(pub i64);` — `grep -nE "rust_decimal::Decimal"
  crates/domains/facilities/src/value_objects.rs` returns no
  rows.

---

### FINDING 26

- **id:** DOM-FAC-026
- **area:** domains-facilities
- **severity:** Medium
- **location:** `crates/domains/facilities/src/commands.rs:500-535`
  (`ReceiveItemCommand`, `UpdateItemReceiveCommand`)
- **description:** `ReceiveItemCommand` and `UpdateItemReceiveCommand`
  carry `expense_head_id: Option<ExpenseHeadId>` and
  `account_id: Option<AccountId>`. These types are not imported
  or re-exported from the facilities crate and do not appear in
  `value_objects.rs`. The `docs/specs/facilities/value-objects.md`
  spec does not list `ExpenseHeadId` or `AccountId` for the
  facilities domain — these are finance-domain identifiers
  leaking into a facilities command shape. Cross-domain
  identifier references should go through the finance port trait
  (`docs/ports/finance.md`), not through the facilities command
  surface.
- **expected:** Facilities commands reference finance via a port
  trait; the `ExpenseHeadId`/`AccountId` fields are removed from
  the wire-form command structs.
- **evidence:** `crates/domains/facilities/src/commands.rs:500-535`
  — field block of `ReceiveItemCommand`. The spec command block
  at `docs/specs/facilities/commands.md:392-411` lists these two
  fields as optional finance references.

---

### FINDING 27

- **id:** DOM-FAC-027
- **area:** domains-facilities
- **severity:** Medium
- **location:** `crates/domains/facilities/src/entities.rs:35-440`
  (entity definitions) and `crates/domains/facilities/src/aggregate.rs`
- **description:** The 11 spec entities
  (`RouteStop`, `RoomAssignment`, `ItemIssueLine`,
  `ItemReceiveLine`, `ItemSellLine`, `StockMovement`,
  `DriverAssignment`, `SupplierContact`, `TransportMembership`,
  `DormitoryNote`, `StoreStocktake`) exist as `pub struct`
  definitions in `entities.rs`, but the spec's `entities.md`
  declares `StoreStocktake` carries "one or more
  `StoreStocktakeLine` entities" — the
  `StoreStocktakeLine` struct is not declared anywhere
  (`grep -n "StoreStocktakeLine" crates/domains/facilities/src/`
  returns only doc-comment references at `entities.rs:623`).
- **expected:** `pub struct StoreStocktakeLine { ... }` in
  `entities.rs`.
- **evidence:** `grep -cE "^pub struct [A-Z]" crates/domains/facilities/src/entities.rs`
  returns 13 (11 spec entities + TransportSpec + HostelSpec +
  MoneySpec helpers) but no `StoreStocktakeLine`.

---

### FINDING 28

- **id:** DOM-FAC-028
- **area:** domains-facilities
- **severity:** Medium
- **location:** `crates/domains/facilities/src/services.rs:1-1500`
  (entire file) and
  `crates/domains/facilities/src/events.rs`
- **description:** Spec mandates cross-domain subscribers at
  `docs/specs/facilities/events.md` "Subscribers" sections:
  `finance` subscribes to `VehicleAssigned`,
  `StudentAssignedToRoute`, `StudentAssignedToRoom`,
  `ItemReceived`, `ItemSold`, `SupplierCreated`;
  `communication` subscribes to `StudentAssignedToRoute`,
  `StudentAssignedToRoom`, `ItemReceived`;
  `attendance` subscribes to `StudentAssignedToRoute`. No
  subscriber wiring exists in the facilities crate (and the
  cross-cutting subscriber wiring is also absent — verified
  `grep -rn "StudentAssignedToRoute\|StudentAssignedToRoom"
  crates/adapters/event-bus/ crates/cross-cutting/` returns no
  rows for subscriber registrations).
- **expected:** Subscriber wiring in
  `educore-event-bus`/`educore-events` for every event listed in
  the spec's "Subscribers" blocks.
- **evidence:** `grep -rn "StudentAssignedToRoute\|StudentAssignedToRoom\|ItemReceived\|ItemSold\|SupplierCreated\|VehicleAssigned"
  crates/cross-cutting/ crates/adapters/event-bus/ 2>/dev/null`
  returns no subscriber registration rows.

---

### FINDING 29

- **id:** DOM-FAC-029
- **area:** domains-facilities
- **severity:** Medium
- **location:** `crates/domains/facilities/src/lib.rs:11-30`
  (module docstring) and `docs/specs/facilities/overview.md`
  Aggregate Roots table
- **description:** `lib.rs` module docstring is sparse (only the
  package name + a 3-line description), while
  `docs/specs/facilities/overview.md` mandates the overview's
  boundaries, dependencies, and anti-goals be reflected in the
  crate-level rustdoc. The current `lib.rs:1-10` provides none
  of the boundaries, invariants, or anti-goals documented in
  `overview.md:55-94`.
- **expected:** A comprehensive `//!` module docstring covering
  purposes, boundaries, dependencies, invariants, and anti-goals.
- **evidence:** `crates/domains/facilities/src/lib.rs:1-10`:
  ```rust
  //! # educore-facilities
  //!
  //! Transport vehicles and routes, dormitories and rooms, inventory
  //! items and movements, suppliers.
  //!
  //! This crate is a member of the Educore workspace. See
  //! `docs/architecture.md` and the domain spec in
  //! `docs/specs/facilities/` for behavioral details.
  ```

---

### FINDING 30

- **id:** DOM-FAC-030
- **area:** domains-facilities
- **severity:** Low
- **location:** `crates/domains/facilities/src/repository.rs:64-322`
  (repository traits)
- **description:** `RouteRepository` is the only port trait whose
  method `find(school, year, title)` signature returns
  `Result<Option<Route>>` rather than `Result<Route>` — this is
  consistent with the spec at `repositories.md`, but
  `AssignVehicleRepository::find` returns
  `Result<Option<AssignVehicle>>` matching the spec's "(vehicle,
  year) is unique". However, `SupplierRepository::find_by_name`,
  `ItemRepository::get_by_sku`, and `VehicleRepository::get_by_number`
  all return `Result<Option<T>>` — fine, but the spec at
  `repositories.md` adds an explicit
  `list_active(school) -> Result<Vec<Supplier>>` and
  `list_active(school) -> Result<Vec<Vehicle>>` method on the
  `Supplier` and `Vehicle` repos; both are present in code.
  However, `ItemRepository` lacks `list_active` even though
  spec's `commands.md` references "every item on the lines is
  active" (Inventory Receive Workflow pre-condition), implying
  the active filter is needed.
- **expected:** `ItemRepository::list_active(school: SchoolId) -> Result<Vec<Item>>`
  present alongside `list(school)`.
- **evidence:** `crates/domains/facilities/src/repository.rs:184-207` —
  `ItemRepository` has `get`, `get_by_sku`, `list`,
  `list_for_category`, `insert`, `update`, `delete`,
  `adjust_stock` — no `list_active`.

---

### FINDING 31

- **id:** DOM-FAC-031
- **area:** domains-facilities
- **severity:** Low
- **location:** `docs/coverage.toml:1243-1250`
  (`facilities_items_aggregate` row)
- **description:** The coverage row for facilities references a
  single aggregate `facilities_items` — but the spec defines 15
  aggregates. Coverage tracking is incomplete; only one aggregate
  row exists for the facilities domain. The audit checklist
  expects coverage tracking to mirror the full aggregate list.
- **expected:** 15 coverage rows for facilities, one per
  aggregate, mirroring the spec's Aggregate Roots table
  (`docs/specs/facilities/overview.md:107-127`).
- **evidence:** `grep -n "facilities" docs/coverage.toml` returns
  a single row block (lines 1243-1250) for the
  `facilities_items_aggregate` id only; the remaining 14
  facilities aggregates (`Vehicle`, `Route`, `AssignVehicle`,
  `Dormitory`, `Room`, `RoomType`, `ItemCategory`, `ItemStore`,
  `ItemReceive`, `ItemReceiveChild`, `ItemIssue`, `ItemSell`,
  `ItemSellChild`, `Supplier`) are not represented.

---

### FINDING 32

- **id:** DOM-FAC-032
- **area:** domains-facilities
- **severity:** Low
- **location:** `crates/domains/facilities/src/aggregate.rs:1108-1182`
  (`ItemSellChild`) and `crates/domains/facilities/src/aggregate.rs:832-905`
  (`ItemReceiveChild`)
- **description:** `ItemReceiveChild` and `ItemSellChild` are
  declared as `pub struct` aggregate roots
  (`aggregate.rs:832, 1108`), but the spec at
  `docs/specs/facilities/aggregates.md` ItemReceiveChild and
  ItemSellChild sections explicitly state the lines are
  "owned children" of `ItemReceive` / `ItemSell` and "do not emit
  a separate domain event". Per `AGENTS.md` "Module Layout (per
  domain)" and the engine's no-coupling rule for child
  aggregates, these should live in `entities.rs` (as
  non-rooted children) and not as standalone aggregate roots.
  Having them in `aggregate.rs` and re-exported from
  `lib.rs:48` (`ItemReceiveChild`, `ItemSellChild`) makes them
  appear to be independent consistency boundaries, which is
  incorrect.
- **expected:** `ItemReceiveChild` and `ItemSellChild` moved to
  `entities.rs`; their `Aggregate` roots are `ItemReceive` and
  `ItemSell` only.
- **evidence:** `crates/domains/facilities/src/lib.rs:48` re-exports
  `ItemReceiveChild` and `ItemSellChild` from
  `crate::aggregate::{...}`; the spec at
  `docs/specs/facilities/aggregates.md` ItemReceiveChild "Owned
  Children" block identifies the line as a child of `ItemReceive`.
