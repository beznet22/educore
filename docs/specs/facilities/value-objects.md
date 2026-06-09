# Facilities Domain — Value Objects

Value objects are immutable, validated at construction, and have no
identity. They are compared by value.

## Identifiers

All identifiers in the facilities domain are typed and
tenant-scoped. Two `VehicleId` values in different schools are
distinct types at the domain level and may be unified only through
explicit cross-tenant commands.

| Identifier            | Backing Type         | Source Column                  |
| --------------------- | -------------------- | ------------------------------ |
| `VehicleId`           | `Id<Vehicle>`        | `facilities_vehicles.id`               |
| `RouteId`             | `Id<Route>`          | `facilities_routes.id`                 |
| `AssignVehicleId`     | `Id<AssignVehicle>`  | `facilities_assign_vehicles.id`        |
| `DormitoryId`         | `Id<Dormitory>`      | `facilities_dormitories.id`        |
| `RoomId`              | `Id<Room>`           | `facilities_rooms.id`             |
| `RoomTypeId`          | `Id<RoomType>`       | `facilities_room_types.id`             |
| `ItemCategoryId`      | `Id<ItemCategory>`   | `facilities_item_categories.id`        |
| `ItemId`              | `Id<Item>`           | `facilities_items.id`                  |
| `ItemStoreId`         | `Id<ItemStore>`      | `facilities_item_stores.id`            |
| `ItemIssueId`         | `Id<ItemIssue>`      | `facilities_item_issues.id`            |
| `ItemReceiveId`      | `Id<ItemReceive>`    | `facilities_item_receives.id`          |
| `ItemReceiveChildId`  | `Id<...>`            | `facilities_item_receive_children.id`  |
| `ItemSellId`          | `Id<ItemSell>`       | `facilities_item_sells.id`             |
| `ItemSellChildId`     | `Id<...>`            | `facilities_item_sell_children.id`     |
| `SupplierId`          | `Id<Supplier>`       | `facilities_suppliers.id`              |
| `RouteStopId`         | `Id<RouteStop>`      | (derived)                      |
| `RoomAssignmentId`    | `Id<...>`            | (derived)                      |
| `TransportMembershipId` | `Id<...>`          | (derived)                      |
| `StoreStocktakeId`    | `Id<...>`            | (derived)                      |

Identifiers from other domains referenced by the facilities domain:

| Identifier         | Source Domain    |
| ------------------ | ---------------- |
| `SchoolId`         | `smsengine-platform` |
| `UserId`           | `smsengine-platform` |
| `StudentId`        | `smsengine-academic` |
| `StaffId`          | `smsengine-hr`        |
| `RoleId`           | `smsengine-rbac`     |
| `AcademicYearId`   | `smsengine-academic` |
| `ClassId`          | `smsengine-academic` |
| `SectionId`        | `smsengine-academic` |
| `TenantContext`    | `smsengine-platform` |

## Names & Numbers

| Type              | Constraints                                              |
| ----------------- | -------------------------------------------------------- |
| `VehicleNumber`   | 1..50 chars, unique within school, alphanumeric+dash     |
| `VehicleModel`    | 1..255 chars                                             |
| `RouteName`       | 1..200 chars, unique within school-year                  |
| `StopName`        | 1..200 chars                                             |
| `DormitoryName`   | 1..200 chars, unique within school-year                  |
| `RoomNumber`      | 1..50 chars, unique within dormitory                    |
| `RoomTypeName`    | 1..255 chars, unique within school                      |
| `CategoryName`    | 1..100 chars, unique within school                      |
| `ItemName`        | 1..100 chars                                             |
| `ItemSku`         | 1..50 chars, unique within school, alphanumeric          |
| `StoreName`       | 1..100 chars, unique within school                      |
| `StoreNumber`     | 1..100 chars                                             |
| `SupplierName`    | 1..100 chars, unique within school                      |
| `ContactPersonName` | 1..191 chars                                           |
| `ReferenceNumber` | 1..191 chars                                             |

## Quantities & Money

