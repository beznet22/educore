# Library Domain — Aggregates

## BookCategory

**Root type:** `BookCategory`
**Identity:** `BookCategoryId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Library

### Purpose

A grouping of books (e.g. "Fiction", "Reference", "Textbook",
"Periodical"). Used for reporting and authorization of issues.

### Invariants

1. `CategoryName` is unique within a school.
2. A `BookCategory` may not be deleted while any `Book` references
   it.

### Commands

- `CreateBookCategory`
- `UpdateBookCategory`
- `DeleteBookCategory`

### Events

- `BookCategoryCreated`
- `BookCategoryUpdated`
- `BookCategoryDeleted`

---

## Book

**Root type:** `Book`
**Identity:** `BookId(SchoolId, Uuid)`

### Purpose

A book master record. Carries the bibliographic metadata
(`title`, `author`, `publisher`, `edition`), the cataloguing
metadata (`isbn_no`, `book_number`, `rack_number`), the
acquisition metadata (`book_price`, `post_date`), and the stock
counter (`quantity`).

### Invariants

1. `BookTitle` is non-empty.
2. `Isbn`, if present, is unique within a school.
3. `BookNumber`, if present, is unique within a school.
4. `Quantity` is non-negative at all times. It represents the
   total number of physical copies held by the library.
5. An `Isbn` and a `BookNumber` are not both required: at least
   one is present on every book.
6. A `Book` belongs to exactly one `BookCategory` and one
   `Subject` (a `SubjectId` is taken from the academic domain
   for cross-listing).
7. The number of copies currently on issue (sum of
   `BookIssue.Quantity` for `Issued`, `Renewed`, or `Overdue`
   issues) may not exceed `Book.Quantity`.
8. A `Book` may not be deleted while any `BookIssue` references
   it in any year.

### Commands

- `AddBook`
- `UpdateBook`
- `DeleteBook`
- `AdjustBookQuantity` (used after acquisition or write-off)

### Events

- `BookAdded`
- `BookUpdated`
- `BookDeleted`
- `BookQuantityAdjusted`

### Consistency Boundary

All book mutations are serialized through the `Book` aggregate
root. The book is loaded by id, mutated in memory, validated, and
persisted with its events in a single transaction. The
`BookIssue` aggregate reads `book.quantity` and the current
sum of open issues atomically; the storage adapter enforces the
"open issues <= quantity" invariant through a conditional
update.

---

## LibraryMember

**Root type:** `LibraryMember`
**Identity:** `LibraryMemberId(SchoolId, Uuid)`

### Purpose

A registered borrower. May be a student or a staff member. Each
member has a `MemberType` (from the role catalog) and a
`StudentStaffId` (the underlying user id from the platform).

### Invariants

1. A `LibraryMember` references exactly one of `StudentId` or
   `StaffId`. In storage the column is `student_staff_id`; the
   domain value object `MemberSpec` is a sum type that
   disambiguates.
2. `MemberType` is a `RoleId` from the RBAC domain. A school
   policy (out of scope for v1) may restrict which roles are
   eligible for membership.
3. A `LibraryMember` is uniquely identified by
   `(member_type, student_staff_id)` within a school-year.
4. A `LibraryMember` is `Active` by default. A deactivated
   member may not receive new issues.
5. A `LibraryMember` may not be deleted while any `BookIssue`
   references them in any year.

### Commands

- `RegisterLibraryMember`
- `UpdateLibraryMember`
- `DeactivateLibraryMember`
- `ReactivateLibraryMember`
- `DeleteLibraryMember`

### Events

- `LibraryMemberRegistered`
- `LibraryMemberUpdated`
- `LibraryMemberDeactivated`
- `LibraryMemberReactivated`
- `LibraryMemberDeleted`

---

## BookIssue

**Root type:** `BookIssue`
**Identity:** `BookIssueId(SchoolId, Uuid)`

### Purpose

An issue of a quantity of copies of a `Book` to a
`LibraryMember`, with a given date, a due date, and a status.

### Invariants

1. The `BookIssue` references exactly one `Book` and one
   `LibraryMember`.
2. `Quantity` is positive.
3. `GivenDate` is on or after the academic year start.
4. `DueDate` is strictly after `GivenDate`.
5. The sum of `Quantity` across open issues for the book may not
   exceed `Book.Quantity`.
6. The book and the member are both active in the current
   academic year.
7. `IssueStatus` is one of `Issued`, `Returned`, `Renewed`,
   `Overdue`, `Lost`. `Overdue` is set by a background job or
   query and reflects a `Returned`-pending state; it is
   terminal only after a return is recorded.
8. A `Returned` or `Lost` issue is immutable.
9. A renewal may be performed only on an `Issued` or `Renewed`
   issue, and only when the member has no overdue book.
10. Renewal extends `DueDate` but does not change `GivenDate` or
    `Quantity`.

### Commands

- `IssueBook`
- `ReturnBook`
- `RenewBook`
- `MarkBookLost`
- `CalculateFine`

### Events

- `BookIssued`
- `BookReturned`
- `BookRenewed`
- `BookMarkedLost`
- `FineCalculated`

### Consistency Boundary

The aggregate is a single header row. Lines are not modeled in
v1: each `BookIssue` covers a single book. The
`Quantity` field is the number of copies issued in the
transaction. Multiple copies of the same book to the same
member on the same day are merged into a single issue by the
command handler.
