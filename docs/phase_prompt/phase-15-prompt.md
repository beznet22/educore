# Phase 15 Prompt — Port Adapters

**Mission:** Deliver the 5 port-adapter crates. **Implementation**, not design. **Spec-faithful** interpretation per `docs/specs/{authentication,notifications,payments,file-storage,integrations}/`. Each crate ships the port trait + one reference impl + an integration test.

**Deliverables:** `crates/adapters/auth/` + `crates/adapters/notify/` + `crates/adapters/payment/` + `crates/adapters/files/` + `crates/adapters/integrations/` with port trait + reference impl per crate, 3-4 service factory functions per crate, net-new `Capability` variants in `educore-rbac`, net-new `AuditTarget` variants in `educore-audit`, vertical-slice test per crate mirroring the Phase 14 pattern (5+2 scenarios). Flip `coverage.toml` rows. Write `PHASE-15-HANDOFF.md` + `phase-16-prompt.md`.

**Required Reading:**
- `docs/handoff/PHASE-14-HANDOFF.md` (11 OQs carry over; OQ #5 notify port, OQ #10 OAuth/PasswordReset are most material for Phase 15)
- `docs/build-plan.md` § "Phase 15" + § "Phase 14 outcome."
- `docs/ports/{authentication,notifications,payments,file-storage,integrations}.md` (the 5 port contracts)
- `docs/specs/operations/overview.md` § "Infrastructure Tables" (OAuth/PasswordReset/migration tables are port-driven)
- `docs/schemas/{event,tenancy}-schema.md`
- `docs/decisions/ADR-013-CrateLayout.md`, `ADR-015-ExternalCrates.md`
- `crates/cross-cutting/settings/src/` (the most recent 9-file template, just shipped in Phase 14)
- `crates/tools/storage-parity/tests/settings_integration.rs` (the mature 5+2 scenario pattern)
- `AGENTS.md`, `docs_guidlines/system.md`, `docs_guidlines/execution_guidlines.md`
- `docs/phase_prompt/README.md` (the canonical prompt template + the closing-agent verification checklist)

**Starting Point:** 23 closed crates (10 cross-cutting + 10 domain + storage-parity + 2 new Phase 14 + umbrella) are the foundation. `educore-settings` is the most recent 9-file template. The 4 port-driven repository port traits in `educore-operations/src/repository.rs` (`OAuthAccessTokenRepository`, `OAuthClientRepository`, `PasswordResetRepository`, `MigrationRepository`) are the starting points for `educore-auth`. The `Operations.Audit.Record` capability (system-tenant) is the starting point for the `educore-notify` port.

**Working With Subagents:** Workstreams: A=`educore-auth` (port + JwtAuthProvider + OAuth tables); B=`educore-notify` (port + email + SMS impls); C=`educore-payment` (port + Stripe impl); D=`educore-files` (port + S3 + local impls); E=`educore-integrations` (port + LMS + video-conferencing impls); F=reconcile cross-crate placeholders + integration tests + coverage flips + handoff docs.

**Per-Deliverable Gotchas:**
- Five new crates in adapters tier. Stick to the port-trait + reference-impl pattern.
- **The 2 Phase 2 settings/operations capability placeholders** (`SettingsManage`/`OperationsManage`) were REMOVED in Phase 14. The 2 AuditTarget placeholders (`SchoolSettings`/`BellSchedule`) were PRESERVED. Do NOT touch either.
- Do NOT add `educore-finance` dep (Phase 8 OQ #6 carry-over through Phase 14 OQ #4).
- Do NOT add `educore-attendance` dep (Phase 10 OQ #5 carry-over through Phase 14 OQ #6).
- Do NOT add `educore-documents` dep (Phase 11 OQ #6 carry-over through Phase 14 OQ #7).
- Do NOT add `educore-academic` dep (Phase 13 OQ #7 carry-over through Phase 14 OQ #8).
- **The OAuth + PasswordReset + migrations tables** (per `docs/specs/operations/overview.md` § "Infrastructure Tables") are PORT-DRIVEN — `educore-auth` implements them as reference impls. The 4 port traits in `educore-operations/src/repository.rs` are the contracts.

**Exit Criteria:** 5 crates shipped; 3-4 service factory functions + service structs per crate; ≥ 5 `coverage.toml` rows flipped; integration tests green per crate on SQLite (always) + PG + MySQL (env-gated); `cargo test/clippy/fmt/lint --workspace` green; `PHASE-15-HANDOFF.md` + `phase-16-prompt.md` + `progress-tracker.md` + `build-plan.md § "Phase 15 outcome."`.

**When You Are Stuck:** `PHASE-14-HANDOFF.md` is the foundation. `cargo run -p educore-core --bin lint --features lint` is the no-gaps gate. The 9-file template is `crates/cross-cutting/settings/src/`. The 5+2 integration test pattern is `crates/tools/storage-parity/tests/settings_integration.rs`. The closing-agent verification checklist is in `docs/phase_prompt/README.md`.

**Subagent Orchestration:** To prevent duplicate work, every phase must enforce: (1) **File-level ownership** — every file in the owned crate is assigned to exactly one subagent. (2) **Section-level pre-allocation** — for files touched by multiple workstreams (e.g. `repository.rs` for multiple port traits), the prep subagent pre-creates named section markers. (3) **Sequential phase gates** — `P0 prep` → `R1 reconcile-prep` → `wave 1/2/3/4/5` parallel workstreams → `R2 reconcile-impl` → `6-tests` → `7-docs` → `R3 final-validation` (9-command gate). (4) **Atomic commits per microtask** — every subagent produces exactly one commit with `Phase 15: <scope> (<workstream>)` message + `Co-Authored-By: Antigravity <antigravity@google.com>` trailer. (5) **Reconciler subagents read-only** — they verify section boundaries + duplicate detection + stub-replacement but never write code.
