# Assessment Domain — Tables

The assessment domain is backed by the following tables. Each table
maps to one or more aggregates; the `aggregate` column tells you
which aggregate owns the row.

| Table                              | Aggregate                       | Notes                                  |
| ---------------------------------- | ------------------------------- | -------------------------------------- |
| `assessment_exam_types`                    | ExamType                        | Exam category catalog                  |
| `assessment_exams`                         | Exam                            | Per-(class,section,subject) exam       |
| `assessment_exam_setups`                   | ExamSetup                       | Section-level exam config              |
| `assessment_exam_schedules`                | ExamSchedule                    | Calendar slot for an exam              |
| `assessment_exam_schedule_subjects`        | ExamScheduleSubject             | Per-subject schedule entry             |
| `assessment_exam_settings`                 | ExamSetting                     | School exam publication                |
| `assessment_exam_signatures`               | ExamSignature                   | Signatures for report cards            |
| `assessment_marks_registers`               | MarksRegister                   | Per-student exam marks                 |
| `assessment_marks_register_children`       | MarksRegisterChild              | Per-subject marks row                  |
| `assessment_mark_stores`                   | MarkStore                       | Consolidated stored marks              |
| `assessment_result_stores`                 | ResultStore                     | Per-subject published result           |
| `assessment_marks_grades`                  | MarksGrade                      | Grade boundary scale                   |
| `assessment_temporary_meritlists`          | TemporaryMeritList              | Merit staging                          |
| `exam_merit_positions`             | MeritPosition                   | Final merit position                   |
| `assessment_exam_marks_registers`          | MarksRegister (alt)             | Alternate column-level table           |
| `all_exam_wise_positions`          | AllExamWisePosition             | Cross-section exam position            |
| `assessment_marks_send_sms`                | (notification trigger)          | Marks-SMS dispatch tracker             |
| `custom_result_settings`           | CustomResultSetting             | Custom result branding                 |
| `assessment_custom_temporary_results`      | CustomTemporaryResult           | Custom result staging                  |
| `exam_step_skips`                  | ExamStepSkip                    | Wizard-skip flag                       |
| `assessment_class_exam_routine_pages`      | ExamRoutinePage                 | Public routine page content            |
| `assessment_frontend_exam_routines`              | FrontendExamRoutine                | Front-end exam routine                 |
| `assessment_frontend_results`                    | FrontendResult                     | Front-end result publication           |
| `frontend_exam_results`            | FrontendExamResult              | Marketing block for results            |
| `assessment_online_exams`                  | OnlineExam                      | Digital exam                           |
| `assessment_online_exam_questions`         | OnlineExamQuestion              | Per-online-exam question               |
| `assessment_online_exam_question_assigns`  | QuestionAssignment              | OnlineExam ↔ QuestionBank link         |
| `assessment_online_exam_question_mu_options` | QuestionMuOption              | MC option                              |
| `assessment_online_exam_marks`             | OnlineExamMark                  | Per-student online exam mark           |
| `assessment_online_exam_student_answer_markings` | OnlineExamStudentAnswerMarking | Student answer + marking        |
| `assessment_student_take_online_exams`     | StudentTakeOnlineExam           | Student attempt at online exam         |
| `assessment_student_take_online_exam_questions` | StudentTakeOnlineExamQuestion | Per-question student response    |
| `assessment_question_groups`               | QuestionGroup                   | Question grouping                      |
| `assessment_question_levels`               | QuestionLevel                   | Difficulty level                       |
| `assessment_seat_plans`                    | SeatPlan                        | Section seat allocation                |
| `assessment_seat_plan_children`            | SeatPlanChild                   | Per-room allocation                    |
| `seat_plans`                       | SeatPlan (alt)                  | Per-student seat plan                  |
| `seat_plan_settings`               | SeatPlanSetting                 | Seat plan branding                     |
| `admit_cards`                      | AdmitCard                       | Admit card                             |
| `admit_card_settings`              | AdmitCardSetting                | Admit card branding                    |
| `teacher_evaluations`              | TeacherEvaluation               | Teacher rating                         |
| `teacher_evaluation_settings`      | TeacherEvaluationSetting        | Per-school evaluation settings         |
| `teacher_remarks`                  | TeacherRemark                   | Teacher narrative remark               |
| `assessment_exam_attendances`              | ExamAttendance                  | Exam-day attendance roll               |
| `assessment_exam_attendance_children`      | ExamAttendanceChild             | Per-student exam attendance            |
| `class_attendances`                | ClassAttendance (legacy summary)| Days opened/absent/present summary     |
| `assessment_question_banks`                | QuestionBank                    | Reusable question pool                 |

## Notes

- Every table includes `school_id` for multi-tenant isolation. The
  `school_id` is `NOT NULL DEFAULT 1` for the bootstrap school.
- Every table includes `created_at`, `updated_at`, `created_by`,
  `updated_by`, `active_status` (where applicable). These are
  managed by the engine's storage adapter.
- `academic_id` references `academic_academic_years` (the per-year scope).
- `record_id` references `student_records.id` and is the per-year
  enrollment handle.
- `class_attendances` and `teacher_evaluation_settings` are
  non-academic-year-scoped: they are per-school summaries.
- `seat_plans` is a per-student seat plan table (not the section
  plan in `assessment_seat_plans`); the engine may project the two
  representations as needed.
- Online exam question-option table
  (`assessment_online_exam_question_mu_options`) is keyed on
  `online_exam_question_id` (shorter name `on_ex_qu_id` in the
  schema).
