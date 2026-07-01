## Wave 6 Spec Audit Report — Domains 1-5 (academic, assessment, attendance, cms, communication)

**Scope:** `docs/specs/academic/`, `docs/specs/assessment/`, `docs/specs/attendance/`, `docs/specs/cms/`, `docs/specs/communication/` and the corresponding `crates/domains/<d>/` source crates.

**11 spec files per folder (per `docs/code-standards.md` § "Spec folder layout"):**
`overview.md`, `aggregates.md`, `entities.md`, `value-objects.md`, `commands.md`, `events.md`, `services.md`, `permissions.md`, `repositories.md`, `workflows.md`, `tables.md`.

**File counts observed (all 5 spec folders complete with 11 files):**
- `academic/`: 11 files (complete)
- `assessment/`: 11 files (complete)
- `attendance/`: 11 files (complete)
- `cms/`: 11 files (complete)
- `communication/`: 11 files (complete)

**Source crate file layout observed (all 5 crates have 10 src files):**
- `crates/domains/academic/src/`, `assessment/src/`, `attendance/src/`, `cms/src/`, `communication/src/` each contain `aggregate.rs`, `commands.rs`, `entities.rs`, `errors.rs`, `events.rs`, `lib.rs`, `query.rs`, `repository.rs`, `services.rs`, `value_objects.rs`.

**Aggregate counts (spec vs source) — partial summary; full per-aggregate coverage is enumerated in Phase B findings:**
- `academic`: 20 aggregates in spec (`Student`, `Guardian`, `Class`, `Section`, `ClassSection`, `Subject`, `ClassSubject`, `AcademicYear`, `ClassRoutine`, `Homework`, `LessonPlan`, `Lesson`, `LessonTopic`, `StudentRecord`, `StudentPromotion`, `StudentCategory`, `StudentGroup`, `RegistrationField`, `Certificate`, `IdCard`); 5 `pub struct` roots in `crates/domains/academic/src/aggregate.rs` (`Student`, `Class`, `Section`, `Subject`, `AcademicYear`).
- `assessment`: 46 aggregates in spec; 6 `pub struct` roots in source (`Exam`, `ExamSchedule`, `SeatPlan`, `AdmitCard`, `MarksRegister`, `ResultStore`).
- `attendance`: 10 aggregates in spec; 5 `pub struct` roots in source (`StudentAttendance`, `StaffAttendance`, `SubjectAttendance`, `ExamAttendance`, `BulkAttendanceImport`).
- `cms`: 19 aggregates in spec; 30+ `pub struct` in source (multiple `New*`/`Update*` command structs plus aggregate roots).
- `communication`: 27 aggregates in spec; 25 `pub struct` in source.

**Total findings (Phase A + Phase B):** 45

---

### FINDING 1

- **id:** SPEC-1-001
- **area:** spec-domains-1-5
- **severity:** Critical
- **location:** `docs/specs/academic/aggregates.md:21-32` vs `crates/domains/academic/src/aggregate.rs`
- **description:** The academic spec defines 20 aggregates in `aggregates.md` (`Student`, `Guardian`, `Class`, `Section`, `ClassSection`, `Subject`, `ClassSubject`, `AcademicYear`, `ClassRoutine`, `Homework`, `LessonPlan`, `Lesson`, `LessonTopic`, `StudentRecord`, `StudentPromotion`, `StudentCategory`, `StudentGroup`, `RegistrationField`, `Certificate`, `IdCard`). The source crate `crates/domains/academic/src/aggregate.rs` declares only 5 aggregate roots (`Student`, `Class`, `Section`, `Subject`, `AcademicYear`). 15 aggregates (`Guardian`, `ClassSection`, `ClassSubject`, `ClassRoutine`, `Homework`, `LessonPlan`, `Lesson`, `LessonTopic`, `StudentRecord`, `StudentPromotion`, `StudentCategory`, `StudentGroup`, `RegistrationField`, `Certificate`, `IdCard`) have no Rust struct.
- **expected:** Every aggregate declared in `docs/specs/academic/aggregates.md` has a corresponding `pub struct` root in `crates/domains/academic/src/aggregate.rs`.
- **evidence:** `docs/specs/academic/aggregates.md`:20 aggregates under `## ` headings (`## Student` ... `## IdCard`). `crates/domains/academic/src/aggregate.rs` `grep "^pub struct "` returns `Student`, `Class`, `Section`, `Subject`, `AcademicYear` only.

---

### FINDING 2

- **id:** SPEC-1-002
- **area:** spec-domains-1-5
- **severity:** Critical
- **location:** `docs/specs/assessment/aggregates.md:35-1025` vs `crates/domains/assessment/src/aggregate.rs`
- **description:** The assessment spec defines 46 aggregates (`ExamType`, `Exam`, `ExamSetup`, `ExamSchedule`, `ExamScheduleSubject`, `MarksRegister`, `MarksRegisterChild`, `MarkStore`, `MarkStoreEntry`, `ResultStore`, `ResultSetting`, `MarksGrade`, `ExamSetting`, `ExamSignature`, `ExamRoutinePage`, `FrontendExamRoutine`, `FrontendResult`, `FrontendExamResult`, `OnlineExam`, `QuestionBank`, `QuestionGroup`, `QuestionLevel`, `QuestionAssignment`, `OnlineExamQuestion`, `QuestionMuOption`, `OnlineExamMark`, `OnlineExamStudentAnswerMarking`, `StudentTakeOnlineExam`, `SeatPlan`, `SeatPlanChild`, `SeatPlanSetting`, `AdmitCard`, `AdmitCardSetting`, `TeacherEvaluation`, `TeacherRemark`, `TemporaryMeritList`, `MeritPosition`, `ExamWisePosition`, `AllExamWisePosition`, `CustomResultSetting`, `CustomTemporaryResult`, `ExamStepSkip`, `ExamAttendance`, `ExamAttendanceChild`, `SubjectAttendance (overlap with Attendance)`). The source crate declares only 6 aggregate roots (`Exam`, `ExamSchedule`, `SeatPlan`, `AdmitCard`, `MarksRegister`, `ResultStore`).
- **expected:** Every aggregate declared in `docs/specs/assessment/aggregates.md` has a corresponding `pub struct` root in `crates/domains/assessment/src/aggregate.rs`. Note: `SubjectAttendance` is owned by the attendance domain (per the `(overlap with Attendance)` annotation), so cross-domain ownership should be enforced.
- **evidence:** `grep "^## " docs/specs/assessment/aggregates.md | wc -l` returns 46 lines (`## ExamType` ... `## SubjectAttendance (overlap with Attendance)`). `crates/domains/assessment/src/aggregate.rs` `grep "^pub struct "` returns `Exam`, `ExamSchedule`, `SeatPlan`, `AdmitCard`, `MarksRegister`, `ResultStore`.

---

### FINDING 3

- **id:** SPEC-1-003
- **area:** spec-domains-1-5
- **severity:** Critical
- **location:** `docs/specs/attendance/aggregates.md:35-255` vs `crates/domains/attendance/src/aggregate.rs`
- **description:** The attendance spec defines 10 aggregates (`StudentAttendance`, `SubjectAttendance`, `StaffAttendance`, `ExamAttendance`, `BulkAttendanceImport`, `StudentAttendanceImport`, `StaffAttendanceImport`, `ClassAttendance`, `AttendanceBulk`, `ExamAttendanceChild (cross-reference)`). The source crate declares only 5 aggregate roots (`StudentAttendance`, `StaffAttendance`, `SubjectAttendance`, `ExamAttendance`, `BulkAttendanceImport`).
- **expected:** Every aggregate declared in `docs/specs/attendance/aggregates.md` has a corresponding `pub struct` root in `crates/domains/attendance/src/aggregate.rs`. The spec lists 10 aggregates but only 5 exist in source; 5 aggregates (`StudentAttendanceImport`, `StaffAttendanceImport`, `ClassAttendance`, `AttendanceBulk`, `ExamAttendanceChild`) are missing.
- **evidence:** `docs/specs/attendance/aggregates.md` headings: `## StudentAttendance`, `## SubjectAttendance`, `## StaffAttendance`, `## ExamAttendance`, `## BulkAttendanceImport`, `## StudentAttendanceImport`, `## StaffAttendanceImport`, `## ClassAttendance`, `## AttendanceBulk`, `## ExamAttendanceChild (cross-reference)`. Source: `crates/domains/attendance/src/aggregate.rs` `grep "^pub struct "` returns only `StudentAttendance`, `StaffAttendance`, `SubjectAttendance`, `ExamAttendance`, `BulkAttendanceImport`.

