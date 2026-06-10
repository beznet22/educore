# Data Migration — Index

This folder is the **single source of truth** for the migration of
the legacy Schoolify/InfixEdu database (`devdb`) to the Educore
schema. The parent `migrations/README.md` summarises the gap and the
naming convention; the files here are the playbook.

## Structure

| File | Phase | What it does |
| --- | --- | --- |
| `00-overview.md` | — | the strategy (Option B), the security context, the phase list |
| `01-engine-tables.md` | 1 | the six engine cross-cutting tables (DDL in `migrations/engine/0000_engine_core.mysql.sql`) |
| `02-id-conversion.md` | 2 | `BIGINT AUTO_INCREMENT` → `CHAR(36)` UUIDv7 strategy |
| `03-domain-renames.md` | 3 | the 310-row table rename map (legacy → engine, by domain) |
| `04-column-additions.md` | 4 | engine invariants added to every aggregate (`version`, `etag`, `last_event_id`, `correlation_id`, `source`, `created_by`, `updated_by`) |
| `05-brand-removal.md` | 5 | InfixEdu brand removal: `infix_*`, `infixedu__*`, `InfixBiometrics` column, `path_infix_style` field, `InfixRole` shadow aggregate, `is_saas` flag, typos |
| `06-field-data-flow.md` | 6 | the 15-table column-by-column field map (the actual transform from legacy column → engine column) |
| `07-verification.md` | 7 | row counts, FK integrity, sample row content, parity test against the engine's repositories |
| `08-cutover.md` | 8 | application cutover to the new database |
| `09-decommission.md` | 9 | 30-day archive of the legacy database |
| `10-rollback.md` | — | pre-scripted inverse of every rename and add-column |
| `11-security.md` | — | credential rotation, git history purge |

Read `00-overview.md` first. Read `11-security.md` in parallel — the
credential in git history is independent of the schema work and must
be addressed first.

## How to read this folder

- **In order** if you are executing the migration.
- **Out of order** if you are reviewing a specific phase. Each file
  is self-contained; cross-references are by file name.

## Convention

- **Backticks around identifiers**: `sm_students`, `academic_students`.
- **Backticks around column names**: `id`, `school_id`, `version`.
- **Backticks around SQL keywords**: `SELECT`, `CHAR(36)`, `CASCADE`.
- **No prose around code blocks**: code is canonical, prose explains
  intent.

## Status

This folder is **planning only**. The SQL emitted by
`migrations/engine/0000_engine_core.mysql.sql` is the only executable artifact.
All other documents describe work that the consumer performs during
the implementation phase. The engine does not run any of these
migrations; the consumer's migration runner does.
