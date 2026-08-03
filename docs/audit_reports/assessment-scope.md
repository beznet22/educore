# educore-assessment invariant scope

**Generated:** Wave 196 (commit `fdeb774` + 1)
**Spec file:** `docs/specs/assessment/aggregates.md`
**Code file:** `crates/domains/assessment/src/aggregate.rs`
**Tests dir:** `crates/domains/assessment/tests/`

## Scope summary

- **44 aggregate headers** in spec (file duplicated; ~22 unique)
- **29 stub aggregates** (spec text is only typed-id uniqueness)
- **15 real-spec aggregates** (spec defines meaningful invariants beyond typed-id)

## Per-real-aggregate status

| # | Aggregate | Spec lines | Real invariants | Code state | Test coverage |
|---|-----------|-----------:|----------------:|------------|---------------|
| 1 | ExamType | 41 | 5 | stub (id+school_id only) | 0 |
| 2 | Exam | 38 | 6+ | partial (2 impl blocks) | ~4 |
| 3 | ExamSetup | 31 | ? | stub | 0 |
| 4 | ExamSchedule | 36 | ? | stub (1 impl block) | 0 |
| 5 | ExamAttendance | 30 | ? | stub | 0 |
| 6 | ExamSetting | 30 | ? | stub | 0 |
| 7 | ExamSignature | 27 | ? | stub | 0 |
| 8 | MarksGrade | 32 | ? | stub | 0 |
| 9 | MarkStore | 33 | ? | stub | 0 |
| 10 | MarksRegister | 33 | ? | stub (1 impl block) | 0 |
| 11 | ResultStore | 37 | ? | stub (1 impl block) | 0 |
| 12 | SeatPlan | 35 | ? | stub (1 impl block) | 0 |
| 13 | OnlineExam | 49 | ? | stub | 0 |
| 14 | QuestionBank | 31 | ? | stub | 0 |
| 15 | TeacherEvaluation | 32 | ? | stub | 0 |

## Current test status

- 185 tests passing, 0 failing, 0 ignored
- Tests live in `crates/domains/assessment/tests/` (21 files)
- `workflows.rs` (56 test fns) and `wave29_final.rs` (19 test fns) are the bulk
- Per-aggregate tests are sparse — most aggregates have only 0-4 test fns

## The stub problem

The `educore-assessment` domain has the same architectural issue as `educore-hr` had
at Wave 171: the spec defines real behavior but the code is just typed-id
placeholders. Building these from scratch in the same wave-by-wave pattern that
worked for HR would take ~15 waves and risks the "building placeholder-stub
aggregates from scratch is anti-pattern" failure mode documented in
`docs/audit_reports/remediation/19-session-handoff.md`.

## Recommended path

1. **Wave 197**: Build a dispatcher wrapper generator tool that can mass-produce
   `dispatch_X` wrappers from a structured manifest. Use it to wrap all service
   functions across all 37 crates, not just assessment. This is the highest
   production-readiness impact work and unblocks all downstream work.
2. **Wave N**: After the dispatcher layer is in place, come back to assessment
   and apply the HR-style per-aggregate sweep (mutators + port traits + service
   functions + behavioral tests + spec reconciliation). The dispatcher wrapper
   for each new service function is then a one-line change.

## Recommended dispatcher wrapper manifest entry

For `educore-assessment`, the first wrapper should be on `ExamType::create_exam_type`
(service function: `create_exam_type`). This mirrors the Wave 192 pattern.

Co-Authored-By: Antigravity <antigravity@google.com>
