# Attendance Domain — Commands

Quick reference of every command the attendance domain exposes. These
commands cover daily student attendance, subject attendance, staff
attendance, bulk imports, and absence notification requests.

The "Events" column lists the events the command emits; consult the
per-domain spec for payload structure.

| Command                          | Capability                | Description                                                                                       | Events                                                                                          | Idempotent? | Offline? |
| -------------------------------- | ------------------------- | ------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- | ----------- | -------- |
| `MarkStudentAttendance`          | `Attendance.Mark`         | Mark a single student's daily attendance (P / A / L / F / H).                                      | `StudentAttendanceMarked`, `StudentAbsentForDay` (on first A transition)                        | yes         | yes      |
| `UpdateStudentAttendance`        | `Attendance.Update`       | Patch an existing student attendance roll.                                                         | `StudentAttendanceUpdated`, `StudentAbsentForDay` or `StudentAttendanceRestored` on transition  | no          | yes      |
| `MarkSubjectAttendance`          | `Attendance.Mark`         | Mark subject-wise attendance for a student on a date.                                             | `SubjectAttendanceMarked`, `SubjectAbsentNotificationRequested` (if absent + notify)            | yes         | yes      |
| `MarkStaffAttendance`            | `Attendance.Mark`         | Mark a staff member's daily attendance.                                                            | `StaffAttendanceMarked`, `StaffAbsentForDay` (on first A transition)                            | yes         | yes      |
| `BulkMarkStudentAttendance`      | `Attendance.BulkMark`     | Mark attendance for an entire section in one command.                                             | `StudentAttendanceMarked` (per student), `StudentAbsentForDay` (per absent, dedup)             | yes         | yes      |
| `ImportAttendance`               | `Attendance.Import`       | Create a `BulkAttendanceImport` job in `Pending` with N rows.                                     | `BulkImportStarted`, `StudentAttendanceImported` (per row)                                       | no          | no       |
| `ValidateBulkImport`             | `Attendance.Import`       | Run validation on a pending import.                                                               | `BulkImportValidated` or `BulkImportFailed` (with row errors)                                   | no          | no       |
| `CommitBulkImport`               | `Attendance.Import`       | Promote validated rows to `StudentAttendance`.                                                    | `BulkImportCommitted`, `StudentAttendanceImported` (per row), `StudentAbsentForDay` (per absent)| no          | no       |
| `CancelBulkImport`               | `Attendance.Import`       | Cancel a pending or validated import.                                                             | `BulkImportCancelled`                                                                           | no          | yes      |
| `RequestAbsenceNotification`     | `Attendance.Notify`       | Request that the communication domain dispatch an absence notification.                            | `AbsenceNotificationRequested`                                                                  | no          | yes      |

## Notes

- Exam-day attendance (`MarkExamAttendance`, `UpdateExamAttendance`)
  belongs to the assessment domain. See
  `docs/commands/assessment.md`.
- `ClassAttendance` is a recomputed projection; it is not written by
  any direct command.
- `Attendance.Admin.Correct` is a hard-delete capability reserved
  for data-correction flows; the regular commands do not expose
  delete.

**See also:** `docs/specs/attendance/commands.md` for full Rust struct
definitions, pre-conditions, effects, and edge-case handling.
