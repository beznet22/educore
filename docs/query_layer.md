# Query Layer Specification

## Purpose

SMScore provides a compile-time-safe, storage-agnostic, Eloquent-like query
layer that supports domain repositories without requiring an ORM, schema
introspection, or runtime reflection.

The query layer is the **only** sanctioned way to read data outside of a
command's aggregate load.

## Non-Goals

- Not an ORM.
- Not Active Record.
- Not schema introspection.
- Not reflection.
- Not a dynamic query builder accepting arbitrary field names.
- Not a migration generator.
- Not a connection pool manager.

## Design Goals

- Compile-time field identifiers.
- Compile-time operators.
- Compile-time sort and pagination.
- Compile-time field-type awareness.
- Storage-agnostic translation in adapters.
- Zero allocation in the happy path where possible.
- Async-friendly.
- Suitable for both embedded SQLite and large PostgreSQL deployments.

## Field Definitions

Each aggregate defines a `Field` enum listing its queryable fields. The enum
implements `FieldOf<A>` for the aggregate type `A`, providing compile-time
metadata for adapters.

```rust
pub enum StudentField {
    Id,
    SchoolId,
    AdmissionNo,
    FirstName,
    LastName,
    FullName,
    DateOfBirth,
    Status,
    ClassId,
    SectionId,
    AcademicYearId,
    GuardianId,
    CreatedAt,
    UpdatedAt,
}

impl FieldOf<Student> for StudentField {
    fn column(&self) -> &'static str { match self {
        Self::Id => "id",
        Self::SchoolId => "school_id",
        Self::AdmissionNo => "admission_no",
        // ...
    }}
    fn value_type(&self) -> ValueType { match self {
        Self::Id => ValueType::Uuid,
        Self::AdmissionNo => ValueType::String,
        Self::Status => ValueType::Enum(StudentStatus::VALUES),
        // ...
    }}
}
```

## Query Expression

A query is a value object built by chained methods. Each method returns
`Self`, allowing fluent construction.

```rust
pub struct StudentQuery {
    school_id: SchoolId,
    filters: Vec<Filter<StudentField>>,
    order: Vec<OrderBy<StudentField>>,
    offset: u32,
    limit: u32,
}

impl StudentQuery {
    pub fn new(school_id: SchoolId) -> Self { ... }
    pub fn where_eq(self, field: StudentField, value: impl Into<FieldValue>) -> Self { ... }
    pub fn where_in(self, field: StudentField, values: Vec<FieldValue>) -> Self { ... }
    pub fn where_between(self, field: StudentField, lo: FieldValue, hi: FieldValue) -> Self { ... }
    pub fn where_null(self, field: StudentField) -> Self { ... }
    pub fn where_not_null(self, field: StudentField) -> Self { ... }
    pub fn order_by(self, field: StudentField) -> Self { ... }
    pub fn order_by_desc(self, field: StudentField) -> Self { ... }
    pub fn page(self, offset: u32, limit: u32) -> Self { ... }
    pub fn limit(self, n: u32) -> Self { ... }
    pub fn offset(self, n: u32) -> Self { ... }
}
```

A `Filter<Field>` is a typed tuple of `(field, operator, value)`. The
operator is also an enum:

```rust
pub enum Op { Eq, Ne, Lt, Lte, Gt, Gte, In, NotIn, Between, IsNull, IsNotNull, Like, ILike }
```

## Repository Use

Repositories accept a query and translate it for the underlying storage.

```rust
#[async_trait]
pub trait StudentRepository: Send + Sync {
    async fn query(&self, q: StudentQuery) -> Result<Vec<Student>>;
    async fn count(&self, q: StudentQuery) -> Result<u64>;
    async fn get(&self, id: StudentId) -> Result<Option<Student>>;
    async fn insert(&self, student: &Student) -> Result<()>;
    async fn update(&self, student: &Student) -> Result<()>;
    async fn delete(&self, id: StudentId) -> Result<()>;
}
```

The default `StorageAdapter` implementation of `StudentRepository` translates
the query into the underlying SQL (or other) dialect. The translation is
exhaustive at compile time because the `Field` enum has a finite number of
variants.

## Fluent API Style

Domain methods may also offer a fluent ergonomic layer built on top of the
generic query:

```rust
impl StudentQuery {
    pub fn active(self) -> Self {
        self.where_eq(StudentField::Status, StudentStatus::Active)
    }
    pub fn in_class(self, class_id: ClassId) -> Self {
        self.where_eq(StudentField::ClassId, class_id)
    }
    pub fn in_section(self, section_id: SectionId) -> Self {
        self.where_eq(StudentField::SectionId, section_id)
    }
    pub fn admitted_after(self, date: NaiveDate) -> Self {
        self.where_gt(StudentField::CreatedAt, date)
    }
}
```

This is sugar; the engine never relies on string keys or magic names.

## Domain-Specific Optimized Queries

Repositories may expose domain-specific queries that bypass the generic
builder when a more efficient path exists:

```rust
impl StudentRepository for PostgresStudentRepository {
    async fn active_in_term(
        &self,
        school_id: SchoolId,
        term_id: AcademicYearId,
    ) -> Result<Vec<Student>> {
        // Specialized SQL using a covering index.
    }
}
```

Optimized queries are explicit capabilities, not generic magic.

## Aggregation & Reporting

The query layer supports typed aggregations for reports:

```rust
let summary = engine
    .students()
    .aggregate(StudentAggregate::Count)
    .group_by(StudentField::ClassId)
    .where_eq(StudentField::Status, StudentStatus::Active)
    .execute()
    .await?;
```

Aggregations are `Count`, `Sum`, `Avg`, `Min`, `Max` over numeric fields.

## Pagination

Pagination is a first-class concept. The query returns a `Page<T>`:

```rust
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub offset: u32,
    pub limit: u32,
}
```

The total count is computed by the same query (with a `count(*)` swap) so
callers can render correct page numbers.

## Tenant Filtering

Every query MUST carry a `SchoolId`. The engine refuses to execute a query
without a tenant filter. Storage adapters may also enforce row-level security
as a defense in depth.

```rust
let q = StudentQuery::new(tenant.school_id())
    .where_eq(StudentField::Status, StudentStatus::Active);
```

A query that omits the school id is a compile error:

```rust
// Does not compile.
let q = StudentQuery::default()
    .where_eq(StudentField::Status, StudentStatus::Active);
```

The default constructor is private; only `StudentQuery::new(school_id)` is
public.

## Cursor Pagination

For large result sets (e.g. attendance over a year), the query layer
supports cursor pagination:

```rust
let mut cursor = StudentCursor::after(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
while let Some(page) = engine.students().next_page(cursor, 100).await? {
    cursor = page.next_cursor;
    // ...
}
```

Cursors are opaque `Cursor` values owned by the storage adapter.

## Transactional Semantics

Queries do not run in transactions by default. Commands that need a
transaction acquire one through the storage port's `Transaction` type and
issue reads and writes inside it. See `docs/ports/storage.md`.

## Error Handling

Repository methods return `Result<T, StorageError>`. The engine maps storage
errors to `DomainError::Infrastructure` and logs them through `tracing`.

## Performance

- Repositories MUST avoid `SELECT *`. They select only the columns the
  aggregate needs.
- Indexes are owned by the consumer's migrations. The engine documents
  expected indexes per aggregate in `docs/specs/<domain>/indexes.md`.
- Adapters should batch queries where the engine issues many lookups in a
  single command.

## Object Safety

The query layer does not require trait objects. The query is a value type
that the engine passes to the storage adapter by value or `&`.

## Testing

- The query builder has unit tests for every operator and edge case.
- Adapters have integration tests against a real database.
- Optimized domain queries have dedicated tests verifying that they are used
  in place of the generic builder where expected.

## Anti-Patterns (forbidden)

```rust
// NEVER
let q = engine.students().query()
    .where("status", "active")
    .order("last_name");

// NEVER
let raw = sqlx::query_as::<_, Student>("SELECT * FROM students WHERE school_id = $1");

// NEVER
let json: serde_json::Value = storage.fetch("students", id).await?;
```

## Summary

The query layer is:

- Compile-time safe.
- Storage-agnostic.
- Domain-friendly.
- Allocation-conscious.
- Reflection-free.
- Eloquent-like in ergonomics.
- Idiomatic Rust in implementation.
