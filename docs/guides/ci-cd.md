# CI/CD Guide

## Goal

Establish a continuous integration and deployment pipeline for an
SMSengine consumer application.

## Build

```yaml
name: build
on: [push, pull_request]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Format check
        run: cargo fmt --all -- --check
      - name: Clippy
        run: cargo clippy --workspace --all-targets -- -D warnings
      - name: Test
        run: cargo test --workspace
      - name: Build release
        run: cargo build --workspace --release
```

## Integration Tests

Integration tests require a real database. Use testcontainers:

```yaml
  integration:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_PASSWORD: test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --workspace --features integration
        env:
          DATABASE_URL: postgres://postgres:test@localhost/smscore_test
```

## Cross-Compilation

```yaml
  cross:
    strategy:
      matrix:
        target:
          - x86_64-unknown-linux-gnu
          - aarch64-unknown-linux-gnu
          - x86_64-apple-darwin
          - x86_64-pc-windows-msvc
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - run: cargo build --workspace --release --target ${{ matrix.target }}
```

## Coverage

```yaml
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin
      - name: Coverage
        run: cargo tarpaulin --workspace --out xml
      - name: Upload to codecov
        uses: codecov/codecov-action@v3
```

## Mutation Testing

```yaml
  mutants:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install mutants
        run: cargo install cargo-mutants
      - name: Mutate
        run: cargo mutants --workspace
```

## Performance Benchmarks

```yaml
  bench:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Bench
        run: cargo bench --workspace
      - name: Compare to baseline
        uses: .../benchmark-action@v1
        with:
          baseline: main
```

## Release

```yaml
  release:
    if: startsWith(github.ref, 'refs/tags/v')
    runs-on: ubuntu-latest
    needs: [build, integration, cross, coverage]
    steps:
      - uses: actions/checkout@v4
      - name: Build binaries
        run: |
          cargo build --release --target x86_64-unknown-linux-gnu
          cargo build --release --target aarch64-unknown-linux-gnu
      - name: Sign
        run: cosign sign-blob ...
      - name: Publish
        run: |
          gh release create ${{ github.ref_name }} \
            target/x86_64-unknown-linux-gnu/release/smscore \
            target/aarch64-unknown-linux-gnu/release/smscore
```

## Pre-commit Hooks

```yaml
  - repo: local
    hooks:
      - id: cargo-fmt
        name: cargo fmt
        entry: cargo fmt --all -- --check
        language: system
        pass_filenames: false
      - id: cargo-clippy
        name: cargo clippy
        entry: cargo clippy --workspace --all-targets -- -D warnings
        language: system
        pass_filenames: false
```

## Deployment

For containerized deployment:

```dockerfile
FROM rust:1.75 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/smscore /usr/local/bin/
ENTRYPOINT ["smscore"]
```

The image uses `debian-slim` (musl for static binaries) and includes
CA certificates for TLS.

## Database Migrations

The consumer's migrations run as a separate step:

```yaml
  deploy:
    runs-on: ubuntu-latest
    steps:
      - name: Migrate
        run: |
          cargo run --bin migrate -- up
        env:
          DATABASE_URL: ${{ secrets.DATABASE_URL }}
```

The engine does not own migrations; the consumer does.

## Observability

Export traces to Jaeger / Tempo via OpenTelemetry:

```rust
tracing_subscriber::registry()
    .with(tracing_opentelemetry::layer().with_tracer(opentelemetry_otlp::new_pipeline()
        .with_trace_config(...)
        .install_batch(opentelemetry_sdk::runtime::Tokio)?))
    .init();
```

Export metrics to Prometheus:

```rust
let recorder = PrometheusBuilder::new()
    .install_recorder()?;
```

## Backup

Database backups are run by the consumer:

```bash
pg_dump -Fc $DATABASE_URL > backups/$(date +%Y%m%d).dump
```

Encrypted and uploaded to S3 with versioning. Retention: 30 daily, 12
monthly, 7 yearly.

## Disaster Recovery

A documented runbook for restoring from backup, replaying events,
and rebuilding tenants. Tested quarterly.

## Cost Optimization

- Use spot instances for non-critical workloads.
- Use S3 Intelligent-Tiering for backups.
- Use connection pooling (max 20 per pod).
- Cache frequently-read aggregates (e.g. school info, current
  academic year).
- Index aggressively; re-index during off-peak hours.
