# Attendance Domain Overview

## Purpose

The attendance domain owns the daily and per-period presence
record for students, the daily presence record for staff, and the
exam-day presence record for students. It captures how a school
knows who was present, who was late, who was absent, and who was
on approved leave — for a day, a period, a class, or an exam.

It is the source of truth for absence detection, attendance
reports, and the trigger for guardian notifications when a student
is absent. It depends on the academic domain for class, section,
subject, and student identity.

## Responsibilities

- Daily attendance capture for students, per class-section.
- Subject (period) attendance capture for students.
- Daily attendance capture for staff.
- Exam-day attendance capture (delegated to assessment for the
  dedicated exam-day aggregate, but consumed here for summaries).
- Absence detection (full-day and period-level).
- Absence notifications to guardians (event-driven; delivery is
  owned by communication).
- Bulk import of attendance from external systems (CSV, biometric
  devices).
- Attendance reports (daily, weekly, monthly, by class, by student,
  by staff).
- Late-marking rules and half-day rules.
- Holiday-aware attendance expectations.

## Boundaries

The attendance domain does **not** own:

- Student identity, class assignment, promotion (see
  `specs/academic/`).
- Exam-day per-subject attendance as an operational roll (the
  `ExamAttendance` aggregate is owned by assessment; this domain
  consumes it for summaries).
- Notification dispatch (see `specs/communication/`).
- Fees/fines for low attendance (see `specs/finance/`).

The attendance domain **does** provide identifier types and value
objects that other domains depend on: `StudentAttendanceId`,
`SubjectAttendanceId`, `StaffAttendanceId`,
`StudentAttendanceImportId`.

## Dependencies

- `smscore-core` — error types, identifier trait, validation.
- `smscore-platform` — `SchoolId`, `UserId`, `TenantContext`.
- `smscore-rbac` — capability checks.
- `smscore-events` — domain event publishing.
- `smscore-academic` — `StudentId`, `ClassId`, `SectionId`,
  `SubjectId`, `AcademicYearId`, `StaffId`, `StudentRecordId`.

## Domain Invariants

1. Every attendance aggregate is anchored to exactly one `SchoolId`.
2. A `StudentAttendance` is unique per
   `(school_id, student_id, attendance_date)` within an academic
   year.
3. A `SubjectAttendance` is unique per
   `(school_id, student_id, subject_id, attendance_date)`.
4. A `StaffAttendance` is unique per
   `(school_id, staff_id, attendance_date)`.
5. Attendance dates are not in the future relative to the school's
   `Clock` at the time of marking.
6. The same attendance cannot be both `Present` and `Absent`.
7. A student marked `Late` for the day contributes to
   `Late` count but counts as present for the day unless the school
   rule converts `Late` to `HalfDay` after a threshold.
8. A student marked `HalfDay` counts as `0.5` present for the day
   in all reports.
9. A student marked `OnLeave` is not absent; the leave is approved
   by a separate workflow (HR/staff domain for staff; academic
   leave-of-absence for students).
10. Bulk imports are idempotent on
    `(school_id, student_id, attendance_date, source)`; a duplicate
    import is rejected.
11. `StudentAbsentForDay` is emitted exactly once per
    `(student_id, attendance_date)` even if multiple attendance
    rows are later updated; the event is the source of truth for
    guardian notifications.
12. The attendance system never deletes a row; updates are append-
    only via the `Updated` events.
13. Attendance expectations are gated on the academic calendar: a
    date that is a holiday is not an "absent" date even when no
    attendance row exists.

## Aggregate Roots

| Aggregate                   | Root Type             | Purpose                                 |
| --------------------------- | --------------------- | --------------------------------------- |
| StudentAttendance           | `StudentAttendance`   | Daily presence per student              |
| SubjectAttendance           | `SubjectAttendance`   | Per-period presence per student         |
| StaffAttendance             | `StaffAttendance`     | Daily presence per staff                |
| ExamAttendance              | `ExamAttendance`      | Exam-day per-subject presence (delegate)|
| BulkAttendanceImport        | `BulkAttendanceImport`| A bulk import job                       |

Each aggregate is documented in detail under
`docs/specs/attendance/aggregates.md`.

## Cross-Domain Impact

When a student is marked absent for the day, the attendance domain
emits `StudentAbsentForDay`. The following domains may subscribe:

- `communication` — sends an SMS/push/email to the guardian.
- `finance` — may accrue an absence fine if the school's policy
  applies.
- `academic` — flags the student in the daily attendance summary
  for the class teacher.
- `assessment` — feeds the per-exam attendance summary used in
  report cards.

When a staff member is marked absent, `StaffAbsentForDay` is
emitted:

- `hr` — may deduct leave or trigger a substitute workflow.
- `finance` — applies no-pay rules where configured.
- `communication` — notifies the school admin and the staff's
  department.

When attendance is imported, `AttendanceImported` is emitted:

- `attendance` (self) — materializes the import rows.
- `finance` — none by default.

When exam attendance is marked (assessment-owned), the attendance
domain subscribes to `ExamAttendanceMarked` and produces
`ClassAttendance` summaries for the academic year.

## Consumers

- Web admin UI (class teacher, attendance cell).
- Teacher app (mark daily and period attendance).
- Mobile kiosk / biometric device.
- Parent app (view attendance, receive absence notification).
- AI agent (mark, update, query, report).

## Anti-Goals

- The attendance domain does not present data to humans. It exposes
  commands, events, and queries.
- The attendance domain does not deliver notifications. Delivery is
  a port; the domain publishes events.
- The attendance domain does not own HR leave-of-absence workflows
  for staff. It consumes `LeaveApproved` events from HR to mark
  staff as `OnLeave`.
- The attendance domain does not compute fees or fines. It emits
  events that finance consumes.
