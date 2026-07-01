## Wave 4 Foundation Audit Report — `educore-core`

**Scope:** `crates/infra/core/` — the `educore-core` package (Cargo.toml, src/lib.rs, src/error.rs, src/ids.rs, src/value_objects.rs, src/clock.rs, src/tenant.rs, src/query.rs, src/lint.rs, src/bin/lint.rs). Cross-referenced against `docs/build-plan.md` § Phase 0 (lines 156-200) and § The No-Gaps Gates (lines 1829-1940), `docs/code-standards.md`, `AGENTS.md`, and the per-crate lint module spec.

**Total findings:** 26

---

### FINDING 1

- **id:** CORE-001
- **area:** infra
- **severity:** Critical
- **location:** `crates/infra/core/src/lint.rs:107-156` (`runner::check_coverage_matrix`)
- **description:** The coverage-matrix sync check declared by `docs/build-plan.md:1925-1932` (item 5 of the No-Gaps Gates) is implemented as a no-op. The function reads `docs/coverage.toml`, builds a `status_tested: Option<(String, usize)>` accumulator across the lines, and then explicitly discards the result with `let _ = status_tested;` (line 154). No `Violation` is ever appended to `_report`, regardless of whether a `Tested` row is missing a `tests = "..."` path or pointing at a nonexistent file. The check is gated by `#[cfg(feature = "lint")]` and declared in the lint's own docs as one of the five no-gaps gates; an empty pass would silently let any `Tested` row lie.
- **expected:** `docs/build-plan.md:1925-1932`: "the lint reads `docs/coverage.toml` and verifies: Every `Tested` row has a `tests` path that exists." Plus `AGENTS.md` "Validation Checklist (per PR)": coverage matrix is a per-PR gate; the lint must enforce it.
- **evidence:**
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

### FINDING 2

- **id:** CORE-002
- **area:** infra
- **severity:** Critical
- **location:** `crates/infra/core/src/lint.rs:1-300` (entire lint sub-module) vs. `docs/build-plan.md:1880-1935` (No-Gaps Gates item 2)
- **description:** The lint sub-module's public surface is `pub fn run(repo_root) -> LintReport` (`src/lint.rs:160-166`), which calls only `check_coverage_matrix` and `check_anti_patterns`. `docs/build-plan.md` § The No-Gaps Gates item 2 enumerates FIVE categories of checks: (1) spec→code direction (`tables.md` → `aggregate.rs`, `commands.md` → `commands.rs`, `events.md` → `events.rs`, `migrations/engine/*.sql` → `create_<table>_ddl()`); (2) code→spec direction; (3) anti-patterns; (4) parity; (5) coverage-matrix sync. The implementation only attempts (3) and (5), and (5) is a no-op (see CORE-001). Checks 1, 2, and 4 are entirely absent; the runner has no functions to perform them.
- **expected:** `docs/build-plan.md:1880-1935`: the `lint::run` runner is the per-crate gate that catches "missing handlers, anti-patterns, reverse-direction drift, and matrix lies." All five sub-checks must be invoked.
- **evidence:**
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

### FINDING 3

- **id:** CORE-003
- **area:** infra
- **severity:** Critical
- **location:** `crates/infra/core/src/lint.rs:181-238` (`scan_file_for_anti_patterns`)
- **description:** The anti-pattern scan only detects `.unwrap()`, `.unwrap_err()`, `panic!(`, `todo!()`, `unimplemented!()` (needle array at line 220). It does NOT detect `.expect(`, which `AGENTS.md` and `docs/code-standards.md` explicitly forbid alongside `unwrap`. It does NOT detect `as` casts on numerics (also banned), `serde_json::Value` (banned in domain code), or `HashMap<String, T>` (banned for domain data) — all four of these are called out by name in the build plan § The No-Gaps Gates item 3 and in `AGENTS.md` "Validation Checklist".
- **expected:** `docs/build-plan.md:1917-1923`: "Anti-patterns: No `unimplemented!()`, `todo!()`, or `// TODO: implement` in production code (test code is exempt via `#[cfg(test)]` detection). No `as` on numerics in domain crates (per `AGENTS.md`'s `as` ban). No `serde_json::Value` in domain code. No `HashMap<String, T>` for domain data."
- **evidence:**
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

