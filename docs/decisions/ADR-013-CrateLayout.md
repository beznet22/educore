# ADR-013: Crate Layout

## Status

Accepted.

## Context

SMSengine is a school-domain engine. It is organized into
15 bounded contexts (academic, finance, hr, attendance,
assessment, library, facilities, communication,
documents, events, cms, platform, rbac, settings,
operations) plus a small set of cross-cutting
foundations and port implementations. Across these
contexts the engine contains ~310 domain tables,
~1500 aggregates, and tens of thousands of value
objects, commands, and events. The crate layout
must:

- **Scale past ~50 crates.** A flat `crates/<name>/`
  tree with 30+ sibling directories stops being
  navigable. A contributor should be able to answer
  "where do I look for X?" by reading the tier name
  alone.
- **Enforce layer boundaries.** A domain crate
  that imports an adapter is a layer-boundary
  violation; the same goes for a cross-cutting
  crate importing a domain. Convention is too soft;
  the boundary must be enforced at build time.
- **Preserve the "embed what you need" promise.**
  A consumer building a small admin tool may want
  only `academic` and `rbac`. The crate layout must
  not force them to pull in adapters, settings, or
  the entire umbrella.
- **Keep compile-time iteration fast.** A change to
  `smsengine-academic` should not trigger a rebuild
  of `smsengine-storage-postgres` or
  `smsengine-notify`. The 34-crate granularity is
  already at the right level; the layout must
  preserve it.
- **Avoid workspace metadata duplication.** A
  sub-workspace model in Cargo requires duplicating
  `[workspace.dependencies]` and `[workspace.lints]`
  in every sub-workspace's `Cargo.toml`. The layout
  should be enforceable from a single source of
  truth.

The naive approaches (one giant crate, flat sibling
crates, sub-workspaces) each fail one or more of
the constraints above. The 5-tier layout adopted by
this ADR resolves the tension by treating
**directory organization** as the primary grouping
mechanism, **single-root `[workspace]`** as the
metadata source, and a **lint sub-module** as the
boundary enforcement.

## Decision

SMSengine is organized as a single Cargo workspace
with **34 crates grouped into 5 tiers + 1 umbrella**.
The 5 tiers are directory-organized under `crates/`
and the single root `Cargo.toml` is the source of
truth for workspace metadata. Tier boundaries are
enforced at build time by a `smsengine-core::lint`
sub-module that statically inspects each crate's
declared dependencies.

Concretely:

### The 5 tiers

| Tier | Path | Count | Purpose |
| --- | --- | --- | --- |
| core | `crates/core/` | 3 | Infrastructure: errors, identifiers, value objects, query AST, proc-macro, storage port |
| cross-cutting | `crates/cross-cutting/` | 7 | Cross-domain foundations: platform, rbac, events, audit, settings, operations, calendar |
| domains | `crates/domains/` | 10 | The 10 domain bounded contexts (academic, finance, hr, ...) |
| adapters | `crates/adapters/` | 9 | Port implementations: 3 storage adapters + 6 port adapters (auth, event-bus, files, integrations, notify, payment) |
| tools | `crates/tools/` | 4 | Dev tooling: testkit, storage-parity, cli, sdk |
| umbrella | `crates/smsengine/` | 1 | Re-exports the public surface of all 34 internal crates |

### Dependency direction

```text
core  ←  cross-cutting  ←  domains  ←  tools
                          ↑
                          └──  adapters  (also depends on core + cross-cutting)
```

- `core` depends on nothing in the workspace.
- `cross-cutting` depends on `core`.
- `domains` depends on `core` and `cross-cutting`
  (and may depend on other `domains` crates only
  with explicit justification in an ADR).
- `adapters` depends on `core` and `cross-cutting`.
- `tools` depends on `core`, `cross-cutting`,
  `domains`, and `adapters`.
- The `smsengine` umbrella crate re-exports each
  internal crate under its short name
  (`pub use smsengine_core as core;`,
  `pub use smsengine_academic as academic;`, ...).
  Consumers therefore write
  `smsengine::academic::commands::*` and never need
  to know the internal `smsengine-` prefix on the
  package name.

### Boundary enforcement

The boundary is enforced at **two levels**:

1. **Glob patterns in the root `Cargo.toml`.** The
   virtual workspace uses one `members = [...]` glob
   per tier (`crates/core/*/Cargo.toml`,
   `crates/cross-cutting/*/Cargo.toml`, ...). This
   means a single `cargo build --workspace` covers
   the entire engine, and a single
   `[workspace.dependencies]` /
   `[workspace.lints]` block applies to all 34
   crates. We considered Cargo's sub-workspace
   feature (a `Cargo.toml` per tier with its own
   `[workspace]` table) and chose glob patterns
   instead: sub-workspaces require duplicating
   `[workspace.dependencies]` and
   `[workspace.lints]` in each sub-workspace's
   `Cargo.toml`, which is a high maintenance cost
   for no enforcement benefit.
2. **`smsengine-core::lint` sub-module.** A build-
   time check that walks every domain crate's
   declared dependencies and rejects any import
   that crosses a tier boundary upward (e.g. a
   `crates/domains/<x>/` crate that depends on a
   `crates/adapters/<y>/` crate, or a
   `crates/cross-cutting/<x>/` crate that depends
   on a `crates/domains/<y>/` crate). The lint
   sub-module is the **authoritative boundary
   enforcer**; the directory organization is for
   humans, the lint is for the compiler.

The lint sub-module is a **Phase 0 deliverable** of
`docs/build-plan.md` and lives in
`crates/core/engine-core/src/lint.rs`. See
`docs/build-plan.md` § "The No-Gaps Gates" for the
full gate list.

### Crate status

All 34 crates are scaffolded. Implementation begins
in Phase 0 of `docs/build-plan.md`.

The 5-tier layout was adopted in this restructure.
All 34 crates retain their `smsengine-<name>` package
names; only directory paths changed. The full path
mapping is in the table above and in `AGENTS.md` §
"Tier System".

## Rationale

### Why 5 tiers, not 3 (foundation/business/edges)

A 3-tier "foundation / business / edges" grouping
would mix the 6 cross-cutting foundations
(platform, rbac, events, audit, settings,
operations) with the 14 domain crates in a single
business tier. A contributor landing in the
`business/` directory would see 20+ crates and have
no way to tell, at a glance, which are foundations
and which are domains. Splitting cross-cutting out
into its own tier lets a domain contributor navigate
to `crates/domains/` and see the 10 domain crates
in isolation; cross-cutting foundations are a
deliberate hop away, and the dependency direction
makes that hop explicit. The 5-tier model also
isolates `tools/` (testkit, cli, sdk,
storage-parity) from the runtime crates, which
makes it obvious at a glance that `smsengine-cli`
is not part of the engine's release artifact.

### Why directory organization, not sub-workspaces

Cargo's sub-workspace feature (a `Cargo.toml` per
tier with its own `[workspace]` table) is a natural
way to group crates, but it has a real cost: every
sub-workspace's `Cargo.toml` must duplicate
`[workspace.dependencies]` and `[workspace.lints]`
(there is no `[workspace.dependencies]` inheritance
across sub-workspaces). With 5 tiers, that is 5
copies of the workspace metadata, all of which must
be kept in sync by hand. A single root `[workspace]`
with glob `members = [...]` patterns achieves the
same organizational benefit (one directory per
tier, one set of crates per directory) at zero
maintenance cost. The tier boundaries are still
enforced; they are enforced by
`smsengine-core::lint` rather than by Cargo
metadata.

### Why strict enforcement via lint, not convention

A domain crate that imports an adapter is a
layer-boundary violation. So is a cross-cutting
crate importing a domain crate. These violations
are easy to introduce by accident and hard to
spot in code review. The AGENTS.md file documents
the rule, but a contributor who skims a domain
crate's `Cargo.toml` and sees an adapter listed as
a dependency will not be stopped by AGENTS.md
alone. The `smsengine-core::lint` sub-module walks
the workspace at build time and rejects any
upward tier import. The cost is small (one module
of ~200 lines); the benefit is that the boundary
is enforced by the compiler rather than by
discipline.

### Migration history

The engine was first scaffolded as a flat 29-crate
layout under `crates/<name>/`. Five additional
crates (`smsengine-audit`, `smsengine-operations`,
`smsengine-testkit`, `smsengine-cli`,
`smsengine-storage-parity`) were added during the
v1 scaffold pass to reach 34. The 5-tier restructure
moved all 34 crates into the directory organization
described above; package names (`smsengine-<name>`)
and crate contents are unchanged. The full migration
is recorded in `docs/decisions/ADR-013-CrateLayout.md`
(this document) and cross-referenced from
`AGENTS.md` § "Tier System".

