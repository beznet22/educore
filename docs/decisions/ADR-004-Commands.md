# ADR-004: Command-Oriented Execution

## Status

Accepted.

## Context

There are two ways to model state mutation in a domain:

- **Method-on-aggregate**: external code holds a reference to
  the aggregate and calls `student.withdraw()`, `invoice.pay()`,
  `payroll.approve()`. The aggregate is the boundary.
- **Command-oriented**: external code submits a command value
  object; the engine validates, authorizes, dispatches, and
  emits events. The aggregate is internal; the command is the
  boundary.

The first model is what most DDD tutorials show. The second
is what production systems use.

The first model has problems at scale. A method on an
aggregate:

- Does not carry an actor — you cannot tell who called it
  without an extra argument.
- Does not carry a correlation id — you cannot trace
  causation across aggregates.
- Does not carry an idempotency key — retrying a network
  call may execute the method twice.
- Is hard to intercept — middleware for validation,
  authorization, audit, persistence, and event publishing
  would have to be repeated on every method.
- Is hard to test from the outside — the consumer's
  integration tests would have to construct an aggregate
  and call its method, bypassing all the framework.

The school domain is also deeply cross-cutting. Admitting a
student is a single user action, but it touches academic,
finance, library, communication, and potentially transport
in one workflow. A method-on-aggregate model would require
the admission code to know about every other domain.

## Decision

SMSengine adopts **command-oriented execution** as the only
sanctioned way to mutate engine state.

Concretely:

1. **Every mutation is a command** — a typed value object that
   carries the tenant context, the payload, and the metadata.
2. **A single command bus** is the entry point. Consumers call
   `engine.execute(command)` or `engine.<domain>().<action>(cmd)`.
3. **The dispatcher** runs a fixed pipeline: authenticate →
   validate → authorize → load aggregate → check preconditions
   → mutate → persist → publish events → write audit.
4. **The aggregate is internal to the dispatcher.** Consumers
   do not hold references to aggregates. Consumers do not
   call aggregate methods directly. There is no "public"
   `student.withdraw()` API.
5. **Commands are versioned.** `command_version` is part of
   the envelope; consumers send the version they expect.
6. **Commands are idempotent on `idempotency_key`.** A retry
   of the same key returns the original outcome (see
   `ADR-014-Idempotency.md`).
7. **Commands are bulk-aware.** A bulk command is a first-
   class command, not a loop of individual commands. The
   bulk command's transaction is all-or-nothing.
8. **Commands have explicit failure modes.** A command
   returns `Result<CommandOutcome, DomainError>` with a
   typed error variant (see `command-schema.md` § 14).

The command catalog is the engine's public surface. It is
documented per domain in `docs/specs/<domain>/commands.md`
and in the engine's rustdoc.

## Consequences

### Positive

- **A single, auditable mutation path.** Every state change
  flows through the same pipeline. The audit, the event
  emission, the persistence, the authorization are
  impossible to skip.
- **A single, testable surface.** The consumer's integration
  tests submit commands; they do not have to construct
  aggregates.
- **Cross-domain workflows are explicit.** A command can
  emit multiple events; subscribers react; the
  `correlation_id` ties the workflow together.
- **Idempotency is structural.** A command carries its
  idempotency key; the engine handles retries.
- **AI agents see the same surface.** The command catalog
  is the agent's tool list. The agent invokes a command;
  the engine handles validation, authorization, and
  persistence.
- **Middleware is composable.** New cross-cutting concerns
  (rate limiting, feature flags, A/B testing) are inserted
  into the pipeline without touching domain code.

### Negative

- **Indirection.** A developer must follow the command from
  the entry point, through the dispatcher, into the
  aggregate, and back out through the events. The
  indirection is a cost; the auditability is the benefit.
- **Boilerplate per command.** A command struct, its
  capability list, its dispatcher, its preconditions, its
  events, its tests. The `smsengine-core` crate provides
  macros and derive helpers to reduce boilerplate.
- **No "transaction script" escape hatch.** If a consumer
  wants to do "just a quick update" without a command, the
  answer is "no, write a command." This is intentional.
- **Async is mandatory at the boundary.** The dispatcher is
  async; the consumer's code is async. Pure-domain logic
  stays sync.

### Mitigations

- A command macro (`#[derive(Command)]`) generates the
  boilerplate (capability list, type name, validation
  hooks).
- A code generator can read the engine's spec and emit
  command skeletons, event skeletons, and tests.
- The engine's test kit provides a `TestEngine` that runs
  commands in-memory without an event bus or a real
  storage adapter, reducing the test loop.

## Alternatives Considered

### 1. Method-on-aggregate (repository pattern)

The consumer holds a `Student` and calls `student.withdraw()`.
Rejected because the consumer would have to re-implement
validation, authorization, audit, idempotency, and event
publishing on every call site.

### 2. RPC / function calls per domain

`studentService.withdraw(studentId, reason)`. Rejected for
the same reasons, with the added cost of network serialization
even in single-process deployments.

### 3. Event-sourced commands

The consumer submits a command that is itself a partial
event; the engine appends to the event log and projects.
We considered this; the engine's events are the source of
truth, but the **command** is the public surface. The
event log is an internal projection used for audit,
offline sync, and event-driven integration.

### 4. CQRS with separate write model

Two models: write (commands) and read (queries). SMSengine
already has this — the command layer is the write model;
the query layer is the read model. The query layer is
built on the `#[derive(DomainQuery)]` procedural macro,
which emits compile-time field enums and state builders
that translate into a typed `QueryNode` AST. Adapters
translate the AST into the storage dialect. We do not,
however, require that the write model and the read model
live in different processes or different storage. The
consumer can colocate them.

### 5. GraphQL mutations

GraphQL is a query language, not a mutation language. The
engine's commands are invoked from a GraphQL resolver;
GraphQL is the consumer's choice, not the engine's.
