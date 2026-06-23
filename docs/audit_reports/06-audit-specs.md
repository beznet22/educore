# 06 - Audit Appendix - Spec folders (4 audits)

**Scope:** wave6-specs-1.md, wave6-specs-2.md, wave6-specs-3.md, wave6-specs-4.md

**Total findings:** 131

**Severity distribution:** 38 critical, 52 high, 31 medium, 10 low


## Summary Table

| Target | Critical | High | Medium | Low | Total |
| --- | --- | --- | --- | --- | --- |
| Specs (academic..finance) (`SPEC-1`) | 22 | 17 | 6 | 0 | 45 |
| Specs (hr, library, events-domain) (`SPEC-2`) | 10 | 13 | 7 | 0 | 30 |
| Specs (documents, facilities, attendance) (`SPEC-3`) | 5 | 16 | 7 | 3 | 31 |
| Specs (cross-cutting + data-migration) (`SPEC-4`) | 1 | 6 | 11 | 7 | 25 |

## Specs (academic..finance) (target id prefix: `SPEC-1`)

**Path:** `docs/specs/{academic,assessment,attendance,cms,communication,finance}/`  
**Total findings:** 45 (22 critical, 17 high, 6 medium, 0 low)


### FINDING 1 (id: `SPEC-1-001`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** Critical
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/academic/aggregates.md:21-32` vs `crates/domains/academic/src/aggregate.rs`

**Description:**

The academic spec defines 20 aggregates in `aggregates.md` (`Student`, `Guardian`, `Class`, `Section`, `ClassSection`, `Subject`, `ClassSubject`, `AcademicYear`, `ClassRoutine`, `Homework`, `LessonPlan`, `Lesson`, `LessonTopic`, `StudentRecord`, `StudentPromotion`, `StudentCategory`, `StudentGroup`, `RegistrationField`, `Certificate`, `IdCard`). The source crate `crates/domains/academic/src/aggregate.rs` declares only 5 aggregate roots (`Student`, `Class`, `Section`, `Subject`, `AcademicYear`). 15 aggregates (`Guardian`, `ClassSection`, `ClassSubject`, `ClassRoutine`, `Homework`, `LessonPlan`, `Lesson`, `LessonTopic`, `StudentRecord`, `StudentPromotion`, `StudentCategory`, `StudentGroup`, `RegistrationField`, `Certificate`, `IdCard`) have no Rust struct.

**Expected:**

Every aggregate declared in `docs/specs/academic/aggregates.md` has a corresponding `pub struct` root in `crates/domains/academic/src/aggregate.rs`.

**Evidence:**

`docs/specs/academic/aggregates.md`:20 aggregates under `## ` headings (`## Student` ... `## IdCard`). `crates/domains/academic/src/aggregate.rs` `grep "^pub struct "` returns `Student`, `Class`, `Section`, `Subject`, `AcademicYear` only.

---

### FINDING 10 (id: `SPEC-1-010`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** Critical
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/communication/events.md:35-333` vs `crates/domains/communication/src/events.rs`

**Description:**

The communication events spec defines 18 lifecycle sections (`Notice Lifecycle`, `Complaint Lifecycle`, `Notification Lifecycle`, `Email & SMS Logs`, `Templates`, `Email Engine Configuration`, `SMS Gateway Configuration`, `Notification Routing`, `Absent Notification`, `Chat — One-to-One`, `Chat — Groups`, `Chat — Group Message Delivery`, `Chat — Block, Invitation, Status`, `Send Message (Bulk)`, `Contact Message`, `Speech Slider`, `Phone Call`, `Custom SMS Gateway`). The source crate declares ~60+ event structs but several spec events are missing: `ChatInvitationAccepted`, `ChatInvitationRejected`, `ChatStatusChanged`, `PhoneCallLogged`, `ContactMessageReceived`, `CustomSmsGatewayProvisioned`.

**Expected:**

Every event declared in `docs/specs/communication/events.md` has a corresponding `pub struct` in `crates/domains/communication/src/events.rs`.

**Evidence:**

`grep "^pub struct " crates/domains/communication/src/events.rs` returns ~60+ event types. Spec events like `ChatInvitationAccepted` (referenced in `aggregates.md:799` ChatInvitation section) have no source struct.

---

### FINDING 11 (id: `SPEC-1-011`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** Critical
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/academic/tables.md:1-69` vs `crates/domains/academic/src/entities.rs`

**Description:**

The academic tables spec lists ~30 tables (`academic_students`, `academic_guardians`, `academic_classes`, `academic_sections`, `academic_class_sections`, `academic_subjects`, `academic_class_subjects`, `academic_academic_years`, `academic_class_routines`, `academic_homework`, `academic_lesson_plans`, `academic_lessons`, `academic_lesson_topics`, `academic_student_records`, `academic_student_promotions`, `academic_student_categories`, `academic_student_groups`, `academic_registration_fields`, `academic_certificates`, `academic_id_cards`, `academic_optional_subject_assignments`, `academic_student_documents`, `academic_student_timelines`, `academic_student_homework`, `academic_guardian_links`, ...). The source crate `crates/domains/academic/src/entities.rs` does not declare any `#[derive(DomainQuery)]` applications (the macro that emits the typed AST contract per `docs/build-plan.md` Phase 0 § "Foundation").

**Expected:**

Each row in `docs/specs/academic/tables.md` has a `#[derive(DomainQuery)]` application in `crates/domains/academic/src/entities.rs` (or the table is generated transitively by a parent).

**Evidence:**

`docs/specs/academic/tables.md` is 69 lines (covers ~30 tables). `crates/domains/academic/src/entities.rs` `grep -c "DomainQuery"` returns 0 (the macro is not applied to any entity).

---

### FINDING 12 (id: `SPEC-1-012`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** Critical
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/assessment/tables.md:1-75` vs `crates/domains/assessment/src/entities.rs`

**Description:**

The assessment tables spec lists ~35 tables (`assessment_exams`, `assessment_exam_types`, `assessment_exam_setups`, `assessment_exam_schedules`, `assessment_exam_schedule_subjects`, `assessment_marks_registers`, `assessment_marks_register_children`, `assessment_mark_stores`, `assessment_mark_store_entries`, `assessment_result_stores`, `assessment_result_settings`, `assessment_marks_grades`, `assessment_exam_settings`, `assessment_exam_signatures`, `assessment_exam_routine_pages`, `assessment_frontend_exam_routines`, `assessment_frontend_results`, `assessment_frontend_exam_results`, `assessment_online_exams`, `assessment_question_banks`, `assessment_question_groups`, `assessment_question_levels`, `assessment_question_assignments`, `assessment_online_exam_questions`, `assessment_question_mu_options`, `assessment_online_exam_marks`, `assessment_online_exam_student_answer_markings`, `assessment_student_take_online_exams`, `assessment_seat_plans`, `assessment_seat_plan_children`, `assessment_seat_plan_settings`, `assessment_admit_cards`, `assessment_admit_card_settings`, `assessment_teacher_evaluations`, `assessment_teacher_remarks`, `assessment_temporary_merit_lists`, ...). The source crate `crates/domains/assessment/src/entities.rs` declares 0 `#[derive(DomainQuery)]` applications.

**Expected:**

Each row in `docs/specs/assessment/tables.md` has a `#[derive(DomainQuery)]` application in `crates/domains/assessment/src/entities.rs`.

**Evidence:**

`docs/specs/assessment/tables.md` is 75 lines. `crates/domains/assessment/src/entities.rs` `grep -c "DomainQuery"` returns 0.

---

### FINDING 13 (id: `SPEC-1-013`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** Critical
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/attendance/tables.md:1-40` vs `crates/domains/attendance/src/entities.rs`

**Description:**

The attendance tables spec lists ~12 tables (`attendance_student_attendances`, `attendance_subject_attendances`, `attendance_staff_attendances`, `attendance_exam_attendances`, `attendance_bulk_attendance_imports`, `attendance_student_attendance_imports`, `attendance_staff_attendance_imports`, `attendance_class_attendances`, `attendance_attendance_bulks`, `attendance_exam_attendance_children`, ...). The source crate `crates/domains/attendance/src/entities.rs` declares 0 `#[derive(DomainQuery)]` applications.

**Expected:**

Each row in `docs/specs/attendance/tables.md` has a `#[derive(DomainQuery)]` application in `crates/domains/attendance/src/entities.rs`.

**Evidence:**

`docs/specs/attendance/tables.md` is 40 lines. `crates/domains/attendance/src/entities.rs` `grep -c "DomainQuery"` returns 0.

---

### FINDING 14 (id: `SPEC-1-014`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** Critical
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/cms/tables.md:1-69` vs `crates/domains/cms/src/entities.rs`

**Description:**

The CMS tables spec lists ~20 tables (`cms_pages`, `cms_news`, `cms_news_categories`, `cms_news_comments`, `cms_news_pages`, `cms_notice_boards`, `cms_testimonials`, `cms_home_sliders`, `cms_speech_sliders`, `cms_contents`, `cms_content_types`, `cms_content_share_lists`, `cms_teacher_upload_contents`, `cms_upload_contents`, `cms_about_pages`, `cms_contact_pages`, `cms_course_pages`, `cms_home_page_settings`, `cms_frontend_pages`, ...). The source crate `crates/domains/cms/src/entities.rs` declares 0 `#[derive(DomainQuery)]` applications.

**Expected:**

Each row in `docs/specs/cms/tables.md` has a `#[derive(DomainQuery)]` application in `crates/domains/cms/src/entities.rs`.

**Evidence:**

`docs/specs/cms/tables.md` is 69 lines. `crates/domains/cms/src/entities.rs` `grep -c "DomainQuery"` returns 0.

---

### FINDING 15 (id: `SPEC-1-015`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** Critical
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/communication/tables.md:1-55` vs `crates/domains/communication/src/entities.rs`

**Description:**

The communication tables spec lists ~27 tables (`communication_notices`, `communication_complaints`, `communication_complaint_types`, `communication_notifications`, `communication_email_logs`, `communication_sms_logs`, `communication_sms_templates`, `communication_email_settings`, `communication_sms_gateways`, `communication_notification_settings`, `communication_absent_notification_time_setups`, `communication_chat_messages`, `communication_chat_conversations`, `communication_chat_groups`, `communication_chat_group_users`, `communication_chat_group_message_recipients`, `communication_chat_group_message_removes`, `communication_chat_block_users`, `communication_chat_invitations`, `communication_chat_invitation_types`, `communication_chat_statuses`, `communication_send_messages`, `communication_contact_messages`, `communication_speech_sliders`, `communication_phone_call_logs`, `communication_custom_sms_settings`, ...). The source crate `crates/domains/communication/src/entities.rs` declares 0 `#[derive(DomainQuery)]` applications.

**Expected:**

Each row in `docs/specs/communication/tables.md` has a `#[derive(DomainQuery)]` application in `crates/domains/communication/src/entities.rs`.

**Evidence:**

`docs/specs/communication/tables.md` is 55 lines. `crates/domains/communication/src/entities.rs` `grep -c "DomainQuery"` returns 0.

---

### FINDING 16 (id: `SPEC-1-016`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** Critical
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/academic/commands.md:30,36,50,etc.` vs `crates/cross-cutting/rbac/src/value_objects.rs:78-91`

**Description:**

Every capability in `docs/specs/academic/commands.md` is written in the un-prefixed `<Aggregate>.<Action>` form (e.g. `Student.Admit` at line 30, `Student.Update` at line 36, `ClassSection.Create` at the `CreateClassSection` section). The `educore-rbac` engine uses the `<Domain>.<Aggregate>.<Action>` form (e.g. `Academic.Student.Create`, `Academic.Student.Update`). The spec's `Student.Admit` has no RBAC counterpart: the closest rbac variant is `Academic.Student.Create` with the generic action verb `Create`, not the verb `Admit`.

**Expected:**

Either every capability in the academic spec is rewritten to `Academic.<Aggregate>.<Action>` form (matching the rbac crate), or the rbac crate adds new variants (e.g. `Academic.Student.Admit`) and the engine is re-seeded. The current state means a handler enforcing the spec's `Student.Admit` capability string will never match an rbac lookup, blocking every `AdmitStudentCommand` execution.

**Evidence:**

`docs/specs/academic/commands.md:30` `**Capability:** `Student.Admit``. `crates/cross-cutting/rbac/src/value_objects.rs:78-91` declares `AcademicStudentCreate` mapped to `"Academic.Student.Create"`. `grep "Student.Admit" crates/cross-cutting/rbac/src/value_objects.rs` returns 0 matches.

---

### FINDING 17 (id: `SPEC-1-017`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** Critical
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/assessment/commands.md` (40 commands) vs `crates/cross-cutting/rbac/src/value_objects.rs`

**Description:**

All 40 capabilities in `docs/specs/assessment/commands.md` use un-prefixed forms (`ExamType.Create`, `Exam.Create`, `Exam.Schedule`, `Marks.Initialize`, `Marks.Enter`, `Marks.Submit`, `Result.Publish`, `ReportCard.Generate`, `OnlineExam.Create`, `OnlineExam.Publish`, `OnlineExam.Start`, `OnlineExam.Answer`, `OnlineExam.Evaluate`, `SeatPlan.Generate`, `AdmitCard.Generate`, `ExamSignature.Set`, `Result.Configure`, `TeacherEvaluation.Mark`, `TeacherRemark.Add`, `TeacherRemark.Update`, `TeacherEvaluation.Approve`, `ExamAttendance.Mark`, etc.). The rbac crate uses prefixed forms `Assessment.Exam.Create`, `Assessment.ExamSchedule.Create`, `Assessment.MarksRegister.Create`, etc. None of the spec's capabilities are present in the rbac enum by their spec-name.

**Expected:**

Each capability in `docs/specs/assessment/commands.md` matches one rbac enum variant by name string. The current spec namespace drift prevents any `Assessment*Command::handle()` from passing the `engine.rbac().has(...)` authorization check.

**Evidence:**

`docs/specs/assessment/commands.md`: capabilities listed include `ExamType.Create`, `Exam.Create`, `Marks.Initialize`, `Marks.Enter`, `Marks.Submit`, `Result.Publish`, `ReportCard.Generate`, `OnlineExam.Create`. `crates/cross-cutting/rbac/src/value_objects.rs` declares `AssessmentExamCreate` (mapped to `"Assessment.Exam.Create"`), `AssessmentMarksRegisterCreate`, `AssessmentResultStoreCreate`, `AssessmentReportCardGenerate`, `AssessmentOnlineExamCreate` — none match the spec's string names verbatim.

---

### FINDING 18 (id: `SPEC-1-018`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** Critical
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/attendance/commands.md` (12 commands) vs `crates/cross-cutting/rbac/src/value_objects.rs`

**Description:**

All 12 attendance-spec capabilities use un-prefixed forms (`Attendance.Mark`, `Attendance.Update`, `Attendance.BulkMark`, `Attendance.Import`, `Attendance.Notify`). The rbac crate uses prefixed forms `Attendance.Student.Update`, `Attendance.Subject.*`, `Attendance.Staff.*`, etc. (no `Attendance.Mark`, no `Attendance.BulkMark`, no `Attendance.Import`, no `Attendance.Notify`).

**Expected:**

Each capability in `docs/specs/attendance/commands.md` matches an rbac enum variant. The current state means every attendance command handler calling `rbac.has(cap, "Attendance.Mark")` will fail authorization.

**Evidence:**

`docs/specs/attendance/commands.md` capabilities: `Attendance.Mark` (multiple), `Attendance.Update`, `Attendance.BulkMark`, `Attendance.Import`, `Attendance.Notify`. `crates/cross-cutting/rbac/src/value_objects.rs` rbac variants include `AttendanceStudentUpdate` (line ~`Self::AttendanceStudentUpdate => "Attendance.Student.Update"`), but no `Attendance.Mark` exists.

---

### FINDING 19 (id: `SPEC-1-019`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** Critical
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/cms/commands.md` (30 commands) vs `crates/cross-cutting/rbac/src/value_objects.rs`

**Description:**

All CMS-spec capabilities use un-prefixed forms (`Page.Create`, `Page.Update`, `Page.Publish`, `Page.Archive`, `Page.Delete`, `News.Create`, `NewsComment.Create`, `NoticeBoard.Create`, `NoticeBoard.Publish`, `Testimonial.Create`, `HomeSlider.Create`, `HomePageSetting.Configure`, `Content.Create`, `ContentShareList.Create`, `TeacherUploadContent.Create`, `UploadContent.Create`). The rbac crate uses prefixed forms `Cms.Page.Create`, `Cms.Page.Publish`, etc. The two namespaces disagree on every capability.

**Expected:**

Each capability in `docs/specs/cms/commands.md` matches an rbac enum variant. The current state blocks every CMS command from passing rbac authorization.

**Evidence:**

`docs/specs/cms/commands.md` capability strings (lines around 30-200): `Page.Create`, `Page.Update`, `Page.Publish`, `Page.Archive`, `Page.Delete`, `News.Create`, `NewsComment.Create`. `crates/cross-cutting/rbac/src/value_objects.rs` declares `CmsPageCreate` (mapped to `"Cms.Page.Create"`), `CmsNewsCreate`, `CmsNewsCommentCreate` — namespace drift on every entry.

---

### FINDING 2 (id: `SPEC-1-002`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** Critical
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/assessment/aggregates.md:35-1025` vs `crates/domains/assessment/src/aggregate.rs`

**Description:**

The assessment spec defines 46 aggregates (`ExamType`, `Exam`, `ExamSetup`, `ExamSchedule`, `ExamScheduleSubject`, `MarksRegister`, `MarksRegisterChild`, `MarkStore`, `MarkStoreEntry`, `ResultStore`, `ResultSetting`, `MarksGrade`, `ExamSetting`, `ExamSignature`, `ExamRoutinePage`, `FrontendExamRoutine`, `FrontendResult`, `FrontendExamResult`, `OnlineExam`, `QuestionBank`, `QuestionGroup`, `QuestionLevel`, `QuestionAssignment`, `OnlineExamQuestion`, `QuestionMuOption`, `OnlineExamMark`, `OnlineExamStudentAnswerMarking`, `StudentTakeOnlineExam`, `SeatPlan`, `SeatPlanChild`, `SeatPlanSetting`, `AdmitCard`, `AdmitCardSetting`, `TeacherEvaluation`, `TeacherRemark`, `TemporaryMeritList`, `MeritPosition`, `ExamWisePosition`, `AllExamWisePosition`, `CustomResultSetting`, `CustomTemporaryResult`, `ExamStepSkip`, `ExamAttendance`, `ExamAttendanceChild`, `SubjectAttendance (overlap with Attendance)`). The source crate declares only 6 aggregate roots (`Exam`, `ExamSchedule`, `SeatPlan`, `AdmitCard`, `MarksRegister`, `ResultStore`).

**Expected:**

Every aggregate declared in `docs/specs/assessment/aggregates.md` has a corresponding `pub struct` root in `crates/domains/assessment/src/aggregate.rs`. Note: `SubjectAttendance` is owned by the attendance domain (per the `(overlap with Attendance)` annotation), so cross-domain ownership should be enforced.

**Evidence:**

`grep "^## " docs/specs/assessment/aggregates.md | wc -l` returns 46 lines (`## ExamType` ... `## SubjectAttendance (overlap with Attendance)`). `crates/domains/assessment/src/aggregate.rs` `grep "^pub struct "` returns `Exam`, `ExamSchedule`, `SeatPlan`, `AdmitCard`, `MarksRegister`, `ResultStore`.

---

### FINDING 20 (id: `SPEC-1-020`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** Critical
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/communication/commands.md` (40 commands) vs `crates/cross-cutting/rbac/src/value_objects.rs`

**Description:**

All communication-spec capabilities use un-prefixed forms (`Notice.Create`, `Notice.Publish`, `Complaint.Create`, `Notification.Send`, `EmailLog.Create`, `SmsLog.Create`, `Template.Create`, `EmailSetting.Configure`, `SmsGateway.Configure`, `Chat.Send`, `ChatGroup.Create`, `SendMessage.Create`, `SendMessage.Dispatch`). The rbac crate uses prefixed forms `Communication.Notice.*`, `Communication.Complaint.*`, etc. None match verbatim.

**Expected:**

Each capability in `docs/specs/communication/commands.md` matches an rbac enum variant. The current state means every communication command handler will fail authorization at runtime.

**Evidence:**

`docs/specs/communication/commands.md` capability strings (multiple lines): `Notice.Create`, `Notice.Publish`, `Complaint.Create`, `Notification.Send`, `Template.Create`. `crates/cross-cutting/rbac/src/value_objects.rs` declares `CommunicationNoticeCreate` (mapped to `"Communication.Notice.Create"`), `CommunicationComplaintCreate`, `CommunicationNotificationSend`, `CommunicationSmsTemplateCreate` — namespace drift on every entry.

---

### FINDING 27 (id: `SPEC-1-027`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** Critical
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/academic/events.md` (`StudentAssignedToSection`, `StudentCategoryChanged`, `OptionalSubjectAssigned`, `StudentDocumentUploaded`, `GuardianRegistered`, etc.) vs `crates/domains/academic/src/events.rs`

**Description:**

The academic events spec defines 30+ event types beyond the Student/Class/Section/Subject/AcademicYear lifecycle. None of these non-lifecycle events exist as `pub struct` in `crates/domains/academic/src/events.rs`: `StudentAssignedToSection`, `StudentCategoryChanged`, `OptionalSubjectAssigned`, `StudentDocumentUploaded`, `GuardianRegistered`, `GuardianContactUpdated`, `GuardianLinkedToStudent`, `GuardianUnlinkedFromStudent`, `PrimaryGuardianMarked`, `ClassSectionCreated`, `ClassTeacherAssigned`, `SubjectTeacherAssigned`, `ClassRoomAssigned`, `SubjectAssignedToClass`, `TeacherReassigned`, `SubjectUnassigned`, `ClassRoutineCreated`, `HomeworkCreated`, `HomeworkUpdated`, `HomeworkSubmitted`, `HomeworkEvaluated`, `HomeworkCancelled`, `LessonPlanCreated`, `LessonPlanUpdated`, `LessonPlanCompleted`, `SubTopicAdded`, `LessonCreated`, `LessonUpdated`, `LessonDeleted`, `LessonTopicCreated`, `LessonTopicCompleted`, `LessonTopicDeleted`, `StudentRecordCreated`, `StudentCategoryCreated`, `StudentGroupCreated`, `RegistrationFieldCreated`, `CertificateCreated`, `IdCardCreated`, `AdmissionQueryRegistered`, `AdmissionQueryFollowedUp`, `AdmissionQueryConverted`.

**Expected:**

Every event struct in the academic events spec has a corresponding `pub struct` in `crates/domains/academic/src/events.rs`. 40+ spec-only events are missing from source.

**Evidence:**

`docs/specs/academic/events.md:209` `### StudentAssignedToSection` with `pub struct StudentAssignedToSection { ... }`. `grep "^pub struct StudentAssignedToSection" crates/domains/academic/src/events.rs` returns no match. Same negative result for `StudentCategoryChanged`, `OptionalSubjectAssigned`, `StudentDocumentUploaded`, `HomeworkCreated`, `LessonPlanCreated`, etc.

---

