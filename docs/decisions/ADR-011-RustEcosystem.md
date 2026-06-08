# ADR-011: Rust Ecosystem Standards

## Status

Accepted.

## Context

SMScore is a Rust crate ecosystem. The Rust ecosystem has
strong conventions: edition, MSRV, async runtime, error
handling, serialization, logging, observability, testing,
dependency management. The engine's standards must be
specific, not aspirational, because a vague standard
allows drift.

The school domain also has its own performance
characteristics. A daily attendance marking for 1,000
students happens in a 30-minute window. A payroll for 200
staff happens once a month. A report card for 1,000
students happens at the end of each term. The engine's
standards must be fast enough to handle these peaks
without tuning, on a small server.

The school domain also has its own correctness
characteristics. Money must not have rounding errors.
PII must not be logged. Time zones must not corrupt
timestamps. Identifiers must not be confused. The engine's
standards must encode these invariants.

## Decision

SMScore adopts the **Rust ecosystem's modern standards**
as the engineering baseline. These standards are
mandatory for every crate in the engine.

1. **Edition:** 2021.
2. **MSRV:** 1.75 (the latest stable LTS at the time of
   release). Crate-level `rust-version` is set.
3. **Async:** `tokio` is the runtime. The
   `smscore-core` crate re-exports `tokio` selectively
   so domain code does not import `tokio` directly.
4. **Error handling:** `thiserror` for public error
   types, `anyhow` for glue code (CLI, scripts). Domain
   errors are enums with `#[from]` conversions.
5. **Serialization:** `serde` is the framework. Domain
   events and commands use `#[derive(Serialize,
   Deserialize)]`. `serde_json::Value` is forbidden in
   domain code; typed wrappers are mandatory.
6. **Logging:** `tracing` for structured logs.
   `tracing-subscriber` is the only sanctioned
   subscriber. Sensitive data (PII, money values, auth
   tokens) is never logged.
7. **Testing:** `cargo test`, `#[tokio::test]` for
   async tests, `proptest` for property tests,
   `insta` for snapshot tests, `criterion` for
   benchmarks.
8. **Linting:** `cargo clippy --workspace --all-targets
   -- -D warnings` is the bar. `cargo fmt` is enforced.
9. **Documentation:** Every public item has a
   rustdoc comment. Crate-level `lib.rs` describes
   what the crate owns and what it depends on.
10. **Dependencies:** Minimum, audited, pinned. The
    workspace's `Cargo.lock` is committed. Dependencies
    are reviewed per PR.
11. **Numeric conversions:** `TryFrom` / `TryInto`.
    `as` on numerics is forbidden.
12. **Unsafe:** Forbidden in domain code. Adapters may
    use it only with a `// SAFETY:` comment and
    review.
13. **Panics:** `unwrap()`, `expect()`, `panic!` are
    forbidden in production paths.
14. **Cross-compilation:** `rustls` instead of
    `native-tls`. Builds verified for `x86_64-unknown-
    linux-gnu`, `aarch64-unknown-linux-gnu`,
    `x86_64-apple-darwin`, `x86_64-pc-windows-msvc`.
15. **Async-friendly APIs:** All I/O is async. Domain
    logic is sync wherever possible. No blocking I/O
    inside an async function.

## Consequences

### Positive

- **Predictable baseline.** A new contributor can
  read `code-standards.md` once and know the rules.
- **Tooling alignment.** Standard tools (`cargo
  clippy`, `cargo fmt`, `cargo test`) work out of
  the box.
- **MSRV clarity.** A consumer on an older toolchain
  knows the engine will not work; they upgrade.
- **Ecosystem familiarity.** A Rust developer reads
  the engine and recognizes idioms: `thiserror`,
  `serde`, `tracing`, `tokio`.
- **Performance.** `rustls`, `tokio`, `criterion`
  benchmarks, no `as` on numerics — the engine is
  fast and correct by default.
- **Safety.** No `unsafe` in domain code, no panics,
  no implicit `as` truncations.

### Negative

- **Strict MSRV.** Consumers on a slightly older
  toolchain cannot use the engine. We accept this
  for predictability.
- **No alternative async runtime.** The engine is
  `tokio`-bound. A consumer using `async-std` cannot
  reuse the engine's ports directly. We accept this
  for the ecosystem alignment.
- **Clippy is opinionated.** Some `clippy` lints are
  noise. We disable the noise and keep the
  substance.
- **Compile time.** The strict type system, the
  derives, the workspace size — compile time grows.
  We accept it for the safety.

### Mitigations

- The `smscore-core` crate re-exports common
  building blocks so domain crates have a
  consistent surface.
- A `cargo xtask` script (or equivalent) wires
  the standard CI checks.
- The `build-plan.md` describes a phased
  delivery that respects the standards from day
  one.
- A "Rust style" rustdoc page summarizes the
  standards for new contributors.

## Alternatives Considered

### 1. `async-std` as the async runtime

A smaller, simpler runtime. Rejected because
`tokio` is the de facto standard in the Rust
async ecosystem; the engine's value comes from
ecosystem alignment, not runtime novelty.

### 2. `no_std` for the core

The engine compiles without `std`. Rejected
because the engine's I/O assumptions
(async, threading, network) require `std`. A
`no_std` adapter for embedded devices is
possible but is a separate effort.

### 3. Custom error handling

A bespoke error type and propagation model.
Rejected because `thiserror` and `anyhow` are
the standard; custom errors confuse consumers.

### 4. JSON-only serialization

The engine uses `serde_json` directly. Rejected
because `serde` is the abstraction; consumers
can swap the format (bincode, postcard, CBOR)
without changing domain code.

### 5. No async

The engine is fully sync. Rejected because
storage adapters are async (PostgreSQL,
network APIs), and forcing them to block
would harm throughput. Async at the boundary,
sync in the domain — the standard pattern.

### 6. No MSRV

The engine supports "any reasonable Rust."
Rejected because the engine uses modern
features (impl Trait, async fn in trait,
etc.) and pinning an MSRV is a forcing function
for clarity.
