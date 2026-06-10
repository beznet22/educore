# Educore Code Standards

These standards are mandatory. They are the rules every implementation must
follow. They are deliberately simple and focused on production correctness.

## Rust Standards

- Edition: `2021`.
- MSRV: 1.75 (or the latest stable LTS at the time of release).
- All public APIs are documented with rustdoc.
- All `unsafe` is forbidden in domain code. Adapters may use `unsafe` only
  when justified with a `// SAFETY:` comment and reviewed.
- All `unwrap`, `expect`, `panic!` are forbidden in production code paths.
- All fallible APIs return `Result<T, DomainError>`.
- Errors are typed with `thiserror` for public APIs, `anyhow` for glue code.
- Numeric conversions use `TryFrom`/`TryInto`. `as` is forbidden on numerics.
- `Send + Sync` is preserved for all public types in async contexts.
- Object safety is preserved for all port traits.

## DDD Rules

- Aggregates own their consistency boundary. External code must not reach
  into an aggregate's children.
- Value objects are immutable and validated at construction.
- Identifiers are typed (`StudentId`, `GuardianId`, ...), not raw `u64`.
- Domain events describe **facts that happened**, in past tense.
- Commands propose actions, in imperative tense.
- Domain services orchestrate logic that does not fit in one aggregate.
- Policies are pure functions over state, returning a decision.
- Specifications are composable predicates.

## Hexagonal Rules

- Domain code depends on no adapter.
- Adapters depend only on the engine and on the concrete infrastructure crate
  (e.g. `sqlx`, `aws-sdk-s3`).
- Ports are traits. Adapters implement traits. No port may import an adapter.
- Ports own no state. They are passed by reference or `Arc`.

## Module Rules

```text
crates/domains/<domain>/        <-- package name: educore-<domain>
├── src/
│   ├── lib.rs
│   ├── aggregate.rs
│   ├── entities.rs
│   ├── value_objects.rs
│   ├── commands.rs
│   ├── events.rs
│   ├── services.rs
│   ├── repository.rs   // port trait
│   ├── query.rs        // query builder
│   └── errors.rs
├── tests/
│   ├── admission.rs
│   ├── promotion.rs
│   └── ...
├── Cargo.toml
└── README.md
```

- One crate per domain.
- `lib.rs` re-exports the public surface.
- Public types are gated by re-exports. Crate-private types stay private.

## Spec folder layout

Per-domain documentation lives in `docs/specs/<domain>/`. The 11 files
per spec folder are:

| File | Purpose |
| --- | --- |
| `overview.md` | Domain philosophy, scope, dependencies |
| `aggregates.md` | Aggregate root definitions (Rust structs) |
| `entities.md` | Macro-emitted entity rows (the `#[derive(DomainQuery)]` inputs) |
| `value-objects.md` | Value object definitions (Rust types) |
| `commands.md` | Command structs + handlers |
| `events.md` | Domain event structs |
| `services.md` | Domain services (stateless operations) |
| `permissions.md` | RBAC capability mapping |
| `repositories.md` | Repository port trait + methods |
| `workflows.md` | Multi-aggregate workflows (e.g. admission) |
| `tables.md` | Table layout (column list, indexes, FKs) |

Note: spec folders use `services.md` and `permissions.md` (not
`policies.md`), and `workflows.md` (not `errors.md`). This is by
design — `services.md` hosts policy logic, `permissions.md` hosts
RBAC capability maps, and `workflows.md` hosts error scenarios.

## Type Safety

- No `serde_json::Value` in domain code. Use typed wrappers or `serde(tag = "type")`
  enums.
- No raw string field names in queries. Use `StudentField`, `ClassField`, etc.
- No `HashMap<String, T>` for domain data. Use typed structs.
- Identifier types prevent cross-aggregate confusion at compile time.

```rust
pub struct StudentId(SchoolId, Uuid);

impl StudentId {
    pub fn new(school: SchoolId) -> Self { ... }
    pub fn school(&self) -> SchoolId { self.0 }
}
```

## Error Handling

- Every public fallible function returns `Result<T, DomainError>`.
- `DomainError` is a `thiserror` enum with `#[from]` conversions where
  appropriate.
- Engine-level errors include a `kind` discriminant
  (`Validation`, `NotFound`, `Conflict`, `Forbidden`, `Infrastructure`).
- The command dispatcher converts infrastructure errors into a generic
  `Infrastructure` variant; domain errors pass through unchanged.
- Tests assert on error variants, not on display strings.

## Async

- All I/O is async.
- Repositories and ports use `async_trait`.
- Domain logic is sync wherever possible.
- No blocking I/O inside an async function. Use `tokio::task::spawn_blocking`
  for genuinely blocking work.

## Logging & Observability

- Use `tracing` for structured logs.
- Every command emits a `command.start` and `command.end` event with duration
  and outcome.
- Every domain event is logged at `INFO` with the aggregate id.
- Sensitive data (PII, financial values, auth tokens) is **never** logged.

## Testing

- Unit tests live alongside code in `#[cfg(test)] mod tests`.
- Integration tests live in `tests/`.
- Tests use the in-memory storage adapter unless they need a real database.
- Every command has at least one happy-path test and one error-path test.
- Every event has a roundtrip serialization test.
- Every value object has a validation test.
- Every policy has a decision-table test.

## Documentation

- Every public item has a rustdoc comment.
- Module-level `//!` documentation describes the module's purpose.
- Crate-level `lib.rs` describes what the crate owns and what it depends on.
- Diagrams in `docs/diagrams/` use Mermaid and are kept up to date.

## Dependency Rules

- A domain crate may depend on:
  - `educore-core`
  - `educore-platform`
  - `educore-rbac`
  - `educore-events`
  - Other domain crates only with explicit justification in an ADR.
- A domain crate may **not** depend on:
  - Any adapter crate
  - Any infrastructure crate
  - `tokio` directly (only through `educore-core` re-exports where needed)
  - `serde_json::Value`

## Cross-Compilation

- All dependencies use `rustls` instead of `native-tls`.
- No platform-specific code outside adapters.
- CI verifies builds for `x86_64-unknown-linux-gnu`,
  `aarch64-unknown-linux-gnu`, `x86_64-apple-darwin`, and
  `x86_64-pc-windows-msvc`.

## Forbidden Patterns

- `unwrap()`, `expect()`, `panic!` in production paths.
- `as` on numeric types.
- `String` field names in queries.
- `serde_json::Value` in domain code.
- `tokio::main` in library crates.
- Service locators.
- Dependency injection containers.
- Runtime reflection.
- `lazy_static` / `once_cell` outside of narrow technical needs.
- Comments that narrate code; comments that document **why**.
- Tests of the form `assert!(true)` or `fn it_works`.

## Validation Checklist (per PR)

- [ ] `cargo build --workspace` passes
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo fmt --all -- --check` passes
- [ ] No new `unwrap`/`expect`/`panic` in non-test code
- [ ] No new `as` on numerics
- [ ] No new `serde_json::Value` in domain code
- [ ] Public items documented
- [ ] At least one integration test added for new behavior
- [ ] Diagrams updated if structure changed
- [ ] ADRs updated if architectural decisions changed
