# Facilities Domain — Commands

Commands describe intent. They are validated, authorized, and
dispatched to the relevant aggregate. Every command produces zero
or more events that are recorded in the event log.

All commands carry a `TenantContext` (school + actor + correlation)
and are rejected if the actor lacks the required capability.

## Transport

### CreateVehicle

```rust
pub struct CreateVehicleCommand {
    pub tenant: TenantContext,
    pub vehicle_no: VehicleNumber,
    pub vehicle_model: VehicleModel,
    pub made_year: Option<MadeYear>,
    pub driver_id: Option<StaffId>,
    pub note: Option<String>,
}
```

**Capability:** `Vehicle.Create`
**Pre-conditions:** `vehicle_no` is unique within the school. The
`driver_id`, if present, is a current staff member.

**Effects:** Creates a `Vehicle` and emits `VehicleCreated`. If a
`driver_id` is supplied, also emits `DriverAssignedToVehicle`.

### UpdateVehicle

```rust
pub struct UpdateVehicleCommand {
    pub tenant: TenantContext,
    pub vehicle_id: VehicleId,
    pub patch: VehiclePatch,
}
```

`VehiclePatch` carries `vehicle_model`, `made_year`, `note`, and
`active_status`. `vehicle_no` is immutable in the current year;
use a transfer or deactivation flow to change it.

**Capability:** `Vehicle.Update`
**Effects:** Emits `VehicleUpdated`.

### AssignDriverToVehicle

```rust
pub struct AssignDriverToVehicleCommand {
    pub tenant: TenantContext,
    pub vehicle_id: VehicleId,
    pub driver_id: StaffId,
}
```

**Capability:** `Vehicle.AssignDriver`
**Pre-conditions:** Vehicle is active. Driver is a current staff
member. No other vehicle currently has this driver as the primary
driver (use a release command first).

**Effects:** Emits `DriverAssignedToVehicle` and closes any prior
driver assignment for the same vehicle.

### DeactivateVehicle

```rust
pub struct DeactivateVehicleCommand {
    pub tenant: TenantContext,
    pub vehicle_id: VehicleId,
    pub reason: DeactivationReason,
}
```

**Capability:** `Vehicle.Deactivate`
**Effects:** Sets the vehicle's status to `Retired` or
`Maintenance` and emits `VehicleDeactivated`. Active
`AssignVehicle` rows must be closed first or the command is
rejected.

### DeleteVehicle

```rust
pub struct DeleteVehicleCommand {
    pub tenant: TenantContext,
    pub vehicle_id: VehicleId,
}
```

**Capability:** `Vehicle.Delete`
**Pre-conditions:** Vehicle has no `AssignVehicle` rows in any
academic year. Vehicle has no historical driver assignments.

**Effects:** Emits `VehicleDeleted`.

### CreateRoute

```rust
pub struct CreateRouteCommand {
    pub tenant: TenantContext,
    pub academic_year_id: AcademicYearId,
    pub title: RouteName,
    pub fare: Fare,
    pub distance: Option<Distance>,
    pub stops: Vec<RouteStopSpec>,
}
```

**Capability:** `Route.Create`
**Pre-conditions:** `title` is unique within the school and
academic year. Stops, if any, have unique `stop_order` values.

**Effects:** Creates a `Route` with its ordered stops and emits
`RouteCreated`. Each stop emits `StopAddedToRoute`.

### UpdateRoute

```rust
pub struct UpdateRouteCommand {
    pub tenant: TenantContext,
    pub route_id: RouteId,
    pub patch: RoutePatch,
}
```

`RoutePatch` carries mutable fields: `fare`, `distance`, `title`
(title must remain unique within the school-year).

**Capability:** `Route.Update`
**Effects:** Emits `RouteUpdated`.

### AddStopToRoute / UpdateStopOnRoute / RemoveStopFromRoute

