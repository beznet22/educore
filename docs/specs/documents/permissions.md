# Documents Domain — Permissions

Permissions are capability strings. They are not roles. The RBAC domain
maps capabilities to roles.

## Naming

```text
<Domain>.<Aggregate>.<Action>
```

## Capabilities

### Document (Cross-Cutting)

- `Document.Read`

### Form

- `Form.Upload`
- `Form.Update`
- `Form.Delete`
- `Form.Read`
- `Form.Read.Public` (granted to anonymous visitors on the public site)

### Postal

- `Postal.Dispatch`
- `Postal.Receive`
- `Postal.Update`
- `Postal.Delete`
- `Postal.Read`

## Default Role Mapping

The platform's default role catalog binds the following:

| Role        | Capabilities (highlights)                                            |
| ----------- | -------------------------------------------------------------------- |
| SuperAdmin  | All                                                                 |
| SchoolAdmin | All within the school                                               |
| Teacher     | Form.Read, Form.Upload, Postal.Read                                 |
| Student     | Form.Read (when published)                                          |
| Parent      | Form.Read (when published)                                          |
| Reception   | Form.*, Postal.*                                                    |
| Public      | Form.Read.Public (granted to anonymous visitors)                    |

The default mapping is a starting point and is configurable per school.

## Authorization Pattern

Capabilities are checked at the command boundary. The engine never
trusts the caller to assert their own role.

```rust
if !engine.rbac().has(actor_id, Capability::PostalDispatch).await? {
    return Err(DomainError::forbidden("missing capability"));
}
```

A secondary check is performed for `Form.Read.Public`: the form must
have `show_public = true` and the actor must hold `Form.Read.Public`
or `Form.Read`.

## Read vs Write

Read capabilities are explicit. The engine does not assume that
`Form.Read` implies `Form.Upload`. A consumer may grant only
read-only access to a parent or auditor.

## Tenant Isolation

Every capability check is paired with a tenant check. The actor must
be authenticated to the school that owns the target aggregate. There
is no cross-tenant capability elevation.
