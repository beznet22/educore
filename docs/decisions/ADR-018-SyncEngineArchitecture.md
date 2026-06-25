# ADR-018: Sync Engine Architecture

## Status

Accepted, 2026-06-12.

Amended 2026-06-25: `sync-inprocess` lives in `crates/cross-cutting/`
(not `crates/adapters/` as originally specified in § 3). It is a
reference implementation for tests, not a port implementation, so
the cross-cutting tier is the correct home per ADR-013's tier system.

## Context

Educore is **embeddable**: a consumer links the engine into a
mobile app, a desktop app, a Tauri shell, a WASM bundle, or
a backend service. The first three categories are
**local-first** by necessity (rural schools, military
bases, field trips, intermittent satellite links — see
[ADR-008](./ADR-008-OfflineFirst.md)). The fourth category
is the **central** server that every device syncs with.

The naive design — embed a "sync engine" inside every
domain crate — pollutes the domain with transport
concerns. A second naive design — make the consumer
hand-roll a worker from scratch — punishes everyone who
just wants a working offline-first app in a few hours.

The existing design (per
[`docs/guides/saas-backend.md` § "The Sync Engine"](../guides/saas-backend.md#the-sync-engine-offline--central),
lines 464-573) is a **separate worker process** that
reads from the local SQLite outbox and POSTs to the
central API. That design is correct and it stays. We are
**extending** it with an in-process reference
implementation so a consumer can ship an offline-first
app on day one without standing up the worker
infrastructure.

The engine already provides the necessary primitives
from prior ADRs:

- The **outbox** lives in `StorageAdapter::Transaction`
  (see [`docs/ports/storage.md`](../ports/storage.md#outbox)).
- **Idempotent commands** per
  [ADR-014](./ADR-014-Idempotency.md) — every command
  carries an `idempotency_key`; a retry returns the
  original outcome.
- **Typed events** with `event_id` (UUIDv7), `version`,
  and `etag` per [ADR-008](./ADR-008-OfflineFirst.md).
- The **audit log** per the `AuditSink` port.

What is missing is a reference implementation of the
relay and a wire protocol that makes the relay swappable.

## Decision

The engine splits the sync problem into **primitives**,
**wire protocol**, and **adapters**, with two adapter
implementations sharing the same protocol.

### 1. The engine ships the primitives

The engine owns:

- The **outbox** (in the storage adapter transaction).
- The **idempotency table** (in the storage adapter
  transaction).
- The **event log** (in the storage adapter transaction).
- The **`AuditSink` port** for conflict events.
- **Typed event envelopes** with `event_id`, `version`,
  `etag`, `school_id`, `tenant_id`, `correlation_id`.

The engine does **not** ship a relay, a transport, or a
conflict resolver. Those are consumer concerns.

### 2. The consumer's sync-engine worker (the production shape)

The production shape — already documented in
[`docs/guides/saas-backend.md` lines 489-496](../guides/saas-backend.md#the-sync-engine-offline--central)
— is a **separate worker process** that:

1. Reads pending events from the local outbox.
2. POSTs them to `central /v1/sync`.
3. On `200`: deletes the events from the outbox.
4. On `4xx`: marks the event as conflicted, alerts the
   user, stops retrying.
5. On `5xx` / network error: backs off and retries.

The worker is a **consumer concern**. The engine does
not ship this process; the SaaS backend guide
(`docs/guides/saas-backend.md` lines 899-902) describes
the deployment. A consumer who needs the production
shape reads the guide and writes the worker for their
language of choice (Rust, Go, Python — the wire
protocol is language-agnostic).

### 3. The engine also ships an in-process reference implementation

For consumers who want offline-first without standing
up a worker, the engine ships:

- **`crates/cross-cutting/sync/`** (package
  `educore-sync`) — the sync **primitives**: cursor
  tracking, change observation, snapshot
  (de)serialization, retry policy, conflict mapping.
  These are reusable in any deployment shape.
- **`crates/cross-cutting/sync-inprocess/`** (package
  `educore-sync-inprocess`) — an in-process reference
  implementation
  that drains the local outbox and applies remote
  snapshots **without** any network I/O. The "remote"
  store is just a `StorageAdapter` on the same process.
  Useful for testing, single-process embedded apps, and
  "30 minutes to a working offline-first app" demos.
- **`crates/adapters/sync-http/`** (package
  `educore-sync-http`) — a thin HTTP client that talks
  the wire protocol against a remote `/v1/sync`
  endpoint. The consumer's worker process wraps this
  client.
- **`crates/adapters/sync-null/`** (package
  `educore-sync-null`) — a no-op adapter. Returns
  `NotSupported` from every method. The default when
  the `sync` feature is disabled.

The wire protocol is **identical** across the
in-process and HTTP adapters. The protocol spec lives
at [`docs/ports/sync.md`](../ports/sync.md). Swapping
adapters is a one-line change in the engine builder:

```rust
// Default (no sync): the engine has no sync capability
let engine = Engine::builder()
    .storage(postgres)
    .build()?;

// Local-first (default when `sync` feature is on):
// the in-process adapter moves events between two
// StorageAdapter instances on the same process.
let engine = Engine::builder()
    .storage(postgres)
    .sync(EducoreSyncAdapter::in_process(local_sqlite, remote_postgres))
    .build()?;

// Production (a separate worker process):
// the HTTP adapter talks the wire protocol.
let engine = Engine::builder()
    .storage(postgres)
    .sync(WorkerHttpSyncAdapter::connect(central_url, device_token))
    .build()?;
```

### 4. The `SyncAdapter` is a build feature

The `SyncAdapter` is **opt-in** via a feature flag on
the umbrella crate:

```toml
# crates/educore/Cargo.toml
[features]
default = []
sync = ["educore-sync", "educore-sync-inprocess"]
```

Without the `sync` feature, the engine has **no** sync
capability (the `sync()` builder method is gated behind
the feature). With the feature on, consumers pick:

- `EducoreSyncAdapter::in_process(local, remote)` —
  default; the in-process adapter; local-first.
- `WorkerHttpSyncAdapter::connect(url, token)` —
  production; the HTTP adapter against a remote worker.
- A consumer may write their own adapter (e.g. MQTT,
  gRPC, file drop) by implementing the `SyncAdapter`
  trait.

The feature is **default-off** because the engine's
non-sync crates (domains, audit, events) do not need
it. A consumer using only the audit log should not
pay the dependency cost of the sync adapters.

**Feature-flag reconciliation note (2026-06-24):** the
`[features]` block shown above (`default = []; sync = [...]`)
has not landed in `crates/educore/Cargo.toml`. As of the
2026-06-24 audit the umbrella's `Cargo.toml` lists
`educore-sync` and `educore-sync-inprocess` as unconditional
dependencies (no `[features]` block, no `cfg(feature =
"sync")` gates on the SDK). See FINDING 27 in
`docs/audit_reports/findings/wave5-docs-2.md`. The feature
flag is the correct design but is a follow-up implementation
item, not part of this ADR's acceptance.

**Sync-crate location note (2026-06-24, resolved
2026-06-25 by Status amendment):** the directory
paths quoted in section 3 above
(`crates/cross-cutting/sync/`,
`crates/cross-cutting/sync-inprocess/`,
`crates/adapters/sync-http/`,
`crates/adapters/sync-null/`) disagree with the filesystem
in one remaining way: the `sync-http` and `sync-null`
crates named in section 3 do not yet exist on disk —
only `educore-sync` and `educore-sync-inprocess` are
scaffolded. (The earlier `crates/adapters/sync-inprocess/`
discrepancy is resolved by the 2026-06-25 Status
amendment: the cross-cutting tier is the correct home
for a reference implementation, not a port
implementation.) The `crates/cross-cutting/sync/`
location for the primitives crate is correct.

### 5. Four new methods on `StorageAdapter`

The sync engine needs to observe local changes and
bulk-apply snapshots without bypassing the command
path. Four new methods on `StorageAdapter`:

```rust
pub trait StorageAdapter: Send + Sync {
    // ... existing methods ...

    /// Stream change events from the local store.
    /// Default impl returns NotSupported.
    fn watch_changes(&self) -> Result<ChangeStream>;

    /// Apply a server-side snapshot to the local store.
    /// The snapshot is a sequence of typed events; the
    /// local store runs each event through the command
    /// layer to validate it before applying.
    /// Default impl returns NotSupported.
    fn apply_snapshot(&self, snapshot: Snapshot) -> Result<()>;

    /// Return the highest event_id the local store
    /// has applied. Used to anchor the next pull.
    /// Default impl returns NotSupported.
    fn cursor_for(&self) -> Result<Cursor>;

    /// Mark the local store as having applied through
    /// the given cursor. Called after a successful
    /// snapshot apply.
    /// Default impl returns NotSupported.
    fn advance_cursor(&self, cursor: Cursor) -> Result<()>;
}
```

**Default implementations return `NotSupported`.** A
storage adapter that does not implement these methods
(such as the in-memory testkit) compiles cleanly; the
sync engine detects `NotSupported` at runtime and
  reports it. Only the four shipped storage adapters
  override these methods: `educore-storage-surrealdb`
  (Phase 0 primary; per [`ADR-017`](./ADR-017-SurrealDBFirst.md))
  plus the Phase 1 trio `educore-storage-postgres`,
  `educore-storage-mysql`, `educore-storage-sqlite`.

The methods **do not** bypass the command path:
`apply_snapshot` runs each event through the domain
command layer for validation. A snapshot from an
untrusted source is rejected by the same domain rules
as a live event.

**Signature reconciliation note (2026-06-24):** the
signatures sketched in section 5 above
(`watch_changes() -> Result<ChangeStream>` with no
parameters; `cursor_for() -> Result<Cursor>` and
`advance_cursor(cursor)` likewise parameter-less)
disagree with the more detailed sketches in
[ADR-017 § "Four new methods on `StorageAdapter`"](./ADR-017-SurrealDBFirst.md#four-new-methods-on-storageadapter)
(which carry explicit `school_id` and `since` / `stream`
parameters). The two ADRs were drafted in parallel and
neither was canonicalised against the implementation.
The **authoritative signatures** live in
[`crates/infra/storage/src/port.rs`](../../crates/infra/storage/src/port.rs):
`watch_changes(filter: ChangeFilter)` (carries `school_id`
inside the filter struct), `apply_snapshot(snapshot:
SchoolSnapshot)`, `cursor_for(school_id: SchoolId)`,
`advance_cursor(school_id: SchoolId, to: VersionCursor)`.
Both ADR sketches must be treated as illustrative; consult
`port.rs` before implementing a new storage adapter. See
FINDING 28 in `docs/audit_reports/findings/wave5-docs-2.md`.

### 6. Conflict model: server-authoritative

The conflict model is **server-authoritative**, per
[ADR-008 § 6](./ADR-008-OfflineFirst.md):

- A command whose `version` does not match the
  server's current `version` is a conflict.
- The engine surfaces the conflict to the user as
  `DomainError::Conflict::StaleVersion`. The user
  decides whether to retry, merge, or discard.
- For **field-level** conflicts (e.g. two devices
  edited different fields of the same record), the
  default is **last-writer-wins per field**, with the
  losing field logged in the audit log.
- **No CRDTs. No silent auto-merge.** The user is
  always in the loop on a conflict.
- The audit log records every conflict and every
  resolution, so the operator can review.

The conflict policy is configurable per domain via a
`ConflictResolver` trait (already specified in
[ADR-008 § "Mitigations"](./ADR-008-OfflineFirst.md#mitigations)).

### 7. Multi-tenant safety on device

A device is **single-school per device**. The on-device
SQLite database is keyed by a `school_id`; the
encryption key is derived from the device's
school-specific credentials.

- The **server** enforces the `school_id` filter at
  snapshot time. A client never receives data from a
  school it is not authorized for.
- The **local** store only contains data the actor is
  authorized for. A school admin's device has the
  school's data; a parent's device has only the
  parent's children's data; a teacher's device has
  only the teacher's classes.
- **Multi-school admins** (a user authorized across
  multiple schools) use the **SaaS backend** (per
  [`docs/guides/saas-backend.md` § "Multi-Tenancy"](../guides/saas-backend.md#multi-tenancy),
  lines 50-80: the `TenantId → SchoolId` mapping is
  resolved at the central server). The local device
  is single-school; the SaaS backend is the
  multi-school surface.

Encryption on the device is the consumer's
responsibility (e.g. SQLCipher for SQLite). The engine
does not ship encryption; the local store is opaque
to the engine.

## Rationale

- **Primitives stay in the engine, transport stays
  out.** The engine has historically been very strict
  about not shipping transports (see
  [ADR-008 § 8](./ADR-008-OfflineFirst.md#decision),
  which says "The engine does not ship a sync
  protocol"). This ADR respects that boundary: the
  engine ships the metadata and the guarantees; the
  consumer picks the transport.

- **Two adapters, one protocol.** Shipping an
  in-process reference implementation and an HTTP
  adapter against the same protocol means consumers
  can start with the in-process adapter (zero
  infrastructure) and swap to the HTTP adapter when
  they need scale — without rewriting their app.
  The wire protocol is the seam.

- **Feature-flag the sync surface.** Sync is a
  non-trivial dependency (HTTP client, retry policy,
  snapshot (de)serialization). Consumers who do not
  need it (e.g. a server-only deployment) should not
  pay for it. The `sync` feature keeps the default
  build lean.

- **The existing worker design is not replaced.** The
  `docs/guides/saas-backend.md` worker design is
  correct for production scale. The in-process
  adapter is an **additional** option, not a
  replacement. The HTTP adapter is what the worker
  process uses; the worker itself is a consumer
  concern, not engine code.

- **Storage adapter stays small.** Four new methods
  with default `NotSupported` impls. The three
  shipped storage adapters override them. Consumers
  writing a new storage adapter for an unsupported
  backend (e.g. a non-shipped `educore-storage-cockroachdb`)
  can opt out of sync by simply not overriding the
  four methods.

- **Multi-tenant safety is enforced at the server,
  not the device.** The device is a thin client. It
  only has data it is authorized for. A compromised
  device cannot leak data from other schools
  because that data is not on the device.

## Consequences

### Positive

- (+) A consumer can ship a working offline-first
  app in ~30 minutes by adding
  `features = ["sync"]` and using
  `EducoreSyncAdapter::in_process(local, remote)`.
- (+) The wire protocol is documented at
  [`docs/ports/sync.md`](../ports/sync.md) and is
  stable across in-process and HTTP adapters.
- (+) The existing `docs/guides/saas-backend.md`
  worker design is preserved; the HTTP adapter is
  what the worker uses.
- (+) Storage port additions are minimal (4 methods,
  default `NotSupported`); existing storage adapters
  are unchanged at the public API surface.
- (+) Conflict policy is configurable per domain.
- (+) Multi-tenant safety is structural: the local
  device is single-school, the SaaS backend is
  multi-school.

### Negative

- (-) Four new methods on `StorageAdapter`. Even
  with default `NotSupported` impls, the trait
  surface grows. Mitigated by the default impl:
  most consumers do not need to know the methods
  exist.
- (-) New crates: `crates/cross-cutting/sync/`,
  `crates/cross-cutting/sync-inprocess/`,
  `crates/adapters/sync-http/`,
  `crates/adapters/sync-null/`. Four new packages
  in the workspace.
- (-) The `educore-sync-http` adapter is a thin
  HTTP client; the consumer's worker process is
  still consumer code. The engine does not ship
  the worker binary.
- (-) The wire protocol is a new public contract.
  Changes to `docs/ports/sync.md` are breaking
  changes for the HTTP adapter.
- (-) On-device encryption is consumer
  responsibility. The engine does not provide it
  out of the box.

### Mitigations

- The four new `StorageAdapter` methods are
  documented as "default `NotSupported`; storage
  adapters override". The trait's
  `#[non_exhaustive]` attribute (already present)
  prevents downstream consumers from breaking when
  new methods are added.
- The wire protocol is versioned. A v1 / v2 field
  in the envelope lets the HTTP adapter negotiate
  with a server on a different version.
- The `educore-sync` package re-exports a
  `ConflictPolicy` trait that consumers implement
  per domain; the default is field-level LWW.
- The existing `docs/guides/offline-sync.md` is
  updated to point at the in-process + worker pair,
  so the consumer-facing guide stays the source of
  truth for "how do I do offline-first on Educore".

## Alternatives Considered

### 1. Ship only the worker design

Document the worker design (as `saas-backend.md`
already does) and let consumers implement it
themselves. Rejected because the implementation
effort is significant (~200-400 lines of
boilerplate per consumer) and the wire protocol
would have to be rediscovered by each consumer.

### 2. Ship only the in-process adapter

Skip the HTTP adapter; let consumers write their
own transport. Rejected because the wire protocol
needs a reference implementation, and the
production-scale deployment (the worker process)
needs an HTTP client.

### 3. Make sync a domain crate, not a cross-cutting crate

`crates/domains/sync/`. Rejected because sync is
not a domain — it is a port concern that sits
above all domains. The sync engine moves events
**between** domains; it does not own a domain.

### 4. Embed the sync engine in the engine core

`educore-core` includes the sync engine. Rejected
because it couples every consumer (including
server-only deployments) to the sync code path.
The feature flag is the right boundary.

### 5. Use CRDTs for conflict resolution

A la
[Yjs](https://github.com/yjs/yjs) /
[Automerge](https://automerge.org/). Rejected per
[ADR-008 § "Alternatives Considered" § 3](./ADR-008-OfflineFirst.md#3-crdts).
CRDTs are powerful for collaborative editing but
less natural for the school domain, and the
version + etag model covers the common case.
CRDTs are available as a consumer-supplied
`ConflictResolver` for domains that need them.

### 6. Make the local store multi-school

Allow a single device to hold data for multiple
schools. Rejected because the
authentication/authorization surface on a
multi-school device is significantly more
complex (which school's session is active? which
school's encryption key?), and the use case
(multi-school admins) is well served by the SaaS
backend.

## See also

- [ADR-008: Offline-Capable Design](./ADR-008-OfflineFirst.md) —
  the engine's offline model (events in, events out,
  with idempotency and version-based conflict
  resolution). This ADR builds on the model.
- [ADR-014: Idempotency](./ADR-014-Idempotency.md) —
  the command envelope's `idempotency_key`; the
  sync engine relies on it for safe retries.
- [ADR-013: Crate Layout](./ADR-013-CrateLayout.md) —
  the new crates (`educore-sync`,
  `educore-sync-inprocess`, `educore-sync-http`,
  `educore-sync-null`) follow the existing tier
  system. `educore-sync` is cross-cutting; the
  three adapters are adapters.
- [`docs/ports/sync.md`](../ports/sync.md) —
  the wire protocol spec (to be written as part of
  this ADR's implementation).
- [`docs/ports/storage.md`](../ports/storage.md) —
  the four new `StorageAdapter` methods
  (`watch_changes`, `apply_snapshot`, `cursor_for`,
  `advance_cursor`) are documented in the storage
  port spec.
- [`docs/specs/sync/overview.md`](../specs/sync/overview.md) —
  the consumer-facing spec (to be written as part
  of this ADR's implementation).
- [`docs/guides/saas-backend.md` § "The Sync Engine"](../guides/saas-backend.md#the-sync-engine-offline--central) —
  the existing worker-process design that this ADR
  extends. The HTTP adapter is what the worker
  process uses.
- [`docs/guides/offline-sync.md`](../guides/offline-sync.md) —
  the consumer-facing offline-first guide; updated
  to point at the in-process + worker pair.
