//! # educore-core::lint
//!
//! Build-time enforcer for the engine's spec/code/anti-pattern
//! invariants. Per `docs/build-plan.md` § "The No-Gaps Gates":
//!
//! 1. **Spec → code direction:** every `docs/specs/<domain>/tables.md`
//!    row has a corresponding `#[derive(DomainQuery)]` struct in
//!    `crates/domains/<domain>/src/aggregate.rs` (matched by table
//!    name). Every `docs/commands/<domain>.md` entry has a
//!    corresponding handler in `crates/domains/<domain>/src/commands.rs`.
//!    Every `docs/events/<domain>.md` entry has a corresponding
//!    event in `crates/domains/<domain>/src/events.rs`.
//! 2. **Code → spec direction:** every public struct, command, and
//!    event has a spec row. The lint fails on undocumented public
//!    items.
//! 3. **Anti-patterns:** no `unwrap`/`expect`/`panic`/`todo!` in
//!    production code, no `as` casts on numerics, no
//!    `serde_json::Value` in domain code, no `HashMap<String, T>`
//!    for domain data.
//! 4. **Parity:** every `DomainQuery` macro call has a
//!    corresponding spec row, and every spec row has a
//!    corresponding macro call.
//! 5. **Coverage matrix sync:** the lint reads `docs/coverage.toml`
//!    and verifies that every `Tested` row has a `tests` path that
//!    exists on disk.
//!
//! The full set of checks lives in the [`runner`] module and is
//! invoked by the `lint` binary (`src/bin/lint.rs`).
//!
//! ## Feature flag
//!
//! The sub-module is gated behind the `lint` Cargo feature so the
//! release build does not pull in `walkdir` or `toml` deps. The
//! consumer-facing binary is:
//!
//! ```text
//! cargo run -p educore-core --bin lint --features lint
//! ```
//!
//! Exit code is `0` on success, `1` on any violation. The
//! `tools/scripts/check-graph-freshness.sh` script invokes the same
//! binary as part of the no-gaps gate (item 4 in the build plan).

#![cfg(feature = "lint")]

use std::path::{Path, PathBuf};

/// A single lint violation. Carries enough context for the caller
/// to print a useful message and pinpoint the offending file.
#[derive(Debug, Clone)]
pub struct Violation {
    /// Short identifier of the check that fired (e.g. `"unwrap_in_prod"`,
    /// `"missing_tests_path"`).
    pub check: String,
    /// The file the violation was found in (relative to the repo root).
    pub file: PathBuf,
    /// 1-indexed line number, when the check can pinpoint one.
    pub line: Option<usize>,
    /// Human-readable description of the violation.
    pub message: String,
}

/// Aggregated report of all violations found by the lint runner.
#[derive(Debug, Default)]
pub struct LintReport {
    /// Every violation found across the workspace.
    pub violations: Vec<Violation>,
}

impl LintReport {
    /// Returns `true` when the runner found no violations.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }

    /// Prints the report to stderr in a stable, parseable format.
    /// Each line is `<check>\t<file>:<line>\t<message>`. Returns the
    /// number of violations printed.
    pub fn print_to_stderr(&self) -> usize {
        for v in &self.violations {
            let line = v.line.map_or(0, std::convert::Into::into);
            eprintln!("{}\t{}:{}\t{}", v.check, v.file.display(), line, v.message);
        }
        self.violations.len()
    }
}

/// Walks the workspace and returns every [`Violation`] it finds.
///
/// `repo_root` is the absolute path to the Educore repo root (the
/// directory that contains `Cargo.toml` and `docs/`). The function
/// is pure: it does not modify the filesystem.
///
/// Only the spec → code direction (item 1 of the No-Gaps Gates) is
/// fully implemented. The remaining checks
/// (`check_coverage_matrix`, `check_anti_patterns`) are best-effort
/// scanners that catch the common cases but leave the hard problems
/// to follow-up PRs; see `docs/audit_reports/findings/wave4-core.md`
/// findings CORE-001..CORE-006 for the open work.
pub fn run(repo_root: &Path) -> LintReport {
    let mut report = LintReport::default();
    runner::check_spec_to_code(repo_root, &mut report);
    runner::check_code_to_spec(repo_root, &mut report);
    runner::check_parity(repo_root, &mut report);
    runner::check_coverage_matrix(repo_root, &mut report);
    runner::check_anti_patterns(repo_root, &mut report);
    report
}

/// The check implementations, grouped by the no-gates gate they
/// belong to. Each function appends to `report`.
pub mod runner {
    use std::fs;
    use std::path::Path;

    use super::{LintReport, Violation};

    /// Spec → code direction (item 1 of the No-Gaps Gates). Walks
    /// `docs/specs/<domain>/{aggregates,commands,events}.md` and
    /// verifies every aggregate, command, and event declared in the
    /// spec has a corresponding definition in the matching
    /// `crates/domains/<domain>/src/*.rs` file.
    ///
    /// The three sub-checks use slightly different strategies because
    /// the spec format varies across domains:
    ///
    /// - `aggregates.md` is consistent across all 10 domains: every
    ///   aggregate is introduced by an `## <Name>` heading, and the
    ///   aggregate itself is described in prose (no `pub struct`
    ///   block). We extract the `## <Name>` headings.
    /// - `commands.md` is less consistent: some headings bundle
    ///   multiple commands (`## CreateClass / UpdateClass /
    ///   DeleteClass`). The robust strategy is to extract
    ///   `pub struct <Name>Command` declarations from the markdown
    ///   directly, which gives one entry per command regardless of
    ///   how the headings were split.
    /// - `events.md` mixes `## <Category>` (e.g. "## Exam Lifecycle")
    ///   and `### <EventName>` (e.g. "### StudentAdmitted") headings
    ///   across the 10 domains. We extract `pub struct <Name>`
    ///   declarations from the body that follows the first
    ///   `## <Category>` heading, which skips the prelude code
    ///   block (containing `EventEnvelope<E>` and friends) and
    ///   yields one entry per event.
    pub fn check_spec_to_code(repo_root: &Path, report: &mut LintReport) {
        let specs_dir = repo_root.join("docs").join("specs");
        let domains_dir = repo_root.join("crates").join("domains");
        let Ok(specs_entries) = fs::read_dir(&specs_dir) else {
            return;
        };
        for spec_entry in specs_entries.flatten() {
            let domain_path = spec_entry.path();
            if !domain_path.is_dir() {
                continue;
            }
            let domain_name = match spec_entry.file_name().into_string() {
                Ok(s) => s,
                Err(_) => continue,
            };
            let code_root = domains_dir.join(&domain_name);
            if !code_root.join("src").is_dir() {
                // No Rust crate for this spec; the engine has
                // cross-cutting specs (events, operations, platform,
                // rbac, settings, sync) that live under `docs/specs/`
                // but not under `crates/domains/`. They are out of
                // scope for the spec → code direction check.
                continue;
            }
            check_aggregates(&domain_path, &code_root, repo_root, report);
            check_commands(&domain_path, &code_root, repo_root, report);
            check_events(&domain_path, &code_root, repo_root, report);
        }
    }

    fn check_aggregates(
        domain_path: &Path,
        code_root: &Path,
        repo_root: &Path,
        report: &mut LintReport,
    ) {
        let spec_file = domain_path.join("aggregates.md");
        let src_file = code_root.join("src").join("aggregate.rs");
        let (Ok(spec), Ok(src)) = (fs::read_to_string(&spec_file), fs::read_to_string(&src_file)) else {
            return;
        };
        for agg in extract_h2_headings(&spec) {
            if !item_defined(&src, "struct", &agg) {
                report.violations.push(missing_spec_item(
                    repo_root,
                    &spec_file,
                    "spec_to_code:missing_aggregate",
                    "aggregate",
                    &agg,
                    &format!("pub struct {agg}"),
                    &src_file,
                ));
            }
        }
    }

    fn check_commands(
        domain_path: &Path,
        code_root: &Path,
        repo_root: &Path,
        report: &mut LintReport,
    ) {
        let spec_file = domain_path.join("commands.md");
        let src_file = code_root.join("src").join("commands.rs");
        let (Ok(spec), Ok(src)) = (fs::read_to_string(&spec_file), fs::read_to_string(&src_file)) else {
            return;
        };
        for cmd in extract_struct_decls_after_first_heading(&spec, "Command") {
            let expected = format!("{cmd}Command");
            if !item_defined(&src, "struct", &expected) {
                report.violations.push(missing_spec_item(
                    repo_root,
                    &spec_file,
                    "spec_to_code:missing_command",
                    "command",
                    &cmd,
                    &format!("pub struct {expected}"),
                    &src_file,
                ));
            }
        }
    }

