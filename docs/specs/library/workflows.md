# Library Domain — Workflows

Workflows orchestrate commands, queries, and policies to fulfill
a business goal. They are documented as ordered, conditional
steps.

## Book Cataloging Workflow

```text
1. Librarian defines book categories (CreateBookCategory).
2. Librarian adds a book (AddBook) with metadata, ISBN or
   book number, category, and total copies.
3. The book becomes part of the searchable catalog.
4. The librarian may adjust quantity (AdjustBookQuantity) for
   acquisitions or write-offs.
5. The librarian may update bibliographic metadata
   (UpdateBook).
6. The book is removed only when no historical issue
   references it (DeleteBook); the catalog history is
   preserved by the event log.
```

**Pre-conditions:**
- A category must exist before books can be added under it.
- The book number or ISBN is unique within the school.

**Failure paths:**
- Duplicate ISBN → `ValidationError::UniqueViolation`.
- Quantity reduced below open issues → `ConflictError::InUse`.

## Member Registration Workflow

```text
1. Student or staff member is admitted/hired (handled by
   academic and HR domains).
2. The library subscribes to StudentAdmitted (and the HR
   equivalent for staff) and auto-creates a LibraryMember
   (RegisterLibraryMember) for the new member in the current
   academic year. Auto-creation is idempotent.
3. Librarian may register a member manually for cases where
   the auto-create flow did not run (e.g. mid-year transfer).
4. Librarian may update the member (UpdateLibraryMember) for
   non-critical fields.
5. When a member leaves the school, the member is deactivated
   (DeactivateLibraryMember). New issues are blocked; open
   issues are surfaced for return.
6. When all open issues are closed, the member may be deleted
   (DeleteLibraryMember) to remove stale records.
```

**Edge cases:**
- A member is re-admitted after withdrawal → a new
  `LibraryMember` is created for the new year; the old
  member's history is preserved.
- A staff member changes role → the member remains active;
  the new role is recorded on the next update command.

## Book Issue Workflow

```text
1. Member visits the library and requests a book.
2. Librarian searches the catalog (SearchBooks).
3. Librarian issues the book (IssueBook) with a quantity, a
   given date, and a due date.
4. The system validates stock, member eligibility, and
   per-member cap.
5. The system decrements the available stock and emits
   BookIssued.
6. Communication sends a notification to the member with the
   due date.
7. Member returns the book (ReturnBook) on or before the
   due date. The system marks the issue Returned, restores
   stock, and emits BookReturned. If the return is late, a
   fine is calculated (CalculateFine) and FineCalculated is
   emitted; finance posts the receivable.
8. If the book is lost, the librarian marks the issue Lost
   (MarkBookLost). Stock is decremented to reflect the loss.
   A replacement-cost fine may be calculated.
9. If the book needs more time, the member or librarian
   renews (RenewBook) before the due date. The system extends
   the due date and emits BookRenewed.
```

**Pre-conditions:**
- The book is in the catalog with stock available.
- The member is active and not at the per-member cap.
- The school-configured `MaxBooksPerMember` is respected.

**Failure paths:**
- Insufficient stock → `ConflictError::OutOfStock`.
- Member at cap → `ConflictError::MemberCapExceeded`.
- Inactive member → `ForbiddenError::MemberInactive`.

## Book Renewal Workflow

```text
1. The member or librarian initiates a renewal (RenewBook)
   with a new due date.
2. The system validates:
   - The issue is currently Issued or Renewed.
   - The member has no overdue issues.
   - The new due date is strictly after the current due
     date.
   - The renewal count is below the school-configured
     `MaxRenewalsPerIssue`.
3. The system appends a BookIssueRenewal history row and
   emits BookRenewed.
4. Communication sends a notification with the new due date.
```

**Edge cases:**
- A renewal that would push the due date past the academic
  year end is allowed but flagged in the audit log.
- A renewal while the member has an overdue issue is
  rejected; the overdue must be resolved first.

## Return & Fine Workflow

```text
1. Member returns the book (ReturnBook).
2. The system marks the issue as Returned and restores stock.
3. The system computes days_overdue = max(0, return_date -
   due_date).
4. The system computes the fine = days_overdue * fine_per_day
   (from school configuration).
5. If the fine is positive, the system emits FineCalculated
   and finance posts the receivable.
6. The fine may be waived (WaiveBookIssueFine) by an actor
   with the BookIssue.WaiveFine capability. Finance is
   informed and the receivable is reversed.
```

**Edge cases:**
- A return exactly on the due date produces zero days
  overdue and zero fine.
- A return before the due date is allowed and produces no
  fine.
- A return after multiple renewals is evaluated against the
  final (current) due date.

## Overdue Reporting Workflow

```text
1. A scheduled job runs daily (or on demand by the
   librarian) to evaluate open issues.
2. For each open issue with `due_date < as_of`, the system
   surfaces it as overdue.
3. The system produces the OverdueIssuesReport.
4. Communication sends a reminder to the member (and, for
   students, to the linked guardian).
5. A grace period (configured per school) elapses; if the
   book is still not returned, a final notice is sent.
6. If the book remains unreturned past the school's hard
   cutoff, the librarian may mark the book lost
   (MarkBookLost) and a replacement-cost fine is calculated.
```

## Reports

The library domain exposes read models and reports:

- `BookCatalogReport` — books with stock-on-hand, copies on
  issue, copies available.
- `OverdueIssuesReport` — open issues past their due date,
  per member.
- `MemberIssueReport` — open and historical issues for a
  member, including fines.
- `MemberFineReport` — outstanding and paid fines per
  member.
- `CategoryStockReport` — total stock and copies on issue
  per category.
- `IssueActivityReport` — issues and returns in a date
  range, by book or by member.
- `LostBooksReport` — historical losses with replacement
  cost and outstanding receivable.

Reports are read-only and do not mutate state. They are
produced either synchronously through the query layer or
asynchronously as materialized views rebuilt from the event
log.

## Idempotency

- `AddBook` is idempotent on `isbn_no` or `book_number`
  within a school. A duplicate returns the existing book.
- `RegisterLibraryMember` is idempotent on `(member_type,
  member, academic_year_id)`. A duplicate returns the
  existing member.
- `IssueBook` is idempotent on
  `(book_id, library_member_id, given_date, due_date)` for
  the same quantity. A duplicate returns the prior issue.
- `ReturnBook` is idempotent on `book_issue_id`. A second
  return for a returned issue is a no-op.
- `RenewBook` is idempotent on `(book_issue_id, new_due_date)`.
  A duplicate returns the prior renewal.
- `CalculateFine` is naturally idempotent: it always
  recomputes from the current state and the as-of date.

## Cross-Domain Coordination

- The library domain subscribes to `StudentAdmitted` to
  auto-create a member; the subscriber is idempotent.
- The library domain subscribes to `StudentWithdrawn` to
  flag open issues for return; the member is not deleted.
- The library domain emits `BookIssued`, `BookReturned`,
  `BookRenewed`, `BookMarkedLost`, and `FineCalculated`.
  Finance subscribes to `FineCalculated` (and the implicit
  loss event in `BookMarkedLost`) to post receivables.
  Communication subscribes to `BookIssued`, `BookRenewed`,
  and overdue evaluations to notify the member.
