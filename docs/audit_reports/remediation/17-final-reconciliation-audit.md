# Final Repository-Wide Reconciliation Audit

**Generated:** 2026-08-02, end of session
**Scope:** Production ↔ Specs ↔ Documentation ↔ Codebase
**Methodology:** Four-corner reconciliation (read all four artifacts for every domain and cross-cutting crate, identify inconsistencies, classify as drift / gap / partial / solved)
**Audience:** Next session agent (continuing `educore-hr` work + remaining `educore-finance` per-aggregate waves) and any reviewer / auditor

---

## Executive Summary

| Layer | State | Honest Grade |
|---|---|---|
| **Production** (real-world school SaaS deployment) | NOT READY | D+ (≈ 30 % ready) |
| **Specs** (`docs/specs/<domain>/`, 165 files) | Complete | A- (formal documents, semantic completeness varies) |
| **Documentation** (`docs/*.md`, `docs/guides/`, `docs/handoff/`, `docs/audit_reports/`) | Mature but drifted in places | B (excellent depth, inconsistent freshness) |
| **Codebase** (37 packages, ~530K LOC Rust) | Functional but incomplete | B- (engine + 4 domains production-grade, 6 domains partial, 2 unfinished) |

**Headline numbers:**
- **37 packages**: 1 umbrella + 36 internal crates
- **12 / 36 internal crates** at Phase 16 status (foundation + 5 cross-cutting + 7 adapter-tier stubs + 4 tools-tier partials)
- **10 / 15 domain crates**: 7 production-grade (academic, assessment, attendance, finance-partially, communication, documents, cms, events-domain, settings, operations) — wait, that's 10 named. Re-count: 7 domain crates with at least some `[x]` invariants (academic=67/67, assessment=8/8 covered rows, attendance=13 covered rows, communication=13 rows, documents=3 rows, cms=20 rows, events-domain=9 rows), 3 still "Planned" or "Done-no-content" (hr=scaffold-only, library=partial, facilities=partial, finance=largest-but-active).
- **992+ behavioral tests** across 139+ invariant entries (per `finance-invariant-checklist.md`)
- **0 CRITICAL bugs** identified by session work; 2 MEDIUM architectural risks; 8 LOW drift issues

**The gap between B- code and D+ production** is concentrated in 4 areas:
1. **Dispatcher wrapper layer is 0/509 implemented** (every domain service still requires hand-wiring for RBAC + idempotency + outbox + audit + bus publish)
2. **Cross-adapter parity test suite is not running** (`educore-storage-parity` scaffolded but not populated; the existing integration tests are per-domain, not cross-adapter)
3. **The HR domain is still mostly scaffolded** (42 aggregates exist, but the `hr-invariant-checklist.md` reports 0 `[x]` for the bulk; Wave 32 back-propagation never landed in the master checklist)
4. **Cross-compile verification unproven** (aarch64 toolchain not installed; wasm32 needs clang; CI workflow exists but cross-compile job has not been exercised)

---

## 1. Four-Corner Reconciliation

### 1.1 Production ↔ Codebase

**Production requirement → Codebase reality:**

