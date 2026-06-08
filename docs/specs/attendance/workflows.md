# Attendance Domain — Workflows

Workflows orchestrate commands, queries, and policies to fulfill a
business goal. They are documented as ordered, conditional steps.

## Daily Attendance Capture

```text
1. The class teacher opens the attendance page for the day.
2. The system pre-populates the student list for the
   (class, section, date) and the existing attendance row (if any).
3. The teacher marks each student Present, Absent, Late, HalfDay,
   Holiday, or OnLeave.
4. The teacher clicks Save.
5. The engine issues one MarkStudentAttendance per student (or a
   single BulkMarkStudentAttendance for the whole section).
6. The engine emits StudentAttendanceMarked per student. For each
   student newly marked Absent, it emits StudentAbsentForDay.
7. Communication subscribes to StudentAbsentForDay and sends
   notifications to guardians (when notify=true).
8. The ClassAttendance summary is updated.
```

**Pre-conditions:**
- The date is a school day (not a holiday, not a future date).
- The class-section has students enrolled.
- The teacher is the assigned class teacher (or higher privilege).

**Failure paths:**
- Future date → `ValidationError::FutureDate`.
- Holiday → `ValidationError::Holiday` (override allowed only with
  `Attendance.Admin.Override` capability).
- Student not enrolled → `ValidationError::NotEnrolled`.
- Out-of-scope teacher → `ForbiddenError::OutOfScope`.

## Late Marking

```text
1. The school day begins. A student arrives after the
   LateThreshold (school setting, e.g. 08:00).
2. The class teacher marks the student Late.
3. The engine emits StudentAttendanceMarked with type L.
4. The ClassAttendance summary increments DaysLate.
5. If the school policy converts Late to HalfDay after N late
   arrivals in a month, the engine emits a system-internal
   SubjectAttendanceUpdated or StudentAttendanceUpdated for the
   affected students.
6. Communication may send a "late arrival" notice to the guardian
   (subscribed).
```

**Edge cases:**
- A student marked Late and then Present (correction) →
  `StudentAttendanceUpdated` with `from=L, to=P`.
- A student marked HalfDay and then Absent (correction) →
  `StudentAttendanceUpdated` with `from=F, to=A`. A second
  `StudentAbsentForDay` is **not** emitted (deduplicated per day).

## Absence Notification

```text
1. MarkStudentAttendance is issued with type=A and notify=true.
2. The engine emits StudentAbsentForDay.
3. Communication subscribes and dispatches an SMS/email/push to
   the guardian via the notification port.
4. The dispatch result (delivered, failed, queued) is recorded as
   a notification log (in the communication domain).
5. The teacher may re-mark the student Present, which emits
   StudentAttendanceUpdated and StudentAttendanceRestored.
6. The restored event may be used by communication to send an
   "all clear" notice (configurable).
```

**Edge cases:**
- Multiple updates to the same day → only the first transition
  into `Absent` emits `StudentAbsentForDay`. Subsequent
  re-marks to `Absent` are no-ops for the notification path.
- A student is absent and the guardian is not reachable →
  notification is queued and retried by the communication
  adapter.
- A staff member is marked absent → `StaffAbsentForDay` triggers
  the HR/finance reaction chain (not the guardian path).

## Bulk Import

```text
1. The attendance cell exports the day's attendance from a
   biometric device or HR system.
2. The cell uploads a CSV (or the device pushes via the API).
3. ImportAttendance is issued with the source string and the rows.
4. The engine creates a BulkAttendanceImport (status Pending) and
   one StudentAttendanceImport per row.
5. The engine emits BulkImportStarted.
6. The cell (or system) issues ValidateBulkImport.
7. The engine runs validation:
   - Students must be enrolled in the academic year.
   - Dates must not be in the future.
   - Attendance types must parse.
8. On success → BulkImportValidated. On failure → BulkImportFailed
   with per-row errors.
9. The cell issues CommitBulkImport.
10. The engine promotes each StudentAttendanceImport to a
    StudentAttendance row, emits BulkImportCommitted and one
    StudentAttendanceImported per row. For each absent student,
    emits StudentAbsentForDay.
11. Communication dispatches absence notifications.
```

**Edge cases:**
- Duplicate row in the import file → engine keeps the first and
  flags the rest as `RowError::Duplicate`.
- A student is in the import but not enrolled in the section for
  the date → `RowError::NotEnrolled`. The job moves to `Failed`
  and no rows are committed.
- The import overlaps with a previously committed import for the
  same source/date → engine rejects with
  `ValidationError::DuplicateImport`.

## Exam-Day Attendance

The exam-day per-subject attendance is owned by the assessment
domain and is documented in `docs/specs/assessment/workflows.md`.
The attendance domain subscribes to `ExamAttendanceMarked` and
updates the `ClassAttendance` summary used in report cards.

```text
1. The exam begins. The class teacher issues MarkExamAttendance
   (assessment command).
2. The engine emits ExamAttendanceMarked.
3. The attendance domain subscribes and recomputes
   ClassAttendance.days_present / days_absent for the relevant
   exam type.
4. The assessment domain uses the per-student mark during result
   computation.
```

## Attendance Reports

```text
1. The school admin or attendance cell requests a report.
2. The engine queries the relevant rows:
   - Daily: rows for a single date, grouped by class-section.
   - Weekly: rows for a date range, grouped by class-section or
     student.
   - Monthly: rows for a calendar month, grouped by class-section
     or student.
   - ByClass: percentage present per class over a date range.
   - ByStudent: percentage present per student over a date range.
   - ByStaff: percentage present per staff member over a date
     range.
3. The engine returns a typed report payload. The consumer adapter
   renders PDF/HTML/CSV.
```

**Pre-conditions:**
- The actor has the relevant report capability.
- The date range is bounded (e.g. one year max) to prevent
  runaway queries.

**Edge cases:**
- A date range spans a holiday → holidays are excluded from
  `days_opened` (the school calendar is consulted).
- A student was promoted mid-range → reports are split at the
  promotion boundary.

## Idempotency

- `MarkStudentAttendance` for the same
  `(student_id, attendance_date)` overwrites the prior row. The
  emit count tracks the number of updates; only the first
  transition into `Absent` triggers `StudentAbsentForDay`.
- `BulkMarkStudentAttendance` is idempotent on
  `(class_id, section_id, attendance_date)`. A second call within
  the same minute returns the prior result and emits
  `StudentAttendanceUpdated` rather than `StudentAttendanceMarked`.
- `ImportAttendance` is idempotent on
  `(school_id, source, attendance_date)`. A duplicate is rejected
  unless the school explicitly cancels the previous import.

## Failure Path Summary

| Stage              | Failure                                | Engine Response                          |
| ------------------ | -------------------------------------- | ---------------------------------------- |
| Daily marking      | Future date                            | `ValidationError::FutureDate`            |
| Daily marking      | Holiday                                | `ValidationError::Holiday`               |
| Daily marking      | Out-of-scope teacher                   | `ForbiddenError::OutOfScope`             |
| Subject marking    | Subject not assigned to section        | `ValidationError::SubjectNotAssigned`    |
| Staff marking      | Staff not active                       | `ValidationError::StaffInactive`         |
| Bulk import        | Row validation failure                 | `BulkImportFailed` with `RowError` list  |
| Bulk import        | Duplicate import                       | `ValidationError::DuplicateImport`       |
| Bulk commit        | Import not validated                   | `ValidationError::ImportNotValidated`    |
| Report             | Date range too large                   | `ValidationError::RangeTooLarge`         |
