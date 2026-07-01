## Wave 1 Academic Domain Audit Report

**Scope:** `crates/domains/academic/`, `docs/specs/academic/`, `docs/commands/academic.md`, `docs/events/academic.md`, `docs/handoff/PHASE-3-HANDOFF.md`, `AGENTS.md` (the academic row).

**Total findings:** 66

---

### FINDING 1

- **id:** DOMAIN-ACM-001
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/academic/src/lib.rs:24` and `docs/handoff/PHASE-3-HANDOFF.md:33-44`
- **description:** The module-level documentation and the Phase 3 hand-off state inconsistent counts for the shipped artifacts. `lib.rs` says "the 23 typed command shapes", "the 19 typed events", and "the 19 pure factory functions" (`crates/domains/academic/src/lib.rs:24-30`); the hand-off repeats "23 typed commands, 19 typed events, 19 services" (`docs/handoff/PHASE-3-HANDOFF.md:6-11`). The actual code defines 22 command structs, 23 event structs, and 23 service functions.
- **expected:** Self-consistent module documentation.
- **evidence:**
  - `crates/domains/academic/src/lib.rs:24` `//! - [`commands`] — the 23 typed command shapes`
  - `crates/domains/academic/src/lib.rs:25` `//! - [`events`] — the 19 typed events implementing`
  - `crates/domains/academic/src/lib.rs:27` `//! - [`services`] — the 19 pure factory functions`
  - `docs/handoff/PHASE-3-HANDOFF.md:6-7` `5 aggregates ... 23 typed commands, 19 typed events implementing \`DomainEvent\`, 19 pure factory services`

---

### FINDING 2

- **id:** DOMAIN-ACM-002
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/academic/src/aggregate.rs:100`
- **description:** The `Student` aggregate uses raw `String` for the admission number instead of the typed value object `AdmissionNumber` that the spec mandates and `value_objects.rs` already provides.
- **expected:** `pub admission_no: AdmissionNumber` (per engine rule "Compile-time safety over strings" in `AGENTS.md`).
- **evidence:** `crates/domains/academic/src/aggregate.rs:64` `pub admission_no: String,` vs. `crates/domains/academic/src/value_objects.rs:262-265` `pub struct AdmissionNumber(String);` and the spec at `docs/specs/academic/value-objects.md:67` `AdmissionNumber   | 1..50 chars, unique within school`.

---

### FINDING 3

- **id:** DOMAIN-ACM-003
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/academic/src/aggregate.rs:66-68`
- **description:** The `Student` aggregate uses raw `String` for `first_name` and `last_name` instead of the typed value object `PersonName` defined in `value_objects.rs`. The spec mandates typed wrappers for names.
- **expected:** `pub first_name: PersonName` and `pub last_name: PersonName`.
- **evidence:** `crates/domains/academic/src/aggregate.rs:66-68` `pub first_name: String,\n    pub last_name: String,` vs. `crates/domains/academic/src/value_objects.rs:136-165` defining `PersonName` with validation.

---

### FINDING 4

- **id:** DOMAIN-ACM-004
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/academic/src/aggregate.rs:80-82, 84-86`
- **description:** The `Student` aggregate uses `Option<String>` for `mobile`, `email`, `current_address`, `permanent_address`. The spec mandates typed wrappers `PhoneNumber`, `EmailAddress`, and `Address` (all available in `value_objects.rs:211, 258`).
- **expected:** `pub mobile: Option<PhoneNumber>`, `pub email: Option<EmailAddress>`, `pub current_address: Option<Address>`, `pub permanent_address: Option<Address>`.
- **evidence:** `crates/domains/academic/src/aggregate.rs:80-86` declares the four fields as `Option<String>`. `crates/domains/academic/src/value_objects.rs:211-253` defines `Address` and `crates/domains/academic/src/value_objects.rs:801-920` defines the reason wrappers — but no `PhoneNumber` or `EmailAddress` typed wrapper exists (see Finding DOMAIN-ACM-005).

---

### FINDING 5

- **id:** DOMAIN-ACM-005
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/academic/src/value_objects.rs` (entire file)
- **description:** The `PhoneNumber` and `EmailAddress` typed value objects named in the spec and referenced by `Aggregate`/`Command` fields do not exist. The spec at `docs/specs/academic/value-objects.md:58-59` mandates them with E.164 / RFC 5322 validation, but the code only carries `validate_email_optional` and `validate_mobile_optional` as crate-private helper functions in `commands.rs:602-642` that operate on raw `&str`.
- **expected:** `pub struct PhoneNumber(String)` and `pub struct EmailAddress(String)` with constructor-time validation.
- **evidence:** `docs/specs/academic/value-objects.md:58-59` lists `EmailAddress` and `PhoneNumber`; `crates/domains/academic/src/value_objects.rs` has no matching struct (verified via `grep -n 'PhoneNumber\|EmailAddress' crates/domains/academic/src/value_objects.rs` returning no struct definitions). `crates/domains/academic/src/commands.rs:602-642` exposes only `pub(crate) fn validate_email_optional` / `validate_mobile_optional` helper functions.

---

### FINDING 6

- **id:** DOMAIN-ACM-006
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/academic/src/events.rs:53-78` (`StudentAdmitted`)
- **description:** The `StudentAdmitted` event struct diverges from the spec. The spec mandates `admission_no: AdmissionNumber`, `full_name: FullName`, and `guardian_ids: Vec<GuardianId>`; the code carries raw `String` fields for admission_no, first_name, last_name and omits `guardian_ids` entirely.
- **expected:** `pub admission_no: AdmissionNumber`, `pub full_name: FullName`, `pub guardian_ids: Vec<GuardianId>` (per `docs/specs/academic/events.md:39-49`).
- **evidence:**
  - `docs/specs/academic/events.md:39-49` spec struct uses typed wrappers.
  - `crates/domains/academic/src/events.rs:53-78` actual struct: `pub admission_no: String,\n    pub first_name: String,\n    pub last_name: String,` and no `guardian_ids` field.

---

### FINDING 7

- **id:** DOMAIN-ACM-007
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/academic/src/events.rs:438-518` (`StudentPromoted`)
- **description:** The `StudentPromoted` event struct is missing four fields the spec mandates and uses raw `String` for the roll number instead of the typed wrapper `RollNumber`. The spec lists `from_record_id`, `to_record_id`, `from_roll_no`, and `promotion_id`; the code omits all four.
- **expected:** `pub from_record_id: StudentRecordId`, `pub to_record_id: StudentRecordId`, `pub from_roll_no: RollNumber`, `pub to_roll_no: RollNumber`, `pub promotion_id: StudentPromotionId`.
- **evidence:**
  - `docs/specs/academic/events.md:158-174` spec struct.
  - `crates/domains/academic/src/events.rs:438-464` actual struct has `pub to_roll_no: String` and no `from_record_id`, `to_record_id`, `from_roll_no`, or `promotion_id`.

---

### FINDING 8

