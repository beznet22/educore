# AGENTS.md

Orientation for AI agents and human developers working in the
**SMSengine** repository. The brand is **SMSengine**; the package
namespace is **`smsengine`**; internal crates publish under
**`smsengine-<name>`**. Use these forms everywhere. No legacy names
are permitted in new code, comments, commit messages, or
documentation.

## Project Identity

- **Brand (prose):** SMSengine
- **Umbrella package:** `smsengine` (`crates/smsengine/`)
- **Internal package names:** `smsengine-<name>` (e.g. `smsengine-academic`)
- **Internal crate directories:** `crates/<name>/` (the `smsengine-`
  prefix is dropped from the directory name; the package name and the
  directory name are intentionally different)
- **Public package registry path:** `smsengine::*`

## Workspace Layout

```text
<workspace-root>/                   <-- repository root on disk
├── Cargo.toml                       <-- virtual workspace root
├── crates/
│   ├── smsengine/                   <-- umbrella crate (smsengine)
│   ├── core/                        <-- package: smsengine-core
│   ├── query-derive/                <-- package: smsengine-query-derive
│   ├── platform/                    <-- package: smsengine-platform
│   ├── rbac/                        <-- package: smsengine-rbac
│   ├── settings/                    <-- package: smsengine-settings
│   ├── events/                      <-- package: smsengine-events
│   ├── academic/                    <-- package: smsengine-academic
│   ├── assessment/                  <-- package: smsengine-assessment
│   ├── attendance/                  <-- package: smsengine-attendance
│   ├── finance/                     <-- package: smsengine-finance
│   ├── hr/                          <-- package: smsengine-hr
│   ├── library/                     <-- package: smsengine-library
│   ├── facilities/                  <-- package: smsengine-facilities
│   ├── communication/               <-- package: smsengine-communication
│   ├── events-domain/               <-- package: smsengine-events-domain
│   ├── documents/                   <-- package: smsengine-documents
│   ├── cms/                         <-- package: smsengine-cms
│   ├── storage/                     <-- package: smsengine-storage
│   ├── storage-postgres/            <-- package: smsengine-storage-postgres
│   ├── storage-mysql/               <-- package: smsengine-storage-mysql
│   ├── storage-sqlite/              <-- package: smsengine-storage-sqlite
│   ├── storage-parity/              <-- package: smsengine-storage-parity (cross-adapter test suite)
│   ├── audit/                       <-- package: smsengine-audit (audit log writer)
│   ├── operations/                  <-- package: smsengine-operations (operations domain)
│   ├── testkit/                     <-- package: smsengine-testkit (in-memory test adapters)
│   ├── cli/                         <-- package: smsengine-cli (sample binary)
│   ├── auth/                        <-- package: smsengine-auth
│   ├── notify/                      <-- package: smsengine-notify
│   ├── payment/                     <-- package: smsengine-payment
│   ├── files/                       <-- package: smsengine-files
│   ├── event-bus/                   <-- package: smsengine-event-bus
│   ├── integrations/                <-- package: smsengine-integrations
│   └── sdk/                         <-- package: smsengine-sdk
├── docs/                            <-- documentation operating system
│   ├── project-overview.md
│   ├── architecture.md
│   ├── build-plan.md
│   ├── code-standards.md
│   ├── library-docs.md
│   ├── progress-tracker.md
│   ├── query_layer.md
│   ├── specs/                       <-- 15 domain specs × 11 files
│   ├── ports/                       <-- 7 port contracts
│   ├── commands/                    <-- 15 command catalogs
│   ├── events/                      <-- 15 event catalogs
│   ├── schemas/                     <-- 6 cross-cutting schemas
│   │   ├── database-schema.md       <-- engine invariants
│   │   ├── event-schema.md          <-- outbox + event_log spec
│   │   ├── audit-schema.md          <-- audit_log spec
│   │   ├── command-schema.md        <-- idempotency spec
│   │   ├── tenancy-schema.md        <-- school_id RLS spec
│   │   ├── query-schema.md          <-- query AST
│   │   ├── sql-dialects/            <-- per-dialect DDL conventions
│   │   │   ├── README.md            <-- index + runtime DDL emission flow
│   │   │   ├── mysql.md             <-- MySQL 8+ conventions
│   │   │   ├── postgresql.md        <-- PostgreSQL 14+ conventions
│   │   │   ├── sqlite.md            <-- SQLite 3.x conventions
│   │   │   └── comparison.md        <-- feature-by-feature table
│   │   └── data-migration/          <-- 13 files: README + 00-overview..11-security
│   ├── decisions/                   <-- 14 ADRs
│   ├── diagrams/                    <-- 7 Mermaid diagrams
│   ├── research/                    <-- 16 research files
│   └── guides/                      <-- 17 implementation guides + README (18 files)
├── migrations/                      <-- SQL migration scripts
│   ├── README.md                    <-- engine target schema, gap, plan
│   ├── engine/                      <-- canonical DDL for 6 cross-cutting tables, 3 dialects
│   │   ├── README.md                <-- index of the 3 dialect files
│   │   ├── 0000_engine_core.mysql.sql    <-- MySQL 8+ reference
│   │   ├── 0000_engine_core.postgres.sql <-- PostgreSQL 14+ reference
│   │   └── 0000_engine_core.sqlite.sql    <-- SQLite 3.x reference
│   └── 0001_*.sql..0015_*.sql       <-- legacy Schoolify dump (research source)
├── schoolify/                       <-- legacy Laravel project (read-only)
└── docs_guidlines/                  <-- three authoritative guideline docs
```

