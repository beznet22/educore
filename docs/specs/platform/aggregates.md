# Platform Domain — Aggregates

## School

**Root type:** `School`
**Identity:** `SchoolId(Uuid)` (schools are globally unique; not
tenant-scoped in the same way as other aggregates)
**Tenant:** None (school is the tenant itself)

### Purpose

The tenant root. Represents a school or organization that operates
the engine. Holds the school's identity, contact information,
package, plan, and active status.

### Owned Children

- `User` — all users of the school.
- `Course`, `CourseCategory` — the school's course catalog.
- `CustomField` — the school's custom field definitions.
- `ChartOfAccount` — the school's accounting heads.
- `BaseGroup`, `BaseSetup` — the school's lookup values.
- `Module` enablement (`SchoolModule`).
- `Plugin` enablement (`Plugin`).
- `AddOn` registrations.
- `FrontendPermission` — the school's public-facing rules.
- `HeaderMenuManager` — the public header menu.
- `PhotoGallery`, `VideoGallery` — the public galleries.
- `Instruction` — front-office instructions.
- `SocialMediaIcon` — the school's social links.
- `Visitor`, `ToDo` — operational logs.
- `AmountTransfer` — the school's fund transfers.

### Invariants

1. A `School::school_name` is non-empty.
2. A `School::domain` is unique across the platform.
3. A `School::school_code` is unique across the platform.
4. A `School::email` is RFC-valid and unique.
5. A `School::active_status` is `Approved` or `Pending`.
6. A `School::is_enabled` is `Yes` or `No` (login enable/disable).
7. A `School::plan_type` carries the package's billing mode.
8. The bootstrap `School` (id 1) cannot be deleted.
9. A `School::starting_date` is on or before `ending_date` when
   both are set.
10. A `School::region` references a known continent/country id.

### Commands

- `CreateSchool`
- `UpdateSchool`
- `DeactivateSchool`
- `ApproveSchool`
- `DisableLogin`
- `EnableLogin`

### Events

- `SchoolCreated`
- `SchoolUpdated`
- `SchoolDeactivated`
- `SchoolApproved`
- `LoginDisabled`
- `LoginEnabled`

### Consistency Boundary

A school is loaded by id, mutated in memory, validated, and
persisted with its events in a single transaction. Children
(Users, Courses, etc.) are independent aggregates and are
mutated through their own commands.

---

## User

**Root type:** `User`
**Identity:** `UserId(SchoolId, Uuid)`
**Tenant:** `SchoolId`

### Purpose

An actor in the system. Holds the user's identity, contact
information, status, language preference, role binding, and
authentication-related fields (random code, notification token,
remember token, OTP).

### Owned Children

- `PersonalAccessToken` — the user's API tokens.
- `OtpCode` — the OTPs issued to the user.
- `VideoUpload` — videos the user has uploaded (if teacher).

### Invariants

1. A `User` belongs to exactly one school.
2. A `User::email` is unique within `(school_id, lower(email))`.
3. A `User::phone_number` is unique within `(school_id,
   normalized_phone)`.
4. A `User::username` is unique within `(school_id,
   lower(username))`.
5. A `User::usertype` is one of the closed enum values.
6. A `User::active_status` is a boolean (1 = active, 0 = inactive).
7. A `User::is_administrator` is `Yes` or `No`.
8. A `User::role_id` references a valid role in the same school.
9. A `User::language` is a known `LanguageCode`.
10. A `User::is_registered` is a boolean indicating whether the
    user has completed self-registration.
11. Deleting a `User` is soft (`active_status=0`); hard delete is
    only available to operators via a port-driven action.

### Commands

- `RegisterUser`
- `UpdateUser`
- `DeactivateUser`
- `ReactivateUser`
- `ChangeUserRole`
- `VerifyEmail`
- `ResetPassword`

### Events

- `UserRegistered`
- `UserUpdated`
- `UserDeactivated`
- `UserReactivated`
- `UserRoleChanged`
- `EmailVerified`
- `PasswordReset`

