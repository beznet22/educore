# Query Layer Specification

## Purpose

Educore provides a compile-time-safe, storage-agnostic, Eloquent-like
query layer through **procedural derive macros** that emit typed AST
components and state builders, combined with developer-authored
**extension traits** for domain-specific scopes. The layer supports
**closure-based nested relational filters** (`where_has`) and **strict
compile-time eager loading** (`.with(...)`) — both enforced at the type
level, with no runtime proxies, lazy getters, or string-based column
resolution.

The query layer is the **only** sanctioned way to read data outside of a
command's aggregate load.

## Non-Goals

- Not an ORM.
- Not Active Record.
- Not schema introspection.
- Not runtime reflection.
- Not a dynamic query builder accepting arbitrary field names.
- Not a migration generator.
- Not a connection pool manager.
- **Not** a source of generated SQL/NoSQL syntax. The macros emit AST,
  not queries.
- **Not** a lazy-loading framework. Lazy accessors and runtime proxies
  are categorically forbidden.

## Design Goals

- Compile-time field identifiers via macro-generated `*Field` enums.
- Compile-time operators (`Op::Eq`, `Op::In`, ...).
- Compile-time sort and pagination state.
- Compile-time field-type awareness.
- Storage-agnostic translation in adapters, working from the macro
  AST.
- Zero allocation in the happy path where possible.
- Async-friendly.
- Suitable for both embedded SQLite and large PostgreSQL deployments.
- Full IDE autocomplete through `rust-analyzer`-visible enums and
  builders.

## Macro Architecture

The query layer is built on top of the custom `#[derive(DomainQuery)]`
procedural macro. This macro is the **only** sanctioned way to expose a
domain record to the query layer.

### Macro Scope & Boundaries (HARD RULES)

The macro is **strictly** an AST generator. It MUST NOT:

- Generate raw SQL, NoSQL, or any storage-specific syntax.
- Generate string column names consumed by the query layer.
- Perform any runtime I/O.
- Establish database connections.
- Parse structural parameters from external files.
- Emit hidden magic, dynamic dispatch paths, or `Box<dyn Any>` payloads.

The macro MAY emit:

- A field-exhaustiveness enum (`StudentField`).
- A typed state builder (`StudentQueryBuilder`).
- Relation enums (`StudentRelation`).
- A `QueryNode` AST (used internally by the engine).

### Attribute-Driven Opt-in

Fields are excluded from query generation by default. A field is
queryable only when decorated:

```rust
#[derive(DomainQuery)]
pub struct Student {
    pub id: Uuid,                              // #[query(ignore)] implicitly

    #[query(sortable)]
    pub last_name: String,

    #[query(filterable)]
    pub status: StudentStatus,

    #[query(filterable, relation = "Parent", builder = "ParentQueryBuilder")]
    pub parent_id: Uuid,

    // Hydration targets are always ignored from filter generation
    #[query(ignore)]
    pub parent: Option<Parent>,
}
```

Attribute grammar:

| Attribute                                       | Meaning                                              |
| ----------------------------------------------- | ---------------------------------------------------- |
| `#[query(filterable)]`                          | Field can be used in a `.where_*` clause             |
| `#[query(sortable)]`                            | Field can be used in `.order_by(...)`                |
| `#[query(relation = "Name", builder = "T")]`    | Field links to a related entity's builder            |
| `#[query(ignore)]`                              | Field is excluded from both enum and builder         |
| (no attribute)                                  | Field is excluded from query generation              |

## Macro-Generated Artifacts

Given the `Student` definition above, the macro deterministically
emits **two discrete types** and a relation enum. No third-party
generator runs at compile time. No token streams leak into the public
API.

### 1. Field Exhaustiveness Enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StudentField {
    Status,
    LastName,
    ClassId,
    ParentId,
}
```

The enum is **exhaustive over queryable fields**. Every variant
corresponds to a single field decorated with `#[query(...)]`. A
non-decorated field produces no variant.

### 2. Type-Safe State Builder

```rust
pub struct StudentQueryBuilder {
    school_id: SchoolId,
    filters: Vec<QueryNode<StudentField>>,
    orders: Vec<OrderNode<StudentField>>,
    offset: u32,
    limit: u32,
    relations: BTreeSet<StudentRelation>,
}
```

The builder is a **state machine**, not a fluent abstract type. Every
method returns `Self`; predicates and orderings are collected into
internal vectors and compiled into a `QueryNode<StudentField>` AST
when `.await?` is called.

### 3. Relation Enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StudentRelation {
    Parent,
}
```

A relation enum is emitted per aggregate that declares at least one
field with a `relation` attribute. The relation enum is the single
typed channel through which `where_has` and `with` are expressed.

## Compile-Time Safety: The Core Pillar

The query builder relies entirely on macro-generated enums. **String
selectors are categorically forbidden.**

### Anti-Patterns vs. Preferred Patterns

```rust
// FORBIDDEN: runtime string match against a column
.where("students.status", "active")

