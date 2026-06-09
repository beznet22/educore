# Facilities Domain — Tables

The facilities domain is backed by the following tables. Each
table maps to one or more aggregates; the `aggregate` column
tells you which aggregate owns the row.

| Table                            | Aggregate              | Notes                                    |
| -------------------------------- | ---------------------- | ---------------------------------------- |
| `facilities_vehicles`                    | Vehicle                | Vehicle master                           |
| `facilities_routes`                      | Route                  | Route master                             |
| `facilities_route_stops`                 | RouteStop              | Ordered stops on a route                 |
| `facilities_assign_vehicles`             | AssignVehicle          | Vehicle-route-year pairing               |
| `facilities_transport_memberships`       | TransportMembership    | Student-in-assignment membership         |
| `facilities_dormitories`             | Dormitory              | Dormitory master                         |
| `facilities_room_types`                  | RoomType               | Room type catalog                        |
| `facilities_rooms`                  | Room                   | Rooms under a dormitory                  |
| `facilities_room_assignments`            | RoomAssignment         | Student-to-bed allocations               |
| `facilities_item_categories`             | ItemCategory           | Item grouping                            |
| `facilities_items`                       | Item                   | Item master with stock-on-hand           |
| `facilities_item_stores`                 | ItemStore              | Store catalog                            |
| `facilities_item_issues`                 | ItemIssue              | Goods issue note (header)                |
| `facilities_item_receives`               | ItemReceive            | Goods receive note (header)              |
| `facilities_item_receive_children`       | ItemReceiveChild       | Goods receive line                       |
| `facilities_item_sells`                  | ItemSell               | Item sale (header)                       |
| `facilities_item_sell_children`          | ItemSellChild          | Item sale line                           |
| `facilities_suppliers`                   | Supplier               | Vendor master                            |
| `facilities_supplier_contacts`           | SupplierContact        | Additional supplier contacts             |
| `facilities_driver_assignments`          | DriverAssignment       | Driver-to-vehicle history                |
| `facilities_store_stocktakes`            | StoreStocktake         | Stocktake header                         |
| `facilities_store_stocktake_lines`       | StoreStocktake line    | Stocktake line                           |
| `facilities_dormitory_notes`             | DormitoryNote          | Administrative notes on a dormitory      |

## Field Mapping

The canonical field mapping (from the migration schema) is
documented below. Storage adapters MAY use the same column names
when implementing the port. The engine is column-name agnostic;
the mapping is a recommendation, not a requirement.

### Vehicle

| Column            | Type                | Maps to                              |
| ----------------- | ------------------- | ------------------------------------ |
| `id`              | `u64` / `Uuid`      | `VehicleId`                          |
| `vehicle_no`      | `VARCHAR(255)`      | `VehicleNumber`                      |
| `vehicle_model`   | `VARCHAR(255)`      | `VehicleModel`                       |
| `made_year`       | `INT`               | `MadeYear`                           |
| `note`            | `TEXT`              | `Note`                               |
| `active_status`   | `TINYINT`           | `ActiveStatus`                       |
| `driver_id`       | `u64`               | `DriverId` (StaffId)                 |
| `school_id`       | `u64`               | `SchoolId`                           |
| `academic_id`     | `u64`               | `AcademicYearId`                     |
| `created_at`      | `TIMESTAMP`         | engine-managed                       |
| `updated_at`      | `TIMESTAMP`         | engine-managed                       |
| `created_by`      | `u64`               | engine-managed                       |
| `updated_by`      | `u64`               | engine-managed                       |

### Route

| Column            | Type                | Maps to                              |
| ----------------- | ------------------- | ------------------------------------ |
| `id`              | `u64` / `Uuid`      | `RouteId`                            |
| `title`           | `VARCHAR(200)`      | `RouteName`                          |
| `fare`            | `DECIMAL`           | `Fare`                               |
| `distance`        | `DECIMAL`           | `Distance`                           |
| `active_status`   | `TINYINT`           | `ActiveStatus`                       |
| `school_id`       | `u64`               | `SchoolId`                           |
| `academic_id`     | `u64`               | `AcademicYearId`                     |

### RouteStop