---

## OtpCode

**Root type:** `OtpCode`
**Identity:** `OtpCodeId(SchoolId, Uuid)`

### Purpose

A one-time password issued to a user, typically for the
forgot-password flow or for the 2FA second factor. Carries the
OTP payload (hashed), the expiry timestamp, and the consumption
state.

### Invariants

1. An `OtpCode` references a `User`.
2. An `OtpCode::otp_code` is a hashed payload (the plaintext is
   never stored).
3. An `OtpCode::expired_time` is a timestamp; the code is
   rejected after this point.
4. An `OtpCode` may be consumed at most once.

### Commands

- `IssueOtp`
- `VerifyOtp`
- `ExpireOtp`

### Events

- `OtpIssued`
- `OtpVerified`
- `OtpExpired`

---

## Course

**Root type:** `Course`
**Identity:** `CourseId(SchoolId, Uuid)`

### Purpose

An online course published on the school's public site. Carries
title, image, category, overview, outline, prerequisites,
resources, and stats.

### Invariants

1. A `Course` belongs to exactly one school.
2. A `Course::title` is non-empty.
3. A `Course::category_id` references a valid `CourseCategory`.
4. A `Course` is unique by `(school_id, title)`.
5. A `Course::active_status` is a boolean.

### Commands

- `CreateCourse`
- `UpdateCourse`
- `DeleteCourse`
- `PublishCourse`
- `UnpublishCourse`

### Events

- `CourseCreated`
- `CourseUpdated`
- `CourseDeleted`
- `CoursePublished`
- `CourseUnpublished`

---

## CourseCategory

**Root type:** `CourseCategory`
**Identity:** `CourseCategoryId(SchoolId, Uuid)`

### Purpose

A grouping for online courses.

### Invariants

1. A `CourseCategory::category_name` is non-empty.
2. A `CourseCategory` is unique by `(school_id, category_name)`.
3. A `CourseCategory` cannot be deleted if any `Course`
   references it.

### Commands

- `CreateCourseCategory`
- `UpdateCourseCategory`
- `DeleteCourseCategory`

### Events

- `CourseCategoryCreated`
- `CourseCategoryUpdated`
- `CourseCategoryDeleted`

---

## CoursePage

**Root type:** `CoursePage`
**Identity:** `CoursePageId(SchoolId, Uuid)`

### Purpose

A landing-page entry tied to a course, used by the public site
to render course detail pages. May carry additional media and
structured content not present on the base `Course` row.

### Invariants

1. A `CoursePage` references one `Course`.
2. A `CoursePage` is unique per `Course`.

### Commands

- `CreateCoursePage`
- `UpdateCoursePage`
- `DeleteCoursePage`

### Events

- `CoursePageCreated`
- `CoursePageUpdated`
- `CoursePageDeleted`

---

## CustomField

**Root type:** `CustomField`
**Identity:** `CustomFieldId(SchoolId, Uuid)`

### Purpose

A user-defined field on a form (student registration, staff
registration, admission, etc.). Defines the label, type, length
constraints, and whether the field is required.

### Invariants

1. A `CustomField` is unique by `(school_id, form_name, label)`.
2. A `CustomField::type` is one of the supported input types
   (text, number, select, date, file, textarea).
3. A `CustomField::min_max_length` is a string
   `"min..max"` parsed at construction.
4. A `CustomField::min_max_value` is a string
   `"min..max"` parsed at construction.
5. A `CustomField::name_value` is a comma-separated list of
   allowed values for `select` fields.
6. A `CustomField` carries an `academic_id` for academic-year
   scoping.

### Commands

- `CreateCustomField`
- `UpdateCustomField`
- `DeleteCustomField`

### Events

- `CustomFieldCreated`
- `CustomFieldUpdated`
- `CustomFieldDeleted`

---

## CustomFieldValue

**Root type:** `CustomFieldValue`
**Identity:** `CustomFieldValueId(SchoolId, Uuid)`

### Purpose

