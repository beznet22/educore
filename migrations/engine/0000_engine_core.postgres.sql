-- migrations/engine/0000_engine_core.postgres.sql
-- Canonical PostgreSQL 14+ DDL for the 6 engine cross-cutting tables.
-- All 6 tables are wrapped in the `engine` schema. Consumers set
-- `search_path = engine, public` per session.
-- Authoritative source — do not edit per-environment.
-- See: docs/schemas/sql-dialects/postgresql.md
--      docs/schemas/sql-dialects/README.md (Runtime DDL emission section)

-- =============================================================================
-- Educore — Engine Cross-Cutting Tables
-- =============================================================================
-- File:        0000_engine_core.postgres.sql
-- Target DBs:  PostgreSQL 14+
-- Strategy:    Option B (side-by-side + cutover). This file is the FIRST
--              migration applied to the new database (e.g. `devdb_v2`).
--
-- These six tables are owned by the engine, not by any domain. They are
-- wrapped in the `engine` schema so that consumer code can `SET search_path
-- = engine, public` and reference them by their short names. This is
-- the only place the engine uses a schema qualifier; aggregate tables
-- emitted by the macro use a configurable schema (default: `public`).
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
--   docs/schemas/sql-dialects/postgresql.md
--   docs/schemas/sql-dialects/mysql.md
--   docs/schemas/sql-dialects/sqlite.md
--   docs/schemas/sql-dialects/comparison.md
--
-- Every aggregate table across the engine MUST include the engine
-- invariants: id, school_id, created_at, updated_at, created_by,
-- updated_by, active_status, version, etag, last_event_id,
-- correlation_id, source. See docs/schemas/database-schema.md § 2, § 5, § 9.
-- =============================================================================

-- The engine schema is the canonical home for the 6 cross-cutting tables.
-- It is created idempotently so that re-running this script is safe.
CREATE SCHEMA IF NOT EXISTS engine;
SET search_path = engine, public;

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
CREATE TABLE IF NOT EXISTS engine.outbox (
    event_id        UUID         NOT NULL,
    event_type      VARCHAR(191) NOT NULL,
    event_version   INT          NOT NULL,
    school_id       UUID         NOT NULL,
    aggregate_id    UUID         NOT NULL,
    aggregate_type  VARCHAR(64)  NOT NULL,
    actor_id        UUID         NOT NULL,
    correlation_id  UUID         NOT NULL,
    causation_id    UUID             NULL,
    occurred_at     TIMESTAMPTZ  NOT NULL,
    recorded_at     TIMESTAMPTZ  NOT NULL,
    payload         JSONB        NOT NULL,
    enqueued_at     TIMESTAMPTZ  NOT NULL,
    published_at    TIMESTAMPTZ      NULL,
    attempts        INT          NOT NULL DEFAULT 0,
    last_error      TEXT             NULL,
    PRIMARY KEY (event_id)
);

CREATE INDEX IF NOT EXISTS idx_outbox_school_enqueued
    ON engine.outbox (school_id, enqueued_at);
CREATE INDEX IF NOT EXISTS idx_outbox_published
    ON engine.outbox (published_at, enqueued_at);
CREATE INDEX IF NOT EXISTS idx_outbox_aggregate
    ON engine.outbox (aggregate_type, aggregate_id, occurred_at);
CREATE INDEX IF NOT EXISTS idx_outbox_correlation
    ON engine.outbox (correlation_id);

-- -----------------------------------------------------------------------------
-- 2. audit_log — append-only, write-once compliance trail
-- -----------------------------------------------------------------------------
-- Reference: docs/schemas/audit-schema.md § 13
-- Every state-changing command, every authentication event, every
-- authorization denial, every capability change, every cross-tenant
-- operation, every backup/restore, every settings change touching
-- security, every school lifecycle event is recorded here. The
-- before/after snapshots are JSONB for flexibility and indexability.
--
-- The audit sink port is implemented by the consumer; the engine
-- writes through it. Append-only is enforced by database privileges
-- (INSERT-only role) and by the engine's lack of update/delete ops.
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS engine.audit_log (
    audit_id        UUID         NOT NULL,
    school_id       UUID         NOT NULL,
    actor_id        UUID         NOT NULL,
    actor_type      VARCHAR(16)  NOT NULL,
    action          VARCHAR(191) NOT NULL,
    resource_type   VARCHAR(64)  NOT NULL,
    resource_id     UUID         NOT NULL,
    event_id        UUID             NULL,
    command_id      UUID             NULL,
    correlation_id  UUID         NOT NULL,
    occurred_at     TIMESTAMPTZ  NOT NULL,
    recorded_at     TIMESTAMPTZ  NOT NULL,
    ip              VARCHAR(45)     NULL,
    user_agent      VARCHAR(512)    NULL,
    session_id      UUID             NULL,
    before_snapshot JSONB            NULL,
    after_snapshot  JSONB            NULL,
    metadata        JSONB            NULL,
    cross_tenant    BOOLEAN      NOT NULL DEFAULT FALSE,
    source          VARCHAR(16)  NOT NULL,
    -- Partitioned by (school_id, month). The composite primary key is
    -- required because PostgreSQL partitioned tables must include all
    -- partition key columns in every UNIQUE/PRIMARY KEY constraint.
    PRIMARY KEY (audit_id, school_id, occurred_at)
) PARTITION BY RANGE (school_id, occurred_at);

