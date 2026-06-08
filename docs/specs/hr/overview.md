# HR Domain Overview

## Purpose

The HR domain owns the lifecycle of every school employee, the
payroll that compensates them, the leave they take, the attendance
they record, and the catalog of departments, designations, salary
grades, and hourly rates that organize them. It is the human-
resources spine of the school.

The domain is multi-tenant: every staff record is anchored to a
`SchoolId`. A single binary can serve many schools with independent
staff rosters, salary templates, and leave policies.

## Responsibilities

- Staff registration, profile, and lifecycle.
- Departments and designations.
- Class teacher and subject teacher assignments.
- Leave types, leave policies, leave requests, approvals.
- Staff attendance and bulk attendance imports.
- Payroll generation, approval, and payment.
- Salary templates and hourly rates.
- Leave deduction computation on payroll.
- Staff registration field customization.
- Bulk import of staff records.

## Boundaries

The HR domain does **not** own:

- Student identity, classes, or attendance (see `specs/academic/` and
  `specs/attendance/`).
- Banking, expenses, or wallet (see `specs/finance/`). The HR domain
  computes payroll; the finance domain records the payment.
- Authentication, RBAC, or platform identity (see `specs/platform/`
  and `specs/rbac/`). The HR domain refers to `UserId` and `RoleId`
  but does not own them.

The HR domain **does** own every staff record, every leave, every
payroll line, and every hourly rate. The cross-domain bridge to
finance happens through the `PayrollGenerate` and `PayrollEarnDeduc`
aggregates (HR-owned writes; finance reads and pays).

## Dependencies

- `smscore-core` — error types, result, identifier trait.
- `smscore-platform` — `SchoolId`, `UserId`, `TenantContext`.
- `smscore-rbac` — capability checks.
- `smscore-events` — domain event publishing.
- `smscore-academic` — `ClassId`, `SectionId`, `SubjectId`,
  `AcademicYearId` (for class teacher and subject teacher
  assignments).

## Domain Invariants

1. Every staff record is anchored to a `SchoolId`.
2. A `Staff` belongs to at most one `Department` and one
   `Designation` at a time.
3. A `Staff` has exactly one `UserId` binding (one platform user per
   staff member).
4. A `LeaveDefine` is unique by `(role_id, type_id)` or by
   `(user_id, type_id)` per school and academic year.
5. A `LeaveRequest` is `pending` until approved or rejected.
6. A `LeaveRequest` is approved only if the staff has sufficient
   remaining `LeaveDefine` balance.
7. A `PayrollGenerate` is generated before it is approved, and
   approved before it is paid. `paid` is terminal.
8. `gross_salary == basic_salary + total_earning` and
   `net_salary == gross_salary - total_deduction - tax` on a
   `PayrollGenerate`.
9. A `Staff` cannot be deleted while active
   `SmAssignClassTeacher`, `LeaveRequest`, or `PayrollGenerate`
   references it.
10. A `LeaveType` cannot be deleted while any `LeaveDefine` or
    `LeaveRequest` references it.
11. A `Department` or `Designation` cannot be deleted while any
    `Staff` references it.
12. An `HourlyRate` is unique by `grade` within a school.
13. A `SalaryTemplate` is unique by `salary_grades` within a school.

## Aggregate Roots

| Aggregate                   | Root Type                       | Purpose                                      |
| --------------------------- | ------------------------------- | -------------------------------------------- |
| Staff                       | `Staff`                         | Staff identity, profile, lifecycle          |
| Department                  | `Department`                    | A department in the school                  |
| Designation                 | `Designation`                   | A job title                                  |
| LeaveType                   | `LeaveType`                     | A leave category (e.g. Casual, Sick)        |
| LeaveDefine                 | `LeaveDefine`                   | The leave entitlement per role or user      |
| LeaveRequest                | `LeaveRequest`                  | A request for leave                          |
| StaffAttendance             | `StaffAttendance`               | A daily attendance row for a staff member   |
| StaffAttendanceImport       | `StaffAttendanceImport`         | A staging row for bulk attendance imports   |
| AssignClassTeacher          | `AssignClassTeacher`            | A class teacher assignment                   |
| HourlyRate                  | `HourlyRate`                    | An hourly rate per grade                     |
| SalaryTemplate              | `SalaryTemplate`                | A reusable salary grade and structure        |
| PayrollGenerate             | `PayrollGenerate`               | The monthly payroll run for a staff member  |
| PayrollEarnDeduc            | `PayrollEarnDeduc`              | A single earnings or deductions line         |
| LeaveDeductionInfo          | `LeaveDeductionInfo`            | A per-payroll leave-deduction record         |
| StaffRegistrationField      | `StaffRegistrationField`        | A custom field on the staff registration    |
| StaffImportBulkTemporary    | `StaffImportBulkTemporary`      | A staging row for a bulk staff import        |

Each aggregate is documented in detail under
`docs/specs/hr/aggregates.md`.

## Cross-Domain Impact

When a `Staff` is registered, the HR domain emits `StaffRegistered`.
The following domains may subscribe:

- `rbac` — assign the staff's role to the platform user.
- `finance` — bind the staff's salary template or hourly rate.
- `communication` — send a welcome message to the staff.

When a `LeaveRequest` is approved, the HR domain emits
`LeaveApproved`. The finance domain subscribes and:

- Adjusts the staff's leave balance for the period.
- When payroll is generated, computes the extra-leave deduction
  via `LeaveDeductionInfo`.

When a `PayrollGenerate` is paid by the finance domain, the HR domain
receives the `PayrollPaid` event and:

- Updates the staff's payroll history.
- Closes the payroll.

When a `Staff` is unregistered, the HR domain emits
`StaffUnregistered`. The following domains may subscribe:

- `academic` — release any class teacher or subject teacher
  assignment.
- `finance` — close the staff's payroll.
- `rbac` — revoke the staff's role.

## Consumers

- Web admin UI (HR manager).
- Mobile staff app (apply for leave, view attendance, view payslips).
- Mobile teacher app (apply for leave, mark attendance).
- Desktop payroll tool (generate payroll, print payslips).
- Bank reconciliation (verify payroll payments).
- AI agent (approve leave, generate payroll, register staff).

## Anti-Goals

- The HR domain does not connect to any biometric or attendance
  hardware. Attendance ingestion is a port.
- The HR domain does not pay staff. The finance domain pays; the HR
  domain computes the amount.
- The HR domain does not own platform identity. It binds a
  `UserId` to a `Staff` but does not manage passwords or
  authentication.
- The HR domain does not present data to humans. It exposes
  commands, events, and queries.
