# 04 — Engine Invariants Per Aggregate (Phase 4)

## Goal

For every aggregate table in `devdb_v2`, ensure the seven engine
invariant columns are present, correctly typed, and constrained per
`docs/schemas/database-schema.md` § 2, § 5, § 9, and § 11.

## The seven engine-invariant columns

| Column | Type | Nullable | Default | Doc ref |
| --- | --- | --- | --- | --- |
| `created_at` | `TIMESTAMP` | NOT NULL | (engine-managed) | § 2 |
| `updated_at` | `TIMESTAMP` | NOT NULL | (engine-managed) | § 2 |
| `created_by` | `CHAR(36)` | NOT NULL | `SYSTEM_USER_ID` | § 2 |
| `updated_by` | `CHAR(36)` | NOT NULL | `SYSTEM_USER_ID` | § 2 |
| `active_status` | `TINYINT` | NOT NULL | `1` | § 2, § 6 |
| `version` | `BIGINT` | NOT NULL | `1` | § 5, § 9 |
| `etag` | `CHAR(32)` | NOT NULL | (per-row computed) | § 5, § 9 |
| `last_event_id` | `CHAR(36)` | NULL | `NULL` | § 5 |
| `correlation_id` | `CHAR(36)` | NULL | `NULL` | § 5 |
| `source` | `VARCHAR(16)` | NULL | `NULL` | § 5 |

(That's 10 columns; the user's earlier summary said 6 NEW + 4
existing. The `created_at` / `updated_at` / `active_status` columns
are usually present in the legacy dump; the `version` / `etag` /
`last_event_id` / `correlation_id` / `source` / `created_by` /
`updated_by` are NEW.)

## Per-table recipe

For every aggregate table `T` in `devdb_v2`, apply:

```sql
-- 1. Add the engine-invariant columns (no-op if already present)
ALTER TABLE T
  ADD COLUMN IF NOT EXISTS version BIGINT NOT NULL DEFAULT 1,
  ADD COLUMN IF NOT EXISTS etag CHAR(32) NOT NULL DEFAULT '',
  ADD COLUMN IF NOT EXISTS last_event_id CHAR(36) NULL,
  ADD COLUMN IF NOT EXISTS correlation_id CHAR(36) NULL,
  ADD COLUMN IF NOT EXISTS source VARCHAR(16) NULL;

-- 2. Backfill etag with a deterministic placeholder
--    (the engine recomputes the real etag on the first read-write)
UPDATE T SET etag = blake3(id || school_id)[:16] WHERE etag = '';

-- 3. Ensure created_at and updated_at are NOT NULL
--    (legacy has them nullable; tighten)
UPDATE T SET created_at = COALESCE(created_at, UTC_TIMESTAMP(6))
WHERE created_at IS NULL;
UPDATE T SET updated_at = COALESCE(updated_at, created_at, UTC_TIMESTAMP(6))
WHERE updated_at IS NULL;

ALTER TABLE T MODIFY COLUMN created_at TIMESTAMP NOT NULL;
ALTER TABLE T MODIFY COLUMN updated_at TIMESTAMP NOT NULL;

-- 4. Ensure active_status is NOT NULL DEFAULT 1
UPDATE T SET active_status = 1 WHERE active_status IS NULL;
ALTER TABLE T MODIFY COLUMN active_status TINYINT NOT NULL DEFAULT 1;

-- 5. Add the canonical (school_id, active_status) index
CREATE INDEX idx_<T>_school_active ON T (school_id, active_status);

-- 6. Tighten school_id to NOT NULL
--    (legacy has it nullable with DEFAULT 1; engine has it NOT NULL)
UPDATE T SET school_id = (SELECT id FROM platform_schools LIMIT 1)
WHERE school_id IS NULL;
ALTER TABLE T MODIFY COLUMN school_id CHAR(36) NOT NULL;

-- 7. Add the (school_id, ...) composite for tenant-scoped lookups
--    where the engine's domain has a natural alternate key
--    (e.g. (school_id, admission_number) on academic_students,
--     (school_id, code) on academic_subjects)
--    These are added per-table, not as a blanket rule.
```

## Foreign-key actions: `CASCADE` → `RESTRICT`

Per `docs/schemas/database-schema.md` § 4, the engine's default
referential action is `ON DELETE RESTRICT`. Soft delete
(`active_status = 0`) is the deletion mechanism; hard delete is
reserved for GDPR erasure and is itself a capability-gated command.

The legacy dump has `ON DELETE CASCADE` on most FKs (especially on
`school_id` and `user_id`). This is unsafe — deleting a school would
cascade-delete every student, every grade, every payment. The
engine refuses to honour that.

