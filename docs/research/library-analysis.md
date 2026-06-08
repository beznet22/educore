# Library Domain — Business Analysis

## Purpose

The library domain owns the school's library: books,
members, issues, returns, reservations, and fines.
It is one of the simpler operational domains, but
it has unique edge cases (overdue, lost, damaged,
reserved).

This document describes how a school library works
in real schools, with the edge cases that real
schools hit.

## Key Concepts

- **Book** — a book in the catalog (title, author,
  ISBN, publisher, year, edition).
- **BookCategory** — a category of book
  ("Fiction", "Non-Fiction", "Reference",
  "Textbook").
- **BookCopy** — a physical copy of a book
  (each copy has a unique accession number).
- **Member** — a library member (student, staff,
  or external).
- **MembershipType** — a category of membership
  (Student, Staff, External) with borrowing
  rules.
- **BookIssue** — a record of a book issued to a
  member.
- **BookReturn** — a record of a book returned by
  a member.
- **BookReservation** — a reservation of a book
  that is currently issued.
- **LibrarySubject** — a subject area used to
  classify books.
- **LibrarySettings** — per-school library
  configuration.

## Real-World Scenarios

### Book Catalog Management

The librarian adds books to the catalog:

1. The librarian creates a `Book` aggregate
   with the bibliographic details (title,
   author, ISBN, publisher, year, edition,
   category, subject).
2. The librarian creates one or more
   `BookCopy` records, each with a unique
   accession number (a barcode).
3. The copies are physically labeled with the
   accession number.
4. The book is available for issue.

In real schools, the library has thousands of
books, organized by category and subject. The
catalog supports search by title, author,
ISBN, category, and subject.

### Book Issue

A student borrows a book:

1. The student presents the book and the
   library card at the circulation desk.
2. The librarian scans the book's accession
   number and the student's library card.
3. The librarian selects the issue duration
   (e.g. 14 days).
4. The system creates a `BookIssue` with the
   book, the member, the issue date, the due
   date, and the issuer.
5. The book is checked out.

In real schools, the issue is **per-copy**: a
book may have multiple copies; each copy can
be issued independently. The engine's
`BookCopy` aggregate is the unit of issue.

### Book Return

The student returns the book:

1. The librarian scans the book's accession
   number.
2. The librarian locates the open `BookIssue`.
3. The librarian records the return date.
4. The system computes any overdue fine.
5. The `BookIssue` is closed.

In real schools, the return is **on-time** or
**overdue**. An overdue return triggers a
fine; a damaged book triggers a damage fee.

### Overdue Fine

A student returns a book late. The librarian:

1. Records the return with the actual return
   date.
2. The system computes the overdue duration
   (return_date - due_date).
3. The system computes the fine based on the
   school's policy (e.g. ₹2 per day).
4. The fine is added to the student's library
   account.
5. The parent is notified (via the
   communication domain).

In real schools, fines are:
- **Per-day** — a fixed amount per day
  overdue.
