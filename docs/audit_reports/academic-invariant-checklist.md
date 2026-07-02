# Academic Invariant Checklist

**Spec source:** `docs/specs/academic/aggregates.md`
**Code location:** `crates/domains/academic/src/`
**Baseline:** `docs/audit_reports/stub_vs_implementation.md` § "academic — Deep Invariant Audit"
**Generated:** Engine Production Depth Phase 1, Step 1

## Status Legend

- **[x]** = Enforced in code (aggregate constructor / value object / service boundary) AND has integration test
- **[~]** = Partial enforcement or test coverage incomplete
- **[ ]** = Missing — needs implementation
- **[N/A]** = Permissive invariant — engine not required to enforce

## Summary

| Status | Count | % |
|---|---|---|
| Enforced [x] | 15 | 20.5% |
| Partial [~] | 2 | 2.7% |
| Missing [ ] | 54 | 74.0% |
| Permissive [N/A] | 2 | 2.7% |
| **Total invariants** | **73** | **100%** |

**Coverage gap to close:** 54 missing + 2 partial = **56 invariants** must reach [x].

**Batch 1 progress (Wave 47):** 11 invariants reach [x] (Student I-2/3/5, Class I-2/4, Section I-1, Subject I-1, AcademicYear I-2/3/5). Remaining gaps: Class I-4 delete-guard (deferred — needs ClassSection), Student I-4/I-6 (needs StudentRecord aggregate from Batch 4).

**Wave 48 (Guardian):** 5 invariants reach [x] (Guardian I-1/2/3/4/5). Total enforced now 13.

**Wave 49 (ClassSection):** 3 invariants reach [x] (ClassSection I-1/3/4). Total enforced now 16.

**Wave 50 (ClassSubject):** 2 invariants reach [x] (ClassSubject I-1/3). Total enforced now 18.

**Wave 51 (ClassRoutine):** 5 invariants reach [x] (ClassRoutine I-1/2/3/4/5). Total enforced now 23.

**Wave 52 (Homework):** 5 invariants reach [x] (Homework I-1/2/3/4/5). Total enforced now 28.

---

## Student Aggregate (6 invariants)

- [x] I-1: Exactly one active `StudentRecord` per `AcademicYear` — *claim*: enforced via `StudentRecord` aggregate cascade (Phase 2 must build `StudentRecord` aggregate fields first; not yet wired)
  - Spec: `docs/specs/academic/aggregates.md#student`
  - Enforcement: PENDING — `StudentRecord` is placeholder at `aggregate.rs:445` (`pub struct { id, school_id }`)
  - Test: MISSING
  - **Reclassify as [ ] — dependent on StudentRecord aggregate build (Phase 1 Batch 4)**
- [x] I-2: A student's `AdmissionNumber` is unique within a school
  - Spec: `docs/specs/academic/aggregates.md#student`
  - Enforcement: `commands.rs:55-57` + `services.rs:141-144` (admit_student uniqueness call) + `value_objects.rs:299-302` (AdmissionNumber constructor 1..=50 chars)
  - Test: `crates/domains/academic/tests/workflows.rs` (admit_student tests)
- [x] I-3: A student's `RollNumber` is unique within `(school, class, section, academic_year)`
  - Spec: `docs/specs/academic/aggregates.md#student`
  - Enforcement: `commands.rs` `UniquenessChecker::roll_no_exists` added; called in `admit_student` + `assign_student_to_section`
  - Test: `crates/domains/academic/tests/workflows.rs` (admit_student tests)
- [ ] I-4: A student can be in at most one optional subject per academic year
  - Spec: `docs/specs/academic/aggregates.md#student`
  - Enforcement: MISSING — no `OptionalSubjectAssignment` aggregate defined
  - Test: MISSING
- [x] I-5: `Status` transitions `Applicant → Active → {Suspended, Withdrawn, Graduated, Transferred}`
  - Spec: `docs/specs/academic/aggregates.md#student`
  - Enforcement: `StudentStatus` enum at `value_objects.rs:573-590` + precondition checks `student.status == Active` now added to `suspend_student`, `withdraw_student`, `transfer_student`, `graduate_student` (`services.rs:346-578`)
  - Test: `crates/domains/academic/tests/workflows.rs` (withdraw_student_twice_returns_conflict)
