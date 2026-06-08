# ADR-014: Idempotent Commands

## Status

Accepted.

## Context>

A school operation that fails halfway through is a
school operation that has to be redone. The bus driver
who tapped the parent's phone and the network went
down: did the attendance register or not? The
accountant who clicked "Collect Payment" and the
browser hung: was the payment recorded? The teacher
who marked homework for 30 students and the tablet
froze: are 30 submissions sitting in limbo, or zero?

The naive approach — "the user can just retry" — is
not safe. A retry may execute the same command twice.
Two admissions of the same student produce two
records. Two collections of the same fee produce a
double credit. The school's books go wrong in
audible, expensive ways.

The school's operations also run on flaky networks.
A parent app in a low-connectivity area, a teacher's
tablet on a field trip, a bus's mobile data — none
of these can rely on a single request/response
cycle. Retries are a fact of life.

A second naive approach — "make the network reliable"
— is not under the engine's control. The engine
must be safe under an unreliable network.

## Decision

SMScore commands are **idempotent on
`idempotency_key`**. A retry of the same command
with the same `idempotency_key` produces the same
result and emits no additional events.

Concretely:

1. **Every command envelope carries an
   `idempotency_key`** (UUIDv7, caller-generated).
   The key is optional but recommended; the engine
   accepts commands without one but does not
   dedupe them.
2. **The engine stores the command's outcome** keyed
   by `(school_id, command_type, idempotency_key)`.
   The store is a port. Default implementation is a
   database table with a unique index.
3. **A retry with the same key** returns the
   original outcome without re-executing the
   command. The actor sees the same result and the
   same events (referenced by id) as the original.
4. **A retry with the same key but a different
   payload** is rejected with `IdempotencyConflict`
   error. The caller must either send the original
   payload (the safe retry) or generate a new
   `idempotency_key` (a new operation).
5. **The idempotency record is retained for the
   consumer-configured window** (default 7 days).
   After the window, a retry of the same key is
   treated as a new command.
6. **The idempotency record carries the command
   outcome**: the result, the emitted event ids,
   the new aggregate version, the new etag, the
   duration. A retry returns the same outcome
   verbatim.
7. **Idempotency is enforced at the command
   dispatcher**, not in the aggregate. The
   aggregate is unaware of the key; the
   dispatcher decides whether to call the
   aggregate or to return the stored outcome.
8. **Bulk commands are idempotent on the bulk
   key.** The bulk command's `idempotency_key`
   dedups the whole batch. There is no per-item
   idempotency for bulk; a partial replay of a
   bulk command must use a new key and
   re-execute the whole batch (with the failure
   policy deciding what to do with already-
   applied items).
9. **The async command handle carries the
   `idempotency_key`.** A poll for status is
   safe to retry.

The idempotency guarantee is part of the engine's
**at-least-once** delivery model. The consumer
network is unreliable; the engine is correct.

## Consequences

### Positive

- **Retries are safe.** A flaky network that drops
  a response does not produce a duplicate
  operation. The parent's payment is recorded
  once, even if the parent's app retries the
  POST.
- **Offline-first is natural.** A device queues
  commands locally with their `idempotency_key`.
  When connectivity returns, the device ships
  the queue. Duplicates (because the same
  command was sent twice from different paths)
  are deduped on the server.
- **AI agents are safe.** An agent runtime that
  retries on transient failures produces no
  duplicates.
- **The engine's contracts are precise.** A
  command is "exactly-once in effect, at-least-
  once in attempt." The audit log records every
  attempt; the domain state records the single
  effect.
- **Bulk operations are also safe.** A bulk
  admit of 30 students retries as a single
  operation; the database sees one transaction
  on success or one rollback on failure.

### Negative

- **Storage cost.** The idempotency record per
  command is one row, retained for the
  configured window. At 50,000 events/day
  with 7-day retention, ~350,000 rows. This is
  a tiny fraction of the school's data; the
  cost is negligible.
- **Latency on the retry path.** A retry
  performs one extra read (the idempotency
  record lookup) before either returning the
  stored outcome or executing. The read is
  indexed and sub-millisecond.
- **Cross-domain chains need careful key
  generation.** A command that triggers a
  cascade of domain reactions (e.g. admit
  student triggers fees assignment) must
  propagate the `idempotency_key` (or
  generate a deterministic child key) so the
  cascade is also dedupable.
- **Async commands store the outcome when
  complete, not when accepted.** A long-running
  async command's idempotency record is
  written only when the command finishes. A
  retry during the run sees no record; the
  engine rejects the retry with
  `IdempotencyPending` and asks the caller to
  wait for completion.

### Mitigations

- The `idempotency_record` table is indexed
  on `(school_id, command_type, idempotency_key)`
  and is partitioned by month in the default
  PostgreSQL adapter.
- The cross-domain cascade is documented per
  command: the child commands use a
  deterministic key derived from the parent's
  `idempotency_key` and the child's slot. A
  retry of the parent produces the same
  children.
- A background job purges idempotency records
  older than the configured window. The
  consumer schedules it.
- The `IdempotencyPending` error is documented
  in `command-schema.md` § 14 with a clear
  remediation: poll for status, do not retry.

## Alternatives Considered

### 1. Optimistic concurrency only

The aggregate's `version` prevents concurrent
writes, but not duplicate writes. A retry of
the same command finds the same `version`,
loads the aggregate, applies the same change,
and the `version` increments again. The
result is a duplicate effect.

Rejected for the school domain: the cost of
a duplicate admission, a duplicate payment,
or a duplicate payroll is too high.

### 2. Server-generated `idempotency_key`

The engine assigns the key. Rejected because
the caller cannot deduplicate retries across
client restarts. The caller-generated key is
the standard.

### 3. Two-phase commit

Distributed transactions. Rejected for the
same reasons as `ADR-005`: failure modes are
catastrophic.

### 4. CRDT-style merges

Conflict-free replicated data types absorb
duplicates. Powerful for collaboration; for
the school domain, the cost (semantic
complexity) outweighs the benefit. The
`version` + `etag` model is the engine's
discipline for non-idempotent cases.

### 5. Compensating transactions

A retry triggers a compensating action.
Rejected because the school's commands are
not always reversible (a withdrawn student
can be re-admitted, but the audit trail
shows the detour). Idempotency is cleaner.

### 6. No idempotency

"Just be careful with retries." Rejected
because the engine's value proposition is
that consumers do not have to be careful.
The engine handles it.
