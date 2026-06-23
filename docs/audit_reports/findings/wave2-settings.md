# Audit findings: educore-settings (Phase 14 / cross-cutting)

**Scope:** `crates/cross-cutting/settings/` (10 src files: `lib.rs`,
`aggregate.rs`, `entities.rs`, `value_objects.rs`, `commands.rs`,
`events.rs`, `services.rs`, `repository.rs`, `query.rs`, `errors.rs`),
`docs/specs/settings/` (11 files), `docs/commands/settings.md`,
`docs/events/settings.md`, `docs/coverage.toml` (16 settings rows),
`docs/handoff/PHASE-14-HANDOFF.md`, `AGENTS.md`.

**Spec aggregates per `docs/specs/settings/aggregates.md`** (15):
GeneralSettings, Language, LanguagePhrase, BaseSetup, BaseGroup,
DateFormat, Style, BackgroundSetting, DashboardSetting, CustomLink,
ColorTheme, Theme, Color, BehaviorRecordSetting, SetupAdmin.

**Spec commands per `docs/commands/settings.md`** (53 rows): all
per-aggregate commands enumerated in `commands.md`.

**Spec events per `docs/events/settings.md`** (52 rows): all
per-aggregate events enumerated.

**Total findings:** 28

---

### FINDING 1

- **id:** CROSSCUT-SET-001
- **area:** cross-cutting
- **severity:** High
- **location:** crates/cross-cutting/settings/src/aggregate.rs:43-107
- **description:** The `GeneralSettings` aggregate stores ~38 spec'd
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
- **expected:** Per `docs/specs/settings/value-objects.md`: "All value
  objects implement `Validate` and refuse construction when validation
  fails: ... Construction is the only entry point: `let pattern =
  DateFormatPattern::new(\"%Y-%m-%d\")?;`"
- **evidence:** `pub school_name: String, ... pub file_size: u64, ...
  pub two_factor: bool, ... pub fees_status: i32, ... pub
  active_status: bool,` (crates/cross-cutting/settings/src/aggregate.rs:46-104)

### FINDING 2

- **id:** CROSSCUT-SET-002
- **area:** cross-cutting
- **severity:** High
- **location:** crates/cross-cutting/settings/src/aggregate.rs:43-107
- **description:** The 36 module-toggle feature flags enumerated in
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
- **expected:** Per `docs/specs/settings/value-objects.md` § "Module
  Toggles": each toggle should be a typed `bool` value object on the
  `GeneralSettings` aggregate, validated at construction.
- **evidence:** `pub module_toggles: ModuleTogglePatch,` (crates/cross-cutting/settings/src/aggregate.rs:90)

### FINDING 3

- **id:** CROSSCUT-SET-003
- **area:** cross-cutting
- **severity:** Medium
- **location:** docs/specs/settings/tables.md:13-22
- **description:** `docs/specs/settings/tables.md` lists 14 owned
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
- **expected:** Per `docs/specs/settings/tables.md` § "Notes" line 81:
  "`settings_base_setups.base_group_id` references
  `settings_base_groups.id` and cascades on delete." — therefore
  `settings_base_groups` should appear as an owned table row.
- **evidence:** `| \`settings_base_setups\`               | BaseSetup                 | Lookup values                      |` (docs/specs/settings/tables.md:13); `- \`settings_base_setups.base_group_id\` references \`settings_base_groups.id\`` (docs/specs/settings/tables.md:81)

### FINDING 4

- **id:** CROSSCUT-SET-004
- **area:** cross-cutting
- **severity:** High
- **location:** crates/cross-cutting/settings/src/aggregate.rs (entire file)
- **description:** Zero `#[derive(DomainQuery)]` macros are applied to
  any struct in `aggregate.rs` or `entities.rs` — the engine's
  macro-driven query layer is not wired into the settings crate. The
  14 owned settings tables documented in `docs/specs/settings/tables.md`
  (lines 9-22) have no macro-emitted typed query structs in the crate.
  `query.rs` (309 lines) contains only empty placeholder structs (e.g.
  `pub struct GeneralSettingsQuery { /* Fields filled in by Workstream
  A. */ }`).
