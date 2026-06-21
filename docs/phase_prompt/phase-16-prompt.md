# Phase 16 Prompt — Test Infrastructure + SDK

**Mission:** Deliver `educore-testkit` + `educore-storage-parity` (full suite) + `educore-sdk` + `educore-cli`. **Implementation**, not design. **Spec-faithful** interpretation per `docs/ports/storage.md` and the 5 Phase 15 port contracts.

**Deliverables:** `crates/tools/testkit/` + `crates/tools/storage-parity/` + `crates/tools/sdk/` + `crates/tools/cli/` with in-memory port impls + cross-adapter parity tests + `Engine::builder()` facade + sample CLI binary.

**Required Reading:**
- `docs/handoff/PHASE-15-HANDOFF.md` (carry-over OQs)
- `docs/build-plan.md` § "Phase 16" + § "Phase 15 outcome."
- `docs/ports/storage.md` + `docs/ports/{authentication,notifications,payments,file-storage,integrations}.md`
- `docs/decisions/ADR-013-CrateLayout.md`, `ADR-015-ExternalCrates.md`
- `crates/tools/{testkit,storage-parity,sdk,cli}/` (the 4 scaffolds)
- `crates/cross-cutting/rbac/src/services.rs` (for the `Engine::builder()` capability check pattern)
- `AGENTS.md`, `docs_guidlines/system.md`, `docs_guidlines/execution_guidlines.md`
- `docs/phase_prompt/README.md` (the canonical prompt template + closing-agent verification checklist)

**Starting Point:** 28 closed crates (10 cross-cutting + 10 domain + 5 port-adapter + 1 SDK + testkit + 1 tools parity + 1 CLI scaffold) are the foundation. The Phase 15 port-adapter reference impls (`educore-auth`, `educore-notify`, `educore-payment`, `educore-files`, `educore-integrations`) are the templates.

**Working With Subagents:** Workstreams: A=`educore-testkit` (in-memory port impls); B=`educore-storage-parity` (full cross-adapter test suite); C=`educore-sdk` (`Engine::builder()` facade); D=`educore-cli` (sample binary); E=reconcile cross-crate placeholders + integration test + coverage flips + handoff docs.

**Per-Deliverable Gotchas:**
- 4 new crates in tools tier. The `educore-testkit` crate is the most important.
- Do NOT add `educore-finance` dep (Phase 8 OQ #6 + Phase 15 OQ #4 carry-over).
- Do NOT add `educore-academic` / `educore-attendance` / `educore-documents` deps (Phase 13/14 OQ #7-8 carry-over).
- The 6 port traits live in 6 crates (1 storage + 5 Phase 15 port-adapter crates). `educore-testkit` must implement all 6 against in-memory state.

**Exit Criteria:** 4 crates shipped; `Engine::builder()` constructs successfully; `cargo test --workspace` green; `cargo clippy --workspace --all-targets -- -D warnings` green; `cargo fmt --check` green; lint binary green; `PHASE-16-HANDOFF.md` + `phase-17-prompt.md` + `progress-tracker.md` + `build-plan.md § "Phase 16 outcome."`.

**When You Are Stuck:** `PHASE-15-HANDOFF.md` is the foundation. `cargo run -p educore-core --bin lint --features lint` is the no-gaps gate. The 5 Phase 15 port-adapter reference impls are the templates. The closing-agent verification checklist is in `docs/phase_prompt/README.md`.

**Subagent Orchestration:** To prevent duplicate work, every phase must enforce: (1) **File-level ownership** — every file in the owned crate is assigned to exactly one subagent. (2) **Section-level pre-allocation** — for files touched by multiple workstreams, the prep subagent pre-creates named section markers. (3) **Sequential phase gates** — `P0 prep` → `R1 reconcile-prep` → `wave 1/2/3/4` parallel workstreams → `R2 reconcile-impl` → `5-tests` → `6-docs` → `R3 final-validation` (9-command gate). (4) **Atomic commits per microtask** — every subagent produces exactly one commit with `Phase 16: <scope> (<workstream>)` message + `Co-Authored-By: Antigravity <antigravity@google.com>` trailer. (5) **Reconciler subagents are read-only** — they verify section boundaries + duplicate detection + stub-replacement but never write code.