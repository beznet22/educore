# Event Replay Guide

## Goal

Reconstruct state from events. New projections, debugging, and audit
reconstruction all use replay.

## Why Replay

- **New projection**: a new aggregate view (e.g. "average marks by
  class over time") is built by replaying all relevant events.
- **Bug recovery**: if a projection diverges from the source of
  truth, replay rebuilds it.
- **Audit**: reconstruct what happened in a time window.
- **Migration**: when an aggregate's shape changes, replay with a
  transformer rebuilds it in the new shape.

## Replay Strategies

### Full Replay

Read all events from the start of time and apply them in order.

```rust
async fn rebuild_projection<E: DomainEvent>(
    store: &dyn EventStore,
    projection: &mut Projection,
) -> Result<()> {
    let mut stream = store.read_all().await?;
    while let Some(envelope) = stream.next().await {
        let event = envelope.payload;
        projection.apply(event).await?;
    }
    Ok(())
}
```

Full replay is slow for large event logs. Use it for one-off
operations (e.g. initial population of a new projection).

### Incremental Replay

Read events since the last projection update.

```rust
let last_offset = projection.last_event_id().await?;
let stream = store.read_from(last_offset).await?;
while let Some(envelope) = stream.next().await {
    projection.apply(envelope.payload).await?;
    projection.record_offset(envelope.event_id).await?;
}
```

Incremental replay is fast. Use it for ongoing projection
maintenance.

### Snapshot + Tail

Periodically snapshot a projection. Replay from the snapshot,
then apply the tail.

```rust
let snapshot = store.read_snapshot(projection_id, since).await?;
projection.restore_from(snapshot).await?;
let stream = store.read_from(snapshot.last_event_id).await?;
// ... apply tail
```

Snapshots reduce replay time for large event logs.

## Event Store

The event store is the storage adapter's outbox plus a log:

```sql
CREATE TABLE event_log (
    event_id UUID PRIMARY KEY,
    event_type TEXT NOT NULL,
    aggregate_id UUID NOT NULL,
    school_id INT NOT NULL,
    payload JSONB NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX ix_event_log_school_occurred ON event_log (school_id, occurred_at);
CREATE INDEX ix_event_log_aggregate ON event_log (aggregate_id, recorded_at);
```

The engine writes events to the outbox (in the same transaction as
the state change) and a separate process copies them to the
event_log for replay.

## Projections

A projection is a derived view built from events. Examples:

- `StudentActiveRoster`: maintained by `StudentAdmitted`,
  `StudentWithdrawn`, `StudentSuspended`, `StudentReinstated`.
- `ClassRollCount`: maintained by `StudentAssignedToSection`,
  `StudentWithdrawn`, `StudentPromoted`.
- `OutstandingFees`: maintained by `FeesInvoiceGenerated`,
  `PaymentReceived`, `FeesCarriedForward`.
- `AttendanceRate`: maintained by `StudentAttendanceMarked`.

Each projection has:

- An apply function per event type.
- A query function for reads.
- An offset tracker (for incremental replay).
- An optional snapshot (for snapshot+tail).

## Worked Example: Building a New Projection

Suppose we need a projection that tracks the count of admitted
students per day:

```rust
pub struct DailyAdmissionCount {
    counts: BTreeMap<NaiveDate, u64>,
    last_event_id: Option<EventId>,
}

impl DailyAdmissionCount {
    pub fn apply(&mut self, event: &EventEnvelope) -> Result<()> {
        match event.event_type {
            "StudentAdmitted" => {
                let payload: StudentAdmitted = serde_json::from_value(event.payload.clone())?;
                *self.counts.entry(payload.admission_date).or_default() += 1;
            }
            _ => {}
        }
        self.last_event_id = Some(event.event_id);
        Ok(())
    }
}

async fn build_projection(store: &dyn EventStore) -> Result<DailyAdmissionCount> {
    let mut proj = DailyAdmissionCount::default();
    let mut stream = store.read_all().await?;
    while let Some(env) = stream.next().await {
        proj.apply(&env?)?;
    }
    Ok(proj)
}
```

## Versioned Events

When an event's shape changes, the projection's apply function
must handle both old and new versions:

```rust
match (event.event_type, event.schema_version) {
    ("StudentAdmitted", 1) => { /* old shape */ }
    ("StudentAdmitted", 2) => { /* new shape */ }
    _ => {}
}
```

The engine documents schema versions in
`docs/schemas/event-schema.md`.

## Tombstones

Events are never deleted. To "remove" a projection row, the
projection applies a `Deleted` event. The engine may emit
`AggregateDeleted` events for explicit deletions.

## Idempotency in Replay

Replays are idempotent. The projection's apply function must be
deterministic given the same event. If the function is non-deterministic
(e.g. uses the current time), the projection will diverge on replay.

Use a frozen clock in tests. In production, the engine stamps events
with `occurred_at`, which the projection uses instead of `Instant::now()`.

## Snapshot Storage

```sql
CREATE TABLE projection_snapshots (
    projection_id TEXT NOT NULL,
    school_id INT NOT NULL,
    last_event_id UUID NOT NULL,
    snapshot JSONB NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (projection_id, school_id)
);
```

Snapshots are taken periodically (e.g. every 1000 events or every
hour, whichever comes first).

## Testing

- A test that replaying the same event log produces the same
  projection state.
- A test of incremental replay from an offset.
- A test of snapshot+tail correctness.
- A test of versioned event handling.
- A test of large-scale replay performance (e.g. 1M events in
  <10s).

## Anti-Patterns

- ❌ Replaying into the same projection that emitted the events
  (infinite loop).
- ❌ Non-deterministic apply functions.
- ❌ Replaying across tenants without filtering.
- ❌ Replaying in parallel without ordering guarantees.
- ❌ Mutating event payloads during replay.