The value of a `CustomField` for a specific entity (student,
staff, admission, etc.). Decoupled from the field definition so
the field can be updated without rewriting every value.

### Invariants

1. A `CustomFieldValue` references exactly one `CustomField`.
2. A `CustomFieldValue` is unique by
   `(custom_field_id, entity_type, entity_id)`.
3. `entity_type` is one of the closed enum values
   (`Student`, `Staff`, `Admission`, `Course`).
4. `field_value` is the raw string; the engine does not enforce
   type-specific validation here (that is the field's
   responsibility at write time).

### Commands

- `SetCustomFieldValue`
- `ClearCustomFieldValue`

### Events

- `CustomFieldValueSet`
- `CustomFieldValueCleared`

---

## ChartOfAccount

**Root type:** `ChartOfAccount`
**Identity:** `ChartOfAccountId(SchoolId, Uuid)`

### Purpose

The head list for the finance domain. Each head is either an
income or expense account.

### Invariants

1. A `ChartOfAccount::head` is non-empty.
2. A `ChartOfAccount::type` is `Expense` (`E`) or `Income` (`I`).
3. A `ChartOfAccount` is unique by `(school_id, lower(head))`.

### Commands

- `CreateChartOfAccount`
- `UpdateChartOfAccount`
- `DeleteChartOfAccount`

### Events

- `ChartOfAccountCreated`
- `ChartOfAccountUpdated`
- `ChartOfAccountDeleted`

---

## BaseGroup

**Root type:** `BaseGroup`
**Identity:** `BaseGroupId(SchoolId, Uuid)`

### Purpose

A grouping of `BaseSetup` values. Used to organize lookup
tables by category (e.g. "Gender", "Religion", "Caste",
"Occupation").

### Invariants

1. A `BaseGroup::name` is unique within `(school_id, name)`.
2. A `BaseGroup` cannot be deleted if any `BaseSetup` references
   it.

### Commands

- `CreateBaseGroup`
- `UpdateBaseGroup`
- `DeleteBaseGroup`

### Events

- `BaseGroupCreated`
- `BaseGroupUpdated`
- `BaseGroupDeleted`

---

## BaseSetup

**Root type:** `BaseSetup`
**Identity:** `BaseSetupId(SchoolId, Uuid)`

### Purpose

A configurable lookup value. Belongs to a `BaseGroup` and
appears in dropdowns throughout the engine.

### Invariants

1. A `BaseSetup::base_setup_name` is non-empty.
2. A `BaseSetup` references exactly one `BaseGroup`.

### Commands

- `CreateBaseSetup`
- `UpdateBaseSetup`
- `DeleteBaseSetup`

### Events

- `BaseSetupCreated`
- `BaseSetupUpdated`
- `BaseSetupDeleted`

---

## Module

**Root type:** `Module`
**Identity:** `ModuleId(SchoolId, Uuid)`

### Purpose

A top-level functional area of the engine (e.g. `Student
Information`, `Fees Collection`, `Examination`). Each module
hosts a tree of `ModuleLink` entries.

### Invariants

1. A `Module::name` is unique within `(school_id, name)`.
2. A `Module::order` is a non-negative integer.
3. A `Module` cannot be deleted if any `ModuleLink` references
   it.

### Commands

- `CreateModule`
- `UpdateModule`
- `DeleteModule`
- `ReorderModules`

### Events

- `ModuleCreated`
- `ModuleUpdated`
- `ModuleDeleted`

---

## ModuleLink

**Root type:** `ModuleLink`
**Identity:** `ModuleLinkId(SchoolId, Uuid)`

### Purpose

A menu item within a module. A module link is the storage
representation of "this menu item exists in this module for
this school". The RBAC domain binds roles to module links
through `RolePermission`.

### Invariants

1. A `ModuleLink::name` is non-empty.
2. A `ModuleLink::route` is non-empty.
3. A `ModuleLink` references exactly one `Module`.

### Commands

- `CreateModuleLink`
- `UpdateModuleLink`
- `DeleteModuleLink`

