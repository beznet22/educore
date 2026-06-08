# Academic Domain — Events

Domain events describe facts that have already happened. They are
immutable, append-only records used for cross-domain integration, audit,
and event sourcing.

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

## Student Lifecycle

### StudentAdmitted

```rust
pub struct StudentAdmitted {
    pub student_id: StudentId,
    pub admission_no: AdmissionNumber,
    pub full_name: FullName,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub academic_year_id: AcademicYearId,
    pub guardian_ids: Vec<GuardianId>,
    pub admission_date: NaiveDate,
}
```

**Subscribers:**
- `finance` — create fees assignment.
- `library` — create library member.
- `communication` — send welcome message to guardians.
- `transport` — if transport was assigned, set up route expectation.

### StudentProfileUpdated

```rust
pub struct StudentProfileUpdated {
    pub student_id: StudentId,
    pub changed_fields: Vec<&'static str>,
}
```

### StudentAssignedToSection

```rust
pub struct StudentAssignedToSection {
    pub student_id: StudentId,
    pub from_section_id: SectionId,
    pub to_section_id: SectionId,
    pub reason: AssignmentReason,
}
```

### StudentCategoryChanged

```rust
pub struct StudentCategoryChanged {
    pub student_id: StudentId,
    pub from_category_id: Option<StudentCategoryId>,
    pub to_category_id: StudentCategoryId,
    pub effective_from: NaiveDate,
}
```

### OptionalSubjectAssigned

```rust
pub struct OptionalSubjectAssigned {
    pub student_id: StudentId,
    pub subject_id: SubjectId,
    pub academic_year_id: AcademicYearId,
}
```

### StudentDocumentUploaded

```rust
pub struct StudentDocumentUploaded {
    pub student_id: StudentId,
    pub document_id: StudentDocumentId,
    pub title: String,
    pub file: FileReference,
}
```

### StudentSuspended

```rust
pub struct StudentSuspended {
    pub student_id: StudentId,
    pub reason: SuspensionReason,
    pub effective_from: NaiveDate,
    pub expected_return: Option<NaiveDate>,
}
```

### StudentReinstated

```rust
pub struct StudentReinstated {
    pub student_id: StudentId,
    pub effective_from: NaiveDate,
}
```

### StudentWithdrawn

```rust
pub struct StudentWithdrawn {
    pub student_id: StudentId,
    pub reason: WithdrawalReason,
    pub effective_from: NaiveDate,
}
```

**Subscribers:**
- `finance` — finalizes outstanding balances.
- `library` — flags outstanding books for return.
- `transport` — removes student from route.

### StudentTransferred

```rust
pub struct StudentTransferred {
    pub student_id: StudentId,
    pub destination_school_id: SchoolId,
    pub reason: TransferReason,
    pub effective_from: NaiveDate,
}
```

### StudentPromoted

```rust
pub struct StudentPromoted {
    pub student_id: StudentId,
    pub from_record_id: StudentRecordId,
    pub to_record_id: StudentRecordId,
    pub from_class_id: ClassId,
    pub to_class_id: ClassId,
    pub from_section_id: SectionId,
    pub to_section_id: SectionId,
    pub from_academic_year_id: AcademicYearId,
    pub to_academic_year_id: AcademicYearId,
    pub from_roll_no: RollNumber,
    pub to_roll_no: RollNumber,
    pub result_status: ResultStatus,
    pub promotion_id: StudentPromotionId,
}
```

**Subscribers:**
- `finance` — rolls over balances and assigns new fees master.
- `attendance` — resets daily expectation for the new class.
- `assessment` — archives prior marks and prepares new exam schedules.

### StudentGraduated

```rust
pub struct StudentGraduated {
    pub student_id: StudentId,
    pub academic_year_id: AcademicYearId,
    pub graduation_date: NaiveDate,
}
```

## Guardian Lifecycle

- `GuardianRegistered`
- `GuardianContactUpdated`
- `GuardianLinkedToStudent { guardian_id, student_id, relation, is_primary }`
- `GuardianUnlinkedFromStudent { guardian_id, student_id }`
- `PrimaryGuardianMarked { guardian_id, student_id }`

## Class & Section