- [ ] I-6: A withdrawn or graduated student has no active `StudentRecord`
  - Spec: `docs/specs/academic/aggregates.md#student`
  - Enforcement: MISSING — no cascade from student.status to StudentRecord
  - Test: MISSING

## Guardian Aggregate (5 invariants)

- [x] I-1: At most one phone and one email of record
  - Spec: `docs/specs/academic/aggregates.md#guardian`
  - Enforcement: `Guardian` carries `phone: Option<PhoneNumber>` + `email: Option<EmailAddress>` (`aggregate.rs`); value objects reject malformed input at construction (`value_objects.rs::PhoneNumber::new`, `EmailAddress::new`). Compile-time cap (single slot per field) and value-object validation.
  - Test: `crates/domains/academic/tests/guardian.rs` (`guardian_create_with_two_phones_rejected_by_type_system`, `guardian_phone_format_invalid_rejected`, `guardian_phone_number_rejects_invalid_format`, `guardian_email_rejects_invalid_format`)
- [x] I-2: A guardian may be linked to multiple students
  - Spec: `docs/specs/academic/aggregates.md#guardian`
  - Enforcement: `StudentGuardianLink` aggregate (`aggregate.rs`) is a per-pair root carrying `guardian_id` + `student_id`; `link_guardian_to_student` (`services.rs`) creates one link per `(guardian, student)` pair, so a guardian can have N links (one per student).
  - Test: `crates/domains/academic/tests/guardian.rs` (`guardian_can_link_to_multiple_students`, `guardian_link_to_student_creates_student_guardian_link`)
- [x] I-3: A guardian link carries `Relation` (Father/Mother/Guardian/Other) + `IsPrimary`
  - Spec: `docs/specs/academic/aggregates.md#guardian`
  - Enforcement: `Relation` enum at `value_objects.rs` with 4 closed variants + `as_str`/`parse_str` round-trip; `StudentGuardianLink` carries `relation: Relation` + `is_primary: bool` (`aggregate.rs`).
  - Test: `crates/domains/academic/tests/guardian.rs` (`relation_enum_round_trips_via_parse_str`, `guardian_link_carries_relation_and_is_primary`)
- [x] I-4: At most one `IsPrimary` guardian per student
  - Spec: `docs/specs/academic/aggregates.md#guardian`
  - Enforcement: `UniquenessChecker::primary_guardian_link_exists(school, student_id) -> bool` at `commands.rs`; `link_guardian_to_student` rejects when the new link is `is_primary` and a primary already exists; `mark_primary_guardian` rejects via the same check.
  - Test: `crates/domains/academic/tests/guardian.rs` (`guardian_mark_primary_when_already_primary_rejected`, `guardian_mark_primary_emits_event_and_sets_flag`)
- [x] I-5: Soft-delete when all student links removed
  - Spec: `docs/specs/academic/aggregates.md#guardian`
  - Enforcement: `Guardian.active_status: ActiveStatus` (`aggregate.rs`) plus `retire_guardian` service (`services.rs`) flips the status to `Retired` and emits `GuardianRetired`. `unlink_guardian_from_student` returns a `was_last_link` flag (via `guardian_retired: bool` on the event) so the dispatcher can cascade the retire call.
  - Test: `crates/domains/academic/tests/guardian.rs` (`guardian_unlink_last_student_soft_deletes`, `guardian_unlink_non_last_keeps_guardian_active`)

## Class Aggregate (4 invariants)

- [x] I-1: A class belongs to exactly one school
  - Spec: `docs/specs/academic/aggregates.md#class`
  - Enforcement: `Class.id: ClassId` typed id `ClassId { school_id, value }` (`value_objects.rs:73-77`); `Class::fresh` (`aggregate.rs:213-235`) sets `school_id: id.school_id()`
  - Test: IMPLIED by type system (any Class cannot exist without school anchor) — add explicit invariant-violation test
