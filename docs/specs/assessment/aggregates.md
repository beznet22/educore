# Assessment Domain — Aggregates

## ExamType

**Root type:** `ExamType`
**Identity:** `ExamTypeId(SchoolId, Uuid)`
**Tenant:** `SchoolId`

### Purpose

A category of exam offered by the school (e.g. "Mid-Term", "Final",
"Monthly Test"). Defines whether results of this type are averaged
across multiple instances, what percentage they contribute, and the
average mark used for normalization.

### Owned Children

- `Exam` — exams created under this type.

### Invariants

1. A title is unique within a school.
2. `Percentage` is in `[0, 100]`.
3. `IsAverage` marks the type as "averaged across instances" (e.g. the
   best of two monthly tests contributes 100%).
4. `AverageMark` is non-negative and is the cap used when computing
   averages across instances.
5. The parent/child relationship (`parent_id`) allows composite exam
   types (a final term is composed of mid-terms).

### Commands

- `CreateExamType`
- `UpdateExamType`
- `DeleteExamType`

### Events

- `ExamTypeCreated`
- `ExamTypeUpdated`
- `ExamTypeDeleted`

---

## Exam

**Root type:** `Exam`
**Identity:** `ExamId(SchoolId, Uuid)`

### Purpose

A specific exam instance: one `(class, section, subject)` under one
`ExamType` for an academic year. Carries `ExamMark` and `PassMark`.

### Owned Children

- `ExamSetup` (one per section).
- `ExamSchedule` (one per section).
- `MarksRegister` (one per student).
- `SeatPlan` (one per section).

### Invariants

1. Unique by `(exam_type_id, class_id, section_id, subject_id,
   academic_year_id)` within a school.
2. `PassMark <= ExamMark` and both are non-negative.
3. An `Exam` cannot be deleted while `MarksRegister` rows reference
   it.

### Commands

- `CreateExam`
- `UpdateExam`
- `DeleteExam`

### Events

- `ExamCreated`
- `ExamUpdated`
- `ExamDeleted`

---

## ExamSetup

**Root type:** `ExamSetup`
**Identity:** `ExamSetupId(SchoolId, Uuid)`

### Purpose

The per-section configuration that augments an `Exam` for a
particular section: `ExamTitle`, `ExamMark`, `Subject`, `Section`. A
section-level setup is required to enter marks and to publish
results.

### Invariants

1. One `ExamSetup` per `(exam_id, section_id)` per academic year.
2. `ExamMark` may override the parent exam's mark.
3. Deletion is blocked if marks have been entered against it.

### Commands

- `CreateExamSetup`
- `UpdateExamSetup`
- `DeleteExamSetup`

### Events

- `ExamSetupCreated`
- `ExamSetupUpdated`
- `ExamSetupDeleted`

---

## ExamSchedule

**Root type:** `ExamSchedule`
**Identity:** `ExamScheduleId(SchoolId, Uuid)`

### Purpose

The calendar slot (date + start/end time + room + period) for an
exam. Lives at the `(exam, class, section)` level. Carries a list of
subject-level entries via `ExamScheduleSubject`.

### Owned Children

- `ExamScheduleSubject` — one row per subject in the schedule.

### Invariants

1. Unique by `(exam_id, class_id, section_id)` per academic year.
2. `StartTime < EndTime`.
3. A teacher cannot be assigned to two overlapping schedules.
4. A room cannot be assigned to two overlapping schedules.
5. Date is within the academic year.

### Commands

- `ScheduleExam`
- `UpdateExamSchedule`
- `CancelExamSchedule`

### Events

- `ExamScheduled`
- `ExamScheduleUpdated`
- `ExamScheduleCancelled`

---

## MarksRegister

**Root type:** `MarksRegister`
**Identity:** `MarksRegisterId(SchoolId, Uuid)`

### Purpose

A per-student container that holds the marks obtained for an
exam. There is one `MarksRegister` per `(exam_id, student_id)`. Its
children (`MarksRegisterChild`) hold the per-subject marks.

### Owned Children

- `MarksRegisterChild` — one per subject in the exam.

### Invariants

1. Unique by `(exam_id, student_id)` per academic year.
2. A register may be `Active` or `Cancelled`.
3. When the parent register is cancelled, all child rows are
   cancelled in the same transaction.

### Commands

- `InitializeMarksRegister`
- `CancelMarksRegister`

### Events

- `MarksRegisterCreated`
- `MarksRegisterCancelled`

---

## MarkStore

**Root type:** `MarkStore`
**Identity:** `MarkStoreId(SchoolId, Uuid)`

### Purpose

A consolidated marks row produced by the result-computation domain
service. Distinct from `MarksRegister` in that it represents the
**stored** result after combining input from `MarksRegister` and
custom result settings, including teacher remarks.

### Invariants

1. Unique by `(exam_setup_id, exam_type_id, student_record_id,
   subject_id)`.
2. `IsAbsent` mirrors the input mark register's absence flag.
3. `TotalMarks` is the sum across the exam, recorded per subject.
4. `TeacherRemarks` is free text bounded to 2000 chars.

