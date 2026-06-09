# RBAC Domain — Value Objects

Value objects are immutable, validated at construction, and have no
identity. They are compared by value.

## Identifiers

All identifiers in the RBAC domain are typed and tenant-scoped.
The generic `Id<S, T>` wrapper carries the `SchoolId` of the owning
school and the local id (`Uuid`).

| Identifier                   | Backing Type                | Notes                              |
| ---------------------------- | --------------------------- | ---------------------------------- |
| `RoleId`                     | `Id<Role>`                  | A role within a school             |
| `CapabilityId`               | `Id<Capability>`            | A permission row (a capability)    |
| `PermissionSectionId`        | `Id<PermissionSection>`     | A UI grouping label                |
| `AssignPermissionId`         | `Id<AssignPermission>`      | A capability-to-role grant         |
| `ModulePermissionId`         | `Id<ModulePermission>`      | A dashboard-level permission group |
| `ModulePermissionAssignId`   | `Id<ModulePermissionAssign>`| A module-permission-to-role grant  |
| `RolePermissionId`           | `Id<RolePermission>`        | A module-link-to-role grant        |
| `TwoFactorSettingId`         | `Id<TwoFactorSetting>`      | The school's 2FA policy row        |
| `RoleBindingId`              | `Id<RoleBinding>`           | A user-to-role binding             |
| `PermissionOverrideId`       | `Id<PermissionOverride>`    | A per-actor capability override    |
| `TwoFactorDeliveryId`        | `Id<TwoFactorDelivery>`     | A single 2FA OTP delivery attempt  |

## Role

| Type            | Constraints                                                          |
| --------------- | -------------------------------------------------------------------- |
| `RoleName`      | 1..100 chars, unique within `(school_id, normalized_lower)`          |
| `RoleType`      | `System` or `Custom`                                                 |
| `RoleStatus`    | `Active` or `Inactive`                                               |
| `RoleNamePatch` | Partial update: `name?`, `type?` (immutable `name` for system roles) |

## Capability

| Type          | Constraints                                                       |
| ------------- | ----------------------------------------------------------------- |
| `Capability`  | Closed enum; see `docs/schemas/capability-enum.md` for full list  |
| `CapabilityString` | Newtype around `String`; parses to `Capability` or errors    |
| `CapabilityDomain` | `'Academic' | 'Assessment' | 'Attendance' | 'Finance' | 'Hr' | 'Library' | 'Facilities' | 'Communication' | 'Documents' | 'Cms' | 'Platform' | 'Rbac' | 'Settings' | 'Operations' | 'Events'` |
| `CapabilityAction` | Verb in the present tense: `'Create' | 'Read' | 'Update' | 'Delete' | 'Admit' | 'Promote' | '...' `  |
| `CapabilityScope` | `'Tenant' | 'System'`; system-scoped capabilities are not assigned to roles |

The `Capability` enum derives `Display`, `FromStr`, `Serialize`,
`Deserialize`. The `FromStr` impl is total — unknown strings yield a
`ValueError::UnknownCapability`.

```rust
pub enum Capability {
    StudentAdmit,
    StudentUpdate,
    StudentRead,
    StudentSuspend,
    StudentReinstate,
    StudentWithdraw,
    StudentTransfer,
    StudentPromote,
    StudentGraduate,
    StudentAssignSection,
    // ... one variant per documented capability across all domains
    RbacRoleCreate,
    RbacRoleUpdate,
    RbacRoleDelete,
    RbacCapabilityAssign,
    RbacCapabilityRevoke,
    RbacTwoFactorConfigure,
    RbacModulePermissionCreate,
    RbacModulePermissionAssign,
    PlatformSchoolCreate,
    PlatformUserRegister,
    PlatformUserUpdate,
    PlatformUserDeactivate,
    PlatformOtpIssue,
    PlatformOtpVerify,
    PlatformModuleEnable,
    PlatformAddonInstall,
    SettingsGeneralUpdate,
    SettingsLanguageAdd,
    SettingsThemeConfigure,
    OperationsBackupCreate,
    OperationsBackupRestore,
    OperationsJobSchedule,
    OperationsVersionBump,
    OperationsAuditRead,
}
```

