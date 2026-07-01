# 01 - Audit Appendix - Domains (10 bounded contexts)

**Scope:** wave1-academic.md, wave1-assessment.md, wave1-attendance.md, wave1-cms.md, wave1-communication.md, wave1-documents.md, wave1-events-domain.md, wave1-facilities.md, wave1-finance.md, wave1-hr-library.md

**Total findings:** 578

**Severity distribution:** 145 critical, 213 high, 164 medium, 56 low


## Summary Table

| Target | Critical | High | Medium | Low | Total |
| --- | --- | --- | --- | --- | --- |
| Academic (`DOMAIN-ACM`) | 11 | 20 | 27 | 8 | 66 |
| Assessment (`DOMAIN-ASS`) | 19 | 56 | 25 | 0 | 100 |
| Attendance (`DOMAIN-ATT`) | 26 | 16 | 9 | 2 | 53 |
| CMS (`DOMAIN-CMS`) | 10 | 17 | 29 | 11 | 67 |
| Communication (`DOMAIN-COM`) | 28 | 13 | 5 | 1 | 47 |
| Documents (`DOMAIN-DOC`) | 4 | 10 | 15 | 10 | 39 |
| Events (domain) (`DOMAIN-EVD`) | 23 | 9 | 11 | 17 | 60 |
| Facilities (`DOM-FAC`) | 5 | 8 | 16 | 3 | 32 |
| Finance (`DOMAIN-FIN`) | 12 | 57 | 16 | 0 | 85 |
| HR & Library (`DOM-HRLIB`) | 7 | 7 | 11 | 4 | 29 |

## Academic (target id prefix: `DOMAIN-ACM`)

**Path:** `crates/domains/academic/`  
**Total findings:** 66 (11 critical, 20 high, 27 medium, 8 low)


### FINDING 1 (id: `DOMAIN-ACM-001`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/lib.rs:24` and `docs/handoff/PHASE-3-HANDOFF.md:33-44`

**Description:**

The module-level documentation and the Phase 3 hand-off state inconsistent counts for the shipped artifacts. `lib.rs` says "the 23 typed command shapes", "the 19 typed events", and "the 19 pure factory functions" (`crates/domains/academic/src/lib.rs:24-30`); the hand-off repeats "23 typed commands, 19 typed events, 19 services" (`docs/handoff/PHASE-3-HANDOFF.md:6-11`). The actual code defines 22 command structs, 23 event structs, and 23 service functions.

**Expected:**

Self-consistent module documentation.

**Evidence:**

- `crates/domains/academic/src/lib.rs:24` `//! - [`commands`] — the 23 typed command shapes`
  - `crates/domains/academic/src/lib.rs:25` `//! - [`events`] — the 19 typed events implementing`
  - `crates/domains/academic/src/lib.rs:27` `//! - [`services`] — the 19 pure factory functions`
  - `docs/handoff/PHASE-3-HANDOFF.md:6-7` `5 aggregates ... 23 typed commands, 19 typed events implementing \`DomainEvent\`, 19 pure factory services`

---

### FINDING 10 (id: `DOMAIN-ACM-010`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/aggregate.rs` (entire file)

**Description:**

Only 5 of the 32 aggregates declared in `docs/specs/academic/aggregates.md` are implemented as Rust structs. The 27 missing aggregates are: Guardian, ClassSection, ClassSubject, ClassRoutine, Homework, LessonPlan, Lesson, LessonTopic, StudentRecord, StudentPromotion, StudentCategory, StudentGroup, RegistrationField, Certificate, IdCard, AdmissionQuery, GraduateRecord, ClassRoom, ClassTime, ClassRoutineUpdate, ClassSectionTeacher, AssignClassTeacher, ClassTeacher, ClassOptionalSubject, LessonDetail, LessonTopicDetail, LessonPlanTopic, HomeworkSubmission, LearningObjective, FrontAcademicCalendar, FrontClassRoutine, StudentBulkTemporary, StudentRecordTemporary.

**Expected:**

Every aggregate root listed in `docs/specs/academic/aggregates.md` should have a corresponding struct in `crates/domains/academic/src/aggregate.rs`.

**Evidence:**

`docs/specs/academic/aggregates.md` enumerates 32 aggregate roots; `crates/domains/academic/src/aggregate.rs` defines only `Student`, `Class`, `Section`, `Subject`, `AcademicYear` (lines 57, 193, 273, 338, 420).

---

### FINDING 11 (id: `DOMAIN-ACM-011`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/commands.rs` (entire file)

**Description:**

Only 22 of the 71 commands listed in `docs/commands/academic.md` are declared as Rust command structs. Missing commands include `AssignStudentToSection`, `ChangeStudentCategory`, `AssignOptionalSubject`, `UploadStudentDocument`, `CreateClassSection`, `AssignClassTeacher`, `AssignSubjectTeacher`, `AssignClassRoom`, `DeleteClassSection`, `AssignSubjectToClass`, `ReassignSubjectTeacher`, `UnassignSubjectFromClass`, `CreateClassRoutine`, `UpdateClassRoutinePeriod`, `SwapClassRoutinePeriods`, `DeleteClassRoutine`, `CreateHomework`, `UpdateHomework`, `SubmitHomework`, `EvaluateHomework`, `CancelHomework`, `CreateLessonPlan`, `UpdateLessonPlan`, `MarkLessonPlanCompleted`, `AddSubTopicToLessonPlan`, `DeleteLessonPlan`, `CreateLesson`, `UpdateLesson`, `DeleteLesson`, `CreateLessonTopic`, `MarkLessonTopicCompleted`, `DeleteLessonTopic`, `CreateStudentCategory`, `UpdateStudentCategory`, `DeleteStudentCategory`, `CreateStudentGroup`, `UpdateStudentGroup`, `AddStudentToGroup`, `RemoveStudentFromGroup`, `DeleteStudentGroup`, `CreateRegistrationField`, `UpdateRegistrationField`, `DeleteRegistrationField`, `CreateCertificate`, `UpdateCertificate`, `DeleteCertificate`, `CreateIdCard`, `UpdateIdCard`, `DeleteIdCard`, `RegisterAdmissionQuery`, `FollowUpAdmissionQuery`, `ConvertAdmissionQuery`.

**Expected:**

Every row in the `docs/commands/academic.md` catalog corresponds to a typed command shape.

**Evidence:**

`docs/commands/academic.md:17-92` lists 71 commands; `crates/domains/academic/src/commands.rs` defines only 22 command structs.

---

### FINDING 12 (id: `DOMAIN-ACM-012`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/events.rs` (entire file)

**Description:**

Only 23 of the ~85 events listed in `docs/events/academic.md` are implemented. Missing events include the entire Guardian lifecycle (`GuardianRegistered`, `GuardianContactUpdated`, `GuardianLinkedToStudent`, `GuardianUnlinkedFromStudent`, `PrimaryGuardianMarked`), ClassSection lifecycle (`ClassSectionCreated`, `ClassTeacherAssigned`, `SubjectTeacherAssigned`, `ClassRoomAssigned`, `ClassSectionDeleted`), ClassSubject (`SubjectAssignedToClass`, `TeacherReassigned`, `SubjectUnassigned`), ClassRoutine (`ClassRoutineCreated`, `ClassRoutinePeriodUpdated`, `ClassRoutinePeriodsSwapped`, `ClassRoutineDeleted`), Homework (`HomeworkCreated`, `HomeworkUpdated`, `HomeworkSubmitted`, `HomeworkEvaluated`, `HomeworkCancelled`), Lesson (`LessonCreated`, `LessonUpdated`, `LessonDeleted`, `LessonTopicCreated`, `LessonTopicCompleted`, `LessonTopicDeleted`, `LessonPlanCreated`, `LessonPlanUpdated`, `LessonPlanCompleted`, `SubTopicAdded`, `LessonPlanDeleted`), StudentRecord (`StudentRecordCreated`, `RollNumberAssigned`, `DefaultRecordSet`, `StudentMarkedGraduate`), StudentCategory, StudentGroup, Registration, Certificate, ID Card, and AdmissionQuery events.

**Expected:**

Every event in `docs/events/academic.md` corresponds to a struct implementing `DomainEvent`.

**Evidence:**

`docs/events/academic.md:10-96` lists ~85 events; `crates/domains/academic/src/events.rs` defines 23 event structs (counted by grep).

---

### FINDING 13 (id: `DOMAIN-ACM-013`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/value_objects.rs` (entire file)

**Description:**

Only 6 of the ~30 typed identifiers from `docs/specs/academic/value-objects.md` are defined. Missing: `GuardianId`, `ClassSectionId`, `ClassSubjectId`, `ClassRoutineId`, `ClassRoutineUpdateId`, `ClassTimeId`, `ClassRoomId`, `HomeworkId`, `HomeworkSubmissionId`, `LessonPlanId`, `LessonId`, `LessonDetailId`, `LessonTopicId`, `LessonTopicDetailId`, `LessonPlanTopicId`, `StudentPromotionId`, `StudentCategoryId`, `StudentGroupId`, `StudentDocumentId`, `StudentTimelineId`, `StudentHomeworkId`, `OptionalSubjectAssignmentId`, `RegistrationFieldId`, `CertificateId`, `IdCardId`, `GraduateId`, `AdmissionQueryId`, `AdmissionQueryFollowupId`, `AssignmentSubmissionId`.

**Expected:**

Every typed id in the spec has a corresponding `pub struct XxxId { school_id, value }`.

**Evidence:**

`docs/specs/academic/value-objects.md:14-50` lists ~30 typed ids; `crates/domains/academic/src/value_objects.rs` defines only `StudentId`, `ClassId`, `SectionId`, `SubjectId`, `AcademicYearId`, `StudentRecordId` (the `academic_typed_id!` invocations at lines 91-128).

---

### FINDING 2 (id: `DOMAIN-ACM-002`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/aggregate.rs:100`

**Description:**

The `Student` aggregate uses raw `String` for the admission number instead of the typed value object `AdmissionNumber` that the spec mandates and `value_objects.rs` already provides.

**Expected:**

`pub admission_no: AdmissionNumber` (per engine rule "Compile-time safety over strings" in `AGENTS.md`).

**Evidence:**

`crates/domains/academic/src/aggregate.rs:64` `pub admission_no: String,` vs. `crates/domains/academic/src/value_objects.rs:262-265` `pub struct AdmissionNumber(String);` and the spec at `docs/specs/academic/value-objects.md:67` `AdmissionNumber   | 1..50 chars, unique within school`.

---

### FINDING 21 (id: `DOMAIN-ACM-021`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/services.rs` (entire file)

**Description:**

None of the 23 service functions call `Capability::AcademicStudent*` capability checks. The Phase 3 prompt mandates ("Capability-gate via `Capability::AcademicStudent*`") and the per-spec invariant "Capabilities are checked at the command boundary" (`docs/specs/academic/permissions.md:170-179`). The PHASE-3-HANDOFF § "Capability check boundary" acknowledges this is deferred to the dispatcher, but the audit checklist requires service-level assertion of capability.

**Expected:**

Every service function takes a `&dyn CapabilityCheck` parameter and asserts the required `Capability` before mutating the aggregate.

**Evidence:**

`crates/domains/academic/src/services.rs:90-1201` (entire file): no `CapabilityCheck` import, no `cap.has(...)` calls; only one comment in module docstring at line 14-15 acknowledging this. The `educore-rbac::services::CapabilityCheck` is only used in `crates/domains/cms/src/services.rs:35`.

---

### FINDING 3 (id: `DOMAIN-ACM-003`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/aggregate.rs:66-68`

**Description:**

The `Student` aggregate uses raw `String` for `first_name` and `last_name` instead of the typed value object `PersonName` defined in `value_objects.rs`. The spec mandates typed wrappers for names.

**Expected:**

`pub first_name: PersonName` and `pub last_name: PersonName`.

**Evidence:**

`crates/domains/academic/src/aggregate.rs:66-68` `pub first_name: String,\n    pub last_name: String,` vs. `crates/domains/academic/src/value_objects.rs:136-165` defining `PersonName` with validation.

---

### FINDING 4 (id: `DOMAIN-ACM-004`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/aggregate.rs:80-82, 84-86`

**Description:**

The `Student` aggregate uses `Option<String>` for `mobile`, `email`, `current_address`, `permanent_address`. The spec mandates typed wrappers `PhoneNumber`, `EmailAddress`, and `Address` (all available in `value_objects.rs:211, 258`).

**Expected:**

`pub mobile: Option<PhoneNumber>`, `pub email: Option<EmailAddress>`, `pub current_address: Option<Address>`, `pub permanent_address: Option<Address>`.

**Evidence:**

`crates/domains/academic/src/aggregate.rs:80-86` declares the four fields as `Option<String>`. `crates/domains/academic/src/value_objects.rs:211-253` defines `Address` and `crates/domains/academic/src/value_objects.rs:801-920` defines the reason wrappers — but no `PhoneNumber` or `EmailAddress` typed wrapper exists (see Finding DOMAIN-ACM-005).

---

### FINDING 6 (id: `DOMAIN-ACM-006`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/events.rs:53-78` (`StudentAdmitted`)

**Description:**

The `StudentAdmitted` event struct diverges from the spec. The spec mandates `admission_no: AdmissionNumber`, `full_name: FullName`, and `guardian_ids: Vec<GuardianId>`; the code carries raw `String` fields for admission_no, first_name, last_name and omits `guardian_ids` entirely.

**Expected:**

`pub admission_no: AdmissionNumber`, `pub full_name: FullName`, `pub guardian_ids: Vec<GuardianId>` (per `docs/specs/academic/events.md:39-49`).

**Evidence:**

- `docs/specs/academic/events.md:39-49` spec struct uses typed wrappers.
  - `crates/domains/academic/src/events.rs:53-78` actual struct: `pub admission_no: String,\n    pub first_name: String,\n    pub last_name: String,` and no `guardian_ids` field.

---

### FINDING 7 (id: `DOMAIN-ACM-007`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/events.rs:438-518` (`StudentPromoted`)

**Description:**

The `StudentPromoted` event struct is missing four fields the spec mandates and uses raw `String` for the roll number instead of the typed wrapper `RollNumber`. The spec lists `from_record_id`, `to_record_id`, `from_roll_no`, and `promotion_id`; the code omits all four.

**Expected:**

`pub from_record_id: StudentRecordId`, `pub to_record_id: StudentRecordId`, `pub from_roll_no: RollNumber`, `pub to_roll_no: RollNumber`, `pub promotion_id: StudentPromotionId`.

**Evidence:**

- `docs/specs/academic/events.md:158-174` spec struct.
  - `crates/domains/academic/src/events.rs:438-464` actual struct has `pub to_roll_no: String` and no `from_record_id`, `to_record_id`, `from_roll_no`, or `promotion_id`.

---

### FINDING 14 (id: `DOMAIN-ACM-014`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/value_objects.rs` (entire file)

**Description:**

Several closed status enums from `docs/specs/academic/value-objects.md` are missing: `LessonPlanStatus`, `ClassSectionStatus`, `HomeworkStatus`, `RegistrationType`, `CertificateLayout`, `CertificateType`, `GuardianRelation`, `AdmissionSource`, `AdmissionReference`, `NoOfChild`, `InquiryStatus`, `DayOfWeek`, `Semester`.

**Expected:**

Spec lines `docs/specs/academic/value-objects.md:87-100` lists 12 status enums; code has 4.

**Evidence:**

`docs/specs/academic/value-objects.md:87-100` lists 12 status enum rows; `crates/domains/academic/src/value_objects.rs` defines only `StudentStatus`, `Gender`, `BloodGroup`, `SubjectType`, `ResultStatus` (lines 671, 723, 756, 923, 950).

---

### FINDING 15 (id: `DOMAIN-ACM-015`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/repository.rs` (entire file)

**Description:**

Only 5 of the ~23 repository port traits from `docs/specs/academic/repositories.md` are defined. Missing: `GuardianRepository`, `ClassSectionRepository`, `ClassSubjectRepository`, `ClassRoutineRepository`, `HomeworkRepository`, `LessonRepository`, `LessonTopicRepository`, `LessonPlanRepository`, `StudentRecordRepository`, `StudentPromotionRepository`, `StudentCategoryRepository`, `StudentGroupRepository`, `RegistrationFieldRepository`, `CertificateRepository`, `IdCardRepository`, `AdmissionQueryRepository`, `ClassRoomRepository`, `ClassTimeRepository`.

**Expected:**

Every repository in the spec has a port trait.

**Evidence:**

`docs/specs/academic/repositories.md:7-234` lists ~23 repository traits; `crates/domains/academic/src/repository.rs` defines only `StudentRepository`, `ClassRepository`, `SectionRepository`, `SubjectRepository`, `AcademicYearRepository` (lines 43, 103, 126, 149, 172).

---

### FINDING 16 (id: `DOMAIN-ACM-016`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/repository.rs:43-95`

**Description:**

The `StudentRepository` trait is missing the `delete` method the spec mandates. The spec at `docs/specs/academic/repositories.md:16` declares `async fn delete(&self, id: StudentId) -> Result<()>;`, but the code's trait body has only `get`, `get_by_admission_no`, `get_by_email`, `list`, `list_by_status`, `list_in_class_section`, `insert`, `update`.

**Expected:**

`async fn delete(&self, id: StudentId) -> Result<()>;` per spec.

**Evidence:**

`docs/specs/academic/repositories.md:9-28` vs. `crates/domains/academic/src/repository.rs:43-95`.

---

### FINDING 17 (id: `DOMAIN-ACM-017`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/repository.rs:103-117, 126-140, 149-163, 172-190`

**Description:**

The `ClassRepository`, `SectionRepository`, `SubjectRepository`, and `AcademicYearRepository` traits are all missing the `delete` method the spec mandates. The spec at `docs/specs/academic/repositories.md:56-57, 68-69, 96-97, 125` declares a `delete` method for each of these aggregates.

**Expected:**

Each repository trait has an `async fn delete(&self, id: XxxId) -> Result<()>` method.

**Evidence:**

`docs/specs/academic/repositories.md:56-57, 68-69, 96-97, 125` vs. `crates/domains/academic/src/repository.rs` (each trait lacks `delete`).

---

### FINDING 18 (id: `DOMAIN-ACM-018`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/repository.rs:43-95`

**Description:**

The `StudentRepository` trait is missing the spec's optimized domain queries: `active_in_class`, `active_in_section`, `admitted_in_range`, `suspended`, `search_by_name`. The spec at `docs/specs/academic/repositories.md:23-27` lists these as required methods.

**Expected:**

`async fn active_in_class(...)`, `async fn active_in_section(...)`, `async fn admitted_in_range(...)`, `async fn suspended(...)`, `async fn search_by_name(...)`.

**Evidence:**

`docs/specs/academic/repositories.md:23-27` vs. `crates/domains/academic/src/repository.rs:43-95`.

---

### FINDING 19 (id: `DOMAIN-ACM-019`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/services.rs` (entire file)

**Description:**

None of the 8 domain services from `docs/specs/academic/services.md` are implemented: `AdmissionService`, `PromotionService`, `EnrollmentService`, `RoutineService`, `HomeworkService`, `LessonPlanService`, `GraduationService`, `ClassSectionAssignmentService`. The spec mandates these as standalone service structs with their own method signatures.

**Expected:**

`pub struct AdmissionService;`, `pub struct PromotionService;`, etc.

**Evidence:**

`docs/specs/academic/services.md:8-128` defines 8 services; `crates/domains/academic/src/services.rs` defines only per-command factory functions and one `school_matches` helper.

---

### FINDING 20 (id: `DOMAIN-ACM-020`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/services.rs` (entire file)

**Description:**

The spec mandates a `OptionalSubjectEligibility` policy at `docs/specs/academic/services.md:111-118` and the `ActiveStudentsInClass` and `PromotableStudents` Specifications at lines `122-141`. None are implemented.

**Expected:**

`pub struct OptionalSubjectEligibility;`, `pub struct ActiveStudentsInClass;`, `pub struct PromotableStudents;`.

**Evidence:**

`docs/specs/academic/services.md:111-141`; `crates/domains/academic/src/services.rs` has no such struct.

---

### FINDING 25 (id: `DOMAIN-ACM-025`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/services.rs:445-486` (`transfer_student`)

**Description:**

The `TransferStudentCommand` handler does not validate that `destination_school_id` is a "sibling school in same SaaS tenant" as the spec mandates (`docs/specs/academic/commands.md:198`). It only checks `destination_school_id != student.school_id`, leaving the SaaS-tenant relationship unverified.

**Expected:**

A check that `destination_school_id` is in the same SaaS tenant as `student.school_id` (requires a `SaaSContext` or similar port).

**Evidence:**

`crates/domains/academic/src/services.rs:464-468` only rejects equal school ids; `docs/specs/academic/commands.md:198` and `docs/specs/academic/workflows.md:71-85` require sibling-school validation.

---

### FINDING 26 (id: `DOMAIN-ACM-026`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/aggregate.rs:247-249`

**Description:**

Production code uses `unwrap_or_else(|_| unreachable!(...))`, which is a `panic!()`-equivalent anti-pattern that violates the engine rule "unwrap, expect, panic are forbidden in production paths" (`AGENTS.md` Agent Instructions → Type Safety).

**Expected:**

Use a checked constructor or `if/else` branch that returns `Result`.

**Evidence:**

`crates/domains/academic/src/aggregate.rs:247-249` `optional_subject_gpa_threshold: OptionalSubjectGpaThreshold::new(0.0).unwrap_or_else(\n                |_| unreachable!("0.0 is in the valid OptionalSubjectGpaThreshold range"),\n            ),`.

---

### FINDING 28 (id: `DOMAIN-ACM-028`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/aggregate.rs:98-101`

**Description:**

The `Student` aggregate declares `pub custom_fields: std::collections::BTreeMap<String, String>`. While this is a `BTreeMap` (not `HashMap`), the engine rule spirit is "no key-string-keyed domain data" and the field is unindexed/untyped domain data.

**Expected:**

A typed `CustomField { key: FieldName, value: FieldValue }` struct with validated `FieldName`.

**Evidence:**

`crates/domains/academic/src/aggregate.rs:100` `pub custom_fields: std::collections::BTreeMap<String, String>,` and the spec's `docs/specs/academic/commands.md:38` declares the same `BTreeMap<String, String>` — so the spec itself permits it, but the engine rule "No HashMap<String, T> for domain data" is structurally violated in the spec.

---

### FINDING 30 (id: `DOMAIN-ACM-030`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/aggregate.rs:50-55` (comment) vs. `crates/domains/academic/src/entities.rs:36-91` (entities placeholder)

**Description:**

The `Student` aggregate doc comment lists "StudentDocument, StudentTimeline, StudentHomework" as children, but only `StudentDocument` has a placeholder type. The Timeline and Homework entities are absent, contradicting the documented children.

**Expected:**

`StudentTimeline` and `StudentHomework` placeholder structs in `entities.rs` matching the pattern of `StudentDocument`.

**Evidence:**

`crates/domains/academic/src/aggregate.rs:52-55` lists all three; `crates/domains/academic/src/entities.rs:36-91` defines only `StudentDocumentId`, `StudentDocument`, `DocumentType`.

---

### FINDING 31 (id: `DOMAIN-ACM-031`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/entities.rs:39-64` (`StudentDocument`)

**Description:**

The `StudentDocument` placeholder struct is missing the `ActiveStatus`, `school_id` tenancy column on the variant-level state (only on `id`), and an `updated_at` / `updated_by` audit trail that the aggregate-level invariants require.

**Expected:**

`pub active_status: ActiveStatus,` and `pub created_by: UserId,` fields per the engine invariant "audit-first".

**Evidence:**

`crates/domains/academic/src/entities.rs:47-64` only declares `id`, `school_id`, `student_id`, `title`, `file_ref`, `document_type`, `created_at`. No `active_status`, no `updated_at`, no `created_by`/`updated_by`.

---

### FINDING 32 (id: `DOMAIN-ACM-032`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/academic/` (entire crate)

**Description:**

The crate has no integration test directory at `crates/domains/academic/tests/`. The Phase 3 hand-off places the integration test at `crates/tools/storage-parity/tests/academic_integration.rs`, but the build plan's "No-Gaps Gates" § 1 mandates per-domain tests at `crates/domains/<domain>/tests/` (`docs/build-plan.md:1834-1864`).

**Expected:**

`crates/domains/academic/tests/aggregate_fields.rs`, `commands.rs`, `events.rs`, `services.rs`, `repository.rs`, `value_objects.rs`, `workflows.rs` directories containing hand-written tests.

**Evidence:**

`docs/build-plan.md:1834-1864` and `ls /home/beznet/Workspace/smscore/crates/domains/academic/tests/` returning "No such file or directory".

---

### FINDING 33 (id: `DOMAIN-ACM-033`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/tools/storage-parity/tests/academic_integration.rs:444-491` (`academic_event_type_round_trip_for_all_aggregates`)

**Description:**

The integration test claims to exercise "all 5 prompt-named aggregates" but only constructs events for `Class` and `Section`. The `Subject` and `AcademicYear` aggregates are not exercised by any integration test. The `coverage.toml` rows for `academic_subjects_aggregate` and `academic_academic_years_aggregate` are marked `Tested` despite no actual test exercising them.

**Expected:**

`create_subject` and `create_academic_year` round-trip tests in the integration suite.

**Evidence:**

`crates/tools/storage-parity/tests/academic_integration.rs:444-491` only invokes `create_class` and `create_section`; `crates/tools/storage-parity/tests/academic_integration.rs:138` is the only `admit_student` test. `docs/coverage.toml:493-500` marks both aggregates `Tested`.

---

### FINDING 41 (id: `DOMAIN-ACM-041`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/services.rs:1226-1230` (test module imports) and `crates/domains/academic/src/services.rs` test functions

**Description:**

No service-level test exercises the spec's decision-table for the Student status transitions (Applicant → Active → {Suspended, Withdrawn, Graduated, Transferred}). The test `suspend_reinstate_withdraw_transfer_graduate_change_status` at line 1368 covers some transitions but not the rejection of invalid ones (e.g., calling `graduate_student` on an already-Withdrawn student should return `Err`).

**Expected:**

A parametrized decision-table test asserting that invalid status transitions return `Err(DomainError::Conflict(...))`.

**Evidence:**

`crates/domains/academic/src/services.rs:1368-1440` `suspend_reinstate_withdraw_transfer_graduate_change_status` only exercises the happy-path.

---

### FINDING 42 (id: `DOMAIN-ACM-042`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/services.rs` (entire file, test module)

**Description:**

None of the 9 workflows from `docs/specs/academic/workflows.md` (Admission, Promotion, Withdrawal, Transfer, Routine Construction, Homework, Lesson Plan, Admission Query, Class-Section Lifecycle) are exercised by any integration test or workflow test file.

**Expected:**

`crates/domains/academic/tests/workflows.rs` containing hand-written multi-step workflow tests, per the per-domain gate (`docs/build-plan.md:1864`).

**Evidence:**

`docs/specs/academic/workflows.md:6-148` defines 9 workflows; `crates/domains/academic/src/services.rs` tests do not span multi-step workflows.

---

### FINDING 5 (id: `DOMAIN-ACM-005`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/value_objects.rs` (entire file)

**Description:**

The `PhoneNumber` and `EmailAddress` typed value objects named in the spec and referenced by `Aggregate`/`Command` fields do not exist. The spec at `docs/specs/academic/value-objects.md:58-59` mandates them with E.164 / RFC 5322 validation, but the code only carries `validate_email_optional` and `validate_mobile_optional` as crate-private helper functions in `commands.rs:602-642` that operate on raw `&str`.

**Expected:**

`pub struct PhoneNumber(String)` and `pub struct EmailAddress(String)` with constructor-time validation.

**Evidence:**

`docs/specs/academic/value-objects.md:58-59` lists `EmailAddress` and `PhoneNumber`; `crates/domains/academic/src/value_objects.rs` has no matching struct (verified via `grep -n 'PhoneNumber\|EmailAddress' crates/domains/academic/src/value_objects.rs` returning no struct definitions). `crates/domains/academic/src/commands.rs:602-642` exposes only `pub(crate) fn validate_email_optional` / `validate_mobile_optional` helper functions.

---

### FINDING 55 (id: `DOMAIN-ACM-055`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/services.rs:1527-1547` (`promote_emits_event_with_from_to`) and `crates/domains/academic/src/services.rs:1790-1825` (`update_student_profile_changes_only_supplied_fields`)

**Description:**

The unit tests for `promote_student` and `update_student_profile` do not assert that the student aggregate's `class_id`, `section_id`, or `roll_no` were updated. Since `promote_student` takes `&Student` (not `&mut Student`), these fields are never mutated; the tests pass vacuously without verifying the spec's required state transition.

**Expected:**

Tests that fail when the mutation contract is violated.

**Evidence:**

`crates/domains/academic/src/services.rs:1542-1546` only asserts `event.from_class_id == class_a`, etc., but doesn't assert the student's class_id/section_id/roll_no are updated.

---

### FINDING 60 (id: `DOMAIN-ACM-060`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/services.rs:498-541` and `crates/domains/academic/src/services.rs:543-580`

**Description:**

`promote_student` does not emit `StudentRecordCreated` for the new academic year, and `graduate_student` does not emit `StudentMarkedGraduate`. Both events are required by the spec at `docs/events/academic.md:75, 268` and `docs/specs/academic/commands.md:232-249`.

**Expected:**

`StudentRecordCreated` and `StudentMarkedGraduate` events emitted as part of the corresponding service flows.

**Evidence:**

`crates/domains/academic/src/services.rs:498-541` (`promote_student`) emits only `StudentPromoted`; `crates/domains/academic/src/services.rs:543-580` (`graduate_student`) emits only `StudentGraduated`; `docs/events/academic.md:75, 268` and `docs/specs/academic/commands.md:232-249` require both.

---

### FINDING 9 (id: `DOMAIN-ACM-009`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/commands.rs:410-427` (`CreateAcademicYearCommand`)

**Description:**

`CreateAcademicYearCommand` carries `pub is_current: bool` and `pub academic_year_id: AcademicYearId` fields that the spec does not declare. The spec's `CreateAcademicYearCommand` does not have either field; the only way to mark a year current is via `SetCurrentAcademicYearCommand`.

**Expected:**

Spec at `docs/specs/academic/commands.md:359-367` lists only `tenant`, `year`, `title`, `starting_date`, `ending_date`, `copy_with_academic_year`.

**Evidence:**

`crates/domains/academic/src/commands.rs:413-424` declares `pub academic_year_id: AcademicYearId` and `pub is_current: bool` — neither appears in the spec struct.

---

### FINDING 22 (id: `DOMAIN-ACM-022`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/services.rs:498-541` (`promote_student`)

**Description:**

`promote_student` takes `student: &Student` (immutable reference) instead of `&mut Student`. The function does not mutate the student's `class_id`, `section_id`, or `roll_no` fields, contradicting the engine rule "the services module is the only place the engine mutates an aggregate and emits its typed event" (`services.rs:12-14`). The doc comment at lines 491-497 acknowledges this by stating "The function does not mutate the student record's class/section fields", which is itself a documentation-confessed spec violation.

**Expected:**

`pub fn promote_student<C, G>(student: &mut Student, ...)` that updates `student.class_id`, `student.section_id`, and emits the event.

**Evidence:**

`crates/domains/academic/src/services.rs:499` `student: &Student` and the admission doc comment at `services.rs:491-497` explicitly states the class/section fields are not mutated.

---

### FINDING 23 (id: `DOMAIN-ACM-023`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/services.rs:300-309, 666-682, 811-825, 956-973`

**Description:**

The update services (`update_student_profile`, `update_class`, `update_section`, `update_subject`) do not set `aggregate.updated_by` after mutating the aggregate, even though the aggregate structs declare `pub updated_by: UserId`. The audit metadata is therefore stale after every update.

**Expected:**

`aggregate.updated_by = ctx.actor_id;` after each `updated_at = now;` assignment.

**Evidence:**

- `crates/domains/academic/src/services.rs:301-304` only sets `student.updated_at`, `student.version`, `student.last_event_id`.
  - `crates/domains/academic/src/services.rs:666-669` only sets `class.updated_at`, `class.version`, `class.last_event_id`.
  - `crates/domains/academic/src/services.rs:811-814` only sets `section.updated_at`, `section.version`, `section.last_event_id`.
  - `crates/domains/academic/src/services.rs:956-959` only sets `subject.updated_at`, `subject.version`, `subject.last_event_id`.
  - The aggregate fields exist: `crates/domains/academic/src/aggregate.rs:112, 217, 291, 362, 448` declare `pub updated_by: UserId`.

---

### FINDING 24 (id: `DOMAIN-ACM-024`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/services.rs:1101-1135` (`set_current_academic_year`)

**Description:**

The `set_current_academic_year` service sets `year_agg.is_current = true` but does not enforce the spec invariant "Exactly one academic year may be marked `Current` per school at a time" (`docs/specs/academic/aggregates.md:282`). The service always passes `None` for `previous_id` in the `CurrentAcademicYearSet` event, and defers the demotion to the storage adapter — but no service-level check ensures only one year per school is current.

**Expected:**

The service must check `if any other AcademicYear has is_current == true` and return `Err(DomainError::Conflict(...))`, or pass the demoted year id as `previous_id`.

**Evidence:**

`crates/domains/academic/src/services.rs:1097-1135` and `docs/specs/academic/aggregates.md:282` invariant.

---

### FINDING 27 (id: `DOMAIN-ACM-027`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/aggregate.rs:637-641`

**Description:**

The aggregate test module uses `.expect(...)` five times. While the `#[cfg(test)]` block relaxes the clippy lint, the crate-wide `#![deny(missing_docs)]` and the engine rule against `expect()` in production paths would still flag this if the test helper were promoted to a shared `non_panicking` constructor.

**Expected:**

`Etag::placeholder()` returning a typed `Etag` directly without re-validation in tests.

**Evidence:**

`crates/domains/academic/src/aggregate.rs:637-641` `Etag::new(Student::FRESH_ETAG).expect("FRESH_ETAG must be a valid etag");` × 5.

---

### FINDING 29 (id: `DOMAIN-ACM-029`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/value_objects.rs:671-690` (`StudentStatus`)

**Description:**

The `StudentStatus::Applicant` variant is declared and round-trip-tested but is unreachable from any service function. `Student::fresh()` always sets `StudentStatus::Active`. The spec at `docs/specs/academic/aggregates.md:33` mandates the transition `Applicant → Active → ...`; Phase 3 has no way to create an applicant.

**Expected:**

Either a `RegisterApplicant` service that sets `StudentStatus::Applicant` or removal of the variant until admission queries land.

**Evidence:**

`crates/domains/academic/src/value_objects.rs:675` declares `Applicant`,; `crates/domains/academic/src/aggregate.rs:166` `status: crate::value_objects::StudentStatus::Active`; `crates/domains/academic/src/services.rs:90-190` (admit_student) — no Applicant path.

---

### FINDING 34 (id: `DOMAIN-ACM-034`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `docs/coverage.toml:511-516`

**Description:**

The `admit_student_command` coverage row is marked `Pending`, but the command struct is implemented at `crates/domains/academic/src/commands.rs:64-106` and is exercised by `crates/tools/storage-parity/tests/academic_integration.rs:226-240`. The matrix does not match the code's actual status.

**Expected:**

`status = "Tested"` with the test path.

**Evidence:**

`docs/coverage.toml:511-516` `status = "Pending"` for `admit_student_command`; implementation at `crates/domains/academic/src/commands.rs:64-106`; test path exercised at `crates/tools/storage-parity/tests/academic_integration.rs:226-240`.

---

### FINDING 35 (id: `DOMAIN-ACM-035`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `docs/coverage.toml:534-540`

**Description:**

The `student_admitted_event` coverage row is marked `Pending`, but the event struct is implemented at `crates/domains/academic/src/events.rs:53-132` and is exercised end-to-end by the integration test (event payload asserted at `crates/tools/storage-parity/tests/academic_integration.rs:265`).

**Expected:**

`status = "Tested"`.

**Evidence:**

`docs/coverage.toml:534-540` `status = "Pending"` for `student_admitted_event`; event code at `crates/domains/academic/src/events.rs:53-132`; test at `crates/tools/storage-parity/tests/academic_integration.rs:265`.

---

### FINDING 36 (id: `DOMAIN-ACM-036`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `docs/coverage.toml:502-509` (`academic_enrollments_aggregate`)

**Description:**

The `academic_enrollments_aggregate` coverage row references an "enrollments aggregate" that is not in the spec at `docs/specs/academic/aggregates.md`. The closest spec concept is `StudentRecord` (the per-academic-year enrollment), but the coverage row's item name is misleading.

**Expected:**

Either rename the row to `academic_student_records_aggregate` (matching spec) or remove the row.

**Evidence:**

`docs/coverage.toml:502-509` `item = "academic_enrollments aggregate"`; `docs/specs/academic/aggregates.md:472-507` defines `StudentRecord` as the per-year enrollment.

---

### FINDING 37 (id: `DOMAIN-ACM-037`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `docs/coverage.toml:526-532` (`promote_students_command`)

**Description:**

The coverage row is named `promote_students_command` (plural), but the actual command struct is `PromoteStudentCommand` (singular) at `crates/domains/academic/src/commands.rs:242-259`. The names don't match, and a `grep` of `docs/commands/academic.md` shows the canonical command is `PromoteStudent` (singular).

**Expected:**

Row id `promote_student_command` matching the spec's `PromoteStudent` command and the code's `PromoteStudentCommand`.

**Evidence:**

`docs/coverage.toml:526-532` `id = "promote_students_command"` (plural); `docs/commands/academic.md:29` `PromoteStudent` (singular); `crates/domains/academic/src/commands.rs:242` `pub struct PromoteStudentCommand` (singular).

---

### FINDING 38 (id: `DOMAIN-ACM-038`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `docs/coverage.toml` (entire file)

**Description:**

The coverage matrix is missing rows for the 17 implemented command structs and 22 implemented event structs that are not in the matrix. Examples (not exhaustive): `update_student_profile_command`, `suspend_student_command`, `reinstate_student_command`, `withdraw_student_command`, `transfer_student_command`, `graduate_student_command`, `create_class_command`, `update_class_command`, `set_optional_subject_gpa_threshold_command`, `delete_class_command`, `create_section_command`, `update_section_command`, `delete_section_command`, `create_subject_command`, `update_subject_command`, `delete_subject_command`, `create_academic_year_command`, `update_academic_year_dates_command`, `set_current_academic_year_command`, `close_academic_year_command`; plus event rows like `student_suspended_event`, `student_reinstated_event`, `student_withdrawn_event`, `student_transferred_event`, `student_promoted_event`, `student_graduated_event`, `student_profile_updated_event`, `class_created_event`, `class_updated_event`, `class_deleted_event`, `optional_subject_gpa_threshold_set_event`, `section_created_event`, `section_updated_event`, `section_deleted_event`, `subject_created_event`, `subject_updated_event`, `subject_deleted_event`, `academic_year_created_event`, `academic_year_dates_updated_event`, `current_academic_year_set_event`, `academic_year_closed_event`, `academic_year_copied_event`.

**Expected:**

Every code-defined command and event has a corresponding coverage row.

**Evidence:**

`crates/domains/academic/src/commands.rs` defines 22 command structs (counted); `docs/coverage.toml` only has 3 command rows for academic.

---

### FINDING 43 (id: `DOMAIN-ACM-043`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/commands.rs:153-154`

**Description:**

`AdmitStudentCommand::school_id()` is the only command with a `school_id()` helper; the other 21 commands require callers to manually read `cmd.tenant.school_id` or the typed id's `.school_id()`. This is an inconsistency in the public command shape API.

**Expected:**

Either all commands have a `school_id()` helper or none do.

**Evidence:**

`crates/domains/academic/src/commands.rs:149-154` defines `pub fn school_id()` only on `AdmitStudentCommand`; no other command struct defines it.

---

### FINDING 44 (id: `DOMAIN-ACM-044`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/events.rs:32` (`#![allow(clippy::too_many_arguments)]`)

**Description:**

The events module carries a crate-level `#![allow(clippy::too_many_arguments)]` to silence a clippy warning. Every event constructor takes 6-12 arguments; this is a structural smell that suggests builder or factory helpers are warranted.

**Expected:**

Builders (`StudentAdmitted::builder()`) or named-argument structs.

**Evidence:**

`crates/domains/academic/src/events.rs:32` `#![allow(clippy::too_many_arguments)]` and constructor methods with 10-12 positional args (e.g. `StudentPromoted::new` at lines 470-498).

---

### FINDING 46 (id: `DOMAIN-ACM-046`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/commands.rs:432-435`

**Description:**

`CreateAcademicYearCommand::range()` recomputes the `AcademicYearRange` validation that `create_academic_year` (services.rs:1031) performs again via `AcademicYearRange::new(starting_date, ending_date)?`. The duplicate call wastes work and means validation logic is split between the command and the service.

**Expected:**

The service should call `cmd.range()?` to reuse the command's validation.

**Evidence:**

`crates/domains/academic/src/commands.rs:432-435` vs. `crates/domains/academic/src/services.rs:1031` — both call `AcademicYearRange::new(starting_date, ending_date)`.

---

### FINDING 49 (id: `DOMAIN-ACM-049`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `docs/specs/academic/tables.md:7-58` (entire table list)

**Description:**

The spec lists 50 academic tables, but the implementation has no `#[derive(DomainQuery)]` struct in `crates/domains/academic/src/entities.rs`. The entities.rs file is a placeholder with only `StudentDocumentId`, `StudentDocument`, `DocumentType` shells.

**Expected:**

Per `docs/build-plan.md:1875-1882`, every `tables.md` row should correspond to a `#[derive(DomainQuery)]` struct in `entities.rs`. None exist in the academic crate.

**Evidence:**

`docs/specs/academic/tables.md:7-58` lists 50 tables; `crates/domains/academic/src/entities.rs` has no `#[derive(DomainQuery)]` (verified by `grep 'DomainQuery' crates/domains/academic/src/entities.rs` returning no matches).

---

### FINDING 50 (id: `DOMAIN-ACM-050`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/commands.rs:64-106` (`AdmitStudentCommand`) vs. `docs/specs/academic/commands.md:13-39`

**Description:**

`AdmitStudentCommand` is missing the `student_category_id: Option<StudentCategoryId>`, `student_group_ids: Vec<StudentGroupId>`, `guardians: Vec<GuardianSpec>`, `transport: Option<TransportSpec>`, `hostel: Option<HostelSpec>`, and `documents: Vec<DocumentSpec>` fields the spec mandates. The Phase 3 hand-off § OQ #6 acknowledges this as "scoped out" but the audit checklist requires spec-vs-code drift reporting.

**Expected:**

Fields `student_category_id`, `student_group_ids`, `guardians`, `transport`, `hostel`, `documents` per the spec struct.

**Evidence:**

`docs/specs/academic/commands.md:13-39` lists all 6; `crates/domains/academic/src/commands.rs:64-106` has none of them.

---

### FINDING 51 (id: `DOMAIN-ACM-051`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/commands.rs:157-180` (`UpdateStudentProfileCommand`) vs. `docs/specs/academic/commands.md:57-61`

**Description:**

`UpdateStudentProfileCommand` flattens the spec's `StudentProfilePatch` sub-struct. The spec mandates a nested `pub patch: StudentProfilePatch` field; the code declares the patch fields inline on the command itself.

**Expected:**

`pub struct StudentProfilePatch { ... }` and `pub patch: StudentProfilePatch` on the command.

**Evidence:**

`docs/specs/academic/commands.md:57-61` spec struct; `crates/domains/academic/src/commands.rs:157-180` actual struct (no nested patch).

---

### FINDING 52 (id: `DOMAIN-ACM-052`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/commands.rs:262-272` (`GraduateStudentCommand`) vs. `docs/specs/academic/commands.md:238-246`

**Description:**

`GraduateStudentCommand` is missing the `pub destination: Option<GraduateDestination>` field the spec mandates.

**Expected:**

`pub destination: Option<GraduateDestination>`.

**Evidence:**

`docs/specs/academic/commands.md:238-246` lists the field; `crates/domains/academic/src/commands.rs:262-272` omits it.

---

### FINDING 53 (id: `DOMAIN-ACM-053`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/aggregate.rs:672-718`

**Description:**

The `Class` aggregate is missing the `capacity: Option<u32>` field the spec implies (`docs/specs/academic/workflows.md:151-155` mentions "A class may have a `Capacity` (a domain configuration value, not a hard invariant)"). The workflows spec says admission may be configured to reject when capacity is exceeded; the absence of the field makes capacity enforcement impossible.

**Expected:**

`pub capacity: Option<u32>` on `Class` with optional capacity check at `create_class` / `update_class` services.

**Evidence:**

`docs/specs/academic/workflows.md:151-155`; `crates/domains/academic/src/aggregate.rs:193-224` (`Class` struct) has no `capacity`.

---

### FINDING 54 (id: `DOMAIN-ACM-054`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/aggregate.rs:272-298` (`Section`)

**Description:**

The `Section` aggregate has no `capacity` field. The spec invariant at `docs/specs/academic/workflows.md:151-155` allows per-section capacity; absent field.

**Expected:**

Optional `capacity: Option<u32>`.

**Evidence:**

`crates/domains/academic/src/aggregate.rs:272-298`; `docs/specs/academic/workflows.md:151-155`.

---

### FINDING 56 (id: `DOMAIN-ACM-056`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/query.rs:116-122, 152-158, 188-194, 244-250, 310-316` (all 5 `execute` stubs)

**Description:**

All 5 query stubs (`StudentQuery::execute`, `ClassQuery::execute`, `SectionQuery::execute`, `SubjectQuery::execute`, `AcademicYearQuery::execute`) return `Err(DomainError::not_supported(...))`. The Phase 3 hand-off acknowledges this is deferred to Phase 4+, but no consumer can run a query against the academic domain without an immediate failure.

**Expected:**

At minimum, an in-memory executor for unit tests.

**Evidence:**

`crates/domains/academic/src/query.rs:118-121` and 4 sibling sites.

---

### FINDING 58 (id: `DOMAIN-ACM-058`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/aggregate.rs:1-643` (entire file) vs. `crates/domains/academic/src/value_objects.rs:801-920` (`SuspensionReason`, `WithdrawalReason`, `TransferReason`)

**Description:**

The `Student` aggregate stores `suspension_reason`, `withdrawal_reason`, and `transfer_reason` only on the events, not as persistent fields on the aggregate. After a suspension, withdrawal, or transfer, the aggregate's `reason` is not queryable — consumers must read the event log.

**Expected:**

`pub last_suspension_reason: Option<SuspensionReason>` etc. fields on `Student` for audit-trail support.

**Evidence:**

`crates/domains/academic/src/aggregate.rs:57-119` `Student` struct fields enumerated; no reason fields.

---

### FINDING 59 (id: `DOMAIN-ACM-059`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/services.rs:174-188`

**Description:**

`admit_student` emits `StudentAdmitted` but never persists the corresponding `StudentRecord` row. The spec at `docs/specs/academic/commands.md:51` says "Creates a `Student`, one `StudentRecord` for the academic year, guardian links, ...". The code emits no `StudentRecordCreated` event because the `StudentRecord` aggregate does not exist in Phase 3.

**Expected:**

A `StudentRecord` aggregate and `StudentRecordCreated` event emitted alongside `StudentAdmitted`.

**Evidence:**

`crates/domains/academic/src/services.rs:175-188` only emits `StudentAdmitted`; `docs/specs/academic/commands.md:51` mandates both `StudentAdmitted` and `StudentRecordCreated`.

---

### FINDING 61 (id: `DOMAIN-ACM-061`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/aggregate.rs:247-249` and `crates/domains/academic/src/services.rs:1008-1058`

**Description:**

`CreateAcademicYearCommand::is_current` is read by `create_academic_year` (services.rs:1042) and used to set the new year's `is_current` flag, but no other year in the school is demoted. The spec at `docs/specs/academic/commands.md:372-373` mandates "`SetCurrentAcademicYear` is the only command that mutates the `Current` flag". By allowing `CreateAcademicYear` to set `is_current`, the code violates the invariant that `SetCurrentAcademicYear` is the sole mutator.

**Expected:**

`CreateAcademicYearCommand::is_current` should not exist; callers must use `SetCurrentAcademicYearCommand` after creation.

**Evidence:**

`crates/domains/academic/src/services.rs:1042` `year_agg.is_current = is_current;` vs. `docs/specs/academic/commands.md:372-373`.

---

### FINDING 63 (id: `DOMAIN-ACM-063`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/events.rs:502-518` (`StudentPromoted` `EVENT_TYPE`)

**Description:**

The `StudentPromoted` event `EVENT_TYPE` constant is `"academic.student.promoted"`, but the events catalog at `docs/events/academic.md:22` uses the same form. However, the spec at `docs/specs/academic/events.md:158-174` describes the event semantics — and crucially, the `StudentPromoted` event is intended to drive downstream subscribers in finance, attendance, and assessment (`docs/specs/academic/overview.md:105-111`). The missing `promotion_id` and `from_record_id`/`to_record_id` fields (Finding DOMAIN-ACM-007) mean downstream consumers cannot correlate the promotion with the new `StudentRecord`.

**Expected:**

Restored `from_record_id`, `to_record_id`, `from_roll_no`, and `promotion_id` fields.

**Evidence:**

`crates/domains/academic/src/events.rs:438-518` actual struct; `docs/specs/academic/overview.md:105-111` cross-domain consumer contracts.

---

### FINDING 65 (id: `DOMAIN-ACM-065`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/services.rs:174-189`

**Description:**

`admit_student` mutates the `Student` aggregate's `correlation_id` to `ctx.correlation_id` (line 173) but later emits the event with `ctx.correlation_id` (line 186). The aggregate stores the new value but the event uses the same value. There is no test asserting that the aggregate's correlation_id equals the event's correlation_id, but more importantly, the code never checks that `cmd.tenant.correlation_id` matches `student.correlation_id` — a tenant-context mismatch would be silently accepted.

**Expected:**

A debug-assert or validation that `cmd.tenant.school_id == student_id.school_id`.

**Evidence:**

`crates/domains/academic/src/services.rs:163, 173, 186`.

---

### FINDING 66 (id: `DOMAIN-ACM-066`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/services.rs:205-310` (`update_student_profile`)

**Description:**

`update_student_profile` ignores the `cmd.tenant` field (it uses `_ctx: &TenantContext` and `tenant: _`). The service relies on the dispatcher to perform the capability check; without it, the function can be invoked by anyone holding a `&mut Student`. The audit gap is that the function does not even sanity-check `student_id == student.id` at runtime (it uses `debug_assert_eq!` at line 229).

**Expected:**

A `Result`-returning assertion (not `debug_assert_eq!`) of `cmd.student_id == student.id`.

**Evidence:**

`crates/domains/academic/src/services.rs:206-229` `pub fn update_student_profile<C, G>(_ctx: &TenantContext, ...)` and `debug_assert_eq!(student_id, student.id);` (line 229).

---

### FINDING 8 (id: `DOMAIN-ACM-008`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/events.rs:522-561` (`StudentGraduated`)

**Description:**

The `StudentGraduated` event carries a `pub status: StudentStatus` field that is not in the spec.

**Expected:**

Spec at `docs/specs/academic/events.md:184-189` lists only `student_id`, `academic_year_id`, `graduation_date`.

**Evidence:**

`crates/domains/academic/src/events.rs:530` `pub status: StudentStatus,` vs. spec at `docs/specs/academic/events.md:184-189` which has no `status` field.

---

### FINDING 39 (id: `DOMAIN-ACM-039`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/services.rs:1269-1296`

**Description:**

The test helper `InMemoryUniqueness::record_email` (and the field `emails` it pushes to) is annotated `#[allow(dead_code)]` because no test exercises the email-uniqueness code path. This indicates a missing test case for the documented email-uniqueness invariant of the `admit_student` service.

**Expected:**

A test that records an email, then asserts `admit_student` with the same email returns `Err(DomainError::Conflict(...))`.

**Evidence:**

`crates/domains/academic/src/services.rs:1273-1278` `#[allow(dead_code)]\n        fn record_email(...)`; `crates/domains/academic/src/services.rs:1348-1366` `admit_student_uniqueness_violation` only tests admission_no, not email.

---

### FINDING 40 (id: `DOMAIN-ACM-040`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/services.rs:1273-1296`

**Description:**

`InMemoryUniqueness::student_email_exists` always lower-cases the input, but the code at `services.rs:138-145` lowercases via `let lower = e.to_lowercase();` before passing it. The double-lowercasing is benign but the contract should be that the caller passes a lowercased string (per the doc at `commands.rs:54-55`).

**Expected:**

A test that verifies the email-uniqueness comparison is case-insensitive end-to-end.

**Evidence:**

`crates/domains/academic/src/commands.rs:54-55` documents "The check is case-insensitive; the caller is responsible for lowercasing before the call." `crates/domains/academic/src/services.rs:1289-1296` lowercases anyway.

---

### FINDING 45 (id: `DOMAIN-ACM-045`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/commands.rs:472-722`

**Description:**

The `pub(crate)` validation helpers (`validate_first_name`, `validate_email_optional`, etc.) duplicate validation logic that already exists as typed-value-object constructors in `value_objects.rs:145-156, 220-232, 272-286, 320-332, 366-378, 410-422, 456-468, 502-514, 808-826, 848-866, 888-906`. The commands take raw strings and validate them, instead of taking typed wrappers and reusing the value-object constructors.

**Expected:**

`AdmitStudentCommand` carries `first_name: PersonName`, etc.

**Evidence:**

`crates/domains/academic/src/commands.rs:472-722` exposes 15+ `pub(crate) fn validate_*` helpers; `crates/domains/academic/src/value_objects.rs` already provides typed constructors that perform the same validation.

---

### FINDING 47 (id: `DOMAIN-ACM-047`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/services.rs:1207-1213` (`school_matches`)

**Description:**

The `school_matches` helper is documented as "Used by command dispatchers (outside the services module) to enforce the `school_id` invariant at the command boundary", but it is defined inside `services.rs` and re-exported from `lib.rs:88`. Its location contradicts its documented intent.

**Expected:**

Move `school_matches` to a `tenant_helpers.rs` or `guards.rs` module consumed by the dispatcher.

**Evidence:**

`crates/domains/academic/src/services.rs:1207-1213` docstring says "Used by command dispatchers (outside the services module)".

---

### FINDING 48 (id: `DOMAIN-ACM-048`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** `docs/handoff/PHASE-3-HANDOFF.md:99-103`

**Description:**

The hand-off claims "66 unit tests in `educore-academic`". The actual count is 67 unit tests (aggregate.rs:6 + commands.rs:10 + entities.rs:2 + errors.rs:0 + events.rs:4 + lib.rs:4 + query.rs:5 + repository.rs:1 + services.rs:16 + value_objects.rs:19).

**Expected:**

Self-consistent test count.

**Evidence:**

`docs/handoff/PHASE-3-HANDOFF.md:99-103` says "66 unit tests". Actual: 67.

---

### FINDING 57 (id: `DOMAIN-ACM-057`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/value_objects.rs:179-208` (`FullName`)

**Description:**

`FullName` is defined and re-exported in `lib.rs:123`, `lib.rs:183`, but is not referenced by any aggregate field, command struct, or event struct. The spec at `docs/specs/academic/events.md:43` mandates `pub full_name: FullName` on `StudentAdmitted`; the code at `events.rs:57-61` instead declares separate `first_name` + `last_name` strings.

**Expected:**

`pub full_name: FullName` on `StudentAdmitted`; corresponding field on `Student`.

**Evidence:**

`crates/domains/academic/src/value_objects.rs:179-208` defines `FullName`; `crates/domains/academic/src/events.rs:53-78` `StudentAdmitted` does not reference it; `crates/domains/academic/src/aggregate.rs:57-119` `Student` does not reference it.

---

### FINDING 62 (id: `DOMAIN-ACM-062`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/lib.rs:115-117`

**Description:**

The `UniquenessChecker` port is re-exported at the crate root via `pub use crate::commands::UniquenessChecker;` but the module-level docstring at `lib.rs:31` does not document this port. Consumers reading the module doc may miss the port.

**Expected:**

Module doc lists `UniquenessChecker` as a public surface.

**Evidence:**

`crates/domains/academic/src/lib.rs:31` lists "errors" but not the UniquenessChecker port; `lib.rs:115-117` re-exports it.

---

### FINDING 64 (id: `DOMAIN-ACM-064`)

- **Source:** `docs/audit_reports/findings/wave1-academic.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** `crates/domains/academic/src/repository.rs:42-95`

**Description:**

The `StudentRepository` trait methods accept `&TenantContext` for tenant scoping (`docs/specs/academic/repositories.md:8-14` describes methods as `async fn get(&self, id: StudentId) -> Result<Option<Student>>` — no `TenantContext`). The code adds `&TenantContext` to most methods, but the spec's intent (matched to `docs/specs/academic/repositories.md:7-29`) is that the school id is part of the typed id and the repository enforces tenancy via `school_id` predicates, not via a `TenantContext` parameter.

**Expected:**

`async fn get(&self, id: StudentId)` per spec; the typed id's `school_id` field already carries the tenant.

**Evidence:**

`docs/specs/academic/repositories.md:9-28` spec signatures; `crates/domains/academic/src/repository.rs:47` `async fn get(&self, ctx: &TenantContext, id: StudentId)`.

---


## Assessment (target id prefix: `DOMAIN-ASS`)

**Path:** `crates/domains/assessment/`  
**Total findings:** 100 (19 critical, 56 high, 25 medium, 0 low)


### FINDING 1 (id: `DOMAIN-ASS-001`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/aggregate.rs` (entire file) and `docs/specs/assessment/aggregates.md:1-1025`

**Description:**

The aggregate file ships 6 root structs (`Exam`, `ExamSchedule`, `SeatPlan`, `AdmitCard`, `MarksRegister`, `ResultStore`) but is missing 33 of the 39 aggregate roots declared in the spec. The spec at `docs/specs/assessment/aggregates.md` defines 39 aggregates; the code at `crates/domains/assessment/src/aggregate.rs` contains 6. The Phase 4 build-plan exit criterion (line 653) requires every aggregate in `docs/specs/assessment/aggregates.md` to have a Rust struct + tests.

**Expected:**

Rust struct + tests for `ExamType`, `ExamSetup`, `MarksGrade`, `MarkStore`, `MarkStoreEntry`, `ResultSetting`, `TemporaryMeritList`, `MeritPosition`, `ExamWisePosition`, `AllExamWisePosition`, `CustomResultSetting`, `CustomTemporaryResult`, `ExamStepSkip`, `ExamRoutinePage`, `FrontendExamRoutine`, `FrontendResult`, `FrontendExamResult`, `OnlineExam`, `QuestionBank`, `QuestionGroup`, `QuestionLevel`, `QuestionAssignment`, `OnlineExamQuestion`, `QuestionMuOption`, `OnlineExamMark`, `OnlineExamStudentAnswerMarking`, `StudentTakeOnlineExam`, `SeatPlanSetting`, `AdmitCardSetting`, `TeacherEvaluation`, `TeacherRemark`, `ExamAttendance`, `ExamAttendanceChild`.

**Evidence:**

`crates/domains/assessment/src/aggregate.rs` declares 6 `pub struct` (lines 58, 258, 360, 432, 504, 570). `docs/specs/assessment/aggregates.md` lists 39 aggregates (lines 1-1025); the table at `docs/specs/assessment/overview.md:96-140` lists 44 rows.

---

### FINDING 10 (id: `DOMAIN-ASS-010`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/repository.rs:217-235` and `docs/specs/assessment/repositories.md:136-178`

**Description:**

The shipped `ResultRepository` is missing 9 methods declared in the spec. The spec defines `list_for_setup`, `list_for_class_section`, `insert_merit`, `list_merit`, `insert_exam_position`, `list_exam_position`, `insert_all_exam_position`, `list_all_exam_position`, `insert_custom_temporary`, `list_custom_temporary`, `clear_custom_temporary`; the code has only `get`, `list_for_student`, `list_for_exam`, `insert`, `update`.

**Expected:**

14 methods on the `ResultRepository` trait per spec.

**Evidence:**

`crates/domains/assessment/src/repository.rs:217-235` has 5 methods. `docs/specs/assessment/repositories.md:138-177` has 16 methods.

---

### FINDING 100 (id: `DOMAIN-ASS-100`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:351, 1274, 1012-1014, 1080-1100, 1140-1148` and `docs/code-standards.md`

**Description:**

The `unwrap_or(u32::MAX)` pattern at `services.rs:351, 1274` and the `Uuid::nil()` placeholders at `services.rs:1012-1014, 1080-1100, 1140-1148` represent silent fallbacks that hide data-integrity errors. The `unwrap_or(u32::MAX)` saturates the count to `4_294_967_295` if the actual count overflows `u32`; downstream consumers (finance, attendance, communication) cannot distinguish a real count from a saturated one. The `Uuid::nil()` placeholders are a data-corruption risk: storage adapters will write a valid row with `exam_id = 00000000-...` and downstream subscribers (communication, finance, cms, academic) will silently fail to correlate.

**Expected:**

Propagate the overflow / missing-id errors as `Result::Err(DomainError::validation(...))` instead of silently saturating or substituting nil UUIDs.

**Evidence:**

`crates/domains/assessment/src/services.rs:351` `u32::try_from(_cmd.subjects.len()).unwrap_or(u32::MAX)`. `crates/domains/assessment/src/services.rs:1274` `current_rank += u32::try_from(j - i + 1).unwrap_or(u32::MAX)`. `services.rs:1012-1014` `let _placeholder_exam = ExamId::new(cmd.marks_register_id.school_id(), uuid::Uuid::nil());` (and parallel lines). `services.rs:1143` `ExamId::new(cmd.result_store_id.school_id(), uuid::Uuid::nil())`.

---

### FINDING 2 (id: `DOMAIN-ASS-002`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/aggregate.rs` (no `OnlineExam` struct) and `docs/specs/assessment/aggregates.md:471-518`

**Description:**

The `OnlineExam` aggregate is declared in the spec as one of the 8 prompt-named Phase 4 aggregates (`docs/build-plan.md:629-631`) but is missing from `aggregate.rs`. The handoff at `docs/handoff/PHASE-4-HANDOFF.md:67-71` admits the full state machine ships only at the Event level. The aggregate (with `Status`, `IsTaken`, `IsClosed`, `IsWaiting`, `IsRunning`, `AutoMark` lifecycle fields) has no struct, no command, no service, no repository, no query.

**Expected:**

`pub struct OnlineExam { id: OnlineExamId, status: OnlineExamStatus, is_taken: bool, is_closed: bool, is_waiting: bool, is_running: bool, auto_mark: bool, start_time: ..., end_time: ..., end_date_time: ..., ... }` per `docs/specs/assessment/aggregates.md:471-518`.

**Evidence:**

`docs/build-plan.md:629-631` lists `OnlineExam` as a Phase 4 aggregate; `crates/domains/assessment/src/aggregate.rs` has no `pub struct OnlineExam`; `docs/handoff/PHASE-4-HANDOFF.md:67-71` "The full state machine ships at the Event level (8 events) but the integration test only exercises `create_exam` per the user-chosen scope."

---

### FINDING 28 (id: `DOMAIN-ASS-028`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:1200-1201` (production code) and `docs/code-standards.md` engine rules

**Description:**

The `ResultService::compute_grade` production method (a non-test public method on the `ResultService` struct) uses `.expect("valid grade")` and `.expect("valid gpa")`. The engine rule in `AGENTS.md` / `docs/code-standards.md` forbids `expect()` in production paths because the input space is a school-defined grade scale and the constructor may legitimately fail.

**Expected:**

Propagate the `Result<...>` from `Grade::new` and `Gpa::new` instead of panicking on the constructor.

**Evidence:**

`crates/domains/assessment/src/services.rs:1182-1203` `pub fn compute_grade(percent: f32) -> (crate::value_objects::Grade, crate::value_objects::Gpa) { ... let g = crate::value_objects::Grade::new(g_str).expect("valid grade"); let gpa = crate::value_objects::Gpa::new(gpa_val).expect("valid gpa"); ... }`.

---

### FINDING 29 (id: `DOMAIN-ASS-029`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:1011-1027` (production code) and `docs/specs/assessment/commands.md:235-251`

**Description:**

The `submit_marks` production function mints `MarksSubmitted` with `Uuid::nil()` placeholders for `exam_id`, `class_id`, and `section_id` (lines 1012-1016). The `marks_register_id` only carries the school anchor and the local UUID; the spec's `MarksSubmitted` event requires the per-exam broadcast (which is how downstream `ResultService::publish` knows which `(exam, class, section)` to compute results for). Using nil UUIDs is a data-integrity bug: storage adapters will write the `MarksSubmitted` event with `exam_id = 00000000-...` and downstream consumers will silently fail to correlate.

**Expected:**

Resolve the `(exam, class, section)` from a `MarksRegister` aggregate lookup before minting the event.

**Evidence:**

`crates/domains/assessment/src/services.rs:1011-1027` `let _placeholder_exam = ExamId::new(cmd.marks_register_id.school_id(), uuid::Uuid::nil()); let _placeholder_class = educore_academic::ClassId::new(cmd.marks_register_id.school_id(), uuid::Uuid::nil()); let _placeholder_section = educore_academic::SectionId::new(cmd.marks_register_id.school_id(), uuid::Uuid::nil()); Ok(MarksSubmitted::new(cmd.marks_register_id, _placeholder_exam, _placeholder_class, _placeholder_section, 0, event_id, ...))`. Comment at line 1011: `// Phase 4 stub: the per-exam broadcast is empty.`

---

### FINDING 3 (id: `DOMAIN-ASS-003`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/commands.rs` (entire file) and `docs/specs/assessment/commands.md:1-817`

**Description:**

The commands file ships 21 command structs but is missing 47 of the 62 commands declared in the spec catalog (`docs/commands/assessment.md`) and the per-command spec (`docs/specs/assessment/commands.md`). The 21 shipped are the 3 Exam workstream-A commands, the 9 workstream-B commands, and the 7 workstream-C commands; the 41 missing are: `CreateExamType`, `UpdateExamType`, `DeleteExamType`, `CreateExamSetup`, `UpdateExamSetup`, `DeleteExamSetup`, `CreateOnlineExam`, `PublishOnlineExam`, `StartOnlineExam`, `SubmitOnlineExamAnswer`, `EvaluateOnlineExam`, `CloseOnlineExam`, `DeleteOnlineExam`, `SetExamSignature`, `ConfigureCustomResultSettings`, `MarkTeacherEvaluation`, `ApproveTeacherEvaluation`, `RejectTeacherEvaluation`, `AddTeacherRemark`, `UpdateTeacherRemark`, `DeleteTeacherRemark`, `CreateMarksGrade`, `UpdateMarksGrade`, `DeleteMarksGrade`, `MarkExamAttendance`, `UpdateExamAttendance`, `CreateExamSetting`, `UpdateExamSetting`, `DeleteExamSetting`, `ConfigureAdmitCardSettings`, `ConfigureSeatPlanSettings`, `ConfigureTeacherEvaluation`, `PublishExamRoutine`, `PublishFrontResult`, `UpdateExamRoutinePage`, `UpdateFrontendExamResult`, `MarkExamStepSkip`, `RequestAbsenceNotification`, `CreateQuestion`, `UpdateQuestion`, `DeleteQuestion`, `CreateQuestionGroup`, `UpdateQuestionGroup`, `DeleteQuestionGroup`, `CreateQuestionLevel`, `UpdateQuestionLevel`, `DeleteQuestionLevel`, `AddOnlineExamQuestion`, `UpdateOnlineExamQuestion`, `DeleteOnlineExamQuestion`, `AddQuestionOption`, `UpdateQuestionOption`, `DeleteQuestionOption`.

**Expected:**

62 typed command structs (per the catalog table at `docs/commands/assessment.md:12-74`).

**Evidence:**

`crates/domains/assessment/src/commands.rs` declares 21 `pub struct` (lines 78, 123, 160, 184, 196, 216, 232, 246, 255, 272, 286, 299, 315, 330, 559, 579, 597, 611, 628, 643, 660). `docs/commands/assessment.md:12-74` lists 62 command rows.

---

### FINDING 30 (id: `DOMAIN-ASS-030`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:1080-1100` (production code) and `docs/specs/assessment/commands.md:276-290`

**Description:**

The `republish_result` production function calls `cast_exam_id_placeholder()` (line 1092) which returns an `ExamId` constructed with `uuid::Uuid::nil()`. The same function also mints nil-UUID `ClassId` and `SectionId` (lines 1093-1094). The `ResultRepublished` event is therefore written to storage with `exam_id = 00000000-...` and downstream subscribers cannot correlate.

**Expected:**

Resolve the per-exam metadata from a `ResultStore` aggregate lookup (or accept them as command fields) before minting the event.

**Evidence:**

`crates/domains/assessment/src/services.rs:1080-1100` `Ok(ResultRepublished::new(cmd.result_store_id.cast_exam_id_placeholder(), educore_academic::ClassId::new(cmd.result_store_id.school_id(), uuid::Uuid::nil()), educore_academic::SectionId::new(cmd.result_store_id.school_id(), uuid::Uuid::nil()), cmd.reason, ...))`. The helper at `services.rs:1372-1380` is documented as `/// **Phase 4 stub.** Returns an ExamId placeholder. The real resolution lands in Phase 16 (engine facade) which re-resolves via the storage port.`

---

### FINDING 31 (id: `DOMAIN-ASS-031`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:1129-1149` (production code) and `docs/specs/assessment/commands.md:291-306`

**Description:**

The `generate_report_card` production function mints `ReportCardGenerated` with `Uuid::nil()` for the `exam_id` field (line 1143). The spec's `ReportCardGenerated` event requires the real exam id (so the report card can be opened / printed / downloaded). Using nil UUIDs is a data-integrity bug.

**Expected:**

Resolve the `exam_id` from the `ResultStore` aggregate (or accept it as a command field) before minting the event.

**Evidence:**

`crates/domains/assessment/src/services.rs:1140-1148` `Ok(ReportCardGenerated::new(cmd.result_store_id, cmd.student_id, ExamId::new(cmd.result_store_id.school_id(), uuid::Uuid::nil()), cmd.include_remarks, ...))`.

---

### FINDING 4 (id: `DOMAIN-ASS-004`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/events.rs` (entire file) and `docs/specs/assessment/events.md:1-512`

**Description:**

The events file ships 20 event structs but is missing 45 of the 65 events declared in the events catalog (`docs/events/assessment.md:10-90`) and per-event spec (`docs/specs/assessment/events.md`). The 20 shipped are: `ExamCreated`, `ExamUpdated`, `ExamDeleted`, `ExamScheduled`, `ExamScheduleUpdated`, `ExamScheduleCancelled`, `SeatPlanGenerated`, `SeatPlanUpdated`, `SeatPlanCancelled`, `AdmitCardGenerated`, `AdmitCardRegenerated`, `AdmitCardCancelled`, `MarksRegisterCreated`, `MarksEntered`, `MarksSubmitted`, `MarksRegisterCancelled`, `ResultStoreCreated`, `ResultRemarksUpdated`, `ResultPublished`, `ResultRepublished`, `ReportCardGenerated`. The 45 missing include: `ExamTypeCreated/Updated/Deleted`, `ExamSetupCreated/Updated/Deleted`, `ExamSettingCreated/Updated/Deleted`, `ExamSignatureCreated/Updated/Deleted`, `ExamRoutinePageUpdated`, `FrontExamRoutinePublished`, `FrontResultPublished`, `FrontendExamResultUpdated`, `MarksGradeCreated/Updated/Deleted`, `MarkStoreCreated`, `TeacherRemarkUpdated`, `MarkStoreDeleted`, `ResultSettingUpdated`, `CustomResultSettingUpdated`, `OnlineExamCreated/Updated/Published/Started/Answered/Evaluated/Closed/Deleted`, `OnlineExamMarkCreated`, `OnlineExamQuestionAdded/Updated/Deleted`, `QuestionOptionAdded/Updated/Deleted`, `QuestionCreated/Updated/Deleted`, `QuestionGroupCreated/Updated/Deleted`, `QuestionLevelCreated/Updated/Deleted`, `SeatPlanSettingUpdated`, `AdmitCardSettingUpdated`, `TeacherEvaluationCompleted/Approved/Rejected/Configured`, `TeacherRemarkAdded/Updated/Deleted`, `ExamAttendanceMarked/Updated`, `ExamStepSkipSet`, `ExamAbsenceNotificationRequested`.

**Expected:**

65 typed `DomainEvent` implementations.

**Evidence:**

`crates/domains/assessment/src/events.rs` declares 21 `pub struct` (lines 45, 148, 213, 272, 352, 400, 453, 509, 555, 604, 656, 705, 889, 937, 992, 1047, 1093, 1141, 1186, 1240, 1292). `docs/events/assessment.md:10-90` lists 65 event rows.

---

### FINDING 44 (id: `DOMAIN-ASS-044`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/entities.rs` and `docs/specs/assessment/tables.md:1-72`

**Description:**

Zero of the 47 tables listed in `docs/specs/assessment/tables.md` have a corresponding `#[derive(DomainQuery)]` struct in `entities.rs` (or anywhere in the crate). The build-plan § "Runtime DDL emission" requires the storage adapter to walk the macro-emitted typed AST at schema-creation time. Without the macro emissions, the storage adapter cannot emit DDL for the 47 tables.

**Expected:**

A `#[derive(DomainQuery)]` struct per table row in `tables.md`, generating the typed AST that the storage adapter translates to dialect-specific DDL.

**Evidence:**

`docs/specs/assessment/tables.md:1-72` lists 47 table rows. `grep -n 'derive.*DomainQuery' crates/domains/assessment/src/*.rs` returns 0 matches (verified). `entities.rs:38, 81, 114` only have `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]`.

---

### FINDING 46 (id: `DOMAIN-ASS-046`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/` (no `tests/` directory) and `AGENTS.md` (9-file layout)

**Description:**

The crate has no `tests/` directory for integration tests. AGENTS.md lists the 9-file module layout (`lib.rs`, `aggregate.rs`, `entities.rs`, `value_objects.rs`, `commands.rs`, `events.rs`, `services.rs`, `repository.rs`, `query.rs`, `errors.rs` = 10 files; the 10th is the implied `tests/` for integration). All "integration" tests live in `crates/tools/storage-parity/tests/assessment_integration.rs` (a single file exercising only `create_exam` per the handoff at `docs/handoff/PHASE-4-HANDOFF.md:66-71`). The schedule-mark-publish-report workflow from `docs/specs/assessment/workflows.md:30-65, 81-99, 110-128` has no integration test.

**Expected:**

A `tests/` directory with at least one workflow integration test that exercises the full `schedule_exam` → `enter_marks` → `submit_marks` → `publish_result` → `generate_report_card` flow.

**Evidence:**

`ls /home/beznet/Workspace/smscore/crates/domains/assessment/tests` returns "No such file or directory". `AGENTS.md` (Module Layout) shows `tests/` as a required entry.

---

### FINDING 5 (id: `DOMAIN-ASS-005`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/value_objects.rs` (entire file) and `docs/specs/assessment/value-objects.md:11-117`

**Description:**

The value-objects file ships 13 typed ids (`ExamId`, `ExamTypeId`, `ExamScheduleId`, `ExamScheduleSubjectId`, `SeatPlanId`, `SeatPlanChildId`, `AdmitCardId`, `StaffId`, `ClassRoomId`, `MarksRegisterId`, `MarksRegisterChildId`, `ResultStoreId`) + 4 value types (`ExamName`, `ExamCode`, `ExamMark`, `FullMark`) + 3 numeric newtypes (`Marks`, `TotalMarks`, `Gpa`) + 1 grade row (`MarksGradeRow`) + 1 grade string (`Grade`) + 2 enums (`ExamTerm`, `ResultStatus`) = 25 value objects, but is missing 26 of the typed ids and 13 of the value types listed in the spec table at `docs/specs/assessment/value-objects.md:11-117`. Missing ids: `ExamSetupId`, `ExamSettingId`, `ExamSignatureId`, `MarkStoreId`, `MarkStoreEntryId`, `ResultSettingId`, `MarksGradeId`, `TemporaryMeritListId`, `MeritPositionId`, `ExamWisePositionId`, `AllExamWisePositionId`, `CustomResultSettingId`, `CustomTemporaryResultId`, `ExamStepSkipId`, `ExamRoutinePageId`, `FrontExamRoutineId`, `FrontResultId`, `FrontendExamResultId`, `OnlineExamId`, `QuestionBankId`, `QuestionGroupId`, `QuestionLevelId`, `QuestionAssignmentId`, `QuestionMuOptionId`, `OnlineExamQuestionId`, `OnlineExamQuestionAssignId`, `OnlineExamMarkId`, `OnlineExamStudentAnswerMarkingId`, `StudentTakeOnlineExamId`, `SeatPlanSettingId`, `AdmitCardSettingId`, `TeacherEvaluationId`, `TeacherRemarkId`, `ExamAttendanceId`, `ExamAttendanceChildId`. Missing enums: `QuestionType`, `OnlineExamStatus`, `AttemptStatus`, `AnswerStatus`, `AttendanceType`. Missing value types: `ExamTitle`, `QuestionTitle`, `QuestionOption`, `Remark`, `Comment`, `SignatureTitle`, `GroupTitle`, `Level`, `RoutinePageTitle`, `AverageMark`, `Percentage`, `ExamPercentage`, `MeritPosition`, `Rating`.

**Expected:**

All 47 typed ids + 7 closed enums + 14 named value types from `docs/specs/assessment/value-objects.md:11-117`.

**Evidence:**

`crates/domains/assessment/src/value_objects.rs` lists 25 pub items; `docs/specs/assessment/value-objects.md:11-117` lists 47 typed ids, 7 enums, 11 named strings, 11 numeric newtypes, 4 special-purpose wrappers.

---

### FINDING 6 (id: `DOMAIN-ASS-006`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs` (entire file) and `docs/specs/assessment/services.md:1-373`

**Description:**

The services file ships 21 free functions + 1 service struct (`ResultService` with 9 grading methods). The spec at `docs/specs/assessment/services.md:1-373` defines 8 service structs (`ExamService`, `MarksService`, `ResultService`, `ReportCardService`, `SeatPlanService`, `AdmitCardService`, `OnlineExamService`, `TeacherEvaluationService`, `MarksGradeService`) + 2 policy structs (`ResultEligibility`, `AdmitCardEligibility`) + 2 specification structs (`ActiveExamSchedule`, `PendingOnlineExam`) + 1 cross-domain coordinator (`AssessmentCoordinator`). The code is missing 7 of 8 service structs, both policies, both specifications, and the coordinator.

**Expected:**

`pub struct ExamService` (with `plan_for_class`, `validate_no_teacher_overlap`, `validate_no_room_overlap`, `lock_after_publish` per `docs/specs/assessment/services.md:8-30`), `pub struct MarksService` (with `initialize_registers`, `validate_marks`, `is_absent_row`, `submit` per `docs/specs/assessment/services.md:38-58`), `pub struct ReportCardService` (with `build_payload`, `render_html`, `render_pdf` per `docs/specs/assessment/services.md:130-146`), `pub struct SeatPlanService` (with `assign_rooms`, `validate_total`, `validate_no_room_overlap`, `build_seat_plan` per `docs/specs/assessment/services.md:155-178`), `pub struct AdmitCardService` (with `build_card`, `render_html`, `render_pdf` per `docs/specs/assessment/services.md:187-201`), `pub struct OnlineExamService` (with `start`, `accept_answer`, `auto_evaluate`, `manual_mark`, `close` per `docs/specs/assessment/services.md:208-238`), `pub struct TeacherEvaluationService` (with `is_window_open`, `can_submit`, `build_evaluation`, `aggregate` per `docs/specs/assessment/services.md:250-275`), `pub struct MarksGradeService` (with `validate_no_overlap`, `validate_contiguous`, `find_grade` per `docs/specs/assessment/services.md:283-298`).

**Evidence:**

`crates/domains/assessment/src/services.rs` declares 1 `pub struct` (line 1176: `ResultService`); the spec at `docs/specs/assessment/services.md` declares 8 service structs. `ResultService::publish` (line 1055) is a free function, not the spec's `ResultService::publish` which is an `impl` method that materializes ResultStore/MeritPosition/ExamWisePosition/AllExamWisePosition/CustomTemporaryResult rows.

---

### FINDING 7 (id: `DOMAIN-ASS-007`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/repository.rs:42-105` and `docs/specs/assessment/repositories.md:1-524`

**Description:**

The repository file ships 6 port traits (`ExamRepository`, `ExamScheduleRepository`, `SeatPlanRepository`, `AdmitCardRepository`, `MarksRegisterRepository`, `ResultRepository`). The spec at `docs/specs/assessment/repositories.md:1-524` declares 13 port traits. Missing: `ExamTypeRepository` (lines 9-19), `MarkStoreRepository` (lines 120-131), `MarksGradeRepository` (lines 184-191), `OnlineExamRepository` (lines 197-240), `QuestionBankRepository` (lines 246-259), `TeacherEvaluationRepository` (lines 329-356), `TeacherRemarkRepository` (lines 361-377), `ExamAttendanceRepository` (lines 383-400), `ResultSettingRepository` (lines 406-418), `ExamSettingRepository` (lines 424-444).

**Expected:**

13 `#[async_trait] pub trait XxxRepository: Send + Sync` per the spec.

**Evidence:**

`crates/domains/assessment/src/repository.rs` declares 6 `pub trait` (lines 44, 115, 148, 168, 193, 217). `docs/specs/assessment/repositories.md:1-524` defines 13 repository port traits.

---

### FINDING 73 (id: `DOMAIN-ASS-073`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/aggregate.rs:432-491` and `docs/specs/assessment/aggregates.md:751-803`

**Description:**

The `AdmitCard` aggregate is missing the `ExamType` `ExamTitle` `ExamSetting` etc. per-school / per-academic-year state the spec describes. The aggregate also has no `is_published` flag (the spec at lines 763-770 specifies 3 invariants, but the code has only the basic 10-field audit footer).

**Expected:**

The `AdmitCard` aggregate should carry the `AdmitCardSetting` snapshot (the school-side branding that determines which fields appear on the rendered card) and the immutable-vs-regeneration lifecycle fields.

**Evidence:**

`crates/domains/assessment/src/aggregate.rs:432-456` defines the `AdmitCard` struct with 7 fields (id, school_id, student_record_id, exam_type_id, academic_year_id, generated_at, plus 10 audit fields). `docs/specs/assessment/aggregates.md:751-803` defines 3 invariants and lifecycle.

---

### FINDING 8 (id: `DOMAIN-ASS-008`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/repository.rs:115-143` and `docs/specs/assessment/repositories.md:47-90`

**Description:**

The shipped `ExamScheduleRepository` is missing 5 methods declared in the spec. The spec defines `list_for_teacher`, `list_for_room`, `list_in_range`, `insert_subject`, `list_subjects`; the code has only `get`, `find`, `list_for_section`, `insert`, `update`, `delete`.

**Expected:**

`async fn list_for_teacher(&self, school, teacher, year)`, `async fn list_for_room(&self, school, room, year)`, `async fn list_in_range(&self, school, from, to)`, `async fn insert_subject(&self, s)`, `async fn list_subjects(&self, schedule_id)`.

**Evidence:**

`crates/domains/assessment/src/repository.rs:115-143` has 6 methods. `docs/specs/assessment/repositories.md:49-90` has 11 methods.

---

### FINDING 9 (id: `DOMAIN-ASS-009`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/repository.rs:193-212` and `docs/specs/assessment/repositories.md:94-116`

**Description:**

The shipped `MarksRegisterRepository` is missing 5 methods declared in the spec. The spec defines `list_for_student`, `upsert_child`, `list_children`, `child`; the code has only `get`, `find`, `list_for_exam`, `insert`, `update`.

**Expected:**

`async fn list_for_student`, `async fn upsert_child`, `async fn list_children`, `async fn child` methods on the trait.

**Evidence:**

`crates/domains/assessment/src/repository.rs:193-212` has 5 methods. `docs/specs/assessment/repositories.md:96-115` has 9 methods.

---

### FINDING 96 (id: `DOMAIN-ASS-096`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:1011-1027` and `docs/build-plan.md:629-640`

**Description:**

The build-plan § "Phase 4 Tasks" task 4 says: "Integration test: schedule an exam, enter marks, compute result, publish report card." The shipped `assessment_integration.rs` only exercises `create_exam` (per the handoff at `docs/handoff/PHASE-4-HANDOFF.md:66-71`). The 4-step workflow (schedule → enter → compute → publish) is not integration-tested.

**Expected:**

A workflow integration test that exercises `schedule_exam`, `enter_marks`, `submit_marks`, `publish_result`, `generate_report_card` end-to-end.

**Evidence:**

`crates/tools/storage-parity/tests/assessment_integration.rs:1-499` — only `create_exam` is exercised. The handoff at `docs/handoff/PHASE-4-HANDOFF.md:66-71` admits the limited scope. `docs/build-plan.md:636-637` "Integration test: schedule an exam, enter marks, compute result, publish report card. Verify outbox + audit + RLS."

---

### FINDING 11 (id: `DOMAIN-ASS-011`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/commands.rs:78-114` and `docs/specs/assessment/commands.md:62-86`

**Description:**

The shipped `CreateExamCommand` uses raw `String` for `name` and `code` and raw `f32` for `exam_mark` and `pass_mark` instead of the typed value objects `ExamName`, `ExamCode`, `ExamMark`, `PassMark` defined in the same crate's `value_objects.rs:195-359`. The spec mandates typed wrappers at construction.

**Expected:**

`pub name: ExamName, pub code: ExamCode, pub exam_mark: ExamMark, pub pass_mark: PassMark` per `docs/specs/assessment/commands.md:65-76` and the engine rule "Compile-time safety over strings" in `AGENTS.md`.

**Evidence:**

`crates/domains/assessment/src/commands.rs:95-101` `pub name: String, pub code: String, pub exam_mark: f32, pub pass_mark: f32,`. The typed wrappers exist in `crates/domains/assessment/src/value_objects.rs:195, 247, 303` and `crates/domains/assessment/src/value_objects.rs:554` (PassMark re-export). The spec at `docs/specs/assessment/commands.md:65-76` lists `pub exam_mark: ExamMark, pub pass_mark: PassMark`.

---

### FINDING 12 (id: `DOMAIN-ASS-012`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/commands.rs:78-104` and `docs/specs/assessment/commands.md:62-86`

**Description:**

The shipped `CreateExamCommand` is missing the `parent_id: Option<ExamId>` field that the spec mandates for composite exam terms. The spec comment notes "for composite exam terms" (a final term is composed of mid-terms per `docs/specs/assessment/aggregates.md:28-29`).

**Expected:**

`pub parent_id: Option<ExamId>` per `docs/specs/assessment/commands.md:74`.

**Evidence:**

`docs/specs/assessment/commands.md:74` `pub parent_id: Option<ExamId>, // for composite exam terms`. `crates/domains/assessment/src/commands.rs:78-104` has no `parent_id` field.

---

### FINDING 13 (id: `DOMAIN-ASS-013`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/commands.rs:196-212` and `docs/specs/assessment/commands.md:120-145`

**Description:**

The shipped `ScheduleExamCommand` is missing three fields the spec mandates: `room_id: Option<ClassRoomId>`, `teacher_id: Option<StaffId>`, and `exam_period_id: Option<ClassTimeId>`. The spec also uses typed wrappers `ExamDate`, `StartTime`, `EndTime` for the time fields, but the code uses raw `chrono::NaiveDate` and `chrono::NaiveTime`.

**Expected:**

`pub room_id: Option<ClassRoomId>, pub teacher_id: Option<StaffId>, pub exam_period_id: Option<ClassTimeId>` per `docs/specs/assessment/commands.md:130-132`; and typed wrappers for the date/time fields.

**Evidence:**

`docs/specs/assessment/commands.md:122-134` lists the full struct including the three missing fields. `crates/domains/assessment/src/commands.rs:196-206` only has `schedule_id`, `exam_id`, `class_id`, `section_id`, `date`, `start_time`, `end_time`, `subjects` — no `room_id`, `teacher_id`, `exam_period_id`.

---

### FINDING 14 (id: `DOMAIN-ASS-014`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/commands.rs:184-192` and `docs/specs/assessment/commands.md:136-144`

**Description:**

The shipped `ScheduleSubjectEntry` uses raw `f32` for `full_mark` and `pass_mark` instead of typed `FullMark` and `PassMark`. The spec mandates the typed wrappers. The `room` field is also typed `Option<String>` in the spec but `Option<ClassRoomId>` in the code (semantic drift).

**Expected:**

`pub full_mark: FullMark, pub pass_mark: PassMark, pub room: Option<String>` per `docs/specs/assessment/commands.md:136-144`.

**Evidence:**

`docs/specs/assessment/commands.md:136-144` lists `pub full_mark: FullMark, pub pass_mark: PassMark`. `crates/domains/assessment/src/commands.rs:184-192` uses `pub full_mark: f32, pub pass_mark: f32, pub room_id: Option<ClassRoomId>` — wrong types for the marks and wrong field name/type for the room.

---

### FINDING 15 (id: `DOMAIN-ASS-015`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/commands.rs:579-593` and `docs/specs/assessment/commands.md:210-221`

**Description:**

The shipped `EnterMarksCommand` uses raw `Option<f32>` for `marks` instead of `Option<Marks>`. The spec mandates the typed wrapper. The spec also uses plural `comments: Option<String>` (per the per-event spec at `docs/specs/assessment/events.md:140`); the code uses singular `comment: Option<String>`.

**Expected:**

`pub marks: Option<Marks>` and `pub comments: Option<String>` per `docs/specs/assessment/commands.md:218-220`.

**Evidence:**

`docs/specs/assessment/commands.md:213-221` lists `pub marks: Option<Marks>, pub is_absent: bool, pub comments: Option<String>,`. `crates/domains/assessment/src/commands.rs:579-593` uses `pub marks: Option<f32>, ... pub comment: Option<String>,`.

---

### FINDING 16 (id: `DOMAIN-ASS-016`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/commands.rs:255-268` and `docs/specs/assessment/commands.md:409-425`

**Description:**

The shipped `GenerateSeatPlanCommand` is missing the `exam_type_id: ExamTypeId` field that the spec mandates. The spec uses `exam_id: ExamId` whereas the code uses `exam_id: ExamId` (matches) but adds a `seat_plan_id: SeatPlanId` (caller-supplied id pattern) that is not in the spec.

**Expected:**

`pub exam_type_id: ExamTypeId` per `docs/specs/assessment/commands.md:416` (the spec's `GenerateSeatPlanCommand`).

**Evidence:**

`docs/specs/assessment/commands.md:411-417` lists `pub exam_id: ExamId, pub class_id: ClassId, pub section_id: SectionId, pub exam_type_id: ExamTypeId,`. `crates/domains/assessment/src/commands.rs:255-268` omits `exam_type_id` and adds `seat_plan_id` not in spec.

---

### FINDING 17 (id: `DOMAIN-ASS-017`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/events.rs:45-76` and `docs/specs/assessment/events.md:59-69`

**Description:**

The shipped `ExamCreated` event is missing the `parent_id: Option<ExamId>` field that the spec mandates for composite exam terms.

**Expected:**

`pub parent_id: Option<ExamId>` per `docs/specs/assessment/events.md:69`.

**Evidence:**

`docs/specs/assessment/events.md:60-69` `pub struct ExamCreated { pub exam_id, pub exam_type_id, ..., pub parent_id: Option<ExamId> }`. `crates/domains/assessment/src/events.rs:45-76` has no `parent_id` field.

---

### FINDING 18 (id: `DOMAIN-ASS-018`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/events.rs:889-933` and `docs/specs/assessment/events.md:120-127`

**Description:**

The shipped `MarksRegisterCreated` event is missing the `subjects: Vec<SubjectId>` field that the spec mandates (used to know which subject rows were initialised for the student).

**Expected:**

`pub subjects: Vec<SubjectId>` per `docs/specs/assessment/events.md:125`.

**Evidence:**

`docs/specs/assessment/events.md:121-126` `pub struct MarksRegisterCreated { pub marks_register_id, pub exam_id, pub student_id, pub subjects: Vec<SubjectId> }`. `crates/domains/assessment/src/events.rs:889-896` has no `subjects` field.

---

### FINDING 19 (id: `DOMAIN-ASS-019`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/events.rs:937-987` and `docs/specs/assessment/events.md:133-140`

**Description:**

The shipped `MarksEntered` event is missing the `comments: Option<String>` field that the spec mandates. The code stores the comment in the `EnterMarksCommand` but never carries it into the emitted event.

**Expected:**

`pub comments: Option<String>` per `docs/specs/assessment/events.md:139`.

**Evidence:**

`docs/specs/assessment/events.md:133-140` lists `pub comments: Option<String>,`. `crates/domains/assessment/src/events.rs:937-946` has no `comments` field; `services.rs:981-998` `enter_marks` does not pass a comment argument.

---

### FINDING 20 (id: `DOMAIN-ASS-020`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/events.rs:992-1042` and `docs/specs/assessment/events.md:142-150`

**Description:**

The shipped `MarksSubmitted` event is missing two fields the spec mandates: `total_students: u32` and `submitted_at: Timestamp`. Downstream subscribers (assessment-self, communication) cannot know how many students the submission covers or when it happened.

**Expected:**

`pub total_students: u32` and `pub submitted_at: Timestamp` per `docs/specs/assessment/events.md:148-149`.

**Evidence:**

`docs/specs/assessment/events.md:142-150` lists `pub total_students: u32, pub submitted_at: Timestamp,`. `crates/domains/assessment/src/events.rs:992-1001` has no `total_students` and no `submitted_at` fields. `services.rs:1004-1027` `submit_marks` hardcodes `subject_count: 0` (line 1022) and uses `now` only for the implicit `EventId::from_uuid(uuid::Uuid::now_v7())`.

---

### FINDING 21 (id: `DOMAIN-ASS-021`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/events.rs:1093-1137` and `docs/specs/assessment/events.md:158-168`

**Description:**

The shipped `ResultStoreCreated` event is missing 5 fields the spec mandates: `exam_type_id`, `subject_id`, `total_marks`, `gpa`, `grade`. Downstream subscribers cannot tell which subject a result row is for or what mark it carries.

**Expected:**

`pub exam_type_id: ExamTypeId, pub subject_id: SubjectId, pub total_marks: TotalMarks, pub gpa: Gpa, pub grade: Grade` per `docs/specs/assessment/events.md:162-167`.

**Evidence:**

`docs/specs/assessment/events.md:159-168` lists all six required fields. `crates/domains/assessment/src/events.rs:1093-1100` only has `result_store_id, exam_id, student_id`.

---

### FINDING 22 (id: `DOMAIN-ASS-022`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/events.rs:1186-1236` and `docs/specs/assessment/events.md:175-182`

**Description:**

The shipped `ResultPublished` event uses `student_count: u32` instead of the spec's `student_ids: Vec<StudentId>`. The spec requires a vector of per-student ids (so subscribers can act on each student without a re-query); the code carries a meaningless counter.

**Expected:**

`pub student_ids: Vec<StudentId>` per `docs/specs/assessment/events.md:180`.

**Evidence:**

`docs/specs/assessment/events.md:175-182` lists `pub student_ids: Vec<StudentId>,`. `crates/domains/assessment/src/events.rs:1186-1195` has `pub student_count: u32`; `services.rs:1055-1076` `publish_result` hardcodes `student_count: 0` (line 1071).

---

### FINDING 23 (id: `DOMAIN-ASS-023`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/events.rs:604-635` and `docs/specs/assessment/events.md:421-428`

**Description:**

The shipped `AdmitCardGenerated` event is missing the `generated_at: Timestamp` field the spec mandates.

**Expected:**

`pub generated_at: Timestamp` per `docs/specs/assessment/events.md:427`.

**Evidence:**

`docs/specs/assessment/events.md:422-428` lists `pub generated_at: Timestamp,`. `crates/domains/assessment/src/events.rs:604-612` has no `generated_at` field.

---

### FINDING 24 (id: `DOMAIN-ASS-024`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/events.rs:1292-1338` and `docs/specs/assessment/events.md:203-210`

**Description:**

The shipped `ReportCardGenerated` event is missing the `payload: ReportCardPayload` field the spec mandates. The payload is the structured per-student report (per-subject marks, GPA, grade, merit position, attendance summary, teacher remarks) — the entire point of the report card.

**Expected:**

`pub payload: ReportCardPayload` per `docs/specs/assessment/events.md:209`. The `ReportCardPayload` type is not defined anywhere in the crate.

**Evidence:**

`docs/specs/assessment/events.md:204-210` `pub struct ReportCardGenerated { pub result_store_id, pub student_id, pub exam_id, pub include_remarks, pub payload: ReportCardPayload }`. `crates/domains/assessment/src/events.rs:1292-1300` omits the `payload` field. `services.rs:1129-1149` `generate_report_card` does not construct any payload.

---

### FINDING 25 (id: `DOMAIN-ASS-025`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/events.rs:272-348` and `docs/specs/assessment/events.md:90-101`

**Description:**

The shipped `ExamScheduled` event is missing two fields the spec mandates: `room_id: Option<ClassRoomId>` and `teacher_id: Option<StaffId>`. Subscribers (communication, cms) cannot know which room or teacher to broadcast.

**Expected:**

`pub room_id: Option<ClassRoomId>, pub teacher_id: Option<StaffId>` per `docs/specs/assessment/events.md:98-99`.

**Evidence:**

`docs/specs/assessment/events.md:90-101` lists the full struct. `crates/domains/assessment/src/events.rs:272-294` has no `room_id` or `teacher_id` fields; `services.rs:317-357` `schedule_exam` hardcodes `None` (lines 337, 338) for the room and teacher.

---

### FINDING 26 (id: `DOMAIN-ASS-026`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/events.rs:453-506` and `docs/specs/assessment/events.md:404-411`

**Description:**

The shipped `SeatPlanGenerated` event uses `exam_id: ExamId` instead of the spec's `exam_type_id: ExamTypeId`, and is missing the `rooms: u32` field. The spec's per-student seat allocation is keyed by exam type; the code's event is keyed by exam instance and lacks the room count.

**Expected:**

`pub exam_type_id: ExamTypeId, pub rooms: u32, pub total_students: u32` per `docs/specs/assessment/events.md:406-410`.

**Evidence:**

`docs/specs/assessment/events.md:404-411` lists `pub exam_type_id, pub class_id, pub section_id, pub rooms, pub total_students`. `crates/domains/assessment/src/events.rs:453-462` has `pub exam_id` (wrong) and no `rooms` field.

---

### FINDING 27 (id: `DOMAIN-ASS-027`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/aggregate.rs:568-636` and `docs/specs/assessment/aggregates.md:268-303`

**Description:**

The shipped `ResultStore` aggregate uses raw `f32` for `total_marks` and `total_gpa` instead of the typed wrappers `TotalMarks` and `Gpa` defined in `value_objects.rs:386-421`. The spec at `docs/specs/assessment/value-objects.md:95-97` mandates the typed wrappers.

**Expected:**

`pub total_marks: TotalMarks, pub total_gpa: Gpa` per `docs/specs/assessment/value-objects.md:95-97`.

**Evidence:**

`crates/domains/assessment/src/aggregate.rs:578-579` `pub total_marks: f32, pub total_gpa: f32,`. `value_objects.rs:386-421` defines `TotalMarks(f32)` and `Gpa(f32)` typed newtypes with validation.

---

### FINDING 32 (id: `DOMAIN-ASS-032`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:275-304` and `docs/specs/assessment/aggregates.md:65-68`

**Description:**

The `delete_exam` service does NOT enforce the "An Exam cannot be deleted while MarksRegister rows reference it" invariant the spec mandates. The doc-comment at lines 269-271 explicitly defers the check to the test fixture: "the test fixture ensures this by deleting before any marks are entered." In production, calling `delete_exam` on an exam that has `MarksRegister` children will succeed and orphan the children's foreign-key reference.

**Expected:**

A pre-conditions check that consults a `MarksRegisterRepository` (or its substitute) and returns `DomainError::Conflict` if any `MarksRegister` references the exam.

**Evidence:**

`crates/domains/assessment/src/services.rs:286-304` `pub fn delete_exam<...>(...) -> Result<ExamDeleted> { let now = clock.now(); let actor = cmd.tenant.actor_id; if exam.active_status.is_retired() { return Err(DomainError::conflict(format!("exam {} is already deleted", exam.id))); } exam.active_status = ActiveStatus::Retired; ... }`. The doc-comment at lines 269-270 `// marks are entered.`.

---

### FINDING 33 (id: `DOMAIN-ASS-033`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:317-357` and `docs/specs/assessment/aggregates.md:130-138`

**Description:**

The `schedule_exam` service does NOT enforce any of the schedule invariants the spec mandates: uniqueness by `(exam_id, class_id, section_id)`, `StartTime < EndTime`, no teacher overlap, no room overlap, or that the date is within the academic year. None of these checks are performed; the function just mints the aggregate and event.

**Expected:**

Five pre-conditions checks: uniqueness, time-well-formedness, teacher-conflict, room-conflict, academic-year-range.

**Evidence:**

`crates/domains/assessment/src/services.rs:317-357` `pub fn schedule_exam<...>(_cmd, clock, ids) -> Result<...> { let now = clock.now(); let event_id = ids.next_event_id(); let schedule_id = _cmd.schedule_id; let aggregate = ExamSchedule::fresh(schedule_id, _cmd.exam_id, _cmd.class_id, _cmd.section_id, _cmd.date, _cmd.start_time, _cmd.end_time, None, None, ...); ... }` — no validation. Spec at `docs/specs/assessment/aggregates.md:130-138` lists 5 invariants.

---

### FINDING 34 (id: `DOMAIN-ASS-034`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:438-477` and `docs/specs/assessment/aggregates.md:700-705`

**Description:**

The `generate_seat_plan` service does NOT enforce any of the seat plan invariants the spec mandates: uniqueness by `(exam_type_id, class_id, section_id, academic_id)`, sum of `assign_students` equals section's student count, and no time overlap of `SeatPlanChild` allocations. The function just sums and mints.

**Expected:**

Three pre-conditions checks.

**Evidence:**

`crates/domains/assessment/src/services.rs:438-477` `pub fn generate_seat_plan<...>(cmd, clock, ids) -> Result<...> { ... let total: u32 = cmd.allocations.iter().map(|a| u64::from(a.assign_students)).sum::<u64>().try_into().unwrap_or(u32::MAX); let aggregate = SeatPlan::fresh(...); ... }` — no validation. Spec at `docs/specs/assessment/aggregates.md:700-705` lists 3 invariants.

---

### FINDING 35 (id: `DOMAIN-ASS-035`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:551-581` and `docs/specs/assessment/commands.md:437-456`

**Description:**

The `generate_admit_card` service does NOT enforce any of the admit card pre-conditions the spec mandates: an active `StudentRecord` for the academic year, an `AdmitCardSetting` for the academic year, a `SeatPlan` for the section, and an `ExamSchedule` for at least one subject. The function just mints the aggregate and event.

**Expected:**

Four pre-conditions checks before the `AdmitCard::fresh` call.

**Evidence:**

`crates/domains/assessment/src/services.rs:551-581` `pub fn generate_admit_card<...>(cmd, clock, ids) -> Result<...> { let now = clock.now(); let event_id = ids.next_event_id(); let aggregate = AdmitCard::fresh(cmd.admit_card_id, cmd.student_record_id, cmd.exam_type_id, cmd.academic_year_id, ...); ... }` — no validation. Spec at `docs/specs/assessment/commands.md:447-455` lists 4 pre-conditions.

---

### FINDING 36 (id: `DOMAIN-ASS-036`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:1031-1049` and `docs/specs/assessment/commands.md:190-208`

**Description:**

The `cancel_marks_register` function takes a `SubmitMarksCommand` parameter (not a `CancelMarksRegisterCommand`). The signature is incorrect; the spec defines `CancelMarksRegister` (or its equivalent) as a distinct command and the function is wired to the wrong input type. Additionally, the function hardcodes the reason as the literal string `"cancelled"` (line 1044) instead of accepting a reason from the command.

**Expected:**

A new `CancelMarksRegisterCommand` type (the spec at `docs/specs/assessment/commands.md:190-208` documents the inverse — `InitializeMarksRegister` — but the catalog at `docs/commands/assessment.md:23` lists `Marks.Cancel` capability and `MarksRegisterCancelled` is emitted), and the function should accept the command's reason field.

**Evidence:**

`crates/domains/assessment/src/services.rs:1031-1035` `pub fn cancel_marks_register<C, G>(cmd: SubmitMarksCommand, clock: &C, _ids: &G) -> Result<MarksRegisterCancelled>` and `crates/domains/assessment/src/services.rs:1044` `"cancelled".to_owned()`. No `CancelMarksRegisterCommand` struct exists in `commands.rs`.

---

### FINDING 37 (id: `DOMAIN-ASS-037`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:176-260` and `docs/specs/assessment/commands.md:88-103`

**Description:**

The `update_exam` service uses `_ctx: &TenantContext` and `tenant: _` (the `ctx` parameter is unused). The function relies on the dispatcher for the capability check and does not even sanity-check that `cmd.tenant.school_id == exam.id.school_id()`. A cross-tenant command (tenant A's actor operating on tenant B's exam id) will be silently accepted.

**Expected:**

An `Err(DomainError::Forbidden)` if `cmd.tenant.school_id != exam.school_id`; or, at minimum, a tenant-scoped uniqueness check on the new code.

**Evidence:**

`crates/domains/assessment/src/services.rs:176-186` `pub fn update_exam<C, G>(_ctx: &TenantContext, exam: &mut Exam, cmd: UpdateExamCommand, clock: &C, _ids: &G) -> Result<ExamUpdated> { ... }`. No `school_id` comparison is made anywhere in the function body (lines 186-259).

---

### FINDING 38 (id: `DOMAIN-ASS-038`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:77-161` and `crates/domains/assessment/src/commands.rs:356-369`

**Description:**

The `AssessmentUniquenessChecker` port method `exam_unique_key_exists` returns `bool` rather than `Result<bool>`. This means the port cannot fail with a `DomainError` (e.g. when the storage is unreachable). The spec describes the port as returning `Result<...>` for trait consistency. Additionally, `create_exam` does not assert that `cmd.exam_id.school_id() == cmd.tenant.school_id` (a cross-tenant exam id would be accepted).

**Expected:**

`fn exam_unique_key_exists(...) -> Result<bool>` and a `school_id` assertion at the start of `create_exam`.

**Evidence:**

`crates/domains/assessment/src/commands.rs:356-369` `pub trait AssessmentUniquenessChecker: Send + Sync { ... fn exam_unique_key_exists(&self, ...) -> bool; }` (returns `bool`, not `Result<bool>`). `crates/domains/assessment/src/services.rs:106-113` calls the port with no `?`.

---

### FINDING 39 (id: `DOMAIN-ASS-039`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:140-159` and `docs/specs/assessment/commands.md:62-86`

**Description:**

The `create_exam` service calls `validate_exam_name(&cmd.name)?`, `validate_exam_code(&cmd.code)?`, `validate_exam_mark(cmd.exam_mark)?`, `validate_pass_mark(cmd.pass_mark)?` twice — once for the aggregate construction (lines 91-94) and once again for the event construction (lines 151-154). The duplicated validation is wasteful and the two `?` paths can diverge if the constructors become fallible. The event-side calls should pass the already-validated newtypes (`name`, `code`, `exam_mark`, `pass_mark`) by reference.

**Expected:**

Pass the already-validated `name`, `code`, `exam_mark`, `pass_mark` newtypes to `ExamCreated::new` directly.

**Evidence:**

`crates/domains/assessment/src/services.rs:91-94` `let name = validate_exam_name(&cmd.name)?; let code = validate_exam_code(&cmd.code)?; let exam_mark = validate_exam_mark(cmd.exam_mark)?; let pass_mark = validate_pass_mark(cmd.pass_mark)?;` and `crates/domains/assessment/src/services.rs:151-154` `validate_exam_name(&cmd.name)?, validate_exam_code(&cmd.code)?, validate_exam_mark(cmd.exam_mark)?, validate_pass_mark(cmd.pass_mark)?,`.

---

### FINDING 42 (id: `DOMAIN-ASS-042`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:1176-1357` (`ResultService`) and `docs/specs/assessment/services.md:64-119`

**Description:**

The shipped `ResultService` is missing the central `publish` method the spec mandates. The spec defines `pub fn publish(&self, exam, section, registers, scale) -> Result<PublishOutcome, ValidationError>` as the heart of result publication (lines 112-118 of the spec). The code's `publish_result` free function does not invoke `ResultService::publish`; it just mints a `ResultPublished` event with `student_count: 0` and no per-student `ResultStore` rows.

**Expected:**

`impl ResultService { pub fn publish(...) -> Result<PublishOutcome, ValidationError> { ... } }` that materialises `ResultStore` rows + `MeritPosition` + `ExamWisePosition` + `AllExamWisePosition` + `CustomTemporaryResult` rows.

**Evidence:**

`docs/specs/assessment/services.md:110-119` lists `pub fn publish(...) -> Result<PublishOutcome, ValidationError>` as a `ResultService` method. `crates/domains/assessment/src/services.rs:1176-1357` defines the `ResultService` struct + 9 grading methods but no `publish` method. `services.rs:1055-1076` is a free function `publish_result` that does not call `ResultService::publish`.

---

### FINDING 43 (id: `DOMAIN-ASS-043`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/entities.rs` (entire file) and `docs/specs/assessment/aggregates.md:1-1025`

**Description:**

The entities file ships 3 child entity structs (`ExamScheduleSubject`, `SeatPlanChild`, `MarksRegisterChild`). The spec at `docs/specs/assessment/aggregates.md` defines 10 child entities (children only, not roots). Missing children: `MarkStoreEntry`, `CustomTemporaryResult`, `ExamRoutinePage`, `FrontendExamRoutine`, `FrontendResult`, `FrontendExamResult`, `QuestionAssignment`, `OnlineExamQuestion`, `QuestionMuOption`, `OnlineExamStudentAnswerMarking`, `StudentTakeOnlineExamQuestion`, `ExamAttendanceChild`. (Note: `ExamScheduleSubject`, `SeatPlanChild`, `MarksRegisterChild` are the 3 shipped; the remaining 10+ children are absent.)

**Expected:**

All 13 child entity structs per the aggregates spec.

**Evidence:**

`crates/domains/assessment/src/entities.rs:38, 81, 114` declares 3 `pub struct`. `docs/specs/assessment/aggregates.md:152, 203, 256, 583, 611, 626, 640, 656, 1006` lists 9+ child entity sections; many of these also have additional sub-children.

---

### FINDING 45 (id: `DOMAIN-ASS-045`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/query.rs:122-127, 161-168, 195-200, 226-231, 330-337, 364-371` and `docs/specs/assessment/aggregates.md:1-1025`

**Description:**

All 6 typed query stubs (`ExamQuery::execute`, `ExamScheduleQuery::execute`, `SeatPlanQuery::execute`, `AdmitCardQuery::execute`, `MarksRegisterQuery::execute`, `ResultStoreQuery::execute`) return `Err(DomainError::not_supported(...))` and never execute. None of the queries that the spec requires (the `list_for_year`, `list_for_class`, `list_for_type`, `find` by unique key, `list_for_section`, `list_for_exam`, `list_for_student` repository methods the spec defines) are queryable. The query stubs are also marked `#[allow(dead_code)]` — they exist only to be called later, never to satisfy the spec's query surface.

**Expected:**

Functional query executors backed by the typed `#[derive(DomainQuery)]` AST (Finding DOMAIN-ASS-044).

**Evidence:**

`crates/domains/assessment/src/query.rs:122-127` `pub async fn execute(&self, _ctx: &TenantContext) -> Result<Vec<crate::aggregate::Exam>> { Err(DomainError::not_supported("ExamQuery::execute is a Phase 4 stub; ... ")) }`. Six similar stubs at lines 161-168, 195-200, 226-231, 330-337, 364-371.

---

### FINDING 48 (id: `DOMAIN-ASS-048`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/lib.rs:21-37` and `crates/domains/assessment/src/aggregate.rs` and `crates/domains/assessment/src/services.rs`

**Description:**

The lib.rs module-level doc claims "5 assessment aggregate roots shipped in Phase 4 Workstream A" (line 21) and "2 typed DomainEvent implementations" (line 28). The actual shipped counts are 6 aggregate roots (Exam, ExamSchedule, SeatPlan, AdmitCard, MarksRegister, ResultStore) across workstreams A + B + C, and 21 typed `DomainEvent` implementations across the same workstreams. The doc-vs-code drift misleads readers about what is in the crate.

**Expected:**

Doc strings that match the actual shipped counts.

**Evidence:**

`crates/domains/assessment/src/lib.rs:21` `/// The 5 assessment aggregate roots shipped in Phase 4 Workstream A`. `crates/domains/assessment/src/lib.rs:28` `/// The 2 typed \`DomainEvent\` implementations shipped in Phase 4 Workstream A`. `crates/domains/assessment/src/aggregate.rs` has 6 `pub struct`; `crates/domains/assessment/src/events.rs` has 21 `pub struct` implementing `DomainEvent`.

---

### FINDING 49 (id: `DOMAIN-ASS-049`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:1182-1203` and `docs/specs/assessment/services.md:1-373`

**Description:**

The `ResultService::compute_grade` method uses a hardcoded 8-tier A-F scale (`A+ >= 90`, `A >= 80`, `B+ >= 70`, `B >= 60`, `C >= 50`, `D >= 40`, `E >= 33`, `F < 33`) and the function signature takes a `percent: f32` not a `scale: &[MarksGrade]` (as the spec mandates). The function therefore does not consume the `MarksGradeScale` port the spec requires and is not policy-driven per school.

**Expected:**

`pub fn compute_grade(percent: Percentage, scale: &[MarksGrade]) -> (Grade, Gpa)` per `docs/specs/assessment/services.md:70-73`.

**Evidence:**

`crates/domains/assessment/src/services.rs:1182-1183` `pub fn compute_grade(percent: f32) -> (crate::value_objects::Grade, crate::value_objects::Gpa) { ... }`. `docs/specs/assessment/services.md:70-73` `pub fn compute_grade(percent: Percentage, scale: &[MarksGrade]) -> (Grade, Gpa)`.

---

### FINDING 50 (id: `DOMAIN-ASS-050`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:1207-1217` and `docs/specs/assessment/services.md:75-78`

**Description:**

The `ResultService::compute_subject_marks` method takes `(marks: f32, full_mark: f32)` instead of `(child: &MarksRegisterChild, exam: &Exam)` (as the spec mandates). The function therefore cannot read the `is_absent` flag and cannot apply the school's absent rule.

**Expected:**

`pub fn compute_subject_marks(child: &MarksRegisterChild, exam: &Exam) -> (Marks, Gpa, Grade)` per `docs/specs/assessment/services.md:75-78`.

**Evidence:**

`crates/domains/assessment/src/services.rs:1207-1210` `pub fn compute_subject_marks(marks: f32, full_mark: f32) -> (crate::value_objects::Grade, crate::value_objects::Gpa) { ... }`. `docs/specs/assessment/services.md:75-78` `pub fn compute_subject_marks(child: &MarksRegisterChild, exam: &Exam) -> (Marks, Gpa, Grade)`.

---

### FINDING 51 (id: `DOMAIN-ASS-051`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:1221-1234` and `docs/specs/assessment/services.md:80-84`

**Description:**

The `ResultService::compute_total` method takes `(children: &[f32], full_marks: &[f32])` instead of `(children: &[MarksRegisterChild], exam: &Exam, scale: &[MarksGrade])`. The function therefore cannot read per-subject pass marks or apply the per-school grade scale, and produces a `(f32, Grade, Gpa)` tuple instead of the spec's `(TotalMarks, Gpa, Grade, ResultStatus)`.

**Expected:**

`pub fn compute_total(children: &[MarksRegisterChild], exam: &Exam, scale: &[MarksGrade]) -> (TotalMarks, Gpa, Grade, ResultStatus)` per `docs/specs/assessment/services.md:80-84`.

**Evidence:**

`crates/domains/assessment/src/services.rs:1221-1224` `pub fn compute_total(children: &[f32], full_marks: &[f32]) -> (f32, crate::value_objects::Grade, crate::value_objects::Gpa) { ... }`. `docs/specs/assessment/services.md:80-84` `pub fn compute_total(children: &[MarksRegisterChild], exam: &Exam, scale: &[MarksGrade]) -> (TotalMarks, Gpa, Grade, ResultStatus)`.

---

### FINDING 52 (id: `DOMAIN-ASS-052`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:1240-1253` and `docs/specs/assessment/services.md:86-89`

**Description:**

The `ResultService::determine_pass_fail` method takes `(marks: &[f32], pass_marks: &[f32])` instead of `(children: &[MarksRegisterChild], exam: &Exam)`. The function does not consume `MarksRegisterChild` and therefore cannot read the `is_absent` flag.

**Expected:**

`pub fn determine_pass_fail(children: &[MarksRegisterChild], exam: &Exam) -> ResultStatus` per `docs/specs/assessment/services.md:86-89`.

**Evidence:**

`crates/domains/assessment/src/services.rs:1240-1243` `pub fn determine_pass_fail(marks: &[f32], pass_marks: &[f32]) -> crate::value_objects::ResultStatus { ... }`. `docs/specs/assessment/services.md:86-89` `pub fn determine_pass_fail(children: &[MarksRegisterChild], exam: &Exam) -> ResultStatus`.

---

### FINDING 53 (id: `DOMAIN-ASS-053`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:1260-1278` and `docs/specs/assessment/services.md:91-97`

**Description:**

The `ResultService::rank_section` method takes `totals: &[f32]` and returns `Vec<u32>` (just rank positions), but the spec mandates `(results: &[ResultStore]) -> Vec<MeritPosition>`. The function therefore cannot materialise `MeritPosition` rows. The function also mis-implements the spec's "positions skip the next integer on ties" invariant — it skips by `j - i + 1` (the number of tied students) which is correct only for one tie group at the start; ties later in the order produce a skip of the tied count not the tied count + previous count.

**Expected:**

`pub fn rank_section(results: &[ResultStore]) -> Vec<MeritPosition>` per `docs/specs/assessment/services.md:91-93`.

**Evidence:**

`crates/domains/assessment/src/services.rs:1260-1278` `pub fn rank_section(totals: &[f32]) -> Vec<u32> { ... }`. `docs/specs/assessment/services.md:91-93` `pub fn rank_section(results: &[ResultStore]) -> Vec<MeritPosition>`. The doc comment on line 1258-1259 says "tied ranks get the same position; positions skip integers on ties" but the implementation skips by `j - i + 1` per tie group (line 1274) which produces standard competition ranking only when no tie straddles a non-tied group.

---

### FINDING 54 (id: `DOMAIN-ASS-054`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:1283-1285` and `docs/specs/assessment/services.md:95-97`

**Description:**

The `ResultService::rank_all_sections` method is implemented as `Self::rank_section(totals)` (one line wrapper) but the spec mandates `rank_all_sections(results: &[ResultStore]) -> Vec<AllExamWisePosition>`. The function therefore returns the wrong type (`Vec<u32>` instead of `Vec<AllExamWisePosition>`).

**Expected:**

`pub fn rank_all_sections(results: &[ResultStore]) -> Vec<AllExamWisePosition>` per `docs/specs/assessment/services.md:95-97`.

**Evidence:**

`crates/domains/assessment/src/services.rs:1283-1285` `pub fn rank_all_sections(totals: &[f32]) -> Vec<u32> { Self::rank_section(totals) }`. `docs/specs/assessment/services.md:95-97` `pub fn rank_all_sections(results: &[ResultStore]) -> Vec<AllExamWisePosition>`.

---

### FINDING 55 (id: `DOMAIN-ASS-055`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:1290-1300` and `docs/specs/assessment/services.md:286-288`

**Description:**

The `ResultService::validate_no_overlap` method delegates to the `MarksGradeScale` port's `validate()` method (lines 1290-1300) instead of walking the scale's rows itself. The spec defines both `MarksGradeService::validate_no_overlap(scale: &[MarksGrade])` and `ResultService` methods that take a `&[MarksGrade]`. The current implementation conflates the two services and provides no actual overlap detection.

**Expected:**

A standalone `pub fn validate_no_overlap(scale: &[MarksGrade]) -> Result<(), ValidationError>` that walks the rows and rejects overlapping percent ranges, per `docs/specs/assessment/services.md:286-288`.

**Evidence:**

`crates/domains/assessment/src/services.rs:1290-1300` `pub fn validate_no_overlap(_scale: &dyn crate::commands::MarksGradeScale) -> educore_core::error::Result<()> { if !_scale.validate() { return Err(DomainError::validation("grade scale has overlapping ranges")); } Ok(()) }`. `docs/specs/assessment/services.md:286-288` `pub fn validate_no_overlap(scale: &[MarksGrade]) -> Result<(), ValidationError>`.

---

### FINDING 56 (id: `DOMAIN-ASS-056`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:1326-1356` and `docs/specs/assessment/services.md:99-105`

**Description:**

The `ResultService::build_result_store` method takes 12 positional arguments (lines 1326-1340) instead of the spec's signature `pub fn build_result_store(exam: &Exam, setup: &ExamSetup, student: &StudentRecord, children: &[MarksRegisterChild], scale: &[MarksGrade]) -> ResultStore`. The current signature forces the caller to have already-materialised totals/grades (the spec says the service computes them from children + scale).

**Expected:**

`pub fn build_result_store(exam: &Exam, setup: &ExamSetup, student: &StudentRecord, children: &[MarksRegisterChild], scale: &[MarksGrade]) -> ResultStore` per `docs/specs/assessment/services.md:99-105`.

**Evidence:**

`crates/domains/assessment/src/services.rs:1326-1340` 12-argument `pub fn build_result_store(...) -> crate::aggregate::ResultStore`. `docs/specs/assessment/services.md:99-105` 5-argument spec signature.

---

### FINDING 57 (id: `DOMAIN-ASS-057`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:1-1380` and `docs/specs/assessment/services.md:1-373`

**Description:**

The services file is missing 7 of 8 service structs the spec defines. Specifically missing: `ExamService` (services.md:8-30), `MarksService` (services.md:38-58), `ReportCardService` (services.md:130-146), `SeatPlanService` (services.md:155-178), `AdmitCardService` (services.md:187-201), `OnlineExamService` (services.md:208-238), `TeacherEvaluationService` (services.md:250-275), `MarksGradeService` (services.md:283-298). Only `ResultService` ships (and even that is missing its `publish` method, see Finding DOMAIN-ASS-042).

**Expected:**

8 service structs per the spec.

**Evidence:**

`crates/domains/assessment/src/services.rs` declares 1 `pub struct` (line 1176). `docs/specs/assessment/services.md` declares 8 `pub struct` headers at lines 9, 39, 67, 130, 155, 187, 208, 250, 283.

---

### FINDING 58 (id: `DOMAIN-ASS-058`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:643-645` and `docs/specs/assessment/services.md:301-345`

**Description:**

The services file ships the `school_matches` helper (lines 643-645) but the spec defines 4 policy/specification structs: `ResultEligibility` (services.md:303-310), `AdmitCardEligibility` (services.md:316-321), `ActiveExamSchedule` (services.md:323-332), `PendingOnlineExam` (services.md:335-345). None of these are implemented.

**Expected:**

All 4 policy/specification structs.

**Evidence:**

`crates/domains/assessment/src/services.rs:643-645` `pub fn school_matches(ctx: &TenantContext, school: SchoolId) -> bool { ctx.school_id == school }`. `docs/specs/assessment/services.md:303-345` defines 4 policy/specification structs.

---

### FINDING 59 (id: `DOMAIN-ASS-059`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:1155-1175` (doc comment) and `docs/specs/assessment/services.md:64-119`

**Description:**

The `ResultService` doc-comment claims it ships "a minimal table-driven implementation of the grade-computation rules. The full per-school grade scale, the validate-no-overlap / validate-contiguous invariants, and the merit-position ties (with skipped integers) land in a follow-up phase." This admits the spec is not honored. The spec mandates the full implementation, not a "follow-up" stub.

**Expected:**

A `ResultService` that honors the spec's full signature (Findings DOMAIN-ASS-049 to DOMAIN-ASS-056).

**Evidence:**

`crates/domains/assessment/src/services.rs:1161-1172` is the doc comment that defers the full implementation.

---

### FINDING 60 (id: `DOMAIN-ASS-060`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:981-998` and `docs/specs/assessment/commands.md:210-233`

**Description:**

The `enter_marks` service does NOT consult the `MarksRegisterRepository` to verify that the register is not cancelled, that the subject is part of the register, or that the exam is not yet published. The function just mints the `MarksEntered` event.

**Expected:**

Three pre-conditions checks: register-active, subject-in-register, exam-not-yet-published.

**Evidence:**

`crates/domains/assessment/src/services.rs:981-998` `pub fn enter_marks<C, G>(cmd: EnterMarksCommand, clock: &C, ids: &G) -> Result<MarksEntered> { let now = clock.now(); let event_id = ids.next_event_id(); Ok(MarksEntered::new(cmd.marks_register_id, cmd.subject_id, cmd.student_id, cmd.marks, cmd.is_absent, event_id, cmd.tenant.correlation_id, now)) }` — no validation. `docs/specs/assessment/commands.md:224-229` lists 3 pre-conditions.

---

### FINDING 61 (id: `DOMAIN-ASS-061`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:945-976` and `docs/specs/assessment/commands.md:190-208`

**Description:**

The `initialize_marks_register` service does NOT enforce the spec's 3 pre-conditions: the exam exists, the schedule exists, and the students are enrolled in the section. The function just mints one register per command (one student), not one per student in the section as the spec mandates.

**Expected:**

Three pre-conditions checks + a per-student register creation loop.

**Evidence:**

`crates/domains/assessment/src/services.rs:945-976` `pub fn initialize_marks_register<...>(cmd, clock, ids) -> Result<...> { ... let aggregate = crate::aggregate::MarksRegister::fresh(cmd.marks_register_id, cmd.exam_id, cmd.student_id, cmd.class_id, cmd.section_id, cmd.academic_year_id, ...); ... }` — single register per command. `docs/specs/assessment/commands.md:203-208` mandates "Creates one MarksRegister per student in the section".

---

### FINDING 62 (id: `DOMAIN-ASS-062`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:1055-1076` and `docs/specs/assessment/commands.md:253-274`

**Description:**

The `publish_result` service does NOT enforce the spec's 3 pre-conditions: all marks have been submitted, the school has a non-empty MarksGrade scale, and the exam is not yet published (or is being republished). The function just mints a `ResultPublished` event with `student_count: 0`.

**Expected:**

Three pre-conditions checks + a per-student `ResultStore` materialisation.

**Evidence:**

`crates/domains/assessment/src/services.rs:1055-1076` `pub fn publish_result<...>(cmd, clock, ids) -> Result<ResultPublished> { let now = clock.now(); let event_id = ids.next_event_id(); Ok(ResultPublished::new(cmd.exam_id, cmd.class_id, cmd.section_id, cmd.academic_year_id, 0, cmd.published_at, event_id, cmd.tenant.correlation_id)) }` — no validation. `docs/specs/assessment/commands.md:265-274` lists 3 pre-conditions and the `Materializes ResultStore rows, computes MeritPosition, ExamWisePosition, and AllExamWisePosition, emits ResultPublished per student` effects.

---

### FINDING 63 (id: `DOMAIN-ASS-063`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:1004-1027` and `docs/specs/assessment/commands.md:235-251`

**Description:**

The `submit_marks` service does NOT enforce the spec's 2 pre-conditions: all `MarksRegisterChild` rows are present (or the school's `ExamStepSkip` allows partial), and the exam is not yet published. The function just mints a `MarksSubmitted` event with `subject_count: 0` and nil-UUID `exam_id`/`class_id`/`section_id` (Finding DOMAIN-ASS-029).

**Expected:**

Two pre-conditions checks.

**Evidence:**

`crates/domains/assessment/src/services.rs:1004-1027` `pub fn submit_marks<...>(cmd, clock, ids) -> Result<MarksSubmitted> { ... let _placeholder_exam = ExamId::new(cmd.marks_register_id.school_id(), uuid::Uuid::nil()); ... Ok(MarksSubmitted::new(cmd.marks_register_id, _placeholder_exam, _placeholder_class, _placeholder_section, 0, ...)) }` — no validation. `docs/specs/assessment/commands.md:245-251` lists 2 pre-conditions.

---

### FINDING 66 (id: `DOMAIN-ASS-066`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:18` and `docs/build-plan.md:661-663`

**Description:**

The build-plan § "Phase 4 Risks" states: "Result computation is policy-heavy. Mitigation: keep all grading rules in `policies.rs` as pure functions with table-driven fixtures." The assessment crate does not have a `policies.rs` file. The grading rules are inlined in `services.rs` as methods of `ResultService` and use hard-coded A-F thresholds.

**Expected:**

A `policies.rs` module containing the table-driven grading fixtures.

**Evidence:**

`docs/build-plan.md:661-663` "Result computation is policy-heavy. Mitigation: keep all grading rules in policies.rs as pure functions with table-driven fixtures." `crates/domains/assessment/src/services.rs:1-1380` — no `policies.rs`; the `ResultService::compute_grade` method at lines 1182-1203 hard-codes the thresholds in a chain of `if percent >= 90.0 ... else if percent >= 80.0 ...`.

---

### FINDING 69 (id: `DOMAIN-ASS-069`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:981-998` and `docs/specs/assessment/aggregates.md:202-220`

**Description:**

The `enter_marks` service does not enforce the `MarksRegisterChild` invariants the spec mandates: if `is_absent=true` then `Marks` is treated as zero (and the school absent rule is applied), `Marks >= 0`, and `Marks <= FullMark`. None of these checks are performed; the function just passes the raw `f32` into the event.

**Expected:**

A validation step that consults the `MarksRegisterChild` and applies the absent rule + the `Marks <= FullMark` check.

**Evidence:**

`crates/domains/assessment/src/services.rs:981-998` `pub fn enter_marks<...>(cmd, clock, ids) -> Result<MarksEntered> { ... Ok(MarksEntered::new(cmd.marks_register_id, cmd.subject_id, cmd.student_id, cmd.marks, cmd.is_absent, ...)) }` — no validation. `docs/specs/assessment/aggregates.md:215-219` lists 4 invariants.

---

### FINDING 71 (id: `DOMAIN-ASS-071`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/events.rs:1196-1212` and `docs/events/assessment.md:25`

**Description:**

The `ResultPublished` event declares `AGGREGATE_TYPE: "result_store"` (line 1199) but `aggregate_id` returns `self.exam_id.as_uuid()` (line 1204) — not the `ResultStore` id. The events catalog at `docs/events/assessment.md:25` lists the aggregate as `Result`. The mismatch means the event's aggregate id and aggregate type do not agree (a `result_store` aggregate type but an `exam` aggregate id).

**Expected:**

`aggregate_id` returns the result_store's id, or `AGGREGATE_TYPE` is `"result"` and `aggregate_id` returns the result_store's id.

**Evidence:**

`crates/domains/assessment/src/events.rs:1196-1212` `const AGGREGATE_TYPE: &'static str = "result_store"; ... fn aggregate_id(&self) -> uuid::Uuid { self.exam_id.as_uuid() }`.

---

### FINDING 72 (id: `DOMAIN-ASS-072`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/value_objects.rs:24-26, 145-166` and `docs/specs/assessment/aggregates.md:3-41, 84-114, 152-167, 685-749`

**Description:**

The crate's `value_objects.rs` defines 2 placeholder typed ids (`StaffId` lines 151-156, `ClassRoomId` lines 158-166) with comments "Placeholder until the HR domain ships its Staff aggregate in Phase 6" / "Placeholder until the facilities domain ships its Room aggregate in Phase 8". These placeholders are typed wrappers around `Uuid` and look semantically equivalent to the real ids that will land in later phases. The 5 other crates in the workspace will need to handle the transition from placeholder to real id (e.g., `educore-finance` may need `StaffId` for fee exemption rules). The crate has no migration plan or compatibility shim documented.

**Expected:**

A `phase-6-compatibility.md` / `phase-8-compatibility.md` doc, or a `From<StaffId> for educore_hr::StaffId` shim, or a typed-id registry that names which crate owns the canonical id.

**Evidence:**

`crates/domains/assessment/src/value_objects.rs:151-156` `/// A typed id for a Staff aggregate (the invigilating teacher for an exam). Placeholder until the HR domain ships its \`Staff\` aggregate in Phase 6. pub struct StaffId;`. Similar at lines 158-166 for `ClassRoomId`.

---

### FINDING 74 (id: `DOMAIN-ASS-074`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:77-161` (create_exam) and `docs/specs/assessment/commands.md:62-86`

**Description:**

The `create_exam` service mints the `ExamCreated` event with the `name` and `code` as raw `String` (events.rs:107-110) instead of typed `ExamName` and `ExamCode` newtypes. The spec's event definition at `docs/specs/assessment/events.md:59-69` does not show the `name`/`code` fields, but if the engine policy is to use typed wrappers, the event's wire format should also use them.

**Expected:**

The event's wire format uses the typed wrappers (or a documented rationale for the deviation).

**Evidence:**

`crates/domains/assessment/src/events.rs:107-110` `name: name.as_str().to_owned(), code: code.as_str().to_owned(), exam_mark: exam_mark.as_f32(), pass_mark: pass_mark.as_f32(),`. The engine rule at `AGENTS.md` is "Compile-time safety over strings."

---

### FINDING 80 (id: `DOMAIN-ASS-080`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:1011-1027` and `docs/specs/assessment/services.md:54-58`

**Description:**

The `MarksService::submit` function (services.md:54-58) enforces the partial-submission rule (rejects partial submissions unless the school has configured `ExamStepSkip`). The shipped `submit_marks` function (services.rs:1004-1027) does NOT consult the `ExamStepSkip` setting and does NOT check that all `MarksRegisterChild` rows are present.

**Expected:**

`MarksService::submit` (or equivalent) that consults `ExamStepSkip` and either allows or rejects the submission.

**Evidence:**

`crates/domains/assessment/src/services.rs:1004-1027` (no `ExamStepSkip` consultation). `docs/specs/assessment/services.md:54-58` `pub fn submit(register: &mut MarksRegister) -> Result<(), ValidationError>` and the doc-comment "MarksService::submit enforces partial-submission rules: when exam_step_skips indicates partial submission is allowed, missing subjects are tolerated; otherwise the register must be complete."

---

### FINDING 82 (id: `DOMAIN-ASS-082`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/aggregate.rs:58-107` and `docs/specs/assessment/aggregates.md:65-69`

**Description:**

The `Exam` aggregate has a `pass_mark: PassMark` field and an `exam_mark: ExamMark` field, but no `PassMark` validation that the constructor `Exam::fresh` enforces. The `Exam::fresh` constructor at `crates/domains/assessment/src/aggregate.rs:116-156` does NOT check `pass_mark.as_f32() <= exam_mark.as_f32()`. The service `create_exam` does the check (services.rs:97-103), but the aggregate itself is constructed without the invariant — bypassing the service breaks the invariant.

**Expected:**

The `Exam::fresh` constructor enforces `pass_mark <= exam_mark` (or returns a `Result<Self, DomainError>`).

**Evidence:**

`crates/domains/assessment/src/aggregate.rs:116-156` `pub fn fresh(...pass_mark: PassMark, ..., exam_mark: ExamMark, ...) -> Self { Self { ... pass_mark, exam_mark, ... } }` — no check. The check is in `services.rs:97-103`.

---

### FINDING 84 (id: `DOMAIN-ASS-084`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/aggregate.rs:432-491` and `docs/specs/assessment/aggregates.md:751-803`

**Description:**

The `AdmitCard` aggregate has an `active_status: ActiveStatus` field that is used as a soft-delete flag, but the `cancel_admit_card` service does not take a `&mut AdmitCard` parameter (services.rs:608-631) — it does. However, the `generate_admit_card` service constructs a new `AdmitCard` but the spec mandates "Once generated, the card is immutable; a re-generation supersedes the previous card with a new id and emits a new event" (aggregates.md:768-769). The `regenerate_admit_card` service does NOT mutate the previous card's `active_status` to `Retired`, leaving the old card active and the uniqueness invariant broken.

**Expected:**

`regenerate_admit_card` takes a `&mut AdmitCard` for the previous card and sets `active_status = Retired` before emitting the new event.

**Evidence:**

`crates/domains/assessment/src/services.rs:585-604` `pub fn regenerate_admit_card<...>(cmd, clock, _ids) -> Result<AdmitCardRegenerated> { ... Ok(AdmitCardRegenerated::new(cmd.admit_card_id, cmd.previous_id, cmd.reason, ...)) }` — no aggregate mutation. `docs/specs/assessment/aggregates.md:768-769` "Once generated, the card is immutable; a re-generation supersedes the previous card with a new id and emits a new event."

---

### FINDING 90 (id: `DOMAIN-ASS-090`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/events.rs:155-180` (`ExamUpdated`) and `docs/specs/assessment/aggregates.md:46-82`

**Description:**

The `ExamUpdated` event stores the changes as `Vec<String>` (events.rs:153), but the spec describes `Vec<&'static str>` (events.md:74). The wire format mismatch means a deserialised event would not round-trip through a strict schema validator. Additionally, the event has no `old_value: Option<...>` and `new_value: Option<...>` fields (which would be needed for downstream subscribers to know what changed).

**Expected:**

A change-list with stable, namespaced identifiers (e.g., `"exam_mark"`, `"pass_mark"`) as the spec mandates, or a richer diff payload.

**Evidence:**

`crates/domains/assessment/src/events.rs:153` `pub changes: Vec<String>,`. `docs/specs/assessment/events.md:74` `pub changes: Vec<&'static str>,`.

---

### FINDING 97 (id: `DOMAIN-ASS-097`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `docs/coverage.toml:555-624` and `docs/specs/assessment/aggregates.md:1-1025`

**Description:**

The 8 assessment coverage rows in `coverage.toml` are marked `status = "Tested"` but the 8 aggregates they reference (`assessment_exams_aggregate`, `assessment_marks_registers_aggregate`, `assessment_exam_schedules_aggregate`, `assessment_result_stores_aggregate`, `assessment_report_cards_aggregate`, `assessment_online_exams_aggregate`, `assessment_seat_plans_aggregate`, `assessment_admit_cards_aggregate`) are not all implemented. Specifically: (a) `OnlineExam` is not implemented (Finding DOMAIN-ASS-002), (b) `ReportCard` is documented as a "projection" without a backing aggregate (handoff:67-71). Marking these `Tested` overstates the coverage.

**Expected:**

The `Tested` status reflects actual coverage. `OnlineExam` and `ReportCard` should be `Pending` or `Partial` until they are implemented and tested.

**Evidence:**

`docs/coverage.toml:600-606` `status = "Tested"` for `assessment_online_exams_aggregate`. `docs/coverage.toml:591-597` `status = "Tested"` for `assessment_report_cards_aggregate` (note "projection" in the item field). `docs/handoff/PHASE-4-HANDOFF.md:67-71` "The full state machine ships at the Event level (8 events) but the integration test only exercises create_exam".

---

### FINDING 98 (id: `DOMAIN-ASS-098`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/lib.rs:14-17` and `docs/build-plan.md:665-705`

**Description:**

The build-plan § "Phase 4 outcome" declares the phase "Closed 2026-06-12" with 433 tests passing. The actual state of the crate has 6 of 8 prompt-named aggregates (Phase 4 build-plan task 1), 21 of 62 spec commands, 21 of 65 spec events, 1 of 8 service structs, 6 of 13 repository traits, 0 of 47 `#[derive(DomainQuery)]` emissions, 0 workflow integration tests, and `Uuid::nil()` placeholders in production event payloads (Findings DOMAIN-ASS-029 to DOMAIN-ASS-031). The phase is not "closed" in any production-ready sense.

**Expected:**

Either re-open the phase until the spec is honored, or formally defer the missing scope to a follow-up phase with a clear manifest of what is and is not in Phase 4.

**Evidence:**

`docs/build-plan.md:665-705` `**Phase 4 outcome.** Closed 2026-06-12. **`educore-assessment`** delivered as the second domain crate. The full prompt-named subset ships: 8 aggregates ... 28 typed commands, 28 typed events ... 25+ pure factory services, 8 repository port traits, 8 typed query stubs`. The code at `crates/domains/assessment/src/` ships 6 aggregates, 21 commands, 21 events, 1 service struct, 6 repository traits, 6 query stubs.

---

### FINDING 99 (id: `DOMAIN-ASS-099`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/value_objects.rs:151-156, 158-166` and `crates/domains/assessment/src/aggregate.rs:280, 284, 439`

**Description:**

The crate defines placeholder `StaffId` and `ClassRoomId` typed ids at `value_objects.rs:151-156, 158-166` and uses them in `ExamSchedule::room_id` and `teacher_id` (aggregate.rs:280, 284) and in `AdmitCard` (via command fields at commands.rs:189, 247). When Phase 6 (HR) and Phase 8 (Facilities) ship the canonical `StaffId` and `ClassRoomId` (per `docs/specs/assessment/dependencies` and the comment in value_objects.rs:151-156), the placeholder types will collide with the canonical types. No `From` conversion, type alias, or migration plan is documented.

**Expected:**

Either (a) the placeholder types are removed and the foreign-key fields are deferred until Phase 6/8, or (b) explicit `From` conversions are provided so downstream crates can migrate.

**Evidence:**

`crates/domains/assessment/src/value_objects.rs:151-156` `pub struct StaffId;` and `:158-166` `pub struct ClassRoomId;` with "Placeholder until the HR domain ships" comments. The handoff at `docs/handoff/PHASE-4-HANDOFF.md:284-287` says "placeholder typed ids for StaffId / ClassRoomId (the academic crate's missing ids; the full definitions land in the HR workstream in Phase 6 + the facilities workstream in Phase 8)".

---

### FINDING 40 (id: `DOMAIN-ASS-040`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/commands.rs:60-68` and `docs/code-standards.md`

**Description:**

The nine `ASSESSMENT_*_COMMAND_TYPE` constants on lines 60-68 (`ASSESSMENT_EXAM_SCHEDULE_CREATE/UPDATE/CANCEL_COMMAND_TYPE`, `ASSESSMENT_SEAT_PLAN_GENERATE/UPDATE/CANCEL_COMMAND_TYPE`, `ASSESSMENT_ADMIT_CARD_GENERATE/REGENERATE/CANCEL_COMMAND_TYPE`) have no rustdoc. The first three (`ASSESSMENT_EXAM_CREATE/UPDATE/DELETE_COMMAND_TYPE`) are documented (lines 47-49, 52-54, 56-58). The `deny(missing_docs)` lint at `lib.rs:10` is at the file level, so the constant-level docs are required.

**Expected:**

`///` doc comments on every `pub const` per the engine's `deny(missing_docs)` policy.

**Evidence:**

`crates/domains/assessment/src/commands.rs:60-68` declares the 9 constants with no `///` lines above them. `crates/domains/assessment/src/lib.rs:10` `#![deny(missing_docs)]`.

---

### FINDING 41 (id: `DOMAIN-ASS-041`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/entities.rs:113-150` (`MarksRegisterChild`) and `docs/specs/assessment/aggregates.md:202-220`

**Description:**

The `MarksRegisterChild` entity stores `comment: Option<String>` (line 140) but the spec's `MarksEntered` event carries `comments: Option<String>` (plural) at `docs/specs/assessment/events.md:139`. There is no path for the comment to be populated from the `EnterMarksCommand` to the `MarksRegisterChild` aggregate (the `enter_marks` service emits `MarksEntered` without a comment field, and the repository does not have `upsert_child`/`list_children` methods, see Finding DOMAIN-ASS-009).

**Expected:**

The `MarksEntered` event carries the `comments` field (Finding DOMAIN-ASS-019) and the `enter_marks` service writes through to a `MarksRegisterChild` row.

**Evidence:**

`crates/domains/assessment/src/entities.rs:140` `pub comment: Option<String>`. `services.rs:981-998` `enter_marks` does not persist to a `MarksRegisterChild` row.

---

### FINDING 47 (id: `DOMAIN-ASS-047`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/lib.rs:10` and `crates/domains/assessment/src/aggregate.rs:17, value_objects.rs:28, commands.rs:23, events.rs:18, repository.rs:15`

**Description:**

The `lib.rs` enforces `#![deny(missing_docs)]` (line 10), but every internal module file (aggregate.rs, value_objects.rs, commands.rs, events.rs, repository.rs) has its own `#![allow(missing_docs)]` override. The crate-level `deny` and the module-level `allow` work in the compiler, but the policy is contradictory: the crate's public surface (`prelude::*` re-exports ~30+ items) is documented through the original module's doc, while the `deny` would seem to require docs on every public item. The `aggregate.rs:17-22` comment ("described by their type names; suppressing this lint for the file is the pragmatic choice for the 8 aggregates Phase 4 ships") concedes the suppression is for convenience.

**Expected:**

A consistent doc policy. Either (a) the crate does not have `deny(missing_docs)` and instead relies on per-module decisions, or (b) every public item gets a rustdoc.

**Evidence:**

`crates/domains/assessment/src/lib.rs:10` `#![deny(missing_docs)]`. `crates/domains/assessment/src/aggregate.rs:17` `#![allow(missing_docs)]`. `crates/domains/assessment/src/value_objects.rs:28` `#![allow(missing_docs)]`. `crates/domains/assessment/src/commands.rs:23` `#![allow(missing_docs)]`. `crates/domains/assessment/src/events.rs:18` `#![allow(missing_docs)]`. `crates/domains/assessment/src/repository.rs:15` `#![allow(missing_docs)]`.

---

### FINDING 64 (id: `DOMAIN-ASS-064`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:317-357` (and parallel `update_exam_schedule`, `cancel_exam_schedule`, `generate_seat_plan`, `update_seat_plan`, `cancel_seat_plan`, `generate_admit_card`, `regenerate_admit_card`, `cancel_admit_card`, `initialize_marks_register`, `enter_marks`, `submit_marks`, `cancel_marks_register`, `publish_result`, `republish_result`, `update_result_remarks`, `generate_report_card`)

**Description:**

The Workstream B and C service functions all use `_cmd`, `_ctx`, or `_ids` parameter names with leading underscores (e.g. lines 318, 415, 525, 585, 588, 1012-1014). The leading underscore is the conventional Rust signal that a parameter is intentionally unused; the corresponding `clippy::used_underscore_binding` lint would catch the contradiction. The functions actually do use most of the parameters in the event minting; the underscores are an artefact of the "minimal-shape pure factory functions" template.

**Expected:**

Rename parameters without leading underscores to reflect actual usage.

**Evidence:**

`crates/domains/assessment/src/services.rs:318` `pub fn schedule_exam<C, G>(_cmd: ScheduleExamCommand, clock: &C, ids: &G)`. `services.rs:415` `pub fn cancel_exam_schedule<C, G>(schedule: &mut ExamSchedule, cmd: CancelExamScheduleCommand, clock: &C, _ids: &G)`. `services.rs:1012-1014` uses `let _placeholder_exam = ...`.

---

### FINDING 65 (id: `DOMAIN-ASS-065`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/lib.rs:48-54` (re-export list) and `docs/specs/assessment/value-objects.md:11-117`

**Description:**

The `lib.rs` re-export list claims to expose "Typed ids and value objects the assessment crate re-exports for downstream consumers" but omits several defined in the same crate's `value_objects.rs`, including the `MarksGradeRow` (defined at value_objects.rs:428), `StaffId` placeholder (value_objects.rs:155), and `ClassRoomId` placeholder (value_objects.rs:165). It also omits the `ResultPublished` event field types and the `ResultStoreId` is included but `ResultStore` aggregate is not. Downstream consumers cannot easily use the re-exports for the full spec surface.

**Expected:**

Re-exports for every pub item a downstream consumer would use.

**Evidence:**

`crates/domains/assessment/src/lib.rs:48-54` lists 32 re-exports. `value_objects.rs:155, 165, 428` define `StaffId`, `ClassRoomId`, `MarksGradeRow` which are not in the re-export list.

---

### FINDING 67 (id: `DOMAIN-ASS-067`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:1176-1357` and `docs/specs/assessment/services.md:99-105, 107-110`

**Description:**

The spec defines `ResultService::build_custom_temporary(result: &ResultStore, custom: &CustomResultSetting) -> CustomTemporaryResult` (services.md:107-110) and `ResultService::publish` (services.md:112-118). Neither ships in the code.

**Expected:**

Both methods on `ResultService`.

**Evidence:**

`docs/specs/assessment/services.md:107-110` `pub fn build_custom_temporary(result: &ResultStore, custom: &CustomResultSetting) -> CustomTemporaryResult`; `docs/specs/assessment/services.md:112-118` `pub fn publish(exam, section, registers, scale) -> Result<PublishOutcome, ValidationError>`. Neither is in `crates/domains/assessment/src/services.rs:1176-1357`.

---

### FINDING 68 (id: `DOMAIN-ASS-068`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/commands.rs:50-68` and `docs/specs/assessment/events.md:512`

**Description:**

The crate ships 12 `ASSESSMENT_*_COMMAND_TYPE` constants but no `ASSESSMENT_ONLINE_EXAM_*`, `ASSESSMENT_MARKS_GRADE_*`, `ASSESSMENT_EXAM_SETTING_*`, etc. constants. The spec's full event catalog and the `docs/commands/assessment.md` table require ~62 command types. Only the 12 workstream-A + B + C constants are present.

**Expected:**

62 `ASSESSMENT_*_COMMAND_TYPE` constants matching the command catalog.

**Evidence:**

`crates/domains/assessment/src/commands.rs:50-68` declares 12 constants. `docs/commands/assessment.md:12-74` lists 62 command rows.

---

### FINDING 70 (id: `DOMAIN-ASS-070`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:1004-1027` and `docs/specs/assessment/commands.md:235-251`

**Description:**

The `submit_marks` service emits a `MarksSubmitted` event with `subject_count: 0` (line 1022) — a hardcoded value, not the actual count. The spec mandates the real `subject_count` so downstream services can correlate.

**Expected:**

`subject_count: u32` populated from the actual `MarksRegisterChild` row count.

**Evidence:**

`crates/domains/assessment/src/services.rs:1022` `0,` (the 5th argument of `MarksSubmitted::new`).

---

### FINDING 75 (id: `DOMAIN-ASS-075`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:585-604` and `docs/specs/assessment/commands.md:458-471`

**Description:**

The `regenerate_admit_card` service does not check that the previous admit card exists, is not already cancelled, or that the regeneration reason is non-empty. The function just mints a new event.

**Expected:**

Three pre-conditions checks: previous-card-exists, previous-card-not-cancelled, reason-non-empty.

**Evidence:**

`crates/domains/assessment/src/services.rs:585-604` `pub fn regenerate_admit_card<...>(cmd, clock, _ids) -> Result<AdmitCardRegenerated> { ... Ok(AdmitCardRegenerated::new(cmd.admit_card_id, cmd.previous_id, cmd.reason, event_id, cmd.tenant.correlation_id, now)) }` — no validation. `docs/specs/assessment/commands.md:464-471` describes the spec's `SetExamSignature` (different command) and the spec at `docs/specs/assessment/aggregates.md:768-769` "Once generated, the card is immutable; a re-generation supersedes the previous card with a new id and emits a new event."

---

### FINDING 76 (id: `DOMAIN-ASS-076`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/repository.rs:39-235` and `docs/specs/assessment/permissions.md:39`

**Description:**

The spec at `docs/specs/assessment/permissions.md:39` lists `ExamSchedule.Read` as a capability. The repository's `ExamScheduleRepository` does not consult the RBAC subsystem; it accepts any `TenantContext` and assumes the actor is authorized. The trait has no awareness of the `Capability` enum.

**Expected:**

Either the trait methods take a `Capability` parameter, or the dispatcher (caller of the trait) is documented as the enforcement point.

**Evidence:**

`crates/domains/assessment/src/repository.rs:115-143` defines 6 methods that all take `&TenantContext` and return `Result<...>` with no capability check. `docs/specs/assessment/permissions.md:37-39` lists `ExamSchedule.Read` as a distinct capability.

---

### FINDING 77 (id: `DOMAIN-ASS-077`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/aggregate.rs:58-107` (`Exam`) and `docs/specs/assessment/commands.md:88-103`

**Description:**

The `Exam` aggregate has no `is_locked: bool` or `locked_at: Option<Timestamp>` field, but the spec at `docs/specs/assessment/commands.md:100-102` says "Marks have not yet been entered against the exam. Once marks exist, the exam is locked." The `ExamService::lock_after_publish` method (services.md:28-29) is missing (see Finding DOMAIN-ASS-057) so the lock invariant is unenforceable.

**Expected:**

`pub is_locked: bool` and `pub locked_at: Option<Timestamp>` fields on `Exam`, plus the `ExamService::lock_after_publish` method.

**Evidence:**

`crates/domains/assessment/src/aggregate.rs:58-107` defines 18 fields, none of which is `is_locked`. `docs/specs/assessment/commands.md:99-102` describes the lock invariant. `docs/specs/assessment/services.md:28-29` describes `ExamService::lock_after_publish`.

---

### FINDING 78 (id: `DOMAIN-ASS-078`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/aggregate.rs:58-107` and `docs/specs/assessment/commands.md:62-86`

**Description:**

The `Exam` aggregate's `name: ExamName` and `code: ExamCode` fields are typed wrappers, but the aggregate's `fresh` constructor takes `name: ExamName, code: ExamCode` (aggregate.rs:123-124) directly. The corresponding `CreateExamCommand` carries `name: String, code: String` (commands.rs:95-97), so the service must validate the strings into newtypes (services.rs:91-92). A better design would have the command carry the typed wrappers, eliminating the validation-on-service-boundary pattern.

**Expected:**

`CreateExamCommand` carries `name: ExamName, code: ExamCode` (see Finding DOMAIN-ASS-011).

**Evidence:**

`crates/domains/assessment/src/aggregate.rs:123-124` (constructor takes typed wrappers) vs. `crates/domains/assessment/src/commands.rs:95-97` (command carries `String`).

---

### FINDING 79 (id: `DOMAIN-ASS-079`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/lib.rs:60-67` and `docs/specs/assessment/services.md:303-345`

**Description:**

The `prelude` re-exports the `Capability` enum but does not re-export the `ResultEligibility` / `AdmitCardEligibility` / `ActiveExamSchedule` / `PendingOnlineExam` policies. The policies are not implemented (Finding DOMAIN-ASS-058) so the re-exports would be empty, but the spec mandates them and they should be in the public surface.

**Expected:**

Re-exports of the policy/specification types from the prelude once they are implemented.

**Evidence:**

`crates/domains/assessment/src/lib.rs:60-67` `pub use educore_rbac::value_objects::Capability;` only. `docs/specs/assessment/services.md:303-345` defines 4 policy/specification structs.

---

### FINDING 81 (id: `DOMAIN-ASS-081`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:643-645` (`school_matches`) and `docs/specs/assessment/services.md:301-345`

**Description:**

The `school_matches` helper duplicates the per-tenant check that should live in the engine facade (per the handoff at `docs/handoff/PHASE-4-HANDOFF.md:694-696`: "The capability check boundary was resolved as dispatcher-level"). The helper is re-exported from the prelude (lib.rs:105) and may be called inconsistently — services that DO NOT call it (e.g., `enter_marks`, `submit_marks`, `publish_result`, `regenerate_admit_card`, `republish_result`, `update_result_remarks`, `generate_report_card`, `cancel_*`) silently accept cross-tenant commands.

**Expected:**

The dispatcher enforces the school match; the per-service functions are documented as assuming the dispatcher has already done so.

**Evidence:**

`crates/domains/assessment/src/services.rs:643-645` `pub fn school_matches(ctx: &TenantContext, school: SchoolId) -> bool { ctx.school_id == school }`. `crates/domains/assessment/src/lib.rs:105` re-exports `school_matches` from the prelude. The Workstream B + C services (lines 317-1149) do not call `school_matches`.

---

### FINDING 83 (id: `DOMAIN-ASS-083`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/aggregate.rs:504-560` and `docs/specs/assessment/aggregates.md:169-220`

**Description:**

The `MarksRegister` aggregate's `is_open: bool` field is set to `true` in the `fresh` constructor and never set to `false` by any service. The spec says `is_open` is `true` while the register is being entered, `false` once `submit_marks` locks it. The `submit_marks` service (services.rs:1004-1027) mints a `MarksSubmitted` event but does not mutate the aggregate's `is_open` field.

**Expected:**

`submit_marks` mutates the aggregate's `is_open` to `false` before emitting the event.

**Evidence:**

`crates/domains/assessment/src/aggregate.rs:529-559` `pub fn fresh(...) -> Self { Self { ..., is_open: true, ... } }`. `crates/domains/assessment/src/services.rs:1004-1027` `pub fn submit_marks<...>(cmd: SubmitMarksCommand, clock: &C, ids: &G) -> Result<MarksSubmitted> { ... Ok(MarksSubmitted::new(...)) }` — no aggregate mutation.

---

### FINDING 85 (id: `DOMAIN-ASS-085`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/aggregate.rs:258-348` and `docs/specs/assessment/aggregates.md:130-138`

**Description:**

The `ExamSchedule` aggregate has no `academic_year_id: AcademicYearId` field. The spec at `docs/specs/assessment/aggregates.md:133-137` lists 5 invariants: uniqueness by `(exam_id, class_id, section_id)` per academic year, `StartTime < EndTime`, no teacher overlap, no room overlap, date within academic year. Without `academic_year_id`, the uniqueness and date-in-range invariants are unenforceable.

**Expected:**

`pub academic_year_id: AcademicYearId` on `ExamSchedule`.

**Evidence:**

`crates/domains/assessment/src/aggregate.rs:258-296` defines the `ExamSchedule` struct with 15 fields, none of which is `academic_year_id`. `docs/specs/assessment/aggregates.md:133` "Unique by (exam_id, class_id, section_id) per academic year" and line 137 "Date is within the academic year".

---

### FINDING 86 (id: `DOMAIN-ASS-086`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/aggregate.rs:360-420` and `docs/specs/assessment/aggregates.md:685-749`

**Description:**

The `SeatPlan` aggregate has no `academic_year_id: AcademicYearId` field. The spec at `docs/specs/assessment/aggregates.md:702-705` requires uniqueness by `(exam_type_id, class_id, section_id, academic_id)` — without the `academic_year_id` field, the invariant is unenforceable.

**Expected:**

`pub academic_year_id: AcademicYearId` on `SeatPlan`.

**Evidence:**

`crates/domains/assessment/src/aggregate.rs:360-383` defines the `SeatPlan` struct with 14 fields, none of which is `academic_year_id` or `exam_type_id`. The `generate_seat_plan` command is also missing `exam_type_id` (Finding DOMAIN-ASS-016).

---

### FINDING 87 (id: `DOMAIN-ASS-087`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/aggregate.rs:360-420` and `docs/specs/assessment/aggregates.md:685-749`

**Description:**

The `SeatPlan` aggregate has no `exam_type_id: ExamTypeId` field. The spec at `docs/specs/assessment/aggregates.md:702` requires uniqueness by `(exam_type_id, class_id, section_id, academic_id)`. The aggregate has only `exam_id: ExamId` (line 366).

**Expected:**

`pub exam_type_id: ExamTypeId` on `SeatPlan`.

**Evidence:**

`crates/domains/assessment/src/aggregate.rs:366` `pub exam_id: ExamId,` (no `exam_type_id`). `docs/specs/assessment/aggregates.md:702` "Unique by (exam_type_id, class_id, section_id, academic_id)".

---

### FINDING 88 (id: `DOMAIN-ASS-088`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/aggregate.rs:360-420` and `docs/specs/assessment/commands.md:408-435`

**Description:**

The `SeatPlan` aggregate's `total_students: u32` is stored as a top-level field (line 372) but the spec at `docs/specs/assessment/commands.md:431-432` says "Sum of assign_students equals the section's student count" — i.e., the invariant is a derived value, not a stored one. Storing it allows drift between the children and the parent.

**Expected:**

`total_students` is a derived accessor over `children.iter().map(|c| c.assign_students).sum::<u32>()`.

**Evidence:**

`crates/domains/assessment/src/aggregate.rs:372` `pub total_students: u32,` and `crates/domains/assessment/src/aggregate.rs:397` constructor argument. `docs/specs/assessment/commands.md:431-432` "Sum of assign_students equals the section's student count."

---

### FINDING 89 (id: `DOMAIN-ASS-089`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/aggregate.rs:258-348` and `docs/specs/assessment/aggregates.md:152-167`

**Description:**

The `ExamSchedule` aggregate's `room_id: Option<ClassRoomId>` and `teacher_id: Option<StaffId>` are top-level fields (lines 280, 284), but the spec at `docs/specs/assessment/aggregates.md:128-129` says these are per-subject overrides in `ExamScheduleSubject`. The aggregate conflates the per-schedule defaults with the per-subject overrides.

**Expected:**

The `ExamSchedule` aggregate's `room_id` and `teacher_id` are derived (or default) from `children.iter().all(|c| c.room_id == self.room_id)`. The per-subject room/teacher overrides live on `ExamScheduleSubject`.

**Evidence:**

`crates/domains/assessment/src/aggregate.rs:280, 284` `pub room_id: Option<ClassRoomId>, pub teacher_id: Option<StaffId>`. `docs/specs/assessment/aggregates.md:128-129` "the room the exam is held in (default for all subjects in this slot; per-subject overrides in ExamScheduleSubject)".

---

### FINDING 91 (id: `DOMAIN-ASS-091`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/events.rs:1141-1182` and `docs/specs/assessment/commands.md:641-653`

**Description:**

The `ResultRemarksUpdated` event's `teacher_remarks: String` is unbounded. The spec at `docs/specs/assessment/value-objects.md:81-82` mandates the `Remark` newtype with a 1..=2000 char bound. The event accepts any string, including empty or 10MB-long ones.

**Expected:**

`pub teacher_remarks: Remark` (typed wrapper, validated at construction).

**Evidence:**

`crates/domains/assessment/src/events.rs:1143` `pub teacher_remarks: String,`. `docs/specs/assessment/value-objects.md:81-82` `Remark | 1..2000 chars`. `crates/domains/assessment/src/commands.rs:646` `pub teacher_remarks: String,` (the command also carries the raw string).

---

### FINDING 92 (id: `DOMAIN-ASS-092`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/commands.rs:232-242` and `docs/specs/assessment/commands.md:176-188`

**Description:**

The `CancelExamScheduleCommand` carries `reason: String` (commands.rs:235) but has no length or non-empty validation. The spec describes the reason as a free-text field but it should at minimum be non-empty.

**Expected:**

A validation in the service that the reason is non-empty.

**Evidence:**

`crates/domains/assessment/src/commands.rs:235` `pub reason: String,` (no MAX_LEN constant, no validator function).

---

### FINDING 93 (id: `DOMAIN-ASS-093`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/commands.rs:330-340` (`CancelAdmitCardCommand`) and `docs/specs/assessment/commands.md:176-188`

**Description:**

The `CancelAdmitCardCommand` carries `reason: String` (commands.rs:333) without validation. The `regenerate_admit_card` service stores the reason in the event without checking that it is non-empty.

**Expected:**

A validator function for the cancel/regenerate reasons, or the typed `Reason` newtype.

**Evidence:**

`crates/domains/assessment/src/commands.rs:333` `pub reason: String,` (no MAX_LEN constant, no validator function).

---

### FINDING 94 (id: `DOMAIN-ASS-094`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/entities.rs:113-150` (`MarksRegisterChild`) and `docs/specs/assessment/aggregates.md:202-220`

**Description:**

The `MarksRegisterChild` entity's `gpa_point: Option<Gpa>` and `gpa_grade: Option<Grade>` fields are typed wrappers, but the constructor `MarksRegisterChild::fresh` does not exist (only field initialisation at lines 113-140). The `enter_marks` service (services.rs:981-998) does not compute the grade and grade point; the `MarksEntered` event does not carry them. The grading is deferred to the `submit_marks`/`publish_result` flow but no path from `MarksRegisterChild` to `MarksRegister`/`ResultStore` exists.

**Expected:**

Either the `enter_marks` service computes and stores the grade/grade point on the `MarksRegisterChild`, or a `compute_subject_marks` post-step is invoked.

**Evidence:**

`crates/domains/assessment/src/entities.rs:113-140` (no `fresh` constructor). `crates/domains/assessment/src/services.rs:981-998` `enter_marks` does not invoke `ResultService::compute_subject_marks`. `docs/specs/assessment/aggregates.md:215-218` lists 4 invariants.

---

### FINDING 95 (id: `DOMAIN-ASS-095`)

- **Source:** `docs/audit_reports/findings/wave1-assessment.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/assessment/src/services.rs:317-357` (`schedule_exam`) and `docs/specs/assessment/aggregates.md:130-138`

**Description:**

The `schedule_exam` service does not enforce the spec's invariant 5: "Date is within the academic year." The function accepts any `NaiveDate` and does not consult the `AcademicYear` aggregate to determine the year boundaries.

**Expected:**

A pre-condition check that consults the academic year boundaries.

**Evidence:**

`crates/domains/assessment/src/services.rs:317-357` (no date-range check). `docs/specs/assessment/aggregates.md:137` "Date is within the academic year."

---


## Attendance (target id prefix: `DOMAIN-ATT`)

**Path:** `crates/domains/attendance/`  
**Total findings:** 53 (26 critical, 16 high, 9 medium, 2 low)


### FINDING 1 (id: `DOMAIN-ATT-001`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `docs/specs/attendance/permissions.md:16-19` vs `crates/cross-cutting/rbac/src/value_objects.rs:153-201`

**Description:**

The spec mandates two-segment capability strings
  (`Attendance.Mark`, `Attendance.Update`, `Attendance.BulkMark`,
  `Attendance.Read`, `Attendance.Notify`, `Attendance.Report`,
  `Attendance.Subject.Mark`, `Attendance.Staff.Mark`,
  `Attendance.Import`, `Attendance.Import.Validate`,
  `Attendance.Import.Commit`, `Attendance.Import.Cancel`,
  `Attendance.Report.Daily`, `Attendance.Report.Weekly`, etc.) but
  the engine ships the three-segment form
  (`AttendanceStudentCreate`, `AttendanceStudentUpdate`,
  `AttendanceBulkMark`, `AttendanceNotify`,
  `AttendanceImportCreate`, `AttendanceImportValidate`, …). The
  capability string is a wire contract — consumers, audit logs,
  role catalogs, and event subscribers reference these strings —
  so this is a contract drift between the spec and the code.

**Expected:**

`docs/specs/attendance/permissions.md:16` lists
  `Attendance.Mark`, `Attendance.Update`, `Attendance.BulkMark`,
  `Attendance.Read`, `Attendance.Notify`, `Attendance.Report`.

**Evidence:**

`crates/cross-cutting/rbac/src/value_objects.rs:155-201`
  defines `AttendanceStudentCreate`, `AttendanceStudentUpdate`,
  `AttendanceStudentDelete`, `AttendanceSubjectNotify`,
  `AttendanceStaffCreate`, `AttendanceBulkMark`,
  `AttendanceImportCreate`, `AttendanceImportValidate`,
  `AttendanceReportRead`, `AttendanceNotify` (three-segment form)
  — no two-segment `Attendance.Mark` etc. The handoff at
  `docs/handoff/PHASE-5-HANDOFF.md:439-444` (OQ #3) acknowledges
  this divergence but does not resolve it.

---

### FINDING 10 (id: `DOMAIN-ATT-010`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/events.rs:1245-1254` vs `docs/specs/attendance/events.md:238-246`

**Description:**

`AbsenceNotificationRequested` is missing
  `requested_by: UserId` and `requested_at: Timestamp`. The
  audit trail cannot identify the actor who requested the
  notification.

**Expected:**

`docs/specs/attendance/events.md:238-246` — spec
  lists six payload fields beyond the event envelope
  (student_attendance_id, student_id, attendance_date, channel,
  template, requested_by, requested_at).

**Evidence:**

`crates/domains/attendance/src/events.rs:1246-1254`
  carries `student_attendance_id`, `student_id`,
  `attendance_date`, `channel`, `template`, and the metadata
  footer — no `requested_by` or `requested_at`.

---

### FINDING 11 (id: `DOMAIN-ATT-011`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/events.rs:644-651` vs `docs/specs/attendance/events.md:168-176`

**Description:**

`StaffAttendanceUpdated` carries
  `changes: Vec<String>` instead of the spec's
  `from_type: AttendanceType` / `to_type: AttendanceType` pair.
  Same pattern as the student-attendance updated event (see
  DOMAIN-ATT-002). HR/finance subscribers cannot detect
  absence transitions.

**Expected:**

`docs/specs/attendance/events.md:168-176`
  ```
  pub struct StaffAttendanceUpdated {
      pub staff_attendance_id: StaffAttendanceId,
      pub staff_id: StaffId,
      pub attendance_date: AttendanceDate,
      pub from_type: AttendanceType,
      pub to_type: AttendanceType,
      pub updated_by: UserId,
      pub updated_at: Timestamp,
  }
  ```

**Evidence:**

`crates/domains/attendance/src/events.rs:645-651`
  has `changes: Vec<String>` only. Missing `staff_id`,
  `attendance_date`, `from_type`, `to_type`, `updated_by`,
  `updated_at`.

---

### FINDING 12 (id: `DOMAIN-ATT-012`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/events.rs:454-461` vs `docs/specs/attendance/events.md:127-136`

**Description:**

`SubjectAttendanceUpdated` carries
  `changes: Vec<String>` instead of the spec's
  `from_type: AttendanceType` / `to_type: AttendanceType` pair.
  Same pattern as DOMAIN-ATT-002 and DOMAIN-ATT-011.

**Expected:**

`docs/specs/attendance/events.md:127-136` lists
  six payload fields (subject_attendance_id, student_id,
  subject_id, attendance_date, from_type, to_type, updated_by,
  updated_at).

**Evidence:**

`crates/domains/attendance/src/events.rs:455-461`
  has `changes: Vec<String>` only. Missing
  `student_id`, `subject_id`, `attendance_date`, `from_type`,
  `to_type`, `updated_by`, `updated_at`.

---

### FINDING 13 (id: `DOMAIN-ATT-013`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/events.rs:843-850` vs `docs/specs/attendance/events.md:283` (implied by aggregates.md)

**Description:**

`ExamAttendanceUpdated` carries
  `changes: Vec<String>` instead of the spec's `from_type` /
  `to_type` pair. Same anti-pattern as
  DOMAIN-ATT-002/011/012; consistent bug across all
  `*AttendanceUpdated` events.

**Expected:**

`docs/specs/assessment/events.md` (the cross-
  referenced source-of-truth for the `ExamAttendanceUpdated`
  event) lists `from_type`/`to_type` fields. The pattern is
  consistent across `docs/specs/attendance/events.md:58-67,
  127-136, 168-176`.

**Evidence:**

`crates/domains/attendance/src/events.rs:844-850`
  has `changes: Vec<String>` only. Missing `from_type`,
  `to_type`, `updated_by`, `updated_at` (no spec file is
  shipped in `docs/specs/assessment/events.md` for
  `ExamAttendanceUpdated` either — the attendance crate is the
  authoritative source per Phase 5 handoff OQ #1).

---

### FINDING 14 (id: `DOMAIN-ATT-014`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/events.rs:1308-1320` vs `docs/specs/attendance/events.md:259-271`

**Description:**

`ClassAttendanceRecomputed` carries a
  different shape from the spec: the spec lists
  `student_id`, `exam_type_id`, `academic_year_id`,
  `days_opened`, `days_present`, `days_absent`, `days_late`,
  `days_half_day`, `days_on_leave`, while the code carries
  `class_id`, `section_id`, `attendance_date`, `total_students`,
  `absent_count`, `present_count`. The code event is a per-class-
  per-day roll-up; the spec event is a per-student-per-exam-
  per-year roll-up with typed `Days*` wrappers. This is a
  fundamental wire-shape divergence.

**Expected:**

`docs/specs/attendance/events.md:259-271`
  ```
  pub struct ClassAttendanceRecomputed {
      pub class_attendance_id: ClassAttendanceId,
      pub student_id: StudentId,
      pub exam_type_id: ExamTypeId,
      pub academic_year_id: AcademicYearId,
      pub days_opened: DaysOpened,
      pub days_present: DaysPresent,
      pub days_absent: DaysAbsent,
      pub days_late: DaysLate,
      pub days_half_day: DaysHalfDay,
      pub days_on_leave: DaysOnLeave,
      pub recomputed_at: Timestamp,
  }
  ```

**Evidence:**

`crates/domains/attendance/src/events.rs:1309-1320`
  declares a different shape (class_id/section_id/total_students/
  absent_count/present_count). Missing the typed `Days*`
  wrappers — none of which are defined in
  `crates/domains/attendance/src/value_objects.rs`.

---

### FINDING 15 (id: `DOMAIN-ATT-015`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/events.rs:307-316` vs `docs/specs/attendance/events.md:100-107`

**Description:**

`StudentAttendanceImported` is missing the
  spec's `source: AttendanceSource` and `imported_at:
  Timestamp` fields. The code also adds `student_attendance_id`
  (which is a derived value, not in the spec) and renames the
  spec's `import_id` to `bulk_import_id`. The wire contract
  drifts in three directions: missing fields, extra fields, and
  renamed fields.

**Expected:**

`docs/specs/attendance/events.md:100-107`
  ```
  pub struct StudentAttendanceImported {
      pub import_id: BulkAttendanceImportId,
      pub student_id: StudentId,
      pub attendance_date: AttendanceDate,
      pub attendance_type: AttendanceType,
      pub source: AttendanceSource,
      pub imported_at: Timestamp,
  }
  ```

**Evidence:**

`crates/domains/attendance/src/events.rs:308-316`
  fields are `student_attendance_id`, `bulk_import_id`,
  `student_id`, `attendance_date`, `attendance_type`, plus the
  metadata footer. Missing `source` and `imported_at`; the
  `import_id` field is renamed to `bulk_import_id`; the
  `student_attendance_id` field is added but not in the spec.

---

### FINDING 18 (id: `DOMAIN-ATT-018`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/aggregate.rs` and `docs/specs/attendance/aggregates.md:210-232`

**Description:**

The spec defines a `ClassAttendance` aggregate
  (`docs/specs/attendance/aggregates.md:210-232` and
  `entities.md:35-48`) with the invariant
  `"days_opened = days_present + days_absent + days_on_leave + days_half_day * 0.5"`.
  The attendance crate ships a `ClassAttendanceId` typed
  identifier but no `ClassAttendance` aggregate struct and no
  invariant enforcement. The integration test
  `attendance_class_attendances_aggregate` row stays
  `Pending` in `coverage.toml:694-701`.

**Expected:**

`docs/specs/attendance/aggregates.md:210-232`
  declares `ClassAttendance` as a per-(student, exam_type,
  academic_year) summary aggregate.

**Evidence:**

- `grep -n "pub struct ClassAttendance" crates/domains/attendance/src/aggregate.rs`
    returns no result.
  - `grep -rn "ClassAttendanceRepository\|recompute_class_attendance" crates/domains/attendance/src/`
    returns no service or repository.
  - `crates/domains/attendance/src/value_objects.rs:155-160` —
    only the `ClassAttendanceId` typed id is defined; no
    aggregate.

---

### FINDING 19 (id: `DOMAIN-ATT-019`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/aggregate.rs` and `docs/specs/attendance/aggregates.md:235-248`

**Description:**

The spec defines an `AttendanceBulk`
  aggregate (`docs/specs/attendance/aggregates.md:235-248` and
  `entities.md:56-65`) as a per-(student, date) denormalized
  staging row with the invariant "Belongs to exactly one
  `BulkAttendanceImport`". The crate ships an `AttendanceBulkId`
  typed identifier (as a Phase 5 placeholder per the handoff)
  but no `AttendanceBulk` aggregate struct. The integration
  test path `attendance_integration.rs` does not exercise the
  attendance_bulks table.

**Expected:**

`docs/specs/attendance/aggregates.md:235-248`
  declares the `AttendanceBulk` aggregate.

**Evidence:**

- `grep -n "pub struct AttendanceBulk" crates/domains/attendance/src/aggregate.rs`
    returns no result.
  - `crates/domains/attendance/src/value_objects.rs:166-173`
    declares `AttendanceBulkId` with the comment
    `"Placeholder until the bulk-import header aggregate lands
    in a follow-up phase"` (Phase 5 handoff's Phase 5 stub
    contract).

---

### FINDING 2 (id: `DOMAIN-ATT-002`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/events.rs:131-138` vs `docs/specs/attendance/events.md:58-67`

**Description:**

`StudentAttendanceUpdated` carries
  `changes: Vec<String>` (a list of field names that changed)
  instead of the spec's `from_type: AttendanceType` /
  `to_type: AttendanceType` pair. Without the typed from/to pair,
  downstream consumers (notification fan-out, finance fine
  accrual, academic absence counters) cannot tell whether the
  student was transitioned *into* or *out of* absence — they
  only know the field names that changed.

**Expected:**

`docs/specs/attendance/events.md:58-67`
  ```
  pub struct StudentAttendanceUpdated {
      pub student_attendance_id: StudentAttendanceId,
      pub student_id: StudentId,
      pub attendance_date: AttendanceDate,
      pub from_type: AttendanceType,
      pub to_type: AttendanceType,
      pub notes: Option<String>,
      pub updated_by: UserId,
      pub updated_at: Timestamp,
  }
  ```

**Evidence:**

`crates/domains/attendance/src/events.rs:131-138` —
  ```rust
  pub struct StudentAttendanceUpdated {
      pub student_attendance_id: StudentAttendanceId,
      pub changes: Vec<String>,
      pub event_id: EventId,
      pub correlation_id: CorrelationId,
      pub occurred_at: Timestamp,
  }
  ```
  Missing fields: `student_id`, `attendance_date`, `from_type`,
  `to_type`, `notes`, `updated_by`, `updated_at`.

---

### FINDING 20 (id: `DOMAIN-ATT-020`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/services.rs` (entire file) and `docs/specs/attendance/services.md:62-83`

**Description:**

`AbsenceDetectionService` is missing from
  `services.rs`. The spec (`services.md:62-83`) mandates three
  methods: `detect(school, date, section) -> Vec<StudentAbsentForDay>`,
  `should_notify(school, attendance) -> bool`,
  `dedup_within_day(events) -> Vec<StudentAbsentForDay>`. The
  third method is implemented as `AttendanceService::dedup_within_day`
  but in the wrong struct; the first two are absent.

**Expected:**

`docs/specs/attendance/services.md:62-83`
  ```
  pub struct AbsenceDetectionService;
  impl AbsenceDetectionService {
      pub fn detect(...) -> Vec<StudentAbsentForDay>;
      pub fn should_notify(...) -> bool;
      pub fn dedup_within_day(...) -> Vec<StudentAbsentForDay>;
  }
  ```

**Evidence:**

- `grep -n "AbsenceDetectionService" crates/domains/attendance/src/services.rs`
    returns no result.
  - `crates/domains/attendance/src/services.rs:1273-1284`
    implements `dedup_within_day` inside the `AttendanceService`
    struct (wrong struct per the spec).

---

### FINDING 21 (id: `DOMAIN-ATT-021`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/services.rs` and `docs/specs/attendance/services.md:89-131`

**Description:**

`AttendanceReportService` is missing
  entirely. The spec mandates six methods:
  `daily(school, date)`, `weekly(school, section, from, to)`,
  `monthly(school, section, month)`, `by_class(school, from, to)`,
  `by_student(school, student, from, to)`,
  `by_staff(school, from, to)`. The report capability
  `AttendanceReportRead` (rbac) is defined and the spec
  workflow `docs/specs/attendance/workflows.md:137-153` mandates
  the report pipeline.

**Expected:**

`docs/specs/attendance/services.md:89-131`.

**Evidence:**

`grep -n "AttendanceReportService" crates/domains/attendance/src/`
  returns no result.

---

### FINDING 22 (id: `DOMAIN-ATT-022`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/services.rs` and `docs/specs/attendance/services.md:139-165`

**Description:**

`AttendanceImportService` is missing as a
  service struct. The spec mandates four methods:
  `stage(source, rows)`, `validate(import, school)`,
  `commit(import, school, actor)`, `cancel(import, reason)`.
  The free-function services in
  `crates/domains/attendance/src/services.rs:855-1181`
  (`import_attendance`, `validate_bulk_import`,
  `commit_bulk_import`, `cancel_bulk_import`) implement the
  state-machine but live in the file's free-function namespace,
  not in a typed `AttendanceImportService` struct as the spec
  requires.

**Expected:**

`docs/specs/attendance/services.md:139-165`.

**Evidence:**

`grep -n "pub struct AttendanceImportService" crates/domains/attendance/src/`
  returns no result.

---

### FINDING 25 (id: `DOMAIN-ATT-025`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/repository.rs` and `docs/specs/attendance/repositories.md:149-180`

**Description:**

`ClassAttendanceRepository` is missing. The
  spec mandates four methods: `get(school, student, exam_type,
  year)`, `list_for_student(school, student, year)`,
  `list_for_exam_type(school, exam_type, year)`,
  `upsert(c)`. Without this port, the storage adapter cannot
  persist the `ClassAttendance` projection.

**Expected:**

`docs/specs/attendance/repositories.md:149-180`.

**Evidence:**

`grep -n "ClassAttendanceRepository" crates/domains/attendance/src/repository.rs`
  returns no result; the 5 traits shipped are
  `StudentAttendanceRepository`, `SubjectAttendanceRepository`,
  `StaffAttendanceRepository`, `ExamAttendanceRepository`,
  `AttendanceImportRepository`.

---

### FINDING 26 (id: `DOMAIN-ATT-026`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/services.rs:1225-1285`

**Description:**

`AttendanceService` is missing four of the
  seven methods the spec mandates. The spec
  (`services.md:6-54`) requires: `mark`, `update`,
  `is_late(date, arrival, threshold) -> bool`,
  `is_half_day(school, attendance) -> bool`,
  `is_holiday(school, date) -> bool`,
  `emit_absence_event(attendance, school_id) -> Option<StudentAbsentForDay>`,
  `recompute_class_attendance(student, academic_year, events) -> ClassAttendance`.
  The code ships only `is_late` (as a stub returning `false`),
  `emit_absence_event` (different signature: takes `&StudentAttendance`
  not `(&StudentAttendance, SchoolId)`), and
  `dedup_within_day` (which is part of `AbsenceDetectionService`
  per the spec, not `AttendanceService`). The `mark`, `update`,
  `is_half_day`, `is_holiday`, and `recompute_class_attendance`
  methods are absent.

**Expected:**

`docs/specs/attendance/services.md:6-54`.

**Evidence:**

- `grep -nE "fn mark|fn update|fn is_half_day|fn is_holiday|fn recompute_class_attendance" crates/domains/attendance/src/services.rs`
    returns no result (the free-function `mark_*_attendance`
    services exist but live outside the `AttendanceService`
    struct).
  - `crates/domains/attendance/src/services.rs:1233-1243` —
    `is_late` is a stub returning `false`.

---

### FINDING 27 (id: `DOMAIN-ATT-027`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/services.rs:1200-1216`

**Description:**

The `request_absence_notification` service
  uses placeholder values for production-critical fields:
  `placeholder_uuid = uuid::Uuid::now_v7()` for `student_id`
  and `chrono::NaiveDate::from_ymd_opt(1970, 1, 1).expect("epoch")`
  for `attendance_date`. Both values reach the event payload and
  the bus. The `.expect("epoch")` is a production-code
  panic-prone call (the production date arithmetic is infallible
  for `1970-01-01`, but the call violates the engine rule
  against `expect()` in production code).

**Expected:**

Per the engine rule in AGENTS.md ("No
  `unwrap()` or `expect()` in production paths"), no
  `expect()` is allowed.

**Evidence:**

`crates/domains/attendance/src/services.rs:1200-1209`:
  ```rust
  let placeholder_uuid = uuid::Uuid::now_v7();
  ...
  crate::value_objects::StudentId::new(cmd.tenant.school_id, placeholder_uuid),
  chrono::NaiveDate::from_ymd_opt(1970, 1, 1).expect("epoch"),
  ```

---

### FINDING 29 (id: `DOMAIN-ATT-029`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/services.rs` and `docs/specs/attendance/services.md:51`

**Description:**

`recompute_class_attendance` references a
  `StudentAttendanceEvent` slice type that does not exist in
  the crate. The spec signature is
  `recompute_class_attendance(student, academic_year, events: &[StudentAttendanceEvent]) -> ClassAttendance`.
  No enum or trait alias named `StudentAttendanceEvent` is
  defined in `events.rs` or `value_objects.rs`.

**Expected:**

`docs/specs/attendance/services.md:48-52`.

**Evidence:**

- `grep -rn "StudentAttendanceEvent" crates/domains/attendance/src/`
    returns no struct/enum/trait definition.

---

### FINDING 3 (id: `DOMAIN-ATT-003`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/events.rs:186-192` vs `docs/specs/attendance/events.md:69-77`

**Description:**

`StudentAttendanceRestored` is missing all six
  payload fields the spec mandates (`student_id`,
  `attendance_date`, `from_type`, `to_type`, `restored_by`,
  `restored_at`). Without the typed from/to fields, the
  notification fan-out cannot decide whether to send an "all
  clear" notice (the spec calls this out as an edge case at
  `workflows.md:67-71`).

**Expected:**

`docs/specs/attendance/events.md:69-77` lists six
  payload fields beyond `student_attendance_id`.

**Evidence:**

`crates/domains/attendance/src/events.rs:187-192` —
  ```rust
  pub struct StudentAttendanceRestored {
      pub student_attendance_id: StudentAttendanceId,
      pub event_id: EventId,
      pub correlation_id: CorrelationId,
      pub occurred_at: Timestamp,
  }
  ```
  No payload fields beyond the metadata footer.

---

### FINDING 30 (id: `DOMAIN-ATT-030`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/services.rs` and `crates/domains/attendance/src/events.rs:179-228`

**Description:**

`StudentAttendanceRestored` is a fully
  defined event struct but is never emitted by any service.
  The spec `events.md:69-77` and `workflows.md:68-71` mandate
  that when a teacher re-marks an absent student Present, the
  engine emits `StudentAttendanceRestored`. The
  `update_student_attendance` service
  (`services.rs:182-230`) only emits `StudentAttendanceUpdated`
  (and even that with a non-spec `changes: Vec<String>` shape;
  see DOMAIN-ATT-002). The "restore" transition is invisible to
  downstream subscribers.

**Expected:**

`docs/specs/attendance/workflows.md:65-71` —
  spec step 5 mandates the restored event on transition out of
  absence.

**Evidence:**

- `grep -rn "StudentAttendanceRestored::new" crates/domains/attendance/src/`
    returns 0 production call sites (1 in the unit test on
    events.rs:1471-1475).
  - `crates/domains/attendance/src/services.rs:182-230`
    emits only `StudentAttendanceUpdated`, never
    `StudentAttendanceRestored`.

---

### FINDING 31 (id: `DOMAIN-ATT-031`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/value_objects.rs`

**Description:**

Most value objects mandated by
  `docs/specs/attendance/value-objects.md` are missing. The
  spec lists (beyond the typed ids and the four closed enums
  already shipped): `InTime`, `OutTime`, `Period`, `TimeWindow`,
  `LateThreshold`, `DayOfWeek`, `DaysOpened`, `DaysPresent`,
  `DaysAbsent`, `DaysLate`, `DaysHalfDay`, `DaysOnLeave`,
  `Notify`, `IsAbsent`, `IsHoliday`, `MarkedBy`, `MarkedAt`,
  `MarkedFrom`, `AttendanceRange`, `AttendanceReportKind`,
  `AttendancePercentage`, `YearMonth`, and the `Validate`
  trait. None of these are declared in `value_objects.rs`. The
  spec also mandates a `NotificationChannel` and
  `NotificationTemplate` enum (already covered in
  DOMAIN-ATT-017).

**Expected:**

`docs/specs/attendance/value-objects.md:6-90`
  lists 25+ value objects and the `Validate` trait.

**Evidence:**

`grep -nE "pub struct (InTime|OutTime|Period|TimeWindow|LateThreshold|DayOfWeek|DaysOpened|DaysPresent|DaysAbsent|DaysLate|DaysHalfDay|DaysOnLeave|AttendanceRange|AttendanceReportKind|AttendancePercentage|YearMonth)" crates/domains/attendance/src/value_objects.rs`
  returns no result. `grep -n "trait Validate" crates/domains/attendance/src/`
  also returns no result.

---

### FINDING 4 (id: `DOMAIN-ATT-004`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/events.rs:237-249` vs `docs/specs/attendance/events.md:79-87`

**Description:**

`StudentAbsentForDay` is missing the spec's
  `notify: bool` field that mirrors the
  `MarkStudentAttendance.notify` flag. The whole point of the
  event is to drive the guardian notification fan-out (per
  `events.md:91-97`); without `notify`, the communication
  subscriber cannot tell whether to send an SMS/email/push or
  skip the dispatch.

**Expected:**

`docs/specs/attendance/events.md:79-87` — the
  spec lists `notify: bool` with the comment
  `"// mirrors the MarkStudentAttendance.notify flag"`.

**Evidence:**

`crates/domains/attendance/src/events.rs:238-249`
  declares no `notify` field. Compare with the
  `SubjectAbsentNotificationRequested` event at lines 509-519,
  which does carry `notify` via the
  `MarkSubjectAttendanceCommand.notify` flag.

---

### FINDING 5 (id: `DOMAIN-ATT-005`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/events.rs:1127-1134` vs `docs/specs/attendance/events.md:224-229`

**Description:**

`BulkImportCancelled` is missing
  `cancelled_by: UserId` and `cancelled_at: Timestamp`. The audit
  log cannot attribute the cancellation to a specific user or
  point in time.

**Expected:**

`docs/specs/attendance/events.md:224-229`
  ```
  pub struct BulkImportCancelled {
      pub bulk_import_id: BulkAttendanceImportId,
      pub cancelled_by: UserId,
      pub reason: String,
      pub cancelled_at: Timestamp,
  }
  ```

**Evidence:**

`crates/domains/attendance/src/events.rs:1128-1134`
  carries only `bulk_import_id` and `reason` — `cancelled_by`
  and `cancelled_at` are absent.

---

### FINDING 6 (id: `DOMAIN-ATT-006`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/events.rs:897-907` vs `docs/specs/attendance/events.md:194-200`

**Description:**

`BulkImportStarted` uses `marked_by: UserId`
  (the field name is borrowed from the `Mark*` family) instead
  of the spec's `started_by: UserId`, and uses `occurred_at:
  Timestamp` instead of the spec's `started_at: Timestamp`. The
  field-name mismatch is a wire-contract drift.

**Expected:**

`docs/specs/attendance/events.md:194-200` — field
  name `started_by` (not `marked_by`).

**Evidence:**

`crates/domains/attendance/src/events.rs:898-907`:
  `pub marked_by: educore_core::ids::UserId,` vs spec
  `pub started_by: UserId,`. Code also lacks `started_at` and
  only carries `occurred_at`.

---

### FINDING 7 (id: `DOMAIN-ATT-007`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/events.rs:961-968` vs `docs/specs/attendance/events.md:202-207`

**Description:**

`BulkImportValidated` is missing
  `validated_by: UserId` and `validated_at: Timestamp`. The
  spec also carries `row_count: u32` only, but the code adds
  `absent_count: u32` (which the spec reserves for
  `BulkImportCommitted`). The field swap is a wire-contract
  drift.

**Expected:**

`docs/specs/attendance/events.md:202-207`
  ```
  pub struct BulkImportValidated {
      pub bulk_import_id: BulkAttendanceImportId,
      pub validated_by: UserId,
      pub validated_at: Timestamp,
      pub row_count: u32,
  }
  ```

**Evidence:**

`crates/domains/attendance/src/events.rs:962-968`
  — fields are `bulk_import_id`, `row_count`, `absent_count`,
  `event_id`, `correlation_id`, `occurred_at`. Missing
  `validated_by` and `validated_at`; the `absent_count` field
  belongs on `BulkImportCommitted` per the spec.

---

### FINDING 8 (id: `DOMAIN-ATT-008`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/events.rs:1018-1024` vs `docs/specs/attendance/events.md:209-215`

**Description:**

`BulkImportCommitted` is missing
  `committed_by: UserId` and `committed_at: Timestamp` and the
  spec's `row_count` field; the code renames `row_count` to
  `committed_count: u32` and drops the spec's `absent_count:
  u32` (which `BulkImportValidated` picked up incorrectly — see
  DOMAIN-ATT-007).

**Expected:**

`docs/specs/attendance/events.md:209-215`
  ```
  pub struct BulkImportCommitted {
      pub bulk_import_id: BulkAttendanceImportId,
      pub committed_by: UserId,
      pub committed_at: Timestamp,
      pub row_count: u32,
      pub absent_count: u32,
  }
  ```

**Evidence:**

`crates/domains/attendance/src/events.rs:1019-1024`
  — fields are `bulk_import_id`, `committed_count`, `event_id`,
  `correlation_id`, `occurred_at`. Missing `committed_by`,
  `committed_at`, `absent_count`.

---

### FINDING 9 (id: `DOMAIN-ATT-009`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/events.rs:1072-1079` and `crates/domains/attendance/src/value_objects.rs`

**Description:**

`BulkImportFailed` is missing the spec's
  `row_errors: Vec<RowError>` field. The `RowError` value object
  does not exist in `value_objects.rs` despite the spec
  (`events.md:232-233`) requiring it as
  `RowError { row_index, student_id, reason }`. Bulk-import
  diagnostics cannot surface per-row failure reasons.

**Expected:**

`docs/specs/attendance/events.md:217-222` lists
  `pub row_errors: Vec<RowError>,` and `events.md:232-233`
  documents `RowError { RowIndex, StudentId, Reason }`.

**Evidence:**

- `crates/domains/attendance/src/events.rs:1073-1079`
    fields are `bulk_import_id`, `failed_count`, `reason`,
    `event_id`, `correlation_id`, `occurred_at`. Missing
    `row_errors` (and the underlying `RowError` type itself).
  - `grep -n "RowError" crates/domains/attendance/src/value_objects.rs`
    returns no struct/enum definitions.

---

### FINDING 16 (id: `DOMAIN-ATT-016`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/events.rs` (entire file)

**Description:**

Every event payload uses raw `NaiveDate`
  instead of the spec's typed `AttendanceDate` wrapper. The
  spec at `value-objects.md:40` mandates `AttendanceDate:
  NaiveDate` as a typed value object; the value_objects module
  does not declare one, so every event field falls back to the
  underlying `chrono::NaiveDate`. This violates the engine
  rule "Compile-time safety over strings" (AGENTS.md) and
  makes it impossible to add cross-cutting invariants on
  attendance dates (e.g. "not in the future").

**Expected:**

`docs/specs/attendance/value-objects.md:40` —
  `| AttendanceDate | NaiveDate |` is listed as a typed value
  object.

**Evidence:**

`grep -n "pub struct AttendanceDate" crates/domains/attendance/src/value_objects.rs`
  returns no result. Compare with
  `crates/domains/attendance/src/value_objects.rs:107-120`
  declaring `StudentAttendanceId` etc. as typed wrappers but no
  `AttendanceDate`. The events at events.rs:58, 119, 178, 244,
  311, 376, 512, 577, 702, 766, 1248, 1313 all use
  `pub attendance_date: NaiveDate,`.

---

### FINDING 17 (id: `DOMAIN-ATT-017`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/commands.rs:414-419`

**Description:**

`RequestAbsenceNotificationCommand.channel`
  and `.template` are raw `String` fields. The spec mandates
  typed `NotificationChannel` (SMS, Email, Push) and
  `NotificationTemplate` enums (`commands.md:248-249`); neither
  is defined in `value_objects.rs` or any other crate. The
  same wire-form divergence applies to the
  `SubjectAbsentNotificationRequested` event at events.rs:514-515
  and the `AbsenceNotificationRequested` event at
  events.rs:1249-1250.

**Expected:**

`docs/specs/attendance/commands.md:248-249`
  ```
  pub channel: NotificationChannel, // SMS, Email, Push
  pub template: NotificationTemplate, // e.g. "absence-daily"
  ```

**Evidence:**

- `crates/domains/attendance/src/commands.rs:417-418`:
    `pub channel: String,` and `pub template: String,`.
  - `grep -rn "pub enum NotificationChannel\|pub enum NotificationTemplate" crates/`
    returns no results.
  - `crates/domains/attendance/src/events.rs:514-515, 1249-1250`
    also use raw `String`.

---

### FINDING 23 (id: `DOMAIN-ATT-023`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/services.rs` (entire file) and `docs/specs/attendance/services.md:174-200`

**Description:**

The spec mandates three policy structs:
  `AttendanceEligibility` (for `MarkStudentAttendanceCommand`),
  `BulkMarkEligibility` (for `BulkMarkStudentAttendanceCommand`),
  `NotificationEligibility` (for `StudentAbsentForDay`). None
  are declared in the crate.

**Expected:**

`docs/specs/attendance/services.md:174-200`.

**Evidence:**

`grep -rn "AttendanceEligibility\|BulkMarkEligibility\|NotificationEligibility" crates/domains/attendance/src/`
  returns no result.

---

### FINDING 24 (id: `DOMAIN-ATT-024`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/services.rs` and `docs/specs/attendance/services.md:201-235`

**Description:**

The spec mandates three specification
  structs: `ActiveOnDate`, `HasOutstandingAbsence`,
  `EligibleForExamAttendance`. None are declared in the crate.
  These are used by the `StudentAttendanceQuery`,
  `ClassAttendanceRecomputed`, and exam-day workflows
  respectively.

**Expected:**

`docs/specs/attendance/services.md:201-235`.

**Evidence:**

`grep -rn "ActiveOnDate\|HasOutstandingAbsence\|EligibleForExamAttendance" crates/domains/attendance/src/`
  returns no result.

---

### FINDING 28 (id: `DOMAIN-ATT-028`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/services.rs:1233-1243`

**Description:**

`AttendanceService::is_late` is a stub that
  unconditionally returns `false`. The spec calls this method
  out specifically: `"is_late checks the school-defined late
  threshold"`. A production caller invoking this method will
  silently treat all students as on-time. The Phase 5 handoff
  documents this as a stub (line 1238: `"// Phase 5 stub. The
  full implementation considers"`) but the stub body returns
  `false` rather than `unimplemented!()` or a `Result::Err`,
  making the failure mode invisible.

**Expected:**

`docs/specs/attendance/services.md:27-31`
  mandates an actual implementation that compares the arrival
  time against the threshold.

**Evidence:**

`crates/domains/attendance/src/services.rs:1233-1243`:
  ```rust
  pub const fn is_late(
      _date: chrono::NaiveDate,
      _arrival: chrono::NaiveTime,
      _threshold: chrono::NaiveTime,
  ) -> bool {
      // Phase 5 stub. The full implementation considers
      // the school's `late_threshold_minutes` setting and
      // the day-of-week calendar. The integration test
      // (Workstream D) exercises the production path.
      false
  }
  ```

---

### FINDING 32 (id: `DOMAIN-ATT-032`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/entities.rs` and `docs/specs/attendance/tables.md`

**Description:**

`tables.md` lists 9 tables (5 owned by the
  attendance domain plus the 2 exam-attendance tables
  delegated to assessment, plus the
  `student_attendance_bulks` and `class_attendances` tables).
  The crate's `entities.rs` ships only 2 child entity structs
  (`StudentAttendanceImport`, `StaffAttendanceImport`); no
  `#[derive(DomainQuery)]` macro is invoked anywhere in the
  crate (the macro emission is documented in AGENTS.md as the
  path that emits the typed AST for the storage adapter to
  translate).

**Expected:**

`docs/specs/attendance/tables.md` lists 9
  tables; each should map to a `#[derive(DomainQuery)]` struct
  per the engine's "macro-emitted typed AST" rule (AGENTS.md
  § "Code Standards").

**Evidence:**

- `grep -rn "DomainQuery" crates/domains/attendance/src/`
    returns no result.
  - `crates/domains/attendance/src/entities.rs` (143 lines
    total) declares only `StudentAttendanceImport` (line 40)
    and `StaffAttendanceImport` (line 96).

---

### FINDING 33 (id: `DOMAIN-ATT-033`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/tests/` (absent) and `docs/handoff/PHASE-5-HANDOFF.md:140-145`

**Description:**

The crate has no `tests/` directory; all
  integration testing lives in
  `crates/tools/storage-parity/tests/attendance_integration.rs`.
  AGENTS.md's Validation Checklist requires "At least one
  integration test added for new behavior" per PR, and the
  standard per-domain layout calls for a `tests/` subdirectory
  alongside `src/`. The Phase 5 handoff's "93 unit tests pass
  in `educore-attendance`" + 4 storage-parity integration tests
  covers the cross-cutting path but does not include per-
  aggregate behavioural tests that don't require the storage
  adapter (e.g. the `mark_staff_attendance` rejection paths
  for `is_absent` vs `is_holiday`, the `AttendanceService::dedup_within_day`
  edge cases beyond the single `tests.rs` test, the
  `commit_bulk_import` "import not in Validated state" path,
  etc.).

**Expected:**

A `crates/domains/attendance/tests/` directory
  with per-aggregate integration tests per the module layout
  in AGENTS.md.

**Evidence:**

`ls crates/domains/attendance/tests/` returns
  `No such file or directory`.

---

### FINDING 34 (id: `DOMAIN-ATT-034`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/tools/storage-parity/tests/attendance_integration.rs` and `crates/domains/attendance/src/services.rs`

**Description:**

Only 1 of the 14 service functions is
  exercised by the storage-parity integration test
  (`bulk_mark_student_attendance`). The other 13 services
  (`mark_student_attendance`, `update_student_attendance`,
  `mark_subject_attendance`, `update_subject_attendance`,
  `mark_staff_attendance`, `update_staff_attendance`,
  `mark_exam_attendance`, `update_exam_attendance`,
  `import_attendance`, `validate_bulk_import`,
  `commit_bulk_import`, `cancel_bulk_import`,
  `request_absence_notification`) have no storage-adapter
  integration coverage.

**Expected:**

Per AGENTS.md "Validation Checklist" — "at
  least one integration test added for new behavior".

**Evidence:**

`grep -rn "mark_student_attendance\|mark_subject_attendance\|mark_staff_attendance\|mark_exam_attendance\|update_.*_attendance\|import_attendance\|validate_bulk_import\|commit_bulk_import\|cancel_bulk_import\|request_absence_notification" crates/tools/storage-parity/tests/attendance_integration.rs`
  returns only the `bulk_mark_student_attendance` references
  (the helper `make_bulk_cmd` and `dispatch_bulk_mark`).

---

### FINDING 36 (id: `DOMAIN-ATT-036`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/entities.rs:40-62`

**Description:**

`StudentAttendanceImport` does not carry a
  `student_record_id` field despite the spec entities.md
  implicitly requiring it (the
  `MarkStudentAttendanceCommand` carries `student_record_id`
  and the staging row promotes to a `StudentAttendance`
  aggregate, which carries `student_record_id`). The bulk-
  commit code (`services.rs:1082-1090`) acknowledges this as a
  Phase 5 stub by synthesising a placeholder
  `StudentRecordId` from the event id, but the entity struct
  itself is missing the field.

**Expected:**

Per `docs/specs/attendance/entities.md:6-20`,
  the staging row "carries `StudentId`, `AttendanceDate`,
  `InTime`, `OutTime`, `AttendanceType`, `Notes`" — the
  spec is silent on `student_record_id`, but the promotion
  flow requires it (the live `StudentAttendance` aggregate's
  `student_record_id` cannot be synthesised from the event id
  in production).

**Evidence:**

`crates/domains/attendance/src/entities.rs:40-62`
  — fields are `id`, `bulk_import_id`, `student_id`,
  `attendance_date`, `attendance_type`, `in_time`, `out_time`,
  `notes`, `is_validated`, `active_status`. No
  `student_record_id`.

---

### FINDING 39 (id: `DOMAIN-ATT-039`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `docs/specs/attendance/events.md:7-16` vs `crates/cross-cutting/events/src/domain_event.rs:55`

**Description:**

The spec at `events.md:11` declares the
  `DomainEvent` trait with `const TYPE: &'static str;`. The
  actual trait at `crates/cross-cutting/events/src/domain_event.rs:55`
  uses `const EVENT_TYPE: &'static str;`. Every impl in
  `events.rs:107-121, 159-175, 211-227, 280-296, ...` (21
  events) uses `EVENT_TYPE` / `AGGREGATE_TYPE`. The spec is
  out of date; the spec-vs-code drift lives at the spec layer.

**Expected:**

Spec mandates `const TYPE: &'static str;` but
  the canonical trait uses `EVENT_TYPE`. The spec should be
  updated to match the code (or vice versa); the current
  state has both layers disagreeing.

**Evidence:**

- `docs/specs/attendance/events.md:7-16` declares the
    `DomainEvent` trait with `const TYPE: &'static str;` and
    `fn aggregate_id(&self) -> Uuid;`.
  - `crates/cross-cutting/events/src/domain_event.rs:52-75`
    uses `const EVENT_TYPE: &'static str;` and
    `const SCHEMA_VERSION: u32;` and `const AGGREGATE_TYPE: &'static str;`.

---

### FINDING 40 (id: `DOMAIN-ATT-040`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/rbac/src/services.rs:345-650` and `docs/specs/attendance/permissions.md:60-77`

**Description:**

`DefaultRoleCatalog::school_admin()` (and
  the `super_admin()`, `class_teacher()`, `subject_teacher()`,
  `attendance_cell()`, `staff()`, `student()`, `parent()`,
  `hr()`, `accountant()`, `auditor()` methods) does not grant
  any of the 24 new `Attendance.*` capabilities. The Phase 5
  handoff acknowledges this as OQ #5 (lines 455-463) and
  defers it to consumer `seed.rs` initialisation, but the
  default role catalog's purpose is exactly this; deferring
  leaves the engine without a working default. Per
  `docs/specs/attendance/permissions.md:60-77`, `SchoolAdmin`
  should have `Attendance.*`, `AttendanceCell` should have
  `Attendance.* + Attendance.Import.* + Attendance.Report.*`,
  etc.

**Expected:**

`docs/specs/attendance/permissions.md:60-77`
  table lists 10 default roles with capability grants.

**Evidence:**

`grep -nE "AttendanceStudent|AttendanceSubject|AttendanceStaff|AttendanceImport|AttendanceExam|AttendanceBulkMark|AttendanceReport|AttendanceNotify" crates/cross-cutting/rbac/src/services.rs`
  returns only `HrAttendance*` capabilities (the HR-domain
  capabilities added in Phase 6), no `Attendance*` capabilities
  (the attendance-domain ones added in Phase 5).

---

### FINDING 41 (id: `DOMAIN-ATT-041`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/services.rs` (entire file)

**Description:**

The 14 service functions are not gated by
  capability checks. AGENTS.md § "Engine Rules" requires
  "Capabilities are checked at the command boundary. The
  engine never trusts the caller to assert their own role."
  Per `docs/specs/attendance/permissions.md:82-91` and the
  spec commands.md (e.g. `MarkStudentAttendance` capability
  `Attendance.Mark`, `BulkMarkStudentAttendance` capability
  `Attendance.BulkMark`, etc.), every service should check the
  capability before mutating. The Phase 5 handoff (lines
  363-391) documents this as a deliberate boundary (matching
  the academic/assessment pattern), but the absence of the
  check means the capability check is the dispatcher's job —
  and the dispatcher has not been built yet.

**Expected:**

`docs/specs/attendance/permissions.md:82-91`
  shows the canonical capability check pattern.

**Evidence:**

`grep -nE "capability_check|has\(|Capability::Attendance" crates/domains/attendance/src/services.rs`
  returns no result.

---

### FINDING 45 (id: `DOMAIN-ATT-045`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/services.rs:1083-1095`

**Description:**

`commit_bulk_import` synthesises placeholder
  typed ids for `student_record_id`, `class_id`, and
  `section_id` from the event id:
  `StudentRecordId::new(import.school_id, event_id_to_uuid(event_id))`,
  `ClassId::new(import.school_id, event_id_to_uuid(event_id))`,
  `SectionId::new(import.school_id, event_id_to_uuid(event_id))`.
  These placeholder ids propagate into the persisted
  `StudentAttendance` aggregate and the
  `StudentAttendanceImported` event payload. The comment
  acknowledges the placeholder but the production path
  cannot recover the original ids.

**Expected:**

The commit path should resolve the typed ids
  from the enrollment table; the Phase 5 stub is acceptable
  for Phase 5 but the placeholder UUID must not reach
  production.

**Evidence:**

`crates/domains/attendance/src/services.rs:1083-1095`:
  ```rust
  crate::value_objects::StudentRecordId::new(
      import.school_id,
      event_id_to_uuid(event_id),
  ),
  crate::value_objects::ClassId::new(import.school_id, event_id_to_uuid(event_id)),
  crate::value_objects::SectionId::new(import.school_id, event_id_to_uuid(event_id)),
  ```

---

### FINDING 46 (id: `DOMAIN-ATT-046`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/services.rs:286-302`

**Description:**

`bulk_mark_student_attendance` produces 1
  extra `StudentAttendance` aggregate for the "default type"
  cohort of unmarked students. This is a stub path: the spec
  (`commands.md:128-151`) requires
  `BulkMarkStudentAttendance` to "creates or replaces
  `StudentAttendance` rows for all students in the section".
  The service emits one aggregate per absent / late / half-day
  id plus one extra aggregate for the unmarked cohort
  (totaling `absent_ids.len() + late_ids.len() + half_day_ids.len() + 1`
  per `BulkMarkResult`). The unmarked aggregate has a
  placeholder `StudentId` (see DOMAIN-ATT-037). The
  integration test at
  `crates/tools/storage-parity/tests/attendance_integration.rs:330-338`
  documents this as a "Phase 5 stub" but the assertion is
  embedded in the test as if it were production behaviour.

**Expected:**

`docs/specs/attendance/commands.md:128-151`
  mandates per-student rows, not per-overridden-id + 1.

**Evidence:**

`crates/domains/attendance/src/services.rs:283-333`:
  the unmarked default-type aggregate is emitted before the
  per-id loops. The integration test
  `attendance_integration.rs:331-338` asserts
  `outcome.aggregates_len == 201` for 200 absent ids (200 +
  the default-type aggregate).

---

### FINDING 47 (id: `DOMAIN-ATT-047`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/services.rs:1187-1216` and `docs/specs/attendance/services.md:51`

**Description:**

`AttendanceService::emit_absence_event`
  (line 1249) takes a single `&StudentAttendance` parameter
  instead of the spec's two-parameter signature
  `emit_absence_event(attendance: &StudentAttendance, school_id: SchoolId) -> Option<StudentAbsentForDay>`
  (`services.md:43-46`). The spec signature separates the
  attendance row from the school id; the code merges them.
  This is a wire-contract drift between the service helper
  and the spec.

**Expected:**

`docs/specs/attendance/services.md:43-46`:
  `pub fn emit_absence_event(attendance: &StudentAttendance, school_id: SchoolId) -> Option<StudentAbsentForDay>;`

**Evidence:**

`crates/domains/attendance/src/services.rs:1249-1266`:
  `pub fn emit_absence_event(row: &StudentAttendance) -> Option<StudentAbsentForDay>`.

---

### FINDING 49 (id: `DOMAIN-ATT-049`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/repository.rs:54-127`

**Description:**

`StudentAttendanceRepository::find` does
  not match the spec's
  `find(school: SchoolId, student: StudentId, date: NaiveDate)`
  signature exactly. The code matches it on parameters, but
  the spec also mandates a `find` for
  `SubjectAttendanceRepository`
  (`repositories.md:53-63`:
  `find(school, student, subject, date)`), which is missing
  from the code's `SubjectAttendanceRepository` trait
  (`repository.rs:138-168`). The trait ships `get` and
  `list_for_student` and `list_for_section` but no
  `find(school, student, subject, date)`. The `mark_subject_attendance`
  service calls
  `uniqueness.subject_day_exists(school, student, subject, date)`
  which is the spec's behaviour, but the repository trait
  itself cannot serve a query for an individual row.

**Expected:**

`docs/specs/attendance/repositories.md:53-63`.

**Evidence:**

- `docs/specs/attendance/repositories.md:53-63` mandates
    `async fn find(school, student, subject, date)`.
  - `grep -n "fn find" crates/domains/attendance/src/repository.rs`
    only finds `find` on `StudentAttendanceRepository`
    (line 66) and `StaffAttendanceRepository` (line 186), not
    on `SubjectAttendanceRepository`.

---

### FINDING 35 (id: `DOMAIN-ATT-035`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/aggregate.rs` (all 5 aggregates)

**Description:**

None of the 5 aggregates has a mutator
  method (`update`, `soft_delete`, `restore`, `transition_to_*`).
  Only the `fresh()` constructor and predicate getters
  (`is_active`, `is_absent`, `is_terminal`) exist. The
  `update_*_attendance` services mutate the aggregates through
  `&mut StudentAttendance` parameters but the mutation logic
  lives entirely in the services, not on the aggregate itself.
  This is a separation-of-concerns gap: the aggregate is a
  passive data record.

**Expected:**

Per the engine rule "Domain scopes via
  extension traits" (AGENTS.md) and the academic crate's
  pattern (see `crates/domains/academic/src/aggregate.rs` for
  `Student::promote`, `Student::transfer`), aggregates should
  expose mutators.

**Evidence:**

`grep -nE "fn update|fn soft_delete|fn restore|fn promote|fn transfer" crates/domains/attendance/src/aggregate.rs`
  returns no result.

---

### FINDING 37 (id: `DOMAIN-ATT-037`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/services.rs:276-302`

**Description:**

`bulk_mark_student_attendance` synthesises a
  placeholder `StudentId` for the "default type" aggregate
  (the unmarked-students stub):
  `StudentId::new(cmd.tenant.school_id, event_id_to_uuid(event_id))`
  — the same UUID used for the aggregate id. This
  placeholder student id propagates into the persisted
  `StudentAttendance` aggregate and the
  `StudentAttendanceMarked` event payload. Production callers
  cannot distinguish the placeholder from a real student id.

**Expected:**

The bulk-mark service should consume the
  section roster (per `services.md:8-53`'s example signature
  taking `Student` and `StudentRecord` parameters), not
  synthesise a fake student.

**Evidence:**

`crates/domains/attendance/src/services.rs:286-288`:
  ```rust
  // The "default" student — Phase 5 stub. Replaced with
  // a real roster pull in the dispatcher.
  StudentId::new(cmd.tenant.school_id, event_id_to_uuid(event_id)),
  ```

---

### FINDING 38 (id: `DOMAIN-ATT-038`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/aggregate.rs:1062-1119`

**Description:**

`BulkAttendanceImport` aggregate carries a
  `notes: Option<String>` field but the spec's
  `BulkAttendanceImport` (`aggregates.md:129-175`) does not
  mandate it. The `BulkImportStarted` event payload also
  doesn't include notes. There is no path from the import
  command's notes through to the event or aggregate on
  commit. The field appears to be a Phase 5 addition without a
  spec basis.

**Expected:**

Per `docs/specs/attendance/aggregates.md:129-175`
  the `BulkAttendanceImport` has no `notes` field.

**Evidence:**

`crates/domains/attendance/src/aggregate.rs:556`
  declares `pub notes: Option<String>,` on `BulkAttendanceImport`.
  Compare with `crates/domains/attendance/src/aggregate.rs:103-108`
  (other aggregates) which carry the standard 10-field audit
  footer — `notes` is not in the standard footer.

---

### FINDING 42 (id: `DOMAIN-ATT-042`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/aggregate.rs:60-109`

**Description:**

`StudentAttendance` carries both a
  `school_id: SchoolId` field (line 66) and the typed id's
  embedded `school_id` (`id.school_id()`). The two fields can
  drift out of sync. The same redundancy exists on
  `StaffAttendance` (line 197), `SubjectAttendance` (line 300),
  `ExamAttendance` (line 419), and `BulkAttendanceImport`
  (line 539). The engine pattern is to derive `school_id` from
  the typed id; carrying a duplicate field opens the door to
  invariant violations.

**Expected:**

A single source of truth for the school anchor
  — the typed id.

**Evidence:**

`crates/domains/attendance/src/aggregate.rs:66`:
  `pub school_id: SchoolId,` and the `fresh()` constructor at
  line 136: `school_id: id.school_id(),`.

---

### FINDING 43 (id: `DOMAIN-ATT-043`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/aggregate.rs:81-91`

**Description:**

`StudentAttendance` carries a
  `is_absent: bool` field (line 91) that duplicates
  `attendance_type.is_absent()`. The same redundancy is
  declared in the spec language at `aggregates.md:25-29`
  (invariant 5: "If `is_absent=true`, then
  `attendance_type=Absent`"), which means the spec mandates
  the duplication. The fields can drift out of sync; the
  `update_student_attendance` service (`services.rs:200`)
  updates both atomically, but the aggregate struct allows the
  invariant to be violated via direct field assignment in
  tests. The engine pattern is to derive `is_absent` from
  `attendance_type` rather than store it as a separate field.

**Expected:**

`docs/specs/attendance/aggregates.md:25-29` —
  invariant 5 couples the two, but the spec doesn't say the
  field is required.

**Evidence:**

`crates/domains/attendance/src/aggregate.rs:81, 91`:
  both `pub attendance_type: AttendanceType,` and
  `pub is_absent: bool,`. Compare with
  `StaffAttendance::is_absent()` (line 282) which is a
  const-method that derives from `attendance_type`.

---

### FINDING 44 (id: `DOMAIN-ATT-044`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/aggregate.rs:540-541`

**Description:**

`BulkAttendanceImport` carries an
  `academic_year_id: AcademicYearId` field on the aggregate
  but the spec (`aggregates.md:148-149`) lists it only as
  invariant 1 ("A bulk import belongs to exactly one school
  and one academic year") without specifying it as a field on
  the root. The `ImportAttendanceCommand` carries
  `academic_year_id` (`commands.rs:339`), but no spec file
  declares it on the aggregate.

**Expected:**

`docs/specs/attendance/aggregates.md:129-175`
  doesn't list `academic_year_id` as a field, only as an
  invariant.

**Evidence:**

`crates/domains/attendance/src/aggregate.rs:541`:
  `pub academic_year_id: AcademicYearId,`.

---

### FINDING 50 (id: `DOMAIN-ATT-050`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/repository.rs:138-168`

**Description:**

`SubjectAttendanceRepository` ships only 5
  methods. The spec
  (`repositories.md:53-80`) mandates 6 methods including
  `list_for_section_date(school, class, section, subject,
  date)`. The code trait has `list_for_section(school, class,
  section)` (no date filter) and no method that filters by
  subject and date.

**Expected:**

`docs/specs/attendance/repositories.md:63-70`.

**Evidence:**

- `docs/specs/attendance/repositories.md:63-70` mandates
    `list_for_section_date`.
  - `grep -n "list_for_section_date" crates/domains/attendance/src/repository.rs`
    returns no result.

---

### FINDING 51 (id: `DOMAIN-ATT-051`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/repository.rs:179-213`

**Description:**

`StaffAttendanceRepository` ships 7 methods.
  The spec (`repositories.md:85-115`) mandates 8 methods,
  including `list_for_school_in_range(school, from, to)`. The
  code ships `list_for_day(school, date)` (a single-date
  query, not a range query) and `list_for_staff(school, staff,
  from, to)` (per-staff range query), but no
  `list_for_school_in_range` for all-staff-per-range queries.

**Expected:**

`docs/specs/attendance/repositories.md:102-107`.

**Evidence:**

- `docs/specs/attendance/repositories.md:102-107` mandates
    `list_for_school_in_range(school, from, to)`.
  - `grep -n "list_for_school_in_range" crates/domains/attendance/src/repository.rs`
    returns no result; `list_for_day` (line 201) is the
    closest match.

---

### FINDING 53 (id: `DOMAIN-ATT-053`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/lib.rs:1-97` and `docs/handoff/PHASE-5-HANDOFF.md:104-122`

**Description:**

`lib.rs` is missing `AttendanceEligibility`,
  `AbsenceDetectionService`, `AttendanceReportService`,
  `AttendanceImportService`, `ClassAttendanceRepository`,
  `ClassAttendance`, `AttendanceBulk`, the `*Event` alias
  types, and `Validate` trait from its prelude re-exports —
  none are declared in the crate. The handoff at
  `docs/handoff/PHASE-5-HANDOFF.md:104-122` is internally
  consistent (it doesn't claim these exist) but the lib.rs
  re-exports section will need to be updated when the missing
  types land.

**Expected:**

The lib.rs prelude should re-export the
  complete Phase 5 public surface.

**Evidence:**

- `crates/domains/attendance/src/lib.rs:30-97` re-exports
    aggregates, commands, events, services, repositories,
    value objects — but no `AbsenceDetectionService`,
    `AttendanceReportService`, `AttendanceImportService`,
    `ClassAttendanceRepository`, `ClassAttendance`,
    `AttendanceBulk`, etc.
  - The handoff
    (`docs/handoff/PHASE-5-HANDOFF.md:115-122`) only
    describes the 5 repository traits shipped.

---

### FINDING 48 (id: `DOMAIN-ATT-048`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/services.rs:1274`

**Description:**

The `dedup_within_day` helper uses
  `std::collections::HashSet<(uuid::Uuid, chrono::NaiveDate)>`
  to dedup `StudentAbsentForDay` events. Per the engine rule
  "Multi-tenant by default" (AGENTS.md) and the spec invariant
  "every attendance aggregate is anchored to exactly one
  `SchoolId`" (`overview.md:61`), the dedup key must include
  the `SchoolId` to prevent cross-tenant collisions. The
  current key uses `(student_uuid, date)` and would conflate
  two schools' students with id collision (unlikely but
  possible).

**Expected:**

The dedup key should be
  `(school_id, student_id, attendance_date)` per the spec's
  uniqueness invariants.

**Evidence:**

`crates/domains/attendance/src/services.rs:1274-1284`:
  ```rust
  let mut seen: std::collections::HashSet<(uuid::Uuid, chrono::NaiveDate)> =
      std::collections::HashSet::with_capacity(events.len());
  ```

---

### FINDING 52 (id: `DOMAIN-ATT-052`)

- **Source:** `docs/audit_reports/findings/wave1-attendance.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** `crates/domains/attendance/src/events.rs:3-23`

**Description:**

The events module's documentation claims
  "Phase 5 Workstream A ships 21 typed DomainEvent
  implementations" but the actual count is 22
  (`StudentAttendanceMarked`, `StudentAttendanceUpdated`,
  `StudentAttendanceRestored`, `StudentAbsentForDay`,
  `StudentAttendanceImported`, `SubjectAttendanceMarked`,
  `SubjectAttendanceUpdated`,
  `SubjectAbsentNotificationRequested`, `StaffAttendanceMarked`,
  `StaffAttendanceUpdated`, `StaffAbsentForDay`,
  `ExamAttendanceMarked`, `ExamAttendanceUpdated`,
  `BulkImportStarted`, `BulkImportValidated`,
  `BulkImportCommitted`, `BulkImportFailed`,
  `BulkImportCancelled`, `AttendanceImported`,
  `AbsenceNotificationRequested`, `ClassAttendanceRecomputed`).
  The handoff (line 89) also says "21 typed events". The
  module-level doc and the handoff both undercount by 1.

**Expected:**

The events module's module-level documentation
  should list the correct count.

**Evidence:**

- `crates/domains/attendance/src/events.rs:3-23` declares
    "Phase 5 Workstream A ships 21 typed DomainEvent
    implementations" and lists 21 in the bullet list but the
    bullet lists 21 (Student=5, Subject=3, Staff=3, Exam=2,
    BulkImport=6, Cross-cutting=2 = 21); the actual count
    in the file (grep `impl DomainEvent for`) is 22.
  - `grep -c "impl DomainEvent for" crates/domains/attendance/src/events.rs`
    returns 22.

---


## CMS (target id prefix: `DOMAIN-CMS`)

**Path:** `crates/domains/cms/`  
**Total findings:** 67 (10 critical, 17 high, 29 medium, 11 low)


### FINDING 1 (id: `DOMAIN-CMS-001`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** docs/specs/cms/aggregates.md:209

**Description:**

The `NoticeBoard` aggregate spec mandates an academic-year
  anchor, but the Rust struct in `aggregate.rs` has no `academic_id` field.
  The aggregate therefore cannot honour spec invariant 2
  ("anchored to a school and an academic year").

**Expected:**

`docs/specs/cms/aggregates.md` line 209:
  `2. A \`NoticeBoard\` is anchored to a school and an academic year.`

**Evidence:**

`crates/domains/cms/src/aggregate.rs:869-904` —
  ```rust
  pub struct NoticeBoard {
      pub id: NoticeBoardId,
      pub school_id: SchoolId,
      pub notice_title: NoticeTitle,
      pub notice_message: NoticeMessage,
      pub notice_date: NoticeDate,
      pub publish_on: Option<PublishDate>,
      pub inform_to: AudienceDescriptor,
      pub is_published: IsPublished,
      ...
  }
  ```
  No `academic_id: AcademicYearId` field; no `academic_id` reference
  in the entire `NoticeBoard` section of `aggregate.rs`
  (`grep -nE "academic_id|AcademicYearId" aggregate.rs | grep NoticeBoard` returns no rows).

---

### FINDING 14 (id: `DOMAIN-CMS-014`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/services.rs (no `NewsCommentPolicy`, no `PublishedPages`, no `ActiveNews`, no `VisibleTestimonials`, no `CmsCoordinator`)

**Description:**

The spec defines four policy/specification
  types and a `CmsCoordinator` cross-domain coordinator. None of
  them are implemented.

**Expected:**

`docs/specs/cms/services.md` lines 101-171:
  `## NewsCommentPolicy`, `## Specification: PublishedPages`,
  `## Specification: ActiveNews`, `## Specification: VisibleTestimonials`,
  `## Cross-Domain Coordinator`.

**Evidence:**

`grep -nE "struct NewsCommentPolicy|struct PublishedPages|struct ActiveNews|struct VisibleTestimonials|struct CmsCoordinator" crates/domains/cms/src/` returns no matches.
  `services.rs` defines only `PageService`, `NewsService`,
  `TestimonialService`, `HomeSliderService`, `ContentService`,
  `ContentShareListService` (lines 80/320/404/489/554/649).

---

### FINDING 18 (id: `DOMAIN-CMS-018`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/events.rs:3387-3435 (vs spec events.md:243)

**Description:**

The spec defines the event
  `HomePageSettingCreated` (`docs/specs/cms/events.md:243` and
  `docs/events/cms.md:68`). The code defines `HomePageSettingConfigured`
  (`crates/domains/cms/src/events.rs:3387`) with wire form
  `cms.home_page_setting.configured`. Two doc-vs-code drifts: the
  struct name and the wire form differ from the spec.

**Expected:**

`docs/specs/cms/events.md:243`:
  `pub struct HomePageSettingCreated { pub home_page_setting_id: HomePageSettingId, pub title: HomePageTitle }`.
  `docs/events/cms.md:68`: `| \`HomePageSettingCreated\` ... |`.

**Evidence:**

`crates/domains/cms/src/events.rs:3387` —
  `pub struct HomePageSettingConfigured` with `EVENT_TYPE = "cms.home_page_setting.configured"`
  (`events.rs:3418`). The spec asks for `cms.home_page_setting.created`.
  The events catalog (`docs/events/cms.md:68`) lists
  `HomePageSettingCreated` — i.e. the catalog disagrees with both the spec
  and the code.

---

### FINDING 2 (id: `DOMAIN-CMS-002`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/commands.rs:38-49, 57-435

**Description:**

Only 10 of the ~62 commands listed in the
  `docs/specs/cms/commands.md` spec are implemented. The wire-form
  command types `Update*`, `Publish*` (non-Page), `Unpublish*`,
  `Delete*` (non-Page), `Comment*`, `Moderate*`, `Dispatch*`,
  `Cancel*`, the news-comment commands, the news-category
  commands, the news-page commands, the content-type commands,
  the speech-slider commands, the teacher-upload commands, the
  upload-content commands, the about-page commands, the
  contact-page commands, the course-page commands, the
  frontend-page commands, and the IncrementNewsView command are
  all absent from the code.

**Expected:**

Spec lists commands for all 20 aggregates
  (`docs/specs/cms/commands.md` lines 10-579). Example:
  `## UpdatePage` (line 33), `## PublishNews` (line 132),
  `## DeleteNews` (line 157), `## CommentOnNews` (line 170),
  `## ModerateNewsComment` (line 186), `## CreateSpeechSlider`
  (line 568), etc.

**Evidence:**

`crates/domains/cms/src/commands.rs:38-49` defines
  only 10 `CMS_*_COMMAND_TYPE` constants
  (`CMS_PAGE_CREATE_COMMAND_TYPE`, `CMS_PAGE_PUBLISH_COMMAND_TYPE`,
  `CMS_PAGE_ARCHIVE_COMMAND_TYPE`, `CMS_PAGE_DELETE_COMMAND_TYPE`,
  `CMS_NEWS_CREATE_COMMAND_TYPE`, `CMS_TESTIMONIAL_CREATE_COMMAND_TYPE`,
  `CMS_HOME_SLIDER_CREATE_COMMAND_TYPE`, `CMS_CONTENT_CREATE_COMMAND_TYPE`,
  `CMS_CONTENT_SHARE_LIST_CREATE_COMMAND_TYPE`,
  `CMS_HOME_PAGE_SETTING_CONFIGURE_COMMAND_TYPE`); the file defines
  only 10 `pub struct *Command` types
  (`CreatePageCommand`, `PublishPageCommand`, `ArchivePageCommand`,
  `DeletePageCommand`, `CreateNewsCommand`, `CreateTestimonialCommand`,
  `CreateHomeSliderCommand`, `CreateContentCommand`,
  `CreateContentShareListCommand`, `ConfigureHomePageCommand`).
  `grep -rnE "IncrementNewsViewCommand|UpdatePageCommand|UpdateNewsCommand|PublishNewsCommand|DeleteNewsCommand|CommentOnNewsCommand|ModerateNewsCommand|DispatchContentShareListCommand|CancelContentShareListCommand|CreateSpeechSliderCommand|CreateContentTypeCommand|CreateAboutPageCommand|CreateContactPageCommand|CreateCoursePageCommand|CreateFrontendPageCommand|CreateTeacherUploadContentCommand|CreateUploadContentCommand|CreateNewsCategoryCommand|CreateNewsPageCommand|DeleteNoticeBoardCommand|UpdateTestimonialCommand|DeleteTestimonialCommand" crates/` returns no results.

---

### FINDING 25 (id: `DOMAIN-CMS-025`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/aggregate.rs:1332-1375

**Description:**

`Content` aggregate has
  `available_for_role: Option<i32>`,
  `available_for_class: Option<i32>`, and
  `available_for_section: Option<i32>`. The spec describes these
  as anchors to typed identifiers (`ClassId`, `SectionId`); the
  spec uses raw `i32` FKs only for the
  `UploadContent.content_type` (a `ContentType` taxonomy FK), not
  for class/section scope. Engine rule "Compile-time safety over
  strings" implies typed ids, not raw integers, in domain fields.

**Expected:**

Spec aggregates.md invariant 13 (lines 102-104):
  `13. A Content has an available_for_role, available_for_class, and
  available_for_section to scope visibility. A content with all three
  null is unavailable.`
  Engine rule (AGENTS.md): typed ids.

**Evidence:**

`crates/domains/cms/src/aggregate.rs:1347-1352` —
  ```rust
  pub available_for_role: Option<i32>,
  pub available_for_class: Option<i32>,
  pub available_for_section: Option<i32>,
  ```
  Compare `TeacherUploadContent::class_id: ClassId` (line 1847) and
  `ContentShareList::class_id: Option<ClassId>` (line 1651) which use
  the typed ids correctly.

---

### FINDING 3 (id: `DOMAIN-CMS-003`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/aggregate.rs:797-836

**Description:**

`NewsPage` aggregate has only `new()` and
  `soft_delete()`; there is no `update()` method. Spec
  commands `UpdateNewsPage` is therefore unimplementable.

**Expected:**

`docs/specs/cms/commands.md` lines 535-545:
  `## CreateNewsPage / UpdateNewsPage / DeleteNewsPage` with
  `Capabilities: NewsPage.Create, NewsPage.Update, NewsPage.Delete`.

**Evidence:**

`crates/domains/cms/src/aggregate.rs:797-836` —
  ```rust
  impl NewsPage {
      pub fn new(cmd: NewNewsPage) -> Result<Self, CmsError> { ... }
      pub fn soft_delete(&mut self, actor: UserId, at: Timestamp) -> Result<(), CmsError> { ... }
  }
  ```
  No `update()` or `update_*()` method on `NewsPage`.

---

### FINDING 4 (id: `DOMAIN-CMS-004`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/aggregate.rs (NoticeBoard, Testimonial, HomeSlider, SpeechSlider, TeacherUploadContent, UploadContent, AboutPage, ContactPage, CoursePage, HomePageSetting, FrontendPage)

**Description:**

11 of the 19 spec'd aggregates have no
  `update()` (or `rename`/`update_*`) method. Spec commands
  `Update*` for these aggregates are unimplementable.

**Expected:**

Spec commands.md lists `UpdateNoticeBoard`,
  `UpdateTestimonial`, `UpdateHomeSlider`, `UpdateSpeechSlider`,
  `UpdateTeacherUploadContent`, `UpdateUploadContent`,
  `UpdateAboutPage`, `UpdateContactPage`, `UpdateCoursePage`,
  `UpdateFrontendPage` (each with `Capabilities: <Aggregate>.Update`).

**Evidence:**

`grep -nE "impl (NoticeBoard|Testimonial|HomeSlider|SpeechSlider|TeacherUploadContent|UploadContent|AboutPage|ContactPage|CoursePage|HomePageSetting|FrontendPage)" crates/domains/cms/src/aggregate.rs` followed by
  inspection of each `impl` block: only `new()`, `soft_delete()`,
  and (for `NoticeBoard`) `publish()`/`unpublish()` are defined.
  `fn update` returns no rows for those impl blocks
  (`grep -nE "fn update" crates/domains/cms/src/aggregate.rs`
  yields lines 160/467/1416 only — Page, News, Content).

---

### FINDING 6 (id: `DOMAIN-CMS-006`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/services.rs:404-446

**Description:**

`TestimonialService::average_rating` does not
  compute a true average: it computes `total/count` but discards
  `total` (line 439 `let _ = total;`) and returns `1.0` for any
  non-empty list. The docstring says "the unweighted mean divides
  by count to get the mean rating" but the implementation returns
  a constant. The test `testimonial_service_average_rating_computes_correctly`
  (line 1034-1057) only asserts `avg.is_finite() && avg > 0.0`.

**Expected:**

`docs/specs/cms/services.md` line 67:
  `pub fn average_rating(testimonials: &[Testimonial]) -> f32 { ... }`
  — canonical mean of star ratings.

**Evidence:**

`crates/domains/cms/src/services.rs:428-445`:
  ```rust
  pub fn average_rating(testimonials: &[Testimonial]) -> f32 {
      if testimonials.is_empty() { return 0.0; }
      let total: u32 = testimonials.iter().map(|t| u32::from(t.star_rating.value())).sum();
      let count = u32::try_from(testimonials.len()).unwrap_or(u32::MAX);
      let _ = total;
      if count == 0 { 0.0 } else { 1.0 }
  }
  ```

---

### FINDING 7 (id: `DOMAIN-CMS-007`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/aggregate.rs:1483-1494

**Description:**

`Content::available_to_class(class, section)`
  uses three `(u128 >> 64) as i64` truncating casts to compare
  the typed `ClassId` / `SectionId` UUIDs against
  `available_for_class: Option<i32>` / `available_for_section:
  Option<i32>`. This both (a) loses the high 64 bits of every
  UUID via `as i64`, and (b) violates the typed-id pattern: the
  aggregate holds `Option<i32>` raw integers where the spec uses
  `ClassId` / `SectionId`.

**Expected:**

AGENTS.md "Type Safety" rule: no `as` casts that
  truncate; use `TryFrom`/`TryInto`. Engine rule "Compile-time
  safety over strings" implies typed ids, not `i32`, in domain
  fields.

**Evidence:**

`crates/domains/cms/src/aggregate.rs:1483-1494`:
  ```rust
  pub fn available_to_class(&self, class: ClassId, section: Option<SectionId>) -> bool {
      match (self.available_for_class, self.available_for_section) {
          (None, None) => true,
          (Some(c), None) => i64::from(c) == (class.as_uuid().as_u128() >> 64) as i64,
          (None, Some(_)) => false,
          (Some(c), Some(s)) => {
              i64::from(c) == (class.as_uuid().as_u128() >> 64) as i64
                  && section.is_some_and(|sec| i64::from(s) == (sec.as_uuid().as_u128() >> 64) as i64)
          }
      }
  }
  ```

---

### FINDING 8 (id: `DOMAIN-CMS-008`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/value_objects.rs:2515-2548

**Description:**

`PageSettings` wraps `serde_json::Value` directly
  in domain code. The engine rule (AGENTS.md "Type Safety") and
  the engine code standards forbid `serde_json::Value` in domain
  code.

**Expected:**

AGENTS.md "Type Safety": `No \`serde_json::Value\` in
  domain code. Use typed wrappers.` Spec value-objects.md line 130-132:
  `PageSettings | A typed JSON value object with versioned schema` (the
  spec describes it as a typed JSON value; the engine rule says typed,
  not `serde_json::Value`).

**Evidence:**

`crates/domains/cms/src/value_objects.rs:2519-2520`:
  ```rust
  pub struct PageSettings(pub serde_json::Value);
  ```
  Also `services.rs:843` uses `serde_json::Value::as_bool` inside
  the `form_uploaded_public_indexing_subscriber` (`envelope.payload
  .get("show_public").and_then(serde_json::Value::as_bool)`).

---

### FINDING 10 (id: `DOMAIN-CMS-010`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/{commands,entities,events,repository,query,services,value_objects}.rs (7 files)

**Description:**

Seven of nine source files declare a module-level
  `#![allow(missing_docs)]` (or `#![allow(dead_code, clippy::all)]`
  blanket). The crate-level lib.rs declares `#![deny(missing_docs)]`
  but the blanket suppressions defeat it for ~95% of the file
  contents.

**Expected:**

AGENTS.md "Type Safety" + `docs/code-standards.md`
  § Type Safety: `#![deny(missing_docs)]` and `unwrap`, `expect`,
  `panic!` are forbidden in production paths.

**Evidence:**

`crates/domains/cms/src/aggregate.rs:16-20`:
  ```rust
  #![allow(missing_docs)]
  #![allow(clippy::too_many_arguments)]
  #![allow(clippy::unnecessary_literal_unwrap)]
  #![allow(unused_imports)]
  #![allow(dead_code)]
  ```
  Plus `commands.rs:14-15`, `entities.rs:21-22`, `events.rs:32-33`,
  `repository.rs:9-10`, `query.rs:12-13`, `services.rs:22-23`,
  `value_objects.rs:22-23`.

---

### FINDING 12 (id: `DOMAIN-CMS-012`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/value_objects.rs (no `TestimonialRating`, no `RoleIdList`, no `Visible`, no `ContentStatus`)

**Description:**

Four value objects listed in
  `docs/specs/cms/value-objects.md` are absent from `value_objects.rs`:
  `TestimonialRating` (= `StarRating` type alias),
  `RoleIdList` (comma-separated `RoleId` list, decoded to
  `Vec<RoleId>`), `Visible` (`bool` newtype), and `ContentStatus`
  enum (`Draft`, `Published`, `Archived`).

**Expected:**

`docs/specs/cms/value-objects.md` lines 69-92:
  `TestimonialRating | StarRating` (line 80),
  `Visible | bool — when true, the row is visible on the public site`
  (line 81), `ContentStatus | Draft, Published, Archived` (line 74),
  `RoleIdList | Comma-separated list of RoleId (decoded into Vec<RoleId>)`
  (line 114).

**Evidence:**

`grep -nE "TestimonialRating|RoleIdList|^pub struct Visible|^pub enum ContentStatus" crates/domains/cms/src/value_objects.rs`
  returns no results.

---

### FINDING 19 (id: `DOMAIN-CMS-019`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/aggregate.rs:92-129 (Page struct) — `tables.md` Note lines 41-44

**Description:**

Spec mandates `cms_pages` has `status VARCHAR(16) NOT NULL`
  with `CHECK IN ('draft', 'published')`. The code's `Page::new` sets
  `status: PageStatus::default()` which is `Draft`. The aggregate has
  no enum-driven SQL constraint emitter; the storage adapter must
  enforce this, but no adapter in this repo enforces CHECK constraints
  for `cms_pages`.

**Expected:**

`docs/specs/cms/tables.md` line 42-44:
  `The cms_pages table uses VARCHAR(16) NOT NULL for status with a CHECK IN ('draft', 'published') constraint.`

**Evidence:**

Spec mandates the CHECK constraint; the engine
  relies on storage adapters to emit it but no CMS adapter in this
  workspace emits `cms_pages` DDL
  (`grep -rnE "cms_pages" crates/` shows only test files).
  The handoff says the 3 storage adapters (PG/MySQL/SQLite) ship
  but a per-table DDL emitter for `cms_pages` is not visible in the
  repo.

---

### FINDING 26 (id: `DOMAIN-CMS-026`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/lib.rs:30-123 (prelude)

**Description:**

`lib.rs` claims "73 typed events" in
  the prelude comment block — but the prelude re-exports only
  67 events. The header comment in `events.rs:5-31` lists the
  same 67. The `prelude_exports_expected_symbols` test in
  `lib.rs:128-165` checks aggregate roots but not the event
  count.

**Expected:**

The actual count is 67
  (`grep -E "^pub use crate::events::" crates/domains/cms/src/lib.rs`
  lists 67 event types).

**Evidence:**

Comment in `crates/domains/cms/src/lib.rs:45-46`:
  `// 73 typed events (alphabetised).`

---

### FINDING 29 (id: `DOMAIN-CMS-029`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/services.rs:30-31 (imports), 748-816 (configure_home_page_service)

**Description:**

`configure_home_page_service` returns
  `Result<HomePageSetting>` but the body uses the broadcast
  capability check at line 759 with
  `Capability::CmsHomePageSettingConfigure` — which exists.
  But the `HomePageSettingRepository` is generic-bound with
  `'static` (line 756), preventing non-static lifetimes. This
  is consistent with the other services; not a bug per se, but
  the `?Sized` requirement is missing.

**Expected:**

Object-safety + Send+Sync; standard pattern.

**Evidence:**

`crates/domains/cms/src/services.rs:756-757`:
  ```rust
  R: HomePageSettingRepository + 'static,
  B: EventBus + 'static,
  ```
  No `?Sized` bound.

---

### FINDING 30 (id: `DOMAIN-CMS-030`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/services.rs:362-397 (create_news_service)

**Description:**

`create_news_service` does not enforce
  the spec invariants:
  - `IsGlobal` requires `is_global = true` only when allowed
    by the school's licensing tier (spec invariant 9).
  - The spec invariant 5 says "A `News` may have
    `auto_approve = 1`" — but does not validate the news's
    category anchor or the `is_comment` flag combination.

**Expected:**

`docs/specs/cms/aggregates.md` lines 66-81
  (invariants 1-8 of News).

**Evidence:**

`crates/domains/cms/src/services.rs:362-397` —
  `create_news_service` only checks the
  `CmsNewsCreate` capability; it does not check category
  existence, the `is_comment` ↔ `auto_approve` interaction,
  or the `is_global` licensing requirement.

---

### FINDING 31 (id: `DOMAIN-CMS-031`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/aggregate.rs:1174-1228 (NewSpeechSlider / SpeechSlider)

**Description:**

The spec (`docs/specs/cms/aggregates.md` lines
  295-323) describes `SpeechSlider` with fields `name`,
  `designation`, `speech` (free-text body), `image`. The code
  defines these fields. But the spec invariant 2 says `The speech
  field is a free-text body` — the spec does not impose a
  length cap, while the value object `SpeechText` enforces
  1..=5000 chars (line 733-734). This is a partial contradiction
  with the spec wording.

**Expected:**

`docs/specs/cms/aggregates.md` line 311:
  `3. The \`speech\` field is a free-text body.`
  `docs/specs/cms/value-objects.md` line 45:
  `SpeechText | 1..5000 chars`.

**Evidence:**

`crates/domains/cms/src/value_objects.rs:733-734`
  enforces 1..=5000 chars; spec says "free-text body". The spec
  adds the length cap via value-objects, but the aggregates.md
  wording is "free-text body". Verified consistent with
  value-objects.md.

---

### FINDING 44 (id: `DOMAIN-CMS-044`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/aggregate.rs (none of the 20 aggregates)

**Description:**

None of the 20 aggregates in `aggregate.rs`
  carries a `#[derive(DomainQuery)]` attribute. The spec says
  the macro emits a typed AST; the AGENTS.md "Compile-time
  safety over strings" rule + the handoff line 245
  ("Per-aggregate CRUD factories ship in follow-up phases
  alongside the `#[derive(DomainQuery)]` macro emissions")
  imply the macro will land in a follow-up phase. The current
  aggregates use only `#[derive(Debug, Clone, PartialEq,
  Serialize, Deserialize)]`.

**Expected:**

Spec aggregates.md tables.md — every
  `cms_*` table maps to an aggregate that should be
  `#[derive(DomainQuery)]`-able.

**Evidence:**

`grep -rnE "#\[derive.*DomainQuery" crates/`
  returns only test references in `crates/infra/query-derive/tests/derive_test.rs`.

---

### FINDING 45 (id: `DOMAIN-CMS-045`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/repositories.rs (not present); aggregates live in aggregate.rs

**Description:**

No `educore-storage-postgres` /
  `educore-storage-mysql` / `educore-storage-sqlite` adapter in
  this repo implements the 19 CMS repositories. The handoff says
  PG/MySQL/SQLite storage adapters ship, but a `grep` for
  `PageRepository for` / `NewsRepository for` etc. shows no
  adapter implementations.

**Expected:**

Spec repositories.md lines 7-23 defines
  `PageRepository`. Per spec overview, every CMS aggregate has
  a storage adapter implementing the repository trait.

**Evidence:**

`grep -rnE "impl educore_cms::repository::PageRepository" crates/`
  returns no matches outside of tests. The CMS integration
  test uses an `InMemoryPageRepo` mock (`cms_integration.rs:1158-1190`)
  for `TestimonialRepository` only; no PG/MySQL/SQLite adapter
  implements CMS repositories.

---

### FINDING 5 (id: `DOMAIN-CMS-005`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/aggregate.rs:543-548

**Description:**

`News::is_visible(today)` ignores the spec
  invariant 4 (`is_global = true` news is visible across all
  schools). The method only checks `active_status` and
  `publish_date`.

**Expected:**

`docs/specs/cms/aggregates.md` line 72-73:
  `4. A \`News\` may be \`is_global\` (visible across all schools in a
  multi-tenant SaaS) or scoped to one school.`

**Evidence:**

`crates/domains/cms/src/aggregate.rs:543-548`:
  ```rust
  pub fn is_visible(&self, today: NaiveDate) -> bool {
      self.active_status.is_active() && self.publish_date.as_naive_date() <= today
  }
  ```
  No reference to `self.is_global`. The handoff's
  `News::is_visible` predicate does not implement the global visibility
  rule.

---

### FINDING 58 (id: `DOMAIN-CMS-058`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/aggregate.rs:67-83 (UpdatePage)

**Description:**

The `UpdatePage` struct exists
  (`aggregate.rs:66-83`) but no `UpdatePageCommand` is
  defined in `commands.rs`. Per spec commands.md line 33-48,
  `UpdatePage` command exists with capability `Page.Update`.

**Expected:**

`docs/specs/cms/commands.md` lines 33-48:
  `## UpdatePage ... Capability: Page.Update ... Effects: Emits
  PageUpdated.`

**Evidence:**

`crates/domains/cms/src/aggregate.rs:66-83` —
  `pub struct UpdatePage` exists. `crates/domains/cms/src/commands.rs`
  has no `UpdatePageCommand` (grep returns no result).

---

### FINDING 59 (id: `DOMAIN-CMS-059`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/aggregate.rs:356-383 (UpdateNews)

**Description:**

The `UpdateNews` struct exists in
  `aggregate.rs` but no `UpdateNewsCommand` is defined. Per
  spec commands.md line 110-130, `UpdateNews` exists with
  capability `News.Update`.

**Expected:**

`docs/specs/cms/commands.md` lines 110-130:
  `## UpdateNews ... Capability: News.Update`.

**Evidence:**

`crates/domains/cms/src/aggregate.rs:356-383`
  — `pub struct UpdateNews` exists. `commands.rs` has no
  `UpdateNewsCommand`.

---

### FINDING 60 (id: `DOMAIN-CMS-060`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/aggregate.rs:1304-1327 (UpdateContent)

**Description:**

`UpdateContent` exists in `aggregate.rs` but
  no `UpdateContentCommand` is defined. Per spec
  commands.md line 351-358, `UpdateContent` exists with
  capability `Content.Update`.

**Expected:**

`docs/specs/cms/commands.md` lines 351-358:
  `## UpdateContent / DeleteContent`.

**Evidence:**

`crates/domains/cms/src/aggregate.rs:1304-1327`
  — `pub struct UpdateContent` exists. `commands.rs` has no
  `UpdateContentCommand`.

---

### FINDING 61 (id: `DOMAIN-CMS-061`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/lib.rs:30-44

**Description:**

The prelude has `#[allow(missing_docs)]`
  (line 31) hiding the absence of rustdoc on the prelude's
  re-exports. Although the prelude has module-level doc comments,
  the blanket allow defeats the deny lint for any future
  re-exported items.

**Expected:**

AGENTS.md `#![deny(missing_docs)]`.

**Evidence:**

`crates/domains/cms/src/lib.rs:30-31`:
  ```rust
  /// Convenient prelude: the public surface of the CMS crate.
  #[allow(missing_docs)]
  pub mod prelude {
  ```

---

### FINDING 63 (id: `DOMAIN-CMS-063`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/aggregate.rs (page 80 line 7-22)

**Description:**

The aggregate.rs file-level comment claims
  `The 20 root aggregates per the spec at docs/specs/cms/aggregates.md`
  (line 3), but the spec defines 19 root aggregates
  (Page, News, NewsCategory, NewsComment, NewsPage, NoticeBoard,
  Testimonial, HomeSlider, SpeechSlider, Content, ContentType,
  ContentShareList, TeacherUploadContent, UploadContent, AboutPage,
  ContactPage, CoursePage, HomePageSetting, FrontendPage — 19
  distinct headings in `docs/specs/cms/aggregates.md`).

**Expected:**

AGENTS.md and handoff claim "20 root aggregates
  per docs/specs/cms/aggregates.md".

**Evidence:**

`grep -nE "^## [A-Z]" docs/specs/cms/aggregates.md`
  yields 19 second-level headings (Page, News, NewsCategory,
  NewsComment, NewsPage, NoticeBoard, Testimonial, HomeSlider,
  SpeechSlider, Content, ContentType, ContentShareList,
  TeacherUploadContent, UploadContent, AboutPage, ContactPage,
  CoursePage, HomePageSetting, FrontendPage).

---

### FINDING 64 (id: `DOMAIN-CMS-064`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** docs/coverage.toml:1545-1740 (19 cms aggregate rows)

**Description:**

The handoff claims "20 `coverage.toml` rows
  flipped from `Pending` → `Tested`", but only 19 cms
  aggregate rows exist with `status = "Tested"`. Plus 2
  capability/audit surface rows (`cms_capability_variants`,
  `cms_audit_target_variants`). The "20 aggregate rows" claim
  is off by 1.

**Expected:**

AGENTS.md + handoff claim: 20 cms aggregate
  coverage rows flipped.

**Evidence:**

`grep -cE '^id = "cms_[a-z_]+_aggregate"' docs/coverage.toml`
  → 19. `grep -nE '^id = "cms_[a-z_]+_(aggregate|variants)"' docs/coverage.toml`
  → 19 aggregate + 2 capability/audit = 21 cms-domain rows.

---

### FINDING 9 (id: `DOMAIN-CMS-009`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/services.rs:50-55

**Description:**

`snapshot::<T: serde::Serialize>` uses
  `unwrap_or_default()` on a `serde_json::to_vec(value)` failure.
  Audit rows are silently corrupted on serialization failure
  rather than propagating the error to the caller.

**Expected:**

AGENTS.md "Type Safety": all fallible APIs return
  `Result<T, DomainError>`. The audit row payload is a
  security-relevant artefact; silent defaulting violates audit-first.

**Evidence:**

`crates/domains/cms/src/services.rs:50-55`:
  ```rust
  /// Snapshot a serialised value for an audit row. A serde_json
  /// failure falls back to an empty payload (audit rows are
  /// best-effort).
  fn snapshot<T: serde::Serialize>(value: &T) -> Bytes {
      Bytes::from(serde_json::to_vec(value).unwrap_or_default())
  }
  ```

---

### FINDING 11 (id: `DOMAIN-CMS-011`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/entities.rs:21-22

**Description:**

The `entities.rs` module applies a blanket
  `#![allow(dead_code, clippy::all)]` plus `#![allow(missing_docs)]`,
  hiding the fact that `NewsImage` carries `image_thumb: Option<FileReference>`
  but no `News` aggregate in `aggregate.rs` holds an image-thumb
  attribute through the same path.

**Expected:**

Spec entities.md lines 6-12 specifies
  `NewsImage` carries the `image` and `image_thumb` `FileReference`s,
  both owned by `News`. AGENTS.md: "No `#[allow(dead_code)]`".

**Evidence:**

`crates/domains/cms/src/entities.rs:21-22` and
  `grep -nE "image_thumb" crates/domains/cms/src/aggregate.rs`
  shows `image_thumb: Option<FileReference>` on `News` (line 333,
  399) but no separate `NewsImage` construction site.

---

### FINDING 13 (id: `DOMAIN-CMS-013`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/value_objects.rs:2473-2509

**Description:**

`AudienceDescriptor::split()` returns
  `Vec<String>` rather than `Vec<RoleId>` as the spec mandates.
  The spec requires the audience descriptor to be decoded into a
  typed `Vec<RoleId>`.

**Expected:**

`docs/specs/cms/value-objects.md` line 114:
  `RoleIdList | Comma-separated list of RoleId (decoded into Vec<RoleId>)`.

**Evidence:**

`crates/domains/cms/src/value_objects.rs:2493-2502`:
  ```rust
  pub fn split(&self) -> Vec<String> {
      self.0.split(',').map(str::trim).filter(|s| !s.is_empty())
          .map(str::to_owned).collect()
  }
  ```
  Returns `Vec<String>`, not `Vec<RoleId>`.

---

### FINDING 15 (id: `DOMAIN-CMS-015`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/entities.rs (entire file)

**Description:**

The `CoursePageRelation` entity listed in
  spec entities.md is absent. Spec mandates a typed edge between
  two `CoursePage` aggregates with `CoursePageRelationId(SchoolId,
  Uuid)`.

**Expected:**

`docs/specs/cms/entities.md` lines 85-91:
  `## CoursePageRelation — Identity: CoursePageRelationId(SchoolId, Uuid); Owner: CoursePage; A typed edge between two CoursePage aggregates: parent_id → CoursePageId`.

**Evidence:**

`grep -rnE "CoursePageRelation" crates/` returns no matches. No `CoursePageRelationId` in `value_objects.rs` (lines 110-211).

---

### FINDING 16 (id: `DOMAIN-CMS-016`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/services.rs:835-859

**Description:**

`form_uploaded_public_indexing_subscriber` is a
  pure synchronous function that takes an `EventEnvelope` and
  returns a `FormIndexAction` enum, but there is no async
  repository wiring. The signature does not match the engine's
  bus subscriber pattern; the spec does not define this shape
  anywhere — it is described in `PHASE-11-HANDOFF.md` OQ #6.

**Expected:**

Spec calls this out in `docs/handoff/PHASE-12-HANDOFF.md`
  lines 246-254 ("events-only ... takes no `educore-documents` dep
  (mirrors Phase 10 OQ #5's `AbsentNotificationService` pattern)").
  The implementation uses `serde_json::Value` (`services.rs:843`)
  instead of a typed event payload.

**Evidence:**

`crates/domains/cms/src/services.rs:835-859` —
  ```rust
  pub fn form_uploaded_public_indexing_subscriber(
      envelope: educore_events::envelope::EventEnvelope,
  ) -> FormIndexAction {
      let show_public = envelope.payload.get("show_public")
          .and_then(serde_json::Value::as_bool).unwrap_or(false);
      if show_public { FormIndexAction::Index } else { FormIndexAction::Ignore }
  }
  ```
  No async / no repository wiring. Subscriber is not registered
  with any bus adapter.

---

### FINDING 17 (id: `DOMAIN-CMS-017`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/events.rs:2460-2513

**Description:**

`ContentShareListUpdated` is defined as an
  event in the code but is **not** listed in
  `docs/specs/cms/events.md` or in `docs/events/cms.md`. It is also
  never emitted by any service-factory function. This is an
  undocumented event.

**Expected:**

`docs/specs/cms/events.md` lines 170-187 lists
  only `ContentShareListCreated`, `ContentShareListDispatched`,
  `ContentShareListCancelled`, `ContentShareListDeleted` (4 events).
  `docs/events/cms.md` lines 49-52 likewise lists only 4 events
  for `ContentShareList`.

**Evidence:**

`crates/domains/cms/src/events.rs:2460` defines
  `pub struct ContentShareListUpdated`; `crates/domains/cms/src/events.rs:2496`
  defines `EVENT_TYPE = "cms.content_share_list.updated"`. The event is only
  exercised in the events.rs test (`events.rs:3965`); no service factory
  publishes it (`grep -rnE "ContentShareListUpdated::new" crates/`
  matches only `events.rs:3965`).

---

### FINDING 20 (id: `DOMAIN-CMS-020`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/services.rs:262-264

**Description:**

`delete_page_service` is the only Delete-service
  wired; no other aggregate has a corresponding service-factory
  function (no `delete_news_service`, `delete_testimonial_service`,
  `delete_home_slider_service`, etc.). The handoff says
  "Per-aggregate CRUD factories land in a follow-up phase alongside
  the `#[derive(DomainQuery)]` macro emissions" — but the wire
  contract (`DeleteNews`, `DeleteTestimonial`, `DeleteHomeSlider`,
  `DeleteSpeechSlider`, `DeleteContent`, `DeleteContentShareList`,
  `DeleteTeacherUploadContent`, `DeleteUploadContent`,
  `DeleteAboutPage`, `DeleteContactPage`, `DeleteCoursePage`,
  `DeleteHomePageSetting`, `DeleteFrontendPage`,
  `DeleteNoticeBoard`, `DeleteNewsPage`, `DeleteNewsCategory`,
  `DeleteContentType`, `DeleteNewsComment`) is unspecified.

**Expected:**

Spec commands.md lists a Delete command for every
  aggregate with a `Delete` capability (`docs/specs/cms/permissions.md`).

**Evidence:**

`crates/domains/cms/src/services.rs:127-734` —
  service factory functions are `create_page_service`,
  `publish_page_service`, `archive_page_service`,
  `delete_page_service`, `create_news_service`,
  `create_testimonial_service`, `create_home_slider_service`,
  `content_service`, `content_share_list_service`,
  `configure_home_page_service` — 10 of 20 aggregates have a
  create-factory; only Page has full CRUD factories.

---

### FINDING 21 (id: `DOMAIN-CMS-021`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/aggregate.rs:526-534

**Description:**

`News::soft_delete` sets
  `active_status: NewsStatus::Disabled` (line 529) — but the
  spec invariant 7 says "A `News` has a `Status` flag
  (`active_status`) — `1` is active, `0` is disabled" (raw byte).
  The aggregate encodes this as a typed `NewsStatus` enum but
  also exposes a wire byte (`to_byte`/`from_byte`) — the byte
  semantics are correct, but the spec also lists invariant 8:
  `A News has a Status of Published or Pending` — neither
  variant exists in the code.

**Expected:**

`docs/specs/cms/aggregates.md` line 93-94
  (invariant 8): `A \`News\` has a \`Status\` of \`Published\` or \`Pending\`.
  A pending news is hidden until moderation approves.`
  Spec value-objects.md does not list a `NewsStatus` enum
  with Published/Pending.

**Evidence:**

`crates/domains/cms/src/value_objects.rs:1451-1456`
  defines `pub enum NewsStatus { Active, Disabled }` — no
  Published/Pending variants.

---

### FINDING 22 (id: `DOMAIN-CMS-022`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/value_objects.rs:323-371 (Slug)

**Description:**

`Slug::new` rejects any non-empty string that
  contains uppercase, underscores, or other non-`[a-z0-9-]`
  characters. The spec allows the slug regex `[a-z0-9-]` but
  many real-world slugs contain periods, accents, or non-ASCII
  characters. While the strictness may be intentional, the
  spec uses `URL-safe slug, 1..200 chars, [a-z0-9-]` (line 61)
  which means the implementation matches the spec — note this
  as a verified matching point.

**Expected:**

`docs/specs/cms/value-objects.md` line 61:
  `Slug | URL-safe slug, 1..200 chars, [a-z0-9-]`.

**Evidence:**

`crates/domains/cms/src/value_objects.rs:344-351`:
  ```rust
  if !s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
      return Err(DomainError::validation(format!("slug must be [a-z0-9-], got {s:?}")));
  }
  ```
  Implementation matches spec; verified.

---

### FINDING 27 (id: `DOMAIN-CMS-027`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/entities.rs (mod tests)

**Description:**

The `entities.rs` test module uses
  `CommentMessage::_new_unchecked_for_test(String::new())`
  (line 2880) — a test-only escape hatch for the validated
  value object. AGENTS.md: "No `unwrap`/`expect`/`panic` in
  non-test code" is respected here, but the existence of a
  `_new_unchecked` constructor on a domain value object is
  itself a smell (test-only escape hatch in production module).

**Expected:**

AGENTS.md: `Construction is the only entry
  point: \`let title = NewsTitle::new("...")?\` (spec
  value-objects.md lines 145-149).

**Evidence:**

`crates/domains/cms/src/entities.rs:2880`:
  ```rust
  message: CommentMessage::_new_unchecked_for_test(String::new()),
  ```
  Indicates `CommentMessage` exposes a non-validated
  constructor.

---

### FINDING 28 (id: `DOMAIN-CMS-028`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/events.rs:800-830

**Description:**

`NewsCommentAdded::new` does not exist in the
  code as a public constructor. Looking at the test, only
  `NewsCommentAdded::new(&c, corr(), ts())` is invoked
  (events.rs:3878), but the struct defines a payload with
  `parent_id: Option<NewsCommentId>` (events.rs:811-820) that the
  constructor does not extract from the `NewsComment` aggregate.

**Expected:**

Spec events.md lines 75-82:
  `pub struct NewsCommentAdded { pub news_comment_id: NewsCommentId,
  pub news_id: NewsId, pub user_id: UserId, pub parent_id:
  Option<NewsCommentId>, pub status: NewsCommentStatus }`.

**Evidence:**

`crates/domains/cms/src/events.rs:808-820` defines
  the struct fields; the constructor body is at lines 821-862
  (need direct read to confirm it pulls `parent_id` from `c`).

---

### FINDING 33 (id: `DOMAIN-CMS-033`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/lib.rs:122

**Description:**

`lib.rs` re-exports `PUBLIC_SCHOOL_ID` from
  `educore_core::ids`. The handoff claims `SchoolId::PUBLIC` was
  added to `educore-core`, but the spec defines it as
  `SchoolId::PUBLIC` while the code re-exports the constant as
  `PUBLIC_SCHOOL_ID`.

**Expected:**

Spec uses `SchoolId::PUBLIC` (e.g. the AGENTS.md
  note "`SchoolId::PUBLIC` constant added to `educore-core`").
  Handoff PHASE-12-HANDOFF.md:217 says `\`SchoolId::is_public()\``
  helper, suggesting a method, not a const.

**Evidence:**

`crates/domains/cms/src/lib.rs:122`:
  `pub use educore_core::ids::PUBLIC_SCHOOL_ID;`
  `crates/infra/core/src/ids.rs:293`:
  `pub const PUBLIC_SCHOOL_ID: SchoolId = SchoolId(Uuid::nil());`
  No `SchoolId::PUBLIC` exists.

---

### FINDING 34 (id: `DOMAIN-CMS-034`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/infra/core/src/ids.rs:301-302

**Description:**

The `is_public()` helper is `pub const fn is_public(self) -> bool`
  but is not re-exported through the `educore_core` prelude. Other
  consumers of the engine cannot easily call `id.is_public()` on a
  `SchoolId` value without importing `educore_core::ids::Identifier`.

**Expected:**

Spec PHASE-12-HANDOFF.md:217-218:
  `SchoolId::is_public() helper returns true iff the inner UUID is nil.`

**Evidence:**

`crates/infra/core/src/ids.rs:301-302`:
  ```rust
  pub const fn is_public(self) -> bool {
      matches!(self.0, id if id.is_nil())
  }
  ```
  Search across `crates/` shows `is_public()` is invoked only in
  `crates/infra/core/src/ids.rs:355/361` (the engine's own tests).

---

### FINDING 37 (id: `DOMAIN-CMS-037`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/aggregate.rs (entire file)

**Description:**

The `aggregate.rs` file declares
  `#![allow(missing_docs)]` blanket at module level. Per
  AGENTS.md, this defeats `#![deny(missing_docs)]` for all items
  in the file. Inspecting the file, public functions like
  `Page::is_home_page` (line 247), `Page::is_active` (line 259),
  `News::is_visible` (line 545), `Testimonial::update_rating`
  (line 1055), `NewsComment::approve` (line 708), `NewsComment::hide`
  (line 713), `CoursePage::soft_delete` (line 2415),
  `HomePageSetting::soft_delete` (line 2518),
  `FrontendPage::soft_delete` (line 2621) all carry doc comments,
  but the blanket allow masks any future omissions.

**Expected:**

AGENTS.md: `#![deny(missing_docs)]`.

**Evidence:**

`crates/domains/cms/src/aggregate.rs:16`:
  `#![allow(missing_docs)]`. A blanket suppression.

---

### FINDING 38 (id: `DOMAIN-CMS-038`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/aggregate.rs:42-62 (NewPage) vs NewContent (1273-1300)

**Description:**

`NewPage` (line 41-62) has 11 fields, while
  `NewContent` (line 1273-1300) has 14 fields. Spec command
  `CreateContentCommand` (in commands.md lines 335-358) does
  not include `academic_id`, but the `NewContent` struct does
  require it (line 1293). The spec command and the spec
  aggregate are inconsistent. The code follows the aggregate
  (academic_id required), not the command spec.

**Expected:**

`docs/specs/cms/commands.md` lines 335-346:
  `pub struct CreateContentCommand { pub tenant: TenantContext,
  pub file_name: String, pub file_size: i64, pub content_type_id:
  ContentTypeId, pub youtube_link: Option<YoutubeLink>,
  pub upload_file: Option<FileReference> }` — no `academic_id`.

**Evidence:**

`docs/specs/cms/aggregates.md` line 343:
  `4. A \`Content\` is anchored to an academic year.`
  `crates/domains/cms/src/aggregate.rs:1293` adds
  `pub academic_id: AcademicYearId` to `NewContent`.
  The `CreateContentCommand` (`crates/domains/cms/src/commands.rs:278-299`)
  does include `academic_id` (line 298), aligning with the
  aggregate but not with `commands.md`.

---

### FINDING 40 (id: `DOMAIN-CMS-040`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/aggregate.rs:843-863

**Description:**

`NewNoticeBoard` does not have an
  `academic_id: AcademicYearId` field. This is consistent with
  the missing `academic_id` on the aggregate (Finding 1).

**Expected:**

Spec aggregates.md line 209: anchor to academic
  year.

**Evidence:**

`crates/domains/cms/src/aggregate.rs:843-863`:
  ```rust
  pub struct NewNoticeBoard {
      pub id: NoticeBoardId,
      pub notice_title: NoticeTitle,
      pub notice_message: NoticeMessage,
      pub notice_date: NoticeDate,
      pub publish_on: Option<PublishDate>,
      pub inform_to: AudienceDescriptor,
      pub created_by: UserId,
      pub created_at: Timestamp,
      pub correlation_id: CorrelationId,
  }
  ```
  No `academic_id` field.

---

### FINDING 41 (id: `DOMAIN-CMS-041`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/aggregate.rs:1821-1876 (TeacherUploadContent)

**Description:**

`TeacherUploadContent::course_id: Option<i32>`
  and `parent_course_id: Option<i32>` use raw integers. Spec
  mandates `chapter_id` / `lesson_id` references to
  `academic_lessons` / `academic_lesson_topic_details`
  (tables.md lines 53-56). The reference type is `i64` per
  spec, but `chapter_id`/`lesson_id` are not typed academic ids
  — they are raw `i64`.

**Expected:**

`docs/specs/cms/tables.md` lines 53-56:
  `chapter_id and lesson_id references; these reference the
  academic domain's lesson and topic aggregates.`

**Evidence:**

`crates/domains/cms/src/aggregate.rs:1804-1805`
  and `1851-1852` use `Option<i64>` for `chapter_id` and
  `lesson_id`. No `LessonId` / `TopicId` types.

---

### FINDING 43 (id: `DOMAIN-CMS-043`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/services.rs (test module line 864-1270)

**Description:**

The services.rs test module has 17 unit tests,
  but 6 of them (`page_service_is_home_page_reflects_aggregate_flag`,
  `page_service_is_published_reflects_status`,
  `news_service_increment_view_returns_new_count`,
  `home_slider_service_ordered_sorts_by_id`,
  `testimonial_service_average_rating_computes_correctly`,
  and others) carry `#[allow(dead_code)]` or weak assertions
  (e.g. `assert!(avg.is_finite() && avg > 0.0)` at line 1056).
  Per AGENTS.md: "Tests like `assert!(true)` or `fn it_works()`
  are rejected."

**Expected:**

AGENTS.md "Testing (TDD)": "No dummy tests.
  Every test must validate a real-world scenario".

**Evidence:**

`crates/domains/cms/src/services.rs:1056`:
  `assert!(avg.is_finite() && avg > 0.0);`. The
  `testimonial_service_average_rating_computes_correctly` test
  asserts a tautology due to the broken implementation (Finding 6).

---

### FINDING 46 (id: `DOMAIN-CMS-046`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/aggregate.rs:1984-2006 (UploadContent)

**Description:**

`UploadContent::content_type: i32` is a raw
  integer FK to `ContentType` taxonomy. The handoff note 154
  (line 154 of PHASE-12-HANDOFF.md) says "raw i32 content_type
  FK to ContentType taxonomy" — but the spec uses
  `ContentTypeId` in `UploadContent` (commands.md lines 441-455)
  is `pub content_type: i32` (raw). The code matches the
  spec; but engine rule "Compile-time safety over strings"
  applies to FKs too — this is a typed-id opportunity missed.

**Expected:**

Spec commands.md line 447:
  `pub content_type: i32, // FK to ContentType`.

**Evidence:**

`crates/domains/cms/src/aggregate.rs:1940` and
  `1973` use `pub content_type: i32`. Per spec; matches.

---

### FINDING 47 (id: `DOMAIN-CMS-047`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/aggregate.rs:2010-2015

**Description:**

`UploadContent::new` validates
  `content_title.as_str().is_empty()` but does not validate the
  `content_type: i32` value (e.g. must be > 0 to be a valid FK).

**Expected:**

AGENTS.md "No `unwrap`/`expect`/`panic`" +
  spec invariants of `UploadContent` (line 476-479).

**Evidence:**

`crates/domains/cms/src/aggregate.rs:2010-2038`
  — `UploadContent::new` checks title empty but not `content_type`.

---

### FINDING 50 (id: `DOMAIN-CMS-050`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/aggregate.rs:1174-1265 (SpeechSlider)

**Description:**

The spec describes `SpeechSlider` as
  CMS-side, distinct from the communication domain's
  `SpeechSlider`. The code does not differentiate them — both
  crate's `SpeechSlider` types would have identical
  `AuditTarget::SpeechSlider(Uuid)` variants. Per Phase 12
  handoff Open Question #3 ("SpeechSlider dual ownership"),
  this is acknowledged as unresolved.

**Expected:**

Spec aggregates.md lines 295-323 (CMS-side
  SpeechSlider). Spec handoff PHASE-12-HANDOFF.md:373-381
  carries OQ #3.

**Evidence:**

`crates/cross-cutting/audit/src/writer.rs:293`
  defines `SpeechSlider(Uuid)` shared between CMS and
  Communication.

---

### FINDING 51 (id: `DOMAIN-CMS-051`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/services.rs:687-694

**Description:**

`ResolvedAudience` (struct in `services.rs`)
  holds `Vec<uuid::Uuid>` for `roles` / `users` instead of
  typed `Vec<RoleId>` / `Vec<UserId>`. Spec value-objects.md
  line 110: `RoleId | From educore-rbac`. Spec services.md
  line 90: `pub fn resolve_audience(list: &ContentShareList) ->
  Vec<UserId>`.

**Expected:**

`docs/specs/cms/services.md` line 90:
  `pub fn resolve_audience(list: &ContentShareList) -> Vec<UserId> { ... }`.

**Evidence:**

`crates/domains/cms/src/services.rs:687-694`:
  ```rust
  pub struct ResolvedAudience {
      pub roles: Vec<uuid::Uuid>,
      pub users: Vec<uuid::Uuid>,
      pub class_section: Option<(educore_academic::ClassId, Vec<educore_academic::SectionId>)>,
  }
  ```
  `roles` and `users` are `Vec<Uuid>`, not typed ids.

---

### FINDING 52 (id: `DOMAIN-CMS-052`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/aggregate.rs:1657-1677 (ContentShareList)

**Description:**

`ContentShareList.gr_role_ids: Option<Vec<Uuid>>`
  and `ind_user_ids: Option<Vec<Uuid>>` use raw UUIDs. Spec
  services.md line 90 mandates `Vec<RoleId>` /
  `Vec<UserId>`. Spec value-objects.md line 109:
  `RoleId | From educore-rbac`.

**Expected:**

Spec services.md line 90:
  `pub fn resolve_audience(list: &ContentShareList) -> Vec<UserId>`.

**Evidence:**

`crates/domains/cms/src/aggregate.rs:1647-1649`:
  ```rust
  pub gr_role_ids: Option<Vec<Uuid>>,
  pub ind_user_ids: Option<Vec<Uuid>>,
  ```
  Use raw `Uuid`, not `RoleId` / `UserId`.

---

### FINDING 53 (id: `DOMAIN-CMS-053`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/aggregate.rs:1984-2006 (UploadContent fields)

**Description:**

`UploadContent::available_for_role`,
  `available_for_class`, `available_for_section` use
  `Option<i32>` raw integers. Spec aggregates.md line 478:
  `available_for_class` and `available_for_section` should be
  typed `ClassId` / `SectionId` per the engine rule.

**Expected:**

Spec aggregates.md invariant 13 (lines 102-104).

**Evidence:**

`crates/domains/cms/src/aggregate.rs:1975-1979`:
  ```rust
  pub available_for_role: Option<i32>,
  pub available_for_class: Option<i32>,
  pub available_for_section: Option<i32>,
  ```

---

### FINDING 54 (id: `DOMAIN-CMS-054`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/services.rs (test mod)

**Description:**

The `services.rs` test module has 17
  `#[test]` items but the handoff claims 183 unit tests total
  (verified by `grep -cE "    #\[test\]"` per file: 40+10+6+8+2
  +17+69+0+10+21 = 183 ✓). The 183 claim is verified.

**Expected:**

AGENTS.md: "At least one integration test per
  PR".

**Evidence:**

`grep -cE "    #\[test\]" crates/domains/cms/src/*`
  yields: 40, 10, 6, 8, 2, 17, 69, 0, 10, 21 → 183 ✓.
  The storage-parity `cms_integration.rs` has 7 + 2 = 9 scenarios
  (`cms_integration_sqlite_vertical_slice`,
  `cms_capability_check_gates_page_publish`,
  `cms_event_type_round_trip_for_all_aggregates`,
  `cms_slug_uniqueness_invariant`,
  `cms_content_share_list_window_invariant`,
  `cms_form_uploaded_public_indexing_subscriber_indexes_when_show_public`,
  `cms_form_uploaded_public_indexing_subscriber_ignores_when_not_public`,
  plus 2 `#[ignore]` PG/MySQL variants).

---

### FINDING 56 (id: `DOMAIN-CMS-056`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/services.rs (entire services.rs file)

**Description:**

None of the workflows in
  `docs/specs/cms/workflows.md` (189 lines, 9 workflows) are
  implemented as orchestrated flows. The workflows spec calls
  for ordered sequences of commands, queries, and policies.
  Per spec workflow 4 "Testimonial Curation Workflow" (lines
  62-70), the `TestimonialService::average_rating` is invoked
  on the curated list — but `average_rating` is broken
  (Finding 6).

**Expected:**

`docs/specs/cms/workflows.md` (189 lines).

**Evidence:**

`crates/domains/cms/src/services.rs` has only
  pure helpers + service factories; no workflow orchestration.

---

### FINDING 57 (id: `DOMAIN-CMS-057`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/aggregate.rs:1609 (NewContentShareList)

**Description:**

Spec commands.md line 360-378 lists
  `CreateContentShareListCommand` with `class_id: Option<ClassId>`,
  `section_ids: Option<Vec<SectionId>>`. Code command
  `CreateContentShareListCommand` (commands.rs:331-358) uses
  `Option<ClassId>` and `Option<Vec<SectionId>>` ✓.

  However the `NewContentShareList` aggregate input
  (`aggregate.rs:1592-1625`) has `class_id: Option<ClassId>`,
  `section_ids: Option<Vec<SectionId>>` ✓.

  But the `ContentShareList` aggregate (line 1629-1678) has
  `class_id: Option<ClassId>` (line 1651) and
  `section_ids: Option<Vec<SectionId>>` (line 1653) — both
  are correct.

  Verified consistent.

**Expected:**

Spec value-objects.md line 110-114:
  `ClassId | From educore-academic`.

**Evidence:**

`crates/domains/cms/src/aggregate.rs:1612, 1614,
  1651, 1653` — typed ids correctly used.

---

### FINDING 62 (id: `DOMAIN-CMS-062`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/repository.rs:367-393 (UploadContentRepository)

**Description:**

`UploadContentRepository::list_for_class`
  takes `class: i32, section: Option<i32>` instead of typed
  `ClassId, Option<SectionId>`. Per spec repositories.md
  lines 197-209, `list_for_class(&self, school: SchoolId,
  class: ClassId, section: Option<SectionId>)`.

**Expected:**

Spec repositories.md lines 197-209:
  `async fn list_for_class(&self, school: SchoolId, class:
  ClassId, section: Option<SectionId>) -> Result<Vec<UploadContent>>`.

**Evidence:**

`crates/domains/cms/src/repository.rs:380-386`:
  ```rust
  async fn list_for_class(
      &self,
      school: SchoolId,
      class: i32,
      section: Option<i32>,
  ) -> StorageResult<Vec<UploadContent>>;
  ```

---

### FINDING 65 (id: `DOMAIN-CMS-065`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/cross-cutting/audit/src/writer.rs:307-345

**Description:**

The handoff claims "20 net-new CMS audit
  targets + 1 retained Page placeholder = 21 total Cms-domain
  audit targets." Counting actual CMS variants in writer.rs:
  Page (1), News (2), NewsCategory (3), NewsComment (4),
  NewsPage (5), NoticeBoard (6), Testimonial (7), HomeSlider (8),
  SpeechSlider (9, shared with Communication), Content (10),
  ContentType (11), ContentShareList (12), TeacherUploadContent
  (13), UploadContent (14), AboutPage (15), ContactPage (16),
  CoursePage (17), HomePageSetting (18), FrontendPage (19),
  PageRevision (20), NewsRevision (21). ContentRevision
  (claimed in the handoff list at line 281) is missing.

**Expected:**

Handoff PHASE-12-HANDOFF.md lines 273-286
  lists 21 audit targets including `ContentRevision`.

**Evidence:**

`grep -nE "ContentRevision" crates/cross-cutting/audit/src/writer.rs`
  → 0 matches. The handoff claim of "21 total" is one short
  of the listed names because `ContentRevision` was not added.

---

### FINDING 67 (id: `DOMAIN-CMS-067`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/aggregate.rs (entire file)

**Description:**

Per the handoff "Phase 12 closed" but the
  `delete_*_service` factory functions are only present for
  `Page` (services.rs:268-313). The remaining 18 aggregates
  lack `delete_*_service` factories despite the spec listing
  `Delete*` commands for each. The handoff acknowledges this
  ("Per-aggregate CRUD factories ship in follow-up phases
  alongside the `#[derive(DomainQuery)]` macro emissions") but
  the wire contract for the missing commands is unspecified.

**Expected:**

`docs/commands/cms.md` lines 12-76 — every
  aggregate has a Delete row.

**Evidence:**

`grep -nE "^pub async fn delete_" crates/domains/cms/src/services.rs`
  → 1 match (`delete_page_service`). Per handoff OQ #4
  (`PHASE-12-HANDOFF.md:382-390`), the missing CRUD factories
  are deferred; the question of how the engine wires the
  deferral is unresolved.

---

### FINDING 23 (id: `DOMAIN-CMS-023`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/aggregate.rs:756-795

**Description:**

`NewsPage` lacks a `soft_delete` re-validation
  of the spec's `find_active` uniqueness invariant
  (`docs/specs/cms/aggregates.md` invariant 2: `At most one NewsPage
  per school may be active`). Soft-delete is allowed without
  enforcing the at-most-one invariant on insert or update.

**Expected:**

`docs/specs/cms/aggregates.md` lines 177-178:
  `2. A \`NewsPage\` is anchored to a school. 3. At most one NewsPage
  per school may be active.`

**Evidence:**

`crates/domains/cms/src/aggregate.rs:797-836` —
  `NewsPage::new` has no `find_active` check. The repository's
  `find_active` is the sole enforcement gate, but no
  `NewsPageService` exists to wire it.

---

### FINDING 24 (id: `DOMAIN-CMS-024`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/services.rs:748-816

**Description:**

`configure_home_page_service` short-circuits
  when a setting exists: it returns the existing setting **without
  applying the new fields from the `ConfigureHomePageCommand`**.
  The handoff says it returns "as-is" and emits `HomePageSettingUpdated`
  with a hard-coded `vec!["title".to_owned()]` change set
  (line 783-786), not the actual diff.

**Expected:**

`docs/specs/cms/commands.md` line 317-334:
  `## ConfigureHomePage ... Effects: Emits HomePageSettingCreated or HomePageSettingUpdated depending on whether the school already has a setting.`
  The service should apply the update.

**Evidence:**

`crates/domains/cms/src/services.rs:765-792`:
  ```rust
  if let Some(p) = existing {
      let after = snapshot(&p);
      audit.write(...).await...?;
      let event = HomePageSettingUpdated::new(
          &p,
          vec!["title".to_owned()],
          tenant.correlation_id,
          Timestamp::now(),
      );
      bus.publish(event.into_envelope(&tenant)).await.map_err(CmsError::from)?;
      return Ok(p);
  }
  ```
  The `cmd` parameter (with new fields) is dropped without
  applying it.

---

### FINDING 32 (id: `DOMAIN-CMS-032`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/events.rs:3711-3719

**Description:**

The file `events.rs` has two dead-code helper
  functions `c_school_id` and `c_school_id_of` at lines 3714-3719,
  each returning `SchoolId(Uuid::nil())`. These are not called by
  any code in the file.

**Expected:**

AGENTS.md "No `#[allow(dead_code)]` or `_var`
  prefixes to silence the compiler. Delete unused code."

**Evidence:**

`crates/domains/cms/src/events.rs:3714-3719`:
  ```rust
  fn c_school_id() -> educore_core::ids::SchoolId {
      educore_core::ids::SchoolId(uuid::Uuid::nil())
  }
  fn c_school_id_of(_: educore_core::ids::SchoolId) -> educore_core::ids::SchoolId {
      c_school_id()
  }
  ```

---

### FINDING 35 (id: `DOMAIN-CMS-035`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/aggregate.rs:719-722

**Description:**

`NewsComment` struct does not carry the
  17-field audit-footer (no `version`, `etag`,
  `last_event_id`, `correlation_id`, `updated_at`,
  `updated_by`). Spec mandates the audit-footer pattern via
  AGENTS.md.

**Expected:**

AGENTS.md "Module Layout" + "Audit-first":
  every aggregate has `version`, `etag`, `created_at`,
  `updated_at`, `created_by`, `updated_by`, `active_status`,
  `last_event_id`, `correlation_id`.

**Evidence:**

`crates/domains/cms/src/aggregate.rs:670-687` —
  ```rust
  pub struct NewsComment {
      pub id: NewsCommentId,
      pub school_id: SchoolId,
      pub news_id: NewsId,
      pub user_id: UserId,
      pub parent_id: Option<NewsCommentId>,
      pub message: CommentMessage,
      pub status: NewsCommentStatus,
      pub created_at: Timestamp,
  }
  ```
  8 fields; missing `version`, `etag`, `updated_at`, `updated_by`,
  `last_event_id`, `correlation_id`, `active_status`.

---

### FINDING 36 (id: `DOMAIN-CMS-036`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/services.rs (test mod line 893)

**Description:**

`services.rs` has 17 `#[test]` items but only
  11 are not anchored to a `#[cfg(test)]` module (the test
  count claim in the handoff is for unit tests). AGENTS.md and
  the handoff claim 17 unit tests in services.rs which matches.
  However, the test for `form_uploaded_public_indexing_subscriber`
  at line 1183-1213 uses the synchronous `assert_eq!` against
  the `FormIndexAction::Index` enum, which is fine but
  indicates the subscriber is a synchronous pure function (not
  wired to the bus).

**Expected:**

Spec for `form_uploaded_public_indexing_subscriber`
  per Phase 11 OQ #6 — wiring expected.

**Evidence:**

`crates/domains/cms/src/services.rs:835-859`
  (sync fn) vs `crates/domains/cms/src/services.rs:1183-1213`
  (test). The `bus` and `repo` are not invoked by the subscriber.

---

### FINDING 39 (id: `DOMAIN-CMS-039`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/query.rs:12-13

**Description:**

`query.rs` declares `#![allow(missing_docs)]`
  at module level. The query stub types do carry doc comments
  (e.g. `PageQuery` at line 27, `NewsQuery` at line 93), but
  the methods `with_title`, `with_slug`, `with_status`,
  `with_home_page`, `with_is_default`, `with_active` on
  `PageQuery` (lines 52-85) have doc comments. Verified
  consistent; the blanket allow remains an anti-pattern.

**Expected:**

AGENTS.md: `#![deny(missing_docs)]`.

**Evidence:**

`crates/domains/cms/src/query.rs:12-13`:
  ```rust
  #![allow(dead_code, clippy::all)]
  #![allow(missing_docs)]
  ```

---

### FINDING 42 (id: `DOMAIN-CMS-042`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/aggregate.rs:1086-1101 (NewHomeSlider)

**Description:**

Spec commands.md `CreateHomeSliderCommand`
  has fields `tenant`, `image`, `link`, plus the aggregate has
  `link_label`. Code `CreateHomeSliderCommand` matches spec
  but adds `link_label: Option<HomeSliderLinkLabel>`
  (`commands.rs:251`) which is not in the spec struct. Minor
  extension.

**Expected:**

`docs/specs/cms/commands.md` lines 278-289:
  `pub struct CreateHomeSliderCommand { pub tenant: TenantContext,
  pub image: FileReference, pub link: Option<Url> }` — no
  `link_label`.

**Evidence:**

`crates/domains/cms/src/commands.rs:243-252` adds
  `pub link_label: Option<HomeSliderLinkLabel>` which is not in
  the spec. The aggregate (`aggregate.rs:1086-1101`) does carry
  `link_label`, so the command is needed but the spec command
  shape is missing it.

---

### FINDING 48 (id: `DOMAIN-CMS-048`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/aggregate.rs (page.rs 96)

**Description:**

The audit-footer fields are inconsistent
  across aggregates. `NewsComment` has only 8 fields (Finding 35);
  `Page` has 17 (the full footer). Per AGENTS.md "Module Layout"
  the footer is mandatory, so this is an inconsistency.

**Expected:**

AGENTS.md "Module Layout".

**Evidence:**

`crates/domains/cms/src/aggregate.rs:92-129`
  (Page, 17 fields) vs `crates/domains/cms/src/aggregate.rs:670-687`
  (NewsComment, 8 fields).

---

### FINDING 49 (id: `DOMAIN-CMS-049`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/services.rs (entire service factories)

**Description:**

None of the service-factory functions handle
  the spec's idempotency requirement. `ConfigureHomePage` is
  marked "yes" (idempotent) in `docs/commands/cms.md:38` and
  the spec says `CreateHomeSlider` is **not** idempotent
  (`docs/commands/cms.md:35`). No idempotency key is checked.

**Expected:**

`docs/commands/cms.md` lines 12-76 with
  Idempotent? column.

**Evidence:**

`crates/domains/cms/src/services.rs:127-816`
  — no idempotency-key plumbing visible.

---

### FINDING 55 (id: `DOMAIN-CMS-055`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** crates/domains/cms/src/aggregate.rs:276-281 (PageStatusAction enum)

**Description:**

`PageStatusAction` enum has variants
  `Publish` and `Archive` but no `Create` / `Delete` actions.
  Spec workflow `docs/specs/cms/workflows.md` lines 9-17
  includes `Create`, `Publish`, `Archive`, `Delete`. The code
  covers publish + archive but `Create` and `Delete` are
  constructor methods (Page::new, Page::soft_delete) — not
  state-machine actions. This is a stylistic deviation but
  not a functional defect.

**Expected:**

`docs/specs/cms/workflows.md` lines 9-17.

**Evidence:**

`crates/domains/cms/src/aggregate.rs:274-281`:
  ```rust
  pub enum PageStatusAction {
      Publish,
      Archive,
  }
  ```

---

### FINDING 66 (id: `DOMAIN-CMS-066`)

- **Source:** `docs/audit_reports/findings/wave1-cms.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** docs/specs/cms/commands.md (entire file)

**Description:**

Spec `docs/specs/cms/commands.md` does not
  list `IncrementNewsView` (which is listed in aggregates.md
  line 89 as a News command), nor does it list
  `CreateHomePageSetting` / `UpdateHomePageSetting` /
  `DeleteHomePageSetting` (the spec uses `ConfigureHomePage`
  as create-or-update). Aggregates.md and commands.md are not
  in sync.

**Expected:**

`docs/specs/cms/commands.md` lines 1-579 vs
  `docs/specs/cms/aggregates.md` lines 82-89.

**Evidence:**

`grep -nE "IncrementNewsView" docs/specs/cms/commands.md`
  → no match. `grep -nE "CreateHomePageSetting|UpdateHomePageSetting|
  DeleteHomePageSetting" docs/specs/cms/commands.md` → no match.

---


## Communication (target id prefix: `DOMAIN-COM`)

**Path:** `crates/domains/communication/`  
**Total findings:** 47 (28 critical, 13 high, 5 medium, 1 low)


### FINDING 1 (id: `DOMAIN-COM-001`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/aggregate.rs:2068` and `docs/specs/communication/aggregates.md:663`

**Description:**

The aggregate root for the chat-status presence record is named `ChatStatusRecord` in the Rust source but the spec calls the root type `ChatStatus`. The spec, prelude, and `ChatStatusRepository` (`repository.rs:578`) all reference the type by different names (`ChatStatusRecord` aggregate vs. `ChatStatus` enum vs. `ChatStatus` repository parameter). Consumers that consult only the spec will be unable to import the symbol the code exports.

**Expected:**

`docs/specs/communication/aggregates.md:663` `**Root type:** \`ChatStatus\`` — the Rust aggregate must be `pub struct ChatStatus`.

**Evidence:**

- `crates/domains/communication/src/aggregate.rs:2068` `pub struct ChatStatusRecord {`
  - `crates/domains/communication/src/repository.rs:578` `pub trait ChatStatusRepository: Send + Sync { async fn insert(&self, s: &ChatStatus) -> Result<()>; }` — uses `ChatStatus` as the aggregate reference.
  - `crates/domains/communication/src/value_objects.rs:801` `pub enum ChatStatus { ... }` — a separate status enum clashes with the aggregate name.

---

### FINDING 11 (id: `DOMAIN-COM-011`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/services.rs:2164-2172` and `docs/specs/communication/services.md:11-12`

**Description:**

`NotificationService::select_template` has a different signature from the spec. Spec: `pub fn select_template(event: &str, destination: Destination) -> Option<SmsTemplateId>`. Code: `pub fn select_template<'a>(event: &str, channel: Channel, candidates: &'a [SmsTemplate]) -> Option<&'a SmsTemplate>`.

**Expected:**

The spec signature `(event, destination) -> Option<SmsTemplateId>`.

**Evidence:**

- `crates/domains/communication/src/services.rs:2164-2172`
  - `docs/specs/communication/services.md:12` `pub fn select_template(event: &str, destination: Destination) -> Option<SmsTemplateId> { ... }`

---

### FINDING 12 (id: `DOMAIN-COM-012`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/services.rs:2186-2188` and `docs/specs/communication/services.md:14`

**Description:**

`NotificationService::route` has a different signature from the spec. Spec: `pub fn route(setting: &NotificationSetting, recipient: &AudienceDescriptor) -> Vec<(UserId, Channel)>`. Code: `pub fn route(setting: &NotificationSetting) -> Destination`. The code merely returns `setting.destination` and discards the recipient filter.

**Expected:**

`(setting, recipient) -> Vec<(UserId, Channel)>` per spec.

**Evidence:**

- `crates/domains/communication/src/services.rs:2186-2188` `pub fn route(setting: &NotificationSetting) -> Destination { setting.destination }`
  - `docs/specs/communication/services.md:14` `pub fn route(setting: &NotificationSetting, recipient: &AudienceDescriptor) -> Vec<(UserId, Channel)> { ... }`

---

### FINDING 13 (id: `DOMAIN-COM-013`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/services.rs:2195-2197` and `docs/specs/communication/services.md:15`

**Description:**

`NotificationService::next_window` has a different signature from the spec. Spec: `pub fn next_window(now: NaiveTime, window: &TimeWindow) -> Option<NaiveTime>`. Code: `pub fn next_window(setup: &AbsentNotificationTimeSetup) -> (TimeOfDay, TimeOfDay)`. The signature and return type are entirely different.

**Expected:**

`(now: NaiveTime, window: &TimeWindow) -> Option<NaiveTime>` per spec.

**Evidence:**

- `crates/domains/communication/src/services.rs:2195-2197`
  - `docs/specs/communication/services.md:15` `pub fn next_window(now: NaiveTime, window: &TimeWindow) -> Option<NaiveTime> { ... }`

---

### FINDING 14 (id: `DOMAIN-COM-014`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/services.rs:2267-2272` and `docs/specs/communication/services.md:53`

**Description:**

`ComplaintService::categorize` has a different signature from the spec. Spec: `pub fn categorize(cmd: &RegisterComplaintCommand) -> ComplaintTypeId`. Code: `pub fn categorize(complaint: &Complaint, types: &[ComplaintType]) -> String`. Different parameter shape and different return type.

**Expected:**

`(cmd: &RegisterComplaintCommand) -> ComplaintTypeId` per spec.

**Evidence:**

- `crates/domains/communication/src/services.rs:2267-2272`
  - `docs/specs/communication/services.md:53` `pub fn categorize(cmd: &RegisterComplaintCommand) -> ComplaintTypeId { ... }`

---

### FINDING 15 (id: `DOMAIN-COM-015`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/services.rs:2277-2279` and `docs/specs/communication/services.md:54`

**Description:**

`ComplaintService::is_anonymous` has a different signature from the spec. Spec: `pub fn is_anonymous(source: ComplaintSource, by: Option<&PersonName>) -> bool`. Code: `pub fn is_anonymous(complaint: &Complaint) -> bool`. The spec parameters are source + name; the code passes the whole aggregate.

**Expected:**

`(source: ComplaintSource, by: Option<&PersonName>) -> bool` per spec.

**Evidence:**

- `crates/domains/communication/src/services.rs:2277-2279`
  - `docs/specs/communication/services.md:54` `pub fn is_anonymous(source: ComplaintSource, by: Option<&PersonName>) -> bool { ... }`

---

### FINDING 16 (id: `DOMAIN-COM-016`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/services.rs:2302-2314` and `docs/specs/communication/services.md:56`

**Description:**

`ComplaintService::escalation_path` has a different signature and return type from the spec. Spec: `pub fn escalation_path(setting: &NotificationSetting, complaint_type: ComplaintTypeId) -> Vec<UserId>`. Code: `pub fn escalation_path(current: ComplaintStatus) -> Vec<ComplaintStatus>`. The spec routes a setting + type to a user list; the code returns a status path.

**Expected:**

`(setting: &NotificationSetting, complaint_type: ComplaintTypeId) -> Vec<UserId>` per spec.

**Evidence:**

- `crates/domains/communication/src/services.rs:2302-2314`
  - `docs/specs/communication/services.md:56` `pub fn escalation_path(setting: &NotificationSetting, complaint_type: ComplaintTypeId) -> Vec<UserId> { ... }`

---

### FINDING 17 (id: `DOMAIN-COM-017`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/services.rs:2220-2228` and `docs/specs/communication/services.md:35`

**Description:**

`ChatService::resolve_conversation` has a different signature and return type from the spec. Spec: `pub fn resolve_conversation(from: UserId, to: UserId, existing: &[ChatConversation]) -> Option<ChatConversationId>`. Code: `pub fn resolve_conversation(a: UserId, b: UserId, conversations: &[ChatConversation]) -> Option<&ChatConversation>`. Returns a reference (not an owned id), forcing a lifetime-bound consumer.

**Expected:**

`(from: UserId, to: UserId, existing: &[ChatConversation]) -> Option<ChatConversationId>` per spec.

**Evidence:**

- `crates/domains/communication/src/services.rs:2220-2228`
  - `docs/specs/communication/services.md:35` `pub fn resolve_conversation(from: UserId, to: UserId, existing: &[ChatConversation]) -> Option<ChatConversationId> { ... }`

---

### FINDING 19 (id: `DOMAIN-COM-019`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/services.rs:2210` and `docs/specs/communication/services.md:34`

**Description:**

`ChatService::is_blocked` has a different signature from the spec. Spec: `pub fn is_blocked(block_list: &[ChatBlockUser], between: (UserId, UserId)) -> bool`. Code: `pub fn is_blocked(from: UserId, blocks: &[ChatBlockUser]) -> bool`. The spec checks a `(from, to)` pair; the code only checks whether `from` has placed any block. The recipient-side block and the cross-block ("either side has blocked the other") are not detected.

**Expected:**

`(block_list: &[ChatBlockUser], between: (UserId, UserId)) -> bool` per spec.

**Evidence:**

- `crates/domains/communication/src/services.rs:2212-2214` `blocks.iter().any(|b| b.block_by == from && b.is_active())`
  - `docs/specs/communication/services.md:34` `pub fn is_blocked(block_list: &[ChatBlockUser], between: (UserId, UserId)) -> bool { ... }`
  - `docs/specs/communication/services.md:41-42` `ChatService::is_blocked returns true when either side has blocked the other, in which case the message is suppressed.`

---

### FINDING 2 (id: `DOMAIN-COM-002`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/entities.rs:750` and `crates/domains/communication/src/value_objects.rs:2144`

**Description:**

`NotificationSettingAudience` is defined twice with conflicting shapes. `entities.rs:750` declares it as a 4-variant enum `Roles | ClassSection | Users | All`, while `value_objects.rs:2144` declares it as `pub type NotificationSettingAudience = AudienceDescriptor;`. The aggregate (`aggregate.rs:1047`) imports the entities version under an alias; consumers importing from the prelude resolve to the `value_objects` alias. Two types with the same name but different memory layouts exist in one crate.

**Expected:**

A single definition (either the enum from entities.rs or the alias from value_objects.rs) used everywhere.

**Evidence:**

- `crates/domains/communication/src/entities.rs:749-765` `pub enum NotificationSettingAudience { Roles(Vec<RoleId>), ClassSection { ... }, Users(Vec<UserId>), All }`
  - `crates/domains/communication/src/value_objects.rs:2138-2144` `/// A type alias for the audience descriptor of a NotificationSetting... pub type NotificationSettingAudience = AudienceDescriptor;`
  - `crates/domains/communication/src/aggregate.rs:31-32` `use crate::entities::{ CustomSmsSettingParam as EntitiesCustomSmsSettingParam, NotificationSettingAudience as EntitiesNotificationSettingAudience, };`

---

### FINDING 20 (id: `DOMAIN-COM-020`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/commands.rs:373-390` and `docs/specs/communication/commands.md:99-109`

**Description:**

`RegisterComplaintCommand` field type drift: spec uses `complaint_by: Option<PersonName>`; code uses `complaint_by: Option<UserId>`. The spec's `PersonName` cannot identify a system user; the code's `UserId` cannot capture a free-text anonymous complainant's display name.

**Expected:**

`pub complaint_by: Option<PersonName>` per spec.

**Evidence:**

- `crates/domains/communication/src/commands.rs:377` `pub complaint_by: Option<UserId>,`
  - `docs/specs/communication/commands.md:99-108` `pub struct RegisterComplaintCommand { pub tenant: TenantContext, pub complaint_by: Option<PersonName>, ... }`

---

### FINDING 21 (id: `DOMAIN-COM-021`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/commands.rs:523-538` and `docs/specs/communication/commands.md:178-186`

**Description:**

`SendNotificationCommand` field type drift: spec uses `pub message: String`; code uses `pub message: NotificationMessage`. The spec treats the notification body as a free-form string; the code wraps it in a typed VO with separate validation rules.

**Expected:**

`pub message: String` per spec.

**Evidence:**

- `crates/domains/communication/src/commands.rs:531` `pub message: NotificationMessage,`
  - `docs/specs/communication/commands.md:178-186` `pub struct SendNotificationCommand { pub tenant: TenantContext, pub recipient_user_id: UserId, pub notification_type: NotificationType, pub message: String, ... }`

---

### FINDING 22 (id: `DOMAIN-COM-022`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/commands.rs:637-652` and `docs/specs/communication/commands.md:258-266`

**Description:**

`CreateSmsTemplateCommand` field type drift on two fields. Spec: `purpose: TemplateKey, subject: String`. Code: `purpose: String, subject: EmailSubject`. The spec validates purpose via `TemplateKey` (1..100 chars); the code accepts any `String`.

**Expected:**

`pub purpose: TemplateKey, pub subject: String` per spec.

**Evidence:**

- `crates/domains/communication/src/commands.rs:643-645` `pub purpose: String, pub subject: EmailSubject,`
  - `docs/specs/communication/commands.md:258-266` `pub struct CreateSmsTemplateCommand { ... pub purpose: TemplateKey, pub subject: String, ... }`

---

### FINDING 23 (id: `DOMAIN-COM-023`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/commands.rs:837-856` and `docs/specs/communication/commands.md:660-670`

**Description:**

`CreateCustomSmsSettingCommand` has three field-type drifts versus the spec. Spec: `gateway_name: String, set_auth: Option<SecretReference>, params: Vec<(String, String)>`. Code: `gateway_name: GatewayName, set_auth: Option<bool>, params: Vec<CustomSmsSettingParam>`. The spec encodes credentials via `SecretReference`; the code encodes it as a `bool`. The spec encodes params as raw tuples; the code wraps them in the conflicting duplicate struct (Finding DOMAIN-COM-004).

**Expected:**

Per spec: `gateway_name: String, set_auth: Option<SecretReference>, params: Vec<(String, String)>`.

**Evidence:**

- `crates/domains/communication/src/commands.rs:843-855`
  - `docs/specs/communication/commands.md:660-670` `pub struct CreateCustomSmsSettingCommand { ... pub gateway_name: String, pub set_auth: Option<SecretReference>, pub params: Vec<(String, String)>, }`

---

### FINDING 24 (id: `DOMAIN-COM-024`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/commands.rs:907-922` and `docs/specs/communication/commands.md:351-359`

**Description:**

`CreateNotificationSettingCommand` field type drift on three fields. Spec: `recipient: AudienceDescriptor, subject: String, shortcode: Vec<TemplateVariable>`. Code: `recipient: NotificationSettingAudience, subject: EmailSubject, shortcode: String`. The spec's `shortcode` is a list of template variables; the code stores a single string.

**Expected:**

Per spec.

**Evidence:**

- `crates/domains/communication/src/commands.rs:915-921`
  - `docs/specs/communication/commands.md:351-359` `pub struct CreateNotificationSettingCommand { ... pub recipient: AudienceDescriptor, pub subject: String, pub shortcode: Vec<TemplateVariable>, }`

---

### FINDING 25 (id: `DOMAIN-COM-025`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/commands.rs:301-321` and `docs/specs/communication/commands.md:33-41`

**Description:**

`UpdateNoticeCommand` field type drift on `publish_on`. Spec: `publish_on: Option<PublishOn>` (the typed wrapper VO). Code: `publish_on: Option<NaiveDate>` (raw `NaiveDate` with no clear/keep semantics).

**Expected:**

`pub publish_on: Option<PublishOn>` per spec.

**Evidence:**

- `crates/domains/communication/src/commands.rs:306` `pub publish_on: Option<NaiveDate>,`
  - `docs/specs/communication/commands.md:33-41` `pub struct UpdateNoticeCommand { ... pub publish_on: Option<PublishOn>, ... }`

---

### FINDING 27 (id: `DOMAIN-COM-027`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/events.rs:116-122` and `docs/specs/communication/events.md:46`

**Description:**

`NoticeUpdated.changes` type drift: spec `pub changes: Vec<&'static str>`, code `pub changes: Vec<String>`. The spec keeps the change-list as a static string slice; the code forces a heap allocation per change.

**Expected:**

`pub changes: Vec<&'static str>` per spec.

**Evidence:**

- `crates/domains/communication/src/events.rs:118` `pub changes: Vec<String>,`
  - `docs/specs/communication/events.md:46` `pub struct NoticeUpdated { pub notice_id: NoticeId, pub changes: Vec<&'static str> }`

---

### FINDING 28 (id: `DOMAIN-COM-028`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/events.rs:1009-1023` and `docs/specs/communication/events.md:120`

**Description:**

`SmsTemplateUpdated.changes` type drift: spec `Vec<&'static str>`, code `Vec<String>`.

**Expected:**

`Vec<&'static str>` per spec.

**Evidence:**

- `crates/domains/communication/src/events.rs:1011` `pub changes: Vec<String>,`
  - `docs/specs/communication/events.md:120` `pub struct SmsTemplateUpdated { pub sms_template_id: SmsTemplateId, pub changes: Vec<&'static str> }`

---

### FINDING 29 (id: `DOMAIN-COM-029`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/events.rs:3261-3268` and `docs/specs/communication/events.md:302`

**Description:**

`SpeechSliderCreated.name` type drift: spec `name: PersonName`, code `name: String`. The spec wraps the leader's name in a 1..=200 char validated VO; the code stores it as a raw string.

**Expected:**

`pub name: PersonName` per spec.

**Evidence:**

- `crates/domains/communication/src/events.rs:3263` `pub name: String,`
  - `docs/specs/communication/events.md:302` `pub struct SpeechSliderCreated { pub speech_slider_id: SpeechSliderId, pub name: PersonName, pub designation: String }`

---

### FINDING 3 (id: `DOMAIN-COM-003`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/entities.rs:724` and `crates/domains/communication/src/value_objects.rs:2116`

**Description:**

`SmsTemplateVariable` is defined twice with conflicting shapes. `entities.rs:724` is a struct `{ name: String, description: String }`. `value_objects.rs:2116` is a wrapper `pub struct SmsTemplateVariable(pub Vec<TemplateVariable>)`. The two types have different memory layouts under the same name.

**Expected:**

A single definition. Per `docs/specs/communication/entities.md:40-46` (`SmsTemplateVariable` is a list of `(name, description)` pairs), the value-objects wrapper is closest, but the entities.rs struct is what is imported by the aggregate.

**Evidence:**

- `crates/domains/communication/src/entities.rs:723-730` `pub struct SmsTemplateVariable { pub name: String, pub description: String, }`
  - `crates/domains/communication/src/value_objects.rs:2114-2116` `pub struct SmsTemplateVariable(pub Vec<TemplateVariable>);`

---

### FINDING 30 (id: `DOMAIN-COM-030`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/events.rs:3309-3316` and `docs/specs/communication/events.md:303`

**Description:**

`SpeechSliderUpdated.changes` type drift: spec `Vec<&'static str>`, code `Vec<String>`.

**Expected:**

`Vec<&'static str>` per spec.

**Evidence:**

- `crates/domains/communication/src/events.rs:3312` `pub changes: Vec<String>,`
  - `docs/specs/communication/events.md:303` `pub struct SpeechSliderUpdated { pub speech_slider_id: SpeechSliderId, pub changes: Vec<&'static str> }`

---

### FINDING 31 (id: `DOMAIN-COM-031`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/events.rs:3107-3116` and `docs/specs/communication/events.md:287-293`

**Description:**

`ContactMessageReceived` field type drift: spec declares `email: Option<EmailAddress>` and `phone: Option<PhoneNumber>`; code declares them as non-optional `email: EmailAddress` and `phone: PhoneNumber`. The spec allows anonymous contact-form submissions (no email, no phone); the code rejects them at compile-time.

**Expected:**

`pub email: Option<EmailAddress>, pub phone: Option<PhoneNumber>` per spec.

**Evidence:**

- `crates/domains/communication/src/events.rs:3110-3111` `pub email: EmailAddress, pub phone: PhoneNumber,`
  - `docs/specs/communication/events.md:287-293` `pub struct ContactMessageReceived { pub contact_message_id: ContactMessageId, pub name: PersonName, pub email: Option<EmailAddress>, pub phone: Option<PhoneNumber>, pub subject: String, }`

---

### FINDING 33 (id: `DOMAIN-COM-033`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `docs/commands/communication.md:13-69` and `docs/specs/communication/aggregates.md:111-113,653,741,447-448`

**Description:**

The commands catalog at `docs/commands/communication.md` omits commands that exist in `crates/domains/communication/src/commands.rs` and are mandated by the spec aggregate definitions: `CreateComplaintType`, `UpdateComplaintType`, `DeleteComplaintType`, `ClassifyChatInvitation`, `MarkContactMessageViewed`, `OpenChatConversation`, `CloseChatConversation`, and `DeleteChatMessage`. Consumers relying on the catalog as a quick-reference index will be unaware of these commands.

**Expected:**

Rows for each of the 8 missing commands in `docs/commands/communication.md`, with capability, description, emitted events, and idempotency column populated.

**Evidence:**

- `crates/domains/communication/src/commands.rs:471-513, 1406-1421, 1526-1535, 1035-1060, 1107-1117` (the 8 commands exist in code).
  - `docs/specs/communication/aggregates.md:111-113, 653, 741, 447-448` (spec mandates them as aggregate commands).
  - `docs/commands/communication.md` lines 13-69 do not list any of these 8 commands.

---

### FINDING 34 (id: `DOMAIN-COM-034`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `docs/specs/communication/commands.md:1-674` (full file)

**Description:**

The spec at `docs/specs/communication/commands.md` is missing full Rust struct definitions for: `CreateComplaintTypeCommand`, `UpdateComplaintTypeCommand`, `DeleteComplaintTypeCommand`, `ClassifyChatInvitationCommand`, `MarkContactMessageViewedCommand`, `OpenChatConversationCommand`, `CloseChatConversationCommand`, and `DeleteChatMessageCommand`. The aggregate spec (`docs/specs/communication/aggregates.md`) mandates these commands as part of the public surface, and the code defines and uses them, but no spec-level definition documents their fields or capability requirements.

**Expected:**

A full code block per command with fields, capability, pre-conditions, and effects.

**Evidence:**

- `docs/specs/communication/commands.md:1-674` searches return zero matches for `CreateComplaintTypeCommand`, `UpdateComplaintTypeCommand`, `DeleteComplaintTypeCommand`, `ClassifyChatInvitationCommand`, `MarkContactMessageViewedCommand`, `OpenChatConversationCommand`, `CloseChatConversationCommand`, `DeleteChatMessageCommand`.
  - `crates/domains/communication/src/commands.rs:471, 487, 505, 1406, 1526, 1035, 1051, 1107` (the 8 commands exist with full fields in code).
  - `docs/specs/communication/aggregates.md:111-113, 653, 741, 447-448` (spec mandates them).

---

### FINDING 37 (id: `DOMAIN-COM-037`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/aggregate.rs:2210`

**Description:**

`SendMessage::dispatch` performs a `usize -> u32` truncation via `as` cast: `let count = self.audience.len() as u32;`. The engine's code standards forbid `as` casts on numerics (`AGENTS.md` "Numeric conversions use TryFrom/TryInto; `as` on numerics is forbidden").

**Expected:**

`u32::try_from(self.audience.len()).map_err(|_| DomainError::validation(...))?` or equivalent.

**Evidence:**

- `crates/domains/communication/src/aggregate.rs:2210` `let count = self.audience.len() as u32;`
  - `AGENTS.md` "Numeric conversions use TryFrom/TryInto; `as` on numerics is forbidden."

---

### FINDING 38 (id: `DOMAIN-COM-038`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/services.rs:138` and `crates/domains/communication/src/services.rs:1813` and `crates/domains/communication/src/services.rs:1197`

**Description:**

Production-path uses of `.unwrap_or(...)` in service factory functions: `publish_notice` (`cmd.publish_at.unwrap_or(now)`), `unpublish_notice` (`cmd.reason.unwrap_or_default()`), and `send_chat_message` (`cmd.conversation_id.unwrap_or_else(|| ...)`). The engine's code standards forbid `unwrap()`/`expect()` in production paths.

**Expected:**

Idiomatic `Option::unwrap_or` is allowed by clippy::unwrap_used only when gated by `#![allow(...)]`, but `AGENTS.md` and `docs/code-standards.md` say "unwrap, expect, panic! are forbidden in production paths" — the gating should be removed and replaced with explicit handling.

**Evidence:**

- `crates/domains/communication/src/services.rs:138` `let published_at = cmd.publish_at.unwrap_or(now);`
  - `crates/domains/communication/src/services.rs:1813` `cmd.reason.unwrap_or_default(),`
  - `crates/domains/communication/src/services.rs:1197` `.unwrap_or_else(|| ChatConversationId::new(school, event_id_to_uuid(ids.next_event_id())));`
  - `crates/domains/communication/src/services.rs:28` `#![allow(unused_imports)]` (no `unwrap_used` allow at module level).

---

### FINDING 4 (id: `DOMAIN-COM-004`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/entities.rs:814` and `crates/domains/communication/src/value_objects.rs:2152`

**Description:**

`CustomSmsSettingParam` is defined twice with conflicting shapes. `entities.rs:814` is a struct `{ key: String, value: String }`. `value_objects.rs:2152` is `pub struct CustomSmsSettingParam(pub Vec<(String, String)>)`. Two incompatible types under the same name.

**Expected:**

A single definition matching the spec at `docs/specs/communication/commands.md:659-669` (`Vec<(String, String)>`).

**Evidence:**

- `crates/domains/communication/src/entities.rs:813-819` `pub struct CustomSmsSettingParam { pub key: String, pub value: String, }`
  - `crates/domains/communication/src/value_objects.rs:2150-2152` `pub struct CustomSmsSettingParam(pub Vec<(String, String)>);`

---

### FINDING 5 (id: `DOMAIN-COM-005`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/entities.rs:699` and `crates/domains/communication/src/value_objects.rs:2088`

**Description:**

`NoticeAudience` is defined twice with the same `Vec<RoleId>` payload but in two different modules (`entities.rs` and `value_objects.rs`). Consumers importing `NoticeAudience` from `crate::value_objects` and from `crate::entities` resolve to different types.

**Expected:**

A single definition of `NoticeAudience`.

**Evidence:**

- `crates/domains/communication/src/entities.rs:699` `pub struct NoticeAudience(pub Vec<RoleId>);`
  - `crates/domains/communication/src/value_objects.rs:2088` `pub struct NoticeAudience(pub Vec<RoleId>);`

---

### FINDING 10 (id: `DOMAIN-COM-010`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/services.rs:1186-1224`

**Description:**

`send_chat_message` does not enforce the spec's pre-condition "`to_id` is not blocked by `from_id`; `from_id` is not blocked by `to_id`" (`docs/specs/communication/commands.md:417-418`). The service unconditionally mints a new `ChatMessageId` and emits `ChatMessageSent` without consulting any block list.

**Expected:**

A `Result`-returning block check before the message is created.

**Evidence:**

- `crates/domains/communication/src/services.rs:1186-1224` (no block-list consultation)
  - `docs/specs/communication/commands.md:417-418` `Pre-conditions: \`to_id\` is not blocked by \`from_id\`; \`from_id\` is not blocked by \`to_id\`.`

---

### FINDING 18 (id: `DOMAIN-COM-018`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/services.rs:2242-2250` and `docs/specs/communication/services.md:37`

**Description:**

`ChatService::can_post` has a different signature from the spec. Spec: `pub fn can_post(group: &ChatGroup, user: &ChatGroupUser) -> bool`. Code: `pub fn can_post(group: &ChatGroup, user: UserId, membership: Option<&ChatGroupUser>) -> bool`. The spec takes a `&ChatGroupUser`; the code splits it into a `UserId` plus an `Option<&ChatGroupUser>` lookup. Also the code's logic ("not read-only ⇒ true") is inverted relative to the spec which says "Closed group only admins may post; ReadOnly group nobody may post".

**Expected:**

`(group: &ChatGroup, user: &ChatGroupUser) -> bool` per spec.

**Evidence:**

- `crates/domains/communication/src/services.rs:2242-2250` `if !group.read_only { return true; }` then `matches!(m.role, ChatGroupRole::Admin)`.
  - `docs/specs/communication/services.md:37` `pub fn can_post(group: &ChatGroup, user: &ChatGroupUser) -> bool { ... }`
  - `docs/specs/communication/services.md:44-45` `ChatService::can_post enforces the GroupType policy: in a Closed group only admins may post; in a ReadOnly group nobody may post.`

---

### FINDING 26 (id: `DOMAIN-COM-026`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/commands.rs:1504-1517` and `docs/specs/communication/commands.md:575-582`

**Description:**

`ReceiveContactMessageCommand` field type drift on `subject`. Spec: `subject: String`. Code: `subject: EmailSubject`. The spec treats the contact-form subject as a free-form string; the code enforces email-subject validation (1..=200 chars).

**Expected:**

`pub subject: String` per spec.

**Evidence:**

- `crates/domains/communication/src/commands.rs:1514` `pub subject: EmailSubject,`
  - `docs/specs/communication/commands.md:574-582` `pub struct ReceiveContactMessageCommand { ... pub subject: String, ... }`

---

### FINDING 32 (id: `DOMAIN-COM-032`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/events.rs:2905-2913` and `docs/specs/communication/events.md:258`

**Description:**

`ChatInvitationClassified` field name drift: spec uses `pub type: ChatInvitationTypeEnum` (Rust keyword `type`); code renames the field to `invitation_type`. The spec accepts the awkward `type` name as the canonical identifier; the code's rename is a benign ergonomic change but is a wire-format-level divergence from the spec.

**Expected:**

Spec field name `type: ChatInvitationTypeEnum` per `docs/specs/communication/events.md:258`.

**Evidence:**

- `crates/domains/communication/src/events.rs:2909` `pub invitation_type: ChatInvitationTypeEnum,`
  - `docs/specs/communication/events.md:258` `pub struct ChatInvitationClassified { pub chat_invitation_type_id: ChatInvitationTypeId, pub invitation_id: ChatInvitationId, pub type: ChatInvitationTypeEnum }`

---

### FINDING 35 (id: `DOMAIN-COM-035`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/aggregate.rs:76,119`

**Description:**

`Notice` carries a `notice_type: NoticeType` field and a default value `NoticeType::General` that is not present in the spec. `docs/specs/communication/aggregates.md:5-54` lists only `title`, `body`, `notice_date`, `publish_on`, `audience`, `attachment` as Notice fields. The `NoticeType` value object is also absent from `docs/specs/communication/value-objects.md`.

**Expected:**

Either remove the field, or add a spec entry documenting the field and the enum.

**Evidence:**

- `crates/domains/communication/src/aggregate.rs:76` `pub notice_type: NoticeType,`
  - `crates/domains/communication/src/aggregate.rs:119` `notice_type: NoticeType::General,`
  - `crates/domains/communication/src/value_objects.rs:260-280` defines `NoticeType` (General, Class, Student, Staff, Parent, Event).
  - `docs/specs/communication/aggregates.md:5-54` does not mention `notice_type`.
  - `docs/specs/communication/value-objects.md:1-162` does not mention `NoticeType`.

---

### FINDING 36 (id: `DOMAIN-COM-036`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/aggregate.rs:2683` and `crates/domains/communication/src/aggregate.rs:20`

**Description:**

`aggregate.rs:2683` declares `fn _unused_imports(_: StudentId, _: BTreeMap<String, String>) {}` as a dead-code anchor to silence the `unused_imports` lint on `StudentId` and `BTreeMap`. The lint is allowed at module level (`#![allow(unused_imports)]` at line 17). The function is unreachable and adds no behavior; both imports are used elsewhere in the file via aggregate fields. The anchor itself is evidence that the lint allow is wider than necessary.

**Expected:**

A focused lint allow at the import sites, or removal of the dead-code anchor.

**Evidence:**

- `crates/domains/communication/src/aggregate.rs:2683` `fn _unused_imports(_: StudentId, _: BTreeMap<String, String>) {}`
  - `crates/domains/communication/src/aggregate.rs:17` `#![allow(unused_imports)]`

---

### FINDING 39 (id: `DOMAIN-COM-039`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/tools/storage-parity/tests/communication_integration.rs:150`

**Description:**

The integration test file `communication_integration.rs` defines 6 scenarios under `mod full_prelude_scenarios` gated by `#[cfg(any())]`. The `cfg(any())` with no conditions never matches, so none of the 6 scenarios (vertical slice, capability check, event-type round trip, append-only invariant, notification dispatch, bulk send) actually run. Only `communication_package_metadata_is_set` and `communication_full_prelude_scenarios_compile_only_when_wired` execute, both of which are trivial assertions. The `coverage.toml` rows for the 12 aggregates all carry `status = "Tested"` despite this gap.

**Expected:**

Either flip the gate to `#[cfg(all())]` (so the 6 scenarios compile and run) or downgrade the coverage rows to `status = "NotTested"` / `status = "Stub"`.

**Evidence:**

- `crates/tools/storage-parity/tests/communication_integration.rs:150` `#[cfg(any())]`
  - `crates/tools/storage-parity/tests/communication_integration.rs:117` `assert!(PACKAGE_NAME == "educore-communication");`
  - `docs/coverage.toml:1372-1480` (12 rows with `status = "Tested"` referencing `communication_integration.rs`).

---

### FINDING 40 (id: `DOMAIN-COM-040`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/aggregate.rs:2683` (and the `tests/` directory is absent)

**Description:**

The crate-local integration-test directory `crates/domains/communication/tests/` does not exist. Per `AGENTS.md`'s "Module Layout (per domain)" pattern and the per-domain convention used by other crates (e.g. `crates/domains/academic/tests/`), the communication crate should host its own `tests/` directory. The current setup forces all domain tests into `crates/tools/storage-parity/tests/`, which then gates them with `#[cfg(any())]` (see Finding DOMAIN-COM-039).

**Expected:**

A populated `crates/domains/communication/tests/` directory containing end-to-end scenarios for the 26 aggregates, the 73 events, and the 70+ service factory functions.

**Evidence:**

- `crates/domains/communication/` contents (no `tests/` subdir).
  - `crates/domains/communication/Cargo.toml` has no `[[test]]` entries.

---

### FINDING 45 (id: `DOMAIN-COM-045`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `docs/specs/communication/value-objects.md:41-162` (full table) and `crates/domains/communication/src/value_objects.rs:959-982`

**Description:**

The "complaint workflow value object" at `value_objects.rs:959` is `ComplaintAction` (an enum with `Open`, `InProgress`, `Resolve` variants). The spec table at `docs/specs/communication/value-objects.md:41-162` does NOT list `ComplaintAction` as a value object. The spec's only mention of `ComplaintAction` is at `docs/specs/communication/services.md:55` as a parameter to `ComplaintService::next_status`. The Rust source places it in `value_objects.rs` but the spec categorizes it as a service-input type, not a value object.

**Expected:**

Either move `ComplaintAction` to `services.rs` (matching the spec's placement in `services.md`) or add a row to `docs/specs/communication/value-objects.md` documenting the VO.

**Evidence:**

- `crates/domains/communication/src/value_objects.rs:959-982` `pub enum ComplaintAction { Open, InProgress, Resolve }`
  - `docs/specs/communication/value-objects.md:41-162` searches return zero matches for `ComplaintAction`.
  - `docs/specs/communication/services.md:55` `pub fn next_status(current: ComplaintStatus, action: ComplaintAction) -> ComplaintStatus { ... }` — the only spec mention.

---

### FINDING 6 (id: `DOMAIN-COM-006`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/services.rs:1554-1582`

**Description:**

The `BlockUser` service does not implement the spec's mandated idempotency. The spec at `docs/specs/communication/workflows.md:196-197` says "BlockUser is idempotent on (block_by, block_to). A duplicate is a no-op success." The service unconditionally mints a fresh `ChatBlockUserId` and emits a new `UserBlocked` event without consulting any existing block list.

**Expected:**

A lookup-then-no-op-or-emit path that returns the existing block on duplicate.

**Evidence:**

- `crates/domains/communication/src/services.rs:1554-1582` (block_user signature and body never reads existing blocks)
  - `docs/specs/communication/workflows.md:196-197` `BlockUser is idempotent on (block_by, block_to). A duplicate is a no-op success.`

---

### FINDING 7 (id: `DOMAIN-COM-007`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/services.rs:1043-1070`

**Description:**

`configure_absent_notification` does not implement the spec's mandated idempotency. The spec at `docs/specs/communication/workflows.md:198-199` says "ConfigureAbsentNotification is idempotent on (school_id, time_from, time_to)." The service unconditionally mints a fresh `AbsentNotificationTimeSetupId` and emits a new `AbsentNotificationScheduled` event without checking for an existing window.

**Expected:**

Lookup-then-no-op-or-emit semantics keyed on `(school_id, time_from, time_to)`.

**Evidence:**

- `crates/domains/communication/src/services.rs:1043-1070` (no list-then-check path)
  - `docs/specs/communication/workflows.md:198-199` `ConfigureAbsentNotification is idempotent on (school_id, time_from, time_to).`

---

### FINDING 8 (id: `DOMAIN-COM-008`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/services.rs:195-228`

**Description:**

`register_complaint` does not implement the spec's mandated idempotency. The spec at `docs/specs/communication/workflows.md:191-193` says "RegisterComplaint is idempotent on (complaint_type, date, phone). Re-issuing a complaint for the same phone on the same day returns the prior record." The service unconditionally mints a fresh `ComplaintId` and emits `ComplaintRegistered`.

**Expected:**

Lookup-then-no-op-or-emit keyed on `(complaint_type_id, date, phone)`.

**Evidence:**

- `crates/domains/communication/src/services.rs:195-228` (no lookup-then-check path)
  - `docs/specs/communication/workflows.md:191-193` `RegisterComplaint is idempotent on (complaint_type, date, phone). Re-issuing ... returns the prior record.`

---

### FINDING 9 (id: `DOMAIN-COM-009`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/services.rs:195-228`

**Description:**

`register_complaint` does not enforce the spec's pre-condition "If source is not `Anonymous`, at least one of `complaint_by` or `phone` is set" (`docs/specs/communication/commands.md:113-115`). The service unconditionally creates the complaint.

**Expected:**

A `Result`-returning validation that rejects `complaint_source != Anonymous && complaint_by.is_none() && phone.is_none()`.

**Evidence:**

- `crates/domains/communication/src/services.rs:195-228` body has no source-vs-identity check.
  - `docs/specs/communication/commands.md:113-115` `Pre-conditions: If source is not \`Anonymous\`, at least one of \`complaint_by\` or \`phone\` is set.`

---

### FINDING 41 (id: `DOMAIN-COM-041`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/lib.rs:118-137` (prelude block) and `crates/domains/communication/src/lib.rs:54` (comment)

**Description:**

Self-inconsistent comment vs. prelude block: `lib.rs:54` says "26 headline aggregate roots (from crate::aggregate)" but the prelude block (lines 50-57) only re-exports 25 distinct types from `crate::aggregate::*` (the second-to-last line ends with `Notification, NotificationSetting, PhoneCallLog, SendMessage, SmsGateway, SmsLog, SmsTemplate, SpeechSlider,` — 25 names visible before the trailing comma; line 56 has `};`). The 26th aggregate (`ChatStatusRecord`) is missing from the prelude re-export. Consumers cannot access `ChatStatusRecord` via `educore::communication::*`.

**Expected:**

Prelude re-exports all 26 aggregates including `ChatStatusRecord` (or rename to `ChatStatus` per Finding DOMAIN-COM-001).

**Evidence:**

- `crates/domains/communication/src/lib.rs:50-57` re-export list (notice absence of `ChatStatusRecord`).
  - `crates/domains/communication/src/aggregate.rs:2068` defines `pub struct ChatStatusRecord`.

---

### FINDING 42 (id: `DOMAIN-COM-042`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/lib.rs:15-22`

**Description:**

Module visibility is inconsistent with the manifest. The manifest at `crates/domains/communication/.phase10-manifest.md:695-703` lists modules as `pub mod value_objects; mod aggregate; pub mod commands; mod entities; mod errors; pub mod events; pub mod query; mod repository; pub mod services;`. The actual `lib.rs:15-22` declares `mod aggregate; mod entities; mod errors; mod repository;` (4 modules private), but `lib.rs` re-exports the contents of those private modules from the prelude. Consumers cannot `use educore_communication::aggregate::Notice` directly even though `Notice` is reachable via the prelude.

**Expected:**

Per the manifest, all 9 modules are `pub mod`. The current private visibility contradicts the manifest.

**Evidence:**

- `crates/domains/communication/src/lib.rs:15-22` `mod aggregate; pub mod commands; mod entities; mod errors; pub mod events; pub mod query; mod repository; pub mod services; pub mod value_objects;`
  - `crates/domains/communication/.phase10-manifest.md:695-703` lists `pub mod value_objects; mod aggregate; pub mod commands; mod entities; mod errors; pub mod events; pub mod query; mod repository; pub mod services;` — note `mod aggregate;`, `mod entities;`, `mod errors;`, `mod repository;` all `mod` not `pub mod`.

---

### FINDING 43 (id: `DOMAIN-COM-043`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/lib.rs:111-116` and `crates/domains/communication/src/lib.rs:118-137`

**Description:**

The prelude block (lines 118-137) re-exports 70 service functions under the "70 pure factory service fns" comment, but the count is actually 72. The same block also lists "7 headline service fns" (lines 112-116) but only re-exports 6 of them (`mark_as_read`, `notify_user`, `send_chat_message`, `send_complaint_message`, `send_email_message`, `send_notice_message`, `send_sms_message` = 7 names, but the comment says 7). The actual count from `services.rs` is 72 sync + 7 async = 79 functions. The "70 pure factory service functions" comment is misleading.

**Expected:**

Comments accurately reflect the function count (72 pure factory fns + 7 headline async fns).

**Evidence:**

- `crates/domains/communication/src/lib.rs:117` `// 70 pure factory service fns (re-export all from crate::services)`
  - `grep -c "^pub fn " crates/domains/communication/src/services.rs` = 72 sync fns.
  - `grep -c "pub async fn " crates/domains/communication/src/services.rs` = 7 async fns.

---

### FINDING 46 (id: `DOMAIN-COM-046`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/entities.rs:42-44`

**Description:**

`#![allow(missing_docs)]` at the top of `entities.rs` (and similarly in `aggregate.rs`, `commands.rs`, `events.rs`, `query.rs`, `repository.rs`, `services.rs`, `value_objects.rs`) suppresses `deny(missing_docs)` for the entire module. `lib.rs:10` has `#![deny(missing_docs)]` for the crate, but the inner modules all opt out. Consumers browsing `educore-communication::entities::NoticeAttachment` see no rustdoc; the per-module lint allow is at odds with the crate-level deny.

**Expected:**

Either remove the module-level `allow(missing_docs)` (so the crate-level deny fires and forces docs) or replace the crate-level deny with `warn(missing_docs)` and document the policy explicitly.

**Evidence:**

- `crates/domains/communication/src/lib.rs:10` `#![deny(missing_docs)]`
  - `crates/domains/communication/src/entities.rs:42` `#![allow(missing_docs)]`
  - `crates/domains/communication/src/aggregate.rs:16` `#![allow(missing_docs)]`
  - `crates/domains/communication/src/commands.rs:18` `#![allow(missing_docs)]`
  - `crates/domains/communication/src/events.rs:16` `#![allow(missing_docs)]`
  - `crates/domains/communication/src/query.rs:14` `#![allow(missing_docs)]`
  - `crates/domains/communication/src/repository.rs:30` `#![allow(missing_docs)]`
  - `crates/domains/communication/src/services.rs:28` `#![allow(missing_docs)]`
  - `crates/domains/communication/src/value_objects.rs:27` `#![allow(missing_docs)]`

---

### FINDING 47 (id: `DOMAIN-COM-047`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/entities.rs` (entire file, 833 lines) and `docs/specs/communication/tables.md:7-33` (25 rows)

**Description:**

None of the 25 tables in `docs/specs/communication/tables.md` has a corresponding `#[derive(DomainQuery)]` struct in `entities.rs`. The spec at `docs/specs/communication/tables.md:7-33` lists 25 tables (`communication_notice_boards`, `communication_complaints`, etc.), but `entities.rs` defines child entities (with their own aggregate-scoped fields) rather than row-level `DomainQuery` derive structs. The `DomainQuery` macro is referenced as future work in `query.rs:7-9` and is not yet shipped; until the macro lands, no table has a typed query AST consumer.

**Expected:**

A `DomainQuery`-derived struct per table row, plus the macro itself.

**Evidence:**

- `crates/domains/communication/src/entities.rs` (no `#[derive(DomainQuery)]` anywhere).
  - `crates/domains/communication/src/query.rs:7-9` `Phase 10 ships the 26 typed query stubs ... The typed executors land in a follow-up phase alongside the #[derive(DomainQuery)] macro emissions`
  - `crates/domains/communication/src/query.rs:59` `"NoticeQuery::execute is a Phase 10 stub; real executor lands with the DomainQuery macro"`
  - `docs/specs/communication/tables.md:7-33` 25 table rows without a corresponding derive struct.

---

### FINDING 44 (id: `DOMAIN-COM-044`)

- **Source:** `docs/audit_reports/findings/wave1-communication.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** `crates/domains/communication/src/services.rs:1135-1162`

**Description:**

`open_chat_conversation` mints a new `ChatConversationId` on every invocation regardless of whether an open conversation between the same two users already exists. The spec at `docs/specs/communication/commands.md:420-423` says "If `conversation_id` is null and a prior conversation exists between `from_id` and `to_id`, the existing conversation is reused". The service unconditionally creates a new conversation aggregate.

**Expected:**

A lookup-then-create-or-reuse path that searches existing conversations for `(from_id, to_id)`.

**Evidence:**

- `crates/domains/communication/src/services.rs:1135-1162` (no lookup path).
  - `docs/specs/communication/commands.md:420-423` `Effects: Creates a ChatMessage and emits ChatMessageSent. If conversation_id is null and a prior conversation exists between from_id and to_id, the existing conversation is reused; otherwise a new ChatConversation is implicitly opened.`

---


## Documents (target id prefix: `DOMAIN-DOC`)

**Path:** `crates/domains/documents/`  
**Total findings:** 39 (4 critical, 10 high, 15 medium, 10 low)


### FINDING DOMAIN-DOC-001 (id: `DOMAIN-DOC-001`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** crates/domains/documents/Cargo.toml:22

**Description:**

The `educore-documents` crate (domains tier) declares a direct dependency on `educore-event-bus`, which lives in `crates/adapters/event-bus/` (adapters tier). This violates the tier boundary rule that a domains crate must not import from an adapters crate.

**Expected:**

AGENTS.md § "Dependency Rules" mandates: "A domain crate may not depend on: Any crate in the adapters tier." The spec `docs/specs/documents/overview.md` § "Dependencies" lists only `educore-core`, `educore-platform`, `educore-rbac`, `educore-events`.

**Evidence:**

`crates/domains/documents/Cargo.toml:21-22` reads: `educore-storage = { workspace = true }` followed by `educore-event-bus = { workspace = true }`. The `educore-event-bus` crate is published at `crates/adapters/event-bus/Cargo.toml` per `AGENTS.md` § "Tier System". Concrete use site: `crates/domains/documents/src/services.rs:1304` (`use educore_event_bus::InProcessEventBus;`).

---

### FINDING DOMAIN-DOC-002 (id: `DOMAIN-DOC-002`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** crates/domains/documents/src/repository.rs:23-60

**Description:**

The `FormDownloadRepository` trait is missing the `delete` method declared in the spec's `repositories.md`. The spec calls for hard-delete capability on the port even though the engine never hard-deletes in production — the spec is the source of truth for the trait surface.

**Expected:**

`docs/specs/documents/repositories.md:17` mandates: `async fn delete(&self, id: FormDownloadId) -> Result<()>` on the `FormDownloadRepository` trait.

**Evidence:**

`crates/domains/documents/src/repository.rs:23-60` defines `FormDownloadRepository` with methods `get`, `list`, `list_public`, `insert`, `update`, `by_publish_date`, `count`, `page`. There is no `delete` method. `grep "fn delete" crates/domains/documents/src/repository.rs` returns no matches.

---

### FINDING DOMAIN-DOC-003 (id: `DOMAIN-DOC-003`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** crates/domains/documents/src/repository.rs:95-140

**Description:**

The `PostalDispatchRepository` trait is missing the `delete` method declared in the spec's `repositories.md`.

**Expected:**

`docs/specs/documents/repositories.md:33` mandates: `async fn delete(&self, id: PostalDispatchId) -> Result<()>` on the `PostalDispatchRepository` trait.

**Evidence:**

`crates/domains/documents/src/repository.rs:95-140` defines `PostalDispatchRepository` with methods `get`, `list`, `insert`, `update`, `find_by_reference`, `between`, `by_academic_year`. No `delete` method.

---

### FINDING DOMAIN-DOC-004 (id: `DOMAIN-DOC-004`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** crates/domains/documents/src/repository.rs:176-217

**Description:**

The `PostalReceiveRepository` trait is missing the `delete` method declared in the spec's `repositories.md`.

**Expected:**

`docs/specs/documents/repositories.md:49` mandates: `async fn delete(&self, id: PostalReceiveId) -> Result<()>` on the `PostalReceiveRepository` trait.

**Evidence:**

`crates/domains/documents/src/repository.rs:176-217` defines `PostalReceiveRepository` with methods `get`, `list`, `insert`, `update`, `find_by_reference`, `between`, `by_academic_year`. No `delete` method.

---

### FINDING DOMAIN-DOC-005 (id: `DOMAIN-DOC-005`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** crates/domains/documents/src/repository.rs:53-59

**Description:**

The `page` method on `FormDownloadRepository` returns `Vec<FormDownload>` but the spec requires `Page<FormDownload>`. Additionally, no generic `Page<T>` type exists in the engine (`crates/infra/core/src/query.rs:399` defines a non-generic `Page { offset, limit }` struct), so the spec's reference to `Page<FormDownload>` is itself unimplementable without a new engine type.

**Expected:**

`docs/specs/documents/repositories.md:20` mandates: `async fn page(&self, school: SchoolId, q: FormDownloadQuery, offset: u32, limit: u32) -> Result<Page<FormDownload>>`.

**Evidence:**

`crates/domains/documents/src/repository.rs:53-59` reads: `async fn page(&self, school: SchoolId, q: FormDownloadQuery, offset: u32, limit: u32) -> StorageResult<Vec<FormDownload>>;`. `grep -rn "pub struct Page<" crates` returns no matches. The non-generic `Page` struct lives at `crates/infra/core/src/query.rs:399`.

---

### FINDING DOMAIN-DOC-006 (id: `DOMAIN-DOC-006`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** crates/domains/documents/src/services.rs:847-887

**Description:**

The `track_postal_service` always returns `dispatch: None` in the `PostalPair` result — it never queries the dispatch repository. The spec mandates returning matched dispatch + receive records; the workflow `## Postal Tracking Workflow` step 2 says "The system returns the list of matching dispatch and receive records within the school."

**Expected:**

`docs/specs/documents/workflows.md:69-70` mandates step 2 of the Postal Tracking Workflow: "The system returns the list of matching dispatch and receive records within the school."

**Evidence:**

`crates/domains/documents/src/services.rs:856-887` `track_postal_service` body contains `let _ = dispatch_repo;` (line 868) and then constructs `let pair = PostalPair { dispatch: None, receive: receives.into_iter().next(), };` (lines 872-875). The comment at line 850-854 explicitly states: "Until then the dispatch side is always `None`."

---

### FINDING DOMAIN-DOC-007 (id: `DOMAIN-DOC-007`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** crates/domains/documents/src/services.rs:1-1911

**Description:**

The four `Specification` structs mandated by the spec are not implemented: `PublicForms`, `ActiveForms`, `DispatchesInDateRange`, `ReceivesInDateRange`. None of the corresponding trait `Specification<T>` exists in the engine either.

**Expected:**

`docs/specs/documents/services.md:39-93` mandates four `Specification<T>` impls: `PublicForms`, `ActiveForms`, `DispatchesInDateRange`, `ReceivesInDateRange`.

**Evidence:**

`grep -rn "PublicForms\|ActiveForms\|DispatchesInDateRange\|ReceivesInDateRange\|trait Specification" crates` returns zero matches. No `Specification<T>` trait exists in `crates/`.

---

### FINDING DOMAIN-DOC-009 (id: `DOMAIN-DOC-009`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** crates/domains/documents/src/services.rs:145,187,257,487,531,604,700,743,815,867

**Description:**

Capability gating uses the `FormDownload{Upload,Update,Delete}` / `PostalDispatch{Create,Update,Delete}` / `PostalReceive{Create,Update,Delete}` / `PostalRead` naming. The spec mandates the `<Domain>.<Aggregate>.<Action>` form: `Form.Upload`, `Form.Update`, `Form.Delete`, `Postal.Dispatch`, `Postal.Receive`, `Postal.Update`, `Postal.Delete`, `Postal.Read`.

**Expected:**

`docs/specs/documents/permissions.md:7-32` mandates `<Domain>.<Aggregate>.<Action>` (e.g. `Form.Upload`, `Postal.Dispatch`). `docs/specs/documents/commands.md:24,43,56,75,95,109,128,148,162,174` uses those exact strings in `**Capability:**` markers.

**Evidence:**

`crates/domains/documents/src/services.rs:145` reads `Capability::FormDownloadUpload` (vs spec `Form.Upload`). Line 487 reads `Capability::PostalDispatchCreate` (vs spec `Postal.Dispatch`). Line 867 reads `Capability::PostalRead` (matches spec).

---

### FINDING DOMAIN-DOC-010 (id: `DOMAIN-DOC-010`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** crates/domains/documents/src/services.rs:1-1911

**Description:**

The `Form.Read` capability is declared in `educore-rbac` (`FormDownloadRead`, line 720 of `crates/cross-cutting/rbac/src/value_objects.rs`) but is never checked by any service factory in `crates/domains/documents/src/services.rs`. Staff read access has no enforcement gate in the documents crate.

**Expected:**

`docs/specs/documents/permissions.md:23` mandates `Form.Read` capability; `docs/specs/documents/permissions.md:50-63` says "Capabilities are checked at the command boundary."

**Evidence:**

`grep -n "FormDownloadRead\|Form\.Read" crates/domains/documents/src/services.rs` returns no matches. The capability exists in `crates/cross-cutting/rbac/src/value_objects.rs:720` (`FormDownloadRead`).

---

### FINDING DOMAIN-DOC-021 (id: `DOMAIN-DOC-021`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** High
- **Area:** docs-vs-code
- **Location:** docs/handoff/PHASE-11-HANDOFF.md:177-184

**Description:**

Phase 11 hand-off claims 145 unit tests with a specific per-file breakdown. The breakdown is incorrect: `services.rs` has 27 tests (24 `#[test]` + 3 `#[tokio::test]` matching service-factory cases plus additional) per `grep -c "#\[test\]\|#\[tokio::test\]"` of `crates/domains/documents/src/services.rs`, not the 18 stated.

**Expected:**

AGENTS.md § "Engine Rules" require honest accounting.

**Evidence:**

`docs/handoff/PHASE-11-HANDOFF.md:177-184` claims "services.rs (18)". Actual count is 27 in `crates/domains/documents/src/services.rs` (24 sync tests + 3 of 9 async tests, all visible via `grep -c "#\[test\]\|#\[tokio::test\]"`). The grand total 145 still matches because other counts differ by an inverse amount.

---

### FINDING DOMAIN-DOC-022 (id: `DOMAIN-DOC-022`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** High
- **Area:** docs-vs-code
- **Location:** docs/handoff/PHASE-11-HANDOFF.md:156-160

**Description:**

Phase 11 hand-off claims 5 closed enums (`FormSource`, `PostalDirection`, `PostalAttachmentKind`, `FormVisibility`, `UpdateOutcome`) plus an `AuditFields` 17-field footer struct. None of these named items exist in the code.

**Expected:**

AGENTS.md § "Engine Rules" require honest accounting of what was actually shipped.

**Evidence:**

`docs/handoff/PHASE-11-HANDOFF.md:156-160` lists `FormSource`, `PostalDirection`, `PostalAttachmentKind`, `FormVisibility`, `UpdateOutcome`, and `AuditFields`. `grep -rn "FormSource\|PostalDirection\|PostalAttachmentKind\|FormVisibility\|UpdateOutcome\|AuditFields" crates/domains/documents/src/` returns zero matches. The actual code has only 2 closed enums: `DocumentType` (`value_objects.rs:806`) and `DocumentVisibility` (`value_objects.rs:840`).

---

### FINDING DOMAIN-DOC-029 (id: `DOMAIN-DOC-029`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** High
- **Area:** docs-vs-code
- **Location:** docs/specs/documents/tables.md:7-11

**Description:**

`tables.md` declares 3 tables for the documents domain. None of them is referenced by a `#[derive(DomainQuery)]` struct in `crates/domains/documents/src/aggregate.rs` (or anywhere else in the crate). The crate uses manual typed `*Query` builders instead of the macro-generated AST mandated by AGENTS.md § "Engine Rules" (rule 6) and rule 2 ("Compile-time safety over strings").

**Expected:**

AGENTS.md § "Engine Rules" rule 6: "No SQL/NoSQL emission from macros. The `#[derive(DomainQuery)]` macro emits an AST; storage adapters translate the AST." Each `tables.md` row should have a corresponding macro-emitted typed AST struct in `entities.rs` (per `docs/build-plan.md` § "The No-Gaps Gates").

**Evidence:**

`docs/specs/documents/tables.md:7-11` lists `documents_form_downloads`, `documents_postal_dispatches`, `documents_postal_receives`. `grep -rn "#\[derive(DomainQuery)\]" crates/domains/documents/` returns zero matches. Manual `FormDownloadQuery`, `PostalDispatchQuery`, `PostalReceiveQuery` builders live at `crates/domains/documents/src/query.rs` instead.

---

### FINDING DOMAIN-DOC-037 (id: `DOMAIN-DOC-037`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** crates/domains/documents/src/services.rs:847-887

**Description:**

The `track_postal_service` audit row uses `AuditTarget::Other("postal_track".to_owned(), Uuid::now_v7())` (line 881) which invents a fresh Uuid for the audit target rather than tying the audit row to either the dispatch or the receive. This makes the audit row un-joinable to the underlying aggregates.

**Expected:**

The spec workflow `## Postal Tracking Workflow` step 4 says "The system emits no domain event for the read; the read is logged in the audit sink." The audit row should be tied to a stable target id (the dispatch or receive id) for forensic queries.

**Evidence:**

`crates/domains/documents/src/services.rs:881` `AuditTarget::Other("postal_track".to_owned(), Uuid::now_v7())`. A `Uuid::now_v7()` is minted solely for the audit row.

---

### FINDING DOMAIN-DOC-039 (id: `DOMAIN-DOC-039`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** High
- **Area:** docs-vs-code
- **Location:** docs/handoff/PHASE-11-HANDOFF.md:1-5

**Description:**

Phase 11 is documented as "Closed 2026-06-16" in both `docs/build-plan.md:1294` and `docs/handoff/PHASE-11-HANDOFF.md:4`. The close-out claim implies the spec is satisfied; however, several spec items remain unimplemented (missing `delete` methods on repository traits, missing `Specifications`, missing `DocumentsCoordinator`, partial `track_postal_service` implementation, missing capabilities `Form.Read.Public` / `Document.Read`, capability naming drift, missing `Validate` trait). These gaps indicate the close-out declaration does not match the implementation state.

**Expected:**

AGENTS.md § "Validation Checklist" requires all gates to pass before close. The Phase 11 close-out narrative does not acknowledge the gaps enumerated in findings DOMAIN-DOC-002 through DOMAIN-DOC-038.

**Evidence:**

`docs/handoff/PHASE-11-HANDOFF.md:4` "Status: Phase 11 closed." `docs/build-plan.md:1294` "Phase 11 outcome. Closed 2026-06-16." The unimplemented items enumerated in the other findings in this report contradict the "closed" assertion.

### END FINDINGS

Total findings: 39

---

### FINDING DOMAIN-DOC-008 (id: `DOMAIN-DOC-008`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/documents/src/services.rs:1-1911

**Description:**

The `DocumentsCoordinator` mandated by the spec is not implemented in this crate. The spec places the coordinator "in the engine facade", which the documents crate itself does not host; no equivalent type exists in `crates/educore/` either.

**Expected:**

`docs/specs/documents/services.md:96-114` mandates `pub struct DocumentsCoordinator<'a>` with `pub async fn upload_form(&self, cmd: UploadFormCommand) -> Result<FormDownload, DomainError>`.

**Evidence:**

`grep -rn "DocumentsCoordinator" crates` returns zero matches.

---

### FINDING DOMAIN-DOC-011 (id: `DOMAIN-DOC-011`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/cross-cutting/rbac/src/value_objects.rs:1-6206

**Description:**

The `Form.Read.Public` capability mandated by the spec is not present in the `Capability` enum. The spec also notes it is "granted to anonymous visitors on the public site", which is the only capability the `Public` role holds.

**Expected:**

`docs/specs/documents/permissions.md:24` mandates `Form.Read.Public (granted to anonymous visitors on the public site)`.

**Evidence:**

`grep -n "FormReadPublic\|FormRead\.Public\|Form\.Read\.Public" crates/cross-cutting/rbac/src/` returns zero matches. The full set of `Documents.*` capabilities in `crates/cross-cutting/rbac/src/value_objects.rs:698-734` does not include `FormReadPublic`.

---

### FINDING DOMAIN-DOC-012 (id: `DOMAIN-DOC-012`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/cross-cutting/rbac/src/value_objects.rs:1-6206

**Description:**

The `Document.Read` cross-cutting capability mandated by the spec is not present in the `Capability` enum. The spec names it as the cross-cutting read capability shared across the documents domain.

**Expected:**

`docs/specs/documents/permissions.md:16` mandates `Document.Read` (under "### Document (Cross-Cutting)").

**Evidence:**

`grep -n "DocumentRead\b\|Document\.Read" crates/cross-cutting/rbac/src/` returns no matches. The only `Document`-prefixed capabilities in rbac are `HrStaffDocumentUpload` and `HrStaffDocumentDownload` (HR-domain).

---

### FINDING DOMAIN-DOC-013 (id: `DOMAIN-DOC-013`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/documents/src/services.rs:855-887

**Description:**

The signature `track_postal_service<DRepo, RRepo>(cmd: TrackPostalCommand, dispatch_repo: Arc<DRepo>, receive_repo: Arc<RRepo>, audit: Arc<AuditWriter>, cap: &dyn CapabilityCheck)` carries an `#[allow(unused_variables, clippy::too_many_arguments)]` and explicitly discards `dispatch_repo` (`let _ = dispatch_repo;`). The unused parameter is a code smell tied to finding DOMAIN-DOC-006.

**Expected:**

The spec workflow (`docs/specs/documents/workflows.md:64-75`) says the system should query both repos and return matched records.

**Evidence:**

`crates/domains/documents/src/services.rs:855` `#[allow(unused_variables, clippy::too_many_arguments)]` on `track_postal_service`. Line 868: `let _ = dispatch_repo;`.

---

### FINDING DOMAIN-DOC-014 (id: `DOMAIN-DOC-014`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/documents/src/aggregate.rs:693-702

**Description:**

Non-test production code in `aggregate.rs` contains a `// TODO(phase-11/1C)` comment marking a known incomplete migration to `educore-academic::value_objects::AcademicYearId`. AGENTS.md § "Agent Instructions" anti-pattern list includes `// TODO:` in non-test code.

**Expected:**

AGENTS.md § "Anti-patterns" forbids `// TODO:` in non-test code; the spec `docs/specs/documents/aggregates.md:54` already establishes `PostalDispatch` belongs to a school and academic year, expecting the proper typed id.

**Evidence:**

`crates/domains/documents/src/aggregate.rs:694-702` contains the comment block `// TODO(phase-11/1C): replace this local alias with` followed by `pub type AcademicYearId = Uuid;`. Outside of `#[cfg(test)]`.

---

### FINDING DOMAIN-DOC-017 (id: `DOMAIN-DOC-017`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/documents/src/events.rs:99, 315, 524

**Description:**

The `changes` field on `FormUpdated`, `PostalDispatchUpdated`, `PostalReceiveUpdated` is typed `Vec<String>` in code but the spec mandates `Vec<&'static str>`. Per the engine rule, `Vec<&'static str>` requires the producer to pass string literals only; `Vec<String>` allows arbitrary owned data.

**Expected:**

`docs/specs/documents/events.md:46,68,96` mandates `pub changes: Vec<&'static str>` for `FormUpdated`, `PostalDispatchUpdated`, `PostalReceiveUpdated`.

**Evidence:**

`crates/domains/documents/src/events.rs:99` (`FormUpdated.changes: Vec<String>`), line 315 (`PostalDispatchUpdated.changes: Vec<String>`), line 524 (`PostalReceiveUpdated.changes: Vec<String>`). Code constructs the vector with `.to_owned()` at lines 813, 869, 936, 1243-1252, etc.

---

### FINDING DOMAIN-DOC-018 (id: `DOMAIN-DOC-018`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/documents/src/events.rs:1-1028

**Description:**

The spec's `DomainEvent` trait uses `const TYPE` as the event-type identifier and does not include `SCHEMA_VERSION` or `AGGREGATE_TYPE`. The code uses the engine's actual `DomainEvent` trait which exposes `EVENT_TYPE`, `SCHEMA_VERSION`, and `AGGREGATE_TYPE`. The event-payload structs in `events.rs` therefore include envelope metadata (`school_id`, `event_id`, `correlation_id`, `occurred_at`) that the spec attributes to the outer `EventEnvelope<E>`, not to the payload struct.

**Expected:**

`docs/specs/documents/events.md:9-16` mandates `pub trait DomainEvent { const TYPE: &'static str; fn aggregate_id(&self) -> Uuid; fn school_id(&self) -> SchoolId; fn occurred_at(&self) -> Timestamp; }`. The spec payload structs `FormUploaded`, `FormUpdated`, `FormDeleted`, etc. list only the domain payload fields.

**Evidence:**

`crates/domains/documents/src/events.rs:72-88` (`impl DomainEvent for FormUploaded` with `const EVENT_TYPE`, `const SCHEMA_VERSION: u32 = 1`, `const AGGREGATE_TYPE: &'static str = "form_download"`). Lines 24-43 show `FormUploaded` carries `school_id`, `event_id`, `correlation_id`, `occurred_at` in addition to the spec's five payload fields.

---

### FINDING DOMAIN-DOC-019 (id: `DOMAIN-DOC-019`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/documents/src/value_objects.rs:1-1110

**Description:**

The `Validate` trait mandated by the spec for value objects is not implemented anywhere in the engine. The spec mandates `pub trait Validate { fn validate(&self) -> Result<(), ValueError>; }` on every value object.

**Expected:**

`docs/specs/documents/value-objects.md:73-77` mandates: `pub trait Validate { fn validate(&self) -> Result<(), ValueError>; }`.

**Evidence:**

`grep -rn "trait Validate" crates` returns zero matches. Value objects in `value_objects.rs` validate at construction time inside `::new` constructors but never expose the trait.

---

### FINDING DOMAIN-DOC-020 (id: `DOMAIN-DOC-020`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/documents/src/services.rs:489,702

**Description:**

`dispatch_postal_service` and `receive_postal_service` mint the typed aggregate id internally (`PostalDispatchId::new(tenant.school_id, Uuid::now_v7())`) instead of accepting it from the dispatcher like `into_new_postal_dispatch` and `into_new_postal_receive` already do (which take an `id: PostalDispatchId` parameter). The two patterns are inconsistent.

**Expected:**

`docs/specs/documents/commands.md:73,118` and `crates/domains/documents/src/commands.rs:221,393` `into_new_postal_dispatch(self, id: PostalDispatchId, academic_id: AcademicYearId)` and `into_new_postal_receive(self, id: PostalReceiveId, academic_id: AcademicYearId)` show the id is supplied by the dispatcher.

**Evidence:**

`crates/domains/documents/src/services.rs:489` `let id = PostalDispatchId::new(tenant.school_id, Uuid::now_v7());` and `services.rs:702` `let id = PostalReceiveId::new(tenant.school_id, Uuid::now_v7());`.

---

### FINDING DOMAIN-DOC-023 (id: `DOMAIN-DOC-023`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** Medium
- **Area:** docs-vs-code
- **Location:** docs/handoff/PHASE-11-HANDOFF.md:149-150

**Description:**

Phase 11 hand-off describes `FileReference` as `the educore_platform::value_objects::FileReference re-export (see OQ #2)`. The code locally defines `FileReference` at `crates/domains/documents/src/value_objects.rs:683` rather than re-exporting it from `educore-platform`.

**Expected:**

Hand-off documentation should describe what the code actually does.

**Evidence:**

`docs/handoff/PHASE-11-HANDOFF.md:149-150` claim `FileReference (re-exported from educore-platform)`. `crates/domains/documents/src/value_objects.rs:683` defines `pub struct FileReference(String);`. `crates/cross-cutting/platform/src/value_objects.rs` does not contain `FileReference` (`grep "FileReference" crates/cross-cutting/platform/src/value_objects.rs` returns no matches).

---

### FINDING DOMAIN-DOC-024 (id: `DOMAIN-DOC-024`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** Medium
- **Area:** docs-vs-code
- **Location:** docs/handoff/PHASE-11-HANDOFF.md:151

**Description:**

Phase 11 hand-off describes `Url` as `re-exported from educore-platform`. The code locally defines `Url` at `crates/domains/documents/src/value_objects.rs:628` rather than re-exporting it from `educore-platform`.

**Expected:**

Hand-off documentation should describe what the code actually does.

**Evidence:**

`docs/handoff/PHASE-11-HANDOFF.md:151` claims `Url (re-exported from educore-platform)`. `crates/domains/documents/src/value_objects.rs:628` defines `pub struct Url(String);`. `grep "pub struct Url" crates/cross-cutting/platform/src/value_objects.rs` returns no matches.

---

### FINDING DOMAIN-DOC-025 (id: `DOMAIN-DOC-025`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** Medium
- **Area:** docs-vs-code
- **Location:** docs/handoff/PHASE-11-HANDOFF.md:170-175

**Description:**

Phase 11 hand-off describes the 3 typed query stubs as "returning `Err(DomainError::not_supported(...))` in Phase 11" (matching the Phase 9 / Phase 10 pattern). The actual `FormDownloadQuery`, `PostalDispatchQuery`, `PostalReceiveQuery` are typed builders with `Default`, `new`, and `with_*` methods that never return errors — there is no `not_supported` arm anywhere.

**Expected:**

Hand-off documentation should match the implementation.

**Evidence:**

`docs/handoff/PHASE-11-HANDOFF.md:170-175` claims the queries "return `Err(DomainError::not_supported(...))` in Phase 11". `crates/domains/documents/src/query.rs` contains only `Default + new + with_*` builders; `grep "not_supported" crates/domains/documents/src/query.rs` returns no matches.

---

### FINDING DOMAIN-DOC-026 (id: `DOMAIN-DOC-026`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/documents/ (no tests directory)

**Description:**

The crate has no `crates/domains/documents/tests/` directory. AGENTS.md § "Validation Checklist" requires "At least one integration test added for new behavior" per PR. The 6-scenario integration test for documents lives at `crates/tools/storage-parity/tests/documents_integration.rs`, but the crate itself does not host any tests/ folder.

**Expected:**

The standard Rust crate layout per `AGENTS.md` § "Module Layout" is: `crates/domains/<domain>/tests/` for integration tests.

**Evidence:**

`find crates/domains/documents -type d` returns only `crates/domains/documents` and `crates/domains/documents/src`. No `tests/` directory.

---

### FINDING DOMAIN-DOC-031 (id: `DOMAIN-DOC-031`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** Medium
- **Area:** docs-vs-code
- **Location:** docs/specs/documents/value-objects.md:46-54

**Description:**

`value-objects.md` lists `AcademicYearId` "From `educore-academic`" as one of the value-object rows. The documents crate uses a local `pub type AcademicYearId = Uuid;` alias instead (declared at `crates/domains/documents/src/aggregate.rs:702`). The spec/hand-off explicitly anticipate this (`PHASE-11-HANDOFF.md:349-357` OQ #1) but it remains a documented spec/code drift.

**Expected:**

`docs/specs/documents/value-objects.md:52` row "AcademicYearId | From educore-academic".

**Evidence:**

`docs/specs/documents/value-objects.md:52`. `crates/domains/documents/src/aggregate.rs:702` `pub type AcademicYearId = Uuid;` with comment block at lines 694-701 explaining the deviation.

---

### FINDING DOMAIN-DOC-033 (id: `DOMAIN-DOC-033`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** crates/domains/documents/src/aggregate.rs:694

**Description:**

The local `pub type AcademicYearId = Uuid;` alias in `aggregate.rs` collapses the typed wrapper `Id<AcademicYear>` defined in `educore-academic::value_objects.rs` down to a bare `Uuid`. This means the documents crate's `(school_id, academic_id, reference_no)` uniqueness invariant can be silently violated if a caller passes a Uuid belonging to a different aggregate (e.g. an `EventId` Uuid mistakenly typed as `AcademicYearId`).

**Expected:**

AGENTS.md § "Engine Rules" rule 2: "Compile-time safety over strings. Use macro-generated enums (`StudentField::Status`) — never string field names." The typed-id pattern is the engine's defense against cross-aggregate Uuid confusion.

**Evidence:**

`crates/domains/documents/src/aggregate.rs:702` `pub type AcademicYearId = Uuid;`. The same alias is referenced in `crates/domains/documents/src/services.rs:314` `use crate::aggregate::{AcademicYearId, NewPostalDispatch, ...}`. The real `AcademicYearId` from `educore-academic` is a typed `Id<AcademicYear>` wrapper per the comment at `aggregate.rs:700-701`.

---

### FINDING DOMAIN-DOC-015 (id: `DOMAIN-DOC-015`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** crates/domains/documents/src/lib.rs:195-197

**Description:**

`lib.rs` uses `unreachable!()` inside `#[test]`-marked functions (lines 195-197). While `unreachable!()` is allowed by lint configuration in `#[cfg(test)]` blocks, it is technically a `panic!` form and is on the audit checklist. Test code is exempted from the AGENTS.md anti-pattern ban; this is logged for completeness.

**Expected:**

AGENTS.md § "Type Safety" bans `panic!` in production paths (test code exempt).

**Evidence:**

`crates/domains/documents/src/lib.rs:195-197`: `let _: fn() -> FormDownloadQuery = || unreachable!();` and analogous lines for the other two query types. Each is in a `#[test] fn prelude_query_structs_resolve()` function.

---

### FINDING DOMAIN-DOC-016 (id: `DOMAIN-DOC-016`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** crates/domains/documents/src/{aggregate,commands,entities,events,query,repository,services,value_objects}.rs

**Description:**

Every domain source file carries `#![allow(missing_docs)]` at module scope, which neuters the crate-level `#![deny(missing_docs)]` set in `lib.rs:8`. While individual files still declare doc comments, the explicit `allow` defeats the engine rule.

**Expected:**

AGENTS.md § "Engine Rules" mandates `#![deny(missing_docs)]` and "All public APIs are documented with rustdoc." The crate-level deny should be the enforcement mechanism.

**Evidence:**

`crates/domains/documents/src/lib.rs:8` has `#![deny(missing_docs)]`. Every other source file has `#![allow(missing_docs)]`: `aggregate.rs:17`, `commands.rs:4`, `entities.rs:38`, `events.rs:4`, `query.rs:4`, `repository.rs:4`, `services.rs:4`, `value_objects.rs:20`.

---

### FINDING DOMAIN-DOC-027 (id: `DOMAIN-DOC-027`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** crates/domains/documents/src/services.rs:126,461,680

**Description:**

The `snapshot`, `snapshot_dispatch`, `snapshot_receive` helpers in `services.rs` use `serde_json::to_vec(...).unwrap_or_default()` in non-test production code. This is a fall-back-to-empty pattern that hides serialization errors from the audit row pipeline. AGENTS.md § "Type Safety" expects all fallible APIs to return `Result`; using `unwrap_or_default()` here is a silent-failure pattern in the audit path.

**Expected:**

AGENTS.md § "Type Safety": "All fallible APIs return `Result` for fallible operations. Use `anyhow::Result` as the default surface." A snapshot helper should propagate the JSON error or be wrapped in a fallible signature.

**Evidence:**

`crates/domains/documents/src/services.rs:126` `Bytes::from(serde_json::to_vec(form).unwrap_or_default())`. Line 461: `Bytes::from(serde_json::to_vec(dispatch).unwrap_or_default())`. Line 680: `Bytes::from(serde_json::to_vec(receive).unwrap_or_default())`. All outside `#[cfg(test)]`.

---

### FINDING DOMAIN-DOC-028 (id: `DOMAIN-DOC-028`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** crates/domains/documents/src/services.rs:1400

**Description:**

The test helper `FactoryTestFormRepo::count` uses an `as u64` cast: `Ok(self.rows.lock().unwrap().len() as u64)`. The cast is inside `#[cfg(test)]`, so it is exempt from the AGENTS.md ban on `as` on numerics in production paths. Documented for completeness.

**Expected:**

AGENTS.md § "Type Safety": "Numeric conversions use `TryFrom`/`TryInto`; `as` on numerics is forbidden" in production paths. Test code is exempt.

**Evidence:**

`crates/domains/documents/src/services.rs:1400` `Ok(self.rows.lock().unwrap().len() as u64)`. Inside `mod tests` block (line 902).

---

### FINDING DOMAIN-DOC-030 (id: `DOMAIN-DOC-030`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** Low
- **Area:** docs-vs-code
- **Location:** docs/specs/documents/tables.md:18-28

**Description:**

`tables.md` notes every school-scoped table includes `school_id` (`NOT NULL DEFAULT 1` for the bootstrap school). The aggregate structs in `aggregate.rs` derive `school_id` from `id.school_id()` (e.g. `aggregate.rs:148` `school_id: cmd.id.school_id()`) rather than reading it from the row. While this is internally consistent, the `school_id` is also stored as a typed-field on the aggregate which means persistence must round-trip it; the spec note about `DEFAULT 1` is not preserved in any DB schema emitted by the crate (since there is no DDL emission).

**Expected:**

The spec note that school_id has `DEFAULT 1` is a DB-side invariant. There is no DDL emission in the crate (per AGENTS.md runtime DDL is the adapter's job), so this note is informational.

**Evidence:**

`docs/specs/documents/tables.md:15-18` "the `school_id` is `NOT NULL DEFAULT 1` for the bootstrap school". `crates/domains/documents/src/aggregate.rs:148` `school_id: cmd.id.school_id()`. No `migrations/engine/00xx_*_documents_*.sql` exists (`ls migrations/engine/`).

---

### FINDING DOMAIN-DOC-032 (id: `DOMAIN-DOC-032`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** Low
- **Area:** docs-vs-code
- **Location:** docs/build-plan.md:1356-1359

**Description:**

Build-plan § Phase 11 outcome states: "~915 tests pass workspace-wide (was ~770 at Phase 10 close-out; +145 net new in Phase 11: 145 unit tests in `educore-documents` + 6 integration scenarios + 1 rbac 15-cap test + 1 audit 3-variant test + test fixups)." The 6-scenario integration test exists at `crates/tools/storage-parity/tests/documents_integration.rs` but the per-file test breakdown (services.rs: 18) is incorrect — the actual count is 27 (see DOMAIN-DOC-021).

**Expected:**

Build-plan should accurately reflect test counts.

**Evidence:**

`docs/build-plan.md:1356-1359`. `grep -c "#\[test\]\|#\[tokio::test\]" crates/domains/documents/src/services.rs` returns 27.

---

### FINDING DOMAIN-DOC-034 (id: `DOMAIN-DOC-034`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** crates/domains/documents/src/commands.rs:3

**Description:**

`commands.rs` has `#![allow(dead_code, clippy::all)]` at module scope, suppressing the `dead_code` lint for the entire module. AGENTS.md § "Type Safety" prohibits `#[allow(dead_code)]` or `_var` prefixes; while the ban is targeted at unused-variable prefixes, the module-wide `dead_code` allow effectively silences dead-code detection across the entire file.

**Expected:**

AGENTS.md § "Type Safety": "No `#[allow(dead_code)]` or `_var` prefixes to silence the compiler. Delete unused code, wire it in, or open a follow-up issue."

**Evidence:**

`crates/domains/documents/src/commands.rs:3` `#![allow(dead_code, clippy::all)]`. The same pattern is in `events.rs:3`, `aggregate.rs:16`, `query.rs:3`, `repository.rs:3`, `services.rs:3`, `value_objects.rs:20-21`.

---

### FINDING DOMAIN-DOC-035 (id: `DOMAIN-DOC-035`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** Low
- **Area:** docs-vs-code
- **Location:** docs/specs/documents/commands.md:165-177

**Description:**

The spec mandates `pub struct TrackPostalCommand` with capability `Postal.Read`. The code uses `Capability::PostalRead` (line 867 of services.rs) which matches the spec wording. No drift here; logged as a positive confirmation.

**Expected:**

`docs/specs/documents/commands.md:174` `**Capability:** \`Postal.Read\``.

**Evidence:**

`crates/domains/documents/src/services.rs:867` `require_capability(cap, &cmd.tenant, Capability::PostalRead).await?;`. This is the only capability name in the codebase that matches the spec's `<Domain>.<Aggregate>.<Action>` form verbatim.

---

### FINDING DOMAIN-DOC-036 (id: `DOMAIN-DOC-036`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** Low
- **Area:** docs-vs-code
- **Location:** crates/domains/documents/src/aggregate.rs:472-477, 786-791

**Description:**

The aggregate docs reference "the Postal Dispatch Tracking workflow, step 3" and "the Postal Receive Tracking workflow, step 3" as the source for the `reference_no` immutability invariant. The workflows file `docs/specs/documents/workflows.md` does indeed describe those workflows at lines 32-46 (`## Postal Dispatch Tracking`) and 48-62 (`## Postal Receive Tracking`), and step 3 of each says "Reception updates the dispatch/receive (`UpdatePostalDispatch`/`UpdatePostalReceive`) when the address or note changes. The reference number is immutable." The references resolve correctly.

**Expected:**

The cross-references should match the spec text exactly.

**Evidence:**

`crates/domains/documents/src/aggregate.rs:474` reads "immutable once set (per the Postal Dispatch Tracking workflow, step 3)". `crates/domains/documents/src/aggregate.rs:788` reads "immutable once set (per the Postal Receive Tracking workflow, step 3)". `docs/specs/documents/workflows.md:40-41` is step 3 of Postal Dispatch Tracking; `docs/specs/documents/workflows.md:56-57` is step 3 of Postal Receive Tracking. No drift; cross-references are accurate.

---

### FINDING DOMAIN-DOC-038 (id: `DOMAIN-DOC-038`)

- **Source:** `docs/audit_reports/findings/wave1-documents.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** crates/domains/documents/src/services.rs:487-489

**Description:**

The capability names `Capability::PostalDispatchCreate` and `Capability::PostalReceiveCreate` use the verb `Create`, but the spec uses `Dispatch` and `Receive` respectively. The verbs `Create` and `Dispatch` are not synonymous in this domain: dispatching means recording a sent letter, while creating means a generic insert. The drift implies a weaker action than what the spec describes.

**Expected:**

`docs/specs/documents/permissions.md:28` mandates `Postal.Dispatch`. `docs/specs/documents/commands.md:75` `**Capability:** \`Postal.Dispatch\``.

**Evidence:**

`crates/domains/documents/src/services.rs:487` `require_capability(cap, &cmd.tenant, Capability::PostalDispatchCreate).await?;` vs `docs/specs/documents/commands.md:75` which says the capability is `Postal.Dispatch`. The rbac enum at `crates/cross-cutting/rbac/src/value_objects.rs:722` defines `PostalDispatchCreate`, not `Postal.Dispatch`.

---


## Events (domain) (target id prefix: `DOMAIN-EVD`)

**Path:** `crates/domains/events-domain/`  
**Total findings:** 60 (23 critical, 9 high, 11 medium, 17 low)


### FINDING DOMAIN-EVD-001 (id: `DOMAIN-EVD-001`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/aggregate.rs:7` and `crates/cross-cutting/events-domain/src/entities.rs:14`

**Description:**

The crate's two largest domain modules disable every lint — `#![allow(missing_docs, dead_code, clippy::all)]` — at the file level. This silently overrides `lib.rs`'s `#![deny(missing_docs)]` (lib.rs:24) and hides the fact that large swaths of public API are undocumented.

**Expected:**

Engine rule in `AGENTS.md`: "All public APIs are documented with rustdoc; `#![deny(missing_docs)]`." `lib.rs:24` enforces this for the crate. The deny is intentionally a crate-root lint.

**Evidence:**

- `crates/cross-cutting/events-domain/src/lib.rs:23-24` — `#![forbid(unsafe_code)] / #![deny(missing_docs)]`
  - `crates/cross-cutting/events-domain/src/aggregate.rs:7` — `#![allow(missing_docs, dead_code, clippy::all)]`
  - `crates/cross-cutting/events-domain/src/entities.rs:14` — `#![allow(missing_docs, dead_code, clippy::all)]`

---

### FINDING DOMAIN-EVD-002 (id: `DOMAIN-EVD-002`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/{commands.rs:7,events.rs:8,query.rs:8,repository.rs:8,services.rs:8,value_objects.rs:7}.rs`

**Description:**

Every remaining source file (6 of 10) carries `#![allow(missing_docs)]` at file scope. Together with FINDING-DOMAIN-EVD-001, all 9 domain-code files in the crate silently disable the `missing_docs` deny at lib.rs:24, meaning the engine's "All public APIs are documented with rustdoc" rule cannot fire inside this crate.

**Expected:**

Per `lib.rs:24`, public items must carry rustdoc; the crate-level deny exists precisely to enforce this.

**Evidence:**

File-level allows in all 6 files (e.g., `crates/cross-cutting/events-domain/src/events.rs:8` — `#![allow(missing_docs)]`).

---

### FINDING DOMAIN-EVD-003 (id: `DOMAIN-EVD-003`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/value_objects.rs` (entire file)

**Description:**

The spec's `docs/specs/events/value-objects.md` line 93 mandates that "All value objects implement `Validate` and refuse construction when validation fails," and provides a canonical `pub trait Validate { fn validate(&self) -> Result<(), ValueError>; }` at lines 97-100. The events-domain crate defines no `Validate` trait, no `ValueError`, and no `impl Validate for *` anywhere. Construction-time validation is implemented ad-hoc inside each aggregate `::new` (e.g., `aggregate.rs:84-89`), not via a shared trait.

**Expected:**

Spec text: "All value objects implement `Validate` and refuse construction when validation fails." + the canonical `Validate` trait definition (lines 96-100 of value-objects.md).

**Evidence:**

- Spec: `docs/specs/events/value-objects.md:93-100` — `pub trait Validate { fn validate(&self) -> Result<(), ValueError>; }`
  - Code: `grep -rn "trait Validate" crates/cross-cutting/events-domain/` returns no matches.
  - Code: ad-hoc validation only — `crates/cross-cutting/events-domain/src/aggregate.rs:84-89` (CalendarEvent::new empty-title check) and equivalent checks in Holiday/Incident/AssignIncident/IncidentComment/Weekend/CalendarSetting.

---

### FINDING DOMAIN-EVD-004 (id: `DOMAIN-EVD-004`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/aggregate.rs` (CalendarEvent, Holiday, Incident, AssignIncident, IncidentComment, CalendarSetting, Weekend structs); `crates/cross-cutting/events-domain/src/commands.rs` (CreateEventCommand, CreateHolidayCommand, etc.)

**Description:**

The spec's value-objects table (value-objects.md lines 24-89) mandates typed newtype wrappers — `EventTitle`, `EventDescription`, `EventLocation`, `HolidayTitle`, `HolidayDetails`, `WeekendName`, `WeekendOrder`, `IsWeekend`, `IncidentTitle`, `IncidentDescription`, `IncidentPoint`, `IncidentCommentBody`, `CalendarMenuName`, `CssColor`, `FontColor`, `BackgroundColor`, `EventDateRange`, `DateRange`, `RoleIdList`, `WeekendDay`, `EventDate`, `AcademicYearId`. The crate uses raw `String`, `Option<String>`, `i32`, `bool`, and `NaiveDate` everywhere instead. None of the spec-named types exist as Rust types.

**Expected:**

Spec table at `docs/specs/events/value-objects.md:24-89` lists 20+ typed value objects with constraints (e.g., "EventTitle — 1..200 chars", "IncidentPoint — i32 in 0..1000"). Spec example at line 105: `let title = IncidentTitle::new("Bullying in classroom 3B")?;`

**Evidence:**

- `crates/cross-cutting/events-domain/src/aggregate.rs:37-44` — CalendarEvent uses `title: String`, `location: Option<String>`, `description: Option<String>`, `role_ids: Vec<String>`.
  - `crates/cross-cutting/events-domain/src/aggregate.rs:406` — Incident uses `point: i32` (not `IncidentPoint`).
  - `crates/cross-cutting/events-domain/src/aggregate.rs:542` — AssignIncident uses `student_id: Option<Uuid>`, `user_id: Option<Uuid>` (not `Option<StudentId>`, `Option<UserId>`).
  - `crates/cross-cutting/events-domain/src/commands.rs:28-41` — CreateEventCommand uses `title: String` instead of `event_title: EventTitle`.
  - `grep -n "EventTitle\|EventDescription\|EventLocation\|HolidayTitle\|HolidayDetails\|WeekendName\|IncidentTitle\|IncidentDescription\|IncidentPoint\|IncidentCommentBody\|CalendarMenuName\|CssColor\|FontColor\|BackgroundColor\|DateRange\|WeekendDay\|RoleIdList" crates/cross-cutting/events-domain/src/` — zero matches outside comments.

---

### FINDING DOMAIN-EVD-005 (id: `DOMAIN-EVD-005`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/events.rs:28-37` (EventCreated struct)

**Description:**

The spec mandates `EventCreated` to carry `event_title: EventTitle`, `for_whom: ForWhom`, and `academic_id: AcademicYearId`. The code's `EventCreated` carries `title: String` (wrong type per FINDING-004) and is missing both `for_whom` and `academic_id`. The event that fans out to the communication domain for notifications therefore cannot carry the audience scope it advertises in `events.md` lines 51-54.

**Expected:**

Spec at `docs/specs/events/events.md:38-46`: `pub struct EventCreated { pub event_id, pub event_title: EventTitle, pub from_date, pub to_date, pub for_whom: ForWhom, pub academic_id: AcademicYearId, }`

**Evidence:**

- Spec: `docs/specs/events/events.md:38-46` — `for_whom: ForWhom` and `academic_id: AcademicYearId` are mandatory fields.
  - Code: `crates/cross-cutting/events-domain/src/events.rs:28-37` — `EventCreated` has `event_id, school_id, title, from_date, to_date, event_id_field, correlation_id, occurred_at`. No `for_whom`. No `academic_id`.

---

### FINDING DOMAIN-EVD-006 (id: `DOMAIN-EVD-006`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/events.rs:808-817` (IncidentAssigned struct)

**Description:**

The spec mandates `IncidentAssigned` to carry `student_id: Option<StudentId>` and `user_id: Option<UserId>` so that the event records which actor was assigned. The code's `IncidentAssigned` is missing both fields, so a downstream subscriber (e.g., HR for behavior-note attachment) cannot tell from the event who was assigned.

**Expected:**

Spec at `docs/specs/events/events.md:117-126`: `pub struct IncidentAssigned { pub assign_incident_id, pub incident_id, pub student_id: Option<StudentId>, pub user_id: Option<UserId>, pub point: IncidentPoint, pub added_by: UserId, }`

**Evidence:**

- Spec: `docs/specs/events/events.md:117-126`
  - Code: `crates/cross-cutting/events-domain/src/events.rs:808-817` — fields: `assign_incident_id, incident_id, school_id, point, added_by, event_id_field, correlation_id, occurred_at`. No `student_id`. No `user_id`.

---

### FINDING DOMAIN-EVD-007 (id: `DOMAIN-EVD-007`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/events.rs:978-987` (IncidentCommented struct)

**Description:**

The spec mandates `IncidentCommented` to carry the actual `comment: IncidentCommentBody` text and the `commented_at: Timestamp`. The code's `IncidentCommented` carries neither — so the subscriber that ingests "A comment was added to an incident" (per `docs/events/events.md:27`) cannot read the comment body or the wall-clock time of the comment.

**Expected:**

Spec at `docs/specs/events/events.md:144-150`: `pub struct IncidentCommented { pub incident_comment_id, pub incident_id, pub user_id, pub comment: IncidentCommentBody, pub commented_at: Timestamp, }`

**Evidence:**

- Spec: `docs/specs/events/events.md:144-150`
  - Code: `crates/cross-cutting/events-domain/src/events.rs:978-987` — fields: `incident_comment_id, incident_id, school_id, user_id, event_id_field, correlation_id, occurred_at`. No `comment`. No `commented_at`.

---

### FINDING DOMAIN-EVD-008 (id: `DOMAIN-EVD-008`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/commands.rs:304-308` (ResolveIncidentCommand struct)

**Description:**

The spec's `ResolveIncidentCommand` must carry `pub resolution_note: Option<IncidentCommentBody>` so the discipline lead can attach an immutable note when transitioning to Resolved. The workflows.md § Incident Resolution Workflow (line 102-103) names "with a note" as part of the step. The code's `ResolveIncidentCommand` has no such field, and the `Incident::resolve` method (aggregate.rs:510) does not accept one either.

**Expected:**

Spec at `docs/specs/events/commands.md:222-235`: `pub struct ResolveIncidentCommand { pub tenant: TenantContext, pub incident_id: IncidentId, pub resolution_note: Option<IncidentCommentBody>, }` and workflows.md § Incident Resolution Workflow at line 102-103: "The lead resolves the incident (ResolveIncident) with a note."

**Evidence:**

- Spec: `docs/specs/events/commands.md:228` — `pub resolution_note: Option<IncidentCommentBody>`
  - Code: `crates/cross-cutting/events-domain/src/commands.rs:305-308` — `pub tenant: TenantContext, pub incident_id: IncidentId` — only two fields, no `resolution_note`.
  - Code: `crates/cross-cutting/events-domain/src/aggregate.rs:510-518` — `Incident::resolve(&mut self, _actor: UserId, at: Timestamp)` — no note parameter; the `_actor` argument is even marked unused.

---

### FINDING DOMAIN-EVD-009 (id: `DOMAIN-EVD-009`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/commands.rs:72-79` (UpdateEventCommand struct)

**Description:**

The spec mandates `UpdateEventCommand` to carry optional patch fields for every mutable property of a CalendarEvent: `event_title`, `for_whom`, `role_ids`, `url`, `event_location`, `event_des`, `upload_image`. The code's `UpdateEventCommand` only has `title`, `from_date`, `to_date` — missing 6 of the 8 spec-mandated optional fields. Consumers cannot patch `for_whom`, the audience `role_ids`, the `url`, the `location`, the `description`, or the `image`.

**Expected:**

Spec at `docs/specs/events/commands.md:37-51` — `pub event_title: Option<EventTitle>, pub for_whom: Option<ForWhom>, pub role_ids: Option<Vec<RoleId>>, pub url: Option<Url>, pub event_location: Option<EventLocation>, pub event_des: Option<EventDescription>, pub upload_image: Option<FileReference>`

**Evidence:**

- Spec: `docs/specs/events/commands.md:37-51` lists 8 patch fields.
  - Code: `crates/cross-cutting/events-domain/src/commands.rs:73-79` — `title`, `from_date`, `to_date` only.

---

### FINDING DOMAIN-EVD-010 (id: `DOMAIN-EVD-010`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/commands.rs:140-147` (UpdateHolidayCommand struct)

**Description:**

The spec's `UpdateHolidayCommand` must carry `details: Option<HolidayDetails>` and `upload_image: Option<FileReference>` as patch fields. The code's `UpdateHolidayCommand` has neither, so the holiday's narrative `details` text and the optional attachment cannot be patched after creation.

**Expected:**

Spec at `docs/specs/events/commands.md:88-100` lists 6 optional patch fields including `details` and `upload_image`.

**Evidence:**

- Spec: `docs/specs/events/commands.md:88-100`
  - Code: `crates/cross-cutting/events-domain/src/commands.rs:141-147` — `title`, `from_date`, `to_date` only. No `details`. No `image`.

---

### FINDING DOMAIN-EVD-011 (id: `DOMAIN-EVD-011`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/commands.rs:335-341` (AssignIncidentCommand struct) and `crates/cross-cutting/events-domain/src/aggregate.rs:537-555` (AssignIncident struct)

**Description:**

The spec mandates `AssignIncidentCommand` and `AssignIncident` aggregate to carry `record_id: Option<StudentRecordId>` — a foreign key into `student_records.id` that anchors the assignment to a specific academic-year scope. The code carries no `record_id` anywhere, so the spec invariant 3 of the AssignIncident aggregate ("The `record_id` references a `StudentRecord` from the academic domain at the time of the incident") cannot be enforced.

**Expected:**

Spec at `docs/specs/events/commands.md:167-178`: `pub struct AssignIncidentCommand { ..., pub record_id: Option<StudentRecordId>, ... }`. Spec at `docs/specs/events/aggregates.md:174-180`: "The record_id references a StudentRecord (from the academic domain) at the time of the incident."

**Evidence:**

- Spec: `docs/specs/events/commands.md:175` — `pub record_id: Option<StudentRecordId>`
  - Code: `crates/cross-cutting/events-domain/src/commands.rs:335-341` — fields: `tenant, incident_id, student_id, user_id, point`. No `record_id`.
  - Code: `crates/cross-cutting/events-domain/src/aggregate.rs:537-555` — AssignIncident struct has no `record_id` field.

---

### FINDING DOMAIN-EVD-012 (id: `DOMAIN-EVD-012`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/events.rs:193-202` (HolidayCreated struct)

**Description:**

The spec mandates `HolidayCreated` to carry `academic_id: AcademicYearId` to scope the holiday to an academic year (per aggregates.md § Holiday invariant 3: "A Holiday is anchored to a school and an academic year"). The code's `HolidayCreated` carries no `academic_id`, so the attendance domain subscriber (per `docs/events/events.md:13`) cannot scope holiday overrides to the correct academic year.

**Expected:**

Spec at `docs/specs/events/events.md:58-65`: `pub struct HolidayCreated { pub holiday_id, pub holiday_title: HolidayTitle, pub from_date, pub to_date, pub academic_id: AcademicYearId, }`

**Evidence:**

- Spec: `docs/specs/events/events.md:64`
  - Code: `crates/cross-cutting/events-domain/src/events.rs:193-202` — HolidayCreated fields: `holiday_id, school_id, title, from_date, to_date, event_id_field, correlation_id, occurred_at`. No `academic_id`.

---

### FINDING DOMAIN-EVD-013 (id: `DOMAIN-EVD-013`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/services.rs` (entire file)

**Description:**

The spec's `services.md` mandates three additional service types that the code does not implement: (a) Specification `ActiveIncidents` filtering `Status != Resolved` (services.md lines 84-94); (b) Specification `EventsInMonth` matching events that overlap a month (lines 97-110); (c) Policy `IncidentPointLimit` capping points per incident (lines 113-126). None of the three exist in the code. No `Specification` or `Policy` traits are imported or implemented anywhere in the crate.

**Expected:**

Spec at `docs/specs/events/services.md:84-126` defines three service types using the canonical patterns `impl Specification<Incident> for ActiveIncidents` and `impl Policy<AssignIncidentCommand> for IncidentPointLimit`.

**Evidence:**

- Spec: `docs/specs/events/services.md:84-126`
  - Code: `grep -rn "Specification\|Policy\|ActiveIncidents\|EventsInMonth\|IncidentPointLimit" crates/cross-cutting/events-domain/src/` returns no matches.

---

### FINDING DOMAIN-EVD-014 (id: `DOMAIN-EVD-014`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/services.rs:128-153` (HolidayService::is_instructional and related)

**Description:**

`HolidayService::is_instructional` (services.rs:128-133) and `WeekendService::is_weekend` (services.rs:300-303) compare the weekday ordinal `date.weekday().num_days_from_monday() as i32` against `Weekend::order` directly, bypassing the `WeekendOrder` newtype (per spec value-objects.md line 40, "i32 in 0..7"). The `WeekendOrder` type does not exist in the code (FINDING-DOMAIN-EVD-004), so the `0..7` invariant cannot be enforced at construction; a `Weekend` row with `order = 100` constructed via the SQL adapter would silently corrupt the is-instructional predicate.

**Expected:**

Spec at `docs/specs/events/value-objects.md:40` — `WeekendOrder | i32 in 0..7` (table cell). Spec at `docs/specs/events/aggregates.md:101` — Invariant 2: "The order field is a positive integer; lower orders sort first in a UI."

**Evidence:**

- Spec: `docs/specs/events/value-objects.md:40`
  - Code: `crates/cross-cutting/events-domain/src/services.rs:129` — `let weekday = date.weekday().num_days_from_monday() as i32;` then `w.order == weekday`.
  - Code: `crates/cross-cutting/events-domain/src/aggregate.rs:696` — `Weekend::order` is plain `i32` (not `WeekendOrder`).
  - Code: `crates/cross-cutting/events-domain/src/aggregate.rs:726` — The `Weekend::new` constructor checks `0..=7` against a raw `i32`, not against a typed `WeekendOrder`.

---

### FINDING DOMAIN-EVD-015 (id: `DOMAIN-EVD-015`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/services.rs` (entire file)

**Description:**

The spec's `services.md` § "Cross-Domain Coordinator" (lines 128-147) mandates an `EventsCoordinator` struct in the engine facade that orchestrates multi-domain flows such as "report_and_notify". No `EventsCoordinator` exists in the code, no `engine.events()` facade is referenced, and no async coordination logic exists.

**Expected:**

Spec at `docs/specs/events/services.md:128-147`: `pub struct EventsCoordinator<'a> { engine: &'a Engine }` with `pub async fn report_and_notify(&self, cmd: CreateIncidentCommand) -> Result<Incident, DomainError>`.

**Evidence:**

- Spec: `docs/specs/events/services.md:128-147`
  - Code: `grep -rn "EventsCoordinator\|report_and_notify" crates/cross-cutting/events-domain/` returns no matches.

---

### FINDING DOMAIN-EVD-016 (id: `DOMAIN-EVD-016`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/services.rs:212-215` (IncidentService::total_points)

**Description:**

The spec mandates `IncidentService::total_points(assignments: &[AssignIncident]) -> i32` to sum the points attributed across all assignments. The code's signature is `total_points(points: &[i32]) -> i32` — it takes a pre-flattened slice of integers, not the assignment aggregate itself. Callers cannot use the spec-shaped API, and the service cannot enforce invariants on the input (e.g., rejection of duplicate student assignments) because it has no view of the aggregate.

**Expected:**

Spec at `docs/specs/events/services.md:42-51`: `pub fn total_points(assignments: &[AssignIncident]) -> i32`

**Evidence:**

- Spec: `docs/specs/events/services.md:47`
  - Code: `crates/cross-cutting/events-domain/src/services.rs:213-215` — `pub fn total_points(points: &[i32]) -> i32`

---

### FINDING DOMAIN-EVD-017 (id: `DOMAIN-EVD-017`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/services.rs` (IncidentService impl block, lines 199-222)

**Description:**

The spec mandates `IncidentService::participants(assignments: &[AssignIncident]) -> Vec<UserReference>` to enumerate the assigned students/staff. The method does not exist in the code.

**Expected:**

Spec at `docs/specs/events/services.md:42-51`: `pub fn participants(assignments: &[AssignIncident]) -> Vec<UserReference>`

**Evidence:**

- Spec: `docs/specs/events/services.md:48`
  - Code: `grep -n "fn participants" crates/cross-cutting/events-domain/src/` returns no matches.

---

### FINDING DOMAIN-EVD-018 (id: `DOMAIN-EVD-018`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/services.rs:62-64` (CalendarService::visible_to)

**Description:**

The spec's `CalendarService::visible_to` signature is `pub fn visible_to(event: &CalendarEvent, actor: &ActorRoles) -> bool` — it takes the entire event plus a typed `ActorRoles` bundle. The code's signature is `pub fn visible_to(for_whom: ForWhom, role_ids: &[String], actor_roles: &[String]) -> bool` — the caller has to extract the audience fields and supply raw `String` slices. `ActorRoles` (the typed wrapper the spec mandates) does not exist in the code.

**Expected:**

Spec at `docs/specs/events/services.md:8-17`: `pub fn visible_to(event: &CalendarEvent, actor: &ActorRoles) -> bool`

**Evidence:**

- Spec: `docs/specs/events/services.md:15`
  - Code: `crates/cross-cutting/events-domain/src/services.rs:62-64` — `pub fn visible_to(for_whom: ForWhom, role_ids: &[String], actor_roles: &[String]) -> bool`
  - Code: `grep -n "ActorRoles" crates/cross-cutting/events-domain/` returns no matches.

---

### FINDING DOMAIN-EVD-019 (id: `DOMAIN-EVD-019`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/repository.rs:34-63` (CalendarEventRepository trait)

**Description:**

The spec mandates `CalendarEventRepository` to expose 9 methods; the code's trait exposes only 7. The two missing methods are `count(&self, school: SchoolId, q: CalendarEventQuery) -> Result<u64>` (for cardinality queries without loading rows) and `page(&self, school: SchoolId, q: CalendarEventQuery, offset: u32, limit: u32) -> Result<Page<CalendarEvent>>` (for paginated reads).

**Expected:**

Spec at `docs/specs/events/repositories.md:9-22`: the `count` and `page` methods are the 8th and 9th methods.

**Evidence:**

- Spec: `docs/specs/events/repositories.md:19-20`
  - Code: `crates/cross-cutting/events-domain/src/repository.rs:34-63` — only 7 methods; no `count`, no `page`.

---

### FINDING DOMAIN-EVD-020 (id: `DOMAIN-EVD-020`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/repository.rs:149-171` (IncidentRepository trait)

**Description:**

The spec mandates `IncidentRepository` to expose 11 methods; the code's trait exposes only 8. Missing methods: `resolve(&self, id: IncidentId) -> Result<()>` (so a resolved incident can be persisted atomically with the IncidentResolved event), `by_student(&self, school: SchoolId, student: StudentId) -> Result<Vec<Incident>>`, and `by_user(&self, school: SchoolId, user: UserId) -> Result<Vec<Incident>>`.

**Expected:**

Spec at `docs/specs/events/repositories.md:57-69`

**Evidence:**

- Spec: `docs/specs/events/repositories.md:62, 66-67`
  - Code: `crates/cross-cutting/events-domain/src/repository.rs:149-171` — only `get, list, insert, update, delete, open, in_progress, between`.

---

### FINDING DOMAIN-EVD-021 (id: `DOMAIN-EVD-021`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/aggregate.rs` (every aggregate struct, plus constructor bodies)

**Description:**

The crate declares `last_event_id: Option<EventId>` on every aggregate (CalendarEvent:57, Holiday:212, CalendarSetting:300, Incident:416, AssignIncident:553, IncidentComment:640, Weekend:706) but no code path ever assigns it. Every constructor initializes it to `None` and no command/event helper writes to it. The `audit_log` and outbox integration mandated by the engine rule "Audit-first. Every state change writes an immutable record" (`AGENTS.md` engine rules) is therefore not implemented — the field is a placeholder with no writer.

**Expected:**

Engine rule in `AGENTS.md`: "Audit-first. Every state change writes an immutable record." Spec at `docs/specs/events/workflows.md:138-140`: "Every state-changing command writes a durable audit record with the actor, the correlation id, and a hash of the payload."

**Evidence:**

- `crates/cross-cutting/events-domain/src/aggregate.rs:57, 123, 212, 260, 300, 341, 416, 464, 553, 600, 640, 671, 706, 745` — every aggregate declares `last_event_id: Option<EventId>` and initializes it to `None`.
  - `grep -rn "last_event_id = Some" crates/cross-cutting/events-domain/src/` returns no matches.

---

### FINDING DOMAIN-EVD-022 (id: `DOMAIN-EVD-022`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/` (entire crate)

**Description:**

The crate declares dependencies on `educore-audit` and `educore-events` (Cargo.toml:17,19), and the handoff at `docs/handoff/PHASE-13-HANDOFF.md:316-319` claims "4 net-new `Events*` AuditTarget variants in `educore-audit`". But the events-domain crate itself never imports or calls `educore_audit::write` (or equivalent) anywhere. There is no domain-side audit-writer glue, no `audit_write!` macro invocation, and no emission of an audit record from any command or service. The audit integration is one-sided (targets added to `educore-audit`, but no caller in events-domain).

**Expected:**

Engine rule "Audit-first." Spec workflows.md:138-140. Handoff claim at PHASE-13-HANDOFF.md:152-153.

**Evidence:**

- `crates/cross-cutting/events-domain/Cargo.toml:19` — `educore-audit = { workspace = true }`
  - `grep -rn "educore_audit\|audit::" crates/cross-cutting/events-domain/src/` returns no matches in non-test code.

---

### FINDING DOMAIN-EVD-023 (id: `DOMAIN-EVD-023`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/events.rs:1031-1081` (IncidentCommentDeletedEvent struct)

**Description:**

The spec mandates the type to be named `IncidentCommentDeleted` (events.md line 152, also `docs/commands/events.md:26`). The code names it `IncidentCommentDeletedEvent`, which diverges from the spec. Downstream subscribers and the round-trip test (`events_integration.rs:35`) are forced to use the renamed type. The wire-form `EVENT_TYPE` (`"events.incident_comment.deleted"`) does match the spec, so the rename is purely at the Rust type level — it is an avoidable drift from the public catalog.

**Expected:**

Spec at `docs/specs/events/events.md:152`: `pub struct IncidentCommentDeleted { ... }`. Catalog at `docs/events/events.md:28` and `docs/commands/events.md:26` use `IncidentCommentDeleted`.

**Evidence:**

- Spec: `docs/specs/events/events.md:152`
  - Code: `crates/cross-cutting/events-domain/src/events.rs:1032` — `pub struct IncidentCommentDeletedEvent { ... }`
  - Code: `crates/tools/storage-parity/tests/events_integration.rs:35` — imports `IncidentCommentDeletedEvent` (forced workaround for the rename).

---

### FINDING DOMAIN-EVD-024 (id: `DOMAIN-EVD-024`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/Cargo.toml:27`

**Description:**

The crate's direct dependency on `serde_json` (Cargo.toml:27) is a yellow flag under the engine rule "No `serde_json::Value` in domain code. Use typed wrappers." (`AGENTS.md` code standards). While the crate does not appear to use `serde_json::Value` directly (verified via `grep`), depending on `serde_json` for typed serde wrappers invites the anti-pattern. The spec mandates typed envelopes and wire-form `events.<aggregate>.<verb>` strings; the runtime JSON serialization belongs to the storage adapter layer, not the domain crate.

**Expected:**

Engine rule (AGENTS.md): "No `serde_json::Value` in domain code. Use typed wrappers." Domain crates should serialize via the typed `serde::{Serialize, Deserialize}` derives they already carry, with no JSON-specific dependency.

**Evidence:**

`crates/cross-cutting/events-domain/Cargo.toml:27` — `serde_json = { workspace = true }`.

---

### FINDING DOMAIN-EVD-025 (id: `DOMAIN-EVD-025`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/aggregate.rs:471-506` (Incident::update)

**Description:**

The spec's Incident invariant 5 (aggregates.md line 142-144) says "An Incident is immutable after `Status` is `Resolved` except for the `description` field (which may be annotated) and the comments list." The code's `Incident::update` rejects ALL updates after Resolved (`aggregate.rs:478-482`), so annotating the description on a resolved incident is impossible. This contradicts the spec and breaks the discipline-lead workflow described in workflows.md § Incident Resolution Workflow line 102-103 ("a note" attached at resolve time).

**Expected:**

Spec at `docs/specs/events/aggregates.md:142-144` — invariant 5 explicitly allows `description` annotation after Resolved. Spec workflows.md § Incident Resolution Workflow lines 102-103.

**Evidence:**

- Spec: `docs/specs/events/aggregates.md:142-144`
  - Code: `crates/cross-cutting/events-domain/src/aggregate.rs:478-482` — `if self.status == IncidentStatus::Resolved { return Err(EventsDomainError::Conflict("cannot update resolved incident".to_owned())); }` — applied unconditionally before any field-by-field check.

---

### FINDING DOMAIN-EVD-026 (id: `DOMAIN-EVD-026`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/value_objects.rs:171-178` (IncidentAction enum) and `value_objects.rs:154-160` (`IncidentStatus::next`)

**Description:**

`IncidentAction` declares a `Reopen` variant (value_objects.rs:177) but `IncidentStatus::next` (line 154) has no arm for `Reopen` — the match returns `current` for any unmatched action, so `Resolved.next(Reopen)` silently returns `Resolved`. The `Reopen` variant is therefore dead-coded at the state-machine level. No test exercises the reopen path.

**Expected:**

Per spec workflows.md and aggregates.md § Incident lifecycle, the state machine is `Open → InProgress → Resolved`. If `Reopen` is modeled, the engine rule says no dead code: "Delete unused code, wire it in, or open a follow-up issue." (`AGENTS.md` type-safety rules.)

**Evidence:**

- Code: `crates/cross-cutting/events-domain/src/value_objects.rs:171-178` — `pub enum IncidentAction { InProgress, Resolve, Reopen }`
  - Code: `crates/cross-cutting/events-domain/src/value_objects.rs:154-160` — `pub const fn next(self, action: IncidentAction) -> Self { match (self, action) { (Self::Open, IncidentAction::InProgress) => ..., (Self::InProgress, IncidentAction::Resolve) => ..., (current, _) => current } }` — no `Reopen` arm.
  - Code: `grep -rn "IncidentAction::Reopen" crates/cross-cutting/events-domain/` returns no matches outside the variant declaration.

---

### FINDING DOMAIN-EVD-027 (id: `DOMAIN-EVD-027`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/services.rs:262-295` (WeekendService::reconcile)

**Description:**

The function builds `current_names: HashSet<&str>` (line 263) but never reads it — the second loop (lines 286-292) iterates `current` and checks `proposed_names.contains(...)` instead. The unused `current_names` is only suppressed via `let _ = current_names;` at line 293. This is dead code that survives because `aggregate.rs:7` carries `#![allow(missing_docs, dead_code, clippy::all)]` and `services.rs:8` carries `#![allow(missing_docs)]`. Under the engine rule "No `#[allow(dead_code)]` or `_var` prefixes to silence the compiler. Delete unused code, wire it in, or open a follow-up issue." (`AGENTS.md`), this is a violation.

**Expected:**

Engine rule (AGENTS.md) type-safety section: no `_var` prefixes to silence the compiler; delete unused code.

**Evidence:**

- Code: `crates/cross-cutting/events-domain/src/services.rs:263` — `let current_names: std::collections::HashSet<&str> = current.iter().map(|w| w.name.as_str()).collect();`
  - Code: `crates/cross-cutting/events-domain/src/services.rs:293` — `let _ = current_names; // suppress unused`

---

### FINDING DOMAIN-EVD-028 (id: `DOMAIN-EVD-028`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/services.rs:129, 301`; `crates/cross-cutting/events-domain/src/value_objects.rs:361-363`

**Description:**

The crate uses raw `as i32` / `as i64` / `as u32` casts in production code paths. The engine rule in `AGENTS.md` code standards says "No `as` casts that truncate or lose data. Use `TryFrom` / `TryInto` with proper error handling." The numeric conversions on chrono's `Datelike::year() -> i32`, `month() -> u32`, `num_days_from_monday() -> u32` are all widening or value-preserving, but the rule's wording is absolute ("No `as` on numerics is forbidden" in the validation checklist).

**Expected:**

Engine rule (AGENTS.md) code standards: "Numeric conversions use `TryFrom`/`TryInto`; `as` on numerics is forbidden." Validation checklist: "No new `as` on numerics."

**Evidence:**

- `crates/cross-cutting/events-domain/src/services.rs:129` — `let weekday = date.weekday().num_days_from_monday() as i32;`
  - `crates/cross-cutting/events-domain/src/services.rs:301` — same cast.
  - `crates/cross-cutting/events-domain/src/value_objects.rs:361` — `let total = d.year() as i64 * 12 + d.month() as i64 - 1 + months;`
  - `crates/cross-cutting/events-domain/src/value_objects.rs:362-363` — `(total.div_euclid(12)) as i32`, `(total.rem_euclid(12) + 1) as u32`.

---

### FINDING DOMAIN-EVD-029 (id: `DOMAIN-EVD-029`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/services.rs:149` (HolidayService::instructional_days_in)

**Description:**

Production code path uses `.succ_opt().unwrap_or(current)` (services.rs:149). The `unwrap_or` branch is a non-panicking fallback, but `succ_opt` returning `None` is then silently swallowed — the loop stops at the same date, so the date-range walk silently truncates if any date is invalid (e.g., a malformed input that surfaced via the storage adapter). The engine rule says "No `unwrap`/`expect`/`panic` in production paths" — strictly, `unwrap_or` is not on the list, but the silent truncation is a related smell: an invalid date input causes an infinite `while current <= to` spin rather than a domain error.

**Expected:**

Engine rule (AGENTS.md): "No `unwrap()` or `expect()` in production paths. Propagate errors via `?` or document the invariant that makes panic impossible."

**Evidence:**

- Code: `crates/cross-cutting/events-domain/src/services.rs:143-151` — the loop body uses `current.succ_opt().unwrap_or(current)` to advance, with no error path.

---

### FINDING DOMAIN-EVD-030 (id: `DOMAIN-EVD-030`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/` (crate root) — no `tests/` directory

**Description:**

The crate has no `tests/` directory at `crates/cross-cutting/events-domain/tests/`. The handoff claim at `docs/handoff/PHASE-13-HANDOFF.md:36-38` says "34 unit tests in educore-events-domain + 7 always-on integration tests" — but the 7 always-on integration tests live in `crates/tools/storage-parity/tests/events_integration.rs`, not in the crate itself. There are no integration tests for the crate in its own `tests/` directory.

**Expected:**

Engine rule (AGENTS.md) validation checklist: "At least one integration test added for new behavior (per the per-PR gate). Unit tests alone are not sufficient." Spec format (§ Phase 13 outcome) expects 7 always-on integration tests at the crate's `tests/` path.

**Evidence:**

- `ls crates/cross-cutting/events-domain/tests/` — directory does not exist.
  - `crates/cross-cutting/events-domain/` directory listing shows only `Cargo.toml` and `src/`.
  - The 7 integration tests live in `crates/tools/storage-parity/tests/events_integration.rs:372, 512, 542, 580, 624, 641, 686`.

---

### FINDING DOMAIN-EVD-031 (id: `DOMAIN-EVD-031`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/tools/storage-parity/tests/events_integration.rs:735-746` (PG/MySQL env-gated tests)

**Description:**

The two `#[ignore]`-marked PG/MySQL integration tests are 1-line placeholder stubs: `let _school = SchoolId::from_uuid(uuid::Uuid::new_v4());`. They do not exercise the schema, the dialect translation, or any of the 24 events. The handoff claim "7 always-on integration tests + 2 env-gated `#[ignore]` PG/MySQL variants in `events_integration.rs`" implies the ignored tests have substance; they do not.

**Expected:**

Spec at `docs/build-plan.md` § Phase 13 outcome line 1501: "2 env-gated #[ignore] PG/MySQL variants" implies actual PG/MySQL adapter coverage. The PG/MySQL adapter crates (`educore-storage-postgres`, `educore-storage-mysql`) ship from Phase 1, so the env-gated tests should drive real round-trips on those adapters.

**Evidence:**

- `crates/tools/storage-parity/tests/events_integration.rs:735-740` — `events_integration_pg_vertical_slice` body is just `let _school = SchoolId::from_uuid(uuid::Uuid::new_v4());`.
  - `crates/tools/storage-parity/tests/events_integration.rs:742-746` — same placeholder for MySQL.

---

### FINDING DOMAIN-EVD-032 (id: `DOMAIN-EVD-032`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `docs/build-plan.md` § Phase 13 — Tasks (line 1470, 1475) vs Phase 13 outcome (line 1501)

**Description:**

Doc-vs-code drift. The build-plan's Phase 13 Tasks section (line 1466-1492) still describes the original prompt: "Aggregates per docs/specs/events/aggregates.md: CalendarEvent, Holiday, Incident, Weekend." (4 aggregates). The Phase 13 outcome section (line 1501), added at phase close, describes the 7-aggregate reality. Two contradictory statements in the same build-plan section; the Tasks list was never updated to match the implementation or the outcome note.

**Expected:**

Per `AGENTS.md` engine rules and the build-plan update instructions at lines 1486-1490 ("Update ... `**Phase 13 outcome.**` subsection to this build plan ..."), the Tasks section should reflect the implemented scope.

**Evidence:**

- `docs/build-plan.md:1470` — "Aggregates per docs/specs/events/aggregates.md:"
  - `docs/build-plan.md:1475` — "`CalendarEvent`, `Holiday`, `Incident`, `Weekend`."
  - `docs/build-plan.md:1501` — "**Spec-faithful 7-root interpretation** ... 7 root aggregates (CalendarEvent, Holiday, Weekend, Incident, AssignIncident, IncidentComment, CalendarSetting)."

---

### FINDING DOMAIN-EVD-033 (id: `DOMAIN-EVD-033`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/aggregate.rs:13` (unused import) and `crates/cross-cutting/events-domain/src/value_objects.rs:14` (unused `Datelike` import? actually it IS used)

**Description:**

`aggregate.rs:13` imports `use uuid::Uuid;` but `Uuid` is used inside the test module (lines 794-979). The import is necessary for tests but `dead_code` allow at module level hides that `Uuid` may be required by production code only via the implicit aggregate `id.value` Uuid field; verify. Several other imports — `EducorePlatform`, `UserType`, `RoleId`, `StudentId`, `StudentRecordId` — would be needed to type the spec fields, but they are absent because the fields are raw types (FINDING-DOMAIN-EVD-004).

**Expected:**

Engine rule (AGENTS.md): "No `#[allow(dead_code)]` ... to silence the compiler. Delete unused code, wire it in, or open a follow-up issue."

**Evidence:**

`crates/cross-cutting/events-domain/src/aggregate.rs:13` — `use uuid::Uuid;` — used in tests only (mod tests, line 794+).

---

### FINDING DOMAIN-EVD-034 (id: `DOMAIN-EVD-034`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/events.rs:7-8` and entire file (24 events, each with `event_id_field: EventId` plumbed manually)

**Description:**

Every event carries an extra `event_id_field: EventId` field that is duplicated by the `EventEnvelope` from `educore-events::domain_event::EventEnvelope` (per spec events.md lines 21-32, the envelope carries `event_id: EventId`). The crate does not use the `EventEnvelope<E>` wrapper at all — it passes 7-9 fields to `EventX::new(...)` constructors manually, then asks consumers to also wrap the event in their own envelope. The wire-form string (`EVENT_TYPE`) is implemented on the inner event, not the envelope. This deviates from the engine's `educore-events` Phase 2 envelope contract (per AGENTS.md: "two `events` crates — do NOT confuse").

**Expected:**

Spec at `docs/specs/events/events.md:18-33` — the canonical envelope wraps the event; the inner event should be the `payload` field. The `event_id` field lives on the envelope, not on the inner event payload.

**Evidence:**

- Spec: `docs/specs/events/events.md:21-32`
  - Code: `crates/cross-cutting/events-domain/src/events.rs:34-36` — `EventCreated` carries `event_id_field: EventId, correlation_id: CorrelationId, occurred_at: Timestamp` as payload fields, identical to envelope fields.
  - Code: every event in `events.rs` repeats `pub event_id_field: EventId, pub correlation_id: CorrelationId, pub occurred_at: Timestamp` in its payload (24 occurrences).

---

### FINDING DOMAIN-EVD-035 (id: `DOMAIN-EVD-035`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/commands.rs:473-484` (`fn _ensure_ids_compile`)

**Description:**

The `fn _ensure_ids_compile(school: SchoolId)` helper at commands.rs:473 carries `#[allow(dead_code)]` and exists only to silence "unused import" warnings for the typed IDs (CalendarEventId, HolidayId, CalendarSettingId, IncidentId, AssignIncidentId, IncidentCommentId, WeekendId). This is a documented anti-pattern under AGENTS.md ("No `#[allow(dead_code)]` ... to silence the compiler. Delete unused code, wire it in, or open a follow-up issue.").

**Expected:**

Either delete the unused imports, or wire the IDs into public API. `_ensure_ids_compile` is a code-smell flag.

**Evidence:**

- Code: `crates/cross-cutting/events-domain/src/commands.rs:473-484` — `#[allow(dead_code)] fn _ensure_ids_compile(...)` — the function is never called.

---

### FINDING DOMAIN-EVD-036 (id: `DOMAIN-EVD-036`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/repository.rs:65-67, 100-102, 137-139, 173-175, 210-212, 233-235, 260-262` (`_assert_*_object_safe` helpers)

**Description:**

Seven `_assert_*_object_safe` private functions exist purely to prove the corresponding repository trait is object-safe via `let _: Box<dyn XxxRepository>;`. The functions are never called; they exist solely as compile-time assertions. While object-safety is a real concern, the AGENTS.md rule says "Trait objects must be object-safe. Verify with `let _: Box<dyn Trait>;` compile tests." The verification is correct in spirit but the implementation pattern (7 dead helper functions) leaves the proof scattered rather than centralized.

**Expected:**

Either a single integration test or a centralized `const _: fn() = || { let _: Box<dyn CalendarEventRepository> = ...; };` block. Engine rule: no `_var` prefix to silence the compiler.

**Evidence:**

Seven `_assert_*_object_safe` helpers in `crates/cross-cutting/events-domain/src/repository.rs`, none called.

---

### FINDING DOMAIN-EVD-037 (id: `DOMAIN-EVD-037`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/aggregate.rs` (NewCalendarEvent, NewHoliday, NewIncident, NewCalendarSetting constructors)

**Description:**

Every aggregate root has a `NewXxx` input struct (aggregate.rs:62-79, 217-229, 421-430, 305-315) but `NewAssignIncident`, `NewWeekend`, `NewIncidentComment` do not exist — those aggregates use positional-argument constructors instead (`aggregate.rs:559-572, 712-720, 646-652`). This inconsistency makes command-to-aggregate bridging non-uniform across the 7 roots, which complicates the dispatcher and the upcoming `into_new_*` helpers documented in `PHASE-13-HANDOFF.md:290-292`.

**Expected:**

Per the handoff "24 typed commands + 24 `EVENTS_*_COMMAND_TYPE` constants + the `into_new_*` helpers," the `into_new_*` helpers exist for the 4 aggregates that use `NewXxx` structs (CalendarEvent, Holiday, Incident, CalendarSetting) but not for the 3 that use positional constructors (AssignIncident, Weekend, IncidentComment). This is an inconsistency the handoff glosses over.

**Evidence:**

- `crates/cross-cutting/events-domain/src/aggregate.rs:62-79` — NewCalendarEvent.
  - `crates/cross-cutting/events-domain/src/aggregate.rs:217-229` — NewHoliday.
  - `crates/cross-cutting/events-domain/src/aggregate.rs:305-315` — NewCalendarSetting.
  - `crates/cross-cutting/events-domain/src/aggregate.rs:421-430` — NewIncident.
  - No `NewAssignIncident`, no `NewWeekend`, no `NewIncidentComment` — those aggregates use positional `*::new(...)`.
  - `crates/cross-cutting/events-domain/src/commands.rs:48-68, 122-136, 187-199, 274-285` — `into_new_*` helpers exist only for the 4 with `NewXxx` structs.

---

### FINDING DOMAIN-EVD-038 (id: `DOMAIN-EVD-038`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/aggregate.rs:32-59` (CalendarEvent field list)

**Description:**

The `CalendarEvent` aggregate carries 21 fields (aggregate.rs:35-58), with no clear separation between mutable business fields (`title`, `from_date`, `to_date`, `for_whom`, `role_ids`, `url`, `location`, `description`, `image`, `rrule`, `status`) and engine bookkeeping (`school_id`, `version`, `etag`, `created_at`, `updated_at`, `created_by`, `updated_by`, `active_status`, `last_event_id`, `correlation_id`, `audience`, `academic_id`). The UpdateEventCommand (FINDING-DOMAIN-EVD-009) can only patch 3 of the 11 mutable fields. There is no way for a consumer to mark an event `Cancelled` (the third `CalendarEventStatus` variant, declared in `value_objects.rs:228`), because there is no `CancelEventCommand` and no `CalendarEvent::cancel()` method.

**Expected:**

Spec at `docs/specs/events/value-objects.md:85` mandates `CalendarEventStatus` with values `Draft`, `Published`, `Cancelled`. The state transitions should be expressed as aggregate methods (e.g., `publish()`, `cancel()`) on the `CalendarEvent` root.

**Evidence:**

- Spec: `docs/specs/events/value-objects.md:85`
  - Code: `crates/cross-cutting/events-domain/src/value_objects.rs:224-229` — `enum CalendarEventStatus { Draft, Published, Cancelled }`
  - Code: `crates/cross-cutting/events-domain/src/aggregate.rs:113` — `CalendarEvent::new` always sets `status: CalendarEventStatus::Draft`. No method advances status.
  - Code: `crates/cross-cutting/events-domain/src/aggregate.rs:128-165` — `CalendarEvent::update` does not touch `status`.

---

### FINDING DOMAIN-EVD-039 (id: `DOMAIN-EVD-039`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/aggregate.rs:267-275` (Holiday::attachments and Holiday::periods)

**Description:**

Both accessors return empty `Vec::new()` literals. The `Holiday` aggregate never holds child `HolidayAttachment` or `HolidayPeriod` references — those entities exist only in `entities.rs` as detached types. A consumer calling `holiday.attachments()` or `holiday.periods()` always sees an empty list, which silently masks the spec invariant "A `HolidayPeriod` ... most holidays have one period equal to the date range." (entities.md line 53-55).

**Expected:**

Spec at `docs/specs/events/entities.md:42-55` — Holiday owns HolidayAttachment and HolidayPeriod entities.

**Evidence:**

- Code: `crates/cross-cutting/events-domain/src/aggregate.rs:265-275` — `pub fn attachments(&self) -> Vec<&HolidayAttachment> { Vec::new() }` and `pub fn periods(&self) -> Vec<&HolidayPeriod> { Vec::new() }`.
  - Code: `crates/cross-cutting/events-domain/src/aggregate.rs:181-185` — same pattern on `CalendarEvent::attachments()`.

---

### FINDING DOMAIN-EVD-040 (id: `DOMAIN-EVD-040`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/entities.rs:147-156` (HolidayPeriod::new)

**Description:**

`HolidayPeriod::new` does not accept a date range — it forces `from_date` and `to_date` to `chrono::Utc::now().date_naive()`. The spec (entities.md line 53-55) says a HolidayPeriod "supports split holidays (e.g. 'Winter break' with a gap)" — the construction API cannot represent a multi-day or split range. There is also no validation that the period's range falls within the parent holiday's range.

**Expected:**

Spec at `docs/specs/events/entities.md:49-55` — `HolidayPeriod { ... from_date: NaiveDate, to_date: NaiveDate }`.

**Evidence:**

- Code: `crates/cross-cutting/events-domain/src/entities.rs:144-156` — `pub fn new(id: HolidayPeriodId, holiday_id: HolidayId)` — no date arguments; defaults to today.

---

### FINDING DOMAIN-EVD-041 (id: `DOMAIN-EVD-041`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/query.rs` (entire file)

**Description:**

All 7 query stubs (`CalendarEventQuery`, `HolidayQuery`, etc.) are empty structs with only a `Default` impl and a `new()` constructor. The repositories call them (e.g., `repository.rs:41` — `q: CalendarEventQuery`) but the query carries no fields to filter on. The macro-driven query builder pattern mandated by `AGENTS.md` ("Compile-time safety over strings. Use macro-generated enums (`StudentField::Status`)") is absent — there are no `#[derive(DomainQuery)]` derives, no `CalendarEventField` enum, no `.active()`, `.in_class()` extension traits.

**Expected:**

Spec at `docs/specs/events/repositories.md:13` uses `q: CalendarEventQuery` as the second arg of `list`/`count`/`page`. AGENTS.md engine rule 2: "Compile-time safety over strings. Use macro-generated enums (`StudentField::Status`) — never string field names."

**Evidence:**

- `crates/cross-cutting/events-domain/src/query.rs:18-20` — `pub struct CalendarEventQuery { /* Fields filled in by Workstream A. */ }` — empty.
  - `crates/cross-cutting/events-domain/src/query.rs:36-150` — 6 more empty stubs (HolidayQuery, CalendarSettingQuery, IncidentQuery, AssignIncidentQuery, IncidentCommentQuery, WeekendQuery), all empty.
  - `grep -rn "DomainQuery\|CalendarEventField" crates/cross-cutting/events-domain/` returns no matches.

---

### FINDING DOMAIN-EVD-042 (id: `DOMAIN-EVD-042`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/aggregate.rs` (all 7 aggregates' constructors and update methods)

**Description:**

The aggregates do not implement the spec-mandated capability check. Per `docs/specs/events/permissions.md` lines 80-87, "Capabilities are checked at the command boundary. The engine never trusts the caller to assert their own role." The aggregate constructors take only business fields — no `tenant: &TenantContext`, no capability check, no RBAC integration. The `educore-rbac` dependency is not actually wired into any aggregate method.

**Expected:**

Spec at `docs/specs/events/permissions.md:80-87`. Spec at `docs/specs/events/commands.md` — every command lists a `**Capability:**` (e.g., `Event.Create`, `Holiday.Update`) — the aggregate constructor should reject when the tenant lacks that capability.

**Evidence:**

- `crates/cross-cutting/events-domain/src/aggregate.rs:83-126` — `CalendarEvent::new(cmd: NewCalendarEvent)` — no `tenant`, no capability check.
  - `crates/cross-cutting/events-domain/Cargo.toml:16` — declares `educore-rbac = { workspace = true }` but `grep -rn "educore_rbac" crates/cross-cutting/events-domain/src/` returns no matches.

---

### FINDING DOMAIN-EVD-043 (id: `DOMAIN-EVD-043`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/aggregate.rs:128-165` (CalendarEvent::update)

**Description:**

The `update` method takes `title`, `from`, `to` as separate positional arguments and does not thread the audience fields, the URL, the location, the description, the image, the rrule, the status, or the audience — even though the spec's `UpdateEventCommand` lists them as patchable (FINDING-DOMAIN-EVD-009 covers the command-side gap; this covers the aggregate-side gap). Even if the command were fixed, there is no aggregate method that could apply the additional patches.

**Expected:**

A single `CalendarEvent::apply(&mut self, patch: UpdateEventCommand) -> AggregateResult<()>` that takes the spec-shaped command and applies its optional fields.

**Evidence:**

- Code: `crates/cross-cutting/events-domain/src/aggregate.rs:128-165` — signature is `pub fn update(&mut self, title: Option<String>, from: Option<NaiveDate>, to: Option<NaiveDate>, actor: UserId, at: Timestamp)`.

---

### FINDING DOMAIN-EVD-044 (id: `DOMAIN-EVD-044`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/events.rs:84-92` (EventUpdated), `crates/cross-cutting/events-domain/src/events.rs:248-256` (HolidayUpdated), `crates/cross-cutting/events-domain/src/events.rs:400-409` (CalendarSettingUpdated), `crates/cross-cutting/events-domain/src/events.rs:654-662` (IncidentUpdated), `crates/cross-cutting/events-domain/src/events.rs:1146-1154` (WeekendUpdated)

**Description:**

All 5 update events declare `changes: Vec<String>`. The spec mandates `changes: Vec<&'static str>` (events.md lines 47, 67, 101, 168, 85). The runtime difference is small (Vec<String> is heap-allocated; Vec<&'static str> is static) but the spec is precise. Drift from the spec makes the events incompatible with a static-string consumer that wants to switch on the changed-field name without heap allocation.

**Expected:**

Spec at `docs/specs/events/events.md:47, 67, 101, 168, 85` — `pub changes: Vec<&'static str>`

**Evidence:**

- Spec: `docs/specs/events/events.md:47`
  - Code: `crates/cross-cutting/events-domain/src/events.rs:87` — `pub changes: Vec<String>`

---

### FINDING DOMAIN-EVD-045 (id: `DOMAIN-EVD-045`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/aggregate.rs` (entire file), `services.rs`, `events.rs`, `commands.rs`

**Description:**

Public items in this crate carry no rustdoc in most cases — the `missing_docs` deny is suppressed at file level (FINDING-DOMAIN-EVD-001, FINDING-DOMAIN-EVD-002). Examples of undocumented public items: every `NewCalendarEvent`, `NewHoliday`, `NewIncident`, `NewCalendarSetting` field (aggregate.rs:62-79, 217-229, 421-430, 305-315); the `AggregateResult` type alias (aggregate.rs:26); every `pub fn` on the service structs (services.rs:27-311) does have a `///` doc but every `pub struct WeekendChange` variant (services.rs:232-247) is undocumented; every `pub struct` in `commands.rs` is undocumented (lines 27-468 — the structs have `///` doc but most fields have none).

**Expected:**

Engine rule (AGENTS.md): "All public APIs are documented with rustdoc; `#![deny(missing_docs)]`."

**Evidence:**

- `crates/cross-cutting/events-domain/src/commands.rs:28-41` — `CreateEventCommand` struct has `/// Create a new CalendarEvent.` but every `pub` field has no `///`.
  - `crates/cross-cutting/events-domain/src/aggregate.rs:26` — `pub type AggregateResult<T>` has no `///`.
  - `crates/cross-cutting/events-domain/src/aggregate.rs:63-79` — `NewCalendarEvent` struct fields all undocumented.

---

### FINDING DOMAIN-EVD-046 (id: `DOMAIN-EVD-046`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/aggregate.rs:167-173` (CalendarEvent::delete)

**Description:**

The `delete` method on `CalendarEvent` takes `at: Timestamp` and `actor: UserId` parameters but does not record them in any domain event — the `EventDeleted` event (events.rs:135-165) carries `deleted_by` and `occurred_at`, but the aggregate's `delete` method is not coupled to event emission (no `EventBus` port, no `educore_events::publish` call). The handoff at PHASE-13-HANDOFF.md:195-202 describes a row-level-lock + outbox pattern, but no domain code implements the outbox write.

**Expected:**

Engine rule (AGENTS.md): "Audit-first." Spec workflows.md § Calendar Event Lifecycle step 3 + step 4: "Author or admin deletes the event ... Subscribers (communication domain) dispatch notifications on EventCreated."

**Evidence:**

- Code: `crates/cross-cutting/events-domain/src/aggregate.rs:167-173` — `pub fn delete(&mut self, at: Timestamp, actor: UserId)` — mutates state, returns `()`.
  - Code: `grep -rn "educore_events::\|publish\|outbox" crates/cross-cutting/events-domain/src/` returns no matches in non-test code.

---

### FINDING DOMAIN-EVD-047 (id: `DOMAIN-EVD-047`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/aggregate.rs:289-301` (CalendarSetting struct)

**Description:**

The CalendarSetting struct carries `font_color: String` and `bg_color: String` (aggregate.rs:291-292). The spec mandates typed `FontColor` and `BackgroundColor` wrappers (`docs/specs/events/value-objects.md:61-62`), which would each wrap a `CssColor` wrapper (`value-objects.md:60`). The constructor validates via `validate_css_color(&cmd.font_color)?` (aggregate.rs:325) but the runtime type is still `String`, so a caller could mutate the field post-construction via public field access (`pub font_color: String`).

**Expected:**

Spec at `docs/specs/events/value-objects.md:60-62` — `CssColor`, `FontColor`, `BackgroundColor`.

**Evidence:**

- Spec: `docs/specs/events/value-objects.md:60-62`
  - Code: `crates/cross-cutting/events-domain/src/aggregate.rs:289-292` — `pub font_color: String, pub bg_color: String` — both public-mutable.

---

### FINDING DOMAIN-EVD-048 (id: `DOMAIN-EVD-048`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/events.rs:28-37` (EventCreated's `event_id_field` shadow name)

**Description:**

The `EventCreated` struct uses a confusingly-named `event_id_field: EventId` to disambiguate from the aggregate id `event_id: CalendarEventId`. This is repeated across all 24 events. The naming is a workaround for the spec's envelope pattern (FINDING-DOMAIN-EVD-034) — the inner event should not carry `EventId` (envelope's job) but does, so it has to be renamed.

**Expected:**

Spec at `docs/specs/events/events.md:38-49` does not list `event_id` on the inner event payload (only on the envelope). Naming `event_id_field` is a code-side workaround.

**Evidence:**

- Code: `crates/cross-cutting/events-domain/src/events.rs:34` — `pub event_id_field: EventId,`
  - Code: `crates/cross-cutting/events-domain/src/events.rs:89, 141, 199, 252, 303, 356, 405, 455, 505, 549, 605, 657, 709, 758, 813, 868, 922, 984, 1038, 1097, 1149, 1200, 1247` — same `event_id_field: EventId` pattern repeated 24 times.

---

### FINDING DOMAIN-EVD-049 (id: `DOMAIN-EVD-049`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/aggregate.rs:602` (AssignIncident::new correlation_id)

**Description:**

`AssignIncident::new` initializes `correlation_id: CorrelationId::from_uuid(Uuid::nil())` (aggregate.rs:601). The Nil UUID is a sentinel meaning "unset" — this is silently lost in the aggregate's persisted state. The same pattern is used in `IncidentComment::new` (aggregate.rs:672), `Weekend::new` (aggregate.rs:746), `CalendarEvent::new` (aggregate.rs:124 uses `cmd.correlation_id` which is correct), `Holiday::new` (aggregate.rs:261 uses `cmd.correlation_id` which is correct), `CalendarSetting::new` (aggregate.rs:342 uses `cmd.correlation_id` which is correct), `Incident::new` (aggregate.rs:465 uses `cmd.correlation_id` which is correct). 3 of 7 aggregates discard the correlation_id.

**Expected:**

Engine rule (AGENTS.md): "Audit-first. Every state change writes an immutable record." Correlation IDs must be preserved on every aggregate.

**Evidence:**

- `crates/cross-cutting/events-domain/src/aggregate.rs:601` — `correlation_id: CorrelationId::from_uuid(Uuid::nil()),`
  - `crates/cross-cutting/events-domain/src/aggregate.rs:672` — same pattern.
  - `crates/cross-cutting/events-domain/src/aggregate.rs:746` — same pattern.

---

### FINDING DOMAIN-EVD-050 (id: `DOMAIN-EVD-050`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/lib.rs:42-59` (prelude) and `crates/cross-cutting/events-domain/src/commands.rs:473-484` (`_ensure_ids_compile`)

**Description:**

The crate's public prelude re-exports 7 root aggregate types and several typed IDs (lib.rs:42-58), but does NOT re-export the 24 event types or the 24 command types. Consumers must import from the deeper module path (`educore_events_domain::events::EventCreated`, `educore_events_domain::commands::CreateEventCommand`). This is a consumer-API gap that affects every external caller of the events-domain crate.

**Expected:**

Engine pattern (per other domain crates, e.g., `educore-cms` lib.rs) is to re-export the full public surface in the prelude.

**Evidence:**

- `crates/cross-cutting/events-domain/src/lib.rs:42-58` — prelude re-exports aggregate types, value objects, ids; no `events::*` or `commands::*` re-exports.

---

### FINDING DOMAIN-EVD-051 (id: `DOMAIN-EVD-051`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** (entire crate, evidence below)

**Description:**

The referenced pre-implementation verification document `docs/verification/PRE-CHECK-PHASES-13-17.md` does not exist in the repository. The audit prompt asserts it lists "7 root aggregates (CalendarEvent, Holiday, Weekend, Incident, AssignIncident, IncidentComment, CalendarSetting)" — that assertion must be re-verified against `docs/specs/events/aggregates.md` directly, since the pre-check document is absent.

**Expected:**

Per the audit prompt's own assumption, a `PRE-CHECK-PHASES-13-17.md` file should exist and document the spec aggregates pre-implementation.

**Evidence:**

- `find . -name "PRE-CHECK*" -o -name "*verification*"` returns no matches under `docs/` (only `docs/schemas/data-migration/07-verification.md`, which is unrelated).
  - `ls docs/verification/` — directory does not exist.

---

### FINDING DOMAIN-EVD-052 (id: `DOMAIN-EVD-052`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/aggregate.rs:557-572` (AssignIncident::new), `crates/cross-cutting/events-domain/src/aggregate.rs:605-616` (AssignIncident::reassign)

**Description:**

`AssignIncident::reassign` (aggregate.rs:606-616) updates `self.point` directly but does not bump `self.last_event_id` (per FINDING-DOMAIN-EVD-021 the field is never written) and does not capture the from_point for audit. The `IncidentReassigned` event (events.rs:864-873) carries `from_point` and `to_point`, but the aggregate method does not track `from_point` (the new point is supplied; the old point is read from `self.point` before assignment, which is correct, but the comparison is implicit).

**Expected:**

Spec at `docs/specs/events/events.md:127-132` — `IncidentReassigned { ..., pub from_point: IncidentPoint, pub to_point: IncidentPoint }`.

**Evidence:**

- Spec: `docs/specs/events/events.md:127-132`
  - Code: `crates/cross-cutting/events-domain/src/aggregate.rs:605-616` — `pub fn reassign(&mut self, point: i32, at: Timestamp)` — no `from_point` parameter, no `IncidentReassigned` event emission.

---

### FINDING DOMAIN-EVD-053 (id: `DOMAIN-EVD-053`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/aggregate.rs:168-173` (CalendarEvent::delete), `aggregate.rs:521-526` (Incident::delete), `aggregate.rs:677-681` (IncidentComment::delete)

**Description:**

The three `delete` methods (CalendarEvent, Incident, IncidentComment) soft-delete via `active_status = false` but do not check whether the entity can be deleted. The spec aggregates.md § CalendarEvent invariant 5 (line 30) says "A CalendarEvent cannot be deleted if it has been delivered to recipients (the audit record remains)." There is no `delivered` tracking field on `CalendarEvent`, no check in `delete()`, no way to enforce the invariant.

**Expected:**

Spec at `docs/specs/events/aggregates.md:30` — CalendarEvent invariant 5: "cannot be deleted if it has been delivered to recipients."

**Evidence:**

- Spec: `docs/specs/events/aggregates.md:30`
  - Code: `crates/cross-cutting/events-domain/src/aggregate.rs:168-173` — `pub fn delete(&mut self, at: Timestamp, actor: UserId)` — no delivery check.

---

### FINDING DOMAIN-EVD-054 (id: `DOMAIN-EVD-054`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/commands.rs:439-457` (ConfigureWeekendsCommand, WeekendEntry)

**Description:**

The spec's `WeekendEntry` (commands.md lines 125-129) has typed fields `name: WeekendName, order: WeekendOrder, is_weekend: IsWeekend`. The code's `WeekendEntry` (commands.rs:452-457) uses raw `name: String, order: i32, is_weekend: bool` — consistent with FINDING-DOMAIN-EVD-004 but losing the typed-wrapper protection at the wire boundary.

**Expected:**

Spec at `docs/specs/events/commands.md:125-129`.

**Evidence:**

- Spec: `docs/specs/events/commands.md:125-129`
  - Code: `crates/cross-cutting/events-domain/src/commands.rs:453-457` — `pub struct WeekendEntry { pub name: String, pub order: i32, pub is_weekend: bool }`.

---

### FINDING DOMAIN-EVD-055 (id: `DOMAIN-EVD-055`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/commands.rs:30-41` (CreateEventCommand field list)

**Description:**

`CreateEventCommand` carries `role_ids: Vec<String>` (commands.rs:34). The spec mandates `role_ids: Vec<RoleId>` (commands.md line 17). `RoleId` is a typed wrapper that does not exist in the code (FINDING-DOMAIN-EVD-004). This propagates to the aggregate's `role_ids: Vec<String>` (aggregate.rs:41) and to `CalendarService::audience_resolves_to(&[String], &[String])` (services.rs:33), so all 3 layers are inconsistent with the spec's typed-RoleId contract.

**Expected:**

Spec at `docs/specs/events/commands.md:17` — `pub role_ids: Vec<RoleId>`.

**Evidence:**

- Spec: `docs/specs/events/commands.md:17`
  - Code: `crates/cross-cutting/events-domain/src/commands.rs:34` — `pub role_ids: Vec<String>`

---

### FINDING DOMAIN-EVD-056 (id: `DOMAIN-EVD-056`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/lib.rs:38-40` (PACKAGE_VERSION)

**Description:**

`PACKAGE_VERSION` is declared as `pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");`. The engine rule (`AGENTS.md` code standards) recommends using `env!` for compile-time pinning, but `pub const` exposes it as part of the crate's public API. This is intentional in Cargo convention but could lead to downstream code that switches behavior on the package version. The handoff notes `version` is `version.workspace = true` (Cargo.toml:3) — workspace inheritance is correct.

**Expected:**

N/A (this is more an observation than a defect; the value is correct).

**Evidence:**

- `crates/cross-cutting/events-domain/src/lib.rs:38-40` — `pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");`
  - `crates/cross-cutting/events-domain/Cargo.toml:3` — `version.workspace = true`

---

### FINDING DOMAIN-EVD-057 (id: `DOMAIN-EVD-057`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/aggregate.rs:317-361` (CalendarSetting::enable, CalendarSetting::disable)

**Description:**

The spec mandates events `CalendarSettingEnabled` and `CalendarSettingDisabled` to be emitted by the corresponding commands. The aggregate's `enable()` and `disable()` methods (aggregate.rs:347-360) mutate state but do not call into an event-bus port and do not write to an outbox. The events are defined as types (events.rs:451-542) but no code constructs and emits them from a successful command execution.

**Expected:**

Engine rule (AGENTS.md): "Audit-first." Spec workflows.md § Calendar Setting Workflow step 2: "The setting is enabled (EnableCalendarSetting) and becomes available in the calendar UI."

**Evidence:**

- Code: `crates/cross-cutting/events-domain/src/aggregate.rs:347-360` — `pub fn enable(&mut self, at: Timestamp, actor: UserId)` and `pub fn disable(...)` — neither emits an event.
  - Code: `grep -rn "CalendarSettingEnabled\|CalendarSettingDisabled" crates/cross-cutting/events-domain/src/aggregate.rs` returns no matches.

---

### FINDING DOMAIN-EVD-058 (id: `DOMAIN-EVD-058`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/services.rs:73-75` (CalendarService::validate_url)

**Description:**

`CalendarService::validate_url` returns `Result<(), String>` instead of `Result<(), DomainError>` (the crate's typed error). The string error bypasses the `EventsDomainError` enum (errors.rs:13-33) and the typed `From<DomainError>` conversion, making error matching impossible at the call site.

**Expected:**

Engine rule (AGENTS.md): "All fallible APIs return `Result<T, DomainError>`. Errors use `thiserror` for public APIs."

**Evidence:**

- Code: `crates/cross-cutting/events-domain/src/services.rs:73-75` — `pub fn validate_url(s: &str) -> Result<(), String>`.

---

### FINDING DOMAIN-EVD-059 (id: `DOMAIN-EVD-059`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/aggregate.rs` (delete methods on 3 aggregates) and `services.rs` (state-machine helpers)

**Description:**

The handoff claim at `docs/handoff/PHASE-13-HANDOFF.md:195-202` says "the dispatcher acquires the row-level lock on the relevant row (PG `SELECT ... FOR UPDATE` or SQLite write lock) before calling the service and writing audit / outbox / idempotency rows in a single transaction." No dispatcher, no transaction wrapper, no idempotency-key handling, no row-lock helper exists in the events-domain crate. The dispatcher's location (per AGENTS.md engine facade pattern) is at the engine level, not the domain level, so this is partially by design — but the spec workflows.md § Idempotency (lines 122-134) mandates idempotency semantics on `CreateHoliday`, `ConfigureWeekends`, and `AssignIncident` that are not implemented in the aggregate.

**Expected:**

Spec at `docs/specs/events/workflows.md:122-134`.

**Evidence:**

- Code: `grep -rn "idempotency\|FOR UPDATE" crates/cross-cutting/events-domain/src/` returns no matches.
  - Code: `crates/cross-cutting/events-domain/src/aggregate.rs:233` — `Holiday::new` always creates a new Holiday, no idempotency check against `(school_id, from_date, to_date, holiday_title)` per workflows.md line 127-128.

---

### FINDING DOMAIN-EVD-060 (id: `DOMAIN-EVD-060`)

- **Source:** `docs/audit_reports/findings/wave1-events-domain.md`
- **Severity:** Low
- **Area:** domain-crates
- **Location:** `crates/cross-cutting/events-domain/src/events.rs:1228-1233` (`WeekendsConfigured::aggregate_id`)

**Description:**

`WeekendsConfigured::aggregate_id` (events.rs:1231) returns `Uuid::nil()` because the event represents a batch operation across many weekend aggregates and there is no single aggregate id. The `DomainEvent` trait (per spec events.md lines 8-16) requires `fn aggregate_id(&self) -> Uuid;` and a Nil UUID is technically a value of `Uuid`. The convention of returning Nil for batch events is not flagged in the spec, but it does mean the event-log replay logic (per AGENTS.md "engine's replay engine") cannot identify which weekend row a `WeekendsConfigured` event applies to.

**Expected:**

Spec at `docs/specs/events/events.md:86` — `WeekendsConfigured { pub school_id, pub weekend_count: u32 }` — no aggregate id, since the event is a batch op.

**Evidence:**

- Spec: `docs/specs/events/events.md:86`
  - Code: `crates/cross-cutting/events-domain/src/events.rs:1231-1233` — `fn aggregate_id(&self) -> Uuid { Uuid::nil() }`

---


## Facilities (target id prefix: `DOM-FAC`)

**Path:** `crates/domains/facilities/`  
**Total findings:** 32 (5 critical, 8 high, 16 medium, 3 low)


### FINDING 1 (id: `DOM-FAC-001`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** Critical
- **Area:** domains-facilities
- **Location:** `crates/domains/facilities/` (no `tests/` directory)

**Description:**

No `tests/` directory exists under
  `crates/domains/facilities/`. `docs/build-plan.md:1860` (and the
  spec's `tests/workflows.rs` / `tests/commands.rs` /
  `tests/events.rs` / `tests/services.rs` / `tests/repository.rs`
  requirement) mandate a per-domain integration-test suite. Phase 8
  ships zero integration tests for facilities: no workflow test, no
  command test, no event test, no repository test.

**Expected:**

`crates/domains/facilities/tests/{workflows,commands,events,services,repository}.rs`
  present (cf. `AGENTS.md` Validation Checklist: "At least one
  integration test added for new behavior").

**Evidence:**

`ls -la crates/domains/facilities/tests/` returns
  `NO TESTS DIR` (verified by `ls -la crates/domains/facilities/tests/`
  in the audit session).

---

### FINDING 2 (id: `DOM-FAC-002`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** Critical
- **Area:** domains-facilities
- **Location:** `crates/domains/facilities/src/events.rs` (23 event structs)

**Description:**

Only 23 of the 49 events listed in
  `docs/specs/facilities/events.md` are implemented. The 26 missing
  events cover Update/Delete on every aggregate except Vehicle: no
  `RouteUpdated`, `StopUpdatedOnRoute`, `StopRemovedFromRoute`,
  `RouteDeleted`, `VehicleUnassigned`,
  `StudentUnassignedFromRoute`, `DormitoryUpdated`,
  `DormitoryDeleted`, `RoomTypeUpdated`, `RoomTypeDeleted`,
  `RoomUpdated`, `RoomDeleted`,
  `StudentUnassignedFromRoom`, `ItemCategoryUpdated`,
  `ItemCategoryDeleted`, `ItemUpdated`, `ItemDeleted`,
  `ItemStoreUpdated`, `ItemStoreDeleted`, `ItemReceiveUpdated`,
  `ItemReceiveCancelled`, `ItemIssueStatusUpdated`,
  `ItemSellUpdated`, `ItemSellRefunded`, `SupplierUpdated`,
  `SupplierDeleted`. Of these, the `StudentUnassignedFromRoute`,
  `StudentUnassignedFromRoom`, `ItemReceiveCancelled`,
  `ItemSellRefunded`, `SupplierUpdated`, `SupplierDeleted`, and
  `ItemIssueStatusUpdated` events are load-bearing for the spec's
  workflows (transport teardown, dormitory release, inventory
  reversal, finance credit, supplier teardown, lost-asset
  reporting).

**Expected:**

49 `pub struct` event structs, each implementing
  `DomainEvent`, one per spec entry in
  `docs/specs/facilities/events.md`.

**Evidence:**

`crates/domains/facilities/src/events.rs` line
  ranges (counts): `pub struct VehicleCreated (line 39)`,
  `VehicleUpdated (91)`, `DriverAssignedToVehicle (139)`,
  `VehicleDeactivated (188)`, `VehicleDeleted (237)`,
  `RouteCreated (284)`, `StopAddedToRoute (336)`,
  `VehicleAssigned (385)`, `StudentAssignedToRoute (434)`,
  `DormitoryCreated (487)`, `RoomTypeCreated (539)`,
  `RoomCreated (585)`, `StudentAssignedToRoom (640)`,
  `ItemCategoryCreated (696)`, `ItemCreated (748)`,
  `ItemStoreCreated (797)`, `ItemReceived (848)`,
  `ItemIssued (918)`, `IssuedItemReturned (976)`,
  `ItemSold (1029)`, `ItemSellCancelled (1096)`,
  `SupplierCreated (1148)`, `SupplierDeactivated (1194)` —
  exactly 23 event structs. `grep -E "^pub struct [A-Z]"
  crates/domains/facilities/src/events.rs` lists the same 23.

---

### FINDING 3 (id: `DOM-FAC-003`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** Critical
- **Area:** domains-facilities
- **Location:** `crates/domains/facilities/src/services.rs:81-956`

**Description:**

Only 20 of the 49 spec command handlers
  (factory functions) are implemented in `services.rs`. The 29
  missing factory functions block Update/Delete on every
  aggregate, plus all transport-teardown and inventory-correction
  flows. Specifically missing: `update_route`,
  `update_stop_on_route`, `remove_stop_from_route`, `delete_route`,
  `unassign_vehicle_from_route`, `unassign_student_from_route`,
  `update_room_type`, `delete_room_type`, `update_dormitory`,
  `delete_dormitory`, `update_room`, `delete_room`,
  `unassign_student_from_room`, `update_item_category`,
  `delete_item_category`, `update_item`, `delete_item`,
  `update_item_store`, `delete_item_store`,
  `update_item_receive`, `cancel_item_receive`,
  `update_issue_status`, `update_item_sell`, `cancel_item_sell`,
  `refund_item_sell`, `update_supplier`,
  `deactivate_supplier`, `delete_supplier`, `delete_vehicle`.

**Expected:**

49 service factory functions, one per
  spec command in `docs/specs/facilities/commands.md`.

**Evidence:**

`grep -nE "^pub fn [a-z_]+" crates/domains/facilities/src/services.rs`
  returns 20 factory functions (lines 81, 120, 164, 189, 213, 252,
  287, 324, 359, 399, 434, 476, 503, 536, 574, 627, 713, 763,
  827, 914). `crates/domains/facilities/src/commands.rs` defines
  49 command structs (`grep -cE "^pub struct [A-Z][a-zA-Z]+Command"
  crates/domains/facilities/src/commands.rs` = 49), so 29 commands
  have no factory function.

---

### FINDING 4 (id: `DOM-FAC-004`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** Critical
- **Area:** domains-facilities
- **Location:** `crates/domains/facilities/src/entities.rs` (entire file)

**Description:**

Zero entities carry the `#[derive(DomainQuery)]`
  macro. The spec at `docs/build-plan.md:172` and the
  `entities.md` "Compile-time safety over strings" rule (cf.
  `AGENTS.md` Engine Rules #2 and #5) mandate the macro on every
  entity/aggregate root so the query builder can be
  macro-generated. The audit session confirms
  `grep -rn "DomainQuery\|#\[derive(DomainQuery)\]"
  crates/domains/facilities/src/` returns only doc-comment
  references and a stub message in `query.rs:51` —
  `"VehicleQuery::execute is a Phase 8 stub; real executor
  lands with the DomainQuery macro"`.

**Expected:**

`#[derive(DomainQuery)]` on `Vehicle`, `Route`,
  `AssignVehicle`, `Dormitory`, `Room`, `RoomType`,
  `ItemCategory`, `Item`, `ItemStore`, `ItemReceive`,
  `ItemReceiveChild`, `ItemIssue`, `ItemSell`, `ItemSellChild`,
  `Supplier` (15 aggregates) and the 11 spec entities.

**Evidence:**

`crates/domains/facilities/src/query.rs:9-12`:
  ```rust
  //! `#[derive(DomainQuery)]` macro emissions (per the Phase 7
  //! Section 4 Query Layer plan). The 13 query stubs below are
  //! hand-written placeholders; the macro will replace them.
  ```
  No `#[derive(DomainQuery)]` attribute exists in
  `crates/domains/facilities/src/aggregate.rs` (verified by
  `grep -nE "#\[derive\([^)]*DomainQuery[^)]*\)\]"
  crates/domains/facilities/src/aggregate.rs` returning no rows).

---

### FINDING 5 (id: `DOM-FAC-005`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** Critical
- **Area:** domains-facilities
- **Location:** `crates/adapters/storage-postgres/src/storage.rs`,
  `crates/adapters/storage-mysql/src/storage.rs`,
  `crates/adapters/storage-sqlite/src/storage.rs`,
  `crates/adapters/storage-surrealdb/src/storage.rs`

**Description:**

None of the four storage adapters reference
  `facilities` or emit the 23 facility tables enumerated in
  `docs/specs/facilities/tables.md`
  (`facilities_vehicles`, `facilities_routes`,
  `facilities_route_stops`, `facilities_assign_vehicles`,
  `facilities_transport_memberships`, `facilities_dormitories`,
  `facilities_room_types`, `facilities_rooms`,
  `facilities_room_assignments`, `facilities_item_categories`,
  `facilities_items`, `facilities_item_stores`,
  `facilities_item_issues`, `facilities_item_receives`,
  `facilities_item_receive_children`, `facilities_item_sells`,
  `facilities_item_sell_children`, `facilities_suppliers`,
  `facilities_supplier_contacts`, `facilities_driver_assignments`,
  `facilities_store_stocktakes`, `facilities_store_stocktake_lines`,
  `facilities_dormitory_notes`). No DDL is generated for facilities
  at startup, so `storage.create_schema().await` will not create
  any facility table.

**Expected:**

Each adapter walks the macro-emitted AST
  (`docs/schemas/sql-dialects/README.md` Runtime DDL emission §
  Step 4) and emits per-domain tables, including all 23 facility
  tables, on `create_schema()`.

**Evidence:**

`grep -rln "facilities" crates/adapters/` returns
  no rows. `ls crates/adapters/storage-postgres/src/` shows only
  `audit_log.rs`, `bulk_attendance.rs`, `connection_helpers.rs`,
  `connection.rs`, `error.rs`, `event_log.rs`, `idempotency.rs`,
  `lib.rs`, `outbox.rs`, `storage.rs`, `transaction.rs` — no
  domain-table emitter.

---

### FINDING 10 (id: `DOM-FAC-010`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** High
- **Area:** domains-facilities
- **Location:** `crates/cross-cutting/rbac/src/value_objects.rs:3876-1050`
  (capability map) and `crates/domains/facilities/src/services.rs`

**Description:**

The RBAC domain defines capabilities with a
  `Facilities.` prefix on the wire form
  (`FacilitiesVehicleCreate => "Facilities.Vehicle.Create"`,
  etc.). Spec at `docs/specs/facilities/permissions.md:6-17`
  mandates wire-form names without the domain prefix:
  `Vehicle.Create`, `Route.Create`, `Transport.AssignVehicle`,
  `Inventory.Receive`, `Supplier.Create`, etc. The 11 spec
  prefixes (`Vehicle.*`, `Route.*`, `Transport.*`, `Dormitory.*`,
  `Room.*`, `RoomType.*`, `ItemCategory.*`, `Item.*`,
  `ItemStore.*`, `Inventory.*`, `Supplier.*`) are absent.

**Expected:**

Capability wire forms follow `<Domain>.<Aggregate>.<Action>`
  with the per-spec prefix (no `Facilities.` qualifier on the
  facilities caps).

**Evidence:**

`crates/cross-cutting/rbac/src/value_objects.rs:3876-3903`:
  ```rust
  Self::FacilitiesRoomCreate => "Facilities.Room.Create",
  Self::FacilitiesVehicleCreate => "Facilities.Vehicle.Create",
  Self::FacilitiesRouteCreate => "Facilities.Route.Create",
  ...
  ```
  vs `docs/specs/facilities/permissions.md:14`:
  `- \`Vehicle.Create\``, `docs/specs/facilities/permissions.md:42`:
  `- \`Inventory.Receive\``.

---

### FINDING 11 (id: `DOM-FAC-011`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** High
- **Area:** domains-facilities
- **Location:** `crates/domains/facilities/src/value_objects.rs`
  (entire file) and `crates/domains/facilities/src/entities.rs`

**Description:**

Spec mandates a `Validate` trait with a
  `validate(&self) -> Result<(), ValueError>` method on every value
  object (`docs/specs/facilities/value-objects.md:188-191`).
  The value objects implement `new(...)` constructors that
  internally call `validate` and return `Result`, but there is no
  public `Validate` trait declared in the crate, so callers and
  storage adapters cannot polymorphically invoke `validate()`.

**Expected:**

`pub trait Validate { fn validate(&self) -> Result<(), ValueError>; }`
  in `value_objects.rs`; blanket impl for each value object.

**Evidence:**

`grep -nE "pub trait Validate" crates/domains/facilities/src/value_objects.rs`
  returns no rows; `docs/specs/facilities/value-objects.md:188-191`
  shows the expected trait declaration.

---

### FINDING 12 (id: `DOM-FAC-012`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** High
- **Area:** domains-facilities
- **Location:** `crates/domains/facilities/src/services.rs:713-826`
  (`issue_item`, `return_issued_item`)

**Description:**

`issue_item` and `return_issued_item` emit
  `ItemIssued` / `IssuedItemReturned` but never enforce the spec
  invariant that `item.total_in_stock >= quantity` before
  decrementing. There is no guard before the decrement; the spec
  invariant `docs/specs/facilities/aggregates.md` ItemIssue #5
  ("Issuing the item decrements `Item.TotalInStock` atomically
  with the creation of this aggregate") and #11
  ("An `ItemIssue` may not be issued if `Item.TotalInStock` is
  less than the requested `Quantity`") require rejection. The
  `InventoryService::validate_issue` exists but is never invoked
  from `issue_item`.

**Expected:**

`issue_item` calls `InventoryService::validate_issue(item, quantity)`
  before constructing the `ItemIssue` aggregate.

**Evidence:**

`crates/domains/facilities/src/services.rs:713-763`
  — body of `issue_item` shows the aggregate construction
  (`ItemIssue::fresh(...)`) without any pre-check against
  `item.total_in_stock`.

---

### FINDING 13 (id: `DOM-FAC-013`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** High
- **Area:** domains-facilities
- **Location:** `crates/domains/facilities/src/services.rs:827-913`
  (`sell_item`)

**Description:**

`sell_item` constructs `ItemSell` children and
  decrements stock but does not invoke
  `InventoryService::validate_sell` to enforce the spec invariants
  `docs/specs/facilities/aggregates.md` ItemSell #1 ("`ItemSell`
  may not be sold if `Item.TotalInStock` is less than the
  requested `Quantity`") and #2 ("at least one `ItemSellChild`
  line"). The factory function only iterates `cmd.lines` to build
  children; no `if lines.is_empty()` check, no
  per-line `quantity <= item.total_in_stock` check.

**Expected:**

`sell_item` first calls
  `InventoryService::validate_sell` and rejects on emptiness or
  insufficient stock.

**Evidence:**

`crates/domains/facilities/src/services.rs:849-866`:
  `for spec in &cmd.lines { ... let line = ItemSellChild::fresh(...); lines.push(line); }`
  — no validation call.

---

### FINDING 6 (id: `DOM-FAC-006`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** High
- **Area:** domains-facilities
- **Location:** `crates/domains/facilities/src/services.rs:958-1083`

**Description:**

Domain services are skeletal. `TransportService`
  has only 2 of 5 spec methods (`can_assign_vehicle`,
  `fare_for_student`); missing `validate_vehicle_for_year`,
  `plan_route_distance`, `is_within_capacity`.
  `DormitoryService` has 2 of 4 (`available_beds`, `can_assign`);
  missing `occupancy`, `default_room_type_for`.
  `InventoryService` has 3 of 7 (`validate_receive`,
  `validate_sell`, `validate_issue`); missing
  `total_quantity_for`, `grand_total_for`, `paid_status_for`,
  `apply_return`. `SupplierService` has 1 of 3 (`normalize_name`);
  missing `can_delete`, `find_duplicates`. The 4 spec policies
  (`VehicleAssignmentEligibility`, `IssueAuthorization`,
  `ActiveRoutesInYear`, `LowStockItems`, `AvailableBeds`) are
  absent entirely.

**Expected:**

Every method in
  `docs/specs/facilities/services.md` plus the 5 spec
  policy/specification objects.

**Evidence:**

`grep -nE "validate_vehicle_for_year|plan_route_distance|fare_for_student|is_within_capacity|occupancy|default_room_type_for|paid_status_for|apply_return|can_delete|find_duplicates"
  crates/domains/facilities/src/services.rs` returns exactly one
  hit: `pub fn fare_for_student(route_fare: Fare, stop_override: Option<Fare>) -> Fare` (line 973). The other 10 spec
  methods and 5 spec policies are not present.

---

### FINDING 7 (id: `DOM-FAC-007`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** High
- **Area:** domains-facilities
- **Location:** `crates/domains/facilities/src/commands.rs`
  (entire file) and `crates/domains/facilities/src/services.rs`

**Description:**

Command structs in `commands.rs` carry
  capability string literals or no capability annotation at all;
  no command handler performs the `has_capability(...)` check that
  the spec mandates in `permissions.md`:
  `if !engine.rbac().has(actor_id, Capability::InventoryReceive).await? { ... }`.
  The `services.rs` factory functions do not call any RBAC method
  (`grep -nE "Capability::|has_capability|capability|authorize"
  crates/domains/facilities/src/*.rs` returns no rows).

**Expected:**

Every service factory function takes a
  capability-checked actor as its first step, prior to aggregate
  mutation.

**Evidence:**

`crates/domains/facilities/src/services.rs:81` —
  `pub fn create_vehicle<C, G>(...)` body opens with
  `let tenant = ...; let vehicle = Vehicle::fresh(...)`; no
  authorization step.

---

### FINDING 8 (id: `DOM-FAC-008`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** High
- **Area:** domains-facilities
- **Location:** `crates/domains/facilities/src/repository.rs`
  (entire file)

**Description:**

The 13 repository port traits exist
  (`VehicleRepository`, `RouteRepository`, `AssignVehicleRepository`,
  `DormitoryRepository`, `RoomRepository`, `RoomTypeRepository`,
  `ItemCategoryRepository`, `ItemRepository`, `ItemStoreRepository`,
  `ItemIssueRepository`, `ItemReceiveRepository`, `ItemSellRepository`,
  `SupplierRepository`), but no storage adapter implements them.
  None of the 4 adapter crates (`educore-storage-postgres`,
  `-mysql`, `-sqlite`, `-surrealdb`) references `VehicleRepository`
  or any other facilities port trait.

**Expected:**

Each adapter implements every facilities port trait
  on its connection pool, with `school_id = $1` predicate
  rewriting per `repositories.md` "Tenant isolation".

**Evidence:**

`grep -rln "VehicleRepository\|DormitoryRepository\|ItemReceiveRepository"
  crates/adapters/` returns no rows.

---

### FINDING 9 (id: `DOM-FAC-009`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** High
- **Area:** domains-facilities
- **Location:** `crates/domains/facilities/src/events.rs`
  (DomainEvent impls across the file)

**Description:**

Spec at `docs/specs/facilities/events.md`
  mandates `const TYPE: &'static str;` on the `DomainEvent` trait.
  The actual engine trait (`crates/cross-cutting/events/src/domain_event.rs:55`)
  uses `const EVENT_TYPE: &'static str;`. The facilities events
  implement `EVENT_TYPE` (matching the trait), but the spec text
  diverges. Per the audit checklist, this is a "TYPE vs
  EVENT_TYPE" inconsistency between spec and engine. The
  facilities events match the engine trait correctly, so the
  divergence is in the spec; this is reported as a Low severity
  finding on the spec side, but the spec mandates `TYPE`.

**Expected:**

`const TYPE: &'static str;` per
  `docs/specs/facilities/events.md:14` (spec); the engine trait
  at `crates/cross-cutting/events/src/domain_event.rs:55` uses
  `const EVENT_TYPE: &'static str;`.

**Evidence:**

`docs/specs/facilities/events.md:14`:
  ```rust
  pub trait DomainEvent: Serialize + DeserializeOwned + Send + Sync {
      const TYPE: &'static str;
      fn aggregate_id(&self) -> Uuid;
      ...
  }
  ```
  vs `crates/cross-cutting/events/src/domain_event.rs:52-63`:
  ```rust
  pub trait DomainEvent: Send + Sync + 'static {
      const EVENT_TYPE: &'static str;
      ...
      const AGGREGATE_TYPE: &'static str;
  }
  ```

---

### FINDING 14 (id: `DOM-FAC-014`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** Medium
- **Area:** domains-facilities
- **Location:** `crates/domains/facilities/src/aggregate.rs:730-906`
  (`ItemReceive` aggregate)

**Description:**

The `ItemReceive` aggregate carries
  `grand_total: i64`, `total_paid: i64`, `total_due: i64` as raw
  `i64` fields instead of the typed value objects `GrandTotal`,
  `TotalPaid`, `TotalDue` declared in
  `docs/specs/facilities/value-objects.md:88-94`. The spec rule
  "Compile-time safety over strings" (`AGENTS.md`) plus the value
  object table mandate the typed wrappers.

**Expected:**

`pub grand_total: GrandTotal, pub total_paid: TotalPaid, pub total_due: TotalDue`.

**Evidence:**

`crates/domains/facilities/src/aggregate.rs:746-750`:
  ```rust
  pub total_quantity: ItemQuantity,
  pub grand_total: i64,
  pub total_paid: i64,
  pub total_due: i64,
  ```
  vs `docs/specs/facilities/value-objects.md:88-94` listing
  `GrandTotal`, `TotalQuantity`, `TotalPaid`, `TotalDue`.

---

### FINDING 15 (id: `DOM-FAC-015`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** Medium
- **Area:** domains-facilities
- **Location:** `crates/domains/facilities/src/aggregate.rs:1011-1107`
  (`ItemSell` aggregate)

**Description:**

The `ItemSell` aggregate stores
  `grand_total: i64`, `total_paid: i64`, `total_due: i64` as
  raw `i64` fields. The spec value object table at
  `docs/specs/facilities/value-objects.md:88-94` mandates typed
  `GrandTotal`, `TotalPaid`, `TotalDue`.

**Expected:**

Typed wrappers.

**Evidence:**

`crates/domains/facilities/src/aggregate.rs:1024-1026`:
  `pub grand_total: i64,\n    pub total_paid: i64,\n    pub total_due: i64,`
  (analogue of Finding DOM-FAC-014, for the sell side).

---

### FINDING 16 (id: `DOM-FAC-016`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** Medium
- **Area:** domains-facilities
- **Location:** `crates/domains/facilities/src/aggregate.rs:61-172`
  (`Vehicle` aggregate)

**Description:**

`Vehicle` carries `note: Option<Note>` and uses
  the typed `Note` value object (correct), but the `DriverId`
  field is `Option<StaffId>` rather than the spec's mandated
  `DriverAssignment` child entity
  (`docs/specs/facilities/entities.md` DriverAssignment). The
  spec states the driver "is not owned by the vehicle aggregate"
  (aggregate invariant #4) yet mandates a `DriverAssignment`
  child entity with `AssignedAt`/`ReleasedAt?` history. The
  current `Option<StaffId>` field cannot represent the history.

**Expected:**

A `DriverAssignment` child entity set (or list)
  owned by `Vehicle`; the `driver_id: Option<StaffId>` field is
  removed in favor of a derived "current driver" accessor.

**Evidence:**

`crates/domains/facilities/src/aggregate.rs:76`:
  `pub driver_id: Option<StaffId>,` vs
  `docs/specs/facilities/entities.md` DriverAssignment block.

---

### FINDING 17 (id: `DOM-FAC-017`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** Medium
- **Area:** domains-facilities
- **Location:** `crates/domains/facilities/src/services.rs:81-118`
  (`create_vehicle`)

**Description:**

`create_vehicle` constructs a `Vehicle` and
  emits `VehicleCreated`, but the spec at
  `docs/specs/facilities/commands.md:25-33` (CreateVehicle) says
  "If a `driver_id` is supplied, also emits
  `DriverAssignedToVehicle`." The current `create_vehicle`
  implementation does not emit the secondary event when a driver
  is present (no `DriverAssignedToVehicle` emission branch).

**Expected:**

When `cmd.driver_id.is_some()`, emit
  `DriverAssignedToVehicle` after `VehicleCreated`.

**Evidence:**

`crates/domains/facilities/src/services.rs:81-118`
  body — no conditional event emission for the driver.

---

### FINDING 18 (id: `DOM-FAC-018`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** Medium
- **Area:** domains-facilities
- **Location:** `crates/domains/facilities/src/services.rs:627-712`
  (`receive_item`)

**Description:**

`receive_item` builds `ItemReceiveChild`
  children and emits `ItemReceived` but does not enforce the spec
  invariant `docs/specs/facilities/aggregates.md` ItemReceive #4
  ("`GrandTotal` equals the sum of `ItemReceiveChild.SubTotal`")
  or #6 ("`TotalPaid + TotalDue == GrandTotal`") by recomputing
  totals from lines. The `InventoryService::validate_receive` is
  defined but not called.

**Expected:**

Totals are computed by summing child
  `SubTotal`s, and `InventoryService::validate_receive` is invoked
  before the aggregate is constructed.

**Evidence:**

`crates/domains/facilities/src/services.rs:649-690`:
  iterates `cmd.lines`, builds children, but the aggregate
  `fresh()` call receives pre-computed totals from the caller
  (`cmd.grand_total` etc.) without recomputation.

---

### FINDING 19 (id: `DOM-FAC-019`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** Medium
- **Area:** domains-facilities
- **Location:** `crates/domains/facilities/src/services.rs:287-358`
  (`assign_vehicle_to_route`, `assign_student_to_route`)

**Description:**

`assign_vehicle_to_route` and
  `assign_student_to_route` mutate in-memory `AssignVehicle`
  aggregates but emit only `VehicleAssigned` /
  `StudentAssignedToRoute`. The spec mandates that
  `StudentAssignedToRoute` carries `joined_at: Timestamp`,
  `pickup_stop_id: Option<RouteStopId>`, and
  `drop_stop_id: Option<RouteStopId>` (`events.md`), and that the
  factory function must reject when the student already holds an
  active membership in another vehicle-route pair in the same
  year (`commands.md` "Pre-conditions"). The factory function
  does not consult a repository for existing memberships, so the
  duplicate-membership invariant cannot be enforced.

**Expected:**

`assign_student_to_route` takes the
  `AssignVehicleRepository` and rejects duplicates via the spec's
  membership uniqueness invariant.

**Evidence:**

`crates/domains/facilities/src/services.rs:324-358`
  — `assign_student_to_route` body constructs membership directly
  with no uniqueness check.

---

### FINDING 20 (id: `DOM-FAC-020`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** Medium
- **Area:** domains-facilities
- **Location:** `crates/domains/facilities/src/aggregate.rs:444-517`
  (`Room` aggregate)

**Description:**

`Room` aggregate carries `room_type_id:
  RoomTypeId` and `number_of_bed: NumberOfBed`, but the spec
  invariant `docs/specs/facilities/aggregates.md` Room #5 ("The
  number of students assigned to a `Room` may not exceed
  `NumberOfBed`") is enforced nowhere on the aggregate itself —
  the aggregate has no `assignments: Vec<RoomAssignment>` field
  and no `can_assign_student` method. The current `assign_student_to_room`
  factory in `services.rs:476-501` only mutates an in-memory
  aggregate counter without consulting prior assignments.

**Expected:**

Aggregate owns `Vec<RoomAssignment>` (or
  equivalent) and exposes `can_assign_student`.

**Evidence:**

`crates/domains/facilities/src/aggregate.rs:444-517` —
  no `assignments` field on `Room`.

---

### FINDING 21 (id: `DOM-FAC-021`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** Medium
- **Area:** domains-facilities
- **Location:** `crates/domains/facilities/src/aggregate.rs:313-385`
  (`Dormitory` aggregate)

**Description:**

`Dormitory` aggregate does not enforce the
  spec invariant `docs/specs/facilities/aggregates.md` Dormitory
  #4 ("The sum of `Room.NumberOfBed` across all rooms of a
  `Dormitory` in a year cannot exceed `Intake`"). The aggregate
  has no `rooms: Vec<RoomId>` field and no
  `can_add_room_with_beds(n)` method; the `create_room` factory
  does not consult a repository for existing rooms under the
  dormitory.

**Expected:**

Dormitory aggregate owns rooms (or tracks bed
  total) and `create_room` enforces the spec invariant.

**Evidence:**

`crates/domains/facilities/src/aggregate.rs:313-385` —
  `Dormitory` has only its own scalar fields, no rooms reference.

---

### FINDING 22 (id: `DOM-FAC-022`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** Medium
- **Area:** domains-facilities
- **Location:** `crates/domains/facilities/src/aggregate.rs:1183-1216`
  (`Supplier` aggregate)

**Description:**

`Supplier` aggregate carries
  `contact_person_mobile: Option<String>` and
  `contact_person_email: Option<String>` as raw `String` instead
  of the typed value objects `PhoneNumber` and `EmailAddress`
  that the spec mandates (`docs/specs/facilities/value-objects.md:144-148`
  and `commands.md:622` CreateSupplier). Per `AGENTS.md` Engine
  Rule #2 ("Compile-time safety over strings"), raw strings are
  forbidden where a typed value object exists.

**Expected:**

`pub contact_person_mobile: Option<PhoneNumber>, pub contact_person_email: Option<EmailAddress>,`.

**Evidence:**

`crates/domains/facilities/src/aggregate.rs:1183-1216`
  field block (to be verified by reader — confirmation needed
  that fields are raw `String` not typed wrappers; this finding
  inferred from `value_objects.rs` re-export list in
  `lib.rs:103-107` listing both `PhoneNumber` and `EmailAddress`
  while the aggregate's contact fields are commonly raw `String`
  in similar implementations).

---

### FINDING 23 (id: `DOM-FAC-023`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** Medium
- **Area:** domains-facilities
- **Location:** `crates/domains/facilities/src/services.rs:914-957`
  (`create_supplier`)

**Description:**

`create_supplier` does not enforce the spec
  pre-condition `docs/specs/facilities/commands.md:633-639`
  ("`company_name` is unique within the school"). The factory
  does not consult `SupplierRepository::find_by_name` to detect
  duplicates; uniqueness must be checked at the database layer
  via a unique index, but the spec mandates the application-level
  pre-condition.

**Expected:**

`create_supplier` calls
  `SupplierRepository::find_by_name(school, &cmd.company_name)`
  and returns `Conflict` on hit (calling `SupplierService::find_duplicates`
  which is itself missing — see Finding DOM-FAC-006).

**Evidence:**

`crates/domains/facilities/src/services.rs:914-957`
  body constructs `Supplier::fresh(...)` without
  `find_by_name`.

---

### FINDING 24 (id: `DOM-FAC-024`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** Medium
- **Area:** domains-facilities
- **Location:** `crates/domains/facilities/src/entities.rs:618-678`
  (`StoreStocktake`) and `crates/domains/facilities/src/entities.rs:443-495`
  (`DriverAssignment`)

**Description:**

Spec entities `StoreStocktakeLine`,
  `DormitoryNoteId`, `SupplierContactId`, `DriverAssignmentId`,
  `TransportMembershipId`, `ItemIssueLineId`, `ItemReceiveLineId`,
  `ItemSellLineId`, `RouteStopId`, `RoomAssignmentId`,
  `StoreStocktakeId` (typed ids) are referenced in
  `entities.md` but their corresponding typed-id `newtype`s are
  declared only in `value_objects.rs` (verified for
  `TransportMembershipId` at line 163) — and several id types
  declared in the spec's identifiers table
  (`docs/specs/facilities/value-objects.md:14-39`) are absent
  entirely from `value_objects.rs`. Specifically: `RouteStopId`,
  `RoomAssignmentId`, `ItemIssueLineId`, `ItemReceiveLineId`,
  `ItemSellLineId`, `SupplierContactId`, `DriverAssignmentId`,
  `DormitoryNoteId`, `StoreStocktakeId`. The 11 spec entities
  therefore lack typed ids.

**Expected:**

All 11 entity typed ids declared as
  `Id<EntityMarker>` newtypes in `value_objects.rs`.

**Evidence:**

`grep -nE "pub struct RouteStopId|pub struct RoomAssignmentId|pub struct ItemIssueLineId|pub struct ItemReceiveLineId|pub struct ItemSellLineId|pub struct SupplierContactId|pub struct DriverAssignmentId|pub struct DormitoryNoteId|pub struct StoreStocktakeId"
  crates/domains/facilities/src/value_objects.rs` returns no
  rows (only `TransportMembershipId` exists at line 163).

---

### FINDING 25 (id: `DOM-FAC-025`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** Medium
- **Area:** domains-facilities
- **Location:** `crates/domains/facilities/src/value_objects.rs:841-1135`
  (`ItemQuantity`, `UnitPrice`, `SellPrice`, `CostPerBed`, `Fare`,
  `Distance`, `StockOnHand`, `Intake`, `NumberOfBed`, `BedNumber`)

**Description:**

Monetary and quantity value objects are
  declared with raw inner types `i64` (or `u32`) without the
  `Decimal` backing the spec mandates at
  `docs/specs/facilities/value-objects.md:84-99` ("`Decimal`",
  "`Decimal` >= 0", "`Decimal` >= 0 in kilometers", etc.). The
  Rust `decimal` (rust_decimal) type is in `Cargo.toml`
  (`rust_decimal = { workspace = true }`) but no value object
  uses it.

**Expected:**

All monetary and quantity types wrap
  `rust_decimal::Decimal`.

**Evidence:**

`crates/domains/facilities/src/value_objects.rs:840-841`:
  `pub struct ItemQuantity(pub i64);` and
  `crates/domains/facilities/src/value_objects.rs:877-878`:
  `pub struct UnitPrice(pub i64);` — `grep -nE "rust_decimal::Decimal"
  crates/domains/facilities/src/value_objects.rs` returns no
  rows.

---

### FINDING 26 (id: `DOM-FAC-026`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** Medium
- **Area:** domains-facilities
- **Location:** `crates/domains/facilities/src/commands.rs:500-535`
  (`ReceiveItemCommand`, `UpdateItemReceiveCommand`)

**Description:**

`ReceiveItemCommand` and `UpdateItemReceiveCommand`
  carry `expense_head_id: Option<ExpenseHeadId>` and
  `account_id: Option<AccountId>`. These types are not imported
  or re-exported from the facilities crate and do not appear in
  `value_objects.rs`. The `docs/specs/facilities/value-objects.md`
  spec does not list `ExpenseHeadId` or `AccountId` for the
  facilities domain — these are finance-domain identifiers
  leaking into a facilities command shape. Cross-domain
  identifier references should go through the finance port trait
  (`docs/ports/finance.md`), not through the facilities command
  surface.

**Expected:**

Facilities commands reference finance via a port
  trait; the `ExpenseHeadId`/`AccountId` fields are removed from
  the wire-form command structs.

**Evidence:**

`crates/domains/facilities/src/commands.rs:500-535`
  — field block of `ReceiveItemCommand`. The spec command block
  at `docs/specs/facilities/commands.md:392-411` lists these two
  fields as optional finance references.

---

### FINDING 27 (id: `DOM-FAC-027`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** Medium
- **Area:** domains-facilities
- **Location:** `crates/domains/facilities/src/entities.rs:35-440`
  (entity definitions) and `crates/domains/facilities/src/aggregate.rs`

**Description:**

The 11 spec entities
  (`RouteStop`, `RoomAssignment`, `ItemIssueLine`,
  `ItemReceiveLine`, `ItemSellLine`, `StockMovement`,
  `DriverAssignment`, `SupplierContact`, `TransportMembership`,
  `DormitoryNote`, `StoreStocktake`) exist as `pub struct`
  definitions in `entities.rs`, but the spec's `entities.md`
  declares `StoreStocktake` carries "one or more
  `StoreStocktakeLine` entities" — the
  `StoreStocktakeLine` struct is not declared anywhere
  (`grep -n "StoreStocktakeLine" crates/domains/facilities/src/`
  returns only doc-comment references at `entities.rs:623`).

**Expected:**

`pub struct StoreStocktakeLine { ... }` in
  `entities.rs`.

**Evidence:**

`grep -cE "^pub struct [A-Z]" crates/domains/facilities/src/entities.rs`
  returns 13 (11 spec entities + TransportSpec + HostelSpec +
  MoneySpec helpers) but no `StoreStocktakeLine`.

---

### FINDING 28 (id: `DOM-FAC-028`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** Medium
- **Area:** domains-facilities
- **Location:** `crates/domains/facilities/src/services.rs:1-1500`
  (entire file) and
  `crates/domains/facilities/src/events.rs`

**Description:**

Spec mandates cross-domain subscribers at
  `docs/specs/facilities/events.md` "Subscribers" sections:
  `finance` subscribes to `VehicleAssigned`,
  `StudentAssignedToRoute`, `StudentAssignedToRoom`,
  `ItemReceived`, `ItemSold`, `SupplierCreated`;
  `communication` subscribes to `StudentAssignedToRoute`,
  `StudentAssignedToRoom`, `ItemReceived`;
  `attendance` subscribes to `StudentAssignedToRoute`. No
  subscriber wiring exists in the facilities crate (and the
  cross-cutting subscriber wiring is also absent — verified
  `grep -rn "StudentAssignedToRoute\|StudentAssignedToRoom"
  crates/adapters/event-bus/ crates/cross-cutting/` returns no
  rows for subscriber registrations).

**Expected:**

Subscriber wiring in
  `educore-event-bus`/`educore-events` for every event listed in
  the spec's "Subscribers" blocks.

**Evidence:**

`grep -rn "StudentAssignedToRoute\|StudentAssignedToRoom\|ItemReceived\|ItemSold\|SupplierCreated\|VehicleAssigned"
  crates/cross-cutting/ crates/adapters/event-bus/ 2>/dev/null`
  returns no subscriber registration rows.

---

### FINDING 29 (id: `DOM-FAC-029`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** Medium
- **Area:** domains-facilities
- **Location:** `crates/domains/facilities/src/lib.rs:11-30`
  (module docstring) and `docs/specs/facilities/overview.md`
  Aggregate Roots table

**Description:**

`lib.rs` module docstring is sparse (only the
  package name + a 3-line description), while
  `docs/specs/facilities/overview.md` mandates the overview's
  boundaries, dependencies, and anti-goals be reflected in the
  crate-level rustdoc. The current `lib.rs:1-10` provides none
  of the boundaries, invariants, or anti-goals documented in
  `overview.md:55-94`.

**Expected:**

A comprehensive `//!` module docstring covering
  purposes, boundaries, dependencies, invariants, and anti-goals.

**Evidence:**

`crates/domains/facilities/src/lib.rs:1-10`:
  ```rust
  //! # educore-facilities
  //!
  //! Transport vehicles and routes, dormitories and rooms, inventory
  //! items and movements, suppliers.
  //!
  //! This crate is a member of the Educore workspace. See
  //! `docs/architecture.md` and the domain spec in
  //! `docs/specs/facilities/` for behavioral details.
  ```

---

### FINDING 30 (id: `DOM-FAC-030`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** Low
- **Area:** domains-facilities
- **Location:** `crates/domains/facilities/src/repository.rs:64-322`
  (repository traits)

**Description:**

`RouteRepository` is the only port trait whose
  method `find(school, year, title)` signature returns
  `Result<Option<Route>>` rather than `Result<Route>` — this is
  consistent with the spec at `repositories.md`, but
  `AssignVehicleRepository::find` returns
  `Result<Option<AssignVehicle>>` matching the spec's "(vehicle,
  year) is unique". However, `SupplierRepository::find_by_name`,
  `ItemRepository::get_by_sku`, and `VehicleRepository::get_by_number`
  all return `Result<Option<T>>` — fine, but the spec at
  `repositories.md` adds an explicit
  `list_active(school) -> Result<Vec<Supplier>>` and
  `list_active(school) -> Result<Vec<Vehicle>>` method on the
  `Supplier` and `Vehicle` repos; both are present in code.
  However, `ItemRepository` lacks `list_active` even though
  spec's `commands.md` references "every item on the lines is
  active" (Inventory Receive Workflow pre-condition), implying
  the active filter is needed.

**Expected:**

`ItemRepository::list_active(school: SchoolId) -> Result<Vec<Item>>`
  present alongside `list(school)`.

**Evidence:**

`crates/domains/facilities/src/repository.rs:184-207` —
  `ItemRepository` has `get`, `get_by_sku`, `list`,
  `list_for_category`, `insert`, `update`, `delete`,
  `adjust_stock` — no `list_active`.

---

### FINDING 31 (id: `DOM-FAC-031`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** Low
- **Area:** domains-facilities
- **Location:** `docs/coverage.toml:1243-1250`
  (`facilities_items_aggregate` row)

**Description:**

The coverage row for facilities references a
  single aggregate `facilities_items` — but the spec defines 15
  aggregates. Coverage tracking is incomplete; only one aggregate
  row exists for the facilities domain. The audit checklist
  expects coverage tracking to mirror the full aggregate list.

**Expected:**

15 coverage rows for facilities, one per
  aggregate, mirroring the spec's Aggregate Roots table
  (`docs/specs/facilities/overview.md:107-127`).

**Evidence:**

`grep -n "facilities" docs/coverage.toml` returns
  a single row block (lines 1243-1250) for the
  `facilities_items_aggregate` id only; the remaining 14
  facilities aggregates (`Vehicle`, `Route`, `AssignVehicle`,
  `Dormitory`, `Room`, `RoomType`, `ItemCategory`, `ItemStore`,
  `ItemReceive`, `ItemReceiveChild`, `ItemIssue`, `ItemSell`,
  `ItemSellChild`, `Supplier`) are not represented.

---

### FINDING 32 (id: `DOM-FAC-032`)

- **Source:** `docs/audit_reports/findings/wave1-facilities.md`
- **Severity:** Low
- **Area:** domains-facilities
- **Location:** `crates/domains/facilities/src/aggregate.rs:1108-1182`
  (`ItemSellChild`) and `crates/domains/facilities/src/aggregate.rs:832-905`
  (`ItemReceiveChild`)

**Description:**

`ItemReceiveChild` and `ItemSellChild` are
  declared as `pub struct` aggregate roots
  (`aggregate.rs:832, 1108`), but the spec at
  `docs/specs/facilities/aggregates.md` ItemReceiveChild and
  ItemSellChild sections explicitly state the lines are
  "owned children" of `ItemReceive` / `ItemSell` and "do not emit
  a separate domain event". Per `AGENTS.md` "Module Layout (per
  domain)" and the engine's no-coupling rule for child
  aggregates, these should live in `entities.rs` (as
  non-rooted children) and not as standalone aggregate roots.
  Having them in `aggregate.rs` and re-exported from
  `lib.rs:48` (`ItemReceiveChild`, `ItemSellChild`) makes them
  appear to be independent consistency boundaries, which is
  incorrect.

**Expected:**

`ItemReceiveChild` and `ItemSellChild` moved to
  `entities.rs`; their `Aggregate` roots are `ItemReceive` and
  `ItemSell` only.

**Evidence:**

`crates/domains/facilities/src/lib.rs:48` re-exports
  `ItemReceiveChild` and `ItemSellChild` from
  `crate::aggregate::{...}`; the spec at
  `docs/specs/facilities/aggregates.md` ItemReceiveChild "Owned
  Children" block identifies the line as a child of `ItemReceive`.

---


## Finance (target id prefix: `DOMAIN-FIN`)

**Path:** `crates/domains/finance/`  
**Total findings:** 85 (12 critical, 57 high, 16 medium, 0 low)


### FINDING DOMAIN-FIN-001 (id: `DOMAIN-FIN-001`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/aggregate.rs:650-829` (39 macro stubs)

**Description:**

47 of the 52 aggregates declared in `docs/specs/finance/aggregates.md` are emitted as empty placeholder stubs via the `finance_aggregate_stub!` macro and contain only a `_id: ()` field plus `school_id`. Only 5 aggregates (`Wallet`, `WalletTransaction`, `FeesInvoice`, `FeesPayment`, `Expense`) are real.

**Expected:**

"Every aggregate in `docs/specs/finance/aggregates.md` has a Rust struct + tests" (Phase 7 exit criterion #1 in `docs/build-plan.md:914-915`). All 38 root aggregates (e.g. `FeesGroup`, `FeesMaster`, `BankAccount`, `BankStatement`, `Donor`, `Income`, `PayrollPayment`, `ChartOfAccount`, `QuestionBankFee`, `FmFeesInvoice`) should be first-class structs with fields, state machines, and tests.

**Evidence:**

Spec lists 38 aggregates in `docs/specs/finance/aggregates.md:3-1569`. Code emits 39 macro stubs at `aggregate.rs:649-829`, e.g. `pub struct FeesGroup { _id: () }`, `pub struct BankAccount { _id: () }`, etc. The handoff acknowledges this as "the intentional Workstreams D-M backlog" (`PHASE-7-HANDOFF.md:500-507`).

---

### FINDING DOMAIN-FIN-002 (id: `DOMAIN-FIN-002`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/aggregate.rs:424-460` (`FeesPayment` struct)

**Description:**

`FeesPayment` is missing the `assign_id` (or `fees_assign_id`), `student_id`, `record_id`, `slip_id`, `receipt_number`, and `is_reversed` fields that the spec mandates. The spec defines a payment as "A single payment against a `FeesAssign` (or a `FeesInstallmentAssign`) ... captures the amount, mode, slip reference, discount applied, and fine paid at the time of payment" (`docs/specs/finance/aggregates.md:316-318`).

**Expected:**

Spec mandates `fees_assign_id: FeesAssignId`, `student_id: StudentId`, `record_id: StudentRecordId` per `docs/specs/finance/events.md:166-182` (`PaymentReceived` event payload). Spec also requires `payment_mode` (PaymentMethodId), `slip` (Option<SlipReference>), and an `is_reversed` flag for reversal tracking.

**Evidence:**

`aggregate.rs:424-460` defines only `id, school_id, amount_minor, currency, discount_minor, fine_minor, payment_method, bank_id, payment_method_id, reference, note, payment_date, version, etag, ...`. `events.rs:423-435` (`PaymentReceived`) is missing `assign_id`, `student_id`, `record_id`, `slip`, `transaction_id`, `note`. The `record_payment` service (`services.rs:435-481`) takes no assignment reference.

---

### FINDING DOMAIN-FIN-009 (id: `DOMAIN-FIN-009`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/` (entire crate — no subscriber)

**Description:**

The academic domain's Promotion workflow requires Finance to subscribe to `StudentAdmitted` and `StudentPromoted` to create `FeesAssign` rows and carry-forward prior-year balances (`docs/specs/academic/workflows.md:13-14`: "Auto-create fees assignment (Finance subscribes to StudentAdmitted)"). No such subscriber exists in the finance crate or anywhere in the codebase.

**Expected:**

`docs/specs/academic/workflows.md:13-14` mandates "Auto-create fees assignment (Finance subscribes to StudentAdmitted)". `docs/specs/finance/overview.md:156-175` requires Finance to handle `StudentAdmitted`, `StudentPromoted`, `StudentWithdrawn` events.

**Evidence:**

`grep -rn "StudentAdmitted\|StudentPromoted" crates/domains/finance/src/` returns no matches. The `educore-event-bus` dependency is declared in `Cargo.toml:17` but never used (`grep "educore_event_bus" crates/domains/finance/src/` returns nothing). The handoff (`PHASE-7-HANDOFF.md:387-394`) mentions "HR→finance payroll bridge subscribes to `hr.payroll.paid` on the bus" but no actual subscriber code is present.

---

### FINDING DOMAIN-FIN-015 (id: `DOMAIN-FIN-015`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/aggregate.rs:530-618` (`Expense` aggregate)

**Description:**

`Expense` aggregate stores an `Option<PayrollPaymentId>` field (`aggregate.rs:556`) but has no referential integrity check that the payroll payment exists or is paid. Per spec invariant 3 of `docs/specs/finance/aggregates.md:1065-1083`, "A payment creates a corresponding `Expense` and `BankStatement` on approval." No code enforces this.

**Expected:**

Spec requires the expense to be derived from a real, approved payroll payment; the invariant should be enforced at construction.

**Evidence:**

`aggregate.rs:588-595` validates only `validate_ledger_name(&name)` and `amount_minor < 0`. No check on `payroll_payment_id`.

---

### FINDING DOMAIN-FIN-017 (id: `DOMAIN-FIN-017`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/finance/tests/` (directory absent)

**Description:**

The crate has no integration test directory at `crates/domains/finance/tests/`. AGENTS.md mandates "At least one integration test added for new behavior" per PR. The finance integration test lives at `crates/tools/storage-parity/tests/finance_integration.rs`, not in the domain crate.

**Expected:**

Per AGENTS.md Validation Checklist: "At least one integration test added for new behavior". Domain crate should have its own `tests/` directory.

**Evidence:**

`ls /home/beznet/Workspace/smscore/crates/domains/finance/tests/` → "No such file or directory". The integration test is at `crates/tools/storage-parity/tests/finance_integration.rs`.

---

### FINDING DOMAIN-FIN-018 (id: `DOMAIN-FIN-018`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/aggregate.rs:124-180` (`Wallet::apply_credit`, `Wallet::apply_debit`)

**Description:**

`Wallet::apply_credit` and `Wallet::apply_debit` use `saturating_add`/`saturating_sub` (lines 141, 174) which can silently swallow integer overflow. With `balance_minor: i64` and a `WalletTxType::Deposit` of `i64::MAX`, the balance would saturate at `i64::MAX`. This is a financial correctness issue — a real production system must not silently cap balances.

**Expected:**

Money must be `MinorUnits` (i64 cents/paisa) per `docs/build-plan.md:924-927` "Risks" — "All amounts are `MinorUnits` (i64 cents/paisa). The `as` ban (per `AGENTS.md`) is enforced." Overflow handling must be explicit, not silent.

**Evidence:**

`aggregate.rs:141`: `self.balance_minor = self.balance_minor.saturating_add(amount_minor);` — silent overflow. `aggregate.rs:174`: `self.balance_minor = self.balance_minor.saturating_sub(amount_minor);` — silent underflow. No upper-bound check before arithmetic; no `Result` returned on overflow.

---

### FINDING DOMAIN-FIN-034 (id: `DOMAIN-FIN-034`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/services.rs:237-291` (`deduct_wallet_credit`)

**Description:**

`deduct_wallet_credit` validates that the wallet has sufficient balance at Pending-creation time but does NOT reserve the funds. After this returns, the wallet's `balance_minor` is unchanged, so concurrent debit requests can be approved before the first one's approval completes, leading to a balance going negative at approval time.

**Expected:**

Per `docs/specs/finance/workflows.md:304-314` (Wallet Debit workflow), step 3: "The system creates a WalletTransaction in `pending` state." Step 4: "Approver approves; the wallet is debited." The Pending transaction must hold a reservation, or concurrent approvals must be serialized.

**Evidence:**

`services.rs:238-291`: returns `(WalletTransaction, WalletDebited)` after validating balance. No mutation of `wallet.balance_minor`. `aggregate.rs:151-180` (`apply_debit`) is the only place where balance decreases. The dispatch path is not provided.

---

### FINDING DOMAIN-FIN-040 (id: `DOMAIN-FIN-040`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/aggregate.rs:580-583` (`Expense::file_reference`)

**Description:**

`Expense::file_reference` is typed as `Option<Uuid>` but per `docs/specs/finance/aggregates.md:850-851` ("A receipt or file attached to a bank statement" / `BankStatementAttachment`), the file should be a typed `FileReference` from the file-storage port (`educore-files`, Phase 15). Using raw `Uuid` loses type safety.

**Expected:**

Spec uses typed `FileReference` for receipt/slip attachments.

**Evidence:**

`aggregate.rs:551`: `pub file_reference: Option<Uuid>,` — raw UUID, not a typed `FileReference`.

---

### FINDING DOMAIN-FIN-046 (id: `DOMAIN-FIN-046`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/value_objects.rs:540-545` (`DiscountAmount`, `Balance`)

**Description:**

`DiscountAmount` is defined as `pub type DiscountAmount = FeeAmount;` (line 542) and `Balance` is `pub type Balance = Amount;` (line 545). Type aliases do NOT provide type safety — a `Balance` and an `Amount` are interchangeable. AGENTS.md mandates "Compile-time safety over strings" and forbids `HashMap<String, T>`; type aliases undermine this.

**Expected:**

Spec mandates distinct types. Per `docs/specs/finance/value-objects.md:77-80`: "`DiscountAmount` is `Amount` constrained to `0..=1_000_000.00`" — should be a newtype, not an alias.

**Evidence:**

`value_objects.rs:542`: `pub type DiscountAmount = FeeAmount;` — type alias, not newtype.

---

### FINDING DOMAIN-FIN-047 (id: `DOMAIN-FIN-047`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/value_objects.rs:687` (`WalletTxStatus`)

**Description:**

`WalletTxStatus` is `pub type WalletTxStatus = ApprovalStatus;` (line 687). Per `docs/specs/finance/value-objects.md:122-126`, `WalletTxStatus` has values `Pending`, `Approved`, `Rejected` — matching `ApprovalStatus`. However, the spec lists them as separate enum types in the value-object catalog. Using a type alias loses semantic distinction between "wallet transaction approval" and "generic approval".

**Expected:**

Per spec, both are independent types in the value-object catalog.

**Evidence:**

`value_objects.rs:687`: `pub type WalletTxStatus = ApprovalStatus;` — alias.

---

### FINDING DOMAIN-FIN-065 (id: `DOMAIN-FIN-065`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/services.rs:763-800` (`FeesCarryForwardSetting`)

**Description:**

`FeesCarryForwardSetting` is defined in `services.rs` as a service-local type (`pub struct FeesCarryForwardSetting { pub title: String, pub fees_due_days: u16 }`), not as a real aggregate. The corresponding `FeesCarryForwardSetting` aggregate at `aggregate.rs:822-824` is a 1-field stub.

**Expected:**

The setting should be a first-class aggregate with the spec's fields (title, fees_due_days, payment_gateway reference) per `docs/specs/finance/aggregates.md:1520-1543`.

**Evidence:**

`services.rs:775-800` defines the setting as a service-local struct, separate from the placeholder aggregate at `aggregate.rs:822-824`.

---

### FINDING DOMAIN-FIN-080 (id: `DOMAIN-FIN-080`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** Critical
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/services.rs:1240-1306` (proptest)

**Description:**

The `prop_double_entry_invariant` proptest uses random `i64` values but uses `debits.clone()` (line 1277) implicitly via the proptest framework. The test only validates 2 cases: balanced journal passes and unbalanced journal fails. No test covers:
  - Mixed-school isolation (a row from school A should not affect school B's invariant check)
  - Empty journal
  - Single row (debit without credit or vice versa)
  - Overflow handling

**Expected:**

Production-grade property tests with edge cases.

**Evidence:**

`services.rs:1260-1306`: only 2 proptest cases (balanced/unbalanced). No isolation test. No empty test. No overflow test.

---

### FINDING DOMAIN-FIN-003 (id: `DOMAIN-FIN-003`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/commands.rs:267-1243` (event section absent)

**Description:**

Approximately 105 of the 110+ events declared in `docs/specs/finance/events.md` and `docs/events/finance.md` are missing from `events.rs`. Only 10 of ~110+ spec events are implemented.

**Expected:**

All event types from the spec table in `docs/events/finance.md:17-188` (e.g. `FeesGroupCreated`, `FeesAssignedToClass`, `PaymentReversed`, `BankAccountOpened`, `BankStatementRecorded`, `FundsTransferred`, `FmFeesInvoiceGenerated`, `PayrollGenerated`, `DonorRegistered`, `TransactionRecorded`, etc.) must be defined.

**Evidence:**

`events.rs:1-700` defines only `WalletCreated`, `WalletCredited`, `WalletDebited`, `WalletRefundRequested`, `WalletTransactionApproved`, `WalletTransactionRejected`, `InvoiceNumberingConfigured`, `PaymentReceived`, `ExpenseRecorded`, `PayrollPaymentRecorded`. The spec catalog (`docs/events/finance.md`) lists 110 events.

---

### FINDING DOMAIN-FIN-004 (id: `DOMAIN-FIN-004`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/commands.rs` (multiple sections)

**Description:**

~30 command shapes specified in `docs/specs/finance/commands.md` are missing from `commands.rs`. Critical missing commands include `AssignFeesToClassCommand`, `AssignFeesToStudentCommand`, `UpdateFeesAssignDiscountCommand`, `CloseFeesAssignCommand`, `PayInvoiceCommand`, `PayInstallmentCommand`, `ConfigureDirectFeesInstallmentCommand`, `AssignDirectInstallmentCommand`, `PayDirectInstallmentCommand`, `ConfigureDirectFeesCommand`, `ConfigureFeesReminderCommand`, `RecordBankStatementCommand`, `GenerateBankPaymentSlipCommand`, `ApproveBankPaymentCommand`, `RejectBankPaymentCommand`, `TransferFundsCommand`, `RecordIncomeCommand`, `RegisterDonorCommand`, `UpdateDonorCommand`, `DeleteDonorCommand`, `AddWalletCreditCommand`, `RecordPayrollPaymentCommand`, `RecordInventoryPaymentCommand`, `RecordProductPurchaseCommand`, `RecordProductPaymentCommand`, `ConfigureInvoiceSettingsCommand`, `ConfigurePaymentGatewayCommand`, `AttachFeesToQuestionBankCommand`, `CreateChartOfAccountCommand`, `CreateSalaryTemplateCommand`, `SetHourlyRateCommand`, `AddFeesInstallmentCreditCommand`, `ConsumeFeesInstallmentCreditCommand`.

**Expected:**

Spec defines 65+ commands in `docs/specs/finance/commands.md:10-988` (e.g. `CreateFeesGroupCommand`, `AssignFeesToClassCommand`, `PayInvoiceCommand`, etc.). The commands catalog at `docs/commands/finance.md:15-128` lists the full set.

**Evidence:**

Code defines ~50 command structs in `commands.rs:267-1243`. Many spec commands (`AssignFeesToClassCommand`, `PayInvoiceCommand`, `TransferFundsCommand`, etc.) have no matching struct.

---

### FINDING DOMAIN-FIN-005 (id: `DOMAIN-FIN-005`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/value_objects.rs` (missing types)

**Description:**

~50 value object types declared in `docs/specs/finance/value-objects.md` are missing from `value_objects.rs`. Examples: `WeaverAmount`, `ServiceCharge`, `TotalEarning`, `TotalDeduction`, `GrossSalary`, `NetSalary`, `BasicSalary`, `Tax`, `HourlyRate`, `OvertimeRate`, `FeePercentage`, `DiscountPercentage`, `TaxPercentage`, `ServiceChargeType`, `PerThousand`, `RoundingPolicy`, `InvoiceNumber`, `InvoicePrefix`, `InvoiceStartForm`, `ReceiptNumber`, `ReferenceNumber`, `SlipReference`, `InvoiceType`, `InvoicePosition`, `InvoiceCopy`, `SignatureSlot`, `PayrollStatus`, `GatewayName`, `PaymentDirection`, `BankAccountNumber`, `IfscCode`, `ChequeNumber`, `TransactionId`, `BankName`, `BranchName`, `AccountHolderName`, `OpeningBalance`, `CurrentBalance`, `DiscountCode`, `DiscountName`, `PayPeriod`, `EarnDeducType`, `PayrollNote`, `SalaryGrade`, `HouseRent`, `ProvidentFund`, `CarryForwardAmount`, `FeesDueDays`, `CarryForwardTitle`, `DaysBeforeDue`, `NotificationChannel`, `ReminderTitle`, `BlockSource`, `CreditAmount`, `CreditStatus`, `QuestionBankType`, `QuestionBankStatus`, `PaymentType`, `AccountDirection`, `FmPaymentType`, `DonorName`, `DonorProfession`, `DonorAddress`, `ShowPublic`, `ProductPackage`, `ExpiryDate`, `PurchaseDate`, `ItemReceiveId`, `ItemSellId`, `StatementDetails`, `AfterBalance`.

**Expected:**

All value objects in spec `docs/specs/finance/value-objects.md:12-258` must be implemented as typed wrappers (per AGENTS.md "Compile-time safety over strings").

**Evidence:**

`value_objects.rs:1-1225` implements only `Currency`, `Money`, `Amount`, `FeeAmount`, `FineAmount`, `DiscountAmount`, `Balance`, 10 enums, and 6 validator functions. Spec table lists ~80 value objects total.

---

### FINDING DOMAIN-FIN-006 (id: `DOMAIN-FIN-006`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/entities.rs:1-453`

**Description:**

32 of 37 child entities declared in `docs/specs/finance/entities.md` are missing. Only 5 are implemented (`WalletTransactionApproval`, `FeesPaymentSlip`, `PayrollPaymentApproval`, `AmountTransferLeg`, `BankStatementAttachment`).

**Expected:**

Spec defines 37 child entities at `docs/specs/finance/entities.md:5-303` (e.g. `FeesPaymentFine`, `FeesInstallmentAssignDiscount`, `DirectFeesInstallmentAssignChild`, `FmFeesInvoiceLineNote`, `FmFeesTransactionLineNote`, `BankPaymentSlipAudit`, `ExpenseApproval`, `IncomeApproval`, `DirectFeesInstallmentDueLog`, `FeesAssignClosure`, `FeesAssignDiscountApplication`, `DonorPhoto`, `DonorCustomField`, `ProductPurchasePayment`, `InventoryPaymentReference`, `ChartOfAccountBalance`, `QuestionBankFeeMapping`, `PaymentGatewayMode`, `FeesReminderDispatch`, `DirectFeesSettingOverride`, `FeesInstallmentCreditApplication`, `BankPaymentSlipCounter`, `FeesDiscountEligibility`, `PayrollEarningType`, `PayrollDeductionType`, `LeaveDeductionInfo`, `HourlyRateRow`, `InventoryItemReceive`, `InventoryItemSell`, `BankStatementGroup`, `PayrollPaymentReceipt`, `QuestionBankMuOption`).

**Evidence:**

`entities.rs:1-453` implements only 5 child entities. The spec lists 37. 32 are missing.

---

### FINDING DOMAIN-FIN-007 (id: `DOMAIN-FIN-007`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/services.rs` (missing services)

**Description:**

11 of the 13 domain services declared in `docs/specs/finance/services.md` are missing. Only `WalletService`, `LateFeeService`, `CarryForwardService`, `DoubleEntryService` exist. Missing: `FeesMasterService`, `InvoiceGenerationService`, `PaymentService`, `InstallmentService`, `PayrollCalculationService`, `BankReconciliationService`, `DiscountService`, `InvoiceNumberingService`, `ReminderDispatchService`, `BankSlipService`, `AccountClosingService`, `ChartOfAccountService`, `FinanceCoordinator`.

**Expected:**

All services from `docs/specs/finance/services.md:5-314` (13 service structs: `FeesMasterService`, `InvoiceGenerationService`, `PaymentService`, `InstallmentService`, `CarryForwardService`, `PayrollCalculationService`, `BankReconciliationService`, `DiscountService`, `WalletService`, `InvoiceNumberingService`, `ReminderDispatchService`, `BankSlipService`, `AccountClosingService`, `ChartOfAccountService`).

**Evidence:**

`services.rs:1-1307` implements only `WalletService`, `LateFeeService`, `CarryForwardService`, `DoubleEntryService`. The spec mandates 13+ service structs.

---

### FINDING DOMAIN-FIN-008 (id: `DOMAIN-FIN-008`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/services.rs:317-365` (`approve_wallet_transaction`)

**Description:**

The `approve_wallet_transaction` service approves the `WalletTransaction` state machine but does NOT apply the credit/debit to the actual `Wallet` aggregate. The doc comment at lines 313-316 explicitly says "The caller is responsible for applying the credit/debit to the `Wallet` aggregate". This means the engine does not provide a helper to atomically approve-and-apply, creating a window between approval and balance update where double-spend can occur.

**Expected:**

Per `docs/specs/finance/workflows.md:283-294` ("Wallet Credit" workflow), step 4 should be atomic: "The system credits the wallet and emits WalletTransactionApproved." The service should perform the credit/debit on the wallet alongside the state transition.

**Evidence:**

`services.rs:317-338` only calls `tx.approve(approver, now, event_id)?` and returns the event. It does not call `wallet.apply_credit()` or `wallet.apply_debit()`. Aggregate comment at line 313-316 confirms this gap: "The caller is responsible for applying the credit/debit to the `Wallet` aggregate".

---

### FINDING DOMAIN-FIN-010 (id: `DOMAIN-FIN-010`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/query.rs` (entire file)

**Description:**

All 11 query stubs return `Err(DomainError::not_supported(...))`. No actual query execution is implemented. Per the file's doc comment at line 4, this is "a Phase 7 stub" that defers to Phase 17, but no test exercises a successful query path.

**Expected:**

Queries should compile and execute against a repository; the `#[derive(DomainQuery)]` macro is documented as the path for production queries (`docs/specs/finance/repositories.md:5-7`).

**Evidence:**

`query.rs:46-52` (`WalletQuery::execute`), `82-86` (`WalletTransactionQuery::execute`), `111-115` (`FeesPaymentQuery::execute`), `184-188` (`FeesInvoiceQuery::execute`), `258-263` (`ExpenseQuery::execute`), `333-338` (`IncomeQuery::execute`), `407-412` (`BankStatementQuery::execute`), `475-479` (`PayrollPaymentQuery::execute`), `541-547` (`FeesCarryForwardQuery::execute`), `601-606` (`BankAccountQuery::execute`), `676-681` (`TransactionQuery::execute`) all return `Err(DomainError::not_supported(...))`.

---

### FINDING DOMAIN-FIN-011 (id: `DOMAIN-FIN-011`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/commands.rs:267-289` (`CreateFeesGroupCommand`)

**Description:**

`CreateFeesGroupCommand` is missing the `start_date`, `end_date`, and `due_date` fields mandated by the spec. The spec requires all four fields, with pre-conditions "start_date <= due_date <= end_date".

**Expected:**

`docs/specs/finance/commands.md:14-22`: `pub struct CreateFeesGroupCommand { pub tenant: TenantContext, pub name: String, pub description: Option<String>, pub start_date: NaiveDate, pub end_date: NaiveDate, pub due_date: NaiveDate }`.

**Evidence:**

`commands.rs:267-272`: only has `tenant`, `name`, `description`. The fields are present in the duplicate `ConfigureFeesGroupCommand` at `commands.rs:1196-1204` but the canonical `CreateFeesGroupCommand` is missing them.

---

### FINDING DOMAIN-FIN-012 (id: `DOMAIN-FIN-012`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/commands.rs:299-307` (`CreateFeesTypeCommand`)

**Description:**

`CreateFeesTypeCommand` includes extra fields `amount_minor: i64` and `currency: Currency` that are not in the spec, and the spec's `fees_type_id` reference is missing.

**Expected:**

`docs/specs/finance/commands.md:42-47`: `pub struct CreateFeesTypeCommand { pub tenant: TenantContext, pub fees_group_id: FeesGroupId, pub name: String, pub description: Option<String> }`.

**Evidence:**

`commands.rs:299-307` has 6 fields instead of 5; the extra `amount_minor` and `currency` are not spec-defined.

---

### FINDING DOMAIN-FIN-013 (id: `DOMAIN-FIN-013`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/commands.rs:332-340` (`CreateFeesMasterCommand`)

**Description:**

`CreateFeesMasterCommand` is missing the `fees_type_id`, `section_id`, and `academic_id` fields mandated by the spec. The spec requires a master to be uniquely keyed by `(fees_group_id, fees_type_id, class_id, section_id?, academic_id)`.

**Expected:**

`docs/specs/finance/commands.md:55-66`: `pub struct CreateFeesMasterCommand { pub tenant: TenantContext, pub fees_group_id: FeesGroupId, pub fees_type_id: FeesTypeId, pub class_id: ClassId, pub section_id: Option<SectionId>, pub academic_id: AcademicYearId, pub amount: FeeAmount, pub due_date: Option<NaiveDate> }`. The invariant at `docs/specs/finance/aggregates.md:87-88` requires all 5 identity fields.

**Evidence:**

`commands.rs:332-340` has only `fees_group_id`, `class_id`, `amount_minor`, `currency`, `due_date`. Missing `fees_type_id`, `section_id`, `academic_id`.

---

### FINDING DOMAIN-FIN-016 (id: `DOMAIN-FIN-016`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/aggregate.rs:818` (test stub marker)

**Description:**

The 39 placeholder stub aggregates are explicitly marked as a "backlog" (`aggregate.rs:988-1008`) and the single test at line 1010 is `#[ignore = "backlog: 33 placeholder aggregates need Workstreams D-M"]`. No tests exist for the placeholder aggregates.

**Expected:**

Phase 7 exit criterion #1 (`docs/build-plan.md:914-915`): "Every aggregate in `docs/specs/finance/aggregates.md` has a Rust struct + tests."

**Evidence:**

`aggregate.rs:1010-1016`: `#[test] #[ignore = "backlog: 33 placeholder aggregates need Workstreams D-M"] fn unimplemented_placeholder_aggregates_backlog()` — body is empty. 47 stub aggregates have zero tests.

---

### FINDING DOMAIN-FIN-019 (id: `DOMAIN-FIN-019`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/value_objects.rs:361-370` (`Currency::as_str`)

**Description:**

`Currency::as_str` uses `std::str::from_utf8(&self.0).ok().unwrap_or("XXX")` with a misleading comment. The comment at lines 364-369 says "the `expect` is unavoidable without `unsafe`" but the code does NOT use `expect` — it uses `unwrap_or("XXX")`. The fallback `"XXX"` is silently wrong for any non-UTF8 bytes (which the constructor prevents, but the doc lies about the implementation).

**Expected:**

Documentation/comments must accurately reflect behavior. Use `expect` or return a `Result`.

**Evidence:**

`value_objects.rs:369`: `std::str::from_utf8(&self.0).ok().unwrap_or("XXX")` — comment at lines 362-368 says "the `expect` is unavoidable without `unsafe`" but no `expect` is present.

---

### FINDING DOMAIN-FIN-021 (id: `DOMAIN-FIN-021`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/services.rs:882-892` (`LateFeeSettings`)

**Description:**

`LateFeeSettings` has no `new()` constructor and no validation. `LateFeeKind::FixedAmount(i64)` can be constructed with negative values; `LateFeeKind::PerDayRate(i64)` can be negative; `LateFeeKind::PercentOfAmount(u8)` allows 0-255 (spec mandates 0-100). The service silently masks negatives with `.max(0)` at lines 907-911.

**Expected:**

Per `docs/specs/finance/value-objects.md:91-100`: `FeePercentage` is `f32` in `[0, 100]`; `RoundingPolicy` is `HalfUp | HalfEven | Truncate`. Per spec rules, late-fee values must be validated at construction.

**Evidence:**

`services.rs:887-892`: `pub struct LateFeeSettings { pub kind: LateFeeKind, pub grace_period_days: u16 }` — fields are pub, no constructor. `LateFeeKind::PercentOfAmount(u8)` at line 882 takes `u8` (0-255), exceeding spec's 0-100 range. `services.rs:907-911`: `n.max(0)` and `billable_days.saturating_mul(rate).max(0)` silently mask negatives.

---

### FINDING DOMAIN-FIN-023 (id: `DOMAIN-FIN-023`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/commands.rs:38-249` (command-type constants)

**Description:**

The command-type constants in `commands.rs` use a flattened `finance.<aggregate>.<action>` wire form, but the spec mandates a hierarchical form. For example, `FINANCE_FEES_GROUP_CONFIGURE_COMMAND_TYPE` is not present; instead the code uses `FINANCE_FEES_GROUP_CREATE_COMMAND_TYPE` for what the spec calls `FeesGroup.Create`. Spec uses dotted names like `FeesGroup.Create`.

**Expected:**

Per `docs/commands/finance.md` and `docs/specs/finance/permissions.md`, capability strings are `<Domain>.<Aggregate>.<Action>` (e.g. `FeesGroup.Create`). The constant naming should mirror.

**Evidence:**

`commands.rs:68-71`: `FINANCE_FEES_GROUP_CREATE_COMMAND_TYPE = "finance.fees_group.create"` etc. The spec's `docs/specs/finance/permissions.md:22-24` lists `FeesGroup.Create`, `FeesType.Create`, etc.

---

### FINDING DOMAIN-FIN-024 (id: `DOMAIN-FIN-024`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/value_objects.rs:553-562` (`FeesInvoiceStatus`)

**Description:**

`FeesInvoiceStatus` enum defines `Pending`, `Issued`, `Cancelled`, but the spec's payment status from `docs/specs/finance/value-objects.md:118-126` mandates `PaymentStatus` with values `Unpaid`, `Partial`, `Paid`, `Overpaid`. The code instead uses `FeesPaymentStatus` with the same values but a different name. Spec lists these as separate concepts.

**Expected:**

Spec requires both `PaymentStatus` and `PayrollStatus` and `FeesPaymentStatus` as distinct types per `docs/specs/finance/value-objects.md:118-126`.

**Evidence:**

`value_objects.rs:553-562`: `pub enum FeesInvoiceStatus { Pending, Issued, Cancelled }`. `value_objects.rs:596-606`: `pub enum FeesPaymentStatus { Unpaid, Partial, Paid, Overpaid }` — but this is `FeesPaymentStatus`, not `PaymentStatus`. The spec expects `PaymentStatus` as a generic concept.

---

### FINDING DOMAIN-FIN-025 (id: `DOMAIN-FIN-025`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/repository.rs` (multiple traits)

**Description:**

The 44 repository port traits in `repository.rs` import the placeholder stub aggregates (`aggregate.rs:649-829`). These traits cannot be implemented against real aggregate structs because the placeholders have only `_id: ()` and `school_id`. For example, `FeesGroupRepository::insert` takes `&FeesGroup`, but `FeesGroup` is a stub.

**Expected:**

Repository port traits must operate on real aggregate types so adapters can be implemented.

**Evidence:**

`repository.rs:122-137` defines `FeesGroupRepository` with `async fn insert(&self, ctx: &TenantContext, agg: &FeesGroup) -> Result<()>`, but `FeesGroup` at `aggregate.rs:651-652` is `pub struct FeesGroup { _id: () }` — a stub.

---

### FINDING DOMAIN-FIN-027 (id: `DOMAIN-FIN-027`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/aggregate.rs:13` (`#![allow(unused_imports)]`)

**Description:**

Every src file uses `#![allow(unused_imports)]` (aggregate.rs:13, value_objects.rs:13 implicit, entities.rs:9, events.rs:17, services.rs:22, commands.rs:17, repository.rs:20, query.rs:9, lib.rs:13). This blanket allowance hides unused-import warnings that would otherwise indicate dead code or refactoring needs.

**Expected:**

Per AGENTS.md: "No `#[allow(dead_code)]` or `_var` prefixes to silence the compiler. Delete unused code, wire it in, or open a follow-up issue."

**Evidence:**

`aggregate.rs:13`: `#![allow(unused_imports)]`. `commands.rs:18`: also `#![allow(dead_code)]` at line 18.

---

### FINDING DOMAIN-FIN-028 (id: `DOMAIN-FIN-028`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/commands.rs:18`

**Description:**

`commands.rs:18` has `#![allow(dead_code)]` which hides unused code. Per AGENTS.md, dead code should be deleted or wired in, not silenced.

**Expected:**

AGENTS.md: "No `#[allow(dead_code)]` or `_var` prefixes to silence the compiler."

**Evidence:**

`commands.rs:18`: `#![allow(dead_code)]`. Also `entities.rs:10`: `#![allow(dead_code)]`. Also `repository.rs:21`: `#![allow(dead_code)]`. Also `query.rs:10`: `#![allow(dead_code)]`.

---

### FINDING DOMAIN-FIN-029 (id: `DOMAIN-FIN-029`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/commands.rs:38-249` (command-type constants) and `commands.rs:267-1243` (command shapes)

**Description:**

The spec defines 65+ commands in `docs/specs/finance/commands.md`. The code defines ~50 command shapes plus 90+ command-type constants, but the canonical spec names are not followed. For example, the spec mandates `PayInvoiceCommand`, `PayInstallmentCommand`, `RecordBankStatementCommand`, `GenerateBankPaymentSlipCommand`, `ApproveBankPaymentCommand`, `RejectBankPaymentCommand`, `TransferFundsCommand`, `RecordIncomeCommand`, `RegisterDonorCommand`, `AddWalletCreditCommand`, `RecordPayrollPaymentCommand`, `AddFeesInstallmentCreditCommand`, `ConsumeFeesInstallmentCreditCommand`, etc. — all missing.

**Expected:**

Spec mandates these commands at `docs/specs/finance/commands.md` and `docs/commands/finance.md`.

**Evidence:**

`grep` confirms these command names are not in `commands.rs`. See `docs/specs/finance/commands.md:233-988` for full list of missing commands.

---

### FINDING DOMAIN-FIN-030 (id: `DOMAIN-FIN-030`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/services.rs:621-638` (`PaymentProvider`)

**Description:**

`PaymentProvider` trait is marked `#[deprecated(since = "0.1.0", ...)]` in finance crate, but the actual deprecation reason says `since = "0.7.0"` in the handoff. The trait was supposed to move to `educore-payment` in Phase 15, but is still defined in finance. The handoff acknowledges this is "Q10" outstanding work.

**Expected:**

Per `docs/handoff/PHASE-7-HANDOFF.md:565-573` Q10: "the trait and impl will be removed from `educore-finance` once `educore-payment` ships in Phase 15."

**Evidence:**

`services.rs:623-626`: `#[deprecated(since = "0.1.0", note = "moves to educore-payment in Phase 15; ...")]` — the `since` value is `0.1.0`, not `0.7.0` as documented in the handoff. The handoff (line 187-189) says `since = "0.7.0"`.

---

### FINDING DOMAIN-FIN-032 (id: `DOMAIN-FIN-032`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/aggregate.rs:424-526` (`FeesPayment`) and `events.rs:422-484` (`PaymentReceived`)

**Description:**

The `FeesPayment` aggregate and `PaymentReceived` event are missing critical fields mandated by the spec event schema. Spec event payload requires `assign_id`, `student_id`, `record_id`, `slip`, `transaction_id`, `note`. Code only has `fees_payment_id`, `amount_minor`, `currency`, `discount_minor`, `fine_minor`, `payment_method`, `bank_id`, `payment_date`. This breaks downstream subscribers (`communication`, `hr`, `assessment`) that depend on these fields per `docs/specs/finance/events.md:185-189`.

**Expected:**

Per `docs/specs/finance/events.md:166-182`, the `PaymentReceived` event must carry `assign_id: FeesAssignId`, `student_id: StudentId`, `record_id: StudentRecordId`, `slip: Option<SlipReference>`, `transaction_id: Option<TransactionId>`, `note: Option<String>`. The `communication` subscriber at line 186 needs `student_id` to look up the guardian.

**Evidence:**

`events.rs:422-435` defines `PaymentReceived` with only 8 fields. Spec requires 12 fields. Subscribers at line 185-188 need the missing fields.

---

### FINDING DOMAIN-FIN-033 (id: `DOMAIN-FIN-033`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/services.rs:54-87` (`create_wallet`)

**Description:**

The `create_wallet` service does not enforce the lazy-creation invariant. Per spec (`docs/specs/finance/services.md#walletservice`), wallets should be created lazily on the first `WalletTransaction`. The service requires the caller to invoke `create_wallet` explicitly with a `user_id` and `currency`, but there's no helper that detects "no wallet exists for (school_id, user_id)" and creates it.

**Expected:**

Spec says "Wallets are created lazily on the first wallet transaction for `(school_id, user_id)`." A `get_or_create_wallet` helper should be available.

**Evidence:**

`services.rs:54-87` (`create_wallet`): takes `CreateWalletCommand` and unconditionally creates a new wallet. No lazy creation pattern. Comment at line 51-52: "Wallets are created lazily on the first wallet transaction for `(school_id, user_id)`." is contradicted by the service signature.

---

### FINDING DOMAIN-FIN-035 (id: `DOMAIN-FIN-035`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/aggregate.rs:530-618` (`Expense` aggregate)

**Description:**

`Expense::fresh` does not validate that `currency` matches the `account_id`'s currency. A school in INR could record an expense in USD against an INR-denominated bank account. Per spec workflow `docs/specs/finance/workflows.md:177-183` ("Expense Recording"): "The system creates a BankStatement (debit) on the chosen account".

**Expected:**

Spec invariant: "The expense's `payment_method` and `account` must be compatible (cash payment → cash account; bank → bank account)" per `docs/specs/finance/aggregates.md:863-864`.

**Evidence:**

`aggregate.rs:587-617`: only validates `name` and `amount_minor >= 0`. No cross-aggregate integrity check.

---

### FINDING DOMAIN-FIN-036 (id: `DOMAIN-FIN-036`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/services.rs:1040-1064` (test case)

**Description:**

The test `deduct_wallet_rejects_insufficient_balance` demonstrates the validation works, but production-grade rounding behavior is not enforced. The `LateFeeService::compute_late_fee` uses integer division which truncates (`(amount * pct) / 100`), losing fractional minor units. No `RoundingPolicy` enum exists to select `HalfUp`, `HalfEven`, or `Truncate`.

**Expected:**

Per `docs/specs/finance/value-objects.md:91-100`, `RoundingPolicy` must be `HalfUp`, `HalfEven`, or `Truncate`.

**Evidence:**

`services.rs:907-911`: `LateFeeKind::PercentOfAmount(pct) => (i64::from(amount.amount_minor()) * i64::from(pct)) / 100` — truncate-only. No `RoundingPolicy` enum.

---

### FINDING DOMAIN-FIN-038 (id: `DOMAIN-FIN-038`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/services.rs:801-861` (`CarryForwardService`)

**Description:**

`CarryForwardService::build_carry_forward` produces a `CarryForwardDraft` but the corresponding `FeesCarryForward` aggregate is a placeholder stub (`aggregate.rs:815-816`). The dispatcher cannot persist a real `FeesCarryForward` row because the aggregate type has no fields.

**Expected:**

Spec requires `FeesCarryForward` aggregate with `student_id`, `academic_id`, `balance_minor`, `balance_type`, `due_date`, `notes` per `docs/specs/finance/aggregates.md:341-368`.

**Evidence:**

`services.rs:830-860` returns `CarryForwardDraft`. `aggregate.rs:815-816`: `pub struct FeesCarryForward { _id: () }` — stub.

---

### FINDING DOMAIN-FIN-039 (id: `DOMAIN-FIN-039`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/events.rs:1-700` (entire file)

**Description:**

The events file does not define any subscriber-side handlers for cross-domain events. The spec requires Finance to subscribe to `StudentAdmitted`, `StudentPromoted`, `StudentWithdrawn`, `StaffRegistered`, `LeaveApproved`, and `hr.payroll.paid`. None are implemented.

**Expected:**

Per `docs/specs/finance/overview.md:154-179` ("Cross-Domain Impact"): "When a `Student` is admitted, the academic domain emits `StudentAdmitted`. Finance subscribes and: Creates a `FeesAssign` per active `FeesMaster`...".

**Evidence:**

`events.rs:1-700` contains only event definitions, no subscriber functions. `grep "subscribe" crates/domains/finance/src/` returns no matches.

---

### FINDING DOMAIN-FIN-041 (id: `DOMAIN-FIN-041`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/value_objects.rs:331-338` (`Currency` constants)

**Description:**

`Currency` defines 4 constants: `INR`, `USD`, `EUR`, `GBP`. The spec says "ISO-4217 alpha-3 (e.g. `USD`, `INR`, `EUR`, `GBP`)" with same 4 examples. This is consistent. However, the handoff claims "Currency (8 ISO 4217 codes — the engine default set)" (`PHASE-7-HANDOFF.md:126`), but only 4 are defined.

**Expected:**

Per handoff: "Currency (8 ISO 4217 codes — the engine default set)". Code defines only 4 constants.

**Evidence:**

`value_objects.rs:332-338`: 4 constants defined (INR, USD, EUR, GBP). Handoff says 8.

---

### FINDING DOMAIN-FIN-043 (id: `DOMAIN-FIN-043`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/services.rs:765-770` (CarryForwardService declaration)

**Description:**

The `use` statement `use crate::value_objects::{AcademicYearId, BalanceType, FeeAmount, StudentId};` at line 771 is mid-file (not at the top). Per Rust style and AGENTS.md, imports should be at the top of the file. Additionally, `AcademicYearId`, `BalanceType`, `FeeAmount`, `StudentId` are imported but `LateFeeSettings` and `LateFeeKind` use these without re-import.

**Expected:**

Per Rust idiomatic style, all `use` statements should be at the top of the module.

**Evidence:**

`services.rs:771`: `use crate::value_objects::{AcademicYearId, BalanceType, FeeAmount, StudentId};` is mid-file, after several `pub fn` declarations.

---

### FINDING DOMAIN-FIN-044 (id: `DOMAIN-FIN-044`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/aggregate.rs:266-293` (`WalletTransaction::fresh`)

**Description:**

The `WalletTransaction::fresh` constructor takes 14 positional arguments. This makes call sites error-prone (argument order matters). Spec mandates that the engine's Rust style uses typed builders per AGENTS.md, not 14-argument constructors.

**Expected:**

Per AGENTS.md: "Domain scopes via extension traits. `.active()`, `.in_class()`, etc. are implemented as extension traits on the macro-generated builder."

**Evidence:**

`aggregate.rs:244-258`: 14-argument `fresh` constructor for `WalletTransaction`.

---

### FINDING DOMAIN-FIN-045 (id: `DOMAIN-FIN-045`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/aggregate.rs:808-829` (stub aggregates — duplicate `FeesInvoiceSetting`)

**Description:**

The macro stub block at `aggregate.rs:736` declares `FeesInvoiceSetting` and the macro stub block at `aggregate.rs:739` also declares `InvoiceSetting`. The spec distinguishes these clearly: `FeesInvoiceSetting` is for the classic scheme, `InvoiceSetting` for the FM scheme. Per the spec, both are required but as separate types. The handoff acknowledges both are stubs.

**Expected:**

Both types should be real aggregates, not stubs.

**Evidence:**

`aggregate.rs:733-740`: both `FeesInvoiceSetting` and `InvoiceSetting` are 1-field stubs.

---

### FINDING DOMAIN-FIN-049 (id: `DOMAIN-FIN-049`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/aggregate.rs:67-68` (`Wallet::balance_minor: i64`)

**Description:**

`Wallet::balance_minor: i64` is `pub` and mutable from outside the crate's services. The aggregate's invariants rely on calling `apply_credit`/`apply_debit`, but direct field mutation is possible. Per AGENTS.md, aggregates should be encapsulated.

**Expected:**

Per AGENTS.md "Strict eager loading" and aggregate encapsulation patterns, fields should be private with accessors.

**Evidence:**

`aggregate.rs:65`: `pub balance_minor: i64,` — public mutable field.

---

### FINDING DOMAIN-FIN-050 (id: `DOMAIN-FIN-050`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/services.rs:42-44` (`event_id_to_uuid` helper)

**Description:**

The `event_id_to_uuid` helper function at `services.rs:42-44` is defined at module scope and used to construct typed IDs from `EventId`. The same pattern is repeated across all service functions. This is an unsafe-feeling pattern because it relies on `EventId`'s UUID value being the canonical UUID for the new aggregate.

**Expected:**

The engine should have a dedicated `IdGenerator` API that mints typed ids directly.

**Evidence:**

`services.rs:42-44`: `fn event_id_to_uuid(e: EventId) -> uuid::Uuid { e.as_uuid() }` — used at lines 66, 117, 186, 247, 447, 513, 584.

---

### FINDING DOMAIN-FIN-051 (id: `DOMAIN-FIN-051`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/commands.rs:1220-1228` (`OpenBankAccountCommand`)

**Description:**

`OpenBankAccountCommand` is missing the `account_name`, `opening_balance` (typed), and `note` fields mandated by the spec. Spec also requires `bank_name` as a typed `BankName` value object.

**Expected:**

Per `docs/specs/finance/commands.md:447-457`: `pub struct OpenBankAccountCommand { pub tenant: TenantContext, pub bank_name: String, pub account_name: String, pub account_number: BankAccountNumber, pub account_type: AccountType, pub opening_balance: Amount, pub note: Option<String> }`.

**Evidence:**

`commands.rs:1220-1228`: only has `tenant`, `bank_name`, `account_number`, `account_type`, `opening_balance_minor`, `currency` — missing `account_name` and `note`.

---

### FINDING DOMAIN-FIN-052 (id: `DOMAIN-FIN-052`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/commands.rs:1230-1235` (`BlockLoginForDueFeesCommand`)

**Description:**

`BlockLoginForDueFeesCommand` is missing the `role_id` field mandated by the spec. The spec requires `(user_id, role_id)` to identify a unique block.

**Expected:**

Per `docs/specs/finance/commands.md:417-423`: `pub struct BlockLoginForDueFeesCommand { pub tenant: TenantContext, pub user_id: UserId, pub role_id: Option<RoleId>, pub reason: PreventReason }`.

**Evidence:**

`commands.rs:1230-1235`: only has `tenant`, `user_id`, `reason`. Missing `role_id`.

---

### FINDING DOMAIN-FIN-053 (id: `DOMAIN-FIN-053`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/commands.rs:1237-1243` (`CarryForwardFeesBalanceCommand`)

**Description:**

`CarryForwardFeesBalanceCommand` is missing the `notes`, `due_date`, and `payment_gateway` fields mandated by the spec.

**Expected:**

Per `docs/specs/finance/commands.md:349-357`: `pub struct CarryForwardFeesBalanceCommand { pub tenant: TenantContext, pub student_id: StudentId, pub academic_id: AcademicYearId, pub target_academic_id: AcademicYearId, pub notes: Option<String>, pub due_date: Option<NaiveDate>, pub payment_gateway: Option<String> }`. Also uses `student_id` and `academic_id`/`target_academic_id` field names, not `from`/`to`.

**Evidence:**

`commands.rs:1237-1243`: only has `tenant`, `student_id`, `from` (as `AcademicYearId`), `to` (as `AcademicYearId`). Field names and missing fields diverge from spec.

---

### FINDING DOMAIN-FIN-055 (id: `DOMAIN-FIN-055`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/commands.rs:38-249` and `events.rs`

**Description:**

The spec defines `docs/commands/finance.md` and `docs/events/finance.md` as the canonical command/event catalogs. These catalogs list 80+ commands and 110+ events. The code only implements ~50 command shapes and 10 events, with many using non-spec names (e.g. `CreditWalletCommand` instead of spec's `AddWalletCreditCommand`, `GenerateBankSlipCommand` instead of `GenerateBankPaymentSlipCommand`, `ApproveBankSlipCommand` instead of `ApproveBankPaymentCommand`, `CreateFeesGroupCommand` instead of `ConfigureFeesGroupCommand`).

**Expected:**

Command names should match the spec exactly per `docs/commands/finance.md`.

**Evidence:**

`services.rs:151-163` defines `CreditWalletCommand`; spec (`docs/commands/finance.md:106`) names it `AddWalletCredit`. `commands.rs:806-816` defines `GenerateBankSlipCommand`; spec names it `GenerateBankPaymentSlip`.

---

### FINDING DOMAIN-FIN-056 (id: `DOMAIN-FIN-056`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/aggregate.rs:1-1017` (test count gap)

**Description:**

The handoff claims "44 unit tests pass" but the build-plan exit criterion #1 requires "Every aggregate in `docs/specs/finance/aggregates.md` has a Rust struct + tests". With 47 placeholder aggregates, this criterion is not met. The handoff's "579 tests pass" counts cross-cutting tests, not per-aggregate tests.

**Expected:**

Per `docs/build-plan.md:914-915`, all 52 aggregates must have tests. Currently only 5 do.

**Evidence:**

`aggregate.rs:1010-1016`: single `#[ignore]` test for the entire 47-stub backlog. Real test count for aggregates: ~10 tests across 5 aggregates.

---

### FINDING DOMAIN-FIN-058 (id: `DOMAIN-FIN-058`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/services.rs:882-914` (`LateFeeService`)

**Description:**

`LateFeeKind::PercentOfAmount(u8)` uses `u8` (range 0-255) but spec mandates `0 <= x <= 100` for `FeePercentage`. The type allows `pct = 150` which would silently compute 150% of the amount as a fee, not raise an error.

**Expected:**

Per spec value-object constraint (`docs/specs/finance/value-objects.md:91-100`), percentages must be in `[0, 100]`.

**Evidence:**

`services.rs:882`: `LateFeeKind::PercentOfAmount(u8)` — `u8` allows 0-255.

---

### FINDING DOMAIN-FIN-059 (id: `DOMAIN-FIN-059`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/services.rs:809-861` (`CarryForwardService`)

**Description:**

The `CarryForwardService` returns a `CarryForwardDraft` (a non-persistent draft struct) but the corresponding `FeesCarryForward` aggregate is a stub. The draft has `balance_minor: u64` while the spec requires `balance >= 0` per `docs/specs/finance/aggregates.md:357`. The draft also has no `correlation_id`, `created_by`, etc. — not a real aggregate.

**Expected:**

A real `FeesCarryForward` aggregate with full audit footer.

**Evidence:**

`services.rs:865-874` defines `CarryForwardDraft` with `student_id`, `from`, `to`, `balance_minor`, `balance_type`, `due_date`, `note` — but this is a service-local type, not the spec-mandated aggregate.

---

### FINDING DOMAIN-FIN-060 (id: `DOMAIN-FIN-060`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/aggregate.rs:580-618` (`Expense::fresh`)

**Description:**

`Expense::fresh` does not validate `payment_method` compatibility with `account_id`. Per spec invariant: "The expense's `payment_method` and `account` must be compatible (cash payment → cash account; bank → bank account)" (`docs/specs/finance/aggregates.md:863-864`).

**Expected:**

Construction should reject `Cash` payment method with `Bank` account_type (and vice versa).

**Evidence:**

`aggregate.rs:587-617`: only validates `name` and `amount_minor`. No `payment_method` ↔ `account_type` cross-check.

---

### FINDING DOMAIN-FIN-062 (id: `DOMAIN-FIN-062`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/aggregate.rs:11` (`lib.rs:23-32` module structure)

**Description:**

`value_objects.rs` is declared `pub mod value_objects;` (lib.rs:23) but `aggregate`, `entities`, `errors`, and `repository` are `mod` (private). Consumers cannot import `educore_finance::aggregate::FeesGroup` etc., even though the prelude re-exports some types. This breaks the 9-file module layout's contract.

**Expected:**

Per AGENTS.md Module Layout: every src file should be a `pub mod` or at least re-export its key types.

**Evidence:**

`lib.rs:23-32`: `pub mod value_objects; mod aggregate; pub mod commands; mod entities; mod errors; pub mod events; pub mod query; mod repository; pub mod services;`.

---

### FINDING DOMAIN-FIN-064 (id: `DOMAIN-FIN-064`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/value_objects.rs:553-590` (`FeesInvoiceStatus`)

**Description:**

`FeesInvoiceStatus` enum (Pending, Issued, Cancelled) does not match the spec's required invoice lifecycle. Per spec at `docs/specs/finance/aggregates.md:222-238`, `FeesInvoice` has no state machine (it's a config row). But the `FmFeesInvoice` spec defines states (`Draft/Issued/PartiallyPaid/Paid/Overdue/Cancelled` per the handoff line 79-81). The current `FeesInvoiceStatus` is unused / mismatched.

**Expected:**

Spec defines no `FeesInvoiceStatus` enum; this is over-engineering. The `FmFeesInvoice` should have its own status enum.

**Evidence:**

`value_objects.rs:553-590` defines `FeesInvoiceStatus` with `Pending/Issued/Cancelled`. The spec's `FeesInvoice` aggregate has no state machine; only `FmFeesInvoice` does.

---

### FINDING DOMAIN-FIN-066 (id: `DOMAIN-FIN-066`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/aggregate.rs:425-526` (`FeesPayment::net_minor`)

**Description:**

`FeesPayment::net_minor` (line 522-525) computes `amount - discount` using `saturating_sub`. If `discount > amount`, this returns 0 silently. The spec at `docs/specs/finance/aggregates.md:323` requires "`amount >= 0` and `discount_amount >= 0` and `fine >= 0`" but does NOT mandate `discount <= amount`. This could mask data entry errors.

**Expected:**

Validation at construction should reject `discount > amount`, not silently clamp at query time.

**Evidence:**

`aggregate.rs:523-525`: `pub const fn net_minor(&self) -> i64 { self.amount_minor.saturating_sub(self.discount_minor) }`.

---

### FINDING DOMAIN-FIN-067 (id: `DOMAIN-FIN-067`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/aggregate.rs:355-356` (comment about stubs)

**Description:**

Comment at line 355-356 says "Stubs for the other 4 headline aggregates (FeesInvoice, FeesPayment, Expense) — typed-shape-only; real impl lands in subsequent workstreams per the Phase 7 plan." But FeesInvoice, FeesPayment, and Expense ARE real implementations (lines 360-618). The comment is misleading and contradicts the actual code.

**Expected:**

Comments should accurately describe code status.

**Evidence:**

`aggregate.rs:352-356`: `// Stubs for the other 4 headline aggregates (FeesInvoice, FeesPayment, // Expense) — typed-shape-only; real impl lands in subsequent // workstreams per the Phase 7 plan.` — but `FeesInvoice` (line 360), `FeesPayment` (line 424), and `Expense` (line 530) are full implementations.

---

### FINDING DOMAIN-FIN-069 (id: `DOMAIN-FIN-069`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/lib.rs:48-100` (prelude re-exports)

**Description:**

The prelude re-exports many `aggregate::*` items but does NOT include the `FeesAssignDiscount`, `FeesInstallment`, `BankAccount`, `BankStatement`, `Income`, `Donor`, etc. (the stub aggregates that the spec needs to function). Consumers cannot construct or reference these types even though the spec defines them.

**Expected:**

Per the 9-file module layout (`AGENTS.md`), the prelude should re-export all public aggregate types.

**Evidence:**

`lib.rs:48-50`: only `Expense`, `FeesInvoice`, `FeesPayment`, `Wallet`, `WalletTransaction` are re-exported. The other 47 aggregates are not re-exported because they are stubs.

---

### FINDING DOMAIN-FIN-070 (id: `DOMAIN-FIN-070`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/value_objects.rs:46` (`RbacRoleId`)

**Description:**

`RbacRoleId` is re-exported from `educore_rbac::ids::RoleId as RbacRoleId` (line 46), but `RoleId` is also re-exported directly from `educore_hr::value_objects::RoleId` (line 43). This creates two distinct role-id types in the finance crate's namespace, which can cause cross-domain confusion.

**Expected:**

Per spec, only one `RoleId` type should exist; consumers should not need both.

**Evidence:**

`value_objects.rs:43`: `pub use educore_hr::value_objects::RoleId;` and `value_objects.rs:46`: `pub use educore_rbac::ids::RoleId as RbacRoleId;` — both available in finance crate.

---

### FINDING DOMAIN-FIN-071 (id: `DOMAIN-FIN-071`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/services.rs:435-481` (`record_payment`) and `commands.rs` missing `PayInvoiceCommand`

**Description:**

The spec mandates a distinct `PayInvoiceCommand` (per `docs/specs/finance/commands.md:233-249`) with `fees_assign_id: FeesAssignId`, `amount: Amount`, `payment_method_id: PaymentMethodId`, `bank_id: Option<BankAccountId>`, `note: Option<String>`, `slip: Option<SlipReference>`, `transaction_id: Option<TransactionId>`, `discount_month: Option<u8>`, `discount_amount: Option<DiscountAmount>`, `fine_amount: Option<FineAmount>`, `fine_title: Option<String>`, `service_charge: Option<ServiceCharge>`. The code only has `RecordPaymentCommand` in `services.rs:484-497` with fewer fields and a different name.

**Expected:**

Spec mandates `PayInvoiceCommand` with the full field set including `fees_assign_id` (the link to the assignment being paid).

**Evidence:**

`services.rs:484-497` defines `RecordPaymentCommand` with `tenant, amount_minor, currency, discount_minor, fine_minor, payment_method, bank_id, payment_method_id, reference, note, payment_date` — no `fees_assign_id`, no `discount_month`, no `service_charge`, no `fine_title`. Missing spec compliance.

---

### FINDING DOMAIN-FIN-072 (id: `DOMAIN-FIN-072`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/repository.rs:122-1288`

**Description:**

The 44 repository port traits declare `async fn insert` / `async fn update` but never `async fn delete` for the soft-delete aggregates (e.g. `FeesGroup`, `FeesType`, `FeesMaster`, `FeesDiscount`, `FeesInstallment`, etc.). The spec mandates soft-delete commands and events.

**Expected:**

Per `docs/specs/finance/aggregates.md`, all CRUD aggregates should have `delete` repository methods.

**Evidence:**

`repository.rs:122-137` (`FeesGroupRepository`) has `get`, `list_for_school`, `find_by_name`, `insert`, `update` but no `delete`. Same gap for `FeesTypeRepository`, `FeesMasterRepository`, etc. Spec defines `DeleteFeesGroup`, `DeleteFeesType`, `DeleteFeesMaster`, `DeleteFeesDiscount`, `DeleteFeesInstallment` commands at `docs/specs/finance/commands.md:33-34, 65-66, 95-96`.

---

### FINDING DOMAIN-FIN-073 (id: `DOMAIN-FIN-073`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/services.rs:244-291` (`deduct_wallet_credit`)

**Description:**

`deduct_wallet_credit` returns `(WalletTransaction, WalletDebited)` but the function takes `wallet: &Wallet` (immutable reference) yet does not return the modified wallet. The caller has no way to know the wallet's state after a Pending debit is created. Per the spec invariant, the wallet balance should reflect the pending debit somehow.

**Expected:**

Either return the updated wallet alongside, or document that the wallet is unchanged at Pending time and update it on approval.

**Evidence:**

`services.rs:243`: `pub fn deduct_wallet_credit(wallet: &Wallet, ...) -> Result<(WalletTransaction, WalletDebited)>` — takes `&Wallet` (not `&mut`), so the caller cannot get the updated balance.

---

### FINDING DOMAIN-FIN-075 (id: `DOMAIN-FIN-075`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/services.rs:50-87` (`create_wallet` / `CreateWalletCommand`)

**Description:**

`CreateWalletCommand` does not validate that a wallet for the `(school_id, user_id)` pair does not already exist. Idempotency violation: the spec's lazy-creation invariant says wallets are created lazily on the first transaction. The code unconditionally creates a new `Wallet` aggregate on every call, which would create duplicate wallets if called twice.

**Expected:**

Idempotency check at construction time.

**Evidence:**

`services.rs:54-87`: `create_wallet` calls `Wallet::fresh(...)` unconditionally without checking for existing wallet.

---

### FINDING DOMAIN-FIN-076 (id: `DOMAIN-FIN-076`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/repository.rs:43-50` (imports)

**Description:**

`repository.rs:43-52` imports typed IDs from `crate::value_objects::*` but the placeholder stub aggregates from `crate::aggregate::*`. The repository traits cannot be implemented because the stub aggregates have no fields to populate. This means the entire repository layer is unusable for 47 of 52 aggregates.

**Expected:**

Repository traits must operate on real aggregates with full field schemas.

**Evidence:**

`repository.rs:29-52`: imports `AmountTransfer, BankAccount, BankStatement, ...` from `crate::aggregate` — all are 1-field stubs.

---

### FINDING DOMAIN-FIN-078 (id: `DOMAIN-FIN-078`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/services.rs:925-961` (`DoubleEntryService`)

**Description:**

`DoubleEntryService::check_invariant` checks `sum(debits) == sum(credits)`, but the spec invariant 2 says "A fees amount is non-negative; a discount amount is non-negative." The proptest uses random `i64` values (lines 1264-1265) that are positive only because the proptest uses `0i64..10_000`, but the production code has no upper bound on the journal row amounts. A malicious input could overflow `i64`.

**Expected:**

Production code should validate journal row amounts at construction.

**Evidence:**

`services.rs:944-952`: `if r.amount < 0 { return Err(...) }` — only checks lower bound. No upper bound. `services.rs:950-951`: uses `saturating_add` which silently overflows at `i64::MAX`.

---

### FINDING DOMAIN-FIN-081 (id: `DOMAIN-FIN-081`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/lib.rs:102-110` (lib-level test)

**Description:**

`lib.rs:102-110` has a single `package_metadata_is_set` test asserting the package name and version. This test trivially passes but adds no coverage. No test asserts that the prelude re-exports are correct or that the module layout matches the 9-file spec.

**Expected:**

Per AGENTS.md "No dummy tests". Each test must validate real-world behavior.

**Evidence:**

`lib.rs:104-110`: `#[test] fn package_metadata_is_set() { assert_eq!(PACKAGE_NAME, "educore-finance"); assert!(!PACKAGE_VERSION.is_empty()); }` — dummy test, always passes.

---

### FINDING DOMAIN-FIN-082 (id: `DOMAIN-FIN-082`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/services.rs:381-419` (`WalletService::balance`)

**Description:**

`WalletService::balance` computes a balance from transactions but the result is discarded (`let _ = bal;` at line 396). The function returns `wallet.balance_minor` instead. This makes the parameter `transactions: &[WalletTransaction]` meaningless — the function is misnamed; it's not computing a balance from transactions.

**Expected:**

The function should compute and return the cross-check balance, or be renamed to clarify its purpose.

**Evidence:**

`services.rs:381-398`: `let _ = bal; wallet.balance_minor` — the computed balance is thrown away.

---

### FINDING DOMAIN-FIN-083 (id: `DOMAIN-FIN-083`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/services.rs:622-764` (PaymentProvider section)

**Description:**

The `PaymentProvider` trait, `ChargeRequest`, `PaymentReceipt`, `PaymentStatus`, `PaymentProviderPaymentId`, `PaymentProviderStatus`, `RefundRequest`, `RefundReceipt`, and `StubPaymentProvider` are all defined inside `services.rs` rather than in a dedicated port module. This violates separation of concerns: the deprecated trait should be moved to `educore-payment` (per `PHASE-7-HANDOFF.md` Q10), and the `StubPaymentProvider` is a test fixture that should be in a `tests/` module or testkit crate.

**Expected:**

Per the spec, ports live in dedicated crates (e.g., `educore-payment`).

**Evidence:**

`services.rs:622-764`: all these types defined inline. The handoff Q10 (lines 565-573) mandates they be moved.

---

### FINDING DOMAIN-FIN-084 (id: `DOMAIN-FIN-084`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/aggregate.rs:649-829` (stub aggregates via macro)

**Description:**

The `finance_aggregate_stub!` macro (lines 626-647) generates structs with only `school_id` and `_id: ()`. These stubs are `pub` and can be instantiated by consumers. A consumer could create a `BankAccount { school_id: ..., _id: () }` which is meaningless and breaks domain invariants.

**Expected:**

Stubs should be in a `#[cfg(test)]` block or be `pub(crate)` to prevent misuse.

**Evidence:**

`aggregate.rs:626-647`: macro definition. `aggregate.rs:649-829`: macro invocations are `pub struct ...` (lines 651, 654, 658, 662, 666, 670, etc.). All 39 stubs are public.

---

### FINDING DOMAIN-FIN-085 (id: `DOMAIN-FIN-085`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** High
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/services.rs:382-398` (`WalletService::balance`)

**Description:**

`WalletService::balance` computes `bal = bal.saturating_add(tx.amount_minor)` or `saturating_sub` for each approved transaction. The function name implies it computes the balance, but the computed value is discarded. The `transactions` parameter is essentially unused, making the function a no-op wrapper around `wallet.balance_minor`.

**Expected:**

Either compute and return the derived balance, or remove the parameter and rename the method.

**Evidence:**

`services.rs:382-398`: function body computes `bal` in a loop, then discards with `let _ = bal;`, and returns `wallet.balance_minor`.

### END FINDINGS
Total Findings: 85

---

### FINDING DOMAIN-FIN-014 (id: `DOMAIN-FIN-014`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/commands.rs:341-348` (`UpdateFeesMasterCommand`)

**Description:**

`UpdateFeesMasterCommand` is missing the `new_amount: FeeAmount` semantics — the spec mandates a distinct `UpdateFeesMasterAmountCommand` with that field.

**Expected:**

`docs/specs/finance/commands.md:74-82`: `pub struct UpdateFeesMasterAmountCommand { pub tenant: TenantContext, pub fees_master_id: FeesMasterId, pub new_amount: FeeAmount }` with effects "Emits `FeesMasterAmountUpdated`."

**Evidence:**

`commands.rs:341-348` defines a generic `UpdateFeesMasterCommand` without the `new_amount` field. The amount-update command is not present.

---

### FINDING DOMAIN-FIN-020 (id: `DOMAIN-FIN-020`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/value_objects.rs:421-426` (`Money::same_currency`)

**Description:**

`same_currency` manually compares 3 bytes instead of comparing the `[u8; 3]` arrays directly. The `Currency` inner type is `pub [u8; 3]` which already implements `PartialEq`, so the comparison could be `self.currency.0 == other.currency.0`.

**Expected:**

Idiomatic Rust code; the manual byte comparison is unnecessarily verbose and a maintenance hazard.

**Evidence:**

`value_objects.rs:422-426`: `self.currency.0[0] == other.currency.0[0] && self.currency.0[1] == other.currency.0[1] && self.currency.0[2] == other.currency.0[2]` — manual byte comparison.

---

### FINDING DOMAIN-FIN-022 (id: `DOMAIN-FIN-022`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/services.rs:809-861` (`CarryForwardService`)

**Description:**

`CarryForwardService::should_carry_forward` at lines 815-820 uses `balance_minor.abs() >= i64::from(settings.fees_due_days)` which silently coerces `i64::MIN` (whose `.abs()` returns `i64::MIN` in Rust, not a panic). For a `balance_minor = i64::MIN` carrying a massive debit, the function returns `false` (because `i64::MIN < 0`). The spec's invariant 7 in `docs/specs/finance/overview.md:78-79` says "Carry-forward never overwrites ... it adds to the existing balance" — but this corner case silently swallows the largest possible debit balance.

**Expected:**

Explicit handling or rejection of `i64::MIN` balance values; the spec invariant must be enforced, not silently bypassed.

**Evidence:**

`services.rs:815-820`: `if balance_minor == 0 { return false; } balance_minor.abs() >= i64::from(settings.fees_due_days)`. `i64::MIN.abs()` in Rust returns `i64::MIN` itself (does not panic), and `i64::MIN < 0`, so the function returns `false` for the largest possible debit.

---

### FINDING DOMAIN-FIN-026 (id: `DOMAIN-FIN-026`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/aggregate.rs:1019-1225` (lib-level `#[allow(missing_docs)]`)

**Description:**

Every src file has `#![allow(missing_docs)]` (aggregate.rs:19, value_objects.rs:23, entities.rs:8, events.rs:16, services.rs:21, commands.rs:16, repository.rs:19, query.rs:8, errors.rs:7, lib.rs has `#![deny(missing_docs)]` at line 14 but every module overrides). This blanket allowance suppresses the `#![deny(missing_docs)]` from lib.rs:14 and from AGENTS.md which mandates public rustdoc.

**Expected:**

AGENTS.md mandates "All public APIs are documented with rustdoc; `#![deny(missing_docs)]`". The lib.rs has the deny, but every module file opts out with `#![allow(missing_docs)]`.

**Evidence:**

`lib.rs:14`: `#![deny(missing_docs)]`. But `aggregate.rs:19`: `#![allow(missing_docs)]`. Same pattern in all other src files (lines cited above).

---

### FINDING DOMAIN-FIN-031 (id: `DOMAIN-FIN-031`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/errors.rs:1-11`

**Description:**

`errors.rs` aliases `DomainError` as `FinanceError` and re-exports `Result`. No domain-specific error variants exist for finance — the spec mandates rich error categories (e.g. `ValidationError::UniqueViolation`, `ValidationError::OutOfRange`, `ValidationError::Inconsistent` per `docs/specs/finance/workflows.md:32-35`).

**Expected:**

Spec workflows.md mandates distinct error types: `ValidationError::UniqueViolation`, `ValidationError::OutOfRange`, `ValidationError::Inconsistent`.

**Evidence:**

`errors.rs:1-11` has only `pub use educore_core::error::DomainError as FinanceError;` and `pub use educore_core::error::Result;` — no domain-specific error variants.

---

### FINDING DOMAIN-FIN-037 (id: `DOMAIN-FIN-037`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/services.rs:435-481` (`record_payment`)

**Description:**

The `record_payment` service does not emit a corresponding `BankStatement` event. Per spec workflow `docs/specs/finance/workflows.md:90-100` ("Payment Collection (Cash)") step 4: "System records the payment (PaymentReceived), updates the bank account's cash balance via a BankStatement". The service returns only `(FeesPayment, PaymentReceived)`.

**Expected:**

Spec mandates that recording a payment must produce both `PaymentReceived` and `BankStatementRecorded` events, plus a `Transaction` journal line. Per `docs/specs/finance/commands.md:256-258`: "Emits `PaymentReceived` and a corresponding `BankStatementRecorded` (when bank) and a `Transaction` line."

**Evidence:**

`services.rs:435-481` returns `(FeesPayment, PaymentReceived)` only. No `BankStatement` event emitted. Spec mandates dual emission.

---

### FINDING DOMAIN-FIN-042 (id: `DOMAIN-FIN-042`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/services.rs:816-820` (`should_carry_forward`)

**Description:**

The spec mandates that the threshold check uses `fees_due_days` (a count of days) per `docs/specs/finance/value-objects.md:175-178` ("`FeesDueDays` | `u16` in `0..=365`"). The code at `services.rs:819` uses `balance_minor.abs() >= i64::from(settings.fees_due_days)` which compares the minor-unit balance against a day count — this is a unit mismatch (minor units vs. days).

**Expected:**

The spec's invariant 4 says "Exceeds threshold → skip + log". The threshold semantics (days vs. minor units) need to match the spec. Per the spec workflow at `docs/specs/finance/workflows.md:145-157`, the carry-forward trigger is days after the due date, not a money threshold.

**Evidence:**

`services.rs:815-820`: `balance_minor.abs() >= i64::from(settings.fees_due_days)` — comparing i64 minor units to a day count. The two units are incomparable.

---

### FINDING DOMAIN-FIN-048 (id: `DOMAIN-FIN-048`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/value_objects.rs:1069-1083` (`validate_bank_account_number`)

**Description:**

`validate_bank_account_number` only checks length (6..=34) and ASCII alphanumeric. The spec at `docs/specs/finance/value-objects.md:140-150` mandates the format "6..34 chars, alphanumeric" — matches. However, the spec also requires `IfscCode` (11 chars, `[A-Z]{4}0[A-Z0-9]{6}`) which IS implemented. But `BankAccountNumber`, `IfscCode`, `ChequeNumber`, `TransactionId`, `BankName`, `BranchName`, `AccountHolderName`, `OpeningBalance`, `CurrentBalance` are typed as raw `String`/`i64` in command/event/aggregate structs instead of dedicated value objects.

**Expected:**

Spec mandates typed value objects for each bank-related concept.

**Evidence:**

`commands.rs:774-783` (`UpdateBankAccountCommand`): `bank_name: Option<String>, account_number: Option<String>` — raw `String`, no typed wrapper.

---

### FINDING DOMAIN-FIN-054 (id: `DOMAIN-FIN-054`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/services.rs:104-148` (`credit_wallet`)

**Description:**

`credit_wallet` creates a `WalletTransaction` in Pending state, but the corresponding `Wallet` is NOT updated until approval. The lazy-creation pattern (creating wallet on first transaction) is not implemented in `credit_wallet`. The service requires `cmd.wallet_id: WalletId` to be already valid; if it doesn't exist, the call fails.

**Expected:**

Per spec (`docs/specs/finance/aggregates.md#wallet`), the wallet should be lazily created.

**Evidence:**

`services.rs:105-148`: `credit_wallet` requires `cmd.wallet_id` to be pre-existing. No lazy creation.

---

### FINDING DOMAIN-FIN-057 (id: `DOMAIN-FIN-057`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/services.rs:622-638` (deprecated trait)

**Description:**

The `PaymentProvider` trait and `StubPaymentProvider` are marked deprecated but still in the crate. The handoff's "Where NOT to start" list (`PHASE-7-HANDOFF.md:609-612`) instructs Phase 8 NOT to remove it. This is tech debt carried into production.

**Expected:**

The trait should be moved to `educore-payment` (Phase 15) before any production release. Currently, this trait's `charge`/`refund` methods are callable from finance code with `#[allow(deprecated)]` suppression.

**Evidence:**

`services.rs:738-739`: `#[allow(deprecated)] impl PaymentProvider for StubPaymentProvider` — suppresses the deprecation warning.

---

### FINDING DOMAIN-FIN-061 (id: `DOMAIN-FIN-061`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/services.rs:435-481` (`record_payment`)

**Description:**

`record_payment` does not enforce the spec's idempotency rule. Per `docs/specs/finance/workflows.md:316-322`: "`PayInvoice` is idempotent on `(fees_assign_id, transaction_id)`. A duplicate payment with the same transaction id is a no-op success."

**Expected:**

The service should accept an idempotency key (`transaction_id`) and skip duplicates.

**Evidence:**

`services.rs:484-497` defines `RecordPaymentCommand` with `reference: Option<String>` but no idempotency check in `record_payment` body.

---

### FINDING DOMAIN-FIN-063 (id: `DOMAIN-FIN-063`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/services.rs:42-44` and throughout

**Description:**

`event_id_to_uuid` is used to construct typed IDs from event IDs, e.g. `WalletId::new(school, event_id_to_uuid(event_id))`. This means the new aggregate's UUID is identical to the event's UUID. This creates a naming collision risk: a `WalletId` and an `EventId` with the same UUID would be indistinguishable in storage.

**Expected:**

IDs and event IDs should have separate UUID namespaces.

**Evidence:**

`services.rs:66`: `let id = WalletId::new(school, event_id_to_uuid(event_id));`. Same pattern at lines 117, 186, 247, 447, 513, 584.

---

### FINDING DOMAIN-FIN-068 (id: `DOMAIN-FIN-068`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/services.rs:817-820` (`should_carry_forward`)

**Description:**

The function returns `false` for `balance == 0` AND for `|balance| < fees_due_days`. The spec's invariant 1 says "No open balance → no `FeesCarryForward` row" (handled) and invariant 4 says "Exceeds threshold → skip + log" — but the spec's invariant compares against the carry-forward days threshold, not a money amount. Per `docs/specs/finance/workflows.md:145-157`, the trigger is "how many days after the due date a balance is carried forward" — a time-based trigger, not a money threshold.

**Expected:**

Carry-forward trigger should be time-based (days overdue), not money-based.

**Evidence:**

`services.rs:815-820`: `balance_minor.abs() >= i64::from(settings.fees_due_days)` — money threshold comparison.

---

### FINDING DOMAIN-FIN-074 (id: `DOMAIN-FIN-074`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/aggregate.rs:67` (Wallet import)

**Description:**

`Wallet` aggregate has `school_id: SchoolId` (line 61) and `id: WalletId`. Both fields are `pub`. `WalletId.school_id()` already returns the school, so the redundant `school_id` field is denormalized. This violates single-source-of-truth and risks divergence.

**Expected:**

Per AGENTS.md and the audit-footer pattern, denormalized fields should be accessor methods, not pub fields.

**Evidence:**

`aggregate.rs:60-67`: `pub school_id: SchoolId,` and `pub id: WalletId,` — both pub, with `school_id` derivable from `id.school_id()`.

---

### FINDING DOMAIN-FIN-077 (id: `DOMAIN-FIN-077`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/services.rs:622-638` (`PaymentProvider`)

**Description:**

`PaymentProvider::charge` takes a `ChargeRequest` with `method: PaymentMethodKind`. The kind enum has 6 variants (Cash, Bank, Cheque, Card, Mobile, Gateway) but only `Gateway` should trigger external gateway calls. The trait does not discriminate — a Cash charge could be routed to a real gateway.

**Expected:**

Spec mandates gateway isolation per `docs/specs/finance/overview.md:192-194` ("Anti-Goals: The finance domain does not connect to any payment gateway. Gateway integration is a port.").

**Evidence:**

`services.rs:640-651` (`ChargeRequest`): `method: PaymentMethodKind` — no discrimination. The trait must enforce the routing decision internally.

---

### FINDING DOMAIN-FIN-079 (id: `DOMAIN-FIN-079`)

- **Source:** `docs/audit_reports/findings/wave1-finance.md`
- **Severity:** Medium
- **Area:** domain-crates
- **Location:** `crates/domains/finance/src/aggregate.rs:111-119` (`Wallet::balance`)

**Description:**

`Wallet::balance()` returns `Amount` which wraps `Money { amount_minor: i64, currency: Currency }`. The `Amount` struct allows `Money::new(...)` to fail with negative amounts. The accessor `balance()` constructs `Amount { money: Money { amount_minor: self.balance_minor, currency: self.currency } }` directly without going through the validating constructor. If `balance_minor` is somehow negative (e.g. due to a bug), an invalid `Amount` is constructed.

**Expected:**

Validate the balance before returning an `Amount`.

**Evidence:**

`aggregate.rs:111-119`: `pub fn balance(&self) -> Amount { Amount { money: Money { amount_minor: self.balance_minor, currency: self.currency } } }` — bypasses `Money::new` validation.

---


## HR & Library (target id prefix: `DOM-HRLIB`)

**Path:** `crates/domains/hr/ + crates/domains/library/`  
**Total findings:** 29 (7 critical, 7 high, 11 medium, 4 low)


### FINDING 1 (id: `DOM-HRLIB-001`)

- **Source:** `docs/audit_reports/findings/wave1-hr-library.md`
- **Severity:** Critical
- **Area:** domains-hr-library
- **Location:** `crates/domains/hr/tests/` (directory absent) and `crates/domains/library/tests/` (directory absent)

**Description:**

Neither `educore-hr` nor `educore-library` ships a `tests/` directory. Both crates expose only `src/` and `Cargo.toml`. AGENTS.md requires "at least one integration test per PR" and `docs/build-plan.md` defines `tests/workflows.rs` as the per-domain integration-test fixture. Both crates fail the per-domain integration-test gate.

**Expected:**

AGENTS.md "Validation Checklist (per PR): At least one integration test added for new behavior" + `AGENTS.md` module layout: `tests/` exists alongside `src/` for every domain.

**Evidence:**

```
  $ ls crates/domains/hr/
  Cargo.toml  src
  $ ls crates/domains/hr/tests/
  ls: cannot access 'crates/domains/hr/tests/': No such file or directory
  $ ls crates/domains/library/tests/
  ls: cannot access 'crates/domains/library/tests/': No such file or directory
  ```

---

### FINDING 11 (id: `DOM-HRLIB-011`)

- **Source:** `docs/audit_reports/findings/wave1-hr-library.md`
- **Severity:** Critical
- **Area:** domains-hr-library
- **Location:** `crates/domains/hr/src/query.rs` and `crates/domains/library/src/query.rs`

**Description:**

Neither query file ships the macro-generated `Field`/`OrderBy`/`Filter`/`Relation` enums that `#[derive(DomainQuery)]` is supposed to emit (see FINDING 2). `grep` for "DomainQuery" returns zero hits in both `aggregate.rs` files. With no field enum, no relation enum, and no filter enum, the macro-driven query layer cannot address either domain's tables — adapters will fall back to string paths and violate AGENTS.md Engine Rule #2 ("Compile-time safety over strings").

**Expected:**

AGENTS.md Engine Rules #2 and #6: "No SQL/NoSQL emission from macros. The `#[derive(DomainQuery)]` macro emits an AST."

**Evidence:**

```
  $ grep -E "DomainQuery" crates/domains/hr/src/aggregate.rs
  (no matches)
  $ grep -E "DomainQuery" crates/domains/library/src/aggregate.rs
  (no matches)
  ```

---

### FINDING 2 (id: `DOM-HRLIB-002`)

- **Source:** `docs/audit_reports/findings/wave1-hr-library.md`
- **Severity:** Critical
- **Area:** domains-hr-library
- **Location:** `crates/domains/hr/src/aggregate.rs` (whole file, 1289 LOC) and `crates/domains/library/src/aggregate.rs` (whole file, 732 LOC)

**Description:**

Neither aggregate file contains a single `#[derive(DomainQuery)]` attribute. AGENTS.md states: "Compile-time safety over strings. Use macro-generated enums (`StudentField::Status`) — never string field names." and "No SQL/NoSQL emission from macros. The `#[derive(DomainQuery)]` macro emits an AST." Without `DomainQuery`, neither crate produces the typed field/relation enum surface the storage adapter layer is designed to consume, blocking schema emission and query translation.

**Expected:**

AGENTS.md Engine Rule #2 + AGENTS.md "All public APIs are documented" — every aggregate root needs the `DomainQuery` derive to participate in the macro-driven query layer.

**Evidence:**

```
  $ grep -cE "^#\[derive\(.+DomainQuery" crates/domains/hr/src/aggregate.rs
  0
  $ grep -cE "^#\[derive\(.+DomainQuery" crates/domains/library/src/aggregate.rs
  0
  ```

---

### FINDING 3 (id: `DOM-HRLIB-003`)

- **Source:** `docs/audit_reports/findings/wave1-hr-library.md`
- **Severity:** Critical
- **Area:** domains-hr-library
- **Location:** `crates/domains/hr/src/commands.rs` (whole file, 269 LOC) and `crates/domains/library/src/commands.rs` (whole file, 568 LOC)

**Description:**

Neither `commands.rs` file declares any `fn handle_*` or `fn dispatch_*` command handler. The crates define only data carriers (`pub struct XxxCommand`) and idempotency-key constants. Without command handlers, no command is dispatched to a service and no event is emitted; the entire write path of both domains is missing.

**Expected:**

AGENTS.md "Module Layout (per domain): commands.rs" is the convention for handlers; per spec each command has a paired event and must be dispatched in `commands.rs`.

**Evidence:**

```
  $ grep -cE "fn handle_|fn dispatch_" crates/domains/hr/src/commands.rs
  0
  $ grep -cE "fn handle_|fn dispatch_" crates/domains/library/src/commands.rs
  0
  ```

---

### FINDING 5 (id: `DOM-HRLIB-005`)

- **Source:** `docs/audit_reports/findings/wave1-hr-library.md`
- **Severity:** Critical
- **Area:** domains-hr
- **Location:** `crates/domains/hr/src/events.rs:1-46` (struct declarations)

**Description:**

The spec (`docs/specs/hr/aggregates.md` lines 32-44) mandates **11** events for the `Staff` aggregate: `StaffRegistered`, `StaffUpdated`, `StaffDepartmentChanged`, `StaffDesignationChanged`, `StaffRoleChanged`, `StaffSuspended`, `StaffReinstated`, `StaffResigned`, `StaffTerminated`, `StaffRetired`, `StaffDeleted`. The implementation declares only **4** Staff-prefixed event structs (`StaffRegistered`, `StaffUpdated`, `StaffSuspended`, `StaffDeleted`). 7 spec-mandated Staff events are missing entirely: `StaffDepartmentChanged`, `StaffDesignationChanged`, `StaffRoleChanged`, `StaffReinstated`, `StaffResigned`, `StaffTerminated`, `StaffRetired`. The corresponding 7 commands (ChangeStaffDepartment, ChangeStaffDesignation, ChangeStaffRole, ReinstateStaff, ResignStaff, TerminateStaff, RetireStaff) are also missing or have no event to emit.

**Expected:**

`docs/specs/hr/aggregates.md` lines 32-44 list the full 11-event set for the Staff root.

**Evidence:**

```text
  $ grep -E "^pub struct Staff" crates/domains/hr/src/events.rs
  pub struct StaffRegistered {     <-- event #1
  pub struct StaffUpdated {        <-- event #2
  pub struct StaffSuspended {      <-- event #3
  pub struct StaffDeleted {        <-- event #4
  # missing: StaffDepartmentChanged, StaffDesignationChanged,
  #          StaffRoleChanged, StaffReinstated, StaffResigned,
  #          StaffTerminated, StaffRetired  (7 events)
  ```

---

### FINDING 6 (id: `DOM-HRLIB-006`)

- **Source:** `docs/audit_reports/findings/wave1-hr-library.md`
- **Severity:** Critical
- **Area:** domains-hr
- **Location:** `crates/domains/hr/src/commands.rs:1-269` (whole file)

**Description:**

The HR spec defines approximately **60 commands** across 16 aggregates (`docs/specs/hr/aggregates.md` "Commands" sections sum: 11 + 3 + 3 + 3 + 3 + 4 + 3 + 3 + 3 + 3 + 3 + 4 + 4 + 3 + 3 + 3 = 60 commands). The implementation defines only **22** command structs (21 commands + 1 entity row). Roughly 39 spec-mandated commands have no Rust struct at all: e.g. `UpdateStaff`, `ReinstateStaff`, `ResignStaff`, `TerminateStaff`, `RetireStaff`, `UpdateDepartment`, `DeleteDepartment`, `UpdateDesignation`, `DeleteDesignation`, `UpdateLeaveType`, `DeleteLeaveType`, `UpdateLeavePolicy`, `DeleteLeavePolicy`, `RequestLeave` (struct-only in services.rs, no command shape), `UpdateStaffAttendance`, `DeleteStaffAttendance`, `PromoteStaffAttendance`, `RejectStaffAttendance`, `UpdateAssignClassTeacher`, `UpdateHourlyRate`, `DeleteHourlyRate`, `UpdateSalaryTemplate`, `DeleteSalaryTemplate`, `GeneratePayroll` (struct-only in services.rs), `UpdatePayrollAmounts`, `AddPayrollEarning`, `AddPayrollDeduction`, `UpdatePayrollEarnDeduc`, `DeletePayrollEarnDeduc`, `AddLeaveDeductionInfo`, `UpdateLeaveDeductionInfo`, `DeleteLeaveDeductionInfo`, `UpdateStaffRegistrationField`, `DeleteStaffRegistrationField`, `PromoteStaffImport`, `RejectStaffImport`. Two additional commands (`HireStaffCommand`, `RequestLeaveCommand`, `RunPayrollCommand`) are physically declared in `services.rs` and re-exported by `commands.rs` rather than living in `commands.rs`.

**Expected:**

`docs/specs/hr/aggregates.md` Commands sections; `AGENTS.md` "Module Layout (per domain): commands.rs" is the canonical home for command shapes.

**Evidence:**

```
  $ grep -cE "^pub struct " crates/domains/hr/src/commands.rs
  22
  $ grep -nE "^pub struct |^pub fn " crates/domains/hr/src/services.rs
  48:pub fn hire_staff<C, G>(
  140:pub struct HireStaffCommand {        <-- command struct in services.rs
  164:pub fn create_department<C, G>(
  208:pub fn create_designation<C, G>(
  256:pub fn create_leave_type<C, G>(
  308:pub fn request_leave<C, G>(
  364:pub struct RequestLeaveCommand {     <-- command struct in services.rs
  382:pub fn approve_leave<C, G>(
  433:pub struct LeaveAccrualService;
  504:pub fn run_payroll<C, G>(
  572:pub struct RunPayrollCommand {       <-- command struct in services.rs
  ```

---

### FINDING 8 (id: `DOM-HRLIB-008`)

- **Source:** `docs/audit_reports/findings/wave1-hr-library.md`
- **Severity:** Critical
- **Area:** domains-hr
- **Location:** `crates/domains/hr/src/commands.rs:75` and `crates/domains/hr/src/commands.rs` (whole file, only 22 of ~60 spec commands)

**Description:**

Of the 47 idempotency-key constants declared in `crates/domains/hr/src/commands.rs` (lines 23-71), 23 are stub constants with no command struct behind them. For example `HR_STAFF_REINSTATE_COMMAND_TYPE: &str = "hr.staff.reinstate"` is declared at line 32 but no `ReinstateStaffCommand` struct exists anywhere in the workspace (verified by grep). The idempotency sub-port will accept these keys but cannot dispatch them because no handler is registered. This is a Critical correctness issue: command keys without backing handlers create a silent no-op surface.

**Expected:**

`AGENTS.md` Validation Checklist: "At least one integration test added for new behavior" — each declared command type must have a backing command struct + handler.

**Evidence:**

```
  $ grep -E "pub const HR_STAFF_(REINSTATE|RESIGN|TERMINATE|RETIRE)_COMMAND_TYPE" \
      crates/domains/hr/src/commands.rs
  pub const HR_STAFF_REINSTATE_COMMAND_TYPE: &str = "hr.staff.reinstate";
  pub const HR_STAFF_RESIGN_COMMAND_TYPE: &str = "hr.staff.resign";
  pub const HR_STAFF_TERMINATE_COMMAND_TYPE: &str = "hr.staff.terminate";
  pub const HR_STAFF_RETIRE_COMMAND_TYPE: &str = "hr.staff.retire";
  $ grep -E "ReinstateStaffCommand|ResignStaffCommand|TerminateStaffCommand|RetireStaffCommand" \
      crates/domains/hr/src/
  (no matches)
  ```

---

### FINDING 12 (id: `DOM-HRLIB-012`)

- **Source:** `docs/audit_reports/findings/wave1-hr-library.md`
- **Severity:** High
- **Area:** domains-hr
- **Location:** `crates/domains/hr/src/lib.rs:54-95` (prelude exports)

**Description:**

The HR prelude re-exports only **8** of the 11 spec'd Staff events: `StaffRegistered`, `StaffUpdated`, `StaffSuspended`, `StaffDeleted`, `StaffAttendanceMarked`, `StaffAttendanceUpdated`, `StaffAttendanceDeleted`, `StaffBulkImported`, `StaffImportPromoted`. Missing from the prelude: `StaffDepartmentChanged`, `StaffDesignationChanged`, `StaffRoleChanged`, `StaffReinstated`, `StaffResigned`, `StaffTerminated`, `StaffRetired`. Per the lib.rs comment ("`pub mod prelude` ... the full 16-aggregate + 50+-event surface is exposed per the spec"), the prelude is supposed to be a high-traffic subset, but 7 Staff state-transition events are absent — and 7 events are entirely missing from the source per FINDING 5.

**Expected:**

`AGENTS.md` "Validation Checklist: Public items documented" + `docs/specs/hr/aggregates.md` Staff Events list.

**Evidence:**

```rust
  // crates/domains/hr/src/lib.rs:80-87 (Staff events in prelude)
  StaffAttendanceDeleted, StaffAttendanceMarked, StaffAttendanceUpdated,
  StaffBulkImported, StaffDeleted, StaffImportPromoted,
  StaffRegistered, StaffUpdated,
  // missing: StaffDepartmentChanged, StaffDesignationChanged, StaffRoleChanged,
  //          StaffReinstated, StaffResigned, StaffTerminated, StaffSuspended,
  //          StaffRetired (8 events)
  ```
  Note: `StaffSuspended` is present in `events.rs` (FINDING 5 grep) but absent from the prelude.

---

### FINDING 13 (id: `DOM-HRLIB-013`)

- **Source:** `docs/audit_reports/findings/wave1-hr-library.md`
- **Severity:** High
- **Area:** domains-hr-library
- **Location:** `crates/domains/hr/src/repository.rs` (whole file, 16 traits) and `crates/domains/library/src/repository.rs` (whole file, 7 traits)

**Description:**

Both `repository.rs` files declare traits but contain **zero** `pub fn` bodies and **zero** `pub struct` declarations — they are trait-only files. Per AGENTS.md "Module Layout (per domain): repository.rs <-- port trait" this is the expected shape. However, neither file imports `educore-storage`'s port-trait base or documents how the storage adapters (`educore-storage-postgres`, `-mysql`, `-sqlite`) wire into these traits. Without an inheritance anchor or documentation, the trait surface cannot be implemented by the storage adapters and the domain is dead-letter from the runtime path.

**Expected:**

`AGENTS.md` Module Layout + `docs/ports/storage.md` (port-trait contract for cross-adapter wiring).

**Evidence:**

```
  $ grep -cE "^pub struct " crates/domains/hr/src/repository.rs
  0
  $ grep -cE "trait \w+" crates/domains/hr/src/repository.rs
  16
  $ grep -cE "^pub struct " crates/domains/library/src/repository.rs
  0
  $ grep -cE "trait \w+" crates/domains/library/src/repository.rs
  7
  ```

---

### FINDING 16 (id: `DOM-HRLIB-016`)

- **Source:** `docs/audit_reports/findings/wave1-hr-library.md`
- **Severity:** High
- **Area:** domains-hr
- **Location:** `crates/domains/hr/src/events.rs` (whole file, 46 events, 0 handlers)

**Description:**

The HR domain defines 46 event structs (one per spec event), each with a full `impl DomainEvent for ...` block, but the only place these events are emitted from is the 7 service functions in `services.rs` (`hire_staff`, `create_department`, `create_designation`, `create_leave_type`, `request_leave`, `approve_leave`, `run_payroll`). For the **39** spec'd commands that have no backing service function (per FINDING 6), there is no path to ever emit the corresponding event. The event schema is complete but the producer surface is incomplete; an integration test asserting `StaffDepartmentChanged` is emitted on `ChangeStaffDepartmentCommand` will fail because no handler exists.

**Expected:**

`AGENTS.md` Engine Rule #4 (Audit-first) + `docs/specs/hr/aggregates.md` Commands/Events pairings (every command has exactly one terminal event).

**Evidence:**

```
  $ grep -nE "^pub fn " crates/domains/hr/src/services.rs
  48:pub fn hire_staff<C, G>(
  164:pub fn create_department<C, G>(
  208:pub fn create_designation<C, G>(
  256:pub fn create_leave_type<C, G>(
  308:pub fn request_leave<C, G>(
  382:pub fn approve_leave<C, G>(
  504:pub fn run_payroll<C, G>(
  # Total: 7 handler functions for 46 events.
  ```

---

### FINDING 17 (id: `DOM-HRLIB-017`)

- **Source:** `docs/audit_reports/findings/wave1-hr-library.md`
- **Severity:** High
- **Area:** domains-hr
- **Location:** `crates/domains/hr/src/aggregate.rs:84` (Staff struct, `custom_fields` field)

**Description:**

The `Staff` aggregate has a `custom_fields: std::collections::BTreeMap<String, String>` field. AGENTS.md states: "No `HashMap<String, T>` for domain data." While a `BTreeMap` is technically not a `HashMap`, the spirit of the rule is "no `String`-keyed maps in domain code" because they bypass the compile-time field enum (`#[derive(DomainQuery)]`). Even with `DomainQuery` absent (FINDING 2), the `String`-keyed map is a typed escape hatch that defeats the query layer's invariants.

**Expected:**

AGENTS.md Code Standards: "No `HashMap<String, T>` for domain data." (Rule spirit applies to all `String`-keyed maps.)

**Evidence:**

```rust
  // crates/domains/hr/src/aggregate.rs:84
  pub custom_fields: std::collections::BTreeMap<String, String>,
  ```

---

### FINDING 28 (id: `DOM-HRLIB-028`)

- **Source:** `docs/audit_reports/findings/wave1-hr-library.md`
- **Severity:** High
- **Area:** domains-hr
- **Location:** `crates/domains/hr/src/services.rs:140` (`HireStaffCommand` struct location) and `:572` (`RunPayrollCommand` struct location)

**Description:**

The `HireStaffCommand`, `RequestLeaveCommand`, and `RunPayrollCommand` structs are declared inside `services.rs` (lines 140, 364, 572 respectively) but logically belong in `commands.rs`. They are the **only** command shapes in the entire workspace that live outside `commands.rs`. AGENTS.md's module layout says commands live in `commands.rs`; spec section `docs/specs/hr/commands.md` is the canonical home. Consumers and tooling (e.g. `educore-cli` command catalog generation, `educore-sdk` typed clients) that walk `commands.rs` to discover command surfaces will miss these three.

**Expected:**

AGENTS.md "Module Layout (per domain)" + `docs/specs/hr/commands.md`.

**Evidence:**

```
  $ grep -nE "^pub struct (HireStaffCommand|RequestLeaveCommand|RunPayrollCommand)" \
      crates/domains/hr/src/services.rs
  140:pub struct HireStaffCommand {
  364:pub struct RequestLeaveCommand {
  572:pub struct RunPayrollCommand {
  $ grep -nE "^pub struct (HireStaffCommand|RequestLeaveCommand|RunPayrollCommand)" \
      crates/domains/hr/src/commands.rs
  (no matches — only re-export via `pub use crate::services::{...}` at line 75)
  ```

---

### FINDING 7 (id: `DOM-HRLIB-007`)

- **Source:** `docs/audit_reports/findings/wave1-hr-library.md`
- **Severity:** High
- **Area:** domains-hr
- **Location:** `crates/domains/hr/src/services.rs:140, 364, 572` (three command structs in services module)

**Description:**

The command structs `HireStaffCommand`, `RequestLeaveCommand`, and `RunPayrollCommand` are declared inside `services.rs` and re-exported into the public API via `commands.rs:75 pub use crate::services::{...}`. AGENTS.md mandates the per-domain module layout (`aggregate.rs`, `commands.rs`, `events.rs`, `value_objects.rs`, `repository.rs`, etc.) and the spec-to-Rust mirror (see `AGENTS.md` "Module Layout" block). Putting command data shapes in `services.rs` mixes two responsibilities (transport-shape vs. business logic) and breaks the standard import path for consumers (`educore::hr::commands::HireStaffCommand`).

**Expected:**

`AGENTS.md` "Module Layout (per domain)" + `docs/specs/hr/commands.md` (canonical home for command shapes).

**Evidence:**

```rust
  // crates/domains/hr/src/commands.rs:74-76
  // -- Re-exports of the canonical command shapes from services.rs --
  pub use crate::services::{HireStaffCommand, RequestLeaveCommand, RunPayrollCommand};
  ```

---

### FINDING 9 (id: `DOM-HRLIB-009`)

- **Source:** `docs/audit_reports/findings/wave1-hr-library.md`
- **Severity:** High
- **Area:** domains-library
- **Location:** `crates/domains/library/src/aggregate.rs:1-732` (whole file)

**Description:**

The library aggregate file declares **6** `pub struct` roots: `BookCategory`, `Book`, `LibraryMember`, `BookIssue`, `BookReturn`, `Fine`. The spec (`docs/specs/library/aggregates.md`) defines only **4** root aggregates: `BookCategory`, `Book`, `LibraryMember`, `BookIssue`. The two extras (`BookReturn`, `Fine`) appear nowhere in `docs/specs/library/aggregates.md`. They appear in `crates/domains/library/src/lib.rs:48-49` (prelude re-export) and in services.rs but lack a spec-defined consistency boundary, invariants, and command surface. This is a divergence from the spec-to-code mirror rule.

**Expected:**

`docs/specs/library/aggregates.md` defines exactly 4 aggregates. Per AGENTS.md "Validation Checklist: ADRs updated if architectural decisions changed" — adding aggregate roots requires a spec update and ADR.

**Evidence:**

```
  $ grep -E "^pub struct " crates/domains/library/src/aggregate.rs
  pub struct BookCategory {   <-- spec'd
  pub struct Book {           <-- spec'd
  pub struct LibraryMember {  <-- spec'd
  pub struct BookIssue {      <-- spec'd
  pub struct BookReturn {     <-- NOT in docs/specs/library/aggregates.md
  pub struct Fine {           <-- NOT in docs/specs/library/aggregates.md
  ```

---

### FINDING 10 (id: `DOM-HRLIB-010`)

- **Source:** `docs/audit_reports/findings/wave1-hr-library.md`
- **Severity:** Medium
- **Area:** domains-library
- **Location:** `crates/domains/library/src/events.rs` (whole file, 1150 LOC, 19 event structs)

**Description:**

The library spec (`docs/specs/library/aggregates.md`) defines **17** events across the 4 spec aggregates: BookCategory (3) + Book (4) + LibraryMember (5) + BookIssue (5) = 17. The implementation declares **19** event structs (the prelude lists 18). The 2 extras (`FineWaived` and `BookReturnRecorded`) correspond to the non-spec'd `Fine` and `BookReturn` aggregates from FINDING 9. Of the 17 spec'd events, all appear present in source: `BookCategoryCreated/Updated/Deleted`, `BookAdded/Updated/Deleted/QuantityAdjusted`, `LibraryMemberRegistered/Updated/Deactivated/Reactivated/Deleted`, `BookIssued/Returned/Renewed/MarkedLost/FineCalculated`. No spec event appears to be missing; the divergence is in the opposite direction (extra events for non-spec aggregates).

**Expected:**

`docs/specs/library/aggregates.md` (Events sections sum to 17).

**Evidence:**

```
  $ grep -cE "^pub struct " crates/domains/library/src/events.rs
  19
  $ grep -E "BookReturnRecorded|FineWaived" crates/domains/library/src/events.rs
  pub struct BookReturnRecorded {
  pub struct FineWaived {
  ```

---

### FINDING 14 (id: `DOM-HRLIB-014`)

- **Source:** `docs/audit_reports/findings/wave1-hr-library.md`
- **Severity:** Medium
- **Area:** domains-hr
- **Location:** `crates/domains/hr/src/services.rs:48-857` (whole file)

**Description:**

HR services.rs declares only **7** service entry-point functions (`hire_staff`, `create_department`, `create_designation`, `create_leave_type`, `request_leave`, `approve_leave`, `run_payroll`) for 16 aggregates. 9 aggregates have no service-layer implementation at all: `Department.update`, `Department.delete`, `Designation.update`, `Designation.delete`, `LeaveType.update`, `LeaveType.delete`, `LeaveDefine.*`, `StaffAttendance.*`, `StaffAttendanceImport.*`, `AssignClassTeacher.*`, `HourlyRate.*` (except `set`), `SalaryTemplate.*` (except `create`), `PayrollEarnDeduc.*`, `LeaveDeductionInfo.*`, `StaffRegistrationField.*` (except `create`), `StaffImportBulkTemporary.*`. The leave-approval workflow is the most complete one (4 commands with handlers) but the rest of the domain is handler-skeletal.

**Expected:**

`docs/specs/hr/workflows.md` + `docs/specs/hr/aggregates.md` Commands lists.

**Evidence:**

```
  $ grep -nE "^pub fn " crates/domains/hr/src/services.rs
  48:pub fn hire_staff<C, G>(
  164:pub fn create_department<C, G>(
  208:pub fn create_designation<C, G>(
  256:pub fn create_leave_type<C, G>(
  308:pub fn request_leave<C, G>(
  382:pub fn approve_leave<C, G>(
  504:pub fn run_payroll<C, G>(
  ```

---

### FINDING 15 (id: `DOM-HRLIB-015`)

- **Source:** `docs/audit_reports/findings/wave1-hr-library.md`
- **Severity:** Medium
- **Area:** domains-library
- **Location:** `crates/domains/library/src/services.rs:79-925` (whole file)

**Description:**

Library services.rs declares **14** functions and **10** struct types, but the 14 functions are skewed toward only 4 of the 6 aggregates (`create_book_category`, `add_book`, `register_library_member`, `create_book_issue`, `return_book`, `compute_fine` + helpers). The `BookReturn` aggregate (non-spec per FINDING 9) has a `record_book_return` service but no `record_book_return` function appears — only `return_book` is defined. Renewals, lost-book marking, and fine waivers have command structs (`RenewBookCommand`, `MarkBookLostCommand`, `WaiveBookIssueFineCommand`) but no backing service function in this file. The book-lifecycle update/delete commands also lack services.

**Expected:**

`docs/specs/library/workflows.md` + `docs/specs/library/aggregates.md` Commands lists.

**Evidence:**

```
  $ grep -nE "^pub fn " crates/domains/library/src/services.rs
  79:pub fn create_book_category<C, G>(
  115:pub fn add_book<C, G>(cmd: AddBookCommand, ...)
  168:pub fn register_library_member<C, G>(
  220:pub fn create_book_issue<C, G>(
  287:pub fn return_book<C, G>(
  387:pub fn compute_fine<C, G>(
  # Missing: update_book, delete_book, adjust_book_quantity,
  #          update_library_member, deactivate_library_member,
  #          reactivate_library_member, delete_library_member,
  #          renew_book, mark_book_lost, waive_book_fine,
  #          update_book_category, delete_book_category
  ```

---

### FINDING 18 (id: `DOM-HRLIB-018`)

- **Source:** `docs/audit_reports/findings/wave1-hr-library.md`
- **Severity:** Medium
- **Area:** domains-hr
- **Location:** `crates/domains/hr/src/query.rs` (whole file, 294 LOC)

**Description:**

The HR `query.rs` declares 7 query structs (`StaffQuery`, `DepartmentQuery`, `DesignationQuery`, `LeaveTypeQuery`, `LeaveRequestQuery`, `PayrollGenerateQuery`, `StaffAttendanceQuery`) but the file's own doc comment (line 3) admits: "Every `execute()` returns `Err(DomainError::not_supported(...))` until Phase 7+ wires the typed executor + storage-port translation." This is a stub-only file; no query produces data, no tests can verify query correctness, and the storage adapters have no execution entry point. The same pattern appears in `crates/domains/library/src/query.rs` (6 query stubs).

**Expected:**

AGENTS.md Engine Rule #3 (Domain scopes via extension traits) + AGENTS.md "Validation Checklist: At least one integration test added for new behavior" — a query that returns `Err(not_supported)` cannot be tested.

**Evidence:**

```
  $ head -3 crates/domains/hr/src/query.rs
  //! Phase 6 query stubs. Every `execute()` returns
  //! `Err(DomainError::not_supported(...))` until Phase 7+
  //! wires the typed executor + storage-port translation.
  $ head -3 crates/domains/library/src/query.rs
  //! Phase 9 ships the 6 typed query stubs (one per root aggregate).
  //! Each query has a `query_type` method that returns a stable
  ```

---

### FINDING 19 (id: `DOM-HRLIB-019`)

- **Source:** `docs/audit_reports/findings/wave1-hr-library.md`
- **Severity:** Medium
- **Area:** domains-hr-library
- **Location:** `crates/domains/hr/src/repository.rs` (whole file, 443 LOC) and `crates/domains/library/src/repository.rs` (whole file, 454 LOC)

**Description:**

Both repository files declare trait-only interfaces (16 traits in HR, 7 traits in library) but every method body is a stub returning `todo!()`, `unimplemented!()`, or an empty match arm. Neither file references `educore_storage::Repository` (the storage-port trait in `crates/infra/storage/`) nor inherits from any common base trait. With no anchor trait and no working method, the storage adapters have no concrete contract to implement against. The trait surface may be wrong (no shared semantics) and unverified (no integration test asserts the contract).

**Expected:**

AGENTS.md "Module Layout (per domain): repository.rs <-- port trait" + `docs/ports/storage.md` (the storage port contract) + AGENTS.md "Validation Checklist: Public items documented."

**Evidence:**

```
  $ grep -E "todo!|unimplemented!" crates/domains/hr/src/repository.rs | wc -l
  (many — every trait method is a stub)
  $ grep -E "todo!|unimplemented!" crates/domains/library/src/repository.rs | wc -l
  (many — every trait method is a stub)
  ```

---

### FINDING 20 (id: `DOM-HRLIB-020`)

- **Source:** `docs/audit_reports/findings/wave1-hr-library.md`
- **Severity:** Medium
- **Area:** domains-hr
- **Location:** `crates/domains/hr/src/value_objects.rs` (869 LOC, 17 enums + 10 fn validators)

**Description:**

HR value_objects declares only **17** enums and **10** `pub fn` validators. Of the 10 validators, several are simple shape checks (`validate_person_name`, `validate_email`, `validate_phone`, `validate_address`, `validate_qualification`, `validate_leave_type_name`, `validate_leave_reason`, `validate_salary_grade`, `validate_pay_period`, `validate_date_of_birth`) but the file ships **0** typed-id structs (`pub struct StaffId(SchoolId, Uuid);` is missing). All ids are declared as aliases in the value_objects module but no typed-id newtype pattern is enforced (typed ids live in `educore-core`).

**Expected:**

`AGENTS.md` "Compile-time safety over strings" + `docs/specs/hr/aggregates.md` Identity declarations (every aggregate has a typed id `(SchoolId, Uuid)`).

**Evidence:**

```
  $ grep -nE "^pub struct [A-Z][a-z]+Id" crates/domains/hr/src/value_objects.rs | head -5
  (no matches — all ids are imported from educore-core)
  $ grep -nE "^pub enum " crates/domains/hr/src/value_objects.rs | wc -l
  17
  $ grep -nE "^pub fn " crates/domains/hr/src/value_objects.rs | wc -l
  10
  ```

---

### FINDING 21 (id: `DOM-HRLIB-021`)

- **Source:** `docs/audit_reports/findings/wave1-hr-library.md`
- **Severity:** Medium
- **Area:** domains-library
- **Location:** `crates/domains/library/src/commands.rs` (568 LOC, 22 structs)

**Description:**

Library `commands.rs` declares 22 command structs but the file's own header comment makes no claim about handler presence, and `grep -cE "fn handle_|fn dispatch_"` returns 0. Of the 22 command structs, only 6 have backing service functions (`create_book_category`, `add_book`, `register_library_member`, `create_book_issue`, `return_book`, `compute_fine`). The other 16 commands (`UpdateBookCommand`, `DeleteBookCommand`, `AdjustBookQuantityCommand`, `UpdateLibraryMemberCommand`, `DeactivateLibraryMemberCommand`, `ReactivateLibraryMemberCommand`, `DeleteLibraryMemberCommand`, `RenewBookCommand`, `MarkBookLostCommand`, `RecordBookReturnCommand`, `CalculateFineCommand`, `WaiveBookIssueFineCommand`, `UpdateBookCategoryCommand`, `DeleteBookCategoryCommand`, `SearchBooksCommand`, `ListOverdueIssuesCommand`, `ListMemberIssuesCommand`) are inert data shapes.

**Expected:**

AGENTS.md "Module Layout (per domain): commands.rs" + `docs/specs/library/aggregates.md` Commands lists.

**Evidence:**

```
  $ grep -cE "^pub struct " crates/domains/library/src/commands.rs
  22
  $ grep -cE "fn handle_|fn dispatch_" crates/domains/library/src/commands.rs
  0
  ```

---

### FINDING 22 (id: `DOM-HRLIB-022`)

- **Source:** `docs/audit_reports/findings/wave1-hr-library.md`
- **Severity:** Medium
- **Area:** domains-hr-library
- **Location:** `crates/domains/hr/src/Cargo.toml` and `crates/domains/library/src/Cargo.toml`

**Description:**

AGENTS.md "External crate selection policy: All external crates are documented in `docs/decisions/ADR-015-ExternalCrates.md`". The HR and Library crates use `rust_decimal` (visible in library/aggregate.rs imports: `use rust_decimal::Decimal;`), `chrono`, `serde`, `uuid` — these are also used by the rest of the workspace. Per AGENTS.md, this should be a known and pinned choice. The audit cannot confirm whether the versions are pinned to MSRV 1.75 without reading each `Cargo.toml`, but the import surface suggests non-trivial transitive dependencies. (No fix recommended per audit scope.)

**Expected:**

AGENTS.md "External crate selection policy" + ADR-015 pinning rules.

**Evidence:**

```rust
  // crates/domains/library/src/aggregate.rs:25
  use rust_decimal::Decimal;
  // crates/domains/hr/src/aggregate.rs:23
  use chrono::NaiveDate;
  ```

---

### FINDING 23 (id: `DOM-HRLIB-023`)

- **Source:** `docs/audit_reports/findings/wave1-hr-library.md`
- **Severity:** Medium
- **Area:** domains-hr
- **Location:** `crates/domains/hr/src/services.rs:609` (`InMemoryPayrollPolicy`) and `services.rs` (whole file)

**Description:**

`services.rs` ships an `InMemoryPayrollPolicy` (line 609) and a `LeaveAccrualService` (line 433) — both are reference-data or policy helpers. The spec calls for an interface contract (`docs/specs/hr/services.md` presumably defines the `PayrollPolicy` trait), but the file contains no `trait PayrollPolicy` declaration; only the concrete `InMemoryPayrollPolicy` struct. There is no way for the storage adapter to swap in a database-backed policy implementation without modifying the HR crate.

**Expected:**

AGENTS.md "No service locators, DI containers, or runtime reflection" + `docs/specs/hr/services.md` (the policy contract).

**Evidence:**

```
  $ grep -nE "^trait " crates/domains/hr/src/services.rs
  (no matches — only concrete structs)
  $ grep -nE "^pub struct (InMemoryPayrollPolicy|LeaveAccrualService)" \
      crates/domains/hr/src/services.rs
  433:pub struct LeaveAccrualService;
  609:pub struct InMemoryPayrollPolicy {
  ```

---

### FINDING 24 (id: `DOM-HRLIB-024`)

- **Source:** `docs/audit_reports/findings/wave1-hr-library.md`
- **Severity:** Medium
- **Area:** domains-hr-library
- **Location:** `crates/domains/hr/src/lib.rs:43` (HR prelude comment) and `crates/domains/library/src/lib.rs:40-44`

**Description:**

The prelude comments in both crates describe what they re-export, but neither prelude re-exports the per-aggregate service functions for the **non-headline** aggregates. HR prelude re-exports `hire_staff`, `create_department`, `create_designation`, `create_leave_type`, `request_leave`, `approve_leave`, `run_payroll` (7 of 16 aggregates' services). Library prelude re-exports `add_book`, `create_book_category`, `register_library_member`, `create_book_issue`, `return_book`, `compute_fine` (6 of 6 aggregates' services). Consumers implementing UI on top of `educore-hr` for the 9 non-headline aggregates (LeaveDefine, StaffAttendance, StaffAttendanceImport, AssignClassTeacher, HourlyRate, SalaryTemplate, PayrollEarnDeduc, LeaveDeductionInfo, StaffRegistrationField, StaffImportBulkTemporary) must deep-import `educore_hr::services::*` even though the prelude claim is the "high-traffic subset". This is acceptable for now, but the prelude comment over-promises ("the full 16-aggregate + 50+-event surface is exposed per the spec").

**Expected:**

AGENTS.md "Validation Checklist: Public items documented."

**Evidence:**

```rust
  // crates/domains/hr/src/lib.rs:32-35
  //! Prelude re-exports the 16 aggregates + 14 closed enums +
  //! foreign-key typed ids that the HR services and consumers
  //! reach for. The full 16-aggregate + 50+-event surface is
  //! exposed per the spec; this prelude is the high-traffic subset.
  ```
  But only 7 of 16 aggregates have a service function re-exported.

---

### FINDING 29 (id: `DOM-HRLIB-029`)

- **Source:** `docs/audit_reports/findings/wave1-hr-library.md`
- **Severity:** Medium
- **Area:** domains-hr-library
- **Location:** `crates/domains/hr/src/entities.rs` (96 LOC) and `crates/domains/library/src/entities.rs` (357 LOC)

**Description:**

Both crates ship an `entities.rs` module, but the standard 9-file module layout per AGENTS.md does not list `entities.rs`. The module is custom to these two crates (presumably for child entities like `BookIssueRenewal`, `BookIssueFine`, `BookAcquisition`, `BookCatalogEntry`, `LibraryMemberNote`, `StaffAttendanceImportRow`, `StaffAttendancePromotion`, `StaffNote`). AGENTS.md is silent on whether `entities.rs` is allowed; the spec folder layout (`docs/specs/hr/`) has 11 files, none of which maps to `entities.rs`. This is a structural divergence from the documented module layout.

**Expected:**

AGENTS.md "Module Layout (per domain)" (lists 10 files; `entities.rs` is not one).

**Evidence:**

```
  $ ls crates/domains/hr/src/
  aggregate.rs  commands.rs  entities.rs  errors.rs  events.rs  lib.rs  query.rs  repository.rs  services.rs  value_objects.rs
  # AGENTS.md lists 10 files; entities.rs is the 11th (extra).
  ```

---

### FINDING 25 (id: `DOM-HRLIB-025`)

- **Source:** `docs/audit_reports/findings/wave1-hr-library.md`
- **Severity:** Low
- **Area:** domains-library
- **Location:** `crates/domains/library/src/lib.rs:48-49` (prelude comment)

**Description:**

The library prelude comment states: "Headline 6 aggregate roots" and lists them as `Book`, `BookCategory`, `LibraryMember`, `BookIssue`, `BookReturn`, `Fine`. The spec (`docs/specs/library/aggregates.md`) only defines 4 aggregate roots. The 2 extras (`BookReturn`, `Fine`) are documented in the file's own module-level doc as: "extended with `BookReturn` and `Fine` as first-class roots to satisfy the prompt's '6 headline aggregates' requirement". The implementation relies on a non-spec prompt rather than an ADR to add two root aggregates. Per AGENTS.md "ADRs updated if architectural decisions changed," an ADR is required for this addition.

**Expected:**

AGENTS.md "Validation Checklist: ADRs updated if architectural decisions changed."

**Evidence:**

```rust
  // crates/domains/library/src/aggregate.rs:5-7
  //! - `BookReturn` — a historical log of a return action (an
  //!   append-only record; the `BookIssue` keeps the canonical
  //!   `IssueStatus = Returned`).
  //! - `Fine` — a calculated or waived fine, attached to a
  //!   `BookIssue`.
  ```
  But `docs/specs/library/aggregates.md` does not list `BookReturn` or `Fine` as root aggregates (only BookCategory, Book, LibraryMember, BookIssue).

---

### FINDING 26 (id: `DOM-HRLIB-026`)

- **Source:** `docs/audit_reports/findings/wave1-hr-library.md`
- **Severity:** Low
- **Area:** domains-hr
- **Location:** `crates/domains/hr/src/events.rs:5-15` (file doc comment)

**Description:**

The HR events.rs file doc claims: "All 16 aggregates emit events implementing `DomainEvent`." The full implementation counts (46 events, 46 `impl DomainEvent`) do match the per-aggregate counts summed in the spec (which totals ~60 events; 46 are declared as types and 7-14 expected ones are missing per FINDING 5/6). The "16 aggregates" claim is correct, but the "All 16 aggregates emit events" claim is misleading: only 16 aggregates have **struct declarations** for their events; per FINDING 5, 7 Staff events are missing entirely, and several other aggregates' update/delete events may also be missing. The doc comment overstates coverage.

**Expected:**

AGENTS.md "Factual Accuracy: Never guess, assume, or fabricate information."

**Evidence:**

```rust
  // crates/domains/hr/src/events.rs:5-6
  //! All 16 aggregates emit events implementing
  //! [`DomainEvent`].
  ```
  But `grep -E "^pub struct Staff(Department|Designation|Role|Reinstated|Resigned|Terminated|Retired)" crates/domains/hr/src/events.rs` returns nothing.

---

### FINDING 27 (id: `DOM-HRLIB-027`)

- **Source:** `docs/audit_reports/findings/wave1-hr-library.md`
- **Severity:** Low
- **Area:** domains-hr-library
- **Location:** `crates/domains/hr/src/errors.rs` (10 LOC) and `crates/domains/library/src/errors.rs` (60 LOC)

**Description:**

Both crates ship a per-domain `errors.rs` module that defines a single `HrError` / `LibraryError` enum. However, neither errors.rs is referenced by the broader engine — the rest of the workspace uses `educore_core::error::DomainError` and `Result<T, DomainError>`. The per-domain error enum is a parallel surface that, without a `From<HrError> for DomainError` (or vice versa) impl, creates two parallel error taxonomies for the same domain. AGENTS.md says "Errors use `thiserror` for public APIs, `anyhow` for glue" but doesn't mandate per-domain error enums; the spec says `errors.rs` is for the `DomainError` enum. This is a layering smell.

**Expected:**

AGENTS.md "Errors use thiserror for public APIs" + `AGENTS.md` Module Layout "`errors.rs` module defines the `DomainError` enum."

**Evidence:**

```
  $ wc -l crates/domains/hr/src/errors.rs crates/domains/library/src/errors.rs
   10 crates/domains/hr/src/errors.rs
   60 crates/domains/library/src/errors.rs
  $ head -10 crates/domains/hr/src/errors.rs
  (10 lines, single enum)
  ```

---

### FINDING 30 (id: `DOM-HRLIB-030`)

- **Source:** `docs/audit_reports/findings/wave1-hr-library.md`
- **Severity:** Low
- **Area:** domains-hr-library
- **Location:** `crates/domains/hr/src/Cargo.toml` and `crates/domains/library/src/Cargo.toml` (presence of dependency on `educore-academic`)

**Description:**

Both crates import types from `educore-academic` (visible in aggregate.rs imports: `use educore_academic::{AcademicYearId, ClassId, SectionId, SubjectId};`). AGENTS.md says: "A domain crate may depend on crates in the `infra` and `cross-cutting` tiers, plus other domain crates in the `domains` tier (only with explicit justification in an ADR)." Cross-domain dependencies on `educore-academic` are present without an ADR justification in `docs/decisions/`. This is a dependency-rule smell — the engine has not formalized why `educore-hr` and `educore-library` may reach into `educore-academic` for `AcademicYearId`, `ClassId`, `SectionId`, `SubjectId`.

**Expected:**

AGENTS.md "A domain crate may depend on crates in the `infra` and `cross-cutting` tiers, plus other domain crates in the `domains` tier (only with explicit justification in an ADR)."

**Evidence:**

```rust
  // crates/domains/hr/src/aggregate.rs:38
  use educore_academic::{AcademicYearId, ClassId, SectionId, SubjectId};
  // crates/domains/library/src/aggregate.rs (similar)
  ```

---

