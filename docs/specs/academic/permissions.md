# Academic Domain — Permissions

Permissions are capability strings. They are not roles. The RBAC domain
maps capabilities to roles.

## Naming

```text
<Domain>.<Aggregate>.<Action>
```

## Capabilities

### Student

- `Student.Admit`
- `Student.Update`
- `Student.Read`
- `Student.AssignSection`
- `Student.Suspend`
- `Student.Reinstate`
- `Student.Withdraw`
- `Student.Transfer`
- `Student.Promote`
- `Student.Graduate`
- `Student.Document.Upload`
- `Student.Document.Download`
- `Student.Homework.Submit`
- `Student.Homework.Evaluate`

### Guardian

- `Guardian.Create`
- `Guardian.Update`
- `Guardian.Link`
- `Guardian.Unlink`
- `Guardian.MarkPrimary`

### Class

- `Class.Create`
- `Class.Update`
- `Class.Delete`
- `Class.Read`

### Section

- `Section.Create`
- `Section.Update`
- `Section.Delete`
- `Section.Read`

### ClassSection

- `ClassSection.Create`
- `ClassSection.AssignTeacher`
- `ClassSection.AssignRoom`
- `ClassSection.Delete`

### Subject

- `Subject.Create`
- `Subject.Update`
- `Subject.Delete`
- `Subject.Read`

### ClassSubject

- `ClassSubject.Assign`
- `ClassSubject.Reassign`
- `ClassSubject.Unassign`

### AcademicYear

- `AcademicYear.Create`
- `AcademicYear.Update`
- `AcademicYear.SetCurrent`
- `AcademicYear.Close`
- `AcademicYear.Copy`
- `AcademicYear.Read`

### ClassRoutine

- `ClassRoutine.Create`
- `ClassRoutine.Update`
- `ClassRoutine.Swap`
- `ClassRoutine.Delete`
- `ClassRoutine.Read`

### Homework

- `Homework.Create`
- `Homework.Update`
- `Homework.Submit`
- `Homework.Evaluate`
- `Homework.Cancel`

### Lesson

- `Lesson.Create`
- `Lesson.Update`
- `Lesson.Delete`
- `LessonTopic.Create`
- `LessonTopic.Complete`
- `LessonTopic.Delete`
- `LessonPlan.Create`
- `LessonPlan.Update`
- `LessonPlan.Complete`
- `LessonPlan.AddSubTopic`
- `LessonPlan.Delete`

### StudentCategory

- `StudentCategory.Create`
- `StudentCategory.Update`
- `StudentCategory.Delete`

### StudentGroup

- `StudentGroup.Create`
- `StudentGroup.Update`
- `StudentGroup.AddStudent`
- `StudentGroup.RemoveStudent`
- `StudentGroup.Delete`

### Registration

- `RegistrationField.Create`
- `RegistrationField.Update`
- `RegistrationField.Delete`

### Certificate

- `Certificate.Create`
- `Certificate.Update`
- `Certificate.Delete`
- `Certificate.Issue`

### IdCard

- `IdCard.Create`
- `IdCard.Update`
- `IdCard.Delete`
- `IdCard.Print`

### AdmissionQuery

- `AdmissionQuery.Create`
- `AdmissionQuery.FollowUp`
- `AdmissionQuery.Convert`
- `AdmissionQuery.Close`

## Default Role Mapping

The platform's default role catalog binds the following:

| Role        | Capabilities (highlights)                                            |
| ----------- | -------------------------------------------------------------------- |
| SuperAdmin  | All                                                                 |
| SchoolAdmin | All within the school                                               |
| Teacher     | Class.Read, Subject.Read, Homework.*, Lesson*, ClassRoutine.Read    |
| Student     | Student.Read (self), Homework.Submit                                |
| Parent      | Student.Read (linked), Homework.Read                                |
| Accountant  | Student.Read                                                        |
| Librarian   | Student.Read                                                        |
| Transport   | Student.Read                                                        |

The default mapping is a starting point and is configurable per school.

## Authorization Pattern

Capabilities are checked at the command boundary. The engine never
trusts the caller to assert their own role.

```rust
if !engine.rbac().has(actor_id, Capability::StudentAdmit).await? {
    return Err(DomainError::forbidden("missing capability"));
}
```

Some commands have a secondary ownership check: a student submitting
homework is only allowed to submit their own homework; a teacher
evaluating homework must be the assigned teacher for the class-section.

## Read vs Write

Read capabilities are explicit. The engine does not assume that
"Student.Read" implies "Student.Admit". A consumer may grant only
read-only access to a parent or auditor.

## Tenant Isolation

Every capability check is paired with a tenant check. The actor must
be authenticated to the school that owns the target aggregate. There
is no cross-tenant capability elevation. Cross-tenant operations
(e.g. `Student.Transfer`) are special-cased and require a per-tenant
grant from both schools.