```rust
pub struct AddStopToRouteCommand {
    pub tenant: TenantContext,
    pub route_id: RouteId,
    pub stop_order: u32,
    pub stop_name: StopName,
    pub pickup_time: Option<NaiveTime>,
    pub fare_override: Option<Fare>,
}

pub struct UpdateStopOnRouteCommand {
    pub tenant: TenantContext,
    pub route_id: RouteId,
    pub stop_id: RouteStopId,
    pub patch: RouteStopPatch,
}

pub struct RemoveStopFromRouteCommand {
    pub tenant: TenantContext,
    pub route_id: RouteId,
    pub stop_id: RouteStopId,
}
```

**Capabilities:** `Route.AddStop`, `Route.UpdateStop`,
`Route.RemoveStop`.

**Effects:** Emit `StopAddedToRoute`, `StopUpdatedOnRoute`,
`StopRemovedFromRoute`.

### DeleteRoute

```rust
pub struct DeleteRouteCommand {
    pub tenant: TenantContext,
    pub route_id: RouteId,
}
```

**Capability:** `Route.Delete`
**Pre-conditions:** Route has no `AssignVehicle` rows referencing
it in any year.

**Effects:** Emits `RouteDeleted`.

### AssignVehicleToRoute / UnassignVehicleFromRoute

```rust
pub struct AssignVehicleToRouteCommand {
    pub tenant: TenantContext,
    pub vehicle_id: VehicleId,
    pub route_id: RouteId,
    pub academic_year_id: AcademicYearId,
}

pub struct UnassignVehicleFromRouteCommand {
    pub tenant: TenantContext,
    pub assign_vehicle_id: AssignVehicleId,
}
```

**Capabilities:** `Transport.AssignVehicle`,
`Transport.UnassignVehicle`.

**Pre-conditions:** Vehicle is active. Route is active. Vehicle
has no other `AssignVehicle` row for the same academic year.
Unassigning a vehicle with active student memberships is rejected
unless the actor provides a `force: bool` flag, in which case all
memberships are released first.

**Effects:** Emit `VehicleAssigned` and `VehicleUnassigned`.

### AssignStudentToRoute / UnassignStudentFromRoute

```rust
pub struct AssignStudentToRouteCommand {
    pub tenant: TenantContext,
    pub assign_vehicle_id: AssignVehicleId,
    pub student_id: StudentId,
    pub pickup_stop_id: Option<RouteStopId>,
    pub drop_stop_id: Option<RouteStopId>,
}

pub struct UnassignStudentFromRouteCommand {
    pub tenant: TenantContext,
    pub assign_vehicle_id: AssignVehicleId,
    pub student_id: StudentId,
}
```

**Capabilities:** `Transport.AssignStudent`,
`Transport.UnassignStudent`.

**Pre-conditions:** Student is admitted in the current academic
year. The student does not already hold an active transport
membership in any other vehicle-route pair in the same year.

**Effects:** Emit `StudentAssignedToRoute` and
`StudentUnassignedFromRoute`. Finance may subscribe to apply
transport fees on the basis of the route's fare and stop
overrides.

## Dormitory

### CreateRoomType

```rust
pub struct CreateRoomTypeCommand {
    pub tenant: TenantContext,
    pub name: RoomTypeName,
    pub description: Option<String>,
}
```

**Capability:** `RoomType.Create`
**Effects:** Emits `RoomTypeCreated`.

### UpdateRoomType / DeleteRoomType

```rust
pub struct UpdateRoomTypeCommand { ... }
pub struct DeleteRoomTypeCommand {
    pub tenant: TenantContext,
    pub room_type_id: RoomTypeId,
}
```

**Capabilities:** `RoomType.Update`, `RoomType.Delete`.

### CreateDormitory

```rust
pub struct CreateDormitoryCommand {
    pub tenant: TenantContext,
    pub name: DormitoryName,
    pub dormitory_type: DormitoryType,
    pub address: Option<Address>,
    pub intake: Intake,
    pub description: Option<String>,
}
```

**Capability:** `Dormitory.Create`
**Pre-conditions:** `name` is unique within the school. `intake`
is positive.

**Effects:** Emits `DormitoryCreated`.

### UpdateDormitory / DeleteDormitory

