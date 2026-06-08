# Command Schema

This document is **normative**. It defines the envelope and lifecycle
of every command the engine accepts. Commands are the **only**
sanctioned way to mutate engine state.

## 1. Command Envelope

Every command carries the following envelope:

```text
Command<C> {
    tenant:           TenantContext,    // school, actor, correlation, causation
    idempotency_key:  IdempotencyKey,   // optional but recommended
    command_type:     &'static str,     // "academic.student.admit"
    command_version:  u32,              // 1+
    issued_at:        Timestamp,        // the moment the caller created it
    deadline:         Option<Timestamp>,// optional SLA; the engine may abort
    payload:          C,                // the typed command body
    metadata:         CommandMetadata,  // open-ended, see § 9
}
```

### 1.1 `TenantContext`

```text
TenantContext {
    school_id:     SchoolId,        // the active school
    actor_id:      UserId,          // the active user (or SYSTEM)
    session_id:    Option<SessionId>,
    correlation_id: CorrelationId,  // propagated to every event
    causation_id:  Option<EventId>, // for chained commands
    user_type:     UserType,        // student, parent, teacher, etc.
    locale:        Locale,          // presentation locale
    timezone:      TimeZone,        // presentation timezone
}
```

The `TenantContext` is created at the engine boundary. It is
immutable for the lifetime of a single command.

### 1.2 `IdempotencyKey`

A caller-provided UUIDv7 identifying the intent. Two commands with the
same `idempotency_key` against the same `school_id` and the same
`command_type` MUST produce the same result and emit no additional
events on replay. See `ADR-014-Idempotency.md` and § 6 below.

## 2. Command Naming

```text
<Domain>.<Aggregate>.<Verb>
```

Imperative tense. Examples:

- `Academic.Student.Admit`
- `Academic.Student.Withdraw`
- `Academic.Student.Promote`
- `Assessment.Exam.PublishResult`
- `Assessment.Mark.Record`
- `Attendance.Session.Mark`
- `Finance.Invoice.Generate`
- `Finance.Payment.Collect`
- `Finance.Payroll.Generate`
- `Finance.Payroll.Approve`
- `Finance.Payroll.Pay`
- `Hr.Leave.Apply`
- `Hr.Leave.Approve`
- `Library.Book.Issue`
- `Library.Book.Return`
- `Platform.User.Create`
- `Platform.School.Onboard`
- `Rbac.Role.Assign`

## 3. Validation

A command is validated in three phases:

1. **Structural validation** — The struct shape: required fields
   present, types correct, value objects constructed. Failure here
   is a `Validation` error.
2. **Reference validation** — All referenced ids (`SchoolId`,
   `StudentId`, `ExamId`, etc.) exist and belong to the same
   `school_id` as the tenant. Failure here is `NotFound` or
   `TenantViolation`.
3. **Business pre-conditions** — The state of the target aggregate
   allows the command. Failure here is a domain-specific error
   (`StudentAlreadyAdmitted`, `InvoiceAlreadyClosed`, etc.).

The engine returns a single `Result<CommandOutcome, DomainError>`. The
`DomainError` enum carries a `kind` discriminant and a structured
`details` payload.

## 4. Authorization

After validation, the engine resolves the capability required for the
command. A command is annotated with one or more capabilities:

```rust
pub struct AdmitStudentCommand { /* ... */ }
impl Command for AdmitStudentCommand {
    const TYPE: &'static str = "academic.student.admit";
    const REQUIRED_CAPABILITIES: &'static [Capability] =
        &[Capability::StudentAdmit];
}
```

A multi-capability command requires the actor to hold **all** listed
capabilities. The check is `has_all` unless the type lists a single
capability.

## 5. Dispatch and Lifecycle

```text
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│ Validate     │───▶│ Authorize    │───▶│ Pre-cond.    │
└──────────────┘    └──────────────┘    └──────────────┘
                                              │
                                              ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│ Audit        │◀───│ Persist      │◀───│ Mutate       │
└──────────────┘    └──────────────┘    └──────────────┘
                                              │
                                              ▼
                                       ┌──────────────┐
                                       │ Publish      │
                                       └──────────────┘
```

The full sequence is:

1. The dispatcher receives the command envelope.
2. **Authenticate** the actor via the `AuthProvider` port.
3. **Build** the `TenantContext`.
4. **Validate** structurally.
5. **Check** capabilities via `CapabilityCheckService`.
6. **Resolve** the target aggregate via its repository.
7. **Pre-conditions** — domain rules on the loaded state.
8. **Mutate** the aggregate. The aggregate is responsible for emitting
   events.
9. **Persist** the new state plus all emitted events, transactionally.
10. **Publish** events to the event bus.
11. **Audit** — write the audit record.
12. **Return** the command outcome (new state summary + emitted
    events).

## 6. Idempotency

A command with an `idempotency_key` is replayable. On replay:

- The engine looks up the previous outcome by `(school_id,
  idempotency_key, command_type)`.
- If a previous outcome exists, the engine returns it **without**
  re-executing. The actor sees the same result and the same events
  (referenced by id) as the original.
- If no previous outcome exists, the command executes normally and
  the outcome is stored.

The idempotency store is a port. Default implementation is a database
table with a unique index on `(school_id, command_type,
idempotency_key)`. Storage adapters MUST provide this table.

Idempotency windows: the engine retains idempotency records for the
duration the consumer configures (default 7 days). After expiry, a
replay of the same key is treated as a new command.

