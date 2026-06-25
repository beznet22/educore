# Decisions Needed

**Purpose:** Items that require user input to close. Each blocks
production (Gate-6) until resolved. Linked from
[`12-production-readiness-roadmap.md`](12-production-readiness-roadmap.md).

**How to resolve:** For each item, pick an option (or propose a new
one), then create an ADR in `docs/decisions/` with the canonical
answer. After the ADR lands, the corresponding roadmap item becomes
tickable.

> **Open count:** 8 (D-4, D-5, D-6, D-7, D-8, D-9, D-10, D-11) — must close
> before production deployment.

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

## D-8 — ADR-014 IdempotencyConflict / IdempotencyPending error variants

**Roadmap ID:** `ADR-014-IDEM-CONFLICT-VARIANT`, `FND-CORE-001`
**Source:** `docs/decisions/ADR-014-Idempotency.md` § Decision 4,9 + `docs/audit_reports/findings/wave4-core.md` CORE-003

**The conflict:** ADR-014 explicitly mandates two `DomainError` variants:
- `IdempotencyConflict { key, existing_outcome_ref }` — retry with same key but different payload
- `IdempotencyPending { key, started_at }` — retry during async run

Current code uses generic `Conflict(String)`. The audit report's remediation claim at line 156 says the variants exist, but they do not.

**Options:**
- [ ] **A: Add both variants** (per ADR-014). Consumers must migrate.
- [ ] **B: Keep generic `Conflict(String)`** and amend ADR-014 to remove the variant mandate. Reduces breaking changes for existing callers.
- [ ] **C: Add as `DomainError::Conflict(IdempotencyOutcome)`** — wrap inside the existing `Conflict` variant. Backward compatible.

**Recommended:** A (matches ADR-014 explicit text; consumers are still few since the dispatcher is forward-looking).

**Impact if unresolved:** callers cannot programmatically distinguish retry-conflict from generic conflict; tests will be brittle.

---

## D-9 — Canonical crate count (33 / 36 / 37)

**Roadmap ID:** `ADR-013-COUNTS-DRIFT`
**Source:** `docs/decisions/ADR-013-CrateLayout.md` drift note; `AGENTS.md` line 24; `docs/architecture.md` tier table

**The conflict:** Three numbers in three places.
- ADR-013 original: 3+7+10+9+4 = 33 internal + 1 = 34
- ADR-013 drift note: 3+9+10+10+4 = 36 internal + 1 = 37
- AGENTS.md: "36 internal crates"
- Actual filesystem: 3+9+10+10+4 = 36 internal + 1 = 37

**Options:**
- [ ] **A: Adopt 37** (3 infra + 9 cross-cutting + 10 domains + 10 adapters + 4 tools + 1 umbrella). Update AGENTS.md + ADR-013 reconciliation + architecture.md.
- [ ] **B: Adopt 36** (drop `sync` or `sync-inprocess` from cross-cutting count). Depends on D-11 decision.
- [ ] **C: Re-tally the inventory** before deciding. New crate may have been added recently.

**Recommended:** A — matches filesystem, document as canonical.

**Impact if unresolved:** docs disagree on counts; cross-references break.

---

## D-10 — Sync feature flag (ADR-018 § 4)

**Roadmap ID:** `ADR-018-SYNC-FEATURE-FLAG`
**Source:** `docs/decisions/ADR-018-SyncEngine.md` § 4 + FINDING 27

**The conflict:** ADR-018 § 4 requires `[features] default = []; sync = ["educore-sync", "educore-sync-inprocess"]` in `crates/educore/Cargo.toml` so server-only consumers don't pay the sync dependency cost. Current umbrella has unconditional deps.

**Options:**
- [ ] **A: Add the `sync` feature flag** (per ADR-018). `cargo build -p educore` (default) excludes sync; `cargo build -p educore --features sync` includes it.
- [ ] **B: Make sync always-on** and amend ADR-018. Acceptable if sync cost is small.
- [ ] **C: Split into two umbrella crates** (`educore` core + `educore-sync` extension).

**Recommended:** A (matches ADR-018 explicit text).

**Impact if unresolved:** server deployments pay unnecessary dependency cost; wasm builds may break.

---

## D-11 — Sync-inprocess tier location

**Roadmap ID:** `ADR-018-SYNC-INPROCESS-TIER`
**Source:** `docs/decisions/ADR-018-SyncEngine.md` § 3 + ADR-013 § Tier System

**The conflict:** ADR-018 § 3 lists `crates/adapters/sync-inprocess/` but the actual location is `crates/cross-cutting/sync-inprocess/`. ADR-013 says adapters live at tier 3; cross-cutting at tier 1. The in-process adapter is a reference impl that arguably fits both.

**Options:**
- [ ] **A: Move to `crates/adapters/sync-inprocess/`** (per ADR-018). Tier 3 alignment.
- [ ] **B: Amend ADR-018 to accept cross-cutting location** (since it's a reference impl, not a port impl). Tier 1 alignment.
- [ ] **C: Keep as-is** and amend both ADRs with a reconciliation note.

**Recommended:** B (lowest churn; the crate is a reference impl for tests).

**Impact if unresolved:** docs disagree on tier; future contributors confused.

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
