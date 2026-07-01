# Audit findings: educore-platform (Phase 2)

**Scope:** `crates/cross-cutting/platform/` (10 source files +
1 integration test file), `docs/specs/platform/` (11 spec
files), `docs/commands/platform.md`, `docs/events/platform.md`,
`docs/coverage.toml` (3 platform rows), `docs/handoff/PHASE-2-HANDOFF.md`.

**Total findings:** 48

---

### FINDING 1

- **id:** CROSSCUT-PLAT-001
- **area:** cross-cutting
- **severity:** Critical
- **location:** crates/cross-cutting/platform/src/aggregate.rs:46-91
- **description:** The `School` aggregate is missing four fields
  mandated by `docs/specs/platform/aggregates.md` invariants 4,
  6, 7, 9, 10: `email`, `is_enabled`, `plan_type`,
  `starting_date`, `ending_date`, and `region`. The struct
  cannot satisfy invariants 4 (`School::email is RFC-valid`),
  6 (`is_enabled is Yes or No`), 7 (`plan_type carries the
  package's billing mode`), 9 (`starting_date <= ending_date`),
  or 10 (`region references a known continent/country id`).
- **expected:** `docs/specs/platform/aggregates.md` lines 36-46
  list 10 invariants for `School`; lines 17-34 of the spec's
  `CreateSchoolCommand` carry `email`, `phone`, `address`,
  `starting_date`, `ending_date`, `plan_type`, `contact_type`,
  `region` — all of which need a slot on the aggregate.
- **evidence:** `crates/cross-cutting/platform/src/aggregate.rs:46-91` —
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

### FINDING 2

- **id:** CROSSCUT-PLAT-002
- **area:** cross-cutting
- **severity:** Critical
- **location:** crates/cross-cutting/platform/src/aggregate.rs:142-194
- **description:** The `User` aggregate is missing four fields
  mandated by `docs/specs/platform/aggregates.md` invariants 7,
  9, 10 and the Purpose statement (line 84-86) covering
  authentication material: `is_administrator`, `language`,
  `is_registered`, `random_code`, `notification_token`,
  `device_token`, `remember_token`. The struct can carry the
  `role_ids` binding but no `role_id` for the primary
  single-role invariant (invariant 8 of the spec).
- **expected:** `docs/specs/platform/aggregates.md` lines 94-110
  specify 11 invariants for `User`; lines 84-86 state "Holds
  the user's identity, contact information, status, language
  preference, role binding, and authentication-related fields
  (random code, notification token, remember token, OTP)."
- **evidence:** `crates/cross-cutting/platform/src/aggregate.rs:142-194` —
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

### FINDING 3

- **id:** CROSSCUT-PLAT-003
- **area:** cross-cutting
- **severity:** High
- **location:** crates/cross-cutting/platform/src/value_objects.rs:312-327
- **description:** The `SchoolStatus` enum exposes four
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
- **expected:** `docs/specs/platform/value-objects.md:69-71` —
  `SchoolActiveStatus | Approved, Pending`.
- **evidence:** `crates/cross-cutting/platform/src/value_objects.rs:312-327` —
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

### FINDING 4

- **id:** CROSSCUT-PLAT-004
- **area:** cross-cutting
- **severity:** Critical
- **location:** crates/cross-cutting/platform/src/aggregate.rs:46-91
- **description:** `School::active_status` is typed
  `ActiveStatus` (a unified engine type with variants like
  `Active` / `Retired`) rather than the spec's `SchoolActiveStatus`
  (`Approved` | `Pending`). Spec invariant 5 (`School::active_status
  is Approved or Pending`) is therefore not expressible by the
  type system, and spec invariant 8 (bootstrap school cannot be
  deleted) has no guard.
- **expected:** `docs/specs/platform/aggregates.md:40-43` —
  `5. A School::active_status is Approved or Pending.`;
  `8. The bootstrap School (id 1) cannot be deleted.`
- **evidence:** `crates/cross-cutting/platform/src/aggregate.rs:46-91` —
  `pub active_status: ActiveStatus,` (engine-wide type, not the
  domain-specific `SchoolActiveStatus` value object). No
  `SchoolActiveStatus` type is defined anywhere in
  `crates/cross-cutting/platform/src/value_objects.rs`
  (`grep SchoolActiveStatus value_objects.rs` returns no rows).

---

### FINDING 5

- **id:** CROSSCUT-PLAT-005
- **area:** cross-cutting
- **severity:** Critical
- **location:** crates/cross-cutting/platform/src/aggregate.rs:46-91
- **description:** Spec invariant 8 mandates "The bootstrap
  School (id 1) cannot be deleted." The `School` struct has
  no protection: `DeactivateSchoolCommand` does not check the
  id against the bootstrap id, and `deactivate_school` in
  `services.rs` will retire the bootstrap school unconditionally.
- **expected:** `docs/specs/platform/aggregates.md:43` —
  `8. The bootstrap School (id 1) cannot be deleted.`
- **evidence:** `crates/cross-cutting/platform/src/services.rs:225-260` —
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

### FINDING 6

