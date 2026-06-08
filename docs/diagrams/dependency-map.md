# Domain Dependency Map

A directed graph of every dependency the engine permits
between its crates. The graph is the authoritative answer
to "which crate may import which?" in SMScore.

## 1. Top-Level Dependency Graph

```mermaid
graph TB
    classDef foundation fill:#fff3e0,stroke:#e65100,stroke-width:3px
    classDef cross fill:#e8f5e9,stroke:#1b5e20,stroke-width:2px
    classDef core fill:#e1f5ff,stroke:#01579b,stroke-width:2px
    classDef support fill:#f3e5f5,stroke:#4a148c,stroke-width:2px
    classDef facade fill:#fce4ec,stroke:#880e4f,stroke-width:3px

    Core[smscore-core<br/>identifiers, errors, result, value objects]:::foundation
    Platform[smscore-platform<br/>SchoolId, UserId, TenantContext]:::foundation
    Events[smscore-events<br/>bus, envelope, schema registry]:::cross
    Rbac[smscore-rbac<br/>CapabilityCheckService]:::cross
    Settings[smscore-settings<br/>GeneralSettings]:::cross
    Audit[smscore-audit<br/>audit log, query, retention]:::cross

    Academic[smscore-academic]:::core
    Assessment[smscore-assessment]:::core
    Attendance[smscore-attendance]:::core
    Finance[smscore-finance]:::core
    Hr[smscore-hr]:::core

    Library[smscore-library]:::support
    Facilities[smscore-facilities]:::support
    Communication[smscore-communication]:::support
    Documents[smscore-documents]:::support
    Cms[smscore-cms]:::support
    Operations[smscore-operations]:::support

    Facade[smscore<br/>Engine facade]:::facade
    Testkit[smscore-testkit<br/>in-memory adapters]:::facade
    Macros[smscore-macros<br/>derive macros]:::facade

    Platform --> Core
    Events --> Core
    Rbac --> Platform
    Rbac --> Events
    Settings --> Platform
    Audit --> Platform

    Academic --> Core
    Academic --> Platform
    Academic --> Rbac
    Academic --> Events
    Academic --> Audit

    Assessment --> Academic
    Assessment --> Rbac
    Assessment --> Events
    Assessment --> Audit

    Attendance --> Academic
    Attendance --> Rbac
    Attendance --> Events
    Attendance --> Audit

    Finance --> Academic
    Finance --> Hr
    Finance --> Rbac
    Finance --> Events
    Finance --> Audit

    Hr --> Academic
    Hr --> Rbac
    Hr --> Events
    Hr --> Audit

    Library --> Academic
    Library --> Rbac
    Library --> Events
    Library --> Audit

    Facilities --> Academic
    Facilities --> Rbac
    Facilities --> Events
    Facilities --> Audit

    Communication --> Events
    Communication --> Rbac
    Communication --> Audit

    Documents --> Academic
    Documents --> Hr
    Documents --> Rbac
    Documents --> Events
    Documents --> Audit

    Cms --> Platform
    Cms --> Rbac
    Cms --> Events
    Cms --> Audit

    Operations --> Platform
    Operations --> Rbac
    Operations --> Events
    Operations --> Audit

    Facade --> Core
    Facade --> Platform
    Facade --> Events
    Facade --> Rbac
    Facade --> Settings
    Facade --> Audit
    Facade --> Academic
    Facade --> Assessment
    Facade --> Attendance
    Facade --> Finance
    Facade --> Hr
    Facade --> Library
    Facade --> Facilities
    Facade --> Communication
    Facade --> Documents
    Facade --> Cms
    Facade --> Operations

    Testkit --> Facade
    Macros --> Core
```

## 2. Layered View

```mermaid
graph TB
    L0["Layer 0<br/>smscore-core"]
    L1["Layer 1<br/>smscore-platform, smscore-events"]
    L2["Layer 2<br/>smscore-rbac, smscore-audit, smscore-settings"]
    L3["Layer 3<br/>smscore-academic"]
    L4["Layer 4<br/>smscore-assessment,<br/>smscore-attendance,<br/>smscore-hr, smscore-finance"]
    L5["Layer 5<br/>smscore-library, smscore-facilities,<br/>smscore-communication, smscore-documents,<br/>smscore-cms, smscore-operations"]
    L6["Layer 6<br/>smscore facade + smscore-testkit"]

    L0 --> L1
    L1 --> L2
    L2 --> L3
    L3 --> L4
    L4 --> L5
    L5 --> L6

    style L0 fill:#fff3e0,stroke:#e65100,stroke-width:3px
    style L1 fill:#e8f5e9,stroke:#1b5e20,stroke-width:2px
    style L2 fill:#e1f5ff,stroke:#01579b,stroke-width:2px
    style L3 fill:#f3e5f5,stroke:#4a148c,stroke-width:2px
    style L4 fill:#fce4ec,stroke:#880e4f,stroke-width:2px
    style L5 fill:#e0f7fa,stroke:#006064,stroke-width:2px
    style L6 fill:#f1f8e9,stroke:#33691e,stroke-width:3px
```

## 3. Forbidden Edges

