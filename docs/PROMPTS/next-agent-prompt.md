# Next Agent Prompt — Educore Engine Continuation

> **Copy from here ↓ to a new agent session. Do not modify the prompt before pasting.**
> **Audience:** Any agent picking up Educore engine work after the Wave 169 audit session.
> **Ground truth:** `docs/audit_reports/remediation/18-session-handoff.md` (operational handoff) and `docs/audit_reports/remediation/17-final-reconciliation-audit.md` (four-corner gap analysis). Read both BEFORE making any change.

---

## Your task

You are continuing work on the **Educore** engine — a Rust workspace of 37 packages (1 umbrella + 36 internal crates) implementing a multi-tenant school SaaS platform per `AGENTS.md` and 21 ADRs. The previous session ended at commit `55972c8` on `main`, all pushed to `origin/main`.

### Authoritative context (read these first, in order)

1. **`docs/audit_reports/remediation/18-session-handoff.md`** — operational handoff: repo identity, 20 ADRs, completed work (102 waves), remaining work by priority, blockers + assumptions, modified files, unresolved conflicts, production risks, recommended execution order, exact commands to run first, per-aggregate wave template, commit attribution, engine rules.
2. **`docs/audit_reports/remediation/17-final-reconciliation-audit.md`** — comprehensive gap analysis: four-corner reconciliation (Production ↔ Specs ↔ Documentation ↔ Codebase), 12-category gap breakdown, 10-session remediation roadmap, production deployment checklist.
3. **`AGENTS.md`** — the engine operating contract (naming conventions, tier system, engine rules, validation checklist).
4. **`docs/progress-tracker.md`** — per-crate implementation status (note: Phase 6 was just corrected; verify before trusting other entries).
5. **`docs/audit_reports/hr-invariant-checklist.md`** — HR per-invariant status (the next primary focus).

**Treat these docs as authoritative context, NOT as gospel.** Verify every recommendation against the current repository before making changes — never assume the docs are correct if the code has evolved, and never assume the code is correct if it violates an intentional specification.

### Working methodology (this is the hard requirement)

Throughout the session, **continuously reconcile Production ↔ Specifications ↔ Documentation ↔ Codebase**. Whenever you discover drift, determine the correct source of truth through engineering judgment and:

- **update the code** to match the intended architecture, OR
- **update the documentation/specification** to reflect validated production reality, OR
- **update both** when the architecture has legitimately evolved.

**Do not defer obvious documentation fixes.** If you find a stale doc, an outdated ADR, a wrong invariant tally, a missing entry — fix it in the same session, in the same commit if possible, or in a clearly-named follow-up commit.

**Leave the repository in a strictly better state than you found it.** Before ending the session, update the handoff, progress tracker, audit reports, findings, checklists, and remediation documents with only factual repository state, ensuring the next agent can continue with zero prior context.

**Focus on eliminating production-readiness gaps, not just increasing feature completeness.** A 10th `Real*` aggregate does not help if the dispatcher wrapper layer is still at 0/509 (it is). Read §3.1 of `17-final-reconciliation-audit.md` before deciding what to work on.

### Scope (what to do this session)

Per the prioritized TODO in `18-session-handoff.md` § 5:

1. **Verify clean state** (5 min) — confirm HEAD is at `55972c8`, `git status` is clean (only harness untracked), `cargo check -p educore-hr --tests` is 0 errors, HR tests pass.
2. **Continue the HR per-aggregate wave pipeline** (primary focus). Start with the highest-count aggregate that has the lowest [x] count: **Staff** (8 invariants, 0 `[x]`). Target the I-1 entry first (Tenant anchor from SchoolId) — it's the smallest entry point and unlocks the cross-references from DepartmentHead, DesignationGrade, LeaveRequest, etc.
3. **Interleave with Finance remaining placeholder stubs** if time allows (parallel HR work).
4. **Update the docs as you go.** Every checklist flip must include full file:line evidence. Every new invariant enforcement must be reflected in the master checklist, the progress tracker, and (if a new pattern emerges) the audit report.
5. **End the session with an updated handoff.** Either update `18-session-handoff.md` to reflect your session's work, or create `19-session-handoff.md` as a delta doc. The next agent must be able to start with zero prior context.

