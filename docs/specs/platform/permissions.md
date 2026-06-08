# Platform Domain — Permissions

Permissions are capability strings. They are not roles. The RBAC
domain maps capabilities to roles.

## Naming

```text
<Domain>.<Aggregate>.<Action>
```

For the platform domain, the convention is `Platform.*`.

## Capabilities

### School

- `Platform.School.Create`
- `Platform.School.Read`
- `Platform.School.Update`
- `Platform.School.Deactivate`
- `Platform.School.Approve`
- `Platform.School.DisableLogin`
- `Platform.School.EnableLogin`

### User

- `Platform.User.Register`
- `Platform.User.Read`
- `Platform.User.Update`
- `Platform.User.Deactivate`
- `Platform.User.Reactivate`
- `Platform.User.ChangeRole`
- `Platform.User.VerifyEmail`
- `Platform.User.ResetPassword`
- `Platform.User.ReadSensitive` — for reading PII fields (email,
  phone) of users in a different school.

### OTP

- `Platform.Otp.Issue`
- `Platform.Otp.Verify`
- `Platform.Otp.Expire`

### Course

- `Platform.Course.Create`
- `Platform.Course.Read`
- `Platform.Course.Update`
- `Platform.Course.Delete`
- `Platform.Course.Publish`
- `Platform.Course.Unpublish`

### CourseCategory

- `Platform.CourseCategory.Create`
- `Platform.CourseCategory.Read`
- `Platform.CourseCategory.Update`
- `Platform.CourseCategory.Delete`

### CoursePage

- `Platform.CoursePage.Create`
- `Platform.CoursePage.Read`
- `Platform.CoursePage.Update`
- `Platform.CoursePage.Delete`

### CustomField

- `Platform.CustomField.Create`
- `Platform.CustomField.Read`
- `Platform.CustomField.Update`
- `Platform.CustomField.Delete`
- `Platform.CustomFieldValue.Set`
- `Platform.CustomFieldValue.Read`
- `Platform.CustomFieldValue.Clear`

### ChartOfAccount

- `Platform.ChartOfAccount.Create`
- `Platform.ChartOfAccount.Read`
- `Platform.ChartOfAccount.Update`
- `Platform.ChartOfAccount.Delete`

### BaseSetup

- `Platform.BaseGroup.Create`
- `Platform.BaseGroup.Read`
- `Platform.BaseGroup.Update`
- `Platform.BaseGroup.Delete`
- `Platform.BaseSetup.Create`
- `Platform.BaseSetup.Read`
- `Platform.BaseSetup.Update`
- `Platform.BaseSetup.Delete`

### Module

- `Platform.Module.Create`
- `Platform.Module.Read`
- `Platform.Module.Update`
- `Platform.Module.Delete`
- `Platform.Module.Enable`
- `Platform.Module.Disable`
- `Platform.Module.Reorder`
- `Platform.ModuleLink.Create`
- `Platform.ModuleLink.Read`
- `Platform.ModuleLink.Update`
- `Platform.ModuleLink.Delete`

### AddOn

- `Platform.AddOn.Register`
- `Platform.AddOn.Install`
- `Platform.AddOn.Uninstall`
- `Platform.AddOn.Read`

### ModuleManager

- `Platform.ModuleManager.Register`
- `Platform.ModuleManager.Update`
- `Platform.ModuleManager.RotatePurchaseCode`
- `Platform.ModuleManager.Read`

### ModuleStudentParentInfo

- `Platform.StudentParentMenu.Configure`
- `Platform.StudentParentMenu.Reset`
- `Platform.StudentParentMenu.Read`

### Locale

- `Platform.TimeZone.Register`
- `Platform.TimeZone.Update`
- `Platform.Country.Register`
- `Platform.Country.Update`
- `Platform.Continent.Register`
- `Platform.Continent.Update`
- `Platform.Currency.Create`
- `Platform.Currency.Read`
- `Platform.Currency.Update`
- `Platform.Currency.Delete`
- `Platform.Language.Create`
- `Platform.Language.Read`
- `Platform.Language.Update`
- `Platform.Language.Delete`

### Front Office

