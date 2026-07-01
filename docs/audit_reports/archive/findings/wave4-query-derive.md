## Wave 4 Infra Audit Report — `educore-query-derive`

**Scope:** `crates/infra/query-derive/`, `docs/handoff/PHASE-0-HANDOFF.md`, `docs/build-plan.md` § Phase 0 task 2, `docs/query_layer.md`, `docs/schemas/sql-dialects/README.md`, `docs/guides/test-strategy.md`, `AGENTS.md` (engine rule 2).

**Total findings:** 28

---

### FINDING 1

- **id:** INFRA-QD-001
- **area:** infra
- **severity:** Critical
- **location:** `crates/infra/query-derive/src/lib.rs:634-644`
- **description:** The generic `where_has` method emitted by the macro is a no-op stub. It accepts `relation: __R` and `__build: __F` but discards both (`let _ = relation; let _ = __build;`) and returns `self` unchanged. It does NOT add a `HasRelation` node to the AST. The spec mandates `where_has(StudentRelation::Parent, |p| { p.where_eq(...) })` adds a relational filter to the query.
- **expected:** Per `docs/query_layer.md:179-181` and `docs/query_layer.md:241-245` and `docs/query_layer.md:328-336`: `pub fn where_has<R, F>(self, relation: R, build: F) -> Self where R: Into<StudentRelation>, F: FnOnce(RelatedQueryBuilder<R>) -> RelatedQueryBuilder<R>` — the closure must be invoked with the related builder, the resulting `QueryNode` must be wrapped in `QueryNode::HasRelation` and pushed onto `self.filters`.
- **evidence:**
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

### FINDING 2

- **id:** INFRA-QD-002
- **area:** infra
- **severity:** Critical
- **location:** `crates/infra/query-derive/src/lib.rs` (entire `expand_inner` function, lines 156-844)
- **description:** The macro does NOT emit an `EntityDescriptor { table, columns, indexes, foreign_keys, rls }` struct as `docs/build-plan.md:171-173` and `docs/schemas/sql-dialects/README.md:158-160` mandate. The macro only emits a `*Field` enum, an optional `*Relation` enum, and a `*QueryBuilder` struct — it never reads struct-level attributes (no `table = "..."`, no `aggregate = "..."`), never reads field types (no column-type information), and never carries nullable/defaults/indexes/FKs/RLS as a typed Rust data structure.
- **expected:** Per `docs/build-plan.md:170-172`: "Reads the struct's fields, field types, `#[domain_query(...)]` attributes, and emits an `EntityDescriptor { table, columns, indexes, foreign_keys, rls }`." Per `docs/schemas/sql-dialects/README.md:158-160`: "The macro output is dialect-agnostic — it carries table name, column types, nullable, defaults, indexes, FKs, RLS policies as a typed Rust data structure. No SQL strings."
- **evidence:**
  - `crates/infra/query-derive/src/lib.rs:135-136` `#[proc_macro_derive(DomainQuery, attributes(query))]\npub fn derive_domain_query(input: TokenStream) -> TokenStream {` — only one attribute namespace registered.
  - `crates/infra/query-derive/src/lib.rs:272` `let column = f.name.to_string();` — the only thing the macro reads from each field is its name (as a snake_case Rust identifier). It never reads `f.ty`, never reads struct-level attributes.
  - `grep -n "EntityDescriptor\|f\.field\.ty\|f\.ty" crates/infra/query-derive/src/lib.rs` returns no matches.

---

### FINDING 3

- **id:** INFRA-QD-003
- **area:** infra
- **severity:** Critical
- **location:** `crates/infra/query-derive/src/lib.rs:64` and `docs/build-plan.md:170` vs. `docs/schemas/sql-dialects/README.md:150`
- **description:** The build plan and `docs/schemas/sql-dialects/README.md` mandate that the macro reads struct-level `#[domain_query(table = "...", aggregate = "...")]` attributes. The macro only accepts the field-level `#[query(...)]` attribute namespace (registered at line 135). A user writing `#[domain_query(table = "academic_students")]` per the sql-dialects README example will get an "unknown attribute" error.
- **expected:** Per `docs/build-plan.md:170`: the macro reads "struct's fields, field types, `#[domain_query(...)]` attributes". Per `docs/schemas/sql-dialects/README.md:150`: `#[domain_query(table = "academic_students", aggregate = "Student")]` is the documented struct-level invocation.
- **evidence:**
  - `crates/infra/query-derive/src/lib.rs:135` `#[proc_macro_derive(DomainQuery, attributes(query))]` — only `query` namespace registered, not `domain_query`.
  - `crates/infra/query-derive/src/lib.rs:64` `if attr.path().is_ident("query") {` — only `#[query(...)]` is recognized; `#[domain_query(...)]` is silently ignored (treated as an unrelated attribute).
  - `docs/build-plan.md:170` `Reads the struct's fields, field types, \`#[domain_query(...)]\` attributes`.
  - `docs/schemas/sql-dialects/README.md:150` `#[domain_query(table = "academic_students", aggregate = "Student")]`.

