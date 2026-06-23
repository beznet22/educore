# Production-Readiness Audit - README

## What this is

A consolidated findings report for the Educore engine (a 34-crate Rust workspace for multi-tenant school administration). This directory contains the output of a 7-wave audit that produced 46 finding files in `docs/audit_reports/findings/` totalling 1,878 findings across the engine's domain crates, cross-cutting crates, adapters, infra+tools crates, documentation, spec folders, and three deep-dive audits (workflows, security, tests).

This is a **findings-only report**. No fixes or recommendations are included.


## Severity legend

- **Critical** = blocks deploy
- **High** = major gap / feature unusable
- **Medium** = minor broken
- **Low** = cosmetic

## Headline counts

- **Total findings:** 1878
- **Critical:** 473
- **High:** 666
- **Medium:** 548
- **Low:** 191

## File index

| File | Title |
| --- | --- |
| [`README.md`](README.md) | This file - index, scope, methodology, severity legend, known gaps |
| [`00-master-finding-table.md`](00-master-finding-table.md) | Master finding table - every finding, every severity |
| [`01-audit-domains.md`](01-audit-domains.md) | Per-domain appendix (10 bounded contexts) |
| [`02-audit-cross-cutting.md`](02-audit-cross-cutting.md) | Per-crate appendix (7 cross-cutting crates) |
| [`03-audit-adapters.md`](03-audit-adapters.md) | Per-adapter appendix (10 adapters) |
| [`04-audit-infra-tools.md`](04-audit-infra-tools.md) | Per-crate appendix (7 infra+tools crates) |
| [`05-audit-documentation.md`](05-audit-documentation.md) | Per-doc appendix (9 doc areas) |
| [`06-audit-specs.md`](06-audit-specs.md) | Per-spec appendix (4 spec audits) |
| [`07-audit-workflows.md`](07-audit-workflows.md) | Wave 7 workflows deep-dive appendix |
| [`08-audit-security-tests.md`](08-audit-security-tests.md) | Wave 7 security + tests deep-dive appendix |

## Methodology

Seven audit waves ran sequentially. Each wave produced one or more finding files in `docs/audit_reports/findings/`:

- **Wave 1 (domains, 10 files):** Per-bounded-context audit of `crates/domains/<d>/src/{aggregate,entities,value_objects,commands,events,services,repository,query,errors}.rs` against `docs/specs/<d>/*` and `docs/handoff/PHASE-<n>-HANDOFF.md`. Each finding carries an id of the form `DOMAIN-<d>-NNN` (or `DOM-<d>-NNN` / `DOM-HRLIB-NNN` for facilities/hr-library).
- **Wave 2 (cross-cutting, 7 files):** Per-crate audit of `crates/cross-cutting/<c>/`. Id prefix `CROSSCUT-<c>-NNN` or `CC-<c>-NNN`.
- **Wave 3 (adapters, 10 files):** Per-adapter audit of `crates/adapters/<a>/`. Id prefix `ADAPTER-<a>-NNN` or `ADAPT-<a>-NNN`.
- **Wave 4 (infra + tools, 7 files):** Per-crate audit of `crates/infra/<c>/` and `crates/tools/<c>/`. Id prefix `CORE-NNN`, `INFRA-QD-NNN`, `PORT-STORE-NNN`, `PAR-NNN`, `TOOL-TK-NNN`, `CLI-SDK-NNN`, `UMB-NNN`.
- **Wave 5 (documentation, 6 files):** Doc-by-doc audit of `docs/*.md` and `docs/**` against the codebase. Id prefix `DOC-<g>-NNN` with `DOC-LIB`, `DOC-QL`, `DOC-HO`, `DOC-CAT`, `DOC-PORT`, `DOC-SCHM` for finer-grained targets.
- **Wave 6 (specs, 4 files):** Per-spec-folder audit of `docs/specs/<d>/` for internal consistency and code-drift. Id prefix `SPEC-<g>-NNN`.
- **Wave 7 (deep audits, 3 files):** Cross-cutting deep audits on workflows (`WF-NNN`), security (`SEC-<area>-NNN`), and tests (`TST-NNN`).

