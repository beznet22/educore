# Contributing to Educore

Welcome. Educore is a school-domain engine — a typed, multi-tenant,
event-sourced kernel that captures the business behavior, workflows,
and rules required to operate a real school. Contributions add
aggregates, commands, events, port implementations, or engine
cross-cutting features. This document walks the workflow from spec
to pull request.

> **If you are an AI agent**, read [`AGENTS.md`](AGENTS.md) instead.
> That document has the AI-specific instructions (commit attribution,
> lint commands, machine-readable inputs). The two documents
> cross-link; both are required reading.

## Read first

Before contributing, read these documents in order:

1. [`AGENTS.md`](AGENTS.md) — orientation, crate layout, validation
   checklist (humans skim; AI agents read in full)
2. [`docs/project-overview.md`](docs/project-overview.md) — engine
   philosophy and scope
3. [`docs/architecture.md`](docs/architecture.md) — the system map
4. [`docs/build-plan.md`](docs/build-plan.md) — the 17-phase
   implementation roadmap, the coverage matrix, and the no-gaps
   gates
5. [`docs/code-standards.md`](docs/code-standards.md) — the
   engineering rules (must follow)
6. [`docs/coverage.toml`](docs/coverage.toml) — the machine-readable
   coverage matrix (the source of truth for "is item X
   implemented?")

The 17-phase plan tells you which crate to contribute to and which
phase owns it. The coverage matrix tells you which spec rows are
still `Pending`. The code standards tell you the rules you must
follow.

## The 6-step contribution workflow

Every contribution follows the same six steps, regardless of
whether you are adding a new aggregate, a new command, a new event,
a new port implementation, or a new engine cross-cutting feature.

### Step 1: Spec

Pick the right spec doc and add the new row.

- **New aggregate** → `docs/specs/<domain>/aggregates.md` (full
  contract) and the per-domain command catalog in
  `docs/commands/<domain>.md` if it ships in commands
- **New command** → `docs/specs/<domain>/commands.md` (full
  contract) and the catalog in `docs/commands/<domain>.md`
- **New event** → `docs/specs/<domain>/events.md` (full contract)
  and the catalog in `docs/events/<domain>.md`
- **New value object** → `docs/specs/<domain>/value-objects.md`
- **New port trait** → `docs/ports/<port>.md` (port contracts
  folder, 7 files)
- **New cross-cutting table** → `migrations/engine/0000_engine_core.<dialect>.sql`
  (one file per dialect: `.mysql.sql`, `.postgres.sql`, `.sqlite.sql`)

The spec doc carries the **design contract**. It is human-readable,
not executable. Include:

- A one-line description of the new feature
- The Rust struct definition in a ` ```rust ` block
- The invariants (`NOT NULL`, `CHECK`, `UNIQUE`, `FK`)
- The indexes (especially `(school_id, active_status)`)
- The events emitted (for commands) or the subscribers (for events)
- The capability strings required (for commands)

### Step 2: Code

Create or extend the right file in the crate that owns the feature
(per [`AGENTS.md`](AGENTS.md) § "Crate Inventory" and §
"Tier System"):

- New aggregate struct → `crates/domains/<domain>/src/aggregate.rs`
  with `#[derive(DomainQuery)]`
- New command handler → `crates/domains/<domain>/src/commands.rs`
- New event struct → `crates/domains/<domain>/src/events.rs`
- New value object → `crates/domains/<domain>/src/value_objects.rs`
- New port trait impl → `crates/adapters/<port-adapter>/src/`
- New cross-cutting table → no Rust code needed; the adapter
  `include_str!`s the DDL from `migrations/engine/`

Follow the **module layout** in
[`docs/code-standards.md`](docs/code-standards.md) § "Module Layout
(per domain)" and the **naming convention** in
[`AGENTS.md`](AGENTS.md) § "Naming Convention (Enforced)". Every
public type must be documented (rustdoc, `#![deny(missing_docs)]`).

### Step 3: Test

Add the corresponding test to `crates/domains/<domain>/tests/`.
The seven test files per domain, with their purpose, are:

| File                                            | What it tests                                |
| ----------------------------------------------- | -------------------------------------------- |
| `crates/domains/<d>/tests/aggregate_fields.rs`  | Field-level invariants from `aggregates.md`  |
| `crates/domains/<d>/tests/commands.rs`          | Command handlers from `commands.md`          |
| `crates/domains/<d>/tests/events.rs`            | Event envelopes from `events.md`             |
| `crates/domains/<d>/tests/services.rs`          | Domain services from `services.md`           |
| `crates/domains/<d>/tests/repository.rs`        | Repository port methods from `repositories.md` |
| `crates/domains/<d>/tests/value_objects.rs`     | Value-object validation from `value-objects.md` |
| `crates/domains/<d>/tests/workflows.rs`         | Multi-aggregate workflows from `workflows.md` |

Every test must:

- Reference the spec doc it implements via a comment header:
  `// Implements: docs/specs/academic/aggregates.md#student-admit`
- Cover the happy path **and** at least one error path
  (per [`AGENTS.md`](AGENTS.md) § "Agent Instructions" → "Testing")
- Validate a real-world scenario (round-trip serialization, error
  propagation, trait object dispatch, multi-tenant isolation, etc.)
- Not be a "dummy" test (e.g. `assert!(true)`, `fn it_works()`)
- Not use `unwrap()` or `expect()` in the test body; use `?` and
  return `Result<(), Error>` from the test function

### Step 4: Coverage matrix

Add a row to [`docs/coverage.toml`](docs/coverage.toml) for the
new item. The schema is:

```toml
[[row]]
id      = "stable_snake_case_identifier"
item    = "Human-readable name"
spec    = "relative/path/to/spec.md"
crate   = "educore-<name>"
phase   = 0                # 0..17 per docs/build-plan.md
status  = "Pending"        # Pending | Implemented | Tested | Deprecated
  tests   = "crates/domains/<d>/tests/<file>.rs"  # only when status >= Tested
notes   = "optional free-form note"
```

The status progresses: `Pending` (spec'd) → `Implemented`
(code lands) → `Tested` (integration test passes). `Tested` is
the terminal state; `Deprecated` is the alternative terminal
state.

A PR that adds or removes an item in `docs/specs/` or
`crates/domains/<d>/` without a corresponding change to
`docs/coverage.toml` **fails CI**. The matrix diff is the per-PR
gate.

### Step 5: Lint

Run the cross-reference lint sub-module to verify the spec ↔ code
parity, anti-pattern checks, and matrix sync:

```bash
cargo run -p educore-core --bin lint --features lint
```

The lint is a sub-module of `educore-core` (not a separate
crate), gated behind the `lint` Cargo feature flag. It runs in CI
on every PR and locally on demand. It catches:

- Missing handlers (a spec row with no corresponding code)
- Undocumented public items (code with no corresponding spec row)
- Anti-patterns: `unimplemented!()`, `todo!()`, `// TODO: implement`
  in production code; `as` on numerics; `serde_json::Value` in
  domain code; `HashMap<String, T>` for domain data
- Coverage matrix lies: rows marked `Tested` with no test path,
  or rows referencing spec files that don't exist

If the lint fails, fix the code or the spec — do not silence the
lint.

### Step 6: PR

Before opening a pull request, run the validation checklist in
[`AGENTS.md`](AGENTS.md) § "Validation Checklist (per PR)":

- [ ] `cargo build --workspace` passes
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
      passes
- [ ] `cargo fmt --all -- --check` passes
- [ ] All hand-written tests pass
- [ ] `educore-core::lint` is clean
- [ ] `docs/coverage.toml` is updated and the matrix diff is
      clean
- [ ] No legacy table prefixes (`sm_`, `fm_`, `infix_`, etc.) —
      see the cleanup status in
      [`AGENTS.md`](AGENTS.md) § "Status"
- [ ] Public items are documented (rustdoc)
- [ ] At least one integration test was added for the new
      behavior
- [ ] Diagrams and ADRs updated if the change affects structure
      or architecture

Commit message format follows the existing repository style
(one-line summary, optional body, blank line, optional trailer).
The commit message must not introduce legacy brand references.

## Where things live

Quick reference for new contributors. The 17-phase crate assignment
is the source of truth (see [`AGENTS.md`](AGENTS.md) § "Crate
Inventory").

| You want to add...            | Edit or create...                                                |
| ----------------------------- | ---------------------------------------------------------------- |
| New aggregate                 | `docs/specs/<domain>/aggregates.md` + `crates/domains/<domain>/src/aggregate.rs` |
| New command                   | `docs/specs/<domain>/commands.md` + `docs/commands/<domain>.md` (catalog) + `crates/domains/<domain>/src/commands.rs` |
| New event                     | `docs/specs/<domain>/events.md` + `docs/events/<domain>.md` (catalog) + `crates/domains/<domain>/src/events.rs` |
| New value object              | `docs/specs/<domain>/value-objects.md` + `crates/domains/<domain>/src/value_objects.rs` |
| New domain service            | `docs/specs/<domain>/services.md` + `crates/domains/<domain>/src/services.rs` |
| New repository method         | `docs/specs/<domain>/repositories.md` + `crates/domains/<domain>/src/repository.rs` |
| New workflow (multi-aggregate) | `docs/specs/<domain>/workflows.md` + `crates/domains/<domain>/tests/workflows.rs` |
| New port impl                 | `docs/ports/<port>.md` + `crates/adapters/<port-adapter>/src/`   |
| New engine cross-cutting table | `migrations/engine/0000_engine_core.<dialect>.sql` (3 files) + `docs/schemas/<topic>.md` |
| New ADR                       | `docs/decisions/ADR-NNN-<title>.md` (next number, status `Accepted`) |
| New diagram                   | `docs/diagrams/<name>.md` (Mermaid in a ` ```mermaid ` block)    |
| New research note             | `docs/research/<name>.md`                                          |

## Tier System

The 34 crates are organized into 5 tiers + 1 umbrella. The
tier system enforces a layered dependency direction:

```text
infra  ←  cross-cutting  ←  domains  ←  tools
                           ↑
                           └──  adapters  (depends on infra + cross-cutting)
```

| Tier | Path | What lives here |
| --- | --- | --- |
| **infra** | `crates/infra/` | Infrastructure: errors, identifiers, value objects, query AST, proc-macro, storage port |
| **cross-cutting** | `crates/cross-cutting/` | Cross-domain foundations: platform, rbac, events (envelope), events-domain (calendar), settings, operations, audit |
| **domains** | `crates/domains/` | The 10 domain bounded contexts (academic, finance, hr, etc.) |
| **adapters** | `crates/adapters/` | Port implementations: 3 storage adapters + 6 port adapters |
| **tools** | `crates/tools/` | Dev tooling: testkit, storage-parity, cli, sdk |

When adding a new feature, **identify its tier first**:

- Adding a new error type, identifier, or query AST node → `infra`
- Adding a new cross-domain foundation (e.g. a new port) →
  `cross-cutting` (port trait) or `adapters` (impl)
- Adding a new bounded context or aggregate → `domains`
- Adding a new testkit or CLI command → `tools`

The `educore-core::lint` sub-module verifies the tier
boundaries at build time (the "No-Gaps Gates" mechanism; see
the Read-first list above for the build-plan link).

## Spec authoring rules

A good spec doc is **typed, testable, and complete**. It is the
design contract that the implementation must satisfy. Follow these
rules:

- **Use the 11-file spec folder layout** per domain
  (`overview.md`, `aggregates.md`, `commands.md`, `events.md`,
  `entities.md`, `value-objects.md`, `services.md`, `permissions.md`,
  `repositories.md`, `workflows.md`, `tables.md`). See
  [`docs/code-standards.md`](docs/code-standards.md) § "Spec folder
  layout" for the 11-file mapping.
- **Use the `<domain>_<aggregate>` naming convention** for tables
  (snake_case, no prefixes, no `sm_`/`fm_`/`infix_` legacy
  prefixes — the cleanup is complete, see
  [`AGENTS.md`](AGENTS.md) § "Status").
- **Use macro-generated enums** for field references, not string
  field names (e.g. `StudentField::Status`, not `"status"`).
- **Include the Rust struct definition** in a ` ```rust ` block.
  This is the type contract; the macro emits the AST from it.
- **List the engine invariants**: `id`, `school_id`, `active_status`,
  `created_at`, `updated_at`, `created_by`, `updated_by`, `version`,
  `etag`, `last_event_id`, `correlation_id`, `source`. See
  [`docs/schemas/database-schema.md`](docs/schemas/database-schema.md)
  for the canonical minimum schema.
- **List the indexes** (especially `(school_id, active_status)`).
- **Cross-link related docs** instead of duplicating content.

## Code authoring rules

Quick pointer list; the full rules are in
[`docs/code-standards.md`](docs/code-standards.md) and
[`AGENTS.md`](AGENTS.md) § "Agent Instructions" → "Type Safety".

- Rust edition 2021, MSRV 1.75, `unsafe` forbidden in domain
  code.
- All public APIs are documented with rustdoc;
  `#![deny(missing_docs)]`.
- No `unwrap()`/`expect()`/`panic!()` in production paths;
  propagate errors via `?`.
- No `as` casts that truncate or lose data; use `TryFrom`/`TryInto`.
- All fallible APIs return `Result<T, DomainError>`; use
  `thiserror` for public APIs, `anyhow` for glue.
- `Send + Sync` preserved for all public async types.
- No `serde_json::Value` in domain code; use typed wrappers.
- No `HashMap<String, T>` for domain data.
- No service locators, DI containers, or runtime reflection.
- All dependencies use `rustls`; never `native-tls`.
- Trait objects must be object-safe; verify with
  `let _: Box<dyn Trait>;` compile tests.
- **External crate policy.** All external crates are documented in
  [`docs/decisions/ADR-015-ExternalCrates.md`](docs/decisions/ADR-015-ExternalCrates.md).
  Adding a new external crate requires: (a) `cargo add <crate>
  --workspace`, (b) cross-compile check
  (`cargo build --target aarch64-linux-android` and
  `--target wasm32-unknown-unknown`), (c) ADR update in the same
  commit. The MSRV-floor pinning policy is in
  § "MSRV floor conflict resolution".

## Test authoring rules

The full TDD rules are in [`AGENTS.md`](AGENTS.md) § "Agent
Instructions" → "Testing". Quick summary:

- **No dummy tests** — every test must validate a real-world
  scenario.
- **Test error paths** — verify malformed input, unknown tool
  names, provider failures, and iteration limits produce the
  correct `Result::Err` or fallback behavior.
- **Reference the spec** — every test file starts with a comment
  header pointing to the spec doc it implements.
- **At least one integration test per PR** — unit tests alone are
  not sufficient.

## The coverage matrix workflow

[`docs/coverage.toml`](docs/coverage.toml) is the single source of
truth for "is item X implemented?". The CI step on every PR is
`git diff --exit-code docs/coverage.toml` on PRs that touch
`docs/specs/` or `crates/domains/<d>/`. A PR that adds an aggregate
without a matrix row fails.

When you add a new item:

1. **Add the spec row first** (Step 1 of the workflow).
2. **Add the coverage matrix row** with `status = "Pending"`. This
   is the spec-only state; the lint sub-module will not fail until
   the spec is implemented, but the new `Pending` row is itself a
   flagged entry that the next PR must address.
3. **Implement the code** (Step 2). Update the matrix row to
   `status = "Implemented"`.
4. **Add the integration test** (Step 3). Update the matrix row to
   `status = "Tested"` and set the `tests` field to the test path.
   `Tested` is the terminal state.
5. **Deprecating an item** → set `status = "Deprecated"` and add a
   note pointing to the replacement. Deprecated rows are exempt
   from the per-PR gate.

The full schema is documented in
[`docs/build-plan.md`](docs/build-plan.md) § "The Coverage Matrix".

## The lint sub-module

The cross-reference lint is a sub-module of `educore-core`
(not a separate crate), gated by the `lint` Cargo feature flag.
It is implemented in `educore-core::lint` (the source file is
`crates/infra/core/src/lint.rs` under the new tier-based layout)
and exposed as a binary.

Run it locally with:

```bash
cargo run -p educore-core --bin lint --features lint
```

The `-p educore-core` flag works because the package name is
unchanged even though the source directory moved from
`crates/educore-core/` to `crates/infra/core/`.

It runs in CI on every PR. It catches:

- **Spec → code**: every `docs/specs/<domain>/tables.md` row has
  a corresponding `#[derive(DomainQuery)]` struct in
  `crates/domains/<domain>/src/aggregate.rs`.
- **Code → spec**: every public struct, command, and event has a
  spec row. The build fails on undocumented public items.
- **Anti-patterns**: `unimplemented!()`, `todo!()`, `// TODO: implement`
  in production code; `as` on numerics; `serde_json::Value` in
  domain code; `HashMap<String, T>` for domain data.
- **Coverage matrix sync**: rows marked `Tested` have a `tests`
  path that exists; rows reference spec files that exist;
  code-defined aggregates/commands/events have a row.

If the lint fails, fix the code or the spec — do not silence the
lint.

## The engine graph

A pre-computed knowledge graph of the engine source lives at
`graphify-out/` at the repo root (committed). Read
`graphify-out/GRAPH_REPORT.md` for the god nodes and community
structure; use `graphify query "<question>"` from the repo root
to traverse. The graph auto-rebuilds on every commit via
`graphify hook install` (one-time per-user setup; AST-only
regen, no API cost). A git merge driver keeps `graph.json`
conflict-free across parallel commits. See
[AGENTS.md § "Engine Graph (graphify)"](AGENTS.md#engine-graph-graphify)
and [`docs/decisions/ADR-016-EngineGraph.md`](docs/decisions/ADR-016-EngineGraph.md)
for the full reference. Note: this is a per-user CI gate;
contributors do NOT need to manually update the graph.

## Worked example: adding a new aggregate to `educore-academic`

Let's add a new `academic_guardian_relationships` aggregate that
links a student to a guardian with a relationship type (parent,
grandparent, sibling, foster, etc.). This is a real-world feature
in school management systems.

### Step 1: Spec

Edit `docs/specs/academic/aggregates.md`. Add a row to the table:

```markdown
| `academic_guardian_relationships` | GuardianRelationship | Links a student to a guardian with a relationship type and contact priority |
```

Then add the Rust struct definition in a ` ```rust ` block (this
becomes the type contract that the macro reads):

```rust
pub struct GuardianRelationship {
    pub id: GuardianRelationshipId,
    pub school_id: SchoolId,
    pub student_id: StudentId,
    pub guardian_id: GuardianId,
    pub relationship_type: RelationshipType,  // enum: Parent, Grandparent, Sibling, Foster, Other
    pub is_primary: bool,                    // exactly one per (student, school) is true
    pub is_emergency_contact: bool,
    pub pickup_authorized: bool,
    pub active_status: ActiveStatus,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub version: Version,
    pub etag: Etag,
    pub last_event_id: Option<EventId>,
    pub correlation_id: Option<CorrelationId>,
    pub source: Option<Source>,
}
```

Document the invariants: `UNIQUE (school_id, student_id, guardian_id)`;
`CHECK (is_primary = TRUE) implies exactly one row per
(school_id, student_id)` (enforced in service, not SQL); the
`(school_id, student_id)` index for the "list a student's
guardians" query.

### Step 2: Code

Edit `crates/domains/academic/src/aggregate.rs` (or create the
file if the crate is in scaffold state). Add:

```rust
use educore_core::prelude::*;
use educore_query_derive::DomainQuery;

#[derive(DomainQuery)]
#[domain_query(
    table = "academic_guardian_relationships",
    aggregate = "GuardianRelationship",
)]
pub struct GuardianRelationshipRow {
    pub id: UuidV7,
    pub school_id: UuidV7,
    pub student_id: UuidV7,
    pub guardian_id: UuidV7,
    pub relationship_type: String,  // enum-as-string per engine convention
    pub is_primary: bool,
    pub is_emergency_contact: bool,
    pub pickup_authorized: bool,
    pub active_status: i8,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UuidV7,
    pub updated_by: UuidV7,
    pub version: i64,
    pub etag: String,
    pub last_event_id: Option<UuidV7>,
    pub correlation_id: Option<UuidV7>,
    pub source: Option<String>,
}
```

The macro emits the `EntityDescriptor` AST. The storage adapter
walks the AST at `storage.create_schema()` time and emits the
per-dialect DDL — no manual SQL.

### Step 3: Test

Edit (or create) `crates/domains/academic/tests/aggregate_fields.rs`:

```rust
// Implements: docs/specs/academic/aggregates.md#guardian-relationship
use educore_academic::aggregate::*;

#[test]
fn guardian_relationship_invariants() -> Result<(), Box<dyn std::error::Error>> {
    let rel = GuardianRelationshipRow {
        id: UuidV7::new(),
        school_id: UuidV7::new(),
        student_id: UuidV7::new(),
        guardian_id: UuidV7::new(),
        relationship_type: "Parent".to_string(),
        is_primary: true,
        is_emergency_contact: true,
        pickup_authorized: true,
        active_status: 1,
        created_at: Timestamp::now(),
        updated_at: Timestamp::now(),
        created_by: UuidV7::new(),
        updated_by: UuidV7::new(),
        version: 1,
        etag: "etag-1".to_string(),
        last_event_id: None,
        correlation_id: None,
        source: None,
    };

    // Happy path: invariants hold
    assert!(rel.is_primary);
    assert!(rel.is_emergency_contact);
    assert!(rel.pickup_authorized);
    assert_eq!(rel.relationship_type, "Parent");

    // Error path: at most one is_primary per (school_id, student_id)
    // (enforced by the service layer, not the row constructor)
    Ok(())
}
```

Add tests for the error paths too: a second `is_primary = true`
row for the same `(school_id, student_id)` should fail at the
service layer (the row constructor alone cannot enforce this).

### Step 4: Coverage matrix

Edit `docs/coverage.toml`. Add a new row at the bottom of the
Academic section:

```toml
[[row]]
id = "academic_guardian_relationships_aggregate"
item = "academic_guardian_relationships aggregate"
spec = "docs/specs/academic/aggregates.md"
crate = "educore-academic"
phase = 3
status = "Pending"
```

Once the code lands and the test passes, update the row to:

```toml
[[row]]
id = "academic_guardian_relationships_aggregate"
item = "academic_guardian_relationships aggregate"
spec = "docs/specs/academic/aggregates.md"
crate = "educore-academic"
phase = 3
status = "Tested"
  tests = "crates/domains/academic/tests/aggregate_fields.rs"
```

### Step 5: Lint

Run the lint sub-module to verify everything is in sync:

```bash
cargo run -p educore-core --bin lint --features lint
```

The lint checks that the new aggregate has a spec row (it does,
you just added it) and a code struct (it does, you just added it
in Step 2). It also checks that the matrix row is in `Pending`
state and the spec file exists.

### Step 6: PR

Run the full validation checklist:

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
cargo run -p educore-core --bin lint --features lint
```

Then commit with a descriptive message:

```text
academic: add GuardianRelationship aggregate

Adds the academic_guardian_relationships table, which links a student
to a guardian with a relationship type, primary flag, and pickup
authorization. Required by the contact-priority feature in the
admission workflow.

- spec: docs/specs/academic/aggregates.md (GuardianRelationship row)
- code: crates/domains/academic/src/aggregate.rs (GuardianRelationshipRow)
- test: crates/domains/academic/tests/aggregate_fields.rs (invariants)
- matrix: docs/coverage.toml (academic_guardian_relationships_aggregate)
```

Open the PR. CI runs:

- `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt`
- The lint sub-module (verifies spec ↔ code parity)
- `git diff --exit-code docs/coverage.toml` (verifies the matrix
  was updated)
- The legacy-prefix greps from the cleanup pass

The PR is mergeable once all checks pass.

## See also

- [`AGENTS.md`](AGENTS.md) — AI agent instructions, crate inventory,
  validation checklist
- [`docs/build-plan.md`](docs/build-plan.md) — the 17-phase plan,
  coverage matrix schema, no-gaps gates
- [`docs/code-standards.md`](docs/code-standards.md) — engineering
  rules, module layout, spec folder layout
- [`docs/coverage.toml`](docs/coverage.toml) — the machine-readable
  coverage matrix
- [`docs/architecture.md`](docs/architecture.md) — the system map
- [`docs/progress-tracker.md`](docs/progress-tracker.md) —
  per-crate and per-phase status
- [`docs/decisions/`](docs/decisions/) — 14 architectural
  decisions (ADRs)