- **id:** CROSSCUT-PLAT-006
- **area:** cross-cutting
- **severity:** Critical
- **location:** crates/cross-cutting/platform/src/aggregate.rs:46-242
- **description:** Spec invariant 4 of the School requires
  email uniqueness; the School struct has no email field, so
  no validation. Spec invariant 7 (User `is_administrator`)
  and invariant 10 (User `is_registered`) likewise have no
  fields. Invariant 8 of User ("role_id references a valid
  role") is satisfied by `role_ids: Vec<RoleId>` only in the
  case of multi-role; the spec's `User::role_id` (singular)
  cannot be expressed by the `Vec<RoleId>` shape.
- **expected:** `docs/specs/platform/aggregates.md:39, 104-108`:
  `4. A School::email is RFC-valid and unique.`;
  `7. A User::is_administrator is Yes or No.`;
  `8. A User::role_id references a valid role in the same school.`;
  `10. A User::is_registered is a boolean indicating whether the user has completed self-registration.`
- **evidence:** `crates/cross-cutting/platform/src/aggregate.rs:46-91` and
  `:142-194` — see fields listed in findings 1 and 2. No
  `IsAdministrator`, `IsRegistered`, or `SchoolEmail` value
  object is defined in `value_objects.rs`.

---

### FINDING 7

- **id:** CROSSCUT-PLAT-007
- **area:** cross-cutting
- **severity:** Critical
- **location:** crates/cross-cutting/platform/src/aggregate.rs (full file)
- **description:** Only 2 of the 37 aggregates documented in
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
- **expected:** `docs/specs/platform/aggregates.md` defines 37
  aggregate sections (line count: `grep -c "^## " aggregates.md`
  returns 37).
- **evidence:** `crates/cross-cutting/platform/src/aggregate.rs:46-242`
  defines exactly two aggregates (`School`, `User`). The
  crate's `README.md:7-10` explicitly documents this: "The
  remaining 30 secondary platform aggregates enumerated in
  `docs/specs/platform/aggregates.md` (Course, OtpCode, Module,
  Plugin, ...) are out of scope for Phase 2 and land in later
  phases alongside their owning events." (The number is wrong:
  there are 35 secondary aggregates, not 30.)

---

### FINDING 8

- **id:** CROSSCUT-PLAT-008
- **area:** cross-cutting
- **severity:** Critical
- **location:** crates/cross-cutting/platform/src/commands.rs:59-243
- **description:** Only 6 of the ~117 commands documented in
  `docs/specs/platform/commands.md` are implemented. The
  missing commands include all commands for 35 secondary
  aggregates plus 5 additional commands for `School` and
  `User`: `ApproveSchool`, `DisableLogin`, `EnableLogin`,
  `ReactivateUser`, `ChangeUserRole`, `VerifyEmail`,
  `ResetPassword`. (Note: `DisableLogin` / `EnableLogin` is
  one section with two commands per spec.)
- **expected:** `docs/specs/platform/commands.md` lists
  `ApproveSchool` (line 78-86), `DisableLogin` /
  `EnableLogin` (line 92-105), `ReactivateUser` (line 173-180),
  `ChangeUserRole` (line 185-193), `VerifyEmail` (line 199-207),
  `ResetPassword` (line 213-220) — all part of the
  Phase 2-implemented aggregates (School, User).
- **evidence:** `crates/cross-cutting/platform/src/commands.rs`
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

### FINDING 9

- **id:** CROSSCUT-PLAT-009
- **area:** cross-cutting
- **severity:** Critical
- **location:** crates/cross-cutting/platform/src/events.rs:1-474
- **description:** Only 6 of the ~73 events documented in
  `docs/specs/platform/events.md` are implemented. Missing
  events include all events for the 35 secondary aggregates
  plus 5 additional events for `School` and `User`:
  `SchoolApproved`, `LoginDisabled`, `LoginEnabled`,
  `UserReactivated`, `UserRoleChanged`, `EmailVerified`,
  `PasswordReset`.
- **expected:** `docs/specs/platform/events.md` lines 73-97
  (`SchoolApproved`, `LoginDisabled`, `LoginEnabled`),
  lines 140-177 (`UserReactivated`, `UserRoleChanged`,
  `EmailVerified`, `PasswordReset`) — all required by the
  Phase 2-implemented aggregates.
- **evidence:** `crates/cross-cutting/platform/src/events.rs`
  defines exactly 6 events:
  `SchoolCreated`, `SchoolUpdated`, `SchoolDeactivated`,
  `UserRegistered`, `UserUpdated`, `UserDeactivated`. No
  `SchoolApproved`, `LoginDisabled`, `LoginEnabled`,
  `UserReactivated`, `UserRoleChanged`, `EmailVerified`,
  `PasswordReset`, `OtpIssued`, `CourseCreated`, etc. is
  defined (`grep -c "^pub struct" events.rs` returns 6
  matching `*Event*` names).

---

### FINDING 10

- **id:** CROSSCUT-PLAT-010
- **area:** cross-cutting
- **severity:** Critical
- **location:** crates/cross-cutting/platform/src/events.rs (all 6 events)
- **description:** None of the 6 implemented events have
  subscribers wired in (RBAC, settings, academic, operations,
  communication, cms). The events spec lists subscribers per
  event; spec drift between code and docs is severe.
- **expected:** `docs/specs/platform/events.md:50-53` —
  `SchoolCreated` subscribers `rbac`, `settings`, `academic`;
  line 115-117 — `UserRegistered` subscriber `rbac`;
  line 137-138 — `UserDeactivated` subscribers `rbac`,
  `operations`.
- **evidence:** `crates/cross-cutting/platform/src/events.rs`
  contains only struct definitions and `DomainEvent` trait
  impls (`grep -rn "fn subscriber\|impl Subscriber" events.rs`
  returns no rows). There is no subscriber module at all.

---

### FINDING 11

