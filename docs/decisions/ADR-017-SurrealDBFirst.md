# ADR-017: SurrealDB-First Storage

## Status

Accepted, 2026-06-12.

## Context

Educore is an embeddable school-domain engine. The initial design (per
[`docs/architecture.md` § "Storage Strategy"](../architecture.md#storage-strategy)
and `AGENTS.md` § "Storage Adapters") deferred SurrealDB and MongoDB
to a future release while shipping PostgreSQL, MySQL, and SQLite as
the three primary adapters. The deferral was framed around the
assumption that a separate database server was the natural deployment
model.

The architecture document (§ "Storage Strategy") also states the
engine assumes "a single logical database per school" and
multi-tenancy via `school_id` column filtering — a pattern that maps
cleanly onto SurrealDB's `school_id` field model, not onto
SurrealDB's namespace/database tenancy model. Reusing the existing
multi-tenant column pattern keeps the engine's domain code
dialect-agnostic.

SurrealDB's embedded mode (in-process via the `surrealdb` crate)
collapses the deployment surface to a single binary. Combined with
SurrealDB's tier-2/3 features (graph traversal via
`ONLY … TRAVERSE …`, vector search via `MTREE` indexes, and
`DEFINE EVENT` triggers), this is a better fit for the engine's
"single logical database per school" assumption than running a
separate PostgreSQL/MySQL/SQLite server.

The sync engine (see `docs/build-plan.md` Phase 15+ and the
`educore-sync` plan) requires change-data-capture primitives that are
not present in the original `StorageAdapter` trait:
`watch_changes` (live change feed), `apply_snapshot` (replay a
captured snapshot into a school), `cursor_for` (monotonic per-stream
cursor position), and `advance_cursor` (commit a cursor position
atomically with the writer). These four methods must be added to the
`StorageAdapter` trait, dialect-agnostic, with a default
`NotSupported` implementation; each storage adapter overrides the
ones it can support.

## Decision

SurrealDB becomes the **primary** storage adapter. All four storage
adapters still ship at GA, but the ordering changes:

1. **SurrealDB** — Phase 0 (primary; embedded-only deployment)
2. **PostgreSQL** — Phase 1
3. **MySQL** — Phase 1
4. **SQLite** — Phase 1

### SurrealDB deployment mode: embedded only

SurrealDB is used **embedded** via the `surrealdb` crate (in-process,
no separate DB server). The engine does not depend on a running
`surrealdb start` process; the consumer's binary embeds SurrealDB
directly. Rationale: single-binary deployment matches the
architecture document's "single logical database per school"
assumption and eliminates the operational burden of a second
process.

### Multi-tenancy: `school_id` column, not SurrealDB namespaces

Multi-tenancy uses the **`school_id` column model** that the
PG/MySQL/SQLite adapters already use (per
[`docs/schemas/tenancy-schema.md`](../schemas/tenancy-schema.md)).
The engine does **not** use SurrealDB namespaces or databases per
school. Rationale: keeps the domain code dialect-agnostic; the
query AST does not need a "switch namespace" node; tenants are
filtered via the same `where_has` / `eq(SchoolField::Id, …)`
mechanism used on the relational adapters.

### Four new methods on `StorageAdapter`

The following methods are added to the `StorageAdapter` trait in
`crates/infra/storage/src/port.rs`. All four have a default
implementation that returns `StorageError::NotSupported`. Each
storage adapter overrides the ones it can implement.

| Method | Signature (sketch) | Purpose |
| --- | --- | --- |
| `watch_changes` | `async fn watch_changes(&self, school_id: SchoolId, since: Cursor) -> Result<ChangeStream>` | Live change feed from the writer to consumers (sync engine, live queries, audit projection). |
| `apply_snapshot` | `async fn apply_snapshot(&self, school_id: SchoolId, snapshot: Snapshot) -> Result<()>` | Replay a captured snapshot into a school (used by the sync engine on a fresh client). |
| `cursor_for` | `async fn cursor_for(&self, school_id: SchoolId, stream: StreamId) -> Result<Cursor>` | Read the current monotonic cursor for a per-school stream. |
| `advance_cursor` | `async fn advance_cursor(&self, school_id: SchoolId, stream: StreamId, to: Cursor) -> Result<()>` | Commit a cursor position atomically with a write. |

