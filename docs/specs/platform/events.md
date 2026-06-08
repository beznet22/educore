# Platform Domain — Events

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

## School Lifecycle

### SchoolCreated

```rust
pub struct SchoolCreated {
    pub school_id: SchoolId,
    pub school_name: SchoolName,
    pub school_code: SchoolCode,
    pub domain: Domain,
    pub package_id: Option<SchoolPackageId>,
}
```

**Subscribers:**
- `rbac` — seeds the bootstrap role catalog and assigns the
  school admin to `SuperAdmin`.
- `settings` — seeds the default `GeneralSettings` row.
- `academic` — seeds the first `AcademicYear`.

### SchoolUpdated

```rust
pub struct SchoolUpdated {
    pub school_id: SchoolId,
    pub changed_fields: Vec<&'static str>,
}
```

### SchoolDeactivated

```rust
pub struct SchoolDeactivated {
    pub school_id: SchoolId,
    pub reason: DeactivationReason,
}
```

### SchoolApproved

```rust
pub struct SchoolApproved {
    pub school_id: SchoolId,
    pub effective_from: NaiveDate,
}
```

### LoginDisabled

```rust
pub struct LoginDisabled {
    pub school_id: SchoolId,
    pub reason: String,
}
```

### LoginEnabled

```rust
pub struct LoginEnabled {
    pub school_id: SchoolId,
}
```

## User Lifecycle

### UserRegistered

```rust
pub struct UserRegistered {
    pub user_id: UserId,
    pub email: EmailAddress,
    pub full_name: FullName,
    pub usertype: UserType,
    pub role_id: Option<RoleId>,
    pub is_administrator: IsAdministrator,
}
```

**Subscribers:**
- `rbac` — materializes the role binding; the user inherits the
  role's effective capability set.

### UserUpdated

```rust
pub struct UserUpdated {
    pub user_id: UserId,
    pub changed_fields: Vec<&'static str>,
}
```

### UserDeactivated

```rust
pub struct UserDeactivated {
    pub user_id: UserId,
    pub reason: DeactivationReason,
}
```

**Subscribers:**
- `rbac` — invalidates all sessions for the user.
- `operations` — writes a `UserLogged` audit entry.

### UserReactivated

```rust
pub struct UserReactivated {
    pub user_id: UserId,
}
```

### UserRoleChanged

```rust
pub struct UserRoleChanged {
    pub user_id: UserId,
    pub from_role_id: Option<RoleId>,
    pub to_role_id: RoleId,
}
```

**Subscribers:**
- `rbac` — invalidates the capability cache for the user.

### EmailVerified

```rust
pub struct EmailVerified {
    pub user_id: UserId,
    pub verified_at: Timestamp,
}
```

### PasswordReset

```rust
pub struct PasswordReset {
    pub user_id: UserId,
    pub reset_at: Timestamp,
}
```

## OTP Lifecycle

### OtpIssued

```rust
pub struct OtpIssued {
    pub otp_id: OtpCodeId,
    pub user_id: UserId,
    pub channel: OtpChannel,
    pub expired_time: OtpExpiry,
}
```

**Subscribers:**
- `communication` — delivers the OTP via SMS or email.

### OtpVerified

```rust
pub struct OtpVerified {
    pub otp_id: OtpCodeId,
    pub user_id: UserId,
}
```

### OtpExpired

```rust
pub struct OtpExpired {
    pub otp_id: OtpCodeId,
    pub user_id: UserId,
}
```

## Course Lifecycle

- `CourseCreated { course_id, title, category_id }`
- `CourseUpdated { course_id, changed_fields }`
- `CourseDeleted { course_id }`
- `CoursePublished { course_id }`
- `CourseUnpublished { course_id }`
- `CourseCategoryCreated { category_id, category_name }`
- `CourseCategoryUpdated { category_id, changed_fields }`
- `CourseCategoryDeleted { category_id }`
- `CoursePageCreated { course_page_id, course_id }`
- `CoursePageUpdated { course_page_id, changed_fields }`
- `CoursePageDeleted { course_page_id }`

## Custom Field Lifecycle

- `CustomFieldCreated { custom_field_id, form_name, label, field_type }`
- `CustomFieldUpdated { custom_field_id, changed_fields }`
- `CustomFieldDeleted { custom_field_id }`
- `CustomFieldValueSet { custom_field_id, entity_id, entity_type, value }`
- `CustomFieldValueCleared { custom_field_id, entity_id, entity_type }`

## Chart of Account Lifecycle

- `ChartOfAccountCreated { id, head, account_type }`
- `ChartOfAccountUpdated { id, changed_fields }`
- `ChartOfAccountDeleted { id }`

## Base Setup Lifecycle

- `BaseGroupCreated { id, name }`
- `BaseGroupUpdated { id, changed_fields }`
- `BaseGroupDeleted { id }`
- `BaseSetupCreated { id, name, base_group_id }`
- `BaseSetupUpdated { id, changed_fields }`
- `BaseSetupDeleted { id }`

## Module Lifecycle

### ModuleCreated

```rust
pub struct ModuleCreated {
    pub module_id: ModuleId,
    pub name: ModuleName,
    pub order: ModuleOrder,
}
```

### ModuleEnabled

```rust
pub struct ModuleEnabled {
    pub school_id: SchoolId,
    pub module_id: ModuleId,
    pub enabled_at: Timestamp,
}
```

**Subscribers:**
- `rbac` — materializes the module's `ModuleLink` rows into the
  role's menu.
- `settings` — refreshes the dashboard card layout.

### ModuleDisabled

```rust
pub struct ModuleDisabled {
    pub school_id: SchoolId,
    pub module_id: ModuleId,
    pub disabled_at: Timestamp,
}
```

