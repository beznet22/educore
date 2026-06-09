# Assessment Domain — Repositories

Repositories are ports (Rust traits). Adapters implement them. The
default adapter targets PostgreSQL; an SQLite adapter is provided
for embedded deployments.

## ExamTypeRepository

```rust
#[async_trait]
pub trait ExamTypeRepository: Send + Sync {
    async fn get(&self, id: ExamTypeId) -> Result<Option<ExamType>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<ExamType>>;
    async fn find_by_title(&self, school: SchoolId, title: &str) -> Result<Option<ExamType>>;
    async fn insert(&self, t: &ExamType) -> Result<()>;
    async fn update(&self, t: &ExamType) -> Result<()>;
    async fn delete(&self, id: ExamTypeId) -> Result<()>;
}
```

## ExamRepository

```rust
#[async_trait]
pub trait ExamRepository: Send + Sync {
    async fn get(&self, id: ExamId) -> Result<Option<Exam>>;
    async fn list_for_year(&self, school: SchoolId, year: AcademicYearId) -> Result<Vec<Exam>>;
    async fn list_for_class(&self, school: SchoolId, class: ClassId, year: AcademicYearId) -> Result<Vec<Exam>>;
    async fn list_for_type(&self, school: SchoolId, exam_type: ExamTypeId, year: AcademicYearId) -> Result<Vec<Exam>>;
    async fn find(
        &self,
        school: SchoolId,
        exam_type: ExamTypeId,
        class: ClassId,
        section: SectionId,
        subject: SubjectId,
        year: AcademicYearId,
    ) -> Result<Option<Exam>>;
    async fn insert(&self, e: &Exam) -> Result<()>;
    async fn update(&self, e: &Exam) -> Result<()>;
    async fn delete(&self, id: ExamId) -> Result<()>;
}
```

## ExamScheduleRepository

```rust
#[async_trait]
pub trait ExamScheduleRepository: Send + Sync {
    async fn get(&self, id: ExamScheduleId) -> Result<Option<ExamSchedule>>;
    async fn find(
        &self,
        school: SchoolId,
        exam: ExamId,
        class: ClassId,
        section: SectionId,
        year: AcademicYearId,
    ) -> Result<Option<ExamSchedule>>;
    async fn list_for_section(
        &self,
        school: SchoolId,
        class: ClassId,
        section: SectionId,
        year: AcademicYearId,
    ) -> Result<Vec<ExamSchedule>>;
    async fn list_for_teacher(
        &self,
        school: SchoolId,
        teacher: StaffId,
        year: AcademicYearId,
    ) -> Result<Vec<ExamSchedule>>;
    async fn list_for_room(
        &self,
        school: SchoolId,
        room: ClassRoomId,
        year: AcademicYearId,
    ) -> Result<Vec<ExamSchedule>>;
    async fn list_in_range(
        &self,
        school: SchoolId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<ExamSchedule>>;
    async fn insert(&self, s: &ExamSchedule) -> Result<()>;
    async fn update(&self, s: &ExamSchedule) -> Result<()>;
    async fn delete(&self, id: ExamScheduleId) -> Result<()>;
    async fn insert_subject(&self, s: &ExamScheduleSubject) -> Result<()>;
    async fn list_subjects(&self, schedule_id: ExamScheduleId) -> Result<Vec<ExamScheduleSubject>>;
}
```

## MarksRegisterRepository

```rust
#[async_trait]
pub trait MarksRegisterRepository: Send + Sync {
    async fn get(&self, id: MarksRegisterId) -> Result<Option<MarksRegister>>;
    async fn find(
        &self,
        school: SchoolId,
        exam: ExamId,
        student: StudentId,
    ) -> Result<Option<MarksRegister>>;
    async fn list_for_exam(&self, school: SchoolId, exam: ExamId) -> Result<Vec<MarksRegister>>;
    async fn list_for_student(&self, school: SchoolId, student: StudentId) -> Result<Vec<MarksRegister>>;
    async fn insert(&self, r: &MarksRegister) -> Result<()>;
    async fn update(&self, r: &MarksRegister) -> Result<()>;
    async fn upsert_child(&self, c: &MarksRegisterChild) -> Result<()>;
    async fn list_children(&self, register_id: MarksRegisterId) -> Result<Vec<MarksRegisterChild>>;
    async fn child(
        &self,
        register_id: MarksRegisterId,
        subject: SubjectId,
    ) -> Result<Option<MarksRegisterChild>>;
}
```

