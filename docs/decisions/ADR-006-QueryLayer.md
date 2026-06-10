# ADR-006: Compile-Time-Safe Query Layer (Macro Architecture)

## Status

Accepted.

## Context

The school domain has hundreds of aggregates. The consumer's UI needs
to ask questions like:

- "List active students in class 7B, sorted by last name, paginated."
- "Show me invoices where the due date is in the past and the balance
  is greater than zero."
- "Find all staff members whose leave request is pending."

These questions are domain-specific. They are not generic "SQL where
clauses"; they are "domain queries" expressed in the engine's
ubiquitous language.

The naive approaches are dangerous in a multi-tenant, type-safe engine:

- **Raw SQL**: bypasses the type system, makes refactoring hard, and
  exposes the engine to SQL injection.
- **String field names** (`.where("students.status", "active")`): a
  typo is a runtime error; a rename is a search-and-replace exercise;
  cross-tenant leakage is one forgotten `WHERE school_id = ?` away.
- **Hand-written `*Field` enums** that must be kept in sync with the
  struct: drift is inevitable; the field list becomes out of date the
  moment a column is added.
- **Schema introspection or reflection**: violates the engine's
  `ADR-012: No Reflection` policy and produces O(n) hidden I/O.
- **Lazy-loading runtime proxies**: introduce N+1 by design and
  produce non-deterministic query plans at the worst possible time
  (production, on the hot path).

The "Eloquent-like" approach — chainable methods with string-typed
fields — is what most ORMs offer. It is ergonomic but loses type
safety.

## Decision

Educore adopts a **compile-time-safe, macro-generated query layer**
with **closure-based nested filters** and **strict eager loading**.

Concretely:

1. **Every aggregate derives `#[derive(DomainQuery)]`.** The procedural
   macro reads the struct definition at compile time and emits
   deterministic typed artifacts. There is no runtime introspection
   and no manual field-list maintenance.
2. **The macro emits three discrete types** per aggregate:
   - A field-exhaustiveness enum (`StudentField`) listing only fields
     marked `#[query(filterable)]` or `#[query(sortable)]`.
   - A typed state builder (`StudentQueryBuilder`) that accumulates
     predicates, ordering, pagination, and hydration directives.
   - A relation enum (`StudentRelation`) for fields decorated with
     `#[query(relation = "Name", builder = "T")]`.
3. **The macro is strictly an AST generator.** It MUST NOT emit
   SQL, NoSQL, or any storage-specific syntax. It MUST NOT perform
   runtime I/O or open database connections. The conversion from
   AST to storage execution plan is the storage adapter's
   responsibility.
4. **Field values are strongly typed.** A `StudentField::Status`
   accepts a `StudentStatus`, not a `&str`. A
   `InvoiceField::Balance` accepts a `Money`, not an `f64`.
5. **Attribute-driven opt-in.** A field is queryable only when the
   developer marks it with `#[query(filterable)]` or
   `#[query(sortable)]`. Undecorated fields are excluded from the
   field enum and the builder. Hydration targets are excluded by
   `#[query(ignore)]`.
6. **Domain scopes are extension traits.** Developer-authored
   extension traits (e.g. `StudentQueryScopes`) implement
   semantic methods (`.active()`, `.in_class()`) on top of the
   macro-generated builder. The macro does not emit domain
   language; humans do. This keeps macro output deterministic and
   isolates business vocabulary.
7. **Cross-entity filters use closures.** `where_has(relation, |q| ...)`
   binds the related entity's macro-generated builder inside a
   closure. The closure body is fully typed, with `rust-analyzer`
   autocomplete, and emits a single `HasRelation` AST node that the
   repository translates into a join, `WHERE EXISTS`, or `$lookup`.
8. **Eager loading is explicit and mandatory.** `.with(relation)`
   declares a hydration directive. The repository MUST complete the
   join or batched secondary load before returning. There are no
   async getters on domain models, no lazy proxies, no implicit
   refresh on attribute access.
9. **The builder carries the active `TenantContext` from
   construction.** A `StudentQueryBuilder` cannot be built without
   a `SchoolId`.
10. **No string field names, no `serde_json::Value`, no raw
    column names in consumer code.** A `.where("status", ...)` is a
    compile error. A `.parent().await?` on a hydrated `Student`
    is a syntax error.
11. **Pagination is part of the builder**, with stable cursor or
    page-based semantics. A `Page` value object is the only accepted
    input.
12. **Repositories MAY expose domain-specific optimized queries**
    that bypass the builder for performance (e.g.
    `student_repository.find_active_for_class(class_id)`). Optimized
    methods are still strongly typed, still carry the
    `TenantContext`, and still return fully hydrated graphs.