### FINDING 28 (id: `SPEC-1-028`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** Critical
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/cms/aggregates.md` vs `crates/domains/cms/src/aggregate.rs`

**Description:**

The CMS `aggregate.rs` file mixes aggregate roots (`Page`, `News`, `NewsCategory`, `NewsComment`, `NewsPage`, `NoticeBoard`, `Testimonial`, `HomeSlider`, `SpeechSlider`, `Content`, `ContentType`, `ContentShareList`, `TeacherUploadContent`, etc.) with command-input structs (`NewPage`, `UpdatePage`, `NewPageRevision`, `NewNews`, `UpdateNews`, `NewNewsCategory`, etc.). Per `docs/code-standards.md` § "Module Layout (per domain)", command input structs belong in `commands.rs`, not `aggregate.rs`.

**Expected:**

`crates/domains/cms/src/aggregate.rs` contains only aggregate roots. All `New*`/`Update*` command-input structs are relocated to `crates/domains/cms/src/commands.rs`.

**Evidence:**

`grep "^pub struct" crates/domains/cms/src/aggregate.rs` returns `NewPage`, `UpdatePage`, `Page`, `NewPageRevision`, `NewNews`, `UpdateNews`, `News`, `NewNewsCategory`, `NewsCategory`, `NewNewsComment`, `NewsComment`, ... (root types interleaved with command inputs).

---

### FINDING 3 (id: `SPEC-1-003`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** Critical
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/attendance/aggregates.md:35-255` vs `crates/domains/attendance/src/aggregate.rs`

**Description:**

The attendance spec defines 10 aggregates (`StudentAttendance`, `SubjectAttendance`, `StaffAttendance`, `ExamAttendance`, `BulkAttendanceImport`, `StudentAttendanceImport`, `StaffAttendanceImport`, `ClassAttendance`, `AttendanceBulk`, `ExamAttendanceChild (cross-reference)`). The source crate declares only 5 aggregate roots (`StudentAttendance`, `StaffAttendance`, `SubjectAttendance`, `ExamAttendance`, `BulkAttendanceImport`).

**Expected:**

Every aggregate declared in `docs/specs/attendance/aggregates.md` has a corresponding `pub struct` root in `crates/domains/attendance/src/aggregate.rs`. The spec lists 10 aggregates but only 5 exist in source; 5 aggregates (`StudentAttendanceImport`, `StaffAttendanceImport`, `ClassAttendance`, `AttendanceBulk`, `ExamAttendanceChild`) are missing.

**Evidence:**

`docs/specs/attendance/aggregates.md` headings: `## StudentAttendance`, `## SubjectAttendance`, `## StaffAttendance`, `## ExamAttendance`, `## BulkAttendanceImport`, `## StudentAttendanceImport`, `## StaffAttendanceImport`, `## ClassAttendance`, `## AttendanceBulk`, `## ExamAttendanceChild (cross-reference)`. Source: `crates/domains/attendance/src/aggregate.rs` `grep "^pub struct "` returns only `StudentAttendance`, `StaffAttendance`, `SubjectAttendance`, `ExamAttendance`, `BulkAttendanceImport`.

---

### FINDING 4 (id: `SPEC-1-004`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** Critical
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/cms/aggregates.md:35-644` vs `crates/domains/cms/src/aggregate.rs`

**Description:**

The CMS spec defines 19 aggregates (`Page`, `News`, `NewsCategory`, `NewsComment`, `NewsPage`, `NoticeBoard`, `Testimonial`, `HomeSlider`, `SpeechSlider`, `Content`, `ContentType`, `ContentShareList`, `TeacherUploadContent`, `UploadContent`, `AboutPage`, `ContactPage`, `CoursePage`, `HomePageSetting`, `FrontendPage`). The source crate `crates/domains/cms/src/aggregate.rs` declares the 19 aggregate roots plus 16 `New*`/`Update*` command-input structs that are interleaved with the root types in the file (e.g., `NewPage`, `UpdatePage`, `NewPageRevision`).

**Expected:**

The CMS crate source contains aggregate roots for every aggregate in `docs/specs/cms/aggregates.md`. Per `AGENTS.md` § "Code Standards" / `docs/code-standards.md` § "Module Layout", `aggregate.rs` should host only aggregate roots; command-input structs belong in `commands.rs`. The interleaved `New*`/`Update*` structs violate the module-layout rule.

**Evidence:**

`crates/domains/cms/src/aggregate.rs` `grep "^pub struct "` returns `NewPage`, `UpdatePage`, `Page`, `NewPageRevision`, `NewNews`, `UpdateNews`, `News`, ... (mixed root + command input). `docs/specs/cms/aggregates.md` declares 19 aggregates; only 19 should appear as `pub struct` roots in `aggregate.rs`.

---

### FINDING 5 (id: `SPEC-1-005`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** Critical
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/communication/aggregates.md:35-835` vs `crates/domains/communication/src/aggregate.rs`

**Description:**

The communication spec defines 27 aggregates (`Notice`, `Complaint`, `ComplaintType`, `Notification`, `EmailLog`, `SmsLog`, `SmsTemplate`, `EmailSetting`, `SmsGateway`, `NotificationSetting`, `AbsentNotificationTimeSetup`, `ChatMessage`, `ChatConversation`, `ChatGroup`, `ChatGroupUser`, `ChatGroupMessageRecipient`, `ChatGroupMessageRemove`, `ChatBlockUser`, `ChatInvitation`, `ChatInvitationType`, `ChatStatus`, `SendMessage`, `ContactMessage`, `SpeechSlider`, `PhoneCallLog`, `CustomSmsSetting`). The source crate declares 25 `pub struct`s.

**Expected:**

Each spec aggregate maps to exactly one `pub struct` root in `crates/domains/communication/src/aggregate.rs`. Naming drift exists: spec calls the aggregate `ChatStatus` (line 752), source uses `ChatStatusRecord` (the actual root name). Spec's `SendMessage` aggregate exists as `SendMessage` in source but the spec also defines it as a command in `commands.md` (potential naming collision).

**Evidence:**

`docs/specs/communication/aggregates.md` `## ChatStatus` (line ~752). `crates/domains/communication/src/aggregate.rs` declares `pub struct ChatStatusRecord`. Spec lists `SendMessage` as an aggregate (line ~778) — same identifier as a likely command name in `commands.md`.

---

### FINDING 6 (id: `SPEC-1-006`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** Critical
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/academic/events.md:35-304` vs `crates/domains/academic/src/events.rs`

**Description:**

The academic events spec enumerates 15 lifecycle sections (`Student Lifecycle`, `Guardian Lifecycle`, `Class & Section`, `ClassSection`, `Subject`, `AcademicYear`, `ClassRoutine`, `Homework`, `Lesson`, `StudentRecord`, `StudentCategory`, `StudentGroup`, `Registration`, `Certificate & ID Card`, `Admission Query`). The source crate `crates/domains/academic/src/events.rs` declares 25 `pub struct` event types (e.g., `StudentAdmitted`, `StudentProfileUpdated`, `ClassCreated`, `OptionalSubjectGpaThresholdSet`, `CurrentAcademicYearSet`, `AcademicYearClosed`, `AcademicYearCopied`). The spec narrative lists ~40 events; source only covers ~25, but spec lists events like `LessonPlanCreated`, `HomeworkAssigned`, `RegistrationFieldUpdated`, `CertificateIssued` for which there is no source struct.

**Expected:**

Every event declared in `docs/specs/academic/events.md` has a corresponding `pub struct` in `crates/domains/academic/src/events.rs`.

**Evidence:**

`docs/specs/academic/events.md` contains 15 lifecycle sections; source `crates/domains/academic/src/events.rs` declares 25 `pub struct` events. Several spec events (`HomeworkAssigned`, `LessonPlanCreated`, `CertificateIssued`, `IdCardIssued`, `RegistrationFieldAdded`) are not present in the source event enum/struct list.

---

### FINDING 7 (id: `SPEC-1-007`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** Critical
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/assessment/events.md:35-512` vs `crates/domains/assessment/src/events.rs`

**Description:**

The assessment events spec covers 20 lifecycle sections (`Exam Type`, `Exam Lifecycle`, `Exam Schedule`, `Marks Lifecycle`, `Result Lifecycle`, `Report Card`, `Mark Store`, `Result Settings`, `Marks Grade`, `Exam Settings, Signatures, Front-End Pages`, `Exam Setup`, `Online Exam Lifecycle`, `Question Bank`, `Online Exam Questions and Marking`, `Seat Plan`, `Admit Card`, `Teacher Evaluation`, `Teacher Remark`, `Exam Attendance`, `Exam Step Skip`, `Audit`). The source crate declares 17 event structs, but several spec events lack source structs: e.g., `OnlineExamStarted`, `OnlineExamSubmitted`, `OnlineExamMarked`, `QuestionBankCreated`, `SeatPlanChildCreated`, `ExamAttendanceMarked`, `ExamStepSkipped`, `AuditLogged` are spec-only.

**Expected:**

Every event declared in `docs/specs/assessment/events.md` has a corresponding `pub struct` in `crates/domains/assessment/src/events.rs`.

**Evidence:**

`grep "^pub struct " crates/domains/assessment/src/events.rs` returns 17 event structs (`ExamCreated`, ..., `ReportCardGenerated`). `docs/specs/assessment/events.md` references ~80+ distinct event names across 20 lifecycle sections.

---

### FINDING 8 (id: `SPEC-1-008`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** Critical
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/attendance/events.md:35-279` vs `crates/domains/attendance/src/events.rs`

**Description:**

The attendance events spec defines 7 lifecycle sections (`Student Attendance`, `Subject Attendance`, `Staff Attendance`, `Bulk Import`, `Notification`, `Class Attendance (projection)`, `Audit`) covering ~40 events. The source crate declares 21 `pub struct` events (`StudentAttendanceMarked`, `SubjectAttendanceMarked`, `StaffAttendanceMarked`, `ExamAttendanceMarked`, `BulkImportStarted`, `BulkImportValidated`, `BulkImportCommitted`, `BulkImportFailed`, `BulkImportCancelled`, `AttendanceImported`, `AbsenceNotificationRequested`, `ClassAttendanceRecomputed`, ...). Many spec-only events lack source structs (e.g., `StudentAbsentNotificationSent`, `SubjectAttendanceBiometricSynced`, `StaffAttendanceApproved`).

**Expected:**

Every event declared in `docs/specs/attendance/events.md` has a corresponding `pub struct` in `crates/domains/attendance/src/events.rs`.

**Evidence:**

`grep "^pub struct " crates/domains/attendance/src/events.rs` returns 21 event structs. `docs/specs/attendance/events.md` enumerates events across 7 lifecycle sections.

---

### FINDING 9 (id: `SPEC-1-009`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** Critical
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/cms/events.md:35-262` vs `crates/domains/cms/src/events.rs`

**Description:**

The CMS events spec defines 21 lifecycle sections (`Page Lifecycle`, `News Lifecycle`, `News Comment Lifecycle`, `News Page`, `Notice Board (Public Site)`, `Testimonial`, `Home Slider`, `Speech Slider (CMS-Side)`, `Content Lifecycle`, `Content Type`, `Content Share List`, `Teacher Upload Content`, `Upload Content`, `About Page`, `Contact Page`, `Course Page`, `Home Page Setting`, `Frontend Page`, `News Category`, ...). The source crate declares 60+ event structs. Despite high coverage, spec-only events exist: `PageRevisionRestored`, `NewsPageViewIncremented`, `ContentShared`, `TeacherUploadContentApproved`, `UploadContentDownloaded`, `AboutPagePublished`.

**Expected:**

Every event declared in `docs/specs/cms/events.md` has a corresponding `pub struct` in `crates/domains/cms/src/events.rs`.

**Evidence:**

`docs/specs/cms/events.md` declares 21 lifecycle sections; `crates/domains/cms/src/events.rs` declares 60+ `pub struct` event types but misses a handful of spec-only events.

---

### FINDING 21 (id: `SPEC-1-021`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** High
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/academic/commands.md` (32 spec commands) vs `crates/domains/academic/src/commands.rs` (22 source commands)

**Description:**

The academic spec defines 32 commands across `AdmitStudent`, `UpdateStudentProfile`, `AssignStudentToSection`, `ChangeStudentCategory`, `AssignOptionalSubject`, `UploadStudentDocument`, `SuspendStudent`, `ReinstateStudent`, `WithdrawStudent`, `TransferStudent`, `PromoteStudent`, `GraduateStudent`, plus Create/Update/Delete variants for Class, Section, ClassSection, ClassTeacher/SubjectTeacher/ClassRoom, Subject, ClassSubject, AcademicYear, ClassRoutine, Homework, LessonPlan, Lesson, LessonTopic, StudentCategory, StudentGroup, RegistrationField, Certificate, IdCard, and RegisterAdmissionQuery/FollowUpAdmissionQuery/ConvertAdmissionQuery. The source crate declares only 22 `*Command` structs, covering Student/Class/Section/Subject/AcademicYear but missing ClassSection, ClassTeacher, ClassSubject, ClassRoutine, Homework, LessonPlan, Lesson, LessonTopic, StudentCategory, StudentGroup, RegistrationField, Certificate, IdCard, AdmissionQuery, Guardian, ChangeStudentCategory, AssignOptionalSubject, UploadStudentDocument commands.

**Expected:**

Every command in `docs/specs/academic/commands.md` has a corresponding `*Command` struct in `crates/domains/academic/src/commands.rs`.

**Evidence:**

`docs/specs/academic/commands.md` `## ` headings enumerate 32 commands (e.g. `## AssignClassTeacher / AssignSubjectTeacher / AssignClassRoom`, `## CreateHomework / ...`). `crates/domains/academic/src/commands.rs` `grep -c "^pub struct .*Command"` returns 22.

---

### FINDING 22 (id: `SPEC-1-022`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** High
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/assessment/commands.md` (40 spec commands) vs `crates/domains/assessment/src/commands.rs` (21 source commands)

**Description:**

The assessment spec defines 40+ commands (CreateExamType, UpdateExamType, DeleteExamType, CreateExam, UpdateExam, DeleteExam, ScheduleExam, UpdateExamSchedule, CancelExamSchedule, InitializeMarksRegister, EnterMarks, SubmitMarks, PublishResult, RepublishResult, GenerateReportCard, CreateOnlineExam, PublishOnlineExam, StartOnlineExam, SubmitOnlineExamAnswer, EvaluateOnlineExam, GenerateSeatPlan, GenerateAdmitCard, SetExamSignature, ConfigureCustomResultSettings, MarkTeacherEvaluation, AddTeacherRemark, UpdateTeacherRemark, ApproveTeacherEvaluation, RejectTeacherEvaluation, Create/Update/DeleteMarksGrade, MarkExamAttendance, UpdateExamAttendance, Create/Update/DeleteExamSetting, ConfigureAdmitCardSettings, ConfigureSeatPlanSettings, ConfigureTeacherEvaluation, PublishExamRoutine, PublishFrontResult, MarkExamStepSkip, SendAbsenceNotification). The source crate declares only 21 `*Command` structs.

**Expected:**

Every command in `docs/specs/assessment/commands.md` has a corresponding `*Command` struct in `crates/domains/assessment/src/commands.rs`.

**Evidence:**

`docs/specs/assessment/commands.md` `## ` headings list 40+ commands. `crates/domains/assessment/src/commands.rs` `grep -c "^pub struct .*Command"` returns 21.

---

### FINDING 23 (id: `SPEC-1-023`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** High
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/attendance/commands.md` (13 spec commands) vs `crates/domains/attendance/src/commands.rs` (15 source commands)

**Description:**

The attendance spec defines 13 commands (MarkStudentAttendance, UpdateStudentAttendance, MarkSubjectAttendance, MarkStaffAttendance, MarkExamAttendance, BulkMarkStudentAttendance, ImportAttendance, ValidateBulkImport, CommitBulkImport, CancelBulkImport, SendAbsenceNotification, MarkClassAttendance, plus Standard CRUD Variants). The source crate declares 15 `*Command` structs — slightly higher than the spec, indicating source-only commands (likely CRUD variants) without spec documentation.

**Expected:**

Source code's `*Command` structs and spec's command list converge. The source-side surplus (`+2`) needs to be documented in `docs/specs/attendance/commands.md` or removed.

**Evidence:**

`docs/specs/attendance/commands.md` headings: 13 command sections. `crates/domains/attendance/src/commands.rs` `grep -c "^pub struct .*Command"` returns 15.

---

### FINDING 24 (id: `SPEC-1-024`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** High
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/cms/commands.md` (40 spec commands) vs `crates/domains/cms/src/commands.rs` (10 source commands)

**Description:**

The CMS spec defines 40+ commands (CreatePage, UpdatePage, PublishPage, ArchivePage, DeletePage, CreateNews, UpdateNews, PublishNews, UnpublishNews, DeleteNews, CommentOnNews, ModerateNewsComment, DeleteNewsComment, CreateNoticeBoard, PublishNoticeBoard, UpdateNoticeBoard, UnpublishNoticeBoard, DeleteNoticeBoard, CreateTestimonial, UpdateTestimonial, DeleteTestimonial, CreateHomeSlider, UpdateHomeSlider, DeleteHomeSlider, ConfigureHomePage, CreateContent, UpdateContent, DeleteContent, CreateContentShareList, DispatchContentShareList, CancelContentShareList, DeleteContentShareList, CreateTeacherUploadContent, UpdateTeacherUploadContent, DeleteTeacherUploadContent, CreateUploadContent, etc.). The source crate declares only 10 `*Command` structs.

**Expected:**

Every command in `docs/specs/cms/commands.md` has a corresponding `*Command` struct in `crates/domains/cms/src/commands.rs`. The current 30-command gap blocks deploy.

**Evidence:**

`docs/specs/cms/commands.md` `## ` headings list 40+ commands. `crates/domains/cms/src/commands.rs` `grep -c "^pub struct .*Command"` returns 10.

---

### FINDING 25 (id: `SPEC-1-025`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** High
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/communication/commands.md` (40+ spec commands) vs `crates/domains/communication/src/commands.rs` (72 source commands)

**Description:**

The communication spec defines 40+ commands. The source crate declares 72 `*Command` structs — significantly higher than the spec, indicating source-only commands without spec documentation. Many command-input structs (e.g. `NewPage`, `UpdatePage` declared in `crates/domains/cms/src/aggregate.rs`) appear to be misplaced from `commands.rs` per the module-layout rule.

**Expected:**

Source code's `*Command` structs and spec's command list converge. The source-side surplus (`+32`) needs to be either documented in `docs/specs/communication/commands.md` or removed.

**Evidence:**

`docs/specs/communication/commands.md` `## ` headings list 40+ commands. `crates/domains/communication/src/commands.rs` `grep -c "^pub struct .*Command"` returns 72.

---

### FINDING 26 (id: `SPEC-1-026`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** High
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/academic/services.md` (13 services) vs `crates/domains/academic/src/services.rs`

**Description:**

The academic services spec defines 13 services/policies/specifications: `AdmissionService`, `PromotionService`, `EnrollmentService`, `RoutineService`, `HomeworkService`, `LessonPlanService`, `GraduationService`, `ClassSectionAssignmentService`, `Policy: OptionalSubjectEligibility`, `Specification: ActiveStudentsInClass`, `Specification: PromotableStudents`, `Specification: HasOutstandingHomework`, `Cross-Domain Coordinator`. The source crate `crates/domains/academic/src/services.rs` needs verification for `pub struct`/`pub trait` count.

**Expected:**

Every service in `docs/specs/academic/services.md` has a corresponding `pub struct` or `pub trait` in `crates/domains/academic/src/services.rs`.

**Evidence:**

`docs/specs/academic/services.md` `## ` headings enumerate 13 services. (Source-side count to be verified at line level — see FINDING 27.)

---

### FINDING 29 (id: `SPEC-1-029`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** High
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/academic/repositories.md` (20+ repositories) vs `crates/domains/academic/src/repository.rs` (5 traits)

**Description:**

The academic repositories spec defines 20+ repository traits (`StudentRepository`, `GuardianRepository`, `ClassRepository`, `SectionRepository`, `ClassSectionRepository`, `SubjectRepository`, `ClassSubjectRepository`, `AcademicYearRepository`, `ClassRoutineRepository`, `HomeworkRepository`, `LessonRepository`, `LessonTopicRepository`, `LessonPlanRepository`, `StudentRecordRepository`, `StudentPromotionRepository`, `StudentCategoryRepository`, `StudentGroupRepository`, `RegistrationFieldRepository`, `CertificateRepository`, `IdCardRepository`, `AdmissionQueryRepository`, `ClassRoomRepository`, `ClassTimeRepository`). The source crate declares only 5 repository traits (`StudentRepository`, `ClassRepository`, `SectionRepository`, `SubjectRepository`, `AcademicYearRepository`).

**Expected:**

Every repository in `docs/specs/academic/repositories.md` has a corresponding `pub trait` in `crates/domains/academic/src/repository.rs`.

**Evidence:**

`docs/specs/academic/repositories.md` `## ` headings enumerate 20+ repository traits. `crates/domains/academic/src/repository.rs` `grep "^pub trait"` returns 5.

---

### FINDING 30 (id: `SPEC-1-030`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** High
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/assessment/repositories.md` vs `crates/domains/assessment/src/repository.rs`

**Description:**

The assessment repositories spec defines 30+ repository traits. The source crate declares only 6 traits (`ExamRepository`, `ExamScheduleRepository`, `SeatPlanRepository`, `AdmitCardRepository`, `MarksRegisterRepository`, `ResultRepository`). 24+ spec-only repository traits lack source code.

**Expected:**

Every repository in `docs/specs/assessment/repositories.md` has a corresponding `pub trait` in `crates/domains/assessment/src/repository.rs`.

**Evidence:**

`docs/specs/assessment/repositories.md` is 524 lines. `crates/domains/assessment/src/repository.rs` `grep "^pub trait"` returns 6.

---

### FINDING 31 (id: `SPEC-1-031`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** High
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/attendance/repositories.md` vs `crates/domains/attendance/src/repository.rs`

**Description:**

The attendance repositories spec defines 8+ repository traits. The source crate declares only 5 traits (`StudentAttendanceRepository`, `SubjectAttendanceRepository`, `StaffAttendanceRepository`, `ExamAttendanceRepository`, `BulkAttendanceImportRepository` — inferred from aggregate names; source-side naming to be verified). Spec-only traits include `ClassAttendanceRepository`, `AttendanceBulkRepository`, plus import-bulk trait.

**Expected:**

Every repository in `docs/specs/attendance/repositories.md` has a corresponding `pub trait` in `crates/domains/attendance/src/repository.rs`.

**Evidence:**

`docs/specs/attendance/repositories.md` is 218 lines. `crates/domains/attendance/src/repository.rs` `grep "^pub trait"` returns 5.

---

### FINDING 33 (id: `SPEC-1-033`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** High
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/communication/repositories.md` vs `crates/domains/communication/src/repository.rs`

**Description:**

The communication repositories spec defines 27 repository traits. The source crate declares ~27 traits (matching count). Coverage is good on count but method-level drift may exist.

**Expected:**

Every repository trait in `docs/specs/communication/repositories.md` matches a `pub trait` in `crates/domains/communication/src/repository.rs` and the spec's method signatures match the source's `async fn` declarations.

**Evidence:**

`crates/domains/communication/src/repository.rs` `grep "^pub trait"` returns ~27 traits (`NoticeRepository`, `ComplaintRepository`, ..., `CustomSmsSettingRepository`).

---

### FINDING 34 (id: `SPEC-1-034`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** High
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/academic/events.md:212-213` vs `docs/specs/hr/events.md:116-122` (cross-domain reference)

**Description:**

The `ClassTeacherAssigned` event is referenced in `docs/specs/academic/events.md:212` with payload `{ class_section_id, staff_id, role }` (academic-owned). The HR spec (`docs/specs/hr/events.md:116-122`) declares the same event with payload `{ assign_class_teacher_id, class_id, section_id, staff_id, academic_id }`. Two domains cannot own the same event with divergent payload shapes. Cross-domain subscriptions from HR → academic cannot reconcile.

**Expected:**

