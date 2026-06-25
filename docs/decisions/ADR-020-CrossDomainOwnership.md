# ADR-020: Cross-Domain Aggregate Ownership

## Status

Accepted, 2026-06-25.

## Context

The audit identified 3 aggregate ownership collisions where the same
aggregate is claimed by multiple domains:

1. **SubjectAttendance** — `docs/specs/academic/aggregates.md` vs
   `docs/specs/attendance/aggregates.md`
2. **ExamAttendance** — `docs/specs/academic/aggregates.md` vs
   `docs/specs/assessment/aggregates.md`
3. **SpeechSlider** — `docs/specs/cms/aggregates.md` vs
   `docs/specs/events/aggregates.md`

Without a canonical owner, each domain would publish a competing event,
the migration sequence would be ambiguous, and the repository handle
would be wired in two places.

## Decision

The engine follows the **writable-owner** rule: only the domain that
writes the aggregate's events owns the repository. Other domains can
read via the event log + projection, but cannot write.

### Collision 1: SubjectAttendance

- **Owner:** `attendance` domain
- **Reader:** `academic` (queries via `SubjectAttendanceView` projection)
- **Rationale:** Attendance data is rooted in the attendance process
  (student present/absent). Academic only needs read access to display
  it in the class roster.

### Collision 2: ExamAttendance

- **Owner:** `attendance` domain
- **Source events:** `assessment` publishes `ExamScheduled`,
  `ExamRescheduled`. Attendance subscribes and creates ExamAttendance
  rows.
- **Rationale:** Assessment declares the exam; attendance tracks
  presence during the exam. Attendance writes the records.

### Collision 3: SpeechSlider

- **Owner:** `cms` domain
- **Non-relationship:** events-domain has its own Slider/Carousel
  primitives (calendar announcement widgets) but these are distinct
  aggregates, not SpeechSlider.
- **Rationale:** SpeechSlider is a CMS page component (text overlay on
  an image used in school marketing pages). Events-domain uses
  calendar primitives, not slide widgets.

## Consequences

- Repository handles for SubjectAttendance, ExamAttendance, and
  SpeechSlider live in **one** crate each (attendance or cms).
- Cross-domain readers consume events + projections, not direct repos.
- Tests that span collisions must use the event bus + projection
  pattern (see `crates/domains/attendance/tests/` for example).

## Alternatives considered

- **B: Academic / Assessment owns.** Rejected — these domains don't
  have the daily-attendance business logic.
- **C: New shared domain.** Rejected — adds a 11th domain crate and a
  thin wrapper layer.

## References

- `docs/audit_reports/findings/wave6-specs-1.md`
- `docs/specs/attendance/aggregates.md` (SubjectAttendance)
- `docs/specs/assessment/aggregates.md` (ExamScheduled event)
- `docs/specs/cms/aggregates.md` (SpeechSlider)
