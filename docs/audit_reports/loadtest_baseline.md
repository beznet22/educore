# Load Test Baseline

**Generated:** Phase 5 Step 2, Engine Production Readiness ferment
**Scope:** Engine command pipeline at small-scale SaaS deployment
(100 schools × 10k students ≈ 1M total).
**Harness:** `crates/tools/loadtest/` (committed 2026-07-01)
**Spec authority:** `docs/build-plan.md` Phase 17 Task 2 +
`docs/guides/test-strategy.md` § "Performance Tests"

---

## Goal

Establish a repeatable, scripted benchmark of the engine's command
pipeline at the Phase 17 Task 2 target scale and publish a baseline
throughput + latency table. Future runs (CI, release-blocking
regressions, post-Phase-5 changes) compare against this baseline;
regressions >10% are flagged.

## Methodology

### Scale

- **100 schools** × **10,000 students per school** = ~1,000,000
  total attendance rows. Matches `docs/build-plan.md` Phase 17
  Task 2 verbatim.
- **Bulk size:** 50 rows per `bulk_insert_student_attendances`
  command (configurable via `--bulk-size`).
- **Total commands:** 100 × (10,000 / 50) = **20,000 bulk commands**.
- **Total rows:** 20,000 × 50 = **1,000,000 rows**.

### Backend

- **In-memory** via `educore-testkit::test_world()`. The harness
  intentionally isolates the engine's command-dispatch +
  outbox-drain + transaction-commit cost from per-backend SQL
  variance. Production benchmarks against SurrealDB / PG / MySQL
  live under `crates/tools/storage-parity/`.
- All 7 ports (storage, auth, notify, payment, files,
  integrations, event-bus) are wired to in-memory impls. The bus
  is the default `InProcessEventBus` (1024-channel capacity,
  4096-replay-log).

### Command under test

- `StorageAdapter::bulk_insert_student_attendances` — the
  Phase 5 bulk-marking path documented in
  `docs/ports/storage.md`. Per-row validation: `school_id` must
  match `TenantContext::school_id`; `(school_id, student_id,
  attendance_date)` must be unique within the school.

### Warmup

- 100 warmup commands run before timing starts. Allocator +
  mutex + tokio worker pools are warmed so the first measured
  command reflects steady-state cost.

### Concurrency

- Tokio multi-thread runtime with 4 worker threads. Commands are
  dispatched serially per-school to avoid the
  `(school_id, student_id, attendance_date)` uniqueness check
  racing across batches; the harness exercises the **single-
  threaded hot path** of the storage adapter's bulk-insert.

### Latency measurement

- Per-command `Instant::now()` before/after the storage call.
  Microsecond resolution via `Duration::as_micros()`. Percentiles
  computed via nearest-rank (NIST primary): `index =
  ceil(p × N) − 1`, clamped to `[0, N−1]`.

### Output

- All metrics written to stdout via `tracing::info!` (workspace
  clippy lints forbid `println!`). The `RUST_LOG` env var
  controls verbosity; the default is `info`.

## CLI

```text
# Full Phase 17 Task 2 target (default args)
$ cargo run --release -p educore-loadtest

# Override scale for a 1-second laptop smoke test
$ cargo run --release -p educore-loadtest -- \
    --schools 1 --students-per-school 100 --bulk-size 10

# Increase bulk size to reduce command count (fewer dispatch
# round-trips, more rows per SQL INSERT in production backends)
$ cargo run --release -p educore-loadtest -- --bulk-size 200
```

## Results Table (Baseline — DEFERRED)

| Metric                          | Value      | Source                                      |
| ------------------------------- | ---------- | ------------------------------------------- |
| Run timestamp                   | _deferred_ | CI captures this per-build                  |
| Git SHA                         | _deferred_ | CI captures this per-build                  |
| Runner (CPU, RAM)               | _deferred_ | CI runner label                             |
| Throughput (commands/sec)       | _deferred_ | `LoadtestResult::throughput_cmds_per_sec()` |
| Throughput (rows/sec)           | _deferred_ | `LoadtestResult::throughput_rows_per_sec()` |
| Latency p50 (µs)                | _deferred_ | `LoadtestResult::percentile(0.50)`          |
| Latency p95 (µs)                | _deferred_ | `LoadtestResult::percentile(0.95)`          |
| Latency p99 (µs)                | _deferred_ | `LoadtestResult::percentile(0.99)`          |
| Total wall-clock (ms)           | _deferred_ | `LoadtestResult::elapsed`                   |
| Memory peak RSS (MB)            | _deferred_ | CI captures `/usr/bin/time -v`              |

