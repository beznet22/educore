# Academic Domain — Value Objects

Value objects are immutable, validated at construction, and have no
identity. They are compared by value.

## Identifiers

All identifiers in the academic domain are typed and tenant-scoped. The
generic `Id<S, T>` wrapper carries the `SchoolId` of the owning school
and the local id (`Uuid`). Two `StudentId` values in different schools
are distinct types at the domain level and may be unified only through
explicit cross-tenant commands.

| Identifier                   | Backing Type    | Source Column                     |
| ---------------------------- | --------------- | --------------------------------- |
| `StudentId`                  | `Id<Student>`   | `academic_students.id`                  |
| `GuardianId`                 | `Id<Guardian>`  | `academic_parents.id`                   |
| `ClassId`                    | `Id<Class>`     | `academic_classes.id`                   |
| `SectionId`                  | `Id<Section>`   | `academic_sections.id`                  |
| `ClassSectionId`             | `Id<ClassSection>` | `academic_class_sections.id`         |
| `SubjectId`                  | `Id<Subject>`   | `academic_subjects.id`                  |
| `ClassSubjectId`             | `Id<ClassSubject>` | `academic_assign_subjects.id`        |
| `AcademicYearId`             | `Id<AcademicYear>` | `academic_academic_years.id`         |
| `ClassRoutineId`             | `Id<ClassRoutine>` | `academic_class_routines.id`         |
| `ClassRoutineUpdateId`       | `Id<...>`       | `academic_class_routine_updates.id`     |
| `ClassTimeId`                | `Id<ClassTime>` | `academic_class_times.id`               |
| `ClassRoomId`                | `Id<ClassRoom>` | `academic_class_rooms.id`               |
| `HomeworkId`                 | `Id<Homework>`  | `academic_homeworks.id`                 |
| `HomeworkSubmissionId`       | `Id<...>`       | `academic_homework_students.id`         |
| `LessonPlanId`               | `Id<LessonPlan>` | `lesson_planners.id`             |
| `LessonId`                   | `Id<Lesson>`    | `academic_lessons.id`                   |
| `LessonDetailId`             | `Id<LessonDetail>` | `academic_lesson_details.id`        |
| `LessonTopicId`              | `Id<LessonTopic>` | `academic_lesson_topics.id`          |
| `LessonTopicDetailId`        | `Id<LessonTopicDetail>` | `academic_lesson_topic_details.id` |
| `LessonPlanTopicId`          | `Id<...>`       | `lesson_plan_topics.id`           |
| `StudentRecordId`            | `Id<StudentRecord>` | `student_records.id`         |
| `StudentPromotionId`         | `Id<StudentPromotion>` | `academic_student_promotions.id` |
| `StudentCategoryId`          | `Id<StudentCategory>` | `academic_student_categories.id`   |
| `StudentGroupId`             | `Id<StudentGroup>` | `academic_student_groups.id`         |
| `StudentDocumentId`          | `Id<StudentDocument>` | `academic_student_documents.id`   |
| `StudentTimelineId`          | `Id<StudentTimeline>` | `academic_student_timelines.id`    |
| `StudentHomeworkId`          | `Id<StudentHomework>` | `academic_student_homeworks.id`   |
| `OptionalSubjectAssignmentId`| `Id<...>`       | `academic_optional_subject_assigns.id`  |
| `RegistrationFieldId`        | `Id<RegistrationField>` | `academic_student_registration_fields.id` |
| `CertificateId`              | `Id<Certificate>` | `academic_student_certificates.id`    |
| `IdCardId`                   | `Id<IdCard>`    | `academic_student_id_cards.id`          |
| `GraduateId`                 | `Id<Graduate>`  | `graduates.id`                    |
| `AdmissionQueryId`           | `Id<AdmissionQuery>` | `academic_admission_queries.id`   |
| `AdmissionQueryFollowupId`   | `Id<...>`       | `academic_admission_query_followups.id` |
| `AssignmentSubmissionId`     | `Id<...>`       | `academic_upload_homework_contents.id`  |

## Names

| Type                | Constraints                                                      |
| ------------------- | ---------------------------------------------------------------- |
| `PersonName`        | 1..200 chars, unicode letters and basic punctuation allowed     |
| `FullName`          | Computed from `PersonName` parts                                |
| `EmailAddress`      | RFC 5322 with length cap 200                                    |
| `PhoneNumber`       | E.164 format preferred; alternative national formats accepted    |
| `Address`           | 1..500 chars                                                     |
| `Occupation`        | 1..200 chars                                                     |

## Academic Identifiers

| Type                | Constraints                                                      |
| ------------------- | ---------------------------------------------------------------- |
| `AdmissionNumber`   | 1..50 chars, unique within school                                |
| `RollNumber`        | 1..50 chars (numeric preferred), unique within class-section-year |
| `SubjectCode`       | 1..50 chars, unique within school                                |
| `ClassName`         | 1..200 chars                                                     |
| `SectionName`       | 1..200 chars                                                     |
| `AcademicYearTitle` | 1..200 chars                                                     |

## Dates & Periods

