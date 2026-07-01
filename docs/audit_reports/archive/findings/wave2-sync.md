# Wave 2 Cross-Cutting Audit Report — Sync Engine Architecture

**Scope:** `docs/decisions/ADR-018-SyncEngineArchitecture.md`,
`docs/ports/sync.md`, `docs/specs/sync/` (1 of 11 expected files),
`crates/cross-cutting/sync/` (port + command + health + lib), 
`crates/cross-cutting/sync-inprocess/` (in-process adapter),
`crates/cross-cutting/events/src/sync.rs` (the four typed events),
`crates/infra/storage/src/port.rs` (the four sync methods on
`StorageAdapter`), `crates/tools/testkit/src/sync.rs`, 
`docs/build-plan.md` Phase 0 sync tasks (lines 192-225),
`docs/handoff/PHASE-0-HANDOFF.md` and `PHASE-2-HANDOFF.md`,
`docs/architecture.md` lines 280-364, `docs/guides/saas-backend.md`
lines 522-625, `docs/coverage.toml` rows `sync_port` /
`sync_inprocess_impl` (lines 2297-2314), and `crates/educore/Cargo.toml`
umbrella exports.

**Total findings:** 27

---

### FINDING 1

- **id:** CC-SYNC-001
- **area:** cross-cutting-sync
- **severity:** Critical
- **location:** `crates/educore/Cargo.toml:20-49` (no `[features]` block)
- **description:** ADR-018 § 4 mandates a `sync` Cargo feature on the umbrella crate that gates `educore-sync` and `educore-sync-inprocess`: "`Without the `sync` feature, the engine has **no** sync capability (the `sync()` builder method is gated behind the feature). With the feature on, consumers pick: ...`". The actual `crates/educore/Cargo.toml` has **no `[features]` block at all**; both `educore-sync` and `educore-sync-inprocess` are unconditional dependencies (lines 46-47). The umbrella therefore pulls sync in for every consumer, including server-only deployments.
- **expected:** `docs/decisions/ADR-018-SyncEngineArchitecture.md:101-115` — `[features] default = []; sync = ["educore-sync", "educore-sync-inprocess"]` on `crates/educore/Cargo.toml`. Also `docs/architecture.md:362-364`: "The `sync` feature on the umbrella crate (`educore`) gates the in-process coordinator."
- **evidence:** `crates/educore/Cargo.toml` — `grep -n "feature" crates/educore/Cargo.toml` returns only the line 50 `tokio = { workspace = true, features = ["macros", ...] }` dev-dependency; there is no `[features]` table on the umbrella.

---

### FINDING 2

- **id:** CC-SYNC-002
- **area:** cross-cutting-sync
- **severity:** Critical
- **location:** `docs/decisions/ADR-018-SyncEngineArchitecture.md:92-100` vs `crates/cross-cutting/sync-inprocess/` (on disk)
- **description:** ADR-018 § 3 declares the in-process adapter lives at `crates/adapters/sync-inprocess/` (package `educore-sync-inprocess`). The actual crate lives at `crates/cross-cutting/sync-inprocess/` (cross-cutting tier), not `crates/adapters/sync-inprocess/`. AGENTS.md says the same (`sync-inprocess` is under cross-cutting), but the ADR's stated location disagrees. The `adapters/` tier directory does not have a `sync-inprocess/` subdirectory.
- **expected:** ADR says: `crates/adapters/sync-inprocess/` (adapters tier).
- **evidence:** `find crates -type d -name "sync*"` returns `/home/beznet/Workspace/smscore/crates/cross-cutting/sync-inprocess` and `/home/beznet/Workspace/smscore/crates/cross-cutting/sync`. `ls crates/adapters/` contains `auth, event-bus, files, integrations, notify, payment, storage-mysql, storage-postgres, storage-sqlite, storage-surrealdb` — no `sync-inprocess`.

---

### FINDING 3

- **id:** CC-SYNC-003
- **area:** cross-cutting-sync
- **severity:** Critical
- **location:** `docs/specs/sync/` (directory contents)
- **description:** `docs/specs/sync/` contains only `overview.md` (1162 lines, dated Phase 0). The 11-file layout mandated by `docs/code-standards.md` (overview, aggregates, entities, value-objects, commands, events, services, permissions, repositories, workflows, tables) requires 10 additional files. None of the documented sync aggregates (`OutboxEntry`, `SyncCursor`, `ConflictRecord`, `SyncSubscription`), any value object, any commands spec, any events spec, any services spec, any permissions spec, any repository spec, any workflow spec, any tables spec exist as spec files. The single `overview.md` is also a norm-violating dump of the entire spec into one file (1162 lines).
- **expected:** `docs/code-standards.md` "Spec folder layout" — 11 files per domain/cross-cutting folder.
- **evidence:** `ls /home/beznet/Workspace/smscore/docs/specs/sync/` returns `overview.md` only. The 11-file layout is visible in `docs/specs/platform/` (11 files), `docs/specs/academic/` (11 files), etc.

---

### FINDING 4