### Why deferred?

The agent environment that produced this harness does not have
the same hardware characteristics as CI. Per the task brief:
**"actual benchmark run is deferred to CI with proper
hardware."** The first authoritative baseline run will be
captured by CI on a pinned runner (e.g. `ubuntu-22.04` 4-core
16 GB) and the numbers above will be filled in. Until then, the
harness itself is the deliverable: it is the script that
produces the baseline reproducibly.

### How to fill in the table

```bash
# Local one-shot run
cargo run --release -p educore-loadtest -- \
    --schools 100 --students-per-school 10000 --bulk-size 50 \
    2>&1 | tee loadtest-local.txt

# CI: capture git SHA + runner label
git rev-parse HEAD >> docs/audit_reports/loadtest_baseline.md
echo "Runner: $RUNNER_LABEL" >> docs/audit_reports/loadtest_baseline.md

# Parse tracing output into the table
# (the `info!(latency_p50_us = ...)` etc. lines are stable JSON-
# ish key=value output; a small awk/sed pass fills the table).
```

## Comparison Targets

The build plan does not pin a hard throughput number for the
in-memory backend — production-target benchmarks live against
PG/MySQL/SurrealDB. As a soft signal:

- **Harness compiles:** `cargo build --release -p educore-loadtest`
  exit 0.
- **Harness smoke-tests pass:** `cargo test -p educore-loadtest`
  exit 0; 4 unit tests cover the wiring + percentile math.
- **No regression >10%:** future CI runs diff the table above;
  a >10% drop in `throughput_cmds_per_sec` or a >10% increase
  in `latency_p95` opens an investigation issue.

## Risks / Caveats

- **In-memory only.** The harness does not exercise SQL adapter
  overhead (network round-trips, prepared-statement caching,
  deadlock retry, etc.). Those are owned by
  `crates/tools/storage-parity/`.
- **Single-thread hot path.** The harness serializes commands
  per-school to keep the uniqueness check deterministic. Real
  workloads will fan-out across schools and across the bulk
  pipeline; concurrency overhead is measured separately under
  the parity suite.
- **No `CommandDispatcher` integration yet.** The harness drives
  the storage adapter directly because Phase 3's production
  `CommandDispatcher` is in flight. When Phase 3 lands, this
  harness is updated to route through the dispatcher (RBAC +
  idempotency + outbox + audit pipeline) and the baseline is
  re-published with the additional overhead accounted for.

## Future Work

1. **Dispatcher integration** — re-route the bulk-insert calls
   through `CommandDispatcher::dispatch` (Phase 3 deliverable) to
   measure the full pipeline cost (RBAC check + idempotency
   lookup + outbox append + audit row + bus publish) instead of
   the raw storage-port cost.
2. **Cross-adapter load matrix** — run the same scale (100 ×
   10k) against each of the 4 storage adapters (SurrealDB /
   PG / MySQL / SQLite) and publish a comparison table.
3. **Concurrency sweep** — vary the worker-thread count
   (1 / 2 / 4 / 8 / 16) to characterize the engine's
   scalability ceiling on the in-memory backend.
4. **Memory profiling** — capture peak RSS at the 1M-row scale
   to validate the engine's memory footprint against a real
   SaaS deployment's budget.

## Reproducibility

The harness is hermetic:

- No network access required.
- No external services (DB, queue, cache) required.
- All randomness is `Uuid::new_v4()` from the `uuid` crate; the
  harness does not depend on a clock source beyond
  `Timestamp::now()` for row-level metadata, which does not
  affect throughput/latency measurements.
- The in-memory backend is freshly constructed per run; no
  shared state across runs.

A developer can clone the repo, run the command above, and
obtain the same numbers (modulo hardware variance) on the same
hardware.