- **id:** CROSSCUT-PLAT-011
- **area:** cross-cutting
- **severity:** Critical
- **location:** crates/cross-cutting/platform/src/value_objects.rs (full file)
- **description:** Of the ~100 value objects documented in
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
- **expected:** `docs/specs/platform/value-objects.md` lines
  11-279 enumerate all value objects.
- **evidence:** `crates/cross-cutting/platform/src/value_objects.rs`
  defines only 7 (`grep -E "^pub (struct|enum)" value_objects.rs`
  returns `EmailAddress`, `PhoneNumber`, `HashedPassword`,
  `SchoolStatus`, `UserStatus`, `PackageId`, `RoleId`).

---

### FINDING 12

- **id:** CROSSCUT-PLAT-012
- **area:** cross-cutting
- **severity:** Critical
- **location:** crates/cross-cutting/platform/src/services.rs (full file)
- **description:** The spec defines 17 `XService` structs
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
- **expected:** `docs/specs/platform/services.md` lines 8-258
  define 17 service structs plus 2 policies plus 2
  specifications.
- **evidence:** `crates/cross-cutting/platform/src/services.rs`
  contains only 6 free functions (`create_school`,
  `update_school`, `deactivate_school`, `register_user`,
  `update_user`, `deactivate_user`). `grep -nE "^pub struct" services.rs`
  returns no rows. The service contract from the spec is not
  satisfied.

---

### FINDING 13

- **id:** CROSSCUT-PLAT-013
- **area:** cross-cutting
- **severity:** Critical
- **location:** crates/cross-cutting/platform/src/repository.rs (full file)
- **description:** The spec defines 28 repository port
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
- **expected:** `docs/specs/platform/repositories.md` lines
  7-296 define 28 repository port traits (line 279 lists 16
  additional traits following the standard CRUD pattern).
- **evidence:** `crates/cross-cutting/platform/src/repository.rs`
  defines only 2 traits (`grep -nE "^pub trait" repository.rs`
  returns `SchoolRepository` line 32 and `UserRepository`
  line 71).

---

### FINDING 14

- **id:** CROSSCUT-PLAT-014
- **area:** cross-cutting
- **severity:** High
- **location:** crates/cross-cutting/platform/src/repository.rs:36-66
- **description:** `SchoolRepository::get_by_domain` takes
  `&str`, not the spec's typed `Domain` value object. Same for
  `get_by_code` (`&str` instead of `SchoolCode`). The
  UserRepository's `get_by_email`, `get_by_username`, and
  `get_by_phone` take `&str` rather than the spec's typed
  `EmailAddress`, `Username`, `PhoneNumber` value objects.
  Spec drift from typed wrappers to strings.
- **expected:** `docs/specs/platform/repositories.md:13-14`:
  `async fn get_by_domain(&self, domain: &Domain) -> Result<Option<School>>;`
  `async fn get_by_code(&self, code: &SchoolCode) -> Result<Option<School>>;`.
  Lines 29-31: `get_by_email` takes `&EmailAddress`;
  `get_by_username` takes `&Username`; `get_by_phone` takes
  `&PhoneNumber`.
- **evidence:** `crates/cross-cutting/platform/src/repository.rs:40-44`:
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

### FINDING 15

- **id:** CROSSCUT-PLAT-015
- **area:** cross-cutting
- **severity:** High
- **location:** crates/cross-cutting/platform/src/repository.rs:71-110
- **description:** `UserRepository` is missing the
  `query(UserQuery)`, `count(UserQuery)`, and
  `page(UserQuery, offset, limit)` methods from the spec.
  These are the typed-query plumbing the storage adapter needs.
- **expected:** `docs/specs/platform/repositories.md:37-39`:
  ```rust
  async fn query(&self, q: UserQuery) -> Result<Vec<User>>;
  async fn count(&self, q: UserQuery) -> Result<u64>;
  async fn page(&self, q: UserQuery, offset: u32, limit: u32) -> Result<Page<User>>;
  ```
- **evidence:** `crates/cross-cutting/platform/src/repository.rs:71-110`
  — the trait body has 8 methods (`get`, `get_by_email`,
  `get_by_username`, `get_by_phone`, `list`, `list_by_role`,
  `list_by_usertype`, `insert`, `update`). No `query`,
  `count`, or `page` method exists.

---

### FINDING 16

- **id:** CROSSCUT-PLAT-016
- **area:** cross-cutting
- **severity:** High
- **location:** crates/cross-cutting/platform/src/permissions.md (none — module absent)
- **description:** `docs/specs/platform/permissions.md` defines
  ~115 capability strings grouped by aggregate. There is no
  Rust enum, no capability constant module, no
  `CapabilityCheck` trait, and no capability helper code in
  the platform crate. The platform crate has 6 commands whose
  capability checks are documented but not enforced in code
  (e.g. `Platform.School.Create`, `Platform.User.Register`).
- **expected:** `docs/specs/platform/permissions.md:18-229` —
  per-aggregate `Platform.<Aggregate>.<Action>` capability
  strings; line 251-255 example:
  `engine.rbac().has(actor_id, Capability::PlatformUserRegister).await?`.
- **evidence:** `grep -rn "Capability" crates/cross-cutting/platform/src/`
  returns no `pub enum Capability`, no `Capability::Platform*`,
  no `pub const PLATFORM_*_CAPABILITY` rows. The platform crate
  depends on `educore-rbac` for capability checks per the
  cross-cutting tier (no such dependency exists in
  `Cargo.toml:13-20`).

---

### FINDING 17

