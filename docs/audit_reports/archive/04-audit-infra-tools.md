# 04 - Audit Appendix - Infra + Tools (7 crates)

**Scope:** wave4-core.md, wave4-query-derive.md, wave4-storage-port.md, wave4-storage-parity.md, wave4-testkit.md, wave4-cli-sdk.md, wave4-umbrella.md

**Total findings:** 189

**Severity distribution:** 43 critical, 56 high, 64 medium, 26 low


## Summary Table

| Target | Critical | High | Medium | Low | Total |
| --- | --- | --- | --- | --- | --- |
| Core (infra) (`CORE`) | 6 | 12 | 7 | 1 | 26 |
| Query-Derive (infra) (`INFRA-QD`) | 5 | 6 | 10 | 7 | 28 |
| Storage Port (infra) (`PORT-STORE`) | 13 | 15 | 6 | 2 | 36 |
| Storage Parity (tools) (`PAR`) | 7 | 11 | 9 | 4 | 31 |
| Testkit (tools) (`TOOL-TK`) | 2 | 5 | 13 | 8 | 28 |
| CLI + SDK (tools) (`CLI-SDK`) | 7 | 4 | 9 | 2 | 22 |
| Umbrella (`UMB`) | 3 | 3 | 10 | 2 | 18 |

## Core (infra) (target id prefix: `CORE`)

**Path:** `crates/infra/core/`  
**Total findings:** 26 (6 critical, 12 high, 7 medium, 1 low)


### FINDING 1 (id: `CORE-001`)

- **Source:** `docs/audit_reports/findings/wave4-core.md`
- **Severity:** Critical
- **Area:** infra
- **Location:** `crates/infra/core/src/lint.rs:107-156` (`runner::check_coverage_matrix`)

**Description:**

The coverage-matrix sync check declared by `docs/build-plan.md:1925-1932` (item 5 of the No-Gaps Gates) is implemented as a no-op. The function reads `docs/coverage.toml`, builds a `status_tested: Option<(String, usize)>` accumulator across the lines, and then explicitly discards the result with `let _ = status_tested;` (line 154). No `Violation` is ever appended to `_report`, regardless of whether a `Tested` row is missing a `tests = "..."` path or pointing at a nonexistent file. The check is gated by `#[cfg(feature = "lint")]` and declared in the lint's own docs as one of the five no-gaps gates; an empty pass would silently let any `Tested` row lie.

**Expected:**

`docs/build-plan.md:1925-1932`: "the lint reads `docs/coverage.toml` and verifies: Every `Tested` row has a `tests` path that exists." Plus `AGENTS.md` "Validation Checklist (per PR)": coverage matrix is a per-PR gate; the lint must enforce it.

**Evidence:**

```rust
  crates/infra/core/src/lint.rs:124-154
  let mut status_tested: Option<(String, usize)> = None;
  for (idx, line) in contents.lines().enumerate() {
      let trimmed = line.trim();
      if trimmed.starts_with("id = ") {
          status_tested = None;
      }
      if let Some(rest) = trimmed.strip_prefix("status = ") {
          let value = rest.trim().trim_matches('"');
          if value == "Tested" {
              status_tested = Some((String::new(), idx + 1));
          }
      }
      if trimmed.starts_with("tests = ") {
          if let Some((_, line_no)) = status_tested.as_mut() {
              *line_no = idx + 1;
          }
      }
      if trimmed.is_empty() || trimmed.starts_with("#") {
          continue;
      }
  }
  // The lightweight scan above flags *every* `Tested` row that
  // does not have a `tests = ...` field within its stanza.
  // We don't have a full parser, so the actual per-row
  // verification is delegated to Phase 1+ ...
  let _ = status_tested;
  ```

---

### FINDING 2 (id: `CORE-002`)

- **Source:** `docs/audit_reports/findings/wave4-core.md`
- **Severity:** Critical
- **Area:** infra
- **Location:** `crates/infra/core/src/lint.rs:1-300` (entire lint sub-module) vs. `docs/build-plan.md:1880-1935` (No-Gaps Gates item 2)

**Description:**

The lint sub-module's public surface is `pub fn run(repo_root) -> LintReport` (`src/lint.rs:160-166`), which calls only `check_coverage_matrix` and `check_anti_patterns`. `docs/build-plan.md` § The No-Gaps Gates item 2 enumerates FIVE categories of checks: (1) spec→code direction (`tables.md` → `aggregate.rs`, `commands.md` → `commands.rs`, `events.md` → `events.rs`, `migrations/engine/*.sql` → `create_<table>_ddl()`); (2) code→spec direction; (3) anti-patterns; (4) parity; (5) coverage-matrix sync. The implementation only attempts (3) and (5), and (5) is a no-op (see CORE-001). Checks 1, 2, and 4 are entirely absent; the runner has no functions to perform them.

**Expected:**

`docs/build-plan.md:1880-1935`: the `lint::run` runner is the per-crate gate that catches "missing handlers, anti-patterns, reverse-direction drift, and matrix lies." All five sub-checks must be invoked.

**Evidence:**

```rust
  crates/infra/core/src/lint.rs:160-166
  pub fn run(repo_root: &Path) -> LintReport {
      let mut report = LintReport::default();
      runner::check_coverage_matrix(repo_root, &mut report);
      runner::check_anti_patterns(repo_root, &mut report);
      report
  }
  ```
  No `check_spec_to_code`, `check_code_to_spec`, `check_parity`, or `check_migration_ddl` functions exist anywhere in `src/lint.rs`.

---

### FINDING 3 (id: `CORE-003`)

- **Source:** `docs/audit_reports/findings/wave4-core.md`
- **Severity:** Critical
- **Area:** infra
- **Location:** `crates/infra/core/src/lint.rs:181-238` (`scan_file_for_anti_patterns`)

**Description:**

The anti-pattern scan only detects `.unwrap()`, `.unwrap_err()`, `panic!(`, `todo!()`, `unimplemented!()` (needle array at line 220). It does NOT detect `.expect(`, which `AGENTS.md` and `docs/code-standards.md` explicitly forbid alongside `unwrap`. It does NOT detect `as` casts on numerics (also banned), `serde_json::Value` (banned in domain code), or `HashMap<String, T>` (banned for domain data) — all four of these are called out by name in the build plan § The No-Gaps Gates item 3 and in `AGENTS.md` "Validation Checklist".

**Expected:**

`docs/build-plan.md:1917-1923`: "Anti-patterns: No `unimplemented!()`, `todo!()`, or `// TODO: implement` in production code (test code is exempt via `#[cfg(test)]` detection). No `as` on numerics in domain crates (per `AGENTS.md`'s `as` ban). No `serde_json::Value` in domain code. No `HashMap<String, T>` for domain data."

**Evidence:**

```rust
  crates/infra/core/src/lint.rs:220-225
  for needle in [
      ".unwrap()",
      ".unwrap_err()",
      "panic!(",
      "todo!()",
      "unimplemented!()",
  ] {
      if line.contains(needle) {
          ...
      }
  }
  ```

---

### FINDING 4 (id: `CORE-004`)

- **Source:** `docs/audit_reports/findings/wave4-core.md`
- **Severity:** Critical
- **Area:** infra
- **Location:** `crates/infra/core/src/query.rs:74-89` (Value enum + manual `impl Eq for Value`)

**Description:**

`Value::F64(f64)` is a variant of `Value` (line 74), but the manual `impl Eq for Value {}` at line 89 claims `Value` satisfies the `Eq` trait. `f64` does not implement `Eq` because `NaN != NaN` violates reflexivity. The manual impl is allowed by the compiler because `Eq` is a marker trait with no methods and Rust does not field-check it, but the contract is unsound — any consumer that relies on `Value` being `Eq` (e.g. for `HashMap<Value, _>` or `BTreeMap<Value, _>`) will silently produce NaN-keyed buckets or panic at runtime in `BTreeMap`. This contradicts the engine's type-safety rules.

**Expected:**

`docs/code-standards.md` "Type Safety" / "Forbidden Patterns": "No `HashMap<String, T>` for domain data. Use typed structs." Indirectly: typed value types must be honestly `Eq` if marked so. `AGENTS.md` "Type Safety": "Enforce full type safety at all times."

**Evidence:**

```rust
  crates/infra/core/src/query.rs:74-89
  /// A 64-bit floating point.
  F64(f64),
  ...
  }
  
  impl Eq for Value {}
  ```

---

### FINDING 5 (id: `CORE-005`)

- **Source:** `docs/audit_reports/findings/wave4-core.md`
- **Severity:** Critical
- **Area:** infra
- **Location:** `crates/infra/core/src/ids.rs:10` (rustdoc link) vs. `crates/infra/core/src/clock.rs` (no `id_gen` module)

**Description:**

The rustdoc for `pub mod ids;` references the path `crate::id_gen::IdGenerator` at line 10, but `educore-core` has no `id_gen` module. `IdGenerator` lives in `crate::clock` (`src/clock.rs:79-117`). Any consumer that follows the documented path receives a compile error. This is a dead doc-link and a documentation/code drift blocker.

**Expected:**

`docs/code-standards.md` "Documentation: Every public item has a rustdoc comment." All intra-doc links must resolve. `AGENTS.md`: "Documentation: complete (~302 markdown files, 15 domain specs × 11 files each = 165 spec files)."

**Evidence:**

```rust
  crates/infra/core/src/ids.rs:10
  //! generated by the [`IdGenerator`](crate::id_gen::IdGenerator) port
  ```
  `grep -n "pub mod id_gen" crates/infra/core/src/` returns no matches. `IdGenerator` is defined at `crates/infra/core/src/clock.rs:79`.

---

### FINDING 6 (id: `CORE-006`)

- **Source:** `docs/audit_reports/findings/wave4-core.md`
- **Severity:** Critical
- **Area:** infra
- **Location:** `crates/infra/core/src/lint.rs:160-166` (`pub fn run`) and absence of tier-boundary check

**Description:**

`AGENTS.md` § Tier System mandates that "the `educore-core::lint` sub-module verifies at build time that a crate in `crates/domains/` does not import from `crates/adapters/` or `crates/tools/`, and that a crate in `crates/cross-cutting/` does not import from `crates/domains/`, `crates/adapters/`, or `crates/tools/`." The implemented `lint::run` has no check that walks `Cargo.toml` files or `mod` declarations to enforce these tier boundaries. The function only invokes the coverage-matrix (no-op, see CORE-001) and the anti-pattern scan (incomplete, see CORE-003).

**Expected:**

`AGENTS.md` § Tier System ("Tier boundary enforcement"). `docs/build-plan.md:1830-1934`: the lint is the "per-crate gate" that catches all five categories.

**Evidence:**

```rust
  crates/infra/core/src/lint.rs:160-166
  pub fn run(repo_root: &Path) -> LintReport {
      let mut report = LintReport::default();
      runner::check_coverage_matrix(repo_root, &mut report);
      runner::check_anti_patterns(repo_root, &mut report);
      report
  }
  ```
  No `check_tier_boundary`, no Cargo.toml walking, no import-graph analysis.

---

### FINDING 10 (id: `CORE-010`)

- **Source:** `docs/audit_reports/findings/wave4-core.md`
- **Severity:** High
- **Area:** infra
- **Location:** `crates/infra/core/src/tenant.rs:55-83` (`TenantContext::for_user` and `system` constructors)

**Description:**

Neither constructor validates `school_id`. A caller can pass `PUBLIC_SCHOOL_ID` (the nil UUID, defined at `src/ids.rs:293`) to `for_user`, producing a `TenantContext` whose `school_id` is the public-content anchor — but `user_type` is set to `Teacher`/`Parent`/etc. (a real school-scoped role). The RLS policy at `docs/schemas/tenancy-schema.md` § 4 expects `school_id = PUBLIC` to be reserved for global/public-site aggregates; mixing it with a school-scoped actor role bypasses tenant isolation silently. No `Result` is returned and no `DomainError::Validation` is raised.

**Expected:**

`docs/schemas/tenancy-schema.md` § 3: "`school_id` is mandatory and non-nil for school-scoped actors." Plus `docs/code-standards.md`: "All fallible APIs return `Result<T, DomainError>`."

**Evidence:**

```rust
  crates/infra/core/src/tenant.rs:55-83
  pub fn for_user(
      school_id: SchoolId,
      actor_id: UserId,
      correlation_id: CorrelationId,
      user_type: UserType,
  ) -> Self {
      Self {
          school_id,
          actor_id,
          ...
      }
  }
  ```
  No `if school_id.is_public() { return Err(...) }` check; signature returns `Self`, not `Result<Self, DomainError>`.

---

### FINDING 11 (id: `CORE-011`)

- **Source:** `docs/audit_reports/findings/wave4-core.md`
- **Severity:** High
- **Area:** infra
- **Location:** `crates/infra/core/Cargo.toml:24-34` (`[dependencies]`)

**Description:**

The crate declares six workspace dependencies that are not referenced anywhere in `src/`: `derive_more`, `validator`, `secrecy`, `async-trait`, `indexmap`, `tracing`. `cargo check` succeeds only because `cargo` does not error on unused dependencies by default, but the workspace's own `cargo build` rules (AGENTS.md § Package Manager) require every dependency to be used. Cargo-feature bloat grows compile times across the 33 downstream crates that depend on `educore-core`.

**Expected:**

`AGENTS.md` § Package Manager: "Use **cargo** to manage dependencies and build targets." Unused dependencies should be removed via `cargo remove`.

**Evidence:**

```toml
  crates/infra/core/Cargo.toml:24-34
  [dependencies]
  serde = { workspace = true }
  thiserror = { workspace = true }
  anyhow = { workspace = true }
  uuid = { workspace = true }
  chrono = { workspace = true }
  indexmap = { workspace = true }
  tracing = { workspace = true }
  async-trait = { workspace = true }
  derive_more = { workspace = true }
  validator = { workspace = true }
  secrecy = { workspace = true }
  ```
  `grep -n "derive_more\|validator::\|secrecy::\|async_trait::\|indexmap::\|tracing::" crates/infra/core/src/*.rs crates/infra/core/src/bin/*.rs` returns no matches.

---

### FINDING 12 (id: `CORE-012`)

- **Source:** `docs/audit_reports/findings/wave4-core.md`
- **Severity:** High
- **Area:** infra
- **Location:** `crates/infra/core/src/query.rs:262-278` (`QueryNode::is_empty`)

**Description:**

`is_empty()` returns `true` only when ALL leaves are `And`/`Or`/`Not` whose children recursively empty. The doc comment claims "a degenerate `And(And(...), And(...))` with all-empty children collapses to this in the macro emission," implying that a query with no filters collapses to empty. The implementation never returns `true` for a leaf node (every variant besides `And`/`Or`/`Not` hits the `_ => false` arm at line 270). The unit test at line 539 explicitly documents the contradiction: "an `IsNull` is not degenerate. So this returns false." The macro therefore cannot produce a sentinel "empty filter" node; a query with `Eq(field, value)` is never empty, which means callers cannot use `is_empty()` to skip filter emission.

**Expected:**

`docs/query_layer.md` § "Empty query": an empty filter tree must be representable so adapters can elide the `WHERE` clause. The doc comment at line 261-264 explicitly promises this collapsing behavior.

**Evidence:**

```rust
  crates/infra/core/src/query.rs:262-278
  #[must_use]
  pub fn is_empty(&self) -> bool {
      match self {
          Self::And(a, b) => a.is_empty() && b.is_empty(),
          Self::Or(a, b) => a.is_empty() && b.is_empty(),
          Self::Not(inner) => inner.is_empty(),
          _ => false,
      }
  }
  ```
  No `Self::Empty` variant; leaf nodes are never empty.

---

### FINDING 13 (id: `CORE-013`)

- **Source:** `docs/audit_reports/findings/wave4-core.md`
- **Severity:** High
- **Area:** infra
- **Location:** `crates/infra/core/src/clock.rs:203-230` (`fn deterministic_v7`)

**Description:**

The private `deterministic_v7` helper uses seven `as u8` casts on a `u64` counter (lines 212-226), truncating from 64 to 8 bits each time. AGENTS.md § Agent Instructions → Type Safety: "No `as` casts that truncate or lose data. Use `TryFrom` / `TryInto` with proper error handling." Even though the function is private and the truncation is intentional (encoding the counter into UUID bytes), the casts would be flagged by the lint that AGENTS.md mandates and contradict the engine's strict no-`as` policy.

**Expected:**

`AGENTS.md` § Agent Instructions → Type Safety: "No `as` casts that truncate or lose data." `docs/code-standards.md`: "Numeric conversions use `TryFrom`/`TryInto`. `as` is forbidden on numerics."

**Evidence:**

```rust
  crates/infra/core/src/clock.rs:212-226
  bytes[6] = 0x70 | (counter & 0x0f) as u8;
  bytes[7] = ((counter >> 4) & 0xff) as u8;
  bytes[8] = 0x80 | ((counter >> 12) & 0x3f) as u8;
  bytes[9] = ((counter >> 18) & 0xff) as u8;
  bytes[10] = ((counter >> 26) & 0xff) as u8;
  bytes[11] = ((counter >> 34) & 0xff) as u8;
  bytes[12] = ((counter >> 42) & 0xff) as u8;
  bytes[13] = ((counter >> 50) & 0xff) as u8;
  bytes[14] = ((counter >> 58) & 0x03) as u8;
  ```

---

### FINDING 14 (id: `CORE-014`)

- **Source:** `docs/audit_reports/findings/wave4-core.md`
- **Severity:** High
- **Area:** infra
- **Location:** `crates/infra/core/src/clock.rs:104-118` (`TestClock::set` and `advance`)

**Description:**

`TestClock::set` documents "If the underlying mutex is poisoned (a panic occurred while holding it), the clock is set to the test's epoch as a safe default." The actual implementation at line 110 calls `poisoned.into_inner()` to recover the value and overwrites it with the requested `t` (line 114 `*g = t;`), so the recovery is correct. However, `TestClock::advance` at line 117-130 uses `.unwrap_or(chrono::DateTime::<chrono::Utc>::MAX_UTC)` (line 127) — this is a silent fallback that the doc comment describes as "the clock is clamped to the representable maximum." The fallback is correct for `checked_add_signed` overflow but is invisible to tests: a test that advances by an excessive duration silently pins the clock at `MAX_UTC` rather than failing, masking bugs in test setup. The lint the engine mandates would flag the `unwrap_or` if it scanned test code, but the needle array does not include `unwrap_or` (see CORE-003).

**Expected:**

`AGENTS.md` § Agent Instructions → Type Safety: "No `unwrap()` or `expect()` in production paths." Plus: tests should fail loud on overflow, not silently clamp.

**Evidence:**

```rust
  crates/infra/core/src/clock.rs:117-130
  pub fn advance(&self, by: chrono::Duration) {
      let mut g = match self.inner.lock() {
          Ok(g) => g,
          Err(poisoned) => poisoned.into_inner(),
      };
      let next = g
          .as_datetime()
          .checked_add_signed(by)
          .unwrap_or(chrono::DateTime::<chrono::Utc>::MAX_UTC);
      *g = Timestamp::from_datetime(next);
  }
  ```

---

### FINDING 15 (id: `CORE-015`)

- **Source:** `docs/audit_reports/findings/wave4-core.md`
- **Severity:** High
- **Area:** infra
- **Location:** `crates/infra/core/src/query.rs:447-472` (`to_relational_node`)

**Description:**

`to_relational_node` discards the field identity of every leaf variant, replacing it with the `RelationalField` placeholder (lines 449-461 all emit `QueryNode::Variant(RelationalField, ...)`). The function comment at line 437-446 explains that the placeholder "is used because the related aggregate's field type is unknown to the parent builder; the storage adapter resolves the actual column names at translation time via the `Relation::name`." But because every leaf in the relational subtree maps to the same `RelationalField` (whose `column_name` returns `"<relation>"` per `src/query.rs:388`), the storage adapter receives no field information whatsoever — only the relation's `name` from the wrapping `HasRelation` variant. Any two filters in the same relational closure (e.g. `parent.eq(ParentField::Name, "x")` AND `parent.gt(ParentField::Age, 18)`) become indistinguishable in the AST.

**Expected:**

`docs/query_layer.md` § "Nested relational filters" and `docs/code-standards.md` "Compile-time safety over strings." Filters must be uniquely identifiable at the AST level.

**Evidence:**

```rust
  crates/infra/core/src/query.rs:448-461
  QueryNode::Eq(_, v) => QueryNode::Eq(RelationalField, v),
  QueryNode::Ne(_, v) => QueryNode::Ne(RelationalField, v),
  QueryNode::Lt(_, v) => QueryNode::Lt(RelationalField, v),
  QueryNode::Lte(_, v) => QueryNode::Lte(RelationalField, v),
  QueryNode::Gt(_, v) => QueryNode::Gt(RelationalField, v),
  QueryNode::Gte(_, v) => QueryNode::Gte(RelationalField, v),
  QueryNode::In(_, v) => QueryNode::In(RelationalField, v),
  QueryNode::NotIn(_, v) => QueryNode::NotIn(RelationalField, v),
  QueryNode::Between(_, lo, hi) => QueryNode::Between(RelationalField, lo, hi),
  QueryNode::IsNull(_) => QueryNode::IsNull(RelationalField),
  QueryNode::IsNotNull(_) => QueryNode::IsNotNull(RelationalField),
  QueryNode::Like(_, p) => QueryNode::Like(RelationalField, p),
  QueryNode::ILike(_, p) => QueryNode::ILike(RelationalField, p),
  ```

---

### FINDING 16 (id: `CORE-016`)

- **Source:** `docs/audit_reports/findings/wave4-core.md`
- **Severity:** High
- **Area:** infra
- **Location:** `crates/infra/core/src/ids.rs:54-65` (`UserId`, `SchoolId`, `EventId`, ... field visibility)

**Description:**

All six typed identifier wrappers (`UserId`, `SchoolId`, `EventId`, `CorrelationId`, `SessionId`, `IdempotencyKey`) expose `pub Uuid` as a transparent field. A consumer can mutate the inner UUID in place (e.g. `let mut id = some_school_id; id.0 = another_uuid;`) without going through any constructor. This bypasses the newtype pattern's discipline and makes the documented guarantee ("Two identifiers that share the same underlying UUID but have different Rust types are not interchangeable") enforceable only at construction time, not at use time. `from_uuid_checked` exists for v7 validation but is bypassed by direct field assignment.

**Expected:**

`docs/code-standards.md` "DDD Rules: Identifiers are typed (`StudentId`, `GuardianId`, ...), not raw `u64`." Plus "Value objects are immutable." AGENTS.md § Engine Rules: "Compile-time safety over strings."

**Evidence:**

```rust
  crates/infra/core/src/ids.rs:54-89
  pub struct UserId(pub Uuid);
  ...
  pub struct SchoolId(pub Uuid);
  ...
  pub struct EventId(pub Uuid);
  ...
  pub struct CorrelationId(pub Uuid);
  ...
  pub struct SessionId(pub Uuid);
  ...
  pub struct IdempotencyKey(pub Uuid);
  ```

---

### FINDING 17 (id: `CORE-017`)

- **Source:** `docs/audit_reports/findings/wave4-core.md`
- **Severity:** High
- **Area:** infra
- **Location:** `crates/infra/core/src/ids.rs:221-259` (six `From<Uuid>` impls)

**Description:**

`From<Uuid>` is implemented infallibly for all six identifier types, wrapping the `Uuid` without checking its version. `from_uuid_checked` exists at line 95-103 specifically to validate v7, but `From<Uuid>` provides an unchecked bypass. A consumer who writes `let id: SchoolId = uuid_v4.into();` (or worse, a mis-typed JSON deserializer that materializes an arbitrary UUID) silently constructs a non-v7 identifier. The engine's invariants ("All identifiers are UUIDv7") are violated with no compile-time or runtime signal.

**Expected:**

`docs/schemas/database-schema.md` § 1.4 (cited in `ids.rs:36-39`): "UUIDv7 as the default identifier." Plus `AGENTS.md` engine rule: "Compile-time safety over strings."

**Evidence:**

```rust
  crates/infra/core/src/ids.rs:221-259
  impl From<Uuid> for UserId {
      fn from(u: Uuid) -> Self {
          Self(u)
      }
  }
  
  impl From<Uuid> for SchoolId {
      fn from(u: Uuid) -> Self {
          Self(u)
      }
  }
  ...
  ```

---

### FINDING 18 (id: `CORE-018`)

- **Source:** `docs/audit_reports/findings/wave4-core.md`
- **Severity:** High
- **Area:** infra
- **Location:** `crates/infra/core/src/error.rs:185-200` (`impl From<String>` and `impl From<&str>` for `DomainError`)

**Description:**

`From<String>` and `From<&str>` for `DomainError` unconditionally map to `DomainError::Validation`. Any string — including a "NotFound: student missing" or "Conflict: version stale" message — is silently downgraded to a validation error. Callers using `?` on a `String` error from an adapter lose semantic information. The blanket impl also conflicts with the engine's intent that errors carry a precise `kind` discriminant (see CORE-007).

**Expected:**

`docs/code-standards.md` "Error Handling: Tests assert on error variants, not on display strings. Engine-level errors include a `kind` discriminant." Errors must preserve semantic category end-to-end.

**Evidence:**

```rust
  crates/infra/core/src/error.rs:185-200
  impl From<String> for DomainError {
      #[inline]
      fn from(s: String) -> Self {
          Self::Validation(s)
      }
  }
  
  impl From<&str> for DomainError {
      #[inline]
      fn from(s: &str) -> Self {
          Self::Validation(s.to_owned())
      }
  }
  ```

---

### FINDING 7 (id: `CORE-007`)

- **Source:** `docs/audit_reports/findings/wave4-core.md`
- **Severity:** High
- **Area:** infra
- **Location:** `crates/infra/core/src/error.rs:71-80` (`DomainError::kind`)

**Description:**

`kind()` returns `ErrorKind::Validation` for BOTH `DomainError::Validation` AND `DomainError::NotSupported` (line 75). The doc comment claims "The set of variants is closed; new kinds require a major version bump" but two distinct variants collapse into one `ErrorKind`. Callers that branch on `kind()` cannot distinguish "input failed validation" from "the adapter does not support this operation" — the two are operationally different (validation is a 4xx client error; `NotSupported` is a 501-equivalent server-side capability gap). The `ErrorKind` enum is also missing a `NotSupported` discriminant entirely; the variant exists on `DomainError` but has no kind.

**Expected:**

`docs/code-standards.md` "Error Handling: Engine-level errors include a `kind` discriminant (`Validation`, `NotFound`, `Conflict`, `Forbidden`, `Infrastructure`)." Plus `docs/schemas/event-schema.md` (per the error.rs doc-comment): one variant per kind, no collisions.

**Evidence:**

```rust
  crates/infra/core/src/error.rs:71-80
  pub const fn kind(&self) -> ErrorKind {
      match self {
          Self::Validation(_) | Self::NotSupported(_) => ErrorKind::Validation,
          Self::NotFound(_) => ErrorKind::NotFound,
          ...
      }
  }
  ```
  `ErrorKind` enum at `error.rs:135-156` has no `NotSupported` variant.

---

### FINDING 8 (id: `CORE-008`)

- **Source:** `docs/audit_reports/findings/wave4-core.md`
- **Severity:** High
- **Area:** infra
- **Location:** `crates/infra/core/src/value_objects.rs:212-225` (`Etag::placeholder`)

**Description:**

The doc comment claims "We construct via the public API for symmetry with the rest of the code, but the result is `unwrap_or`-safe — if the validator is ever changed to reject it, the unit test in this file's `tests` mod surfaces the regression." In fact the function does NOT invoke the public API — it directly constructs `Self("00000000000000000000000000000000".to_owned())`, bypassing `Etag::new`'s validator entirely. The "surfaces the regression" claim is false; if the validator is later changed to reject `"0000...0000"`, `Etag::placeholder()` will continue to construct an invalid `Etag` with no test failure.

**Expected:**

`docs/code-standards.md` "Value objects are immutable and validated at construction." All construction paths must route through the validator; bypasses must be flagged by tests.

**Evidence:**

```rust
  crates/infra/core/src/value_objects.rs:212-225
  pub fn placeholder() -> Self {
      // ... We construct via the public API for symmetry
      // with the rest of the code, but the result is
      // `unwrap_or`-safe ...
      Self("00000000000000000000000000000000".to_owned())
  }
  ```
  No call to `Etag::new` anywhere in the function.

---

### FINDING 9 (id: `CORE-009`)

- **Source:** `docs/audit_reports/findings/wave4-core.md`
- **Severity:** High
- **Area:** infra
- **Location:** `crates/infra/core/src/tenant.rs:25-50` (`TenantContext` struct fields)

**Description:**

`TenantContext` exposes seven `pub` fields (`school_id`, `actor_id`, `session_id`, `correlation_id`, `causation_id`, `user_type`, `locale`, `timezone`). Consumers can mutate any field directly after construction, bypassing `for_user`/`system`/the builder. This contradicts the engine's "value object is immutable" rule and the tenancy-schema spec which states the context is "immutable for the lifetime of a single command."

**Expected:**

`docs/schemas/tenancy-schema.md` § 3 (cited in `tenant.rs:7-14`): "The `TenantContext` is **immutable** for the lifetime of a single command." Plus `docs/code-standards.md`: "Value objects are immutable and validated at construction."

**Evidence:**

```rust
  crates/infra/core/src/tenant.rs:25-50
  pub struct TenantContext {
      /// The active school. Mandatory.
      pub school_id: SchoolId,
      /// The active user. Use [`TenantContext::system`] for
      /// system-issued commands (jobs, migrations).
      pub actor_id: UserId,
      /// Optional session boundary.
      pub session_id: Option<SessionId>,
      /// Propagated to every event emitted by the command.
      pub correlation_id: CorrelationId,
      /// For chained commands, the id of the event that caused this
      /// command. `None` for top-level commands.
      pub causation_id: Option<EventId>,
      ...
      pub user_type: UserType,
      pub locale: Locale,
      pub timezone: TimeZone,
  }
  ```

---

### FINDING 19 (id: `CORE-019`)

- **Source:** `docs/audit_reports/findings/wave4-core.md`
- **Severity:** Medium
- **Area:** infra
- **Location:** `crates/infra/core/src/value_objects.rs:62-79` (`Timestamp::epoch`)

**Description:**

`Timestamp::epoch()` uses a `match` over `DateTime::<Utc>::from_timestamp(0, 0)` with a fallback `from_timestamp_nanos(0)` to avoid an `expect`. The two branches both produce the same value (Unix epoch) on every supported chrono version, so the fallback is unreachable. The convoluted double-construction is documented as "satisfies the engine's no-`expect` rule while preserving the const signature" but introduces a redundant code path and obscures intent. A `const fn` that returns the constant string would be simpler.

**Expected:**

`AGENTS.md` § Type Safety / "no `expect`": bypass via unreachable branch is acceptable but should not be reflected in production logic.

**Evidence:**

```rust
  crates/infra/core/src/value_objects.rs:62-79
  pub const fn epoch() -> Self {
      // `from_timestamp(0, 0)` is total and always returns the
      // epoch on every supported chrono version; the `match`
      // is exhaustive without a panic arm. This satisfies the
      // engine's no-`expect` rule while preserving the const
      // signature.
      match DateTime::<Utc>::from_timestamp(0, 0) {
          Some(dt) => Self(dt),
          None => Self(DateTime::<Utc>::from_timestamp_nanos(0)),
      }
  }
  ```

---

### FINDING 20 (id: `CORE-020`)

- **Source:** `docs/audit_reports/findings/wave4-core.md`
- **Severity:** Medium
- **Area:** infra
- **Location:** `crates/infra/core/src/tenant.rs:204-243` (`Locale` and `TimeZone` constructors)

**Description:**

`Locale::new(s: impl Into<String>)` and `TimeZone::new(s: impl Into<String>)` are infallible constructors that accept any string with no BCP 47 / IANA validation. The doc comments at line 199 and line 235 state "The engine does not validate the tag — consumers normalize at the boundary." This contradicts the engine's value-object rule ("validated at construction") and pushes normalization burden to every consumer. A `Locale("totally bogus")` and `TimeZone("not_a_tz")` are silently accepted; downstream rendering and DB columns will fail opaquely.

**Expected:**

`docs/code-standards.md` "Value objects are immutable and validated at construction." `AGENTS.md` Validation Checklist: "Public APIs documented."

**Evidence:**

```rust
  crates/infra/core/src/tenant.rs:204-243
  impl Locale {
      /// Constructs a `Locale` from a raw string. The string is not
      /// validated against the IANA / BCP 47 registry; the engine
      /// treats the value as opaque.
      #[must_use]
      pub fn new(s: impl Into<String>) -> Self {
          Self(s.into())
      }
      ...
  }
  
  impl TimeZone {
      /// Constructs a `TimeZone` from a raw string. The string is not
      /// validated against the IANA tz database; the engine treats
      /// the value as opaque.
      #[must_use]
      pub fn new(s: impl Into<String>) -> Self {
          Self(s.into())
      }
      ...
  }
  ```

---

### FINDING 21 (id: `CORE-021`)

- **Source:** `docs/audit_reports/findings/wave4-core.md`
- **Severity:** Medium
- **Area:** infra
- **Location:** `crates/infra/core/src/value_objects.rs:144-159` (`ActiveStatus` derives)

**Description:**