## 7. Correlation and Causation

- `correlation_id` is generated at the engine boundary. It is
  propagated to every event emitted by the command, and to every
  chained command.
- `causation_id` is set to the `event_id` of the most recent event
  that caused this command (e.g. for a `Finance.Payroll.Generate`
  command issued in response to an `Assessment.Exam.Published`
  event, the causation_id is the event id).
- For a top-level command, `causation_id` is `None`.

## 8. Versioning

A command's `command_version` reflects the payload schema. Producers
MUST:

- Bump `command_version` for any non-backward-compatible change to the
  payload.
- Keep at least the previous version accepted for one deprecation
  window.
- Document the migration path in the command's rustdoc.

Consumers receive a 400-class error if they send a `command_version`
the engine does not recognize.

## 9. Command Metadata

The `metadata` field is a JSON object with the following recommended
keys, all optional:

```text
{
    "source": "web" | "mobile" | "api" | "agent" | "import" | "system",
    "user_agent": "<string>",
    "ip": "<string>",
    "request_id": "<uuid>",
    "device_id": "<string>",
    "feature_flags": { ... },
    "trace": [ "<span id>", ... ]
}
```

The engine never logs PII from `payload` or `metadata`. Sensitive
fields (passwords, OTPs, payment credentials) are redacted by the
command type before logging.

## 10. Cancellation

A command MAY carry a `deadline` after which it is rejected. The
engine does not support "cancel in flight" for individual commands;
a long-running batch is a separate command with its own cancellation
token (see `cancellation_token` on bulk commands).

## 11. Replay

A consumer MAY request "replay this command" by re-sending the same
envelope (same `idempotency_key` or, in absence, the same
`command_type`, `school_id`, `actor_id`, and a client-supplied
replay-token).

Replay uses the same idempotency mechanism. The engine guarantees
that the replay produces the same result and the same events.

## 12. Bulk Variants

Bulk commands (e.g. `Attendance.Session.MarkBulk`,
`Finance.Payment.CollectBulk`) follow the same envelope but carry a
`Vec<C>` of sub-commands. The engine processes the batch as a single
transaction:

- All-or-nothing: any failure rolls back the entire batch.
- The bulk command emits one `BulkCommandStarted` event, one
  `BulkCommandItemProcessed` event per item, and one
  `BulkCommandCompleted` event with the count of successes and
  failures.
- Partial success is **not** permitted in the bulk envelope. For
  partial success, issue individual commands.

The bulk command has a `concurrency_limit` (default 1, sequential)
and a `failure_policy` (default `FailFast`; alternative is
`CollectErrors` which records per-item errors in a result list
without aborting the batch).

## 13. Sync vs Async

A command has a `mode` of:

- **`Sync`** — the engine returns the outcome in the same call. Most
  commands are sync.
- **`Async`** — the engine returns a `CommandHandle` immediately. The
  caller polls `engine.commands.status(handle)` for completion.

Async is reserved for long-running operations: bulk imports,
payroll generation, report generation, and any command whose expected
duration exceeds the consumer's request budget. Async commands emit
`CommandAccepted`, `CommandStarted`, `CommandProgress` (zero or
more), `CommandCompleted` events.

## 14. Failure Semantics

A command fails with one of:

| Error Kind         | Cause                                                          | HTTP equivalent |
| ------------------ | -------------------------------------------------------------- | --------------- |
| `Validation`       | Bad input shape or value object construction failed.           | 400             |
| `Unauthorized`     | Missing or invalid authentication.                             | 401             |
| `Forbidden`        | Actor lacks the required capability.                           | 403             |
| `NotFound`         | Referenced aggregate does not exist.                           | 404             |
| `Conflict`         | Optimistic concurrency failure, version mismatch, etag stale.  | 409             |
| `Precondition`     | Domain pre-condition not met.                                  | 422             |
| `TenantViolation`  | Aggregate is in a different school than the actor.             | 403             |
| `Idempotency`      | Same key replayed with a different payload.                    | 409             |
| `RateLimit`        | Caller exceeded its quota.                                     | 429             |
| `Timeout`          | Operation did not complete within `deadline`.                  | 504             |
| `Infrastructure`   | Storage, bus, or port failure.                                 | 500             |
| `Internal`         | Bug. Engine invariant violation. Should be unreachable.        | 500             |

Errors are typed. Consumers MUST handle the error variants
explicitly; they MUST NOT depend on display strings.

## 15. Domain Error Types

Each domain defines a `*Error` enum that wraps the engine-level
`DomainError` and adds domain-specific variants:

- `StudentError::AlreadyAdmitted`
- `InvoiceError::AlreadyClosed`
- `PayrollError::AlreadyPaid`
- `BookError::NotAvailable`
- `LeaveError::InsufficientBalance`

The engine-level `DomainError` is the public contract. Domain-
specific enums are converted to it at the boundary.

## 16. Command Outcome

A successful command returns a `CommandOutcome`:

```text
CommandOutcome {
    command_id:       CommandId,        // engine-assigned
    result:           OutcomeResult,    // typed per command
    events:           Vec<EventId>,     // emitted by this command
    aggregate_id:     Uuid,             // the mutated root
    aggregate_version: i64,             // new version after mutation
    etag:             Etag,             // new content hash
    duration_ms:      u64,              // execution duration
}
```

The `result` is the command-specific outcome struct (e.g.
`AdmitStudentResult { student_id, admission_no, ... }`).
