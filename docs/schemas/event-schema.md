# Event Schema

This document is **normative**. Every domain event published by SMSengine
MUST conform to the envelope defined here. The envelope is the contract
between producers (domain aggregates), the event bus, and consumers
(other domains, projections, AI agents, audit trails, offline
reconcilers).

## 1. The Envelope

A domain event, as it leaves the engine, is wrapped in a single envelope:

```text
EventEnvelope<E> {
    event_id:          EventId,           // UUIDv7, time-ordered, globally unique
    event_type:        &'static str,      // "academic.student.admitted" form
    event_version:     u32,               // schema version of the payload
    school_id:         SchoolId,          // tenant anchor
    aggregate_id:      Uuid,              // root identifier of the mutated aggregate
    aggregate_type:    &'static str,      // "student", "fees_invoice", "payroll"
    actor_id:          UserId,            // the user (or SYSTEM) that triggered the change
    correlation_id:    CorrelationId,     // the request / workflow that initiated it
    causation_id:      Option<EventId>,   // the event that directly caused this one
    occurred_at:       Timestamp,         // clock time of the event
    recorded_at:       Timestamp,         // clock time of persistence (>= occurred_at)
    payload:           E,                 // the typed payload
    metadata:          EventMetadata,     // open-ended, versioned, see § 6
}
```

### 1.1 Field Semantics

- **`event_id`** — UUIDv7. The id is the canonical primary key of the
  event in the log. Two events MUST NOT share an `event_id`.
- **`event_type`** — A stable dotted string of the form
  `<domain>.<aggregate>.<verb>`. Past tense for the verb. Example:
  `academic.student.admitted`, `finance.payment.collected`.
- **`event_version`** — A monotonically increasing integer. Starts at
  `1` for the initial schema. A payload-breaking change MUST bump this
  number; see § 4.
- **`school_id`** — The tenant the event belongs to. Mandatory.
- **`aggregate_id`** — The id of the aggregate root the event describes.
- **`aggregate_type`** — The string name of the aggregate root type, e.g.
  `student`, `fees_invoice`. Stable across deployments.
- **`actor_id`** — The user (or `SYSTEM`) that triggered the change. For
  automated jobs, this is the job's service user; for system events, the
  engine's `SYSTEM_USER_ID`.
- **`correlation_id`** — Set by the inbound command. Propagates through
  every event in the same request / workflow.
- **`causation_id`** — For events produced by a reaction to another
  event (e.g. `FinanceFeesAssigned` produced in response to
  `AcademicStudentAdmitted`), the id of the causing event.
- **`occurred_at`** — Set by the engine's `Clock` port at the moment
  the aggregate records the event.
- **`recorded_at`** — Set at persistence time. The clock MAY have
  advanced; the delta is small and reflects real persistence latency.
- **`payload`** — A typed, `serde`-serializable Rust struct. The shape
  of the payload is owned by the producing domain.
- **`metadata`** — Open-ended map. See § 6.

## 2. Event Type Naming

```text
<domain>.<aggregate>.<verb_past_tense>
```

Rules:

- Domain is one of the engine's bounded contexts (`academic`, `finance`,
  `hr`, `attendance`, `assessment`, `library`, `facilities`,
  `communication`, `events`, `documents`, `cms`, `platform`, `rbac`,
  `settings`, `operations`).
- Aggregate is the singular noun of the root (`student`,
  `fees_invoice`, `payroll`).
- Verb is past tense, snake_case (`admitted`, `payment_collected`,
  `result_published`).

Examples:

- `academic.student.admitted`
- `academic.student.promoted`
- `academic.student.withdrawn`
- `finance.invoice.generated`
- `finance.payment.collected`
- `finance.bank.amount_transferred`
- `finance.payroll.paid`
- `assessment.result.published`
- `attendance.session.taken`
- `hr.staff.leave_approved`
- `library.book.issued`
- `rbac.capability.assigned`
- `platform.school.onboarded`
- `platform.school.suspended`
- `operations.backup.restored`
- `communication.notice.sent`

## 3. JSON Serialization

When an event leaves the engine (over an event bus, through an API, into
the audit log, or onto offline storage), it is serialized as a single
JSON object:

```json
{
  "event_id": "01939d7a-1b2c-7def-9012-3456789abcde",
  "event_type": "academic.student.admitted",
  "event_version": 1,
  "school_id": "f0e1d2c3-b4a5-4687-8901-23456789abcd",
  "aggregate_id": "12345678-90ab-cdef-1234-567890abcdef",
  "aggregate_type": "student",
  "actor_id": "abcdef01-2345-6789-abcd-ef0123456789",
  "correlation_id": "cccccccc-1111-2222-3333-444444444444",
  "causation_id": null,
  "occurred_at": "2026-06-08T09:30:00.123456Z",
  "recorded_at": "2026-06-08T09:30:00.156Z",
  "payload": {
    "student_id": "12345678-90ab-cdef-1234-567890abcdef",
    "admission_no": "ADM-2026-001234",
    "full_name": "Anita Sharma",
    "class_id": "...",
    "section_id": "...",
    "academic_year_id": "...",
    "guardian_ids": ["..."],
    "admission_date": "2026-06-08"
  },
  "metadata": {
    "source": "web",
    "user_agent": "Mozilla/5.0 ...",
    "ip": "203.0.113.42"
  }
}
```

### 3.1 Wire Format Rules

- Timestamps are RFC 3339 UTC with microsecond precision, ending in `Z`.
- Identifiers are UUIDv7 strings. Never integers.
- All numeric amounts are stringified as fixed-point decimals in the
  payload (e.g. `"1234.56"`). Avoid binary float ambiguity.