### Events

- `ModuleLinkCreated`
- `ModuleLinkUpdated`
- `ModuleLinkDeleted`

---

## AddOn

**Root type:** `AddOn`
**Identity:** `AddOnId(Uuid)` (global, not tenant-scoped)

### Purpose

A registered add-on (a third-party extension point). AddOns
are global across the platform and are activated per school.

### Invariants

1. An `AddOn` has a unique package identifier.
2. An `AddOn` is either installed (active) or not.

### Commands

- `RegisterAddOn` (engine-internal, build-time)
- `InstallAddOn`
- `UninstallAddOn`

### Events

- `AddOnRegistered`
- `AddOnInstalled`
- `AddOnUninstalled`

---

## ModuleManager

**Root type:** `ModuleManager`
**Identity:** `ModuleManagerId(Uuid)` (global)

### Purpose

The metadata for a module manager — the entity that
authenticates module updates, holds the purchase code, the
checksum, and the update URL. Operated by the engine's
integration port, not by school users.

### Invariants

1. A `ModuleManager::email` is RFC-valid.
2. A `ModuleManager::checksum` is the SHA-256 of the installed
   module.
3. A `ModuleManager::is_default` indicates the system default
   manager.

### Commands

- `RegisterModuleManager` (engine-internal)
- `UpdateModuleManager`
- `RotatePurchaseCode`

### Events

- `ModuleManagerRegistered`
- `ModuleManagerUpdated`
- `PurchaseCodeRotated`

---

## ModuleStudentParentInfo

**Root type:** `ModuleStudentParentInfo`
**Identity:** `ModuleStudentParentInfoId(SchoolId, Uuid)`

### Purpose

A per-school configuration of the student/parent menu
visibility. Each row records the modules, menus, and module
name that should appear for student and parent users.

### Invariants

1. A `ModuleStudentParentInfo::module_name` is non-empty.
2. The pair `(school_id, module_name)` is unique.

### Commands

- `ConfigureStudentParentMenu`
- `ResetStudentParentMenu`

### Events

- `StudentParentMenuConfigured`
- `StudentParentMenuReset`

---

## TimeZone

**Root type:** `TimeZone` (global, not tenant-scoped)
**Identity:** `TimeZoneId(Uuid)`

### Purpose

A timezone entry. Seeded once by the engine.

### Invariants

1. A `TimeZone::code` is unique.
2. A `TimeZone::time_zone` is an IANA tz identifier.

### Commands

- `RegisterTimeZone` (engine-internal, build-time)

### Events

- `TimeZoneRegistered`

---

## Country

**Root type:** `Country` (global, not tenant-scoped; the
`platform_countries` table is per-school as a quirk of legacy data,
but the domain model treats it as global)
**Identity:** `CountryId(Uuid)`

### Purpose

A country entry with ISO code, name, native name, phone code,
continent, capital, currency, and languages.

### Invariants

1. A `Country::code` is unique.
2. A `Country::phone` is the country phone prefix.

### Commands

- `RegisterCountry` (engine-internal, build-time)
- `UpdateCountry`

### Events

- `CountryRegistered`
- `CountryUpdated`

---

## Continent

**Root type:** `Continent` (global)
**Identity:** `ContinentId(Uuid)`

### Purpose

A continent entry.

### Invariants

1. A `Continent::code` is unique.
2. A `Continent::name` is non-empty.

### Commands

- `RegisterContinent` (engine-internal, build-time)
- `UpdateContinent`

### Events

- `ContinentRegistered`
- `ContinentUpdated`

---

## Currency

**Root type:** `Currency` (per-school; the storage table is
`platform_currencies` and carries `school_id`; the engine treats the
active currency as a per-school configuration)
**Identity:** `CurrencyId(SchoolId, Uuid)`

### Purpose

A currency with format settings (symbol position, decimal
separator, thousand separator, decimal digits, space flag).

### Invariants