| Column            | Type                | Maps to                              |
| ----------------- | ------------------- | ------------------------------------ |
| `id`              | `u64` / `Uuid`      | `RouteStopId`                        |
| `route_id`        | `u64`               | `RouteId`                            |
| `stop_order`      | `INT`               | order                                |
| `stop_name`       | `VARCHAR`           | `StopName`                           |
| `pickup_time`     | `TIME`              | optional pickup time                 |
| `fare_override`   | `DECIMAL`           | optional fare override               |
| `school_id`       | `u64`               | `SchoolId`                           |

### AssignVehicle

| Column            | Type                | Maps to                              |
| ----------------- | ------------------- | ------------------------------------ |
| `id`              | `u64` / `Uuid`      | `AssignVehicleId`                    |
| `vehicle_id`      | `u64`               | `VehicleId`                          |
| `route_id`        | `u64`               | `RouteId`                            |
| `school_id`       | `u64`               | `SchoolId`                           |
| `academic_id`     | `u64`               | `AcademicYearId`                     |
| `active_status`   | `TINYINT`           | `ActiveStatus`                       |

### Dormitory

| Column            | Type                | Maps to                              |
| ----------------- | ------------------- | ------------------------------------ |
| `id`              | `u64` / `Uuid`      | `DormitoryId`                        |
| `dormitory_name`  | `VARCHAR(200)`      | `DormitoryName`                      |
| `type`            | `CHAR(1)`           | `DormitoryType` (`B`/`G`)            |
| `address`         | `VARCHAR(191)`      | `Address`                            |
| `intake`          | `INT`               | `Intake`                             |
| `description`     | `TEXT`              | `Description`                        |
| `active_status`   | `TINYINT`           | `ActiveStatus`                       |
| `school_id`       | `u64`               | `SchoolId`                           |
| `academic_id`     | `u64`               | `AcademicYearId`                     |

### RoomType

| Column            | Type                | Maps to                              |
| ----------------- | ------------------- | ------------------------------------ |
| `id`              | `u64` / `Uuid`      | `RoomTypeId`                         |
| `type`            | `VARCHAR(255)`      | `RoomTypeName`                       |
| `description`     | `TEXT`              | `Description`                        |
| `active_status`   | `TINYINT`           | `ActiveStatus`                       |
| `school_id`       | `u64`               | `SchoolId`                           |
| `academic_id`     | `u64`               | `AcademicYearId`                     |

### Room

| Column            | Type                | Maps to                              |
| ----------------- | ------------------- | ------------------------------------ |
| `id`              | `u64` / `Uuid`      | `RoomId`                             |
| `name`            | `VARCHAR(255)`      | `RoomNumber`                         |
| `number_of_bed`   | `INT`               | `NumberOfBed`                        |
| `cost_per_bed`    | `DECIMAL(16,2)`     | `CostPerBed`                         |
| `description`     | `TEXT`              | `Description`                        |
| `dormitory_id`    | `u64`               | `DormitoryId`                        |
| `room_type_id`    | `u64`               | `RoomTypeId`                         |
| `active_status`   | `TINYINT`           | `ActiveStatus`                       |
| `school_id`       | `u64`               | `SchoolId`                           |
| `academic_id`     | `u64`               | `AcademicYearId`                     |

### ItemCategory

| Column            | Type                | Maps to                              |
| ----------------- | ------------------- | ------------------------------------ |
| `id`              | `u64` / `Uuid`      | `ItemCategoryId`                     |
| `category_name`   | `VARCHAR(100)`      | `CategoryName`                       |
| `school_id`       | `u64`               | `SchoolId`                           |
| `academic_id`     | `u64`               | `AcademicYearId`                     |

### Item

| Column            | Type                | Maps to                              |
| ----------------- | ------------------- | ------------------------------------ |
| `id`              | `u64` / `Uuid`      | `ItemId`                             |
| `item_name`       | `VARCHAR(100)`      | `ItemName`                           |
| `total_in_stock`  | `DOUBLE(8,2)`       | `StockOnHand`                        |
| `item_sku`        | `VARCHAR`           | `ItemSku`                            |
| `description`     | `VARCHAR(500)`      | `Description`                        |
| `item_category_id`| `u64`               | `ItemCategoryId`                     |
| `school_id`       | `u64`               | `SchoolId`                           |
| `academic_id`     | `u64`               | `AcademicYearId`                     |

