# 01 — Engine Cross-Cutting Tables (Phase 1)

## Goal

Apply `migrations/engine/0000_engine_core.mysql.sql` to the new database
(`devdb_v2`) before any other migration. This creates the six
engine-internal tables and seeds the `system_user` row.

## What the file does

The full DDL is in `migrations/engine/0000_engine_core.mysql.sql`. The summary:

| Table | Purpose | Doc reference |
| --- | --- | --- |
| `outbox` | transactional event publication | `event-schema.md` § 8 |
| `audit_log` | append-only compliance trail | `audit-schema.md` § 13 |
| `idempotency` | command replay safety | `command-schema.md` § 6 |
| `event_log` | retained events for replay | `event-schema.md` § 8, § 9 |
| `schema_registry` | event-type schema catalog | `event-schema.md` § 7 |
| `system_user` | the SYSTEM actor | `database-schema.md` § 2 |

The tables are **unprefixed** and **identical in name** across MySQL,
SQLite, and PostgreSQL — this is the engine's table-parity contract.
Dialect-specific forms are documented in
`docs/schemas/sql-dialects/`.

## Why this is the first migration

1. The engine's `System` actor (the `system_user` row) is referenced
   by every aggregate's `created_by` / `updated_by` when the system
   itself is the actor (background jobs, the outbox relay, the seed
   itself). The row must exist before any aggregate is inserted.
2. The `outbox` table is written to by every state-changing command
   in the same transaction as the aggregate. If the outbox is missing,
   the engine cannot dispatch any command.
3. The `idempotency` table is consulted before every command. If it
   is missing, the engine cannot guarantee replay safety.
4. The `audit_log` is written to by every command. If it is missing,
   the engine's audit sink fails.

Phase 1 has no data movement and no risk. It is a prerequisite for
every other phase.

## Apply order

### MySQL / SQLite (via `devdb_v2`)

```bash
mysql -u smsengine -p devdb_v2 < migrations/engine/0000_engine_core.mysql.sql
```

The DDL uses `CREATE TABLE IF NOT EXISTS` so re-running is safe.

### PostgreSQL (via `devdb_v2`)

```bash
psql -U smsengine -d devdb_v2 -f migrations/engine/0000_engine_core.mysql.sql
```

The PostgreSQL DDL differs only in identifier quoting (`"outbox"`
vs `` `outbox` ``) and the `JSON` type vs `JSONB`. The
`migrations/engine/0000_engine_core.mysql.sql` is the MySQL form; the consumer
emits the PostgreSQL form via the storage adapter or runs the
dialect-specific DDL in `docs/schemas/sql-dialects/postgresql.md`.

## Verify Phase 1

```sql
-- Six tables exist
SELECT table_name FROM information_schema.tables
WHERE table_schema = 'devdb_v2'
  AND table_name IN (
    'outbox', 'audit_log', 'idempotency',
    'event_log', 'schema_registry', 'system_user'
  );

-- system_user has the seeded row
SELECT id, display_name, active_status
FROM system_user
WHERE id = '00000000-0000-7000-8000-000000000001';

-- Indexes exist
SHOW INDEX FROM outbox;
SHOW INDEX FROM audit_log;
SHOW INDEX FROM idempotency;
```

## Failure modes

- **`SET FOREIGN_KEY_CHECKS=0` does not exist in PostgreSQL**: the
  DDL is MySQL-dialect. The PostgreSQL consumer must use the
  dialect-specific form in `docs/schemas/sql-dialects/postgresql.md`,
  which omits the `SET FOREIGN_KEY_CHECKS` lines.
- **`ENGINE=InnoDB` clause is rejected by SQLite**: same — use the
  SQLite form which omits the engine clause and uses `WITHOUT ROWID`
  only where appropriate.
- **`JSON` type does not exist in SQLite**: the SQLite form uses
  `TEXT` for JSON columns (SQLite is dynamically typed; the engine's
  application code parses JSON via serde).

## What this phase does NOT do

- Does not create any domain tables (`academic_students`, etc.).
  Those come in Phases 3–5.
- Does not migrate any data from `devdb`.
- Does not touch the legacy `devdb` database.

## Exit criteria

- All six tables exist in `devdb_v2`.
- The `system_user` row is present.
- All indexes are present.
- The legacy `devdb` is unchanged.
- The consumer's `cargo check` against the new engine schema passes.