- **id:** DOMAIN-ACM-008
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/academic/src/events.rs:522-561` (`StudentGraduated`)
- **description:** The `StudentGraduated` event carries a `pub status: StudentStatus` field that is not in the spec.
- **expected:** Spec at `docs/specs/academic/events.md:184-189` lists only `student_id`, `academic_year_id`, `graduation_date`.
- **evidence:** `crates/domains/academic/src/events.rs:530` `pub status: StudentStatus,` vs. spec at `docs/specs/academic/events.md:184-189` which has no `status` field.

---

### FINDING 9

- **id:** DOMAIN-ACM-009
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/academic/src/commands.rs:410-427` (`CreateAcademicYearCommand`)
- **description:** `CreateAcademicYearCommand` carries `pub is_current: bool` and `pub academic_year_id: AcademicYearId` fields that the spec does not declare. The spec's `CreateAcademicYearCommand` does not have either field; the only way to mark a year current is via `SetCurrentAcademicYearCommand`.
- **expected:** Spec at `docs/specs/academic/commands.md:359-367` lists only `tenant`, `year`, `title`, `starting_date`, `ending_date`, `copy_with_academic_year`.
- **evidence:** `crates/domains/academic/src/commands.rs:413-424` declares `pub academic_year_id: AcademicYearId` and `pub is_current: bool` — neither appears in the spec struct.

---

### FINDING 10

- **id:** DOMAIN-ACM-010
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/academic/src/aggregate.rs` (entire file)
- **description:** Only 5 of the 32 aggregates declared in `docs/specs/academic/aggregates.md` are implemented as Rust structs. The 27 missing aggregates are: Guardian, ClassSection, ClassSubject, ClassRoutine, Homework, LessonPlan, Lesson, LessonTopic, StudentRecord, StudentPromotion, StudentCategory, StudentGroup, RegistrationField, Certificate, IdCard, AdmissionQuery, GraduateRecord, ClassRoom, ClassTime, ClassRoutineUpdate, ClassSectionTeacher, AssignClassTeacher, ClassTeacher, ClassOptionalSubject, LessonDetail, LessonTopicDetail, LessonPlanTopic, HomeworkSubmission, LearningObjective, FrontAcademicCalendar, FrontClassRoutine, StudentBulkTemporary, StudentRecordTemporary.
- **expected:** Every aggregate root listed in `docs/specs/academic/aggregates.md` should have a corresponding struct in `crates/domains/academic/src/aggregate.rs`.
- **evidence:** `docs/specs/academic/aggregates.md` enumerates 32 aggregate roots; `crates/domains/academic/src/aggregate.rs` defines only `Student`, `Class`, `Section`, `Subject`, `AcademicYear` (lines 57, 193, 273, 338, 420).

---

### FINDING 11

- **id:** DOMAIN-ACM-011
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/academic/src/commands.rs` (entire file)
- **description:** Only 22 of the 71 commands listed in `docs/commands/academic.md` are declared as Rust command structs. Missing commands include `AssignStudentToSection`, `ChangeStudentCategory`, `AssignOptionalSubject`, `UploadStudentDocument`, `CreateClassSection`, `AssignClassTeacher`, `AssignSubjectTeacher`, `AssignClassRoom`, `DeleteClassSection`, `AssignSubjectToClass`, `ReassignSubjectTeacher`, `UnassignSubjectFromClass`, `CreateClassRoutine`, `UpdateClassRoutinePeriod`, `SwapClassRoutinePeriods`, `DeleteClassRoutine`, `CreateHomework`, `UpdateHomework`, `SubmitHomework`, `EvaluateHomework`, `CancelHomework`, `CreateLessonPlan`, `UpdateLessonPlan`, `MarkLessonPlanCompleted`, `AddSubTopicToLessonPlan`, `DeleteLessonPlan`, `CreateLesson`, `UpdateLesson`, `DeleteLesson`, `CreateLessonTopic`, `MarkLessonTopicCompleted`, `DeleteLessonTopic`, `CreateStudentCategory`, `UpdateStudentCategory`, `DeleteStudentCategory`, `CreateStudentGroup`, `UpdateStudentGroup`, `AddStudentToGroup`, `RemoveStudentFromGroup`, `DeleteStudentGroup`, `CreateRegistrationField`, `UpdateRegistrationField`, `DeleteRegistrationField`, `CreateCertificate`, `UpdateCertificate`, `DeleteCertificate`, `CreateIdCard`, `UpdateIdCard`, `DeleteIdCard`, `RegisterAdmissionQuery`, `FollowUpAdmissionQuery`, `ConvertAdmissionQuery`.
- **expected:** Every row in the `docs/commands/academic.md` catalog corresponds to a typed command shape.
- **evidence:** `docs/commands/academic.md:17-92` lists 71 commands; `crates/domains/academic/src/commands.rs` defines only 22 command structs.

---

### FINDING 12

- **id:** DOMAIN-ACM-012
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/academic/src/events.rs` (entire file)
- **description:** Only 23 of the ~85 events listed in `docs/events/academic.md` are implemented. Missing events include the entire Guardian lifecycle (`GuardianRegistered`, `GuardianContactUpdated`, `GuardianLinkedToStudent`, `GuardianUnlinkedFromStudent`, `PrimaryGuardianMarked`), ClassSection lifecycle (`ClassSectionCreated`, `ClassTeacherAssigned`, `SubjectTeacherAssigned`, `ClassRoomAssigned`, `ClassSectionDeleted`), ClassSubject (`SubjectAssignedToClass`, `TeacherReassigned`, `SubjectUnassigned`), ClassRoutine (`ClassRoutineCreated`, `ClassRoutinePeriodUpdated`, `ClassRoutinePeriodsSwapped`, `ClassRoutineDeleted`), Homework (`HomeworkCreated`, `HomeworkUpdated`, `HomeworkSubmitted`, `HomeworkEvaluated`, `HomeworkCancelled`), Lesson (`LessonCreated`, `LessonUpdated`, `LessonDeleted`, `LessonTopicCreated`, `LessonTopicCompleted`, `LessonTopicDeleted`, `LessonPlanCreated`, `LessonPlanUpdated`, `LessonPlanCompleted`, `SubTopicAdded`, `LessonPlanDeleted`), StudentRecord (`StudentRecordCreated`, `RollNumberAssigned`, `DefaultRecordSet`, `StudentMarkedGraduate`), StudentCategory, StudentGroup, Registration, Certificate, ID Card, and AdmissionQuery events.
- **expected:** Every event in `docs/events/academic.md` corresponds to a struct implementing `DomainEvent`.
- **evidence:** `docs/events/academic.md:10-96` lists ~85 events; `crates/domains/academic/src/events.rs` defines 23 event structs (counted by grep).

---

### FINDING 13

- **id:** DOMAIN-ACM-013
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/academic/src/value_objects.rs` (entire file)
- **description:** Only 6 of the ~30 typed identifiers from `docs/specs/academic/value-objects.md` are defined. Missing: `GuardianId`, `ClassSectionId`, `ClassSubjectId`, `ClassRoutineId`, `ClassRoutineUpdateId`, `ClassTimeId`, `ClassRoomId`, `HomeworkId`, `HomeworkSubmissionId`, `LessonPlanId`, `LessonId`, `LessonDetailId`, `LessonTopicId`, `LessonTopicDetailId`, `LessonPlanTopicId`, `StudentPromotionId`, `StudentCategoryId`, `StudentGroupId`, `StudentDocumentId`, `StudentTimelineId`, `StudentHomeworkId`, `OptionalSubjectAssignmentId`, `RegistrationFieldId`, `CertificateId`, `IdCardId`, `GraduateId`, `AdmissionQueryId`, `AdmissionQueryFollowupId`, `AssignmentSubmissionId`.
- **expected:** Every typed id in the spec has a corresponding `pub struct XxxId { school_id, value }`.
- **evidence:** `docs/specs/academic/value-objects.md:14-50` lists ~30 typed ids; `crates/domains/academic/src/value_objects.rs` defines only `StudentId`, `ClassId`, `SectionId`, `SubjectId`, `AcademicYearId`, `StudentRecordId` (the `academic_typed_id!` invocations at lines 91-128).