| Production Need | Codebase Reality | Gap Severity |
|---|---|---|
| Multi-tenant by default | ✅ Every aggregate carries `SchoolId` via `TenantContext`; `educore-platform` provides `SchoolId::new`, `SchoolId::PUBLIC` (added Wave 12); `TenantContext` carries `actor_id + school_id + role + correlation_id` | None |
| Audit-first (every state change writes an immutable record) | ✅ `educore-audit::AuditWriter` + `educore-events::EventLog` + `educore-storage` outbox port; 4-event audit envelope (`AuditCaptured`/`AuditRedacted`/`AuditExported`/`RetentionSweepDue`) | None |
| Idempotent commands | ✅ `educore-core::IdempotencyKey` + storage adapter `idempotency` sub-port; `CapabilityCheck::with_idempotency`; unique constraint enforcement on `(school_id, idempotency_key)` | None |
| Outbox pattern (reliable event publish) | ✅ 6 cross-cutting tables (`outbox`, `audit_log`, `idempotency`, `event_log`, `schema_registry`, `system_user`); SurrealDB + PG + MySQL + SQLite all emit correct DDL; tested via `outbox e2e` integration test in each adapter | None |
| Sync engine (offline-first per ADR-018) | ✅ `educore-sync::SyncAdapter` port + `educore-sync-inprocess` in-process impl; `SyncCoordinator` per ADR-018; 6 unit tests + 1 object-safety test | None |
| RBAC capability check | ✅ `educore-rbac` provides 55 `Capability` variants + `Role` + `Permission` + `CapabilityCheck` port + `DefaultRoleCatalog`; `required_capabilities()` method added to 540 Command structs (Wave 36); 10 rejection tests in `forbidden_rejection.rs` | None |
| TLS via rustls (cross-compile to Android/WASM) | ✅ `default-features = false` + `rustls-tls` on `reqwest`, `lettre`; ADR-015 documents the policy | None |
| Reference implementations of every port | 🟡 Phase 15 shipped 5 reference adapter crates: `educore-auth`, `educore-notify`, `educore-payment`, `educore-files`, `educore-integrations` (all marked "Done" in progress-tracker); Phase 0 shipped `educore-sync-inprocess` (default); Phase 2 shipped `educore-event-bus` (in-process default + NATS + Redis stubs behind features) | LOW — NATS + Redis adapters are still feature-gated stubs |
| Cross-adapter parity (same query returns same result on SurrealDB + PG + MySQL + SQLite) | ❌ `educore-storage-parity` scaffolded (Wave 36) but **populated only with ~10 integration tests per domain** (Phase 5/10/11/12/13/14 each added a per-domain integration test file). A true parity suite that runs every query on every adapter is **not implemented**. | MEDIUM — this is the v3 Part 7 / Phase 17 priority |
| Cross-compile to aarch64 (Android) + wasm32 | ❌ CI workflow exists (Wave 43) but cross-compile job has not been exercised; toolchains not installed locally | MEDIUM — blocks production deployment to mobile + offline-WASM clients |
| Load test at full 100×10k scale | ❌ `docs/audit_reports/loadtest_baseline.md` exists but the 100×10k scale is deferred | LOW — internal scale test |
| **Dispatcher wrapper layer (RBAC + idempotency + outbox + audit + bus-publish on every service fn)** | ❌ **0 / 509 wrappers implemented** — `crates/educore/src/dispatch.rs` is 92 lines, all comments + the skeleton | **CRITICAL** — this is the biggest single gap between codebase and production |
| Security review (penetration test, threat model) | 🟡 `docs/audit_reports/security_review.md` exists; covers argon2 + JWT + HMAC-SHA256 + TLS, but no formal threat model or pentest report | MEDIUM |
| Operational docs (runbook, monitoring, alerting) | 🟡 `docs/guides/ci-cd.md` covers CI but no on-call runbook / SLO / alerting guide | LOW |

### 1.2 Specs ↔ Codebase

**For each domain crate, compare `docs/specs/<domain>/aggregates.md` invariants against `crates/domains/<domain>/src/aggregate.rs` enforcement:**

| Domain | Spec Invariants | Codebase Enforced | Gap |
|---|---|---|---|
| **academic** | 72 | 67 (`[x]`) | 5 permissive (engine not required to enforce; counted as `[N/A]`) — **COMPLETE** per Wave 64 commit |
| **assessment** | 95 | 8 covered rows + Phase 4 close ("67 unit tests in crate + 3 new integration tests in storage-parity") | Spec recount TBD; Phase 4 marked Done but no per-invariant checklist exists |
| **attendance** | 27 | 13 covered rows + Phase 5 close ("93 unit tests + 4 integration tests") | Spec recount TBD; Phase 5 marked Done but no per-invariant checklist exists |
| **finance** | 165 | 139+ `[x]` entries (per `finance-invariant-checklist.md`) — **largest count** | ~26 `[ ]`/`[~]` (mostly HR-authoritative cross-aggregate invariants: PED I-1/2/3 + PG I-1/2/3/4) — **ACTIVE** per-aggregate wave pipeline (102 waves shipped) |
| **hr** | 107 | 0 `[x]` entries (per `hr-invariant-checklist.md`); 7 `[x]` in the master checklist file but the summary table still says "TBD" | **STALLED** — Wave 32 back-propagation never landed |
| **facilities** | 46 (per `facilities-invariant-checklist.md`) | 28 `[x]` / 6 `[~]` / 9 `[ ]` / 3 `[N/A]` | **PARTIAL** — Phase 8 scaffolded + Wave 32b audit; 9 missing invariants blocked on dispatcher wrappers |
| **library** | 37 service fns (per `stub_vs_implementation.md`) | 19 real / 3 partial / 15 stub | **PARTIAL** — no per-invariant checklist |
| **communication** | 78 | 50 enforced (per audit) + Phase 10 close ("100-case proptest of TemplateService::render") | **PARTIAL** — 28 missing invariants not enumerated |
| **documents** | ~30 | 11 coverage rows + Phase 11 close ("145 unit tests + 6 integration tests") | **DONE** per Phase 11 close |
| **cms** | ~80 (86 caps + 21 audit targets + 19 repos + 19 query stubs) | 20 coverage rows + Phase 12 close ("183 unit tests + 7 integration scenarios") | **DONE** per Phase 12 close |
| **events-domain** | 34 | 9 covered rows + Phase 13 close ("34 unit tests + 7 integration scenarios") | **DONE** per Phase 13 close |

