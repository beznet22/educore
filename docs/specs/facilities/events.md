# Facilities Domain — Events

Domain events describe facts that have already happened. They are
immutable, append-only records used for cross-domain integration,
audit, and event sourcing.

All events implement:

```rust
pub trait DomainEvent: Serialize + DeserializeOwned + Send + Sync {
    const TYPE: &'static str;
    fn aggregate_id(&self) -> Uuid;
    fn school_id(&self) -> SchoolId;
    fn occurred_at(&self) -> Timestamp;
}
```

The event envelope wraps the event with metadata:

```rust
pub struct EventEnvelope<E> {
    pub event_id: EventId,
    pub event_type: &'static str,
    pub school_id: SchoolId,
    pub aggregate_id: Uuid,
    pub aggregate_type: &'static str,
    pub actor_id: UserId,
    pub correlation_id: CorrelationId,
    pub causation_id: Option<EventId>,
    pub occurred_at: Timestamp,
    pub payload: E,
}
```

## Transport

### VehicleCreated

```rust
pub struct VehicleCreated {
    pub vehicle_id: VehicleId,
    pub vehicle_no: VehicleNumber,
    pub vehicle_model: VehicleModel,
    pub made_year: Option<MadeYear>,
    pub driver_id: Option<StaffId>,
}
```

### VehicleUpdated

```rust
pub struct VehicleUpdated {
    pub vehicle_id: VehicleId,
    pub changes: Vec<&'static str>,
}
```

### DriverAssignedToVehicle

```rust
pub struct DriverAssignedToVehicle {
    pub vehicle_id: VehicleId,
    pub from_driver_id: Option<StaffId>,
    pub to_driver_id: StaffId,
    pub assigned_at: Timestamp,
}
```

### VehicleDeactivated

```rust
pub struct VehicleDeactivated {
    pub vehicle_id: VehicleId,
    pub reason: DeactivationReason,
    pub new_status: VehicleStatus,
}
```

### VehicleDeleted

```rust
pub struct VehicleDeleted {
    pub vehicle_id: VehicleId,
}
```

## Routes

### RouteCreated

```rust
pub struct RouteCreated {
    pub route_id: RouteId,
    pub academic_year_id: AcademicYearId,
    pub title: RouteName,
    pub fare: Fare,
    pub stops: Vec<RouteStopSpec>,
}
```

### RouteUpdated

```rust
pub struct RouteUpdated {
    pub route_id: RouteId,
    pub changes: Vec<&'static str>,
}
```

### StopAddedToRoute / StopUpdatedOnRoute / StopRemovedFromRoute

```rust
pub struct StopAddedToRoute {
    pub route_id: RouteId,
    pub stop_id: RouteStopId,
    pub stop_order: u32,
    pub stop_name: StopName,
}

pub struct StopUpdatedOnRoute {
    pub route_id: RouteId,
    pub stop_id: RouteStopId,
    pub changes: Vec<&'static str>,
}

pub struct StopRemovedFromRoute {
    pub route_id: RouteId,
    pub stop_id: RouteStopId,
}
```

### RouteDeleted

```rust
pub struct RouteDeleted {
    pub route_id: RouteId,
}
```

## Vehicle-to-Route Assignment

### VehicleAssigned

```rust
pub struct VehicleAssigned {
    pub assign_vehicle_id: AssignVehicleId,
    pub vehicle_id: VehicleId,
    pub route_id: RouteId,
    pub academic_year_id: AcademicYearId,
}
```

**Subscribers:**
- `finance` — prepare transport fees based on the route's fare.

### VehicleUnassigned

```rust
pub struct VehicleUnassigned {
    pub assign_vehicle_id: AssignVehicleId,
    pub vehicle_id: VehicleId,
    pub route_id: RouteId,
    pub academic_year_id: AcademicYearId,
}
```

### StudentAssignedToRoute

```rust
pub struct StudentAssignedToRoute {
    pub assign_vehicle_id: AssignVehicleId,
    pub student_id: StudentId,
    pub pickup_stop_id: Option<RouteStopId>,
    pub drop_stop_id: Option<RouteStopId>,
    pub joined_at: Timestamp,
}
```

