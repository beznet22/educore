# Cluster C — Spec ↔ Code Drift

**Root cause:** The 10 domain crates implement 20-50% of the spec'd
aggregates, commands, events, and repositories. Spec files
(`docs/specs/<domain>/`) document the full surface; the crates ship a
fraction. The drift is structural — there is no automated check.

**Estimated findings:** ~600 (largest cluster; ~32% of all findings)

**Source ID prefixes:** `DOMAIN-*`, `DOM-*`, `SPEC-*`

**Blocks deploy:** Yes (partial — for any domain the consumer depends on,
this is a blocker; the deploy-blocker threshold is per-domain).

**Estimated fix scope:** XL. 10 domains × ~5 spec files per domain ×
~50% gap-fill each = a multi-quarter effort.

## Why these findings cluster

The engine's coverage contract is: every spec'd aggregate has a Rust
struct, every spec'd command has a handler, every spec'd event has an
event struct, every spec'd table has a `#[derive(DomainQuery)]`
application. The audit measured the actual coverage:

| Domain | Spec aggregates | Code aggregates | Spec commands | Code commands | Spec events | Code events |
|---|---|---|---|---|---|---|
| academic | 20+ | 5 | 32+ | 22 | 30+ | partial |
| assessment | 46+ | 6 | 40+ | 21 | 50+ | partial |
| attendance | 10 | 5 | 13 | 15 | 25+ | 16 |
| cms | 19 | 19 (drift) | 40+ | 10 | 50+ | partial |
| communication | 27 | 25 | 40+ | 72 | 30+ | partial |
| documents | 20 | partial | 30+ | 11 | 25+ | partial |
| events-domain | 7 | 7 | 24 | 24 | 24 | 24 (✓) |
| facilities | 16+ | partial | 30+ | partial | 25+ | partial |
| finance | 51+ | 10 | 60+ | 30 | 50+ | partial |
| hr | 40+ | partial | 715 lines of spec | 269 lines of code | 30+ | partial |
| library | 9 tables / 4 roots | 4 (orphan tables) | 25+ | partial | 20+ | partial |

The single fully-implemented domain is `events-domain` (Phase 13, the most
recently-closed phase). Every other domain has a spec↔code gap.

## Representative findings (sample)

| Source | ID | Sev | Domain | One-line |
|---|---|---|---|---|
| `wave1-academic.md` | DOM-AC-001 | C | academic | Spec lists 20+ aggregates; code has 5 |
| `wave1-academic.md` | DOM-AC-005 | C | academic | Spec lists 30+ events; only partial emitted |
| `wave1-assessment.md` | DOM-ASS-001 | C | assessment | 46 spec'd aggregates; 6 in code |
| `wave1-assessment.md` | DOM-ASS-005 | C | assessment | 40+ spec'd commands; 21 handlers |
| `wave1-attendance.md` | DOM-ATT-001 | C | attendance | 10 spec aggregates; 5 in code |
| `wave1-cms.md` | DOM-CMS-001 | C | cms | 19 aggregates OK; 40+ commands; only 10 handlers |
| `wave1-communication.md` | DOM-COM-001 | C | communication | 27 aggregates; 25 in code (ChatStatus renamed) |
| `wave1-communication.md` | DOM-COM-005 | C | communication | Spec-only events not in `events.rs` |
| `wave1-finance.md` | DOM-FIN-001 | C | finance | 51 spec aggregates; 10 in code |
| `wave1-finance.md` | DOM-FIN-005 | C | finance | Spec events without corresponding struct |
| `wave1-hr-library.md` | DOM-HRLIB-001 | C | hr | 715 lines of commands in spec; 269 in code |
| `wave1-hr-library.md` | DOM-HRLIB-005 | C | library | 9 tables but only 4 aggregate roots |
| `wave6-specs-1.md` | SPEC-1-001 | C | academic/assessment/... | Zero `#[derive(DomainQuery)]` applications |
| `wave6-specs-2.md` | SPEC-2-001 | C | documents/finance/hr | Zero `#[derive(DomainQuery)]` applications |

## What fixing this requires

**Per-domain gap fill**

For each domain `<d>`:

1. Open `docs/specs/<d>/aggregates.md`. List every aggregate not in
   `crates/domains/<d>/src/aggregate.rs`.
2. Open `docs/specs/<d>/commands.md`. List every command not in
   `crates/domains/<d>/src/commands.rs`.
