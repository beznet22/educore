# Wave 5 — Documentation Audit (Docs Group 5)

**Scope:** `docs/schemas/*.md` (6 schema files), `docs/schemas/sql-dialects/*.md` (5 dialect files), `docs/schemas/data-migration/*.md` (13 files), `migrations/engine/0000_engine_core.{mysql,postgres,sqlite,surreal}.{sql,surql}` (4 canonical DDL files).

**Audit date:** 2026-06-23.

**Checks performed:**
1. Schema spec vs DDL drift (does the canonical DDL match the schema it claims to implement?).
2. Dialect differences (does each of the 4 backends implement the same contract?).
3. Data-migration plan accuracy (do aggregate counts, table renames, and column maps agree with the schema?).

---

### FINDING 1

- **id:** DOC-SCHM-001
- **area:** documentation
- **severity:** High
- **location:** `docs/schemas/audit-schema.md:51-52` vs `migrations/engine/0000_engine_core.{mysql,postgres,sqlite}.sql:109-110, 120-121, 123-124`
- **description:** The `AuditRecord` struct in `audit-schema.md` § 2 uses field names `before` and `after` for the snapshot columns, and the generic schema layout in `audit-schema.md` § 14 (lines 442-443) likewise uses `before JSONB NULL, after JSONB NULL`. But the canonical DDLs in all three shipped backends name these columns `before_snapshot` and `after_snapshot`. The normative spec and the canonical DDL disagree on the column name every storage adapter must emit.
- **expected:** Per the normative spec, the columns are `before` and `after` (`audit-schema.md:51-52`: `before: Option<Value>, // pre-mutation snapshot` and `before JSONB NULL,` at `audit-schema.md:442`).
- **evidence:**
  - `docs/schemas/audit-schema.md:51-52` — `before:          Option<Value>,     // pre-mutation snapshot` / `after:           Option<Value>,     // post-mutation snapshot`
  - `docs/schemas/audit-schema.md:442-443` — `before          JSONB NULL,` / `after           JSONB NULL,`
  - `migrations/engine/0000_engine_core.mysql.sql:109-110` — `before_snapshot JSON             NULL,` / `after_snapshot  JSON             NULL,`
  - `migrations/engine/0000_engine_core.postgres.sql:120-121` — `before_snapshot JSONB            NULL,` / `after_snapshot  JSONB            NULL,`
  - `migrations/engine/0000_engine_core.sqlite.sql:123-124` — `before_snapshot TEXT             NULL,` / `after_snapshot  TEXT             NULL,`

---

### FINDING 2

- **id:** DOC-SCHM-002
- **area:** documentation
- **severity:** High
- **location:** `migrations/engine/0000_engine_core.surreal.surql:130-141` vs `docs/schemas/audit-schema.md:41-57`
- **description:** The SurrealDB canonical DDL defines the `audit_log` table with a substantially different shape than the normative spec. The spec requires `actor_type`, `resource_type`, `resource_id`, `command_id`, `recorded_at`, `ip`, `user_agent`, `session_id`, `source`, `cross_tenant`; the canonical Surreal file renames `resource_type`/`resource_id` to `target_type`/`target_id`, stores `before`/`after` as `bytes` instead of `JSON`/`JSONB`, and omits `actor_type`, `command_id`, `recorded_at`, `ip`, `user_agent`, `session_id`, `source`, `cross_tenant`. It also adds `active_status` which is not in any other dialect.
- **expected:** Per the normative spec, the `audit_log` table contains `actor_type, action, resource_type, resource_id, event_id, command_id, correlation_id, occurred_at, recorded_at, ip, user_agent, session_id, before, after, metadata, cross_tenant, source` (`docs/schemas/audit-schema.md:35-57`).
- **evidence:**
  - `migrations/engine/0000_engine_core.surreal.surql:130-141`:
    ```
    DEFINE FIELD school_id       ON TABLE audit_log TYPE option<uuid>;
    DEFINE FIELD actor_id        ON TABLE audit_log TYPE uuid              ASSERT $value != NONE;
    DEFINE FIELD action          ON TABLE audit_log TYPE string            ASSERT $value != NONE AND string::len($value) <= 191;
    DEFINE FIELD target_type     ON TABLE audit_log TYPE string            ASSERT $value != NONE AND string::len($value) <= 64;
    DEFINE FIELD target_id       ON TABLE audit_log TYPE uuid              ASSERT $value != NONE;
    DEFINE FIELD before          ON TABLE audit_log TYPE option<bytes>;
    DEFINE FIELD after           ON TABLE audit_log TYPE option<bytes>;
    DEFINE FIELD event_id        ON TABLE audit_log TYPE option<uuid>;
    DEFINE FIELD correlation_id  ON TABLE audit_log TYPE uuid              ASSERT $value != NONE;
    DEFINE FIELD occurred_at     ON TABLE audit_log TYPE datetime          ASSERT $value != NONE;
    DEFINE FIELD active_status   ON TABLE audit_log TYPE string            ASSERT $value != NONE;
    DEFINE FIELD metadata        ON TABLE audit_log TYPE option<object>;
    ```
  - The file's own header comment at line 121-126 admits: `"which had fields like audit_id, actor_type, resource_type, ip, user_agent, etc. that did not exist on the storage-port struct"`.

---

### FINDING 3

- **id:** DOC-SCHM-003
- **area:** documentation
- **severity:** Critical
- **location:** `migrations/engine/0000_engine_core.surreal.surql:83, 130, 197` vs `migrations/engine/0000_engine_core.{mysql,postgres,sqlite}.sql:59, 65, 68`
- **description:** The SurrealDB canonical DDL declares `school_id` as `option<uuid>` (NULL) on `outbox`, `audit_log`, and `event_log`, but every other dialect emits `school_id` as `NOT NULL`. The engine invariant requires every tenant-scoped row to carry a non-null `school_id` (`docs/schemas/database-schema.md:48-58`); the canonical SurrealDB file violates this for all three of the cross-cutting tables that need it.
- **expected:** `school_id` is `NOT NULL` on every aggregate and on every cross-cutting table per `docs/schemas/database-schema.md:48-58` and the relational canonical DDLs.
- **evidence:**
  - `migrations/engine/0000_engine_core.surreal.surql:83` — `DEFINE FIELD school_id       ON TABLE outbox TYPE option<uuid>;`
  - `migrations/engine/0000_engine_core.surreal.surql:130` — `DEFINE FIELD school_id       ON TABLE audit_log TYPE option<uuid>;`
  - `migrations/engine/0000_engine_core.surreal.surql:197` — `DEFINE FIELD school_id       ON TABLE event_log TYPE option<uuid>;`
  - `migrations/engine/0000_engine_core.mysql.sql:59` — `school_id       CHAR(36)     NOT NULL,`
  - `migrations/engine/0000_engine_core.postgres.sql:65` — `school_id       UUID         NOT NULL,`
  - `migrations/engine/0000_engine_core.sqlite.sql:68` — `school_id       TEXT         NOT NULL CHECK (length(school_id) = 36),`

---

### FINDING 4

