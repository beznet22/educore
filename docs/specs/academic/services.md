# Academic Domain — Services

Domain services encapsulate business logic that does not fit cleanly in
a single aggregate. They are stateless, sync, and pure (no I/O).

## AdmissionService

```rust
pub struct AdmissionService;

impl AdmissionService {
    pub fn plan(inquiry: &AdmissionQuery, school: &School) -> AdmissionPlan { ... }
    pub fn validate_capacity(class_section: &ClassSection, count: u32) -> Result<(), CapacityError> { ... }
    pub fn assign_admission_number(school: &School, year: &AcademicYear) -> AdmissionNumber { ... }
    pub fn build_student(cmd: AdmitStudentCommand, school: &School) -> Result<Student, ValidationError> { ... }
}
```

`AdmissionService::plan` is read-only. It returns a plan describing the
admission (class assignment, default roll number, transport eligibility)
but does not mutate state. Consumers use this to preview an admission
before committing.

## PromotionService

```rust
pub struct PromotionService;

impl PromotionService {
    pub fn determine_result(marks: &ResultStore, class: &Class) -> ResultStatus { ... }
    pub fn next_class(current: &Class, school: &School) -> Option<ClassId> { ... }
    pub fn assign_roll_numbers(target: &ClassSection, count: u32) -> Vec<RollNumber> { ... }
    pub fn build_promotion(cmd: PromoteStudentCommand, src: &StudentRecord, dst_class: &Class, dst_section: &Section) -> Result<StudentPromotion, ValidationError> { ... }
}
```

`determine_result` implements the school's pass rule: pass mark per
class, optional subject eligibility, GPA threshold, manual override.

## EnrollmentService

```rust
pub struct EnrollmentService;

impl EnrollmentService {
    pub fn build_enrollment(student: &Student, class: &Class, section: &Section, year: &AcademicYear) -> StudentRecord { ... }
    pub fn validate_capacity(class_section: &ClassSection) -> Result<(), CapacityError> { ... }
    pub fn resolve_class_section(class_id: ClassId, section_id: SectionId, year: &AcademicYear) -> ClassSection { ... }
}
```

## RoutineService

```rust
pub struct RoutineService;

impl RoutineService {
    pub fn validate_no_teacher_overlap(slots: &[RoutineSlot]) -> Result<(), ConflictError> { ... }
    pub fn validate_no_room_overlap(slots: &[RoutineSlot]) -> Result<(), ConflictError> { ... }
    pub fn fill_period_grid(slots: Vec<RoutineSlot>, times: &[ClassTime]) -> WeekGrid { ... }
}
```

## HomeworkService

```rust
pub struct HomeworkService;

impl HomeworkService {
    pub fn is_late(homework: &Homework, submission: &HomeworkSubmission) -> bool { ... }
    pub fn default_marks_range(class: &Class) -> Range<u32> { ... }
    pub fn evaluate(homework: &Homework, marks: Marks) -> Result<(), ValidationError> { ... }
}
```

## LessonPlanService

```rust
pub struct LessonPlanService;

impl LessonPlanService {
    pub fn coverage_percent(plan: &LessonPlan, syllabus: &[LessonTopic]) -> f32 { ... }
    pub fn next_uncovered_topic(plan: &LessonPlan, syllabus: &[LessonTopic]) -> Option<&LessonTopic> { ... }
}
```

## GraduationService

```rust
pub struct GraduationService;

impl GraduationService {
    pub fn is_eligible(student: &Student, year: &AcademicYear, school: &School) -> bool { ... }
    pub fn build_graduate_record(student: &Student, year: &AcademicYear) -> GraduateRecord { ... }
}
```

## ClassSectionAssignmentService

```rust
pub struct ClassSectionAssignmentService;

impl ClassSectionAssignmentService {
    pub fn can_assign_teacher(teacher: &Staff, class_section: &ClassSection) -> bool { ... }
    pub fn can_assign_room(room: &ClassRoom, class_section: &ClassSection) -> bool { ... }
}
```

## Policy: OptionalSubjectEligibility

```rust
pub struct OptionalSubjectEligibility;

impl Policy<AssignOptionalSubjectCommand> for OptionalSubjectEligibility {
    type Outcome = Eligible | NotEligible { reason: &'static str };
    fn check(&self, ctx: &Context, cmd: &AssignOptionalSubjectCommand) -> Outcome { ... }
}
```

## Specification: ActiveStudentsInClass

```rust
pub struct ActiveStudentsInClass;

impl Specification<Student> for ActiveStudentsInClass {
    fn is_satisfied_by(&self, s: &Student) -> bool { ... }
}
```

Composed with `And`, `Or`, `Not` for queries.

## Specification: PromotableStudents

```rust
pub struct PromotableStudents;

impl Specification<Student> for PromotableStudents {
    fn is_satisfied_by(&self, s: &Student) -> bool { ... }
}
```

## Specification: HasOutstandingHomework

Used by communication to drive reminder notifications.

## Cross-Domain Coordinator

A thin coordinator lives in the engine facade and orchestrates
multi-domain flows (e.g. admission + library membership + fees
assignment). It is **not** a service; it composes command calls:

```rust
pub struct SchoolAdmissionCoordinator<'a> {
    engine: &'a Engine,
}

impl<'a> SchoolAdmissionCoordinator<'a> {
    pub async fn admit(&self, cmd: AdmitStudentCommand) -> Result<Student, DomainError> {
        let student = self.engine.students().admit(cmd.clone()).await?;
        // Subscribers (library, finance, communication) handle their
        // own side effects in response to the StudentAdmitted event.
        Ok(student)
    }
}
```

Domain services are pure. Cross-domain coordination happens through
events and command composition, never through service-to-service calls.
