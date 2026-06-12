//! `educore-core::lint` binary.
//!
//! Walks the repo and emits a non-zero exit code if any of the
//! no-gaps gate checks fail. Per `docs/build-plan.md` § "The
//! No-Gaps Gates" item 2:
//!
//! ```text
//! cargo run -p educore-core --bin lint --features lint
//! ```
//!
//! Exit codes:
//! - `0` — all checks passed
//! - `1` — at least one violation was found
//! - `2` — usage error (could not determine the repo root)
//! - `3` — the `lint` Cargo feature is not enabled (rebuild
//!   with `--features lint`)
//!
//! The binary is always available (so `cargo build` does not
//! fail), but the actual runner requires the `lint` Cargo
//! feature on `educore-core`.
//!
//! The binary is a CLI tool; the workspace-wide `print_stderr`
//! deny is opted out for this file because the binary's only
//! communication channel is the shell.

#![forbid(unsafe_code)]
#![allow(clippy::print_stderr)]

use std::process::ExitCode;

#[cfg(feature = "lint")]
use std::path::PathBuf;

fn main() -> ExitCode {
    #[cfg(not(feature = "lint"))]
    {
        eprintln!("lint: the `lint` Cargo feature is not enabled. Rebuild with `--features lint`.");
        ExitCode::from(3)
    }
    #[cfg(feature = "lint")]
    {
        run_lint()
    }
}

#[cfg(feature = "lint")]
fn run_lint() -> ExitCode {
    use educore_core::lint;

    let repo_root = std::env::args()
        .nth(1)
        .map_or_else(find_repo_root, PathBuf::from);

    if !repo_root.join("Cargo.toml").exists() {
        eprintln!(
            "lint: {} does not look like the Educore repo root (no Cargo.toml)",
            repo_root.display()
        );
        return ExitCode::from(2);
    }

    let report = lint::run(&repo_root);
    if report.is_clean() {
        ExitCode::SUCCESS
    } else {
        let n = report.print_to_stderr();
        eprintln!("lint: {n} violation(s) found");
        ExitCode::from(1)
    }
}

/// Walk up the directory tree from CWD until a `Cargo.toml` is found.
#[cfg(feature = "lint")]
fn find_repo_root() -> PathBuf {
    let mut p = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        if p.join("Cargo.toml").exists() {
            return p;
        }
        if !p.pop() {
            return PathBuf::from(".");
        }
    }
}
