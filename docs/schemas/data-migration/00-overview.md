# 00 — Overview

## Goal

Migrate the live `devdb` MySQL database (legacy Schoolify / InfixEdu
Laravel project) to the Educore schema. The engine's domain
shapes are documented in `docs/schemas/` and the engine's
aggregates are described in `docs/specs/<domain>/`. The migration
preserves all real school data with no loss.

## Constraints

- **Live database**: `devdb` on `127.0.0.1:3306` is in use by a real
  school. Downtime is acceptable in a planned maintenance window but
  not for a multi-day outage.
- **Real school data**: every row in every table is a real student's
  or staff member's record. The migration must preserve data
  integrity; sample rows must match byte-for-byte after the transform
  (modulo type-driven normalisation).
- **Credential in git history**: the legacy `DATABASE_URL` with a
  real-looking password (`paxxw0rd@2791`) is in commit `5fa148c`.
  See `11-security.md`. **This must be addressed before any public
  push.**

## Strategy: Option B (side-by-side + cutover)

Three options were considered. The chosen strategy is **Option B**.

| Concern | Option A (in-place) | **Option B (side-by-side + cutover)** | Option C (dual-write) |
| --- | --- | --- | --- |
| Downtime | one window, hours | near-zero | zero |
| Reversibility | yes, with rollback script | trivial, point app back at `devdb` | per-table |
| Storage cost | unchanged | ~2x during the run | ~2x during the run |
| Application changes | small (read-side table names) | medium (config + read paths) | large (feature flags per query) |
| Risk to live data | medium (single transaction window) | low (parallel run) | low (per-table) |
| Best for | small school, planned window | small school with no window | multi-tenant SaaS |

**Option B is chosen** because the live `devdb` cannot tolerate a
multi-hour downtime. The trade-off is doubled storage during the run
(acceptable for a single-tenant school-sized DB).

## Phases (the playbook)

The migration runs in eleven phases, each with a focused file in this
folder. The order is mandatory.

| # | Phase | File | Window |
| --- | --- | --- | --- |
| 0 | Pre-flight: backup, credential rotation, rehearsal | this file + `11-security.md` | T-7d to T-1d |
| 1 | Engine cross-cutting tables | `01-engine-tables.md` | T-1d |
| 2 | ID conversion: `BIGINT AUTO_INCREMENT` → `CHAR(36)` UUIDv7 | `02-id-conversion.md` | T-1d (or T-0) |
| 3 | Domain-aware table renames | `03-domain-renames.md` | T-0 |
| 4 | Engine invariants per aggregate table | `04-column-additions.md` | T-0 |
| 5 | Brand removal (InfixEdu, `infix_*`, `infixedu__*`, typos) | `05-brand-removal.md` | T-0 |
| 6 | Field-level data flow (column-by-column) | `06-field-data-flow.md` | T-0 |
| 7 | Verification: row counts, FK integrity, parity | `07-verification.md` | T-0 |
| 8 | Application cutover to `devdb_v2` | `08-cutover.md` | T+0 |
| 9 | Decommission: archive `devdb` after 30 days | `09-decommission.md` | T+30d |
| — | Rollback: pre-scripted inverse | `10-rollback.md` | on demand |
| — | Security: credential + history | `11-security.md` | T-7d (independent of phases) |

## T-7d to T-1d — pre-flight

1. **Backup `devdb`** to a snapshot. Verify the backup restores on a
   clone. Store the snapshot for 90 days.
2. **Rotate the `devuser` password** on the live MySQL. Update the
   consumer's password manager; do NOT commit the new password.
3. **Clone the schema** to a sandbox DB (`devdb_rehearsal`). Re-run
   phases 1–6 on the clone. Verify phases 7. Confirm the script
   works end-to-end before T-0.
4. **Document the rollback** per `10-rollback.md`. Print the rollback
   script. Tape it to the wall.
5. **Notify the school** of the maintenance window. Even with Option B,
   there is a brief cutover moment at T+0.

## T-1d — engine tables (Phase 1)

Apply `migrations/engine/0000_engine_core.mysql.sql` to `devdb_v2`. Six unprefixed
engine cross-cutting tables are created:

- `outbox`
- `audit_log`
- `idempotency`
- `event_log`
- `schema_registry`
- `system_user` (seeded with the `SYSTEM_USER_ID` row)

No data movement. Cheap.

## T-0 — main window (Phases 2–7)

Phases 2–6 are the bulk of the work. They run against `devdb_v2`:
the consumer's ETL reads from `devdb`, transforms per the field-level
data flow in `06-field-data-flow.md`, and writes to `devdb_v2`.

Phase 7 (verification) checks that the transformation is lossless and
the engine's repositories work against the new schema.

## T+0 — cutover (Phase 8)

The consumer's application (the Laravel Schoolify app being replaced,
or the new Educore consumer app) switches its `DATABASE_URL` to
`devdb_v2`. The switch is a config change and a process restart.
Downtime is seconds.

## T+30d — decommission (Phase 9)

The legacy `devdb` is archived (dumped to cold storage, schema
frozen). The sandbox `devdb_rehearsal` is dropped.

## Aggregate counts (for capacity planning)

| Statistic | Count |
| --- | --- |
| Tables to migrate | 310 |
| Columns to migrate | ~3,500 |
| Foreign keys to reissue | ~1,200 |
| Indexes to add | ~310 (`(school_id, active_status)` on every aggregate) |
| Engine-invariant columns to add | 6 × 310 = 1,860 |
| Brand-tainted tables to rename | 8 |
| Misspelled tables to fix | 1 (`continets`) |
| Misspelled columns to fix | 1 (`path_infix_style`) |
| Brand columns to rename | 1 (`InfixBiometrics`) |
| Total expected migration time on a school-sized DB | 2–8 hours (ETL) + 30 minutes (verification) |

## What the engine does not do

The migration is a **consumer concern**. The engine library does not
run any of the phases. The `educore-storage-<db>` adapters emit
DDL that conforms to the schemas documented in
`docs/schemas/sql-dialects/`, but the consumer runs the migrations
through their own migration runner (`refinery`, `sqlx-migrate`,
`diesel`, or hand-rolled SQL).

The `0000_engine_core.sql` file is the only SQL emitted by the engine
project. The rest of the migration is the consumer's work.
