# Educore Architecture

## System Architecture

Educore is a hexagonal, domain-driven, event-driven, command-oriented Rust
engine organized as a Cargo workspace.

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                        Consumer Applications                            │
│   CLI   Desktop   Tauri   Mobile   Web   SaaS   AI Agent   Automation   │
└─────────────────────────────────────┬───────────────────────────────────┘
                                      │ invokes commands
                                      ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                       Engine Facade (educore::Engine)                    │
│                                                                          │
│   students()  attendance()  examinations()  finance()  hr()  ...        │
│   rbac()  library()  transport()  events()  reports()                   │
└─────────────────────────────────────┬───────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                          Command Bus + Dispatcher                        │
│                                                                          │
│   Authn → Authz → Validation → Aggregate Load → Domain Logic            │
│   → Event Emission → Persistence → Bus Publish → Audit Write            │
└─────────────────────────────────────┬───────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                          Domain Core (pure Rust)                         │
│                                                                          │
│   Academic   Assessment   Attendance   Finance   HR   Library           │
│   Facilities Communication Documents   Events   CMS   Platform          │
│   RBAC      Settings     Operations                                    │
│                                                                          │
│   Aggregates • Entities • Value Objects • Domain Services               │
│   Policies • Specifications • Domain Events                             │
└─────────────────────────────────────┬───────────────────────────────────┘
                                      │ ports
                                      ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                              Ports (Traits)                              │
│                                                                          │
│   Storage   Authentication   Notification   Payment                     │
│   FileStorage   EventBus   Identity   Clock   IdGen   Audit              │
│   Integration   Indexer   Search                                         │
└─────────────────────────────────────┬───────────────────────────────────┘
                                      │ implemented by
                                      ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                       Adapters (consumer-supplied)                       │
│                                                                          │
│   PostgreSQL/MySQL/SQLite (+ SurrealDB, MongoDB deferred)   OAuth/SAML/Local                         │
│   SMTP/SMS/Push/FCM              Stripe/PayPal/Cash/Bank                │
│   S3/GCS/Local                   Inproc/Redis/NATS                       │
└─────────────────────────────────────────────────────────────────────────┘
```

## DDD Model

Educore models a school as a set of bounded contexts. Each context has its
own aggregates, value objects, domain services, and policies.

```text
school
├── academic           Student lifecycle, classes, sections, subjects, sessions
├── assessment         Examinations, marks, results, report cards
├── attendance         Daily attendance for students and staff
├── finance            Fees, invoices, payments, expenses, banking
├── hr                 Staff lifecycle, payroll, leave, attendance
├── library            Books, members, issues, returns
├── facilities         Hostel, transport, inventory
├── communication      Notices, complaints, chat, notifications
├── events             Calendar, holidays, incidents
├── documents          Forms, postal dispatch/receive
├── cms                Website pages, news, content
├── rbac               Roles, permissions, capability assignments
├── platform           Multi-school tenancy, users, schools
├── settings           General settings, theming, language
└── operations         Backups, jobs, system versions
```

The aggregate boundaries are derived from the migration schema and are documented
per-domain under `docs/specs/<domain>/aggregates.md`.

## Hexagonal Architecture

The domain core depends on no adapter. It defines ports (Rust traits) and
domain errors. Adapters are out-of-tree consumer crates that implement the
ports.

```text
                              ┌────────────────┐
                              │  Domain Core   │
                              │                │
                              │  Aggregates    │
                              │  Value Objects │
                              │  Domain Svcs   │
                              │  Domain Events │
                              │  Policies      │
                              │  Specifications│
                              └───────┬────────┘
                                      │
                              ┌───────▼────────┐
                              │     Ports      │
                              │  (Traits)      │
                              │  no_std where  │
                              │  possible      │
                              └───────┬────────┘
                                      │
                ┌────────────┬───────┴────────┬─────────────┐
                │            │                │             │
        ┌───────▼──────┐ ┌──▼────────┐ ┌──────▼──────┐ ┌────▼─────┐
        │   Storage    │ │    Auth   │ │Notification │ │ Payment  │
        │   Adapter    │ │  Adapter  │ │   Adapter   │ │ Adapter  │
        └──────────────┘ └───────────┘ └─────────────┘ └──────────┘
```

## Domain Boundaries

A consumer is never required to use every domain. Domains are wired through
the `Engine` facade and can be enabled per deployment. Dependencies between
domains are explicit and one-way where possible:

```text
platform  ───► all domains (provides SchoolId, Tenant, User)
rbac      ───► all domains (provides Capability check)
settings  ───► all domains (provides configuration values)
events    ───► all domains (provides domain event publishing)
audit     ───► all domains (provides audit trail)

