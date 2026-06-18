# Phase 1 Verification Prompt

> Per-phase verify prompt for Phase 1 (Adapter parity),
> rendered from `docs/verification/TEMPLATE.md`. Section A is
> **N/A** for Phase 1 (the phase is already closed and
> verified per the template's own instruction; skip it).
> Section B is the primary scope. The closing-of-close
> auto-fixes any disparities the verification subagent finds.

---

## Mission

Verify that Phase 1's forward-looking prompt
(`docs/phase_prompt/phase-1-prompt.md`), retrospective handoff
(`docs/handoff/PHASE-1-HANDOFF.md`), build-plan section
(`docs/build-plan.md` § "Phase 1"), and on-disk implementation
(`crates/adapters/storage-postgres/src/`,
`crates/adapters/storage-mysql/src/`,
`crates/adapters/storage-sqlite/src/`) are all consistent with
the storage port contract and the source-of-truth priority.
Auto-fix any disparities by dispatching subagents per the
5-layer guarantees.

---

## Source-of-Truth Priority

When the 5 documents above disagree, resolve them in this
order (highest priority first):

1. `docs/specs/<domain>/*.md` — canonical for aggregates,
   commands, events, capabilities, audit targets. **N/A if
   Phase 1 has no domain spec** (adapters tier; the storage
   port contract at `docs/ports/storage.md` + the 3
   dialect-specific DDL files at
   `migrations/engine/0000_engine_core.{postgres,mysql,sqlite}.sql`
   + the dialect-comparison notes at
   `docs/schemas/sql-dialects/comparison.md` take the role of
   "spec" for Phase 1).
2. `docs/build-plan.md` § "Phase 1" — canonical for what the
   phase builds (deliverables, tasks, exit criteria, risks).
3. `docs/handoff/PHASE-1-HANDOFF.md` — the closing agent's
   claim about what was actually shipped (validated against
   the on-disk implementation in priority 4).
4. The implementation in
   `crates/adapters/storage-{postgres,mysql,sqlite}/src/`,
   plus the shared reference DDL in
   `migrations/engine/0000_engine_core.*.sql` — the on-disk
   truth. Source files, tests, `Cargo.toml` dependencies,
   and the umbrella re-exports in
   `crates/educore/src/lib.rs` are the source of truth for
   "what was actually built".
5. `docs/phase_prompt/phase-1-prompt.md` — the input being
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

**Section A is N/A for Phase 1.** Phase 1 closed per
`docs/handoff/PHASE-1-HANDOFF.md`; all 7 Phase 1 exit
criteria are green (per the handoff: the 3 per-adapter
`cargo test -p educore-storage-{postgres,mysql,sqlite}`
runs, `cargo test --workspace`, `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo fmt --all -- --check`,
the `educore-core::lint` binary, and 15
`docs/coverage.toml` rows flipped from `Pending` to
`Tested` with `tests` paths). The verify step runs only
**Section B** (below) for this phase. The
pre-implementation checks (Section A items 1-4) are not
relevant for a closed phase.

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
2. **Build-plan § "Phase 1" is complete.** The build-plan
   section for Phase 1 (between the `## Phase 1` and
   `## Phase 2` headings) contains all 5 sub-sections:
   `Deliverables.`, `Tasks.`, `Exit criteria.`, `Risks.`,
   and `Phase completion documentation.` (per the per-phase
   prompt convention in `docs/phase_prompt/README.md`).
3. **Coverage rows are `Pending`.** Every aggregate or
   feature that the phase plans to ship has a row in
   `docs/coverage.toml` with `status = "Pending"`. The
   `PRE-CHECK-PHASES-13-17.md` snapshot enumerates the
   current `Pending` count; the per-phase preamble lists
   the expected count (spec-faithful vs headline-1).
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

1. **Prompt ↔ Spec.** Every port trait / reference impl in
   `docs/phase_prompt/phase-1-prompt.md` matches the
   corresponding `docs/ports/<port>.md` file. The 3 SQL
   adapter deliverables (`educore-storage-postgres`,
   `educore-storage-mysql`, `educore-storage-sqlite`) are
   named in the prompt's "Deliverables" exactly as they
   appear in `docs/ports/storage.md` § "Cross-Adapter
   Parity". The 4 sub-ports claimed in the prompt
   (`Outbox`, `AuditLog`, `EventLog`, `Idempotency`) match
   the 4 sub-port traits in
   `crates/infra/storage/src/port.rs`. For Phase 1 the
   "spec" is the storage port contract + the 3 dialect
   DDL files + `docs/schemas/sql-dialects/comparison.md`.
2. **Prompt ↔ Build-Plan.** The "Deliverables" + "Tasks" +
   "Exit Criteria" sections in the prompt match the
   build-plan § "Phase 1" section. The build-plan lists
   3 SQL adapter crates; the prompt lists the same 3. The
   build-plan's "Risks." sub-section (MySQL `CHECK` floor
   8.0.16+, SQLite single-writer) is reflected in the
   prompt's "Per-Dialect Gotchas" section. The
   "Per-call transaction model" choice (flag-based
   wrappers per the handoff) is consistent between the
   prompt and the build-plan § "Phase 1 outcome.".
