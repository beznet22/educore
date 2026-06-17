# Phase 0 Verification Prompt

> Per-phase verify prompt for Phase 0 (Foundation), rendered
> from `docs/verification/TEMPLATE.md`. Section A is **N/A**
> for Phase 0 (the phase is already closed and verified per
> the template's own instruction; skip it). Section B is
> the primary scope. The closing-of-close auto-fixes any
> disparities the verification subagent finds.

---

## Mission

Verify that Phase 0's forward-looking prompt
(`docs/phase_prompt/phase-0-prompt.md`), retrospective handoff
(`docs/handoff/PHASE-0-HANDOFF.md`), build-plan section
(`docs/build-plan.md` § "Phase 0"), and on-disk implementation
(`crates/infra/core/src/`, `crates/infra/query-derive/src/`,
`crates/infra/storage/src/`,
`crates/adapters/storage-surrealdb/src/`,
`crates/cross-cutting/sync*/src/`) are all consistent with the
foundation spec (where one exists) and the source-of-truth
priority. Auto-fix any disparities by dispatching subagents
per the 5-layer guarantees.

---

## Source-of-Truth Priority

When the 5 documents above disagree, resolve them in this
order (highest priority first):

1. `docs/specs/<domain>/*.md` — canonical for aggregates,
   commands, events, capabilities, audit targets. **N/A if
   Phase 0 has no domain spec** (foundation tier; the
   foundation references at `docs/specs/platform/`,
   `docs/specs/sync/`, and `docs/ports/storage.md` +
   `docs/ports/sync.md` + `docs/specs/<foundation>/` take
   the role of "spec" for Phase 0).
2. `docs/build-plan.md` § "Phase 0" — canonical for what the
   phase builds (deliverables, tasks, exit criteria, risks).
3. `docs/handoff/PHASE-0-HANDOFF.md` — the closing agent's
   claim about what was actually shipped (validated against
   the on-disk implementation in priority 4).
4. The implementation in
   `crates/infra/{core,query-derive,storage}/src/`,
   `crates/adapters/storage-surrealdb/src/`,
   `crates/cross-cutting/{sync,sync-inprocess}/src/` — the
   on-disk truth. Source files, tests, `Cargo.toml`
   dependencies, and the umbrella re-exports in
   `crates/educore/src/lib.rs` are the source of truth for
   "what was actually built".
5. `docs/phase_prompt/phase-0-prompt.md` — the input being
   verified. **LOWEST priority**: a prompt that diverges
   from priorities 1-4 must be corrected to match, not the
   other way around.

---

## Section A: Pre-Implementation Check

> Run BEFORE the phase is implemented. Applies to Phases
> 13-17 (unimplemented at the time this directory was
> created) and any future phase that is not yet closed.
> Skip this section entirely for Phases 0-11 (already
> closed and verified).

**Section A is N/A for Phase 0.** Phase 0 closed on PR 0 + PR A
(see `docs/handoff/PHASE-0-HANDOFF.md`); all 6 Phase 0 exit
criteria are green. The verify step runs only **Section B**
(below) for this phase. The pre-implementation checks
(Section A items 1-4) are not relevant for a closed phase.

For each item, output `Pass` (with a one-line citation) or
`Fail` (with a one-line citation + the fix the auto-fix
subagent will apply).

1. **Spec exists.** `docs/specs/<domain>/` (if applicable)
   contains the 11 standard files (`overview.md`,
   `aggregates.md`, `commands.md`, `entities.md`, `events.md`,
   `permissions.md`, `repositories.md`, `services.md`,
   `tables.md`, `value-objects.md`, `workflows.md`) per
   `AGENTS.md` § "Module Layout (per domain)". For non-domain
   phases, the port contract or reference doc exists in
   `docs/ports/` or `docs/guides/`.
2. **Build-plan § "Phase 0" is complete.** The build-plan
   section for Phase 0 (between the `## Phase 0` and
   `## Phase 1` headings) contains all 5 sub-sections:
   `Deliverables.`, `Tasks.`, `Exit criteria.`, `Risks.`,
   and `Phase completion documentation.` (per the per-phase
   prompt convention in `docs/phase_prompt/README.md`).
3. **Coverage rows are `Pending`.** Every aggregate or
   feature that the phase plans to ship has a row in
   `docs/coverage.toml` with `status = "Pending"`. The
   `PRE-CHECK-PHASES-13-17.md` snapshot enumerates the
   current `Pending` count; the per-phase preamble lists
   the expected count (spec-faithful vs headline-0).
4. **Scaffold crate is in place.** The `Cargo.toml` +
   `src/lib.rs` for the planned crate(s) exist at
   `crates/<tier>/<name>/` and follow the standard 27-line
   scaffold pattern (`PACKAGE_NAME` + `PACKAGE_VERSION` +
   the 9-file module prelude if it is a domain crate). For
   adapter or tools crates, the `Cargo.toml` declares the
   required `infra` + `cross-cutting` deps per
   `AGENTS.md` § "Tier System".

---

## Section B: Post-Implementation Check

> Run AFTER the phase is implemented. Applies to Phases
> 0-11 (closed) and Phase 12 (in progress — run when Phase
> 12 closes). Skip this section for Phases 13-17 (not yet
> implemented; covered by Section A).

For each dimension, output `Pass` (with a one-line citation
to the file + line range) or `Fail` (with a one-line
citation + the source-of-truth priority chain that resolves
it + the fix the auto-fix subagent will apply).

1. **Prompt ↔ Spec.** Every aggregate name in
   `docs/phase_prompt/phase-0-prompt.md` matches
   `docs/specs/<domain>/aggregates.md` exactly (per the
   closing-agent verification checklist in
   `docs/phase_prompt/README.md`). For non-domain phases,
   every port trait / reference impl in the prompt matches
   the corresponding `docs/ports/<port>.md` file.
2. **Prompt ↔ Build-Plan.** The "Deliverables" + "Tasks" +
   "Exit Criteria" sections in the prompt match the
   build-plan § "Phase 0" section. The headline-0
   interpretation (or spec-faithful declaration) is
   consistent between the prompt and the build-plan.
3. **Prompt ↔ Handoff.** The "Where NOT to start" rules in
   the prompt match the carry-forward rules in the handoff
   (no `educore-finance` dep, no `educore-notify` dep, no
   `educore-attendance` dep, no `educore-documents` dep, etc.,
   as applicable). The "Do NOT" list in the prompt's
   "Per-Deliverable Gotchas" matches the handoff's "Where
   NOT to start" section word-for-word (modulo phase-specific
   additions).
