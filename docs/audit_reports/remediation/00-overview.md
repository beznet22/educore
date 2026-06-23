# Remediation Overview — Triage Summary

**Source:** 1,878 findings across 46 audit files (`docs/audit_reports/findings/`)
**Date:** 2026-06-23

## Severity distribution (full audit)

| Severity | Count | % | Definition |
|---|---|---|---|
| **Critical** | 473 | 25.2% | blocks deploy |
| **High** | 666 | 35.5% | major gap / feature unusable |
| **Medium** | 548 | 29.2% | minor broken |
| **Low** | 191 | 10.2% | cosmetic |
| **TOTAL** | 1,878 | 100% | |

## Cluster breakdown

| Cluster | Est. findings | Severity mix | Fix scope |
|---|---|---|---|
| A — DDL emission gap | ~150 | Critical-heavy | Large (touches macro + 4 adapters) |
| B — Workflow infrastructure | ~80 | Critical-heavy | Medium (bus + outbox + subscribers) |
| C — Spec ↔ code drift | ~600 | Mixed | **XL** (10 domains × 11 spec files × 3 layers each) |
| D — Foundation crate gaps | ~70 | Mixed | Medium (1 crate, but unlocks everything) |
| E — Engine-rule violations | ~400 | Medium-heavy | Large but mechanical (sed-style sweeps) |
| F — Adapter port-contract gaps | ~250 | Critical-heavy | Large (4 adapters × full port surface) |
| G — Doc / version drift | ~215 | Low-Medium | Medium (text-only, ~15 doc files) |

## Top 25 file hotspots (most Critical findings per file)

| Count | File | Cluster |
|---|---|---|
| 41 | `crates/domains/assessment/src/services.rs` | C + E |
| 30 | `crates/domains/finance/src/services.rs` | C + E |
| 28 | `crates/adapters/notify/src/email.rs` | F |
| 26 | `docs/library-docs.md` | G |
| 26 | `crates/infra/query-derive/src/lib.rs` | D + A |
| 26 | `crates/cross-cutting/operations/src/aggregate.rs` | C + E |
| 21 | `crates/domains/finance/src/aggregate.rs` | C |
| 19 | `docs/build-plan.md` | G |
| 18 | `crates/cross-cutting/events-domain/src/aggregate.rs` | C |
| 18 | `crates/adapters/notify/src/sms.rs` | F |
| 17 | `crates/domains/communication/src/services.rs` | C + E |
| 17 | `crates/domains/academic/src/services.rs` | C + E |
| 17 | `crates/adapters/integrations/src/lms.rs` | F |
| 16 | `docs/guides/saas-backend.md` | G |
| 16 | `crates/domains/attendance/src/services.rs` | C + E |
| 16 | `crates/domains/attendance/src/events.rs` | C |
| 16 | `crates/cross-cutting/events/src/event_bus.rs` | B + D |
| 16 | `crates/adapters/event-bus/src/in_process.rs` | B |
| 14 | `docs/query_layer.md` | G |
| 14 | `crates/domains/assessment/src/events.rs` | C |
| 14 | `crates/domains/assessment/src/aggregate.rs` | C |
| 13 | `docs/specs/sync/overview.md` | C |
| 13 | `crates/educore/src/lib.rs` | D + E |
| 13 | `crates/domains/finance/src/commands.rs` | C |
| 13 | `crates/cross-cutting/audit/src/writer.rs` | D + E |

## Top 10 most-impactful findings (cross-cutting blast radius)

These are findings that, if unfixed, cause many downstream findings to be
unfixable. Listed by source file, ID, and severity.

| Source | ID | Sev | One-line |
|---|---|---|---|
| `wave4-storage-port.md` | PORT-STORE-001 | C | `StorageAdapter::migrate()` named `create_schema()` in every consumer doc |
| `wave4-storage-port.md` | PORT-STORE-002 | C | `Transaction` carries no `TenantContext`; sub-ports drift |
| `wave4-storage-port.md` | PORT-STORE-005 | C | `Repository<A>` is generic instead of named per-aggregate |
| `wave4-storage-port.md` | PORT-STORE-008 | C | `EventLogFilter` has no cursor — millions of rows cannot paginate |
| `wave4-storage-port.md` | PORT-STORE-013 | C | Audit log not in same transaction as aggregate mutation |
| `wave4-query-derive.md` | INFRA-QD-001 | C | Macro scaffolded but zero `#[derive(DomainQuery)]` applications |
| `wave4-core.md` | CORE-001 | C | `educore-core::lint` declared at line 7 but not implemented |
| `wave4-core.md` | CORE-002 | C | `EntityDescriptor` AST fields incomplete (no cursor, no joins) |
| `wave7-workflows.md` | WF-001 | C | Zero `tests/workflows.rs` files exist in any domain |
| `wave7-workflows.md` | WF-002 | C | Zero cross-domain subscribers wired to the bus |