```rust
pub struct UpdateDormitoryCommand {
    pub tenant: TenantContext,
    pub dormitory_id: DormitoryId,
    pub patch: DormitoryPatch,
}

pub struct DeleteDormitoryCommand {
    pub tenant: TenantContext,
    pub dormitory_id: DormitoryId,
}
```

`DormitoryPatch` carries `name`, `address`, `intake`, and
`description`. Reducing `intake` below the current sum of
`Room.NumberOfBed` is rejected.

**Capabilities:** `Dormitory.Update`, `Dormitory.Delete`.

### CreateRoom

```rust
pub struct CreateRoomCommand {
    pub tenant: TenantContext,
    pub dormitory_id: DormitoryId,
    pub room_number: RoomNumber,
    pub room_type_id: RoomTypeId,
    pub number_of_bed: NumberOfBed,
    pub cost_per_bed: CostPerBed,
    pub description: Option<String>,
}
```

**Capability:** `Room.Create`
**Pre-conditions:** `room_number` is unique within the dormitory.
`number_of_bed` is positive. The sum of beds across all rooms
(including the new one) does not exceed the dormitory's `intake`.

**Effects:** Emits `RoomCreated`.

### UpdateRoom / DeleteRoom

```rust
pub struct UpdateRoomCommand {
    pub tenant: TenantContext,
    pub room_id: RoomId,
    pub patch: RoomPatch,
}

pub struct DeleteRoomCommand {
    pub tenant: TenantContext,
    pub room_id: RoomId,
}
```

`RoomPatch` carries `room_type_id`, `number_of_bed`,
`cost_per_bed`, and `description`. Reducing `number_of_bed`
below the number of currently-assigned students is rejected.

**Capabilities:** `Room.Update`, `Room.Delete`.

### AssignStudentToRoom / UnassignStudentFromRoom

```rust
pub struct AssignStudentToRoomCommand {
    pub tenant: TenantContext,
    pub room_id: RoomId,
    pub student_id: StudentId,
    pub bed_number: BedNumber,
}

pub struct UnassignStudentFromRoomCommand {
    pub tenant: TenantContext,
    pub room_id: RoomId,
    pub student_id: StudentId,
}
```

**Capabilities:** `Room.AssignStudent`, `Room.UnassignStudent`.

**Pre-conditions:** Student is admitted in the current academic
year. The student does not already hold an active room assignment.
The bed number is in range. The number of currently-assigned
students is less than `number_of_bed`.

**Effects:** Emit `StudentAssignedToRoom` and
`StudentUnassignedFromRoom`. Finance may subscribe to apply
hostel fees on the basis of `cost_per_bed`.

## Inventory

### CreateItemCategory / UpdateItemCategory / DeleteItemCategory

```rust
pub struct CreateItemCategoryCommand {
    pub tenant: TenantContext,
    pub category_name: CategoryName,
}

pub struct UpdateItemCategoryCommand { ... }
pub struct DeleteItemCategoryCommand {
    pub tenant: TenantContext,
    pub item_category_id: ItemCategoryId,
}
```

**Capabilities:** `ItemCategory.Create`, `ItemCategory.Update`,
`ItemCategory.Delete`.

### CreateItem

```rust
pub struct CreateItemCommand {
    pub tenant: TenantContext,
    pub item_name: ItemName,
    pub item_sku: ItemSku,
    pub item_category_id: ItemCategoryId,
    pub description: Option<Description>,
}
```

**Capability:** `Item.Create`
**Pre-conditions:** `item_sku` is unique within the school. The
category exists. Initial `total_in_stock` is zero.

**Effects:** Emits `ItemCreated`.

### UpdateItem / DeleteItem

```rust
pub struct UpdateItemCommand {
    pub tenant: TenantContext,
    pub item_id: ItemId,
    pub patch: ItemPatch,
}

pub struct DeleteItemCommand {
    pub tenant: TenantContext,
    pub item_id: ItemId,
}
```

`ItemPatch` carries `item_name`, `item_category_id`, and
`description`. `item_sku` and `total_in_stock` are not
patchable directly; use `ReceiveItem`, `IssueItem`, or `SellItem`.