### Out of scope (do NOT work on these this session)

- **0/509 dispatcher wrappers** — deferred to post-wave-pipeline sessions (per §3.1 of the audit doc). Do NOT start implementing wrappers this session; the wave pipeline must catch up first or you'll be wiring handlers against aggregates that don't exist yet.
- **Cross-compile verification** (aarch64 + wasm32) — requires toolchain install; defer to a dedicated session.
- **`educore-storage-parity` population** — defer to a dedicated session.
- **`educore-sdk::Engine::builder()`** — defer.
- **Threat model + operational docs** — defer.

### Per-aggregate wave template (proven across 102 waves)

When you start a new per-aggregate wave (e.g., HR Staff I-1), follow the template in `18-session-handoff.md` § 12:

1. Read the spec: `cat docs/specs/<domain>/aggregates.md | sed -n '/^## <Aggregate>/,/^## /p'`
2. Read the existing aggregate: `head -<N> crates/domains/<domain>/src/aggregate.rs`
3. Add the validation helper to `crates/domains/<domain>/src/value_objects.rs` (if not present)
4. Update `Real<Aggregate>::fresh()` to enforce the invariant — return `DomainError::Validation` on failure
5. Update `Real<Aggregate>::update_*()` methods to re-validate (defense-in-depth)
6. Extend `Create<Aggregate>Command` in `commands.rs` if new fields needed
7. Extend the `Created` event in `events.rs` if new fields needed
8. Update the service function in `services.rs`
9. Add re-exports to `lib.rs::prelude` (single-shot edit)
10. Write behavioral tests in `tests/<aggregate>.rs` — model after the closest existing test file
11. Add `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::dbg_macro, missing_docs)]` at the top of the test file
12. Run `cargo check -p <package> --tests && cargo test -p <package> --tests --no-fail-fast`
13. Flip the checklist entry from `[ ]` to `[x]` with full file:line evidence
14. Commit + push with the Co-Authored-By trailer

### Commit attribution (mandatory)

Every commit must end with:
```text
Co-Authored-By: Antigravity <antigravity@google.com>
```

Git identity for AI agents:
```bash
git -c user.name="Educore Dev" -c user.email="dev@educore.local" commit -m "..."
```

Stage files explicitly by name (no `git add -A`). Branch is `main`. Push to `origin main` after every commit. Never force-push.

### Engine rules you must NEVER violate

Per `AGENTS.md` § Engine Rules + § Type Safety:

1. No `unwrap()` or `expect()` in production paths — use `?` or document the invariant
2. No `#[allow(dead_code)]` or `_var` prefixes to silence the compiler — delete unused code
3. No `as` casts that truncate or lose data — use `TryFrom`/`TryInto`
4. No `serde_json::Value` in domain code — use typed wrappers
5. No `HashMap<String, T>` for domain data — use typed structs
6. No service locators, DI containers, runtime reflection
7. No `unsafe` in domain code (`#![forbid(unsafe_code)]`)
8. No `native-tls` — only `rustls`
9. No `tokio` directly in domain code — only through `educore-core` re-exports
10. No glob imports in domain code

Test files may use file-level `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::dbg_macro, missing_docs)]`. Production code must use per-function `#[allow(...)]` only when justified.

### Naming conventions (enforced)

- **Brand in prose:** Educore
- **Code:** `educore` (umbrella), `educore-<name>` (package), `educore_<name>` (Rust extern crate id)
- **Directories:** `crates/<tier>/<name>/` (drop the `educore-` prefix)
- **Legacy names:** Schoolify, InfixEdu are FORBIDDEN in new code (removed from `docs/specs/` in 77 files; git history retains them as audit trail)
- **No new deps without explicit instruction + ADR update**