    fn check_events(
        domain_path: &Path,
        code_root: &Path,
        repo_root: &Path,
        report: &mut LintReport,
    ) {
        let spec_file = domain_path.join("events.md");
        let src_file = code_root.join("src").join("events.rs");
        let (Ok(spec), Ok(src)) = (fs::read_to_string(&spec_file), fs::read_to_string(&src_file)) else {
            return;
        };
        for evt in extract_event_struct_decls(&spec) {
            if !item_defined(&src, "struct", &evt) {
                report.violations.push(missing_spec_item(
                    repo_root,
                    &spec_file,
                    "spec_to_code:missing_event",
                    "event",
                    &evt,
                    &format!("pub struct {evt}"),
                    &src_file,
                ));
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn missing_spec_item(
        repo_root: &Path,
        spec_file: &Path,
        check: &str,
        kind: &str,
        name: &str,
        expected_signature: &str,
        src_file: &Path,
    ) -> Violation {
        Violation {
            check: check.to_string(),
            file: spec_file.strip_prefix(repo_root).unwrap_or(spec_file).to_path_buf(),
            line: None,
            message: format!(
                "{kind} `{name}` declared in spec but missing `{expected_signature}` in {}",
                src_file.display()
            ),
        }
    }

    /// Returns every `## <Name>` heading in `md`, skipping lines
    /// inside fenced code blocks. The document title (`# <Title>`)
    /// is naturally excluded because it is an h1.
    fn extract_h2_headings(md: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut in_code = false;
        for line in md.lines() {
            if is_code_fence(line) {
                in_code = !in_code;
                continue;
            }
            if in_code {
                continue;
            }
            if let Some(rest) = line.strip_prefix("## ") {
                let name = sanitize_heading_text(rest);
                if !name.is_empty() {
                    out.push(name);
                }
            }
        }
        out
    }

    /// Returns every `## <Name>` heading in `md` whose heading text
    /// mentions a `<Suffix>` struct declaration in the body that
    /// follows. The body is the markdown text after the heading;
    /// we extract `pub struct <Name><Suffix>` declarations from it.
    ///
    /// This is more robust than parsing the heading text alone
    /// because it tolerates bundled headings (`## A / B / C`) and
    /// helper structs that the heading does not name.
    fn extract_struct_decls_after_first_heading(md: &str, suffix: &str) -> Vec<String> {
        let mut in_code = false;
        let mut past_first_h2 = false;
        let mut out = Vec::new();
        for line in md.lines() {
            if is_code_fence(line) {
                in_code = !in_code;
                continue;
            }
            if !in_code && line.starts_with("## ") {
                past_first_h2 = true;
            }
            if !past_first_h2 {
                continue;
            }
            for name in extract_struct_names(line, suffix) {
                if !out.contains(&name) {
                    out.push(name);
                }
            }
        }
        out
    }

    /// Returns every event `pub struct <Name>` declaration found in
    /// `md` after the first `## <Category>` heading. This skips the
    /// prelude code block (containing `EventEnvelope<E>` and similar
    /// non-event types) and tolerates both h2-only and h2+h3 heading
    /// styles across the 10 domain specs.
    fn extract_event_struct_decls(md: &str) -> Vec<String> {
        let mut in_code = false;
        let mut past_first_h2 = false;
        let mut out = Vec::new();
        for line in md.lines() {
            if is_code_fence(line) {
                in_code = !in_code;
                continue;
            }
            if !in_code && line.starts_with("## ") {
                past_first_h2 = true;
                continue;
            }
            if !past_first_h2 {
                continue;
            }
            for name in extract_struct_names(line, "") {
                if !out.contains(&name) {
                    out.push(name);
                }
            }
        }
        out
    }

    /// Strips backticks, trims whitespace, and removes trailing
    /// punctuation from a heading line. Headings like
    /// `## Student Lifecycle` return `"Student Lifecycle"`.
    fn sanitize_heading_text(raw: &str) -> String {
        raw.trim()
            .trim_matches('`')
            .trim_end_matches(':')
            .trim()
            .to_string()
    }

    fn is_code_fence(line: &str) -> bool {
        line.trim_start().starts_with("```")
    }

    /// Returns the names of any `pub struct <FullName>`
    /// declarations found on `line`, with the trailing `suffix`
    /// stripped from `FullName` if non-empty. For commands
    /// (`suffix = "Command"`), `pub struct CreateWidgetCommand {}`
    /// yields `"CreateWidget"`. For events (`suffix = ""`),
    /// `pub struct StudentAdmitted {}` yields `"StudentAdmitted"`.
    ///
    /// A line may declare multiple structs; all are returned.
    /// The delimiter check at the end of the name and the
    /// `strip_suffix` boundary check together prevent collisions
    /// with unrelated identifiers (e.g. `CommandHistory` does NOT
    /// match the `"Command"` suffix because it does not end with it).
    fn extract_struct_names(line: &str, suffix: &str) -> Vec<String> {
        let trimmed = line.trim_start();
        let Some(after_pub) = trimmed.strip_prefix("pub struct ") else {
            return Vec::new();
        };
        // The full name extends from the start of `after_pub` to
        // the first delimiter character.
        let name_end = after_pub
            .find(|c: char| matches!(c, ' ' | '{' | '<' | '(' | ';'))
            .unwrap_or(after_pub.len());
        if name_end == 0 {
            return Vec::new();
        }
        let full_name = &after_pub[..name_end];
        if suffix.is_empty() {
            return vec![full_name.to_string()];
        }
        // For non-empty suffix, the full name MUST end with the
        // suffix for this to be a match. This naturally rejects
        // coincidental matches (e.g. `pub struct CommandHistory`
        // does not match `suffix = "Command"`).
        let Some(bare) = full_name.strip_suffix(suffix) else {
            return Vec::new();
        };
        if bare.is_empty() {
            // The full name is exactly the suffix with no preceding
            // identifier (e.g. `pub struct Command`); not a valid
            // command name.
            return Vec::new();
        }
        vec![bare.to_string()]
    }

    /// Returns `true` if `src` contains `pub <kind> <name>` where
    /// the next character after the name is a delimiter (space,
    /// brace, generic, tuple, or semicolon). Accepts `struct` and
    /// `enum`.
    fn item_defined(src: &str, kind: &str, name: &str) -> bool {
        let prefix = format!("pub {kind} {name}");
        for line in src.lines() {
            let trimmed = line.trim_start();
            let Some(after) = trimmed.strip_prefix(&prefix) else {
                continue;
            };
            match after.chars().next() {
                Some(' ') | Some('{') | Some('<') | Some('(') | Some(';') => return true,
                _ => continue,
            }
        }
        false
    }

    /// Code → spec direction (item 2 of the No-Gaps Gates, second
    /// bullet). Walks every domain crate's source files and
    /// verifies that each public `struct` and `enum` has a
    /// matching row in the corresponding spec file.
    ///
    /// Mapping from source file to spec file:
    ///
    /// | Source file      | Spec file                       | Match strategy               |
    /// | ---------------- | ------------------------------- | ---------------------------- |
    /// | `aggregate.rs`   | `docs/specs/<d>/aggregates.md`  | `## <Name>` h2 heading       |
    /// | `commands.rs`    | `docs/specs/<d>/commands.md`    | `## <Name>` or `pub struct <Name>Command` |
    /// | `events.rs`      | `docs/specs/<d>/events.md`      | `pub struct <Name>` decl     |
    /// | `services.rs`    | `docs/specs/<d>/services.md`    | `## <Name>` or `pub struct <Name>` |
    /// | `value_objects.rs` | `docs/specs/<d>/value-objects.md` | `pub struct/enum <Name>` decl OR table row `| \`<Name>\` |` |
    /// | `entities.rs`    | `docs/specs/<d>/entities.md`    | `## <Name>` h2 heading       |
    ///
    /// The check deliberately skips `lib.rs` (re-exports only),
    /// `tests/`, and `examples/`. `pub use ...` re-exports
    /// inside the listed files are also skipped because the
    /// originating crate is the source of truth.
    ///
    /// **Known false positives** (intentionally not addressed by
    /// the simple regex scan; left for the full Rust parser
    /// follow-up):
    ///
    /// - Phantom types like `pub struct PhantomData<T>` are
    ///   rarely public but, if they are, will trigger the check.
    /// - Marker structs used only inside `mod tests` are exempt
    ///   via the `#[cfg(test)]` block scanner from the
    ///   anti-pattern check, but `check_code_to_spec` does not
    ///   honour that exemption (test-only types should not be
    ///   `pub` in the first place).
    /// - Aggregate child structs (`NewPage`, `UpdatePage`,
    ///   `PageRevision`) typically appear inside `aggregate.rs`
    ///   but their spec lives in `aggregates.md` as a section
    ///   under the parent aggregate. The simple `## <Name>`
    ///   match misses them, which produces violations for
    ///   legitimate child types. The full parser will read the
    ///   spec's "Owned Children" table.
    pub fn check_code_to_spec(repo_root: &Path, report: &mut LintReport) {
        let domains_dir = repo_root.join("crates").join("domains");
        let specs_dir = repo_root.join("docs").join("specs");
        let Ok(entries) = fs::read_dir(&domains_dir) else {
            return;
        };
        // Per-file scan plan. The first tuple field is the source
        // file name (relative to the domain crate's `src/`
        // directory), the second is the corresponding spec file
        // (relative to `docs/specs/<domain>/`), and the third is
        // the matcher function that decides whether the public
        // type has a corresponding spec row.
        type Matcher = dyn Fn(&str, &str) -> bool;
        let plans: Vec<(&str, &str, Box<Matcher>)> = vec![
            (
                "aggregate.rs",
                "aggregates.md",
                Box::new(|spec, name| h2_heading_present(spec, name)),
            ),
            (
                "commands.rs",
                "commands.md",
                Box::new(|spec, name| {
                    h2_heading_present(spec, name)
                        || item_defined(spec, "struct", &format!("{name}Command"))
                }),
            ),
            (
                "events.rs",
                "events.md",
                Box::new(|spec, name| item_defined(spec, "struct", name)),
            ),
            (
                "services.rs",
                "services.md",
                Box::new(|spec, name| {
                    h2_heading_present(spec, name) || item_defined(spec, "struct", name)
                }),
            ),
            (
                "value_objects.rs",
                "value-objects.md",
                Box::new(|spec, name| {
                    item_defined(spec, "struct", name)
                        || item_defined(spec, "enum", name)
                        || table_row_present(spec, name)
                }),
            ),
            (
                "entities.rs",
                "entities.md",
                Box::new(|spec, name| h2_heading_present(spec, name)),
            ),
        ];

        for domain_entry in entries.flatten() {
            let domain_path = domain_entry.path();
            if !domain_path.is_dir() {
                continue;
            }
            let domain_name = match domain_entry.file_name().into_string() {
                Ok(s) => s,
                Err(_) => continue,
            };
            let src_dir = domain_path.join("src");
            if !src_dir.is_dir() {
                continue;
            }
            let spec_dir = specs_dir.join(&domain_name);
            for (src_filename, spec_filename, matcher) in &plans {
                let src_file = src_dir.join(src_filename);
                let spec_file = spec_dir.join(spec_filename);
                let Ok(src_contents) = fs::read_to_string(&src_file) else {
                    continue;
                };
                let Ok(spec_contents) = fs::read_to_string(&spec_file) else {
                    // No spec file for this domain — the spec →
                    // code direction check would already have
                    // flagged missing files in the opposite
                    // direction. Here, we report every public
                    // type as undocumented.
                    for (kind, name) in extract_public_types(&src_contents) {
                        report.violations.push(undocumented_public_item(
                            repo_root,
                            &src_file,
                            &spec_file,
                            kind,
                            &name,
                        ));
                    }
                    continue;
                };
                for (kind, name) in extract_public_types(&src_contents) {
                    // For commands.rs, the convention is to name
                    // the struct `<Verb><Entity>Command`. Strip
                    // the suffix so the spec matcher looks for
                    // `pub struct <Verb><Entity>Command`.
                    let lookup_name = if *src_filename == "commands.rs" {
                        name.strip_suffix("Command")
                            .map(str::to_string)
                            .unwrap_or_else(|| name.clone())
                    } else {
                        name.clone()
                    };
                    if matcher(&spec_contents, &lookup_name) {
                        continue;
                    }
                    report.violations.push(undocumented_public_item(
                        repo_root,
                        &src_file,
                        &spec_file,
                        kind,
                        &name,
                    ));
                }
            }
        }
    }

    fn undocumented_public_item(
        repo_root: &Path,
        src_file: &Path,
        spec_file: &Path,
        kind: &str,
        name: &str,
    ) -> Violation {
        let rel = src_file.strip_prefix(repo_root).unwrap_or(src_file);
        let spec_rel = spec_file.strip_prefix(repo_root).unwrap_or(spec_file);
        Violation {
            check: "code_to_spec:undocumented_public_item".to_string(),
            file: rel.to_path_buf(),
            line: None,
            message: format!(
                "public {kind} `{name}` has no matching row in {}",
                spec_rel.display()
            ),
        }
    }

    /// Returns a list of `(kind, name)` pairs for every
    /// `pub struct <Name>` and `pub enum <Name>` declaration in
    /// `src`. Re-exports (`pub use ...`), private items
    /// (`pub(crate)`, `pub(super)`, `pub(in path)`), and
    /// declarations whose first identifier is a generic
    /// parameter (rare; intentionally permissive) are all
    /// skipped. The delimiter check after the name prevents
    /// `pub struct FooBar` from matching `pub struct Foo`.
    ///
    /// Lines that look like attribute continuations
    /// (`#[derive(...)]`, `#[serde(...)]`) are skipped naturally
    /// because they start with `#`, not `pub`.
    fn extract_public_types(src: &str) -> Vec<(&'static str, String)> {
        let mut out = Vec::new();
        let mut seen: Vec<String> = Vec::new();
        for line in src.lines() {
            let trimmed = line.trim_start();
            // Skip `pub use` re-exports — the originating crate
            // owns the spec row.
            if trimmed.starts_with("pub use ") {
                continue;
            }
            // Only plain `pub` — not `pub(crate)` / `pub(super)`
            // / `pub(in path::to)`.
            let after_pub = match trimmed.strip_prefix("pub ") {
                Some(s) => s,
                None => continue,
            };
            let (kind, after_kind) = if let Some(s) = after_pub.strip_prefix("struct ") {
                ("struct", s)
            } else if let Some(s) = after_pub.strip_prefix("enum ") {
                ("enum", s)
            } else {
                continue;
            };
            let name_end = after_kind
                .find(|c: char| matches!(c, ' ' | '{' | '<' | '(' | ';'))
                .unwrap_or(after_kind.len());
            if name_end == 0 {
                continue;
            }
            let name = &after_kind[..name_end];
            // Names start with an uppercase letter and contain
            // only identifier characters.
            if !name
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_uppercase())
            {
                continue;
            }
            if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                continue;
            }
            let kind_static: &'static str = if kind == "struct" { "struct" } else { "enum" };
            let owned = name.to_string();
            if !seen.contains(&owned) {
                seen.push(owned.clone());
                out.push((kind_static, owned));
            }
        }
        out
    }

    /// Returns `true` when `spec` contains an `## <Name>` h2
    /// heading (with the exact name match, modulo whitespace
    /// and a trailing colon). Code fences are skipped.
    fn h2_heading_present(spec: &str, name: &str) -> bool {
        let mut in_code = false;
        for line in spec.lines() {
            if is_code_fence(line) {
                in_code = !in_code;
                continue;
            }
            if in_code {
                continue;
            }
            let Some(rest) = line.strip_prefix("## ") else {
                continue;
            };
            let heading = sanitize_heading_text(rest);
            // Accept exact match or "Name — ..."
            // (headings like "## Page — Website pages").
            if heading == name || heading.starts_with(&format!("{name} ")) {
                return true;
            }
        }
        false
    }

    /// Returns `true` when `spec` contains a table row whose
    /// first cell is the backtick-quoted `name` (e.g.
    /// `` | `PageTitle` | 1..191 chars | ``). Used for the
    /// `value-objects.md` spec which catalogues types in
    /// tables rather than as Rust declarations.
    fn table_row_present(spec: &str, name: &str) -> bool {
        let needle = format!("`{name}`");
        for line in spec.lines() {
            // Only consider table rows (start with `|`) and only
            // match when the name appears in the FIRST cell to
            // avoid false positives from prose mentions.
            let Some(after_pipe) = line.trim_start().strip_prefix('|') else {
                continue;
            };
            let first_cell = after_pipe.split('|').next().unwrap_or("");
            if first_cell.trim() == needle {
                return true;
            }
        }
        false
    }

    /// Parity check (item 4 of the No-Gaps Gates). Walks each
    /// domain crate's `aggregate.rs` / `entities.rs` and the
    /// matching `docs/specs/<domain>/tables.md`, and verifies
    /// that every `#[derive(DomainQuery)]` struct in the source
    /// has a matching aggregate row in the spec, and vice versa.
    ///
    /// The check is **bidirectional**:
    ///
    /// - **Spec → code**: every `| \`table\` | <Aggregate> | ...`
    ///   row whose `Aggregate` cell names a real PascalCase
    ///   identifier must have a corresponding `#[derive(DomainQuery)]`
    ///   struct in the domain crate. Missing struct =>
    ///   `parity:missing_macro` violation.
    /// - **Code → spec**: every `#[derive(DomainQuery)]` struct
    ///   must appear as an aggregate in `tables.md`. Missing
    ///   spec row => `parity:missing_spec_row` violation.
    ///
    /// **Heuristic & known false positives** (documented per
    /// `docs/build-plan.md` § "The No-Gaps Gates"):
    ///
    /// - The macro detector uses a substring match
    ///   (`#[derive(` AND `DomainQuery`) on the same line). This
    ///   tolerates both `#[derive(DomainQuery)]` and
    ///   `#[derive(Debug, DomainQuery)]`, and tolerates macros
    ///   defined as `pub fn DomainQuery` (none today) because
    ///   we require both substrings to coexist.
    /// - The follow-up `pub struct <Name>` is searched within
    ///   the next 5 lines, which is enough for the common
    ///   `#[derive(DomainQuery)]\npub struct Foo {}` pattern and
    ///   tolerates one or two attribute lines in between (e.g.
    ///   `#[serde(rename_all = "snake_case")]`). Macro
    ///   applications with longer attribute chains (>=3 lines
    ///   of intervening attributes) will be MISSED; they will
    ///   NOT trigger a violation because the struct will not be
    ///   attributed to the macro. This is a known limitation
    ///   of the simple regex approach.
    /// - The spec aggregate column parser ignores parenthetical
    ///   annotations (`AcademicYear (legacy)`, `Subject (alt)`)
    ///   by taking the prefix before `(`, and ignores rows whose
    ///   aggregate cell starts with `(` (e.g. `(template)`,
    ///   `(sentinel)`, `(cms)` markers). These are intentional
    ///   non-aggregates; they do not represent Rust types.
    /// - The scan is restricted to `aggregate.rs` and
    ///   `entities.rs` because the engine's spec → code
    ///   invariant pins the macro to those two files (per the
    ///   build plan). Query-stub files (`query.rs`) are
    ///   explicitly out of scope: the actual query types live
    ///   in the `educore-query-derive` crate and the stubs are
    ///   future-shaped placeholders.
    pub fn check_parity(repo_root: &Path, report: &mut LintReport) {
        let specs_dir = repo_root.join("docs").join("specs");
        let domains_dir = repo_root.join("crates").join("domains");
        let Ok(specs_entries) = fs::read_dir(&specs_dir) else {
            return;
        };
        for spec_entry in specs_entries.flatten() {
            let domain_path = spec_entry.path();
            if !domain_path.is_dir() {
                continue;
            }
            let domain_name = match spec_entry.file_name().into_string() {
                Ok(s) => s,
                Err(_) => continue,
            };
            let code_root = domains_dir.join(&domain_name);
            if !code_root.join("src").is_dir() {
                // Cross-cutting spec (events, operations, platform,
                // rbac, settings, sync) — no Rust crate, skip.
                continue;
            }
            let tables_md = domain_path.join("tables.md");
            let aggregate_rs = code_root.join("src").join("aggregate.rs");
            let entities_rs = code_root.join("src").join("entities.rs");
            let spec_contents = fs::read_to_string(&tables_md).unwrap_or_default();
            let spec_aggregates = extract_table_aggregates(&spec_contents);
            let mut macro_structs: Vec<(String, std::path::PathBuf)> = Vec::new();
            for src_file in [&aggregate_rs, &entities_rs] {
                let Ok(src_contents) = fs::read_to_string(src_file) else {
                    continue;
                };
                for name in extract_domain_query_structs(&src_contents) {
                    macro_structs.push((name, src_file.clone()));
                }
            }
            // Bidirectional comparison: every spec aggregate name
            // without a matching struct => `parity:missing_macro`;
            // every macro struct without a matching spec row =>
            // `parity:missing_spec_row`.
            for agg in &spec_aggregates {
                if !macro_structs.iter().any(|(n, _)| n == agg) {
                    report.violations.push(Violation {
                        check: "parity:missing_macro".to_string(),
                        file: tables_md
                            .strip_prefix(repo_root)
                            .unwrap_or(&tables_md)
                            .to_path_buf(),
                        line: None,
                        message: format!(
                            "spec row `{agg}` in {} has no `#[derive(DomainQuery)]` struct in crates/domains/{}/src/",
                            tables_md.display(),
                            domain_name
                        ),
                    });
                }
            }
            for (name, src_file) in &macro_structs {
                if !spec_aggregates.iter().any(|a| a == name) {
                    report.violations.push(Violation {
                        check: "parity:missing_spec_row".to_string(),
                        file: src_file
                            .strip_prefix(repo_root)
                            .unwrap_or(src_file)
                            .to_path_buf(),
                        line: None,
                        message: format!(
                            "`#[derive(DomainQuery)]` on `pub struct {name}` in {} has no matching aggregate row in {}",
                            src_file.display(),
                            tables_md.display()
                        ),
                    });
                }
            }
        }
    }

    /// Returns every `pub struct <Name>` declaration in `src`
    /// that is preceded (within ~5 lines) by a
    /// `#[derive(DomainQuery)]` attribute line. The 5-line window
    /// tolerates one or two attribute lines interleaved between
    /// the derive and the struct (e.g. `#[serde(...)]`,
    /// `#[allow(...)]`).
    ///
    /// The matcher is **deliberately permissive** (it tolerates
    /// the `#[derive(...)]` being on the same line OR within 5
    /// lines above the struct). The downside — false positives
    /// when a different derive attribute is on the line above —
    /// is acceptable because the engine's only proc macro in
    /// this role is `DomainQuery`. The check is cheap enough to
    /// run on every domain crate on every lint invocation.
    fn extract_domain_query_structs(src: &str) -> Vec<String> {
        let lines: Vec<&str> = src.lines().collect();
        let mut out = Vec::new();
        let mut seen: Vec<String> = Vec::new();
        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            // Macro application detection: `#[derive(`
            // substring AND `DomainQuery` substring on the SAME
            // line. The delimiter check after the name in
            // `extract_struct_names` below protects against
            // false matches from `pub struct CommandHistory`
            // when the suffix is `Command`. Here we have no
            // suffix; we just need the struct name.
            if !(trimmed.starts_with("#[derive") && trimmed.contains("DomainQuery")) {
                continue;
            }
            // Look forward up to 5 lines for `pub struct <Name>`.
            // Break out as soon as we see ANY `pub struct`
            // (whether or not it has a usable name) so the same
            // struct is not attributed to the next derive in a
            // chain like:
            //   #[derive(DomainQuery)]
            //   pub struct Foo {}
            //   #[derive(DomainQuery)]
            //   pub struct Bar {}
            for offset in 1..=5 {
                let Some(candidate) = lines.get(idx + offset) else {
                    break;
                };
                if !candidate.trim_start().starts_with("pub struct ") {
                    continue;
                }
                for name in extract_struct_names(candidate, "") {
                    if !seen.contains(&name) {
                        seen.push(name.clone());
                        out.push(name);
                    }
                }
                break;
            }
        }
        out
    }