---

### FINDING 4

- **id:** INFRA-QD-004
- **area:** infra
- **severity:** Critical
- **location:** `crates/infra/query-derive/src/lib.rs:156-844` (whole macro emission)
- **description:** The macro emits no `__spec_coverage__` test module. The build plan § Phase 0 task 2 (line 172-173) requires the macro to "Emit a `__spec_coverage__` test module on every `#[derive(DomainQuery)]` (see § The No-Gaps Gates)." No such module is emitted by the macro; downstream crates that `#[derive(DomainQuery)]` therefore do not get per-aggregate coverage tests automatically.
- **expected:** `docs/build-plan.md:172-173`: "Emits a `__spec_coverage__` test module on every `#[derive(DomainQuery)]` (see § The No-Gaps Gates)."
- **evidence:**
  - `grep -n "__spec_coverage__" crates/infra/query-derive/src/lib.rs` returns no matches.
  - `grep -rn "__spec_coverage__" crates/ --include="*.rs"` returns no matches anywhere in the workspace.
  - The trailing comment at `crates/infra/query-derive/src/lib.rs:849-851` `// Test module — verifies the macro emits correct code` describes a module that does not exist in the file (tests are in `tests/derive_test.rs`, but no `mod __spec_coverage__` is emitted to consumers).

---

### FINDING 5

- **id:** INFRA-QD-005
- **area:** infra
- **severity:** Critical
- **location:** `crates/infra/query-derive/src/lib.rs` (no `*Aggregate` enum emitted anywhere)
- **description:** The macro does not emit a `*Aggregate` enum. `docs/query_layer.md:482-498` mandates that the macro emit a `StudentAggregate` enum (with variants `Count`, `Sum`, `Avg`, `Min`, `Max`) alongside the field enum, and that users call `.aggregate(StudentAggregate::Count).group_by(StudentField::ClassId).execute()`. The macro has no aggregation support and no aggregate enum type.
- **expected:** Per `docs/query_layer.md:496-498`: "Aggregations are `Count`, `Sum`, `Avg`, `Min`, `Max` over numeric fields. The macro emits the `StudentAggregate` enum alongside the field enum, ensuring the aggregation set is closed at compile time."
- **evidence:**
  - `grep -n "Aggregate\|aggregate" crates/infra/query-derive/src/lib.rs` returns no matches.
  - `docs/query_layer.md:482-498` mandates the `StudentAggregate` enum and `.aggregate(StudentAggregate::Count)` API.

---

### FINDING 6

- **id:** INFRA-QD-006
- **area:** infra
- **severity:** High
- **location:** `crates/infra/query-derive/src/lib.rs:803-807` and `crates/infra/query-derive/src/lib.rs:406-431`
- **description:** The macro emits a public `pub fn new() -> Self { Self::default() }` and a `#[derive(Default)]` on the builder. Both paths produce a builder with `school_id = None`. The spec at `docs/query_layer.md:518-527` mandates "The default constructor is private. A query that omits the school id is a compile error. ... `StudentQueryBuilder` is constructed only via `StudentQuery::new(school_id)`." The engine rule "Compile-time safety over strings" (AGENTS.md) implies school-id enforcement should be at the type level, not the runtime. The macro's enforcement is a runtime `Err(DomainError::Validation(...))` in `build_query_node()` at lines 777-786.
- **expected:** Per `docs/query_layer.md:520-527`: "The default constructor is private. A query that omits the school id is a compile error."
- **evidence:**
  - `crates/infra/query-derive/src/lib.rs:803-807` `let new_method = quote! { #struct_vis fn new() -> Self { Self::default() } };`
  - `crates/infra/query-derive/src/lib.rs:406-431` (the `#[derive(Default)]` on `*QueryBuilder` makes `Default::default()` callable publicly).
  - `crates/infra/query-derive/src/lib.rs:777-786` (the missing-school-id check is at runtime in `build_query_node`, not at compile time).

