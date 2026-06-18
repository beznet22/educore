# Phase 13 Prompt — Events Domain (Calendar)

**Mission:** Deliver `educore-events-domain` — `CalendarEvent`, `Holiday`, `Incident`, `Weekend`. **Implementation**, not design. **Spec-faithful** interpretation per `docs/specs/events/aggregates.md`. The Calendar domain is **distinct** from `educore-events` (the envelope crate from Phase 2 — `EventEnvelope`, `EventBus`, `DomainEvent`). Per the prompt + `AGENTS.md` clarification, the Calendar crate is at `crates/cross-cutting/events-domain/` (NOT `crates/domains/events-domain/`).

**Deliverables:** `crates/cross-cutting/events-domain/` with 9-file layout per `AGENTS.md`, 4 root aggregates + child entities, ~12 typed events, ~12 typed commands, 4 repository port traits, 4 query stubs, 2-3 service factory functions + service structs, 8-12 net-new `Capability::Events*` variants (4 Phase 2 `EventsCalendar{Create,Read,Update,Delete}` placeholders retained), 4 net-new `AuditTarget` variants, `educore-rbac` round-trip test + `educore-audit` round-trip test, RRULE subset implementation for the calendar, vertical-slice test mirroring `cms_integration.rs` (5 named scenarios + 2 env-gated). Flip `coverage.toml` rows. Write `PHASE-13-HANDOFF.md` + `phase-14-prompt.md`.

**Required Reading:**
- `docs/handoff/PHASE-12-HANDOFF.md` (9 OQs carry over; Q1 `SchoolId::PUBLIC`, Q3 `SpeechSlider` dual ownership, Q4 service-factory scope are most material for Phase 13)
- `docs/build-plan.md` § "Phase 13" + § "Phase 12 outcome." (note § Risks: the two `events` crates are easy to confuse — `crates/cross-cutting/events/` is the envelope; `crates/cross-cutting/events-domain/` is the calendar)
- `docs/specs/events/` (all 11 files)
- `docs/ports/{event-bus,storage}.md`, `docs/schemas/{tenancy,audit}-schema.md`
- `docs/decisions/ADR-013-CrateLayout.md`, `ADR-015-ExternalCrates.md`
- `crates/cross-cutting/events/src/{lib,domain_event,envelope}.rs`, `crates/cross-cutting/rbac/src/services.rs`, `crates/cross-cutting/audit/src/writer.rs`
- `crates/domains/cms/src/` (the most recent 9-file template, just shipped in Phase 12)
- `crates/tools/storage-parity/tests/cms_integration.rs` (the mature 7-scenario pattern)
- `AGENTS.md`, `docs_guidlines/system.md`, `docs_guidlines/execution_guidlines.md`
- `docs/phase_prompt/README.md` (the canonical prompt template + the closing-agent verification checklist)

**Starting Point:** 20 closed crates (10 cross-cutting + 9 domain + storage-parity) are the foundation. `educore-cms` is the most recent 9-file template. The `educore-events-domain/Cargo.toml` is scaffold-only (5 deps: `core`, `platform`, `rbac`, `events`, `settings`; **no `educore-academic` needed**). `crates/educore/src/lib.rs` already re-exports `events_domain`. The 4 Phase 2 `EventsCalendar{Create,Read,Update,Delete}` placeholders are the start point for `Capability`. The 4 Phase 2 `EventsCalendar*` placeholders are the start point for `AuditTarget` (the build-plan commits to 4 calendar audit targets in Phase 13).

**Working With Subagents:** Workstreams: A=`CalendarEvent` (the headline; RRULE-based recurrence + `Dispatch` action); B=`Holiday` + `Incident` (school-specific calendar metadata); C=`Weekend` (recurring weekly non-instruction day); D=reconcile cross-crate placeholders + integration test + coverage flips + handoff docs.

