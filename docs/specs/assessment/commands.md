# Assessment Domain — Commands

Commands describe intent. They are validated, authorized, and
dispatched to the relevant aggregate. Every command produces zero or
more events that are recorded in the event log.

All commands carry a `TenantContext` (school + actor + correlation)
and are rejected if the actor lacks the required capability.

## CreateExamType

```rust
pub struct CreateExamTypeCommand {
    pub tenant: TenantContext,
    pub title: String,
    pub is_average: bool,
    pub percentage: Percentage,
    pub average_mark: AverageMark,
    pub parent_id: Option<ExamTypeId>,
}
```

**Capability:** `ExamType.Create`
**Pre-conditions:**
- Title is unique within the school.
- `Percentage` is in `[0, 100]`.
- `AverageMark >= 0`.
- `parent_id` (if any) belongs to the same school.

**Effects:** Creates an `ExamType`, emits `ExamTypeCreated`.

## UpdateExamType

```rust
pub struct UpdateExamTypeCommand {
    pub tenant: TenantContext,
    pub exam_type_id: ExamTypeId,
    pub title: Option<String>,
    pub is_average: Option<bool>,
    pub percentage: Option<Percentage>,
    pub average_mark: Option<AverageMark>,
}
```

**Capability:** `ExamType.Update`
**Effects:** Emits `ExamTypeUpdated`.

## DeleteExamType

```rust
pub struct DeleteExamTypeCommand {
    pub tenant: TenantContext,
    pub exam_type_id: ExamTypeId,
}
```

**Capability:** `ExamType.Delete`
**Pre-conditions:** No `Exam` references this type.

**Effects:** Emits `ExamTypeDeleted`.

## CreateExam

```rust
pub struct CreateExamCommand {
    pub tenant: TenantContext,
    pub exam_type_id: ExamTypeId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub subject_id: SubjectId,
    pub exam_mark: ExamMark,
    pub pass_mark: PassMark,
    pub academic_year_id: AcademicYearId,
    pub parent_id: Option<ExamId>, // for composite exam terms
}
```

**Capability:** `Exam.Create`
**Pre-conditions:**
- `exam_type_id`, `class_id`, `section_id`, `subject_id` exist and
  belong to the same school.
- `PassMark <= ExamMark`.
- No existing `Exam` with the same `(exam_type_id, class_id,
  section_id, subject_id, academic_year_id)`.

**Effects:** Creates an `Exam`, emits `ExamCreated`.

## UpdateExam

```rust
pub struct UpdateExamCommand {
    pub tenant: TenantContext,
    pub exam_id: ExamId,
    pub exam_mark: Option<ExamMark>,
    pub pass_mark: Option<PassMark>,
}
```

**Capability:** `Exam.Update`
**Pre-conditions:** Marks have not yet been entered against the
exam. Once marks exist, the exam is locked.

**Effects:** Emits `ExamUpdated`.

## DeleteExam

```rust
pub struct DeleteExamCommand {
    pub tenant: TenantContext,
    pub exam_id: ExamId,
}
```

**Capability:** `Exam.Delete`
**Pre-conditions:** No `MarksRegister` references the exam.

**Effects:** Emits `ExamDeleted`.

## ScheduleExam

```rust
pub struct ScheduleExamCommand {
    pub tenant: TenantContext,
    pub exam_id: ExamId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub date: ExamDate,
    pub start_time: StartTime,
    pub end_time: EndTime,
    pub room_id: Option<ClassRoomId>,
    pub teacher_id: Option<StaffId>,
    pub exam_period_id: Option<ClassTimeId>,
    pub subjects: Vec<ScheduleSubjectEntry>,
}

pub struct ScheduleSubjectEntry {
    pub subject_id: SubjectId,
    pub date: ExamDate,
    pub start_time: StartTime,
    pub end_time: EndTime,
    pub room: Option<String>,
    pub full_mark: FullMark,
    pub pass_mark: PassMark,
}
```