// FORBIDDEN: schema introspection or reflection
.column_for_field("status")

// FORBIDDEN: implicit lazy loading
student.parent().await?

// PREFERRED: strictly typed, macro-generated field
.where_eq(StudentField::Status, StudentStatus::Active)

// PREFERRED: closure-based nested filter
.where_has(StudentRelation::Parent, |parent_q| {
    parent_q.where_eq(ParentField::BillingStatus, BillingStatus::Active)
})

// PREFERRED: explicit compile-time eager load
.with(StudentRelation::Parent)
```

The compiler refuses to accept the forbidden forms. There is no
escape hatch, no `unsafe`, no opt-out flag.

## Query AST

The builder accumulates state and emits a typed AST at the await
boundary:

```rust
pub enum QueryNode<F: FieldKind> {
    Eq(F, Value),
    Ne(F, Value),
    Lt(F, Value),
    Lte(F, Value),
    Gt(F, Value),
    Gte(F, Value),
    In(F, Vec<Value>),
    NotIn(F, Vec<Value>),
    Between(F, Value, Value),
    IsNull(F),
    IsNotNull(F),
    Like(F, Pattern),
    ILike(F, Pattern),
    HasRelation(<Self as HasRelations>::Relation, Box<QueryNode<RelatedField>>),
}

pub struct OrderNode<F: FieldKind> {
    pub field: F,
    pub direction: OrderDirection,
}

pub enum OrderDirection { Asc, Desc }
```

The AST is the **only** contract between the engine and the storage
adapter. Adapters translate the AST into the storage dialect.

## Query Method Semantics

### Filters

```rust
impl StudentQueryBuilder {
    pub fn where_eq<F: IntoFieldValue<StudentField>>(
        mut self,
        field: StudentField,
        value: F,
    ) -> Self;
    pub fn where_in(self, field: StudentField, values: Vec<FieldValue>) -> Self;
    pub fn where_between(self, field: StudentField, lo: FieldValue, hi: FieldValue) -> Self;
    pub fn where_null(self, field: StudentField) -> Self;
    pub fn where_not_null(self, field: StudentField) -> Self;
    pub fn where_like(self, field: StudentField, pattern: Pattern) -> Self;
    pub fn where_ilike(self, field: StudentField, pattern: Pattern) -> Self;
    pub fn where_has<R, F>(self, relation: R, build: F) -> Self
    where
        R: Into<StudentRelation>,
        F: FnOnce(RelatedQueryBuilder<R>) -> RelatedQueryBuilder<R>;
}
```

Each method is total over the macro-generated field enum; adding a
field requires no manual method wiring.

### Ordering & Pagination

```rust
pub fn order_by(mut self, field: StudentField) -> Self;
pub fn order_by_desc(mut self, field: StudentField) -> Self;
pub fn limit(mut self, n: u32) -> Self;
pub fn offset(mut self, n: u32) -> Self;
pub fn page(self, offset: u32, limit: u32) -> Self;
```

### Eager Loading (`.with`)

```rust
pub fn with(mut self, relation: StudentRelation) -> Self;
pub fn with_many(mut self, relations: &[StudentRelation]) -> Self;
```

`.with(...)` adds the relation to the hydration set. The repository
**must** complete all required joins or batched loads before returning
control to the caller. See "Strict Eager Loading" below.

## Domain Scopes via Extension Traits

The macro provides a structurally complete but semantically neutral
builder. Domain-specific semantic methods (`.active()`, `.in_class()`)
are **not** emitted by the macro. They are implemented in developer-
authored **extension traits**, which keeps macro output deterministic
and isolates business language from generated code.

```rust
// 1. Domain language contract — developer-authored
pub trait StudentQueryScopes {
    fn active(self) -> Self;
    fn in_class(self, class_id: Uuid) -> Self;
    fn admitted_after(self, date: NaiveDate) -> Self;
}

// 2. Extension trait implementation on the macro-generated builder
impl StudentQueryScopes for StudentQueryBuilder {
    fn active(mut self) -> Self {
        self.where_eq(StudentField::Status, StudentStatus::Active)
    }

    fn in_class(mut self, class_id: Uuid) -> Self {
        self.where_eq(StudentField::ClassId, class_id)
    }

