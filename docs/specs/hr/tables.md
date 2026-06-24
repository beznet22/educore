# HR Domain — Tables

The HR domain is backed by the following tables from
`migrations/0010_hr.sql`. Each table maps to one or more aggregates;
the `aggregate` column tells you which aggregate owns the row.

| Table                              | Aggregate                 | Notes                                |
| ---------------------------------- | ------------------------- | ------------------------------------ |
| `hr_staffs`                        | Staff                     | The staff member                     | <!-- derive_skip -->
| `hr_departments`             | Department                | A department                         | <!-- derive_skip -->
| `hr_designations`                  | Designation               | A designation                        | <!-- derive_skip -->
| `hr_assign_class_teachers`         | AssignClassTeacher        | A class teacher assignment           | <!-- derive_skip -->
| `hr_leave_types`                   | LeaveType                 | A leave type                         | <!-- derive_skip -->
| `hr_leave_defines`                 | LeaveDefine               | A leave entitlement                  | <!-- derive_skip -->
| `hr_leave_requests`                | LeaveRequest              | A leave request                      | <!-- derive_skip -->
| `hr_staff_attendances`             | StaffAttendance           | A daily attendance row               | <!-- derive_skip -->
| `attendance_staff_attendance_imports`      | StaffAttendanceImport     | A bulk attendance import row         | <!-- derive_skip -->
| `hr_staff_registration_fields`     | StaffRegistrationField    | A staff registration custom field    | <!-- derive_skip -->
| `hr_staff_import_bulk_temporaries`    | StaffImportBulkTemporary  | A bulk staff import staging row      | <!-- derive_skip -->
| `hr_salary_templates`           | SalaryTemplate            | A salary grade template              | <!-- derive_skip -->
| `hr_hourly_rates`                  | HourlyRate                | An hourly rate                       | <!-- derive_skip -->
| `hr_payroll_generates`          | PayrollGenerate (HR-owned; finance reads/pays) | Monthly payroll run | <!-- derive_skip -->
| `hr_payroll_earn_deducs`        | PayrollEarnDeduc (HR-owned; finance reads) | Payroll earnings/deductions line | <!-- derive_skip -->
| `hr_leave_deduction_infos`         | LeaveDeductionInfo        | A leave deduction row on a payroll   | <!-- derive_skip -->

## Notes

- Every table includes `school_id` for multi-tenant isolation. The
  `school_id` is `CHAR(36) NOT NULL` (UUIDv7) for the active school.
- Every table includes `id`, `created_at`, `updated_at`,
  `created_by`, `updated_by`, `active_status`, `version`, `etag`,
  `last_event_id`, `correlation_id`, `source`. These are managed
  by the engine's storage adapter; the seven engine invariants
  per `docs/schemas/database-schema.md` § 2, § 5, § 9.
- `academic_id` references `academic_academic_years` (the per-year scope).
- The staff row carries `designation_id`, `department_id`,
  `role_id`, `user_id`, and `gender_id` as foreign keys to the
  platform's catalog. These are required; the engine refuses
  staff registration if any are missing.
- `is_system_defined` on `hr_designations` and `hr_departments`
  flags system-defined rows. The engine refuses to delete them.
- `staff_number`, `email`, `mobile`, and `user_id` are unique
  within a school; the storage adapter enforces this.
- `casual_leave_quota`, `medical_leave_quota`, and
  `maternity_leave_quota` are stored as `DECIMAL(4,1)`. The legacy
  schema had `metarnity_leave` (typo); the engine's column is
  `maternity_leave_quota` (correct spelling).
- `marital_status` is the canonical column; the legacy
  `merital_status` (typo) is renamed to `marital_status` in the
  migration per `docs/schemas/data-migration/06-field-data-flow.md`.
- `attendance_staff_attendances` (formerly
  `hr_staff_attendances` with a typo) is the canonical table;
  `attendance_type` is encoded as `P`, `L`, `A`, `H`, `F`.
- The payroll tables (`hr_payroll_generates` and
  `hr_payroll_earn_deducs`) are typed as HR aggregates and
  written by HR; the finance domain reads them and writes
  `payroll_payments` against them. The `is_partial` and
  `paid_amount` columns are the only finance-derived fields and
  are updated only by the finance domain.
- `hr_leave_deduction_infos` carries the per-payroll leave-
  deduction record. The HR domain writes it during payroll
  generation; finance reads it for transparency but does not
  mutate it.
- `hr_staff_import_bulk_temporaries` is the staging table for bulk
  staff imports. The HR domain promotes rows into `hr_staffs` on
  success.
- The `Staff` aggregate's full set of fields spans two storage
  models: the main `hr_staffs` row plus the optional
  `hr_staff_import_bulk_temporaries` row during the import process.
  Once promoted, the import row is soft-deleted and the staff
  row carries the canonical profile.
