-- migrations/engine/0000_engine_core.sqlite.sql
-- Canonical SQLite 3.x DDL for the 6 engine cross-cutting tables.
-- SQLite has no native UUID/JSON/TIMESTAMP types; the engine uses
-- TEXT for UUIDs and ISO 8601 TEXT for timestamps. The application
-- layer is responsible for type coercion. There is no RLS in SQLite.
-- Authoritative source — do not edit per-environment.
-- See: docs/schemas/sql-dialects/sqlite.md
--      docs/schemas/sql-dialects/README.md (Runtime DDL emission section)

-- =============================================================================
-- Educore — Engine Cross-Cutting Tables
-- =============================================================================
-- File:        0000_engine_core.sqlite.sql
-- Target DBs:  SQLite 3.x
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
--   docs/schemas/sql-dialects/sqlite.md
--   docs/schemas/sql-dialects/mysql.md
--   docs/schemas/sql-dialects/postgresql.md
--   docs/schemas/sql-dialects/comparison.md
--
-- Every aggregate table across the engine MUST include the engine
-- invariants: id, school_id, created_at, updated_at, created_by,
-- updated_by, active_status, version, etag, last_event_id,
-- correlation_id, source. See docs/schemas/database-schema.md § 2, § 5, § 9.
--
-- SQLite notes:
--   * Foreign keys are off by default; PRAGMA foreign_keys = ON is
--     applied by the adapter at connection time, not here.
--   * There is no native UUID, JSON, or TIMESTAMP type; columns hold
--     TEXT/INTEGER and the application layer coerces.
--   * CHECK(length(x) = 36) enforces that a TEXT column meant to hold
--     a UUIDv7 actually holds a 36-character string.
--   * Timestamps are stored as ISO 8601 TEXT (UTC) for readability.
-- =============================================================================

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
    event_id        TEXT         NOT NULL CHECK (length(event_id) = 36),
    event_type      TEXT         NOT NULL,
    event_version   INTEGER      NOT NULL,
    school_id       TEXT         NOT NULL CHECK (length(school_id) = 36),
    aggregate_id    TEXT         NOT NULL CHECK (length(aggregate_id) = 36),
    aggregate_type  TEXT         NOT NULL,
    actor_id        TEXT         NOT NULL CHECK (length(actor_id) = 36),
    correlation_id  TEXT         NOT NULL CHECK (length(correlation_id) = 36),
    causation_id    TEXT             NULL CHECK (causation_id IS NULL OR length(causation_id) = 36),
    occurred_at     TEXT         NOT NULL,
    recorded_at     TEXT         NOT NULL,
    payload         TEXT         NOT NULL,
    enqueued_at     TEXT         NOT NULL,
    published_at    TEXT             NULL,
    attempts        INTEGER      NOT NULL DEFAULT 0,
    last_error      TEXT             NULL,
    PRIMARY KEY (event_id)
);

CREATE INDEX IF NOT EXISTS idx_outbox_school_enqueued
    ON outbox (school_id, enqueued_at);
CREATE INDEX IF NOT EXISTS idx_outbox_published
    ON outbox (published_at, enqueued_at);
CREATE INDEX IF NOT EXISTS idx_outbox_aggregate
    ON outbox (aggregate_type, aggregate_id, occurred_at);
CREATE INDEX IF NOT EXISTS idx_outbox_correlation
    ON outbox (correlation_id);

-- -----------------------------------------------------------------------------
-- 2. audit_log — append-only, write-once compliance trail
-- -----------------------------------------------------------------------------
-- Reference: docs/schemas/audit-schema.md § 13
-- Every state-changing command, every authentication event, every
-- authorization denial, every capability change, every cross-tenant
-- operation, every backup/restore, every settings change touching
-- security, every school lifecycle event is recorded here. The
-- before/after snapshots are JSON-as-TEXT for flexibility.
--
-- The audit sink port is implemented by the consumer; the engine
-- writes through it. Append-only is enforced by database privileges
-- (INSERT-only role) and by the engine's lack of update/delete ops.
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS audit_log (
    audit_id        TEXT         NOT NULL CHECK (length(audit_id) = 36),
    school_id       TEXT         NOT NULL CHECK (length(school_id) = 36),
    actor_id        TEXT         NOT NULL CHECK (length(actor_id) = 36),
    actor_type      TEXT         NOT NULL,
    action          TEXT         NOT NULL,
    resource_type   TEXT         NOT NULL,
    resource_id     TEXT         NOT NULL CHECK (length(resource_id) = 36),
    event_id        TEXT             NULL CHECK (event_id IS NULL OR length(event_id) = 36),
    command_id      TEXT             NULL CHECK (command_id IS NULL OR length(command_id) = 36),
    correlation_id  TEXT         NOT NULL CHECK (length(correlation_id) = 36),
    occurred_at     TEXT         NOT NULL,
    recorded_at     TEXT         NOT NULL,
    ip              TEXT             NULL,
    user_agent      TEXT             NULL,
    session_id      TEXT             NULL CHECK (session_id IS NULL OR length(session_id) = 36),
    before_snapshot TEXT             NULL,
    after_snapshot  TEXT             NULL,
    metadata        TEXT             NULL,
    cross_tenant    INTEGER      NOT NULL DEFAULT 0 CHECK (cross_tenant IN (0, 1)),
    source          TEXT         NOT NULL,
    PRIMARY KEY (audit_id)
);