3. **Prompt ↔ Handoff.** The "Where NOT to start" rules in
   the prompt match the carry-forward rules in the handoff
   (no `mysql_async` re-add, no `educore-finance` dep, no
   refactor of the flag-based transaction model without
   threading a real `sqlx::Transaction`). The "Do NOT" list
   in the prompt's "Per-Dialect Gotchas" matches the
   handoff's "Where NOT to start" section word-for-word
   (modulo phase-specific additions).
4. **Handoff ↔ Implementation.** The headline correctness
   check claimed in the handoff (e.g. "124 tests pass
   workspace-wide" + the 3 per-adapter outbox e2e tests)
   exists and is green. The 3 adapter crates
   (`educore-storage-postgres`, `educore-storage-mysql`,
   `educore-storage-sqlite`) exist with real impls of all
   4 sub-ports — no `NotSupported` stubs on the SQL
   adapters (this is a deliberate departure from the
   Phase 0 SurrealDB pattern, where only `Outbox` was
   real). The "Open questions" in the handoff (5 items:
   flag-based transactions, `AuditLogEntry` field subset,
   `IdempotencyRecord::command_type` `Box::leak`, stale
   workspace `mysql_async`/`flate2` pins, missing
   `educore-events` envelope crate) are either resolved
   in the implementation or explicitly carried forward
   (with a citation in the per-phase preamble of the
   Phase 2 verify prompt).
5. **Coverage Matrix ↔ Implementation.** Every `Tested`
   row in `docs/coverage.toml` for Phase 1 (15 rows: 4
   engine-table DDL rows × 3 adapters + 3 storage-impl
   rows) has a real implementation in the source tree
   (not a stub returning `Err(not_supported)`). The 4
   engine tables (`outbox`, `idempotency`,
   `schema_registry`, `system_user`) have DDL emitted via
   `include_str!` from
   `migrations/engine/0000_engine_core.{postgres,mysql,sqlite}.sql`
   and real impls in
   `crates/adapters/storage-{postgres,mysql,sqlite}/src/`.
   The `audit_log_ddl_*` and `event_log_ddl_*` rows are
   **not** Phase 1 — those are owned by `educore-audit`
   and `educore-events` (Phase 2). The
   `storage_parity_suite` row stays `Pending` (orphaned
   between Phase 0 and Phase 16 per build-plan §
   "Orphaned items"); Phase 1 does not flip it.

---

## Auto-Fix Rules (per dimension)

The verify agent dispatches one subagent per failing
dimension, with file-level ownership and section-level
pre-allocation per the 5-layer guarantees. The
subagent-scope mapping is:

| Failing dimension | Subagent scope | Files owned |
| --- | --- | --- |
| 1. Prompt ↔ Spec | `fix-prompt-spec` | `docs/phase_prompt/phase-1-prompt.md` |
| 2. Prompt ↔ Build-Plan | `fix-prompt-buildplan` | `docs/phase_prompt/phase-1-prompt.md`, `docs/build-plan.md` § "Phase 1" |
| 3. Prompt ↔ Handoff | `fix-prompt-handoff` | `docs/phase_prompt/phase-1-prompt.md`, `docs/handoff/PHASE-1-HANDOFF.md` |
| 4. Handoff ↔ Implementation | `fix-handoff-impl` | `crates/adapters/storage-{postgres,mysql,sqlite}/src/**` |
| 5. Coverage Matrix ↔ Implementation | `fix-coverage` | `docs/coverage.toml`, `crates/adapters/storage-{postgres,mysql,sqlite}/src/**` (only for stub-flips) |

Multiple dimensions can run in parallel if they own disjoint
files. If two dimensions want to edit the same file (e.g.
dimension 1 + dimension 3 both touch
`docs/phase_prompt/phase-1-prompt.md`), the prep subagent
pre-allocates the file with section markers per the
5-layer guarantees; each subagent's edits stay inside its
assigned section.

The auto-fix subagent produces exactly one atomic commit
per the "Atomic commits per microtask" guarantee. The commit
message is `Phase 1 verify: <dimension> (<workstream>)` with
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
   touched by multiple workstreams (e.g.
   `crates/adapters/storage-postgres/src/transaction.rs` for
   the flag-based transaction model that 2+ dimensions may
   need to inspect, or `phase-1-prompt.md` for two failing
   dimensions), the prep subagent pre-creates the file with
   named section markers
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
   (`docs/handoff/PHASE-1-VERIFY-REPORT.md`) → `R3
   final-validation` (9-command gate). A verify step does
   not advance to the next stage until the prior stage's
   gate passes.
4. **Atomic commits per microtask.** Every subagent produces
   exactly one commit with a
   `Phase 1 verify: <scope> (<workstream>)` message +
   `Co-Authored-By: Antigravity <antigravity@google.com>`
   trailer. The orchestrator inspects `git log --stat` after
   every stage to detect any out-of-scope file. A "do not
   run cargo test" rule applies to the parallel fix wave —
   the orchestrator runs the gate at stage 4, not the
   subagents.
5. **Reconciler subagents are read-only.** `R1`, `R2`, `R3`
   are dedicated reconciler subagents. They verify section
   boundaries + duplicate detection + stub-replacement but
   never write code. A reconciler that finds a violation
   halts the verify step.

---

## Output Format

Write `docs/handoff/PHASE-1-VERIFY-REPORT.md` with these
five sections:

- **Section A — Pre-Implementation Check results.**
  `Pass` / `Fail` per item, with a one-line citation to the
  file + line range. **N/A for Phase 1 (phase is closed).**
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

- [ ] `docs/handoff/PHASE-1-VERIFY-REPORT.md` exists with all
  5 sections (A, B, C, D, E) populated.
- [ ] All Section A items pass (for unimplemented phases
  13-17) or all Section B dimensions pass (for implemented
  phases 0-11 and 12). **For Phase 1: Section A is N/A;
  Section B dimensions 1-5 must all pass.**
- [ ] All listed disparities fixed (or explicitly deferred
  with a rationale + an ADR reference in Section C).
- [ ] One atomic commit with the fixes (per the
  "Atomic commits per microtask" guarantee).
- [ ] `cargo test -p educore-storage-postgres`,
  `cargo test -p educore-storage-mysql`,
  `cargo test -p educore-storage-sqlite` all green (the
  3 adapter crate names come from the per-phase preamble).
- [ ] `cargo test --workspace` green.
- [ ] `cargo build --workspace` green.
- [ ] `cargo run -p educore-core --bin lint --features lint`
  green (the no-gaps gate per `AGENTS.md`).
- [ ] `docs/progress-tracker.md` row for Phase 1 updated
  (status reflects the verified close).

---

# Per-Phase Preamble — Phase 1 (Adapter parity)

**Phase title:** Adapter parity (PostgreSQL + MySQL + SQLite)

**Status:** Implemented (per `docs/handoff/PHASE-1-HANDOFF.md`)

**Build-plan section:** `docs/build-plan.md` lines 313–417 (between the `## Phase 1` and `## Phase 2` headings; closing `---` at line 417).

**Spec:** No spec — Phase 1 is a storage-adapter tier, not a domain phase. Reference contracts:
- `docs/ports/storage.md` (the `StorageAdapter` port + 4 sub-ports + `Transaction` + `Tenant Isolation` + `Migrations` + `Cross-Adapter Parity` sections)
- `docs/schemas/sql-dialects/postgresql.md` + `mysql.md` + `sqlite.md` + `comparison.md` (the per-dialect conventions + the dialect-difference matrix)
- `migrations/engine/0000_engine_core.{postgres,mysql,sqlite}.sql` (the canonical DDL, `include_str!`d by the adapter crates at compile time)

**Handoff:** `docs/handoff/PHASE-1-HANDOFF.md`

**Implementation crates (per `AGENTS.md` § "Tier System"):**
- `crates/adapters/storage-postgres/` (`educore-storage-postgres`): PostgreSQL 14+ adapter. `PostgresConnection::connect(url, school)` opens a `sqlx::PgPool` and registers an `after_connect` hook that issues `SET search_path = engine, public`. RLS via `CREATE POLICY` + `ENABLE ROW LEVEL SECURITY`.
- `crates/adapters/storage-mysql/` (`educore-storage-mysql`): MySQL 8.0+ adapter. `MySQL 8.0.16+` floor for `CHECK`; no native RLS (emulated via `SET @app_tenant_id` + `WHERE school_id = @app_tenant_id` on every query); `utf8mb4_unicode_ci` + `InnoDB`; backtick identifier quoting. URL must include `?multi_statements=true` (or `&multi_statements=true`).
- `crates/adapters/storage-sqlite/` (`educore-storage-sqlite`): SQLite 3.x adapter (embedded / offline mode). `TEXT` with `CHECK(length() = 36)` for UUIDs, `INTEGER` for booleans, ISO 8601 `TEXT` for timestamps, no RLS, no schema namespaces. JSON via the `json1` extension at the application layer. Single-writer deployment model.

**Coverage rows in `docs/coverage.toml` for `phase = 1` (15 total: all `Tested`):**
- `outbox_ddl_pg` — `status = "Tested"` (spec: `migrations/engine/0000_engine_core.postgres.sql`; crate: `educore-storage-postgres`; tests: `crates/adapters/storage-postgres/tests/outbox_e2e.rs`)
- `outbox_ddl_mysql` — `status = "Tested"` (spec: `migrations/engine/0000_engine_core.mysql.sql`; crate: `educore-storage-mysql`; tests: `crates/adapters/storage-mysql/tests/outbox_e2e.rs`)
- `outbox_ddl_sqlite` — `status = "Tested"` (spec: `migrations/engine/0000_engine_core.sqlite.sql`; crate: `educore-storage-sqlite`; tests: `crates/adapters/storage-sqlite/tests/outbox_e2e.rs`)
- `idempotency_ddl_pg` — `status = "Tested"` (spec: `migrations/engine/0000_engine_core.postgres.sql`; crate: `educore-storage-postgres`)
- `idempotency_ddl_mysql` — `status = "Tested"` (spec: `migrations/engine/0000_engine_core.mysql.sql`; crate: `educore-storage-mysql`)
- `idempotency_ddl_sqlite` — `status = "Tested"` (spec: `migrations/engine/0000_engine_core.sqlite.sql`; crate: `educore-storage-sqlite`)
- `schema_registry_ddl_pg` — `status = "Tested"` (spec: `migrations/engine/0000_engine_core.postgres.sql`; crate: `educore-storage-postgres`)
- `schema_registry_ddl_mysql` — `status = "Tested"` (spec: `migrations/engine/0000_engine_core.mysql.sql`; crate: `educore-storage-mysql`)
- `schema_registry_ddl_sqlite` — `status = "Tested"` (spec: `migrations/engine/0000_engine_core.sqlite.sql`; crate: `educore-storage-sqlite`)
- `system_user_ddl_pg` — `status = "Tested"` (spec: `migrations/engine/0000_engine_core.postgres.sql`; crate: `educore-storage-postgres`)
- `system_user_ddl_mysql` — `status = "Tested"` (spec: `migrations/engine/0000_engine_core.mysql.sql`; crate: `educore-storage-mysql`)
- `system_user_ddl_sqlite` — `status = "Tested"` (spec: `migrations/engine/0000_engine_core.sqlite.sql`; crate: `educore-storage-sqlite`)
- `storage_postgres_impl` — `status = "Tested"` (spec: `docs/ports/storage.md`; crate: `educore-storage-postgres`; tests: `crates/adapters/storage-postgres/tests/outbox_e2e.rs`)
- `storage_mysql_impl` — `status = "Tested"` (spec: `docs/ports/storage.md`; crate: `educore-storage-mysql`; tests: `crates/adapters/storage-mysql/tests/outbox_e2e.rs`)
- `storage_sqlite_impl` — `status = "Tested"` (spec: `docs/ports/storage.md`; crate: `educore-storage-sqlite`; tests: `crates/adapters/storage-sqlite/tests/outbox_e2e.rs`)

**Known carry-forward rules relevant to this phase (per `docs/handoff/PHASE-1-HANDOFF.md` "Open questions" + "Where NOT to start"):**
- **Flag-based per-call transactions** — `PostgresTransaction` / `MysqlTransaction` / `SqliteTransaction` are flag-based wrappers; each sub-port call opens its own short `pool.begin()`. The engine's at-least-once dedup (`event_id` PK on outbox; `ON CONFLICT DO NOTHING` / `ON DUPLICATE KEY UPDATE` on idempotency) is the safety net. A future PR could thread a real `sqlx::Transaction` through the sub-port methods for true atomicity. **Do not refactor the flag-based transaction model** in Phase 1 (or any subsequent phase) without also adding real `sqlx::Transaction` support.
- **`AuditLogEntry` / `EventLogEntry` struct fields are a subset of the DDL columns** — the DDL has columns the structs don't carry (`ip`, `user_agent`, `session_id`, `command_id`, `cross_tenant` on `audit_log`; no `active_status` on `event_log`). Adapters fill in safe defaults on write and drop the columns on read. A follow-up PR should reconcile (likely expand the port structs to carry the missing fields). **Defer to a follow-up PR; not blocking.**
- **`IdempotencyRecord::command_type: &'static str` requires a `Box::leak` on the SQLite read path** — the storage port should change the field from `&'static str` to `String` (or `Cow<'static, str>`) in a follow-up. **Defer to a follow-up PR; not blocking.**
- **Workspace `Cargo.toml` still pins `mysql_async` and `flate2`** for historical reasons (the comment block explains the original `flate2/zlib-rs` dependency chain). They are no longer referenced by any workspace crate. **A cleanup PR can drop them; not blocking.**
- **`educore-events` envelope crate is the missing link** — the SQL adapters' `AuditLog::append` and `EventLog::append` take the storage-port structs (not the engine's domain events) because the `educore-events` crate doesn't exist yet. **Phase 2's first task** should land `educore-events` and have the audit/event sub-ports take `EventEnvelope<T: DomainEvent>` instead of the raw port structs.
- **SurrealDB adapter (`crates/adapters/storage-surrealdb/`) stubbed sub-ports** — `AuditLog`, `EventLog`, `Idempotency` return `NotSupported`; only `Outbox` is real. **A future PR** should add the same 4-port parity the SQL adapters now have. **Not blocking Phase 2.**
- **Deferred sync primitives** — all three SQL adapters' `watch_changes` / `apply_snapshot` / `cursor_for` / `advance_cursor` return `NotSupported` (the storage port's default impls). PG's `LISTEN/NOTIFY`, MySQL binlog tail, SQLite polling — all deferred. **Not blocking Phase 2.**
- **Cross-adapter parity test suite** (`educore-storage-parity`) is **Phase 16 work**, not Phase 1. Phase 1 has per-adapter e2e only. The `storage_parity_suite` row in `docs/coverage.toml` stays `Pending` (orphaned between Phase 0 scaffold and Phase 16 full suite per build-plan § "Orphaned items"). **Do not flip it in Phase 1.**
- **Don't touch `educore-core::lint` sub-module** — it is done. Any new lint additions must go through an ADR.
- **No new `unwrap`/`expect` in domain code** — `AGENTS.md` forbids it; the lint will flag it.
- **Don't rename or move crates without an ADR** — the current layout is canonical per `docs/decisions/ADR-013-CrateLayout.md`.
- **Don't re-add `mysql_async` to `educore-storage-mysql`** — the user has explicitly chosen `sqlx` for all three SQL adapters.

