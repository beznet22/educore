# MySQL — DDL Conventions

Target: **MySQL 8.0+** (8.0.16+ for `CHECK` constraints; 8.0.19+ for
inline `CHECK`; 8.0.21+ for `utf8mb4_0900_ai_ci` as default collation;
8.0.23+ for `INVISIBLE` columns).

The reference adapter implementing these conventions is
`educore-storage-mysql`. The DDL strings in this file are emitted
by `MysqlStorageAdapter::create_<table>_ddl()`.

## Identifier quoting

Use **backticks** for every identifier:

```sql
CREATE TABLE `outbox` (
  `event_id` CHAR(36) NOT NULL,
  ...
);
```

This is required for reserved words (e.g. `key`, `order`, `group`,
`table`, `select`) and for case-sensitivity. MySQL identifiers are
case-insensitive on Windows and case-sensitive on Linux/macOS by
default. The engine always uses lowercase identifiers.

## Default settings for every table

```sql
ENGINE=InnoDB
DEFAULT CHARSET=utf8mb4
COLLATE=utf8mb4_unicode_ci
```

`InnoDB` is required for transactional foreign-key enforcement. The
engine rejects `MyISAM`, `MEMORY`, or any non-transactional engine.

`utf8mb4` is required for full Unicode including emoji. The engine
rejects `utf8` (the MySQL alias for `utf8mb3`).

`utf8mb4_unicode_ci` is the engine's default collation. `utf8mb4_0900_ai_ci`
is the MySQL 8 default but is accent-insensitive; the engine prefers
`unicode_ci` for predictable tenant-data sorting.

## Type mapping

The engine's canonical types and their MySQL forms:

| Engine type | MySQL type | Notes |
| --- | --- | --- |
| `CHAR(36)` (UUIDv7) | `CHAR(36)` | utf8mb4 charset; 36 chars = 36 bytes (4-byte chars) |
| `BINARY(16)` (UUIDv7 compact) | `BINARY(16)` | 16 bytes; engine prefers `CHAR(36)` for human readability |
| `BIGINT` | `BIGINT` | signed; no `UNSIGNED` (engine's UUIDs are the canonical id) |
| `INT` | `INT` | signed; no `UNSIGNED` |
| `TINYINT` | `TINYINT` | 1 byte; the engine's `active_status`, `is_*` booleans |
| `BOOLEAN` | `BOOLEAN` | alias for `TINYINT(1)` |
| `VARCHAR(N)` | `VARCHAR(N)` | `N` is the engine's column length; engine uses `VARCHAR(64/128/191/200/255/500)` |
| `TEXT` | `TEXT` | engine uses for long-form text (`notes`, `address`, `description`) |
| `TIMESTAMP` | `TIMESTAMP` | UTC; engine stores UTC always |
| `DATETIME` | `DATETIME` | engine uses for date+time without timezone semantics |
| `DATE` | `DATE` | engine uses for date-only fields |
| `TIME` | `TIME` | engine uses for time-of-day fields |
| `JSON` | `JSON` | engine uses for `payload`, `before_snapshot`, `after_snapshot`, `metadata` |
| `DECIMAL(P,S)` | `DECIMAL(P,S)` | engine uses for money (P=14, S=2) and quantities |
| `ENUM` | not used | engine prefers `VARCHAR(N) NOT NULL` with `CHECK` constraint (portable) |

## Identifier lengths

MySQL's default `max_packet_size` is 64MB; identifier lengths are
limited to 64 characters. The engine's longest identifier is
`idx_academic_student_records_school_active_version` (50 chars),
well within the limit.

## Reserved column names

Per `docs/schemas/database-schema.md` § 12, the engine reserves
the following column names. They are NOT quoted in the engine
contracts, but in MySQL DDL they may need to be backticked if
they collide with MySQL reserved words.

| Column | MySQL reserved? | Quoting needed? |
| --- | --- | --- |
| `id` | no | no |
| `school_id` | no | no |
| `academic_id` | no | no |
| `record_id` | no | no |
| `session_id` | no | no |
| `version` | no | no |
| `etag` | no | no |
| `active_status` | no | no |
| `created_at` | no | no |
| `updated_at` | no | no |
| `created_by` | no | no |
| `updated_by` | no | no |
| `last_event_id` | no | no |
| `order` | yes | yes (when used as a column name) |
| `key` | yes | yes |
| `group` | yes | yes |
| `table` | yes | yes |

The engine's reserved names do not collide with MySQL reserved
words. The engine's `outbox` DDL does not need any column-name
backticks.

## Indexes

```sql
CREATE INDEX idx_<table>_<col1>_<col2> ON `<table>` (`<col1>`, `<col2>`);
```

The engine uses lowercase snake_case for index names. The
`(school_id, active_status)` composite index is mandatory on every
aggregate.

## Foreign keys

```sql
ALTER TABLE `<child>`
  ADD CONSTRAINT `fk_<child>_<col>_<parent>`
  FOREIGN KEY (`<col>`) REFERENCES `<parent>` (`id`)
  ON DELETE RESTRICT
  ON UPDATE RESTRICT;
```

The engine's default referential action is `ON DELETE RESTRICT`.
The engine's `database-schema.md` § 4 enumerates the exceptions
(derived / owned-child rows may use `CASCADE`; advisory references
may use `SET NULL`).

Foreign key names follow the pattern `fk_<child>_<col>_<parent>`.

## Row-level security

MySQL 8 supports RLS via `ROW LEVEL SECURITY` (8.0.23+ via
`SECURITY` invocations). However, MySQL's RLS is more limited
than PostgreSQL's `CREATE POLICY`. The engine's adapter does NOT
rely on MySQL's RLS; it enforces tenant isolation in the
application layer via a `school_id` filter on every query.

The `school_id` predicate is mandatory in every query that does
not explicitly opt into cross-tenant operation. The engine's
storage adapter's `execute_query()` method injects
`WHERE school_id = ?` automatically.

For the optional RLS-based defense:

```sql
-- Per MySQL 8.0.23+
ALTER TABLE `<aggregate>` 
  ADD CONSTRAINT `fk_<aggregate>_school`
  FOREIGN KEY (`school_id`) REFERENCES `platform_schools` (`id`);
```

The application still injects the `school_id` filter; the FK is
the database-level second line of defense.

## `CHECK` constraints

MySQL 8.0.16+ enforces `CHECK` constraints. The engine emits them
on enum-like columns:

```sql
CREATE TABLE `rbac_roles` (
  ...
  `role_type` VARCHAR(16) NOT NULL,
  CONSTRAINT `ck_rbac_roles_role_type` CHECK (`role_type` IN ('system', 'custom')),
  ...
);
```

Pre-8.0.16, the `CHECK` is parsed but not enforced. The engine
recommends MySQL 8.0.21+ for the consumer's deployment.

## Transactions

The engine uses `START TRANSACTION` (or `BEGIN`) and `COMMIT` /
`ROLLBACK`. The default isolation level is `REPEATABLE READ` (MySQL
default). The engine's storage adapter reads with
`SELECT ... FOR UPDATE` for command-dispatch row locks.

## The 6 engine cross-cutting tables — MySQL DDL

The full DDL is in `migrations/engine/0000_engine_core.mysql.sql`. Summary:

```sql
-- 1. outbox
CREATE TABLE IF NOT EXISTS `outbox` (
  `event_id`        CHAR(36)     NOT NULL,
  `event_type`      VARCHAR(191) NOT NULL,
  `event_version`   INT          NOT NULL,
  `school_id`       CHAR(36)     NOT NULL,
  `aggregate_id`    CHAR(36)     NOT NULL,
  `aggregate_type`  VARCHAR(64)  NOT NULL,
  `actor_id`        CHAR(36)     NOT NULL,
  `correlation_id`  CHAR(36)     NOT NULL,
  `causation_id`    CHAR(36)         NULL,
  `occurred_at`     TIMESTAMP    NOT NULL,
  `recorded_at`     TIMESTAMP    NOT NULL,
  `payload`         JSON         NOT NULL,
  `enqueued_at`     TIMESTAMP    NOT NULL,
  `published_at`    TIMESTAMP        NULL,
  `attempts`        INT          NOT NULL DEFAULT 0,
  `last_error`      TEXT             NULL,
  PRIMARY KEY (`event_id`),
  KEY `idx_outbox_school_enqueued` (`school_id`, `enqueued_at`),
  KEY `idx_outbox_published` (`published_at`, `enqueued_at`),
  KEY `idx_outbox_aggregate` (`aggregate_type`, `aggregate_id`, `occurred_at`),
  KEY `idx_outbox_correlation` (`correlation_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- 2. audit_log
-- (...similar pattern...)

-- 3. idempotency
-- (...similar pattern...)

-- 4. event_log
-- (...similar pattern...)

-- 5. schema_registry
-- (...similar pattern...)

-- 6. system_user
-- (...similar pattern, seeded with the SYSTEM_USER_ID row...)
```

The full DDL is in the file.

## A domain aggregate example: `academic_students`

The engine's `academic_students` table in MySQL:

```sql
CREATE TABLE IF NOT EXISTS `academic_students` (
  `id`              CHAR(36)     NOT NULL PRIMARY KEY,
  `school_id`       CHAR(36)     NOT NULL,
  `admission_number` VARCHAR(64) NULL,
  `roll_number`     VARCHAR(32)  NULL,
  `first_name`      VARCHAR(200) NOT NULL,
  `last_name`       VARCHAR(200) NULL,
  `full_name`       VARCHAR(200) NULL,
  `date_of_birth`   DATE         NULL,
  `email`           VARCHAR(200) NULL,
  `mobile`          VARCHAR(32)  NULL,
  `admission_date`  DATE         NULL,
  `photo_storage_key` VARCHAR(191) NULL,
  `gender_id`       CHAR(36)     NULL,
  `blood_group_id`  CHAR(36)     NULL,
  `religion_id`     CHAR(36)     NULL,
  `class_id`        CHAR(36)     NULL,
  `section_id`      CHAR(36)     NULL,
  `academic_id`     CHAR(36)     NULL,
  `category_id`     CHAR(36)     NULL,
  `group_id`        CHAR(36)     NULL,
  `route_id`        CHAR(36)     NULL,
  `vehicle_id`      CHAR(36)     NULL,
  `dormitory_id`    CHAR(36)     NULL,
  `room_id`         CHAR(36)     NULL,
  `guardian_id`     CHAR(36)     NULL,
  `user_id`         CHAR(36)     NULL,
  `role_id`         CHAR(36)     NULL,
  `version`         BIGINT       NOT NULL DEFAULT 1,
  `etag`            CHAR(32)     NOT NULL,
  `last_event_id`   CHAR(36)         NULL,
  `correlation_id`  CHAR(36)         NULL,
  `source`          VARCHAR(16)      NULL,
  `active_status`   TINYINT      NOT NULL DEFAULT 1,
  `created_at`      TIMESTAMP    NOT NULL,
  `updated_at`      TIMESTAMP    NOT NULL,
  `created_by`      CHAR(36)     NOT NULL,
  `updated_by`      CHAR(36)     NOT NULL,
  `id_v7_legacy`    BIGINT UNSIGNED NULL,
  `custom_fields`   JSON             NULL,
  CONSTRAINT `fk_academic_students_school` FOREIGN KEY (`school_id`)
    REFERENCES `platform_schools` (`id`) ON DELETE RESTRICT,
  CONSTRAINT `fk_academic_students_class` FOREIGN KEY (`class_id`)
    REFERENCES `academic_classes` (`id`) ON DELETE RESTRICT,
  -- ... more FKs ...
  KEY `idx_academic_students_school_active` (`school_id`, `active_status`),
  KEY `idx_academic_students_last_event` (`last_event_id`),
  KEY `idx_academic_students_correlation` (`correlation_id`),
  KEY `idx_academic_students_school_admission` (`school_id`, `admission_number`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
```

## Adapter implementation notes

- The `MysqlStorage` adapter uses `sqlx` for the connection pool.
  `sqlx` 0.8+ is the recommended version.
- The adapter emits DDL lazily; the consumer's migration runner
  applies it.
- The adapter reads the DDL from a string constant in the crate.
  The DDL is unit-tested against an in-memory MySQL via
  testcontainers or a local MySQL.
- The adapter enforces `school_id` in the application layer via
  the `WHERE` clause injection. The consumer does not need to
  write `WHERE school_id = ?`; the adapter does it.

## References

- MySQL 8 Reference Manual: `CREATE TABLE` syntax, `utf8mb4` charset,
  `INVISIBLE` columns, `CHECK` constraints.
- The `educore-storage-mysql` crate README.
- `docs/ports/storage.md` § 4: `Configuration` — the engine's
  `MysqlStorage::builder()` pattern.
- `docs/schemas/database-schema.md` § 11: the canonical minimum
  schema.