### FINDING 4

- **id:** CORE-004
- **area:** infra
- **severity:** Critical
- **location:** `crates/infra/core/src/query.rs:74-89` (Value enum + manual `impl Eq for Value`)
- **description:** `Value::F64(f64)` is a variant of `Value` (line 74), but the manual `impl Eq for Value {}` at line 89 claims `Value` satisfies the `Eq` trait. `f64` does not implement `Eq` because `NaN != NaN` violates reflexivity. The manual impl is allowed by the compiler because `Eq` is a marker trait with no methods and Rust does not field-check it, but the contract is unsound — any consumer that relies on `Value` being `Eq` (e.g. for `HashMap<Value, _>` or `BTreeMap<Value, _>`) will silently produce NaN-keyed buckets or panic at runtime in `BTreeMap`. This contradicts the engine's type-safety rules.
- **expected:** `docs/code-standards.md` "Type Safety" / "Forbidden Patterns": "No `HashMap<String, T>` for domain data. Use typed structs." Indirectly: typed value types must be honestly `Eq` if marked so. `AGENTS.md` "Type Safety": "Enforce full type safety at all times."
- **evidence:**
  ```rust
  crates/infra/core/src/query.rs:74-89
  /// A 64-bit floating point.
  F64(f64),
  ...
  }
  
  impl Eq for Value {}
  ```

---

### FINDING 5

- **id:** CORE-005
- **area:** infra
- **severity:** Critical
- **location:** `crates/infra/core/src/ids.rs:10` (rustdoc link) vs. `crates/infra/core/src/clock.rs` (no `id_gen` module)
- **description:** The rustdoc for `pub mod ids;` references the path `crate::id_gen::IdGenerator` at line 10, but `educore-core` has no `id_gen` module. `IdGenerator` lives in `crate::clock` (`src/clock.rs:79-117`). Any consumer that follows the documented path receives a compile error. This is a dead doc-link and a documentation/code drift blocker.
- **expected:** `docs/code-standards.md` "Documentation: Every public item has a rustdoc comment." All intra-doc links must resolve. `AGENTS.md`: "Documentation: complete (~302 markdown files, 15 domain specs × 11 files each = 165 spec files)."
- **evidence:**
  ```rust
  crates/infra/core/src/ids.rs:10
  //! generated by the [`IdGenerator`](crate::id_gen::IdGenerator) port
  ```
  `grep -n "pub mod id_gen" crates/infra/core/src/` returns no matches. `IdGenerator` is defined at `crates/infra/core/src/clock.rs:79`.

---

### FINDING 6

- **id:** CORE-006
- **area:** infra
- **severity:** Critical
- **location:** `crates/infra/core/src/lint.rs:160-166` (`pub fn run`) and absence of tier-boundary check
- **description:** `AGENTS.md` § Tier System mandates that "the `educore-core::lint` sub-module verifies at build time that a crate in `crates/domains/` does not import from `crates/adapters/` or `crates/tools/`, and that a crate in `crates/cross-cutting/` does not import from `crates/domains/`, `crates/adapters/`, or `crates/tools/`." The implemented `lint::run` has no check that walks `Cargo.toml` files or `mod` declarations to enforce these tier boundaries. The function only invokes the coverage-matrix (no-op, see CORE-001) and the anti-pattern scan (incomplete, see CORE-003).
- **expected:** `AGENTS.md` § Tier System ("Tier boundary enforcement"). `docs/build-plan.md:1830-1934`: the lint is the "per-crate gate" that catches all five categories.
- **evidence:**
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

### FINDING 7