- **expected:** Per `docs/specs/settings/aggregates.md` and
  `docs/query_layer.md`, the settings aggregates should expose typed
  query builders via `#[derive(DomainQuery)]` (or equivalent).
- **evidence:** `pub struct GeneralSettingsQuery { // Fields filled in by Workstream A. }` (crates/cross-cutting/settings/src/query.rs:17-19); `grep -c "DomainQuery" aggregate.rs entities.rs` returns 0

### FINDING 5

- **id:** CROSSCUT-SET-005
- **area:** cross-cutting
- **severity:** High
- **location:** crates/cross-cutting/settings/src/aggregate.rs:1708-1721
- **description:** `Theme::activate` contains a guard that is meant to
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
- **expected:** The guard should `return Err(SettingsDomainError::Conflict(...))`
  to enforce the invariant.
- **evidence:** `if prev.is_default && !self.is_default {
        // Demoting a default theme is not allowed unless
        // the new theme is also a default.
    }` (crates/cross-cutting/settings/src/aggregate.rs:1710-1713)

### FINDING 6

- **id:** CROSSCUT-SET-006
- **area:** cross-cutting
- **severity:** High
- **location:** crates/cross-cutting/settings/src/aggregate.rs:1869-1875
- **description:** `Color::delete` does not check `theme_binding_count`
  before soft-deleting, even though `ColorService::can_delete` (in
  `services.rs:548-556`) is defined to enforce this. Spec
  `docs/specs/settings/aggregates.md` § "Color" defines `Color` as
  referenced by `ColorTheme` rows and `docs/specs/settings/services.md`
  § "ColorService" defines `can_delete(color, theme_binding_count) ->
  Result<(), ConflictError>`. The aggregate-level `delete` method
  takes no count parameter and unconditionally soft-deletes, leaving
  the conflict check only enforceable from outside via the service
  helper.
- **expected:** Per the spec service contract, `Color::delete` should
  either take a `theme_binding_count: u64` parameter and return
  `Result<(), SettingsDomainError>`, or the dispatcher must always
  call `ColorService::can_delete` before the aggregate method.
- **evidence:** `/// Soft-deletes the color.
    pub fn delete(&mut self, at: Timestamp, actor: UserId) {
        self.active_status = false;
        self.status = ColorStatus::new(false);` (crates/cross-cutting/settings/src/aggregate.rs:1868-1871)

### FINDING 7

- **id:** CROSSCUT-SET-007
- **area:** cross-cutting
- **severity:** High
- **location:** crates/cross-cutting/settings/src/events.rs:243-251
- **description:** `TimeZoneChanged` uses raw `String` for
  `from_time_zone_id` and `to_time_zone_id`, but the spec
  (`docs/specs/settings/events.md` line 85-86) requires typed
  `Option<TimeZoneId>` and `TimeZoneId`. The spec
  `value-objects.md` line 59 lists `TimeZoneId | From platform`.
  This is a doc-vs-code drift.
- **expected:** `pub from_time_zone_id: Option<TimeZoneId>, pub
  to_time_zone_id: TimeZoneId,` per the spec.
- **evidence:** `pub from_time_zone_id: Option<String>,
    pub to_time_zone_id: String,` (crates/cross-cutting/settings/src/events.rs:246-247)

### FINDING 8

- **id:** CROSSCUT-SET-008
- **area:** cross-cutting
- **severity:** High
- **location:** docs/specs/settings/services.md:127
- **description:** `BehaviorRecordService::patch` (spec line 127)
  takes a `BehaviorRecordPatch` parameter, but no such struct is
  defined anywhere in `entities.rs` or `value_objects.rs` (the
  `BehaviorRecordSetting` aggregate has `apply_update` taking four
  separate `Option<BehaviorFlag>` parameters directly). The spec
  describes a typed patch entity that is absent from the
  implementation, breaking the symmetric patch pattern used by other
  aggregates (e.g. `GeneralSettingsPatch`).
