# SMSengine Progress Tracker

This document tracks the implementation status of SMSengine against
the 17-phase build plan defined in `docs/build-plan.md`. Every row
starts in the **Planned** state and is flipped to **Implementing**
or **Done** as the corresponding phase lands.

## Workspace Status

The workspace has **34 crates**: the umbrella `smsengine` plus 33
internal crates, grouped below by the phase that ships them. Every
crate is scaffolded (`Cargo.toml`, `lib.rs`, `#[forbid(unsafe_code)]`,
`#[deny(missing_docs)]`); none are implemented yet.

| Crate                          | Phase | Spec'd | Implementing | Tested | Notes                          |
| ------------------------------ | ----- | ------ | ------------ | ------ | ------------------------------ |
| `smsengine`                    | -     | Yes    | No           | No     | Umbrella; re-exports all 33    |
| `smsengine-core`               | 0     | Yes    | No           | No     | Errors, ids, value objects, clock |
| `smsengine-query-derive`       | 0     | Yes    | No           | No     | `#[derive(DomainQuery)]` macro |
| `smsengine-storage`            | 0     | Yes    | No           | No     | `StorageAdapter` port + sub-ports |
| `smsengine-storage-postgres`   | 0     | Yes    | No           | No     | Primary adapter; `sqlx` + `rustls` |
| `smsengine-storage-parity`     | 16    | Yes    | No           | No     | Cross-adapter parity suite     |
| `smsengine-storage-mysql`      | 1     | Yes    | No           | No     | `MySQL 8.0+`, RLS via session var |
| `smsengine-storage-sqlite`     | 1     | Yes    | No           | No     | Embedded / offline; `json1`    |
| `smsengine-platform`           | 2     | Yes    | No           | No     | School, User, TenantContext    |
| `smsengine-rbac`               | 2     | Yes    | No           | No     | Capability, Role, Permission   |
| `smsengine-events`             | 2     | Yes    | No           | No     | Envelope crate; `DomainEvent`  |
| `smsengine-event-bus`          | 2     | Yes    | No           | No     | in-process, NATS, Redis impls  |
| `smsengine-audit`              | 2     | Yes    | No           | No     | `AuditLogEntry`, retention     |
| `smsengine-academic`           | 3     | Yes    | No           | No     | First vertical slice; 8 aggs   |
| `smsengine-assessment`         | 4     | Yes    | No           | No     | Exams, marks, results          |
| `smsengine-attendance`         | 5     | Yes    | No           | No     | Student/staff/subject/exam     |
| `smsengine-hr`                 | 6     | Yes    | No           | No     | Staff, leave, payroll          |
| `smsengine-finance`            | 7     | Yes    | No           | No     | Largest spec; double-entry     |
| `smsengine-facilities`         | 8     | Yes    | No           | No     | Dorm, transport, inventory     |
| `smsengine-library`            | 9     | Yes    | No           | No     | Books, issues, fines           |
| `smsengine-communication`      | 10    | Yes    | No           | No     | Notices, complaints, logs      |
| `smsengine-documents`          | 11    | Yes    | No           | No     | Forms, postal                  |
| `smsengine-cms`                | 12    | Yes    | No           | No     | Pages, news, testimonial       |
| `smsengine-events-domain`      | 13    | Yes    | No           | No     | Calendar (distinct from envelope) |
| `smsengine-settings`           | 14    | Yes    | No           | No     | Per-school config, language    |
| `smsengine-operations`         | 14    | Yes    | No           | No     | Bell schedule, substitution    |
| `smsengine-auth`               | 15    | Yes    | No           | No     | `AuthProvider` + JWT impl      |
| `smsengine-notify`             | 15    | Yes    | No           | No     | `NotificationProvider` + email/SMS |
| `smsengine-payment`            | 15    | Yes    | No           | No     | `PaymentProvider` + Stripe     |
| `smsengine-files`              | 15    | Yes    | No           | No     | `FileStorage` + S3/local       |
| `smsengine-integrations`       | 15    | Yes    | No           | No     | LMS, video-conferencing        |
| `smsengine-testkit`            | 16    | Yes    | No           | No     | In-memory impls of 6 ports     |
| `smsengine-sdk`                | 16    | Yes    | No           | No     | `Engine::builder()` facade     |
| `smsengine-cli`                | 16    | Yes    | No           | No     | Sample binary, dogfooding      |

Phase 17 ships no new crates; it hardens the workspace
(multi-tenant suite, load test, cross-compile, security review,
docs audit).

## Phase Progress