## Recommended first fix sequence

Sequencing is informed by `08-dependency-graph.md`. The principle: fix the
highest-leverage root cause first so it mechanically resolves the most
downstream findings.

| Step | Target | Cluster | Effort | What it unlocks |
|---|---|---|---|---|
| 1 | `educore-core::lint` impl | D | M | Auto-detect every spec↔code drift and engine-rule violation |
| 2 | `#[derive(DomainQuery)]` macro complete | A + D | L | Adapter DDL emission becomes possible |
| 3 | `StorageAdapter::create_schema()` per adapter | A | L | All 4 adapters can emit ~310 domain tables |
| 4 | Transaction port: add `TenantContext` + atomic audit | D + F | M | Multi-tenant safety restored; tests can be authoritative |
| 5 | Outbox relay impl + event bus port completion | B | M | Cross-domain workflows can be exercised end-to-end |
| 6 | `tests/workflows.rs` per domain | C | L | Spec-mandated integration test gate satisfied |
| 7 | Aggregate / command / event gap-fill per domain | C | **XL** | Per-domain coverage |
| 8 | Engine-rule sweep (replace `unwrap`, `as`, etc.) | E | L | Code-standards compliance |
| 9 | Adapter port-contract gap-fill | F | L | Adapter completeness |
| 10 | Doc/version drift fixes | G | M | Stale claims removed; crate counts correct |

Steps 1-5 are foundation. Step 6 unblocks CI gates. Steps 7-10 are bulk work.

## Risks of starting in the wrong place

| If you start with... | You will... |
|---|---|
| Cluster C (per-domain gaps) | Spend weeks fixing domains only to discover the macro can't emit them, the lint can't check them, and the bus can't wire them. |
| Cluster E (engine-rule sweep) | Touch every file in the workspace to remove `unwrap` calls, only to have rewriters reintroduce them when the actual handlers land. |
| Cluster F (per-adapter gaps) | Reimplement the same port contract 4 times if the port itself isn't fixed first. |
| Cluster G (doc fixes) | Write true statements that become false again as the underlying code is fixed. |

## Verification of remediation progress

The audit's coverage gates (per `docs/build-plan.md` § "The No-Gaps Gates" line 1825)
are the canonical mechanism for tracking remediation. After each cluster is
fixed, the relevant gate should turn green:

| Gate | What it checks | File / command |
|---|---|---|
| Per-domain integration tests | `crates/domains/<d>/tests/` exists and passes | `cargo test -p <domain>` |
| Cross-reference lint | Spec↔code parity, anti-patterns, matrix sync | `cargo run -p educore-core --bin lint --features lint` |
| Coverage matrix CI | `docs/coverage.toml` rows match reality | `git diff --exit-code docs/coverage.toml` |
| Graph regen freshness | `graphify-out/` is up to date with source | `graphify update .` |

When **all four gates are green for all crates**, the audit can be re-run and
the Critical count should drop to zero. Until then, residual Critical
findings remain deploy-blockers.

## Open questions for the engineering team

1. Should `educore-core::lint` be promoted from scaffold-stub to a CI gate
   *before* the per-domain gaps are filled? (Recommended: yes — it forces
   the macro AST to be spec-aligned before any of the 310 domain tables
   ship.)
2. Should Cluster E (engine rules) be addressed incrementally per-cluster
   rather than as a single sweep? (Recommended: yes — coupling engine-rule
   fixes to their natural owner-cluster avoids merge churn.)
3. Should cluster G (doc drift) be deferred to the post-fix phase, when
   the docs can be written against the fixed reality? (Recommended: yes —
   many doc findings will become moot after clusters A-F land.)
4. Is the per-domain `tests/workflows.rs` gate (build-plan line 1860) still
   the right test surface, or should it move to `crates/tools/storage-parity`?
   (Open question — affects cluster B/C sequencing.)
