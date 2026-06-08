# Assessment Domain — Business Analysis

## Purpose

The assessment domain owns examinations, marks, results,
and report cards. It is the academic domain's companion:
where the academic domain tracks "what is the student
learning?", the assessment domain tracks "how well is
the student learning?"

This document describes how exams, marks, results, and
report cards work in real schools, with the edge cases
that real schools hit.

## Key Concepts

- **ExamType** — a category of exam (e.g. "Unit Test",
  "Mid-Term", "Final"). Each type has a weight in the
  final grade.
- **Exam** — an instance of an exam for a class in an
  academic year. Has a date range and a status.
- **ExamSchedule** — a subject-level schedule for an
  exam. Defines the date, time, room, and invigilator.
- **Mark** — a student's score in a single subject for
  a single exam.
- **Result** — a student's aggregated result for an
  exam, across all subjects.
- **ReportCard** — a formatted document produced from a
  result. The format is a template; the rendering is a
  port concern.
- **Grade** — a letter or numerical grade assigned
  based on marks.
- **PassMark** — the threshold above which a student
  passes. Configurable per school.

## Real-World Scenarios

### Exam Scheduling

When a school schedules an exam:

1. The exam officer creates the `Exam` with a date
   range and an `ExamType`.
2. The exam officer creates one `ExamSchedule` per
   `(class, section, subject)`. Each schedule has a
   date, a time, a room, and an invigilator.
3. The schedule is published. The communication
   domain sends a "Exam Schedule" notice to parents.
4. The exam is conducted. Marks are recorded.
5. The exam is marked "completed" once all marks
   are entered.

A real school may have multiple exams in a year
(quarterly, half-yearly, annually). The exam type's
weight determines how much each exam contributes to
the final grade.

### Mark Entry

A teacher enters marks for their subject, for their
class-section, for the exam. Marks are entered
subject-by-subject, class-by-class, in a typical
school.

In real schools, mark entry is:
- **Confidential** — only the teacher and the exam
  officer see the marks before publication.
- **Single-entry per student** — a student has one
  mark per `(exam, subject)`. Re-entry is an update.
- **Validated** — marks are between 0 and the maximum
  (e.g. 0-100 for a 100-mark exam). Marks above the
  maximum are rejected.
- **Optional absent flag** — an absent student has
  no mark; the engine records "absent" instead of
  zero.
- **Bulk-entered** — the teacher may enter all
  marks in one screen via a CSV upload or a tabular
  UI. The engine supports bulk mark entry.

### Result Computation

