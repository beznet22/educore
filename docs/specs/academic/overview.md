# Academic Domain Overview

## Purpose

The academic domain owns the lifecycle of students, the structure of the
school day, and the catalog of classes, sections, subjects, and academic
sessions. It is the foundational domain: every other operational domain
(attendance, assessment, finance, library, transport) refers to aggregates
defined here.

## Responsibilities

- Student admission, identity, profile, and lifecycle.
- Guardian associations and contact information.
- Class and section structure, including section promotion rules.
- Subject catalog and class-subject mapping.
- Academic year and term (session) lifecycle.
- Class teacher and subject teacher assignments.
- Class routines, period definitions, and room assignments.
- Homework creation, submission, and evaluation.
- Lesson plan, lesson, and topic structure.
- Student promotion between academic years.
- Student withdrawal, transfer, and graduation.
- Student category and group membership.
- Student registration field configuration.
- Student ID card and certificate templates.

## Boundaries

The academic domain does **not** own:

- Daily attendance records (see `specs/attendance/`).
- Marks, grades, exam schedules (see `specs/assessment/`).
- Fee assignment or invoicing (see `specs/finance/`).
- Staff payroll (see `specs/hr/`).
- Library book issues (see `specs/library/`).
- Transport route assignment (see `specs/facilities/`).
- Dormitory room assignment (see `specs/facilities/`).

The academic domain **does** provide identifier types and value objects
that other domains depend on: `StudentId`, `ClassId`, `SectionId`,
`SubjectId`, `AcademicYearId`, `SessionId`, `GuardianId`.

## Dependencies

- `smscore-core` — error types, result, identifier trait.
- `smscore-platform` — `SchoolId`, `UserId`, `TenantContext`.
- `smscore-rbac` — capability checks.
- `smscore-events` — domain event publishing.

## Domain Invariants

1. A `Student` belongs to exactly one school.
2. A `Student` is admitted into exactly one `Class`/`Section` per academic
   year via a `StudentRecord`.
3. An `AcademicYear` cannot overlap another `AcademicYear` in the same
   school.
4. Exactly one `AcademicYear` is current per school at a time.
5. A `ClassSection` is unique per `Class`, `Section`, and `AcademicYear`.
6. A `Subject` is unique by code within a school.
7. A `Class` may not be deleted while active `StudentRecord`s reference it.
8. A `Student` cannot be admitted twice into the same `AcademicYear`.
9. Promotion moves a `StudentRecord` forward, never backward, and produces a
   `StudentPromoted` event.
10. Withdrawn students retain their historical `StudentRecord`s but cannot
    receive new commands other than read.
11. Guardians have at most one primary contact flagged per student.

## Aggregate Roots

| Aggregate         | Root Type            | Purpose                                  |
| ----------------- | -------------------- | ---------------------------------------- |
| Student           | `Student`            | Student identity, profile, lifecycle    |
| Guardian          | `Guardian`           | Guardian identity, contact, relations   |
| Class             | `Class`              | A grade level, with sections, in a year  |
| Section           | `Section`            | A division of a class (e.g. Section A)   |
| ClassSection      | `ClassSection`       | Pairing of class and section in a year   |
| Subject           | `Subject`            | A subject offered in a class             |
| AcademicYear      | `AcademicYear`       | A school year with a date range          |
| ClassRoutine      | `ClassRoutine`       | Weekly schedule for a class-section-subj |
| Homework          | `Homework`           | Assignment given to a class-section-subj |
| LessonPlan        | `LessonPlan`         | Teacher's planned lesson for a day       |
| Lesson            | `Lesson`             | A unit of study                          |
| LessonTopic       | `LessonTopic`        | A topic within a lesson                  |
| StudentRecord     | `StudentRecord`      | A student's enrollment in a year         |
| StudentPromotion  | `StudentPromotion`   | Record of a promotion event              |
| StudentCategory   | `StudentCategory`    | A grouping (e.g. "scholarship", "staff")  |
| StudentGroup      | `StudentGroup`       | A grouping (e.g. "sports team")          |
| RegistrationField | `RegistrationField`  | Custom field on registration form        |
| Certificate       | `Certificate`        | Student certificate template             |
| IdCard            | `IdCard`             | Student ID card template                 |

Each aggregate is documented in detail under `docs/specs/academic/aggregates.md`.

## Cross-Domain Impact

When a `Student` is admitted, the academic domain emits `StudentAdmitted`.
The following domains may subscribe:

- `finance` may auto-create a fees assignment.
- `attendance` may auto-create a daily attendance expectation.
- `library` may auto-issue a library membership.
- `communication` may send a welcome message to the guardian.

When a `Student` is promoted, the academic domain emits `StudentPromoted`.
The following domains may subscribe:

- `finance` rolls over balances and assigns new fees.
- `attendance` resets the daily expectation for the new class.
- `assessment` archives prior marks and prepares new exam schedules.

When a `Student` is withdrawn, the academic domain emits `StudentWithdrawn`.
The following domains may subscribe:

- `finance` finalizes outstanding balances.
- `library` returns outstanding books.
- `transport` removes the student from the route.
- `communication` stops notifications.

## Consumers

- Web admin UI (admit students, manage classes).
- Mobile parent app (view profile, certificates).
- Mobile student app (view homework, lessons).
- Mobile teacher app (mark homework, manage lesson plan).
- AI agent (admit, promote, withdraw, transfer).

## Anti-Goals

- The academic domain does not present data to humans. It exposes commands,
  events, and queries.
- The academic domain does not import or export school data files. Bulk
  import is a port-driven adapter.
- The academic domain does not decide academic policy (e.g. "pass mark").
  That is a configuration value managed by the consumer.
