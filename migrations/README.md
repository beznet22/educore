# Migrations

This folder holds the **engine's** migration set. The migrations are
**owned by the consumer**, not by the engine library. The engine's
storage adapters (`educore-storage-mysql`, `educore-storage-sqlite`,
`educore-storage-postgres`) emit DDL that conforms to the schemas
specified here.

## Folder layout

This folder contains:

- `README.md` (this file) — engine target schema, gap, plan
- `engine/` — canonical DDL for the 6 engine cross-cutting tables, in 3 dialects (MySQL, PostgreSQL, SQLite). Adapters embed these via `include_str!`.
- `0001_*.sql` through `0015_*.sql` — legacy Schoolify/InfixEdu dump (research source only, not engine schema)

## Current state

The 15 numbered files (`migrations/0001_academic.sql` through
`migrations/0015_settings.sql`) are a **dump of the legacy Schoolify /
InfixEdu Laravel project**. They are kept in the repository as a
**research source** (the engine's domain shapes were extracted from
this dump), not as the engine's target schema.

| Statistic | Value |
| --- | --- |
| Migration files | 15 |
| Total tables | 310 |
| Total columns | ~3,500 |
| Total foreign keys | ~1,200 |
| Total indexes | ~1,800 |
| Engine-invariant columns present | `active_status` only (~85% of tables) |
| Engine-invariant columns missing | `version`, `etag`, `last_event_id`, `correlation_id`, `source`, `created_by`/`updated_by` (on most tables) |
| Brand-tainted tables | 8 (`infix_*`, `infixedu__*`, `InfixBiometrics` column) |
| Misspelled tables | 1 (`continets`) |
| Misspelled columns | 1 (`path_infix_style`) |
| Laravel-default tables (not engine) | ~10 (`users`, `roles`, `permissions`, `cache`, `jobs`, etc.) |
| Live-database credential in git history | **YES** — `paxxw0rd@2791` in commit `5fa148c` for `mysql://devuser:...@127.0.0.1:3306/devdb`. **Rotate and purge before any public push.** |

The legacy dump is **not** the engine schema. See
`docs/schemas/database-schema.md` for the engine's invariants, and
`docs/schemas/data-migration/` for the migration plan from legacy to
engine.

## Naming convention (locked in)

The engine guarantees **table parity**: the same table name in MySQL,
SQLite, and PostgreSQL. Consumers do not choose different names per
backend; they choose the same name and let the storage adapter emit
the right dialect.

### Aggregate tables

```text
<domain>_<aggregate>
```

- `<domain>` is one of the 15 engine bounded contexts, lowercase
  singular or short form (see below).
- `<aggregate>` is the singular or plural noun of the root, snake_case.
- The two are joined by a single underscore.
- No prefix. No brand. No `sm_`, `front_`, `check_`, `un_`, `infix_`,
  `infixedu__`, `fm_`.

Examples (15 of 310):

| Legacy | Engine (MySQL/SQLite/PostgreSQL) | Domain |
| --- | --- | --- |
| `sm_students` | `academic_students` | academic |
| `sm_schools` | `platform_schools` | platform |
| `users` | `platform_users` | platform |
| `sm_exams` | `assessment_exams` | assessment |
| `fm_fees_invoices` | `finance_invoices` | finance |
| `infix_roles` | `rbac_roles` | rbac |
| `infixedu__pages` | `cms_pages` | cms |

### The 15 domains

| Domain | `<domain>` |
| --- | --- |
| Academic | `academic` |
| Assessment | `assessment` |
| Attendance | `attendance` |
| Communication | `communication` |
| Documents | `documents` |
| Events | `events` |
| Facilities | `facilities` |
| Finance | `finance` |
| Human Resources | `hr` |
| Library | `library` |
| Content Management | `cms` |
| RBAC | `rbac` |
| Platform | `platform` |
| Settings | `settings` |
| Operations | `operations` |

### Engine cross-cutting tables

Unprefixed. Identical name in all three backends. The `educore-events-domain` crate
is the engine's events-bounded-context (calendar, holidays, incidents)
and uses the `events` prefix for its domain tables; the `outbox` /
`audit_log` / `idempotency` / `event_log` / `schema_registry` /
`system_user` tables are engine-internal and have no prefix.

| Logical | MySQL / SQLite / PostgreSQL |
| --- | --- |
| Outbox | `outbox` |
| Audit log | `audit_log` |
| Idempotency | `idempotency` |
| Event log | `event_log` |
| Schema registry | `schema_registry` |
| System user | `system_user` |

