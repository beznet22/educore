# Domain Dependency Map

A directed graph of every dependency the engine permits
between its 34 crates. Edges flow strictly upward: a
crate may only depend on crates in its own tier or
any tier below it. The authoritative allow/deny list
is [`AGENTS.md` § Dependency Rules](../../AGENTS.md#dependency-rules).

## 1. Crates by Tier

### core tier (3 crates) — infrastructure, depends on nothing

| Crate | Purpose |
| --- | --- |
| `smsengine-core` | Identifiers, `DomainError`, `Result`, value objects, clock, id generator. The root of every dependency. |
| `smsengine-query-derive` | Proc-macro crate that emits the typed query AST for `#[derive(DomainQuery)]`. The only macro crate shipped in v1. |
| `smsengine-storage` | Storage port trait; dialect-agnostic AST translation contract. |

### cross-cutting tier (7 crates) — cross-domain foundations

Each cross-cutting crate depends on `smsengine-core`. Some
additionally depend on `smsengine-platform`.

| Crate | Purpose |
| --- | --- |
| `smsengine-platform` | `School`, `User`, `TenantContext`, modules, custom fields. |
| `smsengine-rbac` | `Capability` enum, `Role` aggregate, `CapabilityCheckService`. |
| `smsengine-events` | `EventEnvelope`, `EventBus` and `Outbox` ports, schema registry. |
| `smsengine-events-domain` | Calendar and event-scheduling domain (the calendar, not the envelope). |
| `smsengine-audit` | `AuditSink`, `AuditQuery`, retention, redactor. Dev-dependency for domains; re-exported by the umbrella. |
| `smsengine-settings` | Per-school general settings, theme, language. |
| `smsengine-operations` | Cross-domain operations, jobs, scheduled tasks. |

### domains tier (10 crates) — the bounded contexts

Each domain crate depends on `smsengine-core`, `smsengine-platform`,
`smsengine-rbac`, and `smsengine-events`. Some additionally depend
on `smsengine-audit`. `smsengine-settings` omits `smsengine-audit`.

| Crate | Purpose |
| --- | --- |
| `smsengine-academic` | Academic year, classes, subjects, timetabling, grading scale. |
| `smsengine-assessment` | Examinations, marks, grade entry, report cards. |
| `smsengine-attendance` | Daily and period attendance, leave, notifications. |
| `smsengine-cms` | Content management: pages, menus, announcements. |
| `smsengine-communication` | Messaging, email, SMS, push channels. |
| `smsengine-documents` | Document storage and metadata (the engine-side catalog; files live in `smsengine-files`). |
| `smsengine-facilities` | Rooms, assets, inventory, maintenance. |
| `smsengine-finance` | Fees, invoices, payments, ledgers. |
| `smsengine-hr` | Staff records, payroll inputs, contracts. |
| `smsengine-library` | Books, lending, reservations. |

### adapters tier (9 crates) — port implementations

Storage and event-bus adapters depend on `smsengine-storage` /
`smsengine-events` and may not be imported by domain crates. The
6 port adapters (auth, files, integrations, notify, payment, ...)
implement the engine's outbound ports for specific backends.

| Crate | Purpose |
| --- | --- |
| `smsengine-storage-postgres` | PostgreSQL 14+ storage adapter; reference DDL emitter. |
| `smsengine-storage-mysql` | MySQL 8.0+ storage adapter; production target. |
| `smsengine-storage-sqlite` | SQLite 3.x storage adapter; embedded and offline mode. |
| `smsengine-event-bus` | In-process `EventBus` adapter implementing the `smsengine-events` port. |
| `smsengine-auth` | Identity, sessions, OAuth/OIDC. |
| `smsengine-files` | Object storage adapter (S3, local, GCS). |
| `smsengine-integrations` | Third-party integration adapters (LMS, SIS, proctors). |
| `smsengine-notify` | Notification channel adapters (email, SMS, push). |
| `smsengine-payment` | Payment gateway adapters (Stripe, Razorpay, manual). |

### tools tier (4 crates) — dev tooling

| Crate | Purpose |
| --- | --- |
| `smsengine-testkit` | In-memory adapters and test helpers; used by domain and integration tests. |
| `smsengine-storage-parity` | Cross-dialect conformance harness; runs the same test suite against every storage adapter. |
| `smsengine-sdk` | High-level consumer SDK; convenient builders for the common flows. |
| `smsengine-cli` | Sample binary CLI (`[[bin]]`); not a library. Demonstrates consumer use of the umbrella. |

### umbrella (1 crate)

| Crate | Purpose |
| --- | --- |
| `smsengine` | Umbrella crate; re-exports every public crate under `smsengine::*` and is the only crate consumers are expected to depend on directly. |

## 2. Top-Level Dependency Graph

The Mermaid diagram below shows the 34 crates and the
permitted dependency edges. Edges point from dependent
to dependency. A crate in a higher tier depends only on
crates in its own tier or lower; it never depends on a
crate in a higher tier. Mermaid identifiers cannot
contain hyphens, so node IDs use underscores; the human-
readable label uses the hyphenated package name.

```mermaid
graph TD
    classDef core fill:#fff3e0,stroke:#e65100,stroke-width:3px
    classDef cross fill:#e8f5e9,stroke:#1b5e20,stroke-width:2px
    classDef domain fill:#f3e5f5,stroke:#4a148c,stroke-width:2px
    classDef adapter fill:#e1f5ff,stroke:#01579b,stroke-width:2px
    classDef tool fill:#e0f7fa,stroke:#006064,stroke-width:3px
    classDef umbrella fill:#fce4ec,stroke:#880e4f,stroke-width:3px

    %% core tier
    core["smsengine-core"]:::core
    query_derive["smsengine-query-derive"]:::core
    storage["smsengine-storage"]:::core
    %% core internal edges
    query_derive --> core
    storage --> core
    %% cross-cutting tier
    platform["smsengine-platform"]:::cross
    rbac["smsengine-rbac"]:::cross
    events["smsengine-events"]:::cross
    events_domain["smsengine-events-domain"]:::cross
    audit["smsengine-audit"]:::cross
    settings["smsengine-settings"]:::cross
    operations["smsengine-operations"]:::cross
    %% cross-cutting internal edges
    platform --> core
    rbac --> core & platform & events
    events --> core
    events_domain --> core & platform & rbac & events & audit
    audit --> platform & core
    settings --> core & platform & rbac & events
    operations --> core & platform & rbac & events & audit
    %% domains tier
    academic["smsengine-academic"]:::domain
    assessment["smsengine-assessment"]:::domain
    attendance["smsengine-attendance"]:::domain
    cms["smsengine-cms"]:::domain
    communication["smsengine-communication"]:::domain
    documents["smsengine-documents"]:::domain
    facilities["smsengine-facilities"]:::domain
    finance["smsengine-finance"]:::domain
    hr["smsengine-hr"]:::domain
    library["smsengine-library"]:::domain
    %% domain internal edges: every domain depends on core+platform+rbac+events;
    %% many also use audit.
    academic & assessment & attendance & cms & communication & documents & facilities & finance & hr & library --> core & platform & rbac & events
    academic & assessment & attendance & cms & communication & documents & facilities & finance & hr & library --> audit
    %% adapters tier
    storage_postgres["smsengine-storage-postgres"]:::adapter
    storage_mysql["smsengine-storage-mysql"]:::adapter
    storage_sqlite["smsengine-storage-sqlite"]:::adapter
    event_bus["smsengine-event-bus"]:::adapter
    auth["smsengine-auth"]:::adapter
    files["smsengine-files"]:::adapter
    integrations["smsengine-integrations"]:::adapter
    notify["smsengine-notify"]:::adapter
    payment["smsengine-payment"]:::adapter
    %% adapter internal edges
    storage_postgres --> storage & core
    storage_mysql --> storage & core
    storage_sqlite --> storage & core
    event_bus --> events & core
    auth --> core & storage & events
    files --> core & storage & events
    integrations --> core & events
    notify --> core & events
    payment --> core & storage & events
    %% tools tier
    testkit["smsengine-testkit"]:::tool
    storage_parity["smsengine-storage-parity"]:::tool
    sdk["smsengine-sdk"]:::tool
    cli["smsengine-cli"]:::tool
    %% tool internal edges
    testkit --> core & storage & events
    storage_parity --> core & storage_postgres & storage_mysql & storage_sqlite
    sdk --> core & platform & events & rbac & settings
    cli --> umbrella & sdk
    %% umbrella re-exports every public crate
    umbrella["smsengine<br/>umbrella"]:::umbrella
    umbrella --> core & query_derive & storage
    umbrella --> platform & rbac & events & events_domain & audit & settings & operations
    umbrella --> academic & assessment & attendance & cms & communication & documents & facilities & finance & hr & library
    umbrella --> storage_postgres & storage_mysql & storage_sqlite & storage_parity & event_bus
    umbrella --> auth & files & integrations & notify & payment
    umbrella --> testkit & sdk
```

## 3. Forbidden Edges

```mermaid
graph LR
    Adapter[Adapter Crate] -.X forbidden.-> Domain[Domain Crate]
    Domain -.X forbidden.-> Adapter
    DomainA[Domain A] -.X forbidden.-> DomainB[Domain B in same tier]
    DomainAny[Domain Crate] -.X forbidden.-> Umbrella[smsengine facade]
    DomainAny2[Domain Crate] -.X forbidden.-> Sdk[smsengine-sdk]
    DomainAny3[Domain Crate] -.X forbidden.-> Tool[tools/* crate]

    classDef forbidden stroke:#b71c1c,stroke-width:3px,fill:#ffebee
    class Adapter,Domain,DomainA,DomainB,DomainAny,DomainAny2,DomainAny3,Umbrella,Sdk,Tool forbidden
```

The engine enforces:

- **Adapters may not be imported by domain crates.**
  The dependency is one-way: domain defines a port;
  adapter implements it.
- **Domain crates may not import other domain crates
  in the same tier.** `smsengine-finance` and
  `smsengine-hr` are both in the domains tier; they
  do not import each other. They communicate through
  events.
- **Domain crates may not import the umbrella, the SDK,
  `smsengine-cli`, or any other `tools/*` crate.** The
  umbrella and SDK re-export the domain surface;
  importing either from a domain would create a cycle.
- **The proc-macro crate `smsengine-query-derive` is a
  leaf.** It depends on `syn`/`quote` only and is used
  by every domain crate at compile time. No runtime
  crate may depend on it.
- **`smsengine-audit`, `smsengine-testkit`, and
  `smsengine-storage-parity` are not load-bearing for
  production.** They are imported only by the umbrella
  and (for `testkit`/`storage-parity`) by the workspace
  test harness. Domains treat them as optional
  dev-dependencies.

## 4. Cross-Domain Coordination Pattern

```mermaid
graph LR
    A[Domain A<br/>aggregate] -- emits event --> Bus[(Event Bus)]
    Bus -- delivered to --> B[Domain B<br/>subscriber]
    B -- emits event --> Bus
    Bus -- delivered to --> C[Domain C<br/>subscriber]

    style A fill:#e1f5ff,stroke:#01579b,stroke-width:2px
    style B fill:#e8f5e9,stroke:#1b5e20,stroke-width:2px
    style C fill:#fff3e0,stroke:#e65100,stroke-width:2px
    style Bus fill:#fce4ec,stroke:#880e4f,stroke-width:3px
```

Cross-domain coordination happens through events.
Domain A does not call Domain B directly. Domain B
subscribes to Domain A's events and reacts. The bus
is the only shared medium. The `EventBus` port lives
in `smsengine-events`; the default in-process adapter
is `smsengine-event-bus`.

## See also

- [`AGENTS.md` § Dependency Rules](../../AGENTS.md#dependency-rules) — the authoritative allow/deny list.
- [`AGENTS.md` § Tier System](../../AGENTS.md#tier-system) — the on-disk tier tree and dependency direction.
- [`AGENTS.md` § Workspace Layout](../../AGENTS.md#workspace-layout) — the on-disk tree of crates.
- [`AGENTS.md` § Project Identity](../../AGENTS.md#project-identity) — package-vs-directory naming convention.
- [`docs/architecture.md`](../architecture.md) — the broader system map.