## MarkStoreRepository

```rust
#[async_trait]
pub trait MarkStoreRepository: Send + Sync {
    async fn get(&self, id: MarkStoreId) -> Result<Option<MarkStore>>;
    async fn list_for_student(&self, school: SchoolId, student: StudentId) -> Result<Vec<MarkStore>>;
    async fn list_for_setup(&self, school: SchoolId, setup: ExamSetupId) -> Result<Vec<MarkStore>>;
    async fn list_for_exam_type(&self, school: SchoolId, exam_type: ExamTypeId) -> Result<Vec<MarkStore>>;
    async fn insert(&self, m: &MarkStore) -> Result<()>;
    async fn update(&self, m: &MarkStore) -> Result<()>;
    async fn delete(&self, id: MarkStoreId) -> Result<()>;
}
```

## ResultRepository

```rust
#[async_trait]
pub trait ResultRepository: Send + Sync {
    async fn get(&self, id: ResultStoreId) -> Result<Option<ResultStore>>;
    async fn list_for_student(&self, school: SchoolId, student: StudentId) -> Result<Vec<ResultStore>>;
    async fn list_for_setup(&self, school: SchoolId, setup: ExamSetupId) -> Result<Vec<ResultStore>>;
    async fn list_for_exam(&self, school: SchoolId, exam: ExamId) -> Result<Vec<ResultStore>>;
    async fn list_for_class_section(
        &self,
        school: SchoolId,
        class: ClassId,
        section: SectionId,
        year: AcademicYearId,
    ) -> Result<Vec<ResultStore>>;
    async fn insert(&self, r: &ResultStore) -> Result<()>;
    async fn update(&self, r: &ResultStore) -> Result<()>;
    async fn insert_merit(&self, m: &MeritPosition) -> Result<()>;
    async fn list_merit(
        &self,
        school: SchoolId,
        class: ClassId,
        section: SectionId,
        exam_term: ExamTypeId,
    ) -> Result<Vec<MeritPosition>>;
    async fn insert_exam_position(&self, p: &ExamWisePosition) -> Result<()>;
    async fn list_exam_position(
        &self,
        school: SchoolId,
        class: ClassId,
        section: SectionId,
        exam: ExamId,
    ) -> Result<Vec<ExamWisePosition>>;
    async fn insert_all_exam_position(&self, p: &AllExamWisePosition) -> Result<()>;
    async fn list_all_exam_position(
        &self,
        school: SchoolId,
        class: ClassId,
        exam: ExamId,
    ) -> Result<Vec<AllExamWisePosition>>;
    async fn insert_custom_temporary(&self, c: &CustomTemporaryResult) -> Result<()>;
    async fn list_custom_temporary(&self, school: SchoolId, exam_type: ExamTypeId) -> Result<Vec<CustomTemporaryResult>>;
    async fn clear_custom_temporary(&self, school: SchoolId, exam_type: ExamTypeId) -> Result<()>;
}
```

## MarksGradeRepository

```rust
#[async_trait]
pub trait MarksGradeRepository: Send + Sync {
    async fn get(&self, id: MarksGradeId) -> Result<Option<MarksGrade>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<MarksGrade>>;
    async fn insert(&self, g: &MarksGrade) -> Result<()>;
    async fn update(&self, g: &MarksGrade) -> Result<()>;
    async fn delete(&self, id: MarksGradeId) -> Result<()>;
}
```

## OnlineExamRepository