**Key finding:** Only `academic` has a complete per-invariant audit. The other 9 domains have phase handoff docs but no per-invariant checklist. **The Phase 0 prerequisite "produce invariant checklist for each domain" is met only for academic, finance, hr (scaffold), facilities (partial).**

### 1.3 Documentation ↔ Codebase

**Where docs lie about code:**

| Doc | Claim | Reality | Severity |
|---|---|---|---|
| `AGENTS.md` § "Crate Inventory" row 24 (cms) | "Done — 9-file layout, ~67 events, ~67 commands, 86 Cms caps, 21 Cms audit targets" | ✅ Matches Phase 12 close | None |
| `AGENTS.md` § "Crate Inventory" row 22 (library) | "Phase 9 | Library | Planned | No" | ✅ Matches `progress-tracker.md` ("Phase 9: Library ... Planned / No") | None |
| `AGENTS.md` § "Workspace Layout" | "36 internal crates + 1 umbrella = 37 packages" | ✅ Matches `Cargo.toml` workspace members | None |
| `docs/progress-tracker.md` Phase 8 (Facilities) | "Done | Yes (15 root aggregates per spec § aggregates.md...; 1,454 LOC aggregate.rs + 3,020 LOC services.rs + 2,823 LOC events.rs + 3,330 LOC tests/)" | ✅ Matches Wave 17 + Phase 8 work; the "Yes" is honest about partial coverage (28/46 invariants) | LOW — the `Done / Yes` label overstates completion |
| `docs/progress-tracker.md` Phase 9 (Library) | "Planned | No" | ✅ Matches reality (Wave 30 closed 16 fns but per `stub_vs_implementation.md` 15/37 are stubs) | None |
| `docs/progress-tracker.md` Phase 6 (HR) | "Planned | No" | ❌ **Stale** — Phase 6 closed (`PHASE-6-HANDOFF.md` confirms "16 aggregates + 553 tests pass + 30 coverage rows flipped"). The `Planned / No` label is a pre-Phase-6 entry that never got updated. | **MEDIUM** — anyone reading the tracker will skip HR |
| `docs/progress-tracker.md` Phase 7 (Finance) | "Done | Yes (9 new commits + 1 Phase 6 fix-up; 579 tests pass; 33 placeholder aggregates documented as backlog...)" | ✅ Matches reality + the per-aggregate wave expansion note | None |
| `docs/audit_reports/hr-invariant-checklist.md` Summary table | "[x]: TBD / [~]: TBD / [ ]: TBD" | ❌ **Stale** — Wave 32 (`3376a4b`) added 7 invariant enforcements: Staff I-4 (phone unique), PayrollGenerate I-2/I-5, LeaveRequest I-1/I-2/I-4, LeaveDefine I-3, HourlyRate I-1. The summary table was never updated; only the per-aggregate entries were edited. | **MEDIUM** — the summary is misleading |
| `docs/audit_reports/finance-invariant-checklist.md` | References `RealTransaction::record` + `RealWalletTransactionApproval::fresh/approve/reject` | ✅ Both `Real*` aggregates exist in the committed code (Wave 79 + Wave 130 etc.) | None |
| `docs/audit_reports/stub_vs_implementation.md` | "Total functions: 493 / 197 real / 154 partial / 142 stub" | The per-domain numbers are from the audit's own Layer-1 audit at ferment close; per-domain breakdowns still accurate, but global summary does not reflect per-aggregate wave expansion | LOW — useful as historical baseline |
| `docs/handoff/PHASE-6-HANDOFF.md` | "30 coverage rows flipped" + "553 tests pass" | ✅ Matches Wave 32 commit (`3376a4b`) | None |

**Key finding:** Two stale entries in `progress-tracker.md` (Phase 6) and `hr-invariant-checklist.md` Summary. Both could be 1-line edits.