| Phase | Title                              | Crates                                                                 | Status   | Exit Criteria Met |
| ----- | ---------------------------------- | ---------------------------------------------------------------------- | -------- | ----------------- |
| 0     | Foundation                         | `core`, `query-derive`, `storage`, `storage-postgres`                  | Planned  | No                |
| 1     | Adapter parity (MySQL + SQLite)    | `storage-mysql`, `storage-sqlite`                                      | Planned  | No                |
| 2     | Cross-cutting foundations          | `platform`, `rbac`, `events`, `event-bus`, `audit`                     | Planned  | No                |
| 3     | Academic (first vertical slice)    | `academic`                                                             | Planned  | No                |
| 4     | Assessment                         | `assessment`                                                           | Planned  | No                |
| 5     | Attendance                         | `attendance`                                                           | Planned  | No                |
| 6     | HR                                 | `hr`                                                                   | Planned  | No                |
| 7     | Finance (largest spec)             | `finance`                                                              | Planned  | No                |
| 8     | Facilities                         | `facilities`                                                           | Planned  | No                |
| 9     | Library                            | `library`                                                              | Planned  | No                |
| 10    | Communication                      | `communication`                                                        | Planned  | No                |
| 11    | Documents                          | `documents`                                                            | Planned  | No                |
| 12    | CMS                                | `cms`                                                                  | Planned  | No                |
| 13    | Events domain (calendar)           | `events-domain`                                                        | Planned  | No                |
| 14    | Settings + Operations              | `settings`, `operations`                                               | Planned  | No                |
| 15    | Port adapters                      | `auth`, `notify`, `payment`, `files`, `integrations`                   | Planned  | No                |
| 16    | Test infrastructure + SDK          | `testkit`, `storage-parity`, `sdk`, `cli`                              | Planned  | No                |
| 17    | Production readiness               | (no new crates)                                                        | Planned  | No                |

## Documentation Status

All 269+ markdown files are spec'd. The split below mirrors the
directory tree under `docs/` plus the migration scripts under
`migrations/`.

| Directory / file                            | Count | Status   |
| ------------------------------------------- | ----- | -------- |
| Top-level docs (`docs/*.md`)                | 7     | Complete |
| `docs/specs/<domain>/` (15 domains x 11)    | 165   | Complete |
| `docs/ports/`                               | 7     | Complete |
| `docs/commands/` (15 domains)               | 15    | Complete |
| `docs/events/` (15 domains)                 | 15    | Complete |
| `docs/schemas/` (6 cross-cutting)           | 6     | Complete |
| `docs/schemas/sql-dialects/`                | 5     | Complete |
| `docs/schemas/data-migration/`              | 13    | Complete |
| `docs/decisions/` (ADRs)                    | 14    | Complete |
| `docs/diagrams/` (Mermaid)                  | 7     | Complete |
| `docs/research/`                            | 16    | Complete |
| `docs/guides/`                              | 18    | Complete |
| `migrations/README.md`                      | 1     | Complete |
| `migrations/engine/0000_engine_core.mysql.sql`           | 1     | Complete |
| `migrations/0001_*.sql` .. `0015_*.sql`     | 15    | Complete |
| `migrations/engine/` (3 dialect DDL + README) | 4    | Complete |

## Coverage Matrix Summary

The full matrix (226+ rows) is **machine-readable** and lives at
[`docs/coverage.toml`](coverage.toml) so CI can diff it on every
PR. The summary below rolls it up to the bucket level. The
**Implemented** column starts at 0 and grows as phases complete.

| Bucket                                                  | Total | Spec'd | Implemented |
| ------------------------------------------------------- | ----- | ------ | ----------- |
| Engine cross-cutting tables (6 x 3 dialects)            | 18    | 18     | 0           |
| Port traits                                             | 7     | 7      | 0           |
| Domain aggregates                                       | ~310  | ~310   | 0           |
| Domain commands                                         | ~225  | ~225   | 0           |
| Domain events                                           | ~280  | ~280   | 0           |
| Storage adapters                                        | 3     | 3      | 0           |
| Port adapters (5 ports + 1 cli binary)                  | 6     | 6      | 0           |
| Reference impls (JWT, email, SMS, Stripe, S3, local, LMS, video) | 8 | 8 | 0      |

The cross-cutting bucket is `outbox`, `audit_log`, `idempotency`,
`event_log`, `schema_registry`, `system_user` rendered in each of
the three dialect DDL files (`postgres`, `mysql`, `sqlite`).
Aggregate / command / event totals derive from the per-domain
specs in `docs/specs/<domain>/aggregates.md`,
`docs/commands/<domain>.md`, and `docs/events/<domain>.md`.

The current `docs/coverage.toml` has an initial scaffold of
~80 representative rows covering the engine cross-cutting tables,
the 7 port traits, and 1-3 aggregates per domain. The full
226+ row matrix is generated by the lint sub-module
(`smsengine-core::lint`, gated behind the `lint` Cargo feature)
once implementation begins.

## See also

- `docs/build-plan.md` § "The 17 phases" — the canonical phase plan
- `docs/build-plan.md` § "The Coverage Matrix" — the matrix schema and CI gate
- `docs/coverage.toml` — the machine-readable coverage matrix
- `docs/architecture.md` — the system map
- `AGENTS.md` § "Status" — high-level status
