# Archive Manifest

This directory holds stale audit documents from the original 7-wave audit
(generated before Phase 6). They are superseded by:

- `docs/audit_reports/stub_vs_implementation.md` (Phase 1 baseline)
- `docs/audit_reports/remediation/12-production-readiness-roadmap.md`'s successor at
  `docs/audit_reports/remediation/13-production-readiness-v2.md` (Phase 6 roadmap)
- `docs/audit_reports/security_review.md` (Phase 5 security review)

Files are retained for historical reference only. Do not edit; they are frozen.

## Top-level stale files (9)

- `00-master-finding-table.md` — Master inventory from 7-wave audit (superseded by `remediation/13-production-readiness-v2.md`)
- `01-audit-domains.md` — Domain appendix from 7-wave audit (superseded by `stub_vs_implementation.md`)
- `02-audit-cross-cutting.md` — Cross-cutting appendix (superseded)
- `03-audit-adapters.md` — Adapter appendix (superseded)
- `04-audit-infra-tools.md` — Infra/tools appendix (superseded)
- `05-audit-documentation.md` — Documentation appendix (superseded)
- `06-audit-specs.md` — Specs appendix (superseded)
- `07-audit-workflows.md` — Workflows appendix (superseded by per-workflow tests in Phase 4)
- `08-audit-security-tests.md` — Security tests appendix (superseded by `security_review.md`)

## Old roadmap (1)

- `12-production-readiness-roadmap.md` — Pre-Phase-6 roadmap (superseded by
  `remediation/13-production-readiness-v2.md`). Moved from `remediation/` and
  archived rather than deleted so the history of how scope was revised remains visible.

## Findings (47 files across 46 audit targets)

### wave1 — Domain deep dives (10 files / 10 targets)

Raw Phase 1 wave 1 findings, one per domain bounded context. Superseded by
`docs/audit_reports/stub_vs_implementation.md`.

- `findings/wave1-academic.md`
- `findings/wave1-assessment.md`
- `findings/wave1-attendance.md`
- `findings/wave1-cms.md`
- `findings/wave1-communication.md`
- `findings/wave1-documents.md`
- `findings/wave1-events-domain.md`
- `findings/wave1-facilities.md`
- `findings/wave1-finance.md`
- `findings/wave1-hr-library.md` — covers two targets (hr + library)

### wave2 — Cross-cutting foundations (7 files / 7 targets)

Raw Phase 2 wave 2 findings, one per cross-cutting crate. Superseded.

- `findings/wave2-audit.md`
- `findings/wave2-events.md`
- `findings/wave2-operations.md`
- `findings/wave2-platform.md`
- `findings/wave2-rbac.md`
- `findings/wave2-settings.md`
- `findings/wave2-sync.md`

### wave3 — Adapters (10 files / 10 targets)

Raw Phase 1 wave 3 findings, one per adapter crate (6 port adapters + 4 storage adapters).
Superseded.

- `findings/wave3-auth.md`
- `findings/wave3-event-bus.md`
- `findings/wave3-files.md`
- `findings/wave3-integrations.md`
- `findings/wave3-notify.md`
- `findings/wave3-payment.md`
- `findings/wave3-storage-mysql.md`
- `findings/wave3-storage-postgres.md`
- `findings/wave3-storage-sqlite.md`
- `findings/wave3-storage-surrealdb.md`

### wave4 — Infra + tools (7 files / 7 targets)

Raw Phase 0 wave 4 findings. Superseded.

- `findings/wave4-cli-sdk.md`
- `findings/wave4-core.md`
- `findings/wave4-query-derive.md`
- `findings/wave4-storage-parity.md`
- `findings/wave4-storage-port.md`
- `findings/wave4-testkit.md`
- `findings/wave4-umbrella.md`

### wave5 — Documentation drift (6 files)

Raw doc-drift findings split into 6 chunks. Superseded by `docs/library-docs.md`
and the `docs/guides/` refresh.

- `findings/wave5-docs-1.md`
- `findings/wave5-docs-2.md`
- `findings/wave5-docs-3.md`
- `findings/wave5-docs-4.md`
- `findings/wave5-docs-5.md`
- `findings/wave5-docs-6.md`

### wave6 — Spec drift (4 files)

Raw spec-drift findings. Superseded by per-domain `docs/specs/<domain>/` refresh.

- `findings/wave6-specs-1.md`
- `findings/wave6-specs-2.md`
- `findings/wave6-specs-3.md`
- `findings/wave6-specs-4.md`

### wave7 — Cross-cutting concerns (3 files)

Raw security, tests, and workflows findings. Superseded by `security_review.md`
and the per-workflow integration tests added in Phase 4.

- `findings/wave7-security.md`
- `findings/wave7-tests.md`
- `findings/wave7-workflows.md`

## Totals

- 9 top-level stale files
- 1 old roadmap (`12-production-readiness-roadmap.md`)
- 47 raw wave-finding files (covering 46 distinct audit targets; `wave1-hr-library.md` covers two)

Total archived: **57 files** (55 stale audit docs per the 7-wave audit + the old roadmap).

## Why archived, not deleted

- The 7-wave audit captured real signal about gaps that existed at that point in time.
  Deleting would erase the trail of how the engine reached production readiness.
- The `remediation/` directory now contains the honest, current roadmap
  (`13-production-readiness-v2.md`) with `[x]`/`[~]`/`[ ]` semantics. The archived
  material remains available for archaeology.
- These files MUST NOT be re-edited or referenced as current source of truth. If you
  find yourself reaching for one, you probably want `stub_vs_implementation.md`,
  `security_review.md`, or `remediation/13-production-readiness-v2.md` instead.
