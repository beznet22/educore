## Wave 1 Assessment Domain Audit Report
**Scope**: `crates/domains/assessment/`, `docs/specs/assessment/`, `docs/commands/assessment.md`, `docs/events/assessment.md`, `docs/handoff/PHASE-4-HANDOFF.md`, `AGENTS.md` (the assessment row).

**Total findings:** 100

---

### FINDING 1

- **id:** DOMAIN-ASS-001
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/assessment/src/aggregate.rs` (entire file) and `docs/specs/assessment/aggregates.md:1-1025`
- **description:** The aggregate file ships 6 root structs (`Exam`, `ExamSchedule`, `SeatPlan`, `AdmitCard`, `MarksRegister`, `ResultStore`) but is missing 33 of the 39 aggregate roots declared in the spec. The spec at `docs/specs/assessment/aggregates.md` defines 39 aggregates; the code at `crates/domains/assessment/src/aggregate.rs` contains 6. The Phase 4 build-plan exit criterion (line 653) requires every aggregate in `docs/specs/assessment/aggregates.md` to have a Rust struct + tests.
- **expected:** Rust struct + tests for `ExamType`, `ExamSetup`, `MarksGrade`, `MarkStore`, `MarkStoreEntry`, `ResultSetting`, `TemporaryMeritList`, `MeritPosition`, `ExamWisePosition`, `AllExamWisePosition`, `CustomResultSetting`, `CustomTemporaryResult`, `ExamStepSkip`, `ExamRoutinePage`, `FrontendExamRoutine`, `FrontendResult`, `FrontendExamResult`, `OnlineExam`, `QuestionBank`, `QuestionGroup`, `QuestionLevel`, `QuestionAssignment`, `OnlineExamQuestion`, `QuestionMuOption`, `OnlineExamMark`, `OnlineExamStudentAnswerMarking`, `StudentTakeOnlineExam`, `SeatPlanSetting`, `AdmitCardSetting`, `TeacherEvaluation`, `TeacherRemark`, `ExamAttendance`, `ExamAttendanceChild`.
- **evidence:** `crates/domains/assessment/src/aggregate.rs` declares 6 `pub struct` (lines 58, 258, 360, 432, 504, 570). `docs/specs/assessment/aggregates.md` lists 39 aggregates (lines 1-1025); the table at `docs/specs/assessment/overview.md:96-140` lists 44 rows.

---

### FINDING 2

- **id:** DOMAIN-ASS-002
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/assessment/src/aggregate.rs` (no `OnlineExam` struct) and `docs/specs/assessment/aggregates.md:471-518`
- **description:** The `OnlineExam` aggregate is declared in the spec as one of the 8 prompt-named Phase 4 aggregates (`docs/build-plan.md:629-631`) but is missing from `aggregate.rs`. The handoff at `docs/handoff/PHASE-4-HANDOFF.md:67-71` admits the full state machine ships only at the Event level. The aggregate (with `Status`, `IsTaken`, `IsClosed`, `IsWaiting`, `IsRunning`, `AutoMark` lifecycle fields) has no struct, no command, no service, no repository, no query.
- **expected:** `pub struct OnlineExam { id: OnlineExamId, status: OnlineExamStatus, is_taken: bool, is_closed: bool, is_waiting: bool, is_running: bool, auto_mark: bool, start_time: ..., end_time: ..., end_date_time: ..., ... }` per `docs/specs/assessment/aggregates.md:471-518`.
- **evidence:** `docs/build-plan.md:629-631` lists `OnlineExam` as a Phase 4 aggregate; `crates/domains/assessment/src/aggregate.rs` has no `pub struct OnlineExam`; `docs/handoff/PHASE-4-HANDOFF.md:67-71` "The full state machine ships at the Event level (8 events) but the integration test only exercises `create_exam` per the user-chosen scope."

---

### FINDING 3

- **id:** DOMAIN-ASS-003
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/assessment/src/commands.rs` (entire file) and `docs/specs/assessment/commands.md:1-817`
- **description:** The commands file ships 21 command structs but is missing 47 of the 62 commands declared in the spec catalog (`docs/commands/assessment.md`) and the per-command spec (`docs/specs/assessment/commands.md`). The 21 shipped are the 3 Exam workstream-A commands, the 9 workstream-B commands, and the 7 workstream-C commands; the 41 missing are: `CreateExamType`, `UpdateExamType`, `DeleteExamType`, `CreateExamSetup`, `UpdateExamSetup`, `DeleteExamSetup`, `CreateOnlineExam`, `PublishOnlineExam`, `StartOnlineExam`, `SubmitOnlineExamAnswer`, `EvaluateOnlineExam`, `CloseOnlineExam`, `DeleteOnlineExam`, `SetExamSignature`, `ConfigureCustomResultSettings`, `MarkTeacherEvaluation`, `ApproveTeacherEvaluation`, `RejectTeacherEvaluation`, `AddTeacherRemark`, `UpdateTeacherRemark`, `DeleteTeacherRemark`, `CreateMarksGrade`, `UpdateMarksGrade`, `DeleteMarksGrade`, `MarkExamAttendance`, `UpdateExamAttendance`, `CreateExamSetting`, `UpdateExamSetting`, `DeleteExamSetting`, `ConfigureAdmitCardSettings`, `ConfigureSeatPlanSettings`, `ConfigureTeacherEvaluation`, `PublishExamRoutine`, `PublishFrontResult`, `UpdateExamRoutinePage`, `UpdateFrontendExamResult`, `MarkExamStepSkip`, `RequestAbsenceNotification`, `CreateQuestion`, `UpdateQuestion`, `DeleteQuestion`, `CreateQuestionGroup`, `UpdateQuestionGroup`, `DeleteQuestionGroup`, `CreateQuestionLevel`, `UpdateQuestionLevel`, `DeleteQuestionLevel`, `AddOnlineExamQuestion`, `UpdateOnlineExamQuestion`, `DeleteOnlineExamQuestion`, `AddQuestionOption`, `UpdateQuestionOption`, `DeleteQuestionOption`.
- **expected:** 62 typed command structs (per the catalog table at `docs/commands/assessment.md:12-74`).
- **evidence:** `crates/domains/assessment/src/commands.rs` declares 21 `pub struct` (lines 78, 123, 160, 184, 196, 216, 232, 246, 255, 272, 286, 299, 315, 330, 559, 579, 597, 611, 628, 643, 660). `docs/commands/assessment.md:12-74` lists 62 command rows.

---

### FINDING 4

- **id:** DOMAIN-ASS-004
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/assessment/src/events.rs` (entire file) and `docs/specs/assessment/events.md:1-512`
- **description:** The events file ships 20 event structs but is missing 45 of the 65 events declared in the events catalog (`docs/events/assessment.md:10-90`) and per-event spec (`docs/specs/assessment/events.md`). The 20 shipped are: `ExamCreated`, `ExamUpdated`, `ExamDeleted`, `ExamScheduled`, `ExamScheduleUpdated`, `ExamScheduleCancelled`, `SeatPlanGenerated`, `SeatPlanUpdated`, `SeatPlanCancelled`, `AdmitCardGenerated`, `AdmitCardRegenerated`, `AdmitCardCancelled`, `MarksRegisterCreated`, `MarksEntered`, `MarksSubmitted`, `MarksRegisterCancelled`, `ResultStoreCreated`, `ResultRemarksUpdated`, `ResultPublished`, `ResultRepublished`, `ReportCardGenerated`. The 45 missing include: `ExamTypeCreated/Updated/Deleted`, `ExamSetupCreated/Updated/Deleted`, `ExamSettingCreated/Updated/Deleted`, `ExamSignatureCreated/Updated/Deleted`, `ExamRoutinePageUpdated`, `FrontExamRoutinePublished`, `FrontResultPublished`, `FrontendExamResultUpdated`, `MarksGradeCreated/Updated/Deleted`, `MarkStoreCreated`, `TeacherRemarkUpdated`, `MarkStoreDeleted`, `ResultSettingUpdated`, `CustomResultSettingUpdated`, `OnlineExamCreated/Updated/Published/Started/Answered/Evaluated/Closed/Deleted`, `OnlineExamMarkCreated`, `OnlineExamQuestionAdded/Updated/Deleted`, `QuestionOptionAdded/Updated/Deleted`, `QuestionCreated/Updated/Deleted`, `QuestionGroupCreated/Updated/Deleted`, `QuestionLevelCreated/Updated/Deleted`, `SeatPlanSettingUpdated`, `AdmitCardSettingUpdated`, `TeacherEvaluationCompleted/Approved/Rejected/Configured`, `TeacherRemarkAdded/Updated/Deleted`, `ExamAttendanceMarked/Updated`, `ExamStepSkipSet`, `ExamAbsenceNotificationRequested`.
- **expected:** 65 typed `DomainEvent` implementations.
- **evidence:** `crates/domains/assessment/src/events.rs` declares 21 `pub struct` (lines 45, 148, 213, 272, 352, 400, 453, 509, 555, 604, 656, 705, 889, 937, 992, 1047, 1093, 1141, 1186, 1240, 1292). `docs/events/assessment.md:10-90` lists 65 event rows.

---

### FINDING 5

- **id:** DOMAIN-ASS-005
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/assessment/src/value_objects.rs` (entire file) and `docs/specs/assessment/value-objects.md:11-117`
- **description:** The value-objects file ships 13 typed ids (`ExamId`, `ExamTypeId`, `ExamScheduleId`, `ExamScheduleSubjectId`, `SeatPlanId`, `SeatPlanChildId`, `AdmitCardId`, `StaffId`, `ClassRoomId`, `MarksRegisterId`, `MarksRegisterChildId`, `ResultStoreId`) + 4 value types (`ExamName`, `ExamCode`, `ExamMark`, `FullMark`) + 3 numeric newtypes (`Marks`, `TotalMarks`, `Gpa`) + 1 grade row (`MarksGradeRow`) + 1 grade string (`Grade`) + 2 enums (`ExamTerm`, `ResultStatus`) = 25 value objects, but is missing 26 of the typed ids and 13 of the value types listed in the spec table at `docs/specs/assessment/value-objects.md:11-117`. Missing ids: `ExamSetupId`, `ExamSettingId`, `ExamSignatureId`, `MarkStoreId`, `MarkStoreEntryId`, `ResultSettingId`, `MarksGradeId`, `TemporaryMeritListId`, `MeritPositionId`, `ExamWisePositionId`, `AllExamWisePositionId`, `CustomResultSettingId`, `CustomTemporaryResultId`, `ExamStepSkipId`, `ExamRoutinePageId`, `FrontExamRoutineId`, `FrontResultId`, `FrontendExamResultId`, `OnlineExamId`, `QuestionBankId`, `QuestionGroupId`, `QuestionLevelId`, `QuestionAssignmentId`, `QuestionMuOptionId`, `OnlineExamQuestionId`, `OnlineExamQuestionAssignId`, `OnlineExamMarkId`, `OnlineExamStudentAnswerMarkingId`, `StudentTakeOnlineExamId`, `SeatPlanSettingId`, `AdmitCardSettingId`, `TeacherEvaluationId`, `TeacherRemarkId`, `ExamAttendanceId`, `ExamAttendanceChildId`. Missing enums: `QuestionType`, `OnlineExamStatus`, `AttemptStatus`, `AnswerStatus`, `AttendanceType`. Missing value types: `ExamTitle`, `QuestionTitle`, `QuestionOption`, `Remark`, `Comment`, `SignatureTitle`, `GroupTitle`, `Level`, `RoutinePageTitle`, `AverageMark`, `Percentage`, `ExamPercentage`, `MeritPosition`, `Rating`.
- **expected:** All 47 typed ids + 7 closed enums + 14 named value types from `docs/specs/assessment/value-objects.md:11-117`.
- **evidence:** `crates/domains/assessment/src/value_objects.rs` lists 25 pub items; `docs/specs/assessment/value-objects.md:11-117` lists 47 typed ids, 7 enums, 11 named strings, 11 numeric newtypes, 4 special-purpose wrappers.

---

### FINDING 6

- **id:** DOMAIN-ASS-006
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/assessment/src/services.rs` (entire file) and `docs/specs/assessment/services.md:1-373`
- **description:** The services file ships 21 free functions + 1 service struct (`ResultService` with 9 grading methods). The spec at `docs/specs/assessment/services.md:1-373` defines 8 service structs (`ExamService`, `MarksService`, `ResultService`, `ReportCardService`, `SeatPlanService`, `AdmitCardService`, `OnlineExamService`, `TeacherEvaluationService`, `MarksGradeService`) + 2 policy structs (`ResultEligibility`, `AdmitCardEligibility`) + 2 specification structs (`ActiveExamSchedule`, `PendingOnlineExam`) + 1 cross-domain coordinator (`AssessmentCoordinator`). The code is missing 7 of 8 service structs, both policies, both specifications, and the coordinator.
- **expected:** `pub struct ExamService` (with `plan_for_class`, `validate_no_teacher_overlap`, `validate_no_room_overlap`, `lock_after_publish` per `docs/specs/assessment/services.md:8-30`), `pub struct MarksService` (with `initialize_registers`, `validate_marks`, `is_absent_row`, `submit` per `docs/specs/assessment/services.md:38-58`), `pub struct ReportCardService` (with `build_payload`, `render_html`, `render_pdf` per `docs/specs/assessment/services.md:130-146`), `pub struct SeatPlanService` (with `assign_rooms`, `validate_total`, `validate_no_room_overlap`, `build_seat_plan` per `docs/specs/assessment/services.md:155-178`), `pub struct AdmitCardService` (with `build_card`, `render_html`, `render_pdf` per `docs/specs/assessment/services.md:187-201`), `pub struct OnlineExamService` (with `start`, `accept_answer`, `auto_evaluate`, `manual_mark`, `close` per `docs/specs/assessment/services.md:208-238`), `pub struct TeacherEvaluationService` (with `is_window_open`, `can_submit`, `build_evaluation`, `aggregate` per `docs/specs/assessment/services.md:250-275`), `pub struct MarksGradeService` (with `validate_no_overlap`, `validate_contiguous`, `find_grade` per `docs/specs/assessment/services.md:283-298`).
- **evidence:** `crates/domains/assessment/src/services.rs` declares 1 `pub struct` (line 1176: `ResultService`); the spec at `docs/specs/assessment/services.md` declares 8 service structs. `ResultService::publish` (line 1055) is a free function, not the spec's `ResultService::publish` which is an `impl` method that materializes ResultStore/MeritPosition/ExamWisePosition/AllExamWisePosition/CustomTemporaryResult rows.

