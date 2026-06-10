# 10 — Rollback

## Goal

The migration is **fully reversible** at every phase. The rollback
script reverts every rename, every column add, every type change,
and every brand removal. The script is generated and printed
**before the migration begins**; it is taped to the wall.

## The principle

Phase B (side-by-side + cutover) means the legacy `devdb` is
untouched during the entire migration. The new `devdb_v2` is built
alongside. If `devdb_v2` fails verification (Phase 7), the
rollback is:

1. Drop `devdb_v2` (the broken new database).
2. The legacy `devdb` is still live and unchanged.
3. The consumer's `DATABASE_URL` is repointed at `devdb`.
4. The migration is re-attempted later.

This is the **outer** rollback: discard the broken attempt. It
works at any phase.

The **inner** rollback (per-phase) is the inverse of each phase's
DDL. It is more expensive to script and is documented here for
reference; in practice the outer rollback is preferred.

## Per-phase rollback scripts

### Phase 0 (pre-flight) rollback

Nothing to roll back. The pre-flight is read-only.

### Phase 1 (engine tables) rollback

```sql
USE devdb_v2;

DROP TABLE IF EXISTS outbox;
DROP TABLE IF EXISTS audit_log;
DROP TABLE IF EXISTS idempotency;
DROP TABLE IF EXISTS event_log;
DROP TABLE IF EXISTS schema_registry;
DROP TABLE IF EXISTS system_user;
```

The `system_user` row is the only seeded row; it is dropped with
the table.

### Phase 2 (ID conversion) rollback

For every aggregate table `T`:

```sql
-- 1. Drop the new CHAR(36) PK constraint
ALTER TABLE T DROP PRIMARY KEY;
ALTER TABLE T DROP COLUMN id;

-- 2. Rename id_v7_legacy back to id
ALTER TABLE T CHANGE COLUMN id_v7_legacy id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT;

-- 3. Restore the AUTO_INCREMENT PK
ALTER TABLE T ADD PRIMARY KEY (id);

-- 4. Re-create the BIGINT FK constraints
-- (the BIGINT -> CHAR(36) FKs need to be reverted in the child tables
--  too; this is per-table and is the inverse of Phase 2's per-table
--  forward script)
```

The `id_v7_legacy` column is restored to its pre-migration BIGINT
value. The UUIDv7 values are discarded.

### Phase 3 (table renames) rollback

For every rename in `03-domain-renames.md`:

```sql
RENAME TABLE <engine_table> TO <legacy_table>;
```

For every drop (Laravel meta), the legacy `devdb` already has the
table, so no rollback is needed (it was never created in
`devdb_v2`).

For every archive (`legacy_<name>`), drop the archive:

```sql
DROP TABLE IF EXISTS legacy_<name>;
```

For every consumer-side addition, drop the new table:

```sql
DROP TABLE IF EXISTS platform_tenants;
-- ... etc.
```

### Phase 4 (engine invariants) rollback

For every aggregate table `T`:

```sql
ALTER TABLE T
  DROP COLUMN version,
  DROP COLUMN etag,
  DROP COLUMN last_event_id,
  DROP COLUMN correlation_id,
  DROP COLUMN source;

-- Tighten school_id back to nullable
ALTER TABLE T MODIFY COLUMN school_id CHAR(36) NULL;

-- Drop the (school_id, active_status) index
DROP INDEX idx_<t>_school_active ON T;

-- Restore the parent-anchor FK actions to CASCADE
-- (the inverse of Phase 4's RESTRICT enforcement)
```

The `created_at` and `updated_at` columns are kept as `TIMESTAMP
NOT NULL`. The legacy NULL was a bug; the engine's NOT NULL is
correct.

### Phase 5 (brand removal) rollback

```sql
-- Restore the dropped module_toggles columns
ALTER TABLE settings_general_settings
  ADD COLUMN Lesson TINYINT DEFAULT 0,
  ADD COLUMN Chat TINYINT DEFAULT 0,
  -- ... 35 columns
  ADD COLUMN InAppLiveClass TINYINT DEFAULT 0;

-- Restore the is_saas columns
ALTER TABLE rbac_roles
  ADD COLUMN is_saas INT DEFAULT 0,
  CHANGE COLUMN is_replicated _ignore_drop BOOLEAN;
-- (rename back; the new is_replicated column is dropped and the
--  old is_saas is restored)
```

