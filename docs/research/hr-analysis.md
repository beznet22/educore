# HR Domain — Business Analysis

## Purpose

The HR domain owns the school's staff lifecycle:
recruitment, onboarding, leave, attendance,
designations, and payroll. The HR domain is
tightly coupled to the finance domain (payroll
flows from HR attendance and leave) and to the
academic domain (staff who teach).

This document describes how HR works in real
schools, with the edge cases that real schools
hit.

## Key Concepts

- **Staff** — a person employed by the school
  (teacher, accountant, driver, etc.).
- **Department** — an organizational unit
  ("Mathematics Department", "Administration").
- **Designation** — a job title
  ("Senior Teacher", "Primary Teacher", "Driver").
- **LeaveType** — a category of leave
  ("Sick Leave", "Casual Leave", "Earned Leave",
  "Maternity Leave").
- **LeaveApplication** — a staff's request for
  leave.
- **LeaveDeduction** — a per-leave-type
  deduction amount (e.g. "1 day of unpaid leave
  deducts ₹500").
- **SalaryTemplate** — a salary structure
  (basic, allowances, deductions).
- **PayrollPeriod** — a month/year payroll
  period.
- **HourlyRate** — an hourly rate for part-time
  staff.
- **StaffAttendance** — a staff's daily clock-
  in / clock-out.
- **StaffDocument** — a staff's document
  (contract, ID, certificates).

## Real-World Scenarios

### Staff Onboarding

A school hires a new staff member:

1. The HR admin creates a `Staff` aggregate
   with the personal details (name, contact,
   address, emergency contact).
2. The staff is linked to a `User` account
   (for portal access).
3. The staff is assigned a `Department`,
   `Designation`, and a `SalaryTemplate`.
4. The staff's `StaffNo` is auto-generated.
5. The staff's documents (contract, ID copy,
   certificates) are uploaded.
6. The staff is added to the relevant class
   routines and subject assignments (in the
   academic domain).
7. The finance domain subscribes to
   `StaffOnboarded` and prepares the staff's
   payroll template.

### Staff Profile Update

A staff member's contact information changes.
The HR admin updates the `Staff` aggregate. The
audit log captures the change.

### Department and Designation Management

The school has departments and designations:

- **Departments**: Mathematics, Science,
  English, Social Studies, Administration,
  Accounts, Transport.
- **Designations**: Principal, Vice Principal,
  Senior Teacher, Teacher, Lab Assistant,
  Accountant, Driver, Conductor, Peon.

The HR admin maintains the catalog. A staff is
linked to one department and one designation.

### Staff Resignation

A staff member resigns. The HR admin:

1. Records the resignation date.
2. Marks the staff's status as `Resigned`.
3. Generates the final payroll (pro-rated for
   the partial month).
4. Returns the staff's documents.
5. Removes the staff from class routines and
   subject assignments (in the academic
   domain).
6. The finance domain finalizes the staff's
   outstanding payments.

The engine emits `StaffResigned`; downstream
systems react.

### Staff Termination

A school terminates a staff member for cause.
The HR admin:

1. Records the termination with a reason.
2. Marks the staff's status as `Terminated`.
3. The staff's portal access is revoked.
4. The final payroll is generated (with
   possible deductions for notice period).
5. The staff is removed from class routines
   and subject assignments.

The audit log captures the termination with
the reason and the principal's approval.

### Leave Application

A staff member applies for leave:

1. The staff submits a `LeaveApplication`
   with the leave type, the from / to dates,
   and the reason.
2. The application is routed to the staff's
   reporting manager for approval.
3. The manager approves (or rejects with a
   reason).
4. The HR admin records the approval.
5. The leave is reflected in the staff's
   leave balance.
6. The finance domain is informed for payroll
   deduction (if unpaid leave).

The engine's `LeaveApplication` aggregate
captures the lifecycle.

### Leave Types and Balances

A school has multiple leave types with
different policies:

- **Sick Leave** — 12 days per year, requires
  a medical certificate for > 3 consecutive
  days.
- **Casual Leave** — 12 days per year, no
  carry-forward.
- **Earned Leave** — accumulated, can be
  carried forward up to 30 days, can be
  encashed at retirement.
- **Maternity Leave** — 180 days, per
  regulation.
- **Paternity Leave** — 15 days, per
  regulation.
- **Unpaid Leave** — beyond the leave
  entitlement, deducts from salary.

The engine's `LeaveType` aggregate captures
the policy; the `LeaveApplication` checks the
balance.

### Leave Approval Workflow

A leave application is approved by:

1. The immediate supervisor (e.g. department
   head).
2. The HR admin (for leave beyond 3 days).
3. The principal (for leave beyond 7 days).

The engine's workflow is configurable per
school. A simple school has a single approver;
a complex school has a multi-step workflow.

### Substitute Teacher

A teacher is on leave. The school arranges a
substitute:

1. The HR admin (or the department head)
   identifies a substitute.
2. The substitute is assigned to the absent
   teacher's classes for the leave period.
