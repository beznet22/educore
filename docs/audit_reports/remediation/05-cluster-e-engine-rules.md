# Cluster E — Engine-Rule Violations

**Root cause:** `docs/code-standards.md` and `AGENTS.md` forbid
`unwrap()`, `expect()`, `panic!()`, `as` casts on numerics,
`serde_json::Value`, and `HashMap<String, T>` for domain data in
production code. Per the audit, every domain crate contains widespread
violations. Per `docs/build-plan.md:73`, `cargo clippy --workspace
--all-targets -- -D warnings` is a Phase 0 exit criterion — currently
this gate fails (per `docs/handoff/PHASE-15-HANDOFF.md`, pre-existing
clippy debt in `educore-settings` / `educore-documents` blocks the
workspace gate).

**Estimated findings:** ~400 (Medium-heavy; but each is one line to fix)

**Source ID prefixes:** `DOMAIN-*` (engine-rule subset), `DOM-*`,
`INFRA-*`, `CORE-*` (engine-rule subset), `CC-*` (subset)

**Blocks deploy:** Yes (per code-standards; mechanical CI gate)

**Estimated fix scope:** Large by line count but **mechanical**. Each
violation is 1-5 lines of code change. Suitable for batch-style PRs.

## Why these findings cluster

The engine's 9 rules (per `docs/code-standards.md` and `AGENTS.md`
§ "Engine Rules") are non-negotiable:

1. **No `unwrap()` / `expect()` / `panic!()`** in production paths.
2. **No `as` on numerics** — use `TryFrom` / `TryInto`.
3. **No `serde_json::Value`** in domain code.
4. **No `HashMap<String, T>`** for domain data.
5. **Compile-time safety over strings** — use macro-generated enums.
6. **Domain scopes via extension traits.**
7. **Closure-based nested relational filters** (`where_has`).
8. **Strict eager loading** (`.with(Relation::Foo)`).
9. **No SQL/NoSQL emission from macros.**

The audit found violations across all 10 domain crates. The violations
are concentrated in:
- `services.rs` files (domain services — heavy unwrap use)
- `aggregate.rs` files (constructor methods — heavy unwrap use)
- `events.rs` files (event payload serialization — `Value` use)
- `commands.rs` files (handler argument parsing)

Top 5 files by violation count:

| Count | File |
|---|---|
| 41 | `crates/domains/assessment/src/services.rs` |
| 30 | `crates/domains/finance/src/services.rs` |
| 17 | `crates/domains/communication/src/services.rs` |
| 17 | `crates/domains/academic/src/services.rs` |
| 16 | `crates/domains/attendance/src/services.rs` |

## Representative findings (sample)

| Source | ID | Sev | One-line |
|---|---|---|---|
| `wave1-academic.md` | DOM-AC-022 | M | `services.rs:42` uses `unwrap()` on validated input |
| `wave1-finance.md` | DOM-FIN-031 | M | `aggregate.rs:118` `as` cast truncates `i64` to `u32` |
| `wave1-events-domain.md` | CC-EVTS-007 | M | `events.rs:54` `serde_json::Value` in payload |
| `wave4-core.md` | CORE-019 | M | `src/lib.rs:23` `HashMap<String, Capability>` for domain data |
| `wave2-rbac.md` | CC-RBAC-022 | M | `value_objects.rs:88` `unwrap()` in `Capability::new` |

(Full list of 400+ findings is in the per-area appendices and the master table.)

## What fixing this requires

For each violation type:

**`unwrap()` / `expect()` / `panic!()`**

- Replace with `?` propagation, returning a `DomainError` variant.
- For tests, `unwrap()` is acceptable.
- For invariants that can never fire (e.g., "this value was validated
  above"), document the invariant in a `// SAFETY:` or
  `// INVARIANT:` comment and use `.expect("invariant: …")`. But per
  AGENTS.md, even this is forbidden in production. So the only option
  is to plumb the error.

**`as` casts**

- Replace with `TryFrom::try_from(...)?`.
- For numeric IDs (e.g., `u64 → u32`), use `Id::<T>::try_from(...)?`.

**`serde_json::Value`**

- Replace with typed wrappers or `#[serde(tag = "type")]` enums.
- For event payloads, use a typed `EventPayload` enum with one variant
  per event type.

**`HashMap<String, T>`**

- Replace with typed structs.
- For dynamic key spaces, use a typed `Index<K, V>` wrapper.

## Suggested fix sequence

This cluster is large but mechanical. Recommended approach:

1. **Run clippy with `#[warn(warnings)]` first** — `cargo clippy
   --workspace --all-targets` will produce a baseline of clippy lints
   that overlap with the engine rules.
2. **Enable the engine-rule clippy lints** in `Cargo.toml` (lint groups
   `clippy::unwrap_used`, `clippy::expect_used`, `clippy::panic`,
   `clippy::cast_possible_truncation`, etc.) as deny-by-default.
3. **Per-crate mechanical sweep** — for each crate, replace all
   `unwrap()` → `?`, `as` → `TryFrom`, `Value` → typed enum. One PR per
   crate. Boring but shippable.
4. **Bulk-replace with `sed` + `cargo fmt`** — for the most common
   patterns (`unwrap()` → `?`), a `sed` pass followed by manual review
   of compilation errors is fastest.
5. **Add a CI gate** — `cargo clippy --workspace --all-targets
   -- -D warnings` as a required check on every PR.

## Verification criteria

- `cargo clippy --workspace --all-targets -- -D warnings` exits 0.
- `cargo run -p educore-core --bin lint --features lint` reports zero
  anti-pattern violations.
- No `unwrap()` / `expect()` / `panic!()` in `crates/domains/*/src/*.rs`
  outside `#[cfg(test)]`.
- No `as` on numerics in `crates/domains/*/src/*.rs`.
- No `serde_json::Value` in `crates/domains/*/src/`.
- No `HashMap<String, T>` for domain data in `crates/domains/*/src/`.

## Risk if left unfixed

- `cargo clippy --workspace --all-targets -- -D warnings` continues to
  fail. The CI gate is unenforced.
- Production code crashes on edge cases that `unwrap` turns into
  panics.
- The audit's `wave7-workflows.md` WF-022 finding already shows
  in-production code paths that can panic.
- The audit's `wave4-core.md` CORE-019 finding shows that
  `HashMap<String, _>` use makes domain data un-typeable.

## Cross-cluster dependencies

- **Unblocks:** Cluster D (once lint detects these mechanically, the
  sweep becomes "fix what lint flags").
- **Depends on:** None — orthogonal to other clusters. Can be done in
  parallel with A, B, C, D.

## Recommended coupling

Per cluster D, once the lint runs, Cluster E becomes a continuous
hygiene task rather than a one-time sweep. Every new PR that introduces
an `unwrap()` is caught at PR time.

For the initial sweep, recommended coupling:

- When fixing a Cluster A / B / C / D file, also fix the engine-rule
  violations in that file. This avoids "I fixed the macro but the new
  function still has unwrap()" regression.

## Files involved

- All `.rs` files under `crates/domains/*/src/` (10 crates × ~10 files each)
- `crates/infra/*/src/` (3 crates)
- `crates/cross-cutting/*/src/` (7 crates)
- `crates/adapters/*/src/` (10 crates)
- `crates/tools/*/src/` (4 crates)

Total files touched: ~150. Total violations: ~400.
