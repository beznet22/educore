# Attendance Domain — Events

Quick reference of every event the attendance domain emits. Events
are immutable, append-only records. Every event carries a typed
`EventEnvelope` and is durably persisted to the aggregate's event
log.

| Event                              | Aggregate                  | Subscribers                                                                       | Description                                                                          | Durable? | Replicated? | Replayable? |
| ---------------------------------- | -------------------------- | --------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ | -------- | ----------- | ----------- |
| `StudentAttendanceMarked`          | `StudentAttendance`        | `attendance` (self)                                                               | A student's daily attendance was marked.                                              | yes      | yes         | yes         |
| `StudentAttendanceUpdated`         | `StudentAttendance`        | `attendance` (self)                                                               | A student's daily attendance was patched.                                             | yes      | yes         | yes         |
| `StudentAttendanceRestored`        | `StudentAttendance`        | `communication`                                                                   | A student was transitioned out of absence.                                            | yes      | yes         | yes         |
| `StudentAbsentForDay`              | `StudentAttendance`        | `communication`, `finance`, `academic`, `assessment`                              | A student was marked absent for a day.                                                | yes      | yes         | yes         |
| `StudentAttendanceImported`        | `BulkAttendanceImport`     | `communication`                                                                   | A single row from a bulk import was processed.                                        | yes      | yes         | yes         |
| `SubjectAttendanceMarked`          | `SubjectAttendance`        | `communication`                                                                   | A subject-wise attendance was marked.                                                | yes      | yes         | yes         |
| `SubjectAttendanceUpdated`         | `SubjectAttendance`        | —                                                                                 | A subject attendance row was patched.                                                 | yes      | yes         | yes         |
| `SubjectAbsentNotificationRequested` | `SubjectAttendance`      | `communication`                                                                   | An absence notification request was raised for a subject absence.                     | yes      | yes         | yes         |
| `StaffAttendanceMarked`            | `StaffAttendance`          | `hr`, `finance`, `communication`                                                  | A staff member's daily attendance was marked.                                         | yes      | yes         | yes         |
| `StaffAttendanceUpdated`           | `StaffAttendance`          | —                                                                                 | A staff attendance row was patched.                                                   | yes      | yes         | yes         |
| `StaffAbsentForDay`                | `StaffAttendance`          | `hr`, `finance`, `communication`                                                  | A staff member was marked absent for a day.                                           | yes      | yes         | yes         |
| `BulkImportStarted`                | `BulkAttendanceImport`     | —                                                                                 | A bulk import job entered `Pending`.                                                 | yes      | yes         | yes         |
| `BulkImportValidated`              | `BulkAttendanceImport`     | —                                                                                 | A bulk import job passed validation.                                                  | yes      | yes         | yes         |
| `BulkImportCommitted`              | `BulkAttendanceImport`     | —                                                                                 | A bulk import job was committed and its rows promoted.                                | yes      | yes         | yes         |
| `BulkImportFailed`                 | `BulkAttendanceImport`     | `communication`                                                                   | A bulk import job failed validation.                                                  | yes      | yes         | yes         |
| `BulkImportCancelled`              | `BulkAttendanceImport`     | —                                                                                 | A bulk import job was cancelled.                                                      | yes      | yes         | yes         |
| `AbsenceNotificationRequested`     | `StudentAttendance`        | `communication`                                                                   | A notification request was raised for a student absence.                              | yes      | yes         | yes         |
| `ClassAttendanceRecomputed`        | `ClassAttendance` (projection) | `assessment`                                                                   | A class attendance summary was recomputed.                                            | yes      | yes         | yes         |

**See also:** `docs/specs/attendance/events.md` for full Rust struct
definitions, the canonical `EventEnvelope`, and per-event
subscribers.