`ActiveStatus` derives `Default` with `#[default]` on `Active` (line 146), but `#[derive(Default)]` on an enum is stable only with explicit discriminant choice. The engine also declares `Active = 1` and `Retired = 0` as explicit discriminants (lines 145, 150). `serde::Deserialize` will accept arbitrary integer discriminants by default — there is no `#[serde(try_from = "u8", into = "u8")]` shim, so a JSON `{"ActiveStatus": 7}` deserializes successfully into the default `Active` variant (via serde's fallback for unknown variants) rather than returning a `DomainError::Validation`. The dedicated `from_byte` validator at line 174-182 enforces the 0/1 contract for in-process construction but is bypassed by deserialization.

**Expected:**

`docs/schemas/database-schema.md` § 6 (cited at line 137): "active_status = 1" / "active_status = 0" with no other values. `AGENTS.md` Type Safety: serialization round-trips must preserve invariants.

**Evidence:**

```rust
  crates/infra/core/src/value_objects.rs:142-159
  #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
  pub enum ActiveStatus {
      /// The row is in use and visible to ordinary queries.
      #[default]
      Active = 1,
      /// The row has been soft-deleted and is hidden from ordinary
      /// queries. It is still queryable via `IncludeRetired::Yes`.
      Retired = 0,
  }
  ```
  No `#[serde(try_from = "u8", into = "u8")]` or custom `Deserialize` impl.

---

### FINDING 22 (id: `CORE-022`)

- **Source:** `docs/audit_reports/findings/wave4-core.md`
- **Severity:** Medium
- **Area:** infra
- **Location:** `crates/infra/core/src/clock.rs:258-272` (`test_clock_set` test)

**Description:**

The `test_clock_set` test (line 258-265) calls `c.set(Timestamp::from_datetime(chrono::Utc::now()))` — invoking the system clock inside a "deterministic" test fixture. The test then sleeps 2ms and asserts `a == b` (the clock did not advance), which works only because `set` overwrites the value rather than advancing. If a future refactor changes `set` to "advance to wall-clock now" (a plausible semantic), the test silently passes because the sleep is consumed; the test no longer proves determinism. The test contradicts the documented purpose of `TestClock`.

**Expected:**

`AGENTS.md` § Agent Instructions → Testing: "No dummy tests. Every test must validate a real-world scenario." `docs/guides/test-strategy.md` (cited in `clock.rs:1-10`): test fixtures should be fully deterministic.

**Evidence:**

```rust
  crates/infra/core/src/clock.rs:258-265
  #[test]
  fn test_clock_set() {
      let c = TestClock::new();
      c.set(Timestamp::from_datetime(chrono::Utc::now()));
      let a = c.now();
      std::thread::sleep(std::time::Duration::from_millis(2));
      let b = c.now();
      assert_eq!(a, b);
  }
  ```

---

### FINDING 23 (id: `CORE-023`)

- **Source:** `docs/audit_reports/findings/wave4-core.md`
- **Severity:** Medium
- **Area:** infra
- **Location:** `crates/infra/core/src/lint.rs:240-262` (`match_block_close`)

**Description:**

`match_block_close` counts `{` and `}` braces line-by-line without skipping braces inside string literals, character literals, line comments, or block comments. The doc comment acknowledges this ("a simple depth counter is enough for the lint's needs ... a false-positive here only widens the exempt window by one or two lines, which is harmless") but a `{` inside a doc comment (e.g. `/// ```rust\n/// let s = "{";\n/// ````) shifts the depth counter, causing the scanner to mark the wrong end of the `mod tests` block and exempt a larger (or smaller) window than intended. Lines containing the forbidden anti-patterns in that mis-counted window are silently passed.

**Expected:**

`AGENTS.md` "Type Safety": "Enforce full type safety at all times." Lint scanners must be correct; a permissive count is a false-negative source.

**Evidence:**

```rust
  crates/infra/core/src/lint.rs:240-262
  fn match_block_close(lines: &[&str], open_line: usize) -> usize {
      let mut depth: u32 = 0;
      let mut started = false;
      for (idx, line) in lines.iter().enumerate().skip(open_line) {
          for ch in line.chars() {
              match ch {
                  '{' => {
                      depth += 1;
                      started = true;
                  }
                  '}' => {
                      depth = depth.saturating_sub(1);
                      if started && depth == 0 {
                          return idx;
                      }
                  }
                  _ => {}
              }
          }
      }
      lines.len().saturating_sub(1)
  }
  ```
  No string-literal or comment skipping.

---

### FINDING 24 (id: `CORE-024`)

- **Source:** `docs/audit_reports/findings/wave4-core.md`
- **Severity:** Medium
- **Area:** infra
- **Location:** `crates/infra/core/src/lint.rs:181-238` (`scan_file_for_anti_patterns` needle array)

**Description:**

The needle array at line 220-225 lists `.unwrap()`, `.unwrap_err()`, `panic!(`, `todo!()`, `unimplemented!()` but does NOT include `.expect(`, `.unwrap_or(` (with no panic but signal of swallowing errors), `unreachable!(`, or `dbg!(`. AGENTS.md and `docs/code-standards.md` forbid `expect()` in production paths alongside `unwrap()`. The lint's doc comment at line 178-181 explicitly claims it flags "unwrap/expect/panic/todo!/unimplemented!" — the doc and the implementation disagree.

**Expected:**

`docs/build-plan.md:1917-1923`: "No `unimplemented!()`, `todo!()`, or `// TODO: implement` in production code." `AGENTS.md` "Type Safety: No `unwrap()` or `expect()` in production paths."

**Evidence:**

```rust
  crates/infra/core/src/lint.rs:181-184
  /// Flags `unwrap`/`expect`/`panic!`/`todo!`/`unimplemented!`
  /// calls in production Rust source. Test code (detected by
  /// `#[cfg(test)]` blocks or by the file living under
  /// `tests/`) is exempt.
  ```
  And line 220-225 (needle array shown in CORE-003): `.expect(` is missing.

---

### FINDING 25 (id: `CORE-025`)

- **Source:** `docs/audit_reports/findings/wave4-core.md`
- **Severity:** Medium
- **Area:** infra
- **Location:** `crates/infra/core/src/lint.rs:42-58` (`Violation::check` and `LintReport::print_to_stderr`)

**Description:**

`Violation::check` is a free-form `String`. `scan_file_for_anti_patterns` populates it with `format!("anti_pattern:{needle}")` (line 230), producing strings like `anti_pattern:.unwrap()` or `anti_pattern:.unwrap_err()`. CI scripts that filter on `check == "anti_pattern:unwrap"` (the conventional dotted identifier) will miss all current violations because the actual string embeds the literal token. The check id is also unstable across changes to the needle list (a future addition of `.expect(` would silently change downstream filters' behaviour).

**Expected:**

`AGENTS.md` Validation Checklist: deterministic CI gating requires stable identifiers.

**Evidence:**

```rust
  crates/infra/core/src/lint.rs:42-58
  #[derive(Debug, Clone)]
  pub struct Violation {
      /// Short identifier of the check that fired (e.g. `"unwrap_in_prod"`,
      /// `"missing_tests_path"`).
      pub check: String,
      ...
  }
  ...
  pub fn print_to_stderr(&self) -> usize {
      for v in &self.violations {
          ...
          eprintln!("{}\t{}:{}\t{}", v.check, v.file.display(), line, v.message);
      }
      ...
  }
  ```
  The example `"unwrap_in_prod"` in the doc comment at line 45 is not the format actually produced.

---

### FINDING 26 (id: `CORE-026`)

- **Source:** `docs/audit_reports/findings/wave4-core.md`
- **Severity:** Low
- **Area:** infra
- **Location:** `crates/infra/core/src/ids.rs:285-301` (`PUBLIC_SCHOOL_ID` constant vs. `SchoolId::PUBLIC` accessor)

**Description:**

`AGENTS.md` § Crate Inventory (Phase 12 entry) and the doc comment at `ids.rs:291` both refer to `SchoolId::PUBLIC` (an associated constant or method on `SchoolId`), but the actual constant is named `PUBLIC_SCHOOL_ID` (free const, line 293) and `SchoolId::is_public(self) -> bool` is an instance method (line 301). The naming convention is inconsistent with `SYSTEM_USER_ID` / `PLATFORM_SCHOOL_ID` (also free consts) and with the doc's prose. Consumers following AGENTS.md will not find `SchoolId::PUBLIC` and may compile-fail.

**Expected:**

`AGENTS.md` § Crate Inventory (Phase 12): "SchoolId::PUBLIC constant added to educore-core."

**Evidence:**

```rust
  crates/infra/core/src/ids.rs:285-301
  /// set `school_id = SchoolId::PUBLIC` on the aggregate and the
  /// public-site port adapter reads across the school boundary.
  pub const PUBLIC_SCHOOL_ID: SchoolId = SchoolId(Uuid::nil());
  
  impl SchoolId {
      /// Returns `true` if this is the well-known public-content
      /// school id (the nil UUID). ...
      #[must_use]
      pub const fn is_public(self) -> bool {
          self.0.is_nil()
      }
  }
  ```
  No `impl SchoolId { pub const PUBLIC: Self = ... }`. `AGENTS.md` § Crate Inventory (Phase 12 row): "SchoolId::PUBLIC constant added to educore-core."

---


## Query-Derive (infra) (target id prefix: `INFRA-QD`)

**Path:** `crates/infra/query-derive/`  
**Total findings:** 28 (5 critical, 6 high, 10 medium, 7 low)


### FINDING 1 (id: `INFRA-QD-001`)

- **Source:** `docs/audit_reports/findings/wave4-query-derive.md`
- **Severity:** Critical
- **Area:** infra
- **Location:** `crates/infra/query-derive/src/lib.rs:634-644`

**Description:**

The generic `where_has` method emitted by the macro is a no-op stub. It accepts `relation: __R` and `__build: __F` but discards both (`let _ = relation; let _ = __build;`) and returns `self` unchanged. It does NOT add a `HasRelation` node to the AST. The spec mandates `where_has(StudentRelation::Parent, |p| { p.where_eq(...) })` adds a relational filter to the query.

**Expected:**

Per `docs/query_layer.md:179-181` and `docs/query_layer.md:241-245` and `docs/query_layer.md:328-336`: `pub fn where_has<R, F>(self, relation: R, build: F) -> Self where R: Into<StudentRelation>, F: FnOnce(RelatedQueryBuilder<R>) -> RelatedQueryBuilder<R>` — the closure must be invoked with the related builder, the resulting `QueryNode` must be wrapped in `QueryNode::HasRelation` and pushed onto `self.filters`.

**Evidence:**

```rust
  crates/infra/query-derive/src/lib.rs:634-644
  let generic_where_has = quote! {
      #struct_vis fn where_has<__R, __F>(mut self, relation: __R, __build: __F) -> Self
      where
          __R: ::std::convert::Into<::educore_core::query::Relation>,
          __F: ::std::ops::FnOnce(::educore_core::query::RelationalField) -> ::educore_core::query::RelationalField,
      {
          let _ = relation;
          let _ = __build;
          self
      }
  };
  ```

---

### FINDING 2 (id: `INFRA-QD-002`)

- **Source:** `docs/audit_reports/findings/wave4-query-derive.md`
- **Severity:** Critical
- **Area:** infra
- **Location:** `crates/infra/query-derive/src/lib.rs` (entire `expand_inner` function, lines 156-844)

**Description:**

The macro does NOT emit an `EntityDescriptor { table, columns, indexes, foreign_keys, rls }` struct as `docs/build-plan.md:171-173` and `docs/schemas/sql-dialects/README.md:158-160` mandate. The macro only emits a `*Field` enum, an optional `*Relation` enum, and a `*QueryBuilder` struct — it never reads struct-level attributes (no `table = "..."`, no `aggregate = "..."`), never reads field types (no column-type information), and never carries nullable/defaults/indexes/FKs/RLS as a typed Rust data structure.

**Expected:**

Per `docs/build-plan.md:170-172`: "Reads the struct's fields, field types, `#[domain_query(...)]` attributes, and emits an `EntityDescriptor { table, columns, indexes, foreign_keys, rls }`." Per `docs/schemas/sql-dialects/README.md:158-160`: "The macro output is dialect-agnostic — it carries table name, column types, nullable, defaults, indexes, FKs, RLS policies as a typed Rust data structure. No SQL strings."

**Evidence:**

- `crates/infra/query-derive/src/lib.rs:135-136` `#[proc_macro_derive(DomainQuery, attributes(query))]\npub fn derive_domain_query(input: TokenStream) -> TokenStream {` — only one attribute namespace registered.
  - `crates/infra/query-derive/src/lib.rs:272` `let column = f.name.to_string();` — the only thing the macro reads from each field is its name (as a snake_case Rust identifier). It never reads `f.ty`, never reads struct-level attributes.
  - `grep -n "EntityDescriptor\|f\.field\.ty\|f\.ty" crates/infra/query-derive/src/lib.rs` returns no matches.

---

### FINDING 3 (id: `INFRA-QD-003`)

- **Source:** `docs/audit_reports/findings/wave4-query-derive.md`
- **Severity:** Critical
- **Area:** infra
- **Location:** `crates/infra/query-derive/src/lib.rs:64` and `docs/build-plan.md:170` vs. `docs/schemas/sql-dialects/README.md:150`

**Description:**

The build plan and `docs/schemas/sql-dialects/README.md` mandate that the macro reads struct-level `#[domain_query(table = "...", aggregate = "...")]` attributes. The macro only accepts the field-level `#[query(...)]` attribute namespace (registered at line 135). A user writing `#[domain_query(table = "academic_students")]` per the sql-dialects README example will get an "unknown attribute" error.

**Expected:**

Per `docs/build-plan.md:170`: the macro reads "struct's fields, field types, `#[domain_query(...)]` attributes". Per `docs/schemas/sql-dialects/README.md:150`: `#[domain_query(table = "academic_students", aggregate = "Student")]` is the documented struct-level invocation.

**Evidence:**

- `crates/infra/query-derive/src/lib.rs:135` `#[proc_macro_derive(DomainQuery, attributes(query))]` — only `query` namespace registered, not `domain_query`.
  - `crates/infra/query-derive/src/lib.rs:64` `if attr.path().is_ident("query") {` — only `#[query(...)]` is recognized; `#[domain_query(...)]` is silently ignored (treated as an unrelated attribute).
  - `docs/build-plan.md:170` `Reads the struct's fields, field types, \`#[domain_query(...)]\` attributes`.
  - `docs/schemas/sql-dialects/README.md:150` `#[domain_query(table = "academic_students", aggregate = "Student")]`.

---

### FINDING 4 (id: `INFRA-QD-004`)

- **Source:** `docs/audit_reports/findings/wave4-query-derive.md`
- **Severity:** Critical
- **Area:** infra
- **Location:** `crates/infra/query-derive/src/lib.rs:156-844` (whole macro emission)

**Description:**

The macro emits no `__spec_coverage__` test module. The build plan § Phase 0 task 2 (line 172-173) requires the macro to "Emit a `__spec_coverage__` test module on every `#[derive(DomainQuery)]` (see § The No-Gaps Gates)." No such module is emitted by the macro; downstream crates that `#[derive(DomainQuery)]` therefore do not get per-aggregate coverage tests automatically.

**Expected:**

`docs/build-plan.md:172-173`: "Emits a `__spec_coverage__` test module on every `#[derive(DomainQuery)]` (see § The No-Gaps Gates)."

**Evidence:**

- `grep -n "__spec_coverage__" crates/infra/query-derive/src/lib.rs` returns no matches.
  - `grep -rn "__spec_coverage__" crates/ --include="*.rs"` returns no matches anywhere in the workspace.
  - The trailing comment at `crates/infra/query-derive/src/lib.rs:849-851` `// Test module — verifies the macro emits correct code` describes a module that does not exist in the file (tests are in `tests/derive_test.rs`, but no `mod __spec_coverage__` is emitted to consumers).

---

### FINDING 5 (id: `INFRA-QD-005`)

- **Source:** `docs/audit_reports/findings/wave4-query-derive.md`
- **Severity:** Critical
- **Area:** infra
- **Location:** `crates/infra/query-derive/src/lib.rs` (no `*Aggregate` enum emitted anywhere)

**Description:**

The macro does not emit a `*Aggregate` enum. `docs/query_layer.md:482-498` mandates that the macro emit a `StudentAggregate` enum (with variants `Count`, `Sum`, `Avg`, `Min`, `Max`) alongside the field enum, and that users call `.aggregate(StudentAggregate::Count).group_by(StudentField::ClassId).execute()`. The macro has no aggregation support and no aggregate enum type.

**Expected:**

Per `docs/query_layer.md:496-498`: "Aggregations are `Count`, `Sum`, `Avg`, `Min`, `Max` over numeric fields. The macro emits the `StudentAggregate` enum alongside the field enum, ensuring the aggregation set is closed at compile time."

**Evidence:**

- `grep -n "Aggregate\|aggregate" crates/infra/query-derive/src/lib.rs` returns no matches.
  - `docs/query_layer.md:482-498` mandates the `StudentAggregate` enum and `.aggregate(StudentAggregate::Count)` API.

---

### FINDING 10 (id: `INFRA-QD-010`)

- **Source:** `docs/audit_reports/findings/wave4-query-derive.md`
- **Severity:** High
- **Area:** infra
- **Location:** `crates/infra/query-derive/src/lib.rs:255-264`

**Description:**

The duplicate-field check compares Rust field-name identifiers (`f.name`), not the PascalCase enum variants the macro emits. Two fields whose names map to the same PascalCase form (e.g., `foo_bar` and `FooBar`, or `naïve_field` and `naive_field` after Unicode normalization issues, or simply `foo__bar` and `foo_bar` after the underscore collapsing logic in `pascal_case`) will silently compile at macro-expansion time and then produce a confusing downstream Rust compile error about duplicate enum variants. The check at lines 255-264 is insufficient.

**Expected:**

A duplicate-PascalCase-variant check inside the macro with a friendly compile error.

**Evidence:**

- `crates/infra/query-derive/src/lib.rs:255-264` `let mut seen_queryable: Vec<&Ident> = Vec::new();\nfor f in &queryable { if seen_queryable.iter().any(|i| **i == f.name) { return Err(...) }\n    seen_queryable.push(&f.name);\n}` — checks `f.name` (the Rust ident), not the PascalCase form.
  - `crates/infra/query-derive/src/lib.rs:266-269` — the macro emits `format_ident!("{}", pascal_case(&f.name.to_string()))` for the variant. If two `f.name` values map to the same `pascal_case(...)` result, the emitted enum has duplicate variants.

---

### FINDING 20 (id: `INFRA-QD-020`)

- **Source:** `docs/audit_reports/findings/wave4-query-derive.md`
- **Severity:** High
- **Area:** infra
- **Location:** `crates/infra/query-derive/src/lib.rs` (zero `#[derive(DomainQuery)]` outside of its own tests)

**Description:**

The macro has zero adoption. `grep -rn "#\[derive(DomainQuery)\]" crates/` finds the attribute only in `crates/infra/query-derive/tests/derive_test.rs` (lines 26, 38, 45, 57). No domain crate, no cross-cutting crate, and no adapter uses it. PHASE-0-HANDOFF.md:38-39 acknowledges this. The macro is shipped but unproven: it has no real-world domain aggregate to validate against, no field-type interplay to handle (because the spec mandates column types but the macro doesn't read them), and no storage adapter consumes its output (the macro emits no `EntityDescriptor`, so no adapter can walk the AST to emit DDL).

**Expected:**

Per `docs/build-plan.md` and `AGENTS.md`, the macro is the foundation of the typed query layer; every `tables.md` row is supposed to be backed by a `#[derive(DomainQuery)]` struct. Per the wave-1 audit reports (e.g., `wave1-academic.md`, `wave1-assessment.md`, `wave1-cms.md`), all 10 domain crates ship without a single `#[derive(DomainQuery)]` derive. Per `docs/handoff/PHASE-0-HANDOFF.md:38-39`: "The `#[derive(DomainQuery)]` macro is real but not yet used by any domain crate. Its tests are the proof of life."

**Evidence:**

`grep -rn "#\[derive(DomainQuery)\]" crates/` returns 4 matches, all in `crates/infra/query-derive/tests/derive_test.rs`.

---

### FINDING 6 (id: `INFRA-QD-006`)

- **Source:** `docs/audit_reports/findings/wave4-query-derive.md`
- **Severity:** High
- **Area:** infra
- **Location:** `crates/infra/query-derive/src/lib.rs:803-807` and `crates/infra/query-derive/src/lib.rs:406-431`

**Description:**

The macro emits a public `pub fn new() -> Self { Self::default() }` and a `#[derive(Default)]` on the builder. Both paths produce a builder with `school_id = None`. The spec at `docs/query_layer.md:518-527` mandates "The default constructor is private. A query that omits the school id is a compile error. ... `StudentQueryBuilder` is constructed only via `StudentQuery::new(school_id)`." The engine rule "Compile-time safety over strings" (AGENTS.md) implies school-id enforcement should be at the type level, not the runtime. The macro's enforcement is a runtime `Err(DomainError::Validation(...))` in `build_query_node()` at lines 777-786.

**Expected:**

Per `docs/query_layer.md:520-527`: "The default constructor is private. A query that omits the school id is a compile error."

**Evidence:**

- `crates/infra/query-derive/src/lib.rs:803-807` `let new_method = quote! { #struct_vis fn new() -> Self { Self::default() } };`
  - `crates/infra/query-derive/src/lib.rs:406-431` (the `#[derive(Default)]` on `*QueryBuilder` makes `Default::default()` callable publicly).
  - `crates/infra/query-derive/src/lib.rs:777-786` (the missing-school-id check is at runtime in `build_query_node`, not at compile time).

---

### FINDING 7 (id: `INFRA-QD-007`)

- **Source:** `docs/audit_reports/findings/wave4-query-derive.md`
- **Severity:** High
- **Area:** infra
- **Location:** `crates/infra/query-derive/src/lib.rs:203-216` and `crates/infra/query-derive/src/lib.rs:266-296`

**Description:**

Both `filterable` and `sortable` decorations merge into the same `*Field` enum without distinction. The macro emits `fn where_eq<V>(mut self, field: StudentField, value: V) -> Self` (line 437) and `fn order_by(mut self, field: StudentField) -> Self` (line 558) with the same field type. A field marked only `#[query(sortable)]` can be passed to `where_eq`, and a field marked only `#[query(filterable)]` can be passed to `order_by`. The spec at `docs/query_layer.md:96-103` says: "`#[query(filterable)]` — Field can be used in a `.where_*` clause; `#[query(sortable)]` — Field can be used in `.order_by(...)`". The compile-time distinction is not enforced.

**Expected:**

Per `docs/query_layer.md:96-103` and the AGENTS.md "Compile-time safety over strings" rule, `filterable` and `sortable` should produce disjoint enums (e.g., `StudentFilterField` vs `StudentSortField`) so the compiler rejects `where_eq(SortField::X, ...)` and `order_by(FilterField::Y)`.

**Evidence:**

- `crates/infra/query-derive/src/lib.rs:203-216` (the `queryable` filter combines `filterable || sortable` into one set).
  - `crates/infra/query-derive/src/lib.rs:437` `fn where_eq<V>(mut self, field: #field_enum_name, value: V) -> Self`.
  - `crates/infra/query-derive/src/lib.rs:558` `fn order_by(mut self, field: #field_enum_name) -> Self`.

---

### FINDING 8 (id: `INFRA-QD-008`)

- **Source:** `docs/audit_reports/findings/wave4-query-derive.md`
- **Severity:** High
- **Area:** infra
- **Location:** `crates/infra/query-derive/src/lib.rs:333`, `crates/infra/query-derive/src/lib.rs:341`, `crates/infra/query-derive/src/lib.rs:367`, `crates/infra/query-derive/src/lib.rs:604`

**Description:**

The macro calls `format_ident!("{relation}")` four times with the raw string from `#[query(relation = "...")]` without validating that the string is a valid Rust identifier. A `relation` value containing non-identifier characters (e.g., `"foo-bar"`, `"foo bar"`, `"123"`, `""`) causes `format_ident!` to panic or emit invalid Rust tokens that surface as a confusing downstream compile error. No defensive check exists.

**Expected:**

A clear compile error pointing at the offending field with a message like "relation name `foo-bar` is not a valid Rust identifier" before any token emission.

**Evidence:**

- `crates/infra/query-derive/src/lib.rs:333` `let variant = format_ident!("{relation}");` (in `relation_variants`).
  - `crates/infra/query-derive/src/lib.rs:341` `let variant = format_ident!("{relation}");` (in `relation_match_arms`).
  - `crates/infra/query-derive/src/lib.rs:367` `let variant = format_ident!("{relation}");` (in `all_relations_slice`).
  - `crates/infra/query-derive/src/lib.rs:604` `let relation_variant = format_ident!("{relation}");` (in `where_has_methods`).

---

### FINDING 9 (id: `INFRA-QD-009`)

- **Source:** `docs/audit_reports/findings/wave4-query-derive.md`
- **Severity:** High
- **Area:** infra
- **Location:** `crates/infra/query-derive/src/lib.rs:330-336`

**Description:**

Two fields with `#[query(relation = "Parent")]` produce duplicate `StudentRelation::Parent` variants in the macro-emitted enum, yielding a confusing Rust compile error "variant `Parent` is already defined". The macro's pre-checks at lines 67-78 only flag duplicate `relation = "..."` on the SAME field, not duplicate `relation` values across two fields. There is no duplicate-relation-name detection.

**Expected:**

A compile error at macro expansion time like "duplicate `relation = \"Parent\"` declared on fields `parent_a` and `parent_b`" before token emission.

**Evidence:**

- `crates/infra/query-derive/src/lib.rs:67-78` — the `parse_field_attrs` function only checks for duplicates within one field's attributes.
  - `crates/infra/query-derive/src/lib.rs:330-336` — the macro iterates relations without deduplicating `relation` strings.
  - `crates/infra/query-derive/src/lib.rs:255-264` — the `seen_queryable` duplicate check only covers the queryable filterable/sortable set, not the relations set.

---

### FINDING 11 (id: `INFRA-QD-011`)

- **Source:** `docs/audit_reports/findings/wave4-query-derive.md`
- **Severity:** Medium
- **Area:** infra
- **Location:** `crates/infra/query-derive/src/lib.rs:605`

**Description:**

The `builder = "..."` value is parsed with `let builder_ty: Ident = syn::parse_str(builder)?;`. `syn::parse_str::<Ident>` only accepts a single bare identifier; a value like `"crate::module::ParentQueryBuilder"` (a path) fails with a confusing error "expected ident". The macro accepts the attribute but does not validate that the string is a syntactically valid Rust type path, and does not validate that the named type actually exists in the caller's scope.

**Expected:**

A friendly compile error like "builder `crate::module::X` is not a valid type path" or "builder type `X` is not in scope at this derive site".

**Evidence:**

- `crates/infra/query-derive/src/lib.rs:605` `let builder_ty: Ident = syn::parse_str(builder)?;` — `Ident` rejects paths.
  - `crates/infra/query-derive/src/lib.rs:73-78` — the duplicate-builder check inside `parse_field_attrs` only checks for duplicate attributes on one field, not path syntax.

---

### FINDING 12 (id: `INFRA-QD-012`)

- **Source:** `docs/audit_reports/findings/wave4-query-derive.md`
- **Severity:** Medium
- **Area:** infra
- **Location:** `crates/infra/query-derive/src/lib.rs:91-105`

**Description:**

`pascal_case` does not validate that the result is a valid Rust identifier. A field whose name is empty (`""`) yields an empty string; a name with leading digits (`"2nd_field"`) yields `"2ndField"` (invalid Rust — identifiers cannot start with a digit); a name with non-ASCII characters is passed through unchanged; a name with consecutive underscores (`"foo__bar"`) and `foo_bar` both yield the same PascalCase form (`FooBar`), causing the collision in Finding INFRA-QD-010. The function has no unit tests and no callers in the workspace that exercise corner cases.

**Expected:**

`pascal_case` should produce only valid Rust identifiers (reject empty input, reject names that start with a digit, normalize non-ASCII) and the macro should detect collisions between field names that map to the same PascalCase form.

**Evidence:**

`crates/infra/query-derive/src/lib.rs:91-105` `fn pascal_case(s: &str) -> String { let mut out = String::with_capacity(s.len()); let mut at_word_start = true; for ch in s.chars() { if ch == '_' { at_word_start = true; } else if at_word_start { out.extend(ch.to_uppercase()); at_word_start = false; } else { out.push(ch); } } out }` — no validation, no Rust-identifier check.

---

### FINDING 13 (id: `INFRA-QD-013`)

- **Source:** `docs/audit_reports/findings/wave4-query-derive.md`
- **Severity:** Medium
- **Area:** infra
- **Location:** `crates/infra/query-derive/src/lib.rs:729-748` (the `if queryable.is_empty()` else-branch)

**Description:**

When the user calls `.build_query_node()` with no filters, the macro emits `QueryNode::And(Box::new(QueryNode::IsNull(first_variant)), Box::new(QueryNode::IsNotNull(first_variant)))`. This is a never-satisfiable predicate: `IS NULL AND IS NOT NULL` for the same field. A user expecting "no filters means all rows" gets zero rows. There is no documentation of this behavior in the spec, and `docs/query_layer.md` does not describe this degenerate-tree shape.

**Expected:**

Either (a) an empty-filter build returns `QueryNode::True` (a documented "match-all" sentinel) or (b) `build_query_node()` errors out with a clear `DomainError::Validation` explaining that no filter was added. The current behavior silently returns zero rows.

**Evidence:**

`crates/infra/query-derive/src/lib.rs:729-748` `::std::option::Option::None => { let first_variant = #field_enum_name::all_variants()[0]; ::educore_core::query::QueryNode::And(::std::boxed::Box::new(::educore_core::query::QueryNode::IsNull(first_variant)), ::std::boxed::Box::new(::educore_core::query::QueryNode::IsNotNull(first_variant)),) }`.

---

### FINDING 14 (id: `INFRA-QD-014`)

- **Source:** `docs/audit_reports/findings/wave4-query-derive.md`
- **Severity:** Medium
- **Area:** infra
- **Location:** `crates/infra/query-derive/src/lib.rs:671-713` (the `queryable.is_empty()` branch)

**Description:**

When the struct has only relations and no queryable fields (e.g., `Bookmark` in `tests/derive_test.rs:57-62`), `__educore_compile` emits `And(HasRelation(sentinel, IsNull), HasRelation(sentinel, IsNotNull))` with `Relation { id: 0, name: "" }` as a sentinel. The comment at lines 673-686 admits this is a workaround. There is no compile-time guard preventing the user from forgetting to call `where_has_<Relation>(...)` before `build_query_node()`. The only test for this path (`no_relations_struct_compiles` at `tests/derive_test.rs:204-213`) verifies only that the macro compiles, not the AST shape.

**Expected:**

A compile-time mechanism (a builder state type or a phantom `relations_present: bool` token) that prevents `build_query_node()` from being called without at least one `where_has_*` filter on a no-fields struct.

**Evidence:**

`crates/infra/query-derive/src/lib.rs:687-712` emits the sentinel `HasRelation(_, IsNull) AND HasRelation(_, IsNotNull)` with `Relation { id: 0, name: "" }`. `crates/infra/query-derive/tests/derive_test.rs:204-213` does not assert the AST shape.

---

### FINDING 15 (id: `INFRA-QD-015`)

- **Source:** `docs/audit_reports/findings/wave4-query-derive.md`
- **Severity:** Medium
- **Area:** infra
- **Location:** `crates/infra/query-derive/src/lib.rs:791`

**Description:**

`let limit = self.limit.unwrap_or(50);` hardcodes a default page limit of 50. No spec mandates this value, and no constant is exposed for callers to override or document. The magic number is buried inside the macro emission.

**Expected:**

Either a documented constant `DEFAULT_PAGE_LIMIT: u32 = 50` exposed from `educore-core`, or the spec to mandate the value.

**Evidence:**

`crates/infra/query-derive/src/lib.rs:791` `let limit = self.limit.unwrap_or(50);`. `grep -rn "limit.*50\|default.*50" docs/query_layer.md docs/build-plan.md` returns no matching spec mandates.

---

### FINDING 16 (id: `INFRA-QD-016`)

- **Source:** `docs/audit_reports/findings/wave4-query-derive.md`
- **Severity:** Medium
- **Area:** infra
- **Location:** `crates/infra/query-derive/src/lib.rs:276-280`

**Description:**

`_field_to_variant_arms` is computed via `queryable.iter().map(...).collect()` and then immediately bound to `_` (line 280) and never used. This is dead code; the `let _` prefix suppresses the unused-variable warning but leaves the computation in the source. The map closure at lines 277-279 references `Self::#ident => #field_enum_name::#variant` — an inverse-direction mapping that the macro never emits anywhere.

**Expected:**

Remove the dead code or wire it into an emitted method.

**Evidence:**

`crates/infra/query-derive/src/lib.rs:276-280` `let _field_to_variant_arms = queryable.iter().map(|f| { let variant = format_ident!("{}", pascal_case(&f.name.to_string())); let ident = &f.name; quote! { Self::#ident => #field_enum_name::#variant } });` — `let _` prefix and no downstream use.

---

### FINDING 17 (id: `INFRA-QD-017`)

- **Source:** `docs/audit_reports/findings/wave4-query-derive.md`
- **Severity:** Medium
- **Area:** infra
- **Location:** `crates/infra/query-derive/src/lib.rs:849-851` and the file's tail

**Description:**

The file ends with a comment `// Test module — verifies the macro emits correct code` followed by a closing divider, but no test module follows. The comment is misleading: tests live in `tests/derive_test.rs`, not inside the source file. There is no `#[cfg(test)] mod tests { ... }` block.

**Expected:**

Either remove the comment or move the test scaffolding inline.

**Evidence:**

`crates/infra/query-derive/src/lib.rs:849-851` `// ============================================================================\n// Test module — verifies the macro emits correct code\n// ============================================================================` — no test code follows; the file ends at line 851.

---

### FINDING 18 (id: `INFRA-QD-018`)

- **Source:** `docs/audit_reports/findings/wave4-query-derive.md`
- **Severity:** Medium
- **Area:** infra
- **Location:** `crates/infra/query-derive/src/lib.rs:157-158` and the macro emission on generic structs

**Description:**

The macro does not handle generic structs. `let struct_name = input.ident.clone();` (line 157) ignores generics. A user writing `#[derive(DomainQuery)] pub struct Foo<T> { pub id: Uuid, #[query(filterable)] pub value: T }` would get an emission with unresolved type parameter `T` in the generated builder, the builder methods (`where_eq`, etc.), and the `*Field` enum. The downstream compile error would surface as `T` not in scope.

**Expected:**

Either (a) the macro rejects generic structs with a clear "DomainQuery does not support generic structs" compile error, or (b) the macro propagates generics through to the emitted types (significantly more work).

**Evidence:**

`crates/infra/query-derive/src/lib.rs:157-158` `let struct_name = input.ident.clone(); let struct_vis = input.vis.clone();` — no `input.generics` handling.

---

### FINDING 19 (id: `INFRA-QD-019`)

- **Source:** `docs/audit_reports/findings/wave4-query-derive.md`
- **Severity:** Medium
- **Area:** infra
- **Location:** `crates/infra/query-derive/src/lib.rs:73-78` and `crates/infra/query-derive/src/lib.rs:243-253`

**Description:**

`#[query(builder = "X")]` alone (with no `relation = "..."`) is silently accepted and then ignored. `parse_field_attrs` merges a `builder` attribute on its own without producing a compile error or warning. The `relations` filter at lines 223-230 requires both `relation` and `builder` to be `Some`, so a field with only `builder` never appears in the relations set. The `builder` value is silently wasted.

**Expected:**

A compile error like "field has `builder = \"X\"` without `relation = \"...\"`" or "builder attribute requires relation attribute".

**Evidence:**

- `crates/infra/query-derive/src/lib.rs:73-78` only checks for duplicate `builder` values on the same field.
  - `crates/infra/query-derive/src/lib.rs:223-230` filters with `relation.as_deref()?` first, so `builder` is silently skipped if `relation` is `None`.
  - `crates/infra/query-derive/src/lib.rs:243-253` checks for `relation` without `builder` but not the inverse.

---

### FINDING 21 (id: `INFRA-QD-021`)

- **Source:** `docs/audit_reports/findings/wave4-query-derive.md`
- **Severity:** Medium
- **Area:** infra
- **Location:** `crates/infra/query-derive/tests/derive_test.rs` (entire file, 233 lines, 19 tests)

**Description:**

Test coverage is incomplete. Missing test cases:
  - No test for tuple struct (should error: "DomainQuery can only be derived for structs with named fields").
  - No test for unit struct (should error: same message).
  - No test for enum (should error: "DomainQuery can only be derived for structs").
  - No test for empty struct (no fields) (should error: "DomainQuery cannot be derived for a struct with no fields").
  - No test for struct with all fields `#[query(ignore)]` (should error).
  - No test for unknown attribute name (e.g., `#[query(filterablee)]` typo — should error per `parse_field_attrs` line 44-49).
  - No test for empty `#[query()]` attribute list (should error: "expected ident").
  - No test for duplicate `relation = "X"` on two fields (Finding INFRA-QD-009).
  - No test for `pascal_case` collision (Finding INFRA-QD-010).
  - No test for `relation = "foo-bar"` (invalid identifier, Finding INFRA-QD-008).
  - No test for `builder = "crate::module::T"` (path, Finding INFRA-QD-011).
  - No test for `builder = "X"` without `relation` (Finding INFRA-QD-019).
  - No test for generic struct (Finding INFRA-QD-018).
  - No snapshot test of macro expansion (per `docs/guides/test-strategy.md:225-238` "Macro Snapshot Tests").
  - No test that asserts the AST shape produced by `__educore_compile()` for filter combinations.
  - No test that asserts `where_has_<Relation>` produces a `QueryNode::HasRelation`.
  - No test that asserts the generic `where_has` (Finding INFRA-QD-001) silently no-ops.
  - No test of the `for_school` requirement at compile time (Finding INFRA-QD-006).

**Expected:**

Each documented and undocumented macro-input shape has at least one integration test verifying the compile error or the emitted AST shape. `docs/guides/test-strategy.md:225-238` mandates snapshot tests.

**Evidence:**

`crates/infra/query-derive/tests/derive_test.rs:64-232` contains 19 tests, none of which exercise the listed edge cases. `grep -c "^#\[test\]" crates/infra/query-derive/tests/derive_test.rs` returns 19.

---

### FINDING 22 (id: `INFRA-QD-022`)

- **Source:** `docs/audit_reports/findings/wave4-query-derive.md`
- **Severity:** Low
- **Area:** infra
- **Location:** `crates/infra/query-derive/src/lib.rs:271-274`

**Description:**

The `Field` impl emitted by the macro returns column names that are the snake_case Rust field names (e.g., `last_name`). There is no mechanism for a user to override the DB column name (no `#[query(column = "lastName")]` attribute or similar). A user whose DB schema has a different naming convention (CamelCase, custom abbreviation, prefix) cannot express it without renaming the Rust field, which fights the engine's "compile-time safety over strings" rule (the field enum and the DB column must stay in lock-step via a single source of truth).

**Expected:**

A `#[query(column = "actual_db_column_name")]` attribute per field that overrides the default Rust-field-name column name.

**Evidence:**

`crates/infra/query-derive/src/lib.rs:272` `let column = f.name.to_string();` — column name is always the Rust field name; no override path.

---

### FINDING 23 (id: `INFRA-QD-023`)

- **Source:** `docs/audit_reports/findings/wave4-query-derive.md`
- **Severity:** Low
- **Area:** infra
- **Location:** `crates/infra/query-derive/src/lib.rs:714-750` (`compile_method`'s happy-path branch)

**Description:**

When the user adds at least one filter, `__educore_compile` folds them left-to-right with `iter.fold(first, |acc, next| QueryNode::And(Box::new(acc), Box::new(next)))`. This produces a left-leaning tree of nested `And` nodes, which has linear depth for N filters. The spec at `docs/query_layer.md:564-566` calls for "sized for the common case (zero to four predicates)" — a tree of depth N is acceptable for small N but is non-ideal for queries with many predicates. No depth-balancing or pairwise fold is emitted.

**Expected:**

A balanced tree shape (or a `QueryNode::AndN(Vec<QueryNode<F>>)` variant) so depth is `O(log N)`.

**Evidence:**

`crates/infra/query-derive/src/lib.rs:740-745` `iter.fold(first, |acc, next| { ::educore_core::query::QueryNode::And(::std::boxed::Box::new(acc), ::std::boxed::Box::new(next)) })` — left fold, linear depth.

---

### FINDING 24 (id: `INFRA-QD-024`)

- **Source:** `docs/audit_reports/findings/wave4-query-derive.md`
- **Severity:** Low
- **Area:** infra
- **Location:** `crates/infra/query-derive/src/lib.rs:152-155`

**Description:**

The `expand_inner` inner fn has `#[allow(clippy::too_many_lines, clippy::similar_names, clippy::needless_pass_by_value, clippy::needless_borrow)]` to suppress four lint advisories. Per `docs/code-standards.md` and AGENTS.md, lint suppressions are discouraged unless justified. The `too_many_lines` and `similar_names` are symptoms of the macro's monolithic single-function structure; refactoring into helpers (parse-fields, build-enum, build-builder, build-impls) would remove the need for the suppression.

**Expected:**

Decompose the macro emission into focused helper functions so each function is short enough that no `clippy::too_many_lines` suppression is needed.

**Evidence:**

`crates/infra/query-derive/src/lib.rs:150-155` `#[allow(\n    clippy::too_many_lines,\n    clippy::similar_names,\n    clippy::needless_pass_by_value,\n    clippy::needless_borrow\n)]\nfn expand_inner(input: DeriveInput) -> syn::Result<TokenStream2> {`.

---

### FINDING 25 (id: `INFRA-QD-025`)

- **Source:** `docs/audit_reports/findings/wave4-query-derive.md`
- **Severity:** Low
- **Area:** infra
- **Location:** `crates/infra/query-derive/src/lib.rs:64` vs. `crates/infra/query-derive/src/lib.rs:135`

**Description:**

The proc-macro registers `attributes(query)` (line 135) but `parse_field_attrs` filters on `attr.path().is_ident("query")` (line 64). The single-attribute register means helper attributes (e.g., `#[query(column = "...")]` if added per Finding INFRA-QD-022) must all live in the `query` namespace. The macro never registers or recognizes a struct-level attribute namespace (e.g., `attributes(domain_query)`), so the `#[domain_query(...)]` struct-level form per Finding INFRA-QD-003 cannot be added without a breaking change to the registration.

**Expected:**

Two namespace registration: `attributes(query)` for field-level and `attributes(domain_query)` for struct-level.

**Evidence:**

- `crates/infra/query-derive/src/lib.rs:135` `#[proc_macro_derive(DomainQuery, attributes(query))]`.
  - `crates/infra/query-derive/src/lib.rs:64` `if attr.path().is_ident("query") {`.

---

### FINDING 26 (id: `INFRA-QD-026`)

- **Source:** `docs/audit_reports/findings/wave4-query-derive.md`
- **Severity:** Low
- **Area:** infra
- **Location:** `crates/infra/query-derive/tests/derive_test.rs:184-191`

**Description:**

`where_has_typed_method_compiles` verifies only that `where_has_Parent(|p| p.where_eq(...))` compiles. It does not assert the resulting AST is `QueryNode::HasRelation(StudentRelation::Parent, ...)`. The closure is invoked; the result is bound to `_b` and discarded. A regression that silently turns `where_has_*` into a no-op (analogous to Finding INFRA-QD-001 for the generic form) would not be caught.

**Expected:**

A test that calls `build_query_node()` and asserts the inner AST contains `QueryNode::HasRelation(StudentRelation::Parent, ...)` with the expected child predicate.

**Evidence:**

`crates/infra/query-derive/tests/derive_test.rs:184-191` `#[test]\nfn where_has_typed_method_compiles() {\n    let g = SystemIdGen;\n    let school = g.next_school_id();\n    let _b = StudentQueryBuilder::new()\n        .for_school(school)\n        .where_has_Parent(|p| p.where_eq(ParentField::City, "Boston"));\n}` — the resulting builder is bound to `_b` and discarded without AST inspection.

---

### FINDING 27 (id: `INFRA-QD-027`)

- **Source:** `docs/audit_reports/findings/wave4-query-derive.md`
- **Severity:** Low
- **Area:** infra
- **Location:** `crates/infra/query-derive/src/lib.rs:109-134` (the rustdoc example)

**Description:**

The rustdoc example for `derive_domain_query` uses `StudentStatus` as the type of `status` (line 127). `StudentStatus` is not defined in the macro crate, not imported via the `prelude::*` glob on line 117, and not in the `educore_core` crate. The example uses `rust,ignore` to suppress compile failure of the example, but it suggests an API that doesn't actually work (a user copying the example into their code will get an unresolved-type error for `StudentStatus`).

**Expected:**

Either (a) define `StudentStatus` in the example prelude, or (b) use a primitive type (e.g., `String`) so the example is self-contained, or (c) drop the `rust,ignore` and make the example actually compile-tested by doc-tests.

**Evidence:**

`crates/infra/query-derive/src/lib.rs:115-131` `/// ```rust,ignore\n/// use educore_query_derive::DomainQuery;\n/// use educore_core::prelude::*;\n///\n/// #[derive(DomainQuery)]\n/// pub struct Student {\n///     pub id: Uuid,\n///\n///     #[query(sortable)]\n///     pub last_name: String,\n///\n///     #[query(filterable)]\n///     pub status: StudentStatus,\n///\n///     #[query(filterable, relation = \"Parent\", builder = \"ParentQueryBuilder\")]\n///     pub parent_id: Uuid,\n/// }\n/// ```\n///` — `StudentStatus` and `ParentQueryBuilder` are undefined.

---

### FINDING 28 (id: `INFRA-QD-028`)

- **Source:** `docs/audit_reports/findings/wave4-query-derive.md`
- **Severity:** Low
- **Area:** infra
- **Location:** `crates/infra/query-derive/src/lib.rs:155` (entry of `expand_inner`) and the entire macro emission

**Description:**

The macro does not emit any reference to the originating struct's documentation or module path. Downstream crates cannot programmatically map a `*Field` enum back to the source struct or source file (the macro hardcodes `stringify!(#builder_name)` in error messages, but never captures `Span::call_site` or `struct_name.span()` for richer diagnostics). The `Span` API in `syn` 2.x supports `.resolved_at()` for cross-crate span tracking.

**Expected:**

Span metadata on the emitted items so `rustc` can point at the original `#[derive(DomainQuery)]` invocation when downstream compilation fails.

**Evidence:**

`crates/infra/query-derive/src/lib.rs:781-784` `concat!(\n    stringify!(#builder_name),\n    \" requires for_school() before build_query_node()\"\n)` — the error message uses only the builder name, not the source span.

---


## Storage Port (infra) (target id prefix: `PORT-STORE`)

**Path:** `crates/infra/storage/`  
**Total findings:** 36 (13 critical, 15 high, 6 medium, 2 low)


### FINDING 1 (id: `PORT-STORE-001`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** Critical
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/port.rs:30-33`

**Description:**

The `StorageAdapter` trait exposes `migrate()` as the schema-emission entry point, but every consumer-facing doc (`AGENTS.md:544, 561`, `docs/build-plan.md:175-179`, `docs/architecture.md:322`, `docs/schemas/sql-dialects/README.md:193-198`, `migrations/engine/README.md:11`) refers to `storage.create_schema().await` as the runtime DDL entry. The method does not exist on the trait; all four adapters (`storage-postgres`, `storage-mysql`, `storage-sqlite`, `storage-surrealdb`) ship a `migrate()` that the docs then rename to `create_schema()` at the consumer boundary. The port name and the consumer name are different for the same operation.

**Expected:**

`docs/build-plan.md:175-179` lists the trait surface as `("create_schema", "apply_command", "query", "begin_tx", …)`. `docs/architecture.md:322` states "the schema is emitted at runtime via `storage.create_schema().await`". The trait is the canonical contract for that method.

**Evidence:**

```rust
  crates/infra/storage/src/port.rs:30-33
  /// Applies the engine's DDL to bring the schema up to the
  /// engine's current version. Idempotent: running on an
  /// already-migrated database returns a no-op report.
  async fn migrate(&self) -> Result<MigrationReport>;
  ```
  `grep -rn "fn create_schema" crates/infra/storage/src/` returns no results.

---

### FINDING 10 (id: `PORT-STORE-010`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** Critical
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/repository.rs:33-92`

**Description:**

The `Repository<A>` trait provides no `stream` method, but `docs/ports/storage.md:148-153` explicitly mandates "the adapter may expose a streaming method … `async fn stream(&self, q: StudentQuery) -> Result<BoxStream<'static, Result<Student>>>`". The build-plan § Phase 5 exit criteria require the bulk-attendance load test to handle 10k rows in <5s; without streaming, the entire result set is loaded into memory. The `list(school_id, offset, limit)` signature at line 73 caps `offset`/`limit` at `u32` (4 billion rows), but a school with 10k attendance marks per day for 365 days = 3.65M rows — loaded as a `Vec<A>` the trait demands materialisation.

**Expected:**

`docs/ports/storage.md:148-153` — streaming method on every per-aggregate repository.

**Evidence:**

```rust
  crates/infra/storage/src/repository.rs:73
  async fn list(&self, school_id: SchoolId, offset: u32, limit: u32) -> Result<Vec<A>>;
  ```
  No `stream` method on the trait. `grep -n "fn stream" crates/infra/storage/src/repository.rs` returns no results.

---

### FINDING 11 (id: `PORT-STORE-011`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** Critical
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/idempotency.rs:62-118` (`Idempotency` trait)

**Description:**

The `Idempotency` trait has no `expires_at` field on `IdempotencyRecord` and no `purge_expired` / `purge` method. The `purge_older_than(school_id, cutoff)` method at line 108 is the only maintenance entry point and is documented as "the consumer configures" — but there is no per-record TTL contract on the port. `docs/schemas/command-schema.md` § 6 mandates "the engine retains idempotency records for the duration the consumer configures (default 7 days)" — yet the port cannot represent `expires_at` in the record, cannot enforce TTL on `lookup` (i.e. "if expired, treat as not-found"), and provides no way for the engine to schedule retention sweeps. Adapters that don't override `purge_older_than` (its default returns `Ok(0)` at line 110-112) silently accumulate rows forever.

**Expected:**

`docs/schemas/command-schema.md` § 6: "The engine retains idempotency records for the duration the consumer configures (default 7 days)." The port must carry an `expires_at` field, an `is_expired()` predicate, and a non-default `purge_older_than` contract.

**Evidence:**

```rust
  crates/infra/storage/src/idempotency.rs:29-44
  pub struct IdempotencyRecord {
      pub school_id: SchoolId,
      pub command_type: &'static str,
      pub idempotency_key: IdempotencyKey,
      pub outcome: bytes::Bytes,
      pub outcome_version: u32,
      pub recorded_at: educore_core::value_objects::Timestamp,
      pub affected_aggregate_ids: Vec<Uuid>,
  }
  ```
  No `expires_at`. And `purge_older_than` default at lines 107-112:
  ```rust
  async fn purge_older_than(
      &self,
      _school_id: SchoolId,
      _cutoff: educore_core::value_objects::Timestamp,
  ) -> Result<u64> {
      Ok(0)
  }
  ```

---

### FINDING 12 (id: `PORT-STORE-012`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** Critical
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/idempotency.rs:86-99`

**Description:**

The `record(record: IdempotencyRecord)` doc-comment promises "Returns `Err(Conflict)` if a record with the same `(school_id, command_type, idempotency_key)` already exists with a different outcome" — but the trait declares no method for comparing outcomes (hash? bytes equality? semantic equivalence?) and no marker field on `IdempotencyRecord` that lets the adapter decide. Wave-3 finding ADAPTER-PG-031 documents the PG adapter collapsing both branches into `DO NOTHING`; the trait-level contract is too ambiguous to implement correctly.

**Expected:**

`docs/schemas/command-schema.md` § 6 and `ADR-014-Idempotency.md` — the port must declare the outcome-comparison contract (a `outcome_hash: Hash` field on `IdempotencyRecord` plus an `equal_outcome(&self, other: &IdempotencyRecord) -> bool` method) so every adapter can determine "different outcome" without ambiguity.

**Evidence:**

```rust
  crates/infra/storage/src/idempotency.rs:86-99
  /// Stores `record`. Returns `Err(Conflict)` if a record with
  /// the same `(school_id, command_type, idempotency_key)`
  /// already exists with a different outcome. Returns `Ok(())`
  /// if the record is a no-op write (same key, same outcome
  /// hash) — the engine uses this for at-least-once delivery
  /// of retries.
  async fn record(&self, record: IdempotencyRecord) -> Result<()>;
  ```
  No `outcome_hash`, no comparison method, no `equal_outcome` predicate.

---

### FINDING 13 (id: `PORT-STORE-013`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** Critical
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/change_stream.rs:46-83`

**Description:**

`ChangeFilter` carries only `school_id`, `since: Option<VersionCursor>`, and `aggregate_types: Vec<AggregateTypeFilter>` — but no `event_types` filter, no `actor_id` filter, no `correlation_id` filter, no `since_time: Option<Timestamp>` (only `since: VersionCursor`), no `until_time` / `until_cursor`. A consumer that wants "all `finance.invoice.generated` events for school X since cursor Y" cannot express that query. The `ChangeEvent` payload (lines 119-135) also lacks `event_type`, so consumers can't route on event-type without re-parsing the payload bytes.

**Expected:**

`docs/schemas/event-schema.md` § 10 (subscription model) — `subscribe("finance.invoice.*")` and `subscribe_aggregate(...)` and `subscribe_school(...)`. The change-stream port must accept an `event_types` filter and the `ChangeEvent` must carry `event_type` as a first-class field.

**Evidence:**

```rust
  crates/infra/storage/src/change_stream.rs:46-56
  pub struct ChangeFilter {
      pub school_id: SchoolId,
      pub since: Option<VersionCursor>,
      pub aggregate_types: Vec<AggregateTypeFilter>,
  }
  ```
  No `event_types`, no `actor_id`, no `correlation_id`, no time-range filter.
  ```rust
  crates/infra/storage/src/change_stream.rs:119-135
  pub struct ChangeEvent {
      pub event_id: EventId,
      pub school_id: SchoolId,
      pub aggregate_type: String,
      pub aggregate_id: uuid::Uuid,
      #[serde(with = "bytes_via_vec")]
      pub payload: bytes::Bytes,
      pub cursor: VersionCursor,
  }
  ```
  No `event_type` field.

---

### FINDING 2 (id: `PORT-STORE-002`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** Critical
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/transaction.rs:51-75` (entire trait body)

**Description:**

The `Transaction` trait carries no `school_id` / `TenantContext` field. The sub-port handles `outbox()`, `audit_log()`, `idempotency()`, and `event_log()` are bare `&dyn Trait` references with no tenant anchor on the trait surface. Per `docs/schemas/tenancy-schema.md` § 4 the storage adapter MUST "reject writes whose `school_id` does not match the caller's `TenantContext::school_id`" — but the trait has no way to receive or expose the active `TenantContext`, so the adapter must hold it in a thread-local or a sibling field. The `bulk_insert_student_attendances` doc-comment at `port.rs:64-66` and `transaction.rs:88-90` says "the row's `school_id` MUST equal the transaction's scoped school (enforced by the adapter)" — but the trait surface never tells the adapter which school that is.

**Expected:**

`docs/schemas/tenancy-schema.md:97-103`: "The storage adapter is responsible for enforcing tenant isolation. The engine always passes a `SchoolId` filter; the adapter MUST add a `school_id = $1` predicate to every read query." The Transaction trait must carry the `TenantContext` (or at least `SchoolId` + `ActorId`) for the sub-port impls to scope reads/writes against.

**Evidence:**

```rust
  crates/infra/storage/src/transaction.rs:52-75
  #[async_trait]
  pub trait Transaction: Send + Sync + std::fmt::Debug {
      async fn commit(self: Box<Self>) -> Result<()>;
      async fn rollback(self: Box<Self>) -> Result<()>;
      fn outbox(&self) -> &dyn Outbox;
      fn audit_log(&self) -> &dyn AuditLog;
      fn idempotency(&self) -> &dyn Idempotency;
      fn event_log(&self) -> &dyn EventLog;
      ...
  }
  ```
  No `tenant()`, `school_id()`, or `actor_id()` accessor.

---

### FINDING 3 (id: `PORT-STORE-003`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** Critical
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/outbox.rs:78-115`

**Description:**

`Outbox::append`, `Outbox::pending`, `Outbox::mark_published`, and `Outbox::pending_count` have no `school_id` parameter on the trait surface, yet the doc-comment at line 105 states "The outbox is partitioned by `school_id` so callers see only envelopes for their school." The trait has no `school_id()` accessor (unlike the `bulk_insert_student_attendances` row, which carries its own `school_id` for cross-validation). Every adapter must hold the school internally, but the trait does not declare this invariant — and `Outbox::pending_count(school_id: SchoolId)` at line 113 takes an explicit school_id that can be *any* school, bypassing the adapter's own scoping. Wave-3 finding ADAPTER-PG-013 confirms this is exploited in practice.

**Expected:**

`docs/schemas/tenancy-schema.md:97-103` — "The storage adapter is responsible for enforcing tenant isolation." The `Outbox` trait must expose a `school_id()` accessor (or a `&TenantContext` field) so the adapter cannot drift, and `pending_count(school_id: SchoolId)` must be removed in favour of a parameterless `pending_count()` that uses the impl's scoped school.

**Evidence:**

```rust
  crates/infra/storage/src/outbox.rs:108-115
  async fn pending(&self, limit: u32) -> Result<Vec<SerializedEnvelope>>;
  async fn mark_published(&self, ids: &[EventId]) -> Result<()>;
  async fn pending_count(&self, school_id: SchoolId) -> Result<u64> {
      ...
  }
  ```

---

### FINDING 4 (id: `PORT-STORE-004`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** Critical
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/audit.rs:96-114`

**Description:**

`AuditLogEntry` is missing seven fields that `docs/schemas/audit-schema.md` § 2 mandates as part of the audit record: `audit_id` (UUIDv7 PK), `actor_type` (`user`/`system`/`agent`/`api_key`), `command_id`, `ip: Option<IpAddr>`, `user_agent: Option<String>`, `session_id: Option<SessionId>`, `cross_tenant: bool`, `source: AuditSource`, and a separate `recorded_at` distinct from `occurred_at`. The canonical PG DDL at `migrations/engine/0000_engine_core.postgres.sql:96-119` declares all 22 columns; the port struct carries 12. Every storage adapter that builds an `AuditLogEntry` cannot populate the missing columns — the wave-3 PG finding ADAPTER-PG-020 documents `#[allow(dead_code)]` annotations on the row struct for the same fields.

**Expected:**

`docs/schemas/audit-schema.md` § 2 (audit record shape) — every field is part of the audit record contract; `migrations/engine/0000_engine_core.postgres.sql:96-119` lists the columns the engine must write.

**Evidence:**

```rust
  crates/infra/storage/src/audit.rs:62-93
  pub struct AuditLogEntry {
      pub school_id: SchoolId,
      pub actor_id: UserId,
      pub action: String,
      pub target_type: String,
      pub target_id: Uuid,
      pub before: Option<bytes::Bytes>,
      pub after: Option<bytes::Bytes>,
      pub event_id: Option<EventId>,
      pub correlation_id: CorrelationId,
      pub occurred_at: Timestamp,
      pub active_status: ActiveStatus,
      pub metadata: serde_json::Value,
  }
  ```
  No `audit_id`, no `actor_type`, no `command_id`, no `ip`, no `user_agent`, no `session_id`, no `cross_tenant`, no `source`, no separate `recorded_at`.

---

### FINDING 5 (id: `PORT-STORE-005`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** Critical
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/idempotency.rs:29-44`

**Description:**

`IdempotencyRecord.command_type` is typed `&'static str`. This forces every adapter that reads the column back to `Box::leak` the VARCHAR value to satisfy the `&'static str` lifetime — wave-3 finding ADAPTER-PG-011 documents the unbounded heap growth in the PG adapter, and wave-3 finding ADAPTER-SQ-006 documents the same leak in the SQLite adapter. The port shape is the cause: `&'static str` is impossible to construct from a heap-allocated column value without a leak. AGENTS.md § "Type Safety" forbids `Box::leak` in production paths.

**Expected:**

`AGENTS.md` § "Type Safety": "No `#[allow(dead_code)]` or `_var` prefixes to silence the compiler. Delete unused code, wire it in, or open a follow-up issue." The port struct must use `String` (or `Cow<'static, str>`) so adapters can return owned data without leaking.

**Evidence:**

```rust
  crates/infra/storage/src/idempotency.rs:29-44
  pub struct IdempotencyRecord {
      pub school_id: SchoolId,
      pub command_type: &'static str,
      pub idempotency_key: IdempotencyKey,
      pub outcome: bytes::Bytes,
      pub outcome_version: u32,
      pub recorded_at: educore_core::value_objects::Timestamp,
      pub affected_aggregate_ids: Vec<Uuid>,
  }
  ```
  And the analogous field on the lookup key struct at line 58:
  ```rust
  pub command_type: &'static str,
  ```

---

### FINDING 6 (id: `PORT-STORE-006`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** Critical
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/transaction.rs:25-37`

**Description:**

The `commit` doc-comment claims "Conflict on a unique-key violation, deadlock, or serialisation failure (the engine retries the command automatically)" — but the port defines no retry policy, no `is_retryable(&self, err)` method, no error classification scheme that lets the engine distinguish retryable conflicts from non-retryable ones. The wave-3 PG finding ADAPTER-PG-025 confirms the engine has no retry path: a bulk-insert unique-key violation returns `DomainError::conflict` and the caller is on its own. The port's stated contract is unenforceable.

**Expected:**

`docs/ports/storage.md:124-127` — "On commit the writes are persisted and the outbox events are released to the event bus." `docs/ports/storage.md:131-137` — the engine retries on conflict. The port must expose a `Conflict`-vs-`Permanent` error distinction (or a retry predicate) so the engine can drive the retry policy.

**Evidence:**

```rust
  crates/infra/storage/src/transaction.rs:26-37
  async fn commit(self: Box<Self>) -> Result<()>;
  /// Rolls the transaction back. ...
  async fn rollback(self: Box<Self>) -> Result<()>;
  ```
  Doc comment lines 32-36:
  ```text
  /// # Errors
  /// - `Conflict` on a unique-key violation, deadlock, or
  ///   serialisation failure (the engine retries the command
  ///   automatically).
  ```
  No retry predicate, no error classification.

---

### FINDING 7 (id: `PORT-STORE-007`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** Critical
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/audit.rs:14-50` (`AuditLogEntry`)

**Description:**

`AuditLogEntry::action` is typed `String` and `AuditLogEntry::target_type` is typed `String`, while `docs/schemas/audit-schema.md` § 2 specifies them as `AuditAction` and `ResourceType` enums. The engine rule at AGENTS.md § "Compile-time safety over strings" mandates macro-generated enums, not free-form strings. The audit-schema canonical DDL declares `action VARCHAR(191) NOT NULL` and `resource_type VARCHAR(64) NOT NULL` — the port's `String` defeats the type-level audit-routing guarantees the spec promises.

**Expected:**

AGENTS.md § "Engine Rules" rule 2: "Compile-time safety over strings. Use macro-generated enums (`StudentField::Status`) — never string field names."

**Evidence:**

```rust
  crates/infra/storage/src/audit.rs:67-69
  pub action: String,
  ...
  pub target_type: String,
  ```
  vs `migrations/engine/0000_engine_core.postgres.sql:101-102`:
  ```
  action          VARCHAR(191) NOT NULL,
  resource_type   VARCHAR(64)  NOT NULL,
  ```
  No `AuditAction` or `ResourceType` enum exists in the crate (`grep -rn "enum AuditAction\|enum ResourceType" crates/infra/storage/src/` returns no results).

---

### FINDING 8 (id: `PORT-STORE-008`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** Critical
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/audit.rs:106-114` (`AuditLog` trait)

**Description:**

The `AuditLog` trait exposes only `append` and `read_for_target` (a per-aggregate history). The audit-schema.md § 5 mandate requires a full `AuditQuery` port with `list`, `get`, `resource_history`, `actor_history`, and filter variants `ByAction`, `ByResource`, `ByActor`, `ByCorrelation`, `ByTimeRange`, `ByEventType`, `ByCustom`. The port provides none of these — there is no method to query "every record of action X in the last 30 days" or "every record for actor Y in window W". `read_for_target` returns at most `limit` rows, with no offset, no cursor, no actor filter, no time-range filter, no correlation filter.

**Expected:**

`docs/schemas/audit-schema.md` § 5 — `AuditQuery` trait with `list`, `get`, `resource_history`, `actor_history` and the `AuditFilter` enum.

**Evidence:**

```rust
  crates/infra/storage/src/audit.rs:106-114
  #[async_trait]
  pub trait AuditLog: Send + Sync {
      async fn append(&self, entry: AuditLogEntry) -> Result<()>;
      async fn read_for_target(
          &self,
          school_id: SchoolId,
          target_id: Uuid,
          limit: u32,
      ) -> Result<Vec<AuditLogEntry>>;
  }
  ```
  `read_for_target` accepts only `(school_id, target_id, limit)` — no actor, no action, no correlation, no time range, no offset, no cursor.

---

### FINDING 9 (id: `PORT-STORE-009`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** Critical
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/repository.rs:33-92`

**Description:**

The `Repository<A>` trait takes only `school_id: SchoolId` on every method; there is no `TenantContext` parameter, so the adapter has no `actor_id`, `correlation_id`, `session_id`, `user_agent`, or `ip` to stamp `created_by`/`updated_by`/`cross_tenant` columns with on `insert`/`update`/`soft_delete`. The audit-schema.md § 14 columns include `created_by` and `updated_by` as NOT NULL — the storage adapter cannot populate them without a separate out-of-band channel. The trait comment at line 31 admits "the engine never observes a half-built result" but provides no way for the engine to pass per-call actor identity.

**Expected:**

`docs/schemas/audit-schema.md` § 14 — `created_by`, `updated_by` columns are mandatory. `docs/schemas/database-schema.md` § 2 — `created_by` and `updated_by` are required on every aggregate table. The port must accept `&TenantContext` so the adapter can read `actor_id` and stamp the row.

**Evidence:**

```rust
  crates/infra/storage/src/repository.rs:36-92
  async fn get(&self, school_id: SchoolId, id: Uuid) -> Result<Option<A>>;
  async fn get_including_retired(&self, school_id: SchoolId, id: Uuid) -> Result<Option<A>>;
  async fn list(&self, school_id: SchoolId, offset: u32, limit: u32) -> Result<Vec<A>>;
  async fn count(&self, school_id: SchoolId) -> Result<u64>;
  async fn insert(&self, school_id: SchoolId, aggregate: &A) -> Result<()>;
  async fn update(&self, school_id: SchoolId, aggregate: &A) -> Result<()>;
  async fn soft_delete(&self, school_id: SchoolId, id: Uuid) -> Result<()>;
  ```
  No `&TenantContext` parameter anywhere.

---

### FINDING 14 (id: `PORT-STORE-014`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** High
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/port.rs:64-82` (`bulk_insert_student_attendances` default impl)

**Description:**

The default implementation of `bulk_insert_student_attendances` returns `DomainError::NotSupported` and the doc-comment at lines 64-82 justifies this with "adapters that don't (e.g. the Phase 0 SurrealDB stub) get the unsupported error, which is the correct answer for that topology." However, `docs/ports/storage.md:469-477` lists a 10k-attendance-row load test as a port requirement, and `docs/build-plan.md` § Phase 5 names the bulk-marking service as a Phase 5 exit criterion. A silent `NotSupported` from the port's default impl allows an adapter to ship without implementing the feature, and the consumer sees `NotSupported` at the first attendance mark — too late to reconfigure the deployment.

**Expected:**

The trait should distinguish "this adapter does not support bulk-marking" (terminal) from "this adapter has not yet implemented bulk-marking" (placeholder that fails loudly at startup). The port contract at `docs/ports/storage.md:469-477` requires 10k attendance marks in <5s — a hard port requirement, not an optional feature.

**Evidence:**

```rust
  crates/infra/storage/src/port.rs:64-82
  async fn bulk_insert_student_attendances(
      &self,
      ctx: &TenantContext,
      rows: &[StudentAttendanceRow],
  ) -> Result<()> {
      let _ = (ctx, rows);
      Err(educore_core::error::DomainError::not_supported(
          "StorageAdapter::bulk_insert_student_attendances is not supported by this adapter",
      ))
  }
  ```

---

### FINDING 15 (id: `PORT-STORE-015`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** High
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/event_log.rs:142-148` (`EventLog::read` and `count`)

**Description:**

`EventLog::read(filter)` and `EventLog::count(filter)` take the filter by value (consuming it), and `EventLogFilter::limit: u32` is the only pagination control. There is no `cursor: Option<EventId>` / `cursor: Option<VersionCursor>` field on `EventLogFilter`, so a consumer cannot paginate "read up to event Y, then continue from Y+1" — it can only do "events in window [since, until] with limit". The doc-comment at line 145 admits "consumers should paginate" but provides no pagination primitive. For a school's 7-year retention with millions of events, this is a hard limit at `u32::MAX = 4.29B` rows per query, with no way to resume a partial read.

**Expected:**

`docs/schemas/event-schema.md` § 9: "Replay is supported: a consumer can request 'all events of type X since event_id Y' or 'all events of aggregate Z up to time T'." The `EventLogFilter` must carry a `since_event_id: Option<EventId>` cursor and the trait must return the next-page cursor alongside the rows.

**Evidence:**

```rust
  crates/infra/storage/src/event_log.rs:96-104
  pub struct EventLogFilter {
      pub school_id: SchoolId,
      pub event_types: Vec<String>,
      pub since: Option<Timestamp>,
      pub until: Option<Timestamp>,
      pub aggregate_id: Option<Uuid>,
      pub limit: u32,
  }
  ```
  No `since_event_id` cursor. And `EventLog::read` at line 147 takes `filter` by value:
  ```rust
  async fn read(&self, filter: EventLogFilter) -> Result<Vec<EventLogEntry>>;
  ```

---

### FINDING 16 (id: `PORT-STORE-016`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** High
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/change_stream.rs:78-83` (`ChangeStream`)

**Description:**

`ChangeStream::close(self)` is documented as "drops the inner stream, which closes the underlying channel" (line 100-103). For a live CDC subscription (PG `LISTEN/NOTIFY`, SurrealDB `LIVE SELECT`), there is no graceful close — the underlying socket/connection is dropped without an unsubscribe / unlisten handshake, leaving the server side holding the subscription until it times out. For a sync engine with N schools and M concurrent subscribers, this accumulates server-side resource leaks proportional to churn.

**Expected:**

The trait must distinguish "drop" (cancels) from "close" (graceful unsubscribe). `docs/ports/storage.md:111-116` describes sync primitives as "live" CDC with reconnect/resume — a hard-drop close breaks the contract.

**Evidence:**

```rust
  crates/infra/storage/src/change_stream.rs:78-103
  pub struct ChangeStream {
      pub inner: Pin<Box<dyn futures::Stream<Item = Result<ChangeEvent, educore_core::error::DomainError>> + Send + Sync>>,
  }
  ...
  pub async fn close(self) -> Result<(), educore_core::error::DomainError> {
      drop(self);
      Ok(())
  }
  ```

---

### FINDING 17 (id: `PORT-STORE-017`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** High
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/transaction.rs:51-75`

**Description:**

The `Transaction` trait has no `begin_nested` / `savepoint` method. `docs/ports/storage.md` lists no savepoints in the contract, but the engine's bulk-marking service (Phase 5) and the multi-step platform-domain commands (Phase 16) require savepoints to scope per-item error handling without rolling back the parent transaction. Without savepoints, a single failed item in a bulk operation forces a full rollback, contradicting `docs/schemas/command-schema.md` § 12's `CollectErrors` failure policy ("records per-item errors in a result list without aborting the batch").

**Expected:**

`docs/schemas/command-schema.md` § 12: "`failure_policy`: default `FailFast`; alternative is `CollectErrors` which records per-item errors in a result list without aborting the batch." The Transaction port must expose `begin_nested(&self) -> Result<Box<dyn Transaction>>` for savepoint scoping.

**Evidence:**

```rust
  crates/infra/storage/src/transaction.rs:51-75
  async fn commit(self: Box<Self>) -> Result<()>;
  async fn rollback(self: Box<Self>) -> Result<()>;
  fn outbox(&self) -> &dyn Outbox;
  fn audit_log(&self) -> &dyn AuditLog;
  fn idempotency(&self) -> &dyn Idempotency;
  fn event_log(&self) -> &dyn EventLog;
  ```
  No `begin_nested`, no `savepoint`, no nested-transaction contract.

---

### FINDING 18 (id: `PORT-STORE-018`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** High
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/outbox.rs:189-194` (`SerializedEnvelope::from_event_envelope`)

**Description:**

`SerializedEnvelope::from_event_envelope` uses `serde_json::to_vec(&envelope.payload).unwrap_or_default()` (line 194) — a serialization failure silently produces an empty `bytes::Bytes` payload (`{}`) rather than propagating the error. A consumer that subscribes to the event bus sees an event with `payload = "{}"`, with no diagnostic that the original payload failed to serialize. The `metadata` field on the source `EventEnvelope` is also dropped without being copied to the outbox row (the canonical PG DDL declares a `metadata` JSONB column on the outbox table).

**Expected:**

`docs/schemas/event-schema.md` § 3 — payload integrity is a wire-format invariant. The port must propagate serialization errors and must copy `metadata` into the outbox row.

**Evidence:**

```rust
  crates/infra/storage/src/outbox.rs:189-194
  pub fn from_event_envelope(envelope: &educore_events::envelope::EventEnvelope) -> Self {
      Self {
          ...
          payload: bytes::Bytes::from(serde_json::to_vec(&envelope.payload).unwrap_or_default()),
      }
  }
  ```
  No `metadata` field on `SerializedEnvelope` (outbox.rs:60-86) — the bus-port envelope's `metadata` is dropped.

---

### FINDING 19 (id: `PORT-STORE-019`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** High
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/audit.rs:55-58`

**Description:**

The `AuditLogEntry` struct's doc-comment says "`before` and `after` are serialised `serde_json::Value` (adapters are free to use any serializable type via `to_audit_value`)" but `before` and `after` are typed `Option<bytes::Bytes>` (line 80-86) and `metadata` is typed `serde_json::Value` (line 92). `docs/schemas/audit-schema.md` § 2.1 mandates three snapshot policies (`None`, `Diff`, `Full`) — the port has no `AuditSnapshotPolicy` field on `AuditLogEntry`, no `to_audit_value` helper, and no way for the caller to declare which policy to apply.

**Expected:**

`docs/schemas/audit-schema.md` § 2.1 — three snapshot policies configurable per domain. The port must carry the policy marker so adapters know whether to capture `None`, `Diff`, or `Full`.

**Evidence:**

```rust
  crates/infra/storage/src/audit.rs:55-58 (module doc)
  /// before and after are serialised `serde_json::Value`
  /// (adapters are free to use any serializable type via
  /// `to_audit_value`).
  ```
  And the struct fields at lines 80-92:
  ```rust
  pub before: Option<bytes::Bytes>,
  pub after: Option<bytes::Bytes>,
  pub metadata: serde_json::Value,
  ```
  No `to_audit_value` helper exists (`grep -rn "fn to_audit_value" crates/infra/storage/src/` returns no results).

---

### FINDING 20 (id: `PORT-STORE-020`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** High
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/audit.rs:53-93`

**Description:**

`AuditLogEntry::metadata` is typed `serde_json::Value`. AGENTS.md § "Type Safety" forbids `serde_json::Value` in domain code: "No `serde_json::Value` in domain code. Use typed wrappers." Audit metadata is a domain concern; the open-ended `Value` defeats the type-safety guarantees the engine's other sub-ports provide.

**Expected:**

AGENTS.md § "Type Safety": "No `serde_json::Value` in domain code. Use typed wrappers."

**Evidence:**

```rust
  crates/infra/storage/src/audit.rs:92
  pub metadata: serde_json::Value,
  ```
  And `educore_core::value_objects` exports typed wrappers (the engine's audit and event envelopes use `AuditMetadata` / `EventMetadata` structs elsewhere); the storage port uses raw `Value` instead.

---

### FINDING 21 (id: `PORT-STORE-021`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** High
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/outbox.rs:60-86` (`SerializedEnvelope`)

**Description:**

`SerializedEnvelope` declares `aggregate_id: Uuid` (line 73) as a raw UUID instead of a typed `AggregateId` (the engine's typed-identifier pattern). Per AGENTS.md § "Type Safety" and `docs/schemas/database-schema.md` § 1.4: "Identifiers are opaque to consumers. Strings are never parsed" — and "The default implementation uses UUIDv7 (time-ordered) for distributed generation and global uniqueness. Adapter implementations MAY swap to ULID, snowflake, or auto-increment integers behind the storage port, but the engine API always returns typed identifier wrappers." The same struct then exposes `aggregate_type: String` (line 74) instead of an enum.

**Expected:**

AGENTS.md § "Type Safety" and `docs/schemas/database-schema.md` § 1.4 — typed identifier wrappers throughout.

**Evidence:**

```rust
  crates/infra/storage/src/outbox.rs:72-74
  pub aggregate_id: Uuid,
  /// Aggregate type name (e.g. "student"). `String` (not
  /// `&'static str`) so the type is `DeserializeOwned`.
  pub aggregate_type: String,
  ```
  No `AggregateId` or `AggregateType` typed wrapper exists in the crate.

---

### FINDING 22 (id: `PORT-STORE-022`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** High
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/transaction.rs:51-75`

**Description:**

The `Transaction` trait is `Drop`-unsafe by construction. A consumer that holds a `Box<dyn Transaction>` and lets it drop without calling `commit` or `rollback` will trigger the default `Drop` impl on the trait object, which performs no rollback (the trait does not require `Drop`). For an SQL adapter that opens a real `sqlx::Transaction`, dropping the trait object also drops the inner `sqlx::Transaction` — the SQL adapter must implement `Drop` on its concrete type to rollback. But the trait surface never declares this requirement, and the testkit adapter (wave-4 finding TOOL-TK-002) shows the in-memory impl does not roll back on drop — the two implementations behave differently for the same `let _ = tx;` consumer code.

**Expected:**

The port must declare a `Drop` requirement (e.g. "impls MUST rollback on drop if neither commit nor rollback has been called") or expose an explicit `discard()` method. The current ambiguity is a silent data-loss path on panic.

**Evidence:**

```rust
  crates/infra/storage/src/transaction.rs:51-75
  #[async_trait]
  pub trait Transaction: Send + Sync + std::fmt::Debug {
      async fn commit(self: Box<Self>) -> Result<()>;
      async fn rollback(self: Box<Self>) -> Result<()>;
      ...
  }
  ```
  No `Drop` requirement, no `discard()`, no cancellation-safety contract. Compare the testkit finding at `crates/tools/testkit/src/storage.rs:454-461` which has `rollback` as a no-op and waves-3 ADAPTER-PG-026 which has `commit` as a no-op.

---

### FINDING 23 (id: `PORT-STORE-023`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** High
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/repository.rs:17-32` (`Repository<A>` trait declaration)

**Description:**

The `Repository<A>` trait comment at lines 17-32 admits "For Phase 0 minimum-viable, we expose a single generic `Repository<A>` trait that all domain crates can use; when a domain needs aggregate-specific methods it can wrap or extend the generic trait." However, `docs/ports/storage.md:13-25` lists ~22 named repository handles per domain (`students`, `guardians`, `classes`, …) with `Arc<dyn StudentRepository>` style wiring — not a single generic `Repository<A>`. The port's generic shape does not match the contract's per-aggregate-handle shape.

**Expected:**

`docs/ports/storage.md:14-25` — one named repository handle per aggregate root, e.g. `fn students(&self) -> Arc<dyn StudentRepository>`. The Phase 0 minimum-viable single-trait shape is a deviation from the documented contract.

**Evidence:**

```rust
  crates/infra/storage/src/repository.rs:33-36
  #[async_trait]
  pub trait Repository<A>: Send + Sync
  where
      A: Send + Sync + Clone + 'static,
  {
  ```
  Single generic trait. Compare `docs/ports/storage.md:14-25` which lists:
  ```text
  fn students(&self) -> Arc<dyn StudentRepository>;
  fn guardians(&self) -> Arc<dyn GuardianRepository>;
  fn classes(&self) -> Arc<dyn ClassRepository>;
  ... (22 named handles)
  ```

---

### FINDING 24 (id: `PORT-STORE-024`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** High
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/change_stream.rs:62-71` (`AggregateTypeFilter`)

**Description:**

`AggregateTypeFilter::Any` is the wildcard variant, but the doc-comment at line 69-71 says "Storage adapters that don't support wildcards may treat this as 'all types'." This is silent semantic drift between adapters — a consumer that subscribes with `Any` and expects a SQL `LIKE '%'` on a no-wildcard backend gets an undocumented "all types" expansion that may or may not include future aggregate types added after subscription start. The contract provides no way for the consumer to detect which behaviour the adapter implements.

**Expected:**

The trait must declare whether `Any` is a literal wildcard match or an "all currently-known types" expansion, and adapters must report their capability (e.g. `supports_wildcard()`).

**Evidence:**

```rust
  crates/infra/storage/src/change_stream.rs:62-71
  pub enum AggregateTypeFilter {
      Exact(String),
      /// Match any aggregate type. Storage adapters that don't
      /// support wildcards may treat this as "all types".
      Any,
  }
  ```

---

### FINDING 25 (id: `PORT-STORE-025`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** High
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/outbox.rs:113-121` (`Outbox::pending_count` default impl)

**Description:**

`Outbox::pending_count`'s default impl materialises every pending row by calling `self.pending(u32::MAX).await?` and counting the resulting `Vec`. For a school with 1M pending outbox rows (a backlog scenario after a sync-engine outage), this allocates a 1M-element `Vec<SerializedEnvelope>` just to read its `len()`. The doc-comment at line 116-121 says "Adapters with efficient `COUNT(*)` support may override" but provides no upper-bound cap on `u32::MAX` and no fallback for adapters that don't override. The default impl is unbounded memory.

**Expected:**

The default impl must use a streaming/chunked count (e.g. `COUNT(*)` via a separate dedicated query, or chunked iteration with `pending(limit)` capped at e.g. 10000), and the trait must enforce a memory bound.

**Evidence:**

```rust
  crates/infra/storage/src/outbox.rs:113-121
  async fn pending_count(&self, school_id: SchoolId) -> Result<u64> {
      // Default implementation: count via `pending` and check
      // length. Adapters with efficient `COUNT(*)` support may
      // override.
      let _ = school_id;
      Ok(self.pending(u32::MAX).await?.len() as u64)
  }
  ```

---

### FINDING 26 (id: `PORT-STORE-026`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** High
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/event_log.rs:118-132` (`EventLogEntry`)

**Description:**

`EventLogEntry` does not enforce the invariant `recorded_at >= occurred_at`. The doc-comment at line 121 says "Wall-clock time of the persistence (≥ `occurred_at`)" but neither the struct nor the `append` method nor any `new` constructor validates this. An adapter that constructs `EventLogEntry::from_serialized_envelope(env)` (line 154-171) sets `recorded_at: Timestamp::now()` — fine if the clock advances monotonically, but a clock skew / NTP step backwards would produce a row with `recorded_at < occurred_at`, breaking the engine's latency-projection invariant.

**Expected:**

The struct must enforce the invariant at construction (e.g. `EventLogEntry::new(...)` returns `Result<Self, DomainError>` with a `Validation` error on `recorded_at < occurred_at`).

**Evidence:**

```rust
  crates/infra/storage/src/event_log.rs:154-171
  pub fn from_serialized_envelope(env: &super::outbox::SerializedEnvelope) -> Self {
      Self {
          ...
          occurred_at: env.occurred_at,
          recorded_at: Timestamp::now(),
          ...
      }
  }
  ```
  No `if recorded_at < occurred_at` check.

---

### FINDING 27 (id: `PORT-STORE-027`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** High
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/outbox.rs:102-107` (`Outbox::append`)

**Description:**

The `append` doc-comment is contradictory: "uniquely identified by `event_id`; duplicates must be rejected (or, equivalently, stored but never published)". The "or equivalently stored but never published" branch allows an adapter to silently swallow duplicate appends — but the `# Errors` section then says "Conflict if an envelope with the same `event_id` was already appended in the same school." An adapter implementing the silent-swallow branch would never return `Conflict`, breaking callers that rely on the error path to detect duplicate dispatch.

**Expected:**

The contract must be precise: either `Conflict` on duplicate (the `# Errors` arm) or `Ok(())` with no error and no observable side effect (the "equivalent" arm). The doc-comment should pick one and remove the ambiguity.

**Evidence:**

```rust
  crates/infra/storage/src/outbox.rs:102-107
  /// ... the event is uniquely identified by `event_id`;
  /// duplicates must be rejected (or, equivalently, stored but
  /// never published).
  ///
  /// # Errors
  /// - `Conflict` if an envelope with the same `event_id` was
  ///   already appended in the same school.
  ```

---

### FINDING 28 (id: `PORT-STORE-028`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** High
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/change_stream.rs:65-83` (`ChangeStream::inner`)

**Description:**

`ChangeStream.inner` is declared `pub` (line 65-66). Per AGENTS.md § "Type Safety" / "Public items documented" and `docs/code-standards.md` "Public APIs are documented with rustdoc", a `pub` field requires rustdoc. The field has no doc comment, but the `pub` visibility exposes the inner stream directly, allowing external code to poll the stream bypassing the `next()` wrapper (which provides error transposition) and bypassing any backpressure the wrapper might apply.

**Expected:**

The field must be `pub(crate)` or `pub(super)`, not `pub`. External consumers should use `ChangeStream::next()` and `ChangeStream::close()` only.

**Evidence:**

```rust
  crates/infra/storage/src/change_stream.rs:64-72
  pub struct ChangeStream {
      /// The inner stream of change events. Boxed and pinned to
      /// keep the type `dyn`-compatible and awaitable.
      pub inner: Pin<Box<...>>,
  }
  ```
  `pub inner: ...` — directly reachable.

---

### FINDING 29 (id: `PORT-STORE-029`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** Medium
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/idempotency.rs:91-93` (`Idempotency::exists`)

**Description:**

The default implementation of `Idempotency::exists` calls `self.lookup(key).await?` which fully deserialises the outcome payload (a JSON blob that may be MB-sized for a bulk-import outcome). The doc-comment says "adapters with a cheap existence check may override" — but for adapters that do not override, every existence probe on a hot path (e.g. the command dispatcher) deserialises the entire outcome payload. Wave-3 finding ADAPTER-SQ-007 documents the consequence: the SQLite adapter doesn't even deserialise `outcome_version` / `affected_aggregate_ids`, hard-coding them to `0` / `Vec::new()`.

**Expected:**

The trait must declare a separate `is_expired(key, now) -> bool` predicate and the default `exists` should not deserialise the payload (a cheap `EXISTS(...)` query or an `outcome_hash` equality check on the key alone).

**Evidence:**

```rust
  crates/infra/storage/src/idempotency.rs:91-93
  async fn exists(&self, key: IdempotencyCompositeKey) -> Result<bool> {
      Ok(self.lookup(key).await?.is_some())
  }
  ```

---

### FINDING 30 (id: `PORT-STORE-030`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** Medium
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/audit.rs` (entire file, no `audit_id` field)

**Description:**

The canonical PG DDL declares `audit_id UUID NOT NULL` as the primary key (`migrations/engine/0000_engine_core.postgres.sql:96, 116`), but `AuditLogEntry` has no `audit_id` field. The adapter must generate the audit id externally and the port provides no helper. For tamper-evidence (`docs/schemas/audit-schema.md` § 3: "the engine provides no update_audit or delete_audit operation"), the audit id is the anchor for hash-chain / MAC contracts that the port surface never declares.

**Expected:**

The port must carry `audit_id: AuditId` (a typed UUIDv7 wrapper) and expose a tamper-evidence hook (e.g. `audit_hash: Etag` or `audit_signature: Signature`).

**Evidence:**

```rust
  crates/infra/storage/src/audit.rs:62-93
  pub struct AuditLogEntry {
      pub school_id: SchoolId,
      pub actor_id: UserId,
      ...
  }
  ```
  No `audit_id`. Compare `migrations/engine/0000_engine_core.postgres.sql:96`:
  ```
  audit_id        UUID         NOT NULL,
  ...
  PRIMARY KEY (audit_id)
  ```

---

### FINDING 31 (id: `PORT-STORE-031`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** Medium
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/student_attendance_row.rs:108-209`

**Description:**

`StudentAttendanceRow` exposes `*_bytes()` accessors that convert UUID fields to 16-byte big-endian `Vec<u8>` — the accessors' doc-comments (lines 110-114, 116-119, etc.) state "Storage adapters bind UUID columns as raw bytes (`BYTEA` / `VARBINARY` / `BLOB`) per the `attendance_student_attendances` DDL". But the canonical PG DDL for the engine cross-cutting tables at `migrations/engine/0000_engine_core.postgres.sql` uses native `UUID` columns (e.g. line 9: `event_id UUID NOT NULL`). The bulk-attendance storage format is "decoupled from the canonical engine form" (the wave-3 finding ADAPTER-PG-021 acknowledges this), and the port wires the decoupling into the type system. Adapters that follow the port's bytes-on-the-wire contract store `BYTEA`; adapters that follow the engine's UUID-native contract store `UUID` — the port shape picks one and forces it on every adapter.

**Expected:**

The port should expose only typed `Uuid` / `SchoolId` accessors, and the storage adapters should handle dialect-native binding (sqlx has `Type<Uuid>` for native UUID, `Type<Bytes>` for BYTEA).

**Evidence:**

```rust
  crates/infra/storage/src/student_attendance_row.rs:108-118
  pub fn school_id_bytes(&self) -> Vec<u8> {
      self.school_id.as_uuid().as_bytes().to_vec()
  }
  /// Returns the row's `id` as a 16-byte big-endian `Vec<u8>`.
  #[must_use]
  pub fn id_bytes(&self) -> Vec<u8> {
      self.id.as_bytes().to_vec()
  }
  ```

---

### FINDING 32 (id: `PORT-STORE-032`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** Medium
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/lib.rs:46-49`

**Description:**

`lib.rs` re-exports `educore_core::error::Result` (line 48) but does not re-export `educore_core::error::DomainError`. Every public trait method returns `Result<T, DomainError>`, but callers must import `DomainError` from `educore_core` separately. This causes every adapter to write `use educore_core::error::DomainError;` redundantly, and creates an inconsistency where the storage port is reachable as `educore_storage::Result` but the error type must be imported from a different path.

**Expected:**

`lib.rs` should re-export `DomainError` alongside `Result` so the public surface is `educore_storage::{Result, DomainError, ...}`.

**Evidence:**

```rust
  crates/infra/storage/src/lib.rs:46-49
  pub use audit::{AuditLog, AuditLogEntry};
  pub use change_stream::{...};
  ...
  // Re-export the `educore_core::error::Result` alias for convenience.
  pub use educore_core::error::Result;
  ```
  No `pub use educore_core::error::DomainError;`.

---

### FINDING 33 (id: `PORT-STORE-033`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** Medium
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/port.rs:88-94`, 100-106, 115-122, 127-133

**Description:**

The four sync-primitive methods (`watch_changes`, `apply_snapshot`, `cursor_for`, `advance_cursor`) all use `let _ = arg;` in their default impls to suppress unused-variable warnings — a pattern clippy `#[must_use]` would flag and a pattern that loses the named-argument intent. Wave-3 finding ADAPTER-PG-036 confirms the PG adapter overrides `watch_changes` and silently swallows all callers; the default `NotSupported` is the right contract, but the silenced argument names hide the API surface from grep / doc extraction.

**Expected:**

Use `_arg` underscore prefixes (`_filter`, `_snapshot`, `_school_id`, `_to`) or `#[allow(unused_variables)]` on the method bodies; do not bind named variables only to drop them.

**Evidence:**

```rust
  crates/infra/storage/src/port.rs:88-94
  async fn watch_changes(&self, filter: ChangeFilter) -> Result<ChangeStream> {
      let _ = filter;
      Err(educore_core::error::DomainError::not_supported(...))
  }
  async fn apply_snapshot(&self, snapshot: SchoolSnapshot) -> Result<()> {
      let _ = snapshot;
      Err(educore_core::error::DomainError::not_supported(...))
  }
  async fn cursor_for(&self, school_id: SchoolId) -> Result<VersionCursor> {
      let _ = school_id;
      Err(educore_core::error::DomainError::not_supported(...))
  }
  async fn advance_cursor(&self, school_id: SchoolId, to: VersionCursor) -> Result<()> {
      let _ = school_id;
      let _ = to;
      Err(educore_core::error::DomainError::not_supported(...))
  }
  ```

---

### FINDING 34 (id: `PORT-STORE-034`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** Medium
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/port.rs:130-133` (`close(self: Box<Self>)`)

**Description:**

`close` consumes `self: Box<Self>` (line 130). The trait is object-safe and consumed as `Arc<dyn StorageAdapter>` (per the module doc at line 10), but `Arc<dyn StorageAdapter>::close` would require the `Arc` to be unwrapped to a `Box` first — the consumer must `Arc::try_unwrap` (which fails if the Arc is shared) and then `Box::new(arc.into_inner())`. The consumer-facing API is `storage.close().await` (per `docs/ports/storage.md:21`), but no method signature on the public `StorageAdapter` accepts `&self`-style close. The port's signature forces the consumer to perform an `Arc::try_unwrap` dance that may fail at runtime.

**Expected:**

The trait should expose a `close(&self)` that signals shutdown via an internal flag (the wave-3 PG adapter uses an `AtomicBool` for exactly this reason), with `Drop` releasing the connection pool as a fallback.

**Evidence:**

```rust
  crates/infra/storage/src/port.rs:130-133
  /// Closes the adapter, releasing all underlying
  /// connections. After `close`, any further call returns
  /// `Err(Infrastructure)`.
  async fn close(self: Box<Self>) -> Result<()>;
  ```
  And the storage-port docs at `docs/ports/storage.md:21`:
  ```text
  async fn close(&self) -> Result<()>;
  ```
  — note `&self`, not `self: Box<Self>`.

---

### FINDING 35 (id: `PORT-STORE-035`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** Low
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/event_log.rs:155-159`

**Description:**

`EventLogEntry` derives `Eq` (line 119) but the struct contains `Vec<u8>`-shaped `bytes::Bytes` payload which uses reference-counted shared memory; two `EventLogEntry` values are `Eq` only if they share the same `Arc` pointer. The engine's relays / projections commonly clone these entries to fan out to multiple consumers; two cloned entries compare `Eq` because they share the same `Arc`, but two entries with byte-identical payloads built independently are NOT `Eq` (they have different `Arc` pointers). This is a foot-gun for adapter parity tests.

**Expected:**

Either drop `Eq` (use `PartialEq` only) or implement byte-wise equality on the payload.

**Evidence:**

```rust
  crates/infra/storage/src/event_log.rs:118-132
  #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  pub struct EventLogEntry {
      ...
      #[serde(with = "bytes_via_vec")]
      pub payload: bytes::Bytes,
      ...
  }
  ```

---

### FINDING 36 (id: `PORT-STORE-036`)

- **Source:** `docs/audit_reports/findings/wave4-storage-port.md`
- **Severity:** Low
- **Area:** infra-storage
- **Location:** `crates/infra/storage/src/port.rs:60-83` (transaction.rs:78-105)

**Description:**

The `bulk_insert_student_attendances` methods on both `StorageAdapter` and `Transaction` carry an identical signature (modulo the `&TenantContext` argument) and identical doc-comments, but the trait surfaces do not share a helper type or trait alias. The same field-by-field validation (school_id match, dedup on `(school_id, student_id, attendance_date)`) is duplicated across both methods and across every adapter (wave-3 finding ADAPTER-PG-025 confirms the dedup logic is re-implemented in `bulk_attendance.rs`). The port exposes no shared validator helper.

**Expected:**

A `BulkAttendanceInsert` trait or `validate_bulk_attendance(ctx, rows) -> Result<Vec<&Row>>` free function in the port, so the validation contract has one source of truth.

**Evidence:**

```rust
  crates/infra/storage/src/port.rs:60-82
  async fn bulk_insert_student_attendances(
      &self,
      ctx: &TenantContext,
      rows: &[StudentAttendanceRow],
  ) -> Result<()> { ... }
  ```
  ```rust
  crates/infra/storage/src/transaction.rs:88-104
  async fn bulk_insert_student_attendances(&self, rows: &[StudentAttendanceRow]) -> Result<()> { ... }
  ```
  Two near-identical signatures, no shared validator.

---


## Storage Parity (tools) (target id prefix: `PAR`)

**Path:** `crates/tools/storage-parity/`  
**Total findings:** 31 (7 critical, 11 high, 9 medium, 4 low)


### FINDING 1 (id: `PAR-001`)

- **Source:** `docs/audit_reports/findings/wave4-storage-parity.md`
- **Severity:** Critical
- **Area:** tools-parity
- **Location:** `crates/tools/storage-parity/tests/parity_transaction_commit_rollback.rs:21-32` (module doc)

**Description:**

The transaction commit/rollback parity test explicitly admits that every shipped adapter (testkit, SQLite, SurrealDB, PostgreSQL, MySQL) implements `commit` and `rollback` as flag-only operations. A rolled-back transaction is documented to leave its writes visible to subsequent transactions. The test only asserts `commit` and `rollback` return `Ok(())` — never that atomicity holds — and the file declares the gap as an open backlog item. Any consumer relying on the engine's published "all sub-port writes are atomic with the command's mutation" guarantee is silently exposed to a write-skew.

**Expected:**

Per `docs/ports/storage.md:133-136`: "On `commit` the writes are persisted and the outbox events are released to the event bus. On `rollback` the writes are discarded and the outbox is cleared." Per `docs/schemas/event-schema.md` (engine invariant): the outbox row is part of the same transaction as the aggregate mutation, guaranteeing atomicity.

**Evidence:**

```rust
  // crates/tools/storage-parity/tests/parity_transaction_commit_rollback.rs:21-32
  //! **Known limitation:** every storage adapter shipped in
  //! Phase 1 (testkit, SQLite, SurrealDB, PostgreSQL, MySQL)
  //! currently implements `Transaction::commit` and
  //! `Transaction::rollback` as flag-only operations. The
  //! sub-port writes are auto-committed at the query boundary
  //! (SQLite/SurrealDB) or live in shared state without
  //! per-transaction isolation (testkit). A rolled-back
  //! transaction therefore MAY leave its writes visible to a
  //! subsequent transaction.
  ```

---

### FINDING 2 (id: `PAR-002`)

- **Source:** `docs/audit_reports/findings/wave4-storage-parity.md`
- **Severity:** Critical
- **Area:** tools-parity
- **Location:** `crates/tools/storage-parity/tests/parity_idempotency_collision.rs:23-30` (module doc) + `parity_idempotency_collision.rs:177-182` (test)

**Description:**

The idempotency parity suite asserts the "same outcome = no-op" path on every backend but explicitly skips the "same key + different outcome = Conflict" path on the three reference adapters (SQLite, SurrealDB, PG, MySQL). Only the in-memory testkit adapter is verified to surface `ErrorKind::Conflict`. The file's module doc admits "the SQLite + SurrealDB reference adapters currently implement `record` as a plain `INSERT` / `INSERT OR REPLACE` and therefore accept a re-write with a different outcome." This is the head-of-line contract from `docs/ports/storage.md § 6` (at-least-once delivery) and `docs/decisions/ADR-014-Idempotency.md`; the parity suite claims all 5 backends support it while only 1 actually does.

**Expected:**

Per `docs/ports/storage.md` § 6 + `ADR-014`: a duplicate idempotency key with a different outcome must surface `Conflict`. The parity matrix at `parity_behavior_matrix.rs:69-74` declares all 5 backends `supported = true` for `idempotency_collision` — that is materially false for 4 of them.

**Evidence:**

```rust
  // crates/tools/storage-parity/tests/parity_idempotency_collision.rs:23-30
  //! 2. Storing the same `(school_id, command_type,
  //!    idempotency_key)` with the **same outcome** is a no-op
  //!    (returns `Ok(())`).
  //!
  //! The "same key + different outcome = Conflict" half of the
  //! contract is enforced by the testkit in-memory backend (see
  //! `crates/tools/testkit/src/storage.rs::IdempotencyHandle`).
  //! The SQLite + SurrealDB reference adapters currently
  //! implement `record` as a plain `INSERT` / `INSERT OR REPLACE`
  ```

---

### FINDING 3 (id: `PAR-003`)

- **Source:** `docs/audit_reports/findings/wave4-storage-parity.md`
- **Severity:** Critical
- **Area:** tools-parity
- **Location:** `crates/tools/storage-parity/tests/parity_outbox_to_event_log_relay.rs:34-43` (module doc) + `parity_outbox_to_event_log_relay.rs:139-150` (test)

**Description:**

The outbox → event log payload-round-trip parity test explicitly skips the payload semantic comparison on SurrealDB because the SurrealDB outbox column is typed `object`, which collapses to `Object {}` on read-back. The test code at lines 139-150 carries a `is_surrealdb_deviation` heuristic that silences the assertion. The matrix at `parity_behavior_matrix.rs` nevertheless marks this `(feature, backend) = (outbox_append, surrealdb)` as `supported = true`. A core parity invariant (the event payload survives the relay without mutation) is unenforced on the engine's primary Phase 0 adapter.

**Expected:**

Per `docs/ports/storage.md § 4` + `docs/schemas/event-schema.md § 1.1`: "the relay is a pure transformation — `event_id` is the canonical primary key, and the payload must survive the round-trip without mutation." The matrix must mark `outbox_append` on SurrealDB as `supported = false` (or `partial`) until the deviation is fixed.

**Evidence:**

```rust
  // crates/tools/storage-parity/tests/parity_outbox_to_event_log_relay.rs:34-43
  //! **Known deviation:** the SurrealDB outbox/event_log adapter
  //! pair is currently known to drop the payload on the
  //! outbox → event_log hop (the outbox column is typed
  //! `object`, which collapses to `Object {}` on the read-back).
  ```
  ```rust
  // crates/tools/storage-parity/tests/parity_outbox_to_event_log_relay.rs:139-150
  let is_surrealdb_deviation = expected_payload
      != serde_json::Value::Object(serde_json::Map::new())
      && actual_payload == serde_json::Value::Object(serde_json::Map::new());
  if !is_surrealdb_deviation {
      assert_eq!(
          actual_payload, expected_payload,
          ...
      );
  }
  ```

---

### FINDING 4 (id: `PAR-004`)

- **Source:** `docs/audit_reports/findings/wave4-storage-parity.md`
- **Severity:** Critical
- **Area:** tools-parity
- **Location:** `crates/tools/storage-parity/tests/parity_event_log_filter.rs:140-159` (test) + module doc lines 19-22

**Description:**

The event log filter parity test on the `aggregate_id` axis explicitly catches and swallows an `Err(_)` that contains the substring `"SurrealUuid"` in its debug string. The test does not fail on SurrealDB; it accepts the error as a "documented deviation" of the engine's primary Phase 0 backend. The deviation marker is `aggregate_id` filter emits `aggregate_id = SurrealUuid::from('<uuid>')` which is not valid SurrealQL. Any consumer that subscribes to event log changes filtered by aggregate id on SurrealDB will hit this bug silently.

**Expected:**

Per `docs/schemas/event-schema.md § 6` + `docs/ports/storage.md § 7`: every axis of `EventLogFilter` must be honored identically across all backends. The matrix row `("event_log_filter", "surrealdb", "surql", true)` at `parity_behavior_matrix.rs:61` is false.

**Evidence:**

```rust
  // crates/tools/storage-parity/tests/parity_event_log_filter.rs:140-159
  match rows {
      Ok(rows) => {
          assert_eq!(rows.len(), 1, ...);
      }
      Err(e) if format!("{e:?}").contains("SurrealUuid") => {
          // SurrealDB known deviation: invalid SurrealQL
          // syntax in the aggregate_id filter. Skipped.
      }
      Err(e) => panic!(...),
  }
  ```

---

### FINDING 5 (id: `PAR-005`)

- **Source:** `docs/audit_reports/findings/wave4-storage-parity.md`
- **Severity:** Critical
- **Area:** tools-parity
- **Location:** `crates/tools/storage-parity/tests/` — every domain integration file except `parity_*.rs` (e.g., `academic_integration.rs:332-440`, `assessment_integration.rs:329-425`, `finance_integration.rs:0`, `library_integration.rs:0`, `hr_integration.rs:0`)

**Description:**

Of the 14 domain vertical-slice integration tests (`academic`, `assessment`, `attendance`, `cms`, `communication`, `documents`, `events`, `facilities`, `finance`, `hr`, `library`, `operations`, `settings`), only `academic`, `assessment`, `attendance`, `cms`, `documents`, `events`, `operations`, `settings` have an `#[ignore]`-d Postgres + MySQL variant; `finance`, `hr`, `library`, `facilities`, `communication` have zero `#[ignore]` variants — they run on SQLite only and provide no parity coverage for the production-target adapters. The crate's `Cargo.toml` dev-dependencies list all four adapters, so the wiring exists; the test surface simply does not use it for 5 of 14 domains.

**Expected:**

Per `docs/build-plan.md:1653-1656` Phase 16 task 2: "a cross-adapter parity test suite that runs the same scenario against PG, MySQL, SQLite, and the in-memory testkit impl, asserting identical observable behavior." Per the README's stated mission: "runs the same schema-creation and CRUD scenarios against all three shipped storage adapters."

**Evidence:**

`grep -c "#\[ignore"` on `tests/finance_integration.rs`, `tests/hr_integration.rs`, `tests/library_integration.rs`, `tests/facilities_integration.rs`, `tests/communication_integration.rs` all return `0`. No `setup_pg`, `setup_mysql`, or `setup_surrealdb` calls appear in any of these 5 files.

---

### FINDING 6 (id: `PAR-006`)

- **Source:** `docs/audit_reports/findings/wave4-storage-parity.md`
- **Severity:** Critical
- **Area:** tools-parity
- **Location:** `crates/tools/storage-parity/tests/cms_integration.rs:1141-1156` (`cms_integration_pg_vertical_slice`, `cms_integration_mysql_vertical_slice`) + `operations_integration.rs:184-194` + `settings_integration.rs:183-193`

**Description:**

The CMS, Operations, and Settings PG/MySQL `#[ignore]`-d vertical-slice variants are empty stubs: they construct `setup_test_env()` (or a bare `_school`) but perform zero storage operations and make zero assertions. The function body is `let _env = setup_test_env().await;` for CMS and `let _school = SchoolId::from_uuid(uuid::Uuid::new_v4());` for the other two. These "tests" cannot pass or fail meaningfully; if `cargo test -- --ignored` is run with the env vars set, they will pass vacuously regardless of whether PG/MySQL actually implement the domain correctly.

**Expected:**

Per `docs/build-plan.md:1653-1656` Phase 16 task 2: the parity test must assert identical observable behavior across PG, MySQL, SQLite, and testkit. An empty stub satisfies neither the build-plan nor the README.

**Evidence:**

```rust
  // crates/tools/storage-parity/tests/cms_integration.rs:1141-1146
  #[tokio::test]
  #[ignore = "requires EDUCORE_PG_URL env var"]
  async fn cms_integration_pg_vertical_slice() {
      // The PG adapter is wired in `educore-storage-parity`; for
      // Phase 12 the SQLite scenario covers the headline path.
      // This test is a placeholder that triggers when the PG
      // URL is set in CI.
      let _env = setup_test_env().await;
  }
  ```
  ```rust
  // crates/tools/storage-parity/tests/operations_integration.rs:184-194
  #[tokio::test]
  #[ignore = "requires EDUCORE_PG_URL env var"]
  async fn operations_integration_pg_vertical_slice() {
      let _school = SchoolId::from_uuid(uuid::Uuid::new_v4());
  }
  ```

---

### FINDING 7 (id: `PAR-007`)

- **Source:** `docs/audit_reports/findings/wave4-storage-parity.md`
- **Severity:** Critical
- **Area:** tools-parity
- **Location:** `crates/tools/storage-parity/tests/documents_integration.rs:1028-1068` (`documents_integration_postgres`, `documents_integration_mysql`)

**Description:**

The Documents PG/MySQL `#[ignore]`-d variants connect, migrate, and immediately discard the adapter (`let _adapter: Arc<dyn educore_storage::StorageAdapter> = Arc::new(adapter);`). They perform zero writes, zero reads, zero assertions. The test passes if the connection succeeds, regardless of whether the documents schema actually round-trips on PG/MySQL. The module doc at line 1022-1026 frames this as "the SQLite scenario covers the headline path" — meaning the entire Documents domain has no parity coverage on the production-target adapters.

**Expected:**

Per `docs/build-plan.md:1653-1656` + `docs/coverage.toml:697` (Documents): parity coverage must extend to PG/MySQL.

**Evidence:**

```rust
  // crates/tools/storage-parity/tests/documents_integration.rs:1028-1068
  #[tokio::test]
  #[ignore = "requires EDUCORE_PG_URL; run with: EDUCORE_PG_URL=postgres://... cargo test -- --ignored"]
  async fn documents_integration_postgres() {
      let url = match std::env::var("EDUCORE_PG_URL") {
          Ok(s) if !s.is_empty() => s,
          _ => return,
      };
      ...
      let adapter = educore_storage_postgres::PostgresStorageAdapter::connect(&url, school)
          .await
          .expect("connect pg");
      adapter.migrate().await.expect("migrate pg");
      let _adapter: Arc<dyn educore_storage::StorageAdapter> = Arc::new(adapter);
  }
  ```

---

### FINDING 10 (id: `PAR-010`)

- **Source:** `docs/audit_reports/findings/wave4-storage-parity.md`
- **Severity:** High
- **Area:** tools-parity
- **Location:** `crates/tools/storage-parity/tests/cross_cutting_integration.rs:400-410` (`cross_cutting_integration_postgres`, `cross_cutting_integration_mysql`)

**Description:**

The PG/MySQL variants of the cross-cutting integration test do not exercise the bus. The SQLite variant subscribes to `test-cross-cutting` via `InProcessEventBus` and asserts the event reaches subscribers (lines 252-267); the PG and MySQL variants only check `outbox.pending() == 0` and `event_log` row count (lines 386-398, 451-461). A PG/MySQL adapter that correctly appends to the event log but never publishes to the bus would pass these tests. The asymmetry is undocumented in the test code.

**Expected:**

Per `docs/ports/storage.md § 4`: the relay publishes to the bus. The parity suite must assert the bus receives the envelope on every backend, including PG/MySQL.

**Evidence:**

```rust
  // crates/tools/storage-parity/tests/cross_cutting_integration.rs:386-398 (PG)
  let tx = adapter.begin().await.expect("begin");
  let pending = tx.outbox().pending(10).await.expect("pending");
  assert!(pending.is_empty(), "PG outbox should be drained");
  let events = tx.event_log().read(...).await.expect("read");
  assert_eq!(events.len(), 1, "PG event_log should have 1 row");
  // No bus.subscribe / bus.next() / EventSubscription check.
  ```

---

### FINDING 11 (id: `PAR-011`)

- **Source:** `docs/audit_reports/findings/wave4-storage-parity.md`
- **Severity:** High
- **Area:** tools-parity
- **Location:** `crates/tools/storage-parity/tests/cross_cutting_integration.rs:361-410` (`pg_rls_blocks_cross_tenant_audit_reads`)

**Description:**

The PG RLS tenant-isolation test depends on a `tenant_b` non-superuser role provisioned by `tools/scripts/pg-rls-test-setup.sql`, per its module doc. The script does not exist in the repository (`find . -name "pg-rls-test-setup.sql"` returns nothing) and the test fallbacks to `EDUCORE_PG_TENANT_B_URL` with `unwrap_or_else(|_| url.clone())` — meaning if the env var is unset, the test silently connects as the superuser and "passes" without exercising RLS. The test cannot fail meaningfully in CI without manual setup that the repo does not encode.

**Expected:**

Per `docs/ports/storage.md:140-149`: tenant isolation is enforced at the storage adapter layer and "a test suite that attempts to read across tenants and fails" must run as a CI gate. The current wiring makes the gate optional and undocumented.

**Evidence:**

```rust
  // crates/tools/storage-parity/tests/cross_cutting_integration.rs:362-365
  /// Phase 2 OQ coverage. Requires the test runner to provision
  /// a non-superuser `tenant_b` role with SELECT on
  /// engine.audit_log BEFORE running this test. The setup
  /// script lives at `tools/scripts/pg-rls-test-setup.sql`
  ```
  ```bash
  $ find . -name "pg-rls-test-setup.sql"
  (no results)
  ```
  ```rust
  // crates/tools/storage-parity/tests/cross_cutting_integration.rs:407-410
  let url_b = std::env::var("EDUCORE_PG_TENANT_B_URL").unwrap_or_else(|_| url.clone());
  // If unset, falls back to superuser URL -> RLS bypassed -> assertion trivially passes.
  ```

---

### FINDING 12 (id: `PAR-012`)

- **Source:** `docs/audit_reports/findings/wave4-storage-parity.md`
- **Severity:** High
- **Area:** tools-parity
- **Location:** `crates/tools/storage-parity/tests/` (12 of 14 domain integration files; e.g., `finance_integration.rs`, `hr_integration.rs`, `library_integration.rs`, `facilities_integration.rs`, `communication_integration.rs`)

**Description:**

No parity test exercises a cross-domain workflow — none of the 14 domain integration files pair commands from two domains. The `finance` → `academic` flow ("student promoted → fee structure regenerates"), the `hr` → `payroll` → `finance` flow, the `library` → `communication` (fine notification) flow, the `attendance` → `communication` (absent notification) flow, the `documents` → `cms` (`form_uploaded_public_indexing_subscriber`) cross-domain subscriber all lack end-to-end parity coverage. The `cms_form_uploaded_public_indexing_subscriber_*` tests at `cms_integration.rs:1079-1115` only verify the pure function (mapping envelope → `FormIndexAction`) and never run it against a real bus + storage adapter.

**Expected:**

Per `docs/specs/academic/workflows.md`, `docs/specs/finance/workflows.md`, `docs/specs/hr/workflows.md`: cross-domain flows are documented scenarios and require parity coverage per `docs/build-plan.md:1653-1656`.

**Evidence:**

`grep -rl "cross_domain\|cross-domain\|cross domain" crates/tools/storage-parity/tests/` returns no results. None of the 26 test files imports from more than one domain crate (verified by inspecting imports in `finance_integration.rs`, `hr_integration.rs`, `library_integration.rs`, `attendance_integration.rs`).

---

### FINDING 13 (id: `PAR-013`)

- **Source:** `docs/audit_reports/findings/wave4-storage-parity.md`
- **Severity:** High
- **Area:** tools-parity
- **Location:** `crates/tools/storage-parity/tests/parity_event_log_filter.rs:174-184` (`event_log_filter_sqlite`)

**Description:**

No parity test exercises `since` and `until` time-range filters on `EventLogFilter`. The module doc at lines 19-22 lists `since` and `until` as filter axes that "must be honored identically by every backend", but the test body never sets `filter.since` or `filter.until` — only school, event_types, aggregate_id, and limit are exercised (the test's own numbered comments 1-6 also omit since/until). The `since`/`until` axes silently bypass CI parity.

**Expected:**

Per `docs/ports/storage.md § 7` + `docs/schemas/event-schema.md § 6`: `EventLogFilter` includes `since` and `until` time bounds that must be honored identically.

**Evidence:**

```rust
  // crates/tools/storage-parity/tests/parity_event_log_filter.rs:19-22 (module doc)
  //! Asserts that `EventLogFilter` (school_id + event_types +
  //! since + until + aggregate_id) is honored identically by
  //! every backend.
  ```
  The test body's filter mutations (lines 87, 100, 134, 161, 173, 181) never reference `filter.since` or `filter.until`.

---

### FINDING 14 (id: `PAR-014`)

- **Source:** `docs/audit_reports/findings/wave4-storage-parity.md`
- **Severity:** High
- **Area:** tools-parity
- **Location:** `crates/tools/storage-parity/tests/parity_idempotency_collision.rs:78-105` (`assert_outcome_conflict_on_testkit`)

**Description:**

The testkit-only outcome-conflict path is the canonical idempotency contract path, yet it has no `#[ignore]`-d PG/MySQL/SurrealDB counterpart and the conflict-detection logic on the SQL adapters is never asserted anywhere. The module doc at lines 23-30 admits the SQL adapters "currently implement `record` as a plain `INSERT` / `INSERT OR REPLACE` and therefore accept a re-write with a different outcome." The matrix at `parity_behavior_matrix.rs:69-74` nonetheless marks all 5 backends `supported = true` for `idempotency_collision`. A consumer relying on at-least-once delivery on PG/MySQL will silently double-write on retry.

**Expected:**

Per `ADR-014-Idempotency.md` + `docs/ports/storage.md § 6`: the conflict path is a hard contract that all 4 production adapters must implement. The matrix must mark the 4 non-testkit rows as `false` until they are wired.

**Evidence:**

```rust
  // crates/tools/storage-parity/tests/parity_idempotency_collision.rs:177-182
  #[tokio::test]
  async fn idempotency_outcome_conflict_testkit() {
      ...
  }
  ```
  No corresponding `idempotency_outcome_conflict_sqlite`, `_postgres`, `_mysql`, or `_surrealdb` test exists in the file.

---

### FINDING 15 (id: `PAR-015`)

- **Source:** `docs/audit_reports/findings/wave4-storage-parity.md`
- **Severity:** High
- **Area:** tools-parity
- **Location:** `crates/tools/storage-parity/tests/parity_audit_cross_tenant_isolation.rs:170-175` (file end) + `crates/tools/storage-parity/tests/cross_cutting_integration.rs:380-410` (`pg_rls_blocks_cross_tenant_audit_reads`)

**Description:**

Tenant isolation is asserted at the application layer (`read_for_target` filters by `school_id`) but the underlying SQL RLS or SurrealDB access scopes are never verified. A PG/MySQL/SurrealDB backend that forgets the `WHERE school_id = ?` clause in its `read_for_target` implementation would pass the application-level assertion if every other backend also forgot the clause — but would fail on real production traffic where the RLS layer is the only line of defense. No parity test stands up a DB-side admin role and confirms it cannot bypass the application filter.

**Expected:**

Per `docs/schemas/tenancy-schema.md § 4` + `docs/ports/storage.md:140-149`: tenant isolation is enforced at the storage adapter layer; the engine refuses to issue queries lacking a tenant filter. The parity suite must include a "DB-admin bypass" test on PG (RLS) and SurrealDB (session auth scopes) to confirm the database-level enforcement.

**Evidence:**

No test file in `crates/tools/storage-parity/tests/` contains the strings `RLS`, `bypass`, `admin`, `policy`, or `EXPLAIN` (verified by `grep -rl "RLS\|bypass\|policy" crates/tools/storage-parity/tests/` — only `pg_rls_blocks_cross_tenant_audit_reads` matches, and it depends on a missing script per PAR-011).

---

### FINDING 16 (id: `PAR-016`)

- **Source:** `docs/audit_reports/findings/wave4-storage-parity.md`
- **Severity:** High
- **Area:** tools-parity
- **Location:** `crates/tools/storage-parity/tests/` (entire suite, 26 files)

**Description:**

No parity test exercises bulk-insert operations. The trait method `Transaction::bulk_insert_student_attendances` (`crates/infra/storage/src/transaction.rs:86-91`) is a documented performance path for bulk attendance marking, and the storage-port contract documents bulk snapshot writes at `docs/ports/storage.md:106`. None of the 158 test attributes in the parity suite cover bulk-insert — no test invokes a service function that internally calls `bulk_insert_student_attendances`, no test imports the method, and the attendance_integration.rs file only inserts single-row attendances.

**Expected:**

Per `docs/build-plan.md:1721-1735` Phase 17 task 2: "Load test: 10k students, bulk fee invoice generation … Target: p95 < 500 ms for a bulk-invoice-of-10k-rows command on PG." The Phase 16 parity suite must assert bulk-insert parity across backends before Phase 17 can establish the p95 baseline.

**Evidence:**

`grep -rln "bulk_insert\|bulk_insert_student" crates/tools/storage-parity/tests/` returns zero results.

---

### FINDING 17 (id: `PAR-017`)

- **Source:** `docs/audit_reports/findings/wave4-storage-parity.md`
- **Severity:** High
- **Area:** tools-parity
- **Location:** `crates/tools/storage-parity/tests/` (entire suite, 26 files)

**Description:**

No parity test exercises concurrent writes. The 158-test suite is entirely sequential; no test uses `tokio::join!`, `tokio::spawn`, or any concurrent primitive. A storage-port contract violation where the SQLite adapter uses a single-writer mutex or the testkit uses a non-locking `RefCell` for outbox/audit/idempotency state — and the SQL adapters use row-level locking — cannot be caught by parity because concurrency is never introduced. This is critical because the parity suite is the engine's only cross-adapter gate.

**Expected:**

Per `docs/ports/storage.md § 3` + § "transaction conflict": the `Conflict(String)` error variant at `docs/ports/storage.md:220` exists precisely to surface write-write conflicts; the parity suite must exercise that path.

**Evidence:**

`grep -rln "join!\|tokio::spawn\|tokio::select" crates/tools/storage-parity/tests/` returns zero results.

---

### FINDING 18 (id: `PAR-018`)

- **Source:** `docs/audit_reports/findings/wave4-storage-parity.md`
- **Severity:** High
- **Area:** tools-parity
- **Location:** `crates/tools/storage-parity/tests/port_auth_integration.rs:127-147` + `port_files_integration.rs:107-128` + `port_integrations_integration.rs:113-134` + `port_notify_integration.rs:117-138` + `port_payment_integration.rs:159-180`

**Description:**

All five Phase 15 port integration files (`port_auth_integration.rs`, `port_files_integration.rs`, `port_integrations_integration.rs`, `port_notify_integration.rs`, `port_payment_integration.rs`) follow an identical structure: 5 sync scenarios (always-on, exercising pure functions + trait surface) + 2 env-gated `#[ignore]`-d async scenarios that **do not actually call the network**. The async scenarios are bare builder calls (`JwtAuthProviderBuilder::new().build()`, `S3FileStorage::builder()...build()`, `LmsIntegrationBuilder::new()...build()`, `StripeProviderBuilder::new()...build()`, `EmailProviderBuilder::new()...build()`) — no `authenticate()` is called, no file is uploaded, no webhook is dispatched. They pass vacuously regardless of whether the port implementations work.

**Expected:**

Per `docs/ports/event-bus.md`, `docs/ports/authentication.md`, `docs/ports/file-storage.md`: the port contracts describe end-to-end behavior (authenticate, put_object, send, etc.) and the parity suite must exercise that path on at least the testkit impl and the reference impl.

**Evidence:**

```rust
  // crates/tools/storage-parity/tests/port_auth_integration.rs:127-138
  #[tokio::test]
  #[ignore = "requires EDUCORE_PORT_ADAPTER_E2E env var; run with: cargo test -- --ignored"]
  async fn port_auth_async_jwt_full_round_trip() {
      let provider = JwtAuthProviderBuilder::new().build();
      let _session = provider
          .authenticate(Credential::Anonymous)
          .await
          .expect("anonymous auth should succeed");
  }
  ```
  Note: the 5 port files for files, integrations, notify, and payment contain only `_storage = ...build()` builders — they never call any port method.

---

### FINDING 8 (id: `PAR-008`)

- **Source:** `docs/audit_reports/findings/wave4-storage-parity.md`
- **Severity:** High
- **Area:** tools-parity
- **Location:** `crates/tools/storage-parity/tests/parity_behavior_matrix.rs:36-94` (entire `PARITY_MATRIX` const) + `parity_behavior_matrix.rs:115-133` (`every_feature_is_either_5_supported_or_fully_unsupported`)

**Description:**

The behavior matrix declares 6 features × 5 backends = 30 rows, but the `every_feature_is_either_5_supported_or_fully_unsupported` test enforces an "all-or-nothing" invariant that the file itself then violates in production. Findings PAR-002/003/004 above document three features (idempotency_collision, outbox_append, event_log_filter) where 1–4 backends silently do not honor the contract yet all are marked `supported = true`. The matrix's "all-or-nothing" assertion passes because every row is `true`, so it actively prevents CI from flagging the partial-coverage gaps that the actual scenario tests admit. The "documentation test" gives a false sense of coverage.

**Expected:**

Per the file's own module doc lines 12-19: the matrix is "the shape the engine contract requires" and "tests assert behaviour, not dialect strings." The whole point is to surface partial coverage. Either the matrix must mark `false` for non-conforming rows, or the suite must add per-row `assert_supported` tests that fail CI when a partial-feature backend is shipped.

**Evidence:**

```rust
  // crates/tools/storage-parity/tests/parity_behavior_matrix.rs:36-94
  const PARITY_MATRIX: &[(&str, &str, &str, bool)] = &[
      // ---- outbox_append (5/5) ----
      ("outbox_append", "testkit", "in-memory", true),
      ("outbox_append", "sqlite", "sqlite3", true),
      ("outbox_append", "surrealdb", "surql", true), // FALSE per PAR-003
      ("outbox_append", "postgres", "pg", true),
      ("outbox_append", "mysql", "mysql", true),
      ...
      ("idempotency_collision", "surrealdb", "surql", true), // FALSE per PAR-002
      ...
  ];
  ```

---

### FINDING 9 (id: `PAR-009`)

- **Source:** `docs/audit_reports/findings/wave4-storage-parity.md`
- **Severity:** High
- **Area:** tools-parity
- **Location:** `crates/tools/storage-parity/tests/parity_cross_backend_equivalence.rs:130-170` (`dispatch_create_school`) + `parity_idempotency_collision.rs:127-151`

**Description:**

The cross-backend parity suite duplicates the `relay_outbox_to_event_log` logic from `common::mod.rs:62-78` in the `dispatch_create_school` helper at `parity_cross_backend_equivalence.rs:130-170`. The duplicated helper drains the outbox **inside** the original transaction rather than as a separate relay pass, then calls `relay(adapter)` after commit, which is a no-op because the outbox is already empty. The module doc at lines 95-99 acknowledges the in-tx drain is what makes the testkit backend (which drains on commit) work. This means the parity test does NOT exercise the production relay path; it only exercises a within-transaction drain that no real consumer code uses.

**Expected:**

Per `docs/ports/storage.md § 4` + `docs/schemas/event-schema.md`: the outbox is consumed by an external relay process; the parity suite should stand up the relay as a separate component and assert the end-to-end flow (write → outbox → relay → event_log). The current shape hides a bug where the relay is broken (writes survive the relay without reaching the event_log) because the test is wired to short-circuit.

**Evidence:**

```rust
  // crates/tools/storage-parity/tests/parity_cross_backend_equivalence.rs:144-170
  // Drain the outbox to the event log WITHIN the transaction
  // so the testkit backend (which drains on commit) does not
  // lose the envelope before the relay runs.
  let pending = tx.outbox().pending(100).await.expect("pending");
  for env in &pending {
      let entry = educore_storage::event_log::EventLogEntry::from_serialized_envelope(env);
      tx.event_log().append(entry).await.expect("event_log append");
      tx.outbox().mark_published(&[env.event_id]).await.expect(...);
  }
  tx.commit().await.expect("commit");
  bus.publish(envelope).await.expect("bus publish");
  let _ = relay(adapter).await; // <-- no-op: outbox already drained
  ```

---

### FINDING 19 (id: `PAR-019`)

- **Source:** `docs/audit_reports/findings/wave4-storage-parity.md`
- **Severity:** Medium
- **Area:** tools-parity
- **Location:** `crates/tools/storage-parity/tests/common/mod.rs:75-86` (`setup_testkit`) + `tests/parity_cross_backend_equivalence.rs:217-241` (`cross_backend_create_school_and_audit_equivalence_testkit` + sqlite + surrealdb)

**Description:**

The 3 always-on backends (testkit, SQLite, SurrealDB) all run in-process via `in_memory(...)`. The parity suite never exercises a file-backed SQLite, a remote SurrealDB, or any other "real" deployment. The module doc at lines 9-22 promises "the test surface for unit / integration tests that do not need a real DB" — but the matrix labels them with `in-memory` / `sqlite3` / `surql` dialects, implying they are representative of those backends. A parity gap between file-backed SQLite and in-memory SQLite would be invisible to the suite.

**Expected:**

Per `docs/schemas/sql-dialects/sqlite.md` (dialect-specific quirks around WAL mode, temp tables, etc.) + `docs/ports/storage.md`: the parity matrix must distinguish between in-memory and persistent backends when dialect differences exist. The current setup conflates them.

**Evidence:**

```rust
  // crates/tools/storage-parity/tests/common/mod.rs:75-86
  pub fn setup_testkit() -> (Arc<dyn StorageAdapter>, SchoolId, TenantContext) {
      let bus: Arc<dyn EventBus> = Arc::new(InProcessEventBus::new());
      let adapter = educore_testkit::storage::InMemoryStorageAdapter::new(bus);
      ...
  }
  pub async fn setup_sqlite() -> ... {
      let adapter = educore_storage_sqlite::SqliteStorageAdapter::in_memory(school).await...;
  }
  pub async fn setup_surrealdb() -> ... {
      let adapter = educore_storage_surrealdb::SurrealStorageAdapter::in_memory(school).await...;
  }
  ```
  No `setup_sqlite_file()` or `setup_surrealdb_remote()` helper exists.

---

### FINDING 20 (id: `PAR-020`)

- **Source:** `docs/audit_reports/findings/wave4-storage-parity.md`
- **Severity:** Medium
- **Area:** tools-parity
- **Location:** `crates/tools/storage-parity/tests/parity_audit_cross_tenant_isolation.rs:68-100` (entire `assert_cross_tenant_isolation`)

**Description:**

The cross-tenant isolation test only exercises the audit_log sub-port. The parity suite never asserts cross-tenant isolation for the event_log, outbox, or idempotency sub-ports. A PG/MySQL/SQLite/SurrealDB adapter that scopes audit reads correctly but leaks event_log rows across tenants (e.g., the `aggregate_id` filter in `parity_event_log_filter.rs:140-159` already has a known SurrealDB syntax bug) would pass this audit-only test.

**Expected:**

Per `docs/schemas/tenancy-schema.md § 4` + `docs/ports/storage.md:140-149`: tenant isolation is a cross-cutting invariant on every sub-port (outbox, audit, event_log, idempotency). The parity suite must assert isolation on each.

**Evidence:**

```rust
  // crates/tools/storage-parity/tests/parity_audit_cross_tenant_isolation.rs:1-15 (module doc)
  //! Asserts that an audit row written for `school_a` is NOT
  //! visible to `read_for_target` when the caller passes
  //! `school_b`...
  ```
  No equivalent `parity_event_log_cross_tenant_isolation.rs`, `parity_outbox_cross_tenant_isolation.rs`, or `parity_idempotency_cross_tenant_isolation.rs` exists.

---

### FINDING 21 (id: `PAR-021`)

- **Source:** `docs/audit_reports/findings/wave4-storage-parity.md`
- **Severity:** Medium
- **Area:** tools-parity
- **Location:** `crates/tools/storage-parity/tests/parity_event_log_filter.rs:174-189` + `parity_audit_cross_tenant_isolation.rs:155-175`

**Description:**

No parity test exercises the `cancelled` / `soft_deleted` / `archived` `ActiveStatus` axis. The `EventLogEntry` struct carries an `active_status: ActiveStatus` field, and the audit log carries an immutable history, but the parity suite never inserts an entry with `ActiveStatus::SoftDeleted` or `Archived` and never verifies that `read` filters on `active_status` or that the event log distinguishes them. The test `cross_backend_create_school_and_audit_equivalence_*` at `parity_cross_backend_equivalence.rs:178-204` asserts `active_status == ActiveStatus::Active` only by default.

**Expected:**

Per `docs/schemas/event-schema.md` + `docs/ports/storage.md § 7`: the engine distinguishes `Active`, `SoftDeleted`, `Archived`; the parity suite must cover all three.

**Evidence:**

`grep -rln "SoftDeleted\|Archived\|ActiveStatus::" crates/tools/storage-parity/tests/` returns zero results.

---

### FINDING 22 (id: `PAR-022`)

- **Source:** `docs/audit_reports/findings/wave4-storage-parity.md`
- **Severity:** Medium
- **Area:** tools-parity
- **Location:** `crates/tools/storage-parity/tests/` (entire suite, 26 files)

**Description:**

No parity test exercises unique-constraint violations. The storage-port sub-ports do not surface a `UniqueViolation` error path explicitly, but the engine's uniqueness invariants (school code, admission no, employee id, ISBN, etc.) are enforced at the application layer. None of the 158 tests attempt to insert a duplicate (school_code, admission_no) pair and assert that the engine surfaces a `Conflict` error. The 5 parity suites assert the happy path only.

**Expected:**

Per `docs/specs/platform/overview.md` + `docs/specs/academic/aggregates.md`: the engine guarantees uniqueness on admission_no, employee_id, ISBN, school_code, and the parity suite must verify that the storage adapter surfaces a conflict error on duplicate insert.

**Evidence:**

`grep -rln "UniqueViolation\|unique_violation\|duplicate" crates/tools/storage-parity/tests/` returns zero results. None of the tests in the 14 domain integration files attempt to insert a duplicate primary key.

---

### FINDING 23 (id: `PAR-023`)

- **Source:** `docs/audit_reports/findings/wave4-storage-parity.md`
- **Severity:** Medium
- **Area:** tools-parity
- **Location:** `crates/tools/storage-parity/tests/` (entire suite, 26 files)

**Description:**

No parity test exercises cascading deletes. The 10 domain crates have aggregate roots with children (e.g., `School → Class → Section → Student → Attendance`, `Wallet → WalletTransaction`, `Library → BookIssue → BookReturn`). None of the tests call a delete that would cascade and assert the children are gone (or correctly preserved per the soft-delete convention). The parity suite does not surface a SQL `FOREIGN KEY … ON DELETE CASCADE` semantic across backends.

**Expected:**

Per `docs/specs/academic/tables.md` + `docs/schemas/sql-dialects/comparison.md`: SQLite defaults to `ON DELETE RESTRICT`, PG/MySQL default varies; the parity suite must assert the cascade behavior is identical.

**Evidence:**

`grep -rln "cascad\|ON DELETE\|delete_cascade" crates/tools/storage-parity/tests/` returns zero results.

---

### FINDING 24 (id: `PAR-024`)

- **Source:** `docs/audit_reports/findings/wave4-storage-parity.md`
- **Severity:** Medium
- **Area:** tools-parity
- **Location:** `crates/tools/storage-parity/tests/parity_cross_backend_equivalence.rs:181-184` + `parity_audit_cross_tenant_isolation.rs:135-140` + `parity_event_log_filter.rs:191-194` + `parity_idempotency_collision.rs:155-158` + `parity_outbox_to_event_log_relay.rs:160-163` + `parity_transaction_commit_rollback.rs:179-182`

**Description:**

The "always-on" trio of backends (testkit + SQLite + SurrealDB) all share the `educore-storage-surrealdb` crate dependency for `setup_surrealdb`. The SurrealDB backend is documented as `Phase 0 primary per ADR-017` (per `common/mod.rs:96`), yet the env-gated PG/MySQL variants outnumber the always-on backends by 2:1. In practice CI on a developer laptop runs 3/5 backends; in CI with env vars set, it runs 5/5 — but only the SurrealDB parity is asserted at every PR. The "always-on" surface is the de-facto engine contract; if SurrealDB is the primary, the test surface should mirror that, not skip it.

**Expected:**

Per `docs/decisions/ADR-017-StorageStrategy.md` (cited at `common/mod.rs:96`): the primary storage backend is the canonical contract. The 5 parity suites should run the same number of assertions on SurrealDB as on PG/MySQL.

**Evidence:**

In each of the 6 parity files, the always-on trio (testkit, sqlite, surrealdb) is structurally identical and asserts identical invariants; the env-gated pair (pg, mysql) does the same. No asymmetry should exist, but per PAR-002/003/004, the SurrealDB row is the one that fails to honor 3 of the 6 contracts.

---

### FINDING 25 (id: `PAR-025`)

- **Source:** `docs/audit_reports/findings/wave4-storage-parity.md`
- **Severity:** Medium
- **Area:** tools-parity
- **Location:** `crates/tools/storage-parity/tests/finance_integration.rs:417` + `crates/tools/storage-parity/tests/library_integration.rs:394` + `crates/tools/storage-parity/tests/facilities_integration.rs:552` + `crates/tools/storage-parity/tests/communication_integration.rs:812`

**Description:**

Four domain integration tests (`finance_integration.rs`, `library_integration.rs`, `facilities_integration.rs`, `communication_integration.rs`) end without a `#[tokio::test]` PG/MySQL/SurrealDB variant. None of them carry `setup_pg`, `setup_mysql`, or `setup_surrealdb` imports; their Cargo.toml-dev-dep adapters (`educore-storage-postgres`, `educore-storage-mysql`, `educore-storage-surrealdb`) are listed but never instantiated. The five backend crate dependencies in `Cargo.toml:33-38` therefore ship for parity but only `educore-storage-sqlite` is exercised by 4 of the 14 domain integration files.

**Expected:**

Per `docs/coverage.toml:697-710`: each domain's parity row names `crates/tools/storage-parity/tests/<domain>_integration.rs`. The test file is required to exercise parity for that domain; the file's existence is not proof of parity coverage.

**Evidence:**

Per-file `#[ignore]` counts: `finance_integration.rs: 0`, `library_integration.rs: 0`, `facilities_integration.rs: 0`, `communication_integration.rs: 0`. The four files contain zero `EDUCORE_PG_URL` / `EDUCORE_MYSQL_URL` references (verified by grep above).

---

### FINDING 26 (id: `PAR-026`)

- **Source:** `docs/audit_reports/findings/wave4-storage-parity.md`
- **Severity:** Medium
- **Area:** tools-parity
- **Location:** `crates/tools/storage-parity/tests/communication_integration.rs:33-100` + `crates/tools/storage-parity/tests/communication_integration.rs:195-770` (`mod compile_full_prelude_scenarios`)

**Description:**

The Communication integration test does not exercise any communication domain command. The always-on tests are `communication_package_metadata_is_set` and `communication_full_prelude_scenarios_compile_only_when_wired` (lines 33-100), which only assert the `PACKAGE_NAME` constant and document scenarios as code comments. The `mod compile_full_prelude_scenarios` nested block at lines 195-770 contains six `#[tokio::test]` attributes, but the module's compile flag is off by default (per the file's module doc lines 41-44: "stubbed behind `compile_full_prelude_scenarios`") and therefore the six scenarios never run under `cargo test`.

**Expected:**

Per `docs/build-plan.md:1653-1656` + `docs/specs/communication/workflows.md`: the parity suite must execute the Communication headline scenarios end-to-end.

**Evidence:**

```rust
  // crates/tools/storage-parity/tests/communication_integration.rs:42-44 (module doc)
  //! Scenarios that depend on symbols
  //! not yet wired into `educore-communication`'s prelude are
  //! stubbed behind `compile_full_prelude_scenarios` (off by
  //! default)
  ```

---

### FINDING 27 (id: `PAR-027`)

- **Source:** `docs/audit_reports/findings/wave4-storage-parity.md`
- **Severity:** Medium
- **Area:** tools-parity
- **Location:** `crates/tools/storage-parity/tests/parity_idempotency_collision.rs:177-182` + `parity_transaction_commit_rollback.rs:147-155` + `parity_event_log_filter.rs:115-160`

**Description:**

The parity suite duplicates the `Transaction::rollback` and `EventLogFilter::aggregate_id` workarounds between files but does not centralize the "known deviations" list. Each file documents its deviation in a free-text module doc paragraph (e.g., PAR-001, PAR-002, PAR-003 above). There is no single registry of known parity gaps that a new developer can consult before adding a backend. The `PARITY_MATRIX` const at `parity_behavior_matrix.rs:36-94` is the closest analogue but lists `supported = true` everywhere, contradicting the actual scenario test outcomes.

**Expected:**

Per `docs/code-standards.md` + `docs/ports/storage.md`: known deviations must be tracked in a single source (e.g., `docs/audit_reports/known-deviations.md` or a `KNOWN_PARITY_DEVIATIONS` const that the matrix itself checks against).

**Evidence:**

Three separate `**Known deviation:**` / `**Known limitation:**` paragraphs at `parity_outbox_to_event_log_relay.rs:34-43`, `parity_idempotency_collision.rs:23-30`, `parity_event_log_filter.rs:53-61` — plus the rollback-known-limitation paragraph at `parity_transaction_commit_rollback.rs:21-32`. None reference each other; the matrix at `parity_behavior_matrix.rs` does not flag them.

---

### FINDING 28 (id: `PAR-028`)

- **Source:** `docs/audit_reports/findings/wave4-storage-parity.md`
- **Severity:** Low
- **Area:** tools-parity
- **Location:** `crates/tools/storage-parity/tests/academic_integration.rs:444-491` + `assessment_integration.rs:455-499` + `attendance_integration.rs:619-784` + `finance_integration.rs:267-417` + `hr_integration.rs:260-385` + `library_integration.rs:246-394` + `facilities_integration.rs:345-552` + `cms_integration.rs:649-1079` + `documents_integration.rs:689-1025`

**Description:**

The 14 domain integration tests follow a "mirrors the Phase N pattern" naming convention documented in each module doc, but the actual test surface is not parity-shaped: 12 of the 14 files run only on SQLite and perform only service-fn + bus assertion, with zero cross-adapter shape. The "parity" claim in the file-level docs (e.g., "Runs on SQLite (always) + PG/MySQL (env-gated)") is misleading because the env-gated variants are absent or empty in 5 of 14 files (see PAR-005/006/007). A reader scanning the module docs would believe parity coverage exists where it does not.

**Expected:**

Per `docs/build-plan.md:1653-1656`: the module doc must accurately describe what the test exercises.

**Evidence:**

Compare `finance_integration.rs:1-16` ("Runs on SQLite (always) + PG/MySQL (env-gated)") with the file's actual contents — no `#[ignore]`-d PG/MySQL test exists, and no `setup_pg`/`setup_mysql` call appears anywhere in the file.

---

### FINDING 29 (id: `PAR-029`)

- **Source:** `docs/audit_reports/findings/wave4-storage-parity.md`
- **Severity:** Low
- **Area:** tools-parity
- **Location:** `crates/tools/storage-parity/tests/finance_integration.rs:376-393` + `library_integration.rs:356-393` + `attendance_integration.rs:619-784` (entire file end)

**Description:**

The `#[test]` (sync) variants of the event-type round-trip tests duplicate `assert_eq!` checks on `EVENT_TYPE` and `AGGREGATE_TYPE` constants, but they exercise the constants directly without going through `serde_json::to_value` + `from_value`. A back-end that returns a payload whose `event_type` field is serialized differently (e.g., the engine's `surrealdb` payload collapse documented in PAR-003) would not be caught by these sync round-trip tests because they only verify the constant string, not the wire form.

**Expected:**

Per `docs/schemas/event-schema.md § 5` (event schema) + `docs/ports/storage.md § 4`: event types are wire-form identifiers and must round-trip through JSON serialization identically on every backend.

**Evidence:**

```rust
  // crates/tools/storage-parity/tests/finance_integration.rs:267-280
  #[test]
  fn finance_event_type_round_trip_for_all_headline_aggregates() {
      ...
      assert_eq!(
          <InvoiceNumberingConfigured as DomainEvent>::EVENT_TYPE,
          "finance.fees_invoice.configured"
      );
  ```
  No `serde_json::to_string(&ev)` + `from_str` round-trip is performed.

---

### FINDING 30 (id: `PAR-030`)

- **Source:** `docs/audit_reports/findings/wave4-storage-parity.md`
- **Severity:** Low
- **Area:** tools-parity
- **Location:** `crates/tools/storage-parity/tests/cms_integration.rs:495-647` + `documents_integration.rs:577-885` + `events_integration.rs:371-685`

**Description:**

The CMS, Documents, and Events integration tests construct their own in-memory mocks (`InMemoryAuditLog`, `InMemoryPageRepo`, `InMemoryFormRepo`) rather than going through the `educore-storage-surrealdb` adapter or `educore-testkit::storage`. The mocks implement `AuditLog`, `PageRepository`, etc. directly via `async_trait`, bypassing the parity suite's own setup helpers (`common::setup_sqlite`, `setup_surrealdb`, `setup_testkit`). A parity gap between the mock impl and the real adapter impl is invisible — the test passes when the mocks behave correctly, regardless of whether the adapters do.

**Expected:**

Per `crates/tools/testkit/src/storage.rs` (in-memory adapter exists) + `crates/adapters/storage-sqlite/src/`: the parity suite must use the same backend plumbing the engine ships, not bespoke in-memory mocks.

**Evidence:**

`grep -rln "InMemoryAuditLog\|InMemoryPageRepo\|InMemoryFormRepo" crates/tools/storage-parity/tests/` returns 3 files (`cms_integration.rs`, `documents_integration.rs`, `events_integration.rs`). None of them use `common::setup_testkit()` or `common::setup_sqlite()`.

---

### FINDING 31 (id: `PAR-031`)

- **Source:** `docs/audit_reports/findings/wave4-storage-parity.md`
- **Severity:** Low
- **Area:** tools-parity
- **Location:** `crates/tools/storage-parity/Cargo.toml:32-38` (`[dev-dependencies]`) + `docs/build-plan.md:1699` (Exit criteria 4: "`cargo test --workspace` green")

**Description:**

The `Cargo.toml` `[dev-dependencies]` list pulls all four production adapters (`educore-storage-sqlite`, `educore-storage-postgres`, `educore-storage-mysql`, `educore-storage-surrealdb`) plus `educore-testkit` plus all five Phase 15 port adapters. This list is a single source of CI compile time pressure — every PR's `cargo test --workspace` must compile all 10 adapter crates even if the parity test surface uses only `educore-storage-sqlite` (per PAR-005/006/007/019/025/026). The dependency graph does not feature-gate the adapters, so a CI runner that lacks network access to crates.io cannot run the suite even though all 14 domain integration tests are SQLite-only.

**Expected:**

Per `docs/code-standards.md` + ADR-015-ExternalCrates.md: external crate selection must consider cross-compile and CI isolation; a parity suite that requires 10 adapter crates to compile but exercises only 1–2 of them in practice is a maintenance and CI cost without a corresponding test coverage benefit.

**Evidence:**

```toml
  # crates/tools/storage-parity/Cargo.toml:32-38
  [dev-dependencies]
  educore-storage-sqlite = { workspace = true }
  educore-storage-postgres = { workspace = true }
  educore-storage-mysql = { workspace = true }
  educore-storage-surrealdb = { workspace = true }
  educore-testkit = { workspace = true }
  educore-auth = { workspace = true }
  educore-notify = { workspace = true }
  educore-payment = { workspace = true }
  educore-files = { workspace = true }
  educore-integrations = { workspace = true }
  ```

---


## Testkit (tools) (target id prefix: `TOOL-TK`)

**Path:** `crates/tools/testkit/`  
**Total findings:** 28 (2 critical, 5 high, 13 medium, 8 low)


### FINDING 1 (id: `TOOL-TK-001`)

- **Source:** `docs/audit_reports/findings/wave4-testkit.md`
- **Severity:** Critical
- **Area:** tools
- **Location:** `crates/tools/testkit/src/storage.rs:431-452`

**Description:**

`InMemoryTransaction::commit` drains the outbox into a local `_pending` Vec and drops it. It never publishes the drained envelopes to the event bus even though the `bus` field is wired to `InProcessEventBus`. As a result, every domain command that writes to the outbox via the testkit adapter emits zero downstream events. The comment at lines 441-446 explicitly admits this: "the in-memory testkit does not republish envelopes to the bus". Any integration test that asserts "after `tx.commit()`, a subscriber on `world.bus` receives the event" will fail silently.

**Expected:**

Per `docs/ports/storage.md:104-108`: "Every state change is written to the outbox in the same transaction as the aggregate mutation. A separate relay reads pending events and publishes them to the event bus. Consumers see at-least-once delivery." Per `docs/build-plan.md:1653-1656`: "in-memory impls of all 6 ports… Consumer tests use these to run domain commands without docker" — implying the in-memory world should preserve end-to-end event semantics.

**Evidence:**

```rust
  crates/tools/testkit/src/storage.rs:431-452
  #[async_trait]
  impl Transaction for InMemoryTransaction {
      async fn commit(self: Box<Self>) -> Result<()> {
          if self.rolled_back.load(Ordering::SeqCst) {
              return Err(DomainError::validation("transaction already rolled back"));
          }
          if self.committed.swap(true, Ordering::SeqCst) {
              return Err(DomainError::validation("transaction already committed"));
          }
          // Drain the outbox; the in-memory testkit does not
          // republish envelopes to the bus (the SerializedEnvelope
          // shape uses owned Strings while EventEnvelope expects
          // &'static str, so a strict conversion is awkward). The
          // outbox-drain test asserts that the outbox is empty
          // after commit; the bus is exercised separately by tests
          // that publish directly via the bus port.
          let _pending: Vec<SerializedEnvelope> = {
              let mut outbox = self.inner.outbox.lock();
              outbox.drain(..).collect()
          };
          Ok(())
      }
  ```

---

### FINDING 2 (id: `TOOL-TK-002`)

- **Source:** `docs/audit_reports/findings/wave4-testkit.md`
- **Severity:** Critical
- **Area:** tools
- **Location:** `crates/tools/testkit/src/storage.rs:454-461`

**Description:**

`InMemoryTransaction::rollback` only flips the `rolled_back` `AtomicBool`; it does NOT discard any staged writes. All sub-port handles (`OutboxHandle::append`, `AuditLogHandle::append`, `EventLogHandle::append`, `IdempotencyHandle::record`) write directly to the shared `Arc<InMemoryInner>` state at call time (lines 86-218). So a rollback does not roll back; subsequent transactions observe the rolled-back writes.

**Expected:**

Per `docs/ports/storage.md` and the `Transaction` trait doc at `crates/infra/storage/src/transaction.rs:45-47`: "Rolls the transaction back. All staged writes are discarded. Consumes the transaction."

**Evidence:**

```rust
  crates/tools/testkit/src/storage.rs:454-461
  async fn rollback(self: Box<Self>) -> Result<()> {
      if self.committed.load(Ordering::SeqCst) {
          return Err(DomainError::validation("transaction already committed"));
      }
      self.rolled_back.store(true, Ordering::SeqCst);
      Ok(())
  }
  ```
  And the test that codifies the broken behavior:
  ```rust
  crates/tools/testkit/src/storage.rs:647-663
  #[test]
  fn begin_rollback_discards_outbox() {
      ...
      tx.outbox().append(sample_envelope(school)).await.unwrap();
      tx.rollback().await.unwrap();
      let tx2 = adapter.begin().await.unwrap();
      let pending = tx2.outbox().pending(10).await.unwrap();
      // The first tx was rolled back so the outbox still
      // has the envelope; the second tx sees it.
      assert_eq!(pending.len(), 1);
  }
  ```

---

### FINDING 18 (id: `TOOL-TK-018`)

- **Source:** `docs/audit_reports/findings/wave4-testkit.md`
- **Severity:** High
- **Area:** tools
- **Location:** `crates/tools/testkit/src/payment.rs:65-83` and `crates/tools/testkit/src/payment.rs:97-114` (`charge` impl)

**Description:**

`InMemoryPaymentProvider` uses an `AtomicU64::fetch_add(1, Ordering::Relaxed)` then `.wrapping_add(1)` to compute the next id. The `Relaxed` ordering combined with `.wrapping_add(1)` on the previous result means the first call returns 2 (because `fetch_add` returns the previous value 0, then `.wrapping_add(1)` makes it 1, but then `Self::default()` initialized the counter to 0 with a prior `fetch_add` returning 0+1=1). Actually the test `id_seq_starts_at_one_and_increments` confirms `peek_id()` returns 1, 2, 3. The `charge`/`refund` use the same `peek_id` so they mint `in-mem-charge-1`, `in-mem-charge-2`. The bug is subtler: `peek_id` returns `fetch_add(..).wrapping_add(1)` which means the first call to `peek_id` returns 1 (since `fetch_add` returns 0 → `0.wrapping_add(1)` = 1), but the underlying counter is now 1. The second call returns 2 (fetch_add returns 1, +1 = 2), counter now 2. This is internally consistent. **However**, `peek_id` mutates `id_seq` even though its name says "peek". The naming lies: every call increments.

**Expected:**

Per `AGENTS.md` § Agent Instructions: "No `#[allow(dead_code)]` or `_var` prefixes to silence the compiler. Delete unused code, wire it in, or open a follow-up issue." `peek_id` should either be renamed to `next_id` (it mutates) or not mutate.

**Evidence:**

```rust
  crates/tools/testkit/src/payment.rs:65-67
  fn peek_id(&self) -> u64 {
      self.id_seq.fetch_add(1, Ordering::Relaxed).wrapping_add(1)
  }
  ```
  And the test that confirms the mutation-on-peek behavior:
  ```rust
  crates/tools/testkit/src/payment.rs:343-348
  #[test]
  fn id_seq_starts_at_one_and_increments() {
      let provider = InMemoryPaymentProvider::new();
      assert_eq!(provider.peek_id(), 1);
      assert_eq!(provider.peek_id(), 2);
      assert_eq!(provider.peek_id(), 3);
  }
  ```

---

### FINDING 21 (id: `TOOL-TK-021`)

- **Source:** `docs/audit_reports/findings/wave4-testkit.md`
- **Severity:** High
- **Area:** tools
- **Location:** `crates/tools/testkit/README.md:1-3`

**Description:**

README states the testkit provides in-memory implementations of "the engine's six ports (storage, auth, notify, payment, files, and event-bus)". This list omits the `IntegrationGateway` port. The lib.rs doc-comment (line 3) and the handoff (line 22 of PHASE-16-HANDOFF.md) both say "seven ports" including `IntegrationGateway`. The `integrations.rs` module ships `InMemoryIntegrationGateway` as a real port impl. So the README is stale.

**Expected:**

Per `docs/build-plan.md:1653-1656` (task 1, lists 6 ports) vs. `docs/handoff/PHASE-16-HANDOFF.md:22-24` and `crates/tools/testkit/src/lib.rs:3-6` (both say 7 ports including integrations): one source must be wrong. The crate actually delivers 7 ports; the build-plan/README are out of sync.

**Evidence:**

```markdown
  crates/tools/testkit/README.md:1-3
  # educore-testkit
  
  The testkit crate provides in-memory implementations of the engine's six ports (storage, auth, notify, payment, files, and event-bus) for use in unit and integration tests.
  ```
  vs.
  ```rust
  crates/tools/testkit/src/lib.rs:1-6
  //! # educore-testkit
  //!
  //! In-memory test adapters for the engine's seven ports
  //! (StorageAdapter + AuthProvider + NotificationProvider +
  //! PaymentProvider + FileStorage + IntegrationGateway +
  //! EventBus). For unit and integration tests only.
  ```
  And the build plan:
  ```
  docs/build-plan.md:1653-1656
  1. `educore-testkit`: in-memory impls of all 6 ports
     (`StorageAdapter`, `AuthProvider`, `NotificationProvider`,
     `PaymentProvider`, `FileStorage`, `EventBus`). Consumer tests use
     these to run domain commands without docker.
  ```

---

### FINDING 3 (id: `TOOL-TK-003`)

- **Source:** `docs/audit_reports/findings/wave4-testkit.md`
- **Severity:** High
- **Area:** tools
- **Location:** `crates/tools/testkit/src/storage.rs:431-477` (entire `impl Transaction for InMemoryTransaction`)

**Description:**

`InMemoryTransaction` does not override `Transaction::bulk_insert_student_attendances` (defined at `crates/infra/storage/src/transaction.rs:86-91`). Because the trait's default implementation returns `DomainError::NotSupported`, any domain command that calls `tx.bulk_insert_student_attendances(&rows)` against the in-memory adapter will fail at runtime — yet the same call against Postgres/MySQL/SQLite adapters succeeds.

**Expected:**

Per `crates/infra/storage/src/transaction.rs:66-86`: the trait explicitly notes the bulk-marking service uses the transactional form "so the outbox appends, the idempotency record, the audit row, and the `StudentAttendance` rows all commit atomically." The testkit is meant to exercise this path; the default `NotSupported` blocks that exercise.

**Evidence:**

```rust
  crates/infra/storage/src/transaction.rs:86-91
  async fn bulk_insert_student_attendances(&self, rows: &[StudentAttendanceRow]) -> Result<()> {
      let _ = rows;
      Err(educore_core::error::DomainError::not_supported(
          "Transaction::bulk_insert_student_attendances is not supported by this adapter",
      ))
  }
  ```
  And the storage-adapter form (which IS implemented):
  ```rust
  crates/tools/testkit/src/storage.rs:307-340
  async fn bulk_insert_student_attendances(
      &self,
      ctx: &TenantContext,
      rows: &[StudentAttendanceRow],
  ) -> Result<()> { ... }
  ```
  No override of the same method on `impl Transaction for InMemoryTransaction` at lines 432-477.

---

### FINDING 4 (id: `TOOL-TK-004`)

- **Source:** `docs/audit_reports/findings/wave4-testkit.md`
- **Severity:** High
- **Area:** tools
- **Location:** `crates/tools/testkit/src/storage.rs:81-107` (`impl Outbox for OutboxHandle`)

**Description:**

The outbox sub-port does not enforce the per-school partition mandated by the port docstring. `OutboxHandle::append` accepts any envelope (no school validation); `OutboxHandle::pending` returns the first `limit` envelopes regardless of school. The port trait at `crates/infra/storage/src/outbox.rs:104-108` documents: "The outbox is partitioned by `school_id` so callers see only envelopes for their school."

**Expected:**

Per `crates/infra/storage/src/outbox.rs:104-108` and `crates/infra/storage/src/outbox.rs:115` (`pending_count` takes `school_id`): `pending` should filter the drain by `school_id`.

**Evidence:**

```rust
  crates/tools/testkit/src/storage.rs:97-100
  async fn pending(&self, limit: u32) -> Result<Vec<SerializedEnvelope>> {
      let outbox = self.0.outbox.lock();
      Ok(outbox.iter().take(limit as usize).cloned().collect())
  }
  ```
  The `school_id` field on each envelope is ignored entirely.

---

### FINDING 5 (id: `TOOL-TK-005`)

- **Source:** `docs/audit_reports/findings/wave4-testkit.md`
- **Severity:** High
- **Area:** tools
- **Location:** `crates/tools/testkit/src/storage.rs:193-219` (`impl Idempotency for IdempotencyHandle`)

**Description:**

`IdempotencyHandle` does not override the default `exists` method (defined at `crates/infra/storage/src/idempotency.rs:90-92`). The default calls `lookup` and checks `is_some`, which is functionally correct but allocates a full `IdempotencyRecord` clone for every existence check (the engine's dispatcher uses this on every retry). The testkit's port-completeness contract is to ship its own idiomatic override matching the in-memory backend, even if the trait default happens to work.

**Expected:**

Per `crates/infra/storage/src/idempotency.rs:86-92`: "adapters with a cheap existence check may override." A `HashMap::contains_key` lookup is strictly cheaper than a `HashMap::get` + `Option::cloned`.

**Evidence:**

```rust
  crates/infra/storage/src/idempotency.rs:90-92
  async fn exists(&self, key: IdempotencyCompositeKey) -> Result<bool> {
      Ok(self.lookup(key).await?.is_some())
  }
  ```
  No `async fn exists` override on `IdempotencyHandle` at `crates/tools/testkit/src/storage.rs:193-219`.

---

### FINDING 10 (id: `TOOL-TK-010`)

- **Source:** `docs/audit_reports/findings/wave4-testkit.md`
- **Severity:** Medium
- **Area:** tools
- **Location:** `crates/tools/testkit/src/auth.rs:124-131` (`validate` impl)

**Description:**

`InMemoryAuthProvider::validate` returns `AuthError::Expired` for any non-`Bearer` `AuthScheme` (Cookie or `Custom`). The port contract permits all three schemes (`Bearer`, `Cookie`, `Custom`), and `Cookie` is the second-most-common auth surface. Returning `Expired` for a never-presented cookie is semantically wrong; the correct variant is `Malformed` (matching `JwtAuthProvider::validate` at `crates/adapters/auth/src/jwt.rs:399-403`).

**Expected:**

Per `crates/adapters/auth/src/port.rs:131-142` and the parallel `JwtAuthProvider::validate` (`crates/adapters/auth/src/jwt.rs:398-404`): the correct error for "wrong scheme" is `AuthError::Malformed`, not `AuthError::Expired`.

**Evidence:**

```rust
  crates/tools/testkit/src/auth.rs:124-131
  async fn validate(&self, token: &AuthToken) -> Result<Session, AuthError> {
      if !matches!(token.scheme, AuthScheme::Bearer) {
          return Err(AuthError::Expired);
      }
      let key = format!("{:?}", token.value);
      let sessions = self.sessions.lock().unwrap_or_else(PoisonError::into_inner);
      sessions.get(&key).cloned().ok_or(AuthError::Expired)
  }
  ```

---

### FINDING 11 (id: `TOOL-TK-011`)

- **Source:** `docs/audit_reports/findings/wave4-testkit.md`
- **Severity:** Medium
- **Area:** tools
- **Location:** `crates/tools/testkit/src/auth.rs:91-167` (`impl AuthProvider`)

**Description:**

The credential-key derivation is unstable for `Credential::Bearer`: it uses `format!("{token:?}")`, which renders the inner `String` via `Debug`. Any non-printable byte, escape, or quote in the token would produce a different string from the value the caller passes back in a subsequent `AuthToken::value` lookup (because `validate` formats `token.value` via `{:?}` and `authenticate` also formats the same `String` via `Debug`). For typical printable ASCII this works, but the lookup key is bound to the `Debug` rendering rather than the value — making the round-trip fragile.

**Expected:**

Per `crates/adapters/auth/src/port.rs:62-72` and the comment in `crates/tools/testkit/src/auth.rs:170-176`: the lookup key should be derived from the credential directly, not from a `Debug` rendering.

**Evidence:**

```rust
  crates/tools/testkit/src/auth.rs:117-119
  let key = credential_key(&credential)?;
  let mut sessions = self.sessions.lock().unwrap_or_else(PoisonError::into_inner);
  sessions.insert(key, session.clone());
  ```
  And:
  ```rust
  crates/tools/testkit/src/auth.rs:185-186
  Credential::Bearer(token) => Ok(format!("{token:?}")),
  ```

---

### FINDING 12 (id: `TOOL-TK-012`)

- **Source:** `docs/audit_reports/findings/wave4-testkit.md`
- **Severity:** Medium
- **Area:** tools
- **Location:** `crates/tools/testkit/src/notify.rs:108-115` (`send` impl)

**Description:**

`InMemoryNotificationProvider::send` does NOT honor the `request.idempotency_key: Option<IdempotencyKey>` field. Every call generates a fresh `NotificationReceipt` with a new `receipt_id`. The port spec at `docs/ports/notifications.md:162-166` mandates: "`idempotency_key` is used by the adapter to deduplicate retries. The engine generates a deterministic key from `(command_id, recipient, template_version)` so the same logical send is not duplicated."

**Expected:**

Per `docs/ports/notifications.md:162-166` and the port trait docstring at `crates/adapters/notify/src/port.rs:1180-1185`: `idempotency_key` deduplicates retries. The testkit `send` must check the key, return the stored receipt on match.

**Evidence:**

```rust
  crates/tools/testkit/src/notify.rs:110-115
  async fn send(&self, request: SendNotification) -> Result<NotificationReceipt> {
      let receipt = Self::make_receipt(&request.channel);
      let mut sends = self.sends.lock().unwrap_or_else(PoisonError::into_inner);
      sends.push(request);
      Ok(receipt)
  }
  ```
  No `idempotency_key` lookup; the `multiple_sends_are_recorded_in_order` test (lines 269-279) asserts the receipts are distinct (which is correct for distinct sends, but does not exercise idempotency).

---

### FINDING 13 (id: `TOOL-TK-013`)

- **Source:** `docs/audit_reports/findings/wave4-testkit.md`
- **Severity:** Medium
- **Area:** tools
- **Location:** `crates/tools/testkit/src/notify.rs:117-134` (`send_bulk` impl)

**Description:**

`InMemoryNotificationProvider::send_bulk` does NOT honor `request.idempotency_key: Option<IdempotencyKey>` either. Each call mints a fresh `bulk_id` from `Uuid::new_v4()` and stores the bulk receipt under that key. A retry with the same `idempotency_key` would create a duplicate bulk send, violating the port spec.

**Expected:**

Per `docs/ports/notifications.md:162-166` and `crates/adapters/notify/src/port.rs:1255-1258`: idempotency deduplication applies to bulk sends as well.

**Evidence:**

```rust
  crates/tools/testkit/src/notify.rs:117-134
  async fn send_bulk(&self, request: SendBulkNotification) -> Result<BulkReceipt> {
      let bulk_id = BulkId::new(format!("in-memory-bulk-{}", Uuid::new_v4()));
      let receipts: Vec<NotificationReceipt> = request
          .recipients
          .iter()
          .map(|_row| Self::make_receipt(&request.channel))
          .collect();

      let bulk_receipt = BulkReceipt {
          bulk_id: bulk_id.clone(),
          receipts,
          failed: Vec::new(),
      };

      let mut bulks = self.bulks.lock().unwrap_or_else(PoisonError::into_inner);
      bulks.insert(bulk_id, bulk_receipt.clone());
      Ok(bulk_receipt)
  }
  ```

---

### FINDING 14 (id: `TOOL-TK-014`)

- **Source:** `docs/audit_reports/findings/wave4-testkit.md`
- **Severity:** Medium
- **Area:** tools
- **Location:** `crates/tools/testkit/src/notify.rs:136-138` (`status` impl)

**Description:**

`InMemoryNotificationProvider::status` returns `DeliveryStatus::Sent` for any `receipt_id`, even for ids that were never sent. The provider should look up its own `sends` Vec (keyed by `receipt_id`) and return the actual stored status, or `NotFound` for unknown ids.

**Expected:**

Per `crates/adapters/notify/src/port.rs:1407-1411`: "Looks up the current delivery status of a previously sent notification." The lookup must consult the actual send store.

**Evidence:**

```rust
  crates/tools/testkit/src/notify.rs:136-138
  async fn status(&self, _receipt_id: NotificationReceiptId) -> Result<DeliveryStatus> {
      Ok(DeliveryStatus::Sent)
  }
  ```

---

### FINDING 16 (id: `TOOL-TK-016`)

- **Source:** `docs/audit_reports/findings/wave4-testkit.md`
- **Severity:** Medium
- **Area:** tools
- **Location:** `crates/tools/testkit/src/files.rs:60-118` (`put` impl)

**Description:**

The idempotency-key lookup at lines 64-73 falls through to a fresh insert if the file referenced by the key has been deleted between the two phases. There is no test for the "idempotency key survives a delete" edge case. A consumer that puts, deletes, then retries the same idempotency-keyed put would see a new file (not the original reference) — violating the contract "A retry with the same token returns the same `FileReference` without re-uploading."

**Expected:**

Per `docs/ports/file-storage.md:78-83`: idempotency is keyed on the `idempotency_key`, not on the underlying file's existence. The lookup must hold the original reference even after `delete`.

**Evidence:**

```rust
  crates/tools/testkit/src/files.rs:64-73
  if let Some(idempotency_key) = request.idempotency_key.as_ref() {
      let idem = self.idempotency_keys.lock();
      if let Some(existing_key) = idem.get(idempotency_key).cloned() {
          drop(idem);
          let store = self.store.lock();
          if let Some((existing_ref, _)) = store.get(&existing_key) {
              return Ok(existing_ref.clone());
          }
      }
  }
  ```
  No test exercises `put → delete → put(same idem key)`.

---

### FINDING 19 (id: `TOOL-TK-019`)

- **Source:** `docs/audit_reports/findings/wave4-testkit.md`
- **Severity:** Medium
- **Area:** tools
- **Location:** `crates/tools/testkit/src/payment.rs:162-181` (`settlement` impl)

**Description:**

`settlement` always returns an empty `Settlement` (zero totals, no lines) regardless of what charges have been minted. The module doc at lines 18-25 acknowledges this and tells consumers to "post-process the receipts directly; the in-memory adapter does not auto-link charges to settlement lines." But this makes `settlement` essentially useless for any test that wants to verify "the engine emits one `PaymentSettled` event per charged payment" — the test would need to inspect receipts manually instead of going through the port surface.

**Expected:**

Per `crates/adapters/payment/src/port.rs:1113-1120`: "Reports the settlement batch covering the requested window. The engine matches settlement lines to `PaymentReceipt` rows by `provider_payment_id` and emits `PaymentSettled` events for each newly-settled line." The adapter should construct settlement lines from its internal charges store.

**Evidence:**

```rust
  crates/tools/testkit/src/payment.rs:162-181
  async fn settlement(
      &self,
      request: SettlementRequest,
  ) -> Result<Settlement, educore_payment::errors::PaymentError> {
      let zero = match Money::new(request.currency.clone(), 0) {
          Ok(m) => m,
          Err(_) => Money::zero(request.currency.clone()),
      };
      Ok(Settlement {
          settlement_id: "in-mem-settlement-1".to_owned(),
          school_id: request.tenant.school_id,
          currency: request.currency.clone(),
          period_start: request.period_start,
          period_end: request.period_end,
          lines: Vec::new(),
          total_gross: zero.clone(),
          total_fees: zero.clone(),
          total_net: zero,
      })
  }
  ```
  The `lines: Vec::new()` is hard-coded; `self.charges` is never consulted.

---

### FINDING 20 (id: `TOOL-TK-020`)

- **Source:** `docs/audit_reports/findings/wave4-testkit.md`
- **Severity:** Medium
- **Area:** tools
- **Location:** `crates/tools/testkit/src/payment.rs:143-148` (`status` impl)

**Description:**

`status` returns `PaymentStatus::Failed { reason: "not found", code: None }` for any unknown `PaymentId`. The port trait has more nuanced status variants (`Pending`, `Authorized`, `Captured`, `Refunded`, `Voided`, `Failed`, `Disputed` per `crates/adapters/payment/src/port.rs`). For an unknown id, the correct response is a `Result::Err(PaymentError::NotFound)` rather than a synthetic `Failed` status — the real Stripe adapter returns 404 → `PaymentError::NotFound`.

**Expected:**

Per `crates/adapters/payment/src/port.rs:1097-1103` and the parallel `stripe.rs:432-466`: `status(payment_id)` returns `Err(PaymentError::NotFound)` for unknown ids, not a synthetic status payload.

**Evidence:**

```rust
  crates/tools/testkit/src/payment.rs:73-83
  fn lookup_status(&self, payment_id: &PaymentId) -> PaymentStatus {
      let charges = self.charges.lock();
      charges
          .values()
          .find(|receipt| receipt.payment_id == *payment_id)
          .map(|receipt| receipt.status.clone())
          .unwrap_or_else(|| PaymentStatus::Failed {
              reason: "not found".to_owned(),
              code: None,
          })
  }
  ```
  The test at lines 297-307 codifies the synthetic-Failed behavior as correct.

---

### FINDING 23 (id: `TOOL-TK-023`)

- **Source:** `docs/audit_reports/findings/wave4-testkit.md`
- **Severity:** Medium
- **Area:** tools
- **Location:** `crates/tools/testkit/src/sync.rs:1-53`

**Description:**

The `sync` module is a placeholder — it exposes a single `dummy_witness()` no-op function and two trivial tests. The module-level doc at lines 6-22 acknowledges "The actual `ChangeStream` and per-school `VersionCursor` table live inside the in-memory storage adapter (see `storage::InMemoryStorageAdapter` and its `watch_changes`, `apply_snapshot`, `cursor_for`, `advance_cursor` methods)." So the sync module exists only because `lib.rs:76` declares `pub mod sync;` and that declaration would otherwise fail to resolve. The sync surface is therefore inaccessible as `educore_testkit::sync::*` — consumers must reach into `educore_testkit::storage::*` instead, which means the `TestkitWorld` does not expose a sync surface.

**Expected:**

Per `docs/build-plan.md:1653-1656`: the testkit is the in-memory backplane for all sync engine integration tests. A `sync` module that is purely a placeholder contradicts the exit criterion "consumer tests use these to run domain commands without docker."

**Evidence:**

```rust
  crates/tools/testkit/src/sync.rs:1-22
  //! # In-memory sync primitives
  //!
  //! The testkit exposes a `sync` module because
  //! [`lib.rs`](crate) declared `pub mod sync;` and the test
  //! harness needs that module to resolve. The actual
  //! `ChangeStream` and per-school `VersionCursor` table live
  //! inside the in-memory storage adapter (see
  //! [`storage::InMemoryStorageAdapter`](crate::storage::InMemoryStorageAdapter)
  //! and its `watch_changes`, `apply_snapshot`, `cursor_for`,
  //! `advance_cursor` methods).
  ...
  ```
  And:
  ```rust
  crates/tools/testkit/src/sync.rs:28-37
  /// No-op witness function.
  ///
  /// Exists so the module compiles and the type system can verify
  /// the `sync` module is wired into the testkit. The actual
  /// sync primitives (`ChangeStream`, `VersionCursor`,
  /// `watch_changes`, `apply_snapshot`, `cursor_for`,
  /// `advance_cursor`) are exposed as methods on the in-memory
  /// storage adapter — see
  /// [`storage::InMemoryStorageAdapter`](crate::storage::InMemoryStorageAdapter).
  pub fn dummy_witness() {}
  ```

---

### FINDING 6 (id: `TOOL-TK-006`)

- **Source:** `docs/audit_reports/findings/wave4-testkit.md`
- **Severity:** Medium
- **Area:** tools
- **Location:** `crates/tools/testkit/src/storage.rs:342-363`

**Description:**

`InMemoryStorageAdapter::watch_changes` ignores the `ChangeFilter::since: Option<VersionCursor>` field. The trait documents `since` as an "Optional resume point; if `None`, the stream starts at the current cursor position for the school" (`crates/infra/storage/src/change_stream.rs:40-41`). With no plumbing to populate `self.inner.change_events` from outbox appends, `watch_changes` always returns an empty stream in practice, regardless of filter parameters.

**Expected:**

Per `crates/infra/storage/src/change_stream.rs:39-41` (the `since` field doc) and `docs/ports/storage.md` § 3: sync consumers pass `since: Some(cursor)` to resume from a checkpoint. The testkit must either honor `since` or document that resume is not supported.

**Evidence:**

```rust
  crates/tools/testkit/src/storage.rs:342-363
  async fn watch_changes(&self, filter: ChangeFilter) -> Result<ChangeStream> {
      use futures::stream;
      let events = self.inner.change_events.lock().clone();
      let matching: Vec<std::result::Result<ChangeEvent, DomainError>> = events
          .into_iter()
          .filter(|e| e.school_id == filter.school_id)
          .filter(|e| {
              if filter.aggregate_types.is_empty() {
                  return true;
              }
              filter.aggregate_types.iter().any(|f| match f {
                  educore_storage::change_stream::AggregateTypeFilter::Exact(n) => {
                      &e.aggregate_type == n
                  }
                  educore_storage::change_stream::AggregateTypeFilter::Any => true,
              })
          })
          .map(Ok)
          .collect();
      let s = stream::iter(matching);
      Ok(ChangeStream { inner: Box::pin(s) })
  }
  ```
  The `filter.since` field is never read; `change_events` is never populated by `outbox().append(...)`.

---

### FINDING 7 (id: `TOOL-TK-007`)

- **Source:** `docs/audit_reports/findings/wave4-testkit.md`
- **Severity:** Medium
- **Area:** tools
- **Location:** `crates/tools/testkit/src/storage.rs:365-371`

**Description:**

`InMemoryStorageAdapter::apply_snapshot` accepts a `SchoolSnapshot` and pushes its `aggregates` into a `Vec<SnapshotAggregate>` (`self.inner.snapshots`) but does not hydrate any in-memory aggregate store. A test that calls `apply_snapshot` then queries for the snapshot's aggregates by id would find nothing. The "apply" is a sink, not a hydration.

**Expected:**

Per `crates/infra/storage/src/change_stream.rs:189-202`: `SchoolSnapshot` is "a bulk snapshot of a school used for first-time client hydration" — the contract is to make the snapshot's aggregates queryable.

**Evidence:**

```rust
  crates/tools/testkit/src/storage.rs:365-371
  async fn apply_snapshot(&self, snapshot: SchoolSnapshot) -> Result<()> {
      let mut store = self.inner.snapshots.lock();
      for agg in snapshot.aggregates {
          store.push(agg);
      }
      Ok(())
  }
  ```
  No aggregate-table read accessors exist on the in-memory backend.

---

### FINDING 8 (id: `TOOL-TK-008`)

- **Source:** `docs/audit_reports/findings/wave4-testkit.md`
- **Severity:** Medium
- **Area:** tools
- **Location:** `crates/tools/testkit/src/storage.rs:61-73` (`InMemoryInner`)

**Description:**

The `_id_seq: AtomicU64` and `_next_id` method exist but are never read or incremented. The bus field on `InMemoryTransaction` (`_bus: Arc<dyn EventBus>`, line 396) is also never used — even though `commit` (lines 433-452) is the obvious site to republish to the bus. These are dead fields the build plan references as "imports model fields the trait surface doesn't exercise yet" (per PHASE-16-HANDOFF OQ #1), but they should be wired or removed.

**Expected:**

Per `AGENTS.md` § Agent Instructions: "No `#[allow(dead_code)]` or `_var` prefixes to silence the compiler. Delete unused code, wire it in, or open a follow-up issue."

**Evidence:**

```rust
  crates/tools/testkit/src/storage.rs:61-73
  pub(crate) struct InMemoryInner {
      pub(crate) outbox: Mutex<Vec<SerializedEnvelope>>,
      pub(crate) audit_log: Mutex<Vec<AuditLogEntry>>,
      pub(crate) event_log: Mutex<Vec<EventLogEntry>>,
      pub(crate) idempotency: Mutex<HashMap<IdempotencyCompositeKey, IdempotencyRecord>>,
      pub(crate) bulk_attendance: Mutex<Vec<(SchoolId, Uuid, NaiveDate, Uuid)>>,
      pub(crate) change_events: Mutex<Vec<ChangeEvent>>,
      pub(crate) cursors: Mutex<HashMap<SchoolId, VersionCursor>>,
      pub(crate) snapshots: Mutex<Vec<SnapshotAggregate>>,
      pub(crate) migrated: AtomicBool,
      pub(crate) closed: AtomicBool,
      pub(crate) _id_seq: AtomicU64,
  }
  ```
  And:
  ```rust
  crates/tools/testkit/src/storage.rs:265-268
  fn _next_id(&self) -> u64 {
      self.inner._id_seq.fetch_add(1, Ordering::Relaxed)
  }
  ```

---

### FINDING 9 (id: `TOOL-TK-009`)

- **Source:** `docs/audit_reports/findings/wave4-testkit.md`
- **Severity:** Medium
- **Area:** tools
- **Location:** `crates/tools/testkit/src/auth.rs:140-167` (`refresh` impl)

**Description:**

`InMemoryAuthProvider::refresh` mints a new `Session` with `capabilities: BTreeSet::<Capability>::new()` (empty) and `metadata: BTreeMap::new()` (empty). Per the `Session` doc at `crates/adapters/auth/src/port.rs:111-112`, `capabilities` is "the pre-computed capability set for this session" that the engine consults instead of the RBAC store. After refresh the user silently loses every granted capability, which would cascade to every subsequent command returning `Forbidden`.

**Expected:**

Per `crates/adapters/auth/src/port.rs:108-125`: `capabilities`, `roles`, and `metadata` are pre-computed at session-issuance time and must be preserved across refresh.

**Evidence:**

```rust
  crates/tools/testkit/src/auth.rs:150-161
  let new_session = Session {
      session_id: new_session_id,
      user_id: old_session.user_id,
      school_ids: old_session.school_ids.clone(),
      active_school_id: old_session.active_school_id,
      roles: old_session.roles.clone(),
      capabilities: BTreeSet::<Capability>::new(),
      mfa_satisfied: old_session.mfa_satisfied,
      issued_at: now,
      expires_at,
      metadata: BTreeMap::new(),
  };
  ```
  The `refresh_mints_new_session_with_same_school` test (lines 269-279) does not assert `capabilities` is preserved.

---

### FINDING 15 (id: `TOOL-TK-015`)

- **Source:** `docs/audit_reports/findings/wave4-testkit.md`
- **Severity:** Low
- **Area:** tools
- **Location:** `crates/tools/testkit/src/files.rs:17-23` (module-level doc) and `crates/tools/testkit/src/files.rs:90-106` (checksum implementation)

**Description:**

The module docstring says "the spec requires a content-addressable SHA-256 hex digest" but the in-memory checksum is `format!("{:x}", content.len())` — a length-derived placeholder. This is documented but the test at line 305 asserts the etag changes on overwrite (which works because length changes), so the placeholder is internally consistent. However, a test asserting two distinct content blobs of the same length produce different checksums would fail.

**Expected:**

Per `docs/ports/file-storage.md:84-87`: "The adapter computes a SHA-256 checksum on upload. The engine verifies the checksum on read." The testkit doc acknowledges the gap and points consumers to the real adapters — this is acceptable but should be tested.

**Evidence:**

```rust
  crates/tools/testkit/src/files.rs:17-23
  //! # Checksum
  //!
  //! The spec requires a content-addressable SHA-256 hex digest.
  //! The in-memory adapter uses a length-based hex placeholder
  //! (`format!("{:x}", content.len())`) so the testkit does not
  //! need to take on a SHA-256 crate dependency. Tests that need a
  //! real content-addressable hash should exercise the
  //! `LocalFileStorage` or `S3FileStorage` reference
  //! implementations instead.
  ```
  And:
  ```rust
  crates/tools/testkit/src/files.rs:92-106
  let checksum = format!("{:x}", request.content.len());
  let now = Timestamp::now();

  let reference = FileReference {
      key: request.key.clone(),
      etag: format!("\"{checksum}\""),
      ...
      checksum: Checksum::new(checksum),
  };
  ```

---

### FINDING 17 (id: `TOOL-TK-017`)

- **Source:** `docs/audit_reports/findings/wave4-testkit.md`
- **Severity:** Low
- **Area:** tools
- **Location:** `crates/tools/testkit/src/files.rs:24` (module doc) and `crates/tools/testkit/src/files.rs:29-33` (imports)

**Description:**

The module doc says "put is idempotent on `PutRequest::idempotency_key`. A retry with the same token returns the original `FileReference` without re-uploading" — and the module docstring lists `parking_lot::Mutex<HashMap<...>>` as the storage type. But the imports at line 33 use `parking_lot::Mutex` while `InMemoryFileStorage` is `#[derive(Default)]` and does not pass `parking_lot::Mutex` correctly through `derive(Default)`. The default implementation (line 54) uses `Self::default()` which calls `Mutex::default()` which is fine for parking_lot. This is correct but worth noting: `InMemoryPaymentProvider` (line 41-55) uses `#[derive(Default)]` while the `charges` and `refunds` fields contain `parking_lot::Mutex`, which works. No actual bug, but inconsistent style.

**Expected:**

Per `AGENTS.md` § Code Standards: idiomatic Rust with clear ownership. Mixing `parking_lot::Mutex` with `std::sync::Mutex` (auth.rs, notify.rs, integrations.rs all use `std::sync::Mutex`) without a documented rationale creates maintenance friction.

**Evidence:**

```rust
  crates/tools/testkit/src/files.rs:25-36
  use std::collections::HashMap;

  use async_trait::async_trait;
  use educore_core::value_objects::Timestamp;
  use educore_files::port::{
      Checksum, FileKey, FileMetadata, FileReference, FileStorage, FileStream, IdempotencyKey,
      PutRequest, SignedUrlOptions, StorageClass,
  };
  use parking_lot::Mutex;
  use tokio::sync::mpsc;

  use educore_files::errors::{FileStorageError, InfrastructureError};
  ```
  vs. `auth.rs:38-39`:
  ```rust
  use std::collections::{BTreeMap, BTreeSet, HashMap};
  use std::sync::{Mutex, PoisonError};
  ```

---

### FINDING 22 (id: `TOOL-TK-022`)

- **Source:** `docs/audit_reports/findings/wave4-testkit.md`
- **Severity:** Low
- **Area:** tools
- **Location:** `crates/tools/testkit/src/event_bus.rs:1-37`

**Description:**

The `event_bus` module is a pure re-export of `educore_event_bus::InProcessEventBus` plus a type alias `InMemoryEventBus = InProcessEventBus`. The `TestkitWorld::bus` field is typed `Arc<dyn educore_events::event_bus::EventBus>`, which means consumers can already construct an `InProcessEventBus` themselves and hand it to `InMemoryStorageAdapter::new(bus)`. The re-export and alias add a layer of indirection without adding functionality.

**Expected:**

Per `AGENTS.md` § Code Standards: "Avoid relative path dependencies outside the workspace." The re-export pattern is acceptable but the alias `InMemoryEventBus` adds no value over `educore_event_bus::InProcessEventBus` — both names refer to the same type.

**Evidence:**

```rust
  crates/tools/testkit/src/event_bus.rs:24-37
  #![forbid(unsafe_code)]
  #![deny(missing_docs)]

  pub use educore_event_bus::InProcessEventBus;

  /// Testkit-local alias for the in-process event bus.
  ///
  /// The alias exists so consumers can write
  /// `use educore_testkit::event_bus::InMemoryEventBus;` without
  /// taking a direct dep on `educore-event-bus`. The underlying
  /// type is `educore_event_bus::InProcessEventBus` (re-exported
  /// above) — see that type's rustdoc for the full MPMC /
  /// replay-log contract.
  pub type InMemoryEventBus = InProcessEventBus;
  ```

---

### FINDING 24 (id: `TOOL-TK-024`)

- **Source:** `docs/audit_reports/findings/wave4-testkit.md`
- **Severity:** Low
- **Area:** tools
- **Location:** `crates/tools/testkit/src/lib.rs:144-175` (test module)

**Description:**

The lib.rs test module contains only three trivial tests (`package_metadata_is_set`, `testkit_world_constructs_with_all_seven_ports`, `test_world_function_constructs_testkit_world`). None of them exercise the integration between ports — for example, no test verifies "after a domain command writes to the outbox, a bus subscriber receives the event" (which would have caught finding TOOL-TK-001).

**Expected:**

Per `AGENTS.md` § Validation Checklist: "At least one integration test added for new behavior." The `TestkitWorld` is the engine's pre-wired in-memory world; tests should exercise at least one cross-port flow.

**Evidence:**

```rust
  crates/tools/testkit/src/lib.rs:144-175
  #[cfg(test)]
  mod tests {
      use super::*;

      #[test]
      fn package_metadata_is_set() {
          assert_eq!(PACKAGE_NAME, "educore-testkit");
          assert!(!PACKAGE_VERSION.is_empty());
      }

      #[test]
      fn testkit_world_constructs_with_all_seven_ports() {
          let world = TestkitWorld::new();
          let _: &std::sync::Arc<storage::InMemoryStorageAdapter> = &world.storage;
          ...
      }

      #[test]
      fn test_world_function_constructs_testkit_world() {
          let _world = test_world();
      }
  }
  ```

---

### FINDING 25 (id: `TOOL-TK-025`)

- **Source:** `docs/audit_reports/findings/wave4-testkit.md`
- **Severity:** Low
- **Area:** tools
- **Location:** `crates/tools/testkit/src/storage.rs:26`

**Description:**

`storage.rs` has a module-level `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]` at line 26, which silences clippy for ALL production code in the file (not just the test module). The other testkit modules (auth.rs, notify.rs, files.rs, payment.rs, etc.) correctly scope the allow to `#[allow(...)]` on the `mod tests` block. The broad module-level allow hides any future production-code violations from clippy.

**Expected:**

Per `AGENTS.md` § Code Standards: "No `unwrap()` or `expect()` in production paths." Per AGENTS.md § Agent Instructions: "No `#[allow(dead_code)]` or `_var` prefixes to silence the compiler."

**Evidence:**

```rust
  crates/tools/testkit/src/storage.rs:26
  #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
  ```
  Compare with `files.rs:217-223`:
  ```rust
  #[cfg(test)]
  #[allow(
      clippy::unwrap_used,
      clippy::expect_used,
      clippy::panic,
      clippy::dbg_macro
  )]
  mod tests { ... }
  ```

---

### FINDING 26 (id: `TOOL-TK-026`)

- **Source:** `docs/audit_reports/findings/wave4-testkit.md`
- **Severity:** Low
- **Area:** tools
- **Location:** `crates/tools/testkit/src/integrations.rs:135-142`

**Description:**

The `invoke` impl uses `match self.invocations.lock() { Ok(mut g) => g.push(request), Err(_) => return Err(IntegrationError::Infrastructure(...)) }` — manual lock-error handling. The same crate uses `parking_lot::Mutex` in storage.rs/files.rs/payment.rs (which never poisons) and `std::sync::Mutex` with `unwrap_or_else(PoisonError::into_inner)` in auth.rs/notify.rs. The integration gateway uses neither idiom: it maps poison to a domain error directly. Three inconsistent lock error-handling strategies across the testkit's in-memory backends.

**Expected:**

Per `AGENTS.md` § Code Standards: idiomatic Rust; one consistent style per crate. The choice between `parking_lot::Mutex` and `std::sync::Mutex` should be deliberate, and the error-handling pattern should match across the same crate's modules.

**Evidence:**

```rust
  crates/tools/testkit/src/integrations.rs:135-142
  async fn invoke(&self, request: IntegrationRequest) -> IntegrationResult<IntegrationResponse> {
      match self.invocations.lock() {
          Ok(mut g) => g.push(request),
          Err(_) => {
              return Err(IntegrationError::Infrastructure(Box::new(
                  std::io::Error::other("InMemoryIntegrationGateway: invocations mutex poisoned"),
              )));
          }
      }
      Ok(IntegrationResponse { ... })
  }
  ```

---

### FINDING 27 (id: `TOOL-TK-027`)

- **Source:** `docs/audit_reports/findings/wave4-testkit.md`
- **Severity:** Low
- **Area:** tools
- **Location:** `crates/tools/testkit/src/auth.rs:202-292` (`mod tests`) and `crates/tools/testkit/src/notify.rs:145-307` (`mod tests`)

**Description:**

Tests for `auth` and `notify` use `futures::executor::block_on` (their own `block_on` helper, defined inside each test module). Tests for `storage` and `files` use `tokio::test` or `tokio::runtime::Runtime::new().unwrap()` + `rt.block_on`. Two different async-execution styles in the same crate.

**Expected:**

Per `AGENTS.md` § Code Standards: idiomatic Rust; consistent test style. The `tokio::test` pattern (used in `event_bus.rs`, `files.rs`, `integrations.rs`) is the workspace convention (per `PHASE-16-HANDOFF.md` test counts). `auth.rs`/`notify.rs`/`payment.rs` deviate.

**Evidence:**

```rust
  crates/tools/testkit/src/auth.rs:222-225
  fn block_on<F: std::future::Future>(future: F) -> F::Output {
      futures::executor::block_on(future)
  }
  ```
  vs.:
  ```rust
  crates/tools/testkit/src/event_bus.rs:80-82
  #[tokio::test]
  async fn publish_and_subscribe_round_trip_through_alias() { ... }
  ```

---

### FINDING 28 (id: `TOOL-TK-028`)

- **Source:** `docs/audit_reports/findings/wave4-testkit.md`
- **Severity:** Low
- **Area:** tools
- **Location:** `crates/tools/testkit/src/payment.rs:333-340` and `crates/tools/testkit/src/payment.rs:343-348`

**Description:**

The test `settlement_returns_empty_batch_in_requested_currency` (lines 333-340) hard-codes the expected `settlement_id` as `"in-mem-settlement-1"`. But `peek_id` is called by `charge` and `refund` on every invocation, so by the time `settlement` is called in a real test scenario the counter has advanced. The hard-coded id would be wrong in any test that runs after a charge or refund. The test is only correct in isolation (no prior charges).

**Expected:**

Per `AGENTS.md` § Agent Instructions: tests must validate real-world scenarios. A test that asserts a value only because the test runs in isolation is fragile.

**Evidence:**

```rust
  crates/tools/testkit/src/payment.rs:333-340
  let settlement = futures::executor::block_on(provider.settlement(req)).unwrap();
  assert_eq!(settlement.settlement_id, "in-mem-settlement-1");
  ```
  The `peek_id` call site:
  ```rust
  crates/tools/testkit/src/payment.rs:171
  settlement_id: "in-mem-settlement-1".to_owned(),
  ```
  (hard-coded; no id_seq involvement in settlement).

---


## CLI + SDK (tools) (target id prefix: `CLI-SDK`)

**Path:** `crates/tools/cli/ + crates/tools/sdk/`  
**Total findings:** 22 (7 critical, 4 high, 9 medium, 2 low)


### FINDING 1 (id: `CLI-SDK-001`)

- **Source:** `docs/audit_reports/findings/wave4-cli-sdk.md`
- **Severity:** Critical
- **Area:** tools-cli-sdk
- **Location:** `crates/tools/sdk/src/engine.rs:41-149`

**Description:**

`docs/library-docs.md:18` shows `Engine::builder()` as the canonical construction entry point, and `crates/tools/sdk/Cargo.toml:8` advertises `Engine::builder` in the package description. Neither exists. The `Engine` impl block (lines 41-149) only defines `test_world()` + 9 port accessors + 4 facade handles. Consumers following the library-docs.md sample will fail to compile on `Engine::builder()`.

**Expected:**

`docs/library-docs.md:11-22` (Construction section) — `let engine = Engine::builder()...build().await?;`

**Evidence:**

```rust
  // crates/tools/sdk/src/engine.rs:41-148
  impl Engine {
      /// Constructs a fresh `Engine` with all 7 ports wired to the
      /// in-memory testkit impls and the default `InProcessEventBus`.
      /// Convenience for consumer tests and dogfooding.
      #[must_use]
      pub fn test_world() -> Self { ... }

      /// Returns a reference to the storage adapter.
      #[must_use]
      pub fn storage(&self) -> &Arc<dyn StorageAdapter> { ... }
      // ... 8 more accessors, NO `builder()` factory ...
  }
  ```

---

### FINDING 2 (id: `CLI-SDK-002`)

- **Source:** `docs/audit_reports/findings/wave4-cli-sdk.md`
- **Severity:** Critical
- **Area:** tools-cli-sdk
- **Location:** `crates/tools/sdk/src/engine.rs:258-289`

**Description:**

`EngineBuilder::build()` is declared `pub fn build(self) -> Result<Engine, SdkError>` (synchronous, no `async`). `docs/library-docs.md:22` shows the consumer sample ending with `.build().await?;`. Awaiting a non-async function is a compile error. The library-docs sample is non-functional as written.

**Expected:**

`docs/library-docs.md:22` — `.build().await?;`

**Evidence:**

```rust
  // crates/tools/sdk/src/engine.rs:258-289
  /// Builds the `Engine`. Returns `Err(SdkError::MissingPort)`
  /// if any required port is not set.
  pub fn build(self) -> Result<Engine, SdkError> {
      let storage = self.storage.ok_or(SdkError::MissingPort("storage"))?;
      let auth = self.auth.ok_or(SdkError::MissingPort("auth"))?;
      // ...
  }
  ```

---

### FINDING 3 (id: `CLI-SDK-003`)

- **Source:** `docs/audit_reports/findings/wave4-cli-sdk.md`
- **Severity:** Critical
- **Area:** tools-cli-sdk
- **Location:** `crates/tools/sdk/src/engine.rs:125-149` (Engine impl) vs `docs/library-docs.md:179-189`

**Description:**

`docs/library-docs.md:179-189` (Common Workflows) advertises 8 high-level accessors that do not exist on `Engine`: `engine.students()`, `engine.students().admit(cmd)`, `engine.students().promote(cmd)`, `engine.attendance().mark(cmd)`, `engine.assessment().enter_marks(cmd)`, `engine.assessment().publish_result(cmd)`, `engine.fees().generate_invoice(cmd)`, `engine.fees().record_payment(cmd)`, `engine.hr().generate_payroll(cmd)`. None of these methods are defined. The Engine exposes only `storage/auth/notify/payment/files/integrations/bus/clock/id_gen/admission/attendance/payment_svc/notify_svc` (lines 71-149). None of the documented high-level workflows are reachable.

**Expected:**

`docs/library-docs.md:179-189` (Common Workflows section).

**Evidence:**

```rust
  // crates/tools/sdk/src/engine.rs:125-149
      /// Returns a handle to the admission facade.
      #[must_use]
      pub fn admission(&self) -> AdmissionService<'_> { ... }
      /// Returns a handle to the attendance facade.
      #[must_use]
      pub fn attendance(&self) -> AttendanceService<'_> { ... }
      /// Returns a handle to the payment facade.
      #[must_use]
      pub fn payment_svc(&self) -> PaymentService<'_> { ... }
      /// Returns a handle to the notification facade.
      #[must_use]
      pub fn notify_svc(&self) -> NotificationService<'_> { ... }
      // NO `students()`, NO `fees()`, NO `assessment()`, NO `hr()`,
      // NO `rbac()`, NO `events()`
  ```

---

### FINDING 4 (id: `CLI-SDK-004`)

- **Source:** `docs/audit_reports/findings/wave4-cli-sdk.md`
- **Severity:** Critical
- **Area:** tools-cli-sdk
- **Location:** `crates/tools/sdk/src/facade.rs:11-31`

**Description:**

`AdmissionService` is documented in `library-docs.md` (Common Workflows § `engine.students().admit(cmd)`) as the surface for admitting students. The actual `AdmissionService` impl only exposes `storage()` (line 29) which returns a `&Arc<dyn StorageAdapter>`. There is no `admit(cmd)` method, no `promote(cmd)` method, no academic command surface. The docstring at lines 22-27 admits this as a stub ("this facade is a thin re-export of the storage adapter for now"). Consumers cannot admit a student through the SDK.

**Expected:**

`docs/library-docs.md:181` — `engine.students().admit(cmd).await?`

**Evidence:**

```rust
  // crates/tools/sdk/src/facade.rs:11-31
  pub struct AdmissionService<'a> {
      engine: &'a Engine,
  }

  impl<'a> AdmissionService<'a> {
      /// Constructs a new admission service bound to `engine`.
      #[must_use]
      pub fn new(engine: &'a Engine) -> Self { ... }

      /// Admits a student. The full command flow lives in the
      /// academic domain crate; this facade is a thin
      /// re-export of the storage adapter for now.
      /// ...
      #[must_use]
      pub fn storage(&self) -> &std::sync::Arc<dyn educore_storage::StorageAdapter> {
          self.engine.storage()
      }
  }
  ```

---

### FINDING 5 (id: `CLI-SDK-005`)

- **Source:** `docs/audit_reports/findings/wave4-cli-sdk.md`
- **Severity:** Critical
- **Area:** tools-cli-sdk
- **Location:** `crates/tools/cli/src/commands.rs:23-53` (fn admit)

**Description:**

The `admit` CLI command takes `--first`, `--last`, `--class`, `--section` as required args (per `lib.rs:30-43`) but never persists a student. The handler only parses the school UUID (line 35), generates two synthetic UUIDs (`student_id`, `correlation_id`), and prints them in a JSON envelope. `class_id` and `section_id` are parsed from the CLI args (lines 36-37) then dropped without ever being used. No call to `educore_academic::services::admit_student` or any storage insert. The handoff doc (`PHASE-16-HANDOFF.md` "What's wired and working" § `educore-cli`) describes this as "academic admission" — the runtime behavior is a JSON echo.

**Expected:**

`docs/build-plan.md:1664` — "`educore-cli`: a sample binary demonstrating daily operations (admit a student, mark attendance, record a payment)"

**Evidence:**

```rust
  // crates/tools/cli/src/commands.rs:23-53
  pub async fn admit(
      school: String, first: String, last: String, class: String, section: String,
  ) -> Result<()> {
      let _world = test_world();
      let school_id = parse_school(&school)?;
      let class_id = parse_uuid(&class, "class")?;   // parsed, dropped
      let section_id = parse_uuid(&section, "section")?;  // parsed, dropped
      let g = SystemIdGen;
      let user = g.next_user_id();
      let corr = g.next_correlation_id();
      let _ctx = TenantContext::for_user(school_id, user, corr, UserType::SchoolAdmin);
      let student_id = g.next_uuid();  // synthetic
      let out = serde_json::json!({
          "school_id": school_id.as_uuid().to_string(),
          "student_id": student_id.to_string(),
          "first_name": first, "last_name": last,
          "class_id": class_id.to_string(), "section_id": section_id.to_string(),
          // ...
      });
      tracing::info!("{}", serde_json::to_string_pretty(&out)?);
      Ok(())
  }
  ```

---

### FINDING 6 (id: `CLI-SDK-006`)

- **Source:** `docs/audit_reports/findings/wave4-cli-sdk.md`
- **Severity:** Critical
- **Area:** tools-cli-sdk
- **Location:** `docs/library-docs.md:223-230` vs workspace tree

**Description:**

`docs/library-docs.md:223-230` (Sample Programs section) states: "A complete `examples/admit_and_enroll.rs` is provided in the workspace that..." and proceeds to list 7 workflow steps (admits, enrolls, attendance, marks, fees, pays, prints). No `examples/` directory exists at the workspace root or under `crates/educore/`. Consumers searching for the canonical example find nothing.

**Expected:**

`docs/library-docs.md:223-230` — Sample Programs section promises `examples/admit_and_enroll.rs`.

**Evidence:**

```text
  $ ls examples/ 2>&1
  ls: cannot access 'examples/': No such file or directory
  $ find . -name "admit_and_enroll.rs" -not -path "./target/*" -not -path "./schoolify/*"
  (no output)
  ```

---

### FINDING 7 (id: `CLI-SDK-007`)

- **Source:** `docs/audit_reports/findings/wave4-cli-sdk.md`
- **Severity:** Critical
- **Area:** tools-cli-sdk
- **Location:** `docs/library-docs.md:103-119, 124-132, 215-221` vs `crates/tools/sdk/src/engine.rs:71-149`

**Description:**

Three accessor methods are documented in library-docs.md but absent on Engine: `engine.auth()` is documented (line 104) but the actual `auth()` method (engine.rs:77) returns `&Arc<dyn AuthProvider>` not a session; `engine.events().subscribe::<T>()` is documented (lines 124-132) but `Engine` has no `events()` method; `engine.rbac().has_capability(...)` is documented (lines 215-221) but `Engine` has no `rbac()` method. None of these 3 accessor paths compile.

**Expected:**

`docs/library-docs.md:103-119` (Subscribing to Events), `docs/library-docs.md:215-221` (Capability Check).

**Evidence:**

```rust
  // crates/tools/sdk/src/engine.rs:71-149 -- the complete accessor set
      pub fn storage(&self) -> &Arc<dyn StorageAdapter> { ... }
      pub fn auth(&self) -> &Arc<dyn AuthProvider> { ... }
      pub fn notify(&self) -> &Arc<dyn NotificationProvider> { ... }
      pub fn payment(&self) -> &Arc<dyn PaymentProvider> { ... }
      pub fn files(&self) -> &Arc<dyn FileStorage> { ... }
      pub fn integrations(&self) -> &Arc<dyn IntegrationGateway> { ... }
      pub fn bus(&self) -> &Arc<dyn EventBus> { ... }
      pub fn clock(&self) -> &Arc<dyn Clock> { ... }
      pub fn id_gen(&self) -> &Arc<dyn IdGenerator> { ... }
      pub fn admission(&self) -> AdmissionService<'_> { ... }
      pub fn attendance(&self) -> AttendanceService<'_> { ... }
      pub fn payment_svc(&self) -> PaymentService<'_> { ... }
      pub fn notify_svc(&self) -> NotificationService<'_> { ... }
      // NO `events()`, NO `rbac()`, NO `students()`, NO `fees()`,
      // NO `assessment()`, NO `hr()`
  ```

---

### FINDING 10 (id: `CLI-SDK-010`)

- **Source:** `docs/audit_reports/findings/wave4-cli-sdk.md`
- **Severity:** High
- **Area:** tools-cli-sdk
- **Location:** `crates/tools/cli/src/commands.rs:25-27, 60-62, 137-139`

**Description:**

All three CLI handlers call `let _world = test_world();` (or `let world = test_world();`) — the in-memory testkit backend. The CLI is documented in `docs/build-plan.md:1664` as "a sample binary demonstrating daily operations". But every CLI invocation spawns a fresh in-process `TestkitWorld` that dies when the binary exits. There is no persistence, no shared state between invocations, no config file, no daemon mode. The CLI cannot actually be used to "admit a student" in any operational sense.

**Expected:**

`docs/build-plan.md:1664` — "demonstrating daily operations (admit a student, mark attendance, record a payment) for developer ergonomics and dogfooding."

**Evidence:**

```rust
  // crates/tools/cli/src/commands.rs:25-27 (admit)
      let _world = test_world();
      let school_id = parse_school(&school)?;
      let class_id = parse_uuid(&class, "class")?;
  // crates/tools/cli/src/commands.rs:60-62 (attendance)
      let world = test_world();
      let school_id = parse_school(&school)?;
      let student_id = parse_uuid(&student, "student")?;
  // crates/tools/cli/src/commands.rs:137-139 (payment)
      let world = test_world();
      let school_id = parse_school(&school)?;
  ```

---

### FINDING 11 (id: `CLI-SDK-011`)

- **Source:** `docs/audit_reports/findings/wave4-cli-sdk.md`
- **Severity:** High
- **Area:** tools-cli-sdk
- **Location:** `crates/educore/src/lib.rs:75-83` (prelude module)

**Description:**

`docs/library-docs.md:8` shows the consumer sample as `use educore::prelude::*;`. The umbrella's prelude module (lines 75-83) re-exports only `educore_core`, `educore_events`, `educore_operations`, `educore_platform`, `educore_rbac`, `educore_sdk`, `educore_settings`. It does NOT flatten `Engine`, `EngineBuilder`, or any of the 4 facade services. Even though `educore_sdk` is re-exported as a module, consumers using `use educore::prelude::*;` must still write `educore::sdk::Engine` or `educore::prelude::educore_sdk::Engine` to reach the SDK surface. The promised "prelude" does not provide the documented ergonomic surface.

**Expected:**

`docs/library-docs.md:8` — `use educore::prelude::*;`

**Evidence:**

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
  ```

---

### FINDING 8 (id: `CLI-SDK-008`)

- **Source:** `docs/audit_reports/findings/wave4-cli-sdk.md`
- **Severity:** High
- **Area:** tools-cli-sdk
- **Location:** `crates/tools/sdk/Cargo.toml:15-37` (dependencies)

**Description:**

The SDK Cargo.toml declares 14 dependencies that are not referenced anywhere in `crates/tools/sdk/src/`: `educore-platform` (no import), `educore-rbac` (no import), `educore-academic` (no import — `AdmissionService` does NOT call academic), `educore-assessment` (no import), `educore-attendance` (no import), `educore-finance` (no import), `educore-hr` (no import), `educore-event-bus` (no import — only the cross-cutting `educore_events::event_bus::EventBus` trait is used), `async-trait` (no `#[async_trait]` macro used), `tracing` (no `tracing::*` macro used), `parking_lot` (no `parking_lot::*` import), `bytes` (no `bytes::*` import), `anyhow` (no `anyhow::*` macro used). Bloat in the dependency closure.

**Expected:**

`docs/code-standards.md` § "Engine Rules" — minimal dependency surface.

**Evidence:**

```bash
  $ grep -h "use " crates/tools/sdk/src/*.rs
  use std::sync::Arc;
  use educore_auth::port::AuthProvider;
  use educore_core::clock::{Clock, IdGenerator, SystemClock, SystemIdGen};
  use educore_events::event_bus::EventBus;
  use educore_files::port::FileStorage;
  use educore_integrations::port::IntegrationGateway;
  use educore_notify::port::NotificationProvider;
  use educore_payment::port::PaymentProvider;
  use educore_storage::StorageAdapter;
  use educore_testkit::TestkitWorld;
  use crate::errors::SdkError;
  use crate::facade::{AdmissionService, AttendanceService, NotificationService, PaymentService};
  use thiserror::Error;
  use educore_core::tenant::TenantContext;
  use crate::engine::Engine;
  use crate::errors::SdkError;
  ```

---

### FINDING 9 (id: `CLI-SDK-009`)

- **Source:** `docs/audit_reports/findings/wave4-cli-sdk.md`
- **Severity:** High
- **Area:** tools-cli-sdk
- **Location:** `crates/tools/cli/src/commands.rs:73-89` (StudentAttendanceRow construction)

**Description:**

The `attendance` CLI command accepts `--student <uuid>` (line 88 in lib.rs) and writes a `StudentAttendanceRow`, but `student_record_id` (line 85), `class_id` (line 86), and `section_id` (line 87) are populated from random `g.next_uuid()` calls rather than the student's actual record/class/section. The persisted row has no referential integrity to the student. Any downstream read or query by class/section will miss these rows.

**Expected:**

`docs/build-plan.md:1664` — "demonstrating daily operations (admit a student, mark attendance, record a payment)". A real attendance row would resolve `student_record_id`/`class_id`/`section_id` from the student lookup, not fabricate them.

**Evidence:**

```rust
  // crates/tools/cli/src/commands.rs:73-89
  let row = StudentAttendanceRow {
      school_id,
      id: g.next_uuid(),
      student_id,
      student_record_id: g.next_uuid(),  // <-- synthetic, not resolved from student_id
      class_id: g.next_uuid(),           // <-- synthetic
      section_id: g.next_uuid(),         // <-- synthetic
      attendance_date: date,
      // ...
  };
  ```

---

### FINDING 12 (id: `CLI-SDK-012`)

- **Source:** `docs/audit_reports/findings/wave4-cli-sdk.md`
- **Severity:** Medium
- **Area:** tools-cli-sdk
- **Location:** `crates/tools/sdk/src/facade.rs:11-31` (AdmissionService)

**Description:**

`AdmissionService` is the only facade service that does not have a domain-specific method. The docstring at lines 22-27 admits "the full command flow lives in the academic domain crate; this facade is a thin re-export of the storage adapter for now". The `AdmissionService` is effectively a typedef for `&Arc<dyn StorageAdapter>` — it does not implement the `AdmitStudentCommand` flow that the academic domain crate provides (`crates/domains/academic/src/services.rs:90`). Consumers wiring the SDK's `AdmissionService` get a storage handle, not an admission workflow.

**Expected:**

`docs/library-docs.md:179-189` — Common Workflows section promises `engine.students().admit(cmd).await?`.

**Evidence:**

```rust
  // crates/tools/sdk/src/facade.rs:11-31
  impl<'a> AdmissionService<'a> {
      /// Admits a student. The full command flow lives in the
      /// academic domain crate; this facade is a thin
      /// re-export of the storage adapter for now.
      /// ...
      #[must_use]
      pub fn storage(&self) -> &std::sync::Arc<dyn educore_storage::StorageAdapter> {
          self.engine.storage()
      }
  }
  ```

---

### FINDING 13 (id: `CLI-SDK-013`)

- **Source:** `docs/audit_reports/findings/wave4-cli-sdk.md`
- **Severity:** Medium
- **Area:** tools-cli-sdk
- **Location:** `crates/tools/sdk/src/facade.rs:53-72, 86-99, 119-128`

**Description:**

All three facade methods (`AttendanceService::mark_bulk`, `PaymentService::charge`, `NotificationService::send`) map the underlying port error to `SdkError::Facade { service: "<name>", message: e.to_string() }`. The `to_string()` call discards the structured error variant. Consumers cannot match on `DomainError::NotFound`, `DomainError::Conflict`, `DomainError::Validation`, etc. — they only see a string. Per `docs/library-docs.md:191-201` (Error Handling section), consumers expect to pattern-match `DomainError` variants; the SDK facade breaks that contract.

**Expected:**

`docs/library-docs.md:191-201` — Error Handling section demonstrates `match ... Err(DomainError::Validation { ... }) ...`.

**Evidence:**

```rust
  // crates/tools/sdk/src/facade.rs:53-72 (mark_bulk)
      pub async fn mark_bulk(
          &self,
          ctx: &TenantContext,
          rows: &[...],
      ) -> Result<(), SdkError> {
          self.engine
              .storage()
              .bulk_insert_student_attendances(ctx, rows)
              .await
              .map_err(|e| SdkError::Facade {
                  service: "AttendanceService",
                  message: e.to_string(),  // discards structured error
              })
      }
  // crates/tools/sdk/src/facade.rs:86-99 (charge)
              .map_err(|e| SdkError::Facade {
                  service: "PaymentService",
                  message: e.to_string(),
              })
  // crates/tools/sdk/src/facade.rs:119-128 (send)
              .map_err(|e| SdkError::Facade {
                  service: "NotificationService",
                  message: e.to_string(),
              })
  ```

---

### FINDING 14 (id: `CLI-SDK-014`)

- **Source:** `docs/audit_reports/findings/wave4-cli-sdk.md`
- **Severity:** Medium
- **Area:** tools-cli-sdk
- **Location:** `crates/tools/cli/src/commands.rs:160-164` (payment handler)

**Description:**

The `payment` CLI handler takes `--invoice <uuid>` (line 64) and wraps it as `CustomerRef::External(CustomerId::new(invoice))` (line 164). The `invoice` arg is parsed as a string with no UUID validation — `CustomerId::new` accepts any string. There is no validation that the invoice exists, no lookup against `finance::Invoice`, no FK constraint enforcement. The CLI happily charges a payment against a non-existent invoice id.

**Expected:**

`docs/ports/payments.md` — payment port contract requires invoice reference to be a valid, existing entity.

**Evidence:**

```rust
  // crates/tools/cli/src/commands.rs:160-164
      let req = ChargeRequest::new(
          ctx,
          money,
          payment_method,
          CustomerRef::External(CustomerId::new(invoice)),  // string parsed, not validated
          g.next_idempotency_key(),
          g.next_correlation_id(),
      );
  ```

---

### FINDING 15 (id: `CLI-SDK-015`)

- **Source:** `docs/audit_reports/findings/wave4-cli-sdk.md`
- **Severity:** Medium
- **Area:** tools-cli-sdk
- **Location:** `crates/tools/cli/src/commands.rs:109-126` (attendance handler)

**Description:**

After `world.storage.bulk_insert_student_attendances(...)` (lines 109-112), the handler prints the synthetic `row.id` (line 119) without reading back from the storage adapter to verify the row was actually persisted. If the insert silently failed (e.g. storage adapter returns `Ok(())` without writing), the CLI exits with success and reports a row id that doesn't exist. There is no read-back assertion.

**Expected:**

Standard CLI hygiene for state-mutating operations — verify the write by re-reading or by asserting on a side effect.

**Evidence:**

```rust
  // crates/tools/cli/src/commands.rs:108-126
      world
          .storage
          .bulk_insert_student_attendances(&ctx, std::slice::from_ref(&row))
          .await
          .map_err(|e| anyhow!("attendance insert failed: {e}"))?;

      let out = serde_json::json!({
          "row_id": row.id.to_string(),  // never read back to confirm persistence
          "school_id": school_id.as_uuid().to_string(),
          "student_id": student_id.to_string(),
          "attendance_date": date.to_string(),
          "attendance_type": attendance_type,
          "is_absent": is_absent,
      });
      tracing::info!("{}", serde_json::to_string_pretty(&out)?);
      Ok(())
  ```

---

### FINDING 16 (id: `CLI-SDK-016`)

- **Source:** `docs/audit_reports/findings/wave4-cli-sdk.md`
- **Severity:** Medium
- **Area:** tools-cli-sdk
- **Location:** `crates/educore/tests/consumer_e2e.rs:65-70` (admit section)

**Description:**

The test function is named `consumer_e2e_admission_attendance_payment_notify_chain`, and the section markers (lines 65-70) declare an "admit section" owned by the E.4 subagent. The section body is: `let _storage = engine.admission().storage(); let student_id = g.next_uuid();`. No admission occurs — the student_id is synthetic and `_storage` is unused. The test claims to verify the full admission workflow but the admit step is a no-op. The remaining 3 steps (attendance + payment + notify) run successfully against an engine that has no admitted students.

**Expected:**

`docs/build-plan.md:1668-1670` — Phase 16 task #5: "A consumer-facing integration test in `crates/educore/tests/consumer_e2e.rs` that uses the SDK + testkit to run a full admission workflow without docker."

**Evidence:**

```rust
  // crates/educore/tests/consumer_e2e.rs:65-70
      // === admit section begin (owner: E.4) ===
      let _storage = engine.admission().storage();
      let student_id = g.next_uuid();
      // === admit section end ===
  ```

---

### FINDING 17 (id: `CLI-SDK-017`)

- **Source:** `docs/audit_reports/findings/wave4-cli-sdk.md`
- **Severity:** Medium
- **Area:** tools-cli-sdk
- **Location:** `crates/tools/sdk/README.md` (missing) vs `crates/tools/sdk/` (directory listing)

**Description:**

The SDK crate has no `README.md`. `crates/tools/cli/README.md` exists but is 1 paragraph that doesn't match what `lib.rs:1-9` advertises (the lib claims 3 subcommands but the README says "starting the runtime, applying migrations, running scheduled jobs, and draining the outbox" — capabilities that don't exist in the binary). Consumers navigating to the SDK crate find zero onboarding material.

**Expected:**

Per AGENTS.md Crate Layout convention — each crate ships with a README describing its purpose and entry point.

**Evidence:**

```bash
  $ ls crates/tools/sdk/
  Cargo.toml  src        # no README.md
  $ cat crates/tools/cli/README.md
  # educore-cli
  The cli crate is a sample binary that demonstrates how a consumer
  wires the Educore for daily operations — starting the runtime,
  applying migrations, running scheduled jobs, and draining the outbox.
  ```
  The CLI lib.rs advertises 3 subcommands (`admit`, `attendance`, `payment`) — the README mentions none of them and instead describes 4 unrelated operations.

---

### FINDING 18 (id: `CLI-SDK-018`)

- **Source:** `docs/audit_reports/findings/wave4-cli-sdk.md`
- **Severity:** Medium
- **Area:** tools-cli-sdk
- **Location:** `docs/library-docs.md:11-22` vs `crates/tools/sdk/src/engine.rs:244-256`

**Description:**

The library-docs sample shows `Engine::builder().clock(SystemClock::new()).id_gen(UuidV7Generator::new())`. The actual builder at lines 244-256 expects `clock(Arc<dyn Clock>)` (so `Arc::new(SystemClock)`, not `SystemClock::new()`) and `id_gen(Arc<dyn IdGenerator>)` (so `Arc::new(SystemIdGen)`, not `UuidV7Generator::new()` — that type doesn't exist in the engine; the only impl is `SystemIdGen` at `crates/tools/sdk/src/engine.rs:65`).

**Expected:**

`docs/library-docs.md:18-19` — Construction section.

**Evidence:**

```rust
  // crates/tools/sdk/src/engine.rs:244-256
      pub fn clock(mut self, clock: Arc<dyn Clock>) -> Self {
          self.clock = Some(clock);
          self
      }

      pub fn id_gen(mut self, id_gen: Arc<dyn IdGenerator>) -> Self {
          self.id_gen = Some(id_gen);
          self
      }
  ```

---

### FINDING 19 (id: `CLI-SDK-019`)

- **Source:** `docs/audit_reports/findings/wave4-cli-sdk.md`
- **Severity:** Medium
- **Area:** tools-cli-sdk
- **Location:** `docs/library-docs.md:14-15` vs `crates/adapters/event-bus/src/in_process.rs` (InProcessEventBus type)

**Description:**

`docs/library-docs.md:15` shows `.event_bus(InProcessBus::new())`. The actual type is `InProcessEventBus` (not `InProcessBus`) and lives in the `educore_event_bus` crate, not the umbrella `educore` crate. Consumers following the sample will fail to compile on `InProcessBus`.

**Expected:**

`docs/library-docs.md:14-15` — Construction section.

**Evidence:**

```rust
  // docs/library-docs.md:13-15
      .event_bus(InProcessBus::new())   // <-- InProcessBus doesn't exist
  // crates/adapters/event-bus/README.md:43-46
  use educore_event_bus::InProcessEventBus;
  let bus: Arc<dyn EventBus> = Arc::new(InProcessEventBus::new());
  ```

---

### FINDING 20 (id: `CLI-SDK-020`)

- **Source:** `docs/audit_reports/findings/wave4-cli-sdk.md`
- **Severity:** Medium
- **Area:** tools-cli-sdk
- **Location:** `docs/library-docs.md:124-132` vs `crates/cross-cutting/events/src/event_bus.rs:48`

**Description:**

`docs/library-docs.md:128-132` shows `engine.events().subscribe::<StudentAdmitted>().await?`. The actual `EventBus::subscribe` signature is `async fn subscribe(&self, options: SubscribeOptions) -> Result<Box<dyn EventSubscription>>` (no generic type parameter; takes `SubscribeOptions`). The library-docs sample uses the wrong call shape (turbofish on a non-generic method) AND calls a non-existent `engine.events()` accessor (see CLI-SDK-007).

**Expected:**

`docs/library-docs.md:124-132` (Subscribing to Events section).

**Evidence:**

```rust
  // crates/cross-cutting/events/src/event_bus.rs:35-48
  pub trait EventBus: Send + Sync + fmt::Debug {
      // ...
      async fn subscribe(&self, options: SubscribeOptions) -> Result<Box<dyn EventSubscription>>;
  }
  ```

---

### FINDING 21 (id: `CLI-SDK-021`)

- **Source:** `docs/audit_reports/findings/wave4-cli-sdk.md`
- **Severity:** Low
- **Area:** tools-cli-sdk
- **Location:** `crates/tools/sdk/src/errors.rs:18-21`

**Description:**

The `SdkError` enum declares an `Engine(String)` variant (lines 18-21) but `grep -rn 'SdkError::Engine' crates/tools/sdk/src/` returns no matches — the variant is never constructed by any SDK method. The three facade methods (`mark_bulk`, `charge`, `send`) all use `SdkError::Facade { service, message }` instead. The `Engine` variant is dead code in the public error enum.

**Expected:**

`crates/tools/sdk/src/errors.rs` (current public surface) — variant should be reachable or removed.

**Evidence:**

```rust
  // crates/tools/sdk/src/errors.rs:5-21
  #[derive(Debug, Error)]
  pub enum SdkError {
      /// A required port was not provided to the builder.
      #[error("missing required port: {0}")]
      MissingPort(&'static str),

      /// A facade method delegation failed.
      #[error("facade error in {service}: {message}")]
      Facade { service: &'static str, message: String },

      /// The underlying engine returned an error.
      #[error("engine error: {0}")]
      Engine(String),  // never constructed in src/
  }
  ```

---

### FINDING 22 (id: `CLI-SDK-022`)

- **Source:** `docs/audit_reports/findings/wave4-cli-sdk.md`
- **Severity:** Low
- **Area:** tools-cli-sdk
- **Location:** `crates/tools/cli/src/lib.rs:8`

**Description:**

`crates/tools/cli/src/lib.rs:8` exports `pub use commands::{admit, attendance, dispatch, payment};` — making the three handler functions `pub` at the crate root. The lib is only consumed as a binary (`main.rs` only uses `dispatch`). Public re-export of internal handlers leaks the implementation surface; consumers wiring the CLI as a library could call `admit`/`attendance`/`payment` directly, bypassing the `Cli` parser and the `Command` enum.

**Expected:**

`crates/tools/cli/Cargo.toml:13` — `[[bin]]` declares this as a binary crate, not a library; library consumers should not be importing its internals.

**Evidence:**

```rust
  // crates/tools/cli/src/lib.rs:1-8
  //! ...
  #![forbid(unsafe_code)]
  #![deny(missing_docs)]

  pub mod commands;
  pub use commands::{admit, attendance, dispatch, payment};
  ```

---


## Umbrella (target id prefix: `UMB`)

**Path:** `crates/educore/`  
**Total findings:** 18 (3 critical, 3 high, 10 medium, 2 low)


### FINDING 1 (id: `UMB-001`)

- **Source:** `docs/audit_reports/findings/wave4-umbrella.md`
- **Severity:** Critical
- **Area:** infra-umbrella
- **Location:** `crates/educore/Cargo.toml` (entire `[dependencies]` table) and `crates/educore/src/lib.rs` (entire re-export block)

**Description:**

`educore-cli` is scaffolded at `crates/tools/cli/` and is entry #35 in AGENTS.md's "Crate Inventory" (Phase 16, Test infrastructure + SDK) — but it is neither listed in the umbrella's `[dependencies]` nor re-exported in `lib.rs`. Consumers who follow AGENTS.md and depend on `educore::cli::*` for the CLI binary facade will get a compile error. AGENTS.md says the umbrella "re-exports the public surface of all 34 internal crates", and `crates/tools/cli/Cargo.toml` defines `[package] name = "educore-cli"` for that role.

**Expected:**

Per AGENTS.md line 153: "The umbrella crate `educore` re-exports the public surface of all 34 internal crates." Per AGENTS.md line 523: row 35 = `educore-cli` (tools, Phase 16).

**Evidence:**

```toml
  # crates/educore/Cargo.toml — [dependencies] section (lines 6-47) lists
  # 34 deps but educore-cli is absent.
  # crates/tools/cli/Cargo.toml exists and defines name = "educore-cli".
  ```

---

### FINDING 2 (id: `UMB-002`)

- **Source:** `docs/audit_reports/findings/wave4-umbrella.md`
- **Severity:** Critical
- **Area:** infra-umbrella
- **Location:** `crates/educore/Cargo.toml` and `crates/educore/src/lib.rs` (no occurrence of `educore-query-derive`)

**Description:**

`educore-query-derive` is the Phase 0 proc-macro crate (AGENTS.md inventory row #2) that provides `#[derive(DomainQuery)]`. It is scaffolded at `crates/infra/query-derive/` and `library-docs.md` references the macro through the engine surface. The umbrella does NOT depend on it in `Cargo.toml` and does NOT re-export it in `lib.rs`. AGENTS.md line 128-137 mandates "The umbrella re-exports each internal crate under its short name" with the example `pub use educore_core as core;`. A proc-macro crate can be re-exported via `pub use ::educore_query_derive::DomainQuery;` (Rust 2018+) but no such re-export exists.

**Expected:**

Per AGENTS.md line 128-137: "The umbrella re-exports each internal crate under its short name: ... `pub use educore_core as core;` ...". Per AGENTS.md line 490: row 2 = `educore-query-derive` (infra, Phase 0). Per docs/library-docs.md: `#[derive(DomainQuery)]` is the documented entry point to the query layer.

**Evidence:**

```rust
  // crates/educore/src/lib.rs — search for "query_derive" returns 0 matches.
  // crates/educore/Cargo.toml — search for "query-derive" returns 0 matches.
  // AGENTS.md:489-490
  //   | 1 | infra | `educore-core` | 0 | Foundation |
  //   | 2 | infra | `educore-query-derive` | 0 | Foundation (proc-macro) |
  ```

---

### FINDING 3 (id: `UMB-003`)

- **Source:** `docs/audit_reports/findings/wave4-umbrella.md`
- **Severity:** Critical
- **Area:** infra-umbrella
- **Location:** `crates/educore/Cargo.toml:46-47` and `crates/educore/src/lib.rs:54-55` vs. `AGENTS.md:479-524` ("Crate Inventory" table)

**Description:**

The umbrella depends on `educore-sync` and `educore-sync-inprocess` and re-exports them as `sync` and `sync_inprocess`. These crates are described in `docs/build-plan.md` § Phase 0 (ADR-018) and are scaffolded at `crates/cross-cutting/sync/` and `crates/cross-cutting/sync-inprocess/`. However, they are NOT present in AGENTS.md's "Crate Inventory" table (rows 1-35). AGENTS.md's preamble (line 24) asserts "The 34 crates are organized into 5 tiers + 1 umbrella", and the table only contains 35 rows (incl. the umbrella). The actual count is 34 internal crates + 2 sync crates = 36 internal crates, but AGENTS.md only documents 34. The umbrella has 36 dependencies but the inventory lists 35.

**Expected:**

Per AGENTS.md line 24: "The 34 crates are organized into 5 tiers + 1 umbrella." Per AGENTS.md line 479-524: Crate Inventory table is "the authoritative source — do not rely on the directory tree or the umbrella re-exports to determine phase assignment."

**Evidence:**

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

### FINDING 4 (id: `UMB-004`)

- **Source:** `docs/audit_reports/findings/wave4-umbrella.md`
- **Severity:** High
- **Area:** infra-umbrella
- **Location:** `crates/educore/Cargo.toml` (no `[features]` section) vs. `docs/specs/sync/overview.md:27-50`

**Description:**

The umbrella crate declares no `[features]` table and no `default-features` line, yet `docs/specs/sync/overview.md` § "Sync as a Build Feature" (lines 38-52) mandates: "The sync module is gated behind a Cargo feature: ```toml [features] sync = ['dep:educore-events', 'dep:educore-rbac'] ```". The umbrella unconditionally pulls `educore-sync` and `educore-sync-inprocess` (Cargo.toml:46-47) and unconditionally re-exports `pub use educore_sync as sync;` (lib.rs:54), so embedded/server-only consumers that should be able to compile without sync cannot.

**Expected:**

Per `docs/specs/sync/overview.md:30`: "`educore::sync` module is gated behind a Cargo feature so the core engine library can be compiled without any sync dependency for embedded / server-only use cases." Per `docs/specs/sync/overview.md:44-46`: `[features] sync = ['dep:educore-events', 'dep:educore-rbac']`.

**Evidence:**

```toml
  # crates/educore/Cargo.toml — grep for "\[features\]" returns 0 matches.
  # crates/educore/Cargo.toml:46-47
  educore-sync = { workspace = true }
  educore-sync-inprocess = { workspace = true }

  # crates/educore/src/lib.rs:54
  pub use educore_sync as sync;
  ```

---

### FINDING 5 (id: `UMB-005`)

- **Source:** `docs/audit_reports/findings/wave4-umbrella.md`
- **Severity:** High
- **Area:** infra-umbrella
- **Location:** `crates/educore/Cargo.toml` (entire `[dependencies]` block) vs. `crates/educore/src/lib.rs:21-62` (re-export block)

**Description:**

The umbrella's `[dependencies]` table lists 34 entries (lines 6-47), but `lib.rs` re-exports only 32 of them as `pub use ... as ...` (lines 21-62). Cross-checking the two: the umbrella includes `educore_storage_parity` as a dep AND re-exports it as `storage_parity`, but it never includes `educore-cli` or `educore-query-derive` in deps. Meanwhile the AGENTS.md inventory lists `educore-cli` (#35) and `educore-query-derive` (#2) as part of the engine. The dependency count and the re-export count are internally consistent (34 ↔ 32 deps minus the 2 not in inventory) but the inventory itself is incomplete (see UMB-003) so the umbrella is internally consistent with a stale inventory, not with reality.

**Expected:**

Per AGENTS.md line 128-137: "The umbrella re-exports each internal crate under its short name ... Consumers therefore write `educore::academic::commands::*` and never need to know the internal `educore-` prefix on the package name." Per AGENTS.md line 153: "re-exports the public surface of all 34 internal crates" — but reality is 36 internal crates.

**Evidence:**

```bash
  # 34 deps in crates/educore/Cargo.toml:
  #   grep -c 'workspace = true' crates/educore/Cargo.toml  →  34 (incl. tokio dev-dep)
  # 32 pub use re-exports in crates/educore/src/lib.rs:
  #   grep -c '^pub use' crates/educore/src/lib.rs           →  32
  #   (sync + sync_inprocess = 2 extra deps not in AGENTS.md inventory)
  ```

---

### FINDING 6 (id: `UMB-006`)

- **Source:** `docs/audit_reports/findings/wave4-umbrella.md`
- **Severity:** High
- **Area:** infra-umbrella
- **Location:** `crates/educore/src/lib.rs:24` (`pub use educore_events as events;`)

**Description:**

The umbrella re-exports `educore-events` as `events`. AGENTS.md (lines 171-177) explicitly warns about the `educore-events` vs `educore-events-domain` distinction: "`educore-events` (cross-cutting tier) is the **event envelope + bus port** (DomainEvent trait, EventEnvelope, EventBus trait). `educore-events-domain` (cross-cutting tier) is the **calendar domain** (CalendarEvent, Holiday, Incident, Weekend aggregates)." But the umbrella re-exports both as just `events` and `events_domain` (lib.rs:28-29), and `library-docs.md:99-114` tells consumers to write `use educore::events::*;` to subscribe to `StudentAdmitted`. That `events::*` glob will pull in DomainEvent + EventEnvelope + EventBus + CalendarEvent + Holiday + Incident + Weekend — the calendar domain's types from a different bounded context are silently in scope under the `events` path. The two crates must remain distinct in the public surface.

**Expected:**

Per AGENTS.md line 171-177: explicit naming distinction between `educore-events` and `educore-events-domain`. Per docs/library-docs.md:108 `use educore::events::*;` is the documented subscription path and must contain ONLY envelope + bus types.

**Evidence:**

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

### FINDING 10 (id: `UMB-010`)

- **Source:** `docs/audit_reports/findings/wave4-umbrella.md`
- **Severity:** Medium
- **Area:** infra-umbrella
- **Location:** `crates/educore/tests/consumer_e2e.rs:19-28`

**Description:**

The consumer-facing end-to-end test imports internal crate paths directly — `use educore_core::clock::{...}`, `use educore_notify::errors::NotificationTemplateId`, `use educore_payment::port::{...}`, `use educore_sdk::Engine`, `use educore_storage::student_attendance_row::StudentAttendanceRow` — instead of using the umbrella's documented `educore::*` re-export paths. Per `docs/library-docs.md:10` and `AGENTS.md:128-137`, the consumer surface is `educore::*`; the test is meant to be a "consumer-facing E2E" (per its module docstring at line 1) but it bypasses the umbrella entirely. Any regression in the umbrella's re-export wiring would not be caught by this test.

**Expected:**

Per `docs/library-docs.md:8-10`: consumer entry point is `use educore::prelude::*;`. Per the test's own module docstring at `crates/educore/tests/consumer_e2e.rs:1-3`: "Consumer-facing end-to-end integration test for the Educore engine." A consumer-facing test should exercise the umbrella's public path.

**Evidence:**

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

### FINDING 11 (id: `UMB-011`)

- **Source:** `docs/audit_reports/findings/wave4-umbrella.md`
- **Severity:** Medium
- **Area:** infra-umbrella
- **Location:** `crates/educore/src/lib.rs:19` (`// ---- Domain crates ------------------------------------------------------`)

**Description:**

The lib.rs comment at line 19 says "Domain crates" but the section also contains `core`, `platform`, `rbac`, `settings`, `operations`, `events`, `events_domain` — 7 of the 17 entries are NOT domain crates per AGENTS.md's tier table. The first alphabetical entry (`core`) is infra tier; `platform`, `rbac`, `settings`, `operations` are cross-cutting tier; `events` is cross-cutting (envelope + bus port); `events_domain` is cross-cutting (calendar domain per AGENTS.md line 174). A consumer reading the umbrella source as the engine's organization map will mis-classify 7 of the 17 entries in this section.

**Expected:**

Per AGENTS.md line 148: cross-cutting tier = platform, rbac, events, events-domain, settings, operations, audit. Per AGENTS.md line 150: adapters tier. The umbrella's section comments should match the tier table.

**Evidence:**

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

### FINDING 12 (id: `UMB-012`)

- **Source:** `docs/audit_reports/findings/wave4-umbrella.md`
- **Severity:** Medium
- **Area:** infra-umbrella
- **Location:** `crates/educore/src/lib.rs:47` (`pub use educore_storage_parity as storage_parity;`)

**Description:**

The umbrella re-exports `educore_storage_parity` as `storage_parity`. `educore-storage-parity` is in the **tools** tier (AGENTS.md line 151: "Dev tooling: testkit, storage-parity, cli (binary), sdk"), intended for cross-adapter test suites, not for consumer runtime use. Exposing it as a top-level `educore::storage_parity` path implies it's part of the consumer surface. Per AGENTS.md line 405: "Test infrastructure: educore-testkit, educore-storage-parity (full suite), educore-sdk, educore-cli" — `educore-testkit` is similarly a tools-tier crate but re-exported at lib.rs:59 with the same exposure problem.

**Expected:**

Per AGENTS.md line 151: tools tier is dev tooling, not consumer surface. Per `docs/build-plan.md` Phase 16: storage-parity is "the cross-adapter test suite", not a runtime consumer crate.

**Evidence:**

```rust
  // crates/educore/src/lib.rs:47, 59
  pub use educore_storage_parity as storage_parity;
  pub use educore_testkit as testkit;

  // AGENTS.md:151
  //   | `tools` | `crates/tools/` | 4 | Dev tooling: testkit, storage-parity, cli (binary), sdk |
  ```

---

### FINDING 13 (id: `UMB-013`)

- **Source:** `docs/audit_reports/findings/wave4-umbrella.md`
- **Severity:** Medium
- **Area:** infra-umbrella
- **Location:** `crates/educore/src/lib.rs:68-74` (docstring on `pub mod prelude`)

**Description:**

The prelude's rustdoc states: "richer re-exports land alongside the `DomainError`, `TenantContext`, `EventEnvelope`, and `Capability` types in the relevant PRs." It also references "Phase 14" twice. This is a planning note embedded in the public API docstring. The umbrella has `#![deny(missing_docs)]` at line 13, so doc comments are required — but a docstring that is a roadmap is not a docstring. Consumers reading `cargo doc --open` (or docs.rs) will see Phase 14 mentioned, which is implementation-leakage.

**Expected:**

Per AGENTS.md § Code Standards (line 393): "All public APIs are documented with rustdoc; `#![deny(missing_docs)]`." Per AGENTS.md § Engine Rules (line 217): "Production-ready. Real schools, real students, real money." — public docstrings must describe what the type IS, not the implementation roadmap.

**Evidence:**

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

### FINDING 14 (id: `UMB-014`)

- **Source:** `docs/audit_reports/findings/wave4-umbrella.md`
- **Severity:** Medium
- **Area:** infra-umbrella
- **Location:** `crates/educore/Cargo.toml` (no `default-features = false` on any storage adapter dep) vs. `docs/library-docs.md:8-24`

**Description:**

The umbrella unconditionally enables the default features of every dependency it pulls in — `educore-storage-postgres`, `educore-storage-mysql`, `educore-storage-sqlite`, `educore-storage-surrealdb` (Cargo.toml:39-42) and the port adapters (Cargo.toml:23-32). A consumer that wants only the SurrealDB backend per ADR-017 (and ADR-017 explicitly says PG/MySQL/SQLite move to Phase 1 as parity adapters) still pulls in `sqlx`, `mysql_async`, `reqwest`, `lettre`, etc. via feature unification. ADR-015 mandates "TLS/SSL Cross-Compilation: Strictly enforce `rustls` instead of `native-tls` ... For crates like `reqwest`, always set `default-features = false`" — but the umbrella inherits those deps without setting `default-features = false`, so a consumer depending on `educore` cannot override the feature set.

**Expected:**

Per AGENTS.md line 391-395: "TLS/SSL Cross-Compilation: Strictly enforce `rustls` instead of `native-tls` to support cross-compilation ... For crates like `reqwest`, always set `default-features = false` and enable the `rustls` or `rustls-tls` feature." Per ADR-017: SurrealDB is primary; PG/MySQL/SQLite are parity adapters — the umbrella should make SurrealDB the default and gate the others.

**Evidence:**

```toml
  # crates/educore/Cargo.toml:39-42
  educore-storage-postgres = { workspace = true }
  educore-storage-mysql = { workspace = true }
  educore-storage-sqlite = { workspace = true }
  educore-storage-surrealdb = { workspace = true }
  # No `default-features = false`, no feature gate.
  ```

---

### FINDING 16 (id: `UMB-016`)

- **Source:** `docs/audit_reports/findings/wave4-umbrella.md`
- **Severity:** Medium
- **Area:** infra-umbrella
- **Location:** `crates/educore/src/lib.rs:65` (`pub const VERSION`)

**Description:**

The `VERSION` constant is a useful re-export of the package version, but its docstring (lib.rs:64) does not cite a stability policy. The umbrella has `#![deny(missing_docs)]`, so the docstring is required — but consumers building tools around `educore::VERSION` (e.g. health checks, telemetry, log enrichment) need to know whether the value is sourced from `Cargo.toml`'s `[workspace.package] version` (single source, bumped manually per release) or from `git describe` output, and whether it is updated at runtime. Without that information, downstream tooling that pins to a specific `VERSION` cannot reason about upgrade safety.

**Expected:**

Per AGENTS.md line 393: "All public APIs are documented with rustdoc". Per AGENTS.md line 395: "All fallible APIs return `Result<T, DomainError>`" — invariants should be documented. The constant's docstring should describe its source and update semantics.

**Evidence:**

```rust
  // crates/educore/src/lib.rs:64-65
  /// Educore version, sourced from the package manifest.
  pub const VERSION: &str = env!("CARGO_PKG_VERSION");
  ```

---

### FINDING 17 (id: `UMB-017`)

- **Source:** `docs/audit_reports/findings/wave4-umbrella.md`
- **Severity:** Medium
- **Area:** infra-umbrella
- **Location:** `crates/educore/src/lib.rs:21-62` (no re-export ordering stability invariant)

**Description:**

The umbrella re-export block does not declare a stability order. `pub use educore_academic as academic;` is at line 21, `pub use educore_events as events;` is at line 28. AGENTS.md (line 128) and `library-docs.md` describe `educore::*` as "a single, stable path" but provide no ordering invariant — any future PR that reorders or inserts new entries alphabetically will shift line numbers, which matters for tools that diff the umbrella's surface (e.g. `cargo-public-api`, `cargo-geiger`, audit scripts). The current ordering is approximately alphabetical within each section, which is a de-facto contract.

**Expected:**

Per AGENTS.md line 153: "The umbrella crate `educore` re-exports the public surface of all 34 internal crates" — "public surface" implies a stable contract. Per docs/library-docs.md:1-2: "the umbrella crate ... Re-exports every domain, port, and adapter crate under a single, stable path" — "stable" is promised but not defined.

**Evidence:**

```rust
  // crates/educore/src/lib.rs:21-62 — 32 pub use statements in 6 sections.
  // No docstring, comment, or test asserts an ordering invariant.
  ```

---

### FINDING 7 (id: `UMB-007`)

- **Source:** `docs/audit_reports/findings/wave4-umbrella.md`
- **Severity:** Medium
- **Area:** infra-umbrella
- **Location:** `crates/educore/src/lib.rs:75-83` (`pub mod prelude`)

**Description:**

The `prelude` module re-exports only 7 paths: `educore_core`, `educore_events`, `educore_operations`, `educore_platform`, `educore_rbac`, `educore_sdk`, `educore_settings`. `docs/library-docs.md` line 10 mandates `use educore::prelude::*;` for the consumer-facing example and shows `Engine::builder()`, `engine.auth()`, `engine.notify()`, `engine.events()`, `engine.rbac()`, `engine.storage()`, `engine.students()`, `engine.attendance()` — none of which are reachable through the current prelude (none of `Engine`, `EngineBuilder`, `StudentService`, `AttendanceService`, `NotifyService`, `PaymentService`, `EventBus`, `RbacProvider`, `StorageAdapter` are flat re-exports). The prelude only re-exports the crate *names*, not the facade types, so `use educore::prelude::*;` gives the consumer 7 modules they still have to navigate into.

**Expected:**

Per docs/library-docs.md:8-10: the consumer example begins with `use educore::prelude::*;` and is followed immediately by `Engine::builder()` — implying prelude must surface `Engine` (and the underlying facade services) directly. Per AGENTS.md Engine Rules § "Compile-time safety over strings": prelude is the documented ergonomic entry.

**Evidence:**

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

### FINDING 8 (id: `UMB-008`)

- **Source:** `docs/audit_reports/findings/wave4-umbrella.md`
- **Severity:** Medium
- **Area:** infra-umbrella
- **Location:** `crates/educore/src/lib.rs:57-59` (Test infrastructure section)

**Description:**

The umbrella's lib.rs is divided into 6 sections (Domain crates, Port adapters, Sync engine, Test infrastructure, High-level SDK). `educore_audit` (re-exported at line 58) is grouped under "Test infrastructure" — but `educore-audit` is in the **cross-cutting** tier per AGENTS.md line 148 and inventory row #13. This is a tier-bucketing mistake in the umbrella: a cross-cutting concern is mis-labeled as test infrastructure, which would mislead consumers reading the umbrella's source as a tier map.

**Expected:**

Per AGENTS.md line 501: row 13 = `educore-audit` (cross-cutting, Phase 2 — Cross-cutting foundations / audit log). Per AGENTS.md line 148: cross-cutting tier table lists `audit` as one of its 7 crates.

**Evidence:**

```rust
  // crates/educore/src/lib.rs:56-59
  // ---- Test infrastructure -------------------------------------------------
  pub use educore_audit as audit;
  pub use educore_testkit as testkit;

  // AGENTS.md:501
  //   | 13 | cross-cutting | `educore-audit` | 2 | Cross-cutting foundations (audit log) |
  ```

---

### FINDING 9 (id: `UMB-009`)

- **Source:** `docs/audit_reports/findings/wave4-umbrella.md`
- **Severity:** Medium
- **Area:** infra-umbrella
- **Location:** `crates/educore/src/lib.rs:20-37` (Domain crates section)

**Description:**

The "Domain crates" section header (lib.rs:20) groups 17 re-exports together, but only 10 of them are actually `domains/`-tier crates (`academic`, `assessment`, `attendance`, `cms`, `communication`, `documents`, `events_domain`, `facilities`, `finance`, `hr`, `library` — 11). The other 6 (`core`, `platform`, `rbac`, `settings`, `operations`, `events`) are infra or cross-cutting. The section header is mislabeled. The umbrella then places `educore-events-domain` (a cross-cutting calendar crate per AGENTS.md line 174-177) under the "Domain crates" banner.

**Expected:**

Per AGENTS.md line 148: cross-cutting tier is platform, rbac, events, events-domain (calendar), settings, operations, audit (7 crates). Per AGENTS.md line 149: domains tier is the 10 domain bounded contexts. The umbrella's section headers should follow the tier table, not bundle all non-adapter crates under "Domain crates".

**Evidence:**

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

### FINDING 15 (id: `UMB-015`)

- **Source:** `docs/audit_reports/findings/wave4-umbrella.md`
- **Severity:** Low
- **Area:** infra-umbrella
- **Location:** `crates/educore/src/lib.rs:21-37` (alphabetical ordering of section entries)

**Description:**

Within the "Domain crates" section (lib.rs:21-37), entries are roughly alphabetical but `educore-core` (line 26) sorts to position 6 (between `cms` and `documents`), while per AGENTS.md's tier table core is infra — it should appear under a "Foundation" section. Similarly `educore-platform` (line 35) sorts between `operations` and `rbac`, mixing cross-cutting crate names with domain crate names alphabetically. A consumer scanning for a specific crate by name will be misled into thinking the umbrella's section order reflects the engine's tier order.

**Expected:**

Per AGENTS.md line 147-151: 5-tier system; consumers are taught to read by tier. The umbrella's section comments should follow tier order (infra → cross-cutting → domains → adapters → tools) for at-a-glance navigation.

**Evidence:**

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

### FINDING 18 (id: `UMB-018`)

- **Source:** `docs/audit_reports/findings/wave4-umbrella.md`
- **Severity:** Low
- **Area:** infra-umbrella
- **Location:** `crates/educore/src/lib.rs:18` (section comment ordering)

**Description:**

The umbrella's `lib.rs` opens with `// ---- Domain crates ----` as the first section comment (line 20), then `// ---- Port adapters` (line 39), then `// ---- Sync engine` (line 53), then `// ---- Test infrastructure` (line 57), then `// ---- High-level SDK` (line 61). This order is not the tier order from AGENTS.md (infra → cross-cutting → domains → adapters → tools) nor the AGENTS.md Crate Inventory row order (rows 1-35: infra first, then cross-cutting, then domains, then adapters, then tools). The umbrella's ordering is domains-first, which does not match either the dependency direction or the inventory table — it inverts the natural top-down read.

**Expected:**

Per AGENTS.md line 158-161: "Layered dependency direction (no cycles, no upward deps): infra ← cross-cutting ← domains ← tools; ↑; └── adapters (also depends on infra + cross-cutting)." The umbrella's source organization should follow this direction.

**Evidence:**

```rust
  // crates/educore/src/lib.rs:19-62 — section order:
  //   1. Domain crates (line 19)
  //   2. Port adapters (line 38)
  //   3. Sync engine (line 52)
  //   4. Test infrastructure (line 56)
  //   5. High-level SDK (line 60)
  // AGENTS.md tier order: infra → cross-cutting → domains → adapters → tools.
  ```

---