| Type              | Notes                                                |
| ----------------- | ---------------------------------------------------- |
| `ItemQuantity`    | `Decimal` > 0, precision matches receive/issue/sell  |
| `UnitPrice`       | `Decimal` >= 0                                       |
| `SellPrice`       | `Decimal` >= 0                                       |
| `SubTotal`        | Computed `Quantity * UnitPrice` (or `SellPrice`)     |
| `GrandTotal`      | `Decimal` sum of child subtotals                     |
| `TotalQuantity`   | `Decimal` sum of child quantities                    |
| `TotalPaid`       | `Decimal` >= 0, <= `GrandTotal`                      |
| `TotalDue`        | `Decimal` = `GrandTotal - TotalPaid`                 |
| `CostPerBed`      | `Decimal` >= 0                                       |
| `Fare`            | `Decimal` >= 0                                       |
| `Distance`        | `Decimal` >= 0 in kilometers                         |
| `Intake`          | `u32` > 0                                            |
| `NumberOfBed`     | `u32` > 0                                            |
| `BedNumber`       | `u32` in `1..NumberOfBed`                            |
| `StockOnHand`     | `Decimal` >= 0                                       |
| `MadeYear`        | `i32` in `1950..current_year`                        |

## Status Enums

| Type                | Values                                                       |
| ------------------- | ------------------------------------------------------------ |
| `DormitoryType`     | `Boys`, `Girls`                                              |
| `IssueStatus`       | `Issued`, `Returned`, `PartiallyReturned`, `Lost`            |
| `PaidStatus`        | `Paid`, `Partial`, `Unpaid`, `Refunded`                      |
| `PaymentMethod`     | `Cash`, `Bank`, `Cheque`, `Card`, `Wallet`                    |
| `SupplierStatus`    | `Active`, `Inactive`, `Blacklisted`                          |
| `VehicleStatus`     | `Active`, `Maintenance`, `Retired`                           |
| `ActiveStatus`      | `Active`, `Inactive`                                         |

## Identity

| Type              | Constraints                                              |
| ----------------- | -------------------------------------------------------- |
| `PhoneNumber`     | E.164 format preferred; alternative national formats     |
| `EmailAddress`    | RFC 5322 with length cap 200                             |
| `Address`         | 1..500 chars                                             |
| `Description`     | 0..500 chars                                             |
| `Note`            | 0..500 chars                                             |

## Specification Helpers

| Type                    | Notes                                                |
| ----------------------- | ---------------------------------------------------- |
| `TransportSpec`         | `(RouteId, AssignVehicleId, PickupStop, DropStop)`   |
| `HostelSpec`            | `(DormitoryId, RoomId?, BedNumber?)`                 |
| `IssueRecipient`        | enum `Staff(StaffId)`, `Student(StudentId)`, `Role(RoleId)` |
| `MoneySpec`             | `(UnitPrice, Quantity, SubTotal)`                    |

## Validation Rules

All value objects implement `Validate` and refuse construction
when validation fails:

```rust
pub trait Validate {
    fn validate(&self) -> Result<(), ValueError>;
}
```

Construction is the only entry point:

```rust
let vehicle_no = VehicleNumber::parse("GJ-05-AB-1234")?;
let qty = ItemQuantity::new(dec!(10.00))?;
```

Parsing returns `Result<T, ValueError>`. There are no setters that
bypass validation. The facilities domain never exposes raw strings
or numerics where a value object exists.

## Cross-Domain Helpers

| Type              | Notes                                                    |
| ----------------- | -------------------------------------------------------- |
| `SchoolId`        | From `smsengine-platform`                                  |
| `UserId`          | From `smsengine-platform`                                  |
| `TenantContext`   | From `smsengine-platform`                                  |
| `StudentId`       | From `smsengine-academic` (read-only reference)            |
| `StaffId`         | From `smsengine-hr` (read-only reference)                  |
| `AcademicYearId`  | From `smsengine-academic` (read-only reference)            |
| `RoleId`          | From `smsengine-rbac` (read-only reference)                |
