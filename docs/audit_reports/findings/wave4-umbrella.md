## Wave 4 Foundation Audit Report — `educore` (Umbrella Crate)

**Scope:** `crates/educore/` (Cargo.toml, src/lib.rs, tests/consumer_e2e.rs); AGENTS.md § "Umbrella" + § "Crate Inventory"; docs/build-plan.md lines 0-200; docs/library-docs.md; docs/specs/sync/overview.md.

**Total findings:** 18

---

### FINDING 1

- **id:** UMB-001
- **area:** infra-umbrella
- **severity:** Critical
- **location:** `crates/educore/Cargo.toml` (entire `[dependencies]` table) and `crates/educore/src/lib.rs` (entire re-export block)
- **description:** `educore-cli` is scaffolded at `crates/tools/cli/` and is entry #35 in AGENTS.md's "Crate Inventory" (Phase 16, Test infrastructure + SDK) — but it is neither listed in the umbrella's `[dependencies]` nor re-exported in `lib.rs`. Consumers who follow AGENTS.md and depend on `educore::cli::*` for the CLI binary facade will get a compile error. AGENTS.md says the umbrella "re-exports the public surface of all 34 internal crates", and `crates/tools/cli/Cargo.toml` defines `[package] name = "educore-cli"` for that role.
- **expected:** Per AGENTS.md line 153: "The umbrella crate `educore` re-exports the public surface of all 34 internal crates." Per AGENTS.md line 523: row 35 = `educore-cli` (tools, Phase 16).
- **evidence:**
  ```toml
  # crates/educore/Cargo.toml — [dependencies] section (lines 6-47) lists
  # 34 deps but educore-cli is absent.
  # crates/tools/cli/Cargo.toml exists and defines name = "educore-cli".
  ```

---

### FINDING 2

