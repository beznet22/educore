# Platform Domain — Commands

Commands describe intent. They are validated, authorized, and
dispatched to the relevant aggregate. Every command produces zero or
more events that are recorded in the event log.

All commands carry a `TenantContext` (school + actor + correlation) and
are rejected if the actor lacks the required capability. Platform
commands for a specific school require the actor to be authenticated
to that school; commands for global aggregates (`Country`,
`Currency`, `TimeZone`, `Continent`, `AddOn`, `ModuleManager`) are
system-internal and use a system tenant context.

## School

### CreateSchool

```rust
pub struct CreateSchoolCommand {
    pub tenant: TenantContext, // system tenant
    pub school_name: SchoolName,
    pub school_code: SchoolCode,
    pub domain: Domain,
    pub email: EmailAddress,
    pub phone: Option<PhoneNumber>,
    pub address: Option<Address>,
    pub starting_date: Option<NaiveDate>,
    pub ending_date: Option<NaiveDate>,
    pub package_id: Option<SchoolPackageId>,
    pub plan_type: PlanType,
    pub contact_type: ContactType,
    pub region: Option<Region>,
}
```

**Capability:** `Platform.School.Create`
**Pre-conditions:** `school_code`, `domain`, `email` are unique.
**Effects:** Creates the `School`, emits `SchoolCreated`, and
returns the new `SchoolId`. Subscribers (RBAC, settings, academic)
seed their bootstrapping data.

### UpdateSchool

```rust
pub struct UpdateSchoolCommand {
    pub tenant: TenantContext,
    pub school_id: SchoolId,
    pub school_name: Option<SchoolName>,
    pub email: Option<EmailAddress>,
    pub phone: Option<PhoneNumber>,
    pub address: Option<Address>,
    pub starting_date: Option<NaiveDate>,
    pub ending_date: Option<NaiveDate>,
    pub package_id: Option<SchoolPackageId>,
    pub plan_type: Option<PlanType>,
    pub region: Option<Region>,
}
```

**Capability:** `Platform.School.Update`
**Effects:** Emits `SchoolUpdated`.

### DeactivateSchool

```rust
pub struct DeactivateSchoolCommand {
    pub tenant: TenantContext, // system tenant
    pub school_id: SchoolId,
    pub reason: DeactivationReason,
}
```

**Capability:** `Platform.School.Deactivate`
**Pre-conditions:** School is not the bootstrap school (id 1).
**Effects:** Sets `active_status = Pending` and emits
`SchoolDeactivated`.

### ApproveSchool

```rust
pub struct ApproveSchoolCommand {
    pub tenant: TenantContext, // system tenant
    pub school_id: SchoolId,
    pub effective_from: NaiveDate,
}
```

**Capability:** `Platform.School.Approve`
**Effects:** Sets `active_status = Approved` and emits
`SchoolApproved`.

### DisableLogin / EnableLogin

```rust
pub struct DisableLoginCommand {
    pub tenant: TenantContext,
    pub school_id: SchoolId,
    pub reason: String,
}

pub struct EnableLoginCommand {
    pub tenant: TenantContext,
    pub school_id: SchoolId,
}
```

**Capabilities:** `Platform.School.DisableLogin`,
`Platform.School.EnableLogin`.
**Effects:** Toggle `is_enabled`. Emit `LoginDisabled` /
`LoginEnabled`.

## User

### RegisterUser

```rust
pub struct RegisterUserCommand {
    pub tenant: TenantContext,
    pub full_name: PersonName,
    pub username: Username,
    pub email: EmailAddress,
    pub phone_number: Option<PhoneNumber>,
    pub password: PasswordHash, // port-supplied hash
    pub usertype: UserType,
    pub role_id: Option<RoleId>,
    pub language: Option<LanguageCode>,
    pub is_administrator: IsAdministrator,
    pub is_registered: IsRegistered,
}
```

**Capability:** `Platform.User.Register`
**Pre-conditions:** `email`, `username`, `phone_number` are unique
within the school. If `role_id` is supplied, the role exists in
the same school.

