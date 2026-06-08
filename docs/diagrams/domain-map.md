# Domain Map

This document maps the engine's 15 bounded contexts and the
directional dependencies between them. The map is the
authoritative view of "what depends on what?" in SMSengine.

## 1. All 15 Domains

```mermaid
graph TB
    subgraph foundation ["Foundation"]
        Platform[platform]
        Rbac[rbac]
        Settings[settings]
        Events[events]
        Audit[audit]
    end

    subgraph core ["Operational Core"]
        Academic[academic]
        Assessment[assessment]
        Attendance[attendance]
        Finance[finance]
        Hr[hr]
    end

    subgraph support ["Support"]
        Library[library]
        Facilities[facilities]
        Communication[communication]
        Documents[documents]
        Cms[cms]
        Operations[operations]
    end

    Rbac --> Platform
    Settings --> Platform
    Events --> Platform
    Audit --> Platform

    Academic --> Platform
    Academic --> Rbac
    Academic --> Events
    Academic --> Audit

    Assessment --> Platform
    Assessment --> Rbac
    Assessment --> Academic
    Assessment --> Events
    Assessment --> Audit

    Attendance --> Platform
    Attendance --> Rbac
    Attendance --> Academic
    Attendance --> Events
    Attendance --> Audit

    Finance --> Platform
    Finance --> Rbac
    Finance --> Academic
    Finance --> Hr
    Finance --> Events
    Finance --> Audit

    Hr --> Platform
    Hr --> Rbac
    Hr --> Academic
    Hr --> Finance
    Hr --> Events
    Hr --> Audit

    Library --> Platform
    Library --> Rbac
    Library --> Academic
    Library --> Events
    Library --> Audit

    Facilities --> Platform
    Facilities --> Rbac
    Facilities --> Academic
    Facilities --> Events
    Facilities --> Audit

    Communication --> Platform
    Communication --> Rbac
    Communication --> Events
    Communication --> Audit

    Documents --> Platform
    Documents --> Rbac
    Documents --> Academic
    Documents --> Hr
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
```

## 2. Foundation Domain Roles

```mermaid
graph LR
    subgraph foundationRoles ["What each foundation domain provides"]
        Platform["platform<br/>SchoolId, UserId, TenantContext<br/>School, User aggregates<br/>Custom fields, lookup data"]
        Rbac["rbac<br/>CapabilityCheckService<br/>Role, Capability, PermissionSection<br/>TwoFactorSetting"]
        Settings["settings<br/>GeneralSettings<br/>Theme, Language, Currency<br/>Per-tenant feature flags"]
        Events["events<br/>EventBus, EventEnvelope<br/>Schema registry, Outbox<br/>Subscription primitives"]
        Audit["audit<br/>AuditSink, AuditQuery<br/>Retention policy, Redactor<br/>Compliance reports"]
    end

    Consumer[Consumer] --> Platform
    Consumer --> Rbac
    Consumer --> Settings
    Consumer --> Events
    Consumer --> Audit
```

## 3. Cross-Domain Event Flow

```mermaid
graph LR
    Academic -- "StudentAdmitted<br/>StudentPromoted<br/>StudentWithdrawn" --> Finance
    Academic -- "StudentAdmitted<br/>StudentWithdrawn" --> Library
    Academic -- "StudentAdmitted" --> Communication
    Academic -- "StudentAdmitted<br/>StudentWithdrawn" --> Facilities
    Assessment -- "ResultPublished" --> Communication
    Assessment -- "ExamScheduled" --> Academic
    Attendance -- "AbsenceRecorded" --> Communication
    Finance -- "PaymentCollected" --> Communication
    Finance -- "PayrollPaid" --> Hr
    Hr -- "StaffOnboarded<br/>StaffWithdrawn" --> Library
    Hr -- "StaffOnboarded" --> Facilities

    classDef domain fill:#e1f5ff,stroke:#01579b,stroke-width:2px
    class Academic,Assessment,Attendance,Finance,Hr,Library,Facilities,Communication domain
```

