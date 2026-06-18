# Phase 6 Verification Prompt (Template)

> Master template for the per-phase verify prompts in
> `docs/verification/`. To create a new per-phase prompt
> (e.g. `18-PHASE-VERIFY-PROMPT.md`), copy this file and
> replace every `6` with the new phase number, then fill in
> the **Per-Phase Preamble** at the bottom.

---

## Mission

Verify that Phase 6's forward-looking prompt
(`docs/phase_prompt/phase-6-prompt.md`), retrospective handoff
(`docs/handoff/PHASE-6-HANDOFF.md`), build-plan section
(`docs/build-plan.md` § "Phase 6"), and on-disk implementation
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
   Phase 6 has no domain spec** (adapter or tools tier).
2. `docs/build-plan.md` § "Phase 6" — canonical for what the
   phase builds (deliverables, tasks, exit criteria, risks).
3. `docs/handoff/PHASE-6-HANDOFF.md` — the closing agent's
   claim about what was actually shipped (validated against
   the on-disk implementation in priority 4).
4. The implementation in `crates/<tier>/<name>/src/` — the
   on-disk truth. Source files, tests, `Cargo.toml`
   dependencies, and the umbrella re-exports in
   `crates/educore/src/lib.rs` are the source of truth for
   "what was actually built".
5. `docs/phase_prompt/phase-6-prompt.md` — the input being
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
2. **Build-plan § "Phase 6" is complete.** The build-plan
   section for Phase 6 (between the `## Phase 6` and
   `## Phase 7` headings) contains all 5 sub-sections:
   `Deliverables.`, `Tasks.`, `Exit criteria.`, `Risks.`,
   and `Phase completion documentation.` (per the per-phase
   prompt convention in `docs/phase_prompt/README.md`).
3. **Coverage rows are `Pending`.** Every aggregate or
   feature that the phase plans to ship has a row in
   `docs/coverage.toml` with `status = "Pending"`. The
   `PRE-CHECK-PHASES-13-17.md` snapshot enumerates the
   current `Pending` count; the per-phase preamble lists
   the expected count (spec-faithful vs headline-6).
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
   `docs/phase_prompt/phase-6-prompt.md` matches
   `docs/specs/<domain>/aggregates.md` exactly (per the
   closing-agent verification checklist in
   `docs/phase_prompt/README.md`). For non-domain phases,
   every port trait / reference impl in the prompt matches
   the corresponding `docs/ports/<port>.md` file.
2. **Prompt ↔ Build-Plan.** The "Deliverables" + "Tasks" +
   "Exit Criteria" sections in the prompt match the
   build-plan § "Phase 6" section. The headline-6
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
   row in `docs/coverage.toml` for Phase 6 has a real
   implementation in the source tree (not a stub returning
   `Err(not_supported)`). Every `Pending` row has either
   a real implementation that should be flipped to `Tested`
   (then flip it as part of the auto-fix) or an explicit
   "deferred to Phase 6+M" rationale in the handoff.

---

## Auto-Fix Rules (per dimension)

The verify agent dispatches one subagent per failing
dimension, with file-level ownership and section-level
pre-allocation per the 5-layer guarantees. The
subagent-scope mapping is:

| Failing dimension | Subagent scope | Files owned |
| --- | --- | --- |
| 1. Prompt ↔ Spec | `fix-prompt-spec` | `docs/phase_prompt/phase-6-prompt.md` |
| 2. Prompt ↔ Build-Plan | `fix-prompt-buildplan` | `docs/phase_prompt/phase-6-prompt.md`, `docs/build-plan.md` § "Phase 6" |
| 3. Prompt ↔ Handoff | `fix-prompt-handoff` | `docs/phase_prompt/phase-6-prompt.md`, `docs/handoff/PHASE-6-HANDOFF.md` |
| 4. Handoff ↔ Implementation | `fix-handoff-impl` | `crates/<tier>/<name>/src/**`, `crates/cross-cutting/{rbac,audit}/src/**` (if prereq 2A/2B is missing) |
| 5. Coverage Matrix ↔ Implementation | `fix-coverage` | `docs/coverage.toml`, `crates/<tier>/<name>/src/**` (only for stub-flips) |

Multiple dimensions can run in parallel if they own disjoint
files. If two dimensions want to edit the same file (e.g.
dimension 1 + dimension 3 both touch
`docs/phase_prompt/phase-6-prompt.md`), the prep subagent
pre-allocates the file with section markers per the
5-layer guarantees; each subagent's edits stay inside its
assigned section.

