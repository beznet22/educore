# 07 — Verification (Phase 7)

## Goal

Confirm that the migration from `devdb` to `devdb_v2` is lossless
and that the engine's storage adapters can read and write the new
schema.

## Verification matrix

| Check | Method | Pass criterion |
| --- | --- | --- |
| Row count per table | `SELECT COUNT(*)` on every engine table | equals `devdb` row count (modulo drops and merges) |
| Row count per FK | engine FK join | every engine FK has a target row |
| Column type conformance | `INFORMATION_SCHEMA.COLUMNS` | every column matches the type spec in `04-column-additions.md` |
| Engine invariants | `INFORMATION_SCHEMA.COLUMNS` | every aggregate has `version`, `etag`, `last_event_id`, `correlation_id`, `source`, `created_by`, `updated_by`, `active_status`, `school_id NOT NULL` |
| Index conformance | `SHOW INDEX` | every aggregate has `(school_id, active_status)` and `(last_event_id)` and `(correlation_id)` |
| FK action conformance | `INFORMATION_SCHEMA.REFERENTIAL_CONSTRAINTS` | every parent-anchor FK is `RESTRICT` (not `CASCADE`) |
| Brand conformance | grep against `INFORMATION_SCHEMA.TABLES` and `COLUMNS` | zero rows containing `infix`, `infixedu`, `InfixBiometrics`, `path_infix_style`, `is_saas` |
| UUIDv7 derivation | sample 50 rows per domain, recompute `uuid_v7(namespace, id_v7_legacy)` | matches the engine's `id` column |
| Storage adapter test | run the engine's repository-trait smoke tests against `devdb_v2` | every method passes |
| Parity test | run the engine's parity test suite (all repositories, all methods) | every method passes across the three backends |

## Row count verification

```sql
-- For each engine table, count rows and compare to legacy
SELECT
  'platform_users' AS engine_table,
  (SELECT COUNT(*) FROM devdb.users) AS legacy_count,
  (SELECT COUNT(*) FROM devdb_v2.platform_users) AS engine_count,
  (SELECT COUNT(*) FROM devdb.users) - (SELECT COUNT(*) FROM devdb_v2.platform_users) AS diff
WHERE diff != 0;
```

A non-zero diff is a **migration bug**, not a documented drop. The
script reports all diffs > 0 for review.

The expected diffs (where they should be non-zero):

- `infixedu__pages`, `infixedu__settings`, `cache`, `cache_locks`,
  `failed_jobs`, `jobs`, `job_batches`, `personal_access_tokens`,
  `migrations` (Laravel), `transcations` (typo) — dropped; engine
  count is 0; legacy count varies.
- `infix_module_student_parent_infos` → `platform_student_parent_menus`
  (rename) — engine count should equal legacy count.
- All other tables — engine count equals legacy count.

## FK integrity

```sql
-- For every engine table with FKs, verify that every referenced row exists
SELECT
  CONCAT('fk_integrity_', fk.name) AS check_name,
  fk.table_name AS child_table,
  fk.referenced_table_name AS parent_table,
  COUNT(*) AS orphan_count
FROM information_schema.referential_constraints fk
JOIN information_schema.key_column_usage kcu
  ON fk.constraint_schema = kcu.constraint_schema
  AND fk.constraint_name = kcu.constraint_name
JOIN devdb_v2.<child_table> c
  ON c.<fk_column> IS NOT NULL
LEFT JOIN devdb_v2.<parent_table> p
  ON c.<fk_column> = p.id
WHERE p.id IS NULL
GROUP BY fk.name, fk.table_name, fk.referenced_table_name;
```

A non-zero `orphan_count` is a **migration bug** — the FK column
was not properly backfilled.

## Engine invariant conformance

