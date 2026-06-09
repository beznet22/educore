# Platform Domain — Entities

Entities have identity and lifecycle but are not aggregate roots. They
are loaded and persisted only through their aggregate root.

## SchoolContact

**Identity:** `SchoolContactId(SchoolId, Uuid)`
**Owner:** `School`

A point of contact for a school — name, role, phone, email. The
underlying school row carries the primary phone/email; a school may
have many contact persons (e.g. principal, accountant, receptionist).

## SchoolPackage

**Identity:** `SchoolPackageId(Uuid)` (global catalog)
**Owner:** `School` (binding)

A package plan that the school subscribes to. Defines student
capacity, staff capacity, storage, and feature flags. The platform
domain records the binding (`package_id` on the school row); the
package catalog itself is global.

## SchoolRegion

**Identity:** `SchoolRegionId(SchoolId, Uuid)`
**Owner:** `School`

The school's region reference. A thin pointer to a `Continent` or
`Country` id; the school row also carries an `int` `region` field
that is a legacy compatibility value.

## UserSession

**Identity:** `UserSessionId(SchoolId, Uuid)`
**Owner:** `User`

An active session for a user. Stores the session id (hashed), the
ip, the user agent, the issued-at, and the expires-at. Sessions
are invalidated on `UserDeactivated`.

## UserPreference

**Identity:** `UserPreferenceId(SchoolId, Uuid)`
**Owner:** `User`

A per-user preference record: `language`, `style_id`, `rtl_ltl`,
`selected_session`, `notification_token`, `device_token`. The base
`User` row also carries these fields directly; the
`UserPreference` entity is the storage for additional
per-user settings that the base row does not hold (e.g. timezone
override, custom dashboard layout).

## UserLogin

**Identity:** `UserLoginId(SchoolId, Uuid)`
**Owner:** `User`

A historical login record (one row per successful login). The
operations domain subscribes to `UserLogged` to produce the audit
log; this entity is a denormalized read-model.

## UserDocument

**Identity:** `UserDocumentId(SchoolId, Uuid)`
**Owner:** `User`

A document uploaded against a user (e.g. national id, contract).

## OtpDelivery

**Identity:** `OtpDeliveryId(SchoolId, Uuid)`
**Owner:** `OtpCode`

A single delivery attempt of an OTP. Stores the channel
(`Sms` or `Email`), the recipient user, the issued timestamp,
the expiry timestamp, the hashed code, and a `consumed` flag.

## CourseInstructor

**Identity:** `CourseInstructorId(SchoolId, Uuid)`
**Owner:** `Course`

A staff member assigned to teach a course. The HR domain owns
the staff record; the platform domain records the assignment.

## CourseMaterial

**Identity:** `CourseMaterialId(SchoolId, Uuid)`
**Owner:** `Course`

A learning material attached to a course (file reference, link,
text).

## CourseEnrollment

**Identity:** `CourseEnrollmentId(SchoolId, Uuid)`
**Owner:** `Course`

A user enrolled in a course. May carry a status and a progress
percentage.

## CourseReview

**Identity:** `CourseReviewId(SchoolId, Uuid)`
**Owner:** `Course`

A user-submitted review of a course, with rating and text.

## CustomFieldOption

**Identity:** `CustomFieldOptionId(SchoolId, Uuid)`
**Owner:** `CustomField`

An option for a `CustomField` of type `select` or `radio`. The
field's `name_value` is a comma-separated list; the
`CustomFieldOption` entity is the typed projection.

## CustomFieldValidation

**Identity:** `CustomFieldValidationId(SchoolId, Uuid)`
**Owner:** `CustomField`

A validation rule attached to a `CustomField` (regex pattern,
required flag, length range, numeric range). The base field row
carries the simpler `min_max_length` and `min_max_value` strings;
the entity is the structured projection.

## ChartOfAccountBalance

**Identity:** `ChartOfAccountBalanceId(SchoolId, Uuid)`
**Owner:** `ChartOfAccount`

A cached running balance for a chart-of-account head. Updated
asynchronously by the finance domain's event subscriber.

## BaseSetupTranslation

**Identity:** `BaseSetupTranslationId(SchoolId, Uuid)`
**Owner:** `BaseSetup`

A per-locale translation of a `BaseSetup` value's display label.

## ModuleLinkPermission

**Identity:** `ModuleLinkPermissionId(SchoolId, Uuid)`
**Owner:** `ModuleLink`

A legacy compatibility row. The RBAC domain's `RolePermission`
is the canonical binding; `ModuleLinkPermission` is a denormalized
projection kept for fast menu rendering.

## ModuleLinkChild

**Identity:** `ModuleLinkChildId(SchoolId, Uuid)`
**Owner:** `ModuleLink`