- **id:** CROSSCUT-PLAT-017
- **area:** cross-cutting
- **severity:** High
- **location:** crates/cross-cutting/platform/src/commands.rs:246-326
- **description:** The 5 validation helper functions
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
- **expected:** `docs/specs/platform/aggregates.md:36-46`
  invariants apply to the `School` aggregate.
- **evidence:** `crates/cross-cutting/platform/src/commands.rs:247-326` —
  5 `pub(crate)` validators that return `educore_core::error::Result`;
  the only caller is `services.rs`. The `School::fresh`
  constructor (`aggregate.rs:104-133`) does no validation;
  `User::fresh` (`:204-242`) does no validation.

---

### FINDING 18

- **id:** CROSSCUT-PLAT-018
- **area:** cross-cutting
- **severity:** High
- **location:** crates/cross-cutting/platform/src/commands.rs (and modules)
- **description:** `docs/specs/platform/commands.md` has no
  `pub fn` validation helpers (they are not part of the
  documented command shapes), but the Rust code defines 5
  `pub(crate)` validators in `commands.rs`. The pub-crate
  scope is fine; the issue is that the helpers are
  inconsistently named (`validate_reason` is generic; others
  are aggregate-specific) and there is no test for
  `validate_school_name` rejecting names with > 200 chars
  (although the test `validate_username_rejects_overlong`
  covers the equivalent for username).
- **expected:** Spec consistency between command shapes and
  validator functions.
- **evidence:** `crates/cross-cutting/platform/src/commands.rs:247-326`:
  5 validators; `commands.rs:382-398` defines 5 tests but the
  test `validate_school_name` is missing.

---

### FINDING 19

- **id:** CROSSCUT-PLAT-019
- **area:** cross-cutting
- **severity:** Critical
- **location:** crates/cross-cutting/platform/src/services.rs:189-198
- **description:** `update_school` computes `bytes` and
  `s_id` but never uses them; the event_id is minted via
  `Uuid::now_v7()` instead. Dead code in a non-test function.
  The surrounding comment is misleading ("the bus port stamps
  its own event id at publish time, so this is informational
  only") but the bytes copy is never observed by anything.
- **expected:** No dead code in production paths.
- **evidence:** `crates/cross-cutting/platform/src/services.rs:193-198`:
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

### FINDING 20

- **id:** CROSSCUT-PLAT-020
- **area:** cross-cutting
- **severity:** Medium
- **location:** crates/cross-cutting/platform/src/services.rs:152, 241, 380, 483
- **description:** `let _ = ctx;` appears in 4 places in
  `services.rs`. The `ctx` parameter is intended to drive
  `school.updated_by = ctx.actor_id` and `event.correlation_id
  = ctx.correlation_id` (lines 185, 247, 421, 489, 498), so
  the `let _ = ctx;` is dead. The unused discard suggests the
  function signature accepts `ctx` for symmetry but the
  compiler-lint bypass is suspect.
- **expected:** No `let _ = ctx;` discards in production
  paths.
- **evidence:** `crates/cross-cutting/platform/src/services.rs:152`
  (in `update_school`), `:241` (in `deactivate_school`), `:380`
  (in `update_user`), `:483` (in `deactivate_user`).

---

### FINDING 21

- **id:** CROSSCUT-PLAT-021
- **area:** cross-cutting
- **severity:** Medium
- **location:** crates/cross-cutting/platform/src/services.rs:386-388, 429
- **description:** `update_user` captures `email_at_call`,
  `display_name_at_call`, `phone_at_call` (lines 386-388) but
  never reads them; the `let _ = (email_at_call, ...)` at
  line 429 discards them. The comments (lines 384-385) say
  "Snapshot the pre-mutation values so the event can carry
  'what changed' without aliasing the post-mutation state",
  but the event construction (lines 430-452) does not read
  the snapshot; it only reads post-mutation state.
- **expected:** No dead captures.
- **evidence:** `crates/cross-cutting/platform/src/services.rs:384-429`:
  ```rust
  let email_at_call = user.email.clone();
  let display_name_at_call = user.display_name.clone();
  let phone_at_call = user.phone_number.clone();
  ...
  let _ = (email_at_call, display_name_at_call, phone_at_call);
  ```

---

### FINDING 22

- **id:** CROSSCUT-PLAT-022
- **area:** cross-cutting
- **severity:** High
- **location:** crates/cross-cutting/platform/src/services.rs:225-260
- **description:** `deactivate_school` accepts
  `DeactivateSchoolCommand::new_status` (any `SchoolStatus`)
  and assigns it to `school.status`. The spec invariant 5
  restricts `School::active_status` to `Approved | Pending`
  only; the service code allows `Suspended` and `Active`
  assignments from any caller, with no validation.
- **expected:** Spec invariant 5 of School limits status to
  two values.
- **evidence:** `crates/cross-cutting/platform/src/services.rs:225-260`:
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

### FINDING 23

- **id:** CROSSCUT-PLAT-023
- **area:** cross-cutting
- **severity:** High
- **location:** crates/cross-cutting/platform/src/entities.rs:36-138
- **description:** Only 4 of the 35 entities documented in
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
- **expected:** `docs/specs/platform/entities.md` defines 35
  entity sections.
- **evidence:** `crates/cross-cutting/platform/src/entities.rs:36-138`
  defines only 4 entities.

---

### FINDING 24

- **id:** CROSSCUT-PLAT-024
- **area:** cross-cutting
- **severity:** Medium
- **location:** crates/cross-cutting/platform/src/entities.rs:93-114
- **description:** `UserPreference::value_json: String` is
  used to store a "typed JSON blob" (line 105-110 comment).
  This is `serde_json::Value`-shaped data represented as a
  string; the engine's value type is JSON-shaped (per
  comment). This is borderline `serde_json::Value`-as-string
  drift and depends on the storage adapter to parse.
