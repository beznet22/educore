# Assessment Domain — Workflows

Workflows orchestrate commands, queries, and policies to fulfill a
business goal. They are documented as ordered, conditional steps.

## Exam Authoring Workflow

```text
1. SchoolAdmin creates one or more ExamType records (e.g. "Mid-Term",
   "Final").
2. SchoolAdmin creates an Exam for each
   (class, section, subject, exam_type) tuple in the academic year.
3. SchoolAdmin creates ExamSetup rows for the section.
4. SchoolAdmin configures ExamSetting (publish date, file, window).
5. (Optional) SchoolAdmin configures CustomResultSetting for the
   exam type.
6. SchoolAdmin configures seat plan settings and admit card settings.
```

**Pre-conditions:**
- The class-section has students enrolled.
- The school's `MarksGrade` scale is non-empty and non-overlapping.
- The academic year is current.

**Failure paths:**
- Duplicate `Exam` for the same tuple → `ValidationError::UniqueViolation`.
- Overlapping `MarksGrade` percentages →
  `ValidationError::GradeScaleOverlap`.

## Exam Scheduling Workflow

```text
1. SchoolAdmin issues ScheduleExam for the (class, section) under
   an exam.
2. For each subject in the exam, a ScheduleSubjectEntry is provided.
3. The engine validates teacher and room conflicts against existing
   schedules.
4. On success, ExamScheduled is emitted and the routine is published
   to the public routine page (subscribed by cms).
5. Communication sends a routine notification to guardians.
```

**Edge cases:**
- A teacher is already booked → `ConflictError::TeacherOverlap`.
  Operator can swap or update the schedule.
- A room is already booked → `ConflictError::RoomOverlap`.
- Schedule falls outside the academic year →
  `ValidationError::OutOfRange`.

## Marks Entry Workflow

```text
1. SchoolAdmin or exam cell initializes the marks register
   (InitializeMarksRegister). One MarksRegister per student is
   created with one MarksRegisterChild per subject.
2. Subject teachers (or exam cell) enter marks per
   (student, subject) via EnterMarks.
3. The exam cell submits the register (SubmitMarks). The register
   is locked.
4. The engine computes grades per subject and emits MarksSubmitted.
5. Result service consumes MarksSubmitted and pre-computes the
   result store draft.
6. SchoolAdmin reviews and publishes (PublishResult). On publish,
   ResultPublished is emitted and merit positions are written.
```

**Pre-conditions:**
- All students in the section are covered.
- The exam is scheduled and not yet published.

**Edge cases:**
- A student is absent → `is_absent=true`; `marks` is treated as
  zero. The student receives the school's absent GPA rule.
- Partial submission is allowed only when the school has
  configured `ExamStepSkip` for the relevant steps. Otherwise the
  submit is rejected.
- A teacher submits wrong subject marks → mark is overwritten on
  the next `EnterMarks` until the register is submitted. After
  submit, marks are immutable and a `RepublishResult` is required.

## Result Publication Workflow

```text
1. Marks have been submitted for the exam.
2. Result service produces a draft ResultStore and a
   TemporaryMeritList.
3. SchoolAdmin reviews the draft.
4. PublishResult is issued per (class, section).
5. The engine:
   a. Computes GPA per subject against MarksGrade.
   b. Sums total marks and GPA per student.
   c. Determines Pass / Fail / Manual status.
   d. Ranks students per section (MeritPosition).
   e. Ranks students across sections (AllExamWisePosition).
   f. Materializes ResultStore rows and CustomTemporaryResult if
      configured.
6. ResultPublished is emitted.
7. Subscribers (communication, finance, cms, academic) react.
```

**Edge cases:**
- Two students have identical total marks → tied rank; positions
  skip the next integer.
- A student has a `Withheld` result status → publish is blocked.
  The school must resolve the withholding (manual override or
  missing data) before re-issuing.
