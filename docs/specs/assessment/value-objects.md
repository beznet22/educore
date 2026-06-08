# Assessment Domain — Value Objects

Value objects are immutable, validated at construction, and have no
identity. They are compared by value.

## Identifiers

All identifiers are typed and tenant-scoped. The generic `Id<S, T>`
wrapper carries the `SchoolId` of the owning school and a `Uuid`.

| Identifier                              | Backing Type                | Notes                              |
| --------------------------------------- | --------------------------- | ---------------------------------- |
| `ExamTypeId`                            | `Id<ExamType>`              | Exam category                      |
| `ExamId`                                | `Id<Exam>`                  | Per-(class,section,subject) exam   |
| `ExamSetupId`                           | `Id<ExamSetup>`             | Section-level exam config          |
| `ExamScheduleId`                        | `Id<ExamSchedule>`          | Calendar slot for an exam          |
| `ExamScheduleSubjectId`                 | `Id<ExamScheduleSubject>`   | Per-subject schedule entry         |
| `ExamSettingId`                         | `Id<ExamSetting>`           | School exam publication            |
| `ExamSignatureId`                       | `Id<ExamSignature>`         | Signature image                    |
| `MarksRegisterId`                       | `Id<MarksRegister>`         | Per-student exam marks             |
| `MarksRegisterChildId`                  | `Id<MarksRegisterChild>`    | Per-subject marks row              |
| `MarkStoreId`                           | `Id<MarkStore>`             | Consolidated stored marks          |
| `MarkStoreEntryId`                      | `Id<MarkStoreEntry>`        | Per-subject stored mark            |
| `ResultStoreId`                         | `Id<ResultStore>`           | Per-subject published result       |
| `ResultSettingId`                       | `Id<ResultSetting>`         | Per-school result settings         |
| `MarksGradeId`                          | `Id<MarksGrade>`            | Grade boundary                     |
| `TemporaryMeritListId`                  | `Id<TemporaryMeritList>`    | Merit staging row                  |
| `MeritPositionId`                      | `Id<MeritPosition>`         | Final merit position               |
| `ExamWisePositionId`                    | `Id<ExamWisePosition>`      | Per-section exam position          |
| `AllExamWisePositionId`                 | `Id<AllExamWisePosition>`   | Cross-section exam position        |
| `CustomResultSettingId`                 | `Id<CustomResultSetting>`   | Custom result branding             |
| `CustomTemporaryResultId`               | `Id<CustomTemporaryResult>` | Custom result staging              |
| `ExamStepSkipId`                        | `Id<ExamStepSkip>`          | Wizard-skip flag                   |
| `ExamRoutinePageId`                     | `Id<ExamRoutinePage>`       | Public routine page content        |
| `FrontExamRoutineId`                    | `Id<FrontExamRoutine>`      | Front-end exam routine             |
| `FrontResultId`                         | `Id<FrontResult>`           | Front-end result publication       |
| `FrontendExamResultId`                  | `Id<FrontendExamResult>`    | Marketing block for results        |
| `OnlineExamId`                          | `Id<OnlineExam>`            | Digital exam                       |
| `QuestionBankId`                        | `Id<QuestionBank>`          | Reusable question                  |
| `QuestionGroupId`                       | `Id<QuestionGroup>`         | Question grouping                  |
| `QuestionLevelId`                       | `Id<QuestionLevel>`         | Difficulty level                   |
| `QuestionAssignmentId`                  | `Id<QuestionAssignment>`    | OnlineExam ↔ QuestionBank          |
| `QuestionMuOptionId`                    | `Id<QuestionMuOption>`      | MC option                          |
| `OnlineExamQuestionId`                  | `Id<OnlineExamQuestion>`    | Per-exam question                  |
| `OnlineExamQuestionAssignId`            | `Id<OnlineExamQuestionAssign>` | Alias of `QuestionAssignment`    |
| `OnlineExamMarkId`                      | `Id<OnlineExamMark>`        | Per-student online exam mark       |
| `OnlineExamStudentAnswerMarkingId`      | `Id<OnlineExamStudentAnswerMarking>` | Per-question answer        |
| `StudentTakeOnlineExamId`               | `Id<StudentTakeOnlineExam>` | Per-student attempt                |
| `SeatPlanId`                            | `Id<SeatPlan>`              | Section seat allocation            |
| `SeatPlanChildId`                       | `Id<SeatPlanChild>`         | Per-room allocation                |
| `SeatPlanSettingId`                     | `Id<SeatPlanSetting>`       | Seat plan branding                 |
| `AdmitCardId`                           | `Id<AdmitCard>`             | Admit card                         |
| `AdmitCardSettingId`                    | `Id<AdmitCardSetting>`      | Admit card branding                |
| `TeacherEvaluationId`                   | `Id<TeacherEvaluation>`     | Teacher rating                     |
| `TeacherRemarkId`                       | `Id<TeacherRemark>`         | Teacher narrative remark           |
| `ExamAttendanceId`                      | `Id<ExamAttendance>`        | Exam-day attendance roll           |
| `ExamAttendanceChildId`                 | `Id<ExamAttendanceChild>`   | Per-student exam attendance        |

## Exam Type Enums