1. A `Currency::code` is unique within `(school_id, code)`.
2. A `Currency::symbol` is non-empty.
3. `decimal_digit` is in `0..=8`.
4. `currency_type` and `currency_position` are encoded values
   whose meanings are documented in the value objects.

### Commands

- `CreateCurrency`
- `UpdateCurrency`
- `DeleteCurrency`

### Events

- `CurrencyCreated`
- `CurrencyUpdated`
- `CurrencyDeleted`

---

## Language

**Root type:** `Language` (per-school)
**Identity:** `LanguageId(SchoolId, Uuid)`

### Purpose

A language entry. May have an RTL flag and a `native` name.

### Invariants

1. A `Language::code` is unique within `(school_id, code)`.
2. `rtl` is a boolean.

### Commands

- `CreateLanguage`
- `UpdateLanguage`
- `DeleteLanguage`

### Events

- `LanguageCreated`
- `LanguageUpdated`
- `LanguageDeleted`

---

## SocialMediaIcon

**Root type:** `SocialMediaIcon`
**Identity:** `SocialMediaIconId(SchoolId, Uuid)`

### Purpose

A social media link with an icon (Facebook, Twitter, etc.).

### Invariants

1. A `SocialMediaIcon::url` is a valid URL.
2. A `SocialMediaIcon::icon` is non-empty.
3. `status` is `Active` or `Inactive`.

### Commands

- `CreateSocialMediaIcon`
- `UpdateSocialMediaIcon`
- `DeleteSocialMediaIcon`

### Events

- `SocialMediaIconCreated`
- `SocialMediaIconUpdated`
- `SocialMediaIconDeleted`

---

## HeaderMenuManager

**Root type:** `HeaderMenuManager`
**Identity:** `HeaderMenuManagerId(SchoolId, Uuid)`

### Purpose

A public-header menu entry. May be of any `Type` (link, page,
custom), may have a parent, and may open in a new tab.

### Invariants

1. A `HeaderMenuManager::type` is one of the supported menu
   element types.
2. `position` is a non-negative integer.
3. `is_newtab` is a boolean.
4. `theme` is a non-empty theme name.
5. `show` is a boolean.

### Commands

- `CreateHeaderMenuItem`
- `UpdateHeaderMenuItem`
- `DeleteHeaderMenuItem`
- `ReorderHeaderMenu`

### Events

- `HeaderMenuItemCreated`
- `HeaderMenuItemUpdated`
- `HeaderMenuItemDeleted`

---

## PhotoGallery

**Root type:** `PhotoGallery`
**Identity:** `PhotoGalleryId(SchoolId, Uuid)`

### Purpose

A photo gallery (a folder of images) for the public site.

### Invariants

1. A `PhotoGallery::name` is non-empty.
2. A `PhotoGallery` may have a parent (sub-galleries).
3. `is_publish` is a boolean.

### Commands

- `CreatePhotoGallery`
- `UpdatePhotoGallery`
- `DeletePhotoGallery`
- `PublishPhotoGallery`
- `UnpublishPhotoGallery`

### Events

- `PhotoGalleryCreated`
- `PhotoGalleryUpdated`
- `PhotoGalleryDeleted`
- `PhotoGalleryPublished`
- `PhotoGalleryUnpublished`

---

## VideoGallery

**Root type:** `VideoGallery`
**Identity:** `VideoGalleryId(SchoolId, Uuid)`

### Purpose

A video gallery entry for the public site. Carries a video
link and a description.

### Invariants

1. A `VideoGallery::name` is non-empty.
2. A `VideoGallery::video_link` is a valid URL.
3. `is_publish` is a boolean.

### Commands

- `CreateVideoGallery`
- `UpdateVideoGallery`
- `DeleteVideoGallery`
- `PublishVideoGallery`
- `UnpublishVideoGallery`

### Events

- `VideoGalleryCreated`
- `VideoGalleryUpdated`
- `VideoGalleryDeleted`
- `VideoGalleryPublished`
- `VideoGalleryUnpublished`

---

## Visitor

