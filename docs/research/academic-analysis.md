# Academic Domain — Business Analysis

## Purpose

The academic domain owns the lifecycle of students and the
structure of the school day. It is the foundational domain
of the engine: every other operational domain refers to
aggregates defined here.

This document describes how admission, class management,
routines, homework, lessons, and student lifecycle
**actually work in real schools**, with the edge cases
that real schools hit.

## Key Concepts

- **Student** — a person enrolled in a school, identified
  by an admission number unique within the school.
- **Guardian** — a parent, legal guardian, or contact
  authorized to act on behalf of a student.
- **Class** — a grade level (e.g. "Grade 7").
- **Section** — a division of a class (e.g. "Section A",
  "Section B"). Each section has a capacity.
- **ClassSection** — the pairing of a class and a section
  in a given academic year.
- **Subject** — a course of study (e.g. "Mathematics",
  "English").
- **AcademicYear** — a school year with a date range and
  a "current" flag.
- **ClassRoutine** — a weekly schedule for a class-section-
  subject, defining which teacher teaches which subject
  on which day at which time.
- **Homework** — an assignment given to a class-section-
  subject.
- **LessonPlan** — a teacher's planned lesson for a
  specific day.
- **StudentRecord** — a student's enrollment in a specific
  academic year. A student has one `StudentRecord` per
  year.
- **Promotion** — moving a `StudentRecord` from one
  academic year to the next.

## Real-World Scenarios

### Admission

When a parent admits a child, the school:

1. Captures the child's profile (name, date of birth,
   gender, address).
2. Captures the guardian(s) (name, phone, email,
   relationship).
3. Assigns an admission number (unique within the
   school).
4. Assigns the child to a class and section for the
   current academic year.
5. Captures optional information: blood group, religion,
   caste, prior school, transport route, hostel
   preference, custom fields.
6. Uploads supporting documents (birth certificate,
   prior school transcript).
7. Records the admission date.

In real schools, admissions are **seasonal**. The
majority happen in the weeks before the academic year
starts. A school may admit a few students mid-year as
transfer cases.

A real school may admit the same student into two
different academic years (e.g. mid-year transfer
into a higher class). The engine's `StudentRecord` is
the per-year enrollment; the `Student` is the
**person**, surviving across years.

### Class and Section Management

A school has a fixed catalog of classes (Grade 1
through Grade 12, or Year 1 through Year 13, or
Kindergarten through Year 12). Each class may have
multiple sections, depending on enrollment. Section
capacities are configured per school.

A school opens sections at the start of the academic
year and may add sections mid-year if enrollment
exceeds expectations. A school rarely closes a
section mid-year; if a section's enrollment drops
below a threshold, students are merged into another
section.

The engine's invariants: a class cannot be deleted
while active `StudentRecord`s reference it; a section
cannot be deleted while active `StudentRecord`s
reference it.

### Academic Year and Term

A school has a single current `AcademicYear` at a time.
The year has a start date and an end date. Within the
year, the school may have terms (semesters, trimesters,
or quarters). The engine's `AcademicYear` aggregate
owns the year-level settings.

The engine emits events on academic year transitions:
`AcademicYearCreated`, `AcademicYearSetCurrent`,
`AcademicYearClosed`. The settings domain subscribes
to refresh its current-year pointer.

### Class Routine

A class routine is the weekly schedule of which teacher
teaches which subject to which class-section at which
time. A routine is a composite of:
- Day of week.
- Period (a time slot).
- Class-section.
- Subject.
- Teacher.
- Room.

A routine may have multiple periods per day. The
school may rotate the routine weekly, bi-weekly, or
keep it static for the year.

In real schools, routines are **printed and posted**.
The engine's command set includes routines creation,
update, and per-day or per-week swap. The engine
emits `ClassRoutineUpdated` and `PeriodSwapped` events;
the timetable projection (a read model) updates
accordingly.

### Homework

A teacher creates a homework assignment for a class-
section-subject. The assignment has:
- A title.
- A description.
- A due date.
- An optional attachment (file or link).

Students submit homework; teachers evaluate. The
engine's `Homework` aggregate tracks the assignment
and the per-student submission status. The engine
emits `HomeworkCreated`, `HomeworkSubmitted`,
`HomeworkEvaluated` events.