**Effects:** Creates the `User`, binds the role, and emits
`UserRegistered`.

### UpdateUser

```rust
pub struct UpdateUserCommand {
    pub tenant: TenantContext,
    pub user_id: UserId,
    pub full_name: Option<PersonName>,
    pub phone_number: Option<PhoneNumber>,
    pub email: Option<EmailAddress>,
    pub language: Option<LanguageCode>,
    pub style_id: Option<StylePreference>,
    pub rtl_ltl: Option<RtlPreference>,
    pub selected_session: Option<SelectedSessionId>,
}
```

**Capability:** `Platform.User.Update`
**Effects:** Emits `UserUpdated`.

### DeactivateUser

```rust
pub struct DeactivateUserCommand {
    pub tenant: TenantContext,
    pub user_id: UserId,
    pub reason: DeactivationReason,
}
```

**Capability:** `Platform.User.Deactivate`
**Effects:** Sets `active_status=0` and emits `UserDeactivated`.
All sessions for the user are invalidated.

### ReactivateUser

```rust
pub struct ReactivateUserCommand {
    pub tenant: TenantContext,
    pub user_id: UserId,
}
```

**Capability:** `Platform.User.Reactivate`
**Effects:** Sets `active_status=1` and emits `UserReactivated`.

### ChangeUserRole

```rust
pub struct ChangeUserRoleCommand {
    pub tenant: TenantContext,
    pub user_id: UserId,
    pub role_id: RoleId,
}
```

**Capability:** `Platform.User.ChangeRole`
**Effects:** Updates the `role_id` reference and emits
`UserRoleChanged`.

### VerifyEmail

```rust
pub struct VerifyEmailCommand {
    pub tenant: TenantContext,
    pub user_id: UserId,
    pub token: String, // port-supplied verification token
}
```

**Capability:** `Platform.User.VerifyEmail`
**Effects:** Sets `verified=1` and `is_email_verified=1` and
emits `EmailVerified`.

### ResetPassword

```rust
pub struct ResetPasswordCommand {
    pub tenant: TenantContext,
    pub user_id: UserId,
    pub new_password_hash: PasswordHash,
}
```

**Capability:** `Platform.User.ResetPassword`
**Effects:** Updates the stored hash and emits `PasswordReset`.

## OTP

### IssueOtp

```rust
pub struct IssueOtpCommand {
    pub tenant: TenantContext,
    pub user_id: UserId,
    pub channel: OtpChannel,
    pub expiry: OtpExpiry,
}
```

**Capability:** `Platform.Otp.Issue`
**Pre-conditions:** User exists and is active.
**Effects:** Creates an `OtpCode`, emits `OtpIssued`, and
publishes a notification through the notification port.

### VerifyOtp

```rust
pub struct VerifyOtpCommand {
    pub tenant: TenantContext,
    pub user_id: UserId,
    pub code: OtpCode,
}
```

**Capability:** `Platform.Otp.Verify`
**Pre-conditions:** Code is not expired and has not been consumed.
**Effects:** Marks the OTP consumed and emits `OtpVerified`.

### ExpireOtp

```rust
pub struct ExpireOtpCommand {
    pub tenant: TenantContext,
    pub otp_id: OtpCodeId,
}
```

**Capability:** `Platform.Otp.Expire` (system)
**Effects:** Marks the OTP expired and emits `OtpExpired`. A
nightly job issues `ExpireOtp` for any code whose `expired_time`
has passed.

## Course

### CreateCourse

```rust
pub struct CreateCourseCommand {
    pub tenant: TenantContext,
    pub title: CourseTitle,
    pub image: CourseImage,
    pub category_id: CourseCategoryId,
    pub overview: Option<CourseOverview>,
    pub outline: Option<CourseOutline>,
    pub prerequisites: Option<Prerequisites>,
    pub resources: Option<Resources>,
    pub stats: Option<Stats>,
}
```

