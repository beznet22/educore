# SMSengine Autonomous Domain Discovery, Architecture Analysis & Documentation Generation Mission

## Mission

You are operating inside the SMSengine repository.

Your objective is to transform the database schema, business knowledge, and domain behavior contained within this repository into a complete, production-grade documentation system that enables developers and AI agents to implement SMSengine without requiring access to the Schoolify project.

This is a domain discovery and specification generation mission.

You are NOT:

* porting Laravel
* rewriting PHP
* migrating Schoolify
* cloning application architecture
* documenting source code

You ARE:

* discovering business knowledge
* discovering workflows
* discovering domain behavior
* discovering business rules
* discovering permissions
* discovering reporting requirements
* discovering lifecycle transitions
* generating implementation-grade specifications

The resulting documentation must become the authoritative source of truth for SMSengine.

---

# Required Reading

Before performing any work, read:

```text
docs_guidlines/system.md
docs_guidlines/query_layer.md
docs_guidlines/execution_guidlines.md
```

These documents are authoritative.

Priority order:

1. docs_guidlines/system.md
2. docs_guidlines/query_layer.md
3. docs_guidlines/execution_guidlines.md

If a conflict exists:

* Prefer simpler architecture.
* Prefer maintainability.
* Prefer production Rust practices.
* Avoid unnecessary abstraction.

---

# What Is SMSengine?

SMSengine is NOT an application.

SMSengine is:

* a reusable school-domain engine
* a business platform crate
* a domain kernel
* a command execution engine
* an event-driven school platform
* an AI-agent execution layer
* an embeddable backend

Consumer applications may include:

* CLI
* Desktop
* Tauri
* Mobile
* Web APIs
* SaaS Platforms
* AI Agents
* Automation Systems

Consumer applications should provide:

* Storage implementation
* Authentication implementation
* Notification implementation
* File storage implementation
* Payment implementation
* External integrations

Everything else belongs inside SMSengine.

Business rules must never leak into consumer applications.

Bad:

```rust
db.students.insert(...)
```

Bad:

```sql
INSERT INTO students (...)
```

Preferred:

```rust
engine
    .students()
    .create_student(command)
    .await?;
```

---

# Knowledge Sources

## Database Truth

Read:

```text
migrations/
```

Migration files are the authoritative source of:

* entities
* relationships
* identifiers
* domain boundaries
* aggregate candidates

---

## Business Knowledge Source

Read:

```text
schoolify/
```

Schoolify exists solely as a domain knowledge source.

Schoolify may be used to discover:

* workflows
* validation behavior
* permissions
* reporting requirements
* business rules
* operational procedures
* lifecycle transitions
* real-world edge cases

Schoolify must NOT influence:

* architecture
* crate layout
* implementation design
* Rust APIs
* naming conventions
* framework decisions

---

# Parallel Multi-Agent Execution Strategy

Do NOT use a single-agent workflow.

Spawn specialized subagents.

Subagents must run in parallel.

Process work in batches.

---

# Batch 1 — Repository Analysis

Run in parallel.

### Documentation Coordinator

Responsibilities:

* read all documentation standards
* create work plan
* assign tasks
* track coverage
* merge findings

### Migration Analysis Agents

One per migration domain.

Examples:

* Academic Agent
* Assessment Agent
* Attendance Agent
* Finance Agent
* HR Agent
* Library Agent
* Platform Agent
* RBAC Agent

Responsibilities:

* inspect schema
* identify aggregates
* identify entities
* identify relationships
* identify boundaries

---

# Batch 2 — Schoolify Discovery

Run in parallel.

Create one discovery agent per domain.

Responsibilities:

* discover workflows
* discover permissions
* discover reporting logic
* discover validation rules
* discover lifecycle transitions
* discover edge cases

Extract business meaning only.

Never extract implementation details.

Never document:

* controllers
* services
* repositories
* function names
* PHP code
* Laravel concepts
* file paths

Document business behavior only.

---

# Batch 3 — Domain Modeling

Run in parallel.

Responsibilities:

Transform discoveries into:

* Aggregates
* Entities
* Value Objects
* Domain Services
* Commands
* Events
* Repositories
* Permissions

Following DDD principles.

---

# Batch 4 — Architecture Validation

Run in parallel.

Responsibilities:

Validate architecture against:

* docs_guidlines/system.md
* docs_guidlines/query_layer.md
* docs_guidlines/execution_guidlines.md

Reject:

* framework-style architecture
* enterprise cargo cult patterns
* unnecessary abstraction
* speculative scalability

Prefer:

* simplicity
* correctness
* maintainability
* explicitness

SMSengine is a domain engine.

Not an industrial-scale distributed platform.

---

# Batch 5 — Documentation Generation

Generate:

```text
docs/specs/*
docs/ports/*
docs/events/*
docs/commands/*
docs/schemas/*
docs/decisions/*
docs/diagrams/*
docs/research/*
```

Every document must be implementation-grade.

---

# Documentation Requirements

