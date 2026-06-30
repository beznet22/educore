# Sync — Research Notes

This file captures the design rationale, prior-art scan, and
research findings that informed the sync subsystem. It
mirrors the `research.md` files other spec folders use to
record why the engine ended up where it is.

## Sources

- **ADR-018 — Sync Engine Architecture** (Accepted
  2026-06-12, amended 2026-06-25). The single most important
  document for this spec. Establishes the in-process reference
  implementation, the transport-agnostic `SyncAdapter` port,
  the saga / compensating-action machinery, and the tier
  placement of `educore-sync-inprocess` in
  `crates/cross-cutting/`.
- **ADR-008 — Offline-First**. Defines the typed events,
  `event_id` (UUIDv7), `version`, and `etag` semantics the
  sync port assumes.
- **ADR-014 — Idempotency**. Defines the `idempotency_key`
  contract the outbox and the wire protocol share.
- **docs/ports/sync.md** — The full port contract: `dispatch`,
  `subscribe`, `snapshot`, `health`, plus the HTTP / WebSocket
  wire protocol (used by the deferred `WorkerHttpSyncAdapter`).
- **docs/guides/saas-backend.md** § "The Sync Engine"
  (lines 464-573). The earlier worker-process design the
  in-process reference extends.

## Prior Art Survey

The engine surveyed four prior sync systems before settling
on the current design:

1. **CRDT-based merge** (Automerge, Yjs). Strong conflict-
   free merge semantics but no domain validation; the
   server is not authoritative, which conflicts with the
   engine's "server is source of truth" invariant. Rejected.
2. **Operational transform** (Google Docs-style). Strong for
   collaborative text but heavyweight for typed domain
   commands. Rejected for the engine's command-shaped
   domain.
3. **Transactional outbox + change feed** (Debezium, LinkedIn
   Brooklin). The pattern the engine adopted: outbox row in
   the same transaction as the domain event; change feed
   driven by the outbox table; client tracks a cursor and
   applies events in order. Chosen because it matches the
   engine's command + event shape exactly.
4. **Custom replicators** (CouchDB, Firebase). Vendor-locked
   storage; the engine's storage-port abstraction makes the
   replicator an adapter concern, not a storage concern.
   Rejected on portability grounds.

## Findings From The Schoolify Codebase

The legacy Laravel project at `schoolify/` does not ship a
sync engine — the local-first / offline-first split postdates
the legacy code. The closest analogue is the
`schoolify_jobs` queue, which writes a row per background job
and a worker process drains it. The transactional-outbox
pattern in this spec is the engine's structured equivalent:
the outbox row replaces the ad-hoc job table, and the change
feed replaces the polling worker. No legacy Laravel table is
reused; the engine's four sync tables are net-new.

The legacy `schoolify_event_log` table (researched in
[`docs/research/schoolify-analysis.md`](../../research/schoolify-analysis.md))
is the closest in spirit to `sync_audit`: an append-only log
of every state change, scoped by school. The engine keeps the
shape (school_id + timestamp + JSON payload) but adds the
typed `event_type` discriminator that the legacy table lacks.

## Open Questions

The following questions are tracked in
[`docs/decisions/`](../../decisions/) and
[`docs/handoff/`](../../handoff/) and are not blockers for
the current spec landing:

- **CRDT vs server-authoritative for non-domain metadata.**
  The engine commits to server-authoritative for domain
  aggregates. UI preferences and per-device settings may
  benefit from a CRDT merge; a follow-up ADR will decide.
- **Multi-device cursor coordination.** A user with two
  devices currently holds independent cursors per device.
  A future PR may add a per-user reconciliation step that
  takes the union of both cursors.
- **Schema-upgrade migration window.** The recovery path
  assumes the client can detect a schema migration via a
  server-side signal. The signal shape is undecided
  (version bump vs feature flag); see ADR-008 § "Schema
  Upgrades".

## Spec Status

This file completes the 11-file spec-folder layout for
`docs/specs/sync/` per
[`docs/code-standards.md`](../../code-standards.md) §
"Spec folder layout". The other ten files
(`aggregates.md`, `value-objects.md`, `commands.md`,
`events.md`, `services.md`, `workflows.md`, `ports.md`,
`errors.md`, `tables.md`, plus the pre-existing
`overview.md`) are now in place.
