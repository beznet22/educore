# PORT-STORAGE-REPOS — Follow-up tracker

**Roadmap item:** PORT-STORAGE-REPOS (P0)
**Status:** Deferred — out of scope for Wave 7 (closure requires adding 215 repository traits across 10 domains)
**Created:** 2026-06-25

## Why deferred

The audit item description estimated ~80 missing aggregate repository handles, but the actual gap (counted 2026-06-25) is **215 missing traits across 10 domains**. The chart below shows the per-domain breakdown:

| Domain         | Spec aggregates | Code traits | Missing | Status |
| -------------- | --------------- | ----------- | ------- | ------ |
| academic       | 20              | 5           | **15**  | open |
| assessment     | 44              | 6           | **38**  | open |
| attendance     | 9               | 10          | 0       | ✅ closed (Wave 8.2) |
| cms            | 66              | 19          | **47**  | open |
| communication  | 27              | 27          | 0       | ✅ closed (Wave 8.2) |
| documents      | 23              | 3           | **20**  | open |
| facilities     | 15              | 15          | 0       | ✅ closed (Wave 8.2) |
| finance        | 73              | 44          | **29**  | open |
| hr             | 68              | 16          | **52**  | open |
| library        | 14              | 9           | 5       | partial (Wave 8.2 closed 3/8) |
| **TOTAL**      | **359**         | **154**     | **205** | 10/215 closed |

## Per-domain breakdown (the missing aggregates)

The full list is auto-generated from the gap (spec aggregate ≠ code trait) using the
roadmap script:

```bash
python3 scripts/enumerate-spec-coverage.py --missing-by-domain
```

Output saved to `docs/audit_reports/remediation/15-missing-repo-traits.md`.

## Suggested approach (Wave 8 candidate)

The work decomposes naturally by domain. Each domain is a single microtask:

| # | Domain       | Missing | Effort | Recommended wave |
| - | ------------ | ------- | ------ | ---------------- |
| 1 | communication | 1  | trivial    | Wave 8.1 |
| 2 | facilities    | 2  | small      | Wave 8.1 |
| 3 | attendance    | 3  | small      | Wave 8.1 |
| 4 | library       | 8  | small      | Wave 8.2 |
| 5 | academic      | 15 | medium     | Wave 8.3 |
| 6 | documents     | 20 | medium     | Wave 8.4 |
| 7 | finance       | 29 | large      | Wave 8.5 |
| 8 | assessment    | 38 | large      | Wave 8.6 |
| 9 | cms           | 47 | very large | Wave 8.7 |
| 10| hr            | 52 | very large | Wave 8.8 |

Wave 8.2 closed 10 traits (4 domains). Remaining 205 in 6 domains.

Each domain agent:
- Reads `docs/specs/<domain>/aggregates.md`
- Cross-references with `crates/domains/<domain>/src/repository.rs`
- For each missing aggregate, adds `pub trait <Name>Repository: Send + Sync` with:
  `fn find`, `fn save`, `fn delete`, `fn list_by_school`
- Updates `crates/domains/<domain>/src/lib.rs` re-export
- Adds at minimum one compile-only test (no impl needed — `dyn <Name>Repository` works)

Estimated total: 5,000-10,000 LoC across 10 commits, ~2-4 hours wall time with parallel agents.

## Acceptance criteria (Wave 8)

- [ ] All 215 missing `pub trait` declarations exist
- [ ] `cargo check -p educore-{domain}` exits 0 for all 10 domains
- [ ] `cargo test -p educore-{domain} --lib` exits 0 (existing tests still pass)
- [ ] `cargo run -p educore-core --bin lint --features lint` exits 0
- [ ] No `unwrap`/`expect`/`panic!` in production code (tests may use)
- [ ] No `as` numeric casts
- [ ] All traits documented
- [ ] Domain-side wiring updated: each trait is exported via `pub mod repository`
