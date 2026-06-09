# Assessment Domain — Entities

Entities have identity and lifecycle but are not aggregate roots.
They are loaded and persisted only through their aggregate root.

## ExamScheduleSubject

**Identity:** `ExamScheduleSubjectId(SchoolId, Uuid)`
**Owner:** `ExamSchedule`

A per-subject entry inside an `ExamSchedule`. Carries `Date`,
`StartTime`, `EndTime`, `Room`, `FullMark`, `PassMark`, and the
`SubjectId` and `ExamScheduleId` foreign keys.

### Invariants

1. Belongs to exactly one `ExamSchedule`.
2. `FullMark > 0`, `PassMark <= FullMark`.

## ExamSetup

**Identity:** `ExamSetupId(SchoolId, Uuid)`
**Owner:** `Exam`

A section-level configuration that augments an exam. Carries
`ExamTitle`, `ExamMark`, `SubjectId`, `SectionId`, `ExamId`,
`ExamTermId`. One per `(exam, section)`.

### Invariants

1. Belongs to exactly one `Exam`.
2. Unique by `(exam_id, section_id)` per academic year.

## ExamSetting

**Identity:** `ExamSettingId(SchoolId, Uuid)`
**Owner:** `School`

A school-wide exam publication: `Title`, `ExamType`, `PublishDate`,
`StartDate`, `EndDate`, optional `File` (file ref), `ActiveStatus`.

### Invariants

1. `StartDate <= EndDate`.
2. `PublishDate <= StartDate`.

## ExamSignature

**Identity:** `ExamSignatureId(SchoolId, Uuid)`
**Owner:** `School`

A signature image used on report cards: `Title`, `Signature` (file
ref), `ActiveStatus`. Multiple signatures may exist (principal, class
teacher, etc.).

### Invariants

1. `Title` is unique per school.

## MarkStoreEntry

**Identity:** `MarkStoreEntryId(SchoolId, Uuid)`
**Owner:** `MarkStore`

A per-subject row inside a `MarkStore`. Carries the per-subject
mark, absence flag, and computed grade. Some implementations
collapse this into the parent `MarkStore` row; the entity exists
where per-subject breakdown is stored separately.

### Invariants

1. Belongs to exactly one `MarkStore`.
2. Unique by `(mark_store_id, subject_id)`.

## ExamWisePosition

**Identity:** `ExamWisePositionId(SchoolId, Uuid)`
**Owner:** `Exam`

A per-section per-exam merit position. `Position`, `TotalMark`,
`Gpa`, `Grade`, `RollNo`, `AdmissionNo`, `RecordId`.

### Invariants

1. Unique by `(class_id, section_id, exam_id, record_id)`.

## MeritPosition

**Identity:** `MeritPositionId(SchoolId, Uuid)`
**Owner:** `Exam`

A per-section per-term merit position. `Position`, `TotalMark`,
`AdmissionNo`, `Gpa`, `Grade`, `RecordId`, `ExamTermId`.

### Invariants

1. Unique by `(class_id, section_id, exam_term_id, record_id)`.

## ResultSetting

**Identity:** `ResultSettingId(SchoolId, Uuid)`
**Owner:** `School`

Per-school result publication settings: which exam types publish,
when, with what header/footer/background, percentage rules, and
front-end visibility.

### Invariants

1. One record per school per academic year.

## QuestionAssignment

**Identity:** `QuestionAssignmentId(SchoolId, Uuid)`
**Owner:** `OnlineExam`

A link between an `OnlineExam` and a `QuestionBank` entry.

### Invariants

1. Belongs to exactly one `OnlineExam`.
2. Unique by `(online_exam_id, question_bank_id)`.

## QuestionMuOption

**Identity:** `QuestionMuOptionId(SchoolId, Uuid)`
**Owner:** `OnlineExamQuestion`

A multiple-choice or multi-select option for a question.
`Title`, `Status` (0=unchecked, 1=checked), `ActiveStatus`.

### Invariants

1. Belongs to exactly one `OnlineExamQuestion`.
2. At least one option must exist for `MultipleChoice` and
   `MultiSelect` question types.

## QuestionGroup

**Identity:** `QuestionGroupId(SchoolId, Uuid)`
**Owner:** `School`