### ModuleLinkCreated / ModuleLinkUpdated / ModuleLinkDeleted

```rust
pub struct ModuleLinkCreated {
    pub module_link_id: ModuleLinkId,
    pub module_id: ModuleId,
    pub name: String,
    pub route: String,
}
```

## AddOn Lifecycle

### AddOnRegistered

```rust
pub struct AddOnRegistered {
    pub add_on_id: AddOnId,
    pub name: PackageName,
    pub version: Version,
}
```

### AddOnInstalled

```rust
pub struct AddOnInstalled {
    pub school_id: SchoolId,
    pub add_on_id: AddOnId,
    pub installed_at: Timestamp,
}
```

### AddOnUninstalled

```rust
pub struct AddOnUninstalled {
    pub school_id: SchoolId,
    pub add_on_id: AddOnId,
    pub uninstalled_at: Timestamp,
}
```

## ModuleManager Lifecycle

- `ModuleManagerRegistered { id, email }`
- `ModuleManagerUpdated { id, changed_fields }`
- `PurchaseCodeRotated { id, new_code_hash }`

## Student / Parent Menu Lifecycle

- `StudentParentMenuConfigured { school_id, module_name, modules, menus }`
- `StudentParentMenuReset { school_id, module_name }`

## Locale Lifecycle

- `TimeZoneRegistered { id, code, time_zone }`
- `CountryRegistered { id, code, name }`
- `CountryUpdated { id, changed_fields }`
- `ContinentRegistered { id, code, name }`
- `ContinentUpdated { id, changed_fields }`
- `CurrencyCreated { id, code, symbol }`
- `CurrencyUpdated { id, changed_fields }`
- `CurrencyDeleted { id }`
- `LanguageCreated { id, code, name }`
- `LanguageUpdated { id, changed_fields }`
- `LanguageDeleted { id }`

## Front Office Lifecycle

- `SocialMediaIconCreated { id, url, icon }`
- `SocialMediaIconUpdated { id, changed_fields }`
- `SocialMediaIconDeleted { id }`
- `HeaderMenuItemCreated { id, type, title, position }`
- `HeaderMenuItemUpdated { id, changed_fields }`
- `HeaderMenuItemDeleted { id }`
- `PhotoGalleryCreated { id, name, parent_id? }`
- `PhotoGalleryUpdated { id, changed_fields }`
- `PhotoGalleryDeleted { id }`
- `PhotoGalleryPublished { id }`
- `PhotoGalleryUnpublished { id }`
- `VideoGalleryCreated { id, name, video_link }`
- `VideoGalleryUpdated { id, changed_fields }`
- `VideoGalleryDeleted { id }`
- `VideoGalleryPublished { id }`
- `VideoGalleryUnpublished { id }`
- `InstructionCreated { id, title }`
- `InstructionUpdated { id, changed_fields }`
- `InstructionDeleted { id }`
- `ExpertTeacherMarked { id, staff_id, position }`
- `ExpertTeacherUnmarked { id, staff_id }`

## Frontend Permission Lifecycle

- `FrontendPermissionCreated { id, name }`
- `FrontendPermissionUpdated { id, changed_fields }`
- `FrontendPermissionDeleted { id }`
- `FrontendPermissionPublished { id }`
- `FrontendPermissionUnpublished { id }`

## Operational Lifecycle

- `VisitorRecorded { id, name, date, in_time? }`
- `VisitorUpdated { id, changed_fields }`
- `VisitorDeleted { id }`
- `ToDoCreated { id, title, due_date }`
- `ToDoUpdated { id, changed_fields }`
- `ToDoCompleted { id, completed_at }`
- `ToDoDeleted { id }`
- `AmountTransferCreated { id, amount, from_method, to_method, transfer_date }`
- `AmountTransferUpdated { id, changed_fields }`
- `AmountTransferDeleted { id }`

## Plugin Lifecycle

### PluginEnabled

```rust
pub struct PluginEnabled {
    pub plugin_id: PluginId,
    pub availability: PluginAvailability,
    pub position: PluginPosition,
}
```

### PluginDisabled

```rust
pub struct PluginDisabled {
    pub plugin_id: PluginId,
}
```

### PluginUpdated

```rust
pub struct PluginUpdated {
    pub plugin_id: PluginId,
    pub changed_fields: Vec<&'static str>,
}
```

## Comment Lifecycle

- `CommentCreated { id, text, is_flagged }`
- `CommentUpdated { id, changed_fields }`
- `CommentFlagged { id, flagged_at }`
- `CommentDeleted { id }`
- `CommentTagCreated { id, tag }`
- `CommentTagDeleted { id, tag }`

## Personal Access Token Lifecycle

### PersonalAccessTokenIssued

```rust
pub struct PersonalAccessTokenIssued {
    pub token_id: PersonalAccessTokenId,
    pub user_id: UserId,
    pub name: TokenName,
    pub abilities: BTreeSet<Capability>,
    pub expires_at: Option<Timestamp>,
}
```

### PersonalAccessTokenRevoked

```rust
pub struct PersonalAccessTokenRevoked {
    pub token_id: PersonalAccessTokenId,
    pub user_id: UserId,
}
```

### PersonalAccessTokenExpired

```rust
pub struct PersonalAccessTokenExpired {
    pub token_id: PersonalAccessTokenId,
    pub user_id: UserId,
    pub expired_at: Timestamp,
}
```

## Video Upload Lifecycle

- `VideoUploaded { id, title, class_id, section_id, youtube_link }`
- `VideoUpdated { id, changed_fields }`
- `VideoDeleted { id }`
