# Facilities Domain — Repositories

Repositories are ports (Rust traits). Adapters implement them. The
default adapter targets PostgreSQL; an SQLite adapter is provided
for embedded deployments.

All repository methods take a `SchoolId` (or operate on a
typed identifier that already embeds it) and refuse to return
data from another school. Tenant isolation is structural.

## VehicleRepository

```rust
#[async_trait]
pub trait VehicleRepository: Send + Sync {
    async fn get(&self, id: VehicleId) -> Result<Option<Vehicle>>;
    async fn get_by_number(&self, school: SchoolId, vehicle_no: &VehicleNumber) -> Result<Option<Vehicle>>;
    async fn insert(&self, vehicle: &Vehicle) -> Result<()>;
    async fn update(&self, vehicle: &Vehicle) -> Result<()>;
    async fn delete(&self, id: VehicleId) -> Result<()>;
    async fn list(&self, school: SchoolId) -> Result<Vec<Vehicle>>;
    async fn list_active(&self, school: SchoolId) -> Result<Vec<Vehicle>>;
    async fn find_by_driver(&self, school: SchoolId, driver_id: StaffId) -> Result<Option<Vehicle>>;
}
```

## RouteRepository

```rust
#[async_trait]
pub trait RouteRepository: Send + Sync {
    async fn get(&self, id: RouteId) -> Result<Option<Route>>;
    async fn find(&self, school: SchoolId, year: AcademicYearId, title: &RouteName) -> Result<Option<Route>>;
    async fn list(&self, school: SchoolId, year: AcademicYearId) -> Result<Vec<Route>>;
    async fn insert(&self, route: &Route) -> Result<()>;
    async fn update(&self, route: &Route) -> Result<()>;
    async fn delete(&self, id: RouteId) -> Result<()>;
    async fn list_stops(&self, route_id: RouteId) -> Result<Vec<RouteStop>>;
}
```

## AssignVehicleRepository

```rust
#[async_trait]
pub trait AssignVehicleRepository: Send + Sync {
    async fn get(&self, id: AssignVehicleId) -> Result<Option<AssignVehicle>>;
    async fn find(&self, vehicle: VehicleId, year: AcademicYearId) -> Result<Option<AssignVehicle>>;
    async fn list_for_vehicle(&self, vehicle: VehicleId) -> Result<Vec<AssignVehicle>>;
    async fn list_for_route(&self, route: RouteId, year: AcademicYearId) -> Result<Vec<AssignVehicle>>;
    async fn insert(&self, assign: &AssignVehicle) -> Result<()>;
    async fn update(&self, assign: &AssignVehicle) -> Result<()>;
    async fn delete(&self, id: AssignVehicleId) -> Result<()>;
    async fn list_members(&self, assign_vehicle_id: AssignVehicleId) -> Result<Vec<TransportMembership>>;
    async fn add_member(&self, m: &TransportMembership) -> Result<()>;
    async fn remove_member(&self, m: &TransportMembership) -> Result<()>;
}
```

## DormitoryRepository

```rust
#[async_trait]
pub trait DormitoryRepository: Send + Sync {
    async fn get(&self, id: DormitoryId) -> Result<Option<Dormitory>>;
    async fn list(&self, school: SchoolId, year: AcademicYearId) -> Result<Vec<Dormitory>>;
    async fn insert(&self, dorm: &Dormitory) -> Result<()>;
    async fn update(&self, dorm: &Dormitory) -> Result<()>;
    async fn delete(&self, id: DormitoryId) -> Result<()>;
}
```

## RoomRepository

```rust
#[async_trait]
pub trait RoomRepository: Send + Sync {
    async fn get(&self, id: RoomId) -> Result<Option<Room>>;
    async fn list_for_dormitory(&self, dorm: DormitoryId) -> Result<Vec<Room>>;
    async fn find_by_number(&self, dorm: DormitoryId, number: &RoomNumber) -> Result<Option<Room>>;
    async fn insert(&self, room: &Room) -> Result<()>;
    async fn update(&self, room: &Room) -> Result<()>;
    async fn delete(&self, id: RoomId) -> Result<()>;
    async fn list_assignments(&self, room: RoomId) -> Result<Vec<RoomAssignment>>;
    async fn current_assignments(&self, room: RoomId) -> Result<Vec<RoomAssignment>>;
    async fn add_assignment(&self, a: &RoomAssignment) -> Result<()>;
    async fn release_assignment(&self, a: &RoomAssignment) -> Result<()>;
}
```

## RoomTypeRepository

```rust
#[async_trait]
pub trait RoomTypeRepository: Send + Sync {
    async fn get(&self, id: RoomTypeId) -> Result<Option<RoomType>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<RoomType>>;
    async fn insert(&self, rt: &RoomType) -> Result<()>;
    async fn update(&self, rt: &RoomType) -> Result<()>;
    async fn delete(&self, id: RoomTypeId) -> Result<()>;
}
```

