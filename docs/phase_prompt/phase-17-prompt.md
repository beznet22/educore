# Phase 17 Prompt — Production readiness

## Mission

Deliver Phase 17 (multi-tenant integration suite, load test, cross-compile matrix, security review, docs audit). **Spec-faithful** per `docs/build-plan.md` § "Phase 17" (lines 1703+).

## Required Reading

- `docs/handoff/PHASE-16-HANDOFF.md` (carry-over)
- `docs/build-plan.md` § "Phase 17" (lines 1703+)
- `docs/guides/saas-backend.md` (50+ integration scenarios)
- `AGENTS.md` (10-point validation checklist)
- `docs_guidlines/system.md`, `docs_guidlines/execution_guidlines.md`
- `docs/phase_prompt/README.md` (closing-agent verification checklist)

## Deliverables

(1) Multi-tenant integration suite (50+ scenarios from `docs/guides/saas-backend.md`); (2) Load test (10k students + bulk fee invoice generation, p95 < 500 ms on PG); (3) Cross-compile matrix (5 targets: Linux x86_64/aarch64, macOS x86_64/aarch64, Windows x86_64); (4) Security review (every public command surface per the 4-point checklist); (5) Docs audit against the 10-point `AGENTS.md` checklist.

## Starting Point

35 closed crates (10 cross-cutting + 10 domain + 5 port-adapter + 1 SDK + 1 CLI + testkit + parity + 4 storage adapters). The Phase 16 hand-off enumerates what shipped.

## Per-Deliverable Gotchas

- 10k-student load test runs on PG (not SurrealDB); document the SurrealDB deviation in the hand-off.
- Cross-compile verifies `rustls` only (no `native-tls`); check Phase 16 dev-deps don't pull in native-tls.
- Security review uses `crates/educore-core::tenant::TenantContext`; the `educore-rbac::CapabilityCheck` pattern is the template.
- Docs audit answers "Yes" to all 10 points in `AGENTS.md` § "Validation Checklist" for every crate.

## Exit Criteria

4 deliverables complete; `cargo test --workspace` green; `cargo clippy --workspace --all-targets -- -D warnings` green (now possible with Phase 16's settings + documents clippy fix); `cargo fmt --check` green; lint binary green; `PHASE-17-HANDOFF.md` + `phase-18-prompt.md` (or "no Phase 18" if Phase 17 is the terminal phase) + `progress-tracker.md` + `build-plan.md § "Phase 17 outcome."` written.

## When You Are Stuck

`PHASE-16-HANDOFF.md` is the foundation. `cargo run -p educore-core --bin lint --features lint` is the no-gaps gate. The 5 Phase 15 port-adapter reference impls are the templates. The closing-agent verification checklist is in `docs/phase_prompt/README.md`.

**Working With Subagents:** Workstreams: A=multi-tenant integration suite; B=load test (PG-only); C=cross-compile matrix (5 targets); D=security review + docs audit (read-only verification sweep).

**Subagent Orchestration:** To prevent duplicate work, every macro subagent enforces: (1) **File-level ownership** — every file is assigned to exactly one subagent. (2) **Section-level pre-allocation** — for files touched by multiple workstreams, the prep subagent pre-creates named section markers. (3) **Sequential phase gates** — P0 prep → R1 reconcile-prep → wave 1/2/3 parallel workstreams → R2 reconcile-impl → 5-tests → 6-docs → R3 final-validation (9-command gate). (4) **Atomic commits per microtask** — every subagent produces exactly one commit with `Phase 17: <scope> (<workstream-letter>)` message + `Co-Authored-By: Antigravity <antigravity@google.com>` trailer. (5) **Reconciler subagents are read-only**.