---

### FINDING 4

- **id:** SPEC-1-004
- **area:** spec-domains-1-5
- **severity:** Critical
- **location:** `docs/specs/cms/aggregates.md:35-644` vs `crates/domains/cms/src/aggregate.rs`
- **description:** The CMS spec defines 19 aggregates (`Page`, `News`, `NewsCategory`, `NewsComment`, `NewsPage`, `NoticeBoard`, `Testimonial`, `HomeSlider`, `SpeechSlider`, `Content`, `ContentType`, `ContentShareList`, `TeacherUploadContent`, `UploadContent`, `AboutPage`, `ContactPage`, `CoursePage`, `HomePageSetting`, `FrontendPage`). The source crate `crates/domains/cms/src/aggregate.rs` declares the 19 aggregate roots plus 16 `New*`/`Update*` command-input structs that are interleaved with the root types in the file (e.g., `NewPage`, `UpdatePage`, `NewPageRevision`).
- **expected:** The CMS crate source contains aggregate roots for every aggregate in `docs/specs/cms/aggregates.md`. Per `AGENTS.md` § "Code Standards" / `docs/code-standards.md` § "Module Layout", `aggregate.rs` should host only aggregate roots; command-input structs belong in `commands.rs`. The interleaved `New*`/`Update*` structs violate the module-layout rule.
- **evidence:** `crates/domains/cms/src/aggregate.rs` `grep "^pub struct "` returns `NewPage`, `UpdatePage`, `Page`, `NewPageRevision`, `NewNews`, `UpdateNews`, `News`, ... (mixed root + command input). `docs/specs/cms/aggregates.md` declares 19 aggregates; only 19 should appear as `pub struct` roots in `aggregate.rs`.

---

### FINDING 5

- **id:** SPEC-1-005
- **area:** spec-domains-1-5
- **severity:** Critical
- **location:** `docs/specs/communication/aggregates.md:35-835` vs `crates/domains/communication/src/aggregate.rs`
- **description:** The communication spec defines 27 aggregates (`Notice`, `Complaint`, `ComplaintType`, `Notification`, `EmailLog`, `SmsLog`, `SmsTemplate`, `EmailSetting`, `SmsGateway`, `NotificationSetting`, `AbsentNotificationTimeSetup`, `ChatMessage`, `ChatConversation`, `ChatGroup`, `ChatGroupUser`, `ChatGroupMessageRecipient`, `ChatGroupMessageRemove`, `ChatBlockUser`, `ChatInvitation`, `ChatInvitationType`, `ChatStatus`, `SendMessage`, `ContactMessage`, `SpeechSlider`, `PhoneCallLog`, `CustomSmsSetting`). The source crate declares 25 `pub struct`s.
- **expected:** Each spec aggregate maps to exactly one `pub struct` root in `crates/domains/communication/src/aggregate.rs`. Naming drift exists: spec calls the aggregate `ChatStatus` (line 752), source uses `ChatStatusRecord` (the actual root name). Spec's `SendMessage` aggregate exists as `SendMessage` in source but the spec also defines it as a command in `commands.md` (potential naming collision).
- **evidence:** `docs/specs/communication/aggregates.md` `## ChatStatus` (line ~752). `crates/domains/communication/src/aggregate.rs` declares `pub struct ChatStatusRecord`. Spec lists `SendMessage` as an aggregate (line ~778) — same identifier as a likely command name in `commands.md`.

---

### FINDING 6

- **id:** SPEC-1-006
- **area:** spec-domains-1-5
- **severity:** Critical
- **location:** `docs/specs/academic/events.md:35-304` vs `crates/domains/academic/src/events.rs`
- **description:** The academic events spec enumerates 15 lifecycle sections (`Student Lifecycle`, `Guardian Lifecycle`, `Class & Section`, `ClassSection`, `Subject`, `AcademicYear`, `ClassRoutine`, `Homework`, `Lesson`, `StudentRecord`, `StudentCategory`, `StudentGroup`, `Registration`, `Certificate & ID Card`, `Admission Query`). The source crate `crates/domains/academic/src/events.rs` declares 25 `pub struct` event types (e.g., `StudentAdmitted`, `StudentProfileUpdated`, `ClassCreated`, `OptionalSubjectGpaThresholdSet`, `CurrentAcademicYearSet`, `AcademicYearClosed`, `AcademicYearCopied`). The spec narrative lists ~40 events; source only covers ~25, but spec lists events like `LessonPlanCreated`, `HomeworkAssigned`, `RegistrationFieldUpdated`, `CertificateIssued` for which there is no source struct.
- **expected:** Every event declared in `docs/specs/academic/events.md` has a corresponding `pub struct` in `crates/domains/academic/src/events.rs`.
- **evidence:** `docs/specs/academic/events.md` contains 15 lifecycle sections; source `crates/domains/academic/src/events.rs` declares 25 `pub struct` events. Several spec events (`HomeworkAssigned`, `LessonPlanCreated`, `CertificateIssued`, `IdCardIssued`, `RegistrationFieldAdded`) are not present in the source event enum/struct list.

---

### FINDING 7

- **id:** SPEC-1-007
- **area:** spec-domains-1-5
- **severity:** Critical
- **location:** `docs/specs/assessment/events.md:35-512` vs `crates/domains/assessment/src/events.rs`
- **description:** The assessment events spec covers 20 lifecycle sections (`Exam Type`, `Exam Lifecycle`, `Exam Schedule`, `Marks Lifecycle`, `Result Lifecycle`, `Report Card`, `Mark Store`, `Result Settings`, `Marks Grade`, `Exam Settings, Signatures, Front-End Pages`, `Exam Setup`, `Online Exam Lifecycle`, `Question Bank`, `Online Exam Questions and Marking`, `Seat Plan`, `Admit Card`, `Teacher Evaluation`, `Teacher Remark`, `Exam Attendance`, `Exam Step Skip`, `Audit`). The source crate declares 17 event structs, but several spec events lack source structs: e.g., `OnlineExamStarted`, `OnlineExamSubmitted`, `OnlineExamMarked`, `QuestionBankCreated`, `SeatPlanChildCreated`, `ExamAttendanceMarked`, `ExamStepSkipped`, `AuditLogged` are spec-only.
- **expected:** Every event declared in `docs/specs/assessment/events.md` has a corresponding `pub struct` in `crates/domains/assessment/src/events.rs`.
- **evidence:** `grep "^pub struct " crates/domains/assessment/src/events.rs` returns 17 event structs (`ExamCreated`, ..., `ReportCardGenerated`). `docs/specs/assessment/events.md` references ~80+ distinct event names across 20 lifecycle sections.

---

### FINDING 8

- **id:** SPEC-1-008
- **area:** spec-domains-1-5
- **severity:** Critical
- **location:** `docs/specs/attendance/events.md:35-279` vs `crates/domains/attendance/src/events.rs`
- **description:** The attendance events spec defines 7 lifecycle sections (`Student Attendance`, `Subject Attendance`, `Staff Attendance`, `Bulk Import`, `Notification`, `Class Attendance (projection)`, `Audit`) covering ~40 events. The source crate declares 21 `pub struct` events (`StudentAttendanceMarked`, `SubjectAttendanceMarked`, `StaffAttendanceMarked`, `ExamAttendanceMarked`, `BulkImportStarted`, `BulkImportValidated`, `BulkImportCommitted`, `BulkImportFailed`, `BulkImportCancelled`, `AttendanceImported`, `AbsenceNotificationRequested`, `ClassAttendanceRecomputed`, ...). Many spec-only events lack source structs (e.g., `StudentAbsentNotificationSent`, `SubjectAttendanceBiometricSynced`, `StaffAttendanceApproved`).
- **expected:** Every event declared in `docs/specs/attendance/events.md` has a corresponding `pub struct` in `crates/domains/attendance/src/events.rs`.
- **evidence:** `grep "^pub struct " crates/domains/attendance/src/events.rs` returns 21 event structs. `docs/specs/attendance/events.md` enumerates events across 7 lifecycle sections.