A grouping for questions (e.g. "Algebra"). `Title`.

### Invariants

1. `Title` is unique per school.

## QuestionLevel

**Identity:** `QuestionLevelId(SchoolId, Uuid)`
**Owner:** `School`

A difficulty level. `Level` (e.g. "Easy", "Medium", "Hard").

### Invariants

1. `Level` is unique per school.

## OnlineExamMark

**Identity:** `OnlineExamMarkId(SchoolId, Uuid)`
**Owner:** `OnlineExam`

The per-student final mark on an online exam. `Marks`, `Abs`
(0/1).

### Invariants

1. Belongs to exactly one `OnlineExam`.
2. Unique by `(online_exam_id, student_id, subject_id)`.

## OnlineExamQuestionAssign

**Identity:** `OnlineExamQuestionAssignId(SchoolId, Uuid)`
**Owner:** `OnlineExam`

Alias for `QuestionAssignment`. Provided here for consistency with
the schema's `assessment_online_exam_question_assigns` table name.

## OnlineExamQuestion

**Identity:** `OnlineExamQuestionId(SchoolId, Uuid)`
**Owner:** `OnlineExam`

A per-online-exam question instance. `Type` (single char code),
`Mark`, `Title`, `TrueFalse`, `SuitableWords`.

### Invariants

1. Belongs to exactly one `OnlineExam`.
2. `Mark > 0`.

## OnlineExamStudentAnswerMarking

**Identity:** `OnlineExamStudentAnswerMarkingId(SchoolId, Uuid)`
**Owner:** `OnlineExam`

The per-question per-student answer and marking. `UserAnswer`,
`AnswerStatus` (right/wrong/partial), `ObtainMarks`, `MarkedBy` (0
for auto-mark, staff id for manual).

### Invariants

1. Belongs to exactly one `OnlineExam`.
2. Unique by `(online_exam_id, student_id, question_id)`.

## StudentTakeOnlineExam

**Identity:** `StudentTakeOnlineExamId(SchoolId, Uuid)`
**Owner:** `OnlineExam`

A student's attempt at an `OnlineExam`. `Status`, `StudentDone`,
`TotalMarks`, `RecordId`, `StudentId`.

### Invariants

1. Belongs to exactly one `OnlineExam`.
2. Unique by `(online_exam_id, student_id, record_id)`.

## SeatPlanChild

**Identity:** `SeatPlanChildId(SchoolId, Uuid)`
**Owner:** `SeatPlan`

A per-room allocation: `RoomId`, `AssignStudents`, `StartTime`,
`EndTime`.

### Invariants

1. Belongs to exactly one `SeatPlan`.
2. `AssignStudents > 0`.
3. `StartTime < EndTime`.

## AdmitCardSetting

**Identity:** `AdmitCardSettingId(SchoolId, Uuid)`
**Owner:** `School`

Per-school branding/layout for admit cards. Boolean display flags
for `StudentPhoto`, `StudentName`, `AdmissionNo`, `ClassSection`,
`ExamName`, `AcademicYear`, `PrincipalSignature`,
`ClassTeacherSignature`, `GuardianName`, `SchoolAddress`,
`StudentDownload`, `ParentDownload`, `StudentNotification`,
`ParentNotification`. File references for principal and teacher
signatures. `AdmitLayout` (1=portrait, 2=landscape),
`AdmitSubTitle`, `Description`.

### Invariants

1. One record per school per academic year.

## SeatPlanSetting

**Identity:** `SeatPlanSettingId(SchoolId, Uuid)`
**Owner:** `School`

Per-school branding/layout for the seat plan. Boolean flags for
`SchoolName`, `StudentPhoto`, `StudentName`, `AdmissionNo`,
`ClassSection`, `ExamName`, `RollNo`, `AcademicYear`.

### Invariants

1. One record per school per academic year.

## ExamRoutinePage

**Identity:** `ExamRoutinePageId(SchoolId, Uuid)`
**Owner:** `School`

Public-facing content block for the exam routine page. `Title`,
`Description`, `MainTitle`, `MainDescription`, `Image`,
`MainImage`, `ButtonText`, `ButtonUrl`, `ActiveStatus`,
`IsParent`, `ClassRoutine` and `ExamRoutine` visibility flags.