---

### FINDING 14

- **id:** DOMAIN-ACM-014
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/academic/src/value_objects.rs` (entire file)
- **description:** Several closed status enums from `docs/specs/academic/value-objects.md` are missing: `LessonPlanStatus`, `ClassSectionStatus`, `HomeworkStatus`, `RegistrationType`, `CertificateLayout`, `CertificateType`, `GuardianRelation`, `AdmissionSource`, `AdmissionReference`, `NoOfChild`, `InquiryStatus`, `DayOfWeek`, `Semester`.
- **expected:** Spec lines `docs/specs/academic/value-objects.md:87-100` lists 12 status enums; code has 4.
- **evidence:** `docs/specs/academic/value-objects.md:87-100` lists 12 status enum rows; `crates/domains/academic/src/value_objects.rs` defines only `StudentStatus`, `Gender`, `BloodGroup`, `SubjectType`, `ResultStatus` (lines 671, 723, 756, 923, 950).

---

### FINDING 15

- **id:** DOMAIN-ACM-015
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/academic/src/repository.rs` (entire file)
- **description:** Only 5 of the ~23 repository port traits from `docs/specs/academic/repositories.md` are defined. Missing: `GuardianRepository`, `ClassSectionRepository`, `ClassSubjectRepository`, `ClassRoutineRepository`, `HomeworkRepository`, `LessonRepository`, `LessonTopicRepository`, `LessonPlanRepository`, `StudentRecordRepository`, `StudentPromotionRepository`, `StudentCategoryRepository`, `StudentGroupRepository`, `RegistrationFieldRepository`, `CertificateRepository`, `IdCardRepository`, `AdmissionQueryRepository`, `ClassRoomRepository`, `ClassTimeRepository`.
- **expected:** Every repository in the spec has a port trait.
- **evidence:** `docs/specs/academic/repositories.md:7-234` lists ~23 repository traits; `crates/domains/academic/src/repository.rs` defines only `StudentRepository`, `ClassRepository`, `SectionRepository`, `SubjectRepository`, `AcademicYearRepository` (lines 43, 103, 126, 149, 172).

---

### FINDING 16

- **id:** DOMAIN-ACM-016
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/academic/src/repository.rs:43-95`
- **description:** The `StudentRepository` trait is missing the `delete` method the spec mandates. The spec at `docs/specs/academic/repositories.md:16` declares `async fn delete(&self, id: StudentId) -> Result<()>;`, but the code's trait body has only `get`, `get_by_admission_no`, `get_by_email`, `list`, `list_by_status`, `list_in_class_section`, `insert`, `update`.
- **expected:** `async fn delete(&self, id: StudentId) -> Result<()>;` per spec.
- **evidence:** `docs/specs/academic/repositories.md:9-28` vs. `crates/domains/academic/src/repository.rs:43-95`.

---

### FINDING 17

- **id:** DOMAIN-ACM-017
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/academic/src/repository.rs:103-117, 126-140, 149-163, 172-190`
- **description:** The `ClassRepository`, `SectionRepository`, `SubjectRepository`, and `AcademicYearRepository` traits are all missing the `delete` method the spec mandates. The spec at `docs/specs/academic/repositories.md:56-57, 68-69, 96-97, 125` declares a `delete` method for each of these aggregates.
- **expected:** Each repository trait has an `async fn delete(&self, id: XxxId) -> Result<()>` method.
- **evidence:** `docs/specs/academic/repositories.md:56-57, 68-69, 96-97, 125` vs. `crates/domains/academic/src/repository.rs` (each trait lacks `delete`).

---

### FINDING 18

- **id:** DOMAIN-ACM-018
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/academic/src/repository.rs:43-95`
- **description:** The `StudentRepository` trait is missing the spec's optimized domain queries: `active_in_class`, `active_in_section`, `admitted_in_range`, `suspended`, `search_by_name`. The spec at `docs/specs/academic/repositories.md:23-27` lists these as required methods.
- **expected:** `async fn active_in_class(...)`, `async fn active_in_section(...)`, `async fn admitted_in_range(...)`, `async fn suspended(...)`, `async fn search_by_name(...)`.
- **evidence:** `docs/specs/academic/repositories.md:23-27` vs. `crates/domains/academic/src/repository.rs:43-95`.

---

### FINDING 19

- **id:** DOMAIN-ACM-019
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/academic/src/services.rs` (entire file)
- **description:** None of the 8 domain services from `docs/specs/academic/services.md` are implemented: `AdmissionService`, `PromotionService`, `EnrollmentService`, `RoutineService`, `HomeworkService`, `LessonPlanService`, `GraduationService`, `ClassSectionAssignmentService`. The spec mandates these as standalone service structs with their own method signatures.
- **expected:** `pub struct AdmissionService;`, `pub struct PromotionService;`, etc.
- **evidence:** `docs/specs/academic/services.md:8-128` defines 8 services; `crates/domains/academic/src/services.rs` defines only per-command factory functions and one `school_matches` helper.

---

### FINDING 20

- **id:** DOMAIN-ACM-020
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/academic/src/services.rs` (entire file)
- **description:** The spec mandates a `OptionalSubjectEligibility` policy at `docs/specs/academic/services.md:111-118` and the `ActiveStudentsInClass` and `PromotableStudents` Specifications at lines `122-141`. None are implemented.
- **expected:** `pub struct OptionalSubjectEligibility;`, `pub struct ActiveStudentsInClass;`, `pub struct PromotableStudents;`.
- **evidence:** `docs/specs/academic/services.md:111-141`; `crates/domains/academic/src/services.rs` has no such struct.

---

### FINDING 21

- **id:** DOMAIN-ACM-021
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/academic/src/services.rs` (entire file)
- **description:** None of the 23 service functions call `Capability::AcademicStudent*` capability checks. The Phase 3 prompt mandates ("Capability-gate via `Capability::AcademicStudent*`") and the per-spec invariant "Capabilities are checked at the command boundary" (`docs/specs/academic/permissions.md:170-179`). The PHASE-3-HANDOFF § "Capability check boundary" acknowledges this is deferred to the dispatcher, but the audit checklist requires service-level assertion of capability.
- **expected:** Every service function takes a `&dyn CapabilityCheck` parameter and asserts the required `Capability` before mutating the aggregate.
- **evidence:** `crates/domains/academic/src/services.rs:90-1201` (entire file): no `CapabilityCheck` import, no `cap.has(...)` calls; only one comment in module docstring at line 14-15 acknowledging this. The `educore-rbac::services::CapabilityCheck` is only used in `crates/domains/cms/src/services.rs:35`.

---

### FINDING 22

