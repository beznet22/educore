# Facilities Domain — Workflows

Workflows orchestrate commands, queries, and policies to fulfill a
business goal. They are documented as ordered, conditional steps.

## Vehicle Lifecycle Workflow

```text
1. SchoolAdmin or Transport officer creates a Vehicle (CreateVehicle).
2. SchoolAdmin assigns a driver from the HR roster
   (AssignDriverToVehicle).
3. The vehicle becomes eligible for route assignment in the current
   academic year.
4. SchoolAdmin or Transport officer assigns the vehicle to a Route
   (AssignVehicleToRoute) for the academic year.
5. Vehicle operates in the current year; driver may be reassigned.
6. End of year:
   a. The vehicle is unassigned from the route
      (UnassignVehicleFromRoute) — usually in batch.
   b. The vehicle may be reassigned in the next academic year via a
      new AssignVehicleToRoute.
7. The vehicle may be deactivated (DeactivateVehicle) for
   maintenance or retirement; this blocks new route assignments
   but does not delete history.
8. The vehicle may be deleted (DeleteVehicle) only if no historical
   assignment exists.
```

**Pre-conditions:**
- A `Vehicle` cannot be assigned to a route without a current
  `AcademicYear`.
- A `Vehicle` cannot be assigned to a route if it is currently
  `Retired`.

**Failure paths:**
- Duplicate vehicle number → `ValidationError::UniqueViolation`.
- Driver already assigned to another vehicle →
  `ConflictError::DriverAlreadyAssigned`.
- Vehicle has active assignments → `ConflictError::ActiveAssignment`.

## Route Assignment Workflow

```text
1. SchoolAdmin creates a Route for the current academic year
   (CreateRoute), including ordered stops.
2. SchoolAdmin adds or removes stops as the corridor changes
   (AddStopToRoute / RemoveStopFromRoute).
3. SchoolAdmin assigns one or more vehicles to the route
   (AssignVehicleToRoute).
4. SchoolAdmin or Transport officer assigns students to a specific
   vehicle (AssignStudentToRoute), with optional pickup and drop
   stops.
5. Finance subscribes to StudentAssignedToRoute and applies
   transport fees using the route's fare plus any stop-level
   override.
6. Communication sends a notification to the guardian with the
   pickup time and stop.
7. End of academic year:
   a. SchoolAdmin runs a batch unassignment of all students
      (UnassignStudentFromRoute) per vehicle.
   b. SchoolAdmin unassigns vehicles from routes
      (UnassignVehicleFromRoute).
   c. Route, vehicle, and student histories are retained for
      reporting.
8. New academic year: routes and vehicles are recreated in the
   new year; assignments are reissued.
```

**Edge cases:**
- A student is withdrawn mid-year → transport membership is
  released automatically as a subscriber to `StudentWithdrawn`.
- A route is deleted mid-year → all assignments for the route are
  rejected unless the route is restored.

## Dormitory Allocation Workflow

```text
1. SchoolAdmin defines a RoomType catalog (CreateRoomType) for
   tariff classes.
2. SchoolAdmin creates a Dormitory (CreateDormitory) with intake
   and gender scope.
3. SchoolAdmin creates Rooms under the dormitory (CreateRoom),
   each pinned to a room type.
4. HostelWarden reviews room availability and assigns students
   (AssignStudentToRoom) to specific beds.
5. Finance subscribes to StudentAssignedToRoom and applies hostel
   fees using the room's `cost_per_bed`.
6. Communication sends a confirmation to the guardian.
7. End of academic year:
   a. HostelWarden releases all students
      (UnassignStudentFromRoom).
   b. The dormitory and rooms remain for the next year; new
      assignments are issued.
8. End of life: SchoolAdmin deletes a room (DeleteRoom) only when
   no historical assignment exists; the dormitory is deleted
   (DeleteDormitory) only when no rooms exist.
```

**Edge cases:**
- A room has more beds than the dormitory's intake → command is
  rejected at validation.
- A student is assigned to a bed in a full room → command is
  rejected at validation.
- A dormitory type is changed after students are assigned → the
  change is allowed but flagged in the audit log.

## Inventory Receive Workflow

```text
1. Procurement officer creates a Supplier (CreateSupplier) if
   the vendor is not on file.
2. Procurement officer receives goods from the supplier:
   a. Issue ReceiveItem with the supplier, store, payment, and
      one or more lines.
   b. The system validates each line's item and stock-on-hand
      capacity.
   c. Item.TotalInStock is incremented per line.
   d. The system emits ItemReceived.
3. Finance subscribes to ItemReceived and posts a payable
   against the supplier.
4. The receive record is updated as payment is made
   (UpdateItemReceive); paid status is recomputed.
5. If the receive is cancelled (CancelItemReceive), stock is
   decremented and the payable is reversed.
```

**Pre-conditions:**
- The supplier is active.
- The store is active.
- Every item on the lines is active.

**Edge cases:**
- Partial payment → `paid_status` is `Partial`.
- Receive against a blacklisted supplier → command is rejected.
- Receive date is before the academic year start → command is
  rejected.