## Naming Convention (Enforced)

| Layer                       | Package name              | Directory             | Rust extern crate id    |
| --------------------------- | ------------------------- | --------------------- | ----------------------- |
| Umbrella                    | `smsengine`               | `crates/smsengine/`   | `smsengine`             |
| Internal (per-domain)       | `smsengine-<name>`        | `crates/<name>/`      | `smsengine_<name>`      |
| Storage adapters (shipped)  | `smsengine-storage-<db>`   | `crates/storage-<db>/`| `smsengine_storage_<db>`|

The umbrella re-exports each internal crate under its short name:

```rust
// crates/smsengine/src/lib.rs
pub use smsengine_core as core;
pub use smsengine_academic as academic;
// ...
```

Consumers therefore write `smsengine::academic::commands::*` and never
need to know the internal `smsengine-` prefix on the package name.

## Authoritative Documents (Read These First)

In priority order:

1. `docs/project-overview.md` — engine philosophy and scope
2. `docs/architecture.md` — the system map
3. `docs/build-plan.md` — the implementation roadmap
4. `docs/code-standards.md` — the engineering rules (must follow)
5. `docs/library-docs.md` — consumer-facing SDK documentation
6. `docs/query_layer.md` — the macro-driven query specification
7. `docs/specs/<domain>/overview.md` — per-domain specifications
8. `docs/ports/*.md` — port contracts
9. `docs/decisions/*.md` — architectural decisions (ADRs)
10. `docs/guides/saas-backend.md` — building a production SaaS on
    top of the library (backend, control plane, identity, sync
    engine, offline clients)

## Engine Rules (Non-Negotiable)

These are repeated from `docs/code-standards.md` and
`docs/project-overview.md` for quick reference. They are the rules
every implementation must follow.

1. **Brand is SMSengine.** Use **SMSengine** in prose and
   **`smsengine`** in code. No legacy names are permitted anywhere.
2. **Compile-time safety over strings.** Use macro-generated enums
   (`StudentField::Status`) — never string field names.
3. **Domain scopes via extension traits.** `.active()`, `.in_class()`,
   etc. are implemented as extension traits on the macro-generated
   builder.
4. **Closure-based nested relational filters.** Use `where_has` with
   a closure bound to the related entity's macro-generated builder.
5. **Strict eager loading.** Use `.with(StudentRelation::Parent)` to
   populate related fields. Lazy accessors and async getters are
   forbidden.
6. **No SQL/NoSQL emission from macros.** The `#[derive(DomainQuery)]`
   macro emits an AST; storage adapters translate the AST.
7. **Multi-tenant by default.** Every aggregate has a `SchoolId`.
8. **Audit-first.** Every state change writes an immutable record.
9. **Production-ready.** Real schools, real students, real money.

## Code Standards (Quick Reference)

See `docs/code-standards.md` for the full rules. Highlights:

- Rust edition `2021`, MSRV `1.75`.
- All public APIs are documented with rustdoc; `#![deny(missing_docs)]`.
- `unsafe` is forbidden in domain code (`#![forbid(unsafe_code)]`).
- `unwrap`, `expect`, `panic!` are forbidden in production paths.
- All fallible APIs return `Result<T, DomainError>`.
- Errors use `thiserror` for public APIs, `anyhow` for glue.
- Numeric conversions use `TryFrom`/`TryInto`; `as` on numerics is
  forbidden.