- **id:** DOMAIN-ACM-022
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/academic/src/services.rs:498-541` (`promote_student`)
- **description:** `promote_student` takes `student: &Student` (immutable reference) instead of `&mut Student`. The function does not mutate the student's `class_id`, `section_id`, or `roll_no` fields, contradicting the engine rule "the services module is the only place the engine mutates an aggregate and emits its typed event" (`services.rs:12-14`). The doc comment at lines 491-497 acknowledges this by stating "The function does not mutate the student record's class/section fields", which is itself a documentation-confessed spec violation.
- **expected:** `pub fn promote_student<C, G>(student: &mut Student, ...)` that updates `student.class_id`, `student.section_id`, and emits the event.
- **evidence:** `crates/domains/academic/src/services.rs:499` `student: &Student` and the admission doc comment at `services.rs:491-497` explicitly states the class/section fields are not mutated.

---

### FINDING 23

- **id:** DOMAIN-ACM-023
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/academic/src/services.rs:300-309, 666-682, 811-825, 956-973`
- **description:** The update services (`update_student_profile`, `update_class`, `update_section`, `update_subject`) do not set `aggregate.updated_by` after mutating the aggregate, even though the aggregate structs declare `pub updated_by: UserId`. The audit metadata is therefore stale after every update.
- **expected:** `aggregate.updated_by = ctx.actor_id;` after each `updated_at = now;` assignment.
- **evidence:**
  - `crates/domains/academic/src/services.rs:301-304` only sets `student.updated_at`, `student.version`, `student.last_event_id`.
  - `crates/domains/academic/src/services.rs:666-669` only sets `class.updated_at`, `class.version`, `class.last_event_id`.
  - `crates/domains/academic/src/services.rs:811-814` only sets `section.updated_at`, `section.version`, `section.last_event_id`.
  - `crates/domains/academic/src/services.rs:956-959` only sets `subject.updated_at`, `subject.version`, `subject.last_event_id`.
  - The aggregate fields exist: `crates/domains/academic/src/aggregate.rs:112, 217, 291, 362, 448` declare `pub updated_by: UserId`.

---

### FINDING 24

- **id:** DOMAIN-ACM-024
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/academic/src/services.rs:1101-1135` (`set_current_academic_year`)
- **description:** The `set_current_academic_year` service sets `year_agg.is_current = true` but does not enforce the spec invariant "Exactly one academic year may be marked `Current` per school at a time" (`docs/specs/academic/aggregates.md:282`). The service always passes `None` for `previous_id` in the `CurrentAcademicYearSet` event, and defers the demotion to the storage adapter — but no service-level check ensures only one year per school is current.
- **expected:** The service must check `if any other AcademicYear has is_current == true` and return `Err(DomainError::Conflict(...))`, or pass the demoted year id as `previous_id`.
- **evidence:** `crates/domains/academic/src/services.rs:1097-1135` and `docs/specs/academic/aggregates.md:282` invariant.

---

### FINDING 25

- **id:** DOMAIN-ACM-025
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/academic/src/services.rs:445-486` (`transfer_student`)
- **description:** The `TransferStudentCommand` handler does not validate that `destination_school_id` is a "sibling school in same SaaS tenant" as the spec mandates (`docs/specs/academic/commands.md:198`). It only checks `destination_school_id != student.school_id`, leaving the SaaS-tenant relationship unverified.
- **expected:** A check that `destination_school_id` is in the same SaaS tenant as `student.school_id` (requires a `SaaSContext` or similar port).
- **evidence:** `crates/domains/academic/src/services.rs:464-468` only rejects equal school ids; `docs/specs/academic/commands.md:198` and `docs/specs/academic/workflows.md:71-85` require sibling-school validation.

---

### FINDING 26

- **id:** DOMAIN-ACM-026
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/academic/src/aggregate.rs:247-249`
- **description:** Production code uses `unwrap_or_else(|_| unreachable!(...))`, which is a `panic!()`-equivalent anti-pattern that violates the engine rule "unwrap, expect, panic are forbidden in production paths" (`AGENTS.md` Agent Instructions → Type Safety).
- **expected:** Use a checked constructor or `if/else` branch that returns `Result`.
- **evidence:** `crates/domains/academic/src/aggregate.rs:247-249` `optional_subject_gpa_threshold: OptionalSubjectGpaThreshold::new(0.0).unwrap_or_else(\n                |_| unreachable!("0.0 is in the valid OptionalSubjectGpaThreshold range"),\n            ),`.

---

### FINDING 27

- **id:** DOMAIN-ACM-027
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/academic/src/aggregate.rs:637-641`
- **description:** The aggregate test module uses `.expect(...)` five times. While the `#[cfg(test)]` block relaxes the clippy lint, the crate-wide `#![deny(missing_docs)]` and the engine rule against `expect()` in production paths would still flag this if the test helper were promoted to a shared `non_panicking` constructor.
- **expected:** `Etag::placeholder()` returning a typed `Etag` directly without re-validation in tests.
- **evidence:** `crates/domains/academic/src/aggregate.rs:637-641` `Etag::new(Student::FRESH_ETAG).expect("FRESH_ETAG must be a valid etag");` × 5.

---

### FINDING 28

- **id:** DOMAIN-ACM-028
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/academic/src/aggregate.rs:98-101`
- **description:** The `Student` aggregate declares `pub custom_fields: std::collections::BTreeMap<String, String>`. While this is a `BTreeMap` (not `HashMap`), the engine rule spirit is "no key-string-keyed domain data" and the field is unindexed/untyped domain data.
- **expected:** A typed `CustomField { key: FieldName, value: FieldValue }` struct with validated `FieldName`.
- **evidence:** `crates/domains/academic/src/aggregate.rs:100` `pub custom_fields: std::collections::BTreeMap<String, String>,` and the spec's `docs/specs/academic/commands.md:38` declares the same `BTreeMap<String, String>` — so the spec itself permits it, but the engine rule "No HashMap<String, T> for domain data" is structurally violated in the spec.

---

### FINDING 29

- **id:** DOMAIN-ACM-029
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/academic/src/value_objects.rs:671-690` (`StudentStatus`)
- **description:** The `StudentStatus::Applicant` variant is declared and round-trip-tested but is unreachable from any service function. `Student::fresh()` always sets `StudentStatus::Active`. The spec at `docs/specs/academic/aggregates.md:33` mandates the transition `Applicant → Active → ...`; Phase 3 has no way to create an applicant.
- **expected:** Either a `RegisterApplicant` service that sets `StudentStatus::Applicant` or removal of the variant until admission queries land.
- **evidence:** `crates/domains/academic/src/value_objects.rs:675` declares `Applicant`,; `crates/domains/academic/src/aggregate.rs:166` `status: crate::value_objects::StudentStatus::Active`; `crates/domains/academic/src/services.rs:90-190` (admit_student) — no Applicant path.

---

### FINDING 30

- **id:** DOMAIN-ACM-030
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/academic/src/aggregate.rs:50-55` (comment) vs. `crates/domains/academic/src/entities.rs:36-91` (entities placeholder)
- **description:** The `Student` aggregate doc comment lists "StudentDocument, StudentTimeline, StudentHomework" as children, but only `StudentDocument` has a placeholder type. The Timeline and Homework entities are absent, contradicting the documented children.
- **expected:** `StudentTimeline` and `StudentHomework` placeholder structs in `entities.rs` matching the pattern of `StudentDocument`.
- **evidence:** `crates/domains/academic/src/aggregate.rs:52-55` lists all three; `crates/domains/academic/src/entities.rs:36-91` defines only `StudentDocumentId`, `StudentDocument`, `DocumentType`.

---

### FINDING 31

- **id:** DOMAIN-ACM-031
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/academic/src/entities.rs:39-64` (`StudentDocument`)
- **description:** The `StudentDocument` placeholder struct is missing the `ActiveStatus`, `school_id` tenancy column on the variant-level state (only on `id`), and an `updated_at` / `updated_by` audit trail that the aggregate-level invariants require.
- **expected:** `pub active_status: ActiveStatus,` and `pub created_by: UserId,` fields per the engine invariant "audit-first".
- **evidence:** `crates/domains/academic/src/entities.rs:47-64` only declares `id`, `school_id`, `student_id`, `title`, `file_ref`, `document_type`, `created_at`. No `active_status`, no `updated_at`, no `created_by`/`updated_by`.