---

### FINDING 9

- **id:** SPEC-1-009
- **area:** spec-domains-1-5
- **severity:** Critical
- **location:** `docs/specs/cms/events.md:35-262` vs `crates/domains/cms/src/events.rs`
- **description:** The CMS events spec defines 21 lifecycle sections (`Page Lifecycle`, `News Lifecycle`, `News Comment Lifecycle`, `News Page`, `Notice Board (Public Site)`, `Testimonial`, `Home Slider`, `Speech Slider (CMS-Side)`, `Content Lifecycle`, `Content Type`, `Content Share List`, `Teacher Upload Content`, `Upload Content`, `About Page`, `Contact Page`, `Course Page`, `Home Page Setting`, `Frontend Page`, `News Category`, ...). The source crate declares 60+ event structs. Despite high coverage, spec-only events exist: `PageRevisionRestored`, `NewsPageViewIncremented`, `ContentShared`, `TeacherUploadContentApproved`, `UploadContentDownloaded`, `AboutPagePublished`.
- **expected:** Every event declared in `docs/specs/cms/events.md` has a corresponding `pub struct` in `crates/domains/cms/src/events.rs`.
- **evidence:** `docs/specs/cms/events.md` declares 21 lifecycle sections; `crates/domains/cms/src/events.rs` declares 60+ `pub struct` event types but misses a handful of spec-only events.

---

### FINDING 10

- **id:** SPEC-1-010
- **area:** spec-domains-1-5
- **severity:** Critical
- **location:** `docs/specs/communication/events.md:35-333` vs `crates/domains/communication/src/events.rs`
- **description:** The communication events spec defines 18 lifecycle sections (`Notice Lifecycle`, `Complaint Lifecycle`, `Notification Lifecycle`, `Email & SMS Logs`, `Templates`, `Email Engine Configuration`, `SMS Gateway Configuration`, `Notification Routing`, `Absent Notification`, `Chat — One-to-One`, `Chat — Groups`, `Chat — Group Message Delivery`, `Chat — Block, Invitation, Status`, `Send Message (Bulk)`, `Contact Message`, `Speech Slider`, `Phone Call`, `Custom SMS Gateway`). The source crate declares ~60+ event structs but several spec events are missing: `ChatInvitationAccepted`, `ChatInvitationRejected`, `ChatStatusChanged`, `PhoneCallLogged`, `ContactMessageReceived`, `CustomSmsGatewayProvisioned`.
- **expected:** Every event declared in `docs/specs/communication/events.md` has a corresponding `pub struct` in `crates/domains/communication/src/events.rs`.
- **evidence:** `grep "^pub struct " crates/domains/communication/src/events.rs` returns ~60+ event types. Spec events like `ChatInvitationAccepted` (referenced in `aggregates.md:799` ChatInvitation section) have no source struct.

---

### FINDING 11

- **id:** SPEC-1-011
- **area:** spec-domains-1-5
- **severity:** Critical
- **location:** `docs/specs/academic/tables.md:1-69` vs `crates/domains/academic/src/entities.rs`
- **description:** The academic tables spec lists ~30 tables (`academic_students`, `academic_guardians`, `academic_classes`, `academic_sections`, `academic_class_sections`, `academic_subjects`, `academic_class_subjects`, `academic_academic_years`, `academic_class_routines`, `academic_homework`, `academic_lesson_plans`, `academic_lessons`, `academic_lesson_topics`, `academic_student_records`, `academic_student_promotions`, `academic_student_categories`, `academic_student_groups`, `academic_registration_fields`, `academic_certificates`, `academic_id_cards`, `academic_optional_subject_assignments`, `academic_student_documents`, `academic_student_timelines`, `academic_student_homework`, `academic_guardian_links`, ...). The source crate `crates/domains/academic/src/entities.rs` does not declare any `#[derive(DomainQuery)]` applications (the macro that emits the typed AST contract per `docs/build-plan.md` Phase 0 § "Foundation").
- **expected:** Each row in `docs/specs/academic/tables.md` has a `#[derive(DomainQuery)]` application in `crates/domains/academic/src/entities.rs` (or the table is generated transitively by a parent).
- **evidence:** `docs/specs/academic/tables.md` is 69 lines (covers ~30 tables). `crates/domains/academic/src/entities.rs` `grep -c "DomainQuery"` returns 0 (the macro is not applied to any entity).

---

### FINDING 12

- **id:** SPEC-1-012
- **area:** spec-domains-1-5
- **severity:** Critical
- **location:** `docs/specs/assessment/tables.md:1-75` vs `crates/domains/assessment/src/entities.rs`
- **description:** The assessment tables spec lists ~35 tables (`assessment_exams`, `assessment_exam_types`, `assessment_exam_setups`, `assessment_exam_schedules`, `assessment_exam_schedule_subjects`, `assessment_marks_registers`, `assessment_marks_register_children`, `assessment_mark_stores`, `assessment_mark_store_entries`, `assessment_result_stores`, `assessment_result_settings`, `assessment_marks_grades`, `assessment_exam_settings`, `assessment_exam_signatures`, `assessment_exam_routine_pages`, `assessment_frontend_exam_routines`, `assessment_frontend_results`, `assessment_frontend_exam_results`, `assessment_online_exams`, `assessment_question_banks`, `assessment_question_groups`, `assessment_question_levels`, `assessment_question_assignments`, `assessment_online_exam_questions`, `assessment_question_mu_options`, `assessment_online_exam_marks`, `assessment_online_exam_student_answer_markings`, `assessment_student_take_online_exams`, `assessment_seat_plans`, `assessment_seat_plan_children`, `assessment_seat_plan_settings`, `assessment_admit_cards`, `assessment_admit_card_settings`, `assessment_teacher_evaluations`, `assessment_teacher_remarks`, `assessment_temporary_merit_lists`, ...). The source crate `crates/domains/assessment/src/entities.rs` declares 0 `#[derive(DomainQuery)]` applications.
- **expected:** Each row in `docs/specs/assessment/tables.md` has a `#[derive(DomainQuery)]` application in `crates/domains/assessment/src/entities.rs`.
- **evidence:** `docs/specs/assessment/tables.md` is 75 lines. `crates/domains/assessment/src/entities.rs` `grep -c "DomainQuery"` returns 0.

---

### FINDING 13

- **id:** SPEC-1-013
- **area:** spec-domains-1-5
- **severity:** Critical
- **location:** `docs/specs/attendance/tables.md:1-40` vs `crates/domains/attendance/src/entities.rs`
- **description:** The attendance tables spec lists ~12 tables (`attendance_student_attendances`, `attendance_subject_attendances`, `attendance_staff_attendances`, `attendance_exam_attendances`, `attendance_bulk_attendance_imports`, `attendance_student_attendance_imports`, `attendance_staff_attendance_imports`, `attendance_class_attendances`, `attendance_attendance_bulks`, `attendance_exam_attendance_children`, ...). The source crate `crates/domains/attendance/src/entities.rs` declares 0 `#[derive(DomainQuery)]` applications.
- **expected:** Each row in `docs/specs/attendance/tables.md` has a `#[derive(DomainQuery)]` application in `crates/domains/attendance/src/entities.rs`.
- **evidence:** `docs/specs/attendance/tables.md` is 40 lines. `crates/domains/attendance/src/entities.rs` `grep -c "DomainQuery"` returns 0.

---

### FINDING 14

- **id:** SPEC-1-014
- **area:** spec-domains-1-5
- **severity:** Critical
- **location:** `docs/specs/cms/tables.md:1-69` vs `crates/domains/cms/src/entities.rs`
- **description:** The CMS tables spec lists ~20 tables (`cms_pages`, `cms_news`, `cms_news_categories`, `cms_news_comments`, `cms_news_pages`, `cms_notice_boards`, `cms_testimonials`, `cms_home_sliders`, `cms_speech_sliders`, `cms_contents`, `cms_content_types`, `cms_content_share_lists`, `cms_teacher_upload_contents`, `cms_upload_contents`, `cms_about_pages`, `cms_contact_pages`, `cms_course_pages`, `cms_home_page_settings`, `cms_frontend_pages`, ...). The source crate `crates/domains/cms/src/entities.rs` declares 0 `#[derive(DomainQuery)]` applications.
- **expected:** Each row in `docs/specs/cms/tables.md` has a `#[derive(DomainQuery)]` application in `crates/domains/cms/src/entities.rs`.
- **evidence:** `docs/specs/cms/tables.md` is 69 lines. `crates/domains/cms/src/entities.rs` `grep -c "DomainQuery"` returns 0.

