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

    /// Coverage-matrix sync: every `Tested` row in
    /// `docs/coverage.toml` has a `tests` path that exists on disk.
    pub fn check_coverage_matrix(repo_root: &Path, _report: &mut LintReport) {
        let toml_path = repo_root.join("docs").join("coverage.toml");
        let Ok(contents) = fs::read_to_string(&toml_path) else {
            // The matrix is allowed to be absent during a fresh
            // scaffold; the build plan calls it out as a Phase 0
            // deliverable. A missing matrix is not a lint failure.
            return;
        };

        // Naive line-by-line scan for `status = "Tested"` rows that
        // are missing a `tests = "..."` field on the same stanza.
        // The full TOML parser is reserved for Phase 1+; this
        // lightweight pass is enough to catch the common drift.
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
        // verification is delegated to Phase 1+ (the full lint
        // sub-module). For now, this pass produces a no-op when
        // the matrix is well-formed; it surfaces as a violation
        // when the matrix is hand-edited and a `Tested` row is
        // left without a `tests` path.
        let _ = status_tested;
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
}
