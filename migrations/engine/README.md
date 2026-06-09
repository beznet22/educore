# `migrations/engine/`

Canonical DDL for the **6 engine cross-cutting tables** (`outbox`,
`audit_log`, `idempotency`, `event_log`, `schema_registry`,
`system_user`) in three SQL dialects.

These files are the **authoritative source of truth** for the
cross-cutting schema. The storage adapter crates
(`smsengine-storage-postgres`, `smsengine-storage-mysql`,
`smsengine-storage-sqlite`) embed them via `include_str!` and apply
them at startup via `storage.create_schema().await`. The DB
round-trip dominates the cost (~6 s for ~310 tables on MySQL);
string build time is <10 ms.

For the full end-to-end flow, see
[`docs/schemas/sql-dialects/README.md` § "Runtime DDL emission"](../../docs/schemas/sql-dialects/README.md#runtime-ddl-emission--end-to-end-flow).

## Files

| File                                       | Dialect       | Schema / namespace | Notes                                                                                          |
| ------------------------------------------ | ------------- | ------------------ | ---------------------------------------------------------------------------------------------- |
| `0000_engine_core.mysql.sql`               | MySQL 8+      | (default schema)   | `ENGINE=InnoDB`, `utf8mb4`, `JSON`, `CHAR(36)`, `BOOLEAN`. Reference file.                      |
| `0000_engine_core.postgres.sql`            | PostgreSQL 14+| `engine` schema    | `UUID`, `JSONB`, `TIMESTAMPTZ`, `SMALLINT`. Consumers set `search_path = engine, public`.      |
| `0000_engine_core.sqlite.sql`              | SQLite 3.x    | (default schema)   | `TEXT` (with `CHECK(length() = 36)` for UUIDs), `INTEGER`, ISO 8601 `TEXT` for timestamps.      |

## Table order (in all three files)

1. `outbox`
2. `audit_log`
3. `idempotency`
4. `event_log`
5. `schema_registry`
6. `system_user`

## Engine invariants preserved in all 6 tables

- `school_id` is `NOT NULL` on every table (multi-tenant invariant).
- UUID columns are `CHAR(36)` / `UUID` / `TEXT CHECK(length() = 36)`.
- Timestamps are `TIMESTAMP` / `TIMESTAMPTZ` / `TEXT` (ISO 8601 UTC).
- JSON columns are `JSON` / `JSONB` / `TEXT` (with json1 at the
  application layer).

## Related

- `migrations/0000_engine_core.sql` — legacy top-level file (the
  pre-dialect-split MySQL copy). **Do not edit**; it is preserved
  for the Group D cleanup task.
- `migrations/0001_*.sql` … `migrations/0015_*.sql` — legacy
  Schoolify/InfixEdu dump (research source only).
- `docs/schemas/database-schema.md` — engine invariants.
- `docs/schemas/event-schema.md` — outbox + event_log spec.
- `docs/schemas/audit-schema.md` — audit_log spec.
- `docs/schemas/command-schema.md` — idempotency spec.
- `docs/schemas/sql-dialects/` — per-dialect conventions and
  feature-by-feature comparison.
