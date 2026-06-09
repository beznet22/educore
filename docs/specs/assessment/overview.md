# Assessment Domain Overview

## Purpose

The assessment domain owns the examination lifecycle of a school: the
catalog of exam types, the design of individual exams, the scheduling
of exam sessions, the entry and submission of marks, the computation
and publication of results, the generation of report cards, the
administration of online (digital) exams, the allocation of seats and
admit cards, and the capture of teacher evaluations and remarks.

It is the consumer of academic facts (classes, sections, subjects,
students, academic years) and produces the artefacts that govern
promotion, ranking, and reporting. It depends on the academic domain
and emits the events that finance, communication, and the public
website depend on.

## Responsibilities

- Defining exam types (e.g. mid-term, final, monthly) and their
  contribution rules (percentage, average, weighted).
- Authoring exams, exam setups, and exam schedules.
- Managing marks registers, mark stores, and result stores.
- Computing grades, GPAs, merit positions, and result statuses.
- Publishing results, generating report cards, and front-end result
  publications.
- Configuring and running online exams (questions, assignments,
  submissions, auto-evaluation).
- Generating seat plans and admit cards for exam sessions.
- Capturing teacher evaluations and teacher remarks on report cards.
- Storing and exposing per-exam class attendance summaries used by
  reports.

## Boundaries

The assessment domain does **not** own:

- Student identity, class assignment, promotion (see `specs/academic/`).
- Daily attendance capture (see `specs/attendance/`).
- Notification dispatch (see `specs/communication/`).
- Public website rendering (see `specs/cms/`).
- Fee computation based on exam results (see `specs/finance/`).

The assessment domain **does** provide identifier types and value
objects that other domains depend on: `ExamId`, `ExamTypeId`,
`ExamScheduleId`, `MarksRegisterId`, `ResultStoreId`, `OnlineExamId`,
`QuestionBankId`, `SeatPlanId`, `AdmitCardId`, `TeacherEvaluationId`.

## Dependencies

- `smsengine-core` — error types, identifier trait, validation.
- `smsengine-platform` — `SchoolId`, `UserId`, `TenantContext`.
- `smsengine-rbac` — capability checks.
- `smsengine-events` — domain event publishing.
- `smsengine-academic` — `StudentId`, `ClassId`, `SectionId`, `SubjectId`,
  `AcademicYearId`, `StaffId`, `ClassRoomId`, `ClassTimeId`.

## Domain Invariants

1. Every assessment aggregate is anchored to exactly one `SchoolId`
   and one `AcademicYearId`.
2. An `Exam` is uniquely identified within a school by
   `(exam_type_id, class_id, section_id, subject_id)` per academic year.
3. An `ExamSchedule` is uniquely identified by
   `(exam_id, class_id, section_id)` per academic year.
4. A `MarksRegister` exists once per `(exam_id, student_id)`.
5. `MarksRegisterChild` rows are unique per `(marks_register_id,
   subject_id)`.
6. A `MarkStore` is unique per
   `(exam_setup_id, exam_type_id, student_id, student_record_id,
   subject_id)`.
7. A `ResultStore` is unique per
   `(exam_setup_id, exam_type_id, student_id, student_record_id,
   subject_id)`.
8. Marks entered must satisfy `0 <= obtained_marks <= full_mark`,
   excluding absence cases which are recorded as `is_absent=true`.
9. Grades are derived from the school's `MarksGrade` scale; a school
   must define a non-empty grade scale before publishing results.
10. An `OnlineExam` is open for taking only when `Status=Published`
    and within `[start_date_time, end_date_time]`.
11. A `SeatPlan` is generated per `(exam_type_id, student_record_id)`
    and is unique per academic year.
12. An `AdmitCard` is generated per `(exam_type_id, student_record_id)`
    and is unique per academic year.
13. A `TeacherRemark` is unique per `(student_id, exam_type_id,
    academic_id)`.
14. Exam-day attendance (`ExamAttendance`) is recorded per
    `(exam_id, subject_id, class_id, section_id)` and is consumed by
    marks registers and reports.
15. Result publication is a one-way operation: once a result is
    published it cannot be unpublished, only superseded by a
    re-publication that emits a new event.

## Aggregate Roots