- `ClassCreated { class_id, class_name, pass_mark }`
- `ClassUpdated { class_id, changes }`
- `OptionalSubjectGpaThresholdSet { class_id, threshold }`
- `ClassDeleted { class_id }`
- `SectionCreated { section_id, section_name }`
- `SectionUpdated { section_id, changes }`
- `SectionDeleted { section_id }`

## ClassSection

- `ClassSectionCreated { class_section_id, class_id, section_id, academic_year_id }`
- `ClassTeacherAssigned { class_section_id, staff_id, role }`
- `SubjectTeacherAssigned { class_section_id, subject_id, staff_id }`
- `ClassRoomAssigned { class_section_id, room_id }`
- `ClassSectionDeleted { class_section_id }`

## Subject

- `SubjectCreated { subject_id, code, name, type, pass_mark }`
- `SubjectUpdated { subject_id, changes }`
- `SubjectDeleted { subject_id }`
- `SubjectAssignedToClass { class_subject_id, class_id, section_id?, subject_id, teacher_id? }`
- `TeacherReassigned { class_subject_id, from_teacher_id, to_teacher_id }`
- `SubjectUnassigned { class_subject_id }`

## AcademicYear

- `AcademicYearCreated { academic_year_id, year, title, range }`
- `AcademicYearDatesUpdated { academic_year_id, from, to }`
- `CurrentAcademicYearSet { academic_year_id, previous_id? }`
- `AcademicYearClosed { academic_year_id }`
- `AcademicYearCopied { to_academic_year_id, from_academic_year_id }`

## ClassRoutine

- `ClassRoutineCreated { class_routine_id, class_id, section_id, subject_id }`
- `ClassRoutinePeriodUpdated { class_routine_id, day, period }`
- `ClassRoutinePeriodsSwapped { class_routine_id, a, b }`
- `ClassRoutineDeleted { class_routine_id }`

## Homework

- `HomeworkCreated { homework_id, class_id, section_id, subject_id, due_date }`
- `HomeworkUpdated { homework_id, changes }`
- `HomeworkSubmitted { homework_id, student_id }`
- `HomeworkEvaluated { homework_id, student_id, marks }`
- `HomeworkCancelled { homework_id, reason }`

## Lesson

- `LessonCreated { lesson_id, class_id, section_id, subject_id }`
- `LessonUpdated { lesson_id, changes }`
- `LessonDeleted { lesson_id }`
- `LessonTopicCreated { lesson_topic_id, lesson_id, title }`
- `LessonTopicCompleted { lesson_topic_id, completed_date }`
- `LessonTopicDeleted { lesson_topic_id }`
- `LessonPlanCreated { lesson_plan_id, lesson_id, topic_id, date }`
- `LessonPlanUpdated { lesson_plan_id, changes }`
- `LessonPlanCompleted { lesson_plan_id, completed_date }`
- `SubTopicAdded { lesson_plan_id, sub_topic_id, title }`
- `LessonPlanDeleted { lesson_plan_id }`

## StudentRecord

- `StudentRecordCreated { student_record_id, student_id, class_id, section_id, academic_year_id }`
- `RollNumberAssigned { student_record_id, roll_no }`
- `DefaultRecordSet { student_record_id, student_id }`
- `StudentMarkedGraduate { student_record_id, student_id }`

## StudentCategory

- `StudentCategoryCreated { id, name }`
- `StudentCategoryUpdated { id, changes }`
- `StudentCategoryDeleted { id }`

## StudentGroup

- `StudentGroupCreated { id, name }`
- `StudentGroupUpdated { id, changes }`
- `StudentAddedToGroup { group_id, student_id }`
- `StudentRemovedFromGroup { group_id, student_id }`
- `StudentGroupDeleted { id }`

## Registration

- `RegistrationFieldCreated { id, field_name, type }`
- `RegistrationFieldUpdated { id, changes }`
- `RegistrationFieldDeleted { id }`

## Certificate & ID Card

- `CertificateCreated { id, name, type }`
- `CertificateUpdated { id, changes }`
- `CertificateDeleted { id }`
- `IdCardCreated { id, title }`
- `IdCardUpdated { id, changes }`
- `IdCardDeleted { id }`

## Admission Query

- `AdmissionQueryRegistered { id, name, class_id }`
- `AdmissionQueryFollowedUp { id, followup_id, response }`
- `AdmissionQueryConverted { id, student_id }`
- `AdmissionQueryClosed { id, reason }`
