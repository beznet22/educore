# Phase 14 → Phase 15 Hand-off

**Audience:** the next agent starting Phase 15 (port adapters:
`educore-auth` + `educore-notify` + `educore-payment` +
`educore-files` + `educore-integrations`).
**Status:** Phase 14 closed. **`educore-settings`** and
**`educore-operations`** are the two new cross-cutting tier
crates shipped. **Spec-faithful 15 + 8 interpretation** (per
the Phase 13 precedent): 15 settings root aggregates + 8
operations root aggregates + 25 child entities + 53+25 typed
events implementing `DomainEvent` + 53+28 typed commands +
15+11 repository port traits + 15+8 typed query stubs + 11+8
service factory structs + 2+3 policies + 3+4 specifications.

## Headline numbers

- **15 settings root aggregates** ship as first-class ports:
  GeneralSettings, Language, LanguagePhrase, BaseSetup,
  BaseGroup, DateFormat, Style, BackgroundSetting,
  DashboardSetting, CustomLink, ColorTheme, Theme, Color,
  BehaviorRecordSetting, SetupAdmin.
- **8 operations root aggregates** ship as first-class ports:
  Backup, Job, FailedJob, SystemVersion, VersionHistory,
  UserLog, MaintenanceSetting, Sidebar.
- **53 typed settings events** + **25 typed operations events**
  with wire form `<domain>.<aggregate>.<verb>`.
- **53 typed settings commands** + **28 typed operations
  commands** with the matching wire form + `COMMAND_TYPE`
  constants.
- **100 net-new `Capability` variants** in `educore-rbac` (66
  Settings.* + 34 Operations.*). The 2 Phase 2
  `SettingsManage`/`OperationsManage` placeholders are
  REMOVED.
- **23 net-new `AuditTarget` variants** in `educore-audit` (15
  Settings + 8 Operations). The 2 Phase 2 `SchoolSettings`/
  `BellSchedule` placeholders are PRESERVED.
- **43 unit tests** in `educore-settings` + **47 unit tests** in
  `educore-operations` + **51 tests** in `educore-rbac` + **29
  tests** in `educore-audit` + **5 + 2 settings_integration
  scenarios** + **5 + 2 operations_integration scenarios** in
  `crates/tools/storage-parity/tests/`.
- **28 `docs/coverage.toml` rows** flipped from `Pending` →
  `Tested` (15 settings + 8 operations + 4 cross-crate
  capability/audit target rows + 2 placeholder rows kept).

## Validation gates (all green)

- `cargo build -p educore-settings` — clean (11 unused-import
  warnings for re-export-only items; no errors)
- `cargo build -p educore-operations` — clean (0 warnings)
- `cargo build --workspace` — clean
- `cargo test -p educore-settings --lib` — **43 passed, 0 failed**
- `cargo test -p educore-operations --lib` — **47 passed, 0
  failed**
- `cargo test -p educore-rbac --lib` — **51 passed, 0 failed**
  (the new `settings_capabilities_round_trip_and_resolve_to_settings_domain`
  + `operations_capabilities_round_trip_and_resolve_to_operations_domain`
  tests assert ≥ 66 Settings caps and ≥ 34 Operations caps)
- `cargo test -p educore-audit --lib` — **29 passed, 0 failed**
  (the new `settings_audit_target_round_trip_for_all_aggregates`
  + `operations_audit_target_round_trip_for_all_aggregates`
  tests assert 15 Settings + 8 Operations targets)
- `cargo test -p educore-storage-parity --test
  settings_integration` — **5 passed, 2 ignored** (env-gated
  PG/MySQL)
- `cargo test -p educore-storage-parity --test
  operations_integration` — **5 passed, 2 ignored** (env-gated
  PG/MySQL)
- `cargo fmt --all -- --check` — clean
- `cargo run -p educore-core --bin lint --features lint` — clean

> **Note on `cargo clippy --workspace --all-targets -- -D
> warnings`:** pre-existing clippy debt in `educore-finance`
> (Phase 7 WIP), `educore-hr` (Phase 6 WIP), and
> `educore-facilities` (Phase 8 WIP) prevents this gate from
> being green at the workspace level. The Phase 14 crates
> themselves pass clippy. The pre-existing issues are
> unrelated to Phase 14 and are documented as outstanding work
> in `docs/progress-tracker.md`.