---

### FINDING 32

- **id:** DOMAIN-ACM-032
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/academic/` (entire crate)
- **description:** The crate has no integration test directory at `crates/domains/academic/tests/`. The Phase 3 hand-off places the integration test at `crates/tools/storage-parity/tests/academic_integration.rs`, but the build plan's "No-Gaps Gates" § 1 mandates per-domain tests at `crates/domains/<domain>/tests/` (`docs/build-plan.md:1834-1864`).
- **expected:** `crates/domains/academic/tests/aggregate_fields.rs`, `commands.rs`, `events.rs`, `services.rs`, `repository.rs`, `value_objects.rs`, `workflows.rs` directories containing hand-written tests.
- **evidence:** `docs/build-plan.md:1834-1864` and `ls /home/beznet/Workspace/smscore/crates/domains/academic/tests/` returning "No such file or directory".

---

### FINDING 33

- **id:** DOMAIN-ACM-033
- **area:** domain-crates
- **severity:** High
- **location:** `crates/tools/storage-parity/tests/academic_integration.rs:444-491` (`academic_event_type_round_trip_for_all_aggregates`)
- **description:** The integration test claims to exercise "all 5 prompt-named aggregates" but only constructs events for `Class` and `Section`. The `Subject` and `AcademicYear` aggregates are not exercised by any integration test. The `coverage.toml` rows for `academic_subjects_aggregate` and `academic_academic_years_aggregate` are marked `Tested` despite no actual test exercising them.
- **expected:** `create_subject` and `create_academic_year` round-trip tests in the integration suite.
- **evidence:** `crates/tools/storage-parity/tests/academic_integration.rs:444-491` only invokes `create_class` and `create_section`; `crates/tools/storage-parity/tests/academic_integration.rs:138` is the only `admit_student` test. `docs/coverage.toml:493-500` marks both aggregates `Tested`.

---

### FINDING 34

- **id:** DOMAIN-ACM-034
- **area:** domain-crates
- **severity:** Medium
- **location:** `docs/coverage.toml:511-516`
- **description:** The `admit_student_command` coverage row is marked `Pending`, but the command struct is implemented at `crates/domains/academic/src/commands.rs:64-106` and is exercised by `crates/tools/storage-parity/tests/academic_integration.rs:226-240`. The matrix does not match the code's actual status.
- **expected:** `status = "Tested"` with the test path.
- **evidence:** `docs/coverage.toml:511-516` `status = "Pending"` for `admit_student_command`; implementation at `crates/domains/academic/src/commands.rs:64-106`; test path exercised at `crates/tools/storage-parity/tests/academic_integration.rs:226-240`.

---

### FINDING 35

- **id:** DOMAIN-ACM-035
- **area:** domain-crates
- **severity:** Medium
- **location:** `docs/coverage.toml:534-540`
- **description:** The `student_admitted_event` coverage row is marked `Pending`, but the event struct is implemented at `crates/domains/academic/src/events.rs:53-132` and is exercised end-to-end by the integration test (event payload asserted at `crates/tools/storage-parity/tests/academic_integration.rs:265`).
- **expected:** `status = "Tested"`.
- **evidence:** `docs/coverage.toml:534-540` `status = "Pending"` for `student_admitted_event`; event code at `crates/domains/academic/src/events.rs:53-132`; test at `crates/tools/storage-parity/tests/academic_integration.rs:265`.

---

### FINDING 36

- **id:** DOMAIN-ACM-036
- **area:** domain-crates
- **severity:** Medium
- **location:** `docs/coverage.toml:502-509` (`academic_enrollments_aggregate`)
- **description:** The `academic_enrollments_aggregate` coverage row references an "enrollments aggregate" that is not in the spec at `docs/specs/academic/aggregates.md`. The closest spec concept is `StudentRecord` (the per-academic-year enrollment), but the coverage row's item name is misleading.
- **expected:** Either rename the row to `academic_student_records_aggregate` (matching spec) or remove the row.
- **evidence:** `docs/coverage.toml:502-509` `item = "academic_enrollments aggregate"`; `docs/specs/academic/aggregates.md:472-507` defines `StudentRecord` as the per-year enrollment.

---

### FINDING 37

- **id:** DOMAIN-ACM-037
- **area:** domain-crates
- **severity:** Medium
- **location:** `docs/coverage.toml:526-532` (`promote_students_command`)
- **description:** The coverage row is named `promote_students_command` (plural), but the actual command struct is `PromoteStudentCommand` (singular) at `crates/domains/academic/src/commands.rs:242-259`. The names don't match, and a `grep` of `docs/commands/academic.md` shows the canonical command is `PromoteStudent` (singular).
- **expected:** Row id `promote_student_command` matching the spec's `PromoteStudent` command and the code's `PromoteStudentCommand`.
- **evidence:** `docs/coverage.toml:526-532` `id = "promote_students_command"` (plural); `docs/commands/academic.md:29` `PromoteStudent` (singular); `crates/domains/academic/src/commands.rs:242` `pub struct PromoteStudentCommand` (singular).

---

### FINDING 38

- **id:** DOMAIN-ACM-038
- **area:** domain-crates
- **severity:** Medium
- **location:** `docs/coverage.toml` (entire file)
- **description:** The coverage matrix is missing rows for the 17 implemented command structs and 22 implemented event structs that are not in the matrix. Examples (not exhaustive): `update_student_profile_command`, `suspend_student_command`, `reinstate_student_command`, `withdraw_student_command`, `transfer_student_command`, `graduate_student_command`, `create_class_command`, `update_class_command`, `set_optional_subject_gpa_threshold_command`, `delete_class_command`, `create_section_command`, `update_section_command`, `delete_section_command`, `create_subject_command`, `update_subject_command`, `delete_subject_command`, `create_academic_year_command`, `update_academic_year_dates_command`, `set_current_academic_year_command`, `close_academic_year_command`; plus event rows like `student_suspended_event`, `student_reinstated_event`, `student_withdrawn_event`, `student_transferred_event`, `student_promoted_event`, `student_graduated_event`, `student_profile_updated_event`, `class_created_event`, `class_updated_event`, `class_deleted_event`, `optional_subject_gpa_threshold_set_event`, `section_created_event`, `section_updated_event`, `section_deleted_event`, `subject_created_event`, `subject_updated_event`, `subject_deleted_event`, `academic_year_created_event`, `academic_year_dates_updated_event`, `current_academic_year_set_event`, `academic_year_closed_event`, `academic_year_copied_event`.
- **expected:** Every code-defined command and event has a corresponding coverage row.
- **evidence:** `crates/domains/academic/src/commands.rs` defines 22 command structs (counted); `docs/coverage.toml` only has 3 command rows for academic.

---

### FINDING 39

- **id:** DOMAIN-ACM-039
- **area:** domain-crates
- **severity:** Low
- **location:** `crates/domains/academic/src/services.rs:1269-1296`
- **description:** The test helper `InMemoryUniqueness::record_email` (and the field `emails` it pushes to) is annotated `#[allow(dead_code)]` because no test exercises the email-uniqueness code path. This indicates a missing test case for the documented email-uniqueness invariant of the `admit_student` service.
- **expected:** A test that records an email, then asserts `admit_student` with the same email returns `Err(DomainError::Conflict(...))`.
- **evidence:** `crates/domains/academic/src/services.rs:1273-1278` `#[allow(dead_code)]\n        fn record_email(...)`; `crates/domains/academic/src/services.rs:1348-1366` `admit_student_uniqueness_violation` only tests admission_no, not email.

