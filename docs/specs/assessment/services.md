# Assessment Domain — Services

Domain services encapsulate business logic that does not fit cleanly
in a single aggregate. They are stateless, sync, and pure (no I/O).

## ExamService

```rust
pub struct ExamService;

impl ExamService {
    pub fn plan_for_class(
        exam_type: &ExamType,
        class: &Class,
        year: AcademicYearId,
    ) -> Vec<ExamSpec>;

    pub fn validate_no_teacher_overlap(
        proposed: &ExamSchedule,
        existing: &[ExamSchedule],
    ) -> Result<(), ConflictError>;

    pub fn validate_no_room_overlap(
        proposed: &ExamSchedule,
        existing: &[ExamSchedule],
    ) -> Result<(), ConflictError>;

    pub fn lock_after_publish(exam: &mut Exam);
}
```

`ExamService::lock_after_publish` is called by the result-publication
service to set a flag on the exam that prevents further mutation
from `UpdateExam`.

## MarksService

```rust
pub struct MarksService;

impl MarksService {
    pub fn initialize_registers(
        exam: &Exam,
        section: &ClassSection,
        subjects: &[Subject],
    ) -> Vec<MarksRegister>;

    pub fn validate_marks(
        marks: Marks,
        full_mark: FullMark,
        pass_mark: PassMark,
    ) -> Result<(), ValidationError>;

    pub fn is_absent_row(child: &MarksRegisterChild) -> bool;

    pub fn submit(register: &mut MarksRegister) -> Result<(), ValidationError>;
}
```

`MarksService::submit` enforces partial-submission rules: when
`exam_step_skips` indicates partial submission is allowed, missing
subjects are tolerated; otherwise the register must be complete.

## ResultService

```rust
pub struct ResultService;

impl ResultService {
    pub fn compute_grade(
        percent: Percentage,
        scale: &[MarksGrade],
    ) -> (Grade, Gpa);

    pub fn compute_subject_marks(
        child: &MarksRegisterChild,
        exam: &Exam,
    ) -> (Marks, Gpa, Grade);

    pub fn compute_total(
        children: &[MarksRegisterChild],
        exam: &Exam,
        scale: &[MarksGrade],
    ) -> (TotalMarks, Gpa, Grade, ResultStatus);

    pub fn determine_pass_fail(
        children: &[MarksRegisterChild],
        exam: &Exam,
    ) -> ResultStatus;

    pub fn rank_section(
        results: &[ResultStore],
    ) -> Vec<MeritPosition>;

    pub fn rank_all_sections(
        results: &[ResultStore],
    ) -> Vec<AllExamWisePosition>;

    pub fn build_result_store(
        exam: &Exam,
        setup: &ExamSetup,
        student: &StudentRecord,
        children: &[MarksRegisterChild],
        scale: &[MarksGrade],
    ) -> ResultStore;

    pub fn build_custom_temporary(
        result: &ResultStore,
        custom: &CustomResultSetting,
    ) -> CustomTemporaryResult;

    pub fn publish(
        exam: &Exam,
        section: &ClassSection,
        registers: &[MarksRegister],
        scale: &[MarksGrade],
    ) -> Result<PublishOutcome, ValidationError>;
}
```

`ResultService::publish` is the heart of result publication. It
materializes `ResultStore` rows, `MeritPosition` rows,
`ExamWisePosition` rows, `AllExamWisePosition` rows, and (if
configured) `CustomTemporaryResult` rows. It is sync and pure; the
caller persists the rows and emits the events.

## ReportCardService

```rust
pub struct ReportCardService;

impl ReportCardService {
    pub fn build_payload(
        result: &ResultStore,
        exam: &Exam,
        student: &StudentRecord,
        section: &ClassSection,
        signatures: &[ExamSignature],
        remarks: Option<&TeacherRemark>,
        attendance: Option<&ClassAttendanceSummary>,
    ) -> ReportCardPayload;

    pub fn render_html(payload: &ReportCardPayload) -> String;
    pub fn render_pdf(payload: &ReportCardPayload) -> Vec<u8>;
}
```

`ReportCardService::render_html` and `render_pdf` are
consumer-supplied adapters; the domain service exposes the
**structured payload** and delegates rendering.

## SeatPlanService

```rust
pub struct SeatPlanService;

impl SeatPlanService {
    pub fn assign_rooms(
        section: &ClassSection,
        allocations: &[SeatPlanAllocation],
    ) -> Result<Vec<StudentSeat>, ConflictError>;

    pub fn validate_total(
        allocations: &[SeatPlanAllocation],
        student_count: u32,
    ) -> Result<(), ValidationError>;

    pub fn validate_no_room_overlap(
        allocations: &[SeatPlanAllocation],
    ) -> Result<(), ConflictError>;

    pub fn build_seat_plan(
        exam_type: &ExamType,
        section: &ClassSection,
        allocations: Vec<SeatPlanAllocation>,
    ) -> SeatPlan;
}
```

`SeatPlanService::assign_rooms` is a deterministic algorithm that
distributes students to rooms (alphabetical, or random per
`SeatPlanSetting`). Consumers may override the algorithm by
subclassing the policy.