```sql
-- Per FK, replace CASCADE with RESTRICT
ALTER TABLE <child> DROP FOREIGN KEY <fk_name>;
ALTER TABLE <child> ADD CONSTRAINT fk_<child>_<col>_<parent>
  FOREIGN KEY (<col>) REFERENCES <parent> (id) ON DELETE RESTRICT;
```

The exceptions to `RESTRICT` are:

- `ON DELETE CASCADE` is permitted only for **derived** or
  **owned-child** rows whose entire existence is bound to the parent
  (e.g. `assessment_marks_register_children` cascade-deleted with
  their parent `assessment_marks_registers`).
- `ON DELETE SET NULL` is permitted only for non-essential,
  advisory references (e.g. an optional `note` linked to a deleted
  class).

These exceptions are reviewed per ADR. For the initial migration,
all FKs go to `RESTRICT` and the engine's audit-driven soft-delete
flow handles the rest.

## Per-domain column addendum

In addition to the seven engine invariants, some domains have
domain-specific column conventions:

### Academic year-scoped rows

`docs/schemas/database-schema.md` § 2 specifies two additional
required columns for academic-year-scoped rows:

| Column | Type | Nullable | Purpose |
| --- | --- | --- | --- |
| `academic_id` | `CHAR(36)` (AcademicYearId) | NOT NULL | the academic year the row belongs to |
| `record_id` | `CHAR(36)` (StudentRecordId) | sometimes | for per-student-year rows, the parent enrollment |

The 165 legacy tables that are academic-year-scoped already have an
`academic_id` column. The type changes from `INT(10) UNSIGNED` to
`CHAR(36)` per the Phase 2 ID conversion. The `NOT NULL` constraint
is tightened.

### Money columns

The engine canonical for money is `DECIMAL(14,2)` per
`docs/code-standards.md` ("Money values use `rust_decimal`"). The
legacy dump has `FLOAT`, `DOUBLE`, `VARCHAR(200)` for money columns
(yes, really — `sm_staffs.basic_salary` is a `varchar(200)`). These
are widened/narrowed in Phase 6's field-level data flow.

### PII columns

Per `docs/schemas/database-schema.md` § 13, PII is encrypted at rest
in production. The migration does NOT encrypt the data; it only
flags the columns. The consumer's deployment wires the encryption
at the column level (TDE) or via application-side encryption.

Columns flagged as PII:

- `email`, `mobile`, `phone`, `national_id_no`, `local_id_no`,
  `bank_account_no`, `driving_license`, `epf_no`, `password`,
  `device_token`, `notification_token`, `reset_code`,
  `emergency_mobile`, `mothers_mobile`, `fathers_mobile`,
  `guardians_mobile`, `current_address`, `permanent_address`.

## Index changes

| Index | When added | Per table |
| --- | --- | --- |
| `idx_<t>_school_active (school_id, active_status)` | always | yes |
| `uq_<t>_school_<natural_key> (school_id, <key>)` | domain-specific | only on tables with a natural key |
| `idx_<t>_last_event_id (last_event_id)` | always | yes (used for outbox replay lookup) |
| `idx_<t>_correlation_id (correlation_id)` | always | yes (used for cross-aggregate event tracing) |

The legacy dump has only the school_id FK indexes (one per FK
column) and a handful of natural-key indexes (e.g. `uq_assign_permissions_school_id_role_permission`).
The engine adds the canonical `(school_id, active_status)` and the
event-tracing indexes per `database-schema.md` § 11.

## Aggregate count

| Statistic | Count |
| --- | --- |
| Tables receiving the 7 engine invariants | 310 |
| Columns added per table | 6 NEW (legacy had `created_at`, `updated_at`, sometimes `active_status`) |
| Total columns added | ~1,860 |
| Total indexes added | ~620 (`(school_id, active_status)` + `last_event_id` + `correlation_id`) |
| FK actions flipped from CASCADE to RESTRICT | ~600 (the parent-anchor FKs on `school_id` and `user_id` in 300+ tables) |
| PII columns flagged | ~30 distinct column names across ~50 tables |

## Exit criteria

- Every aggregate table has all 7 engine invariants.
- Every aggregate table has the `(school_id, active_status)` index.
- Every aggregate table's `school_id` is `NOT NULL`.
- Every parent-anchor FK is `ON DELETE RESTRICT` (not `CASCADE`).
- The `etag` is backfilled with a placeholder for every row.
- `version` is `1` for every row.
- The engine's repository trait can be implemented against the
  schema without further migrations (the storage adapter's
  `to_typed_*` methods succeed for the sample rows).