- **expected:** A `BehaviorRecordPatch` struct should exist per
  `docs/specs/settings/services.md` line 127, with optional fields
  for `student_comment`, `parent_comment`, `student_view`,
  `parent_view`.
- **evidence:** `pub fn patch(setting: &mut BehaviorRecordSetting, patch: BehaviorRecordPatch) { ... }` (docs/specs/settings/services.md:127); `grep -rn "BehaviorRecordPatch" crates/cross-cutting/settings/` returns 0 matches

### FINDING 9

- **id:** CROSSCUT-SET-009
- **area:** cross-cutting
- **severity:** High
- **location:** crates/cross-cutting/settings/src/value_objects.rs:1376-1390
- **description:** `ColorFormat` is declared as a typed wrapper but is
  not in `docs/specs/settings/value-objects.md` and is not consumed
  anywhere in the crate — it is only referenced in a
  `_suppress_unused_value_object_imports` helper
  (`services.rs:699-703`). This is a dead-code value object that
  silently consumes the `ColorFormat` import namespace and could be
  confused with `CurrencyFormat`.
- **expected:** Either the spec should describe `ColorFormat` (it
  does not), or the wrapper should be removed. The current state
  leaks an undocumented type into the public prelude via
  `lib.rs:54`.
- **evidence:** `/// Currency format alias — kept separate from
    /// [\`CurrencyFormat\`] to follow the spec's "ColorFormat"
    /// naming without conflict.
    pub struct ColorFormat(pub String);` (crates/cross-cutting/settings/src/value_objects.rs:1375-1378)

### FINDING 10

- **id:** CROSSCUT-SET-010
- **area:** cross-cutting
- **severity:** Medium
- **location:** crates/cross-cutting/settings/src/value_objects.rs:393-417
- **description:** `RtlLtl` is declared as a typed wrapper for the
  1=RTL/2=LTR discriminator but is never used outside its own test
  (value_objects.rs:1523-1529). The spec `value-objects.md` line 51
  also lists `RtlLtl` as a value object, but no field in any
  aggregate or command stores `RtlLtl`. The aggregate `Language`
  uses `RtlFlag(pub bool)` (line 1086) for the same concept. The
  duplication is undocumented.
- **expected:** Either `RtlLtl` should be wired into a field (e.g.
  `GeneralSettings.rtl_ltl`), or removed. The current state has two
  parallel RTL concepts (`RtlFlag` bool and `RtlLtl` i32) with no
  spec'd mapping.
- **evidence:** `pub struct RtlLtl(pub i32);` (crates/cross-cutting/settings/src/value_objects.rs:395); only references are in the value_objects.rs module tests at lines 1525-1527

### FINDING 11

- **id:** CROSSCUT-SET-011
- **area:** cross-cutting
- **severity:** Medium
- **location:** crates/cross-cutting/settings/src/aggregate.rs:1479-1481
- **description:** `CustomLink::count_links` uses
  `.try_into().unwrap_or(u32::MAX)` to convert from `usize` to `u32`.
  AGENTS.md § "Type Safety" forbids `as` casts on numerics that may
  lose data; `.unwrap_or(u32::MAX)` is a lossy fallback that returns a
  valid-looking but misleading count when the link list exceeds
  `u32::MAX` (impossible in practice, but the fallback hides bugs
  instead of propagating the error).
- **expected:** Per AGENTS.md § "Type Safety", use `TryFrom`/`TryInto`
  with proper error handling — return a `Result<u32,
  SettingsDomainError>` instead of silently substituting
  `u32::MAX`.
- **evidence:** `self.links.len().try_into().unwrap_or(u32::MAX)` (crates/cross-cutting/settings/src/aggregate.rs:1480)

### FINDING 12

- **id:** CROSSCUT-SET-012
- **area:** cross-cutting
- **severity:** High
- **location:** crates/cross-cutting/settings/src/services.rs:338-349
- **description:** `ThemeService::replicate` signature diverges from
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
- **expected:** Match the spec signature and return
  `Result<Theme, SettingsDomainError>`.
