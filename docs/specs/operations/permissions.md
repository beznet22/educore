# Operations Domain — Permissions

Permissions are capability strings. They are not roles. The RBAC
domain maps capabilities to roles.

## Naming

```text
<Domain>.<Aggregate>.<Action>
```

For the operations domain, the convention is `Operations.*`.

## Capabilities

### Backup

- `Operations.Backup.Create`
- `Operations.Backup.Read`
- `Operations.Backup.Delete`
- `Operations.Backup.Restore`
- `Operations.Backup.Activate`
- `Operations.Backup.Deactivate`

### Job

- `Operations.Job.Schedule` (system)
- `Operations.Job.Read`
- `Operations.Job.Cancel` (system)
- `Operations.Job.Reserve` (system)
- `Operations.Job.Complete` (system)
- `Operations.Job.Fail` (system)
- `Operations.Job.Retry` (system)
- `Operations.Job.Purge`

### FailedJob

- `Operations.FailedJob.Read`
- `Operations.FailedJob.Retry` (system)
- `Operations.FailedJob.Delete`
- `Operations.FailedJob.Purge`

### SystemVersion

- `Operations.Version.Register` (system, build-time)
- `Operations.Version.Read`
- `Operations.Version.Update` (system)

### VersionHistory

- `Operations.VersionHistory.Record` (system, build-time)
- `Operations.VersionHistory.Read`

### UserLog

- `Operations.Audit.Record` (system)
- `Operations.Audit.Read`

### Maintenance

- `Operations.Maintenance.Read`
- `Operations.Maintenance.Configure`
- `Operations.Maintenance.Enable`
- `Operations.Maintenance.Disable`

### Sidebar

- `Operations.Sidebar.Create`
- `Operations.Sidebar.Read`
- `Operations.Sidebar.Update`
- `Operations.Sidebar.Delete`
- `Operations.Sidebar.Reorder`

## Default Role Mapping

| Role             | Capabilities (highlights)                                          |
| ---------------- | ------------------------------------------------------------------ |
| SuperAdmin       | All                                                                |
| SchoolAdmin      | `Operations.Backup.*`, `Operations.Maintenance.*`, `Operations.Sidebar.*`, `Operations.Audit.Read` |
| Teacher          | `Operations.Audit.Read (self)`                                    |
| Student          | `Operations.Audit.Read (self)`                                    |

The default mapping is configurable per school. System capabilities
(`Operations.Job.*`, `Operations.Version.*`,
`Operations.VersionHistory.*`, `Operations.Audit.Record`) are held
only by the engine's system tenant and are not assignable to
school users.

## Authorization Pattern

Capabilities are checked at the command boundary:

```rust
if !engine.rbac().has(actor_id, Capability::OperationsBackupRestore).await? {
    return Err(DomainError::forbidden("missing capability"));
}
```

The system tenant (used for `ScheduleJob`, `MarkJobReserved`,
`MarkJobCompleted`, `MarkJobFailed`, `RegisterSystemVersion`,
`RecordVersionHistory`, `RecordUserLog`) bypasses the capability
check; these commands are issued by trusted port adapters.

## Read vs Write

Read capabilities are explicit. The engine does not assume
"Operations.Audit.Read" implies "Operations.Maintenance.Enable".

## Tenant Isolation

Every capability check is paired with a tenant check. The actor
must be authenticated to the school that owns the target
aggregate. Job, system-version, and OAuth commands are
platform-internal and use a system tenant.

## Maintenance Lockout

When `MaintenanceSetting::maintenance_mode=true`, the platform
domain's authentication flow rejects logins from actors who do
not hold `Operations.Maintenance.Disable` (typically
`SuperAdmin`). The check is enforced at the auth port, not by
the operations domain.

## Self-Authorization Guard

The engine refuses to disable maintenance for the last remaining
`SuperAdmin` in a school. A `DisableMaintenance` command from a
non-SuperAdmin while maintenance is enabled is rejected with
`ForbiddenError::MaintenanceLockout`.
