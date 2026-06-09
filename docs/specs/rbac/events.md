# RBAC Domain — Events

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

## Role Lifecycle

### RoleCreated

```rust
pub struct RoleCreated {
    pub role_id: RoleId,
    pub name: RoleName,
    pub role_type: RoleType,
    pub is_replicated: bool,
}
```

**Subscribers:**
- `platform` — invalidates the in-memory role cache.
- `audit` — records a "role created" entry.

### RoleUpdated

```rust
pub struct RoleUpdated {
    pub role_id: RoleId,
    pub changed_fields: Vec<&'static str>,
}
```

### RoleDeleted

```rust
pub struct RoleDeleted {
    pub role_id: RoleId,
    pub previous_name: RoleName,
}
```

**Subscribers:**
- `platform` — removes all user bindings to this role.

### RoleCloned

```rust
pub struct RoleCloned {
    pub source_role_id: RoleId,
    pub new_role_id: RoleId,
    pub new_name: RoleName,
    pub capabilities_copied: u32,
}
```

## Capability Lifecycle

### CapabilityRegistered

```rust
pub struct CapabilityRegistered {
    pub capability: Capability,
    pub lang_name: LangName,
}
```

This event is emitted at build-time by the engine's capability
seed, not by a user command.

### PermissionMetadataUpdated

```rust
pub struct PermissionMetadataUpdated {
    pub capability: Capability,
    pub changed_fields: Vec<&'static str>,
}
```

### CapabilityAssigned

```rust
pub struct CapabilityAssigned {
    pub role_id: RoleId,
    pub capability: Capability,
    pub menu_status: MenuStatus,
    pub saas_schools: Option<BTreeSet<SchoolId>>,
}
```

**Subscribers:**
- `platform` — invalidates the capability cache for this role.
- `audit` — records the grant.

### CapabilityRevoked

```rust
pub struct CapabilityRevoked {
    pub role_id: RoleId,
    pub capability: Capability,
    pub as_denial: bool,
}
```

`as_denial=true` means the row is preserved with `status=Revoked`.
`as_denial=false` means the row was hard-deleted.

### PermissionAssignmentUpdated

```rust
pub struct PermissionAssignmentUpdated {
    pub role_id: RoleId,
    pub capability: Capability,
    pub changed_fields: Vec<&'static str>,
}
```

## Module Permission Lifecycle

### ModulePermissionCreated

```rust
pub struct ModulePermissionCreated {
    pub module_permission_id: ModulePermissionId,
    pub name: ModuleName,
    pub dashboard_id: DashboardId,
}
```

### ModulePermissionUpdated / ModulePermissionDeleted

```rust
pub struct ModulePermissionUpdated {
    pub module_permission_id: ModulePermissionId,
    pub changed_fields: Vec<&'static str>,
}

pub struct ModulePermissionDeleted {
    pub module_permission_id: ModulePermissionId,
}
```

### ModulePermissionAssigned / ModulePermissionRevoked

```rust
pub struct ModulePermissionAssigned {
    pub module_permission_id: ModulePermissionId,
    pub role_id: RoleId,
}

pub struct ModulePermissionRevoked {
    pub module_permission_id: ModulePermissionId,
    pub role_id: RoleId,
}
```

## Menu Link Lifecycle

### MenuLinkGranted

```rust
pub struct MenuLinkGranted {
    pub module_link_id: ModuleLinkId,
    pub role_id: RoleId,
}
```

**Subscribers:**
- `platform` — updates the per-role sidebar projection.

### MenuLinkRevoked

```rust
pub struct MenuLinkRevoked {
    pub module_link_id: ModuleLinkId,
    pub role_id: RoleId,
}
```

## Permission Section Lifecycle

- `PermissionSectionCreated { id, name, position }`
- `PermissionSectionUpdated { id, changed_fields }`
- `PermissionSectionDeleted { id, prior_name }`

## Two-Factor Lifecycle

### TwoFactorConfigured

```rust
pub struct TwoFactorConfigured {
    pub school_id: SchoolId,
    pub via_sms: bool,
    pub via_email: bool,
    pub for_student: TwoFactorMode,
    pub for_parent: TwoFactorMode,
    pub for_teacher: TwoFactorMode,
    pub for_staff: TwoFactorMode,
    pub for_admin: TwoFactorMode,
    pub expired_time: TwoFactorExpiry,
}
```

**Subscribers:**
- `platform` — refreshes its authentication flow to honor the new
  policy.
- `audit` — records the configuration change.

### TwoFactorDeliveryTested

```rust
pub struct TwoFactorDeliveryTested {
    pub school_id: SchoolId,
    pub channel: TwoFactorChannel,
    pub recipient_user_id: UserId,
    pub outcome: DeliveryOutcome,
}
```

`DeliveryOutcome` is `Delivered`, `Failed`, or `Throttled`.

## Permission Override Lifecycle

### PermissionOverrideSet

```rust
pub struct PermissionOverrideSet {
    pub override_id: PermissionOverrideId,
    pub actor_id: UserId,
    pub capability: Capability,
    pub granted: bool,
    pub reason: OverrideReason,
    pub expires_at: Option<Timestamp>,
}
```

### PermissionOverrideCleared

```rust
pub struct PermissionOverrideCleared {
    pub override_id: PermissionOverrideId,
    pub actor_id: UserId,
    pub capability: Capability,
}
```
