# Educore

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)]()
[![Edition](https://img.shields.io/badge/rust-edition%202021-blue.svg)]()
[![MSRV](https://img.shields.io/badge/rust-1.75%2B-blue.svg)]()
[![Status](https://img.shields.io/badge/status-scaffold-yellow.svg)]()

**Educore** is a reusable, embeddable **school-domain engine** for
production software. It captures the business behavior, workflows, and
rules required to operate a real school — admissions, attendance,
examinations, finance, payroll, communication, facilities, library,
transport, and compliance — and exposes them as a command-oriented,
event-driven kernel that any application can drive.

> **Not an application. Not a framework. A domain engine.**

## Why Educore

- **Compile-time safety.** Identifiers, fields, and operators are
  typed; consumer bugs fail at compile time, not at runtime.
- **Multi-tenant by default.** Every aggregate carries a `SchoolId`;
  cross-tenant references are structurally impossible.
- **Audit-first.** Every state change writes an immutable record.
- **Offline-capable.** State changes can be queued and reconciled.
- **AI-agent friendly.** A capability catalog lets tool-using LLMs
  drive the school safely.
- **Hexagonal.** Domain code depends on no adapter; consumers provide
  storage, auth, notify, payment, and file storage implementations.

## Workspace

This repository is a Cargo workspace with **34 crates** organized into 5 tiers:

### core (3 crates) — infrastructure

- `educore-core` — errors, identifiers, value objects, query AST.
- `educore-query-derive` — `#[derive(DomainQuery)]` proc macro.
- `educore-storage` — port trait.

### cross-cutting (7 crates) — cross-domain foundations

- `educore-platform` — tenant, school, user, session.
- `educore-rbac` — role, permission, capability.
- `educore-events` — envelope + bus port.
- `educore-events-domain` — calendar events.
- `educore-settings` — general settings.
- `educore-operations` — backups, jobs, system versions.
- `educore-audit` — audit log writer.

### domains (10 crates) — the 10 domain bounded contexts

- `educore-academic` — students, classes, sections, subjects.
- `educore-assessment` — exams, marks, results.
- `educore-attendance` — student & staff attendance.
- `educore-cms` — pages, news, content.
- `educore-communication` — notices, complaints, chat.
- `educore-documents` — forms, postal.
- `educore-facilities` — transport, dormitory, inventory.
- `educore-finance` — fees, payments, banking, payroll.
- `educore-hr` — staff, payroll, leave.
- `educore-library` — books, members, issues.

### adapters (9 crates) — port implementations

- 3 reference storage adapters: `educore-storage-postgres`, `educore-storage-mysql`, `educore-storage-sqlite`.
- 6 port adapters: `educore-auth`, `educore-notify`, `educore-payment`, `educore-files`, `educore-event-bus`, `educore-integrations`.

### tools (4 crates) — dev tooling

- `educore-testkit` — in-memory test adapters.
- `educore-storage-parity` — cross-adapter parity suite.
- `educore-cli` — sample binary CLI.
- `educore-sdk` — high-level consumer SDK.

### umbrella (1 crate)

- `educore` — re-exports the public surface of all 34 internal crates.

Internal crate directories are named without the `educore-` prefix
(`crates/domains/academic/`), while the published package name keeps
the prefix (`educore-academic`). The umbrella re-exports each
crate under its short name.

See `AGENTS.md` for the full layout, tier rules, and crate
inventory. See `CONTRIBUTING.md` for the spec-to-PR workflow.

## Quickstart (Scaffold Only)

This repository currently contains a **project scaffold** for the
engine. Domain logic, aggregates, value objects, commands, events,
repositories, and storage translations are pending implementation.

```bash
# Build the workspace (compiles, no domain logic yet)
cargo build --workspace

# Lint with workspace lints
cargo clippy --workspace --all-targets -- -D warnings

# Run the (sparse) test suite
cargo test --workspace
```

## Consumer-Facing Example (Target API)

Once implementation lands, a consumer wires the engine like this:

```rust
use educore::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::builder()
        .storage(PostgresAdapter::connect(env::var("DATABASE_URL")?).await?)
        .auth(JwtAuthProvider::from_env()?)
        .notify(EmailNotifier::from_env()?)
        .event_bus(InProcessBus::new())
        .clock(SystemClock::new())
        .id_gen(UuidV7Generator::new())
        .build()
        .await?;

    Ok(())
}
```

See `docs/library-docs.md` for the full consumer-facing walkthrough.

## Storage Adapters

Three reference adapters ship with the engine:

- **PostgreSQL** — primary target (`educore-storage-postgres`)
- **MySQL** — production target, MySQL 8.0+ (`educore-storage-mysql`)
- **SQLite** — embedded / offline mode (`educore-storage-sqlite`)

**Deferred** to a future release (not shipped, consumer may implement
in-tree on demand): SurrealDB, MongoDB. See
`docs/ports/storage.md#future-storage-backends-deferred` for the
rationale.

## Documentation

The authoritative source of truth is the `docs/` directory:

| Document                       | Purpose                              |
| ------------------------------ | ------------------------------------ |
| `docs/project-overview.md`     | Engine philosophy and scope          |
| `docs/architecture.md`         | System map                           |
| `docs/build-plan.md`           | Implementation roadmap               |
| `docs/code-standards.md`       | Engineering rules                    |
| `docs/library-docs.md`         | Consumer-facing SDK docs             |
| `docs/progress-tracker.md`     | Implementation status                |
| `docs/query_layer.md`          | Macro-driven query specification     |
| `docs/specs/<domain>/...`      | Per-domain specifications            |
| `docs/ports/<port>.md`         | Port contracts                       |
| `docs/commands/<domain>.md`    | Command catalogs                     |
| `docs/events/<domain>.md`      | Event catalogs                       |
| `docs/schemas/*.md`            | Cross-cutting schemas                |
| `docs/schemas/sql-dialects/`   | Per-dialect DDL conventions + runtime emission flow |
| `docs/schemas/data-migration/` | 12-file plan from legacy Schoolify schema to engine |
| `docs/decisions/*.md`          | Architectural decisions (ADRs)       |
| `docs/diagrams/*.md`           | Visual maps (Mermaid)                |
| `docs/research/*.md`           | Business knowledge extracted from the legacy Schoolify Laravel project (read-only analysis source) |
| `docs/guides/*.md`             | Implementation guides                |

The engine does **not** apply `.sql` migration files at runtime;
the schema is emitted at startup via `storage.create_schema().await`
from a typed macro AST and the canonical `migrations/engine/0000_*`
files embedded via `include_str!`. See
[`docs/schemas/sql-dialects/README.md`](docs/schemas/sql-dialects/README.md#runtime-ddl-emission--end-to-end-flow)
for the 5-step flow.

## License

Dual-licensed under either of:

- Apache License, Version 2.0 ([`LICENSE-APACHE`](LICENSE-APACHE))
- MIT License ([`LICENSE-MIT`](LICENSE-MIT))

at your option. The canonical text files are in the repo root.

For the full license FAQ (attribution, warranties, patents, SaaS
use, trademarks), see
[`docs/guides/license-faq.md`](docs/guides/license-faq.md).