- **id:** CORE-007
- **area:** infra
- **severity:** High
- **location:** `crates/infra/core/src/error.rs:71-80` (`DomainError::kind`)
- **description:** `kind()` returns `ErrorKind::Validation` for BOTH `DomainError::Validation` AND `DomainError::NotSupported` (line 75). The doc comment claims "The set of variants is closed; new kinds require a major version bump" but two distinct variants collapse into one `ErrorKind`. Callers that branch on `kind()` cannot distinguish "input failed validation" from "the adapter does not support this operation" — the two are operationally different (validation is a 4xx client error; `NotSupported` is a 501-equivalent server-side capability gap). The `ErrorKind` enum is also missing a `NotSupported` discriminant entirely; the variant exists on `DomainError` but has no kind.
- **expected:** `docs/code-standards.md` "Error Handling: Engine-level errors include a `kind` discriminant (`Validation`, `NotFound`, `Conflict`, `Forbidden`, `Infrastructure`)." Plus `docs/schemas/event-schema.md` (per the error.rs doc-comment): one variant per kind, no collisions.
- **evidence:**
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

### FINDING 8

- **id:** CORE-008
- **area:** infra
- **severity:** High
- **location:** `crates/infra/core/src/value_objects.rs:212-225` (`Etag::placeholder`)
- **description:** The doc comment claims "We construct via the public API for symmetry with the rest of the code, but the result is `unwrap_or`-safe — if the validator is ever changed to reject it, the unit test in this file's `tests` mod surfaces the regression." In fact the function does NOT invoke the public API — it directly constructs `Self("00000000000000000000000000000000".to_owned())`, bypassing `Etag::new`'s validator entirely. The "surfaces the regression" claim is false; if the validator is later changed to reject `"0000...0000"`, `Etag::placeholder()` will continue to construct an invalid `Etag` with no test failure.
- **expected:** `docs/code-standards.md` "Value objects are immutable and validated at construction." All construction paths must route through the validator; bypasses must be flagged by tests.
- **evidence:**
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

### FINDING 9

- **id:** CORE-009
- **area:** infra
- **severity:** High
- **location:** `crates/infra/core/src/tenant.rs:25-50` (`TenantContext` struct fields)
- **description:** `TenantContext` exposes seven `pub` fields (`school_id`, `actor_id`, `session_id`, `correlation_id`, `causation_id`, `user_type`, `locale`, `timezone`). Consumers can mutate any field directly after construction, bypassing `for_user`/`system`/the builder. This contradicts the engine's "value object is immutable" rule and the tenancy-schema spec which states the context is "immutable for the lifetime of a single command."
- **expected:** `docs/schemas/tenancy-schema.md` § 3 (cited in `tenant.rs:7-14`): "The `TenantContext` is **immutable** for the lifetime of a single command." Plus `docs/code-standards.md`: "Value objects are immutable and validated at construction."
- **evidence:**
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

### FINDING 10

- **id:** CORE-010
- **area:** infra
- **severity:** High
- **location:** `crates/infra/core/src/tenant.rs:55-83` (`TenantContext::for_user` and `system` constructors)
- **description:** Neither constructor validates `school_id`. A caller can pass `PUBLIC_SCHOOL_ID` (the nil UUID, defined at `src/ids.rs:293`) to `for_user`, producing a `TenantContext` whose `school_id` is the public-content anchor — but `user_type` is set to `Teacher`/`Parent`/etc. (a real school-scoped role). The RLS policy at `docs/schemas/tenancy-schema.md` § 4 expects `school_id = PUBLIC` to be reserved for global/public-site aggregates; mixing it with a school-scoped actor role bypasses tenant isolation silently. No `Result` is returned and no `DomainError::Validation` is raised.
- **expected:** `docs/schemas/tenancy-schema.md` § 3: "`school_id` is mandatory and non-nil for school-scoped actors." Plus `docs/code-standards.md`: "All fallible APIs return `Result<T, DomainError>`."
- **evidence:**
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

### FINDING 11

