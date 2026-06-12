-- =============================================================================
-- pg-rls-test-setup.sql
--
-- Provisions the non-superuser `tenant_b` role and the RLS policy required by
-- `crates/tools/storage-parity/tests/cross_cutting_integration.rs`
-- ::pg_rls_blocks_cross_tenant_audit_reads and the assessment-domain
-- ::pg_rls_blocks_cross_tenant_assessment_reads test (Phase 4).
--
-- Usage:
--   # 1. Run this script as a PG superuser (e.g. `psql -U postgres -f ...`).
--   # 2. Export two URLs before running the integration test:
--   #      export EDUCORE_PG_URL="postgres://postgres:pw@host:5432/educore"
--   #      export EDUCORE_PG_TENANT_B_URL="postgres://tenant_b:pw@host:5432/educore"
--   # 3. Run the gated test:
--   #      cargo test -p educore-storage-parity -- --ignored \
--   #          pg_rls_blocks_cross_tenant
--
-- The script is idempotent — re-running it is safe and resets the policy
-- state. It targets the `engine` schema (per migrations/engine/0000_engine_core
-- .postgres.sql). The script does NOT create the `educore` database or
-- the engine tables; it assumes the storage adapter's `migrate()` has been
-- run via `EDUCORE_PG_URL` (the superuser connection).
--
-- Phase 2 OQ #1 (Phase 2 hand-off § Open questions) — closed in Phase 4
-- (assessment domain). See `docs/handoff/PHASE-4-HANDOFF.md`.
-- =============================================================================

\set ON_ERROR_STOP on

-- -----------------------------------------------------------------------------
-- 1. The `tenant_b` role
-- -----------------------------------------------------------------------------
-- Drop & re-create so the password + attributes are reset. The password
-- here is a test-only constant; the script is not for production use.
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'tenant_b') THEN
        REASSIGN OWNED BY tenant_b TO postgres;
        DROP OWNED BY tenant_b;
        DROP ROLE tenant_b;
    END IF;
END
$$;

CREATE ROLE tenant_b WITH LOGIN PASSWORD 'tenant_b_pw';

-- -----------------------------------------------------------------------------
-- 2. Database + schema grants
-- -----------------------------------------------------------------------------
-- Connect to the current database (the one this script was sourced into).
GRANT CONNECT ON DATABASE current_database() TO tenant_b;
GRANT USAGE  ON SCHEMA engine                       TO tenant_b;

-- -----------------------------------------------------------------------------
-- 3. CRUD grants on the 6 engine cross-cutting tables
-- -----------------------------------------------------------------------------
GRANT SELECT, INSERT, UPDATE, DELETE ON
    engine.outbox,
    engine.audit_log,
    engine.event_log,
    engine.idempotency,
    engine.schema_registry,
    engine.system_user
TO tenant_b;

-- Sequence grants (uuid_generate_v7 etc., if the engine schema uses any).
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA engine TO tenant_b;

-- Default privileges for future tables created by the superuser.
ALTER DEFAULT PRIVILEGES IN SCHEMA engine
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO tenant_b;
ALTER DEFAULT PRIVILEGES IN SCHEMA engine
    GRANT USAGE, SELECT ON SEQUENCES TO tenant_b;

-- -----------------------------------------------------------------------------
-- 4. RLS policy — block cross-tenant reads on `engine.audit_log`
-- -----------------------------------------------------------------------------
-- The engine's row-level security filter: a row is visible to `tenant_b`
-- iff its `school_id` matches the session's `app.current_school_id`
-- GUC. The storage adapter sets this GUC on every connection (see
-- `crates/adapters/storage-postgres/src/connection.rs`).
ALTER TABLE engine.audit_log ENABLE ROW LEVEL SECURITY;
ALTER TABLE engine.audit_log FORCE  ROW LEVEL SECURITY;

DROP POLICY IF EXISTS audit_log_school_isolation ON engine.audit_log;
CREATE POLICY audit_log_school_isolation ON engine.audit_log
    USING (school_id = current_setting('app.current_school_id', true)::uuid)
    WITH CHECK (school_id = current_setting('app.current_school_id', true)::uuid);

