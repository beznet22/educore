# Academic Domain — Tables

The academic domain is backed by the following tables. Each table maps
to one or more aggregates; the `aggregate` column tells you which
aggregate owns the row.

| Table                                    | Aggregate                  | Notes                                  |
| ---------------------------------------- | -------------------------- | -------------------------------------- |
| `academic_academic_years`                      | AcademicYear               | One per school year                    |
| `academic_sessions`                            | AcademicYear (legacy)      | Older alias; prefer `academic_academic_years`|
| `academic_classes`                             | Class                      | Grade level                            |
| `academic_sections`                            | Section                    | Division                               |
| `academic_class_sections`                      | ClassSection               | Pair class+section+year                |
| `academic_class_optional_subject`              | Class                      | Optional subject eligibility           |
| `academic_class_rooms`                         | ClassRoom                  | Physical rooms                         |
| `academic_class_times`                         | ClassTime                  | Period slots                           |
| `academic_class_routines`                      | ClassRoutine               | Weekly routine entry                   |
| `academic_class_routine_updates`               | ClassRoutineUpdate         | Per-period entries                     |
| `hr_class_teachers`                      | ClassSectionTeacher        | Teacher assigned to a section          |
| `hr_assign_class_teachers`               | AssignClassTeacher         | Higher-level assignment                |
| `academic_subjects`                            | Subject                    | Subject catalog                        |
| `library_subjects`                       | Subject (alt)              | Subject alias used by library          |
| `academic_assign_subjects`                     | ClassSubject               | Subject assigned to a class            |
| `academic_students`                            | Student                    | The student                            |
| `academic_parents`                             | Guardian                   | Parent/guardian                        |
| `academic_student_categories`                  | StudentCategory            | Categorization                         |
| `academic_student_groups`                      | StudentGroup               | Non-academic groups                    |
| `academic_student_documents`                   | StudentDocument            | Uploaded docs                          |
| `academic_student_timelines`                   | StudentTimeline            | Timeline events                        |
| `academic_student_homeworks`                   | StudentHomework            | Per-student homework                   |
| `academic_upload_homework_contents`            | HomeworkSubmission file    | Per-student uploaded file              |
| `academic_optional_subject_assigns`            | OptionalSubjectAssignment  | Optional subject picks                 |
| `academic_student_promotions`                  | StudentPromotion           | Promotion history                      |
| `student_records`                        | StudentRecord              | Enrollment per year                    |
| `student_academic_histories`             | StudentHistory             | Free-text history entries              |
| `student_bulk_temporaries`               | BulkImportJob              | Staging for bulk imports               |
| `student_record_temporaries`             | BulkImportJob              | Staging for bulk promotions            |
| `academic_student_excel_formats`               | (template)                 | Excel template                         |
| `academic_student_registration_fields`         | RegistrationField          | Custom registration fields             |
| `academic_student_certificates`                | Certificate                | Certificate templates                  |
| `academic_student_id_cards`                    | IdCard                     | ID card templates                      |
| `academic_homeworks`                           | Homework                   | Homework assignments                   |
| `academic_homework_students`                   | HomeworkSubmission         | Submissions & evaluation               |
| `academic_lessons`                             | Lesson                     | Lesson list                            |
| `academic_lesson_details`                      | LessonDetail               | Versioned lesson snapshots             |
| `academic_lesson_topics`                       | LessonTopic                | Topics                                 |
| `academic_lesson_topic_details`                | LessonTopicDetail          | Topic content                          |
| `lesson_planners`                        | LessonPlan                 | Lesson plan entries                    |
| `lesson_plan_topics`                     | LessonPlanTopic            | Sub-topics in a plan                   |
| `academic_admission_queries`                   | AdmissionQuery             | Inquiry inbox                          |
| `academic_admission_query_followups`           | AdmissionQueryFollowup     | Inquiry followups                      |
| `academic_graduates`                              | GraduateRecord             | Historical graduates                   |
| `student_ratings`                        | StudentRating              | Per-exam rating                        |
| `cms_frontend_academic_calendars`               | FrontAcademicCalendar      | Public calendar publication            |
| `cms_frontend_class_routines`                   | FrontClassRoutine          | Public routine publication             |
| `cms_class_publish_sentinel`                          | (sentinel)                 | Internal marker                        |
| `platform_module_student_parent_infos`      | (cms)                      | Student/parent UI menu                 |

## Notes

- Every table includes `school_id` for multi-tenant isolation. The
  `school_id` is `NOT NULL DEFAULT 1` for the bootstrap school.
- Every table includes `created_at`, `updated_at`, `created_by`,
  `updated_by`, `active_status` (where applicable). These are managed
  by the engine's storage adapter.
- `academic_id` references `academic_academic_years` (the per-year scope).
  Some legacy columns use `session_id` for the same purpose.
- `record_id` references `student_records.id` and is the per-year
  enrollment handle.