```rust
#[async_trait]
pub trait OnlineExamRepository: Send + Sync {
    async fn get(&self, id: OnlineExamId) -> Result<Option<OnlineExam>>;
    async fn list_for_section(
        &self,
        school: SchoolId,
        class: ClassId,
        section: SectionId,
        year: AcademicYearId,
    ) -> Result<Vec<OnlineExam>>;
    async fn list_running(&self, school: SchoolId) -> Result<Vec<OnlineExam>>;
    async fn list_pending(&self, school: SchoolId) -> Result<Vec<OnlineExam>>;
    async fn insert(&self, e: &OnlineExam) -> Result<()>;
    async fn update(&self, e: &OnlineExam) -> Result<()>;
    async fn delete(&self, id: OnlineExamId) -> Result<()>;
    async fn insert_question(&self, q: &OnlineExamQuestion) -> Result<()>;
    async fn list_questions(&self, exam_id: OnlineExamId) -> Result<Vec<OnlineExamQuestion>>;
    async fn get_question(&self, id: OnlineExamQuestionId) -> Result<Option<OnlineExamQuestion>>;
    async fn update_question(&self, q: &OnlineExamQuestion) -> Result<()>;
    async fn delete_question(&self, id: OnlineExamQuestionId) -> Result<()>;
    async fn insert_option(&self, o: &QuestionMuOption) -> Result<()>;
    async fn list_options(&self, question_id: OnlineExamQuestionId) -> Result<Vec<QuestionMuOption>>;
    async fn update_option(&self, o: &QuestionMuOption) -> Result<()>;
    async fn delete_option(&self, id: QuestionMuOptionId) -> Result<()>;
    async fn insert_assignment(&self, a: &QuestionAssignment) -> Result<()>;
    async fn list_assignments(&self, exam_id: OnlineExamId) -> Result<Vec<QuestionAssignment>>;
    async fn insert_mark(&self, m: &OnlineExamMark) -> Result<()>;
    async fn list_marks(&self, exam_id: OnlineExamId) -> Result<Vec<OnlineExamMark>>;
    async fn insert_answer(&self, a: &OnlineExamStudentAnswerMarking) -> Result<()>;
    async fn list_answers(
        &self,
        exam_id: OnlineExamId,
        student: StudentId,
    ) -> Result<Vec<OnlineExamStudentAnswerMarking>>;
    async fn insert_attempt(&self, a: &StudentTakeOnlineExam) -> Result<()>;
    async fn update_attempt(&self, a: &StudentTakeOnlineExam) -> Result<()>;
    async fn get_attempt(
        &self,
        exam_id: OnlineExamId,
        student: StudentId,
        record: StudentRecordId,
    ) -> Result<Option<StudentTakeOnlineExam>>;
    async fn list_attempts(&self, exam_id: OnlineExamId) -> Result<Vec<StudentTakeOnlineExam>>;
}
```

## QuestionBankRepository

```rust
#[async_trait]
pub trait QuestionBankRepository: Send + Sync {
    async fn get(&self, id: QuestionBankId) -> Result<Option<QuestionBank>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<QuestionBank>>;
    async fn list_by_group(&self, school: SchoolId, group: QuestionGroupId) -> Result<Vec<QuestionBank>>;
    async fn list_by_level(&self, school: SchoolId, level: QuestionLevelId) -> Result<Vec<QuestionBank>>;
    async fn insert(&self, q: &QuestionBank) -> Result<()>;
    async fn update(&self, q: &QuestionBank) -> Result<()>;
    async fn delete(&self, id: QuestionBankId) -> Result<()>;
    async fn list_groups(&self, school: SchoolId) -> Result<Vec<QuestionGroup>>;
    async fn list_levels(&self, school: SchoolId) -> Result<Vec<QuestionLevel>>;
    async fn insert_group(&self, g: &QuestionGroup) -> Result<()>;
    async fn insert_level(&self, l: &QuestionLevel) -> Result<()>;
}
```

## SeatPlanRepository