- `Send + Sync` preserved for all public async types.
- No `serde_json::Value` in domain code. Use typed wrappers.
- No `HashMap<String, T>` for domain data.
- No service locators, DI containers, or runtime reflection.
- All dependencies use `rustls`; never `native-tls`.

## Module Layout (per domain)

```text
crates/<domain>/                   <-- directory
├── src/
│   ├── lib.rs
│   ├── aggregate.rs
│   ├── entities.rs
│   ├── value_objects.rs
│   ├── commands.rs
│   ├── events.rs
│   ├── services.rs
│   ├── repository.rs              <-- port trait
│   ├── query.rs                   <-- query builder
│   └── errors.rs
├── tests/
├── Cargo.toml                     <-- [package] name = "smsengine-<domain>"
└── README.md
```

One crate per domain. `lib.rs` re-exports the public surface. Crate-
private types stay private.

> **Note:** the spec folder (`docs/specs/<domain>/`) uses
> `services.md` (not `policies.md`) and `workflows.md` (not
> `errors.md`). The Rust source tree mirrors the spec folder: the
> `services.rs` module hosts policy logic; the `errors.rs` module
> defines the `DomainError` enum. See `docs/code-standards.md` §
> "Spec folder layout" for the 11-file mapping.

## Dependency Rules

A domain crate may depend on:

- `smsengine-core`
- `smsengine-platform`
- `smsengine-rbac`
- `smsengine-events`

Other domain crates only with explicit justification in an ADR.

A domain crate may **not** depend on:

- Any adapter crate
- Any infrastructure crate
- `tokio` directly (only through `smsengine-core` re-exports where needed)
- `serde_json::Value`

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
- [ ] No legacy brand references introduced

## Agent Instructions

These instructions apply to every AI agent (and human developer) that
edits code in this repository. They are the day-to-day operating
contract; the Validation Checklist above is the per-PR gate.

### Package Manager

Use **cargo** to manage dependencies and build targets:

- Build: `cargo build`
- Test: `cargo test`
- Add crate: `cargo add <crate> --package <package-name>` (always use
  `cargo add` to fetch the latest version)
- Create crate: `cargo new --lib --vcs none crates/<name>` (always
  use `--vcs none` to prevent nested git repositories)

### File-Scoped Commands

For work that targets a single crate, scope the command to that
package to avoid spurious rebuilds of the full workspace:

| Task   | Command                            |
| ------ | ---------------------------------- |
| Build  | `cargo build --package <package>`  |
| Test   | `cargo test --package <package>`   |
| Check  | `cargo check --package <package>`  |
| Clippy | `cargo clippy --package <package>` |
| Format | `cargo fmt --all`                  |


### Type Safety

Enforce full type safety at all times:

- **No `unwrap()` or `expect()`** in production paths. Propagate
  errors via `?` or document the invariant that makes panic
  impossible.
- **No `#[allow(dead_code)]`** or `_var` prefixes to silence the
  compiler. Delete unused code, wire it in, or open a follow-up
  issue.
- **No `as` casts** that truncate or lose data. Use `TryFrom` /
  `TryInto` with proper error handling.
- **All public APIs return `Result`** for fallible operations. Use
  `anyhow::Result` as the default surface; use `thiserror` for
  structured error types where callers need to match variants.
- **Trait objects must be object-safe**. Verify with
  `let _: Box<dyn Trait>;` compile tests.

These rules are a strict superset of the "Code Standards" section
above; in case of conflict, this section wins.

### Testing (TDD)

Enforce test-driven development. Write tests **before or alongside**
implementation, never as an afterthought.

- **No dummy tests**. Every test must validate a real-world
  scenario: round-trip serialization, error propagation, trait
  object dispatch, iteration cap enforcement, category filtering,
  etc. Tests like `assert!(true)` or `fn it_works()` are rejected.
- **Test error paths**, not just happy paths. Verify that malformed
  input, unknown tool names, provider failures, and iteration
  limits produce the correct `Result::Err` or fallback behavior.
- **At least one integration test per PR** (per the Validation
  Checklist above). Unit tests alone are not sufficient.

### Key Conventions

- **Workspace layout**: Root `Cargo.toml` is a virtual workspace.
  All crates live under `crates/*`. The Crate Inventory table
  above is the authoritative phase assignment for each crate.
- **Crate isolation**: Avoid relative path dependencies outside
  the workspace. Every `path = "..."` in `Cargo.toml` must point
  to a sibling crate under `crates/`.
- **Imports**: Use explicit crate-relative paths or re-exports in
  public library API surfaces. Avoid glob imports in domain code.
