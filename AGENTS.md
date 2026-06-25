# AGENTS.md

Orientation for AI agents and human developers working in the
**Educore** repository. The brand is **Educore**; the package
namespace is **`educore`**; internal crates publish under
**`educore-<name>`**. Use these forms everywhere. No legacy names
are permitted in new code, comments, commit messages, or
documentation.

## Project Identity

- **Brand (prose):** Educore
- **Umbrella package:** `educore` (`crates/educore/`)
- **Internal package names:** `educore-<name>` (e.g. `educore-academic`)
- **Internal crate directories:** `crates/<tier>/<name>/` (the
  `educore-` prefix is dropped from the directory name; the
  package name and the directory name are intentionally different.
  Each internal crate lives under a tier directory that encodes
  its purpose — see [§ Tier System](#tier-system) below)
- **Public package registry path:** `educore::*`

## Workspace Layout

The 36 internal crates + 1 umbrella = 37 packages are organized
into 5 tiers + 1 umbrella. Tier boundaries are enforced at the
filesystem level (the `educore-core::lint` sub-module verifies
that a crate in `crates/domains/` does not import from
`crates/adapters/` or `crates/tools/`). Each tier has a clear
purpose documented in [§ Tier System](#tier-system) below.

```text
<workspace-root>/                   <-- repository root on disk
├── Cargo.toml                       <-- virtual workspace root (5-tier glob pattern)
├── crates/
│   ├── infra/                       <-- infra tier (infrastructure, no domain knowledge)
│   │   ├── core/                    <-- package: educore-core
│   │   ├── query-derive/            <-- package: educore-query-derive
│   │   └── storage/                 <-- package: educore-storage
│   ├── cross-cutting/               <-- cross-cutting tier (cross-domain foundations)
│   │   ├── platform/                <-- package: educore-platform
│   │   ├── rbac/                    <-- package: educore-rbac
│   │   ├── events/                  <-- package: educore-events (envelope)
│   │   ├── events-domain/           <-- package: educore-events-domain (calendar)
│   │   ├── settings/                <-- package: educore-settings
│   │   ├── operations/              <-- package: educore-operations
│   │   ├── audit/                   <-- package: educore-audit
│   │   ├── sync/                    <-- package: educore-sync (port trait, per ADR-018)
│   │   └── sync-inprocess/          <-- package: educore-sync-inprocess (in-process reference adapter)
│   ├── domains/                     <-- domains tier (the 10 domain bounded contexts)
│   │   ├── academic/                <-- package: educore-academic
│   │   ├── assessment/              <-- package: educore-assessment
│   │   ├── attendance/              <-- package: educore-attendance
│   │   ├── cms/                     <-- package: educore-cms
│   │   ├── communication/           <-- package: educore-communication
│   │   ├── documents/               <-- package: educore-documents
│   │   ├── facilities/              <-- package: educore-facilities
│   │   ├── finance/                 <-- package: educore-finance
│   │   ├── hr/                      <-- package: educore-hr
│   │   └── library/                 <-- package: educore-library
│   ├── adapters/                    <-- adapters tier (port implementations)
│   │   ├── storage-surrealdb/       <-- package: educore-storage-surrealdb (primary per ADR-017)
│   │   ├── storage-postgres/        <-- package: educore-storage-postgres
│   │   ├── storage-mysql/           <-- package: educore-storage-mysql
│   │   ├── storage-sqlite/          <-- package: educore-storage-sqlite
│   │   ├── auth/                    <-- package: educore-auth
│   │   ├── event-bus/               <-- package: educore-event-bus
│   │   ├── files/                   <-- package: educore-files
│   │   ├── integrations/            <-- package: educore-integrations
│   │   ├── notify/                  <-- package: educore-notify
│   │   └── payment/                 <-- package: educore-payment
│   ├── tools/                       <-- tools tier (dev tooling, not in release)
│   │   ├── testkit/                 <-- package: educore-testkit
│   │   ├── storage-parity/          <-- package: educore-storage-parity
│   │   ├── cli/                     <-- package: educore-cli (binary)
│   │   └── sdk/                     <-- package: educore-sdk
│   └── educore/                   <-- umbrella crate
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
│   ├── engine/                      <-- canonical DDL for 6 cross-cutting tables, 4 dialect files
│   │   ├── README.md                <-- index of the 4 dialect files
│   │   ├── 0000_engine_core.mysql.sql    <-- MySQL 8+ reference
│   │   ├── 0000_engine_core.postgres.sql <-- PostgreSQL 14+ reference
│   │   ├── 0000_engine_core.sqlite.sql    <-- SQLite 3.x reference
│   │   └── 0000_engine_core.surreal.surql <-- SurrealDB reference
│   └── 0001_*.sql..0015_*.sql       <-- legacy Schoolify dump (research source)
├── .gitignore                       <-- excludes target/, .DS_Store, etc.
├── .graphifyignore                  <-- graphify exclude list (schoolify/, docs_guidlines/, target/, .git/, graphify-out/cache/, graphify-out/cost.json)
├── graphify-out/                    <-- engine knowledge graph (committed; cache/ and cost.json are gitignored)
├── schoolify/                       <-- legacy Laravel project (read-only)
│   └── graphify-out/                <-- legacy Laravel graph (frozen; research artefact only — not auto-rebuilt)
└── docs_guidlines/                  <-- three authoritative guideline docs
```

## Naming Convention (Enforced)

| Layer                       | Package name              | Directory             | Rust extern crate id    |
| --------------------------- | ------------------------- | --------------------- | ----------------------- |
| Umbrella                    | `educore`               | `crates/educore/`   | `educore`             |
| Internal (per-domain)       | `educore-<name>`        | `crates/<name>/`      | `educore_<name>`      |
| Storage adapters (shipped)  | `educore-storage-<db>`   | `crates/storage-<db>/`| `educore_storage_<db>`|

The umbrella re-exports each internal crate under its short name:

```rust
// crates/educore/src/lib.rs
pub use educore_core as core;
pub use educore_academic as academic;
// ...
```

Consumers therefore write `educore::academic::commands::*` and never
need to know the internal `educore-` prefix on the package name.

## Tier System

The 37 packages are organized into 5 tiers. Each tier has a
distinct purpose, dependency direction, and lifecycle.

| Tier | Path | Count | Purpose | Depends on |
| --- | --- | --- | --- | --- |
| `infra` | `crates/infra/` | 3 | Infrastructure: errors, identifiers, value objects, query AST, proc-macro, storage port | (none) |
| `cross-cutting` | `crates/cross-cutting/` | 9 | Cross-domain foundations: platform, rbac, events envelope, events-domain (calendar), audit, settings, operations, sync port, sync-inprocess | `infra` |
| `domains` | `crates/domains/` | 10 | The 10 domain bounded contexts (academic, finance, hr, ...) | `infra`, `cross-cutting` |
| `adapters` | `crates/adapters/` | 10 | Port implementations: 4 storage adapters (surrealdb, postgres, mysql, sqlite) + 6 port adapters (auth, event-bus, files, integrations, notify, payment) | `infra`, `cross-cutting` |
| `tools` | `crates/tools/` | 4 | Dev tooling: testkit, storage-parity, cli (binary), sdk | `infra`, `cross-cutting`, `domains` |

The umbrella crate `educore` re-exports the public surface of
all 37 packages.

**Layered dependency direction** (no cycles, no upward deps):

```text
infra  ←  cross-cutting  ←  domains  ←  tools
                           ↑
                           └──  adapters  (also depends on infra + cross-cutting)
```

**Tier boundary enforcement:** the `educore-core::lint`
sub-module verifies at build time that a crate in `crates/domains/`
does not import from `crates/adapters/` or `crates/tools/`, and
that a crate in `crates/cross-cutting/` does not import from
`crates/domains/`, `crates/adapters/`, or `crates/tools/`. See
`docs/build-plan.md` § The No-Gaps Gates.

**Note on `educore-events` vs `educore-events-domain`:**
- `educore-events` (cross-cutting tier) is the **event envelope + bus
  port** (DomainEvent trait, EventEnvelope, EventBus trait).
- `educore-events-domain` (cross-cutting tier) is the **calendar
  domain** (CalendarEvent, Holiday, Incident, Weekend aggregates).
  These are distinct crates with distinct packages. Do not
  conflate them.

**Note on `infra/core`:** the `educore-core` package (the engine's
core types — errors, identifiers, value objects, the query AST) lives
at `crates/infra/core/`. The tier is named `infra/` (infrastructure)
to make room for the package name `core/` without a double-naming
collision. The two other crates in the `infra` tier (`query-derive`
and `storage`) keep their original directories because their short
names don't collide with the tier name. This is a naming
convention, not a typographical error.

See `docs/decisions/ADR-013-CrateLayout.md` for the full rationale
and the migration history.

## Authoritative Documents (Read These First)

In priority order:

1. `CONTRIBUTING.md` — the spec-to-PR workflow for human
   developers (the companion to this document; see
   `CONTRIBUTING.md` for the human-facing version)
2. `docs/project-overview.md` — engine philosophy and scope
3. `docs/architecture.md` — the system map
4. `docs/build-plan.md` — the implementation roadmap
5. `docs/code-standards.md` — the engineering rules (must follow)
6. `docs/library-docs.md` — consumer-facing SDK documentation
7. `docs/query_layer.md` — the macro-driven query specification
8. `docs/specs/<domain>/overview.md` — per-domain specifications
9. `docs/ports/*.md` — port contracts
10. `docs/decisions/*.md` — architectural decisions (ADRs)
11. `docs/guides/saas-backend.md` — building a production SaaS on
    top of the library (backend, control plane, identity, sync
    engine, offline clients)
    engine, offline clients)

## Engine Rules (Non-Negotiable)

These are repeated from `docs/code-standards.md` and
`docs/project-overview.md` for quick reference. They are the rules
every implementation must follow.

1. **Brand is Educore.** Use **Educore** in prose and
   **`educore`** in code. No legacy names are permitted anywhere.
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
crates/domains/<domain>/           <-- directory (under the domains tier)
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
├── Cargo.toml                     <-- [package] name = "educore-<domain>"
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

A domain crate may depend on crates in the `infra` and
`cross-cutting` tiers, plus other domain crates in the
`domains` tier (only with explicit justification in an ADR).

A domain crate may **not** depend on:

- Any crate in the `adapters` tier
- Any crate in the `tools` tier
- `tokio` directly (only through `educore-core` re-exports where needed)
- `serde_json::Value`

External crate versions, MSRV pinning, and cross-compile status
(Linux, Android, WASM) are governed by
[`docs/decisions/ADR-015-ExternalCrates.md`](docs/decisions/ADR-015-ExternalCrates.md).

The `educore-core::lint` sub-module verifies these rules at
build time.

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
- **External crate selection policy.** All external crates are
  documented in
  [`docs/decisions/ADR-015-ExternalCrates.md`](docs/decisions/ADR-015-ExternalCrates.md),
  including the chosen version, the alternatives considered, the
  rationale, the cross-compile status (Linux, Android, WASM), and
  any MSRV-floor pinning. When adding a new external crate, update
  the ADR in the same commit. The 11 crates that exceed the
  engine's MSRV floor (1.75) are pinned to their last compatible
  line; the pinning policy is in § "MSRV floor conflict resolution"
  of the ADR.

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

Four reference adapters are shipped:

- `educore-storage-surrealdb` (primary target — embedded + server modes; see ADR-017)
- `educore-storage-postgres` (production target, PostgreSQL 14+)
- `educore-storage-mysql` (production target, MySQL 8.0+)
- `educore-storage-sqlite` (embedded / offline mode)

All four are scaffolded in `crates/adapters/` and shipped from
the engine. See `docs/ports/storage.md` for the port contract and
`docs/decisions/ADR-017-SurrealDBFirst.md` for the rationale.

## Engine Graph (graphify)

A pre-computed knowledge graph of the engine source
(`crates/`, `docs/`, `migrations/`, and the top-level `*.md`
files) lives at `graphify-out/` at the repo root. Read
`graphify-out/GRAPH_REPORT.md` for the god nodes and community
structure. Use `graphify query "<question>"` from the repo root
to traverse the graph when investigating engine behavior, design
rationale, or spec-to-code traceability.

The graph is **auto-rebuilt on every commit** via the local
`graphify hook install` (one-time per-machine setup, AST-only
regen, no API cost). A git merge driver keeps
`graphify-out/graph.json` conflict-free across parallel commits.
The graph is **committed to the repo** for static browsing; the
volatile parts (cost metrics, cache) are gitignored. See
`.graphifyignore` at the repo root for the exclusion list.

The legacy `schoolify/graphify-out/` graph is **frozen** and
retained as a research artefact only. AGENTS.md does not direct
agents to it; it is not auto-rebuilt.

See [`docs/decisions/ADR-016-EngineGraph.md`](docs/decisions/ADR-016-EngineGraph.md)
for the full rationale. The sync engine that powers the
SurrealDB-first adapter strategy and the offline-client replication
topology is documented in
[`docs/decisions/ADR-018-SyncEngine.md`](docs/decisions/ADR-018-SyncEngine.md).

## Status

- Documentation: **complete** (~302 markdown files, 15 domain
  specs × 11 files each = 165 spec files).
- Workspace scaffold: **complete** (37 packages = 36 internal
  crates + 1 umbrella, virtual workspace).
- Storage adapters shipped: **SurrealDB (primary), PostgreSQL,
  MySQL, SQLite** — all four scaffolded in `crates/adapters/`.
- Implementation: **partial**. Phase 0 (foundation + SurrealDB
  adapter + sync engine port) and Phase 12 (CMS, per the
  per-crate annotations below) are landed; remaining phases are
  scaffolded with pending domain logic per `docs/build-plan.md`.
- Domain spec cleanup: **complete**. All legacy-prefixed table
  references (the seven common Laravel/InfixEdu prefixes plus
  the brand-tainted Rust type names) have been removed from
  `docs/specs/` and replaced with engine
  `<domain>_<aggregate>` names. 77 spec files updated,
  ~1033 insertions / ~1102 deletions.
- Build plan: **18 phases** (Phase 0..17) with coverage matrix and
  no-gaps gates documented in `docs/build-plan.md`. The 7 new
  crates are scaffolded and assigned to:
  - `educore-storage-parity` → Phase 0 (cross-adapter test suite)
  - `educore-audit` → Phase 2 (cross-cutting foundations)
  - `educore-operations` → Phase 14 (Settings + Operations)
  - `educore-testkit` → Phase 16 (Test infrastructure + SDK)
  - `educore-cli` → Phase 16 (Test infrastructure + SDK)
  - `educore-sync` → Phase 0 (sync engine port trait, per ADR-018)
  - `educore-sync-inprocess` → Phase 0 (in-process SyncAdapter reference)

## Crate Inventory (per-crate phase assignment)

Every one of the 37 packages is scaffolded. Implementation begins in
Phase 0 of `docs/build-plan.md`. The table below maps each crate
to the phase that implements it. **This is the authoritative
source** — do not rely on the directory tree or the umbrella
re-exports to determine phase assignment.

| # | Tier | Crate | Phase | Title |
| --- | --- | --- | --- | --- |
| 1 | infra | `educore-core` | 0 | Foundation |
| 2 | infra | `educore-query-derive` | 0 | Foundation (proc-macro) |
| 3 | infra | `educore-storage` | 0 | Foundation (port trait) |
| 4 | adapters | `educore-storage-surrealdb` | 0 | Foundation (SurrealDB adapter, primary) |
| 5 | tools | `educore-storage-parity` | 0 | Foundation (cross-adapter test suite) |
| 6 | cross-cutting | `educore-sync` | 0 | Foundation (sync engine port trait, per ADR-018) |
| 7 | cross-cutting | `educore-sync-inprocess` | 0 | Foundation (in-process SyncAdapter reference, per ADR-018) |
| 8 | adapters | `educore-storage-postgres` | 1 | Adapter parity |
| 9 | adapters | `educore-storage-mysql` | 1 | Adapter parity |
| 10 | adapters | `educore-storage-sqlite` | 1 | Adapter parity |
| 11 | cross-cutting | `educore-platform` | 2 | Cross-cutting foundations |
| 12 | cross-cutting | `educore-rbac` | 2 | Cross-cutting foundations |
| 13 | cross-cutting | `educore-events` | 2 | Cross-cutting foundations (envelope) |
| 14 | adapters | `educore-event-bus` | 2 | Cross-cutting foundations (bus port) |
| 15 | cross-cutting | `educore-audit` | 2 | Cross-cutting foundations (audit log) |
| 16 | domains | `educore-academic` | 3 | Academic |
| 17 | domains | `educore-assessment` | 4 | Assessment |
| 18 | domains | `educore-attendance` | 5 | Attendance |
| 19 | domains | `educore-hr` | 6 | HR |
| 20 | domains | `educore-finance` | 7 | Finance |
| 21 | domains | `educore-facilities` | 8 | Facilities |
| 22 | domains | `educore-library` | 9 | Library |
| 23 | domains | `educore-communication` | 10 | Communication |
| 24 | domains | `educore-documents` | 11 | Documents |
| 25 | domains | `educore-cms` | 12 | CMS — spec-faithful (20 root aggregates per `docs/specs/cms/aggregates.md`); 9-file layout; ~67 events, ~67 commands, 86 Cms caps (4 retained Phase 2 placeholders + 82 net-new), 21 Cms audit targets, 19 repos, 19 query stubs, 6 service factory fns + 6 service structs (PageService, NewsService, ContentService, TestimonialService, HomeSliderService, ContentShareListService); `form_uploaded_public_indexing_subscriber` for `documents.form_download.uploaded` (Phase 11 OQ #6); `educore-academic` dep for `ClassId`/`SectionId`/`AcademicYearId`; 183 unit tests in crate + 7-scenario integration test in `storage-parity` (2 env-gated PG/MySQL variants); `SchoolId::PUBLIC` constant added to `educore-core`; 20 `coverage.toml` rows flipped; see `PHASE-12-HANDOFF.md` |
| 26 | domains | `educore-events-domain` | 13 | Events domain (calendar) |
| 27 | cross-cutting | `educore-settings` | 14 | Settings + Operations |
| 28 | cross-cutting | `educore-operations` | 14 | Settings + Operations |
| 29 | adapters | `educore-auth` | 15 | Port adapters |
| 30 | adapters | `educore-notify` | 15 | Port adapters |
| 31 | adapters | `educore-payment` | 15 | Port adapters |
| 32 | adapters | `educore-files` | 15 | Port adapters |
| 33 | adapters | `educore-integrations` | 15 | Port adapters |
| 34 | tools | `educore-testkit` | 16 | Test infrastructure + SDK |
| 35 | tools | `educore-storage-parity` | 16 | (Test infrastructure + SDK) |
| 36 | tools | `educore-sdk` | 16 | Test infrastructure + SDK |
| 37 | tools | `educore-cli` | 16 | Test infrastructure + SDK |
| — | umbrella | `educore` | 0 | re-exports only; first usable at Phase 0+ |

**Note on duplicates:** `educore-storage-parity` is listed at
both Phase 0 and Phase 16. Phase 0 scaffolds the crate; Phase 16
implements the actual test scenarios. The umbrella crate
`educore` is first usable once Phase 0 lands; its re-exports
are wired in `crates/educore/src/lib.rs`.

**Note on `educore-events` vs `educore-events-domain`:**
- `educore-events` (Phase 2) is the **event envelope + bus
  port** (DomainEvent trait, EventEnvelope, EventBus trait).
- `educore-events-domain` (Phase 13) is the **calendar domain**
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
4. **Adapter emission** — `educore-storage-<db>` walks the AST
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

The 6 cross-cutting tables have canonical DDL in four dialect
files under `migrations/engine/` (`0000_engine_core.mysql.sql`,
`0000_engine_core.postgres.sql`, `0000_engine_core.sqlite.sql`,
`0000_engine_core.surreal.surql`).
The `educore-storage-<db>` adapter crates `include_str!` these
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

## graphify

This project has a graphify knowledge graph at graphify-out/.

Rules:
- Before answering architecture or codebase questions, read graphify-out/GRAPH_REPORT.md for god nodes and community structure
- If graphify-out/wiki/index.md exists, navigate it instead of reading raw files
- For cross-module "how does X relate to Y" questions, prefer `graphify query "<question>"`, `graphify path "<A>" "<B>"`, or `graphify explain "<concept>"` over grep — these traverse the graph's EXTRACTED + INFERRED edges instead of scanning files
- After modifying code files in this session, run `graphify update .` to keep the graph current (AST-only, no API cost)
