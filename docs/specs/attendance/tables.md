# Attendance Domain — Tables

The attendance domain is backed by the following tables. Each table
maps to one or more aggregates; the `aggregate` column tells you
which aggregate owns the row.

| Table                              | Aggregate                  | Notes                                  |
| ---------------------------------- | -------------------------- | -------------------------------------- |
| `attendance_student_attendances`           | StudentAttendance          | Daily student presence                 | <!-- derive_skip -->
| `attendance_subject_attendances`           | SubjectAttendance          | Per-period student presence            | <!-- derive_skip -->
| `attendance_staff_attendances`             | StaffAttendance            | Daily staff presence                   | <!-- derive_skip -->
| `attendance_student_attendance_imports`    | StudentAttendanceImport    | Staging row for student import         | <!-- derive_skip -->
| `attendance_staff_attendance_imports`      | StaffAttendanceImport      | Staging row for staff import           | <!-- derive_skip -->
| `student_attendance_bulks`         | AttendanceBulk             | Denormalized staging row               | <!-- derive_skip -->
| `class_attendances`                | ClassAttendance            | Per-(student, exam_type) summary       | <!-- derive_skip -->
| `assessment_exam_attendances`              | ExamAttendance (assessment)| Exam-day per-subject roll (delegated)  | <!-- derive_skip -->
| `assessment_exam_attendance_children`      | ExamAttendanceChild (assessment) | Per-student exam attendance       | <!-- derive_skip -->

## Notes

- Every table includes `school_id` for multi-tenant isolation. The
  `school_id` is `NOT NULL DEFAULT 1` for the bootstrap school.
- Every table includes `created_at`, `updated_at`, `created_by`,
  `updated_by`, `active_status` (where applicable). These are
  managed by the engine's storage adapter.
- `attendance_type` stores the legacy single-character codes
  (`P`, `A`, `L`, `F`, `H`). The engine maps these to the typed
  `AttendanceStatus` enum on read.
- `record_id` and `student_record_id` reference
  `student_records.id` and are the per-year enrollment handle.
- `class_attendances` carries the `days_opened`, `days_absent`,
  `days_present` summary used in report cards. The attendance
  domain recomputes this from the underlying events; the table
  serves as a cached projection.
- `assessment_exam_attendances` and `assessment_exam_attendance_children` are
  physically defined in the attendance migration for convenience
  but their aggregate is owned by the **assessment** domain.
- The `student_attendance_bulks` table is a denormalized staging
  representation used by the consumer's bulk import wizard. The
  engine commits its rows into `attendance_student_attendances`.