- **id:** UMB-002
- **area:** infra-umbrella
- **severity:** Critical
- **location:** `crates/educore/Cargo.toml` and `crates/educore/src/lib.rs` (no occurrence of `educore-query-derive`)
- **description:** `educore-query-derive` is the Phase 0 proc-macro crate (AGENTS.md inventory row #2) that provides `#[derive(DomainQuery)]`. It is scaffolded at `crates/infra/query-derive/` and `library-docs.md` references the macro through the engine surface. The umbrella does NOT depend on it in `Cargo.toml` and does NOT re-export it in `lib.rs`. AGENTS.md line 128-137 mandates "The umbrella re-exports each internal crate under its short name" with the example `pub use educore_core as core;`. A proc-macro crate can be re-exported via `pub use ::educore_query_derive::DomainQuery;` (Rust 2018+) but no such re-export exists.
- **expected:** Per AGENTS.md line 128-137: "The umbrella re-exports each internal crate under its short name: ... `pub use educore_core as core;` ...". Per AGENTS.md line 490: row 2 = `educore-query-derive` (infra, Phase 0). Per docs/library-docs.md: `#[derive(DomainQuery)]` is the documented entry point to the query layer.
- **evidence:**
  ```rust
  // crates/educore/src/lib.rs — search for "query_derive" returns 0 matches.
  // crates/educore/Cargo.toml — search for "query-derive" returns 0 matches.
  // AGENTS.md:489-490
  //   | 1 | infra | `educore-core` | 0 | Foundation |
  //   | 2 | infra | `educore-query-derive` | 0 | Foundation (proc-macro) |
  ```

---

### FINDING 3

- **id:** UMB-003
- **area:** infra-umbrella
- **severity:** Critical
- **location:** `crates/educore/Cargo.toml:46-47` and `crates/educore/src/lib.rs:54-55` vs. `AGENTS.md:479-524` ("Crate Inventory" table)
- **description:** The umbrella depends on `educore-sync` and `educore-sync-inprocess` and re-exports them as `sync` and `sync_inprocess`. These crates are described in `docs/build-plan.md` § Phase 0 (ADR-018) and are scaffolded at `crates/cross-cutting/sync/` and `crates/cross-cutting/sync-inprocess/`. However, they are NOT present in AGENTS.md's "Crate Inventory" table (rows 1-35). AGENTS.md's preamble (line 24) asserts "The 34 crates are organized into 5 tiers + 1 umbrella", and the table only contains 35 rows (incl. the umbrella). The actual count is 34 internal crates + 2 sync crates = 36 internal crates, but AGENTS.md only documents 34. The umbrella has 36 dependencies but the inventory lists 35.
- **expected:** Per AGENTS.md line 24: "The 34 crates are organized into 5 tiers + 1 umbrella." Per AGENTS.md line 479-524: Crate Inventory table is "the authoritative source — do not rely on the directory tree or the umbrella re-exports to determine phase assignment."
- **evidence:**
  ```toml
  # crates/educore/Cargo.toml:46-47
  educore-sync = { workspace = true }
  educore-sync-inprocess = { workspace = true }

  # crates/educore/src/lib.rs:54-55
  pub use educore_sync as sync;
  pub use educore_sync_inprocess as sync_inprocess;

  # AGENTS.md grep for "educore-sync" returns only build-plan references;
  # no row in the Crate Inventory table.
  ```

---

### FINDING 4

- **id:** UMB-004
- **area:** infra-umbrella
- **severity:** High
- **location:** `crates/educore/Cargo.toml` (no `[features]` section) vs. `docs/specs/sync/overview.md:27-50`
- **description:** The umbrella crate declares no `[features]` table and no `default-features` line, yet `docs/specs/sync/overview.md` § "Sync as a Build Feature" (lines 38-52) mandates: "The sync module is gated behind a Cargo feature: ```toml [features] sync = ['dep:educore-events', 'dep:educore-rbac'] ```". The umbrella unconditionally pulls `educore-sync` and `educore-sync-inprocess` (Cargo.toml:46-47) and unconditionally re-exports `pub use educore_sync as sync;` (lib.rs:54), so embedded/server-only consumers that should be able to compile without sync cannot.
- **expected:** Per `docs/specs/sync/overview.md:30`: "`educore::sync` module is gated behind a Cargo feature so the core engine library can be compiled without any sync dependency for embedded / server-only use cases." Per `docs/specs/sync/overview.md:44-46`: `[features] sync = ['dep:educore-events', 'dep:educore-rbac']`.
- **evidence:**
  ```toml
  # crates/educore/Cargo.toml — grep for "\[features\]" returns 0 matches.
  # crates/educore/Cargo.toml:46-47
  educore-sync = { workspace = true }
  educore-sync-inprocess = { workspace = true }

  # crates/educore/src/lib.rs:54
  pub use educore_sync as sync;
  ```

---

### FINDING 5

- **id:** UMB-005
- **area:** infra-umbrella
- **severity:** High
- **location:** `crates/educore/Cargo.toml` (entire `[dependencies]` block) vs. `crates/educore/src/lib.rs:21-62` (re-export block)
- **description:** The umbrella's `[dependencies]` table lists 34 entries (lines 6-47), but `lib.rs` re-exports only 32 of them as `pub use ... as ...` (lines 21-62). Cross-checking the two: the umbrella includes `educore_storage_parity` as a dep AND re-exports it as `storage_parity`, but it never includes `educore-cli` or `educore-query-derive` in deps. Meanwhile the AGENTS.md inventory lists `educore-cli` (#35) and `educore-query-derive` (#2) as part of the engine. The dependency count and the re-export count are internally consistent (34 ↔ 32 deps minus the 2 not in inventory) but the inventory itself is incomplete (see UMB-003) so the umbrella is internally consistent with a stale inventory, not with reality.
- **expected:** Per AGENTS.md line 128-137: "The umbrella re-exports each internal crate under its short name ... Consumers therefore write `educore::academic::commands::*` and never need to know the internal `educore-` prefix on the package name." Per AGENTS.md line 153: "re-exports the public surface of all 34 internal crates" — but reality is 36 internal crates.
- **evidence:**
  ```bash
  # 34 deps in crates/educore/Cargo.toml:
  #   grep -c 'workspace = true' crates/educore/Cargo.toml  →  34 (incl. tokio dev-dep)
  # 32 pub use re-exports in crates/educore/src/lib.rs:
  #   grep -c '^pub use' crates/educore/src/lib.rs           →  32
  #   (sync + sync_inprocess = 2 extra deps not in AGENTS.md inventory)
  ```

---

### FINDING 6

- **id:** UMB-006
- **area:** infra-umbrella
- **severity:** High
- **location:** `crates/educore/src/lib.rs:24` (`pub use educore_events as events;`)
- **description:** The umbrella re-exports `educore-events` as `events`. AGENTS.md (lines 171-177) explicitly warns about the `educore-events` vs `educore-events-domain` distinction: "`educore-events` (cross-cutting tier) is the **event envelope + bus port** (DomainEvent trait, EventEnvelope, EventBus trait). `educore-events-domain` (cross-cutting tier) is the **calendar domain** (CalendarEvent, Holiday, Incident, Weekend aggregates)." But the umbrella re-exports both as just `events` and `events_domain` (lib.rs:28-29), and `library-docs.md:99-114` tells consumers to write `use educore::events::*;` to subscribe to `StudentAdmitted`. That `events::*` glob will pull in DomainEvent + EventEnvelope + EventBus + CalendarEvent + Holiday + Incident + Weekend — the calendar domain's types from a different bounded context are silently in scope under the `events` path. The two crates must remain distinct in the public surface.
- **expected:** Per AGENTS.md line 171-177: explicit naming distinction between `educore-events` and `educore-events-domain`. Per docs/library-docs.md:108 `use educore::events::*;` is the documented subscription path and must contain ONLY envelope + bus types.
- **evidence:**
  ```rust
  // crates/educore/src/lib.rs:28-29
  pub use educore_events as events;
  pub use educore_events_domain as events_domain;

  // AGENTS.md:171-177
  // - educore-events (cross-cutting tier) is the event envelope + bus port ...
  // - educore-events-domain (cross-cutting tier) is the calendar domain
  //   (CalendarEvent, Holiday, Incident, Weekend aggregates).
  ```

---

### FINDING 7

- **id:** UMB-007
- **area:** infra-umbrella
- **severity:** Medium
- **location:** `crates/educore/src/lib.rs:75-83` (`pub mod prelude`)
- **description:** The `prelude` module re-exports only 7 paths: `educore_core`, `educore_events`, `educore_operations`, `educore_platform`, `educore_rbac`, `educore_sdk`, `educore_settings`. `docs/library-docs.md` line 10 mandates `use educore::prelude::*;` for the consumer-facing example and shows `Engine::builder()`, `engine.auth()`, `engine.notify()`, `engine.events()`, `engine.rbac()`, `engine.storage()`, `engine.students()`, `engine.attendance()` — none of which are reachable through the current prelude (none of `Engine`, `EngineBuilder`, `StudentService`, `AttendanceService`, `NotifyService`, `PaymentService`, `EventBus`, `RbacProvider`, `StorageAdapter` are flat re-exports). The prelude only re-exports the crate *names*, not the facade types, so `use educore::prelude::*;` gives the consumer 7 modules they still have to navigate into.
- **expected:** Per docs/library-docs.md:8-10: the consumer example begins with `use educore::prelude::*;` and is followed immediately by `Engine::builder()` — implying prelude must surface `Engine` (and the underlying facade services) directly. Per AGENTS.md Engine Rules § "Compile-time safety over strings": prelude is the documented ergonomic entry.
- **evidence:**
  ```rust
  // crates/educore/src/lib.rs:75-83
  pub mod prelude {
      pub use educore_core;
      pub use educore_events;
      pub use educore_operations;
      pub use educore_platform;
      pub use educore_rbac;
      pub use educore_sdk;
      pub use educore_settings;
  }
  // No `pub use educore_sdk::Engine;`, no `pub use educore_storage::StorageAdapter;`,
  // no `pub use educore_auth::AuthProvider;`, etc.
  ```

---

### FINDING 8

- **id:** UMB-008
- **area:** infra-umbrella
- **severity:** Medium
- **location:** `crates/educore/src/lib.rs:57-59` (Test infrastructure section)
- **description:** The umbrella's lib.rs is divided into 6 sections (Domain crates, Port adapters, Sync engine, Test infrastructure, High-level SDK). `educore_audit` (re-exported at line 58) is grouped under "Test infrastructure" — but `educore-audit` is in the **cross-cutting** tier per AGENTS.md line 148 and inventory row #13. This is a tier-bucketing mistake in the umbrella: a cross-cutting concern is mis-labeled as test infrastructure, which would mislead consumers reading the umbrella's source as a tier map.
- **expected:** Per AGENTS.md line 501: row 13 = `educore-audit` (cross-cutting, Phase 2 — Cross-cutting foundations / audit log). Per AGENTS.md line 148: cross-cutting tier table lists `audit` as one of its 7 crates.
- **evidence:**
  ```rust
  // crates/educore/src/lib.rs:56-59
  // ---- Test infrastructure -------------------------------------------------
  pub use educore_audit as audit;
  pub use educore_testkit as testkit;

  // AGENTS.md:501
  //   | 13 | cross-cutting | `educore-audit` | 2 | Cross-cutting foundations (audit log) |
  ```

---

### FINDING 9

- **id:** UMB-009
- **area:** infra-umbrella
- **severity:** Medium
- **location:** `crates/educore/src/lib.rs:20-37` (Domain crates section)
- **description:** The "Domain crates" section header (lib.rs:20) groups 17 re-exports together, but only 10 of them are actually `domains/`-tier crates (`academic`, `assessment`, `attendance`, `cms`, `communication`, `documents`, `events_domain`, `facilities`, `finance`, `hr`, `library` — 11). The other 6 (`core`, `platform`, `rbac`, `settings`, `operations`, `events`) are infra or cross-cutting. The section header is mislabeled. The umbrella then places `educore-events-domain` (a cross-cutting calendar crate per AGENTS.md line 174-177) under the "Domain crates" banner.
- **expected:** Per AGENTS.md line 148: cross-cutting tier is platform, rbac, events, events-domain (calendar), settings, operations, audit (7 crates). Per AGENTS.md line 149: domains tier is the 10 domain bounded contexts. The umbrella's section headers should follow the tier table, not bundle all non-adapter crates under "Domain crates".
- **evidence:**
  ```rust
  // crates/educore/src/lib.rs:20-37
  // ---- Domain crates ------------------------------------------------------
  pub use educore_academic as academic;
  pub use educore_assessment as assessment;
  pub use educore_attendance as attendance;
  pub use educore_cms as cms;
  pub use educore_communication as communication;
  pub use educore_core as core;          // <-- infra
  pub use educore_documents as documents;
  pub use educore_events as events;      // <-- cross-cutting
  pub use educore_events_domain as events_domain;  // <-- cross-cutting (calendar)
  pub use educore_facilities as facilities;
  pub use educore_finance as finance;
  pub use educore_hr as hr;
  pub use educore_library as library;
  pub use educore_operations as operations;  // <-- cross-cutting
  pub use educore_platform as platform;  // <-- cross-cutting
  pub use educore_rbac as rbac;          // <-- cross-cutting
  pub use educore_settings as settings;  // <-- cross-cutting
  ```

---

### FINDING 10

- **id:** UMB-010
- **area:** infra-umbrella
- **severity:** Medium
- **location:** `crates/educore/tests/consumer_e2e.rs:19-28`
- **description:** The consumer-facing end-to-end test imports internal crate paths directly — `use educore_core::clock::{...}`, `use educore_notify::errors::NotificationTemplateId`, `use educore_payment::port::{...}`, `use educore_sdk::Engine`, `use educore_storage::student_attendance_row::StudentAttendanceRow` — instead of using the umbrella's documented `educore::*` re-export paths. Per `docs/library-docs.md:10` and `AGENTS.md:128-137`, the consumer surface is `educore::*`; the test is meant to be a "consumer-facing E2E" (per its module docstring at line 1) but it bypasses the umbrella entirely. Any regression in the umbrella's re-export wiring would not be caught by this test.
- **expected:** Per `docs/library-docs.md:8-10`: consumer entry point is `use educore::prelude::*;`. Per the test's own module docstring at `crates/educore/tests/consumer_e2e.rs:1-3`: "Consumer-facing end-to-end integration test for the Educore engine." A consumer-facing test should exercise the umbrella's public path.
- **evidence:**
  ```rust
  // crates/educore/tests/consumer_e2e.rs:19-28
  use educore_core::clock::{IdGenerator, SystemIdGen};
  use educore_core::tenant::{TenantContext, UserType};
  use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};
  use educore_notify::errors::NotificationTemplateId;
  use educore_notify::port::{Channel, Priority, Recipient, SendNotification, TemplateRef};
  use educore_payment::port::{
      ChargeRequest, CurrencyCode, CustomerId, CustomerRef, Money, PaymentMethod,
  };
  use educore_sdk::Engine;
  use educore_storage::student_attendance_row::StudentAttendanceRow;
  ```

---

### FINDING 11

- **id:** UMB-011
- **area:** infra-umbrella
- **severity:** Medium
- **location:** `crates/educore/src/lib.rs:19` (`// ---- Domain crates ------------------------------------------------------`)
- **description:** The lib.rs comment at line 19 says "Domain crates" but the section also contains `core`, `platform`, `rbac`, `settings`, `operations`, `events`, `events_domain` — 7 of the 17 entries are NOT domain crates per AGENTS.md's tier table. The first alphabetical entry (`core`) is infra tier; `platform`, `rbac`, `settings`, `operations` are cross-cutting tier; `events` is cross-cutting (envelope + bus port); `events_domain` is cross-cutting (calendar domain per AGENTS.md line 174). A consumer reading the umbrella source as the engine's organization map will mis-classify 7 of the 17 entries in this section.
- **expected:** Per AGENTS.md line 148: cross-cutting tier = platform, rbac, events, events-domain, settings, operations, audit. Per AGENTS.md line 150: adapters tier. The umbrella's section comments should match the tier table.
- **evidence:**
  ```rust
  // crates/educore/src/lib.rs:19
  // ---- Domain crates ------------------------------------------------------
  pub use educore_academic as academic;        // domains
  pub use educore_assessment as assessment;    // domains
  pub use educore_attendance as attendance;    // domains
  pub use educore_cms as cms;                  // domains
  pub use educore_communication as communication;  // domains
  pub use educore_core as core;                // infra
  ...
  ```

---

### FINDING 12

- **id:** UMB-012
- **area:** infra-umbrella
- **severity:** Medium
- **location:** `crates/educore/src/lib.rs:47` (`pub use educore_storage_parity as storage_parity;`)
- **description:** The umbrella re-exports `educore_storage_parity` as `storage_parity`. `educore-storage-parity` is in the **tools** tier (AGENTS.md line 151: "Dev tooling: testkit, storage-parity, cli (binary), sdk"), intended for cross-adapter test suites, not for consumer runtime use. Exposing it as a top-level `educore::storage_parity` path implies it's part of the consumer surface. Per AGENTS.md line 405: "Test infrastructure: educore-testkit, educore-storage-parity (full suite), educore-sdk, educore-cli" — `educore-testkit` is similarly a tools-tier crate but re-exported at lib.rs:59 with the same exposure problem.
- **expected:** Per AGENTS.md line 151: tools tier is dev tooling, not consumer surface. Per `docs/build-plan.md` Phase 16: storage-parity is "the cross-adapter test suite", not a runtime consumer crate.
- **evidence:**
  ```rust
  // crates/educore/src/lib.rs:47, 59
  pub use educore_storage_parity as storage_parity;
  pub use educore_testkit as testkit;

  // AGENTS.md:151
  //   | `tools` | `crates/tools/` | 4 | Dev tooling: testkit, storage-parity, cli (binary), sdk |
  ```

---

### FINDING 13

- **id:** UMB-013
- **area:** infra-umbrella
- **severity:** Medium
- **location:** `crates/educore/src/lib.rs:68-74` (docstring on `pub mod prelude`)
- **description:** The prelude's rustdoc states: "richer re-exports land alongside the `DomainError`, `TenantContext`, `EventEnvelope`, and `Capability` types in the relevant PRs." It also references "Phase 14" twice. This is a planning note embedded in the public API docstring. The umbrella has `#![deny(missing_docs)]` at line 13, so doc comments are required — but a docstring that is a roadmap is not a docstring. Consumers reading `cargo doc --open` (or docs.rs) will see Phase 14 mentioned, which is implementation-leakage.
- **expected:** Per AGENTS.md § Code Standards (line 393): "All public APIs are documented with rustdoc; `#![deny(missing_docs)]`." Per AGENTS.md § Engine Rules (line 217): "Production-ready. Real schools, real students, real money." — public docstrings must describe what the type IS, not the implementation roadmap.
- **evidence:**
  ```rust
  // crates/educore/src/lib.rs:68-74
  /// Prelude of common types consumers are expected to import.
  ///
  /// The prelude's typed re-exports are wired in incrementally as the
  /// underlying crates are implemented in Phase 0 (PRs 3-8). At the
  /// scaffold stage the prelude is intentionally a thin re-export of
  /// crate-level paths so the workspace builds; richer re-exports land
  /// alongside the `DomainError`, `TenantContext`, `EventEnvelope`, and
  /// `Capability` types in the relevant PRs. Phase 14 adds the
  /// `educore_settings` + `educore_operations` re-exports.
  pub mod prelude {
  ```

---

### FINDING 14

- **id:** UMB-014
- **area:** infra-umbrella
- **severity:** Medium
- **location:** `crates/educore/Cargo.toml` (no `default-features = false` on any storage adapter dep) vs. `docs/library-docs.md:8-24`
- **description:** The umbrella unconditionally enables the default features of every dependency it pulls in — `educore-storage-postgres`, `educore-storage-mysql`, `educore-storage-sqlite`, `educore-storage-surrealdb` (Cargo.toml:39-42) and the port adapters (Cargo.toml:23-32). A consumer that wants only the SurrealDB backend per ADR-017 (and ADR-017 explicitly says PG/MySQL/SQLite move to Phase 1 as parity adapters) still pulls in `sqlx`, `mysql_async`, `reqwest`, `lettre`, etc. via feature unification. ADR-015 mandates "TLS/SSL Cross-Compilation: Strictly enforce `rustls` instead of `native-tls` ... For crates like `reqwest`, always set `default-features = false`" — but the umbrella inherits those deps without setting `default-features = false`, so a consumer depending on `educore` cannot override the feature set.
- **expected:** Per AGENTS.md line 391-395: "TLS/SSL Cross-Compilation: Strictly enforce `rustls` instead of `native-tls` to support cross-compilation ... For crates like `reqwest`, always set `default-features = false` and enable the `rustls` or `rustls-tls` feature." Per ADR-017: SurrealDB is primary; PG/MySQL/SQLite are parity adapters — the umbrella should make SurrealDB the default and gate the others.
- **evidence:**
  ```toml
  # crates/educore/Cargo.toml:39-42
  educore-storage-postgres = { workspace = true }
  educore-storage-mysql = { workspace = true }
  educore-storage-sqlite = { workspace = true }
  educore-storage-surrealdb = { workspace = true }
  # No `default-features = false`, no feature gate.
  ```

---

### FINDING 15

- **id:** UMB-015
- **area:** infra-umbrella
- **severity:** Low
- **location:** `crates/educore/src/lib.rs:21-37` (alphabetical ordering of section entries)
- **description:** Within the "Domain crates" section (lib.rs:21-37), entries are roughly alphabetical but `educore-core` (line 26) sorts to position 6 (between `cms` and `documents`), while per AGENTS.md's tier table core is infra — it should appear under a "Foundation" section. Similarly `educore-platform` (line 35) sorts between `operations` and `rbac`, mixing cross-cutting crate names with domain crate names alphabetically. A consumer scanning for a specific crate by name will be misled into thinking the umbrella's section order reflects the engine's tier order.
- **expected:** Per AGENTS.md line 147-151: 5-tier system; consumers are taught to read by tier. The umbrella's section comments should follow tier order (infra → cross-cutting → domains → adapters → tools) for at-a-glance navigation.
- **evidence:**
  ```rust
  // crates/educore/src/lib.rs:21-37 — section is labeled "Domain crates"
  // but mixes:
  //   infra:    core (line 26)
  //   cross-cutting: events (28), events_domain (29), operations (34),
  //                  platform (35), rbac (36), settings (37)
  //   domains:  academic, assessment, attendance, cms, communication,
  //             documents, facilities, finance, hr, library
  ```

---

### FINDING 16

- **id:** UMB-016
- **area:** infra-umbrella
- **severity:** Medium
- **location:** `crates/educore/src/lib.rs:65` (`pub const VERSION`)
- **description:** The `VERSION` constant is a useful re-export of the package version, but its docstring (lib.rs:64) does not cite a stability policy. The umbrella has `#![deny(missing_docs)]`, so the docstring is required — but consumers building tools around `educore::VERSION` (e.g. health checks, telemetry, log enrichment) need to know whether the value is sourced from `Cargo.toml`'s `[workspace.package] version` (single source, bumped manually per release) or from `git describe` output, and whether it is updated at runtime. Without that information, downstream tooling that pins to a specific `VERSION` cannot reason about upgrade safety.
- **expected:** Per AGENTS.md line 393: "All public APIs are documented with rustdoc". Per AGENTS.md line 395: "All fallible APIs return `Result<T, DomainError>`" — invariants should be documented. The constant's docstring should describe its source and update semantics.
- **evidence:**
  ```rust
  // crates/educore/src/lib.rs:64-65
  /// Educore version, sourced from the package manifest.
  pub const VERSION: &str = env!("CARGO_PKG_VERSION");
  ```

---

### FINDING 17

- **id:** UMB-017
- **area:** infra-umbrella
- **severity:** Medium
- **location:** `crates/educore/src/lib.rs:21-62` (no re-export ordering stability invariant)
- **description:** The umbrella re-export block does not declare a stability order. `pub use educore_academic as academic;` is at line 21, `pub use educore_events as events;` is at line 28. AGENTS.md (line 128) and `library-docs.md` describe `educore::*` as "a single, stable path" but provide no ordering invariant — any future PR that reorders or inserts new entries alphabetically will shift line numbers, which matters for tools that diff the umbrella's surface (e.g. `cargo-public-api`, `cargo-geiger`, audit scripts). The current ordering is approximately alphabetical within each section, which is a de-facto contract.
- **expected:** Per AGENTS.md line 153: "The umbrella crate `educore` re-exports the public surface of all 34 internal crates" — "public surface" implies a stable contract. Per docs/library-docs.md:1-2: "the umbrella crate ... Re-exports every domain, port, and adapter crate under a single, stable path" — "stable" is promised but not defined.
- **evidence:**
  ```rust
  // crates/educore/src/lib.rs:21-62 — 32 pub use statements in 6 sections.
  // No docstring, comment, or test asserts an ordering invariant.
  ```

---

### FINDING 18

- **id:** UMB-018
- **area:** infra-umbrella
- **severity:** Low
- **location:** `crates/educore/src/lib.rs:18` (section comment ordering)
- **description:** The umbrella's `lib.rs` opens with `// ---- Domain crates ----` as the first section comment (line 20), then `// ---- Port adapters` (line 39), then `// ---- Sync engine` (line 53), then `// ---- Test infrastructure` (line 57), then `// ---- High-level SDK` (line 61). This order is not the tier order from AGENTS.md (infra → cross-cutting → domains → adapters → tools) nor the AGENTS.md Crate Inventory row order (rows 1-35: infra first, then cross-cutting, then domains, then adapters, then tools). The umbrella's ordering is domains-first, which does not match either the dependency direction or the inventory table — it inverts the natural top-down read.
- **expected:** Per AGENTS.md line 158-161: "Layered dependency direction (no cycles, no upward deps): infra ← cross-cutting ← domains ← tools; ↑; └── adapters (also depends on infra + cross-cutting)." The umbrella's source organization should follow this direction.
- **evidence:**
  ```rust
  // crates/educore/src/lib.rs:19-62 — section order:
  //   1. Domain crates (line 19)
  //   2. Port adapters (line 38)
  //   3. Sync engine (line 52)
  //   4. Test infrastructure (line 56)
  //   5. High-level SDK (line 60)
  // AGENTS.md tier order: infra → cross-cutting → domains → adapters → tools.
  ```