- **id:** CC-SYNC-004
- **area:** cross-cutting-sync
- **severity:** Critical
- **location:** `docs/build-plan.md:192-200` (Phase 0 task 6) and `docs/specs/sync/overview.md:355,372,390,409,427,447,461` (spec body)
- **description:** Both `build-plan.md` Phase 0 task 6 and the spec body reference a `SyncCoordinator` struct/trait and a 5-command / 7-event catalog. The actual code defines a different trait (`SyncAdapter`, `crates/cross-cutting/sync/src/port.rs:37`) with a 4-command catalog (`Start`/`Pause`/`Resume`/`Stop`, `crates/cross-cutting/sync/src/command.rs:23-37`) and 4 events (`SyncStarted`/`SyncPaused`/`SyncResumed`/`SyncStopped`, `crates/cross-cutting/events/src/sync.rs`). The `SyncCoordinator` symbol exists in **no** Rust source file (`grep -rn "SyncCoordinator" crates --include="*.rs"` returns no rows).
- **expected:** `docs/build-plan.md:193-198` — "Defines the `SyncCoordinator` trait, the command catalog (`SyncStart`, `SyncPause`, `SyncResume`, `SyncRequestDelta`, `SyncAcknowledge`), the event catalog (`SyncStarted`, `SyncPaused`, `SyncResumed`, `DeltaAvailable`, `DeltaAcknowledged`, `SyncConflictDetected`), and the shared coordinator struct". Also `docs/specs/sync/overview.md:355,372,...` 7 spec-body events.
- **evidence:** `grep -rn SyncCoordinator crates --include="*.rs"` returns zero hits; `crates/cross-cutting/sync/src/port.rs:37` defines `pub trait SyncAdapter: Send + Sync` with five methods (`start`, `pause`, `resume`, `stop`, `health`).

---

### FINDING 5

- **id:** CC-SYNC-005
- **area:** cross-cutting-sync
- **severity:** Critical
- **location:** `docs/build-plan.md:194-195` vs `crates/cross-cutting/sync/src/command.rs:23-37`
- **description:** The build plan mandates 5 sync commands: `SyncStart`, `SyncPause`, `SyncResume`, `SyncRequestDelta`, `SyncAcknowledge`. The actual command catalog has 4: `SyncCommand::Start(SchoolId)`, `::Pause(SchoolId)`, `::Resume(SchoolId)`, `::Stop(SchoolId)` (note: `Stop` not `Acknowledge`). `SyncRequestDelta` does not exist. `SyncAcknowledge` does not exist. `Stop` (which exists in code) is not listed in the build plan.
- **expected:** `docs/build-plan.md:194-195` — "`SyncStart`, `SyncPause`, `SyncResume`, `SyncRequestDelta`, `SyncAcknowledge`".
- **evidence:** `crates/cross-cutting/sync/src/command.rs:23-37`:
  ```rust
  pub enum SyncCommand {
      Start(SchoolId),
      Pause(SchoolId),
      Resume(SchoolId),
      Stop(SchoolId),
  }
  ```

---

### FINDING 6

- **id:** CC-SYNC-006
- **area:** cross-cutting-sync
- **severity:** Critical
- **location:** `docs/build-plan.md:196-197` vs `crates/cross-cutting/events/src/sync.rs:64-225`
- **description:** The build plan and the Phase 0 handoff list 6 sync events: `SyncStarted`, `SyncPaused`, `SyncResumed`, `DeltaAvailable`, `DeltaAcknowledged`, `SyncConflictDetected`. The actual `educore-events/src/sync.rs` defines **4** events: `SyncStarted` (line 64), `SyncPaused` (line 122), `SyncResumed` (line 158), `SyncStopped` (line 185). The four names in the plan that are not in code: `DeltaAvailable`, `DeltaAcknowledged`, `SyncConflictDetected`. The one name in code that is not in the plan: `SyncStopped`.
- **expected:** `docs/build-plan.md:196-197` — "`SyncStarted`, `SyncPaused`, `SyncResumed`, `DeltaAvailable`, `DeltaAcknowledged`, `SyncConflictDetected`" and `docs/handoff/PHASE-0-HANDOFF.md:36-37` — "SyncStarted, SyncPaused, SyncResumed, DeltaAvailable, DeltaAcknowledged. Missing: SyncAcknowledge command, SyncConflictDetected event."
- **evidence:** `crates/cross-cutting/events/src/sync.rs` — `grep -nE "^pub struct (Sync|Delta)" src/sync.rs` returns 4 rows: `SyncStarted`, `SyncPaused`, `SyncResumed`, `SyncStopped`. `grep -n "DeltaAvailable\|DeltaAcknowledged\|SyncConflictDetected" crates/cross-cutting/events/src/sync.rs` returns no rows.

---

### FINDING 7