## 4. Domain Dependency Hierarchy

```mermaid
graph TB
    L0["Layer 0: Foundation<br/>platform"]
    L1["Layer 1: Cross-cutting<br/>rbac, settings, events, audit"]
    L2["Layer 2: Foundational<br/>academic"]
    L3["Layer 3: Operational<br/>assessment, attendance,<br/>finance, hr, library,<br/>facilities, communication,<br/>documents, cms, operations"]

    L0 --> L1
    L1 --> L2
    L2 --> L3

    style L0 fill:#fff3e0,stroke:#e65100,stroke-width:3px
    style L1 fill:#e8f5e9,stroke:#1b5e20,stroke-width:2px
    style L2 fill:#e1f5ff,stroke:#01579b,stroke-width:2px
    style L3 fill:#f3e5f5,stroke:#4a148c,stroke-width:2px
```

## 5. Domain Responsibility Matrix

| Domain              | Primary aggregate       | Owns                                                          |
| ------------------- | ----------------------- | ------------------------------------------------------------- |
| **platform**        | `School`, `User`        | Tenancy, identity, custom fields, lookup data, modules        |
| **rbac**            | `Role`, `Capability`    | Authorization, role catalog, 2FA policy                       |
| **settings**        | `GeneralSettings`       | Per-tenant configuration, theme, language                    |
| **events**          | (port)                  | Event bus, envelope, schema registry, outbox                 |
| **audit**           | (port)                  | Audit log, query, retention, redaction, compliance reports   |
| **academic**        | `Student`, `Class`      | Student lifecycle, classes, sections, subjects, sessions     |
| **assessment**      | `Exam`, `Mark`          | Examinations, marks, results, report cards                   |
| **attendance**      | `AttendanceSession`     | Daily attendance for students and staff                      |
| **finance**         | `Invoice`, `Payment`    | Fees, payments, banking, expenses, payroll, wallet          |
| **hr**              | `Staff`, `Payroll`      | Staff lifecycle, leave, attendance, designations             |
| **library**         | `Book`, `Member`        | Books, members, issues, returns                              |
| **facilities**      | `TransportRoute`, `Room`| Transport, dormitory, inventory                              |
| **communication**   | `Notice`, `Complaint`   | Notices, complaints, chat, notifications                     |
| **documents**       | `Document`, `Dispatch`  | Forms, postal dispatch / receive                             |
| **cms**             | `Page`, `News`          | Public website content                                       |
| **operations**      | `Backup`, `Job`         | Backups, jobs, system versions, audit projections            |

## 6. Domain Enable / Disable

```mermaid
graph TB
    Engine["smscore::Engine"]
    Engine --> P1[platform + rbac + events + audit]
    Engine --> P2[academic]
    Engine -.optional.-> P3[assessment]
    Engine -.optional.-> P4[attendance]
    Engine -.optional.-> P5[finance]
    Engine -.optional.-> P6[hr]
    Engine -.optional.-> P7[library]
    Engine -.optional.-> P8[facilities]
    Engine -.optional.-> P9[communication]
    Engine -.optional.-> P10[documents]
    Engine -.optional.-> P11[cms]
    Engine -.optional.-> P12[operations]
    Engine -.optional.-> P13[settings]

    style P1 fill:#fff3e0,stroke:#e65100,stroke-width:3px
    style P2 fill:#e8f5e9,stroke:#1b5e20,stroke-width:2px
    style P3,P4,P5,P6,P7,P8,P9,P10,P11,P12,P13 fill:#f5f5f5,stroke:#9e9e9e
```

The four foundation crates (`platform`, `rbac`, `events`, `audit`)
plus `academic` are **mandatory** for every deployment. The
remaining domains are optional; the consumer enables them based
on the deployment's scope. A consumer building a small admin
tool may enable only `academic` and `rbac`. A consumer building
a full SaaS school platform enables all of them.
