# Assessment Domain — Events

Domain events describe facts that have already happened. They are
immutable, append-only records used for cross-domain integration,
audit, and event sourcing.

All events implement:

```rust
pub trait DomainEvent: Serialize + DeserializeOwned + Send + Sync {
    const TYPE: &'static str;
    fn aggregate_id(&self) -> Uuid;
    fn school_id(&self) -> SchoolId;
    fn occurred_at(&self) -> Timestamp;
}
```

The event envelope wraps the event with metadata:

```rust
pub struct EventEnvelope<E> {
    pub event_id: EventId,
    pub event_type: &'static str,
    pub school_id: SchoolId,
    pub aggregate_id: Uuid,
    pub aggregate_type: &'static str,
    pub actor_id: UserId,
    pub correlation_id: CorrelationId,
    pub causation_id: Option<EventId>,
    pub occurred_at: Timestamp,
    pub payload: E,
}
```

## Exam Type

```rust
pub struct ExamTypeCreated {
    pub exam_type_id: ExamTypeId,
    pub title: String,
    pub is_average: bool,
    pub percentage: Percentage,
    pub average_mark: AverageMark,
    pub parent_id: Option<ExamTypeId>,
}

pub struct ExamTypeUpdated {
    pub exam_type_id: ExamTypeId,
    pub changes: Vec<&'static str>,
}

pub struct ExamTypeDeleted {
    pub exam_type_id: ExamTypeId,
}
```

## Exam Lifecycle

```rust
pub struct ExamCreated {
    pub exam_id: ExamId,
    pub exam_type_id: ExamTypeId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub subject_id: SubjectId,
    pub exam_mark: ExamMark,
    pub pass_mark: PassMark,
    pub academic_year_id: AcademicYearId,
    pub parent_id: Option<ExamId>,
}

pub struct ExamUpdated {
    pub exam_id: ExamId,
    pub changes: Vec<&'static str>,
}

pub struct ExamDeleted {
    pub exam_id: ExamId,
}
```

**Subscribers of `ExamCreated`:**
- `assessment` (self) — initialize `ExamSetting` if configured.
- `communication` — none.
- `events` — add the exam window to the school calendar.

## Exam Schedule

```rust
pub struct ExamScheduled {
    pub exam_id: ExamId,
    pub schedule_id: ExamScheduleId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub date: ExamDate,
    pub start_time: StartTime,
    pub end_time: EndTime,
    pub room_id: Option<ClassRoomId>,
    pub teacher_id: Option<StaffId>,
    pub subject_count: u32,
}

pub struct ExamScheduleUpdated {
    pub schedule_id: ExamScheduleId,
    pub changes: Vec<&'static str>,
}

pub struct ExamScheduleCancelled {
    pub schedule_id: ExamScheduleId,
    pub reason: String,
}
```

**Subscribers of `ExamScheduled`:**
- `communication` — broadcast the routine to guardians.
- `cms` — refresh the public exam routine page.

## Marks Lifecycle

```rust
pub struct MarksRegisterCreated {
    pub marks_register_id: MarksRegisterId,
    pub exam_id: ExamId,
    pub student_id: StudentId,
    pub subjects: Vec<SubjectId>,
}

pub struct MarksRegisterCancelled {
    pub marks_register_id: MarksRegisterId,
    pub reason: String,
}

pub struct MarksEntered {
    pub marks_register_id: MarksRegisterId,
    pub subject_id: SubjectId,
    pub student_id: StudentId,
    pub marks: Option<Marks>,
    pub is_absent: bool,
    pub comments: Option<String>,
}

pub struct MarksSubmitted {
    pub marks_register_id: MarksRegisterId,
    pub exam_id: ExamId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub subject_count: u32,
    pub total_students: u32,
    pub submitted_at: Timestamp,
}
```

**Subscribers of `MarksSubmitted`:**
- `assessment` (self) — trigger result computation.

## Result Lifecycle