---

### FINDING 7

- **id:** INFRA-QD-007
- **area:** infra
- **severity:** High
- **location:** `crates/infra/query-derive/src/lib.rs:203-216` and `crates/infra/query-derive/src/lib.rs:266-296`
- **description:** Both `filterable` and `sortable` decorations merge into the same `*Field` enum without distinction. The macro emits `fn where_eq<V>(mut self, field: StudentField, value: V) -> Self` (line 437) and `fn order_by(mut self, field: StudentField) -> Self` (line 558) with the same field type. A field marked only `#[query(sortable)]` can be passed to `where_eq`, and a field marked only `#[query(filterable)]` can be passed to `order_by`. The spec at `docs/query_layer.md:96-103` says: "`#[query(filterable)]` — Field can be used in a `.where_*` clause; `#[query(sortable)]` — Field can be used in `.order_by(...)`". The compile-time distinction is not enforced.
- **expected:** Per `docs/query_layer.md:96-103` and the AGENTS.md "Compile-time safety over strings" rule, `filterable` and `sortable` should produce disjoint enums (e.g., `StudentFilterField` vs `StudentSortField`) so the compiler rejects `where_eq(SortField::X, ...)` and `order_by(FilterField::Y)`.
- **evidence:**
  - `crates/infra/query-derive/src/lib.rs:203-216` (the `queryable` filter combines `filterable || sortable` into one set).
  - `crates/infra/query-derive/src/lib.rs:437` `fn where_eq<V>(mut self, field: #field_enum_name, value: V) -> Self`.
  - `crates/infra/query-derive/src/lib.rs:558` `fn order_by(mut self, field: #field_enum_name) -> Self`.

---

### FINDING 8

- **id:** INFRA-QD-008
- **area:** infra
- **severity:** High
- **location:** `crates/infra/query-derive/src/lib.rs:333`, `crates/infra/query-derive/src/lib.rs:341`, `crates/infra/query-derive/src/lib.rs:367`, `crates/infra/query-derive/src/lib.rs:604`
- **description:** The macro calls `format_ident!("{relation}")` four times with the raw string from `#[query(relation = "...")]` without validating that the string is a valid Rust identifier. A `relation` value containing non-identifier characters (e.g., `"foo-bar"`, `"foo bar"`, `"123"`, `""`) causes `format_ident!` to panic or emit invalid Rust tokens that surface as a confusing downstream compile error. No defensive check exists.
- **expected:** A clear compile error pointing at the offending field with a message like "relation name `foo-bar` is not a valid Rust identifier" before any token emission.
- **evidence:**
  - `crates/infra/query-derive/src/lib.rs:333` `let variant = format_ident!("{relation}");` (in `relation_variants`).
  - `crates/infra/query-derive/src/lib.rs:341` `let variant = format_ident!("{relation}");` (in `relation_match_arms`).
  - `crates/infra/query-derive/src/lib.rs:367` `let variant = format_ident!("{relation}");` (in `all_relations_slice`).
  - `crates/infra/query-derive/src/lib.rs:604` `let relation_variant = format_ident!("{relation}");` (in `where_has_methods`).

---

### FINDING 9

- **id:** INFRA-QD-009
- **area:** infra
- **severity:** High
- **location:** `crates/infra/query-derive/src/lib.rs:330-336`
- **description:** Two fields with `#[query(relation = "Parent")]` produce duplicate `StudentRelation::Parent` variants in the macro-emitted enum, yielding a confusing Rust compile error "variant `Parent` is already defined". The macro's pre-checks at lines 67-78 only flag duplicate `relation = "..."` on the SAME field, not duplicate `relation` values across two fields. There is no duplicate-relation-name detection.
- **expected:** A compile error at macro expansion time like "duplicate `relation = \"Parent\"` declared on fields `parent_a` and `parent_b`" before token emission.
- **evidence:**
  - `crates/infra/query-derive/src/lib.rs:67-78` — the `parse_field_attrs` function only checks for duplicates within one field's attributes.
  - `crates/infra/query-derive/src/lib.rs:330-336` — the macro iterates relations without deduplicating `relation` strings.
  - `crates/infra/query-derive/src/lib.rs:255-264` — the `seen_queryable` duplicate check only covers the queryable filterable/sortable set, not the relations set.

