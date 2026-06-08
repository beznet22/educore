# Library Domain — Commands

Quick reference of every command the library domain exposes. These
commands cover the book catalog, library members, book issue
lifecycle (issue, return, renew, lost, fine calculation), and a small
set of read commands.

The "Events" column lists the events the command emits; consult the
per-domain spec for payload structure.

| Command                          | Capability                      | Description                                                                                       | Events                                                          | Idempotent? | Offline? |
| -------------------------------- | ------------------------------- | ------------------------------------------------------------------------------------------------- | --------------------------------------------------------------- | ----------- | -------- |
| `CreateBookCategory`             | `BookCategory.Create`           | Create a book category.                                                                           | `BookCategoryCreated`                                           | no          | yes      |
| `UpdateBookCategory`             | `BookCategory.Update`           | Patch a book category.                                                                            | `BookCategoryUpdated`                                           | no          | yes      |
| `DeleteBookCategory`             | `BookCategory.Delete`           | Soft-delete a book category with no books.                                                        | `BookCategoryDeleted`                                           | no          | yes      |
| `AddBook`                        | `Book.Add`                      | Add a book to the catalog with initial stock.                                                     | `BookAdded`                                                     | no          | yes      |
| `UpdateBook`                    | `Book.Update`                   | Patch a book's mutable fields.                                                                    | `BookUpdated`                                                   | no          | yes      |
| `DeleteBook`                    | `Book.Delete`                   | Soft-delete a book with no issue history.                                                         | `BookDeleted`                                                   | no          | yes      |
| `AdjustBookQuantity`             | `Book.AdjustQuantity`           | Adjust a book's stock count with a reason.                                                        | `BookQuantityAdjusted`                                          | no          | yes      |
| `RegisterLibraryMember`          | `Member.Register`               | Register a student or staff member as a library member.                                           | `LibraryMemberRegistered`                                       | no          | yes      |
| `UpdateLibraryMember`            | `Member.Update`                 | Patch a library member's mutable fields.                                                          | `LibraryMemberUpdated`                                         | no          | yes      |
| `DeactivateLibraryMember`        | `Member.Deactivate`             | Deactivate a member who holds no open issues.                                                     | `LibraryMemberDeactivated`                                      | no          | yes      |
| `ReactivateLibraryMember`        | `Member.Reactivate`             | Reactivate a member.                                                                              | `LibraryMemberReactivated`                                      | no          | yes      |
| `DeleteLibraryMember`            | `Member.Delete`                 | Soft-delete a member with no issue history.                                                       | `LibraryMemberDeleted`                                          | no          | yes      |
| `IssueBook`                      | `BookIssue.Issue`               | Issue a book to a member.                                                                         | `BookIssued`                                                    | no          | yes      |
| `ReturnBook`                     | `BookIssue.Return`              | Return an issued book.                                                                            | `BookReturned`, `FineCalculated` (if a fine applies)            | no          | yes      |
| `RenewBook`                      | `BookIssue.Renew`               | Renew an issued book.                                                                             | `BookRenewed`                                                   | no          | yes      |
| `MarkBookLost`                   | `BookIssue.MarkLost`            | Mark an issued book as lost.                                                                      | `BookMarkedLost`, `FineCalculated` (if configured)              | no          | yes      |
| `CalculateFine`                  | `BookIssue.CalculateFine`        | Compute and record a fine on an issue.                                                            | `FineCalculated`                                                | no          | yes      |
| `SearchBooks`                    | `Book.Read`                     | Query command: search the book catalog.                                                           | (none — query only)                                             | yes         | yes      |
| `ListOverdueIssues`              | `BookIssue.Read`                | Query command: list overdue issues as of a date.                                                  | (none — query only)                                             | yes         | yes      |
| `ListMemberIssues`               | `Member.Read`                   | Query command: list a member's open and closed issues.                                            | (none — query only)                                             | yes         | yes      |

**See also:** `docs/specs/library/commands.md` for full Rust struct
definitions, pre-conditions, effects, and edge-case handling.
