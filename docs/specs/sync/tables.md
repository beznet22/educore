# Sync — Tables

Sync adds four local tables to the storage schema. The DDL
is emitted by the storage adapter at startup via
`storage.create_schema().await` — the same path as domain
tables. The tables are tenant-scoped (every PK includes
`school_id`) and append-only where the engine's invariants
demand it.

| Table                  | Purpose                                                  | Tenant-scoped |
| ---------------------- | -------------------------------------------------------- | -------------- |
| `local_outbox`         | Pending commands awaiting push                           | yes            |
| `sync_cursor`          | Per-aggregate last-applied server version                | yes            |
| `local_conflict_queue` | Open `ConflictRecord`s                                   | yes            |
| `sync_audit`           | Append-only audit of every sync state transition         | yes            |

## `local_outbox`

| Column            | Type              | Notes                                                |
| ----------------- | ----------------- | ---------------------------------------------------- |
| `school_id`       | `Uuid`            | Part of PK                                           |
| `outbox_id`       | `Uuid`            | Part of PK; the `OutboxEntryId`                      |
| `command_id`      | `Uuid`            | The command the entry is a result of                 |
| `command_type`    | `VARCHAR(128)`    | The static `COMMAND_TYPE` of the producer            |
| `idempotency_key` | `VARCHAR(64)`     | Server-side dedupe key                               |
| `payload`         | `JSONB` / `TEXT`  | The serialized `CommandEnvelope`                     |
| `status`          | `VARCHAR(16)`     | `Pending` \| `InFlight` \| `Acked` \| `Conflict`     |
| `attempt_count`   | `INTEGER`         | Number of push attempts                              |
| `last_error`      | `TEXT`            | Last error message (nullable)                        |
| `next_attempt_at` | `TIMESTAMP`       | Scheduled retry time                                 |
| `enqueued_at`     | `TIMESTAMP`       | Local acceptance time                                |
| `acked_at`        | `TIMESTAMP`       | Time of server ack (nullable)                        |

**Indexes:** `(school_id, status, next_attempt_at)` for the
push loop; `(school_id, idempotency_key)` for dedupe.

## `sync_cursor`

| Column           | Type              | Notes                                              |
| ---------------- | ----------------- | -------------------------------------------------- |
| `school_id`      | `Uuid`            | Part of PK                                         |
| `aggregate_type` | `VARCHAR(64)`     | Part of PK                                         |
| `aggregate_id`   | `Uuid`            | Part of PK                                         |
| `version`        | `VARCHAR(128)`    | Opaque `VersionCursor` string                      |
| `updated_at`     | `TIMESTAMP`       | Last advance time                                  |

**Indexes:** PK only. Lookups are always by full PK; the
table is a flat key-value map of "what was the last version
applied for this aggregate".

## `local_conflict_queue`

| Column            | Type              | Notes                                            |
| ----------------- | ----------------- | ------------------------------------------------ |
| `school_id`       | `Uuid`            | Part of PK                                       |
| `conflict_id`     | `Uuid`            | Part of PK; the `ConflictId`                     |
| `aggregate_type`  | `VARCHAR(64)`     | The aggregate under conflict                     |
| `aggregate_id`    | `Uuid`            | The specific aggregate under conflict            |
| `conflict_kind`   | `VARCHAR(64)`     | `FieldMismatch` \| `VersionStale` \| `DeletedOnRemote` \| `SchemaMismatch` |
| `local_outbox_id` | `Uuid`            | The local entry that diverged                    |
| `remote_event_id` | `Uuid`            | The server event that diverged                   |
| `local_payload`   | `JSONB` / `TEXT`  | Snapshot of the local command payload            |
| `remote_payload`  | `JSONB` / `TEXT`  | Snapshot of the remote event payload             |
| `status`          | `VARCHAR(16)`     | `Open` \| `Resolved`                             |
| `opened_at`       | `TIMESTAMP`       | Detection time                                   |
| `resolved_at`     | `TIMESTAMP`       | Resolution time (nullable)                       |

**Indexes:** `(school_id, status)` for the open-conflict
query.

## `sync_audit`

| Column           | Type              | Notes                                            |
| ---------------- | ----------------- | ------------------------------------------------ |
| `school_id`      | `Uuid`            | Tenant scope                                     |
| `audit_id`       | `Uuid`            | PK; new uuid per row                             |
| `event_type`     | `VARCHAR(64)`     | The sync event that was written                  |
| `aggregate_type` | `VARCHAR(64)`     | Nullable; set when the event targets an aggregate type |
| `actor_id`       | `Uuid`            | The user that triggered the transition           |
| `correlation_id` | `Uuid`            | The originating correlation                      |
| `payload`        | `JSONB` / `TEXT`  | The event payload                                |
| `recorded_at`    | `TIMESTAMP`       | Server-side write time                           |

**Indexes:** `(school_id, recorded_at DESC)` for the audit
view.

## Notes

- Every table includes `school_id` for multi-tenant
  isolation. The storage adapter enforces tenant scope on
  every read and write.
- The tables are emitted only when the `sync` Cargo feature
  is enabled on the umbrella (per
  [`overview.md`](./overview.md) § "Sync as a Build Feature").
  Embedded / server-only deployments that disable the
  feature compile without the sync DDL.
- Append-only tables (`sync_audit`, `local_outbox` once
  acked) are compacted by background jobs; the engine never
  mutates a row in place.
- The `payload` column is `JSONB` on PostgreSQL, `JSON` on
  MySQL, and `TEXT` on SQLite; the engine serializes with
  `serde_json` regardless of dialect.