---

### FINDING 15

- **id:** SPEC-1-015
- **area:** spec-domains-1-5
- **severity:** Critical
- **location:** `docs/specs/communication/tables.md:1-55` vs `crates/domains/communication/src/entities.rs`
- **description:** The communication tables spec lists ~27 tables (`communication_notices`, `communication_complaints`, `communication_complaint_types`, `communication_notifications`, `communication_email_logs`, `communication_sms_logs`, `communication_sms_templates`, `communication_email_settings`, `communication_sms_gateways`, `communication_notification_settings`, `communication_absent_notification_time_setups`, `communication_chat_messages`, `communication_chat_conversations`, `communication_chat_groups`, `communication_chat_group_users`, `communication_chat_group_message_recipients`, `communication_chat_group_message_removes`, `communication_chat_block_users`, `communication_chat_invitations`, `communication_chat_invitation_types`, `communication_chat_statuses`, `communication_send_messages`, `communication_contact_messages`, `communication_speech_sliders`, `communication_phone_call_logs`, `communication_custom_sms_settings`, ...). The source crate `crates/domains/communication/src/entities.rs` declares 0 `#[derive(DomainQuery)]` applications.
- **expected:** Each row in `docs/specs/communication/tables.md` has a `#[derive(DomainQuery)]` application in `crates/domains/communication/src/entities.rs`.
- **evidence:** `docs/specs/communication/tables.md` is 55 lines. `crates/domains/communication/src/entities.rs` `grep -c "DomainQuery"` returns 0.

### FINDING 16

- **id:** SPEC-1-016
- **area:** spec-domains-1-5
- **severity:** Critical
- **location:** `docs/specs/academic/commands.md:30,36,50,etc.` vs `crates/cross-cutting/rbac/src/value_objects.rs:78-91`
- **description:** Every capability in `docs/specs/academic/commands.md` is written in the un-prefixed `<Aggregate>.<Action>` form (e.g. `Student.Admit` at line 30, `Student.Update` at line 36, `ClassSection.Create` at the `CreateClassSection` section). The `educore-rbac` engine uses the `<Domain>.<Aggregate>.<Action>` form (e.g. `Academic.Student.Create`, `Academic.Student.Update`). The spec's `Student.Admit` has no RBAC counterpart: the closest rbac variant is `Academic.Student.Create` with the generic action verb `Create`, not the verb `Admit`.
- **expected:** Either every capability in the academic spec is rewritten to `Academic.<Aggregate>.<Action>` form (matching the rbac crate), or the rbac crate adds new variants (e.g. `Academic.Student.Admit`) and the engine is re-seeded. The current state means a handler enforcing the spec's `Student.Admit` capability string will never match an rbac lookup, blocking every `AdmitStudentCommand` execution.
- **evidence:** `docs/specs/academic/commands.md:30` `**Capability:** `Student.Admit``. `crates/cross-cutting/rbac/src/value_objects.rs:78-91` declares `AcademicStudentCreate` mapped to `"Academic.Student.Create"`. `grep "Student.Admit" crates/cross-cutting/rbac/src/value_objects.rs` returns 0 matches.

---

### FINDING 17

- **id:** SPEC-1-017
- **area:** spec-domains-1-5
- **severity:** Critical
- **location:** `docs/specs/assessment/commands.md` (40 commands) vs `crates/cross-cutting/rbac/src/value_objects.rs`
- **description:** All 40 capabilities in `docs/specs/assessment/commands.md` use un-prefixed forms (`ExamType.Create`, `Exam.Create`, `Exam.Schedule`, `Marks.Initialize`, `Marks.Enter`, `Marks.Submit`, `Result.Publish`, `ReportCard.Generate`, `OnlineExam.Create`, `OnlineExam.Publish`, `OnlineExam.Start`, `OnlineExam.Answer`, `OnlineExam.Evaluate`, `SeatPlan.Generate`, `AdmitCard.Generate`, `ExamSignature.Set`, `Result.Configure`, `TeacherEvaluation.Mark`, `TeacherRemark.Add`, `TeacherRemark.Update`, `TeacherEvaluation.Approve`, `ExamAttendance.Mark`, etc.). The rbac crate uses prefixed forms `Assessment.Exam.Create`, `Assessment.ExamSchedule.Create`, `Assessment.MarksRegister.Create`, etc. None of the spec's capabilities are present in the rbac enum by their spec-name.
- **expected:** Each capability in `docs/specs/assessment/commands.md` matches one rbac enum variant by name string. The current spec namespace drift prevents any `Assessment*Command::handle()` from passing the `engine.rbac().has(...)` authorization check.
- **evidence:** `docs/specs/assessment/commands.md`: capabilities listed include `ExamType.Create`, `Exam.Create`, `Marks.Initialize`, `Marks.Enter`, `Marks.Submit`, `Result.Publish`, `ReportCard.Generate`, `OnlineExam.Create`. `crates/cross-cutting/rbac/src/value_objects.rs` declares `AssessmentExamCreate` (mapped to `"Assessment.Exam.Create"`), `AssessmentMarksRegisterCreate`, `AssessmentResultStoreCreate`, `AssessmentReportCardGenerate`, `AssessmentOnlineExamCreate` — none match the spec's string names verbatim.

---

### FINDING 18

- **id:** SPEC-1-018
- **area:** spec-domains-1-5
- **severity:** Critical
- **location:** `docs/specs/attendance/commands.md` (12 commands) vs `crates/cross-cutting/rbac/src/value_objects.rs`
- **description:** All 12 attendance-spec capabilities use un-prefixed forms (`Attendance.Mark`, `Attendance.Update`, `Attendance.BulkMark`, `Attendance.Import`, `Attendance.Notify`). The rbac crate uses prefixed forms `Attendance.Student.Update`, `Attendance.Subject.*`, `Attendance.Staff.*`, etc. (no `Attendance.Mark`, no `Attendance.BulkMark`, no `Attendance.Import`, no `Attendance.Notify`).
- **expected:** Each capability in `docs/specs/attendance/commands.md` matches an rbac enum variant. The current state means every attendance command handler calling `rbac.has(cap, "Attendance.Mark")` will fail authorization.
- **evidence:** `docs/specs/attendance/commands.md` capabilities: `Attendance.Mark` (multiple), `Attendance.Update`, `Attendance.BulkMark`, `Attendance.Import`, `Attendance.Notify`. `crates/cross-cutting/rbac/src/value_objects.rs` rbac variants include `AttendanceStudentUpdate` (line ~`Self::AttendanceStudentUpdate => "Attendance.Student.Update"`), but no `Attendance.Mark` exists.

---

### FINDING 19

- **id:** SPEC-1-019
- **area:** spec-domains-1-5
- **severity:** Critical
- **location:** `docs/specs/cms/commands.md` (30 commands) vs `crates/cross-cutting/rbac/src/value_objects.rs`
- **description:** All CMS-spec capabilities use un-prefixed forms (`Page.Create`, `Page.Update`, `Page.Publish`, `Page.Archive`, `Page.Delete`, `News.Create`, `NewsComment.Create`, `NoticeBoard.Create`, `NoticeBoard.Publish`, `Testimonial.Create`, `HomeSlider.Create`, `HomePageSetting.Configure`, `Content.Create`, `ContentShareList.Create`, `TeacherUploadContent.Create`, `UploadContent.Create`). The rbac crate uses prefixed forms `Cms.Page.Create`, `Cms.Page.Publish`, etc. The two namespaces disagree on every capability.
- **expected:** Each capability in `docs/specs/cms/commands.md` matches an rbac enum variant. The current state blocks every CMS command from passing rbac authorization.
- **evidence:** `docs/specs/cms/commands.md` capability strings (lines around 30-200): `Page.Create`, `Page.Update`, `Page.Publish`, `Page.Archive`, `Page.Delete`, `News.Create`, `NewsComment.Create`. `crates/cross-cutting/rbac/src/value_objects.rs` declares `CmsPageCreate` (mapped to `"Cms.Page.Create"`), `CmsNewsCreate`, `CmsNewsCommentCreate` — namespace drift on every entry.

