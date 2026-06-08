# ADR-013: One Crate per Domain

## Status

Accepted.

## Context

SMSengine is a school-domain engine. It is organized into
15 bounded contexts: academic, finance, hr, attendance,
assessment, library, facilities, communication,
documents, events, cms, platform, rbac, settings,
operations. Each context has its own aggregates, value
objects, commands, events, repositories, services, and
policies.

The naive approach is a single Rust crate with one
module per domain. This compiles fast and is easy to
navigate in a small codebase. It does not scale:

- **Compile time** — every change to any domain
  triggers a full rebuild of the engine. With 15
  domains and hundreds of aggregates, the
  iteration loop slows to minutes.
- **Visibility control** — `pub` in a single crate
  exposes everything to everything else. The
  domain boundaries become conventions, not
  enforced rules.
- **Versioning** — a consumer who wants only the
  academic domain pulls in finance, hr, and the
  rest. The "embed what you need" promise breaks.
- **Testing** — tests for one domain drag in the
  dependencies of every other domain.
- **Refactoring** — moving an aggregate from one
  domain to another is a single-crate chore.

The school domain's bounded contexts are also
**independently consumable**. A consumer who is
building a small admin tool may want only
`academic` and `rbac`. A consumer building a finance-
heavy product may want only `finance` and `rbac`.
The engine's crate layout should reflect this.

## Decision

SMSengine is organized as a Cargo workspace with **one
crate per domain**, plus a small set of shared
crates.

Concretely:

1. **`smscore-core`** — the foundation crate. Error
   types, identifier trait, result type, value
   object trait, clock, id generator, common
   derives.
2. **`smscore-platform`** — the multi-tenant
   substrate. `SchoolId`, `UserId`, `TenantContext`,
   `School`, `User` aggregates. Depends only on
   `smscore-core`.
3. **`smscore-rbac`** — role and capability
   management. Depends on `smscore-core`,
   `smscore-platform`, `smscore-events`.
4. **`smscore-events`** — event bus port, envelope,
   schema registry, outbox. Depends only on
   `smscore-core`.
5. **`smscore-audit`** — audit log port, query port,
   retention policy, redactor. Depends only on
   `smscore-core`.
6. **`smscore-settings`** — per-tenant configuration
   registry. Depends on `smscore-core`,
   `smscore-platform`.
7. **One crate per domain**:
   - `smscore-academic`
   - `smscore-assessment`
   - `smscore-attendance`
   - `smscore-finance`
   - `smscore-hr`
   - `smscore-library`
   - `smscore-facilities`
   - `smscore-communication`
   - `smscore-documents`
   - `smscore-events-domain`
   - `smscore-cms`
   - `smscore-operations`
8. **`smscore`** — the facade crate. Re-exports
   the engine surface as a single, stable
   `smscore::Engine` API. Consumers depend on
   `smscore` (or, for finer control, on individual
   domain crates).
9. **`smscore-macros`** — derive macros shared
   across crates. Depends on `syn`, `quote`, the
   compiler.
10. **`smscore-testkit`** — in-memory test
    adapters for every port. Depends on every
    domain crate.
11. **`smscore-cli`** — operator tool. Optional.

Each crate has:

- `Cargo.toml` with pinned dependencies.
- `src/lib.rs` describing what the crate owns and
  what it depends on.
- `src/<module>.rs` per file in the standard
  layout (`aggregate.rs`, `entities.rs`,
  `value_objects.rs`, `commands.rs`, `events.rs`,
  `services.rs`, `policies.rs`, `repository.rs`,
  `query.rs`, `errors.rs`).
- `tests/` with integration tests.
- `README.md` summarizing the crate's purpose.

A consumer enables a domain by depending on the
corresponding crate. The `smscore` facade re-exports
the surface for convenience.

## Consequences

### Positive

- **Compile time scales linearly per domain.** A
  change to `smscore-academic` does not trigger
  a rebuild of `smscore-finance`. The iteration
  loop is fast.
- **Visibility is enforced.** Each domain's types
  are `pub` within the crate and `pub` across
  the crate boundary through explicit re-exports.
  A domain that wants to hide an internal type
  does so without ceremony.
- **Consumers pull in only what they need.** A
  small admin tool that needs only `academic`
  and `rbac` compiles with only those two
  crates (and their transitive deps).
- **Refactoring is mechanical.** Moving an
  aggregate from one domain to another is a
  series of file moves and visibility tweaks.
- **Domain boundaries are visible in the
  dependency graph.** A consumer can audit
  "which crates depend on which?" with
  `cargo tree`.
- **Test isolation.** Tests in `smscore-finance`
  do not run when `smscore-academic` is built.

### Negative

- **More crates, more boilerplate.** Each crate
  has its own `Cargo.toml`, `lib.rs`, README.
  This is paid once per crate, not per release.
- **Cross-domain types are an explicit
  re-export.** `StudentId` is defined in
  `smscore-academic`; consumers get it from
  `smscore` or from `smscore-academic` directly.
  The duplication is not real (it's re-exports),
  but the choice is the consumer's.
- **Workspace-wide refactors touch many
  `Cargo.toml` files.** A version bump of a
  shared crate updates the workspace's
  `Cargo.toml` and every dependent crate's
  `Cargo.toml`. This is mechanical.
- **Discoverability.** A consumer landing in
  the repo sees 25+ crates. The
  `architecture.md` and the per-crate README
  mitigate this.

### Mitigations

- The `smscore` facade crate re-exports the
  most common types, so a consumer who wants
  the simple path can depend on a single
  crate.
- The workspace's top-level `Cargo.toml`
  groups the crates and sets the workspace
  dependencies.
- The `code-standards.md` documents the
  per-crate layout and the standard module
  files.
- A `cargo xtask` tool scaffolds a new domain
  crate with the standard layout.

## Alternatives Considered

### 1. Single crate, many modules

All in one `smscore` crate. Rejected per above:
visibility, compile time, and consumption
control suffer.

### 2. One crate per aggregate

Hundreds of crates, one per `Student`,
`Invoice`, etc. Rejected because the
relationships between aggregates of the same
domain are too tight to justify the per-
aggregate boilerplate.

### 3. One crate per layer (commands,
events, aggregates)

`commands/`, `events/`, `aggregates/` as
separate crates. Rejected because a domain
owns its commands, its events, and its
aggregates together; separating them
fragments the bounded context.

### 4. One crate per command or event

Ultra-granular. Rejected for the same reasons
as (2), amplified.

### 5. Workspace per deployment shape
(SaaS, on-premise, mobile)

Three workspaces with overlapping crates.
Rejected because the engine's code path is
the same; the deployment shape is a runtime
configuration, not a compile-time one.

### 6. Hybrid: one crate per domain, but
no `smscore` facade

Consumers depend on individual crates only.
Rejected because the facade provides a
stable, well-known entry point and reduces
"which crate do I depend on for
`Engine`?" friction. The facade is opt-in
by name; consumers can still depend on
individual crates directly.
