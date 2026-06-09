# 05 — Brand Removal (Phase 5)

## Goal

Remove every InfixEdu / InfixLMS / Schoolify brand artifact from
`devdb_v2`'s schema and column names. The engine has no concept of
"InfixEdu"; the brand is the legacy Laravel project's marketing
name, not an engine domain.

## Renames (8 brand-tainted tables)

| Legacy | Engine | Rationale |
| --- | --- | --- |
| `infix_module_infos` | `platform_module_infos` | `infix_` is the brand; engine uses `platform_` for platform-domain tables |
| `infix_module_managers` | `platform_module_managers` | same |
| `infix_module_student_parent_infos` | `platform_student_parent_menus` | drops `infix_` brand and `infos` suffix (engine convention is bare plural or singular) |
| `infix_roles` | `rbac_roles` | `infix_` is the brand; engine uses `rbac_` for the rbac-domain table. The `InfixRole` shadow-aggregate concept is **deleted entirely** (see below). |
| `infix_permission_assigns` | `rbac_permission_assigns` | same |
| `infixedu__pages` | (drop; replaced by `cms_pages`) | the engine's `cms_pages` is the canonical page aggregate; the `infixedu__` double-underscore is a brand artifact |
| `infixedu__settings` | (drop) | the engine's `cms_pages.settings` (JSON column on the page) is the canonical settings location |
| `infixedu__pages_*` indexes (3) | (drop) | indexes on the dropped table drop with the table |

## Drops (3 brand-only tables)

| Legacy | Reason |
| --- | --- |
| `infixedu__pages` | replaced by `cms_pages` |
| `infixedu__settings` | replaced by `cms_pages.settings` |
| `infix_module_student_parent_infos` | replaced by `platform_student_parent_menus` (semantics preserved; the table is renamed, not dropped) |