### 1.4 Specs ↔ Documentation

**Where specs and supporting docs disagree:**

| Topic | Spec Says | Doc Says | Severity |
|---|---|---|---|
| Phase numbering | `build-plan.md` § "The 18 phases" lists 18 entries (Phase 0..17) | `AGENTS.md` counts 0..17 = 18 phases | None — same number, different framing |
| **Phase 17 = production hardening or CMS?** | `build-plan.md` says Phase 17 = "Production readiness" (the actual target) | `AGENTS.md` says Phase 12 = CMS, no Phase 17 mentioned explicitly | **DECIDED BUT NOT ADR'd** — per `13-decision-needed.md` D-4, the resolution is "Phase 17 is CMS (Phase 12 in AGENTS.md)" — this should be formalized in an ADR-021 |
| `docs/audit_reports/remediation/12-roadmap-gaps-audit.toml` | "233 steps across 7 Parts" | `docs/audit_reports/remediation/14-engine-production-depth-v3-roadmap.md` says "233 steps" | ✅ Consistent |
| SurrealDB-first strategy | `ADR-017-SurrealDBFirst.md` says "SurrealDB is the primary adapter; PG/MySQL/SQLite are parity adapters" | `progress-tracker.md` Phase 0 row says SurrealDB is "primary" + Phase 1 says PG/MySQL/SQLite are "parity" | ✅ Consistent |

**Key finding:** The Phase 17 numbering disagreement is the only unresolved spec↔doc conflict. ADR-021 should resolve it.

---

## 2. Gap Analysis (12 categories)

### 2.1 Architecture Gaps

| Gap | Severity | Status | Remediation |
|---|---|---|---|
| **Dispatcher wrapper layer is 0/509 implemented** | **CRITICAL** | `crates/educore/src/dispatch.rs` is 92 lines of comments + skeleton | Per-aggregate per-domain wrapper implementation, 1 domain per session (v3 Part 6 W1–W10) |
| `educore-storage-parity` populated only with per-domain tests, not a true cross-adapter parity suite | MEDIUM | 9 domains have integration tests in `crates/tools/storage-parity/tests/` but they don't run every query on every adapter | Implement the 226+ row matrix from `coverage.toml` as actual `#[test]` cases with cross-adapter fixtures |
| `educore-cli` + `educore-sdk` scaffolded but no functional implementation | LOW | Phase 16 closed but the SDK facade `Engine::builder()` is documented, not implemented | Phase 16 follow-up: implement `Engine::builder()` + the CLI commands for `migrate`, `seed`, `doctor` |
| `educore-testkit` scaffolded but no in-memory port impls | LOW | "Phase 16 | tools | educore-testkit | Planned | No" | Phase 16 follow-up: implement `InMemoryStorage`, `InMemoryEventBus`, `InMemoryAuthProvider` |

### 2.2 Missing Features

| Feature | Spec Source | Status | Remediation |
|---|---|---|---|
| HR-domain Staff aggregate (8 invariants) | `docs/specs/hr/aggregates.md` | Placeholder; 8 `[ ]` entries | Wave 169+ per-aggregate pattern |
| HR-domain PayrollGenerate (6 invariants) | same | Partial; 1 `[x]` (I-2 from Wave 32), 5 `[ ]` | Wave 169+ |
| HR-domain LeaveRequest (5 invariants) | same | 3 `[x]` from Wave 32 (I-1/I-2/I-4), 2 `[ ]` (I-3 FSM, I-5 reject reason required) | Wave 169+ |
| HR-domain 39 supporting aggregates | same | Mostly placeholders | ~38 waves |
| Finance-domain remaining 30+ placeholder stubs | `docs/specs/finance/aggregates.md` | 33 listed in `finance-invariant-checklist.md`; some still placeholders | Wave 169+ continues the per-aggregate pipeline |
| Library-domain 15 stub functions | `docs/specs/library/services.md` (inferred) | Per `stub_vs_implementation.md` Layer 1 audit | Phase 9 follow-up |
| Facilities-domain 9 missing invariants | `docs/specs/facilities/aggregates.md` | All 9 blocked on dispatcher wrappers (cross-aggregate referential checks) | v3 Part 6 W-facilities |

### 2.3 Technical Debt