- **evidence:** `source
            .replicate(new_id, new_title, ts, actor)
            .unwrap_or_else(|_| source.clone())` (crates/cross-cutting/settings/src/services.rs:347-348)

### FINDING 13

- **id:** CROSSCUT-SET-013
- **area:** cross-cutting
- **severity:** Medium
- **location:** crates/cross-cutting/settings/src/aggregate.rs:1938-1962
- **description:** `BehaviorRecordSetting::apply_update` is named
  inconsistently with the other 14 aggregates which all use a method
  named `update`. The spec describes a single
  `UpdateBehaviorRecordSetting` command and does not specify the
  method name; the asymmetric naming (every other aggregate: `update`
  or `activate`, here: `apply_update`) makes the aggregate API
  non-uniform.
- **expected:** All aggregates should expose a uniform method name
  (`update`) for the patch mutation, matching the pattern in
  `Language::update`, `BaseGroup::update`, etc.
- **evidence:** `pub fn apply_update(
        &mut self,
        student_comment: Option<BehaviorFlag>,
        parent_comment: Option<BehaviorFlag>,
        student_view: Option<BehaviorFlag>,
        parent_view: Option<BehaviorFlag>,
        actor: UserId,
        at: Timestamp,
    ) {` (crates/cross-cutting/settings/src/aggregate.rs:1938-1946)

### FINDING 14

- **id:** CROSSCUT-SET-014
- **area:** cross-cutting
- **severity:** Medium
- **location:** crates/cross-cutting/settings/src/aggregate.rs:1159-1165
- **description:** `Style::delete` returns no `Result`, but the
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
- **expected:** Either the aggregate `delete` methods should take
  the relevant reference-count parameter and return `Result<(),
  SettingsDomainError>` matching `Theme::delete` and
  `BehaviorRecordSetting::apply_update`, or the spec should explicitly
  require dispatcher-level service checks.
- **evidence:** `/// Soft-deletes the style.
    pub fn delete(&mut self, at: Timestamp, actor: UserId) {
        self.active_status = false;` (crates/cross-cutting/settings/src/aggregate.rs:1159-1161)

### FINDING 15

- **id:** CROSSCUT-SET-015
- **area:** cross-cutting
- **severity:** High
- **location:** crates/cross-cutting/settings/src/ (entire crate)
- **description:** `crates/cross-cutting/settings/tests/` directory
  does not exist. Per AGENTS.md § "Testing (TDD)": "At least one
  integration test per PR". The integration test for the settings
  domain is in `crates/tools/storage-parity/tests/settings_integration.rs`
  (a different crate), but the settings crate itself has no
  `tests/` directory with crate-local integration tests.
- **expected:** Per AGENTS.md § "Validation Checklist", at least one
  integration test added for new behavior — should be co-located at
  `crates/cross-cutting/settings/tests/`.
- **evidence:** `ls -la crates/cross-cutting/settings/tests/` returns "No such file or directory"

### FINDING 16

- **id:** CROSSCUT-SET-016
- **area:** cross-cutting
- **severity:** Medium
- **location:** crates/cross-cutting/settings/src/aggregate.rs (entire file)
- **description:** No `workflows.rs` module exists in the settings
  crate, despite `docs/specs/settings/workflows.md` (123 lines)
  defining 8 workflows with 36 enumerated steps (Initial School
  Setup, Language Management, Theme Configuration, Base Setup
  Management, Custom Link Configuration, Date Format Configuration,
  Dashboard Configuration, Two-Factor Configuration). The
  workflows.md spec describes "ordered, conditional steps" but no
  workflow orchestrator or handler is implemented.
- **expected:** Per `docs/specs/settings/workflows.md` and the
  standard 9-file module layout (`aggregate.rs`, `entities.rs`,
  `value_objects.rs`, `commands.rs`, `events.rs`, `services.rs`,
  `repository.rs`, `query.rs`, `errors.rs`), an additional
  `workflows.rs` module (or equivalent handler dispatch) should be
  present.
