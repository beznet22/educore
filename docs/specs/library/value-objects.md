# Library Domain — Value Objects

Value objects are immutable, validated at construction, and have
no identity. They are compared by value.

## Identifiers

All identifiers in the library domain are typed and
tenant-scoped. Two `BookId` values in different schools are
distinct types at the domain level and may be unified only
through explicit cross-tenant commands.

| Identifier           | Backing Type            | Source Column              |
| -------------------- | ----------------------- | -------------------------- |
| `BookId`             | `Id<Book>`              | `library_books.id`              |
| `BookCategoryId`     | `Id<BookCategory>`      | `library_book_categories.id`    |
| `LibraryMemberId`    | `Id<LibraryMember>`     | `library_members.id`    |
| `BookIssueId`        | `Id<BookIssue>`         | `library_book_issues.id`        |
| `BookIssueRenewalId` | `Id<...>`               | (derived)                  |
| `BookIssueFineId`    | `Id<...>`               | (derived)                  |
| `BookAcquisitionId`  | `Id<...>`               | (derived)                  |
| `BookCatalogEntryId` | `Id<...>`               | (derived)                  |
| `LibraryMemberNoteId`| `Id<...>`               | (derived)                  |

Identifiers from other domains referenced by the library domain:

| Identifier         | Source Domain      |
| ------------------ | ------------------ |
| `SchoolId`         | `educore-platform`   |
| `UserId`           | `educore-platform`   |
| `StudentId`        | `educore-academic`   |
| `StaffId`          | `educore-hr`         |
| `RoleId`           | `educore-rbac`       |
| `SubjectId`        | `educore-academic`   |
| `AcademicYearId`   | `educore-academic`   |
| `TenantContext`    | `educore-platform`   |

## Bibliographic

| Type              | Constraints                                              |
| ----------------- | -------------------------------------------------------- |
| `Isbn`            | 10 or 13 digits, with optional hyphens; checksum-valid   |
| `BookTitle`       | 1..200 chars                                             |
| `BookNumber`      | 1..200 chars, unique within school (cataloguing number)  |
| `Author`          | 1..200 chars                                             |
| `Publisher`       | 1..200 chars                                             |
| `Edition`         | 1..50 chars                                              |
| `RackNumber`      | 0..50 chars                                              |
| `CategoryName`    | 1..200 chars, unique within school                      |
| `Details`         | 0..500 chars (free-text description)                     |

## Members

| Type              | Constraints                                              |
| ----------------- | -------------------------------------------------------- |
| `MemberId`        | enum `Student(StudentId)` or `Staff(StaffId)`            |
| `MemberType`      | `RoleId` from the RBAC catalog                           |
| `MemberUdId`      | 1..191 chars; the user's external id (e.g. admission no) |

## Issues

| Type              | Constraints                                              |
| ----------------- | -------------------------------------------------------- |
| `IssueQuantity`   | `u32` > 0                                                |
| `GivenDate`       | `NaiveDate` on or after academic year start              |
| `DueDate`         | `NaiveDate` strictly after `GivenDate`                   |
| `ReturnDate`      | `NaiveDate`, >= `GivenDate`                              |
| `RenewalDate`     | `NaiveDate`, on or after `GivenDate`                     |
| `IssueStatus`     | enum `Issued`, `Returned`, `Renewed`, `Overdue`, `Lost`  |
| `IssueNote`       | 0..500 chars                                             |

## Money & Quantities

| Type              | Notes                                                    |
| ----------------- | -------------------------------------------------------- |
| `BookPrice`       | non-negative integer (acquisition price per copy)       |
| `FineAmount`      | `Decimal` >= 0                                           |
| `FinePerDay`      | `Decimal` >= 0                                           |
| `DaysOverdue`     | `u32`                                                    |
| `StockCopies`     | `u32` >= 0                                               |

## Status Enums

| Type                | Values                                                       |
| ------------------- | ------------------------------------------------------------ |
| `IssueStatus`       | `Issued`, `Returned`, `Renewed`, `Overdue`, `Lost`            |
| `MemberStatus`      | `Active`, `Inactive`, `Blocked`                              |
| `BookStatus`        | `Available`, `Catalogued`, `Retired`, `Lost`                 |

## Identity & Contact (read-only references)

| Type              | Notes                                                    |
| ----------------- | -------------------------------------------------------- |
| `PersonName`      | 1..200 chars                                             |
| `EmailAddress`    | RFC 5322                                                 |
| `PhoneNumber`     | E.164 format preferred                                   |

## Validation Rules

All value objects implement `Validate` and refuse construction
when validation fails:

```rust
pub trait Validate {
    fn validate(&self) -> Result<(), ValueError>;
}
```

Construction is the only entry point:

```rust
let isbn = Isbn::parse("978-0-13-468599-1")?;
let due = DueDate::new(given, DurationDays(14))?;
```

Parsing returns `Result<T, ValueError>`. There are no setters
that bypass validation. The library domain never exposes raw
strings or numerics where a value object exists.

## Cross-Domain Helpers

| Type              | Notes                                                    |
| ----------------- | -------------------------------------------------------- |
| `SchoolId`        | From `educore-platform`                                  |
| `UserId`          | From `educore-platform`                                  |
| `TenantContext`   | From `educore-platform`                                  |
| `StudentId`       | From `educore-academic` (read-only reference)            |
| `StaffId`         | From `educore-hr` (read-only reference)                  |
| `AcademicYearId`  | From `educore-academic` (read-only reference)            |
| `RoleId`          | From `educore-rbac` (read-only reference)                |
| `SubjectId`       | From `educore-academic` (read-only reference)            |
