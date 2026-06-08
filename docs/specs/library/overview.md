# Library Domain Overview

## Purpose

The library domain owns the catalog of books, the registration of
library members, and the issue, return, renewal, and fine lifecycle
of borrowed books. It captures the school's reading resources and
their circulation, anchored to the academic year and to the people
who may borrow them.

## Responsibilities

- Book category catalog and book master records.
- Library member registration for students and staff.
- Book issue to a member with a due date.
- Book return with optional return notes.
- Book renewal (extension of due date).
- Overdue tracking and fine calculation.
- Stock tracking (total copies, copies on issue, copies available).
- Per-academic-year scoping of all library activity.

## Boundaries

The library domain does **not** own:

- Student identity, attendance, or promotion (see
  `specs/academic/`).
- Staff identity or HR lifecycle (see `specs/hr/`).
- Fee invoicing or fine collection (see `specs/finance/`).
- Notification dispatch (see `specs/communication/`).
- E-book licensing or digital lending (out of scope; the v1
  domain models physical copies only).
- Inter-library loans (out of scope; the v1 domain assumes a
  single school's library).

The library domain **does** depend on identifier types defined by
the academic and human-resource domains: `StudentId`, `StaffId`,
`RoleId`, `AcademicYearId`. It exposes its own identifier types to
consumers: `BookId`, `BookCategoryId`, `LibraryMemberId`,
`BookIssueId`.

## Dependencies

- `smscore-core` — error types, result, identifier trait.
- `smscore-platform` — `SchoolId`, `UserId`, `TenantContext`.
- `smscore-rbac` — capability checks.
- `smscore-events` — domain event publishing.
- `smscore-academic` — `StudentId`, `AcademicYearId`
  (read-only references).
- `smscore-hr` — `StaffId` (read-only references).
- `smscore-rbac` — `RoleId` (read-only reference).
- `smscore-finance` — receives fine events for invoicing and
  payment.

## Domain Invariants

1. Every book, book category, library member, and book issue
   belongs to exactly one school.
2. Every library aggregate is scoped to one `AcademicYear`. A book
   catalog and a member registration are reused across academic
   years; an `Issue` is always per-year.
3. A `Book` has a unique ISBN (or, when ISBN is missing, a unique
   `BookNumber`) within a school.
4. A `Book` is uniquely identified by `(title, author, publisher,
   edition)` within a school when neither ISBN nor book number is
   available.
5. A `BookCategory` is uniquely named within a school.
6. A `LibraryMember` references exactly one of `StudentId` or
   `StaffId` and one `RoleId`.
7. A `LibraryMember` is uniquely identified by `(member_type,
   student_or_staff_id)` within a school-year.
8. A `BookIssue` references exactly one `Book` and one
   `LibraryMember`.
9. The sum of an issue's `Quantity` across all open and
   outstanding book issues for the same book may not exceed
   `Book.Quantity` (the number of physical copies the library
   holds).
10. A `BookIssue` is in one of the statuses `Issued`, `Returned`,
    `Renewed`, `Overdue`, `Lost`. Renewal transitions `Issued` to
    `Renewed`; return transitions any open status to `Returned`;
    overdue is a derived state computed from the due date.
11. A `BookIssue` may be renewed only if it is currently
    `Issued` or `Renewed` and the member has no overdue book.
12. A returned `BookIssue` is immutable.
13. Fines are calculated on `BookReturned` and on overdue
    evaluations; the amount is non-negative and is the
    responsibility of the member who held the book at the
    return moment.

## Aggregate Roots

| Aggregate      | Root Type         | Purpose                                    |
| -------------- | ----------------- | ------------------------------------------ |
| BookCategory   | `BookCategory`    | A grouping of books (e.g. "Fiction")       |
| Book           | `Book`            | A book master record                       |
| LibraryMember  | `LibraryMember`   | A registered borrower                      |
| BookIssue      | `BookIssue`       | An issue of a book to a member             |

Each aggregate is documented in detail under
`docs/specs/library/aggregates.md`.

## Cross-Domain Impact

When a `Student` is admitted, the library domain may subscribe to
`StudentAdmitted` and auto-create a `LibraryMember` with
`MemberType = Student`. The subscriber is idempotent: a member
already exists for the student is not re-created.

When a `Student` is withdrawn, the library domain subscribes to
`StudentWithdrawn` and flags any open `BookIssue` rows for
return. The member is not deleted; their history is retained.

When a `BookIssue` is returned late, the library domain emits
`FineCalculated` and the finance domain posts the fine as a
receivable against the member.

## Consumers

- Web admin UI (manage catalog, members, issues).
- Librarian desk app (issue, return, renew, search).
- Mobile student app (search the catalog, view active issues,
  request renewals).
- Mobile teacher app (search the catalog, view own issues).
- Mobile parent app (view linked student's issues and fines).
- AI agent (catalog queries, overdue reports, fine
  calculation).

## Anti-Goals

- The library domain does not render barcodes, scan ISBNs, or
  integrate with RFID hardware. Scanning is a port concern.
- The library domain does not index full-text content. Search is
  metadata-only at the engine layer; a search port may add full
  text at the adapter level.
- The library domain does not handle reservations or hold queues.
  The v1 domain models the issue lifecycle only.
- The library domain does not publish reading statistics or
  recommend books. That is a future analytics extension.
- The library domain does not auto-charge fines to the member's
  fee ledger. Finance subscribes to `FineCalculated` and decides
  the billing policy.