**Capability:** `Exam.Schedule`
**Pre-conditions:**
- The exam is created and not yet scheduled (or the existing
  schedule is being replaced).
- The teacher and room have no overlap with other schedules in the
  school at the same time.
- The `date` is within the academic year.

**Effects:** Creates `ExamSchedule` and one `ExamScheduleSubject`
per entry, emits `ExamScheduled`.

## UpdateExamSchedule

```rust
pub struct UpdateExamScheduleCommand {
    pub tenant: TenantContext,
    pub schedule_id: ExamScheduleId,
    pub date: Option<ExamDate>,
    pub start_time: Option<StartTime>,
    pub end_time: Option<EndTime>,
    pub room_id: Option<ClassRoomId>,
    pub teacher_id: Option<StaffId>,
}
```

**Capability:** `Exam.Schedule`
**Effects:** Emits `ExamScheduleUpdated`. Re-runs conflict checks
when time, room, or teacher change.

## CancelExamSchedule

```rust
pub struct CancelExamScheduleCommand {
    pub tenant: TenantContext,
    pub schedule_id: ExamScheduleId,
    pub reason: String,
}
```

**Capability:** `Exam.Schedule`
**Effects:** Marks the schedule inactive, emits
`ExamScheduleCancelled`.

## InitializeMarksRegister

```rust
pub struct InitializeMarksRegisterCommand {
    pub tenant: TenantContext,
    pub exam_id: ExamId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub subject_ids: Vec<SubjectId>,
}
```

**Capability:** `Marks.Initialize`
**Pre-conditions:** Exam exists, schedule exists, students are
enrolled in the section.

**Effects:** Creates one `MarksRegister` per student in the
section and one `MarksRegisterChild` per subject, emits
`MarksRegisterCreated` per register.

## EnterMarks

```rust
pub struct EnterMarksCommand {
    pub tenant: TenantContext,
    pub marks_register_id: MarksRegisterId,
    pub subject_id: SubjectId,
    pub student_id: StudentId,
    pub marks: Option<Marks>, // None means absent
    pub is_absent: bool,
    pub comments: Option<String>,
}
```

**Capability:** `Marks.Enter`
**Pre-conditions:**
- Marks register exists and is not cancelled.
- Subject is part of the register.
- `marks` (if Some) is in `[0, full_mark]`.
- The exam is not yet published.

**Effects:** Upserts the `MarksRegisterChild`, emits `MarksEntered`.
Repeated `EnterMarks` for the same `(register, subject, student)`
overwrites the prior mark; `MarksEntered` is emitted each time.

## SubmitMarks

```rust
pub struct SubmitMarksCommand {
    pub tenant: TenantContext,
    pub marks_register_id: MarksRegisterId,
}
```

**Capability:** `Marks.Submit`
**Pre-conditions:**
- All `MarksRegisterChild` rows are present (or the school
  policy allows partial submission — see `exam_step_skips`).
- The exam is not yet published.

**Effects:** Locks the register, computes grades, emits
`MarksSubmitted` (carries the full per-subject marks).

## PublishResult

```rust
pub struct PublishResultCommand {
    pub tenant: TenantContext,
    pub exam_id: ExamId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub academic_year_id: AcademicYearId,
    pub published_at: Timestamp,
}
```

**Capability:** `Result.Publish`
**Pre-conditions:**
- All marks have been submitted.
- The school has a non-empty `MarksGrade` scale.
- The exam is not yet published (or is being republished).

**Effects:** Materializes `ResultStore` rows, computes
`MeritPosition`, `ExamWisePosition`, and `AllExamWisePosition`,
emits `ResultPublished` per student.

## RepublishResult

```rust
pub struct RepublishResultCommand {
    pub tenant: TenantContext,
    pub result_store_id: ResultStoreId,
    pub reason: String,
    pub republished_at: Timestamp,
}
```

**Capability:** `Result.Publish`
**Effects:** Re-applies result computation, emits
`ResultRepublished`. Marks the previous publication as superseded.

## GenerateReportCard