```rust
pub struct ResultStoreCreated {
    pub result_store_id: ResultStoreId,
    pub exam_id: ExamId,
    pub exam_type_id: ExamTypeId,
    pub student_id: StudentId,
    pub subject_id: SubjectId,
    pub total_marks: TotalMarks,
    pub gpa: Gpa,
    pub grade: Grade,
}

pub struct ResultRemarksUpdated {
    pub result_store_id: ResultStoreId,
    pub teacher_remarks: String,
}

pub struct ResultPublished {
    pub exam_id: ExamId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub academic_year_id: AcademicYearId,
    pub student_ids: Vec<StudentId>,
    pub published_at: Timestamp,
}

pub struct ResultRepublished {
    pub exam_id: ExamId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub reason: String,
    pub republished_at: Timestamp,
}
```

**Subscribers of `ResultPublished`:**
- `communication` — send report-card link to guardians.
- `finance` — close any marks-linked obligations.
- `cms` — refresh the public results listing.
- `academic` — enable promotion decisions for the class.
- `assessment` (self) — write merit positions and
  `AllExamWisePosition` rows.

## Report Card

```rust
pub struct ReportCardGenerated {
    pub result_store_id: ResultStoreId,
    pub student_id: StudentId,
    pub exam_id: ExamId,
    pub include_remarks: bool,
    pub payload: ReportCardPayload, // structured, versioned
}
```

## Mark Store

```rust
pub struct MarkStoreCreated {
    pub mark_store_id: MarkStoreId,
    pub exam_setup_id: ExamSetupId,
    pub exam_type_id: ExamTypeId,
    pub student_id: StudentId,
    pub student_record_id: StudentRecordId,
    pub subject_id: SubjectId,
    pub total_marks: TotalMarks,
    pub is_absent: IsAbsent,
}

pub struct TeacherRemarkUpdated {
    pub mark_store_id: MarkStoreId,
    pub teacher_remarks: String,
}

pub struct MarkStoreDeleted {
    pub mark_store_id: MarkStoreId,
}
```

## Result Settings

```rust
pub struct ResultSettingUpdated {
    pub result_setting_id: ResultSettingId,
    pub changes: Vec<&'static str>,
}

pub struct CustomResultSettingUpdated {
    pub custom_result_setting_id: CustomResultSettingId,
    pub exam_type_id: ExamTypeId,
    pub changes: Vec<&'static str>,
}
```

## Marks Grade

```rust
pub struct MarksGradeCreated {
    pub marks_grade_id: MarksGradeId,
    pub grade: Grade,
    pub gpa: Gpa,
    pub from: f32,
    pub up: f32,
    pub percent_from: f32,
    pub percent_up_to: f32,
}

pub struct MarksGradeUpdated {
    pub marks_grade_id: MarksGradeId,
    pub changes: Vec<&'static str>,
}

pub struct MarksGradeDeleted {
    pub marks_grade_id: MarksGradeId,
}
```

## Exam Settings, Signatures, Front-End Pages

```rust
pub struct ExamSettingCreated {
    pub exam_setting_id: ExamSettingId,
    pub exam_type: ExamTerm,
    pub title: String,
    pub publish_date: PublishDate,
    pub start_date: ExamDate,
    pub end_date: ExamDate,
}

pub struct ExamSettingUpdated { pub exam_setting_id: ExamSettingId, pub changes: Vec<&'static str> }
pub struct ExamSettingDeleted { pub exam_setting_id: ExamSettingId }

pub struct ExamSignatureCreated { pub exam_signature_id: ExamSignatureId, pub title: String, pub signature: FileReference }
pub struct ExamSignatureUpdated { pub exam_signature_id: ExamSignatureId, pub changes: Vec<&'static str> }
pub struct ExamSignatureDeleted { pub exam_signature_id: ExamSignatureId }

pub struct ExamRoutinePageUpdated { pub page_id: ExamRoutinePageId, pub changes: Vec<&'static str> }

pub struct FrontExamRoutinePublished { pub front_id: FrontExamRoutineId, pub title: String, pub publish_date: PublishDate }
pub struct FrontResultPublished { pub front_id: FrontResultId, pub title: String, pub publish_date: PublishDate }
pub struct FrontendExamResultUpdated { pub id: FrontendExamResultId, pub changes: Vec<&'static str> }
```

