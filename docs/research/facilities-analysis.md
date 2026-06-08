# Facilities Domain — Business Analysis

## Purpose

The facilities domain owns the school's physical
resources beyond classrooms: transport, dormitory
(hostel), and inventory. It is the school's
logistics and operations backbone.

This document describes how transport, dormitory,
and inventory work in real schools, with the edge
cases that real schools hit.

## Key Concepts

- **TransportRoute** — a bus route serving a
  geographic area.
- **TransportVehicle** — a bus or van assigned to a
  route.
- **TransportStop** — a pickup / drop-off point on a
  route.
- **TransportAssignment** — a student's assignment to
  a route.
- **Hostel** — a dormitory building (boys, girls, or
  mixed).
- **HostelRoom** — a room in a hostel.
- **RoomAssignment** — a student's assignment to a
  room.
- **ItemCategory** — a category of inventory item
  (stationery, furniture, lab equipment, etc.).
- **Item** — an inventory item with a quantity, unit
  price, and location.
- **ItemIssue** — an issuance of an item to a staff
  member or student.
- **ItemPurchase** — a purchase of items from a
  vendor.

## Real-World Scenarios

### Transport Route Management

A school operates bus routes to bring students from
their homes to school and back. The transport
officer:

1. Defines a `TransportRoute` with a name, a fare,
   and a list of stops.
2. Assigns a `TransportVehicle` (bus) to the route.
3. Assigns a driver and a conductor (staff
   members).
4. Defines the route's schedule (morning pickup
   times, afternoon drop-off times).

The engine's `TransportRoute` aggregate captures
the route. The `TransportStop` aggregate captures
each stop with the pickup and drop-off times.

### Student Transport Assignment

A parent opts in to transport for their child.
The transport officer:

1. Records the child's pickup and drop-off
   locations.
2. Assigns the child to a `TransportRoute`.
3. Records the fare (per month, per term, or per
   year).
4. The engine creates a `TransportAssignment`
   linking the student to the route.

The finance domain subscribes to the
`TransportAssignmentCreated` event and adds the
transport fee to the student's fees assignment.

### Transport Vehicle Capacity

A bus has a capacity of 40 students. The engine
warns if a route is over-assigned. The transport
officer may:
- Add another vehicle to the route.
- Split the route into two.
- Move students to a different route.

The engine's `TransportVehicle` aggregate has a
`capacity` field; the engine validates on
assignment.

### Transport Fare

A school's transport fare varies by distance:
- Up to 5 km: ₹500 / month.
- 5-10 km: ₹800 / month.
- Over 10 km: ₹1,200 / month.

The engine's `TransportRoute` has a `fare` field.
The transport officer sets the fare per route. The
finance domain uses the fare for fees assignment.

### Transport Attendance

The transport conductor marks which students
boarded the bus in the morning. The engine's
`TransportAttendance` aggregate records the
attendance. The attendance domain may consume
this to update the daily attendance.

### Transport Incident

A bus is involved in a minor accident. The
transport officer files an `Incident` (in the
events domain) with type `Transport`. The
incident is linked to the route and the vehicle.

### Hostel Room Management

A school has a hostel for students whose families
live far away. The hostel warden:

1. Defines a `Hostel` (e.g. "Boys Hostel A",
   "Girls Hostel B").
2. Defines `HostelRoom`s with capacity (e.g. 4
   students per room).
3. Assigns students to rooms.

The engine's `Hostel` and `HostelRoom` aggregates
capture the structure. The `RoomAssignment`
captures the per-student assignment.

### Hostel Fees

Hostel residents pay a hostel fee. The fee covers
room, food, and utilities. The engine's
`Hostel` aggregate has a `fee` field. The finance
domain subscribes to `RoomAssignmentCreated` and
adds the hostel fee to the student's fees.

### Hostel Attendance

The hostel warden takes roll call at night. The
engine's `HostelAttendance` aggregate records
the attendance. The warden can see who is in the
hostel at any time.

### Hostel Leave

A hostel student goes home for the weekend. The
warden records the leave. The engine's
`HostelLeave` aggregate captures the leave. The
hostel attendance auto-marks the student as
"Leave" for the duration.

### Inventory Management

A school maintains an inventory of physical items:
- Stationery (pencils, notebooks, registers).
- Furniture (desks, chairs, blackboards).
- Lab equipment (microscopes, beakers).
- Sports equipment (footballs, cricket bats).
- Cleaning supplies.

The inventory officer:
1. Defines `ItemCategory`s (e.g. "Stationery",
   "Furniture").
