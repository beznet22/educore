# ADR-021: Phase Numbering Conventions

## Status

Accepted, 2026-08-02.

## Context

The repository has two slightly different phase-numbering conventions
that have caused repeated confusion across multiple sessions (most
recently flagged in `docs/audit_reports/remediation/13-decision-needed.md`
decision D-4 and again in `docs/audit_reports/remediation/15-continuation-reconciliation.md`
§ 3.2 "The 'Phase 17 = phantom?' decision was resolved but not ADR'd"):

1. **`build-plan.md` § "The 18 phases"** counts phases as Phase 0
   through Phase 17 (18 entries), where:
   - Phase 17 = "Production readiness" (the final hardening phase,
     documented in `build-plan.md` as "the actual target" — multi-tenant
     suite, load test, cross-compile, security review, docs audit).

2. **`AGENTS.md` § "Crate Inventory"** counts phases as Phase 0
   through Phase 12 (13 entries — Phase 0..12 inclusive = 13), where:
   - Phase 12 = `educore-cms` (the CMS domain crate)
   - No explicit Phase 17 entry; the table implicitly assumes Phases
     13–17 ship additional cross-cutting + tools crates after CMS.

The two conventions disagree on:
- **What Phase 17 is.** `build-plan.md` says "production hardening",
  `AGENTS.md` says "doesn't exist in this numbering."
- **How many phases total.** `build-plan.md` says 18 (0..17),
  `AGENTS.md` says 13 (0..12).
- **What to call the CMS phase.** `build-plan.md` would call CMS
  "Phase 12" (the 13th entry); `AGENTS.md` calls it "Phase 12" too,
  but the surrounding context frames Phase 12 as one of 13, not 18.

This is a **numbering-convention mismatch**, not a substance
disagreement. The same work happens regardless of whether we say
"Phase 17" or "Phase 12 + post-CMS phases 13–17."

## Decision

The engine follows the **`build-plan.md` convention** as the canonical
phase numbering. This convention:

1. **Counts phases as 0..17 inclusive (18 phases total).**
2. **Treats Phase 17 as the production-readiness hardening phase**
   (the actual deployment target — multi-tenant suite, load test,
   cross-compile, security review, docs audit).
3. **Maps the phase to crates per `AGENTS.md` § "Crate Inventory"** —
   `AGENTS.md` is the authoritative crate-to-phase mapping for
   Phases 0..12 (the domain crates), and `build-plan.md` adds
   Phases 13..17 as the cross-cutting + tools + hardening work that
   follows CMS.

### The mapping table

| Phase | `build-plan.md` Title | Crates Shipped (per `AGENTS.md` § "Crate Inventory") |
|---|---|---|
| 0 | Foundation | `core`, `query-derive`, `storage`, `storage-surrealdb`, `sync`, `sync-inprocess`, `storage-parity` (scaffold) |
| 1 | Adapter parity | `storage-postgres`, `storage-mysql`, `storage-sqlite` |
| 2 | Cross-cutting foundations | `platform`, `rbac`, `events`, `event-bus`, `audit` |
| 3 | Academic | `academic` |
| 4 | Assessment | `assessment` |
| 5 | Attendance | `attendance` |
| 6 | HR | `hr` |
| 7 | Finance | `finance` |
| 8 | Facilities | `facilities` |
| 9 | Library | `library` |
| 10 | Communication | `communication` |
| 11 | Documents | `documents` |
| 12 | CMS | `cms` |
| 13 | Events domain (calendar) | `events-domain` |
| 14 | Settings + Operations | `settings`, `operations` |
| 15 | Port adapters | `auth`, `notify`, `payment`, `files`, `integrations` |
| 16 | Test infrastructure + SDK | `testkit`, `storage-parity` (full suite), `sdk`, `cli` |
| 17 | Production readiness | (no new crates; hardening: multi-tenant suite, load test, cross-compile, security review, docs audit) |

### Cross-references

- **`AGENTS.md` § "Crate Inventory"** lists 13 numbered rows
  (Phase 0..12), each annotated with the phase title + the crate(s)
  shipped. This is the authoritative mapping for Phases 0..12.
  Phases 13..17 are not in the `AGENTS.md` table because `AGENTS.md`
  predates their definition; future revisions of `AGENTS.md` should
  add them.

- **`progress-tracker.md` § "Phase Progress"** lists all 18 phases
  (Phase 0..17) and is the authoritative tracker for whether a
  phase's exit criteria are met.

- **`build-plan.md` § "The 18 phases"** is the authoritative
  phase-title + scope definition.

## Consequences

### Positive

- **Single source of truth.** Future agents reading any of the three
  documents will see consistent numbering.
- **CMS is unambiguously Phase 12.** No more "Phase 17 = phantom?"
  questions.
- **Phase 17 has a clear scope.** It's the production-readiness
  hardening phase, not an undefined bucket.

### Negative

- **Phases 13..17 are not in `AGENTS.md`'s Crate Inventory table.**
  Future revisions of `AGENTS.md` should add rows for `events-domain`
  (Phase 13), `settings` + `operations` (Phase 14), `auth` +
  `notify` + `payment` + `files` + `integrations` (Phase 15),
  `testkit` + `storage-parity` + `sdk` + `cli` (Phase 16), and a
  row for Phase 17 noting "(no new crates; hardening)."
  This is a documentation update, not a code change.

### Neutral

- **No code changes required.** This ADR resolves the numbering
  convention only; it does not change the engine rules, the tier
  system, the dependency direction, or any other architectural
  decision.

## See also

- `build-plan.md` § "The 18 phases" — the canonical phase scope
- `AGENTS.md` § "Crate Inventory" — the canonical crate-to-phase
  mapping for Phases 0..12
- `progress-tracker.md` § "Phase Progress" — the canonical tracker
  of whether a phase's exit criteria are met
- `docs/audit_reports/remediation/13-decision-needed.md` — the
  open-decision doc where D-4 was resolved
- `docs/audit_reports/remediation/15-continuation-reconciliation.md`
  § 3.2 — the reconciliation note that surfaced this ADR need