-- The same policy on `outbox`, `event_log`, and `idempotency` (the
-- 4 sub-port tables that the engine's `Transaction` exposes).
ALTER TABLE engine.outbox      ENABLE ROW LEVEL SECURITY;
ALTER TABLE engine.outbox      FORCE  ROW LEVEL SECURITY;
DROP POLICY IF EXISTS outbox_school_isolation ON engine.outbox;
CREATE POLICY outbox_school_isolation ON engine.outbox
    USING (school_id = current_setting('app.current_school_id', true)::uuid)
    WITH CHECK (school_id = current_setting('app.current_school_id', true)::uuid);

ALTER TABLE engine.event_log   ENABLE ROW LEVEL SECURITY;
ALTER TABLE engine.event_log   FORCE  ROW LEVEL SECURITY;
DROP POLICY IF EXISTS event_log_school_isolation ON engine.event_log;
CREATE POLICY event_log_school_isolation ON engine.event_log
    USING (school_id = current_setting('app.current_school_id', true)::uuid)
    WITH CHECK (school_id = current_setting('app.current_school_id', true)::uuid);

ALTER TABLE engine.idempotency ENABLE ROW LEVEL SECURITY;
ALTER TABLE engine.idempotency FORCE  ROW LEVEL SECURITY;
DROP POLICY IF EXISTS idempotency_school_isolation ON engine.idempotency;
CREATE POLICY idempotency_school_isolation ON engine.idempotency
    USING (school_id = current_setting('app.current_school_id', true)::uuid)
    WITH CHECK (school_id = current_setting('app.current_school_id', true)::uuid);

-- -----------------------------------------------------------------------------
-- 5. Assessment-domain tables (Phase 4)
-- -----------------------------------------------------------------------------
-- The assessment tables are also gated. The test setup script runs the
-- adapter's `migrate()` for assessment, so by the time the RLS test
-- runs, the `engine.assessment_*` tables exist. (If the engine schema
-- does not yet have assessment tables, the GRANT below is a no-op;
-- re-run the script after the assessment migration is applied.)
DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.tables
        WHERE table_schema = 'engine'
          AND table_name   = 'assessment_exams'
    ) THEN
        EXECUTE $E$
            GRANT SELECT, INSERT, UPDATE, DELETE ON
                engine.assessment_exams,
                engine.assessment_marks_registers,
                engine.assessment_marks_register_children,
                engine.assessment_exam_schedules,
                engine.assessment_exam_schedule_subjects,
                engine.assessment_result_stores,
                engine.assessment_online_exams,
                engine.assessment_seat_plans,
                engine.assessment_seat_plan_children,
                engine.admit_cards
            TO tenant_b
        $E$;

        EXECUTE 'ALTER TABLE engine.assessment_exams                    ENABLE ROW LEVEL SECURITY';
        EXECUTE 'ALTER TABLE engine.assessment_exams                    FORCE  ROW LEVEL SECURITY';
        EXECUTE $E$
            CREATE POLICY assessment_exams_school_isolation ON engine.assessment_exams
                USING (school_id = current_setting(''app.current_school_id'', true)::uuid)
                WITH CHECK (school_id = current_setting(''app.current_school_id'', true)::uuid)
        $E$;

        EXECUTE 'ALTER TABLE engine.assessment_marks_registers          ENABLE ROW LEVEL SECURITY';
        EXECUTE 'ALTER TABLE engine.assessment_marks_registers          FORCE  ROW LEVEL SECURITY';
        EXECUTE $E$
            CREATE POLICY assessment_marks_registers_school_isolation ON engine.assessment_marks_registers
                USING (school_id = current_setting(''app.current_school_id'', true)::uuid)
                WITH CHECK (school_id = current_setting(''app.current_school_id'', true)::uuid)
        $E$;

        EXECUTE 'ALTER TABLE engine.assessment_result_stores            ENABLE ROW LEVEL SECURITY';
        EXECUTE 'ALTER TABLE engine.assessment_result_stores            FORCE  ROW LEVEL SECURITY';
        EXECUTE $E$
            CREATE POLICY assessment_result_stores_school_isolation ON engine.assessment_result_stores
                USING (school_id = current_setting(''app.current_school_id'', true)::uuid)
                WITH CHECK (school_id = current_setting(''app.current_school_id'', true)::uuid)
        $E$;

        EXECUTE 'ALTER TABLE engine.assessment_exam_schedules           ENABLE ROW LEVEL SECURITY';
        EXECUTE 'ALTER TABLE engine.assessment_exam_schedules           FORCE  ROW LEVEL SECURITY';
        EXECUTE $E$
            CREATE POLICY assessment_exam_schedules_school_isolation ON engine.assessment_exam_schedules
                USING (school_id = current_setting(''app.current_school_id'', true)::uuid)
                WITH CHECK (school_id = current_setting(''app.current_school_id'', true)::uuid)
        $E$;

        EXECUTE 'ALTER TABLE engine.assessment_seat_plans                ENABLE ROW LEVEL SECURITY';
        EXECUTE 'ALTER TABLE engine.assessment_seat_plans                FORCE  ROW LEVEL SECURITY';
        EXECUTE $E$
            CREATE POLICY assessment_seat_plans_school_isolation ON engine.assessment_seat_plans
                USING (school_id = current_setting(''app.current_school_id'', true)::uuid)
                WITH CHECK (school_id = current_setting(''app.current_school_id'', true)::uuid)
        $E$;

        EXECUTE 'ALTER TABLE engine.admit_cards                         ENABLE ROW LEVEL SECURITY';
        EXECUTE 'ALTER TABLE engine.admit_cards                         FORCE  ROW LEVEL SECURITY';
        EXECUTE $E$
            CREATE POLICY admit_cards_school_isolation ON engine.admit_cards
                USING (school_id = current_setting(''app.current_school_id'', true)::uuid)
                WITH CHECK (school_id = current_setting(''app.current_school_id'', true)::uuid)
        $E$;
    END IF;
END
$$;

-- -----------------------------------------------------------------------------
-- 6. Done
-- -----------------------------------------------------------------------------
-- Sanity check (verbose; comment out in CI):
--   \du tenant_b
--   \dn
--   \dp engine.audit_log
--   SELECT polname, polrelid::regclass, polcmd FROM pg_policy
--   WHERE polrelid IN ('engine.audit_log'::regclass,
--                      'engine.outbox'::regclass,
--                      'engine.event_log'::regclass,
--                      'engine.idempotency'::regclass);