```rust
pub struct GenerateReportCardCommand {
    pub tenant: TenantContext,
    pub result_store_id: ResultStoreId,
    pub student_id: StudentId,
    pub include_remarks: bool,
}
```

**Capability:** `ReportCard.Generate`
**Pre-conditions:** Result is published.

**Effects:** Materializes a structured report-card payload, emits
`ReportCardGenerated`. The consumer adapter renders PDF/HTML.

## CreateOnlineExam

```rust
pub struct CreateOnlineExamCommand {
    pub tenant: TenantContext,
    pub title: String,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub subject_id: SubjectId,
    pub academic_year_id: AcademicYearId,
    pub date: ExamDate,
    pub start_time: StartTime,
    pub end_time: EndTime,
    pub end_date_time: ExamDateTime,
    pub percentage: Percentage,
    pub instruction: String,
    pub question_bank_ids: Vec<QuestionBankId>,
    pub auto_mark: bool,
}
```

**Capability:** `OnlineExam.Create`
**Pre-conditions:** Class, section, subject, and academic year
exist. `start_time < end_time` and `end_time <= end_date_time`.

**Effects:** Creates `OnlineExam` (status `Pending`) and
`QuestionAssignment` rows for each question bank id, emits
`OnlineExamCreated`.

## PublishOnlineExam

```rust
pub struct PublishOnlineExamCommand {
    pub tenant: TenantContext,
    pub online_exam_id: OnlineExamId,
}
```

**Capability:** `OnlineExam.Publish`
**Pre-conditions:** Exam is in `Pending` status. At least one
`QuestionAssignment` exists.

**Effects:** Sets status to `Published`, emits `OnlineExamPublished`.

## StartOnlineExam

```rust
pub struct StartOnlineExamCommand {
    pub tenant: TenantContext,
    pub online_exam_id: OnlineExamId,
}
```

**Capability:** `OnlineExam.Start`
**Pre-conditions:** Exam is `Published` and within its time window.

**Effects:** Sets `IsWaiting=true` then `IsRunning=true`, emits
`OnlineExamStarted` (per student who begins the exam).

## SubmitOnlineExamAnswer

```rust
pub struct SubmitOnlineExamAnswerCommand {
    pub tenant: TenantContext,
    pub online_exam_id: OnlineExamId,
    pub student_id: StudentId,
    pub question_id: OnlineExamQuestionId,
    pub user_answer: String,
}
```

**Capability:** `OnlineExam.Answer`
**Pre-conditions:**
- Exam is `Running`.
- The student has an active `StudentTakeOnlineExam` with status
  `NotYet`.
- The question belongs to the exam.

**Effects:** Upserts the per-question answer, emits
`OnlineExamAnswered`.

## EvaluateOnlineExam

```rust
pub struct EvaluateOnlineExamCommand {
    pub tenant: TenantContext,
    pub online_exam_id: OnlineExamId,
    pub close_exam: bool,
}
```

**Capability:** `OnlineExam.Evaluate`
**Pre-conditions:** Exam is `Running` and the actor has evaluation
rights (teacher for the section, or exam cell).

**Effects:** Computes per-student `OnlineExamMark`,
`OnlineExamStudentAnswerMarking`, sets `StudentTakeOnlineExam.Status =
GotMarks`. If `close_exam=true`, sets `IsClosed=true` and emits
`OnlineExamClosed`. Emits `OnlineExamEvaluated` per student.

## GenerateSeatPlan

```rust
pub struct GenerateSeatPlanCommand {
    pub tenant: TenantContext,
    pub exam_id: ExamId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub exam_type_id: ExamTypeId,
    pub allocations: Vec<SeatPlanAllocation>,
}

pub struct SeatPlanAllocation {
    pub room_id: ClassRoomId,
    pub assign_students: u32,
    pub start_time: StartTime,
    pub end_time: EndTime,
}
```

**Capability:** `SeatPlan.Generate`
**Pre-conditions:**
- The exam is scheduled.
- Sum of `assign_students` equals the section's student count.
- Allocations do not overlap in time.