### ItemStore

| Column            | Type                | Maps to                              |
| ----------------- | ------------------- | ------------------------------------ |
| `id`              | `u64` / `Uuid`      | `ItemStoreId`                        |
| `store_name`      | `VARCHAR(100)`      | `StoreName`                          |
| `store_no`        | `VARCHAR(100)`      | `StoreNumber`                        |
| `description`     | `VARCHAR(500)`      | `Description`                        |
| `school_id`       | `u64`               | `SchoolId`                           |
| `academic_id`     | `u64`               | `AcademicYearId`                     |

### ItemIssue

| Column            | Type                | Maps to                              |
| ----------------- | ------------------- | ------------------------------------ |
| `id`              | `u64` / `Uuid`      | `ItemIssueId`                        |
| `issue_to`        | `u64`               | recipient (StudentId/StaffId)         |
| `issue_by`        | `u64`               | `UserId`                             |
| `issue_date`      | `DATE`              | `NaiveDate`                          |
| `due_date`        | `DATE`              | optional due-back date               |
| `quantity`        | `INT UNSIGNED`      | `ItemQuantity`                       |
| `issue_status`    | `VARCHAR(191)`      | `IssueStatus`                        |
| `note`            | `VARCHAR(500)`      | `Note`                               |
| `role_id`         | `u64`               | `RoleId`                             |
| `item_category_id`| `u64`               | `ItemCategoryId`                     |
| `item_id`         | `u64`               | `ItemId`                             |
| `active_status`   | `TINYINT`           | `ActiveStatus`                       |
| `school_id`       | `u64`               | `SchoolId`                           |
| `academic_id`     | `u64`               | `AcademicYearId`                     |

### ItemReceive

| Column            | Type                | Maps to                              |
| ----------------- | ------------------- | ------------------------------------ |
| `id`              | `u64` / `Uuid`      | `ItemReceiveId`                      |
| `receive_date`    | `DATE`              | `NaiveDate`                          |
| `reference_no`    | `VARCHAR(191)`      | `ReferenceNumber`                    |
| `grand_total`     | `DECIMAL(20,2)`     | `GrandTotal`                         |
| `total_quantity`  | `DECIMAL(20,2)`     | `TotalQuantity`                      |
| `total_paid`      | `DECIMAL(20,2)`     | `TotalPaid`                          |
| `total_due`       | `DECIMAL(20,2)`     | `TotalDue`                           |
| `expense_head_id` | `INT`               | `ExpenseHeadId` (finance)            |
| `account_id`      | `INT`               | `AccountId` (finance)                |
| `payment_method`  | `VARCHAR(191)`      | `PaymentMethod`                      |
| `paid_status`     | `VARCHAR(191)`      | `PaidStatus`                         |
| `supplier_id`     | `u64`               | `SupplierId`                         |
| `store_id`        | `u64`               | `ItemStoreId`                        |
| `school_id`       | `u64`               | `SchoolId`                           |
| `academic_id`     | `u64`               | `AcademicYearId`                     |

### ItemReceiveChild

| Column            | Type                | Maps to                              |
| ----------------- | ------------------- | ------------------------------------ |
| `id`              | `u64` / `Uuid`      | `ItemReceiveChildId`                 |
| `unit_price`      | `DECIMAL(20,2)`     | `UnitPrice`                          |
| `quantity`        | `DECIMAL(20,2)`     | `ItemQuantity`                       |
| `sub_total`       | `DECIMAL(20,2)`     | `SubTotal`                           |
| `description`     | `VARCHAR(500)`      | `Description`                        |
| `item_id`         | `u64`               | `ItemId`                             |
| `item_receive_id` | `u64`               | `ItemReceiveId`                      |
| `school_id`       | `u64`               | `SchoolId`                           |
| `academic_id`     | `u64`               | `AcademicYearId`                     |

### ItemSell