3. The substitute receives an extra allowance
   (per the school's policy).

The engine's `SubstituteAssignment` captures
the arrangement.

### Staff Attendance

Staff members clock in when they arrive and
clock out when they leave. The engine's
`StaffAttendance` aggregate records:

- Clock-in time.
- Clock-out time.
- Working hours.
- Overtime (if applicable).
- Late arrival (per the school's policy).
- Early departure (per the school's policy).

A staff member who forgets to clock in / out
has a manual entry (with the principal's
approval).

### Overtime

A staff works beyond the normal hours. The
engine computes the overtime based on the
`HourlyRate` and the `StaffAttendance`. The
overtime is added to the monthly payroll.

### Payroll Generation

At the end of the month, the finance domain
generates payroll. The HR domain provides:

- The salary template (per staff).
- The leave deductions (from approved leave
  applications).
- The overtime (from staff attendance).
- The loan deductions (if any).
- The advance deductions (if any).

The finance domain computes the net salary.
The HR domain's role is to provide the inputs.

### Staff Performance Review

A school conducts annual performance reviews.
The engine's `StaffReview` aggregate captures:

- The review period.
- The reviewer (principal or department
  head).
- The ratings (per dimension).
- The comments.
- The recommendations (promotion, increment,
  training, etc.).

The review is confidential; only the principal
and HR see the details.

### Staff Promotion

A staff is promoted. The HR admin:

1. Updates the `Designation` (e.g. "Teacher"
   → "Senior Teacher").
2. Updates the `SalaryTemplate` (with the
   new structure).
3. The increment is effective from a
   specific date.

The engine's `StaffPromoted` event captures
the change; the finance domain's salary
template is updated.

### Staff Training

A staff attends a training. The HR admin
records the training:

- The training title.
- The training provider.
- The dates.
- The cost.
- The certificate (optional).

The engine's `StaffTraining` aggregate
captures the training history.

### Staff Loan

A staff takes a loan from the school (e.g.
salary advance). The HR admin:

1. Records the loan amount, the
   installments, and the start date.
2. The loan is repaid via monthly payroll
   deductions.

The engine's `StaffLoan` aggregate captures
the loan and the per-month deduction.

### Staff Document Management

A staff's documents (contract, ID, certificates,
training records) are stored in the engine.
The engine's `StaffDocument` aggregate captures
the metadata; the file is in the consumer's
file storage.

## Business Rules

1. A `Staff` belongs to exactly one `SchoolId`.
2. A `Staff` is linked to exactly one
   `Department` and one `Designation` at a
   time.
3. A `Staff`'s status transitions are
   `Active → Resigned`, `Active → Terminated`,
   `Active → Retired`. No other transitions.
4. A `LeaveApplication` requires the leave
   type, the from / to dates, and the reason.
5. A `LeaveApplication` is approved by the
   configured approver(s) per the school's
   workflow.
6. A `LeaveApplication` cannot exceed the
   staff's leave balance for the type.
7. A `SalaryTemplate` is unique by
   `(school_id, grade)`.
8. A `StaffAttendance` is recorded for every
   working day.
9. A `Staff`'s portal access is revoked on
   `Resigned` or `Terminated`.
10. A `StaffLoan` cannot exceed the staff's
    annual salary (configurable per school).
11. A `StaffPromotion` requires the principal's
    approval.

## Edge Cases

### Leave Application for Past Dates

A staff member submits a leave application
for last week (they were sick and did not
apply on time). The engine accepts the late
application with a configurable grace
period; the principal approves retroactively.

### Leave Application for Future Dates

A staff applies for leave next month. The
engine accepts; the application is in
`Pending` status. The approver reviews when
the date approaches.

### Leave Balance Exhausted

A staff has used all 12 sick leave days. They
apply for an additional 3 days. The engine
flags the application as `ExceedsBalance`;
the principal may approve as `Unpaid Leave`.

### Multiple Approvers

A school has a 3-step approval workflow
(supervisor → HR → principal). The engine
supports the multi-step workflow with a
configurable approver chain.

### Substitute Not Found

A teacher is on leave. No substitute is
available. The class is cancelled. The
engine's `ClassCancelled` event fires; the
academic domain's routine is updated; the
attendance domain marks the day as
"Cancelled."

### Staff on Maternity Leave

A staff member is on maternity leave for 6
months. Her leave balance is restored at the
end of the leave (per the policy). Her class
assignments are temporarily reassigned.

### Staff Working Part-Time

A part-time teacher works 3 days a week. The
engine's `Staff` aggregate has a
`part_time` flag and a `working_days`
configuration. The payroll is pro-rated.

### Staff with Multiple Roles

A staff member teaches in two departments
(e.g. Mathematics and Computer Science). The
engine supports a primary department and
secondary affiliations.

### Staff Portal Access After Resignation

A staff resigns. Their portal access is
revoked on the resignation date. They cannot
log in. The engine's authentication rejects
their credentials.

### Audit of Staff Changes

A regulator audits the school. They want to
see every staff change in the last year. The
engine's audit log shows the hires, the
resignations, the promotions, the salary
changes, with the actor and the reason.

## Notes for SMSengine Implementation

- The **hr** crate depends on
  `smscore-academic` for class and subject
  assignments and `smscore-finance` for
  payroll integration.
- The HR domain is **eventually consistent**
  with finance (payroll), academic (class
  assignments), and library (membership).
- The HR domain's **leave workflow** is
  configurable per school. The engine's
  approval chain is a value object.
- The HR domain's **payroll inputs** are
  consumed by the finance domain. The engine
  emits `StaffAttendanceRecorded`,
  `LeaveApplicationApproved`, and other
  events; finance aggregates them in the
  payroll.
- The HR domain's **portal access** is
  tied to the `User` aggregate in the
  platform domain. The engine revokes
  access on `StaffResigned` /
  `StaffTerminated`.
- The HR domain's **staff documents** are
  storage-port driven. The engine's
  `StaffDocument` aggregate is the
  metadata; the file is in the consumer's
  storage.
- The HR domain's **reviews** are
  confidential. The audit log captures the
  review but the content is access-
  controlled.
- The HR domain's **loans** are
  integration with finance. The engine
  emits `StaffLoanCreated`; finance
  schedules the deductions.
- The HR domain's **bulk operations** are
  common (e.g. bulk import of 50 staff at
  the start of the year). The engine's
  bulk command is all-or-nothing.
