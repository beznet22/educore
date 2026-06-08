# Events Domain — Permissions

Permissions are capability strings. They are not roles. The RBAC domain
maps capabilities to roles.

## Naming

```text
<Domain>.<Aggregate>.<Action>
```

## Capabilities

### Event

- `Event.Create`
- `Event.Update`
- `Event.Delete`
- `Event.Read`
- `Event.Publish` (admin override for cross-role broadcast)

### Holiday

- `Holiday.Create`
- `Holiday.Update`
- `Holiday.Delete`
- `Holiday.Read`

### Weekend

- `Weekend.Create`
- `Weekend.Update`
- `Weekend.Configure`
- `Weekend.Delete`
- `Weekend.Read`

### Incident

- `Incident.Create`
- `Incident.Update`
- `Incident.Assign`
- `Incident.Reassign`
- `Incident.Unassign`
- `Incident.Comment`
- `Incident.Resolve`
- `Incident.Delete`
- `Incident.Read`

### Incident Comment

- `IncidentComment.Delete`

### Calendar Setting

- `CalendarSetting.Create`
- `CalendarSetting.Update`
- `CalendarSetting.Enable`
- `CalendarSetting.Disable`
- `CalendarSetting.Delete`
- `CalendarSetting.Read`

## Default Role Mapping

The platform's default role catalog binds the following:

| Role        | Capabilities (highlights)                                            |
| ----------- | -------------------------------------------------------------------- |
| SuperAdmin  | All                                                                 |
| SchoolAdmin | All within the school                                               |
| Teacher     | Event.Create, Event.Update, Event.Read, Holiday.Read, Incident.Create, Incident.Comment, Incident.Assign, CalendarSetting.Read |
| Student     | Event.Read                                                          |
| Parent      | Event.Read                                                          |
| Discipline  | Incident.*, Weekend.Read, CalendarSetting.Read                       |
| Reception   | Event.Create, Event.Read, Holiday.Read, CalendarSetting.Read         |

The default mapping is a starting point and is configurable per school.

## Authorization Pattern

Capabilities are checked at the command boundary. The engine never
trusts the caller to assert their own role.

```rust
if !engine.rbac().has(actor_id, Capability::IncidentResolve).await? {
    return Err(DomainError::forbidden("missing capability"));
}
```

Some commands have a secondary ownership check: a teacher creating an
incident in their own class-section is allowed; assigning a student
to an incident requires `Incident.Assign`.

## Read vs Write

Read capabilities are explicit. The engine does not assume that
`Event.Read` implies `Event.Create`. A consumer may grant only
read-only access to a parent or auditor.

## Tenant Isolation

Every capability check is paired with a tenant check. The actor must
be authenticated to the school that owns the target aggregate. There
is no cross-tenant capability elevation.