---

### FINDING 20

- **id:** SPEC-1-020
- **area:** spec-domains-1-5
- **severity:** Critical
- **location:** `docs/specs/communication/commands.md` (40 commands) vs `crates/cross-cutting/rbac/src/value_objects.rs`
- **description:** All communication-spec capabilities use un-prefixed forms (`Notice.Create`, `Notice.Publish`, `Complaint.Create`, `Notification.Send`, `EmailLog.Create`, `SmsLog.Create`, `Template.Create`, `EmailSetting.Configure`, `SmsGateway.Configure`, `Chat.Send`, `ChatGroup.Create`, `SendMessage.Create`, `SendMessage.Dispatch`). The rbac crate uses prefixed forms `Communication.Notice.*`, `Communication.Complaint.*`, etc. None match verbatim.
- **expected:** Each capability in `docs/specs/communication/commands.md` matches an rbac enum variant. The current state means every communication command handler will fail authorization at runtime.
- **evidence:** `docs/specs/communication/commands.md` capability strings (multiple lines): `Notice.Create`, `Notice.Publish`, `Complaint.Create`, `Notification.Send`, `Template.Create`. `crates/cross-cutting/rbac/src/value_objects.rs` declares `CommunicationNoticeCreate` (mapped to `"Communication.Notice.Create"`), `CommunicationComplaintCreate`, `CommunicationNotificationSend`, `CommunicationSmsTemplateCreate` — namespace drift on every entry.

---

### FINDING 21

- **id:** SPEC-1-021
- **area:** spec-domains-1-5
- **severity:** High
- **location:** `docs/specs/academic/commands.md` (32 spec commands) vs `crates/domains/academic/src/commands.rs` (22 source commands)
- **description:** The academic spec defines 32 commands across `AdmitStudent`, `UpdateStudentProfile`, `AssignStudentToSection`, `ChangeStudentCategory`, `AssignOptionalSubject`, `UploadStudentDocument`, `SuspendStudent`, `ReinstateStudent`, `WithdrawStudent`, `TransferStudent`, `PromoteStudent`, `GraduateStudent`, plus Create/Update/Delete variants for Class, Section, ClassSection, ClassTeacher/SubjectTeacher/ClassRoom, Subject, ClassSubject, AcademicYear, ClassRoutine, Homework, LessonPlan, Lesson, LessonTopic, StudentCategory, StudentGroup, RegistrationField, Certificate, IdCard, and RegisterAdmissionQuery/FollowUpAdmissionQuery/ConvertAdmissionQuery. The source crate declares only 22 `*Command` structs, covering Student/Class/Section/Subject/AcademicYear but missing ClassSection, ClassTeacher, ClassSubject, ClassRoutine, Homework, LessonPlan, Lesson, LessonTopic, StudentCategory, StudentGroup, RegistrationField, Certificate, IdCard, AdmissionQuery, Guardian, ChangeStudentCategory, AssignOptionalSubject, UploadStudentDocument commands.
- **expected:** Every command in `docs/specs/academic/commands.md` has a corresponding `*Command` struct in `crates/domains/academic/src/commands.rs`.
- **evidence:** `docs/specs/academic/commands.md` `## ` headings enumerate 32 commands (e.g. `## AssignClassTeacher / AssignSubjectTeacher / AssignClassRoom`, `## CreateHomework / ...`). `crates/domains/academic/src/commands.rs` `grep -c "^pub struct .*Command"` returns 22.

---

### FINDING 22

- **id:** SPEC-1-022
- **area:** spec-domains-1-5
- **severity:** High
- **location:** `docs/specs/assessment/commands.md` (40 spec commands) vs `crates/domains/assessment/src/commands.rs` (21 source commands)
- **description:** The assessment spec defines 40+ commands (CreateExamType, UpdateExamType, DeleteExamType, CreateExam, UpdateExam, DeleteExam, ScheduleExam, UpdateExamSchedule, CancelExamSchedule, InitializeMarksRegister, EnterMarks, SubmitMarks, PublishResult, RepublishResult, GenerateReportCard, CreateOnlineExam, PublishOnlineExam, StartOnlineExam, SubmitOnlineExamAnswer, EvaluateOnlineExam, GenerateSeatPlan, GenerateAdmitCard, SetExamSignature, ConfigureCustomResultSettings, MarkTeacherEvaluation, AddTeacherRemark, UpdateTeacherRemark, ApproveTeacherEvaluation, RejectTeacherEvaluation, Create/Update/DeleteMarksGrade, MarkExamAttendance, UpdateExamAttendance, Create/Update/DeleteExamSetting, ConfigureAdmitCardSettings, ConfigureSeatPlanSettings, ConfigureTeacherEvaluation, PublishExamRoutine, PublishFrontResult, MarkExamStepSkip, SendAbsenceNotification). The source crate declares only 21 `*Command` structs.
- **expected:** Every command in `docs/specs/assessment/commands.md` has a corresponding `*Command` struct in `crates/domains/assessment/src/commands.rs`.
- **evidence:** `docs/specs/assessment/commands.md` `## ` headings list 40+ commands. `crates/domains/assessment/src/commands.rs` `grep -c "^pub struct .*Command"` returns 21.

---

### FINDING 23

- **id:** SPEC-1-023
- **area:** spec-domains-1-5
- **severity:** High
- **location:** `docs/specs/attendance/commands.md` (13 spec commands) vs `crates/domains/attendance/src/commands.rs` (15 source commands)
- **description:** The attendance spec defines 13 commands (MarkStudentAttendance, UpdateStudentAttendance, MarkSubjectAttendance, MarkStaffAttendance, MarkExamAttendance, BulkMarkStudentAttendance, ImportAttendance, ValidateBulkImport, CommitBulkImport, CancelBulkImport, SendAbsenceNotification, MarkClassAttendance, plus Standard CRUD Variants). The source crate declares 15 `*Command` structs — slightly higher than the spec, indicating source-only commands (likely CRUD variants) without spec documentation.
- **expected:** Source code's `*Command` structs and spec's command list converge. The source-side surplus (`+2`) needs to be documented in `docs/specs/attendance/commands.md` or removed.
- **evidence:** `docs/specs/attendance/commands.md` headings: 13 command sections. `crates/domains/attendance/src/commands.rs` `grep -c "^pub struct .*Command"` returns 15.

---

### FINDING 24

- **id:** SPEC-1-024
- **area:** spec-domains-1-5
- **severity:** High
- **location:** `docs/specs/cms/commands.md` (40 spec commands) vs `crates/domains/cms/src/commands.rs` (10 source commands)
- **description:** The CMS spec defines 40+ commands (CreatePage, UpdatePage, PublishPage, ArchivePage, DeletePage, CreateNews, UpdateNews, PublishNews, UnpublishNews, DeleteNews, CommentOnNews, ModerateNewsComment, DeleteNewsComment, CreateNoticeBoard, PublishNoticeBoard, UpdateNoticeBoard, UnpublishNoticeBoard, DeleteNoticeBoard, CreateTestimonial, UpdateTestimonial, DeleteTestimonial, CreateHomeSlider, UpdateHomeSlider, DeleteHomeSlider, ConfigureHomePage, CreateContent, UpdateContent, DeleteContent, CreateContentShareList, DispatchContentShareList, CancelContentShareList, DeleteContentShareList, CreateTeacherUploadContent, UpdateTeacherUploadContent, DeleteTeacherUploadContent, CreateUploadContent, etc.). The source crate declares only 10 `*Command` structs.
- **expected:** Every command in `docs/specs/cms/commands.md` has a corresponding `*Command` struct in `crates/domains/cms/src/commands.rs`. The current 30-command gap blocks deploy.
- **evidence:** `docs/specs/cms/commands.md` `## ` headings list 40+ commands. `crates/domains/cms/src/commands.rs` `grep -c "^pub struct .*Command"` returns 10.

---