## What's wired and working

### `educore-settings` (`crates/cross-cutting/settings/`)

The **fifteenth** crate. 9-file module layout. Lives in the
**cross-cutting** tier (per `docs/build-plan.md` § "Phase 14").
The settings domain is the school's cosmetic and behavioral
configuration. Phase 14 ships spec-faithful (per the Phase 13
precedent): all 15 root aggregates per
`docs/specs/settings/aggregates.md`.

- [`GeneralSettings`](crates/cross-cutting/settings/src/aggregate.rs) —
  per-school singleton. 7 events + 7 commands. Carries ~50
  fields (school identity, contact, currency, language, theme,
  copyright, feature flags, module toggles). The
  `ColorHex::is_valid` + `DateFormatPattern::is_strftime_valid`
  + `LinkHref::is_url` validators are the headline value-object
  correctness checks.
- [`Language`](crates/cross-cutting/settings/src/aggregate.rs) —
  per-school language registry. 5 events + 5 commands. Carries
  code/name/native/rtl flags. `LanguageCode::new` validates
  2..191 chars.
- [`LanguagePhrase`](crates/cross-cutting/settings/src/aggregate.rs) —
  translatable phrase key. 4 events + 4 commands. Carries
  module + default + per-locale translations.
- [`BaseSetup`](crates/cross-cutting/settings/src/aggregate.rs) +
  [`BaseGroup`](crates/cross-cutting/settings/src/aggregate.rs) —
  lookup tables. 3 events + 3 commands each.
- [`DateFormat`](crates/cross-cutting/settings/src/aggregate.rs) —
  `strftime` pattern. 3 events + 3 commands. The
  `DateFormatPattern::is_strftime_valid` validator accepts any
  pattern containing `%`.
- [`Style`](crates/cross-cutting/settings/src/aggregate.rs) —
  color palette / theme profile. 4 events + 4 commands.
- [`BackgroundSetting`](crates/cross-cutting/settings/src/aggregate.rs) —
  background image or color preset. 3 events + 3 commands.
- [`DashboardSetting`](crates/cross-cutting/settings/src/aggregate.rs) —
  dashboard card binding to a role. 3 events + 3 commands.
- [`CustomLink`](crates/cross-cutting/settings/src/aggregate.rs) —
  per-school singleton. 2 events + 2 commands.
- [`ColorTheme`](crates/cross-cutting/settings/src/aggregate.rs) —
  global color binding in a theme. 3 events + 3 commands.
- [`Theme`](crates/cross-cutting/settings/src/aggregate.rs) —
  theme (color mode, background). 5 events + 5 commands.
- [`Color`](crates/cross-cutting/settings/src/aggregate.rs) —
  global color entry. 3 events + 3 commands.
- [`BehaviorRecordSetting`](crates/cross-cutting/settings/src/aggregate.rs) —
  per-school singleton. 1 event + 1 command. Carries 4
  `BehaviorFlag` fields (0..2).
- [`SetupAdmin`](crates/cross-cutting/settings/src/aggregate.rs) —
  purpose/complaint/source/reference entry. 3 events + 3
  commands.

### `educore-operations` (`crates/cross-cutting/operations/`)

The **sixteenth** crate. 9-file module layout. Lives in the
**cross-cutting** tier. The operations domain is the school's
infrastructure-level concerns. Phase 14 ships spec-faithful: all
8 root aggregates per `docs/specs/operations/aggregates.md`.

- [`Backup`](crates/cross-cutting/operations/src/aggregate.rs) —
  per-school backup record. 5 events + 5 commands. Carries
  file_name + source_link + file_type (Database/File/Image).
- [`Job`](crates/cross-cutting/operations/src/aggregate.rs) —
  global pending job. 5 events + 5 commands. State machine:
  Pending → Reserved → Completed/Failed. The
  `JobService::next_backoff(JobAttempts)` exponential-backoff
  calculator is the headline value-object correctness check
  (1, 2, 4, 8, 16, 32, ...).
- [`FailedJob`](crates/cross-cutting/operations/src/aggregate.rs) —
  global terminal record. 3 events + 3 commands.