- **expected:** Typed wrapper or `serde_json::Value` import
  explicitly (with `serde_json` already in `Cargo.toml:19`).
- **evidence:** `crates/cross-cutting/platform/src/entities.rs:93-114`:
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

### FINDING 25

- **id:** CROSSCUT-PLAT-025
- **area:** cross-cutting
- **severity:** Critical
- **location:** crates/cross-cutting/platform/src/entities.rs (full file)
- **description:** `UserSession` lacks a typed session-id
  shape — the spec mandates a hashed session id, but the
  struct carries `session_id: SessionId` (a `Uuid` wrapper).
  Spec invariants 4 (`token is unique and stored as a
  SHA-256 hash`) apply to `PersonalAccessToken` and similar
  properties are expected for sessions (per `entities.md:38`:
  "the opaque session id (hashed; the plaintext is never
  stored)"). The engine's `SessionId` is a `Uuid` newtype and
  is not a hash.
- **expected:** `docs/specs/platform/entities.md:38-42`
  describes hashed session ids.
- **evidence:** `crates/cross-cutting/platform/src/entities.rs:62-84`:
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

### FINDING 26

- **id:** CROSSCUT-PLAT-026
- **area:** cross-cutting
- **severity:** Critical
- **location:** crates/cross-cutting/platform/src/tables.md (no impl)
- **description:** Of the 43 tables listed in
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
- **expected:** `docs/specs/platform/tables.md` lines 7-48
  enumerate 43 tables (cross-cutting and self-reference rows
  included; the spec lists 43 distinct table rows).
- **evidence:** The `aggregate.rs` file has only 2 structs;
  no `#[derive(DomainQuery)]` attribute is present in the
  platform crate (`grep -rn "derive(DomainQuery)" crates/cross-cutting/platform/`
  returns no rows). No typed entities back the 41
  non-School/User tables.

---

### FINDING 27

- **id:** CROSSCUT-PLAT-027
- **area:** cross-cutting
- **severity:** High
- **location:** crates/cross-cutting/platform/src/lib.rs:20-21
- **description:** `lib.rs` has `#![forbid(unsafe_code)]` and
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
- **expected:** Production paths (non-`#[cfg(test)]`) have
  no `unwrap`/`expect`/`panic`/`dbg!`; the `#[cfg(test)]`
  block is exempt.
- **evidence:** `crates/cross-cutting/platform/src/lib.rs:20-21`,
  `aggregate.rs:244-251`, `commands.rs:328-334`,
  `events.rs:475-481`, `value_objects.rs:460-466`,
  `services.rs:505-512`, `query.rs:140-146`,
  `entities.rs:139-145`, `repository.rs:112-118`. Audit
  confirmation: every `unwrap()` / `expect()` in the crate
  lives inside a `#[cfg(test)]` block. No production
  `unwrap()` or `expect()` was found.

---

### FINDING 28

- **id:** CROSSCUT-PLAT-028
- **area:** cross-cutting
- **severity:** High
- **location:** crates/cross-cutting/platform/src/query.rs (full file)
- **description:** `SchoolQuery::execute` (line 74-79) and
  `UserQuery::execute` (line 132-137) return
  `Err(DomainError::NotSupported)` with the explanation
  "Phase 2 stub". The query layer is incomplete: there is
  no `#[derive(DomainQuery)]` macro usage, no AST emission,
  no SQL translation. The crate depends on the deferred
  Phase 3+ implementation.
- **expected:** `docs/specs/platform/repositories.md:37-39`
  specifies typed `query`, `count`, `page` methods on
  `UserRepository` that take `UserQuery`.
- **evidence:** `crates/cross-cutting/platform/src/query.rs:74-79, 132-137`:
  ```rust
  pub async fn execute(self, _school: SchoolId) -> Result<Vec<School>> {
      let _ = self;
      Err(DomainError::not_supported(
          "SchoolQuery::execute is a Phase 2 stub; the typed query executor lands in Phase 3+",
      ))
  }
  ```

---

### FINDING 29

- **id:** CROSSCUT-PLAT-029
- **area:** cross-cutting
- **severity:** Medium
- **location:** crates/cross-cutting/platform/src/query.rs (full file)
- **description:** The `SchoolQuery` and `UserQuery` structs
  are untyped string/option bags — they have no
  `#[derive(DomainQuery)]` annotation, no field-marker
  macro, no `SchoolQueryField` / `UserQueryField` enums for
  compile-time field access. The query layer is hand-rolled
  rather than macro-generated, violating the engine's
  "compile-time safety over strings" rule.
- **expected:** `docs/project-overview.md` and `AGENTS.md`
  rule 2: "Use macro-generated enums (`StudentField::Status`) —
  never string field names."
- **evidence:** `crates/cross-cutting/platform/src/query.rs:27-95`:
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

### FINDING 30

- **id:** CROSSCUT-PLAT-030
- **area:** cross-cutting
- **severity:** High
- **location:** crates/cross-cutting/platform/tests/platform_e2e.rs (full file)
- **description:** The integration test file has 10 test
  functions but none exercise storage parity. The
  `InMemoryUniqueness` test fixture is hand-rolled inside
  the test file (lines 60-115). The crate depends on
  `educore-storage` for the storage port but has no
  storage-parity test (e.g. against an in-memory SQLite
  adapter). The README, handoff, and coverage matrix all
  claim the platform crate has been tested end-to-end.
