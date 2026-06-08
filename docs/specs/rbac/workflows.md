# RBAC Domain — Workflows

Workflows orchestrate commands, queries, and policies to fulfill a
business goal. They are documented as ordered, conditional steps.

## School Bootstrap Workflow

```text
1. Engine creates the School row (from the platform domain).
2. Engine creates the bootstrap SuperAdmin role.
3. Engine seeds the SuperAdmin role with every registered
   Capability (Rbac.* and all domain capabilities).
4. Engine creates the first User (school admin) and binds them
   to the SuperAdmin role.
5. Engine creates the default PermissionSection list.
6. Engine creates a default TwoFactorSetting (via_email=true,
   via_sms=false, all roles set to Optional, expired_time=300).
7. Engine seeds baseline ModulePermissions and assigns them to
   the SuperAdmin role.
```

**Pre-conditions:**

- A school row exists in the platform domain.

**Failure paths:**

- Failure to seed capabilities logs a critical error and aborts
  school activation.
- A partial bootstrap is rolled back; the school is not activated.

## Role Setup Workflow

```text
1. SchoolAdmin (SuperAdmin) creates a custom role
   (CreateRoleCommand).
2. SchoolAdmin assigns capabilities (AssignCapabilityCommand)
   one by one or in bulk.
3. SchoolAdmin grants menu links (GrantMenuLinkCommand) for
   each UI section the role should see.
4. SchoolAdmin assigns dashboard cards (AssignModulePermission).
5. SchoolAdmin reviews the role's effective capability set
   (read-only query).
6. SchoolAdmin binds users to the role (in the platform domain).
```

**Edge cases:**

- A system role cannot be deleted; only renamed.
- Deleting a role with active user bindings is rejected by the
  platform domain with `ConflictError::RoleInUse`.

## Capability Grant / Revoke Workflow

```text
Grant:
1. SchoolAdmin selects a role and a capability.
2. SchoolAdmin issues AssignCapabilityCommand.
3. The system writes AssignPermission with status=Granted.
4. The system invalidates the in-memory capability cache.
5. Affected users gain the capability on their next request.

Revoke:
1. SchoolAdmin selects a role and a capability.
2. SchoolAdmin issues RevokeCapabilityCommand.
3. The system either sets status=Revoked (denial) or
   hard-deletes the row.
4. The system invalidates the in-memory capability cache.
5. Affected users lose the capability on their next request.
```

**Edge cases:**

- A revocation that would leave the actor without the capability
  required to re-grant is rejected (self-revocation guard).
- A `menu_status=Hidden` grant still allows execution; it only
  hides the menu item.

## Two-Factor Enrollment Workflow

```text
1. SchoolAdmin updates the school's TwoFactorSetting
   (ConfigureTwoFactorCommand).
2. The system writes the new policy and emits TwoFactorConfigured.
3. On the next login, the affected roles are prompted for 2FA.
4. The user receives an OTP via the configured channel(s).
5. The user enters the OTP; the platform domain verifies it
   against the OtpCodeRow.
6. The session is granted.
7. A TwoFactorDeliveryTested event is emitted on every
   successful delivery.
```

**Edge cases:**

- An expired OTP is rejected with `ConflictError::OtpExpired`.
- An OTP reused after consumption is rejected with
  `ConflictError::OtpAlreadyUsed`.
- If neither `via_sms` nor `via_email` is enabled, login is
  blocked for roles set to `Required`.

## Audit Workflow

```text
1. Every RBAC command emits a domain event.
2. The event bus delivers the event to all subscribers.
3. The audit sink writes a row referencing the event id, the
   actor, the school, the target aggregate, and the change.
4. A nightly job snapshots the role/capability state to the
   RoleMembershipSnapshot entity.
5. Auditors query the snapshot history to reconstruct
   "who had what capability when".
```

## Role Cloning Workflow

```text
1. SchoolAdmin selects a source role.
2. SchoolAdmin provides a new name.
3. SchoolAdmin issues CloneRoleCommand.
4. The system creates a new Role (type=Custom).
5. The system copies all AssignPermission rows to the new role.
6. The system copies all RolePermission (menu) rows.
7. The system copies all ModulePermissionAssign rows.
8. The system emits RoleCloned.
```

## Override Workflow

```text
1. Security officer identifies a need to grant a single user a
   capability not held by their role (e.g. emergency access).
2. Officer issues SetPermissionOverrideCommand with granted=true
   and a reason.
3. The system writes PermissionOverride and emits
   PermissionOverrideSet.
4. The user gains the capability immediately.
5. The override is auto-cleared at expires_at (if set).
6. Officer can ClearPermissionOverrideCommand to remove it
   early.
```

## Menu Visibility Workflow

```text
1. SchoolAdmin edits a role.
2. SchoolAdmin toggles menu links (GrantMenuLinkCommand or
   RevokeMenuLinkCommand).
3. The system writes RolePermission rows.
4. The platform domain's SidebarEntry projection is updated
   by a subscriber.
5. Affected users see the new menu on next page load.
```

## Bootstrap Lock Recovery

```text
1. All Rbac.* capabilities have been revoked from every user.
2. The SchoolAdmin contacts the engine operator.
3. Operator uses a one-time master key (out-of-band) to issue
   ConfigureTwoFactor with an out-of-band `via_email` set, then
   uses the master key to re-grant Rbac.Capability.Assign to
   SuperAdmin.
4. The system rejects the operation if the master key is
   invalid.
5. The recovery event is logged at CRITICAL severity.
```

The master key is a port concern; the engine does not store it.

## Idempotency

- `CreateRole` is idempotent on `(school_id, name)`. A duplicate
  returns the existing role.
- `AssignCapability` is idempotent on `(role_id, capability)`. A
  duplicate is a no-op success.
- `ConfigureTwoFactor` is idempotent on `school_id` — there is
  exactly one row.
- `DeleteRole` is not idempotent; a second call returns
  `NotFoundError`.
