# ADR-011: Rust Ecosystem Standards

## Status

Accepted.

## Context

Educore is a Rust crate ecosystem. The Rust ecosystem has
strong conventions: edition, MSRV, async runtime, error
handling, serialization, logging, observability, testing,
dependency management. The engine's standards must be
specific, not aspirational, because a vague standard
allows drift.

The school domain also has its own performance
characteristics. A daily attendance marking for 1,000
students happens in a 30-minute window. A payroll for 200
staff happens once a month. A report card for 1,000
students happens at the end of each term. The engine's
standards must be fast enough to handle these peaks
without tuning, on a small server.

The school domain also has its own correctness
characteristics. Money must not have rounding errors.
PII must not be logged. Time zones must not corrupt
timestamps. Identifiers must not be confused. The engine's
standards must encode these invariants.

The query layer, in particular, must produce deterministic,
typed query surfaces that are visible to `rust-analyzer`
from first compile. The macro architecture defined in
`docs_guidlines/query_optimze.md` and
`ADR-006: Compile-Time-Safe Query Layer` is the
authoritative reference; this ADR codifies the
ecosystem patterns that make that architecture work
— typed identifiers, value objects, traits for
ports, builders, and the `#[derive(DomainQuery)]`
macro family.

## Decision

Educore adopts the **Rust ecosystem's modern standards**
as the engineering baseline. These standards are
mandatory for every crate in the engine.

 1. **Edition:** 2021.
 2. **MSRV:** 1.75 (the latest stable LTS at the time of
    release). Crate-level `rust-version` is set.
 3. **Async:** `tokio` is the runtime. The
    `educore-core` crate re-exports `tokio` selectively
    so domain code does not import `tokio` directly.
 4. **Error handling:** `thiserror` for public error
    types, `anyhow` for glue code (CLI, scripts). Domain
    errors are enums with `#[from]` conversions.
 5. **Serialization:** `serde` is the framework. Domain
    events and commands use `#[derive(Serialize,
    Deserialize)]`. `serde_json::Value` is forbidden in
    domain code; typed wrappers are mandatory.
 6. **Logging:** `tracing` for structured logs.
    `tracing-subscriber` is the only sanctioned
    subscriber. Sensitive data (PII, money values, auth
    tokens) is never logged.
 7. **Testing:** `cargo test`, `#[tokio::test]` for
    async tests, `proptest` for property tests,
    `insta` for snapshot tests, `criterion` for
    benchmarks.
 8. **Linting:** `cargo clippy --workspace --all-targets
    -- -D warnings` is the bar. `cargo fmt` is enforced.
 9. **Documentation:** Every public item has a
    rustdoc comment. Crate-level `lib.rs` describes
    what the crate owns and what it depends on.
10. **Dependencies:** Minimum, audited, pinned. The
    workspace's `Cargo.lock` is committed. Dependencies
    are reviewed per PR.
11. **Numeric conversions:** `TryFrom` / `TryInto`.
    `as` on numerics is forbidden.
12. **Unsafe:** Forbidden in domain code. Adapters may
    use it only with a `// SAFETY:` comment and
    review.
13. **Panics:** `unwrap()`, `expect()`, `panic!` are
    forbidden in production paths. Domain code
    returns `Result`; tests may use `unwrap` or
    `expect` because a failing test is a panic by
    design.
14. **Cross-compilation:** `rustls` instead of
    `native-tls`. Builds verified for `x86_64-unknown-
    linux-gnu`, `aarch64-unknown-linux-gnu`,
    `x86_64-apple-darwin`, `x86_64-pc-windows-msvc`.
15. **Async-friendly APIs:** All I/O is async. Domain
    logic is sync wherever possible. No blocking I/O
    inside an async function.
16. **Async traits and object safety:** Port traits
    use `async fn` (Rust 1.75+) and return
    `Result<T, E>`. Traits are **not** object-safe
    by default; `dyn Trait` dispatch is avoided
    in domain code. Where a trait object is
    genuinely required (e.g. a registry of
    plug-in adapters), the trait is explicitly
    marked object-safe and the consumer pays
    the cost knowingly.
17. **`Send + Sync` for shared state:** All state
    held across `.await` points is `Send + Sync`.
    The engine targets a multi-threaded `tokio`
    runtime; single-threaded executors are not
    part of the baseline. A `Rc<T>`, a
    `RefCell<T>`, or any non-`Send` type held
    across an `.await` is a compile error.
