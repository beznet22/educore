# Phase 13 Verification Prompt (Template)

> Master template for the per-phase verify prompts in
> `docs/verification/`. To create a new per-phase prompt
> (e.g. `18-PHASE-VERIFY-PROMPT.md`), copy this file and
> replace every `13` with the new phase number, then fill in
> the **Per-Phase Preamble** at the bottom.

---

## Mission

Verify that Phase 13's forward-looking prompt
(`docs/phase_prompt/phase-13-prompt.md`), retrospective handoff
(`docs/handoff/PHASE-13-HANDOFF.md`), build-plan section
(`docs/build-plan.md` § "Phase 13"), and on-disk implementation
(`crates/<tier>/<name>/src/`) are all consistent with the
domain spec (where one exists) and the source-of-truth
priority. Auto-fix any disparities by dispatching subagents
per the 5-layer guarantees.

---

## Source-of-Truth Priority

When the 5 documents above disagree, resolve them in this
order (highest priority first):

1. `docs/specs/<domain>/*.md` — canonical for aggregates,
   commands, events, capabilities, audit targets. **N/A if
   Phase 13 has no domain spec** (adapter or tools tier).
2. `docs/build-plan.md` § "Phase 13" — canonical for what the
   phase builds (deliverables, tasks, exit criteria, risks).
3. `docs/handoff/PHASE-13-HANDOFF.md` — the closing agent's
   claim about what was actually shipped (validated against
   the on-disk implementation in priority 4).
4. The implementation in `crates/<tier>/<name>/src/` — the
   on-disk truth. Source files, tests, `Cargo.toml`
   dependencies, and the umbrella re-exports in
   `crates/educore/src/lib.rs` are the source of truth for
   "what was actually built".
5. `docs/phase_prompt/phase-13-prompt.md` — the input being
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
2. **Build-plan § "Phase 13" is complete.** The build-plan
   section for Phase 13 (between the `## Phase 13` and
   `## Phase 14` headings) contains all 5 sub-sections:
   `Deliverables.`, `Tasks.`, `Exit criteria.`, `Risks.`,
   and `Phase completion documentation.` (per the per-phase
   prompt convention in `docs/phase_prompt/README.md`).
3. **Coverage rows are `Pending`.** Every aggregate or
   feature that the phase plans to ship has a row in
   `docs/coverage.toml` with `status = "Pending"`. The
   `PRE-CHECK-PHASES-13-17.md` snapshot enumerates the
   current `Pending` count; the per-phase preamble lists
   the expected count (spec-faithful vs headline-13).
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

1. **Prompt ↔ Spec.** Every aggregate name in
   `docs/phase_prompt/phase-13-prompt.md` matches
   `docs/specs/<domain>/aggregates.md` exactly (per the
   closing-agent verification checklist in
   `docs/phase_prompt/README.md`). For non-domain phases,
   every port trait / reference impl in the prompt matches
   the corresponding `docs/ports/<port>.md` file.
2. **Prompt ↔ Build-Plan.** The "Deliverables" + "Tasks" +
   "Exit Criteria" sections in the prompt match the
   build-plan § "Phase 13" section. The headline-13
   interpretation (or spec-faithful declaration) is
   consistent between the prompt and the build-plan.
3. **Prompt ↔ Handoff.** The "Where NOT to start" rules in
   the prompt match the carry-forward rules in the handoff
   (no `educore-finance` dep, no `educore-notify` dep, no
   `educore-attendance` dep, no `educore-documents` dep, etc.,
   as applicable). The "Do NOT" list in the prompt's
   "Per-Deliverable Gotchas" matches the handoff's "Where
   NOT to start" section word-for-word (modulo phase-specific
   additions).