```sql
-- Every aggregate has the 7 engine invariants
SELECT
  t.table_name,
  CASE WHEN MAX(CASE WHEN c.column_name = 'version' THEN 1 ELSE 0 END) = 0 THEN 'MISSING' ELSE 'ok' END AS version_col,
  CASE WHEN MAX(CASE WHEN c.column_name = 'etag' THEN 1 ELSE 0 END) = 0 THEN 'MISSING' ELSE 'ok' END AS etag_col,
  CASE WHEN MAX(CASE WHEN c.column_name = 'last_event_id' THEN 1 ELSE 0 END) = 0 THEN 'MISSING' ELSE 'ok' END AS last_event_id_col,
  CASE WHEN MAX(CASE WHEN c.column_name = 'correlation_id' THEN 1 ELSE 0 END) = 0 THEN 'MISSING' ELSE 'ok' END AS correlation_id_col,
  CASE WHEN MAX(CASE WHEN c.column_name = 'source' THEN 1 ELSE 0 END) = 0 THEN 'MISSING' ELSE 'ok' END AS source_col,
  CASE WHEN MAX(CASE WHEN c.column_name = 'active_status' THEN 1 ELSE 0 END) = 0 THEN 'MISSING' ELSE 'ok' END AS active_status_col,
  CASE WHEN MAX(CASE WHEN c.column_name = 'school_id' THEN 1 ELSE 0 END) = 0 THEN 'MISSING' ELSE 'ok' END AS school_id_col
FROM information_schema.tables t
JOIN information_schema.columns c
  ON t.table_schema = 'devdb_v2'
  AND c.table_schema = 'devdb_v2'
  AND t.table_name = c.table_name
WHERE t.table_schema = 'devdb_v2'
  AND t.table_type = 'BASE TABLE'
  AND t.table_name NOT IN (
    'outbox', 'audit_log', 'idempotency', 'event_log',
    'schema_registry', 'system_user'
  )
GROUP BY t.table_name
HAVING version_col = 'MISSING'
   OR etag_col = 'MISSING'
   OR last_event_id_col = 'MISSING'
   OR correlation_id_col = 'MISSING'
   OR source_col = 'MISSING'
   OR active_status_col = 'MISSING'
   OR school_id_col = 'MISSING';
```

A non-empty result is a **migration bug**.

## Brand conformance

```bash
# Zero matches expected
mysql devdb_v2 -e "
SELECT table_name, column_name
FROM information_schema.columns
WHERE table_schema = 'devdb_v2'
  AND (column_name LIKE '%infix%'
    OR column_name LIKE '%Infix%'
    OR column_name = 'is_saas'
    OR column_name = 'path_infix_style'
    OR column_name = 'InfixBiometrics')
UNION ALL
SELECT table_name, NULL
FROM information_schema.tables
WHERE table_schema = 'devdb_v2'
  AND table_name LIKE '%infix%'
UNION ALL
SELECT table_name, NULL
FROM information_schema.tables
WHERE table_schema = 'devdb_v2'
  AND table_name = 'continets';
"
```

Empty result is the pass criterion.

## UUIDv7 derivation verification

```sql
-- Sample 50 rows per domain, recompute UUIDv7, compare
SELECT
  t.table_name,
  c.id_v7_legacy,
  c.id AS engine_id,
  -- The deterministic UUIDv7 derivation: see 02-id-conversion.md
  UUID_FROM_BIN(
    CONCAT(
      UNHEX(LPAD(HEX(UNIX_TIMESTAMP() * 1000), 12, '0')),  -- simplified
      -- ... full derivation per 02-id-conversion.md
    )
  ) AS expected_id,
  CASE WHEN c.id = UUID_FROM_BIN(...) THEN 'MATCH' ELSE 'MISMATCH' END
FROM devdb_v2.<table> c
WHERE c.id_v7_legacy IS NOT NULL
ORDER BY RAND()
LIMIT 50;
```

All 50 rows must be `MATCH`.

In practice this is a property of the ETL script; the verification
queries the sample rows and recomputes the UUIDv7 in SQL or in a
small Python script.

## Storage adapter test

The engine's `smsengine-storage-<db>` adapters include a smoke
test that runs the repository trait against a seeded database.
This is the same test the engine's CI runs against PostgreSQL,
MySQL, and SQLite. The consumer runs it against `devdb_v2`:

```bash
cargo test -p smsengine-storage-mysql -- --test-threads=1
cargo test -p smsengine-storage-postgres -- --test-threads=1
cargo test -p smsengine-storage-sqlite -- --test-threads=1
```

All three must pass against `devdb_v2`. The test uses the
consumer's `DATABASE_URL` to point at the right database.

## Parity test

The engine's parity test suite is in
`crates/tools/storage-parity/` (added in v1 scaffold). It runs
every repository method against a seeded database and verifies
identical results across adapters. The consumer runs it against
`devdb_v2` after the migration is complete.

## Sample integrity check

A more rigorous check: pick 5 random rows per domain and dump
the legacy row and the engine row side-by-side. Every column that
should match (after transform) does match.

```sql
-- For each domain, 5 random rows
SELECT
  'academic_students' AS table_name,
  devdb.sm_students.id AS legacy_id,
  devdb_v2.academic_students.id_v7_legacy AS engine_legacy_id,
  devdb_v2.academic_students.id AS engine_id,
  devdb.sm_students.first_name AS legacy_first_name,
  devdb_v2.academic_students.first_name AS engine_first_name,
  CASE WHEN devdb.sm_students.first_name = devdb_v2.academic_students.first_name THEN 'MATCH' ELSE 'MISMATCH' END AS check
FROM devdb.sm_students
JOIN devdb_v2.academic_students
  ON devdb_v2.academic_students.id_v7_legacy = devdb.sm_students.id
ORDER BY RAND()
LIMIT 5;
```

The script is templated for all 15 priority tables and produces a
side-by-side report.

## Exit criteria

- Row counts match (modulo documented drops).
- FK integrity is zero-orphan.
- Engine invariants are present on every aggregate.
- Brand artifacts are gone.
- UUIDv7 derivation is correct on a 50-row-per-domain sample.
- Storage adapter tests pass against all three backends.
- Parity test passes.
- Sample integrity check is zero-mismatch.

If any check fails, the migration is **rolled back** per
`10-rollback.md` and the bug is fixed before retry.

## Sample report

After the verification script runs, it produces a report:

```text
SMSengine Migration Verification Report
========================================

Domain         | Tables | Rows    | FK Orphans | Invariants | Brand  | UUIDv7 | Adapter | Parity
---------------|--------|---------|------------|------------|--------|--------|---------|-------
academic       |     50 |  12,847 |          0 |       ok   |   ok   |   ok   |   ok    |   ok
assessment     |     43 |   8,234 |          0 |       ok   |   ok   |   ok   |   ok    |   ok
attendance     |      7 |   3,121 |          0 |       ok   |   ok   |   ok   |   ok    |   ok
communication  |     23 |   1,045 |          0 |       ok   |   ok   |   ok   |   ok    |   ok
documents      |      3 |      87 |          0 |       ok   |   ok   |   ok   |   ok    |   ok
events         |      7 |     421 |          0 |       ok   |   ok   |   ok   |   ok    |   ok
facilities     |     15 |     932 |          0 |       ok   |   ok   |   ok   |   ok    |   ok
finance        |     47 |  21,432 |          0 |       ok   |   ok   |   ok   |   ok    |   ok
hr             |     14 |     234 |          0 |       ok   |   ok   |   ok   |   ok    |   ok
library        |      4 |   1,234 |          0 |       ok   |   ok   |   ok   |   ok    |   ok
cms            |     20 |     876 |          0 |       ok   |   ok   |   ok   |   ok    |   ok
rbac           |     10 |     432 |          0 |       ok   |   ok   |   ok   |   ok    |   ok
platform       |     38 |     198 |          0 |       ok   |   ok   |   ok   |   ok    |   ok
settings       |     14 |      14 |          0 |       ok   |   ok   |   ok   |   ok    |   ok
operations     |     15 |     567 |          0 |       ok   |   ok   |   ok   |   ok    |   ok

Total: 310 tables, ~52,000 rows, 0 errors.
```

The report is the sign-off artifact for the migration.