- **evidence:** `grep "pub mod" crates/cross-cutting/settings/src/lib.rs` returns only `aggregate, commands, entities, errors, events, query, repository, services, value_objects` — no `workflows`

### FINDING 17

- **id:** CROSSCUT-SET-017
- **area:** cross-cutting
- **severity:** Medium
- **location:** crates/cross-cutting/settings/src/aggregate.rs:1409-1415
- **description:** `CustomLink::new` initialises an empty bundle with
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
- **expected:** The aggregate should hold `links: Vec<CustomLinkEntry>`
  (per `entities.md` § "CustomLinkEntry") so individual entry
  invariants can be enforced at the entity level.
- **evidence:** `pub links: Vec<(LinkLabel, LinkHref)>,` (crates/cross-cutting/settings/src/aggregate.rs:1395)

### FINDING 18

- **id:** CROSSCUT-SET-018
- **area:** cross-cutting
- **severity:** Medium
- **location:** crates/cross-cutting/settings/src/entities.rs:598-629
- **description:** `SettingsAuditEntry::new` defaults
  `school_id: SchoolId::from_uuid(Uuid::nil())` rather than taking
  the `SchoolId` as a constructor parameter. This means every
  audit entry created without explicit override will be attributed to
  the nil school — a data-integrity hazard for a multi-tenant
  system. Per AGENTS.md § "Engine Rules" item 7: "Multi-tenant by
  default. Every aggregate has a SchoolId."
- **expected:** `SettingsAuditEntry::new` should take a `school_id:
  SchoolId` parameter (and all other fields) explicitly, with no
  `nil` default.
- **evidence:** `school_id: SchoolId::from_uuid(Uuid::nil()),
        entry_id: Uuid::new_v4(),
        aggregate_type: aggregate_type.into(),` (crates/cross-cutting/settings/src/entities.rs:613-615)

### FINDING 19

- **id:** CROSSCUT-SET-019
- **area:** cross-cutting
- **severity:** Medium
- **location:** crates/cross-cutting/settings/src/aggregate.rs:178-187
- **description:** `GeneralSettings::new` initialises
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
- **expected:** Use `EmailDriver::new("smtp")?` and
  `PreloaderType::new(1)?` (or store as raw fields if the spec
  permits, but the spec mandates `Validate` at construction).
- **evidence:** `// "smtp" is a valid 1..64 char string by spec; bypass validation.
            email_driver: EmailDriver("smtp".to_owned()),` (crates/cross-cutting/settings/src/aggregate.rs:177-178); `// 1 is a valid PreloaderType by spec; bypass validation.
            preloader_type: PreloaderType(1),` (crates/cross-cutting/settings/src/aggregate.rs:186-187)

### FINDING 20

- **id:** CROSSCUT-SET-020
- **area:** cross-cutting
- **severity:** Medium
- **location:** crates/cross-cutting/settings/src/value_objects.rs:395-417
- **description:** `RtlLtl::new` returns `Result<Self, DomainError>`
  (line 398), but the `prelude` re-exports it via
  `crates/cross-cutting/settings/src/lib.rs:61`. The `prelude` does
  not include `DomainError` (the prelude only re-exports
  `SettingsDomainError` from `errors::Result`). Consumers using
  `RtlLtl::new` from the prelude will not have a stable `DomainError`
  import path. (See also FINDING 10 — `RtlLtl` is dead code
  outside its own tests.)
- **expected:** Either remove the dead `RtlLtl` value object or
  re-export the required `DomainError` through the prelude.
- **evidence:** `pub fn new(v: i32) -> Result<Self> {` (crates/cross-cutting/settings/src/value_objects.rs:398); `pub use crate::errors::{Result, SettingsDomainError};` (crates/cross-cutting/settings/src/lib.rs:50) — `DomainError` is not re-exported.

### FINDING 21