- **id:** CC-SYNC-007
- **area:** cross-cutting-sync
- **severity:** Critical
- **location:** `docs/ports/sync.md:48-58` (`SyncAdapter` trait) vs `crates/cross-cutting/sync/src/port.rs:37-66` (actual `SyncAdapter`)
- **description:** The wire-protocol port doc `docs/ports/sync.md` defines a `SyncAdapter` trait with four async methods: `dispatch(envelope)`, `subscribe(filter) -> EventStream`, `snapshot(school_id)`, `health()`. The actual port trait `crates/cross-cutting/sync/src/port.rs:37-66` defines a `SyncAdapter` trait with five methods: `start`, `pause`, `resume`, `stop`, `health`. **None** of `dispatch`/`subscribe`/`snapshot`/`CommandEnvelope`/`EventStream`/`SchoolSnapshot`/`CommandOutcome`/`EventFilter` (the entire port doc API surface, lines 60-417) is implemented in code. The port doc and the code port have **zero overlapping methods** beyond `health`.
- **expected:** `docs/ports/sync.md:48-58` — `pub trait SyncAdapter { async fn dispatch(...); async fn subscribe(...); async fn snapshot(...); async fn health(...); }`.
- **evidence:** `docs/ports/sync.md:48-58` vs `crates/cross-cutting/sync/src/port.rs:37-66`. `grep -rn "CommandEnvelope\|EventStream\|SchoolSnapshot\|CommandOutcome\|fn dispatch\|fn subscribe" crates/cross-cutting/sync crates/cross-cutting/sync-inprocess` returns no rows.

---

### FINDING 8

- **id:** CC-SYNC-008
- **area:** cross-cutting-sync
- **severity:** High
- **location:** `docs/decisions/ADR-018-SyncEngineArchitecture.md:165-194` (ADR § 5 — `StorageAdapter` methods) vs `crates/infra/storage/src/port.rs:112-148` (actual port)
- **description:** ADR-018 § 5 declares the four new `StorageAdapter` methods with parameterless signatures: `watch_changes(&self)`, `apply_snapshot(&self, snapshot)`, `cursor_for(&self)`, `advance_cursor(&self, cursor)`. The actual port signatures differ:
  - ADR: `watch_changes(&self)` → code: `async fn watch_changes(&self, filter: ChangeFilter)`.
  - ADR: `apply_snapshot(&self, snapshot: Snapshot)` → code: `async fn apply_snapshot(&self, snapshot: SchoolSnapshot)`.
  - ADR: `cursor_for(&self)` → code: `async fn cursor_for(&self, school_id: SchoolId)`.
  - ADR: `advance_cursor(&self, cursor: Cursor)` → code: `async fn advance_cursor(&self, school_id: SchoolId, to: VersionCursor)`.
  
  Three of the four are missing the `school_id` scoping the ADR omits; the first is missing the `ChangeFilter` argument. The spec body (`docs/specs/sync/overview.md:643-714`) gives yet a third set: `watch_changes(school_id, aggregate_type, from)`, `cursor_for(school_id, aggregate_type, aggregate_id)`, `advance_cursor(school_id, aggregate_type, aggregate_id, to, transaction)`. Three sources (ADR, port doc, spec body) describe three different APIs.
- **expected:** `docs/decisions/ADR-018-SyncEngineArchitecture.md:165-194`.
- **evidence:** `crates/infra/storage/src/port.rs:112-148`:
  ```rust
  async fn watch_changes(&self, filter: ChangeFilter) -> Result<ChangeStream> { ... }
  async fn apply_snapshot(&self, snapshot: SchoolSnapshot) -> Result<()> { ... }
  async fn cursor_for(&self, school_id: SchoolId) -> Result<VersionCursor> { ... }
  async fn advance_cursor(&self, school_id: SchoolId, to: VersionCursor) -> Result<()> { ... }
  ```

---

### FINDING 9

- **id:** CC-SYNC-009
- **area:** cross-cutting-sync
- **severity:** Critical
- **location:** `crates/adapters/storage-surrealdb/src/storage.rs:129-167`
- **description:** ADR-018 § 5 states "Only the four shipped storage adapters override these methods: `educore-storage-surrealdb` (Phase 0 primary; per ADR-017)...". The actual `SurrealStorageAdapter::watch_changes`, `apply_snapshot`, `cursor_for`, `advance_cursor` (lines 129-167) are all **stubbed** with `NotSupported` — the SurrealDB adapter does **not** override the sync methods. `apply_snapshot` returns "SurrealStorageAdapter::apply_snapshot is not yet implemented" (line 147). The Phase 0 primary sync engine target therefore has zero sync-port coverage; the in-process sync adapter only emits lifecycle events and never reads from the storage change feed.
- **expected:** `docs/decisions/ADR-018-SyncEngineArchitecture.md:188-194` — "Only the four shipped storage adapters override these methods: educore-storage-surrealdb...".
- **evidence:** `crates/adapters/storage-surrealdb/src/storage.rs:129-167` (full block). All four methods return `NotSupported` after logging "StorageAdapter::watch_changes called on a closed adapter" / "apply_snapshot is not yet implemented". The Phase 0 e2e test (`crates/adapters/storage-surrealdb/tests/outbox_e2e.rs`) never calls any of the four sync methods.

---

### FINDING 10