**Capability:** `Platform.Course.Create`
**Effects:** Emits `CourseCreated`.

### UpdateCourse / DeleteCourse / PublishCourse / UnpublishCourse

Standard CRUD on `Course`.

**Capabilities:** `Platform.Course.Update`, `Platform.Course.Delete`,
`Platform.Course.Publish`, `Platform.Course.Unpublish`.

### CreateCourseCategory / UpdateCourseCategory / DeleteCourseCategory

Standard CRUD on `CourseCategory`.

**Capabilities:** `Platform.CourseCategory.Create`,
`Platform.CourseCategory.Update`, `Platform.CourseCategory.Delete`.

### CreateCoursePage / UpdateCoursePage / DeleteCoursePage

Standard CRUD on `CoursePage`.

**Capabilities:** `Platform.CoursePage.Create`,
`Platform.CoursePage.Update`, `Platform.CoursePage.Delete`.

## Custom Field

### CreateCustomField

```rust
pub struct CreateCustomFieldCommand {
    pub tenant: TenantContext,
    pub form_name: FormName,
    pub label: FieldLabel,
    pub field_type: FieldType,
    pub min_max_length: Option<LengthRange>,
    pub min_max_value: Option<ValueRange>,
    pub name_value: Option<NameValueList>,
    pub width: Option<Width>,
    pub required: IsRequired,
    pub academic_id: AcademicYearId,
}
```

**Capability:** `Platform.CustomField.Create`
**Effects:** Emits `CustomFieldCreated`.

### UpdateCustomField / DeleteCustomField

Standard CRUD on `CustomField`.

### SetCustomFieldValue

```rust
pub struct SetCustomFieldValueCommand {
    pub tenant: TenantContext,
    pub custom_field_id: CustomFieldId,
    pub entity_id: Uuid,
    pub entity_type: EntityType,
    pub value: FieldValue,
}
```

**Capability:** `Platform.CustomFieldValue.Set`
**Effects:** Writes a `CustomFieldValue` row and emits
`CustomFieldValueSet`.

### ClearCustomFieldValue

```rust
pub struct ClearCustomFieldValueCommand {
    pub tenant: TenantContext,
    pub custom_field_id: CustomFieldId,
    pub entity_id: Uuid,
    pub entity_type: EntityType,
}
```

**Capability:** `Platform.CustomFieldValue.Clear`
**Effects:** Deletes the value row and emits
`CustomFieldValueCleared`.

## Chart of Account

### CreateChartOfAccount / UpdateChartOfAccount / DeleteChartOfAccount

```rust
pub struct CreateChartOfAccountCommand {
    pub tenant: TenantContext,
    pub head: AccountHead,
    pub account_type: AccountType,
}
```

**Capabilities:** `Platform.ChartOfAccount.Create`,
`Platform.ChartOfAccount.Update`, `Platform.ChartOfAccount.Delete`.

## Base Setup

### CreateBaseGroup / UpdateBaseGroup / DeleteBaseGroup

Standard CRUD on `BaseGroup`.

### CreateBaseSetup / UpdateBaseSetup / DeleteBaseSetup

```rust
pub struct CreateBaseSetupCommand {
    pub tenant: TenantContext,
    pub base_setup_name: BaseSetupName,
    pub base_group_id: BaseGroupId,
}
```

**Capabilities:** `Platform.BaseSetup.Create`,
`Platform.BaseSetup.Update`, `Platform.BaseSetup.Delete`.

## Module

### CreateModule / UpdateModule / DeleteModule / ReorderModules

Standard CRUD on `Module`.

**Capabilities:** `Platform.Module.Create`,
`Platform.Module.Update`, `Platform.Module.Delete`,
`Platform.Module.Reorder`.

### CreateModuleLink / UpdateModuleLink / DeleteModuleLink

Standard CRUD on `ModuleLink`.

**Capabilities:** `Platform.ModuleLink.Create`,
`Platform.ModuleLink.Update`, `Platform.ModuleLink.Delete`.