3. Open `docs/specs/<d>/events.md`. List every event not in
   `crates/domains/<d>/src/events.rs`.
4. Open `docs/specs/<d>/value-objects.md`. List every value object not
   in `crates/domains/<d>/src/value_objects.rs`.
5. Open `docs/specs/<d>/repositories.md`. List every method not on the
   repository trait in `crates/domains/<d>/src/repository.rs`.
6. For each missing item, decide: ship now, defer to a follow-up phase,
   or mark as deprecated (delete from spec).

**Apply `#[derive(DomainQuery)]`**

Per `docs/build-plan.md:172`, every aggregate struct should have
`#[derive(DomainQuery)]`. Currently 0/310 across all domains. This is
the entry point for cluster A.

**Fix naming drift**

Per `wave6-specs-1.md` and `wave6-specs-2.md`, the audit found:

- Spec uses `StudentId`, code uses `StudentIdentifier`.
- Spec table `student`, code uses `students`.
- Spec event `Student.Admitted`, code uses `student.admitted`.
- Cross-domain ownership collisions (SubjectAttendance, ExamAttendance,
  SpeechSlider claimed by two specs each).

Each domain needs a one-time naming pass to align spec ↔ code.

**Cross-domain ownership resolution**

Per the spec-vs-spec collisions, the engineering team needs to decide:

- Which domain owns `SubjectAttendance` — academic or attendance?
- Which domain owns `ExamAttendance` — academic or assessment?
- Which domain owns `SpeechSlider` — cms or events-domain?

Document each decision in an ADR.

## Suggested fix sequence

This cluster is too large for a single sweep. Recommended order:

1. **events-domain** — already complete. Use as the template for the
   other 9 domains.
2. **cms** — closest to completion (19/19 aggregates, only commands +
   events partial). Per `PHASE-12-HANDOFF.md`, this was the most
   recently completed before events-domain.
3. **attendance** — 10 spec, 5 code; smallest gap. Quick wins.
4. **academic** — 20+ spec, 5 code; largest spec, biggest gap. Likely
   takes multiple sub-phases.
5. **assessment** — 46 spec, 6 code. Largest gap.
6. **finance** — 51 spec, 10 code. Complex (cross-domain with hr).
7. **communication** — 27 spec, 25 code; closest to done.
8. **documents** — partial; medium scope.
9. **facilities** — partial; medium scope.
10. **hr + library** — combined per the audit; complex (cross-domain
    with finance and academic).
11. **Naming drift sweep** — once per domain's gap fill is done, do a
    naming pass to align spec ↔ code.
12. **Cross-domain ownership ADR** — resolve the 3 known collisions.

## Verification criteria

- `cargo test -p <domain>` passes with `tests/workflows.rs`,
  `tests/commands.rs`, `tests/events.rs` all present.
- `docs/coverage.toml` has a row per aggregate/command/event with
  `status = "Tested"` (per `docs/build-plan.md` § "The No-Gaps Gates").
- `cargo run -p educore-core --bin lint --features lint` reports zero
  spec↔code drift for the fixed domain.
- `graphify-out/wiki/index.md` (if it exists) shows the domain's full
  surface.

## Risk if left unfixed

- Any consumer of the affected domain will encounter gaps at runtime
  (missing commands, missing events, missing repositories).
- The coverage matrix lies (`docs/coverage.toml` claims Tested; reality
  is partial).
- The audit-found spec drift will propagate: new contributors will add
  code matching the partial state, not the spec.

## Cross-cluster dependencies

- **Unblocks:** Most of cluster F (per-adapter gaps often stem from
  per-domain gaps), Cluster G (doc drift often reflects code drift).
- **Depends on:** Cluster A (macro AST must work before
  `#[derive(DomainQuery)]` can be applied), Cluster B (workflows must
  work end-to-end before `tests/workflows.rs` can pass).

## Files involved

- `crates/domains/<domain>/src/{aggregate,commands,events,value_objects,repository}.rs` (10 domains × 5 files = 50 files)
- `crates/domains/<domain>/tests/{workflows,commands,events,services,repository}.rs` (10 × 5 = 50 new test files)
- `docs/specs/<domain>/{aggregates,entities,commands,events,value-objects,repositories,workflows}.md` (11 spec files per domain × 10 = 110 files)
- `docs/coverage.toml` (the matrix that tracks this)
