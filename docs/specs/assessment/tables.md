# Assessment Domain — Tables

The assessment domain is backed by the following tables. Each table
maps to one or more aggregates; the `aggregate` column tells you
which aggregate owns the row.

| Table                              | Aggregate                       | Notes                                  |
| ---------------------------------- | ------------------------------- | -------------------------------------- |
| `assessment_exam_types`                    | ExamType                        | Exam category catalog                  | <!-- derive_skip -->
| `assessment_exams`                         | Exam                            | Per-(class,section,subject) exam       | <!-- derive_skip -->
| `assessment_exam_setups`                   | ExamSetup                       | Section-level exam config              | <!-- derive_skip -->
| `assessment_exam_schedules`                | ExamSchedule                    | Calendar slot for an exam              | <!-- derive_skip -->
| `assessment_exam_schedule_subjects`        | ExamScheduleSubject             | Per-subject schedule entry             | <!-- derive_skip -->
| `assessment_exam_settings`                 | ExamSetting                     | School exam publication                | <!-- derive_skip -->
| `assessment_exam_signatures`               | ExamSignature                   | Signatures for report cards            | <!-- derive_skip -->
| `assessment_marks_registers`               | MarksRegister                   | Per-student exam marks                 | <!-- derive_skip -->
| `assessment_marks_register_children`       | MarksRegisterChild              | Per-subject marks row                  | <!-- derive_skip -->
| `assessment_mark_stores`                   | MarkStore                       | Consolidated stored marks              | <!-- derive_skip -->
| `assessment_result_stores`                 | ResultStore                     | Per-subject published result           | <!-- derive_skip -->
| `assessment_marks_grades`                  | MarksGrade                      | Grade boundary scale                   | <!-- derive_skip -->
| `assessment_temporary_meritlists`          | TemporaryMeritList              | Merit staging                          | <!-- derive_skip -->
| `exam_merit_positions`             | MeritPosition                   | Final merit position                   | <!-- derive_skip -->
| `assessment_exam_marks_registers`          | MarksRegister (alt)             | Alternate column-level table           | <!-- derive_skip -->
| `all_exam_wise_positions`          | AllExamWisePosition             | Cross-section exam position            | <!-- derive_skip -->
| `assessment_marks_send_sms`                | (notification trigger)          | Marks-SMS dispatch tracker             |
| `custom_result_settings`           | CustomResultSetting             | Custom result branding                 | <!-- derive_skip -->
| `assessment_custom_temporary_results`      | CustomTemporaryResult           | Custom result staging                  | <!-- derive_skip -->
| `exam_step_skips`                  | ExamStepSkip                    | Wizard-skip flag                       | <!-- derive_skip -->
| `assessment_class_exam_routine_pages`      | ExamRoutinePage                 | Public routine page content            | <!-- derive_skip -->
| `assessment_frontend_exam_routines`              | FrontendExamRoutine                | Front-end exam routine                 | <!-- derive_skip -->
| `assessment_frontend_results`                    | FrontendResult                     | Front-end result publication           | <!-- derive_skip -->
| `frontend_exam_results`            | FrontendExamResult              | Marketing block for results            | <!-- derive_skip -->
| `assessment_online_exams`                  | OnlineExam                      | Digital exam                           | <!-- derive_skip -->
| `assessment_online_exam_questions`         | OnlineExamQuestion              | Per-online-exam question               | <!-- derive_skip -->
| `assessment_online_exam_question_assigns`  | QuestionAssignment              | OnlineExam ↔ QuestionBank link         | <!-- derive_skip -->
| `assessment_online_exam_question_mu_options` | QuestionMuOption              | MC option                              | <!-- derive_skip -->
| `assessment_online_exam_marks`             | OnlineExamMark                  | Per-student online exam mark           | <!-- derive_skip -->
| `assessment_online_exam_student_answer_markings` | OnlineExamStudentAnswerMarking | Student answer + marking        | <!-- derive_skip -->
| `assessment_student_take_online_exams`     | StudentTakeOnlineExam           | Student attempt at online exam         | <!-- derive_skip -->
| `assessment_student_take_online_exam_questions` | StudentTakeOnlineExamQuestion | Per-question student response    | <!-- derive_skip -->
| `assessment_question_groups`               | QuestionGroup                   | Question grouping                      | <!-- derive_skip -->
| `assessment_question_levels`               | QuestionLevel                   | Difficulty level                       | <!-- derive_skip -->
| `assessment_seat_plans`                    | SeatPlan                        | Section seat allocation                | <!-- derive_skip -->
| `assessment_seat_plan_children`            | SeatPlanChild                   | Per-room allocation                    | <!-- derive_skip -->
| `seat_plans`                       | SeatPlan (alt)                  | Per-student seat plan                  | <!-- derive_skip -->
| `seat_plan_settings`               | SeatPlanSetting                 | Seat plan branding                     | <!-- derive_skip -->
| `admit_cards`                      | AdmitCard                       | Admit card                             | <!-- derive_skip -->
| `admit_card_settings`              | AdmitCardSetting                | Admit card branding                    | <!-- derive_skip -->
| `teacher_evaluations`              | TeacherEvaluation               | Teacher rating                         | <!-- derive_skip -->
| `teacher_evaluation_settings`      | TeacherEvaluationSetting        | Per-school evaluation settings         | <!-- derive_skip -->
| `teacher_remarks`                  | TeacherRemark                   | Teacher narrative remark               | <!-- derive_skip -->
| `assessment_exam_attendances`              | ExamAttendance                  | Exam-day attendance roll               | <!-- derive_skip -->
| `assessment_exam_attendance_children`      | ExamAttendanceChild             | Per-student exam attendance            | <!-- derive_skip -->
| `class_attendances`                | ClassAttendance (legacy summary)| Days opened/absent/present summary     | <!-- derive_skip -->
| `assessment_question_banks`                | QuestionBank                    | Reusable question pool                 | <!-- derive_skip -->

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