- **id:** CORE-011
- **area:** infra
- **severity:** High
- **location:** `crates/infra/core/Cargo.toml:24-34` (`[dependencies]`)
- **description:** The crate declares six workspace dependencies that are not referenced anywhere in `src/`: `derive_more`, `validator`, `secrecy`, `async-trait`, `indexmap`, `tracing`. `cargo check` succeeds only because `cargo` does not error on unused dependencies by default, but the workspace's own `cargo build` rules (AGENTS.md § Package Manager) require every dependency to be used. Cargo-feature bloat grows compile times across the 33 downstream crates that depend on `educore-core`.
- **expected:** `AGENTS.md` § Package Manager: "Use **cargo** to manage dependencies and build targets." Unused dependencies should be removed via `cargo remove`.
- **evidence:**
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

### FINDING 12

- **id:** CORE-012
- **area:** infra
- **severity:** High
- **location:** `crates/infra/core/src/query.rs:262-278` (`QueryNode::is_empty`)
- **description:** `is_empty()` returns `true` only when ALL leaves are `And`/`Or`/`Not` whose children recursively empty. The doc comment claims "a degenerate `And(And(...), And(...))` with all-empty children collapses to this in the macro emission," implying that a query with no filters collapses to empty. The implementation never returns `true` for a leaf node (every variant besides `And`/`Or`/`Not` hits the `_ => false` arm at line 270). The unit test at line 539 explicitly documents the contradiction: "an `IsNull` is not degenerate. So this returns false." The macro therefore cannot produce a sentinel "empty filter" node; a query with `Eq(field, value)` is never empty, which means callers cannot use `is_empty()` to skip filter emission.
- **expected:** `docs/query_layer.md` § "Empty query": an empty filter tree must be representable so adapters can elide the `WHERE` clause. The doc comment at line 261-264 explicitly promises this collapsing behavior.
- **evidence:**
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

### FINDING 13

- **id:** CORE-013
- **area:** infra
- **severity:** High
- **location:** `crates/infra/core/src/clock.rs:203-230` (`fn deterministic_v7`)
- **description:** The private `deterministic_v7` helper uses seven `as u8` casts on a `u64` counter (lines 212-226), truncating from 64 to 8 bits each time. AGENTS.md § Agent Instructions → Type Safety: "No `as` casts that truncate or lose data. Use `TryFrom` / `TryInto` with proper error handling." Even though the function is private and the truncation is intentional (encoding the counter into UUID bytes), the casts would be flagged by the lint that AGENTS.md mandates and contradict the engine's strict no-`as` policy.
- **expected:** `AGENTS.md` § Agent Instructions → Type Safety: "No `as` casts that truncate or lose data." `docs/code-standards.md`: "Numeric conversions use `TryFrom`/`TryInto`. `as` is forbidden on numerics."
- **evidence:**
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

### FINDING 14

- **id:** CORE-014
- **area:** infra
- **severity:** High
- **location:** `crates/infra/core/src/clock.rs:104-118` (`TestClock::set` and `advance`)
- **description:** `TestClock::set` documents "If the underlying mutex is poisoned (a panic occurred while holding it), the clock is set to the test's epoch as a safe default." The actual implementation at line 110 calls `poisoned.into_inner()` to recover the value and overwrites it with the requested `t` (line 114 `*g = t;`), so the recovery is correct. However, `TestClock::advance` at line 117-130 uses `.unwrap_or(chrono::DateTime::<chrono::Utc>::MAX_UTC)` (line 127) — this is a silent fallback that the doc comment describes as "the clock is clamped to the representable maximum." The fallback is correct for `checked_add_signed` overflow but is invisible to tests: a test that advances by an excessive duration silently pins the clock at `MAX_UTC` rather than failing, masking bugs in test setup. The lint the engine mandates would flag the `unwrap_or` if it scanned test code, but the needle array does not include `unwrap_or` (see CORE-003).
- **expected:** `AGENTS.md` § Agent Instructions → Type Safety: "No `unwrap()` or `expect()` in production paths." Plus: tests should fail loud on overflow, not silently clamp.
- **evidence:**
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

### FINDING 15

