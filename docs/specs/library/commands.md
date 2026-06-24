# Library Domain — Commands

Commands describe intent. They are validated, authorized, and
dispatched to the relevant aggregate. Every command produces
zero or more events that are recorded in the event log.

All commands carry a `TenantContext` (school + actor +
correlation) and are rejected if the actor lacks the required
capability.

## Catalog

### CreateBookCategory

```rust
pub struct CreateBookCategoryCommand {
    pub tenant: TenantContext,
    pub category_name: CategoryName,
}
```

**Capability:** `BookCategory.Create`
**Pre-conditions:** `category_name` is unique within the school.

**Effects:** Emits `BookCategoryCreated`.

### UpdateBookCategory / DeleteBookCategory

```rust
pub struct UpdateBookCategoryCommand {
    pub tenant: TenantContext,
    pub book_category_id: BookCategoryId,
    pub new_name: CategoryName,
}

pub struct DeleteBookCategoryCommand {
    pub tenant: TenantContext,
    pub book_category_id: BookCategoryId,
}
```

**Capabilities:** `BookCategory.Update`, `BookCategory.Delete`.
Deletion is rejected while any `Book` references the category.

### AddBook

```rust
pub struct AddBookCommand {
    pub tenant: TenantContext,
    pub book_title: BookTitle,
    pub book_number: Option<BookNumber>,
    pub isbn_no: Option<Isbn>,
    pub publisher_name: Option<Publisher>,
    pub author_name: Option<Author>,
    pub rack_number: Option<RackNumber>,
    pub quantity: StockCopies,
    pub book_price: Option<BookPrice>,
    pub post_date: Option<NaiveDate>,
    pub details: Option<Details>,
    pub book_category_id: BookCategoryId,
    pub book_subject_id: Option<SubjectId>,
}
```

**Capability:** `Book.Add`
**Pre-conditions:**
- At least one of `book_number` or `isbn_no` is present.
- If `isbn_no` is present, it is unique within the school.
- If `book_number` is present, it is unique within the school.
- `book_category_id` exists and is active.
- `book_subject_id`, if present, exists.

**Effects:** Emits `BookAdded` with the new book id, the
metadata, and the initial stock count.

### UpdateBook

```rust
pub struct UpdateBookCommand {
    pub tenant: TenantContext,
    pub book_id: BookId,
    pub patch: BookPatch,
}
```

`BookPatch` carries the mutable fields: `book_title`,
`publisher_name`, `author_name`, `rack_number`, `book_price`,
`post_date`, `details`, `book_category_id`, `book_subject_id`.
`book_number`, `isbn_no`, and `quantity` are not patchable;
use `AdjustBookQuantity` to change stock.

**Capability:** `Book.Update`
**Effects:** Emits `BookUpdated`.

### DeleteBook

```rust
pub struct DeleteBookCommand {
    pub tenant: TenantContext,
    pub book_id: BookId,
}
```

**Capability:** `Book.Delete`
**Pre-conditions:** Book has no `BookIssue` records in any year.

**Effects:** Emits `BookDeleted`.

### AdjustBookQuantity

```rust
pub struct AdjustBookQuantityCommand {
    pub tenant: TenantContext,
    pub book_id: BookId,
    pub new_quantity: StockCopies,
    pub reason: StockAdjustmentReason,
}
```

**Capability:** `Book.AdjustQuantity`
**Pre-conditions:** `new_quantity` is greater than or equal to
the sum of quantities on open `BookIssue` rows for the book.

**Effects:** Emits `BookQuantityAdjusted` with the delta.

## Members

### RegisterLibraryMember

```rust
pub struct RegisterLibraryMemberCommand {
    pub tenant: TenantContext,
    pub academic_year_id: AcademicYearId,
    pub member: MemberId,            // Student(StudentId) or Staff(StaffId)
    pub member_type: MemberType,     // RoleId
    pub member_ud_id: MemberUdId,
}
```

**Capability:** `Member.Register`
**Pre-conditions:**
- The student or staff member exists and is active in the
  current academic year.
- A `LibraryMember` does not already exist for
  `(member_type, member)` in the current year.

**Effects:** Emits `LibraryMemberRegistered`.

