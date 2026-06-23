# Cluster A — DDL Emission Gap

**Root cause:** `educore-storage-postgres`, `educore-storage-mysql`,
`educore-storage-sqlite`, and `educore-storage-surrealdb` do not implement
`create_schema()`. The `#[derive(DomainQuery)]` proc-macro is scaffolded but
emits no `EntityDescriptor` AST. Consequence: ~310 domain tables documented
in `docs/specs/*/tables.md` have no path from spec → macro → AST → adapter
DDL.

**Estimated findings:** ~150 (Critical-heavy)

**Source ID prefixes:** `ADAPTER-*`, `ADAPT-*`, `PAR-*`, `PORT-STORE-*`, `INFRA-QD-*`

**Blocks deploy:** Yes — without DDL emission, no adapter can host any domain.

**Estimated fix scope:** Large. Touches the macro (1 crate), the port (1 crate),
and all 4 adapter crates.

## Why these findings cluster

The end-to-end DDL pipeline has 4 stages:

1. **Spec** — `docs/specs/<domain>/tables.md` documents each table's columns,
   indexes, FKs, RLS policies. 546 rows across 14 spec files per the audit.
2. **Macro** — `#[derive(DomainQuery)]` is supposed to read a struct's
   fields and emit an `EntityDescriptor { table, columns, indexes,
   foreign_keys, rls }`. Per `docs/coverage.toml` and per
   `docs/build-plan.md:172`, every aggregate-bearing struct should have this
   derive. **Zero** structs do.
3. **Adapter** — At startup, each adapter walks the macro-emitted AST and
   emits dialect-specific DDL (`CREATE TABLE`, `CREATE INDEX`, `ALTER TABLE
   ADD CONSTRAINT`, `CREATE POLICY`). The 6 cross-cutting tables
   (outbox, audit_log, idempotency, event_log, schema_registry, system_user)
   are `include_str!`'d from `migrations/engine/0000_engine_core.<dialect>.sql`.
   The ~310 domain tables are not emitted because the macro produces nothing.
4. **Runtime** — Consumer calls `storage.create_schema().await` once per
   process. Without step 3, this method does not exist on any adapter.

Findings in this cluster fall at one of these 4 stages. Fixing the upstream
stage mechanically resolves the downstream stages.

## Representative findings

| Source | ID | Sev | Stage | One-line |
|---|---|---|---|---|
| `wave4-storage-port.md` | PORT-STORE-001 | C | 4 | `migrate()` vs `create_schema()` naming mismatch |
| `wave4-storage-port.md` | PORT-STORE-013 | C | 3 | Audit log not in same transaction as aggregate |
| `wave4-query-derive.md` | INFRA-QD-001 | C | 2 | Macro emits no AST |
| `wave4-query-derive.md` | INFRA-QD-005 | C | 2 | Macro tests reference files that don't exist |
| `wave3-storage-postgres.md` | ADAPTER-PG-001 | C | 3 | `create_schema()` not implemented |
| `wave3-storage-mysql.md` | ADAPT-MY-001 | C | 3 | `create_schema()` not implemented |
| `wave3-storage-sqlite.md` | ADAPTER-SQ-001 | C | 3 | `create_schema()` not implemented |
| `wave3-storage-surrealdb.md` | ADAPTER-SD-001 | C | 3 | `create_schema()` partial; only 6 cross-cutting tables |
| `wave4-storage-parity.md` | PAR-001 | C | 4 | 3 backend-specific parity failures admitted in test code |
| `wave4-storage-parity.md` | PAR-008 | C | 4 | Behavior matrix masks failures as `supported = true` |

## What fixing this requires

**Stage 2 (macro) — unlocks everything**

- Complete the `EntityDescriptor` AST fields. Per `wave4-core.md` CORE-002,
  missing: cursor pagination, joins, eager-load spec, dialect hints.
- Implement macro emission per the documented AST shape
  (`{ table, columns, indexes, foreign_keys, rls }`).
- Generate the `__spec_coverage__` test module per `docs/build-plan.md:172`.
- Apply `#[derive(DomainQuery)]` to **every** aggregate struct in the 10
  domain crates. Currently 0/310.