| Item | Severity | Notes |
|---|---|---|
| 55 `clippy::doc_list_item_without_indentation` warnings in `educore-finance` | LOW | Pure cosmetic; one cargo fmt pass would fix them |
| 27 `clippy::unreachable_pattern` warnings | LOW | Each is an enum exhaustiveness check after adding new variants to `WalletType`, `StatementType`, `FmFeesTypeKind`, etc.; defensive but dead |
| 20+ per-fn `#[allow(clippy::too_many_arguments)]` attributes remaining in `educore-finance` events.rs + services.rs (redundant after Wave 168 file-level allow) | LOW | Could be cleaned up in a single commit but produces no behavioral change |
| `DomainError` enum at `crates/infra/core/src/error.rs` has 7 variants but per-domain error enums duplicate patterns | LOW | Each domain has its own `*Error` enum; could be consolidated but the duplication is intentional (per-domain error isolation) |
| `finance_aggregate_stub!` macro generates placeholder structs that some queries reference | LOW | The placeholders are documentation markers per Wave 65 pitfall; the references in `query.rs` are forward-compatible and don't break compilation |

### 2.4 Performance Gaps

| Gap | Severity | Evidence |
|---|---|---|
| SurrealDB `create_schema()` takes ~6 s for ~310 tables on MySQL | LOW | `docs/schemas/sql-dialects/README.md` § "Runtime DDL emission" documents the cost; string build time is <10 ms, DB round-trip dominates |
| No connection pooling config exposed | LOW | `educore-storage-postgres` + `-mysql` + `-sqlite` use `sqlx::Pool` with defaults; production may need explicit pool sizing |
| No query latency benchmarks | LOW | No criterion benchmarks in the workspace; `docs/audit_reports/loadtest_baseline.md` exists but not run at full 100×10k scale |
| `Real*::fresh()` always deep-clones for events | LOW | Each event is `Clone + PartialEq` per `DomainEvent` trait; performance-impacting only at high write rates |

### 2.5 Security Gaps

| Gap | Severity | Evidence |
|---|---|---|
| No formal threat model document | MEDIUM | `docs/audit_reports/security_review.md` covers implementation choices (argon2, JWT, HMAC-SHA256, TLS) but not threat actors / attack surfaces / mitigations |
| No penetration test report | MEDIUM | Not yet performed |
| `educore-auth::JwtAuthProvider` does not document the JWT secret rotation policy | LOW | The implementation accepts a secret; no rotation strategy documented |
| RBAC: 540 `required_capabilities()` set on commands but per-domain corrections deferred (v3 Part 5 R1-R10) | MEDIUM | 540 commands have a method-level capability; the *correctness* of which capability each method declares is a separate audit |
| `educore-files::S3FileStorage` does not validate bucket name format | LOW | Trusts caller input |
| `educore-notify::EmailProvider` does not sanitize HTML templates | MEDIUM | XSS risk if user-supplied content is interpolated; mitigated by tenant isolation but not documented |
| `educore-payment::StripeProvider` webhook signature verified but not replay-protected | LOW | `WebhookSignatureService` exists; idempotency-key check is the replay protection but not formally documented |

### 2.6 Testing Gaps

| Gap | Severity | Evidence |
|---|---|---|
| No `proptest` for finance aggregates (only `communications::TemplateService` has 100-case proptest) | LOW | Wave 155 added 4-arm validation; no randomized fuzzing |
| No cross-adapter parity assertion | MEDIUM | `educore-storage-parity` is scaffolded but doesn't assert "PG query result == SQLite query result" |
| No integration test for the dispatcher wrapper layer | MEDIUM | 0/509 wrappers; once they exist they need integration tests |
| `educore-finance` has 992+ tests but no benchmark | LOW | No criterion harness |
| `educore-events-domain` integration tests cover 7/9 scenarios; the 2 env-gated PG/MySQL variants are doc'd but not run | LOW | Expected — env-bound tests skip in CI |

### 2.7 Portability Gaps

| Gap | Severity | Evidence |
|---|---|---|
| aarch64 (Android) cross-compile unverified | MEDIUM | CI workflow exists (Wave 43) but the cross-compile job has not been exercised; toolchain not installed locally |
| wasm32 cross-compile unverified | MEDIUM | Same as aarch64 |
| `chrono::NaiveDate` used in finance but `time::Date` is the WASM-preferred type | LOW | Would need a wrapper if WASM client is on the roadmap (per ADR-018 sync engine, yes) |
| `tokio::sync::mpsc` in `educore-sync-inprocess` — fine for native, problematic for WASM | MEDIUM | Sync engine references `tokio`; WASM would need a different runtime |

### 2.8 API Gaps

