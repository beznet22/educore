# Academic Domain — Commands

Commands describe intent. They are validated, authorized, and dispatched
to the relevant aggregate. Every command produces zero or more events
that are recorded in the event log.

All commands carry a `TenantContext` (school + actor + correlation) and
are rejected if the actor lacks the required capability.

## AdmitStudent

```rust
pub struct AdmitStudentCommand {
    pub tenant: TenantContext,
    pub admission_no: AdmissionNumber,
    pub first_name: PersonName,
    pub last_name: PersonName,
    pub date_of_birth: DateOfBirth,
    pub gender: Gender,
    pub blood_group: Option<BloodGroup>,
    pub religion: Option<String>,
    pub caste: Option<String>,
    pub mobile: Option<PhoneNumber>,
    pub email: Option<EmailAddress>,
    pub current_address: Option<Address>,
    pub permanent_address: Option<Address>,
    pub admission_date: AdmissionDate,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub academic_year_id: AcademicYearId,
    pub roll_no: Option<RollNumber>,
    pub student_category_id: Option<StudentCategoryId>,
    pub student_group_ids: Vec<StudentGroupId>,
    pub guardians: Vec<GuardianSpec>,
    pub transport: Option<TransportSpec>,
    pub hostel: Option<HostelSpec>,
    pub documents: Vec<DocumentSpec>,
    pub custom_fields: BTreeMap<String, String>,
}
```

**Capability:** `Student.Admit`
**Pre-conditions:**
- Admission number unique in school.
- Class, section, and academic year exist.
- Academic year is current.
- At least one guardian is supplied; exactly one is primary.
- Class has free capacity (if capacity is configured).

**Effects:** Creates a `Student`, one `StudentRecord` for the academic
year, guardian links, optional transport/hostel references, and emits
`StudentAdmitted` and `StudentRecordCreated`.

## UpdateStudentProfile

```rust
pub struct UpdateStudentProfileCommand {
    pub tenant: TenantContext,
    pub student_id: StudentId,
    pub patch: StudentProfilePatch,
}
```

`StudentProfilePatch` is a partial update containing only mutable
fields: `first_name`, `last_name`, `gender`, `mobile`, `email`,
addresses, custom fields. Immutable fields (admission number, school id,
academic year) cannot be patched here — use `TransferStudent` or
`PromoteStudent`.

**Capability:** `Student.Update`
**Effects:** Emits `StudentProfileUpdated`.

## AssignStudentToSection

```rust
pub struct AssignStudentToSectionCommand {
    pub tenant: TenantContext,
    pub student_id: StudentId,
    pub section_id: SectionId,
    pub reason: AssignmentReason,
}
```

**Capability:** `Student.AssignSection`
**Pre-conditions:** Student has an active `StudentRecord` in the current
academic year. Target section exists. Section belongs to the student's
class.

**Effects:** Emits `StudentAssignedToSection`.

## ChangeStudentCategory

```rust
pub struct ChangeStudentCategoryCommand {
    pub tenant: TenantContext,
    pub student_id: StudentId,
    pub category_id: StudentCategoryId,
    pub effective_from: NaiveDate,
}
```

**Capability:** `Student.Update`
**Effects:** Emits `StudentCategoryChanged`. Finance domain
recomputes applicable discounts.

## AssignOptionalSubject

```rust
pub struct AssignOptionalSubjectCommand {
    pub tenant: TenantContext,
    pub student_id: StudentId,
    pub subject_id: SubjectId,
    pub academic_year_id: AcademicYearId,
}
```

**Capability:** `Student.Update`
**Pre-conditions:**
- Student's GPA (from latest published result) is at or above the
  class's `OptionalSubjectGpaThreshold`.
- Subject is configured as optional in the class.
- Student has not already been assigned an optional subject for the
  academic year.

**Effects:** Emits `OptionalSubjectAssigned`.

## UploadStudentDocument

```rust
pub struct UploadStudentDocumentCommand {
    pub tenant: TenantContext,
    pub student_id: StudentId,
    pub title: String,
    pub file: FileReference,
    pub document_type: DocumentType,
}
```

**Capability:** `Student.Document.Upload`
**Effects:** Emits `StudentDocumentUploaded` after the file storage
port stores the file.

## SuspendStudent

```rust
pub struct SuspendStudentCommand {
    pub tenant: TenantContext,
    pub student_id: StudentId,
    pub reason: SuspensionReason,
    pub effective_from: NaiveDate,
    pub expected_return: Option<NaiveDate>,
}
```

**Capability:** `Student.Suspend`
**Effects:** Emits `StudentSuspended`. Attendance marks are blocked.
Fee invoices continue to accrue unless waived.

## ReinstateStudent

