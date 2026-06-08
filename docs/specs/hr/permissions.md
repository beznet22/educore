# HR Domain — Permissions

Permissions are capability strings. They are not roles. The RBAC
domain maps capabilities to roles.

## Naming

```text
<Domain>.<Aggregate>.<Action>
```

The HR domain uses `Staff.*`, `Department.*`, `Designation.*`,
`Leave.*`, `Attendance.Staff.*`, `Payroll.*`, `SalaryTemplate.*`,
`HourlyRate.*`, `StaffRegistrationField.*`, `BulkImport.*`,
`Report.*`.

## Capabilities

### Staff

- `Staff.Register`
- `Staff.Update`
- `Staff.Read`
- `Staff.Suspend`
- `Staff.Reinstate`
- `Staff.Resign`
- `Staff.Terminate`
- `Staff.Retire`
- `Staff.Delete`
- `Staff.ChangeDepartment`
- `Staff.ChangeDesignation`
- `Staff.ChangeRole`
- `Staff.AssignSubjectTeacher`
- `Staff.AssignClassTeacher.Create`
- `Staff.AssignClassTeacher.Update`
- `Staff.AssignClassTeacher.Delete`
- `Staff.ImportBulk`
- `Staff.ImportBulk.Promote`
- `Staff.ImportBulk.Reject`
- `Staff.Document.Upload`
- `Staff.Document.Download`

### Department

- `Department.Create`
- `Department.Update`
- `Department.Delete`
- `Department.Read`

### Designation

- `Designation.Create`
- `Designation.Update`
- `Designation.Delete`
- `Designation.Read`

### Leave

- `LeaveType.Create`
- `LeaveType.Update`
- `LeaveType.Delete`
- `LeaveType.Read`
- `LeaveDefine.Create`
- `LeaveDefine.Update`
- `LeaveDefine.Delete`
- `LeaveDefine.Read`
- `Leave.Request`
- `Leave.Approve`
- `Leave.Reject`
- `Leave.Cancel`
- `Leave.Read`

### Attendance

- `Attendance.Staff.Mark`
- `Attendance.Staff.Update`
- `Attendance.Staff.Delete`
- `Attendance.Staff.Read`
- `Attendance.Staff.Import`
- `Attendance.Staff.Import.Promote`
- `Attendance.Staff.Import.Reject`

### Payroll

- `Payroll.Generate`
- `Payroll.Update`
- `Payroll.Approve`
- `Payroll.MarkPaid`
- `Payroll.Read`
- `Payroll.Earning.Add`
- `Payroll.Earning.Update`
- `Payroll.Earning.Delete`
- `Payroll.Deduction.Add`
- `Payroll.Deduction.Update`
- `Payroll.Deduction.Delete`
- `Payroll.LeaveDeduction.Add`
- `Payroll.LeaveDeduction.Update`
- `Payroll.LeaveDeduction.Delete`
- `PayrollPayment.Read`

### Salary & Rate

- `SalaryTemplate.Create`
- `SalaryTemplate.Update`
- `SalaryTemplate.Delete`
- `SalaryTemplate.Read`
- `HourlyRate.Set`
- `HourlyRate.Update`
- `HourlyRate.Delete`
- `HourlyRate.Read`

### Registration Field

- `StaffRegistrationField.Create`
- `StaffRegistrationField.Update`
- `StaffRegistrationField.Delete`
- `StaffRegistrationField.Read`

### Reports

- `Report.StaffRoster`
- `Report.StaffByDepartment`
- `Report.StaffByDesignation`
- `Report.LeaveUsage`
- `Report.LeaveBalance`
- `Report.AttendanceDaily`
- `Report.AttendanceMonthly`
- `Report.AttendanceByStaff`
- `Report.PayrollRegister`
- `Report.PayrollByStaff`
- `Report.PayrollByDepartment`
- `Report.PayrollTax`
- `Report.SalaryStructure`
- `Report.HourlyEarnings`
- `Report.LeaveDeduction`
- `Report.HR.Read` (umbrella)

## Default Role Mapping

The platform's default role catalog binds the following:

| Role          | Capabilities (highlights)                                                |
| ------------- | ------------------------------------------------------------------------ |
| SuperAdmin    | All                                                                      |
| SchoolAdmin   | All within the school                                                    |
| HR            | Staff.*, Department.*, Designation.*, Leave.*, Attendance.Staff.*, Payroll.*, SalaryTemplate.*, HourlyRate.*, StaffRegistrationField.*, Report.HR.* |
| Accountant    | Payroll.Read, PayrollPayment.Read                                       |
| Teacher       | Staff.Read, Leave.Request, Leave.Read, Attendance.Staff.Read, Payroll.Read |
| Driver        | Staff.Read (self), Leave.Request, Attendance.Staff.Read                 |
| Parent        | Staff.Read (linked), Leave.Read (linked)                                 |

The default mapping is a starting point and is configurable per
school.

## Authorization Pattern

Capabilities are checked at the command boundary. The engine never
trusts the caller to assert their own role.

```rust
if !engine.rbac().has(actor_id, Capability::StaffRegister).await? {
    return Err(DomainError::forbidden("missing capability"));
}
```

Some commands have a secondary ownership check: a staff member
applying for leave may only apply for their own leave; the staff
member who approved a leave cannot be the same person who recorded
it; the staff member who generated a payroll cannot be the same
person who approved it (segregation of duties).

## Read vs Write

Read capabilities are explicit. `Staff.Read` does not imply
`Staff.Update`. A consumer may grant only read-only access to a
department head or auditor.

## Tenant Isolation

Every capability check is paired with a tenant check. The actor must
be authenticated to the school that owns the target aggregate.
There is no cross-tenant capability elevation.