### EnableModule

```rust
pub struct EnableModuleCommand {
    pub tenant: TenantContext,
    pub module_id: ModuleId,
}
```

**Capability:** `Platform.Module.Enable`
**Effects:** Adds the module to the school's enabled set and
emits `ModuleEnabled`.

### DisableModule

```rust
pub struct DisableModuleCommand {
    pub tenant: TenantContext,
    pub module_id: ModuleId,
}
```

**Capability:** `Platform.Module.Disable`
**Effects:** Removes the module from the school's enabled set
and emits `ModuleDisabled`.

### InstallAddOn

```rust
pub struct InstallAddOnCommand {
    pub tenant: TenantContext,
    pub add_on_id: AddOnId,
}
```

**Capability:** `Platform.AddOn.Install`
**Pre-conditions:** The add-on's package is available to the
school's plan.

**Effects:** Creates an installation record and emits
`AddOnInstalled`.

### UninstallAddOn

```rust
pub struct UninstallAddOnCommand {
    pub tenant: TenantContext,
    pub add_on_id: AddOnId,
}
```

**Capability:** `Platform.AddOn.Uninstall`
**Effects:** Removes the installation record and emits
`AddOnUninstalled`.

## Locale

### RegisterTimeZone / UpdateTimeZone

System-internal commands for managing the global timezone
catalog. Typically called by build-time seed scripts.

### RegisterCountry / UpdateCountry

System-internal commands for managing the global country
catalog.

### RegisterContinent / UpdateContinent

System-internal commands for managing the global continent
catalog.

### CreateCurrency / UpdateCurrency / DeleteCurrency

Per-school currency catalog management.

**Capabilities:** `Platform.Currency.Create`,
`Platform.Currency.Update`, `Platform.Currency.Delete`.

### CreateLanguage / UpdateLanguage / DeleteLanguage

Per-school language catalog management.

**Capabilities:** `Platform.Language.Create`,
`Platform.Language.Update`, `Platform.Language.Delete`.

## Front Office

### CreateSocialMediaIcon / UpdateSocialMediaIcon / DeleteSocialMediaIcon

Standard CRUD on `SocialMediaIcon`.

**Capabilities:** `Platform.SocialMediaIcon.Create`,
`Platform.SocialMediaIcon.Update`, `Platform.SocialMediaIcon.Delete`.

### CreateHeaderMenuItem / UpdateHeaderMenuItem / DeleteHeaderMenuItem / ReorderHeaderMenu

Standard CRUD on `HeaderMenuManager`.

**Capabilities:** `Platform.HeaderMenu.Create`,
`Platform.HeaderMenu.Update`, `Platform.HeaderMenu.Delete`,
`Platform.HeaderMenu.Reorder`.

### CreatePhotoGallery / UpdatePhotoGallery / DeletePhotoGallery / PublishPhotoGallery / UnpublishPhotoGallery

Standard CRUD on `PhotoGallery`.

**Capabilities:** `Platform.PhotoGallery.Create`,
`Platform.PhotoGallery.Update`, `Platform.PhotoGallery.Delete`,
`Platform.PhotoGallery.Publish`, `Platform.PhotoGallery.Unpublish`.

### CreateVideoGallery / UpdateVideoGallery / DeleteVideoGallery / PublishVideoGallery / UnpublishVideoGallery

Standard CRUD on `VideoGallery`.

**Capabilities:** `Platform.VideoGallery.Create`,
`Platform.VideoGallery.Update`, `Platform.VideoGallery.Delete`,
`Platform.VideoGallery.Publish`, `Platform.VideoGallery.Unpublish`.

### CreateInstruction / UpdateInstruction / DeleteInstruction

Standard CRUD on `Instruction`.

**Capabilities:** `Platform.Instruction.Create`,
`Platform.Instruction.Update`, `Platform.Instruction.Delete`.

### MarkExpertTeacher / UnmarkExpertTeacher / ReorderExpertTeachers

Standard CRUD on `ExpertTeacher`.