18. **Typed identifiers:** `StudentId`, `SchoolId`,
    `InvoiceId` are newtypes around `Uuid` with
    `Deref`/`From` and value-semantic equality.
    A `Uuid` does not substitute for a `StudentId`;
    the type system is the boundary against
    identifier confusion.
19. **Value objects:** `Money`, `DateRange`, `Page`,
    `TenantContext` are value types with constructor
    validation. They are not primitives; they do
    not leak into API surfaces as raw `i64` cents
    or raw `chrono::NaiveDate`.

## Query Architecture Patterns

The query layer follows a strict set of preferred
and avoided patterns. These patterns are the
authoritative reference for how query code is
written in Educore. The full specification is
in `docs_guidlines/query_optimze.md`; the
in-engine contract is in
`ADR-006: Compile-Time-Safe Query Layer`.

### Preferred Patterns

- **`#[derive(DomainQuery)]` procedural macros.**
  Every aggregate derives the macro. The macro
  emits a field enum, a state builder, and
  (where declared) a relation enum. The output
  is typed, named, visible to `rust-analyzer`,
  and refactor-safe. The macro is the
  compile-time counterpart to a runtime
  reflection-based ORM.
- **Extension traits for query scopes.** Semantic
  vocabulary (`.active()`, `.in_class()`) is
  expressed as extension traits implemented on
  the macro-generated builder. The macro does
  not emit domain language; humans do. This
  keeps macro output deterministic and isolates
  business vocabulary from the auto-generated
  surface.
- **Closure-based relational filters.** Cross-
  entity filters are typed closures:
  `where_has(StudentRelation::Parent, |q| { ... })`.
  The closure body is fully typed; the macro
  binds the related entity's builder into the
  closure parameter; the AST output is a single
  `HasRelation` node. There is no string
  templating of nested queries.
- **Explicit `.with()` directives.** Hydration is
  declared by the consumer and executed by the
  repository. There is no implicit fetch on
  access. The consumer reads domain models
  synchronously after the repository returns.
- **Typed identifiers and value objects.** See
  decision points 18 and 19 above.
- **Traits for ports.** The `*Repository`,
  `*Service`, and `*Gateway` traits are the
  consumer's surface. The trait is the API;
  the concrete type is an implementation
  detail of the storage adapter.
- **Builders.** The query builder is a builder.
  Methods consume `self` and return `Self`.
  State is type-checked at every step; an
  invalid intermediate state is a compile
  error, not a runtime panic.
- **Derive macros.** `#[derive(DomainQuery)]`,
  `#[derive(Command)]`,
  `#[derive(DomainEvent)]`,
  `#[derive(Serialize, Deserialize)]`,
  `#[derive(thiserror::Error)]` are the
  standard ways to add behavior to a domain
  type. Hand-written equivalents are
  rejected when a derive exists.

### Avoided Patterns

- **Declarative `macro_rules!` for the query
  API.** The new spec explicitly rejects
  `macro_rules!` for the query layer. A
  declarative macro produces opaque, token-
  shaped output that `rust-analyzer` cannot
  introsect, cannot autocomplete, and cannot
  rename safely. A typo inside a `macro_rules!`
  arm is a runtime error, not a compile
  error. The `#[derive(DomainQuery)]`
  procedural macro is preferred precisely
  because its expansion is a named, typed
  item in the type system. Declarative
  macros are acceptable for small syntactic
  conveniences (e.g. `vec_of![T; n]`) but
  not for query DSLs.
- **Implicit runtime magic.** Hidden
  transactions, implicit reflection pools,
  lazy proxies, runtime proxies, async
  getters on domain models, and `dyn Any`
  payloads are rejected. See
  `ADR-012: No Reflection` for the full
  rejection set.
- **Unchecked string selectors.**
  `.where("status", ...)` is a compile
  error. Field selectors are enum variants
  emitted by the macro. The macro's output
  is the only accepted selector surface.
- **Object-safe traits by default.** A trait
  is object-safe only when the consumer
  has a documented need for `dyn Trait`.
  Async port traits are not object-safe by
  default; the engine uses static dispatch
  through generic parameters.

## Consequences

### Positive

- **Predictable baseline.** A new contributor can
  read `code-standards.md` once and know the rules.
- **Tooling alignment.** Standard tools (`cargo
  clippy`, `cargo fmt`, `cargo test`) work out of
  the box. `rust-analyzer` works end-to-end
  through the query DSL because the macro's
  output is typed and named.
- **MSRV clarity.** A consumer on an older toolchain
  knows the engine will not work; they upgrade.