## Consequences

### Positive

- **Clean mental model.** A contributor knows which
  tier their crate is in (the rule is one of
  {core, cross-cutting, domains, adapters, tools,
  umbrella}); the tier is the first thing to
  communicate in a PR description.
- **Faster navigation.** `ls crates/domains/`
  shows the 10 domain crates in isolation.
  `ls crates/cross-cutting/` shows the 7 cross-
  cutting foundations. A contributor does not have
  to read 34 directory names to find the right
  crate.
- **Strict boundary enforcement.** A domain crate
  cannot import an adapter. A cross-cutting crate
  cannot import a domain crate. The compiler
  rejects the violation; the convention is not
  left to memory.
- **Per-tier CI parallelism.** A future change to
  a domain crate can build only the
  `crates/domains/<x>/` subtree and its
  transitive workspace dependencies, skipping
  the adapter crates entirely. (Not yet wired in
  CI; the build-plan reserves this for a future
  optimization.)
- **Single source of truth for workspace
  metadata.** One root `Cargo.toml` carries
  `[workspace.dependencies]` and
  `[workspace.lints]`; no tier-local copies to
  keep in sync.

### Negative

- **Tier paths are 1 level deeper.** A domain
  crate's source path is `crates/domains/<name>/`
  rather than `crates/<name>/`. Imports between
  crates use the same `smsengine_<name>` path as
  before, but the on-disk path is one level
  deeper.
- **5 tier directories, but no sub-workspace
  `Cargo.toml` files.** The single root
  `Cargo.toml` is the source of truth; contributors
  must not add a `Cargo.toml` at a tier root. This
  is a convention enforced by the lint sub-module
  and by code review.
- **Tier boundary enforcement requires the lint
  sub-module to be implemented.** The lint is a
  Phase 0 deliverable; until it lands, the tier
  boundaries are conventional, not enforced.
- **Glob patterns in the root `Cargo.toml` must be
  kept in sync with the tier layout.** Adding a
  sixth tier in the future requires editing the
  root `Cargo.toml`'s `members` glob. (This is a
  one-line change, not a refactor.)

## Alternatives Considered

| Alternative | Why not chosen |
| --- | --- |
| Flat 34-crate layout (`crates/<name>/`) | Works but doesn't scale past ~50 crates; no layer boundaries; a contributor landing in the repo sees 34 sibling directories with no signal as to which is which |
| Sub-workspaces (5 `[workspace]` files, one per tier) | Each sub-workspace needs its own `[workspace.dependencies]` and `[workspace.lints]`; high maintenance cost (5 copies of workspace metadata, all of which must be kept in sync by hand) |
| 3 tiers (foundation/business/edges) | 14 domain crates mixed with 6 cross-cutting foundations in the same tier; harder to navigate; the foundation/edges distinction doesn't map cleanly onto the engine's actual dependency graph |
| Per-domain repository (polyrepo) | 34 repos with separate version control; CI complexity; loses atomic commits across crates; cross-cutting refactors become multi-PR coordination problems |
| One giant `smsengine` crate | Fails on compile time, visibility control, consumer pull-in scope, and test isolation; see ADR-013 § "Context" for the full list |
| One crate per aggregate | Hundreds of crates; the relationships between aggregates of the same domain are too tight to justify the per-aggregate boilerplate |
| One crate per layer (commands/events/aggregates) | Separates the bounded context; a domain owns its commands, events, and aggregates together; separating them fragments the domain |

## Cross-References

- `AGENTS.md` § "Tier System" — the operator's
  summary of the 5-tier model and the dependency
  direction.
- `docs/build-plan.md` § "Tier System" and
  § "The No-Gaps Gates" — the build-time gates
  that depend on this layout (lint sub-module,
  dependency-direction checks).
- `docs/decisions/ADR-013-CrateLayout.md` — this
  document.
- `crates/smsengine/src/lib.rs` — the umbrella
  re-exports (`pub use smsengine_core as core;`,
  `pub use smsengine_academic as academic;`, ...).