- **id:** DOC-SCHM-004
- **area:** documentation
- **severity:** Critical
- **location:** `migrations/engine/0000_engine_core.surreal.surql:167-175` vs `migrations/engine/0000_engine_core.{mysql,postgres,sqlite}.sql:132-141, 148-160, 151-163`
- **description:** The SurrealDB canonical `idempotency` table is missing `command_id` and `expires_at` (both present in the three relational canonical DDLs and required by the `Idempotency` port contract). It also defines `outcome` as `bytes` whereas the relational canonical DDLs use `JSON` / `JSONB` / `TEXT` with `json_valid` / `jsonb_typeof` CHECKs, and it stores `outcome_version` as an `int` (no analogue in the relational canonical DDLs).
- **expected:** Per `migrations/engine/0000_engine_core.mysql.sql:131-141` and the engine spec `docs/schemas/command-schema.md:148-167`, `idempotency` has columns `school_id, command_type, idempotency_key, command_id, outcome, recorded_at, expires_at`.
- **evidence:**
  - `migrations/engine/0000_engine_core.surreal.surql:167-175`:
    ```
    DEFINE FIELD school_id              ON TABLE idempotency TYPE uuid                ASSERT $value != NONE;
    DEFINE FIELD command_type           ON TABLE idempotency TYPE string              ASSERT $value != NONE AND string::len($value) <= 191;
    DEFINE FIELD idempotency_key        ON TABLE idempotency TYPE uuid                ASSERT $value != NONE;
    DEFINE FIELD outcome                ON TABLE idempotency TYPE bytes               ASSERT $value != NONE;
    DEFINE FIELD outcome_version        ON TABLE idempotency TYPE int                 ASSERT $value != NONE;
    DEFINE FIELD recorded_at            ON TABLE idempotency TYPE datetime            ASSERT $value != NONE;
    DEFINE FIELD affected_aggregate_ids ON TABLE idempotency TYPE option<array<uuid>>;
    ```
  - `migrations/engine/0000_engine_core.mysql.sql:131-141` includes `command_id CHAR(36) NOT NULL` and `expires_at TIMESTAMP NOT NULL` — neither of which exists in the Surreal canonical.

---

### FINDING 5

- **id:** DOC-SCHM-005
- **area:** documentation
- **severity:** High
- **location:** `migrations/engine/0000_engine_core.surreal.surql:193-213` vs `migrations/engine/0000_engine_core.{mysql,postgres,sqlite}.sql:152-170, 171-185, 174-188`
- **description:** The SurrealDB canonical `event_log` uses `schema_version` instead of `event_version`, stores `payload` as `bytes` instead of `JSON`/`JSONB`/`TEXT` with JSON CHECK, and adds an `active_status` field that does not exist in any other dialect's `event_log`. The field rename breaks the contract documented in `docs/schemas/event-schema.md:13-29` (`event_version`) and the engine's typed events.
- **expected:** `event_version` per the spec (`docs/schemas/event-schema.md:16`) and the three relational canonical DDLs; `payload` as JSON-typed.
- **evidence:**
  - `migrations/engine/0000_engine_core.surreal.surql:199` — `DEFINE FIELD schema_version  ON TABLE event_log TYPE int      ASSERT $value != NONE;`
  - `migrations/engine/0000_engine_core.surreal.surql:207` — `DEFINE FIELD payload         ON TABLE event_log TYPE bytes    ASSERT $value != NONE;`
  - `migrations/engine/0000_engine_core.surreal.surql:208` — `DEFINE FIELD active_status   ON TABLE event_log TYPE string   ASSERT $value != NONE;`
  - `migrations/engine/0000_engine_core.mysql.sql:155` — `event_version   INT          NOT NULL,`
  - `migrations/engine/0000_engine_core.mysql.sql:164` — `payload         JSON         NOT NULL,`

---

### FINDING 6

- **id:** DOC-SCHM-006
- **area:** documentation
- **severity:** High
- **location:** `docs/schemas/event-schema.md:251-263`
- **description:** The outbox schema in `event-schema.md` § 8 lists only 9 fields (`event_id`, `event_type`, `event_version`, `school_id`, `payload`, `enqueued_at`, `published_at`, `attempts`, `last_error`) but every canonical DDL defines 17 columns including `aggregate_id`, `aggregate_type`, `actor_id`, `correlation_id`, `causation_id`, `occurred_at`, `recorded_at`. The spec is incomplete and any consumer implementing against the spec alone will omit 8 mandatory columns.
- **expected:** The full outbox column list as in the relational canonical DDLs (`migrations/engine/0000_engine_core.mysql.sql:55-77`).
- **evidence:**
  - `docs/schemas/event-schema.md:251-263`:
    ```
    outbox(
        event_id        EventId       PK,
        event_type      VARCHAR,
        event_version   INT,
        school_id       SchoolId,
        payload         JSON,
        enqueued_at     TIMESTAMP,
        published_at    TIMESTAMP     NULL,
        attempts        INT           DEFAULT 0,
        last_error      TEXT          NULL
    )
    ```
  - `migrations/engine/0000_engine_core.mysql.sql:55-77` defines the same table with 17 columns, including `aggregate_id`, `aggregate_type`, `actor_id`, `correlation_id`, `causation_id`, `occurred_at`, `recorded_at` — none of which appear in the spec.

---

### FINDING 7

- **id:** DOC-SCHM-007
- **area:** documentation
- **severity:** Critical
- **location:** `docs/schemas/audit-schema.md:334-342` vs `migrations/engine/0000_engine_core.postgres.sql:104-137`
- **description:** The PostgreSQL partitioning example in `audit-schema.md` § 13.1 declares `audit_log` as `PARTITION BY RANGE (school_id, date_trunc('month', occurred_at))` with a composite PRIMARY KEY `(school_id, occurred_at, audit_id)` and a per-school, per-month partition naming convention. The canonical PG DDL has none of these: it declares `PRIMARY KEY (audit_id)` as a single-column key and emits no `PARTITION BY` clause, no `pg_cron` rotation, and no per-school partition scheme. The spec describes a feature the canonical DDL does not implement.
- **expected:** Canonical PG DDL implements the partitioning scheme documented in `docs/schemas/audit-schema.md:326-359`.
- **evidence:**
  - `docs/schemas/audit-schema.md:334-342`:
    ```sql
    CREATE TABLE audit_log (
        audit_id        UUID NOT NULL,
        school_id       UUID NOT NULL,
        -- ... other columns ...
        occurred_at     TIMESTAMP NOT NULL,
        PRIMARY KEY (school_id, occurred_at, audit_id)
    ) PARTITION BY RANGE (school_id, date_trunc('month', occurred_at));
    ```
  - `migrations/engine/0000_engine_core.postgres.sql:104-137` has `PRIMARY KEY (audit_id)` (single-column) and no `PARTITION BY` clause anywhere in the file.
  - Additionally, the spec example at line 339 declares `occurred_at TIMESTAMP NOT NULL` whereas the canonical at line 115 uses `TIMESTAMPTZ` — the spec example is not even dialect-correct.

---

### FINDING 8