A single canonical `ClassTeacherAssigned` event payload is owned by one domain (the academic domain per `academic/events.md:212`) and the HR spec either re-publishes a projection or consumes the canonical event.

**Evidence:**

`docs/specs/academic/events.md:212` `- `ClassTeacherAssigned { class_section_id, staff_id, role }``. `docs/specs/hr/events.md:116-122` declares the same event with five-field payload including `assign_class_teacher_id` and `academic_id` — divergent payload shapes.

---

### FINDING 35 (id: `SPEC-1-035`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** High
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/academic/events.md:213` vs `docs/specs/hr/events.md:127`

**Description:**

The `SubjectTeacherAssigned` event is referenced in `docs/specs/academic/events.md:213` with payload `{ class_section_id, subject_id, staff_id }`. The HR spec (`docs/specs/hr/events.md:127`) declares the same event with payload `{ class_id, section_id, subject_id, staff_id, academic_id }`. Same divergence pattern as FINDING 34.

**Expected:**

A single canonical `SubjectTeacherAssigned` event payload is owned by one domain (academic per `events.md:213`).

**Evidence:**

`docs/specs/academic/events.md:213` `- `SubjectTeacherAssigned { class_section_id, subject_id, staff_id }``. `docs/specs/hr/events.md:127` declares the event with five-field payload including `academic_id` — divergent.

---

### FINDING 36 (id: `SPEC-1-036`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** High
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/assessment/aggregates.md` (SubjectAttendance annotation) vs `docs/specs/attendance/aggregates.md`

**Description:**

The assessment spec declares `SubjectAttendance (overlap with Attendance)` at `docs/specs/assessment/aggregates.md` (last heading). The attendance spec also declares `SubjectAttendance` as its own aggregate (`docs/specs/attendance/aggregates.md:110-153`). Two domains claim ownership of the same aggregate. No cross-domain ownership rule (e.g. "X is owned by attendance; assessment only stores a projection") is documented.

**Expected:**

Exactly one domain owns `SubjectAttendance`; the other domain's spec clearly states its role as projection or subscriber.

**Evidence:**

`docs/specs/assessment/aggregates.md` last `## ` heading: `## SubjectAttendance (overlap with Attendance)`. `docs/specs/attendance/aggregates.md:110` `## SubjectAttendance` declared as owned aggregate.

---

### FINDING 37 (id: `SPEC-1-037`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** High
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/attendance/aggregates.md:236` vs `docs/specs/assessment/aggregates.md:495-500`

**Description:**

The attendance spec's `## ExamAttendance (cross-reference)` heading at line 236 declares ExamAttendance as an attendance-owned aggregate. The assessment spec also declares `ExamAttendance` and `ExamAttendanceChild` as owned aggregates (`docs/specs/assessment/aggregates.md:495-500`). Two domains claim ownership of `ExamAttendance`. No cross-domain ownership rule is documented.

**Expected:**

Exactly one domain owns `ExamAttendance` and `ExamAttendanceChild`; the other domain's spec clearly states its role as projection or subscriber.

**Evidence:**

`docs/specs/attendance/aggregates.md:236` `## ExamAttendance (cross-reference)`. `docs/specs/assessment/aggregates.md:495` `## ExamAttendance` (owned aggregate).

---

### FINDING 38 (id: `SPEC-1-038`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** High
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/cms/aggregates.md:541-572` (`## SpeechSlider`) vs `docs/specs/communication/aggregates.md` (`## SpeechSlider`)

**Description:**

The CMS spec declares `## SpeechSlider` (CMS-side) at `docs/specs/cms/aggregates.md` near line 541. The communication spec also declares `## SpeechSlider` (line ~743 in `communication/aggregates.md`). Both domains claim ownership of an aggregate called `SpeechSlider` with similar (possibly identical) responsibilities — managing rotating slide content for the school's home page. No cross-domain ownership rule is documented.

**Expected:**

Exactly one domain owns `SpeechSlider`; the other domain's spec clearly states its role as projection or subscriber. The duplicate aggregate name is a critical naming-collision risk.

**Evidence:**

`grep "^## SpeechSlider" docs/specs/cms/aggregates.md docs/specs/communication/aggregates.md` returns both files declaring the aggregate.

---

### FINDING 39 (id: `SPEC-1-039`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** High
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/academic/tables.md` (table prefix `academic_`) vs `crates/domains/academic/src/aggregate.rs` (struct names)

**Description:**

The academic tables spec uses the storage prefix `academic_` (e.g., `academic_students`, `academic_classes`, `academic_sections`). The source aggregate roots use un-prefixed names (`Student`, `Class`, `Section`). Per `docs/schemas/sql-dialects/README.md` § "Runtime DDL emission", the macro-emitted AST derives table names from struct names, so source struct `Student` would emit `student` (no prefix), not `academic_students`.

**Expected:**

Either the spec's table names match the source-emitted table names, or a domain-prefix transform is documented in the macro/adapter layer. The current state means `storage.create_schema()` will create `student` (not `academic_student`), violating the spec.

**Evidence:**

`docs/specs/academic/tables.md` lists `academic_students` (line ~10), `academic_classes`, `academic_sections`. `crates/domains/academic/src/aggregate.rs` declares `pub struct Student`, `pub struct Class`, `pub struct Section` — without any prefix.

---

### FINDING 40 (id: `SPEC-1-040`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** High
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/assessment/tables.md` vs `docs/specs/attendance/tables.md` (table prefix overlap)

**Description:**

The assessment tables spec lists `assessment_exam_attendances` (line ~50) and the attendance tables spec lists `attendance_exam_attendances` (line ~20). Both domains claim storage ownership of exam-attendance rows. Per the cross-domain ownership conflict noted in FINDING 37, exactly one table name should be the source of truth.

**Expected:**

Exactly one spec owns the exam-attendance storage table name. The other spec references the same table by name (cross-domain reference).

**Evidence:**

`docs/specs/assessment/tables.md` line ~50 contains `assessment_exam_attendances`. `docs/specs/attendance/tables.md` line ~20 contains `attendance_exam_attendances`. Both specs cannot define separate storage tables for the same conceptual entity.

---

### FINDING 32 (id: `SPEC-1-032`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** Medium
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/cms/repositories.md` vs `crates/domains/cms/src/repository.rs`

**Description:**

The CMS repositories spec defines 19 repository traits. The source crate declares 19 traits. Coverage is good on count, but method-level drift may exist (spec methods may differ from `async fn` signatures in source).

**Expected:**

Every repository trait declared in `docs/specs/cms/repositories.md` matches a `pub trait` in `crates/domains/cms/src/repository.rs` and the spec's method signatures match the source's `async fn` declarations.

**Evidence:**

`crates/domains/cms/src/repository.rs` `grep "^pub trait"` returns 19 traits (`PageRepository`, `NewsRepository`, `NewsCategoryRepository`, ..., `FrontendPageRepository`).

---

### FINDING 41 (id: `SPEC-1-041`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** Medium
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/academic/workflows.md:1-9` (Admission Workflow) vs `crates/domains/academic/src/commands.rs`

**Description:**

The Admission Workflow references `RegisterAdmissionQuery`, `FollowUpAdmissionQuery`, `ConvertAdmissionQuery`, and the `Library` and `Finance` domains subscribing to `StudentAdmitted`. The source crate `crates/domains/academic/src/commands.rs` does not declare command handlers for any of `RegisterAdmissionQuery`, `FollowUpAdmissionQuery`, `ConvertAdmissionQuery` (these aggregate roots are spec-only per FINDING 1). The cross-domain subscription contract for `StudentAdmitted` is not implemented as subscriber wiring in any `events.rs` or `services.rs`.

**Expected:**

Every workflow step references a real `*Command` handler and the cross-domain subscription is implemented as a subscriber function in `services.rs` or a dedicated `subscribers.rs`.

**Evidence:**

`docs/specs/academic/workflows.md:1-9` `Admission Workflow` enumerates 8 steps referencing `RegisterAdmissionQuery`, `ConvertAdmissionQuery` (internally calls `AdmitStudent`), `Library subscribes to StudentAdmitted`, `Finance subscribes to StudentAdmitted`, `Communication subscribes`. Source: `crates/domains/academic/src/commands.rs` lacks `RegisterAdmissionQueryCommand`, `FollowUpAdmissionQueryCommand`, `ConvertAdmissionQueryCommand`.

---

### FINDING 42 (id: `SPEC-1-042`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** Medium
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/assessment/workflows.md` vs `crates/domains/assessment/src/commands.rs`

**Description:**

The assessment workflows spec defines cross-domain triggers (e.g., `SendAbsenceNotification (cross-domain trigger)` in `commands.md`). The source crate `commands.rs` lacks the `SendAbsenceNotificationCommand` handler. The attendance domain also lacks the corresponding handler in `crates/domains/attendance/src/commands.rs`. Neither side owns the cross-domain notification trigger.

**Expected:**

A single domain owns the `SendAbsenceNotification` command handler (likely attendance, since `attendance/commands.md` lists `SendAbsenceNotification`); the cross-domain trigger contract is documented.

**Evidence:**

`docs/specs/assessment/commands.md` `## SendAbsenceNotification (cross-domain trigger)`. `docs/specs/attendance/commands.md` `## SendAbsenceNotification`. `grep "SendAbsenceNotificationCommand" crates/domains/assessment/src/commands.rs crates/domains/attendance/src/commands.rs` returns no match.

---

### FINDING 43 (id: `SPEC-1-043`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** Medium
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/academic/permissions.md:Capabilities` vs `crates/cross-cutting/rbac/src/value_objects.rs`

**Description:**

The academic permissions spec lists capabilities in un-prefixed form (`Student.Admit`, `Student.Update`, etc.). The rbac crate declares variants with prefixed enums (`AcademicStudentCreate`, `AcademicStudentRead`, etc.) but does not declare `AcademicStudentAdmit`, `AcademicStudentSuspend`, `AcademicStudentReinstate`, `AcademicStudentWithdraw`, `AcademicStudentTransfer`, `AcademicStudentPromote`, `AcademicStudentGraduate`. The spec commands `SuspendStudent`, `ReinstateStudent`, `WithdrawStudent`, `TransferStudent`, `PromoteStudent`, `GraduateStudent` therefore have no corresponding rbac capability at all (not even a generic `AcademicStudentUpdate`).

**Expected:**

Every spec command has a corresponding rbac capability variant. The lifecycle transitions (suspend, reinstate, withdraw, transfer, promote, graduate) require dedicated rbac variants that the current rbac enum does not provide.

**Evidence:**

`docs/specs/academic/commands.md` capabilities include `Student.Suspend`, `Student.Reinstate`, `Student.Withdraw`, `Student.Transfer`, `Student.Promote`, `Student.Graduate`. `grep "AcademicStudent" crates/cross-cutting/rbac/src/value_objects.rs` returns only `Create`, `Read`, `Update`, `Delete`.

---

### FINDING 44 (id: `SPEC-1-044`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** Medium
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/communication/aggregates.md:778` (`## SendMessage`) vs `docs/specs/communication/commands.md` (commands)

**Description:**

The communication spec declares `## SendMessage` as both an aggregate (in `aggregates.md`) and a command-action prefix (e.g., `SendMessage.Create`, `SendMessage.Dispatch`, `SendMessage.Cancel` in `commands.md`). An aggregate named `SendMessage` is suspicious because the verb-form suggests a command, not a domain entity. This is a naming-collision risk: the aggregate and the command set use the same root word.

**Expected:**

The aggregate is renamed to a noun-form (e.g., `BulkMessage` or `MessageDispatch`) to disambiguate from the command-action prefix. The commands keep their `SendMessage.*` capability strings for backwards compatibility, but the aggregate storage is named differently.

**Evidence:**

`docs/specs/communication/aggregates.md:778` `## SendMessage`. `docs/specs/communication/commands.md` capabilities: `SendMessage.Create`, `SendMessage.Dispatch`, `SendMessage.Cancel` — same root word for both aggregate and command-action namespace.

---

### FINDING 45 (id: `SPEC-1-045`)

- **Source:** `docs/audit_reports/findings/wave6-specs-1.md`
- **Severity:** Medium
- **Area:** spec-domains-1-5
- **Location:** `docs/specs/communication/aggregates.md:752` (`## ChatStatus`) vs `crates/domains/communication/src/aggregate.rs` (`pub struct ChatStatusRecord`)

**Description:**

The communication spec declares the `ChatStatus` aggregate at `docs/specs/communication/aggregates.md:752`. The source crate declares `pub struct ChatStatusRecord` (not `ChatStatus`) in `crates/domains/communication/src/aggregate.rs`. The aggregate root name drifted by a `Record` suffix between spec and source.

**Expected:**

Either the spec aggregate is renamed to `ChatStatusRecord` to match source, or the source root is renamed to `ChatStatus` to match spec. The aggregate root name appears in every event payload and command struct, so the drift has cascading effects on type signatures.

**Evidence:**

`docs/specs/communication/aggregates.md:752` `## ChatStatus`. `crates/domains/communication/src/aggregate.rs` `grep "^pub struct Chat"` returns `ChatStatusRecord` (not `ChatStatus`).

---


## Specs (hr, library, events-domain) (target id prefix: `SPEC-2`)

**Path:** `docs/specs/{hr,library,events-domain}/`  
**Total findings:** 30 (10 critical, 13 high, 7 medium, 0 low)


### FINDING 1 (id: `SPEC-2-001`)

- **Source:** `docs/audit_reports/findings/wave6-specs-2.md`
- **Severity:** Critical
- **Area:** spec-code-drift
- **Location:** `crates/domains/documents/src/**/*.rs` (zero matches) vs `docs/specs/documents/tables.md:11-14`

**Description:**

The documents domain spec defines 3 tables (`documents_form_downloads`, `documents_postal_dispatches`, `documents_postal_receives`), but the `educore-documents` crate has zero `#[derive(DomainQuery)]` applications across all 10 source files. The engine's storage adapter walks the macro-emitted AST to translate queries; without these derives no AST is emitted and the adapter cannot translate a single documents-domain query at runtime.

**Expected:**

At least one `#[derive(DomainQuery)]` application per aggregate in `crates/domains/documents/src/entities.rs` (or wherever the macro is applied) corresponding to the 3 tables documented at `docs/specs/documents/tables.md:11-14`.

**Evidence:**

`grep -c "DomainQuery" /home/beznet/Workspace/smscore/crates/domains/documents/src/*.rs` returns `0` for every file. `docs/specs/documents/tables.md:11-14` lists `documents_form_downloads`, `documents_postal_dispatches`, `documents_postal_receives` as the three storage tables.

---

### FINDING 15 (id: `SPEC-2-015`)