| Aggregate              | Root Type            | Purpose                                    |
| ---------------------- | -------------------- | ------------------------------------------ |
| ExamType               | `ExamType`           | Catalog of exam categories (mid-term, etc) |
| Exam                   | `Exam`               | A specific exam instance per class/subject |
| ExamSetup              | `ExamSetup`          | Per-section per-subject exam config        |
| ExamSchedule           | `ExamSchedule`       | Calendar/slot for an exam                  |
| ExamScheduleSubject    | `ExamScheduleSubject`| Per-subject slot in a schedule             |
| MarksRegister          | `MarksRegister`      | Per-student marks container for an exam    |
| MarksRegisterChild     | `MarksRegisterChild` | Per-subject marks row                      |
| MarkStore              | `MarkStore`          | Consolidated marks per student/subject     |
| ResultStore            | `ResultStore`        | Per-student/subject stored result          |
| TemporaryMeritList     | `TemporaryMeritList` | Staging table for merit computation        |
| MeritPosition          | `MeritPosition`      | Final computed merit position              |
| ExamWisePosition       | `ExamWisePosition`   | Per-section exam position                  |
| AllExamWisePosition    | `AllExamWisePosition`| Cross-section exam position aggregate      |
| CustomResultSetting    | `CustomResultSetting`| Custom branding/print settings for result  |
| CustomTemporaryResult  | `CustomTemporaryResult`| Staging for custom result publication   |
| ResultSetting          | `ResultSetting`      | Per-school result publication settings     |
| MarksGrade             | `MarksGrade`         | Grade boundary scale (A+, A, B, ...)       |
| ExamSetting            | `ExamSetting`        | School-wide exam publication settings      |
| ExamSignature          | `ExamSignature`      | Signatures shown on report cards           |
| ExamRoutinePage        | `ExamRoutinePage`    | Public-facing routine page content         |
| FrontendExamRoutine       | `FrontendExamRoutine`   | Front-end published exam routine           |
| FrontendResult            | `FrontendResult`        | Front-end published result                 |
| FrontendExamResult     | `FrontendExamResult` | Marketing block for exam results           |
| OnlineExam             | `OnlineExam`         | A digital exam instance                    |
| QuestionBank           | `QuestionBank`       | Reusable question pool                     |
| QuestionGroup          | `QuestionGroup`      | Question grouping (e.g. "Algebra")         |
| QuestionLevel          | `QuestionLevel`      | Difficulty level for questions             |
| QuestionAssignment     | `QuestionAssignment` | OnlineExam ↔ QuestionBank link             |
| OnlineExamQuestion     | `OnlineExamQuestion` | Per-online-exam question with options      |
| OnlineExamMark         | `OnlineExamMark`     | Per-student online exam mark               |
| OnlineExamStudentAnswer| `OnlineExamStudentAnswerMarking` | Student answer + marking    |
| StudentTakeOnlineExam  | `StudentTakeOnlineExam` | A student's attempt at an online exam  |
| SeatPlan               | `SeatPlan`           | Per-section seat allocation                |
| SeatPlanChild          | `SeatPlanChild`      | Per-room allocation within a seat plan     |
| AdmitCard              | `AdmitCard`          | Admit card per student per exam type       |
| AdmitCardSetting       | `AdmitCardSetting`   | Admit card layout/branding                 |
| SeatPlanSetting        | `SeatPlanSetting`    | Seat plan layout/branding                  |
| TeacherEvaluation      | `TeacherEvaluation`  | Student rating of a teacher                |
| TeacherRemark          | `TeacherRemark`      | Teacher's narrative remark for a student   |
| ExamStepSkip           | `ExamStepSkip`       | Wizard-skip flags for exam setup           |
| ExamAttendance         | `ExamAttendance`     | Exam-day per-subject attendance roll       |
| ExamAttendanceChild    | `ExamAttendanceChild`| Per-student mark in exam attendance        |

Each aggregate is documented in detail under
`docs/specs/assessment/aggregates.md`.

## Cross-Domain Impact

When an exam is scheduled, the assessment domain emits
`ExamScheduled`. The following domains may subscribe:

- `communication` — broadcast a routine notification to guardians.
- `cms` — refresh the public exam-routine page.
- `events` — populate the school calendar with exam dates.

When marks are submitted, `MarksSubmitted` is emitted. Subscribers:

- `assessment` (self) — recompute the result store and merit list.
- `finance` — adjust fine/exemption rules where they depend on marks.

When a result is published, `ResultPublished` is emitted:

- `communication` — send report card to guardians.
- `finance` — close any marks-linked obligations.
- `cms` — update the public results listing.
- `academic` — enable promotion decisions for the class.

When an admit card is generated, `AdmitCardGenerated` is emitted:

- `communication` — notify the student and guardian that the card is
  available.
- `mobile` — surface the card in the student/parent app.

When exam attendance is captured, `ExamAttendanceMarked` is emitted:

- `assessment` — used during result computation to flag absent
  students.
- `attendance` — feeds the per-exam attendance summary for reports.

## Consumers

- Web admin UI (school admins, exam cell).
- Teacher app (enter marks, schedule, evaluate).
- Student app (view report card, take online exam).
- Parent app (view report card, download admit card).
- AI agent (create exam, schedule, enter marks, publish).

## Anti-Goals

- The assessment domain does not present data to humans. It exposes
  commands, events, and queries.
- The assessment domain does not generate PDF/HTML. Rendering is a
  port-driven adapter; the domain emits structured payload events.
- The assessment domain does not deliver SMS or push notifications.
  Delivery is a port; the domain publishes events.
- The assessment domain does not own academic-year promotion
  decisions. It publishes `ResultPublished`; promotion is the
  academic domain's concern.
