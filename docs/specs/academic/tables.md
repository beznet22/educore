# Academic Domain — Tables

The academic domain is backed by the following tables. Each table maps
to one or more aggregates; the `aggregate` column tells you which
aggregate owns the row.

| Table                                    | Aggregate                  | Notes                                  |
| ---------------------------------------- | -------------------------- | -------------------------------------- |
| `sm_academic_years`                      | AcademicYear               | One per school year                    |
| `sm_sessions`                            | AcademicYear (legacy)      | Older alias; prefer `sm_academic_years`|
| `sm_classes`                             | Class                      | Grade level                            |
| `sm_sections`                            | Section                    | Division                               |
| `sm_class_sections`                      | ClassSection               | Pair class+section+year                |
| `sm_class_optional_subject`              | Class                      | Optional subject eligibility           |
| `sm_class_rooms`                         | ClassRoom                  | Physical rooms                         |
| `sm_class_times`                         | ClassTime                  | Period slots                           |
| `sm_class_routines`                      | ClassRoutine               | Weekly routine entry                   |
| `sm_class_routine_updates`               | ClassRoutineUpdate         | Per-period entries                     |
| `sm_class_teachers`                      | ClassSectionTeacher        | Teacher assigned to a section          |
| `sm_assign_class_teachers`               | AssignClassTeacher         | Higher-level assignment                |
| `sm_subjects`                            | Subject                    | Subject catalog                        |
| `library_subjects`                       | Subject (alt)              | Subject alias used by library          |
| `sm_assign_subjects`                     | ClassSubject               | Subject assigned to a class            |
| `sm_students`                            | Student                    | The student                            |
| `sm_parents`                             | Guardian                   | Parent/guardian                        |
| `sm_student_categories`                  | StudentCategory            | Categorization                         |
| `sm_student_groups`                      | StudentGroup               | Non-academic groups                    |
| `sm_student_documents`                   | StudentDocument            | Uploaded docs                          |
| `sm_student_timelines`                   | StudentTimeline            | Timeline events                        |
| `sm_student_homeworks`                   | StudentHomework            | Per-student homework                   |
| `sm_upload_homework_contents`            | HomeworkSubmission file    | Per-student uploaded file              |
| `sm_optional_subject_assigns`            | OptionalSubjectAssignment  | Optional subject picks                 |
| `sm_student_promotions`                  | StudentPromotion           | Promotion history                      |
| `student_records`                        | StudentRecord              | Enrollment per year                    |
| `student_academic_histories`             | StudentHistory             | Free-text history entries              |
| `student_bulk_temporaries`               | BulkImportJob              | Staging for bulk imports               |
| `student_record_temporaries`             | BulkImportJob              | Staging for bulk promotions            |
| `sm_student_excel_formats`               | (template)                 | Excel template                         |
| `sm_student_registration_fields`         | RegistrationField          | Custom registration fields             |
| `sm_student_certificates`                | Certificate                | Certificate templates                  |
| `sm_student_id_cards`                    | IdCard                     | ID card templates                      |
| `sm_homeworks`                           | Homework                   | Homework assignments                   |
| `sm_homework_students`                   | HomeworkSubmission         | Submissions & evaluation               |
| `sm_lessons`                             | Lesson                     | Lesson list                            |
| `sm_lesson_details`                      | LessonDetail               | Versioned lesson snapshots             |
| `sm_lesson_topics`                       | LessonTopic                | Topics                                 |
| `sm_lesson_topic_details`                | LessonTopicDetail          | Topic content                          |
| `lesson_planners`                        | LessonPlan                 | Lesson plan entries                    |
| `lesson_plan_topics`                     | LessonPlanTopic            | Sub-topics in a plan                   |
| `sm_admission_queries`                   | AdmissionQuery             | Inquiry inbox                          |
| `sm_admission_query_followups`           | AdmissionQueryFollowup     | Inquiry followups                      |
| `graduates`                              | GraduateRecord             | Historical graduates                   |
| `student_ratings`                        | StudentRating              | Per-exam rating                        |
| `front_academic_calendars`               | FrontAcademicCalendar      | Public calendar publication            |
| `front_class_routines`                   | FrontClassRoutine          | Public routine publication             |
| `check_classes`                          | (sentinel)                 | Internal marker                        |
| `infix_module_student_parent_infos`      | (cms)                      | Student/parent UI menu                 |

## Notes

- Every table includes `school_id` for multi-tenant isolation. The
  `school_id` is `NOT NULL DEFAULT 1` for the bootstrap school.
- Every table includes `created_at`, `updated_at`, `created_by`,
  `updated_by`, `active_status` (where applicable). These are managed
  by the engine's storage adapter.
- `academic_id` references `sm_academic_years` (the per-year scope).
  Some legacy columns use `session_id` for the same purpose.
- `record_id` references `student_records.id` and is the per-year
  enrollment handle.