- **id:** CORE-015
- **area:** infra
- **severity:** High
- **location:** `crates/infra/core/src/query.rs:447-472` (`to_relational_node`)
- **description:** `to_relational_node` discards the field identity of every leaf variant, replacing it with the `RelationalField` placeholder (lines 449-461 all emit `QueryNode::Variant(RelationalField, ...)`). The function comment at line 437-446 explains that the placeholder "is used because the related aggregate's field type is unknown to the parent builder; the storage adapter resolves the actual column names at translation time via the `Relation::name`." But because every leaf in the relational subtree maps to the same `RelationalField` (whose `column_name` returns `"<relation>"` per `src/query.rs:388`), the storage adapter receives no field information whatsoever — only the relation's `name` from the wrapping `HasRelation` variant. Any two filters in the same relational closure (e.g. `parent.eq(ParentField::Name, "x")` AND `parent.gt(ParentField::Age, 18)`) become indistinguishable in the AST.
- **expected:** `docs/query_layer.md` § "Nested relational filters" and `docs/code-standards.md` "Compile-time safety over strings." Filters must be uniquely identifiable at the AST level.
- **evidence:**
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

### FINDING 16

- **id:** CORE-016
- **area:** infra
- **severity:** High
- **location:** `crates/infra/core/src/ids.rs:54-65` (`UserId`, `SchoolId`, `EventId`, ... field visibility)
- **description:** All six typed identifier wrappers (`UserId`, `SchoolId`, `EventId`, `CorrelationId`, `SessionId`, `IdempotencyKey`) expose `pub Uuid` as a transparent field. A consumer can mutate the inner UUID in place (e.g. `let mut id = some_school_id; id.0 = another_uuid;`) without going through any constructor. This bypasses the newtype pattern's discipline and makes the documented guarantee ("Two identifiers that share the same underlying UUID but have different Rust types are not interchangeable") enforceable only at construction time, not at use time. `from_uuid_checked` exists for v7 validation but is bypassed by direct field assignment.
- **expected:** `docs/code-standards.md` "DDD Rules: Identifiers are typed (`StudentId`, `GuardianId`, ...), not raw `u64`." Plus "Value objects are immutable." AGENTS.md § Engine Rules: "Compile-time safety over strings."
- **evidence:**
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

### FINDING 17

- **id:** CORE-017
- **area:** infra
- **severity:** High
- **location:** `crates/infra/core/src/ids.rs:221-259` (six `From<Uuid>` impls)
- **description:** `From<Uuid>` is implemented infallibly for all six identifier types, wrapping the `Uuid` without checking its version. `from_uuid_checked` exists at line 95-103 specifically to validate v7, but `From<Uuid>` provides an unchecked bypass. A consumer who writes `let id: SchoolId = uuid_v4.into();` (or worse, a mis-typed JSON deserializer that materializes an arbitrary UUID) silently constructs a non-v7 identifier. The engine's invariants ("All identifiers are UUIDv7") are violated with no compile-time or runtime signal.
- **expected:** `docs/schemas/database-schema.md` § 1.4 (cited in `ids.rs:36-39`): "UUIDv7 as the default identifier." Plus `AGENTS.md` engine rule: "Compile-time safety over strings."
- **evidence:**
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

### FINDING 18

- **id:** CORE-018
- **area:** infra
- **severity:** High
- **location:** `crates/infra/core/src/error.rs:185-200` (`impl From<String>` and `impl From<&str>` for `DomainError`)
- **description:** `From<String>` and `From<&str>` for `DomainError` unconditionally map to `DomainError::Validation`. Any string — including a "NotFound: student missing" or "Conflict: version stale" message — is silently downgraded to a validation error. Callers using `?` on a `String` error from an adapter lose semantic information. The blanket impl also conflicts with the engine's intent that errors carry a precise `kind` discriminant (see CORE-007).
- **expected:** `docs/code-standards.md` "Error Handling: Tests assert on error variants, not on display strings. Engine-level errors include a `kind` discriminant." Errors must preserve semantic category end-to-end.
- **evidence:**
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

### FINDING 19

