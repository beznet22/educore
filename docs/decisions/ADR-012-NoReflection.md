# ADR-012: No Reflection, No Schema Parsing, No Lazy Loading

## Status

Accepted.

## Context

The school domain has hundreds of aggregates. The
tempting shortcut is to make the engine "discoverable"
through runtime mechanisms:

- **Reflection** — at runtime, the engine reflects on
  the aggregate's struct, finds its fields, and
  exposes them as queryable, serializable, validatable.
- **Schema parsing** — at runtime, the engine reads
  a schema file (JSON Schema, Protobuf, GraphQL SDL)
  and generates the type definitions, validators, and
  query builders.
- **Lazy loading** — at runtime, the engine wraps
  domain models in proxies that fetch related
  entities on first attribute access, or hides
  related entities behind `dyn Any` payloads that
  downcast on demand.

All three approaches are popular in dynamic-language
ecosystems (Eloquent, Django ORM, TypeORM, ActiveRecord,
Hibernate). They are ergonomic: the developer
writes a struct, and the framework does the rest.

The Rust type system, by contrast, asks the developer
to write the type definitions **and** the framework
glue. The boilerplate is real.

The cost of the dynamic approaches in a
production-critical engine is also real:

- **Refactoring is unsafe.** Renaming a field does
  not update the schema. The next migration may
  silently break the runtime discovery.
- **Errors are runtime errors.** A field with the
  wrong type is caught only when the field is
  queried, not at build time. A lazy proxy with a
  null connection is caught only when a request
  hits the broken getter.
- **Performance is unpredictable.** Reflection
  involves string lookups, hash maps, and runtime
  allocation. Lazy loading introduces N+1 query
  patterns on the hot path. The engine's attendance
  marking for 1,000 students cannot afford either.
- **Tooling is limited.** A consumer's IDE cannot
  autocomplete a `StudentField::FirstName` if
  `StudentField` is generated at runtime or hidden
  behind a proxy that resolves it on access.
- **Cross-platform is fragile.** Runtime reflection
  on WASM, on embedded, on iOS, on Android behaves
  differently. Lazy proxies that depend on
  thread-local state behave differently across
  executors. The engine cannot rely on either.
- **Auditing is harder.** A field that the engine
  silently added at startup is not in the
  documentation. A proxy that opens a connection
  on first read is invisible in the static call
  graph.

The school's domain is also stable. The aggregates
do not change every sprint. The boilerplate cost
is paid once per aggregate, not per release.

The solution is not "live with the boilerplate" but
"let a compile-time tool generate the boilerplate."
Procedural derive macros (`#[derive(DomainQuery)]`,
`#[derive(Command)]`, `#[derive(DomainEvent)]`) emit
typed artifacts at compile time, with the same
ergonomics as a runtime reflection framework and
none of the runtime cost. They are the canonical
mechanism by which Educore avoids reflection in
the query layer. The macro is preferred over
declarative `macro_rules!` for the query DSL
precisely because the derive macro's output is a
named, typed item visible to `rust-analyzer`,
whereas a `macro_rules!` arm is an opaque token
transformation the language server cannot
introspect.

## Decision

Educore uses **no runtime reflection**, **no
runtime schema parsing**, and **no lazy loading**.
The engine's structure is encoded in Rust types,
derives, and macros — all resolved at compile time.