## ItemCategoryRepository

```rust
#[async_trait]
pub trait ItemCategoryRepository: Send + Sync {
    async fn get(&self, id: ItemCategoryId) -> Result<Option<ItemCategory>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<ItemCategory>>;
    async fn insert(&self, c: &ItemCategory) -> Result<()>;
    async fn update(&self, c: &ItemCategory) -> Result<()>;
    async fn delete(&self, id: ItemCategoryId) -> Result<()>;
}
```

## ItemRepository

```rust
#[async_trait]
pub trait ItemRepository: Send + Sync {
    async fn get(&self, id: ItemId) -> Result<Option<Item>>;
    async fn get_by_sku(&self, school: SchoolId, sku: &ItemSku) -> Result<Option<Item>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<Item>>;
    async fn list_for_category(&self, school: SchoolId, category: ItemCategoryId) -> Result<Vec<Item>>;
    async fn insert(&self, item: &Item) -> Result<()>;
    async fn update(&self, item: &Item) -> Result<()>;
    async fn delete(&self, id: ItemId) -> Result<()>;
    async fn adjust_stock(&self, id: ItemId, delta: Decimal) -> Result<()>;
}
```

`adjust_stock` performs the atomic `UPDATE ... SET total_in_stock
= total_in_stock + $delta WHERE id = $id AND total_in_stock + $delta
>= 0` to enforce the non-negative-stock invariant under concurrency.

## ItemStoreRepository

```rust
#[async_trait]
pub trait ItemStoreRepository: Send + Sync {
    async fn get(&self, id: ItemStoreId) -> Result<Option<ItemStore>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<ItemStore>>;
    async fn insert(&self, s: &ItemStore) -> Result<()>;
    async fn update(&self, s: &ItemStore) -> Result<()>;
    async fn delete(&self, id: ItemStoreId) -> Result<()>;
}
```

## ItemIssueRepository

```rust
#[async_trait]
pub trait ItemIssueRepository: Send + Sync {
    async fn get(&self, id: ItemIssueId) -> Result<Option<ItemIssue>>;
    async fn list_for_item(&self, item: ItemId) -> Result<Vec<ItemIssue>>;
    async fn list_for_recipient(&self, recipient: IssueRecipient) -> Result<Vec<ItemIssue>>;
    async fn list_overdue(&self, school: SchoolId, as_of: NaiveDate) -> Result<Vec<ItemIssue>>;
    async fn list_open(&self, school: SchoolId) -> Result<Vec<ItemIssue>>;
    async fn insert(&self, issue: &ItemIssue) -> Result<()>;
    async fn update(&self, issue: &ItemIssue) -> Result<()>;
}
```

## ItemReceiveRepository

```rust
#[async_trait]
pub trait ItemReceiveRepository: Send + Sync {
    async fn get(&self, id: ItemReceiveId) -> Result<Option<ItemReceive>>;
    async fn list_for_supplier(&self, supplier: SupplierId) -> Result<Vec<ItemReceive>>;
    async fn list_for_store(&self, store: ItemStoreId) -> Result<Vec<ItemReceive>>;
    async fn list_for_date_range(&self, school: SchoolId, from: NaiveDate, to: NaiveDate) -> Result<Vec<ItemReceive>>;
    async fn insert(&self, receive: &ItemReceive) -> Result<()>;
    async fn update(&self, receive: &ItemReceive) -> Result<()>;
    async fn list_lines(&self, receive: ItemReceiveId) -> Result<Vec<ItemReceiveChild>>;
    async fn insert_line(&self, line: &ItemReceiveChild) -> Result<()>;
    async fn update_line(&self, line: &ItemReceiveChild) -> Result<()>;
    async fn delete_line(&self, line: ItemReceiveChildId) -> Result<()>;
}
```

## ItemSellRepository

```rust
#[async_trait]
pub trait ItemSellRepository: Send + Sync {
    async fn get(&self, id: ItemSellId) -> Result<Option<ItemSell>>;
    async fn list_for_buyer(&self, buyer: IssueRecipient) -> Result<Vec<ItemSell>>;
    async fn list_for_date_range(&self, school: SchoolId, from: NaiveDate, to: NaiveDate) -> Result<Vec<ItemSell>>;
    async fn list_open(&self, school: SchoolId) -> Result<Vec<ItemSell>>;
    async fn insert(&self, sell: &ItemSell) -> Result<()>;
    async fn update(&self, sell: &ItemSell) -> Result<()>;
    async fn list_lines(&self, sell: ItemSellId) -> Result<Vec<ItemSellChild>>;
    async fn insert_line(&self, line: &ItemSellChild) -> Result<()>;
    async fn update_line(&self, line: &ItemSellChild) -> Result<()>;
    async fn delete_line(&self, line: ItemSellChildId) -> Result<()>;
}
```

