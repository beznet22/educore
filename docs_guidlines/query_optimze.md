Here is the complete, consolidated set of **Macro Architecture & Optimization Guidelines** for SMSengine. This integrates the structural compile-time safety requirements, closure-based nested relationships, and explicit compile-time eager loading constraints into a cohesive architectural standard.

---

# Optimized Query Layer Architecture Specification

## Macro Generation Strategy

To achieve the ergonomic developer experience of Laravel Eloquent without runtime reflection, schema parsing, or "black-box" macro magic, SMSengine utilizes **Procedural Derive Macros** strictly to generate localized query AST components and builders.

The core of this strategy is the custom `#[derive(DomainQuery)]` macro. When applied to a domain struct, it extracts structural definitions to generate compile-time types.

### Macro Scope & Boundaries

* **AST Generation Only:** Macros **MUST NOT** generate raw SQL, NoSQL, or any database-specific syntax. They are responsible strictly for outputting an intermediate Abstract Syntax Tree (AST) structure representing the query parameters.
* **No Dynamic I/O:** Macros **MUST NOT** perform runtime operations, establish database connections, or parse structural parameters from external files.
* **IDE Autocomplete Alignment:** Macro outputs must be fully visible to `rust-analyzer` by leveraging predictable, typed structures (Enums and Builders) rather than opaque code blocks.
* **Attribute-Driven Opt-in:** Fields are ignored by query generation by default unless decorated with specific query attributes (`#[query(filterable)]`, `#[query(sortable)]`).

---

## Compile-Time Safety & Builder Generation

The query builder relies entirely on macro-generated Enums to avoid unsafe string field manipulation.

### Implementation Standard

Developers define their domain records using the standard `#[derive(DomainQuery)]` macro:

```rust
#[derive(DomainQuery)]
pub struct Student {
    pub id: Uuid,
    
    #[query(sortable)] 
    pub last_name: String,
    
    #[query(filterable)]
    pub status: StudentStatus,
    
    #[query(filterable, relation = "Parent", builder = "ParentQueryBuilder")]
    pub parent_id: Uuid,

    // Hydration targets are excluded from query filter generation
    #[query(ignore)]
    pub parent: Option<Parent>,
}

```

### Macro-Generated Artifacts

From the definition above, the macro automatically outputs two discrete types:

1. **Field Exhaustiveness Enum:**
```rust
pub enum StudentField {
    Status,
    LastName,
    ClassId,
    ParentId,
}

```



```
2. **Type-Safe State Builder:** A `StudentQueryBuilder` that safely collects predicates, orders, and pagination configurations.

### Anti-Patterns vs. Preferred Patterns

```rust
// FORBIDDEN: Runtime parsed strings or raw string match columns
.where("students.status", "active")

// PREFERRED: Strictly typed parameters generated via macro definitions
.where_eq(StudentField::Status, StudentStatus::Active)

```

---

## Domain-Specific Queries (Query Scopes)

To replicate Eloquent's high-level semantic query capabilities (`.active()`, `.in_class()`) without polluting the primary macro generator, SMSengine uses **Extension Traits** implemented on top of the macro-generated builders.

### Scope Separation Pattern

The macro creates the underlying state engine, while developer-defined extension traits implement the domain-specific business language:

```rust
// 1. Definition of the Domain Language Contract
pub trait StudentQueryScopes {
    fn active(self) -> Self;
    fn in_class(self, class_id: Uuid) -> Self;
}

// 2. Concrete Implementation on the Macro-Generated Target
impl StudentQueryScopes for StudentQueryBuilder {
    fn active(mut self) -> Self {
        self.where_eq(StudentField::Status, StudentStatus::Active)
    }
    
    fn in_class(mut self, class_id: Uuid) -> Self {
        self.where_eq(StudentField::ClassId, class_id)
    }
}

```

---

## Nested Relational Queries (`where_has`)

Because procedural macros operate in structural isolation (one struct at a time), cross-entity relationships cannot be inferred implicitly. Relationships must be explicitly declared via attributes to generate typed bridges.

### Closure-Based Multi-Entity Filters

To filter a parent entity using child attributes, the macro evaluates the `relation` and `builder` metadata to emit a closure-driven filtering method on the primary builder.

```rust
engine
    .students()
    .query()
    .active()
    .where_has(StudentRelation::Parent, |parent_query| {
        parent_query.where_eq(ParentField::BillingStatus, BillingStatus::Active)
    })
    .await?;

```

### Behind the Scenes: AST Composition

The closure safely binds the secondary entity's macro-generated builder (`ParentQueryBuilder`), providing compile-time assurance and IDE code completion. The output evaluates into an AST relationship variant:

```rust
pub enum QueryNode {
    Eq(StudentField, Value),
    HasRelation(StudentRelation, Box<QueryAST>), // Bundles nested AST payload
}

```

The conversion of this nested node into structural filters (e.g., SQL `WHERE EXISTS`, `INNER JOIN`, or Mongo `$lookup`) remains strictly inside the domain of the respective **Repository Implementation**.

---

## Data Hydration & Strict Eager Loading

SMSengine **categorically outlaws lazy loading**. To eliminate N+1 latency degradations at compile time, domain models are defined as plain, decoupled structures without underlying database connections or network hooks.

### Eager Loading Execution Framework

* **Explicit Hydration Markers:** Filtering by a relation (`where_has`) and populating a relation (`with`) are separated into two distinct query steps. Related data is only fetched if requested using `.with()`.
* **Zero Lazy Proxies:** Domain models must never expose asynchronous getters or locks to read missing properties. If data is not hydrated, the `Option<T>` or `Vec<T>` property remains `None` or empty.
* **Atomic Query Phase:** Repositories must complete all joint selections or batched queries to entirely hydrate the resulting structural graphs before returning execution control to the application layer.

### Consolidated Execution Spec

```rust
let students = engine
    .students()
    .query()
    .active()
    // 1. Filter constraint applied across structural bound via Closure
    .where_has(StudentRelation::Parent, |parent| {
        parent.where_eq(ParentField::BillingStatus, BillingStatus::Active)
    })
    // 2. Explicitly command repository to execute eager load on execution
    .with(StudentRelation::Parent)
    .await?;

// Usage: Synchronous, safe, zero-cost interaction with no hidden queries
for student in students {
    if let Some(parent) = &student.parent {
        println!("Loaded Parent: {}", parent.last_name);
    }
}

```

---

## Updated Rust Ecosystem Blueprint

| Pattern / Strategy | Architectural Decision | Justification |
| --- | --- | --- |
| **Preferred** | Custom `#[derive]` Procedural Macros | Emits deterministic, compile-time verified builders and structural enums. |
| **Preferred** | Extension Traits for Scopes | Isolates semantic domain logic from automated boilerplate generators. |
| **Preferred** | Closure-Based Relational Filters | Retains complete structural safety across multiple domains. |
| **Preferred** | Explicit `.with()` Directives | Enforces predictable, compile-time guaranteed performance and prevents memory safety escapes. |
| **Avoid** | Declarative `macro_rules!` for APIs | Overly complex to maintain, poorly supported by standard language servers (`rust-analyzer`). |
| **Avoid** | Implicit Runtime Magic | Hidden database transactions, implicit reflection pools, and lazy proxies violate performance boundaries. |
| **Avoid** | Unchecked String Selectors | Standard strings circumvent the compiler, triggering production-stage breakage. |