The `infix_module_student_parent_infos` rename preserves the data
(it's a UI menu registry used by the legacy frontend). The `infixedu__*`
tables are pure legacy synonyms for engine tables that already
exist.

## Column renames (2 brand-tainted columns)

| Legacy column | Legacy table | New column | New table |
| --- | --- | --- | --- |
| `InfixBiometrics int(11) DEFAULT 0` | `sm_general_settings` | `biometrics_enabled TINYINT(1) NOT NULL DEFAULT 0` | `settings_general_settings` |
| `path_infix_style varchar(255) DEFAULT NULL` | `sm_styles` and `themes` | `path_style varchar(255) DEFAULT NULL` | `settings_styles` and `settings_themes` |

The `module_toggles` flat-int columns (Lesson, Chat, FeesCollection,
InfixBiometrics, ...) on `sm_general_settings` are removed entirely
in Phase 5. The engine's capability-based module system replaces
them: a module is either enabled or not based on the school's
`platform_packages.modules` array (a JSON column on
`platform_packages`). The flat-int flags are not migrated.

```sql
-- Phase 5: drop the flat-int module_toggles columns
ALTER TABLE settings_general_settings
  DROP COLUMN Lesson,
  DROP COLUMN Chat,
  DROP COLUMN FeesCollection,
  DROP COLUMN InfixBiometrics,
  DROP COLUMN ResultReports,
  DROP COLUMN TemplateSettings,
  DROP COLUMN MenuManage,
  DROP COLUMN RolePermission,
  DROP COLUMN RazorPay,
  DROP COLUMN Saas,
  DROP COLUMN StudentAbsentNotification,
  DROP COLUMN ParentRegistration,
  DROP COLUMN Zoom,
  DROP COLUMN BBB,
  DROP COLUMN VideoWatch,
  DROP COLUMN Jitsi,
  DROP COLUMN OnlineExam,
  DROP COLUMN SaasRolePermission,
  DROP COLUMN BulkPrint,
  DROP COLUMN HimalayaSms,
  DROP COLUMN XenditPayment,
  DROP COLUMN Wallet,
  DROP COLUMN Lms,
  DROP COLUMN ExamPlan,
  DROP COLUMN University,
  DROP COLUMN Gmeet,
  DROP COLUMN KhaltiPayment,
  DROP COLUMN Raudhahpay,
  DROP COLUMN AppSlider,
  DROP COLUMN BehaviourRecords,
  DROP COLUMN DownloadCenter,
  DROP COLUMN AiContent,
  DROP COLUMN WhatsappSupport,
  DROP COLUMN InAppLiveClass;
```

This drops 35 columns. The engine's module system is capability-
based, not flag-based.

## The `InfixRole` shadow aggregate — full removal

The engine's RBAC design has one `Role` aggregate per school, with
`is_replicated` flag for cross-tenant (SaaS) replication. There is
no `InfixRole` shadow aggregate. The legacy `infix_roles` table's
`is_saas` flag is the engine's `is_replicated` flag.

**Drops from docs:**

- `docs/specs/rbac/aggregates.md` — `## InfixRole` section
  (lines 189-216).
- `docs/specs/rbac/aggregates.md` — `## InfixPermissionAssign`
  section (lines 218-244).
- `docs/specs/rbac/aggregates.md` — `InfixRole` line in the
  owned-children list (line 19).
- `docs/specs/rbac/aggregates.md` — "an `InfixRole` shadow" in the
  `CreateRole` effects (line 32).
- `docs/specs/rbac/aggregates.md` — Invariant 7 about `is_saas`
  (line 35-36) is rewritten to `is_replicated`.
- `docs/specs/rbac/repositories.md` — `## InfixRoleRepository`
  section (lines 112-123).
- `docs/specs/rbac/repositories.md` — `## InfixPermissionAssignRepository`
  section (lines 125-135).
- `docs/specs/rbac/repositories.md` — index
  `ix_infix_roles_school_id_saas` (line 193) is rewritten to
  `ix_rbac_roles_school_replicated`.
- `docs/specs/rbac/value-objects.md` — `InfixRoleId` and
  `InfixPermissionAssignId` rows (lines 18-19).
- `docs/specs/rbac/tables.md` — `infix_permission_assigns` and
  `infix_roles` rows (lines 10-11, 37, 47).
- `docs/specs/rbac/overview.md` — `InfixRole` and
  `InfixPermissionAssign` rows (lines 80-81).
- `docs/specs/rbac/commands.md` — "an `InfixRole` shadow" in
  `CreateRole` effects (line 32).
- `docs/specs/hr/aggregates.md` — Invariants 3 in `Department` and
  `Designation` (lines 92, 123) — rewrite `is_saas` to
  `is_system_defined`.
- `docs/specs/hr/tables.md` — `is_saas` notes (line 38).
- `docs/specs/hr/commands.md` — `is_saas` lines (184, 202).
- `docs/specs/operations/aggregates.md` — `is_saas` line (265).
- `docs/specs/operations/commands.md` — `CreateSidebarEntryCommand.is_saas`
  (line 297) — rewrite to `is_system_defined`.
- `docs/research/rbac-analysis.md` — InfixRole/InfixPermissionAssign
  note (line 32) — full removal.

**Schema-level changes:**

- `infix_roles.is_saas INT(10) UNSIGNED DEFAULT 0` →
  `rbac_roles.is_replicated BOOLEAN NOT NULL DEFAULT FALSE`.
  Backfill: `UPDATE rbac_roles SET is_replicated = TRUE WHERE
  is_saas = 1`.
- `assign_permissions.is_saas TINYINT(4) NOT NULL DEFAULT 0` →
  `rbac_permission_assigns.is_replicated BOOLEAN NOT NULL DEFAULT
  FALSE`. Same backfill.
- `sm_human_departments.is_saas` → `hr_departments.is_system_defined`.
- `sm_designations.is_saas` → `hr_designations.is_system_defined`.
- `sm_module_permissions.is_saas` → `rbac_module_permissions.is_system_defined`.
- `sidebars.is_saas` → `rbac_sidebars.is_system_defined`.
- `infix_module_managers.is_saas` → `platform_module_managers.is_system_defined`.

The `is_saas` semantics is **preserved** in the rename: the new
`is_replicated` / `is_system_defined` flags carry the same meaning.
Only the column name changes.

## Typo fixes

| Legacy | Fixed |
| --- | --- |
| `continets` (table, 1 occurrence in `migrations/0014_platform.sql`) | `continents` |
| `transcations` (table, 1 occurrence in `migrations/0009_finance.sql`) | (drop; table is empty) |
| `metarnity_leave` (column in `sm_staffs`) | `maternity_leave_quota` |
| `twiteer_url` (column in `sm_staffs`) | `twitter_url` |
| `instragram_url` (column in `sm_staffs`) | `instagram_url` |
| `merital_status` (column in `sm_staffs`) | `marital_status` |
| `bank_brach` (column in `sm_staffs`) | `bank_branch` |
| `fm_fees_invoice_chields` (table, 1 occurrence) | `finance_invoice_children` |
| `fm_fees_transcation_children` (table, 1 occurrence) | `finance_fees_transcation_children` (the typo in `transcation` is also fixed; the table itself is preserved) |
| `sm_staff_attendences` (table, 1 occurrence) | `attendance_staff_attendances` |
| `sm_frontend_persmissions` (table, 1 occurrence) | `platform_frontend_permissions` |

The typo list is exhaustive per the legacy schema inventory.

## The InfixLMS module-toggles JSON alternative

The `module_toggles` flat-int columns on `sm_general_settings` are
replaced by a single JSON column on `platform_packages`:

```sql
ALTER TABLE platform_packages
  ADD COLUMN modules JSON NOT NULL DEFAULT '[]';

-- Backfill: every existing school's enabled modules are stored as
-- a JSON array of module names
UPDATE platform_packages pp
SET modules = (
  SELECT JSON_ARRAYAGG(name) FROM (
    SELECT 'Lesson' AS name WHERE (SELECT Lesson FROM sm_general_settings WHERE school_id = ...) = 1
    UNION ALL SELECT 'Chat' WHERE ...
    -- ... 35 modules
  ) AS enabled
);
```

The consumer's UI reads `platform_packages.modules` to render
module toggles in the admin panel.

## Aggregate count

| Statistic | Count |
| --- | --- |
| Brand-tainted tables renamed | 5 (`infix_*` to engine names) |
| Brand-tainted tables dropped | 2 (`infixedu__pages`, `infixedu__settings`) |
| Brand columns renamed | 2 (`InfixBiometrics`, `path_infix_style`) |
| `is_saas` columns renamed to `is_replicated` / `is_system_defined` | 6 |
| Module-toggle flat-int columns dropped | 35 |
| Typo fixes | 11 |
| Doc edits (full removal) | 16 files |

## Exit criteria

- No table or column in `devdb_v2` contains `infix`, `infixedu`,
  `InfixBiometrics`, or `path_infix_style`.
- No reference to `is_saas` in `devdb_v2` (the `is_replicated` and
  `is_system_defined` flags carry the same meaning).
- The 16 spec/doc files have no InfixRole / InfixPermissionAssign /
  infixedu__* / path_infix_style / sm_* / is_saas references.
- A grep for the brand name in the entire `migrations/` and `docs/`
  tree returns zero matches.