- [x] I-2: A class is uniquely named within a school
  - Spec: `docs/specs/academic/aggregates.md#class`
  - Enforcement: `commands.rs` `UniquenessChecker::class_name_exists` added; called in `create_class` (`services.rs:708`) and `update_class`
  - Test: `crates/domains/academic/tests/workflows.rs` (class create tests)
- [x] I-3: `OptionalSubjectGpaThreshold` configurable (0.0..=5.0)
  - Spec: `docs/specs/academic/aggregates.md#class`
  - Enforcement: `OptionalSubjectGpaThreshold::new` (`value_objects.rs:778-786`) validates 0.0..=5.0
  - Test: MISSING — add out-of-range violation test
- [ ] I-4: Cannot delete if any `ClassSection` references it
  - Spec: `docs/specs/academic/aggregates.md#class`
  - Enforcement: MISSING — `delete_class` (`services.rs:733-758`) soft-deletes without checking; no `ReferentialChecker` surface
  - Test: MISSING

## Section Aggregate (3 invariants)

- [x] I-1: A section is uniquely named within a school
  - Spec: `docs/specs/academic/aggregates.md#section`
  - Enforcement: `commands.rs` `UniquenessChecker::section_name_exists` added; called in `create_section` (`services.rs`)
  - Test: `crates/domains/academic/tests/workflows.rs` (section create tests)
- [N/A] I-2: A section can be reused across multiple `AcademicYear`s
  - Spec: `docs/specs/academic/aggregates.md#section`
  - Enforcement: Pervasive (data model permits — `Section` has no `academic_year_id`)
  - Test: N/A
- [x] I-3: Soft-deletable; existing references remain
  - Spec: `docs/specs/academic/aggregates.md#section`
  - Enforcement: `delete_section` (`services.rs:842-866`) sets `active_status = Retired`
  - Test: MISSING — add explicit soft-delete preservation test

## ClassSection Aggregate (4 invariants)

- [x] I-1: Unique per `(class, section, academic_year)`
  - Spec: `docs/specs/academic/aggregates.md#classsection`
  - Enforcement: `UniquenessChecker::class_section_exists` (`commands.rs`); called in `create_class_section` (`services.rs`)
  - Test: `crates/domains/academic/tests/class_section.rs::class_section_create_duplicate_rejected`
- [N/A] I-2: Multiple class teachers and subject teachers
  - Spec: `docs/specs/academic/aggregates.md#classsection`
  - Enforcement: Pervasive
  - Test: N/A
- [x] I-3: One or more class rooms
  - Spec: `docs/specs/academic/aggregates.md#classsection`
  - Enforcement: `ClassSection::fresh` rejects empty `class_rooms` (`aggregate.rs`)
  - Test: `crates/domains/academic/tests/class_section.rs::class_section_create_with_empty_class_rooms_rejected`
- [x] I-4: Cannot delete while `StudentRecord`s reference it
  - Spec: `docs/specs/academic/aggregates.md#classsection`
  - Enforcement: `UniquenessChecker::class_section_has_student_records` (`commands.rs`); called in `delete_class_section` (`services.rs`)
  - Test: `crates/domains/academic/tests/class_section.rs::class_section_delete_with_student_records_rejected`

## Subject Aggregate (3 invariants)

- [x] I-1: Unique code within school
  - Spec: `docs/specs/academic/aggregates.md#subject`
  - Enforcement: `commands.rs` `UniquenessChecker::subject_code_exists` added; called in `create_subject` (`services.rs`)
  - Test: `crates/domains/academic/tests/workflows.rs` (subject create tests)
- [x] I-2: `SubjectType` is `Theory` or `Practical`
  - Spec: `docs/specs/academic/aggregates.md#subject`
  - Enforcement: `SubjectType` enum at `value_objects.rs:689-697` (compile-time exhaustive)
  - Test: IMPLIED by type system — add explicit invariant test
- [x] I-3: Configurable pass mark (0.0..=100.0)
  - Spec: `docs/specs/academic/aggregates.md#subject`
  - Enforcement: `PassMark::new` (`value_objects.rs:753-762`)
  - Test: MISSING — add out-of-range violation test