## SupplierRepository

```rust
#[async_trait]
pub trait SupplierRepository: Send + Sync {
    async fn get(&self, id: SupplierId) -> Result<Option<Supplier>>;
    async fn find_by_name(&self, school: SchoolId, name: &SupplierName) -> Result<Option<Supplier>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<Supplier>>;
    async fn list_active(&self, school: SchoolId) -> Result<Vec<Supplier>>;
    async fn insert(&self, s: &Supplier) -> Result<()>;
    async fn update(&self, s: &Supplier) -> Result<()>;
    async fn delete(&self, id: SupplierId) -> Result<()>;
}
```

## Indexes (recommended)

The default PostgreSQL adapter documents the following indexes;
consumers should declare them in their migrations:

```sql
-- Vehicles
CREATE UNIQUE INDEX ux_vehicles_school_id_vehicle_no
    ON vehicles (school_id, vehicle_no);
CREATE INDEX ix_vehicles_school_id_driver
    ON vehicles (school_id, driver_id) WHERE driver_id IS NOT NULL;
CREATE INDEX ix_vehicles_school_id_active
    ON vehicles (school_id) WHERE active_status = 1;

-- Routes
CREATE UNIQUE INDEX ux_routes_school_id_year_title
    ON routes (school_id, academic_year_id, title);
CREATE INDEX ix_routes_school_id_year
    ON routes (school_id, academic_year_id);

-- AssignVehicle
CREATE UNIQUE INDEX ux_assign_vehicles_vehicle_year
    ON assign_vehicles (vehicle_id, academic_year_id);
CREATE INDEX ix_assign_vehicles_route_year
    ON assign_vehicles (route_id, academic_year_id);
CREATE INDEX ix_assign_vehicles_school_id_year
    ON assign_vehicles (school_id, academic_year_id);

-- Dormitories
CREATE UNIQUE INDEX ux_dormitories_school_id_year_name
    ON dormitories (school_id, academic_year_id, dormitory_name);

-- Rooms
CREATE UNIQUE INDEX ux_rooms_dormitory_id_number
    ON rooms (dormitory_id, number);
CREATE INDEX ix_rooms_school_id_dormitory
    ON rooms (school_id, dormitory_id);
CREATE INDEX ix_rooms_school_id_room_type
    ON rooms (school_id, room_type_id);

-- Item catalog
CREATE UNIQUE INDEX ux_items_school_id_sku
    ON items (school_id, item_sku);
CREATE INDEX ix_items_school_id_category
    ON items (school_id, item_category_id);
CREATE INDEX ix_items_school_id_low_stock
    ON items (school_id, total_in_stock);

-- Item stores
CREATE UNIQUE INDEX ux_item_stores_school_id_name
    ON item_stores (school_id, store_name);

-- Item issues
CREATE INDEX ix_item_issues_school_id_item
    ON item_issues (school_id, item_id);
CREATE INDEX ix_item_issues_school_id_due_date
    ON item_issues (school_id, due_date) WHERE due_date IS NOT NULL;
CREATE INDEX ix_item_issues_school_id_status
    ON item_issues (school_id, issue_status);

-- Item receives
CREATE INDEX ix_item_receives_school_id_supplier
    ON item_receives (school_id, supplier_id);
CREATE INDEX ix_item_receives_school_id_store
    ON item_receives (school_id, store_id);
CREATE INDEX ix_item_receives_school_id_date
    ON item_receives (school_id, receive_date);
CREATE INDEX ix_item_receive_children_school_id_receive
    ON item_receive_children (school_id, item_receive_id);
CREATE INDEX ix_item_receive_children_school_id_item
    ON item_receive_children (school_id, item_id);

-- Item sells
CREATE INDEX ix_item_sells_school_id_buyer
    ON item_sells (school_id, student_staff_id);
CREATE INDEX ix_item_sells_school_id_date
    ON item_sells (school_id, sell_date);
CREATE INDEX ix_item_sells_school_id_paid_status
    ON item_sells (school_id, paid_status);
CREATE INDEX ix_item_sell_children_school_id_sell
    ON item_sell_children (school_id, item_sell_id);
CREATE INDEX ix_item_sell_children_school_id_item
    ON item_sell_children (school_id, item_id);

-- Suppliers
CREATE UNIQUE INDEX ux_suppliers_school_id_name
    ON suppliers (school_id, company_name);
CREATE INDEX ix_suppliers_school_id_status
    ON suppliers (school_id, status);
```

The `school_id` predicate is mandatory for tenant isolation. All
queries are rewritten by the storage adapter to add
`school_id = $1` automatically.