4. **Handoff ↔ Implementation.** The headline correctness
   check claimed in the handoff (e.g. the 100-case proptest
   in `crates/domains/<name>/src/services.rs`) exists and
   is green. The aggregates, command shapes, events, and
   `Capability` / `AuditTarget` variants listed in the
   handoff's "What's wired and working" section are present
   in the on-disk source files. The "Open questions" in the
   handoff are either resolved in the implementation
   or explicitly carried forward (with a citation in the
   per-phase preamble of the next phase's verify prompt).
5. **Coverage Matrix ↔ Implementation.** Every `Tested`
   row in `docs/coverage.toml` for Phase 0 has a real
   implementation in the source tree (not a stub returning
   `Err(not_supported)`). Every `Pending` row has either
   a real implementation that should be flipped to `Tested`
   (then flip it as part of the auto-fix) or an explicit
   "deferred to Phase 0+M" rationale in the handoff.

---

## Auto-Fix Rules (per dimension)

The verify agent dispatches one subagent per failing
dimension, with file-level ownership and section-level
pre-allocation per the 5-layer guarantees. The
subagent-scope mapping is:

| Failing dimension | Subagent scope | Files owned |
| --- | --- | --- |
| 1. Prompt ↔ Spec | `fix-prompt-spec` | `docs/phase_prompt/phase-0-prompt.md` |
| 2. Prompt ↔ Build-Plan | `fix-prompt-buildplan` | `docs/phase_prompt/phase-0-prompt.md`, `docs/build-plan.md` § "Phase 0" |
| 3. Prompt ↔ Handoff | `fix-prompt-handoff` | `docs/phase_prompt/phase-0-prompt.md`, `docs/handoff/PHASE-0-HANDOFF.md` |
| 4. Handoff ↔ Implementation | `fix-handoff-impl` | `crates/infra/{core,query-derive,storage}/src/**`, `crates/adapters/storage-surrealdb/src/**`, `crates/cross-cutting/{sync,sync-inprocess}/src/**` |
| 5. Coverage Matrix ↔ Implementation | `fix-coverage` | `docs/coverage.toml`, `crates/<tier>/<name>/src/**` (only for stub-flips) |

Multiple dimensions can run in parallel if they own disjoint
files. If two dimensions want to edit the same file (e.g.
dimension 1 + dimension 3 both touch
`docs/phase_prompt/phase-0-prompt.md`), the prep subagent
pre-allocates the file with section markers per the
5-layer guarantees; each subagent's edits stay inside its
assigned section.

The auto-fix subagent produces exactly one atomic commit
per the "Atomic commits per microtask" guarantee. The commit
message is `Phase 0 verify: <dimension> (<workstream>)` with
the standard
`Co-Authored-By: Antigravity <antigravity@google.com>` trailer.

---

## Subagent Orchestration (5-Layer Guarantees)

To prevent two or more subagents from being given the same
work, every verify prompt must enforce the following 5-layer
guarantees. These are the same rules that closed Phases
8-11 successfully (the first phase to break these rules
will produce a duplicate-work collision and a non-mergeable
state):

1. **File-level ownership.** Every file in the owned crate
   is assigned to exactly one subagent. No two subagents
   open the same file. The orchestrator maintains a
   file-ownership map in the phase plan and embeds the list
   of forbidden files in every parallel-subagent prompt.
2. **Section-level pre-allocation.** For files that must be
   touched by multiple workstreams (e.g. `aggregate.rs` for
   3+ root aggregates, or `phase-0-prompt.md` for two
   failing dimensions), the prep subagent pre-creates the
   file with named section markers
   (`// === <Aggregate> section begin (owner: <WorkstreamLetter>) ===`
   / `// === <Aggregate> section end ===` for code; or
   `<!-- === <Section> section begin (owner: <WorkstreamLetter>) === -->`
   / `<!-- === <Section> section end === -->` for markdown).
   Each workstream subagent's `Edit` anchors fall strictly
   inside its assigned range. A subagent that crosses a
   marker aborts and reports to the orchestrator.
3. **Sequential phase gates.** The verify step advances
   through fixed stages: `P0 prep` (single subagent,
   scaffolds shared files + cross-crate extensions) →
   `R1 reconcile-prep` (read-only verifier) → `wave 1/2/3`
   parallel fix-subagents → `R2 reconcile-impl` →
   `4-tests` (`cargo test --workspace`) → `5-docs`
   (`docs/handoff/PHASE-0-VERIFY-REPORT.md`) → `R3
   final-validation` (9-command gate). A verify step does
   not advance to the next stage until the prior stage's
   gate passes.
4. **Atomic commits per microtask.** Every subagent produces
   exactly one commit with a
   `Phase 0 verify: <scope> (<workstream>)` message +
   `Co-Authored-By: Antigravity <antigravity@google.com>`
   trailer. The orchestrator inspects `git log --stat` after
   every stage to detect any out-of-scope file. A "do not
   run cargo test" rule applies to the parallel fix wave —
   the orchestrator runs the gate at stage 4, not the
   subagents.
5. **Reconciler subagents are read-only.** `R1`, `R2`, `R3`
   are dedicated reconciler subagents. They verify section
   boundaries + duplicate detection + stub-replacement but
   never write code. A reconciler that finds a violation halts
   the verify step.

---

## Output Format

Write `docs/handoff/PHASE-0-VERIFY-REPORT.md` with these
five sections:

- **Section A — Pre-Implementation Check results.**
  `Pass` / `Fail` per item, with a one-line citation to the
  file + line range. **N/A for Phase 0 (phase is closed).**
- **Section B — Post-Implementation Check results.**
  `Pass` / `Fail` per dimension, with a one-line citation.
- **Section C — Disparities Summary.** Bullet list of every
  item that `Failed` in Section A or Section B, with the
  specific file + line + the source-of-truth priority chain
  that resolves it (e.g. "Spec line 42 says `NoticeBoard`,
  build-plan line 1435 says `Notice` — Spec wins (priority 1),
  fix the build-plan").
- **Section D — Fix Plan.** Ordered list of files to update
  (or "no fixes needed" if both sections pass). Each fix
  item names the file, the change, and the subagent scope
  per the 5-layer guarantees.
- **Section E — GO/NO-GO verdict.** `GO` if all checks pass
  or all disparities are fixed in the same atomic commit;
  `NO-GO` if any fix is deferred or any check is open.

The verify report itself is one atomic commit; the fixes
it triggers are a second atomic commit per the
"Atomic commits per microtask" guarantee.

---

## Done Criteria

The verify step is `Done` when ALL of the following hold:

- [ ] `docs/handoff/PHASE-0-VERIFY-REPORT.md` exists with all
  5 sections (A, B, C, D, E) populated.
- [ ] All Section A items pass (for unimplemented phases
  13-17) or all Section B dimensions pass (for implemented
  phases 0-11 and 12). **For Phase 0: Section A is N/A;
  Section B dimensions 1-5 must all pass.**
- [ ] All listed disparities fixed (or explicitly deferred
  with a rationale + an ADR reference in Section C).
- [ ] One atomic commit with the fixes (per the
  "Atomic commits per microtask" guarantee).
- [ ] `cargo test -p educore-core` green (for implemented phases;
  the crate name comes from the per-phase preamble).
- [ ] `cargo build --workspace` green.
- [ ] `cargo run -p educore-core --bin lint --features lint`
  green (the no-gaps gate per `AGENTS.md`).
- [ ] `docs/progress-tracker.md` row for Phase 0 updated
  (status reflects the verified close).

---

# Per-Phase Preamble — Phase 0 (Foundation)

**Phase title:** Foundation (core + macro + storage port + SurrealDB adapter + sync engine + outbox e2e)

**Status:** Implemented (per `docs/handoff/PHASE-0-HANDOFF.md`)

**Build-plan section:** `docs/build-plan.md` lines 136–311 (between the `## Phase 0` and `## Phase 1` headings; closing `---` at line 311).

**Spec:** No spec — Phase 0 is the foundation tier, not a domain phase. Foundation references:
- `docs/specs/platform/` (the `SchoolId` + `TenantContext` reference)
- `docs/specs/sync/` (the sync event/command catalog)
- `docs/ports/storage.md` + `docs/ports/sync.md` (the port contracts)
- `docs/schemas/database-schema.md` + `docs/schemas/tenancy-schema.md` (the engine invariants)

**Handoff:** `docs/handoff/PHASE-0-HANDOFF.md`

**Implementation crates (per `AGENTS.md` § "Tier System"):**
- `crates/infra/core/` (`educore-core`): identifiers, errors, value objects, `Clock`, `IdGenerator`, query AST, `lint` sub-module
- `crates/infra/query-derive/` (`educore-query-derive`): the `#[derive(DomainQuery)]` proc macro
- `crates/infra/storage/` (`educore-storage`): the `StorageAdapter` port trait + the 4 sub-ports (`Outbox`, `AuditLog`, `EventLog`, `Idempotency`) + the `Transaction` sub-port + the `change_stream` module housing the 4 sync methods (`watch_changes`, `apply_snapshot`, `cursor_for`, `advance_cursor`)
- `crates/adapters/storage-surrealdb/` (`educore-storage-surrealdb`): SurrealDB primary adapter. **Deferred sub-ports:** `AuditLog`, `EventLog`, `Idempotency` return `NotSupported`; only `Outbox` is real in Phase 0. (Real impls land in Phase 1 SQL adapters for `Idempotency`; Phase 2 `educore-audit` + `educore-events` for `AuditLog` + `EventLog`.)
- `crates/cross-cutting/sync/` (`educore-sync`): the sync port + commands + events + `SyncCoordinator` struct
- `crates/cross-cutting/sync-inprocess/` (`educore-sync-inprocess`): the in-process reference impl
- `crates/tools/storage-parity/` (`educore-storage-parity`): scaffold only (full suite lands in Phase 16)

**Coverage rows in `docs/coverage.toml` for `phase = 0` (15 total: 14 `Tested`, 1 `Pending`):**
- `outbox_ddl_surreal` — `status = "Tested"` (spec: `migrations/engine/0000_engine_core.surreal.surql`; crate: `educore-storage-surrealdb`)
- `domain_query_macro` — `status = "Tested"` (spec: `docs/query_layer.md`; crate: `educore-query-derive`)
- `entity_descriptor_ast` — `status = "Tested"` (spec: `docs/query_layer.md`; crate: `educore-core`)
- `school_id_newtype` — `status = "Tested"` (spec: `docs/schemas/tenancy-schema.md`; crate: `educore-core`)
- `uuid_v7_generator` — `status = "Tested"` (spec: `docs/schemas/database-schema.md`; crate: `educore-core`)
- `system_clock` — `status = "Tested"` (spec: `docs/schemas/database-schema.md`; crate: `educore-core`)
- `domain_error_enum` — `status = "Tested"` (spec: `docs/code-standards.md`; crate: `educore-core`)
- `lint_submodule` — `status = "Tested"` (spec: `docs/build-plan.md`; crate: `educore-core`)
- `storage_adapter_port` — `status = "Tested"` (spec: `docs/ports/storage.md`; crate: `educore-storage`)
- `storage_transaction_port` — `status = "Tested"` (spec: `docs/ports/storage.md`; crate: `educore-storage`)
- `storage_outbox_port` — `status = "Tested"` (spec: `docs/ports/storage.md`; crate: `educore-storage`)
- `storage_parity_suite` — `status = "Pending"` (spec: `docs/ports/storage.md`; crate: `educore-storage-parity`; **orphaned** between Phase 0 scaffold and Phase 16 full suite per build-plan § "Orphaned items")
- `sync_port` — `status = "Tested"` (spec: `docs/ports/sync.md`; crate: `educore-sync`)
- `sync_inprocess_impl` — `status = "Tested"` (spec: `docs/ports/sync.md`; crate: `educore-sync-inprocess`)
- `engine_graph_regen` — `status = "Tested"` (spec: `docs/decisions/ADR-016-EngineGraph.md`; crate: `workspace`)

**Known carry-forward rules relevant to this phase (per `docs/handoff/PHASE-0-HANDOFF.md` "Open questions" + "Where NOT to start"):**
- **`EntityDescriptor` AST shape** — concrete `EntityDescriptor` struct lands with the first domain crate (Phase 3). Phase 0 ships the `QueryNode<F>` AST + `Field` / `HasRelations` traits only. Do not refactor around it in subsequent phases.
- **Ad-hoc sync envelope types** — Phase 0 sync uses its own event struct, not `educore_events::EventEnvelope`. Phase 2 should refactor `educore-sync` to depend on the envelope crate.
- **SurrealDB stubbed sub-ports** — `AuditLog`, `EventLog`, `Idempotency` on `educore-storage-surrealdb` return `NotSupported`. Real impls: `Idempotency` in Phase 1 SQL adapters; `AuditLog` + `EventLog` in Phase 2 (`educore-audit` + `educore-events`). Do not add real impls earlier.
- **Deferred sync implementations** — `educore-sync-http` (worker client) deferred to Phase 2; `educore-sync-null` (no-op impl) deferred to Phase 16. Do not add these in Phase 1.
- **Don't touch `educore-core::lint` sub-module** — it is done; the PR 0 fix-up closed exit criterion 5 (clippy clean). Any new lint additions must go through an ADR.
- **No new `unwrap`/`expect` in domain code** — `AGENTS.md` forbids it; the lint will flag it.
- **Don't rename or move crates without an ADR** — the current layout is canonical per `docs/decisions/ADR-013-CrateLayout.md`.

**Pre-implementation gaps found in `PRE-CHECK-PHASES-13-17.md` (if any for this phase):** N/A (Phase 0 is already implemented; PRE-CHECK covers only Phases 13-17).

**Specific verification focus (Phase 0):**
- The 6 source-file modules in `crates/infra/core/src/`: `ids.rs` (`SchoolId`/`UserId`/`EventId`/`CorrelationId` + UUIDv7 generator), `error.rs` (`DomainError` via `thiserror`), `value_objects.rs` (`Timestamp`/`Version`/`Etag`/`ActiveStatus`), `clock.rs` (`Clock` trait + `SystemClock` + `TestClock` + the v7 UUID `IdGenerator` with deterministic test backend), `query.rs` (the `QueryNode<F>` AST + `Field`/`HasRelations` traits), `lint.rs` (the `lint` sub-module gated behind the `lint` Cargo feature)
- The 6 storage port sub-ports / traits (`StorageAdapter`, `Outbox`, `AuditLog`, `EventLog`, `Idempotency`, `Transaction`); **NOT 10** — the build plan defines 4 sub-ports (`Outbox`, `AuditLog`, `Idempotency`, `EventLog`) plus the `Transaction` sub-port plus the `StorageAdapter` trait itself = 6 trait declarations
- The 4 sync methods on the `StorageAdapter` port (in `crates/infra/storage/src/port.rs`): `watch_changes`, `apply_snapshot`, `cursor_for`, `advance_cursor`
- The SurrealDB adapter's deferred status: 3 stub sub-ports (`AuditLog`, `EventLog`, `Idempotency` return `NotSupported`); only `Outbox` is real. The deferred status is documented in `docs/handoff/PHASE-0-HANDOFF.md` § "What's stubbed" and is **not a bug** — it is the Phase 0 baseline per the build plan.
- The `educore-events` cross-cutting crate: **NOT** in Phase 0's scope. The `educore-events` (envelope) crate is a Phase 2 deliverable per the build-plan and `AGENTS.md` § "Tier System". The build-plan does call out that `educore-events-domain` (calendar) is Phase 13; do not conflate the two. The `educore-sync` crate is the only `cross-cutting/events*` crate in Phase 0.
- The SurrealDB driver pin: `surrealdb = "2"` with `kv-mem` + `rustls`, pinned to the last pre-1.75 line per `docs/decisions/ADR-015-ExternalCrates.md`. The `reqwest 0.12.x` pin remains in `[workspace.dependencies]` for the deferred `educore-sync-http` worker (Phase 0+); it is not exercised in Phase 0.
- The `mysql_async` crate is **NOT** in Phase 0 (rejected; MySQL parity is Phase 1 with `sqlx`). The build-plan § "Pre-implementation state" lists `sqlx 0.8.x` and `mysql_async 0.34.x` as Phase 1 pins.
- The `#[derive(DomainQuery)]` macro must emit a `__spec_coverage__` test module on every invocation (per build-plan § Phase 0 task 2 and § "The No-Gaps Gates"). Verify that `crates/infra/query-derive/tests/derive_test.rs` exercises this.
- The SurrealDB outbox e2e test at `crates/adapters/storage-surrealdb/tests/outbox_e2e.rs` must assert the engine invariants: every aggregate has `school_id`, UUIDv7 columns, byte-for-byte DDL match against `migrations/engine/0000_engine_core.surreal.surql`. Verify the e2e runs on the in-memory `surrealdb::Mem` fast path and the testcontainers full path is env-gated.
- The sync e2e test must verify the `SyncCoordinator` (in-process) receives the outbox event alongside the storage read-back (per build-plan task 10 and `docs/ports/sync.md`).