---

### FINDING 7

- **id:** DOMAIN-ASS-007
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/assessment/src/repository.rs:42-105` and `docs/specs/assessment/repositories.md:1-524`
- **description:** The repository file ships 6 port traits (`ExamRepository`, `ExamScheduleRepository`, `SeatPlanRepository`, `AdmitCardRepository`, `MarksRegisterRepository`, `ResultRepository`). The spec at `docs/specs/assessment/repositories.md:1-524` declares 13 port traits. Missing: `ExamTypeRepository` (lines 9-19), `MarkStoreRepository` (lines 120-131), `MarksGradeRepository` (lines 184-191), `OnlineExamRepository` (lines 197-240), `QuestionBankRepository` (lines 246-259), `TeacherEvaluationRepository` (lines 329-356), `TeacherRemarkRepository` (lines 361-377), `ExamAttendanceRepository` (lines 383-400), `ResultSettingRepository` (lines 406-418), `ExamSettingRepository` (lines 424-444).
- **expected:** 13 `#[async_trait] pub trait XxxRepository: Send + Sync` per the spec.
- **evidence:** `crates/domains/assessment/src/repository.rs` declares 6 `pub trait` (lines 44, 115, 148, 168, 193, 217). `docs/specs/assessment/repositories.md:1-524` defines 13 repository port traits.

---

### FINDING 8

- **id:** DOMAIN-ASS-008
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/assessment/src/repository.rs:115-143` and `docs/specs/assessment/repositories.md:47-90`
- **description:** The shipped `ExamScheduleRepository` is missing 5 methods declared in the spec. The spec defines `list_for_teacher`, `list_for_room`, `list_in_range`, `insert_subject`, `list_subjects`; the code has only `get`, `find`, `list_for_section`, `insert`, `update`, `delete`.
- **expected:** `async fn list_for_teacher(&self, school, teacher, year)`, `async fn list_for_room(&self, school, room, year)`, `async fn list_in_range(&self, school, from, to)`, `async fn insert_subject(&self, s)`, `async fn list_subjects(&self, schedule_id)`.
- **evidence:** `crates/domains/assessment/src/repository.rs:115-143` has 6 methods. `docs/specs/assessment/repositories.md:49-90` has 11 methods.

---

### FINDING 9

- **id:** DOMAIN-ASS-009
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/assessment/src/repository.rs:193-212` and `docs/specs/assessment/repositories.md:94-116`
- **description:** The shipped `MarksRegisterRepository` is missing 5 methods declared in the spec. The spec defines `list_for_student`, `upsert_child`, `list_children`, `child`; the code has only `get`, `find`, `list_for_exam`, `insert`, `update`.
- **expected:** `async fn list_for_student`, `async fn upsert_child`, `async fn list_children`, `async fn child` methods on the trait.
- **evidence:** `crates/domains/assessment/src/repository.rs:193-212` has 5 methods. `docs/specs/assessment/repositories.md:96-115` has 9 methods.

---

### FINDING 10

- **id:** DOMAIN-ASS-010
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/assessment/src/repository.rs:217-235` and `docs/specs/assessment/repositories.md:136-178`
- **description:** The shipped `ResultRepository` is missing 9 methods declared in the spec. The spec defines `list_for_setup`, `list_for_class_section`, `insert_merit`, `list_merit`, `insert_exam_position`, `list_exam_position`, `insert_all_exam_position`, `list_all_exam_position`, `insert_custom_temporary`, `list_custom_temporary`, `clear_custom_temporary`; the code has only `get`, `list_for_student`, `list_for_exam`, `insert`, `update`.
- **expected:** 14 methods on the `ResultRepository` trait per spec.
- **evidence:** `crates/domains/assessment/src/repository.rs:217-235` has 5 methods. `docs/specs/assessment/repositories.md:138-177` has 16 methods.

---

### FINDING 11

- **id:** DOMAIN-ASS-011
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/commands.rs:78-114` and `docs/specs/assessment/commands.md:62-86`
- **description:** The shipped `CreateExamCommand` uses raw `String` for `name` and `code` and raw `f32` for `exam_mark` and `pass_mark` instead of the typed value objects `ExamName`, `ExamCode`, `ExamMark`, `PassMark` defined in the same crate's `value_objects.rs:195-359`. The spec mandates typed wrappers at construction.
- **expected:** `pub name: ExamName, pub code: ExamCode, pub exam_mark: ExamMark, pub pass_mark: PassMark` per `docs/specs/assessment/commands.md:65-76` and the engine rule "Compile-time safety over strings" in `AGENTS.md`.
- **evidence:** `crates/domains/assessment/src/commands.rs:95-101` `pub name: String, pub code: String, pub exam_mark: f32, pub pass_mark: f32,`. The typed wrappers exist in `crates/domains/assessment/src/value_objects.rs:195, 247, 303` and `crates/domains/assessment/src/value_objects.rs:554` (PassMark re-export). The spec at `docs/specs/assessment/commands.md:65-76` lists `pub exam_mark: ExamMark, pub pass_mark: PassMark`.

---

### FINDING 12

- **id:** DOMAIN-ASS-012
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/commands.rs:78-104` and `docs/specs/assessment/commands.md:62-86`
- **description:** The shipped `CreateExamCommand` is missing the `parent_id: Option<ExamId>` field that the spec mandates for composite exam terms. The spec comment notes "for composite exam terms" (a final term is composed of mid-terms per `docs/specs/assessment/aggregates.md:28-29`).
- **expected:** `pub parent_id: Option<ExamId>` per `docs/specs/assessment/commands.md:74`.
- **evidence:** `docs/specs/assessment/commands.md:74` `pub parent_id: Option<ExamId>, // for composite exam terms`. `crates/domains/assessment/src/commands.rs:78-104` has no `parent_id` field.

---

### FINDING 13

- **id:** DOMAIN-ASS-013
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/commands.rs:196-212` and `docs/specs/assessment/commands.md:120-145`
- **description:** The shipped `ScheduleExamCommand` is missing three fields the spec mandates: `room_id: Option<ClassRoomId>`, `teacher_id: Option<StaffId>`, and `exam_period_id: Option<ClassTimeId>`. The spec also uses typed wrappers `ExamDate`, `StartTime`, `EndTime` for the time fields, but the code uses raw `chrono::NaiveDate` and `chrono::NaiveTime`.
- **expected:** `pub room_id: Option<ClassRoomId>, pub teacher_id: Option<StaffId>, pub exam_period_id: Option<ClassTimeId>` per `docs/specs/assessment/commands.md:130-132`; and typed wrappers for the date/time fields.
- **evidence:** `docs/specs/assessment/commands.md:122-134` lists the full struct including the three missing fields. `crates/domains/assessment/src/commands.rs:196-206` only has `schedule_id`, `exam_id`, `class_id`, `section_id`, `date`, `start_time`, `end_time`, `subjects` — no `room_id`, `teacher_id`, `exam_period_id`.

---

### FINDING 14

- **id:** DOMAIN-ASS-014
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/commands.rs:184-192` and `docs/specs/assessment/commands.md:136-144`
- **description:** The shipped `ScheduleSubjectEntry` uses raw `f32` for `full_mark` and `pass_mark` instead of typed `FullMark` and `PassMark`. The spec mandates the typed wrappers. The `room` field is also typed `Option<String>` in the spec but `Option<ClassRoomId>` in the code (semantic drift).
- **expected:** `pub full_mark: FullMark, pub pass_mark: PassMark, pub room: Option<String>` per `docs/specs/assessment/commands.md:136-144`.
- **evidence:** `docs/specs/assessment/commands.md:136-144` lists `pub full_mark: FullMark, pub pass_mark: PassMark`. `crates/domains/assessment/src/commands.rs:184-192` uses `pub full_mark: f32, pub pass_mark: f32, pub room_id: Option<ClassRoomId>` — wrong types for the marks and wrong field name/type for the room.

---

### FINDING 15

