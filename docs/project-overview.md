# Educore Project Overview

## Vision

Educore is a reusable, embeddable **school-domain engine** for production software.
It captures the business behavior, workflows, and rules required to operate a real
school — admissions, attendance, examinations, finance, payroll, communication,
facilities, library, transport, and compliance — and exposes them as a
command-oriented, event-driven kernel that any application can drive.

## Mission

Transform the operational knowledge of real schools into a Rust domain engine that:

- Is **portable** across storage engines, runtimes, and consumer surfaces.
- Is **type-safe** end-to-end so consumer bugs fail at compile time.
- Is **AI-agent friendly** so non-human actors can drive the school safely.
- Is **multi-tenant by default** so a single binary can serve many schools.
- Is **audit-first** so every state change leaves a durable trace.
- Is **offline-capable** so field deployments never block on connectivity.

## What Educore Is

- A **domain engine** (the school's business kernel, isolated from any application shell).
- A **business platform crate** (a Cargo workspace consumable by CLI, Tauri, Web, mobile).
- A **DDD framework** (Aggregates, Entities, Value Objects, Domain Services, Policies).
- A **hexagonal architecture framework** (Domain at the center, Ports out, Adapters in).
- An **event-driven school domain kernel** (Commands in, Events out).
- An **AI-agent execution layer** (Capability catalog with capability-based permissions).

## What Educore Is Not

- Not a Laravel port.
- Not a PHP re-implementation.
- Not a database engine.
- Not a UI framework.
- Not a "school app". The product is a kernel, not a screen.
- Not coupled to any storage backend, notification provider, payment gateway, or
  authentication system. Those are ports, never built-ins.

## Target Consumers

| Consumer                | Description                                                                 |
| ----------------------- | --------------------------------------------------------------------------- |
| CLI tools               | Operator scripts, automation pipelines, headless deployments                |
| Desktop applications    | Native school administration tools                                          |
| Tauri applications      | Cross-platform hybrid shells wrapping Educore                               |
| Mobile applications     | Parent, student, and teacher apps                                            |
| Web APIs                | SaaS-style REST / GraphQL surfaces backed by Educore                        |
| SaaS platforms          | Hosted school platforms serving many tenants from one engine                |
| AI agents               | Tool-using LLMs invoking Educore capabilities to assist school staff        |
| Automation systems      | Scheduled jobs, reconciliation, billing, reporting engines                  |

## Core Philosophy

- **Domain first.** Business invariants are encoded as types and rules, not as
  application code.
- **Capabilities, not roles.** Permissions are named capabilities, never string-typed
  role names.
- **Explicit over implicit.** No service locators, no DI containers, no reflection,
  no runtime metadata.
- **Compile-time safety.** Fields, operators, and queries are typed.
- **Events describe the past.** Commands propose; events record what happened.
- **Audit by default.** Every state change writes a durable audit record.
- **Multi-tenant isolation is structural.** Tenant identity is part of every aggregate.
- **Offline is a first-class mode.** State changes can be queued and reconciled.

## Non-Goals

- Migrating Schoolify, Laravel, Eloquent, or any PHP code.
- Producing a UI. Consumer applications provide their own presentation.
- Implementing a single storage engine as canonical. Storage is a port.
- Implementing authentication. Authentication is a port.
- Implementing notification delivery. Delivery is a port.
- Implementing payment processing. Processing is a port.
- Generating a single binary. Consumers compose crates.

## Success Criteria

Educore is successful when:

1. A consumer application can admit a student, take attendance, record marks, and
   collect fees using only the public API and this documentation.
2. Replacing storage from SQLite to PostgreSQL requires no domain code change.
3. A capability-based AI agent can safely call any documented command without
   bypassing business rules.
4. Auditors can reconstruct any historical state change from durable records.
5. Real schools can be operated end-to-end: real students, real examinations,
   real money, real attendance, real reports, real compliance.

## Engine Boundaries

The engine owns:

- The school domain model (aggregates, entities, value objects).
- The command catalog (every business action exposed as a typed command).
- The event catalog (every state change published as a typed event).
- Business rules, validation, lifecycle transitions.
- Permission enforcement.
- Multi-tenant isolation.
- Audit record production.
- Offline synchronization metadata.
- Domain-specific query expressions.

The engine explicitly excludes:

- HTTP servers, RPC servers, gRPC servers.
- Database drivers, connection pools, schema migration tools.
- Authentication drivers (OAuth, SAML, JWT, password hashing).
- Notification transports (SMTP, Twilio, FCM, APNs).
- Payment processors (Stripe, PayPal, Square).
- File storage backends (S3, GCS, MinIO).
- UI templates, JS frameworks, CSS, theming.
- Background job runners (Cron, Sidekiq, BullMQ).
- Reporting renderers (PDF, XLSX).

Each excluded concern is a port that the consumer provides.

## Consumers Provide

- A storage adapter implementing the storage port.
- An authentication provider implementing the auth port.
- A notification provider implementing the notification port.
- A payment provider implementing the payment port.
- A file storage provider implementing the file storage port.
- An event bus implementation if the consumer wants distributed messaging.
- Any external integration (LMS, video conferencing, SMS gateway, etc.) via the
  integration port.

## Engine Style

Educore should feel like a modern, well-engineered Rust crate. It is **not a
framework**. It exposes:

- Traits for ports.
- Builders where useful.
- Derive macros for value objects and identifiers.
- Strongly typed identifiers that cannot be confused across aggregates.
- Strongly typed value objects that validate at construction.
- Explicit error handling — no panics in production paths.
- Async-friendly APIs where I/O happens.
- A query builder that is compile-time checked and storage-agnostic.

## Required Reading Order

1. `docs/project-overview.md` (this document)
2. `docs/architecture.md` (the system map)
3. `docs/build-plan.md` (the implementation roadmap)
4. `docs/code-standards.md` (the engineering rules)
5. `docs/specs/<domain>/overview.md` for each domain in use
6. `docs/ports/*.md` for each port the consumer must implement
7. `docs/commands/*.md` for the command catalog
8. `docs/events/*.md` for the event catalog
9. `docs/schemas/*.md` for cross-cutting schemas
10. `docs/decisions/*.md` for architectural decisions
11. `docs/diagrams/*.md` for visual maps
12. `docs/research/*.md` for the business knowledge source
