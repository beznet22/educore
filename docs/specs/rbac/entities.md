# RBAC Domain — Entities

Entities have identity and lifecycle but are not aggregate roots. They
are loaded and persisted only through their aggregate root.

## RoleBinding

**Identity:** `RoleBindingId(SchoolId, Uuid)`
**Owner:** `User` (in the platform domain) — referenced by id

A platform-level join that says "this user holds this role in this
school". The RBAC domain stores the catalog; the platform domain
stores who is bound. The two are kept consistent by an event
subscriber that listens to `RoleDeleted` and removes all bindings.

## PermissionOverride

**Identity:** `PermissionOverrideId(SchoolId, Uuid)`
**Owner:** `AssignPermission`

A per-actor override of a permission grant. A user with
`PermissionOverride::granted=true` for a `(permission, actor)` pair
gains the capability even if no role grants it; a user with
`granted=false` loses it. The override takes precedence over role
grants. Used for emergency escalation and audit-only access.

## ModuleLinkBinding

**Identity:** `ModuleLinkBindingId(SchoolId, Uuid)`
**Owner:** `ModuleLink` (in the platform domain)

A platform-side projection of `RolePermission` rows, materialized
for fast menu rendering. The platform domain subscribes to
`MenuLinkGranted` / `MenuLinkRevoked` to keep this in sync.

## CapabilityGrantEventProjection

**Identity:** `CapabilityGrantEventProjectionId(SchoolId, CapabilityId)`
**Owner:** `Capability` (read-model)

A read-model row produced by the event subscriber that listens to
`CapabilityAssigned` and `CapabilityRevoked`. Used by
`CapabilityCheckService` to answer authorization questions without
loading the full `AssignPermission` history.

## RoleHierarchyEdge

**Identity:** `RoleHierarchyEdgeId(SchoolId, Uuid)`
**Owner:** `Role`

A potential future edge in a role inheritance DAG. Reserved for
later expansion. Not yet supported in commands; the data model
already has space for it.

## TwoFactorDelivery

**Identity:** `TwoFactorDeliveryId(SchoolId, Uuid)`
**Owner:** `TwoFactorSetting`

A single delivery attempt of a 2FA OTP. Stores the channel used
(`Sms` or `Email`), the recipient user, the OTP (hashed), the
issue timestamp, the expiry timestamp, and a `consumed` flag.
This is the per-delivery audit trail; the `TwoFactorSetting`
aggregate holds the policy, not the deliveries.

## OtpCodeRow

**Identity:** `OtpCodeId(SchoolId, Uuid)`
**Owner:** `User` (in the platform domain)

An OTP issued to a user. The RBAC domain does not own the OTP
generation but does define its expiry semantics when used as a
second factor. The cross-domain projection is `OtpCodeRow` here
for type clarity.

## DashboardSection

**Identity:** `DashboardSectionId(SchoolId, Uuid)`
**Owner:** `ModulePermission` (logical)

A UI grouping of dashboard cards. The RBAC domain tracks
`ModulePermission::dashboard_id`; the platform domain owns the
`DashboardSection` rendering layout.

## PermissionTranslation

**Identity:** `PermissionTranslationId(SchoolId, Uuid)`
**Owner:** `Permission`

A localized label for a permission. The base `Permission` row
stores `lang_name` (a key); `PermissionTranslation` maps a key to
its translated text per locale. The settings domain owns the
phrase catalog; the RBAC domain reads from it.

## RoleMembershipSnapshot

**Identity:** `RoleMembershipSnapshotId(SchoolId, Uuid)`
**Owner:** `School`

A point-in-time snapshot of all role memberships in a school. Used
for compliance audits ("who had the `Finance.Refund` capability on
March 15?"). Produced nightly by a background job.

## CapabilityCatalog

**Identity:** `CapabilityCatalogId(SchoolId, Uuid)`
**Owner:** `School`

A versioned, signed list of every `Capability` registered with a
school at a point in time. Used by the engine to detect drift
between the compiled code and the seeded `Permission` rows. A
fresh school seeds its catalog from the engine's compiled-in
capability set.

## SidebarEntry

**Identity:** `SidebarEntryId(SchoolId, Uuid)`
**Owner:** `Role`

A UI sidebar item rendered for a role. The RBAC domain stores the
binding (which menu items a role sees); the platform domain owns
the visual layout. The data is duplicated in the
`SidebarEntry` projection for fast menu rendering.

## SidebarPosition

**Identity:** `SidebarPositionId(SchoolId, Uuid)`
**Owner:** `SidebarEntry`

The ordered position of a sidebar item within its parent. Stored
as a flat int for sort stability.

## TwoFactorAuditEntry

**Identity:** `TwoFactorAuditEntryId(SchoolId, Uuid)`
**Owner:** `TwoFactorSetting`

A per-event audit row recording a 2FA configuration change. The
event log is the source of truth; this entity is a denormalized
read-model for the security audit screen.

## CapabilitySearchIndex

**Identity:** `CapabilitySearchIndexId(SchoolId, Uuid)`
**Owner:** `Capability`

A denormalized search index over capability names, route strings,
and translation keys. Updated on `CapabilityRegistered` and
`PermissionTranslation` changes. The settings domain owns the
search index port; the RBAC domain publishes the source events.