Concretely:

 1. **Every aggregate is a Rust struct.** Its fields
    are concrete types. No `serde_json::Value` in
    domain code.
 2. **Every aggregate derives
    `#[derive(DomainQuery)]`.** The procedural
    macro reads the struct definition at compile
    time and emits a deterministic, typed
    `*Field` enum, a typed `*QueryBuilder` state
    machine, and (where the developer has marked a
    relation) a `*Relation` enum. The enum is not
    hand-maintained; drift is impossible.
 3. **The macro is strictly an AST generator.** It
    MUST NOT emit SQL, NoSQL, Mongo, or any other
    storage-specific syntax. It MUST NOT open
    database connections, read connection
    strings, or perform any I/O. The conversion
    from the macro's AST output into an execution
    plan is the storage adapter's responsibility,
    not the macro's. The macro's full boundary is
    defined in the "Macro Boundary" section below.
 4. **Every event is a typed Rust struct** with
    `#[derive(Serialize, Deserialize)]`. The schema
    is the type.
 5. **Every command is a typed Rust struct** with
    `#[derive(Command)]`. The macro generates the
    capability list, the type name, the validation
    hooks.
 6. **The capability catalog is a Rust enum**
    (`enum Capability { ... }`). Stringly-typed
    capabilities are forbidden.
 7. **The schema registry is populated at compile
    time.** A build script reads the engine's
    events and commands and emits a JSON catalog
    used for documentation and runtime discovery
    (e.g. an AI agent asking "what commands
    exist?"). The catalog is generated, not
    parsed at runtime.
 8. **No `serde_json::Value` in domain code.** All
    JSON is `serde`'s derived serialize / deserialize.
    A `serde_json::Value` in a domain struct,
    command, event, or query input is a code-review
    blocker.
 9. **No `inventory`, no `linkme`, no
    runtime-registration crates.** Crates that
    discover each other at runtime are forbidden
    in domain code. The macro does not register
    anything at runtime; its registration is the
    derive site in the source file.
10. **No service locators, no DI containers, no
    `lazy_static`, no `once_cell` outside narrow
    technical needs.** Construction is explicit.
11. **No lazy loading.** Domain models do not carry
    runtime proxies, async getters, `dyn Any`
    payloads, or transparent collection wrappers.
    Related entities are either hydrated by the
    repository before the result is returned or
    absent (`Option::None`, empty `Vec`). The
    consumer reads domain models synchronously.
    The full prohibition is in the "Lazy Loading
    Forbidden" section below.
12. **A small set of derive macros and procedural
    macros is allowed**, but they are compile-time
    only. They do not introduce runtime
    registration. Declarative `macro_rules!` is
    rejected for the query DSL; the derive macro
    is preferred because its output is typed and
    visible to `rust-analyzer`.

The engine's type system is the documentation. A
developer can navigate the code with `rust-analyzer`
and discover every capability, every command, every
field, every event. The query layer's full
specification is in `docs_guidlines/query_optimze.md`
and in `ADR-006: Compile-Time-Safe Query Layer`.

## Macro Generation Strategy

Educore's answer to the boilerplate cost of typed
Rust is the **`#[derive(DomainQuery)]` procedural
macro**. The macro is the canonical, compile-time
mechanism for eliminating reflection from the query
layer.

When applied to a domain struct, the macro extracts
the structural definition at compile time and emits
a deterministic, typed query surface:

```rust
#[derive(DomainQuery)]
pub struct Student {
    pub id: Uuid,

    #[query(sortable)]
    pub last_name: String,

    #[query(filterable)]
    pub status: StudentStatus,

    #[query(filterable, relation = "Parent",
            builder = "ParentQueryBuilder")]
    pub parent_id: Uuid,

    // Hydration target; excluded from query surface.
    #[query(ignore)]
    pub parent: Option<Parent>,
}
```

The macro emits three discrete typed artifacts:

 1. **A field-exhaustiveness enum** listing only
    fields marked `#[query(filterable)]` or
    `#[query(sortable)]`. Renaming the struct's
    field is a rename of the enum variant; the
    compiler forces every call site to update.
 2. **A typed state builder** that accumulates
    predicates, ordering, pagination, and
    hydration directives. The builder's methods
    consume `self` and return `Self`; state is
    type-checked at every step.
 3. **A relation enum** for fields decorated with
    `#[query(relation = "...", builder = "...")]`,
    used by `where_has` and `.with()` to bind a
    related aggregate's macro-generated builder.

The macro is **attribute-driven and opt-in**. A
field is included in the query surface only when
the developer marks it explicitly. Undecorated
fields are excluded. Hydration targets are excluded
by `#[query(ignore)]`. There is no implicit
"every field is queryable" mode; the developer
controls the query surface, attribute by attribute.

Domain scopes (`.active()`, `.in_class()`) are
**extension traits** implemented on top of the
macro-generated builder. The macro does not emit
domain language; humans do. This keeps the macro
output deterministic and isolates business
vocabulary from the auto-generated surface.

Cross-entity filters use **closure-based
relational filters**:
`where_has(StudentRelation::Parent, |q| { ... })`.
The closure body is fully typed; the macro binds
the related entity's builder into the closure
parameter; the AST output is a single
`HasRelation` node.

Hydration is **explicit and batched**. The
consumer declares `.with(relation)`; the
repository completes all joint selections and
batched secondary loads before returning. There
is no implicit fetch on access.

The full architecture is defined in
`docs_guidlines/query_optimze.md`; the
in-engine contract is in
`ADR-006: Compile-Time-Safe Query Layer`.

## Macro Boundary

The `#[derive(DomainQuery)]` macro is
**compile-time, pure, and deterministic**. Its
input is the source code. Its output is a set of
typed items appended to the crate. The output is
the same on every machine, on every build, with
no dependency on environment.

The macro MUST NOT:

- Generate raw SQL, NoSQL, Mongo query syntax, or
  any storage-specific syntax. Storage adapters
  own the AST-to-plan translation.
- Open database connections, allocate connection
  pools, or read connection strings.
- Read external schema files, configuration,
  JSON manifests, Protobuf definitions, GraphQL
  SDL, OpenAPI documents, or environment
  variables.
- Touch the file system, the network, the clock,
  or any other source of non-determinism.
- Spawn threads, schedule tasks, or hold state
  across invocations.
- Emit `inventory`, `linkme`, `lazy_static`,
  `once_cell`, or any other runtime-registration
  primitive.
- Emit `serde_json::Value`, `Box<dyn Any>`,
  `Box<dyn Error>`, or other escape-hatch types
  in the generated surface.
- Perform reflection on the surrounding crate,
  scan the file system, or enumerate sibling
  modules.

In short: the macro's job is to read a struct's
syntactic definition and emit typed items. It is
a compile-time AST generator. It is not a
runtime, not an I/O layer, and not a discovery
mechanism.

This boundary is what makes the macro safe to
use in deterministic, reproducible builds, in
CI, in offline builds, and in audited
environments. A macro that opens a connection
or reads a config file is not a macro the engine
will accept.

The `#[derive(Command)]`,
`#[derive(DomainEvent)]`, and related derives
follow the same boundary. The macro emits
typed artifacts; the artifact's consumer is
responsible for I/O.

## Lazy Loading Forbidden

Educore categorically outlaws lazy loading. The
engine is a strict eager-loading system; N+1
patterns are a correctness failure, not a
performance optimization. The following patterns
are rejected in domain code:

- **Runtime proxies.** Domain models do not wrap
  themselves in a proxy that intercepts attribute
  access and triggers a fetch. A `Student` is a
  `Student`; accessing `student.parent` returns
  the `Option<Parent>` that the repository put
  there.
- **Async getters on domain models.** There is no
  `async fn parent(&self) -> Result<Parent>` on a
  domain model. Domain models are synchronous,
  decoupled, and side-effect free. An `async`
  method on a domain type is a code-review
  blocker.
- **`dyn Any` payloads.** Domain models do not
  carry an `Any` slot that downcasts to a related
  entity. `Box<dyn Any>`, `Box<dyn Trait>` where
  `Trait: Any`, and `serde_json::Value` are all
  rejected for the same reason: the type system
  is the boundary, not a runtime trait object or
  an `Any` downcast.
- **Implicit refresh on access.** Accessing a
  field does not trigger a network call, a
  database round-trip, or a cache lookup. If a
  field is `Option<T>`, the value is `None` until
  the repository has populated it; it does not
  "load on demand."
- **Transparent collection wrappers.** A `Vec<T>`
  that appears empty until iterated, or that
  fires a query on the first `next()` call, is a
  hidden N+1 trap. The collection is either
  populated by the repository before return or
  it is empty.

Hydration is **explicit and batched**. The
consumer declares `.with(relation)`; the
repository completes all joint selections and
batched secondary loads before returning control
to the application layer. The domain model is
the result; the consumer reads it synchronously
in a `for` loop, a `match`, or a `let Some(...)`.

A domain model whose fields trigger I/O is not
a domain model; it is a proxy. Educore has no
proxies.

## Consequences

### Positive

- **Refactoring is safe.** Renaming a field
  ripples through the type system; the compiler
  tells the developer every call site that
  needs to change.
- **Errors are compile errors.** A typo in a
  field name, a wrong type, a missing capability
  — all caught at build time.
- **Performance is predictable.** No string
  lookups in the hot path. No N+1 patterns on
  the hot path. The generated code is what the
  developer wrote.
- **Tooling is excellent.** `rust-analyzer`
  autocompletes `StudentField::FirstName`; the
  consumer's IDE can navigate to the field's
  definition. The macro's output is named, typed,
  and refactor-safe.
- **Cross-platform is uniform.** WASM, iOS,
  embedded — the type system is the same
  everywhere. The macro has no platform-dependent
  behavior.
- **Auditing is structural.** Every capability
  the engine knows about is a variant of the
  `Capability` enum. There is no "shadow"
  capability that the engine added at startup
  without the documentation.
- **Compile-time type safety = runtime
  correctness.** A consumer cannot pass a wrong
  type; the engine cannot be tricked by a
  malformed payload; the macro cannot be tricked
  by a runtime mutation of the struct.
- **No N+1.** Hydration is explicit; the
  repository is the only place that knows which
  joins to run; the consumer cannot accidentally
  fetch one row per iteration.

### Negative

- **Boilerplate.** Every aggregate has a
  `*Field` enum (macro-generated), a
  `*Repository` trait, a `*Service`, a
  `*Command` set. The macros reduce the
  boilerplate but do not eliminate it.
- **No "add a field, get a query for free."**
  The developer writes the field, marks it
  `#[query(filterable)]` or
  `#[query(sortable)]`, writes the scope method
  (if any), writes the capability (if any),
  writes the test. This is intentional.
- **Slower initial development.** The
  prototype is bigger; the engine is more
  code to start with. The trade is a
  faster, safer, more refactorable long
  term.
- **Some patterns are awkward.** A consumer
  who wants "give me a list of every
  command in the engine" must use the
  schema registry (a generated JSON file),
  not a runtime reflection call.
- **A consumer who wants "give me a parent
  on demand" must refactor to `.with(...)`.
  The consumer cannot sneak a lazy load past
  the type system. This is intentional.

### Mitigations

- The `educore-query-derive` crate provides
  `#[derive(DomainQuery)]` (and is the only
  proc-macro crate in v1; additional derives
  are added in subsequent phases), which
  reduces the per-aggregate boilerplate to a
  few lines.
- A `educore-cli` tool reads the
  generated schema registry and prints
  the catalog for the consumer.
- The `build-plan.md` describes how the
  derive macros are built up over the
  phased delivery.
- The `code-standards.md` documents the
  "no reflection" rule with examples.
- The `docs_guidlines/query_optimze.md`
  document defines the macro architecture
  in full, with worked examples for the
  preferred and avoided patterns.
- A scope-trait convention is published so
  that the boilerplate per aggregate is
  uniform; the macro does not invent
  domain language, but the convention
  keeps the human-authored boilerplate
  small and predictable.

## Alternatives Considered

### 1. ORM-style reflection

The framework reads the struct at runtime,
infers the table, the fields, the queries. The
popular implementations of this style — Eloquent,
Django ORM, TypeORM, ActiveRecord, Hibernate,
`#[derive(Queryable)]` crates that introspect
at runtime — are rejected per above.

The `#[derive(DomainQuery)]` macro is the
compile-time counterpart: same developer
ergonomics (mark the field, the macro emits
the surface), no runtime cost, no hidden
discovery, no N+1. The trade is "the macro
runs in the developer's `cargo build`" instead
of "the framework runs in the request path."

### 2. Schema-first with JSON Schema or
Protobuf

The developer writes a schema file; the
framework generates the Rust types.
Powerful for cross-language APIs; rejected
for the engine's internal use because the
schema and the type would drift, and the
build step would couple the engine to a
schema language.

### 3. Runtime registration with
`inventory` or `linkme`

Crates register at startup; the engine
discovers them. Rejected because the
engine's domain crates should not depend
on a global registration mechanism. The
`Engine::builder()` API is explicit.

### 4. `serde_json::Value` everywhere

"Flexible" data with runtime validation.
Rejected per code-standards; loses type
safety, makes refactoring unsafe, and
gives the consumer a runtime backdoor
into the domain.

### 5. `Any` and downcasting

Generic escape hatches. Rejected because
they hide the domain behind `Any` and
defeat the type system. `Box<dyn Any>` and
downcast-on-access are a particularly
egregious form of lazy loading and are
rejected on the same grounds as runtime
proxies.

### 6. WebAssembly component model
(wit, wit-bindgen)

Cross-language interfaces. Not relevant
for the engine's internal Rust-only
domain; relevant for the consumer's
FFI / WASM surface, which the engine
does not own.

### 7. Declarative `macro_rules!` for the
query DSL

A consumer-facing query DSL expressed as
`macro_rules!`. Rejected for the query
layer because declarative macros produce
opaque, token-shaped output that
`rust-analyzer` cannot introspect, cannot
autocomplete, and cannot rename.

A typo inside a `macro_rules!` arm is a
runtime error. A consumer who writes
`active!()` cannot navigate to its
definition; the IDE shows the macro
invocation, not the generated code.
Refactoring a query surface built on
`macro_rules!` is a search-and-replace
exercise across the codebase.

The `#[derive(DomainQuery)]` macro is
preferred precisely because its output is
a named, typed item in the type system.
The field enum is a real enum; the
builder is a real struct; `rust-analyzer`
autocompletes both. The macro's expansion
is visible to refactoring tools in a way
that `macro_rules!` expansion is not.
