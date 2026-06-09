# Assessment Domain — Permissions

Permissions are capability strings. They are not roles. The RBAC
domain maps capabilities to roles.

## Naming

```text
<Domain>.<Aggregate>.<Action>
```

## Capabilities

### ExamType

- `ExamType.Create`
- `ExamType.Update`
- `ExamType.Delete`
- `ExamType.Read`

### Exam

- `Exam.Create`
- `Exam.Update`
- `Exam.Delete`
- `Exam.Read`
- `Exam.Schedule`
- `Exam.Configure`

### ExamSetup

- `ExamSetup.Create`
- `ExamSetup.Update`
- `ExamSetup.Delete`

### ExamSchedule

- `ExamSchedule.Read`

### MarksRegister / Marks

- `Marks.Initialize`
- `Marks.Enter`
- `Marks.Submit`
- `Marks.Read`
- `Marks.Cancel`

### MarkStore

- `MarkStore.Create`
- `MarkStore.Update`
- `MarkStore.Delete`
- `MarkStore.Read`

### ResultStore / Result

- `Result.Create`
- `Result.Update`
- `Result.Publish`
- `Result.Configure`
- `Result.Read`

### MarksGrade

- `MarksGrade.Create`
- `MarksGrade.Update`
- `MarksGrade.Delete`
- `MarksGrade.Read`

### ReportCard

- `ReportCard.Generate`
- `ReportCard.Read`
- `ReportCard.Download`

### OnlineExam

- `OnlineExam.Create`
- `OnlineExam.Update`
- `OnlineExam.Delete`
- `OnlineExam.Publish`
- `OnlineExam.Start`
- `OnlineExam.Answer`
- `OnlineExam.Evaluate`
- `OnlineExam.Close`
- `OnlineExam.Read`

### Question Bank

- `Question.Create`
- `Question.Update`
- `Question.Delete`
- `Question.Read`
- `Question.AssignToExam`
- `QuestionGroup.Create`
- `QuestionGroup.Update`
- `QuestionGroup.Delete`
- `QuestionLevel.Create`
- `QuestionLevel.Update`
- `QuestionLevel.Delete`

### SeatPlan

- `SeatPlan.Generate`
- `SeatPlan.Update`
- `SeatPlan.Cancel`
- `SeatPlan.Read`
- `SeatPlan.Configure`

### AdmitCard

- `AdmitCard.Generate`
- `AdmitCard.Read`
- `AdmitCard.Download`
- `AdmitCard.Configure`
- `AdmitCard.Notify`

### TeacherEvaluation

- `TeacherEvaluation.Mark`
- `TeacherEvaluation.Approve`
- `TeacherEvaluation.Configure`
- `TeacherEvaluation.Read`

### TeacherRemark

- `TeacherRemark.Add`
- `TeacherRemark.Update`
- `TeacherRemark.Delete`
- `TeacherRemark.Read`

### ExamAttendance

- `ExamAttendance.Mark`
- `ExamAttendance.Update`
- `ExamAttendance.Read`
- `ExamAttendance.Notify`

### Front-End Publications

- `ExamRoutine.Publish`
- `ExamRoutinePage.Update`
- `FrontendResult.Publish`
- `FrontendExamResult.Update`

### Exam Signature / Settings

- `ExamSignature.Set`
- `ExamSignature.Update`
- `ExamSignature.Delete`
- `ExamSetting.Create`
- `ExamSetting.Update`
- `ExamSetting.Delete`

## Default Role Mapping

The platform's default role catalog binds the following:

| Role            | Capabilities (highlights)                                                       |
| --------------- | -------------------------------------------------------------------------------- |
| SuperAdmin      | All                                                                              |
| SchoolAdmin     | All within the school                                                           |
| ExamCell        | Exam.*, ExamType.*, MarksGrade.*, Result.*, SeatPlan.*, AdmitCard.*, OnlineExam.* |
| Teacher         | Marks.Enter, Marks.Submit, Marks.Read, OnlineExam.Answer, OnlineExam.Evaluate, TeacherRemark.*, ExamAttendance.Mark |
| ClassTeacher    | TeacherRemark.*, ExamAttendance.Mark, Marks.Enter for own section                |
| Student         | Marks.Read (self), OnlineExam.Answer (self), ReportCard.Read (self), TeacherEvaluation.Mark |
| Parent          | Marks.Read (linked), ReportCard.Read, AdmitCard.Read, AdmitCard.Download         |
| Accountant      | Result.Read                                                                      |
| Auditor         | Result.Read, Marks.Read, ExamAttendance.Read (read-only across the school)        |

The default mapping is a starting point and is configurable per
school.

## Authorization Pattern

Capabilities are checked at the command boundary. The engine never
trusts the caller to assert their own role.

```rust
if !engine.rbac().has(actor_id, Capability::ExamPublish).await? {
    return Err(DomainError::forbidden("missing capability"));
}
```

Some commands have a secondary ownership check: a teacher entering
marks must be the assigned teacher for the class-section; a student
submitting an online exam answer must be enrolled in the exam's
class-section.

## Read vs Write

Read capabilities are explicit. The engine does not assume that
`Marks.Read` implies `Marks.Enter`. A consumer may grant only
read-only access to a parent or auditor.

## Tenant Isolation

Every capability check is paired with a tenant check. The actor
must be authenticated to the school that owns the target aggregate.
There is no cross-tenant capability elevation. Cross-tenant
operations (none in assessment today) are special-cased and require
a per-tenant grant from both schools.

## Self-Service Scopes

For `OnlineExam.Answer` and `TeacherEvaluation.Mark`, the engine
also accepts a self-service scope: a student or parent can act on
their own behalf without an explicit `OnlineExam.Answer` capability
grant, provided they authenticate as that user.
