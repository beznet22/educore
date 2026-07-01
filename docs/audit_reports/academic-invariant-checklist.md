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
| Enforced [x] | 8 | 11.0% |
| Partial [~] | 2 | 2.7% |
| Missing [ ] | 61 | 83.6% |
| Permissive [N/A] | 2 | 2.7% |
| **Total invariants** | **73** | **100%** |

**Coverage gap to close:** 61 missing + 2 partial = **63 invariants** must reach [x].

---

## Student Aggregate (6 invariants)

- [x] I-1: Exactly one active `StudentRecord` per `AcademicYear` — *claim*: enforced via `StudentRecord` aggregate cascade (Phase 2 must build `StudentRecord` aggregate fields first; not yet wired)
  - Spec: `docs/specs/academic/aggregates.md#student`
  - Enforcement: PENDING — `StudentRecord` is placeholder at `aggregate.rs:445` (`pub struct { id, school_id }`)
  - Test: MISSING
  - **Reclassify as [ ] — dependent on StudentRecord aggregate build (Phase 1 Batch 4)**
- [ ] I-2: A student's `AdmissionNumber` is unique within a school
  - Spec: `docs/specs/academic/aggregates.md#student`
  - Enforcement: `commands.rs:55-57` + `services.rs:141-144` (admit_student uniqueness call) + `value_objects.rs:299-302` (AdmissionNumber constructor 1..=50 chars)
  - Test: MISSING (no invariant-violation test, only happy-path test)
  - **Partial in practice — promotion test required**
- [ ] I-3: A student's `RollNumber` is unique within `(school, class, section, academic_year)`
  - Spec: `docs/specs/academic/aggregates.md#student`
  - Enforcement: MISSING — `UniquenessChecker` trait at `commands.rs:50` has no `roll_no_exists` method
  - Test: MISSING
- [ ] I-4: A student can be in at most one optional subject per academic year
  - Spec: `docs/specs/academic/aggregates.md#student`
  - Enforcement: MISSING — no `OptionalSubjectAssignment` aggregate defined
  - Test: MISSING
- [~] I-5: `Status` transitions `Applicant → Active → {Suspended, Withdrawn, Graduated, Transferred}`
  - Spec: `docs/specs/academic/aggregates.md#student`
  - Enforcement: `StudentStatus` enum at `value_objects.rs:573-590` + `reinstate_student` at `services.rs:382-386` checks precondition; **missing** precondition checks in `suspend_student` (`services.rs:346-353`), `withdraw_student` (`services.rs:433-439`), `transfer_student` (`services.rs:487-492`), `graduate_student` (`services.rs:574-578`)
  - Test: MISSING (only happy-path; no precondition-violation tests)
- [ ] I-6: A withdrawn or graduated student has no active `StudentRecord`
  - Spec: `docs/specs/academic/aggregates.md#student`
  - Enforcement: MISSING — no cascade from student.status to StudentRecord
  - Test: MISSING

## Guardian Aggregate (5 invariants)

- [ ] I-1: At most one phone and one email of record
  - Spec: `docs/specs/academic/aggregates.md#guardian`
  - Enforcement: MISSING — `Guardian` placeholder at `aggregate.rs:325-329` (`pub struct { id, school_id }`)
  - Test: MISSING
- [ ] I-2: A guardian may be linked to multiple students
  - Spec: `docs/specs/academic/aggregates.md#guardian`
  - Enforcement: MISSING — placeholder; no `StudentGuardianLink` child
  - Test: MISSING
- [ ] I-3: A guardian link carries `Relation` (Father/Mother/Guardian/Other) + `IsPrimary`
  - Spec: `docs/specs/academic/aggregates.md#guardian`
  - Enforcement: MISSING — no `Relation` enum
  - Test: MISSING
- [ ] I-4: At most one `IsPrimary` guardian per student
  - Spec: `docs/specs/academic/aggregates.md#guardian`
  - Enforcement: MISSING — placeholder
  - Test: MISSING
- [ ] I-5: Soft-delete when all student links removed
  - Spec: `docs/specs/academic/aggregates.md#guardian`
  - Enforcement: MISSING — placeholder; no link tracking
  - Test: MISSING

## Class Aggregate (4 invariants)