```rust
pub struct ReinstateStudentCommand {
    pub tenant: TenantContext,
    pub student_id: StudentId,
    pub effective_from: NaiveDate,
    pub note: Option<String>,
}
```

**Capability:** `Student.Reinstate`
**Pre-conditions:** Student is currently suspended.

**Effects:** Emits `StudentReinstated`.

## WithdrawStudent

```rust
pub struct WithdrawStudentCommand {
    pub tenant: TenantContext,
    pub student_id: StudentId,
    pub reason: WithdrawalReason,
    pub effective_from: NaiveDate,
    pub note: Option<String>,
}
```

**Capability:** `Student.Withdraw`
**Effects:** Closes the active `StudentRecord`, marks the student
`Withdrawn`, and emits `StudentWithdrawn`. Library, transport, and
finance domains subscribe to finalize balances and remove references.

## TransferStudent

```rust
pub struct TransferStudentCommand {
    pub tenant: TenantContext,
    pub student_id: StudentId,
    pub destination_school_id: SchoolId, // must be a sibling school in same SaaS tenant
    pub reason: TransferReason,
    pub effective_from: NaiveDate,
}
```

**Capability:** `Student.Transfer`
**Effects:** Closes the active `StudentRecord`, marks the student
`Transferred`, and emits `StudentTransferred` with a destination school
id. The destination school issues its own `AdmitStudent` from the
transfer payload.

## PromoteStudent

```rust
pub struct PromoteStudentCommand {
    pub tenant: TenantContext,
    pub student_id: StudentId,
    pub from_academic_year_id: AcademicYearId,
    pub to_academic_year_id: AcademicYearId,
    pub to_class_id: ClassId,
    pub to_section_id: SectionId,
    pub to_roll_no: RollNumber,
    pub result_status: ResultStatus,
}
```

**Capability:** `Student.Promote`
**Pre-conditions:**
- Source academic year is the previous one relative to target.
- Student has an active `StudentRecord` in the source year.
- Target class-section exists in the target year.
- `to_roll_no` is unique in the target class-section.

**Effects:** Closes the source `StudentRecord`, creates a new
`StudentRecord` in the target year, writes a `StudentPromotion` record,
emits `StudentPromoted`.

## GraduateStudent

```rust
pub struct GraduateStudentCommand {
    pub tenant: TenantContext,
    pub student_id: StudentId,
    pub academic_year_id: AcademicYearId,
    pub graduation_date: NaiveDate,
    pub destination: Option<GraduateDestination>,
}
```

**Capability:** `Student.Graduate`
**Effects:** Emits `StudentGraduated` and `StudentMarkedGraduate`.

## CreateClass / UpdateClass / DeleteClass

Standard CRUD commands on the `Class` aggregate.

```rust
pub struct CreateClassCommand {
    pub tenant: TenantContext,
    pub class_name: ClassName,
    pub pass_mark: PassMark,
}

pub struct UpdateClassCommand { ... }
pub struct DeleteClassCommand { ... }
```

**Capabilities:** `Class.Create`, `Class.Update`, `Class.Delete`.

## CreateSection / UpdateSection / DeleteSection

```rust
pub struct CreateSectionCommand { ... }
pub struct UpdateSectionCommand { ... }
pub struct DeleteSectionCommand { ... }
```

**Capabilities:** `Section.Create`, `Section.Update`, `Section.Delete`.

## CreateClassSection

```rust
pub struct CreateClassSectionCommand {
    pub tenant: TenantContext,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub academic_year_id: AcademicYearId,
    pub default_room_id: Option<ClassRoomId>,
}
```

**Capability:** `ClassSection.Create`
**Pre-conditions:** Combination of (class, section, academic_year) is
unique.

**Effects:** Emits `ClassSectionCreated`.

## AssignClassTeacher / AssignSubjectTeacher / AssignClassRoom

```rust
pub struct AssignClassTeacherCommand {
    pub tenant: TenantContext,
    pub class_section_id: ClassSectionId,
    pub staff_id: StaffId,
    pub role: TeacherRole,
}

pub struct AssignSubjectTeacherCommand {
    pub tenant: TenantContext,
    pub class_section_id: ClassSectionId,
    pub subject_id: SubjectId,
    pub staff_id: StaffId,
}

pub struct AssignClassRoomCommand {
    pub tenant: TenantContext,
    pub class_section_id: ClassSectionId,
    pub room_id: ClassRoomId,
}
```

**Capabilities:** `ClassSection.AssignTeacher`,
`ClassSection.AssignRoom`.

**Effects:** Emit `ClassTeacherAssigned`, `SubjectTeacherAssigned`,
`ClassRoomAssigned`.