**Subscribers:**
- `finance` — apply transport fees with stop-level fare overrides.
- `communication` — notify the guardian of the assignment.
- `attendance` — register the student's expected pick-up and
  drop-off times.

### StudentUnassignedFromRoute

```rust
pub struct StudentUnassignedFromRoute {
    pub assign_vehicle_id: AssignVehicleId,
    pub student_id: StudentId,
    pub left_at: Timestamp,
}
```

## Dormitory & Room

### DormitoryCreated

```rust
pub struct DormitoryCreated {
    pub dormitory_id: DormitoryId,
    pub name: DormitoryName,
    pub dormitory_type: DormitoryType,
    pub intake: Intake,
}
```

### DormitoryUpdated

```rust
pub struct DormitoryUpdated {
    pub dormitory_id: DormitoryId,
    pub changes: Vec<&'static str>,
}
```

### DormitoryDeleted

```rust
pub struct DormitoryDeleted {
    pub dormitory_id: DormitoryId,
}
```

### RoomTypeCreated / RoomTypeUpdated / RoomTypeDeleted

```rust
pub struct RoomTypeCreated { pub room_type_id: RoomTypeId, pub name: RoomTypeName }
pub struct RoomTypeUpdated { pub room_type_id: RoomTypeId, pub changes: Vec<&'static str> }
pub struct RoomTypeDeleted { pub room_type_id: RoomTypeId }
```

### RoomCreated

```rust
pub struct RoomCreated {
    pub room_id: RoomId,
    pub dormitory_id: DormitoryId,
    pub room_number: RoomNumber,
    pub room_type_id: RoomTypeId,
    pub number_of_bed: NumberOfBed,
    pub cost_per_bed: CostPerBed,
}
```

### RoomUpdated / RoomDeleted

```rust
pub struct RoomUpdated { pub room_id: RoomId, pub changes: Vec<&'static str> }
pub struct RoomDeleted { pub room_id: RoomId }
```

### StudentAssignedToRoom

```rust
pub struct StudentAssignedToRoom {
    pub room_id: RoomId,
    pub student_id: StudentId,
    pub bed_number: BedNumber,
    pub assigned_at: Timestamp,
}
```

**Subscribers:**
- `finance` — apply hostel fees using `cost_per_bed`.
- `communication` — notify the guardian of the allocation.

### StudentUnassignedFromRoom

```rust
pub struct StudentUnassignedFromRoom {
    pub room_id: RoomId,
    pub student_id: StudentId,
    pub released_at: Timestamp,
}
```

## Inventory

### ItemCategoryCreated / ItemCategoryUpdated / ItemCategoryDeleted

```rust
pub struct ItemCategoryCreated { pub item_category_id: ItemCategoryId, pub category_name: CategoryName }
pub struct ItemCategoryUpdated { pub item_category_id: ItemCategoryId, pub changes: Vec<&'static str> }
pub struct ItemCategoryDeleted { pub item_category_id: ItemCategoryId }
```

### ItemCreated / ItemUpdated / ItemDeleted

```rust
pub struct ItemCreated {
    pub item_id: ItemId,
    pub item_name: ItemName,
    pub item_sku: ItemSku,
    pub item_category_id: ItemCategoryId,
}

pub struct ItemUpdated { pub item_id: ItemId, pub changes: Vec<&'static str> }
pub struct ItemDeleted { pub item_id: ItemId }
```

### ItemStoreCreated / ItemStoreUpdated / ItemStoreDeleted

```rust
pub struct ItemStoreCreated { pub item_store_id: ItemStoreId, pub store_name: StoreName }
pub struct ItemStoreUpdated { pub item_store_id: ItemStoreId, pub changes: Vec<&'static str> }
pub struct ItemStoreDeleted { pub item_store_id: ItemStoreId }
```

### ItemReceived