**Effects:** Creates a `SeatPlan` and one `SeatPlanChild` per
allocation, emits `SeatPlanGenerated`.

## GenerateAdmitCard

```rust
pub struct GenerateAdmitCardCommand {
    pub tenant: TenantContext,
    pub student_record_id: StudentRecordId,
    pub exam_type_id: ExamTypeId,
    pub academic_year_id: AcademicYearId,
}
```

**Capability:** `AdmitCard.Generate`
**Pre-conditions:**
- The student has an active `StudentRecord` in the academic year.
- The school has an `AdmitCardSetting` for the academic year.
- A `SeatPlan` exists for the student's section and exam type.
- An `ExamSchedule` exists for at least one subject in the exam
  type.

**Effects:** Creates an `AdmitCard`, emits `AdmitCardGenerated`.

## SetExamSignature

```rust
pub struct SetExamSignatureCommand {
    pub tenant: TenantContext,
    pub title: String,
    pub signature: FileReference,
    pub active_status: bool,
}
```

**Capability:** `ExamSignature.Set`
**Effects:** Creates or updates the `ExamSignature`, emits
`ExamSignatureCreated` or `ExamSignatureUpdated`.

## ConfigureCustomResultSettings

```rust
pub struct ConfigureCustomResultSettingsCommand {
    pub tenant: TenantContext,
    pub exam_type_id: ExamTypeId,
    pub exam_percentage: ExamPercentage,
    pub merit_list_setting: String,
    pub print_status: Option<String>,
    pub profile_image: Option<FileReference>,
    pub header_background: Option<FileReference>,
    pub body_background: Option<FileReference>,
    pub vertical_boarder: Option<String>,
    pub academic_year_id: AcademicYearId,
}
```

**Capability:** `Result.Configure`
**Effects:** Upserts `CustomResultSetting`, emits
`CustomResultSettingUpdated`.

## MarkTeacherEvaluation

```rust
pub struct MarkTeacherEvaluationCommand {
    pub tenant: TenantContext,
    pub teacher_id: StaffId,
    pub subject_id: SubjectId,
    pub student_id: StudentId,
    pub record_id: StudentRecordId,
    pub academic_year_id: AcademicYearId,
    pub rating: Rating,
    pub comment: Option<Comment>,
    pub role_id: Option<u32>,
    pub parent_id: Option<TeacherEvaluationId>,
}
```

**Capability:** `TeacherEvaluation.Mark`
**Pre-conditions:**
- `school_id` allows teacher evaluation (see
  `teacher_evaluation_settings.is_enable`).
- The student is enrolled in the subject for the academic year.
- The rating is in `[1, 5]`.

**Effects:** Creates or updates `TeacherEvaluation`, emits
`TeacherEvaluationCompleted`. If the school's `auto_approval` flag
is true, the row is also `Approved`.

## AddTeacherRemark

```rust
pub struct AddTeacherRemarkCommand {
    pub tenant: TenantContext,
    pub teacher_id: StaffId,
    pub student_id: StudentId,
    pub exam_type_id: ExamTypeId,
    pub academic_year_id: AcademicYearId,
    pub remark: Remark,
}
```

**Capability:** `TeacherRemark.Add`
**Pre-conditions:**
- No existing remark for the same `(student_id, exam_type_id,
  academic_id)` in the school.
- The teacher is the class teacher or subject teacher for the
  student's section in the academic year.

**Effects:** Creates a `TeacherRemark`, emits `TeacherRemarkAdded`.

## UpdateTeacherRemark

```rust
pub struct UpdateTeacherRemarkCommand {
    pub tenant: TenantContext,
    pub remark_id: TeacherRemarkId,
    pub remark: Remark,
}
```

**Capability:** `TeacherRemark.Update`
**Effects:** Emits `TeacherRemarkUpdated`.

## ApproveTeacherEvaluation