- [`SystemVersion`](crates/cross-cutting/operations/src/aggregate.rs) —
  global version metadata. 2 events + 2 commands. The
  `VersionName::is_semver` validator accepts `MAJOR.MINOR.PATCH`
  and rejects non-numeric or partial.
- [`VersionHistory`](crates/cross-cutting/operations/src/aggregate.rs) —
  global append-only version bump record. 1 event + 1 command.
- [`UserLog`](crates/cross-cutting/operations/src/aggregate.rs) —
  per-school append-only login record. 1 event + 1 command.
  The `IpAddress::is_valid` validator accepts IPv4, IPv6, or
  empty.
- [`MaintenanceSetting`](crates/cross-cutting/operations/src/aggregate.rs) —
  per-school singleton. 3 events + 3 commands. The
  `MaintenanceLockout` + `DisableMaintenanceGuard` policies
  gate non-SuperAdmin access while maintenance is enabled.
- [`Sidebar`](crates/cross-cutting/operations/src/aggregate.rs) —
  per-role sidebar layout projection. 4 events + 4 commands.
  Carries level (Parent/Child/SubChild) + position ordering.

### Cross-crate extensions

- `educore-rbac`: 100 net-new Capability variants (66 Settings.*
  + 34 Operations.*) per the two specs. The 2 Phase 2
  `SettingsManage` + `OperationsManage` placeholders are
  REMOVED (replaced by the full per-aggregate catalog). The
  `SuperAdmin` role in `DefaultRoleCatalog` now gets all 100
  new caps via a filter on `Capability::all()`. The
  `settings_capabilities_round_trip_and_resolve_to_settings_domain`
  + `operations_capabilities_round_trip_and_resolve_to_operations_domain`
  tests assert the full catalog.
- `educore-audit`: 23 net-new AuditTarget variants (15
  Settings + 8 Operations). The 2 Phase 2 `SchoolSettings` +
  `BellSchedule` placeholders are PRESERVED (mirrors the
  Phase 13 pattern of preserving `CalendarEvent`/`Holiday`/
  `Incident` placeholders). The
  `settings_audit_target_round_trip_for_all_aggregates` +
  `operations_audit_target_round_trip_for_all_aggregates`
  tests assert the full catalog.
- `crates/tools/storage-parity`: new `educore-settings` +
  `educore-operations` deps. New
  `tests/settings_integration.rs` (5 + 2 scenarios) + new
  `tests/operations_integration.rs` (5 + 2 scenarios).

## Cross-crate placeholders

**2 REPLACED** Phase 2 capability placeholders
(`SettingsManage`/`OperationsManage` → 100 net-new per-aggregate
caps).
**2 PRESERVED** Phase 2 AuditTarget placeholders
(`SchoolSettings`/`BellSchedule` — retained for
`DefaultRoleCatalog` consistency; mirrors Phase 13's pattern of
preserving 3 calendar audit-target placeholders).
No `CommunicationMessage*` / `Documents*` / finance
placeholders touched.
The 4 `EventsCalendar*` + 4 `CmsPage*` capability placeholders
and 7 calendar AuditTarget placeholders are NOT touched.

## Concurrency strategy

Per the Phase 9–13 hand-off template: **Phase 14 has no new
concurrency strategy**; append-only invariants are enforced at
the trait level; `VersionHistory` and `UserLog` are append-only
(no update/delete in aggregate, only admin-override
soft-delete). `GeneralSettings`, `CustomLink`,
`BehaviorRecordSetting`, and `MaintenanceSetting` are per-school
singletons (one row per `SchoolId`).

The same row-level lock strategy as Phases 7–13 applies: the
dispatcher acquires the row-level lock on the relevant row
(PG `SELECT ... FOR UPDATE` or SQLite write lock) before
calling the service and writing audit / outbox / idempotency
rows in a single transaction.

Soft-delete pattern: all root aggregates set `active_status =
false` on delete; the row is never hard-deleted; `find_*`
queries filter on `active_status = true`.

## Headline correctness checks

1. **`ColorHex::is_valid`** — accepts `#RGB`/`#RRGGBB`/`#RRGGBBAA`/
   named colors; rejects `#zz`/empty. Verified by the
   `settings_color_hex_validator_subset` integration test.