- **id:** CORE-019
- **area:** infra
- **severity:** Medium
- **location:** `crates/infra/core/src/value_objects.rs:62-79` (`Timestamp::epoch`)
- **description:** `Timestamp::epoch()` uses a `match` over `DateTime::<Utc>::from_timestamp(0, 0)` with a fallback `from_timestamp_nanos(0)` to avoid an `expect`. The two branches both produce the same value (Unix epoch) on every supported chrono version, so the fallback is unreachable. The convoluted double-construction is documented as "satisfies the engine's no-`expect` rule while preserving the const signature" but introduces a redundant code path and obscures intent. A `const fn` that returns the constant string would be simpler.
- **expected:** `AGENTS.md` § Type Safety / "no `expect`": bypass via unreachable branch is acceptable but should not be reflected in production logic.
- **evidence:**
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

### FINDING 20

- **id:** CORE-020
- **area:** infra
- **severity:** Medium
- **location:** `crates/infra/core/src/tenant.rs:204-243` (`Locale` and `TimeZone` constructors)
- **description:** `Locale::new(s: impl Into<String>)` and `TimeZone::new(s: impl Into<String>)` are infallible constructors that accept any string with no BCP 47 / IANA validation. The doc comments at line 199 and line 235 state "The engine does not validate the tag — consumers normalize at the boundary." This contradicts the engine's value-object rule ("validated at construction") and pushes normalization burden to every consumer. A `Locale("totally bogus")` and `TimeZone("not_a_tz")` are silently accepted; downstream rendering and DB columns will fail opaquely.
- **expected:** `docs/code-standards.md` "Value objects are immutable and validated at construction." `AGENTS.md` Validation Checklist: "Public APIs documented."
- **evidence:**
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

### FINDING 21