academic     ────►  assessment, attendance, finance, library
assessment   ────►  academic (read), finance (fees linkage optional)
attendance   ────►  academic, communication (absent notifications)
finance      ────►  academic, hr (payroll), events (payment receipts)
hr           ────►  academic, finance (payroll)
library      ────►  academic
facilities   ────►  academic (transport routes use class/section)
communication ──►  all (sends based on domain events)
documents    ────►  academic, hr (document targets)
cms          ────►  platform
```

Concrete dependency rules:

- `platform` depends on nothing in the engine.
- `rbac` depends only on `platform`.
- `settings` depends only on `platform`.
- `academic` is the foundational domain; everything else builds on it.
- Domain services in one domain must not directly call repositories of another.
  Cross-domain coordination happens through domain events and command
  composition, not through direct service calls.

## Port Architecture

Educore defines the following ports in `docs/ports/`:

| Port              | Trait                         | Consumer adapter example                    |
| ----------------- | ----------------------------- | ------------------------------------------- |
| Storage           | `StorageAdapter`              | Postgres adapter, SQLite adapter            |
| Authentication    | `AuthProvider`                | JWT, OAuth2, SAML, password                 |
| Notification      | `NotificationProvider`        | SMTP, SMS, FCM, APNs                        |
| Payment           | `PaymentProvider`             | Stripe, PayPal, bank slip                   |
| FileStorage       | `FileStorage`                 | S3, GCS, local filesystem                   |
| EventBus          | `EventBus`                    | In-process, NATS, Redis                     |
| IdGenerator       | `IdGenerator`                 | UUIDv7, ULID, snowflake                     |
| Clock             | `Clock`                       | System, frozen test clock                   |
| AuditSink         | `AuditSink`                   | DB, file, SIEM                              |
| SearchIndex       | `SearchIndex`                 | Meilisearch, Typesense, Postgres FTS        |
| Integration       | `IntegrationGateway`          | LMS, video conferencing, SMS gateway        |
| Identity          | `IdentityProvider`            | External IdP                                |

Ports are documented in `docs/ports/*.md` with full method signatures,
error semantics, and adapter responsibilities.

## Event Architecture

The engine is event-driven at the domain level. Every state-changing command:

1. Loads the relevant aggregate.
2. Executes the business operation.
3. Records one or more domain events describing what happened.
4. Persists the new state and uncommitted events transactionally.
5. Publishes the events to the event bus.
6. Writes the audit record referencing the events.

Domain events are typed and serializable. Events are the source of truth for
cross-domain integration, audit reconstruction, analytics, and offline
reconciliation.

Events use a stable schema with:

- `event_id` (UUIDv7)
- `event_type` (fully-qualified type name)
- `aggregate_id` (typed identifier)
- `school_id` (tenant)
- `occurred_at` (timestamp from Clock)
- `actor_id` (user that triggered)
- `correlation_id` (causality across processes)
- `payload` (domain-specific, versioned)

Event schema: `docs/schemas/event-schema.md`.

## Multi-Tenancy

Multi-tenancy is **structural** in Educore. Every aggregate root contains
`SchoolId`. The engine never queries without a tenant filter, the storage
adapters are responsible for enforcing tenant isolation through queries, and
the type system prevents cross-tenant reference by distinguishing `StudentId`
in school A from `StudentId` in school B through their `SchoolId` context.

Tenancy rules:

- Every aggregate is anchored to a `SchoolId`.
- The engine refuses to execute a command whose target aggregate is in a
  different school than the actor's session.
- Cross-school integrations happen through explicit, capability-gated commands
  (e.g. `TransferStudent`).
- Storage adapters MAY add a database-level row-security policy on `school_id`.

Tenancy schema: `docs/schemas/tenancy-schema.md`.

## Command Layer

Commands are the only sanctioned way to mutate state. Each command is:

- A typed value object describing the intent.
- Validated at the boundary (struct shape, references, business preconditions).
- Authorized via the RBAC port (capability check).
- Dispatched to a single aggregate.
- Produces zero or more events.
- Returns the resulting state summary or a domain error.

Consumers never write directly to a storage table. All mutations flow through
commands. The command catalog: `docs/commands/*.md`.

A command has the form:

```rust
pub struct PublishResultCommand {
    pub school_id: SchoolId,
    pub exam_id: ExamId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub actor_id: UserId,
    pub correlation_id: CorrelationId,
    pub published_at: Timestamp,
}
```

The engine exposes commands through the facade:

```rust
engine
    .assessment()
    .publish_result(command)
    .await?;
```

## Storage Strategy

The storage port is a thin trait that repositories translate to. The default
adapters target PostgreSQL, MySQL, and SQLite. SurrealDB and MongoDB adapters
are deferred to a future release; they are admissible through the same trait
when implemented in-tree by a consumer.

The engine assumes:

- A single logical database per school in single-tenant mode.
- A shared database with `school_id` filtering in multi-tenant SaaS.
- Foreign-key aware relational storage by default.
- Document storage via a column-based or JSON adapter when required.

The engine does **not** assume:

- A specific SQL dialect.
- Triggers, stored procedures, or views.
- A specific connection pool implementation.
- A specific migration tool.

Migrations are owned by the consumer.

The end-to-end flow from `docs/specs/<domain>/tables.md` (design
contract) to the runtime DDL string (executable contract) is
documented in
[`docs/schemas/sql-dialects/README.md` § "Runtime DDL emission"](../schemas/sql-dialects/README.md#runtime-ddl-emission--end-to-end-flow).
The engine emits DDL **at schema-creation time** (once per process
lifetime, via `storage.create_schema().await`) from a typed macro
AST and the `include_str!`-embedded canonical SQL for the 6
engine cross-cutting tables.

## Query Layer

The query layer is described in `docs/query_layer.md` and is mandatory. It
is built on the `#[derive(DomainQuery)]` procedural macro, which emits a
field-exhaustiveness enum (`StudentField`), a typed state builder
(`StudentQueryBuilder`), and a relation enum (`StudentRelation`). The
macro is strictly an AST generator — it never produces SQL, NoSQL, or
any storage-specific syntax. Storage translation lives in adapter
crates.

The query layer is:

- Compile-time safe. Field identifiers, operators, ordering, and
  pagination are all typed.
- Macro-driven. Field enums and builders are generated by
  `#[derive(DomainQuery)]`, never hand-written.
- Storage-agnostic. Adapters translate the macro-emitted AST.
- Reflection-free. No runtime introspection, no `serde_json::Value`
  field lookups.
- Schema-introspection-free. The engine does not parse migrations.
- Repository-oriented. The query exists to support repositories.
- Closure-safe for nested relations. `where_has` accepts a closure
  bound to the related entity's macro-generated builder.
- Strictly eager. `.with(...)` is the only path to populate related
  fields; lazy accessors and async getters are forbidden.

Example — typed filter chain with scope trait, nested filter, and
explicit eager load:

```rust
use educore::students::query::{StudentQueryScopes, StudentRelation, ParentField};

let active = engine
    .students()
    .query()
    .active()                          // extension trait scope
    .in_class(class_id)                // extension trait scope
    .where_has(StudentRelation::Parent, |parent_q| {     // closure-bound
        parent_q.where_eq(ParentField::BillingStatus, BillingStatus::Active)
    })
    .with(StudentRelation::Parent)     // explicit hydration
    .page(0, 50)
    .await?;
```

Repositories may expose domain-specific optimized queries that bypass
the generic builder when needed. Optimized queries are typed
capabilities, not generic magic, and must return fully hydrated graphs.

## Reference Documents

- `docs/specs/` — domain specifications
- `docs/ports/` — port contracts
- `docs/commands/` — command catalog
- `docs/events/` — event catalog
- `docs/schemas/` — cross-cutting schemas (tenancy, audit, events, permissions)
- `docs/decisions/` — architectural decisions
- `docs/diagrams/` — visual maps
- `docs/research/` — business knowledge extracted from Schoolify
- `docs/guides/` — implementation guides for specific concerns
- `docs/query_layer.md` — query layer specification
- `docs/build-plan.md` — implementation roadmap
- `docs/code-standards.md` — engineering rules
- `docs/library-docs.md` — consumer-facing SDK documentation

## Tier System

The 34 crates are organized into **5 tiers + 1 umbrella**. The tier
names and dependency direction are fixed at the filesystem level: a
crate in `crates/domains/` may not import from `crates/adapters/` or
`crates/tools/`, and a crate in `crates/cross-cutting/` may not
import from `crates/domains/`, `crates/adapters/`, or
`crates/tools/`. Tier boundaries are enforced at build time by the
`educore-core::lint` sub-module.

| Tier            | Path                            | Count | Purpose |
| --------------- | ------------------------------- | ----- | ------- |
| `infra`         | `crates/infra/`                 | 3     | Infrastructure: errors, identifiers, value objects, query AST, proc-macro, storage port. Depends on nothing. |
| `cross-cutting` | `crates/cross-cutting/`         | 7     | Cross-domain foundations: platform, rbac, events envelope, audit, settings, operations, calendar. Depends on `infra`. |
| `domains`       | `crates/domains/`               | 10    | The 10 domain bounded contexts (academic, finance, hr, ...). Depends on `infra` and `cross-cutting`. |
| `adapters`      | `crates/adapters/`              | 9     | Port implementations: 3 storage adapters + 6 port adapters. Depends on `infra` and `cross-cutting`. |
| `tools`         | `crates/tools/`                 | 4     | Dev tooling: testkit, storage-parity, cli (binary), sdk. Depends on all of the above. |
| umbrella        | `crates/educore/`             | 1     | Re-exports the public surface of all 34 internal crates. |

Layered dependency direction (no cycles, no upward deps):

```text
infra  <-  cross-cutting  <-  domains  <-  tools
                          ^
                          +--  adapters  (also depends on infra + cross-cutting)
```

Internal crate directories are named without the `educore-`
prefix (e.g. `crates/domains/academic/`, `crates/adapters/storage-postgres/`),
while the published package name keeps the prefix
(`educore-academic`, `educore-storage-postgres`). The umbrella
re-exports each internal crate under its short name, so consumers
write `educore::academic::commands::*` and never need to know the
internal `educore-` prefix on the package name.

See `AGENTS.md` § "Tier System" for the full rules, and
`docs/build-plan.md` § "The No-Gaps Gates" for how tier boundaries
are enforced at build time.