A real school's homework is not just a grade; it is
also a record of "did the student submit on time?"
The engine's `HomeworkSubmission` carries both
`status` and `submitted_at`.

### Lesson Plan

A lesson plan is a teacher's plan for a specific day.
It includes:
- The lesson's topic.
- The teaching method.
- The general objectives.
- The previous knowledge required.
- The completion question.
- The lecture video link (YouTube, Vimeo).
- The attachment (PDF, slide deck).
- The note.

The engine's `LessonPlan` aggregate is per-day-per-
class-section-subject. The engine's
`LessonTopic` and `Lesson` aggregates organize the
curriculum into topics and lessons.

In real schools, lesson plans are a regulatory
requirement. The engine's audit log captures every
plan, every update, every completion.

### Student Promotion

At the end of the academic year, the school
**promotes** students to the next class. Promotion is
a `StudentRecord`-level operation: the student's
current `StudentRecord` is closed, and a new
`StudentRecord` is opened in the next academic year,
in the new class.

A real school's promotion is **not** uniform. Some
students are promoted; some are held back; some
graduate (the final class); some withdraw. The engine
supports each transition:
- `PromoteStudent` — to a higher class.
- `HoldStudent` — repeat the same class.
- `GraduateStudent` — for the graduating cohort.
- `WithdrawStudent` — leave the school.

The engine emits `StudentPromoted`, `StudentHeld`,
`StudentGraduated`, `StudentWithdrawn` events. The
finance domain subscribes to roll over balances and
assign new fees. The attendance domain subscribes to
reset the daily expectation. The assessment domain
subscribes to archive prior marks and prepare new
exam schedules.

### Student Withdrawal

A student leaves the school. Reasons include:
- Family relocation.
- Transfer to another school.
- Financial hardship.
- Disciplinary.
- Health.

The withdrawal captures the reason, the date, and
the destination (if transfer). The engine emits
`StudentWithdrawn`; finance finalizes outstanding
balances, library returns outstanding books,
transport removes the student from the route,
communication stops notifications.

The student's `StudentRecord` is closed but retained
for historical reference. The `Student` aggregate's
status becomes `Withdrawn`.

### Student Category and Group

A school groups students for various purposes:
- **Category** — a flag (scholarship, staff child,
  sibling discount, etc.) that affects fees or
  reporting.
- **Group** — a non-fee grouping (sports team, debate
  club, music ensemble).

The engine's `StudentCategory` and `StudentGroup`
aggregates capture these. A student may be in
multiple groups but typically one category at a time.

### Custom Registration Fields