| Type                  | Notes                                                       |
| --------------------- | ----------------------------------------------------------- |
| `AdmissionDate`       | `NaiveDate`                                                 |
| `DateOfBirth`         | `NaiveDate`, must result in age between 2 and 30            |
| `AcademicYearRange`   | `(start, end)`, start strictly before end                  |
| `Period`              | `Time` start, `Time` end                                    |
| `DayOfWeek`           | `Sat..Fri` (1..7 per ISO)                                   |
| `Semester`            | Optional; integer 1..3                                      |

## Status Enums

| Type                | Values                                                                                |
| ------------------- | ------------------------------------------------------------------------------------- |
| `StudentStatus`     | `Applicant`, `Active`, `Suspended`, `Withdrawn`, `Graduated`, `Transferred`           |
| `Gender`            | `Male`, `Female`, `Other`                                                             |
| `BloodGroup`        | `A+`, `A-`, `B+`, `B-`, `AB+`, `AB-`, `O+`, `O-`                                      |
| `SubjectType`       | `Theory`, `Practical`                                                                 |
| `ResultStatus`      | `Pass`, `Fail`, `Manual`                                                              |
| `LessonPlanStatus`  | `Pending`, `InProgress`, `Completed`, `Skipped`                                       |
| `ClassSectionStatus`| `Active`, `Archived`                                                                  |
| `HomeworkStatus`    | `Assigned`, `Submitted`, `Evaluated`, `Cancelled`                                     |
| `RegistrationType`  | `Student`, `Staff`                                                                    |
| `DocumentType`      | `Student`, `Staff`                                                                    |
| `CertificateLayout` | `Portrait`, `Landscape`                                                               |
| `CertificateType`   | `School`, `Course`                                                                    |

## Guardian

| Type                | Constraints                                                      |
| ------------------- | ---------------------------------------------------------------- |
| `GuardianRelation`  | `Father`, `Mother`, `Brother`, `Sister`, `Uncle`, `Aunt`, `Other`|
| `GuardianSpec`      | `full_name`, `relation`, `phone`, `email?`, `occupation?`        |
| `IsPrimary`         | `bool`                                                           |

## Money & Quantities (used in academic value objects)

| Type                 | Notes                                                       |
| -------------------- | ----------------------------------------------------------- |
| `PassMark`           | `f32` in `[0, 100]`                                          |
| `Gpa`                | `f32` in `[0, 5]`                                            |
| `Percentage`         | `f32` in `[0, 100]`                                          |
| `Marks`              | `f32` non-negative                                           |
| `Height`             | `f32` in cm                                                  |
| `Weight`             | `f32` in kg                                                  |
| `Age`                | `u8` in years                                                |

## School Identity Bindings

| Type                  | Notes                                                       |
| --------------------- | ----------------------------------------------------------- |
| `SchoolId`            | From `educore-platform`                                     |
| `TenantContext`       | `(SchoolId, UserId, ...)` from `educore-platform`           |
| `BankAccount`         | Used by finance; academic owns no money                     |

## Admission Inquiry

| Type                | Constraints                                                      |
| ------------------- | ---------------------------------------------------------------- |
| `AdmissionSource`   | 1..100 chars                                                     |
| `AdmissionReference`| 1..100 chars                                                     |
| `NoOfChild`         | `u8` in 1..20                                                     |
| `InquiryStatus`     | `New`, `InProgress`, `Converted`, `Closed`                       |

## Routine

| Type                  | Notes                                                       |
| --------------------- | ----------------------------------------------------------- |
| `Weekdays`            | `BTreeSet<DayOfWeek>`                                       |
| `RoutineSlot`         | `(DayOfWeek, PeriodIndex, SubjectId, TeacherId, RoomId)`    |
| `RoutineConstraint`   | `NoTeacherOverlap`, `NoRoomOverlap`                        |

## Validation Rules

All value objects implement `Validate` and refuse construction when
validation fails:

```rust
pub trait Validate {
    fn validate(&self) -> Result<(), ValueError>;
}
```

Construction is the only entry point:

```rust
let email = EmailAddress::parse("ada@example.com")?;
```

Parsing returns `Result<EmailAddress, ValueError>`. There are no setters
that bypass validation.

## Additional Identifiers

| Identifier | Backing Type | Notes |
| ---------- | ------------ | ----- |

## Additional Enums

| Type | Values |
| ---- | ------ |
| `OptionalSubjectGpaThreshold` | (status/classification enum, see code) |
| `SuspensionReason` | (status/classification enum, see code) |
| `TransferReason` | (status/classification enum, see code) |
| `WithdrawalReason` | (status/classification enum, see code) |

## Additional Identifiers

| Identifier | Backing Type | Notes |
| ---------- | ------------ | ----- |

## Additional Enums

| Type | Values |
| ---- | ------ |
| `OptionalSubjectGpaThreshold` | (status/classification enum, see code) |
| `SuspensionReason` | (status/classification enum, see code) |
| `TransferReason` | (status/classification enum, see code) |
| `WithdrawalReason` | (status/classification enum, see code) |