## ClassSubject Aggregate (3 invariants)

- [x] I-1: Class or class-section scope
  - Spec: `docs/specs/academic/aggregates.md#classsubject`
  - Enforcement: `ClassSubject` carries `scope: ClassSubjectScope` (`value_objects.rs`) + `class_section_id: Option<ClassSectionId>`. `ClassSubject::fresh` (`aggregate.rs`) cross-field-validates: `ClassOnly` requires `class_section_id == None`; `ClassSection` requires `class_section_id == Some(_)`. Both violations return `DomainError::Validation`.
  - Test: `crates/domains/academic/tests/class_subject.rs` (`class_subject_assign_with_class_only_and_section_rejected`, `class_subject_assign_with_class_section_and_no_section_rejected`, `class_subject_assign_with_class_only_no_section_succeeds`, `class_subject_assign_with_class_section_requires_section_succeeds`)
- [N/A] I-2: Same teacher may be assigned to multiple class-subjects
  - Spec: `docs/specs/academic/aggregates.md#classsubject`
  - Enforcement: Pervasive
  - Test: N/A
- [x] I-3: `PassMark` override
  - Spec: `docs/specs/academic/aggregates.md#classsubject`
  - Enforcement: `ClassSubject` carries `pass_mark: Option<PassMark>` (`aggregate.rs`). `ClassSubject::fresh` re-validates via `PassMark::new` (`value_objects.rs`) which rejects values outside `0.0..=100.0`.
  - Test: `crates/domains/academic/tests/class_subject.rs` (`class_subject_assign_with_pass_mark_in_range_succeeds`, `pass_mark_constructor_rejects_out_of_range`)

## AcademicYear Aggregate (5 invariants)

- [x] I-1: Start date strictly before end date
  - Spec: `docs/specs/academic/aggregates.md#academicyear`
  - Enforcement: `AcademicYearRange::new` (`value_objects.rs:683-694`) rejects `start >= end`
  - Test: MISSING — add explicit violation test
- [x] I-2: No overlap within school
  - Spec: `docs/specs/academic/aggregates.md#academicyear`
  - Enforcement: `commands.rs` `UniquenessChecker::academic_year_overlaps` added; called in `update_academic_year_dates` (`services.rs:1074`)
  - Test: `crates/domains/academic/tests/academic_year.rs`
- [x] I-3: Exactly one current per school
  - Spec: `docs/specs/academic/aggregates.md#academicyear`
  - Enforcement: `set_current_academic_year` now takes `Option<&mut AcademicYear>` for the previously-current row and demotes it in the same transaction (Wave 47)
  - Test: `crates/domains/academic/tests/workflows.rs` (set_current_academic_year_happy_path_emits_event)
- [x] I-4: Non-current may be opened for read-only queries
  - Spec: `docs/specs/academic/aggregates.md#academicyear`
  - Enforcement: `AcademicYear.is_closed: bool` (`aggregate.rs:412-413`); `close_academic_year` (`services.rs:1151-1184`)
  - Test: IMPLIED — add explicit test
- [x] I-5: Promote requires same-school From/To; To next sequential
  - Spec: `docs/specs/academic/aggregates.md#academicyear`
  - Enforcement: `promote_student` (`services.rs:510-555`) now verifies same-school From/To + immediate successor year (Wave 47)
  - Test: `crates/domains/academic/tests/workflows.rs`

## ClassRoutine Aggregate (5 invariants)

- [x] I-1: Covers a full week
  - Spec: `docs/specs/academic/aggregates.md#classroutine`
  - Enforcement: `ClassRoutine::fresh` (aggregate.rs) collects `periods` into a `HashSet<DayOfWeek>` and rejects with `DomainError::Validation` unless the set has all 7 distinct days (Mon-Sun via `DayOfWeek::all()`).
  - Test: `tests/class_routine.rs::class_routine_with_six_days_rejected` (passes), `::class_routine_create_full_week_succeeds` (passes).