**Root type:** `Visitor`
**Identity:** `VisitorId(SchoolId, Uuid)`

### Purpose

A visitor log entry. Records name, phone, purpose, in-time,
out-time, and an optional attached file.

### Invariants

1. A `Visitor::name` is non-empty.
2. A `Visitor::date` is a valid date.
3. If both `in_time` and `out_time` are set, `in_time <= out_time`.

### Commands

- `RecordVisitor`
- `UpdateVisitor`
- `DeleteVisitor`

### Events

- `VisitorRecorded`
- `VisitorUpdated`
- `VisitorDeleted`

---

## ToDo

**Root type:** `ToDo`
**Identity:** `ToDoId(SchoolId, Uuid)`

### Purpose

A to-do item with a status (`Complete`, `NotComplete`, or
`Pending`).

### Invariants

1. A `ToDo::todo_title` is non-empty.
2. `complete_status` is `C` (complete), `N` (not complete), or
   `P` (pending).

### Commands

- `CreateToDo`
- `UpdateToDo`
- `MarkToDoComplete`
- `DeleteToDo`

### Events

- `ToDoCreated`
- `ToDoUpdated`
- `ToDoCompleted`
- `ToDoDeleted`

---

## Instruction

**Root type:** `Instruction`
**Identity:** `InstructionId(SchoolId, Uuid)`

### Purpose

A front-office instruction. Carries a title and a description.

### Invariants

1. An `Instruction::title` is non-empty.
2. An `Instruction::description` is non-empty.
3. `active_status` is a boolean.

### Commands

- `CreateInstruction`
- `UpdateInstruction`
- `DeleteInstruction`

### Events

- `InstructionCreated`
- `InstructionUpdated`
- `InstructionDeleted`

---

## ExpertTeacher

**Root type:** `ExpertTeacher`
**Identity:** `ExpertTeacherId(SchoolId, Uuid)`

### Purpose

A featured staff member displayed on the public site. The
underlying staff record is owned by the HR domain; the
`ExpertTeacher` row is a pointer plus a `position` ordering.

### Invariants

1. An `ExpertTeacher::staff_id` references a valid staff
   record.
2. A pair `(school_id, staff_id)` is unique.
3. `position` is a non-negative integer.

### Commands

- `MarkExpertTeacher`
- `UnmarkExpertTeacher`
- `ReorderExpertTeachers`

### Events

- `ExpertTeacherMarked`
- `ExpertTeacherUnmarked`

---

## FrontendPermission

**Root type:** `FrontendPermission`
**Identity:** `FrontendPermissionId(SchoolId, Uuid)`

### Purpose

A public-facing permission flag. Controls whether a public
page is published.

### Invariants

1. A `FrontendPermission::name` is non-empty.
2. `is_published` is a boolean.

### Commands

- `CreateFrontendPermission`
- `UpdateFrontendPermission`
- `DeleteFrontendPermission`
- `PublishFrontendPermission`
- `UnpublishFrontendPermission`

### Events

- `FrontendPermissionCreated`
- `FrontendPermissionUpdated`
- `FrontendPermissionDeleted`
- `FrontendPermissionPublished`
- `FrontendPermissionUnpublished`

---

## AmountTransfer

**Root type:** `AmountTransfer`
**Identity:** `AmountTransferId(SchoolId, Uuid)`

### Purpose

A fund transfer between two accounts (cash to bank, bank to
cash, etc.). The actual movement of money is handled by the
payment port; the `AmountTransfer` row records the operator's
declaration.

### Invariants

1. An `AmountTransfer::amount` is positive.
2. `from_payment_method != to_payment_method` or
   `from_bank_name != to_bank_name`.
3. `transfer_date` is a valid date.

### Commands

- `CreateAmountTransfer`
- `UpdateAmountTransfer`
- `DeleteAmountTransfer`

### Events

- `AmountTransferCreated`
- `AmountTransferUpdated`
- `AmountTransferDeleted`

---

## Plugin

**Root type:** `Plugin`
**Identity:** `PluginId(SchoolId, Uuid)`

