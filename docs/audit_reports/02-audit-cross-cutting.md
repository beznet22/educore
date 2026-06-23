# 02 - Audit Appendix - Cross-cutting (7 crates)

**Scope:** wave2-platform.md, wave2-rbac.md, wave2-operations.md, wave2-settings.md, wave2-events.md, wave2-audit.md, wave2-sync.md

**Total findings:** 253

**Severity distribution:** 51 critical, 93 high, 78 medium, 31 low


## Summary Table

| Target | Critical | High | Medium | Low | Total |
| --- | --- | --- | --- | --- | --- |
| Platform (`CROSSCUT-PLAT`) | 15 | 14 | 14 | 5 | 48 |
| RBAC (`CROSSCUT-RBAC`) | 7 | 18 | 9 | 2 | 36 |
| Operations (`CROSSCUT-OPS`) | 5 | 23 | 17 | 11 | 56 |
| Settings (`CROSSCUT-SET`) | 0 | 10 | 12 | 6 | 28 |
| Events (envelope) (`CC-EVT`) | 6 | 6 | 12 | 4 | 28 |
| Audit (cross-cutting) (`CC-AUD`) | 8 | 12 | 8 | 2 | 30 |
| Sync (cross-cutting) (`CC-SYNC`) | 10 | 10 | 6 | 1 | 27 |

## Platform (target id prefix: `CROSSCUT-PLAT`)

**Path:** `crates/cross-cutting/platform/`  
**Total findings:** 48 (15 critical, 14 high, 14 medium, 5 low)


### FINDING 1 (id: `CROSSCUT-PLAT-001`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Critical
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/aggregate.rs:46-91

**Description:**

The `School` aggregate is missing four fields
  mandated by `docs/specs/platform/aggregates.md` invariants 4,
  6, 7, 9, 10: `email`, `is_enabled`, `plan_type`,
  `starting_date`, `ending_date`, and `region`. The struct
  cannot satisfy invariants 4 (`School::email is RFC-valid`),
  6 (`is_enabled is Yes or No`), 7 (`plan_type carries the
  package's billing mode`), 9 (`starting_date <= ending_date`),
  or 10 (`region references a known continent/country id`).

**Expected:**

`docs/specs/platform/aggregates.md` lines 36-46
  list 10 invariants for `School`; lines 17-34 of the spec's
  `CreateSchoolCommand` carry `email`, `phone`, `address`,
  `starting_date`, `ending_date`, `plan_type`, `contact_type`,
  `region` — all of which need a slot on the aggregate.

**Evidence:**

`crates/cross-cutting/platform/src/aggregate.rs:46-91` —
  ```rust
  pub struct School {
      pub id: SchoolId,
      pub name: String,
      pub domain: Option<String>,
      pub school_code: String,
      pub status: SchoolStatus,
      pub package_id: Option<PackageId>,
      pub version: Version,
      pub etag: Etag,
      pub created_at: Timestamp,
      pub updated_at: Timestamp,
      pub created_by: UserId,
      pub updated_by: UserId,
      pub active_status: ActiveStatus,
      pub last_event_id: Option<EventId>,
      pub correlation_id: CorrelationId,
  }
  ```
  No `email`, `is_enabled`, `plan_type`, `starting_date`,
  `ending_date`, `region`, `phone`, `address`, or `contact_type`
  field exists. The grep for `email|is_enabled|plan_type|
  starting_date|ending_date|region` on `aggregate.rs` returns
  zero rows.

---

### FINDING 10 (id: `CROSSCUT-PLAT-010`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Critical
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/events.rs (all 6 events)

**Description:**

None of the 6 implemented events have
  subscribers wired in (RBAC, settings, academic, operations,
  communication, cms). The events spec lists subscribers per
  event; spec drift between code and docs is severe.

**Expected:**

`docs/specs/platform/events.md:50-53` —
  `SchoolCreated` subscribers `rbac`, `settings`, `academic`;
  line 115-117 — `UserRegistered` subscriber `rbac`;
  line 137-138 — `UserDeactivated` subscribers `rbac`,
  `operations`.

**Evidence:**

`crates/cross-cutting/platform/src/events.rs`
  contains only struct definitions and `DomainEvent` trait
  impls (`grep -rn "fn subscriber\|impl Subscriber" events.rs`
  returns no rows). There is no subscriber module at all.

---

### FINDING 11 (id: `CROSSCUT-PLAT-011`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Critical
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/value_objects.rs (full file)

**Description:**

Of the ~100 value objects documented in
  `docs/specs/platform/value-objects.md`, only 7 are
  implemented in the platform crate (`EmailAddress`,
  `PhoneNumber`, `HashedPassword`, `SchoolStatus`,
  `UserStatus`, `PackageId`, `RoleId`). The spec also defines
  `SchoolName`, `SchoolCode`, `Domain`, `Address`,
  `PersonName`, `FullName`, `Username`, `SchoolActiveStatus`,
  `LoginEnabled`, `ContactType`, `PlanType`, `Region`,
  `IsAdministrator`, `IsRegistered`, `AccessStatus`,
  `LanguagePreference`, `StylePreference`, `RtlPreference`,
  `SelectedSessionId`, `RandomCode`, `NotificationToken`,
  `DeviceToken`, `RememberToken`, `WalletBalance`, `Verified`,
  `TrialEndsAt`, `StripeId`, `CardBrand`, `CardLastFour`,
  `OtpCode`, `OtpExpiry`, `OtpChannel`, `OtpDeliveryMode`,
  `CourseTitle`, `CourseStatus`, `CourseImage`, `CourseOverview`,
  `CourseOutline`, `Prerequisites`, `Resources`, `Stats`,
  `FormName`, `FieldLabel`, `FieldType`, `LengthRange`,
  `ValueRange`, `NameValueList`, `Width`, `IsRequired`,
  `EntityType`, `FieldValue`, `AccountHead`, `AccountType`,
  `BaseGroupName`, `BaseSetupName`, `ModuleName`, `ModuleOrder`,
  `ModuleRoute`, `ParentRoute`, `LangName`, `IconClass`,
  `ModuleInfoType`, `PackageName`, `UpdateUrl`, `PurchaseCode`,
  `Checksum`, `InstalledDomain`, `AddonUrl`, `ActivatedDate`,
  `LangType`, `CountryCode`, `CountryName`, `CountryNative`,
  `PhoneCode`, `ContinentCode`, `ContinentName`, `CurrencyCode`,
  `CurrencyName`, `CurrencySymbol`, `CurrencyType`,
  `CurrencyPosition`, `DecimalDigit`, `DecimalSeparator`,
  `ThousandSeparator`, `SpaceBetween`, `LanguageCode`,
  `LanguageName`, `LanguageNative`, `LanguageUniversal`,
  `RtlFlag`, `TimeZoneCode`, `TimeZoneIana`, `VisitorName`,
  `VisitorId`, `NoOfPerson`, `Purpose`, `InTime`, `OutTime`,
  `VisitorFile`, `ToDoTitle`, `ToDoStatus`, `ToDoDate`,
  `Amount`, `TransferPurpose`, `PaymentMethod`, `BankName`,
  `TransferDate`, `FrontendPermissionName`, `IsPublished`,
  `TokenName`, `TokenHash`, `TokenableType`, `Abilities`,
  `LastUsedAt`, `ExpiresAt`, etc.

**Expected:**

`docs/specs/platform/value-objects.md` lines
  11-279 enumerate all value objects.

**Evidence:**

`crates/cross-cutting/platform/src/value_objects.rs`
  defines only 7 (`grep -E "^pub (struct|enum)" value_objects.rs`
  returns `EmailAddress`, `PhoneNumber`, `HashedPassword`,
  `SchoolStatus`, `UserStatus`, `PackageId`, `RoleId`).

---

### FINDING 12 (id: `CROSSCUT-PLAT-012`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Critical
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/services.rs (full file)

**Description:**

The spec defines 17 `XService` structs
  (`SchoolService`, `UserService`, `OtpService`,
  `CourseService`, `CustomFieldService`, `ChartOfAccountService`,
  `BaseSetupService`, `ModuleService`, `AddOnService`,
  `LocaleService`, `HeaderMenuService`, `VisitorService`,
  `ToDoService`, `AmountTransferService`, `PluginService`,
  `CommentService`, `PersonalAccessTokenService`) and 2
  policies (`UniqueSchoolFields`, `UserUniquenessInSchool`)
  and 2 specifications (`ActiveUsers`, `PublishedCourses`).
  None of these exist as Rust structs. The crate only has
  free factory functions (`create_school`, `update_school`,
  etc.).

**Expected:**

`docs/specs/platform/services.md` lines 8-258
  define 17 service structs plus 2 policies plus 2
  specifications.

**Evidence:**

`crates/cross-cutting/platform/src/services.rs`
  contains only 6 free functions (`create_school`,
  `update_school`, `deactivate_school`, `register_user`,
  `update_user`, `deactivate_user`). `grep -nE "^pub struct" services.rs`
  returns no rows. The service contract from the spec is not
  satisfied.

---

### FINDING 13 (id: `CROSSCUT-PLAT-013`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Critical
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/repository.rs (full file)

**Description:**

The spec defines 28 repository port
  traits. Only 2 are implemented (`SchoolRepository`,
  `UserRepository`). Missing: `OtpCodeRepository`,
  `CourseRepository`, `CourseCategoryRepository`,
  `CoursePageRepository`, `CustomFieldRepository`,
  `CustomFieldValueRepository`, `ChartOfAccountRepository`,
  `BaseGroupRepository`, `BaseSetupRepository`,
  `ModuleRepository`, `ModuleLinkRepository`,
  `AddOnRepository`, `ModuleManagerRepository`,
  `ModuleStudentParentInfoRepository`, `TimeZoneRepository`,
  `CountryRepository`, `ContinentRepository`,
  `CurrencyRepository`, `LanguageRepository`,
  `SocialMediaIconRepository`, `HeaderMenuManagerRepository`,
  `PhotoGalleryRepository`, `VideoGalleryRepository`,
  `VisitorRepository`, `ToDoRepository`,
  `InstructionRepository`, `ExpertTeacherRepository`,
  `FrontendPermissionRepository`, `AmountTransferRepository`,
  `PluginRepository`, `CommentRepository`,
  `CommentTagRepository`, `PersonalAccessTokenRepository`,
  `VideoUploadRepository`.

**Expected:**

`docs/specs/platform/repositories.md` lines
  7-296 define 28 repository port traits (line 279 lists 16
  additional traits following the standard CRUD pattern).

**Evidence:**

`crates/cross-cutting/platform/src/repository.rs`
  defines only 2 traits (`grep -nE "^pub trait" repository.rs`
  returns `SchoolRepository` line 32 and `UserRepository`
  line 71).

---

### FINDING 19 (id: `CROSSCUT-PLAT-019`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Critical
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/services.rs:189-198

**Description:**

`update_school` computes `bytes` and
  `s_id` but never uses them; the event_id is minted via
  `Uuid::now_v7()` instead. Dead code in a non-test function.
  The surrounding comment is misleading ("the bus port stamps
  its own event id at publish time, so this is informational
  only") but the bytes copy is never observed by anything.

**Expected:**

No dead code in production paths.

**Evidence:**

`crates/cross-cutting/platform/src/services.rs:193-198`:
  ```rust
  let event_id = {
      let mut bytes = [0u8; 16];
      let s_id = school.id.as_uuid();
      bytes.copy_from_slice(s_id.as_bytes());
      educore_core::ids::EventId::from_uuid(uuid::Uuid::now_v7())
  };
  ```
  The `bytes` array and `s_id` variable are computed but
  discarded; `Uuid::now_v7()` is what produces the event_id.
  `bytes` does not appear in the returned `SchoolUpdated`
  payload (lines 201-210).

---

### FINDING 2 (id: `CROSSCUT-PLAT-002`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Critical
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/aggregate.rs:142-194

**Description:**

The `User` aggregate is missing four fields
  mandated by `docs/specs/platform/aggregates.md` invariants 7,
  9, 10 and the Purpose statement (line 84-86) covering
  authentication material: `is_administrator`, `language`,
  `is_registered`, `random_code`, `notification_token`,
  `device_token`, `remember_token`. The struct can carry the
  `role_ids` binding but no `role_id` for the primary
  single-role invariant (invariant 8 of the spec).

**Expected:**

`docs/specs/platform/aggregates.md` lines 94-110
  specify 11 invariants for `User`; lines 84-86 state "Holds
  the user's identity, contact information, status, language
  preference, role binding, and authentication-related fields
  (random code, notification token, remember token, OTP)."

**Evidence:**

`crates/cross-cutting/platform/src/aggregate.rs:142-194` —
  ```rust
  pub struct User {
      pub id: UserId,
      pub school_id: SchoolId,
      pub email: EmailAddress,
      pub username: String,
      pub phone_number: Option<PhoneNumber>,
      pub display_name: String,
      pub usertype: UserType,
      pub role_ids: Vec<RoleId>,
      pub status: UserStatus,
      pub password_hash: HashedPassword,
      pub version: Version,
      pub etag: Etag,
      ...
  }
  ```
  No `is_administrator`, `language`/`LanguageCode`,
  `is_registered`, `random_code`, `notification_token`,
  `device_token`, or `remember_token` field is present. The
  grep for `is_administrator|is_registered|language|
  LanguageCode|RandomCode|NotificationToken|DeviceToken|
  RememberToken` on the platform crate returns only one doc
  reference in `entities.rs:89` (a mention inside a comment).

---

### FINDING 25 (id: `CROSSCUT-PLAT-025`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Critical
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/entities.rs (full file)

**Description:**

`UserSession` lacks a typed session-id
  shape — the spec mandates a hashed session id, but the
  struct carries `session_id: SessionId` (a `Uuid` wrapper).
  Spec invariants 4 (`token is unique and stored as a
  SHA-256 hash`) apply to `PersonalAccessToken` and similar
  properties are expected for sessions (per `entities.md:38`:
  "the opaque session id (hashed; the plaintext is never
  stored)"). The engine's `SessionId` is a `Uuid` newtype and
  is not a hash.

**Expected:**

`docs/specs/platform/entities.md:38-42`
  describes hashed session ids.

**Evidence:**

`crates/cross-cutting/platform/src/entities.rs:62-84`:
  ```rust
  pub struct UserSession {
      pub id: Uuid,
      pub school_id: SchoolId,
      pub user_id: UserId,
      pub session_id: SessionId,
      ...
  }
  ```
  `SessionId` is `educore_core::ids::SessionId`, a `Uuid`
  newtype (not a SHA-256 hash). The hashing of the session
  id is implicit and not modeled by the type system.

---

### FINDING 26 (id: `CROSSCUT-PLAT-026`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Critical
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/tables.md (no impl)

**Description:**

Of the 43 tables listed in
  `docs/specs/platform/tables.md`, only 2 are backed by
  Rust aggregate structs (`platform_schools`,
  `platform_users`). The remaining 41 tables
  (`platform_module_infos`, `platform_module_managers`,
  `platform_add_ons`, `platform_amount_transfers`,
  `platform_base_groups`, `platform_chart_of_accounts`,
  `platform_countries`, `platform_courses`,
  `platform_course_categories`, `platform_currencies`,
  `platform_custom_fields`, `platform_custom_field_values`,
  `platform_expert_teachers`, `platform_frontend_permissions`,
  `platform_header_menu_managers`, `platform_instructions`,
  `platform_modules`, `platform_module_links`,
  `platform_student_parent_menus`, `platform_photo_galleries`,
  `platform_schools`, `platform_social_media_icons`,
  `platform_time_zones`, `platform_to_dos`,
  `platform_video_galleries`, `platform_visitors`,
  `platform_users`, `comments`, `comment_pivots`,
  `comment_tags`, `continents`, `continents_typo_legacy` (drop),
  `countries`, `languages`, `personal_access_tokens`,
  `plugins`, `school_modules`, `user_otp_codes`,
  `video_uploads`) have no Rust aggregate struct.

**Expected:**

`docs/specs/platform/tables.md` lines 7-48
  enumerate 43 tables (cross-cutting and self-reference rows
  included; the spec lists 43 distinct table rows).

**Evidence:**

The `aggregate.rs` file has only 2 structs;
  no `#[derive(DomainQuery)]` attribute is present in the
  platform crate (`grep -rn "derive(DomainQuery)" crates/cross-cutting/platform/`
  returns no rows). No typed entities back the 41
  non-School/User tables.

---

### FINDING 4 (id: `CROSSCUT-PLAT-004`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Critical
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/aggregate.rs:46-91

**Description:**

`School::active_status` is typed
  `ActiveStatus` (a unified engine type with variants like
  `Active` / `Retired`) rather than the spec's `SchoolActiveStatus`
  (`Approved` | `Pending`). Spec invariant 5 (`School::active_status
  is Approved or Pending`) is therefore not expressible by the
  type system, and spec invariant 8 (bootstrap school cannot be
  deleted) has no guard.

**Expected:**

`docs/specs/platform/aggregates.md:40-43` —
  `5. A School::active_status is Approved or Pending.`;
  `8. The bootstrap School (id 1) cannot be deleted.`

**Evidence:**

`crates/cross-cutting/platform/src/aggregate.rs:46-91` —
  `pub active_status: ActiveStatus,` (engine-wide type, not the
  domain-specific `SchoolActiveStatus` value object). No
  `SchoolActiveStatus` type is defined anywhere in
  `crates/cross-cutting/platform/src/value_objects.rs`
  (`grep SchoolActiveStatus value_objects.rs` returns no rows).

---

### FINDING 5 (id: `CROSSCUT-PLAT-005`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Critical
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/aggregate.rs:46-91

**Description:**

Spec invariant 8 mandates "The bootstrap
  School (id 1) cannot be deleted." The `School` struct has
  no protection: `DeactivateSchoolCommand` does not check the
  id against the bootstrap id, and `deactivate_school` in
  `services.rs` will retire the bootstrap school unconditionally.

**Expected:**

`docs/specs/platform/aggregates.md:43` —
  `8. The bootstrap School (id 1) cannot be deleted.`

**Evidence:**

`crates/cross-cutting/platform/src/services.rs:225-260` —
  ```rust
  pub fn deactivate_school<C>(
      ctx: &TenantContext,
      school: &mut School,
      cmd: DeactivateSchoolCommand,
      clock: &C,
  ) -> Result<SchoolDeactivated> {
      ...
      let DeactivateSchoolCommand { tenant, school_id, reason, new_status } = cmd;
      debug_assert_eq!(tenant.school_id, school_id);
      ...
      school.status = new_status;
      school.active_status = ActiveStatus::Retired;
      ...
  }
  ```
  No `school_id != PLATFORM_BOOTSTRAP` check; no reference to
  a bootstrap constant. `PLATFORM_BOOTSTRAP` does not exist in
  the crate; only `PLATFORM_SCHOOL_ID` is re-exported from
  `educore-core` in `lib.rs:78`.

---

### FINDING 6 (id: `CROSSCUT-PLAT-006`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Critical
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/aggregate.rs:46-242

**Description:**

Spec invariant 4 of the School requires
  email uniqueness; the School struct has no email field, so
  no validation. Spec invariant 7 (User `is_administrator`)
  and invariant 10 (User `is_registered`) likewise have no
  fields. Invariant 8 of User ("role_id references a valid
  role") is satisfied by `role_ids: Vec<RoleId>` only in the
  case of multi-role; the spec's `User::role_id` (singular)
  cannot be expressed by the `Vec<RoleId>` shape.

**Expected:**

`docs/specs/platform/aggregates.md:39, 104-108`:
  `4. A School::email is RFC-valid and unique.`;
  `7. A User::is_administrator is Yes or No.`;
  `8. A User::role_id references a valid role in the same school.`;
  `10. A User::is_registered is a boolean indicating whether the user has completed self-registration.`

**Evidence:**

`crates/cross-cutting/platform/src/aggregate.rs:46-91` and
  `:142-194` — see fields listed in findings 1 and 2. No
  `IsAdministrator`, `IsRegistered`, or `SchoolEmail` value
  object is defined in `value_objects.rs`.

---

### FINDING 7 (id: `CROSSCUT-PLAT-007`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Critical
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/aggregate.rs (full file)

**Description:**

Only 2 of the 37 aggregates documented in
  `docs/specs/platform/aggregates.md` are implemented as Rust
  structs (`School` and `User`). 35 aggregates are missing:
  OtpCode, Course, CourseCategory, CoursePage, CustomField,
  CustomFieldValue, ChartOfAccount, BaseGroup, BaseSetup,
  Module, ModuleLink, AddOn, ModuleManager,
  ModuleStudentParentInfo, TimeZone, Country, Continent,
  Currency, Language, SocialMediaIcon, HeaderMenuManager,
  PhotoGallery, VideoGallery, Visitor, ToDo, Instruction,
  ExpertTeacher, FrontendPermission, AmountTransfer, Plugin,
  Comment, CommentTag, PersonalAccessToken, VideoUpload
  (and `CommentPivot` is documented as a join entity).

**Expected:**

`docs/specs/platform/aggregates.md` defines 37
  aggregate sections (line count: `grep -c "^## " aggregates.md`
  returns 37).

**Evidence:**

`crates/cross-cutting/platform/src/aggregate.rs:46-242`
  defines exactly two aggregates (`School`, `User`). The
  crate's `README.md:7-10` explicitly documents this: "The
  remaining 30 secondary platform aggregates enumerated in
  `docs/specs/platform/aggregates.md` (Course, OtpCode, Module,
  Plugin, ...) are out of scope for Phase 2 and land in later
  phases alongside their owning events." (The number is wrong:
  there are 35 secondary aggregates, not 30.)

---

### FINDING 8 (id: `CROSSCUT-PLAT-008`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Critical
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/commands.rs:59-243

**Description:**

Only 6 of the ~117 commands documented in
  `docs/specs/platform/commands.md` are implemented. The
  missing commands include all commands for 35 secondary
  aggregates plus 5 additional commands for `School` and
  `User`: `ApproveSchool`, `DisableLogin`, `EnableLogin`,
  `ReactivateUser`, `ChangeUserRole`, `VerifyEmail`,
  `ResetPassword`. (Note: `DisableLogin` / `EnableLogin` is
  one section with two commands per spec.)

**Expected:**

`docs/specs/platform/commands.md` lists
  `ApproveSchool` (line 78-86), `DisableLogin` /
  `EnableLogin` (line 92-105), `ReactivateUser` (line 173-180),
  `ChangeUserRole` (line 185-193), `VerifyEmail` (line 199-207),
  `ResetPassword` (line 213-220) — all part of the
  Phase 2-implemented aggregates (School, User).

**Evidence:**

`crates/cross-cutting/platform/src/commands.rs`
  defines only 6 commands:
  `CreateSchoolCommand`, `UpdateSchoolCommand`,
  `DeactivateSchoolCommand`, `RegisterUserCommand`,
  `UpdateUserCommand`, `DeactivateUserCommand`
  (`grep -c "^pub struct.*Command\b" commands.rs` returns 6).
  `grep -E "^pub struct.*Command\b" commands.rs` enumerates the
  six. No `ApproveSchoolCommand`, `DisableLoginCommand`,
  `EnableLoginCommand`, `ReactivateUserCommand`,
  `ChangeUserRoleCommand`, `VerifyEmailCommand`,
  `ResetPasswordCommand`, `IssueOtpCommand`, `VerifyOtpCommand`,
  `ExpireOtpCommand`, `CreateCourseCommand`, etc. is present.

---

### FINDING 9 (id: `CROSSCUT-PLAT-009`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Critical
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/events.rs:1-474

**Description:**

Only 6 of the ~73 events documented in
  `docs/specs/platform/events.md` are implemented. Missing
  events include all events for the 35 secondary aggregates
  plus 5 additional events for `School` and `User`:
  `SchoolApproved`, `LoginDisabled`, `LoginEnabled`,
  `UserReactivated`, `UserRoleChanged`, `EmailVerified`,
  `PasswordReset`.

**Expected:**

`docs/specs/platform/events.md` lines 73-97
  (`SchoolApproved`, `LoginDisabled`, `LoginEnabled`),
  lines 140-177 (`UserReactivated`, `UserRoleChanged`,
  `EmailVerified`, `PasswordReset`) — all required by the
  Phase 2-implemented aggregates.

**Evidence:**

`crates/cross-cutting/platform/src/events.rs`
  defines exactly 6 events:
  `SchoolCreated`, `SchoolUpdated`, `SchoolDeactivated`,
  `UserRegistered`, `UserUpdated`, `UserDeactivated`. No
  `SchoolApproved`, `LoginDisabled`, `LoginEnabled`,
  `UserReactivated`, `UserRoleChanged`, `EmailVerified`,
  `PasswordReset`, `OtpIssued`, `CourseCreated`, etc. is
  defined (`grep -c "^pub struct" events.rs` returns 6
  matching `*Event*` names).

---

### FINDING 14 (id: `CROSSCUT-PLAT-014`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/repository.rs:36-66

**Description:**

`SchoolRepository::get_by_domain` takes
  `&str`, not the spec's typed `Domain` value object. Same for
  `get_by_code` (`&str` instead of `SchoolCode`). The
  UserRepository's `get_by_email`, `get_by_username`, and
  `get_by_phone` take `&str` rather than the spec's typed
  `EmailAddress`, `Username`, `PhoneNumber` value objects.
  Spec drift from typed wrappers to strings.

**Expected:**

`docs/specs/platform/repositories.md:13-14`:
  `async fn get_by_domain(&self, domain: &Domain) -> Result<Option<School>>;`
  `async fn get_by_code(&self, code: &SchoolCode) -> Result<Option<School>>;`.
  Lines 29-31: `get_by_email` takes `&EmailAddress`;
  `get_by_username` takes `&Username`; `get_by_phone` takes
  `&PhoneNumber`.

**Evidence:**

`crates/cross-cutting/platform/src/repository.rs:40-44`:
  ```rust
  async fn get_by_domain(&self, domain: &str) -> Result<Option<School>>;
  async fn get_by_code(&self, code: &str) -> Result<Option<School>>;
  ```
  `crates/cross-cutting/platform/src/repository.rs:80-87`:
  ```rust
  async fn get_by_email(&self, school: SchoolId, email: &str) -> Result<Option<User>>;
  async fn get_by_username(&self, school: SchoolId, username: &str) -> Result<Option<User>>;
  async fn get_by_phone(&self, school: SchoolId, phone: &str) -> Result<Option<User>>;
  ```

---

### FINDING 15 (id: `CROSSCUT-PLAT-015`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/repository.rs:71-110

**Description:**

`UserRepository` is missing the
  `query(UserQuery)`, `count(UserQuery)`, and
  `page(UserQuery, offset, limit)` methods from the spec.
  These are the typed-query plumbing the storage adapter needs.

**Expected:**

`docs/specs/platform/repositories.md:37-39`:
  ```rust
  async fn query(&self, q: UserQuery) -> Result<Vec<User>>;
  async fn count(&self, q: UserQuery) -> Result<u64>;
  async fn page(&self, q: UserQuery, offset: u32, limit: u32) -> Result<Page<User>>;
  ```

**Evidence:**

`crates/cross-cutting/platform/src/repository.rs:71-110`
  — the trait body has 8 methods (`get`, `get_by_email`,
  `get_by_username`, `get_by_phone`, `list`, `list_by_role`,
  `list_by_usertype`, `insert`, `update`). No `query`,
  `count`, or `page` method exists.

---

### FINDING 16 (id: `CROSSCUT-PLAT-016`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/permissions.md (none — module absent)

**Description:**

`docs/specs/platform/permissions.md` defines
  ~115 capability strings grouped by aggregate. There is no
  Rust enum, no capability constant module, no
  `CapabilityCheck` trait, and no capability helper code in
  the platform crate. The platform crate has 6 commands whose
  capability checks are documented but not enforced in code
  (e.g. `Platform.School.Create`, `Platform.User.Register`).

**Expected:**

`docs/specs/platform/permissions.md:18-229` —
  per-aggregate `Platform.<Aggregate>.<Action>` capability
  strings; line 251-255 example:
  `engine.rbac().has(actor_id, Capability::PlatformUserRegister).await?`.

**Evidence:**

`grep -rn "Capability" crates/cross-cutting/platform/src/`
  returns no `pub enum Capability`, no `Capability::Platform*`,
  no `pub const PLATFORM_*_CAPABILITY` rows. The platform crate
  depends on `educore-rbac` for capability checks per the
  cross-cutting tier (no such dependency exists in
  `Cargo.toml:13-20`).

---

### FINDING 17 (id: `CROSSCUT-PLAT-017`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/commands.rs:246-326

**Description:**

The 5 validation helper functions
  (`validate_school_name`, `validate_school_code`,
  `validate_username`, `validate_display_name`,
  `validate_reason`) are `pub(crate)` and live in
  `commands.rs`, but spec invariants 1, 3, 4, 11 of `School`
  and 1, 4, 11 of `User` belong to the aggregate itself. The
  helpers are imported by `services.rs:34-37` only; there is
  no aggregate-level guard (e.g. `School::new(name)?.validate()?`)
  that the engine can call. Spec invariant 2 ("`School::domain`
  is unique across the platform") is not enforced by code at
  construction time.

**Expected:**

`docs/specs/platform/aggregates.md:36-46`
  invariants apply to the `School` aggregate.

**Evidence:**

`crates/cross-cutting/platform/src/commands.rs:247-326` —
  5 `pub(crate)` validators that return `educore_core::error::Result`;
  the only caller is `services.rs`. The `School::fresh`
  constructor (`aggregate.rs:104-133`) does no validation;
  `User::fresh` (`:204-242`) does no validation.

---

### FINDING 18 (id: `CROSSCUT-PLAT-018`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/commands.rs (and modules)

**Description:**

`docs/specs/platform/commands.md` has no
  `pub fn` validation helpers (they are not part of the
  documented command shapes), but the Rust code defines 5
  `pub(crate)` validators in `commands.rs`. The pub-crate
  scope is fine; the issue is that the helpers are
  inconsistently named (`validate_reason` is generic; others
  are aggregate-specific) and there is no test for
  `validate_school_name` rejecting names with > 200 chars
  (although the test `validate_username_rejects_overlong`
  covers the equivalent for username).

**Expected:**

Spec consistency between command shapes and
  validator functions.

**Evidence:**

`crates/cross-cutting/platform/src/commands.rs:247-326`:
  5 validators; `commands.rs:382-398` defines 5 tests but the
  test `validate_school_name` is missing.

---

### FINDING 22 (id: `CROSSCUT-PLAT-022`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/services.rs:225-260

**Description:**

`deactivate_school` accepts
  `DeactivateSchoolCommand::new_status` (any `SchoolStatus`)
  and assigns it to `school.status`. The spec invariant 5
  restricts `School::active_status` to `Approved | Pending`
  only; the service code allows `Suspended` and `Active`
  assignments from any caller, with no validation.

**Expected:**

Spec invariant 5 of School limits status to
  two values.

**Evidence:**

`crates/cross-cutting/platform/src/services.rs:225-260`:
  ```rust
  pub fn deactivate_school<C>(...) -> Result<SchoolDeactivated> {
      ...
      school.status = new_status;
      school.active_status = ActiveStatus::Retired;
      ...
  }
  ```
  No `if !matches!(new_status, SchoolStatus::Approved |
  SchoolStatus::Pending)` guard.

---

### FINDING 23 (id: `CROSSCUT-PLAT-023`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/entities.rs:36-138

**Description:**

Only 4 of the 35 entities documented in
  `docs/specs/platform/entities.md` are implemented
  (`SchoolContact`, `UserSession`, `UserPreference`,
  `UserLogin`). Missing: `SchoolPackage`, `SchoolRegion`,
  `UserDocument`, `OtpDelivery`, `CourseInstructor`,
  `CourseMaterial`, `CourseEnrollment`, `CourseReview`,
  `CustomFieldOption`, `CustomFieldValidation`,
  `ChartOfAccountBalance`, `BaseSetupTranslation`,
  `ModuleLinkPermission`, `ModuleLinkChild`, `ModuleLinkRoute`,
  `AddOnManifest`, `AddOnInstallation`, `ModuleManagerEndpoint`,
  `VisitorAttachment`, `ToDoAssignee`, `ToDoComment`,
  `InstructionAttachment`, `FrontendPermissionOverride`,
  `AmountTransferAttachment`, `AmountTransferReversal`,
  `PluginConfig`, `PluginHook`, `CommentMention`,
  `PersonalAccessTokenLastUsed`, `VideoUploadChapter`,
  `VideoUploadView`, `ModuleInfo` (referenced by
  `platform_module_infos` table).

**Expected:**

`docs/specs/platform/entities.md` defines 35
  entity sections.

**Evidence:**

`crates/cross-cutting/platform/src/entities.rs:36-138`
  defines only 4 entities.

---

### FINDING 27 (id: `CROSSCUT-PLAT-027`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/lib.rs:20-21

**Description:**

`lib.rs` has `#![forbid(unsafe_code)]` and
  `#![deny(missing_docs)]`. The `#[cfg(test)] mod tests`
  block at line 117-152 and `aggregate.rs:244-251`,
  `commands.rs:328-334`, `events.rs:475-481`,
  `value_objects.rs:460-466`, `services.rs:505-512`,
  `query.rs:140-146`, `entities.rs:139-145`,
  `repository.rs:112-118` all use `#[allow(...)]` on the
  inner `mod tests` to permit `unwrap_used`,
  `expect_used`, `panic`, `dbg_macro`, and `dead_code`. The
  audit task states these are forbidden in non-test code
  paths; the `#[cfg(test)] mod tests` exemption is normal.

**Expected:**

Production paths (non-`#[cfg(test)]`) have
  no `unwrap`/`expect`/`panic`/`dbg!`; the `#[cfg(test)]`
  block is exempt.

**Evidence:**

`crates/cross-cutting/platform/src/lib.rs:20-21`,
  `aggregate.rs:244-251`, `commands.rs:328-334`,
  `events.rs:475-481`, `value_objects.rs:460-466`,
  `services.rs:505-512`, `query.rs:140-146`,
  `entities.rs:139-145`, `repository.rs:112-118`. Audit
  confirmation: every `unwrap()` / `expect()` in the crate
  lives inside a `#[cfg(test)]` block. No production
  `unwrap()` or `expect()` was found.

---

### FINDING 28 (id: `CROSSCUT-PLAT-028`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/query.rs (full file)

**Description:**

`SchoolQuery::execute` (line 74-79) and
  `UserQuery::execute` (line 132-137) return
  `Err(DomainError::NotSupported)` with the explanation
  "Phase 2 stub". The query layer is incomplete: there is
  no `#[derive(DomainQuery)]` macro usage, no AST emission,
  no SQL translation. The crate depends on the deferred
  Phase 3+ implementation.

**Expected:**

`docs/specs/platform/repositories.md:37-39`
  specifies typed `query`, `count`, `page` methods on
  `UserRepository` that take `UserQuery`.

**Evidence:**

`crates/cross-cutting/platform/src/query.rs:74-79, 132-137`:
  ```rust
  pub async fn execute(self, _school: SchoolId) -> Result<Vec<School>> {
      let _ = self;
      Err(DomainError::not_supported(
          "SchoolQuery::execute is a Phase 2 stub; the typed query executor lands in Phase 3+",
      ))
  }
  ```

---

### FINDING 3 (id: `CROSSCUT-PLAT-003`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/value_objects.rs:312-327

**Description:**

The `SchoolStatus` enum exposes four
  variants (`Pending`, `Approved`, `Suspended`, `Active`) but
  the spec mandates only two (`Approved`, `Pending`) per
  `docs/specs/platform/value-objects.md` lines 68-72 and
  `docs/specs/platform/aggregates.md` invariant 5
  ("`A School::active_status is Approved or Pending`"). The
  `Suspended` and `Active` variants are spec drift; the
  comment on `Active` (lines 322-326) admits "The distinction
  between Active and Approved is a legacy of the Schoolify
  data model; the engine treats them identically for query
  purposes".

**Expected:**

`docs/specs/platform/value-objects.md:69-71` —
  `SchoolActiveStatus | Approved, Pending`.

**Evidence:**

`crates/cross-cutting/platform/src/value_objects.rs:312-327` —
  ```rust
  pub enum SchoolStatus {
      #[default]
      Pending,
      Approved,
      Suspended,
      Active,
  }
  ```

---

### FINDING 30 (id: `CROSSCUT-PLAT-030`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/tests/platform_e2e.rs (full file)

**Description:**

The integration test file has 10 test
  functions but none exercise storage parity. The
  `InMemoryUniqueness` test fixture is hand-rolled inside
  the test file (lines 60-115). The crate depends on
  `educore-storage` for the storage port but has no
  storage-parity test (e.g. against an in-memory SQLite
  adapter). The README, handoff, and coverage matrix all
  claim the platform crate has been tested end-to-end.

**Expected:**

`AGENTS.md` "Test infrastructure + SDK"
  phase 16 deliverable: storage-parity tests.

**Evidence:**

`crates/cross-cutting/platform/tests/platform_e2e.rs:38`:
  `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`
  No `tokio::test` against a real storage adapter; no
  `educore_storage_sqlite::SqliteStorage` import. `grep -n
  "storage\|Storage" tests/platform_e2e.rs` returns only doc
  comments.

---

### FINDING 38 (id: `CROSSCUT-PLAT-038`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** docs/specs/platform/workflows.md (201 lines, full file)

**Description:**

11 of the 14 workflows in
  `docs/specs/platform/workflows.md` have zero
  implementation. Workflows with no code path: OTP
  Verification (lines 43-61), Course Management (lines
  63-74), Module Installation (lines 76-88), AddOn
  Installation (lines 90-100), Custom Field Configuration
  (lines 102-113), Header Menu Configuration (lines
  115-123), Visitor Log (lines 125-133), ToDo (lines
  135-143), Amount Transfer (lines 145-154), Personal Access
  Token (lines 156-166), Comment Moderation (lines 168-177).
  The 3 partial workflows (School Onboarding, User
  Registration, User Deactivation) cover only the
  first 1-2 steps and rely on subscribers (rbac, settings,
  academic, operations, communication, cms) that do not
  exist in code.

**Expected:**

14 workflows, all steps implemented.

**Evidence:**

`crates/cross-cutting/platform/src/`
  contains no subscriber modules, no
  `OnSchoolCreatedSubscriber`/`OnUserRegisteredSubscriber`
  etc.; `grep -rn "Subscriber\|subscriber" src/` returns no
  rows.

---

### FINDING 45 (id: `CROSSCUT-PLAT-045`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** docs/coverage.toml (platform rows)

**Description:**

`docs/coverage.toml` has only 3
  `educore-platform` rows (`platform_schools_aggregate`,
  `platform_users_aggregate`, `platform_sessions_aggregate`).
  The `platform_sessions_aggregate` row references
  `UserSession`, which is a child entity in `entities.rs:62-84`
  (not a spec-listed aggregate; the spec has no
  `Session` aggregate). Coverage drift between the spec
  (37 aggregates) and the matrix (3 rows).

**Expected:**

Per-aggregate coverage matrix rows matching
  the spec.

**Evidence:**

`docs/coverage.toml:368-393`:
  ```toml
  [[row]]
  id = "platform_schools_aggregate"
  ...
  [[row]]
  id = "platform_users_aggregate"
  ...
  [[row]]
  id = "platform_sessions_aggregate"
  item = "platform_sessions aggregate"
  spec = "docs/specs/platform/aggregates.md"
  crate = "educore-platform"
  phase = 2
  status = "Tested"
  tests = "crates/cross-cutting/platform/tests/platform_e2e.rs"
  ```
  No `platform_*_aggregate` rows for the 35 missing
  aggregates.

---

### FINDING 48 (id: `CROSSCUT-PLAT-048`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** docs/handoff/PHASE-2-HANDOFF.md:73-104

**Description:**

Phase 2 is marked **closed** in the
  handoff (line 4: "Status: Phase 2 closed") but the
  platform crate ships only 2 of 37 spec aggregates,
  6 of ~117 commands, 6 of ~73 events, 7 of ~100 value
  objects, 2 of 28 repository traits, 0 of 17 services.
  `cargo build --workspace` and `cargo test --workspace`
  pass because the scaffold is compilable, but the
  production-readiness surface area is much smaller than
  the spec implies. The handoff's own caveat (line 75:
  "The prompt-named subset (no 30 secondary aggregates)")
  is correct but understates the gap (35 secondary
  aggregates, not 30).

**Expected:**

Phase 2 closed means the spec surface for
  Phase 2 is implemented; the spec surface for the
  platform crate spans all 37 aggregates regardless of
  phase.

**Evidence:**

`docs/handoff/PHASE-2-HANDOFF.md:73-104`
  lists only the School and User work; lines 291-292 say
  "Do NOT add the 30 secondary platform aggregates". The
  README at `crates/cross-cutting/platform/README.md:7-10`
  repeats the same scope. The audit task states
  "**Total findings:** 48" — Phase 2 readiness for the
  platform crate is partial.

---

### FINDING 20 (id: `CROSSCUT-PLAT-020`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/services.rs:152, 241, 380, 483

**Description:**

`let _ = ctx;` appears in 4 places in
  `services.rs`. The `ctx` parameter is intended to drive
  `school.updated_by = ctx.actor_id` and `event.correlation_id
  = ctx.correlation_id` (lines 185, 247, 421, 489, 498), so
  the `let _ = ctx;` is dead. The unused discard suggests the
  function signature accepts `ctx` for symmetry but the
  compiler-lint bypass is suspect.

**Expected:**

No `let _ = ctx;` discards in production
  paths.

**Evidence:**

`crates/cross-cutting/platform/src/services.rs:152`
  (in `update_school`), `:241` (in `deactivate_school`), `:380`
  (in `update_user`), `:483` (in `deactivate_user`).

---

### FINDING 21 (id: `CROSSCUT-PLAT-021`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/services.rs:386-388, 429

**Description:**

`update_user` captures `email_at_call`,
  `display_name_at_call`, `phone_at_call` (lines 386-388) but
  never reads them; the `let _ = (email_at_call, ...)` at
  line 429 discards them. The comments (lines 384-385) say
  "Snapshot the pre-mutation values so the event can carry
  'what changed' without aliasing the post-mutation state",
  but the event construction (lines 430-452) does not read
  the snapshot; it only reads post-mutation state.

**Expected:**

No dead captures.

**Evidence:**

`crates/cross-cutting/platform/src/services.rs:384-429`:
  ```rust
  let email_at_call = user.email.clone();
  let display_name_at_call = user.display_name.clone();
  let phone_at_call = user.phone_number.clone();
  ...
  let _ = (email_at_call, display_name_at_call, phone_at_call);
  ```

---

### FINDING 24 (id: `CROSSCUT-PLAT-024`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/entities.rs:93-114

**Description:**

`UserPreference::value_json: String` is
  used to store a "typed JSON blob" (line 105-110 comment).
  This is `serde_json::Value`-shaped data represented as a
  string; the engine's value type is JSON-shaped (per
  comment). This is borderline `serde_json::Value`-as-string
  drift and depends on the storage adapter to parse.

**Expected:**

Typed wrapper or `serde_json::Value` import
  explicitly (with `serde_json` already in `Cargo.toml:19`).

**Evidence:**

`crates/cross-cutting/platform/src/entities.rs:93-114`:
  ```rust
  pub struct UserPreference {
      ...
      pub value_json: String,
      ...
  }
  ```
  `Cargo.toml:19` includes `serde_json`. No import of
  `serde_json::Value` in this struct (audit-friendly), but
  `String`-as-JSON drift is implicit.

---

### FINDING 29 (id: `CROSSCUT-PLAT-029`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/query.rs (full file)

**Description:**

The `SchoolQuery` and `UserQuery` structs
  are untyped string/option bags — they have no
  `#[derive(DomainQuery)]` annotation, no field-marker
  macro, no `SchoolQueryField` / `UserQueryField` enums for
  compile-time field access. The query layer is hand-rolled
  rather than macro-generated, violating the engine's
  "compile-time safety over strings" rule.

**Expected:**

`docs/project-overview.md` and `AGENTS.md`
  rule 2: "Use macro-generated enums (`StudentField::Status`) —
  never string field names."

**Evidence:**

`crates/cross-cutting/platform/src/query.rs:27-95`:
  ```rust
  pub struct SchoolQuery {
      pub status_filter: Option<...>,
      pub name_contains: Option<String>,
      pub code_contains: Option<String>,
  }
  pub struct UserQuery {
      pub usertype_filter: Option<UserType>,
      pub status_filter: Option<...>,
      pub role_filter: Option<RoleId>,
      pub username_contains: Option<String>,
      pub email_contains: Option<String>,
  }
  ```
  `grep -n "derive(DomainQuery)" crates/cross-cutting/platform/`
  returns no rows.

---

### FINDING 31 (id: `CROSSCUT-PLAT-031`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/tests/platform_e2e.rs (full file)

**Description:**

The integration test file (10 tests) does
  not exercise the failure path of `update_school` when the
  supplied domain conflicts. The spec's `UpdateSchool`
  effect ("Emits `SchoolUpdated`") requires the
  domain-uniqueness guard, but no test asserts
  `update_school` returns `Err(Conflict)` on a duplicate
  domain.

**Expected:**

Tests must validate real-world scenarios
  including error paths (per `AGENTS.md` testing rules).

**Evidence:**

`crates/cross-cutting/platform/tests/platform_e2e.rs:269-305`
  contains 2 `update_school_increments_version` calls; both
  are happy paths with `domain: None`. No test exercises the
  conflict path; the 6 test cases listed in the file doc
  (lines 12-33) match the integration test plan, but the
  conflict path is missing.

---

### FINDING 32 (id: `CROSSCUT-PLAT-032`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/tests/platform_e2e.rs (full file)

**Description:**

The integration tests do not exercise the
  `SchoolUpdated` envelope's `changed_fields` list for
  multi-field updates. The single-field update is tested
  (`event1.changed_fields == vec!["name"]`, line 293; same
  for `event2` line 304), but a test that asserts
  `changed_fields` carries both `name` and `domain` after a
  compound `update_school` call is absent.

**Expected:**

Coverage of multi-field update behavior.

**Evidence:**

`crates/cross-cutting/platform/tests/platform_e2e.rs:269-305` —
  both `UpdateSchoolCommand` calls have `domain: None` and
  `package_id: None`, exercising only the `name`-change path.

---

### FINDING 33 (id: `CROSSCUT-PLAT-033`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/tests/platform_e2e.rs (full file)

**Description:**

No integration test covers `update_user`.
  The unit test `register_user_emits_event` exists
  (`services.rs:663-689`) and the integration test
  `user_register_emits_user_registered_event` exists, but
  no test calls `update_user` or asserts `UserUpdated` event
  metadata. This is a test gap for a documented command.

**Expected:**

Integration tests for every command in the
  spec section.

**Evidence:**

`grep -n "update_user\|UserUpdated" crates/cross-cutting/platform/tests/platform_e2e.rs`
  returns no rows.

---

### FINDING 34 (id: `CROSSCUT-PLAT-034`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/tests/platform_e2e.rs (full file)

**Description:**

No integration test covers `deactivate_school`
  end-to-end. The integration test
  `deactivate_user_sets_active_status_retired` (line 308)
  covers the user path, but the symmetric `deactivate_school`
  path is tested only as a unit test in `services.rs:640-661`
  and does not assert the `SchoolDeactivated` event envelope
  metadata.

**Expected:**

Integration test coverage for both
  deactivation paths.

**Evidence:**

`grep -n "deactivate_school\|SchoolDeactivated" crates/cross-cutting/platform/tests/platform_e2e.rs`
  returns no rows.

---

### FINDING 35 (id: `CROSSCUT-PLAT-035`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/services.rs:225-260

**Description:**

`deactivate_school` does not produce an
  integration test of its event envelope. The `update_school`
  envelope metadata is not tested either. Per the crate's
  `services.rs:189-191` comment, the event id is "a
  placeholder event id here for the envelope (the bus port
  stamps its own event id at publish time, so this is
  informational only)"; an envelope round-trip test would
  document the placeholder behavior.

**Expected:**

Integration tests for `SchoolDeactivated`
  and `SchoolUpdated` envelope metadata.

**Evidence:**

`crates/cross-cutting/platform/tests/platform_e2e.rs`
  contains `school_create_emits_school_created_event`,
  `user_register_emits_user_registered_event`,
  `envelope_propagates_correlation_id`,
  `school_starts_pending_and_event_id_round_trips`, but no
  `school_deactivate_emits_school_deactivated_event`,
  `school_update_emits_school_updated_event`,
  `user_update_emits_user_updated_event`,
  `user_deactivate_emits_user_deactivated_event`.

---

### FINDING 39 (id: `CROSSCUT-PLAT-039`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/services.rs:194-197, 249, 423, 491

**Description:**

`update_school`, `deactivate_school`,
  `update_user`, `deactivate_user` all mint a placeholder
  `EventId::from_uuid(uuid::Uuid::now_v7())` and use it as
  `last_event_id`. The crate's `create_school` and
  `register_user` use the `IdGenerator` port (lines 102,
  330). Spec drift: `update_*` and `deactivate_*` bypass the
  port and mint their own id, defeating the test
  determinism that `create_*` provides.

**Expected:**

`educore_core::clock::IdGenerator` port
  usage in all mutation services.

**Evidence:**

`crates/cross-cutting/platform/src/services.rs:194-197`:
  `educore_core::ids::EventId::from_uuid(uuid::Uuid::now_v7())`;
  line 249 same; line 423 same; line 491 same. Only lines
  102 and 330 use `_ids.next_event_id()` (via `IdGenerator`
  port).

---

### FINDING 40 (id: `CROSSCUT-PLAT-040`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/services.rs:131-212

**Description:**

`update_school` accepts an `IdGenerator`
  parameter on `create_school` (line 56) and `register_user`
  (line 276) but `update_school` (line 131), `deactivate_school`
  (line 225), `update_user` (line 362), and `deactivate_user`
  (line 467) do not take an `IdGenerator`. The
  `IdGenerator` boundary is inconsistent — mutators that
  emit events should consume the port; `update_*` and
  `deactivate_*` do not.

**Expected:**

Uniform port consumption across all
  mutation services.

**Evidence:**

`crates/cross-cutting/platform/src/services.rs:131, 225, 362, 467` —
  4 service functions without `IdGenerator` parameters.
  Lines 56 and 276 have `where G: IdGenerator + ?Sized`
  constraints.

---

### FINDING 42 (id: `CROSSCUT-PLAT-042`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/commands.rs:172-243

**Description:**

`CreateSchoolCommand::new`,
  `RegisterUserCommand::new`, `DeactivateSchoolCommand::new`,
  `DeactivateUserCommand::new` are convenience
  constructors that bypass the typed fields. `new` for
  `CreateSchoolCommand` omits `domain` and `package_id`
  (set to `None`), `RegisterUserCommand::new` hard-codes
  `usertype: UserType::Staff` (line 212) and `role_ids:
  Vec::new()`. These are reasonable for tests but they are
  the only constructors in the public API; the spec lists
  full struct fields.

**Expected:**

Spec-shaped command struct initialization
  (e.g. with all fields explicit).

**Evidence:**

`crates/cross-cutting/platform/src/commands.rs:175-189, 195-216, 222-229, 235-242`.

---

### FINDING 46 (id: `CROSSCUT-PLAT-046`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/value_objects.rs:399-413

**Description:**

`PackageId` is `#[serde(transparent)]`
  with the inner `Uuid` exposed (`pub Uuid`). The struct
  field is `pub`, allowing external code to mutate the
  inner id without going through `from_uuid`. Spec drift
  (the spec describes `PackageId` as an `Id<Package>` and
  generally wraps `Uuid` privately).

**Expected:**

Private inner field; constructor-only
  initialization.

**Evidence:**

`crates/cross-cutting/platform/src/value_objects.rs:399-413`:
  ```rust
  pub struct PackageId(pub Uuid);
  ```

---

### FINDING 47 (id: `CROSSCUT-PLAT-047`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/value_objects.rs:430-452

**Description:**

`RoleId(SchoolId, Uuid)` is
  `#[serde(transparent)]` over `SchoolId`? No, it is not
  annotated, so it serializes as a tuple of two fields. The
  spec defines `RoleId` from `educore-rbac` (per
  `value-objects.md:64`). The platform crate defines its
  own `RoleId` rather than depending on `educore-rbac`.
  Spec drift on ownership.

**Expected:**

`docs/specs/platform/value-objects.md:64`:
  `RoleId | From educore-rbac`.

**Evidence:**

`crates/cross-cutting/platform/src/value_objects.rs:430-458`:
  ```rust
  pub struct RoleId(pub SchoolId, pub Uuid);
  ```
  The platform crate does not depend on `educore-rbac` in
  `Cargo.toml:13-20`; both fields are `pub`, allowing
  direct mutation.

---

### FINDING 36 (id: `CROSSCUT-PLAT-036`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Low
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/events.rs:109-130

**Description:**

`SchoolUpdated::package_id: Option<Uuid>`
  uses `Uuid` instead of the spec's `Option<SchoolPackageId>`
  (per `docs/specs/platform/events.md:45`: `package_id:
  Option<SchoolPackageId>`). Spec drift to the raw id type.

**Expected:**

`docs/specs/platform/events.md:45` —
  `pub package_id: Option<SchoolPackageId>`.

**Evidence:**

`crates/cross-cutting/platform/src/events.rs:122`:
  `pub package_id: Option<Uuid>,` (`grep -n "package_id" events.rs`
  returns this single occurrence; `PackageId` is the
  platform-defined newtype, but the event uses bare `Uuid`).

---

### FINDING 37 (id: `CROSSCUT-PLAT-037`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Low
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/events.rs:339-348

**Description:**

`UserUpdated::role_ids` (if it existed)
  is not modeled; the event has no `role_ids` field. Spec
  invariant for `UserUpdated` only requires `changed_fields`
  per spec, but the event cannot reflect `role_ids` changes
  because the aggregate's `role_ids` change does not emit a
  separate event. The `ChangeUserRole` command is missing
  entirely (Finding 8).

**Expected:**

`docs/specs/platform/events.md:121-125`:
  `pub struct UserUpdated { pub user_id: UserId, pub changed_fields: Vec<&'static str> }`.

**Evidence:**

`crates/cross-cutting/platform/src/events.rs:333-355`
  defines `UserUpdated` without `role_ids`; the missing
  `ChangeUserRole` command means no event carries role
  changes.

---

### FINDING 41 (id: `CROSSCUT-PLAT-041`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Low
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/events.rs:339-355

**Description:**

`UserUpdated` carries a `phone_number:
  Option<String>` field (line 347) instead of the typed
  `Option<PhoneNumber>` value object. Spec drift to a raw
  string; loses the E.164 validation guarantee.

**Expected:**

`docs/specs/platform/events.md:121-125` —
  `pub struct UserUpdated { pub user_id, pub changed_fields }`
  (no `phone_number` field per spec; per `commands.md:140-153`
  `UpdateUserCommand`, `phone_number` is `Option<PhoneNumber>`).

**Evidence:**

`crates/cross-cutting/platform/src/events.rs:339-355`:
  ```rust
  pub struct UserUpdated {
      ...
      pub phone_number: Option<String>,
      ...
  }
  ```

---

### FINDING 43 (id: `CROSSCUT-PLAT-043`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Low
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/services.rs:308-312

**Description:**

`register_user` returns
  `Err(DomainError::Conflict(format!("email {:?} is already
  in use within the school", email.as_str())))`. The error
  format string uses `{:?}` for an `EmailAddress` that
  already implements `Display` (via `value_objects.rs:99-103`).
  Use of `{:?}` is anti-pattern (yields quoted debug form
  with escapes); `{}` should be used.

**Expected:**

Display formatter usage over Debug in
  user-facing error messages.

**Evidence:**

`crates/cross-cutting/platform/src/services.rs:302-306, 393-397`:
  two occurrences of `email {:?}` in error messages.

---

### FINDING 44 (id: `CROSSCUT-PLAT-044`)

- **Source:** `docs/audit_reports/findings/wave2-platform.md`
- **Severity:** Low
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/platform/src/services.rs:74, 241, 380, 483

**Description:**

Multiple services destructure
  `tenant: TenantContext` from the command and then
  `debug_assert_eq!(tenant.school_id, school_id)` but the
  destructured `ctx` variable is the same `tenant` and is
  later used (via `ctx.actor_id`, `ctx.correlation_id`).
  The variable is captured by both the destructure and the
  `let ctx = tenant;` rebind (lines 74, 298). The pattern
  is repetitive and error-prone.

**Expected:**

Consistent destructure naming and
  `debug_assert_eq!` usage.

**Evidence:**

`crates/cross-cutting/platform/src/services.rs:74, 152, 241, 298, 380, 483` —
  the `let _ = ctx;` and `let ctx = tenant;` patterns
  appear in 5 services.

---


## RBAC (target id prefix: `CROSSCUT-RBAC`)

**Path:** `crates/cross-cutting/rbac/`  
**Total findings:** 36 (7 critical, 18 high, 9 medium, 2 low)


### FINDING 1 (id: `CROSSCUT-RBAC-001`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** Critical
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/rbac/src/value_objects.rs:4148-4767

**Description:**

`Capability::all()` is missing 46 of the 654 enum
  variants — every Phase 15 variant (Auth, Notify, Payment, Files,
  Integrations). The function enumerates 608 variants; the enum
  contains 654 variants. The 46 missing are: `AuthLogin`,
  `AuthLogout`, `AuthRefresh`, `AuthRevoke`, `AuthPasswordReset`,
  `OAuthAccessTokenRead`, `OAuthAccessTokenRevoke`, `OAuthClientRead`,
  `OAuthClientManage`, `PasswordResetRequest`, `PasswordResetConfirm`,
  `MfaEnroll`, `MfaVerify`, `NotifyEmailSend`, `NotifySmsSend`,
  `NotifyPushSend`, `NotifyInApp`, `NotifyVoice`, `NotifyWebhook`,
  `NotifyTemplateRead`, `NotifyTemplateWrite`, `NotifyBulkSend`,
  `PaymentCharge`, `PaymentRefund`, `PaymentStatus`, `PaymentMethodList`,
  `PaymentWebhook`, `PaymentSettlement`, `BankSlipGenerate`,
  `BankSlipApprove`, `FilesPut`, `FilesGet`, `FilesDelete`,
  `FilesSignedUrl`, `FilesCopy`, `FilesMove`, `FilesVisibilityChange`,
  `FilesLifecycle`, `IntegrationInvoke`,
  `IntegrationListCapabilities`, `IntegrationHealth`,
  `IntegrationConfigure`, `WebhookOut`, `PollingIn`, `LmsRosterSync`,
  `VideoSchedule`.

**Expected:**

`Capability::all()` must enumerate every variant of the
  `Capability` enum so consumers can iterate the full catalog.

**Evidence:**

`crates/cross-cutting/rbac/src/value_objects.rs:4765`
  final entry is `Self::OperationsSidebarReorder`; the enum continues
  with `AuthLogin` at line 1375, `NotifyEmailSend` at 1411,
  `PaymentCharge` at 1436, `FilesPut` at 1459, `IntegrationInvoke` at
  1480. `comm -23` between the enum variant set and the `all()` set
  shows exactly 46 missing items (all Phase 15 caps). Confirmed by
  `bash> comm -23 /tmp/enum_variants.txt /tmp/all_variants.txt | wc -l`
  returning `46`.

---

### FINDING 2 (id: `CROSSCUT-RBAC-002`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** Critical
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/rbac/src/services.rs:344-347

**Description:**

`DefaultRoleCatalog::super_admin()` is defined as
  `Capability::all().iter().copied().collect()`. Because `all()` is
  missing 46 variants (Finding 1), the `super_admin` set is missing
  every Auth, Notify, Payment, Files, and Integrations capability. The
  spec mandates: "The SuperAdmin role is a system role and cannot be
  deleted. It holds every registered `Capability` at the time of
  school creation."

**Expected:**

`docs/specs/rbac/permissions.md:84-86` — "The SuperAdmin
  role is a system role and cannot be deleted. It holds every
  registered Capability at the time of school creation and is
  refreshed on engine startup to pick up newly registered
  capabilities."

**Evidence:**

`crates/cross-cutting/rbac/src/services.rs:344-347`:
  ```rust
  #[must_use]
  pub fn super_admin() -> BTreeSet<Capability> {
      Capability::all().iter().copied().collect()
  }
  ```
  No explicit seeding of the 46 Phase 15 caps; relies on
  `Capability::all()`.

---

### FINDING 3 (id: `CROSSCUT-RBAC-003`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** Critical
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/rbac/src/services.rs:762-776

**Description:**

`DefaultRoleCatalog::school_admin()` adds a
  filter-on-`Capability::all()` for `Auth./Notify./Payment./Files./
  Integrations.` prefixed capabilities. Because `all()` is missing all
  46 Phase 15 variants (Finding 1), the filter is a no-op and the
  `school_admin` set has zero Auth/Notify/Payment/Files/Integrations
  capabilities — contradicting the spec mapping.

**Expected:**

`docs/specs/rbac/permissions.md:75` — SchoolAdmin row
  says "All Rbac.Role.*, all Rbac.Capability.*, all domain
  capabilities".

**Evidence:**

`crates/cross-cutting/rbac/src/services.rs:762-776`:
  ```rust
  s.extend(
      crate::value_objects::Capability::all()
          .iter()
          .copied()
          .filter(|c| {
              let s = c.as_str();
              s.starts_with("Settings.")
                  || s.starts_with("Operations.")
                  || s.starts_with("Auth.")
                  || s.starts_with("Notify.")
                  || s.starts_with("Payment.")
                  || s.starts_with("Files.")
                  || s.starts_with("Integrations.")
          }),
  );
  ```
  Filter depends on `Capability::all()`, which is missing the 46
  Phase 15 caps.

---

### FINDING 6 (id: `CROSSCUT-RBAC-006`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** Critical
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/rbac/src/commands.rs:1-193

**Description:**

17 of 22 RBAC commands declared in
  `docs/commands/rbac.md` are absent. Implemented: `CreateRole`,
  `UpdateRole`, `DeleteRole`, `AssignCapability`, `RevokeCapability`
  (5). Missing: `CloneRole`, `DeletePermissionAssignment`,
  `UpdatePermissionAssignment`, `CreateModulePermission`,
  `UpdateModulePermission`, `DeleteModulePermission`,
  `AssignModulePermission`, `RevokeModulePermission`, `GrantMenuLink`,
  `RevokeMenuLink`, `CreatePermissionSection`,
  `UpdatePermissionSection`, `DeletePermissionSection`,
  `ConfigureTwoFactor`, `TestTwoFactorDelivery`,
  `SetPermissionOverride`, `ClearPermissionOverride`.

**Expected:**

`docs/commands/rbac.md:11-34` — table of 22 commands.

**Evidence:**

`crates/cross-cutting/rbac/src/commands.rs:1-193`
  defines only the 5 phase-2 command structs. `grep -nE "CloneRole
  Command|DeletePermissionAssignmentCommand|UpdatePermissionAssign
  mentCommand|ConfigureTwoFactorCommand|SetPermissionOverrideCommand"
  crates/cross-cutting/rbac/src/commands.rs` returns no matches.

---

### FINDING 7 (id: `CROSSCUT-RBAC-007`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** Critical
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/rbac/src/events.rs:1-478

**Description:**

18 of 23 RBAC events declared in
  `docs/events/rbac.md` are absent. Implemented: `RoleCreated`,
  `RoleUpdated`, `RoleDeleted`, `CapabilityAssigned`,
  `CapabilityRevoked` (5). Missing: `RoleCloned`,
  `CapabilityRegistered`, `PermissionMetadataUpdated`,
  `PermissionAssignmentUpdated`, `ModulePermissionCreated`,
  `ModulePermissionUpdated`, `ModulePermissionDeleted`,
  `ModulePermissionAssigned`, `ModulePermissionRevoked`,
  `MenuLinkGranted`, `MenuLinkRevoked`, `PermissionSectionCreated`,
  `PermissionSectionUpdated`, `PermissionSectionDeleted`,
  `TwoFactorConfigured`, `TwoFactorDeliveryTested`,
  `PermissionOverrideSet`, `PermissionOverrideCleared`.

**Expected:**

`docs/events/rbac.md:9-33` — table of 23 events.

**Evidence:**

`crates/cross-cutting/rbac/src/events.rs:1-478`
  defines only the 5 phase-2 events. `grep -nE
  "RoleCloned|CapabilityRegistered|PermissionMetadataUpdated|
  ModulePermissionCreated|TwoFactorConfigured|PermissionOverrideSet"
  crates/cross-cutting/rbac/src/events.rs` returns no matches.

---

### FINDING 8 (id: `CROSSCUT-RBAC-008`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** Critical
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/rbac/src/services.rs:283-331

**Description:**

7 of 8 services declared in
  `docs/specs/rbac/services.md` are absent. Implemented:
  `CapabilityCheck` (named `CapabilityCheck` instead of
  `CapabilityCheckService` per spec) and a partial `RoleService`
  (missing `expand_with_inheritance`). Missing: `TwoFactorService`,
  `PermissionSectionService`, `MenuLinkService`,
  `ModulePermissionService`, `OverrideService`,
  `BootstrapService`, plus the two policies (`SystemRoleImmutability`,
  `SelfRevocationGuard`) and the two specifications
  (`RolesWithCapability`, `ActiveRoles`).

**Expected:**

`docs/specs/rbac/services.md:1-191` — 8 services + 2
  policies + 2 specifications.

**Evidence:**

`crates/cross-cutting/rbac/src/services.rs:1-1192`
  defines only `CapabilityCheck`, `InMemoryCapabilityCheck`,
  `RoleService`, `DefaultRoleCatalog`. `grep -nE
  "struct TwoFactorService|struct PermissionSectionService|struct
  MenuLinkService|struct ModulePermissionService|struct
  OverrideService|struct BootstrapService|struct
  SystemRoleImmutability|struct SelfRevocationGuard|struct
  RolesWithCapability|struct ActiveRoles"
  crates/cross-cutting/rbac/src/services.rs` returns no matches.

---

### FINDING 9 (id: `CROSSCUT-RBAC-009`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** Critical
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/rbac/src/repository.rs:1-162

**Description:**

6 of 10 RBAC repository port traits declared in
  `docs/specs/rbac/repositories.md` are absent. Implemented:
  `RoleRepository`, `AssignPermissionRepository`,
  `PermissionRepository`, `PermissionSectionRepository` (4). Missing:
  `ModulePermissionRepository`, `ModulePermissionAssignRepository`,
  `RolePermissionRepository`, `TwoFactorSettingRepository`,
  `PermissionOverrideRepository`, `TwoFactorDeliveryRepository`.

**Expected:**

`docs/specs/rbac/repositories.md:1-135` — 10 port
  traits.

**Evidence:**

`crates/cross-cutting/rbac/src/repository.rs:1-162`
  defines 4 traits. `grep -nE "trait
  ModulePermissionRepository|trait RolePermissionRepository|trait
  TwoFactorSettingRepository|trait PermissionOverrideRepository"
  crates/cross-cutting/rbac/src/repository.rs` returns no matches.

---

### FINDING 10 (id: `CROSSCUT-RBAC-010`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** docs/specs/rbac/value-objects.md:12-24

**Description:**

8 of 11 typed identifiers declared in the spec
  value-objects table are absent. The spec lists: `RoleId`,
  `CapabilityId`, `PermissionSectionId`, `AssignPermissionId`,
  `ModulePermissionId`, `ModulePermissionAssignId`, `RolePermissionId`,
  `TwoFactorSettingId`, `RoleBindingId`, `PermissionOverrideId`,
  `TwoFactorDeliveryId`. Code has: `RoleId`, `PermissionId` (renamed
  from `CapabilityId`), `PermissionSectionId`, `AssignPermissionId` (4
  of 11). Missing: `ModulePermissionId`, `ModulePermissionAssignId`,
  `RolePermissionId`, `TwoFactorSettingId`, `RoleBindingId`,
  `PermissionOverrideId`, `TwoFactorDeliveryId`.

**Expected:**

`docs/specs/rbac/value-objects.md:12-24` — table of
  11 typed identifiers.

**Evidence:**

`crates/cross-cutting/rbac/src/ids.rs:57-75` defines
  4 typed id structs. `grep -nE "ModulePermissionId|RolePermissionId|
  TwoFactorSettingId|PermissionOverrideId" crates/cross-cutting/rbac/
  src/ids.rs` returns no matches.

---

### FINDING 11 (id: `CROSSCUT-RBAC-011`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** docs/specs/rbac/value-objects.md:122-130

**Description:**

4 of 4 two-factor value objects from the spec are
  absent from `value_objects.rs`. Spec defines: `TwoFactorChannel`
  (`Sms | Email`), `TwoFactorMode` (present, but spec also requires
  `Required | Optional | Disabled` as the only variants),
  `TwoFactorExpiry` (`u32` seconds, 0..86400, typically 60..3600),
  `OtpCode` (4..10 digits), `OtpHash` (OtpCode after hashing).
  Code has only `TwoFactorMode` (with the correct variants but
  missing the `from_repr` constructor spec mentions) and is missing
  `TwoFactorChannel`, `TwoFactorExpiry`, `OtpCode`, `OtpHash`.

**Expected:**

`docs/specs/rbac/value-objects.md:122-130`.

**Evidence:**

`crates/cross-cutting/rbac/src/value_objects.rs:1-6206`
  defines 8 value objects; `TwoFactorChannel` and `TwoFactorExpiry`
  are not present. `grep -nE "TwoFactorChannel|TwoFactorExpiry|
  OtpCode|OtpHash" crates/cross-cutting/rbac/src/value_objects.rs`
  returns no matches (the literal `TwoFactor` appears only inside
  the comment at line 5762-5764).

---

### FINDING 12 (id: `CROSSCUT-RBAC-012`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** docs/specs/rbac/value-objects.md:96-112

**Description:**

11 of 12 permission-metadata value objects from
  the spec are absent. Spec defines: `PermissionName` (1..191 chars),
  `Route` (1..191 chars), `ParentRoute`, `PermissionType` (present),
  `LangName` (1..191 chars), `Icon` (up to 2000 chars), `Position`
  (`i32`), `RelateToChild` (`bool`), `IsMenu` (`bool`),
  `IsAdmin` (`bool`), `IsTeacher` (`bool`), `IsStudent` (`bool`),
  `IsParent` (`bool`), `IsAlumni` (`bool`), `AlternateModule`. Code
  has only `PermissionType` (encoded as enum with `Menu/SubMenu/
  Action` byte variants). Missing: `PermissionName`, `Route`,
  `ParentRoute`, `LangName`, `Icon`, `Position`, `RelateToChild`,
  `IsMenu`, `IsAdmin`, `IsTeacher`, `IsStudent`, `IsParent`,
  `IsAlumni`, `AlternateModule`.

**Expected:**

`docs/specs/rbac/value-objects.md:96-112` — table of
  12 permission-metadata types.

**Evidence:**

`grep -nE "struct PermissionName|struct Route|struct
  LangName|struct Icon|struct Position" crates/cross-cutting/rbac/
  src/value_objects.rs` returns no matches. The `Permission` struct
  in `aggregate.rs:97-126` carries only `lang_name: String` (no
  validation), `module: String`, `type_: PermissionType`, no
  `route`, `parent_route`, `icon`, `position`, `is_menu`,
  `is_admin`, `is_teacher`, `is_student`, `is_parent`, `is_alumni`,
  or `alternate_module` fields.

---

### FINDING 13 (id: `CROSSCUT-RBAC-013`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** docs/specs/rbac/value-objects.md:132-140

**Description:**

4 of 4 module value objects from the spec are
  absent. Spec defines: `ModuleName` (1..200 chars, unique within
  `school_id`), `DashboardId` (`u32`), `ModulePosition` (`i32`),
  `ModuleStatus` (`Active | Inactive`). Code has none.

**Expected:**

`docs/specs/rbac/value-objects.md:132-140`.

**Evidence:**

`grep -nE "struct ModuleName|struct DashboardId|
  struct ModulePosition|struct ModuleStatus" crates/cross-cutting/
  rbac/src/value_objects.rs` returns no matches.

---

### FINDING 15 (id: `CROSSCUT-RBAC-015`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** docs/specs/rbac/entities.md:1-150

**Description:**

All 15 RBAC entities declared in
  `docs/specs/rbac/entities.md` are absent (some are platform-side
  projections, some are owned by the RBAC domain). Spec entities:
  `RoleBinding`, `PermissionOverride`, `ModuleLinkBinding`,
  `CapabilityGrantEventProjection`, `RoleHierarchyEdge`,
  `TwoFactorDelivery`, `OtpCodeRow`, `DashboardSection`,
  `PermissionTranslation`, `RoleMembershipSnapshot`,
  `CapabilityCatalog`, `SidebarEntry`, `SidebarPosition`,
  `TwoFactorAuditEntry`, `CapabilitySearchIndex`. Code has only
  `AssignPermission` (not in the entities.md list, but in
  `aggregates.md`).

**Expected:**

`docs/specs/rbac/entities.md:1-150` — 15 entities
  documented.

**Evidence:**

`crates/cross-cutting/rbac/src/entities.rs:1-162`
  defines only `AssignPermission`. `grep -nE "RoleBinding|
  PermissionOverride|ModuleLinkBinding|TwoFactorDelivery" crates/
  cross-cutting/rbac/src/entities.rs` returns no matches (the
  comment at line 6-7 lists them as "land in later phases").

---

### FINDING 16 (id: `CROSSCUT-RBAC-016`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** docs/specs/rbac/permissions.md:19-63

**Description:**

6 of 9 permission groups declared in
  `docs/specs/rbac/permissions.md` have no corresponding
  `Capability` variants. The spec lists 9 groups: `Rbac.Role` (8
  caps), `Rbac.Capability` (4 caps), `Rbac.Section` (4 caps), `Rbac.
  ModulePermission` (6 caps), `Rbac.TwoFactor` (3 caps), `Rbac.
  Override` (3 caps). Code has 12 `Rbac.*` caps (`RbacRole*` +
  `RbacCapability*` + `RbacBootstrap`) but zero `Rbac.Section`,
  `Rbac.ModulePermission`, `Rbac.TwoFactor`, or `Rbac.Override`
  variants — the cap string forms `Rbac.Section.Create` etc. are
  not parseable.

**Expected:**

`docs/specs/rbac/permissions.md:19-63` — 9 permission
  groups and 30+ cap string forms.

**Evidence:**

`crates/cross-cutting/rbac/src/value_objects.rs:46-69`
  lists 12 `Rbac.*` variants. `grep -nE "RbacSection|RbacModule
  Permission|RbacTwoFactor|RbacOverride" crates/cross-cutting/rbac/
  src/value_objects.rs` returns no matches. The `from_str_opt`
  function at line 4772-5442 has no arm for any of the 20+ spec
  `Rbac.Section.* / Rbac.ModulePermission.* / Rbac.TwoFactor.* /
  Rbac.Override.*` strings.

---

### FINDING 17 (id: `CROSSCUT-RBAC-017`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** docs/specs/rbac/permissions.md:23-28

**Description:**

2 of 8 `Rbac.Role` cap variants from the spec are
  absent. Spec: `Rbac.Role.GrantMenu`, `Rbac.Role.RevokeMenu`. Code
  has `RbacRoleCreate/Read/Update/Delete/Manage/Clone` but not
  `RbacRoleGrantMenu` or `RbacRoleRevokeMenu`.

**Expected:**

`docs/specs/rbac/permissions.md:27-28` — "`Rbac.Role.
  GrantMenu`, `Rbac.Role.RevokeMenu`".

**Evidence:**

`crates/cross-cutting/rbac/src/value_objects.rs:46-69`
  lists 6 RbacRole caps. `grep -nE "RbacRoleGrantMenu|RbacRoleRevoke
  Menu" crates/cross-cutting/rbac/src/value_objects.rs` returns no
  matches. The commands in `commands.md:26-27` (`GrantMenuLink`,
  `RevokeMenuLink`) reference these cap strings.

---

### FINDING 18 (id: `CROSSCUT-RBAC-018`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** docs/specs/rbac/workflows.md:79-92

**Description:**

Two-Factor Enrollment Workflow (workflows.md
  § 4) is unimplemented. The workflow describes 7 steps:
  `SchoolAdmin updates TwoFactorSetting` → `TwoFactorConfigured`
  emitted → 2FA prompted on login → OTP via channel → user enters
  OTP → session granted → `TwoFactorDeliveryTested` emitted. The
  `ConfigureTwoFactor` command, the `TwoFactorSetting` aggregate,
  the `TwoFactorConfigured` event, the `TwoFactorDeliveryTested`
  event, and the `TwoFactorService` are all absent (see Findings
  5, 6, 7, 8).

**Expected:**

`docs/specs/rbac/workflows.md:79-92`.

**Evidence:**

`grep -rnE "TwoFactorConfigured|TwoFactorDelivery
  Tested|ConfigureTwoFactor" crates/cross-cutting/rbac/src/`
  returns no matches. `grep -nE "TwoFactor" crates/cross-cutting/
  rbac/src/` returns only one comment reference at
  `value_objects.rs:5762`.

---

### FINDING 19 (id: `CROSSCUT-RBAC-019`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** docs/specs/rbac/workflows.md:115-126

**Description:**

Role Cloning Workflow is unimplemented. The
  workflow requires `CloneRole` command + `RoleCloned` event + the
  ability to copy `AssignPermission`, `RolePermission`, and
  `ModulePermissionAssign` rows. None of these are present (see
  Findings 6, 7, 9).

**Expected:**

`docs/specs/rbac/workflows.md:115-126` — 8-step
  workflow ending in `RoleCloned` emit.

**Evidence:**

`grep -nE "CloneRole|RoleCloned" crates/cross-
  cutting/rbac/src/` returns no matches. The `Capability::RbacRole
  Clone` variant exists at `value_objects.rs:58` but no command or
  event uses it.

---

### FINDING 20 (id: `CROSSCUT-RBAC-020`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** docs/specs/rbac/workflows.md:128-141

**Description:**

Override Workflow is unimplemented. The workflow
  requires `SetPermissionOverride` command, `PermissionOverrideSet`
  event, `PermissionOverride` aggregate, `OverrideService`, and
  `PermissionOverrideRepository` — all absent (see Findings 5, 6, 7,
  8, 9).

**Expected:**

`docs/specs/rbac/workflows.md:128-141` — 6-step
  workflow.

**Evidence:**

`grep -rnE "SetPermissionOverride|PermissionOverride
  Set|PermissionOverride" crates/cross-cutting/rbac/src/`
  returns only one reference at
  `services.rs:29` (a comment about a Phase 2 follow-up).

---

### FINDING 21 (id: `CROSSCUT-RBAC-021`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** docs/specs/rbac/workflows.md:143-153

**Description:**

Menu Visibility Workflow is unimplemented. The
  workflow requires `GrantMenuLink` / `RevokeMenuLink` commands,
  `MenuLinkGranted` / `MenuLinkRevoked` events, `RolePermission`
  aggregate, `MenuLinkService`, `RolePermissionRepository` — all
  absent (see Findings 5, 6, 7, 8, 9).

**Expected:**

`docs/specs/rbac/workflows.md:143-153` — 5-step
  workflow.

**Evidence:**

`grep -rnE "GrantMenuLink|RevokeMenuLink|MenuLink"
  crates/cross-cutting/rbac/src/` returns no matches. The
  `Capability::RbacRoleGrantMenu` / `RbacRoleRevokeMenu` cap
  variants are also absent (Finding 17).

---

### FINDING 22 (id: `CROSSCUT-RBAC-022`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** docs/specs/rbac/workflows.md:6-31

**Description:**

School Bootstrap Workflow is unimplemented. The
  workflow describes 7 steps: create school → bootstrap
  SuperAdmin role → seed every Capability → create first user →
  create default PermissionSection list → create default
  TwoFactorSetting → seed baseline ModulePermissions. The crate
  has no `BootstrapService` (Finding 8) and no command handler
  implementation, so the bootstrap path is not wired.

**Expected:**

`docs/specs/rbac/workflows.md:6-31` — 7-step
  workflow with pre-conditions and failure paths.

**Evidence:**

`grep -rnE "BootstrapService|seed_role_catalog|
  default_two_factor_setting" crates/cross-cutting/rbac/src/`
  returns no matches. The `PHASE-2-HANDOFF.md:130-131` claims
  the 10 default role constructors are shipped, but does not
  describe a BootstrapService that wires them into a new school.

---

### FINDING 23 (id: `CROSSCUT-RBAC-023`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** docs/specs/rbac/commands.md:69-82

**Description:**

`CloneRoleCommand` struct is absent despite
  `RbacRoleClone` capability and `RoleCloned` event both being
  documented. The `Capability::RbacRoleClone` variant exists
  (value_objects.rs:58) but no command consumes it.

**Expected:**

`docs/specs/rbac/commands.md:69-82` — full
  `CloneRoleCommand` struct definition.

**Evidence:**

`grep -nE "CloneRoleCommand|struct CloneRole"
  crates/cross-cutting/rbac/src/commands.rs` returns no matches.

---

### FINDING 24 (id: `CROSSCUT-RBAC-024`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** docs/specs/rbac/commands.md:120-148

**Description:**

`DeletePermissionAssignmentCommand` and
  `UpdatePermissionAssignmentCommand` structs are absent. The
  spec defines both with their own `Capability` requirements
  and effects. The hard-delete vs. soft-denial distinction
  is collapsed into a single `as_denial: bool` flag on
  `RevokeCapabilityCommand` (commands.rs:81-91), which conflates
  two distinct user-intents from the spec.

**Expected:**

`docs/specs/rbac/commands.md:120-148` — two
  separate commands.

**Evidence:**

`grep -nE "DeletePermissionAssignmentCommand|Update
  PermissionAssignmentCommand" crates/cross-cutting/rbac/src/
  commands.rs` returns no matches.

---

### FINDING 25 (id: `CROSSCUT-RBAC-025`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** docs/specs/rbac/commands.md:241-275

**Description:**

`ConfigureTwoFactorCommand` and
  `TestTwoFactorDeliveryCommand` structs are absent despite
  the `Rbac.TwoFactor.Configure` capability being documented
  (spec permissions.md:54-57).

**Expected:**

`docs/specs/rbac/commands.md:241-275` — full
  command struct definitions including the per-role
  `TwoFactorMode` fields and the test-delivery recipient.

**Evidence:**

`grep -nE "ConfigureTwoFactorCommand|TestTwoFactor
  DeliveryCommand" crates/cross-cutting/rbac/src/commands.rs`
  returns no matches.

---

### FINDING 26 (id: `CROSSCUT-RBAC-026`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** docs/specs/rbac/commands.md:279-307

**Description:**

`SetPermissionOverrideCommand` and
  `ClearPermissionOverrideCommand` structs are absent. The
  spec defines them with `OverrideReason`, `expires_at: Option
  <Timestamp>`, and `PermissionOverrideId` typed identifiers.

**Expected:**

`docs/specs/rbac/commands.md:279-307` — full
  command struct definitions.

**Evidence:**

`grep -nE "SetPermissionOverrideCommand|Clear
  PermissionOverrideCommand" crates/cross-cutting/rbac/src/
  commands.rs` returns no matches.

---

### FINDING 4 (id: `CROSSCUT-RBAC-004`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/rbac/src/services.rs:1088-1093,
  crates/cross-cutting/rbac/tests/rbac_e2e.rs:58-64

**Description:**

`super_admin_role_includes_every_capability` test
  passes by symmetry: both the test assertion and the
  `super_admin()` impl iterate `Capability::all()`. Because `all()` is
  missing 46 variants (Finding 1), the test only confirms the
  (truncated) 608 are in the (truncated) 608 — it does not confirm
  the spec invariant that "SuperAdmin holds every registered
  Capability".

**Expected:**

A test that asserts `super_admin` contains every
  variant of the `Capability` enum, not just every entry in `all()`.

**Evidence:**

`crates/cross-cutting/rbac/src/services.rs:1088-1093`:
  ```rust
  #[test]
  fn super_admin_role_includes_every_capability() {
      let all: BTreeSet<Capability> = DefaultRoleCatalog::super_admin();
      for c in Capability::all() {
          assert!(all.contains(c), "missing capability {c:?} in super_admin");
      }
  }
  ```
  Identical pattern in `crates/cross-cutting/rbac/tests/rbac_e2e.rs:58-64`.

---

### FINDING 5 (id: `CROSSCUT-RBAC-005`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/rbac/src/aggregate.rs:1-7,
  docs/specs/rbac/aggregates.md:189-320

**Description:**

5 of 8 RBAC aggregates declared in the spec are
  absent from `aggregate.rs` (or anywhere in the crate). The spec
  declares: `ModulePermission`, `ModulePermissionAssign`,
  `RolePermission`, `TwoFactorSetting`, `PermissionOverride`. The
  crate's own `lib.rs` docstring acknowledges this: "The five
  secondary RBAC aggregates (`TwoFactorSetting`, `Override`,
  `ModulePermission`, `ModulePermissionAssign`, `RolePermission`)
  land in later phases."

**Expected:**

`docs/specs/rbac/aggregates.md` lines 189-320 list 5
  secondary aggregates; each has a spec-defined Commands, Events,
  and Invariants section.

**Evidence:**

`crates/cross-cutting/rbac/src/aggregate.rs:1-7`
  module docstring confirms only `Role`, `Permission`,
  `PermissionSection` are in scope. `grep -nE "struct
  ModulePermission|struct RolePermission|struct TwoFactorSetting|struct
  PermissionOverride" crates/cross-cutting/rbac/src/` returns no
  matches.

---

### FINDING 14 (id: `CROSSCUT-RBAC-014`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** docs/specs/rbac/value-objects.md:39-44

**Description:**

4 of 4 spec value-object categories are absent
  from `value_objects.rs`: `CapabilityString` (newtype around
  `String` with validated construction; spec line 167-177),
  `CapabilityAction` (verb in present tense enum; spec line 42),
  `CapabilityScope` (`Tenant | System`; spec line 43), and the
  `RoleStatus` / `RoleNamePatch` value objects from the Role
  category (spec line 32-33).

**Expected:**

`docs/specs/rbac/value-objects.md:32-44` and 167-177.

**Evidence:**

`grep -nE "struct CapabilityString|enum
  CapabilityAction|enum CapabilityScope|enum RoleStatus|struct
  RoleNamePatch" crates/cross-cutting/rbac/src/value_objects.rs`
  returns no matches. The `Capability` enum has a `domain()` and
  `aggregate()` method but no `action()` enumeration as a typed
  enum (the `action()` method returns `&'static str`).

---

### FINDING 27 (id: `CROSSCUT-RBAC-027`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** docs/specs/rbac/value-objects.md:14-15

**Description:**

The spec calls the storage-row id `CapabilityId`
  (`Id<Capability>`), but the code names it `PermissionId`. This
  is a doc-vs-code drift; consumers reading the spec expect
  `CapabilityId` and the public API exposes `PermissionId`.

**Expected:**

`docs/specs/rbac/value-objects.md:15` — "`CapabilityId`
  | `Id<Capability>` | A permission row (a capability)".

**Evidence:**

`crates/cross-cutting/rbac/src/ids.rs:62-65`:
  ```rust
  rbac_typed_id! {
      /// A typed id for a [`Permission`](crate::aggregate::Permission) row.
      pub struct PermissionId;
  }
  ```
  The struct is named `PermissionId`, not `CapabilityId`.

---

### FINDING 28 (id: `CROSSCUT-RBAC-028`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/rbac/src/value_objects.rs:1110

**Description:**

The `Permission` struct stores `lang_name: String`
  with no validation, despite the spec requiring `LangName` to be
  1..191 chars (spec value-objects.md:103). The associated value
  object `LangName` is not implemented (Finding 12).

**Expected:**

`docs/specs/rbac/value-objects.md:103` — "`LangName` |
  1..191 chars, the i18n key".

**Evidence:**

`crates/cross-cutting/rbac/src/aggregate.rs:108-109`:
  ```rust
  /// Localized display name (i18n key, not the translated text).
  pub lang_name: String,
  ```
  No length check at construction; the `Permission` struct uses
  `String` rather than a typed `LangName` newtype.

---

### FINDING 29 (id: `CROSSCUT-RBAC-029`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/rbac/src/commands.rs:27-37

**Description:**

The `CreateRoleCommand` does not validate that
  `role_type == System` requires the `RbacRoleManage` capability.
  The spec mandates "system roles require `Rbac.Role.Manage`" but
  the validation is pushed to the (non-existent) command handler.
  The struct itself accepts any `RoleType` regardless of caller
  capability.

**Expected:**

`docs/specs/rbac/commands.md:27-30` — "Pre-conditions:
  ... `role_type` is allowed for the actor (system roles require
  `Rbac.Role.Manage`)."

**Evidence:**

`crates/cross-cutting/rbac/src/commands.rs:27-37`
  has no validation method; the struct is a passive data carrier.
  `grep -n "validate\|RbacRoleManage" crates/cross-cutting/rbac/
  src/commands.rs` returns no matches.

---

### FINDING 30 (id: `CROSSCUT-RBAC-030`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/rbac/src/services.rs:199-213

**Description:**

The bootstrap backstop in
  `apply_bootstrap_backstop` grants every `Rbac.*` capability to
  any actor with `RbacRoleManage` (or any system actor), including
  the `RbacBootstrap` capability. The spec self-revocation guard
  (permissions.md:148-156) requires that the engine refuse any
  revocation that would leave the actor without the capability
  required to undo the command. There is no `SelfRevocationGuard`
  policy / spec implementation (Finding 8).

**Expected:**

`docs/specs/rbac/permissions.md:146-157` —
  "Self-Revocation Guard" section.

**Evidence:**

`crates/cross-cutting/rbac/src/services.rs:199-213`
  hard-codes the backstop. `grep -nE "SelfRevocationGuard|self_revo
  cation" crates/cross-cutting/rbac/src/` returns only
  `errors.rs:78-83` (an unused error constructor; no enforcement).

---

### FINDING 33 (id: `CROSSCUT-RBAC-033`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/rbac/src/commands.rs:55-61

**Description:**

`DeleteRoleCommand` does not check
  `RoleService::can_delete` at construction. The pre-condition
  (non-system, no user bindings) is delegated to the (non-
  existent) command handler. The struct carries no validation
  method, so any caller can submit a `DeleteRoleCommand` for a
  system role.

**Expected:**

`docs/specs/rbac/commands.md:62-65` — "Pre-
  conditions: Role is not of type System. No users are bound to
  the role (the platform domain reports the count)."

**Evidence:**

`crates/cross-cutting/rbac/src/commands.rs:55-61`:
  ```rust
  pub struct DeleteRoleCommand {
      pub tenant: TenantContext,
      pub role_id: RoleId,
  }
  ```
  No `validate(&self, role: &Role) -> Result<()>` method, no
  reference to `RoleService::can_delete`. The validation helper
  exists in `services.rs:312-320` but is only invoked from
  tests, never from the command.

---

### FINDING 34 (id: `CROSSCUT-RBAC-034`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/rbac/tests/rbac_e2e.rs:1-566

**Description:**

19 integration tests exist but none of them
  test the actual command-handler path, the event-bus
  subscription, the storage adapter, or the
  `super_admin`-vs-`Capability::all()` invariant
  (Finding 1). The 5 Phase 15 cap-roundtrip tests
  (`tests/auth_caps.rs`, `tests/notify_caps.rs`,
  `tests/payment_caps.rs`, `tests/files_caps.rs`,
  `tests/integrations_caps.rs`) verify the variants exist in
  the enum and parse, but never check that they appear in
  `Capability::all()`.

**Expected:**

Tests that exercise command handlers, the event
  bus, the storage adapter, and a `super_admin`-covers-`all
  Capability` invariant that does not iterate the broken
  `all()` (see Finding 4).

**Evidence:**

`grep -nE "Command|Handler|EventBus|Storage"
  crates/cross-cutting/rbac/tests/rbac_e2e.rs` returns no
  matches for any handler invocation. The 5 phase-15 cap tests
  iterate a hard-coded `const *_VARIANTS: &[Capability]` array
  rather than `Capability::all()`.

---

### FINDING 35 (id: `CROSSCUT-RBAC-035`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** docs/coverage.toml:395-411

**Description:**

The coverage matrix has 2 rbac rows
  (`rbac_roles_aggregate`, `rbac_capabilities_aggregate`). It
  does not represent the spec surface of 8 tables (Finding
  36), 22 commands (Finding 6), 23 events (Finding 7), 8
  services (Finding 8), or 10 repository traits (Finding 9).
  The Phase 2 hand-off says "2 `coverage.toml` rows flipped"
  but the spec folder defines a much larger surface that the
  matrix does not enumerate.

**Expected:**

Per-table / per-command / per-event / per-service
  / per-repository coverage rows.

**Evidence:**

`grep -nE "rbac_(section|module|menu|override|
  two_factor|permission_assignment)" docs/coverage.toml` returns
  no matches. The only `rbac_*` rows are at lines 396 and 405.

---

### FINDING 36 (id: `CROSSCUT-RBAC-036`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** docs/specs/rbac/tables.md:7-16

**Description:**

`docs/specs/rbac/tables.md` lists 8 tables:
  `assign_permissions`, `permissions`, `permission_sections`,
  `roles`, `rbac_module_permissions`,
  `rbac_module_permission_assigns`, `rbac_role_permissions`,
  `two_factor_settings`. The codebase implements 4 (the
  3 aggregates + 1 entity from Finding 5); the 4 secondary
  tables (`rbac_module_permissions`,
  `rbac_module_permission_assigns`, `rbac_role_permissions`,
  `two_factor_settings`) have no DDL or repository. The PHASE
  -2-HANDOFF.md:296-299 acknowledges the deferral ("Do NOT
  add the 5 secondary RBAC aggregates... Phase 2's `docs/
  build-plan.md` § 'Phase 3' only requires the academic
  domain") but the spec doc remains the source of truth and
  is non-conformant.

**Expected:**

`docs/specs/rbac/tables.md:7-16` — 8 tables.

**Evidence:**

`grep -nE "rbac_module_permissions|rbac_role_permis
  sions|two_factor_settings" crates/cross-cutting/rbac/src/`
  returns no matches.

---

### FINDING 31 (id: `CROSSCUT-RBAC-031`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** Low
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/rbac/src/services.rs:42-54

**Description:**

The `CapabilityExplanation` struct uses
  `pub overrides: Vec<CapabilityOverride>` (a Phase 2 stub with
  `id: Uuid, granted: bool`) but the spec defines
  `overrides: Vec<PermissionOverrideId>` (a typed id). The struct
  is intended to be the wire contract for the audit log and the
  "why is this denied?" diagnostic screen, and the divergence
  means the explanation payload is non-conformant.

**Expected:**

`docs/specs/rbac/services.md:53-56` — "`pub overrides:
  Vec<PermissionOverrideId>`".

**Evidence:**

`crates/cross-cutting/rbac/src/services.rs:42-54`:
  ```rust
  pub struct CapabilityExplanation {
      pub capability: Capability,
      pub decision: bool,
      pub role_grants: Vec<RoleId>,
      pub overrides: Vec<CapabilityOverride>,
      pub system_fallback: bool,
  }
  ```
  `overrides` carries `CapabilityOverride` (id+granted bool), not
  `PermissionOverrideId`.

---

### FINDING 32 (id: `CROSSCUT-RBAC-032`)

- **Source:** `docs/audit_reports/findings/wave2-rbac.md`
- **Severity:** Low
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/rbac/src/services.rs:162-180

**Description:**

`InMemoryCapabilityCheck::grants_for` comments
  acknowledge that "the storage-backed impl will read the
  user→role bindings" but the in-memory implementation
  enumerates ALL roles in the school, not the actor's bound
  roles. This means a Teacher in a school where any role holds
  `RbacRoleManage` will be reported as having `RbacBootstrap`
  (and every other Rbac.* cap). The `rbac_bootstrap_is_never_
  revocable` test at rbac_e2e.rs:111-151 works around this by
  using a fresh `other_school` to verify the deny path.

**Expected:**

Per-role binding lookup, not school-wide sum.

**Evidence:**

`crates/cross-cutting/rbac/src/services.rs:162-180`:
  ```rust
  // The Phase 2 in-memory check accepts a single role id via
  // the session. For now we just sum all roles in the school
  // — the storage-backed impl will read the user→role
  // bindings.
  let mut caps = BTreeSet::new();
  for set in by_school.values() {
      caps.extend(set.iter().copied());
  }
  ```
  No `actor.role_ids` filter; the test at rbac_e2e.rs:143-150
  explicitly creates an `other_school` to avoid this bug.

---


## Operations (target id prefix: `CROSSCUT-OPS`)

**Path:** `crates/cross-cutting/operations/`  
**Total findings:** 56 (5 critical, 23 high, 17 medium, 11 low)


### FINDING 1 (id: `CROSSCUT-OPS-001`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Critical
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/lib.rs:24` and `crates/cross-cutting/operations/src/aggregate.rs:9,  commands.rs:10-11,  entities.rs:6,  events.rs:8-9,  query.rs:6-7,  repository.rs:9-10,  services.rs:7-8,  value_objects.rs:6-7`

**Description:**

The crate-level `#![deny(missing_docs)]` at `lib.rs:24` is silently shadowed by `#![allow(missing_docs)]` at the top of every other source file. The compiler accepts the inner allows, so the deny has no effect anywhere in the crate. Every public item in `aggregate.rs`, `commands.rs`, `entities.rs`, `events.rs`, `query.rs`, `repository.rs`, `services.rs`, and `value_objects.rs` is published without rustdoc.

**Expected:**

Public items in the operations crate are documented with rustdoc per `AGENTS.md` and `docs/code-standards.md` ("All public APIs are documented with rustdoc; `#![deny(missing_docs)]`").

**Evidence:**

`crates/cross-cutting/operations/src/lib.rs:24` `#![deny(missing_docs)]` vs. `crates/cross-cutting/operations/src/aggregate.rs:9` `#![allow(missing_docs, dead_code, clippy::all)]` (and the matching allow lines on the other 7 source files).

---

### FINDING 14 (id: `CROSSCUT-OPS-014`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Critical
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/events.rs` and `docs/specs/operations/events.md` (no `UserLogDeleted` listed)

**Description:**

The spec at `docs/specs/operations/workflows.md:72` says the nightly purge is "logged as a `DeleteUserLog` event for compliance." Neither `docs/specs/operations/events.md` (events catalog) nor `crates/cross-cutting/operations/src/events.rs` defines a `UserLogDeleted` / `DeleteUserLog` event. The compliance audit trail for the per-tenant `UserLog` retention sweep is therefore not modeled.

**Expected:**

A `UserLogDeleted` event in `events.md` and `events.rs` (with `log_id`, `school_id`, `actor_id`, `purged_at`, etc.) that the retention job emits on every deleted row.

**Evidence:**

`docs/specs/operations/workflows.md:64-73` "User Log Retention Workflow" - step 6: "The purge is logged as a DeleteUserLog event for compliance." `docs/specs/operations/events.md:228-242` lists `UserLogged` only. `crates/cross-cutting/operations/src/events.rs:972-1036` defines `UserLogged` only (no `UserLogDeleted` struct, no `EVENT_TYPE` for `"operations.user_log.deleted"`). `crates/cross-cutting/operations/src/commands.rs:307-336` defines `RecordUserLogCommand` only (no `DeleteUserLogCommand`).

---

### FINDING 2 (id: `CROSSCUT-OPS-002`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Critical
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/aggregate.rs:39-56` (`Backup`), `:142-159` (`Job`), `:276-293` (`FailedJob`), `:342-355` (`SystemVersion`), `:439-451` (`VersionHistory`), `:490-507` (`UserLog`), `:551-568` (`MaintenanceSetting`), `:659-681` (`Sidebar`)

**Description:**

None of the 8 root aggregates has a `delete` (or `soft_delete`) method. The spec defines delete commands for `Backup` (`DeleteBackupCommand`), `FailedJob` (`DeleteFailedJobCommand`), `MaintenanceSetting` (implicit in `ConfigureMaintenanceCommand` reconfig flow), and `Sidebar` (`DeleteSidebarEntryCommand`), and the spec says "the engine refuses to delete system-defined sidebar rows" - a rule that can only be enforced by a `delete` method on the aggregate. The Phase 14 hand-off acknowledges "all root aggregates set `active_status = false` on delete" but no aggregate has the method to do it.

**Expected:**

Each root aggregate (other than the append-only `UserLog` / `VersionHistory`) exposes a `delete(actor, at, event_id)` (or `soft_delete`) method that sets `active_status = false` and emits the corresponding event.

**Evidence:**

Spec at `docs/specs/operations/aggregates.md:32` lists `DeleteBackup`, `:115` `DeleteFailedJob`, `:284` `DeleteSidebarEntry`. `crates/cross-cutting/operations/src/aggregate.rs:39-56` (`Backup` struct + impl block, no `delete`); `:142-159` (`Job`, no `delete`); `:659-781` (`Sidebar`, only `new`, `is_system`, `reorder`, `set_ignore`, `set_active` - no `delete`).

---

### FINDING 29 (id: `CROSSCUT-OPS-029`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Critical
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/query.rs:15-153` (8 query stubs)

**Description:**

All 8 query stubs in `query.rs` are empty structs with only a `Default` impl and a `new()` constructor; none of them uses the `#[derive(DomainQuery)]` macro, none of them references the 15 tables in `docs/specs/operations/tables.md`, and none of them has a field, a `where_has` clause, or a `with` clause. The tables listed in `tables.md` are: `failed_jobs`, `jobs`, `operations_maintenance_settings`, `migrations`, `oauth_access_tokens`, `oauth_auth_codes`, `oauth_clients`, `oauth_personal_access_clients`, `oauth_refresh_tokens`, `password_resets`, `rbac_sidebars`, `operations_backups`, `operations_system_versions`, `operations_user_logs`, `operations_version_histories` (15 rows) - none is emitted by the `DomainQuery` macro from this crate. The crate does not depend on `educore-query-derive`.

**Expected:**

At least the 5 owned-aggregate tables (`operations_backups`, `jobs`, `failed_jobs`, `operations_system_versions`, `operations_version_histories`, `operations_user_logs`, `operations_maintenance_settings`, `rbac_sidebars`) have `#[derive(DomainQuery)]` structs with typed fields and a `with`/`.active()` style API.

**Evidence:**

`docs/specs/operations/tables.md:7-23` (15 table rows). `crates/cross-cutting/operations/src/query.rs:15-153` (8 empty struct stubs). `crates/cross-cutting/operations/Cargo.toml:13-27` does not include `educore-query-derive`. `crates/cross-cutting/operations/src/` has zero `#[derive(DomainQuery)]` attributes (grep returns no matches).

---

### FINDING 30 (id: `CROSSCUT-OPS-030`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Critical
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/tests/` (does not exist)

**Description:**

There is no `crates/cross-cutting/operations/tests/` directory; the operations crate has zero integration tests in the conventional cargo location. The 47 unit tests in the source files pass, but the engine's validation rule ("At least one integration test per PR") and the "9-file layout" template require a `tests/` directory for the crate. The only operations integration tests are in `crates/tools/storage-parity/tests/operations_integration.rs` (which is in a different crate).

**Expected:**

A `crates/cross-cutting/operations/tests/` directory with at least one integration test file (e.g. `tests/integration.rs`) covering command to aggregate to event flow.

**Evidence:**

`find /home/beznet/Workspace/smscore/crates/cross-cutting/operations -type d` returns only `crates/cross-cutting/operations` and `crates/cross-cutting/operations/src` (no `tests/`). `crates/tools/storage-parity/tests/operations_integration.rs:1-194` is the only operations-integration test file in the workspace.

---

### FINDING 10 (id: `CROSSCUT-OPS-010`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/services.rs:362-372` (`MaintenanceService::applies_to_role`)

**Description:**

The spec signature at `docs/specs/operations/services.md:96` is `pub fn applies_to_role(setting: &MaintenanceSetting, role: &Role) -> bool`; the code at `services.rs:362-372` takes `role_label: &str` and string-matches against `setting.applicable_for`. The `Role` struct is owned by the `educore-rbac` domain and the spec explicitly notes the cross-domain binding.

**Expected:**

`pub fn applies_to_role(setting: &MaintenanceSetting, role: &Role) -> bool` (with a local `Role` mirror or via the rbac re-export).

**Evidence:**

`docs/specs/operations/services.md:96` `pub fn applies_to_role(setting: &MaintenanceSetting, role: &Role) -> bool { ... }`. `crates/cross-cutting/operations/src/services.rs:362` `pub fn applies_to_role(setting: &MaintenanceSetting, role_label: &str) -> bool { if setting.applicable_for.is_all() { return true; } setting.applicable_for.as_str().split(',').map(str::trim).any(|s| s == role_label) }`.

---

### FINDING 11 (id: `CROSSCUT-OPS-011`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/services.rs:418-482` (`SidebarService`)

**Description:**

The spec at `docs/specs/operations/services.md:108` defines `pub fn tree(entries: &[Sidebar], role: RoleId) -> Vec<SidebarNode>` returning a tree of `SidebarNode` values. The code defines only `pub fn tree_order(entries: &[Sidebar]) -> Vec<(crate::value_objects::SidebarId, i32)>` returning a flat `Vec<(id, level)>`. Both the `tree` method and the `SidebarNode` struct are absent.

**Expected:**

A `SidebarNode` struct (with `id`, `level`, `children: Vec<SidebarNode>` or similar) and a `tree` method that builds the hierarchical projection for a role.

**Evidence:**

`docs/specs/operations/services.md:108-111` `pub fn tree(entries: &[Sidebar], role: RoleId) -> Vec<SidebarNode> { ... }` (and 3 sub-methods). `crates/cross-cutting/operations/src/services.rs:426` `pub fn tree_order(entries: &[Sidebar]) -> Vec<(crate::value_objects::SidebarId, i32)> { ... }`. No `tree` method, no `SidebarNode` struct in `entities.rs` or `services.rs`.

---

### FINDING 12 (id: `CROSSCUT-OPS-012`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/services.rs:567-637` (policies and specifications)

**Description:**

The spec at `docs/specs/operations/services.md:130-205` defines `Policy<Cmd>` and `Specification<T>` traits and gives the `OneRestoreInProgress`, `MaintenanceLockout`, and `DisableMaintenanceGuard` policies plus `ActiveBackups`, `DatabaseBackups`, `SuccessfulLogins`, and `FailedLogins` specifications as implementations of those traits. The code declares all seven as zero-sized unit structs with free `check` / `is_satisfied_by` functions, not as trait impls. A consumer that wants to call a generic policy dispatcher (e.g. `policy_registry.dispatch::<RestoreBackupCommand>(cmd)`) cannot do so.

**Expected:**

`pub trait Policy<C: Command> { type Outcome; fn check(&self, ctx, cmd) -> Outcome; }` and `pub trait Specification<T> { fn is_satisfied_by(&self, t: &T) -> bool; }`, with the seven concrete types implementing the traits.

**Evidence:**

`docs/specs/operations/services.md:131-137` `impl Policy<RestoreBackupCommand> for OneRestoreInProgress`; `:172-174` `impl Specification<Backup> for ActiveBackups`. `crates/cross-cutting/operations/src/services.rs:567-579` (`OneRestoreInProgress` with free `check` function, no `impl Policy`); `:646-654` (`ActiveBackups` with free `is_satisfied_by`, no `impl Specification`).

---

### FINDING 13 (id: `CROSSCUT-OPS-013`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/services.rs:620-637` (`DisableMaintenanceGuard`)

**Description:**

The spec at `docs/specs/operations/permissions.md:124-129` and `services.md:153-162` says the guard "refuses to disable maintenance for the last remaining `SuperAdmin` in a school." The code at `services.rs:620-637` checks the actor's role label only (`super_admin` / `school_admin` / case-insensitive variants) and never checks the count of remaining `SuperAdmin` actors. The "self-authorization" semantic is missing.

**Expected:**

A second-actor check: if the only remaining `SuperAdmin` for the school issues `DisableMaintenance`, the policy returns `Deny`.

**Evidence:**

`docs/specs/operations/permissions.md:124-128` "The engine refuses to disable maintenance for the last remaining SuperAdmin in a school. A DisableMaintenance command from a non-SuperAdmin while maintenance is enabled is rejected with ForbiddenError::MaintenanceLockout." `crates/cross-cutting/operations/src/services.rs:620-637` `pub fn check(actor_role_label: &str) -> Result<(), String> { if actor_role_label.eq_ignore_ascii_case("super_admin") || ... { Ok(()) } else { Err(...) } }` - no count check, no school id parameter.

---

### FINDING 15 (id: `CROSSCUT-OPS-015`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/services.rs:561-637, 645-705` (policies and specifications) and `crates/cross-cutting/operations/src/lib.rs:26-34` (module surface)

**Description:**

No `Policy` or `Specification` trait is declared in the crate (see Finding 12) and no subscriber or dispatcher is wired to enforce the `OneRestoreInProgress` policy on `RestoreBackupCommand`, the `MaintenanceLockout` policy on `LoginCommand`, or the `DisableMaintenanceGuard` policy on `DisableMaintenanceCommand`. The seven unit-struct policies are never invoked from any handler.

**Expected:**

A `services::dispatch_policies` module (or `educore-core`-level policy registry) that wires each policy to the matching command handler.

**Evidence:**

`docs/specs/operations/permissions.md:91-97` "Capabilities are checked at the command boundary: `if !engine.rbac().has(actor_id, Capability::OperationsBackupRestore).await? { return Err(DomainError::forbidden(...)) }`." `crates/cross-cutting/operations/src/services.rs:567-637` declares the policies but no handler calls them; no `mod dispatch` or `mod handler` exists in `lib.rs:26-34`.

---

### FINDING 16 (id: `CROSSCUT-OPS-016`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/commands.rs:307-336` (`RecordUserLogCommand`) vs `docs/specs/operations/commands.md:225-235`

**Description:**

`RecordUserLogCommand` in the code carries an extra `pub academic_id: Option<AcademicYearRef>` field that the spec does not declare. The spec's `RecordUserLogCommand` lists `tenant`, `user_id`, `role_id`, `ip_address`, `user_agent`, `outcome`, `failure_reason` - 7 fields. The code adds `academic_id` as the 8th. Per the engine's typed-wrapper rule, an extra undocumented field is a drift; the spec at `docs/specs/operations/aggregates.md:206-207` (UserLog invariant 7) talks about `UserLog::academic_id` so the field is required at the aggregate level, but the command shape is supposed to be the wire form.

**Expected:**

Either the spec is updated to list `academic_id` in the `RecordUserLogCommand` struct literal, or the command is left at the spec's 7 fields and the `academic_id` is filled in by the dispatcher from the actor's current academic year.

**Evidence:**

`docs/specs/operations/commands.md:225-235` `pub struct RecordUserLogCommand { pub tenant: TenantContext, pub user_id: UserId, pub role_id: RoleId, pub ip_address: IpAddress, pub user_agent: UserAgent, pub outcome: LoginOutcome, pub failure_reason: Option<LoginFailureReason>, }` (no `academic_id`). `crates/cross-cutting/operations/src/commands.rs:307-316` includes `pub academic_id: Option<AcademicYearRef>,` (line 311).

---

### FINDING 19 (id: `CROSSCUT-OPS-019`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/commands.rs:345-352` (`ConfigureMaintenanceCommand`)

**Description:**

All four fields of `ConfigureMaintenanceCommand` are typed as `Option<...>` in the code; the spec at `docs/specs/operations/commands.md:244-252` declares them as required (no `Option`). The current type makes the command unable to express the spec's "creates or updates the school's `MaintenanceSetting`" effect with required fields, and the implementation needs an additional `reconfigure`-style command to express the partial-update flow.

**Expected:**

The spec's signature (title, sub_title, image, applicable_for all required) - or, if the partial-update use case is intended, a separate `ReconfigureMaintenanceCommand` (which already exists as `MaintenanceSetting::reconfigure` on the aggregate at `aggregate.rs:607-632`).

**Evidence:**

`docs/specs/operations/commands.md:244-252` `pub struct ConfigureMaintenanceCommand { pub tenant: TenantContext, pub title: MaintenanceTitle, pub sub_title: MaintenanceSubTitle, pub image: Option<MaintenanceImage>, pub applicable_for: MaintenanceApplicableFor, }` (note: `image` is `Option`, but `title`, `sub_title`, `applicable_for` are required). `crates/cross-cutting/operations/src/commands.rs:345-352` `pub struct ConfigureMaintenanceCommand { pub tenant: TenantContext, pub title: Option<MaintenanceTitle>, pub sub_title: Option<MaintenanceSubTitle>, pub image: Option<MaintenanceImage>, pub applicable_for: Option<MaintenanceApplicableFor>, }` (all four are `Option`).

---

### FINDING 20 (id: `CROSSCUT-OPS-020`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/aggregate.rs:607-632` (`MaintenanceSetting::reconfigure`)

**Description:**

`MaintenanceSetting::reconfigure` takes `image: Option<Option<MaintenanceImage>>` (a double-`Option` to distinguish "leave unchanged" from "clear the image"); the spec at `docs/specs/operations/commands.md:243-256` defines `ConfigureMaintenanceCommand::image: Option<MaintenanceImage>` (single `Option`). The double-`Option` is not driven by any command shape in the code (the command uses single `Option`) - the aggregate API is not reachable from the command as written.

**Expected:**

A matching single-`Option` API, or a separate clear-image command.

**Evidence:**

`crates/cross-cutting/operations/src/aggregate.rs:607-632` `pub fn reconfigure(&mut self, title: Option<MaintenanceTitle>, sub_title: Option<MaintenanceSubTitle>, image: Option<Option<MaintenanceImage>>, applicable_for: Option<MaintenanceApplicableFor>, ...)`. `crates/cross-cutting/operations/src/commands.rs:345-352` `pub struct ConfigureMaintenanceCommand { pub image: Option<MaintenanceImage>, ... }` (single `Option`).

---

### FINDING 21 (id: `CROSSCUT-OPS-021`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/aggregate.rs:172` (`Job::new`)

**Description:**

`Job::new` validates that the payload is non-empty by inspecting the `JobPayload` wrapper's inner string (line 176-180), but the spec at `docs/specs/operations/aggregates.md:65` says "A `Job::payload` is a serialized command envelope" - the spec means a JSON-encoded command, not just a non-empty string. The current `JobPayload::new` (at `value_objects.rs:996-1012`) accepts any non-empty 1..65000-char string, so a plain `"hello"` would pass validation even though it's not a valid command envelope.

**Expected:**

`JobPayload::new` calls (or is paired with) a JSON-deserialization step that verifies the payload matches a `CommandEnvelope` schema.

**Evidence:**

`docs/specs/operations/aggregates.md:62-66` "Invariants: 1. A Job::queue is a non-empty string. 2. A Job::payload is a serialized command envelope." `crates/cross-cutting/operations/src/value_objects.rs:996-1012` `pub fn new(s: impl Into<String>) -> Result<Self> { if s.is_empty() || s.len() > 65000 { return Err(...); } Ok(Self(s)) }` - no JSON shape check.

---

### FINDING 22 (id: `CROSSCUT-OPS-022`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/aggregate.rs:439-478` (`VersionHistory`)

**Description:**

The `VersionHistory` struct has a field named `version_` (trailing underscore) to avoid colliding with the `version` field. The trailing-underscore is the engine's documented anti-pattern ("No `#[allow(dead_code)]` or `_var` prefixes to silence the compiler" in `AGENTS.md` and `docs/code-standards.md`). The field is the audit version counter (an Engine implementation detail) and is not declared in the spec; the public field is misnamed.

**Expected:**

The aggregate field is renamed (e.g. to `audit_version` or the audit counter is stored as a private field).

**Evidence:**

`crates/cross-cutting/operations/src/aggregate.rs:445-446` `pub version_: Version,`. `AGENTS.md` section "Type Safety" prohibits `_var` prefixes.

---

### FINDING 23 (id: `CROSSCUT-OPS-023`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/aggregate.rs:439-478` (`VersionHistory`)

**Description:**

`VersionHistory::new` does not initialize the `updated_at` / `updated_by` / `etag` fields; the struct does not have those fields at all, so the append-only invariant is enforced at the type level (no `update` method), but the audit-trail fields are missing. The spec at `docs/specs/operations/aggregates.md:166-174` lists 5 invariants; the implementation does not model `updated_at` even though every other aggregate has it.

**Expected:**

A consistent audit-field set across all 8 root aggregates, with `VersionHistory` either explicitly append-only (no `updated_at`) and the field set documented, or carrying the same audit fields as the others.

**Evidence:**

`docs/specs/operations/aggregates.md:165-174` "Invariants: 1. A VersionHistory::version is non-empty. ... 5. VersionHistory rows are append-only." (no mention of `updated_at`). `crates/cross-cutting/operations/src/aggregate.rs:445-451` has `id`, `version`, `release_date`, `url`, `notes`, `version_`, `etag`, `created_at`, `created_by`, `last_event_id`, `correlation_id` (no `updated_at` / `updated_by`).

---

### FINDING 24 (id: `CROSSCUT-OPS-024`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/aggregate.rs:75-99` (`Backup::new`) and `services.rs:362-372` (`MaintenanceService::applies_to_role`)

**Description:**

`Backup::new` validates `if !cmd.active_status && cmd.restore_in_progress` (line 76-80) but `RestoreBackup` is implemented as a flag flip on the aggregate (`mark_restoring` at `aggregate.rs:102-107`) and not as a port-driven storage operation. The spec at `docs/specs/operations/aggregates.md:23-25` and `commands.md:45-60` says `RestoreBackup` "triggers the restore through the storage port" and "After restore, the platform domain invalidates its in-memory caches." Neither the storage port invocation nor the platform cache invalidation is implemented.

**Expected:**

A `RestoreBackupService::execute(backup, storage_port, platform_cache)` that calls the storage port, emits `BackupRestored`, and notifies the platform subscriber.

**Evidence:**

`docs/specs/operations/aggregates.md:25` "A Backup cannot be hard-deleted while a restore is in progress." `docs/specs/operations/commands.md:45-60` `RestoreBackupCommand` effects: "Triggers the restore through the storage port, emits BackupRestored. After restore, the platform domain invalidates its in-memory caches." `crates/cross-cutting/operations/src/aggregate.rs:102-115` `mark_restoring` and `clear_restoring` only flip the flag - no storage port call, no platform subscriber.

---

### FINDING 25 (id: `CROSSCUT-OPS-025`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/aggregate.rs:141-265` (`Job`)

**Description:**

`Job::fail` (line 241-251) sets the job's status to `Failed` and updates `last_event_id`, but it does not create a `FailedJob` row. The spec at `docs/specs/operations/events.md:135-137` says "`FailedJob` is created from this event by the operations subscriber" (referring to `JobFailed`); the spec at `docs/specs/operations/workflows.md:42-46` says step 6-7: "On failure, the runner increments attempts; if the retry budget is exhausted, the runner issues MarkJobFailedCommand... The operations domain records a FailedJob row and emits FailedJobRecorded." There is no `JobFailedSubscriber` (or equivalent) in the operations crate.

**Expected:**

A subscriber that observes `JobFailed` and creates a `FailedJob` row (emitting `FailedJobRecorded`).

**Evidence:**

`docs/specs/operations/events.md:135-137` **Subscribers:** `FailedJob is created from this event by the operations subscriber`. `docs/specs/operations/workflows.md:42-46` step 6-7. `crates/cross-cutting/operations/src/aggregate.rs:241-251` `Job::fail` only mutates the job. `crates/cross-cutting/operations/src/` has no `subscriber.rs`, no `on_job_failed` handler, no `RecordFailedJob` aggregator.

---

### FINDING 26 (id: `CROSSCUT-OPS-026`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/services.rs` (entire) and `crates/cross-cutting/operations/src/lib.rs:26-34` (module surface)

**Description:**

The 9 service structs and the 3 policies / 4 specifications are pure helper modules - there is no command-handler / dispatcher module in the operations crate. The spec at `docs/specs/operations/overview.md:111-123` says events drive cross-domain flows; `docs/specs/operations/permissions.md:91-97` says capabilities are checked at the command boundary. Neither a command handler, a capability check, nor an event subscriber is implemented. All 24 commands are wire-form-only data structs.

**Expected:**

A `handlers.rs` (or per-aggregate `*_service.rs`) module that wires each command to its aggregate method, runs the capability check, emits the events, and writes the audit / outbox / idempotency rows.

**Evidence:**

`docs/specs/operations/commands.md` (24 commands listed); `docs/specs/operations/permissions.md:91-97` capability check pattern. `crates/cross-cutting/operations/src/lib.rs:26-34` declares `aggregate, commands, entities, errors, events, query, repository, services, value_objects` - no `handler` or `dispatch` module.

---

### FINDING 27 (id: `CROSSCUT-OPS-027`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/commands.rs:439-444` (`UpdateSidebarEntryCommand`) and `aggregate.rs:659-781` (`Sidebar`)

**Description:**

`UpdateSidebarEntryCommand` carries `position`, `level`, `ignore`, `active_status` - but the `Sidebar` aggregate has no `update` method that takes these four fields together, no `set_position`, no `set_level`. The two existing setters (`set_ignore` at `aggregate.rs:752-762`, `set_active` at `aggregate.rs:766-777`) handle two of the four fields individually. A command handler that wires `UpdateSidebarEntryCommand` to the aggregate has no method to call.

**Expected:**

A `Sidebar::update(&mut self, position, level, ignore, active_status, actor, at, event_id)` method (or a handler that calls the four setters in sequence).

**Evidence:**

`docs/specs/operations/commands.md:436-444` `pub struct UpdateSidebarEntryCommand { pub tenant: TenantContext, pub sidebar_id: SidebarId, pub position: Option<SidebarPosition>, pub level: Option<SidebarLevel>, pub ignore: Option<SidebarIgnoreFlag>, pub active_status: Option<SidebarActiveStatus>, }`. `crates/cross-cutting/operations/src/aggregate.rs:703-777` impl block lists only `new`, `is_system`, `reorder`, `set_ignore`, `set_active`.

---

### FINDING 28 (id: `CROSSCUT-OPS-028`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/aggregate.rs:175-199` (`Job::new`) and `services.rs:113-114` (`JobService::can_retry`)

**Description:**

The spec at `docs/specs/operations/aggregates.md:67-70` defines invariant 4 (`A Job::available_at is a Unix timestamp; the job is not runnable before this time.`) and invariant 5 (`A Job::reserved_at is a Unix timestamp; if set, the job is currently being processed by a worker.`). `Job::new` does not validate that `available_at` is not in the past (which would be a worker-port concern, not a domain invariant), but the more important issue is that `JobService::can_retry` at `services.rs:112-114` is `pub fn can_retry(job: &Job, max_attempts: u8) -> bool { job.attempts.0 < max_attempts }` and never inspects the job's `available_at` or `reserved_at` - a job that has been reserved but is "available" again would be flagged as retryable.

**Expected:**

`can_retry` also checks `job.status == Pending` and `job.available_at <= now`.

**Evidence:**

`docs/specs/operations/aggregates.md:64-70` invariants 3-5. `crates/cross-cutting/operations/src/services.rs:112-114` `pub fn can_retry(job: &Job, max_attempts: u8) -> bool { job.attempts.0 < max_attempts }`.

---

### FINDING 3 (id: `CROSSCUT-OPS-003`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/aggregate.rs:659-781` (`Sidebar`)

**Description:**

The `Sidebar` aggregate has no `update` method to back `UpdateSidebarEntryCommand`, which can patch `position`, `level`, `ignore`, and `active_status`. The aggregate has `set_ignore` and `set_active` setters but no `set_position`, `set_level`, or general `update`; the `Sidebar` invariant ("`rbac_sidebars.is_system_defined` flags system-defined rows; the engine refuses to delete them") has no enforcement point.

**Expected:**

A `Sidebar::update(position, level, ignore, active_status, actor, at, event_id)` method (or per-field setters) that emits `SidebarEntryUpdated`.

**Evidence:**

`docs/specs/operations/commands.rs:436-444` `UpdateSidebarEntryCommand` carries `position`, `level`, `ignore`, `active_status`. `crates/cross-cutting/operations/src/aggregate.rs:703-777` impl block has `new`, `is_system`, `reorder`, `set_ignore`, `set_active` only. `docs/specs/operations/aggregates.md:270-273` is_system_defined invariant.

---

### FINDING 4 (id: `CROSSCUT-OPS-004`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/aggregate.rs:39-132` (`Backup`)

**Description:**

The `Backup` aggregate uses raw `bool` for `active_status` and `restore_in_progress`, plus a raw `bool` for `MaintenanceSetting::maintenance_mode`. The spec defines typed wrappers `BackupActiveStatus` (`docs/specs/operations/value-objects.md:35`), `MaintenanceMode` (`:94`), and `SidebarIsSaas` (`:106`) that are absent from `value_objects.rs`.

**Expected:**

Per the engine rule "Compile-time safety over strings" (and its analogue for booleans), the aggregate fields should use the typed wrappers `BackupActiveStatus`, `MaintenanceMode`, and the per-school sidebar should carry a typed `SidebarIsSaas`.

**Evidence:**

`docs/specs/operations/value-objects.md:34-35` `| BackupActiveStatus | bool |`; `:94` `| MaintenanceMode | bool |`; `:106` `| SidebarIsSaas | bool |`. `crates/cross-cutting/operations/src/value_objects.rs` has no struct named `BackupActiveStatus`, `MaintenanceMode`, or `SidebarIsSaas`. `crates/cross-cutting/operations/src/aggregate.rs:46-47` `pub active_status: bool, pub restore_in_progress: bool,`; `:558` `pub maintenance_mode: bool,`.

---

### FINDING 5 (id: `CROSSCUT-OPS-005`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/value_objects.rs` (entire file) and `entities.rs:31-200`

**Description:**

Six typed value objects mandated by `docs/specs/operations/value-objects.md` are absent: `JobReservedAt` (`:44`), `JobCreatedAt` (`:46`), `FailedAt` (`:58`), `MigrationName` (`:113`), `MigrationBatch` (`:114`), and the entire OAuth / PasswordReset section (`:118-135`). The first three are engine-owned; the Migration / OAuth / PasswordReset types are port-driven per `docs/specs/operations/commands.md:313-342` and `tables.md:87-95`, so the spec is OK on those being port-driven - but `JobReservedAt`, `JobCreatedAt`, and `FailedAt` are owned by the operations domain's `Job` / `FailedJob` aggregates and have no typed wrapper in code.

**Expected:**

`JobReservedAt`, `JobCreatedAt`, `FailedAt` typed wrappers in `value_objects.rs` (parallel to `JobAvailableAt` which is already a `pub use Timestamp as JobAvailableAt;` alias at `:1037`).

**Evidence:**

`docs/specs/operations/value-objects.md:44-46` lists `JobReservedAt`, `JobAvailableAt`, `JobCreatedAt`; `:58` lists `FailedAt`. `crates/cross-cutting/operations/src/value_objects.rs:1037` `pub use educore_core::value_objects::Timestamp as JobAvailableAt;` - only one of the three Job timestamp types is wrapped; the other two are missing.

---

### FINDING 6 (id: `CROSSCUT-OPS-006`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/entities.rs:31-201, 541-573` (entity structs)

**Description:**

13 entity-level typed identifiers mandated by `docs/specs/operations/entities.md` are absent from `value_objects.rs` and the entity fields use raw `Uuid`: `FailedJobExceptionId`, `SystemVersionFeatureId`, `VersionHistoryNoteId`, `UserLogContextId`, `UserLogSessionId`, `MaintenanceOverrideId`, `SidebarEntryId`, `SidebarRouteId`, `JobQueueId`, `SystemVersionManifestId`, `SidebarRoleBindingId`, `SystemVersionCapabilityId`, `VersionMigrationId`. The spec's `entities.md` declares each of these as `Identity: <Name>(SchoolId, Uuid)` or `<Name>(Uuid)` and the engine's safety rule requires typed identifiers.

**Expected:**

13 typed identifier structs in `value_objects.rs`; entity fields in `entities.rs` use those typed ids instead of raw `Uuid`.

**Evidence:**

`docs/specs/operations/entities.md:54-55` `FailedJobException | Identity: FailedJobExceptionId(Uuid)`; `:64-65` `SystemVersionFeatureId(Uuid)`; `:73-74` `VersionHistoryNoteId(Uuid)`; `:81-82` `UserLogContextId(SchoolId, Uuid)`; `:91-92` `UserLogSessionId(SchoolId, Uuid)`; `:99-100` `MaintenanceOverrideId(SchoolId, Uuid)`; `:118-119` `SidebarEntryId(SchoolId, Uuid)`; `:126-127` `SidebarRouteId(SchoolId, Uuid)`; `:154-155` `JobQueueId(SchoolId, Uuid)`; `:171-172` `SystemVersionManifestId(Uuid)`; `:179-180` `AuditPartitionId(SchoolId, Uuid)`; `:188-189` `SidebarRoleBindingId(SchoolId, Uuid)`; `:197-198` `SystemVersionCapabilityId(Uuid)`; `:206-208` `VersionMigrationId(Uuid)`. `crates/cross-cutting/operations/src/entities.rs:212, 222, 242, 252, 275, 285, 315-316, 347, 359, 611, 622, 715, 725, 747, 759` all use raw `Uuid`. `AuditPartitionId` and `BackupScheduleId`/`BackupRetentionId`/`JobAttemptId` are present, but the other 13 are not.

---

### FINDING 7 (id: `CROSSCUT-OPS-007`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/services.rs:99-101` (`JobService::is_reserved`)

**Description:**

`JobService::is_reserved` ignores the `now: Timestamp` parameter that the spec mandates. The spec signature at `docs/specs/operations/services.md:27` is `pub fn is_reserved(job: &Job, now: Timestamp) -> bool`; the code at `services.rs:99-101` declares only `pub fn is_reserved(job: &Job) -> bool` and implements it as `matches!(job.status, JobStatus::Reserved) && job.reserved_at.is_some()`. The spec's intent (parity with `is_available` on line 106-108) is to compare the reservation timestamp against `now` so stale reservations can be detected.

**Expected:**

`pub fn is_reserved(job: &Job, now: Timestamp) -> bool` with body that checks the reservation has not expired (e.g. `reserved_at + lease > now`).

**Evidence:**

`docs/specs/operations/services.md:27` `pub fn is_reserved(job: &Job, now: Timestamp) -> bool { ... }`. `crates/cross-cutting/operations/src/services.rs:99-101` `pub fn is_reserved(job: &Job) -> bool { matches!(job.status, JobStatus::Reserved) && job.reserved_at.is_some() }`.

---

### FINDING 8 (id: `CROSSCUT-OPS-008`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/services.rs:181-200` (`FailedJobService::extract_exception_type`)

**Description:**

The spec signature at `docs/specs/operations/services.md:44` is `pub fn extract_exception_type(exception: &str) -> Option<&'static str>`; the code at `services.rs:181` declares `pub fn extract_exception_type(exception: &str) -> Option<&str>`. The function returns a slice of the input `exception` (it cannot be `'static`), so the return type is at minimum inconsistent with the spec - a caller relying on `'static` will not compile against the actual signature.

**Expected:**

Code matches spec exactly, or the spec is corrected and a return-type contract is documented.

**Evidence:**

`docs/specs/operations/services.md:44` `pub fn extract_exception_type(exception: &str) -> Option<&'static str> { ... }`. `crates/cross-cutting/operations/src/services.rs:181` `pub fn extract_exception_type(exception: &str) -> Option<&str> {`.

---

### FINDING 9 (id: `CROSSCUT-OPS-009`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/services.rs:291-299` (`UserLogService::partition`)

**Description:**

The spec signature at `docs/specs/operations/services.md:77` is `pub fn partition(log: &[UserLog], partition: AuditPartition) -> Vec<&UserLog>`; the code at `services.rs:291-299` takes `partition_label: &str` and matches on `correlation_id.to_string() == partition_label`. The spec intends a partition-bucketed view via the `AuditPartition` entity (with `label`, `period_start`, `period_end`, `entry_count`); the code's correlation-id-as-label proxy is a different (and more error-prone) partitioning scheme.

**Expected:**

`pub fn partition(log: &[UserLog], partition: &AuditPartition) -> Vec<&UserLog>` that filters by `period_start <= logged_at <= period_end`.

**Evidence:**

`docs/specs/operations/services.md:77` `pub fn partition(log: &[UserLog], partition: AuditPartition) -> Vec<&UserLog> { ... }`. `crates/cross-cutting/operations/src/services.rs:291-299` `pub fn partition<'a>(log: &'a [UserLog], partition_label: &str) -> Vec<&'a UserLog> { log.iter().filter(|l| l.correlation_id.to_string() == partition_label).collect() }`.

---

### FINDING 17 (id: `CROSSCUT-OPS-017`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/commands.rs:99-105` (`ScheduleJobCommand::available_at`)

**Description:**

`ScheduleJobCommand::available_at` is typed as `Timestamp` (the raw `educore_core::value_objects::Timestamp`); the spec at `docs/specs/operations/commands.md:81` declares the field as `JobAvailableAt` (a typed wrapper). The same drift applies to every other command field that the spec wraps in a typed value object (e.g. `JobQueue` in `ScheduleJobCommand` matches; `JobPayload` in `ScheduleJobCommand` matches; `FailedJobException` in `RecordFailedJobCommand` and `MarkJobFailedCommand` matches - these are OK; the drift is on the timestamp).

**Expected:**

`pub available_at: crate::value_objects::JobAvailableAt` (which is already declared as `pub use Timestamp as JobAvailableAt;` at `value_objects.rs:1037`).

**Evidence:**

`docs/specs/operations/commands.md:78-84` `pub struct ScheduleJobCommand { ..., pub available_at: JobAvailableAt, }`. `crates/cross-cutting/operations/src/commands.rs:99-105` `pub available_at: Timestamp,`.

---

### FINDING 18 (id: `CROSSCUT-OPS-018`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/value_objects.rs:457-489` (`SidebarIgnoreFlag`)

**Description:**

The spec at `docs/specs/operations/value-objects.md:105` declares the type as `SidebarIgnore` (no `Flag` suffix); the code at `value_objects.rs:457` names it `SidebarIgnoreFlag`. The same prefix drift appears in the `repositories.md` "Indexes" section which references the column as `rbac_sidebars.ignore` (consistent with both names) - but the type name in the public API is non-conformant.

**Expected:**

`pub struct SidebarIgnore(pub i32);` matching the spec.

**Evidence:**

`docs/specs/operations/value-objects.md:105` `| SidebarIgnore | i32 (0=Show, 1=Hide, 2=Disabled) |`. `crates/cross-cutting/operations/src/value_objects.rs:457` `pub struct SidebarIgnoreFlag(pub i32);` and re-exported in `lib.rs:66` as `pub use crate::value_objects::..., SidebarIgnoreFlag, ...`.

---

### FINDING 31 (id: `CROSSCUT-OPS-031`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/aggregate.rs:39-99` (`Backup`)

**Description:**

`Backup::new` returns the actor's id as both `created_by` and `updated_by` (line 94-95), and there is no `delete` method. The spec at `docs/specs/operations/aggregates.md:5-25` and `commands.md:32-43` defines a `DeleteBackupCommand` that emits `BackupDeleted`; the aggregate's `updated_by` field is never set to the actor who deletes the row because there is no `delete` method. A consumer attempting to call `repository.delete(backup_id)` cannot populate the audit trail from the aggregate.

**Expected:**

A `Backup::delete(actor, at, event_id)` method that sets `active_status = false`, `updated_by = actor`, and returns the row state.

**Evidence:**

`docs/specs/operations/commands.md:33-43` `DeleteBackupCommand` effects: "Deletes the Backup row and the underlying file, emits BackupDeleted." `crates/cross-cutting/operations/src/aggregate.rs:73-132` impl block: `new`, `mark_restoring`, `clear_restoring`, `mark_active`, `mark_inactive` only.

---

### FINDING 32 (id: `CROSSCUT-OPS-032`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/aggregate.rs:312-332` (`FailedJob`)

**Description:**

`FailedJob` has only `new` and no `delete` (which the spec at `docs/specs/operations/aggregates.md:115` declares as `DeleteFailedJob`). A failed-job retention sweep that hard-deletes rows cannot be modelled - the repository's `delete` is the only way to remove rows, bypassing any audit-trail update.

**Expected:**

A `FailedJob::delete(actor, at, event_id)` method (or a port-driven purge that the service layer calls).

**Evidence:**

`docs/specs/operations/aggregates.md:113-116` lists `RecordFailedJob`, `RetryFailedJob`, `DeleteFailedJob` commands. `crates/cross-cutting/operations/src/aggregate.rs:310-332` impl block: only `new` (no `delete` or `retry` method either).

---

### FINDING 33 (id: `CROSSCUT-OPS-033`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/aggregate.rs:550-649` (`MaintenanceSetting`)

**Description:**

`MaintenanceSetting` has no `delete` (or `soft_delete`) method. The spec at `docs/specs/operations/aggregates.md:241-251` implies the singleton can be replaced via `ConfigureMaintenanceCommand` but doesn't model a hard delete; however, the per-school `MaintenanceSettingId` is a typed `Id<MaintenanceSetting>` and the repository at `repository.rs:244-251` has only `get`, `insert`, `update` - no `delete`. The missing aggregate `delete` is paired with a missing repository `delete`, leaving the per-school singleton effectively un-removable.

**Expected:**

A repository `delete(&self, school: SchoolId)` and an aggregate `MaintenanceSetting::soft_delete(actor, at, event_id)` (or a documented "singleton cannot be deleted" invariant).

**Evidence:**

`docs/specs/operations/aggregates.md:241-251` "ConfigureMaintenance, EnableMaintenance, DisableMaintenance" (no `DeleteMaintenance` listed). `crates/cross-cutting/operations/src/aggregate.rs:583-649` `MaintenanceSetting` impl block: `configure`, `reconfigure`, `enable`, `disable` only. `crates/cross-cutting/operations/src/repository.rs:243-251` `MaintenanceSettingRepository` trait: `get`, `insert`, `update` (no `delete`).

---

### FINDING 34 (id: `CROSSCUT-OPS-034`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/aggregate.rs:39-132` (`Backup`)

**Description:**

`Backup` has no `restore_completed` method (only `mark_restoring` and `clear_restoring` at `aggregate.rs:102-115`). The spec at `docs/specs/operations/aggregates.md:23-25` and `commands.md:45-60` says `RestoreBackup` "triggers the restore through the storage port, emits BackupRestored. After restore, the platform domain invalidates its in-memory caches." The current `clear_restoring` only flips the boolean; it does not emit `BackupRestored` (the spec event for restore is `BackupRestored`, not `BackupMarkedInactive`).

**Expected:**

A `Backup::restore_complete(actor, at, event_id)` method that sets `restore_in_progress = false`, populates audit fields, and (per the dispatcher) emits `BackupRestored`.

**Evidence:**

`docs/specs/operations/commands.md:55-60` "Effects: Triggers the restore through the storage port, emits BackupRestored. After restore, the platform domain invalidates its in-memory caches." `crates/cross-cutting/operations/src/aggregate.rs:102-115` `mark_restoring` and `clear_restoring` - no event emission, no `BackupRestored` mapping.

---

### FINDING 35 (id: `CROSSCUT-OPS-035`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/aggregate.rs:399-427` (`SystemVersion::update`)

**Description:**

`SystemVersion::update` only accepts `Option<VersionTitle>` and `Option<VersionFeatures>` and silently ignores the case where both are `None` (the body just falls through). The spec at `docs/specs/operations/commands.md:188-200` defines `UpdateSystemVersionCommand` with both `title` and `features` as `Option`, but the expected effect is "Emits `SystemVersionUpdated`" - a no-op update would still emit the event.

**Expected:**

Either reject the call when both fields are `None` (returning an `Err`), or always emit `SystemVersionUpdated` even when both are `None`.

**Evidence:**

`docs/specs/operations/commands.md:198-200` "Effects: Emits SystemVersionUpdated." `crates/cross-cutting/operations/src/aggregate.rs:399-427` `pub fn update(&mut self, title: Option<VersionTitle>, features: Option<VersionFeatures>, actor: UserId, at: Timestamp, event_id: EventId) -> AggregateResult<()>` - body does nothing when both are `None`, and the return value is unused by the dispatcher (which would always emit the event).

---

### FINDING 36 (id: `CROSSCUT-OPS-036`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/aggregate.rs:75-99` (`Backup::new`) and `commands.rs:31-37` (`CreateBackupCommand`)

**Description:**

`CreateBackupCommand` does not carry a `restore_in_progress` field (per spec at `docs/specs/operations/commands.md:17-24` - the spec command is `{ tenant, file_name, source_link, file_type, lang_type }` with no `restore_in_progress`); but `Backup::new` requires a `restore_in_progress: bool` field on `NewBackup` (line 67), defaulting it to `false` in the wire flow. The spec command struct cannot construct a `NewBackup` because the `restore_in_progress` field is not provided by the wire form.

**Expected:**

The `Backup::new` API should derive `restore_in_progress` from aggregate state (it is always `false` at creation) and the `NewBackup` struct should not require it.

**Evidence:**

`docs/specs/operations/commands.md:17-24` `CreateBackupCommand` (5 fields, no `restore_in_progress`). `crates/cross-cutting/operations/src/commands.rs:31-37` (matches the spec). `crates/cross-cutting/operations/src/aggregate.rs:60-71` `pub struct NewBackup { ..., pub restore_in_progress: bool, ... }` - the 6th field, not in the command, must be supplied by the dispatcher.

---

### FINDING 37 (id: `CROSSCUT-OPS-037`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/services.rs:140-148` (`JobService::purge_completed`)

**Description:**

`JobService::purge_completed` partitions a `Vec<Job>` into `Completed` and `pending`, but the "pending" name is misleading because it also includes `Reserved` and `Failed`. A reader of the function name "purge_completed" would expect a `Failed` purge option, and the returned vec semantics are conflated.

**Expected:**

Rename the `pending` vec to `kept` and document the `Reserved + Failed` inclusion, or add a `purge_failed` companion.

**Evidence:**

`docs/specs/operations/services.md:32` `pub fn purge_completed(jobs: &mut Vec<Job>) -> Vec<Job> { ... }`. `crates/cross-cutting/operations/src/services.rs:140-148` `let (done, pending): (Vec<Job>, Vec<Job>) = jobs.drain(..).partition(|j| matches!(j.status, JobStatus::Completed)); *jobs = pending; done`.

---

### FINDING 38 (id: `CROSSCUT-OPS-038`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/repository.rs:244-251` (`MaintenanceSettingRepository`)

**Description:**

The `MaintenanceSettingRepository` trait has no `delete` method, so the per-school singleton cannot be removed. The spec at `docs/specs/operations/repositories.md:99-108` lists `get`, `insert`, `update` only - but the spec at `docs/specs/operations/aggregates.md:233` says "A `MaintenanceSetting` exists at most once per `SchoolId`" and the spec at `commands.md:241-281` does not define a `DeleteMaintenance` command, so the missing repository `delete` is consistent with the spec. The aggregate also has no `delete` method, so the two are in sync. This is a documentation gap rather than a bug - but the audit's "Missing repositories" check expects `delete` to exist; the omission is a deliberate spec choice that is not documented as such.

**Expected:**

Either add the repository `delete` (and the matching aggregate method), or document the "singleton is permanently addressable" invariant in `aggregates.md`.

**Evidence:**

`docs/specs/operations/repositories.md:99-108` lists 3 methods (`get`, `insert`, `update`) - no `delete`. `crates/cross-cutting/operations/src/repository.rs:243-251` matches the spec. The spec at `docs/specs/operations/aggregates.md:231-251` is silent on whether the singleton can be deleted.

---

### FINDING 39 (id: `CROSSCUT-OPS-039`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/repository.rs:108-128` (`FailedJobRepository`) and `docs/specs/operations/repositories.md:41-55`

**Description:**

The spec at `docs/specs/operations/repositories.md:46-54` requires `get_by_uuid(&self, uuid: &FailedJobUuid) -> Result<Option<FailedJob>>`. The code at `repository.rs:108-128` defines the trait, but the aggregate `FailedJob` does not have a `uuid` field uniqueness validator (e.g. checking that the `uuid` is not nil). The repository can return a duplicate `FailedJob` for the same business uuid; the trait's `get_by_uuid` has no DB-level constraint backing it (since no DDL is emitted from the operations crate).

**Expected:**

A unique constraint on `failed_jobs.uuid` (per spec at `tables.md:46` and `repositories.md:183` `CREATE UNIQUE INDEX ux_failed_jobs_uuid`).

**Evidence:**

`docs/specs/operations/tables.md:46-48` "`failed_jobs.uuid` is a unique business identifier separate from the auto-increment id." `docs/specs/operations/repositories.md:183` `CREATE UNIQUE INDEX ux_failed_jobs_uuid ON failed_jobs (uuid);`. `crates/cross-cutting/operations/src/aggregate.rs:310-332` `FailedJob` has no constructor-time validation of the `uuid` (e.g. `uuid != Uuid::nil()`).

---

### FINDING 40 (id: `CROSSCUT-OPS-040`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/aggregate.rs:75-99` (`Backup`) and `docs/specs/operations/aggregates.md:17-25`

**Description:**

The spec says `Backup::file_name` is unique within `(school_id, file_name)`; the code's `Backup::new` does not check uniqueness (the repository is expected to enforce it, but the spec says the aggregate is loaded, validated, and persisted in a single transaction). Without a uniqueness check in the aggregate, two `CreateBackupCommand` calls in the same transaction with the same `(school_id, file_name)` would both succeed at the aggregate level and fail at the database.

**Expected:**

A uniqueness check in `Backup::new` (or a clear contract that the repository enforces it).

**Evidence:**

`docs/specs/operations/aggregates.md:17-25` invariant 2: "A Backup::file_name is non-empty and unique within (school_id, file_name)." `docs/specs/operations/aggregates.md:43-47` "Consistency Boundary: A Backup is loaded by id, mutated in memory, validated, and persisted with its events in a single transaction." `crates/cross-cutting/operations/src/aggregate.rs:75-99` `Backup::new` does not check uniqueness.

---

### FINDING 41 (id: `CROSSCUT-OPS-041`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/events.rs:75-86` (`BackupCreated` `aggregate_id`)

**Description:**

`BackupCreated::aggregate_id` returns `self.backup_id.as_uuid()` - the local UUID without the school id. A consumer reconstructing the aggregate's `BackupId` from the event cannot recover the school id. The `BackupId` is tenant-scoped (`school_id: SchoolId, value: Uuid`), so `as_uuid()` strips the tenant.

**Expected:**

The event includes `school_id` (it does - line 32 `pub school_id: SchoolId`) and consumers should reconstruct the typed id from `(school_id, value)`; the spec at `events.md:39-46` lists `BackupCreated { pub backup_id: BackupId, pub file_name: ..., pub file_type: ..., pub created_at: ... }` - the typed `BackupId` carries the school, so the spec is consistent, but the code's `as_uuid()` return is lossy.

**Evidence:**

`docs/specs/operations/events.md:39-46` `pub struct BackupCreated { pub backup_id: BackupId, pub file_name: BackupFileName, pub file_type: BackupFileType, pub created_at: Timestamp, }`. `crates/cross-cutting/operations/src/events.rs:77-79` `fn aggregate_id(&self) -> Uuid { self.backup_id.as_uuid() }`. `crates/cross-cutting/operations/src/value_objects.rs:40-52` `BackupId::as_uuid` returns only the `value: Uuid`, dropping the school id.

---

### FINDING 42 (id: `CROSSCUT-OPS-042`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/aggregate.rs:39-132` (`Backup`) and `aggregate.rs:102-115` (`mark_restoring` / `clear_restoring`)

**Description:**

`Backup` has `restore_in_progress: bool` (a typed raw bool, see Finding 4) and a `mark_restoring` / `clear_restoring` pair that flips it. The spec at `docs/specs/operations/aggregates.md:43-47` says "Concurrent `RestoreBackup` commands on the same backup are serialized." There is no concurrency control in the aggregate; two `RestoreBackupCommand` calls in different transactions can both pass the aggregate check and both call `mark_restoring` - the second one will fail only at the database level (if a unique index exists). The spec's "serialized" invariant has no enforcement point.

**Expected:**

A row-level lock or a serialized concurrency primitive on the `Backup` row (the spec at `docs/handoff/PHASE-14-HANDOFF.md:218-222` says "the dispatcher acquires the row-level lock" - but no dispatcher is implemented).

**Evidence:**

`docs/specs/operations/aggregates.md:43-47` Consistency Boundary. `docs/handoff/PHASE-14-HANDOFF.md:218-222` "the dispatcher acquires the row-level lock on the relevant row." `crates/cross-cutting/operations/src/aggregate.rs:102-115` `mark_restoring` and `clear_restoring` - no concurrency guard.

---

### FINDING 43 (id: `CROSSCUT-OPS-043`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/aggregate.rs:75-99` (`Backup::new`) and `events.rs:28-86` (`BackupCreated`)

**Description:**

`BackupCreated` event is emitted by the dispatcher (which doesn't exist - see Finding 26), but the event's `created_at` and `occurred_at` are independent timestamps (line 35 `pub created_at: Timestamp` and line 38 `pub occurred_at: Timestamp`). The spec at `docs/specs/operations/events.md:39-46` lists only `created_at`; the code adds `occurred_at` (per the `DomainEvent` trait). A consumer that filters by `occurred_at` will see a different value than a consumer that filters by `created_at` - the two are never reconciled.

**Expected:**

A documented contract that `created_at == occurred_at` for `BackupCreated`, or a single `occurred_at` field.

**Evidence:**

`docs/specs/operations/events.md:39-46` lists only `created_at`. `crates/cross-cutting/operations/src/events.rs:30-40` has both `pub created_at: Timestamp,` (line 35) and `pub occurred_at: Timestamp,` (line 38).

---

### FINDING 44 (id: `CROSSCUT-OPS-044`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/aggregate.rs:75-99` (`Backup::new`) and `value_objects.rs:495-514` (`BackupFileName`)

**Description:**

`BackupFileName::new` validates 1..255 chars but does not validate that the file name is a valid filename (e.g. no slashes, no nulls, no `..` path traversal). The spec at `docs/specs/operations/aggregates.md:18-19` says "A `Backup::file_name` is non-empty and unique within (school_id, file_name)" - silent on character set, but the spec at `value-objects.md:31` says "1..255 chars, unique within (school_id, file_name)". A `file_name` of `"../../etc/passwd"` would pass the current validator and could be passed to the file-storage port as a path.

**Expected:**

A filename-shape validator (no path separators, no `..`, no null bytes).

**Evidence:**

`crates/cross-cutting/operations/src/value_objects.rs:499-514` `pub fn new(s: &str) -> Result<Self> { if s.is_empty() || s.len() > 255 { return Err(...); } Ok(Self(s.to_owned())) }` - no character-class check. `docs/specs/operations/aggregates.md:18-19` and `value-objects.md:31` silent on character set.

---

### FINDING 45 (id: `CROSSCUT-OPS-045`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/aggregate.rs:490-540` (`UserLog`) and `events.rs:973-1036` (`UserLogged`)

**Description:**

`UserLog` is declared append-only in the spec (invariant 8) and the code has no `update` or `delete` method on the aggregate - good. But the spec at `docs/specs/operations/workflows.md:62-73` says "A nightly job (port) partitions the log by month" and "purges UserLog rows older than the school's retention policy (default 365 days). The purge is logged as a `DeleteUserLog` event for compliance." The repository's `UserLogRepository::purge_older_than` (at `repository.rs:229`) hard-deletes rows without going through the aggregate (which has no `delete` method), and no `UserLogDeleted` event is emitted (Finding 14).

**Expected:**

A `UserLogService::purge_with_audit(rows, actor, at)` that hard-deletes each row and emits a `UserLogDeleted` event (which is missing - see Finding 14).

**Evidence:**

`docs/specs/operations/workflows.md:62-73` "User Log Retention Workflow" steps 4-6. `crates/cross-cutting/operations/src/repository.rs:228-229` `async fn purge_older_than(&self, school: SchoolId, cutoff: Timestamp) -> StorageResult<u64>;`. `crates/cross-cutting/operations/src/aggregate.rs:509-540` `UserLog` impl has no `delete` or `purge` method.

---

### FINDING 46 (id: `CROSSCUT-OPS-046`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Low
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/commands.rs:288-297` (`RecordVersionHistoryCommand::into_input`)

**Description:**

`RecordVersionHistoryCommand::into_input` (line 288-297) consumes `self` and returns a `VersionHistoryInput`. The `VersionHistory::new` constructor (at `aggregate.rs:457-479`) takes `VersionHistoryInput` as its second argument. The connection from command to aggregate is wired for this one command, but the `VersionHistory` aggregate has no `id` field on `VersionHistoryInput` - the dispatcher must supply a `VersionHistoryId` from outside. Per the spec at `docs/specs/operations/commands.md:204-219` the command does not carry the id either. The convention should be documented or the command should carry an optional `Option<VersionHistoryId>` for the upsert case.

**Expected:**

Either a documented "id is generated by the engine" contract in `commands.md` (the spec at `:212-219` is silent on the id), or an optional id field.

**Evidence:**

`docs/specs/operations/commands.md:204-219` `RecordVersionHistoryCommand` has no id field. `crates/cross-cutting/operations/src/commands.rs:288-297` `into_input` returns a `VersionHistoryInput` (no id). `crates/cross-cutting/operations/src/aggregate.rs:457-479` `VersionHistory::new` takes `id: VersionHistoryId` as a separate parameter (line 459).

---

### FINDING 47 (id: `CROSSCUT-OPS-047`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Low
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/aggregate.rs:312-332` (`FailedJob::new`)

**Description:**

`FailedJob::new` takes `original_job_id: JobId` and `queue: FailedJobQueue` (which is a separate type from `JobQueue`, per `value_objects.rs:929-948` `FailedJobQueue(pub String)` vs `value_objects.rs:972-990` `JobQueue(pub String)`). Both wrap a `String` and both validate 1..191 chars, but the type split requires the dispatcher to construct two distinct types from the same source data. The spec at `docs/specs/operations/value-objects.md:55` says `FailedJobQueue | 1..191 chars` and at `:41` `JobQueue | 1..191 chars (e.g. default, emails, webhooks)` - the spec uses different type names, so the type split is correct per the spec, but the duplicate validation logic is an anti-pattern.

**Expected:**

A shared `QueueName` newtype that both `JobQueue` and `FailedJobQueue` alias to.

**Evidence:**

`docs/specs/operations/value-objects.md:41` `JobQueue | 1..191 chars` and `:55` `FailedJobQueue | 1..191 chars`. `crates/cross-cutting/operations/src/value_objects.rs:972-990` `JobQueue` and `crates/cross-cutting/operations/src/value_objects.rs:929-948` `FailedJobQueue` have duplicate 1..191-char validators (lines 935-937 vs 978-980).

---

### FINDING 48 (id: `CROSSCUT-OPS-048`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Low
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/value_objects.rs:1037` (`JobAvailableAt` alias)

**Description:**

`pub use educore_core::value_objects::Timestamp as JobAvailableAt;` (line 1037) is the only `Job*Timestamp` typed wrapper; the spec at `docs/specs/operations/value-objects.md:44-46` declares three (`JobReservedAt`, `JobAvailableAt`, `JobCreatedAt`). The other two are missing (see Finding 5). The single `JobAvailableAt` alias is correctly emitted but it is not used in `commands.rs` (see Finding 17).

**Expected:**

Either remove the unused alias (if `Timestamp` is the canonical form) or use it in `ScheduleJobCommand` and elsewhere.

**Evidence:**

`crates/cross-cutting/operations/src/value_objects.rs:1037` `pub use educore_core::value_objects::Timestamp as JobAvailableAt;`. `crates/cross-cutting/operations/src/commands.rs:99-105` `pub available_at: Timestamp,` (uses raw `Timestamp`, not the alias).

---

### FINDING 49 (id: `CROSSCUT-OPS-049`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Low
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/value_objects.rs:1097-1108` (`MaintenanceImage`)

**Description:**

`MaintenanceImage::new(name: impl Into<String>) -> Self` (line 1101-1107) bypasses the `FileReference::new` validator - the docstring on line 1105-1106 says "No validation here: callers may pre-validate via `FileReference::new`." The spec at `docs/specs/operations/value-objects.md:92` says `MaintenanceImage | FileReference?` (a `FileReference` optional); a direct `MaintenanceImage::new("")` succeeds.

**Expected:**

`MaintenanceImage::new` calls `FileReference::new` and propagates the error.

**Evidence:**

`docs/specs/operations/value-objects.md:92` `| MaintenanceImage | FileReference? |`. `crates/cross-cutting/operations/src/value_objects.rs:1097-1108` `pub fn new(name: impl Into<String>) -> Self { Self(FileReference(name.into())) }` - no validation, comment on line 1105-1106 admits the gap.

---

### FINDING 50 (id: `CROSSCUT-OPS-050`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Low
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/aggregate.rs:75-99` (`Backup::new`)

**Description:**

`Backup::new` rejects the combination `active_status=false && restore_in_progress=true` (line 76-80) with `Validation`. The spec at `docs/specs/operations/aggregates.md:23` says "A `Backup::active_status` is a boolean" and `:25` says "A `Backup` cannot be hard-deleted while a restore is in progress." The combined invariant "an inactive backup cannot have a restore in progress" is not in the spec - it is a code-original rule.

**Expected:**

Document the combined invariant in `aggregates.md` or remove the check.

**Evidence:**

`docs/specs/operations/aggregates.md:15-25` (spec lists 6 invariants, none is the combined check). `crates/cross-cutting/operations/src/aggregate.rs:76-80` `if !cmd.active_status && cmd.restore_in_progress { return Err(OperationsDomainError::Validation("inactive backup cannot have restore in progress".to_owned())); }`.

---

### FINDING 51 (id: `CROSSCUT-OPS-051`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Low
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/services.rs:239-246` (`SystemVersionService::is_compatible`)

**Description:**

`SystemVersionService::is_compatible` returns true if `c.0 == s.0 && c.0 != 0` (line 224-228). The `c.0 != 0` rule is not in the spec at `docs/specs/operations/services.md:54-58` which says "Returns true if `client` is compatible with `server` (same major version)." The spec's intent is "same major version" - a major version of 0 is typically the pre-release / development version, and the spec is silent on whether 0.0.0 should be considered compatible.

**Expected:**

Either document the 0.0.0 rule or remove the `c.0 != 0` short-circuit.

**Evidence:**

`docs/specs/operations/services.md:54-58` `pub fn is_compatible(client: &VersionName, server: &VersionName) -> bool { ... }` (no 0.0.0 mention). `crates/cross-cutting/operations/src/services.rs:222-228` `c.0 == s.0 && c.0 != 0`.

---

### FINDING 52 (id: `CROSSCUT-OPS-052`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Low
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/value_objects.rs:754-788` (`IpAddress`)

**Description:**

`IpAddress::is_valid` accepts a leading-zero octet (e.g. `010.0.0.1`) as `is_valid` returns `true` only if the part starts with a non-zero digit OR is exactly "0" - the unit test at `value_objects.rs:1283` `assert!(!IpAddress::is_valid("192.0.02.1"))` checks for the case but the `is_valid_ipv4` function at `value_objects.rs:790-815` correctly rejects `part.starts_with('0') && part.len() > 1` (line 799-801). This is correct - but `IpAddress::new("010.0.0.1")` is rejected by the constructor, so the spec is consistent. The finding is null: the test at line 1283 already proves the rejection. (Documented for completeness; no defect.)

**Expected:**

N/A - this is a verification of correct behavior, not a defect. The unit test at `value_objects.rs:1283` proves the rule.

**Evidence:**

`crates/cross-cutting/operations/src/value_objects.rs:799-801` rejects leading-zero octets. `crates/cross-cutting/operations/src/value_objects.rs:1283` `assert!(!IpAddress::is_valid("192.0.02.1"));` (leading zero).

---

### FINDING 53 (id: `CROSSCUT-OPS-053`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Low
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/aggregate.rs:457-479` (`VersionHistory::new`)

**Description:**

`VersionHistory::new` is a `#[must_use]` constructor (line 457) that does not return a `Result`, even though `BackupFileName` and `HistoryVersion` (used in `VersionHistoryInput::new`) both return `Result` and can fail. The path is: command to `into_input` (returns `VersionHistoryInput`) to `VersionHistory::new` (no Result). The validation that occurred at command construction is the only validation; the aggregate's `new` accepts the validated input and cannot fail. The pattern is correct but the `#[must_use]` is over-broad - the aggregate's `new` doesn't have a side-effecting builder, so `#[must_use]` is appropriate.

**Expected:**

Either the spec should be updated to clarify the validation flow, or the `#[must_use]` annotation is fine as-is.

**Evidence:**

`crates/cross-cutting/operations/src/aggregate.rs:457-479` `#[must_use] pub fn new(id: VersionHistoryId, input: VersionHistoryInput, created_by: UserId, correlation_id: CorrelationId, at: Timestamp) -> Self` (no Result). `crates/cross-cutting/operations/src/entities.rs:893-908` `VersionHistoryInput::new` returns `Self` (not Result), and the underlying `HistoryVersion` / `HistoryReleaseDate` / `HistoryNotes` validators are at `value_objects.rs:660-752`.

---

### FINDING 54 (id: `CROSSCUT-OPS-054`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Low
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/aggregate.rs:514-540` (`UserLog::new`)

**Description:**

`UserLog::new` (line 514-540) does not validate that the academic year (`academic_id: Option<AcademicYearRef>`) is consistent with the user's school. A `UserLog` could carry an `AcademicYearRef` with a different `school_id` than the `UserLog::school_id` (which is `input.school_id`). The spec at `docs/specs/operations/aggregates.md:206-207` says "A `UserLog::academic_id` references a valid `AcademicYearId`" but is silent on tenant consistency.

**Expected:**

A check that `academic_id.school_id == log.school_id` when `academic_id.is_some()`.

**Evidence:**

`docs/specs/operations/aggregates.md:203-208` invariants 4-7. `crates/cross-cutting/operations/src/aggregate.rs:514-540` `UserLog::new` body does not compare `input.academic_id.map(|a| a.school_id)` against `input.school_id`.

---

### FINDING 55 (id: `CROSSCUT-OPS-055`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Low
- **Area:** cross-cutting
- **Location:** `crates/cross-cutting/operations/src/aggregate.rs:733-735` (`Sidebar::is_system`)

**Description:**

`Sidebar::is_system` is the only consumer-side check of the `is_system_defined` invariant, but the spec at `docs/specs/operations/aggregates.md:270-273` says "the engine refuses to delete them" - implying a `delete` method that checks the flag. With no `delete` method (Finding 2), the `is_system` helper is unused by any aggregate method. The audit's "test for system-defined deletion" is not possible to write.

**Expected:**

A `Sidebar::delete(actor, at, event_id)` method that returns `Err(OperationsDomainError::Forbidden("system-defined sidebar cannot be deleted"))` when `is_system_defined.0 == true`.

**Evidence:**

`docs/specs/operations/aggregates.md:270-273` "`is_system_defined` flags system-defined rows; the engine refuses to delete them." `crates/cross-cutting/operations/src/aggregate.rs:733-735` `pub const fn is_system(&self) -> bool { self.is_system_defined.0 }` - no caller in `aggregate.rs:703-777`.

---

### FINDING 56 (id: `CROSSCUT-OPS-056`)

- **Source:** `docs/audit_reports/findings/wave2-operations.md`
- **Severity:** Low
- **Area:** cross-cutting
- **Location:** `docs/handoff/PHASE-14-HANDOFF.md:36-40` and `docs/coverage.toml:2039-2128`

**Description:**

The Phase 14 hand-off says "47 unit tests in `educore-operations`" and the coverage matrix has 10 operations rows flipped from `Pending` to `Tested` (8 aggregate rows + 1 capability row + 1 audit-target row), all marked `Tested`. The actual test count from `#[test]` attributes is 12+11+11+2+3+6+2 = 47 in source files (matches the hand-off), but no row in `docs/coverage.toml` references the `tests` directory because the `tests/` directory does not exist (Finding 30). The coverage matrix therefore claims `Tested` for all 8 root aggregates but each row's `tests` field only points at the `aggregate.rs` unit tests - there is no integration test coverage for any of the 8 aggregates.

**Expected:**

A row per aggregate pointing to the integration test file in `crates/cross-cutting/operations/tests/` (which doesn't exist).

**Evidence:**

`docs/coverage.toml:2038-2128` 10 operations rows, all `status = "Tested"`. `docs/handoff/PHASE-14-HANDOFF.md:53-55` "**47 passed, 0 failed**" (unit tests only). `crates/cross-cutting/operations/tests/` does not exist (Finding 30).

### END FINDINGS
Total Findings: 56

---


## Settings (target id prefix: `CROSSCUT-SET`)

**Path:** `crates/cross-cutting/settings/`  
**Total findings:** 28 (0 critical, 10 high, 12 medium, 6 low)


### FINDING 1 (id: `CROSSCUT-SET-001`)

- **Source:** `docs/audit_reports/findings/wave2-settings.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/settings/src/aggregate.rs:43-107

**Description:**

The `GeneralSettings` aggregate stores ~38 spec'd
  value-object fields as raw Rust primitives (`String`, `bool`, `i32`,
  `Option<...>`) directly on the struct, instead of as typed wrappers
  declared in `value_objects.rs`. The spec
  (`docs/specs/settings/value-objects.md` § "General Settings") lists
  `SiteTitle`, `SchoolName`, `SchoolCode`, `Address`, `PhoneNumber`,
  `EmailAddress`, `FileSize`, `LogoFile`, `FaviconFile`, `SystemVersion`,
  `CopyrightText`, `ApiUrl`, `WebsiteUrl`, `PromotionSetting`,
  `SystemPurchaseCode`, `SystemActivatedDate`, `LastUpdate`, `EnvatoUser`,
  `EnvatoItemId`, `SystemDomain`, `WeekStartId`, `AttendanceLayout`,
  `SoftwareVersion`, `MultipleRoll`, `ResultType`, `DirectFeesAssign`,
  `WithGuardian`, `PreloaderStatus`, `PreloaderImage`, `DueFeesLogin`,
  `TwoFactor`, `ActiveTheme` (✓ present), `QueueConnection` (✓),
  `IsCustomSaas`, `IsComment`, `AutoApprove`, `BlogSearch`,
  `RecentBlog`, `AcademicId`, `UnAcademicId`, `BehaviorRecords`,
  `LmsCheckout`, `SessionYear`, `FeesStatus`, `SubTopicEnable` as
  separate value-object types with `Validate` traits.

**Expected:**

Per `docs/specs/settings/value-objects.md`: "All value
  objects implement `Validate` and refuse construction when validation
  fails: ... Construction is the only entry point: `let pattern =
  DateFormatPattern::new(\"%Y-%m-%d\")?;`"

**Evidence:**

`pub school_name: String, ... pub file_size: u64, ...
  pub two_factor: bool, ... pub fees_status: i32, ... pub
  active_status: bool,` (crates/cross-cutting/settings/src/aggregate.rs:46-104)

---

### FINDING 12 (id: `CROSSCUT-SET-012`)

- **Source:** `docs/audit_reports/findings/wave2-settings.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/settings/src/services.rs:338-349

**Description:**

`ThemeService::replicate` signature diverges from
  the spec (`docs/specs/settings/services.md` § "ThemeService" line
  88). The spec signature is
  `replicate(source: &Theme, new_title: ThemeTitle, school: SchoolId)
  -> Result<Theme, ValidationError>`; the implementation is
  `replicate(source: &Theme, new_title: ThemeTitle, new_id: ThemeId,
  at: DateTime<Utc>, actor: UserId) -> Theme` (no `Result`, takes
  `new_id` instead of `school`, takes extra `at`/`actor` params).
  The fallback `.unwrap_or_else(|_| source.clone())` swallows any
  validation error from `Theme::replicate` and returns the source
  theme instead — a silent data-corruption hazard.

**Expected:**

Match the spec signature and return
  `Result<Theme, SettingsDomainError>`.

**Evidence:**

`source
            .replicate(new_id, new_title, ts, actor)
            .unwrap_or_else(|_| source.clone())` (crates/cross-cutting/settings/src/services.rs:347-348)

---

### FINDING 15 (id: `CROSSCUT-SET-015`)

- **Source:** `docs/audit_reports/findings/wave2-settings.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/settings/src/ (entire crate)

**Description:**

`crates/cross-cutting/settings/tests/` directory
  does not exist. Per AGENTS.md § "Testing (TDD)": "At least one
  integration test per PR". The integration test for the settings
  domain is in `crates/tools/storage-parity/tests/settings_integration.rs`
  (a different crate), but the settings crate itself has no
  `tests/` directory with crate-local integration tests.

**Expected:**

Per AGENTS.md § "Validation Checklist", at least one
  integration test added for new behavior — should be co-located at
  `crates/cross-cutting/settings/tests/`.

**Evidence:**

`ls -la crates/cross-cutting/settings/tests/` returns "No such file or directory"

---

### FINDING 2 (id: `CROSSCUT-SET-002`)

- **Source:** `docs/audit_reports/findings/wave2-settings.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/settings/src/aggregate.rs:43-107

**Description:**

The 36 module-toggle feature flags enumerated in
  `docs/specs/settings/value-objects.md` § "Module Toggles"
  (`LessonEnabled`, `ChatEnabled`, `FeesCollectionEnabled`,
  `IncomeHeadId`, `BiometricsEnabled`, `ResultReportsEnabled`,
  `TemplateSettingsEnabled`, `MenuManageEnabled`,
  `RolePermissionEnabled`, `RazorPayEnabled`, `SaasEnabled`,
  `StudentAbsentNotificationEnabled`, `ParentRegistrationEnabled`,
  `ZoomEnabled`, `BbbEnabled`, `VideoWatchEnabled`, `JitsiEnabled`,
  `OnlineExamEnabled`, `SaasRolePermissionEnabled`, `BulkPrintEnabled`,
  `HimalayaSmsEnabled`, `XenditPaymentEnabled`, `WalletEnabled`,
  `LmsEnabled`, `ExamPlanEnabled`, `UniversityEnabled`, `GmeetEnabled`,
  `KhaltiPaymentEnabled`, `RaudhahpayEnabled`, `AppSliderEnabled`,
  `DownloadCenterEnabled`, `AiContentEnabled`, `WhatsappSupportEnabled`,
  `InAppLiveClassEnabled`, `FeesStatus`, `LmsCheckout`) are stored as
  a single `ModuleTogglePatch` BTreeMap rather than as named typed
  wrappers, eliminating compile-time field safety for these flags.

**Expected:**

Per `docs/specs/settings/value-objects.md` § "Module
  Toggles": each toggle should be a typed `bool` value object on the
  `GeneralSettings` aggregate, validated at construction.

**Evidence:**

`pub module_toggles: ModuleTogglePatch,` (crates/cross-cutting/settings/src/aggregate.rs:90)

---

### FINDING 4 (id: `CROSSCUT-SET-004`)

- **Source:** `docs/audit_reports/findings/wave2-settings.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/settings/src/aggregate.rs (entire file)

**Description:**

Zero `#[derive(DomainQuery)]` macros are applied to
  any struct in `aggregate.rs` or `entities.rs` — the engine's
  macro-driven query layer is not wired into the settings crate. The
  14 owned settings tables documented in `docs/specs/settings/tables.md`
  (lines 9-22) have no macro-emitted typed query structs in the crate.
  `query.rs` (309 lines) contains only empty placeholder structs (e.g.
  `pub struct GeneralSettingsQuery { /* Fields filled in by Workstream
  A. */ }`).

**Expected:**

Per `docs/specs/settings/aggregates.md` and
  `docs/query_layer.md`, the settings aggregates should expose typed
  query builders via `#[derive(DomainQuery)]` (or equivalent).

**Evidence:**

`pub struct GeneralSettingsQuery { // Fields filled in by Workstream A. }` (crates/cross-cutting/settings/src/query.rs:17-19); `grep -c "DomainQuery" aggregate.rs entities.rs` returns 0

---

### FINDING 5 (id: `CROSSCUT-SET-005`)

- **Source:** `docs/audit_reports/findings/wave2-settings.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/settings/src/aggregate.rs:1708-1721

**Description:**

`Theme::activate` contains a guard that is meant to
  prevent demoting a default theme to non-default status but the guard
  body is empty — the function silently continues without returning an
  error. The comment says "Demoting a default theme is not allowed
  unless the new theme is also a default" but no `return Err(...)` is
  issued. This means a default theme can be silently demoted at
  runtime, contradicting the spec invariant
  `docs/specs/settings/aggregates.md` § "Theme" item 7
  ("is_default is a boolean; the engine refuses to delete a default
  theme") and the related policy in `services.md`
  (`OneDefaultThemePerSchool`).

**Expected:**

The guard should `return Err(SettingsDomainError::Conflict(...))`
  to enforce the invariant.

**Evidence:**

`if prev.is_default && !self.is_default {
        // Demoting a default theme is not allowed unless
        // the new theme is also a default.
    }` (crates/cross-cutting/settings/src/aggregate.rs:1710-1713)

---

### FINDING 6 (id: `CROSSCUT-SET-006`)

- **Source:** `docs/audit_reports/findings/wave2-settings.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/settings/src/aggregate.rs:1869-1875

**Description:**

`Color::delete` does not check `theme_binding_count`
  before soft-deleting, even though `ColorService::can_delete` (in
  `services.rs:548-556`) is defined to enforce this. Spec
  `docs/specs/settings/aggregates.md` § "Color" defines `Color` as
  referenced by `ColorTheme` rows and `docs/specs/settings/services.md`
  § "ColorService" defines `can_delete(color, theme_binding_count) ->
  Result<(), ConflictError>`. The aggregate-level `delete` method
  takes no count parameter and unconditionally soft-deletes, leaving
  the conflict check only enforceable from outside via the service
  helper.

**Expected:**

Per the spec service contract, `Color::delete` should
  either take a `theme_binding_count: u64` parameter and return
  `Result<(), SettingsDomainError>`, or the dispatcher must always
  call `ColorService::can_delete` before the aggregate method.

**Evidence:**

`/// Soft-deletes the color.
    pub fn delete(&mut self, at: Timestamp, actor: UserId) {
        self.active_status = false;
        self.status = ColorStatus::new(false);` (crates/cross-cutting/settings/src/aggregate.rs:1868-1871)

---

### FINDING 7 (id: `CROSSCUT-SET-007`)

- **Source:** `docs/audit_reports/findings/wave2-settings.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/settings/src/events.rs:243-251

**Description:**

`TimeZoneChanged` uses raw `String` for
  `from_time_zone_id` and `to_time_zone_id`, but the spec
  (`docs/specs/settings/events.md` line 85-86) requires typed
  `Option<TimeZoneId>` and `TimeZoneId`. The spec
  `value-objects.md` line 59 lists `TimeZoneId | From platform`.
  This is a doc-vs-code drift.

**Expected:**

`pub from_time_zone_id: Option<TimeZoneId>, pub
  to_time_zone_id: TimeZoneId,` per the spec.

**Evidence:**

`pub from_time_zone_id: Option<String>,
    pub to_time_zone_id: String,` (crates/cross-cutting/settings/src/events.rs:246-247)

---

### FINDING 8 (id: `CROSSCUT-SET-008`)

- **Source:** `docs/audit_reports/findings/wave2-settings.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** docs/specs/settings/services.md:127

**Description:**

`BehaviorRecordService::patch` (spec line 127)
  takes a `BehaviorRecordPatch` parameter, but no such struct is
  defined anywhere in `entities.rs` or `value_objects.rs` (the
  `BehaviorRecordSetting` aggregate has `apply_update` taking four
  separate `Option<BehaviorFlag>` parameters directly). The spec
  describes a typed patch entity that is absent from the
  implementation, breaking the symmetric patch pattern used by other
  aggregates (e.g. `GeneralSettingsPatch`).

**Expected:**

A `BehaviorRecordPatch` struct should exist per
  `docs/specs/settings/services.md` line 127, with optional fields
  for `student_comment`, `parent_comment`, `student_view`,
  `parent_view`.

**Evidence:**

`pub fn patch(setting: &mut BehaviorRecordSetting, patch: BehaviorRecordPatch) { ... }` (docs/specs/settings/services.md:127); `grep -rn "BehaviorRecordPatch" crates/cross-cutting/settings/` returns 0 matches

---

### FINDING 9 (id: `CROSSCUT-SET-009`)

- **Source:** `docs/audit_reports/findings/wave2-settings.md`
- **Severity:** High
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/settings/src/value_objects.rs:1376-1390

**Description:**

`ColorFormat` is declared as a typed wrapper but is
  not in `docs/specs/settings/value-objects.md` and is not consumed
  anywhere in the crate — it is only referenced in a
  `_suppress_unused_value_object_imports` helper
  (`services.rs:699-703`). This is a dead-code value object that
  silently consumes the `ColorFormat` import namespace and could be
  confused with `CurrencyFormat`.

**Expected:**

Either the spec should describe `ColorFormat` (it
  does not), or the wrapper should be removed. The current state
  leaks an undocumented type into the public prelude via
  `lib.rs:54`.

**Evidence:**

`/// Currency format alias — kept separate from
    /// [\`CurrencyFormat\`] to follow the spec's "ColorFormat"
    /// naming without conflict.
    pub struct ColorFormat(pub String);` (crates/cross-cutting/settings/src/value_objects.rs:1375-1378)

---

### FINDING 10 (id: `CROSSCUT-SET-010`)

- **Source:** `docs/audit_reports/findings/wave2-settings.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/settings/src/value_objects.rs:393-417

**Description:**

`RtlLtl` is declared as a typed wrapper for the
  1=RTL/2=LTR discriminator but is never used outside its own test
  (value_objects.rs:1523-1529). The spec `value-objects.md` line 51
  also lists `RtlLtl` as a value object, but no field in any
  aggregate or command stores `RtlLtl`. The aggregate `Language`
  uses `RtlFlag(pub bool)` (line 1086) for the same concept. The
  duplication is undocumented.

**Expected:**

Either `RtlLtl` should be wired into a field (e.g.
  `GeneralSettings.rtl_ltl`), or removed. The current state has two
  parallel RTL concepts (`RtlFlag` bool and `RtlLtl` i32) with no
  spec'd mapping.

**Evidence:**

`pub struct RtlLtl(pub i32);` (crates/cross-cutting/settings/src/value_objects.rs:395); only references are in the value_objects.rs module tests at lines 1525-1527

---

### FINDING 11 (id: `CROSSCUT-SET-011`)

- **Source:** `docs/audit_reports/findings/wave2-settings.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/settings/src/aggregate.rs:1479-1481

**Description:**

`CustomLink::count_links` uses
  `.try_into().unwrap_or(u32::MAX)` to convert from `usize` to `u32`.
  AGENTS.md § "Type Safety" forbids `as` casts on numerics that may
  lose data; `.unwrap_or(u32::MAX)` is a lossy fallback that returns a
  valid-looking but misleading count when the link list exceeds
  `u32::MAX` (impossible in practice, but the fallback hides bugs
  instead of propagating the error).

**Expected:**

Per AGENTS.md § "Type Safety", use `TryFrom`/`TryInto`
  with proper error handling — return a `Result<u32,
  SettingsDomainError>` instead of silently substituting
  `u32::MAX`.

**Evidence:**

`self.links.len().try_into().unwrap_or(u32::MAX)` (crates/cross-cutting/settings/src/aggregate.rs:1480)

---

### FINDING 13 (id: `CROSSCUT-SET-013`)

- **Source:** `docs/audit_reports/findings/wave2-settings.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/settings/src/aggregate.rs:1938-1962

**Description:**

`BehaviorRecordSetting::apply_update` is named
  inconsistently with the other 14 aggregates which all use a method
  named `update`. The spec describes a single
  `UpdateBehaviorRecordSetting` command and does not specify the
  method name; the asymmetric naming (every other aggregate: `update`
  or `activate`, here: `apply_update`) makes the aggregate API
  non-uniform.

**Expected:**

All aggregates should expose a uniform method name
  (`update`) for the patch mutation, matching the pattern in
  `Language::update`, `BaseGroup::update`, etc.

**Evidence:**

`pub fn apply_update(
        &mut self,
        student_comment: Option<BehaviorFlag>,
        parent_comment: Option<BehaviorFlag>,
        student_view: Option<BehaviorFlag>,
        parent_view: Option<BehaviorFlag>,
        actor: UserId,
        at: Timestamp,
    ) {` (crates/cross-cutting/settings/src/aggregate.rs:1938-1946)

---

### FINDING 14 (id: `CROSSCUT-SET-014`)

- **Source:** `docs/audit_reports/findings/wave2-settings.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/settings/src/aggregate.rs:1159-1165

**Description:**

`Style::delete` returns no `Result`, but the
  service helper `StyleService::can_delete` (services.rs:276-286)
  enforces the "cannot delete default style" invariant. The aggregate
  method unconditionally soft-deletes, leaving the invariant
  enforceable only via the service. The same pattern gap exists for
  `BaseGroup::delete` (line 806), `BaseSetup::delete` (line 895),
  `DateFormat::delete` (line 986), `SetupAdmin::delete` (line 2077),
  `BackgroundSetting::delete` (line 1284), `DashboardSetting::delete`
  (line 1374), `ColorTheme::delete` (line 1569), and `CustomLink`
  (no delete at all — the spec `aggregates.md` § "CustomLink"
  describes UpdateCustomLinks / ResetCustomLinks but the aggregate
  has no `delete`; spec commands.md says ResetCustomLinks is the
  reset action, so this is consistent — see finding 18).

**Expected:**

Either the aggregate `delete` methods should take
  the relevant reference-count parameter and return `Result<(),
  SettingsDomainError>` matching `Theme::delete` and
  `BehaviorRecordSetting::apply_update`, or the spec should explicitly
  require dispatcher-level service checks.

**Evidence:**

`/// Soft-deletes the style.
    pub fn delete(&mut self, at: Timestamp, actor: UserId) {
        self.active_status = false;` (crates/cross-cutting/settings/src/aggregate.rs:1159-1161)

---

### FINDING 16 (id: `CROSSCUT-SET-016`)

- **Source:** `docs/audit_reports/findings/wave2-settings.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/settings/src/aggregate.rs (entire file)

**Description:**

No `workflows.rs` module exists in the settings
  crate, despite `docs/specs/settings/workflows.md` (123 lines)
  defining 8 workflows with 36 enumerated steps (Initial School
  Setup, Language Management, Theme Configuration, Base Setup
  Management, Custom Link Configuration, Date Format Configuration,
  Dashboard Configuration, Two-Factor Configuration). The
  workflows.md spec describes "ordered, conditional steps" but no
  workflow orchestrator or handler is implemented.

**Expected:**

Per `docs/specs/settings/workflows.md` and the
  standard 9-file module layout (`aggregate.rs`, `entities.rs`,
  `value_objects.rs`, `commands.rs`, `events.rs`, `services.rs`,
  `repository.rs`, `query.rs`, `errors.rs`), an additional
  `workflows.rs` module (or equivalent handler dispatch) should be
  present.

**Evidence:**

`grep "pub mod" crates/cross-cutting/settings/src/lib.rs` returns only `aggregate, commands, entities, errors, events, query, repository, services, value_objects` — no `workflows`

---

### FINDING 17 (id: `CROSSCUT-SET-017`)

- **Source:** `docs/audit_reports/findings/wave2-settings.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/settings/src/aggregate.rs:1409-1415

**Description:**

`CustomLink::new` initialises an empty bundle with
  `links: Vec::new()` (no links). The spec
  `docs/specs/settings/aggregates.md` § "CustomLink" item 4 says "Each
  link must be a valid URL or empty" but the constructor accepts no
  parameters and unconditionally creates an empty bundle, while the
  spec commands.md § "Custom Link" describes only `UpdateCustomLinks`
  and `ResetCustomLinks` — there is no `AddCustomLink` /
  `DeleteCustomLink` per-link command. This means individual link
  entries cannot be added or removed; the entire bundle must be
  replaced atomically. The `links: Vec<(LinkLabel, LinkHref)>`
  field is unnamed (no entity wrapper for individual entries) even
  though `docs/specs/settings/entities.md` § "CustomLinkEntry"
  defines a typed projection. The aggregate embeds `Vec<(LinkLabel,
  LinkHref)>` directly instead of using the entity.

**Expected:**

The aggregate should hold `links: Vec<CustomLinkEntry>`
  (per `entities.md` § "CustomLinkEntry") so individual entry
  invariants can be enforced at the entity level.

**Evidence:**

`pub links: Vec<(LinkLabel, LinkHref)>,` (crates/cross-cutting/settings/src/aggregate.rs:1395)

---

### FINDING 18 (id: `CROSSCUT-SET-018`)

- **Source:** `docs/audit_reports/findings/wave2-settings.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/settings/src/entities.rs:598-629

**Description:**

`SettingsAuditEntry::new` defaults
  `school_id: SchoolId::from_uuid(Uuid::nil())` rather than taking
  the `SchoolId` as a constructor parameter. This means every
  audit entry created without explicit override will be attributed to
  the nil school — a data-integrity hazard for a multi-tenant
  system. Per AGENTS.md § "Engine Rules" item 7: "Multi-tenant by
  default. Every aggregate has a SchoolId."

**Expected:**

`SettingsAuditEntry::new` should take a `school_id:
  SchoolId` parameter (and all other fields) explicitly, with no
  `nil` default.

**Evidence:**

`school_id: SchoolId::from_uuid(Uuid::nil()),
        entry_id: Uuid::new_v4(),
        aggregate_type: aggregate_type.into(),` (crates/cross-cutting/settings/src/entities.rs:613-615)

---

### FINDING 19 (id: `CROSSCUT-SET-019`)

- **Source:** `docs/audit_reports/findings/wave2-settings.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/settings/src/aggregate.rs:178-187

**Description:**

`GeneralSettings::new` initialises
  `email_driver: EmailDriver("smtp".to_owned())` and
  `preloader_type: PreloaderType(1)` via direct struct construction
  rather than the validated constructors `EmailDriver::new` and
  `PreloaderType::new`. The code includes explicit comments
  (`// "smtp" is a valid 1..64 char string by spec; bypass
  validation.` and `// 1 is a valid PreloaderType by spec; bypass
  validation.`) acknowledging this is a deliberate bypass. This
  violates the spec invariant `docs/specs/settings/value-objects.md`
  § "Validation Rules": "Construction is the only entry point".
  Bypassing validation in the constructor is a documented anti-pattern.

**Expected:**

Use `EmailDriver::new("smtp")?` and
  `PreloaderType::new(1)?` (or store as raw fields if the spec
  permits, but the spec mandates `Validate` at construction).

**Evidence:**

`// "smtp" is a valid 1..64 char string by spec; bypass validation.
            email_driver: EmailDriver("smtp".to_owned()),` (crates/cross-cutting/settings/src/aggregate.rs:177-178); `// 1 is a valid PreloaderType by spec; bypass validation.
            preloader_type: PreloaderType(1),` (crates/cross-cutting/settings/src/aggregate.rs:186-187)

---

### FINDING 20 (id: `CROSSCUT-SET-020`)

- **Source:** `docs/audit_reports/findings/wave2-settings.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/settings/src/value_objects.rs:395-417

**Description:**

`RtlLtl::new` returns `Result<Self, DomainError>`
  (line 398), but the `prelude` re-exports it via
  `crates/cross-cutting/settings/src/lib.rs:61`. The `prelude` does
  not include `DomainError` (the prelude only re-exports
  `SettingsDomainError` from `errors::Result`). Consumers using
  `RtlLtl::new` from the prelude will not have a stable `DomainError`
  import path. (See also FINDING 10 — `RtlLtl` is dead code
  outside its own tests.)

**Expected:**

Either remove the dead `RtlLtl` value object or
  re-export the required `DomainError` through the prelude.

**Evidence:**

`pub fn new(v: i32) -> Result<Self> {` (crates/cross-cutting/settings/src/value_objects.rs:398); `pub use crate::errors::{Result, SettingsDomainError};` (crates/cross-cutting/settings/src/lib.rs:50) — `DomainError` is not re-exported.

---

### FINDING 21 (id: `CROSSCUT-SET-021`)

- **Source:** `docs/audit_reports/findings/wave2-settings.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/settings/src/aggregate.rs:1480

**Description:**

`CustomLink::count_links` uses
  `self.links.len().try_into().unwrap_or(u32::MAX)`. The
  `try_into` succeeds for any list under `u32::MAX` items, but the
  silent fallback masks any future ABI/type change. This is the
  same numeric-conversion anti-pattern flagged in AGENTS.md § "Type
  Safety". The aggregate uses `Vec` (untyped) instead of a typed
  `CustomLinkEntry` (see FINDING 17) so there is no compile-time
  cap on `links.len()`.

**Expected:**

Replace with an explicit
  `u32::try_from(self.links.len()).map_err(|_| SettingsDomainError::Validation(...))?`.

**Evidence:**

`self.links.len().try_into().unwrap_or(u32::MAX)` (crates/cross-cutting/settings/src/aggregate.rs:1480)

---

### FINDING 26 (id: `CROSSCUT-SET-026`)

- **Source:** `docs/audit_reports/findings/wave2-settings.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/settings/src/services.rs:601-643

**Description:**

`OnlyOneActiveStyle` and `OneDefaultThemePerSchool`
  policies (services.rs:601-643) are defined as plain structs with
  inherent `check` methods, but `docs/specs/settings/services.md`
  lines 169-184 show them implementing a `Policy<C>` trait:
  `impl Policy<ActivateStyleCommand> for OnlyOneActiveStyle` and
  `impl Policy<CreateThemeCommand> for OneDefaultThemePerSchool`. The
  spec trait-based dispatch is replaced with free-function inherent
  methods (`check(target: &Style, others: &[Style])` and
  `check(target: &Theme, others: &[Theme])`) which take different
  parameter shapes than the spec (the spec takes the command,
  e.g. `ActivateStyleCommand`, not the resolved aggregate). This
  breaks the policy-spec-to-dispatcher wiring described in the spec.

**Expected:**

Per spec, policies should implement `Policy<C>` and
  receive the command context; the current implementation deviates
  from this contract.

**Evidence:**

`pub struct OnlyOneActiveStyle;

impl OnlyOneActiveStyle {
    /// Returns \`Ok(())\` if activating \`target\` is allowed (no other
    /// style is currently active).
    pub fn check(target: &Style, others: &[Style]) -> Result<(), &'static str> {` (crates/cross-cutting/settings/src/services.rs:601-606); spec: `impl Policy<ActivateStyleCommand> for OnlyOneActiveStyle` (docs/specs/settings/services.md:170)

---

### FINDING 3 (id: `CROSSCUT-SET-003`)

- **Source:** `docs/audit_reports/findings/wave2-settings.md`
- **Severity:** Medium
- **Area:** cross-cutting
- **Location:** docs/specs/settings/tables.md:13-22

**Description:**

`docs/specs/settings/tables.md` lists 14 owned
  settings tables (`settings_behaviour_record_settings`,
  `settings_colors`, `settings_color_theme`,
  `settings_background_settings`, `settings_base_setups`,
  `settings_custom_links`, `settings_dashboard_settings`,
  `settings_date_formats`, `settings_general_settings`,
  `settings_languages`, `settings_language_phrases`,
  `settings_setup_admins`, `settings_styles`, `settings_themes`),
  but the `BaseGroup` aggregate has no corresponding
  `settings_base_groups` table row in the table — the table only
  mentions `settings_base_setups` (line 13). However, the spec notes
  on line 81 reference `settings_base_groups.id` as the FK target of
  `settings_base_setups.base_group_id`. This is a doc-vs-spec drift
  on the table list itself.

**Expected:**

Per `docs/specs/settings/tables.md` § "Notes" line 81:
  "`settings_base_setups.base_group_id` references
  `settings_base_groups.id` and cascades on delete." — therefore
  `settings_base_groups` should appear as an owned table row.

**Evidence:**

`| \`settings_base_setups\`               | BaseSetup                 | Lookup values                      |` (docs/specs/settings/tables.md:13); `- \`settings_base_setups.base_group_id\` references \`settings_base_groups.id\`` (docs/specs/settings/tables.md:81)

---

### FINDING 22 (id: `CROSSCUT-SET-022`)

- **Source:** `docs/audit_reports/findings/wave2-settings.md`
- **Severity:** Low
- **Area:** cross-cutting
- **Location:** docs/specs/settings/commands.md:8, crates/cross-cutting/settings/src/commands.rs:151-204

**Description:**

`SeedGeneralSettingsCommand` is declared in
  `commands.rs:151-204` (with `COMMAND_TYPE = "settings.general_settings.seed"`)
  but is not enumerated in `docs/specs/settings/commands.md` or in
  `docs/commands/settings.md`. This is a spec-vs-code drift — the
  spec does not describe a "seed" command, and `docs/handoff/PHASE-14-HANDOFF.md`
  line 27 advertises "53 typed settings commands" but
  `commands.rs` has 54 typed command structs (1 extra: `SeedGeneralSettingsCommand`).

**Expected:**

Either add the `SeedGeneralSettingsCommand` to the
  spec docs/commands/settings.md and docs/specs/settings/commands.md,
  or remove it from `commands.rs` if it is unused.

**Evidence:**

`pub const COMMAND_TYPE: &'static str = "settings.general_settings.seed";` (crates/cross-cutting/settings/src/commands.rs:174); the command does not appear in docs/specs/settings/commands.md (1-405) or docs/commands/settings.md (1-69)

---

### FINDING 23 (id: `CROSSCUT-SET-023`)

- **Source:** `docs/audit_reports/findings/wave2-settings.md`
- **Severity:** Low
- **Area:** cross-cutting
- **Location:** docs/handoff/PHASE-14-HANDOFF.md:26-27

**Description:**

The handoff advertises "53 typed settings events"
  and "53 typed settings commands" but the actual code
  (`events.rs` and `commands.rs`) defines 52 event structs and 54
  command structs. The discrepancy is small but breaks the
  hand-off's own numerical claims: events.rs has 52 `pub struct`
  definitions matching `docs/events/settings.md` (52 rows); commands.rs
  has 54 with one extra (`SeedGeneralSettingsCommand`, see FINDING 22).

**Expected:**

The handoff numbers should reconcile with the code:
  either bump to "54 commands" (and add `Seed` to the catalog) or
  drop `Seed` from the code.

**Evidence:**

`**53 typed settings events** + **25 typed operations events**` (docs/handoff/PHASE-14-HANDOFF.md:26); `awk '/^pub struct /{count++}' crates/cross-cutting/settings/src/events.rs` returns 52; `awk '/^pub struct .*Command /' crates/cross-cutting/settings/src/commands.rs` returns 54

---

### FINDING 24 (id: `CROSSCUT-SET-024`)

- **Source:** `docs/audit_reports/findings/wave2-settings.md`
- **Severity:** Low
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/settings/src/value_objects.rs:6-7

**Description:**

`value_objects.rs` declares
  `#![allow(dead_code, clippy::all)]` (line 6) and
  `#![allow(missing_docs)]` (line 7) at module level. Per AGENTS.md
  § "Type Safety": "No `#[allow(dead_code)]` ... to silence the
  compiler. Delete unused code, wire it in, or open a follow-up
  issue." The blanket `allow(dead_code)` masks dead-code findings
  (e.g. FINDING 10: `RtlLtl`, FINDING 9: `ColorFormat`) instead of
  removing the unused declarations. `aggregate.rs` (line 9) and
  `entities.rs` (line 6) carry the same blanket `allow(missing_docs,
  dead_code, clippy::all)`.

**Expected:**

Remove the blanket `allow(dead_code)` (and the
  unused value objects it masks), per AGENTS.md § "Type Safety".

**Evidence:**

`#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]` (crates/cross-cutting/settings/src/value_objects.rs:6-7)

---

### FINDING 25 (id: `CROSSCUT-SET-025`)

- **Source:** `docs/audit_reports/findings/wave2-settings.md`
- **Severity:** Low
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/settings/src/events.rs:7, crates/cross-cutting/settings/src/commands.rs:7, crates/cross-cutting/settings/src/services.rs:7, crates/cross-cutting/settings/src/repository.rs:8, crates/cross-cutting/settings/src/query.rs:7

**Description:**

Each of `events.rs`, `commands.rs`, `services.rs`,
  `repository.rs`, `query.rs` declares
  `#![allow(dead_code, clippy::all)]` at module level. Per AGENTS.md
  § "Type Safety": "No `#[allow(dead_code)]` ... to silence the
  compiler. Delete unused code, wire it in, or open a follow-up
  issue." The blanket allowance masks any dead-code warnings that
  might point to unwired pieces (e.g. policy types, query stubs).

**Expected:**

Remove the blanket `allow(dead_code)` annotations.

**Evidence:**

`#![allow(dead_code, clippy::all)]` (crates/cross-cutting/settings/src/events.rs:7); same pattern in commands.rs:7, services.rs:7, repository.rs:8, query.rs:7

---

### FINDING 27 (id: `CROSSCUT-SET-027`)

- **Source:** `docs/audit_reports/findings/wave2-settings.md`
- **Severity:** Low
- **Area:** cross-cutting
- **Location:** crates/cross-cutting/settings/src/aggregate.rs:154-155, crates/cross-cutting/settings/src/aggregate.rs:518-519

**Description:**

Multiple aggregates use the pattern
  `school_id: cmd.id.school_id()` to derive `school_id` from the id
  (e.g. `GeneralSettings::new`, `Language::new`, `BaseGroup::new`,
  `BaseSetup::new`, `DateFormat::new`, `Style::new`,
  `BackgroundSetting::new`, `DashboardSetting::new`, `Theme::new`,
  `BehaviorRecordSetting::new`, `SetupAdmin::new`). The `Color`,
  `ColorTheme` aggregates are correctly global and have no
  `school_id`, but `GeneralSettings::new` (line 154) calls
  `cmd.id.school_id()` BEFORE initialising `id: cmd.id` (line 155),
  relying on field-order semantics. This works because `cmd.id` is
  `Copy` (via `SchoolId` + `Uuid`) but is a fragile pattern that
  relies on the macro-generated `settings_typed_id!` impl (which
  returns `school_id` by value at line 47 of `value_objects.rs`).
  If the id macro is changed to non-`Copy`, every constructor in
  the file would break.

**Expected:**

Either initialise `id` first then derive `school_id`
  in a second statement, or rely on a `cmd.id.school_id()` call after
  `id` is assigned.

**Evidence:**

`school_id: cmd.id.school_id(),
            id: cmd.id,
            school_name: cmd.school_name,` (crates/cross-cutting/settings/src/aggregate.rs:154-156)

---

### FINDING 28 (id: `CROSSCUT-SET-028`)

- **Source:** `docs/audit_reports/findings/wave2-settings.md`
- **Severity:** Low
- **Area:** cross-cutting
- **Location:** docs/specs/settings/tables.md (full file)

**Description:**

`docs/specs/settings/tables.md` lists 14 owned
  settings tables and 6 cross-domain tables (total 20 rows including
  duplicates: `settings_languages` and `settings_date_formats`
  appear twice each — once owned, once cross-domain-referenced).
  The spec text duplicates these table names without flagging the
  duplication, and the table is missing the `settings_base_groups`
  row despite the FK reference on line 81. The 14 owned tables
  correspond to 15 root aggregates (the discrepancy is `BaseGroup`
  having no table row). This is a doc-vs-spec drift.

**Expected:**

A unified, deduplicated table list with every root
  aggregate mapped to one owned table (including `settings_base_groups`).

**Evidence:**

`awk -F'|' '/^\| `/{gsub(/^ +| +$/, "", $2); print $2}' docs/specs/settings/tables.md` returns 20 rows with `settings_languages` and `settings_date_formats` each appearing twice

### END FINDINGS

**Total findings:** 28

---


## Events (envelope) (target id prefix: `CC-EVT`)

**Path:** `crates/cross-cutting/events/`  
**Total findings:** 28 (6 critical, 6 high, 12 medium, 4 low)


### FINDING 1 (id: `CC-EVT-001`)

- **Source:** `docs/audit_reports/findings/wave2-events.md`
- **Severity:** Critical
- **Area:** cross-cutting-events
- **Location:** `crates/cross-cutting/events/src/envelope.rs:47-103`

**Description:**

`EventEnvelope` is missing the `recorded_at: Timestamp` field that `docs/schemas/event-schema.md` § 1 declares as a mandatory wire-format field. The schema spec explicitly defines `recorded_at` as "the clock time of persistence (>= occurred_at)" and shows it in the canonical JSON sample (§ 3), but the Rust envelope only carries `occurred_at` and `published_at`. `published_at` is set by the bus adapter on `publish` and is not equivalent to `recorded_at` (which is set at outbox persistence time per the storage-port `EventLogEntry::from_serialized_envelope` at `crates/infra/storage/src/event_log.rs:175-187`). Events emitted through `into_envelope` carry no `recorded_at` and consumers cannot compute ingestion latency from the envelope alone.

**Expected:**

`docs/schemas/event-schema.md:32` (`recorded_at: Timestamp`) and `:77` (JSON sample) declare `recorded_at` as a required wire field.

**Evidence:**

```rust
  pub struct EventEnvelope {
      pub event_id: EventId,
      pub event_type: &'static str,
      pub schema_version: u32,
      pub school_id: SchoolId,
      pub aggregate_id: Uuid,
      pub aggregate_type: &'static str,
      pub actor_id: UserId,
      pub correlation_id: CorrelationId,
      pub causation_id: Option<EventId>,
      pub occurred_at: Timestamp,
      pub published_at: Option<Timestamp>,
      pub payload: serde_json::Value,
  }
  ```

---

### FINDING 2 (id: `CC-EVT-002`)

- **Source:** `docs/audit_reports/findings/wave2-events.md`
- **Severity:** Critical
- **Area:** cross-cutting-events
- **Location:** `crates/cross-cutting/events/src/envelope.rs:47-103`

**Description:**

`EventEnvelope` carries no `metadata: EventMetadata` field, but `docs/schemas/event-schema.md` § 1 declares `metadata: EventMetadata` (an open-ended, versioned key map) as a required envelope field and § 6 enumerates the recommended keys (`source`, `user_agent`, `ip`, `request_id`, `device_id`, `session_id`, `geo`, `feature_flags`, `trace`). The audit / outbox / central-fan-out / sync consumers cannot stamp distributed-trace ids, source-channel (`web` / `mobile` / `api` / `agent` / `import` / `system`), or request id onto the wire without a field on the envelope. The `RawPayload` helper at `domain_event.rs:174-204` exists but carries only `correlation_id` and `actor_id` and is never invoked by any producer.

**Expected:**

`docs/schemas/event-schema.md:39` (`metadata: EventMetadata`) and § 6 key catalogue.

**Evidence:**

```rust
  // No `metadata` field in the struct above. The schema spec
  // requires metadata; the field is absent.
  ```
  And at `crates/cross-cutting/events/src/domain_event.rs:178-204`:
  ```rust
  pub struct RawPayload {
      pub payload: serde_json::Value,
      pub correlation_id: CorrelationId,
      pub actor_id: UserId,
  }
  ```
  No `metadata` carrier exists; `RawPayload::new` is never called from any other crate (`grep RawPayload crates/` returns only the definition site and its `impl` block).

---

### FINDING 3 (id: `CC-EVT-003`)

- **Source:** `docs/audit_reports/findings/wave2-events.md`
- **Severity:** Critical
- **Area:** cross-cutting-events
- **Location:** `crates/cross-cutting/events/src/event_bus.rs:55-65, 84-117` and `docs/ports/event-bus.md:60-66`

**Description:**

`EventSubscription::ack` and `EventSubscription::nack` return `Result<AckOutcome>` in code, but the bus-port contract at `docs/ports/event-bus.md:60-66` specifies `Result<()>` for both. The deviation is non-trivial: any consumer written against the port-doc signature will not compile (or will silently discard the `AckOutcome::Unknown` / `Failed` discriminant) against the actual trait. Additionally, `AckOutcome::Unknown` is unrepresentable in the spec contract; the spec requires that `ack` be idempotent and the return type be `()`. There is no migration note in either the bus-port doc or the Phase 2 hand-off acknowledging this divergence.

**Expected:**

`docs/ports/event-bus.md:60-66`:
  ```rust
  async fn ack(&mut self, event_id: EventId) -> Result<()>;
  async fn nack(&mut self, event_id: EventId, requeue: bool) -> Result<()>;
  ```

**Evidence:**

```rust
  async fn ack(&mut self, event_id: EventId) -> Result<AckOutcome>;
  async fn nack(&mut self, event_id: Event_id: EventId, requeue: bool) -> Result<AckOutcome>;
  ```

---

### FINDING 4 (id: `CC-EVT-004`)

- **Source:** `docs/audit_reports/findings/wave2-events.md`
- **Severity:** Critical
- **Area:** cross-cutting-events
- **Location:** `crates/cross-cutting/events/src/domain_event.rs:54-122` and `docs/specs/events/events.md:10-16`

**Description:**

The `DomainEvent` trait shipped in code does not match the shape declared in `docs/specs/events/events.md:10-16`. The spec declares:
  ```rust
  pub trait DomainEvent: Serialize + DeserializeOwned + Send + Sync {
      const TYPE: &'static str;
      fn aggregate_id(&self) -> Uuid;
      fn school_id(&self) -> SchoolId;
      fn occurred_at(&self) -> Timestamp;
  }
  ```
  but the code declares `DomainEvent: Send + Sync + 'static` (no `Serialize + DeserializeOwned` bound), renames the const from `TYPE` to `EVENT_TYPE`, and adds two extra consts (`SCHEMA_VERSION`, `AGGREGATE_TYPE`) plus an `event_id()` getter. All 10+ domain impls in `crates/domains/*/src/events.rs` (e.g. `crates/domains/finance/src/events.rs:71-73`, `crates/domains/hr/src/events.rs:121-123`) implement the code-shape, not the spec-shape, so the spec and the code have drifted in opposite directions: code has more required surface, spec has fewer required bounds.

**Expected:**

`docs/specs/events/events.md:10-16` spec text quoted above.

**Evidence:**

```rust
  pub trait DomainEvent: Send + Sync + 'static {
      const EVENT_TYPE: &'static str;
      const SCHEMA_VERSION: u32;
      const AGGREGATE_TYPE: &'static str;
      fn event_id(&self) -> EventId;
      fn aggregate_id(&self) -> Uuid;
      fn school_id(&self) -> SchoolId;
      fn occurred_at(&self) -> Timestamp;
      fn to_value(&self) -> serde_json::Value where Self: Serialize { ... }
      fn into_envelope(self, ctx: &TenantContext) -> EventEnvelope where Self: Sized + Serialize { ... }
  }
  ```

---

### FINDING 5 (id: `CC-EVT-005`)

- **Source:** `docs/audit_reports/findings/wave2-events.md`
- **Severity:** Critical
- **Area:** cross-cutting-events
- **Location:** `crates/cross-cutting/events/src/` (whole crate) and `docs/schemas/event-schema.md:128-145`

**Description:**

`docs/schemas/event-schema.md` § 7 ("Schema Registry") mandates a port with two public methods: `engine.events.list()` returning `(event_type, current_version, deprecated_versions)`, and `engine.events.schema(event_type, version)` returning the JSON schema. The `educore-events` crate ships no such port, no `SchemaRegistry` trait, no in-memory default implementation, and no re-export of an `engine.events` facade. Phase 2 hand-off § "Open questions" does not mention this gap. The 6th cross-cutting table (`schema_registry`) referenced in `docs/build-plan.md` Phase 2 task 6 has a DDL but no port surface for it in `educore-events`.

**Expected:**

`docs/schemas/event-schema.md:128-145` (Schema Registry section quoted above).

**Evidence:**

```text
  $ grep -rn 'SchemaRegistry\|schema_registry\|events.list\|events.schema' crates/cross-cutting/events/
  (no matches)
  ```

---

### FINDING 6 (id: `CC-EVT-006`)

- **Source:** `docs/audit_reports/findings/wave2-events.md`
- **Severity:** Critical
- **Area:** cross-cutting-events
- **Location:** `crates/cross-cutting/events/src/event_bus.rs:52-66` and `docs/ports/event-bus.md:22-26`

**Description:**

`EventBus::subscribe` is declared as `async fn subscribe(&self, options: SubscribeOptions) -> Result<Box<dyn EventSubscription>>` in code, but the bus-port contract at `docs/ports/event-bus.md:22-26` declares it as `Result<EventSubscription>` (no box, no `dyn`). The deviation from the doc is consistent with object-safety, but the spec doc is the contract that downstream consumers (Phase 3+ domain subscribers) will code against. The worked-example subscription code in `docs/ports/event-bus.md:177-197` writes `let mut sub = engine.events().subscribe(...)` without any `Box<dyn _>` deref, so it cannot compile against the actual trait signature.

**Expected:**

`docs/ports/event-bus.md:24` `async fn subscribe(&self, options: SubscribeOptions) -> Result<EventSubscription>;`

**Evidence:**

```rust
  async fn subscribe(&self, options: SubscribeOptions) -> Result<Box<dyn EventSubscription>>;
  ```
  vs.
  ```rust
  // From docs/ports/event-bus.md worked example:
  let mut sub = engine.events().subscribe(SubscribeOptions { ... }).await?;
  ```

---

### FINDING 10 (id: `CC-EVT-010`)

- **Source:** `docs/audit_reports/findings/wave2-events.md`
- **Severity:** High
- **Area:** cross-cutting-events
- **Location:** `crates/cross-cutting/events/src/event_bus.rs:159-183, 220-244` and `crates/adapters/event-bus/src/in_process.rs:301-318`

**Description:**

The two filter-routing paths in the bus port — `Topic::Aggregate(d, a)` matched in the in-process adapter via `topic_matches`, and `EventFilter::AggregateType(t)` matched via `filter_matches` — disagree on what "aggregate" means. `Topic::Aggregate("platform", "school")` matches an envelope whose `aggregate_topic()` (i.e. domain prefix + `aggregate_type`) equals `platform.school`. `EventFilter::AggregateType("school")` matches the envelope's raw `aggregate_type` field, which is just `"school"`. A subscriber that subscribes to `Topic::Aggregate("platform", "school")` AND attaches a defensive `EventFilter::AggregateType("school")` to the same `SubscribeOptions` will receive an envelope that passes the topic check but fail the filter check on every event whose `aggregate_type` contains the domain prefix (e.g. `platform_school` in some encodings). The two naming schemes are mixed in the same file and have no shared helper.

**Expected:**

Either both `Topic` and `EventFilter` operate on the same field, or the docs explicitly state the asymmetry.

**Evidence:**

```rust
  // In topic_matches (crates/adapters/event-bus/src/in_process.rs:301-318):
  Topic::Aggregate(d, a) => env.aggregate_topic() == format!("{d}.{a}"),

  // In filter_matches (crates/cross-cutting/events/src/event_bus.rs:220-244):
  Self::AggregateType(t) => envelope.aggregate_type == *t,
  ```
  Note: `aggregate_topic()` (envelope.rs:111-122) returns `{domain}.{aggregate_type}`, e.g. `platform.school`; `aggregate_type` is just `school`.

---

### FINDING 11 (id: `CC-EVT-011`)

- **Source:** `docs/audit_reports/findings/wave2-events.md`
- **Severity:** High
- **Area:** cross-cutting-events
- **Location:** `crates/cross-cutting/events/src/event_bus.rs:30-66`

**Description:**

The `EventBus` port trait has no retry / DLQ / requeue configuration surface. Per `docs/ports/event-bus.md:104-108` ("Events that fail repeatedly (configurable N retries) are routed to a dead letter queue") and ADR-005 ("the outbox + relay pattern is mandatory for at-least-once delivery"), the bus is contracted to support configurable retries and a DLQ. The trait carries `nack(requeue: bool)` but no `max_retries` / `dead_letter_topic` / `visibility_timeout` knob; consumers have no way to opt into retry-with-backoff behaviour. The in-process bus hard-codes `AckOutcome::Accepted` for all acks/nacks (see CC-EVT-008), so the entire retry + DLQ stack is unimplemented for the default adapter.

**Expected:**

Bus-port contract § "Dead Letter Queue" (quoted); `docs/ports/event-bus.md:104-108`.

**Evidence:**

```rust
  #[async_trait]
  pub trait EventBus: Send + Sync + fmt::Debug {
      async fn publish(&self, envelope: EventEnvelope) -> Result<PublishReceipt>;
      async fn publish_batch(&self, envelopes: Vec<EventEnvelope>) -> Result<BatchReceipt>;
      async fn subscribe(&self, options: SubscribeOptions) -> Result<Box<dyn EventSubscription>>;
  }
  ```
  No `retry`, `dlq`, or `requeue` API exists on the trait.

---

### FINDING 12 (id: `CC-EVT-012`)

- **Source:** `docs/audit_reports/findings/wave2-events.md`
- **Severity:** High
- **Area:** cross-cutting-events
- **Location:** `crates/cross-cutting/events/src/domain_event.rs:133-138` and `crates/cross-cutting/events/src/lib.rs:59` and `crates/cross-cutting/platform/src/lib.rs:86` and `crates/domains/hr/src/events.rs:55`

**Description:**

The `EventFactory` trait is declared as a "recommended pattern" template in `domain_event.rs:133-138` and re-exported from the prelude of `educore-events`, `educore-platform`, and `educore-hr`, but no `impl EventFactory for SomeEvent` exists anywhere in the workspace (`grep "impl EventFactory" crates/` returns zero results). The Phase 2 hand-off § "What's wired" documents `EventFactory` as shipped. The 4 sync events in `sync.rs` define `now()` / `for_session()` constructors that do not satisfy the `fn mint(occurred_at, event_id) -> Self` signature. Consumers reading the prelude believe they have a typed builder; they don't.

**Expected:**

Either remove the trait (and its re-exports) until a domain implements it, or implement it for the sync events and the platform events to match the documented intent.

**Evidence:**

```rust
  pub trait EventFactory: DomainEvent + Sized {
      /// Mint a new event with a fresh `event_id` and the given
      /// `occurred_at`.
      #[must_use]
      fn mint(occurred_at: Timestamp, event_id: EventId) -> Self;
  }
  ```
  No implementation exists in the workspace.

---

### FINDING 7 (id: `CC-EVT-007`)

- **Source:** `docs/audit_reports/findings/wave2-events.md`
- **Severity:** High
- **Area:** cross-cutting-events
- **Location:** `crates/cross-cutting/events/src/event_bus.rs:280-296`

**Description:**

`BatchReceipt::is_fully_accepted()` documents itself as "Returns `true` if every receipt in the batch succeeded" but its implementation is `!self.receipts.is_empty()`. The body of the function contradicts the doc comment: a batch with one successful and one failed receipt will report `is_fully_accepted() == true`. A producer relying on this method to gate downstream work (e.g. an outbox relay that only deletes the source rows when the batch is fully accepted) will silently delete rows for partially-failed batches. The in-process bus adapter at `crates/adapters/event-bus/src/in_process.rs:202-219` short-circuits on the first failure inside `publish_batch`, so the bug is latent today, but the doc/API contract is wrong.

**Expected:**

Doc comment matches behaviour: `is_fully_accepted` should iterate `receipts` and assert a per-receipt success field, OR the doc should be rewritten to "returns true iff any receipt was produced" and the field set extended with a per-receipt status.

**Evidence:**

```rust
  /// Returns `true` if every receipt in the batch succeeded.
  /// Adapters that don't support atomic batching always return
  /// `true` here (the per-receipt `PublishReceipt` carries the
  /// per-envelope status, but the trait doesn't model
  /// per-receipt failure for batches; producers that need
  /// that granularity call `publish` in a loop).
  #[must_use]
  pub fn is_fully_accepted(&self) -> bool {
      !self.receipts.is_empty()
  }
  ```

---

### FINDING 8 (id: `CC-EVT-008`)

- **Source:** `docs/audit_reports/findings/wave2-events.md`
- **Severity:** High
- **Area:** cross-cutting-events
- **Location:** `crates/adapters/event-bus/src/in_process.rs:472-487`

**Description:**

The in-process bus `ack` and `nack` are no-ops that return `AckOutcome::Accepted` without doing anything. Per `docs/ports/event-bus.md:104-108` ("Dead Letter Queue. Events that fail repeatedly (configurable N retries) are routed to a dead letter queue") and ADR-005 § "Decision" item 4 ("outbox + relay pattern is mandatory for at-least-once delivery"), the bus-port promises at-least-once delivery with retry + DLQ. The in-process adapter has no retry counter, no DLQ sink, no `requeue` semantics on `nack`, and no `visibility_timeout` enforcement — the `visibility_timeout` field on `SubscribeOptions` is consumed by no code path. A consumer that calls `nack(id, true)` expecting the envelope to re-arrive will silently lose it.

**Expected:**

Bus-port contract § "Dead Letter Queue" and § "At-Least-Once Delivery"; `docs/ports/event-bus.md:104-108`.

**Evidence:**

```rust
  async fn ack(&mut self, _event_id: EventId) -> educore_core::error::Result<AckOutcome> {
      // In-process delivery is direct; ack is a no-op.
      Ok(AckOutcome::Accepted)
  }

  async fn nack(
      &mut self,
      _event_id: EventId,
      _requeue: bool,
  ) -> educore_core::error::Result<AckOutcome> {
      // In-process delivery is direct; nack is a no-op.
      Ok(AckOutcome::Accepted)
  }
  ```

---

### FINDING 9 (id: `CC-EVT-009`)

- **Source:** `docs/audit_reports/findings/wave2-events.md`
- **Severity:** High
- **Area:** cross-cutting-events
- **Location:** `crates/cross-cutting/events/src/domain_event.rs:107-115` and `crates/cross-cutting/events/src/outbox.rs:32-46`

**Description:**

Both `DomainEvent::to_value` (the default payload serializer) and `outbox::payload_bytes` / `outbox::envelope_bytes` silently swallow serialization failures via `.unwrap_or(serde_json::Value::Null)` / `.unwrap_or_default()`. A producer that constructs an event with an unserializable field (e.g. a non-`Serialize` value inside a `serde_json::Value` it didn't construct itself, a `SecretString` without `Serialize`, a f32 NaN, or any future `Serialize` impl bug) will publish a `payload: null` envelope onto the bus. Consumers that do not null-check will then deserialize the broken event and crash, or worse, store a meaningless `null` payload in the event log. The doc-comment on `to_value` notes the relaxation but does not flag it as a contract violation. `AGENTS.md` forbids `unwrap`/`expect`/`panic` in production paths; `.unwrap_or_default()` here is a silent form of the same anti-pattern.

**Expected:**

Either (a) `to_value` returns `Result<serde_json::Value, EventError>` and `payload_bytes` propagates the error, or (b) the outbox/appender layer rejects the event with a `DomainError::Validation` before the bad payload reaches the bus.

**Evidence:**

```rust
  fn to_value(&self) -> serde_json::Value
  where
      Self: Serialize,
  {
      serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
  }
  ```
  And at `crates/cross-cutting/events/src/outbox.rs:32-46`:
  ```rust
  pub fn payload_bytes(envelope: &EventEnvelope) -> bytes::Bytes {
      bytes::Bytes::from(serde_json::to_vec(&envelope.payload).unwrap_or_default())
  }
  ```

---

### FINDING 13 (id: `CC-EVT-013`)

- **Source:** `docs/audit_reports/findings/wave2-events.md`
- **Severity:** Medium
- **Area:** cross-cutting-events
- **Location:** `crates/cross-cutting/events/src/domain_event.rs:54-58` and `docs/specs/events/events.md:10-16`

**Description:**

The `DomainEvent` trait bound in code is `Send + Sync + 'static`, but the events-domain spec (`docs/specs/events/events.md:10-16`) and `docs/schemas/event-schema.md` § 12 ("Event Immutability") imply events must be `Serialize + DeserializeOwned` so the engine can persist them in the outbox and replay them from the event log. The default `into_envelope` helper at `domain_event.rs:107-115` only requires `Self: Sized + Serialize`, and the outbox helper `SerializedEnvelope::from_event_envelope` at `crates/infra/storage/src/outbox.rs:139-160` calls `serde_json::to_vec(&envelope.payload)` — meaning the engine implicitly assumes every event's `payload: serde_json::Value` is serializable (fine, it's `Value`) but the *typed event struct itself* has no `Deserialize` bound, so an event stored in the event log as JSON cannot be reconstructed into the typed struct by the engine.

**Expected:**

Either add `DeserializeOwned` to the trait bound (matching the spec), or document the asymmetry: the typed event is publish-only; the event-log row is the durable record; consumers re-materialise typed events themselves.

**Evidence:**

```rust
  pub trait DomainEvent: Send + Sync + 'static { ... }
  ```
  Spec: `pub trait DomainEvent: Serialize + DeserializeOwned + Send + Sync { ... }` (`docs/specs/events/events.md:10-16`).

---

### FINDING 14 (id: `CC-EVT-014`)

- **Source:** `docs/audit_reports/findings/wave2-events.md`
- **Severity:** Medium
- **Area:** cross-cutting-events
- **Location:** `crates/cross-cutting/events/src/event_bus.rs:159-170`

**Description:**

`EventFilter::Capability` accepts `String` rather than the typed `educore_rbac::Capability` enum. The doc comment at `event_bus.rs:155-165` acknowledges this is "to avoid a circular cross-cutting → cross-cutting dependency", but the trade-off means the bus-port filter is stringly-typed and a typo at the call site (e.g. `EventFilter::Capability("platfrom.user.read".into())`) will silently match no events. The `matches` implementation also uses `envelope.event_type.starts_with(s.as_str())`, which is a substring match: a filter `"fin"` would match `"finance.invoice.generated"`, `"finance.payment.collected"`, `"finance.fees_invoice.configured"`, AND any future event type whose type begins with `"fin"`.

**Expected:**

Either typed `Capability` with a `rbac` dep in `Cargo.toml`, or a domain-prefix exact-match instead of `starts_with`.

**Evidence:**

```rust
  pub enum EventFilter {
      ...
      /// The capability namespace is owned by `educore-rbac::Capability`;
      /// for Phase 2 the filter is a `String` (stringly-typed) to avoid
      /// a circular `cross-cutting → cross-cutting` dependency.
      Capability(String),
      ...
  }
  ...
  Self::Capability(s) => {
      envelope.payload.get("capability").and_then(|v| v.as_str()) == Some(s.as_str())
          || envelope.event_type.starts_with(s.as_str())
  }
  ```

---

### FINDING 15 (id: `CC-EVT-015`)

- **Source:** `docs/audit_reports/findings/wave2-events.md`
- **Severity:** Medium
- **Area:** cross-cutting-events
- **Location:** `crates/cross-cutting/events/src/sync.rs:42-58, 80-115, 140-175, 200-235`

**Description:**

The four typed sync events have `AGGREGATE_TYPE = "sync_session"` but `EVENT_TYPE = "sync.session.*"`. The naming convention in `docs/schemas/event-schema.md:51` (`<domain>.<aggregate>.<verb>`) requires the middle component to match the aggregate name. With `aggregate_type = "sync_session"`, the natural event_type would be `sync.sync_session.started`. The code emits `sync.session.started` (using `session` as the aggregate component) and stores `sync_session` as the aggregate name, so the `aggregate_topic()` helper at `envelope.rs:111-122` produces `sync.sync_session` (domain prefix `sync` + `aggregate_type` `sync_session`), which does not match the `<domain>.<aggregate>` topic convention used by `Topic::Aggregate`. Every consumer subscribing to `Topic::Aggregate("sync", "session")` will receive zero events.

**Expected:**

Either rename `AGGREGATE_TYPE` to `"session"` (matching the dot-separated event_type), or rename `EVENT_TYPE` to `"sync.sync_session.started"` (matching the aggregate_type).

**Evidence:**

```rust
  impl DomainEvent for SyncStarted {
      const EVENT_TYPE: &'static str = "sync.session.started";
      const SCHEMA_VERSION: u32 = 1;
      const AGGREGATE_TYPE: &'static str = "sync_session";
      ...
  }
  ```

---

### FINDING 16 (id: `CC-EVT-016`)

- **Source:** `docs/audit_reports/findings/wave2-events.md`
- **Severity:** Medium
- **Area:** cross-cutting-events
- **Location:** `crates/cross-cutting/events/src/envelope.rs:50` and `crates/infra/storage/src/outbox.rs:54-79, 139-160` and `docs/schemas/event-schema.md:30`

**Description:**

The wire-format field name `schema_version` in `EventEnvelope` and `SerializedEnvelope` conflicts with `docs/schemas/event-schema.md:30`, which defines the canonical name as `event_version`. The bus-port doc at `docs/ports/event-bus.md:34-49` uses `schema_version`, so the two spec docs disagree with each other and the code matches the bus-port spec. Phase 3+ consumers written against the event-schema spec will look for `event_version` in the JSON and find nothing.

**Expected:**

Pick one name and update both spec docs and the code in lockstep. `event_version` (event-schema) or `schema_version` (bus-port).

**Evidence:**

```rust
  // From crates/cross-cutting/events/src/envelope.rs:50:
  pub schema_version: u32,
  // From docs/schemas/event-schema.md:30:
  event_version:     u32,               // schema version of the payload
  ```

---

### FINDING 17 (id: `CC-EVT-017`)

- **Source:** `docs/audit_reports/findings/wave2-events.md`
- **Severity:** Medium
- **Area:** cross-cutting-events
- **Location:** `crates/cross-cutting/events/src/domain_event.rs:170-204`

**Description:**

`RawPayload` (a JSON wrapper carrying `payload`, `correlation_id`, `actor_id`) is declared at `domain_event.rs:170-204` and re-exported nowhere (no `pub use` in `lib.rs:51-67`, not in the prelude). It is documented as "for the audit / outbox writers that need to stamp the `correlation_id` into the JSON body when no typed event is available" but no audit writer or outbox writer calls `RawPayload::new`. The type is dead code.

**Expected:**

Either delete the type or wire it into the audit writer / integration test that needs the audit-sink stamp.

**Evidence:**

```rust
  pub struct RawPayload {
      pub payload: serde_json::Value,
      pub correlation_id: CorrelationId,
      pub actor_id: UserId,
  }
  ```
  `grep "RawPayload::new\|RawPayload {" crates/` returns only the definition site.

---

### FINDING 18 (id: `CC-EVT-018`)

- **Source:** `docs/audit_reports/findings/wave2-events.md`
- **Severity:** Medium
- **Area:** cross-cutting-events
- **Location:** `crates/cross-cutting/events/src/event_bus.rs:108-130, 187-198`

**Description:**

`Topic::EventType(&'static str)`, `Topic::Aggregate(&'static str, &'static str)`, `Topic::Domain(&'static str)`, and `EventFilter::EventType(&'static str)` / `EventFilter::AggregateType(&'static str)` all require `&'static str` arguments. A consumer that discovers an event type at runtime (e.g. by reading `engine.events.list()` per `docs/schemas/event-schema.md:139`) cannot construct a `SubscribeOptions` from the dynamic `String` without `Box::leak` or a similar lifetime hack. The `Serialize` / `Deserialize` derives that would let the type carry a runtime string are also absent.

**Expected:**

Either `String` (with `Serialize`/`Deserialize`) or `Cow<'static, str>` so dynamic discovery works without `Box::leak`.

**Evidence:**

```rust
  pub enum Topic {
      Domain(&'static str),
      Aggregate(&'static str, &'static str),
      EventType(&'static str),
      Tenant(SchoolId),
      All,
  }
  ...
  pub enum EventFilter {
      EventType(&'static str),
      AggregateType(&'static str),
      SchoolId(SchoolId),
      Capability(String),
      Expression(Box<EventFilterExpr>),
  }
  ```

---

### FINDING 19 (id: `CC-EVT-019`)

- **Source:** `docs/audit_reports/findings/wave2-events.md`
- **Severity:** Medium
- **Area:** cross-cutting-events
- **Location:** `crates/cross-cutting/events/src/event_bus.rs:99-118`

**Description:**

`SubscribeOptions::batch_size` is documented to be clamped to a "sane range (e.g. 1..=1024)" by adapters, but neither the trait nor the in-process adapter enforces any range. A caller passing `batch_size = 0` or `batch_size = u32::MAX` will silently get the un-clamped value at the broadcast-channel layer (the in-process bus `clamp_capacity` only clamps the channel capacity, not `batch_size`). The `visibility_timeout` field is also unused at the trait level — no adapter reads it.

**Expected:**

Trait-level validation in `SubscribeOptions::new` or `for_consumer`, plus adapter enforcement per the bus-port doc.

**Evidence:**

```rust
  /// Maximum number of envelopes the subscription may buffer
  /// locally. Adapters clamp this to a sane range (e.g. 1..=1024).
  pub batch_size: u32,
  /// Visibility timeout for in-flight envelopes. After this
  /// duration the bus may redeliver the envelope to another
  /// consumer.
  pub visibility_timeout: Duration,
  ```
  No clamp in `for_consumer` (defaults to 32 / 300s) or in `in_process.rs`.

---

### FINDING 20 (id: `CC-EVT-020`)

- **Source:** `docs/audit_reports/findings/wave2-events.md`
- **Severity:** Medium
- **Area:** cross-cutting-events
- **Location:** `crates/cross-cutting/events/src/event_bus.rs:55-65, 78-101`

**Description:**

`EventBus` has no `unsubscribe` method. The audit checklist asks about a clean unsubscribe; the only path is `Box<dyn EventSubscription>::close(self)`, which consumes the subscription. A consumer holding the subscription by reference (e.g. inside an actor loop or an axum `WebSocket` task) cannot cancel without taking the subscription by value. There is no method like `EventBus::unsubscribe(&self, consumer: &ConsumerId)` that closes the subscription from the bus side, nor any way to enumerate active subscriptions.

**Expected:**

Either an explicit `EventBus::unsubscribe(&self, ConsumerId)` method, or a doc note explaining that subscription lifetime is consumer-owned.

**Evidence:**

```rust
  #[async_trait]
  pub trait EventBus: Send + Sync + fmt::Debug {
      async fn publish(&self, envelope: EventEnvelope) -> Result<PublishReceipt>;
      async fn publish_batch(&self, envelopes: Vec<EventEnvelope>) -> Result<BatchReceipt>;
      async fn subscribe(&self, options: SubscribeOptions) -> Result<Box<dyn EventSubscription>>;
  }
  ```
  No `unsubscribe`.

---

### FINDING 21 (id: `CC-EVT-021`)

- **Source:** `docs/audit_reports/findings/wave2-events.md`
- **Severity:** Medium
- **Area:** cross-cutting-events
- **Location:** `crates/cross-cutting/events/src/event_bus.rs:78-101` and `crates/adapters/event-bus/src/in_process.rs:370-410`

**Description:**

`EventSubscription::next` and `EventSubscription::ack` / `nack` have no documentation of cancellation safety. Per the Rust async ecosystem norm (the tokio docs and `cc-EVT-013` open question in the Phase 2 hand-off), a `Future` is either `CancelSafe` (can be dropped at any await point without side effects) or not (dropping loses state). The in-process adapter's `next()` polls a `broadcast::Receiver::recv()` — if dropped mid-await, the receiver stays subscribed and the next `next()` call continues correctly, so the adapter is cancel-safe by accident. `ack`/`nack` are no-ops, so they're trivially cancel-safe. But the trait docs at `event_bus.rs:78-101` say nothing, so a future NATS/Redis adapter can introduce a non-cancel-safe state machine without violating any documented contract.

**Expected:**

Doc comments on each trait method stating the cancellation-safety guarantee (or explicitly marking it `#[must_use]`).

**Evidence:**

```rust
  /// Returns the next envelope, or `None` if the subscription
  /// is closed. Errors are surfaced as `Some(Err(_))`.
  async fn next(&mut self) -> Option<Result<EventEnvelope>>;

  /// Acknowledges processing of `event_id`. Idempotent.
  async fn ack(&mut self, event_id: EventId) -> Result<AckOutcome>;
  ```
  No cancellation-safety statement on either method.

---

### FINDING 22 (id: `CC-EVT-022`)

- **Source:** `docs/audit_reports/findings/wave2-events.md`
- **Severity:** Medium
- **Area:** cross-cutting-events
- **Location:** `crates/cross-cutting/events/src/event_bus.rs:262-298`

**Description:**

`BatchReceipt` lacks any per-receipt status field. The doc comment at `:271-275` acknowledges this: "the trait doesn't model per-receipt failure for batches; producers that need that granularity call `publish` in a loop". But the in-process adapter at `crates/adapters/event-bus/src/in_process.rs:202-219` short-circuits `publish_batch` on the first failure inside the loop and returns `Ok(BatchReceipt { receipts: [...up to the failure], correlation_id: None })` — so a partial batch is indistinguishable from a successful batch except by counting receipts. There is no `BatchFailure` variant or per-receipt `Ok`/`Err` enum to carry the failure.

**Expected:**

Either add `BatchItemStatus` to `PublishReceipt` (mirroring the doc's own suggestion), or change `BatchReceipt::receipts` to `Vec<Result<PublishReceipt, EventError>>`.

**Evidence:**

```rust
  pub struct BatchReceipt {
      /// Per-envelope receipts, in the order the envelopes were
      /// submitted.
      pub receipts: Vec<PublishReceipt>,
      /// The correlation id of the batch, if any. ...
      pub correlation_id: Option<CorrelationId>,
  }
  ```

---

### FINDING 23 (id: `CC-EVT-023`)

- **Source:** `docs/audit_reports/findings/wave2-events.md`
- **Severity:** Medium
- **Area:** cross-cutting-events
- **Location:** `crates/cross-cutting/events/src/event_bus.rs:255-260` and `docs/ports/event-bus.md:182-187`

**Description:**

`ConsumerId` is the only stable identifier for a subscription; it is used for offset tracking and observability per the doc. However, the bus-port worked example at `docs/ports/event-bus.md:194` uses `ConsumerId::new("welcome-emailer")` as the consumer id for a single in-process consumer, and the doc-comment on `ConsumerId::new` (`:255-258`) says the string "is expected to be stable across process restarts". The trait provides no method to look up a subscription by `ConsumerId`, no method to enumerate active `ConsumerId`s, no method to read a consumer's offset / lag — and `EventBus::subscribe` does not enforce that the `ConsumerId` is unique across concurrent subscriptions. Two concurrent `subscribe` calls with the same `ConsumerId` will silently create two parallel subscriptions.

**Expected:**

Bus-port contract section "Subscription Model" (`docs/ports/event-bus.md:184-195`); offset tracking is the consumer's responsibility only because the port is silent on it.

**Evidence:**

```rust
  #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
  #[serde(transparent)]
  pub struct ConsumerId(pub String);
  ```

---

### FINDING 24 (id: `CC-EVT-024`)

- **Source:** `docs/audit_reports/findings/wave2-events.md`
- **Severity:** Medium
- **Area:** cross-cutting-events
- **Location:** `crates/cross-cutting/events/src/event_bus.rs:34-46` and `crates/cross-cutting/events/src/event_bus.rs:138-157`

**Description:**

`EventFilter::Expression(Box<EventFilterExpr>)` is the only way to compose filters; there is no flat `Filter::All(Vec<EventFilter>)` or `Filter::Any(Vec<EventFilter>)` shape. The `EventFilterExpr` enum has 4 variants (`And`, `Or`, `Not`, `Leaf`), all binary except `Leaf`. Constructing an N-way OR requires a right-leaning tree: `Or(Leaf(A), Or(Leaf(B), Leaf(C)))`, which is awkward and forces `Box` allocations for every internal node. The audit checklist asks about subscription filter expressiveness; the current shape is minimal.

**Expected:**

`EventFilter::All(Vec<EventFilter>)` / `Any(Vec<EventFilter>)` variants for N-ary composition, OR an explicit note that the tree shape is intentional.

**Evidence:**

```rust
  pub enum EventFilterExpr {
      And(Box<Self>, Box<Self>),
      Or(Box<Self>, Box<Self>),
      Not(Box<Self>),
      Leaf(Box<EventFilter>),
  }
  ```

---

### FINDING 25 (id: `CC-EVT-025`)

- **Source:** `docs/audit_reports/findings/wave2-events.md`
- **Severity:** Low
- **Area:** cross-cutting-events
- **Location:** `crates/cross-cutting/events/src/domain_event.rs:144-167`

**Description:**

`EmittedEvent<T>` is a small wrapper that pairs `T: DomainEvent` with the `TenantContext` it was emitted under. It is documented as the recommended construction pattern (`EmittedEvent::new(event, ctx).into_envelope()`), is re-exported in the prelude, and is unit-tested at `domain_event.rs:267-285`. No domain crate uses it: `crates/domains/*/src/services.rs` constructs `into_envelope` directly from a typed event + `TenantContext` (e.g. `crates/domains/finance/src/services.rs`). The wrapper is dead in production.

**Expected:**

Either remove `EmittedEvent` from the prelude, or document the Phase 3+ convention that domain services return `EmittedEvent<T>` instead of `(T, EventEnvelope)`.

**Evidence:**

```rust
  pub struct EmittedEvent<T: DomainEvent + Serialize> {
      pub event: T,
      pub ctx: TenantContext,
  }
  ```
  `grep "EmittedEvent::new" crates/` returns only the test in `domain_event.rs`.

---

### FINDING 26 (id: `CC-EVT-026`)

- **Source:** `docs/audit_reports/findings/wave2-events.md`
- **Severity:** Low
- **Area:** cross-cutting-events
- **Location:** `crates/cross-cutting/events/src/event_bus.rs:75-87`

**Description:**

`EventSubscription::next()` returns `Option<Result<EventEnvelope>>` — a `None` means "subscription closed". A consumer cannot distinguish a slow broker (no message yet) from a permanently closed subscription without timing out. The bus-port spec at `docs/ports/event-bus.md:108` says the same shape, so the implementation matches the spec, but the shape conflates "idle" with "closed" which is a known footgun. The audit checklist calls this out as an edge case.

**Expected:**

Either an explicit `RecvOutcome { Idle, Envelope, Closed }` enum, or a separate `EventSubscription::is_closed` method.

**Evidence:**

```rust
  /// Returns the next envelope, or `None` if the subscription
  /// is closed. Errors are surfaced as `Some(Err(_))`.
  async fn next(&mut self) -> Option<Result<EventEnvelope>>;
  ```

---

### FINDING 27 (id: `CC-EVT-027`)

- **Source:** `docs/audit_reports/findings/wave2-events.md`
- **Severity:** Low
- **Area:** cross-cutting-events
- **Location:** `crates/cross-cutting/events/src/envelope.rs:43-46` and `crates/cross-cutting/events/src/outbox.rs:6-17`

**Description:**

The `EventEnvelope` carries `&'static str` for `event_type` and `aggregate_type`, which means it does NOT implement `DeserializeOwned`. The envelope.rs:138-142 test comment acknowledges this ("`EventEnvelope` has `&'static str` fields (per the bus-port contract), so it does NOT implement `DeserializeOwned`"). The round-trip bridge to `SerializedEnvelope` (which uses `String` for the same fields) lives in `crates/infra/storage/src/outbox.rs:139-160` and `crates/infra/storage/src/event_log.rs:175-187` — i.e. the envelope cannot round-trip through the bus-port; only the storage-port mirror can. Any consumer code (audit writer, integration test, central-fan-out) that wants to re-materialise the typed envelope from a stored row must depend on the storage port rather than the events port.

**Expected:**

Either move the bridge to `educore-events` (reverse the tier dep that PHASE-2-HANDOFF § "Open questions" #6 flags), or document the storage-port dependency at the bus-port consumer's API surface.

**Evidence:**

```rust
  // crates/cross-cutting/events/src/envelope.rs:43-46:
  // **Stability:** the field set, names, and order are part of the
  // engine's public API. Renames or removals are breaking changes
  // and require an ADR.
  pub struct EventEnvelope {
      ...
      pub event_type: &'static str,
      ...
      pub aggregate_type: &'static str,
      ...
  }
  ```
  And the bridge lives at `crates/infra/storage/src/outbox.rs:139-160`, in `educore-storage` (infra), not in `educore-events` (cross-cutting).

---

### FINDING 28 (id: `CC-EVT-028`)

- **Source:** `docs/audit_reports/findings/wave2-events.md`
- **Severity:** Low
- **Area:** cross-cutting-events
- **Location:** `crates/cross-cutting/events/src/event_bus.rs:124-135`

**Description:**

`StartPosition::FromTimestamp` and `StartPosition::FromEventId` rely on UUIDv7 time ordering for cursor semantics. The doc on `FromEventId` says "UUIDv7 is time-ordered: lexicographic comparison gives chronological ordering" (`crates/adapters/event-bus/src/in_process.rs:329-333`). This is correct for UUIDv7 minted by the same generator, but the bus-port spec at `docs/ports/event-bus.md:55-60` does not require that all `event_id`s on the bus are UUIDv7 — any consumer that hand-mints a UUIDv4 (or a UUIDv7 with a non-monotonic clock skew, or a foreign system's UUID) will be ordered incorrectly. The cursor code at `in_process.rs:325-345` performs a raw lex compare without verifying that the cursor itself is UUIDv7.

**Expected:**

Bus-port spec section "Schema Versioning" + "Replay" (`docs/ports/event-bus.md:71-83`) and event-schema § 1.1 require UUIDv7 but don't assert it on the wire.

**Evidence:**

```rust
  StartPosition::FromEventId(id) => {
      env.event_id.as_uuid() > id.as_uuid()
  }
  ```
  No version-bit check; a UUIDv4 cursor would sort lexicographically by its random bits, not by time.

---


## Audit (cross-cutting) (target id prefix: `CC-AUD`)

**Path:** `crates/cross-cutting/audit/`  
**Total findings:** 30 (8 critical, 12 high, 8 medium, 2 low)


### FINDING 1 (id: `CC-AUD-001`)

- **Source:** `docs/audit_reports/findings/wave2-audit.md`
- **Severity:** Critical
- **Area:** cross-cutting-audit
- **Location:** `crates/cross-cutting/audit/src/writer.rs:824-869` (AuditWriter::write) and `crates/domains/cms/src/services.rs:143-163` (create_page_service) plus `crates/domains/documents/src/services.rs:138-167`

**Description:**

Every state-changing service that wires `AuditWriter` calls `audit.write(...)` AFTER the repository mutation has already been awaited (`repo.insert(...).await?` then `audit.write(...).await?`). There is no transaction object passed into the service and no `tx.audit_log().append(...)` shape; the audit row is therefore committed in an independent transaction from the aggregate row. Per the audit-first invariant ("every state change produces exactly one audit row inside the same transaction as the state change itself" — `crates/cross-cutting/audit/src/lib.rs:7-9`), the audit log can be missing for a committed aggregate row, or present for an aggregate row that was rolled back by a subsequent step.

**Expected:**

Per `docs/schemas/audit-schema.md` § 2 and the engine rule "Audit-first" (`AGENTS.md` "Engine Rules" #8): audit and aggregate writes share one transaction. The `AuditLog` sub-port must be addressable via `Transaction::audit_log()` (analogous to `Transaction::outbox()`) and the service factory must call them in the same `tx`.

**Evidence:**

```rust
  // crates/domains/cms/src/services.rs:143-163
  repo.insert(&page)
      .await
      .map_err(|e| CmsError::Infrastructure(e.to_string()))?;
  let after = snapshot(&page);
  audit
      .write(&tenant, AuditAction::Create, AuditTarget::Page(...), None, Some(after))
      .await
      .map_err(|e| CmsError::Infrastructure(e.to_string()))?;
  // Two independent awaits = two independent transactions
  ```

---

### FINDING 2 (id: `CC-AUD-002`)

- **Source:** `docs/audit_reports/findings/wave2-audit.md`
- **Severity:** Critical
- **Area:** cross-cutting-audit
- **Location:** `crates/cross-cutting/audit/src/writer.rs` (entire file — 1474 lines, zero hits for `hash`, `chain`, `signature`, `mac`, `hmac`, `verify`, `prev_audit_id`) and `migrations/engine/0000_engine_core.{postgres,mysql,sqlite}.sql` audit_log DDL

**Description:**

Audit rows have no signature, MAC, or hash chain. The `audit_log` table exposes `audit_id` (UUIDv7) but no `prev_audit_id`, `signature`, or `mac` column. A DBA or storage-adapter-level attacker can rewrite `before`/`after` snapshots in historical rows; the engine's only integrity defense is INSERT-only database privileges, which the engine does not configure. Per `docs/schemas/audit-schema.md` § 3, the audit log must be tamper-evident; the spec names WORM replication as the default defense but the engine ships no WORM configuration either.

**Expected:**

Per `docs/schemas/audit-schema.md` § 3: tamper-evident storage — either a hash chain (`prev_audit_hash` + `audit_hash` columns), per-row signatures, or mandatory WORM replication. At minimum a `prev_audit_hash` + `audit_hash` pair so the engine can verify the chain end-to-end.

**Evidence:**

`grep -n 'hash\|chain\|signature\|mac\|hmac\|verify' crates/cross-cutting/audit/src/*.rs` returns zero hits across the four source files. `migrations/engine/0000_engine_core.postgres.sql:104-126` has 20 columns; none are tamper-evidence columns. The writer at `crates/cross-cutting/audit/src/writer.rs:824-869` constructs an `AuditLogEntry` with no hash payload.

---

### FINDING 3 (id: `CC-AUD-003`)

- **Source:** `docs/audit_reports/findings/wave2-audit.md`
- **Severity:** Critical
- **Area:** cross-cutting-audit
- **Location:** `crates/infra/storage/src/audit.rs:55-101` (AuditLogEntry struct) vs `migrations/engine/0000_engine_core.postgres.sql:104-126` (audit_log DDL) vs `docs/schemas/audit-schema.md:51-77` (AuditRecord struct)

**Description:**

The `AuditLogEntry` struct carries only 14 fields, but the canonical `audit_log` DDL has 20 columns and `docs/schemas/audit-schema.md` § 2 mandates 18 named fields. The struct is missing `audit_id` (regenerated by each adapter on append — no idempotency), `actor_type`, `command_id`, `recorded_at`, `ip`, `user_agent`, `session_id`, `cross_tenant`, and `source`. The writer at `crates/cross-cutting/audit/src/writer.rs:833-849` hardcodes `metadata: serde_json::Value::Null`, `active_status: ActiveStatus::Active`, and `event_id: None`. The struct is therefore a subset of both the spec and the DDL.

**Expected:**

Per `docs/schemas/audit-schema.md` § 2 the canonical `AuditRecord` carries all 18 named fields; the port struct must carry the same shape so the adapter can pass through everything the DDL stores.

**Evidence:**

```rust
  // crates/infra/storage/src/audit.rs:55-101
  pub struct AuditLogEntry {
      pub school_id: SchoolId,
      pub actor_id: UserId,
      pub action: String,
      pub target_type: String,
      pub target_id: Uuid,
      pub before: Option<bytes::Bytes>,
      pub after: Option<bytes::Bytes>,
      pub event_id: Option<EventId>,
      pub correlation_id: CorrelationId,
      pub occurred_at: Timestamp,
      pub active_status: ActiveStatus,
      pub metadata: serde_json::Value,
      // MISSING: audit_id, actor_type, command_id, recorded_at,
      //          ip, user_agent, session_id, cross_tenant, source
  }
  ```

---

### FINDING 4 (id: `CC-AUD-004`)

- **Source:** `docs/audit_reports/findings/wave2-audit.md`
- **Severity:** Critical
- **Area:** cross-cutting-audit
- **Location:** `crates/infra/storage/src/audit.rs:117-156` (AuditLog trait) vs `docs/schemas/audit-schema.md:121-168` (AuditQuery port trait spec)

**Description:**

`AuditLog` trait exposes only `append` and `read_for_target`. Per `docs/schemas/audit-schema.md` § 5 the engine defines an `AuditQuery` port with five methods: `list(tenant, filter, page)`, `get(tenant, audit_id)`, `resource_history(tenant, type, id, page)`, `actor_history(tenant, actor_id, page)`, plus `AuditFilter` (ByAction, ByResource, ByActor, ByCorrelation, ByTimeRange, ByEventType, Custom). None of these exist. A consumer cannot answer "what happened to this student?" (spec § 6) or "every payroll payment in Q1" (spec § 7) without scanning the table client-side.

**Expected:**

Per `docs/schemas/audit-schema.md` § 5-7: an `AuditQuery` port trait with `list`, `get`, `resource_history`, `actor_history`, and a `Page`-typed paginator. All queries must be tenant-scoped via `TenantContext`.

**Evidence:**

```rust
  // crates/infra/storage/src/audit.rs:117-156
  #[async_trait]
  pub trait AuditLog: Send + Sync {
      async fn append(&self, entry: AuditLogEntry) -> Result<()>;
      async fn read_for_target(
          &self, school_id: SchoolId, target_id: Uuid, limit: u32,
      ) -> Result<Vec<AuditLogEntry>>;
      // MISSING: list, get, resource_history, actor_history, AuditFilter, Page
  }
  ```

---

### FINDING 5 (id: `CC-AUD-005`)

- **Source:** `docs/audit_reports/findings/wave2-audit.md`
- **Severity:** Critical
- **Area:** cross-cutting-audit
- **Location:** `migrations/engine/0000_engine_core.postgres.sql:104-126` (and equivalent MySQL/SQLite audit_log DDLs)

**Description:**

The PostgreSQL audit_log DDL has zero `ROW LEVEL SECURITY` clauses, zero `CREATE POLICY` statements, zero `ENABLE ROW LEVEL SECURITY` / `FORCE ROW LEVEL SECURITY` clauses. `docs/schemas/sql-dialects/postgresql.md` (referenced from `docs/schemas/audit-schema.md` § 10 / § 13) requires PG to use `CREATE POLICY` + `ENABLE ROW LEVEL SECURITY` and the adapter to issue `SET LOCAL app.current_school_id = ?` on every transaction. The DDL has none of this — the only `school_id` filter is the WHERE clause in the SELECT path, which a malicious SQL session can defeat. RLS bypass via superuser is acknowledged in `docs/build-plan.md` Phase 2 Risks, but the engine emits no policies at all.

**Expected:**

Per `docs/schemas/tenancy-schema.md` and the AGENTS.md Engine Rule #7 ("Multi-tenant by default"): PG `audit_log` table must `ENABLE ROW LEVEL SECURITY` and `FORCE ROW LEVEL SECURITY` with policies keyed on `current_setting('app.current_school_id')`.

**Evidence:**

```sql
  -- migrations/engine/0000_engine_core.postgres.sql:104-126
  CREATE TABLE IF NOT EXISTS engine.audit_log (
      audit_id        UUID         NOT NULL,
      ...
      PRIMARY KEY (audit_id)
  );
  -- No ALTER TABLE ... ENABLE ROW LEVEL SECURITY
  -- No CREATE POLICY ...
  ```

---

### FINDING 6 (id: `CC-AUD-006`)

- **Source:** `docs/audit_reports/findings/wave2-audit.md`
- **Severity:** Critical
- **Area:** cross-cutting-audit
- **Location:** `crates/infra/storage/src/audit.rs:117-156` (AuditLog trait) and `crates/cross-cutting/audit/src/writer.rs` (entire writer module — no purge method)

**Description:**

There is no `purge_expired(before)` method on the `AuditLog` trait and no engine-side sweep implementation. The only retention path is `AuditWriter::maybe_sweep` (`writer.rs:871-902`) which emits a `RetentionSweepDue` event and delegates the actual `DELETE FROM audit_log ...` to a consumer subscriber. The consumer is responsible for *both* subscribing to the event and *writing the SQL* — neither operation is provided by the engine. A consumer that omits the subscription grows the audit table without bound (10M rows/day per `docs/build-plan.md` Phase 2 Risks); a consumer that wires it but uses the wrong predicate deletes the wrong rows.

**Expected:**

Per `docs/schemas/audit-schema.md` § 3 and § 9: the engine provides a typed retention-sweep port that the consumer adapter implements; the engine's `AuditLog` trait exposes `purge_expired(before) -> Result<u64>` so the implementation is uniform across dialects.

**Evidence:**

```rust
  // crates/infra/storage/src/audit.rs:117-156 — no purge/cleanup method exists.
  // crates/cross-cutting/audit/src/writer.rs:871-902 — only emits an event, no SQL.
  pub async fn maybe_sweep(&self, school_id: SchoolId) -> Result<()> {
      ...
      self.emit_sweep_due(school_id, cutoff, now).await?;
      Ok(())
  }
  ```

---

### FINDING 7 (id: `CC-AUD-007`)

- **Source:** `docs/audit_reports/findings/wave2-audit.md`
- **Severity:** Critical
- **Area:** cross-cutting-audit
- **Location:** `grep -rln AuditWriter crates/domains/` — output limited to `crates/domains/cms/src/services.rs` and `crates/domains/documents/src/services.rs`; 8 of 10 domain crates do not import `AuditWriter` at all

**Description:**

Only the CMS (Phase 12) and Documents (Phase 11) domain crates wire `AuditWriter`. The other 8 domain crates (`educore-academic`, `educore-assessment`, `educore-attendance`, `educore-hr`, `educore-finance`, `educore-facilities`, `educore-library`, `educore-communication`) plus `educore-events-domain` (Phase 13 calendar) have no audit emission in their service factories. Per the no-gaps gate (`docs/build-plan.md` "no-gaps gates") and the engine rule "Audit-first", every state-changing command must write an audit row; the gate is met for 2 of 10 domains.

**Expected:**

Every domain crate that defines state-changing commands imports `educore_audit::writer::AuditWriter` and calls `audit.write(...)` inside each `*_service` factory function, scoped to the correct `AuditTarget` variant.

**Evidence:**

`grep -rln 'AuditWriter' crates/domains/` returns exactly two paths: `crates/domains/cms/src/services.rs` and `crates/domains/documents/src/services.rs`. `crates/domains/academic/src/services.rs`, `crates/domains/finance/src/services.rs`, etc. contain zero matches.

---

### FINDING 8 (id: `CC-AUD-008`)

- **Source:** `docs/audit_reports/findings/wave2-audit.md`
- **Severity:** Critical
- **Area:** cross-cutting-audit
- **Location:** `crates/cross-cutting/audit/src/writer.rs:824-834` (AuditWriter::write signature)

**Description:**

`AuditWriter::write(&self, ctx, action, target, before, after)` does not accept `ip`, `user_agent`, `session_id`, `command_id`, `cross_tenant`, `actor_type`, or `source` as parameters. The signature forces every caller to lose these fields. `docs/schemas/audit-schema.md` § 4 mandates that authentication events, authorisation denials, capability changes, cross-tenant ops, and security-relevant settings changes all write to the audit log; without `ip`/`user_agent`/`session_id`, a regulator cannot answer "who attempted this login from where?". The `TenantContext` type does not carry these fields either — the writer drops them at the boundary.

**Expected:**

Either extend `TenantContext` with `Option<IpAddr>`, `Option<String>` (user agent), `Option<SessionId>`, `Option<CommandId>`, `bool` (cross_tenant), `ActorType`, `AuditSource`, or add a parallel `AuditContext` struct threaded through `write`. The current 5-arg signature is below the spec's 18-field minimum.

**Evidence:**

```rust
  // crates/cross-cutting/audit/src/writer.rs:824-834
  pub async fn write(
      &self,
      ctx: &TenantContext,
      action: AuditAction,
      target: AuditTarget,
      before: Option<bytes::Bytes>,
      after: Option<bytes::Bytes>,
  ) -> Result<()>
  // No ip, user_agent, session_id, command_id, cross_tenant, actor_type, source
  ```

---

### FINDING 10 (id: `CC-AUD-010`)

- **Source:** `docs/audit_reports/findings/wave2-audit.md`
- **Severity:** High
- **Area:** cross-cutting-audit
- **Location:** `crates/cross-cutting/audit/src/writer.rs:38-44` (SENTINEL_TARGET_ID) and `crates/cross-cutting/audit/src/writer.rs:884` (read_for_target call)

**Description:**

`SENTINEL_TARGET_ID = Uuid::nil()` is used by `maybe_sweep` to look up "the oldest audit row for the school". The DDL `migrations/engine/0000_engine_core.sqlite.sql:107-125` has no CHECK constraint preventing `audit_log.target_id = Uuid::nil()` (SQLite CHECK on `audit_id` only — `length(audit_id) = 36` — not `target_id`). The PG/MySQL DDLs have no CHECK either. An attacker who can write to the audit table (or a bug that stores nil for a system actor's action) can inject a row with `target_id = Uuid::nil()` and an arbitrary `occurred_at` to either force spurious sweep emissions or defeat the oldest-row discovery.

**Expected:**

A typed discovery port: a dedicated `oldest_row_for_school(school_id) -> Option<Timestamp>` method on the `AuditLog` trait, not a sentinel overload of `read_for_target`. Plus a `CHECK (target_id <> '00000000-0000-0000-0000-000000000000')` on the DDL.

**Evidence:**

```rust
  // crates/cross-cutting/audit/src/writer.rs:38-44
  pub const SENTINEL_TARGET_ID: Uuid = Uuid::nil();
  // crates/cross-cutting/audit/src/writer.rs:884
  let rows = self.audit_log.read_for_target(school_id, SENTINEL_TARGET_ID, 1).await?;
  ```

---

### FINDING 11 (id: `CC-AUD-011`)

- **Source:** `docs/audit_reports/findings/wave2-audit.md`
- **Severity:** High
- **Area:** cross-cutting-audit
- **Location:** `crates/cross-cutting/audit/src/writer.rs:849` (`event_id: None`) and `crates/cross-cutting/audit/src/writer.rs:843-846` (comment block)

**Description:**

`AuditWriter::write` always sets `event_id: None` with a Phase-3-deferred comment: "Phase 2: the audit row is decoupled from the event-log row. Phase 3 will wire `event_id` when command handlers run inside the same transaction as the outbox emit." Per `docs/schemas/audit-schema.md` § 4 item 1: "Every state-changing command that successfully completes (after persistence, before response)" produces an audit row. Per spec § 5 `AuditFilter::ByEventType`, an auditor queries "every audit row that mirrors event X". With `event_id` always `None`, the correlation between audit rows and event-log rows is lost; `command_id` is `NULL` on the wire because the port struct does not carry it.

**Expected:**

`AuditWriter::write` accepts an `EventId` parameter (or pulls it from the same transaction context as the outbox envelope) so the audit row and the event-log row share the event id; the port struct's `event_id` field is populated.

**Evidence:**

```rust
  // crates/cross-cutting/audit/src/writer.rs:843-846
  // Phase 2: the audit row is decoupled from the
  // event-log row. Phase 3 will wire `event_id` when
  // command handlers run inside the same transaction
  // as the outbox emit.
  event_id: None,
  ```

---

### FINDING 12 (id: `CC-AUD-012`)

- **Source:** `docs/audit_reports/findings/wave2-audit.md`
- **Severity:** High
- **Area:** cross-cutting-audit
- **Location:** `crates/adapters/storage-postgres/src/audit_log.rs:90-111` (into_entry) and `crates/adapters/storage-mysql/src/audit_log.rs:85-106` (into_entry) and `crates/adapters/storage-sqlite/src/audit_log.rs:78-99` (to_entry)

**Description:**

All three SQL adapters read `active_status` from the row but the DDL does not have an `active_status` column; the adapters hardcode `active_status: ActiveStatus::Active` on every read. The PG/MySQL/SQLite adapter docstrings admit this: "The DDL does not carry an `active_status` column. The audit log is append-only; we set `Active` on read". But the spec (audit-schema.md § 10: "the engine does not allow editing or deletion of audit records") makes the column meaningless — if it can never be `Retired`, no storage column is needed; if the engine intends a future retire path, the column must exist. The current state is a phantom field.

**Expected:**

Either (a) remove `active_status` from `AuditLogEntry` (the column is not in the DDL and never will be), or (b) add `active_status` to the DDL and wire a `retire_audit` port method (the latter contradicts append-only).

**Evidence:**

```rust
  // crates/adapters/storage-postgres/src/audit_log.rs:103-110
  // The DDL does not carry an `active_status`
  // column. The audit log is append-only; we set
  // `Active` on read and rely on the engine's
  // `INCLUDE_RETIRED` query to surface retired
  // rows in the future.
  active_status: ActiveStatus::Active,
  ```

---

### FINDING 13 (id: `CC-AUD-013`)

- **Source:** `docs/audit_reports/findings/wave2-audit.md`
- **Severity:** High
- **Area:** cross-cutting-audit
- **Location:** `crates/adapters/storage-sqlite/src/audit_log.rs:140` (`actor_type` bind `"user"`) and `crates/adapters/storage-sqlite/src/audit_log.rs:158` (`source` bind `"system"`) and module doc table at lines 17-32

**Description:**

The SQLite adapter hardcodes `actor_type = "user"` and `source = "system"` for every appended row, regardless of the entry's actual `actor_id`. The module's own doc table at lines 17-32 admits this: "`actor_type` = `"user"` (literal); `source` = `"system"` (literal)". The PG and MySQL adapters detect `SYSTEM_USER_ID` correctly (`crates/adapters/storage-postgres/src/audit_log.rs:148-152`), but SQLite does not — a system-initiated audit row written through SQLite will be tagged `actor_type = "user"` and `source = "system"`. The values are mutually inverted from PG/MySQL (PG/MySQL default `source = "api"`, SQLite defaults to `"system"`), and the `actor_type` inversion means a SQLite-backed deployment cannot distinguish user-initiated from system-initiated audits.

**Expected:**

Per the spec § 4 item 2 ("Every state-changing command that fails — the audit record includes the error kind and message") and item 3 (authentication events) and the PG/MySQL adapter pattern: SQLite adapter must inspect `entry.actor_id == SYSTEM_USER_ID` and bind the same `actor_type` discriminator PG/MySQL use. `source` must be a constructor parameter or default to `"api"` to match PG/MySQL.

**Evidence:**

```rust
  // crates/adapters/storage-sqlite/src/audit_log.rs:140
  .bind("user")  // actor_type hardcoded
  // crates/adapters/storage-sqlite/src/audit_log.rs:158
  .bind("system")  // source hardcoded — wrong default
  ```

---

### FINDING 14 (id: `CC-AUD-014`)

- **Source:** `docs/audit_reports/findings/wave2-audit.md`
- **Severity:** High
- **Area:** cross-cutting-audit
- **Location:** `crates/cross-cutting/audit/src/writer.rs:823-869` (AuditWriter::write) and `crates/cross-cutting/audit/src/writer.rs:871-902` (maybe_sweep)

**Description:**

`AuditWriter` is constructed with `Arc<dyn AuditLog>` and calls `audit_log.append(...)` and `audit_log.read_for_target(...)` directly on the handle. There is no `Transaction` parameter, no `&dyn StorageTransaction` accessor (contrast `crates/infra/storage/src/transaction.rs` which exposes `outbox()`, `audit_log()`, `event_log()`, `idempotency()` per the storage-port design). The writer is therefore forced to operate outside any transaction — confirming Finding 1's atomicity gap from the writer's perspective.

**Expected:**

Either the writer takes a `&dyn StorageTransaction` and uses `tx.audit_log()`, or the writer is deprecated in favour of per-domain code calling `tx.audit_log().append(entry)` directly inside the same transaction that mutates the aggregate.

**Evidence:**

```rust
  // crates/cross-cutting/audit/src/writer.rs:770-783 (AuditWriter struct)
  pub struct AuditWriter {
      audit_log: std::sync::Arc<dyn AuditLog>,
      bus: std::sync::Arc<dyn EventBus>,
      clock: std::sync::Arc<dyn Clock>,
      policy: RetentionPolicy,
      last_sweep_at: Mutex<Option<Timestamp>>,
  }
  // No transaction parameter; append goes straight to the global Arc<dyn AuditLog>
  ```

---

### FINDING 15 (id: `CC-AUD-015`)

- **Source:** `docs/audit_reports/findings/wave2-audit.md`
- **Severity:** High
- **Area:** cross-cutting-audit
- **Location:** `crates/cross-cutting/audit/src/writer.rs:924-951` (emit_sweep_due)

**Description:**

`emit_sweep_due` mints a fresh `Uuid::now_v7()` for the sweep event's correlation id and publishes via `TenantContext::system(school_id, system_corr)`. The original audit row that triggered the sweep is not carried in the event payload; the subscriber cannot reconstruct "which audit row's age triggered this sweep". Per `docs/schemas/audit-schema.md` § 15 "Audit-Driven Subscriptions", subscribers need enough context to act — the current event carries `school_id`, `cutoff`, `at` but not the `audit_id` of the oldest row, its `target_type`, or the `actor_id` that wrote it.

**Expected:**

The `RetentionSweepDue` payload includes `oldest_audit_id: Uuid`, `oldest_target_type: String`, and the per-school `retention_category` the sweep applies to (per spec § 9 the policy differs per record type).

**Evidence:**

```rust
  // crates/cross-cutting/audit/src/writer.rs:941-948
  async fn emit_sweep_due(&self, school_id: SchoolId, cutoff: Timestamp, at: Timestamp) -> Result<()> {
      let event = RetentionSweepDue::new(school_id, cutoff, at);
      let system_corr = educore_core::ids::CorrelationId(Uuid::now_v7());
      let ctx = TenantContext::system(school_id, system_corr);
      // payload carries only school_id, cutoff, at
  ```

---

### FINDING 16 (id: `CC-AUD-016`)

- **Source:** `docs/audit_reports/findings/wave2-audit.md`
- **Severity:** High
- **Area:** cross-cutting-audit
- **Location:** `crates/cross-cutting/audit/src/retention.rs:29-58` (RetentionPolicy) and `crates/cross-cutting/audit/src/writer.rs:770-783` (AuditWriter.policy)

**Description:**

`RetentionPolicy` is a single struct with one `retention_days` field. There is no per-category retention (authentication, authorization denials, capability changes, finance, payroll, academic, library/facilities, settings, backup, agent — the ten categories in spec § 9). The `AuditWriter` field is also a single `policy: RetentionPolicy`. The engine therefore cannot enforce the spec's "7 years for finance / payroll / academic" while keeping "18 months for authentication" within the same deployment; a consumer must choose one retention for all categories and either over-retain authentication events (storage cost) or under-retain finance events (compliance gap).

**Expected:**

Per `docs/schemas/audit-schema.md` § 9: a `RetentionPolicy` enum or struct with one field per category. The `AuditWriter::write` API takes the `target_type` and looks up the matching retention bucket; the sweep threshold is computed per category.

**Evidence:**

```rust
  // crates/cross-cutting/audit/src/retention.rs:29-44
  pub struct RetentionPolicy {
      pub retention_days: u32,
      pub sweep_check_interval: Duration,
  }
  // Only one retention_days; no per-category model
  ```

---

### FINDING 17 (id: `CC-AUD-017`)

- **Source:** `docs/audit_reports/findings/wave2-audit.md`
- **Severity:** High
- **Area:** cross-cutting-audit
- **Location:** `crates/adapters/storage-postgres/src/audit_log.rs:53-78` (AuditLogRow struct) and `crates/adapters/storage-mysql/src/audit_log.rs:47-72` and `crates/adapters/storage-sqlite/src/audit_log.rs:55-76`

**Description:**

All three SQL adapters' `AuditLogRow` derive `sqlx::FromRow` but the PG adapter has eight `#[allow(dead_code)]` annotations (lines 54, 57, 64, 68, 70, 72, 74, 78) marking `audit_id`, `actor_type`, `command_id`, `recorded_at`, `ip`, `user_agent`, `session_id`, `cross_tenant`, `source` as never-read. The MySQL adapter has the same eight `#[allow(dead_code)]` annotations. The SQLite adapter has `#![allow(dead_code)]` at the struct level (line 56). The data is fetched from the database and dropped — a regulator querying "show me every audit row where `cross_tenant = TRUE`" gets rows back from PG, but the adapter has discarded those columns on the read path.

**Expected:**

Either drop the columns from the SELECT (and from the DDL) or surface them through the port struct so consumers can filter on them.

**Evidence:**

```rust
  // crates/adapters/storage-postgres/src/audit_log.rs:53-78 (excerpt)
  struct AuditLogRow {
      #[allow(dead_code)] audit_id: Uuid,
      ...
      #[allow(dead_code)] actor_type: String,
      ...
      #[allow(dead_code)] command_id: Option<Uuid>,
      ...
      #[allow(dead_code)] ip: Option<String>,
      ...
      #[allow(dead_code)] cross_tenant: bool,
      #[allow(dead_code)] source: String,
  }
  ```

---

### FINDING 18 (id: `CC-AUD-018`)

- **Source:** `docs/audit_reports/findings/wave2-audit.md`
- **Severity:** High
- **Area:** cross-cutting-audit
- **Location:** `migrations/engine/0000_engine_core.{postgres,mysql,sqlite}.sql` audit_log DDLs (lines 104-126 PG, 93-122 MySQL, 107-130 SQLite)

**Description:**

The three dialect DDLs are consistent on the 20 columns but disagree on the partitioning story. The PG DDL (`0000_engine_core.postgres.sql:104-126`) does NOT include `PARTITION BY RANGE (school_id, date_trunc('month', occurred_at))` that `docs/schemas/audit-schema.md` § 13.1 mandates as the canonical PG layout. The MySQL DDL does NOT include `PARTITION BY KEY (school_id) PARTITIONS 12` that § 13.2 mandates. The SQLite DDL is a plain table (consistent with § 13.3's "SQLite has no native partitioning"). The audit-schema spec and the canonical DDL are out of sync; a consumer following the DDL will get unpartitioned tables and the partitioning the spec promises is missing in production.

**Expected:**

PG DDL must `PARTITION BY RANGE (school_id, date_trunc('month', occurred_at))` with at least a default `audit_log_default` partition. MySQL DDL must `PARTITION BY KEY (school_id) PARTITIONS 12`. The current DDLs implement none of this.

**Evidence:**

`grep -n 'PARTITION' migrations/engine/0000_engine_core.postgres.sql migrations/engine/0000_engine_core.mysql.sql` returns zero matches.

---

### FINDING 19 (id: `CC-AUD-019`)

- **Source:** `docs/audit_reports/findings/wave2-audit.md`
- **Severity:** High
- **Area:** cross-cutting-audit
- **Location:** `crates/cross-cutting/audit/src/writer.rs:1456-1472` (`advance_sweep_clock_first_call_seeds_returns_false`)

**Description:**

The unit test `advance_sweep_clock_first_call_seeds_returns_false` at lines 1456-1472 ends with `let _ = (clock, policy, now);` — it binds three locals and then discards them. There are no assertions. The test name advertises "first call seeds returns false" but the test body has no `advance_sweep_clock(...)` call, no `assert!`, no `assert_eq!`. The test passes (because `cargo test` runs to completion) but does not validate the helper it claims to validate. Per AGENTS.md and `docs/code-standards.md`, every test must validate a real-world scenario.

**Expected:**

A test that constructs an `AuditWriter` (or exercises the helper directly via a `pub(crate)` accessor) and asserts that `advance_sweep_clock(t0)` records `t0` and returns `false`.

**Evidence:**

```rust
  // crates/cross-cutting/audit/src/writer.rs:1456-1472
  #[test]
  fn advance_sweep_clock_first_call_seeds_returns_false() {
      use educore_core::clock::TestClock;
      let clock = std::sync::Arc::new(TestClock::new());
      let policy = RetentionPolicy::default();
      let now = Timestamp::from_datetime(Utc::now());
      let _ = (clock, policy, now);
      // NO ASSERTIONS — test passes regardless of behaviour
  }
  ```

---

### FINDING 20 (id: `CC-AUD-020`)

- **Source:** `docs/audit_reports/findings/wave2-audit.md`
- **Severity:** High
- **Area:** cross-cutting-audit
- **Location:** `crates/cross-cutting/audit/src/writer.rs` (entire file) and `crates/infra/storage/src/audit.rs:117-156` (AuditLog trait)

**Description:**

There is no `verify_chain` function on the `AuditWriter` and no `verify` method on the `AuditLog` trait. Per `docs/schemas/audit-schema.md` § 3 the audit log must be tamper-evident; the spec mandates WORM as the default defense but also leaves room for hash-chain verification. Even without a hash chain (Finding 2), the engine should expose a `verify_no_gaps(school_id, since, until) -> Result<()>` function that asserts `occurred_at` is monotonically non-decreasing per `(school_id, audit_id)` ordering — a minimal integrity check that can detect accidental gaps from a misconfigured retention sweep.

**Expected:**

A `verify_audit_log(school_id, since, until)` function that re-reads the audit log via `read_for_target` and asserts monotonic ordering. Plus, once a hash chain is added (Finding 2), a `verify_chain(school_id, since, until)` that walks `prev_audit_hash` links.

**Evidence:**

`grep -n 'fn verify' crates/cross-cutting/audit/src/*.rs crates/infra/storage/src/audit.rs` returns zero hits.

---

### FINDING 9 (id: `CC-AUD-009`)

- **Source:** `docs/audit_reports/findings/wave2-audit.md`
- **Severity:** High
- **Area:** cross-cutting-audit
- **Location:** `crates/cross-cutting/audit/src/retention.rs:34-58` (RetentionPolicy::default) vs `docs/schemas/audit-schema.md:271-285` (retention table)

**Description:**

`RetentionPolicy::default()` returns `retention_days: 90`. `docs/schemas/audit-schema.md` § 9 specifies: 7 years for capability/role changes, finance mutations, payroll mutations, and academic mutations; 36 months for authorization denials and AI agent actions; 18 months for authentication events; 3 years for library/facilities, settings, and backups. The engine's default (90 days) destroys finance and academic audit trails within 3 months — a regulatory non-conformance for FERPA, GDPR financial-record retention, and the § 9 spec table.

**Expected:**

Per `docs/schemas/audit-schema.md` § 9: a tiered retention model. The default `RetentionPolicy` should be the lowest-tier (18 months / authentication) and consumers in regulated sectors must override per category. At minimum, separate policies for `auth`, `authz_denial`, `capability_change`, `finance`, `payroll`, `academic`, `library_facilities`, `settings`, `backup`, `agent`.

**Evidence:**

```rust
  // crates/cross-cutting/audit/src/retention.rs:49-57
  fn default() -> Self {
      Self {
          retention_days: 90,
          sweep_check_interval: Duration::from_secs(3600),
      }
  }
  ```

---

### FINDING 21 (id: `CC-AUD-021`)

- **Source:** `docs/audit_reports/findings/wave2-audit.md`
- **Severity:** Medium
- **Area:** cross-cutting-audit
- **Location:** `crates/cross-cutting/audit/src/writer.rs:101-115` (AuditAction enum)

**Description:**

`AuditAction` has 7 fixed variants (`Create`, `Update`, `Delete`, `Approve`, `Login`, `Logout`, `Configure`) plus the catch-all `Other(String)`. `docs/schemas/audit-schema.md` § 4 lists 9 categories that must be recorded: state-changing commands (create/update/delete), failed commands, authentication events, authorization events, capability/role changes, cross-tenant operations, backup/restore/migration, security-relevant settings changes, school lifecycle. The engine's 7-variant enum collapses all of these into 7 wire strings; a regulator filtering on `action = 'capability_grant'` (one of the spec's named categories) cannot because the wire form is either `configure` or an unconstrained `Other("capability_grant")` string with no validation. The `Other` arm accepts arbitrary strings — no allow-list, no schema version check.

**Expected:**

Either expand `AuditAction` to cover all spec § 4 categories (`Fail`, `Authorize`, `CapabilityGrant`, `CapabilityRevoke`, `CrossTenant`, `Backup`, `Restore`, `Migrate`, `SchoolOnboard`, `SchoolSuspend`, `SchoolTransfer`) or introduce a separate `AuditActionCategory` enum at the port boundary so `Other(String)` cannot bypass the taxonomy.

**Evidence:**

```rust
  // crates/cross-cutting/audit/src/writer.rs:101-115
  pub enum AuditAction {
      Create, Update, Delete, Approve, Login, Logout, Configure,
      Other(String),
  }
  ```

---

### FINDING 22 (id: `CC-AUD-022`)

- **Source:** `docs/audit_reports/findings/wave2-audit.md`
- **Severity:** Medium
- **Area:** cross-cutting-audit
- **Location:** `crates/adapters/storage-postgres/src/audit_log.rs:146` (`let audit_id = Uuid::now_v7();`) and `crates/adapters/storage-mysql/src/audit_log.rs:130` and `crates/adapters/storage-sqlite/src/audit_log.rs:118`

**Description:**

Each SQL adapter regenerates `audit_id` at write time via `Uuid::now_v7()`. The `AuditLogEntry` struct does not carry an `audit_id` field (Finding 3). If the storage-port `AuditLog::append` is retried (the at-least-once delivery model the engine uses, per `docs/schemas/command-schema.md`), each retry mints a new `audit_id` — the audit row is duplicated with a new primary key. There is no idempotency key for the audit table itself; dedup must rely on `(school_id, correlation_id, target_id, action)` and that index does not exist in any of the three DDLs.

**Expected:**

Per `docs/schemas/command-schema.md` idempotency model: the audit writer accepts an optional idempotency key (typically the originating `command_id`), and the DDL has a unique index on `(school_id, command_id)` so retried appends fail with a constraint violation the engine catches.

**Evidence:**

```rust
  // crates/adapters/storage-postgres/src/audit_log.rs:146
  let audit_id = Uuid::now_v7();  // New every retry
  ```

---

### FINDING 23 (id: `CC-AUD-023`)

- **Source:** `docs/audit_reports/findings/wave2-audit.md`
- **Severity:** Medium
- **Area:** cross-cutting-audit
- **Location:** `crates/cross-cutting/audit/src/writer.rs:530-760` (`AuditTarget::target_id`) and `crates/cross-cutting/audit/src/writer.rs:222-230` (`AuditTarget::Other(String, Uuid)`)

**Description:**

`AuditTarget::Other(String, Uuid)` accepts an arbitrary `target_type` string with no validation. A consumer can construct `AuditTarget::Other("../../etc/passwd".to_owned(), Uuid::now_v7())` and the wire form passes through unchanged. Per the AGENTS.md Engine Rule #2 ("Compile-time safety over strings. Use macro-generated enums — never string field names"), the audit crate's catch-all variant is itself a string the macro-equivalent generator never validates. The aggregate variant list has 130+ entries — adding a new aggregate requires editing both the enum and the 130+ arm `match` in `target_type()` and `target_id()`, but the `Other` variant lets callers skip that work and bypass the compile-time safety.

**Expected:**

Either remove `AuditTarget::Other` (force compile-time registration for every aggregate) or constrain `Other(String, Uuid)` to a `&'static str` with a regex allow-list of `[a-z][a-z0-9_]*`.

**Evidence:**

```rust
  // crates/cross-cutting/audit/src/writer.rs:222-230
  Other(String, Uuid),
  ```

---

### FINDING 24 (id: `CC-AUD-024`)

- **Source:** `docs/audit_reports/findings/wave2-audit.md`
- **Severity:** Medium
- **Area:** cross-cutting-audit
- **Location:** `docs/coverage.toml:75-99` (audit_log_ddl_{pg,mysql,sqlite} rows) and `crates/cross-cutting/audit/tests/audit_e2e.rs` (entire file)

**Description:**

`docs/coverage.toml` rows 75-99 mark `audit_log_ddl_pg`, `audit_log_ddl_mysql`, `audit_log_ddl_sqlite` as `status = "Tested"` with `tests = "crates/cross-cutting/audit/tests/audit_e2e.rs"`. But `audit_e2e.rs` contains zero DDL verification — `grep -n 'ddl\|0000_engine\|migrate\|DDL' crates/cross-cutting/audit/tests/audit_e2e.rs` returns zero hits. The tests exercise `AuditWriter` against in-memory mocks (`InMemoryAuditLog` at lines 60-95); the canonical `migrations/engine/*.sql` DDL is never loaded, parsed, or compared. The "Tested" status is incorrect coverage.

**Expected:**

A DDL byte-match test per `crates/adapters/storage-surrealdb/tests/outbox_e2e.rs` pattern: load the SQL file, run it against the dialect, read back `information_schema.columns`, and assert the 20 audit_log columns and 5 indexes match the spec.

**Evidence:**

`grep -n 'ddl\|0000_engine\|migrate\|DDL\|CREATE TABLE' crates/cross-cutting/audit/tests/audit_e2e.rs` returns zero matches.

---

### FINDING 25 (id: `CC-AUD-025`)

- **Source:** `docs/audit_reports/findings/wave2-audit.md`
- **Severity:** Medium
- **Area:** cross-cutting-audit
- **Location:** `crates/cross-cutting/audit/src/events.rs:35-110` (`RetentionSweepDue` event) and `crates/cross-cutting/audit/src/writer.rs:931-951` (emit_sweep_due)

**Description:**

`RetentionSweepDue::new(school_id, cutoff, at)` mints the `event_id` via `Uuid::now_v7()`. There is no `audit_id` of the row that triggered the sweep, no `actor_id` (only the system actor via `TenantContext::system`), no `target_type`, no `command_id`. The event payload is the three-field struct with no schema-versioned extension point. Per `docs/schemas/audit-schema.md` § 15 ("Audit-Driven Subscriptions"), subscribers need the trigger context; the current event is too narrow to support a sweep that distinguishes per-category retentions (Finding 16) or archives before deleting (spec § 9: "archive to cold storage and remove from the active audit log").

**Expected:**

Extend `RetentionSweepDue` with `oldest_audit_id: Uuid`, `oldest_target_type: String`, `retention_category: RetentionCategory`, `action: SweepAction { Archive, Purge, ArchiveThenPurge }`.

**Evidence:**

```rust
  // crates/cross-cutting/audit/src/events.rs:50-60
  pub struct RetentionSweepDue {
      pub event_id: Uuid,
      pub school_id: SchoolId,
      pub cutoff: Timestamp,
      pub at: Timestamp,
  }
  ```

---

### FINDING 26 (id: `CC-AUD-026`)

- **Source:** `docs/audit_reports/findings/wave2-audit.md`
- **Severity:** Medium
- **Area:** cross-cutting-audit
- **Location:** `crates/cross-cutting/audit/src/writer.rs:50-760` (AuditTarget enum)

**Description:**

The `AuditTarget` enum has 130+ variants (cross-cutting + 10 domains + 10 Phase 15 port-adapter targets). The naming is inconsistent within the finance domain: `WalletTransaction` (one variant) and `Transaction` (a separate variant for the double-entry journal line). The variant names `WalletTransaction` and `Transaction` collide in human reading — both wire forms are distinct (`"wallet_transaction"` vs `"transaction"`) but the spec at `docs/specs/finance/aggregates.md` would need careful cross-referencing to map domain aggregates to enum variants. The Phase 2 placeholder `SchoolSettings` and `BellSchedule` are kept as variants even though `educore-settings` defines typed replacements (`GeneralSettings`, `Language`, etc.) — per the comment at `writer.rs:392-400`, the placeholders are kept "for `DefaultRoleCatalog` consistency" but the role catalog has no dependency on the audit enum's variant list.

**Expected:**

Rename `Transaction` to `JournalEntry` (or similar) to disambiguate from `WalletTransaction`. Audit the 130+ variants against `docs/specs/*/aggregates.md` for one-to-one mapping and rename anything ambiguous.

**Evidence:**

```rust
  // crates/cross-cutting/audit/src/writer.rs:222-224, 234-236
  WalletTransaction(Uuid),
  ...
  Transaction(Uuid),  // double-entry journal line — name collision with WalletTransaction
  ```

---

### FINDING 27 (id: `CC-AUD-027`)

- **Source:** `docs/audit_reports/findings/wave2-audit.md`
- **Severity:** Medium
- **Area:** cross-cutting-audit
- **Location:** `crates/cross-cutting/audit/tests/audit_e2e.rs:64-95` (InMemoryAuditLog mock)

**Description:**

The in-memory `AuditLog` mock used by `audit_e2e.rs` interprets the `SENTINEL_TARGET_ID` as "return the oldest entry for the school". The PG/MySQL/SQLite adapters interpret `read_for_target(school, SENTINEL_TARGET_ID, limit)` literally — they run `WHERE school_id = $1 AND resource_id = $2` (PG at `audit_log.rs:201`), so the sentinel lookup against a real database returns zero rows (no row has `target_id = Uuid::nil()`). The e2e tests therefore exercise a behaviour the SQL adapters do not implement. The `maybe_sweep` method is therefore untested against the SQL adapters — the integration test that the cross-cutting integration test (`crates/tools/storage-parity/tests/cross_cutting_integration.rs`) would need to run is missing.

**Expected:**

Either add a dedicated `oldest_row_for_school(school_id) -> Option<AuditLogEntry>` method to the `AuditLog` trait (per Finding 10), or have the SQL adapters' `read_for_target` short-circuit on the sentinel.

**Evidence:**

```rust
  // crates/adapters/storage-postgres/src/audit_log.rs:201
  WHERE school_id = $1 AND resource_id = $2  // SENTINEL_TARGET_ID matches no real row
  // crates/cross-cutting/audit/tests/audit_e2e.rs:75-87 (mock honours sentinel differently)
  ```

---

### FINDING 28 (id: `CC-AUD-028`)

- **Source:** `docs/audit_reports/findings/wave2-audit.md`
- **Severity:** Medium
- **Area:** cross-cutting-audit
- **Location:** `crates/adapters/storage-surrealdb/src/audit.rs:23-42` (module header doc) and `crates/adapters/storage-surrealdb/src/lib.rs`

**Description:**

The SurrealDB `AuditLog` implementation exists at `crates/adapters/storage-surrealdb/src/audit.rs` with full `AuditRow`, `from_entry`, `to_entry`, `append`, `read_for_target`, and 5 unit tests, but the module is **not wired into `lib.rs`** (the file header at lines 23-42 acknowledges this). The `stubs.rs` `SurrealAuditLog` stub is still the active implementation. `AGENTS.md` line 8 (Storage Adapters) explicitly states the SurrealDB adapter is "deferred to a future release and is not shipped from the engine". The crate ships a working 290-line `audit.rs` file that is dead code from the consumer's perspective.

**Expected:**

Either remove `crates/adapters/storage-surrealdb/src/audit.rs` (per `AGENTS.md` "SurrealDB and MongoDB adapters are deferred to a future release and are not shipped from the engine"), or wire it into `lib.rs` and ship the adapter.

**Evidence:**

`crates/adapters/storage-surrealdb/src/audit.rs:23-42` "This module is **not yet wired into `lib.rs`** — A'.1 will add `pub mod audit;` to the crate root once the stub in `stubs.rs` has been removed."

---

### FINDING 29 (id: `CC-AUD-029`)

- **Source:** `docs/audit_reports/findings/wave2-audit.md`
- **Severity:** Low
- **Area:** cross-cutting-audit
- **Location:** `crates/cross-cutting/audit/src/writer.rs:820-823` (AuditWriter::new) and `crates/cross-cutting/audit/src/writer.rs:903-929` (advance_sweep_clock) vs `crates/cross-cutting/audit/src/retention.rs:106-160` (RetentionSweeper)

**Description:**

Two implementations of the threshold-check state machine exist: `RetentionSweeper::should_sweep(now, policy)` in `retention.rs` (lines 132-148) and `AuditWriter::advance_sweep_clock(now)` in `writer.rs` (lines 904-929). They implement identical logic (`last_sweep_at` seeding + interval comparison) with the only difference being the storage of `last_sweep_at` (one is an `Option<Timestamp>` field on the struct, the other a `Mutex<Option<Timestamp>>` inside `AuditWriter`). The two paths can drift if one is updated and the other forgotten. The unit tests cover `RetentionSweeper` (`retention.rs:178-269`) but `advance_sweep_clock` has only the no-op test from Finding 19.

**Expected:**

Delete `advance_sweep_clock` and call `RetentionSweeper::should_sweep` from `maybe_sweep`. The `AuditWriter` can hold a `RetentionSweeper` instance instead of a raw `Mutex<Option<Timestamp>>`.

**Evidence:**

```rust
  // crates/cross-cutting/audit/src/retention.rs:132-148 (RetentionSweeper::should_sweep)
  pub fn should_sweep(&mut self, now: Timestamp, policy: &RetentionPolicy) -> bool { ... }
  // crates/cross-cutting/audit/src/writer.rs:904-929 (AuditWriter::advance_sweep_clock)
  fn advance_sweep_clock(&self, now: Timestamp) -> bool { ... }
  // Same logic, duplicated
  ```

---

### FINDING 30 (id: `CC-AUD-030`)

- **Source:** `docs/audit_reports/findings/wave2-audit.md`
- **Severity:** Low
- **Area:** cross-cutting-audit
- **Location:** `docs/ports/` directory listing — `audit.md` does not exist (the ports folder contains authentication.md, event-bus.md, file-storage.md, integrations.md, notifications.md, payments.md, storage.md, sync.md — no `audit.md`)

**Description:**

`docs/ports/audit.md` is not present. Per the `AGENTS.md` Authoritative Documents list ("`docs/ports/*.md` — port contracts"), every engine port has a port-contract markdown. The `AuditLog` sub-port is declared at `crates/infra/storage/src/audit.rs:117-156`, has 4 adapter implementations, and is consumed by every domain's service factories — yet has no port contract. Consumers integrating a new dialect or a custom audit sink have no specification document to follow; the only references are the 12-line `///` doc comments on `AuditLog` itself.

**Expected:**

A `docs/ports/audit.md` following the structure of `docs/ports/storage.md` and `docs/ports/event-bus.md`: trait signature, adapter inventory, per-dialect notes, atomicity contract, idempotency contract, WORM deployment guide.

**Evidence:**

`ls docs/ports/` shows no `audit.md`; only `authentication.md`, `event-bus.md`, `file-storage.md`, `integrations.md`, `notifications.md`, `payments.md`, `storage.md`, `sync.md`.

---


## Sync (cross-cutting) (target id prefix: `CC-SYNC`)

**Path:** `crates/cross-cutting/sync/`  
**Total findings:** 27 (10 critical, 10 high, 6 medium, 1 low)


### FINDING 1 (id: `CC-SYNC-001`)

- **Source:** `docs/audit_reports/findings/wave2-sync.md`
- **Severity:** Critical
- **Area:** cross-cutting-sync
- **Location:** `crates/educore/Cargo.toml:20-49` (no `[features]` block)

**Description:**

ADR-018 § 4 mandates a `sync` Cargo feature on the umbrella crate that gates `educore-sync` and `educore-sync-inprocess`: "`Without the `sync` feature, the engine has **no** sync capability (the `sync()` builder method is gated behind the feature). With the feature on, consumers pick: ...`". The actual `crates/educore/Cargo.toml` has **no `[features]` block at all**; both `educore-sync` and `educore-sync-inprocess` are unconditional dependencies (lines 46-47). The umbrella therefore pulls sync in for every consumer, including server-only deployments.

**Expected:**

`docs/decisions/ADR-018-SyncEngineArchitecture.md:101-115` — `[features] default = []; sync = ["educore-sync", "educore-sync-inprocess"]` on `crates/educore/Cargo.toml`. Also `docs/architecture.md:362-364`: "The `sync` feature on the umbrella crate (`educore`) gates the in-process coordinator."

**Evidence:**

`crates/educore/Cargo.toml` — `grep -n "feature" crates/educore/Cargo.toml` returns only the line 50 `tokio = { workspace = true, features = ["macros", ...] }` dev-dependency; there is no `[features]` table on the umbrella.

---

### FINDING 10 (id: `CC-SYNC-010`)

- **Source:** `docs/audit_reports/findings/wave2-sync.md`
- **Severity:** Critical
- **Area:** cross-cutting-sync
- **Location:** `docs/specs/sync/overview.md:883,1048` and `:42,64,849,885,941`

**Description:**

The sync spec repeatedly references a worker binary `educore-worker` and a server crate `educore-sync-server` / `educore-sync-server-http`. Neither exists in the workspace. The actual crates are `educore-sync` and `educore-sync-inprocess` (cross-cutting tier). The umbrella binary is `educore-cli` (per AGENTS.md Crate Inventory row 35). The spec's references to `educore-worker` and `educore-sync-server` are non-existent constructs. Note: this finding is also filed in `wave6-specs-4.md` finding 7 (SPEC-4-007) but is restated here because it is a blocker for sync engine deployment.

**Expected:**

Sync spec should reference the actual crate names; the worker binary (if/when shipped) must be a real workspace member.

**Evidence:**

`grep -n "educore-worker\|educore-sync-server" docs/specs/sync/overview.md` returns 6 rows: `:42, :64, :849, :883, :885, :941, :1048`. `find crates -type d -name "*worker*" -o -name "*sync-server*"` returns no rows.

---

### FINDING 11 (id: `CC-SYNC-011`)

- **Source:** `docs/audit_reports/findings/wave2-sync.md`
- **Severity:** Critical
- **Area:** cross-cutting-sync
- **Location:** `crates/cross-cutting/sync-inprocess/src/lib.rs` (entire crate, 390 lines)

**Description:**

Per `docs/build-plan.md:217-222` Phase 0 task 10, the sync integration test "insert one outbox row and verify the in-process consumer received the event via the `SyncCoordinator`". The actual integration test in `crates/cross-cutting/sync-inprocess/src/lib.rs:204-390` only exercises the `InProcessSyncAdapter`'s own `start`/`pause`/`resume`/`stop` lifecycle against an `InProcessEventBus`. It never inserts an outbox row; it never invokes the `Outbox` sub-port on any storage adapter; it never asserts a domain event flows from a storage operation through the sync engine to a consumer. The Phase 0 sync e2e referenced in the handoff (`crates/adapters/storage-surrealdb/tests/outbox_e2e.rs`) does the outbox round-trip but does **not** wire `InProcessSyncAdapter` to receive the event.

**Expected:**

`docs/build-plan.md:217-222` — "with the in-process sync impl wired into the Phase 0 outbox scenario, insert one outbox row and verify the in-process consumer received the event via the `SyncCoordinator`".

**Evidence:**

`grep -n "outbox\|Outbox" crates/cross-cutting/sync-inprocess/src/lib.rs` returns no rows; `grep -n "InProcessSyncAdapter\|educore_sync" crates/adapters/storage-surrealdb/tests/outbox_e2e.rs` returns no rows.

---

### FINDING 2 (id: `CC-SYNC-002`)

- **Source:** `docs/audit_reports/findings/wave2-sync.md`
- **Severity:** Critical
- **Area:** cross-cutting-sync
- **Location:** `docs/decisions/ADR-018-SyncEngineArchitecture.md:92-100` vs `crates/cross-cutting/sync-inprocess/` (on disk)

**Description:**

ADR-018 § 3 declares the in-process adapter lives at `crates/adapters/sync-inprocess/` (package `educore-sync-inprocess`). The actual crate lives at `crates/cross-cutting/sync-inprocess/` (cross-cutting tier), not `crates/adapters/sync-inprocess/`. AGENTS.md says the same (`sync-inprocess` is under cross-cutting), but the ADR's stated location disagrees. The `adapters/` tier directory does not have a `sync-inprocess/` subdirectory.

**Expected:**

ADR says: `crates/adapters/sync-inprocess/` (adapters tier).

**Evidence:**

`find crates -type d -name "sync*"` returns `/home/beznet/Workspace/smscore/crates/cross-cutting/sync-inprocess` and `/home/beznet/Workspace/smscore/crates/cross-cutting/sync`. `ls crates/adapters/` contains `auth, event-bus, files, integrations, notify, payment, storage-mysql, storage-postgres, storage-sqlite, storage-surrealdb` — no `sync-inprocess`.

---

### FINDING 3 (id: `CC-SYNC-003`)

- **Source:** `docs/audit_reports/findings/wave2-sync.md`
- **Severity:** Critical
- **Area:** cross-cutting-sync
- **Location:** `docs/specs/sync/` (directory contents)

**Description:**

`docs/specs/sync/` contains only `overview.md` (1162 lines, dated Phase 0). The 11-file layout mandated by `docs/code-standards.md` (overview, aggregates, entities, value-objects, commands, events, services, permissions, repositories, workflows, tables) requires 10 additional files. None of the documented sync aggregates (`OutboxEntry`, `SyncCursor`, `ConflictRecord`, `SyncSubscription`), any value object, any commands spec, any events spec, any services spec, any permissions spec, any repository spec, any workflow spec, any tables spec exist as spec files. The single `overview.md` is also a norm-violating dump of the entire spec into one file (1162 lines).

**Expected:**

`docs/code-standards.md` "Spec folder layout" — 11 files per domain/cross-cutting folder.

**Evidence:**

`ls /home/beznet/Workspace/smscore/docs/specs/sync/` returns `overview.md` only. The 11-file layout is visible in `docs/specs/platform/` (11 files), `docs/specs/academic/` (11 files), etc.

---

### FINDING 4 (id: `CC-SYNC-004`)

- **Source:** `docs/audit_reports/findings/wave2-sync.md`
- **Severity:** Critical
- **Area:** cross-cutting-sync
- **Location:** `docs/build-plan.md:192-200` (Phase 0 task 6) and `docs/specs/sync/overview.md:355,372,390,409,427,447,461` (spec body)

**Description:**

Both `build-plan.md` Phase 0 task 6 and the spec body reference a `SyncCoordinator` struct/trait and a 5-command / 7-event catalog. The actual code defines a different trait (`SyncAdapter`, `crates/cross-cutting/sync/src/port.rs:37`) with a 4-command catalog (`Start`/`Pause`/`Resume`/`Stop`, `crates/cross-cutting/sync/src/command.rs:23-37`) and 4 events (`SyncStarted`/`SyncPaused`/`SyncResumed`/`SyncStopped`, `crates/cross-cutting/events/src/sync.rs`). The `SyncCoordinator` symbol exists in **no** Rust source file (`grep -rn "SyncCoordinator" crates --include="*.rs"` returns no rows).

**Expected:**

`docs/build-plan.md:193-198` — "Defines the `SyncCoordinator` trait, the command catalog (`SyncStart`, `SyncPause`, `SyncResume`, `SyncRequestDelta`, `SyncAcknowledge`), the event catalog (`SyncStarted`, `SyncPaused`, `SyncResumed`, `DeltaAvailable`, `DeltaAcknowledged`, `SyncConflictDetected`), and the shared coordinator struct". Also `docs/specs/sync/overview.md:355,372,...` 7 spec-body events.

**Evidence:**

`grep -rn SyncCoordinator crates --include="*.rs"` returns zero hits; `crates/cross-cutting/sync/src/port.rs:37` defines `pub trait SyncAdapter: Send + Sync` with five methods (`start`, `pause`, `resume`, `stop`, `health`).

---

### FINDING 5 (id: `CC-SYNC-005`)

- **Source:** `docs/audit_reports/findings/wave2-sync.md`
- **Severity:** Critical
- **Area:** cross-cutting-sync
- **Location:** `docs/build-plan.md:194-195` vs `crates/cross-cutting/sync/src/command.rs:23-37`

**Description:**

The build plan mandates 5 sync commands: `SyncStart`, `SyncPause`, `SyncResume`, `SyncRequestDelta`, `SyncAcknowledge`. The actual command catalog has 4: `SyncCommand::Start(SchoolId)`, `::Pause(SchoolId)`, `::Resume(SchoolId)`, `::Stop(SchoolId)` (note: `Stop` not `Acknowledge`). `SyncRequestDelta` does not exist. `SyncAcknowledge` does not exist. `Stop` (which exists in code) is not listed in the build plan.

**Expected:**

`docs/build-plan.md:194-195` — "`SyncStart`, `SyncPause`, `SyncResume`, `SyncRequestDelta`, `SyncAcknowledge`".

**Evidence:**

`crates/cross-cutting/sync/src/command.rs:23-37`:
  ```rust
  pub enum SyncCommand {
      Start(SchoolId),
      Pause(SchoolId),
      Resume(SchoolId),
      Stop(SchoolId),
  }
  ```

---

### FINDING 6 (id: `CC-SYNC-006`)

- **Source:** `docs/audit_reports/findings/wave2-sync.md`
- **Severity:** Critical
- **Area:** cross-cutting-sync
- **Location:** `docs/build-plan.md:196-197` vs `crates/cross-cutting/events/src/sync.rs:64-225`

**Description:**

The build plan and the Phase 0 handoff list 6 sync events: `SyncStarted`, `SyncPaused`, `SyncResumed`, `DeltaAvailable`, `DeltaAcknowledged`, `SyncConflictDetected`. The actual `educore-events/src/sync.rs` defines **4** events: `SyncStarted` (line 64), `SyncPaused` (line 122), `SyncResumed` (line 158), `SyncStopped` (line 185). The four names in the plan that are not in code: `DeltaAvailable`, `DeltaAcknowledged`, `SyncConflictDetected`. The one name in code that is not in the plan: `SyncStopped`.

**Expected:**

`docs/build-plan.md:196-197` — "`SyncStarted`, `SyncPaused`, `SyncResumed`, `DeltaAvailable`, `DeltaAcknowledged`, `SyncConflictDetected`" and `docs/handoff/PHASE-0-HANDOFF.md:36-37` — "SyncStarted, SyncPaused, SyncResumed, DeltaAvailable, DeltaAcknowledged. Missing: SyncAcknowledge command, SyncConflictDetected event."

**Evidence:**

`crates/cross-cutting/events/src/sync.rs` — `grep -nE "^pub struct (Sync|Delta)" src/sync.rs` returns 4 rows: `SyncStarted`, `SyncPaused`, `SyncResumed`, `SyncStopped`. `grep -n "DeltaAvailable\|DeltaAcknowledged\|SyncConflictDetected" crates/cross-cutting/events/src/sync.rs` returns no rows.

---

### FINDING 7 (id: `CC-SYNC-007`)

- **Source:** `docs/audit_reports/findings/wave2-sync.md`
- **Severity:** Critical
- **Area:** cross-cutting-sync
- **Location:** `docs/ports/sync.md:48-58` (`SyncAdapter` trait) vs `crates/cross-cutting/sync/src/port.rs:37-66` (actual `SyncAdapter`)

**Description:**

The wire-protocol port doc `docs/ports/sync.md` defines a `SyncAdapter` trait with four async methods: `dispatch(envelope)`, `subscribe(filter) -> EventStream`, `snapshot(school_id)`, `health()`. The actual port trait `crates/cross-cutting/sync/src/port.rs:37-66` defines a `SyncAdapter` trait with five methods: `start`, `pause`, `resume`, `stop`, `health`. **None** of `dispatch`/`subscribe`/`snapshot`/`CommandEnvelope`/`EventStream`/`SchoolSnapshot`/`CommandOutcome`/`EventFilter` (the entire port doc API surface, lines 60-417) is implemented in code. The port doc and the code port have **zero overlapping methods** beyond `health`.

**Expected:**

`docs/ports/sync.md:48-58` — `pub trait SyncAdapter { async fn dispatch(...); async fn subscribe(...); async fn snapshot(...); async fn health(...); }`.

**Evidence:**

`docs/ports/sync.md:48-58` vs `crates/cross-cutting/sync/src/port.rs:37-66`. `grep -rn "CommandEnvelope\|EventStream\|SchoolSnapshot\|CommandOutcome\|fn dispatch\|fn subscribe" crates/cross-cutting/sync crates/cross-cutting/sync-inprocess` returns no rows.

---

### FINDING 9 (id: `CC-SYNC-009`)

- **Source:** `docs/audit_reports/findings/wave2-sync.md`
- **Severity:** Critical
- **Area:** cross-cutting-sync
- **Location:** `crates/adapters/storage-surrealdb/src/storage.rs:129-167`

**Description:**

ADR-018 § 5 states "Only the four shipped storage adapters override these methods: `educore-storage-surrealdb` (Phase 0 primary; per ADR-017)...". The actual `SurrealStorageAdapter::watch_changes`, `apply_snapshot`, `cursor_for`, `advance_cursor` (lines 129-167) are all **stubbed** with `NotSupported` — the SurrealDB adapter does **not** override the sync methods. `apply_snapshot` returns "SurrealStorageAdapter::apply_snapshot is not yet implemented" (line 147). The Phase 0 primary sync engine target therefore has zero sync-port coverage; the in-process sync adapter only emits lifecycle events and never reads from the storage change feed.

**Expected:**

`docs/decisions/ADR-018-SyncEngineArchitecture.md:188-194` — "Only the four shipped storage adapters override these methods: educore-storage-surrealdb...".

**Evidence:**

`crates/adapters/storage-surrealdb/src/storage.rs:129-167` (full block). All four methods return `NotSupported` after logging "StorageAdapter::watch_changes called on a closed adapter" / "apply_snapshot is not yet implemented". The Phase 0 e2e test (`crates/adapters/storage-surrealdb/tests/outbox_e2e.rs`) never calls any of the four sync methods.

---

### FINDING 12 (id: `CC-SYNC-012`)

- **Source:** `docs/audit_reports/findings/wave2-sync.md`
- **Severity:** High
- **Area:** cross-cutting-sync
- **Location:** `crates/tools/testkit/src/sync.rs:1-43` (entire file)

**Description:**

`crates/tools/testkit/src/sync.rs` is a 43-line placeholder. It exports a single `dummy_witness()` no-op function. The crate's doc-comment (lines 14-23) says "The actual sync primitives (`ChangeStream`, `VersionCursor`, `watch_changes`, `apply_snapshot`, `cursor_for`, `advance_cursor`) are exposed as methods on the in-memory storage adapter — see `storage::InMemoryStorageAdapter`." The testkit therefore does not expose any sync primitives of its own, yet `docs/coverage.toml:2193` declares `tests = "crates/tools/testkit/src/{storage,auth,notify,payment,files,integrations,event_bus,sync}.rs"` as if `sync.rs` provides sync test fixtures. The placeholder function does not consume a `SyncAdapter` nor a `StorageAdapter`.

**Expected:**

A testkit module exposing pre-built `InProcessSyncAdapter` instances wired to a `tokio::sync::broadcast` consumer registry (per `docs/architecture.md:355-364` "30 minutes to a working offline-first app").

**Evidence:**

`crates/tools/testkit/src/sync.rs:35-40` — `pub fn dummy_witness() {}` only. `crates/tools/testkit/src/sync.rs:1-43` total — no types, no traits, no struct.

---

### FINDING 13 (id: `CC-SYNC-013`)

- **Source:** `docs/audit_reports/findings/wave2-sync.md`
- **Severity:** High
- **Area:** cross-cutting-sync
- **Location:** `docs/specs/sync/overview.md:1134-1140` (Phase 0 status block) vs spec body `:501,517,533,547,563,579`

**Description:**

The "Phase 0 status" block at `docs/specs/sync/overview.md:1134-1140` uses command and event names that do not appear anywhere else in the spec body or in code: `SyncStart`, `SyncPause`, `SyncResume`, `SyncRequestDelta`, `SyncAcknowledge`, `DeltaAvailable`, `DeltaAcknowledged`, `SyncConflictDetected`. The spec body (lines 501-588) uses `RequestSyncCommand`, `PauseSyncCommand`, `ResumeSyncCommand`, `ResolveConflictCommand`, `SwitchSchoolCommand`, `ApplyRemoteChangeCommand`. The code uses `SyncCommand::{Start,Pause,Resume,Stop}` and events `SyncStarted/SyncPaused/SyncResumed/SyncStopped`. Three sources (Phase 0 status block, spec body, code) name the same surface three different ways. `SyncStopped` is claimed "deferred" at line 1140 but is the only Stop-equivalent in code (`crates/cross-cutting/events/src/sync.rs:185`).

**Expected:**

A single canonical command/event catalog, used by the spec body, the Phase 0 status block, and the code.

**Evidence:**

`docs/specs/sync/overview.md:1134-1140`:
  ```text
  **Commands shipped (4 of 6):** `SyncStart`, `SyncPause`,
  `SyncResume`, `SyncRequestDelta`. The `SyncAcknowledge`
  command is deferred (the in-process impl acknowledges
  inline in the test path).
  **Events shipped (5 of 7):** `SyncStarted`, `SyncPaused`,
  `SyncResumed`, `DeltaAvailable`, `DeltaAcknowledged`.
  `SyncConflictDetected` and `SyncStopped` are deferred.
  ```
  vs `docs/specs/sync/overview.md:501` `## RequestSyncCommand`. Same overlap with `docs/audit_reports/findings/wave6-specs-4.md` finding SPEC-4-006.

---

### FINDING 14 (id: `CC-SYNC-014`)

- **Source:** `docs/audit_reports/findings/wave2-sync.md`
- **Severity:** High
- **Area:** cross-cutting-sync
- **Location:** `docs/specs/sync/overview.md:222-227` (SyncSubscription aggregate) and `:245-258` (SyncSubscription invariants)

**Description:**

`SyncSubscription` is documented as an aggregate with `Idle`/`Streaming`/`Backoff`/`Paused`/`Stalled` states, per-aggregate-type subscriptions, `pause`/`resume` semantics, and a backoff policy. The implementation collapses subscription state to a single `SyncStatus` enum (`Running`/`Paused`/`Stopped`, `crates/cross-cutting/sync/src/health.rs:23-35`) at the **adapter** level, not per-(school, aggregate_type). There is no `SyncSubscription` struct, no `SubscriptionState` enum with `Streaming`/`Backoff`/`Stalled` variants, no per-aggregate-type cursor table, no backoff policy implementation. A multi-school consumer (`SwitchSchoolCommand` per `:547-563`) is impossible.

**Expected:**

`docs/specs/sync/overview.md:226-227` and `:245-258` — SyncSubscription as a per-(school, aggregate_type, client_id) aggregate with five-state state machine.

**Evidence:**

`crates/cross-cutting/sync/src/health.rs:23-35` defines only `enum SyncStatus { Running, Paused, Stopped }` (3 states, not 5). `grep -rn "SyncSubscription\|SubscriptionState\|Stalled\|Backoff" crates/cross-cutting/sync crates/cross-cutting/sync-inprocess` returns no rows in source.

---

### FINDING 15 (id: `CC-SYNC-015`)

- **Source:** `docs/audit_reports/findings/wave2-sync.md`
- **Severity:** High
- **Area:** cross-cutting-sync
- **Location:** `docs/specs/sync/overview.md:165-209` (OutboxEntry aggregate), `:286-330` (ConflictRecord), `:222-227` (SyncSubscription)

**Description:**

The sync spec defines four bookkeeping aggregates (`OutboxEntry`, `SyncCursor`, `ConflictRecord`, `SyncSubscription`) at `docs/specs/sync/overview.md:212-258`. **None** of these aggregates is implemented in code. `grep -rn "OutboxEntry\|SyncCursor\|ConflictRecord" crates/cross-cutting/sync crates/cross-cutting/sync-inprocess crates/cross-cutting/events` returns no rows (the storage-port `Outbox` sub-port is a different type — `crates/infra/storage/src/outbox.rs`, not the bookkeeping aggregate). The spec's "tables" section (`:1107-1133`) declares four storage tables (`local_outbox`, `sync_cursor`, `local_conflict_queue`, `sync_audit`); no migration emits them, and the SurrealDB adapter does not have these table definitions.

**Expected:**

`docs/specs/sync/overview.md:212-258` and `:1107-1133` — 4 aggregates + 4 tables implemented and emitted.

**Evidence:**

`grep -rn "OutboxEntry\|SyncCursor\|ConflictRecord\|local_outbox\|sync_cursor\|local_conflict_queue" crates/ migrations/` returns zero hits in any source or migration file. `migrations/engine/0000_engine_core.surreal.surql` does not contain these four tables.

---

### FINDING 16 (id: `CC-SYNC-016`)

- **Source:** `docs/audit_reports/findings/wave2-sync.md`
- **Severity:** High
- **Area:** cross-cutting-sync
- **Location:** `crates/cross-cutting/sync/src/port.rs:35-66` (whole trait)

**Description:**

The `SyncAdapter` trait surface has no `dispatch`, `subscribe`, or `snapshot` methods, so the wire-protocol port (`docs/ports/sync.md:48-58`) cannot be implemented against this trait. A consumer cannot drive an actual offline-first client (which needs to push outbox entries to the central store and subscribe to remote events) using the published trait. The published trait is session-control only (`start`/`pause`/`resume`/`stop`/`health`); the wire-protocol port doc promises a full bidirectional sync API that the trait cannot deliver.

**Expected:**

`docs/ports/sync.md:48-58` defines `dispatch`, `subscribe`, `snapshot`, `health` on `SyncAdapter`.

**Evidence:**

`crates/cross-cutting/sync/src/port.rs:37-66` defines exactly `start`, `pause`, `resume`, `stop`, `health`. No `dispatch`/`subscribe`/`snapshot` rows.

---

### FINDING 17 (id: `CC-SYNC-017`)

- **Source:** `docs/audit_reports/findings/wave2-sync.md`
- **Severity:** High
- **Area:** cross-cutting-sync
- **Location:** `crates/cross-cutting/sync-inprocess/src/lib.rs:60-77` (whole `InProcessSyncAdapter`)

**Description:**

`InProcessSyncAdapter` does **not** read from a local outbox and does **not** dispatch outbox events to consumers. ADR-018 § 3 (`educore-sync-inprocess` "drains the local outbox and applies remote snapshots without any network I/O") and `docs/build-plan.md:204-206` Phase 0 task 7 ("owns an in-process `EventBus` and dispatches every outbox event to a registered set of in-process consumers") both require outbox-driven fan-out. The actual adapter only listens for `SyncCommand::{Start,Pause,Resume,Stop}` and publishes the corresponding lifecycle event; it has no reference to an `Outbox` sub-port, no registered consumer set, no drain loop.

**Expected:**

`docs/build-plan.md:204-206` — "in-process `EventBus` and dispatches every outbox event to a registered set of in-process consumers".

**Evidence:**

`grep -n "outbox\|Outbox\|drain\|consumer\|subscriber" crates/cross-cutting/sync-inprocess/src/lib.rs` returns zero rows. The `InProcessSyncAdapter` struct (`:72-77`) holds only `bus: Arc<dyn EventBus>` and `state: Arc<Mutex<SyncHealth>>`; no outbox/consumer fields.

---

### FINDING 18 (id: `CC-SYNC-018`)

- **Source:** `docs/audit_reports/findings/wave2-sync.md`
- **Severity:** High
- **Area:** cross-cutting-sync
- **Location:** `docs/specs/sync/overview.md:830-895` (`SyncCoordinator` + `WorkerHttpSyncAdapter` services) and `:1041-1053` (wire protocol design)

**Description:**

The spec defines two services: the in-process `SyncCoordinator` (which "owns the per-(school, aggregate_type) subscription state" and runs push/pull loops) and `WorkerHttpSyncAdapter` (which is "purely a transport binding"). The implementation is `InProcessSyncAdapter`, a session-control stub with no push loop, no pull loop, no subscription state. There is no `SyncCoordinator` struct; the `WorkerHttpSyncAdapter` is not implemented; the wire-protocol HTTP client (`docs/ports/sync.md`) is not implemented. The "two deployments, same bookkeeping" claim is structurally violated.

**Expected:**

`docs/specs/sync/overview.md:830-895`.

**Evidence:**

`grep -rn "SyncCoordinator\|WorkerHttpSyncAdapter\|push_loop\|pull_loop" crates --include="*.rs"` returns no rows in source. `crates/cross-cutting/sync-inprocess/src/lib.rs:160-170` implements `start`/`pause`/`resume`/`stop`/`health` only.

---

### FINDING 19 (id: `CC-SYNC-019`)

- **Source:** `docs/audit_reports/findings/wave2-sync.md`
- **Severity:** High
- **Area:** cross-cutting-sync
- **Location:** `docs/specs/sync/overview.md:1107-1133` (tables)

**Description:**

The sync spec's "Tables" section lists four sync tables (`local_outbox`, `sync_cursor`, `local_conflict_queue`, `sync_audit`). The migration directory `migrations/engine/` ships one SurrealDB DDL file (`0000_engine_core.surreal.surql`, 50+ lines) which contains only the 6 cross-cutting engine tables (`outbox`, `audit_log`, `idempotency`, `event_log`, `schema_registry`, `system_user`); none of the four sync-specific tables are emitted. The runtime DDL emission flow at `docs/schemas/sql-dialects/README.md` therefore will not create the sync tables at startup. Even though `storage.create_schema()` is invoked by the consumer, the sync machinery has nowhere to persist cursors or outbox entries.

**Expected:**

`docs/specs/sync/overview.md:1107-1133` — 4 sync tables in `migrations/engine/0000_engine_core.surreal.surql` (or per-domain emitted by macro).

**Evidence:**

`grep -n "local_outbox\|sync_cursor\|local_conflict_queue\|sync_audit" migrations/engine/0000_engine_core.surreal.surql` returns no rows.

---

### FINDING 20 (id: `CC-SYNC-020`)

- **Source:** `docs/audit_reports/findings/wave2-sync.md`
- **Severity:** High
- **Area:** cross-cutting-sync
- **Location:** `docs/ports/sync.md:419-435` ("Configuration" section)

**Description:**

The port doc says `WorkerHttpSyncAdapter::builder()` is the production wiring path, with `SYNC_ENGINE_URL`, `DEVICE_TOKEN`, exponential backoff config, etc. The `WorkerHttpSyncAdapter` is **not implemented** in any crate (no `educore-sync-http` crate exists; `crates/adapters/` has no `sync-http/`). The build plan notes (`docs/build-plan.md:71` and `:205-210`) mark it "deferred to Phase 2" but `crates/educore/Cargo.toml` already depends on `educore-sync-inprocess` unconditionally (Finding 1), so the deferred state is not enforced.

**Expected:**

Either `educore-sync-http` is implemented and wired, or the umbrella does not promise `WorkerHttpSyncAdapter` is available.

**Evidence:**

`grep -rn "WorkerHttpSyncAdapter\|educore-sync-http" crates --include="*.rs" --include="*.toml"` returns no source-tree rows. `find crates -type d -name "sync-http"` returns no rows.

---

### FINDING 8 (id: `CC-SYNC-008`)

- **Source:** `docs/audit_reports/findings/wave2-sync.md`
- **Severity:** High
- **Area:** cross-cutting-sync
- **Location:** `docs/decisions/ADR-018-SyncEngineArchitecture.md:165-194` (ADR § 5 — `StorageAdapter` methods) vs `crates/infra/storage/src/port.rs:112-148` (actual port)

**Description:**

ADR-018 § 5 declares the four new `StorageAdapter` methods with parameterless signatures: `watch_changes(&self)`, `apply_snapshot(&self, snapshot)`, `cursor_for(&self)`, `advance_cursor(&self, cursor)`. The actual port signatures differ:
  - ADR: `watch_changes(&self)` → code: `async fn watch_changes(&self, filter: ChangeFilter)`.
  - ADR: `apply_snapshot(&self, snapshot: Snapshot)` → code: `async fn apply_snapshot(&self, snapshot: SchoolSnapshot)`.
  - ADR: `cursor_for(&self)` → code: `async fn cursor_for(&self, school_id: SchoolId)`.
  - ADR: `advance_cursor(&self, cursor: Cursor)` → code: `async fn advance_cursor(&self, school_id: SchoolId, to: VersionCursor)`.
  
  Three of the four are missing the `school_id` scoping the ADR omits; the first is missing the `ChangeFilter` argument. The spec body (`docs/specs/sync/overview.md:643-714`) gives yet a third set: `watch_changes(school_id, aggregate_type, from)`, `cursor_for(school_id, aggregate_type, aggregate_id)`, `advance_cursor(school_id, aggregate_type, aggregate_id, to, transaction)`. Three sources (ADR, port doc, spec body) describe three different APIs.

**Expected:**

`docs/decisions/ADR-018-SyncEngineArchitecture.md:165-194`.

**Evidence:**

`crates/infra/storage/src/port.rs:112-148`:
  ```rust
  async fn watch_changes(&self, filter: ChangeFilter) -> Result<ChangeStream> { ... }
  async fn apply_snapshot(&self, snapshot: SchoolSnapshot) -> Result<()> { ... }
  async fn cursor_for(&self, school_id: SchoolId) -> Result<VersionCursor> { ... }
  async fn advance_cursor(&self, school_id: SchoolId, to: VersionCursor) -> Result<()> { ... }
  ```

---

### FINDING 21 (id: `CC-SYNC-021`)

- **Source:** `docs/audit_reports/findings/wave2-sync.md`
- **Severity:** Medium
- **Area:** cross-cutting-sync
- **Location:** `crates/cross-cutting/sync/src/port.rs:60-66` (`stop` method)

**Description:**

The `SyncAdapter::stop` doc-comment (`:63-65`) says "Idempotent: stopping an already-stopped school is a no-op." But the implementation in `InProcessSyncAdapter::send_command` (`crates/cross-cutting/sync-inprocess/src/lib.rs:135-140`) **always** transitions `state.status = SyncStatus::Stopped` and **always** emits a `SyncStopped` event, even when the school was already stopped. The "no-op" promise is not honored; duplicate `Stop` calls produce duplicate events. No idempotency guard.

**Expected:**

`crates/cross-cutting/sync/src/port.rs:63-65` — "Idempotent: stopping an already-stopped school is a no-op."

**Evidence:**

`crates/cross-cutting/sync-inprocess/src/lib.rs:124-150` — `SyncCommand::Stop(s)` unconditionally sets `status = Stopped` and unconditionally publishes `SyncStopped::now(school)`.

---

### FINDING 22 (id: `CC-SYNC-022`)

- **Source:** `docs/audit_reports/findings/wave2-sync.md`
- **Severity:** Medium
- **Area:** cross-cutting-sync
- **Location:** `crates/cross-cutting/sync/src/health.rs:23-35` (`SyncStatus` enum)

**Description:**

The `SyncStatus` enum has 3 variants (`Running`, `Paused`, `Stopped`); the spec body (`docs/specs/sync/overview.md:245-258`) mandates 5 per-subscription states (`Idle`, `Streaming`, `Backoff`, `Paused`, `Stalled`). The implementation collapses subscription state to adapter-level. Even at adapter-level, the ADR § 5 design implies 4 states (Started/Paused/Resumed/Stopped) but the code emits `Running` on both `Start` and `Resume` (`:115-129`), making the `Running` state ambiguous (does it mean "just started" or "resumed"?).

**Expected:**

`docs/specs/sync/overview.md:245-258` — 5 per-subscription states; or, at minimum, separate `Started` and `Running` adapter states.

**Evidence:**

`crates/cross-cutting/sync/src/health.rs:23-35`:
  ```rust
  pub enum SyncStatus {
      Running,
      Paused,
      Stopped,
  }
  ```
  `crates/cross-cutting/sync-inprocess/src/lib.rs:108-115` and `:127-129` both write `state.status = SyncStatus::Running` for `Start` and `Resume`.

---

### FINDING 23 (id: `CC-SYNC-023`)

- **Source:** `docs/audit_reports/findings/wave2-sync.md`
- **Severity:** Medium
- **Area:** cross-cutting-sync
- **Location:** `crates/cross-cutting/events/src/sync.rs:64-94` (`SyncStarted` struct)

**Description:**

`SyncStarted` carries only `event_id` / `school_id` / `at`. The spec body mandates additional fields: `subscription_id: SyncSubscriptionId`, `aggregate_type: AggregateType`, `from_version: VersionCursor`, `request_id: Uuid` (`docs/specs/sync/overview.md:355-368`). The minimal struct cannot express the per-subscription identity or the cursor from which the subscription started, so downstream consumers (e.g. audit, UI) cannot correlate a `SyncStarted` event with the originating subscription.

**Expected:**

`docs/specs/sync/overview.md:358-368` — full payload.

**Evidence:**

`crates/cross-cutting/events/src/sync.rs:64-76` — `pub struct SyncStarted { pub event_id: Uuid; pub school_id: SchoolId; pub at: Timestamp; }`. No `subscription_id`, no `aggregate_type`, no `from_version`, no `request_id`.

---

### FINDING 24 (id: `CC-SYNC-024`)

- **Source:** `docs/audit_reports/findings/wave2-sync.md`
- **Severity:** Medium
- **Area:** cross-cutting-sync
- **Location:** `crates/cross-cutting/events/src/sync.rs:122-225` (`SyncPaused`/`SyncResumed`/`SyncStopped`)

**Description:**

`SyncPaused`/`SyncResumed`/`SyncStopped` carry an `Option<Uuid>` `session_started_event_id`, but the implementation in `crates/cross-cutting/sync-inprocess/src/lib.rs:131-150` always mints the events via `SyncPaused::now(school)` / `SyncResumed::now(school)` / `SyncStopped::now(school)` — the `for_session` correlator constructors (which would set `session_started_event_id`) are never invoked. The correlation field is therefore always `None` at runtime. The `into_envelope` in the adapter also drops the `event_id` from the typed event (the adapter creates a fresh envelope instead).

**Expected:**

`docs/specs/sync/overview.md:381-396, 415-429, 451-465` — events carrying `subscription_id`, `aggregate_type`, and `session_started_event_id` (correlated).

**Evidence:**

`crates/cross-cutting/sync-inprocess/src/lib.rs:131-150`:
  ```rust
  SyncCommand::Pause(_) => SyncPaused::now(school).into_envelope(&ctx),
  SyncCommand::Resume(_) => SyncResumed::now(school).into_envelope(&ctx),
  SyncCommand::Stop(_) => SyncStopped::now(school).into_envelope(&ctx),
  ```
  `SyncPaused::now(school)` constructs `session_started_event_id: None`.

---

### FINDING 25 (id: `CC-SYNC-025`)

- **Source:** `docs/audit_reports/findings/wave2-sync.md`
- **Severity:** Medium
- **Area:** cross-cutting-sync
- **Location:** `docs/specs/sync/overview.md:981-1001` (permissions table) and `crates/cross-cutting/sync/src/port.rs` (whole file)

**Description:**

The spec's permissions table (`docs/specs/sync/overview.md:981-1001`) lists 6 sync capabilities: `Sync.Request`, `Sync.Pause`, `Sync.Resume`, `Sync.ResolveConflict`, `Sync.SwitchSchool`, `Sync.CompactOutbox`. The `educore-sync` crate depends on `educore-core` only (no `educore-rbac` dependency, `crates/cross-cutting/sync/Cargo.toml:13-23`); the port trait (`crates/cross-cutting/sync/src/port.rs`) takes only `SchoolId`, no `actor_id`, no capability check. `grep -rn "Sync\\.Request\|Sync\\.Pause\|Sync\\.ResolveConflict" crates/cross-cutting/sync crates/cross-cutting/sync-inprocess` returns no rows. No sync command performs RBAC.

**Expected:**

`docs/specs/sync/overview.md:981-1001` — capability-gated commands.

**Evidence:**

`crates/cross-cutting/sync/Cargo.toml:13-23` lists `educore-core`, `educore-events`, `async-trait`, `bytes`, `serde`, `serde_json`, `tracing`, `uuid`, `futures`. No `educore-rbac` row. `crates/cross-cutting/sync/src/port.rs:35-66` has no actor parameter.

---

### FINDING 26 (id: `CC-SYNC-026`)

- **Source:** `docs/audit_reports/findings/wave2-sync.md`
- **Severity:** Medium
- **Area:** cross-cutting-sync
- **Location:** `docs/ports/sync.md:560-585` (audit events) and `crates/cross-cutting/sync-inprocess/src/lib.rs` (entire)

**Description:**

The wire-protocol port doc promises 7 audit events: `SyncDispatched`, `SyncConflictSurfaced`, `SyncConflictResolved`, `SyncSnapshotTaken`, `SyncSubscribed`, `SyncHealthFailed`, `SyncReconnected` (lines 560-585). The `educore-sync` and `educore-sync-inprocess` crates emit **no** audit events at all — there is no `AuditSink` port injection, no `audit_log` call. The in-process adapter's only side effect on a state transition is the typed lifecycle event publish. The Phase 2 audit cross-cutting crate (`educore-audit`) is **not** a dependency of either sync crate.

**Expected:**

`docs/ports/sync.md:560-585` — 7 audit events written to `audit_log`.

**Evidence:**

`grep -n "AuditSink\|audit_log\|SyncDispatched\|SyncConflictSurfaced\|educore-audit" crates/cross-cutting/sync/Cargo.toml crates/cross-cutting/sync-inprocess/Cargo.toml crates/cross-cutting/sync/src/*.rs crates/cross-cutting/sync-inprocess/src/*.rs` returns no rows.

---

### FINDING 27 (id: `CC-SYNC-027`)

- **Source:** `docs/audit_reports/findings/wave2-sync.md`
- **Severity:** Low
- **Area:** cross-cutting-sync
- **Location:** `docs/specs/sync/overview.md:1107` and `docs/build-plan.md:194`

**Description:**

Minor terminology drift between sources: the spec body calls the in-process service `SyncCoordinator` (`docs/specs/sync/overview.md:830`), the build plan calls it `SyncCoordinator` (`:194`) but the implementation class is `InProcessSyncAdapter`. The handoff (`docs/handoff/PHASE-0-HANDOFF.md:30`) calls it "the in-process coordinator"; the ADR-018 § 3 calls it `EducoreSyncAdapter::in_process()` — **none** of these names match the actual type `InProcessSyncAdapter`. The umbrella crate's exported alias is `sync_inprocess` (`:55`). Four different names (`SyncCoordinator`, `InProcessSyncAdapter`, `EducoreSyncAdapter`, `sync_inprocess`) for one struct.

**Expected:**

A single canonical name used consistently across spec, build plan, ADR, code, handoff, and umbrella.

**Evidence:**

`docs/specs/sync/overview.md:830` `### SyncCoordinator (in-process reference)`; `docs/build-plan.md:194` `SyncCoordinator trait`; `docs/decisions/ADR-018-SyncEngineArchitecture.md:93` `EducoreSyncAdapter::in_process()`; `crates/educore/src/lib.rs:55` `pub use educore_sync_inprocess as sync_inprocess;`; `crates/cross-cutting/sync-inprocess/src/lib.rs:72` `pub struct InProcessSyncAdapter`.

---

