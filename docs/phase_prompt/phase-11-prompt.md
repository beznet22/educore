# Phase 11 Prompt — Documents

**Mission:** Deliver `educore-documents`. `FormDownload`, `PostalDispatch`, `PostalReceive`. **Implementation**, not design.

**Deliverables:** `crates/domains/documents/` with 9-file layout, 3 root aggregates (`FormDownload`, `PostalDispatch`, `PostalReceive`) + child entities, ~9 typed command shapes, ~9 typed events, 3 repository ports, 3 query stubs, 3 service factory functions, capability additions (new `Documents.*` group), audit additions. Vertical-slice test mirroring `communication_integration.rs`. Flip `coverage.toml` rows. Write `PHASE-11-HANDOFF.md` + `phase-12-prompt.md`.

**Required Reading:**
- `docs/handoff/PHASE-10-HANDOFF.md` (8 OQs carry over; Q1 the spec-faithful interpretation, Q2 the `NotificationProvider` port, Q6 the `ChatStatusRecord` rename are the most material for Phase 11)
- `docs/build-plan.md` § "Phase 11" + § "Phase 10 outcome."
- `docs/specs/documents/` (all 11 files: overview, aggregates, entities, value-objects, events, commands, services, repositories, permissions, tables, workflows)
- `docs/ports/{event-bus,storage,files}.md`, `docs/schemas/{tenancy,audit}-schema.md`
- `docs/decisions/ADR-013-CrateLayout.md`, `ADR-015-ExternalCrates.md`
- `crates/cross-cutting/events/src/{lib,domain_event,envelope}.rs`, `crates/cross-cutting/rbac/src/services.rs`, `crates/cross-cutting/audit/src/writer.rs`
- `crates/domains/communication/src/` (the most recent 9-file template)
- `crates/domains/library/src/` (the proptest pattern at `services.rs`)
- `crates/tools/storage-parity/tests/communication_integration.rs` (the 6-scenario vertical-slice template)
- `AGENTS.md`, `docs_guidlines/system.md`, `docs_guidlines/execution_guidlines.md`

**Starting Point:** 17 closed crates (7 cross-cutting + 8 domain + storage-parity + settings) are the foundation. `educore-communication` is the most recent 9-file template. No Phase 2 placeholders for `documents`; greenfield.

**Workstreams:** A=`FormDownload` (upload/update/delete + the `FileStorage` port); B=`PostalDispatch`; C=`PostalReceive`; D=reconcile cross-crate placeholders + integration test + coverage flips + handoff docs.

**Per-Deliverable Gotchas:**
- Ninth domain crate — stick to 9-file layout.
- File attachments go through the `FileStorage` port. The real `S3`/`local` impl is Phase 15; Phase 11 emits a `FileReference` and uses the port boundary.
- `PostalDispatch.reference_no` and `PostalReceive.reference_no` are unique within `(school_id, academic_id)` when set — enforce at the trait level.
- Soft delete only — `active_status` flag, never hard delete.
- Do NOT add a `educore-finance` dep (Phase 8 OQ #6 + Phase 10 OQ #3 carry forward).
- Do NOT add a `educore-notify` dep (Phase 10 OQ #4 carries forward — port lands in Phase 15).

**Exit Criteria:** 3 root aggregates + 9-file layout; `upload_form` + `dispatch_postal` + `receive_postal` service functions; capability-gate every mutation via `Capability::Documents*`; integration test green on SQLite (always) + PG + MySQL (env-gated); `cargo test/clippy/fmt/lint --workspace` green; ≥ 3 `coverage.toml` rows flipped; `PHASE-11-HANDOFF.md` + `phase-12-prompt.md` + `progress-tracker.md` + `build-plan.md` § "Phase 11 outcome.".

**When You Are Stuck:** `PHASE-10-HANDOFF.md` is the foundation. `cargo run -p educore-core --bin lint --features lint` is the no-gaps gate. The 9-file template is `crates/domains/communication/src/`. The append-only invariant pattern is `crates/domains/communication/src/repository.rs` (the `EmailLogRepository` + `SmsLogRepository` + `ChatStatusRepository` traits).