- **TLS/SSL Cross-Compilation**: Strictly enforce `rustls` instead
  of `native-tls` to support cross-compilation (e.g. Android
  ARM64). For crates like `reqwest`, always set
  `default-features = false` and enable the `rustls` or `rustls-tls`
  feature.

## Storage Adapters

Three reference adapters are shipped:

- `smsengine-storage-postgres` (primary target)
- `smsengine-storage-mysql` (production target, MySQL 8.0+)
- `smsengine-storage-sqlite` (embedded / offline mode)

The SurrealDB and MongoDB adapters are **deferred to a future release**
and are **not** shipped from the engine. See
`docs/ports/storage.md#future-storage-backends-deferred` for the
rationale and the path for consumers who need a deferred adapter.

## Graphify

A pre-computed knowledge graph of the legacy `schoolify/` codebase
lives at `schoolify/graphify-out/`. Read
`schoolify/graphify-out/GRAPH_REPORT.md` for god nodes and community
structure. Use `cd schoolify && graphify query "<question>"` to
traverse the graph when investigating legacy behavior. The graph is a
**navigation aid for the Laravel source only** — never copy code from
it into SMSengine.

The `schoolify/` tree is the legacy Laravel project. It is **not**
the engine schema. The engine's target schema is in
`docs/schemas/` and the migration plan from legacy to engine is in
`docs/schemas/data-migration/`. The 15 `migrations/0001_*.sql` through
`migrations/0015_*.sql` files are a Schoolify dump, not the engine
schema — see `migrations/README.md` for the gap and the migration
plan.

## Status

- Documentation: **complete** (~302 markdown files, 15 domain
  specs × 11 files each = 165 spec files).
- Workspace scaffold: **complete** (34 crates, virtual workspace).
- Storage adapters shipped: **PostgreSQL, MySQL, SQLite**.
- Storage adapters deferred: **SurrealDB, MongoDB** (consumer may
  implement in-tree on demand).
- Implementation: **not started** — scaffold only. Domain logic,
  aggregates, value objects, commands, events, repositories, and
  storage translations are pending.
- Domain spec cleanup: **complete**. All legacy-prefixed table
  references (the seven common Laravel/InfixEdu prefixes plus
  the brand-tainted Rust type names) have been removed from
  `docs/specs/` and replaced with engine
  `<domain>_<aggregate>` names. 77 spec files updated,
  ~1033 insertions / ~1102 deletions.
- Build plan: **17 phases** (Phase 0..17) with coverage matrix and
  no-gaps gates documented in `docs/build-plan.md`. The 5 new
  crates are scaffolded and assigned to:
  - `smsengine-storage-parity` → Phase 0 (cross-adapter test suite)
  - `smsengine-audit` → Phase 2 (cross-cutting foundations)
  - `smsengine-operations` → Phase 14 (Settings + Operations)
  - `smsengine-testkit` → Phase 16 (Test infrastructure + SDK)
  - `smsengine-cli` → Phase 16 (Test infrastructure + SDK)

## Crate Inventory (per-crate phase assignment)

Every one of the 34 workspace crates is scaffolded. Implementation
begins in Phase 0 of `docs/build-plan.md`. The table below maps
each crate to the phase that implements it. **This is the
authoritative source** — do not rely on the directory tree or the
umbrella re-exports to determine phase assignment.

