-- migrations/engine/0000_engine_core.mysql.sql
-- Canonical MySQL 8+ DDL for the 6 engine cross-cutting tables.
-- This is the reference file. Adapters embed it via `include_str!`.
-- Authoritative source — do not edit per-environment.
-- See: docs/schemas/sql-dialects/mysql.md
--      docs/schemas/sql-dialects/README.md (Runtime DDL emission section)

-- =============================================================================
-- Educore — Engine Cross-Cutting Tables
-- =============================================================================
-- File:        0000_engine_core.mysql.sql
-- Target DBs:  MySQL 8+
-- Strategy:    Option B (side-by-side + cutover). This file is the FIRST
--              migration applied to the new database (e.g. `devdb_v2`).
--
-- These six tables are owned by the engine, not by any domain. They are
-- UNPREFIXED so that the engine's table parity guarantee holds across all
-- three backends (MySQL, SQLite, PostgreSQL use identical table names).
--
--   - outbox           -> transactional event publication (event-schema.md § 8)
--   - audit_log        -> immutable compliance trail (audit-schema.md § 13)
--   - idempotency      -> command replay safety (command-schema.md § 6)
--   - event_log        -> retained events for replay / projections
--   - schema_registry  -> event-type schema catalog (event-schema.md § 7)
--   - system_user      -> the SYSTEM actor referenced by every aggregate
--
-- The DDL below is the canonical form. For dialect-specific variations
-- (identifier quoting, type mapping, RLS, etc.) see:
--
--   docs/schemas/sql-dialects/mysql.md
--   docs/schemas/sql-dialects/sqlite.md
--   docs/schemas/sql-dialects/postgresql.md
--   docs/schemas/sql-dialects/comparison.md
--
-- Every aggregate table across the engine MUST include the engine
-- invariants: id, school_id, created_at, updated_at, created_by,
-- updated_by, active_status, version, etag, last_event_id,
-- correlation_id, source. See docs/schemas/database-schema.md § 2, § 5, § 9.
-- =============================================================================

SET FOREIGN_KEY_CHECKS=0;