These four methods are **dialect-agnostic** in the trait signature;
the storage adapter chooses the native mechanism (SurrealDB live
queries for `watch_changes`; the `outbox` table for PG/MySQL/SQLite).
The default `NotSupported` implementation lets the trait land in
Phase 0 even though some adapters will not override all four until
later phases.

## Rationale

- **Single-binary deployment.** SurrealDB embedded means one process
  to ship, one process to monitor, one process to scale. The
  architecture document's "single logical database per school"
  assumption is realized without a separate DB process.
- **Tier-2/3 features for free.** SurrealDB gives the engine
  graph traversal (`TRAVERSE`), vector search (`MTREE`), and
  DB-side triggers (`DEFINE EVENT`) on the same adapter. The
  relational adapters do not have an apples-to-apples equivalent
  out of the box (PG has `pgvector` and `pgRouting`; MySQL and
  SQLite need extensions or hand-rolled code).
- **Multi-tenant model is unchanged.** Reusing the `school_id`
  column pattern means the engine's domain code, the macro-emitted
  query AST, and the storage port trait do not need a SurrealDB-
  specific tenancy mode. The 10 domain crates remain dialect-
  agnostic.
- **Sync engine requirements are now first-class.** The four new
  methods are the contract the sync engine was missing; adding
  them to the trait before Phase 0 means the sync engine (Phase 15)
  has a stable contract to depend on.
- **Four adapters still ship at GA.** SurrealDB is primary, not
  exclusive. Consumers who already operate PostgreSQL/MySQL/SQLite
  in production can still use those adapters — they just move to
  Phase 1.

## Parity surface

The engine does **not** promise SurrealDB feature parity across
all four adapters. The contract is:

| Capability | SurrealDB | PostgreSQL | MySQL | SQLite |
| --- | --- | --- | --- | --- |
| Pure CRUD (insert / update / delete / select) | ✓ | ✓ | ✓ | ✓ |
| Idempotency (`idempotency` table + `Idempotency-Key` header) | ✓ | ✓ | ✓ | ✓ |
| Outbox (`outbox` table + `event_log` projection) | ✓ | ✓ | ✓ | ✓ |
| Audit log (`audit_log` table) | ✓ | ✓ | ✓ | ✓ |
| `watch_changes` (live change feed) | ✓ (SurrealDB live queries) | Phase 1 (logical replication or `LISTEN/NOTIFY`) | Phase 1 (binlog tail) | not supported (single-writer, no live feed) |
| `apply_snapshot` | ✓ | ✓ | ✓ | ✓ |
| `cursor_for` / `advance_cursor` | ✓ | ✓ | ✓ | ✓ |
| Graph traversal (`TRAVERSE`) | ✓ | not built-in (use `pgRouting` or hand-rolled recursive CTE) | not built-in (hand-rolled recursive CTE) | not built-in (hand-rolled recursive CTE) |
| Vector search (`MTREE`) | ✓ | not built-in (use `pgvector`) | not built-in (MySQL 8.0+ has limited vector support) | not built-in (use `sqlite-vss`) |
| DB-side triggers (`DEFINE EVENT`) | ✓ | ✓ (`CREATE TRIGGER`) | ✓ (`CREATE TRIGGER`) | ✓ (`CREATE TRIGGER`) |

Consumers who need graph traversal, vector search, or DB-side
triggers on a relational adapter must add the relevant extension
themselves (`pgvector` on PG, `sqlite-vss` on SQLite, etc.). The
engine does not abstract these capabilities behind the storage
port.

## Consequences

### Phase and inventory changes

- **Phase 0** swaps `educore-storage-postgres` for
  `educore-storage-surrealdb` as the primary storage adapter. The
  Postgres adapter is scaffolded (per the build plan) and lands
  in **Phase 1** alongside MySQL and SQLite.