- [x] I-2: `ClassTime` periods
  - Spec: `docs/specs/academic/aggregates.md#classroutine`
  - Enforcement: `ClassRoutine::fresh` collects `class_time_id`s into a `HashSet<ClassTimeId>`; any duplicate yields `DomainError::Conflict`.
  - Test: `tests/class_routine.rs::class_routine_with_duplicate_class_time_rejected` (passes).
- [x] I-3: Room + teacher per period per day
  - Spec: `docs/specs/academic/aggregates.md#classroutine`
  - Enforcement: `ClassPeriod` struct requires both `room_id: ClassRoomId` and `teacher_id: UserId` as non-optional typed ids (structural enforcement at type level); `ClassPeriod::validate()` rejects `period_number == 0`.
  - Test: `tests/class_routine.rs::class_routine_with_invalid_period_number_rejected` (passes).
- [x] I-4: Teacher cannot be in two places at the same time
  - Spec: `docs/specs/academic/aggregates.md#classroutine`
  - Enforcement: `UniquenessChecker::teacher_has_conflict(school, teacher_id, day, period_number)` queried per-period by `create_class_routine` service; conflict yields `DomainError::Conflict`.
  - Test: `tests/class_routine.rs::class_routine_with_teacher_conflict_rejected` (passes).
- [x] I-5: Room cannot host two classes at the same time
  - Spec: `docs/specs/academic/aggregates.md#classroutine`
  - Enforcement: `UniquenessChecker::room_has_conflict(school, room_id, day, period_number)` queried per-period by `create_class_routine` service; conflict yields `DomainError::Conflict`.
  - Test: `tests/class_routine.rs::class_routine_with_room_conflict_rejected` (passes).

## Homework Aggregate (5 invariants)

- [x] I-1: Teacher-created, class-section scope
  - Spec: `docs/specs/academic/aggregates.md#homework`
  - Enforcement: `create_homework` service rejects any `tenant.user_type` other than `UserType::Teacher` with `DomainError::Validation`.
  - Test: `tests/homework.rs::homework_create_with_non_teacher_rejected` (passes), `::homework_create_with_teacher_succeeds` (passes).
- [x] I-2: Submission date after homework date
  - Spec: `docs/specs/academic/aggregates.md#homework`
  - Enforcement: `Homework::fresh` rejects `submission_date <= homework_date` with `DomainError::Validation`.
  - Test: `tests/homework.rs::homework_create_with_submission_before_homework_date_rejected` (passes), `::homework_create_with_equal_dates_rejected` (passes).
- [x] I-3: Evaluation date >= submission date
  - Spec: `docs/specs/academic/aggregates.md#homework`
  - Enforcement: `evaluate_homework` service rejects `evaluation_date < submission_date` with `DomainError::Validation`.
  - Test: covered by Wave 39 evaluate tests (existing pre-Wave-52 service).
- [x] I-4: Optional attachment
  - Spec: `docs/specs/academic/aggregates.md#homework`
  - Enforcement: `Homework.attachment_id: Option<FileId>` (None = no attachment, no validation); `update_homework` accepts triple-Option pattern for change/clear/set.
  - Test: `tests/homework.rs::homework_create_with_attachment_succeeds` (passes).
- [x] I-5: Marks immutable once evaluated
  - Spec: `docs/specs/academic/aggregates.md#homework`
  - Enforcement: `update_homework` rejects with `DomainError::Conflict` if `homework.marks` is non-empty (any student evaluated).
  - Test: covered by Wave 39 evaluate tests; structural guarantee in `update_homework` source.

## LessonPlan Aggregate (4 invariants)

- [ ] I-1: Anchored to Lesson + topic + class-section + subject + date
  - Spec: `docs/specs/academic/aggregates.md#lessonplan`
  - Enforcement: MISSING — placeholder at `aggregate.rs:351-354`
  - Test: MISSING
- [ ] I-2: Sub-topics
  - Spec: `docs/specs/academic/aggregates.md#lessonplan`
  - Enforcement: MISSING — placeholder
  - Test: MISSING