Once all marks are entered for an exam, the engine
computes the result:
- Total marks.
- Percentage.
- Grade (per the school's grading scale).
- Pass / fail (per the school's pass mark).
- Rank (per class-section, if enabled).

The engine's `Result` aggregate is the canonical
record of the computed outcome. The engine emits
`ResultComputed` events; the report card projection
updates.

### Result Publication

A real school publishes results in a controlled way:
- A pre-publication review by the principal.
- A scheduled publication time.
- A notice to parents ("results are now available").
- A parent portal where the parent sees the report
  card and can download it.

The engine's `PublishResult` command captures the
publication. The engine emits `ResultPublished`
events; the communication domain sends notices; the
report card projection becomes visible to parents.

### Report Card Generation

A report card is a formatted document showing the
student's result, the class teacher's comment, the
principal's remark, and the attendance summary.

The engine's `ReportCard` aggregate is the canonical
record; the actual PDF is rendered by a port-driven
adapter. The engine's command set includes:
- `GenerateReportCard` — for a single result.
- `GenerateBulkReportCards` — for all results in an
  exam.
- `PrintReportCard` — invokes the consumer's PDF
  adapter.

### Grade Scales

Real schools use a variety of grading scales:
- Letter grades: A, B, C, D, F.
- Numerical grades: 1-10, 1-5.
- Percentage: 0-100%.
- Cumulative grade point average (GPA).

The engine's `GradeScale` is a per-school
configuration. The engine computes the grade from
the marks using the configured scale.

### Mark Distribution and Analytics

A real school may want to analyze marks:
- Average per subject.
- Pass rate per class.
- Distribution (how many As, Bs, etc.).
- Comparison to prior years.
- Subject-wise topper.

The engine's `Report.Generate` command produces
these analytics. The command is capability-gated.

## Business Rules

1. Marks are between 0 and the exam's maximum. Marks
   outside this range are rejected.
2. A student has at most one mark per `(exam, subject)`.
3. Marks cannot be entered for an exam that is not
   yet "in progress" or "completed."
4. Marks can be updated before publication. After
   publication, marks can only be updated with a
   `Mark.Update` command and an audit reason.
5. Results are computed from marks; results cannot
   be entered manually (they are derived).
6. Result publication requires all marks for the
   exam to be entered.
7. Report cards are generated from results; report
   cards cannot be generated before the result is
   computed.
8. The pass mark and grade scale are per-school
   configuration; the engine reads them from the
   settings domain.
9. An exam cannot be deleted once marks have been
   entered. An exam can be "voided" with a reason.

## Edge Cases

### Absent Student

A student is absent for an exam. The engine records
"absent" instead of zero. The student's result shows
"absent" for that subject. The pass/fail decision
accounts for absences per the school's policy
(configured in settings).

### Re-Exam

A student who fails is allowed a re-exam. The
re-exam is a new `Exam` of type "Re-Exam" linked to
the original. The engine's marks for the re-exam
override the original marks for the result.

### Make-Up Exam

A student who misses an exam for a medical reason is
allowed a make-up. The make-up is a new `ExamSchedule`
for the same exam, with a different date. The engine
records the mark in the same `Mark` row (overwriting
"absent").

### Subject With No Students

An exam schedule is created for a subject that has
no enrolled students. The engine allows creation
but emits a warning. The school may cancel the
schedule.

### Exam Schedule Clash

Two exams are scheduled at the same time for the
same class. The engine's validation rejects the
clash on creation. The exam officer reschedules.

### Bulk Mark Upload with Errors

A teacher uploads a CSV with 30 students' marks. 28
are valid; 2 have marks above the maximum. The
engine's bulk command is **all-or-nothing**: the
2 invalid marks fail the entire upload. The teacher
fixes the CSV and re-uploads.

### Partial Result Publication

A school wants to publish results for some classes
but not others. The engine's `PublishResult`
command supports a class filter. The other classes
remain unpublished.

### Result Correction After Publication

A teacher notices a mark entry error after
publication. The school allows correction with
auditing. The engine's `Mark.Update` command
requires a `correction_reason` and is capability-
gated to the exam officer. The result is re-
computed; the report card is regenerated; the
audit log captures the correction.

### Subject Failed but Overall Passed

A student fails one subject but passes overall
(per the school's promotion policy). The engine's
result shows the subject as "fail" and the overall
as "pass." The report card shows both.

### Tie in Rank

Two students have the same total marks. The
engine's rank computation breaks the tie by:
1. Higher marks in the primary subject.
2. Higher marks in the next subject.
3. Alphabetical by name.

The tie-breaking policy is configurable per school.

## Notes for SMScore Implementation

- The **assessment** crate depends on
  `smscore-academic` for `StudentId`, `ClassId`,
  `SubjectId`, `AcademicYearId`.
- Marks are **write-once-ish**: a mark is created,
  may be updated before publication, and may be
  corrected (with audit) after publication. The
  engine's audit log is the canonical record of
  every change.
- Results are **derived from marks**. The engine's
  `Result` aggregate is a projection; the marks
  are the source of truth. The engine supports
  result recomputation from marks.
- Publication is a **transition**: an exam has
  states `Draft → InProgress → Completed →
  Published → Archived`. Each transition is a
  command and an event.
- The grading scale and pass mark are **per-school
  configuration**. The engine reads them at
  command dispatch time.
- Bulk mark entry is a **first-class command**.
  The engine's bulk command is all-or-nothing;
  partial success is not supported.
- Report card generation is **port-driven**. The
  engine's `ReportCard` aggregate is the canonical
  record; the PDF is rendered by a consumer
  adapter.
- Analytics are **port-driven**. The engine's
  `Report.Generate` command produces
  capability-gated analytics; the consumer chooses
  the format.
- The domain's events (`ExamScheduled`,
  `MarksEntered`, `ResultComputed`,
  `ResultPublished`, `ReportCardGenerated`) drive
  downstream projections and communication.