CREATE INDEX IF NOT EXISTS idx_audit_log_school_time
    ON audit_log (school_id, occurred_at);
CREATE INDEX IF NOT EXISTS idx_audit_log_actor
    ON audit_log (actor_id, occurred_at);
CREATE INDEX IF NOT EXISTS idx_audit_log_resource
    ON audit_log (resource_type, resource_id, occurred_at);
CREATE INDEX IF NOT EXISTS idx_audit_log_correlation
    ON audit_log (correlation_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_action
    ON audit_log (action, occurred_at);

-- -----------------------------------------------------------------------------
-- 3. idempotency — command replay safety
-- -----------------------------------------------------------------------------
-- Reference: docs/schemas/command-schema.md § 6
-- A command with the same (school_id, command_type, idempotency_key)
-- triple produces the same result without re-executing. The unique
-- primary key makes a duplicate insert fail; the engine catches the
-- constraint violation and returns the prior outcome.
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS idempotency (
    school_id        TEXT         NOT NULL CHECK (length(school_id) = 36),
    command_type     TEXT         NOT NULL,
    idempotency_key  TEXT         NOT NULL CHECK (length(idempotency_key) = 36),
    command_id       TEXT         NOT NULL CHECK (length(command_id) = 36),
    outcome          TEXT         NOT NULL,
    recorded_at      TEXT         NOT NULL,
    expires_at       TEXT         NOT NULL,
    PRIMARY KEY (school_id, command_type, idempotency_key)
);

CREATE INDEX IF NOT EXISTS idx_idempotency_expires
    ON idempotency (expires_at);

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
    event_id        TEXT         NOT NULL CHECK (length(event_id) = 36),
    event_type      TEXT         NOT NULL,
    event_version   INTEGER      NOT NULL,
    school_id       TEXT         NOT NULL CHECK (length(school_id) = 36),
    aggregate_id    TEXT         NOT NULL CHECK (length(aggregate_id) = 36),
    aggregate_type  TEXT         NOT NULL,
    actor_id        TEXT         NOT NULL CHECK (length(actor_id) = 36),
    correlation_id  TEXT         NOT NULL CHECK (length(correlation_id) = 36),
    causation_id    TEXT             NULL CHECK (causation_id IS NULL OR length(causation_id) = 36),
    occurred_at     TEXT         NOT NULL,
    recorded_at     TEXT         NOT NULL,
    payload         TEXT         NOT NULL,
    PRIMARY KEY (event_id)
);

CREATE INDEX IF NOT EXISTS idx_event_log_school_time
    ON event_log (school_id, occurred_at);
CREATE INDEX IF NOT EXISTS idx_event_log_type_time
    ON event_log (event_type, occurred_at);
CREATE INDEX IF NOT EXISTS idx_event_log_aggregate
    ON event_log (aggregate_type, aggregate_id, occurred_at);
CREATE INDEX IF NOT EXISTS idx_event_log_correlation
    ON event_log (correlation_id);

-- -----------------------------------------------------------------------------
-- 5. schema_registry — event-type schema catalog
-- -----------------------------------------------------------------------------
-- Reference: docs/schemas/event-schema.md § 7
-- Records every published event_type and its version(s). The engine
-- uses this to validate that producers do not emit unrecognized
-- versions and to drive multi-version publication during deprecation
-- windows. A consumer may back this with Confluent / Apicurio.
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS schema_registry (
    event_type      TEXT         NOT NULL,
    event_version   INTEGER      NOT NULL,
    schema_json     TEXT         NOT NULL,
    deprecated_at   TEXT             NULL,
    migration_path  TEXT             NULL,
    registered_at   TEXT         NOT NULL,
    PRIMARY KEY (event_type, event_version)
);

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
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS system_user (
    id            TEXT         NOT NULL CHECK (length(id) = 36),
    display_name  TEXT         NOT NULL,
    active_status INTEGER      NOT NULL DEFAULT 1 CHECK (active_status IN (0, 1)),
    created_at    TEXT         NOT NULL,
    PRIMARY KEY (id)
);

-- Seed the single SYSTEM_USER_ID row. The UUIDv7 is generated by the
-- engine's IdGenerator at first boot and recorded here. Consumers
-- MUST NOT insert additional rows.
-- INSERT OR IGNORE makes the seed idempotent.
INSERT OR IGNORE INTO system_user (id, display_name, active_status, created_at)
VALUES ('00000000-0000-7000-8000-000000000001', 'SYSTEM', 1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));
