# Documentation Operating System

This is the documentation operating system for the SMSengine school management system. It is a collection of markdown files organized in a hierarchical structure.

**SMSengine is not a school application**.
It is a:

* Domain Engine
* Business Platform Crate
* DDD Framework
* Hexagonal Architecture Framework
* Event-Driven School Domain Kernel
* AI-Agent Execution Engine

The biggest mistake would be organizing docs around screens, pages, or UI features. Instead, organize around **domains, commands, aggregates, services, ports, events, and adapters**.

---

# Recommended Repository Structure

```text
SMSengine/
│
├── docs/
│
├── migrations/
│   ├── 0001_academic.sql
│   ├── 0002_assessment.sql
│   ├── ...
│   └── 0015_settings.sql
│
├── schoolify/
│   └── (Laravel Knowledge Base)
│
└── crates/
    └── smsengine/
```

---

# Documentation Architecture

```text
AGENTS.md
README.md
docs/
│
├── project-overview.md
├── architecture.md
├── build-plan.md
├── code-standards.md
├── library-docs.md
├── progress-tracker.md
│
├── specs/
│
├── domains/
│
├── ports/
│
├── commands/
│
├── events/
│
├── schemas/
│
├── decisions/
│
├── diagrams/
│
├── guides/
│
└── research/
```

---

# Top-Level Documents

---

## project-overview.md

Business vision.

Answers:

```text
What is SMSengine?
Why does it exist?
Who consumes it?
```

Contents:

```text
Vision
Mission
Goals
Target Consumers
Core Philosophy
Non-Goals
Success Criteria
```

---

## architecture.md

High-level architecture only.

Never put implementation details here.

Contents:

```text
System Architecture

DDD Model

Hexagonal Architecture

Domain Boundaries

Port Architecture

Event Architecture

Multi-Tenancy

Command Layer

Storage Strategy

Reference Documents
```

This document becomes the map.

Everything else is referenced from here.

---

## build-plan.md

Roadmap.

```text
Foundation
Domain Layer
Ports
Storage Adapters
Event Bus
AI Layer
SDK Layer
Production
```

---

## code-standards.md

Engineering rules.

```text
Rust Standards

DDD Rules

Hexagonal Rules

Module Rules

Testing Rules

Documentation Rules
```

---

## library-docs.md

Consumer-facing SDK docs.

Example:

```rust
let engine = SchoolEngine::builder()
    .storage(postgres)
    .auth(auth)
    .build();
```

---

## progress-tracker.md

Implementation status.

---

# New: Research Layer

Because you have Schoolify.

```text
research/
│
├── schoolify-analysis.md
│
├── academic-analysis.md
├── assessment-analysis.md
├── attendance-analysis.md
├── finance-analysis.md
├── hr-analysis.md
└── ...
```

Purpose:

Document business logic extracted from Laravel.

Example:

```text
How Schoolify admits students

How promotions work

How report cards work

How fee collection works
```

This prevents repeatedly reading Laravel code.

---

# Domain Specifications

This becomes the most important folder.

```text
specs/
│
├── academic/
├── assessment/
├── attendance/
├── cms/
├── communication/
├── documents/
├── events/
├── facilities/
├── finance/
├── hr/
├── library/
├── operations/
├── platform/
├── rbac/
└── settings/
```

---

# Example Domain Specification

```text
specs/academic/
```

Contains:

```text
overview.md
aggregates.md
entities.md
value-objects.md
commands.md
events.md
permissions.md
workflows.md
services.md
repositories.md
tables.md
```

---

## overview.md

Purpose:

```text
What is Academic Domain?
Responsibilities
Boundaries
Dependencies
```

---

## aggregates.md

DDD aggregate definitions.

Example:

```text
Student

Class

Section

AcademicSession

AcademicTerm
```

---

## entities.md

Domain entities.

Example:

```text
Student

Guardian

Enrollment
```

---

## value-objects.md

Example:

```text
StudentId

AdmissionNumber

EmailAddress

PhoneNumber
```

---

## commands.md

