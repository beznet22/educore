# Library Domain — Entities

Entities have identity and lifecycle but are not aggregate roots.
They are loaded and persisted only through their aggregate root.

## BookCatalogEntry

**Identity:** `BookCatalogEntryId(SchoolId, Uuid)`
**Owner:** `Book`

A versioned view of a book's cataloguing metadata. A new entry
is appended whenever `AddBook` or `UpdateBook` is issued. The
current state is the latest entry; history is the full log.

## BookAcquisition

**Identity:** `BookAcquisitionId(SchoolId, Uuid)`
**Owner:** `Book`

A single procurement event for a book. Carries `Vendor`,
`InvoiceNumber`, `UnitCost`, `Quantity`, and `AcquiredAt`. The
sum of acquisitions for a book is the total cost basis; the
engine does not surface cost analytics in v1, but the data is
preserved.

## LibraryMemberNote

**Identity:** `LibraryMemberNoteId(SchoolId, Uuid)`
**Owner:** `LibraryMember`

A free-text administrative note about a member (overdue pattern,
lost book, account hold). Has `Author`, `Body`, `CreatedAt`,
`VisibleToMember: bool`.

## BookIssueRenewal

**Identity:** `BookIssueRenewalId(SchoolId, Uuid)`
**Owner:** `BookIssue`

A historical record of a renewal. Has `RenewedAt`, `FromDueDate`,
`ToDueDate`, `RenewedBy: UserId`. The current due date is the
`ToDueDate` of the most recent renewal.

## BookIssueFine

**Identity:** `BookIssueFineId(SchoolId, Uuid)`
**Owner:** `BookIssue`

A historical record of a fine. Has `CalculatedAt`, `DaysOverdue`,
`PerDayRate`, `Amount`, `Waived: bool`, `WaivedBy: Option<UserId>`,
`WaivedReason: Option<String>`. A `BookIssue` may have at most
one open fine at a time; previous fines remain as history.

## BookReservation (future)

**Identity:** `BookReservationId(SchoolId, Uuid)`
**Owner:** `Book`

A future-scope entity for hold queues. Not in v1. Documented
here to record the design intent: a reservation carries
`MemberId`, `ReservedAt`, `ExpiresAt`, and `FulfilledBy:
Option<BookIssueId>`.

## LibrarySettings

**Identity:** `LibrarySettingsId(SchoolId, Uuid)`
**Owner:** `School` (in settings domain)

The per-school library configuration: `MaxBooksPerMember`,
`DefaultLoanDays`, `MaxRenewalsPerIssue`, `FinePerDay`,
`FineCurrency`, and a list of `EligibleMemberTypes`. The
library domain reads these but does not own the entity.