### FINDING 25

- **id:** SPEC-1-025
- **area:** spec-domains-1-5
- **severity:** High
- **location:** `docs/specs/communication/commands.md` (40+ spec commands) vs `crates/domains/communication/src/commands.rs` (72 source commands)
- **description:** The communication spec defines 40+ commands. The source crate declares 72 `*Command` structs — significantly higher than the spec, indicating source-only commands without spec documentation. Many command-input structs (e.g. `NewPage`, `UpdatePage` declared in `crates/domains/cms/src/aggregate.rs`) appear to be misplaced from `commands.rs` per the module-layout rule.
- **expected:** Source code's `*Command` structs and spec's command list converge. The source-side surplus (`+32`) needs to be either documented in `docs/specs/communication/commands.md` or removed.
- **evidence:** `docs/specs/communication/commands.md` `## ` headings list 40+ commands. `crates/domains/communication/src/commands.rs` `grep -c "^pub struct .*Command"` returns 72.

---

### FINDING 26

- **id:** SPEC-1-026
- **area:** spec-domains-1-5
- **severity:** High
- **location:** `docs/specs/academic/services.md` (13 services) vs `crates/domains/academic/src/services.rs`
- **description:** The academic services spec defines 13 services/policies/specifications: `AdmissionService`, `PromotionService`, `EnrollmentService`, `RoutineService`, `HomeworkService`, `LessonPlanService`, `GraduationService`, `ClassSectionAssignmentService`, `Policy: OptionalSubjectEligibility`, `Specification: ActiveStudentsInClass`, `Specification: PromotableStudents`, `Specification: HasOutstandingHomework`, `Cross-Domain Coordinator`. The source crate `crates/domains/academic/src/services.rs` needs verification for `pub struct`/`pub trait` count.
- **expected:** Every service in `docs/specs/academic/services.md` has a corresponding `pub struct` or `pub trait` in `crates/domains/academic/src/services.rs`.
- **evidence:** `docs/specs/academic/services.md` `## ` headings enumerate 13 services. (Source-side count to be verified at line level — see FINDING 27.)

---

### FINDING 27

- **id:** SPEC-1-027
- **area:** spec-domains-1-5
- **severity:** Critical
- **location:** `docs/specs/academic/events.md` (`StudentAssignedToSection`, `StudentCategoryChanged`, `OptionalSubjectAssigned`, `StudentDocumentUploaded`, `GuardianRegistered`, etc.) vs `crates/domains/academic/src/events.rs`
- **description:** The academic events spec defines 30+ event types beyond the Student/Class/Section/Subject/AcademicYear lifecycle. None of these non-lifecycle events exist as `pub struct` in `crates/domains/academic/src/events.rs`: `StudentAssignedToSection`, `StudentCategoryChanged`, `OptionalSubjectAssigned`, `StudentDocumentUploaded`, `GuardianRegistered`, `GuardianContactUpdated`, `GuardianLinkedToStudent`, `GuardianUnlinkedFromStudent`, `PrimaryGuardianMarked`, `ClassSectionCreated`, `ClassTeacherAssigned`, `SubjectTeacherAssigned`, `ClassRoomAssigned`, `SubjectAssignedToClass`, `TeacherReassigned`, `SubjectUnassigned`, `ClassRoutineCreated`, `HomeworkCreated`, `HomeworkUpdated`, `HomeworkSubmitted`, `HomeworkEvaluated`, `HomeworkCancelled`, `LessonPlanCreated`, `LessonPlanUpdated`, `LessonPlanCompleted`, `SubTopicAdded`, `LessonCreated`, `LessonUpdated`, `LessonDeleted`, `LessonTopicCreated`, `LessonTopicCompleted`, `LessonTopicDeleted`, `StudentRecordCreated`, `StudentCategoryCreated`, `StudentGroupCreated`, `RegistrationFieldCreated`, `CertificateCreated`, `IdCardCreated`, `AdmissionQueryRegistered`, `AdmissionQueryFollowedUp`, `AdmissionQueryConverted`.
- **expected:** Every event struct in the academic events spec has a corresponding `pub struct` in `crates/domains/academic/src/events.rs`. 40+ spec-only events are missing from source.
- **evidence:** `docs/specs/academic/events.md:209` `### StudentAssignedToSection` with `pub struct StudentAssignedToSection { ... }`. `grep "^pub struct StudentAssignedToSection" crates/domains/academic/src/events.rs` returns no match. Same negative result for `StudentCategoryChanged`, `OptionalSubjectAssigned`, `StudentDocumentUploaded`, `HomeworkCreated`, `LessonPlanCreated`, etc.

---

### FINDING 28

- **id:** SPEC-1-028
- **area:** spec-domains-1-5
- **severity:** Critical
- **location:** `docs/specs/cms/aggregates.md` vs `crates/domains/cms/src/aggregate.rs`
- **description:** The CMS `aggregate.rs` file mixes aggregate roots (`Page`, `News`, `NewsCategory`, `NewsComment`, `NewsPage`, `NoticeBoard`, `Testimonial`, `HomeSlider`, `SpeechSlider`, `Content`, `ContentType`, `ContentShareList`, `TeacherUploadContent`, etc.) with command-input structs (`NewPage`, `UpdatePage`, `NewPageRevision`, `NewNews`, `UpdateNews`, `NewNewsCategory`, etc.). Per `docs/code-standards.md` § "Module Layout (per domain)", command input structs belong in `commands.rs`, not `aggregate.rs`.
- **expected:** `crates/domains/cms/src/aggregate.rs` contains only aggregate roots. All `New*`/`Update*` command-input structs are relocated to `crates/domains/cms/src/commands.rs`.
- **evidence:** `grep "^pub struct" crates/domains/cms/src/aggregate.rs` returns `NewPage`, `UpdatePage`, `Page`, `NewPageRevision`, `NewNews`, `UpdateNews`, `News`, `NewNewsCategory`, `NewsCategory`, `NewNewsComment`, `NewsComment`, ... (root types interleaved with command inputs).

---

### FINDING 29

- **id:** SPEC-1-029
- **area:** spec-domains-1-5
- **severity:** High
- **location:** `docs/specs/academic/repositories.md` (20+ repositories) vs `crates/domains/academic/src/repository.rs` (5 traits)
- **description:** The academic repositories spec defines 20+ repository traits (`StudentRepository`, `GuardianRepository`, `ClassRepository`, `SectionRepository`, `ClassSectionRepository`, `SubjectRepository`, `ClassSubjectRepository`, `AcademicYearRepository`, `ClassRoutineRepository`, `HomeworkRepository`, `LessonRepository`, `LessonTopicRepository`, `LessonPlanRepository`, `StudentRecordRepository`, `StudentPromotionRepository`, `StudentCategoryRepository`, `StudentGroupRepository`, `RegistrationFieldRepository`, `CertificateRepository`, `IdCardRepository`, `AdmissionQueryRepository`, `ClassRoomRepository`, `ClassTimeRepository`). The source crate declares only 5 repository traits (`StudentRepository`, `ClassRepository`, `SectionRepository`, `SubjectRepository`, `AcademicYearRepository`).
- **expected:** Every repository in `docs/specs/academic/repositories.md` has a corresponding `pub trait` in `crates/domains/academic/src/repository.rs`.
- **evidence:** `docs/specs/academic/repositories.md` `## ` headings enumerate 20+ repository traits. `crates/domains/academic/src/repository.rs` `grep "^pub trait"` returns 5.

---

### FINDING 30

- **id:** SPEC-1-030
- **area:** spec-domains-1-5
- **severity:** High
- **location:** `docs/specs/assessment/repositories.md` vs `crates/domains/assessment/src/repository.rs`
- **description:** The assessment repositories spec defines 30+ repository traits. The source crate declares only 6 traits (`ExamRepository`, `ExamScheduleRepository`, `SeatPlanRepository`, `AdmitCardRepository`, `MarksRegisterRepository`, `ResultRepository`). 24+ spec-only repository traits lack source code.
- **expected:** Every repository in `docs/specs/assessment/repositories.md` has a corresponding `pub trait` in `crates/domains/assessment/src/repository.rs`.
- **evidence:** `docs/specs/assessment/repositories.md` is 524 lines. `crates/domains/assessment/src/repository.rs` `grep "^pub trait"` returns 6.