The `Capability` enum is the single source of truth for the catalog.
`Permission` rows in storage carry the same string form; the engine
verifies at startup that every `Permission` row maps to a known
variant.

## Permission Metadata

| Type              | Constraints                                                       |
| ----------------- | ----------------------------------------------------------------- |
| `PermissionName`  | 1..191 chars, the human-readable name                            |
| `Route`           | 1..191 chars, the UI route fragment (e.g. `student.admit`)       |
| `ParentRoute`     | Optional, the parent route (e.g. `student.index`)                |
| `PermissionType`  | `Menu`, `SubMenu`, or `Action` (encoded as `1`, `2`, `3`)         |
| `LangName`        | 1..191 chars, the i18n key                                        |
| `Icon`            | Up to 2000 chars, an icon class or svg payload                   |
| `Position`        | `i32` ordering hint                                              |
| `RelateToChild`   | `bool`                                                            |
| `IsMenu`          | `bool`                                                            |
| `IsAdmin`         | `bool`                                                            |
| `IsTeacher`       | `bool`                                                            |
| `IsStudent`       | `bool`                                                            |
| `IsParent`        | `bool`                                                            |
| `IsAlumni`        | `bool`                                                            |
| `AlternateModule` | Optional fallback module identifier                              |

## Assignment Overrides

| Type                | Constraints                                                       |
| ------------------- | ----------------------------------------------------------------- |
| `AssignmentStatus`  | `Granted` or `Revoked` (a deliberate denial)                     |
| `MenuStatus`        | `Visible` or `Hidden`                                             |
| `SaasSchoolList`    | `BTreeSet<SchoolId>`; non-empty in SaaS deployments              |

## Two-Factor

| Type                | Constraints                                                       |
| ------------------- | ----------------------------------------------------------------- |
| `TwoFactorChannel`  | `Sms`, `Email`                                                    |
| `TwoFactorMode`     | `Required`, `Optional`, `Disabled` (encoded 1, 2, 3)              |
| `TwoFactorExpiry`   | `u32` seconds, 0..86400 (typically 60..3600)                      |
| `OtpCode`           | 4..10 digits, the OTP payload (stored hashed in projection)       |
| `OtpHash`           | `OtpCode` after hashing with the configured algorithm             |

## Module

| Type                | Constraints                                                       |
| ------------------- | ----------------------------------------------------------------- |
| `ModuleName`        | 1..200 chars, unique within `school_id`                           |
| `DashboardId`       | `u32`, references a dashboard card                               |
| `ModulePosition`    | `i32`, sort order                                                |
| `ModuleStatus`      | `Active` or `Inactive`                                           |

## Validation Rules

All value objects implement `Validate` and refuse construction when
validation fails:

```rust
pub trait Validate {
    fn validate(&self) -> Result<(), ValueError>;
}
```

Construction is the only entry point:

```rust
let cap = Capability::from_str("Student.Admit")?;
let role_name = RoleName::new("Teacher")?;
let mode = TwoFactorMode::from_repr(1)?;
```

Parsing returns `Result<T, ValueError>`. There are no setters that
bypass validation.

## Type-Safe Wrappers

```rust
pub struct CapabilityString(String);

impl CapabilityString {
    pub fn new(s: &str) -> Result<Self, ValueError> {
        let cap: Capability = s.parse()?;
        Ok(Self(cap.to_string()))
    }
    pub fn as_capability(&self) -> Capability {
        self.0.parse().expect("validated at construction")
    }
}
```

`CapabilityString` is the type carried in the storage row. It cannot
be constructed without first parsing to a known `Capability`.