- **id:** CC-SYNC-010
- **area:** cross-cutting-sync
- **severity:** Critical
- **location:** `docs/specs/sync/overview.md:883,1048` and `:42,64,849,885,941`
- **description:** The sync spec repeatedly references a worker binary `educore-worker` and a server crate `educore-sync-server` / `educore-sync-server-http`. Neither exists in the workspace. The actual crates are `educore-sync` and `educore-sync-inprocess` (cross-cutting tier). The umbrella binary is `educore-cli` (per AGENTS.md Crate Inventory row 35). The spec's references to `educore-worker` and `educore-sync-server` are non-existent constructs. Note: this finding is also filed in `wave6-specs-4.md` finding 7 (SPEC-4-007) but is restated here because it is a blocker for sync engine deployment.
- **expected:** Sync spec should reference the actual crate names; the worker binary (if/when shipped) must be a real workspace member.
- **evidence:** `grep -n "educore-worker\|educore-sync-server" docs/specs/sync/overview.md` returns 6 rows: `:42, :64, :849, :883, :885, :941, :1048`. `find crates -type d -name "*worker*" -o -name "*sync-server*"` returns no rows.

---

### FINDING 11

- **id:** CC-SYNC-011
- **area:** cross-cutting-sync
- **severity:** Critical
- **location:** `crates/cross-cutting/sync-inprocess/src/lib.rs` (entire crate, 390 lines)
- **description:** Per `docs/build-plan.md:217-222` Phase 0 task 10, the sync integration test "insert one outbox row and verify the in-process consumer received the event via the `SyncCoordinator`". The actual integration test in `crates/cross-cutting/sync-inprocess/src/lib.rs:204-390` only exercises the `InProcessSyncAdapter`'s own `start`/`pause`/`resume`/`stop` lifecycle against an `InProcessEventBus`. It never inserts an outbox row; it never invokes the `Outbox` sub-port on any storage adapter; it never asserts a domain event flows from a storage operation through the sync engine to a consumer. The Phase 0 sync e2e referenced in the handoff (`crates/adapters/storage-surrealdb/tests/outbox_e2e.rs`) does the outbox round-trip but does **not** wire `InProcessSyncAdapter` to receive the event.
- **expected:** `docs/build-plan.md:217-222` — "with the in-process sync impl wired into the Phase 0 outbox scenario, insert one outbox row and verify the in-process consumer received the event via the `SyncCoordinator`".
- **evidence:** `grep -n "outbox\|Outbox" crates/cross-cutting/sync-inprocess/src/lib.rs` returns no rows; `grep -n "InProcessSyncAdapter\|educore_sync" crates/adapters/storage-surrealdb/tests/outbox_e2e.rs` returns no rows.

---

### FINDING 12

- **id:** CC-SYNC-012
- **area:** cross-cutting-sync
- **severity:** High
- **location:** `crates/tools/testkit/src/sync.rs:1-43` (entire file)
- **description:** `crates/tools/testkit/src/sync.rs` is a 43-line placeholder. It exports a single `dummy_witness()` no-op function. The crate's doc-comment (lines 14-23) says "The actual sync primitives (`ChangeStream`, `VersionCursor`, `watch_changes`, `apply_snapshot`, `cursor_for`, `advance_cursor`) are exposed as methods on the in-memory storage adapter — see `storage::InMemoryStorageAdapter`." The testkit therefore does not expose any sync primitives of its own, yet `docs/coverage.toml:2193` declares `tests = "crates/tools/testkit/src/{storage,auth,notify,payment,files,integrations,event_bus,sync}.rs"` as if `sync.rs` provides sync test fixtures. The placeholder function does not consume a `SyncAdapter` nor a `StorageAdapter`.
- **expected:** A testkit module exposing pre-built `InProcessSyncAdapter` instances wired to a `tokio::sync::broadcast` consumer registry (per `docs/architecture.md:355-364` "30 minutes to a working offline-first app").
- **evidence:** `crates/tools/testkit/src/sync.rs:35-40` — `pub fn dummy_witness() {}` only. `crates/tools/testkit/src/sync.rs:1-43` total — no types, no traits, no struct.

---

### FINDING 13

- **id:** CC-SYNC-013
- **area:** cross-cutting-sync
- **severity:** High
- **location:** `docs/specs/sync/overview.md:1134-1140` (Phase 0 status block) vs spec body `:501,517,533,547,563,579`
- **description:** The "Phase 0 status" block at `docs/specs/sync/overview.md:1134-1140` uses command and event names that do not appear anywhere else in the spec body or in code: `SyncStart`, `SyncPause`, `SyncResume`, `SyncRequestDelta`, `SyncAcknowledge`, `DeltaAvailable`, `DeltaAcknowledged`, `SyncConflictDetected`. The spec body (lines 501-588) uses `RequestSyncCommand`, `PauseSyncCommand`, `ResumeSyncCommand`, `ResolveConflictCommand`, `SwitchSchoolCommand`, `ApplyRemoteChangeCommand`. The code uses `SyncCommand::{Start,Pause,Resume,Stop}` and events `SyncStarted/SyncPaused/SyncResumed/SyncStopped`. Three sources (Phase 0 status block, spec body, code) name the same surface three different ways. `SyncStopped` is claimed "deferred" at line 1140 but is the only Stop-equivalent in code (`crates/cross-cutting/events/src/sync.rs:185`).
- **expected:** A single canonical command/event catalog, used by the spec body, the Phase 0 status block, and the code.
- **evidence:** `docs/specs/sync/overview.md:1134-1140`:
  ```text
  **Commands shipped (4 of 6):** `SyncStart`, `SyncPause`,
  `SyncResume`, `SyncRequestDelta`. The `SyncAcknowledge`
  command is deferred (the in-process impl acknowledges
  inline in the test path).
  **Events shipped (5 of 7):** `SyncStarted`, `SyncPaused`,
  `SyncResumed`, `DeltaAvailable`, `DeltaAcknowledged`.
  `SyncConflictDetected` and `SyncStopped` are deferred.
  ```
  vs `docs/specs/sync/overview.md:501` `## RequestSyncCommand`. Same overlap with `docs/audit_reports/findings/wave6-specs-4.md` finding SPEC-4-006.