- **id:** DOC-SCHM-008
- **area:** documentation
- **severity:** Critical
- **location:** `docs/schemas/audit-schema.md:368-377` vs `migrations/engine/0000_engine_core.mysql.sql:93-120`
- **description:** The MySQL partitioning example in `audit-schema.md` § 13.2 declares `audit_log` with `BINARY(16)` id types, `DATETIME(6)` timestamps, composite PRIMARY KEY `(school_id, occurred_at, audit_id)`, and `PARTITION BY KEY (school_id) PARTITIONS 12`. The canonical MySQL DDL uses `CHAR(36)` ids, `TIMESTAMP` (not `DATETIME(6)`), single-column `PRIMARY KEY (audit_id)`, and no partitioning. Every column type and structural choice in the spec example disagrees with the canonical DDL.
- **expected:** Canonical MySQL DDL implements the partitioning scheme documented in `docs/schemas/audit-schema.md:360-398`.
- **evidence:**
  - `docs/schemas/audit-schema.md:368-377`:
    ```sql
    CREATE TABLE audit_log (
        audit_id        BINARY(16) NOT NULL,
        school_id       BINARY(16) NOT NULL,
        -- ... other columns ...
        occurred_at     DATETIME(6) NOT NULL,
        PRIMARY KEY (school_id, occurred_at, audit_id)
    ) ENGINE=InnoDB
      PARTITION BY KEY (school_id) PARTITIONS 12;
    ```
  - `migrations/engine/0000_engine_core.mysql.sql:93-120` uses `CHAR(36)` (lines 94-95), `TIMESTAMP` (line 104), `PRIMARY KEY (audit_id)` (line 114), and contains no `PARTITION BY` clause.

---

### FINDING 9

- **id:** DOC-SCHM-009
- **area:** documentation
- **severity:** High
- **location:** `docs/schemas/sql-dialects/postgresql.md:122-160` vs `migrations/engine/0000_engine_core.postgres.sql:1-240`
- **description:** The PostgreSQL dialect spec declares "PG has the most expressive RLS of the three backends. The engine **requires** RLS as a defense-in-depth" and shows the canonical `CREATE POLICY` SQL for `school_isolation_<aggregate>`. The canonical PG DDL file emits zero `ALTER TABLE ... ENABLE ROW LEVEL SECURITY` statements and zero `CREATE POLICY` statements — none of the 6 cross-cutting tables have RLS. The spec requires a feature the canonical DDL does not implement.
- **expected:** Per `docs/schemas/sql-dialects/postgresql.md:122-160`, the canonical PG DDL emits `ALTER TABLE ... ENABLE ROW LEVEL SECURITY`, `FORCE ROW LEVEL SECURITY`, and `CREATE POLICY school_isolation_<aggregate> ...` for every aggregate.
- **evidence:**
  - `docs/schemas/sql-dialects/postgresql.md:124-140`:
    ```sql
    ALTER TABLE "<aggregate>" ENABLE ROW LEVEL SECURITY;
    ALTER TABLE "<aggregate>" FORCE ROW LEVEL SECURITY;
    CREATE POLICY "school_isolation_<aggregate>" ON "<aggregate>"
      USING ("school_id" = current_setting('app.current_school_id')::UUID)
      WITH CHECK ("school_id" = current_setting('app.current_school_id')::UUID);
    ```
  - `migrations/engine/0000_engine_core.postgres.sql` (entire 240-line file) contains zero matches for `ROW LEVEL SECURITY`, `FORCE ROW LEVEL`, or `CREATE POLICY`.

---

### FINDING 10

- **id:** DOC-SCHM-010
- **area:** documentation
- **severity:** High
- **location:** `docs/schemas/sql-dialects/comparison.md:192` vs `migrations/engine/0000_engine_core.postgres.sql:104-137`
- **description:** The cross-cutting-table feature availability table claims PG `audit_log` is `"Native (with RLS, append-only role)"`, but the canonical PG DDL contains neither RLS nor an `INSERT`-only role grant. The comparison table claims a feature the canonical DDL does not ship.
- **expected:** Per the comparison table, PG canonical `audit_log` should include RLS (`CREATE POLICY`) and an append-only role setup.
- **evidence:**
  - `docs/schemas/sql-dialects/comparison.md:193` — `| audit_log | Native | Native | Native (with RLS, append-only role) |`
  - `migrations/engine/0000_engine_core.postgres.sql:104-137` contains no `ROW LEVEL SECURITY`, no `CREATE POLICY`, and no `GRANT` statements.

---

### FINDING 11

- **id:** DOC-SCHM-011
- **area:** documentation
- **severity:** Medium
- **location:** `docs/schemas/database-schema.md:121, 183, 220`
- **description:** `database-schema.md` is internally inconsistent about the canonical type for `etag`. § 5 (line 121) lists `etag CHAR(32) / hash`; § 9 (line 183) lists `etag BINARY(16) / hash`; § 11 (line 220) emits `etag BINARY(16) NOT NULL`. The same normative document gives three different recommendations.
- **expected:** A single canonical type for `etag` (consistent with `database-schema.md` being normative per `audit-schema.md` and `event-schema.md` precedent).
- **evidence:**
  - `docs/schemas/database-schema.md:121` — `| etag | CHAR(32) / hash | Content hash for conflict resolution. See § 9. |`
  - `docs/schemas/database-schema.md:183` — `| etag | BINARY(16) / hash | Content-addressed hash of the row's mutable fields. Used for client-side conflict check. |`
  - `docs/schemas/database-schema.md:220` — `etag            BINARY(16)     NOT NULL,`

---

### FINDING 12

- **id:** DOC-SCHM-012
- **area:** documentation
- **severity:** Medium
- **location:** `docs/schemas/database-schema.md:220` vs `docs/schemas/sql-dialects/{mysql,postgresql,sqlite,surrealdb}.md`
- **description:** The canonical minimum schema in `database-schema.md` § 11 declares `etag BINARY(16)`, but every per-dialect aggregate-table example uses `CHAR(32)` (or its dialect equivalent). The aggregate examples are the implementation guidance; the canonical minimum schema is the spec. They disagree.
- **expected:** `etag CHAR(32)` (and dialect equivalents) on aggregate tables, consistent with the per-dialect examples and the engine's UUIDv7 byte-for-byte hashing rationale.
- **evidence:**
  - `docs/schemas/database-schema.md:220` — `etag            BINARY(16)     NOT NULL,`
  - `docs/schemas/sql-dialects/mysql.md:263` — ``etag`            CHAR(32)     NOT NULL,`
  - `docs/schemas/sql-dialects/postgresql.md:415` — `"etag"              CHAR(32)     NOT NULL,`
  - `docs/schemas/sql-dialects/sqlite.md:363` — `"etag"              TEXT NOT NULL,` (with length-32 CHECK)
  - `docs/schemas/sql-dialects/surrealdb.md:710-711` — `DEFINE FIELD etag            ON TABLE academic_students TYPE string ASSERT string::length($value) = 32;`

---

### FINDING 13