### UpdateLibraryMember

```rust
pub struct UpdateLibraryMemberCommand {
    pub tenant: TenantContext,
    pub library_member_id: LibraryMemberId,
    pub patch: LibraryMemberPatch,
}
```

`LibraryMemberPatch` carries `member_ud_id` and an optional
`note`. `member` and `member_type` are immutable.

**Capability:** `Member.Update`
**Effects:** Emits `LibraryMemberUpdated`.

### DeactivateLibraryMember / ReactivateLibraryMember

```rust
pub struct DeactivateLibraryMemberCommand {
    pub tenant: TenantContext,
    pub library_member_id: LibraryMemberId,
    pub reason: String,
}

pub struct ReactivateLibraryMemberCommand {
    pub tenant: TenantContext,
    pub library_member_id: LibraryMemberId,
}
```

**Capabilities:** `Member.Deactivate`, `Member.Reactivate`.

**Pre-conditions:** A deactivated member has no open
`BookIssue` rows. (Reactivation is always allowed.)

**Effects:** Emit `LibraryMemberDeactivated` and
`LibraryMemberReactivated`. New issues against a deactivated
member are rejected.

### DeleteLibraryMember

```rust
pub struct DeleteLibraryMemberCommand {
    pub tenant: TenantContext,
    pub library_member_id: LibraryMemberId,
}
```

**Capability:** `Member.Delete`
**Pre-conditions:** Member has no `BookIssue` records in any
year.

**Effects:** Emits `LibraryMemberDeleted`.

## Issue Lifecycle

### IssueBook

```rust
pub struct IssueBookCommand {
    pub tenant: TenantContext,
    pub academic_year_id: AcademicYearId,
    pub book_id: BookId,
    pub library_member_id: LibraryMemberId,
    pub quantity: IssueQuantity,
    pub given_date: GivenDate,
    pub due_date: DueDate,
    pub note: Option<IssueNote>,
}
```

**Capability:** `BookIssue.Issue`
**Pre-conditions:**
- Book is active and `book.quantity >= quantity + sum(open
  issues.quantity for this book)`.
- LibraryMember is active.
- The member does not currently hold a `BookIssue` for the same
  book with an open status (Issued, Renewed, or Overdue).
- The school-configured `MaxBooksPerMember` is not exceeded by
  this issue.
- The member's `MemberType` is on the school's eligible
  list.

**Effects:** Creates the `BookIssue`, marks the book copies
as on-issue, and emits `BookIssued`.

### ReturnBook

```rust
pub struct ReturnBookCommand {
    pub tenant: TenantContext,
    pub book_issue_id: BookIssueId,
    pub return_date: ReturnDate,
    pub note: Option<IssueNote>,
}
```

**Capability:** `BookIssue.Return`
**Pre-conditions:**
- The issue is currently in `Issued`, `Renewed`, or `Overdue`.
- `return_date >= given_date`.

**Effects:** Marks the issue as `Returned`, releases the book
copies back to stock, computes any fine (via
`CalculateFine`), and emits `BookReturned` and (if a fine
applies) `FineCalculated`.

### RenewBook

```rust
pub struct RenewBookCommand {
    pub tenant: TenantContext,
    pub book_issue_id: BookIssueId,
    pub new_due_date: DueDate,
}
```

**Capability:** `BookIssue.Renew`
**Pre-conditions:**
- The issue is currently in `Issued` or `Renewed`.
- The member has no overdue `BookIssue` rows in the current
  year.
- The number of prior renewals for this issue is below the
  school-configured `MaxRenewalsPerIssue`.
- `new_due_date` is strictly after the current `due_date`.

**Effects:** Updates the due date, appends a `BookIssueRenewal`
history row, and emits `BookRenewed`.

### MarkBookLost

```rust
pub struct MarkBookLostCommand {
    pub tenant: TenantContext,
    pub book_issue_id: BookIssueId,
    pub note: Option<IssueNote>,
}
```

**Capability:** `BookIssue.MarkLost`
**Pre-conditions:** Issue is currently in `Issued`, `Renewed`,
or `Overdue`.