---

### FINDING 14

- **id:** CC-SYNC-014
- **area:** cross-cutting-sync
- **severity:** High
- **location:** `docs/specs/sync/overview.md:222-227` (SyncSubscription aggregate) and `:245-258` (SyncSubscription invariants)
- **description:** `SyncSubscription` is documented as an aggregate with `Idle`/`Streaming`/`Backoff`/`Paused`/`Stalled` states, per-aggregate-type subscriptions, `pause`/`resume` semantics, and a backoff policy. The implementation collapses subscription state to a single `SyncStatus` enum (`Running`/`Paused`/`Stopped`, `crates/cross-cutting/sync/src/health.rs:23-35`) at the **adapter** level, not per-(school, aggregate_type). There is no `SyncSubscription` struct, no `SubscriptionState` enum with `Streaming`/`Backoff`/`Stalled` variants, no per-aggregate-type cursor table, no backoff policy implementation. A multi-school consumer (`SwitchSchoolCommand` per `:547-563`) is impossible.
- **expected:** `docs/specs/sync/overview.md:226-227` and `:245-258` — SyncSubscription as a per-(school, aggregate_type, client_id) aggregate with five-state state machine.
- **evidence:** `crates/cross-cutting/sync/src/health.rs:23-35` defines only `enum SyncStatus { Running, Paused, Stopped }` (3 states, not 5). `grep -rn "SyncSubscription\|SubscriptionState\|Stalled\|Backoff" crates/cross-cutting/sync crates/cross-cutting/sync-inprocess` returns no rows in source.

---

### FINDING 15

- **id:** CC-SYNC-015
- **area:** cross-cutting-sync
- **severity:** High
- **location:** `docs/specs/sync/overview.md:165-209` (OutboxEntry aggregate), `:286-330` (ConflictRecord), `:222-227` (SyncSubscription)
- **description:** The sync spec defines four bookkeeping aggregates (`OutboxEntry`, `SyncCursor`, `ConflictRecord`, `SyncSubscription`) at `docs/specs/sync/overview.md:212-258`. **None** of these aggregates is implemented in code. `grep -rn "OutboxEntry\|SyncCursor\|ConflictRecord" crates/cross-cutting/sync crates/cross-cutting/sync-inprocess crates/cross-cutting/events` returns no rows (the storage-port `Outbox` sub-port is a different type — `crates/infra/storage/src/outbox.rs`, not the bookkeeping aggregate). The spec's "tables" section (`:1107-1133`) declares four storage tables (`local_outbox`, `sync_cursor`, `local_conflict_queue`, `sync_audit`); no migration emits them, and the SurrealDB adapter does not have these table definitions.
- **expected:** `docs/specs/sync/overview.md:212-258` and `:1107-1133` — 4 aggregates + 4 tables implemented and emitted.
- **evidence:** `grep -rn "OutboxEntry\|SyncCursor\|ConflictRecord\|local_outbox\|sync_cursor\|local_conflict_queue" crates/ migrations/` returns zero hits in any source or migration file. `migrations/engine/0000_engine_core.surreal.surql` does not contain these four tables.

---

### FINDING 16

- **id:** CC-SYNC-016
- **area:** cross-cutting-sync
- **severity:** High
- **location:** `crates/cross-cutting/sync/src/port.rs:35-66` (whole trait)
- **description:** The `SyncAdapter` trait surface has no `dispatch`, `subscribe`, or `snapshot` methods, so the wire-protocol port (`docs/ports/sync.md:48-58`) cannot be implemented against this trait. A consumer cannot drive an actual offline-first client (which needs to push outbox entries to the central store and subscribe to remote events) using the published trait. The published trait is session-control only (`start`/`pause`/`resume`/`stop`/`health`); the wire-protocol port doc promises a full bidirectional sync API that the trait cannot deliver.
- **expected:** `docs/ports/sync.md:48-58` defines `dispatch`, `subscribe`, `snapshot`, `health` on `SyncAdapter`.
- **evidence:** `crates/cross-cutting/sync/src/port.rs:37-66` defines exactly `start`, `pause`, `resume`, `stop`, `health`. No `dispatch`/`subscribe`/`snapshot` rows.

---

### FINDING 17

