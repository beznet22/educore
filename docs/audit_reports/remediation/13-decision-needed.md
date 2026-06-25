# Decisions Needed

**Purpose:** Items that require user input to close. Each blocks
production (Gate-6) until resolved. Linked from
[`12-production-readiness-roadmap.md`](12-production-readiness-roadmap.md).

**How to resolve:** For each item, pick an option (or propose a new
one), then create an ADR in `docs/decisions/` with the canonical
answer. After the ADR lands, the corresponding roadmap item becomes
tickable.

> **Open count:** 4 (D-4, D-5, D-6, D-7) — must close before
> production deployment.

---

## D-4 — Phase 17: real or phantom?

**Roadmap ID:** `D-4`
**Source:** `docs/build-plan.md`
**Current evidence:** Phase 17 is mentioned 6 times in `build-plan.md` AND AGENTS.md lists 18 phases (0..17).

**The conflict:** the audit report says Phase 17 is missing, but it appears to be present. Either:
- The audit is wrong (Phase 17 is documented, just check)
- Phase 17's scope differs from the audit's assumption
- The 18 phases are differently numbered

**Options:**
- [ ] **A: Phase 17 is `CMS` (Phase 12 in AGENTS.md).** No new phase needed. Update audit finding.
- [ ] **B: Phase 17 is a NEW phase for post-CMS work** (e.g., dev tools, SDK). Document its scope in `build-plan.md`.
- [ ] **C: Merge phases** (e.g., fold Phase 17 into Phase 16). Reduces total phase count.

**Recommended:** A (likely the audit was stale; Phase 17 = CMS, already implemented).

**Impact if unresolved:** docs/build-plan.md ↔ AGENTS.md version drift, audit noise.

---

## D-5 — Cross-domain ownership collisions (3 ADRs needed)

**Roadmap ID:** `D-5`
**Source:** `docs/audit_reports/findings/wave6-specs-1.md`
**Current evidence:** 3 aggregates are claimed by multiple domains.

### Collision 1: SubjectAttendance

- **academic** spec: `docs/specs/academic/aggregates.md` (probably)
- **attendance** spec: `docs/specs/attendance/aggregates.md` § SubjectAttendance (verified)

**Question:** Is `SubjectAttendance` a child of `Attendance` (owned by attendance domain) or a derived view (owned by academic)?

**Options:**
- [ ] **A: attendance owns it.** academic can read but not write.
- [ ] **B: academic owns it.** attendance publishes events that academic consumes.
- [ ] **C: New shared domain** for attendance tracking. (Major refactor.)

**Recommended:** A — attendance owns attendance data, regardless of who queries it.

### Collision 2: ExamAttendance

- **academic** spec: (likely)
- **assessment** spec: `docs/specs/assessment/aggregates.md` § ExamAttendance (probably)

**Same pattern as Collision 1.**

**Recommended:** A — attendance owns (assessment publishes `ExamScheduled` events that attendance consumes).

### Collision 3: SpeechSlider

- **cms** spec: `docs/specs/cms/aggregates.md`
- **events-domain** spec: `docs/specs/events/aggregates.md` § SpeechSlider (?)

**Question:** Is `SpeechSlider` a CMS page type or an event-domain calendar item?

**Recommended:** A — cms owns (events-domain uses calendar primitives, not slide widgets).

**Impact if unresolved:** ambiguous ownership = ambiguous repository = ambiguous migrations.

---

## D-6 — SurrealDB vs Postgres as primary backend

**Roadmap ID:** `D-6`
**Source:** `docs/audit_reports/findings/wave5-docs-1.md`
**Current evidence:**
- `AGENTS.md` (line ~330): "educore-storage-surrealdb (primary target — embedded + server modes; see ADR-017)"
- `ADR-017-SurrealDBFirst.md`: explicit SurrealDB-first decision
- `ADR-018-SyncEngine.md`: implies Postgres for sync targets
- `docs/project-overview.md`: (need to check)

**The conflict:** ADR-017 says SurrealDB-first. ADR-018 implies Postgres. These are not necessarily contradictory (SurrealDB for primary app data, Postgres for sync engine metadata), but the docs don't make this clear.

**Options:**
- [ ] **A: SurrealDB is primary (ADR-017 stands).** Postgres is ONLY for sync engine metadata. Update ADR-018 to make this explicit.
- [ ] **B: Postgres is primary (override ADR-017).** SurrealDB is the embedded/offline mode. Create ADR-019.
- [ ] **C: Both are first-class.** Adapter choice is per-deployment. Update both ADRs + AGENTS.md to remove "primary" language.

**Recommended:** A (matches current implementation; ADR-017's intent is SurrealDB for app data).

**Impact if unresolved:** confusion in deployment docs, conflict in contributor onboarding.

---

## D-7 — Public API renames

**Roadmap ID:** `D-7`
**Source:** `docs/audit_reports/findings/wave5-docs-2.md`
**Current evidence:** Identity types and event types are documented under names that differ from the code.

**Examples needed:** The audit cites specific examples — need to enumerate them before deciding.

**Process:**
1. Run `scripts/find-rename-candidates.sh` (not yet built) — scans for `<X>Identifier` vs `<X>Id` drift
2. Compile a list of canonical names per type
3. Decide: rename code → docs, OR rename docs → code
4. Create ADR-020 (or extension to existing ADR) with the canonical mapping
5. Apply renames via mechanical refactor (large PR — split per crate)

**Options:**
- [ ] **A: Keep current code names; fix docs.** Lowest churn; consumers see no change.
- [ ] **B: Rename code to match docs.** Higher churn; consumers see new names.
- [ ] **C: Pick one canonical set; update both.** Middle ground; one breaking rename.

**Recommended:** A (least disruption).

**Impact if unresolved:** docs lie; consumers copy wrong names; support burden.

---

## Decision resolution workflow

1. Pick option above (or propose new one) → comment in this file or in the matching roadmap row.
2. Lead agent creates ADR in `docs/decisions/ADR-NNN-<topic>.md`.
3. ADR includes: Context, Decision, Consequences, Alternatives considered.
4. Roadmap row becomes tickable (the check is the ADR's filename + content keywords).
5. Once all D-* items resolved → Gate-6 turns `[x]`.

---

## See also

- [`12-production-readiness-roadmap.md`](12-production-readiness-roadmap.md) — the roadmap
- [`12-roadmap-data.toml`](12-roadmap-data.toml) — data file (where these items live)
- `docs/decisions/ADR-013..018-*.md` — existing ADRs