    /// Parses `tables.md` and returns the deduplicated list of
    /// aggregate names declared in the second column of each
    /// markdown table row.
    ///
    /// The expected format is the same across all 10 domain
    /// specs:
    ///
    /// ```text
    /// | Table                        | Aggregate       | Notes           |
    /// | ---------------------------- | --------------- | --------------- |
    /// | `academic_students`          | Student         | The student     |
    /// | `academic_classes`           | Class           | Grade level     |
    /// | `student_records`            | StudentRecord   | Enrollment/year |
    /// ```
    ///
    /// The function tolerates:
    /// - Header rows (`Table | Aggregate | Notes |`)
    /// - Separator rows (`--- | --- | ---`)
    /// - Parenthetical annotations on the aggregate cell
    ///   (`AcademicYear (legacy)` => `AcademicYear`)
    /// - Trailing free-text annotations after a single space
    ///   (`HomeworkSubmission file` => `HomeworkSubmission`)
    /// - Empty aggregate cells (skipped)
    /// - Code-fenced rows (skipped; no tables.md ships tables
    ///   inside code fences today but the rule is consistent
    ///   with the other parsers)
    ///
    /// Cells that begin with `(` (template / sentinel / cms
    /// markers) are NOT aggregates and are skipped.
    ///
    /// Rows carrying the HTML comment `<!-- derive_skip -->` are
    /// also skipped. This marker is the engine's documented
    /// opt-out for the spec → `#[derive(DomainQuery)]` parity
    /// check. It is used on rows whose aggregate is intentionally
    /// not backed by a `#[derive(DomainQuery)]` struct today —
    /// typically because the query-derive proc-macro is
    /// currently broken (see commit `968baa76`) and adding the
    /// derive would not parse. When the macro is fixed, the
    /// marker can be removed row-by-row and the corresponding
    /// `#[derive(DomainQuery)]` annotation added.
    fn extract_table_aggregates(md: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut seen: Vec<String> = Vec::new();
        let mut in_code = false;
        for line in md.lines() {
            if is_code_fence(line) {
                in_code = !in_code;
                continue;
            }
            if in_code {
                continue;
            }
            // Per-row opt-out for the parity check (see function
            // doc). Placed as a trailing HTML comment on the
            // table row so it stays out of the rendered content
            // while remaining greppable in source.
            if line.contains("<!-- derive_skip -->") {
                continue;
            }
            // Only consider table rows.
            let Some(after_pipe) = line.trim_start().strip_prefix('|') else {
                continue;
            };
            let cells: Vec<&str> = after_pipe.split('|').collect();
            // Need at least: cell[0] (table), cell[1] (aggregate),
            // cell[2] (notes). The leading/trailing pipes produce
            // empty cells at the ends; trim them.
            if cells.len() < 3 {
                continue;
            }
            let table_cell = cells[0].trim();
            let agg_cell = cells[1].trim();
            // Skip header rows (the literal word `Table` or
            // separator rows with dashes).
            if table_cell.eq_ignore_ascii_case("Table")
                || table_cell.starts_with("---")
                || table_cell.is_empty()
            {
                continue;
            }
            // Skip non-aggregate markers.
            if agg_cell.is_empty() || agg_cell.starts_with('(') {
                continue;
            }
            // Strip parenthetical annotations:
            // `AcademicYear (legacy)` => `AcademicYear`.
            let before_paren = agg_cell.split('(').next().unwrap_or(agg_cell).trim();
            // If the cell is "HomeworkSubmission file", keep
            // only the first word (the struct name) and drop
            // the free-text trailing word. If the cell is just
            // "HomeworkSubmission", the split yields the same.
            let name = before_paren.split_whitespace().next().unwrap_or("");
            // Validate that the result looks like a Rust
            // identifier (PascalCase).
            if name.is_empty() {
                continue;
            }
            if !name.chars().next().is_some_and(char::is_alphabetic) {
                continue;
            }
            if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                continue;
            }
            if !seen.iter().any(|s| s == name) {
                seen.push(name.to_string());
                out.push(name.to_string());
            }
        }
        out
    }

    /// Parses a single `key = value` line from a TOML stanza.
    /// Returns `(key, value)` where `value` is the unquoted string
    /// (trailing quotes stripped, no escape processing). Returns
    /// `None` for non-assignment lines, multi-line values, or
    /// array/table assignments.
    ///
    /// The lint only consumes string-valued scalars (`id`,
    /// `status`, `tests`), so the parser stays narrow on purpose.
    /// A full `toml` crate is reserved for Phase 1+ follow-ups.
    fn parse_toml_kv(line: &str) -> Option<(String, String)> {
        let eq = line.find('=')?;
        let key = line[..eq].trim();
        let rest = line[eq + 1..].trim();
        if key.is_empty() || !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return None;
        }
        // String literal: "..." or '...'. Skip leading/trailing
        // whitespace inside the quotes.
        let bytes = rest.as_bytes();
        let quote = *bytes.first()?;
        if quote != b'"' && quote != b'\'' {
            // Non-string scalar (number, bool, array, inline
            // table). The lint does not need these.
            return None;
        }
        let quote_char = quote as char;
        let close = rest[1..].rfind(quote_char)? + 1;
        let value = rest[1..close].trim().to_string();
        Some((key.to_string(), value))
    }

    /// Coverage-matrix sync (item 5 of the No-Gaps Gates):
    ///
    /// 1. Every `[[row]]` in `docs/coverage.toml` whose
    ///    `status = "Tested"` MUST have a `tests = "..."` field
    ///    AND that path must exist on disk. Violations:
    ///    - `coverage_matrix:missing_tests_field` — `Tested` row
    ///      with no `tests` field.
    ///    - `coverage_matrix:missing_tests_path` — `Tested` row
    ///      whose `tests` path does not exist.
    /// 2. Every file under `crates/**/tests/*.rs` on disk MUST be
    ///    referenced by at least one `[[row]]` (regardless of
    ///    status). Violation:
    ///    - `coverage_matrix:orphan_tests_path` — file exists but
    ///      no row references it.
    ///
    /// The parser is deliberately line-based: we accumulate the
    /// current `[[row]]` block's `id`, `status`, and `tests`
    /// fields as we encounter them, and finalize the row when we
    /// hit the next `[[row]]` header (or EOF). This is enough for
    /// the well-formed, hand-edited matrix the engine ships; the
    /// full `toml` parser is reserved for Phase 1+ follow-ups.
    ///
    /// Edge cases:
    /// - A missing `docs/coverage.toml` is allowed during a fresh
    ///   scaffold and is not a violation (build plan Phase 0).
    /// - Inline `# comments` and blank lines are skipped, but
    ///   `# ...` substrings inside string values are not (the
    ///   matrix never uses them in practice).
    /// - Paths are normalised against `repo_root`. The `tests`
    ///   field may be a comma-separated list of paths; each is
    ///   verified independently.
    pub fn check_coverage_matrix(repo_root: &Path, report: &mut LintReport) {
        let toml_path = repo_root.join("docs").join("coverage.toml");
        let Ok(contents) = fs::read_to_string(&toml_path) else {
            // The matrix is allowed to be absent during a fresh
            // scaffold; the build plan calls it out as a Phase 0
            // deliverable. A missing matrix is not a lint failure.
            return;
        };

        // (id, status, tests, header_line)
        let mut rows: Vec<(String, String, Option<String>, usize)> = Vec::new();
        let mut cur_id = String::new();
        let mut cur_status = String::new();
        let mut cur_tests: Option<String> = None;
        let mut cur_header_line: usize = 0;
        for (idx, line) in contents.lines().enumerate() {
            let trimmed = line.trim();
            // Row header: finalize the previous block first.
            if trimmed.starts_with("[[row]]") {
                if !cur_id.is_empty() || !cur_status.is_empty() || cur_tests.is_some() {
                    rows.push((
                        std::mem::take(&mut cur_id),
                        std::mem::take(&mut cur_status),
                        cur_tests.take(),
                        cur_header_line,
                    ));
                } else {
                    cur_id = String::new();
                    cur_status = String::new();
                    cur_tests = None;
                }
                cur_header_line = idx + 1;
                continue;
            }
            // Blank lines and comments reset nothing but contribute nothing.
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            // Field assignments: `key = "value"` (string). We
            // only consume string-valued scalars (`id`, `status`,
            // `tests`); other field types are ignored.
            if let Some((key, value)) = parse_toml_kv(trimmed) {
                match key.as_str() {
                    "id" => cur_id = value,
                    "status" => cur_status = value,
                    "tests" => cur_tests = Some(value),
                    _ => {}
                }
            }
        }
        // Flush the trailing block (file may not end with a newline).
        if !cur_id.is_empty() || !cur_status.is_empty() || cur_tests.is_some() {
            rows.push((cur_id, cur_status, cur_tests, cur_header_line));
        }

        // Verify every Tested row and collect referenced tests paths.
        let mut referenced_tests: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        for (id, status, tests, header_line) in &rows {
            if status != "Tested" {
                if let Some(t) = tests {
                    for path in t.split(',') {
                        let p = path.trim();
                        if !p.is_empty() {
                            referenced_tests.insert(p.to_string());
                        }
                    }
                }
                continue;
            }
            match tests {
                None => {
                    report.violations.push(Violation {
                        check: "coverage_matrix:missing_tests_field".to_string(),
                        file: toml_path
                            .strip_prefix(repo_root)
                            .unwrap_or(&toml_path)
                            .to_path_buf(),
                        line: Some(*header_line),
                        message: format!(
                            "coverage row `{id}` has status=\"Tested\" but no `tests = \"...\"` field"
                        ),
                    });
                }
                Some(t) => {
                    // The matrix allows a comma-separated list of
                    // paths (e.g. one row covering both the
                    // domain aggregate and the integration test).
                    // Each path is verified independently.
                    let mut missing: Vec<&str> = Vec::new();
                    for path in t.split(',') {
                        let p = path.trim();
                        if p.is_empty() {
                            continue;
                        }
                        referenced_tests.insert(p.to_string());
                        let abs = repo_root.join(p);
                        if !abs.exists() {
                            missing.push(p);
                        }
                    }
                    if !missing.is_empty() {
                        report.violations.push(Violation {
                            check: "coverage_matrix:missing_tests_path".to_string(),
                            file: toml_path
                                .strip_prefix(repo_root)
                                .unwrap_or(&toml_path)
                                .to_path_buf(),
                            line: Some(*header_line),
                            message: format!(
                                "coverage row `{id}` references tests path(s) {} which do not exist on disk",
                                missing
                                    .iter()
                                    .map(|p| format!("`{p}`"))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                        });
                    }
                }
            }
        }

        // Orphan check: every `crates/**/tests/*.rs` must be referenced.
        let crates_dir = repo_root.join("crates");
        walk_dir(&crates_dir, &mut |path| {
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                return;
            }
            if !path.components().any(|c| c.as_os_str() == "tests") {
                return;
            }
            let Some(rel) = path.strip_prefix(repo_root).ok() else {
                return;
            };
            // Normalise to forward slashes for TOML-consistent comparison.
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            if !referenced_tests.contains(&rel_str) {
                report.violations.push(Violation {
                    check: "coverage_matrix:orphan_tests_path".to_string(),
                    file: rel.to_path_buf(),
                    line: None,
                    message: format!(
                        "tests path `{rel_str}` exists on disk but no `[[row]]` in docs/coverage.toml references it"
                    ),
                });
            }
        });
    }

    /// Anti-pattern scan: forbids `unwrap`/`expect`/`panic`/`todo!`/
    /// `unimplemented!` in production (non-test) Rust source files.
    /// The check is conservative: a `#[cfg(test)]` block is
    /// permitted to use any of these.
    pub fn check_anti_patterns(repo_root: &Path, report: &mut LintReport) {
        let crates_dir = repo_root.join("crates");
        walk_dir(&crates_dir, &mut |path| {
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                return;
            }
            scan_file_for_anti_patterns(path, repo_root, report);
        });
    }

    fn walk_dir<F: FnMut(&Path)>(dir: &Path, f: &mut F) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk_dir(&path, f);
            } else {
                f(&path);
            }
        }
    }

    /// Flags `unwrap`/`expect`/`panic!`/`todo!`/`unimplemented!`
    /// calls in production Rust source. Test code (detected by
    /// `#[cfg(test)]` blocks or by the file living under
    /// `tests/`) is exempt.
    fn scan_file_for_anti_patterns(path: &Path, repo_root: &Path, report: &mut LintReport) {
        let Ok(contents) = fs::read_to_string(path) else {
            return;
        };
        // Skip the lint source itself — it intentionally contains
        // the pattern needles as string literals, which would
        // otherwise self-trigger the scan. Also skip the
        // companion `bin/lint.rs` binary for the same reason.
        let rel = path.strip_prefix(repo_root).unwrap_or(path);
        let rel_str = rel.to_string_lossy();
        if rel_str.ends_with("src/lint.rs") || rel_str.ends_with("src/bin/lint.rs") {
            return;
        }
        if rel.components().any(|c| c.as_os_str() == "tests") {
            return;
        }
        // Domain crates (and infra) enforce an extra set of
        // restrictions: no `serde_json::Value`, no `HashMap<String,
        // _>` for domain data. Tests, examples, and other crates
        // are exempt from these (e.g. ports and adapters may
        // legitimately use `serde_json::Value` for transport-layer
        // concerns).
        let is_domain_code = rel_str.starts_with("crates/domains/")
            || rel_str.starts_with("crates/infra/");
        // Find `#[cfg(test)]` attributes. The test block is the
        // nearest `mod tests { ... }` opening that follows within
        // a small window (up to 10 lines, to accommodate the
        // `#[allow(...)]` attribute block that is conventionally
        // interposed between `#[cfg(test)]` and `mod tests`).
        // Calls inside the resulting block are exempt.
        let lines: Vec<&str> = contents.lines().collect();
        let mut test_block_ranges: Vec<(usize, usize)> = Vec::new();
        for (idx, line) in lines.iter().enumerate() {
            if !line.trim().starts_with("#[cfg(test)]") {
                continue;
            }
            for offset in 1..=10 {
                let Some(candidate) = lines.get(idx + offset) else {
                    break;
                };
                let trimmed = candidate.trim_start();
                if trimmed.starts_with("mod tests") && trimmed.contains('{') {
                    let open = idx + offset;
                    let close = match_block_close(&lines, open);
                    test_block_ranges.push((open, close));
                    break;
                }
            }
        }
        for (idx, line) in lines.iter().enumerate() {
            if test_block_ranges
                .iter()
                .any(|&(lo, hi)| idx >= lo && idx <= hi)
            {
                continue;
            }
            for needle in [
                ".unwrap()",
                ".unwrap_err()",
                ".expect(",
                "panic!(",
                "todo!()",
                "unimplemented!()",
            ] {
                if line.contains(needle) {
                    report.violations.push(Violation {
                        check: format!("anti_pattern:{needle}"),
                        file: rel.to_path_buf(),
                        line: Some(idx + 1),
                        message: format!("forbidden `{needle}` in production code"),
                    });
                }
            }
            // `as` cast on a numeric type. We deliberately do NOT
            // flag `as` when the right-hand side is a trait object
            // (e.g. `as &dyn Trait`) or a pointer type. The
            // AGENTS.md rule forbids `as` casts that truncate or
            // lose data; that maps to numeric conversions. We use a
            // simple whitespace + identifier check rather than a
            // full Rust parser — false positives are tolerable as
            // long as they are rare and the line they point to
            // clearly shows the `as` use.
            if let Some(as_pos) = line.find(" as ") {
                let after = line[as_pos + 4..].trim_start();
                let numeric_target = after
                    .split(|c: char| !c.is_alphanumeric() && c != '_')
                    .next()
                    .unwrap_or("");
                let is_numeric = [
                    "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128",
                    "isize", "f32", "f64", "bool",
                ]
                .contains(&numeric_target);
                if is_numeric {
                    report.violations.push(Violation {
                        check: format!("anti_pattern:as_{numeric_target}"),
                        file: rel.to_path_buf(),
                        line: Some(idx + 1),
                        message: format!(
                            "forbidden `as {numeric_target}` cast in production code (use TryFrom)"
                        ),
                    });
                }
            }
            // Domain-only rules: no `serde_json::Value`, no
            // `HashMap<String, _>` for domain data.
            if is_domain_code {
                if line.contains("serde_json::Value") {
                    report.violations.push(Violation {
                        check: "anti_pattern:serde_json_value".to_string(),
                        file: rel.to_path_buf(),
                        line: Some(idx + 1),
                        message: "forbidden `serde_json::Value` in domain code (use typed wrappers)"
                            .to_string(),
                    });
                }
                if line.contains("HashMap<String,") || line.contains("HashMap < String ,") {
                    report.violations.push(Violation {
                        check: "anti_pattern:hashmap_string".to_string(),
                        file: rel.to_path_buf(),
                        line: Some(idx + 1),
                        message:
                            "forbidden `HashMap<String, _>` for domain data (use typed structs)"
                                .to_string(),
                    });
                }
            }
        }
    }

    /// Returns the index of the closing brace that matches the
    /// `{` opening a block at `open_line`. Brace counting
    /// ignores braces inside line/block comments and string
    /// literals for the purpose of skipping over them. A simple
    /// depth counter is enough for the lint's needs: comments
    /// rarely contain `{`/`}` that would skew the count, and a
    /// false-positive here only widens the exempt window by one
    /// or two lines, which is harmless.
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
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    /// Creates a unique temporary directory under `std::env::temp_dir()`.
    /// Avoids a `tempfile` dep — the directory is removed at the end
    /// of the test via the returned [`TempPath`] guard.
    fn fresh_tempdir(label: &str) -> TempPath {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let pid = std::process::id();
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut p = std::env::temp_dir();
        p.push(format!("educore-lint-{label}-{pid}-{n}"));
        if p.exists() {
            let _ = std::fs::remove_dir_all(&p);
        }
        std::fs::create_dir_all(&p).expect("create tempdir");
        TempPath(p)
    }

    /// Removes the directory on drop. Best-effort: a leak on test
    /// failure is acceptable; the OS reclaims `std::env::temp_dir()`
    /// eventually.
    struct TempPath(PathBuf);

    impl Drop for TempPath {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn empty_repo_produces_clean_report() {
        let tmp = fresh_tempdir("empty");
        let r = run(&tmp.0);
        assert!(
            r.is_clean(),
            "expected clean report on empty repo, got {} violation(s): {:#?}",
            r.violations.len(),
            r.violations
        );
    }

    #[test]
    fn missing_aggregate_is_reported() {
        let tmp = fresh_tempdir("missing-aggregate");
        let spec_dir = tmp.0.join("docs/specs/test");
        std::fs::create_dir_all(&spec_dir).unwrap();
        std::fs::write(
            spec_dir.join("aggregates.md"),
            "# Test Domain — Aggregates\n\n## Widget\n\n## Gadget\n",
        )
        .unwrap();
        let code_dir = tmp.0.join("crates/domains/test/src");
        std::fs::create_dir_all(&code_dir).unwrap();
        std::fs::write(
            code_dir.join("aggregate.rs"),
            "pub struct Widget { _phantom: () }\n",
        )
        .unwrap();

        let r = run(&tmp.0);
        assert_eq!(r.violations.len(), 1, "violations = {:?}", r.violations);
        let v = &r.violations[0];
        assert_eq!(v.check, "spec_to_code:missing_aggregate");
        assert!(v.message.contains("Gadget"), "message = {}", v.message);
        assert!(
            v.message.contains("pub struct Gadget"),
            "message = {}",
            v.message
        );
    }

    #[test]
    fn missing_command_is_reported() {
        let tmp = fresh_tempdir("missing-command");
        let spec_dir = tmp.0.join("docs/specs/test");
        std::fs::create_dir_all(&spec_dir).unwrap();
        std::fs::write(
            spec_dir.join("commands.md"),
            "# Test Domain — Commands\n\n## CreateWidget\n\n```rust\npub struct CreateWidgetCommand {}\n```\n\n## DeleteWidget\n\n```rust\npub struct DeleteWidgetCommand {}\n```\n",
        )
        .unwrap();
        let code_dir = tmp.0.join("crates/domains/test/src");
        std::fs::create_dir_all(&code_dir).unwrap();
        std::fs::write(
            code_dir.join("commands.rs"),
            "pub struct CreateWidgetCommand {}\n",
        )
        .unwrap();

        let r = run(&tmp.0);
        assert_eq!(r.violations.len(), 1, "violations = {:?}", r.violations);
        let v = &r.violations[0];
        assert_eq!(v.check, "spec_to_code:missing_command");
        assert!(v.message.contains("DeleteWidget"), "message = {}", v.message);
        assert!(
            v.message.contains("DeleteWidgetCommand"),
            "message = {}",
            v.message
        );
    }

    #[test]
    fn missing_event_is_reported() {
        let tmp = fresh_tempdir("missing-event");
        let spec_dir = tmp.0.join("docs/specs/test");
        std::fs::create_dir_all(&spec_dir).unwrap();
        // The real spec format puts the prelude (EventEnvelope,
        // DomainEvent trait, etc.) BEFORE the first h2 heading; the
        // lint parser skips everything before the first h2 to
        // exclude the prelude. This fixture mirrors that layout.
        std::fs::write(
            spec_dir.join("events.md"),
            "# Test Domain — Events\n\n```rust\npub struct EventEnvelope<E> {}\n```\n\n## Widget Lifecycle\n\n```rust\npub struct WidgetCreated {}\npub struct WidgetDestroyed {}\n```\n",
        )
        .unwrap();
        let code_dir = tmp.0.join("crates/domains/test/src");
        std::fs::create_dir_all(&code_dir).unwrap();
        std::fs::write(
            code_dir.join("events.rs"),
            "pub struct WidgetCreated {}\n",
        )
        .unwrap();

        let r = run(&tmp.0);
        assert_eq!(r.violations.len(), 1, "violations = {:?}", r.violations);
        let v = &r.violations[0];
        assert_eq!(v.check, "spec_to_code:missing_event");
        assert!(
            v.message.contains("WidgetDestroyed"),
            "message = {}",
            v.message
        );
        assert!(
            v.message.contains("pub struct WidgetDestroyed"),
            "message = {}",
            v.message
        );
    }

    // ---- Anti-pattern detection tests ----

    /// Helper: write a single Rust source file under a domain
    /// crate and run the lint, returning the violations.
    fn scan_domain_source(label: &str, src: &str) -> Vec<Violation> {
        let tmp = fresh_tempdir(label);
        let code_dir = tmp.0.join("crates/domains/test/src");
        std::fs::create_dir_all(&code_dir).unwrap();
        std::fs::write(code_dir.join("aggregate.rs"), src).unwrap();
        run(&tmp.0).violations
    }

    #[test]
    fn anti_pattern_unwrap_in_prod_is_reported() {
        let v = scan_domain_source(
            "anti-unwrap",
            "pub struct Widget { value: Option<u32> }\nimpl Widget { pub fn value(&self) -> u32 { self.value.unwrap() } }\n",
        );
        assert!(
            v.iter().any(|x| x.check == "anti_pattern:.unwrap()"),
            "expected `.unwrap()` violation, got: {:#?}",
            v
        );
    }

    #[test]
    fn anti_pattern_expect_in_prod_is_reported() {
        let v = scan_domain_source(
            "anti-expect",
            "pub struct Widget { value: Option<u32> }\nimpl Widget { pub fn value(&self) -> u32 { self.value.expect(\"always set\") } }\n",
        );
        assert!(
            v.iter().any(|x| x.check == "anti_pattern:.expect("),
            "expected `.expect(` violation, got: {:#?}",
            v
        );
    }

    #[test]
    fn anti_pattern_panic_in_prod_is_reported() {
        let v = scan_domain_source(
            "anti-panic",
            "pub struct Widget {}\nimpl Widget { pub fn check(&self) { panic!(\"nope\"); } }\n",
        );
        assert!(
            v.iter().any(|x| x.check == "anti_pattern:panic!("),
            "expected `panic!` violation, got: {:#?}",
            v
        );
    }

    #[test]
    fn anti_pattern_as_numeric_cast_in_prod_is_reported() {
        let v = scan_domain_source(
            "anti-as",
            "pub struct Widget { value: u64 }\nimpl Widget { pub fn value(&self) -> u32 { self.value as u32 } }\n",
        );
        assert!(
            v.iter()
                .any(|x| x.check == "anti_pattern:as_u32" || x.check.starts_with("anti_pattern:as_")),
            "expected `as u32` violation, got: {:#?}",
            v
        );
    }

    #[test]
    fn anti_pattern_serde_json_value_in_domain_is_reported() {
        let v = scan_domain_source(
            "anti-value",
            "pub struct Widget { payload: serde_json::Value }\n",
        );
        assert!(
            v.iter()
                .any(|x| x.check == "anti_pattern:serde_json_value"),
            "expected `serde_json::Value` violation, got: {:#?}",
            v
        );
    }

    #[test]
    fn anti_pattern_hashmap_string_in_domain_is_reported() {
        let v = scan_domain_source(
            "anti-hashmap",
            "use std::collections::HashMap;\npub struct Widget { fields: HashMap<String, String> }\n",
        );
        assert!(
            v.iter()
                .any(|x| x.check == "anti_pattern:hashmap_string"),
            "expected `HashMap<String,` violation, got: {:#?}",
            v
        );
    }

    #[test]
    fn anti_pattern_unwrap_in_test_block_is_exempt() {
        // Same `.unwrap()` pattern, but inside a `#[cfg(test)]`
        // block — the lint must NOT report it.
        let v = scan_domain_source(
            "anti-unwrap-test-exempt",
            "pub struct Widget {}\n#[cfg(test)]\nmod tests { #[test] fn it_works() { let _ = Option::Some(1).unwrap(); } }\n",
        );
        assert!(
            !v.iter().any(|x| x.check.starts_with("anti_pattern:")),
            "expected no anti-pattern violations in test block, got: {:#?}",
            v
        );
    }

    #[test]
    fn anti_pattern_serde_json_value_in_adapter_is_exempt() {
        // Adapters are not "domain code" per the lint's policy —
        // they may use `serde_json::Value` for transport concerns.
        let tmp = fresh_tempdir("anti-value-adapter-exempt");
        let code_dir = tmp.0.join("crates/adapters/foo/src");
        std::fs::create_dir_all(&code_dir).unwrap();
        std::fs::write(
            code_dir.join("lib.rs"),
            "pub struct Widget { payload: serde_json::Value }\n",
        )
        .unwrap();
        let r = run(&tmp.0);
        assert!(
            !r.violations
                .iter()
                .any(|x| x.check == "anti_pattern:serde_json_value"),
            "expected NO `serde_json::Value` violation in adapter, got: {:#?}",
            r.violations
        );
    }

    // ---- Code → spec direction tests ----

    /// Helper: write a `commands.rs`-equivalent fixture with a
    /// matching `commands.md` spec, and run the lint.
    fn scan_code_to_spec_commands(
        label: &str,
        spec_md: &str,
        commands_rs: &str,
    ) -> Vec<Violation> {
        let tmp = fresh_tempdir(label);
        let spec_dir = tmp.0.join("docs/specs/test");
        std::fs::create_dir_all(&spec_dir).unwrap();
        std::fs::write(spec_dir.join("commands.md"), spec_md).unwrap();
        // Provide stubs for the other spec files so the runner
        // does not flag them as missing — we are testing the
        // code → spec direction in isolation.
        std::fs::write(
            spec_dir.join("aggregates.md"),
            "# Test Domain — Aggregates\n",
        )
        .unwrap();
        std::fs::write(spec_dir.join("events.md"), "# Test Domain — Events\n").unwrap();
        std::fs::write(spec_dir.join("services.md"), "# Test Domain — Services\n").unwrap();
        std::fs::write(
            spec_dir.join("value-objects.md"),
            "# Test Domain — Value Objects\n",
        )
        .unwrap();
        std::fs::write(spec_dir.join("entities.md"), "# Test Domain — Entities\n").unwrap();
        let code_dir = tmp.0.join("crates/domains/test/src");
        std::fs::create_dir_all(&code_dir).unwrap();
        // Stubs for the other source files — empty content keeps
        // them out of the code → spec scan's path.
        std::fs::write(code_dir.join("aggregate.rs"), "").unwrap();
        std::fs::write(code_dir.join("events.rs"), "").unwrap();
        std::fs::write(code_dir.join("services.rs"), "").unwrap();
        std::fs::write(code_dir.join("value_objects.rs"), "").unwrap();
        std::fs::write(code_dir.join("entities.rs"), "").unwrap();
        std::fs::write(code_dir.join("lib.rs"), "").unwrap();
        std::fs::write(code_dir.join("commands.rs"), commands_rs).unwrap();
        run(&tmp.0).violations
    }

    #[test]
    fn code_to_spec_undocumented_public_struct_is_reported() {
        // The spec only mentions `CreateWidget`. The Rust code
        // declares a `pub struct DeleteWidgetCommand` that has no
        // matching row in `commands.md`.
        let spec = "# Test Domain — Commands\n\n## CreateWidget\n\n```rust\npub struct CreateWidgetCommand {}\n```\n";
        let code = "pub struct CreateWidgetCommand {}\npub struct DeleteWidgetCommand {}\n";
        let v = scan_code_to_spec_commands("code-to-spec-undocumented", spec, code);
        assert!(
            v.iter().any(|x| x.check == "code_to_spec:undocumented_public_item"
                && x.message.contains("DeleteWidgetCommand")),
            "expected undocumented violation for `DeleteWidgetCommand`, got: {:#?}",
            v
        );
        // `CreateWidgetCommand` should NOT be flagged because the
        // matcher strips the `Command` suffix and finds
        // `pub struct CreateWidgetCommand` in the spec.
        assert!(
            !v.iter().any(|x| x.check == "code_to_spec:undocumented_public_item"
                && x.message.contains("CreateWidgetCommand")),
            "did not expect undocumented violation for `CreateWidgetCommand`, got: {:#?}",
            v
        );
    }

    #[test]
    fn code_to_spec_documented_public_struct_is_clean() {
        // Both the spec and the Rust code declare `CreateWidget`.
        // No code → spec violation should fire.
        let spec = "# Test Domain — Commands\n\n## CreateWidget\n\n```rust\npub struct CreateWidgetCommand {}\n```\n";
        let code = "pub struct CreateWidgetCommand {}\n";
        let v = scan_code_to_spec_commands("code-to-spec-clean", spec, code);
        assert!(
            !v.iter().any(|x| x.check == "code_to_spec:undocumented_public_item"),
            "expected NO undocumented violations when spec and code match, got: {:#?}",
            v
        );
    }

    #[test]
    fn code_to_spec_aggregate_reexports_are_exempt() {
        // `aggregate.rs` has one re-export (`pub use ...`) and one
        // real declaration (`pub struct Orphan { _phantom: () }`).
        // The re-export must not trigger a violation; the orphan
        // declaration must, because `aggregates.md` has no `##
        // Orphan` heading.
        let tmp = fresh_tempdir("code-to-spec-reexport-exempt");
        let spec_dir = tmp.0.join("docs/specs/test");
        std::fs::create_dir_all(&spec_dir).unwrap();
        std::fs::write(
            spec_dir.join("aggregates.md"),
            "# Test Domain — Aggregates\n\n## Widget\n",
        )
        .unwrap();
        std::fs::write(spec_dir.join("commands.md"), "# Test Domain — Commands\n").unwrap();
        std::fs::write(spec_dir.join("events.md"), "# Test Domain — Events\n").unwrap();
        std::fs::write(spec_dir.join("services.md"), "# Test Domain — Services\n").unwrap();
        std::fs::write(
            spec_dir.join("value-objects.md"),
            "# Test Domain — Value Objects\n",
        )
        .unwrap();
        std::fs::write(spec_dir.join("entities.md"), "# Test Domain — Entities\n").unwrap();
        let code_dir = tmp.0.join("crates/domains/test/src");
        std::fs::create_dir_all(&code_dir).unwrap();
        std::fs::write(code_dir.join("commands.rs"), "").unwrap();
        std::fs::write(code_dir.join("events.rs"), "").unwrap();
        std::fs::write(code_dir.join("services.rs"), "").unwrap();
        std::fs::write(code_dir.join("value_objects.rs"), "").unwrap();
        std::fs::write(code_dir.join("entities.rs"), "").unwrap();
        std::fs::write(code_dir.join("lib.rs"), "").unwrap();
        std::fs::write(
            code_dir.join("aggregate.rs"),
            // Re-export of a type that lives in another crate (the
            // usual pattern in domain crate preludes). The lint
            // must skip these.
            "pub use educore_core::ids::WidgetId;\n\
             // A truly orphan declaration with no spec row.\n\
             pub struct Orphan { _phantom: () }\n",
        )
        .unwrap();
        let r = run(&tmp.0);
        let code_to_spec: Vec<_> = r
            .violations
            .iter()
            .filter(|x| x.check == "code_to_spec:undocumented_public_item")
            .collect();
        assert!(
            code_to_spec.iter().any(|x| x.message.contains("Orphan")),
            "expected violation for `Orphan`, got: {:#?}",
            code_to_spec
        );
        assert!(
            !code_to_spec.iter().any(|x| x.message.contains("WidgetId")),
            "did not expect violation for re-exported `WidgetId`, got: {:#?}",
            code_to_spec
        );
    }

    // ---- Parity check tests ----

    /// Helper: lay down a domain crate with the given `aggregate.rs`
    /// source and the given `tables.md` content, then run the lint.
    /// Mirrors the layout that `check_parity` expects (one spec
    /// row per aggregate in the second column of the markdown
    /// table).
    fn scan_parity(label: &str, aggregate_rs: &str, tables_md: &str) -> Vec<Violation> {
        let tmp = fresh_tempdir(label);
        let spec_dir = tmp.0.join("docs/specs/test");
        std::fs::create_dir_all(&spec_dir).unwrap();
        std::fs::write(spec_dir.join("tables.md"), tables_md).unwrap();
        let code_dir = tmp.0.join("crates/domains/test/src");
        std::fs::create_dir_all(&code_dir).unwrap();
        std::fs::write(code_dir.join("aggregate.rs"), aggregate_rs).unwrap();
        std::fs::write(code_dir.join("entities.rs"), "").unwrap();
        run(&tmp.0).violations
    }

    #[test]
    fn parity_missing_macro_application_is_reported() {
        // `tables.md` declares `Widget` AND `Gadget` aggregates.
        // Only `Widget` has a `#[derive(DomainQuery)]` struct in
        // `aggregate.rs`. The lint must report `Gadget` as a
        // `parity:missing_macro` violation.
        let tables_md = "\
| Table             | Aggregate | Notes       |
| ----------------- | --------- | ----------- |
| `test_widgets`    | Widget    | Has macro   |
| `test_gadgets`    | Gadget    | Missing one |
";
        let aggregate_rs = "\
#[derive(Debug, Clone, DomainQuery)]
pub struct Widget { _phantom: () }
";
        let v = scan_parity("parity-missing-macro", aggregate_rs, tables_md);
        let parity: Vec<_> = v.iter().filter(|x| x.check.starts_with("parity:")).collect();
        assert_eq!(parity.len(), 1, "violations = {:#?}", parity);
        let only = parity[0];
        assert_eq!(only.check, "parity:missing_macro");
        assert!(
            only.message.contains("Gadget"),
            "expected Gadget in message, got: {}",
            only.message
        );
        assert!(
            only.message.contains("tables.md"),
            "expected tables.md in message, got: {}",
            only.message
        );
    }

    #[test]
    fn parity_missing_spec_row_is_reported() {
        // `tables.md` only declares `Widget`. `aggregate.rs`
        // derives `DomainQuery` for both `Widget` AND `Orphan`.
        // The lint must report `Orphan` as a
        // `parity:missing_spec_row` violation.
        let tables_md = "\
| Table             | Aggregate | Notes |
| ----------------- | --------- | ----- |
| `test_widgets`    | Widget    | ok    |
";
        let aggregate_rs = "\
#[derive(Debug, Clone, DomainQuery)]
pub struct Widget { _phantom: () }
#[derive(Debug, Clone, DomainQuery)]
pub struct Orphan { _phantom: () }
";
        let v = scan_parity("parity-missing-spec-row", aggregate_rs, tables_md);
        let parity: Vec<_> = v.iter().filter(|x| x.check.starts_with("parity:")).collect();
        assert_eq!(parity.len(), 1, "violations = {:#?}", parity);
        let only = parity[0];
        assert_eq!(only.check, "parity:missing_spec_row");
        assert!(
            only.message.contains("Orphan"),
            "expected Orphan in message, got: {}",
            only.message
        );
    }

    // ---- Coverage-matrix sync tests ----

    /// Helper: lay down `docs/coverage.toml` plus optional
    /// `crates/...` fixtures, then run the lint.
    fn scan_coverage_matrix(
        label: &str,
        coverage_toml: &str,
        crates_layout: &[(&str, &str)],
    ) -> Vec<Violation> {
        let tmp = fresh_tempdir(label);
        std::fs::create_dir_all(tmp.0.join("docs")).unwrap();
        std::fs::write(tmp.0.join("docs/coverage.toml"), coverage_toml).unwrap();
        for (rel_path, contents) in crates_layout {
            let full = tmp.0.join(rel_path);
            if let Some(parent) = full.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::write(&full, contents).unwrap();
        }
        run(&tmp.0).violations
    }

    #[test]
    fn coverage_matrix_missing_tests_path_is_reported() {
        // A `Tested` row whose `tests = "..."` points to a file
        // that does NOT exist on disk. The lint must flag it.
        let coverage = "\
[[row]]
id = \"orphan_tested\"
status = \"Tested\"
tests = \"crates/domains/test/tests/ghost_e2e.rs\"
";
        let v = scan_coverage_matrix("coverage-missing-path", coverage, &[]);
        let matrix: Vec<_> = v
            .iter()
            .filter(|x| x.check.starts_with("coverage_matrix:"))
            .collect();
        assert_eq!(matrix.len(), 1, "violations = {:#?}", matrix);
        let only = matrix[0];
        assert_eq!(only.check, "coverage_matrix:missing_tests_path");
        assert!(
            only.message.contains("ghost_e2e.rs"),
            "expected ghost_e2e.rs in message, got: {}",
            only.message
        );
        assert!(
            only.message.contains("orphan_tested"),
            "expected row id in message, got: {}",
            only.message
        );
    }

    #[test]
    fn coverage_matrix_orphan_tests_path_is_reported() {
        // A tests file that exists on disk but no row in
        // coverage.toml references it. The lint must flag it.
        // We also add a row that DOES reference a different
        // tests file, so the orphan check is the only thing
        // that fires (no `missing_tests_path` violation for
        // the referenced row, because the referenced file
        // exists).
        let coverage = "\
[[row]]
id = \"referenced_row\"
status = \"Implemented\"
tests = \"crates/domains/test/tests/referenced.rs\"
";
        let crates = &[
            ("crates/domains/test/tests/referenced.rs", "// referenced\n"),
            ("crates/domains/test/tests/orphan.rs", "// orphan\n"),
        ];
        let v = scan_coverage_matrix("coverage-orphan", coverage, crates);
        let matrix: Vec<_> = v
            .iter()
            .filter(|x| x.check.starts_with("coverage_matrix:"))
            .collect();
        assert_eq!(matrix.len(), 1, "violations = {:#?}", matrix);
        let only = matrix[0];
        assert_eq!(only.check, "coverage_matrix:orphan_tests_path");
        assert!(
            only.message.contains("orphan.rs"),
            "expected orphan.rs in message, got: {}",
            only.message
        );
        assert!(
            !matrix.iter().any(|x| x.message.contains("missing_tests_path")),
            "did not expect missing_tests_path violation, got: {:#?}",
            matrix
        );
    }
}