A school may want to capture information beyond the
default profile (e.g. "allergies", "emergency contact
2", "hostel preference"). The engine's
`RegistrationField` aggregate lets a school admin
define custom fields that the admission form
captures. The engine's `CustomField` mechanism
(provided by the platform domain) is the foundation.

### Certificates and ID Cards

A school issues certificates (transfer certificate,
conduct certificate, bonafide certificate) and ID
cards. The engine's `Certificate` and `IdCard`
aggregates are templates; the actual document
generation is a port-driven concern (PDF renderer
provided by the consumer).

## Business Rules

The engine's academic domain enforces the following
rules:

1. A `Student` belongs to exactly one `SchoolId`.
2. A `Student` is admitted into exactly one
   `Class`/`Section` per academic year via a
   `StudentRecord`.
3. An `AcademicYear` cannot overlap another
   `AcademicYear` in the same school.
4. Exactly one `AcademicYear` is current per school
   at a time.
5. A `ClassSection` is unique per `Class`, `Section`,
   and `AcademicYear`.
6. A `Subject` is unique by code within a school.
7. A `Class` may not be deleted while active
   `StudentRecord`s reference it.
8. A `Student` cannot be admitted twice into the same
   `AcademicYear`.
9. Promotion moves a `StudentRecord` forward, never
   backward, and produces a `StudentPromoted` event.
10. Withdrawn students retain their historical
    `StudentRecord`s but cannot receive new commands
    other than read.
11. Guardians have at most one primary contact flagged
    per student.
12. A `ClassRoutine` requires a teacher who is
    assigned to the school and a subject that the
    class is enrolled in.
13. A `Homework`'s due date is in the future at
    creation.
14. A `LessonPlan`'s lesson date is in the past or
    present (you cannot plan for the future).

## Edge Cases

### Mid-Year Transfer

A student transfers from school A to school B mid-
year. The source school issues a transfer
certificate and withdraws the student. The
destination school admits the student. The
engine's `TransferStudent` command is cross-tenant
and capability-gated; the academic domain emits
`StudentTransferred` events on both sides.

### Family of Three Siblings

A family has three children in the school. The
parent's user account is the guardian for all three.
The parent's portal shows three `StudentRecord`s,
three sets of marks, three sets of attendance, one
set of fees. The engine's `Guardian` aggregate
links to multiple `Student` aggregates; the
parent's effective capabilities include the
parent's own plus the read-only access to each
child.

### Student Whose Guardian Is Also a Staff Member

A teacher's child attends the same school. The
teacher's user account is both a staff member
(parent role) and a guardian (parent role). The
engine's `TenantContext` resolves the active role
based on the action: a `Mark.Create` command
activates the staff role; a `FeesAssign.Read`
command for the child activates the parent role.

### Student With Scholarship

A student has a scholarship. The scholarship
waives 50% of tuition. The engine's
`StudentCategory` is `Scholarship`; the engine's
finance domain's `FeesAssign` carries a discount.
The parent's portal shows the original amount, the
discount, and the net due.

### Student With Both Parents in the System

A school admits a child with two parents. The
engine's `Guardian` aggregate supports multiple
links; the engine marks one as `IsPrimary`. The
parent portal's user is the primary guardian; the
secondary guardian has read-only access.

### Class with More Students Than Capacity

A school admits 35 students into a section whose
capacity is 30. The school's policy is to exceed
the capacity for a one-time intake. The engine
emits a warning but admits. The school's admin
reviews the section's enrollment and may split it
into a new section.

### Student Who Fails to Promote

A student fails the year-end exam. The school
holds them back. The engine's `HoldStudent`
command creates a new `StudentRecord` for the
same class in the next academic year, with a
`reason` field capturing the hold reason.

### Student Who Withdraws Mid-Term

A family relocates mid-term. The school
withdraws the student. The engine's
`WithdrawStudent` command captures the reason
and the date. The finance domain finalizes
the outstanding balance; the parent pays
before records are released.

### Student with Repeated Absences

A student is absent for 30 consecutive days. The
school may auto-withdraw the student. The
engine does not auto-withdraw; the school's
policy (configured in the settings domain) is
evaluated by a domain service that proposes
the withdrawal. A human operator confirms.

### Section with No Students

A section is created at the start of the year but
no students are assigned. The section is "open"
but empty. The engine does not auto-close empty
sections; the school may keep them as a future
option or close them manually.

## Notes for SMScore Implementation

- The **academic** crate is the foundational domain.
  Every other operational domain depends on it.
- The `Student` aggregate is the most queried and
  the most mutated. Storage adapter performance
  matters.
- The `StudentRecord` is the per-year enrollment;
  it is the join key for fees, marks, attendance.
  Index on `(school_id, student_id, academic_year_id)`
  is mandatory.
- Promotion is a **command composition**: a single
  `PromoteStudent` command closes the old
  `StudentRecord` and opens a new one. The
  cross-domain events fire after the transaction.
- The academic domain's commands are
  **idempotent on `idempotency_key`**. A retry
  of an admission does not create a duplicate
  student.
- The domain's events drive the finance domain's
  fees assignment, the library domain's membership
  creation, the transport domain's route
  assignment, and the communication domain's
  welcome message. The cross-domain coordination
  is event-driven; the academic domain does not
  call the other domains directly.
- The domain's value objects
  (`AdmissionNumber`, `RollNumber`, `PersonName`,
  `DateOfBirth`) are validated at construction.
  A consumer cannot construct an invalid
  `AdmissionNumber`.
- The domain's policies (`PromotionPolicy`,
  `AdmissionPolicy`) are pure functions over
  state. They are easy to test and easy to
  override per school.
- The domain's repositories are port traits. The
  default storage adapter implements them on
  PostgreSQL, SQLite, and SurrealDB.
- The domain's commands are
  capability-gated. The RBAC domain's
  `CapabilityCheckService` is the gatekeeper.
- The domain's audit log captures every
  command and every event. The audit log is
  the school's institutional memory.