- **expected:** `AGENTS.md` "Test infrastructure + SDK"
  phase 16 deliverable: storage-parity tests.
- **evidence:** `crates/cross-cutting/platform/tests/platform_e2e.rs:38`:
  `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`
  No `tokio::test` against a real storage adapter; no
  `educore_storage_sqlite::SqliteStorage` import. `grep -n
  "storage\|Storage" tests/platform_e2e.rs` returns only doc
  comments.

---

### FINDING 31

- **id:** CROSSCUT-PLAT-031
- **area:** cross-cutting
- **severity:** Medium
- **location:** crates/cross-cutting/platform/tests/platform_e2e.rs (full file)
- **description:** The integration test file (10 tests) does
  not exercise the failure path of `update_school` when the
  supplied domain conflicts. The spec's `UpdateSchool`
  effect ("Emits `SchoolUpdated`") requires the
  domain-uniqueness guard, but no test asserts
  `update_school` returns `Err(Conflict)` on a duplicate
  domain.
- **expected:** Tests must validate real-world scenarios
  including error paths (per `AGENTS.md` testing rules).
- **evidence:** `crates/cross-cutting/platform/tests/platform_e2e.rs:269-305`
  contains 2 `update_school_increments_version` calls; both
  are happy paths with `domain: None`. No test exercises the
  conflict path; the 6 test cases listed in the file doc
  (lines 12-33) match the integration test plan, but the
  conflict path is missing.

---

### FINDING 32

- **id:** CROSSCUT-PLAT-032
- **area:** cross-cutting
- **severity:** Medium
- **location:** crates/cross-cutting/platform/tests/platform_e2e.rs (full file)
- **description:** The integration tests do not exercise the
  `SchoolUpdated` envelope's `changed_fields` list for
  multi-field updates. The single-field update is tested
  (`event1.changed_fields == vec!["name"]`, line 293; same
  for `event2` line 304), but a test that asserts
  `changed_fields` carries both `name` and `domain` after a
  compound `update_school` call is absent.
- **expected:** Coverage of multi-field update behavior.
- **evidence:** `crates/cross-cutting/platform/tests/platform_e2e.rs:269-305` —
  both `UpdateSchoolCommand` calls have `domain: None` and
  `package_id: None`, exercising only the `name`-change path.

---

### FINDING 33

- **id:** CROSSCUT-PLAT-033
- **area:** cross-cutting
- **severity:** Medium
- **location:** crates/cross-cutting/platform/tests/platform_e2e.rs (full file)
- **description:** No integration test covers `update_user`.
  The unit test `register_user_emits_event` exists
  (`services.rs:663-689`) and the integration test
  `user_register_emits_user_registered_event` exists, but
  no test calls `update_user` or asserts `UserUpdated` event
  metadata. This is a test gap for a documented command.
- **expected:** Integration tests for every command in the
  spec section.
- **evidence:** `grep -n "update_user\|UserUpdated" crates/cross-cutting/platform/tests/platform_e2e.rs`
  returns no rows.

---

### FINDING 34

- **id:** CROSSCUT-PLAT-034
- **area:** cross-cutting
- **severity:** Medium
- **location:** crates/cross-cutting/platform/tests/platform_e2e.rs (full file)
- **description:** No integration test covers `deactivate_school`
  end-to-end. The integration test
  `deactivate_user_sets_active_status_retired` (line 308)
  covers the user path, but the symmetric `deactivate_school`
  path is tested only as a unit test in `services.rs:640-661`
  and does not assert the `SchoolDeactivated` event envelope
  metadata.
- **expected:** Integration test coverage for both
  deactivation paths.
- **evidence:** `grep -n "deactivate_school\|SchoolDeactivated" crates/cross-cutting/platform/tests/platform_e2e.rs`
  returns no rows.

---

### FINDING 35

- **id:** CROSSCUT-PLAT-035
- **area:** cross-cutting
- **severity:** Medium
- **location:** crates/cross-cutting/platform/src/services.rs:225-260
- **description:** `deactivate_school` does not produce an
  integration test of its event envelope. The `update_school`
  envelope metadata is not tested either. Per the crate's
  `services.rs:189-191` comment, the event id is "a
  placeholder event id here for the envelope (the bus port
  stamps its own event id at publish time, so this is
  informational only)"; an envelope round-trip test would
  document the placeholder behavior.
- **expected:** Integration tests for `SchoolDeactivated`
  and `SchoolUpdated` envelope metadata.
- **evidence:** `crates/cross-cutting/platform/tests/platform_e2e.rs`
  contains `school_create_emits_school_created_event`,
  `user_register_emits_user_registered_event`,
  `envelope_propagates_correlation_id`,
  `school_starts_pending_and_event_id_round_trips`, but no
  `school_deactivate_emits_school_deactivated_event`,
  `school_update_emits_school_updated_event`,
  `user_update_emits_user_updated_event`,
  `user_deactivate_emits_user_deactivated_event`.

---

### FINDING 36

- **id:** CROSSCUT-PLAT-036
- **area:** cross-cutting
- **severity:** Low
- **location:** crates/cross-cutting/platform/src/events.rs:109-130
- **description:** `SchoolUpdated::package_id: Option<Uuid>`
  uses `Uuid` instead of the spec's `Option<SchoolPackageId>`
  (per `docs/specs/platform/events.md:45`: `package_id:
  Option<SchoolPackageId>`). Spec drift to the raw id type.