## CreateSubject / UpdateSubject / DeleteSubject

```rust
pub struct CreateSubjectCommand {
    pub tenant: TenantContext,
    pub subject_name: String,
    pub subject_code: SubjectCode,
    pub subject_type: SubjectType,
    pub pass_mark: PassMark,
}
```

**Capabilities:** `Subject.Create`, `Subject.Update`, `Subject.Delete`.

## AssignSubjectToClass / ReassignTeacher / UnassignSubject

```rust
pub struct AssignSubjectToClassCommand {
    pub tenant: TenantContext,
    pub class_id: ClassId,
    pub section_id: Option<SectionId>,
    pub subject_id: SubjectId,
    pub teacher_id: Option<StaffId>,
    pub pass_mark_override: Option<PassMark>,
}
```

**Capabilities:** `ClassSubject.Assign`, `ClassSubject.Reassign`,
`ClassSubject.Unassign`.

## CreateAcademicYear / UpdateAcademicYearDates / SetCurrentAcademicYear / CloseAcademicYear

```rust
pub struct CreateAcademicYearCommand {
    pub tenant: TenantContext,
    pub year: String,         // e.g. "2026"
    pub title: String,        // e.g. "Academic Year 2026-2027"
    pub starting_date: NaiveDate,
    pub ending_date: NaiveDate,
    pub copy_with_academic_year: Option<AcademicYearId>, // seed from existing
}
```

**Capabilities:** `AcademicYear.Create`, `AcademicYear.Update`,
`AcademicYear.SetCurrent`, `AcademicYear.Close`.

`SetCurrentAcademicYear` is the only command that mutates the `Current`
flag. It demotes the previous current year.

`CopyAcademicYear` (typed as `CreateAcademicYearCommand` with
`copy_with_academic_year`) is an idempotent deep copy: classes,
sections, class-sections, subjects, class-subjects, and routines are
copied from the source year. It does not copy students.

## CreateClassRoutine / UpdateClassRoutinePeriod / SwapClassRoutinePeriods / DeleteClassRoutine

```rust
pub struct CreateClassRoutineCommand {
    pub tenant: TenantContext,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub subject_id: SubjectId,
    pub academic_year_id: AcademicYearId,
    pub slots: Vec<RoutineSlot>,
}
```

**Capabilities:** `ClassRoutine.Create`, `ClassRoutine.Update`,
`ClassRoutine.Swap`, `ClassRoutine.Delete`.

`SwapClassRoutinePeriods` swaps two slots in one transaction.

## CreateHomework / UpdateHomework / SubmitHomework / EvaluateHomework / CancelHomework

```rust
pub struct CreateHomeworkCommand {
    pub tenant: TenantContext,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub subject_id: SubjectId,
    pub homework_date: NaiveDate,
    pub submission_date: NaiveDate,
    pub description: String,
    pub marks: Option<Marks>,
    pub file: Option<FileReference>,
}

pub struct SubmitHomeworkCommand {
    pub tenant: TenantContext,
    pub homework_id: HomeworkId,
    pub student_id: StudentId,
    pub description: Option<String>,
    pub file: Option<FileReference>,
}

pub struct EvaluateHomeworkCommand {
    pub tenant: TenantContext,
    pub homework_id: HomeworkId,
    pub student_id: StudentId,
    pub marks: Marks,
    pub teacher_comments: Option<String>,
}
```

**Capabilities:** `Homework.Create`, `Homework.Update`,
`Homework.Submit`, `Homework.Evaluate`, `Homework.Cancel`.

## CreateLessonPlan / UpdateLessonPlan / MarkLessonPlanCompleted / AddSubTopic / DeleteLessonPlan

```rust
pub struct CreateLessonPlanCommand {
    pub tenant: TenantContext,
    pub lesson_id: LessonId,
    pub topic_id: LessonTopicId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub subject_id: SubjectId,
    pub lesson_date: NaiveDate,
    pub sub_topic: Option<String>,
    pub lecture_youtube_link: Option<Url>,
    pub lecture_video: Option<FileReference>,
    pub attachment: Option<FileReference>,
    pub teaching_method: Option<String>,
    pub general_objectives: Option<String>,
    pub previous_knowledge: Option<String>,
    pub comp_question: Option<String>,
    pub note: Option<String>,
}
```

**Capabilities:** `LessonPlan.Create`, `LessonPlan.Update`,
`LessonPlan.Complete`, `LessonPlan.AddSubTopic`, `LessonPlan.Delete`.

## CreateLesson / UpdateLesson / DeleteLesson

```rust
pub struct CreateLessonCommand {
    pub tenant: TenantContext,
    pub lesson_title: String,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub subject_id: SubjectId,
    pub topics: Vec<LessonTopicSpec>,
}
```