A PostgreSQL consumer MAY wrap these in an `engine` schema
(`engine.outbox`, etc.) and set `search_path = engine, public` per
session. The engine's adapter handles the translation; the consumer's
choice is invisible to the engine. See
`docs/schemas/sql-dialects/postgresql.md`.

The canonical SQL for these six tables lives in
`migrations/engine/0000_engine_core.<dialect>.sql` (one file per
supported dialect).

### Reserved column names

Per `docs/schemas/database-schema.md` § 12, the following column names
are reserved on every aggregate-bearing table. They MUST NOT be used
for any other purpose.

```text
id              school_id           academic_id         record_id
session_id      version             etag                active_status
created_at      updated_at          created_by           updated_by
last_event_id
```

### ID type: UUIDv7

Per `docs/schemas/database-schema.md` § 1.4 and
`docs/schemas/tenancy-schema.md` § 2, every identifier is a
**time-ordered UUIDv7** stored as `CHAR(36)`. The `INT AUTO_INCREMENT`
convention inherited from Laravel is replaced. The migration strategy
(BIGINT → UUIDv7) is detailed in
`docs/schemas/data-migration/02-id-conversion.md`.

### Engine invariants on every aggregate

Per `docs/schemas/database-schema.md` § 2, § 5, § 9, the canonical
minimum schema for any new aggregate table:

```sql
CREATE TABLE <table> (
    id              CHAR(36)     PRIMARY KEY,
    school_id       CHAR(36)     NOT NULL,
    active_status   TINYINT      NOT NULL DEFAULT 1,
    created_at      TIMESTAMP    NOT NULL,
    updated_at      TIMESTAMP    NOT NULL,
    created_by      CHAR(36)     NOT NULL,
    updated_by      CHAR(36)     NOT NULL,
    version         BIGINT       NOT NULL DEFAULT 1,
    etag            CHAR(32)     NOT NULL,
    last_event_id   CHAR(36)         NULL,
    correlation_id  CHAR(36)         NULL,
    source          VARCHAR(16)      NULL,
    -- aggregate-specific columns --
    CONSTRAINT fk_<table>_school FOREIGN KEY (school_id)
        REFERENCES platform_schools (id) ON DELETE RESTRICT
);

CREATE INDEX idx_<table>_school_active
    ON <table> (school_id, active_status);
```

Dialect-specific forms (MySQL, SQLite, PostgreSQL) are in
`docs/schemas/sql-dialects/`.

## The gap between current state and engine target

| Concern | Current state (legacy dump) | Engine target |
| --- | --- | --- |
| Table names | `sm_*`, `infix_*`, `infixedu__*`, `fm_*`, `front_*`, `check_*`, `un_*`, plus Laravel defaults (`users`, `roles`, `cache`, etc.) | `<domain>_<aggregate>` (15 domains) or unprefixed (engine cross-cutting) |
| ID type | `BIGINT UNSIGNED AUTO_INCREMENT` or `INT(10) UNSIGNED AUTO_INCREMENT` (Laravel convention) | `CHAR(36)` carrying UUIDv7 |
| Tenant anchor | `school_id INT UNSIGNED DEFAULT 1` (nullable in many tables) | `school_id CHAR(36) NOT NULL` + RLS |
| Audit columns | `created_at` + `updated_at` only | + `created_by`, `updated_by`, `version`, `etag`, `last_event_id`, `correlation_id`, `source` |
| Soft delete | `active_status TINYINT DEFAULT 1` (present in ~85% of tables) | `active_status TINYINT NOT NULL DEFAULT 1` (mandatory) |
| Brand artifacts | `infix_*`, `infixedu__*`, `InfixBiometrics`, `path_infix_style`, `InfixRole` shadow aggregate, `is_saas` flag | None; renamed to engine domain prefixes; `is_saas` becomes `is_replicated` on the engine's `Role` |
| Engine cross-cutting tables | none | `outbox`, `audit_log`, `idempotency`, `event_log`, `schema_registry`, `system_user` |
| RLS | none | PG: `CREATE POLICY`; MySQL: app-layer filter; SQLite: app-layer filter |
| Typos | `continets` (continents), `transcations` (transactions), `metarnity_leave`, `twiteer_url`, `instragram_url`, `merital_status`, `fron_academic_calendars`, `un_*` columns | spelling fixed at migration |

## The plan

Detailed in `docs/schemas/data-migration/`. The plan has 11 phases
across 12 focused files plus a `00-overview.md` narrative:

