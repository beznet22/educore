# ADR-001: Domain-Driven Design

## Status

Accepted.

## Context

Schools are not just data. A school is a living system with deep
rules: a student can be admitted in one year and not the next, an
invoice can be partial-paid but not over-paid, a teacher can be
assigned to a class only if they hold the matching subject
qualification, a payroll can be generated only after attendance has
been recorded for the period. These are not application-level
"checks" that can be sprinkled on top of CRUD. They are the
**domain's invariants** — the rules that, if violated, make the
system wrong in a way the school cannot recover from operationally.

A traditional layered architecture (controllers over services over
repositories over an ORM) tends to dissolve these invariants. The
business rule that "an invoice's paid amount cannot exceed its
total" lives in a service method, gets duplicated in a UI
validator, gets re-implemented in an export job, and quietly
diverges. Three months later, the accountant is staring at a
negative balance and nobody can find which copy of the rule is
authoritative.

The school domain is also **complex** in the technical sense — it
spans at least 15 bounded contexts (academic, finance, hr,
attendance, assessment, library, transport, communication, etc.),
each with its own aggregates, lifecycles, and policies. The naive
approach — a single, monolithic "Student" record joined to
everything — does not survive contact with the realities of
operational data (multiple academic years, partial payments, audit
trails).

## Decision

Educore adopts **Domain-Driven Design** as the architectural
philosophy for the engine.

Concretely:

1. The engine is organized as a set of **bounded contexts**, one
   per domain. Each bounded context has its own aggregates, value
   objects, domain services, policies, and events.
2. Every aggregate root enforces its own **consistency boundary**:
   external code may not reach into an aggregate's children. All
   mutations are serialized through the root.
3. **Value objects** are immutable and validated at construction.
   An `AdmissionNumber`, an `EmailAddress`, a `Money`, a
   `DateOfBirth` is a value object — never a `String`, never an
   `i64`.
4. **Identifiers** are typed (`StudentId`, `ClassId`, `FeeId`).
   A `StudentId` is `(SchoolId, Uuid)`. Two different aggregates
   cannot accidentally share an id type.
5. **Domain events** describe facts that have already happened, in
   past tense (`StudentAdmitted`, `PaymentCollected`). They are
   the source of truth for cross-domain integration.
6. **Commands** describe intent, in imperative tense
   (`AdmitStudent`, `CollectPayment`). They are the only way to
   mutate state.
7. **Domain services** orchestrate logic that does not fit in one
   aggregate (e.g. promoting a class of students, generating
   monthly payroll).
8. **Policies** are pure functions over state, returning a
   decision. They are easy to test and easy to compose.
9. **Specifications** are composable predicates for queries.
10. The **ubiquitous language** is enforced by the type system.
    The Rust types themselves carry the vocabulary of the school.

The model is encoded in the engine's rustdoc and in this
documentation. Domain experts, not engineers, are the final
authority on the model.

## Consequences

### Positive

- **Invariants live in one place.** The rule that an invoice's
  paid amount cannot exceed its total is enforced by the
  `Invoice::record_payment` method, with a test. There is no
  second copy to drift.
- **The type system prevents whole categories of bugs.** You
  cannot pass a `ClassId` where a `StudentId` is expected. You
  cannot construct an `EmailAddress` that is not RFC 5322-valid.
- **The domain model survives implementation changes.** Storage
  adapters, UIs, and event buses can be swapped without touching
  the model.
- **Cross-domain integration is explicit.** The `StudentAdmitted`
  event is the contract between academic and finance. There is
  no implicit join or shared schema.
- **AI agents and humans share the same model.** The capability
  catalog is the same regardless of who invokes the command.

### Negative

- **Higher upfront design cost.** DDD requires deep modelling
  before code. The engine cannot be built by churning out
  endpoints.
- **Bounded context boundaries require negotiation.** When finance
  needs student data, the academic domain publishes events;
  finance does not reach in. This requires discipline.
- **The ubiquitous language is a long-term commitment.** Renaming
  an aggregate ripples through every layer. The benefit is
  stability; the cost is inflexibility.
- **The engine's surface is large.** A domain has aggregates,
  commands, events, value objects, services, policies,
  specifications, repositories. A small team can drown.
- **Eventual consistency between bounded contexts.** A
  `StudentAdmitted` event reaches finance asynchronously. The
  consumer must handle "the fees assignment does not exist yet"
  gracefully.

### Mitigations

- The engine's domain crates are sized for incremental delivery
  (see `build-plan.md`). One bounded context at a time.
- The `educore-core` crate provides shared building blocks
  (identifier trait, error type, result, value object trait) to
  reduce boilerplate.
- The `educore-events` crate provides a stable event bus and
  subscription primitives.
- The `educore-events` semantics are "at-least-once with
  idempotency," which absorbs the eventual-consistency window.

## Alternatives Considered

### 1. Transaction Script

Each command is a procedure. Business rules live in services.
Rejected because the school domain is too complex; the
transaction-script model would dissolve the invariants the
moment a second team touches the code.

### 2. Active Record

Aggregates know how to persist themselves. Rejected because it
ties the domain model to a specific storage technology, which
violates the engine's port-driven storage policy.

### 3. Anemic Domain Model

Aggregates are data bags; all logic is in services. Rejected
because it pushes the same invariant problem out of the aggregate
and into every service that touches it. The DDD "rich domain
model" is a deliberate correction of this anti-pattern.

### 4. CRUD over a single shared schema

A single `Student` table joined to everything. Rejected because
it cannot model the school's reality (multiple academic years,
per-year records, partial payments, soft delete vs hard delete,
audit trails).

### 5. Microservices per Aggregate

Each aggregate is its own service. Rejected because it
over-distributes; the engine's surface becomes operational
overhead (network, deployment, observability) that drowns the
domain. A modular monolith with clear bounded contexts is the
sweet spot for an embeddable engine.