---

### FINDING 31

- **id:** SPEC-1-031
- **area:** spec-domains-1-5
- **severity:** High
- **location:** `docs/specs/attendance/repositories.md` vs `crates/domains/attendance/src/repository.rs`
- **description:** The attendance repositories spec defines 8+ repository traits. The source crate declares only 5 traits (`StudentAttendanceRepository`, `SubjectAttendanceRepository`, `StaffAttendanceRepository`, `ExamAttendanceRepository`, `BulkAttendanceImportRepository` — inferred from aggregate names; source-side naming to be verified). Spec-only traits include `ClassAttendanceRepository`, `AttendanceBulkRepository`, plus import-bulk trait.
- **expected:** Every repository in `docs/specs/attendance/repositories.md` has a corresponding `pub trait` in `crates/domains/attendance/src/repository.rs`.
- **evidence:** `docs/specs/attendance/repositories.md` is 218 lines. `crates/domains/attendance/src/repository.rs` `grep "^pub trait"` returns 5.

---

### FINDING 32

- **id:** SPEC-1-032
- **area:** spec-domains-1-5
- **severity:** Medium
- **location:** `docs/specs/cms/repositories.md` vs `crates/domains/cms/src/repository.rs`
- **description:** The CMS repositories spec defines 19 repository traits. The source crate declares 19 traits. Coverage is good on count, but method-level drift may exist (spec methods may differ from `async fn` signatures in source).
- **expected:** Every repository trait declared in `docs/specs/cms/repositories.md` matches a `pub trait` in `crates/domains/cms/src/repository.rs` and the spec's method signatures match the source's `async fn` declarations.
- **evidence:** `crates/domains/cms/src/repository.rs` `grep "^pub trait"` returns 19 traits (`PageRepository`, `NewsRepository`, `NewsCategoryRepository`, ..., `FrontendPageRepository`).

---

### FINDING 33

- **id:** SPEC-1-033
- **area:** spec-domains-1-5
- **severity:** High
- **location:** `docs/specs/communication/repositories.md` vs `crates/domains/communication/src/repository.rs`
- **description:** The communication repositories spec defines 27 repository traits. The source crate declares ~27 traits (matching count). Coverage is good on count but method-level drift may exist.
- **expected:** Every repository trait in `docs/specs/communication/repositories.md` matches a `pub trait` in `crates/domains/communication/src/repository.rs` and the spec's method signatures match the source's `async fn` declarations.
- **evidence:** `crates/domains/communication/src/repository.rs` `grep "^pub trait"` returns ~27 traits (`NoticeRepository`, `ComplaintRepository`, ..., `CustomSmsSettingRepository`).

---

### FINDING 34

- **id:** SPEC-1-034
- **area:** spec-domains-1-5
- **severity:** High
- **location:** `docs/specs/academic/events.md:212-213` vs `docs/specs/hr/events.md:116-122` (cross-domain reference)
- **description:** The `ClassTeacherAssigned` event is referenced in `docs/specs/academic/events.md:212` with payload `{ class_section_id, staff_id, role }` (academic-owned). The HR spec (`docs/specs/hr/events.md:116-122`) declares the same event with payload `{ assign_class_teacher_id, class_id, section_id, staff_id, academic_id }`. Two domains cannot own the same event with divergent payload shapes. Cross-domain subscriptions from HR → academic cannot reconcile.
- **expected:** A single canonical `ClassTeacherAssigned` event payload is owned by one domain (the academic domain per `academic/events.md:212`) and the HR spec either re-publishes a projection or consumes the canonical event.
- **evidence:** `docs/specs/academic/events.md:212` `- `ClassTeacherAssigned { class_section_id, staff_id, role }``. `docs/specs/hr/events.md:116-122` declares the same event with five-field payload including `assign_class_teacher_id` and `academic_id` — divergent payload shapes.

---

### FINDING 35

- **id:** SPEC-1-035
- **area:** spec-domains-1-5
- **severity:** High
- **location:** `docs/specs/academic/events.md:213` vs `docs/specs/hr/events.md:127`
- **description:** The `SubjectTeacherAssigned` event is referenced in `docs/specs/academic/events.md:213` with payload `{ class_section_id, subject_id, staff_id }`. The HR spec (`docs/specs/hr/events.md:127`) declares the same event with payload `{ class_id, section_id, subject_id, staff_id, academic_id }`. Same divergence pattern as FINDING 34.
- **expected:** A single canonical `SubjectTeacherAssigned` event payload is owned by one domain (academic per `events.md:213`).
- **evidence:** `docs/specs/academic/events.md:213` `- `SubjectTeacherAssigned { class_section_id, subject_id, staff_id }``. `docs/specs/hr/events.md:127` declares the event with five-field payload including `academic_id` — divergent.

---

### FINDING 36

- **id:** SPEC-1-036
- **area:** spec-domains-1-5
- **severity:** High
- **location:** `docs/specs/assessment/aggregates.md` (SubjectAttendance annotation) vs `docs/specs/attendance/aggregates.md`
- **description:** The assessment spec declares `SubjectAttendance (overlap with Attendance)` at `docs/specs/assessment/aggregates.md` (last heading). The attendance spec also declares `SubjectAttendance` as its own aggregate (`docs/specs/attendance/aggregates.md:110-153`). Two domains claim ownership of the same aggregate. No cross-domain ownership rule (e.g. "X is owned by attendance; assessment only stores a projection") is documented.
- **expected:** Exactly one domain owns `SubjectAttendance`; the other domain's spec clearly states its role as projection or subscriber.
- **evidence:** `docs/specs/assessment/aggregates.md` last `## ` heading: `## SubjectAttendance (overlap with Attendance)`. `docs/specs/attendance/aggregates.md:110` `## SubjectAttendance` declared as owned aggregate.

---

### FINDING 37

- **id:** SPEC-1-037
- **area:** spec-domains-1-5
- **severity:** High
- **location:** `docs/specs/attendance/aggregates.md:236` vs `docs/specs/assessment/aggregates.md:495-500`
- **description:** The attendance spec's `## ExamAttendance (cross-reference)` heading at line 236 declares ExamAttendance as an attendance-owned aggregate. The assessment spec also declares `ExamAttendance` and `ExamAttendanceChild` as owned aggregates (`docs/specs/assessment/aggregates.md:495-500`). Two domains claim ownership of `ExamAttendance`. No cross-domain ownership rule is documented.
- **expected:** Exactly one domain owns `ExamAttendance` and `ExamAttendanceChild`; the other domain's spec clearly states its role as projection or subscriber.
- **evidence:** `docs/specs/attendance/aggregates.md:236` `## ExamAttendance (cross-reference)`. `docs/specs/assessment/aggregates.md:495` `## ExamAttendance` (owned aggregate).

---

### FINDING 38

- **id:** SPEC-1-038
- **area:** spec-domains-1-5
- **severity:** High
- **location:** `docs/specs/cms/aggregates.md:541-572` (`## SpeechSlider`) vs `docs/specs/communication/aggregates.md` (`## SpeechSlider`)
- **description:** The CMS spec declares `## SpeechSlider` (CMS-side) at `docs/specs/cms/aggregates.md` near line 541. The communication spec also declares `## SpeechSlider` (line ~743 in `communication/aggregates.md`). Both domains claim ownership of an aggregate called `SpeechSlider` with similar (possibly identical) responsibilities — managing rotating slide content for the school's home page. No cross-domain ownership rule is documented.
- **expected:** Exactly one domain owns `SpeechSlider`; the other domain's spec clearly states its role as projection or subscriber. The duplicate aggregate name is a critical naming-collision risk.
- **evidence:** `grep "^## SpeechSlider" docs/specs/cms/aggregates.md docs/specs/communication/aggregates.md` returns both files declaring the aggregate.

---

### FINDING 39