```rust
#[async_trait]
pub trait SeatPlanRepository: Send + Sync {
    async fn get(&self, id: SeatPlanId) -> Result<Option<SeatPlan>>;
    async fn find(
        &self,
        school: SchoolId,
        exam_type: ExamTypeId,
        class: ClassId,
        section: SectionId,
        year: AcademicYearId,
    ) -> Result<Option<SeatPlan>>;
    async fn list_for_exam_type(
        &self,
        school: SchoolId,
        exam_type: ExamTypeId,
    ) -> Result<Vec<SeatPlan>>;
    async fn insert(&self, s: &SeatPlan) -> Result<()>;
    async fn update(&self, s: &SeatPlan) -> Result<()>;
    async fn delete(&self, id: SeatPlanId) -> Result<()>;
    async fn insert_child(&self, c: &SeatPlanChild) -> Result<()>;
    async fn list_children(&self, plan_id: SeatPlanId) -> Result<Vec<SeatPlanChild>>;
    async fn get_setting(&self, school: SchoolId, year: AcademicYearId) -> Result<Option<SeatPlanSetting>>;
    async fn upsert_setting(&self, s: &SeatPlanSetting) -> Result<()>;
}
```

## AdmitCardRepository

```rust
#[async_trait]
pub trait AdmitCardRepository: Send + Sync {
    async fn get(&self, id: AdmitCardId) -> Result<Option<AdmitCard>>;
    async fn find(
        &self,
        school: SchoolId,
        student_record: StudentRecordId,
        exam_type: ExamTypeId,
        year: AcademicYearId,
    ) -> Result<Option<AdmitCard>>;
    async fn list_for_student(
        &self,
        school: SchoolId,
        student_record: StudentRecordId,
    ) -> Result<Vec<AdmitCard>>;
    async fn list_for_exam_type(
        &self,
        school: SchoolId,
        exam_type: ExamTypeId,
    ) -> Result<Vec<AdmitCard>>;
    async fn insert(&self, a: &AdmitCard) -> Result<()>;
    async fn update(&self, a: &AdmitCard) -> Result<()>;
    async fn delete(&self, id: AdmitCardId) -> Result<()>;
    async fn get_setting(
        &self,
        school: SchoolId,
        year: AcademicYearId,
    ) -> Result<Option<AdmitCardSetting>>;
    async fn upsert_setting(&self, s: &AdmitCardSetting) -> Result<()>;
}
```

## TeacherEvaluationRepository

```rust
#[async_trait]
pub trait TeacherEvaluationRepository: Send + Sync {
    async fn get(&self, id: TeacherEvaluationId) -> Result<Option<TeacherEvaluation>>;
    async fn find(
        &self,
        school: SchoolId,
        teacher: StaffId,
        subject: SubjectId,
        student: StudentId,
        record: StudentRecordId,
        year: AcademicYearId,
    ) -> Result<Option<TeacherEvaluation>>;
    async fn list_for_teacher(
        &self,
        school: SchoolId,
        teacher: StaffId,
        year: AcademicYearId,
    ) -> Result<Vec<TeacherEvaluation>>;
    async fn list_for_student(
        &self,
        school: SchoolId,
        student: StudentId,
    ) -> Result<Vec<TeacherEvaluation>>;
    async fn insert(&self, e: &TeacherEvaluation) -> Result<()>;
    async fn update(&self, e: &TeacherEvaluation) -> Result<()>;
    async fn get_setting(&self, school: SchoolId) -> Result<Option<TeacherEvaluationSetting>>;
    async fn upsert_setting(&self, s: &TeacherEvaluationSetting) -> Result<()>;
}
```

## TeacherRemarkRepository