4. **Handoff ↔ Implementation.** The headline correctness
   check claimed in the handoff (e.g. the 100-case proptest
   in `crates/domains/<name>/src/services.rs`) exists and
   is green. The aggregates, command shapes, events, and
   `Capability` / `AuditTarget` variants listed in the
   handoff's "What's wired and working" section are present
   in the on-disk source files. The "Open questions" in
   the handoff are either resolved in the implementation
   or explicitly carried forward (with a citation in the
   per-phase preamble of the next phase's verify prompt).
5. **Coverage Matrix ↔ Implementation.** Every `Tested`
   row in `docs/coverage.toml` for Phase 13 has a real
   implementation in the source tree (not a stub returning
   `Err(not_supported)`). Every `Pending` row has either
   a real implementation that should be flipped to `Tested`
   (then flip it as part of the auto-fix) or an explicit
   "deferred to Phase 13+M" rationale in the handoff.

---

## Auto-Fix Rules (per dimension)

The verify agent dispatches one subagent per failing
dimension, with file-level ownership and section-level
pre-allocation per the 5-layer guarantees. The
subagent-scope mapping is:

| Failing dimension | Subagent scope | Files owned |
| --- | --- | --- |
| 1. Prompt ↔ Spec | `fix-prompt-spec` | `docs/phase_prompt/phase-13-prompt.md` |
| 2. Prompt ↔ Build-Plan | `fix-prompt-buildplan` | `docs/phase_prompt/phase-13-prompt.md`, `docs/build-plan.md` § "Phase 13" |
| 3. Prompt ↔ Handoff | `fix-prompt-handoff` | `docs/phase_prompt/phase-13-prompt.md`, `docs/handoff/PHASE-13-HANDOFF.md` |
| 4. Handoff ↔ Implementation | `fix-handoff-impl` | `crates/<tier>/<name>/src/**`, `crates/cross-cutting/{rbac,audit}/src/**` (if prereq 2A/2B is missing) |
| 5. Coverage Matrix ↔ Implementation | `fix-coverage` | `docs/coverage.toml`, `crates/<tier>/<name>/src/**` (only for stub-flips) |

Multiple dimensions can run in parallel if they own disjoint
files. If two dimensions want to edit the same file (e.g.
dimension 1 + dimension 3 both touch
`docs/phase_prompt/phase-13-prompt.md`), the prep subagent
pre-allocates the file with section markers per the
5-layer guarantees; each subagent's edits stay inside its
assigned section.

The auto-fix subagent produces exactly one atomic commit
per the "Atomic commits per microtask" guarantee. The commit
message is `Phase 13 verify: <dimension> (<workstream>)` with
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
2. **Section-level pre-allocation.** For files that must
   be touched by multiple workstreams (e.g. `aggregate.rs` for
   3+ root aggregates, or `phase-13-prompt.md` for two
   failing dimensions), the prep subagent pre-creates the
   file with named section markers
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
   (`docs/handoff/PHASE-13-VERIFY-REPORT.md`) → `R3
   final-validation` (9-command gate). A verify step does
   not advance to the next stage until the prior stage's
   gate passes.
4. **Atomic commits per microtask.** Every subagent produces
   exactly one commit with a
   `Phase 13 verify: <scope> (<workstream>)` message +
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

Write `docs/handoff/PHASE-13-VERIFY-REPORT.md` with these
five sections:

- **Section A — Pre-Implementation Check results.**
  `Pass` / `Fail` per item, with a one-line citation to the
  file + line range.
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

- [ ] `docs/handoff/PHASE-13-VERIFY-REPORT.md` exists with all
  5 sections (A, B, C, D, E) populated.
- [ ] All Section A items pass (for unimplemented phases
  13-17) or all Section B dimensions pass (for implemented
  phases 0-11 and 12).
- [ ] All listed disparities fixed (or explicitly deferred
  with a rationale + an ADR reference in Section C).
- [ ] One atomic commit with the fixes (per the
  "Atomic commits per microtask" guarantee).
- [ ] `cargo test -p <crate>` green (for implemented phases;
  the crate name comes from the per-phase preamble).
- [ ] `cargo build --workspace` green.
- [ ] `cargo run -p educore-core --bin lint --features lint`
  green (the no-gaps gate per `AGENTS.md`).
- [ ] `docs/progress-tracker.md` row for Phase 13 updated
  (status reflects the verified close).

---

## Per-Phase Preamble — Phase 13 (Events / calendar)

**Phase title:** Events (calendar domain)

**Status:** **UNIMPLEMENTED** — the scaffold `educore-events-domain` exists in `crates/cross-cutting/events-domain/` but the implementation is not yet started.

**Build-plan section:** `docs/build-plan.md` lines 1400–1435 (per `PRE-CHECK-PHASES-13-17.md`; between `## Phase 13 — Events domain (calendar)` and `## Phase 14 — Settings + Operations` headings).

**Spec:** `docs/specs/events/` (11 files) — `overview.md`, `aggregates.md`, `commands.md`, `entities.md`, `events.md`, `permissions.md`, `repositories.md`, `services.md`, `tables.md`, `value-objects.md`, `workflows.md`. Per the pre-check, `aggregates.md` defines **7 root aggregates**: `CalendarEvent`, `Holiday`, `Weekend`, `Incident`, `AssignIncident`, `IncidentComment`, `CalendarSetting` — the build-plan's "4 aggregates" headline (`CalendarEvent`, `Holiday`, `Incident`, `Weekend`) is a subset, not the full set.

**Spec aggregate count:** 7 root aggregates (per `docs/specs/events/aggregates.md` headings: `## CalendarEvent` line 3, `## Holiday` line 52, `## Weekend` line 85, `## Incident` line 123, `## AssignIncident` line 162, `## IncidentComment` line 196, `## CalendarSetting` line 225). Spec-faithful count; the build-plan's "4 aggregates" headline is incomplete.

**Handoff:** `docs/handoff/PHASE-13-HANDOFF.md` — **DOES NOT EXIST YET** (phase is unimplemented). To be created in the phase-13 closing-agent commit per `docs/build-plan.md` § Phase 13 task 4.

**Implementation crate:** `crates/cross-cutting/events-domain/` (`educore-events-domain`) — per the pre-check. Scaffold state: `Cargo.toml` (19 lines) declares deps on `educore-core`, `educore-platform`, `educore-rbac`, `educore-events`, `educore-settings`; `src/lib.rs` is 27 lines (standard `PACKAGE_NAME`/`PACKAGE_VERSION` scaffold).

**Coverage rows in `docs/coverage.toml` for `phase = 13` or `crate = "educore-events-domain"`:**
- `events_calendar_events_aggregate` — `status = "Pending"` (spec: `docs/specs/events/aggregates.md`; crate: `educore-events-domain`)

**Spec-faithful target count for the closing task:** 7 root aggregates ⇒ expect 7 `events_<aggregate>_aggregate` rows in `docs/coverage.toml` after Phase 13 closes. Per prior-phase precedent (Phase 9 = 10 rows for 6 aggregates, Phase 10 = 13 rows for 26 aggregates, Phase 11 = 3 rows for 3 aggregates), the verify prompt's closing task is to flag the ~6 missing rows (`events_holidays_aggregate`, `events_weekends_aggregate`, `events_incidents_aggregate`, `events_assign_incidents_aggregate`, `events_incident_comments_aggregate`, `events_calendar_settings_aggregate`).

**Section A: Pre-Implementation Check (active now):**
- [ ] Spec exists: `docs/specs/events/` has 11 files (verified by V-pre)
- [ ] Build-plan § "Phase 13" is complete: lines 1400–1435 contain all 5 sub-sections (`Deliverables.`, `Tasks.`, `Exit criteria.`, `Risks.`, `Phase completion documentation.`) — verified
- [ ] Coverage rows exist with `status = "Pending"`: 1 row (`events_calendar_events_aggregate`); **gap**: 6 more expected
- [ ] Scaffold crate exists: `crates/cross-cutting/events-domain/Cargo.toml` (19 lines) + `src/lib.rs` (27 lines) — verified

**Section B: Post-Implementation Check (when Phase 13 closes):**
- The 7 root aggregates vs the spec's commands/events (CalendarEvent has 3 commands / 3 events; Holiday has 3/3; Weekend has 4/4; Incident has 4/4; AssignIncident has 3/3; IncidentComment has 2/2; CalendarSetting has 5/5)
- The 1 Pending coverage row (`events_calendar_events_aggregate`) flipped to `Tested`; 6 new rows added per the spec-faithful target
- The 4 build-plan headline aggregates (CalendarEvent, Holiday, Incident, Weekend) are a subset of the 7 spec aggregates; verify the implementation covers all 7
- `crates/cross-cutting/events-domain/Cargo.toml` declares the standard cross-cutting deps: `educore-core`, `educore-platform`, `educore-rbac`, `educore-events`, `educore-academic` (cross-crate dep; **MUST be added**), `educore-settings` (forward-dep; see Pre-Implementation Gaps)
- `Cargo.toml` does **NOT** declare `educore-finance`, `educore-notify`, or `educore-attendance` (per the carry-forward rules)
- The RRULE integration test (RFC 5545 subset) exists in `crates/cross-cutting/events-domain/tests/rrule_weekly_with_holiday.rs` (or similar) and is green

**Pre-implementation gaps found in PRE-CHECK-PHASES-13-17.md (relevant to Phase 13):**
- **Coverage row undercount:** 1 row (`events_calendar_events_aggregate`) for 7 spec aggregates. The verify prompt's closing task is to flag the ~6 missing rows (`events_holidays_aggregate`, `events_weekends_aggregate`, `events_incidents_aggregate`, `events_assign_incidents_aggregate`, `events_incident_comments_aggregate`, `events_calendar_settings_aggregate`).
- **Build-plan / spec headline mismatch:** build-plan § Phase 13 line 1404 says "4 aggregates" (`CalendarEvent`, `Holiday`, `Incident`, `Weekend`); spec `aggregates.md` lists 7 (adds `AssignIncident`, `IncidentComment`, `CalendarSetting`). The verify prompt documents this. Per the prior-phase precedent (Phase 8 = 11 aggregates, Phase 9 = 6, Phase 10 = 26, Phase 11 = 3), the spec-faithful view wins. The closing task is to either (a) add 3 aggregates to the build-plan § "Phase 13" headline, or (b) add a "(spec-faithful: 7 aggregates)" annotation.
- **`educore-academic` dep missing from `events-domain/Cargo.toml`:** The spec's `overview.md` `## Dependencies` section lists `educore-academic` for `ClassId`/`SectionId`/`SubjectId` audience references. The current scaffold has no `educore-academic` dep. The closing task is to add the dep.
- **`educore-settings` forward-dep risk:** The `events-domain/Cargo.toml` declares `educore-settings` as a dep, but `educore-settings` is a Phase 14 deliverable. This is a forward-reference risk: Phase 13 alone will not compile. The closing task is to either (a) drop the `educore-settings` dep until Phase 14 lands, or (b) keep it and accept the circular build-order risk.
- **Tier placement:** `educore-events-domain` lives in `crates/cross-cutting/` not `crates/domains/`. Per `AGENTS.md` § "Tier System" the `events-domain` crate is a domain bounded context, not a cross-cutting concern; the verify prompt's closing task is to confirm whether the events-domain should be moved to `crates/domains/events-domain/` (a new directory) or whether the cross-cutting placement is correct (the spec lists it as a domain, not a cross-cutting foundation; the build-plan headline says "Events domain (calendar)").
- **Two `events` crates naming risk (per `AGENTS.md` § "Note on `educore-events` vs `educore-events-domain`" and build-plan § "Phase 13" Risks):** `crates/cross-cutting/events/` is the envelope (Phase 2, closed); `crates/cross-cutting/events-domain/` is the calendar (Phase 13, unimplemented). The closing task is to verify the `lib.rs` header of each crate makes the distinction explicit.
- **Stale `progress-tracker.md` row:** the `docs/progress-tracker.md` row for Phase 13 should exist as "Pending" but is not yet validated.

**Specific verification focus (for Section A only, the active section):**
- The 7 spec aggregates vs the 1 Pending coverage row (matrix gap)
- The `educore-academic` dep gap (closing task)
- The `educore-settings` forward-dep (closing task)
- The tier placement (`cross-cutting/` vs `domains/`)
- The spec's 11 files: all present and well-formed
- The build-plan § "Phase 13" Tasks: complete

**Known carry-forward rules relevant to this phase:** None (Phase 13 is the first phase without prior OQ carry-forwards; it's the first phase after the in-progress Phase 12).

**Build-plan headline vs spec-faithful precedent:** Per Phases 8-11, the spec-faithful view wins when the build-plan headline and the spec aggregate list diverge. The verify prompt's Section B dimension 2 (Prompt ↔ Build-Plan) confirms the prompt is consistent with the spec (7 aggregates, not 4). The 4-aggregate headline in the build-plan is treated as a headline subset, not a ceiling.

**Section B (when Phase 13 closes):**
- The 7 root aggregates vs the ~24 commands (3+3+4+4+3+2+5) vs the ~25 events (3+3+4+4+3+2+5+1) — verify all are wired in `commands.rs` and `events.rs` per the macro pattern
- The `CalendarEvent` aggregate (the largest by far; recurring-event computation via RRULE service)
- The `Holiday` aggregate (per-school calendar; `HolidayCreated` consumed by attendance)
- The `Weekend` aggregate (per-school weekend configuration; `WeekendConfigured` batch command)
- The `Incident` aggregate (the 4-state machine: `Open` → `InProgress` → `Resolved` → `Archived`; the `Resolved` state freezes mutations per invariant 5)
- The `AssignIncident` aggregate (links `Incident` to `ClassId`/`SectionId`/`SubjectId` from the academic domain — the cross-crate dep on `educore-academic` is the reason)
- The `IncidentComment` aggregate (append-only; `IncidentCommented` event)
- The `CalendarSetting` aggregate (categorical label for the calendar UI; `EnableCalendarSetting` / `DisableCalendarSetting` commands)
- The `educore-events-domain` `Cargo.toml` deps: `educore-core`, `educore-platform`, `educore-rbac`, `educore-events`, `educore-academic` (cross-crate dep; **MUST be added** in the closing task)
- The `educore-events-domain` `Cargo.toml` does **NOT** declare `educore-finance` (Phase 8 OQ #6 + Phase 10 OQ #3 + Phase 11 OQ #3 carry-forward), `educore-notify` (Phase 10 OQ #4 + Phase 11 OQ #4 carry-forward), `educore-attendance` (Phase 10 OQ #5 + Phase 11 OQ #5 carry-forward), or `educore-documents` — verify the closing commit does not add any of these
- The umbrella `crates/educore/src/lib.rs` re-exports `educore_events_domain` as `educore::events_domain` (per the re-export pattern in `AGENTS.md` § "Naming Convention (Enforced)")