A sub-action of a `ModuleLink` (e.g. "view", "create", "edit",
"delete"). Used by the RBAC domain to compute fine-grained
permissions within a single link.

## ModuleLinkRoute

**Identity:** `ModuleLinkRouteId(SchoolId, Uuid)`
**Owner:** `ModuleLink`

A named route for a `ModuleLink`. The base `ModuleLink::route` is
the primary route; `ModuleLinkRoute` is the set of sub-routes
(e.g. `student.create` for the `student` link).

## AddOnManifest

**Identity:** `AddOnManifestId(Uuid)` (global)
**Owner:** `AddOn`

A typed manifest of an add-on: name, version, dependencies,
permissions it requires, hooks it registers. Stored in
storage as a JSON blob but modeled as a typed entity.

## AddOnInstallation

**Identity:** `AddOnInstallationId(SchoolId, Uuid)`
**Owner:** `AddOn`

A per-school installation record. Carries the installed version,
the installation date, the license key (if any), and the
enabled/disabled flag.

## ModuleManagerEndpoint

**Identity:** `ModuleManagerEndpointId(Uuid)` (global)
**Owner:** `ModuleManager`

A registered endpoint of a module manager (e.g. update URL,
purchase code validator, checksum verifier). The base row
carries one URL; the entity is the set of endpoints with their
typed roles.

## VisitorAttachment

**Identity:** `VisitorAttachmentId(SchoolId, Uuid)`
**Owner:** `Visitor`

A file attached to a visitor log entry (e.g. id scan).

## ToDoAssignee

**Identity:** `ToDoAssigneeId(SchoolId, Uuid)`
**Owner:** `ToDo`

A user assigned to a to-do. A to-do may have multiple assignees.

## ToDoComment

**Identity:** `ToDoCommentId(SchoolId, Uuid)`
**Owner:** `ToDo`

A comment thread on a to-do.

## InstructionAttachment

**Identity:** `InstructionAttachmentId(SchoolId, Uuid)`
**Owner:** `Instruction`

A file attached to a front-office instruction.

## FrontendPermissionOverride

**Identity:** `FrontendPermissionOverrideId(SchoolId, Uuid)`
**Owner:** `FrontendPermission`

A per-role override of a frontend permission. Allows a specific
role to see (or not see) a public page even if the default policy
says otherwise.

## AmountTransferAttachment

**Identity:** `AmountTransferAttachmentId(SchoolId, Uuid)`
**Owner:** `AmountTransfer`

A receipt or supporting document attached to an amount transfer.

## AmountTransferReversal

**Identity:** `AmountTransferReversalId(SchoolId, Uuid)`
**Owner:** `AmountTransfer`

A reversal record. The engine does not allow negative amounts; a
reversal is a new transfer that offsets the original. The
`AmountTransferReversal` entity is the explicit link.

## PluginConfig

**Identity:** `PluginConfigId(SchoolId, Uuid)`
**Owner:** `Plugin`

A typed configuration value for a plugin (key, value, type).

## PluginHook

**Identity:** `PluginHookId(SchoolId, Uuid)`
**Owner:** `Plugin`

A hook registered by a plugin. The engine's integration port
exposes a fixed set of hook names; `PluginHook` is the
binding from plugin to hook with handler metadata.

## CommentMention

**Identity:** `CommentMentionId(SchoolId, Uuid)`
**Owner:** `Comment`

A user mentioned in a comment. The platform sends a notification
when a mention is created.

## PersonalAccessTokenLastUsed

**Identity:** `PersonalAccessTokenLastUsedId(Uuid)` (global)
**Owner:** `PersonalAccessToken`

A denormalized last-used timestamp projection. Updated
asynchronously.

## VideoUploadChapter

**Identity:** `VideoUploadChapterId(SchoolId, Uuid)`
**Owner:** `VideoUpload`

A chapter marker on a video upload (timestamp, title).

## VideoUploadView

**Identity:** `VideoUploadViewId(SchoolId, Uuid)`
**Owner:** `VideoUpload`

A per-user view record (who watched, when, how much).

## ModuleInfo

**Identity:** `ModuleInfoId(SchoolId, Uuid)`
**Owner:** `Module` (logical; used by RBAC to map module ids
to their display info)

A denormalized module info row carrying the module's display
metadata (icon, lang name, route, parent route, type). The
`type` field discriminates `module` (1), `module_link` (2),
`module_link_crud` (3). This aggregate replaces the legacy
`InfixModuleInfo` brand artifact.

## ModuleManager

**Identity:** `ModuleManagerId(Uuid)` (global)
**Owner:** `ModuleManager`

A module-manager row carrying the platform's per-tenant module
configuration (`is_default`, `addon_url`, `lang_type`). This
aggregate replaces the legacy `InfixModuleManager` brand artifact
and lives in the `platform` domain, not as a SaaS-scoped shadow.