### Commands

- `SaveMarkStore` (typically produced by the result service)
- `UpdateTeacherRemark`
- `DeleteMarkStore`

### Events

- `MarkStoreCreated`
- `TeacherRemarkUpdated`
- `MarkStoreDeleted`

---

## ResultStore

**Root type:** `ResultStore`
**Identity:** `ResultStoreId(SchoolId, Uuid)`

### Purpose

The published, per-student per-subject result row, including GPA
point, grade, total marks, and teacher remarks. Drives report cards
and merit position calculations.

### Invariants

1. Unique by `(exam_setup_id, exam_type_id, student_record_id,
   subject_id)`.
2. `TotalGpaPoint` and `TotalGpaGrade` are derived (they may be
   cached for read performance).
3. A result is `Published` only after `ResultStore.Publish` is
   called; pre-publication rows are drafts.
4. Publishing freezes per-subject marks; subsequent updates require
   a `RepublishResult` command that emits a new event.

### Commands

- `SaveResultStore`
- `UpdateResultRemarks`
- `PublishResult`
- `RepublishResult`

### Events

- `ResultStoreCreated`
- `ResultRemarksUpdated`
- `ResultPublished`
- `ResultRepublished`

---

## MarksGrade

**Root type:** `MarksGrade`
**Identity:** `MarksGradeId(SchoolId, Uuid)`

A grade boundary: `GradeName`, `Gpa`, `From`, `Up`, `PercentFrom`,
`PercentUpTo`, `Description`. Defines the school's grade scale used
to compute GPA and grade from a percentage.

### Invariants

1. `From < Up` for every grade.
2. `PercentFrom < PercentUpTo`.
3. The school's grade scale must be non-overlapping in percentage
   range and contiguous (no gaps).
4. Exactly one `MarksGrade` per school may be `Gpa = 0.0` (the
   "Fail" boundary).

### Commands

- `CreateMarksGrade`
- `UpdateMarksGrade`
- `ReorderMarksGrade`
- `DeleteMarksGrade`

### Events

- `MarksGradeCreated`
- `MarksGradeUpdated`
- `MarksGradeDeleted`

---

## ExamSetting

**Root type:** `ExamSetting`
**Identity:** `ExamSettingId(SchoolId, Uuid)`

### Purpose

A school-wide publication of an exam: `Title`, `ExamType`, `PublishDate`,
`StartDate`, `EndDate`, optional `File`. Used for front-end exam
announcements and as a logical "exam window" marker.

### Invariants

1. `StartDate <= EndDate`.
2. `PublishDate <= StartDate`.
3. One per school per exam type per academic year, by default.

### Commands

- `CreateExamSetting`
- `UpdateExamSetting`
- `DeleteExamSetting`

### Events

- `ExamSettingCreated`
- `ExamSettingUpdated`
- `ExamSettingDeleted`

---

## ExamSignature

**Root type:** `ExamSignature`
**Identity:** `ExamSignatureId(SchoolId, Uuid)`

A signature that may appear on report cards and admit cards:
`Title`, `Signature` (file reference), `ActiveStatus`.

### Invariants

1. Title is unique per school.
2. A signature is inactive when removed; existing reports still
   reference the original file.

### Commands

- `SetExamSignature`
- `UpdateExamSignature`
- `DeleteExamSignature`

### Events

- `ExamSignatureCreated`
- `ExamSignatureUpdated`
- `ExamSignatureDeleted`

---

## OnlineExam

**Root type:** `OnlineExam`
**Identity:** `OnlineExamId(SchoolId, Uuid)`

### Purpose

A digital exam for a `(class, section, subject)` in an academic year.
Carries lifecycle flags: `Status` (Pending/Published),
`IsTaken`, `IsClosed`, `IsWaiting`, `IsRunning`, `AutoMark`, and
time window `StartTime`/`EndTime`/`EndDateTime`.

### Owned Children

- `QuestionAssignment` — links to `QuestionBank` rows.
- `OnlineExamQuestion` — exam-specific question instances.
- `OnlineExamMark` — per-student final mark.
- `StudentTakeOnlineExam` — per-student attempt record.

### Invariants

1. Unique by `(class_id, section_id, subject_id, academic_id)`
   when the status is `Published`.
2. `StartTime < EndTime` and `EndTime <= EndDateTime`.
3. Lifecycle transitions: `Pending → Published → (Running →
   Closed)`. `IsWaiting` is a transient state set by the start
   service.
4. Once `IsClosed=true`, no more answers are accepted.
5. `AutoMark=true` triggers automatic evaluation on close.

### Commands

- `CreateOnlineExam`
- `UpdateOnlineExam`
- `PublishOnlineExam`
- `StartOnlineExam`
- `CloseOnlineExam`
- `DeleteOnlineExam`

### Events

- `OnlineExamCreated`
- `OnlineExamUpdated`
- `OnlineExamPublished`
- `OnlineExamStarted`
- `OnlineExamClosed`
- `OnlineExamDeleted`