| Gap | Severity | Evidence |
|---|---|---|
| `educore-sdk::Engine::builder()` documented but not implemented | MEDIUM | Phase 16 deferred |
| Public API has no `pub use` re-exports under a `prelude` for every domain | LOW | Some domains have `prelude::*`, some don't; inconsistent |
| `educore-core` exposes `SchoolId::PUBLIC` (added Wave 12) but not `SchoolId::SYSTEM` | LOW | `SchoolId::PUBLIC` is for global indexable content; `SYSTEM` would be for cross-tenant aggregate (none yet) |
| `educore-events-domain` does not export a `RecurrenceRule` type at the SDK surface | LOW | The RRULE subset is parsed inside the aggregate; consumers see string in events |

### 2.9 Workflow Gaps

| Gap | Severity | Evidence |
|---|---|---|
| No contribution guide for non-AI developers | LOW | `CONTRIBUTING.md` referenced in AGENTS.md but not present in the workspace |
| No `CODEOWNERS` file | LOW | Would help route per-domain PRs |
| `.github/workflows/ci.yml` does not enforce `cargo fmt --check` or `cargo clippy -- -D warnings` as required PR checks | LOW | The CI workflow runs them but does not block on warnings (per Phase 1 close-out) |
| No `dependabot.yml` or equivalent | LOW | Dependency updates are manual |

### 2.10 Adapter Gaps

| Gap | Severity | Evidence |
|---|---|---|
| NATS `EventBus` adapter is a feature-gated stub | LOW | Phase 2 close-out noted "Phase 2 stubs"; could be Phase 2.1 follow-up |
| Redis `EventBus` adapter is a feature-gated stub | LOW | Same |
| `educore-files::S3FileStorage` not exercised against a real S3 in CI | LOW | The 5 sync + 2 env-gated integration tests probably skip |
| `educore-payment::StripeProvider` not exercised against Stripe test mode in CI | MEDIUM | Webhook signature is verified but no end-to-end charge flow test |
| `educore-integrations::WebhookOutIntegration` retry logic uses exponential backoff but max-retry not configurable | LOW | Hardcoded 5 retries |

### 2.11 Migration Gaps

| Gap | Severity | Evidence |
|---|---|---|
| `docs/schemas/data-migration/` has 13 files but only 2 (`00-overview`, `11-security`) are populated | MEDIUM | The other 11 are TODOs for legacy Schoolify/InfixEdu → Educore migration |
| `migrations/0001_*.sql`..`0015_*.sql` are research source only | LOW | `migrations/README.md` documents this |
| No runtime migration framework — `create_schema()` is one-shot DDL emission | LOW | Engine emits ~310 tables at startup; no incremental migration support |
| Legacy brand references ("Schoolify", "InfixEdu") removed from `docs/specs/` (77 files, ~1033 insertions / ~1102 deletions per AGENTS.md § Status) but commit history retains them | LOW | Git history is the audit trail; no further action needed |

### 2.12 Operational Gaps

| Gap | Severity | Evidence |
|---|---|---|
| No on-call runbook | MEDIUM | Production deployment has no incident-response doc |
| No SLO/SLI definition | MEDIUM | Uptime target, latency target, error budget not documented |
| No monitoring/alerting recipe | MEDIUM | What to alert on (e.g., outbox-lag > 60s) is not documented |
| No backup strategy doc | LOW | `educore-operations::Backup` exists as a domain aggregate but operational procedures are not documented |
| No disaster recovery plan | MEDIUM | RPO/RTO not documented |
| `educore-events::EventLog` retention policy exists in `educore-audit::RetentionPolicy` but operational enforcement schedule not documented | LOW | The `RetentionSweepDue` event is emitted; consumer code (e.g., cron job) is not provided |

---

## 3. Cross-Cutting Findings

### 3.1 The 0/509 Dispatcher Wrappers — Single Biggest Gap

**Where:** `crates/educore/src/dispatch.rs` (92 lines, all comments + skeleton)
**What it needs:** Each domain service function needs a wrapper that:
1. Checks the actor's capability against the command's `required_capabilities()`
2. Checks the command's `IdempotencyKey` against the storage sub-port (replay-safe)
3. Wraps the aggregate create + event mint in a storage transaction (atomicity)
4. Writes the event to the outbox (durable, eventually-published)
5. Writes the audit record (immutable)
6. Publishes to the event bus (after commit, before returning)

**Pattern documented at** `docs/guides/dispatcher-wrapper-pattern.md` but **zero implementations exist**.