| Type                  | Values                                                                          |
| --------------------- | ------------------------------------------------------------------------------- |
| `ExamTerm`            | `MidTerm`, `Final`, `Monthly`, `Weekly`, `UnitTest`, `Mock`, `Custom`           |
| `QuestionType`        | `MultipleChoice`, `TrueFalse`, `ShortAnswer`, `FillBlank`, `MultiSelect`        |
| `OnlineExamStatus`    | `Pending`, `Published`, `Waiting`, `Running`, `Closed`                          |
| `AttemptStatus`       | `NotYet`, `Submitted`, `GotMarks`                                               |
| `AnswerStatus`        | `Right`, `Wrong`, `Partial`, `Unattempted`                                      |
| `ResultStatus`        | `Pass`, `Fail`, `Manual`, `Withheld`                                            |
| `AttendanceType`      | `Present`, `Absent`, `Late`, `Holiday`, `HalfDay`                               |

## Names and Codes

| Type                | Constraints                                                      |
| ------------------- | ---------------------------------------------------------------- |
| `ExamName`          | 1..200 chars, unique within `(school, exam_type, year)`          |
| `ExamCode`          | 1..50 chars, unique within school                                |
| `ExamTitle`         | 1..200 chars, used in the report card header                     |
| `QuestionTitle`     | 1..2000 chars, the question text                                 |
| `QuestionOption`    | 1..500 chars                                                     |
| `Remark`            | 1..2000 chars                                                    |
| `Comment`           | 1..2000 chars                                                    |
| `SignatureTitle`    | 1..100 chars                                                     |
| `GroupTitle`        | 1..200 chars                                                     |
| `Level`             | 1..100 chars                                                     |
| `RoutinePageTitle`  | 1..200 chars                                                     |

## Marks and Grades

| Type                 | Constraints                                                    |
| -------------------- | -------------------------------------------------------------- |
| `Marks`              | `f32` non-negative                                             |
| `ExamMark`           | `f32` in `(0, 1000]`                                           |
| `PassMark`           | `f32` in `[0, 100]`                                            |
| `FullMark`           | `f32` in `(0, 1000]`                                           |
| `TotalMarks`         | `f32` non-negative, sum across subjects                        |
| `AverageMark`        | `f32` non-negative, used for averaging exam types              |
| `Gpa`                | `f32` in `[0, 5]`                                              |
| `Grade`              | `String` in `{"A+", "A", "A-", "B+", "B", "C+", "C", "D", "F"}`|
| `Percentage`         | `f32` in `[0, 100]`                                            |
| `ExamPercentage`     | `f32` in `[0, 100]`, contribution of an exam type             |
| `MeritPosition`      | `u32` in `[1, +inf)`                                           |
| `Rating`             | `u8` in `[1, 5]`                                               |

## Schedule and Time

| Type                 | Constraints                                                    |
| -------------------- | -------------------------------------------------------------- |
| `ExamDate`           | `NaiveDate`                                                    |
| `ExamWindow`         | `(start: NaiveDate, end: NaiveDate)`                           |
| `StartTime`          | `Time`                                                         |
| `EndTime`            | `Time`                                                         |
| `ExamDateTime`       | `NaiveDateTime`                                                |
| `ExamPeriod`         | `(start_time, end_time, period_index)`                         |
| `PublishDate`        | `NaiveDate`                                                    |
| `TimeTableEntry`     | `(day, period, subject_id, teacher_id, room_id)`               |
| `DayOfWeek`          | ISO `Mon..Sun` (1..7)                                          |

## Boolean Flags

| Type                       | Notes                                                    |
| -------------------------- | -------------------------------------------------------- |
| `IsAverage`                | Marks an exam type as averaged across instances           |
| `IsAbsent`                 | 0=present, 1=absent                                      |
| `AutoMark`                 | 0=manual evaluation, 1=auto on close                     |
| `IsTaken`                  | 0=not yet attempted, 1=attempted                         |
| `IsClosed`                 | 0=open, 1=closed for submissions                          |
| `IsWaiting`                | 0=not waiting, 1=waiting for start                       |
| `IsRunning`                | 0=not running, 1=running                                 |
| `IsParent`                 | 0=child page, 1=parent page                              |
| `ActiveStatus`             | 0=inactive, 1=active                                     |

## Identification for Reports

| Type                       | Constraints                                              |
| -------------------------- | -------------------------------------------------------- |
| `AdmissionNumber`          | from `smscore-academic`, 1..50 chars                     |
| `RollNumber`               | from `smscore-academic`, 1..50 chars                     |
| `StudentName`              | from `smscore-academic`, 1..200 chars                    |
| `SubjectsString`           | Comma-separated subject names for printing               |
| `MarksString`              | Comma-separated marks for printing                       |
| `SubjectsIdString`         | Comma-separated subject ids                              |

## School Identity Bindings

| Type                  | Notes                                                       |
| --------------------- | ----------------------------------------------------------- |
| `SchoolId`            | From `smscore-platform`                                     |
| `TenantContext`       | `(SchoolId, UserId, ...)` from `smscore-platform`           |
| `UserId`              | From `smscore-platform`                                     |
| `CorrelationId`       | From `smscore-platform`                                     |

## Validation Rules

All value objects implement `Validate` and refuse construction when
validation fails:

```rust
pub trait Validate {
    fn validate(&self) -> Result<(), ValueError>;
}
```

Construction is the only entry point:

```rust
let exam_mark = ExamMark::new(100.0)?;
let pass = PassMark::new(40.0)?;
let grade = Grade::parse("A+")?;
```

Parsing returns `Result<T, ValueError>`. There are no setters that
bypass validation.

## Cross-Reference

- `StudentId`, `ClassId`, `SectionId`, `SubjectId`, `AcademicYearId`,
  `StaffId`, `ClassRoomId`, `ClassTimeId` — from `smscore-academic`.