## Exam Setup

```rust
pub struct ExamSetupCreated { pub exam_setup_id: ExamSetupId, pub exam_id: ExamId, pub section_id: SectionId, pub exam_mark: ExamMark }
pub struct ExamSetupUpdated { pub exam_setup_id: ExamSetupId, pub changes: Vec<&'static str> }
pub struct ExamSetupDeleted { pub exam_setup_id: ExamSetupId }
```

## Online Exam Lifecycle

```rust
pub struct OnlineExamCreated {
    pub online_exam_id: OnlineExamId,
    pub title: String,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub subject_id: SubjectId,
    pub date: ExamDate,
    pub start_time: StartTime,
    pub end_time: EndTime,
    pub end_date_time: ExamDateTime,
    pub auto_mark: bool,
}

pub struct OnlineExamUpdated {
    pub online_exam_id: OnlineExamId,
    pub changes: Vec<&'static str>,
}

pub struct OnlineExamPublished {
    pub online_exam_id: OnlineExamId,
    pub published_at: Timestamp,
}

pub struct OnlineExamStarted {
    pub online_exam_id: OnlineExamId,
    pub student_id: StudentId,
    pub started_at: Timestamp,
}

pub struct OnlineExamAnswered {
    pub online_exam_id: OnlineExamId,
    pub student_id: StudentId,
    pub question_id: OnlineExamQuestionId,
    pub user_answer: String,
    pub submitted_at: Timestamp,
}

pub struct OnlineExamEvaluated {
    pub online_exam_id: OnlineExamId,
    pub student_id: StudentId,
    pub total_marks: TotalMarks,
    pub evaluated_at: Timestamp,
}

pub struct OnlineExamClosed {
    pub online_exam_id: OnlineExamId,
    pub closed_at: Timestamp,
}

pub struct OnlineExamDeleted {
    pub online_exam_id: OnlineExamId,
}
```

**Subscribers of `OnlineExamStarted`:**
- `communication` — none directly; the `OnlineExam` system shows
  the running exam to the section.
- `assessment` (self) — record the attempt's start.

## Question Bank

```rust
pub struct QuestionCreated { pub question_id: QuestionBankId, pub question_group_id: Option<QuestionGroupId>, pub question_level_id: Option<QuestionLevelId>, pub question_type: QuestionType, pub mark: Marks, pub title: String }
pub struct QuestionUpdated { pub question_id: QuestionBankId, pub changes: Vec<&'static str> }
pub struct QuestionDeleted { pub question_id: QuestionBankId }

pub struct QuestionGroupCreated { pub group_id: QuestionGroupId, pub title: String }
pub struct QuestionGroupUpdated { pub group_id: QuestionGroupId, pub changes: Vec<&'static str> }
pub struct QuestionGroupDeleted { pub group_id: QuestionGroupId }

pub struct QuestionLevelCreated { pub level_id: QuestionLevelId, pub level: String }
pub struct QuestionLevelUpdated { pub level_id: QuestionLevelId, pub changes: Vec<&'static str> }
pub struct QuestionLevelDeleted { pub level_id: QuestionLevelId }
```

## Online Exam Questions and Marking

```rust
pub struct OnlineExamQuestionAdded { pub question_id: OnlineExamQuestionId, pub online_exam_id: OnlineExamId, pub question_type: QuestionType, pub mark: Marks }
pub struct OnlineExamQuestionUpdated { pub question_id: OnlineExamQuestionId, pub changes: Vec<&'static str> }
pub struct OnlineExamQuestionDeleted { pub question_id: OnlineExamQuestionId }

pub struct QuestionOptionAdded { pub option_id: QuestionMuOptionId, pub question_id: OnlineExamQuestionId, pub title: String, pub status: u8 }
pub struct QuestionOptionUpdated { pub option_id: QuestionMuOptionId, pub changes: Vec<&'static str> }
pub struct QuestionOptionDeleted { pub option_id: QuestionMuOptionId }

pub struct OnlineExamMarkCreated { pub mark_id: OnlineExamMarkId, pub online_exam_id: OnlineExamId, pub student_id: StudentId, pub marks: Marks, pub is_absent: IsAbsent }
```

