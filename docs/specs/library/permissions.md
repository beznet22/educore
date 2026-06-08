# Library Domain — Permissions

Permissions are capability strings. They are not roles. The
RBAC domain maps capabilities to roles.

## Naming

```text
<Domain>.<Aggregate>.<Action>
```

The library domain uses the prefixes `Library.*`, `Book.*`,
`BookCategory.*`, `Member.*`, and `BookIssue.*`.

## Capabilities

### Library (cross-cutting)

- `Library.Read`
- `Library.Configure` — manage `LibrarySettings` in the
  settings domain.
- `Library.Report` — view aggregate reports (overdue, stock,
  fine roll-up).

### BookCategory

- `BookCategory.Create`
- `BookCategory.Update`
- `BookCategory.Delete`
- `BookCategory.Read`

### Book

- `Book.Add`
- `Book.Update`
- `Book.Delete`
- `Book.Read`
- `Book.AdjustQuantity`
- `Book.Search`

### Member

- `Member.Register`
- `Member.Update`
- `Member.Delete`
- `Member.Read`
- `Member.Deactivate`
- `Member.Reactivate`

### BookIssue

- `BookIssue.Issue`
- `BookIssue.Return`
- `BookIssue.Renew`
- `BookIssue.MarkLost`
- `BookIssue.CalculateFine`
- `BookIssue.Read`
- `BookIssue.WaiveFine`

## Default Role Mapping

The platform's default role catalog binds the following for
the library domain:

| Role         | Capabilities (highlights)                                                  |
| ------------ | --------------------------------------------------------------------------- |
| SuperAdmin   | All                                                                         |
| SchoolAdmin  | All within the school                                                       |
| Librarian    | `Book.*`, `BookCategory.*`, `Member.*`, `BookIssue.*`, `Library.*`         |
| Teacher      | `Book.Read`, `Book.Search`, `BookIssue.Read` (own), `Member.Read` (own)    |
| Student      | `Book.Read`, `Book.Search`, `BookIssue.Read` (own), `BookIssue.Renew` (own)|
| Parent       | `Book.Read`, `BookIssue.Read` (linked students)                            |
| Accountant   | `BookIssue.Read`, `Member.Read`                                            |
| Transport    | `Library.Read`                                                              |

The default mapping is a starting point and is configurable
per school.

## Authorization Pattern

Capabilities are checked at the command boundary. The engine
never trusts the caller to assert their own role.

```rust
if !engine.rbac().has(actor_id, Capability::BookIssueIssue).await? {
    return Err(DomainError::forbidden("missing capability"));
}
```

Some commands have a secondary ownership check: a student
renewing a book may only renew their own issues; a parent
viewing issues may only see issues for linked students; a
teacher renewing may only renew their own.

## Read vs Write

Read capabilities are explicit. The engine does not assume
that `Book.Read` implies `Book.Add`. A consumer may grant
read-only access to a parent or auditor.

## Fine Waiver

A fine waiver (`BookIssue.WaiveFine`) is a privileged
capability. It allows an actor to mark a `BookIssueFine` as
`Waived = true` with a reason. The waiver is itself an event
for audit:

```rust
pub struct WaiveBookIssueFineCommand {
    pub tenant: TenantContext,
    pub book_issue_fine_id: BookIssueFineId,
    pub reason: String,
}
```

**Capability:** `BookIssue.WaiveFine`
**Effects:** Emits `FineWaived` and updates the
`BookIssueFine.Waived` flag. Finance is informed and the
receivable is reversed.

## Tenant Isolation

Every capability check is paired with a tenant check. The
actor must be authenticated to the school that owns the
target aggregate. There is no cross-tenant capability
elevation. Cross-school library networks (e.g. inter-library
loans) are special-cased and require a per-tenant grant from
both schools; v1 does not support them.

## Audit Requirements

Every command in the library domain writes a durable audit
record referencing the originating `actor_id`,
`correlation_id`, and the event(s) it produced. The audit
record retains:

- The originating capability used to authorize the command.
- The pre-image and post-image of the affected aggregate (or a
  diff if the aggregate is large).
- The full event payload(s) for cross-domain replay.

Read commands (`*.Read`) are not audited at the audit sink,
but queries that surface personal data (e.g. listing a
member's issue history) must be logged at `INFO` with the
`actor_id` and `correlation_id` for compliance.