**Per-Deliverable Gotchas:**
- Eleventh crate, BUT in the **cross-cutting** tier (not domains!) — the build plan places `events-domain` in cross-cutting. Read `docs/build-plan.md` § "Phase 13" carefully. Stick to the 9-file layout per `AGENTS.md`.
- **Two `events` crates are easy to confuse.** `crates/cross-cutting/events/` is the envelope (Phase 2, locked). `crates/cross-cutting/events-domain/` is the Calendar domain (Phase 13). Document this clearly in both `lib.rs` headers and in `AGENTS.md`.
- RRULE subset (RFC 5545) for `CalendarEvent` recurrence — `FREQ` (DAILY/WEEKLY/MONTHLY/YEARLY), `INTERVAL`, `COUNT`, `UNTIL`. Holidays override recurring events on specific dates.
- 4 Phase 2 placeholders `EventsCalendar{Create,Read,Update,Delete}`. Net-new variants follow the wire form `<Domain>.<Aggregate>.<Action>` (`Events.Holiday.Create`, `Events.Incident.Update`, `Events.Weekend.Read`, etc.).
- The Calendar domain **does not** require `educore-academic` (no class / section / academic year references). It does require the standard `educore-platform::SchoolId` + `educore-platform::UserId` + `TenantContext`.
- `educore-events-domain` is the second cross-cutting domain crate (after `educore-settings` / `educore-operations` in Phase 14). It still uses the standard 9-file layout per `AGENTS.md`.
- Spec-faithful scope: 4 root aggregates per `docs/specs/events/aggregates.md` + child entities, ~12 typed events, ~12 typed commands, 4 repository port traits, 4 query stubs, 2-3 service factory fns (one per service struct in `services.md`), 8-12 net-new `Capability` variants, 4 net-new `AuditTarget` variants.
- Do NOT add a `educore-finance` dep (Phase 8 OQ #6 + Phase 10 OQ #3 + Phase 11 OQ #4 + Phase 12 OQ #5 carry-over).
- Do NOT add a `educore-notify` dep (Phase 10 OQ #4 + Phase 11 OQ #4 + Phase 12 OQ #6 carry-over — port lands in Phase 15).
- Do NOT add a `educore-attendance` dep (Phase 10 OQ #5 + Phase 12 OQ #7 carry-over).
- Do NOT add a `educore-documents` dep (Phase 11 OQ #6 + Phase 12 OQ #8 carry-over).
- Do NOT add a `educore-academic` dep (per spec — the Calendar does NOT reference class/section/year).

**Exit Criteria:** 4 root aggregates + 9-file layout; 2-3 service factory functions + 2-3 service structs; ≥ 4 `coverage.toml` rows flipped; integration test green on SQLite (always) + PG + MySQL (env-gated); `cargo test/clippy/fmt/lint --workspace` green; `PHASE-13-HANDOFF.md` + `phase-14-prompt.md` + `progress-tracker.md` + `build-plan.md § "Phase 13 outcome."`.

**When You Are Stuck:** `PHASE-12-HANDOFF.md` is the foundation. `cargo run -p educore-core --bin lint --features lint` is the no-gaps gate. The 9-file template is `crates/domains/cms/src/`. The proptest pattern is `crates/domains/documents/src/services.rs` (the 100-case proptest near the bottom of the file). The closing-agent verification checklist is in `docs/phase_prompt/README.md`.

**Subagent Orchestration:** To prevent duplicate work, every phase must enforce: (1) **File-level ownership** — every file in the owned crate is assigned to exactly one subagent. (2) **Section-level pre-allocation** — for files touched by multiple workstreams (e.g. `aggregate.rs` for 4 root aggregates), the prep subagent pre-creates named section markers; each workstream subagent's `Edit` anchors fall strictly inside its assigned range. (3) **Sequential phase gates** — `P0 prep` → `R1 reconcile-prep` → `wave 1/2/3` parallel workstreams → `R2 reconcile-impl` → `4-tests` → `5-docs` → `R3 final-validation` (9-command gate). (4) **Atomic commits per microtask** — every subagent produces exactly one commit with `Phase 13: <scope> (<workstream>)` message + `Co-Authored-By: Antigravity <antigravity@google.com>` trailer. (5) **Reconciler subagents are read-only** — they verify section boundaries + duplicate detection + stub-replacement but never write code.