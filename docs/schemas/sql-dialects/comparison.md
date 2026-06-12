# SQL Dialect Comparison

A feature-by-feature side-by-side comparison of MySQL 8+,
SQLite 3.x, and PostgreSQL 14+ for the engine's storage adapters.

Legend:

- **Native** — fully supported, no adapter workaround needed.
- **Adapter** — supported via the adapter's application-layer logic.
- **Workaround** — possible but requires non-trivial work in the
  adapter or the consumer.
- **No** — not supported.

## Identifier & schema

| Feature | MySQL 8+ | SQLite 3.x | PostgreSQL 14+ |
| --- | --- | --- | --- |
| Identifier quoting | backticks `` ` `` | backticks or double quotes `"` | double quotes `"` |
| Identifier case-sensitivity | Linux/macOS: case-sensitive; Windows: case-insensitive | case-sensitive | case-sensitive (when quoted) |
| Max identifier length | 64 chars | unlimited | 63 chars (NAMEDATALEN - 1) |
| Schema (namespace) | database (different `db.educore`) | `ATTACH DATABASE` | native `SCHEMA` |
| Per-session `search_path` | n/a (use `USE db`) | n/a | `SET search_path = ...` |
| Table name parity | ✓ (`academic_students` etc.) | ✓ | ✓ (or `engine.academic_students` if wrapped) |

## Type system

| Engine type | MySQL 8+ | SQLite 3.x | PostgreSQL 14+ |
| --- | --- | --- | --- |
| `CHAR(36)` (UUIDv7) | `CHAR(36)` | `TEXT` (with `length() = 36` check) | `UUID` (native) |
| `BINARY(16)` | `BINARY(16)` | `BLOB` | `BYTEA` |
| `BIGINT` | `BIGINT` (signed) | `INTEGER` (8 bytes) | `BIGINT` (signed) |
| `INT` | `INT` (signed) | `INTEGER` (4 bytes) | `INTEGER` (signed) |
| `TINYINT` | `TINYINT` (signed) | `INTEGER` (with range check) | `SMALLINT` (with range check) |
| `BOOLEAN` | `BOOLEAN` (alias for `TINYINT(1)`) | `INTEGER` (with `CHECK IN (0,1)`) | `BOOLEAN` |
| `VARCHAR(N)` | `VARCHAR(N)` (utf8mb4 = `N` chars, `4N` bytes) | `TEXT` (no length limit) | `VARCHAR(N)` (utf8 = `N` chars, `4N` bytes) |
| `TEXT` | `TEXT` (utf8mb4) | `TEXT` (utf8) | `TEXT` (utf8) |
| `TIMESTAMP` (UTC) | `TIMESTAMP` (UTC, `TIMESTAMP` = `DATETIME` since 8.0.19) | `TEXT` (ISO 8601 UTC string) | `TIMESTAMPTZ` (UTC, with timezone) |
| `DATE` | `DATE` | `TEXT` (`YYYY-MM-DD`) | `DATE` |
| `JSON` | `JSON` (binary storage, fast) | `TEXT` (with `json_valid()` check) | `JSONB` (binary storage, indexable) |
| `DECIMAL(P,S)` | `DECIMAL(P,S)` (engine uses for money) | `TEXT` (engine stores as `rust_decimal` string) | `NUMERIC(P,S)` |
| `ENUM` | not used (use `VARCHAR + CHECK`) | not used (use `TEXT + CHECK`) | not used (use `VARCHAR + CHECK`) |
| `STRICT` tables | n/a (always strict) | `STRICT` keyword (3.37+) | always strict |
| `WITHOUT ROWID` | n/a (always rowid) | yes (for lookup tables) | n/a |

## Constraints

| Feature | MySQL 8+ | SQLite 3.x | PostgreSQL 14+ |
| --- | --- | --- | --- |
| Primary key | yes | yes | yes |
| Unique constraint | yes (`UNIQUE KEY`) | yes (`UNIQUE`) | yes (`UNIQUE`) |
| `NULLS NOT DISTINCT` (treat NULL as equal) | 8.0.13+ (default is NULLs distinct) | 3.x (NULLs distinct) | 15.0+ (`NULLS NOT DISTINCT` clause) |
| Foreign key | yes (InnoDB) | yes (with `PRAGMA foreign_keys = ON`) | yes |
| `ON DELETE` actions | `CASCADE`, `RESTRICT`, `SET NULL`, `NO ACTION` | same | same, plus `DEFERRABLE INITIALLY DEFERRED` |
| `CHECK` constraint | 8.0.16+ (enforced) | yes (3.3.0+) | yes (always) |
| `DEFERRABLE` constraint | not supported | not supported | yes |
| Partial index | 8.0.13+ (`WHERE` clause) | yes (native) | yes (native) |
| Functional index | yes (8.0.13+) | yes (3.9.0+) | yes |
| `UPSERT` / `MERGE` | `INSERT ... ON DUPLICATE KEY UPDATE` | `INSERT OR REPLACE` / `ON CONFLICT` (3.24+) | `INSERT ... ON CONFLICT` (9.5+) / `MERGE` (15.0+) |

## Indexes

| Feature | MySQL 8+ | SQLite 3.x | PostgreSQL 14+ |
| --- | --- | --- | --- |
| Default index type | BTREE | BTREE | BTREE |
| `BRIN` index | not supported | not supported | yes |
| `GIN` index | not supported | not supported | yes |
| `HASH` index | not supported (only MEMORY engine, deprecated) | not supported | yes |
| `INCLUDE` columns | not supported | not supported | yes (covering index) |
| Partial index `WHERE` | 8.0.13+ | yes | yes |

## Row-level security (tenant isolation)

| Feature | MySQL 8+ | SQLite 3.x | PostgreSQL 14+ |
| --- | --- | --- | --- |
| Native RLS | 8.0.23+ (limited; requires `SECURITY` invocations) | **No** | **Native, full-featured** |
| Policy syntax | `CREATE POLICY ... ON ... USING (...)` (limited) | n/a | `CREATE POLICY ... ON ... USING (...) WITH CHECK (...)` |
| `BYPASSRLS` role attribute | not supported | n/a | yes |
| Per-session `SET LOCAL` | limited (variables are global) | n/a | yes |
| Per-row `current_setting()` | yes | n/a | yes |
| The engine's `school_id` filter | adapter-enforced in `WHERE` | adapter-enforced in `WHERE` | `CREATE POLICY` (preferred) + adapter-enforced `WHERE` (defense in depth) |

The engine's recommendation: **use PG for production**; the
RLS-based tenant isolation is a defense-in-depth that MySQL and
SQLite cannot provide. MySQL and SQLite rely solely on the
adapter-enforced `WHERE school_id = ?` filter.

## Transactions & concurrency

| Feature | MySQL 8+ | SQLite 3.x | PostgreSQL 14+ |
| --- | --- | --- | --- |
| Default isolation | REPEATABLE READ | SERIALIZABLE (effectively) | READ COMMITTED |
| `READ COMMITTED` | yes | yes | yes (default) |
| `REPEATABLE READ` | yes (default) | n/a (always serialized) | yes |
| `SERIALIZABLE` | yes | yes (always) | yes |
| `SELECT ... FOR UPDATE` | yes (InnoDB) | yes (with `BEGIN IMMEDIATE`) | yes |
| `SELECT ... FOR SHARE` | yes | yes | yes |
| Optimistic concurrency | `version` column (engine's) | `version` column | `version` column (and `xmin` system column) |
| WAL (write-ahead log) | n/a (InnoDB is always WAL) | yes (`PRAGMA journal_mode = WAL`) | yes (always) |
| Multi-version concurrency control (MVCC) | limited | yes (per-connection snapshot) | yes (per-transaction snapshot) |
| Connection pooling | native | single-writer / multi-reader (WAL) | native |
| Savepoints | yes | yes | yes |

## Generated / computed columns

| Feature | MySQL 8+ | SQLite 3.x | PostgreSQL 14+ |
| --- | --- | --- | --- |
| `GENERATED ALWAYS AS ... STORED` | yes | yes (3.31+) | yes (12.0+) |
| `GENERATED ALWAYS AS ... VIRTUAL` | not supported | not supported | not supported (use a view) |
| Generated column in primary key | not allowed | not allowed | not allowed |
| Generated column with `AUTO_INCREMENT` / `SERIAL` | not allowed | not allowed | not allowed |

The engine does not use generated columns; the `id` is generated
in application code (the `IdGenerator` port).

## Identity / sequence

| Feature | MySQL 8+ | SQLite 3.x | PostgreSQL 14+ |
| --- | --- | --- | --- |
| `AUTO_INCREMENT` | yes (`BIGINT AUTO_INCREMENT`) | `INTEGER PRIMARY KEY AUTOINCREMENT` | `BIGINT GENERATED BY DEFAULT AS IDENTITY` (10+) / `BIGSERIAL` (legacy) |
| `INSERT ... RETURNING` | not supported | yes (3.35+) | yes |
| Sequence object | not (table-level only) | not (table-level only) | yes (`CREATE SEQUENCE`) |
| `gen_random_uuid()` | 8.0+ (`UUID()` function) | not built-in (use `randomblob(16)`) | 13.0+ (`gen_random_uuid()` in core) |
| UUID v7 generation | not built-in (use app code) | not built-in (use app code) | not built-in (use app code) |
| Engine's `IdGenerator` | app-level UUIDv7 (Rust crate `uuid`) | app-level | app-level |

The engine generates UUIDv7 in Rust via the `uuid` crate
(version 1.10+ with `v7` feature). The SQL `AUTO_INCREMENT` /
`SERIAL` is not used by the engine's aggregates (every id is
UUIDv7).

## JSON

| Feature | MySQL 8+ | SQLite 3.x | PostgreSQL 14+ |
| --- | --- | --- | --- |
| Native JSON type | `JSON` (binary) | `TEXT` (parsed on read) | `JSONB` (binary) |
| Index on JSON path | 8.0+ (multi-valued indexes) | not supported | yes (GIN) |
| `JSON_TABLE` | 8.0.4+ | not supported | yes (`jsonb_to_recordset`, `jsonb_path_query`) |
| `JSON_EXTRACT` | yes | `json_extract` (3.38+) | `jsonb_extract_path` |
| `JSON_VALID` | 8.0+ | `json_valid` (3.38+) | `jsonb_typeof` (manual) |
| Mutation in place | limited | not supported | yes (PL/pgSQL `jsonb_set`) |

The engine uses `JSON` / `JSONB` for the `outbox.payload`,
`audit_log.before_snapshot` / `after_snapshot` / `metadata`, and
`idempotency.outcome` columns. The engine's `serde_json::Value`
type is the application-level representation; the adapter handles
serialization.

## Full-text search

| Feature | MySQL 8+ | SQLite 3.x | PostgreSQL 14+ |
| --- | --- | --- | --- |
| Native full-text | `FULLTEXT` index (InnoDB, MyISAM) | `FTS5` (3.9+) | `tsvector` / `tsquery` + GIN |
| Multi-language stemming | limited (English only) | yes (via `unicode61` tokenizer) | yes (Snowball, etc.) |
| `MATCH ... AGAINST` | yes | yes (`MATCH ... AGAINST`) | yes (`@@` operator) |

The engine's search is not in the cross-cutting tables; domain
tables may add full-text indexes as needed. The engine does not
mandate a search strategy.

## Encoding & collation

| Feature | MySQL 8+ | SQLite 3.x | PostgreSQL 14+ |
| --- | --- | --- | --- |
| Default charset | `utf8mb4` (set per table) | `utf8` (file encoding) | `UTF8` (database encoding) |
| Default collation | `utf8mb4_unicode_ci` (engine's choice) | `BINARY` (no collation) | `en_US.utf8` (consumer's choice) |
| Per-column collation | yes | no | yes |
| Emoji support | yes (utf8mb4) | yes (utf8) | yes (UTF8) |
| Multi-byte indexing | yes | yes (using `length()` not `char_length()`) | yes |

## Backup & restore

| Feature | MySQL 8+ | SQLite 3.x | PostgreSQL 14+ |
| --- | --- | --- | --- |
| Online backup | `mysqldump` (logical), `mysqlbackup` (physical, hot) | `VACUUM INTO` (3.27+) / file copy (with WAL) | `pg_dump` (logical), `pg_basebackup` (physical, hot) |
| Point-in-time recovery | yes (binary log) | n/a (single file; snapshots at file level) | yes (WAL archiving) |
| Logical replication | yes | n/a (use Litestream / LiteFS) | yes (built-in) |
| Read replicas | yes (async / semi-sync) | yes (manual via `ATTACH DATABASE`) | yes (streaming replication) |

## Extensions

| Feature | MySQL 8+ | SQLite 3.x | PostgreSQL 14+ |
| --- | --- | --- | --- |
| `pgcrypto` (PG) | n/a | n/a | yes |
| `mysql_fdw` / `postgres_fdw` | not supported | not supported | yes (foreign data wrappers) |
| `pg_stat_statements` | n/a | n/a | yes |
| `JSON_TABLE` | 8.0.4+ | not supported | yes |

## The 6 engine cross-cutting tables — feature availability

| Table | MySQL 8+ | SQLite 3.x | PostgreSQL 14+ |
| --- | --- | --- | --- |
| `outbox` | Native | Native (with `STRICT` + `WITHOUT ROWID`) | Native (with RLS) |
| `audit_log` | Native | Native | Native (with RLS, append-only role) |
| `idempotency` | Native (unique PK) | Native (composite PK) | Native (with `ON CONFLICT DO NOTHING` for atomic insert) |
| `event_log` | Native (retention via consumer-side job) | Native | Native (with `pg_partman` for partitioning if needed) |
| `schema_registry` | Native | Native | Native |
| `system_user` | Native | Native (single row) | Native (single row, RLS-friendly) |

## Domain aggregate tables — feature availability

| Engine invariant | MySQL 8+ | SQLite 3.x | PostgreSQL 14+ |
| --- | --- | --- | --- |
| `id CHAR(36) PRIMARY KEY` (UUIDv7) | Native | Native (as `TEXT` with length check) | Native (`UUID` type) |
| `school_id CHAR(36) NOT NULL` + RLS | adapter-enforced | adapter-enforced | Native (RLS) + adapter-enforced |
| `version BIGINT NOT NULL DEFAULT 1` | Native | Native | Native |
| `etag CHAR(32) NOT NULL` | Native | Native (as `TEXT` with length check) | Native |
| `last_event_id CHAR(36) NULL` | Native | Native | Native (as `UUID`) |
| `correlation_id CHAR(36) NULL` | Native | Native | Native (as `UUID`) |
| `source VARCHAR(16) NULL` | Native | Native | Native |
| `active_status TINYINT NOT NULL DEFAULT 1` | Native | Native (as `INTEGER` with `CHECK IN (0,1)`) | Native (as `BOOLEAN`) |
| `created_at TIMESTAMP NOT NULL` | Native | Native (as `TEXT` ISO 8601) | Native (as `TIMESTAMPTZ`) |
| `updated_at TIMESTAMP NOT NULL` | Native | Native | Native |
| `created_by CHAR(36) NOT NULL` | Native | Native | Native (as `UUID`) |
| `updated_by CHAR(36) NOT NULL` | Native | Native | Native (as `UUID`) |
| `id_v7_legacy BIGINT UNSIGNED NULL` | Native (transitional) | Native (transitional) | Native (transitional) |
| `(school_id, active_status)` index | Native | Native | Native |
| FK with `ON DELETE RESTRICT` | Native | Native (with `PRAGMA foreign_keys = ON`) | Native (with `DEFERRABLE INITIALLY DEFERRED`) |
| `CHECK` on enum-like column | 8.0.16+ | Native | Native |

## Per-backend recommendation

For production SaaS deployments:

- **Use PostgreSQL** if the consumer can run a managed PG
  instance. The engine's RLS-based tenant isolation is a
  defense-in-depth that MySQL and SQLite cannot provide. PG is the
  primary target.

- **Use MySQL** if the consumer's hosting platform (e.g.
  PlanetScale, Vitess, AWS Aurora MySQL) is MySQL-based. The
  adapter's `WHERE school_id = ?` filter is the only tenant
  isolation. The consumer's MySQL version must be 8.0.16+ for
  `CHECK` constraints and 8.0.23+ for native RLS (advisory).

- **Use SQLite** for embedded / offline / single-process
  deployments (Tauri desktop, mobile via UniFFI, CLI). The
  adapter's `WHERE school_id = ?` filter and the
  `PRAGMA foreign_keys = ON` are the only tenant isolation. SQLite's
  `STRICT` tables (3.37+) and `WITHOUT ROWID` (engine's preference
  for lookup tables) are required.

## SurrealDB feature comparison

| Feature | MySQL 8+ | SQLite 3.x | PostgreSQL 14+ | SurrealDB 3.x |
| --- | --- | --- | --- | --- |
| Identifier quoting | backticks | double-quotes | double-quotes | backticks |
| DDL extensions | CHECK, INDEX, TRIGGER | CHECK, INDEX, TRIGGER | CHECK, INDEX, TRIGGER, POLICY | DEFINE FIELD, DEFINE INDEX, DEFINE EVENT |
| Outbox | polling | polling | LISTEN/NOTIFY | LIVE SELECT |
| Multi-tenancy | schema-per-DB | DB-file | schema-per-DB + RLS | namespace + DB + school_id |
| Vector search | ❌ | sqlite-vss ext | pgvector ext | native `<|N,COSINE|>` |
| Graph traversal | ❌ | ❌ | ❌ (or ltree) | native `->` |
| Embedded mode | ❌ | ✅ (single file) | ❌ | ✅ (RocksDB or in-memory) |
| Watch changes for sync | polling | polling | LISTEN/NOTIFY | LIVE SELECT |
| Idempotency support | ✅ | ✅ | ✅ | ✅ (all support the engine's outbox pattern) |
| ACID | ✅ | ✅ | ✅ | ✅ |

## See also

- `mysql.md` — MySQL 8+ DDL conventions
- `sqlite.md` — SQLite 3.x DDL conventions
- `postgresql.md` — PostgreSQL 14+ DDL conventions
- `docs/ports/storage.md` — the storage port contract
- `docs/schemas/database-schema.md` — engine invariants
- `docs/schemas/event-schema.md` § 8 — the outbox table spec
- `docs/schemas/audit-schema.md` § 13 — the audit_log table spec
- `docs/schemas/command-schema.md` § 6 — the idempotency table spec