```rust
pub struct ApproveTeacherEvaluationCommand {
    pub tenant: TenantContext,
    pub evaluation_id: TeacherEvaluationId,
}
```

**Capability:** `TeacherEvaluation.Approve`
**Effects:** Emits `TeacherEvaluationApproved`.

## RejectTeacherEvaluation

```rust
pub struct RejectTeacherEvaluationCommand {
    pub tenant: TenantContext,
    pub evaluation_id: TeacherEvaluationId,
    pub reason: String,
}
```

**Capability:** `TeacherEvaluation.Approve`
**Effects:** Marks the row inactive, emits
`TeacherEvaluationRejected`.

## CreateMarksGrade / UpdateMarksGrade / DeleteMarksGrade

```rust
pub struct CreateMarksGradeCommand {
    pub tenant: TenantContext,
    pub grade_name: Grade,
    pub gpa: Gpa,
    pub from: f32,
    pub up: f32,
    pub percent_from: f32,
    pub percent_up_to: f32,
    pub description: Option<String>,
}
```

**Capabilities:** `MarksGrade.Create`, `MarksGrade.Update`,
`MarksGrade.Delete`.

The grade scale must be non-overlapping and contiguous; the engine
rejects overlapping or gapped ranges.

## MarkExamAttendance / UpdateExamAttendance

```rust
pub struct MarkExamAttendanceCommand {
    pub tenant: TenantContext,
    pub exam_id: ExamId,
    pub subject_id: SubjectId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub academic_year_id: AcademicYearId,
    pub marks: Vec<ExamAttendanceMark>,
}

pub struct ExamAttendanceMark {
    pub student_id: StudentId,
    pub student_record_id: StudentRecordId,
    pub attendance_type: AttendanceType, // P or A
}
```

**Capability:** `ExamAttendance.Mark`
**Pre-conditions:**
- Exam is scheduled.
- All students in the section are covered.

**Effects:** Upserts the `ExamAttendance` roll and child rows,
emits `ExamAttendanceMarked`.

`UpdateExamAttendance` updates an existing roll and emits
`ExamAttendanceUpdated`.

## CreateExamSetting / UpdateExamSetting / DeleteExamSetting

```rust
pub struct CreateExamSettingCommand {
    pub tenant: TenantContext,
    pub exam_type: ExamTerm,
    pub title: String,
    pub publish_date: PublishDate,
    pub start_date: ExamDate,
    pub end_date: ExamDate,
    pub file: Option<FileReference>,
}
```

**Capabilities:** `ExamSetting.Create`, `ExamSetting.Update`,
`ExamSetting.Delete`.

## ConfigureAdmitCardSettings

```rust
pub struct ConfigureAdmitCardSettingsCommand {
    pub tenant: TenantContext,
    pub academic_year_id: AcademicYearId,
    pub student_photo: bool,
    pub student_name: bool,
    pub admission_no: bool,
    pub class_section: bool,
    pub exam_name: bool,
    pub academic_year: bool,
    pub principal_signature: bool,
    pub class_teacher_signature: bool,
    pub guardian_name: bool,
    pub school_address: bool,
    pub student_download: bool,
    pub parent_download: bool,
    pub student_notification: bool,
    pub parent_notification: bool,
    pub principal_signature_photo: Option<FileReference>,
    pub teacher_signature_photo: Option<FileReference>,
    pub admit_layout: AdmitLayout, // 1 portrait, 2 landscape
    pub admit_sub_title: Option<String>,
    pub description: Option<String>,
}
```

**Capability:** `AdmitCard.Configure`
**Effects:** Upserts `AdmitCardSetting`, emits
`AdmitCardSettingUpdated`.

## ConfigureSeatPlanSettings

```rust
pub struct ConfigureSeatPlanSettingsCommand {
    pub tenant: TenantContext,
    pub academic_year_id: AcademicYearId,
    pub school_name: bool,
    pub student_photo: bool,
    pub student_name: bool,
    pub admission_no: bool,
    pub class_section: bool,
    pub exam_name: bool,
    pub roll_no: bool,
    pub academic_year: bool,
}
```