**Capabilities:** `Item.Update`, `Item.Delete`.

### CreateItemStore / UpdateItemStore / DeleteItemStore

```rust
pub struct CreateItemStoreCommand {
    pub tenant: TenantContext,
    pub store_name: StoreName,
    pub store_number: Option<StoreNumber>,
    pub description: Option<Description>,
}

pub struct UpdateItemStoreCommand { ... }
pub struct DeleteItemStoreCommand { ... }
```

**Capabilities:** `ItemStore.Create`, `ItemStore.Update`,
`ItemStore.Delete`.

### ReceiveItem

```rust
pub struct ReceiveItemCommand {
    pub tenant: TenantContext,
    pub academic_year_id: AcademicYearId,
    pub receive_date: NaiveDate,
    pub reference_no: Option<ReferenceNumber>,
    pub supplier_id: SupplierId,
    pub store_id: ItemStoreId,
    pub total_paid: TotalPaid,
    pub payment_method: PaymentMethod,
    pub paid_status: PaidStatus,
    pub expense_head_id: Option<ExpenseHeadId>,
    pub account_id: Option<AccountId>,
    pub lines: Vec<ItemReceiveLineSpec>,
    pub description: Option<Description>,
}
```

`ItemReceiveLineSpec` carries `item_id`, `unit_price`,
`quantity`, and an optional `description`.

**Capability:** `Inventory.Receive`
**Pre-conditions:** Supplier exists. Store exists. Every line's
`item_id` exists. `lines` is non-empty. `total_paid <= grand_total`.
`payment_method` and `paid_status` are consistent with
`total_paid` (paid status of `Paid` requires
`total_paid == grand_total`).

**Effects:** Creates the `ItemReceive` and its `ItemReceiveChild`
lines, increments `Item.TotalInStock` for each line, and emits
`ItemReceived`. Finance may subscribe to post the payable against
the supplier.

### UpdateItemReceive / CancelItemReceive

```rust
pub struct UpdateItemReceiveCommand {
    pub tenant: TenantContext,
    pub item_receive_id: ItemReceiveId,
    pub patch: ItemReceivePatch,
}

pub struct CancelItemReceiveCommand {
    pub tenant: TenantContext,
    pub item_receive_id: ItemReceiveId,
    pub reason: String,
}
```

`UpdateItemReceive` may add, edit, or remove lines, and may
update `total_paid`, `payment_method`, and `paid_status`.
Cancellation reverses stock and emits `ItemReceiveCancelled` plus
a finance-side reversal.

**Capabilities:** `Inventory.UpdateReceive`,
`Inventory.CancelReceive`.

### IssueItem

```rust
pub struct IssueItemCommand {
    pub tenant: TenantContext,
    pub academic_year_id: AcademicYearId,
    pub issue_to: IssueRecipient,
    pub issue_by: UserId,
    pub issue_date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub item_category_id: ItemCategoryId,
    pub item_id: ItemId,
    pub quantity: ItemQuantity,
    pub note: Option<Note>,
}
```

**Capability:** `Inventory.Issue`
**Pre-conditions:** Item exists. Item is in the same category.
`item.total_in_stock >= quantity`. `issue_to` is a valid
`IssueRecipient` (a `StaffId`, a `StudentId`, or a `RoleId`).

**Effects:** Creates the `ItemIssue`, decrements
`item.total_in_stock` by `quantity`, and emits `ItemIssued`.

### UpdateIssueStatus / ReturnIssuedItem

```rust
pub struct UpdateIssueStatusCommand {
    pub tenant: TenantContext,
    pub item_issue_id: ItemIssueId,
    pub new_status: IssueStatus,
}

pub struct ReturnIssuedItemCommand {
    pub tenant: TenantContext,
    pub item_issue_id: ItemIssueId,
    pub returned_quantity: ItemQuantity,
}
```

Returning an issued item increments `item.total_in_stock` by the
returned quantity (partial returns are allowed) and emits
`IssuedItemReturned` with the new status. `UpdateIssueStatus` is
used to mark an issue as `Lost` (no stock change) and emits
`ItemIssueStatusUpdated`.

