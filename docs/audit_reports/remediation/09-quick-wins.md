# Quick Wins — First Things to Fix

**Purpose:** List 12-15 specific findings that are:
- High visibility (customer-facing or developer-experience)
- Don't block anything else
- Small (1-3 file changes per item)
- Unambiguously fixable

These can be picked up by anyone, in any order, and shipped as a PR
without coordinating with the larger cluster work.

## QW-1: Implement `cargo run -p educore-core --bin lint` exit code

**Source:** `wave4-core.md` CORE-001
**Effort:** 1 hour (just make it return `ExitCode::SUCCESS` instead of
the placeholder)
**Visibility:** Developer-facing. CI gate.
**Notes:** The lint binary at `crates/infra/core/src/bin/lint.rs:42`
already returns `ExitCode::SUCCESS` when `report.is_clean()`. Make
sure the binary runs without panicking. Even an empty pass-through
satisfies the CI step.

## QW-2: Fix `BatchReceipt::is_fully_accepted` bug

**Source:** `wave2-events.md` CC-EVT-007
**Effort:** 1-2 hours
**Visibility:** Bug; affects every batch publish.
**Notes:** The function returns true on partial failure. One-line fix.

## QW-3: Replace `Box::leak` in storage adapters with interning

**Source:** `wave3-storage-postgres.md` ADAPTER-PG-011,
`wave3-storage-mysql.md` ADAPT-MY-007, `wave3-storage-sqlite.md`
ADAPTER-SQ-006
**Effort:** 1-2 days per adapter
**Visibility:** Performance + memory leak fix. Affects every command.
**Notes:** The pattern `Box::leak(...)` for `&'static str` parameters
leaks memory per command. Replace with `OnceCell` or `LazyLock` keyed
by the `command_type` value. Mechanical.

## QW-4: Add explicit `Drop` impl to `Transaction` port

**Source:** `wave4-storage-port.md` PORT-STORE-014
**Effort:** 1 day
**Visibility:** Correctness; affects every uncommitted transaction.
**Notes:** Currently, dropping a `Transaction` without committing
silently commits (via `sqlx::Transaction::Drop`). This is the opposite
of the engine's explicit-rollback contract. Add an explicit `Drop` that
calls `rollback().await` (with `tokio::runtime::Handle::current()`).

## QW-5: Implement `Outbox::pending_count` without materialising rows

**Source:** `wave4-storage-port.md` PORT-STORE-028
**Effort:** 1 day per adapter
**Visibility:** Memory fix.
**Notes:** The default `pending_count` impl calls `pending(u32::MAX)`
and counts. For schools with millions of outbox rows, this is a memory
explosion. Add `SELECT COUNT(*)` to each adapter.

## QW-6: Add `school_id` index to all 6 cross-cutting tables

**Source:** `wave3-storage-mysql.md` ADAPT-MY-013
**Effort:** 1 hour per adapter (3 indexes per table × 6 tables)
**Visibility:** Performance; affects every per-school query.
**Notes:** All 4 adapters are missing indexes on `school_id` for the
cross-cutting tables (`outbox`, `audit_log`, `idempotency`, `event_log`,
`schema_registry`, `system_user`). Trivial DDL change.

## QW-7: Implement `JWT_SECRET` env-var loading with warning

**Source:** `wave3-auth.md` ADAPT-AUTH-002
**Effort:** 1-2 days
**Visibility:** Security; affects every auth flow.
**Notes:** Currently the JWT signing key is randomly generated per
process. This means every restart invalidates all sessions. Read from
`JWT_SECRET` env var; if absent, log a warning and generate a random
key (for dev) or fail (for production).

## QW-8: Add rate-limiting middleware to auth endpoints

**Source:** `wave3-auth.md` ADAPT-AUTH-007
**Effort:** 3-5 days
**Visibility:** Security; prevents brute-force.
**Notes:** No rate-limit on `/login`, `/register`, `/forgot-password`.
Add a `tower-governor` or hand-rolled rate limiter. Per-IP, per-endpoint.

## QW-9: Add `wave4-umbrella.md` re-exports for `cli` and `query_derive`