- The `Crate Inventory` table in `AGENTS.md` updates:
  - Add row 4a: `educore-storage-surrealdb` (adapters tier,
    Phase 0).
  - Renumber row 4 to Phase 1.
  - Update the "Storage adapters shipped" line in `AGENTS.md` §
    "Status" to list SurrealDB first.
- `docs/build-plan.md` Phase 0 and Phase 1 sections update to
  reflect the reordering.

### Documentation changes

- `docs/architecture.md` lines 264-267 (the "Storage Strategy"
  paragraph that defers SurrealDB) are rewritten to reflect the
  new ordering. The "single logical database per school" assumption
  and the `school_id` column model are unchanged.
- A new file `migrations/engine/0000_engine_core.surreal.surql`
  is the canonical DDL for the 6 engine cross-cutting tables
  (`outbox`, `audit_log`, `idempotency`, `event_log`,
  `schema_registry`, `system_user`) on SurrealDB. The
  `educore-storage-surrealdb` adapter `include_str!`s this file at
  compile time, mirroring the pattern used by the PG/MySQL/SQLite
  adapters.
- A new file `docs/schemas/sql-dialects/surrealdb.md` documents
  the SurrealQL conventions used by the adapter: type mapping
  (UUID → `string` or `record<…>`, `Decimal` → `decimal`),
  index types (`UNIQUE`, `MTREE`), graph relations (`RELATE` /
  `->`), event triggers (`DEFINE EVENT`), and the `school_id`
  tenancy model. The file is added to the
  `docs/schemas/sql-dialects/README.md` index.
- A new row is added to the `docs/schemas/sql-dialects/comparison.md`
  table for SurrealDB's parity surface (per § "Parity surface"
  above).

### Crate and trait changes

- `crates/infra/storage/src/port.rs` gains the four new methods
  (`watch_changes`, `apply_snapshot`, `cursor_for`,
  `advance_cursor`) with default `NotSupported` implementations.
- `crates/adapters/storage-surrealdb/` is the new Phase 0 crate,
  scaffolded as part of the Phase 0 work. It depends on
  `surrealdb` (pinned per [`ADR-015`](./ADR-015-ExternalCrates.md))
  and overrides all four new methods.
- The `educore-storage` port trait's rustdoc explains the sync
  engine contract; the four new methods are documented as
  "dialect-agnostic; default returns `NotSupported`; sync engine
  consumers must check capability at startup."

**Signature reconciliation note (2026-06-24):** the
signatures sketched in the table above (`watch_changes`
takes `school_id` and `since`; `cursor_for` and
`advance_cursor` take `school_id` and `stream`) disagree
with the simpler sketches in
[ADR-018 § "5. Four new methods on `StorageAdapter`"](./ADR-018-SyncEngineArchitecture.md)
(`watch_changes` no params; `cursor_for` no params;
`advance_cursor` takes a bare `cursor`). The two ADRs were
written in parallel and neither was canonicalised against
the implementation. The **authoritative signatures** live
in [`crates/infra/storage/src/port.rs`](../../crates/infra/storage/src/port.rs):
`watch_changes(filter: ChangeFilter)` (carries `school_id`
inside the filter struct), `apply_snapshot(snapshot:
SchoolSnapshot)`, `cursor_for(school_id: SchoolId)`,
`advance_cursor(school_id: SchoolId, to: VersionCursor)`.
Treat the sketches in this table as illustrative; consult
`port.rs` before implementing a new storage adapter.

### MSRV / external crate impact

- The `surrealdb` crate is added to
  [`ADR-015`](./ADR-015-ExternalCrates.md) § "Decision" with:
  (a) the chosen version, (b) alternatives considered (`sqlx`
  as the only mature alternative), (c) rationale, (d) the
  cross-compile status (Tier 1 Linux/macOS/Win; Tier 2 Android
  aarch64 status depends on `surrealdb`'s release at the time
  of pin), (e) the MSRV conflict status.