- **id:** CC-SYNC-017
- **area:** cross-cutting-sync
- **severity:** High
- **location:** `crates/cross-cutting/sync-inprocess/src/lib.rs:60-77` (whole `InProcessSyncAdapter`)
- **description:** `InProcessSyncAdapter` does **not** read from a local outbox and does **not** dispatch outbox events to consumers. ADR-018 § 3 (`educore-sync-inprocess` "drains the local outbox and applies remote snapshots without any network I/O") and `docs/build-plan.md:204-206` Phase 0 task 7 ("owns an in-process `EventBus` and dispatches every outbox event to a registered set of in-process consumers") both require outbox-driven fan-out. The actual adapter only listens for `SyncCommand::{Start,Pause,Resume,Stop}` and publishes the corresponding lifecycle event; it has no reference to an `Outbox` sub-port, no registered consumer set, no drain loop.
- **expected:** `docs/build-plan.md:204-206` — "in-process `EventBus` and dispatches every outbox event to a registered set of in-process consumers".
- **evidence:** `grep -n "outbox\|Outbox\|drain\|consumer\|subscriber" crates/cross-cutting/sync-inprocess/src/lib.rs` returns zero rows. The `InProcessSyncAdapter` struct (`:72-77`) holds only `bus: Arc<dyn EventBus>` and `state: Arc<Mutex<SyncHealth>>`; no outbox/consumer fields.

---

### FINDING 18

- **id:** CC-SYNC-018
- **area:** cross-cutting-sync
- **severity:** High
- **location:** `docs/specs/sync/overview.md:830-895` (`SyncCoordinator` + `WorkerHttpSyncAdapter` services) and `:1041-1053` (wire protocol design)
- **description:** The spec defines two services: the in-process `SyncCoordinator` (which "owns the per-(school, aggregate_type) subscription state" and runs push/pull loops) and `WorkerHttpSyncAdapter` (which is "purely a transport binding"). The implementation is `InProcessSyncAdapter`, a session-control stub with no push loop, no pull loop, no subscription state. There is no `SyncCoordinator` struct; the `WorkerHttpSyncAdapter` is not implemented; the wire-protocol HTTP client (`docs/ports/sync.md`) is not implemented. The "two deployments, same bookkeeping" claim is structurally violated.
- **expected:** `docs/specs/sync/overview.md:830-895`.
- **evidence:** `grep -rn "SyncCoordinator\|WorkerHttpSyncAdapter\|push_loop\|pull_loop" crates --include="*.rs"` returns no rows in source. `crates/cross-cutting/sync-inprocess/src/lib.rs:160-170` implements `start`/`pause`/`resume`/`stop`/`health` only.

---

### FINDING 19

- **id:** CC-SYNC-019
- **area:** cross-cutting-sync
- **severity:** High
- **location:** `docs/specs/sync/overview.md:1107-1133` (tables)
- **description:** The sync spec's "Tables" section lists four sync tables (`local_outbox`, `sync_cursor`, `local_conflict_queue`, `sync_audit`). The migration directory `migrations/engine/` ships one SurrealDB DDL file (`0000_engine_core.surreal.surql`, 50+ lines) which contains only the 6 cross-cutting engine tables (`outbox`, `audit_log`, `idempotency`, `event_log`, `schema_registry`, `system_user`); none of the four sync-specific tables are emitted. The runtime DDL emission flow at `docs/schemas/sql-dialects/README.md` therefore will not create the sync tables at startup. Even though `storage.create_schema()` is invoked by the consumer, the sync machinery has nowhere to persist cursors or outbox entries.
- **expected:** `docs/specs/sync/overview.md:1107-1133` — 4 sync tables in `migrations/engine/0000_engine_core.surreal.surql` (or per-domain emitted by macro).
- **evidence:** `grep -n "local_outbox\|sync_cursor\|local_conflict_queue\|sync_audit" migrations/engine/0000_engine_core.surreal.surql` returns no rows.

---

### FINDING 20

- **id:** CC-SYNC-020
- **area:** cross-cutting-sync
- **severity:** High
- **location:** `docs/ports/sync.md:419-435` ("Configuration" section)
- **description:** The port doc says `WorkerHttpSyncAdapter::builder()` is the production wiring path, with `SYNC_ENGINE_URL`, `DEVICE_TOKEN`, exponential backoff config, etc. The `WorkerHttpSyncAdapter` is **not implemented** in any crate (no `educore-sync-http` crate exists; `crates/adapters/` has no `sync-http/`). The build plan notes (`docs/build-plan.md:71` and `:205-210`) mark it "deferred to Phase 2" but `crates/educore/Cargo.toml` already depends on `educore-sync-inprocess` unconditionally (Finding 1), so the deferred state is not enforced.
- **expected:** Either `educore-sync-http` is implemented and wired, or the umbrella does not promise `WorkerHttpSyncAdapter` is available.
- **evidence:** `grep -rn "WorkerHttpSyncAdapter\|educore-sync-http" crates --include="*.rs" --include="*.toml"` returns no source-tree rows. `find crates -type d -name "sync-http"` returns no rows.

---

### FINDING 21

