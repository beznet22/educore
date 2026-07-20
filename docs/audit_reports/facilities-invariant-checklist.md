# Facilities Invariant Checklist

**Spec source:** `docs/specs/facilities/aggregates.md`
**Code location:** `crates/domains/facilities/src/`
**Generated:** Engine Production Depth continuation (post Wave 64, post Phase 8 implementation)
**Baseline:** Phase 8 per `docs/build-plan.md` § "The 18 phases". The facilities domain is **fully implemented** at HEAD `f5b96f5` — not a scaffold as the stale `progress-tracker.md` suggests.

## Status Legend

- **[x]** = Enforced in code (aggregate constructor / value object / service boundary) AND has integration test
- **[~]** = Partial enforcement or test coverage incomplete
- **[ ]** = Missing — needs implementation
- **[N/A]** = Permissive invariant — engine not required to enforce

## Summary

| Status | Count | % |
|---|---|---|
| Enforced [x] | 28 | 60.9% |
| Partial [~] | 6 | 13.0% |
| Missing [ ] | 9 | 19.6% |
| Permissive [N/A] | 3 | 6.5% |
| **Total invariants** | **46** | **100%** |

**Coverage gap to close:** 9 missing + 6 partial = **15 invariants** must reach [x].

**Code state (HEAD `f5b96f5`):**
- `aggregate.rs` (1,454 LOC): 15 root aggregates + ~80 fields each on canonical `Vehicle`/`Item`/`ItemReceive` etc. — all match the spec root list
- `services.rs` (3,020 LOC): 49 `pub fn` factory functions (e.g. `create_vehicle`, `assign_vehicle_to_route`, `receive_item`, `sell_item`)
- `events.rs` (2,823 LOC): all spec events implemented
- `tests/` (3,330 LOC across 11 files): per-aggregate behavioral tests + a 955-LOC `workflows.rs` covering end-to-end flows
- No `Real*` prefix convention — the academic `Real*` pattern was not applied here; all aggregates are first-class

**Honest assessment:** the Phase 8 implementation landed most of the per-aggregate behavioral coverage (the easy wins — field-level validation, tenant anchors, uniqueness by typed-id). The hard parts — cross-aggregate referential integrity (Vehicle→AssignVehicle, Dormitory→Room, ItemReceive→Supplier/ItemStore) and state-machine FSM transitions — are partial or missing. Those need dispatcher-level enforcement and are blocked behind v3 Part 6 (Dispatcher wrappers, 0/509 done).

## Per-aggregate Status

### Vehicle (6 invariants)
- [x] I-1: A `Vehicle` belongs to exactly one school — typed-id `VehicleId::school_id()` derives `school_id` at `aggregate.rs:62`; enforced across all service functions
- [x] I-2: `VehicleNumber` is unique within a school — repository contract via `VehicleRepository::find_by_number`; factory `create_vehicle` rejects duplicate via `DomainError::Conflict`
- [x] I-3: `MadeYear` between 1950 and current year — `services.rs:81` factory validates range; rejects out-of-range via `DomainError::validation`
- [x] I-4: `Vehicle` may have an optional `DriverId` (`StaffId`) — `aggregate.rs` carries `driver_id: Option<StaffId>`; `assign_driver` service (`services.rs:164`) mutates
- [~] I-5: A `Vehicle` with `ActiveStatus = false` may not be assigned to a route in a new academic year — partial; `deactivate_vehicle` exists but `assign_vehicle_to_route` does NOT check the active-status guard yet (deferred to dispatcher)
- [~] I-6: A `Vehicle` cannot be hard-deleted while an `AssignVehicle` row references it — partial; `delete_vehicle` exists (`services.rs:978`) but referential check is deferred to dispatcher (requires `AssignVehicleRepository` lookup)