This is the most-mechanical rollback. In practice the outer rollback
is preferred.

### Phase 6 (field data flow) rollback

The field-level data flow wrote new rows to `devdb_v2`. The
rollback truncates the engine tables and re-runs the ETL with
the inverse transforms. If the ETL has a bug, the forward and
inverse runs produce a different result; manual reconciliation is
needed.

The pragmatic approach: **do not try to roll back Phase 6 in
isolation**. If the field-level data flow is wrong, drop
`devdb_v2` and start over (outer rollback).

### Phase 7 (verification) rollback

Verification is read-only. Nothing to roll back.

### Phase 8 (cutover) rollback

The cutover is a config change. The rollback is:

```bash
# 1. Stop the consumer's HTTP process
systemctl stop educore-backend

# 2. Revert DATABASE_URL to devdb
# (restore the pre-cutover .env from backup)

# 3. Restart the consumer's HTTP process
systemctl start educore-backend

# 4. Verify the app works against devdb

# 5. Notify the school: "We rolled back to the legacy system.
#    No data was lost; the migration attempt has been reverted."
```

Downtime: ~30 seconds.

### Phase 9 (decommission) rollback

Decommission is irreversible. The `devdb` is dropped at T+90d.
Before T+90d, the read-only `devdb` can be made read-write again
with:

```sql
UNLOCK TABLES;
```

This is a one-line rollback. The 90-day window is the safety
margin.

## Rollback timing

The rollback is most reliable when invoked **early** (Phase 1
through Phase 5) because the changes are mostly schema. Once
Phase 6 (field data flow) has run, the new tables hold real data
that the legacy tables don't, and the rollback is more complex.

The pragmatic rule:

| Phase completed | Rollback strategy |
| --- | --- |
| Phase 0–5 | outer rollback: drop `devdb_v2`, retry later |
| Phase 6 | outer rollback: drop `devdb_v2`, retry later |
| Phase 7 (verification failed) | outer rollback: drop `devdb_v2`, retry later |
| Phase 8 (cutover failed) | cutover rollback: re-point `DATABASE_URL` at `devdb` |
| Phase 9 (decommission started) | decommission rollback: re-make `devdb` read-write |

## The pre-scripted rollback

Before T-0, the consumer prints the rollback script and tapes it to
the wall. The script is:

```bash
#!/usr/bin/env bash
# migrations/rollback.sh
# Generated by the consumer's migration tool on T-1d.
# Run this to roll back the migration to devdb_v2.

set -euo pipefail

echo "Rolling back Educore migration to devdb_v2"
echo "================================================"

# 1. Stop the consumer's HTTP process
systemctl stop educore-backend || true
systemctl stop educore-outbox-relay || true
systemctl stop educore-audit-sink || true

# 2. Drop devdb_v2
mysql -e "DROP DATABASE IF EXISTS devdb_v2;"

# 3. Re-point DATABASE_URL to devdb
install -m 0600 .env.legacy .env

# 4. Restart the consumer's HTTP process
systemctl start educore-backend

# 5. Verify the app works
curl -fsS http://localhost:8080/healthz || {
  echo "ERROR: app health check failed; investigate"
  exit 1
}

echo "Rollback complete. devdb is live; devdb_v2 is dropped."
echo "Notify the school: rollback successful, no data loss."
```

The script is **idempotent** and can be run at any phase.

## Rollback verification

After the rollback, run the verification:

```sql
-- The legacy devdb has the original row counts
SELECT
  'sm_students' AS table_name,
  (SELECT COUNT(*) FROM devdb.sm_students) AS legacy_count;

-- A sample row from devdb matches the snapshot taken at T-1d
SELECT * FROM devdb.sm_students WHERE id = <sample_id>;
-- Compare to the snapshot:
-- diff <(mysqldump --where="id=<sample_id>" devdb sm_students) \
--      <(cat snapshots/devdb_<id>.sql)
```

If the sample row matches the snapshot, the rollback is
verified.

## Exit criteria

- `devdb_v2` is dropped (or `devdb` is read-only if decommission
  started).
- `DATABASE_URL` points at `devdb`.
- The consumer's app works against `devdb`.
- The school's IT records note the rollback.
- The migration is re-attempted later with a fix.
