# ADR-019: Public API Naming Convention

## Status

Accepted, 2026-06-25.

## Context

The audit report (`docs/audit_reports/findings/wave5-docs-2.md`) identified
that public API types are documented under names that differ from the code
(e.g., `<X>Identifier` in docs vs `<X>Id` in code, or vice versa).

Consumers copy doc examples into their code and discover the types don't
exist under those names, increasing support burden.

## Decision

The engine follows the **code-as-canonical** rule: public API names live
in the Rust source. Documentation (`docs/specs/`, `docs/library-docs.md`)
must use the exact name from `pub use` re-exports in
`crates/educore/src/lib.rs`.

When docs drift from code:
1. Fix the docs (preferred — lowest churn, no breaking change for consumers).
2. If the doc name is more discoverable, rename the code (requires ADR
   amendment + major version bump).

## Conventions

- **Identity types** use the `Id` suffix, not `Identifier`:
  - `StudentId`, `SchoolId`, `ClassId`, `UserId`, etc.
- **Event types** use the past-tense verb form (`StudentAdmitted`, not
  `StudentAdmission`).
- **Command types** use the imperative verb form (`AdmitStudent`, not
  `StudentAdmission`).
- **Aggregate types** use the noun form (`Student`, `Class`, `Book`).
- **Capability strings** use `<Domain>.<Aggregate>.<Action>` form
  (`Library.Book.Create`).

## Consequences

- Consumers can `cargo doc` to verify any name in docs.
- Lint rule (see ADR-013 reconciliation) flags doc files containing
  `.Identifier` or `.Admission` strings adjacent to Rust types.
- The `Educore` brand is preserved in prose; `educore` is the code
  namespace (see ADR-013).

## Alternatives considered

- **B: Rename code to match docs.** Rejected — higher churn, breaking
  change for existing consumers.
- **C: Pick one canonical set; update both.** Rejected — same churn as B
  with no compensating benefit.

## References

- `docs/audit_reports/findings/wave5-docs-2.md`
- `docs/decisions/ADR-013-CrateLayout.md` (brand convention)
- `docs/specs/<domain>/aggregates.md` (per-domain naming)