---

## QuestionBank

**Root type:** `QuestionBank`
**Identity:** `QuestionBankId(SchoolId, Uuid)`

### Purpose

A reusable pool of questions (e.g. "Algebra Question Set 2026").
A question has a `Type` (Multiple Choice, True/False, Short Answer,
Fill-in-Blank, Multi-Select), `Mark`, `Title`, optional `TrueFalse`
flag, and `SuitableWords` for fill-in-the-blank.

### Invariants

1. `Mark > 0`.
2. `Type` is one of the supported `QuestionType` variants.
3. The bank is uniquely titled within a school.

### Commands

- `CreateQuestion`
- `UpdateQuestion`
- `DeleteQuestion`

### Events

- `QuestionCreated`
- `QuestionUpdated`
- `QuestionDeleted`

---

## StudentTakeOnlineExam

**Root type:** `StudentTakeOnlineExam`
**Identity:** `StudentTakeOnlineExamId(SchoolId, Uuid)`

A student's attempt at an `OnlineExam`: `Status` (Not Yet,
Submitted, Got Marks), `StudentDone`, `TotalMarks`. Children include
`StudentTakeOnlineExamQuestion` (per-question response).

### Invariants

1. Unique by `(online_exam_id, student_id, record_id)`.
2. `Status` transitions: `NotYet → Submitted → GotMarks`.
3. Once `Status=GotMarks`, no further answers are accepted.

### Commands

- `StartOnlineExam` (creates the attempt)
- `SubmitOnlineExamAnswer`
- `EvaluateOnlineExam`

### Events

- `OnlineExamStarted` (per student)
- `OnlineExamAnswered`
- `OnlineExamEvaluated`

---

## SeatPlan

**Root type:** `SeatPlan`
**Identity:** `SeatPlanId(SchoolId, Uuid)`

### Purpose

The seat allocation for one section for one exam type in an
academic year. Has children `SeatPlanChild` for per-room
allocations (room, students count, start/end time).

### Owned Children

- `SeatPlanChild` — one per room.

### Invariants

1. Unique by `(exam_type_id, class_id, section_id, academic_id)`.
2. A student receives at most one seat plan per academic year per
   exam type.
3. `SeatPlanChild` room allocations must not overlap in time.

### Commands

- `GenerateSeatPlan`
- `UpdateSeatPlan`
- `CancelSeatPlan`

### Events

- `SeatPlanGenerated`
- `SeatPlanUpdated`
- `SeatPlanCancelled`

---

## AdmitCard

**Root type:** `AdmitCard`
**Identity:** `AdmitCardId(SchoolId, Uuid)`

### Purpose

The admit card issued to a student for an exam type in an academic
year. A card is generated per `(student_record_id, exam_type_id)`
and references the school's `AdmitCardSetting` at generation time.

### Invariants

1. Unique by `(student_record_id, exam_type_id, academic_id)`.
2. A card is generated only when the exam is scheduled and seat
   plan exists for the section.
3. Once generated, the card is immutable; a re-generation
   supersedes the previous card with a new id and emits a new
   event.

### Commands

- `GenerateAdmitCard`
- `RegenerateAdmitCard`
- `CancelAdmitCard`

### Events

- `AdmitCardGenerated`
- `AdmitCardRegenerated`
- `AdmitCardCancelled`

---

## TeacherEvaluation

**Root type:** `TeacherEvaluation`
**Identity:** `TeacherEvaluationId(SchoolId, Uuid)`

### Purpose

A student rating of a teacher for a subject in a record, with
`Rating`, `Comment`, `Status` (0=pending, 1=approved), `RoleId`,
`ParentId` (used for threaded evaluation), `AcademicId`.

### Invariants

1. Unique by `(student_id, teacher_id, subject_id, record_id,
   academic_id)` per school.
2. `Status` transitions: `0 → 1`. Rejection sets the row inactive.
3. A student can rate a teacher only for subjects they are
   enrolled in.

### Commands

- `MarkTeacherEvaluation`
- `ApproveTeacherEvaluation`
- `RejectTeacherEvaluation`

### Events

- `TeacherEvaluationCompleted`
- `TeacherEvaluationApproved`
- `TeacherEvaluationRejected`

---

## TeacherRemark

**Root type:** `TeacherRemark`
**Identity:** `TeacherRemarkId(SchoolId, Uuid)`

A teacher's narrative remark for a student for an exam type in an
academic year. Free text `Remark`, the `TeacherId`, `StudentId`,
`ExamTypeId`, `AcademicId`.

### Invariants

1. Unique by `(student_id, exam_type_id, academic_id)` per school.
2. `Remark` length is bounded to 2000 chars.

### Commands

- `AddTeacherRemark`
- `UpdateTeacherRemark`
- `DeleteTeacherRemark`

### Events

- `TeacherRemarkAdded`
- `TeacherRemarkUpdated`
- `TeacherRemarkDeleted`

---