- **id:** CC-SYNC-021
- **area:** cross-cutting-sync
- **severity:** Medium
- **location:** `crates/cross-cutting/sync/src/port.rs:60-66` (`stop` method)
- **description:** The `SyncAdapter::stop` doc-comment (`:63-65`) says "Idempotent: stopping an already-stopped school is a no-op." But the implementation in `InProcessSyncAdapter::send_command` (`crates/cross-cutting/sync-inprocess/src/lib.rs:135-140`) **always** transitions `state.status = SyncStatus::Stopped` and **always** emits a `SyncStopped` event, even when the school was already stopped. The "no-op" promise is not honored; duplicate `Stop` calls produce duplicate events. No idempotency guard.
- **expected:** `crates/cross-cutting/sync/src/port.rs:63-65` — "Idempotent: stopping an already-stopped school is a no-op."
- **evidence:** `crates/cross-cutting/sync-inprocess/src/lib.rs:124-150` — `SyncCommand::Stop(s)` unconditionally sets `status = Stopped` and unconditionally publishes `SyncStopped::now(school)`.

---

### FINDING 22

- **id:** CC-SYNC-022
- **area:** cross-cutting-sync
- **severity:** Medium
- **location:** `crates/cross-cutting/sync/src/health.rs:23-35` (`SyncStatus` enum)
- **description:** The `SyncStatus` enum has 3 variants (`Running`, `Paused`, `Stopped`); the spec body (`docs/specs/sync/overview.md:245-258`) mandates 5 per-subscription states (`Idle`, `Streaming`, `Backoff`, `Paused`, `Stalled`). The implementation collapses subscription state to adapter-level. Even at adapter-level, the ADR § 5 design implies 4 states (Started/Paused/Resumed/Stopped) but the code emits `Running` on both `Start` and `Resume` (`:115-129`), making the `Running` state ambiguous (does it mean "just started" or "resumed"?).
- **expected:** `docs/specs/sync/overview.md:245-258` — 5 per-subscription states; or, at minimum, separate `Started` and `Running` adapter states.
- **evidence:** `crates/cross-cutting/sync/src/health.rs:23-35`:
  ```rust
  pub enum SyncStatus {
      Running,
      Paused,
      Stopped,
  }
  ```
  `crates/cross-cutting/sync-inprocess/src/lib.rs:108-115` and `:127-129` both write `state.status = SyncStatus::Running` for `Start` and `Resume`.

---

### FINDING 23

- **id:** CC-SYNC-023
- **area:** cross-cutting-sync
- **severity:** Medium
- **location:** `crates/cross-cutting/events/src/sync.rs:64-94` (`SyncStarted` struct)
- **description:** `SyncStarted` carries only `event_id` / `school_id` / `at`. The spec body mandates additional fields: `subscription_id: SyncSubscriptionId`, `aggregate_type: AggregateType`, `from_version: VersionCursor`, `request_id: Uuid` (`docs/specs/sync/overview.md:355-368`). The minimal struct cannot express the per-subscription identity or the cursor from which the subscription started, so downstream consumers (e.g. audit, UI) cannot correlate a `SyncStarted` event with the originating subscription.
- **expected:** `docs/specs/sync/overview.md:358-368` — full payload.
- **evidence:** `crates/cross-cutting/events/src/sync.rs:64-76` — `pub struct SyncStarted { pub event_id: Uuid; pub school_id: SchoolId; pub at: Timestamp; }`. No `subscription_id`, no `aggregate_type`, no `from_version`, no `request_id`.

---

### FINDING 24

- **id:** CC-SYNC-024
- **area:** cross-cutting-sync
- **severity:** Medium
- **location:** `crates/cross-cutting/events/src/sync.rs:122-225` (`SyncPaused`/`SyncResumed`/`SyncStopped`)
- **description:** `SyncPaused`/`SyncResumed`/`SyncStopped` carry an `Option<Uuid>` `session_started_event_id`, but the implementation in `crates/cross-cutting/sync-inprocess/src/lib.rs:131-150` always mints the events via `SyncPaused::now(school)` / `SyncResumed::now(school)` / `SyncStopped::now(school)` — the `for_session` correlator constructors (which would set `session_started_event_id`) are never invoked. The correlation field is therefore always `None` at runtime. The `into_envelope` in the adapter also drops the `event_id` from the typed event (the adapter creates a fresh envelope instead).
- **expected:** `docs/specs/sync/overview.md:381-396, 415-429, 451-465` — events carrying `subscription_id`, `aggregate_type`, and `session_started_event_id` (correlated).
- **evidence:** `crates/cross-cutting/sync-inprocess/src/lib.rs:131-150`:
  ```rust
  SyncCommand::Pause(_) => SyncPaused::now(school).into_envelope(&ctx),
  SyncCommand::Resume(_) => SyncResumed::now(school).into_envelope(&ctx),
  SyncCommand::Stop(_) => SyncStopped::now(school).into_envelope(&ctx),
  ```
  `SyncPaused::now(school)` constructs `session_started_event_id: None`.

---

### FINDING 25