Critical for AI-native architecture.

Example:

```rust
CreateStudentCommand

TransferStudentCommand

PromoteStudentCommand

WithdrawStudentCommand
```

---

## events.md

Example:

```rust
StudentCreated

StudentPromoted

StudentTransferred

StudentWithdrawn
```

---

## permissions.md

Example:

```rust
AcademicPermission::CreateStudent

AcademicPermission::TransferStudent

AcademicPermission::PromoteStudent
```

---

## services.md

Domain services.

Example:

```text
AdmissionService

PromotionService

EnrollmentService
```

---

## repositories.md

Port definitions.

Example:

```rust
StudentRepository

EnrollmentRepository
```

---

## tables.md

Mapping to SQL schema.

Example:

```text
students

student_guardians

student_enrollments
```

---

# Ports Layer

Critical for Hexagonal Architecture.

```text
ports/
│
├── storage.md
├── authentication.md
├── notifications.md
├── payments.md
├── file-storage.md
├── event-bus.md
└── integrations.md
```

---

## storage.md

Defines:

```rust
StudentRepository

AttendanceRepository

ResultRepository
```

---

## authentication.md

Defines:

```rust
AuthProvider
```

---

## notifications.md

Defines:

```rust
NotificationProvider
```

---

## payments.md

Defines:

```rust
PaymentGateway
```

---

## file-storage.md

Defines:

```rust
FileStorage
```

---

## event-bus.md

Defines:

```rust
EventBus
```

---

# Commands Layer

AI agent execution layer.

```text
commands/
│
├── academic.md
├── attendance.md
├── assessment.md
├── finance.md
├── hr.md
└── ...
```

Example:

```rust
CreateStudentCommand

PublishResultCommand

MarkAttendanceCommand

PayInvoiceCommand
```

This becomes the capability catalog.

---

# Events Layer

```text
events/
│
├── academic.md
├── attendance.md
├── finance.md
├── hr.md
└── ...
```

Example:

```rust
StudentCreated

AttendanceMarked

InvoicePaid

ResultPublished
```

---

# Schemas Layer

Global rules.

```text
schemas/
│
├── database-schema.md
├── event-schema.md
├── permission-schema.md
├── command-schema.md
├── tenancy-schema.md
└── audit-schema.md
```

---

## tenancy-schema.md

Defines:

```rust
SchoolId
```

Requirements:

```text
Every aggregate root
must contain school_id
```

---

## audit-schema.md

Defines:

```rust
AuditLog
```

Structure:

```rust
actor_id
resource
action
timestamp
metadata
```

---

# Decisions Layer

ADR records.

```text
decisions/
│
├── ADR-001-DDD.md
├── ADR-002-Hexagonal.md
├── ADR-003-MultiTenancy.md
├── ADR-004-Commands.md
├── ADR-005-Events.md
└── ...
```

---

# Diagrams Layer

```text
diagrams/
│
├── domain-map.md
├── aggregate-map.md
├── event-flow.md
├── command-flow.md
├── dependency-map.md
├── permission-map.md
└── deployment-map.md
```

---

# What Architecture.md Should Reference

```text
architecture.md

├── specs/*
├── ports/*
├── commands/*
├── events/*
├── schemas/*
├── decisions/*
├── diagrams/*
└── research/*
```


## AGENTS & DEVELOPER README

Should be Created for AI agents and developers, and must be updated as system evolves:

```text
AGENTS.md
README.md
```

---

# Final Documentation Hierarchy

```text
project-overview.md
        │
        ▼
architecture.md
        │
        ├── specs/
        ├── ports/
        ├── commands/
        ├── events/
        ├── schemas/
        ├── diagrams/
        ├── decisions/
        └── research/
                │
                ▼
            schoolify/
```

This structure reflects the reality of SMSengine:

**Schoolify** = business knowledge source

**Migrations** = database truth

**Specs** = domain truth

**Architecture** = system map

**Commands** = AI execution surface

**Ports** = integration contracts

**Events** = system communication

**SMSengine** = reusable Rust school-domain engine, not an application.
