# Academic Domain — Aggregates

## Student

**Root type:** `Student`
**Identity:** `StudentId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Academic

### Purpose

Represents a person enrolled in a school, including their identity, profile,
status, and links to the aggregates that organize their school life.

### Owned Children

- `StudentRecord` — one per `AcademicYear` (the canonical enrollment).
- `StudentCategory` membership (via reference).
- `StudentGroup` membership (via reference).
- `OptionalSubjectAssignment` (zero or more).
- `StudentDocument` (zero or more).
- `StudentTimeline` (zero or more; read-only history).
- `StudentHomework` (zero or more; per-subject homework).

### Invariants

1. A student has exactly one active `StudentRecord` per `AcademicYear`.
2. A student's `AdmissionNumber` is unique within a school.
3. A student's `RollNumber` is unique within `(school, class, section,
   academic_year)`.
4. A student can be in at most one optional subject per academic year.
5. A student's `Status` transitions are: `Applicant → Active →
   {Suspended, Withdrawn, Graduated, Transferred}`. No other transitions.
6. A withdrawn or graduated student has no active `StudentRecord`.

### Commands

- `AdmitStudent`
- `UpdateStudentProfile`
- `AssignStudentToSection`
- `ChangeStudentCategory`
- `AssignOptionalSubject`
- `UploadStudentDocument`
- `SuspendStudent`
- `ReinstateStudent`
- `WithdrawStudent`
- `TransferStudent`
- `PromoteStudent`
- `GraduateStudent`

### Events

- `StudentAdmitted`
- `StudentProfileUpdated`
- `StudentAssignedToSection`
- `StudentCategoryChanged`
- `OptionalSubjectAssigned`
- `StudentDocumentUploaded`
- `StudentSuspended`
- `StudentReinstated`
- `StudentWithdrawn`
- `StudentTransferred`
- `StudentPromoted`
- `StudentGraduated`

### Consistency Boundary

All student mutations are serialized through the `Student` aggregate root.
A student is loaded by id, mutated in memory, validated, and persisted with
its events in a single transaction.

---

## Guardian

**Root type:** `Guardian`
**Identity:** `GuardianId(SchoolId, Uuid)`

### Purpose

Represents a parent, legal guardian, or contact authorized to act on behalf
of a student.

### Invariants

1. A guardian has at most one phone and one email of record.
2. A guardian may be linked to multiple students.
3. A guardian link carries a `Relation` (Father, Mother, Guardian, Other)
   and a flag `IsPrimary` (used for communication routing).
4. There is at most one `IsPrimary` guardian per student.
5. A guardian is soft-deleted when all their student links are removed.

### Commands

- `RegisterGuardian`
- `UpdateGuardianContact`
- `LinkGuardianToStudent`
- `UnlinkGuardianFromStudent`
- `MarkPrimaryGuardian`

### Events

- `GuardianRegistered`
- `GuardianContactUpdated`
- `GuardianLinkedToStudent`
- `GuardianUnlinkedFromStudent`
- `PrimaryGuardianMarked`

---

## Class

**Root type:** `Class`
**Identity:** `ClassId(SchoolId, Uuid)`

### Purpose

A grade level offered by the school (e.g. "Grade 1", "Year 7", "Class 10").

### Invariants

1. A class belongs to exactly one school.
2. A class is uniquely named within a school.
3. A class may have an `OptionalSubjectGpaThreshold` that defines the
   minimum GPA required to take optional subjects.
4. A class cannot be deleted if any `ClassSection` references it.

### Commands

- `CreateClass`
- `UpdateClass`
- `SetOptionalSubjectGpaThreshold`
- `DeleteClass`

### Events

- `ClassCreated`
- `ClassUpdated`
- `OptionalSubjectGpaThresholdSet`
- `ClassDeleted`

---

## Section

**Root type:** `Section`
**Identity:** `SectionId(SchoolId, Uuid)`

### Purpose

A division of a class (e.g. "Section A", "Blue Group").

### Invariants

1. A section is uniquely named within a school.
2. A section can be reused across multiple `AcademicYear`s.
3. A section can be soft-deleted; existing references remain.

### Commands

- `CreateSection`
- `UpdateSection`
- `DeleteSection`

### Events

- `SectionCreated`
- `SectionUpdated`
- `SectionDeleted`

---

## ClassSection

**Root type:** `ClassSection`
**Identity:** `ClassSectionId(SchoolId, Uuid)`

### Purpose

A pairing of a class and a section in a specific academic year. This is
the unit that students are enrolled into and that class routines are
scheduled against.

### Invariants

1. A `ClassSection` is unique per `(class, section, academic_year)`.
2. A `ClassSection` may have multiple class teachers and subject teachers.
3. A `ClassSection` may have one or more class rooms assigned.
4. A `ClassSection` cannot be deleted while `StudentRecord`s reference it.

### Commands

- `CreateClassSection`
- `AssignClassTeacher`
- `AssignSubjectTeacher`
- `AssignClassRoom`
- `DeleteClassSection`

### Events

- `ClassSectionCreated`
- `ClassTeacherAssigned`
- `SubjectTeacherAssigned`
- `ClassRoomAssigned`
- `ClassSectionDeleted`

---

## Subject

**Root type:** `Subject`
**Identity:** `SubjectId(SchoolId, Uuid)`

### Purpose

A subject offered in a class (e.g. "Mathematics", "Physics").

### Invariants

1. A subject is uniquely identified by code within a school.
2. A subject has a `SubjectType` of `Theory` or `Practical`.
3. A subject's pass mark is configurable and used in assessment.

### Commands

- `CreateSubject`
- `UpdateSubject`
- `DeleteSubject`

### Events

- `SubjectCreated`
- `SubjectUpdated`
- `SubjectDeleted`

---

## ClassSubject

**Root type:** `ClassSubject`
**Identity:** `ClassSubjectId(SchoolId, Uuid)`

### Purpose

The assignment of a subject to a class (and possibly a section) within
an academic year, with a teacher.

### Invariants

1. A subject may be assigned to a class or to a class-section.
2. The same teacher may be assigned to multiple class-subjects.
3. The assignment has a `PassMark` that overrides the subject default if
   present.

### Commands

- `AssignSubjectToClass`
- `ReassignTeacher`
- `UnassignSubject`

### Events

- `SubjectAssignedToClass`
- `TeacherReassigned`
- `SubjectUnassigned`

---

## AcademicYear

**Root type:** `AcademicYear`
**Identity:** `AcademicYearId(SchoolId, Uuid)`

### Purpose

A school year (e.g. "2025-2026") with a defined start and end date.

### Invariants

1. An academic year's start date is strictly before its end date.
2. Academic years do not overlap within a school.
3. Exactly one academic year may be marked `Current` per school.
4. A non-current academic year may be opened for read-only queries.
5. Promoting a student requires both a `From` and `To` academic year, both
   in the same school, and the `To` year must be the next sequential year.

### Commands

- `CreateAcademicYear`
- `UpdateAcademicYearDates`
- `SetCurrentAcademicYear`
- `CloseAcademicYear`
- `CopyAcademicYear` (deep copy classes, sections, subjects into a new year)

### Events

- `AcademicYearCreated`
- `AcademicYearDatesUpdated`
- `CurrentAcademicYearSet`
- `AcademicYearClosed`
- `AcademicYearCopied`

---

## ClassRoutine

**Root type:** `ClassRoutine`
**Identity:** `ClassRoutineId(SchoolId, Uuid)`

### Purpose

The weekly schedule for a class-section-subject combination. Periods are
defined via `ClassTime` slots.

### Invariants

1. A class routine covers a full week.
2. Periods may be class periods or breaks; a period is identified by
   `ClassTimeId`.
3. A room and a teacher are assigned per period per day.
4. A teacher cannot be in two places at the same time.
5. A room cannot host two classes at the same time.

### Commands

- `CreateClassRoutine`
- `UpdateClassRoutinePeriod`
- `SwapClassRoutinePeriods`
- `DeleteClassRoutine`

### Events

- `ClassRoutineCreated`
- `ClassRoutinePeriodUpdated`
- `ClassRoutinePeriodsSwapped`
- `ClassRoutineDeleted`

---

## Homework

**Root type:** `Homework`
**Identity:** `HomeworkId(SchoolId, Uuid)`

### Purpose

An assignment given to students in a class-section, for a subject, with a
submission deadline.

### Invariants

1. A homework is created by a teacher and assigned to a class-section.
2. Submission date is after homework date.
3. Evaluation date is on or after submission date.
4. A homework may have an attached file.
5. A homework may be evaluated; once evaluated, the marks are immutable
   per student.

### Commands

- `CreateHomework`
- `UpdateHomework`
- `SubmitHomework` (student)
- `EvaluateHomework` (teacher)
- `CancelHomework`

### Events

- `HomeworkCreated`
- `HomeworkUpdated`
- `HomeworkSubmitted`
- `HomeworkEvaluated`
- `HomeworkCancelled`

---

## LessonPlan

**Root type:** `LessonPlan`
**Identity:** `LessonPlanId(SchoolId, Uuid)`

### Purpose

A teacher's plan for a specific lesson topic on a specific date, with
teaching method, objectives, and materials.

### Invariants

1. A lesson plan is anchored to a `Lesson`, a topic, a class-section, a
   subject, and a date.
2. A lesson plan may include sub-topics.
3. A lesson plan has a `CompletedStatus` (Pending, InProgress, Completed,
   Skipped).
4. Multiple teachers may share lesson plan templates but each scheduled
   occurrence is owned by one teacher.

### Commands

- `CreateLessonPlan`
- `UpdateLessonPlan`
- `MarkLessonPlanCompleted`
- `AddSubTopic`
- `DeleteLessonPlan`

### Events

- `LessonPlanCreated`
- `LessonPlanUpdated`
- `LessonPlanCompleted`
- `SubTopicAdded`
- `LessonPlanDeleted`

---

## Lesson

**Root type:** `Lesson`
**Identity:** `LessonId(SchoolId, Uuid)`

### Purpose

A unit of study within a subject, owned by a class-section.

### Invariants

1. A lesson is uniquely identified by title within a class-section-subject.
2. A lesson has zero or more topics.
3. A lesson has a creation user and a creation timestamp.

### Commands

- `CreateLesson`
- `UpdateLesson`
- `DeleteLesson`

### Events

- `LessonCreated`
- `LessonUpdated`
- `LessonDeleted`

---

## LessonTopic

**Root type:** `LessonTopic`
**Identity:** `LessonTopicId(SchoolId, Uuid)`

### Purpose

A topic within a lesson, trackable through a syllabus.

### Invariants

1. A topic belongs to one lesson.
2. A topic has a `CompletedStatus` and a `CompletedDate` if completed.

### Commands

- `CreateLessonTopic`
- `MarkTopicCompleted`
- `DeleteLessonTopic`

### Events

- `LessonTopicCreated`
- `LessonTopicCompleted`
- `LessonTopicDeleted`

---

## StudentRecord

**Root type:** `StudentRecord`
**Identity:** `StudentRecordId(SchoolId, Uuid)`

### Purpose

A student's enrollment in a specific `(class, section, academic_year)`.
The student has one `StudentRecord` per academic year they are enrolled.

### Invariants

1. A student has at most one non-graduate, non-withdrawn `StudentRecord`
   per academic year.
2. The `RollNumber` is unique within `(class, section, academic_year)`.
3. A `StudentRecord` may be `IsDefault` (current default) per student.
4. A `StudentRecord` is `IsPromote=false` until a `StudentPromoted` event
   closes it.
5. A `StudentRecord` is `IsGraduate=true` when the student graduates.
6. A `StudentRecord` may carry an `AdmissionNumber` (carried over from
   admission) and may be assigned a new one on promotion.

### Commands

- `EnrollStudent` (typically produced by `AdmitStudent` or `PromoteStudent`)
- `SetRollNumber`
- `SetDefaultRecord`
- `MarkGraduate`

### Events

- `StudentRecordCreated`
- `RollNumberAssigned`
- `DefaultRecordSet`
- `StudentMarkedGraduate`

---

## StudentPromotion

**Root type:** `StudentPromotion`
**Identity:** `StudentPromotionId(SchoolId, Uuid)`

### Purpose

A historical record of a promotion event, capturing the previous and new
class, section, session, roll number, and result status.

### Invariants

1. A `StudentPromotion` references both `From` and `To` `StudentRecord`s.
2. A `ResultStatus` is `Pass`, `Fail`, or `Manual` (operator decision).
3. A `StudentPromotion` is immutable once written.

### Commands

- (None — promotion is produced by the `PromoteStudent` command, which
  creates the `StudentRecord` for the new year and a `StudentPromotion`
  record.)

### Events

- `StudentPromoted` (carries the full promotion payload)

---

## StudentCategory

**Root type:** `StudentCategory`
**Identity:** `StudentCategoryId(SchoolId, Uuid)`

### Purpose

A categorization for students (e.g. "Scholarship", "Sibling", "Staff
Child"). Used for fee discounts and reporting.

### Invariants

1. A category is uniquely named within a school.

### Commands

- `CreateStudentCategory`
- `UpdateStudentCategory`
- `DeleteStudentCategory`

### Events

- `StudentCategoryCreated`
- `StudentCategoryUpdated`
- `StudentCategoryDeleted`

---

## StudentGroup

**Root type:** `StudentGroup`
**Identity:** `StudentGroupId(SchoolId, Uuid)`

### Purpose

A grouping of students for non-academic purposes (e.g. "Sports Team",
"Chess Club").

### Invariants

1. A group is uniquely named within a school.
2. A student can be in many groups.

### Commands

- `CreateStudentGroup`
- `UpdateStudentGroup`
- `AddStudentToGroup`
- `RemoveStudentFromGroup`
- `DeleteStudentGroup`

### Events

- `StudentGroupCreated`
- `StudentGroupUpdated`
- `StudentAddedToGroup`
- `StudentRemovedFromGroup`
- `StudentGroupDeleted`

---

## RegistrationField

**Root type:** `RegistrationField`
**Identity:** `RegistrationFieldId(SchoolId, Uuid)`

### Purpose

A custom field on the student or staff registration form.

### Invariants

1. A field has a `FieldName`, `LabelName`, and `Type` (Student or Staff).
2. A field has `IsRequired`, `IsVisible`, and editability flags.
3. A field has an `AdminSection` for placement on the form.

### Commands

- `CreateRegistrationField`
- `UpdateRegistrationField`
- `DeleteRegistrationField`

### Events

- `RegistrationFieldCreated`
- `RegistrationFieldUpdated`
- `RegistrationFieldDeleted`

---

## Certificate

**Root type:** `Certificate`
**Identity:** `CertificateId(SchoolId, Uuid)`

### Purpose

A configurable certificate template (transfer, character, course
completion, etc.).

### Invariants

1. A certificate has a layout (Portrait or Landscape), a body template,
   a footer with up to three labels, and an optional photo flag.
2. A certificate may have an attached file (PDF or image template).
3. A certificate has a `DefaultFor` flag for course certificates.

### Commands

- `CreateCertificate`
- `UpdateCertificate`
- `DeleteCertificate`

### Events

- `CertificateCreated`
- `CertificateUpdated`
- `CertificateDeleted`

---

## IdCard

**Root type:** `IdCard`
**Identity:** `IdCardId(SchoolId, Uuid)`

### Purpose

A configurable student ID card template.

### Invariants

1. A template has boolean flags for which fields to display
   (admission number, name, class, photo, etc.).
2. A template has layout dimensions and spacing parameters.

### Commands

- `CreateIdCard`
- `UpdateIdCard`
- `DeleteIdCard`

### Events

- `IdCardCreated`
- `IdCardUpdated`
- `IdCardDeleted`