| File | Phase | Action |
| --- | --- | --- |
| `00-overview.md` | — | strategy, security, phases summary |
| `01-engine-tables.md` | 1 | the six engine cross-cutting tables (`migrations/engine/`) |
| `02-id-conversion.md` | 2 | BIGINT → UUIDv7 strategy |
| `03-domain-renames.md` | 3 | the 310-row table rename map |
| `04-column-additions.md` | 4 | engine invariants added to every aggregate |
| `05-brand-removal.md` | 5 | InfixEdu / `infix_*` / typos |
| `06-field-data-flow.md` | 6 | the 15-table column-by-column field map |
| `07-verification.md` | 7 | row counts, FK integrity, parity |
| `08-cutover.md` | 8 | application cutover to new database |
| `09-decommission.md` | 9 | 30-day archive |
| `10-rollback.md` | — | pre-scripted inverse |
| `11-security.md` | — | credential rotation, history purge |

The strategy committed in this plan is **Option B: side-by-side +
cutover**. The legacy database `devdb` stays live; a new `devdb_v2`
receives the engine schema; ETL copies the data with the field-level
transforms in `06-field-data-flow.md`; the consumer's application
cuts over; 30 days later `devdb` is archived.

## Engine cross-cutting DDL

The 6 engine cross-cutting tables have canonical DDL in three dialects
under `migrations/engine/`:

| File | Dialect | Use |
| --- | --- | --- |
| `engine/0000_engine_core.mysql.sql` | MySQL 8+ | InnoDB, `utf8mb4_unicode_ci`, backtick identifiers |
| `engine/0000_engine_core.postgres.sql` | PostgreSQL 14+ | `engine` schema, native `UUID`/`JSONB`/`TIMESTAMPTZ`, `CREATE POLICY` RLS |
| `engine/0000_engine_core.sqlite.sql` | SQLite 3.x | `TEXT` for ids, ISO 8601 timestamps, no RLS |

The `educore-storage-<db>` adapter crates `include_str!` these
files at compile time and emit the dialect-specific DDL at
`storage.create_schema()` time. The MySQL file is also the
authoritative reference for the 6 tables; the PG and SQLite files
are translations of that reference into their respective dialects.

The legacy Schoolify/InfixEdu dump (`0001_*.sql` through `0015_*.sql`)
is **research source only** — it is the Schoolify project, not the
engine schema. The legacy→engine map is in
`docs/schemas/data-migration/03-domain-renames.md`.

## Open questions for the consumer

1. **Laravel meta tables** (`migrations`, `cache`, `cache_locks`,
   `failed_jobs`, `personal_access_tokens`, `jobs`, `job_batches`,
   `telescope_entries`, `password_resets`, `oauth_*`, `sessions`):
   **drop** (engine doesn't model them; the Laravel app that is
   replaced by Educore doesn't need them) or **keep** (preserve
   Laravel's auth pipeline during the transition)? Recommendation:
   drop. The engine's `platform_sessions` and `platform_password_resets`
   replace them.

2. **Consumer-side tables to add** (not in any legacy dump; the
   engine provides the domain but not the SaaS layer):
   - `platform_tenants` (consumer SaaS workspace; multiple schools per
     workspace)
   - `platform_subscriptions` (Stripe sync)
   - `platform_packages` (plan catalog)
   - `platform_regions` (multi-region routing)
   - `platform_invitations` (onboarding)
   These are documented in `docs/architecture.md` and
   `docs/guides/saas-backend.md`. They are added in the same migration
   window as the engine tables.

3. **ID-column transitional `id_v7_legacy BIGINT UNSIGNED NULL`**: keep
   for 90 days, then drop. The deterministic UUIDv7 derivation
   (`uuid_v7(legacy_id, table_namespace)`) is idempotent; re-running
   the migration is a no-op. See
   `docs/schemas/data-migration/02-id-conversion.md`.

## Security reminder

The live `devdb` password is in git history (commit `5fa148c`).
**Before any public push, or before the implementation phase begins:**

1. **Rotate** the `devuser` password on the live MySQL.
2. **Purge** the credential from git history with `git filter-repo`
   (or BFG).
3. **Add** the new password to your password manager, not to a
   committed `.env`.
4. **Verify** the purge with `git log -p -- .env | grep -i paxxw0rd`
   (or equivalent) — it should return nothing.

The detailed security checklist is in
`docs/schemas/data-migration/11-security.md`.

## See also

- `migrations/engine/README.md` — index of the 3 dialect DDL files
- `docs/schemas/sql-dialects/README.md` § Runtime DDL emission — how the adapter uses these files
- `docs/build-plan.md` § Phase 0 — when this DDL is emitted
- `docs/schemas/data-migration/` — the legacy→engine map