Each finding has the structured fields: id, area, severity (Critical/High/Medium/Low), location, description, expected (per spec/docs), and evidence (file paths + line numbers).


## Severity distribution by source file

| Source file | Critical | High | Medium | Low | Total |
| --- | --- | --- | --- | --- | --- |
| `findings/wave1-academic.md` | 11 | 20 | 27 | 8 | 66 |
| `findings/wave1-assessment.md` | 19 | 56 | 25 | 0 | 100 |
| `findings/wave1-attendance.md` | 26 | 16 | 9 | 2 | 53 |
| `findings/wave1-cms.md` | 10 | 17 | 29 | 11 | 67 |
| `findings/wave1-communication.md` | 28 | 13 | 5 | 1 | 47 |
| `findings/wave1-documents.md` | 4 | 10 | 15 | 10 | 39 |
| `findings/wave1-events-domain.md` | 23 | 9 | 11 | 17 | 60 |
| `findings/wave1-facilities.md` | 5 | 8 | 16 | 3 | 32 |
| `findings/wave1-finance.md` | 12 | 57 | 16 | 0 | 85 |
| `findings/wave1-hr-library.md` | 7 | 7 | 11 | 4 | 29 |
| `findings/wave2-audit.md` | 8 | 12 | 8 | 2 | 30 |
| `findings/wave2-events.md` | 6 | 6 | 12 | 4 | 28 |
| `findings/wave2-operations.md` | 5 | 23 | 17 | 11 | 56 |
| `findings/wave2-platform.md` | 15 | 14 | 14 | 5 | 48 |
| `findings/wave2-rbac.md` | 7 | 18 | 9 | 2 | 36 |
| `findings/wave2-settings.md` | 0 | 10 | 12 | 6 | 28 |
| `findings/wave2-sync.md` | 10 | 10 | 6 | 1 | 27 |
| `findings/wave3-auth.md` | 6 | 12 | 15 | 5 | 38 |
| `findings/wave3-event-bus.md` | 5 | 8 | 9 | 0 | 22 |
| `findings/wave3-files.md` | 5 | 13 | 9 | 1 | 28 |
| `findings/wave3-integrations.md` | 5 | 10 | 20 | 7 | 42 |
| `findings/wave3-notify.md` | 12 | 24 | 31 | 7 | 74 |
| `findings/wave3-payment.md` | 7 | 11 | 5 | 1 | 24 |
| `findings/wave3-storage-mysql.md` | 5 | 7 | 9 | 3 | 24 |
| `findings/wave3-storage-postgres.md` | 13 | 13 | 15 | 6 | 47 |
| `findings/wave3-storage-sqlite.md` | 5 | 14 | 15 | 16 | 50 |
| `findings/wave3-storage-surrealdb.md` | 11 | 21 | 6 | 0 | 38 |
| `findings/wave4-cli-sdk.md` | 7 | 4 | 9 | 2 | 22 |
| `findings/wave4-core.md` | 6 | 12 | 7 | 1 | 26 |
| `findings/wave4-query-derive.md` | 5 | 6 | 10 | 7 | 28 |
| `findings/wave4-storage-parity.md` | 7 | 11 | 9 | 4 | 31 |
| `findings/wave4-storage-port.md` | 13 | 15 | 6 | 2 | 36 |
| `findings/wave4-testkit.md` | 2 | 5 | 13 | 8 | 28 |
| `findings/wave4-umbrella.md` | 3 | 3 | 10 | 2 | 18 |
| `findings/wave5-docs-1.md` | 3 | 17 | 15 | 2 | 37 |
| `findings/wave5-docs-2.md` | 20 | 8 | 0 | 0 | 28 |
| `findings/wave5-docs-3.md` | 4 | 17 | 6 | 2 | 29 |
| `findings/wave5-docs-4.md` | 8 | 13 | 11 | 3 | 35 |
| `findings/wave5-docs-5.md` | 4 | 7 | 16 | 9 | 36 |
| `findings/wave5-docs-6.md` | 34 | 12 | 4 | 0 | 50 |
| `findings/wave6-specs-1.md` | 22 | 17 | 6 | 0 | 45 |
| `findings/wave6-specs-2.md` | 10 | 13 | 7 | 0 | 30 |
| `findings/wave6-specs-3.md` | 5 | 16 | 7 | 3 | 31 |
| `findings/wave6-specs-4.md` | 1 | 6 | 11 | 7 | 25 |
| `findings/wave7-security.md` | 13 | 17 | 12 | 0 | 42 |
| `findings/wave7-tests.md` | 13 | 22 | 12 | 5 | 52 |
| `findings/wave7-workflows.md` | 23 | 6 | 1 | 1 | 31 |