- Re-publication: `RepublishResult` supersedes the prior
  publication; the previous event remains in the log.

## Report Card Generation Workflow

```text
1. The student/parent requests a report card for a published
   result.
2. GenerateReportCard is issued. The engine checks that a
   ResultPublished event exists for the student and exam.
3. The engine materializes a structured ReportCardPayload
   including:
   - per-subject marks, GPA, grade
   - total marks, GPA, grade
   - merit position
   - attendance summary (per class_attendances)
   - teacher remarks
   - school signatures
4. ReportCardGenerated is emitted.
5. The consumer adapter renders PDF/HTML and delivers via the
   download notification port.
```

**Edge cases:**
- A signature image is missing → the report card is generated with
  a placeholder; a `SignatureMissing` warning is logged.
- A `TeacherRemark` is missing → the field is left blank; the
  school admin may add it before generating.

## Online Exam Lifecycle

```text
1. SchoolAdmin or exam cell creates an OnlineExam (CreateOnlineExam)
   in Pending status, attaching questions from the QuestionBank.
2. Online exam questions can be customized in-place
   (AddOnlineExamQuestion, UpdateOnlineExamQuestion,
   DeleteOnlineExamQuestion) and options can be added/updated/
   deleted (AddQuestionOption, UpdateQuestionOption,
   DeleteQuestionOption).
3. SchoolAdmin publishes the online exam (PublishOnlineExam).
4. On the exam date and within the time window, the system sets
   IsWaiting and then IsRunning (StartOnlineExam).
5. Students submit answers (SubmitOnlineExamAnswer) per question.
6. The exam cell evaluates (EvaluateOnlineExam) — auto-marked or
   manually marked per question.
7. The exam cell closes the exam (EvaluateOnlineExam with
   close_exam=true). OnlineExamClosed is emitted.
8. Report card or subject marks include the online exam result if
   configured.
```

**Edge cases:**
- A student submits after `IsClosed` → rejected with
  `ConflictError::ExamClosed`.
- A student misses the time window → `Status=NotYet` at close; the
  engine flags absent.
- A `QuestionType` is `FillBlank` → `SuitableWords` is matched
  case-insensitive; partial credit is `AnswerStatus::Partial`.
- A `QuestionType` is `MultiSelect` with no option selected → no
  marks awarded.

## Seat Plan Generation Workflow

```text
1. SchoolAdmin configures SeatPlanSetting for the academic year.
2. SchoolAdmin issues GenerateSeatPlan for a section and exam type.
3. The engine validates:
   - The exam is scheduled.
   - Allocations are non-overlapping in time.
   - The total assigned equals the section's student count.
4. The engine creates a SeatPlan and SeatPlanChild rows.
5. Students are then assigned to rooms by the engine (deterministic
   per school policy: alphabetical, or random).
6. SeatPlanGenerated is emitted.
7. Communication may notify guardians of the assigned room
   (subscribed).
```

**Edge cases:**
- The seat plan is for a holiday or off-day → engine allows
  generation but flags a warning.
- A student is transferred after seat plan generation → the seat
  plan is regenerated for that student with a new `AdmitCard`.

## Admit Card Generation Workflow

```text
1. SchoolAdmin configures AdmitCardSetting for the academic year.
2. SchoolAdmin issues GenerateAdmitCard for each (student, exam
   type) tuple.
3. The engine validates:
   - The student has an active StudentRecord for the year.
   - A SeatPlan exists for the student's section.
   - The AdmitCardSetting is configured.
4. The engine creates an AdmitCard; the consumer adapter renders
   PDF/HTML.
5. AdmitCardGenerated is emitted.
6. Communication sends a download link to the student and
   guardian.
7. Parents/students download the admit card via the
   AdmitCard.Download capability.
```

**Edge cases:**
- The admit card is regenerated after a schedule change → a new
  AdmitCard is issued; the previous one is marked cancelled.
- An admit card is requested before the exam is scheduled →
  rejected with `ValidationError::ExamNotScheduled`.

