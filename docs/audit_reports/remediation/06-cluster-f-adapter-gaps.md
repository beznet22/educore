# Cluster F — Adapter Port-Contract Gaps

**Root cause:** The 4 storage adapters (`educore-storage-postgres`,
`educore-storage-mysql`, `educore-storage-sqlite`,
`educore-storage-surrealdb`) and the in-process event-bus adapter do not
implement the storage port contract in full. The 6 port-adapter crates
(`educore-auth`, `educore-event-bus`, `educore-files`,
`educore-integrations`, `educore-notify`, `educore-payment`) similarly
have port-trait gaps.

**Estimated findings:** ~250 (Critical-heavy per adapter)

**Source ID prefixes:** `ADAPTER-*`, `ADAPT-*`, `PAR-*`, `ADAPT-EB-*`

**Blocks deploy:** Yes (per adapter — any adapter with Critical gaps is
not deployable for production use).

**Estimated fix scope:** Large. 4 storage adapters + 6 port adapters ×
full port surface. Estimated 2-3 months of focused work.

## Why these findings cluster

The storage port (`crates/infra/storage/src/`) defines:

- `StorageAdapter` (with `create_schema`, `apply_command`, `query`,
  `begin_tx`, `commit_tx`, `rollback_tx`)
- `Transaction` (with sub-port handles)
- `Outbox`, `AuditLog`, `Idempotency`, `EventLog` (sub-ports)
- `Repository<A>` (per-aggregate CRUD)
- `Query` (the AST consumer)

The audit found that **no adapter implements the full surface**. Each
adapter has 5-15 Critical gaps. The pattern repeats across adapters:

- `create_schema()` doesn't exist (or only emits 6 cross-cutting tables)
- `Transaction::commit` is a no-op or `sqlx::Transaction::Drop`-dependent
- `Outbox` partition enforcement is missing
- `Idempotency::record` always returns `Ok`, never `Conflict`
- `Repository::save` doesn't stamp `created_by` / `updated_by`
- SQL queries are partially parameterized (some string interpolation)

For the port-adapter crates:

- `educore-auth`: anonymous credential returns SYSTEM_USER_ID with full
  session; no rate-limit; no JWT secret rotation; no cross-process
  revocation.
- `educore-notify`: email/SMS providers have retry logic but no DLQ;
  provider failures downgrade silently.
- `educore-payment`: webhook signature verification incomplete;
  idempotency-key enforcement partial; no PCI-scope marker.
- `educore-files`: upload/download paths don't honor tenant context.
- `educore-integrations`: webhook retries forever; no exponential
  backoff cap; no signing-key rotation.
- `educore-event-bus`: in-process ack/nack are no-ops; no outbox drain;
  broadcast channel silent failures.

## Representative findings

### Storage adapters

| Source | ID | Sev | Adapter | One-line |
|---|---|---|---|---|
| `wave3-storage-postgres.md` | ADAPTER-PG-001 | C | postgres | `create_schema()` not implemented |
| `wave3-storage-postgres.md` | ADAPTER-PG-005 | C | postgres | `Transaction::commit` is `sqlx::Transaction::Drop`-dependent |
| `wave3-storage-postgres.md` | ADAPTER-PG-011 | C | postgres | `Box::leak` per command_type on every lookup |
| `wave3-storage-mysql.md` | ADAPT-MY-001 | C | mysql | `create_schema()` not implemented |
| `wave3-storage-mysql.md` | ADAPT-MY-005 | C | mysql | `commit`/`rollback` are no-ops (broken ACID) |
| `wave3-storage-mysql.md` | ADAPT-MY-009 | C | mysql | Idempotency never returns `Conflict` (always overwrites) |
| `wave3-storage-sqlite.md` | ADAPTER-SQ-001 | C | sqlite | `create_schema()` not implemented |
| `wave3-storage-sqlite.md` | ADAPTER-SQ-006 | C | sqlite | `Box::leak` per command_type on every lookup |
| `wave3-storage-surrealdb.md` | ADAPTER-SD-001 | C | surrealdb | `create_schema()` partial; only 6 cross-cutting tables |
| `wave3-storage-surrealdb.md` | ADAPTER-SD-005 | C | surrealdb | `apply_snapshot`/`watch_changes`/`cursor_for`/`advance_cursor` are stubs |
| `wave4-storage-parity.md` | PAR-001 | C | all | 3 backend-specific parity failures admitted in test code |
| `wave4-storage-parity.md` | PAR-008 | C | all | Behavior matrix masks failures as `supported = true` |

### Port adapters