### Invariants

1. One record per school.

## FrontendExamRoutine

**Identity:** `FrontExamRoutineId(SchoolId, Uuid)`
**Owner:** `School`

A front-end publication of a specific exam routine. `Title`,
`PublishDate`, `ResultFile`.

### Invariants

1. `PublishDate` is in the past relative to the visibility check.

## FrontendResult

**Identity:** `FrontResultId(SchoolId, Uuid)`
**Owner:** `School`

A front-end publication of a result. `Title`, `PublishDate`,
`ResultFile`, `Link`.

## FrontendExamResult

**Identity:** `FrontendExamResultId(SchoolId, Uuid)`
**Owner:** `School`

Marketing block for the exam-results landing page. `Title`,
`Description`, `MainTitle`, `MainDescription`, `Image`,
`MainImage`, `ButtonText`, `ButtonUrl`, `ActiveStatus`.

### Invariants

1. One record per school.

## TemporaryMeritList

**Identity:** `TemporaryMeritListId(SchoolId, Uuid)`
**Owner:** `Exam`

A staging row produced during merit computation. `Iid`,
`StudentId`, `MeritOrder`, `StudentName`, `AdmissionNo`,
`SubjectsIdString`, `SubjectsString`, `MarksString`, `TotalMarks`,
`AverageMark`, `GpaPoint`, `Result`, `ExamId`, `ClassId`,
`SectionId`, `RollNo`.

### Invariants

1. Cleared after publication.

## AllExamWisePosition

**Identity:** `AllExamWisePositionId(SchoolId, Uuid)`
**Owner:** `Exam`

A cross-section per-exam position aggregate. `Position`,
`TotalMark`, `RollNo`, `AdmissionNo`, `Gpa`, `Grade`, `RecordId`.

### Invariants

1. Unique by `(class_id, exam_id, record_id)`.
2. Sections are merged into a single ranking.

## ExamStepSkip

**Identity:** `ExamStepSkipId(SchoolId, Uuid)`
**Owner:** `School`

A wizard-skip flag. `Name`, `CreatedBy`, `UpdatedBy`. Marks
specific steps of the exam-setup wizard as skippable for this
school.

### Invariants

1. `Name` is unique per school.

## CustomResultSetting

**Identity:** `CustomResultSettingId(SchoolId, Uuid)`
**Owner:** `School`

Per-school per-exam-type branding for the result. `ExamTypeId`,
`ExamPercentage`, `MeritListSetting`, `PrintStatus`, `ProfileImage`,
`HeaderBackground`, `BodyBackground`, `AcademicYear`,
`VerticalBoarder`.

### Invariants

1. One record per `(school_id, exam_type_id, academic_id)`.

## CustomTemporaryResult

**Identity:** `CustomTemporaryResultId(SchoolId, Uuid)`
**Owner:** `ResultStore`

A staging row produced during custom result publication. `StudentId`,
`AdmissionNo`, `FullName`, `Term1`, `Gpa1`, `Term2`, `Gpa2`,
`Term3`, `Gpa3`, `FinalResult`, `FinalGrade`.

### Invariants

1. Cleared after publication.

## ExamAttendance

**Identity:** `ExamAttendanceId(SchoolId, Uuid)`
**Owner:** `Exam`

The exam-day per-subject attendance roll. `SubjectId`, `ExamId`,
`ClassId`, `SectionId`, `ActiveStatus`.

### Invariants

1. Unique by `(exam_id, subject_id, class_id, section_id,
   academic_id)`.

## ExamAttendanceChild

**Identity:** `ExamAttendanceChildId(SchoolId, Uuid)`
**Owner:** `ExamAttendance`

The per-student mark inside an exam attendance. `AttendanceType`
(P=Present, A=Absent), `StudentRecordId`, `ClassId`, `SectionId`,
`StudentId`.

### Invariants

1. Belongs to exactly one `ExamAttendance`.
2. Unique by `(exam_attendance_id, student_id)`.

## SubjectAttendance (cross-reference)

The `SubjectAttendance` aggregate is owned by the **attendance**
domain and is documented in `docs/specs/attendance/aggregates.md`.
Assessment does not own it but consumes its summaries for reports.