2. **`JobService::next_backoff(JobAttempts(n))`** — exponential
   1, 2, 4, 8, 16, 32, ... Verified by the
   `operations_job_service_next_backoff` integration test.
3. **`VersionName::is_semver`** — strict `MAJOR.MINOR.PATCH`
   validation. Verified by the
   `operations_integration_sqlite_vertical_slice` test.
4. **`IpAddress::is_valid`** — IPv4, IPv6, or empty. Verified by
   the same test.
5. **`BehaviorFlag::new(0|1|2)`** — accepts Off/On/Inherit.
   Verified by the `settings_behavior_flag_validator` test.

## Open questions

1. **`Color` and `ColorTheme` are global** (NEW) — these
   aggregates have no `school_id` in their typed ids. The
   events for these aggregates carry `SchoolId::from_uuid(Uuid::nil())`
   in `school_id()`. A follow-up phase may promote them to
   tenant-scoped if the consumer deployment requires
   per-school color palettes.
2. **`Job`, `FailedJob`, `SystemVersion`, `VersionHistory` are
   global** (carry-over from spec) — these aggregates have no
   `school_id`. The events for these aggregates carry
   `SchoolId::from_uuid(Uuid::nil())` in `school_id()`. A
   follow-up phase may add `tenant_id` for SaaS deployments.
3. **`UserLog` and `VersionHistory` are append-only** (carry-over
   from spec) — no `update` or `delete` method on the aggregate.
   Only admin-override soft-delete is allowed. A follow-up
   phase may add a `purge_older_than` method.