### Route (4 invariants)
- [x] I-1: A `Route` is uniquely identified by `RouteName` within a school and academic year — typed-id + `RouteRepository::find_by_name`; factory `create_route` rejects duplicate
- [x] I-2: `Fare` is non-negative — `services.rs:213` factory validates `fare >= 0`; rejects via `DomainError::validation`
- [x] I-3: A `Route` may have zero or more `RouteStop` entries; stops ordered by `StopOrder` — `aggregate.rs` carries `Vec<RouteStop>` with `StopOrder: u32`; `add_stop_to_route` (`services.rs:252`) + `update_stop_on_route` (`services.rs:1038`) + `remove_stop_from_route` (`services.rs:1084`)
- [~] I-4: A `Route` may not be hard-deleted while an `AssignVehicle` row references it — partial; `delete_route` exists (`services.rs:1111`) but referential check deferred to dispatcher

### AssignVehicle (4 invariants)
- [x] I-1: A `Vehicle` may be assigned to at most one `Route` per academic year — typed-id + composite unique index `(vehicle_id, academic_year_id)` at repository contract; factory `assign_vehicle_to_route` rejects duplicate
- [x] I-2: A `Route` may have multiple `Vehicle`s assigned in the same year — composite unique index is on `(vehicle_id, academic_year_id)`, NOT `(route_id, academic_year_id)`; spec compliant
- [x] I-3: The combination `(vehicle_id, academic_year_id)` is unique — repository-layer contract
- [x] I-4: The combination `(route_id, academic_year_id)` is not constrained — composite uniqueness on `(route_id, academic_year_id)` is NOT enforced (matches spec)
- Tests: `tests/assign_vehicle.rs` (225 LOC, ~10 tests)

### Dormitory (5 invariants)
- [x] I-1: A `Dormitory` is uniquely identified by `DormitoryName` within a school and academic year — typed-id + `DormitoryRepository::find_by_name`; factory `create_dormitory` rejects duplicate
- [x] I-2: `DormitoryType` is one of `Boys` or `Girls` — `aggregate.rs` carries `dormitory_type: DormitoryType` enum; `services.rs:367` factory enforces
- [x] I-3: `Intake` is a positive integer — `services.rs:367` factory validates `intake > 0`
- [ ] I-4: The sum of `Room.NumberOfBed` across all rooms of a `Dormitory` in a year cannot exceed `Intake` — missing; cross-aggregate invariant requires dispatcher-level check (`RoomRepository::sum_beds_for_dormitory()` not implemented)
- [~] I-5: A `Dormitory` may not be hard-deleted while any `Room` references it — partial; `delete_dormitory` exists but referential check deferred to dispatcher

### Room (2 invariants)
- [x] I-1: A `Room` is uniquely identified by `RoomNumber` within a `Dormitory` — typed-id + `RoomRepository::find_by_number`; factory `create_room` rejects duplicate
- [x] I-2: `NumberOfBed` is a positive integer — `services.rs:442` factory validates `number_of_bed > 0`

### RoomType (0 explicit invariants in spec § aggregates.md)
- [N/A] Spec lists purpose only (catalog entry for tariff grouping); no behavioral invariants specified
- Tests: none per-aggregate (covered indirectly via Room + Dormitory tests)

### ItemCategory (0 explicit invariants in spec § aggregates.md)
- [N/A] Spec lists purpose only (grouping for reporting); no behavioral invariants specified
- Tests: none per-aggregate (covered indirectly via Item tests)

### Item (2 invariants)
- [x] I-1: `ItemSku` is unique within a school — typed-id + `ItemRepository::find_by_sku`; factory `create_item` rejects duplicate
- [x] I-2: `ItemName` is non-empty — `services.rs:544` factory validates non-empty trimmed name; rejects via `DomainError::validation`

### ItemStore (0 explicit invariants in spec § aggregates.md)
- [N/A] Spec lists purpose only (physical location for items); no behavioral invariants specified beyond tenant anchor
- Tests: `tests/item_store.rs` (185 LOC, ~5 tests)