| Column            | Type                | Maps to                              |
| ----------------- | ------------------- | ------------------------------------ |
| `id`              | `u64` / `Uuid`      | `ItemSellId`                         |
| `student_staff_id`| `INT`               | buyer (StudentId/StaffId)            |
| `sell_date`       | `DATE`              | `NaiveDate`                          |
| `reference_no`    | `VARCHAR(50)`       | `ReferenceNumber`                    |
| `grand_total`     | `DECIMAL(20,2)`     | `GrandTotal`                         |
| `total_quantity`  | `DECIMAL(20,2)`     | `TotalQuantity`                      |
| `total_paid`      | `DECIMAL(20,2)`     | `TotalPaid`                          |
| `total_due`       | `DECIMAL(20,2)`     | `TotalDue`                           |
| `income_head_id`  | `INT`               | `IncomeHeadId` (finance)             |
| `account_id`      | `INT`               | `AccountId` (finance)                |
| `payment_method`  | `VARCHAR(191)`      | `PaymentMethod`                      |
| `paid_status`     | `VARCHAR(191)`      | `PaidStatus`                         |
| `role_id`         | `u64`               | `RoleId`                             |
| `school_id`       | `u64`               | `SchoolId`                           |
| `academic_id`     | `u64`               | `AcademicYearId`                     |

### ItemSellChild

| Column            | Type                | Maps to                              |
| ----------------- | ------------------- | ------------------------------------ |
| `id`              | `u64` / `Uuid`      | `ItemSellChildId`                    |
| `sell_price`      | `DECIMAL(20,2)`     | `SellPrice`                          |
| `quantity`        | `DECIMAL(20,2)`     | `ItemQuantity`                       |
| `sub_total`       | `DECIMAL(20,2)`     | `SubTotal`                           |
| `description`     | `VARCHAR(500)`      | `Description`                        |
| `item_sell_id`    | `u64`               | `ItemSellId`                         |
| `item_id`         | `u64`               | `ItemId`                             |
| `school_id`       | `u64`               | `SchoolId`                           |
| `academic_id`     | `u64`               | `AcademicYearId`                     |

### Supplier

| Column                 | Type                | Maps to                              |
| ---------------------- | ------------------- | ------------------------------------ |
| `id`                   | `u64` / `Uuid`      | `SupplierId`                         |
| `company_name`         | `VARCHAR(100)`      | `SupplierName`                       |
| `company_address`      | `VARCHAR(500)`      | `Address`                            |
| `contact_person_name`  | `VARCHAR(191)`      | `ContactPersonName`                  |
| `contact_person_mobile`| `VARCHAR(191)`      | `PhoneNumber`                        |
| `contact_person_email` | `VARCHAR(100)`      | `EmailAddress`                       |
| `cotact_person_address`| `VARCHAR(500)`      | `Address` (typo preserved for parity)|
| `description`          | `VARCHAR(500)`      | `Description`                        |
| `active_status`        | `TINYINT`           | `ActiveStatus`                       |
| `school_id`            | `u64`               | `SchoolId`                           |
| `academic_id`          | `u64`               | `AcademicYearId`                     |

## Notes

- Every table includes `school_id` for multi-tenant isolation. The
  `school_id` is `NOT NULL` and indexed.
- Every table includes `created_at`, `updated_at`, `created_by`,
  `updated_by`, `active_status` (where applicable). These are
  managed by the engine's storage adapter.
- `academic_id` references `academic_academic_years` (the per-year
  scope) and exists on every operational table.
- Monetary fields use `DECIMAL(20,2)`. The engine treats them as
  `Decimal` at the type level; storage adapters serialize to the
  appropriate SQL type.
- The `type` column on `facilities_dormitories` uses a `CHAR(1)` with
  values `B` (Boys) and `G` (Girls); the engine maps it to the
  `DormitoryType` enum on read and writes the corresponding
  short code on save.
- The `paid_status` column on `facilities_item_sells` carries a comment
  in the original migration: `P = paid, PP = partially paid, U =
  unpaid, R = ----`. The engine maps these to `Paid`, `Partial`,
  `Unpaid`, and `Refunded` respectively, with the `R` value
  reserved for refunds; historical `R` values map to `Refunded`.
- Foreign keys use `ON DELETE CASCADE`. The engine does not rely
  on database cascades for invariant enforcement; the
  application layer checks referential integrity before issuing
  the delete command.