## Seat Plan

```rust
pub struct SeatPlanGenerated {
    pub seat_plan_id: SeatPlanId,
    pub exam_type_id: ExamTypeId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub rooms: u32,
    pub total_students: u32,
}

pub struct SeatPlanUpdated { pub seat_plan_id: SeatPlanId, pub changes: Vec<&'static str> }
pub struct SeatPlanCancelled { pub seat_plan_id: SeatPlanId }

pub struct SeatPlanSettingUpdated { pub setting_id: SeatPlanSettingId, pub changes: Vec<&'static str> }
```

## Admit Card

```rust
pub struct AdmitCardGenerated {
    pub admit_card_id: AdmitCardId,
    pub student_record_id: StudentRecordId,
    pub exam_type_id: ExamTypeId,
    pub academic_year_id: AcademicYearId,
    pub generated_at: Timestamp,
}

pub struct AdmitCardRegenerated { pub admit_card_id: AdmitCardId, pub previous_id: AdmitCardId, pub reason: String }
pub struct AdmitCardCancelled { pub admit_card_id: AdmitCardId, pub reason: String }

pub struct AdmitCardSettingUpdated { pub setting_id: AdmitCardSettingId, pub changes: Vec<&'static str> }
```

**Subscribers of `AdmitCardGenerated`:**
- `communication` — notify the student and guardian.
- `mobile` — surface the card in the student/parent app.

## Teacher Evaluation

```rust
pub struct TeacherEvaluationCompleted {
    pub evaluation_id: TeacherEvaluationId,
    pub teacher_id: StaffId,
    pub subject_id: SubjectId,
    pub student_id: StudentId,
    pub record_id: StudentRecordId,
    pub academic_year_id: AcademicYearId,
    pub rating: Rating,
    pub comment: Option<String>,
    pub auto_approved: bool,
}

pub struct TeacherEvaluationApproved { pub evaluation_id: TeacherEvaluationId, pub approved_by: StaffId }
pub struct TeacherEvaluationRejected { pub evaluation_id: TeacherEvaluationId, pub reason: String }

pub struct TeacherEvaluationConfigured { pub is_enable: bool, pub submitted_by: Vec<String>, pub rating_submission_time: String, pub auto_approval: bool, pub from_date: Option<NaiveDate>, pub to_date: Option<NaiveDate> }
```

## Teacher Remark

```rust
pub struct TeacherRemarkAdded {
    pub remark_id: TeacherRemarkId,
    pub teacher_id: StaffId,
    pub student_id: StudentId,
    pub exam_type_id: ExamTypeId,
    pub academic_year_id: AcademicYearId,
    pub remark: Remark,
}

pub struct TeacherRemarkUpdated { pub remark_id: TeacherRemarkId, pub remark: Remark }
pub struct TeacherRemarkDeleted { pub remark_id: TeacherRemarkId }
```

## Exam Attendance

```rust
pub struct ExamAttendanceMarked {
    pub exam_attendance_id: ExamAttendanceId,
    pub exam_id: ExamId,
    pub subject_id: SubjectId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub present_count: u32,
    pub absent_count: u32,
    pub marked_by: UserId,
    pub marked_at: Timestamp,
}

pub struct ExamAttendanceUpdated { pub exam_attendance_id: ExamAttendanceId, pub changes: Vec<&'static str> }
```

**Subscribers of `ExamAttendanceMarked`:**
- `assessment` (self) — flag absent students during result
  computation.
- `communication` — none directly; the consumer may subscribe to
  send absence notifications to guardians.

## Exam Step Skip

```rust
pub struct ExamStepSkipSet { pub skip_id: ExamStepSkipId, pub name: String }
```

## Audit

All events are recorded in the per-aggregate event log and emitted
on the event bus. Consumers and adapters consume from the bus to
project read models, send notifications, render report cards, and
refresh public listings.