    fn admitted_after(mut self, date: NaiveDate) -> Self {
        self.where_gt(StudentField::CreatedAt, date)
    }
}
```

This separation enforces the **Scope Separation Pattern**: macros
generate the engine, humans author the vocabulary.

### Why Extension Traits

- Macro output is fully visible to `rust-analyzer`.
- Domain language is co-located with domain code, not with the
  procedural macro crate.
- Scopes compose via ordinary trait imports; no implicit `import *`
  in user code.
- Scopes are unit-testable in isolation.

## Nested Relational Queries (`where_has`)

Cross-entity relationships cannot be inferred implicitly — the macro
operates on one struct at a time. Relationships are declared via the
`#[query(relation = "Name", builder = "T")]` attribute, and the macro
emits a closure-driven `where_has` method on the parent builder.

### Closure-Based Multi-Entity Filters

```rust
engine
    .students()
    .query()
    .active()
    .where_has(StudentRelation::Parent, |parent_query| {
        parent_query
            .where_eq(ParentField::BillingStatus, BillingStatus::Active)
            .where_eq(ParentField::City, City::Boston)
    })
    .await?;
```

The closure receives the related entity's macro-generated builder
(`ParentQueryBuilder`). The closure body is fully typed. `rust-analyzer`
provides autocomplete on `ParentField::*`, `ParentStatus::*`, and
every macro-generated scope for `Parent`.

### AST Composition

The macro emits a typed bridge between the parent and child ASTs:

```rust
pub enum QueryNode<F: FieldKind> {
    // ...
    HasRelation(<Self as HasRelations>::Relation, Box<QueryNode<RelatedField>>),
}
```

The conversion of this nested node into a structural filter
(`WHERE EXISTS`, `INNER JOIN`, Mongo `$lookup`, etc.) is the
**repository implementation's** responsibility. The macros do not
emit SQL, NoSQL, or any storage-specific construct.

### Forwarding Multiple `where_has`

```rust
let q = engine.students().query()
    .where_has(StudentRelation::Parent, |p| p.active_billing())
    .where_has(StudentRelation::ClassSection, |c| {
        c.where_eq(ClassSectionField::RoomId, room_id)
    });
```

Each `where_has` adds a `HasRelation` node to the parent AST. The
repository composes them as conjunction (`AND`).

## Strict Eager Loading

Lazy loading is **categorically forbidden**. There is no `Option<T>`
async getter, no runtime proxy, no hidden database round-trip on
attribute access.

### The Hard Rules

1. **Zero lazy proxies.** Domain models do not expose asynchronous
   accessors. There is no `.parent().await?` syntax on a hydrated
   `Student`. There never will be.
2. **Hydration markers are explicit.** A `with(StudentRelation::Parent)`
   on the query is the **only** way to populate the `parent` field of
   the returned `Student`. If the field is unhydrated, it remains
   `None` (or empty for `Vec<T>`).
3. **Atomic query phase.** Repositories complete all joins and
   batched secondary queries before returning. Control returns to the
   application layer only after the entire structural graph is
   hydrated.
4. **Compile-time enforcement.** A domain method that consumes a
   `Student` and dereferences `student.parent` without first issuing
   `.with(StudentRelation::Parent)` will produce a runtime `None` —
   not a query. There is no implicit N+1.

### Hydration Workflow

```rust
let students = engine
    .students()
    .query()
    .active()
    // Filter by parent attribute via closure
    .where_has(StudentRelation::Parent, |parent| {
        parent.where_eq(ParentField::BillingStatus, BillingStatus::Active)
    })
    // Command the repository to eagerly load the relation on execution
    .with(StudentRelation::Parent)
    .await?;

// Synchronous, safe, zero-cost interaction with no hidden queries
for student in students {
    if let Some(parent) = &student.parent {
        println!("Loaded Parent: {}", parent.last_name);
    }
}
```

### Distinguishing `where_has` from `with`

| Method         | Purpose                                              | Returns                       |
| -------------- | ---------------------------------------------------- | ----------------------------- |
| `.where_has`   | Filter parent rows by attribute of related row       | None (predicate in AST)      |
| `.with`        | Hydrate the related field on returned rows           | None (hydration directive)   |

These are **independent** operations. Filtering by a relation does
not imply loading it. Loading a relation does not filter by it. Both
operations may be used in the same query.

### Hydration Failure Semantics

If `.with(...)` is requested but the storage adapter cannot perform
the hydration (e.g. due to a missing index or a per-row constraint),
the adapter returns `StorageError::HydrationFailure` and the engine
maps it to `DomainError::Infrastructure`. The application never
receives a partially hydrated graph.

## Repository Integration

Repositories accept the macro-emitted AST and translate it for the
underlying storage. The repository trait itself does not know about
any specific backend.

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

The default storage adapter translates the `QueryNode<StudentField>`
AST into PostgreSQL, SQLite, SurrealDB, or MongoDB execution plans.
The translation is **exhaustive at compile time** because the field
enum has a finite number of variants and the AST is closed.

## Domain-Specific Optimized Queries

Repositories may expose domain-specific queries that bypass the
generic builder when a more efficient path exists:

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

