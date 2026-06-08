# ADR-008: Offline-Capable Design

## Status

Accepted.

## Context

Many schools do not have reliable internet. A school in a
rural area may have a satellite link that drops several
times a day. A school on a military base may be
air-gapped. A school on a field trip has no connectivity
at all. In all of these cases, the school's staff need
the system to keep working: take attendance, record
marks, collect fees (in cash), issue books, log a
disciplinary note.

The system also needs to **reconcile** when connectivity
returns. The teacher who took attendance offline should
not lose her work. The administrator who collected a fee
in cash should not have to re-enter it. The marks
recorded on the bus ride home should appear in the
server's database when the device syncs.

The naive approach — "wait for connectivity" — is
unacceptable. The school's operations cannot pause
because the network is down.

A second naive approach — "two-way sync via a custom
protocol" — is a research project, not an engine
feature. We need a robust, well-understood model.

## Decision

SMScore is **offline-capable**: aggregates carry the
metadata needed for conflict-free reconciliation, and the
engine's command / event model is naturally suited to
event-log-based sync.

Concretely:

1. **Every aggregate carries `version` and `etag` columns.**
   `version` is a monotonically increasing integer; `etag`
   is a content hash. The pair supports optimistic
   concurrency and conflict detection.
2. **Every domain event carries `event_id`** (UUIDv7).
   Idempotency on the consumer is enforced by
   `event_id` deduplication.
3. **The engine's command envelope carries
   `idempotency_key`.** A client retrying a failed command
   sends the same `idempotency_key`; the engine returns the
   original outcome.
4. **A local store mirrors the server's domain events.**
   The local store is the projection an offline device
   uses. It can be SQLite, IndexedDB, or a custom file
   format — the engine's storage port is implemented
   locally.
5. **Outgoing commands are queued locally** with their
   `idempotency_key`. When connectivity returns, the
   client ships the queue to the server in order.
6. **The server applies the commands in order.** A
   command whose `version` does not match the server's
   current `version` is a conflict. Conflict resolution
   follows the rules in `database-schema.md` § 9.
7. **The engine emits `version` increments and `etag`
   changes for every mutation.** A client can poll for
   changes since a given `version` or `event_id`.
8. **The engine does not ship a sync protocol.** The
   sync protocol is a port-driven consumer concern. The
   engine provides the metadata and the guarantees; the
   consumer implements the transport (HTTP long-poll,
   WebSocket, MQTT, file drop, etc.).

The offline model is "events in, events out, with
idempotency and version-based conflict resolution."
This is the same model used by event-sourced systems
and CRDTs; the engine adopts the discipline without
requiring event sourcing at the aggregate level.

## Consequences

### Positive

- **The school keeps working offline.** A teacher takes
  attendance on a tablet in the bus; it syncs when the
  bus returns to the school.
- **No data loss on retry.** A flaky network that drops
  a command's response does not cause a duplicate
  admission; the `idempotency_key` dedups.
- **Conflict detection is structural.** Every aggregate
  has a `version` and `etag`; the server detects when a
  client is operating on stale data.
- **Sync is a port.** A consumer can use any sync
  transport (HTTP, MQTT, file drop) without changing
  domain code.
- **AI agents can operate offline too.** An agent that
  is invoked from a CLI in a low-connectivity
  environment queues its commands and syncs later.

### Negative

- **Local storage is mandatory on offline devices.**
  The consumer must implement the local store; the
  engine provides the in-memory testkit as a starting
  point.
- **Conflict resolution is policy-dependent.** The
  default is last-writer-wins on `version`, with the
  conflict logged. A consumer needing stronger
  semantics (e.g. CRDT merges for attendance) must
  implement the policy.
- **The local store can drift from the server.** A
  long offline window produces a large delta. The
  consumer's sync worker handles the delta; the
  engine does not constrain the window length.
- **`etag` and `version` are not free.** They add two
  columns per table and two checks per write. The
  cost is small relative to the benefit.

### Mitigations

- The `smscore-storage` port provides a `LocalAdapter`
  trait that consumers implement for offline devices.
  The trait is the same as the server-side
  `StorageAdapter`; the difference is in the
  transport.
- A `ConflictResolver` trait lets consumers plug in
  domain-specific conflict resolution policies
  (e.g. "attendance is the union of all marks").
- The engine's audit log captures every conflict and
  every resolution, so the operator can review.
- The CLI scaffold (`smscore sync`) demonstrates the
  pattern.

## Alternatives Considered

### 1. Online-only

The system requires connectivity. Rejected because the
target deployment scenarios (rural, military, field
trip) are real and the school cannot pause.

### 2. Full event sourcing

The aggregate's state is rebuilt from the event log.
Powerful but expensive; doubles the storage and
operational cost. The engine's aggregates are the
source of truth for state; events are the source of
truth for history. Offline sync uses events; aggregates
are loaded on demand.

### 3. CRDTs

Conflict-free replicated data types. Powerful for
collaborative editing, less natural for the
school domain. The engine's version + etag model
covers the common case; CRDTs are available as an
adapter for domains that need them.

### 4. Master-slave replication

A central server, slave devices that mirror. Rejected
because the school domain has many writers (staff,
students, parents) and the master becomes a single
point of failure.

### 5. Two-phase commit

A protocol for distributed transactions. Rejected
because the failure modes are catastrophic; events
with idempotency are sufficient.