```mermaid
graph LR
    Adapter[Adapter Crate] -.X forbidden.-> Domain[Domain Crate]
    Domain -.X forbidden.-> Adapter
    DomainA[Domain A] -.X forbidden.-> DomainB[Domain B in same layer]

    classDef forbidden stroke:#b71c1c,stroke-width:3px,fill:#ffebee
    class Adapter,Domain,DomainA,DomainB forbidden
```

The engine enforces:

- **Adapters may not be imported by domain crates.**
  The dependency is one-way: domain defines a port;
  adapter implements it.
- **Domain crates may not import other domain crates
  in the same layer.** `smscore-finance` and
  `smscore-hr` are both layer 4; they do not import
  each other. They communicate through events.
- **Domain crates may not import the facade.** The
  facade re-exports the domain surface; importing
  it from a domain would create a cycle.

## 4. Foundation Crate Internal Layout

```mermaid
graph TB
    Core[smscore-core]
    Platform[smscore-platform]
    Events[smscore-events]
    Rbac[smscore-rbac]
    Settings[smscore-settings]
    Audit[smscore-audit]

    subgraph coreSub [smscore-core]
        Id[identifiers<br/>Id trait, SchoolId, UuidId]
        Err[errors<br/>DomainError, kind, From]
        Vo[value objects<br/>Money, Email, Phone, DateOfBirth]
        Result[result<br/>Result + DomainError]
        Clock[clock<br/>Clock port, SystemClock]
        IdGen[id gen<br/>IdGenerator port, UuidV7]
    end

    subgraph platSub [smscore-platform]
        Sch[School aggregate]
        Usr[User aggregate]
        Tctx[TenantContext]
        Module[Module, ModuleLink]
        Cf[CustomField, LookupData]
    end

    subgraph evSub [smscore-events]
        Envelope[EventEnvelope]
        Bus[EventBus port]
        Outbox[Outbox port]
        Reg[Schema registry]
    end

    subgraph rbacSub [smscore-rbac]
        Cap[Capability enum]
        Role[Role aggregate]
        Svc[CapabilityCheckService]
    end

    subgraph setSub [smscore-settings]
        Gs[GeneralSettings]
        Theme[Theme]
        Lang[Language]
    end

    subgraph audSub [smscore-audit]
        Sink[AuditSink]
        Query[AuditQuery]
        Ret[RetentionPolicy]
        Red[Redactor]
    end

    Core --> Id
    Core --> Err
    Core --> Vo
    Core --> Result
    Core --> Clock
    Core --> IdGen

    Platform --> Core
    Events --> Core
    Rbac --> Platform
    Rbac --> Events
    Settings --> Platform
    Audit --> Platform
```

## 5. Domain Crate Standard Layout

```mermaid
graph TB
    Domain["smscore-<domain>/"]
    Domain --> Lib["src/lib.rs<br/>(re-exports public surface)"]
    Domain --> Aggregate["src/aggregate.rs<br/>(root types, invariants)"]
    Domain --> Entities["src/entities.rs<br/>(child entities)"]
    Domain --> ValueObjects["src/value_objects.rs<br/>(typed wrappers)"]
    Domain --> Commands["src/commands.rs<br/>(command structs + handlers)"]
    Domain --> Events["src/events.rs<br/>(event types)"]
    Domain --> Services["src/services.rs<br/>(domain services)"]
    Domain --> Policies["src/policies.rs<br/>(pure decision functions)"]
    Domain --> Repository["src/repository.rs<br/>(port trait)"]
    Domain --> Query["src/query.rs<br/>(query builder)"]
    Domain --> Errors["src/errors.rs<br/>(domain error enum)"]
    Domain --> Tests["tests/<br/>(integration tests)"]
    Domain --> Readme["README.md<br/>(crate purpose)"]
```

## 6. Cross-Domain Coordination Pattern

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
is the only shared medium.

## 7. Workspace Layout

```mermaid
graph TB
    Workspace["smscore (workspace)"]
    Workspace --> Cargo["Cargo.toml<br/>(workspace manifest)"]
    Workspace --> CargoLock["Cargo.lock<br/>(committed)"]
    Workspace --> Crates["crates/"]

    Crates --> Core["smscore-core/"]
    Crates --> Platform["smscore-platform/"]
    Crates --> Rbac["smscore-rbac/"]
    Crates --> Events["smscore-events/"]
    Crates --> Settings["smscore-settings/"]
    Crates --> Audit["smscore-audit/"]
    Crates --> Academic["smscore-academic/"]
    Crates --> Assessment["smscore-assessment/"]
    Crates --> Attendance["smscore-attendance/"]
    Crates --> Finance["smscore-finance/"]
    Crates --> Hr["smscore-hr/"]
    Crates --> Library["smscore-library/"]
    Crates --> Facilities["smscore-facilities/"]
    Crates --> Communication["smscore-communication/"]
    Crates --> Documents["smscore-documents/"]
    Crates --> Cms["smscore-cms/"]
    Crates --> Operations["smscore-operations/"]
    Crates --> Facade["smscore/<br/>(facade)"]
    Crates --> Testkit["smscore-testkit/"]
    Crates --> Macros["smscore-macros/"]
    Crates --> Cli["smscore-cli/"]
```