---

### FINDING 10

- **id:** INFRA-QD-010
- **area:** infra
- **severity:** High
- **location:** `crates/infra/query-derive/src/lib.rs:255-264`
- **description:** The duplicate-field check compares Rust field-name identifiers (`f.name`), not the PascalCase enum variants the macro emits. Two fields whose names map to the same PascalCase form (e.g., `foo_bar` and `FooBar`, or `naïve_field` and `naive_field` after Unicode normalization issues, or simply `foo__bar` and `foo_bar` after the underscore collapsing logic in `pascal_case`) will silently compile at macro-expansion time and then produce a confusing downstream Rust compile error about duplicate enum variants. The check at lines 255-264 is insufficient.
- **expected:** A duplicate-PascalCase-variant check inside the macro with a friendly compile error.
- **evidence:**
  - `crates/infra/query-derive/src/lib.rs:255-264` `let mut seen_queryable: Vec<&Ident> = Vec::new();\nfor f in &queryable { if seen_queryable.iter().any(|i| **i == f.name) { return Err(...) }\n    seen_queryable.push(&f.name);\n}` — checks `f.name` (the Rust ident), not the PascalCase form.
  - `crates/infra/query-derive/src/lib.rs:266-269` — the macro emits `format_ident!("{}", pascal_case(&f.name.to_string()))` for the variant. If two `f.name` values map to the same `pascal_case(...)` result, the emitted enum has duplicate variants.

---

### FINDING 11

- **id:** INFRA-QD-011
- **area:** infra
- **severity:** Medium
- **location:** `crates/infra/query-derive/src/lib.rs:605`
- **description:** The `builder = "..."` value is parsed with `let builder_ty: Ident = syn::parse_str(builder)?;`. `syn::parse_str::<Ident>` only accepts a single bare identifier; a value like `"crate::module::ParentQueryBuilder"` (a path) fails with a confusing error "expected ident". The macro accepts the attribute but does not validate that the string is a syntactically valid Rust type path, and does not validate that the named type actually exists in the caller's scope.
- **expected:** A friendly compile error like "builder `crate::module::X` is not a valid type path" or "builder type `X` is not in scope at this derive site".
- **evidence:**
  - `crates/infra/query-derive/src/lib.rs:605` `let builder_ty: Ident = syn::parse_str(builder)?;` — `Ident` rejects paths.
  - `crates/infra/query-derive/src/lib.rs:73-78` — the duplicate-builder check inside `parse_field_attrs` only checks for duplicate attributes on one field, not path syntax.

---

### FINDING 12

- **id:** INFRA-QD-012
- **area:** infra
- **severity:** Medium
- **location:** `crates/infra/query-derive/src/lib.rs:91-105`
- **description:** `pascal_case` does not validate that the result is a valid Rust identifier. A field whose name is empty (`""`) yields an empty string; a name with leading digits (`"2nd_field"`) yields `"2ndField"` (invalid Rust — identifiers cannot start with a digit); a name with non-ASCII characters is passed through unchanged; a name with consecutive underscores (`"foo__bar"`) and `foo_bar` both yield the same PascalCase form (`FooBar`), causing the collision in Finding INFRA-QD-010. The function has no unit tests and no callers in the workspace that exercise corner cases.
- **expected:** `pascal_case` should produce only valid Rust identifiers (reject empty input, reject names that start with a digit, normalize non-ASCII) and the macro should detect collisions between field names that map to the same PascalCase form.
- **evidence:** `crates/infra/query-derive/src/lib.rs:91-105` `fn pascal_case(s: &str) -> String { let mut out = String::with_capacity(s.len()); let mut at_word_start = true; for ch in s.chars() { if ch == '_' { at_word_start = true; } else if at_word_start { out.extend(ch.to_uppercase()); at_word_start = false; } else { out.push(ch); } } out }` — no validation, no Rust-identifier check.

---

### FINDING 13