## Inventory Issue Workflow

```text
1. InventoryClerk selects a recipient (a role, a staff member, or
   a student) and an item.
2. InventoryClerk issues IssueItem with the quantity and
   optional due-back date.
3. The system validates Item.TotalInStock >= quantity.
4. Item.TotalInStock is decremented; the ItemIssue is created.
5. The system emits ItemIssued.
6. The recipient uses the item; on return, the clerk issues
   ReturnIssuedItem with the returned quantity.
7. Item.TotalInStock is incremented by the returned quantity;
   the status becomes Returned or PartiallyReturned.
8. If the item is lost, the clerk updates the issue status
   (UpdateIssueStatus) to Lost; no stock change.
```

**Pre-conditions:**
- The recipient role is authorized to receive the item
  category (a school policy expressed as a domain service).

**Edge cases:**
- Issue quantity exceeds stock → command is rejected.
- Returned quantity exceeds issued quantity → command is
  rejected.
- Returned quantity is zero → command is rejected.

## Inventory Sell Workflow

```text
1. InventoryClerk prepares an ItemSell for a buyer (a staff
   member or a student) and one or more item lines.
2. The system validates stock and totals.
3. Item.TotalInStock is decremented per line; the ItemSell is
   created.
4. The system emits ItemSold.
5. Finance subscribes to ItemSold and posts the income and any
   receivable.
6. The sale may be updated (UpdateItemSell) for corrections
   before the day closes.
7. The sale may be cancelled (CancelItemSell); stock is reversed.
8. A refund (RefundItemSell) is allowed up to the paid amount;
   it emits ItemSellRefunded.
```

**Pre-conditions:**
- Every line's `item_id` is active.
- `total_paid <= grand_total`.
- `paid_status` is consistent with `total_paid`.

**Edge cases:**
- Sell quantity exceeds stock → command is rejected.
- Refund amount exceeds total paid → command is rejected.
- Cancellation after partial refund → net effect must be
  non-negative.

## Supplier Management Workflow

```text
1. Procurement officer creates a Supplier (CreateSupplier) with
   company name and contact.
2. Finance subscribes to SupplierCreated and registers the
   supplier as a payable counterparty.
3. Procurement uses the supplier on ItemReceive records.
4. Procurement may update the supplier (UpdateSupplier) as
   contacts change.
5. When the supplier is no longer used, Procurement deactivates
   the supplier (DeactivateSupplier). Historical receives remain
   intact; new receives are rejected.
6. When all historical records are cleared, Procurement may
   hard-delete the supplier (DeleteSupplier).
```

## Inventory Reorder & Stocktake Workflow

```text
1. SchoolAdmin or Procurement sets a per-item reorder threshold
   (out of scope for the v1 domain; surfaced as a configuration
   value on the item).
2. A scheduled job queries items with total_in_stock below the
   threshold and emits a procurement task.
3. InventoryClerk performs a stocktake (CreateStoreStocktake)
   that captures physical counts.
4. The system diffs the counts against Item.TotalInStock and
   raises a StockAdjusted event (out of scope for the v1
   domain; surfaced as a corrective receive or issue).
```

## Idempotency

- `CreateVehicle` is idempotent on `vehicle_no` within a school.
  A duplicate returns the existing vehicle.
- `CreateRoute` is idempotent on `(school_id, academic_year_id,
  title)`. A duplicate returns the existing route.
- `AssignVehicleToRoute` is idempotent on `(vehicle_id,
  academic_year_id)`. A duplicate returns the prior assignment.
- `AssignStudentToRoute` is idempotent on `(assign_vehicle_id,
  student_id)`. A duplicate returns the prior membership.
- `AssignStudentToRoom` is idempotent on `(room_id, student_id)`.
  A duplicate returns the prior assignment.
- `ReceiveItem` is not naturally idempotent (each receive is a
  distinct document with its own reference number). However, the
  command accepts an `idempotency_key` so that retries from the
  caller side do not create duplicate receives.
- `IssueItem` and `SellItem` are also documented with
  `idempotency_key` to make retry-safe client workflows possible.

## Reports

The facilities domain exposes read models and reports:

- `TransportRosterReport` — per route, the vehicle and the
  students assigned, with pickup and drop stops.
- `DormitoryOccupancyReport` — per dormitory, the count of
  occupied beds and a list of assigned students.
- `InventoryOnHandReport` — per item, current `total_in_stock`
  with valuation (sum of recent receive prices).
- `InventoryMovementReport` — per item, all receives, issues, and
  sells in a date range, with running balance.
- `SupplierPayableReport` — per supplier, the sum of open
  receives and the corresponding payable balances.
- `ItemSellByBuyerReport` — per buyer, the items sold and the
  totals paid in a date range.
- `OverdueIssuesReport` — issues whose `due_date` is in the past
  and whose status is not `Returned`.
- `LowStockReport` — items whose `total_in_stock` is below a
  configured reorder threshold.

Reports are read-only and do not mutate state. They are produced
either synchronously through the query layer or asynchronously as
materialized views rebuilt from the event log.
