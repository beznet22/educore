# Cluster D — Foundation Crate Gaps

**Root cause:** `educore-core` (the foundation crate depended on by 33
other crates) ships with several declared-but-not-implemented pieces:
the `lint` sub-module, the `EntityDescriptor` AST fields, the
`IdempotencyRecord` contract, and the `Transaction` tenant-context
plumbing. The storage port trait has multiple gaps. The umbrella crate
has re-export gaps.

**Estimated findings:** ~70 (Critical + Medium + Low mix)

**Source ID prefixes:** `CORE-*`, `PORT-STORE-*` (subset), `INFRA-*`, `UMB-*`

**Blocks deploy:** Yes (partial). The lint module is the build-plan's own
gating mechanism (per `docs/build-plan.md:72`); without it, every other
fix is un-verifiable.

**Estimated fix scope:** Medium. One crate (`educore-core`) plus
`educore-storage` and `educore`. High leverage per line of code.

## Why these findings cluster

The foundation crate is the dependency graph's root. Every other crate
depends on its types (`DomainError`, identifier types, value objects,
the AST). When the foundation is incomplete, downstream code either:

- Doesn't use it (e.g., re-implements types — `wave4-core.md` CORE-008)
- Uses an incomplete version (e.g., `Transaction` without
  `TenantContext` — `wave4-storage-port.md` PORT-STORE-002)