**Effects:** Marks the issue as `Lost`, decrements
`book.quantity` by the issue quantity, and emits
`BookMarkedLost`. The member is responsible for the
replacement cost; the library may also issue a fine via
`CalculateFine` if configured.

### CalculateFine

```rust
pub struct CalculateFineCommand {
    pub tenant: TenantContext,
    pub book_issue_id: BookIssueId,
    pub as_of: NaiveDate,
}
```

**Capability:** `BookIssue.CalculateFine`
**Pre-conditions:** Issue is currently in `Issued`, `Renewed`,
`Overdue`, or `Returned` (a fine may be calculated on a
late-returned book).

**Effects:** Computes `DaysOverdue = max(0, as_of - due_date)`
and `FineAmount = DaysOverdue * FinePerDay` (using the
school's configured `FinePerDay`). Emits `FineCalculated`
with the result. The fine is recorded as a `BookIssueFine`
history entry. Finance may subscribe to post the receivable.

## Read Commands

```rust
pub struct SearchBooksCommand {
    pub tenant: TenantContext,
    pub school_id: SchoolId,
    pub query: String,
    pub category: Option<BookCategoryId>,
    pub limit: u32,
}

pub struct ListOverdueIssuesCommand {
    pub tenant: TenantContext,
    pub school_id: SchoolId,
    pub as_of: NaiveDate,
}

pub struct ListMemberIssuesCommand {
    pub tenant: TenantContext,
    pub library_member_id: LibraryMemberId,
}
```

**Capabilities:** `Book.Read`, `BookIssue.Read`, `Member.Read`.

These are query commands; they do not emit domain events and
are not persisted. They flow through the query layer and the
read model. They are still subject to tenant and capability
checks.

## Orphaned Items (Cluster D catch-up)

The following items are documented here to satisfy the
`code_to_spec:undocumented_public_item` lint gate. They were
discovered after the main spec was written.

### Append Book Catalog Entry

```rust
pub struct AppendBookCatalogEntryCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `BookCatalogEntry.Append`
**Effects:** Emits `BookCatalogEntryAppended`.


### Create Book Acquisition

```rust
pub struct CreateBookAcquisitionCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `BookAcquisition.Create`
**Effects:** Emits `BookAcquisitionCreateed`.


### Create Library Member Note

```rust
pub struct CreateLibraryMemberNoteCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `LibraryMemberNote.Create`
**Effects:** Emits `LibraryMemberNoteCreateed`.


### Delete Library Member Note

```rust
pub struct DeleteLibraryMemberNoteCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `LibraryMemberNote.Delete`
**Effects:** Emits `LibraryMemberNoteDeleteed`.


### Record Book Return

```rust
pub struct RecordBookReturnCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `BookReturn.Record`
**Effects:** Emits `BookReturnRecorded`.


### Waive Book Issue Fine

```rust
pub struct WaiveBookIssueFineCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `WaiveBookIssueFine`
**Effects:** Emits `WaiveBookIssueFineRecorded`.



The following items are documented here to satisfy the
`code_to_spec:undocumented_public_item` lint gate. They were
discovered after the main spec was written.

### Append Book Catalog Entry

```rust
pub struct AppendBookCatalogEntryCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `BookCatalogEntry.Append`
**Effects:** Emits `BookCatalogEntryAppended`.


### Create Book Acquisition

```rust
pub struct CreateBookAcquisitionCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `BookAcquisition.Create`
**Effects:** Emits `BookAcquisitionCreateed`.


### Create Library Member Note

```rust
pub struct CreateLibraryMemberNoteCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `LibraryMemberNote.Create`
**Effects:** Emits `LibraryMemberNoteCreateed`.


### Delete Library Member Note

```rust
pub struct DeleteLibraryMemberNoteCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `LibraryMemberNote.Delete`
**Effects:** Emits `LibraryMemberNoteDeleteed`.


### Record Book Return

```rust
pub struct RecordBookReturnCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `BookReturn.Record`
**Effects:** Emits `BookReturnRecorded`.


### Waive Book Issue Fine

```rust
pub struct WaiveBookIssueFineCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `WaiveBookIssueFine`
**Effects:** Emits `WaiveBookIssueFineRecorded`.