| Source | ID | Sev | Adapter | One-line |
|---|---|---|---|---|
| `wave3-auth.md` | ADAPT-AUTH-001 | C | auth | Anonymous credential returns fully-formed session |
| `wave3-auth.md` | ADAPT-AUTH-002 | C | auth | Default JWT signing key is random per process |
| `wave3-auth.md` | ADAPT-AUTH-007 | C | auth | No rate-limit / lockout / brute-force protection |
| `wave3-event-bus.md` | ADAPT-EB-001 | C | event-bus | In-process ack/nack are no-ops |
| `wave3-event-bus.md` | ADAPT-EB-005 | C | event-bus | No outbox drain in the in-process adapter |
| `wave3-notify.md` | ADAPT-NOT-005 | C | notify | No DLQ; provider failures downgrade silently |
| `wave3-payment.md` | ADAPT-PAY-001 | C | payment | Idempotency-key contract partial |
| `wave3-payment.md` | ADAPT-PAY-005 | C | payment | Webhook signature verification incomplete |
| `wave3-files.md` | ADAPT-FILE-003 | C | files | Upload/download paths don't honor tenant context |
| `wave3-integrations.md` | ADAPT-INT-005 | C | integrations | Webhook retries forever; no exponential backoff cap |

## What fixing this requires

**Storage adapters (per adapter)**

For each of the 4 storage adapters:

1. Implement `create_schema()` (depends on cluster A's macro emission).
2. Implement proper `Transaction::commit` / `rollback` (not relying on
   `Drop`).
3. Add `TenantContext` to every public method.
4. Implement `Outbox::pending` partition enforcement.
5. Implement `Idempotency::record` that can return `Conflict`.
6. Implement `Repository::save` that stamps `created_by` / `updated_by`.
7. Parameterize all SQL queries (no string interpolation).
8. Implement `change_stream` (`apply_snapshot`, `watch_changes`,
   `cursor_for`, `advance_cursor`).

**Port adapters (per adapter)**

For each of the 6 port adapters:

1. **`auth`**: implement proper JWT secret loading (env var with
   default-and-warn); rate limiting; cross-process revocation
   (Redis or DB-backed set); lockout policy; MFA enforcement; OAuth
   state parameter validation.
2. **`event-bus`**: implement proper ack/nack; outbox drain;
   backpressure; subscriber cancellation safety; dead-letter queue.
3. **`notify`**: implement DLQ; provider failover; exponential backoff;
   rate-limit per provider; bounce handling.
4. **`payment`**: implement webhook signature verification;
   idempotency-key enforcement on every charge/refund; PCI-scope
   markers; refund flow; partial capture; multi-currency.
5. **`files`**: implement tenant context on every operation;
   virus-scan integration; presigned URL expiry; storage quota
   enforcement.
6. **`integrations`**: implement exponential backoff cap (e.g.,
   5 retries with jitter); webhook signing-key rotation; replay
   protection (nonce + timestamp window); circuit breaker.

## Suggested fix sequence

1. **Storage adapters, Postgres first** (the reference). Use the
   `migrations/engine/0000_engine_core.postgres.sql` for the 6
   cross-cutting tables as the canonical DDL.
2. **Storage adapters, MySQL + SQLite** (similar to Postgres; dialect
   tweaks).
3. **Storage adapter, SurrealDB** (different model; DEFINE statements
   instead of CREATE TABLE).
4. **`educore-auth`** (highest customer-facing risk; no auth = no
   product).
5. **`educore-event-bus`** (depends on cluster B; outbox relay).
6. **`educore-payment`** (PCI scope; highest security stakes).
7. **`educore-notify`, `educore-files`, `educore-integrations`**
   (lower criticality; can be last).

For each adapter, fix Critical findings first, then High, then Medium,
then Low.

## Verification criteria

- For each storage adapter:
  - `cargo test -p educore-storage-<db>` passes
  - `create_schema()` round-trip on a fresh DB instance produces the
    expected schema
  - Parity test `parity_cross_backend_equivalence` passes for the
    adapter
- For each port adapter:
  - Integration tests in `crates/adapters/<adapter>/tests/` (newly
    added) pass
  - Idempotency, retry, and backoff behaviors are exercised

## Risk if left unfixed

- **Storage adapters**: any consumer running on Postgres/MySQL/SQLite/
  SurrealDB will encounter broken transactions, missing audit, and
  silent overwrites.
- **`auth`**: anonymous credentials get full sessions; JWT tokens are
  forgeable across processes; no brute-force protection.
- **`payment`**: charge/refund idempotency is unenforced; webhook
  spoofing possible.
- **`notify`**: provider failures silently lost; no retry recovery.
- **`files`**: cross-tenant file access possible.
- **`integrations`**: webhook storms possible; signing keys never
  rotate.

## Cross-cluster dependencies

- **Depends on:** Cluster A (DDL emission), Cluster B (workflow infra
  for event-bus), Cluster D (port trait stability).
- **Unblocks:** Cluster C (per-domain parity tests can validate adapter
  behavior).

## Files involved

- `crates/adapters/storage-postgres/src/` (~15 files)
- `crates/adapters/storage-mysql/src/` (~15 files)
- `crates/adapters/storage-sqlite/src/` (~15 files)
- `crates/adapters/storage-surrealdb/src/` (~15 files)
- `crates/adapters/auth/src/` (~10 files)
- `crates/adapters/event-bus/src/` (~5 files)
- `crates/adapters/payment/src/` (~10 files)
- `crates/adapters/notify/src/` (~10 files)
- `crates/adapters/files/src/` (~5 files)
- `crates/adapters/integrations/src/` (~10 files)

Total: ~110 files across 10 adapter crates.
