# ADR-002: Hexagonal Architecture

## Status

Accepted.

## Context

SMSengine is an embeddable engine. It will be consumed by:

- A SaaS platform serving thousands of schools on PostgreSQL.
- An on-premise deployment for a single school on SQLite.
- A Tauri desktop app for an administrator working offline.
- A mobile parent app with intermittent connectivity.
- A Web API for a school district.
- An AI agent calling commands through a tool interface.

These consumers do not agree on:

- Storage: PostgreSQL, SQLite, SurrealDB, MongoDB, in-memory.
- Authentication: OAuth, SAML, JWT, password, magic link.
- Notifications: SMTP, SMS, FCM, APNs, in-process.
- Payments: Stripe, PayPal, bank slip, cash.
- File storage: S3, GCS, local filesystem, in-memory.
- Event bus: NATS, Redis, in-process.
- Clock: system, frozen, accelerated.

If the engine's domain code imported any of these directly — even
`tokio`, even `serde_json::Value` — the engine would either
constrain consumers to one technology stack, or accumulate
adapter logic until the domain is unreadable.

The engine's surface is also a **business** surface, not a
**technical** surface. An accountant does not care whether the
invoice is in PostgreSQL or SurrealDB; the accountant cares that
the invoice total cannot go negative. The engine's public API
should reflect the business, not the implementation.

## Decision

SMSengine adopts **hexagonal architecture** (also known as
"ports and adapters").

Concretely:

1. The **domain core** is a set of pure-Rust crates that import
   no infrastructure. No `tokio` in domain logic. No
   `serde_json::Value` in domain types. No `sqlx`, no
   `reqwest`, no `aws-sdk-s3`. Domain code depends only on
   `smsengine-core` and other domain crates.
2. **Ports** are Rust traits that define what the engine needs
   from the outside world. The engine owns the trait
   definitions. Adapters implement them.
3. **Adapters** are out-of-tree consumer crates. The engine
   does not ship adapters; consumers (or the consumer's library
   vendor) provide them.
4. Ports include: `StorageAdapter`, `AuthProvider`,
   `NotificationProvider`, `PaymentProvider`, `FileStorage`,
   `EventBus`, `IdGenerator`, `Clock`, `AuditSink`,
   `SearchIndex`, `IntegrationGateway`, `IdentityProvider`,
   `ConfigurationService`.
5. Domain logic that needs I/O takes a `&dyn Port` (or
   `Arc<dyn Port>`) as a constructor argument. The dependency
   is explicit and testable.
6. The engine ships a **testkit** with in-memory adapters for
   every port. Tests use the testkit; production uses the
   consumer's real adapters.

The engine's `Engine` facade wires the chosen adapters and
exposes a typed, async, business-facing API.

## Consequences

### Positive

- **Storage is a port, not a built-in.** Replacing PostgreSQL
  with SQLite, or with an in-memory store for tests, requires
  no domain code change.
- **Authentication is a port.** A consumer can wire OAuth, SAML,
  or local password without touching the engine.
- **The engine can be tested in-memory.** The testkit
  provides adapters that fit in a `#[tokio::test]` body with
  no infrastructure. This dramatically shortens the
  test-loop.
- **The domain is portable.** The same engine runs in a
  browser (via WASM, with appropriate ports), on a phone, on
  a server, or in a CLI.
- **The business surface is clean.** The accountant deals
  with `Invoice`, `Payment`, `BankStatement` — never with
  `tokio_postgres` or `aws-sdk-s3`.
- **AI agents see the same surface.** The capability catalog
  is the same regardless of the underlying storage or auth
  provider.

### Negative

- **Indirection has a cost.** A developer new to the codebase
  must learn "find the trait, find the adapter, find the
  consumer wiring" before they can follow a request through
  the system. This is offset by a single diagram (see
  `diagrams/dependency-map.md`).
- **More traits, more boilerplate.** Each port is a trait with
  a method for every operation. The engine's repository
  traits are not tiny. We accept this cost for portability.
- **No global service locator.** Code that needs a port takes
  it as a constructor argument. This is verbose at the
  composition root but pays for itself in testability.
- **Async / sync is a port-level decision.** The engine's
  ports are async because the engine assumes I/O. Pure-domain
  helpers stay sync.

### Mitigations

- The `smsengine-core` crate re-exports `tracing`, `async_trait`,
  and a curated set of common helpers to keep adapter code
  short.
- A standard `Engine::builder()` API wires the most common
  adapter combinations in a few lines.
- Documentation at `docs/ports/` for every port is mandatory
  and is generated alongside the trait.

## Alternatives Considered

### 1. Layered architecture (controllers → services → repos)

The default for many CRUD apps. Rejected because it couples the
domain to a particular storage technology (repositories own
SQL) and because it tends to dissolve invariants across
service boundaries.

### 2. Clean architecture

Conceptually similar to hexagonal. We accept clean
architecture's vocabulary (`entities`, `use cases`, `interface
adapters`) but find the more port-centric hexagonal framing
clearer for a Rust-idiomatic engine.

### 3. Plugin / service-locator pattern

Domain code reaches for a global registry to find an adapter.
Rejected because it makes the domain untestable in isolation
and obscures dependencies. Constructor injection is explicit
and clear.

### 4. Direct dependency on infrastructure crates

The engine imports `sqlx`, `aws-sdk-s3`, `tokio`, etc. Rejected
because it locks the engine to those technologies, bloats
domain code, and contradicts the engine's "embeddable
anywhere" goal.

### 5. Microkernel / OSGi-style

A plugin registry with versioned bundles. Rejected because
the engine is a library, not an OS. The Rust trait system
gives us the same modularity with type safety.