- Field names are `snake_case` and match the Rust struct field names.
- Optional fields are present and `null`, NOT omitted.
- Enum values are lower-snake strings (`"active"`, `"withdrawn"`).
- Lists are JSON arrays. Maps are JSON objects.
- Empty collections are `[]` or `{}`, not `null`.

## 4. Event Versioning

### 4.1 When to Bump

`event_version` MUST be bumped when a payload change is **not backward
compatible**:

- Removing a field.
- Changing a field's type.
- Renaming a field.
- Reinterpreting a field's meaning.

Backward-compatible changes do NOT bump `event_version`:

- Adding a new optional field.
- Adding a new enum variant in a way consumers handle with a default.

### 4.2 Multi-Version Publication

When a payload shape changes, the engine MUST publish **both** the old
and the new version for a deprecation window, until the consumer adapter
is updated. Consumers MUST be able to opt into the new version explicitly.

The schema registry (§ 7) records:

- All known versions of an event type.
- The deprecation date of older versions.
- The migration path from each version to the current.

### 4.3 Forward and Backward Compatibility

- A consumer reading the current schema MUST accept a payload of the
  same type at any older version. New optional fields are tolerated.
- A consumer reading an older schema MUST accept a payload of the
  current version by ignoring unknown fields. Producers MAY add new
  optional fields without bumping the version.

## 5. Causation and Correlation

- **`correlation_id`** is generated at the boundary when a command
  enters the engine. It propagates through every event the command
  produces, including all cross-domain reactions. It is the join key
  for "show me everything that happened because of this request."
- **`causation_id`** is the `event_id` of the event that directly
  produced this event. For the first event in a request, it is `None`.
  For a reaction event, it is the `event_id` of the triggering event.
- A consumer resolving a reaction chain follows `causation_id` links
  backwards.

## 6. Metadata

The `metadata` field is a JSON object with the following recommended
keys, all optional:

```text
{
    "source": "web" | "mobile" | "api" | "agent" | "import" | "system",
    "user_agent": "<string>",
    "ip": "<string>",
    "request_id": "<uuid>",          // the inbound HTTP/RPC request id
    "device_id": "<string>",
    "session_id": "<uuid>",
    "geo": { "country": "...", "region": "..." },
    "feature_flags": { ... },
    "trace": [ "<span id>", ... ]    // for distributed tracing
}
```

Metadata MUST NOT contain PII that is not also in the payload. It MUST
NOT contain credentials, tokens, or secrets.

## 7. Schema Registry

The engine maintains a schema registry. The registry:

- Records the canonical type name of every event (`event_type`).
- Records every published `event_version` and its JSON schema.
- Tracks deprecation dates and migration guides.
- Exposes a discovery endpoint: `engine.events.list()` returns
  `(event_type, current_version, deprecated_versions)`.
- Exposes a per-type schema fetch:
  `engine.events.schema(event_type, version)`.

The schema registry is a port. Default implementation is in-process. A
consumer MAY provide a Confluent / Apicurio / Protobuf-style registry
adapter.

## 8. Outbox Pattern

For at-least-once delivery across process restarts, the engine writes
events to an **outbox table** in the same database transaction that
mutates the aggregate. A relay process polls the outbox and publishes
to the event bus, then marks rows as published.

The outbox is a port-driven concern. The engine guarantees:

- A successful command commits both the aggregate state change and the
  outbox row in a single transaction.
- The outbox row's `event_id` matches the envelope's `event_id`.
- The relay is idempotent on `event_id` (a second publish is a no-op).
- Consumers MUST be idempotent on `event_id`.

The outbox table is documented at the storage level; the engine does
not own its schema beyond the column names:

```text
outbox(
    event_id        EventId       PK,
    event_type      VARCHAR,
    event_version   INT,
    school_id       SchoolId,
    payload         JSON,
    enqueued_at     TIMESTAMP,
    published_at    TIMESTAMP     NULL,
    attempts        INT           DEFAULT 0,
    last_error      TEXT          NULL
)
```

## 9. Consumer Acknowledgement and Replay

- Consumers acknowledge events by `event_id`. An unacknowledged event
  is replayable.
- Replay is supported: a consumer can request "all events of type X
  since event_id Y" or "all events of aggregate Z up to time T."
- The engine retains events for the consumer-configured retention
  period. After that, events are compacted to a projection; raw events
  are still available in cold storage for compliance.

## 10. Subscription Model

The engine supports:

- **Topic-based subscription** — `subscribe("finance.invoice.*")`
  matches by dotted-prefix globs.
- **Aggregate subscription** — `subscribe_aggregate("fees_invoice",
  invoice_id)`.
- **Tenant subscription** — `subscribe_school(school_id)`.
- **Type-exact subscription** — `subscribe("academic.student.admitted")`.

Each subscription produces a stream of `EventEnvelope<E>` in
occurred-at order. Out-of-order delivery across domains is the
consumer's responsibility.

## 11. Delivery Semantics

- The engine guarantees **at-least-once** delivery to the bus.
- Consumers MUST be idempotent on `event_id`.
- Exactly-once is not promised and is the consumer's responsibility.

## 12. Event Immutability

Events are immutable. The engine MUST NOT support "edit event" or
"delete event" operations on the log. A retraction is itself a new
event (`<type>Retracted`).

## 13. Compliance Hooks

- Every event is mirrored to the audit log with the same `event_id`.
- The audit log is write-once.
- Personal data within payloads is subject to the data-retention and
  erasure rules in `audit-schema.md`.