- **id:** CC-SYNC-025
- **area:** cross-cutting-sync
- **severity:** Medium
- **location:** `docs/specs/sync/overview.md:981-1001` (permissions table) and `crates/cross-cutting/sync/src/port.rs` (whole file)
- **description:** The spec's permissions table (`docs/specs/sync/overview.md:981-1001`) lists 6 sync capabilities: `Sync.Request`, `Sync.Pause`, `Sync.Resume`, `Sync.ResolveConflict`, `Sync.SwitchSchool`, `Sync.CompactOutbox`. The `educore-sync` crate depends on `educore-core` only (no `educore-rbac` dependency, `crates/cross-cutting/sync/Cargo.toml:13-23`); the port trait (`crates/cross-cutting/sync/src/port.rs`) takes only `SchoolId`, no `actor_id`, no capability check. `grep -rn "Sync\\.Request\|Sync\\.Pause\|Sync\\.ResolveConflict" crates/cross-cutting/sync crates/cross-cutting/sync-inprocess` returns no rows. No sync command performs RBAC.
- **expected:** `docs/specs/sync/overview.md:981-1001` — capability-gated commands.
- **evidence:** `crates/cross-cutting/sync/Cargo.toml:13-23` lists `educore-core`, `educore-events`, `async-trait`, `bytes`, `serde`, `serde_json`, `tracing`, `uuid`, `futures`. No `educore-rbac` row. `crates/cross-cutting/sync/src/port.rs:35-66` has no actor parameter.

---

### FINDING 26

- **id:** CC-SYNC-026
- **area:** cross-cutting-sync
- **severity:** Medium
- **location:** `docs/ports/sync.md:560-585` (audit events) and `crates/cross-cutting/sync-inprocess/src/lib.rs` (entire)
- **description:** The wire-protocol port doc promises 7 audit events: `SyncDispatched`, `SyncConflictSurfaced`, `SyncConflictResolved`, `SyncSnapshotTaken`, `SyncSubscribed`, `SyncHealthFailed`, `SyncReconnected` (lines 560-585). The `educore-sync` and `educore-sync-inprocess` crates emit **no** audit events at all — there is no `AuditSink` port injection, no `audit_log` call. The in-process adapter's only side effect on a state transition is the typed lifecycle event publish. The Phase 2 audit cross-cutting crate (`educore-audit`) is **not** a dependency of either sync crate.
- **expected:** `docs/ports/sync.md:560-585` — 7 audit events written to `audit_log`.
- **evidence:** `grep -n "AuditSink\|audit_log\|SyncDispatched\|SyncConflictSurfaced\|educore-audit" crates/cross-cutting/sync/Cargo.toml crates/cross-cutting/sync-inprocess/Cargo.toml crates/cross-cutting/sync/src/*.rs crates/cross-cutting/sync-inprocess/src/*.rs` returns no rows.

---

### FINDING 27

- **id:** CC-SYNC-027
- **area:** cross-cutting-sync
- **severity:** Low
- **location:** `docs/specs/sync/overview.md:1107` and `docs/build-plan.md:194`
- **description:** Minor terminology drift between sources: the spec body calls the in-process service `SyncCoordinator` (`docs/specs/sync/overview.md:830`), the build plan calls it `SyncCoordinator` (`:194`) but the implementation class is `InProcessSyncAdapter`. The handoff (`docs/handoff/PHASE-0-HANDOFF.md:30`) calls it "the in-process coordinator"; the ADR-018 § 3 calls it `EducoreSyncAdapter::in_process()` — **none** of these names match the actual type `InProcessSyncAdapter`. The umbrella crate's exported alias is `sync_inprocess` (`:55`). Four different names (`SyncCoordinator`, `InProcessSyncAdapter`, `EducoreSyncAdapter`, `sync_inprocess`) for one struct.
- **expected:** A single canonical name used consistently across spec, build plan, ADR, code, handoff, and umbrella.
- **evidence:** `docs/specs/sync/overview.md:830` `### SyncCoordinator (in-process reference)`; `docs/build-plan.md:194` `SyncCoordinator trait`; `docs/decisions/ADR-018-SyncEngineArchitecture.md:93` `EducoreSyncAdapter::in_process()`; `crates/educore/src/lib.rs:55` `pub use educore_sync_inprocess as sync_inprocess;`; `crates/cross-cutting/sync-inprocess/src/lib.rs:72` `pub struct InProcessSyncAdapter`.

---

## Summary

**Implementation status:** The sync engine has a partial Phase 0 scaffold
(two crates, port trait, four events, in-process adapter). The wire-protocol
port doc (`docs/ports/sync.md`) and the spec body (`docs/specs/sync/overview.md`)
describe a far larger surface (full session control + outbox + cursor +
conflict + subscription + snapshot + HTTP wire protocol) that is **not
implemented**. The umbrella crate has no `sync` feature (Finding 1). The
sync storage-port methods on the SurrealDB adapter are all stubbed
(Finding 9). The in-process adapter does not drain an outbox (Finding 17).
The testkit placeholder exports `dummy_witness()` (Finding 12). Three
sources (ADR, port doc, spec body) describe three different APIs (Finding 8).
The sync spec folder has 1 of 11 expected files (Finding 3).
