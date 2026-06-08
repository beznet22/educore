# Facilities Domain — Aggregates

## Vehicle

**Root type:** `Vehicle`
**Identity:** `VehicleId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Facilities — Transport

### Purpose

Represents a school-owned or contracted vehicle used to transport
students between home stops and the school.

### Owned Children

- Vehicle assignment records (referenced by `AssignVehicle` aggregate
  in the same year; not owned as a child).

### Invariants

1. A `Vehicle` belongs to exactly one school.
2. `VehicleNumber` is unique within a school.
3. `MadeYear`, if present, is between 1950 and the current calendar
   year.
4. A `Vehicle` may have an optional `DriverId` (a `StaffId` from the
   human-resource domain). The driver is not owned by the vehicle
   aggregate.
5. A `Vehicle` with `ActiveStatus = false` may not be assigned to a
   route in a new academic year.
6. A `Vehicle` cannot be hard-deleted while an `AssignVehicle` row
   references it in any year.

### Commands

- `CreateVehicle`
- `UpdateVehicle`
- `AssignDriverToVehicle`
- `DeactivateVehicle`
- `DeleteVehicle`

### Events

- `VehicleCreated`
- `VehicleUpdated`
- `DriverAssignedToVehicle`
- `VehicleDeactivated`
- `VehicleDeleted`

### Consistency Boundary

All vehicle mutations are serialized through the `Vehicle` aggregate
root. The vehicle is loaded by id, mutated in memory, validated, and
persisted with its events in a single transaction.

---

## Route

**Root type:** `Route`
**Identity:** `RouteId(SchoolId, Uuid)`

### Purpose

A transport route describes a path between a starting area and the
school, carries a fare, and is the unit to which a vehicle is
assigned.

### Invariants

1. A `Route` is uniquely identified by `RouteName` within a school
   and academic year.
2. `Fare` is non-negative.
3. A `Route` may have zero or more `RouteStop` entries; stops are
   ordered by `StopOrder` (a `u32`).
4. A `Route` may not be hard-deleted while an `AssignVehicle` row
   references it in any year.

### Owned Children

- `RouteStop` (zero or more ordered stops).

### Commands

- `CreateRoute`
- `UpdateRoute`
- `AddStopToRoute`
- `UpdateStopOnRoute`
- `RemoveStopFromRoute`
- `DeleteRoute`

### Events

- `RouteCreated`
- `RouteUpdated`
- `StopAddedToRoute`
- `StopUpdatedOnRoute`
- `StopRemovedFromRoute`
- `RouteDeleted`

---

## AssignVehicle

**Root type:** `AssignVehicle`
**Identity:** `AssignVehicleId(SchoolId, Uuid)`

### Purpose

Represents the assignment of a `Vehicle` to a `Route` for a given
`AcademicYear`. It is the join entity that transport desk and
attendance workflows operate on, and the unit through which students
are attached to a vehicle-route pair.

### Invariants

1. A `Vehicle` may be assigned to at most one `Route` per academic
   year.
2. A `Route` may have multiple `Vehicle`s assigned in the same year
   (e.g. two buses on the same corridor).
3. The combination `(vehicle_id, academic_year_id)` is unique.
4. The combination `(route_id, academic_year_id)` is not constrained
   to a single vehicle; the same route may have many assignments.

### Commands

- `AssignVehicleToRoute`
- `UnassignVehicleFromRoute`
- `AssignStudentToRoute`
- `UnassignStudentFromRoute`

### Events

- `VehicleAssigned`
- `VehicleUnassigned`
- `StudentAssignedToRoute`
- `StudentUnassignedFromRoute`

### Consistency Boundary

The assignment aggregate owns the vehicle-route-year pairing. The
student-to-route membership is held as a set of `StudentId` values
on the assignment and mutated by dedicated commands so the event
log records each change.

---

## Dormitory

**Root type:** `Dormitory`
**Identity:** `DormitoryId(SchoolId, Uuid)`

### Purpose

A residential building with a defined intake, gender scope, and
address. Owns its rooms.

### Invariants

1. A `Dormitory` is uniquely identified by `DormitoryName` within a
   school and academic year.
2. `DormitoryType` is one of `Boys` or `Girls`.
3. `Intake` is a positive integer.
4. The sum of `Room.NumberOfBed` across all rooms of a `Dormitory`
   in a year cannot exceed `Intake`.
5. A `Dormitory` may not be hard-deleted while any `Room` references
   it.

### Owned Children

- `Room` (zero or more; see `Room` aggregate).

### Commands

- `CreateDormitory`
- `UpdateDormitory`
- `DeleteDormitory`

### Events

- `DormitoryCreated`
- `DormitoryUpdated`
- `DormitoryDeleted`

---

## Room

**Root type:** `Room`
**Identity:** `RoomId(SchoolId, Uuid)`

### Purpose

A room within a `Dormitory`, with a type, number of beds, and a
per-bed cost.

### Invariants

1. A `Room` is uniquely identified by `RoomNumber` within a
   `Dormitory`.
2. `NumberOfBed` is a positive integer.
3. `CostPerBed` is non-negative.
4. A `Room` is bound to one `RoomType` aggregate.
5. The number of students assigned to a `Room` may not exceed
   `NumberOfBed`.

### Owned Children

- `RoomAssignment` (zero or more; current and historical students
  assigned to a bed in the room).

### Commands

- `CreateRoom`
- `UpdateRoom`
- `DeleteRoom`
- `AssignStudentToRoom`
- `UnassignStudentFromRoom`

### Events

- `RoomCreated`
- `RoomUpdated`
- `RoomDeleted`
- `StudentAssignedToRoom`
- `StudentUnassignedFromRoom`

---

## RoomType

**Root type:** `RoomType`
**Identity:** `RoomTypeId(SchoolId, Uuid)`

### Purpose

A catalog entry describing a room classification (e.g. "Single",
"Double", "Dormitory Style"). Used to group rooms by tariff.

### Invariants

1. A `RoomType` is uniquely named within a school.
2. A `RoomType` may not be deleted while any `Room` references it.

### Commands

- `CreateRoomType`
- `UpdateRoomType`
- `DeleteRoomType`

### Events

- `RoomTypeCreated`
- `RoomTypeUpdated`
- `RoomTypeDeleted`

---

## ItemCategory

**Root type:** `ItemCategory`
**Identity:** `ItemCategoryId(SchoolId, Uuid)`

### Purpose

A grouping of items (e.g. "Stationery", "Lab Equipment",
"Uniforms"). Used for reporting and authorization of issues.

### Invariants

1. `CategoryName` is unique within a school.
2. A `ItemCategory` may not be deleted while any `Item` references
   it.

### Commands

- `CreateItemCategory`
- `UpdateItemCategory`
- `DeleteItemCategory`

### Events

- `ItemCategoryCreated`
- `ItemCategoryUpdated`
- `ItemCategoryDeleted`

---

## Item

**Root type:** `Item`
**Identity:** `ItemId(SchoolId, Uuid)`

### Purpose

An inventory master record, owned by one `ItemCategory`, with a
unique SKU and a maintained `TotalInStock` value.

### Invariants

1. `ItemSku` is unique within a school.
2. `ItemName` is non-empty.
3. `TotalInStock` is non-negative at all times. It is updated only
   by `ReceiveItem`, `IssueItem`, or `SellItem`.
4. An `Item` belongs to exactly one `ItemCategory`.
5. An `Item` may not be deleted while any `ItemIssue`, `ItemReceive`
   or `ItemSell` references it.

### Commands

- `CreateItem`
- `UpdateItem`
- `DeleteItem`

### Events

- `ItemCreated`
- `ItemUpdated`
- `ItemDeleted`

---

## ItemStore

**Root type:** `ItemStore`
**Identity:** `ItemStoreId(SchoolId, Uuid)`

### Purpose

A physical or logical location where items are kept. A school may
operate one or many stores (e.g. "Main Store", "Sports Store",
"Lab Store").

### Invariants

1. `StoreName` is unique within a school.
2. A `ItemStore` may not be deleted while any `ItemReceive`
   references it in any year.

### Commands

- `CreateItemStore`
- `UpdateItemStore`
- `DeleteItemStore`

### Events

- `ItemStoreCreated`
- `ItemStoreUpdated`
- `ItemStoreDeleted`

---

## ItemIssue

**Root type:** `ItemIssue`
**Identity:** `ItemIssueId(SchoolId, Uuid)`

### Purpose

A goods-issue note. Records an issue of a quantity of an `Item` to
a recipient (a role, a staff member, or a student) for a given date
with an optional due-back date.

### Invariants

1. The `ItemIssue` references exactly one `Item` and one
   `ItemCategory`.
2. `Quantity` is positive.
3. `IssueDate` is on or after the academic year start.
4. `IssueStatus` is one of `Issued`, `Returned`, `PartiallyReturned`,
   `Lost`.
5. The recipient is identified by `RoleId` and an optional
   `IssueTo` (a `StudentId` or `StaffId`).
6. Issuing the item decrements `Item.TotalInStock` atomically with
   the creation of this aggregate.

### Owned Children

- None. The aggregate is a header; the line is the `Item` itself.

### Commands

- `IssueItem`
- `UpdateIssueStatus`
- `ReturnIssuedItem`

### Events

- `ItemIssued`
- `ItemIssueStatusUpdated`
- `IssuedItemReturned`

---

## ItemReceive

**Root type:** `ItemReceive`
**Identity:** `ItemReceiveId(SchoolId, Uuid)`

### Purpose

A goods-receive note (GRN). Records the receipt of items from a
`Supplier` into an `ItemStore`, with totals, payment state, and one
or more `ItemReceiveChild` lines.

### Invariants

1. The `ItemReceive` references exactly one `Supplier` and one
   `ItemStore`.
2. The aggregate has at least one `ItemReceiveChild` line at all
   times.
3. `ReceiveDate` is on or after the academic year start.
4. `GrandTotal` equals the sum of `ItemReceiveChild.SubTotal`.
5. `TotalQuantity` equals the sum of `ItemReceiveChild.Quantity`.
6. `TotalPaid + TotalDue == GrandTotal`.
7. `PaidStatus` is one of `Paid`, `Partial`, `Unpaid`.
8. Posting a receive increments `Item.TotalInStock` for each line
   atomically with the creation of the aggregate.

### Owned Children

- `ItemReceiveChild` (one or more line entries).

### Commands

- `ReceiveItem`
- `UpdateItemReceive`
- `CancelItemReceive`

### Events

- `ItemReceived`
- `ItemReceiveUpdated`
- `ItemReceiveCancelled`

---

## ItemReceiveChild

**Root type:** `ItemReceiveChild`
**Identity:** `ItemReceiveChildId(SchoolId, Uuid)`

### Purpose

A single line on an `ItemReceive`. Records the unit price,
quantity, and subtotal for a specific `Item` received in the GRN.

### Invariants

1. The line references exactly one `Item`.
2. `UnitPrice` is non-negative.
3. `Quantity` is positive.
4. `SubTotal == UnitPrice * Quantity` (computed at construction).
5. A line is created atomically with its parent `ItemReceive`.

### Commands

- `ReceiveItem` (composite, creates the line within the parent
  receive command).
- `UpdateItemReceive` may add, edit, or remove lines.

### Events

- `ItemReceived` (emitted by the parent aggregate; the line itself
  does not emit a separate domain event).

---

## ItemSell

**Root type:** `ItemSell`
**Identity:** `ItemSellId(SchoolId, Uuid)`

### Purpose

A sale header. Records the sale of one or more items to a staff
member or a student, with totals, payment state, and one or more
`ItemSellChild` lines.

### Invariants

1. The aggregate references a `RoleId` and an optional buyer
   identifier (`StudentId` or `StaffId`).
2. The aggregate has at least one `ItemSellChild` line at all
   times.
3. `SellDate` is on or after the academic year start.
4. `GrandTotal` equals the sum of `ItemSellChild.SubTotal`.
5. `TotalQuantity` equals the sum of `ItemSellChild.Quantity`.
6. `TotalPaid + TotalDue == GrandTotal`.
7. `PaidStatus` is one of `Paid`, `Partial`, `Unpaid`, `Refunded`.
8. Posting a sale decrements `Item.TotalInStock` for each line
   atomically with the creation of the aggregate.

### Owned Children

- `ItemSellChild` (one or more line entries).

### Commands

- `SellItem`
- `UpdateItemSell`
- `CancelItemSell`
- `RefundItemSell`

### Events

- `ItemSold`
- `ItemSellUpdated`
- `ItemSellCancelled`
- `ItemSellRefunded`

---

## ItemSellChild

**Root type:** `ItemSellChild`
**Identity:** `ItemSellChildId(SchoolId, Uuid)`

### Purpose

A single line on an `ItemSell`. Records the sell price, quantity,
and subtotal for a specific `Item` sold in the sale.

### Invariants

1. The line references exactly one `Item`.
2. `SellPrice` is non-negative.
3. `Quantity` is positive.
4. `SubTotal == SellPrice * Quantity` (computed at construction).
5. A line is created atomically with its parent `ItemSell`.

### Commands

- `SellItem` (composite).
- `UpdateItemSell` may add, edit, or remove lines.

### Events

- `ItemSold` (emitted by the parent aggregate; the line itself does
  not emit a separate domain event).

---

## Supplier

**Root type:** `Supplier`
**Identity:** `SupplierId(SchoolId, Uuid)`

### Purpose

A vendor contact master. Used as the supplier on `ItemReceive`
records and as the counterparty on finance payables.

### Invariants

1. `SupplierName` is unique within a school.
2. `ContactPersonMobile`, if present, is a valid `PhoneNumber`.
3. `ContactPersonEmail`, if present, is a valid `EmailAddress`.
4. A `Supplier` may not be hard-deleted while any `ItemReceive`
   references it in any year.

### Commands

- `CreateSupplier`
- `UpdateSupplier`
- `DeactivateSupplier`
- `DeleteSupplier`

### Events

- `SupplierCreated`
- `SupplierUpdated`
- `SupplierDeactivated`
- `SupplierDeleted`
