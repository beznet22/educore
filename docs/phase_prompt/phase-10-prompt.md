# Educore Phase 10 — Communication

## Mission
Deliver `educore-communication` — notice, complaint, chat message, email log, SMS log, notification setting. **Implementation**, not design.

## Deliverables
`crates/domains/communication/` with 9-file layout, 6 headline aggregates (`Notice`, `Complaint`, `ChatMessage`, `EmailLog`, `SmsLog`, `NotificationSetting`) + child entities, ~30 typed command shapes, ~18 typed events, 6 repository ports, 6 query stubs, 6 service factory functions, capability additions (the new `Communication.*` group + the 4 Phase 2 placeholders for dedup), audit additions. Vertical-slice test mirroring `library_integration.rs`. Flip `coverage.toml` rows. Write `PHASE-10-HANDOFF.md` + `phase-11-prompt.md`.

## Required Reading
- `docs/handoff/PHASE-9-HANDOFF.md` (7 OQs carry over; Q2 the 6-aggregate interpretation, Q5 the ISBN checksum validation, Q7 the `MemberId` sum type are the most material for Phase 10)
- `docs/build-plan.md` § "Phase 10" + § "Phase 9 outcome."
- `docs/specs/communication/` (all 11 files)
- `docs/ports/{event-bus,storage}.md`, `docs/schemas/{tenancy,audit}-schema.md`
- `docs/decisions/ADR-013-CrateLayout.md`, `ADR-015-ExternalCrates.md`
- `crates/cross-cutting/events/src/{lib,domain_event,envelope}.rs`, `crates/cross-cutting/rbac/src/services.rs`, `crates/cross-cutting/audit/src/writer.rs`
- `crates/domains/library/src/` (the most recent 9-file template)
- `crates/domains/finance/src/` (the proptest pattern at services.rs:1259)
- `crates/tools/storage-parity/tests/library_integration.rs` (the vertical-slice template)
- `AGENTS.md`, `docs_guidlines/system.md`, `docs_guidlines/execution_guidlines.md`

## Starting Point
16 closed crates (7 cross-cutting + 7 domain + storage-parity + settings) are the foundation. `educore-library` is the most recent 9-file template. The 4 Phase 2 `CommunicationMessage{Create,Read,Update,Delete}` placeholders are the start point. The 33 finance placeholder aggregates remain as the Workstreams D-M backlog.

## Working With Subagents
Workstreams: A=`Notice`; B=`Complaint`; C=`ChatMessage`; D=`EmailLog` + `SmsLog`; E=`NotificationSetting` + the notification-dispatch service; F=reconcile cross-crate placeholders + integration test + coverage flips + handoff docs.

## Per-Deliverable Gotchas
- Eighth domain crate — stick to 9-file layout.
- Notice types: `general`, `class`, `student`, `staff`, `parent`, `event`. Each has a different `recipient_scope` and a different set of read-capability checks.
- Email log is append-only (no status mutation). SMS log is append-only.
- `NotificationSetting` is a per-user configuration; the dispatcher reads it to decide delivery channel (email vs SMS vs both).
- The 4 Phase 2 `CommunicationMessage*` placeholders will likely dedup against the new `Notice*` / `Complaint*` / `ChatMessage*` variants. Use the same wire-form dedup pattern as Phase 9.
- Do NOT add a `educore-finance` dep (Phase 8 OQ #6 carries forward).
- Do NOT add a `educore-notify` dep (the engine `NotificationProvider` port lives in `educore-notify` and lands in Phase 15; Phase 10 emits events only and lets the consumer wire the subscriber).

## Exit Criteria
6 headline aggregates + 9-file layout; `notify_user` + `mark_as_read` + `send_*_message` service functions; the `CommunicationEvent` event family; capability-gate every read command via `Capability::CommunicationMessageRead`; integration test green on SQLite (always) + PG + MySQL (env-gated); `cargo test/clippy/fmt/lint --workspace` green; ≥ 6 `coverage.toml` rows flipped; `PHASE-10-HANDOFF.md` + `phase-11-prompt.md` + `progress-tracker.md` + `build-plan.md` § "Phase 10 outcome.".

## When You Are Stuck
`PHASE-9-HANDOFF.md` is the foundation. `cargo run -p educore-core --bin lint --features lint` is the no-gaps gate. The 9-file template is `crates/domains/library/src/`. The proptest pattern is `crates/domains/library/src/services.rs` (the 100-case proptest at the bottom of the file).
