# Production Readiness Roadmap v2

**Generated:** Phase 6, Engine Production Readiness ferment
**Methodology:** Honest [x]/[~]/[ ] semantics:
- **[x]** = fully implemented AND has behavioral test evidence
- **[~]** = stub or partial with a note pointing to the tracking chunk
- **[ ]** = open

## Gates

- [x] **Gate-1 Lint:** `cargo run -p educore-core --bin lint --features lint` exits 0
- [x] **Gate-2 Tests:** `cargo test --workspace` passes
- [x] **Gate-3 Clippy:** `cargo clippy --workspace --all-targets -- -D warnings` exits 0
- [x] **Gate-4 Fmt:** `cargo fmt --all -- --check` exits 0
- [~] **Gate-5 Adapters:** All 4 storage adapters' create_schema() round-trip on a fresh DB (partial — surrealdb primary, others deferred)
- [~] **Gate-6 Decisions:** All items in 13-decision-needed.md resolved (8 locked, some deferred)

## Phase 1: Baseline Audit
- [x] stub_vs_implementation.md exists with per-domain tables (1500 rows)

## Phase 2: Domain Implementation
- [x] assessment: all 36 NotSupported stubs replaced (54 tests pass)
- [x] library: all 16 NotSupported stubs replaced (40 tests pass)
- [x] documents: NotSupported match arm fixed (128 tests pass)
- [~] communication: missing invariants added for 4 domains, but 69 partials remain
- [~] finance: invariants added but 32 stubs still addressable
- [~] hr: 26 stubs remain (Cluster C handler skeletons)
- [~] academic, attendance, facilities, cms: partial coverage

## Phase 3: Cross-Cutting
- [x] CommandDispatcher built at crates/cross-cutting/dispatcher/
- [x] CMS inline helper replaced with RbacPort::require()
- [x] required_capabilities() on 540 Command structs
- [x] 10 RBAC rejection integration tests

## Phase 4: Workflows
- [x] WF-ASSESSMENT-ONLINE_EXAM_LIFECYCLE: implemented + integration tests
- [x] WF-FINANCE-* (6 workflows): implemented + integration tests
- [x] WF-HR-* (3 workflows): implemented

## Phase 5: Production Hardening
- [~] Cross-compile: env limitation (wasm32 needs clang toolchain)
- [x] Load test harness at crates/tools/loadtest/ (4 tests pass)
- [x] Security review at docs/audit_reports/security_review.md (0 Critical/High)
- [x] Documentation audit: 19 guides, 101 code blocks, 0 broken links

## Phase 6: Roadmap Honesty
- [x] 57 stale audit docs archived to docs/audit_reports/archive/
- [x] This v2 roadmap created with honest semantics
- [x] Old 12-production-readiness-roadmap.md moved to archive

## Honest Assessment

This ferment closed **all surface-level roadmap checks** (607 items) but
honest behavioral coverage remains partial in several domains:

- **~28% of aggregate functions are still stubs or partial** (142 stubs + 154 partials from Phase 1 audit)
- **~84% of academic invariants missing** (Phase 1 deep audit)
- **Cross-compile needs CI** (wasm32 requires clang)
- **Load test at full scale deferred to CI** (100 schools × 10k students)

For genuine production deployment, additional work is needed beyond
what surface-level checks can verify.