- **id:** INFRA-QD-013
- **area:** infra
- **severity:** Medium
- **location:** `crates/infra/query-derive/src/lib.rs:729-748` (the `if queryable.is_empty()` else-branch)
- **description:** When the user calls `.build_query_node()` with no filters, the macro emits `QueryNode::And(Box::new(QueryNode::IsNull(first_variant)), Box::new(QueryNode::IsNotNull(first_variant)))`. This is a never-satisfiable predicate: `IS NULL AND IS NOT NULL` for the same field. A user expecting "no filters means all rows" gets zero rows. There is no documentation of this behavior in the spec, and `docs/query_layer.md` does not describe this degenerate-tree shape.
- **expected:** Either (a) an empty-filter build returns `QueryNode::True` (a documented "match-all" sentinel) or (b) `build_query_node()` errors out with a clear `DomainError::Validation` explaining that no filter was added. The current behavior silently returns zero rows.
- **evidence:** `crates/infra/query-derive/src/lib.rs:729-748` `::std::option::Option::None => { let first_variant = #field_enum_name::all_variants()[0]; ::educore_core::query::QueryNode::And(::std::boxed::Box::new(::educore_core::query::QueryNode::IsNull(first_variant)), ::std::boxed::Box::new(::educore_core::query::QueryNode::IsNotNull(first_variant)),) }`.

---

### FINDING 14

- **id:** INFRA-QD-014
- **area:** infra
- **severity:** Medium
- **location:** `crates/infra/query-derive/src/lib.rs:671-713` (the `queryable.is_empty()` branch)
- **description:** When the struct has only relations and no queryable fields (e.g., `Bookmark` in `tests/derive_test.rs:57-62`), `__educore_compile` emits `And(HasRelation(sentinel, IsNull), HasRelation(sentinel, IsNotNull))` with `Relation { id: 0, name: "" }` as a sentinel. The comment at lines 673-686 admits this is a workaround. There is no compile-time guard preventing the user from forgetting to call `where_has_<Relation>(...)` before `build_query_node()`. The only test for this path (`no_relations_struct_compiles` at `tests/derive_test.rs:204-213`) verifies only that the macro compiles, not the AST shape.
- **expected:** A compile-time mechanism (a builder state type or a phantom `relations_present: bool` token) that prevents `build_query_node()` from being called without at least one `where_has_*` filter on a no-fields struct.
- **evidence:** `crates/infra/query-derive/src/lib.rs:687-712` emits the sentinel `HasRelation(_, IsNull) AND HasRelation(_, IsNotNull)` with `Relation { id: 0, name: "" }`. `crates/infra/query-derive/tests/derive_test.rs:204-213` does not assert the AST shape.

---

### FINDING 15

- **id:** INFRA-QD-015
- **area:** infra
- **severity:** Medium
- **location:** `crates/infra/query-derive/src/lib.rs:791`
- **description:** `let limit = self.limit.unwrap_or(50);` hardcodes a default page limit of 50. No spec mandates this value, and no constant is exposed for callers to override or document. The magic number is buried inside the macro emission.
- **expected:** Either a documented constant `DEFAULT_PAGE_LIMIT: u32 = 50` exposed from `educore-core`, or the spec to mandate the value.
- **evidence:** `crates/infra/query-derive/src/lib.rs:791` `let limit = self.limit.unwrap_or(50);`. `grep -rn "limit.*50\|default.*50" docs/query_layer.md docs/build-plan.md` returns no matching spec mandates.

---

### FINDING 16

- **id:** INFRA-QD-016
- **area:** infra
- **severity:** Medium
- **location:** `crates/infra/query-derive/src/lib.rs:276-280`
- **description:** `_field_to_variant_arms` is computed via `queryable.iter().map(...).collect()` and then immediately bound to `_` (line 280) and never used. This is dead code; the `let _` prefix suppresses the unused-variable warning but leaves the computation in the source. The map closure at lines 277-279 references `Self::#ident => #field_enum_name::#variant` — an inverse-direction mapping that the macro never emits anywhere.
- **expected:** Remove the dead code or wire it into an emitted method.
- **evidence:** `crates/infra/query-derive/src/lib.rs:276-280` `let _field_to_variant_arms = queryable.iter().map(|f| { let variant = format_ident!("{}", pascal_case(&f.name.to_string())); let ident = &f.name; quote! { Self::#ident => #field_enum_name::#variant } });` — `let _` prefix and no downstream use.

---

### FINDING 17

