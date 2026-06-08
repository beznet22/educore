# RBAC Domain — Permissions

The RBAC domain is the **catalog owner** of capabilities and the
**administrator** of every role/permission binding. The catalog
itself is closed and code-generated; the engine refuses to load an
unknown `Capability` string at startup.

## Naming

```text
<Domain>.<Aggregate>.<Action>
```

For the RBAC domain itself, the convention is `Rbac.*`. All other
domain capabilities follow `<Domain>.<Aggregate>.<Action>`.

## Capabilities

### Rbac.Role

- `Rbac.Role.Create`
- `Rbac.Role.Read`
- `Rbac.Role.Update`
- `Rbac.Role.Delete`
- `Rbac.Role.Manage` — required to modify system roles.
- `Rbac.Role.Clone`
- `Rbac.Role.GrantMenu`
- `Rbac.Role.RevokeMenu`

### Rbac.Capability

- `Rbac.Capability.Read`
- `Rbac.Capability.Assign`
- `Rbac.Capability.Revoke`
- `Rbac.Capability.UpdateMetadata`

### Rbac.Section

- `Rbac.Section.Create`
- `Rbac.Section.Read`
- `Rbac.Section.Update`
- `Rbac.Section.Delete`

### Rbac.ModulePermission

- `Rbac.ModulePermission.Create`
- `Rbac.ModulePermission.Read`
- `Rbac.ModulePermission.Update`
- `Rbac.ModulePermission.Delete`
- `Rbac.ModulePermission.Assign`
- `Rbac.ModulePermission.Revoke`

### Rbac.TwoFactor

- `Rbac.TwoFactor.Configure`
- `Rbac.TwoFactor.Read`
- `Rbac.TwoFactor.Test`

### Rbac.Override

- `Rbac.Override.Set`
- `Rbac.Override.Clear`
- `Rbac.Override.Read`

## Default Role Mapping

The engine seeds every new school with a baseline role catalog. The
mapping below is the **suggested default**; the platform may override
per school. The RBAC domain does not auto-assign capabilities when a
new capability is added; assignments are explicit.

| Role             | Capabilities (highlights)                                                    |
| ---------------- | ---------------------------------------------------------------------------- |
| SuperAdmin       | All `Rbac.*`, all domain capabilities                                        |
| SchoolAdmin      | All `Rbac.Role.*`, all `Rbac.Capability.*`, all domain capabilities          |
| Teacher          | `Rbac.Capability.Read`, `Rbac.Role.Read`                                     |
| Student          | `Rbac.Capability.Read`                                                       |
| Parent           | `Rbac.Capability.Read`                                                       |
| Accountant       | `Rbac.Capability.Read`                                                       |
| Librarian        | `Rbac.Capability.Read`                                                       |
| TransportManager | `Rbac.Capability.Read`                                                       |

The `SuperAdmin` role is a system role and cannot be deleted. It
holds every registered `Capability` at the time of school creation
and is refreshed on engine startup to pick up newly registered
capabilities.

## Authorization Pattern

Capabilities are checked at the command boundary. The engine never
trusts the caller to assert their own role. The check is implemented
in the `CapabilityCheckService`:

```rust
if !engine.rbac().has(actor_id, Capability::RbacRoleCreate).await? {
    return Err(DomainError::forbidden("missing capability"));
}
```

A grant is evaluated as:

```text
effective = role_grants OR permission_override.granted
denied    = permission_override.granted == false
result    = (effective AND NOT denied) OR system_role
```

The `system_role` clause is a backstop: the `SuperAdmin` role is
always considered to hold every `Rbac.*` capability, regardless of
explicit assignment.

## Self-Authorization

The RBAC domain is the only domain whose commands require elevated
`Rbac.*` capabilities, which are administered by the RBAC domain
itself. To break the bootstrap deadlock, the engine ships with a
**system capability** `Rbac.Bootstrap` that is held by the
`SuperAdmin` role and is **never** revocable. The bootstrap token is
required to seed the first user (the school admin) and the first
`Role` row.

After bootstrap, the system admin can grant or revoke any other
capability, but cannot remove the bootstrap capability from
`SuperAdmin`. The only way to remove it is to delete the entire
`SuperAdmin` role, which is itself blocked because `SuperAdmin` is
a system role and cannot be deleted.

## Read vs Write

Read capabilities are explicit. The engine does not assume that
"Rbac.Capability.Read" implies "Rbac.Capability.Assign". A consumer
may grant only read-only access to a security auditor.

## Tenant Isolation

Every capability check is paired with a tenant check. The actor must
be authenticated to the school that owns the target aggregate. RBAC
commands may not be executed against another school's role catalog.

In SaaS mode, a `saas_schools` list on the `AssignPermission` row
restricts the grant to the listed schools. The
`CapabilityCheckService` evaluates the actor's current school
against the grant's scope; if the school is not in the list, the
grant is not effective.

## Self-Revocation Guard

The engine refuses any command whose effect would leave the actor
without the capability required to undo the command. Concretely:

- A `Rbac.Role.Delete` on the last `Rbac.Role.Create`-holding role
  is rejected.
- A `Rbac.Capability.Revoke` on the last `Rbac.Capability.Assign`
  grant is rejected.

This invariant is enforced by the `CapabilityCheckService` before
the command is dispatched.

## Capability Stability

Once a `Capability` variant is registered and seeded into a school's
`Permission` table, the engine guarantees that the string form
remains stable for the lifetime of the major version. Renaming a
variant requires a migration that updates the stored strings
atomically.