- `Platform.SocialMediaIcon.Create`
- `Platform.SocialMediaIcon.Read`
- `Platform.SocialMediaIcon.Update`
- `Platform.SocialMediaIcon.Delete`
- `Platform.HeaderMenu.Create`
- `Platform.HeaderMenu.Read`
- `Platform.HeaderMenu.Update`
- `Platform.HeaderMenu.Delete`
- `Platform.HeaderMenu.Reorder`
- `Platform.PhotoGallery.Create`
- `Platform.PhotoGallery.Read`
- `Platform.PhotoGallery.Update`
- `Platform.PhotoGallery.Delete`
- `Platform.PhotoGallery.Publish`
- `Platform.PhotoGallery.Unpublish`
- `Platform.VideoGallery.Create`
- `Platform.VideoGallery.Read`
- `Platform.VideoGallery.Update`
- `Platform.VideoGallery.Delete`
- `Platform.VideoGallery.Publish`
- `Platform.VideoGallery.Unpublish`
- `Platform.Instruction.Create`
- `Platform.Instruction.Read`
- `Platform.Instruction.Update`
- `Platform.Instruction.Delete`
- `Platform.ExpertTeacher.Create`
- `Platform.ExpertTeacher.Read`
- `Platform.ExpertTeacher.Delete`
- `Platform.ExpertTeacher.Reorder`
- `Platform.FrontendPermission.Create`
- `Platform.FrontendPermission.Read`
- `Platform.FrontendPermission.Update`
- `Platform.FrontendPermission.Delete`
- `Platform.FrontendPermission.Publish`
- `Platform.FrontendPermission.Unpublish`

### Operational

- `Platform.Visitor.Create`
- `Platform.Visitor.Read`
- `Platform.Visitor.Update`
- `Platform.Visitor.Delete`
- `Platform.ToDo.Create`
- `Platform.ToDo.Read`
- `Platform.ToDo.Update`
- `Platform.ToDo.Complete`
- `Platform.ToDo.Delete`
- `Platform.AmountTransfer.Create`
- `Platform.AmountTransfer.Read`
- `Platform.AmountTransfer.Update`
- `Platform.AmountTransfer.Delete`

### Plugin

- `Platform.Plugin.Enable`
- `Platform.Plugin.Disable`
- `Platform.Plugin.Update`
- `Platform.Plugin.Read`

### Comment

- `Platform.Comment.Create`
- `Platform.Comment.Read`
- `Platform.Comment.Update`
- `Platform.Comment.Flag`
- `Platform.Comment.Delete`
- `Platform.CommentTag.Create`
- `Platform.CommentTag.Delete`

### PersonalAccessToken

- `Platform.Token.Issue`
- `Platform.Token.Revoke`
- `Platform.Token.Read`

### VideoUpload

- `Platform.Video.Create`
- `Platform.Video.Read`
- `Platform.Video.Update`
- `Platform.Video.Delete`

## Default Role Mapping

| Role             | Capabilities (highlights)                                              |
| ---------------- | ---------------------------------------------------------------------- |
| SuperAdmin       | All                                                                    |
| SchoolAdmin      | All within the school                                                 |
| Teacher          | `Platform.User.Read`, `Platform.Course.*`, `Platform.Video.*`         |
| Student          | `Platform.User.Read (self)`, `Platform.Course.Read`                   |
| Parent           | `Platform.User.Read (self)`, `Platform.Course.Read`                   |
| Accountant       | `Platform.ChartOfAccount.*`, `Platform.AmountTransfer.*`              |
| Receptionist     | `Platform.Visitor.*`                                                  |
| Librarian        | `Platform.BaseSetup.Read`                                             |
| Driver           | `Platform.User.Read (self)`                                           |

The default mapping is configurable per school.

## Authorization Pattern

Capabilities are checked at the command boundary:

```rust
if !engine.rbac().has(actor_id, Capability::PlatformUserRegister).await? {
    return Err(DomainError::forbidden("missing capability"));
}
```

A user updating their own profile is allowed with `Platform.User.Read`
(self) and the secondary ownership check on `user_id == actor_id`.

## Read vs Write

Read capabilities are explicit. The engine does not assume
"Platform.User.Read" implies "Platform.User.Update".

## Tenant Isolation

Every capability check is paired with a tenant check. Cross-tenant
operations on platform data (e.g. moving a user from school A to
school B) are not supported; the user must be re-registered in the
target school.

## Self-Authorization

The bootstrap user (school admin) holds the `SuperAdmin` role at
school creation. The engine refuses to demote or deactivate the
last remaining `SuperAdmin` in a school.