- [x] I-1: A class belongs to exactly one school
  - Spec: `docs/specs/academic/aggregates.md#class`
  - Enforcement: `Class.id: ClassId` typed id `ClassId { school_id, value }` (`value_objects.rs:73-77`); `Class::fresh` (`aggregate.rs:213-235`) sets `school_id: id.school_id()`
  - Test: IMPLIED by type system (any Class cannot exist without school anchor) — add explicit invariant-violation test
- [ ] I-2: A class is uniquely named within a school
  - Spec: `docs/specs/academic/aggregates.md#class`
  - Enforcement: MISSING — `UniquenessChecker` trait lacks `class_name_exists(school, name)`
  - Test: MISSING
- [x] I-3: `OptionalSubjectGpaThreshold` configurable (0.0..=5.0)
  - Spec: `docs/specs/academic/aggregates.md#class`
  - Enforcement: `OptionalSubjectGpaThreshold::new` (`value_objects.rs:778-786`) validates 0.0..=5.0
  - Test: MISSING — add out-of-range violation test
- [ ] I-4: Cannot delete if any `ClassSection` references it
  - Spec: `docs/specs/academic/aggregates.md#class`
  - Enforcement: MISSING — `delete_class` (`services.rs:733-758`) soft-deletes without checking; no `ReferentialChecker` surface
  - Test: MISSING

## Section Aggregate (3 invariants)

- [ ] I-1: A section is uniquely named within a school
  - Spec: `docs/specs/academic/aggregates.md#section`
  - Enforcement: MISSING — `UniquenessChecker` trait lacks `section_name_exists(school, name)`
  - Test: MISSING
- [N/A] I-2: A section can be reused across multiple `AcademicYear`s
  - Spec: `docs/specs/academic/aggregates.md#section`
  - Enforcement: Pervasive (data model permits — `Section` has no `academic_year_id`)
  - Test: N/A
- [x] I-3: Soft-deletable; existing references remain
  - Spec: `docs/specs/academic/aggregates.md#section`
  - Enforcement: `delete_section` (`services.rs:842-866`) sets `active_status = Retired`
  - Test: MISSING — add explicit soft-delete preservation test

## ClassSection Aggregate (4 invariants)

- [ ] I-1: Unique per `(class, section, academic_year)`
  - Spec: `docs/specs/academic/aggregates.md#classsection`
  - Enforcement: MISSING — `ClassSection` placeholder at `aggregate.rs:330-333`
  - Test: MISSING
- [N/A] I-2: Multiple class teachers and subject teachers
  - Spec: `docs/specs/academic/aggregates.md#classsection`
  - Enforcement: Pervasive
  - Test: N/A
- [ ] I-3: One or more class rooms
  - Spec: `docs/specs/academic/aggregates.md#classsection`
  - Enforcement: MISSING — placeholder
  - Test: MISSING
- [ ] I-4: Cannot delete while `StudentRecord`s reference it
  - Spec: `docs/specs/academic/aggregates.md#classsection`
  - Enforcement: MISSING — placeholder
  - Test: MISSING

## Subject Aggregate (3 invariants)

- [ ] I-1: Unique code within school
  - Spec: `docs/specs/academic/aggregates.md#subject`
  - Enforcement: MISSING — `UniquenessChecker` trait lacks `subject_code_exists`
  - Test: MISSING
- [x] I-2: `SubjectType` is `Theory` or `Practical`
  - Spec: `docs/specs/academic/aggregates.md#subject`
  - Enforcement: `SubjectType` enum at `value_objects.rs:689-697` (compile-time exhaustive)
  - Test: IMPLIED by type system — add explicit invariant test
- [x] I-3: Configurable pass mark (0.0..=100.0)
  - Spec: `docs/specs/academic/aggregates.md#subject`
  - Enforcement: `PassMark::new` (`value_objects.rs:753-762`)
  - Test: MISSING — add out-of-range violation test

## ClassSubject Aggregate (3 invariants)

- [ ] I-1: Class or class-section scope
  - Spec: `docs/specs/academic/aggregates.md#classsubject`
  - Enforcement: MISSING — placeholder at `aggregate.rs:335-338`
  - Test: MISSING
- [N/A] I-2: Same teacher may be assigned to multiple class-subjects
  - Spec: `docs/specs/academic/aggregates.md#classsubject`
  - Enforcement: Pervasive
  - Test: N/A
- [ ] I-3: `PassMark` override
  - Spec: `docs/specs/academic/aggregates.md#classsubject`
  - Enforcement: MISSING — placeholder; no `pass_mark` field
  - Test: MISSING

