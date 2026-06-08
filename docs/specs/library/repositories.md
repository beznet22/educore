# Library Domain — Repositories

Repositories are ports (Rust traits). Adapters implement them.
The default adapter targets PostgreSQL; an SQLite adapter is
provided for embedded deployments.

All repository methods take a `SchoolId` (or operate on a
typed identifier that already embeds it) and refuse to return
data from another school. Tenant isolation is structural.

## BookCategoryRepository

```rust
#[async_trait]
pub trait BookCategoryRepository: Send + Sync {
    async fn get(&self, id: BookCategoryId) -> Result<Option<BookCategory>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<BookCategory>>;
    async fn find_by_name(&self, school: SchoolId, name: &CategoryName) -> Result<Option<BookCategory>>;
    async fn insert(&self, c: &BookCategory) -> Result<()>;
    async fn update(&self, c: &BookCategory) -> Result<()>;
    async fn delete(&self, id: BookCategoryId) -> Result<()>;
}
```

## BookRepository

```rust
#[async_trait]
pub trait BookRepository: Send + Sync {
    async fn get(&self, id: BookId) -> Result<Option<Book>>;
    async fn get_by_isbn(&self, school: SchoolId, isbn: &Isbn) -> Result<Option<Book>>;
    async fn get_by_book_number(&self, school: SchoolId, book_number: &BookNumber) -> Result<Option<Book>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<Book>>;
    async fn list_for_category(&self, school: SchoolId, category: BookCategoryId) -> Result<Vec<Book>>;
    async fn search(&self, school: SchoolId, query: &str, category: Option<BookCategoryId>, limit: u32) -> Result<Vec<Book>>;
    async fn insert(&self, book: &Book) -> Result<()>;
    async fn update(&self, book: &Book) -> Result<()>;
    async fn delete(&self, id: BookId) -> Result<()>;
    async fn adjust_quantity(&self, id: BookId, new_quantity: StockCopies) -> Result<()>;
    async fn list_with_availability(&self, school: SchoolId) -> Result<Vec<BookWithAvailability>>;
}
```

`list_with_availability` joins `books` with the current sum of
open issues per book, returning `BookWithAvailability` for the
catalog browse page.

`adjust_quantity` performs the atomic
`UPDATE ... SET quantity = $new WHERE id = $id AND $new >=
(SELECT coalesce(sum(quantity), 0) FROM book_issues WHERE
book_id = $id AND status IN ('Issued', 'Renewed', 'Overdue'))`
to enforce the "open issues <= quantity" invariant under
concurrency.

## LibraryMemberRepository

```rust
#[async_trait]
pub trait LibraryMemberRepository: Send + Sync {
    async fn get(&self, id: LibraryMemberId) -> Result<Option<LibraryMember>>;
    async fn find(&self, school: SchoolId, year: AcademicYearId, member: MemberId, member_type: MemberType) -> Result<Option<LibraryMember>>;
    async fn list(&self, school: SchoolId, year: AcademicYearId) -> Result<Vec<LibraryMember>>;
    async fn list_active(&self, school: SchoolId, year: AcademicYearId) -> Result<Vec<LibraryMember>>;
    async fn insert(&self, m: &LibraryMember) -> Result<()>;
    async fn update(&self, m: &LibraryMember) -> Result<()>;
    async fn deactivate(&self, id: LibraryMemberId) -> Result<()>;
    async fn reactivate(&self, id: LibraryMemberId) -> Result<()>;
    async fn delete(&self, id: LibraryMemberId) -> Result<()>;
}
```

## BookIssueRepository

```rust
#[async_trait]
pub trait BookIssueRepository: Send + Sync {
    async fn get(&self, id: BookIssueId) -> Result<Option<BookIssue>>;
    async fn list_for_member(&self, member: LibraryMemberId) -> Result<Vec<BookIssue>>;
    async fn list_for_book(&self, book: BookId) -> Result<Vec<BookIssue>>;
    async fn list_open(&self, school: SchoolId, as_of: NaiveDate) -> Result<Vec<BookIssue>>;
    async fn list_overdue(&self, school: SchoolId, as_of: NaiveDate) -> Result<Vec<BookIssue>>;
    async fn list_for_date_range(&self, school: SchoolId, from: NaiveDate, to: NaiveDate) -> Result<Vec<BookIssue>>;
    async fn open_quantity_for_book(&self, book: BookId) -> Result<u32> { ... }
    async fn insert(&self, issue: &BookIssue) -> Result<()>;
    async fn update(&self, issue: &BookIssue) -> Result<()>;
    async fn append_renewal(&self, r: &BookIssueRenewal) -> Result<()>;
    async fn list_renewals(&self, issue: BookIssueId) -> Result<Vec<BookIssueRenewal>>;
    async fn append_fine(&self, f: &BookIssueFine) -> Result<()>;
    async fn list_fines(&self, issue: BookIssueId) -> Result<Vec<BookIssueFine>>;
    async fn waive_fine(&self, fine_id: BookIssueFineId, by: UserId, reason: String) -> Result<()>;
}
```

`list_overdue` returns issues whose stored status is
`Issued`, `Renewed`, or `Overdue` and whose `due_date` is
strictly before `as_of`. The status is reported as `Overdue`
when read.

`open_quantity_for_book` is an optimized aggregate used by
the issue command to enforce the open-issues invariant
without loading all open issues.

## Indexes (recommended)

The default PostgreSQL adapter documents the following
indexes; consumers should declare them in their migrations:

```sql
-- Book categories
CREATE UNIQUE INDEX ux_book_categories_school_id_name
    ON book_categories (school_id, category_name);

-- Books
CREATE UNIQUE INDEX ux_books_school_id_isbn
    ON books (school_id, isbn_no) WHERE isbn_no IS NOT NULL;
CREATE UNIQUE INDEX ux_books_school_id_book_number
    ON books (school_id, book_number) WHERE book_number IS NOT NULL;
CREATE INDEX ix_books_school_id_category
    ON books (school_id, book_category_id);
CREATE INDEX ix_books_school_id_subject
    ON books (school_id, book_subject_id) WHERE book_subject_id IS NOT NULL;
CREATE INDEX ix_books_school_id_title
    ON books (school_id, book_title);
CREATE INDEX ix_books_school_id_rack
    ON books (school_id, rack_number) WHERE rack_number IS NOT NULL;

-- Library members
CREATE UNIQUE INDEX ux_library_members_school_id_year_member
    ON library_members (school_id, academic_year_id, member_type, student_staff_id);
CREATE INDEX ix_library_members_school_id_year_status
    ON library_members (school_id, academic_year_id, active_status);
CREATE INDEX ix_library_members_school_id_student
    ON library_members (school_id, student_staff_id) WHERE member_type = 'student';

-- Book issues
CREATE INDEX ix_book_issues_school_id_member
    ON book_issues (school_id, library_member_id);
CREATE INDEX ix_book_issues_school_id_book
    ON book_issues (school_id, book_id);
CREATE INDEX ix_book_issues_school_id_status_due
    ON book_issues (school_id, issue_status, due_date);
CREATE INDEX ix_book_issues_school_id_due_date
    ON book_issues (school_id, due_date)
    WHERE issue_status IN ('Issued', 'Renewed', 'Overdue');
CREATE INDEX ix_book_issues_school_id_year
    ON book_issues (school_id, academic_year_id);
```

The `school_id` predicate is mandatory for tenant isolation.
All queries are rewritten by the storage adapter to add
`school_id = $1` automatically.
