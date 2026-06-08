# Assessment Domain — Tables

The assessment domain is backed by the following tables. Each table
maps to one or more aggregates; the `aggregate` column tells you
which aggregate owns the row.

| Table                              | Aggregate                       | Notes                                  |
| ---------------------------------- | ------------------------------- | -------------------------------------- |
| `sm_exam_types`                    | ExamType                        | Exam category catalog                  |
| `sm_exams`                         | Exam                            | Per-(class,section,subject) exam       |
| `sm_exam_setups`                   | ExamSetup                       | Section-level exam config              |
| `sm_exam_schedules`                | ExamSchedule                    | Calendar slot for an exam              |
| `sm_exam_schedule_subjects`        | ExamScheduleSubject             | Per-subject schedule entry             |
| `sm_exam_settings`                 | ExamSetting                     | School exam publication                |
| `sm_exam_signatures`               | ExamSignature                   | Signatures for report cards            |
| `sm_marks_registers`               | MarksRegister                   | Per-student exam marks                 |
| `sm_marks_register_children`       | MarksRegisterChild              | Per-subject marks row                  |
| `sm_mark_stores`                   | MarkStore                       | Consolidated stored marks              |
| `sm_result_stores`                 | ResultStore                     | Per-subject published result           |
| `sm_marks_grades`                  | MarksGrade                      | Grade boundary scale                   |
| `sm_temporary_meritlists`          | TemporaryMeritList              | Merit staging                          |
| `exam_merit_positions`             | MeritPosition                   | Final merit position                   |
| `sm_exam_marks_registers`          | MarksRegister (alt)             | Alternate column-level table           |
| `all_exam_wise_positions`          | AllExamWisePosition             | Cross-section exam position            |
| `sm_marks_send_sms`                | (notification trigger)          | Marks-SMS dispatch tracker             |
| `custom_result_settings`           | CustomResultSetting             | Custom result branding                 |
| `sm_custom_temporary_results`      | CustomTemporaryResult           | Custom result staging                  |
| `exam_step_skips`                  | ExamStepSkip                    | Wizard-skip flag                       |
| `sm_class_exam_routine_pages`      | ExamRoutinePage                 | Public routine page content            |
| `front_exam_routines`              | FrontExamRoutine                | Front-end exam routine                 |
| `front_results`                    | FrontResult                     | Front-end result publication           |
| `frontend_exam_results`            | FrontendExamResult              | Marketing block for results            |
| `sm_online_exams`                  | OnlineExam                      | Digital exam                           |
| `sm_online_exam_questions`         | OnlineExamQuestion              | Per-online-exam question               |
| `sm_online_exam_question_assigns`  | QuestionAssignment              | OnlineExam ↔ QuestionBank link         |
| `sm_online_exam_question_mu_options` | QuestionMuOption              | MC option                              |
| `sm_online_exam_marks`             | OnlineExamMark                  | Per-student online exam mark           |
| `sm_online_exam_student_answer_markings` | OnlineExamStudentAnswerMarking | Student answer + marking        |
| `sm_student_take_online_exams`     | StudentTakeOnlineExam           | Student attempt at online exam         |
| `sm_student_take_online_exam_questions` | StudentTakeOnlineExamQuestion | Per-question student response    |
| `sm_question_groups`               | QuestionGroup                   | Question grouping                      |
| `sm_question_levels`               | QuestionLevel                   | Difficulty level                       |
| `sm_seat_plans`                    | SeatPlan                        | Section seat allocation                |
| `sm_seat_plan_children`            | SeatPlanChild                   | Per-room allocation                    |
| `seat_plans`                       | SeatPlan (alt)                  | Per-student seat plan                  |
| `seat_plan_settings`               | SeatPlanSetting                 | Seat plan branding                     |
| `admit_cards`                      | AdmitCard                       | Admit card                             |
| `admit_card_settings`              | AdmitCardSetting                | Admit card branding                    |
| `teacher_evaluations`              | TeacherEvaluation               | Teacher rating                         |
| `teacher_evaluation_settings`      | TeacherEvaluationSetting        | Per-school evaluation settings         |
| `teacher_remarks`                  | TeacherRemark                   | Teacher narrative remark               |
| `sm_exam_attendances`              | ExamAttendance                  | Exam-day attendance roll               |
| `sm_exam_attendance_children`      | ExamAttendanceChild             | Per-student exam attendance            |
| `class_attendances`                | ClassAttendance (legacy summary)| Days opened/absent/present summary     |
| `sm_question_banks`                | QuestionBank                    | Reusable question pool                 |

## Notes

- Every table includes `school_id` for multi-tenant isolation. The
  `school_id` is `NOT NULL DEFAULT 1` for the bootstrap school.
- Every table includes `created_at`, `updated_at`, `created_by`,
  `updated_by`, `active_status` (where applicable). These are
  managed by the engine's storage adapter.
- `academic_id` references `sm_academic_years` (the per-year scope).
- `record_id` references `student_records.id` and is the per-year
  enrollment handle.
- `class_attendances` and `teacher_evaluation_settings` are
  non-academic-year-scoped: they are per-school summaries.
- `seat_plans` is a per-student seat plan table (not the section
  plan in `sm_seat_plans`); the engine may project the two
  representations as needed.
- Online exam question-option table
  (`sm_online_exam_question_mu_options`) is keyed on
  `online_exam_question_id` (shorter name `on_ex_qu_id` in the
  schema).