```rust
pub struct ItemReceived {
    pub item_receive_id: ItemReceiveId,
    pub supplier_id: SupplierId,
    pub store_id: ItemStoreId,
    pub receive_date: NaiveDate,
    pub grand_total: GrandTotal,
    pub total_quantity: TotalQuantity,
    pub total_paid: TotalPaid,
    pub total_due: TotalDue,
    pub paid_status: PaidStatus,
    pub lines: Vec<ItemReceiveLinePayload>,
}
```

`ItemReceiveLinePayload` carries `item_id`, `unit_price`,
`quantity`, and `sub_total`.

**Subscribers:**
- `finance` — record the payable against the supplier; update the
  expense head.
- `communication` — notify procurement that the GRN is posted.

### ItemReceiveUpdated

```rust
pub struct ItemReceiveUpdated {
    pub item_receive_id: ItemReceiveId,
    pub changes: Vec<&'static str>,
}
```

### ItemReceiveCancelled

```rust
pub struct ItemReceiveCancelled {
    pub item_receive_id: ItemReceiveId,
    pub reason: String,
    pub reversed_lines: Vec<ItemReceiveLinePayload>,
}
```

### ItemIssued

```rust
pub struct ItemIssued {
    pub item_issue_id: ItemIssueId,
    pub item_id: ItemId,
    pub item_category_id: ItemCategoryId,
    pub issue_to: IssueRecipient,
    pub issue_by: UserId,
    pub issue_date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub quantity: ItemQuantity,
    pub note: Option<Note>,
}
```

### ItemIssueStatusUpdated

```rust
pub struct ItemIssueStatusUpdated {
    pub item_issue_id: ItemIssueId,
    pub from_status: IssueStatus,
    pub to_status: IssueStatus,
}
```

### IssuedItemReturned

```rust
pub struct IssuedItemReturned {
    pub item_issue_id: ItemIssueId,
    pub item_id: ItemId,
    pub returned_quantity: ItemQuantity,
    pub new_status: IssueStatus,
}
```

### ItemSold

```rust
pub struct ItemSold {
    pub item_sell_id: ItemSellId,
    pub buyer: IssueRecipient,
    pub role_id: RoleId,
    pub sell_date: NaiveDate,
    pub grand_total: GrandTotal,
    pub total_quantity: TotalQuantity,
    pub total_paid: TotalPaid,
    pub total_due: TotalDue,
    pub paid_status: PaidStatus,
    pub lines: Vec<ItemSellLinePayload>,
}
```

`ItemSellLinePayload` carries `item_id`, `sell_price`, `quantity`,
and `sub_total`.

**Subscribers:**
- `finance` — record the income and the receivable.

### ItemSellUpdated

```rust
pub struct ItemSellUpdated {
    pub item_sell_id: ItemSellId,
    pub changes: Vec<&'static str>,
}
```

### ItemSellCancelled

```rust
pub struct ItemSellCancelled {
    pub item_sell_id: ItemSellId,
    pub reason: String,
    pub reversed_lines: Vec<ItemSellLinePayload>,
}
```

### ItemSellRefunded

```rust
pub struct ItemSellRefunded {
    pub item_sell_id: ItemSellId,
    pub refund_amount: TotalPaid,
    pub new_paid_status: PaidStatus,
}
```

## Suppliers

### SupplierCreated

```rust
pub struct SupplierCreated {
    pub supplier_id: SupplierId,
    pub company_name: SupplierName,
    pub contact_person_name: Option<ContactPersonName>,
    pub contact_person_mobile: Option<PhoneNumber>,
    pub contact_person_email: Option<EmailAddress>,
}
```

**Subscribers:**
- `finance` — register the supplier on the payables counterparty
  list.

### SupplierUpdated

```rust
pub struct SupplierUpdated {
    pub supplier_id: SupplierId,
    pub changes: Vec<&'static str>,
}
```

### SupplierDeactivated

```rust
pub struct SupplierDeactivated {
    pub supplier_id: SupplierId,
    pub reason: String,
    pub new_status: SupplierStatus,
}
```

### SupplierDeleted

```rust
pub struct SupplierDeleted {
    pub supplier_id: SupplierId,
}
```