**Capabilities:** `Platform.ExpertTeacher.Create`,
`Platform.ExpertTeacher.Delete`, `Platform.ExpertTeacher.Reorder`.

### CreateFrontendPermission / UpdateFrontendPermission / DeleteFrontendPermission / PublishFrontendPermission / UnpublishFrontendPermission

Standard CRUD on `FrontendPermission`.

**Capabilities:** `Platform.FrontendPermission.Create`,
`Platform.FrontendPermission.Update`,
`Platform.FrontendPermission.Delete`,
`Platform.FrontendPermission.Publish`,
`Platform.FrontendPermission.Unpublish`.

## Operational

### RecordVisitor / UpdateVisitor / DeleteVisitor

Standard CRUD on `Visitor`.

**Capabilities:** `Platform.Visitor.Create`,
`Platform.Visitor.Update`, `Platform.Visitor.Delete`.

### CreateToDo / UpdateToDo / MarkToDoComplete / DeleteToDo

Standard CRUD on `ToDo`.

**Capabilities:** `Platform.ToDo.Create`, `Platform.ToDo.Update`,
`Platform.ToDo.Complete`, `Platform.ToDo.Delete`.

### CreateAmountTransfer / UpdateAmountTransfer / DeleteAmountTransfer

Standard CRUD on `AmountTransfer`.

**Capabilities:** `Platform.AmountTransfer.Create`,
`Platform.AmountTransfer.Update`, `Platform.AmountTransfer.Delete`.

## Plugin

### EnablePlugin / DisablePlugin / UpdatePlugin

```rust
pub struct EnablePluginCommand {
    pub tenant: TenantContext,
    pub plugin_id: PluginId,
    pub availability: PluginAvailability,
    pub position: PluginPosition,
    pub applicable_for: Option<String>,
}

pub struct UpdatePluginCommand {
    pub tenant: TenantContext,
    pub plugin_id: PluginId,
    pub is_enable: Option<bool>,
    pub availability: Option<PluginAvailability>,
    pub show_admin_panel: Option<bool>,
    pub show_website: Option<bool>,
    pub showing_page: Option<ShowingPage>,
    pub applicable_for: Option<String>,
    pub position: Option<PluginPosition>,
}
```

**Capabilities:** `Platform.Plugin.Enable`,
`Platform.Plugin.Disable`, `Platform.Plugin.Update`.

## Comment

### CreateComment / UpdateComment / FlagComment / DeleteComment

Standard CRUD on `Comment`.

**Capabilities:** `Platform.Comment.Create`,
`Platform.Comment.Update`, `Platform.Comment.Flag`,
`Platform.Comment.Delete`.

### CreateCommentTag / DeleteCommentTag

Standard CRUD on `CommentTag`.

**Capabilities:** `Platform.CommentTag.Create`,
`Platform.CommentTag.Delete`.

## Personal Access Token

### IssuePersonalAccessToken

```rust
pub struct IssuePersonalAccessTokenCommand {
    pub tenant: TenantContext,
    pub user_id: UserId,
    pub name: TokenName,
    pub abilities: BTreeSet<Capability>,
    pub expires_at: Option<Timestamp>,
}
```

**Capability:** `Platform.Token.Issue`
**Effects:** Creates a `PersonalAccessToken` and emits
`PersonalAccessTokenIssued`. The plaintext token is returned
exactly once to the caller and is never persisted.

### RevokePersonalAccessToken

```rust
pub struct RevokePersonalAccessTokenCommand {
    pub tenant: TenantContext,
    pub token_id: PersonalAccessTokenId,
}
```

**Capability:** `Platform.Token.Revoke`
**Effects:** Hard-deletes the token and emits
`PersonalAccessTokenRevoked`.

## Video Upload

### UploadVideo / UpdateVideo / DeleteVideo

Standard CRUD on `VideoUpload`.

**Capabilities:** `Platform.Video.Create`, `Platform.Video.Update`,
`Platform.Video.Delete`.
