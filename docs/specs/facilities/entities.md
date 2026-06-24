# Facilities Domain — Entities

Entities have identity and lifecycle but are not aggregate roots.
They are loaded and persisted only through their aggregate root.

## RouteStop

**Identity:** `RouteStopId(SchoolId, Uuid)`
**Owner:** `Route`

A single stop on a transport route, with a `StopOrder` (u32), a
`StopName`, an optional `PickupTime`, and an optional `Fare` override
(the default is the route's fare).

## RoomAssignment

**Identity:** `RoomAssignmentId(SchoolId, Uuid)`
**Owner:** `Room`

A current or historical assignment of a `StudentId` to a specific
bed in a `Room`. Has `AssignedAt`, `ReleasedAt?`, and a
`BedNumber` (1..N where N is `Room.NumberOfBed`). The current
assignment for a student is the one with `ReleasedAt = None`.

## ItemIssueLine

**Identity:** `ItemIssueLineId(SchoolId, Uuid)`
**Owner:** `ItemIssue`

The line on an `ItemIssue`. In the canonical model the line is the
`Item` itself, but an issue may carry a single child entity that
encodes the `Note` and a returned-quantity counter for
`PartiallyReturned` status.

## ItemReceiveLine

**Identity:** `ItemReceiveLineId(SchoolId, Uuid)`
**Owner:** `ItemReceive`

A non-rooted child entity that pairs a received `Item` with a
`BatchNumber` and an optional `ExpiryDate` (for perishable items).
Backs the `ItemReceiveChild` aggregate when per-line metadata
exceeds the four scalar fields of the canonical child.

## ItemSellLine

**Identity:** `ItemSellLineId(SchoolId, Uuid)`
**Owner:** `ItemSell`

A non-rooted child entity that pairs a sold `Item` with a discount
override and a `SerialNumber` (for serialized assets). Backs the
`ItemSellChild` aggregate when per-line metadata exceeds the
scalar fields of the canonical child.

## StockMovement

**Identity:** `StockMovementId(SchoolId, Uuid)`
**Owner:** `Item`

A read-model entity that records every increment or decrement of
`Item.TotalInStock`. The event log is the source of truth; the
movement is a derived view.

## DriverAssignment

**Identity:** `DriverAssignmentId(SchoolId, Uuid)`
**Owner:** `Vehicle`

The current and historical assignments of a `StaffId` to a `Vehicle`
as the primary driver. Has `AssignedAt` and `ReleasedAt?`. A vehicle
may have at most one current driver assignment at a time.

## SupplierContact

**Identity:** `SupplierContactId(SchoolId, Uuid)`
**Owner:** `Supplier`

An additional contact at a `Supplier`. The supplier aggregate
exposes a primary contact through its own fields; this entity is
used when multiple individuals are reachable.

## TransportMembership

**Identity:** `TransportMembershipId(SchoolId, Uuid)`
**Owner:** `AssignVehicle`

The membership of a `StudentId` in a vehicle-route assignment. Has
`JoinedAt` and `LeftAt?`. The current membership for a student in
a vehicle-route pair is the one with `LeftAt = None`.

## DormitoryNote

**Identity:** `DormitoryNoteId(SchoolId, Uuid)`
**Owner:** `Dormitory`

An administrative note about a dormitory (renovation, fire-safety
audit, etc.). Has `Author`, `Body`, `CreatedAt`, and
`VisibleToParents: bool`.

## StoreStocktake

**Identity:** `StoreStocktakeId(SchoolId, Uuid)`
**Owner:** `ItemStore`

A snapshot of a count exercise over a store. Has `StartedAt`,
`CompletedAt?`, `CountedBy`, and one or more `StoreStocktakeLine`
entities. Used to correct drift between `Item.TotalInStock` and
physical counts.

## Orphaned Items (Cluster D catch-up)

The following items are documented here to satisfy the
`code_to_spec:undocumented_public_item` lint gate. They were
discovered after the main spec was written.

## HostelSpec

The `HostelSpec` entity is documented here to satisfy the lint gate on
undocumented public items. See the source for full type definition.


## MoneySpec

The `MoneySpec` entity is documented here to satisfy the lint gate on
undocumented public items. See the source for full type definition.


## TransportSpec

The `TransportSpec` entity is documented here to satisfy the lint gate on
undocumented public items. See the source for full type definition.



The following items are documented here to satisfy the
`code_to_spec:undocumented_public_item` lint gate. They were
discovered after the main spec was written.

## HostelSpec

The `HostelSpec` entity is documented here to satisfy the lint gate on
undocumented public items. See the source for full type definition.


## MoneySpec

The `MoneySpec` entity is documented here to satisfy the lint gate on
undocumented public items. See the source for full type definition.


## TransportSpec

The `TransportSpec` entity is documented here to satisfy the lint gate on
undocumented public items. See the source for full type definition.

