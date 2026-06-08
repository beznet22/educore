# Report Card Generation Guide

## Goal

Generate a report card for a student in a class-section-term
combination, computing GPA, grade, subject-wise marks, total marks,
and class position.

## Inputs

- **MarksRegister**: per-subject marks entered by teachers.
- **MarksGrade**: grade boundaries configured by the school.
- **ResultSettings**: configuration (pass mark, GPA scale, weight
  per exam).
- **ExamSchedule**: which exams count toward the report.

## Workflow

```text
1. School configures MarksGrades (A+ = 90-100, A = 80-89, ...).
2. School configures ResultSettings.
3. School schedules exams.
4. Teachers enter marks.
5. School closes marks entry.
6. School triggers ResultComputation.
7. The engine computes per-subject: total marks, grade, GPA.
8. The engine computes class: total marks, GPA, grade, position.
9. The school publishes the result.
10. Parents/students see the report card.
11. Optional: print as PDF.
```

## Marks Entry

```rust
engine.assessment().enter_marks(EnterMarksCommand {
    tenant,
    exam_id,
    class_id,
    section_id,
    subject_id,
    student_records: vec![
        StudentMark { student_id, marks: 85.0, absent: false },
        StudentMark { student_id, marks: 90.0, absent: false },
        StudentMark { student_id, marks: 0.0, absent: true },
    ],
    entered_by: teacher_id,
}).await?;
```

## Marks Closure

When all subjects are entered, the school closes the marks register:

```rust
engine.assessment().close_marks(CloseMarksCommand {
    tenant,
    exam_id,
    class_id,
    section_id,
}).await?;
```

The engine emits `MarksClosed` and the register becomes read-only.

## Result Computation

```rust
engine.assessment().compute_result(ComputeResultCommand {
    tenant,
    exam_id,
    class_id,
    section_id,
    academic_year_id,
}).await?;
```

The engine:

1. Sums each student's marks across subjects.
2. Computes GPA per subject (using the configured scale).
3. Computes overall GPA (weighted average across subjects).
4. Determines grade (A, B, C, ...).
5. Computes class position (1st, 2nd, ...).
6. Computes section position.
7. Records `ResultStore` entries.

## GPA Computation

The engine uses the school's grade configuration:

```rust
pub struct MarksGrade {
    pub grade_name: String,         // "A+", "A", "B", ...
    pub point: f32,                 // 5.0, 4.0, 3.0, ...
    pub percent_from: f32,          // 90.0
    pub percent_to: f32,            // 100.0
    pub description: String,        // "Excellent"
}
```

The engine looks up the grade for the percentage and emits the
grade and point.

## Result Publication

The school publishes the result:

```rust
engine.assessment().publish_result(PublishResultCommand {
    tenant,
    exam_id,
    class_id,
    section_id,
    published_by: principal_id,
}).await?;
```

The engine:

1. Sets the result status to `Published`.
2. Emits `ResultPublished`.
3. Sends notifications to parents (via the notification port).
4. Generates report cards (PDF or other format, consumer-supplied
   adapter).

## Report Card Output

A report card contains:

- School header (name, logo, address).
- Student information (name, admission number, class, section).
- Subject table: subject, marks obtained, max marks, grade, GPA.
- Summary: total marks, total max, percentage, GPA, grade.
- Class position, section position.
- Attendance summary.
- Teacher remarks.
- Principal signature.
- Date.

The output format is consumer-supplied. The engine produces a
`ReportCard` aggregate; the consumer renders it.

## Online Exam Integration

Online exams (MCQ) integrate with the report card:

1. The school creates an `OnlineExam`.
2. Students take the exam via the LMS or in-class.
3. The system auto-grades and records `OnlineExamMark`.
4. On closure, the marks feed into the report card as a
   `Source::OnlineExam` entry.

## Edge Cases

- **Tie in total marks**: position is shared. The next position is
  skipped (e.g. 1st, 1st, 3rd).
- **Subject failed**: the student may still have a passing GPA if
  the school allows. Otherwise, `ResultStatus::Fail`.
- **Absent in all subjects**: `ResultStatus::Incomplete`.
- **Optional subject**: included if chosen; otherwise excluded from
  GPA computation.

## Promotion Eligibility

The promotion workflow reads the published result:

```rust
let result = engine.assessment().get_result(student_id, exam_id).await?;
if result.passed() {
    engine.students().promote(...).await?;
}
```

## Audit

Every marks entry, closure, computation, and publication is audited.
Marks are immutable once the register is closed; corrections require
a new register version.

## Worked Example

A school runs a mid-term report card:

```rust
// 1. Configure grades
engine.assessment().configure_grades(...).await?;

// 2. Schedule exam
let exam = engine.assessment().create_exam(CreateExamCommand {
    tenant,
    name: "Mid-Term 2026",
    exam_type: ExamType::MidTerm,
    class_id,
    section_id,
    academic_year_id,
}).await?;

// 3. Enter marks per subject
for subject in subjects {
    engine.assessment().enter_marks(EnterMarksCommand {
        ...,
        subject_id: subject.id,
        student_records: ...,
    }).await?;
}

// 4. Close and compute
engine.assessment().close_marks(...).await?;
engine.assessment().compute_result(...).await?;

// 5. Publish
engine.assessment().publish_result(...).await?;
```

Parents receive an SMS / email with a link to view the report card.
