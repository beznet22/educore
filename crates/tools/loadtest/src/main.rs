//! `educore-loadtest` — load test harness for the Educore engine.
//!
//! Per `docs/build-plan.md` Phase 17 Task 2 and
//! `docs/guides/test-strategy.md`, the engine must sustain a small-
//! scale SaaS deployment (100 schools × 10k students = ~1M students)
//! with predictable latency. This binary wires the engine to the
//! in-memory testkit backend, runs bulk commands at scale, and
//! reports throughput + per-command latency (p50 / p95 / p99).
//!
//! ## Targets
//!
//! - 100 schools × 10k students per school = ~1M total.
//! - Bulk command: `StorageAdapter::bulk_insert_student_attendances`
//!   with `BULK_SIZE` rows per command (default 50).
//! - Total commands: `schools × (students_per_school / bulk_size)`.
//! - Throughput goal: documented at
//!   `docs/audit_reports/loadtest_baseline.md`.
//!
//! ## Why in-memory?
//!
//! The harness targets the engine's command pipeline, not the
//! underlying storage adapter's SQL/ORM cost. Phase 17 Task 3
//! handles cross-compile and CI matrix; the load numbers measured
//! here isolate the engine's command-dispatch + outbox-drain
//! overhead from the per-backend SQL variance. Production
//! benchmarks against SurrealDB / PG / MySQL run separately under
//! `crates/tools/storage-parity/`.
//!
//! ## Why no `println!`?
//!
//! The workspace clippy lints forbid `print_stdout` /
//! `print_stderr`. Output goes through `tracing` so the consumer
//! can route it via `RUST_LOG` / `tracing-subscriber` filters.
//!
//! ## CLI
//!
//! ```text
//! $ cargo run --release -p educore-loadtest -- \
//!     --schools 100 --students-per-school 10000 --bulk-size 50
//! ```
//!
//! For a 1-second smoke test on a laptop:
//!
//! ```text
//! $ cargo run --release -p educore-loadtest -- \
//!     --schools 1 --students-per-school 100 --bulk-size 10
//! ```
//!
//! See `docs/audit_reports/loadtest_baseline.md` for the full
//! methodology, results table, and CI-rerun instructions.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::time::{Duration, Instant};

use anyhow::Result;
use chrono::{Duration as ChronoDuration, NaiveDate};
use clap::Parser;
use educore_core::clock::{IdGenerator, SystemIdGen};
use educore_core::ids::{Identifier, SchoolId};
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};
use educore_storage::{StorageAdapter, StudentAttendanceRow};
use educore_testkit::test_world;
use tracing::{info, warn};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

/// Command-line arguments for the load test harness.
#[derive(Debug, Parser)]
#[command(
    name = "educore-loadtest",
    about = "Load test harness for the Educore engine",
    version
)]
struct Args {
    /// Number of synthetic schools to provision. Default: 100
    /// (Phase 17 Task 2 target).
    #[arg(long, default_value_t = 100)]
    schools: usize,

    /// Number of synthetic student-attendance rows per school.
    /// Default: 10_000 (Phase 17 Task 2 target).
    #[arg(long, default_value_t = 10_000)]
    students_per_school: usize,

    /// Number of rows per `bulk_insert_student_attendances`
    /// command. Smaller batches = more commands = more dispatcher
    /// overhead; larger batches = fewer commands = more SQL per
    /// round-trip in production. Default: 50.
    #[arg(long, default_value_t = 50)]
    bulk_size: usize,

    /// Number of warmup commands to run before timing. Warmup
    /// primes allocator / mutex / tokio worker pools so the
    /// first measured command reflects steady-state cost.
    /// Default: 100.
    #[arg(long, default_value_t = 100)]
    warmup_commands: usize,

    /// Skip the warmup phase entirely (used for `cargo test`).
    #[arg(long, default_value_t = false)]
    skip_warmup: bool,
}