```rust
#[async_trait]
pub trait TeacherRemarkRepository: Send + Sync {
    async fn get(&self, id: TeacherRemarkId) -> Result<Option<TeacherRemark>>;
    async fn find(
        &self,
        school: SchoolId,
        student: StudentId,
        exam_type: ExamTypeId,
        year: AcademicYearId,
    ) -> Result<Option<TeacherRemark>>;
    async fn list_for_student(&self, school: SchoolId, student: StudentId) -> Result<Vec<TeacherRemark>>;
    async fn list_for_teacher(&self, school: SchoolId, teacher: StaffId) -> Result<Vec<TeacherRemark>>;
    async fn insert(&self, r: &TeacherRemark) -> Result<()>;
    async fn update(&self, r: &TeacherRemark) -> Result<()>;
    async fn delete(&self, id: TeacherRemarkId) -> Result<()>;
}
```

## ExamAttendanceRepository

```rust
#[async_trait]
pub trait ExamAttendanceRepository: Send + Sync {
    async fn get(&self, id: ExamAttendanceId) -> Result<Option<ExamAttendance>>;
    async fn find(
        &self,
        school: SchoolId,
        exam: ExamId,
        subject: SubjectId,
        class: ClassId,
        section: SectionId,
        year: AcademicYearId,
    ) -> Result<Option<ExamAttendance>>;
    async fn list_for_exam(&self, school: SchoolId, exam: ExamId) -> Result<Vec<ExamAttendance>>;
    async fn insert(&self, a: &ExamAttendance) -> Result<()>;
    async fn update(&self, a: &ExamAttendance) -> Result<()>;
    async fn upsert_child(&self, c: &ExamAttendanceChild) -> Result<()>;
    async fn list_children(&self, attendance_id: ExamAttendanceId) -> Result<Vec<ExamAttendanceChild>>;
}
```

## ResultSettingRepository

```rust
#[async_trait]
pub trait ResultSettingRepository: Send + Sync {
    async fn get(&self, school: SchoolId, year: AcademicYearId) -> Result<Option<ResultSetting>>;
    async fn upsert(&self, s: &ResultSetting) -> Result<()>;
    async fn get_custom(
        &self,
        school: SchoolId,
        exam_type: ExamTypeId,
        year: AcademicYearId,
    ) -> Result<Option<CustomResultSetting>>;
    async fn list_custom(&self, school: SchoolId) -> Result<Vec<CustomResultSetting>>;
    async fn upsert_custom(&self, c: &CustomResultSetting) -> Result<()>;
}
```

## ExamSettingRepository

```rust
#[async_trait]
pub trait ExamSettingRepository: Send + Sync {
    async fn get(&self, id: ExamSettingId) -> Result<Option<ExamSetting>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<ExamSetting>>;
    async fn list_for_type(
        &self,
        school: SchoolId,
        exam_type: ExamTerm,
    ) -> Result<Vec<ExamSetting>>;
    async fn insert(&self, s: &ExamSetting) -> Result<()>;
    async fn update(&self, s: &ExamSetting) -> Result<()>;
    async fn delete(&self, id: ExamSettingId) -> Result<()>;
    async fn get_signature(
        &self,
        school: SchoolId,
        title: &str,
    ) -> Result<Option<ExamSignature>>;
    async fn list_signatures(&self, school: SchoolId) -> Result<Vec<ExamSignature>>;
    async fn upsert_signature(&self, s: &ExamSignature) -> Result<()>;
    async fn delete_signature(&self, id: ExamSignatureId) -> Result<()>;
}
```

## Indexes (recommended)

The default PostgreSQL adapter documents the following indexes;
consumers should declare them in their migrations:

```sql
CREATE INDEX ix_assessment_exams_school_id_year_type
    ON assessment_exams (school_id, academic_id, exam_type_id);
CREATE UNIQUE INDEX ux_assessment_exams_school_id_unique
    ON assessment_exams (school_id, academic_id, exam_type_id, class_id, section_id, subject_id, parent_id);

CREATE INDEX ix_assessment_exam_schedules_school_id_class_section
    ON assessment_exam_schedules (school_id, academic_id, class_id, section_id);
CREATE INDEX ix_assessment_exam_schedules_school_id_teacher_date
    ON assessment_exam_schedules (school_id, academic_id, teacher_id, date);
CREATE INDEX ix_assessment_exam_schedules_school_id_room_date
    ON assessment_exam_schedules (school_id, academic_id, room_id, date);

CREATE INDEX ix_assessment_marks_registers_school_id_exam
    ON assessment_marks_registers (school_id, academic_id, exam_id);
CREATE UNIQUE INDEX ux_assessment_marks_registers_school_id_exam_student
    ON assessment_marks_registers (school_id, academic_id, exam_id, student_id);
CREATE INDEX ix_assessment_marks_register_children_school_id_register
    ON assessment_marks_register_children (school_id, academic_id, marks_register_id);

CREATE INDEX ix_assessment_mark_stores_school_id_setup_student
    ON assessment_mark_stores (school_id, academic_id, exam_setup_id, student_id);
CREATE INDEX ix_assessment_result_stores_school_id_exam_setup_student
    ON assessment_result_stores (school_id, academic_id, exam_setup_id, student_id);
CREATE INDEX ix_assessment_result_stores_school_id_exam_type_class_section
    ON assessment_result_stores (school_id, academic_id, exam_type_id, class_id, section_id);
CREATE INDEX ix_exam_merit_positions_school_id_section_term
    ON exam_merit_positions (school_id, academic_id, class_id, section_id, exam_term_id);
CREATE INDEX ix_all_exam_wise_positions_school_id_class
    ON all_exam_wise_positions (school_id, academic_id, class_id);

CREATE INDEX ix_assessment_marks_grades_school_id
    ON assessment_marks_grade_rules (school_id, academic_id);

CREATE INDEX ix_assessment_online_exams_school_id_class_section
    ON assessment_online_exams (school_id, academic_id, class_id, section_id);
CREATE INDEX ix_assessment_online_exams_school_id_status_date
    ON assessment_online_exams (school_id, status, date);
CREATE INDEX ix_assessment_online_exam_questions_school_id_exam
    ON assessment_online_exam_questions (school_id, online_exam_id);
CREATE INDEX ix_assessment_student_take_online_exams_school_id_exam_student
    ON assessment_student_take_online_exams (school_id, online_exam_id, student_id);
CREATE INDEX ix_assessment_student_take_online_exam_questions_school_id_take
    ON assessment_student_take_online_exam_questions (school_id, take_online_exam_id);

CREATE INDEX ix_assessment_seat_plans_school_id_type_class_section
    ON assessment_seat_plans (school_id, academic_id, exam_id, class_id, section_id);
CREATE UNIQUE INDEX ux_assessment_seat_plans_school_id_unique
    ON assessment_seat_plans (school_id, academic_id, exam_id, class_id, section_id);

CREATE INDEX ix_admit_cards_school_id_student_record_type
    ON admit_cards (school_id, academic_id, student_record_id, exam_type_id);
CREATE UNIQUE INDEX ux_admit_cards_school_id_unique
    ON admit_cards (school_id, academic_id, student_record_id, exam_type_id);

CREATE INDEX ix_teacher_evaluations_school_id_teacher_subject
    ON teacher_evaluations (school_id, teacher_id, subject_id, academic_id);
CREATE UNIQUE INDEX ux_teacher_remarks_school_id_student_exam
    ON teacher_remarks (school_id, student_id, exam_type_id, academic_id);

CREATE INDEX ix_assessment_exam_attendances_school_id_exam_subject_section
    ON assessment_exam_attendances (school_id, academic_id, exam_id, subject_id, class_id, section_id);
CREATE INDEX ix_assessment_exam_attendance_children_school_id_attendance
    ON assessment_exam_attendance_children (school_id, exam_attendance_id);

CREATE INDEX ix_custom_result_settings_school_id_exam_type
    ON custom_result_settings (school_id, academic_id, exam_type_id);
CREATE INDEX ix_assessment_exam_signatures_school_id
    ON assessment_exam_signatures (school_id, academic_id);
CREATE INDEX ix_assessment_exam_settings_school_id_year
    ON assessment_exam_settings (school_id, academic_id);
```

The `school_id` predicate is mandatory for tenant isolation.