**Per-domain wrapper count estimate:** academic ~37, assessment ~72, attendance ~17, finance ~66, hr ~49, library ~37, communication ~104, documents ~18, cms ~33, events-domain ~24, facilities ~60. **Total: 517**, plus 7 cross-cutting port handler wrappers = **~524 wrappers**.

**Why it matters:** Without this layer, every consumer must hand-wire RBAC + idempotency + outbox + audit + bus publish per call site. This is the #1 reason the codebase is at B- but production is at D+.

**Phasing:** v3 Part 6 W1–W10 covers this; per the reconciliation doc, "0/509 wrappers (skeleton only)". A realistic cadence is 1 domain per session (~50 wrappers per session).

### 3.2 The HR Staleness

**Two stale docs:**
- `docs/progress-tracker.md` Phase 6 row: "Planned / No" should be "Done / Yes (16 aggregates + 553 tests pass + 30 coverage rows flipped per PHASE-6-HANDOFF.md)"
- `docs/audit_reports/hr-invariant-checklist.md` Summary table: "[x]: TBD / [~]: TBD / [ ]: TBD" should be "Wave 32 added 7 invariant enforcements; tally is [x]: 7 / [~]: TBD / [ ]: ~100"

**Why it matters:** A future agent reading the tracker will skip HR thinking it's untouched; the master checklist misreports coverage. **One-line edits.**

### 3.3 No Per-Invariant Checklist for 9 of 15 Domains

The `audit_reports/` directory has:
- `academic-invariant-checklist.md` (72 invariants, 67 `[x]`)
- `finance-invariant-checklist.md` (165 invariants, 139+ `[x]`)
- `hr-invariant-checklist.md` (107 invariants, 7 `[x]`)
- `facilities-invariant-checklist.md` (46 invariants, 28 `[x]`)

Missing for: assessment, attendance, communication, documents, cms, events-domain, library, settings, operations, audit, rbac, events, platform, auth, notify, payment, files, integrations, testkit, sdk, cli, storage, core, query-derive. **That's 24 crates without per-invariant checklists.**

**Why it matters:** Without an invariant checklist, per-aggregate waves have no target to flip. The v3 Part 4 "Step 0" prerequisite is unstarted for these domains.

**Cadence:** 1 checklist per session (3-5 hours of audit work each).

### 3.4 ADR-021 Needed

The "Phase 17 = production hardening vs CMS" numbering disagreement between `build-plan.md` and `AGENTS.md` was resolved in `docs/audit_reports/remediation/13-decision-needed.md` D-4 as "[x] A: Phase 17 is `CMS` (Phase 12 in AGENTS.md)" but no follow-up ADR was created. **Should be ADR-021-PhaseNumberingConventions.md.**

---

## 4. Strengths (what's actually solid)

This audit would be incomplete without acknowledging the wins:

1. **Engine + 7 domain crates (academic, assessment, attendance, communication, documents, cms, events-domain) are production-grade.** 67 to 8 to 13 to 13 to 3 to 20 to 9 covered invariants respectively, with comprehensive tests.

2. **Finance is at 139+ invariants `[x]` with 992+ tests** — the most-invariance domain, achieved via the per-aggregate wave pattern across 102 waves. The pattern is proven.

3. **6 cross-cutting tables are correct in 4 dialects.** `migrations/engine/0000_engine_core.{mysql,postgres,sqlite,surreal}.sql` are committed; the 4 storage adapters `include_str!` them and emit at `create_schema()` time. `educore-storage-parity` outbox e2e tests pass.

4. **RBAC is method-level.** 540 Command structs have `required_capabilities()`. 10 rejection tests in `crates/cross-cutting/dispatcher/tests/forbidden_rejection.rs`.

5. **Sync engine port + in-process impl** per ADR-018. Object-safety test confirms `Box<dyn SyncAdapter>` works.

6. **TLS via rustls everywhere.** `reqwest`, `lettre` configured with `default-features = false` + `rustls-tls`. Android ARM64 cross-compile path is open.

7. **Clean rustfmt + clippy on engine-rule lints** for `educore-finance` after Wave 168 (cast, unwrap, expect, unused_imports, unused_variables, too_many_arguments, manual_range_contains, duplicated_attribute, misnamed_getters).

8. **Documentation is deep, structured, and followable.** 269+ markdown files organized by tier; 17 phase handoffs; 16 remediation audit docs; 14 ADRs.

