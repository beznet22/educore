# ADR-005: Event-Driven Design

## Status

Accepted.

## Context

A school is not a single aggregate. It is a web of them.
Admitting a student in the academic domain implies work in
finance (assign fees), library (issue membership), communication
(send welcome message), and possibly transport (set up route).
The work in the secondary domains should not block the primary
command, and a failure in one secondary domain should not
invalidate the primary success.

A direct call from the academic command to the finance
command would couple the domains. The academic domain would
have to know the finance domain's command catalog, the finance
domain's pre-conditions, and the finance domain's error
modes. A change in either domain would ripple.

The school is also a system of record. An auditor asking
"what happened to student X last Tuesday?" must be able to
reconstruct the answer from durable records, not from
"the database state at the time" (which may have been
overwritten) or from "the live aggregate" (which has since
moved on).

Offline operation is increasingly common. A school in a
rural deployment may submit attendance for a week before
regaining connectivity. The system must reconcile out-of-
order events without losing any.

## Decision

SMSengine adopts **event-driven design** at the domain level.

Concretely:

1. **Every state-changing command emits one or more domain
   events.** The events are typed, versioned, and serialized
   as JSON via the engine's event envelope
   (`schemas/event-schema.md`).
2. **The aggregate is the source of truth for its own state
   and the source of its own events.** The aggregate produces
   events in `handle(command)`; the dispatcher persists
   events and state in a single transaction.
3. **The event bus is a port.** Default implementations are
   in-process, NATS, and Redis. Consumers choose.
4. **The outbox pattern is mandatory for at-least-once
   delivery.** The engine writes events to an outbox table
   in the same database transaction as the aggregate
   mutation; a relay process publishes to the bus.
5. **Subscribers consume events by type and tenant.** They
   are responsible for their own idempotency on `event_id`.
6. **Cross-domain integration is by event, never by direct
   call.** The academic domain does not call the finance
   domain's command bus. It emits `StudentAdmitted`; the
   finance domain subscribes and decides what to do.
7. **Events are immutable.** There is no edit or delete.
   Retraction is a new event (`<Type>Retracted`).
8. **Events have a stable schema with versioning.** A
   breaking payload change bumps `event_version`; both old
   and new versions are accepted during a deprecation
   window.
9. **The audit log mirrors every event.** Every event in
   the bus has a corresponding row in the audit log,
   written transactionally with the aggregate.
10. **Offline sync uses the event log as the source.** A
    consumer's local store can be reconstructed by applying
    events in order; conflicts are resolved on `version` and
    `etag`.

## Consequences

### Positive

- **Decoupled bounded contexts.** Academic does not know
  Finance. The contract between them is the event schema,
  which is versioned and stable.
- **Asynchronous fanout.** A single `StudentAdmitted` event
  reaches all subscribers in parallel; a slow subscriber
  does not block the primary command.
- **Reconstructable history.** Every event is durable. The
  audit log is the timeline; the aggregate's current state
  is a projection.
- **Cross-deployment integration.** The same event that
  reaches a local subscriber can reach a remote analytics
  service, an external BI tool, or a regulator's data lake.
- **Offline-first is natural.** A device records events
  locally; when it reconnects, it ships the events; the
  server applies them in order, idempotently.
- **AI agents can subscribe.** An agent that watches the
  event stream is a first-class consumer.

### Negative

- **Eventual consistency is the rule.** A `StudentAdmitted`
  event reaches finance in milliseconds, not synchronously.
  Consumers that need synchronous cross-domain coordination
  must use **command composition** (a parent command that
  calls child commands in the same transaction) instead of
  events.
- **Schema evolution is forever.** Once an event is on the
  bus, consumers may be reading it. Removing a field
  requires a deprecation window of "publish both versions
  for N months."
- **Event log grows forever (without compaction).** Storage
  cost and retention policy become operational concerns.
- **Debugging is harder.** "Why is the invoice total wrong?"
  requires reading the event log, not just the current row.
  The audit log mitigates this.
- **Idempotency on the consumer is mandatory.** Without it,
  a redelivery produces double-fees. The engine's event
  envelope carries `event_id` precisely for this.

### Mitigations

- The engine ships with a **schema registry** that
  documents every event type, every version, and the
  deprecation timeline. See `event-schema.md` § 7.
- The **outbox + relay** pattern is provided as a
  port-driven default. Consumers do not have to implement
  it from scratch.
- **Event sourcing** is supported but not required.
  Aggregates are the source of truth for state; events
  are the source of truth for history. A consumer can
  project a read model from events but does not have to.
- **Compaction and archival** are port-driven. A consumer
  with strict storage limits can compact old events to a
  snapshot and store the snapshot, with the full log
  available on demand for compliance.

## Alternatives Considered

### 1. Direct service-to-service calls

A `StudentAdmitted` event in academic calls
`finance.assign_fees(...)` synchronously. Rejected because
it couples the domains and creates distributed-transaction
problems. (A student can be admitted even if finance is
down; finance catches up via the event.)

### 2. Shared database with cross-table triggers

Reactive logic in triggers. Rejected because it ties
domain logic to a specific database, hides business
rules in DDL, and is impossible to test in isolation.

### 3. Two-phase commit across domains

XA or similar distributed transactions. Rejected because
the failure modes are catastrophic and the operational
overhead is high. Events with idempotency are good
enough.

### 4. Polling

Subscribers poll the source for changes. Rejected because
it is wasteful, slow, and produces eventually-stale
projections.

### 5. No events; only state

The aggregate is the only source of truth; there are no
events. Rejected because it loses history, blocks
cross-domain integration, and makes offline sync
impossible.