### Purpose

A registered front-office plugin (e.g. Google Analytics,
chat widget, SEO plugin). Plugins have a short code, an
availability scope (admin/website/both), a position, and a
showing page.

### Invariants

1. A `Plugin::name` is non-empty.
2. A `Plugin::short_code` is unique within `(school_id,
   short_code)`.
3. `availability` is `Admin`, `Website`, or `Both`.
4. `is_enable`, `show_admin_panel`, `show_website` are
   booleans.

### Commands

- `EnablePlugin`
- `DisablePlugin`
- `UpdatePlugin`

### Events

- `PluginEnabled`
- `PluginDisabled`
- `PluginUpdated`

---

## Comment

**Root type:** `Comment`
**Identity:** `CommentId(SchoolId, Uuid)`

### Purpose

A free-text comment with optional flagged status, attached to
an academic year and tenant.

### Invariants

1. A `Comment::text` is non-empty.
2. `is_flagged` is a boolean.
3. `type` is a free-form string (e.g. `note`, `feedback`).

### Commands

- `CreateComment`
- `UpdateComment`
- `FlagComment`
- `DeleteComment`

### Events

- `CommentCreated`
- `CommentUpdated`
- `CommentFlagged`
- `CommentDeleted`

---

## CommentTag

**Root type:** `CommentTag`
**Identity:** `CommentTagId(SchoolId, Uuid)`

### Purpose

A tag value used to label comments. Tags are global in the
sense that the engine does not enforce tenant uniqueness; the
underlying table is the storage row.

### Invariants

1. A `CommentTag::tag` is unique.
2. A `CommentTag::tag` is non-empty.

### Commands

- `CreateCommentTag`
- `DeleteCommentTag`

### Events

- `CommentTagCreated`
- `CommentTagDeleted`

---

## CommentPivot

**Root type:** `CommentPivot` (a join entity, not a root)
**Identity:** `(CommentId, CommentTagId)`

### Purpose

The many-to-many join between `Comment` and `CommentTag`.

### Invariants

1. A `CommentPivot` row exists only when both endpoints exist.
2. The pair `(comment_id, comment_tag_id)` is unique.

### Commands

- (None — managed through `CreateComment` and `UpdateComment`
  which accept a list of tag ids.)

### Events

- (Subscribed from `CommentCreated` / `CommentUpdated`.)

---

## PersonalAccessToken

**Root type:** `PersonalAccessToken`
**Identity:** `PersonalAccessTokenId(Uuid)` (global, owned by
the `User` it belongs to)

### Purpose

A personal access token (PAT) used by API consumers. The
engine does not implement OAuth flows; PATs are the simple
"bearer token" mechanism. Carries the hashed token, the
abilities, and the expiry.

### Invariants

1. A `PersonalAccessToken::token` is unique and stored as a
   SHA-256 hash.
2. A `PersonalAccessToken::tokenable_type` is the owning
   aggregate type name.
3. `expires_at` is optional; if set, the token is rejected
   after that timestamp.
4. The abilities list is non-empty.

### Commands

- `IssuePersonalAccessToken`
- `RevokePersonalAccessToken`
- `ExpirePersonalAccessToken` (system)

### Events

- `PersonalAccessTokenIssued`
- `PersonalAccessTokenRevoked`
- `PersonalAccessTokenExpired`

---

## VideoUpload

**Root type:** `VideoUpload`
**Identity:** `VideoUploadId(SchoolId, Uuid)`

### Purpose

A class-section video upload, typically a YouTube link tied
to a teacher's lesson for a class/section.

### Invariants

1. A `VideoUpload::title` is non-empty.
2. A `VideoUpload::youtube_link` is a valid YouTube URL.
3. `class_id` and `section_id` reference valid class/section
   ids in the academic domain.

### Commands

- `UploadVideo`
- `UpdateVideo`
- `DeleteVideo`

### Events

- `VideoUploaded`
- `VideoUpdated`
- `VideoDeleted`