## AdmitCardService

```rust
pub struct AdmitCardService;

impl AdmitCardService {
    pub fn build_card(
        student: &StudentRecord,
        exam_type: &ExamType,
        setting: &AdmitCardSetting,
        seat_plan: &SeatPlan,
        schedule: &ExamSchedule,
    ) -> AdmitCardPayload;

    pub fn render_html(payload: &AdmitCardPayload) -> String;
    pub fn render_pdf(payload: &AdmitCardPayload) -> Vec<u8>;
}
```

## OnlineExamService

```rust
pub struct OnlineExamService;

impl OnlineExamService {
    pub fn start(
        exam: &mut OnlineExam,
        now: Timestamp,
    ) -> Result<(), ValidationError>;

    pub fn accept_answer(
        exam: &OnlineExam,
        attempt: &mut StudentTakeOnlineExam,
        question: &OnlineExamQuestion,
        answer: String,
    ) -> Result<(), ValidationError>;

    pub fn auto_evaluate(
        exam: &OnlineExam,
        attempt: &mut StudentTakeOnlineExam,
    ) -> EvaluationResult;

    pub fn manual_mark(
        exam: &OnlineExam,
        attempt: &mut StudentTakeOnlineExam,
        per_question: BTreeMap<OnlineExamQuestionId, Marks>,
        marked_by: StaffId,
    ) -> EvaluationResult;

    pub fn close(
        exam: &mut OnlineExam,
        now: Timestamp,
    ) -> Result<(), ValidationError>;
}
```

`OnlineExamService::auto_evaluate` produces a `EvaluationResult` that
contains the per-question `OnlineExamStudentAnswerMarking` rows and
the final `OnlineExamMark`. The caller persists and emits the
events.

## TeacherEvaluationService

```rust
pub struct TeacherEvaluationService;

impl TeacherEvaluationService {
    pub fn is_window_open(
        setting: &TeacherEvaluationSetting,
        now: NaiveDate,
    ) -> bool;

    pub fn can_submit(
        setting: &TeacherEvaluationSetting,
        actor: &Actor,
    ) -> bool;

    pub fn build_evaluation(
        teacher: &Staff,
        subject: &Subject,
        student: &Student,
        record: &StudentRecord,
        rating: Rating,
        comment: Option<String>,
    ) -> TeacherEvaluation;

    pub fn aggregate(
        evaluations: &[TeacherEvaluation],
    ) -> TeacherRatingSummary;
}
```

`TeacherEvaluationService::aggregate` produces a per-teacher summary
for reports (average rating, count, etc.).

## MarksGradeService

```rust
pub struct MarksGradeService;

impl MarksGradeService {
    pub fn validate_no_overlap(
        scale: &[MarksGrade],
    ) -> Result<(), ValidationError>;

    pub fn validate_contiguous(
        scale: &[MarksGrade],
    ) -> Result<(), ValidationError>;

    pub fn find_grade(
        percent: Percentage,
        scale: &[MarksGrade],
    ) -> Option<&MarksGrade>;
}
```

## Policy: ResultEligibility

```rust
pub struct ResultEligibility;

impl Policy<PublishResultCommand> for ResultEligibility {
    type Outcome = Eligible | NotEligible { reason: &'static str };
    fn check(&self, ctx: &Context, cmd: &PublishResultCommand) -> Outcome { ... }
}
```

A result is eligible for publication when all `MarksRegisterChild`
rows are present (or partial submission is allowed), the
`MarksGrade` scale is valid, and the exam is not yet published.

## Policy: AdmitCardEligibility

A student is eligible for an admit card when their section's seat
plan is generated, the exam schedule exists, and the
`AdmitCardSetting` is configured.

## Specification: ActiveExamSchedule

```rust
pub struct ActiveExamSchedule;

impl Specification<ExamSchedule> for ActiveExamSchedule {
    fn is_satisfied_by(&self, s: &ExamSchedule) -> bool { ... }
}
```

Used by queries to list "currently active" or "upcoming" exam
schedules.

## Specification: PendingOnlineExam

```rust
pub struct PendingOnlineExam;

impl Specification<OnlineExam> for PendingOnlineExam {
    fn is_satisfied_by(&self, e: &OnlineExam) -> bool { ... }
}
```

Used by queries to list online exams that need attention.

## Cross-Domain Coordinator

A thin coordinator lives in the engine facade and orchestrates
multi-domain flows (e.g. result publication + report card +
notification). It is **not** a service; it composes command calls:

```rust
pub struct AssessmentCoordinator<'a> {
    engine: &'a Engine,
}

impl<'a> AssessmentCoordinator<'a> {
    pub async fn publish_result(
        &self,
        cmd: PublishResultCommand,
    ) -> Result<ResultSummary, DomainError> {
        self.engine.assessment().publish_result(cmd.clone()).await?;
        // Subscribers (communication, finance, cms, academic) handle
        // their own side effects in response to ResultPublished.
        Ok(ResultSummary::from(cmd))
    }
}
```

Domain services are pure. Cross-domain coordination happens
through events and command composition, never through
service-to-service calls.