## Teacher Evaluation Workflow

```text
1. SchoolAdmin configures TeacherEvaluationSetting for the school:
   enable flag, who can submit (student, parent), submission window,
   auto-approval.
2. When enabled, students/parents may rate teachers for subjects
   they take (MarkTeacherEvaluation).
3. If auto-approval is on, the row is immediately approved.
4. If not, the exam cell or school admin reviews and approves
   (ApproveTeacherEvaluation) or rejects (RejectTeacherEvaluation).
5. Aggregated ratings feed reports.
```

**Edge cases:**
- The school has not enabled evaluation → `MarksTeacherEvaluation`
  is rejected with `ValidationError::FeatureDisabled`.
- A student tries to rate a teacher for a subject they don't take
  → `ValidationError::NotEnrolled`.
- The submission window is closed → `ValidationError::WindowClosed`.

## Teacher Remark Workflow

```text
1. The class teacher or subject teacher adds a remark
   (AddTeacherRemark) for a (student, exam type) in the academic
   year.
2. The remark is unique per (student, exam type, academic year).
3. The remark is rendered on the report card.
4. The teacher may update (UpdateTeacherRemark) or delete
   (DeleteTeacherRemark) the remark before result publication.
   After publication, only updates are allowed; deletion requires
   SchoolAdmin.
```

## Exam-Day Attendance Workflow

```text
1. The school day begins. The exam schedule is active.
2. The class teacher marks exam attendance per subject
   (MarkExamAttendance).
3. ExamAttendanceMarked is emitted.
4. The result service consumes exam attendance during result
   computation. Students marked absent receive
   `is_absent=true` on the corresponding MarksRegisterChild.
5. Reports show per-exam attendance summaries (per
   class_attendances).
```

**Edge cases:**
- A student is marked absent on the exam but present on the
  register → engine surfaces `ConflictError::AttendanceConflict`.
  The operator must reconcile.
- Attendance is marked after the exam was already published → the
  operator must republish.

## Idempotency

- `CreateExam` is idempotent on
  `(school_id, exam_type_id, class_id, section_id, subject_id,
  academic_year_id)`. A duplicate returns the existing exam.
- `CreateExamType` is idempotent on `(school_id, title)`. A
  duplicate returns the existing exam type.
- `CreateMarksGrade` rejects overlap. Updating an existing
  `MarksGrade` is the supported way to adjust the scale.
- `EnterMarks` is not idempotent in the strict sense — repeated
  calls overwrite marks. The submit is the closure step.
- `GenerateAdmitCard` is idempotent on
  `(student_record_id, exam_type_id, academic_id)`. A duplicate
  returns the existing card unless the school explicitly requests
  regeneration.
- `MarkTeacherEvaluation` is idempotent on
  `(student_id, teacher_id, subject_id, record_id,
  academic_year_id)`. A duplicate updates the existing row.

## Failure Path Summary

| Stage             | Failure                                         | Engine Response                          |
| ----------------- | ----------------------------------------------- | ---------------------------------------- |
| Exam authoring    | Duplicate tuple                                | `ValidationError::UniqueViolation`       |
| Scheduling        | Teacher/room overlap                            | `ConflictError::Overlap`                 |
| Marks entry       | Marks out of range                              | `ValidationError::OutOfRange`            |
| Marks entry       | Marks for an uninitialized subject              | `NotFoundError`                          |
| Marks submission  | Partial submission when not allowed             | `ValidationError::PartialSubmission`     |
| Result publication| Empty or overlapping `MarksGrade`               | `ValidationError::GradeScaleInvalid`     |
| Online exam       | Submission after close                          | `ConflictError::ExamClosed`              |
| Admit card        | Exam not scheduled                              | `ValidationError::ExamNotScheduled`      |
| Teacher evaluation| Feature disabled or window closed               | `ValidationError::FeatureDisabled`       |