- **id:** CROSSCUT-SET-021
- **area:** cross-cutting
- **severity:** Medium
- **location:** crates/cross-cutting/settings/src/aggregate.rs:1480
- **description:** `CustomLink::count_links` uses
  `self.links.len().try_into().unwrap_or(u32::MAX)`. The
  `try_into` succeeds for any list under `u32::MAX` items, but the
  silent fallback masks any future ABI/type change. This is the
  same numeric-conversion anti-pattern flagged in AGENTS.md § "Type
  Safety". The aggregate uses `Vec` (untyped) instead of a typed
  `CustomLinkEntry` (see FINDING 17) so there is no compile-time
  cap on `links.len()`.
- **expected:** Replace with an explicit
  `u32::try_from(self.links.len()).map_err(|_| SettingsDomainError::Validation(...))?`.
- **evidence:** `self.links.len().try_into().unwrap_or(u32::MAX)` (crates/cross-cutting/settings/src/aggregate.rs:1480)

### FINDING 22

- **id:** CROSSCUT-SET-022
- **area:** cross-cutting
- **severity:** Low
- **location:** docs/specs/settings/commands.md:8, crates/cross-cutting/settings/src/commands.rs:151-204
- **description:** `SeedGeneralSettingsCommand` is declared in
  `commands.rs:151-204` (with `COMMAND_TYPE = "settings.general_settings.seed"`)
  but is not enumerated in `docs/specs/settings/commands.md` or in
  `docs/commands/settings.md`. This is a spec-vs-code drift — the
  spec does not describe a "seed" command, and `docs/handoff/PHASE-14-HANDOFF.md`
  line 27 advertises "53 typed settings commands" but
  `commands.rs` has 54 typed command structs (1 extra: `SeedGeneralSettingsCommand`).
- **expected:** Either add the `SeedGeneralSettingsCommand` to the
  spec docs/commands/settings.md and docs/specs/settings/commands.md,
  or remove it from `commands.rs` if it is unused.
- **evidence:** `pub const COMMAND_TYPE: &'static str = "settings.general_settings.seed";` (crates/cross-cutting/settings/src/commands.rs:174); the command does not appear in docs/specs/settings/commands.md (1-405) or docs/commands/settings.md (1-69)

### FINDING 23

- **id:** CROSSCUT-SET-023
- **area:** cross-cutting
- **severity:** Low
- **location:** docs/handoff/PHASE-14-HANDOFF.md:26-27
- **description:** The handoff advertises "53 typed settings events"
  and "53 typed settings commands" but the actual code
  (`events.rs` and `commands.rs`) defines 52 event structs and 54
  command structs. The discrepancy is small but breaks the
  hand-off's own numerical claims: events.rs has 52 `pub struct`
  definitions matching `docs/events/settings.md` (52 rows); commands.rs
  has 54 with one extra (`SeedGeneralSettingsCommand`, see FINDING 22).
- **expected:** The handoff numbers should reconcile with the code:
  either bump to "54 commands" (and add `Seed` to the catalog) or
  drop `Seed` from the code.
- **evidence:** `**53 typed settings events** + **25 typed operations events**` (docs/handoff/PHASE-14-HANDOFF.md:26); `awk '/^pub struct /{count++}' crates/cross-cutting/settings/src/events.rs` returns 52; `awk '/^pub struct .*Command /' crates/cross-cutting/settings/src/commands.rs` returns 54

### FINDING 24

- **id:** CROSSCUT-SET-024
- **area:** cross-cutting
- **severity:** Low
- **location:** crates/cross-cutting/settings/src/value_objects.rs:6-7
- **description:** `value_objects.rs` declares
  `#![allow(dead_code, clippy::all)]` (line 6) and
  `#![allow(missing_docs)]` (line 7) at module level. Per AGENTS.md
  § "Type Safety": "No `#[allow(dead_code)]` ... to silence the
  compiler. Delete unused code, wire it in, or open a follow-up
  issue." The blanket `allow(dead_code)` masks dead-code findings
  (e.g. FINDING 10: `RtlLtl`, FINDING 9: `ColorFormat`) instead of
  removing the unused declarations. `aggregate.rs` (line 9) and
  `entities.rs` (line 6) carry the same blanket `allow(missing_docs,
  dead_code, clippy::all)`.