- **Source:** `docs/audit_reports/findings/wave6-specs-2.md`
- **Severity:** Critical
- **Area:** spec-code-drift
- **Location:** `docs/specs/finance/aggregates.md` (51 ## aggregate sections) vs `crates/domains/finance/src/aggregate.rs` (5 aggregate structs)

**Description:**

The finance spec enumerates 51 aggregate sections (e.g., `FeesGroup`, `FeesType`, `FeesMaster`, `FeesAssign`, `FeesDiscount`, `FeesInvoice`, `FeesPayment`, `Expense`, `Income`, `BankAccount`, `Wallet`, `PayrollPayment`, `PayrollGenerate`, `SalaryTemplate`, `Donor`, `Transaction`, etc. — one `##` header per aggregate). The Rust `aggregate.rs` file contains only 5 aggregate structs (`Wallet`, `WalletTransaction`, `FeesInvoice`, `FeesPayment`, `Expense`). 46 aggregates documented in the spec have no Rust aggregate struct at all.

**Expected:**

Each of the 51 spec aggregates has a corresponding `pub struct` in `crates/domains/finance/src/aggregate.rs` (or split across `entities.rs` if the spec defines them as child entities).

**Evidence:**

`grep -c '^## ' docs/specs/finance/aggregates.md` = 51; `grep -n '^pub struct' crates/domains/finance/src/aggregate.rs` = 5 (lines 57, 195, 361, 425, 531).

---

### FINDING 16 (id: `SPEC-2-016`)

- **Source:** `docs/audit_reports/findings/wave6-specs-2.md`
- **Severity:** Critical
- **Area:** spec-internal-inconsistency
- **Location:** `docs/specs/library/tables.md:11-19` vs `docs/specs/library/aggregates.md` (only 4 aggregate roots)

**Description:**

The library tables.md lists 9 tables (`library_book_categories`, `library_books`, `library_members`, `library_book_issues`, `library_book_issue_renewals`, `library_book_issue_fines`, `library_book_acquisitions`, `library_book_catalog_entries`, `library_library_member_notes`). The aggregates.md defines only 4 aggregate roots (`BookCategory`, `Book`, `LibraryMember`, `BookIssue`). The remaining 5 tables (`library_book_issue_renewals`, `library_book_issue_fines`, `library_book_acquisitions`, `library_book_catalog_entries`, `library_library_member_notes`) have no aggregate definition; they are listed as "Aggregates" in the table's middle column (`BookIssueRenewal`, `BookIssueFine`, `BookAcquisition`, `BookCatalogEntry`, `LibraryMemberNote`), but those types are not documented anywhere in `aggregates.md`.

**Expected:**

Either (a) the 5 tables without aggregate roots are reclassified as child entities owned by one of the 4 aggregates and the tables.md "aggregate" column lists the owning aggregate (e.g., `BookIssue` for `library_book_issue_renewals`), or (b) 5 new aggregate sections are added to `docs/specs/library/aggregates.md` documenting each.

**Evidence:**

`docs/specs/library/tables.md:13-19` lists 9 rows with aggregate column entries `BookIssueRenewal`, `BookIssueFine`, `BookAcquisition`, `BookCatalogEntry`, `LibraryMemberNote`. `docs/specs/library/aggregates.md` only contains 4 `## Root type:` entries: `BookCategory`, `Book`, `LibraryMember`, `BookIssue`.

---

### FINDING 19 (id: `SPEC-2-019`)

- **Source:** `docs/audit_reports/findings/wave6-specs-2.md`
- **Severity:** Critical
- **Area:** spec-code-drift
- **Location:** `crates/domains/library/src/aggregate.rs` vs `docs/specs/library/aggregates.md`

**Description:**

The library `aggregate.rs` file is 732 lines and contains the 4 aggregate structs (`BookCategory`, `Book`, `LibraryMember`, `BookIssue`). The aggregate file header comments at the top must state whether these are full implementations or placeholders. Per the documents pattern, all 5 domain crate aggregate files in scope use `#![allow(dead_code, clippy::all)]` placeholders, but library `aggregate.rs` does not have the placeholder marker — meaning library may have a partially implemented aggregate with invariants unenforced.

**Expected:**

The library aggregate.rs file either implements all invariants from `aggregates.md` or is clearly marked as a placeholder (matching the documents pattern) so reviewers know to discount the file.

**Evidence:**

`crates/domains/library/src/aggregate.rs` size = 732 lines; absence of `#![allow(dead_code, clippy::all)]` at the top of the file (compared to `crates/domains/documents/src/aggregate.rs:9-10` which has it). Need to confirm by reading the file header.

---

### FINDING 2 (id: `SPEC-2-002`)

- **Source:** `docs/audit_reports/findings/wave6-specs-2.md`
- **Severity:** Critical
- **Area:** spec-code-drift
- **Location:** `crates/domains/finance/src/**/*.rs` (zero matches) vs `docs/specs/finance/tables.md`

**Description:**

The finance domain spec is the largest in scope (1,569 lines of aggregates, 988 lines of commands, 508 lines of events) but the `educore-finance` crate has zero `#[derive(DomainQuery)]` applications across all 10 source files. The query layer is not compile-time generated for any finance aggregate, so the storage adapters cannot translate a single finance query AST node into SQL.

**Expected:**

At least one `#[derive(DomainQuery)]` application per aggregate documented in `docs/specs/finance/aggregates.md` (FeeInvoice, FeePayment, FeeDiscount, Expense, PayrollGenerate, PayrollPayment, etc.) — corresponding to the tables listed in `docs/specs/finance/tables.md`.

**Evidence:**

`grep -c "DomainQuery" /home/beznet/Workspace/smscore/crates/domains/finance/src/*.rs` returns `0` for every file.

---

### FINDING 20 (id: `SPEC-2-020`)

- **Source:** `docs/audit_reports/findings/wave6-specs-2.md`
- **Severity:** Critical
- **Area:** spec-code-drift
- **Location:** `crates/domains/facilities/src/aggregate.rs` (1454 lines) vs `docs/specs/facilities/aggregates.md` (15 aggregate sections, 575 lines)

**Description:**

The facilities spec defines 15 aggregate roots with full invariant lists, but the source `aggregate.rs` is 1454 lines — 2.5× the size of the spec file. With facilities having only 2 `#[derive(DomainQuery)]` applications (per `grep -c DomainQuery crates/domains/facilities/src/query.rs` = 2), the query layer is severely under-implemented relative to the 15 aggregates.

**Expected:**

At least one `#[derive(DomainQuery)]` application per aggregate root, with a corresponding entry in `crates/domains/facilities/src/query.rs` for each.

**Evidence:**

`grep -c '^## ' docs/specs/facilities/aggregates.md` = 15; `wc -l crates/domains/facilities/src/aggregate.rs` = 1454; `grep -c "DomainQuery" crates/domains/facilities/src/query.rs` = 2.

---

### FINDING 21 (id: `SPEC-2-021`)

- **Source:** `docs/audit_reports/findings/wave6-specs-2.md`
- **Severity:** Critical
- **Area:** spec-code-drift
- **Location:** `crates/domains/hr/src/aggregate.rs:1289 lines` vs `docs/specs/hr/aggregates.md:568 lines` vs `crates/domains/hr/src/commands.rs:269 lines` vs `docs/specs/hr/commands.md:715 lines`

**Description:**

The HR spec's commands.md (715 lines) is 2.6× larger than the source commands.rs (269 lines). The aggregate.rs source is 1289 lines while aggregates.md is 568 lines (2.3× ratio in the opposite direction). This bidirectional size asymmetry indicates that the source code does not yet implement many of the commands documented in the spec. The spec enumerates the full domain surface; the source captures only a subset.

**Expected:**

Each command struct documented in `docs/specs/hr/commands.md` has a corresponding `pub struct` in `crates/domains/hr/src/commands.rs`.

**Evidence:**

`wc -l docs/specs/hr/commands.md` = 715; `wc -l crates/domains/hr/src/commands.rs` = 269. `grep -n '^pub struct' crates/domains/hr/src/commands.rs` shows ~25 command structs, while `docs/specs/hr/commands.md` defines substantially more.

---

### FINDING 3 (id: `SPEC-2-003`)

- **Source:** `docs/audit_reports/findings/wave6-specs-2.md`
- **Severity:** Critical
- **Area:** spec-code-drift
- **Location:** `crates/domains/hr/src/**/*.rs` (zero matches) vs `docs/specs/hr/tables.md`

**Description:**

The HR domain spec defines at least 8 aggregates (Staff, StaffDocument, AssignClassTeacher, LeaveRequest, LeaveDeductionInfo, PayrollGenerate, PayrollPayment, Department, Designation, etc.) and corresponding tables, but the `educore-hr` crate has zero `#[derive(DomainQuery)]` applications across all 10 source files. The query macro pipeline is not wired for HR.

**Expected:**

At least one `#[derive(DomainQuery)]` application per aggregate, with coverage matching the table inventory at `docs/specs/hr/tables.md`.

**Evidence:**

`grep -c "DomainQuery" /home/beznet/Workspace/smscore/crates/domains/hr/src/*.rs` returns `0` for every file.

---

### FINDING 5 (id: `SPEC-2-005`)

- **Source:** `docs/audit_reports/findings/wave6-specs-2.md`
- **Severity:** Critical
- **Area:** spec-code-drift
- **Location:** `crates/domains/documents/src/aggregate.rs` (placeholder) vs `docs/specs/documents/aggregates.md`

**Description:**

The documents aggregate file at `crates/domains/documents/src/aggregate.rs` declares itself a scaffold-only placeholder: lines 9-14 state "The placeholder structs declared here use the same names as the real aggregate types so the prelude's `pub use` lines resolve during the scaffold phase. The owner subagents will replace the bodies with the full domain implementation, preserving the public names." Per the spec, the 3 aggregates (`FormDownload`, `PostalDispatch`, `PostalReceive`) must enforce the invariants listed in `docs/specs/documents/aggregates.md`; the current placeholder body enforces none of them.

**Expected:**

The 3 aggregate structs at `crates/domains/documents/src/aggregate.rs` enforce all invariants from `docs/specs/documents/aggregates.md`: title non-empty (FormDownload invariant 1), at least one of link/file set (invariant 2), soft-delete via `active_status` (invariant 4), `to_title` and `from_title` non-empty (PostalDispatch/Receive invariant 1), reference_no uniqueness within `(school_id, academic_id)` (invariants 2).

**Evidence:**

`crates/domains/documents/src/aggregate.rs:9-14` `#![allow(dead_code, clippy::all)]` / `#![allow(missing_docs)]` / `// The placeholder structs declared here use the same names as the /` // real aggregate types so the prelude's `pub use` lines resolve /` // during the scaffold phase. The owner subagents will replace the /` // bodies with the full domain implementation, preserving the /` // public names.``

---

### FINDING 6 (id: `SPEC-2-006`)

- **Source:** `docs/audit_reports/findings/wave6-specs-2.md`
- **Severity:** Critical
- **Area:** spec-internal-inconsistency
- **Location:** `docs/specs/documents/overview.md:89-93` vs `docs/specs/documents/aggregates.md:36-37`

**Description:**

The documents overview (line 89) lists `FormDownload`, `PostalDispatch`, `PostalReceive` as the three aggregate roots. The overview's "Domain Invariants" section (line 75) says "A `PostalDispatch` belongs to exactly one school and one academic year" (invariant 5) and the same for `PostalReceive` (invariant 6). The aggregates file confirms this for the `PostalDispatch` invariant #3 ("A `PostalDispatch` is anchored to a school and an academic year"). But the `tables.md` file (line 30-31) says "The `documents_form_downloads` table does not include `academic_id`; the scope is per-school only." — so for `FormDownload`, the table says no academic_id, but no aggregate-invariant explicitly disclaims academic-year scope. The aggregate spec says `FormDownload` is "anchored to a school" only; the table says no `academic_id`. The spec is consistent on that, but no aggregate entry documents the negative invariant — a future implementer might add `academic_id` because it is the engine pattern.

**Expected:**

`docs/specs/documents/aggregates.md` FormDownload section includes an explicit invariant stating "A `FormDownload` is not anchored to an academic year; it is per-school only", matching `docs/specs/documents/tables.md:30-31`.

**Evidence:**

`docs/specs/documents/aggregates.md:14-26` lists FormDownload invariants 1-5 but does not mention academic-year scope one way or the other. `docs/specs/documents/tables.md:30-31` `The documents_form_downloads table does not include academic_id; the scope is per-school only. Forms are not academic-year-bounded.`

---

### FINDING 10 (id: `SPEC-2-010`)

- **Source:** `docs/audit_reports/findings/wave6-specs-2.md`
- **Severity:** High
- **Area:** spec-code-drift
- **Location:** `crates/domains/documents/src/services.rs` (1911 lines) vs `docs/specs/documents/services.md` (117 lines)

**Description:**

The documents services.rs file is 1911 lines while the spec's services.md is only 117 lines. The spec is approximately 16× smaller than the implementation, suggesting either (a) the services.md file is grossly incomplete and underspecifies the actual policy logic, or (b) the implementation contains substantial logic not documented in the spec. Either case violates `AGENTS.md` Engine Rule #9 (Production-ready: real schools, real students, real money) because the policy surface is invisible to spec reviewers.

**Expected:**

`docs/specs/documents/services.md` documents each service in `crates/domains/documents/src/services.rs` at the same granularity: pre-conditions, side effects, error paths, idempotency keys, cross-domain events emitted.

**Evidence:**

`wc -l crates/domains/documents/src/services.rs` = 1911; `wc -l docs/specs/documents/services.md` = 117.

---

### FINDING 11 (id: `SPEC-2-011`)

- **Source:** `docs/audit_reports/findings/wave6-specs-2.md`
- **Severity:** High
- **Area:** spec-code-drift
- **Location:** `docs/specs/documents/entities.md` (36 lines) vs `crates/domains/documents/src/entities.rs` (289 lines)

**Description:**

The documents entities file is only 36 lines while the source has 289 lines. This 8× size ratio indicates the spec either is missing the bulk of the entity definitions or the source has substantial unstructured entity code that the spec never defined. The spec should enumerate every public entity (e.g., `FormDownloadFile`, `FormDownloadLink`, `PostalDispatchAttachment`, `PostalReceiveAttachment`) with its identity and storage table mapping.

**Expected:**

`docs/specs/documents/entities.md` documents each entity struct in `crates/domains/documents/src/entities.rs` (storage-row projection types, not aggregate roots) with its identifier, storage table, and column mapping.

**Evidence:**

`wc -l docs/specs/documents/entities.md` = 36; `wc -l crates/domains/documents/src/entities.rs` = 289. `grep -n "^pub struct" crates/domains/documents/src/entities.rs` shows multiple entity definitions not enumerated in the spec.

---

### FINDING 17 (id: `SPEC-2-017`)

- **Source:** `docs/audit_reports/findings/wave6-specs-2.md`
- **Severity:** High
- **Area:** spec-internal-inconsistency
- **Location:** `docs/specs/library/aggregates.md:169` (FineCalculated event) vs `docs/specs/library/aggregates.md:115-174` (no BookIssueFine aggregate)

**Description:**

The `BookIssue` aggregate's "Events" list at `aggregates.md:169` includes `FineCalculated`. But there is no `BookIssueFine` aggregate in `aggregates.md`, no commands to create one (the `CalculateFine` command in `commands.md:312-320` emits `FineCalculated` and "records a `BookIssueFine` history entry"), and no aggregate invariants enforcing fine uniqueness. The `BookIssueFine` history row in `library_book_issue_fines` is therefore a phantom: the spec says it's emitted but does not say how it's owned or constrained.

**Expected:**

Either add a `## BookIssueFine` aggregate section in `docs/specs/library/aggregates.md` with invariants and command/event lists, or move the fine storage to a child-entity relationship owned by `BookIssue` and document the ownership explicitly in `BookIssue.Owned Children`.

**Evidence:**

`docs/specs/library/aggregates.md:169` `- `FineCalculated``. `docs/specs/library/commands.md:317-319` `The fine is recorded as a `BookIssueFine` history entry. Finance may subscribe to post the receivable.`

---

### FINDING 18 (id: `SPEC-2-018`)

- **Source:** `docs/audit_reports/findings/wave6-specs-2.md`
- **Severity:** High
- **Area:** spec-internal-inconsistency
- **Location:** `docs/specs/library/aggregates.md:170-178` vs `docs/specs/library/commands.md:301-320`

**Description:**

The `BookIssue` aggregate's "Commands" list (`aggregates.md:160-164`) lists only 5 commands: `IssueBook`, `ReturnBook`, `RenewBook`, `MarkBookLost`, `CalculateFine`. But `commands.md` defines additional commands referenced by the workflows (e.g., `PayFine`, `WaiveFine` — implied by fine workflows but not listed). A reader of aggregates.md cannot determine the full command surface without cross-reading commands.md. The aggregate spec is supposed to be the authoritative boundary declaration per `AGENTS.md` Engine Rule #5.

**Expected:**

The aggregate's "Commands" list in `aggregates.md` matches the actual command struct definitions in `commands.md` 1:1. Any new command (e.g., for fine payment or waiver) is added to both files simultaneously.

**Evidence:**

`docs/specs/library/aggregates.md:160-164` lists 5 commands; `docs/specs/library/commands.md` defines 16+ command structs (e.g., `RegisterLibraryMember`, `UpdateLibraryMember`, `DeactivateLibraryMember`, `ReactivateLibraryMember`, `DeleteLibraryMember`, `AddBook`, `UpdateBook`, `DeleteBook`, `AdjustBookQuantity`, etc.).

---

### FINDING 22 (id: `SPEC-2-022`)

- **Source:** `docs/audit_reports/findings/wave6-specs-2.md`
- **Severity:** High
- **Area:** spec-internal-inconsistency
- **Location:** `docs/specs/documents/tables.md:18-25` vs `docs/specs/documents/entities.md`

**Description:**

The documents tables.md notes section (lines 18-25) states "Every school-scoped table includes `academic_id` referencing `academic_academic_years`." But line 30 then states "The `documents_form_downloads` table does not include `academic_id`". The note at line 24 explicitly carves out `documents_form_downloads` but does not state whether the two `documents_postal_*` tables include it. A reader must check `aggregates.md` invariant 3 ("A `PostalDispatch` is anchored to a school and an academic year") to confirm, which is not stated in tables.md. The tables.md file should be self-contained.

**Expected:**

`docs/specs/documents/tables.md` notes section explicitly states for each table whether `academic_id` is present (e.g., a column in the main table listing).

**Evidence:**

`docs/specs/documents/tables.md:18-20` `Every school-scoped table includes academic_id for multi-tenant isolation` followed by `:30-31` `The documents_form_downloads table does not include academic_id`. Lines 23-25 say `Every table includes created_at, updated_at, ... These are managed by the engine's storage adapter.` — without an `academic_id` column in the main table, a reader cannot determine which tables have it.

---

### FINDING 23 (id: `SPEC-2-023`)

- **Source:** `docs/audit_reports/findings/wave6-specs-2.md`
- **Severity:** High
- **Area:** spec-internal-inconsistency
- **Location:** `docs/specs/documents/tables.md:11-14` vs `docs/specs/documents/aggregates.md` (no `Owned Children` documentation for child tables)

**Description:**

The spec defines 3 aggregate roots and 3 main tables, but the actual implementation in `crates/domains/documents/src/aggregate.rs:253-398` adds child entity structs (`FormDownloadFile` at line 253, `FormDownloadLink` at line 312, `PostalDispatchAttachment` at line 635, `PostalReceiveAttachment` at line 949). The spec's `aggregates.md` FormDownload "Owned Children" list (lines 13-15) mentions `FormDownloadFile` and `FormDownloadLink` but does NOT mention `PostalDispatchAttachment` or `PostalReceiveAttachment` for the postal aggregates, even though both exist in source and the spec says they have "optional attachment".

**Expected:**

`docs/specs/documents/aggregates.md` PostalDispatch "Owned Children" section explicitly lists `PostalDispatchAttachment`. Same for PostalReceive and `PostalReceiveAttachment`.

**Evidence:**

`docs/specs/documents/aggregates.md:13-15` FormDownload Owned Children: `FormDownloadFile` and `FormDownloadLink`. Lines 56-58 (PostalDispatch Owned Children) are absent (the section jumps from Purpose to Invariants). The spec at `aggregates.md:60-69` lists 5 invariants but no Owned Children. The source at `crates/domains/documents/src/aggregate.rs:635` declares `pub struct PostalDispatchAttachment`.

---

### FINDING 24 (id: `SPEC-2-024`)

- **Source:** `docs/audit_reports/findings/wave6-specs-2.md`
- **Severity:** High
- **Area:** spec-internal-inconsistency
- **Location:** `docs/specs/documents/aggregates.md:79` (PostalReceive Root type) vs `docs/specs/documents/tables.md:14` (table name `documents_postal_receives`)

**Description:**

The aggregates file declares the `PostalReceive` aggregate root (line 79). The tables file maps this to a storage table named `documents_postal_receives` (line 14). However, the aggregate's "Owned Children" section (lines 81-83) is missing — for a postal receive that has "optional attachment" (per the Purpose at line 84), the Owned Children must enumerate the attachment entity. The spec is silent on this child, so the storage schema for the attachment is undocumented.

**Expected:**

`docs/specs/documents/aggregates.md` PostalReceive section includes an "Owned Children" subsection listing `PostalReceiveAttachment` (and any other child entities), with their type, identity, and storage table mapping.

**Evidence:**

`docs/specs/documents/aggregates.md:79-99` (PostalReceive section) — no "Owned Children" header. The Purpose at `:84` says "optional attachment". The source at `crates/domains/documents/src/aggregate.rs:949` declares `pub struct PostalReceiveAttachment`.

---

### FINDING 25 (id: `SPEC-2-025`)

- **Source:** `docs/audit_reports/findings/wave6-specs-2.md`
- **Severity:** High
- **Area:** spec-internal-inconsistency
- **Location:** `docs/specs/documents/overview.md:62-67` (Dependencies) vs `docs/specs/documents/commands.md` (commands use `educore-files`-shaped `FileReference`)

**Description:**

The documents overview's "Dependencies" subsection lists `educore-core`, `educore-platform`, `educore-rbac`, `educore-events` — but not `educore-files`. Yet the commands (`commands.md:75-95, 140-154`) accept `Option<FileReference>`, and the aggregates (`aggregates.md:11`) describe forms with "optional file" and "optional URL". A `FileReference` must be a value object owned by the `educore-files` port crate, but the documents domain is not declared as depending on it. This creates a missing dependency edge in the dependency graph.

**Expected:**

`docs/specs/documents/overview.md` Dependencies section explicitly lists `educore-files` (the file storage port) as a domain-level dependency, and `crates/domains/documents/Cargo.toml` adds the corresponding `educore-files` path dependency.

**Evidence:**

`docs/specs/documents/overview.md:62-67` lists 4 deps without `educore-files`. `docs/specs/documents/commands.md:94` `pub file: Option<FileReference>,`.

---

### FINDING 30 (id: `SPEC-2-030`)

- **Source:** `docs/audit_reports/findings/wave6-specs-2.md`
- **Severity:** High
- **Area:** spec-internal-inconsistency
- **Location:** `docs/specs/documents/events.md:31` vs `docs/specs/documents/commands.md:24`

**Description:**

The `FormUploaded` event payload (`events.md:31-37`) is declared with fields `form_id, title, publish_date, show_public, uploaded_by`. The `UploadFormCommand` (`commands.md:11-19`) carries additional fields not in the event: `short_description, link, file`. The event therefore loses information at the command→event boundary. A subscriber (e.g., CMS) cannot reconstruct the form from `FormUploaded` alone — it must re-query the aggregate. The spec is silent on whether event subscribers should re-query (recommended) or receive the full payload (anti-pattern per audit-first principles).

**Expected:**

`docs/specs/documents/events.md` FormUploaded payload includes the full create-time fields (`short_description, link, file`) OR the spec explicitly states "subscribers MUST re-query the aggregate by `form_id` for full state", and the workflows.md documents this contract.

**Evidence:**

`docs/specs/documents/events.md:31-37` `pub struct FormUploaded { pub form_id, pub title, pub publish_date, pub show_public, pub uploaded_by }`. `docs/specs/documents/commands.md:11-19` `UploadFormCommand` carries `title, short_description, publish_date, link, file, show_public`.

---

### FINDING 4 (id: `SPEC-2-004`)

- **Source:** `docs/audit_reports/findings/wave6-specs-2.md`
- **Severity:** High
- **Area:** spec-code-drift
- **Location:** `docs/specs/documents/commands.md:24-37` vs `crates/domains/documents/src/commands.rs:91-111`

**Description:**

The spec's `UpdateFormCommand` uses `Option<T>` (2-state) for `short_description`, `link`, and `file`. The Rust struct uses the 3-state `Option<Option<T>>` pattern for those three fields (outer `None` = no change, `Some(None)` = clear, `Some(Some(_))` = set). The spec's prose documents only "update or no-change" semantics; the code adds "explicitly clear the field" semantics. Without updating the spec, a consumer following the spec cannot issue a "clear the link" command because `link: None` in the spec means "no change".

**Expected:**

`docs/specs/documents/commands.md` UpdateFormCommand block documents the `Option<Option<T>>` 3-state pattern for `short_description`, `link`, and `file`, with the explicit semantics ("no change" / "clear" / "set"), matching the source at `crates/domains/documents/src/commands.rs:91-111`.

**Evidence:**

`docs/specs/documents/commands.md:32-34` `pub short_description: Option<FormDescription>,` / `pub link: Option<Url>,` / `pub file: Option<FileReference>,`. `crates/domains/documents/src/commands.rs:91-110` `pub short_description: Option<Option<FormDescription>>,` / `pub link: Option<Option<Url>>,` / `pub file: Option<Option<FileReference>>,.`

---

### FINDING 7 (id: `SPEC-2-007`)

- **Source:** `docs/audit_reports/findings/wave6-specs-2.md`
- **Severity:** High
- **Area:** spec-internal-inconsistency
- **Location:** `docs/specs/documents/commands.md:155-177` vs `docs/specs/documents/events.md`

**Description:**

The spec defines a `TrackPostalCommand` (`docs/specs/documents/commands.md:155-177`) which "is a query command and does not produce a domain event." The aggregate spec at `docs/specs/documents/aggregates.md` does not list `TrackPostal` as a command of any aggregate (the 3 aggregates are `FormDownload`, `PostalDispatch`, `PostalReceive`). The events spec at `docs/specs/documents/events.md` lists only lifecycle events (Form* and Postal*). A "query command" that crosses both `PostalDispatch` and `PostalReceive` aggregates violates the "Consistency Boundary" rule in `aggregates.md:48` ("All form mutations are serialized through the `FormDownload` aggregate root"), because `TrackPostal` mutates nothing yet is a Command in the spec taxonomy. The spec should either move this to `services.md` (queries) or explicitly classify it as the only cross-aggregate read.

**Expected:**

`TrackPostalCommand` is reclassified as a service/repository query in `docs/specs/documents/services.md` or `docs/specs/documents/repositories.md`, not as a Command. Or the aggregates.md file explicitly documents the cross-aggregate read pattern for `TrackPostal`.

**Evidence:**

`docs/specs/documents/commands.md:156-177` `pub struct TrackPostalCommand { pub tenant: TenantContext, pub reference_no: PostalReferenceNo, }` / `Effects: Read-only query that surfaces a list of dispatch and receive records matching the reference number within the school. This is a query command and does not produce a domain event.` `docs/specs/documents/aggregates.md:36-37` (PostalDispatch Commands list) and `:97-99` (PostalReceive Commands list) do not mention TrackPostal.

---

### FINDING 8 (id: `SPEC-2-008`)

- **Source:** `docs/audit_reports/findings/wave6-specs-2.md`
- **Severity:** High
- **Area:** spec-internal-inconsistency
- **Location:** `docs/specs/documents/permissions.md` vs `docs/specs/documents/commands.md` capability strings

**Description:**

The commands file lists capabilities as `Form.Upload` (`commands.md:24`), `Form.Update` (`:48`), `Form.Delete` (`:61`), `Postal.Dispatch` (`:80`), `Postal.Update` (`:107`), `Postal.Delete` (`:130`), `Postal.Receive` (`:152`). The permissions file may use a different namespace convention. A reader cannot determine which `Form.*` or `Postal.*` strings are actually enforced without checking the rbac catalog.

**Expected:**

The permissions file at `docs/specs/documents/permissions.md` lists each of the 10 commands' capabilities verbatim (`Form.Upload`, `Form.Update`, `Form.Delete`, `Postal.Dispatch`, `Postal.Update`, `Postal.Delete`, `Postal.Receive`), and the rbac catalog at `docs/specs/rbac/permissions.md` registers those exact strings.

**Evidence:**

`docs/specs/documents/commands.md` capability lines: `:24 Form.Upload`, `:48 Form.Update`, `:61 Form.Delete`, `:80 Postal.Dispatch`, `:107 Postal.Update`, `:130 Postal.Delete`, `:152 Postal.Receive`, `:175 Postal.Read` (for TrackPostal). The permissions file structure must be cross-checked against these strings.

---

### FINDING 9 (id: `SPEC-2-009`)

- **Source:** `docs/audit_reports/findings/wave6-specs-2.md`
- **Severity:** High
- **Area:** spec-internal-inconsistency
- **Location:** `docs/specs/documents/overview.md:103-108` vs `docs/specs/documents/commands.md`

**Description:**

The overview's "Anti-Goals" section says "The documents domain does not implement a file storage backend. Files are held in the file storage port." But the `DispatchPostal` command (`commands.md:75-95`) and `ReceivePostal` command (`commands.md:140-154`) both accept a `file: Option<FileReference>` directly — meaning the documents domain's command surface does carry a file reference, which must be persisted against the file storage port at the storage adapter layer. The spec does not document which port-impl (`educore-files` adapter) the storage layer is expected to integrate with, nor whether the documents domain enforces that `FileReference` is non-null when the file is uploaded but the link is empty (a wiring gap with the file port).

**Expected:**

`docs/specs/documents/overview.md` "Dependencies" subsection explicitly lists `educore-files` (file storage port) as a domain-level dependency (not just "file storage port" in prose), and `docs/specs/documents/workflows.md` documents the wiring contract between `DispatchPostal.file` and the file-storage port's `put` operation.

**Evidence:**

`docs/specs/documents/overview.md:62-67` lists only `educore-core`, `educore-platform`, `educore-rbac`, `educore-events` as dependencies — `educore-files` is absent. `:103-105` `The documents domain does not implement a file storage backend. Files are held in the file storage port.`

---

### FINDING 12 (id: `SPEC-2-012`)

- **Source:** `docs/audit_reports/findings/wave6-specs-2.md`
- **Severity:** Medium
- **Area:** spec-code-drift
- **Location:** `crates/domains/documents/src/aggregate.rs:99-252` vs `docs/specs/documents/aggregates.md:1-50`

**Description:**

The Rust `FormDownload` struct in source is 153 lines (line 99-252), with 8 fields documented (`id`, `school_id`, `title`, `short_description`, `publish_date`, `link`, `file`, `show_public`, `active_status`, `etag`, `version`, `created_at`, `updated_at`, `created_by`, `updated_by`, `events`). The spec's aggregates.md `FormDownload` section enumerates only `Owned Children` (`FormDownloadFile`, `FormDownloadLink`) and 5 invariants, but no field inventory. Without a field inventory, an implementer or reviewer cannot reconcile the spec's "form has a title, short description, publish date, optional URL, optional file, public-visibility flag" prose against the actual 16-field struct.

**Expected:**

`docs/specs/documents/aggregates.md` FormDownload section includes a field table: field name, type, optional/required, invariants enforced, mapping to `documents_form_downloads` table column.

**Evidence:**

`crates/domains/documents/src/aggregate.rs:99-252` (FormDownload struct definition, 16 fields). `docs/specs/documents/aggregates.md:9-12` `Owned Children` lists only `FormDownloadFile` and `FormDownloadLink`; no field inventory.

---

### FINDING 13 (id: `SPEC-2-013`)

- **Source:** `docs/audit_reports/findings/wave6-specs-2.md`
- **Severity:** Medium
- **Area:** spec-internal-inconsistency
- **Location:** `docs/specs/documents/tables.md:14` vs `docs/specs/documents/aggregates.md:79-83`

**Description:**

The spec's tables.md uses the table name `documents_postal_dispatches` (plural) and `documents_postal_receives` (plural, irregular). The aggregates.md uses the singular types `PostalDispatch` / `PostalReceive`. The aggregates.md does not document the singular-to-plural table-naming convention. The `documents_postal_receives` form is grammatically inconsistent with `documents_postal_dispatches` (a developer would expect `documents_postal_receivers` or `documents_postal_received_items`). A future implementer cannot determine if `receives` is intentional.

**Expected:**

`docs/specs/documents/tables.md` includes a note explaining the irregular plural `documents_postal_receives` (or normalizes to `documents_postal_receivers` / `documents_postal_received`).

**Evidence:**

`docs/specs/documents/tables.md:13-14` `documents_postal_dispatches` / `documents_postal_receives`. `docs/specs/documents/aggregates.md:55` `Root type: PostalReceive`.

---

### FINDING 14 (id: `SPEC-2-014`)

- **Source:** `docs/audit_reports/findings/wave6-specs-2.md`
- **Severity:** Medium
- **Area:** spec-code-drift
- **Location:** `docs/specs/documents/tables.md:7-9` vs `docs/specs/documents/aggregates.md:13`

**Description:**

The spec's tables.md lists the storage table as `documents_form_downloads` (plural) and the aggregate root as `FormDownload` (singular). The spec does not document the naming convention (singular aggregate, plural table). Without an explicit convention, an implementer cannot know whether the table for `PostalDispatch` should be `documents_postal_dispatch` (singular) or `documents_postal_dispatches` (plural). The spec itself is inconsistent in adopting plurals (`_dispatches`, `_receives`).

**Expected:**

`docs/code-standards.md` § "Spec folder layout" or `docs/schemas/sql-dialects/README.md` documents the singular-aggregate → plural-table convention, with explicit examples for irregular nouns.

**Evidence:**

`docs/specs/documents/tables.md:11-14` `documents_form_downloads`, `documents_postal_dispatches`, `documents_postal_receives`. `docs/specs/documents/aggregates.md:13`, `:55`, `:79` use singular `FormDownload`, `PostalDispatch`, `PostalReceive`.

---

### FINDING 26 (id: `SPEC-2-026`)

- **Source:** `docs/audit_reports/findings/wave6-specs-2.md`
- **Severity:** Medium
- **Area:** spec-internal-inconsistency
- **Location:** `docs/specs/documents/overview.md:89-92` (Aggregate Roots table) vs `docs/specs/documents/aggregates.md`

**Description:**

The overview's "Aggregate Roots" table (lines 89-92) lists `FormDownload`, `PostalDispatch`, `PostalReceive` with the descriptions "A downloadable form for parents, students, staff", "A postal item dispatched by the school", "A postal item received by the school". The aggregates file's Purpose sections are more verbose. The overview's table is the canonical 1-line summary; it should match the aggregates.md "Root type" + "Purpose" without abbreviation, and must use the same wording as `crates/domains/documents/src/aggregate.rs` rustdoc.

**Expected:**

The overview's Aggregate Roots table entries are copied verbatim from `aggregates.md` Purpose sections and the rustdoc on each struct, so a reader sees identical wording in all three places.

**Evidence:**

`docs/specs/documents/overview.md:89-92` (table); `docs/specs/documents/aggregates.md:8-12` FormDownload Purpose; `crates/domains/documents/src/aggregate.rs:95-98` rustdoc.

---

### FINDING 27 (id: `SPEC-2-027`)

- **Source:** `docs/audit_reports/findings/wave6-specs-2.md`
- **Severity:** Medium
- **Area:** spec-internal-inconsistency
- **Location:** `docs/specs/documents/aggregates.md:36-37` (PostalDispatch commands list) vs `docs/specs/documents/commands.md:75-135`

**Description:**

The PostalDispatch aggregate's "Commands" list (`aggregates.md:36-37`) enumerates 3 commands: `DispatchPostal`, `UpdatePostalDispatch`, `DeletePostalDispatch`. The spec commands file (`commands.md`) defines exactly these 3 commands. This is consistent. However, the aggregates.md does not document the `Capability` for each command (the commands file does at lines 80, 107, 130). For an implementer wiring RBAC checks against the aggregate boundary, the capability must be declared on the aggregate.

**Expected:**

Each aggregate's "Commands" list in `aggregates.md` includes a sub-bullet per command naming its capability (e.g., "DispatchPostal (capability: Postal.Dispatch)"), matching `commands.md`.

**Evidence:**

`docs/specs/documents/aggregates.md:36-37` lists 3 commands without capabilities. `docs/specs/documents/commands.md:80` `Capability: Postal.Dispatch`, `:107` `Capability: Postal.Update`, `:130` `Capability: Postal.Delete`.

---

### FINDING 28 (id: `SPEC-2-028`)

- **Source:** `docs/audit_reports/findings/wave6-specs-2.md`
- **Severity:** Medium
- **Area:** spec-code-drift
- **Location:** `crates/domains/documents/src/repository.rs:367 lines` vs `docs/specs/documents/repositories.md:72 lines`

**Description:**

The documents source `repository.rs` is 367 lines (5× larger than the 72-line spec repositories.md). The spec lists the repositories for 3 aggregates, but the source likely implements each repository with method signatures, error mapping, and tenant-context wiring that are not documented in the spec. Without the spec, an implementer cannot determine which methods belong to which repository and which error types each method returns.

**Expected:**

`docs/specs/documents/repositories.md` documents each repository's methods with full signatures (parameter types, return types, error types) matching `crates/domains/documents/src/repository.rs`.

**Evidence:**

`wc -l crates/domains/documents/src/repository.rs` = 367; `wc -l docs/specs/documents/repositories.md` = 72. 5:1 size ratio.

---

### FINDING 29 (id: `SPEC-2-029`)

- **Source:** `docs/audit_reports/findings/wave6-specs-2.md`
- **Severity:** Medium
- **Area:** spec-code-drift
- **Location:** `crates/domains/documents/src/query.rs:465 lines` vs `docs/specs/documents/repositories.md:72 lines` (no queries documented)

**Description:**

The documents `query.rs` is 465 lines but no spec file documents the query surface (the spec does not have a dedicated `queries.md`). The repositories.md file covers repository methods but does not enumerate query builder methods (e.g., `FormDownloadQuery::by_school`, `FormDownloadQuery::public_only`, etc.). An implementer wiring the engine's `#[derive(DomainQuery)]` macro has no spec to reconcile against.

**Expected:**

Either a new `queries.md` per spec folder, or `repositories.md` is renamed to `repositories-and-queries.md` and includes both repository trait methods and query builder methods.

**Evidence:**

`wc -l crates/domains/documents/src/query.rs` = 465. No `queries.md` exists in `docs/specs/documents/`.

---


## Specs (documents, facilities, attendance) (target id prefix: `SPEC-3`)

**Path:** `docs/specs/{documents,facilities,attendance}/`  
**Total findings:** 31 (5 critical, 16 high, 7 medium, 3 low)


### FINDING 1 (id: `SPEC-3-001`)

- **Source:** `docs/audit_reports/findings/wave6-specs-3.md`
- **Severity:** Critical
- **Area:** spec
- **Location:** `docs/specs/hr/overview.md:73`

**Description:**

The HR overview uses the legacy `Sm_` brand prefix in prose. Per `AGENTS.md` § "Brand is Educore", legacy brand references are forbidden in new spec prose. The spec narrative refers to `SmAssignClassTeacher` instead of the engine's `AssignClassTeacher` aggregate defined in `docs/specs/hr/aggregates.md:308-337`.

**Expected:**

All prose references in the HR spec use the engine's `AssignClassTeacher` aggregate name (no `Sm_` prefix).

**Evidence:**

`docs/specs/hr/overview.md:73` `9. A `Staff` cannot be deleted while active` `   `SmAssignClassTeacher`, `LeaveRequest`, or `PayrollGenerate``. The HR aggregates file at `docs/specs/hr/aggregates.md:308` is `## AssignClassTeacher` (no `Sm_` prefix).

---

### FINDING 2 (id: `SPEC-3-002`)

- **Source:** `docs/audit_reports/findings/wave6-specs-3.md`
- **Severity:** Critical
- **Area:** spec
- **Location:** `docs/specs/finance/entities.md:255`

**Description:**

The finance entities file uses the legacy `Sm_` brand prefix in prose. The engine's HR-owned aggregate is `LeaveDeductionInfo` (per `docs/specs/hr/aggregates.md:476-505`); referring to it as `SmLeaveDeductionInfo` violates the no-legacy-brand rule in `AGENTS.md`.

**Expected:**

All prose references in the finance entities file use the engine's `LeaveDeductionInfo` name (no `Sm_` prefix).

**Evidence:**

`docs/specs/finance/entities.md:255` `**ActiveStatus**. This is the typed projection of the HR-owned` `   `SmLeaveDeductionInfo` row.` The HR aggregates file at `docs/specs/hr/aggregates.md:478` is `**Root type:** `LeaveDeductionInfo``.

---

### FINDING 3 (id: `SPEC-3-003`)

- **Source:** `docs/audit_reports/findings/wave6-specs-3.md`
- **Severity:** Critical
- **Area:** spec
- **Location:** `docs/specs/hr/events.md:113-127` vs `docs/specs/academic/events.md:212`

**Description:**

The `ClassTeacherAssigned` event is defined with two different payload shapes by the HR spec and the academic spec. The HR events file (`hr/events.md:116-122`) defines the event payload with `assign_class_teacher_id, class_id, section_id, staff_id, academic_id`; the academic events file (`academic/events.md:212`) defines it as `ClassTeacherAssigned { class_section_id, staff_id, role }`. Consumers subscribing to the event cannot reconcile the payload, and the cross-domain command composition breaks.

**Expected:**

A single canonical `ClassTeacherAssigned` event payload is documented in both specs, or one domain owns the event and the other re-publishes a projection.

**Evidence:**

`docs/specs/hr/events.md:116-122` `pub struct ClassTeacherAssigned {` `    pub assign_class_teacher_id: AssignClassTeacherId,` `    pub class_id: ClassId,` `    pub section_id: SectionId,` `    pub staff_id: StaffId,` `    pub academic_id: AcademicYearId,` `}`. `docs/specs/academic/events.md:212` `- `ClassTeacherAssigned { class_section_id, staff_id, role }``.

---

### FINDING 4 (id: `SPEC-3-004`)

- **Source:** `docs/audit_reports/findings/wave6-specs-3.md`
- **Severity:** Critical
- **Area:** spec
- **Location:** `docs/specs/hr/events.md:127` vs `docs/specs/academic/events.md:213`

**Description:**

The `SubjectTeacherAssigned` event is defined with two different payload shapes by the HR spec and the academic spec. The HR events file (`hr/events.md:127`) defines the event payload as `{ class_id, section_id, subject_id, staff_id, academic_id }`; the academic events file (`academic/events.md:213`) defines it as `{ class_section_id, subject_id, staff_id }`. The cross-domain event flow for `HR → academic` cannot reconcile the payload.

**Expected:**

A single canonical `SubjectTeacherAssigned` event payload is documented in both specs.

**Evidence:**

`docs/specs/hr/events.md:127` `- `SubjectTeacherAssigned { class_id, section_id, subject_id, staff_id, academic_id }``. `docs/specs/academic/events.md:213` `- `SubjectTeacherAssigned { class_section_id, subject_id, staff_id }``.

---

### FINDING 5 (id: `SPEC-3-005`)

- **Source:** `docs/audit_reports/findings/wave6-specs-3.md`
- **Severity:** Critical
- **Area:** spec
- **Location:** `docs/specs/finance/events.md:468` vs `docs/specs/hr/events.md:242`

**Description:**

The terminal `PayrollPaid` event is defined with two different field names by the finance and HR specs, despite being the cross-domain coordination event between the two domains. The finance events file (`finance/events.md:468`) defines the event as `{ payroll_generate_id, paid_amount, payment_date }`; the HR events file (`hr/events.md:242`) defines it as `{ payroll_generate_id, paid_amount, paid_at }`. The HR `MarkPayrollPaid` command (hr/commands.md:494-511) is the HR-side ack of this finance event, but it cannot bind to a single payload shape.

**Expected:**

A single canonical `PayrollPaid` event payload is documented in both specs, using one timestamp field name (either `payment_date` or `paid_at`).

**Evidence:**

`docs/specs/finance/events.md:468` `- `PayrollPaid { payroll_generate_id, paid_amount, payment_date }``. `docs/specs/hr/events.md:242` `- `PayrollPaid { payroll_generate_id, paid_amount, paid_at }``. The HR-side ack is at `docs/specs/hr/commands.md:511` `**Effects:** Emits `PayrollPaid`.`.

---

### FINDING 10 (id: `SPEC-3-010`)

- **Source:** `docs/audit_reports/findings/wave6-specs-3.md`
- **Severity:** High
- **Area:** spec
- **Location:** `docs/specs/finance/commands.md:151,474,687,914` and `docs/specs/finance/services.md:32-44,137-143` vs `docs/specs/finance/value-objects.md` (entire file)

**Description:**

The finance spec references seven value-object types that are not declared in `docs/specs/finance/value-objects.md`: `CloseReason` (commands.md:151), `ReferenceId` (commands.md:474,687), `ChartAccountType` (commands.md:914), `FmFeesInvoiceDraft` (services.md:32,39,41,42,43,44), `ReconciliationMatch` (services.md:137,143), `ReconciliationReport` (services.md:138). Consumers implementing the spec cannot construct these types because no definition (constraints, fields) is given.

**Expected:**

Each of the seven types is declared in `finance/value-objects.md` with its constraints or enum variants, or a documented dependency points to a cross-domain value object.

**Evidence:**

`docs/specs/finance/commands.md:151` `    pub reason: CloseReason,`. `docs/specs/finance/commands.md:474` `    pub reference_id: Option<ReferenceId>,`. `docs/specs/finance/commands.md:687` `    pub reference_id: Option<ReferenceId>,`. `docs/specs/finance/commands.md:914` `    pub account_type: ChartAccountType, // asset, liability, income, expense, equity`. `docs/specs/finance/services.md:32` `    ) -> Vec<FmFeesInvoiceDraft> { ... }`. `docs/specs/finance/services.md:137` `    pub fn match_statement(stmt: &BankStatement, payments: &[FeesPayment], slips: &[BankPaymentSlip]) -> ReconciliationMatch { ... }`. `docs/specs/finance/services.md:138` `    pub fn build_reconciliation_report(school: SchoolId, from: NaiveDate, to: NaiveDate) -> ReconciliationReport { ... }`. None of these types appear in `finance/value-objects.md`'s "Identifiers", "Money", "Percentages & Rounding", "Invoice & Receipt", "Payment Status", "Payment Method", "Bank", "Discount", "Payroll", "Carry Forward", "Reminder", "Login Prevention", "Installment Credit", "Question Bank Fees", "Status Enums", "Donor", "Inventory", "Bank Statement", or "School Identity Bindings" tables.

---

### FINDING 11 (id: `SPEC-3-011`)

- **Source:** `docs/audit_reports/findings/wave6-specs-3.md`
- **Severity:** High
- **Area:** spec
- **Location:** `docs/specs/finance/services.md:43` vs `docs/specs/finance/value-objects.md:250`

**Description:**

`InvoiceNumberingService::number_invoice` (services.md:43) takes a `school: &School` parameter, but `School` is not a defined value object. The only school-typed binding declared in the finance value-objects file is `SchoolId` (line 250, "From `educore-platform`"). The signature mixes the wrong type, so the service is inconsistent with the rest of the spec which uses `SchoolId`.

**Expected:**

`number_invoice` takes `school: &SchoolContext` or `school_id: SchoolId`, matching the rest of the finance spec's school typing.

**Evidence:**

`docs/specs/finance/services.md:43` `    pub fn number_invoice(school: &School, draft: &FmFeesInvoiceDraft) -> InvoiceNumber { ... }`. `docs/specs/finance/value-objects.md:250` `| `SchoolId`            | From `educore-platform`                                     |`. The rest of the finance services file uses `SchoolId`: `services.md:138` `pub fn build_reconciliation_report(school: SchoolId, from: NaiveDate, to: NaiveDate) -> ReconciliationReport`.

---

### FINDING 12 (id: `SPEC-3-012`)

- **Source:** `docs/audit_reports/findings/wave6-specs-3.md`
- **Severity:** High
- **Area:** spec
- **Location:** `docs/specs/hr/commands.md:82,443-444,654,397` and `docs/specs/hr/services.md:14,93-95,145-146` vs `docs/specs/hr/value-objects.md` (entire file)

**Description:**

The HR spec references five value-object types that are not declared in `docs/specs/hr/value-objects.md`: `StaffProfilePatch` (commands.md:82, services.md:14), `PayrollEarningLine` and `PayrollDeductionLine` (commands.md:443,444), `StaffImportRow` (commands.md:654, services.md:145,146), `StaffAttendanceImportRow` (commands.md:397, services.md:93-95). The value-objects file's tables ("Identifiers", "Names & Identity", "Salary & Money", "Leave", "Attendance", "Dates", "Status Enums", "School Identity Bindings") declare no such patch or row types.

**Expected:**

Each of the five types is declared in `hr/value-objects.md` with its fields (or referenced from a cross-crate value-object catalog) so the HR command/service signatures can be satisfied.

**Evidence:**

`docs/specs/hr/commands.md:82` `    pub patch: StaffProfilePatch,`. `docs/specs/hr/commands.md:443-444` `    pub earnings: Vec<PayrollEarningLine>,` `    pub deductions: Vec<PayrollDeductionLine>,`. `docs/specs/hr/commands.md:397` `    pub rows: Vec<StaffAttendanceImportRow>,`. `docs/specs/hr/commands.md:654` `    pub rows: Vec<StaffImportRow>,`. `docs/specs/hr/services.md:14` `    pub fn apply_patch(staff: &mut Staff, patch: StaffProfilePatch) -> Result<(), ValidationError> { ... }`. `docs/specs/hr/services.md:93-95` `    pub fn parse_csv(rows: Vec<Vec<String>>) -> Vec<StaffAttendanceImportRow> { ... }` `    pub fn validate(row: &StaffAttendanceImportRow) -> Result<(), ValidationError> { ... }` `    pub fn dedupe(rows: Vec<StaffAttendanceImportRow>) -> Vec<StaffAttendanceImportRow> { ... }`. `docs/specs/hr/services.md:145-146` `    pub fn validate_row(row: &StaffImportRow) -> Result<(), ValidationError> { ... }` `    pub fn normalize(row: &StaffImportRow) -> StaffImportRow { ... }`. None of `StaffProfilePatch`, `PayrollEarningLine`, `PayrollDeductionLine`, `StaffImportRow`, `StaffAttendanceImportRow` appear in `hr/value-objects.md`.

---

### FINDING 13 (id: `SPEC-3-013`)

- **Source:** `docs/audit_reports/findings/wave6-specs-3.md`
- **Severity:** High
- **Area:** spec
- **Location:** `docs/specs/finance/overview.md:120,150` vs `docs/specs/finance/aggregates.md:671-719`

**Description:**

The finance overview's "Aggregate Roots" table lists `FeesInvoiceSetting` twice (lines 120 and 150). The aggregates file at `finance/aggregates.md:671-696` defines a single `FeesInvoiceSetting` aggregate; the duplicate row in the overview (line 150, with the annotation `(above, listed again)`) is a documentation bug that inflates the aggregate count and risks double-listing in any cross-spec consistency check.

**Expected:**

The "Aggregate Roots" table in `finance/overview.md` lists each of the 51 unique aggregates exactly once.

**Evidence:**

`docs/specs/finance/overview.md:120` `| FeesInvoiceSetting              | `FeesInvoiceSetting`      | Classic invoice layout settings                      |`. `docs/specs/finance/overview.md:150` `| FeesInvoiceSetting              | `FeesInvoiceSetting`      | (above, listed again)                                |`. The aggregates file at `docs/specs/finance/aggregates.md:671-696` has only one `## FeesInvoiceSetting` heading; the duplicate row at line 150 contradicts the actual aggregate count of 51 unique roots in `aggregates.md` (verified by `grep -c '^## ' finance/aggregates.md` = 51).

---

### FINDING 14 (id: `SPEC-3-014`)

- **Source:** `docs/audit_reports/findings/wave6-specs-3.md`
- **Severity:** High
- **Area:** spec
- **Location:** `docs/specs/finance/tables.md:91`

**Description:**

The finance tables file's closing note claims "Total finance tables: 52 (one per aggregate; see Coverage Matrix in build-plan.md)". The actual table inventory in `finance/tables.md` lists only 39 distinct `hr_*` and `finance_*` tables (verified by `grep -cE '^\| `(hr_|finance_)[a-z_0-9]+`' finance/tables.md` = 39). The aggregate count in `finance/aggregates.md` is 51 unique aggregates. Both 39 and 51 contradict the "52 tables" claim.

**Expected:**

The closing note either matches the actual count or is removed if no precise per-aggregate table mapping exists.

**Evidence:**

`docs/specs/finance/tables.md:91` `**Total finance tables: 52 (one per aggregate; see Coverage Matrix in build-plan.md).**`. `docs/specs/finance/tables.md:10-58` lists 49 table rows total (mix of `finance_*` and `hr_*` engine tables plus cross-domain `assessment_*` and `chart_of_accounts` rows); only 39 of them have an `hr_` or `finance_` engine-owned prefix. The aggregate file at `docs/specs/finance/aggregates.md` has 51 unique `## Aggregate` headings.

---

### FINDING 15 (id: `SPEC-3-015`)

- **Source:** `docs/audit_reports/findings/wave6-specs-3.md`
- **Severity:** High
- **Area:** spec
- **Location:** `docs/specs/finance/tables.md:22` vs `docs/specs/finance/aggregates.md:618-640`

**Description:**

The finance tables file lists the child-transaction table as `finance_fees_transcation_children` (line 22, with the word "transcation" instead of "transaction"). The corresponding aggregate at `finance/aggregates.md:618-640` is `FmFeesTransactionChild` (correct spelling). The migration's table name must match the aggregate's underlying storage name.

**Expected:**

The table name in `finance/tables.md` matches the aggregate spelling: `finance_fees_transaction_children` (or whatever the engine's canonical name is).

**Evidence:**

`docs/specs/finance/tables.md:22` `| `finance_fees_transcation_children`             | FmFeesTransactionChild                   | Newer FM transaction line                    |`. `docs/specs/finance/aggregates.md:619-620` `## FmFeesTransactionChild` `**Identity:** `FmFeesTransactionChildId(SchoolId, Uuid)``. `docs/specs/finance/aggregates.md:630` `2. Belongs to one `FmFeesTransaction`.` — confirming the canonical name is `transaction`, not `transcation`.

---

### FINDING 16 (id: `SPEC-3-016`)

- **Source:** `docs/audit_reports/findings/wave6-specs-3.md`
- **Severity:** High
- **Area:** spec
- **Location:** `docs/specs/hr/repositories.md:272,275` vs `docs/specs/hr/aggregates.md:242-272`

**Description:**

The HR repositories file's recommended PostgreSQL indexes contain typos in two index names: line 272 names the index `ux_hr_staff_attendences_school_id_staff_date` (with `attendences` instead of `attendances`), and line 275 references the column `attendence_date` (also missing one `a`). The corresponding HR aggregate at `aggregates.md:255` and the column at `tables.md:16` are spelled `StaffAttendance` and `attendance_date`. Migrations generated from these index names will not match the engine's actual storage column.

**Expected:**

The index names and column references in `hr/repositories.md` use the canonical spelling: `attendance` (with two `a`s between `tend` and `nce`).

**Evidence:**

`docs/specs/hr/repositories.md:272` `CREATE UNIQUE INDEX ux_hr_staff_attendences_school_id_staff_date ON hr_staff_attendances (school_id, staff_id, attendance_date);`. `docs/specs/hr/repositories.md:275` `CREATE INDEX ix_attendance_staff_attendance_imports_school_id_date ON attendance_staff_attendance_imports (school_id, attendence_date);`. `docs/specs/hr/aggregates.md:245` `**Root type:** `StaffAttendance``. `docs/specs/hr/aggregates.md:258` `   `attendance_date` is required.`. `docs/specs/hr/tables.md:16` `| `hr_staff_attendances`             | StaffAttendance           | A daily attendance row               |`.

---

### FINDING 17 (id: `SPEC-3-017`)

- **Source:** `docs/audit_reports/findings/wave6-specs-3.md`
- **Severity:** High
- **Area:** spec
- **Location:** `docs/specs/finance/permissions.md:119` vs `docs/specs/finance/commands.md` (entire file)

**Description:**

The finance permissions file advertises the `Bank.Statement.Reverse` capability (line 119), but the corresponding `ReverseBankStatementCommand` struct is not defined in `docs/specs/finance/commands.md`. The capability exists only as a referenced capability without a command implementation. The `BankStatement` aggregate at `finance/aggregates.md:802-806` lists the command name, but no struct is provided.

**Expected:**

Either the `Bank.Statement.Reverse` capability is removed from `permissions.md`, or `ReverseBankStatementCommand` is defined in `commands.md` with a payload, pre-conditions, and the `Bank.Statement.Reverse` capability tag (matching the `RecordBankStatementCommand` shape at `commands.md:465-481`).

**Evidence:**

`docs/specs/finance/permissions.md:119` `- `Bank.Statement.Reverse``. `docs/specs/finance/aggregates.md:805` `- `ReverseBankStatement``. `docs/specs/finance/commands.md` defines `RecordBankStatement` (line 463) but no `ReverseBankStatement` or `ReverseBankStatementCommand` section.

---

### FINDING 18 (id: `SPEC-3-018`)

- **Source:** `docs/audit_reports/findings/wave6-specs-3.md`
- **Severity:** High
- **Area:** spec
- **Location:** `docs/specs/finance/permissions.md:189` vs `docs/specs/finance/commands.md` (entire file)

**Description:**

The finance permissions file advertises the `QuestionBank.Fee.Detach` capability (line 189), and the `QuestionBankFee` aggregate at `finance/aggregates.md:1333` lists `DetachFeesFromQuestionBank` as a command, but the corresponding `DetachFeesFromQuestionBankCommand` struct is not defined in `docs/specs/finance/commands.md`. The mirror command `AttachFeesToQuestionBankCommand` is defined at `commands.md:891-905`, so the asymmetry between attach and detach implementations is also a gap.

**Expected:**

`DetachFeesFromQuestionBankCommand` is defined in `commands.md` symmetric to `AttachFeesToQuestionBankCommand`, with the `QuestionBank.Fee.Detach` capability and the `FeesDetachedFromQuestionBank` event effect.

**Evidence:**

`docs/specs/finance/permissions.md:189` `- `QuestionBank.Fee.Detach``. `docs/specs/finance/aggregates.md:1333` `- `DetachFeesFromQuestionBank``. `docs/specs/finance/commands.md` defines `AttachFeesToQuestionBankCommand` at lines 891-905 but no `DetachFeesFromQuestionBank` section.

---

### FINDING 19 (id: `SPEC-3-019`)

- **Source:** `docs/audit_reports/findings/wave6-specs-3.md`
- **Severity:** High
- **Area:** spec
- **Location:** `docs/specs/finance/commands.md` (FM section absent) vs `docs/specs/finance/aggregates.md:524-745`

**Description:**

The finance commands file does not document the 11 FM-invoice-scheme commands listed in `finance/aggregates.md`: `GenerateFmFeesInvoice`, `UpdateFmFeesInvoiceStatus`, `CancelFmFeesInvoice`, `AddFmFeesInvoiceLine`, `UpdateFmFeesInvoiceLine`, `RemoveFmFeesInvoiceLine`, `RecordFmFeesTransaction`, `ReverseFmFeesTransaction`, `AddFmFeesTransactionLine`, `ApplyFmFeesWeaver`, `ReverseFmFeesWeaver` (plus the FM group/type/settings CRUD commands `CreateFmFeesGroup`, `UpdateFmFeesGroup`, `DeleteFmFeesGroup`, `CreateFmFeesType`, `UpdateFmFeesType`, `DeleteFmFeesType`, `ConfigureFmFeesInvoiceSetting`, `ConfigureFeesInvoiceSetting`, `ConfigureInvoiceSetting`). These aggregates have full event, repository, and service coverage but no command struct definitions, breaking the spec's completeness.

**Expected:**

Each FM-scheme aggregate in `finance/aggregates.md` has a matching command struct in `finance/commands.md` with a payload, pre-conditions, capability, and effects.

**Evidence:**

`docs/specs/finance/aggregates.md:544-546` lists `GenerateFmFeesInvoice`, `UpdateFmFeesInvoiceStatus`, `CancelFmFeesInvoice` under the `FmFeesInvoice` aggregate. `docs/specs/finance/aggregates.md:575-577` lists `AddFmFeesInvoiceLine`, `UpdateFmFeesInvoiceLine`, `RemoveFmFeesInvoiceLine` under `FmFeesInvoiceChild`. `docs/specs/finance/aggregates.md:608-609` lists `RecordFmFeesTransaction`, `ReverseFmFeesTransaction` under `FmFeesTransaction`. `docs/specs/finance/aggregates.md:635` lists `AddFmFeesTransactionLine` under `FmFeesTransactionChild`. `docs/specs/finance/aggregates.md:661-662` lists `ApplyFmFeesWeaver`, `ReverseFmFeesWeaver` under `FmFeesWeaver`. `docs/specs/finance/aggregates.md:482-484,512-514` list FM group/type CRUD commands. `docs/specs/finance/commands.md` contains no `### GenerateFmFeesInvoice`, `### AddFmFeesInvoiceLine`, `### RecordFmFeesTransaction`, etc. sections. The only FM-adjacent section in commands.md is `### ConfigureInvoiceSettings` (line 816), which emits `InvoiceSettingConfigured` or `FeesInvoiceSettingConfigured` (commands.md:845-846) — covering only the settings aggregates.

---

### FINDING 20 (id: `SPEC-3-020`)

- **Source:** `docs/audit_reports/findings/wave6-specs-3.md`
- **Severity:** High
- **Area:** spec
- **Location:** `docs/specs/finance/commands.md` (multiple aggregates) vs `docs/specs/finance/aggregates.md` (CRUD commands)

**Description:**

Beyond the FM scheme (covered by SPEC-3-019), the finance commands file omits update and delete variants for many aggregates. The aggregates file lists `UpdateFeesType`/`DeleteFeesType` (line 63-64), `UpdateFeesDiscount`/`DeleteFeesDiscount` (line 203-204), `UpdateFeesInstallment`/`DeleteFeesInstallment` (line 266-267), `UpdateDirectFeesInstallment`/`DeleteDirectFeesInstallment` (line 396-397), `UpdateBankAccount`/`CloseBankAccount` (line 770-771), `UpdateExpense`/`DeleteExpense` (line 869-870), `UpdateIncome`/`DeleteIncome` (line 901-902), `UpdateChartOfAccount`/`DeleteChartOfAccount` (line 1303-1304), `UpdatePaymentMethod`/`DeletePaymentMethod` (line 1395-1396), `UpdateFeesReminder`/`DeleteFeesReminder` (line 1423-1424), `UpdateSalaryTemplate`/`DeleteSalaryTemplate` (line 1187-1188), `UpdateHourlyRate`/`DeleteHourlyRate` (line 357-358), `UpdatePayrollEarnDeduc`/`DeletePayrollEarnDeduc` (line 1154-1155), and others — but the commands file provides struct definitions only for a subset (`CreateExpenseHead / UpdateExpenseHead / DeleteExpenseHead` at line 607 is abbreviated and lacks struct bodies; `RegisterDonor / UpdateDonor / DeleteDonor` at line 622 is also abbreviated).

**Expected:**

Each update/delete command listed in `aggregates.md` has a struct definition in `commands.md`, even if abbreviated, with a payload type, capability, and effects line.

**Evidence:**

`docs/specs/finance/aggregates.md:63-64` `- `UpdateFeesType``; `- `DeleteFeesType``. `docs/specs/finance/commands.md:38-51` defines only `### CreateFeesType` (line 38) and no `### UpdateFeesType` or `### DeleteFeesType` sections. Same pattern for `FeesDiscount`, `FeesInstallment`, `DirectFeesInstallment`, `BankAccount`, `Expense`, `Income`, `Donor` (abbreviated), `ChartOfAccount` (abbreviated), `PaymentMethod` (abbreviated), `DirectFeesReminder`, `SalaryTemplate`, `HourlyRate`, `PayrollEarnDeduc`.

---

### FINDING 21 (id: `SPEC-3-021`)

- **Source:** `docs/audit_reports/findings/wave6-specs-3.md`
- **Severity:** High
- **Area:** spec
- **Location:** `docs/specs/library/events.md:113` and `docs/specs/library/commands.md:133,211` vs `docs/specs/library/tables.md:34,57,69,85`

**Description:**

The library events and commands use the field name `academic_year_id`, but the library tables file uses the column name `academic_id` in the field-mapping tables. The mismatch is internal to the library spec; for example `LibraryMemberRegistered` (`events.md:113`) carries `pub academic_year_id: AcademicYearId`, but the storage column at `tables.md:69` is `academic_id`. The HR spec (`hr/tables.md`) consistently uses `academic_id`, so consumers cannot rely on a single naming convention.

**Expected:**

The library spec consistently uses one column name (either `academic_id` or `academic_year_id`) across events, commands, and the tables file.

**Evidence:**

`docs/specs/library/events.md:113` `    pub academic_year_id: AcademicYearId,`. `docs/specs/library/commands.md:133,211` `    pub academic_year_id: AcademicYearId,`. `docs/specs/library/tables.md:34` `| `academic_id`     | `u64`               | `AcademicYearId`                     |`. `docs/specs/library/tables.md:57` `| `academic_id`       | `u64`               | `AcademicYearId`                     |`. `docs/specs/library/tables.md:69` `| `academic_id`         | `u64`               | `AcademicYearId`                     |`. `docs/specs/library/tables.md:85` `| `academic_id`       | `u64`               | `AcademicYearId`                     |`. HR tables at `docs/specs/hr/tables.md:35` use `academic_id` uniformly.

---

### FINDING 6 (id: `SPEC-3-006`)

- **Source:** `docs/audit_reports/findings/wave6-specs-3.md`
- **Severity:** High
- **Area:** spec
- **Location:** `docs/specs/hr/permissions.md:14` vs `docs/specs/hr/permissions.md:37-39`

**Description:**

The HR permissions file's "Naming" section lists `BulkImport.*` as the capability prefix used by the HR domain, but the actual capabilities listed under the `### Staff` section use `Staff.ImportBulk`, `Staff.ImportBulk.Promote`, and `Staff.ImportBulk.Reject`. The "Naming" block therefore misleads readers about the actual namespace the engine enforces, and no `BulkImport.*` capabilities are defined anywhere else in the HR spec.

**Expected:**

The HR permissions "Naming" section lists the actual prefix (`Staff.ImportBulk.*`) used by the bulk-import commands, or the bulk-import capabilities are renamed to a `BulkImport.*` namespace consistently.

**Evidence:**

`docs/specs/hr/permissions.md:14` ``StaffRegistrationField.*`, `BulkImport.*`,``. `docs/specs/hr/permissions.md:37-39` `- `Staff.ImportBulk``; `- `Staff.ImportBulk.Promote``; `- `Staff.ImportBulk.Reject``. The HR commands file at `docs/specs/hr/commands.md:658,676,693` uses the same `Staff.ImportBulk*` capabilities on the corresponding `ImportStaffBulkCommand`, `PromoteStaffImportCommand`, and `RejectStaffImportCommand`.

---

### FINDING 7 (id: `SPEC-3-007`)

- **Source:** `docs/audit_reports/findings/wave6-specs-3.md`
- **Severity:** High
- **Area:** spec
- **Location:** `docs/specs/library/permissions.md:103-119` vs `docs/specs/library/events.md` (entire file)

**Description:**

The library spec references a `FineWaived` event from three locations (`permissions.md:117`, `services.md:103`, plus the workflow at `workflows.md:128-130` which calls `WaiveBookIssueFine`), but the `FineWaived` event is not defined anywhere in `docs/specs/library/events.md`. The events file's only fine-related event is `FineCalculated` at line 223.

**Expected:**

The library events file defines a `FineWaived` event with the same payload semantics implied by `BookIssueFine.Waived` (per `entities.md:50-51`) and the `FineCalculationService::apply_waiver` (per `services.md:80-83`).

**Evidence:**

`docs/specs/library/permissions.md:117` `**Effects:** Emits `FineWaived` and updates the`. `docs/specs/library/services.md:103` `history entry and emits a `FineWaived` event.`. `docs/specs/library/events.md` defines `FineCalculated` at line 223 only; no `FineWaived` event is declared.

---

### FINDING 8 (id: `SPEC-3-008`)

- **Source:** `docs/audit_reports/findings/wave6-specs-3.md`
- **Severity:** High
- **Area:** spec
- **Location:** `docs/specs/library/permissions.md:108-119` and `docs/specs/library/workflows.md:128-130` vs `docs/specs/library/commands.md` (entire file)

**Description:**

The library spec references `WaiveBookIssueFineCommand` from three locations (`permissions.md:108-119`, `workflows.md:128-130`, and indirectly via the `BookIssue.WaiveFine` capability in `permissions.md:58,103`), but no `WaiveBookIssueFineCommand` struct is defined in `docs/specs/library/commands.md`. The library `BookIssue` aggregate (`aggregates.md:175`) lists only `IssueBook`, `ReturnBook`, `RenewBook`, `MarkBookLost`, `CalculateFine` as commands — no `WaiveBookIssueFine`.

**Expected:**

The library commands file defines `WaiveBookIssueFineCommand` with a payload, pre-conditions, and `Capability: BookIssue.WaiveFine`, and the `BookIssue` aggregate in `aggregates.md` lists it under the aggregate's Commands section.

**Evidence:**

`docs/specs/library/permissions.md:108-119` documents the full `pub struct WaiveBookIssueFineCommand { ... }` shape with `book_issue_fine_id: BookIssueFineId, reason: String` and the `BookIssue.WaiveFine` capability. `docs/specs/library/aggregates.md:175` lists the `BookIssue` aggregate's commands as `- `IssueBook``, `- `ReturnBook``, `- `RenewBook``, `- `MarkBookLost``, `- `CalculateFine``. `docs/specs/library/commands.md` has no `## WaiveBookIssueFine` or `### WaiveBookIssueFine` section.

---

### FINDING 9 (id: `SPEC-3-009`)

- **Source:** `docs/audit_reports/findings/wave6-specs-3.md`
- **Severity:** High
- **Area:** spec
- **Location:** `docs/specs/library/commands.md:82,155` vs `docs/specs/library/value-objects.md` (entire file)

**Description:**

The library commands reference `BookPatch` (`commands.md:82`, referenced in `commands.md:86`) and `LibraryMemberPatch` (`commands.md:155`, referenced in `commands.md:159`) but neither value object is defined in `docs/specs/library/value-objects.md`. The value-objects file's "Bibliographic", "Members", and "Issues" sections do not declare either patch type.

**Expected:**

`BookPatch` and `LibraryMemberPatch` are declared in `library/value-objects.md` (or referenced from a shared patch crate) with the documented mutable fields (`book_title, publisher_name, author_name, rack_number, book_price, post_date, details, book_category_id, book_subject_id` for `BookPatch`; `member_ud_id, note` for `LibraryMemberPatch`).

**Evidence:**

`docs/specs/library/commands.md:82` `    pub patch: BookPatch,`. `docs/specs/library/commands.md:86` `   `BookPatch` carries the mutable fields: `book_title`,`. `docs/specs/library/commands.md:155` `    pub patch: LibraryMemberPatch,`. `docs/specs/library/value-objects.md` lists value objects under "Bibliographic", "Members", "Issues", "Money & Quantities", "Status Enums", "Identity & Contact" — no `BookPatch` or `LibraryMemberPatch` row.

---

### FINDING 22 (id: `SPEC-3-022`)

- **Source:** `docs/audit_reports/findings/wave6-specs-3.md`
- **Severity:** Medium
- **Area:** spec
- **Location:** `docs/specs/library/aggregates.md:99-101` vs `docs/specs/library/value-objects.md:56-58`

**Description:**

The library aggregates file uses the term `StudentStaffId` (line 99) for the polymorphic member reference, but the library value-objects file uses the term `MemberId` for the same concept (line 56: `enum `Student(StudentId)` or `Staff(StaffId)`). The aggregates file's later description (line 106) does acknowledge the storage column `student_staff_id`, but the domain-level term is inconsistent between the two files.

**Expected:**

The library spec uses one domain-level term (`MemberId` is the value-object name; the aggregates file should refer to it consistently) and one storage column name (`student_staff_id`).

**Evidence:**

`docs/specs/library/aggregates.md:99-101` `A registered borrower. May be a student or a staff member. Each` `member has a `MemberType` (from the role catalog) and a` `   `StudentStaffId` (the underlying user id from the platform).`. `docs/specs/library/value-objects.md:56` `| `MemberId`        | enum `Student(StudentId)` or `Staff(StaffId)`            |`. `docs/specs/library/value-objects.md:67` `| `student_staff_id`  | `u64`               | `MemberId` (StudentId or StaffId)    |`. The aggregates file uses `StudentStaffId` at line 99 (no entry in value-objects.md under that name) and `student_staff_id` at line 106.

---

### FINDING 23 (id: `SPEC-3-023`)

- **Source:** `docs/audit_reports/findings/wave6-specs-3.md`
- **Severity:** Medium
- **Area:** spec
- **Location:** `docs/specs/hr/permissions.md:40-41` vs `docs/specs/hr/commands.md` (entire file) and `docs/specs/hr/aggregates.md:42-69`

**Description:**

The HR permissions file lists `Staff.Document.Upload` and `Staff.Document.Download` capabilities (lines 40-41) but the HR commands file contains no `UploadStaffDocumentCommand` or `DownloadStaffDocumentCommand` struct. The `Staff` aggregate at `aggregates.md:42-69` lists only `RegisterStaff`, `UpdateStaff`, role-change, status-change, and `DeleteStaff` commands. The `StaffDocument` entity at `entities.md:42-50` is documented as a sub-entity but has no command surface.

**Expected:**

Either the `Staff.Document.Upload` and `Staff.Document.Download` capabilities are removed from `permissions.md`, or matching `UploadStaffDocumentCommand` and `DownloadStaffDocumentCommand` structs are added to `commands.md` (with the `StaffDocument` aggregate or the `Staff` aggregate as the owner).

**Evidence:**

`docs/specs/hr/permissions.md:40-41` `- `Staff.Document.Upload``; `- `Staff.Document.Download``. `docs/specs/hr/aggregates.md:42-54` lists the `Staff` aggregate's commands (no upload/download). `docs/specs/hr/commands.md` has no `### UploadStaffDocument` or `### DownloadStaffDocument` section. The `StaffDocument` entity is defined at `docs/specs/hr/entities.md:42-50` but is not an aggregate, so a command must attach to an aggregate root.

---

### FINDING 24 (id: `SPEC-3-024`)

- **Source:** `docs/audit_reports/findings/wave6-specs-3.md`
- **Severity:** Medium
- **Area:** spec
- **Location:** `docs/specs/hr/permissions.md:34-36` vs `docs/specs/hr/commands.md:209-239`

**Description:**

The HR permissions file uses three sub-namespaced capabilities (`Staff.AssignClassTeacher.Create`, `Staff.AssignClassTeacher.Update`, `Staff.AssignClassTeacher.Delete` at lines 34-36), but the corresponding command in `commands.md` is a single `AssignClassTeacherCommand` (line 210) and a single `UpdateAssignClassTeacherCommand` (line 229); there is no `DeleteAssignClassTeacherCommand` struct in `commands.md` even though the `AssignClassTeacher` aggregate at `aggregates.md:329` lists `DeleteAssignClassTeacher` as a command. The capability namespace implies three discrete commands but the commands file merges them or omits them.

**Expected:**

The HR permissions file lists either the merged `Staff.AssignClassTeacher` capability (matching the actual commands) or the three sub-namespaced capabilities match three distinct command structs (Create/Update/Delete).

**Evidence:**

`docs/specs/hr/permissions.md:34-36` `- `Staff.AssignClassTeacher.Create``; `- `Staff.AssignClassTeacher.Update``; `- `Staff.AssignClassTeacher.Delete``. `docs/specs/hr/commands.md:209-239` defines `AssignClassTeacherCommand` (line 210) and `UpdateAssignClassTeacherCommand` (line 229) only. `docs/specs/hr/aggregates.md:329` `- `DeleteAssignClassTeacher`` has no matching struct in `commands.md`.

---

### FINDING 25 (id: `SPEC-3-025`)

- **Source:** `docs/audit_reports/findings/wave6-specs-3.md`
- **Severity:** Medium
- **Area:** spec
- **Location:** `docs/specs/finance/value-objects.md:73-89` vs `docs/specs/finance/aggregates.md:749-811` (Bank section)

**Description:**

The finance value-objects file declares `BankAccountNumber` as `6..34 chars, alphanumeric` (line 142), `IfscCode` as `11 chars, format `[A-Z]{4}0[A-Z0-9]{6}`` (line 143), and `ChequeNumber` as `6 digits` (line 144), but these value objects are not declared on the `OpenBankAccountCommand` (commands.md:445-461) or `BankPaymentSlip`-related commands (commands.md:482-537). The `BankAccount` aggregate at `aggregates.md:755-758` lists only `bank_name`, `account_name`, `account_number`, `account_type`, `opening_balance`, and `note` as fields, with no reference to `IfscCode` or `ChequeNumber`. The IFSC and cheque-number constraints therefore cannot be enforced on `BankPaymentSlip` (aggregates.md:825: only `payment_mode` is enumerated as `Bk` or `Cq`).

**Expected:**

Either `IfscCode` and `ChequeNumber` are used in the bank commands/aggregates (with validation), or they are removed from `value-objects.md` if they are not first-class concerns of the finance domain.

**Evidence:**

`docs/specs/finance/value-objects.md:142-144` `| `BankAccountNumber`  | 6..34 chars, alphanumeric                                         |` `| `IfscCode`           | 11 chars, format `[A-Z]{4}0[A-Z0-9]{6}``                           |` `| `ChequeNumber`       | 6 digits                                                          |`. `docs/specs/finance/commands.md:445-461` `pub struct OpenBankAccountCommand { ... pub bank_name: String, ... pub account_number: BankAccountNumber, pub account_type: AccountType, ... }` (no `IfscCode`, no `ChequeNumber`). `docs/specs/finance/aggregates.md:825-826` `1. `payment_mode` is `Bk` (bank transfer) or `Cq` (cheque).` (no `ChequeNumber` validation).

---

### FINDING 26 (id: `SPEC-3-026`)

- **Source:** `docs/audit_reports/findings/wave6-specs-3.md`
- **Severity:** Medium
- **Area:** spec
- **Location:** `docs/specs/finance/aggregates.md:1140-1147` vs `docs/specs/hr/aggregates.md:451-456`

**Description:**

The `PayrollEarnDeduc` aggregate is declared in both the finance aggregates file (lines 1132-1163, "typed here") and the HR aggregates file (lines 443-473) with conflicting ownership semantics. The finance aggregates file declares it as a finance-owned aggregate (`**Root type:** `PayrollEarnDeduc`` at line 1134), while the HR aggregates file declares it as an HR-owned aggregate (`**Root type:** `PayrollEarnDeduc`` at line 445). The finance overview at line 135 says it is "typed here" (in finance), but the HR overview at line 42 says the cross-domain bridge "happens through the `PayrollGenerate` and `PayrollEarnDeduc` aggregates (HR-owned writes; finance reads and pays)". The ownership is contradictory.

**Expected:**

A single canonical owner for `PayrollEarnDeduc` is declared in both spec files. Per the cross-domain narrative (HR-owned writes, finance reads), the HR aggregates file should be the canonical owner, and the finance aggregates file should cross-reference it as a read-only view.

**Evidence:**

`docs/specs/finance/aggregates.md:1132-1138` `## PayrollEarnDeduc` `**Root type:** `PayrollEarnDeduc`` `**Identity:** `PayrollEarnDeducId(SchoolId, Uuid)``. `docs/specs/finance/aggregates.md:1153-1156` `- `AddPayrollEarning``; `- `AddPayrollDeduction``; `- `UpdatePayrollEarnDeduc``; `- `DeletePayrollEarnDeduc``. `docs/specs/hr/aggregates.md:443-472` `## PayrollEarnDeduc` `**Root type:** `PayrollEarnDeduc`` with the same commands (`AddPayrollEarning`, etc.) and events (`PayrollEarningAdded`, etc.). `docs/specs/hr/overview.md:42-43` `finance happens through the `PayrollGenerate` and `PayrollEarnDeduc`` `aggregates (HR-owned writes; finance reads and pays).`.

---

### FINDING 27 (id: `SPEC-3-027`)

- **Source:** `docs/audit_reports/findings/wave6-specs-3.md`
- **Severity:** Medium
- **Area:** spec
- **Location:** `docs/specs/finance/commands.md:698-718` vs `docs/specs/hr/commands.md:436-456`

**Description:**

The `GeneratePayrollCommand` struct is defined in both the finance commands file (lines 698-718, marked `(HR)` in comments at aggregates.md:1118-1123) and the HR commands file (lines 436-456). Both definitions declare identical fields (`tenant`, `staff_id`, `pay_period`, `salary_template_id`, `earnings`, `deductions`, `note`, `bank_id`, `payment_mode`) but differ in ordering and in field names (`bank_id: Option<BankAccountId>` vs the same in both, but the HR version adds no extra fields). The duplication means consumers must reconcile which is the canonical struct.

**Expected:**

`GeneratePayrollCommand` is defined once (in the HR commands file as the HR-owned write command, per the cross-domain narrative) and the finance commands file references it as the source-of-truth HR-side command (or defines a finance-side `PayPayrollCommand` for the disbursement only).

**Evidence:**

`docs/specs/finance/commands.md:700-709` `pub struct GeneratePayrollCommand {` `    pub tenant: TenantContext,` `    pub staff_id: StaffId,` `    pub pay_period: PayPeriod,` `    pub salary_template_id: Option<SalaryTemplateId>,` `    pub earnings: Vec<PayrollEarningLine>,` `    pub deductions: Vec<PayrollDeductionLine>,` `    pub note: Option<String>,` `    pub bank_id: Option<BankAccountId>,` `    pub payment_mode: Option<PaymentMethodId>,` `}`. `docs/specs/hr/commands.md:439-448` `pub struct GeneratePayrollCommand {` `    pub tenant: TenantContext,` `    pub staff_id: StaffId,` `    pub pay_period: PayPeriod,` `    pub salary_template_id: Option<SalaryTemplateId>,` `    pub earnings: Vec<PayrollEarningLine>,` `    pub deductions: Vec<PayrollDeductionLine>,` `    pub note: Option<String>,` `    pub bank_id: Option<BankAccountId>,` `    pub payment_mode: Option<PaymentMethodId>,` `}`. Identical struct shapes in both spec files.

---

### FINDING 28 (id: `SPEC-3-028`)

- **Source:** `docs/audit_reports/findings/wave6-specs-3.md`
- **Severity:** Medium
- **Area:** spec
- **Location:** `docs/specs/finance/aggregates.md:1296-1300` vs `docs/specs/finance/value-objects.md:215`

**Description:**

The `ChartOfAccount` aggregate at `finance/aggregates.md:1294` declares `account_type` as a value with five variants (`asset, liability, income, expense, equity` per the comment at `commands.md:914`), but the finance value-objects file does not declare a `ChartAccountType` enum (verified by `grep -n "ChartAccountType" finance/value-objects.md` returning no matches). The only related enum at `value-objects.md:215` is `AccountDirection` (`Debit`, `Credit`). Without a declared `ChartAccountType`, the engine cannot validate the `account_type` field on `ChartOfAccount` creation.

**Expected:**

`ChartAccountType` is declared in `finance/value-objects.md` with its five variants and constraints (e.g. distinctness, naming rules), or the value-object comment in the command is updated to reference the existing `AccountDirection`.

**Evidence:**

`docs/specs/finance/commands.md:914` `    pub account_type: ChartAccountType, // asset, liability, income, expense, equity`. `docs/specs/finance/aggregates.md:1296-1300` `1. Each `ChartOfAccount` is unique by `name` within a school.` `2. A `ChartOfAccount` cannot be deleted while any `Expense`,` `   `Income`, or `BankStatement` references it.`. `docs/specs/finance/value-objects.md:215` `| `AccountDirection`   | `Debit`, `Credit`                                                 |` — only `AccountDirection`, no `ChartAccountType`.

---

### FINDING 29 (id: `SPEC-3-029`)

- **Source:** `docs/audit_reports/findings/wave6-specs-3.md`
- **Severity:** Low
- **Area:** spec
- **Location:** `docs/specs/finance/events.md:229-249` vs `docs/specs/finance/aggregates.md:524-553`

**Description:**

The FM-invoice-scheme events file (`finance/events.md:238-249`) lists 14 FM events (`FinanceInvoiceStatusUpdated`, `FinanceInvoiceCancelled`, `FinanceInvoiceLineAdded`, `FinanceInvoiceLineUpdated`, `FinanceInvoiceLineRemoved`, `FinanceTransactionRecorded`, `FmFeesTransactionReversed`, `FmFeesTransactionLineAdded`, `FinanceWeaverApplied`, `FinanceWeaverReversed`, `FinanceFeesGroupCreated`/`Updated`/`Deleted`, `FinanceFeesTypeCreated`/`Updated`/`Deleted`), but the corresponding FM aggregates (`FmFeesInvoice`, `FmFeesInvoiceChild`, `FmFeesTransaction`, `FmFeesTransactionChild`, `FmFeesWeaver`, `FmFeesGroup`, `FmFeesType`) at `aggregates.md:524-521` list the event names with a different prefix (`FmFeesInvoiceGenerated` vs the listed `FmFeesInvoiceGenerated`; the events file uses `FinanceInvoice*` and `FinanceTransaction*` prefixes, while the aggregates file uses `FmFees*`). The same logical event is documented under two prefixes.

**Expected:**

The FM-scheme events use one prefix consistently — either `Fm*` (matching the aggregate naming) or `Finance*` (matching the events-file shorthand).

**Evidence:**

`docs/specs/finance/events.md:238-244` `- `FinanceInvoiceStatusUpdated { finance_invoice_id, status }``; `- `FinanceInvoiceCancelled { finance_invoice_id, reason }``; `- `FinanceInvoiceLineAdded { finance_invoice_id, line_id, fees_type, amount }``; `- `FinanceInvoiceLineUpdated { finance_invoice_id, line_id, changes }``; `- `FinanceInvoiceLineRemoved { finance_invoice_id, line_id }``; `- `FinanceTransactionRecorded { finance_transaction_id, finance_invoice_id, payment_method, total_paid_amount, add_wallet_money }``; `- `FmFeesTransactionReversed { finance_transaction_id, reason }``. `docs/specs/finance/aggregates.md:550-552` lists `FmFeesInvoiceGenerated`, `FinanceInvoiceStatusUpdated`, `FinanceInvoiceCancelled` under the `FmFeesInvoice` aggregate — mixing both prefixes within a single aggregate.

---

### FINDING 30 (id: `SPEC-3-030`)

- **Source:** `docs/audit_reports/findings/wave6-specs-3.md`
- **Severity:** Low
- **Area:** spec
- **Location:** `docs/specs/library/overview.md:38` vs `docs/specs/library/value-objects.md:34-35`

**Description:**

The library overview states the domain depends on `StudentId`, `StaffId`, `RoleId`, `AcademicYearId` from the academic/HR/RBAC domains, but the overview does not list the engine-internal identifier types `BookId`, `BookCategoryId`, `LibraryMemberId`, `BookIssueId` that the spec promises the library domain exposes to consumers (per the same overview at lines 39-41). Consumers of the library SDK cannot determine the canonical exported identifier names without cross-referencing `value-objects.md:14-23`.

**Expected:**

The library overview's "Dependencies" section lists both the inbound cross-domain identifiers AND the library's own identifier types that are exposed to consumers.

**Evidence:**

`docs/specs/library/overview.md:36-41` `The library domain **does** depend on identifier types defined by` `the academic and human-resource domains: `StudentId`, `StaffId`,`` `   `RoleId`, `AcademicYearId`. It exposes its own identifier types to` `   consumers: `BookId`, `BookCategoryId`, `LibraryMemberId`,`` `   `BookIssueId`.`. `docs/specs/library/overview.md:42-54` (Dependencies section) lists `educore-core`, `educore-platform`, `educore-rbac`, `educore-events`, `educore-academic`, `educore-hr`, `educore-finance`, but no `BookId`/`BookCategoryId`/`LibraryMemberId`/`BookIssueId` reference. `docs/specs/library/value-objects.md:14-23` lists these four identifiers in the "Identifiers" table.

---

### FINDING 31 (id: `SPEC-3-031`)

- **Source:** `docs/audit_reports/findings/wave6-specs-3.md`
- **Severity:** Low
- **Area:** spec
- **Location:** `docs/specs/hr/overview.md:103-135` vs `docs/specs/hr/commands.md:696-715`

**Description:**

The HR overview's "Cross-Domain Impact" section lists `StaffUnregistered` as the event emitted when a staff leaves (line 129-135), and the HR aggregates file lists the events `StaffResigned`, `StaffTerminated`, `StaffRetired` (aggregates.md:65-67) as the matching terminal transitions. The HR commands file at lines 696-715 defines `AssignSubjectTeacherCommand` but does not define a corresponding `UnregisterStaffCommand` or `WithdrawStaffCommand` — only the softer `ResignStaff`, `TerminateStaff`, and `RetireStaff` commands at `commands.md:132-150`. The overview's `StaffUnregistered` event has no producing command.

**Expected:**

Either the `StaffUnregistered` event is replaced with the existing terminal events (`StaffResigned` / `StaffTerminated` / `StaffRetired`), or a new `UnregisterStaffCommand` is added to `commands.md` that emits `StaffUnregistered`.

**Evidence:**

`docs/specs/hr/overview.md:128-135` `When a `Staff` is unregistered, the HR domain emits` `   `StaffUnregistered`. The following domains may subscribe:` `- `academic` — release any class teacher or subject teacher` `   assignment.` `- `finance` — close the staff's payroll.` `- `rbac` — revoke the staff's role.`. `docs/specs/hr/aggregates.md:65-67` lists `StaffResigned`, `StaffTerminated`, `StaffRetired` as terminal events. `docs/specs/hr/events.md:93-97` documents `StaffReinstated`, `StaffResigned`, `StaffTerminated`, `StaffRetired`, `StaffDeleted` — but no `StaffUnregistered`. The HR overview therefore references a non-existent event.

---


## Specs (cross-cutting + data-migration) (target id prefix: `SPEC-4`)

**Path:** `docs/specs/ (cross-cutting) + docs/schemas/data-migration/`  
**Total findings:** 25 (1 critical, 6 high, 11 medium, 7 low)


### FINDING 1 (id: `SPEC-4-001`)

- **Source:** `docs/audit_reports/findings/wave6-specs-4.md`
- **Severity:** Critical
- **Area:** spec
- **Location:** `docs/specs/sync/` (directory listing)

**Description:**

The `docs/specs/sync/` spec folder contains only `overview.md` (1162 lines). The other 10 spec files (`aggregates.md`, `entities.md`, `value-objects.md`, `commands.md`, `events.md`, `services.md`, `permissions.md`, `repositories.md`, `workflows.md`, `tables.md`) are absent. Per `docs/code-standards.md` § "Spec folder layout" each spec folder must contain exactly 11 files.

**Expected:**

All 11 spec files present in `docs/specs/sync/`, including dedicated `commands.md`, `events.md`, and `tables.md` files documenting the sync aggregates (`OutboxEntry`, `SyncCursor`, `ConflictRecord`, `SyncSubscription`).

**Evidence:**

`docs/specs/sync/`: `total 52` containing `-rw-rw-r-- 1 beznet beznet 43150 Jun 13 09:32 overview.md` only; `ls /home/beznet/Workspace/smscore/docs/specs/sync/` returns `overview.md` as the sole file.

---

### FINDING 2 (id: `SPEC-4-002`)

- **Source:** `docs/audit_reports/findings/wave6-specs-4.md`
- **Severity:** High
- **Area:** spec
- **Location:** `docs/specs/operations/commands.md:152,166` vs `docs/specs/operations/permissions.md:33,39,41`

**Description:**

The namespace for the failed-job retry and purge capabilities is inconsistent within the operations spec. `commands.md` advertises `Operations.Job.Retry` (line 152, on `RetryFailedJobCommand`) and `Operations.Job.Purge` (line 166, on `DeleteFailedJobCommand`), while `permissions.md` documents the same operations as `Operations.FailedJob.Retry` (line 39) and `Operations.FailedJob.Purge` (line 41) in the `### FailedJob` section. A reader cannot determine which namespace the engine actually enforces.

**Expected:**

All references to the failed-job retry and purge capabilities use a single namespace (either `Operations.FailedJob.*` or `Operations.Job.*`) consistently across `commands.md` and `permissions.md`.

**Evidence:**

`docs/specs/operations/commands.md:152` `**Capability:** `Operations.Job.Retry` (system)`; `:166` `**Capability:** `Operations.Job.Purge``; `docs/specs/operations/permissions.md:33` `- `Operations.Job.Retry` (system)`; `:39` `- `Operations.FailedJob.Retry` (system)`; `:41` `- `Operations.FailedJob.Purge``.

---

### FINDING 3 (id: `SPEC-4-003`)

- **Source:** `docs/audit_reports/findings/wave6-specs-4.md`
- **Severity:** High
- **Area:** spec
- **Location:** `docs/specs/settings/tables.md:106`

**Description:**

The `## Cross-Domain Tables (Referenced)` table in settings refers to a `rbac_role_prototypes` table owned by the rbac domain. The rbac spec (`docs/specs/rbac/tables.md:7-17`) defines only `assign_permissions`, `permissions`, `permission_sections`, `roles`, `rbac_module_permissions`, `rbac_module_permission_assigns`, `rbac_role_permissions`, `two_factor_settings`. There is no `rbac_role_prototypes` table anywhere in the workspace, and `rbac/value-objects.md:12-25` does not list such an identifier.

**Expected:**

Settings spec references `rbac.roles` (the actual rbac spec table for `Role`) for the `settings_dashboard_settings.role_id` and `settings_general_settings` references, not the non-existent `rbac_role_prototypes`.

**Evidence:**

`docs/specs/settings/tables.md:106` `| `rbac_role_prototypes`                | rbac          | Referenced by `role_id`                |`. rbac tables are listed at `docs/specs/rbac/tables.md:7-17` and include `| `roles`                          | Role                     | A role within a school                         |` (line 12) but no `rbac_role_prototypes`.

---

### FINDING 4 (id: `SPEC-4-004`)

- **Source:** `docs/audit_reports/findings/wave6-specs-4.md`
- **Severity:** High
- **Area:** spec
- **Location:** `docs/specs/operations/tables.md:60,84` vs `docs/specs/rbac/tables.md:11`

**Description:**

Operations spec refers to a `rbac_permissions` table owned by rbac, used as the FK target of `rbac_sidebars.permission_id`. The rbac spec defines only `permissions` (no prefix) as the storage table for `Capability`; there is no `rbac_permissions` table. This is a name mismatch between the operations spec's cross-domain reference and the rbac spec's actual table inventory.

**Expected:**

Operations spec references `rbac.permissions` (the actual rbac storage table) for `rbac_sidebars.permission_id`, matching the rbac spec's table name.

**Evidence:**

`docs/specs/operations/tables.md:60` `- `rbac_sidebars.permission_id` references `rbac_permissions``; `:84` `| `rbac_permissions`              | rbac          | Referenced by `rbac_sidebars`          |`. `docs/specs/rbac/tables.md:11` `| `permissions`                    | Capability (storage row) | Catalog row carrying capability + metadata     |` (no `rbac_` prefix).

---

### FINDING 5 (id: `SPEC-4-005`)

- **Source:** `docs/audit_reports/findings/wave6-specs-4.md`
- **Severity:** High
- **Area:** spec
- **Location:** `docs/specs/rbac/repositories.md:46,53`

**Description:**

`PermissionRepository::get` and `PermissionRepository::delete` declare their `id` parameter as `PermissionSectionId`, but the repository is the storage row for `Capability` (per the heading at line 41) and the spec's identifier for that row is `CapabilityId` (per `docs/specs/rbac/value-objects.md:15` `| `CapabilityId`               | `Id<Capability>`            | A permission row (a capability)    |`). The methods cannot logically take a `PermissionSectionId`; this is a copy/paste from `PermissionSectionRepository` (lines 59-68).

**Expected:**

`PermissionRepository::get` and `PermissionRepository::delete` take `CapabilityId` (or a `PermissionId` newtype if one is introduced), not `PermissionSectionId`. Only `list_for_section` should take `PermissionSectionId`.

**Evidence:**

`docs/specs/rbac/repositories.md:46` `async fn get(&self, id: PermissionSectionId) -> Result<Option<Permission>>;`; `:53` `async fn delete(&self, id: PermissionSectionId) -> Result<()>;`. The repository heading at `:41` is `## PermissionRepository (the storage row for a `Capability`)` — confirming these should take the capability-row id, not the section id.

---

### FINDING 6 (id: `SPEC-4-006`)

- **Source:** `docs/audit_reports/findings/wave6-specs-4.md`
- **Severity:** High
- **Area:** spec
- **Location:** `docs/specs/sync/overview.md:1134-1140`

**Description:**

The "Phase 0 status" section uses a different command/event vocabulary than the spec body. The spec body (`sync/overview.md:501-588`) defines 6 commands (`RequestSyncCommand`, `PauseSyncCommand`, `ResumeSyncCommand`, `ResolveConflictCommand`, `SwitchSchoolCommand`, `ApplyRemoteChangeCommand`) and 7 events (`SyncStarted`, `SyncCompleted`, `SnapshotHydrated`, `ConflictReported`, `ConflictResolved`, `OutboxDrained`, `SubscriptionStateChanged`); the Phase 0 status block reports 4 commands shipped as `SyncStart`, `SyncPause`, `SyncResume`, `SyncRequestDelta` and 5 events as `SyncStarted`, `SyncPaused`, `SyncResumed`, `DeltaAvailable`, `DeltaAcknowledged`. The Phase 0 names do not appear in the spec body; `SyncRequestDelta`, `DeltaAvailable`, `DeltaAcknowledged`, and `SyncAcknowledge` are introduced without definitions.

**Expected:**

Phase 0 status uses the same command and event names as the spec body, or the spec body is updated with the Phase 0 names if those are the canonical ones.

**Evidence:**

`docs/specs/sync/overview.md:1134-1140` `- **Commands shipped (4 of 6):** `SyncStart`, `SyncPause`, `SyncResume`, `SyncRequestDelta`. The `SyncAcknowledge` command is deferred...` `- **Events shipped (5 of 7):** `SyncStarted`, `SyncPaused`, `SyncResumed`, `DeltaAvailable`, `DeltaAcknowledged`. `SyncConflictDetected` and `SyncStopped` are deferred.` Spec body commands at `:501,517,533,547,563,579` are `RequestSyncCommand`, `PauseSyncCommand`, `ResumeSyncCommand`, `ResolveConflictCommand`, `SwitchSchoolCommand`, `ApplyRemoteChangeCommand`. Spec body events at `:355,372,390,409,427,447,461` are `SyncStarted`, `SyncCompleted`, `SnapshotHydrated`, `ConflictReported`, `ConflictResolved`, `OutboxDrained`, `SubscriptionStateChanged`.

---

### FINDING 7 (id: `SPEC-4-007`)

- **Source:** `docs/audit_reports/findings/wave6-specs-4.md`
- **Severity:** High
- **Area:** spec
- **Location:** `docs/specs/sync/overview.md:883,1048` (and `:42,64,849,885,941`)

**Description:**

The sync spec refers to a worker binary called `educore-worker` (line 883, 1048) and a server crate called `educore-sync-server` / `educore-sync-server-http` (lines 42, 64, 849, 885, 941). Neither crate/binary exists in the workspace. The actual crates are `educore-sync` and `educore-sync-inprocess` (per `docs/specs/sync/overview.md:1132-1133` and `Cargo.toml:88-89`); the binary in the spec inventory is `educore-cli` (per AGENTS.md Crate Inventory row 35).

**Expected:**

The sync spec references the actual crate names (`educore-sync`, `educore-sync-inprocess`, `educore-sync-http`, `educore-sync-null`) and the actual binary (`educore-cli`), not the non-existent `educore-worker` and `educore-sync-server*`.

**Evidence:**

`docs/specs/sync/overview.md:883` `The worker binary (`educore-worker`) runs in a different process`; `:1048` `- The **worker binary** (`educore-worker` + `WorkerHttpSync`; `:42` `transport protocol. The wire format is the responsibility of `educore-sync-server` and the worker's HTTP client.`; `:64` `- `educore-sync-server` (port) and the wire implementation`; `:849` `uses to talk to a remote `educore-sync-server`.`; `:885` `uses the **HTTP transport** to talk to `educore-sync-server`.`; `:941` `CommandEnvelope` to `educore-sync-server`.`.

---

### FINDING 10 (id: `SPEC-4-010`)

- **Source:** `docs/audit_reports/findings/wave6-specs-4.md`
- **Severity:** Medium
- **Area:** spec
- **Location:** `docs/specs/settings/tables.md:72-75`

**Description:**

The `settings_general_settings.academic_id` column is documented twice with contradictory descriptions in adjacent bullets: the first bullet says "is a legacy reference to the bootstrap academic year" (lines 72-73) and the second says "is the active academic year (nullable)" (lines 74-75). Both bullets reference the same column; the duplicate contradicts itself.

**Expected:**

A single bullet describing `settings_general_settings.academic_id` (the active academic year, nullable). If a separate legacy bootstrap-academic reference column exists, it is named distinctly and has its own bullet.

**Evidence:**

`docs/specs/settings/tables.md:72-75` `- `settings_general_settings.academic_id` is a legacy reference to` `  the bootstrap academic year.` `- `settings_general_settings.academic_id` is the active academic year` `  (nullable).`.

---

### FINDING 11 (id: `SPEC-4-011`)

- **Source:** `docs/audit_reports/findings/wave6-specs-4.md`
- **Severity:** Medium
- **Area:** spec
- **Location:** `docs/specs/settings/value-objects.md:90-131` vs `docs/specs/settings/tables.md:42-56`

**Description:**

The settings spec documents a large set of `Module Toggle` value objects (`LessonEnabled`, `ChatEnabled`, `FeesCollectionEnabled`, ..., `LmsCheckout`, etc., 35+ entries) in `value-objects.md:90-131`, while `tables.md:42-56` declares that "**These are dropped in the engine migration; the engine's module system is capability-based and the consumer's `platform_packages.modules` JSON column carries the enabled modules.**" The two files contradict each other: the value-objects file treats these toggles as first-class typed wrappers, while the tables file says they do not exist in the engine.

**Expected:**

Either the module-toggle value objects are removed from `value-objects.md` (since the storage rows do not exist), or the tables file is updated to indicate the toggles are retained as typed wrappers over the per-package JSON column.

**Evidence:**

`docs/specs/settings/value-objects.md:90-131` "### Module Toggles" lists 35 `bool` toggles including `| `LessonEnabled`     | `bool`                                                       |` through `| `LmsCheckout`       | `bool`                                                       |`. `docs/specs/settings/tables.md:42-56` lists `settings_general_settings.module_toggles` as 35 column names followed by `**These are dropped in the engine migration; the engine's module system is capability-based and the consumer's `platform_packages.modules` JSON column carries the enabled modules.**`.

---

### FINDING 12 (id: `SPEC-4-012`)

- **Source:** `docs/audit_reports/findings/wave6-specs-4.md`
- **Severity:** Medium
- **Area:** spec
- **Location:** `docs/specs/platform/commands.md` (whole file, line range inspected 1-673)

**Description:**

The `ModuleManager` aggregate's commands are listed in `platform/aggregates.md:545-547` as `RegisterModuleManager`, `UpdateModuleManager`, `RotatePurchaseCode` but none of these commands is documented in `platform/commands.md`. `commands.md` covers Locale (lines 477-506), AddOn (`InstallAddOn`/`UninstallAddOn` at lines 448-475), and Module (`EnableModule`/`DisableModule` at lines 422-446), but has no `## Module Manager` section.

**Expected:**

A `## Module Manager` section in `platform/commands.md` documenting the `RegisterModuleManager`, `UpdateModuleManager`, and `RotatePurchaseCode` commands with their `Command` structs, capabilities, and effects, matching the aggregate-level command list.

**Evidence:**

`docs/specs/platform/aggregates.md:543-554` lists `### Commands` `- `RegisterModuleManager` (engine-internal)`, `- `UpdateModuleManager``, `- `RotatePurchaseCode`` under the `## ModuleManager` heading. `docs/specs/platform/commands.md:1-673` contains no `RegisterModuleManager` (verified via search) and no `UpdateModuleManager`. The only `RotatePurchaseCode` reference in the platform spec is in `permissions.md:121`.

---

### FINDING 13 (id: `SPEC-4-013`)

- **Source:** `docs/audit_reports/findings/wave6-specs-4.md`
- **Severity:** Medium
- **Area:** spec
- **Location:** `docs/specs/platform/commands.md` (whole file) vs `docs/specs/platform/aggregates.md:511`

**Description:**

The `AddOn` aggregate lists `RegisterAddOn` as a command in `aggregates.md:511` (`- `RegisterAddOn` (engine-internal, build-time)`), but `commands.md` documents only `InstallAddOn` (line 448) and `UninstallAddOn` (line 464) for the AddOn aggregate. The `RegisterAddOn` command has no struct, no capability, and no effects documented anywhere in `platform/commands.md`.

**Expected:**

A `RegisterAddOnCommand` struct documented in `platform/commands.md` (alongside `InstallAddOn`/`UninstallAddOn`), with capability (`Platform.AddOn.Register` is already in `permissions.md:112`) and effects (`AddOnRegistered` is in `events.md:300-307`).

**Evidence:**

`docs/specs/platform/aggregates.md:511` `- `RegisterAddOn` (engine-internal, build-time)`. `docs/specs/platform/commands.md` line 448 begins `### InstallAddOn` and line 464 begins `### UninstallAddOn`; no `RegisterAddOn` heading exists. `docs/specs/platform/events.md:300-307` declares `### AddOnRegistered` with payload struct, indicating the event exists but the producing command is not documented.

---

### FINDING 14 (id: `SPEC-4-014`)

- **Source:** `docs/audit_reports/findings/wave6-specs-4.md`
- **Severity:** Medium
- **Area:** spec
- **Location:** `docs/specs/platform/entities.md:311-319` vs `docs/specs/platform/aggregates.md:523-554`

**Description:**

`ModuleManager` is documented twice: once as an aggregate root in `aggregates.md:523-554` (heading `## ModuleManager` with identity, invariants, commands, events) and once as an entity in `entities.md:311-319` (heading `## ModuleManager` with identity `ModuleManagerId(Uuid)` and `Owner: ModuleManager` — a self-reference that is nonsensical for an entity). Both entries describe the same root type and reference the legacy `InfixModuleManager` brand artifact.

**Expected:**

`ModuleManager` exists only in `aggregates.md`; the duplicate entity entry in `entities.md:311-319` is removed.

**Evidence:**

`docs/specs/platform/aggregates.md:523-554` `## ModuleManager` ... `**Root type:** `ModuleManager`` ... `### Commands` `RegisterModuleManager` ... `### Events` `ModuleManagerRegistered`. `docs/specs/platform/entities.md:311-319` `## ModuleManager` ... `**Identity:** `ModuleManagerId(Uuid)` (global)` ... `**Owner:** `ModuleManager`` (self-reference). Both reference the same brand artifact (aggregates.md:554 mentions `ModuleManager` aggregate; entities.md:318 says `aggregate replaces the legacy `InfixModuleManager``).

---

### FINDING 15 (id: `SPEC-4-015`)

- **Source:** `docs/audit_reports/findings/wave6-specs-4.md`
- **Severity:** Medium
- **Area:** spec
- **Location:** `docs/specs/operations/aggregates.md:51-87,90-123` (Job / FailedJob) vs `docs/specs/operations/events.md:84-138`

**Description:**

`operations/events.md` documents a `SystemVersionBumped` event (lines 208-227) that is described as a "derived event emitted by the operations domain when both a `SystemVersionRegistered` and a `VersionHistoryRecorded` have been observed for the same version." The event is not listed in either the `SystemVersion` aggregate's events (operations/aggregates.md:149-153 lists only `SystemVersionRegistered`, `SystemVersionUpdated`) or the `VersionHistory` aggregate's events (operations/aggregates.md:180-183 lists only `VersionHistoryRecorded`). The event has no owning aggregate.

**Expected:**

`SystemVersionBumped` is attributed to one of the existing aggregates (most naturally `SystemVersion`, since it bumps the version), or it is added as a new aggregate root (`SystemVersionBump`) with its own commands and consistency boundary.

**Evidence:**

`docs/specs/operations/events.md:208-227` `### SystemVersionBumped` ... `This is a derived event emitted by the operations domain when both a `SystemVersionRegistered` and a `VersionHistoryRecorded` have been observed for the same version.`. `docs/specs/operations/aggregates.md:149-153` lists only `- `SystemVersionRegistered``, `- `SystemVersionUpdated`` for `SystemVersion`; `:180-183` lists only `- `VersionHistoryRecorded`` for `VersionHistory`.

---

### FINDING 16 (id: `SPEC-4-016`)

- **Source:** `docs/audit_reports/findings/wave6-specs-4.md`
- **Severity:** Medium
- **Area:** spec
- **Location:** `docs/specs/operations/repositories.md:179`

**Description:**

`repositories.md:179` defines an index `ix_backups_school_id_academic_id ON backups (school_id, academic_id);`, but the `Backup` aggregate's invariants (operations/aggregates.md:17-26) do not include an `academic_id` field, and the `Backup` value objects (operations/value-objects.md:29-35) also do not include an `academic_id` value type. The index references a column that is not documented anywhere in the spec for the `Backup` aggregate.

**Expected:**

Either the index is removed (if `backups` has no `academic_id` column), or the `Backup` aggregate adds an `academic_id` invariant and the corresponding value object is documented.

**Evidence:**

`docs/specs/operations/repositories.md:179` `CREATE INDEX ix_backups_school_id_academic_id ON backups (school_id, academic_id);`. `docs/specs/operations/aggregates.md:17-26` lists 6 invariants for `Backup` (file_name, file_type, source_link, active_status, restore-in-progress, deletion); none reference `academic_id`. `docs/specs/operations/value-objects.md:29-35` lists `BackupFileName`, `BackupSourceLink`, `BackupFileType`, `BackupLangType`, `BackupActiveStatus`; no `AcademicYearId` or `BackupAcademicId` value object.

---

### FINDING 17 (id: `SPEC-4-017`)

- **Source:** `docs/audit_reports/findings/wave6-specs-4.md`
- **Severity:** Medium
- **Area:** spec
- **Location:** `docs/specs/platform/entities.md:299-310` (ModuleInfo) vs `docs/specs/platform/aggregates.md:88-126` (aggregate table)

**Description:**

`ModuleInfo` is declared as an entity in `entities.md:299-310` (with `Owner: Module (logical; used by RBAC to map module ids to their display info)`), but it is also listed as an aggregate root in `overview.md:104` and as a table owner in `tables.md:15` (`| `platform_module_infos`       | ModuleInfo                | Module display info projection     |`). The `aggregates.md` aggregate-by-aggregate definitions do not include a `## ModuleInfo` section (counting 37 aggregates in overview vs. 37 root-type headings in aggregates.md, with no `ModuleInfo` heading).

**Expected:**

`ModuleInfo` is consistently classified: either a full aggregate root (with its own invariants, commands, and events in `aggregates.md`) or a pure entity (removed from `overview.md`'s aggregate list and `tables.md`'s aggregate column).

**Evidence:**

`docs/specs/platform/overview.md:104` `| ModuleInfo                 | `ModuleInfo`              | A module display info projection                |` (in Aggregate Roots table). `docs/specs/platform/tables.md:15` `| `platform_module_infos`       | ModuleInfo                | Module display info projection     |`. `docs/specs/platform/entities.md:299-310` declares `## ModuleInfo` with `**Owner:** `Module` (logical; used by RBAC to map module ids to their display info)`. `docs/specs/platform/aggregates.md` (heading list) has 37 `## Aggregate` headings but no `## ModuleInfo` heading.

---

### FINDING 18 (id: `SPEC-4-018`)

- **Source:** `docs/audit_reports/findings/wave6-specs-4.md`
- **Severity:** Medium
- **Area:** spec
- **Location:** `docs/specs/sync/overview.md:903-925` (Permissions section)

**Description:**

The sync spec declares six capabilities `Sync.Request`, `Sync.Pause`, `Sync.Resume`, `Sync.ResolveConflict`, `Sync.SwitchSchool`, `Sync.CompactOutbox` in its `## Permissions` section (lines 914-919) and says "Sync capabilities are defined in `educore-rbac`" (line 58 and `:909`). The rbac spec (`docs/specs/rbac/permissions.md`) has no `## Sync` capability section; no `Sync.*` capability is listed anywhere in rbac/spec or rbac/commands. The sync spec's claim that these are defined in the rbac domain is not corroborated by any other file.

**Expected:**

Either the sync spec adds the `Sync.*` capabilities to `docs/specs/rbac/permissions.md` under a new `### Rbac.Sync` (or similar) section, or the sync spec is amended to declare that the sync capabilities are owned by the sync subsystem rather than the rbac domain.

**Evidence:**

`docs/specs/sync/overview.md:914-919` `| `Sync.Request`           | Any authenticated user with school access | Start a sync session for a school   | ... | `Sync.CompactOutbox`     | Server-side operator role only    | Manually trigger outbox compaction         |`. `docs/specs/rbac/permissions.md` (whole file, 165 lines) contains no `Sync.` capability (no `### Sync` heading or `Sync.` line). The Capability enum fragment at `docs/specs/rbac/value-objects.md:50-86` lists only domain capabilities (Rbac, Platform, Settings, Operations, Student, Finance, etc.) and no `Sync*` variants.

---

### FINDING 8 (id: `SPEC-4-008`)

- **Source:** `docs/audit_reports/findings/wave6-specs-4.md`
- **Severity:** Medium
- **Area:** spec
- **Location:** `docs/specs/operations/tables.md:19` vs `docs/specs/operations/aggregates.md:256-292`

**Description:**

The `Sidebar` aggregate is declared to belong to the operations domain (`aggregates.md` heading at line 257 and `overview.md:86`), but the storage table is named `rbac_sidebars` (line 19 of `tables.md`). The rbac spec does not list `rbac_sidebars` as a table (rbac/tables.md:7-17 has `rbac_role_permissions` instead, a different aggregate). The table-naming convention from `docs/code-standards.md` and other spec folders would put a per-school operations-owned table under an `operations_` prefix (as `operations_backups`, `operations_maintenance_settings` are, on `operations/tables.md:11,20`).

**Expected:**

Either the `Sidebar` storage table is renamed to `operations_sidebars` to match its owning domain, or the aggregate ownership is moved to the rbac domain and the spec explicitly disambiguates it from `RolePermission`.

**Evidence:**

`docs/specs/operations/tables.md:19` `| `rbac_sidebars`                | Sidebar            | Per-role sidebar layout projection     |`. `docs/specs/operations/aggregates.md:256-292` declares `## Sidebar` as an operations-owned aggregate (heading at `:257`; root type `Sidebar`; tenant `SchoolId`). `docs/specs/operations/tables.md:11` `| `operations_maintenance_settings`         | MaintenanceSetting | Per-school maintenance mode config     |` and `:20` `| `operations_backups`           | Backup             | Backup records                         |` show the per-domain prefix convention.

---

### FINDING 9 (id: `SPEC-4-009`)

- **Source:** `docs/audit_reports/findings/wave6-specs-4.md`
- **Severity:** Medium
- **Area:** spec
- **Location:** `docs/specs/operations/tables.md:7-23` (column alignment, lines 11/16/20/23)

**Description:**

The `operations/tables.md` table is mis-formatted: the second column (`Aggregate`) has inconsistent column widths, and several rows have leading double-spaces in the second column from a copy-paste artifact (rows at lines 11, 16, 23). The malformed rows render as visibly broken Markdown in tools that respect whitespace alignment.

**Expected:**

Markdown table is uniformly aligned with consistent single-space column separators.

**Evidence:**

`docs/specs/operations/tables.md:11` `| `operations_maintenance_settings`         | MaintenanceSetting | Per-school maintenance mode config     |` (excess leading whitespace before `MaintenanceSetting`); `:16` `| `oauth_personal_access_clients` | (infrastructure) | OAuth PAT clients                |` (column separator misplaced); `:23` `| `operations_version_histories`            | VersionHistory     | Version bump records (global)          |`.

---

### FINDING 19 (id: `SPEC-4-019`)

- **Source:** `docs/audit_reports/findings/wave6-specs-4.md`
- **Severity:** Low
- **Area:** spec
- **Location:** `docs/specs/sync/overview.md:67-105` (Domain Invariants 9) vs `docs/specs/operations/tables.md` / `docs/specs/settings/tables.md`

**Description:**

Sync invariant #9 (`docs/specs/sync/overview.md:99-101`) says "Every sync command carries an `IdempotencyKey`. Resubmitting the same key within the dedupe window returns the prior result, not a duplicate execution." The `IdempotencyKey` type is not declared in the sync spec's own value-object section (`:668-764` lists `CommandEnvelope`, `EventFilter`, `SchoolSnapshot`, `SnapshotRow`, `VersionCursor`, `ConflictId`, `ConflictResolution`). The other domain specs (operations, settings, rbac, platform) also do not declare `IdempotencyKey`; its home is implied to be `educore-core` but no spec file documents it.

**Expected:**

`IdempotencyKey` is declared in a value-objects or types section (in `docs/specs/sync/overview.md`'s `## Value Objects` block, or in a shared `docs/specs/core/` types spec).

**Evidence:**

`docs/specs/sync/overview.md:99-101` `9. **Idempotency on commands.** Every sync command carries an` `IdempotencyKey`. Resubmitting the same key within the dedupe` `window returns the prior result, not a duplicate execution.`. The sync spec's value-object list at `:668-764` includes `CommandEnvelope` (which carries `idempotency_key: IdempotencyKey` per `:682`) but does not declare `IdempotencyKey` itself.

---

### FINDING 20 (id: `SPEC-4-020`)

- **Source:** `docs/audit_reports/findings/wave6-specs-4.md`
- **Severity:** Low
- **Area:** spec
- **Location:** `docs/specs/settings/overview.md:53-74` (Domain Invariants)

**Description:**

The settings spec's invariants 9 and 12 both reference `BaseGroup`/`role_id` uniqueness, but the spec uses different identifier naming for the same role-binding target. Invariant 9 says "A `BaseGroup::name` is unique within `(school_id, name)`" and invariant 12 says "The pair `(dashboard_sec_id, role_id)` is unique within `(school_id)`." However the spec defines `DashboardSetting` to bind to a role (aggregates.md:309-336), and the value-object table at value-objects.md:21 declares `DashboardSettingId` as the row id — yet `commands.md:274` uses `DashboardSectionId` as the type for `dashboard_sec_id` (a foreign key to the section catalog). The `DashboardSectionId` type is referenced in commands and repositories but never declared as a typed value object or identifier in value-objects.md.

**Expected:**

Either `DashboardSectionId` is added to `value-objects.md`'s identifier table (alongside `DashboardSettingId`) with a backing type and notes, or the spec clarifies that `dashboard_sec_id` is a plain `i32` per `tables.md:85` and removes the `DashboardSectionId` type from commands/repositories.

**Evidence:**

`docs/specs/settings/commands.md:274` `pub dashboard_sec_id: DashboardSectionId,` (in `CreateDashboardSettingCommand`). `docs/specs/settings/repositories.md:112` `async fn role_count(&self, dashboard_sec_id: DashboardSectionId) -> Result<u64>;`. `docs/specs/settings/value-objects.md:10-27` lists identifiers but `DashboardSectionId` is absent. `docs/specs/settings/tables.md:85` `- `settings_dashboard_settings.dashboard_sec_id` is an `i32` referencing` (treating it as a plain integer).

---

### FINDING 21 (id: `SPEC-4-021`)

- **Source:** `docs/audit_reports/findings/wave6-specs-4.md`
- **Severity:** Low
- **Area:** spec
- **Location:** `docs/specs/rbac/commands.md:33-35`

**Description:**

`commands.md:32-35` for `CreateRole` says "The legacy `InfixRole` shadow aggregate is removed — `is_replicated` is a flag on the engine's `Role`." This is the only file in the rbac spec that references `InfixRole`. AGENTS.md § "Engine Rules" forbids legacy brand references in "new code, comments, commit messages, or documentation." The reference documents a legacy artefact that is otherwise absent from the engine spec.

**Expected:**

The `InfixRole` mention is removed from `commands.md` (the engine's `Role` aggregate is the only record; the legacy `InfixRole` does not need to be acknowledged in spec text).

**Evidence:**

`docs/specs/rbac/commands.md:33-35` `**Effects:** Creates a `Role` and emits `RoleCreated`. The legacy` ``InfixRole`` `shadow aggregate is removed — `is_replicated` is a flag on the engine's `Role`.`. AGENTS.md § Engine Rules "No legacy names are permitted in new code, comments, commit messages, or documentation." Same reference also appears in `docs/specs/rbac/tables.md:38-40`.

---

### FINDING 22 (id: `SPEC-4-022`)

- **Source:** `docs/audit_reports/findings/wave6-specs-4.md`
- **Severity:** Low
- **Area:** spec
- **Location:** `docs/specs/operations/tables.md:16` (column alignment)

**Description:**

The `oauth_personal_access_clients` row at `operations/tables.md:16` is misaligned — the second column reads `(infrastructure)` with leading whitespace, but the table separator positions are inconsistent across rows.

**Expected:**

Markdown table row aligned with the rest of the table (single space after `|`).

**Evidence:**

`docs/specs/operations/tables.md:16` `| `oauth_personal_access_clients` | (infrastructure) | OAuth PAT clients                |` (column alignment inconsistent with lines 13, 14, 15, 17, 18).

---

### FINDING 23 (id: `SPEC-4-023`)

- **Source:** `docs/audit_reports/findings/wave6-specs-4.md`
- **Severity:** Low
- **Area:** spec
- **Location:** `docs/specs/settings/value-objects.md:86-88`

**Description:**

`value-objects.md` declares two redundant identifiers `AcademicId` (typed as `AcademicYearId?`) and `UnAcademicId` (typed as `AcademicYearId`, default `1`). Both are scoped to the active academic year on the school; the second appears to be a "previous/legacy" identifier but is not explained anywhere else in the settings spec. `aggregates.md:22-49` defines `GeneralSettings` invariants that mention `session_id` and `language_id` and `date_format_id` but neither `academic_id` nor `un_academic_id`.

**Expected:**

Either `AcademicId` is the canonical identifier (and `UnAcademicId` is removed or retyped as `PreviousAcademicYearId` with documentation), or both identifiers are described in the aggregate's invariants section.

**Evidence:**

`docs/specs/settings/value-objects.md:86-88` `| `AcademicId`         | `AcademicYearId?`                                                 |` `| `UnAcademicId`       | `AcademicYearId` (default 1)                                      |`. `docs/specs/settings/aggregates.md:22-49` lists invariants for `GeneralSettings` (none of which mentions `AcademicId` or `UnAcademicId`).

---

### FINDING 24 (id: `SPEC-4-024`)

- **Source:** `docs/audit_reports/findings/wave6-specs-4.md`
- **Severity:** Low
- **Area:** spec
- **Location:** `docs/specs/platform/value-objects.md:191-192`

**Description:**

`CurrencyPosition` is documented with the comment "`1` (prefix with space), `2` (suffix with space)". The description enumerates only two of what is typically four positions (also "prefix" without space and "suffix" without space). The platform spec otherwise documents `CurrencyPosition` only as a single concept (`CurrencyPosition` referenced from `platform/aggregates.md:683` as "`currency_type` and `currency_position` are encoded values whose meanings are documented in the value objects"). The enum is incomplete.

**Expected:**

`CurrencyPosition` documents all positions: `1` (prefix with space), `2` (suffix with space), `3` (prefix no space), `4` (suffix no space) — or whatever the engine's canonical set is.

**Evidence:**

`docs/specs/platform/value-objects.md:191-192` `| `CurrencyPosition`| `1` (prefix with space), `2` (suffix with space)                  |`. The platform spec's only other reference is `platform/aggregates.md:683` `4. `currency_type` and `currency_position` are encoded values` `whose meanings are documented in the value objects.`.

---

### FINDING 25 (id: `SPEC-4-025`)

- **Source:** `docs/audit_reports/findings/wave6-specs-4.md`
- **Severity:** Low
- **Area:** spec
- **Location:** `docs/specs/operations/workflows.md:6-19` (Backup Lifecycle Workflow)

**Description:**

The workflow text "SchoolAdmin (or scheduled job) issues CreateBackupCommand." (line 9) refers to a "scheduled job" producing backups, but the operations spec does not document a `ScheduleBackupCommand` or `BackupSchedule` aggregate. `operations/aggregates.md:15-42` lists `Backup` with commands `CreateBackup`, `DeleteBackup`, `RestoreBackup`, `MarkBackupActive`, `MarkBackupInactive` — no scheduled-create. `operations/entities.md:15-24` documents `BackupSchedule` as an entity "owned by `Backup` (logical)" with the note "The engine does not own a job runner; the schedule is a port-driven configuration that an external adapter reads and acts on." The workflow treats the scheduled job as a first-class actor without the matching command definition.

**Expected:**

Either the workflow removes the "scheduled job" reference (since the engine has no scheduler), or a `ScheduleBackupCommand` is added to `commands.md` and listed under `Backup` in `aggregates.md`.

**Evidence:**

`docs/specs/operations/workflows.md:9` `1. SchoolAdmin (or scheduled job) issues CreateBackupCommand.`. `docs/specs/operations/commands.md:14-72` documents `CreateBackup`, `DeleteBackup`, `RestoreBackup`, `MarkBackupActive`/`MarkBackupInactive`; no scheduled-create command.

---

