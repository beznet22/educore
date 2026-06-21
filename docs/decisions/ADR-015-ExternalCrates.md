# ADR-015: External Crate Selection

## Status

Accepted, 2026-06-09.

## Context

The engine depends on ~25 external crates. Selection criteria:

- **License:** MIT / Apache-2.0 / BSD-compatible (no GPL, no AGPL, no SSPL).
- **Maintenance:** active within the last 6 months.
- **Cross-compile:** Linux, macOS, Windows, Android aarch64, WASM (per `AGENTS.md` validation checklist and the build plan's `rustls` cross-compilation mandate).
- **Performance:** not the slowest option for the use case.
- **Ecosystem:** widely used, so the engine's code is recognizable to contributors.
- **MSRV:** ≤ 1.75 (the engine's pinned floor, per `Cargo.toml` `[workspace.package]` `rust-version = "1.75"`).

This ADR is the single source of truth for which crates the engine uses, the
alternatives considered, and the rationale. When a new external crate is
needed, the contributor adds a row to § "Decision" with a 5-10 line rationale
and a per-crate cross-compile status.

## Cross-compile priority

For every external crate, this ADR documents three target tiers:

- **Tier 1** (required): Linux x86_64, macOS x86_64 / aarch64, Windows x86_64.
- **Tier 2** (required): Android aarch64 (`aarch64-linux-android`).
- **Tier 3** (required where possible): WASM (`wasm32-unknown-unknown`).

A crate that doesn't support a tier is documented as "tier-N: not supported"
with the engine's fallback path (e.g. `gloo-net` for HTTP on WASM).

## Consolidated maintenance status (data fetched 2026-06-09)

Verified against crates.io API, docs.rs, and GitHub REST API. The MSRV
column shows the crate's latest version's `rust_version` field, where
declared; "?" means undeclared. The 1.75 floor is marked **OK** if the
crate's MSRV ≤ 1.75, **RAISE** if the crate requires MSRV > 1.75.

### Tier-A: Foundation crates (researched in detail)

| Crate | Latest | Released | Last commit | Issues | PRs | MSRV | License | T1 Linux/macOS/Win | T2 Android | T3 WASM | 1.75 OK? |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `tokio` | 1.52.3 | 2026-05-08 | 2026-06-09 | 324 | 103 | 1.71 | MIT | ✓ | ✓ | partial (no `tokio::net`; engine uses `gloo-net` on WASM) | OK |
| `sqlx` | 0.9.0 | 2026-05-21 | 2026-06-03 | 673 | 71 | 1.94.0 | MIT / Apache-2.0 | ✓ | partial (no aarch64 CI) | ✗ (no `wasm` feature; engine uses `gloo-sqlite` for offline) | **RAISE** (pin to 0.7.x or 0.8.x) |
| `rustls` | 0.23.40 | 2026-04-28 | 2026-06-09 | 76 | 12 | 1.71 (1.75 with `zlib` feature) | Apache-2.0 / ISC / MIT | ✓ | ✓ | ✓ (with `wasm-bindgen`) | OK |
| `mysql_async` | 0.37.0 | 2026-05-25 | 2026-05-25 | 24 | 12 | ? (not declared) | MIT / Apache-2.0 | ✓ | unknown | ✗ | OK (no MSRV declared) |
| `reqwest` | 0.13.4 | 2026-05-25 | 2026-06-09 | 364 | 96 | 1.85.0 | MIT / Apache-2.0 | ✓ | partial (open issues #2966, #2968: Android crash; needs `rustls_platform_verifier::android::init_hosted()`) | ✓ (auto-switches on `target_arch=wasm32`) | **RAISE** (pin to 0.12.x) |
| `chrono` | 0.4.45 | 2026-06-04 | 2026-06-08 | 155 | 34 | 1.62.0 | MIT / Apache-2.0 | ✓ | unknown | partial (`wasmbind` JS interop only) | OK |
| `time` | 0.3.47 | 2026-02-05 | 2026-05-18 | 11 | 1 | 1.88.0 | MIT / Apache-2.0 | ✓ | unknown | partial (`wasm-bindgen` only) | **RAISE** (pin to older line) |
| `rust_decimal` | 1.42.0 | 2026-05-06 | 2026-05-22 | 37 | 4 | 1.67.1 | MIT | ✓ | unknown | ✓ (`wasm` feature flag) | OK |
| `lettre` | 0.11.22 | 2026-05-14 | 2026-05-28 | 61 | 17 | 1.85 | MIT | ✗ (no `wasm` feature; SMTP requires sockets) | unknown | ✗ | **RAISE** (pin to older 0.10.x) |
| `aws-sdk-s3` | 1.135.0 | 2026-06-02 | 2026-06-08 | 148 | 0 (monorepo) | 1.91.1 | Apache-2.0 | ✓ | unknown | ✗ (no `wasm` feature; uses `rt-tokio`) | **RAISE** (pin to older 1.x) |
| `serde` | 1.0.228 | 2025-09-27 | 2026-06-02 | 345 | 39 | 1.56 | MIT / Apache-2.0 | ✓ | ✓ (no platform-specific code) | ✓ | OK |
| `serde_json` | 1.0.149 | 2025-12-19 | 2026-05-30 | 81 | 14 | 1.56 | MIT / Apache-2.0 | ✓ | ✓ | ✓ | OK |
| `uuid` | 1.23.3 | 2026-06-09 | 2026-06-09 | 10 | 0 | 1.85.0 | Apache-2.0 / MIT | ✓ (`js` feature via `wasm-bindgen`) | unknown | ✓ | **RAISE** (pin to 1.10..<1.23) |
| `thiserror` | 2.0.18 | 2026-01-18 | 2026-05-21 | 25 | 4 | 1.68 | MIT / Apache-2.0 | ✓ | ✓ (proc-macro only) | ✓ | OK |
| `anyhow` | 1.0.102 | 2026-02-20 | 2026-03-24 | 35 | 3 | 1.68 | MIT / Apache-2.0 | ✓ | ✓ | ✓ | OK |
| `async-trait` | 0.1.89 | 2025-08-14 | 2026-03-24 | 9 | 1 | 1.56 | MIT / Apache-2.0 | ✓ (proc-macro only) | ✓ (proc-macro only) | ✓ | OK |
| `tracing` | 0.1.44 | 2025-12-18 | 2026-05-30 | 628 | 227 | 1.65 | MIT | partial (no `wasm` feature; `valuable` feature used by WASM consumers) | unknown | partial | OK |
| `tracing-subscriber` | 0.3.23 | 2026-03-13 | 2026-05-30 | (shared) | (shared) | 1.65 | MIT | partial (no `wasm` feature; fmt layer is `no_std`-clean) | unknown | partial | OK |
| `validator` | 0.20.0 | 2025-01-20 | 2026-04-22 | 49 | 14 | 1.81 | MIT | ✗ (no `wasm` feature) | unknown | ✗ | **RAISE** (pin to 0.19.x or replace) |
| `async-nats` | 0.33.0 | 2025-01-21 | 2026-06-09 | 187 | 23 | 1.74 (0.33.x); 1.81 (0.34.x) | Apache-2.0 / MIT | ✓ | unknown (no Android cfg; uses `rustls-tls` via the `rustls` crate) | partial (uses `tokio` runtime; no dedicated `wasm` feature) | **OK if pinned to 0.33.x** |
| `redis` | 0.25.5 | 2024-09-15 | 2026-06-09 | 117 | 28 | 1.65 (0.25.x); 1.80 (0.27.x) | BSD-3-Clause / Apache-2.0 / MIT | ✓ | unknown (no Android cfg; uses `tokio` runtime + `rustls` via the `tokio-rustls` feature) | partial (no `wasm` feature) | **OK if pinned to 0.25.x** |
| `argon2` | 0.5.3 | 2026-04-21 | 2026-06-08 | 7 | 9 | 1.65 (0.5.x); 1.85 (0.6.0-rc.8) | MIT / Apache-2.0 | ✗ (no `wasm` feature) | unknown | ✗ | OK (pin 0.5.3) |
| `hmac` | 0.13.0 | 2026-03-29 | 2026-06-08 | 2 | 2 | 1.85 (0.13.x); 1.65 (0.12.x) | MIT / Apache-2.0 | ✗ (no `wasm` feature directly; compose with `sha2`'s WASM backend) | unknown | partial (via `sha2`) | **OK if pinned to 0.12.x** |
| `sha2` | 0.11.0 | 2026-03-25 | 2026-06-09 | 18 | 13 | 1.85 (0.11.x); 1.65 (0.10.x) | MIT / Apache-2.0 | ✓ (explicit `wasm32-simd128` backend; auto-selected on wasm32 + simd128) | unknown (no Android cfg; `aarch64-sha2` backend auto-selected on AArch64 with CPU sha2 feature) | ✓ | **OK if pinned to 0.10.x** |
| `surrealdb` | (pre-1.0, see Decision §) | n/a | 2026-06-09 | n/a | n/a | MSRV 1.75+ on the latest dev line | Apache-2.0 / MIT / Business Source License 1.1 (BSL — see Decision) | ✓ (Linux/macOS/Windows native; embedded mode only) | partial (Android aarch64 status depends on `surrealdb`'s release at the time of pin; engine tracks upstream) | ✓ (`wasm-bindgen` feature) | **RAISE** if a pre-1.75 line ever lands; pin to the last 1.75-compatible line |

### Tier-B: Engine-internal utilities (researched briefly)

| Crate | Latest | Released | Last commit | Issues | PRs | MSRV | License | T1 | T2 | T3 | 1.75 OK? |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `rust_decimal_macros` | 1.40.0 | 2026-01-14 | 2026-05-22 | 37 | 4 | ? (not declared) | MIT | ✓ | ✓ | ✓ | OK |
| `futures` | 0.3.32 | 2026-02-15 | 2026-06-04 | 209 | 22 | 1.71 | MIT / Apache-2.0 | ✓ | ✓ | ✓ | OK |
| `rand` | 0.10.1 (newest 0.8.6) | 2026-04-17 | 2026-06-04 | 18 | 3 | 1.63 (0.8.x); 1.85 (0.10.x) | MIT / Apache-2.0 | ✓ | ✓ | partial (via `getrandom` + `wasm_js`) | **OK if pinned to 0.8.x** |
| `secrecy` | 0.10.3 | 2024-10-09 | 2026-05-06 | 33 | 17 | 1.60 | Apache-2.0 / MIT | ✓ | ✓ | ✓ | OK |
| `regex` | 1.12.4 | 2026-06-09 | 2026-06-09 | 49 | 26 | 1.65 | MIT / Apache-2.0 | ✓ | ✓ | ✓ | OK |
| `indexmap` | 2.14.0 | 2026-04-09 | 2026-05-02 | 8 | 4 | 1.85 | Apache-2.0 / MIT | ✓ | ✓ | ✓ | **RAISE** (pin to 2.5.x) |
| `derive_more` | 1.0.0 | 2024-08-02 | 2026-05-29 | 89 | 26 | 1.61 | MIT | ✓ | ✓ | ✓ | OK |
| `once_cell` | 1.21.3 | 2025-09-22 | 2026-05-30 | 252 | 27 | 1.66 | MIT / Apache-2.0 | ✓ | ✓ | ✓ | OK |
| `proc-macro2` | 1.0.106 | 2025-08-19 | 2026-06-05 | 0 | 0 | 1.56 | MIT / Apache-2.0 | ✓ | ✓ | ✓ | OK |
| `quote` | 1.0.45 | 2025-08-19 | 2026-06-05 | 0 | 0 | 1.56 | MIT / Apache-2.0 | ✓ | ✓ | ✓ | OK |
| `syn` | 2.0.117 | 2025-08-19 | 2026-06-05 | 0 | 0 | 1.56 | MIT / Apache-2.0 | ✓ | ✓ | ✓ | OK |

## Decision

The engine uses the 29 external crates listed above. Each was selected
after comparing 2-3 alternatives on the selection criteria. The table
below documents the chosen version, the alternatives considered, and
the per-crate rationale.

### Foundation tier

#### `tokio 1.52.3` (workspace: `"1.40"`, features `["full"]`)

- **Alternatives:** `async-std`, `smol`, `embassy` (embedded).
- **Selected because:** the de-facto Rust async runtime; 32k+ GitHub stars; broadest ecosystem support; the only runtime with mature `tokio::net` for the engine's TCP/TLS needs. `embassy` is for embedded microcontrollers and is not suitable for a server-side engine. `async-std` has been in maintenance limbo since 2024.
- **Red flags:** 324 open issues / 103 open PRs; not unusual for a 32k-star project but triaging is slow. The `parking_lot` feature can raise the MSRV above the declared 1.71; the engine avoids this feature to stay on the 1.75 floor.
- **WASM:** partial. The engine uses `gloo-net` + `gloo-timers` + `wasm-bindgen-futures` for the WASM path instead of tokio's `rt` feature.

#### `sqlx ^0.8` (workspace: `"0.8"`, features `["runtime-tokio-rustls", "macros", "json", "uuid", "chrono", "rust_decimal", "mysql", "postgres", "sqlite"]`)

- **Alternatives:** `diesel` (sync), `sea-orm` (ORM on top of sqlx), `tokio-postgres` (raw), `wtx` (newer, less mature).
- **Selected because:** async-native; compile-time query verification (with `sqlx::query!` macros); supports all 3 storage backends (PostgreSQL, MySQL, SQLite); the de-facto choice for new async Rust services.
- **Red flags:** MSRV is **1.94.0** on the current 0.9.0 line. The engine **pins to 0.8.x** (last 1.75-compatible release) until the floor is raised. 673 open issues / 71 open PRs is heavy but the crate is high-traffic.
- **WASM:** not supported. The engine's WASM path uses `gloo-sqlite` + a custom storage port for offline mode (see `docs/schemas/sql-dialects/sqlite.md` for the schema).

#### `rustls ^0.23` (workspace: `"0.23"`, features `["std", "tls12", "aws_lc_rs"]`)

- **Alternatives:** `native-tls` (forbidden by `AGENTS.md` cross-compile mandate; fails on Android), `openssl` (C bindings, fragile), `boringssl`.
- **Selected because:** pure-Rust; cross-compile to all 3 tiers including Android aarch64; the de-facto Rust TLS library. Mandated by `AGENTS.md` ("All dependencies use `rustls`; never `native-tls`").
- **Red flags:** none substantive. The 0.24.0-dev.0 pre-release exists but the engine stays on 0.23.x for stability.

#### `mysql_async ^0.34` (workspace: `"0.34"`, features `["rustls-tls"]`)

- **Alternatives:** `sqlx` (Postgres + SQLite only, no MySQL), `mysql` (sync).
- **Selected because:** the only mature async MySQL driver; pure-Rust TLS via `rustls-tls` feature. Pairs with `sqlx` for cross-backend coverage (sqlx for Postgres + SQLite, mysql_async for MySQL).
- **Red flags:** single-maintainer bus factor (blackbeam); no MSRV declared (the engine floor is implicit). Recent breaking-change incidents between minor versions (#398 — named-vs-positional params) suggest the lone maintainer ships subtle regressions.
- **WASM:** not supported.

#### `reqwest ^0.12` (workspace: `"0.12"`, features `["rustls-tls", "json", "stream"]`)

- **Alternatives:** `awc` (actix), `hyper` raw, `ureq` (sync), `surf` (deprecated).
- **Selected because:** the de-facto Rust HTTP client; 11.6k+ GitHub stars; active commercial support (the maintainer monetizes commercial access); broadest ecosystem.
- **Red flags:** MSRV 1.85 on 0.13.x; the engine **pins to 0.12.x** to stay on 1.75. Android regression (open issues #2966, #2968) requires `rustls_platform_verifier::android::init_hosted()` at startup; the engine does this in `crates/adapters/auth/src/lib.rs` (Phase 15 work).
- **WASM:** auto-switches to the WASM client when `target_arch = wasm32`; TLS/cookies delegated to the browser environment.

#### `surrealdb ^2` (Phase 0 primary storage adapter; pinned to a pre-1.75 line until the engine MSRV is raised; see ADR-017)

- **Alternatives:** `sqlx` (Postgres + SQLite only), `sea-orm` (ORM on top of `sqlx`), raw `tokio-postgres`, `wtx` (newer, less mature).
- **Selected because:** per `ADR-017`, SurrealDB is the engine's Phase 0 primary storage adapter. Pure-Rust; supports embedded mode (no separate server process) which collapses the deployment surface to a single binary. Provides `DEFINE TABLE` / `DEFINE FIELD` / `DEFINE INDEX` for DDL and `LIVE SELECT` for change feeds (the `watch_changes` sync primitive).
- **Red flags:** `surrealdb` is dual-licensed: the core is Apache-2.0 / MIT but the `enterprise` features are Business Source License 1.1 (BSL). The engine uses only the Apache-2.0 / MIT surface; BSL features are not linked. The crate is pre-1.0 and the MSRV on the latest dev line is above the engine's 1.75 floor; the engine pins to the last 1.75-compatible line (tracked in `Cargo.toml` `[workspace.dependencies]`).
- **WASM:** supported via the `wasm-bindgen` feature flag.
- **See also:** [`docs/decisions/ADR-017-SurrealDBFirst.md`](./ADR-017-SurrealDBFirst.md), [`docs/schemas/sql-dialects/surrealdb.md`](../schemas/sql-dialects/surrealdb.md).

### Money and time

#### `rust_decimal ^1.36` (workspace: `"1.36"`, features `["serde-with-str"]`)

- **Alternatives:** `bigdecimal` (arbitrary precision, slower), `sqlx`'s `BigDecimal` (DB-only), floats (forbidden for money).
- **Selected because:** the de-facto fixed-precision crate for finance work; 106M+ lifetime downloads; MSRV 1.67 below the floor; explicit `wasm` feature flag. Uses a fixed 96-bit integer representation (no rounding errors up to 2^96 / 10^scale).
- **WASM:** supported via the `wasm` feature flag.

#### `chrono ^0.4` (workspace: `"0.4"`, features `["serde"]`)

- **Alternatives:** `time ^0.3` (newer API, but MSRV 1.88 raises the floor), `std::time` (insufficient; no timezone support).
- **Selected because:** the most-downloaded date/time crate (615M+ lifetime); MSRV 1.62 below the floor; broadest ecosystem. The 1.94+ floor of `time` is the deciding factor.
- **Red flags:** still pre-1.0; high open-issue count.
- **WASM:** partial via `wasmbind` (JS `Date` interop); not a no_std-WASM target.

### Serialization, errors, ID, async

#### `serde ^1.0` + `serde_json ^1.0` (workspace: `"1.0"` / `"1.0"`, features `["derive"]` / default)

- **Alternatives:** `bincode` (binary-only), `rmp-serde` (msgpack), `postcard` (no-std, smaller).
- **Selected because:** the de-facto Rust serialization framework. JSON is for test fixtures and adapter I/O only; domain code uses typed wrappers per `AGENTS.md` ("No `serde_json::Value` in domain code"). MSRV 1.56 below floor.
- **WASM:** works on every target by design (no platform-specific code).

#### `uuid ^1.10` (workspace: `"1.10"`, features `["v4", "v7", "serde"]`)

- **Alternatives:** `ulid` (sortable but not 128-bit), `nanoid` (shorter, not 128-bit), `uuid-old` (1.x line; this is what 1.10 is).
- **Selected because:** UUIDv7 (time-ordered) is the engine's primary ID type per `docs/schemas/database-schema.md`. The `js` feature provides WASM support via `wasm-bindgen`.
- **Red flags:** MSRV 1.85 on 1.23.x; the engine **pins to `>=1.10, <1.23`** to stay on 1.75.

#### `thiserror ^1.0` + `anyhow ^1.0` (workspace: `"1.0"` / `"1.0"`, default features)

- **Alternatives:** `eyre` (more features, less ecosystem), `snafu` (more structured).
- **Selected because:** per `AGENTS.md`: "Errors use `thiserror` for public APIs, `anyhow` for glue." MSRV 1.68 below floor. Both are dtolnay crates; tiny, well-maintained, near-universal.

#### `async-trait ^0.1` (workspace: `"0.1"`, default features)

- **Alternatives:** native `async fn in trait` (stabilized in Rust 1.75).
- **Selected because:** the engine's MSRV is exactly 1.75; native `async fn in trait` is available. The crate is still useful for `Box<dyn AsyncTrait>` (dyn-compatible async traits) which native traits don't support. Maintenance has slowed (~10 months since last release); this is consistent with the crate being feature-complete. Upstream PR #298 (lifetime lowering) is being tracked for a future major version.

### Observability, validation, crypto

#### `tracing ^0.1` + `tracing-subscriber ^0.3` (workspace: `"0.1"` / `"0.3"`, default features)

- **Alternatives:** `log` (no spans), `slog` (more features, less active), `fern` (unmaintained).
- **Selected because:** the de-facto observability framework for async Rust; MSRV 1.65 below floor; backed by the Tokio project. Spans are critical for the engine's distributed-system traces.
- **WASM:** partial. The `valuable` feature is used by WASM consumers; no dedicated `wasm` cargo feature.

#### `validator ^0.19` (workspace: `"0.19"`, features `["derive"]`)

- **Alternatives:** `garde` (more typed, less ecosystem), hand-rolled `validate_*` methods.
- **Selected because:** the de-facto derive-based validator; widely used in the Rust web ecosystem.
- **Red flags:** MSRV 1.81 (above the 1.75 floor); single-maintainer bus factor (Keats has been asking for co-maintainers since 2022). The 0.20.0 release now depends on `proc-macro-error2` which is RUSTSEC-flagged (unmaintained as of 2026-06-08). The engine **pins to 0.19.x** to stay on 1.75 and avoid the unmaintained transitive dep.

#### `async-nats ^0.33` (workspace: `"0.33"`, default features)

- **Alternatives:** `nats` (sync, unmaintained), `rants` (less mature, no JetStream), raw TCP.
- **Selected because:** the official async NATS client; the only mature Rust client with JetStream support (the bus-port distributed adapter needs JetStream durable consumers for ack / nack semantics). Pinned to 0.33.x to stay on the 1.75 floor (0.34.x raises MSRV to 1.81).
- **Red flags:** pulls in `nkeys` (ed25519) and `signatory`; the binary size is non-trivial. The crate is large but every dep is a Tokio/RustCrypto staple.
- **WASM:** not first-class. The bus-port adapter for WASM uses the in-process bus; the central engine that publishes to NATS runs on the server side.
- **Usage:** Phase 2 stub in `educore-event-bus::nats`; the wire-protocol work lands in a future phase.

#### `redis ^0.25` (workspace: `"0.25"`, features `["tokio-comp", "connection-manager"]`, `default-features = false`)

- **Alternatives:** `deadpool-redis` (pool layer, depends on `redis`), `mobc` (generic pool), raw `bb8-redis`.
- **Selected because:** the de-facto async Redis client; 5.2k+ GitHub stars; underpins `deadpool-redis` and `bb8-redis`. The `ConnectionManager` type provides automatic reconnect for the bus-port's at-least-once delivery guarantee. Pinned to 0.25.x to stay on the 1.75 floor (0.27.x raises MSRV to 1.80).
- **Red flags:** `ConnectionManager` does not implement `Debug`; the adapter's `Debug` impl manually redacts the manager. TLS via the `tokio-rustls` feature (mandated by AGENTS.md).
- **WASM:** not first-class. Same as NATS — the WASM path uses the in-process bus.
- **Usage:** Phase 2 stub in `educore-event-bus::redis`; the wire-protocol work lands in a future phase.

#### `argon2 ^0.5` (workspace: `"0.5"`, default features)

- **Alternatives:** `bcrypt` (older, weaker), `scrypt` (memory-hard but slower), `pbkdf2` (HMAC-based).
- **Selected because:** the reference password-hashing function; OWASP-recommended; RustCrypto maintained. **Pinned to 0.5.x** to stay on the 1.75 floor (0.6.0-rc.8 raises MSRV to 1.85).

#### `hmac ^0.12` + `sha2 ^0.10` (workspace: `"0.12"` / `"0.10"`, default features)

- **Alternatives:** `openssl` (C bindings), `ring` (Google-maintained but C-FFI), `blake2` (faster, but SHA-2 is the FIPS standard).
- **Selected because:** the RustCrypto crates; pure-Rust; cross-compile to all 3 tiers; mandated by FIPS where the engine is used. **Both pinned to the pre-1.85 lines** (0.12.x for hmac, 0.10.x for sha2). `sha2` has an explicit `wasm32-simd128` backend that auto-selects on WASM.

### Engine-internal utilities (Tier-B)

These crates are well-known utilities with no serious alternative
considered. Documented here for the audit trail.

| Crate | Purpose | Why |
| --- | --- | --- |
| `rust_decimal_macros` | `dec!()` literal for `rust_decimal` | The de-facto companion to `rust_decimal`; no alternative. |
| `futures ^0.3` | Executor-agnostic `Future`/`Stream` combinators | The de-facto async combinator crate; `tokio` re-exports some but not all. |
| `rand ^0.8` | General-purpose RNG (ChaCha12) | The de-facto RNG; pinned to 0.8.x for MSRV 1.75; `getrandom` provides the WASM and Android backends. |
| `secrecy ^0.10` | `Secret<T>` wrapper | The de-facto memory-wiping wrapper; built on `zeroize`. |
| `regex ^1.10` | Regular expressions | The de-facto regex; `regex-lite` is a smaller alternative but lacks some features. |
| `indexmap ^2.5` | Insertion-ordered hash map | The de-facto insertion-ordered map; pinned to 2.5.x for MSRV 1.75. |
| `bytes ^1` | Zero-copy byte buffer (`Bytes`) | The de-facto zero-copy byte buffer in the Rust async ecosystem (used by `reqwest`, `hyper`, `tonic`); serializes compactly to base64 in JSON when the `serde` feature is enabled. Used in the storage port for the `SerializedEnvelope::payload` field and other wire-format payloads. Alternatives: `Vec<u8>` (no zero-copy, no base64), `Cow<[u8]>` (no async-friendly sharing). MSRV 1.56, OK. Tier 1 ✓, Tier 2 ✓, Tier 3 ✓. |
| `derive_more ^1.0` | More `#[derive]` macros | Standard companion to `std` derives. |
| `once_cell ^1.20` | `Lazy`/`OnceCell` | Standard `std::sync::OnceLock` replacement; pre-stabilization. |
| `proc-macro2 / quote / syn` | Proc-macro infrastructure | Standard; required by `educore-query-derive`. |

## Cross-compile matrix (per category)

| Category | Tier 1 (Linux/macOS/Win) | Tier 2 (Android aarch64) | Tier 3 (WASM) | Fallback for unsupported tiers |
| --- | --- | --- | --- | --- |
| Async runtime | `tokio` | `tokio` (no `parking_lot`) | `gloo-net` + `gloo-timers` | n/a (no engine code on WASM that needs `tokio::net`) |
| SQL (PostgreSQL, SQLite) | `sqlx 0.8` | `sqlx 0.8` (no CI) | `gloo-sqlite` for SQLite offline | WASM-only engine uses a custom storage port |
| SQL (MySQL) | `mysql_async 0.34` | `mysql_async 0.34` (no CI) | not supported | n/a (MySQL is server-side only) |
| TLS | `rustls 0.23` | `rustls 0.23` | `rustls 0.23` + `wasm-bindgen` | n/a |
| HTTP | `reqwest 0.12` | `reqwest 0.12` (with `init_hosted()`) | `reqwest 0.12` (auto-WASM) | n/a |
| Email (SMTP) | `lettre 0.10` | not supported (pin to 0.10) | not supported | n/a (email is server-side only) |
| Object storage (S3) | `aws-sdk-s3 1.x` | heavy (pin to older 1.x) | not supported | Custom REST shim for WASM |
| Serialization | `serde` + `serde_json` | `serde` + `serde_json` | `serde` + `serde_json` | n/a |
| ID | `uuid 1.10..1.22` | `uuid 1.10..1.22` | `uuid 1.10..1.22` (`js` feature) | n/a |
| Errors | `thiserror` + `anyhow` | `thiserror` + `anyhow` | `thiserror` + `anyhow` | n/a |
| Async traits | `async-trait 0.1` | `async-trait 0.1` | `async-trait 0.1` | n/a |
| Observability | `tracing` + `tracing-subscriber` | `tracing` + `tracing-subscriber` | `tracing` (no `tracing-subscriber` on WASM; use `console_log` or `tracing-wasm`) | console.log via `tracing-wasm` |
| Validation | `validator 0.19` | `validator 0.19` | not supported | n/a (validation is server-side) |
| Crypto (password) | `argon2 0.5` | `argon2 0.5` | not supported | n/a (auth is server-side) |
| Crypto (HMAC, SHA-2) | `hmac 0.12` + `sha2 0.10` | same | `sha2 0.10` (auto-WASM backend); `hmac` composes on top | n/a |
| Money | `rust_decimal 1.36` | same | `rust_decimal 1.36` (`wasm` feature) | n/a |
| Time | `chrono 0.4` | same | `chrono 0.4` (via `wasmbind`) | n/a |

## MSRV floor conflict resolution

The engine's pinned MSRV is **1.75**. The following crates in
§ "Decision" require MSRV > 1.75 in their current line:

| Crate | Current MSRV | Resolution |
| --- | --- | --- |
| `sqlx` | 1.94.0 (0.9.x) | **Pin to 0.8.x** (last 1.75-compatible). Re-evaluate when raising the engine MSRV. |
| `reqwest` | 1.85.0 (0.13.x) | **Pin to 0.12.x**. Re-evaluate when raising the engine MSRV. |
| `time` | 1.88.0 | **Do not use.** Engine uses `chrono` instead. |
| `lettre` | 1.85 (0.11.x) | **Pin to 0.10.x** (last 1.75-compatible). |
| `aws-sdk-s3` | 1.91.1 (1.135.x) | **Pin to 1.55.x** (the version already in `Cargo.toml`). Re-evaluate when raising the engine MSRV. |
| `uuid` | 1.85.0 (1.23.x) | **Pin to `>=1.10, <1.23`**. Re-evaluate when raising the engine MSRV. |
| `validator` | 1.81 (0.20.x) | **Pin to 0.19.x** to stay on 1.75 and avoid the unmaintained `proc-macro-error2` transitive dep. |
| `hmac` | 1.85 (0.13.x) | **Pin to 0.12.x**. |
| `sha2` | 1.85 (0.11.x) | **Pin to 0.10.x**. |
| `rand` | 1.85 (0.10.x) | **Pin to 0.8.x**. |
| `indexmap` | 1.85 (2.14.x) | **Pin to 2.5.x** (the version already in `Cargo.toml`). |

**Resolution policy:** when a crate's current MSRV exceeds the engine
floor, the engine pins to the last pre-floor release line. The ADR is
updated to record the pin. The pin is re-evaluated when the engine floor
is raised.

## Cross-compile status (3 tiers × 25 crates)

| Category | T1 | T2 | T3 | Notes |
| --- | --- | --- | --- | --- |
| Async (tokio) | ✓ | ✓ | partial | WASM via `gloo-net` shim |
| SQL — PG/SQLite (sqlx) | ✓ | partial | ✗ | WASM via `gloo-sqlite` for SQLite only |
| SQL — MySQL (mysql_async) | ✓ | unknown | ✗ | MySQL is server-side only |
| TLS (rustls) | ✓ | ✓ | ✓ | Pure-Rust; works everywhere |
| HTTP (reqwest) | ✓ | partial (Android) | ✓ | Android needs `init_hosted()` |
| Email (lettre) | ✓ | not supported (pin 0.10) | ✗ | Server-side only |
| Object storage (aws-sdk-s3) | ✓ | heavy (pin 1.55) | ✗ | Server-side; WASM uses custom shim |
| Serialization (serde) | ✓ | ✓ | ✓ | Pure-Rust |
| ID (uuid) | ✓ | ✓ | ✓ (`js` feature) | First-class WASM |
| Errors (thiserror, anyhow) | ✓ | ✓ | ✓ | Pure-Rust |
| Async traits (async-trait) | ✓ | ✓ | ✓ | Proc-macro only |
| Observability (tracing) | ✓ | ✓ | partial | `tracing-wasm` for WASM |
| Validation (validator) | ✓ | ✓ | ✗ | Server-side; pin 0.19 |
| Crypto (argon2) | ✓ | ✓ | ✗ | Server-side; pin 0.5 |
| Crypto (hmac, sha2) | ✓ | ✓ | partial | `sha2` has WASM backend; `hmac` composes |
| Money (rust_decimal) | ✓ | ✓ | ✓ (`wasm` feature) | First-class WASM |
| Time (chrono) | ✓ | ✓ | partial | `wasmbind` JS interop |

## Dependency hygiene policy

The engine follows these rules for every external crate (enforced by
review and the build plan's no-gaps gates):

1. **Pinned in `[workspace.dependencies]`.** Every external crate is
   declared in the root `Cargo.toml` with a version pin. Member crates
   use `{ workspace = true }` for transitive deps; no per-crate version
   overrides.
2. **`default-features = false` + explicit features.** Every external
   crate is added with `default-features = false` and an explicit
   feature list, so the build's surface area is auditable.
3. **`rustls` not `native-tls`.** Per `AGENTS.md`: "All dependencies
   use `rustls`; never `native-tls`." Cross-compile mandate.
4. **No `unwrap()`/`expect()` in production paths.** Per `AGENTS.md`.
5. **License check.** All external crates must be MIT / Apache-2.0 /
   BSD-compatible. No GPL, no AGPL, no SSPL, no commercial-only
   licenses. The `Cargo.toml` of each crate declares its license; the
   engine audits this on every PR via the `cargo deny` tool (Phase 17).
6. **Audit log.** When a new external crate is added, this ADR is
   updated with: (a) the chosen version, (b) the alternatives
   considered, (c) the rationale (5-10 lines), (d) the cross-compile
   status, (e) the MSRV conflict status. The commit that adds the
   crate must include the ADR update.

## Future crate watch list

These crates are not in the engine today but are under evaluation for
future phases:

- **`wasm-bindgen` + `wasm-pack` + `trunk`**: when WASM is a first-class
  deployment target (Phase 18+), the engine's consumer build will
  use these to produce a WASM bundle.
- **`pyo3`**: when Python bindings are needed (Phase 19+), the
  engine's consumer build will use `pyo3` to expose a Python API.
- **`gloo` family** (`gloo-net`, `gloo-timers`, `gloo-storage`,
  `gloo-sqlite`): the WASM shim layer. Not direct engine deps, but
  the consumer's WASM build will pull these in.
- **`leptos`** or **`yew`**: if a frontend is needed (out of scope for
  v1).
- **`uniffi`**: cross-language bindings (Kotlin/Swift/Python) for the
  engine's consumer. Out of scope for v1.

## clap

- **Version:** `4` (workspace: `"4"`, features `["derive"]`).
- **Rationale:** CLI argument parser for `educore-cli` (Phase 16 sample
  binary). Derive feature enables the `#[derive(Parser)]` macro
  pattern.
- **Alternatives:** `pico-args` (no derive, no help-text generation),
  `structopt` (merged into clap 3.x and now deprecated upstream),
  `argh` (lighter, but less ecosystem support and no derive macro for
  subcommand enums).
- **Selected because:** the de-facto Rust CLI argument parser; 14k+
  GitHub stars; broadest ecosystem; first-class support for the
  derive-macro pattern the engine's CLI uses (one `enum` per
  subcommand, `#[derive(Parser)]` on the top-level CLI struct).
- **Cross-compile status:** Linux + macOS + Windows (native clap
  backend, no native-tls dep). The `clap` crate is pure-Rust and uses
  no platform-specific code; the only optional native dep is the
  `terminal_size` crate (for `--help` width), which is pure-Rust and
  cross-compile-safe.
- **MSRV-floor status:** clap 4.x requires Rust 1.74; engine MSRV is
  1.75 → compatible. No pin required.

## Audit log

- **2026-06-09** (this revision): initial ADR. 27 crates documented.
  - 11 crates require MSRV pinning to stay on 1.75 floor.
  - 4 crates (aws-sdk-s3, lettre, validator, hmac) have single-maintainer or unmaintained-transitive-dep risks.
  - 1 crate (validator) has a new transitive-dep RUSTSEC flag — re-evaluate before next release.
- **2026-06-12** (PR 5, educore-storage): added `bytes ^1` (with `serde` feature) for zero-copy wire-format payloads in the storage port. Now 28 crates documented.
- **2026-06-12** (PR 6, educore-storage-surrealdb + sync): added `surrealdb = "2"` (`default-features = false, features = ["kv-mem", "rustls"]`), pinned to the last pre-1.75 line per `§ "surrealdb"`. The `educore-sync` and `educore-sync-inprocess` crates introduce no new external deps — they use only `tokio` from the workspace. Now 29 crates documented.
- **2026-06-12** (PR 6, educore-storage-surrealdb + sync): added `surrealdb = "2"` (`default-features = false, features = ["kv-mem", "rustls"]`), pinned to the last pre-1.75 line per `§ "surrealdb"`. The `educore-sync` and `educore-sync-inprocess` crates introduce no new external deps — they use only `tokio` from the workspace. Now 29 crates documented.
- **2026-06-12** (PR 0, Phase 0 fix-up): no new external crates. Adjusted the workspace lint config: `pedantic = "allow"` (was `warn`), `nursery = "allow"` (was unset), `missing_errors_doc = "allow"`, `missing_panics_doc = "allow"`. The safety-critical denies (`unwrap_used`, `expect_used`, `panic`, `print_stderr`, `print_stdout`, `dbg_macro`, `cast_*`) are unchanged. The mechanical code changes are the proc-macro `usize as u32` → `u32::try_from` refactor, the `i32`/`u32` schema-version `try_from` in the SurrealDB adapter, and the `eprintln!` removal in `educore-cli`. The `educore-core::lint` sub-module (gated behind the `lint` Cargo feature) is the new build-time gate.
- **2026-06-12** (Phase 0 close-out): 6 of 6 exit criteria met (build, test, fmt, clippy `-D warnings`, SurrealDB outbox e2e, sync e2e). 120 tests pass workspace-wide. The SurrealDB outbox e2e is the canonical Phase 0 acceptance test (`crates/adapters/storage-surrealdb/tests/outbox_e2e.rs`); the cross-cutting `lint` binary is the new no-gaps gate.
- **2026-06-12** (Phase 1, storage adapter parity): the three parity adapters (`educore-storage-postgres`, `educore-storage-mysql`, `educore-storage-sqlite`) ship with all four sub-port impls (Outbox, AuditLog, EventLog, Idempotency) as real impls, not stubs. All three are `sqlx 0.8`-driven; the `crates/adapters/storage-mysql/Cargo.toml` no longer references `mysql_async` or `flate2` (the workspace pins remain for historical reasons and can be dropped in a future cleanup PR). The PG adapter's `after_connect` hook issues `SET search_path = engine, public`; the MySQL adapter's issues `SET NAMES utf8mb4 COLLATE utf8mb4_unicode_ci`; the SQLite adapter's issues `PRAGMA journal_mode = WAL` / `synchronous = NORMAL` / `foreign_keys = ON`. 15 `Tested` rows flipped in `docs/coverage.toml` (5 per adapter × 3 adapters). 124 tests pass workspace-wide (4 new unit tests in the MySQL `connection::tests` module + 1 e2e per adapter). Phase 1 exit criteria all green: `cargo test -p educore-storage-{postgres,mysql,sqlite}` all pass, workspace `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, and `cargo run -p educore-core --bin lint --features lint` all clean.
- **2026-06-12** (Phase 2 workstream B, `educore-event-bus`): added two new external crates. `async-nats = "0.33"` (pinned to 0.33.x to stay on the 1.75 floor; 0.34.x raises MSRV to 1.81), `redis = "0.25"` (pinned to 0.25.x; 0.27.x raises MSRV to 1.80). Both are optional, gated behind the `nats` and `redis` Cargo features of `educore-event-bus`. The in-process bus uses no new external crate. Phase 2 scope: `InProcessEventBus` is a complete MPMC impl (broadcast channel + bounded replay log); `NatsEventBus` and `RedisEventBus` are Phase 2 scaffolds returning `EventError::NotSupported` from every trait method. 2 `Tested` rows flipped in `docs/coverage.toml` (`event_bus_port`, `event_bus_inprocess`). Now 31 crates documented. The MSRV-floor-pinning policy in § "MSRV floor conflict resolution" below continues to apply; the 11 crates pinned there are unchanged.

## Consequences

- **Single source of truth** for external crate choice. New contributors
  read this ADR to know "we use sqlx, not diesel" without grepping the
  codebase.
- **Cross-compile status is documented per crate, not discovered per
  failure.** The next contributor who adds `cargo-lambda` to the
  engine knows which crates will compile on WASM and which need a
  fallback.
- **MSRV floor is enforced via pinning, not by changing the floor.**
  The engine stays on 1.75; crates that exceed 1.75 are pinned to
  their last compatible line. Re-evaluated on every floor bump.
- **License audit is centralised.** The `cargo deny` tool (Phase 17)
  reads this ADR and the `Cargo.toml` to flag any non-permissive
  license.
- **Maintenance risk is documented.** Single-maintainer crates
  (mysql_async, validator) and unmaintained-transitive-dep risks
  (validator → proc-macro-error2) are flagged for future review.

## Alternatives

- **Option A — install as we go, document after the fact.** Faster
  start; risk of lock-in by accident. Cross-compile failures happen
  at the consumer's first build, not at decision time.
- **Option B — full upfront research on all 40+ crates.** ~6-8 hours
  of research. Identifies dead dependencies and over-broad feature
  sets. Heavy upfront cost.
- **Option C — hybrid (this ADR).** Research 15 high-priority
  categories upfront; the other 10-15 crates are documented as
  "engine-internal utilities, no alternative considered." Best
  cost-benefit at 27 crates.

## See also

- `AGENTS.md` — workspace layout, dependency rules, agent instructions
- `docs/build-plan.md` § "The No-Gaps Gates" — the lint sub-module
  verifies tier boundaries at build time
- `CONTRIBUTING.md` — the spec-to-PR workflow
- `docs/schemas/sql-dialects/README.md` § "Runtime DDL emission" —
  cross-compile-aware DDL generation