**Source:** `wave4-umbrella.md` UMB-001, UMB-002
**Effort:** 1 hour
**Visibility:** Public API; affects every consumer.
**Notes:** `crates/educore/src/lib.rs` is missing `pub use
educore_cli as cli;` and `pub use educore_query_derive as query_derive;`.
Two-line fix.

## QW-10: Add `[features]` table to `educore` umbrella

**Source:** `wave4-umbrella.md` UMB-005
**Effort:** 1 day
**Visibility:** Public API; enables opt-in crate subsets.
**Notes:** Per the sync spec, the umbrella should expose feature flags
for `sync`, `lint`, `runtime-ddl`, `parity`. Mechanical Cargo.toml
edit.

## QW-11: Add explicit `select!` cancellation to all in-process bus handlers

**Source:** `wave3-event-bus.md` ADAPT-EB-007
**Effort:** 3-5 days
**Visibility:** Correctness; prevents hangs.
**Notes:** The in-process event bus uses `tokio::sync::broadcast` which
silently drops messages when there are no receivers. Add explicit
slow-consumer detection and per-subscriber timeout.

## QW-12: Add `Idempotency::record` returns `Conflict`

**Source:** `wave4-storage-port.md` PORT-STORE-011 (and per-adapter
wave3-* findings)
**Effort:** 2-3 days per adapter
**Visibility:** Correctness; enables idempotent-command detection.
**Notes:** Currently `record` always returns `Ok`, never `Conflict`.
Add the duplicate-detection logic. Each adapter implements its own
(unique index, upsert, etc.).

## QW-13: Add `Outbox` partition enforcement

**Source:** `wave4-testkit.md` TOOL-TK-004
**Effort:** 1-2 days per adapter
**Visibility:** Tenant safety; prevents cross-school outbox reads.
**Notes:** `OutboxHandle::pending(limit)` returns the first `limit`
rows regardless of school. Add `school_id` filter to each adapter's
SQL.

## QW-14: Add explicit `Send + Sync` bounds to all port trait methods

**Source:** `wave4-core.md` CORE-014
**Effort:** 1 day (mostly compile-error fixing)
**Visibility:** Enables async use across thread boundaries.
**Notes:** Several port trait methods lack explicit `Send`/`Sync`
bounds. The async_trait macro usually infers these, but the
`wave4-core.md` audit found 3-4 cases where they don't.

## QW-15: Add `dbg!()` removal in domain code

**Source:** `wave5-docs-2.md` DOC-2-018 (and wave1-* domain audits)
**Effort:** 1 day
**Visibility:** Performance + correctness.
**Notes:** Several domain crates have leftover `dbg!()` calls in
production paths. Search-and-replace.

## Summary table

| ID | Source finding | Effort | Severity | Cluster |
|---|---|---|---|---|
| QW-1 | CORE-001 | 1h | C | D |
| QW-2 | CC-EVT-007 | 1-2h | C | B |
| QW-3 | ADAPTER-PG-011, etc. | 1-2d | C | F |
| QW-4 | PORT-STORE-014 | 1d | C | D + F |
| QW-5 | PORT-STORE-028 | 1d | H | F |
| QW-6 | ADAPT-MY-013 | 1h | M | F |
| QW-7 | ADAPT-AUTH-002 | 1-2d | C | F |
| QW-8 | ADAPT-AUTH-007 | 3-5d | C | F |
| QW-9 | UMB-001, UMB-002 | 1h | C | D |
| QW-10 | UMB-005 | 1d | H | D |
| QW-11 | ADAPT-EB-007 | 3-5d | C | F |
| QW-12 | PORT-STORE-011 | 2-3d | C | F |
| QW-13 | TOOL-TK-004 | 1-2d | C | B + F |
| QW-14 | CORE-014 | 1d | M | D |
| QW-15 | DOC-2-018 | 1d | M | E |

Total estimated effort for all 15 quick wins: ~3-4 weeks (1 person).

## How to use this list

1. Pick any item that has free capacity in the team.
2. Open a PR titled `remediation: QW-N <one-line description>`.
3. Reference the source finding ID in the PR body.
4. Land the PR.
5. Move on to the next item.

None of these 15 items blocks any other fix. All are independently
mergeable. None requires coordination with the cluster-sequencing work.