-- -----------------------------------------------------------------------------
-- 1. outbox — transactional event publication
-- -----------------------------------------------------------------------------
-- Reference: docs/schemas/event-schema.md § 8
-- Every state-changing command writes the events it emits to this table
-- in the same database transaction that mutates the aggregate. A relay
-- process (consumer-side) polls pending rows and publishes them to the
-- event bus, then marks them as published.
--
-- The event_id is the canonical primary key and is the dedup token
-- across the entire pipeline (outbox -> bus -> consumer -> projection).
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS outbox (
    event_id        CHAR(36)     NOT NULL,
    event_type      VARCHAR(191) NOT NULL,
    event_version   INT          NOT NULL,
    school_id       CHAR(36)     NOT NULL,
    aggregate_id    CHAR(36)     NOT NULL,
    aggregate_type  VARCHAR(64)  NOT NULL,
    actor_id        CHAR(36)     NOT NULL,
    correlation_id  CHAR(36)     NOT NULL,
    causation_id    CHAR(36)         NULL,
    occurred_at     TIMESTAMP    NOT NULL,
    recorded_at     TIMESTAMP    NOT NULL,
    payload         JSON         NOT NULL,
    enqueued_at     TIMESTAMP    NOT NULL,
    published_at    TIMESTAMP        NULL,
    attempts        INT          NOT NULL DEFAULT 0,
    last_error      TEXT             NULL,
    PRIMARY KEY (event_id),
    KEY idx_outbox_school_enqueued (school_id, enqueued_at),
    KEY outbox_school_id_idx (school_id),
    KEY idx_outbox_published (published_at, enqueued_at),
    KEY idx_outbox_aggregate (aggregate_type, aggregate_id, occurred_at),
    KEY idx_outbox_correlation (correlation_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- -----------------------------------------------------------------------------
-- 2. audit_log — append-only, write-once compliance trail
-- -----------------------------------------------------------------------------
-- Reference: docs/schemas/audit-schema.md § 13
-- Every state-changing command, every authentication event, every
-- authorization denial, every capability change, every cross-tenant
-- operation, every backup/restore, every settings change touching
-- security, every school lifecycle event is recorded here. The
-- before/after snapshots are JSON for flexibility.
--
-- The audit sink port is implemented by the consumer; the engine
-- writes through it. Append-only is enforced by database privileges
-- (INSERT-only role) and by the engine's lack of update/delete ops.
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS audit_log (
    audit_id        CHAR(36)     NOT NULL,
    school_id       CHAR(36)     NOT NULL,
    actor_id        CHAR(36)     NOT NULL,
    actor_type      VARCHAR(16)  NOT NULL,
    action          VARCHAR(191) NOT NULL,
    resource_type   VARCHAR(64)  NOT NULL,
    resource_id     CHAR(36)     NOT NULL,
    event_id        CHAR(36)         NULL,
    command_id      CHAR(36)         NULL,
    correlation_id  CHAR(36)     NOT NULL,
    occurred_at     TIMESTAMP    NOT NULL,
    recorded_at     TIMESTAMP    NOT NULL,
    ip              VARCHAR(45)     NULL,
    user_agent      VARCHAR(512)    NULL,
    session_id      CHAR(36)         NULL,
    before_snapshot JSON             NULL,
    after_snapshot  JSON             NULL,
    metadata        JSON             NULL,
    cross_tenant    BOOLEAN      NOT NULL DEFAULT FALSE,
    source          VARCHAR(16)  NOT NULL,
    PRIMARY KEY (audit_id),
    KEY idx_audit_log_school_time (school_id, occurred_at),
    KEY audit_log_school_id_idx (school_id),
    KEY idx_audit_log_actor (actor_id, occurred_at),
    KEY idx_audit_log_resource (resource_type, resource_id, occurred_at),
    KEY idx_audit_log_correlation (correlation_id),
    KEY idx_audit_log_action (action, occurred_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- -----------------------------------------------------------------------------
-- 3. idempotency — command replay safety
-- -----------------------------------------------------------------------------
-- Reference: docs/schemas/command-schema.md § 6
-- A command with the same (school_id, command_type, idempotency_key)
-- triple produces the same result without re-executing. The unique
-- index makes a duplicate insert fail; the engine catches the
-- constraint violation and returns the prior outcome.
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS idempotency (
    school_id        CHAR(36)     NOT NULL,
    command_type     VARCHAR(191) NOT NULL,
    idempotency_key  CHAR(36)     NOT NULL,
    command_id       CHAR(36)     NOT NULL,
    outcome          JSON         NOT NULL,
    recorded_at      TIMESTAMP    NOT NULL,
    expires_at       TIMESTAMP    NOT NULL,
    PRIMARY KEY (school_id, command_type, idempotency_key),
    KEY idempotency_school_id_idx (school_id),
    KEY idx_idempotency_expires (expires_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- -----------------------------------------------------------------------------
-- 4. event_log — retained events for replay and projection rebuild
-- -----------------------------------------------------------------------------
-- Reference: docs/schemas/event-schema.md § 8, § 9
-- The outbox is the relay's read source; the event_log is the durable
-- store of every event ever emitted. Consumers replay from this table
-- to rebuild projections. Retention is consumer-configurable
-- (default 90 days; cold-archive to S3/Glacier after that).
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS event_log (
    event_id        CHAR(36)     NOT NULL,
    event_type      VARCHAR(191) NOT NULL,
    event_version   INT          NOT NULL,
    school_id       CHAR(36)     NOT NULL,
    aggregate_id    CHAR(36)     NOT NULL,
    aggregate_type  VARCHAR(64)  NOT NULL,
    actor_id        CHAR(36)     NOT NULL,
    correlation_id  CHAR(36)     NOT NULL,
    causation_id    CHAR(36)         NULL,
    occurred_at     TIMESTAMP    NOT NULL,
    recorded_at     TIMESTAMP    NOT NULL,
    payload         JSON         NOT NULL,
    PRIMARY KEY (event_id),
    KEY idx_event_log_school_time (school_id, occurred_at),
    KEY event_log_school_id_idx (school_id),
    KEY idx_event_log_type_time (event_type, occurred_at),
    KEY idx_event_log_aggregate (aggregate_type, aggregate_id, occurred_at),
    KEY idx_event_log_correlation (correlation_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- -----------------------------------------------------------------------------
-- 5. schema_registry — event-type schema catalog
-- -----------------------------------------------------------------------------
-- Reference: docs/schemas/event-schema.md § 7
-- Records every published event_type and its version(s). The engine
-- uses this to validate that producers do not emit unrecognized
-- versions and to drive multi-version publication during deprecation
-- windows. A consumer may back this with Confluent / Apicurio.
--
-- NOTE (ADAPT-MY-008): `schema_registry` is an engine-global catalog,
-- not a multi-tenant table — event-type schemas are shared across all
-- schools by design. There is intentionally no `school_id` column and
-- therefore no `school_id` index. See `system_user` below for the
-- other exempt table.
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS schema_registry (
    event_type      VARCHAR(191) NOT NULL,
    event_version   INT          NOT NULL,
    schema_json     JSON         NOT NULL,
    deprecated_at   TIMESTAMP        NULL,
    migration_path  TEXT             NULL,
    registered_at   TIMESTAMP    NOT NULL,
    PRIMARY KEY (event_type, event_version)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- -----------------------------------------------------------------------------
-- 6. system_user — the SYSTEM actor
-- -----------------------------------------------------------------------------
-- Reference: docs/schemas/database-schema.md § 2
-- Every aggregate row's `created_by` / `updated_by` references a real
-- user id. When the engine itself is the actor (background jobs, the
-- outbox relay, migrations, system-generated events), the actor is
-- this single row. The id is the well-known `SYSTEM_USER_ID` constant
-- in the engine code; the UUIDv7 value is fixed at engine
-- initialization and seeded here.
--
-- NOTE (ADAPT-MY-008): `system_user` is a single-row engine-global
-- table representing the SYSTEM actor. It is intentionally not
-- tenant-scoped (the row IS the system actor for every school), so
-- there is no `school_id` column and no `school_id` index. The PK on
-- `id` is the only index needed for the single seeded row.
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS system_user (
    id          CHAR(36)     NOT NULL,
    display_name VARCHAR(200) NOT NULL,
    active_status TINYINT     NOT NULL DEFAULT 1,
    created_at  TIMESTAMP    NOT NULL,
    PRIMARY KEY (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Seed the single SYSTEM_USER_ID row. The UUIDv7 is generated by the
-- engine's IdGenerator at first boot and recorded here. Consumers
-- MUST NOT insert additional rows.
INSERT IGNORE INTO system_user (id, display_name, active_status, created_at)
VALUES ('00000000-0000-7000-8000-000000000001', 'SYSTEM', 1, UTC_TIMESTAMP(6));

SET FOREIGN_KEY_CHECKS=1;
