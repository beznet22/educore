# Attendance Domain — Permissions

Permissions are capability strings. They are not roles. The RBAC
domain maps capabilities to roles.

## Naming

```text
<Domain>.<Aggregate>.<Action>
```

## Capabilities

### Student Attendance

- `Attendance.Mark`
- `Attendance.Update`
- `Attendance.BulkMark`
- `Attendance.Read`
- `Attendance.Notify`
- `Attendance.Report`

### Subject Attendance

- `Attendance.Subject.Mark`
- `Attendance.Subject.Update`
- `Attendance.Subject.Read`
- `Attendance.Subject.Notify`

### Staff Attendance

- `Attendance.Staff.Mark`
- `Attendance.Staff.Update`
- `Attendance.Staff.Read`
- `Attendance.Staff.Report`

### Bulk Import

- `Attendance.Import`
- `Attendance.Import.Validate`
- `Attendance.Import.Commit`
- `Attendance.Import.Cancel`

### Exam Attendance (delegated to assessment)

The exam-day attendance is owned by the assessment domain. The
attendance domain only consumes the events; no separate
capabilities are exposed here. The assessment capabilities
`ExamAttendance.Mark` and `ExamAttendance.Update` apply.

### Reports

- `Attendance.Report.Daily`
- `Attendance.Report.Weekly`
- `Attendance.Report.Monthly`
- `Attendance.Report.ByClass`
- `Attendance.Report.ByStudent`
- `Attendance.Report.ByStaff`

## Default Role Mapping

The platform's default role catalog binds the following:

| Role            | Capabilities (highlights)                                                       |
| --------------- | -------------------------------------------------------------------------------- |
| SuperAdmin      | All                                                                              |
| SchoolAdmin     | All within the school                                                           |
| AttendanceCell  | Attendance.*, Attendance.Import.*, Attendance.Report.*, Attendance.Staff.Mark   |
| ClassTeacher    | Attendance.Mark, Attendance.Update, Attendance.BulkMark, Attendance.Read for own section |
| SubjectTeacher  | Attendance.Subject.Mark, Attendance.Subject.Update, Attendance.Subject.Read for own subject |
| Teacher         | Attendance.Mark, Attendance.Subject.Mark, Attendance.Read for own classes        |
| Staff           | Attendance.Staff.Mark (self), Attendance.Staff.Read (self)                       |
| Student         | Attendance.Read (self)                                                           |
| Parent          | Attendance.Read (linked student)                                                 |
| HR              | Attendance.Staff.Read, Attendance.Staff.Report                                   |
| Accountant      | Attendance.Read (limited to dates and counts)                                    |
| Auditor         | Attendance.*.Read, Attendance.*.Report (read-only across the school)             |

The default mapping is a starting point and is configurable per
school.

## Authorization Pattern

Capabilities are checked at the command boundary. The engine never
trusts the caller to assert their own role.

```rust
if !engine.rbac().has(actor_id, Capability::AttendanceMark).await? {
    return Err(DomainError::forbidden("missing capability"));
}
```

Some commands have a secondary ownership check: a class teacher can
mark attendance only for the class-sections they are assigned to; a
subject teacher can mark subject attendance only for the subjects
they are assigned to.

## Read vs Write

Read capabilities are explicit. The engine does not assume that
`Attendance.Read` implies `Attendance.Mark`. A consumer may grant
only read-only access to a parent or auditor.

## Tenant Isolation

Every capability check is paired with a tenant check. The actor
must be authenticated to the school that owns the target aggregate.
There is no cross-tenant capability elevation.

## Self-Service Scopes

For `Attendance.Staff.Mark` with a self-service scope, the engine
also accepts an actor whose `UserId` matches the `StaffId`. No
additional capability is required for self-marking when the actor
authenticates as that staff member.

## Scoped Authorization

A `ClassTeacher` role is granted the `Attendance.Mark` capability
with a scope restricting it to the teacher's assigned class-sections.
The engine evaluates the scope at the command boundary:

```rust
let scope = engine.rbac().scope(actor_id, Capability::AttendanceMark).await?;
if !scope.allows_section(cmd.section_id) {
    return Err(DomainError::forbidden("out of scope"));
}
```