- **id:** CORE-021
- **area:** infra
- **severity:** Medium
- **location:** `crates/infra/core/src/value_objects.rs:144-159` (`ActiveStatus` derives)
- **description:** `ActiveStatus` derives `Default` with `#[default]` on `Active` (line 146), but `#[derive(Default)]` on an enum is stable only with explicit discriminant choice. The engine also declares `Active = 1` and `Retired = 0` as explicit discriminants (lines 145, 150). `serde::Deserialize` will accept arbitrary integer discriminants by default — there is no `#[serde(try_from = "u8", into = "u8")]` shim, so a JSON `{"ActiveStatus": 7}` deserializes successfully into the default `Active` variant (via serde's fallback for unknown variants) rather than returning a `DomainError::Validation`. The dedicated `from_byte` validator at line 174-182 enforces the 0/1 contract for in-process construction but is bypassed by deserialization.
- **expected:** `docs/schemas/database-schema.md` § 6 (cited at line 137): "active_status = 1" / "active_status = 0" with no other values. `AGENTS.md` Type Safety: serialization round-trips must preserve invariants.
- **evidence:**
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

### FINDING 22

- **id:** CORE-022
- **area:** infra
- **severity:** Medium
- **location:** `crates/infra/core/src/clock.rs:258-272` (`test_clock_set` test)
- **description:** The `test_clock_set` test (line 258-265) calls `c.set(Timestamp::from_datetime(chrono::Utc::now()))` — invoking the system clock inside a "deterministic" test fixture. The test then sleeps 2ms and asserts `a == b` (the clock did not advance), which works only because `set` overwrites the value rather than advancing. If a future refactor changes `set` to "advance to wall-clock now" (a plausible semantic), the test silently passes because the sleep is consumed; the test no longer proves determinism. The test contradicts the documented purpose of `TestClock`.
- **expected:** `AGENTS.md` § Agent Instructions → Testing: "No dummy tests. Every test must validate a real-world scenario." `docs/guides/test-strategy.md` (cited in `clock.rs:1-10`): test fixtures should be fully deterministic.
- **evidence:**
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

### FINDING 23

- **id:** CORE-023
- **area:** infra
- **severity:** Medium
- **location:** `crates/infra/core/src/lint.rs:240-262` (`match_block_close`)
- **description:** `match_block_close` counts `{` and `}` braces line-by-line without skipping braces inside string literals, character literals, line comments, or block comments. The doc comment acknowledges this ("a simple depth counter is enough for the lint's needs ... a false-positive here only widens the exempt window by one or two lines, which is harmless") but a `{` inside a doc comment (e.g. `/// ```rust\n/// let s = "{";\n/// ````) shifts the depth counter, causing the scanner to mark the wrong end of the `mod tests` block and exempt a larger (or smaller) window than intended. Lines containing the forbidden anti-patterns in that mis-counted window are silently passed.
- **expected:** `AGENTS.md` "Type Safety": "Enforce full type safety at all times." Lint scanners must be correct; a permissive count is a false-negative source.
- **evidence:**
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

### FINDING 24

- **id:** CORE-024
- **area:** infra
- **severity:** Medium
- **location:** `crates/infra/core/src/lint.rs:181-238` (`scan_file_for_anti_patterns` needle array)
- **description:** The needle array at line 220-225 lists `.unwrap()`, `.unwrap_err()`, `panic!(`, `todo!()`, `unimplemented!()` but does NOT include `.expect(`, `.unwrap_or(` (with no panic but signal of swallowing errors), `unreachable!(`, or `dbg!(`. AGENTS.md and `docs/code-standards.md` forbid `expect()` in production paths alongside `unwrap()`. The lint's doc comment at line 178-181 explicitly claims it flags "unwrap/expect/panic/todo!/unimplemented!" — the doc and the implementation disagree.
- **expected:** `docs/build-plan.md:1917-1923`: "No `unimplemented!()`, `todo!()`, or `// TODO: implement` in production code." `AGENTS.md` "Type Safety: No `unwrap()` or `expect()` in production paths."
- **evidence:**
  ```rust
  crates/infra/core/src/lint.rs:181-184
  /// Flags `unwrap`/`expect`/`panic!`/`todo!`/`unimplemented!`
  /// calls in production Rust source. Test code (detected by
  /// `#[cfg(test)]` blocks or by the file living under
  /// `tests/`) is exempt.
  ```
  And line 220-225 (needle array shown in CORE-003): `.expect(` is missing.

---

### FINDING 25

- **id:** CORE-025
- **area:** infra
- **severity:** Medium
- **location:** `crates/infra/core/src/lint.rs:42-58` (`Violation::check` and `LintReport::print_to_stderr`)
- **description:** `Violation::check` is a free-form `String`. `scan_file_for_anti_patterns` populates it with `format!("anti_pattern:{needle}")` (line 230), producing strings like `anti_pattern:.unwrap()` or `anti_pattern:.unwrap_err()`. CI scripts that filter on `check == "anti_pattern:unwrap"` (the conventional dotted identifier) will miss all current violations because the actual string embeds the literal token. The check id is also unstable across changes to the needle list (a future addition of `.expect(` would silently change downstream filters' behaviour).
- **expected:** `AGENTS.md` Validation Checklist: deterministic CI gating requires stable identifiers.
- **evidence:**
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

### FINDING 26

- **id:** CORE-026
- **area:** infra
- **severity:** Low
- **location:** `crates/infra/core/src/ids.rs:285-301` (`PUBLIC_SCHOOL_ID` constant vs. `SchoolId::PUBLIC` accessor)
- **description:** `AGENTS.md` § Crate Inventory (Phase 12 entry) and the doc comment at `ids.rs:291` both refer to `SchoolId::PUBLIC` (an associated constant or method on `SchoolId`), but the actual constant is named `PUBLIC_SCHOOL_ID` (free const, line 293) and `SchoolId::is_public(self) -> bool` is an instance method (line 301). The naming convention is inconsistent with `SYSTEM_USER_ID` / `PLATFORM_SCHOOL_ID` (also free consts) and with the doc's prose. Consumers following AGENTS.md will not find `SchoolId::PUBLIC` and may compile-fail.
- **expected:** `AGENTS.md` § Crate Inventory (Phase 12): "SchoolId::PUBLIC constant added to educore-core."
- **evidence:**
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