The final documentation must allow:

* human developers
* AI coding agents

to implement the domain without opening Schoolify.

If Schoolify is still required:

The documentation is incomplete.

---

# Strict Prohibitions

Never include:

* PHP code
* Laravel code
* Eloquent code
* controller names
* service names
* repository names
* function names
* class names
* file paths
* source references

Forbidden:

"StudentController::store creates students"

Allowed:

"Student admission creates a student record and associates it with an academic session."

Document behavior.

Never document implementation.

---

# Domain Specification Requirements

Each domain specification must contain:

## Overview

Purpose.

Responsibilities.

Boundaries.

Dependencies.

---

## Aggregates

Aggregate roots.

Invariants.

Consistency boundaries.

---

## Entities

Relationships.

Lifecycle.

Rules.

---

## Value Objects

Identifiers.

Validation requirements.

Domain-specific types.

---

## Commands

Examples:

* CreateStudent
* PromoteStudent
* PublishResult
* RecordAttendance
* GenerateInvoice

Commands represent executable domain actions.

---

## Events

Examples:

* StudentCreated
* StudentPromoted
* AttendanceMarked
* InvoicePaid
* ResultPublished

Events represent completed actions.

---

## Business Rules

Validation rules.

State transitions.

Constraints.

Edge cases.

Approvals.

Exceptions.

---

## Workflows

Implementation-grade workflows.

Examples:

* Admission
* Promotion
* Fee Collection
* Attendance
* Examinations
* Recruitment

---

## Permissions

Capability-based.

Never role strings.

Examples:

```text
Student.Create
Student.Update
Student.Transfer
Finance.Invoice.Generate
Finance.Payment.Record
```

---

## Reports

Required reports.

Expected outputs.

Business requirements.

---

## Audit Requirements

Required audit trails.

Immutable records.

Traceability requirements.

---

## Multi-Tenancy Requirements

school_id requirements.

Tenant isolation rules.

Cross-tenant restrictions.

---

## Offline Synchronization Requirements

Versioning.

Conflict resolution.

Synchronization expectations.

---

# Query Layer Requirements

Follow:

```text
docs_guidlines/query_layer.md
```

The query layer is mandatory.

Provide an Eloquent-like developer experience while remaining fully type-safe and idiomatic Rust.

The query layer is NOT:

* an ORM
* Active Record
* reflection system
* schema parser

The query layer IS:

* compile-time safe
* storage-agnostic
* repository-oriented
* high-performance
* Rust-native

Document:

* query capabilities
* filtering
* sorting
* pagination
* aggregation
* repository responsibilities
* domain-specific optimized queries

Prefer:

```rust
.where_eq(StudentField::Status, StudentStatus::Active)
```

Avoid:

```rust
.where("status", "active")
```

Repositories may define optimized domain-specific queries.

Document them.

---

# Architecture Principles

Follow:

* Domain Driven Design
* Hexagonal Architecture
* Event-Driven Design
* Command-Oriented Execution
* Multi-Tenant by Default
* Audit-First Design
* Offline-Capable Design
* AI-Agent Friendly Design

Do not over-engineer.

Avoid enterprise architecture for its own sake.

Choose the simplest architecture that satisfies production requirements.

---

# Engineering Standards

Follow:

```text
docs_guidlines/execution_guidlines.md
```

These standards are mandatory.

This includes:

* Cargo usage
* Workspace conventions
* Testing standards
* Type safety requirements
* Error handling requirements
* Cross-compilation requirements
* Commit attribution requirements

All generated architecture must naturally support those standards.

---

# Rust Architecture Expectations

SMSengine should feel like a modern Rust crate.

Prefer:

* traits for ports
* builders where useful
* derive macros for ergonomics
* strongly typed identifiers
* strongly typed value objects
* explicit error handling

Avoid:

* service locators
* dependency injection containers
* reflection systems
* runtime metadata registries
* excessive generic abstractions
* framework magic

---

# Production Engineering Goal

SMSengine is intended to manage:

* students
* staff
* attendance
* examinations
* report cards
* payments
* invoices
* communication
* facilities
* compliance records

Design for real schools.

Not toy projects.

Not tutorials.

Not hypothetical systems.

---

# Final Validation Checklist

Before work is considered complete, verify:

1. Can an experienced Rust engineer implement the domain from the documentation alone?
2. Can an AI coding agent implement the domain from the documentation alone?
3. Can Schoolify be removed without losing business knowledge?
4. Does the architecture remain simple and maintainable?
5. Does the design follow docs_guidlines/system.md?
6. Does the design follow docs_guidlines/query_layer.md?
7. Does the design follow docs_guidlines/execution_guidlines.md?
8. Is the architecture production-ready?
9. Is the architecture idiomatic Rust?
10. Is the architecture free from unnecessary enterprise complexity?

If any answer is "No", continue researching, refining, and expanding the documentation.

The final deliverable is a complete documentation operating system that becomes the authoritative source of truth for the SMSengine domain engine.