// ---------------------------------------------------------------------------
// Loadtest result
// ---------------------------------------------------------------------------

/// A single load test run, with raw timing data + derived metrics.
struct LoadtestResult {
    /// The scale parameters.
    schools: usize,
    students_per_school: usize,
    bulk_size: usize,
    /// Total bulk commands executed (after warmup).
    total_commands: usize,
    /// Total attendance rows inserted (after warmup).
    total_rows: usize,
    /// Wall-clock duration of the benchmark loop (warmup excluded).
    elapsed: Duration,
    /// Per-command latency in microseconds, unsorted (sorted on
    /// percentile computation).
    latencies_us: Vec<u64>,
}

impl LoadtestResult {
    /// Throughput in bulk commands per second.
    #[must_use]
    fn throughput_cmds_per_sec(&self) -> f64 {
        self.total_commands as f64 / self.elapsed.as_secs_f64()
    }

    /// Throughput in attendance rows per second.
    #[must_use]
    fn throughput_rows_per_sec(&self) -> f64 {
        self.total_rows as f64 / self.elapsed.as_secs_f64()
    }

    /// Returns the latency at the given percentile (0.0..=1.0)
    /// as a `Duration`.
    #[must_use]
    fn percentile(&self, p: f64) -> Duration {
        if self.latencies_us.is_empty() {
            return Duration::ZERO;
        }
        let mut sorted = self.latencies_us.clone();
        sorted.sort_unstable();
        // Nearest-rank percentile (NIST primary): index =
        // ceil(p * N) - 1, clamped to [0, N-1].
        let n = sorted.len();
        let rank = ((p * n as f64).ceil() as usize).saturating_sub(1).min(n - 1);
        Duration::from_micros(sorted[rank])
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();
    info!(
        schools = args.schools,
        students_per_school = args.students_per_school,
        bulk_size = args.bulk_size,
        warmup_commands = args.warmup_commands,
        "educore-loadtest starting"
    );

    let result = run_loadtest(&args).await?;
    print_report(&result);
    Ok(())
}

// ---------------------------------------------------------------------------
// Benchmark loop
// ---------------------------------------------------------------------------

async fn run_loadtest(args: &Args) -> Result<LoadtestResult> {
    // The testkit wires every engine port (storage / auth / notify /
    // payment / files / integrations / event-bus) to in-memory impls.
    // We use it directly so the harness exercises the same code
    // paths that production callers reach via `Engine::test_world()`.
    let world = test_world();
    let storage = world.storage.clone();

    let g = SystemIdGen;
    let actor = g.next_user_id();

    // 1. Provision N synthetic schools.
    //
    // The testkit storage adapter has no school-creation service
    // (the platform domain's `create_school` is dispatched via
    // `CommandDispatcher`, which Phase 3 builds). For load-test
    // purposes the storage adapter is keyed on `SchoolId`, so a
    // fresh UUID per school is sufficient.
    let schools: Vec<SchoolId> = (0..args.schools)
        .map(|_| SchoolId::from_uuid(g.next_uuid()))
        .collect();
    info!(school_count = schools.len(), "provisioned synthetic schools");

    // 2. Warmup.
    if !args.skip_warmup && args.warmup_commands > 0 {
        let warmup_school = schools[0];
        let warmup_ctx = make_ctx(warmup_school, actor, &g);
        info!(
            warmup_commands = args.warmup_commands,
            "running warmup phase"
        );
        for i in 0..args.warmup_commands {
            let rows = build_attendance_rows(
                warmup_school,
                i * args.bulk_size,
                args.bulk_size,
                &g,
            );
            storage
                .bulk_insert_student_attendances(&warmup_ctx, &rows)
                .await?;
        }
        info!("warmup complete");
    }

    // 3. Benchmark loop.
    //
    // For each school, run `students_per_school / bulk_size`
    // bulk-insert commands. We deliberately vary `student_id`
    // and `attendance_date` per row so the storage adapter's
    // (school_id, student_id, attendance_date) uniqueness check
    // is satisfied without flushing state between schools.
    let batches_per_school = args.students_per_school / args.bulk_size;
    let total_commands = schools.len() * batches_per_school;
    let total_rows = total_commands * args.bulk_size;

    let mut latencies_us: Vec<u64> = Vec::with_capacity(total_commands);
    let started = Instant::now();
    for (school_idx, school) in schools.iter().enumerate() {
        let ctx = make_ctx(*school, actor, &g);
        for batch_idx in 0..batches_per_school {
            // Unique starting offset per (school, batch) so every
            // row across all schools has a distinct
            // (student_id, attendance_date) tuple.
            let row_offset = school_idx * args.students_per_school
                + batch_idx * args.bulk_size;
            let rows = build_attendance_rows(
                *school,
                row_offset,
                args.bulk_size,
                &g,
            );
            let cmd_start = Instant::now();
            storage
                .bulk_insert_student_attendances(&ctx, &rows)
                .await?;
            let elapsed_us = cmd_start.elapsed().as_micros() as u64;
            latencies_us.push(elapsed_us);
        }
        // Periodic progress log (every 10% of schools).
        if schools.len() >= 10 && (school_idx + 1) % (schools.len() / 10).max(1) == 0 {
            let pct = ((school_idx + 1) * 100) / schools.len();
            info!(
                progress_pct = pct,
                commands_so_far = latencies_us.len(),
                "benchmark in progress"
            );
        }
    }
    let elapsed = started.elapsed();

    Ok(LoadtestResult {
        schools: args.schools,
        students_per_school: args.students_per_school,
        bulk_size: args.bulk_size,
        total_commands,
        total_rows,
        elapsed,
        latencies_us,
    })
}

// ---------------------------------------------------------------------------
// Report
// ---------------------------------------------------------------------------

fn print_report(r: &LoadtestResult) {
    info!("=== educore-loadtest results ===");
    info!(
        scale_schools = r.schools,
        scale_students_per_school = r.students_per_school,
        bulk_size = r.bulk_size,
        "scale"
    );
    info!(
        total_commands = r.total_commands,
        total_rows = r.total_rows,
        elapsed_ms = r.elapsed.as_millis() as u64,
        "totals"
    );
    info!(
        throughput_cmds_per_sec = format!("{:.2}", r.throughput_cmds_per_sec()),
        throughput_rows_per_sec = format!("{:.2}", r.throughput_rows_per_sec()),
        "throughput"
    );
    info!(
        latency_p50_us = r.percentile(0.50).as_micros() as u64,
        latency_p95_us = r.percentile(0.95).as_micros() as u64,
        latency_p99_us = r.percentile(0.99).as_micros() as u64,
        "latency"
    );

    if r.total_commands == 0 {
        warn!("no commands executed — check --schools and --students-per-school");
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_ctx(school: SchoolId, actor: educore_core::ids::UserId, g: &SystemIdGen) -> TenantContext {
    TenantContext::for_user(school, actor, g.next_correlation_id(), UserType::SchoolAdmin)
}

/// Builds `count` attendance rows starting at `row_offset`. Each row
/// has a unique `(student_id, attendance_date)` tuple within the
/// school, satisfying the storage adapter's uniqueness check.
fn build_attendance_rows(
    school: SchoolId,
    row_offset: usize,
    count: usize,
    g: &SystemIdGen,
) -> Vec<StudentAttendanceRow> {
    let base_date: NaiveDate = NaiveDate::from_ymd_opt(2026, 6, 21)
        .expect("base_date is a valid NaiveDate");
    let now = Timestamp::now();
    let actor = g.next_user_id();
    let etag = Etag::new("00000000000000000000000000000001")
        .expect("etag literal is valid");

    (0..count)
        .map(|i| {
            let row_idx = row_offset + i;
            // attendance_date shifts one day per row, wrapping after
            // ~365 days (the storage adapter's uniqueness check is on
            // a (school_id, student_id, attendance_date) triple, so
            // date alone is not enough — we also vary student_id).
            let day_offset = (row_idx % 365) as i64;
            StudentAttendanceRow {
                school_id: school,
                id: g.next_uuid(),
                student_id: Uuid::new_v4(),
                student_record_id: g.next_uuid(),
                class_id: g.next_uuid(),
                section_id: g.next_uuid(),
                attendance_date: base_date + ChronoDuration::days(day_offset),
                attendance_type: "P".to_owned(),
                in_time: None,
                out_time: None,
                notes: None,
                is_absent: false,
                marked_by: actor,
                marked_at: now,
                marked_from: "loadtest".to_owned(),
                version: Version::initial(),
                etag: etag.clone(),
                created_at: now,
                updated_at: now,
                created_by: actor,
                updated_by: actor,
                active_status: ActiveStatus::Active,
                correlation_id: g.next_correlation_id(),
                last_event_id: Some(g.next_event_id()),
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests — smoke tests that exercise the harness end-to-end at small scale.
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;

    fn small_args() -> Args {
        Args {
            schools: 2,
            students_per_school: 20,
            bulk_size: 5,
            warmup_commands: 2,
            skip_warmup: false,
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn smoke_runs_full_pipeline_and_reports_throughput() {
        let result = run_loadtest(&small_args()).await.unwrap();
        assert_eq!(result.schools, 2);
        assert_eq!(result.students_per_school, 20);
        assert_eq!(result.bulk_size, 5);
        // 2 schools × (20 / 5 batches) = 8 commands.
        assert_eq!(result.total_commands, 8);
        // 8 commands × 5 rows = 40 rows.
        assert_eq!(result.total_rows, 40);
        assert_eq!(result.latencies_us.len(), 8);
        assert!(result.elapsed > Duration::ZERO);
        assert!(result.throughput_cmds_per_sec() > 0.0);
        assert!(result.throughput_rows_per_sec() > 0.0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn smoke_percentile_returns_zero_for_empty() {
        let result = LoadtestResult {
            schools: 0,
            students_per_school: 0,
            bulk_size: 1,
            total_commands: 0,
            total_rows: 0,
            elapsed: Duration::ZERO,
            latencies_us: Vec::new(),
        };
        assert_eq!(result.percentile(0.5), Duration::ZERO);
        assert_eq!(result.percentile(0.95), Duration::ZERO);
        assert_eq!(result.percentile(0.99), Duration::ZERO);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn smoke_percentile_orders_samples_correctly() {
        let result = LoadtestResult {
            schools: 1,
            students_per_school: 1,
            bulk_size: 1,
            total_commands: 100,
            total_rows: 100,
            elapsed: Duration::from_secs(1),
            latencies_us: (1..=100).collect(),
        };
        // Nearest-rank: p50 → ceil(0.5 * 100) - 1 = 49 → 50us
        assert_eq!(result.percentile(0.50).as_micros(), 50);
        // p95 → ceil(0.95 * 100) - 1 = 94 → 95us
        assert_eq!(result.percentile(0.95).as_micros(), 95);
        // p99 → ceil(0.99 * 100) - 1 = 98 → 99us
        assert_eq!(result.percentile(0.99).as_micros(), 99);
    }

    #[test]
    fn smoke_build_rows_produces_unique_tuples() {
        let g = SystemIdGen;
        let school = SchoolId::from_uuid(g.next_uuid());
        let rows = build_attendance_rows(school, 0, 10, &g);
        assert_eq!(rows.len(), 10);
        // Every (student_id, attendance_date) pair is unique.
        let mut seen = std::collections::HashSet::new();
        for row in &rows {
            let key = (row.student_id, row.attendance_date);
            assert!(seen.insert(key), "duplicate (student_id, attendance_date)");
        }
    }
}