- **Capped** — maximum fine (e.g. equal to
  the book's price).
- **Waived** — for genuine reasons (e.g.
  medical leave) with the librarian's
  approval.

### Book Reservation

A student wants a book that is currently
issued. The student reserves the book:

1. The student searches the catalog.
2. The student clicks "Reserve" on an issued
   book.
3. The system creates a `BookReservation` for
   the student.
4. When the book is returned, the system
   notifies the next reservation in the queue.
5. The student has N days to pick up the
   book.

In real schools, reservations are:
- **First-come-first-served** — the queue
  is in reservation order.
- **Time-limited** — the student has N
  days to pick up; after that, the
  reservation expires.
- **Cancellable** — the student may cancel
  the reservation.

### Book Lost

A student reports a book as lost. The
librarian:

1. Records the loss with a date.
2. Marks the `BookCopy` as `Lost`.
3. Computes the replacement cost (book
   price + processing fee).
4. The cost is added to the student's
   library account.
5. The parent is notified.

### Book Damaged

A student returns a damaged book. The
librarian:

1. Records the damage with a description
   and photos (optional).
2. Marks the `BookCopy` as `Damaged`.
3. Computes the repair / replacement cost.
4. The cost is added to the student's
   library account.

### Library Membership

A student is automatically a library
member upon admission (the academic
domain's `StudentAdmitted` event drives
the library's `MemberCreated`). A staff
member is a member upon onboarding.

An external member (e.g. a parent's
relative) may apply for membership with
the librarian's approval.

### Library Card

A member has a library card (with a unique
barcode). The card is used for issue and
return. The engine's `MemberCard` aggregate
captures the card details; the physical
card is issued by the librarian.

### Library Fine Settlement

A student has outstanding library fines.
The student settles the fines at the
library. The librarian:

1. Records the payment (cash or wallet).
2. The fines are cleared.
3. The receipt is issued.

In real schools, library fines are also
visible in the parent's portal (with the
finance domain's link). Some schools
deduct library fines from the caution
deposit (if any).

### Library Inventory Audit

At the end of the year, the librarian
audits the collection:

1. The librarian performs a physical
   count.
2. The system shows the expected count
   per `Book`.
3. The librarian records discrepancies
   (lost, missing, damaged).
4. The system updates the inventory.

### Library Reports

The librarian generates reports:
- Most-issued books.
- Most-active members.
- Overdue list.
- Fine collection summary.
- Category-wise distribution.

The engine's `Report.Library.Generate`
command produces these reports.

## Business Rules

1. A `Book` is unique by `(school_id, isbn)`.
2. A `BookCopy` is unique by
   `(school_id, accession_number)`.
3. A `BookIssue` is unique by
   `(book_copy_id, open)`. Only one open
   issue per copy.
4. A `BookIssue`'s due date is in the
   future at issue time.
5. A `BookReturn` cannot precede the issue
   date.
6. A `Member` cannot have more than N
   books issued at a time (per
   `MembershipType`).
7. A `BookReservation` is unique by
   `(book_id, member_id, active)`. Only
   one active reservation per book per
   member.
8. A `BookCopy` cannot be issued if it is
   in `Lost`, `Damaged`, or `Archived`
   status.
9. The overdue fine is per-day, capped at
   the book's price.
10. The library membership is auto-created
    on `StudentAdmitted` and
    `StaffOnboarded`.

## Edge Cases

### Book Issued, Member Withdraws

A student has an issued book and then
withdraws from the school. The library
domain receives the `StudentWithdrawn`
event. The librarian records the book as
"to be returned." The student's
`Member` status becomes `Withdrawn`. The
book is not lost; it is "in possession of
a withdrawn student." The parent is
notified to return the book.

### Staff Member Resigns with Library Books

A staff member resigns with library books.
The HR domain's `StaffResigned` event
triggers the library to mark the staff's
issues. The librarian requests the return
before processing the final settlement.

### Reservation Queue for a Popular Book

A book is popular. 10 students have
reservations. The book is returned. The
first student in the queue is notified;
they have 3 days to pick up. If they do
not, the next student is notified. The
process repeats until the book is picked
up or the queue is exhausted.

### Reference Book (Cannot Be Issued)

A school has reference books (encyclopedias,
dictionaries) that cannot be issued. The
book's `is_reference` flag prevents
issuance. The book may be read in the
library only.

### Multi-Volume Set

A book is part of a multi-volume set
(e.g. an encyclopedia in 20 volumes).
Each volume is a separate `Book` with a
shared `set_id`. The librarian may issue
volumes independently or as a set (per
the school's policy).

### Book Replacement

A book is lost. The school buys a
replacement. The librarian:
1. Marks the lost copy as `Replaced`.
2. Adds a new `BookCopy` with a new
   accession number.
3. The book's `total_copies` increases.

### Library Closed for Inventory

The library is closed for a week for
inventory. The librarian:
1. Records the closure.
2. The engine's `LibraryClosed` event
   prevents new issues / returns during
   the period.
3. Pre-existing issues continue; their
   due dates are extended by the closure
   duration.

### Bulk Book Return

A class teacher returns 30 books at the
end of the term. The librarian records
the bulk return. The engine's bulk
return command is all-or-nothing; a
single validation failure aborts the
batch.

### Lost and Paid, Then Found

A student pays for a lost book, then
finds it. The librarian:
1. Reverses the lost record.
2. Refunds the cost (or credits the
   wallet).
3. The book is back in circulation.

## Notes for SMSengine Implementation

- The **library** crate depends on
  `smscore-academic` for `StudentId` and
  `smscore-hr` for `StaffId`.
- The library domain is **eventually
  consistent** with academic (member
  creation on admission), HR (member
  status on resignation), and finance
  (fine settlement).
- The library domain's **book copy** is
  the unit of issue. The engine's
  `BookIssue` is per-copy, not per-book.
- The library domain's **overdue
  computation** is per-school
  configuration. The engine reads the
  policy from the settings domain.
- The library domain's **fine
  settlement** is integrated with
  finance. The engine emits
  `LibraryFineSettled`; finance records
  the payment.
- The library domain's **bulk
  operations** are common. The engine's
  bulk commands (bulk issue, bulk
  return) are all-or-nothing.
- The library domain's **reports** are
  capability-gated. Only the librarian
  and the school admin can read
  library reports.
- The library domain's **events**
  (`BookIssued`, `BookReturned`,
  `BookOverdue`, `BookLost`,
  `BookDamaged`, `BookReserved`) drive
  downstream notifications and
  projections.
