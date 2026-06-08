# Facilities Domain — Permissions

Permissions are capability strings. They are not roles. The RBAC
domain maps capabilities to roles.

## Naming

```text
<Domain>.<Aggregate>.<Action>
```

The facilities domain uses the prefix `Vehicle.*`, `Route.*`,
`Transport.*`, `Dormitory.*`, `Room.*`, `RoomType.*`,
`ItemCategory.*`, `Item.*`, `ItemStore.*`, `Inventory.*`, and
`Supplier.*`.

## Capabilities

### Vehicle

- `Vehicle.Create`
- `Vehicle.Update`
- `Vehicle.Delete`
- `Vehicle.Read`
- `Vehicle.AssignDriver`
- `Vehicle.Deactivate`

### Route

- `Route.Create`
- `Route.Update`
- `Route.Delete`
- `Route.Read`
- `Route.AddStop`
- `Route.UpdateStop`
- `Route.RemoveStop`

### Transport

- `Transport.AssignVehicle`
- `Transport.UnassignVehicle`
- `Transport.AssignStudent`
- `Transport.UnassignStudent`
- `Transport.Read`

### Dormitory

- `Dormitory.Create`
- `Dormitory.Update`
- `Dormitory.Delete`
- `Dormitory.Read`

### Room

- `Room.Create`
- `Room.Update`
- `Room.Delete`
- `Room.Read`
- `Room.AssignStudent`
- `Room.UnassignStudent`

### RoomType

- `RoomType.Create`
- `RoomType.Update`
- `RoomType.Delete`
- `RoomType.Read`

### ItemCategory

- `ItemCategory.Create`
- `ItemCategory.Update`
- `ItemCategory.Delete`
- `ItemCategory.Read`

### Item

- `Item.Create`
- `Item.Update`
- `Item.Delete`
- `Item.Read`

### ItemStore

- `ItemStore.Create`
- `ItemStore.Update`
- `ItemStore.Delete`
- `ItemStore.Read`

### Inventory

- `Inventory.Receive`
- `Inventory.UpdateReceive`
- `Inventory.CancelReceive`
- `Inventory.Issue`
- `Inventory.UpdateIssue`
- `Inventory.ReturnIssued`
- `Inventory.Sell`
- `Inventory.UpdateSell`
- `Inventory.CancelSell`
- `Inventory.RefundSell`
- `Inventory.Read`

### Supplier

- `Supplier.Create`
- `Supplier.Update`
- `Supplier.Delete`
- `Supplier.Deactivate`
- `Supplier.Read`

## Default Role Mapping

The platform's default role catalog binds the following for the
facilities domain:

| Role            | Capabilities (highlights)                                                |
| --------------- | ------------------------------------------------------------------------- |
| SuperAdmin      | All                                                                       |
| SchoolAdmin     | All within the school                                                     |
| Transport       | `Transport.*`, `Route.Read`, `Vehicle.Read`, `Vehicle.AssignDriver`      |
| HostelWarden    | `Dormitory.*`, `Room.*`, `RoomType.*`, `Transport.Read`                  |
| InventoryClerk  | `Item.*`, `ItemCategory.*`, `ItemStore.*`, `Inventory.*`, `Supplier.Read` |
| Procurement     | `Supplier.*`, `Inventory.Receive`, `Inventory.Read`, `Item.Read`          |
| Accountant      | `Inventory.Read`, `Supplier.Read`                                         |
| Teacher         | `Inventory.Read`, `Room.Read`, `Transport.Read`                           |
| Student         | `Inventory.Read` (catalog only)                                           |
| Parent          | `Room.Read` (linked students), `Transport.Read` (linked students)         |

The default mapping is a starting point and is configurable per
school.

## Authorization Pattern

Capabilities are checked at the command boundary. The engine never
trusts the caller to assert their own role.

```rust
if !engine.rbac().has(actor_id, Capability::InventoryReceive).await? {
    return Err(DomainError::forbidden("missing capability"));
}
```

Some commands have a secondary ownership check: only the warden of
the dormitory may allocate a room; only the procurement officer
for the school may deactivate a supplier; only the assigned
transport desk officer may attach a student to a vehicle-route
pair.

## Read vs Write

Read capabilities are explicit. The engine does not assume that
`Vehicle.Read` implies `Vehicle.Create`. A consumer may grant
read-only access to a parent or auditor.

## Tenant Isolation

Every capability check is paired with a tenant check. The actor
must be authenticated to the school that owns the target
aggregate. There is no cross-tenant capability elevation.
Cross-school logistics (e.g. inter-school transport) are
special-cased and require a per-tenant grant from both schools.

## Audit Requirements

Every command in the facilities domain writes a durable audit
record referencing the originating `actor_id`, `correlation_id`,
and the event(s) it produced. The audit record retains:

- The originating capability used to authorize the command.
- The pre-image and post-image of the affected aggregate (or a
  diff if the aggregate is large).
- The full event payload(s) for cross-domain replay.

Read commands (`*.Read`) are not audited at the audit sink, but
queries that surface personal data (e.g. listing students
assigned to a room) must be logged at `INFO` with the
`actor_id` and `correlation_id` for compliance.