- **Ecosystem familiarity.** A Rust developer reads
  the engine and recognizes idioms: `thiserror`,
  `serde`, `tracing`, `tokio`, derive macros,
  extension traits, builders, value objects.
- **Performance.** `rustls`, `tokio`, `criterion`
  benchmarks, no `as` on numerics, no reflection,
  no N+1 — the engine is fast and correct by
  default.
- **Safety.** No `unsafe` in domain code, no panics,
  no implicit `as` truncations, no non-`Send`
  state across `.await` points, no identifier
  confusion, no hidden I/O behind a domain
  method call.
- **Refactor safety.** The `#[derive(DomainQuery)]`
  output is a named item in the type system;
  renaming a field ripples through the field
  enum, the builder, the scope traits, and the
  call sites. The compiler is the refactoring
  tool.

### Negative

- **Strict MSRV.** Consumers on a slightly older
  toolchain cannot use the engine. We accept this
  for predictability.
- **No alternative async runtime.** The engine is
  `tokio`-bound. A consumer using `async-std` cannot
  reuse the engine's ports directly. We accept this
  for the ecosystem alignment.
- **Clippy is opinionated.** Some `clippy` lints are
  noise. We disable the noise and keep the
  substance.
- **Compile time.** The strict type system, the
  derives, the workspace size — compile time grows.
  We accept it for the safety.
- **More trait boilerplate.** Async port traits
  with `async fn` are not object-safe. A consumer
  who wants `dyn Repository` must build a wrapper
  explicitly. We accept this for static-dispatch
  performance and monomorphization.
- **No ad-hoc `macro_rules!` shortcuts.** A consumer
  who wants a one-off DSL in `macro_rules!` for a
  one-off query surface is told to use the derive
  macro instead. We accept the friction for
  tooling alignment.

### Mitigations

- The `educore-core` crate re-exports common
  building blocks so domain crates have a
  consistent surface.
- A `cargo xtask` script (or equivalent) wires
  the standard CI checks.
- The `build-plan.md` describes a phased
  delivery that respects the standards from day
  one.
- A "Rust style" rustdoc page summarizes the
  standards for new contributors.
- The `educore-query-derive` crate provides
  `#[derive(DomainQuery)]` (and is the only
  proc-macro crate in v1; additional derives
  are added in subsequent phases) so the
  per-aggregate boilerplate is a few lines of
  attribute decoration, not a hundred lines
  of hand-written enum.
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

### 1. `async-std` as the async runtime

A smaller, simpler runtime. Rejected because
`tokio` is the de facto standard in the Rust
async ecosystem; the engine's value comes from
ecosystem alignment, not runtime novelty.

### 2. `no_std` for the core

The engine compiles without `std`. Rejected
because the engine's I/O assumptions
(async, threading, network) require `std`. A
`no_std` adapter for embedded devices is
possible but is a separate effort.

### 3. Custom error handling

A bespoke error type and propagation model.
Rejected because `thiserror` and `anyhow` are
the standard; custom errors confuse consumers.

### 4. JSON-only serialization

The engine uses `serde_json` directly. Rejected
because `serde` is the abstraction; consumers
can swap the format (bincode, postcard, CBOR)
without changing domain code.

### 5. No async

The engine is fully sync. Rejected because
storage adapters are async (PostgreSQL,
network APIs), and forcing them to block
would harm throughput. Async at the boundary,
sync in the domain — the standard pattern.

### 6. No MSRV

The engine supports "any reasonable Rust."
Rejected because the engine uses modern
features (`async fn` in trait, impl Trait,
the `#[derive(DomainQuery)]` macro family)
and pinning an MSRV is a forcing function
for clarity.

### 7. Declarative `macro_rules!` for the
query DSL

A consumer-facing query DSL expressed as
`macro_rules!` arms (`.active!()`,
`.in_class!(uuid)`). Rejected for the query
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

The `#[derive(DomainQuery)]` procedural
macro is preferred precisely because its
output is a named, typed item in the type
system. The field enum is a real enum; the
builder is a real struct; `rust-analyzer`
autocompletes both. The macro's expansion
is visible to refactoring tools in a way
that `macro_rules!` expansion is not.

Declarative `macro_rules!` remains
acceptable for small syntactic
conveniences where the IDE gap is
acceptable (e.g. `vec_of![T; n]`,
`matches!(...)`, internal test
fixtures). It is rejected specifically
for the consumer-facing query DSL.
