# ADR-003: Multi-Tenant by Default

## Status

Accepted.

## Context

A school is a self-contained world. Its students, staff, fees,
classes, and audit log are not shared with another school. In a
SaaS deployment, a single engine instance serves many schools
from a single binary; in an on-premise deployment, a single
engine serves a single school from a single database. In both
cases, the engine must keep one school's data invisible to
another.

The naive approach — separate databases per school, separate
processes per school — is operationally heavy. The school domain
is also not large per tenant (a school has thousands of students,
not millions), so a shared database with proper isolation is
usually the right answer.

The failure mode is severe. A bug that leaks student data from
school A to a parent in school B is not a "small bug" — it is a
breach of trust, a regulatory violation, and a PR disaster. The
default must be safe; safety must be structural, not
"remembered by the developer."

## Decision

SMSengine is **multi-tenant by default**, with structural
isolation:

1. **Every aggregate root carries a `SchoolId`** as part of its
   typed identifier (`StudentId(SchoolId, Uuid)`,
   `InvoiceId(SchoolId, Uuid)`, etc.).
2. **The `TenantContext` is required** at every command
   boundary. The engine refuses a command without a
   `TenantContext`.
3. **Storage adapters enforce `school_id` filtering** on every
   read. The engine's query layer carries the active
   `school_id` from the `TenantContext`; the adapter injects it
   into the query. The query is constructed via the
   `#[derive(DomainQuery)]`-generated builder, which is a closed
   type that only accepts a `SchoolId` at construction time. The
   builder cannot be built without a tenant.
4. **Row-level security policies** are mandatory on every
   aggregate table in the default PostgreSQL adapter. The
   policies are configured by the storage adapter; the consumer
   does not opt in.
5. **Cross-tenant operations are explicit and capability-gated**.
   Only `Platform.CrossTenant` plus a small catalog of
   cross-tenant commands (`Platform.School.Transfer`,
   `Platform.School.Suspend`, etc.) can act across schools.
6. **A bootstrap school** holds global lookup data
   (`Country`, `Currency`, `TimeZone`, `Language`) and the
   `SuperAdmin` role. It is invisible to all other schools.
7. **Per-tenant configuration** is the rule. Every configurable
   value (theme, language, currency, payment gateway) is
   anchored to a `SchoolId`.
8. **Single-tenant on-premise** is a deployment mode, not a
   different code path. The same engine, the same `school_id`
   column, the same row-level security policies — just with
   one school in the database.

## Consequences

### Positive

- **Safety is structural.** A developer who writes
  `engine.students().query()` does not need to remember to
  filter by `school_id`; the storage adapter does it
  automatically. The query layer's types require a
  `TenantContext`; the database refuses unfiltered reads.
  The `#[derive(DomainQuery)]` macro makes this even more
  robust: a builder without a `SchoolId` does not exist in the
  type system.
- **No cross-school data leaks.** A bug that exposes one
  school's data to a user in another school would require
  bypassing the type system, the query layer, the storage
  adapter, and the row-level security policy. The defense is
  in depth.
- **SaaS and on-premise share the same code path.** A consumer
  who starts as a single-tenant on-premise installation can
  move to SaaS without changing the engine's domain code.
- **Regulatory compliance is easier.** Each school's data is
  isolated, deletion is scoped, audit is per-tenant, and
  reporting is per-tenant by default.
- **Multi-region deployments are tractable.** A school in the
  EU can be served from an EU database; a school in the US
  from a US database. The engine's tenant model aligns with
  data-residency requirements.

### Negative

- **The `SchoolId` is everywhere.** Every aggregate, every
  identifier, every query, every event carries it. This
  adds a small amount of boilerplate, mitigated by the
  `smscore-core` identifier wrappers, the query layer's
  tenant binding, and the `#[derive(DomainQuery)]` macro
  (whose generated builder requires a `SchoolId` at
  construction time).
- **Global reporting is awkward.** "How many students
  across all schools?" is not a free query; it requires an
  explicit, capability-gated reporting command. We accept
  this cost.
- **Cross-tenant workflows are explicit.** A `TransferStudent`
  command is more elaborate than just moving a row. The
  elaborate command is the cost of safety.
- **Bootstrap school is non-standard.** Some developers
  expect every record to belong to a "real" school. The
  bootstrap school is a concept they must learn.

### Mitigations

- The `smscore-platform` crate provides `SchoolId`,
  `TenantContext`, and the cross-tenant port in a single,
  well-documented module.
- The `smscore-rbac` crate treats `SuperAdmin` as a system
  role; consumers cannot delete it.
- The default PostgreSQL storage adapter configures
  row-level security on every table at install time.
- The CLI scaffold (`smscore new --multi-tenant`) generates
  a starter configuration that wires the policies.

## Alternatives Considered

### 1. Single-tenant only

The engine serves one school per deployment. Rejected
because the operational cost of "one process per school"
is high in SaaS, and because consumers would re-implement
multi-tenancy on top of the engine — inconsistently and
insecurely.

### 2. Database per school

Each school has its own database. Rejected because it
makes cross-school reporting hard, multiplies operational
overhead, and forces the engine to abstract over
"connection per school" without a clear benefit over
shared-database-with-row-security for typical school
sizes.

### 3. Schema per school

Each school has its own database schema in a shared
database. Rejected because it pushes multi-tenancy into
the consumer's migration tooling, and because row-level
security is the more portable approach (works on
SQLite-with-extension, on SurrealDB, etc.).

### 4. No tenant binding in the type system

Tenancy is enforced only at the storage layer. Rejected
because the type system is the cheapest and most reliable
place to encode it.

### 5. Tenant per row, but no per-tenant config

A school's data is isolated, but its configuration is
global. Rejected because real schools have wildly
different needs (different currencies, different
languages, different grading scales). Per-tenant
configuration is a hard requirement.