### Loop-guard rules (learned the hard way)

When you find yourself repeating the same edit-and-run cycle without progress:

1. **STOP.** State in one sentence what is failing and why.
2. **List 2+ alternative approaches you haven't tried.**
3. **Pick the most promising one and try THAT.** Don't repeat the failed approach.

When you find yourself reading/updating files via bash (cat/head/tail/sed), **use the dedicated tool instead**:

- File reads → `read` with offset/limit
- File edits → `edit` with surgical anchors
- File writes → `write` (only for new files or full rewrites)
- Content search → `grep` (respects `.gitignore`)
- File find → `find` or `ls`

When you encounter multi-edit bundles that silently drop entries, **switch to single-shot `edit` calls**. The `edit` tool's multi-edit mode is unreliable in this workspace — the proven pattern is one `edit` call per change.

---

## First 5 commands to run

```bash
cd /home/beznet/Workspace/smscore

# 1. Verify clean state (should end with 55972c8)
git log --oneline -5
git status

# 2. Verify the HR crate builds + tests pass
cargo check -p educore-hr --tests 2>&1 | tail -3
cargo test -p educore-hr --tests --no-fail-fast 2>&1 | tail -3

# 3. Re-read the authoritative context (don't skip this — your model has no prior knowledge)
cat docs/audit_reports/remediation/18-session-handoff.md | head -100
cat docs/audit_reports/remediation/17-final-reconciliation-audit.md | head -80

# 4. Read the HR Staff spec to plan your first wave
cat docs/specs/hr/aggregates.md | sed -n '/^## Staff/,/^## /p' | head -100

# 5. Find the existing Staff aggregate in code
grep -n "pub struct Staff\|impl RealStaff\|impl Staff" crates/domains/hr/src/aggregate.rs | head -5
```

**After verifying state, start with the per-aggregate wave template on HR Staff I-1 (Tenant anchor).** That single wave will take 15–45 minutes and will surface every pattern you'll need for the next 100 HR waves.

---

## Reminder of what the prior session identified

- **7 HR invariants are `[x]`** (all from Wave 32: Staff I-4, PayrollGenerate I-2/I-5, LeaveRequest I-1/I-2/I-4, LeaveDefine I-3, HourlyRate I-1).
- **100 HR invariants are `[ ]`** — your backlog.
- **~30 finance placeholder stubs remain** — your secondary backlog.
- **0/509 dispatcher wrappers** — the #1 production gap, deferred to post-wave sessions.
- **2 stale docs were just fixed** (progress-tracker Phase 6, HR checklist Summary) — verify they're still correct.
- **ADR-021 was just created** — verify it's referenced from `AGENTS.md` (it may need a follow-up to add Phases 13–17 to the Crate Inventory table).

**Don't trust any of the above without verifying.** That's the methodology.

---

## When you finish

Before ending the session, leave the repo in a strictly better state:

1. **All work committed + pushed** to `origin/main`.
2. **`git status` clean** (only harness state untracked: `.kimchi/`, session HTML).
3. **`cargo check --workspace` clean** (0 errors; warnings OK if pre-existing).
4. **`cargo test --workspace --no-fail-fast` green** (or document any failing tests with engineering rationale).
5. **Updated handoff doc** — either amend `18-session-handoff.md` or create `19-session-handoff.md` with: completed work, remaining work, prioritized TODOs, blockers + assumptions, architectural decisions made, modified files, unresolved conflicts, production risks, recommended next execution order, exact first commands.
6. **Updated progress tracker + invariant checklists** — every invariant you promoted to `[x]` must be reflected in `docs/audit_reports/<domain>-invariant-checklist.md` with full file:line evidence.
7. **No leftover TODO comments in code** (unless explicitly requested by the user).
8. **No dropped multi-edit entries** — verify all your prelude/lib.rs edits actually landed.

The repo is yours to improve. Make it strictly better than you found it.