4. **No `educore-finance` dep** (carry-over from Phase 8 OQ
   #6, Phase 10 OQ #3, Phase 11 OQ #4, Phase 12 OQ #5, Phase
   13 OQ #3). Phase 14 had no finance touch.
5. **No `educore-notify` dep** (carry-over from Phase 10 OQ
   #4, Phase 11 OQ #4, Phase 12 OQ #6, Phase 13 OQ #4). Phase
   14 had no notify fan-out. **Phase 15 will land the
   `educore-notify` port**, which the operations domain's
   `RecordUserLog` command will eventually fan out to.
6. **No `educore-attendance` dep** (carry-over from Phase 10
   OQ #5, Phase 12 OQ #7, Phase 13 OQ #5). Phase 14 had no
   attendance integration.
7. **No `educore-documents` dep** (carry-over from Phase 11
   OQ #6, Phase 12 OQ #8, Phase 13 OQ #6). Phase 14 had no
   documents integration.
8. **No `educore-academic` dep** (carry-over from Phase 13
   OQ #7). Phase 14 settings uses a local `AcademicYearRef`
   Uuid newtype in `value_objects.rs` (mirror of events-domain
   pattern). A follow-up phase may promote to a true
   cross-crate `AcademicYearId` re-export.
9. **`Sidebar::level` is a closed enum** (NEW) — the spec
   defines `level` as `i32` (1=Parent, 2=Child, 3=SubChild).
   Phase 14 ships a `SidebarLevel` closed enum for type safety
   (mirroring events-domain's `ForWhom` pattern). A follow-up
   phase may add a `From<i32>` conversion for storage-adapter
   compatibility.
10. **OAuth + PasswordReset + migrations tables are
    port-driven** (carry-over from spec) — documented in
    `docs/specs/operations/overview.md` § "Infrastructure
    Tables (Documented, Not Owned)". The 4 port-driven
    repository port traits (`OAuthAccessTokenRepository`,
    `OAuthClientRepository`, `PasswordResetRepository`,
    `MigrationRepository`) are in the operations crate's
    `repository.rs` with `#[allow(dead_code)]` to document the
    contract. **Phase 15 will land the `educore-auth` port**,
    which will implement these.
11. **System-tenant commands bypass capability check** (carry-
    over from spec) — `ScheduleJob`, `MarkJobReserved/
    Completed/Failed`, `RegisterSystemVersion`,
    `RecordVersionHistory`, `RecordUserLog` use
    `TenantContext::system()` and bypass the capability check
    (per `docs/specs/operations/permissions.md` §
    "Authorization Pattern").

## Where NOT to start (Phase 15)

- Do NOT add a `educore-finance` dep (Phase 8 OQ #6 + Phase 10
  OQ #3 + Phase 11 OQ #4 + Phase 12 OQ #5 + Phase 13 OQ #3 +
  Phase 14 OQ #4).
- Do NOT add a `educore-attendance` dep (Phase 10 OQ #5 +
  Phase 12 OQ #7 + Phase 13 OQ #5 + Phase 14 OQ #6).
- Do NOT add a `educore-documents` dep (Phase 11 OQ #6 + Phase
  12 OQ #8 + Phase 13 OQ #6 + Phase 14 OQ #7).
- Do NOT add a `educore-academic` dep (Phase 13 OQ #7 + Phase
  14 OQ #8).
- Do NOT remove the 4 `EventsCalendar*` + 4 `CmsPage*`
  capability placeholders (they were preserved in Phase 13 +
  Phase 12). Do NOT add them back either.
- Do NOT remove the 7 calendar AuditTarget placeholders
  (3 from Phase 2 + 4 from Phase 13). Do NOT add them back
  either.
- Do NOT remove the 2 settings/operations AuditTarget
  placeholders (`SchoolSettings`/`BellSchedule`). Do NOT add
  them back either.
- Do NOT touch the 18 closed crates other than the additive
  rbac + audit extensions + the 2 `Cargo.toml` additions to
  storage-parity. Per `ADR-013-CrateLayout.md`, the
  cross-crate modifications are all non-breaking additive
  (with the documented exception of removing the 2
  SettingsManage/OperationsManage capability placeholders,
  which the Phase 14 hand-off documents as a deliberate
  decision to mirror Phase 13's 7-root interpretation).
- Do NOT touch `educore-core::lint`. The lint binary passes;
  the tier-boundary checker remains a stub.
- Do NOT remove the 4 Phase 2 `CommunicationMessage*`
  capability placeholders or add them back. They were
  deduplicated in Phase 10.
- Do NOT remove the 4 Phase 2 `CmsPage*` capability
  placeholders or add them back. They were preserved in
  Phase 12.
- Do NOT fix the pre-existing clippy debt in finance / hr /
  facilities (Phase 7/6/8 WIP). Document it in
  `docs/progress-tracker.md` and open follow-up PRs.

## Key files for the next agent

- `crates/cross-cutting/settings/src/` — 9-file layout, 9943
  lines, 43 tests passing. The complete template for future
  cross-cutting domain crates.
- `crates/cross-cutting/operations/src/` — 9-file layout, 6716
  lines, 47 tests passing. The template for append-only +
  global-aggregate domains.
- `crates/cross-cutting/rbac/src/value_objects.rs` — line 1136
  (`SettingsManage`/`OperationsManage` REMOVED), lines
  1140-1290 (66 net-new Settings.* caps), lines 1293-1432
  (34 net-new Operations.* caps), line ~1654 (`domain()` match
  arms), line ~2495 (`aggregate()` match arms), line ~3632
  (`as_str()` match arms), line ~4257 (`all_variants` list),
  line ~4885 (`from_str_opt` parser).
- `crates/cross-cutting/audit/src/writer.rs` — line ~362
  (15 net-new Settings.* variants), line ~393 (8 net-new
  Operations.* variants), line ~539 (`target_type()` match
  arms), line ~697 (`target_id()` match arms).
- `crates/tools/storage-parity/Cargo.toml` — the new
  `educore-settings` + `educore-operations` deps.
- `crates/tools/storage-parity/tests/settings_integration.rs` —
  5 + 2 scenarios.
- `crates/tools/storage-parity/tests/operations_integration.rs` —
  5 + 2 scenarios.
- `docs/coverage.toml` — 28 new rows flipped from `Pending` →
  `Tested` (15 settings + 8 operations + 4 cross-crate
  capability/audit target rows + 2 placeholder rows kept).
- `docs/handoff/PHASE-14-HANDOFF.md` — this hand-off.
- `docs/phase_prompt/phase-15-prompt.md` — the next-phase
  prompt (≤ 50 lines, all 8 sections).

## Where to ask

Open a GitHub issue for design questions. The Phase 14 prompt
is the source of truth for Phase 14's scope; the next-phase
prompt is the source of truth for Phase 15's scope. For
disputes, defer to `AGENTS.md` (engine rules) and
`ADR-013-CrateLayout.md` (tier definitions).
