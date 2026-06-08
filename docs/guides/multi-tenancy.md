# Multi-Tenancy Guide

## Goal

Every command, query, and event is bound to a school. Cross-school
operations are explicit and capability-gated.

## SchoolId

`SchoolId` is a typed `Uuid` (or a typed integer depending on the
adapter) carried on every aggregate root. It is created when a school
is provisioned and is immutable for the school's lifetime.

```rust
let school_id = SchoolId::new(Uuid::new_v4());
```

## TenantContext

A `TenantContext` is the runtime scope for a request:

```rust
pub struct TenantContext {
    pub school_id: SchoolId,
    pub user_id: UserId,
    pub session_id: SessionId,
    pub correlation_id: CorrelationId,
    pub clock: Arc<dyn Clock>,
}
```

Every command carries a `TenantContext`. The engine refuses to
execute a command that targets an aggregate whose `SchoolId` differs
from `tenant.school_id`.

## Storage Enforcement

The storage adapter is responsible for filtering by `school_id`. The
engine's typed query layer always includes the school id in the
generated SQL. The adapter should also enforce row-level security
defense in depth.

```sql
ALTER TABLE students ENABLE ROW LEVEL SECURITY;
CREATE POLICY students_school_isolation ON students
    USING (school_id = current_setting('app.current_school_id')::int);
```

The consumer sets `app.current_school_id` per connection from
`tenant.school_id`.

## Cross-Tenant Operations

Some operations span tenants (e.g. district-wide reports, school
transfers). The engine models these as **explicit** commands:

- `TransferStudentCommand { source_school_id, destination_school_id, ... }`
- `DistrictReportQuery { district_id, ... }`

These commands require elevated capabilities (e.g. `Student.Transfer`,
`Report.District`) and are typically restricted to super-admin users.

The engine emits cross-tenant events with both source and destination
school ids:

```rust
pub struct StudentTransferred {
    pub student_id: StudentId,
    pub source_school_id: SchoolId,
    pub destination_school_id: SchoolId,
    ...
}
```

Subscribers handle each side independently.

## Tenant Onboarding

A new school is created via `CreateSchoolCommand`. The platform
domain emits `SchoolCreated`. Each domain that needs school-scoped
data subscribes to `SchoolCreated` and provisions its initial state
(e.g. default settings, sample class names, etc.).

## Tenant Deletion

Tenant deletion is a destructive operation. The engine does not
provide a soft-delete or "archive" tenant command in v1. Consumers
who need it implement it as a privileged command that:

1. Issues `WithdrawStudent` for all active students.
2. Closes all open invoices, payroll runs, and library issues.
3. Emits `SchoolDeleted`.
4. Deletes all school-scoped data.

The engine refuses to delete a school that has outstanding balances
or active enrollments unless explicitly overridden.

## Tenant Configuration

Per-tenant configuration lives in the settings domain. The engine
loads it on command dispatch and passes it to services. Configuration
is not part of `TenantContext`; it is queried separately.

## Testing Multi-Tenancy

The engine includes a test suite that:

- Creates two schools with similar data.
- Verifies that commands in one school cannot read or write the
  other's data.
- Verifies that cross-tenant events are visible only to authorized
  consumers.