- **id:** DOMAIN-ASS-015
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/commands.rs:579-593` and `docs/specs/assessment/commands.md:210-221`
- **description:** The shipped `EnterMarksCommand` uses raw `Option<f32>` for `marks` instead of `Option<Marks>`. The spec mandates the typed wrapper. The spec also uses plural `comments: Option<String>` (per the per-event spec at `docs/specs/assessment/events.md:140`); the code uses singular `comment: Option<String>`.
- **expected:** `pub marks: Option<Marks>` and `pub comments: Option<String>` per `docs/specs/assessment/commands.md:218-220`.
- **evidence:** `docs/specs/assessment/commands.md:213-221` lists `pub marks: Option<Marks>, pub is_absent: bool, pub comments: Option<String>,`. `crates/domains/assessment/src/commands.rs:579-593` uses `pub marks: Option<f32>, ... pub comment: Option<String>,`.

---

### FINDING 16

- **id:** DOMAIN-ASS-016
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/commands.rs:255-268` and `docs/specs/assessment/commands.md:409-425`
- **description:** The shipped `GenerateSeatPlanCommand` is missing the `exam_type_id: ExamTypeId` field that the spec mandates. The spec uses `exam_id: ExamId` whereas the code uses `exam_id: ExamId` (matches) but adds a `seat_plan_id: SeatPlanId` (caller-supplied id pattern) that is not in the spec.
- **expected:** `pub exam_type_id: ExamTypeId` per `docs/specs/assessment/commands.md:416` (the spec's `GenerateSeatPlanCommand`).
- **evidence:** `docs/specs/assessment/commands.md:411-417` lists `pub exam_id: ExamId, pub class_id: ClassId, pub section_id: SectionId, pub exam_type_id: ExamTypeId,`. `crates/domains/assessment/src/commands.rs:255-268` omits `exam_type_id` and adds `seat_plan_id` not in spec.

---

### FINDING 17

- **id:** DOMAIN-ASS-017
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/events.rs:45-76` and `docs/specs/assessment/events.md:59-69`
- **description:** The shipped `ExamCreated` event is missing the `parent_id: Option<ExamId>` field that the spec mandates for composite exam terms.
- **expected:** `pub parent_id: Option<ExamId>` per `docs/specs/assessment/events.md:69`.
- **evidence:** `docs/specs/assessment/events.md:60-69` `pub struct ExamCreated { pub exam_id, pub exam_type_id, ..., pub parent_id: Option<ExamId> }`. `crates/domains/assessment/src/events.rs:45-76` has no `parent_id` field.

---

### FINDING 18

- **id:** DOMAIN-ASS-018
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/events.rs:889-933` and `docs/specs/assessment/events.md:120-127`
- **description:** The shipped `MarksRegisterCreated` event is missing the `subjects: Vec<SubjectId>` field that the spec mandates (used to know which subject rows were initialised for the student).
- **expected:** `pub subjects: Vec<SubjectId>` per `docs/specs/assessment/events.md:125`.
- **evidence:** `docs/specs/assessment/events.md:121-126` `pub struct MarksRegisterCreated { pub marks_register_id, pub exam_id, pub student_id, pub subjects: Vec<SubjectId> }`. `crates/domains/assessment/src/events.rs:889-896` has no `subjects` field.

---

### FINDING 19

- **id:** DOMAIN-ASS-019
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/events.rs:937-987` and `docs/specs/assessment/events.md:133-140`
- **description:** The shipped `MarksEntered` event is missing the `comments: Option<String>` field that the spec mandates. The code stores the comment in the `EnterMarksCommand` but never carries it into the emitted event.
- **expected:** `pub comments: Option<String>` per `docs/specs/assessment/events.md:139`.
- **evidence:** `docs/specs/assessment/events.md:133-140` lists `pub comments: Option<String>,`. `crates/domains/assessment/src/events.rs:937-946` has no `comments` field; `services.rs:981-998` `enter_marks` does not pass a comment argument.

---

### FINDING 20

- **id:** DOMAIN-ASS-020
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/events.rs:992-1042` and `docs/specs/assessment/events.md:142-150`
- **description:** The shipped `MarksSubmitted` event is missing two fields the spec mandates: `total_students: u32` and `submitted_at: Timestamp`. Downstream subscribers (assessment-self, communication) cannot know how many students the submission covers or when it happened.
- **expected:** `pub total_students: u32` and `pub submitted_at: Timestamp` per `docs/specs/assessment/events.md:148-149`.
- **evidence:** `docs/specs/assessment/events.md:142-150` lists `pub total_students: u32, pub submitted_at: Timestamp,`. `crates/domains/assessment/src/events.rs:992-1001` has no `total_students` and no `submitted_at` fields. `services.rs:1004-1027` `submit_marks` hardcodes `subject_count: 0` (line 1022) and uses `now` only for the implicit `EventId::from_uuid(uuid::Uuid::now_v7())`.

---

### FINDING 21

- **id:** DOMAIN-ASS-021
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/events.rs:1093-1137` and `docs/specs/assessment/events.md:158-168`
- **description:** The shipped `ResultStoreCreated` event is missing 5 fields the spec mandates: `exam_type_id`, `subject_id`, `total_marks`, `gpa`, `grade`. Downstream subscribers cannot tell which subject a result row is for or what mark it carries.
- **expected:** `pub exam_type_id: ExamTypeId, pub subject_id: SubjectId, pub total_marks: TotalMarks, pub gpa: Gpa, pub grade: Grade` per `docs/specs/assessment/events.md:162-167`.
- **evidence:** `docs/specs/assessment/events.md:159-168` lists all six required fields. `crates/domains/assessment/src/events.rs:1093-1100` only has `result_store_id, exam_id, student_id`.

---

### FINDING 22

- **id:** DOMAIN-ASS-022
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/events.rs:1186-1236` and `docs/specs/assessment/events.md:175-182`
- **description:** The shipped `ResultPublished` event uses `student_count: u32` instead of the spec's `student_ids: Vec<StudentId>`. The spec requires a vector of per-student ids (so subscribers can act on each student without a re-query); the code carries a meaningless counter.
- **expected:** `pub student_ids: Vec<StudentId>` per `docs/specs/assessment/events.md:180`.
- **evidence:** `docs/specs/assessment/events.md:175-182` lists `pub student_ids: Vec<StudentId>,`. `crates/domains/assessment/src/events.rs:1186-1195` has `pub student_count: u32`; `services.rs:1055-1076` `publish_result` hardcodes `student_count: 0` (line 1071).

---

### FINDING 23

- **id:** DOMAIN-ASS-023
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/events.rs:604-635` and `docs/specs/assessment/events.md:421-428`
- **description:** The shipped `AdmitCardGenerated` event is missing the `generated_at: Timestamp` field the spec mandates.
- **expected:** `pub generated_at: Timestamp` per `docs/specs/assessment/events.md:427`.
- **evidence:** `docs/specs/assessment/events.md:422-428` lists `pub generated_at: Timestamp,`. `crates/domains/assessment/src/events.rs:604-612` has no `generated_at` field.

---

### FINDING 24

- **id:** DOMAIN-ASS-024
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/events.rs:1292-1338` and `docs/specs/assessment/events.md:203-210`
- **description:** The shipped `ReportCardGenerated` event is missing the `payload: ReportCardPayload` field the spec mandates. The payload is the structured per-student report (per-subject marks, GPA, grade, merit position, attendance summary, teacher remarks) — the entire point of the report card.
- **expected:** `pub payload: ReportCardPayload` per `docs/specs/assessment/events.md:209`. The `ReportCardPayload` type is not defined anywhere in the crate.
- **evidence:** `docs/specs/assessment/events.md:204-210` `pub struct ReportCardGenerated { pub result_store_id, pub student_id, pub exam_id, pub include_remarks, pub payload: ReportCardPayload }`. `crates/domains/assessment/src/events.rs:1292-1300` omits the `payload` field. `services.rs:1129-1149` `generate_report_card` does not construct any payload.

---

### FINDING 25

- **id:** DOMAIN-ASS-025
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/events.rs:272-348` and `docs/specs/assessment/events.md:90-101`
- **description:** The shipped `ExamScheduled` event is missing two fields the spec mandates: `room_id: Option<ClassRoomId>` and `teacher_id: Option<StaffId>`. Subscribers (communication, cms) cannot know which room or teacher to broadcast.
- **expected:** `pub room_id: Option<ClassRoomId>, pub teacher_id: Option<StaffId>` per `docs/specs/assessment/events.md:98-99`.
- **evidence:** `docs/specs/assessment/events.md:90-101` lists the full struct. `crates/domains/assessment/src/events.rs:272-294` has no `room_id` or `teacher_id` fields; `services.rs:317-357` `schedule_exam` hardcodes `None` (lines 337, 338) for the room and teacher.

---

### FINDING 26

- **id:** DOMAIN-ASS-026
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/events.rs:453-506` and `docs/specs/assessment/events.md:404-411`
- **description:** The shipped `SeatPlanGenerated` event uses `exam_id: ExamId` instead of the spec's `exam_type_id: ExamTypeId`, and is missing the `rooms: u32` field. The spec's per-student seat allocation is keyed by exam type; the code's event is keyed by exam instance and lacks the room count.
- **expected:** `pub exam_type_id: ExamTypeId, pub rooms: u32, pub total_students: u32` per `docs/specs/assessment/events.md:406-410`.
- **evidence:** `docs/specs/assessment/events.md:404-411` lists `pub exam_type_id, pub class_id, pub section_id, pub rooms, pub total_students`. `crates/domains/assessment/src/events.rs:453-462` has `pub exam_id` (wrong) and no `rooms` field.

---

### FINDING 27

- **id:** DOMAIN-ASS-027
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/aggregate.rs:568-636` and `docs/specs/assessment/aggregates.md:268-303`
- **description:** The shipped `ResultStore` aggregate uses raw `f32` for `total_marks` and `total_gpa` instead of the typed wrappers `TotalMarks` and `Gpa` defined in `value_objects.rs:386-421`. The spec at `docs/specs/assessment/value-objects.md:95-97` mandates the typed wrappers.
- **expected:** `pub total_marks: TotalMarks, pub total_gpa: Gpa` per `docs/specs/assessment/value-objects.md:95-97`.
- **evidence:** `crates/domains/assessment/src/aggregate.rs:578-579` `pub total_marks: f32, pub total_gpa: f32,`. `value_objects.rs:386-421` defines `TotalMarks(f32)` and `Gpa(f32)` typed newtypes with validation.

---

### FINDING 28

- **id:** DOMAIN-ASS-028
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/assessment/src/services.rs:1200-1201` (production code) and `docs/code-standards.md` engine rules
- **description:** The `ResultService::compute_grade` production method (a non-test public method on the `ResultService` struct) uses `.expect("valid grade")` and `.expect("valid gpa")`. The engine rule in `AGENTS.md` / `docs/code-standards.md` forbids `expect()` in production paths because the input space is a school-defined grade scale and the constructor may legitimately fail.
- **expected:** Propagate the `Result<...>` from `Grade::new` and `Gpa::new` instead of panicking on the constructor.
- **evidence:** `crates/domains/assessment/src/services.rs:1182-1203` `pub fn compute_grade(percent: f32) -> (crate::value_objects::Grade, crate::value_objects::Gpa) { ... let g = crate::value_objects::Grade::new(g_str).expect("valid grade"); let gpa = crate::value_objects::Gpa::new(gpa_val).expect("valid gpa"); ... }`.

---

### FINDING 29

- **id:** DOMAIN-ASS-029
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/assessment/src/services.rs:1011-1027` (production code) and `docs/specs/assessment/commands.md:235-251`
- **description:** The `submit_marks` production function mints `MarksSubmitted` with `Uuid::nil()` placeholders for `exam_id`, `class_id`, and `section_id` (lines 1012-1016). The `marks_register_id` only carries the school anchor and the local UUID; the spec's `MarksSubmitted` event requires the per-exam broadcast (which is how downstream `ResultService::publish` knows which `(exam, class, section)` to compute results for). Using nil UUIDs is a data-integrity bug: storage adapters will write the `MarksSubmitted` event with `exam_id = 00000000-...` and downstream consumers will silently fail to correlate.
- **expected:** Resolve the `(exam, class, section)` from a `MarksRegister` aggregate lookup before minting the event.
- **evidence:** `crates/domains/assessment/src/services.rs:1011-1027` `let _placeholder_exam = ExamId::new(cmd.marks_register_id.school_id(), uuid::Uuid::nil()); let _placeholder_class = educore_academic::ClassId::new(cmd.marks_register_id.school_id(), uuid::Uuid::nil()); let _placeholder_section = educore_academic::SectionId::new(cmd.marks_register_id.school_id(), uuid::Uuid::nil()); Ok(MarksSubmitted::new(cmd.marks_register_id, _placeholder_exam, _placeholder_class, _placeholder_section, 0, event_id, ...))`. Comment at line 1011: `// Phase 4 stub: the per-exam broadcast is empty.`

---

### FINDING 30

- **id:** DOMAIN-ASS-030
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/assessment/src/services.rs:1080-1100` (production code) and `docs/specs/assessment/commands.md:276-290`
- **description:** The `republish_result` production function calls `cast_exam_id_placeholder()` (line 1092) which returns an `ExamId` constructed with `uuid::Uuid::nil()`. The same function also mints nil-UUID `ClassId` and `SectionId` (lines 1093-1094). The `ResultRepublished` event is therefore written to storage with `exam_id = 00000000-...` and downstream subscribers cannot correlate.
- **expected:** Resolve the per-exam metadata from a `ResultStore` aggregate lookup (or accept them as command fields) before minting the event.
- **evidence:** `crates/domains/assessment/src/services.rs:1080-1100` `Ok(ResultRepublished::new(cmd.result_store_id.cast_exam_id_placeholder(), educore_academic::ClassId::new(cmd.result_store_id.school_id(), uuid::Uuid::nil()), educore_academic::SectionId::new(cmd.result_store_id.school_id(), uuid::Uuid::nil()), cmd.reason, ...))`. The helper at `services.rs:1372-1380` is documented as `/// **Phase 4 stub.** Returns an ExamId placeholder. The real resolution lands in Phase 16 (engine facade) which re-resolves via the storage port.`

---

### FINDING 31

- **id:** DOMAIN-ASS-031
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/assessment/src/services.rs:1129-1149` (production code) and `docs/specs/assessment/commands.md:291-306`
- **description:** The `generate_report_card` production function mints `ReportCardGenerated` with `Uuid::nil()` for the `exam_id` field (line 1143). The spec's `ReportCardGenerated` event requires the real exam id (so the report card can be opened / printed / downloaded). Using nil UUIDs is a data-integrity bug.
- **expected:** Resolve the `exam_id` from the `ResultStore` aggregate (or accept it as a command field) before minting the event.
- **evidence:** `crates/domains/assessment/src/services.rs:1140-1148` `Ok(ReportCardGenerated::new(cmd.result_store_id, cmd.student_id, ExamId::new(cmd.result_store_id.school_id(), uuid::Uuid::nil()), cmd.include_remarks, ...))`.

---

### FINDING 32

- **id:** DOMAIN-ASS-032
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/services.rs:275-304` and `docs/specs/assessment/aggregates.md:65-68`
- **description:** The `delete_exam` service does NOT enforce the "An Exam cannot be deleted while MarksRegister rows reference it" invariant the spec mandates. The doc-comment at lines 269-271 explicitly defers the check to the test fixture: "the test fixture ensures this by deleting before any marks are entered." In production, calling `delete_exam` on an exam that has `MarksRegister` children will succeed and orphan the children's foreign-key reference.
- **expected:** A pre-conditions check that consults a `MarksRegisterRepository` (or its substitute) and returns `DomainError::Conflict` if any `MarksRegister` references the exam.
- **evidence:** `crates/domains/assessment/src/services.rs:286-304` `pub fn delete_exam<...>(...) -> Result<ExamDeleted> { let now = clock.now(); let actor = cmd.tenant.actor_id; if exam.active_status.is_retired() { return Err(DomainError::conflict(format!("exam {} is already deleted", exam.id))); } exam.active_status = ActiveStatus::Retired; ... }`. The doc-comment at lines 269-270 `// marks are entered.`.

---

### FINDING 33

- **id:** DOMAIN-ASS-033
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/services.rs:317-357` and `docs/specs/assessment/aggregates.md:130-138`
- **description:** The `schedule_exam` service does NOT enforce any of the schedule invariants the spec mandates: uniqueness by `(exam_id, class_id, section_id)`, `StartTime < EndTime`, no teacher overlap, no room overlap, or that the date is within the academic year. None of these checks are performed; the function just mints the aggregate and event.
- **expected:** Five pre-conditions checks: uniqueness, time-well-formedness, teacher-conflict, room-conflict, academic-year-range.
- **evidence:** `crates/domains/assessment/src/services.rs:317-357` `pub fn schedule_exam<...>(_cmd, clock, ids) -> Result<...> { let now = clock.now(); let event_id = ids.next_event_id(); let schedule_id = _cmd.schedule_id; let aggregate = ExamSchedule::fresh(schedule_id, _cmd.exam_id, _cmd.class_id, _cmd.section_id, _cmd.date, _cmd.start_time, _cmd.end_time, None, None, ...); ... }` — no validation. Spec at `docs/specs/assessment/aggregates.md:130-138` lists 5 invariants.

---

### FINDING 34

- **id:** DOMAIN-ASS-034
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/services.rs:438-477` and `docs/specs/assessment/aggregates.md:700-705`
- **description:** The `generate_seat_plan` service does NOT enforce any of the seat plan invariants the spec mandates: uniqueness by `(exam_type_id, class_id, section_id, academic_id)`, sum of `assign_students` equals section's student count, and no time overlap of `SeatPlanChild` allocations. The function just sums and mints.
- **expected:** Three pre-conditions checks.
- **evidence:** `crates/domains/assessment/src/services.rs:438-477` `pub fn generate_seat_plan<...>(cmd, clock, ids) -> Result<...> { ... let total: u32 = cmd.allocations.iter().map(|a| u64::from(a.assign_students)).sum::<u64>().try_into().unwrap_or(u32::MAX); let aggregate = SeatPlan::fresh(...); ... }` — no validation. Spec at `docs/specs/assessment/aggregates.md:700-705` lists 3 invariants.

---

### FINDING 35

- **id:** DOMAIN-ASS-035
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/services.rs:551-581` and `docs/specs/assessment/commands.md:437-456`
- **description:** The `generate_admit_card` service does NOT enforce any of the admit card pre-conditions the spec mandates: an active `StudentRecord` for the academic year, an `AdmitCardSetting` for the academic year, a `SeatPlan` for the section, and an `ExamSchedule` for at least one subject. The function just mints the aggregate and event.
- **expected:** Four pre-conditions checks before the `AdmitCard::fresh` call.
- **evidence:** `crates/domains/assessment/src/services.rs:551-581` `pub fn generate_admit_card<...>(cmd, clock, ids) -> Result<...> { let now = clock.now(); let event_id = ids.next_event_id(); let aggregate = AdmitCard::fresh(cmd.admit_card_id, cmd.student_record_id, cmd.exam_type_id, cmd.academic_year_id, ...); ... }` — no validation. Spec at `docs/specs/assessment/commands.md:447-455` lists 4 pre-conditions.

---

### FINDING 36

- **id:** DOMAIN-ASS-036
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/services.rs:1031-1049` and `docs/specs/assessment/commands.md:190-208`
- **description:** The `cancel_marks_register` function takes a `SubmitMarksCommand` parameter (not a `CancelMarksRegisterCommand`). The signature is incorrect; the spec defines `CancelMarksRegister` (or its equivalent) as a distinct command and the function is wired to the wrong input type. Additionally, the function hardcodes the reason as the literal string `"cancelled"` (line 1044) instead of accepting a reason from the command.
- **expected:** A new `CancelMarksRegisterCommand` type (the spec at `docs/specs/assessment/commands.md:190-208` documents the inverse — `InitializeMarksRegister` — but the catalog at `docs/commands/assessment.md:23` lists `Marks.Cancel` capability and `MarksRegisterCancelled` is emitted), and the function should accept the command's reason field.
- **evidence:** `crates/domains/assessment/src/services.rs:1031-1035` `pub fn cancel_marks_register<C, G>(cmd: SubmitMarksCommand, clock: &C, _ids: &G) -> Result<MarksRegisterCancelled>` and `crates/domains/assessment/src/services.rs:1044` `"cancelled".to_owned()`. No `CancelMarksRegisterCommand` struct exists in `commands.rs`.

---

### FINDING 37

- **id:** DOMAIN-ASS-037
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/services.rs:176-260` and `docs/specs/assessment/commands.md:88-103`
- **description:** The `update_exam` service uses `_ctx: &TenantContext` and `tenant: _` (the `ctx` parameter is unused). The function relies on the dispatcher for the capability check and does not even sanity-check that `cmd.tenant.school_id == exam.id.school_id()`. A cross-tenant command (tenant A's actor operating on tenant B's exam id) will be silently accepted.
- **expected:** An `Err(DomainError::Forbidden)` if `cmd.tenant.school_id != exam.school_id`; or, at minimum, a tenant-scoped uniqueness check on the new code.
- **evidence:** `crates/domains/assessment/src/services.rs:176-186` `pub fn update_exam<C, G>(_ctx: &TenantContext, exam: &mut Exam, cmd: UpdateExamCommand, clock: &C, _ids: &G) -> Result<ExamUpdated> { ... }`. No `school_id` comparison is made anywhere in the function body (lines 186-259).

---

### FINDING 38

- **id:** DOMAIN-ASS-038
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/services.rs:77-161` and `crates/domains/assessment/src/commands.rs:356-369`
- **description:** The `AssessmentUniquenessChecker` port method `exam_unique_key_exists` returns `bool` rather than `Result<bool>`. This means the port cannot fail with a `DomainError` (e.g. when the storage is unreachable). The spec describes the port as returning `Result<...>` for trait consistency. Additionally, `create_exam` does not assert that `cmd.exam_id.school_id() == cmd.tenant.school_id` (a cross-tenant exam id would be accepted).
- **expected:** `fn exam_unique_key_exists(...) -> Result<bool>` and a `school_id` assertion at the start of `create_exam`.
- **evidence:** `crates/domains/assessment/src/commands.rs:356-369` `pub trait AssessmentUniquenessChecker: Send + Sync { ... fn exam_unique_key_exists(&self, ...) -> bool; }` (returns `bool`, not `Result<bool>`). `crates/domains/assessment/src/services.rs:106-113` calls the port with no `?`.

---

### FINDING 39

- **id:** DOMAIN-ASS-039
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/services.rs:140-159` and `docs/specs/assessment/commands.md:62-86`
- **description:** The `create_exam` service calls `validate_exam_name(&cmd.name)?`, `validate_exam_code(&cmd.code)?`, `validate_exam_mark(cmd.exam_mark)?`, `validate_pass_mark(cmd.pass_mark)?` twice — once for the aggregate construction (lines 91-94) and once again for the event construction (lines 151-154). The duplicated validation is wasteful and the two `?` paths can diverge if the constructors become fallible. The event-side calls should pass the already-validated newtypes (`name`, `code`, `exam_mark`, `pass_mark`) by reference.
- **expected:** Pass the already-validated `name`, `code`, `exam_mark`, `pass_mark` newtypes to `ExamCreated::new` directly.
- **evidence:** `crates/domains/assessment/src/services.rs:91-94` `let name = validate_exam_name(&cmd.name)?; let code = validate_exam_code(&cmd.code)?; let exam_mark = validate_exam_mark(cmd.exam_mark)?; let pass_mark = validate_pass_mark(cmd.pass_mark)?;` and `crates/domains/assessment/src/services.rs:151-154` `validate_exam_name(&cmd.name)?, validate_exam_code(&cmd.code)?, validate_exam_mark(cmd.exam_mark)?, validate_pass_mark(cmd.pass_mark)?,`.

---

### FINDING 40

- **id:** DOMAIN-ASS-040
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/assessment/src/commands.rs:60-68` and `docs/code-standards.md`
- **description:** The nine `ASSESSMENT_*_COMMAND_TYPE` constants on lines 60-68 (`ASSESSMENT_EXAM_SCHEDULE_CREATE/UPDATE/CANCEL_COMMAND_TYPE`, `ASSESSMENT_SEAT_PLAN_GENERATE/UPDATE/CANCEL_COMMAND_TYPE`, `ASSESSMENT_ADMIT_CARD_GENERATE/REGENERATE/CANCEL_COMMAND_TYPE`) have no rustdoc. The first three (`ASSESSMENT_EXAM_CREATE/UPDATE/DELETE_COMMAND_TYPE`) are documented (lines 47-49, 52-54, 56-58). The `deny(missing_docs)` lint at `lib.rs:10` is at the file level, so the constant-level docs are required.
- **expected:** `///` doc comments on every `pub const` per the engine's `deny(missing_docs)` policy.
- **evidence:** `crates/domains/assessment/src/commands.rs:60-68` declares the 9 constants with no `///` lines above them. `crates/domains/assessment/src/lib.rs:10` `#![deny(missing_docs)]`.

---

### FINDING 41

- **id:** DOMAIN-ASS-041
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/assessment/src/entities.rs:113-150` (`MarksRegisterChild`) and `docs/specs/assessment/aggregates.md:202-220`
- **description:** The `MarksRegisterChild` entity stores `comment: Option<String>` (line 140) but the spec's `MarksEntered` event carries `comments: Option<String>` (plural) at `docs/specs/assessment/events.md:139`. There is no path for the comment to be populated from the `EnterMarksCommand` to the `MarksRegisterChild` aggregate (the `enter_marks` service emits `MarksEntered` without a comment field, and the repository does not have `upsert_child`/`list_children` methods, see Finding DOMAIN-ASS-009).
- **expected:** The `MarksEntered` event carries the `comments` field (Finding DOMAIN-ASS-019) and the `enter_marks` service writes through to a `MarksRegisterChild` row.
- **evidence:** `crates/domains/assessment/src/entities.rs:140` `pub comment: Option<String>`. `services.rs:981-998` `enter_marks` does not persist to a `MarksRegisterChild` row.

---

### FINDING 42

- **id:** DOMAIN-ASS-042
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/services.rs:1176-1357` (`ResultService`) and `docs/specs/assessment/services.md:64-119`
- **description:** The shipped `ResultService` is missing the central `publish` method the spec mandates. The spec defines `pub fn publish(&self, exam, section, registers, scale) -> Result<PublishOutcome, ValidationError>` as the heart of result publication (lines 112-118 of the spec). The code's `publish_result` free function does not invoke `ResultService::publish`; it just mints a `ResultPublished` event with `student_count: 0` and no per-student `ResultStore` rows.
- **expected:** `impl ResultService { pub fn publish(...) -> Result<PublishOutcome, ValidationError> { ... } }` that materialises `ResultStore` rows + `MeritPosition` + `ExamWisePosition` + `AllExamWisePosition` + `CustomTemporaryResult` rows.
- **evidence:** `docs/specs/assessment/services.md:110-119` lists `pub fn publish(...) -> Result<PublishOutcome, ValidationError>` as a `ResultService` method. `crates/domains/assessment/src/services.rs:1176-1357` defines the `ResultService` struct + 9 grading methods but no `publish` method. `services.rs:1055-1076` is a free function `publish_result` that does not call `ResultService::publish`.

---

### FINDING 43

- **id:** DOMAIN-ASS-043
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/entities.rs` (entire file) and `docs/specs/assessment/aggregates.md:1-1025`
- **description:** The entities file ships 3 child entity structs (`ExamScheduleSubject`, `SeatPlanChild`, `MarksRegisterChild`). The spec at `docs/specs/assessment/aggregates.md` defines 10 child entities (children only, not roots). Missing children: `MarkStoreEntry`, `CustomTemporaryResult`, `ExamRoutinePage`, `FrontendExamRoutine`, `FrontendResult`, `FrontendExamResult`, `QuestionAssignment`, `OnlineExamQuestion`, `QuestionMuOption`, `OnlineExamStudentAnswerMarking`, `StudentTakeOnlineExamQuestion`, `ExamAttendanceChild`. (Note: `ExamScheduleSubject`, `SeatPlanChild`, `MarksRegisterChild` are the 3 shipped; the remaining 10+ children are absent.)
- **expected:** All 13 child entity structs per the aggregates spec.
- **evidence:** `crates/domains/assessment/src/entities.rs:38, 81, 114` declares 3 `pub struct`. `docs/specs/assessment/aggregates.md:152, 203, 256, 583, 611, 626, 640, 656, 1006` lists 9+ child entity sections; many of these also have additional sub-children.

---

### FINDING 44

- **id:** DOMAIN-ASS-044
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/assessment/src/entities.rs` and `docs/specs/assessment/tables.md:1-72`
- **description:** Zero of the 47 tables listed in `docs/specs/assessment/tables.md` have a corresponding `#[derive(DomainQuery)]` struct in `entities.rs` (or anywhere in the crate). The build-plan § "Runtime DDL emission" requires the storage adapter to walk the macro-emitted typed AST at schema-creation time. Without the macro emissions, the storage adapter cannot emit DDL for the 47 tables.
- **expected:** A `#[derive(DomainQuery)]` struct per table row in `tables.md`, generating the typed AST that the storage adapter translates to dialect-specific DDL.
- **evidence:** `docs/specs/assessment/tables.md:1-72` lists 47 table rows. `grep -n 'derive.*DomainQuery' crates/domains/assessment/src/*.rs` returns 0 matches (verified). `entities.rs:38, 81, 114` only have `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]`.

---

### FINDING 45

- **id:** DOMAIN-ASS-045
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/query.rs:122-127, 161-168, 195-200, 226-231, 330-337, 364-371` and `docs/specs/assessment/aggregates.md:1-1025`
- **description:** All 6 typed query stubs (`ExamQuery::execute`, `ExamScheduleQuery::execute`, `SeatPlanQuery::execute`, `AdmitCardQuery::execute`, `MarksRegisterQuery::execute`, `ResultStoreQuery::execute`) return `Err(DomainError::not_supported(...))` and never execute. None of the queries that the spec requires (the `list_for_year`, `list_for_class`, `list_for_type`, `find` by unique key, `list_for_section`, `list_for_exam`, `list_for_student` repository methods the spec defines) are queryable. The query stubs are also marked `#[allow(dead_code)]` — they exist only to be called later, never to satisfy the spec's query surface.
- **expected:** Functional query executors backed by the typed `#[derive(DomainQuery)]` AST (Finding DOMAIN-ASS-044).
- **evidence:** `crates/domains/assessment/src/query.rs:122-127` `pub async fn execute(&self, _ctx: &TenantContext) -> Result<Vec<crate::aggregate::Exam>> { Err(DomainError::not_supported("ExamQuery::execute is a Phase 4 stub; ... ")) }`. Six similar stubs at lines 161-168, 195-200, 226-231, 330-337, 364-371.

---

### FINDING 46

- **id:** DOMAIN-ASS-046
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/assessment/` (no `tests/` directory) and `AGENTS.md` (9-file layout)
- **description:** The crate has no `tests/` directory for integration tests. AGENTS.md lists the 9-file module layout (`lib.rs`, `aggregate.rs`, `entities.rs`, `value_objects.rs`, `commands.rs`, `events.rs`, `services.rs`, `repository.rs`, `query.rs`, `errors.rs` = 10 files; the 10th is the implied `tests/` for integration). All "integration" tests live in `crates/tools/storage-parity/tests/assessment_integration.rs` (a single file exercising only `create_exam` per the handoff at `docs/handoff/PHASE-4-HANDOFF.md:66-71`). The schedule-mark-publish-report workflow from `docs/specs/assessment/workflows.md:30-65, 81-99, 110-128` has no integration test.
- **expected:** A `tests/` directory with at least one workflow integration test that exercises the full `schedule_exam` → `enter_marks` → `submit_marks` → `publish_result` → `generate_report_card` flow.
- **evidence:** `ls /home/beznet/Workspace/smscore/crates/domains/assessment/tests` returns "No such file or directory". `AGENTS.md` (Module Layout) shows `tests/` as a required entry.

---

### FINDING 47

- **id:** DOMAIN-ASS-047
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/assessment/src/lib.rs:10` and `crates/domains/assessment/src/aggregate.rs:17, value_objects.rs:28, commands.rs:23, events.rs:18, repository.rs:15`
- **description:** The `lib.rs` enforces `#![deny(missing_docs)]` (line 10), but every internal module file (aggregate.rs, value_objects.rs, commands.rs, events.rs, repository.rs) has its own `#![allow(missing_docs)]` override. The crate-level `deny` and the module-level `allow` work in the compiler, but the policy is contradictory: the crate's public surface (`prelude::*` re-exports ~30+ items) is documented through the original module's doc, while the `deny` would seem to require docs on every public item. The `aggregate.rs:17-22` comment ("described by their type names; suppressing this lint for the file is the pragmatic choice for the 8 aggregates Phase 4 ships") concedes the suppression is for convenience.
- **expected:** A consistent doc policy. Either (a) the crate does not have `deny(missing_docs)` and instead relies on per-module decisions, or (b) every public item gets a rustdoc.
- **evidence:** `crates/domains/assessment/src/lib.rs:10` `#![deny(missing_docs)]`. `crates/domains/assessment/src/aggregate.rs:17` `#![allow(missing_docs)]`. `crates/domains/assessment/src/value_objects.rs:28` `#![allow(missing_docs)]`. `crates/domains/assessment/src/commands.rs:23` `#![allow(missing_docs)]`. `crates/domains/assessment/src/events.rs:18` `#![allow(missing_docs)]`. `crates/domains/assessment/src/repository.rs:15` `#![allow(missing_docs)]`.

---

### FINDING 48

- **id:** DOMAIN-ASS-048
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/lib.rs:21-37` and `crates/domains/assessment/src/aggregate.rs` and `crates/domains/assessment/src/services.rs`
- **description:** The lib.rs module-level doc claims "5 assessment aggregate roots shipped in Phase 4 Workstream A" (line 21) and "2 typed DomainEvent implementations" (line 28). The actual shipped counts are 6 aggregate roots (Exam, ExamSchedule, SeatPlan, AdmitCard, MarksRegister, ResultStore) across workstreams A + B + C, and 21 typed `DomainEvent` implementations across the same workstreams. The doc-vs-code drift misleads readers about what is in the crate.
- **expected:** Doc strings that match the actual shipped counts.
- **evidence:** `crates/domains/assessment/src/lib.rs:21` `/// The 5 assessment aggregate roots shipped in Phase 4 Workstream A`. `crates/domains/assessment/src/lib.rs:28` `/// The 2 typed \`DomainEvent\` implementations shipped in Phase 4 Workstream A`. `crates/domains/assessment/src/aggregate.rs` has 6 `pub struct`; `crates/domains/assessment/src/events.rs` has 21 `pub struct` implementing `DomainEvent`.

---

### FINDING 49

- **id:** DOMAIN-ASS-049
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/services.rs:1182-1203` and `docs/specs/assessment/services.md:1-373`
- **description:** The `ResultService::compute_grade` method uses a hardcoded 8-tier A-F scale (`A+ >= 90`, `A >= 80`, `B+ >= 70`, `B >= 60`, `C >= 50`, `D >= 40`, `E >= 33`, `F < 33`) and the function signature takes a `percent: f32` not a `scale: &[MarksGrade]` (as the spec mandates). The function therefore does not consume the `MarksGradeScale` port the spec requires and is not policy-driven per school.
- **expected:** `pub fn compute_grade(percent: Percentage, scale: &[MarksGrade]) -> (Grade, Gpa)` per `docs/specs/assessment/services.md:70-73`.
- **evidence:** `crates/domains/assessment/src/services.rs:1182-1183` `pub fn compute_grade(percent: f32) -> (crate::value_objects::Grade, crate::value_objects::Gpa) { ... }`. `docs/specs/assessment/services.md:70-73` `pub fn compute_grade(percent: Percentage, scale: &[MarksGrade]) -> (Grade, Gpa)`.

---

### FINDING 50

- **id:** DOMAIN-ASS-050
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/services.rs:1207-1217` and `docs/specs/assessment/services.md:75-78`
- **description:** The `ResultService::compute_subject_marks` method takes `(marks: f32, full_mark: f32)` instead of `(child: &MarksRegisterChild, exam: &Exam)` (as the spec mandates). The function therefore cannot read the `is_absent` flag and cannot apply the school's absent rule.
- **expected:** `pub fn compute_subject_marks(child: &MarksRegisterChild, exam: &Exam) -> (Marks, Gpa, Grade)` per `docs/specs/assessment/services.md:75-78`.
- **evidence:** `crates/domains/assessment/src/services.rs:1207-1210` `pub fn compute_subject_marks(marks: f32, full_mark: f32) -> (crate::value_objects::Grade, crate::value_objects::Gpa) { ... }`. `docs/specs/assessment/services.md:75-78` `pub fn compute_subject_marks(child: &MarksRegisterChild, exam: &Exam) -> (Marks, Gpa, Grade)`.

---

### FINDING 51

- **id:** DOMAIN-ASS-051
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/services.rs:1221-1234` and `docs/specs/assessment/services.md:80-84`
- **description:** The `ResultService::compute_total` method takes `(children: &[f32], full_marks: &[f32])` instead of `(children: &[MarksRegisterChild], exam: &Exam, scale: &[MarksGrade])`. The function therefore cannot read per-subject pass marks or apply the per-school grade scale, and produces a `(f32, Grade, Gpa)` tuple instead of the spec's `(TotalMarks, Gpa, Grade, ResultStatus)`.
- **expected:** `pub fn compute_total(children: &[MarksRegisterChild], exam: &Exam, scale: &[MarksGrade]) -> (TotalMarks, Gpa, Grade, ResultStatus)` per `docs/specs/assessment/services.md:80-84`.
- **evidence:** `crates/domains/assessment/src/services.rs:1221-1224` `pub fn compute_total(children: &[f32], full_marks: &[f32]) -> (f32, crate::value_objects::Grade, crate::value_objects::Gpa) { ... }`. `docs/specs/assessment/services.md:80-84` `pub fn compute_total(children: &[MarksRegisterChild], exam: &Exam, scale: &[MarksGrade]) -> (TotalMarks, Gpa, Grade, ResultStatus)`.

---

### FINDING 52

- **id:** DOMAIN-ASS-052
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/services.rs:1240-1253` and `docs/specs/assessment/services.md:86-89`
- **description:** The `ResultService::determine_pass_fail` method takes `(marks: &[f32], pass_marks: &[f32])` instead of `(children: &[MarksRegisterChild], exam: &Exam)`. The function does not consume `MarksRegisterChild` and therefore cannot read the `is_absent` flag.
- **expected:** `pub fn determine_pass_fail(children: &[MarksRegisterChild], exam: &Exam) -> ResultStatus` per `docs/specs/assessment/services.md:86-89`.
- **evidence:** `crates/domains/assessment/src/services.rs:1240-1243` `pub fn determine_pass_fail(marks: &[f32], pass_marks: &[f32]) -> crate::value_objects::ResultStatus { ... }`. `docs/specs/assessment/services.md:86-89` `pub fn determine_pass_fail(children: &[MarksRegisterChild], exam: &Exam) -> ResultStatus`.

---

### FINDING 53

- **id:** DOMAIN-ASS-053
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/services.rs:1260-1278` and `docs/specs/assessment/services.md:91-97`
- **description:** The `ResultService::rank_section` method takes `totals: &[f32]` and returns `Vec<u32>` (just rank positions), but the spec mandates `(results: &[ResultStore]) -> Vec<MeritPosition>`. The function therefore cannot materialise `MeritPosition` rows. The function also mis-implements the spec's "positions skip the next integer on ties" invariant — it skips by `j - i + 1` (the number of tied students) which is correct only for one tie group at the start; ties later in the order produce a skip of the tied count not the tied count + previous count.
- **expected:** `pub fn rank_section(results: &[ResultStore]) -> Vec<MeritPosition>` per `docs/specs/assessment/services.md:91-93`.
- **evidence:** `crates/domains/assessment/src/services.rs:1260-1278` `pub fn rank_section(totals: &[f32]) -> Vec<u32> { ... }`. `docs/specs/assessment/services.md:91-93` `pub fn rank_section(results: &[ResultStore]) -> Vec<MeritPosition>`. The doc comment on line 1258-1259 says "tied ranks get the same position; positions skip integers on ties" but the implementation skips by `j - i + 1` per tie group (line 1274) which produces standard competition ranking only when no tie straddles a non-tied group.

---

### FINDING 54

- **id:** DOMAIN-ASS-054
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/services.rs:1283-1285` and `docs/specs/assessment/services.md:95-97`
- **description:** The `ResultService::rank_all_sections` method is implemented as `Self::rank_section(totals)` (one line wrapper) but the spec mandates `rank_all_sections(results: &[ResultStore]) -> Vec<AllExamWisePosition>`. The function therefore returns the wrong type (`Vec<u32>` instead of `Vec<AllExamWisePosition>`).
- **expected:** `pub fn rank_all_sections(results: &[ResultStore]) -> Vec<AllExamWisePosition>` per `docs/specs/assessment/services.md:95-97`.
- **evidence:** `crates/domains/assessment/src/services.rs:1283-1285` `pub fn rank_all_sections(totals: &[f32]) -> Vec<u32> { Self::rank_section(totals) }`. `docs/specs/assessment/services.md:95-97` `pub fn rank_all_sections(results: &[ResultStore]) -> Vec<AllExamWisePosition>`.

---

### FINDING 55

- **id:** DOMAIN-ASS-055
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/services.rs:1290-1300` and `docs/specs/assessment/services.md:286-288`
- **description:** The `ResultService::validate_no_overlap` method delegates to the `MarksGradeScale` port's `validate()` method (lines 1290-1300) instead of walking the scale's rows itself. The spec defines both `MarksGradeService::validate_no_overlap(scale: &[MarksGrade])` and `ResultService` methods that take a `&[MarksGrade]`. The current implementation conflates the two services and provides no actual overlap detection.
- **expected:** A standalone `pub fn validate_no_overlap(scale: &[MarksGrade]) -> Result<(), ValidationError>` that walks the rows and rejects overlapping percent ranges, per `docs/specs/assessment/services.md:286-288`.
- **evidence:** `crates/domains/assessment/src/services.rs:1290-1300` `pub fn validate_no_overlap(_scale: &dyn crate::commands::MarksGradeScale) -> educore_core::error::Result<()> { if !_scale.validate() { return Err(DomainError::validation("grade scale has overlapping ranges")); } Ok(()) }`. `docs/specs/assessment/services.md:286-288` `pub fn validate_no_overlap(scale: &[MarksGrade]) -> Result<(), ValidationError>`.

---

### FINDING 56

- **id:** DOMAIN-ASS-056
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/services.rs:1326-1356` and `docs/specs/assessment/services.md:99-105`
- **description:** The `ResultService::build_result_store` method takes 12 positional arguments (lines 1326-1340) instead of the spec's signature `pub fn build_result_store(exam: &Exam, setup: &ExamSetup, student: &StudentRecord, children: &[MarksRegisterChild], scale: &[MarksGrade]) -> ResultStore`. The current signature forces the caller to have already-materialised totals/grades (the spec says the service computes them from children + scale).
- **expected:** `pub fn build_result_store(exam: &Exam, setup: &ExamSetup, student: &StudentRecord, children: &[MarksRegisterChild], scale: &[MarksGrade]) -> ResultStore` per `docs/specs/assessment/services.md:99-105`.
- **evidence:** `crates/domains/assessment/src/services.rs:1326-1340` 12-argument `pub fn build_result_store(...) -> crate::aggregate::ResultStore`. `docs/specs/assessment/services.md:99-105` 5-argument spec signature.

---

### FINDING 57

- **id:** DOMAIN-ASS-057
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/services.rs:1-1380` and `docs/specs/assessment/services.md:1-373`
- **description:** The services file is missing 7 of 8 service structs the spec defines. Specifically missing: `ExamService` (services.md:8-30), `MarksService` (services.md:38-58), `ReportCardService` (services.md:130-146), `SeatPlanService` (services.md:155-178), `AdmitCardService` (services.md:187-201), `OnlineExamService` (services.md:208-238), `TeacherEvaluationService` (services.md:250-275), `MarksGradeService` (services.md:283-298). Only `ResultService` ships (and even that is missing its `publish` method, see Finding DOMAIN-ASS-042).
- **expected:** 8 service structs per the spec.
- **evidence:** `crates/domains/assessment/src/services.rs` declares 1 `pub struct` (line 1176). `docs/specs/assessment/services.md` declares 8 `pub struct` headers at lines 9, 39, 67, 130, 155, 187, 208, 250, 283.

---

### FINDING 58

- **id:** DOMAIN-ASS-058
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/services.rs:643-645` and `docs/specs/assessment/services.md:301-345`
- **description:** The services file ships the `school_matches` helper (lines 643-645) but the spec defines 4 policy/specification structs: `ResultEligibility` (services.md:303-310), `AdmitCardEligibility` (services.md:316-321), `ActiveExamSchedule` (services.md:323-332), `PendingOnlineExam` (services.md:335-345). None of these are implemented.
- **expected:** All 4 policy/specification structs.
- **evidence:** `crates/domains/assessment/src/services.rs:643-645` `pub fn school_matches(ctx: &TenantContext, school: SchoolId) -> bool { ctx.school_id == school }`. `docs/specs/assessment/services.md:303-345` defines 4 policy/specification structs.

---

### FINDING 59

- **id:** DOMAIN-ASS-059
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/services.rs:1155-1175` (doc comment) and `docs/specs/assessment/services.md:64-119`
- **description:** The `ResultService` doc-comment claims it ships "a minimal table-driven implementation of the grade-computation rules. The full per-school grade scale, the validate-no-overlap / validate-contiguous invariants, and the merit-position ties (with skipped integers) land in a follow-up phase." This admits the spec is not honored. The spec mandates the full implementation, not a "follow-up" stub.
- **expected:** A `ResultService` that honors the spec's full signature (Findings DOMAIN-ASS-049 to DOMAIN-ASS-056).
- **evidence:** `crates/domains/assessment/src/services.rs:1161-1172` is the doc comment that defers the full implementation.

---

### FINDING 60

- **id:** DOMAIN-ASS-060
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/services.rs:981-998` and `docs/specs/assessment/commands.md:210-233`
- **description:** The `enter_marks` service does NOT consult the `MarksRegisterRepository` to verify that the register is not cancelled, that the subject is part of the register, or that the exam is not yet published. The function just mints the `MarksEntered` event.
- **expected:** Three pre-conditions checks: register-active, subject-in-register, exam-not-yet-published.
- **evidence:** `crates/domains/assessment/src/services.rs:981-998` `pub fn enter_marks<C, G>(cmd: EnterMarksCommand, clock: &C, ids: &G) -> Result<MarksEntered> { let now = clock.now(); let event_id = ids.next_event_id(); Ok(MarksEntered::new(cmd.marks_register_id, cmd.subject_id, cmd.student_id, cmd.marks, cmd.is_absent, event_id, cmd.tenant.correlation_id, now)) }` — no validation. `docs/specs/assessment/commands.md:224-229` lists 3 pre-conditions.

---

### FINDING 61

- **id:** DOMAIN-ASS-061
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/services.rs:945-976` and `docs/specs/assessment/commands.md:190-208`
- **description:** The `initialize_marks_register` service does NOT enforce the spec's 3 pre-conditions: the exam exists, the schedule exists, and the students are enrolled in the section. The function just mints one register per command (one student), not one per student in the section as the spec mandates.
- **expected:** Three pre-conditions checks + a per-student register creation loop.
- **evidence:** `crates/domains/assessment/src/services.rs:945-976` `pub fn initialize_marks_register<...>(cmd, clock, ids) -> Result<...> { ... let aggregate = crate::aggregate::MarksRegister::fresh(cmd.marks_register_id, cmd.exam_id, cmd.student_id, cmd.class_id, cmd.section_id, cmd.academic_year_id, ...); ... }` — single register per command. `docs/specs/assessment/commands.md:203-208` mandates "Creates one MarksRegister per student in the section".

---

### FINDING 62

- **id:** DOMAIN-ASS-062
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/services.rs:1055-1076` and `docs/specs/assessment/commands.md:253-274`
- **description:** The `publish_result` service does NOT enforce the spec's 3 pre-conditions: all marks have been submitted, the school has a non-empty MarksGrade scale, and the exam is not yet published (or is being republished). The function just mints a `ResultPublished` event with `student_count: 0`.
- **expected:** Three pre-conditions checks + a per-student `ResultStore` materialisation.
- **evidence:** `crates/domains/assessment/src/services.rs:1055-1076` `pub fn publish_result<...>(cmd, clock, ids) -> Result<ResultPublished> { let now = clock.now(); let event_id = ids.next_event_id(); Ok(ResultPublished::new(cmd.exam_id, cmd.class_id, cmd.section_id, cmd.academic_year_id, 0, cmd.published_at, event_id, cmd.tenant.correlation_id)) }` — no validation. `docs/specs/assessment/commands.md:265-274` lists 3 pre-conditions and the `Materializes ResultStore rows, computes MeritPosition, ExamWisePosition, and AllExamWisePosition, emits ResultPublished per student` effects.

---

### FINDING 63

- **id:** DOMAIN-ASS-063
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/services.rs:1004-1027` and `docs/specs/assessment/commands.md:235-251`
- **description:** The `submit_marks` service does NOT enforce the spec's 2 pre-conditions: all `MarksRegisterChild` rows are present (or the school's `ExamStepSkip` allows partial), and the exam is not yet published. The function just mints a `MarksSubmitted` event with `subject_count: 0` and nil-UUID `exam_id`/`class_id`/`section_id` (Finding DOMAIN-ASS-029).
- **expected:** Two pre-conditions checks.
- **evidence:** `crates/domains/assessment/src/services.rs:1004-1027` `pub fn submit_marks<...>(cmd, clock, ids) -> Result<MarksSubmitted> { ... let _placeholder_exam = ExamId::new(cmd.marks_register_id.school_id(), uuid::Uuid::nil()); ... Ok(MarksSubmitted::new(cmd.marks_register_id, _placeholder_exam, _placeholder_class, _placeholder_section, 0, ...)) }` — no validation. `docs/specs/assessment/commands.md:245-251` lists 2 pre-conditions.

---

### FINDING 64

- **id:** DOMAIN-ASS-064
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/assessment/src/services.rs:317-357` (and parallel `update_exam_schedule`, `cancel_exam_schedule`, `generate_seat_plan`, `update_seat_plan`, `cancel_seat_plan`, `generate_admit_card`, `regenerate_admit_card`, `cancel_admit_card`, `initialize_marks_register`, `enter_marks`, `submit_marks`, `cancel_marks_register`, `publish_result`, `republish_result`, `update_result_remarks`, `generate_report_card`)
- **description:** The Workstream B and C service functions all use `_cmd`, `_ctx`, or `_ids` parameter names with leading underscores (e.g. lines 318, 415, 525, 585, 588, 1012-1014). The leading underscore is the conventional Rust signal that a parameter is intentionally unused; the corresponding `clippy::used_underscore_binding` lint would catch the contradiction. The functions actually do use most of the parameters in the event minting; the underscores are an artefact of the "minimal-shape pure factory functions" template.
- **expected:** Rename parameters without leading underscores to reflect actual usage.
- **evidence:** `crates/domains/assessment/src/services.rs:318` `pub fn schedule_exam<C, G>(_cmd: ScheduleExamCommand, clock: &C, ids: &G)`. `services.rs:415` `pub fn cancel_exam_schedule<C, G>(schedule: &mut ExamSchedule, cmd: CancelExamScheduleCommand, clock: &C, _ids: &G)`. `services.rs:1012-1014` uses `let _placeholder_exam = ...`.

---

### FINDING 65

- **id:** DOMAIN-ASS-065
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/assessment/src/lib.rs:48-54` (re-export list) and `docs/specs/assessment/value-objects.md:11-117`
- **description:** The `lib.rs` re-export list claims to expose "Typed ids and value objects the assessment crate re-exports for downstream consumers" but omits several defined in the same crate's `value_objects.rs`, including the `MarksGradeRow` (defined at value_objects.rs:428), `StaffId` placeholder (value_objects.rs:155), and `ClassRoomId` placeholder (value_objects.rs:165). It also omits the `ResultPublished` event field types and the `ResultStoreId` is included but `ResultStore` aggregate is not. Downstream consumers cannot easily use the re-exports for the full spec surface.
- **expected:** Re-exports for every pub item a downstream consumer would use.
- **evidence:** `crates/domains/assessment/src/lib.rs:48-54` lists 32 re-exports. `value_objects.rs:155, 165, 428` define `StaffId`, `ClassRoomId`, `MarksGradeRow` which are not in the re-export list.

---

### FINDING 66

- **id:** DOMAIN-ASS-066
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/services.rs:18` and `docs/build-plan.md:661-663`
- **description:** The build-plan § "Phase 4 Risks" states: "Result computation is policy-heavy. Mitigation: keep all grading rules in `policies.rs` as pure functions with table-driven fixtures." The assessment crate does not have a `policies.rs` file. The grading rules are inlined in `services.rs` as methods of `ResultService` and use hard-coded A-F thresholds.
- **expected:** A `policies.rs` module containing the table-driven grading fixtures.
- **evidence:** `docs/build-plan.md:661-663` "Result computation is policy-heavy. Mitigation: keep all grading rules in policies.rs as pure functions with table-driven fixtures." `crates/domains/assessment/src/services.rs:1-1380` — no `policies.rs`; the `ResultService::compute_grade` method at lines 1182-1203 hard-codes the thresholds in a chain of `if percent >= 90.0 ... else if percent >= 80.0 ...`.

---

### FINDING 67

- **id:** DOMAIN-ASS-067
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/assessment/src/services.rs:1176-1357` and `docs/specs/assessment/services.md:99-105, 107-110`
- **description:** The spec defines `ResultService::build_custom_temporary(result: &ResultStore, custom: &CustomResultSetting) -> CustomTemporaryResult` (services.md:107-110) and `ResultService::publish` (services.md:112-118). Neither ships in the code.
- **expected:** Both methods on `ResultService`.
- **evidence:** `docs/specs/assessment/services.md:107-110` `pub fn build_custom_temporary(result: &ResultStore, custom: &CustomResultSetting) -> CustomTemporaryResult`; `docs/specs/assessment/services.md:112-118` `pub fn publish(exam, section, registers, scale) -> Result<PublishOutcome, ValidationError>`. Neither is in `crates/domains/assessment/src/services.rs:1176-1357`.

---

### FINDING 68

- **id:** DOMAIN-ASS-068
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/assessment/src/commands.rs:50-68` and `docs/specs/assessment/events.md:512`
- **description:** The crate ships 12 `ASSESSMENT_*_COMMAND_TYPE` constants but no `ASSESSMENT_ONLINE_EXAM_*`, `ASSESSMENT_MARKS_GRADE_*`, `ASSESSMENT_EXAM_SETTING_*`, etc. constants. The spec's full event catalog and the `docs/commands/assessment.md` table require ~62 command types. Only the 12 workstream-A + B + C constants are present.
- **expected:** 62 `ASSESSMENT_*_COMMAND_TYPE` constants matching the command catalog.
- **evidence:** `crates/domains/assessment/src/commands.rs:50-68` declares 12 constants. `docs/commands/assessment.md:12-74` lists 62 command rows.

---

### FINDING 69

- **id:** DOMAIN-ASS-069
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/services.rs:981-998` and `docs/specs/assessment/aggregates.md:202-220`
- **description:** The `enter_marks` service does not enforce the `MarksRegisterChild` invariants the spec mandates: if `is_absent=true` then `Marks` is treated as zero (and the school absent rule is applied), `Marks >= 0`, and `Marks <= FullMark`. None of these checks are performed; the function just passes the raw `f32` into the event.
- **expected:** A validation step that consults the `MarksRegisterChild` and applies the absent rule + the `Marks <= FullMark` check.
- **evidence:** `crates/domains/assessment/src/services.rs:981-998` `pub fn enter_marks<...>(cmd, clock, ids) -> Result<MarksEntered> { ... Ok(MarksEntered::new(cmd.marks_register_id, cmd.subject_id, cmd.student_id, cmd.marks, cmd.is_absent, ...)) }` — no validation. `docs/specs/assessment/aggregates.md:215-219` lists 4 invariants.

---

### FINDING 70

- **id:** DOMAIN-ASS-070
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/assessment/src/services.rs:1004-1027` and `docs/specs/assessment/commands.md:235-251`
- **description:** The `submit_marks` service emits a `MarksSubmitted` event with `subject_count: 0` (line 1022) — a hardcoded value, not the actual count. The spec mandates the real `subject_count` so downstream services can correlate.
- **expected:** `subject_count: u32` populated from the actual `MarksRegisterChild` row count.
- **evidence:** `crates/domains/assessment/src/services.rs:1022` `0,` (the 5th argument of `MarksSubmitted::new`).

---

### FINDING 71

- **id:** DOMAIN-ASS-071
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/events.rs:1196-1212` and `docs/events/assessment.md:25`
- **description:** The `ResultPublished` event declares `AGGREGATE_TYPE: "result_store"` (line 1199) but `aggregate_id` returns `self.exam_id.as_uuid()` (line 1204) — not the `ResultStore` id. The events catalog at `docs/events/assessment.md:25` lists the aggregate as `Result`. The mismatch means the event's aggregate id and aggregate type do not agree (a `result_store` aggregate type but an `exam` aggregate id).
- **expected:** `aggregate_id` returns the result_store's id, or `AGGREGATE_TYPE` is `"result"` and `aggregate_id` returns the result_store's id.
- **evidence:** `crates/domains/assessment/src/events.rs:1196-1212` `const AGGREGATE_TYPE: &'static str = "result_store"; ... fn aggregate_id(&self) -> uuid::Uuid { self.exam_id.as_uuid() }`.

---

### FINDING 72

- **id:** DOMAIN-ASS-072
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/value_objects.rs:24-26, 145-166` and `docs/specs/assessment/aggregates.md:3-41, 84-114, 152-167, 685-749`
- **description:** The crate's `value_objects.rs` defines 2 placeholder typed ids (`StaffId` lines 151-156, `ClassRoomId` lines 158-166) with comments "Placeholder until the HR domain ships its Staff aggregate in Phase 6" / "Placeholder until the facilities domain ships its Room aggregate in Phase 8". These placeholders are typed wrappers around `Uuid` and look semantically equivalent to the real ids that will land in later phases. The 5 other crates in the workspace will need to handle the transition from placeholder to real id (e.g., `educore-finance` may need `StaffId` for fee exemption rules). The crate has no migration plan or compatibility shim documented.
- **expected:** A `phase-6-compatibility.md` / `phase-8-compatibility.md` doc, or a `From<StaffId> for educore_hr::StaffId` shim, or a typed-id registry that names which crate owns the canonical id.
- **evidence:** `crates/domains/assessment/src/value_objects.rs:151-156` `/// A typed id for a Staff aggregate (the invigilating teacher for an exam). Placeholder until the HR domain ships its \`Staff\` aggregate in Phase 6. pub struct StaffId;`. Similar at lines 158-166 for `ClassRoomId`.

---

### FINDING 73

- **id:** DOMAIN-ASS-073
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/assessment/src/aggregate.rs:432-491` and `docs/specs/assessment/aggregates.md:751-803`
- **description:** The `AdmitCard` aggregate is missing the `ExamType` `ExamTitle` `ExamSetting` etc. per-school / per-academic-year state the spec describes. The aggregate also has no `is_published` flag (the spec at lines 763-770 specifies 3 invariants, but the code has only the basic 10-field audit footer).
- **expected:** The `AdmitCard` aggregate should carry the `AdmitCardSetting` snapshot (the school-side branding that determines which fields appear on the rendered card) and the immutable-vs-regeneration lifecycle fields.
- **evidence:** `crates/domains/assessment/src/aggregate.rs:432-456` defines the `AdmitCard` struct with 7 fields (id, school_id, student_record_id, exam_type_id, academic_year_id, generated_at, plus 10 audit fields). `docs/specs/assessment/aggregates.md:751-803` defines 3 invariants and lifecycle.

---

### FINDING 74

- **id:** DOMAIN-ASS-074
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/services.rs:77-161` (create_exam) and `docs/specs/assessment/commands.md:62-86`
- **description:** The `create_exam` service mints the `ExamCreated` event with the `name` and `code` as raw `String` (events.rs:107-110) instead of typed `ExamName` and `ExamCode` newtypes. The spec's event definition at `docs/specs/assessment/events.md:59-69` does not show the `name`/`code` fields, but if the engine policy is to use typed wrappers, the event's wire format should also use them.
- **expected:** The event's wire format uses the typed wrappers (or a documented rationale for the deviation).
- **evidence:** `crates/domains/assessment/src/events.rs:107-110` `name: name.as_str().to_owned(), code: code.as_str().to_owned(), exam_mark: exam_mark.as_f32(), pass_mark: pass_mark.as_f32(),`. The engine rule at `AGENTS.md` is "Compile-time safety over strings."

---

### FINDING 75

- **id:** DOMAIN-ASS-075
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/assessment/src/services.rs:585-604` and `docs/specs/assessment/commands.md:458-471`
- **description:** The `regenerate_admit_card` service does not check that the previous admit card exists, is not already cancelled, or that the regeneration reason is non-empty. The function just mints a new event.
- **expected:** Three pre-conditions checks: previous-card-exists, previous-card-not-cancelled, reason-non-empty.
- **evidence:** `crates/domains/assessment/src/services.rs:585-604` `pub fn regenerate_admit_card<...>(cmd, clock, _ids) -> Result<AdmitCardRegenerated> { ... Ok(AdmitCardRegenerated::new(cmd.admit_card_id, cmd.previous_id, cmd.reason, event_id, cmd.tenant.correlation_id, now)) }` — no validation. `docs/specs/assessment/commands.md:464-471` describes the spec's `SetExamSignature` (different command) and the spec at `docs/specs/assessment/aggregates.md:768-769` "Once generated, the card is immutable; a re-generation supersedes the previous card with a new id and emits a new event."

---

### FINDING 76

- **id:** DOMAIN-ASS-076
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/assessment/src/repository.rs:39-235` and `docs/specs/assessment/permissions.md:39`
- **description:** The spec at `docs/specs/assessment/permissions.md:39` lists `ExamSchedule.Read` as a capability. The repository's `ExamScheduleRepository` does not consult the RBAC subsystem; it accepts any `TenantContext` and assumes the actor is authorized. The trait has no awareness of the `Capability` enum.
- **expected:** Either the trait methods take a `Capability` parameter, or the dispatcher (caller of the trait) is documented as the enforcement point.
- **evidence:** `crates/domains/assessment/src/repository.rs:115-143` defines 6 methods that all take `&TenantContext` and return `Result<...>` with no capability check. `docs/specs/assessment/permissions.md:37-39` lists `ExamSchedule.Read` as a distinct capability.

---

### FINDING 77

- **id:** DOMAIN-ASS-077
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/assessment/src/aggregate.rs:58-107` (`Exam`) and `docs/specs/assessment/commands.md:88-103`
- **description:** The `Exam` aggregate has no `is_locked: bool` or `locked_at: Option<Timestamp>` field, but the spec at `docs/specs/assessment/commands.md:100-102` says "Marks have not yet been entered against the exam. Once marks exist, the exam is locked." The `ExamService::lock_after_publish` method (services.md:28-29) is missing (see Finding DOMAIN-ASS-057) so the lock invariant is unenforceable.
- **expected:** `pub is_locked: bool` and `pub locked_at: Option<Timestamp>` fields on `Exam`, plus the `ExamService::lock_after_publish` method.
- **evidence:** `crates/domains/assessment/src/aggregate.rs:58-107` defines 18 fields, none of which is `is_locked`. `docs/specs/assessment/commands.md:99-102` describes the lock invariant. `docs/specs/assessment/services.md:28-29` describes `ExamService::lock_after_publish`.

---

### FINDING 78

- **id:** DOMAIN-ASS-078
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/assessment/src/aggregate.rs:58-107` and `docs/specs/assessment/commands.md:62-86`
- **description:** The `Exam` aggregate's `name: ExamName` and `code: ExamCode` fields are typed wrappers, but the aggregate's `fresh` constructor takes `name: ExamName, code: ExamCode` (aggregate.rs:123-124) directly. The corresponding `CreateExamCommand` carries `name: String, code: String` (commands.rs:95-97), so the service must validate the strings into newtypes (services.rs:91-92). A better design would have the command carry the typed wrappers, eliminating the validation-on-service-boundary pattern.
- **expected:** `CreateExamCommand` carries `name: ExamName, code: ExamCode` (see Finding DOMAIN-ASS-011).
- **evidence:** `crates/domains/assessment/src/aggregate.rs:123-124` (constructor takes typed wrappers) vs. `crates/domains/assessment/src/commands.rs:95-97` (command carries `String`).

---

### FINDING 79

- **id:** DOMAIN-ASS-079
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/assessment/src/lib.rs:60-67` and `docs/specs/assessment/services.md:303-345`
- **description:** The `prelude` re-exports the `Capability` enum but does not re-export the `ResultEligibility` / `AdmitCardEligibility` / `ActiveExamSchedule` / `PendingOnlineExam` policies. The policies are not implemented (Finding DOMAIN-ASS-058) so the re-exports would be empty, but the spec mandates them and they should be in the public surface.
- **expected:** Re-exports of the policy/specification types from the prelude once they are implemented.
- **evidence:** `crates/domains/assessment/src/lib.rs:60-67` `pub use educore_rbac::value_objects::Capability;` only. `docs/specs/assessment/services.md:303-345` defines 4 policy/specification structs.

---

### FINDING 80

- **id:** DOMAIN-ASS-080
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/services.rs:1011-1027` and `docs/specs/assessment/services.md:54-58`
- **description:** The `MarksService::submit` function (services.md:54-58) enforces the partial-submission rule (rejects partial submissions unless the school has configured `ExamStepSkip`). The shipped `submit_marks` function (services.rs:1004-1027) does NOT consult the `ExamStepSkip` setting and does NOT check that all `MarksRegisterChild` rows are present.
- **expected:** `MarksService::submit` (or equivalent) that consults `ExamStepSkip` and either allows or rejects the submission.
- **evidence:** `crates/domains/assessment/src/services.rs:1004-1027` (no `ExamStepSkip` consultation). `docs/specs/assessment/services.md:54-58` `pub fn submit(register: &mut MarksRegister) -> Result<(), ValidationError>` and the doc-comment "MarksService::submit enforces partial-submission rules: when exam_step_skips indicates partial submission is allowed, missing subjects are tolerated; otherwise the register must be complete."

---

### FINDING 81

- **id:** DOMAIN-ASS-081
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/assessment/src/services.rs:643-645` (`school_matches`) and `docs/specs/assessment/services.md:301-345`
- **description:** The `school_matches` helper duplicates the per-tenant check that should live in the engine facade (per the handoff at `docs/handoff/PHASE-4-HANDOFF.md:694-696`: "The capability check boundary was resolved as dispatcher-level"). The helper is re-exported from the prelude (lib.rs:105) and may be called inconsistently — services that DO NOT call it (e.g., `enter_marks`, `submit_marks`, `publish_result`, `regenerate_admit_card`, `republish_result`, `update_result_remarks`, `generate_report_card`, `cancel_*`) silently accept cross-tenant commands.
- **expected:** The dispatcher enforces the school match; the per-service functions are documented as assuming the dispatcher has already done so.
- **evidence:** `crates/domains/assessment/src/services.rs:643-645` `pub fn school_matches(ctx: &TenantContext, school: SchoolId) -> bool { ctx.school_id == school }`. `crates/domains/assessment/src/lib.rs:105` re-exports `school_matches` from the prelude. The Workstream B + C services (lines 317-1149) do not call `school_matches`.

---

### FINDING 82

- **id:** DOMAIN-ASS-082
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/aggregate.rs:58-107` and `docs/specs/assessment/aggregates.md:65-69`
- **description:** The `Exam` aggregate has a `pass_mark: PassMark` field and an `exam_mark: ExamMark` field, but no `PassMark` validation that the constructor `Exam::fresh` enforces. The `Exam::fresh` constructor at `crates/domains/assessment/src/aggregate.rs:116-156` does NOT check `pass_mark.as_f32() <= exam_mark.as_f32()`. The service `create_exam` does the check (services.rs:97-103), but the aggregate itself is constructed without the invariant — bypassing the service breaks the invariant.
- **expected:** The `Exam::fresh` constructor enforces `pass_mark <= exam_mark` (or returns a `Result<Self, DomainError>`).
- **evidence:** `crates/domains/assessment/src/aggregate.rs:116-156` `pub fn fresh(...pass_mark: PassMark, ..., exam_mark: ExamMark, ...) -> Self { Self { ... pass_mark, exam_mark, ... } }` — no check. The check is in `services.rs:97-103`.

---

### FINDING 83

- **id:** DOMAIN-ASS-083
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/assessment/src/aggregate.rs:504-560` and `docs/specs/assessment/aggregates.md:169-220`
- **description:** The `MarksRegister` aggregate's `is_open: bool` field is set to `true` in the `fresh` constructor and never set to `false` by any service. The spec says `is_open` is `true` while the register is being entered, `false` once `submit_marks` locks it. The `submit_marks` service (services.rs:1004-1027) mints a `MarksSubmitted` event but does not mutate the aggregate's `is_open` field.
- **expected:** `submit_marks` mutates the aggregate's `is_open` to `false` before emitting the event.
- **evidence:** `crates/domains/assessment/src/aggregate.rs:529-559` `pub fn fresh(...) -> Self { Self { ..., is_open: true, ... } }`. `crates/domains/assessment/src/services.rs:1004-1027` `pub fn submit_marks<...>(cmd: SubmitMarksCommand, clock: &C, ids: &G) -> Result<MarksSubmitted> { ... Ok(MarksSubmitted::new(...)) }` — no aggregate mutation.

---

### FINDING 84

- **id:** DOMAIN-ASS-084
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/aggregate.rs:432-491` and `docs/specs/assessment/aggregates.md:751-803`
- **description:** The `AdmitCard` aggregate has an `active_status: ActiveStatus` field that is used as a soft-delete flag, but the `cancel_admit_card` service does not take a `&mut AdmitCard` parameter (services.rs:608-631) — it does. However, the `generate_admit_card` service constructs a new `AdmitCard` but the spec mandates "Once generated, the card is immutable; a re-generation supersedes the previous card with a new id and emits a new event" (aggregates.md:768-769). The `regenerate_admit_card` service does NOT mutate the previous card's `active_status` to `Retired`, leaving the old card active and the uniqueness invariant broken.
- **expected:** `regenerate_admit_card` takes a `&mut AdmitCard` for the previous card and sets `active_status = Retired` before emitting the new event.
- **evidence:** `crates/domains/assessment/src/services.rs:585-604` `pub fn regenerate_admit_card<...>(cmd, clock, _ids) -> Result<AdmitCardRegenerated> { ... Ok(AdmitCardRegenerated::new(cmd.admit_card_id, cmd.previous_id, cmd.reason, ...)) }` — no aggregate mutation. `docs/specs/assessment/aggregates.md:768-769` "Once generated, the card is immutable; a re-generation supersedes the previous card with a new id and emits a new event."

---

### FINDING 85

- **id:** DOMAIN-ASS-085
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/assessment/src/aggregate.rs:258-348` and `docs/specs/assessment/aggregates.md:130-138`
- **description:** The `ExamSchedule` aggregate has no `academic_year_id: AcademicYearId` field. The spec at `docs/specs/assessment/aggregates.md:133-137` lists 5 invariants: uniqueness by `(exam_id, class_id, section_id)` per academic year, `StartTime < EndTime`, no teacher overlap, no room overlap, date within academic year. Without `academic_year_id`, the uniqueness and date-in-range invariants are unenforceable.
- **expected:** `pub academic_year_id: AcademicYearId` on `ExamSchedule`.
- **evidence:** `crates/domains/assessment/src/aggregate.rs:258-296` defines the `ExamSchedule` struct with 15 fields, none of which is `academic_year_id`. `docs/specs/assessment/aggregates.md:133` "Unique by (exam_id, class_id, section_id) per academic year" and line 137 "Date is within the academic year".

---

### FINDING 86

- **id:** DOMAIN-ASS-086
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/assessment/src/aggregate.rs:360-420` and `docs/specs/assessment/aggregates.md:685-749`
- **description:** The `SeatPlan` aggregate has no `academic_year_id: AcademicYearId` field. The spec at `docs/specs/assessment/aggregates.md:702-705` requires uniqueness by `(exam_type_id, class_id, section_id, academic_id)` — without the `academic_year_id` field, the invariant is unenforceable.
- **expected:** `pub academic_year_id: AcademicYearId` on `SeatPlan`.
- **evidence:** `crates/domains/assessment/src/aggregate.rs:360-383` defines the `SeatPlan` struct with 14 fields, none of which is `academic_year_id` or `exam_type_id`. The `generate_seat_plan` command is also missing `exam_type_id` (Finding DOMAIN-ASS-016).

---

### FINDING 87

- **id:** DOMAIN-ASS-087
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/assessment/src/aggregate.rs:360-420` and `docs/specs/assessment/aggregates.md:685-749`
- **description:** The `SeatPlan` aggregate has no `exam_type_id: ExamTypeId` field. The spec at `docs/specs/assessment/aggregates.md:702` requires uniqueness by `(exam_type_id, class_id, section_id, academic_id)`. The aggregate has only `exam_id: ExamId` (line 366).
- **expected:** `pub exam_type_id: ExamTypeId` on `SeatPlan`.
- **evidence:** `crates/domains/assessment/src/aggregate.rs:366` `pub exam_id: ExamId,` (no `exam_type_id`). `docs/specs/assessment/aggregates.md:702` "Unique by (exam_type_id, class_id, section_id, academic_id)".

---

### FINDING 88

- **id:** DOMAIN-ASS-088
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/assessment/src/aggregate.rs:360-420` and `docs/specs/assessment/commands.md:408-435`
- **description:** The `SeatPlan` aggregate's `total_students: u32` is stored as a top-level field (line 372) but the spec at `docs/specs/assessment/commands.md:431-432` says "Sum of assign_students equals the section's student count" — i.e., the invariant is a derived value, not a stored one. Storing it allows drift between the children and the parent.
- **expected:** `total_students` is a derived accessor over `children.iter().map(|c| c.assign_students).sum::<u32>()`.
- **evidence:** `crates/domains/assessment/src/aggregate.rs:372` `pub total_students: u32,` and `crates/domains/assessment/src/aggregate.rs:397` constructor argument. `docs/specs/assessment/commands.md:431-432` "Sum of assign_students equals the section's student count."

---

### FINDING 89

- **id:** DOMAIN-ASS-089
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/assessment/src/aggregate.rs:258-348` and `docs/specs/assessment/aggregates.md:152-167`
- **description:** The `ExamSchedule` aggregate's `room_id: Option<ClassRoomId>` and `teacher_id: Option<StaffId>` are top-level fields (lines 280, 284), but the spec at `docs/specs/assessment/aggregates.md:128-129` says these are per-subject overrides in `ExamScheduleSubject`. The aggregate conflates the per-schedule defaults with the per-subject overrides.
- **expected:** The `ExamSchedule` aggregate's `room_id` and `teacher_id` are derived (or default) from `children.iter().all(|c| c.room_id == self.room_id)`. The per-subject room/teacher overrides live on `ExamScheduleSubject`.
- **evidence:** `crates/domains/assessment/src/aggregate.rs:280, 284` `pub room_id: Option<ClassRoomId>, pub teacher_id: Option<StaffId>`. `docs/specs/assessment/aggregates.md:128-129` "the room the exam is held in (default for all subjects in this slot; per-subject overrides in ExamScheduleSubject)".

---

### FINDING 90

- **id:** DOMAIN-ASS-090
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/events.rs:155-180` (`ExamUpdated`) and `docs/specs/assessment/aggregates.md:46-82`
- **description:** The `ExamUpdated` event stores the changes as `Vec<String>` (events.rs:153), but the spec describes `Vec<&'static str>` (events.md:74). The wire format mismatch means a deserialised event would not round-trip through a strict schema validator. Additionally, the event has no `old_value: Option<...>` and `new_value: Option<...>` fields (which would be needed for downstream subscribers to know what changed).
- **expected:** A change-list with stable, namespaced identifiers (e.g., `"exam_mark"`, `"pass_mark"`) as the spec mandates, or a richer diff payload.
- **evidence:** `crates/domains/assessment/src/events.rs:153` `pub changes: Vec<String>,`. `docs/specs/assessment/events.md:74` `pub changes: Vec<&'static str>,`.

---

### FINDING 91

- **id:** DOMAIN-ASS-091
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/assessment/src/events.rs:1141-1182` and `docs/specs/assessment/commands.md:641-653`
- **description:** The `ResultRemarksUpdated` event's `teacher_remarks: String` is unbounded. The spec at `docs/specs/assessment/value-objects.md:81-82` mandates the `Remark` newtype with a 1..=2000 char bound. The event accepts any string, including empty or 10MB-long ones.
- **expected:** `pub teacher_remarks: Remark` (typed wrapper, validated at construction).
- **evidence:** `crates/domains/assessment/src/events.rs:1143` `pub teacher_remarks: String,`. `docs/specs/assessment/value-objects.md:81-82` `Remark | 1..2000 chars`. `crates/domains/assessment/src/commands.rs:646` `pub teacher_remarks: String,` (the command also carries the raw string).

---

### FINDING 92

- **id:** DOMAIN-ASS-092
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/assessment/src/commands.rs:232-242` and `docs/specs/assessment/commands.md:176-188`
- **description:** The `CancelExamScheduleCommand` carries `reason: String` (commands.rs:235) but has no length or non-empty validation. The spec describes the reason as a free-text field but it should at minimum be non-empty.
- **expected:** A validation in the service that the reason is non-empty.
- **evidence:** `crates/domains/assessment/src/commands.rs:235` `pub reason: String,` (no MAX_LEN constant, no validator function).

---

### FINDING 93

- **id:** DOMAIN-ASS-093
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/assessment/src/commands.rs:330-340` (`CancelAdmitCardCommand`) and `docs/specs/assessment/commands.md:176-188`
- **description:** The `CancelAdmitCardCommand` carries `reason: String` (commands.rs:333) without validation. The `regenerate_admit_card` service stores the reason in the event without checking that it is non-empty.
- **expected:** A validator function for the cancel/regenerate reasons, or the typed `Reason` newtype.
- **evidence:** `crates/domains/assessment/src/commands.rs:333` `pub reason: String,` (no MAX_LEN constant, no validator function).

---

### FINDING 94

- **id:** DOMAIN-ASS-094
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/assessment/src/entities.rs:113-150` (`MarksRegisterChild`) and `docs/specs/assessment/aggregates.md:202-220`
- **description:** The `MarksRegisterChild` entity's `gpa_point: Option<Gpa>` and `gpa_grade: Option<Grade>` fields are typed wrappers, but the constructor `MarksRegisterChild::fresh` does not exist (only field initialisation at lines 113-140). The `enter_marks` service (services.rs:981-998) does not compute the grade and grade point; the `MarksEntered` event does not carry them. The grading is deferred to the `submit_marks`/`publish_result` flow but no path from `MarksRegisterChild` to `MarksRegister`/`ResultStore` exists.
- **expected:** Either the `enter_marks` service computes and stores the grade/grade point on the `MarksRegisterChild`, or a `compute_subject_marks` post-step is invoked.
- **evidence:** `crates/domains/assessment/src/entities.rs:113-140` (no `fresh` constructor). `crates/domains/assessment/src/services.rs:981-998` `enter_marks` does not invoke `ResultService::compute_subject_marks`. `docs/specs/assessment/aggregates.md:215-218` lists 4 invariants.

---

### FINDING 95

- **id:** DOMAIN-ASS-095
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/assessment/src/services.rs:317-357` (`schedule_exam`) and `docs/specs/assessment/aggregates.md:130-138`
- **description:** The `schedule_exam` service does not enforce the spec's invariant 5: "Date is within the academic year." The function accepts any `NaiveDate` and does not consult the `AcademicYear` aggregate to determine the year boundaries.
- **expected:** A pre-condition check that consults the academic year boundaries.
- **evidence:** `crates/domains/assessment/src/services.rs:317-357` (no date-range check). `docs/specs/assessment/aggregates.md:137` "Date is within the academic year."

---

### FINDING 96

- **id:** DOMAIN-ASS-096
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/assessment/src/services.rs:1011-1027` and `docs/build-plan.md:629-640`
- **description:** The build-plan § "Phase 4 Tasks" task 4 says: "Integration test: schedule an exam, enter marks, compute result, publish report card." The shipped `assessment_integration.rs` only exercises `create_exam` (per the handoff at `docs/handoff/PHASE-4-HANDOFF.md:66-71`). The 4-step workflow (schedule → enter → compute → publish) is not integration-tested.
- **expected:** A workflow integration test that exercises `schedule_exam`, `enter_marks`, `submit_marks`, `publish_result`, `generate_report_card` end-to-end.
- **evidence:** `crates/tools/storage-parity/tests/assessment_integration.rs:1-499` — only `create_exam` is exercised. The handoff at `docs/handoff/PHASE-4-HANDOFF.md:66-71` admits the limited scope. `docs/build-plan.md:636-637` "Integration test: schedule an exam, enter marks, compute result, publish report card. Verify outbox + audit + RLS."

---

### FINDING 97

- **id:** DOMAIN-ASS-097
- **area:** domain-crates
- **severity:** High
- **location:** `docs/coverage.toml:555-624` and `docs/specs/assessment/aggregates.md:1-1025`
- **description:** The 8 assessment coverage rows in `coverage.toml` are marked `status = "Tested"` but the 8 aggregates they reference (`assessment_exams_aggregate`, `assessment_marks_registers_aggregate`, `assessment_exam_schedules_aggregate`, `assessment_result_stores_aggregate`, `assessment_report_cards_aggregate`, `assessment_online_exams_aggregate`, `assessment_seat_plans_aggregate`, `assessment_admit_cards_aggregate`) are not all implemented. Specifically: (a) `OnlineExam` is not implemented (Finding DOMAIN-ASS-002), (b) `ReportCard` is documented as a "projection" without a backing aggregate (handoff:67-71). Marking these `Tested` overstates the coverage.
- **expected:** The `Tested` status reflects actual coverage. `OnlineExam` and `ReportCard` should be `Pending` or `Partial` until they are implemented and tested.
- **evidence:** `docs/coverage.toml:600-606` `status = "Tested"` for `assessment_online_exams_aggregate`. `docs/coverage.toml:591-597` `status = "Tested"` for `assessment_report_cards_aggregate` (note "projection" in the item field). `docs/handoff/PHASE-4-HANDOFF.md:67-71` "The full state machine ships at the Event level (8 events) but the integration test only exercises create_exam".

---

### FINDING 98

- **id:** DOMAIN-ASS-098
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/lib.rs:14-17` and `docs/build-plan.md:665-705`
- **description:** The build-plan § "Phase 4 outcome" declares the phase "Closed 2026-06-12" with 433 tests passing. The actual state of the crate has 6 of 8 prompt-named aggregates (Phase 4 build-plan task 1), 21 of 62 spec commands, 21 of 65 spec events, 1 of 8 service structs, 6 of 13 repository traits, 0 of 47 `#[derive(DomainQuery)]` emissions, 0 workflow integration tests, and `Uuid::nil()` placeholders in production event payloads (Findings DOMAIN-ASS-029 to DOMAIN-ASS-031). The phase is not "closed" in any production-ready sense.
- **expected:** Either re-open the phase until the spec is honored, or formally defer the missing scope to a follow-up phase with a clear manifest of what is and is not in Phase 4.
- **evidence:** `docs/build-plan.md:665-705` `**Phase 4 outcome.** Closed 2026-06-12. **`educore-assessment`** delivered as the second domain crate. The full prompt-named subset ships: 8 aggregates ... 28 typed commands, 28 typed events ... 25+ pure factory services, 8 repository port traits, 8 typed query stubs`. The code at `crates/domains/assessment/src/` ships 6 aggregates, 21 commands, 21 events, 1 service struct, 6 repository traits, 6 query stubs.

---

### FINDING 99

- **id:** DOMAIN-ASS-099
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/assessment/src/value_objects.rs:151-156, 158-166` and `crates/domains/assessment/src/aggregate.rs:280, 284, 439`
- **description:** The crate defines placeholder `StaffId` and `ClassRoomId` typed ids at `value_objects.rs:151-156, 158-166` and uses them in `ExamSchedule::room_id` and `teacher_id` (aggregate.rs:280, 284) and in `AdmitCard` (via command fields at commands.rs:189, 247). When Phase 6 (HR) and Phase 8 (Facilities) ship the canonical `StaffId` and `ClassRoomId` (per `docs/specs/assessment/dependencies` and the comment in value_objects.rs:151-156), the placeholder types will collide with the canonical types. No `From` conversion, type alias, or migration plan is documented.
- **expected:** Either (a) the placeholder types are removed and the foreign-key fields are deferred until Phase 6/8, or (b) explicit `From` conversions are provided so downstream crates can migrate.
- **evidence:** `crates/domains/assessment/src/value_objects.rs:151-156` `pub struct StaffId;` and `:158-166` `pub struct ClassRoomId;` with "Placeholder until the HR domain ships" comments. The handoff at `docs/handoff/PHASE-4-HANDOFF.md:284-287` says "placeholder typed ids for StaffId / ClassRoomId (the academic crate's missing ids; the full definitions land in the HR workstream in Phase 6 + the facilities workstream in Phase 8)".

---

### FINDING 100

- **id:** DOMAIN-ASS-100
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/assessment/src/services.rs:351, 1274, 1012-1014, 1080-1100, 1140-1148` and `docs/code-standards.md`
- **description:** The `unwrap_or(u32::MAX)` pattern at `services.rs:351, 1274` and the `Uuid::nil()` placeholders at `services.rs:1012-1014, 1080-1100, 1140-1148` represent silent fallbacks that hide data-integrity errors. The `unwrap_or(u32::MAX)` saturates the count to `4_294_967_295` if the actual count overflows `u32`; downstream consumers (finance, attendance, communication) cannot distinguish a real count from a saturated one. The `Uuid::nil()` placeholders are a data-corruption risk: storage adapters will write a valid row with `exam_id = 00000000-...` and downstream subscribers (communication, finance, cms, academic) will silently fail to correlate.
- **expected:** Propagate the overflow / missing-id errors as `Result::Err(DomainError::validation(...))` instead of silently saturating or substituting nil UUIDs.
- **evidence:** `crates/domains/assessment/src/services.rs:351` `u32::try_from(_cmd.subjects.len()).unwrap_or(u32::MAX)`. `crates/domains/assessment/src/services.rs:1274` `current_rank += u32::try_from(j - i + 1).unwrap_or(u32::MAX)`. `services.rs:1012-1014` `let _placeholder_exam = ExamId::new(cmd.marks_register_id.school_id(), uuid::Uuid::nil());` (and parallel lines). `services.rs:1143` `ExamId::new(cmd.result_store_id.school_id(), uuid::Uuid::nil())`.

---

### END FINDINGS

Total findings: 100