---

### FINDING 40

- **id:** DOMAIN-ACM-040
- **area:** domain-crates
- **severity:** Low
- **location:** `crates/domains/academic/src/services.rs:1273-1296`
- **description:** `InMemoryUniqueness::student_email_exists` always lower-cases the input, but the code at `services.rs:138-145` lowercases via `let lower = e.to_lowercase();` before passing it. The double-lowercasing is benign but the contract should be that the caller passes a lowercased string (per the doc at `commands.rs:54-55`).
- **expected:** A test that verifies the email-uniqueness comparison is case-insensitive end-to-end.
- **evidence:** `crates/domains/academic/src/commands.rs:54-55` documents "The check is case-insensitive; the caller is responsible for lowercasing before the call." `crates/domains/academic/src/services.rs:1289-1296` lowercases anyway.

---

### FINDING 41

- **id:** DOMAIN-ACM-041
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/academic/src/services.rs:1226-1230` (test module imports) and `crates/domains/academic/src/services.rs` test functions
- **description:** No service-level test exercises the spec's decision-table for the Student status transitions (Applicant → Active → {Suspended, Withdrawn, Graduated, Transferred}). The test `suspend_reinstate_withdraw_transfer_graduate_change_status` at line 1368 covers some transitions but not the rejection of invalid ones (e.g., calling `graduate_student` on an already-Withdrawn student should return `Err`).
- **expected:** A parametrized decision-table test asserting that invalid status transitions return `Err(DomainError::Conflict(...))`.
- **evidence:** `crates/domains/academic/src/services.rs:1368-1440` `suspend_reinstate_withdraw_transfer_graduate_change_status` only exercises the happy-path.

---

### FINDING 42

- **id:** DOMAIN-ACM-042
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/academic/src/services.rs` (entire file, test module)
- **description:** None of the 9 workflows from `docs/specs/academic/workflows.md` (Admission, Promotion, Withdrawal, Transfer, Routine Construction, Homework, Lesson Plan, Admission Query, Class-Section Lifecycle) are exercised by any integration test or workflow test file.
- **expected:** `crates/domains/academic/tests/workflows.rs` containing hand-written multi-step workflow tests, per the per-domain gate (`docs/build-plan.md:1864`).
- **evidence:** `docs/specs/academic/workflows.md:6-148` defines 9 workflows; `crates/domains/academic/src/services.rs` tests do not span multi-step workflows.

---

### FINDING 43

- **id:** DOMAIN-ACM-043
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/academic/src/commands.rs:153-154`
- **description:** `AdmitStudentCommand::school_id()` is the only command with a `school_id()` helper; the other 21 commands require callers to manually read `cmd.tenant.school_id` or the typed id's `.school_id()`. This is an inconsistency in the public command shape API.
- **expected:** Either all commands have a `school_id()` helper or none do.
- **evidence:** `crates/domains/academic/src/commands.rs:149-154` defines `pub fn school_id()` only on `AdmitStudentCommand`; no other command struct defines it.

---

### FINDING 44

- **id:** DOMAIN-ACM-044
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/academic/src/events.rs:32` (`#![allow(clippy::too_many_arguments)]`)
- **description:** The events module carries a crate-level `#![allow(clippy::too_many_arguments)]` to silence a clippy warning. Every event constructor takes 6-12 arguments; this is a structural smell that suggests builder or factory helpers are warranted.
- **expected:** Builders (`StudentAdmitted::builder()`) or named-argument structs.
- **evidence:** `crates/domains/academic/src/events.rs:32` `#![allow(clippy::too_many_arguments)]` and constructor methods with 10-12 positional args (e.g. `StudentPromoted::new` at lines 470-498).

---

### FINDING 45

- **id:** DOMAIN-ACM-045
- **area:** domain-crates
- **severity:** Low
- **location:** `crates/domains/academic/src/commands.rs:472-722`
- **description:** The `pub(crate)` validation helpers (`validate_first_name`, `validate_email_optional`, etc.) duplicate validation logic that already exists as typed-value-object constructors in `value_objects.rs:145-156, 220-232, 272-286, 320-332, 366-378, 410-422, 456-468, 502-514, 808-826, 848-866, 888-906`. The commands take raw strings and validate them, instead of taking typed wrappers and reusing the value-object constructors.
- **expected:** `AdmitStudentCommand` carries `first_name: PersonName`, etc.
- **evidence:** `crates/domains/academic/src/commands.rs:472-722` exposes 15+ `pub(crate) fn validate_*` helpers; `crates/domains/academic/src/value_objects.rs` already provides typed constructors that perform the same validation.

---

### FINDING 46

- **id:** DOMAIN-ACM-046
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/academic/src/commands.rs:432-435`
- **description:** `CreateAcademicYearCommand::range()` recomputes the `AcademicYearRange` validation that `create_academic_year` (services.rs:1031) performs again via `AcademicYearRange::new(starting_date, ending_date)?`. The duplicate call wastes work and means validation logic is split between the command and the service.
- **expected:** The service should call `cmd.range()?` to reuse the command's validation.
- **evidence:** `crates/domains/academic/src/commands.rs:432-435` vs. `crates/domains/academic/src/services.rs:1031` — both call `AcademicYearRange::new(starting_date, ending_date)`.

---

### FINDING 47

- **id:** DOMAIN-ACM-047
- **area:** domain-crates
- **severity:** Low
- **location:** `crates/domains/academic/src/services.rs:1207-1213` (`school_matches`)
- **description:** The `school_matches` helper is documented as "Used by command dispatchers (outside the services module) to enforce the `school_id` invariant at the command boundary", but it is defined inside `services.rs` and re-exported from `lib.rs:88`. Its location contradicts its documented intent.
- **expected:** Move `school_matches` to a `tenant_helpers.rs` or `guards.rs` module consumed by the dispatcher.
- **evidence:** `crates/domains/academic/src/services.rs:1207-1213` docstring says "Used by command dispatchers (outside the services module)".

---

### FINDING 48

- **id:** DOMAIN-ACM-048
- **area:** domain-crates
- **severity:** Low
- **location:** `docs/handoff/PHASE-3-HANDOFF.md:99-103`
- **description:** The hand-off claims "66 unit tests in `educore-academic`". The actual count is 67 unit tests (aggregate.rs:6 + commands.rs:10 + entities.rs:2 + errors.rs:0 + events.rs:4 + lib.rs:4 + query.rs:5 + repository.rs:1 + services.rs:16 + value_objects.rs:19).
- **expected:** Self-consistent test count.
- **evidence:** `docs/handoff/PHASE-3-HANDOFF.md:99-103` says "66 unit tests". Actual: 67.

---

### FINDING 49

- **id:** DOMAIN-ACM-049
- **area:** domain-crates
- **severity:** Medium
- **location:** `docs/specs/academic/tables.md:7-58` (entire table list)
- **description:** The spec lists 50 academic tables, but the implementation has no `#[derive(DomainQuery)]` struct in `crates/domains/academic/src/entities.rs`. The entities.rs file is a placeholder with only `StudentDocumentId`, `StudentDocument`, `DocumentType` shells.
- **expected:** Per `docs/build-plan.md:1875-1882`, every `tables.md` row should correspond to a `#[derive(DomainQuery)]` struct in `entities.rs`. None exist in the academic crate.
- **evidence:** `docs/specs/academic/tables.md:7-58` lists 50 tables; `crates/domains/academic/src/entities.rs` has no `#[derive(DomainQuery)]` (verified by `grep 'DomainQuery' crates/domains/academic/src/entities.rs` returning no matches).

