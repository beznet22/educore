# Library Domain — Tables

The library domain is backed by the following tables. Each
table maps to one or more aggregates; the `aggregate` column
tells you which aggregate owns the row.

| Table                            | Aggregate           | Notes                                    |
| -------------------------------- | ------------------- | ---------------------------------------- |
| `sm_book_categories`             | BookCategory        | Category catalog                         |
| `sm_books`                       | Book                | Book master                              |
| `sm_library_members`             | LibraryMember       | Registered borrowers                     |
| `sm_book_issues`                 | BookIssue           | Issue, return, renew, lost               |
| `sm_book_issue_renewals`         | BookIssueRenewal    | Renewal history                          |
| `sm_book_issue_fines`            | BookIssueFine       | Fine history                             |
| `sm_book_acquisitions`           | BookAcquisition     | Acquisition history                      |
| `sm_book_catalog_entries`        | BookCatalogEntry    | Cataloguing metadata history             |
| `sm_library_member_notes`        | LibraryMemberNote   | Administrative notes on a member         |

## Field Mapping

The canonical field mapping (from the migration schema) is
documented below. Storage adapters MAY use the same column
names when implementing the port. The engine is
column-name agnostic; the mapping is a recommendation, not a
requirement.

### BookCategory

| Column            | Type                | Maps to                              |
| ----------------- | ------------------- | ------------------------------------ |
| `id`              | `u64` / `Uuid`      | `BookCategoryId`                     |
| `category_name`   | `VARCHAR(200)`      | `CategoryName`                       |
| `school_id`       | `u64`               | `SchoolId`                           |
| `academic_id`     | `u64`               | `AcademicYearId`                     |
| `created_at`      | `TIMESTAMP`         | engine-managed                       |
| `updated_at`      | `TIMESTAMP`         | engine-managed                       |

### Book

| Column              | Type                | Maps to                              |
| ------------------- | ------------------- | ------------------------------------ |
| `id`                | `u64` / `Uuid`      | `BookId`                             |
| `book_title`        | `VARCHAR(200)`      | `BookTitle`                          |
| `book_number`       | `VARCHAR(200)`      | `BookNumber`                         |
| `isbn_no`           | `VARCHAR(200)`      | `Isbn`                               |
| `publisher_name`    | `VARCHAR(200)`      | `Publisher`                          |
| `author_name`       | `VARCHAR(200)`      | `Author`                             |
| `rack_number`       | `VARCHAR(50)`       | `RackNumber`                         |
| `quantity`          | `INT`               | `StockCopies`                        |
| `book_price`        | `INT`               | `BookPrice`                          |
| `post_date`         | `DATE`              | `NaiveDate`                          |
| `details`           | `VARCHAR(500)`      | `Details`                            |
| `active_status`     | `TINYINT`           | `ActiveStatus`                       |
| `book_subject_id`   | `u64`               | `SubjectId`                          |
| `book_category_id`  | `u64`               | `BookCategoryId`                     |
| `school_id`         | `u64`               | `SchoolId`                           |
| `academic_id`       | `u64`               | `AcademicYearId`                     |

### LibraryMember

| Column              | Type                | Maps to                              |
| ------------------- | ------------------- | ------------------------------------ |
| `id`                | `u64` / `Uuid`      | `LibraryMemberId`                    |
| `member_ud_id`      | `VARCHAR(191)`      | `MemberUdId`                         |
| `active_status`     | `TINYINT`           | `ActiveStatus`                       |
| `member_type`       | `u64`               | `MemberType` (RoleId)                |
| `student_staff_id`  | `u64`               | `MemberId` (StudentId or StaffId)    |
| `school_id`         | `u64`               | `SchoolId`                           |
| `academic_id`       | `u64`               | `AcademicYearId`                     |

### BookIssue

| Column              | Type                | Maps to                              |
| ------------------- | ------------------- | ------------------------------------ |
| `id`                | `u64` / `Uuid`      | `BookIssueId`                        |
| `quantity`          | `INT`               | `IssueQuantity`                      |
| `given_date`        | `DATE`              | `GivenDate`                          |
| `due_date`          | `DATE`              | `DueDate`                            |
| `issue_status`      | `VARCHAR(191)`      | `IssueStatus`                        |
| `note`              | `VARCHAR(500)`      | `IssueNote`                          |
| `active_status`     | `TINYINT`           | `ActiveStatus`                       |
| `book_id`           | `u64`               | `BookId`                             |
| `member_id`         | `u64`               | `LibraryMemberId`                    |
| `school_id`         | `u64`               | `SchoolId`                           |
| `academic_id`       | `u64`               | `AcademicYearId`                     |

## Notes

- Every table includes `school_id` for multi-tenant isolation.
  The `school_id` is `NOT NULL` and indexed.
- Every table includes `created_at`, `updated_at`,
  `created_by`, `updated_by`, `active_status` (where
  applicable). These are managed by the engine's storage
  adapter.
- `academic_id` references `sm_academic_years` (the
  per-year scope) and exists on every operational table.
- The `student_staff_id` column on `sm_library_members` is a
  polymorphic reference: it is either a `StudentId` or a
  `StaffId`. The `member_type` column (a `RoleId`) tells the
  domain layer how to interpret the reference. The storage
  adapter enforces that the value is consistent with the
  member type (a `RoleId` of `Student` requires the reference
  to resolve to a `StudentId`).
- The `member_id` column on `sm_book_issues` references
  `sm_library_members.id` (a `LibraryMemberId`), not the
  underlying user table. This keeps the library domain
  decoupled from the platform user table.
- Foreign keys use `ON DELETE CASCADE`. The engine does not
  rely on database cascades for invariant enforcement; the
  application layer checks referential integrity before
  issuing the delete command.
- The `book_price` column on `sm_books` is `INT` in the
  source migration. The engine treats it as a non-negative
  integer and surfaces it as a `BookPrice` value object.
  Storage adapters MAY widen the column to `DECIMAL(20,2)`
  in their own migrations.
- The `quantity` column on `sm_books` is `INT` in the source
  migration. The engine treats it as a `u32` and surfaces it
  as `StockCopies`. Storage adapters MAY widen the column
  when fractional stock is needed (e.g. for shared sets in
  classroom libraries); in v1 the engine assumes whole
  copies.