| # | Crate | Phase | Title |
| --- | --- | --- | --- |
| 1 | `smsengine-core` | 0 | Foundation |
| 2 | `smsengine-query-derive` | 0 | Foundation (proc-macro) |
| 3 | `smsengine-storage` | 0 | Foundation (port trait) |
| 4 | `smsengine-storage-postgres` | 0 | Foundation (PG adapter) |
| 5 | `smsengine-storage-parity` | 0 | Foundation (cross-adapter test suite) |
| 6 | `smsengine-storage-mysql` | 1 | Adapter parity |
| 7 | `smsengine-storage-sqlite` | 1 | Adapter parity |
| 8 | `smsengine-platform` | 2 | Cross-cutting foundations |
| 9 | `smsengine-rbac` | 2 | Cross-cutting foundations |
| 10 | `smsengine-events` | 2 | Cross-cutting foundations (envelope) |
| 11 | `smsengine-event-bus` | 2 | Cross-cutting foundations (bus port) |
| 12 | `smsengine-audit` | 2 | Cross-cutting foundations (audit log) |
| 13 | `smsengine-academic` | 3 | Academic |
| 14 | `smsengine-assessment` | 4 | Assessment |
| 15 | `smsengine-attendance` | 5 | Attendance |
| 16 | `smsengine-hr` | 6 | HR |
| 17 | `smsengine-finance` | 7 | Finance |
| 18 | `smsengine-facilities` | 8 | Facilities |
| 19 | `smsengine-library` | 9 | Library |
| 20 | `smsengine-communication` | 10 | Communication |
| 21 | `smsengine-documents` | 11 | Documents |
| 22 | `smsengine-cms` | 12 | CMS |
| 23 | `smsengine-events-domain` | 13 | Events domain (calendar) |
| 24 | `smsengine-settings` | 14 | Settings + Operations |
| 25 | `smsengine-operations` | 14 | Settings + Operations |
| 26 | `smsengine-auth` | 15 | Port adapters |
| 27 | `smsengine-notify` | 15 | Port adapters |
| 28 | `smsengine-payment` | 15 | Port adapters |
| 29 | `smsengine-files` | 15 | Port adapters |
| 30 | `smsengine-integrations` | 15 | Port adapters |
| 31 | `smsengine-testkit` | 16 | Test infrastructure + SDK |
| 32 | `smsengine-storage-parity` | 16 | (Test infrastructure + SDK) |
| 33 | `smsengine-sdk` | 16 | Test infrastructure + SDK |
| 34 | `smsengine-cli` | 16 | Test infrastructure + SDK |
| — | `smsengine` (umbrella) | 0 | re-exports only; first usable at Phase 0+ |

**Note on duplicates:** `smsengine-storage-parity` is listed at
both Phase 0 and Phase 16. Phase 0 scaffolds the crate; Phase 16
implements the actual test scenarios. The umbrella crate
`smsengine` is first usable once Phase 0 lands; its re-exports
are wired in `crates/smsengine/src/lib.rs`.

**Note on `smsengine-events` vs `smsengine-events-domain`:**
- `smsengine-events` (Phase 2) is the **event envelope + bus
  port** (DomainEvent trait, EventEnvelope, EventBus trait).
- `smsengine-events-domain` (Phase 13) is the **calendar domain**
  (CalendarEvent, Holiday, Incident, Weekend aggregates).
  These are distinct crates with distinct crates. Do not
  conflate them.

## Runtime DDL emission (where the schema actually lives)

The engine does **not** apply `.sql` migration files at runtime.
The schema is **emitted** by the storage adapter at startup via
`storage.create_schema().await`. The end-to-end flow is documented
in
[`docs/schemas/sql-dialects/README.md` § "Runtime DDL emission"](docs/schemas/sql-dialects/README.md#runtime-ddl-emission--end-to-end-flow):

1. **Design contract** — `docs/specs/<domain>/tables.md`
   (human-readable, not executable).
2. **Type contract** — `crates/<domain>/src/aggregate.rs`
   (Rust struct, compiled).
3. **Machine contract** — `crates/<domain>/src/entities.rs`
   (macro-emitted typed AST, dialect-agnostic).
4. **Adapter emission** — `smsengine-storage-<db>` walks the AST
   at schema-creation time and emits the dialect-specific DDL
   string. The 6 engine cross-cutting tables
   (`outbox`, `audit_log`, `idempotency`, `event_log`,
   `schema_registry`, `system_user`) are hard-coded in the
   adapter via `include_str!`. The ~310 domain tables are
   macro-emitted.
5. **Consumer startup** — `storage.create_schema().await` runs
   the DDL once per process lifetime. The DB round-trip
   dominates the cost (~6 s for ~310 tables on MySQL); string
   build time is <10 ms.

The 6 cross-cutting tables have canonical DDL in three dialects under
`migrations/engine/` (`0000_engine_core.mysql.sql`,
`0000_engine_core.postgres.sql`, `0000_engine_core.sqlite.sql`).
The `smsengine-storage-<db>` adapter crates `include_str!` these
files at compile time. The `migrations/0001_*.sql`–
`migrations/0015_*.sql` files are the legacy Schoolify/InfixEdu
dump (research source only). The data-migration plan from legacy
to engine is in `docs/schemas/data-migration/`.

## Co-Authoring

AI-generated commits must include the trailer specified in the
**Commit Attribution** subsection of Agent Instructions above:

```text
Co-Authored-By: Antigravity <antigravity@google.com>
```

This is the canonical attribution for every AI-authored commit in
this repository. No other `Co-Authored-By` trailer is accepted for
AI agents.