The query layer is documented in `docs/query_layer.md` and in the
per-domain `repositories.md` files.

## Consequences

### Positive

- **Field-name typos are compile errors.** A consumer writing
  `StudentField::Statuz` fails to compile.
- **Type mismatches are compile errors.** A consumer passing a
  `String` for a `DateTime` field fails to compile.
- **Tenant binding is mandatory.** A query cannot be built without
  a `SchoolId`.
- **Drift between struct and field enum is impossible.** The
  macro regenerates the enum from the source of truth on every
  compile.
- **The query layer is its own documentation.** The `StudentField`
  enum is the list of queryable fields; reading it tells the
  consumer what they can ask.
- **Refactoring is safe.** Renaming a field ripples through the
  type system; consumers are forced to update.
- **No SQL injection.** The macro emits a typed AST; the consumer
  never assembles a string.
- **No N+1.** Lazy accessors do not exist. Hydration is explicit
  via `.with(...)`, and the repository completes the join before
  returning.
- **No runtime reflection.** The macro runs at compile time; the
  emitted builders carry no reflection metadata.
- **Domain scopes are isolated.** Extension traits are unit-testable
  in isolation; the macro generator remains a pure compile-time
  tool.
- **`rust-analyzer` autocomplete works end-to-end.** Every type the
  developer touches — field enum, relation enum, scope method —
  is visible, named, and completable.
- **Domain-specific optimized queries are still type-safe.** A
  repository method `find_active_for_class(class_id)` is checked
  at compile time; the consumer cannot pass the wrong type.

### Negative

- **The macro is a procedural-macro crate.** A new crate is added
  to the workspace. Build times increase by the macro expansion
  cost.
- **The builder is verbose compared to Eloquent.** A query that
  would be one line of Eloquent is several lines in the engine.
  The tradeoff is intentional.
- **Domain-specific queries are sometimes faster than the
  builder.** The builder is generic; an optimized repository
  method can use a covering index. The builder is the floor, not
  the ceiling.
- **Consumers cannot write ad-hoc SQL.** For edge cases (e.g.
  complex reports), the consumer uses the `Report.Generate`
  command, which is itself capability-gated.
- **Extension traits must be implemented per aggregate.** A scope
  trait is required for each semantic vocabulary.

### Mitigations

- The macro crate (`educore-query-derive`) is itself cargo-cacheable
  and incremental.
- Builder trait provides common methods (`where_eq`, `where_in`,
  `order_by`, `limit`, `paginate`) that work for any aggregate.
- An `IndexHint` mechanism lets the consumer (or the storage
  adapter) declare which index a query should use, without changing
  the type.
- The `Report.Generate` command handles ad-hoc analytics in a
  typed, capability-gated way.
- A scope-trait convention is published so that the boilerplate
  per aggregate is uniform.

## Alternatives Considered

### 1. Raw SQL with parameter binding

The consumer writes SQL. Rejected because it bypasses the type
system, makes refactoring hard, and exposes the engine to SQL
injection.

### 2. Eloquent-like string-typed field names

The consumer writes `query.where("first_name", "Anita")`. Rejected
because typos are runtime errors and refactoring is unsafe.

### 3. ORM with `#[derive(Queryable)]` and reflection

The consumer marks a struct queryable; the ORM infers fields at
runtime. Rejected because the engine's code standards forbid
runtime reflection (see `ADR-012: No Reflection`).

### 4. Hand-written `*Field` enums per aggregate

The developer maintains `StudentField` and `StudentQuery` by hand.
Rejected because drift between the struct and the field enum is
inevitable, and the macro eliminates the entire class of drift
bugs.

### 5. GraphQL

The consumer writes a GraphQL query. Rejected because GraphQL is a
wire format, not a query language that maps to the engine's
aggregates. The consumer can expose a GraphQL API on top of the
engine's query layer; the engine itself is not GraphQL.

### 6. No query layer; consumer reads from raw tables

The consumer writes its own SQL. Rejected because it bypasses every
invariant the engine has.

### 7. Stored procedures

Database-stored domain logic. Rejected because the engine does not
own a specific database, and because business rules belong in
Rust, not in DDL.

### 8. Lazy-loading runtime proxies

The ORM exposes async getters that fetch on first access.
Rejected because it makes query plans non-deterministic, hides
N+1 patterns, and violates the engine's "no implicit I/O"
principle. The engine enforces strict eager loading via `.with(...)`
and forbids async getters on domain models.