**Stage 3 (adapters) — requires stage 2**

- Implement `StorageAdapter::create_schema()` in each of the 4 adapter
  crates. Walk the macro-emitted AST; emit dialect-specific DDL.
- For SurrealDB: emit `DEFINE TABLE` / `DEFINE FIELD` / `DEFINE INDEX`.
- For Postgres: emit `CREATE TABLE` with native types, RLS policies.
- For MySQL: same minus RLS (MySQL 8 has policies but with caveats).
- For SQLite: emit `CREATE TABLE` with `WITHOUT ROWID` where appropriate.
- Honor the 7 engine invariants per `docs/schemas/database-schema.md` § 2/5/9.

**Stage 4 (runtime + tests) — requires stages 2 + 3**

- Update `educore-storage-parity` to assert each adapter emits the same set
  of tables (modulo dialect).
- Update `wave4-storage-parity.md` findings PAR-001 through PAR-031 to
  reflect the new emission.
- Add integration test: spin up each adapter, call `create_schema()`,
  verify the schema matches `migrations/engine/0000_engine_core.<dialect>.sql`
  for the 6 cross-cutting tables byte-for-byte.

## Suggested fix sequence

1. **Audit current macro output** — open `crates/infra/query-derive/src/lib.rs`,
   identify what the macro currently emits, document the gap to the AST.
2. **Complete the AST types** in `crates/infra/core/src/query.rs` (cursor,
   joins, eager-load).
3. **Implement macro emission** to match the AST.
4. **Apply `#[derive(DomainQuery)]` to one domain** (academic, the largest
   vertical slice). Verify the macro round-trips.
5. **Apply to remaining 9 domains** mechanically. Add `__spec_coverage__`
   tests per domain.
6. **Implement `create_schema()` in Postgres adapter** as the reference.
   Verify against `migrations/engine/0000_engine_core.postgres.sql` for the
   6 cross-cutting tables.
7. **Port to MySQL, SQLite, SurrealDB** adapters.
8. **Update parity suite** to assert schema equivalence.

## Verification criteria

- `cargo run -p educore-core --bin lint --features lint` reports zero
  missing-macro-application findings.
- `cargo build -p educore-storage-postgres --features runtime-ddl` produces
  the full DDL string.
- `storage.create_schema().await` on a fresh PG/MySQL/SQLite/SurrealDB
  instance creates 6 + ~310 tables.
- Parity test `parity_cross_backend_equivalence` (per
  `wave4-storage-parity.md`) passes for all 4 backends.
- `docs/coverage.toml` rows for `tables.md` go from 0 → ~310.

## Risk if left unfixed

- No adapter can host any domain. The engine is non-deployable.
- Every `wave1-*` and `wave6-*` finding about "struct not in aggregate.rs"
  becomes irrelevant — the gap is upstream, in the macro.
- Every `wave3-*` adapter finding about "DDL not emitted" remains.
- The 4 storage adapters in `crates/adapters/` become unused code.

## Cross-cluster dependencies

- **Unblocks:** Cluster C (spec↔code drift can be mechanically validated
  per spec row once the macro emits per struct), Cluster F (adapter gaps
  become feasible to verify), Cluster D (lint module can verify macro
  output).
- **Depends on:** Cluster D (foundation crate — the AST types live there).

## Files involved

- `crates/infra/query-derive/src/lib.rs` (the macro)
- `crates/infra/core/src/query.rs` (the AST)
- `crates/adapters/storage-postgres/src/schema.rs` (Postgres DDL)
- `crates/adapters/storage-mysql/src/schema.rs` (MySQL DDL)
- `crates/adapters/storage-sqlite/src/schema.rs` (SQLite DDL)
- `crates/adapters/storage-surrealdb/src/schema.rs` (SurrealDB DDL)
- `crates/tools/storage-parity/` (the parity suite)
- `migrations/engine/0000_engine_core.<dialect>.sql` (the 6 cross-cutting
  tables' canonical DDL)
