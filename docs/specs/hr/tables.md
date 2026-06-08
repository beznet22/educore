# HR Domain — Tables

The HR domain is backed by the following tables from
`migrations/0010_hr.sql`. Each table maps to one or more aggregates;
the `aggregate` column tells you which aggregate owns the row.

| Table                              | Aggregate                 | Notes                                |
| ---------------------------------- | ------------------------- | ------------------------------------ |
| `sm_staffs`                        | Staff                     | The staff member                     |
| `sm_human_departments`             | Department                | A department                         |
| `sm_designations`                  | Designation               | A designation                        |
| `sm_assign_class_teachers`         | AssignClassTeacher        | A class teacher assignment           |
| `sm_leave_types`                   | LeaveType                 | A leave type                         |
| `sm_leave_defines`                 | LeaveDefine               | A leave entitlement                  |
| `sm_leave_requests`                | LeaveRequest              | A leave request                      |
| `sm_staff_attendences`             | StaffAttendance           | A daily attendance row               |
| `sm_staff_attendance_imports`      | StaffAttendanceImport     | A bulk attendance import row         |
| `sm_staff_registration_fields`     | StaffRegistrationField    | A staff registration custom field    |
| `staff_import_bulk_temporaries`    | StaffImportBulkTemporary  | A bulk staff import staging row      |
| `sm_hr_salary_templates`           | SalaryTemplate            | A salary grade template              |
| `sm_hourly_rates`                  | HourlyRate                | An hourly rate                       |
| `sm_hr_payroll_generates`          | PayrollGenerate (HR-owned; finance reads/pays) | Monthly payroll run |
| `sm_hr_payroll_earn_deducs`        | PayrollEarnDeduc (HR-owned; finance reads) | Payroll earnings/deductions line |
| `sm_leave_deduction_infos`         | LeaveDeductionInfo        | A leave deduction row on a payroll   |

## Notes

- Every table includes `school_id` for multi-tenant isolation. The
  `school_id` is `NOT NULL DEFAULT 1` for the bootstrap school.
- Every table includes `created_at`, `updated_at`, `created_by`,
  `updated_by`, `active_status` (where applicable). These are
  managed by the engine's storage adapter.
- `academic_id` references `sm_academic_years` (the per-year scope).
- The staff row carries `designation_id`, `department_id`,
  `role_id`, `user_id`, and `gender_id` as foreign keys to the
  platform's catalog. These are required; the engine refuses
  staff registration if any are missing.
- `is_saas` on `sm_designations` and `sm_human_departments` flags
  system-defined rows. The engine refuses to delete them.
- `staff_no`, `email`, `mobile`, and `user_id` are unique within a
  school; the storage adapter enforces this.
- `casual_leave`, `medical_leave`, and `metarnity_leave` (note the
  source spelling) are stored as `String` for backward
  compatibility; the typed projection stores them as `u32`.
- `merital_status` is a legacy alias of `marital_status`; the
  engine reads the canonical `marital_status`.
- `sm_staff_attendences` (note the spelling) is the canonical
  table; `attendance_type` is encoded as `P`, `L`, `A`, `H`, `F`.
- The payroll tables (`sm_hr_payroll_generates` and
  `sm_hr_payroll_earn_deducs`) are typed as HR aggregates and
  written by HR; the finance domain reads them and writes
  `payroll_payments` against them. The `is_partial` and
  `paid_amount` columns are the only finance-derived fields and
  are updated only by the finance domain.
- `sm_leave_deduction_infos` carries the per-payroll leave-
  deduction record. The HR domain writes it during payroll
  generation; finance reads it for transparency but does not
  mutate it.
- `staff_import_bulk_temporaries` is the staging table for bulk
  staff imports. The HR domain promotes rows into `sm_staffs` on
  success.
- The `Staff` aggregate's full set of fields spans two storage
  models: the main `sm_staffs` row plus the optional
  `staff_import_bulk_temporaries` row during the import process.
  Once promoted, the import row is soft-deleted and the staff
  row carries the canonical profile.