- **expected:** `docs/specs/platform/events.md:45` —
  `pub package_id: Option<SchoolPackageId>`.
- **evidence:** `crates/cross-cutting/platform/src/events.rs:122`:
  `pub package_id: Option<Uuid>,` (`grep -n "package_id" events.rs`
  returns this single occurrence; `PackageId` is the
  platform-defined newtype, but the event uses bare `Uuid`).

---

### FINDING 37

- **id:** CROSSCUT-PLAT-037
- **area:** cross-cutting
- **severity:** Low
- **location:** crates/cross-cutting/platform/src/events.rs:339-348
- **description:** `UserUpdated::role_ids` (if it existed)
  is not modeled; the event has no `role_ids` field. Spec
  invariant for `UserUpdated` only requires `changed_fields`
  per spec, but the event cannot reflect `role_ids` changes
  because the aggregate's `role_ids` change does not emit a
  separate event. The `ChangeUserRole` command is missing
  entirely (Finding 8).
- **expected:** `docs/specs/platform/events.md:121-125`:
  `pub struct UserUpdated { pub user_id: UserId, pub changed_fields: Vec<&'static str> }`.
- **evidence:** `crates/cross-cutting/platform/src/events.rs:333-355`
  defines `UserUpdated` without `role_ids`; the missing
  `ChangeUserRole` command means no event carries role
  changes.

---

### FINDING 38

- **id:** CROSSCUT-PLAT-038
- **area:** cross-cutting
- **severity:** High
- **location:** docs/specs/platform/workflows.md (201 lines, full file)
- **description:** 11 of the 14 workflows in
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
- **expected:** 14 workflows, all steps implemented.
- **evidence:** `crates/cross-cutting/platform/src/`
  contains no subscriber modules, no
  `OnSchoolCreatedSubscriber`/`OnUserRegisteredSubscriber`
  etc.; `grep -rn "Subscriber\|subscriber" src/` returns no
  rows.

---

### FINDING 39

- **id:** CROSSCUT-PLAT-039
- **area:** cross-cutting
- **severity:** Medium
- **location:** crates/cross-cutting/platform/src/services.rs:194-197, 249, 423, 491
- **description:** `update_school`, `deactivate_school`,
  `update_user`, `deactivate_user` all mint a placeholder
  `EventId::from_uuid(uuid::Uuid::now_v7())` and use it as
  `last_event_id`. The crate's `create_school` and
  `register_user` use the `IdGenerator` port (lines 102,
  330). Spec drift: `update_*` and `deactivate_*` bypass the
  port and mint their own id, defeating the test
  determinism that `create_*` provides.
- **expected:** `educore_core::clock::IdGenerator` port
  usage in all mutation services.
- **evidence:** `crates/cross-cutting/platform/src/services.rs:194-197`:
  `educore_core::ids::EventId::from_uuid(uuid::Uuid::now_v7())`;
  line 249 same; line 423 same; line 491 same. Only lines
  102 and 330 use `_ids.next_event_id()` (via `IdGenerator`
  port).

---

### FINDING 40

- **id:** CROSSCUT-PLAT-040
- **area:** cross-cutting
- **severity:** Medium
- **location:** crates/cross-cutting/platform/src/services.rs:131-212
- **description:** `update_school` accepts an `IdGenerator`
  parameter on `create_school` (line 56) and `register_user`
  (line 276) but `update_school` (line 131), `deactivate_school`
  (line 225), `update_user` (line 362), and `deactivate_user`
  (line 467) do not take an `IdGenerator`. The
  `IdGenerator` boundary is inconsistent — mutators that
  emit events should consume the port; `update_*` and
  `deactivate_*` do not.
- **expected:** Uniform port consumption across all
  mutation services.
- **evidence:** `crates/cross-cutting/platform/src/services.rs:131, 225, 362, 467` —
  4 service functions without `IdGenerator` parameters.
  Lines 56 and 276 have `where G: IdGenerator + ?Sized`
  constraints.

---

### FINDING 41

- **id:** CROSSCUT-PLAT-041
- **area:** cross-cutting
- **severity:** Low
- **location:** crates/cross-cutting/platform/src/events.rs:339-355
- **description:** `UserUpdated` carries a `phone_number:
  Option<String>` field (line 347) instead of the typed
  `Option<PhoneNumber>` value object. Spec drift to a raw
  string; loses the E.164 validation guarantee.
- **expected:** `docs/specs/platform/events.md:121-125` —
  `pub struct UserUpdated { pub user_id, pub changed_fields }`
  (no `phone_number` field per spec; per `commands.md:140-153`
  `UpdateUserCommand`, `phone_number` is `Option<PhoneNumber>`).
- **evidence:** `crates/cross-cutting/platform/src/events.rs:339-355`:
  ```rust
  pub struct UserUpdated {
      ...
      pub phone_number: Option<String>,
      ...
  }
  ```

---

### FINDING 42

- **id:** CROSSCUT-PLAT-042
- **area:** cross-cutting
- **severity:** Medium
- **location:** crates/cross-cutting/platform/src/commands.rs:172-243
- **description:** `CreateSchoolCommand::new`,
  `RegisterUserCommand::new`, `DeactivateSchoolCommand::new`,
  `DeactivateUserCommand::new` are convenience
  constructors that bypass the typed fields. `new` for
  `CreateSchoolCommand` omits `domain` and `package_id`
  (set to `None`), `RegisterUserCommand::new` hard-codes
  `usertype: UserType::Staff` (line 212) and `role_ids:
  Vec::new()`. These are reasonable for tests but they are
  the only constructors in the public API; the spec lists
  full struct fields.