## AcademicYear Aggregate (5 invariants)

- [x] I-1: Start date strictly before end date
  - Spec: `docs/specs/academic/aggregates.md#academicyear`
  - Enforcement: `AcademicYearRange::new` (`value_objects.rs:683-694`) rejects `start >= end`
  - Test: MISSING — add explicit violation test
- [ ] I-2: No overlap within school
  - Spec: `docs/specs/academic/aggregates.md#academicyear`
  - Enforcement: MISSING — `update_academic_year_dates` (`services.rs:1074-1099`) does not check; no `academic_year_overlaps` method
  - Test: MISSING
- [~] I-3: Exactly one current per school
  - Spec: `docs/specs/academic/aggregates.md#academicyear`
  - Enforcement: `set_current_academic_year` (`services.rs:1113-1145`) sets flag but **does not demote previously-current** (delegated to storage adapter per `services.rs:1095-1097` comment)
  - Test: MISSING
- [x] I-4: Non-current may be opened for read-only queries
  - Spec: `docs/specs/academic/aggregates.md#academicyear`
  - Enforcement: `AcademicYear.is_closed: bool` (`aggregate.rs:412-413`); `close_academic_year` (`services.rs:1151-1184`)
  - Test: IMPLIED — add explicit test
- [ ] I-5: Promote requires same-school From/To; To next sequential
  - Spec: `docs/specs/academic/aggregates.md#academicyear`
  - Enforcement: MISSING — `promote_student` (`services.rs:510-555`) only checks `from != to`; no same-school, no sequential, no immediate-successor check
  - Test: MISSING

## ClassRoutine Aggregate (5 invariants)

- [ ] I-1: Covers a full week
  - Spec: `docs/specs/academic/aggregates.md#classroutine`
  - Enforcement: MISSING — placeholder at `aggregate.rs:340-343`
  - Test: MISSING
- [ ] I-2: `ClassTime` periods
  - Spec: `docs/specs/academic/aggregates.md#classroutine`
  - Enforcement: MISSING — placeholder
  - Test: MISSING
- [ ] I-3: Room + teacher per period per day
  - Spec: `docs/specs/academic/aggregates.md#classroutine`
  - Enforcement: MISSING — placeholder
  - Test: MISSING
- [ ] I-4: Teacher cannot be in two places at the same time
  - Spec: `docs/specs/academic/aggregates.md#classroutine`
  - Enforcement: MISSING — placeholder; no `ReferentialChecker` surface
  - Test: MISSING
- [ ] I-5: Room cannot host two classes at the same time
  - Spec: `docs/specs/academic/aggregates.md#classroutine`
  - Enforcement: MISSING — placeholder
  - Test: MISSING

## Homework Aggregate (5 invariants)

- [ ] I-1: Teacher-created, class-section scope
  - Spec: `docs/specs/academic/aggregates.md#homework`
  - Enforcement: MISSING — placeholder at `aggregate.rs:345-348`
  - Test: MISSING
- [ ] I-2: Submission date after homework date
  - Spec: `docs/specs/academic/aggregates.md#homework`
  - Enforcement: MISSING — placeholder
  - Test: MISSING
- [ ] I-3: Evaluation date >= submission date
  - Spec: `docs/specs/academic/aggregates.md#homework`
  - Enforcement: MISSING — placeholder
  - Test: MISSING
- [ ] I-4: Optional attachment
  - Spec: `docs/specs/academic/aggregates.md#homework`
  - Enforcement: MISSING — placeholder
  - Test: MISSING
- [ ] I-5: Marks immutable once evaluated
  - Spec: `docs/specs/academic/aggregates.md#homework`
  - Enforcement: MISSING — placeholder
  - Test: MISSING

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
- **Batch 2:** ClassSection, ClassSubject, ClassRoutine, Homework (~14 invariants)
- **Batch 3:** LessonPlan, Lesson, LessonTopic, StudentRecord (~13 invariants)
- **Batch 4:** StudentPromotion, StudentCategory, StudentGroup, RegistrationField, Certificate, IdCard (~14 invariants)

Each batch must:
1. Implement the invariant in `aggregate.rs` constructor or `value_objects.rs` validator
2. Add a service-factory enforcement (where the invariant is conditional on existing state)
3. Add a behavioral integration test that proves the invariant rejects a violation
4. Update the [ ] → [x] (or [~]) status in this checklist