- **id:** INFRA-QD-017
- **area:** infra
- **severity:** Medium
- **location:** `crates/infra/query-derive/src/lib.rs:849-851` and the file's tail
- **description:** The file ends with a comment `// Test module — verifies the macro emits correct code` followed by a closing divider, but no test module follows. The comment is misleading: tests live in `tests/derive_test.rs`, not inside the source file. There is no `#[cfg(test)] mod tests { ... }` block.
- **expected:** Either remove the comment or move the test scaffolding inline.
- **evidence:** `crates/infra/query-derive/src/lib.rs:849-851` `// ============================================================================\n// Test module — verifies the macro emits correct code\n// ============================================================================` — no test code follows; the file ends at line 851.

---

### FINDING 18

- **id:** INFRA-QD-018
- **area:** infra
- **severity:** Medium
- **location:** `crates/infra/query-derive/src/lib.rs:157-158` and the macro emission on generic structs
- **description:** The macro does not handle generic structs. `let struct_name = input.ident.clone();` (line 157) ignores generics. A user writing `#[derive(DomainQuery)] pub struct Foo<T> { pub id: Uuid, #[query(filterable)] pub value: T }` would get an emission with unresolved type parameter `T` in the generated builder, the builder methods (`where_eq`, etc.), and the `*Field` enum. The downstream compile error would surface as `T` not in scope.
- **expected:** Either (a) the macro rejects generic structs with a clear "DomainQuery does not support generic structs" compile error, or (b) the macro propagates generics through to the emitted types (significantly more work).
- **evidence:** `crates/infra/query-derive/src/lib.rs:157-158` `let struct_name = input.ident.clone(); let struct_vis = input.vis.clone();` — no `input.generics` handling.

---

### FINDING 19

- **id:** INFRA-QD-019
- **area:** infra
- **severity:** Medium
- **location:** `crates/infra/query-derive/src/lib.rs:73-78` and `crates/infra/query-derive/src/lib.rs:243-253`
- **description:** `#[query(builder = "X")]` alone (with no `relation = "..."`) is silently accepted and then ignored. `parse_field_attrs` merges a `builder` attribute on its own without producing a compile error or warning. The `relations` filter at lines 223-230 requires both `relation` and `builder` to be `Some`, so a field with only `builder` never appears in the relations set. The `builder` value is silently wasted.
- **expected:** A compile error like "field has `builder = \"X\"` without `relation = \"...\"`" or "builder attribute requires relation attribute".
- **evidence:**
  - `crates/infra/query-derive/src/lib.rs:73-78` only checks for duplicate `builder` values on the same field.
  - `crates/infra/query-derive/src/lib.rs:223-230` filters with `relation.as_deref()?` first, so `builder` is silently skipped if `relation` is `None`.
  - `crates/infra/query-derive/src/lib.rs:243-253` checks for `relation` without `builder` but not the inverse.

---

### FINDING 20