**Pre-implementation gaps found in `PRE-CHECK-PHASES-13-17.md` (if any for this phase):** N/A (Phase 1 is already implemented; PRE-CHECK covers only Phases 13-17).

**Specific verification focus (Phase 1):**
- The 3 SQL adapter crates (`educore-storage-postgres`, `educore-storage-mysql`, `educore-storage-sqlite`) all ship with **real impls of all 4 sub-ports** (`Outbox`, `AuditLog`, `EventLog`, `Idempotency`) — **no `NotSupported` stubs**. This is a deliberate departure from the Phase 0 SurrealDB pattern (where only `Outbox` was real).
- The 3 storage adapters' `src/` file layout per `AGENTS.md` § "Module Layout": each has `connection.rs`, `migrations.rs` (DDL `include_str!`), `outbox.rs`, `audit_log.rs`, `event_log.rs`, `idempotency.rs`, plus a shared `schema_registry.rs` + `system_user.rs`, plus a `transaction.rs` (flag-based `PostgresTransaction` / `MysqlTransaction` / `SqliteTransaction`), plus per-dialect `introspect.rs` and `translate.rs` modules where present.
- The `sqlx` version pin: **0.8.x** for all three SQL adapters (per workspace `Cargo.toml` + the build-plan's "Per-call transaction model" choice). The previous plan to use `mysql_async` for MySQL was rejected; `mysql_async` and the transitive `flate2` direct dep have been removed from `crates/adapters/storage-mysql/Cargo.toml`.
- The `rustls` feature flag for `sqlx` (no `native-tls`) per `AGENTS.md` § "Code Standards" + `docs/decisions/ADR-015-ExternalCrates.md`. Verify `default-features = false` + `rustls` is set on every `sqlx` entry in all 3 adapter `Cargo.toml` files.
- The 3 `[[dev-dependencies]]` blocks in the adapter `Cargo.toml` files reference the cross-dialect parity test suite (`educore-storage-parity`); verify the e2e tests at `crates/adapters/storage-{postgres,mysql,sqlite}/tests/outbox_e2e.rs` are present and env-gated per the handoff:
  - PG e2e gated on `EDUCORE_PG_URL` (skips with `tracing::info!` when unset)
  - MySQL e2e gated on `EDUCORE_MYSQL_URL` (same skip pattern as PG)
  - SQLite e2e **always runs in CI** (uses `SqliteConnection::in_memory(school)`)
- The 6 engine cross-cutting tables (`outbox`, `audit_log`, `event_log`, `idempotency`, `schema_registry`, `system_user`) are emitted at startup via `storage.create_schema().await` per `docs/schemas/sql-dialects/README.md` § "Runtime DDL emission". Phase 1 ships the 4 tables owned by the storage port (`outbox`, `idempotency`, `schema_registry`, `system_user`); the 2 tables owned by cross-cutting crates (`audit_log`, `event_log`) ship in Phase 2 (`educore-audit` + `educore-events`).
- The 3 dialect-specific DDL files at `migrations/engine/0000_engine_core.{postgres,mysql,sqlite}.sql` are the **canonical reference** for the DDL; the adapter crates `include_str!` them at compile time. Verify each adapter's `migrations.rs` (or equivalent) does `include_str!(...)` against the matching dialect file.
- The `mysql_async` and `flate2` workspace deps can be dropped in a cleanup PR (per the handoff "Open questions" #4); not blocking Phase 1 close. If the cleanup PR lands in a later phase, the verify step should also re-validate the workspace builds.
- The tenant-isolation contract per `docs/ports/storage.md` § "Tenant Isolation": the storage adapter is responsible for enforcing tenant isolation; the engine always passes a `SchoolId` filter; the adapter MUST add a `school_id = $1` predicate to every read query. Verify each adapter's read paths include the `school_id` filter.
- The 4 sync methods on the `StorageAdapter` port (`watch_changes`, `apply_snapshot`, `cursor_for`, `advance_cursor`) return `NotSupported` (the storage port's default impls) on all 3 SQL adapters. This is **not a bug** — it is the Phase 1 baseline per the build plan and handoff.
- The `EventLog::read` / `count` build the `WHERE` clause dynamically with `build_select` (a `String` SQL + a `Vec<FilterParam>` enum). Values are bound positionally; **no `format!` interpolation of user input** — verify this against the handoff's "What's wired and working" section.
- The `Idempotency::record` `ON CONFLICT` / `ON DUPLICATE KEY UPDATE` pattern matches the dialect: PG uses `ON CONFLICT (school_id, command_type, idempotency_key) DO NOTHING`; MySQL uses `ON DUPLICATE KEY UPDATE command_id = command_id` (a no-op self-assignment); SQLite uses `INSERT OR IGNORE` (or equivalent `ON CONFLICT`).
- The `mark_published` SQL differs by dialect: PG uses `ANY($1)` with a `Vec<Uuid>` bind; MySQL uses `IN (?)` expanded via `sqlx::QueryBuilder<MySql>` with `.push_bind` / `.separated(", ")` (since `Vec<T>` does not implement `Encode<MySql>` / `Type<MySql>` in sqlx 0.8 — only `Vec<u8>` does); SQLite uses positional `?` expansion. Verify each adapter's `mark_published` matches its dialect.
- All `sqlx::query[_as/_scalar]` calls in the MySQL adapter need the `sqlx::MySql` turbofish (sqlx 0.8 with all 3 driver features enabled picks `Postgres` as the default; the MySQL adapter must disambiguate explicitly).
- All `Utc::now()` / `NOW()` SQL calls in the MySQL adapter use `UTC_TIMESTAMP(6)` (UTC, microsecond precision) — `NOW()` returns local time on some MySQL configurations.
- SQLite UUIDs are bound as `uuid::fmt::Hyphenated` (the canonical 36-char hyphenated text form) — `sqlx::types::Uuid` would map to `BLOB(16)`, which would violate the `CHECK(length(x) = 36)` invariant on the DDL.
- The `after_connect` hook on each adapter sets the dialect's per-connection session state: PG issues `SET search_path = engine, public`; MySQL issues `SET NAMES utf8mb4 COLLATE utf8mb4_unicode_ci`; SQLite issues `PRAGMA journal_mode = WAL`, `PRAGMA synchronous = NORMAL`, `PRAGMA foreign_keys = ON`.
- The MySQL `ensure_multi_statements` helper in `connection.rs` idempotently appends `?multi_statements=true` to the URL; 4 unit tests cover the no-query, with-query, already-present, and case-insensitive cases. Verify the unit tests are present.
- The build-plan § "Phase 1 outcome." claims 124 tests pass workspace-wide; verify by running `cargo test --workspace` and comparing the count (124 was the Phase 0 close-out count + 4 from the MySQL `connection::tests` URL helper unit tests + 3 per-adapter outbox e2e = 131 expected at Phase 1 close).
- The cross-adapter test (all 4 adapters in one scenario) is **Phase 16 work**, not Phase 1 (per the handoff). Phase 1 has per-adapter e2e only. The `storage_parity_suite` row in `docs/coverage.toml` is the Phase 16 deliverable.
