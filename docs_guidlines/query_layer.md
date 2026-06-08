# Additional Architecture Requirements

## Query Layer Philosophy

SMSengine must provide a lightweight, high-performance, fully type-safe query layer.

The goal is developer ergonomics similar to Laravel Eloquent's query builder while remaining idiomatic Rust.

This is NOT:

* an ORM
* an Active Record implementation
* a schema introspection engine
* a runtime reflection system
* a dynamic query builder

This IS:

* a compile-time type-safe query builder
* a thin abstraction over storage adapters
* optimized for domain repositories
* optimized for business workflows
* optimized for Rust performance characteristics

---

## Design Goals

Consumers should be able to write expressive queries:

```rust
engine
    .students()
    .query()
    .where_eq(StudentField::Status, StudentStatus::Active)
    .where_eq(StudentField::ClassId, class_id)
    .order_by(StudentField::LastName)
    .paginate(page)
    .await?;
```

or

```rust
engine
    .students()
    .query()
    .active()
    .in_class(class_id)
    .await?;
```

The API should feel familiar to Eloquent users while remaining fully typed.

---

## Compile-Time Safety

The query layer must provide:

* compile-time field definitions
* compile-time operators
* compile-time sorting
* compile-time filtering
* compile-time pagination

Avoid:

* string field names
* runtime schema parsing
* runtime reflection
* dynamic column discovery

Forbidden:

```rust
.where("students.status", "active")
```

Preferred:

```rust
.where_eq(StudentField::Status, StudentStatus::Active)
```

---

## Schema Strategy

Do NOT generate queries from database schema inspection.

Do NOT parse migrations at runtime.

Do NOT depend on reflection.

Instead:

* define fields using Rust types
* generate builders using derive macros where appropriate
* keep query construction explicit and predictable

---

## Repository Integration

The query builder exists to support repositories.

Example:

```rust
pub trait StudentRepository {
    async fn query(
        &self,
        query: StudentQuery
    ) -> Result<Vec<Student>>;
}
```

Repository implementations remain responsible for translating query definitions into:

* PostgreSQL
* SQLite
* SurrealDB
* MongoDB

specific execution plans.

---

## Domain-Specific Queries

Repositories may expose optimized domain queries.

Example:

```rust
students()
    .active_in_term(term_id)
    .await?;
```

or

```rust
attendance()
    .missing_for_date(date)
    .await?;
```

These should be modeled as explicit domain capabilities rather than forcing all logic through generic query operators.

---

## Query Builder Documentation

Documentation agents must create specifications describing:

* query capabilities
* filtering capabilities
* sorting capabilities
* pagination capabilities
* aggregate queries
* reporting queries
* repository responsibilities

The documentation must define behavior and contracts, not implementation details.

---

## Rust Ecosystem Standards

Architecture decisions must follow modern Rust practices.

Preferred:

* traits for ports
* builders where appropriate
* derive macros for ergonomics
* strongly typed identifiers
* strongly typed value objects
* explicit error handling
* async traits where necessary

Avoid:

* Java enterprise patterns
* dependency injection containers
* service locators
* reflection systems
* runtime metadata registries
* excessive generic abstractions
* unnecessary macro magic

SMSengine should feel like a modern Rust crate rather than a framework.

---

## Performance Requirements

The query layer should be:

* allocation conscious
* async friendly
* zero-reflection
* zero runtime schema inspection
* storage-agnostic
* suitable for embedded SQLite deployments
* suitable for large PostgreSQL deployments

The architecture should prioritize simplicity and correctness before abstraction.

---

## Production Engineering Standards

SMSengine is a production system.

All generated architecture and specifications must assume:

* real schools
* real students
* real financial records
* real attendance records
* real audit requirements

Avoid toy-project architecture.

Avoid tutorial-grade implementations.

Design for maintainability, correctness, portability, and long-term evolution.

---

## Repository Analysis Rules

When analyzing Schoolify:

DO extract:

* business workflows
* business rules
* permissions
* validation requirements
* reporting requirements
* lifecycle transitions
* domain terminology

DO NOT extract:

* PHP implementations
* controller structures
* service structures
* repository structures
* Eloquent usage
* framework conventions
* code organization

The objective is domain discovery, not source code migration.