**Capability:** `SeatPlan.Configure`
**Effects:** Upserts `SeatPlanSetting`, emits
`SeatPlanSettingUpdated`.

## ConfigureTeacherEvaluation

```rust
pub struct ConfigureTeacherEvaluationCommand {
    pub tenant: TenantContext,
    pub is_enable: bool,
    pub submitted_by: Vec<String>, // e.g. ["student", "parent"]
    pub rating_submission_time: String, // e.g. "any", "after_exam"
    pub auto_approval: bool,
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
}
```

**Capability:** `TeacherEvaluation.Configure`
**Effects:** Upserts the per-school `TeacherEvaluationSetting`,
emits `TeacherEvaluationConfigured`.

## PublishExamRoutine / PublishFrontResult

```rust
pub struct PublishExamRoutineCommand {
    pub tenant: TenantContext,
    pub title: String,
    pub publish_date: PublishDate,
    pub result_file: Option<FileReference>,
}

pub struct PublishFrontResultCommand {
    pub tenant: TenantContext,
    pub title: String,
    pub publish_date: PublishDate,
    pub result_file: Option<FileReference>,
    pub link: Option<String>,
}
```

**Capabilities:** `ExamRoutine.Publish`, `FrontendResult.Publish`.
**Effects:** Emits `FrontendExamRoutinePublished` /
`FrontendResultPublished`.

## UpdateExamRoutinePage / UpdateFrontendExamResult

Marketer-facing updates to the public landing pages.

```rust
pub struct UpdateExamRoutinePageCommand {
    pub tenant: TenantContext,
    pub title: Option<String>,
    pub description: Option<String>,
    pub main_title: Option<String>,
    pub main_description: Option<String>,
    pub image: Option<FileReference>,
    pub main_image: Option<FileReference>,
    pub button_text: Option<String>,
    pub button_url: Option<String>,
    pub is_parent: bool,
    pub class_routine: Visibility,
    pub exam_routine: Visibility,
}
```

**Capability:** `ExamRoutinePage.Update`
**Effects:** Emits `ExamRoutinePageUpdated`.

## MarkExamStepSkip

```rust
pub struct MarkExamStepSkipCommand {
    pub tenant: TenantContext,
    pub name: String,
}
```

**Capability:** `Exam.Configure`
**Effects:** Creates or updates `ExamStepSkip`, emits
`ExamStepSkipSet`.

## SendAbsenceNotification (cross-domain trigger)

Although notification dispatch is owned by the communication domain,
assessment emits the trigger event. The corresponding command in
assessment is `RequestAbsenceNotification`:

```rust
pub struct RequestAbsenceNotificationCommand {
    pub tenant: TenantContext,
    pub exam_attendance_id: ExamAttendanceId,
    pub channel: NotificationChannel, // SMS, Email, Push
}
```

**Capability:** `ExamAttendance.Notify`
**Effects:** Emits `ExamAbsenceNotificationRequested` (the
communication domain subscribes).

## Standard CRUD Variants

The following aggregates expose standard create/update/delete
commands:

- `QuestionBank` — `CreateQuestion`, `UpdateQuestion`,
  `DeleteQuestion` (capabilities `Question.Create`,
  `Question.Update`, `Question.Delete`).
- `QuestionGroup` — `CreateQuestionGroup`, `UpdateQuestionGroup`,
  `DeleteQuestionGroup`.
- `QuestionLevel` — `CreateQuestionLevel`, `UpdateQuestionLevel`,
  `DeleteQuestionLevel`.
- `OnlineExamQuestion` — `AddOnlineExamQuestion`,
  `UpdateOnlineExamQuestion`, `DeleteOnlineExamQuestion`. An option
  CRUD pair (`AddQuestionOption`, `UpdateQuestionOption`,
  `DeleteQuestionOption`) lives on `QuestionMuOption`.
- `ExamStepSkip` — `MarkExamStepSkip` (see above).