CREATE INDEX IF NOT EXISTS idx_audit_log_school_time
    ON engine.audit_log (school_id, occurred_at);
CREATE INDEX IF NOT EXISTS idx_audit_log_actor
    ON engine.audit_log (actor_id, occurred_at);
CREATE INDEX IF NOT EXISTS idx_audit_log_resource
    ON engine.audit_log (resource_type, resource_id, occurred_at);
CREATE INDEX IF NOT EXISTS idx_audit_log_correlation
    ON engine.audit_log (correlation_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_action
    ON engine.audit_log (action, occurred_at);

-- -----------------------------------------------------------------------------
-- 3. idempotency — command replay safety
-- -----------------------------------------------------------------------------
-- Reference: docs/schemas/command-schema.md § 6
-- A command with the same (school_id, command_type, idempotency_key)
-- triple produces the same result without re-executing. The unique
-- constraint makes a duplicate insert fail; the engine catches the
-- constraint violation and returns the prior outcome.
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS engine.idempotency (
    school_id        UUID         NOT NULL,
    command_type     VARCHAR(191) NOT NULL,
    idempotency_key  UUID         NOT NULL,
    command_id       UUID         NOT NULL,
    outcome          JSONB        NOT NULL,
    recorded_at      TIMESTAMPTZ  NOT NULL,
    expires_at       TIMESTAMPTZ  NOT NULL,
    PRIMARY KEY (school_id, command_type, idempotency_key)
);

CREATE INDEX IF NOT EXISTS idx_idempotency_expires
    ON engine.idempotency (expires_at);

-- -----------------------------------------------------------------------------
-- 4. event_log — retained events for replay and projection rebuild
-- -----------------------------------------------------------------------------
-- Reference: docs/schemas/event-schema.md § 8, § 9
-- The outbox is the relay's read source; the event_log is the durable
-- store of every event ever emitted. Consumers replay from this table
-- to rebuild projections. Retention is consumer-configurable
-- (default 90 days; cold-archive to S3/Glacier after that).
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS engine.event_log (
    event_id        UUID         NOT NULL,
    event_type      VARCHAR(191) NOT NULL,
    event_version   INT          NOT NULL,
    school_id       UUID         NOT NULL,
    aggregate_id    UUID         NOT NULL,
    aggregate_type  VARCHAR(64)  NOT NULL,
    actor_id        UUID         NOT NULL,
    correlation_id  UUID         NOT NULL,
    causation_id    UUID             NULL,
    occurred_at     TIMESTAMPTZ  NOT NULL,
    recorded_at     TIMESTAMPTZ  NOT NULL,
    payload         JSONB        NOT NULL,
    PRIMARY KEY (event_id)
);

CREATE INDEX IF NOT EXISTS idx_event_log_school_time
    ON engine.event_log (school_id, occurred_at);
CREATE INDEX IF NOT EXISTS idx_event_log_type_time
    ON engine.event_log (event_type, occurred_at);
CREATE INDEX IF NOT EXISTS idx_event_log_aggregate
    ON engine.event_log (aggregate_type, aggregate_id, occurred_at);
CREATE INDEX IF NOT EXISTS idx_event_log_correlation
    ON engine.event_log (correlation_id);

-- -----------------------------------------------------------------------------
-- 5. schema_registry — event-type schema catalog
-- -----------------------------------------------------------------------------
-- Reference: docs/schemas/event-schema.md § 7
-- Records every published event_type and its version(s). The engine
-- uses this to validate that producers do not emit unrecognized
-- versions and to drive multi-version publication during deprecation
-- windows. A consumer may back this with Confluent / Apicurio.
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS engine.schema_registry (
    event_type      VARCHAR(191) NOT NULL,
    event_version   INT          NOT NULL,
    schema_json     JSONB        NOT NULL,
    deprecated_at   TIMESTAMPTZ      NULL,
    migration_path  TEXT             NULL,
    registered_at   TIMESTAMPTZ  NOT NULL,
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
CREATE TABLE IF NOT EXISTS engine.system_user (
    id            UUID         NOT NULL,
    display_name  VARCHAR(200) NOT NULL,
    active_status SMALLINT     NOT NULL DEFAULT 1,
    created_at    TIMESTAMPTZ  NOT NULL,
    PRIMARY KEY (id)
);

-- Seed the single SYSTEM_USER_ID row. The UUIDv7 is generated by the
-- engine's IdGenerator at first boot and recorded here. Consumers
-- MUST NOT insert additional rows.
-- ON CONFLICT DO NOTHING makes the seed idempotent.
INSERT INTO engine.system_user (id, display_name, active_status, created_at)
VALUES ('00000000-0000-7000-8000-000000000001', 'SYSTEM', 1, NOW())
ON CONFLICT (id) DO NOTHING;

-- -----------------------------------------------------------------------------
-- audit_log default partition
-- -----------------------------------------------------------------------------
-- Default partition for any (school_id, occurred_at) tuple that does not
-- fall into an explicitly-created monthly partition. Per the spec
-- (§ 13.1) partitions are created lazily by the audit sink at month
-- boundaries; this default partition guarantees that no audit row is ever
-- rejected by the partitioner while consumers ship the historical
-- partitioning job. A monitoring job should alert when this partition
-- grows above the threshold defined in docs/schemas/audit-schema.md.
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS engine.audit_log_default
    PARTITION OF engine.audit_log DEFAULT;