Optimized queries are explicit capabilities, not generic magic, and
**do not** violate the eager-loading rules. They return fully
hydrated aggregates, or fully typed projections, never partial views.

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

Aggregations are `Count`, `Sum`, `Avg`, `Min`, `Max` over numeric
fields. The macro emits the `StudentAggregate` enum alongside the
field enum, ensuring the aggregation set is closed at compile time.

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

The total count is computed by the same AST (with a `count(*)` swap)
so callers can render correct page numbers.

## Tenant Filtering

Every query MUST carry a `SchoolId`. The `StudentQueryBuilder` is
constructed only via `StudentQuery::new(school_id)`. The default
constructor is private. A query that omits the school id is a compile
error. Storage adapters may also enforce row-level security as a
defense in depth.

```rust
let q = StudentQueryBuilder::new(tenant.school_id())
    .where_eq(StudentField::Status, StudentStatus::Active);
```

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
transaction acquire one through the storage port's `Transaction` type
and issue reads and writes inside it. See `docs/ports/storage.md`.

## Error Handling

Repository methods return `Result<T, StorageError>`. The engine maps
storage errors to `DomainError::Infrastructure` and logs them through
`tracing`.

## Performance

- Repositories MUST avoid `SELECT *`. They select only the columns the
  aggregate needs.
- Indexes are owned by the consumer's migrations. The engine documents
  expected indexes per aggregate in `docs/specs/<domain>/indexes.md`.
- Adapters should batch queries where the engine issues many lookups
  in a single command.
- The macro emits a builder whose `Vec` allocations are sized for the
  common case (zero to four predicates). Beyond that, a small-vector
  optimization keeps the builder near-zero-cost.
- The `with(...)` set is internally a `BTreeSet`, so duplicate
  hydration directives are O(log n) and free of side effects.

## Object Safety

The query layer does not require trait objects. The query is a value
type that the engine passes to the storage adapter by value or `&`.

## Testing

- The query builder has unit tests for every operator and edge case.
- Adapters have integration tests against a real database.
- Optimized domain queries have dedicated tests verifying that they
  are used in place of the generic builder where expected.
- Macro expansion is snapshot-tested per domain to detect drift.
- The closure-based `where_has` is tested for AST composition and
  repository translation.
- The `with` directive is tested to confirm: (a) the related field is
  hydrated when requested, (b) the field remains `None` when omitted,
  (c) hydration failures surface as `StorageError::HydrationFailure`.

## Updated Rust Ecosystem Blueprint

| Pattern / Strategy                          | Architectural Decision                          | Justification                                                                 |
| ------------------------------------------- | ----------------------------------------------- | ----------------------------------------------------------------------------- |
| **Preferred**                               | Custom `#[derive(DomainQuery)]` Procedural Macro | Emits deterministic, compile-time verified builders and field enums.          |
| **Preferred**                               | Extension Traits for Scopes                    | Isolates semantic domain logic from automated boilerplate.                   |
| **Preferred**                               | Closure-Based Relational Filters (`where_has`)  | Retains structural safety across multiple domains.                            |
| **Preferred**                               | Explicit `.with()` Directives                  | Enforces predictable, compile-time guaranteed performance; eliminates N+1.    |
| **Avoid**                                   | Declarative `macro_rules!` for query APIs       | Opaque to `rust-analyzer`; hard to maintain.                                 |
| **Avoid**                                   | Implicit Runtime Magic (lazy proxies, reflection) | Hidden I/O violates performance and auditability boundaries.                |
| **Avoid**                                   | Unchecked String Selectors                      | Strings circumvent the compiler, breaking production.                         |
| **Avoid**                                   | Macro-Generated SQL / NoSQL                    | Macros emit AST, not storage syntax. Storage translation lives in adapters.   |
| **Avoid**                                   | Runtime DB Connections from Macros              | Macros are pure compile-time. Adapters own connections.                       |

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

// NEVER
let parent = student.parent().await?;        // no lazy accessor exists
let parent = student.load_parent().await?;   // no async loader exists
student.refresh_parent().await?;             // no implicit refresh exists

// NEVER
let q = engine.students().query()
    .with(StudentRelation::Parent)            // missing where_has - works but inefficient
    .where_eq(StudentField::Status, StudentStatus::Active);  // consider using where_has for filter
```

## Summary

The query layer is:

- Compile-time safe, via macro-generated field enums and state builders.
- Storage-agnostic — adapters translate the AST.
- Domain-friendly — extension traits host business vocabulary.
- Closure-based for nested relational filters.
- Strictly eager — `.with(...)` is the only hydration path; lazy
  proxies and async getters are forbidden.
- Eloquent-like in ergonomics.
- Idiomatic Rust in implementation.
- `rust-analyzer`-friendly — every type the developer touches is
  visible, named, and autocompleteable.