## Severity distribution by audit area

| Area (top-level) | Critical | High | Medium | Low | Total |
| --- | --- | --- | --- | --- | --- |
| Domains | 145 | 209 | 160 | 52 | 566 |
| Cross-cutting | 51 | 93 | 78 | 31 | 253 |
| Adapters | 74 | 133 | 134 | 46 | 387 |
| Infra + Tools | 43 | 56 | 64 | 26 | 189 |
| Documentation | 73 | 78 | 56 | 20 | 227 |
| Spec folders | 38 | 52 | 31 | 10 | 131 |
| Workflows | 23 | 6 | 1 | 1 | 31 |
| Security | 13 | 17 | 12 | 0 | 42 |
| Tests | 13 | 22 | 12 | 5 | 52 |

## Severity distribution by target id prefix

| Target id prefix | Critical | High | Medium | Low | Total |
| --- | --- | --- | --- | --- | --- |
| `ADAPT-EB` | 5 | 8 | 9 | 0 | 22 |
| `ADAPT-MY` | 5 | 7 | 9 | 3 | 24 |
| `ADAPT-PAY` | 7 | 11 | 5 | 1 | 24 |
| `ADAPTER-AUTH` | 6 | 12 | 15 | 5 | 38 |
| `ADAPTER-FILE` | 5 | 13 | 9 | 1 | 28 |
| `ADAPTER-INT` | 5 | 10 | 20 | 7 | 42 |
| `ADAPTER-NOT` | 12 | 24 | 31 | 7 | 74 |
| `ADAPTER-PG` | 13 | 13 | 15 | 6 | 47 |
| `ADAPTER-SQ` | 5 | 14 | 15 | 16 | 50 |
| `ADAPTER-SR` | 11 | 21 | 6 | 0 | 38 |
| `CC-AUD` | 8 | 12 | 8 | 2 | 30 |
| `CC-EVT` | 6 | 6 | 12 | 4 | 28 |
| `CC-SYNC` | 10 | 10 | 6 | 1 | 27 |
| `CLI-SDK` | 7 | 4 | 9 | 2 | 22 |
| `CORE` | 6 | 12 | 7 | 1 | 26 |
| `CROSSCUT-OPS` | 5 | 23 | 17 | 11 | 56 |
| `CROSSCUT-PLAT` | 15 | 14 | 14 | 5 | 48 |
| `CROSSCUT-RBAC` | 7 | 18 | 9 | 2 | 36 |
| `CROSSCUT-SET` | 0 | 10 | 12 | 6 | 28 |
| `DOC-1` | 3 | 17 | 15 | 2 | 37 |
| `DOC-2` | 20 | 8 | 0 | 0 | 28 |
| `DOC-6` | 34 | 12 | 4 | 0 | 50 |
| `DOC-CAT` | 0 | 9 | 8 | 2 | 19 |
| `DOC-HO` | 0 | 6 | 2 | 2 | 10 |
| `DOC-LIB` | 4 | 5 | 2 | 0 | 11 |
| `DOC-PORT` | 8 | 4 | 3 | 1 | 16 |
| `DOC-QL` | 0 | 6 | 2 | 0 | 8 |
| `DOC-SCHM` | 4 | 7 | 16 | 9 | 36 |
| `DOM-FAC` | 5 | 8 | 16 | 3 | 32 |
| `DOM-HRLIB` | 7 | 7 | 11 | 4 | 29 |
| `DOMAIN-ACM` | 11 | 20 | 27 | 8 | 66 |
| `DOMAIN-ASS` | 19 | 56 | 25 | 0 | 100 |
| `DOMAIN-ATT` | 26 | 16 | 9 | 2 | 53 |
| `DOMAIN-CMS` | 10 | 17 | 29 | 11 | 67 |
| `DOMAIN-COM` | 28 | 13 | 5 | 1 | 47 |
| `DOMAIN-DOC` | 4 | 10 | 15 | 10 | 39 |
| `DOMAIN-EVD` | 23 | 9 | 11 | 17 | 60 |
| `DOMAIN-FIN` | 12 | 57 | 16 | 0 | 85 |
| `INFRA-QD` | 5 | 6 | 10 | 7 | 28 |
| `PAR` | 7 | 11 | 9 | 4 | 31 |
| `PORT-STORE` | 13 | 15 | 6 | 2 | 36 |
| `SEC-AUDIT` | 3 | 2 | 2 | 0 | 7 |
| `SEC-AUTH` | 5 | 6 | 5 | 0 | 16 |
| `SEC-PLAT` | 3 | 2 | 2 | 0 | 7 |
| `SEC-RBAC` | 2 | 3 | 1 | 0 | 6 |
| `SEC-SECRETS` | 0 | 1 | 1 | 0 | 2 |
| `SEC-STORAGE` | 0 | 3 | 1 | 0 | 4 |
| `SPEC-1` | 22 | 17 | 6 | 0 | 45 |
| `SPEC-2` | 10 | 13 | 7 | 0 | 30 |
| `SPEC-3` | 5 | 16 | 7 | 3 | 31 |
| `SPEC-4` | 1 | 6 | 11 | 7 | 25 |
| `TOOL-TK` | 2 | 5 | 13 | 8 | 28 |
| `TST` | 13 | 22 | 12 | 5 | 52 |
| `UMB` | 3 | 3 | 10 | 2 | 18 |
| `WF` | 23 | 6 | 1 | 1 | 31 |

