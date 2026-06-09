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
crates. The workspace today contains **34 crates**;
the inventory below is the canonical list.

Concretely:

**Foundation (2 crates)**

1. **`smsengine-core`** — the foundation crate. Error
   types, identifier trait, result type, value
   object trait, clock, id generator, common
   derives.
2. **`smsengine-platform`** — the multi-tenant
   substrate. `SchoolId`, `UserId`, `TenantContext`,
   `School`, `User` aggregates. Depends only on
   `smsengine-core`.

**Cross-cutting (3 crates)**

3. **`smsengine-rbac`** — role and capability
   management. Depends on `smsengine-core`,
   `smsengine-platform`, `smsengine-events`.
4. **`smsengine-events`** — event bus port, envelope,
   schema registry, outbox. Depends only on
   `smsengine-core`.
5. **`smsengine-audit`** — audit log port, query port,
   retention policy, redactor. Depends only on
   `smsengine-core`. (Added in v1 scaffold; see
   `crates/audit/`.)

**Per-tenant (1 crate)**

6. **`smsengine-settings`** — per-tenant configuration
   registry. Depends on `smsengine-core`,
   `smsengine-platform`.

**Domain (12 crates)**

7. `smsengine-academic`
8. `smsengine-assessment`
9. `smsengine-attendance`
10. `smsengine-finance`
11. `smsengine-hr`
12. `smsengine-library`
13. `smsengine-facilities`
14. `smsengine-communication`
15. `smsengine-documents`
16. `smsengine-events-domain`
17. `smsengine-cms`
18. `smsengine-operations` (added in v1 scaffold;
    see `crates/operations/`.)

**Storage (5 crates)**

19. **`smsengine-storage`** — the storage port trait.
20. **`smsengine-storage-postgres`** — PostgreSQL
    adapter (primary target).
21. **`smsengine-storage-mysql`** — MySQL 8.0+
    adapter (production target).
22. **`smsengine-storage-sqlite`** — SQLite adapter
    (embedded / offline mode).
23. **`smsengine-storage-parity`** — cross-dialect
    conformance test harness. (Added in v1
    scaffold; see `crates/storage-parity/`.) Not
    a runtime adapter; it runs the same query
    suite against all three adapters and asserts
    identical observable behavior.

**Port adapters (6 crates)**

24. **`smsengine-auth`** — authentication and
    identity.
25. **`smsengine-event-bus`** — concrete event
    bus implementations.
26. **`smsengine-files`** — file storage port.
27. **`smsengine-integrations`** — third-party
    integration adapters.
28. **`smsengine-notify`** — notification delivery
    (email, SMS, push).
29. **`smsengine-payment`** — payment gateway port.

**Proc-macro (1 crate)**

30. **`smsengine-query-derive`** — the
    `#[derive(DomainQuery)]` proc macro. Depends
    on `syn`, `quote`, the compiler. This is the
    only proc-macro crate in v1; additional
    derives are added in subsequent phases.

**Test infrastructure (1 crate)**

31. **`smsengine-testkit`** — in-memory test
    adapters for every port. Depends on every
    domain crate. (Added in v1 scaffold; see
    `crates/testkit/`.)

**Operator tooling (1 crate)**

32. **`smsengine-cli`** — operator tool. Optional.
    (Added in v1 scaffold; see `crates/cli/`.)

**High-level SDK (1 crate)**

33. **`smsengine-sdk`** — consumer-facing high-level
    SDK on top of the umbrella.

**Umbrella (1 crate)**

34. **`smsengine`** — the facade crate. Re-exports
    the engine surface as a single, stable
    `smsengine::Engine` API. Consumers depend on
    `smsengine` (or, for finer control, on individual
    domain crates). The 34 re-exports are listed in
    `crates/smsengine/src/lib.rs`.

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
corresponding crate. The `smsengine` facade re-exports
the surface for convenience.

### Crate status

All 34 crates are scaffolded. Implementation begins
in Phase 0 of `docs/build-plan.md`.

### Positive

- **Compile time scales linearly per domain.** A
  change to `smsengine-academic` does not trigger
  a rebuild of `smsengine-finance`. The iteration
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
- **Test isolation.** Tests in `smsengine-finance`
  do not run when `smsengine-academic` is built.

### Negative

- **More crates, more boilerplate.** Each crate
  has its own `Cargo.toml`, `lib.rs`, README.
  This is paid once per crate, not per release.
- **Cross-domain types are an explicit
  re-export.** `StudentId` is defined in
  `smsengine-academic`; consumers get it from
  `smsengine` or from `smsengine-academic` directly.
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

- The `smsengine` facade crate re-exports the
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

All in one `smsengine` crate. Rejected per above:
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
no `smsengine` facade

Consumers depend on individual crates only.
Rejected because the facade provides a
stable, well-known entry point and reduces
"which crate do I depend on for
`Engine`?" friction. The facade is opt-in
by name; consumers can still depend on
individual crates directly.