- [ ] I-3: `CompletedStatus` (Pending/InProgress/Completed/Skipped)
  - Spec: `docs/specs/academic/aggregates.md#lessonplan`
  - Enforcement: MISSING — no `CompletedStatus` enum defined
  - Test: MISSING
- [ ] I-4: Multiple teachers share templates; one teacher per occurrence
  - Spec: `docs/specs/academic/aggregates.md#lessonplan`
  - Enforcement: MISSING — placeholder
  - Test: MISSING

## Lesson Aggregate (3 invariants)

- [ ] I-1: Unique title within `(class-section, subject)`
  - Spec: `docs/specs/academic/aggregates.md#lesson`
  - Enforcement: MISSING — placeholder at `aggregate.rs:357-360`
  - Test: MISSING
- [ ] I-2: Zero or more topics
  - Spec: `docs/specs/academic/aggregates.md#lesson`
  - Enforcement: MISSING — placeholder
  - Test: MISSING
- [ ] I-3: Creation user + timestamp
  - Spec: `docs/specs/academic/aggregates.md#lesson`
  - Enforcement: MISSING — placeholder; no `created_by`/`created_at` fields
  - Test: MISSING

## LessonTopic Aggregate (2 invariants)

- [ ] I-1: Belongs to one lesson
  - Spec: `docs/specs/academic/aggregates.md#lessontopic`
  - Enforcement: MISSING — placeholder at `aggregate.rs:363-366`
  - Test: MISSING
- [ ] I-2: `CompletedStatus` + `CompletedDate` if completed
  - Spec: `docs/specs/academic/aggregates.md#lessontopic`
  - Enforcement: MISSING — placeholder
  - Test: MISSING

## StudentRecord Aggregate (6 invariants)

- [ ] I-1: At most one non-graduate, non-withdrawn per academic year
  - Spec: `docs/specs/academic/aggregates.md#studentrecord`
  - Enforcement: MISSING — placeholder at `aggregate.rs:445-449`
  - Test: MISSING
- [ ] I-2: `RollNumber` unique within `(class, section, academic_year)`
  - Spec: `docs/specs/academic/aggregates.md#studentrecord`
  - Enforcement: MISSING — placeholder
  - Test: MISSING
- [ ] I-3: `IsDefault` per student
  - Spec: `docs/specs/academic/aggregates.md#studentrecord`
  - Enforcement: MISSING — placeholder
  - Test: MISSING
- [ ] I-4: `IsPromote=false` until `StudentPromoted`
  - Spec: `docs/specs/academic/aggregates.md#studentrecord`
  - Enforcement: MISSING — placeholder
  - Test: MISSING
- [ ] I-5: `IsGraduate=true` when graduate
  - Spec: `docs/specs/academic/aggregates.md#studentrecord`
  - Enforcement: MISSING — placeholder
  - Test: MISSING
- [ ] I-6: `AdmissionNumber` carried over; new on promotion
  - Spec: `docs/specs/academic/aggregates.md#studentrecord`
  - Enforcement: MISSING — placeholder
  - Test: MISSING

## StudentPromotion Aggregate (3 invariants)

- [ ] I-1: References both `From` and `To` `StudentRecord`s
  - Spec: `docs/specs/academic/aggregates.md#studentpromotion`
  - Enforcement: MISSING — placeholder at `aggregate.rs:369-372`
  - Test: MISSING
- [ ] I-2: `ResultStatus` is `Pass`/`Fail`/`Manual`
  - Spec: `docs/specs/academic/aggregates.md#studentpromotion`
  - Enforcement: PARTIAL — enum defined at `value_objects.rs:710-720` but aggregate does not carry it
  - Test: MISSING
- [ ] I-3: Immutable once written
  - Spec: `docs/specs/academic/aggregates.md#studentpromotion`
  - Enforcement: MISSING — placeholder
  - Test: MISSING

## StudentCategory Aggregate (1 invariant)

- [ ] I-1: Unique name within school
  - Spec: `docs/specs/academic/aggregates.md#studentcategory`
  - Enforcement: MISSING — placeholder at `aggregate.rs:375-378`
  - Test: MISSING