**Capabilities:** `Lesson.Create`, `Lesson.Update`, `Lesson.Delete`.

## CreateLessonTopic / MarkTopicCompleted / DeleteLessonTopic

```rust
pub struct CreateLessonTopicCommand {
    pub tenant: TenantContext,
    pub lesson_id: LessonId,
    pub topic_title: String,
}
```

**Capabilities:** `LessonTopic.Create`, `LessonTopic.Complete`,
`LessonTopic.Delete`.

## CreateStudentCategory / UpdateStudentCategory / DeleteStudentCategory

```rust
pub struct CreateStudentCategoryCommand {
    pub tenant: TenantContext,
    pub category_name: String,
}
```

**Capabilities:** `StudentCategory.Create`, `StudentCategory.Update`,
`StudentCategory.Delete`.

## CreateStudentGroup / UpdateStudentGroup / AddStudentToGroup / RemoveStudentFromGroup / DeleteStudentGroup

```rust
pub struct CreateStudentGroupCommand { ... }
pub struct AddStudentToGroupCommand {
    pub tenant: TenantContext,
    pub group_id: StudentGroupId,
    pub student_id: StudentId,
}
```

**Capabilities:** `StudentGroup.Create`, `StudentGroup.Update`,
`StudentGroup.AddStudent`, `StudentGroup.RemoveStudent`,
`StudentGroup.Delete`.

## CreateRegistrationField / UpdateRegistrationField / DeleteRegistrationField

```rust
pub struct CreateRegistrationFieldCommand {
    pub tenant: TenantContext,
    pub field_name: String,
    pub label_name: String,
    pub registration_type: RegistrationType, // student or staff
    pub is_required: bool,
    pub is_show: bool,
    pub student_edit: bool,
    pub parent_edit: bool,
    pub staff_edit: bool,
    pub admin_section: AdminSection,
    pub position: i32,
}
```

**Capabilities:** `RegistrationField.Create`, `RegistrationField.Update`,
`RegistrationField.Delete`.

## CreateCertificate / UpdateCertificate / DeleteCertificate

```rust
pub struct CreateCertificateCommand {
    pub tenant: TenantContext,
    pub name: String,
    pub header_left_text: Option<String>,
    pub body: String,
    pub body_two: Option<String>,
    pub certificate_no: Option<String>,
    pub certificate_type: CertificateType,
    pub footer_left_text: Option<String>,
    pub footer_center_text: Option<String>,
    pub footer_right_text: Option<String>,
    pub student_photo: bool,
    pub file: Option<FileReference>,
    pub layout: CertificateLayout,
    pub body_font_family: String,
    pub body_font_size: String,
    pub height_mm: u32,
    pub width_mm: u32,
    pub default_for: Option<CourseId>,
}
```

**Capabilities:** `Certificate.Create`, `Certificate.Update`,
`Certificate.Delete`.

## CreateIdCard / UpdateIdCard / DeleteIdCard

```rust
pub struct CreateIdCardCommand {
    pub tenant: TenantContext,
    pub title: String,
    pub logo: Option<FileReference>,
    pub signature: Option<FileReference>,
    pub background_img: Option<FileReference>,
    pub page_layout_style: String,
    pub user_photo_style: String,
    pub user_photo_width_mm: u32,
    pub user_photo_height_mm: u32,
    pub page_width_mm: u32,
    pub page_height_mm: u32,
    pub top_space_mm: u32,
    pub bottom_space_mm: u32,
    pub right_space_mm: u32,
    pub left_space_mm: u32,
    pub show: IdCardFieldFlags,
}
```

`IdCardFieldFlags` is a `bitflags` struct: admission_no, student_name,
class, father_name, mother_name, student_address, phone_number, dob,
blood.

**Capabilities:** `IdCard.Create`, `IdCard.Update`, `IdCard.Delete`.

## RegisterAdmissionQuery / FollowUpAdmissionQuery / ConvertAdmissionQuery

```rust
pub struct RegisterAdmissionQueryCommand {
    pub tenant: TenantContext,
    pub name: String,
    pub phone: Option<PhoneNumber>,
    pub email: Option<EmailAddress>,
    pub address: Option<Address>,
    pub description: Option<String>,
    pub date: NaiveDate,
    pub class_id: ClassId,
    pub source: Option<String>,
    pub reference: Option<String>,
    pub no_of_child: u8,
}
```

**Capabilities:** `AdmissionQuery.Create`, `AdmissionQuery.FollowUp`,
`AdmissionQuery.Convert`.

`ConvertAdmissionQuery` chains into `AdmitStudent` and closes the
inquiry with status `Converted`.