## How to read this report

- The master table (00) lists every finding as a one-line row, sorted by severity then by source file. Use it for triage and to find every finding in a particular source file.
- Appendices 01-06 group findings by their **target** (the crate, doc, or spec folder the finding is about). Use these to see all problems attributable to a single target.
- Appendix 07 (workflows) and appendix 08 (security + tests) are deep-dive audits that cross-cut multiple crates; the targets within them are sub-areas of the deep audit.
- Each finding block contains: id, source file path, severity, area, location, description, expected (per spec/docs), and evidence (file paths + line numbers + excerpts).

## Known gaps


- **Source-file format drift:** Three finding files use `### FINDING DOMAIN-<d>-NNN` headers (wave1-documents.md, wave1-events-domain.md, wave1-finance.md) instead of the more common `### FINDING N` + `- **id:** ...` pattern. Both forms are preserved in this report (the heading displays the id directly for the three drift files).
- **Overlap risk:** Some findings are repeated across wave files when the same defect was observable from multiple audit angles (e.g. a violation of `AGENTS.md` tier rules surfaces in both the domain audit and the cross-cutting audit). Each finding is listed once, in the appendix whose target id prefix matches the finding id. The master table lists all 1,878 findings from all 46 files.
- **Wave 7 (workflows, security, tests) was the most consequential deep audit.** It contributed 125 findings (3 files) that cut across multiple crates and surfaced systemic issues: state-machine incompleteness, missing saga compensation, RBAC bypass paths, secret-handling inconsistencies, parity-test scaffolding gaps.
- **No code was modified.** This report is read-only aggregation. Remediation is downstream work.

## Provenance


All findings were written by Wave 1-7 subagents in this audit session and are stored under `docs/audit_reports/findings/`. This directory contains the consolidated output only.