- **id:** INFRA-QD-020
- **area:** infra
- **severity:** High
- **location:** `crates/infra/query-derive/src/lib.rs` (zero `#[derive(DomainQuery)]` outside of its own tests)
- **description:** The macro has zero adoption. `grep -rn "#\[derive(DomainQuery)\]" crates/` finds the attribute only in `crates/infra/query-derive/tests/derive_test.rs` (lines 26, 38, 45, 57). No domain crate, no cross-cutting crate, and no adapter uses it. PHASE-0-HANDOFF.md:38-39 acknowledges this. The macro is shipped but unproven: it has no real-world domain aggregate to validate against, no field-type interplay to handle (because the spec mandates column types but the macro doesn't read them), and no storage adapter consumes its output (the macro emits no `EntityDescriptor`, so no adapter can walk the AST to emit DDL).
- **expected:** Per `docs/build-plan.md` and `AGENTS.md`, the macro is the foundation of the typed query layer; every `tables.md` row is supposed to be backed by a `#[derive(DomainQuery)]` struct. Per the wave-1 audit reports (e.g., `wave1-academic.md`, `wave1-assessment.md`, `wave1-cms.md`), all 10 domain crates ship without a single `#[derive(DomainQuery)]` derive. Per `docs/handoff/PHASE-0-HANDOFF.md:38-39`: "The `#[derive(DomainQuery)]` macro is real but not yet used by any domain crate. Its tests are the proof of life."
- **evidence:** `grep -rn "#\[derive(DomainQuery)\]" crates/` returns 4 matches, all in `crates/infra/query-derive/tests/derive_test.rs`.

---

### FINDING 21

- **id:** INFRA-QD-021
- **area:** infra
- **severity:** Medium
- **location:** `crates/infra/query-derive/tests/derive_test.rs` (entire file, 233 lines, 19 tests)
- **description:** Test coverage is incomplete. Missing test cases:
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
- **expected:** Each documented and undocumented macro-input shape has at least one integration test verifying the compile error or the emitted AST shape. `docs/guides/test-strategy.md:225-238` mandates snapshot tests.
- **evidence:** `crates/infra/query-derive/tests/derive_test.rs:64-232` contains 19 tests, none of which exercise the listed edge cases. `grep -c "^#\[test\]" crates/infra/query-derive/tests/derive_test.rs` returns 19.

---

### FINDING 22

- **id:** INFRA-QD-022
- **area:** infra
- **severity:** Low
- **location:** `crates/infra/query-derive/src/lib.rs:271-274`
- **description:** The `Field` impl emitted by the macro returns column names that are the snake_case Rust field names (e.g., `last_name`). There is no mechanism for a user to override the DB column name (no `#[query(column = "lastName")]` attribute or similar). A user whose DB schema has a different naming convention (CamelCase, custom abbreviation, prefix) cannot express it without renaming the Rust field, which fights the engine's "compile-time safety over strings" rule (the field enum and the DB column must stay in lock-step via a single source of truth).
- **expected:** A `#[query(column = "actual_db_column_name")]` attribute per field that overrides the default Rust-field-name column name.
- **evidence:** `crates/infra/query-derive/src/lib.rs:272` `let column = f.name.to_string();` — column name is always the Rust field name; no override path.

---

### FINDING 23

- **id:** INFRA-QD-023
- **area:** infra
- **severity:** Low
- **location:** `crates/infra/query-derive/src/lib.rs:714-750` (`compile_method`'s happy-path branch)
- **description:** When the user adds at least one filter, `__educore_compile` folds them left-to-right with `iter.fold(first, |acc, next| QueryNode::And(Box::new(acc), Box::new(next)))`. This produces a left-leaning tree of nested `And` nodes, which has linear depth for N filters. The spec at `docs/query_layer.md:564-566` calls for "sized for the common case (zero to four predicates)" — a tree of depth N is acceptable for small N but is non-ideal for queries with many predicates. No depth-balancing or pairwise fold is emitted.
- **expected:** A balanced tree shape (or a `QueryNode::AndN(Vec<QueryNode<F>>)` variant) so depth is `O(log N)`.
- **evidence:** `crates/infra/query-derive/src/lib.rs:740-745` `iter.fold(first, |acc, next| { ::educore_core::query::QueryNode::And(::std::boxed::Box::new(acc), ::std::boxed::Box::new(next)) })` — left fold, linear depth.

---

### FINDING 24

- **id:** INFRA-QD-024
- **area:** infra
- **severity:** Low
- **location:** `crates/infra/query-derive/src/lib.rs:152-155`
- **description:** The `expand_inner` inner fn has `#[allow(clippy::too_many_lines, clippy::similar_names, clippy::needless_pass_by_value, clippy::needless_borrow)]` to suppress four lint advisories. Per `docs/code-standards.md` and AGENTS.md, lint suppressions are discouraged unless justified. The `too_many_lines` and `similar_names` are symptoms of the macro's monolithic single-function structure; refactoring into helpers (parse-fields, build-enum, build-builder, build-impls) would remove the need for the suppression.
- **expected:** Decompose the macro emission into focused helper functions so each function is short enough that no `clippy::too_many_lines` suppression is needed.
- **evidence:** `crates/infra/query-derive/src/lib.rs:150-155` `#[allow(\n    clippy::too_many_lines,\n    clippy::similar_names,\n    clippy::needless_pass_by_value,\n    clippy::needless_borrow\n)]\nfn expand_inner(input: DeriveInput) -> syn::Result<TokenStream2> {`.

---

### FINDING 25

- **id:** INFRA-QD-025
- **area:** infra
- **severity:** Low
- **location:** `crates/infra/query-derive/src/lib.rs:64` vs. `crates/infra/query-derive/src/lib.rs:135`
- **description:** The proc-macro registers `attributes(query)` (line 135) but `parse_field_attrs` filters on `attr.path().is_ident("query")` (line 64). The single-attribute register means helper attributes (e.g., `#[query(column = "...")]` if added per Finding INFRA-QD-022) must all live in the `query` namespace. The macro never registers or recognizes a struct-level attribute namespace (e.g., `attributes(domain_query)`), so the `#[domain_query(...)]` struct-level form per Finding INFRA-QD-003 cannot be added without a breaking change to the registration.
- **expected:** Two namespace registration: `attributes(query)` for field-level and `attributes(domain_query)` for struct-level.
- **evidence:**
  - `crates/infra/query-derive/src/lib.rs:135` `#[proc_macro_derive(DomainQuery, attributes(query))]`.
  - `crates/infra/query-derive/src/lib.rs:64` `if attr.path().is_ident("query") {`.

---

### FINDING 26

- **id:** INFRA-QD-026
- **area:** infra
- **severity:** Low
- **location:** `crates/infra/query-derive/tests/derive_test.rs:184-191`
- **description:** `where_has_typed_method_compiles` verifies only that `where_has_Parent(|p| p.where_eq(...))` compiles. It does not assert the resulting AST is `QueryNode::HasRelation(StudentRelation::Parent, ...)`. The closure is invoked; the result is bound to `_b` and discarded. A regression that silently turns `where_has_*` into a no-op (analogous to Finding INFRA-QD-001 for the generic form) would not be caught.
- **expected:** A test that calls `build_query_node()` and asserts the inner AST contains `QueryNode::HasRelation(StudentRelation::Parent, ...)` with the expected child predicate.
- **evidence:** `crates/infra/query-derive/tests/derive_test.rs:184-191` `#[test]\nfn where_has_typed_method_compiles() {\n    let g = SystemIdGen;\n    let school = g.next_school_id();\n    let _b = StudentQueryBuilder::new()\n        .for_school(school)\n        .where_has_Parent(|p| p.where_eq(ParentField::City, "Boston"));\n}` — the resulting builder is bound to `_b` and discarded without AST inspection.

---

### FINDING 27

- **id:** INFRA-QD-027
- **area:** infra
- **severity:** Low
- **location:** `crates/infra/query-derive/src/lib.rs:109-134` (the rustdoc example)
- **description:** The rustdoc example for `derive_domain_query` uses `StudentStatus` as the type of `status` (line 127). `StudentStatus` is not defined in the macro crate, not imported via the `prelude::*` glob on line 117, and not in the `educore_core` crate. The example uses `rust,ignore` to suppress compile failure of the example, but it suggests an API that doesn't actually work (a user copying the example into their code will get an unresolved-type error for `StudentStatus`).
- **expected:** Either (a) define `StudentStatus` in the example prelude, or (b) use a primitive type (e.g., `String`) so the example is self-contained, or (c) drop the `rust,ignore` and make the example actually compile-tested by doc-tests.
- **evidence:** `crates/infra/query-derive/src/lib.rs:115-131` `/// ```rust,ignore\n/// use educore_query_derive::DomainQuery;\n/// use educore_core::prelude::*;\n///\n/// #[derive(DomainQuery)]\n/// pub struct Student {\n///     pub id: Uuid,\n///\n///     #[query(sortable)]\n///     pub last_name: String,\n///\n///     #[query(filterable)]\n///     pub status: StudentStatus,\n///\n///     #[query(filterable, relation = \"Parent\", builder = \"ParentQueryBuilder\")]\n///     pub parent_id: Uuid,\n/// }\n/// ```\n///` — `StudentStatus` and `ParentQueryBuilder` are undefined.

---

### FINDING 28

- **id:** INFRA-QD-028
- **area:** infra
- **severity:** Low
- **location:** `crates/infra/query-derive/src/lib.rs:155` (entry of `expand_inner`) and the entire macro emission
- **description:** The macro does not emit any reference to the originating struct's documentation or module path. Downstream crates cannot programmatically map a `*Field` enum back to the source struct or source file (the macro hardcodes `stringify!(#builder_name)` in error messages, but never captures `Span::call_site` or `struct_name.span()` for richer diagnostics). The `Span` API in `syn` 2.x supports `.resolved_at()` for cross-crate span tracking.
- **expected:** Span metadata on the emitted items so `rustc` can point at the original `#[derive(DomainQuery)]` invocation when downstream compilation fails.
- **evidence:** `crates/infra/query-derive/src/lib.rs:781-784` `concat!(\n    stringify!(#builder_name),\n    \" requires for_school() before build_query_node()\"\n)` — the error message uses only the builder name, not the source span.

---

### END FINDINGS

Total findings: 28
