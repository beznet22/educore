# Academic Domain — Tables

The academic domain is backed by the following tables. Each table maps
to one or more aggregates; the `aggregate` column tells you which
aggregate owns the row.

| Table                                    | Aggregate                  | Notes                                  |
| ---------------------------------------- | -------------------------- | -------------------------------------- |
| `academic_academic_years`                      | AcademicYear               | One per school year                    | <!-- derive_skip -->
| `academic_sessions`                            | AcademicYear (legacy)      | Older alias; prefer `academic_academic_years`| <!-- derive_skip -->
| `academic_classes`                             | Class                      | Grade level                            | <!-- derive_skip -->
| `academic_sections`                            | Section                    | Division                               | <!-- derive_skip -->
| `academic_class_sections`                      | ClassSection               | Pair class+section+year                | <!-- derive_skip -->
| `academic_class_optional_subject`              | Class                      | Optional subject eligibility           | <!-- derive_skip -->
| `academic_class_rooms`                         | ClassRoom                  | Physical rooms                         | <!-- derive_skip -->
| `academic_class_times`                         | ClassTime                  | Period slots                           | <!-- derive_skip -->
| `academic_class_routines`                      | ClassRoutine               | Weekly routine entry                   | <!-- derive_skip -->
| `academic_class_routine_updates`               | ClassRoutineUpdate         | Per-period entries                     | <!-- derive_skip -->
| `hr_class_teachers`                      | ClassSectionTeacher        | Teacher assigned to a section          | <!-- derive_skip -->
| `hr_assign_class_teachers`               | AssignClassTeacher         | Higher-level assignment                | <!-- derive_skip -->
| `academic_subjects`                            | Subject                    | Subject catalog                        | <!-- derive_skip -->
| `library_subjects`                       | Subject (alt)              | Subject alias used by library          | <!-- derive_skip -->
| `academic_assign_subjects`                     | ClassSubject               | Subject assigned to a class            | <!-- derive_skip -->
| `academic_students`                            | Student                    | The student                            | <!-- derive_skip -->
| `academic_parents`                             | Guardian                   | Parent/guardian                        | <!-- derive_skip -->
| `academic_student_categories`                  | StudentCategory            | Categorization                         | <!-- derive_skip -->
| `academic_student_groups`                      | StudentGroup               | Non-academic groups                    | <!-- derive_skip -->
| `academic_student_documents`                   | StudentDocument            | Uploaded docs                          | <!-- derive_skip -->
| `academic_student_timelines`                   | StudentTimeline            | Timeline events                        | <!-- derive_skip -->
| `academic_student_homeworks`                   | StudentHomework            | Per-student homework                   | <!-- derive_skip -->
| `academic_upload_homework_contents`            | HomeworkSubmission file    | Per-student uploaded file              | <!-- derive_skip -->
| `academic_optional_subject_assigns`            | OptionalSubjectAssignment  | Optional subject picks                 | <!-- derive_skip -->
| `academic_student_promotions`                  | StudentPromotion           | Promotion history                      | <!-- derive_skip -->
| `student_records`                        | StudentRecord              | Enrollment per year                    | <!-- derive_skip -->
| `student_academic_histories`             | StudentHistory             | Free-text history entries              | <!-- derive_skip -->
| `student_bulk_temporaries`               | BulkImportJob              | Staging for bulk imports               | <!-- derive_skip -->
| `student_record_temporaries`             | BulkImportJob              | Staging for bulk promotions            | <!-- derive_skip -->
| `academic_student_excel_formats`               | (template)                 | Excel template                         |
| `academic_student_registration_fields`         | RegistrationField          | Custom registration fields             | <!-- derive_skip -->
| `academic_student_certificates`                | Certificate                | Certificate templates                  | <!-- derive_skip -->
| `academic_student_id_cards`                    | IdCard                     | ID card templates                      | <!-- derive_skip -->
| `academic_homeworks`                           | Homework                   | Homework assignments                   | <!-- derive_skip -->
| `academic_homework_students`                   | HomeworkSubmission         | Submissions & evaluation               | <!-- derive_skip -->
| `academic_lessons`                             | Lesson                     | Lesson list                            | <!-- derive_skip -->
| `academic_lesson_details`                      | LessonDetail               | Versioned lesson snapshots             | <!-- derive_skip -->
| `academic_lesson_topics`                       | LessonTopic                | Topics                                 | <!-- derive_skip -->
| `academic_lesson_topic_details`                | LessonTopicDetail          | Topic content                          | <!-- derive_skip -->
| `lesson_planners`                        | LessonPlan                 | Lesson plan entries                    | <!-- derive_skip -->
| `lesson_plan_topics`                     | LessonPlanTopic            | Sub-topics in a plan                   | <!-- derive_skip -->
| `academic_admission_queries`                   | AdmissionQuery             | Inquiry inbox                          | <!-- derive_skip -->
| `academic_admission_query_followups`           | AdmissionQueryFollowup     | Inquiry followups                      | <!-- derive_skip -->
| `academic_graduates`                              | GraduateRecord             | Historical graduates                   | <!-- derive_skip -->
| `student_ratings`                        | StudentRating              | Per-exam rating                        | <!-- derive_skip -->
| `cms_frontend_academic_calendars`               | FrontAcademicCalendar      | Public calendar publication            | <!-- derive_skip -->
| `cms_frontend_class_routines`                   | FrontClassRoutine          | Public routine publication             | <!-- derive_skip -->
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