---

### FINDING 50

- **id:** DOMAIN-ACM-050
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/academic/src/commands.rs:64-106` (`AdmitStudentCommand`) vs. `docs/specs/academic/commands.md:13-39`
- **description:** `AdmitStudentCommand` is missing the `student_category_id: Option<StudentCategoryId>`, `student_group_ids: Vec<StudentGroupId>`, `guardians: Vec<GuardianSpec>`, `transport: Option<TransportSpec>`, `hostel: Option<HostelSpec>`, and `documents: Vec<DocumentSpec>` fields the spec mandates. The Phase 3 hand-off § OQ #6 acknowledges this as "scoped out" but the audit checklist requires spec-vs-code drift reporting.
- **expected:** Fields `student_category_id`, `student_group_ids`, `guardians`, `transport`, `hostel`, `documents` per the spec struct.
- **evidence:** `docs/specs/academic/commands.md:13-39` lists all 6; `crates/domains/academic/src/commands.rs:64-106` has none of them.

---

### FINDING 51

- **id:** DOMAIN-ACM-051
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/academic/src/commands.rs:157-180` (`UpdateStudentProfileCommand`) vs. `docs/specs/academic/commands.md:57-61`
- **description:** `UpdateStudentProfileCommand` flattens the spec's `StudentProfilePatch` sub-struct. The spec mandates a nested `pub patch: StudentProfilePatch` field; the code declares the patch fields inline on the command itself.
- **expected:** `pub struct StudentProfilePatch { ... }` and `pub patch: StudentProfilePatch` on the command.
- **evidence:** `docs/specs/academic/commands.md:57-61` spec struct; `crates/domains/academic/src/commands.rs:157-180` actual struct (no nested patch).

---

### FINDING 52

- **id:** DOMAIN-ACM-052
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/academic/src/commands.rs:262-272` (`GraduateStudentCommand`) vs. `docs/specs/academic/commands.md:238-246`
- **description:** `GraduateStudentCommand` is missing the `pub destination: Option<GraduateDestination>` field the spec mandates.
- **expected:** `pub destination: Option<GraduateDestination>`.
- **evidence:** `docs/specs/academic/commands.md:238-246` lists the field; `crates/domains/academic/src/commands.rs:262-272` omits it.

---

### FINDING 53

- **id:** DOMAIN-ACM-053
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/academic/src/aggregate.rs:672-718`
- **description:** The `Class` aggregate is missing the `capacity: Option<u32>` field the spec implies (`docs/specs/academic/workflows.md:151-155` mentions "A class may have a `Capacity` (a domain configuration value, not a hard invariant)"). The workflows spec says admission may be configured to reject when capacity is exceeded; the absence of the field makes capacity enforcement impossible.
- **expected:** `pub capacity: Option<u32>` on `Class` with optional capacity check at `create_class` / `update_class` services.
- **evidence:** `docs/specs/academic/workflows.md:151-155`; `crates/domains/academic/src/aggregate.rs:193-224` (`Class` struct) has no `capacity`.

---

### FINDING 54

- **id:** DOMAIN-ACM-054
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/academic/src/aggregate.rs:272-298` (`Section`)
- **description:** The `Section` aggregate has no `capacity` field. The spec invariant at `docs/specs/academic/workflows.md:151-155` allows per-section capacity; absent field.
- **expected:** Optional `capacity: Option<u32>`.
- **evidence:** `crates/domains/academic/src/aggregate.rs:272-298`; `docs/specs/academic/workflows.md:151-155`.

---

### FINDING 55

- **id:** DOMAIN-ACM-055
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/academic/src/services.rs:1527-1547` (`promote_emits_event_with_from_to`) and `crates/domains/academic/src/services.rs:1790-1825` (`update_student_profile_changes_only_supplied_fields`)
- **description:** The unit tests for `promote_student` and `update_student_profile` do not assert that the student aggregate's `class_id`, `section_id`, or `roll_no` were updated. Since `promote_student` takes `&Student` (not `&mut Student`), these fields are never mutated; the tests pass vacuously without verifying the spec's required state transition.
- **expected:** Tests that fail when the mutation contract is violated.
- **evidence:** `crates/domains/academic/src/services.rs:1542-1546` only asserts `event.from_class_id == class_a`, etc., but doesn't assert the student's class_id/section_id/roll_no are updated.

---

### FINDING 56

- **id:** DOMAIN-ACM-056
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/academic/src/query.rs:116-122, 152-158, 188-194, 244-250, 310-316` (all 5 `execute` stubs)
- **description:** All 5 query stubs (`StudentQuery::execute`, `ClassQuery::execute`, `SectionQuery::execute`, `SubjectQuery::execute`, `AcademicYearQuery::execute`) return `Err(DomainError::not_supported(...))`. The Phase 3 hand-off acknowledges this is deferred to Phase 4+, but no consumer can run a query against the academic domain without an immediate failure.
- **expected:** At minimum, an in-memory executor for unit tests.
- **evidence:** `crates/domains/academic/src/query.rs:118-121` and 4 sibling sites.

---

### FINDING 57

- **id:** DOMAIN-ACM-057
- **area:** domain-crates
- **severity:** Low
- **location:** `crates/domains/academic/src/value_objects.rs:179-208` (`FullName`)
- **description:** `FullName` is defined and re-exported in `lib.rs:123`, `lib.rs:183`, but is not referenced by any aggregate field, command struct, or event struct. The spec at `docs/specs/academic/events.md:43` mandates `pub full_name: FullName` on `StudentAdmitted`; the code at `events.rs:57-61` instead declares separate `first_name` + `last_name` strings.
- **expected:** `pub full_name: FullName` on `StudentAdmitted`; corresponding field on `Student`.
- **evidence:** `crates/domains/academic/src/value_objects.rs:179-208` defines `FullName`; `crates/domains/academic/src/events.rs:53-78` `StudentAdmitted` does not reference it; `crates/domains/academic/src/aggregate.rs:57-119` `Student` does not reference it.

---

### FINDING 58

- **id:** DOMAIN-ACM-058
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/academic/src/aggregate.rs:1-643` (entire file) vs. `crates/domains/academic/src/value_objects.rs:801-920` (`SuspensionReason`, `WithdrawalReason`, `TransferReason`)
- **description:** The `Student` aggregate stores `suspension_reason`, `withdrawal_reason`, and `transfer_reason` only on the events, not as persistent fields on the aggregate. After a suspension, withdrawal, or transfer, the aggregate's `reason` is not queryable — consumers must read the event log.
- **expected:** `pub last_suspension_reason: Option<SuspensionReason>` etc. fields on `Student` for audit-trail support.
- **evidence:** `crates/domains/academic/src/aggregate.rs:57-119` `Student` struct fields enumerated; no reason fields.