- **expected:** Remove the blanket `allow(dead_code)` (and the
  unused value objects it masks), per AGENTS.md § "Type Safety".
- **evidence:** `#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]` (crates/cross-cutting/settings/src/value_objects.rs:6-7)

### FINDING 25

- **id:** CROSSCUT-SET-025
- **area:** cross-cutting
- **severity:** Low
- **location:** crates/cross-cutting/settings/src/events.rs:7, crates/cross-cutting/settings/src/commands.rs:7, crates/cross-cutting/settings/src/services.rs:7, crates/cross-cutting/settings/src/repository.rs:8, crates/cross-cutting/settings/src/query.rs:7
- **description:** Each of `events.rs`, `commands.rs`, `services.rs`,
  `repository.rs`, `query.rs` declares
  `#![allow(dead_code, clippy::all)]` at module level. Per AGENTS.md
  § "Type Safety": "No `#[allow(dead_code)]` ... to silence the
  compiler. Delete unused code, wire it in, or open a follow-up
  issue." The blanket allowance masks any dead-code warnings that
  might point to unwired pieces (e.g. policy types, query stubs).
- **expected:** Remove the blanket `allow(dead_code)` annotations.
- **evidence:** `#![allow(dead_code, clippy::all)]` (crates/cross-cutting/settings/src/events.rs:7); same pattern in commands.rs:7, services.rs:7, repository.rs:8, query.rs:7

### FINDING 26

- **id:** CROSSCUT-SET-026
- **area:** cross-cutting
- **severity:** Medium
- **location:** crates/cross-cutting/settings/src/services.rs:601-643
- **description:** `OnlyOneActiveStyle` and `OneDefaultThemePerSchool`
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
- **expected:** Per spec, policies should implement `Policy<C>` and
  receive the command context; the current implementation deviates
  from this contract.
- **evidence:** `pub struct OnlyOneActiveStyle;

impl OnlyOneActiveStyle {
    /// Returns \`Ok(())\` if activating \`target\` is allowed (no other
    /// style is currently active).
    pub fn check(target: &Style, others: &[Style]) -> Result<(), &'static str> {` (crates/cross-cutting/settings/src/services.rs:601-606); spec: `impl Policy<ActivateStyleCommand> for OnlyOneActiveStyle` (docs/specs/settings/services.md:170)

### FINDING 27

- **id:** CROSSCUT-SET-027
- **area:** cross-cutting
- **severity:** Low
- **location:** crates/cross-cutting/settings/src/aggregate.rs:154-155, crates/cross-cutting/settings/src/aggregate.rs:518-519
- **description:** Multiple aggregates use the pattern
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
- **expected:** Either initialise `id` first then derive `school_id`
  in a second statement, or rely on a `cmd.id.school_id()` call after
  `id` is assigned.
- **evidence:** `school_id: cmd.id.school_id(),
            id: cmd.id,
            school_name: cmd.school_name,` (crates/cross-cutting/settings/src/aggregate.rs:154-156)

### FINDING 28

- **id:** CROSSCUT-SET-028
- **area:** cross-cutting
- **severity:** Low
- **location:** docs/specs/settings/tables.md (full file)
- **description:** `docs/specs/settings/tables.md` lists 14 owned
  settings tables and 6 cross-domain tables (total 20 rows including
  duplicates: `settings_languages` and `settings_date_formats`
  appear twice each — once owned, once cross-domain-referenced).
  The spec text duplicates these table names without flagging the
  duplication, and the table is missing the `settings_base_groups`
  row despite the FK reference on line 81. The 14 owned tables
  correspond to 15 root aggregates (the discrepancy is `BaseGroup`
  having no table row). This is a doc-vs-spec drift.
- **expected:** A unified, deduplicated table list with every root
  aggregate mapped to one owned table (including `settings_base_groups`).
- **evidence:** `awk -F'|' '/^\| `/{gsub(/^ +| +$/, "", $2); print $2}' docs/specs/settings/tables.md` returns 20 rows with `settings_languages` and `settings_date_formats` each appearing twice

### END FINDINGS

**Total findings:** 28