- Pretends to use it (e.g., a struct that looks like it implements
  `DomainEvent` but doesn't — `wave2-events.md` CC-EVT-004)

The audit's lint module (`educore-core::lint`) was supposed to catch
these. It was declared at `crates/infra/core/src/lint.rs:7-20` but never
implemented. Per `docs/build-plan.md:72`, this was a Phase 0.5 fix-up
item that was never closed.

## Representative findings

| Source | ID | Sev | Topic | One-line |
|---|---|---|---|---|
| `wave4-core.md` | CORE-001 | C | lint | `educore-core::lint` declared at line 7 but not implemented |
| `wave4-core.md` | CORE-002 | C | AST | `EntityDescriptor` AST missing cursor / joins / eager-load |
| `wave4-core.md` | CORE-003 | C | errors | `DomainError` missing variants for `IdempotencyConflict` |
| `wave4-core.md` | CORE-004 | H | lint | Tier-boundary check declared but not implemented |
| `wave4-core.md` | CORE-005 | H | lint | Anti-pattern check (unwrap/as/Value) not implemented |
| `wave4-storage-port.md` | PORT-STORE-001 | C | port | `migrate()` vs `create_schema()` naming |
| `wave4-storage-port.md` | PORT-STORE-002 | C | port | `Transaction` carries no `TenantContext` |
| `wave4-storage-port.md` | PORT-STORE-005 | C | port | `Repository<A>` is generic, not per-aggregate |
| `wave4-storage-port.md` | PORT-STORE-008 | C | port | `EventLogFilter` has no cursor |
| `wave4-storage-port.md` | PORT-STORE-013 | C | port | Audit not in same transaction as aggregate |
| `wave4-storage-port.md` | PORT-STORE-014 | H | port | Bulk-insert silent fallback to default |
| `wave4-umbrella.md` | UMB-001 | C | umbrella | `educore-cli` missing from umbrella |
| `wave4-umbrella.md` | UMB-002 | C | umbrella | `educore-query-derive` missing from umbrella |
| `wave4-umbrella.md` | UMB-005 | H | umbrella | No `[features]` despite sync spec mandating feature gating |

## What fixing this requires

**`educore-core::lint` (CORE-001)**

Implement the lint per `docs/build-plan.md:1866-1910`:

1. **Spec → code direction:**
   - Walk `docs/specs/<domain>/tables.md`. For each table row, find a
     `#[derive(DomainQuery)]` in `crates/domains/<domain>/src/aggregate.rs`
     with matching table name. Report missing.
   - Walk `docs/commands/<domain>.md`. For each command, find a handler
     in `crates/domains/<domain>/src/commands.rs`. Report missing.
   - Walk `docs/events/<domain>.md`. For each event, find a struct in
     `crates/domains/<domain>/src/events.rs`. Report missing.
   - Walk `migrations/engine/*.sql`. For each table, find a
     `create_<table>_ddl()` in each adapter crate. Report missing.

2. **Code → spec direction:**
   - Walk all public structs/commands/events. For each, check a spec
     row exists. Report undocumented.

3. **Anti-patterns:**
   - `unimplemented!()`, `todo!()`, `// TODO: implement` in production
     code (test code exempt via `#[cfg(test)]`).
   - `as` on numerics in domain crates.
   - `serde_json::Value` in domain code.
   - `HashMap<String, T>` for domain data.

4. **Parity:** every `DomainQuery` call ↔ spec row, both directions.

5. **Coverage matrix sync:** read `docs/coverage.toml`, verify:
   - Every `Tested` row has a `tests` path that exists.
   - Every code-defined item has a row.
   - No row references a missing spec/command/event file.

**`EntityDescriptor` AST (CORE-002)**

Complete the AST in `crates/infra/core/src/query.rs`:

- `cursor: Option<CursorSpec>` — for pagination
- `joins: Vec<JoinSpec>` — for eager-loading
- `eager_load: Vec<EagerLoadSpec>` — for the `.with(...)` API
- `dialect_hints: DialectHints` — for adapter-specific emit

**`Transaction` port (PORT-STORE-002, 013, 014)**

Add:

- `tenant: TenantContext` field — every transaction is scoped
- `audit_handle: AuditLogHandle` — atomic audit write in same tx
- `bulk_insert_*` methods — non-default implementation
- `savepoint(name) -> Self` — for nested transactions
- `Drop` impl — explicit rollback on drop (current: relies on
  `sqlx::Transaction::Drop`)

**`Repository` port (PORT-STORE-005)**

Refactor from `Repository<A>` to named per-aggregate handles:

```rust
trait StudentRepo { fn find(&self, ...); fn save(&self, ...); ... }
trait GuardianRepo { ... }
```

This matches the spec's per-aggregate repository pattern.

**`EventLogFilter` (PORT-STORE-008)**

Add `cursor: Option<Cursor>` field. Pagination via cursor, not offset.

**Umbrella crate (UMB-001, 002, 005)**

- Add `educore-cli` re-export under `educore::cli`.
- Add `educore-query-derive` re-export under `educore::query_derive`.
- Add `[features]` table with at least: `default`, `sync`, `lint`.

## Suggested fix sequence

1. **`EntityDescriptor` AST completion** — 1 day. Unblocks macro work.
2. **`Transaction` port rewrite** — 3-5 days. Includes
   `TenantContext`, `audit_handle`, `savepoint`, `Drop`. Touches all 4
   adapter crates (impl updates).
3. **`Repository` port refactor** — 1-2 weeks. Mechanical migration
   from `Repository<A>` to named handles per aggregate. Touches every
   domain crate's repository.rs.
4. **`EventLogFilter` cursor** — 1-2 days. Update impls in 4 adapters.
5. **`educore-core::lint`** — 2-4 weeks. The big one. Start with the
   spec→code direction (smallest scope), then anti-patterns, then
   parity, then matrix sync.
6. **Umbrella re-exports** — 1 day. Plus feature gate plumbing.

## Verification criteria

- `cargo run -p educore-core --bin lint --features lint` runs and exits 0
  on a green codebase.
- `cargo build --workspace` still compiles after the `Transaction`
  rewrite.
- `cargo test --workspace` passes after the `Repository` refactor.
- `educore::*` exports `cli` and `query_derive` as accessible paths.
- `DomainError::IdempotencyConflict` exists and is matchable.

## Risk if left unfixed

- The coverage matrix lies (`docs/coverage.toml` says "Tested" but
  reality is partial). CI gates are not enforceable.
- Every cross-cutting change (e.g., adding a new aggregate type) requires
  manual spec-update coordination because there's no machine-checked
  invariant.
- The umbrella's missing re-exports mean consumers can't use the affected
  crates through the public surface.
- The Transaction port's missing `TenantContext` means tenant safety is
  per-adapter, not enforced by the type system.

## Cross-cluster dependencies

- **Unblocks:** Cluster A (macro can emit complete AST), Cluster E (lint
  detects violations mechanically), Cluster F (adapter port contract
  becomes stable).
- **Depends on:** None — this is the dependency-graph root.

## Files involved

- `crates/infra/core/src/lint.rs` (the lint, declared at line 7)
- `crates/infra/core/src/bin/lint.rs` (the CLI binary)
- `crates/infra/core/src/query.rs` (the AST)
- `crates/infra/core/src/error.rs` (DomainError variants)
- `crates/infra/storage/src/transaction.rs` (Transaction port)
- `crates/infra/storage/src/repository.rs` (Repository port)
- `crates/infra/storage/src/event_log.rs` (EventLogFilter)
- `crates/educore/src/lib.rs` (umbrella re-exports)
- All 4 adapter crates (impl updates after port changes)
- All 10 domain crates (repository refactor)