---

### FINDING 59

- **id:** DOMAIN-ACM-059
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/academic/src/services.rs:174-188`
- **description:** `admit_student` emits `StudentAdmitted` but never persists the corresponding `StudentRecord` row. The spec at `docs/specs/academic/commands.md:51` says "Creates a `Student`, one `StudentRecord` for the academic year, guardian links, ...". The code emits no `StudentRecordCreated` event because the `StudentRecord` aggregate does not exist in Phase 3.
- **expected:** A `StudentRecord` aggregate and `StudentRecordCreated` event emitted alongside `StudentAdmitted`.
- **evidence:** `crates/domains/academic/src/services.rs:175-188` only emits `StudentAdmitted`; `docs/specs/academic/commands.md:51` mandates both `StudentAdmitted` and `StudentRecordCreated`.

---

### FINDING 60

- **id:** DOMAIN-ACM-060
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/academic/src/services.rs:498-541` and `crates/domains/academic/src/services.rs:543-580`
- **description:** `promote_student` does not emit `StudentRecordCreated` for the new academic year, and `graduate_student` does not emit `StudentMarkedGraduate`. Both events are required by the spec at `docs/events/academic.md:75, 268` and `docs/specs/academic/commands.md:232-249`.
- **expected:** `StudentRecordCreated` and `StudentMarkedGraduate` events emitted as part of the corresponding service flows.
- **evidence:** `crates/domains/academic/src/services.rs:498-541` (`promote_student`) emits only `StudentPromoted`; `crates/domains/academic/src/services.rs:543-580` (`graduate_student`) emits only `StudentGraduated`; `docs/events/academic.md:75, 268` and `docs/specs/academic/commands.md:232-249` require both.

---

### FINDING 61

- **id:** DOMAIN-ACM-061
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/academic/src/aggregate.rs:247-249` and `crates/domains/academic/src/services.rs:1008-1058`
- **description:** `CreateAcademicYearCommand::is_current` is read by `create_academic_year` (services.rs:1042) and used to set the new year's `is_current` flag, but no other year in the school is demoted. The spec at `docs/specs/academic/commands.md:372-373` mandates "`SetCurrentAcademicYear` is the only command that mutates the `Current` flag". By allowing `CreateAcademicYear` to set `is_current`, the code violates the invariant that `SetCurrentAcademicYear` is the sole mutator.
- **expected:** `CreateAcademicYearCommand::is_current` should not exist; callers must use `SetCurrentAcademicYearCommand` after creation.
- **evidence:** `crates/domains/academic/src/services.rs:1042` `year_agg.is_current = is_current;` vs. `docs/specs/academic/commands.md:372-373`.

---

### FINDING 62

- **id:** DOMAIN-ACM-062
- **area:** domain-crates
- **severity:** Low
- **location:** `crates/domains/academic/src/lib.rs:115-117`
- **description:** The `UniquenessChecker` port is re-exported at the crate root via `pub use crate::commands::UniquenessChecker;` but the module-level docstring at `lib.rs:31` does not document this port. Consumers reading the module doc may miss the port.
- **expected:** Module doc lists `UniquenessChecker` as a public surface.
- **evidence:** `crates/domains/academic/src/lib.rs:31` lists "errors" but not the UniquenessChecker port; `lib.rs:115-117` re-exports it.

---

### FINDING 63

- **id:** DOMAIN-ACM-063
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/academic/src/events.rs:502-518` (`StudentPromoted` `EVENT_TYPE`)
- **description:** The `StudentPromoted` event `EVENT_TYPE` constant is `"academic.student.promoted"`, but the events catalog at `docs/events/academic.md:22` uses the same form. However, the spec at `docs/specs/academic/events.md:158-174` describes the event semantics — and crucially, the `StudentPromoted` event is intended to drive downstream subscribers in finance, attendance, and assessment (`docs/specs/academic/overview.md:105-111`). The missing `promotion_id` and `from_record_id`/`to_record_id` fields (Finding DOMAIN-ACM-007) mean downstream consumers cannot correlate the promotion with the new `StudentRecord`.
- **expected:** Restored `from_record_id`, `to_record_id`, `from_roll_no`, and `promotion_id` fields.
- **evidence:** `crates/domains/academic/src/events.rs:438-518` actual struct; `docs/specs/academic/overview.md:105-111` cross-domain consumer contracts.

---

### FINDING 64

- **id:** DOMAIN-ACM-064
- **area:** domain-crates
- **severity:** Low
- **location:** `crates/domains/academic/src/repository.rs:42-95`
- **description:** The `StudentRepository` trait methods accept `&TenantContext` for tenant scoping (`docs/specs/academic/repositories.md:8-14` describes methods as `async fn get(&self, id: StudentId) -> Result<Option<Student>>` — no `TenantContext`). The code adds `&TenantContext` to most methods, but the spec's intent (matched to `docs/specs/academic/repositories.md:7-29`) is that the school id is part of the typed id and the repository enforces tenancy via `school_id` predicates, not via a `TenantContext` parameter.
- **expected:** `async fn get(&self, id: StudentId)` per spec; the typed id's `school_id` field already carries the tenant.
- **evidence:** `docs/specs/academic/repositories.md:9-28` spec signatures; `crates/domains/academic/src/repository.rs:47` `async fn get(&self, ctx: &TenantContext, id: StudentId)`.

---

### FINDING 65

- **id:** DOMAIN-ACM-065
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/academic/src/services.rs:174-189`
- **description:** `admit_student` mutates the `Student` aggregate's `correlation_id` to `ctx.correlation_id` (line 173) but later emits the event with `ctx.correlation_id` (line 186). The aggregate stores the new value but the event uses the same value. There is no test asserting that the aggregate's correlation_id equals the event's correlation_id, but more importantly, the code never checks that `cmd.tenant.correlation_id` matches `student.correlation_id` — a tenant-context mismatch would be silently accepted.
- **expected:** A debug-assert or validation that `cmd.tenant.school_id == student_id.school_id`.
- **evidence:** `crates/domains/academic/src/services.rs:163, 173, 186`.

---

### FINDING 66

- **id:** DOMAIN-ACM-066
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/academic/src/services.rs:205-310` (`update_student_profile`)
- **description:** `update_student_profile` ignores the `cmd.tenant` field (it uses `_ctx: &TenantContext` and `tenant: _`). The service relies on the dispatcher to perform the capability check; without it, the function can be invoked by anyone holding a `&mut Student`. The audit gap is that the function does not even sanity-check `student_id == student.id` at runtime (it uses `debug_assert_eq!` at line 229).
- **expected:** A `Result`-returning assertion (not `debug_assert_eq!`) of `cmd.student_id == student.id`.
- **evidence:** `crates/domains/academic/src/services.rs:206-229` `pub fn update_student_profile<C, G>(_ctx: &TenantContext, ...)` and `debug_assert_eq!(student_id, student.id);` (line 229).

---

### END FINDINGS
Total Findings: 66