The auto-fix subagent produces exactly one atomic commit
per the "Atomic commits per microtask" guarantee. The commit
message is `Phase 6 verify: <dimension> (<workstream>)` with
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
2. **Section-level pre-allocation.** For files that must be
   touched by multiple workstreams (e.g. `aggregate.rs` for
   3+ root aggregates, or `phase-6-prompt.md` for two
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
   (`docs/handoff/PHASE-6-VERIFY-REPORT.md`) → `R3
   final-validation` (9-command gate). A verify step does
   not advance to the next stage until the prior stage's
   gate passes.
4. **Atomic commits per microtask.** Every subagent produces
   exactly one commit with a
   `Phase 6 verify: <scope> (<workstream>)` message +
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

Write `docs/handoff/PHASE-6-VERIFY-REPORT.md` with these
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

- [ ] `docs/handoff/PHASE-6-VERIFY-REPORT.md` exists with all
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
- [ ] `docs/progress-tracker.md` row for Phase 6 updated
  (status reflects the verified close).

---

## Per-Phase Preamble

> Copy this section to the bottom of the per-phase prompt
> and fill in the bracketed fields. For Phases 14-17 (the
> unimplemented phases at the time this directory was
> created), use `PRE-CHECK-PHASES-13-17.md` as the source
> for the spec path, scaffold line counts, coverage row
> undercount, and carry-forward rules.

**Phase 6 — <Title>**

- **Spec:** `<path to docs/specs/<domain>/>` (or "no spec"
  for adapter / tools tier phases; reference the port
  contract or guide doc instead).
- **Handoff:** `docs/handoff/PHASE-6-HANDOFF.md`.
- **Build-plan section:** `docs/build-plan.md` lines
  <start>-<end> (between the `## Phase 6` and `## Phase 7`
  headings).
- **Implementation crate(s):** `crates/<tier>/<name>/`
  (one entry per crate the phase ships).
- **Coverage row IDs:** <list the `Pending` / `Tested` row
  IDs from `docs/coverage.toml` for this phase, with the
  spec-faithful target count if Section A applies>.
- **Carry-forward rules:** <paste the "Where NOT to start"
  + "Open questions" from the prior handoff, plus the
  phase-specific rules from the build-plan § "Phase 6"
  "Risks." sub-section>.

---

# Per-Phase Preamble — Phase 6 (HR)

**Phase title:** HR

**Status:** Implemented (per `docs/handoff/PHASE-6-HANDOFF.md`)

**Build-plan section:** `docs/build-plan.md` lines 837–875

**Spec:** `docs/specs/hr/` (11 files)

**Spec aggregate count:** 16 root aggregates per `docs/specs/hr/aggregates.md` (Staff, Department, Designation, LeaveType, LeaveDefine, LeaveRequest, StaffAttendance, StaffAttendanceImport, AssignClassTeacher, HourlyRate, SalaryTemplate, PayrollGenerate, PayrollEarnDeduc, LeaveDeductionInfo, StaffRegistrationField, StaffImportBulkTemporary)

**Handoff:** `docs/handoff/PHASE-6-HANDOFF.md`

**Implementation crate:** `crates/domains/hr/` (`educore-hr`)

**Coverage rows in `docs/coverage.toml` for `phase = 6` or `crate = "educore-hr"`:**
- `hr_staff_aggregate` — `status = "Tested"` (spec: `docs/specs/hr/aggregates.md#staff`)
- `hr_department_aggregate` — `status = "Tested"` (spec: `docs/specs/hr/aggregates.md#department`)
- `hr_designation_aggregate` — `status = "Tested"` (spec: `docs/specs/hr/aggregates.md#designation`)
- `hr_leave_type_aggregate` — `status = "Tested"` (spec: `docs/specs/hr/aggregates.md#leavetype`)
- `hr_leave_define_aggregate` — `status = "Tested"` (spec: `docs/specs/hr/aggregates.md#leavedefine`)
- `hr_leave_request_aggregate` — `status = "Tested"` (spec: `docs/specs/hr/aggregates.md#leaeverequest`)
- `hr_staff_attendance_aggregate` — `status = "Tested"` (spec: `docs/specs/hr/aggregates.md#staffattendance`)
- `hr_staff_attendance_import_aggregate` — `status = "Tested"` (spec: `docs/specs/hr/aggregates.md#staffattendanceimport`)
- `hr_assign_class_teacher_aggregate` — `status = "Tested"` (spec: `docs/specs/hr/aggregates.md#assignclassteacher`)
- `hr_hourly_rate_aggregate` — `status = "Tested"` (spec: `docs/specs/hr/aggregates.md#hourlyrate`)
- `hr_salary_template_aggregate` — `status = "Tested"` (spec: `docs/specs/hr/aggregates.md#salarytemplate`)
- `hr_payroll_generate_aggregate` — `status = "Tested"` (spec: `docs/specs/hr/aggregates.md#payrollgenerate`)
- `hr_payroll_earn_deduc_aggregate` — `status = "Tested"` (spec: `docs/specs/hr/aggregates.md#payrollearndeduc`)
- `hr_leave_deduction_info_aggregate` — `status = "Tested"` (spec: `docs/specs/hr/aggregates.md#leavedeductioninfo`)
- `hr_staff_registration_field_aggregate` — `status = "Tested"` (spec: `docs/specs/hr/aggregates.md#staffregistrationfield`)
- `hr_staff_import_bulk_temporary_aggregate` — `status = "Tested"` (spec: `docs/specs/hr/aggregates.md#staffimportbulktemporary`)
- `hr_payrolls_aggregate` — `status = "Tested"` (spec: `docs/specs/hr/aggregates.md#payrollgenerate`)
- `staff_registered_event` — `status = "Tested"` (spec: `docs/specs/hr/events.md#staff-registered`)
- `staff_deleted_event` — `status = "Tested"` (spec: `docs/specs/hr/events.md#staff-deleted`)
- `department_created_event` — `status = "Tested"` (spec: `docs/specs/hr/events.md#department-created`)
- `designation_created_event` — `status = "Tested"` (spec: `docs/specs/hr/events.md#designation-created`)
- `leave_type_created_event` — `status = "Tested"` (spec: `docs/specs/hr/events.md#leave-type-created`)
- `leave_policy_defined_event` — `status = "Tested"` (spec: `docs/specs/hr/events.md#leave-policy-defined`)
- `leave_requested_event` — `status = "Tested"` (spec: `docs/specs/hr/events.md#leave-requested`)
- `leave_approved_event` — `status = "Tested"` (spec: `docs/specs/hr/events.md#leave-approved`)
- `staff_attendance_marked_event` — `status = "Tested"` (spec: `docs/specs/hr/events.md#staff-attendance-marked`)
- `class_teacher_assigned_event` — `status = "Tested"` (spec: `docs/specs/hr/events.md#class-teacher-assigned`)
- `hourly_rate_set_event` — `status = "Tested"` (spec: `docs/specs/hr/events.md#hourly-rate-set`)
- `salary_template_created_event` — `status = "Tested"` (spec: `docs/specs/hr/events.md#salary-template-created`)
- `payroll_generated_event` — `status = "Tested"` (spec: `docs/specs/hr/events.md#payroll-generated`)
- `payroll_approved_event` — `status = "Tested"` (spec: `docs/specs/hr/events.md#payroll-approved`)
- `payroll_paid_event` — `status = "Tested"` (spec: `docs/specs/hr/events.md#payroll-paid`)
- `staff_bulk_imported_event` — `status = "Tested"` (spec: `docs/specs/hr/events.md#staff-bulk-imported`)

(33 rows total: 17 `*_aggregate` + 16 `*_event`. Note the handoff claims 30 rows were flipped; the discrepancy is that 3 rows were already `Tested` at Phase 5 close-out. Reconcile the count in the verify report.)

**Known carry-forward rules relevant to this phase:**
- The handoff claims Phase 6 was a complete delivery (per Subagent 2's survey).
- The commit `8789659` is titled `chore(workspace): commit Phase 6/8/9 WIP as foundation for Phase 10` — this is misleading; the handoff is the source of truth (the work is complete).

**Pre-implementation gaps found in PRE-CHECK-PHASES-13-17.md (if any for this phase):** N/A.

**Known secondary-doc gaps (from the Subagent 2 survey of Phase 11 close-out):**
- `docs/progress-tracker.md` row 75 says Phase 6 HR is "Planned" with "No" exit criteria — **stale** (the handoff says it's done).
- `docs/build-plan.md` is missing `**Phase 6 outcome.**` section (the other closed phases have one).

**Specific verification focus:**
- The 16 root aggregates vs the 50+ events (per handoff)
- The 33 coverage Tested rows (must match the 16 aggregates — note the count mismatch: 33 > 16, suggesting some Pending rows should be split or merged; verify prompt handles this)
- The 4 prereq + 7 workstream commits
- The `Staff` aggregate (the largest; 4-level payroll computation pipeline)
- The `Payroll` aggregate
- The `LeaveRequest` state machine: Pending → Approved/Rejected/Cancelled
- The `LeaveDeductionInfo` aggregate (the spec-vs-implementation divergence: there is a separate `LeaveDeductionInfo` for actual deduction math, separate from `PayrollEarnDeduc`)
- The `educore-hr` `Cargo.toml` deps: `educore-core`, `educore-platform`, `educore-rbac`, `educore-events`, `educore-events-domain`, `educore-academic` (cross-domain dep)
- The misleading commit message `8789659` (note in the verify report that the title says "WIP" but the handoff claims complete)