- The Phase 0 PR that adds `educore-storage-surrealdb` and the
  `surrealdb` dep must include the `ADR-015` update in the same
  commit (per `ADR-015` § "Dependency hygiene policy" rule 6).

### Sync engine contract

- The sync engine (Phase 15) now has a stable trait contract for
  the four methods. SurrealDB supports all four natively. The
  relational adapters add support for `watch_changes` in Phase 1
  (mechanism per the Parity surface table above).
- SQLite returns `NotSupported` for `watch_changes` permanently
  (single-writer, no live feed). Sync engine consumers on SQLite
  use polling + `apply_snapshot` instead of a live change feed;
  the engine's docs make this contract explicit.

## Alternatives

- **Keep PostgreSQL as the primary adapter, defer SurrealDB to
  Phase 18+.** Rejected — the single-binary deployment story is
  the strongest differentiator, and SurrealDB's tier-2/3 features
  cover real consumer needs (graph traversal for academic
  prerequisites, vector search for content search,
  DEFINE EVENT for audit triggers).
- **Use SurrealDB namespaces per school instead of `school_id`
  column.** Rejected — leaks SurrealDB-specific tenancy into the
  domain code; the macro-emitted query AST would need a
  "switch namespace" node; the relational adapters would have no
  apples-to-apples pattern to follow.
- **Use a separate SurrealDB server (`surrealdb start`) instead
  of embedded.** Rejected — defeats the single-binary deployment
  story. Embedded SurrealDB is stable and production-ready for
  the engine's per-school single-binary use case.
- **Ship only SurrealDB; drop the relational adapters.** Rejected
  — many consumers already operate PG/MySQL/SQLite in production
  and want the engine to drop in. The four adapters still ship
  at GA; SurrealDB is primary, not exclusive.
- **Add the four new methods to the trait as required (no
  default `NotSupported` impl).** Rejected — would force every
  adapter to implement all four in Phase 0, blocking the
  relational adapters' Phase 1 work. The default
  `NotSupported` impl lets the trait land cleanly; sync-engine
  consumers check capability at startup.

## See also

- [`docs/architecture.md` § "Storage Strategy"](../architecture.md#storage-strategy)
  — the storage port contract (lines 262-292 are rewritten as
  part of this ADR)
- [`docs/specs/port-contracts/storage.md`](../ports/storage.md) —
  the `StorageAdapter` trait contract; the four new methods
  are added here
- [`docs/specs/port-contracts/storage.md#future-storage-backends-deferred`](../ports/storage.md#future-storage-backends-deferred)
  — the prior deferral language that this ADR supersedes
- [`docs/schemas/sql-dialects/README.md`](../schemas/sql-dialects/README.md) —
  runtime DDL emission flow (SurrealDB adapter follows the same
  pattern as PG/MySQL/SQLite)
- [`docs/schemas/sql-dialects/surrealdb.md`](../schemas/sql-dialects/surrealdb.md)
  — the new SurrealQL conventions file
- [`docs/schemas/tenancy-schema.md`](../schemas/tenancy-schema.md) —
  the `school_id` RLS spec (unchanged by this ADR)
- [`migrations/engine/0000_engine_core.surreal.surql`](../../migrations/engine/0000_engine_core.surreal.surql)
  — the new canonical DDL for the 6 engine cross-cutting tables
- [`ADR-015`](./ADR-015-ExternalCrates.md) — external crate
  selection; the `surrealdb` row is added here
- [`ADR-016`](./ADR-016-EngineGraph.md) — engine graph (no
  change; SurrealDB adapter is just one more crate in the
  graph)
- [`AGENTS.md` § "Crate Inventory"](../../AGENTS.md#crate-inventory-per-crate-phase-assignment)
  — the per-crate phase assignment; the inventory table updates
  per this ADR
- [`AGENTS.md` § "Storage Adapters"](../../AGENTS.md#storage-adapters) —
  the adapter list; SurrealDB is added as primary