**Capabilities:** `Inventory.UpdateIssue`,
`Inventory.ReturnIssued`.

### SellItem

```rust
pub struct SellItemCommand {
    pub tenant: TenantContext,
    pub academic_year_id: AcademicYearId,
    pub buyer: IssueRecipient,
    pub role_id: RoleId,
    pub sell_date: NaiveDate,
    pub reference_no: Option<ReferenceNumber>,
    pub total_paid: TotalPaid,
    pub payment_method: PaymentMethod,
    pub paid_status: PaidStatus,
    pub income_head_id: Option<IncomeHeadId>,
    pub account_id: Option<AccountId>,
    pub lines: Vec<ItemSellLineSpec>,
    pub description: Option<Description>,
}
```

`ItemSellLineSpec` carries `item_id`, `sell_price`, `quantity`,
and an optional `description`.

**Capability:** `Inventory.Sell`
**Pre-conditions:** Every line's `item_id` exists. `lines` is
non-empty. For each line, `item.total_in_stock >= quantity`.
`total_paid <= grand_total`. `payment_method` and `paid_status`
are consistent with `total_paid`.

**Effects:** Creates the `ItemSell` and its `ItemSellChild` lines,
decrements `item.total_in_stock` for each line, and emits
`ItemSold`. Finance may subscribe to post the income against the
buyer.

### UpdateItemSell / CancelItemSell / RefundItemSell

```rust
pub struct UpdateItemSellCommand { ... }
pub struct CancelItemSellCommand {
    pub tenant: TenantContext,
    pub item_sell_id: ItemSellId,
    pub reason: String,
}
pub struct RefundItemSellCommand {
    pub tenant: TenantContext,
    pub item_sell_id: ItemSellId,
    pub amount: TotalPaid,
}
```

Cancellation reverses stock. A refund may be partial; it emits
`ItemSellRefunded` and posts a finance-side credit. Updates emit
`ItemSellUpdated`.

**Capabilities:** `Inventory.UpdateSell`, `Inventory.CancelSell`,
`Inventory.RefundSell`.

## Suppliers

### CreateSupplier

```rust
pub struct CreateSupplierCommand {
    pub tenant: TenantContext,
    pub company_name: SupplierName,
    pub company_address: Option<Address>,
    pub contact_person_name: Option<ContactPersonName>,
    pub contact_person_mobile: Option<PhoneNumber>,
    pub contact_person_email: Option<EmailAddress>,
    pub contact_person_address: Option<Address>,
    pub description: Option<Description>,
}
```

**Capability:** `Supplier.Create`
**Pre-conditions:** `company_name` is unique within the school.
`contact_person_email`, if present, is a valid `EmailAddress`.
`contact_person_mobile`, if present, is a valid `PhoneNumber`.

**Effects:** Emits `SupplierCreated`.

### UpdateSupplier

```rust
pub struct UpdateSupplierCommand {
    pub tenant: TenantContext,
    pub supplier_id: SupplierId,
    pub patch: SupplierPatch,
}
```

`SupplierPatch` carries all supplier fields except `company_name`
and `school_id`.

**Capability:** `Supplier.Update`
**Effects:** Emits `SupplierUpdated`.

### DeactivateSupplier

```rust
pub struct DeactivateSupplierCommand {
    pub tenant: TenantContext,
    pub supplier_id: SupplierId,
    pub reason: String,
}
```

**Capability:** `Supplier.Deactivate`
**Effects:** Sets `supplier.status` to `Inactive` (or
`Blacklisted`). Emits `SupplierDeactivated`. Active item receives
are not blocked (historical records remain valid), but new
receives against the supplier are rejected.

### DeleteSupplier

```rust
pub struct DeleteSupplierCommand {
    pub tenant: TenantContext,
    pub supplier_id: SupplierId,
}
```

**Capability:** `Supplier.Delete`
**Pre-conditions:** Supplier has no `ItemReceive` records in any
year.

**Effects:** Emits `SupplierDeleted`.