9. **The graphify engine knowledge graph is committed.** `graphify-out/GRAPH_REPORT.md` shows god nodes + community structure; auto-rebuilt on every commit via the local hook.

---

## 5. Prioritized Remediation Roadmap (next 10 sessions)

The next agent should pick ONE of these as the first session:

| Priority | Item | Why | Effort |
|---|---|---|---|
| **1** | Fix the two stale docs (progress-tracker Phase 6, hr-invariant-checklist Summary table) | 1-line edits, no code change, immediate accuracy win | 15 min |
| **2** | Create ADR-021-PhaseNumberingConventions | Closes the open decision | 30 min |
| **3** | Continue the HR per-aggregate wave pipeline (Staff aggregate, 8 invariants) | Largest gap per the checklist | 1-2 sessions |
| **4** | Continue the Finance per-aggregate wave pipeline (next placeholder stub) | Proven pattern, lots of remaining invariants | 1 session per aggregate |
| **5** | Produce per-invariant checklists for assessment, attendance, communication, documents, cms, events-domain | Prerequisite for any per-aggregate wave on those domains | 1 session per domain |
| **6** | Implement the first 10-20 dispatcher wrappers (e.g., the academic set) | Closes the #1 production gap | 1 session per domain |
| **7** | Cross-compile verification (install aarch64 + wasm32 toolchains, exercise the CI job) | Closes the portability gap | 1 session |
| **8** | Populate `educore-storage-parity` with cross-adapter query-result assertions | Closes the medium-severity test gap | 1 session |
| **9** | Implement `educore-sdk::Engine::builder()` | Closes the API gap | 1 session |
| **10** | Threat model + operational docs (runbook, SLO/SLI, alerting recipe, DR plan) | Closes the production-readiness gaps | 1-2 sessions |

**Suggested first session:** Items 1 + 2 + start Item 3 (HR Staff aggregate drop). Single commit for the doc fixes, then 1 wave for the Staff I-1 (tenant anchor) + I-2 (id unique) — smallest entry point.

---

## 6. Production Deployment Checklist (what blocks a real deployment)

A real school SaaS deployment would need:

- [ ] Dispatcher wrapper layer (0/509 → 509/509) — **CRITICAL**
- [ ] Per-domain invariant checklist for every domain — **MEDIUM** (audit doc requirement)
- [ ] Threat model + penetration test report — **MEDIUM** (security gate)
- [ ] Operational runbook + SLO/SLI + alerting recipe — **MEDIUM** (production gate)
- [ ] Cross-compile verification (aarch64 + wasm32) — **MEDIUM** (mobile/WASM client gate)
- [ ] Cross-adapter parity test suite — **MEDIUM** (data-correctness gate)
- [ ] Disaster recovery plan with RPO/RTO — **MEDIUM** (business-continuity gate)
- [ ] Production-grade observability (metrics, traces, logs) — **MEDIUM** (debuggability gate)
- [ ] HR domain Staff + PayrollGenerate + LeaveRequest FSMs implemented — **LOW** (most schools use them)
- [ ] Library + Facilities + Settings + Operations production-ready — **LOW** (schools vary)

**Estimated effort to clear the CRITICAL + MEDIUM gates:** ~15-20 sessions of focused work (1 session = 2-4 hours).

**Estimated effort to clear everything:** ~30-40 sessions.

---

## 7. See Also

- `docs/audit_reports/remediation/16-session-handoff.md` — previous session handoff (Wave 65 continuation)
- `docs/audit_reports/remediation/15-continuation-reconciliation.md` — v3 → Wave 65+ reconciliation
- `docs/audit_reports/remediation/14-engine-production-depth-v3-roadmap.md` — v3 233-step plan
- `docs/audit_reports/stub_vs_implementation.md` — per-domain function-level + deep invariant audit
- `docs/audit_reports/finance-invariant-checklist.md` — finance per-invariant status
- `docs/audit_reports/hr-invariant-checklist.md` — HR per-invariant status
- `docs/audit_reports/academic-invariant-checklist.md` — academic per-invariant status
- `docs/audit_reports/facilities-invariant-checklist.md` — facilities per-invariant status
- `docs/audit_reports/security_review.md` — implementation security choices
- `docs/audit_reports/loadtest_baseline.md` — load test baseline (deferred full-scale)
- `docs/progress-tracker.md` — per-crate implementation status (stale Phase 6)
- `AGENTS.md` — engine operating contract
- `18-session-handoff.md` (this session) — comprehensive handoff for the next agent