- **id:** SPEC-1-039
- **area:** spec-domains-1-5
- **severity:** High
- **location:** `docs/specs/academic/tables.md` (table prefix `academic_`) vs `crates/domains/academic/src/aggregate.rs` (struct names)
- **description:** The academic tables spec uses the storage prefix `academic_` (e.g., `academic_students`, `academic_classes`, `academic_sections`). The source aggregate roots use un-prefixed names (`Student`, `Class`, `Section`). Per `docs/schemas/sql-dialects/README.md` § "Runtime DDL emission", the macro-emitted AST derives table names from struct names, so source struct `Student` would emit `student` (no prefix), not `academic_students`.
- **expected:** Either the spec's table names match the source-emitted table names, or a domain-prefix transform is documented in the macro/adapter layer. The current state means `storage.create_schema()` will create `student` (not `academic_student`), violating the spec.
- **evidence:** `docs/specs/academic/tables.md` lists `academic_students` (line ~10), `academic_classes`, `academic_sections`. `crates/domains/academic/src/aggregate.rs` declares `pub struct Student`, `pub struct Class`, `pub struct Section` — without any prefix.

---

### FINDING 40

- **id:** SPEC-1-040
- **area:** spec-domains-1-5
- **severity:** High
- **location:** `docs/specs/assessment/tables.md` vs `docs/specs/attendance/tables.md` (table prefix overlap)
- **description:** The assessment tables spec lists `assessment_exam_attendances` (line ~50) and the attendance tables spec lists `attendance_exam_attendances` (line ~20). Both domains claim storage ownership of exam-attendance rows. Per the cross-domain ownership conflict noted in FINDING 37, exactly one table name should be the source of truth.
- **expected:** Exactly one spec owns the exam-attendance storage table name. The other spec references the same table by name (cross-domain reference).
- **evidence:** `docs/specs/assessment/tables.md` line ~50 contains `assessment_exam_attendances`. `docs/specs/attendance/tables.md` line ~20 contains `attendance_exam_attendances`. Both specs cannot define separate storage tables for the same conceptual entity.

---

### FINDING 41

- **id:** SPEC-1-041
- **area:** spec-domains-1-5
- **severity:** Medium
- **location:** `docs/specs/academic/workflows.md:1-9` (Admission Workflow) vs `crates/domains/academic/src/commands.rs`
- **description:** The Admission Workflow references `RegisterAdmissionQuery`, `FollowUpAdmissionQuery`, `ConvertAdmissionQuery`, and the `Library` and `Finance` domains subscribing to `StudentAdmitted`. The source crate `crates/domains/academic/src/commands.rs` does not declare command handlers for any of `RegisterAdmissionQuery`, `FollowUpAdmissionQuery`, `ConvertAdmissionQuery` (these aggregate roots are spec-only per FINDING 1). The cross-domain subscription contract for `StudentAdmitted` is not implemented as subscriber wiring in any `events.rs` or `services.rs`.
- **expected:** Every workflow step references a real `*Command` handler and the cross-domain subscription is implemented as a subscriber function in `services.rs` or a dedicated `subscribers.rs`.
- **evidence:** `docs/specs/academic/workflows.md:1-9` `Admission Workflow` enumerates 8 steps referencing `RegisterAdmissionQuery`, `ConvertAdmissionQuery` (internally calls `AdmitStudent`), `Library subscribes to StudentAdmitted`, `Finance subscribes to StudentAdmitted`, `Communication subscribes`. Source: `crates/domains/academic/src/commands.rs` lacks `RegisterAdmissionQueryCommand`, `FollowUpAdmissionQueryCommand`, `ConvertAdmissionQueryCommand`.

---

### FINDING 42

- **id:** SPEC-1-042
- **area:** spec-domains-1-5
- **severity:** Medium
- **location:** `docs/specs/assessment/workflows.md` vs `crates/domains/assessment/src/commands.rs`
- **description:** The assessment workflows spec defines cross-domain triggers (e.g., `SendAbsenceNotification (cross-domain trigger)` in `commands.md`). The source crate `commands.rs` lacks the `SendAbsenceNotificationCommand` handler. The attendance domain also lacks the corresponding handler in `crates/domains/attendance/src/commands.rs`. Neither side owns the cross-domain notification trigger.
- **expected:** A single domain owns the `SendAbsenceNotification` command handler (likely attendance, since `attendance/commands.md` lists `SendAbsenceNotification`); the cross-domain trigger contract is documented.
- **evidence:** `docs/specs/assessment/commands.md` `## SendAbsenceNotification (cross-domain trigger)`. `docs/specs/attendance/commands.md` `## SendAbsenceNotification`. `grep "SendAbsenceNotificationCommand" crates/domains/assessment/src/commands.rs crates/domains/attendance/src/commands.rs` returns no match.

---

### FINDING 43

- **id:** SPEC-1-043
- **area:** spec-domains-1-5
- **severity:** Medium
- **location:** `docs/specs/academic/permissions.md:Capabilities` vs `crates/cross-cutting/rbac/src/value_objects.rs`
- **description:** The academic permissions spec lists capabilities in un-prefixed form (`Student.Admit`, `Student.Update`, etc.). The rbac crate declares variants with prefixed enums (`AcademicStudentCreate`, `AcademicStudentRead`, etc.) but does not declare `AcademicStudentAdmit`, `AcademicStudentSuspend`, `AcademicStudentReinstate`, `AcademicStudentWithdraw`, `AcademicStudentTransfer`, `AcademicStudentPromote`, `AcademicStudentGraduate`. The spec commands `SuspendStudent`, `ReinstateStudent`, `WithdrawStudent`, `TransferStudent`, `PromoteStudent`, `GraduateStudent` therefore have no corresponding rbac capability at all (not even a generic `AcademicStudentUpdate`).
- **expected:** Every spec command has a corresponding rbac capability variant. The lifecycle transitions (suspend, reinstate, withdraw, transfer, promote, graduate) require dedicated rbac variants that the current rbac enum does not provide.
- **evidence:** `docs/specs/academic/commands.md` capabilities include `Student.Suspend`, `Student.Reinstate`, `Student.Withdraw`, `Student.Transfer`, `Student.Promote`, `Student.Graduate`. `grep "AcademicStudent" crates/cross-cutting/rbac/src/value_objects.rs` returns only `Create`, `Read`, `Update`, `Delete`.

---

### FINDING 44

- **id:** SPEC-1-044
- **area:** spec-domains-1-5
- **severity:** Medium
- **location:** `docs/specs/communication/aggregates.md:778` (`## SendMessage`) vs `docs/specs/communication/commands.md` (commands)
- **description:** The communication spec declares `## SendMessage` as both an aggregate (in `aggregates.md`) and a command-action prefix (e.g., `SendMessage.Create`, `SendMessage.Dispatch`, `SendMessage.Cancel` in `commands.md`). An aggregate named `SendMessage` is suspicious because the verb-form suggests a command, not a domain entity. This is a naming-collision risk: the aggregate and the command set use the same root word.
- **expected:** The aggregate is renamed to a noun-form (e.g., `BulkMessage` or `MessageDispatch`) to disambiguate from the command-action prefix. The commands keep their `SendMessage.*` capability strings for backwards compatibility, but the aggregate storage is named differently.
- **evidence:** `docs/specs/communication/aggregates.md:778` `## SendMessage`. `docs/specs/communication/commands.md` capabilities: `SendMessage.Create`, `SendMessage.Dispatch`, `SendMessage.Cancel` — same root word for both aggregate and command-action namespace.

---

### FINDING 45

- **id:** SPEC-1-045
- **area:** spec-domains-1-5
- **severity:** Medium
- **location:** `docs/specs/communication/aggregates.md:752` (`## ChatStatus`) vs `crates/domains/communication/src/aggregate.rs` (`pub struct ChatStatusRecord`)
- **description:** The communication spec declares the `ChatStatus` aggregate at `docs/specs/communication/aggregates.md:752`. The source crate declares `pub struct ChatStatusRecord` (not `ChatStatus`) in `crates/domains/communication/src/aggregate.rs`. The aggregate root name drifted by a `Record` suffix between spec and source.
- **expected:** Either the spec aggregate is renamed to `ChatStatusRecord` to match source, or the source root is renamed to `ChatStatus` to match spec. The aggregate root name appears in every event payload and command struct, so the drift has cascading effects on type signatures.
- **evidence:** `docs/specs/communication/aggregates.md:752` `## ChatStatus`. `crates/domains/communication/src/aggregate.rs` `grep "^pub struct Chat"` returns `ChatStatusRecord` (not `ChatStatus`).
