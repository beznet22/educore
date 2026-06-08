# Facilities Domain Overview

## Purpose

The facilities domain owns the physical operations of the school beyond
the classroom. It manages student transport, dormitory allocation,
inventory and item movement, and supplier relationships. It anchors
operational assets (vehicles, dormitories, rooms, items, stores) to the
school and coordinates their lifecycle with the academic, finance, and
human-resource domains.

## Responsibilities

- Vehicle registration, assignment, and route linking.
- Route definition with origin, destination, and fare.
- Vehicle assignment to routes per academic year.
- Student transport assignment per vehicle-route pair.
- Dormitory creation, capacity, and gender scoping.
- Room type catalog and room allocation within dormitories.
- Student room assignment and bed allocation.
- Item category catalog and item master records.
- Item stores and stock-on-hand tracking.
- Goods receive notes (GRN) against suppliers and stores.
- Goods issue notes (GIN) to roles, staff, or students.
- Item sale to staff and students with payment tracking.
- Supplier master and contact directory.
- Inventory valuation and reorder triggers.

## Boundaries

The facilities domain does **not** own:

- Student identity, attendance, or promotion (see `specs/academic/`).
- Fee invoicing or payment reconciliation (see `specs/finance/`).
- Staff identity, payroll, or leave (see `specs/hr/`).
- Library books and issues (see `specs/library/`).
- Notification dispatch (see `specs/communication/`).
- Authentication (see `specs/platform/`).

The facilities domain **does** depend on identifier types defined by
the academic and human-resource domains: `StudentId`, `StaffId`,
`ClassId`, `SectionId`, `AcademicYearId`, `RoleId`. It exposes its own
identifier types to consumers: `VehicleId`, `RouteId`,
`AssignVehicleId`, `DormitoryId`, `RoomId`, `RoomTypeId`, `ItemId`,
`ItemCategoryId`, `ItemStoreId`, `ItemIssueId`, `ItemReceiveId`,
`ItemReceiveChildId`, `ItemSellId`, `ItemSellChildId`, `SupplierId`.

## Dependencies

- `smscore-core` — error types, result, identifier trait.
- `smscore-platform` — `SchoolId`, `UserId`, `TenantContext`.
- `smscore-rbac` — capability checks.
- `smscore-events` — domain event publishing.
- `smscore-academic` — `StudentId`, `ClassId`, `SectionId`,
  `AcademicYearId`, `RoleId` (read-only references).
- `smscore-hr` — `StaffId` (read-only references).
- `smscore-finance` — receives payment events for item sales and GRNs.

## Domain Invariants

1. Every vehicle, route, dormitory, room, item, item category, item
   store, item issue, item receive, item receive child, item sell,
   item sell child, and supplier belongs to exactly one school.
2. Every operational aggregate is scoped to one `AcademicYear`. A
   vehicle, dormitory, or item may be reused across academic years
   only by issuing a new assignment record in the new year.
3. A `Vehicle` is uniquely identified by `VehicleNumber` within a
   school.
4. A `Route` is uniquely identified by `RouteName` within a school
   and academic year.
5. A `Vehicle` may be assigned to at most one `Route` per academic
   year through the `AssignVehicle` aggregate.
6. A `Dormitory` has a `DormitoryType` of `Boys` or `Girls` and a
   defined `Intake` capacity. The sum of `Room.NumberOfBed` cannot
   exceed the dormitory's `Intake`.
7. A `Room` is uniquely identified by `RoomNumber` within a
   `Dormitory`. A room has a `RoomType` and a `CostPerBed` value.
8. A `Student` may hold at most one active room assignment in a
   given academic year.
9. An `Item` has a unique `ItemSku` within a school and belongs to
   exactly one `ItemCategory`.
10. `Item.TotalInStock` must equal the sum of `ItemReceiveChild.Quantity`
    minus the sum of `ItemIssue.Quantity` and `ItemSellChild.Quantity`
    for that item in the same academic year.
11. An `ItemIssue` may not be issued if `Item.TotalInStock` is less
    than the requested `Quantity`. The issue atomically decrements
    stock.
12. An `ItemSell` line may not be sold if `Item.TotalInStock` is less
    than the requested `Quantity`. The sale atomically decrements
    stock.
13. An `ItemReceive` must reference a `Supplier` and a `ItemStore`
    and contain at least one `ItemReceiveChild` line.
14. A `Supplier` is uniquely identified by `SupplierName` within a
    school.

## Aggregate Roots

| Aggregate           | Root Type         | Purpose                                    |
| ------------------- | ----------------- | ------------------------------------------ |
| Vehicle             | `Vehicle`         | A school-owned or contracted vehicle       |
| Route               | `Route`           | A transport route with fare                |
| AssignVehicle       | `AssignVehicle`   | Pairing a vehicle to a route in a year     |
| Dormitory           | `Dormitory`       | A hostel building with capacity            |
| Room                | `Room`            | A room within a dormitory                  |
| RoomType            | `RoomType`        | A room type catalog entry                  |
| ItemCategory        | `ItemCategory`    | A grouping for items                       |
| Item                | `Item`            | An inventory master record                 |
| ItemStore           | `ItemStore`       | A physical or virtual store location       |
| ItemIssue           | `ItemIssue`       | Goods issue note (header)                  |
| ItemReceive         | `ItemReceive`     | Goods receive note (header)                |
| ItemReceiveChild    | `ItemReceiveChild`| Goods receive line                         |
| ItemSell            | `ItemSell`        | Sale of an item (header)                   |
| ItemSellChild       | `ItemSellChild`   | Sale of an item (line)                     |
| Supplier            | `Supplier`        | A vendor contact master                    |

Each aggregate is documented in detail under
`docs/specs/facilities/aggregates.md`.

## Cross-Domain Impact

When a `Student` is admitted, the facilities domain does not auto-act.
Transport and dormitory assignment are explicit commands. When a
`Student` is withdrawn, the facilities domain may receive
`StudentWithdrawn` and produces side effects:

- Removes the student from any active `AssignVehicle` membership.
- Releases the student's room assignment.
- Closes any pending `ItemIssue` lines addressed to the student.
- Refunds in-flight `ItemSell` invoices.

When an `AcademicYear` is closed, the facilities domain produces read
snapshots for end-of-year reporting and forces re-assignment of every
operational record into the new year (per the school's continuity
policy).

When a `Supplier` is created, the finance domain is informed so that
the supplier may be used on future expense heads.

When an `ItemReceive` is posted, the finance domain receives
`ItemReceived` and records the payable against the supplier. The
total paid, total due, and payment method are mirrored from finance
back into the receive record.

## Consumers

- Web admin UI (manage vehicles, dormitories, inventory).
- Transport desk app (assign students to vehicles).
- Hostel warden app (allocate rooms, track occupancy).
- Inventory clerk app (receive, issue, sell).
- Procurement app (manage suppliers, raise POs).
- Mobile parent app (view child's room, transport assignment).
- AI agent (transport allocation, dormitory reassignment,
  inventory queries).

## Anti-Goals

- The facilities domain does not render maps, render GPS routes, or
  integrate with mapping providers. Route geometry is a port concern.
- The facilities domain does not capture signatures, photos, or ID
  scans. Identity is a port concern.
- The facilities domain does not run maintenance schedules for
  vehicles. That is a future operations extension.
- The facilities domain does not compute payroll, taxes, or
  accounting entries. Finance subscribes to its events.
- The facilities domain does not own meal plans, warden rosters, or
  visitor logs. Those are out of scope.