- **id:** DOC-SCHM-013
- **area:** documentation
- **severity:** Medium
- **location:** `docs/schemas/sql-dialects/postgresql.md:283` vs `migrations/engine/0000_engine_core.postgres.sql:117`
- **description:** The PostgreSQL dialect spec example for `audit_log.ip` uses `INET` (PG's native IP-address type), but the canonical PG DDL declares `ip VARCHAR(45) NULL`. A consumer implementing from the spec will emit `INET`; the canonical DDL emitted at startup emits `VARCHAR(45)`. The two are not equivalent (`INET` validates IP syntax; `VARCHAR(45)` accepts any 45-char string).
- **expected:** Either the canonical PG DDL uses `INET` to match the spec, or the spec is updated to `VARCHAR(45)`.
- **evidence:**
  - `docs/schemas/sql-dialects/postgresql.md:283` — `"ip"              INET,`
  - `migrations/engine/0000_engine_core.postgres.sql:117` — `ip              VARCHAR(45)     NULL,`
  - `migrations/engine/0000_engine_core.mysql.sql:106` — `ip              VARCHAR(45)     NULL,` (also `VARCHAR(45)`)

---

### FINDING 14

- **id:** DOC-SCHM-014
- **area:** documentation
- **severity:** Medium
- **location:** `migrations/engine/0000_engine_core.postgres.sql:73, 120-122, 153, 183, 208` vs `docs/schemas/sql-dialects/postgresql.md:249, 286-288, 314, 339, 359`
- **description:** The PostgreSQL dialect spec declares `JSONB` columns must carry `CHECK (jsonb_typeof(...) = 'object')` constraints (e.g. `outbox.payload`, `audit_log.before_snapshot`, `audit_log.after_snapshot`, `audit_log.metadata`, `idempotency.outcome`, `event_log.payload`, `schema_registry.schema_json`). The canonical PG DDL file emits zero such CHECK constraints — every JSONB column is declared `NULL` or `NOT NULL` with no JSON-shape validation. The spec mandates a constraint the canonical DDL omits.
- **expected:** Per `docs/schemas/sql-dialects/postgresql.md:58` — `"native JSONB; engine emits JSONB NOT NULL CHECK (jsonb_typeof(\"payload\") = 'object')"`.
- **evidence:**
  - `docs/schemas/sql-dialects/postgresql.md:249` — `"payload"         JSONB        NOT NULL CHECK (jsonb_typeof("payload") = 'object'),`
  - `docs/schemas/sql-dialects/postgresql.md:286` — `"before_snapshot" JSONB        CHECK ("before_snapshot" IS NULL OR jsonb_typeof("before_snapshot") = 'object'),`
  - `migrations/engine/0000_engine_core.postgres.sql:73` — `payload         JSONB        NOT NULL,` (no CHECK)
  - `migrations/engine/0000_engine_core.postgres.sql:120` — `before_snapshot JSONB            NULL,` (no CHECK)

---

### FINDING 15

- **id:** DOC-SCHM-015
- **area:** documentation
- **severity:** Medium
- **location:** `migrations/engine/0000_engine_core.sqlite.sql:76, 123-125, 156, 186, 211` vs `docs/schemas/sql-dialects/sqlite.md:202, 238-240, 265, 290, 309`
- **description:** The SQLite dialect spec declares every JSON-typed column must carry `CHECK (json_valid(...))` (or `json_valid(...) IS NULL OR json_valid(...)` for nullable). The canonical SQLite DDL emits zero such CHECK constraints — every JSON column is plain `TEXT NOT NULL` / `TEXT NULL`.
- **expected:** Per `docs/schemas/sql-dialects/sqlite.md:202` — `"payload"         TEXT NOT NULL CHECK (json_valid("payload")),`.
- **evidence:**
  - `docs/schemas/sql-dialects/sqlite.md:202` — `"payload"         TEXT NOT NULL CHECK (json_valid("payload")),`
  - `docs/schemas/sql-dialects/sqlite.md:238` — `"before_snapshot" TEXT CHECK ("before_snapshot" IS NULL OR json_valid("before_snapshot")),`
  - `migrations/engine/0000_engine_core.sqlite.sql:76` — `payload         TEXT         NOT NULL,` (no CHECK)
  - `migrations/engine/0000_engine_core.sqlite.sql:123-125` — `before_snapshot TEXT             NULL, after_snapshot  TEXT             NULL, metadata        TEXT             NULL,` (no CHECKs)

---

### FINDING 16

- **id:** DOC-SCHM-016
- **area:** documentation
- **severity:** Medium
- **location:** `docs/schemas/sql-dialects/surrealdb.md:30, 47-49` vs `migrations/engine/0000_engine_core.surreal.surql:71-260`
- **description:** The SurrealDB dialect spec says "Use **backticks** for every identifier" (line 30) and "SurrealDB accepts both backticks and double quotes for identifier quoting. The engine uses backticks" (lines 46-49). The canonical SurrealDB DDL file uses no quoting at all on any identifier — table names, field names, index names, and column references are all bare (unquoted).
- **expected:** Per the spec, every identifier in the canonical SurrealDB DDL is backtick-quoted.
- **evidence:**
  - `docs/schemas/sql-dialects/surrealdb.md:30` — `Use **backticks** for every identifier:`
  - `docs/schemas/sql-dialects/surrealdb.md:47-49` — `SurrealDB accepts both backticks and double quotes for identifier quoting. The engine uses backticks to match the MySQL adapter`
  - `migrations/engine/0000_engine_core.surreal.surql:71` — `DEFINE TABLE outbox SCHEMAFULL` (no backticks)
  - `migrations/engine/0000_engine_core.surreal.surql:74` — `DEFINE FIELD event_id        ON TABLE outbox TYPE uuid     ASSERT $value != NONE;` (no backticks)
  - `migrations/engine/0000_engine_core.surreal.surql:97` — `DEFINE INDEX idx_outbox_event_id        ON TABLE outbox COLUMNS event_id      UNIQUE;` (no backticks)

---

### FINDING 17

- **id:** DOC-SCHM-017
- **area:** documentation
- **severity:** Low
- **location:** `docs/schemas/sql-dialects/comparison.md:246` vs `migrations/engine/0000_engine_core.surreal.surql`
- **description:** The SurrealDB feature-comparison row claims `"Identifier quoting | backticks"`, but the canonical SurrealDB DDL file does not use backticks (see Finding 16). The comparison table is out of sync with the canonical artifact.
- **expected:** The comparison row matches the canonical quoting convention used in the DDL.
- **evidence:**
  - `docs/schemas/sql-dialects/comparison.md:246` — `| Identifier quoting | backticks | double-quotes | double-quotes | backticks |` (SurrealDB column)
  - `migrations/engine/0000_engine_core.surreal.surql` (full file) — no backtick characters appear anywhere in the DDL.

---

### FINDING 18

- **id:** DOC-SCHM-018
- **area:** documentation
- **severity:** High
- **location:** `docs/schemas/sql-dialects/postgresql.md:16, 32-34` vs `migrations/engine/0000_engine_core.postgres.sql:61, 104, 148, 171, 205, 226`
- **description:** The PG dialect spec example shows `CREATE TABLE "outbox" (...)` with the bare table name (no schema prefix). The canonical PG DDL wraps all 6 tables in an `engine` schema (`CREATE TABLE IF NOT EXISTS engine.outbox (...)`, `engine.audit_log`, `engine.idempotency`, `engine.event_log`, `engine.schema_registry`, `engine.system_user`). The example contradicts the canonical form, and the engine contract claim "the same table name in all backends" (`docs/schemas/sql-dialects/README.md:65`) is broken — PG sees `engine.outbox` while MySQL and SQLite see `outbox`.
- **expected:** The dialect spec example matches the canonical PG DDL (table names qualified with `engine.`) or the canonical DDL drops the `engine.` prefix.
- **evidence:**
  - `docs/schemas/sql-dialects/postgresql.md:16` — `CREATE TABLE "outbox" ( "event_id" UUID NOT NULL, ... );`
  - `docs/schemas/sql-dialects/postgresql.md:32-34` — `CREATE TABLE "outbox" ( ... );`
  - `migrations/engine/0000_engine_core.postgres.sql:61` — `CREATE TABLE IF NOT EXISTS engine.outbox (`
  - `migrations/engine/0000_engine_core.postgres.sql:104` — `CREATE TABLE IF NOT EXISTS engine.audit_log (`
  - `migrations/engine/0000_engine_core.postgres.sql:148` — `CREATE TABLE IF NOT EXISTS engine.idempotency (`
  - `migrations/engine/0000_engine_core.postgres.sql:171` — `CREATE TABLE IF NOT EXISTS engine.event_log (`
  - `migrations/engine/0000_engine_core.postgres.sql:205` — `CREATE TABLE IF NOT EXISTS engine.schema_registry (`
  - `migrations/engine/0000_engine_core.postgres.sql:226` — `CREATE TABLE IF NOT EXISTS engine.system_user (`

---

### FINDING 19

- **id:** DOC-SCHM-019
- **area:** documentation
- **severity:** Medium
- **location:** `migrations/engine/0000_engine_core.postgres.sql:229` vs `docs/schemas/database-schema.md:58` and `docs/schemas/sql-dialects/postgresql.md:50`
- **description:** The canonical PG DDL declares `system_user.active_status` as `SMALLINT NOT NULL DEFAULT 1`, but `database-schema.md` § 2 specifies `active_status TINYINT / BOOLEAN no` (the per-dialect mapping in `postgresql.md` § Type mapping says `TINYINT` maps to `SMALLINT (with CHECK range)`, or `BOOLEAN` for booleans). Using `SMALLINT` for a boolean-style flag is unusual and disagrees with the per-dialect type mapping.
- **expected:** `system_user.active_status` is `BOOLEAN NOT NULL DEFAULT TRUE` per the dialect mapping (consistent with PG's native boolean type and the SQLite `INTEGER ... CHECK IN (0,1)` form).
- **evidence:**
  - `migrations/engine/0000_engine_core.postgres.sql:229` — `active_status SMALLINT     NOT NULL DEFAULT 1,`
  - `docs/schemas/database-schema.md:58` — `| active_status | TINYINT / BOOLEAN | no |`
  - `docs/schemas/sql-dialects/postgresql.md:50` — `| TINYINT | SMALLINT (with CHECK range) | engine uses BOOLEAN for booleans and SMALLINT for 1-byte ints |`
  - `migrations/engine/0000_engine_core.mysql.sql:205` — `active_status TINYINT     NOT NULL DEFAULT 1,` (different from PG)
  - `migrations/engine/0000_engine_core.sqlite.sql:232` — `active_status INTEGER      NOT NULL DEFAULT 1 CHECK (active_status IN (0, 1)),`

---

### FINDING 20

- **id:** DOC-SCHM-020
- **area:** documentation
- **severity:** Low
- **location:** `docs/schemas/sql-dialects/mysql.md:51`
- **description:** The MySQL type-mapping table says `CHAR(36) (UUIDv7) | CHAR(36) | utf8mb4 charset; 36 chars = 36 bytes (4-byte chars)`. The parenthetical "(4-byte chars)" implies a UUID character occupies 4 bytes in `utf8mb4`. A UUID is ASCII (digits 0-9, letters a-f, hyphens), so each character occupies 1 byte in utf8mb4; the storage cost is 36 bytes, not 144. The parenthetical is misleading and contradicts the canonical 36-byte storage claim.
- **expected:** A correct note such as `36 chars = 36 bytes (UUIDs are pure ASCII, so each char is 1 byte even in utf8mb4)`.
- **evidence:**
  - `docs/schemas/sql-dialects/mysql.md:51` — `| CHAR(36) (UUIDv7) | CHAR(36) | utf8mb4 charset; 36 chars = 36 bytes (4-byte chars) |`

---

### FINDING 21

- **id:** DOC-SCHM-021
- **area:** documentation
- **severity:** Medium
- **location:** `docs/schemas/data-migration/02-id-conversion.md:78-86` vs `docs/schemas/data-migration/02-id-conversion.md:79-82` and `migrations/engine/0000_engine_core.{mysql,postgres,sqlite}.sql`
- **description:** The `uuid_v7(namespace, legacy_id)` derivation formula in § 02 uses a "fixed engine epoch — the engine's first commit timestamp, e.g. 2026-01-01T00:00:00.000Z" as the UUIDv7 timestamp component. But the engine's `system_user` row in every canonical DDL uses the literal id `'00000000-0000-7000-8000-000000000001'` — which is not a valid UUIDv7 (it does not encode a `2026-01-01` timestamp in the high 48 bits; it is an all-zero + variant-`7` constant). Two engines implementing per spec will derive different `id_v7_legacy`-based ids depending on which constant they treat as the "engine epoch".
- **expected:** Either the spec names the fixed epoch explicitly and uses a real UUIDv7 encoding of it, or it notes that the engine `system_user` id is an out-of-band constant that does not follow the `uuid_v7()` derivation.
- **evidence:**
  - `docs/schemas/data-migration/02-id-conversion.md:78-86`:
    ```
    uuid_v7(namespace, legacy_id) = UUIDv7(
        timestamp = <a fixed "engine epoch" — the engine's first commit
                    timestamp, e.g. 2026-01-01T00:00:00.000Z>,
        sub_ms    = (legacy_id % 4096),
        rand_a    = (legacy_id >> 12) & 0xFFF,
        rand_b    = blake3(namespace || legacy_id)[0..62 bits]
    )
    ```
  - `migrations/engine/0000_engine_core.mysql.sql:214` — `VALUES ('00000000-0000-7000-8000-000000000001', 'SYSTEM', 1, UTC_TIMESTAMP(6));`
  - `migrations/engine/0000_engine_core.postgres.sql:239` — `VALUES ('00000000-0000-7000-8000-000000000001', 'SYSTEM', 1, NOW())`
  - `migrations/engine/0000_engine_core.sqlite.sql:241` — `VALUES ('00000000-0000-7000-8000-000000000001', 'SYSTEM', 1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));`

---

### FINDING 22

- **id:** DOC-SCHM-022
- **area:** documentation
- **severity:** Medium
- **location:** `docs/schemas/data-migration/03-domain-renames.md:120` vs `docs/schemas/data-migration/03-domain-renames.md:151` (dup claim)
- **description:** The assessment-domain table list declares `sm_online_exam_questions (dup)` as a drop (line 120). The same row appears nowhere else in the rename map (the academic domain treats `sm_online_exam_questions` as a kept table at line 118: `sm_online_exam_questions | academic_online_exam_questions`). The two domain lists disagree about whether this table is kept or dropped.
- **expected:** A single decision for `sm_online_exam_questions` — either kept (academic owns it) or dropped (assessment owns the canonical form).
- **evidence:**
  - `docs/schemas/data-migration/03-domain-renames.md:118` (Academic domain) — `| sm_online_exam_questions | academic_online_exam_questions |`
  - `docs/schemas/data-migration/03-domain-renames.md:120` (Assessment domain) — `| sm_online_exam_questions (dup) | (drop; see academic) |`

---

### FINDING 23

- **id:** DOC-SCHM-023
- **area:** documentation
- **severity:** Medium
- **location:** `docs/schemas/data-migration/03-domain-renames.md:440-441` vs `docs/schemas/data-migration/03-domain-renames.md:36-75` and `03-domain-renames.md:113-130`
- **description:** The aggregate count at the end of `03-domain-renames.md` says `total = 320` (290 renames + 7 archives + 10 drops + 6 keep + 7 consumer-side adds), but summing the per-domain rename lists shows: Platform 38 + Academic 50 + Assessment 43 + Attendance 7 + Communication 23 + Documents 3 + Events 7 + Facilities 15 + Finance 47 + HR 14 + Library 4 + CMS 20 + RBAC 10 + Settings 14 + Operations 15 = 310 renames alone (plus 7 archives and 7 consumer-side adds). The 310-renames figure exceeds the 290-rename figure in the same file.
- **expected:** The aggregate-count table matches the sum of the per-domain rename lists.
- **evidence:**
  - `docs/schemas/data-migration/03-domain-renames.md:440-441` — `| rename | ~290 |` / `| **total** | **320** |`
  - `docs/schemas/data-migration/03-domain-renames.md:36-414` (per-domain lists): Platform 38 (line 35) + Academic 50 (line 77) + Assessment 43 (line 133) + Attendance 7 (line 180) + Communication 23 (line 192) + Documents 3 (line 220) + Events 7 (line 228) + Facilities 15 (line 240) + Finance 47 (line 260) + HR 14 (line 311) + Library 4 (line 330) + CMS 20 (line 339) + RBAC 10 (line 363) + Settings 14 (line 378) + Operations 15 (line 397) = 310 rename rows.

---

### FINDING 24

- **id:** DOC-SCHM-024
- **area:** documentation
- **severity:** Low
- **location:** `docs/schemas/data-migration/03-domain-renames.md:396`
- **description:** The Settings-domain rename list declares `transcations (typo) | (drop; legacy table was empty per 0009_finance.sql)`. The entry is listed under the Settings domain, but `transcations` is a finance-domain typo (a `transactions` table in the `migrations/0009_finance.sql` file) and is logically part of the Finance-domain rename map, not Settings.
- **expected:** The typo drop is documented under the Finance domain or in the dedicated typo-fix section (`05-brand-removal.md:158`), not under Settings.
- **evidence:**
  - `docs/schemas/data-migration/03-domain-renames.md:394` — `| transcations (typo) | (drop; legacy table was empty per 0009_finance.sql) |`
  - The same typo is also documented in `docs/schemas/data-migration/05-brand-removal.md:161` — `| transcations (table, 1 occurrence in migrations/0009_finance.sql) | (drop; table is empty) |`.

---

### FINDING 25

- **id:** DOC-SCHM-025
- **area:** documentation
- **severity:** Medium
- **location:** `docs/schemas/data-migration/04-column-additions.md:11-23`
- **description:** The "engine invariant columns" table in § 04 is titled "the seven engine-invariant columns" but lists 10 rows (`created_at`, `updated_at`, `created_by`, `updated_by`, `active_status`, `version`, `etag`, `last_event_id`, `correlation_id`, `source`). The header comment at lines 24-28 acknowledges the discrepancy ("That's 10 columns; the user's earlier summary said 6 NEW + 4 existing"), but the table is still titled "seven" and the count `6 NEW` in the aggregate count at line 172 (`Columns added per table | 6 NEW`) is itself stale against the 10-column list. The doc has not been corrected.
- **expected:** A consistent count (10 columns total, ~6 NEW) presented in the table header and the aggregate count.
- **evidence:**
  - `docs/schemas/data-migration/04-column-additions.md:9` — `## The seven engine-invariant columns`
  - `docs/schemas/data-migration/04-column-additions.md:11-23` — table with 10 rows
  - `docs/schemas/data-migration/04-column-additions.md:172` — `| Columns added per table | 6 NEW (legacy had created_at, updated_at, sometimes active_status) |`

---

### FINDING 26

- **id:** DOC-SCHM-026
- **area:** documentation
- **severity:** Medium
- **location:** `docs/schemas/data-migration/05-brand-removal.md:99-136` vs `docs/specs/rbac/aggregates.md`, `docs/specs/rbac/repositories.md`, `docs/specs/rbac/value-objects.md`, `docs/specs/rbac/tables.md`, `docs/specs/rbac/overview.md`, `docs/specs/rbac/commands.md`, `docs/specs/hr/aggregates.md`, `docs/specs/hr/tables.md`, `docs/specs/hr/commands.md`, `docs/specs/operations/aggregates.md`, `docs/specs/operations/commands.md`, `docs/research/rbac-analysis.md`
- **description:** The "Drops from docs" list in `05-brand-removal.md` specifies 16 specific doc edits (line ranges like "lines 189-216") that must be applied to remove `InfixRole` / `InfixPermissionAssign` / `is_saas` references from the spec tree. The expected target files (`docs/specs/rbac/*`, `docs/specs/hr/*`, `docs/specs/operations/*`, `docs/research/rbac-analysis.md`) reference 12 distinct file paths. None of those 16 edits can be verified because the spec-tree directories and files do not exist in the current repository.
- **expected:** Either the target spec files exist and the migration has been completed (in which case the doc should be updated to point at the new line numbers), or the spec tree is incomplete and the migration is not yet executable.
- **evidence:**
  - `docs/schemas/data-migration/05-brand-removal.md:99-136` enumerates 16 specific doc edits, each tied to a specific line number in a specific file:
    - `docs/specs/rbac/aggregates.md` lines 189-216, 218-244, 19, 32, 35-36
    - `docs/specs/rbac/repositories.md` lines 112-123, 125-135, 193
    - `docs/specs/rbac/value-objects.md` lines 18-19
    - `docs/specs/rbac/tables.md` lines 10-11, 37, 47
    - `docs/specs/rbac/overview.md` lines 80-81
    - `docs/specs/rbac/commands.md` line 32
    - `docs/specs/hr/aggregates.md` lines 92, 123
    - `docs/specs/hr/tables.md` line 38
    - `docs/specs/hr/commands.md` lines 184, 202
    - `docs/specs/operations/aggregates.md` line 265
    - `docs/specs/operations/commands.md` line 297
    - `docs/research/rbac-analysis.md` line 32
  - `docs/specs/` and `docs/research/` are listed in the AGENTS.md layout (`docs/specs/<domain>/`, `docs/research/`) but the per-domain spec files referenced by `05-brand-removal.md` do not exist as siblings of the doc.
  - The migration is presented as a Phase 5 to-do, not a done.

---

### FINDING 27

- **id:** DOC-SCHM-027
- **area:** documentation
- **severity:** Low
- **location:** `docs/schemas/data-migration/00-overview.md:49-60`
- **description:** The phase table in `00-overview.md` lists 10 numbered phases (0..8, plus rollback at "—" and security at "—"). The `README.md` index lists the same phases as 11 numbered phases plus the two "—" entries. The header text at line 44 says "eleven phases". The two doc files disagree on the total count.
- **expected:** The phase list and the index use the same count and the same phase numbers.
- **evidence:**
  - `docs/schemas/data-migration/00-overview.md:44` — `The migration runs in eleven phases, each with a focused file in this folder.`
  - `docs/schemas/data-migration/00-overview.md:47-60` (table) lists 10 numbered phases (0 through 8, plus 2 "—" entries)
  - `docs/schemas/data-migration/README.md:12-22` lists 12 numbered items (00 through 11) plus Rollback and Security

---

### FINDING 28

- **id:** DOC-SCHM-028
- **area:** documentation
- **severity:** Low
- **location:** `docs/schemas/data-migration/01-engine-tables.md:53-58`
- **description:** The "Apply order" section in `01-engine-tables.md` shows two bash commands: a `mysql` command for "MySQL / SQLite" that pipes `0000_engine_core.mysql.sql` into `devdb_v2`, and a `psql` command for "PostgreSQL" that does the same. Both commands feed the MySQL DDL to PostgreSQL. The file then notes "The PostgreSQL DDL differs only in identifier quoting (`outbox` vs `outbox`) and the `JSON` type vs `JSONB`", but feeds MySQL DDL to PG. The dialect mismatch is acknowledged in prose but the actual command is wrong.
- **expected:** The PostgreSQL apply command uses `0000_engine_core.postgres.sql`, not `0000_engine_core.mysql.sql`.
- **evidence:**
  - `docs/schemas/data-migration/01-engine-tables.md:49` — `mysql -u educore -p devdb_v2 < migrations/engine/0000_engine_core.mysql.sql`
  - `docs/schemas/data-migration/01-engine-tables.md:57` — `psql -U educore -d devdb_v2 -f migrations/engine/0000_engine_core.mysql.sql` (feeds MySQL DDL to psql)
  - `docs/schemas/data-migration/01-engine-tables.md:61-65` — `The PostgreSQL DDL differs only in identifier quoting ("outbox" vs `outbox`) and the JSON type vs JSONB.` (acknowledges dialect difference but still uses the MySQL file)

---

### FINDING 29

- **id:** DOC-SCHM-029
- **area:** documentation
- **severity:** Medium
- **location:** `docs/schemas/data-migration/01-engine-tables.md:71-86` vs `docs/schemas/data-migration/01-engine-tables.md:69-71`
- **description:** The "Verify Phase 1" section ends with `SHOW INDEX FROM outbox;` / `SHOW INDEX FROM audit_log;` / `SHOW INDEX FROM idempotency;`. These are MySQL-specific statements. The same section is meant to verify Phase 1 across MySQL/SQLite/PostgreSQL, but the verification commands are MySQL-only.
- **expected:** Per-dialect verification commands (or a note that the SQL shown is MySQL-specific and PostgreSQL/SQLite equivalents are required).
- **evidence:**
  - `docs/schemas/data-migration/01-engine-tables.md:83-85`:
    ```sql
    SHOW INDEX FROM outbox;
    SHOW INDEX FROM audit_log;
    SHOW INDEX FROM idempotency;
    ```
  - `SHOW INDEX FROM` is MySQL syntax; PG uses `\d+ outbox` in `psql`, SQLite uses `PRAGMA index_list('outbox');`.

---

### FINDING 30

- **id:** DOC-SCHM-030
- **area:** documentation
- **severity:** Low
- **location:** `docs/schemas/data-migration/05-brand-removal.md:50-87`
- **description:** The brand-removal doc lists 35 `module_toggles` columns being dropped from `sm_general_settings`. The `DROP COLUMN` statement ends at `InAppLiveClass` (line 86) but the introductory text says "This drops 35 columns". Counting the columns in the `DROP` list: `Lesson, Chat, FeesCollection, InfixBiometrics, ResultReports, TemplateSettings, MenuManage, RolePermission, RazorPay, Saas, StudentAbsentNotification, ParentRegistration, Zoom, BBB, VideoWatch, Jitsi, OnlineExam, SaasRolePermission, BulkPrint, HimalayaSms, XenditPayment, Wallet, Lms, ExamPlan, University, Gmeet, KhaltiPayment, Raudhahpay, AppSlider, BehaviourRecords, DownloadCenter, AiContent, WhatsappSupport, InAppLiveClass` = 34 columns.
- **expected:** Either 34 in the count or 35 in the `DROP` list.
- **evidence:**
  - `docs/schemas/data-migration/05-brand-removal.md:89-90` — `This drops 35 columns. The engine's module system is capability-based, not flag-based.`
  - `docs/schemas/data-migration/05-brand-removal.md:53-86` lists 34 `DROP COLUMN` statements (33 unique column names + `InfixBiometrics` which is also renamed).
  - `docs/schemas/data-migration/05-brand-removal.md:206` — `| Module-toggle flat-int columns dropped | 35 |` — repeats the 35 figure in the aggregate count.

---

### FINDING 31

- **id:** DOC-SCHM-031
- **area:** documentation
- **severity:** Medium
- **location:** `docs/schemas/data-migration/06-field-data-flow.md:111-113` vs `docs/schemas/data-migration/06-field-data-flow.md:179-180`
- **description:** The `academic_students` field map (table 1) declares `parent_id int FK sm_parents` becoming `guardian_id CHAR(36) FK academic_parents` (line 111), but the `users → platform_users` map (table 3) declares `role_id int(10) UNSIGNED NULL FK infix_roles (CASCADE)` becoming `CHAR(36) NOT NULL FK rbac_roles (RESTRICT)` (line 177-178) — the same column name `role_id` but in two different rows the `NOT NULL` semantics differ from the original. The doc does not flag this; both fields are rewritten as `NOT NULL` against a legacy nullable column.
- **expected:** Either keep `NULL` semantics (engine may not require every user to hold a role) or document the semantic change explicitly.
- **evidence:**
  - `docs/schemas/data-migration/06-field-data-flow.md:177-178` — `| role_id | int(10) UNSIGNED NULL FK infix_roles (CASCADE) | role_id | CHAR(36) NOT NULL FK rbac_roles (RESTRICT) | INT → UUIDv7; CASCADE → RESTRICT; tighten |`
  - Legacy `users.role_id` is nullable per the schema; engine forces it non-null with no documented backfill for users that have no role.

---

### FINDING 32

- **id:** DOC-SCHM-032
- **area:** documentation
- **severity:** Low
- **location:** `docs/schemas/data-migration/06-field-data-flow.md:270-280`
- **description:** The `sm_staffs → hr_staffs` field map (table 8) lists `is_saas int DEFAULT 0` becoming `is_saas_staff BOOLEAN NOT NULL DEFAULT FALSE`. Other parts of the same migration plan (`05-brand-removal.md:146-150`) rename `is_saas` to `is_replicated` on roles and `is_system_defined` on other aggregates. The HR-specific rename to `is_saas_staff` (preserving the brand-tainted prefix) is inconsistent with the broader migration direction.
- **expected:** A consistent rename rule for `is_saas` across all aggregates (the migration's stated direction is to drop the `is_saas` brand artifact).
- **evidence:**
  - `docs/schemas/data-migration/06-field-data-flow.md:280` — `| is_saas | int DEFAULT 0 | is_saas_staff | BOOLEAN NOT NULL DEFAULT FALSE | rename; tighten |`
  - `docs/schemas/data-migration/05-brand-removal.md:146-150` — `is_saas → is_replicated` (rbac_roles, rbac_permission_assigns), `is_system_defined` (hr_departments, hr_designations, etc.)
  - `is_saas_staff` is not referenced anywhere else in the migration plan.

---

### FINDING 33

- **id:** DOC-SCHM-033
- **area:** documentation
- **severity:** Medium
- **location:** `docs/schemas/data-migration/06-field-data-flow.md:436-444` vs `docs/schemas/data-migration/05-brand-removal.md:178-193`
- **description:** The end of `06-field-data-flow.md` shows the backfill query for `platform_packages.modules JSON ARRAYAGG(name)` reading from `sm_general_settings` flat-int columns. But `05-brand-removal.md:50-87` says those flat-int columns are dropped from `sm_general_settings` entirely (35 `DROP COLUMN` statements). The backfill in § 06 reads columns that § 05 drops — the two phases cannot both run as ordered.
- **expected:** Either the modules JSON backfill runs before the `DROP COLUMN` (and the migration ordering in `00-overview.md` must put Phase 4 modules-backfill before Phase 5 drop), or the JSON-array is sourced from a snapshot, not the live `sm_general_settings` table.
- **evidence:**
  - `docs/schemas/data-migration/06-field-data-flow.md:436-444`:
    ```sql
    UPDATE platform_packages pp
    JOIN sm_general_settings gs ON gs.school_id = pp.school_id
    SET pp.modules = JSON_ARRAYAGG(name) FROM (
      SELECT 'Lesson' AS name WHERE gs.Lesson = 1
      UNION ALL SELECT 'Chat' WHERE gs.Chat = 1
      -- ... 35 modules
    ) AS enabled;
    ```
  - `docs/schemas/data-migration/05-brand-removal.md:50-87` — `DROP COLUMN Lesson`, `DROP COLUMN Chat`, ... (35 columns dropped from `settings_general_settings`).

---

### FINDING 34

- **id:** DOC-SCHM-034
- **area:** documentation
- **severity:** Low
- **location:** `docs/schemas/data-migration/07-verification.md:144-152`
- **description:** The "UUIDv7 derivation verification" SQL block is incomplete: it shows `UUID_FROM_BIN( CONCAT( UNHEX(LPAD(HEX(UNIX_TIMESTAMP() * 1000), 12, '0')), -- simplified -- ... full derivation per 02-id-conversion.md ) )` and `CASE WHEN c.id = UUID_FROM_BIN(...) THEN 'MATCH' ELSE 'MISMATCH' END`. The `UUID_FROM_BIN(...)` expression is left as an ellipsis. The verification script as written cannot be executed; consumers must rewrite the derivation from scratch.
- **expected:** A complete, executable verification query that runs the actual `uuid_v7(namespace, legacy_id)` derivation from `02-id-conversion.md:78-86`.
- **evidence:**
  - `docs/schemas/data-migration/07-verification.md:144-152`:
    ```sql
    UUID_FROM_BIN(
      CONCAT(
        UNHEX(LPAD(HEX(UNIX_TIMESTAMP() * 1000), 12, '0')),  -- simplified
        -- ... full derivation per 02-id-conversion.md
      )
    ) AS expected_id,
    CASE WHEN c.id = UUID_FROM_BIN(...) THEN 'MATCH' ELSE 'MISMATCH' END
    ```

---

### FINDING 35

- **id:** DOC-SCHM-035
- **area:** documentation
- **severity:** Low
- **location:** `docs/schemas/data-migration/11-security.md:14-17`
- **description:** The "credential in git history" section presents `DATABASE_URL="mysql://devuser:paxxw0rd@2791@127.0.0.1:3306/devdb"` as a real credential that needs rotation. The URL contains the literal string `paxxw0rd@2791` (which has a stray `@` in the middle, breaking URL parsing — the host portion `@2791@127.0.0.1` is not valid). The doc reproduces the credential verbatim in the rotation procedure (line 14-17) and the verification grep commands (lines 91-92). Any consumer copy-pasting this URL into a `.env` for testing will fail URL parsing; anyone running the verification grep will match the rotation doc itself.
- **expected:** A redacted placeholder (`mysql://devuser:<REDACTED>@127.0.0.1:3306/devdb`) in the doc body, with the real credential recorded only in a credential vault or an out-of-band reference.
- **evidence:**
  - `docs/schemas/data-migration/11-security.md:14-17`:
    ```
    DATABASE_URL="mysql://devuser:paxxw0rd@2791@127.0.0.1:3306/devdb"
    ```
  - `docs/schemas/data-migration/11-security.md:91-92`:
    ```
    git log -p --all -- .env | grep -i paxxw0rd
    git log -p --all -- .env | grep -i 2791
    ```
  - The grep commands will match the `11-security.md` file itself (the verbatim occurrence), causing false positives on every run.

---

### FINDING 36

- **id:** DOC-SCHM-036
- **area:** documentation
- **severity:** Medium
- **location:** `docs/schemas/data-migration/11-security.md:184-205` vs `docs/schemas/audit-schema.md:262-274`
- **description:** The retention table in `11-security.md` (line 191-200) lists retention periods for `Authentication events | 18 months`, `Authorization denials | 36 months`, `AI agent actions | 36 months`. The retention table in `audit-schema.md` § 9 (line 266-275) lists the same record types but with different periods (`Authorization denials | 36 months` matches; `Authentication events | 18 months` matches; `AI agent actions | 36 months` matches) — the tables appear to agree. However, `11-security.md` omits the `Finance mutations | 7 years`, `Payroll mutations | 7 years`, `Capability / role changes | 7 years`, `Library / facilities mutations | 3 years`, `Backup events | 3 years` rows that are in `audit-schema.md`. The two tables should be aligned (or one should be the source of truth).
- **expected:** A single retention table referenced from both files.
- **evidence:**
  - `docs/schemas/data-migration/11-security.md:191-200` lists 8 record types.
  - `docs/schemas/audit-schema.md:264-275` lists 10 record types.
  - `Capability / role changes | 7 years` and `Payroll mutations | 7 years` and `Backup events | 3 years` are present in `audit-schema.md` but absent from `11-security.md`.

---

### END FINDINGS

Total findings: **36**

Counts by severity:
- Critical: 4 (DOC-SCHM-003, DOC-SCHM-004, DOC-SCHM-007, DOC-SCHM-008)
- High: 7 (DOC-SCHM-001, DOC-SCHM-002, DOC-SCHM-005, DOC-SCHM-006, DOC-SCHM-009, DOC-SCHM-010, DOC-SCHM-018)
- Medium: 16 (DOC-SCHM-011, DOC-SCHM-012, DOC-SCHM-013, DOC-SCHM-014, DOC-SCHM-015, DOC-SCHM-016, DOC-SCHM-019, DOC-SCHM-021, DOC-SCHM-022, DOC-SCHM-023, DOC-SCHM-025, DOC-SCHM-026, DOC-SCHM-029, DOC-SCHM-031, DOC-SCHM-033, DOC-SCHM-036)
- Low: 9 (DOC-SCHM-017, DOC-SCHM-020, DOC-SCHM-024, DOC-SCHM-027, DOC-SCHM-028, DOC-SCHM-030, DOC-SCHM-032, DOC-SCHM-034, DOC-SCHM-035)