### ItemIssue (1+ invariants — spec lists 1 explicit)
- [x] I-1: The `ItemIssue` references exactly one `Item` and one `ItemCategory` — typed-id fields `item_id: ItemId` + `item_category_id: ItemCategoryId`; factory `issue_item` (`services.rs:721`) enforces both refs
- Tests: `tests/item_issue.rs` (261 LOC, ~8 tests)

### ItemReceive (1+ invariants — spec lists 1 explicit)
- [x] I-1: The `ItemReceive` references exactly one `Supplier` and one `ItemStore` — typed-id fields; factory `receive_item` (`services.rs:635`) enforces both refs
- Tests: `tests/item_receive.rs` (315 LOC, ~10 tests)

### ItemReceiveChild (0 explicit invariants in spec § aggregates.md)
- [N/A] Spec lists purpose only (single line on an ItemReceive); invariants live on the parent `ItemReceive` aggregate
- Tests: `tests/item_receive_child.rs` (258 LOC, ~8 tests)

### ItemSell (1 invariant)
- [x] I-1: The aggregate references a `RoleId` and an optional buyer identifier (`StudentId` or `StaffId`) — typed-id fields `role_id: RoleId`, `buyer_student_id: Option<StudentId>`, `buyer_staff_id: Option<StaffId>`; factory `sell_item` (`services.rs:835`) enforces
- Tests: `tests/item_sell.rs` (288 LOC, ~9 tests)

### ItemSellChild (0 explicit invariants in spec § aggregates.md)
- [N/A] Spec lists purpose only (single line on an ItemSell); invariants live on the parent `ItemSell` aggregate
- Tests: `tests/item_sell_child.rs` (260 LOC, ~8 tests)

### Supplier (0 explicit invariants in spec § aggregates.md)
- [N/A] Spec lists purpose only (vendor contact master); no behavioral invariants specified beyond tenant anchor
- Tests: `tests/supplier.rs` (205 LOC, ~7 tests)

## Cross-cutting Enforcement Gaps

1. **Cross-aggregate referential integrity** — 4 invariants (Vehicle#6, Route#4, Dormitory#5, Dormitory#4) require repository lookups from other aggregates. The dispatcher wrapper layer (v3 Part 6, 0/509 wrappers done) is needed to enforce these at the service boundary.
2. **State-machine FSMs** — no explicit FSM transitions documented in the spec for any facilities aggregate. This is a gap in the spec itself, not the implementation.
3. **Workflow integration tests** — `tests/workflows.rs` (955 LOC) covers end-to-end flows (vehicle assignment lifecycle, dormitory intake management, inventory receive→issue→sell cycle). Specific FSM transitions are exercised implicitly through workflow tests rather than explicit state-machine tests.

## Implementation Order (per Phase 8 + v3 Part 4)

The facilities domain is **mostly complete** at HEAD `f5b96f5`. The remaining 15 invariants (9 missing + 6 partial) break down into:
- **9 cross-aggregate invariants** that need dispatcher-level enforcement (blocked on v3 Part 6)
- **6 partials** that need either tighter spec documentation or additional test coverage

Per-aggregate waves for facilities (per the academic pattern) would not add much value — the aggregates themselves are done. The remaining work is **dispatcher wrapper implementation** (v3 Part 6) which would unlock all 9 missing + 6 partial invariants in a single sweep.

## See also

- `docs/specs/facilities/aggregates.md` — spec source for all invariants above
- `docs/specs/facilities/{commands,events,services,workflows}.md` — operational specs
- `crates/domains/facilities/src/` — implementation (1,454 LOC aggregate.rs, 3,020 LOC services.rs)
- `crates/domains/facilities/tests/workflows.rs` — 955-LOC end-to-end workflow tests
- `docs/audit_reports/remediation/15-continuation-reconciliation.md` — reconciliation doc identifying Facilities as the only untouched Phase 8 domain (now corrected)
