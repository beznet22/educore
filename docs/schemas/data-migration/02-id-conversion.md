# 02 — ID Conversion: `BIGINT AUTO_INCREMENT` → `CHAR(36)` UUIDv7 (Phase 2)

## Goal

Replace every legacy `id BIGINT UNSIGNED AUTO_INCREMENT` (or
`INT(10) UNSIGNED AUTO_INCREMENT`) with a typed UUIDv7 identifier
stored as `CHAR(36)`. This is a per-table conversion that preserves
the data, transforms the type, and keeps a transitional
`id_v7_legacy BIGINT UNSIGNED NULL` column for 90 days to support
reversible ETL and join-mapping.

## Why UUIDv7

Per `docs/schemas/database-schema.md` § 1.4 and
`docs/schemas/tenancy-schema.md` § 2, the engine's canonical
identifier type is `Uuid` (specifically UUIDv7 — time-ordered, globally
unique, generated client-side by the engine's `IdGenerator` port).

UUIDv7 is preferred over UUIDv4 because it is time-ordered: rows
inserted later have lexicographically larger ids, which preserves
insert order in B-tree indexes without an extra `created_at` sort. This
matters for the engine's `outbox`, `event_log`, and `audit_log` tables
which are append-only and time-ordered.

UUIDv7 is preferred over `BIGINT AUTO_INCREMENT` because:

- It is globally unique across all backends and replicas without
  coordination.
- It is portable across MySQL, SQLite, and PostgreSQL with the same
  storage form (`CHAR(36)`).
- It carries the timestamp in the high bits, giving the engine a
  cheap "when was this created" hint without an extra column.

## The conversion strategy

### Per-table steps (in order, per table)

```sql
-- 1. Add the transitional BIGINT column
ALTER TABLE <table>
  ADD COLUMN id_v7_legacy BIGINT UNSIGNED NULL AFTER id;

-- 2. Backfill: capture the existing AUTO_INCREMENT value before
--    dropping the AUTO_INCREMENT
UPDATE <table> SET id_v7_legacy = id;

-- 3. Drop the AUTO_INCREMENT PRIMARY KEY constraint
ALTER TABLE <table> DROP PRIMARY KEY;

-- 4. Add the new CHAR(36) id column
ALTER TABLE <table> ADD COLUMN id_new CHAR(36) NOT NULL DEFAULT '';

-- 5. Backfill the new id from the legacy id using a deterministic
--    UUIDv7 derivation: uuid_v7(<table_namespace>, legacy_id)
--    The engine's `id_v7_legacy` column carries the BIGINT.
--    The new id is a function of (table_namespace, legacy_id) and
--    therefore idempotent.
UPDATE <table>
SET id_new = uuid_v7(<table_namespace>, id_v7_legacy);

-- 6. Promote id_new to PRIMARY KEY
ALTER TABLE <table> DROP COLUMN id;
ALTER TABLE <table> CHANGE COLUMN id_new id CHAR(36) NOT NULL PRIMARY KEY;

-- 7. Add the engine-invariant columns that the engine will need
--    (Phase 4 covers this for the wider set; for ID-only, just
--    ensure the PK index is on `id`)
```

### The `uuid_v7(<namespace>, <legacy_id>)` function

UUIDv7 is a 128-bit identifier: 48 bits of unix-ms timestamp in the
high bits, then 12 bits of random or sub-ms-precision, then 62 bits
of randomness, with version (`7`) and variant bits set per RFC 9562.

The engine's deterministic UUIDv7 derivation is:

```text
uuid_v7(namespace, legacy_id) = UUIDv7(
    timestamp = <a fixed "engine epoch" — the engine's first commit
                timestamp, e.g. 2026-01-01T00:00:00.000Z>,
    sub_ms    = (legacy_id % 4096),
    rand_a    = (legacy_id >> 12) & 0xFFF,
    rand_b    = blake3(namespace || legacy_id)[0..62 bits]
)
```

The output is deterministic for a given `(namespace, legacy_id)` pair.
Re-running the migration is a no-op. The `namespace` is a per-table
constant (`acad.student`, `fin.invoice`, etc.) that prevents the
same legacy id in two different tables from colliding.

### Why not use `UUID()` (random UUIDv4)?

UUIDv4 is allowed by the engine but UUIDv7 is preferred for time
ordering. The conversion strategy uses UUIDv7.

### Why not use the engine's live `IdGenerator`?

The `IdGenerator` port generates UUIDv7 from the current clock.
That is the runtime behavior. The migration derives UUIDv7 from
the legacy BIGINT id so that the mapping is deterministic and
reversible (re-running the migration produces the same ids). Once
the engine is operational, the `IdGenerator` takes over for new
rows.

## Foreign keys

Every FK that references the legacy `id` column must be updated to
reference the new `CHAR(36) id`. The column type changes; the
referenced column changes; the FK constraint is reissued under the
new name.

The number of FKs to reissue is approximately the number of FKs in
the schema (~1,200) minus the few that point at the engine's own
tables (e.g. the outbox references nothing; the system_user has
inbound FKs from every aggregate's `created_by`).

```sql
-- For each FK:
ALTER TABLE <child> DROP FOREIGN KEY <fk_name>;
ALTER TABLE <child> ADD CONSTRAINT fk_<child>_<col>_<parent>
  FOREIGN KEY (<col>) REFERENCES <parent> (id) ON DELETE RESTRICT;
```

The `ON DELETE RESTRICT` (per `database-schema.md` § 4) replaces
the legacy `ON DELETE CASCADE` for parent anchors (school_id, user_id).

## Indexes

The implicit PK index on the new `CHAR(36) id` is created
automatically. Composite indexes that lead with `id` are unchanged
(Laravel didn't use them, but the engine's query layer might add
them in the future). The new `(school_id, active_status)` index
per `database-schema.md` § 11 is added in Phase 4.

## The transitional `id_v7_legacy` column

The transitional column is kept for 90 days post-cutover. It is
used by:

- Rollback scripts (Phase rollback path) to re-create the BIGINT PK.
- Bug investigation when a downstream system still has a reference
  to the legacy id.
- A future "legacy compatibility" consumer adapter that exposes the
  legacy ids to external systems that have not yet migrated.

Drop the transitional column after 90 days:

```sql
ALTER TABLE <table> DROP COLUMN id_v7_legacy;
```

## What goes wrong if Phase 2 is skipped

The engine's typed identifiers (`StudentId`, `InvoiceId`, etc.) are
UUIDv7 at the Rust type level. If the DB column is still `BIGINT
AUTO_INCREMENT`, the storage adapter's `to_typed_id(row.id)` returns a
synthetic UUIDv7 derived from the legacy BIGINT, which works at
runtime but:

- Breaks parity with the engine's reference adapters (which expect
  `CHAR(36)`).
- Prevents cross-system integration (an external system cannot take
  a `StudentId` and look it up in the engine's storage).
- Prevents the consumer's application from storing UUIDv7 in its own
  cache or message broker (the BIGINT is engine-internal, not
  portable).

Phase 2 is mandatory.

## Aggregate count for Phase 2

- **310 tables** to convert.
- **~1,200 FKs** to reissue (some are inbound from `created_by` /
  `updated_by` to `system_user`; some are intra-domain; some are
  cross-domain).
- **~6,000 lines of `ALTER TABLE`** if done by hand; ~1,500 lines
  with templating.

## Exit criteria

- Every `id` column is `CHAR(36) NOT NULL PRIMARY KEY`.
- No `AUTO_INCREMENT` remains in any CREATE TABLE or ALTER TABLE.
- Every FK that referenced the legacy `id` now references the new
  `CHAR(36) id`.
- The `id_v7_legacy` column is present and backfilled.
- A sample of 10 random rows per domain produces the same
  `(school_id, ...)` lookups as the legacy `devdb` (the BIGINT is
  not the lookup key in any engine code; the verification is that
  the UUIDv7 derivation is correct).
