//! # Parity behavior matrix (Phase 16)
//!
//! A documentation test that asserts the parity matrix has the
//! shape the engine contract requires. The matrix is a `const`
//! slice of `(feature, backend, dialect, supported)` rows that
//! lists which `(feature × backend)` combinations are required
//! to behave identically. The first two tests below validate the
//! shape; the rest of the suite
//! (`parity_cross_backend_equivalence.rs`,
//! `parity_outbox_to_event_log_relay.rs`,
//! `parity_idempotency_collision.rs`,
//! `parity_audit_cross_tenant_isolation.rs`,
//! `parity_event_log_filter.rs`,
//! `parity_transaction_commit_rollback.rs`) runs the same
//! scenario against every backend and asserts identical
//! observable behaviour.
//!
//! The matrix is intentionally a `const` slice (not a CSV / TOML
//! file) so the test harness can iterate over it without any
//! runtime dependency on a parser crate.

#![cfg(test)]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

/// The parity matrix. One row per `(feature × backend)`
/// combination that must behave identically across the engine's
/// five storage backends.
///
/// Columns:
/// - `feature` — the storage-port sub-port behaviour under test
///   (`outbox_append`, `audit_log_append`, ...).
/// - `backend` — one of `testkit`, `sqlite`, `surrealdb`,
///   `postgres`, `mysql`.
/// - `dialect` — the SQL dialect (or `in-memory` / `surql` for
///   the non-SQL backends). Recorded for the audit trail; tests
///   themselves assert behaviour, not dialect strings.
/// - `supported` — `true` if the backend supports the feature
///   today; `false` if the backend has a documented deviation.
const PARITY_MATRIX: &[(&str, &str, &str, bool)] = &[
    // ---- outbox_append (5/5) ----
    ("outbox_append", "testkit", "in-memory", true),
    ("outbox_append", "sqlite", "sqlite3", true),
    ("outbox_append", "surrealdb", "surql", true),
    ("outbox_append", "postgres", "pg", true),
    ("outbox_append", "mysql", "mysql", true),
    // ---- audit_log_append (5/5) ----
    ("audit_log_append", "testkit", "in-memory", true),
    ("audit_log_append", "sqlite", "sqlite3", true),
    ("audit_log_append", "surrealdb", "surql", true),
    ("audit_log_append", "postgres", "pg", true),
    ("audit_log_append", "mysql", "mysql", true),
    // ---- audit_log_read_for_target (5/5) ----
    (
        "audit_log_read_for_target",
        "testkit",
        "in-memory",
        true,
    ),
    ("audit_log_read_for_target", "sqlite", "sqlite3", true),
    (
        "audit_log_read_for_target",
        "surrealdb",
        "surql",
        true,
    ),
    ("audit_log_read_for_target", "postgres", "pg", true),
    ("audit_log_read_for_target", "mysql", "mysql", true),
    // ---- event_log_filter (5/5) ----
    ("event_log_filter", "testkit", "in-memory", true),
    ("event_log_filter", "sqlite", "sqlite3", true),
    ("event_log_filter", "surrealdb", "surql", true),
    ("event_log_filter", "postgres", "pg", true),
    ("event_log_filter", "mysql", "mysql", true),
    // ---- idempotency_collision (5/5) ----
    ("idempotency_collision", "testkit", "in-memory", true),
    ("idempotency_collision", "sqlite", "sqlite3", true),
    ("idempotency_collision", "surrealdb", "surql", true),
    ("idempotency_collision", "postgres", "pg", true),
    ("idempotency_collision", "mysql", "mysql", true),
    // ---- transaction_commit_rollback (5/5) ----
    (
        "transaction_commit_rollback",
        "testkit",
        "in-memory",
        true,
    ),
    (
        "transaction_commit_rollback",
        "sqlite",
        "sqlite3",
        true,
    ),
    (
        "transaction_commit_rollback",
        "surrealdb",
        "surql",
        true,
    ),
    ("transaction_commit_rollback", "postgres", "pg", true),
    ("transaction_commit_rollback", "mysql", "mysql", true),
];

/// The set of always-on backends. The PG / MySQL variants are
/// env-gated and are therefore not part of the default
/// "every test always runs" surface; they are listed in the
/// matrix but their `supported` flag is enforced through the
/// `setup_pg` / `setup_mysql` `Option`-returning helpers.
const ALWAYS_ON_BACKENDS: &[&str] = &["testkit", "sqlite", "surrealdb"];

/// All five backends the engine targets. The matrix must list
/// at least one row per backend so the parity coverage is
/// comprehensive.
const ALL_BACKENDS: &[&str] = &["testkit", "sqlite", "surrealdb", "postgres", "mysql"];

#[test]
fn parity_matrix_is_non_empty() {
    assert!(
        PARITY_MATRIX.len() >= 25,
        "parity matrix must cover at least 5 features × 5 backends (25 rows); got {}",
        PARITY_MATRIX.len()
    );
}

#[test]
fn every_backend_has_at_least_one_row() {
    for backend in ALL_BACKENDS {
        assert!(
            PARITY_MATRIX.iter().any(|row| row.1 == *backend),
            "backend {backend} has no rows in PARITY_MATRIX"
        );
    }
}

#[test]
fn every_feature_has_rows_for_every_always_on_backend() {
    let features: Vec<&str> = PARITY_MATRIX
        .iter()
        .map(|row| row.0)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    for feature in &features {
        for backend in ALWAYS_ON_BACKENDS {
            assert!(
                PARITY_MATRIX
                    .iter()
                    .any(|row| row.0 == *feature && row.1 == *backend),
                "feature {feature} is missing a row for backend {backend}"
            );
        }
    }
}

#[test]
fn every_feature_is_either_5_supported_or_fully_unsupported() {
    // For each feature, count the `supported = true` rows. The
    // expected contract is "all-or-nothing": a feature is either
    // supported on every backend or unsupported on every
    // backend. A partial-coverage feature would indicate a
    // missing adapter implementation that the parity suite
    // would silently miss.
    let by_feature: std::collections::BTreeMap<&str, Vec<bool>> =
        PARITY_MATRIX
            .iter()
            .fold(std::collections::BTreeMap::new(), |mut acc, row| {
                acc.entry(row.0).or_default().push(row.3);
                acc
            });
    for (feature, supported_flags) in &by_feature {
        let any = supported_flags.iter().any(|s| *s);
        let all = supported_flags.iter().all(|s| *s);
        assert!(
            any == all,
            "feature {feature} is partially supported ({}/{} backends); the parity contract is all-or-nothing",
            supported_flags.iter().filter(|s| **s).count(),
            supported_flags.len()
        );
    }
}

#[test]
fn every_row_has_a_non_empty_dialect() {
    for (feature, backend, dialect, _supported) in PARITY_MATRIX {
        assert!(!feature.is_empty(), "feature label must not be empty");
        assert!(!backend.is_empty(), "backend label must not be empty");
        assert!(!dialect.is_empty(), "dialect label must not be empty");
    }
}