- **expected:** Spec-shaped command struct initialization
  (e.g. with all fields explicit).
- **evidence:** `crates/cross-cutting/platform/src/commands.rs:175-189, 195-216, 222-229, 235-242`.

---

### FINDING 43

- **id:** CROSSCUT-PLAT-043
- **area:** cross-cutting
- **severity:** Low
- **location:** crates/cross-cutting/platform/src/services.rs:308-312
- **description:** `register_user` returns
  `Err(DomainError::Conflict(format!("email {:?} is already
  in use within the school", email.as_str())))`. The error
  format string uses `{:?}` for an `EmailAddress` that
  already implements `Display` (via `value_objects.rs:99-103`).
  Use of `{:?}` is anti-pattern (yields quoted debug form
  with escapes); `{}` should be used.
- **expected:** Display formatter usage over Debug in
  user-facing error messages.
- **evidence:** `crates/cross-cutting/platform/src/services.rs:302-306, 393-397`:
  two occurrences of `email {:?}` in error messages.

---

### FINDING 44

- **id:** CROSSCUT-PLAT-044
- **area:** cross-cutting
- **severity:** Low
- **location:** crates/cross-cutting/platform/src/services.rs:74, 241, 380, 483
- **description:** Multiple services destructure
  `tenant: TenantContext` from the command and then
  `debug_assert_eq!(tenant.school_id, school_id)` but the
  destructured `ctx` variable is the same `tenant` and is
  later used (via `ctx.actor_id`, `ctx.correlation_id`).
  The variable is captured by both the destructure and the
  `let ctx = tenant;` rebind (lines 74, 298). The pattern
  is repetitive and error-prone.
- **expected:** Consistent destructure naming and
  `debug_assert_eq!` usage.
- **evidence:** `crates/cross-cutting/platform/src/services.rs:74, 152, 241, 298, 380, 483` —
  the `let _ = ctx;` and `let ctx = tenant;` patterns
  appear in 5 services.

---

### FINDING 45

- **id:** CROSSCUT-PLAT-045
- **area:** cross-cutting
- **severity:** High
- **location:** docs/coverage.toml (platform rows)
- **description:** `docs/coverage.toml` has only 3
  `educore-platform` rows (`platform_schools_aggregate`,
  `platform_users_aggregate`, `platform_sessions_aggregate`).
  The `platform_sessions_aggregate` row references
  `UserSession`, which is a child entity in `entities.rs:62-84`
  (not a spec-listed aggregate; the spec has no
  `Session` aggregate). Coverage drift between the spec
  (37 aggregates) and the matrix (3 rows).
- **expected:** Per-aggregate coverage matrix rows matching
  the spec.
- **evidence:** `docs/coverage.toml:368-393`:
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

### FINDING 46

- **id:** CROSSCUT-PLAT-046
- **area:** cross-cutting
- **severity:** Medium
- **location:** crates/cross-cutting/platform/src/value_objects.rs:399-413
- **description:** `PackageId` is `#[serde(transparent)]`
  with the inner `Uuid` exposed (`pub Uuid`). The struct
  field is `pub`, allowing external code to mutate the
  inner id without going through `from_uuid`. Spec drift
  (the spec describes `PackageId` as an `Id<Package>` and
  generally wraps `Uuid` privately).
- **expected:** Private inner field; constructor-only
  initialization.
- **evidence:** `crates/cross-cutting/platform/src/value_objects.rs:399-413`:
  ```rust
  pub struct PackageId(pub Uuid);
  ```

---

### FINDING 47

- **id:** CROSSCUT-PLAT-047
- **area:** cross-cutting
- **severity:** Medium
- **location:** crates/cross-cutting/platform/src/value_objects.rs:430-452
- **description:** `RoleId(SchoolId, Uuid)` is
  `#[serde(transparent)]` over `SchoolId`? No, it is not
  annotated, so it serializes as a tuple of two fields. The
  spec defines `RoleId` from `educore-rbac` (per
  `value-objects.md:64`). The platform crate defines its
  own `RoleId` rather than depending on `educore-rbac`.
  Spec drift on ownership.
- **expected:** `docs/specs/platform/value-objects.md:64`:
  `RoleId | From educore-rbac`.
- **evidence:** `crates/cross-cutting/platform/src/value_objects.rs:430-458`:
  ```rust
  pub struct RoleId(pub SchoolId, pub Uuid);
  ```
  The platform crate does not depend on `educore-rbac` in
  `Cargo.toml:13-20`; both fields are `pub`, allowing
  direct mutation.

---

### FINDING 48

- **id:** CROSSCUT-PLAT-048
- **area:** cross-cutting
- **severity:** High
- **location:** docs/handoff/PHASE-2-HANDOFF.md:73-104
- **description:** Phase 2 is marked **closed** in the
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
- **expected:** Phase 2 closed means the spec surface for
  Phase 2 is implemented; the spec surface for the
  platform crate spans all 37 aggregates regardless of
  phase.
- **evidence:** `docs/handoff/PHASE-2-HANDOFF.md:73-104`
  lists only the School and User work; lines 291-292 say
  "Do NOT add the 30 secondary platform aggregates". The
  README at `crates/cross-cutting/platform/README.md:7-10`
  repeats the same scope. The audit task states
  "**Total findings:** 48" — Phase 2 readiness for the
  platform crate is partial.

---

### END FINDINGS