2. Adds `Item`s to the inventory with quantity,
   unit price, and location (e.g. "Science Lab
   Cabinet 3").
3. Records purchases from vendors.
4. Records issues to staff or students.
5. Records returns.
6. Records write-offs (damaged, lost).

The engine's `Item` aggregate has a `quantity`
field. The `ItemIssue` reduces the quantity; the
`ItemPurchase` increases it. The engine
maintains the running balance.

### Inventory Threshold Alert

An item's quantity falls below a threshold (e.g.
"only 5 notebooks left"). The engine's
`InventoryAlert` is generated. The inventory
officer is notified to reorder.

### Inventory Audit

The school performs a physical inventory audit
at the end of the year. The audit compares the
engine's recorded quantity with the physical
count. Discrepancies are recorded. The engine's
`InventoryAdjustment` captures the audit.

### Inventory Item Depreciation

A school's furniture depreciates over time. The
engine's `Item` has a `purchase_date` and a
`depreciation_rate`. The engine's reports compute
the current book value.

## Business Rules

1. A `TransportRoute` is unique by `(school_id,
   name)`.
2. A `TransportVehicle` has a `capacity` that the
   engine validates on assignment.
3. A `TransportAssignment` is unique by
   `(school_id, student_id, academic_year_id)`.
4. A `HostelRoom`'s occupancy does not exceed
   its capacity.
5. A `RoomAssignment` is unique by
   `(school_id, student_id, academic_year_id)`.
6. An `Item`'s quantity is non-negative.
7. An `ItemIssue` does not exceed the available
   quantity.
8. An `Item`'s `quantity` is the running balance;
   it is not directly writable except by issues
   and purchases.
9. A `TransportRoute` cannot be deleted while
   active `TransportAssignment`s reference it.
10. A `Hostel` cannot be deleted while active
    `RoomAssignment`s reference it.

## Edge Cases

### Bus Breakdown

A bus breaks down mid-route. The transport
officer arranges an alternate vehicle. The
engine's `TransportRoute` events capture the
incident. Parents are notified (via the
communication domain).

### Student Who Misses the Bus

A student misses the morning bus. The parent
brings the child to school. The engine's
transport attendance marks the student as
"Missed bus." The daily attendance marks the
student as "Present" (since they arrived
independently).

### Hostel Student with Extended Leave

A hostel student goes home for a month. The
hostel warden records the leave. The hostel fee
may be paused (per school policy). The engine's
leave application captures the duration.

### Inventory Write-Off

A lab microscope is broken. The science teacher
reports it. The inventory officer records a
write-off with a reason. The engine's
`ItemWriteOff` reduces the quantity.

### Inventory Discrepancy

The physical count of chairs is 5 less than the
engine's record. The audit captures the
discrepancy with a reason. The engine's
`InventoryAdjustment` updates the quantity.

### Transport Route Split

A route becomes too long. The transport officer
splits it into two routes. The engine's
`TransportRouteSplit` event captures the change.
The students on the affected stops are re-
assigned to the new route.

### Hostel Room Change

A student changes rooms. The warden records the
move. The engine's `RoomAssignment` is closed
for the old room and opened for the new room.
The audit log captures the move.

### Multiple Items in One Issue

A teacher needs 5 laptops for a class. The
inventory officer issues 5 laptops in one
transaction. The engine's bulk issue is all-or-
nothing.

### Inventory with No Vendor

An item is donated to the school. The inventory
officer records the item with a "donation"
source. The engine's `Item` aggregate supports a
`source` field.

## Notes for SMScore Implementation

- The **facilities** crate depends on
  `smscore-academic` for `StudentId`, `ClassId`,
  `SectionId`, and `smscore-hr` for `StaffId`.
- The domain's **transport** sub-aggregate is
  tightly coupled to finance (transport fees)
  and attendance (transport attendance). The
  cross-domain coordination is event-driven.
- The domain's **hostel** sub-aggregate is
  tightly coupled to finance (hostel fees) and
  attendance (hostel attendance). The cross-
  domain coordination is event-driven.
- The domain's **inventory** sub-aggregate
  maintains the running balance per item. The
  engine's `Item` is the canonical record.
- The domain's bulk operations are
  **all-or-nothing**. A single validation
  failure aborts the batch.
- The domain's events (`TransportAssignmentCreated`,
  `RoomAssignmentCreated`, `ItemIssued`,
  `ItemPurchased`, `ItemWriteOff`) drive
  downstream projections and finance fees.
- The domain's reports are
  **capability-gated**. The transport officer
  reads transport reports; the warden reads
  hostel reports; the inventory officer reads
  inventory reports.
- The domain's audit log captures every
  change. The audit log is the canonical
  record for compliance and dispute
  resolution.