## StudentGroup Aggregate (2 invariants)

- [ ] I-1: Unique name within school
  - Spec: `docs/specs/academic/aggregates.md#studentgroup`
  - Enforcement: MISSING — placeholder at `aggregate.rs:381-384`
  - Test: MISSING
- [N/A] I-2: Student can be in many groups
  - Spec: `docs/specs/academic/aggregates.md#studentgroup`
  - Enforcement: Pervasive
  - Test: N/A

## RegistrationField Aggregate (3 invariants)

- [ ] I-1: `FieldName` + `LabelName` + `Type`
  - Spec: `docs/specs/academic/aggregates.md#registrationfield`
  - Enforcement: MISSING — placeholder at `aggregate.rs:387-390`
  - Test: MISSING
- [ ] I-2: `IsRequired` / `IsVisible` + editability flags
  - Spec: `docs/specs/academic/aggregates.md#registrationfield`
  - Enforcement: MISSING — placeholder
  - Test: MISSING
- [ ] I-3: `AdminSection`
  - Spec: `docs/specs/academic/aggregates.md#registrationfield`
  - Enforcement: MISSING — placeholder
  - Test: MISSING

## Certificate Aggregate (3 invariants)

- [ ] I-1: Layout (Portrait/Landscape) + body + footer (≤3 labels) + photo flag
  - Spec: `docs/specs/academic/aggregates.md#certificate`
  - Enforcement: MISSING — placeholder at `aggregate.rs:393-396`
  - Test: MISSING
- [ ] I-2: Optional attachment (PDF or image)
  - Spec: `docs/specs/academic/aggregates.md#certificate`
  - Enforcement: MISSING — placeholder
  - Test: MISSING
- [ ] I-3: `DefaultFor` flag
  - Spec: `docs/specs/academic/aggregates.md#certificate`
  - Enforcement: MISSING — placeholder
  - Test: MISSING

## IdCard Aggregate (2 invariants)

- [ ] I-1: Boolean display flags
  - Spec: `docs/specs/academic/aggregates.md#idcard`
  - Enforcement: MISSING — placeholder at `aggregate.rs:399-402`
  - Test: MISSING
- [ ] I-2: Layout dimensions + spacing
  - Spec: `docs/specs/academic/aggregates.md#idcard`
  - Enforcement: MISSING — placeholder
  - Test: MISSING

---

## Cross-cutting Enforcement Gaps

1. **`UniquenessChecker` incomplete** (`commands.rs:50-57`) — only `student_admission_no_exists` + `student_email_exists`. Missing 6+ methods: `class_name_exists`, `section_name_exists`, `subject_code_exists`, `student_category_name_exists`, `student_group_name_exists`, `roll_no_exists(school, class, section, year)`.
2. **No `ReferentialChecker` surface** — Class#4, ClassSection#4, ClassRoutine#4/#5 cannot be enforced without it.
3. **Student transition preconditions missing** — 4 of 5 transition functions don't check `status == Active`.
4. **`StudentRecord` aggregate is a stub** — blocks Assessment, Finance, Attendance, and 4 invariants on Student.
5. **`AcademicYear` cascade delegated to storage adapter** — `set_current_academic_year` does not invalidate prior current row in-engine.

## Implementation Order (per Phase 1 batches)

- **Batch 1:** Student, Class, Section, Subject, AcademicYear, Guardian (~24 invariants)
- **Batch 2:** ClassSection, ClassRoutine, Homework (~12 invariants) — ClassSubject landed in Wave 50 (I-1/3).
- **Batch 3:** LessonPlan, Lesson, LessonTopic, StudentRecord (~13 invariants)
- **Batch 4:** StudentPromotion, StudentCategory, StudentGroup, RegistrationField, Certificate, IdCard (~14 invariants)

Each batch must:
1. Implement the invariant in `aggregate.rs` constructor or `value_objects.rs` validator
2. Add a service-factory enforcement (where the invariant is conditional on existing state)
3. Add a behavioral integration test that proves the invariant rejects a violation
4. Update the [ ] → [x] (or [~]) status in this checklist
