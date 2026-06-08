# Academic Domain — Repositories

Repositories are ports (Rust traits). Adapters implement them. The
default adapter targets PostgreSQL; an SQLite adapter is provided for
embedded deployments.

## StudentRepository

```rust
#[async_trait]
pub trait StudentRepository: Send + Sync {
    async fn get(&self, id: StudentId) -> Result<Option<Student>>;
    async fn get_by_admission_no(&self, school: SchoolId, admission_no: &AdmissionNumber) -> Result<Option<Student>>;
    async fn insert(&self, student: &Student) -> Result<()>;
    async fn update(&self, student: &Student) -> Result<()>;
    async fn delete(&self, id: StudentId) -> Result<()>;

    async fn query(&self, q: StudentQuery) -> Result<Vec<Student>>;
    async fn count(&self, q: StudentQuery) -> Result<u64>;
    async fn page(&self, q: StudentQuery, offset: u32, limit: u32) -> Result<Page<Student>>;

    // Optimized domain queries
    async fn active_in_class(&self, school: SchoolId, class: ClassId, year: AcademicYearId) -> Result<Vec<Student>>;
    async fn active_in_section(&self, school: SchoolId, class: ClassId, section: SectionId, year: AcademicYearId) -> Result<Vec<Student>>;
    async fn admitted_in_range(&self, school: SchoolId, from: NaiveDate, to: NaiveDate) -> Result<Vec<Student>>;
    async fn suspended(&self, school: SchoolId) -> Result<Vec<Student>>;
    async fn search_by_name(&self, school: SchoolId, query: &str, limit: u32) -> Result<Vec<Student>>;
}
```

## GuardianRepository

```rust
#[async_trait]
pub trait GuardianRepository: Send + Sync {
    async fn get(&self, id: GuardianId) -> Result<Option<Guardian>>;
    async fn list_for_student(&self, student: StudentId) -> Result<Vec<Guardian>>;
    async fn primary_for_student(&self, student: StudentId) -> Result<Option<Guardian>>;
    async fn insert(&self, guardian: &Guardian) -> Result<()>;
    async fn update(&self, guardian: &Guardian) -> Result<()>;
    async fn link(&self, link: &GuardianLink) -> Result<()>;
    async fn unlink(&self, link: &GuardianLink) -> Result<()>;
    async fn find_by_phone(&self, school: SchoolId, phone: &PhoneNumber) -> Result<Vec<Guardian>>;
    async fn find_by_email(&self, school: SchoolId, email: &EmailAddress) -> Result<Vec<Guardian>>;
}
```

## ClassRepository

```rust
#[async_trait]
pub trait ClassRepository: Send + Sync {
    async fn get(&self, id: ClassId) -> Result<Option<Class>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<Class>>;
    async fn insert(&self, class: &Class) -> Result<()>;
    async fn update(&self, class: &Class) -> Result<()>;
    async fn delete(&self, id: ClassId) -> Result<()>;
}
```

## SectionRepository

```rust
#[async_trait]
pub trait SectionRepository: Send + Sync {
    async fn get(&self, id: SectionId) -> Result<Option<Section>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<Section>>;
    async fn insert(&self, section: &Section) -> Result<()>;
    async fn update(&self, section: &Section) -> Result<()>;
    async fn delete(&self, id: SectionId) -> Result<()>;
}
```

## ClassSectionRepository

```rust
#[async_trait]
pub trait ClassSectionRepository: Send + Sync {
    async fn get(&self, id: ClassSectionId) -> Result<Option<ClassSection>>;
    async fn find(&self, class: ClassId, section: SectionId, year: AcademicYearId) -> Result<Option<ClassSection>>;
    async fn list(&self, school: SchoolId, year: AcademicYearId) -> Result<Vec<ClassSection>>;
    async fn insert(&self, cs: &ClassSection) -> Result<()>;
    async fn update(&self, cs: &ClassSection) -> Result<()>;
    async fn delete(&self, id: ClassSectionId) -> Result<()>;
}
```

## SubjectRepository

```rust
#[async_trait]
pub trait SubjectRepository: Send + Sync {
    async fn get(&self, id: SubjectId) -> Result<Option<Subject>>;
    async fn find_by_code(&self, school: SchoolId, code: &SubjectCode) -> Result<Option<Subject>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<Subject>>;
    async fn insert(&self, subject: &Subject) -> Result<()>;
    async fn update(&self, subject: &Subject) -> Result<()>;
    async fn delete(&self, id: SubjectId) -> Result<()>;
}
```

## ClassSubjectRepository

```rust
#[async_trait]
pub trait ClassSubjectRepository: Send + Sync {
    async fn get(&self, id: ClassSubjectId) -> Result<Option<ClassSubject>>;
    async fn list_for_class(&self, class: ClassId, year: AcademicYearId) -> Result<Vec<ClassSubject>>;
    async fn list_for_section(&self, class: ClassId, section: SectionId, year: AcademicYearId) -> Result<Vec<ClassSubject>>;
    async fn list_for_teacher(&self, teacher: StaffId, year: AcademicYearId) -> Result<Vec<ClassSubject>>;
    async fn insert(&self, cs: &ClassSubject) -> Result<()>;
    async fn update(&self, cs: &ClassSubject) -> Result<()>;
    async fn delete(&self, id: ClassSubjectId) -> Result<()>;
}
```

## AcademicYearRepository

```rust
#[async_trait]
pub trait AcademicYearRepository: Send + Sync {
    async fn get(&self, id: AcademicYearId) -> Result<Option<AcademicYear>>;
    async fn current(&self, school: SchoolId) -> Result<Option<AcademicYear>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<AcademicYear>>;
    async fn insert(&self, year: &AcademicYear) -> Result<()>;
    async fn update(&self, year: &AcademicYear) -> Result<()>;
    async fn close(&self, id: AcademicYearId) -> Result<()>;
}
```

## ClassRoutineRepository

```rust
#[async_trait]
pub trait ClassRoutineRepository: Send + Sync {
    async fn get(&self, id: ClassRoutineId) -> Result<Option<ClassRoutine>>;
    async fn find(&self, class: ClassId, section: SectionId, subject: SubjectId, year: AcademicYearId) -> Result<Option<ClassRoutine>>;
    async fn list_for_section(&self, class: ClassId, section: SectionId, year: AcademicYearId) -> Result<Vec<ClassRoutine>>;
    async fn list_for_teacher(&self, teacher: StaffId, year: AcademicYearId) -> Result<Vec<ClassRoutine>>;
    async fn list_for_room(&self, room: ClassRoomId, year: AcademicYearId) -> Result<Vec<ClassRoutine>>;
    async fn insert(&self, routine: &ClassRoutine) -> Result<()>;
    async fn update(&self, routine: &ClassRoutine) -> Result<()>;
    async fn delete(&self, id: ClassRoutineId) -> Result<()>;
}
```

## HomeworkRepository

```rust
#[async_trait]
pub trait HomeworkRepository: Send + Sync {
    async fn get(&self, id: HomeworkId) -> Result<Option<Homework>>;
    async fn list_for_class(&self, class: ClassId, section: SectionId, year: AcademicYearId) -> Result<Vec<Homework>>;
    async fn list_for_student(&self, student: StudentId) -> Result<Vec<Homework>>;
    async fn list_due_on(&self, school: SchoolId, date: NaiveDate) -> Result<Vec<Homework>>;
    async fn insert(&self, h: &Homework) -> Result<()>;
    async fn update(&self, h: &Homework) -> Result<()>;
    async fn delete(&self, id: HomeworkId) -> Result<()>;
    async fn submit(&self, sub: &HomeworkSubmission) -> Result<()>;
    async fn evaluate(&self, sub: &HomeworkSubmission) -> Result<()>;
    async fn submission(&self, homework: HomeworkId, student: StudentId) -> Result<Option<HomeworkSubmission>>;
}
```

## LessonRepository

```rust
#[async_trait]
pub trait LessonRepository: Send + Sync {
    async fn get(&self, id: LessonId) -> Result<Option<Lesson>>;
    async fn list_for_class_subject(&self, class: ClassId, section: SectionId, subject: SubjectId) -> Result<Vec<Lesson>>;
    async fn insert(&self, l: &Lesson) -> Result<()>;
    async fn update(&self, l: &Lesson) -> Result<()>;
    async fn delete(&self, id: LessonId) -> Result<()>;
}
```

## LessonTopicRepository

```rust
#[async_trait]
pub trait LessonTopicRepository: Send + Sync {
    async fn get(&self, id: LessonTopicId) -> Result<Option<LessonTopic>>;
    async fn list_for_lesson(&self, lesson: LessonId) -> Result<Vec<LessonTopic>>;
    async fn insert(&self, t: &LessonTopic) -> Result<()>;
    async fn update(&self, t: &LessonTopic) -> Result<()>;
    async fn delete(&self, id: LessonTopicId) -> Result<()>;
}
```

## LessonPlanRepository

```rust
#[async_trait]
pub trait LessonPlanRepository: Send + Sync {
    async fn get(&self, id: LessonPlanId) -> Result<Option<LessonPlan>>;
    async fn list_for_class_subject(&self, class: ClassId, section: SectionId, subject: SubjectId, year: AcademicYearId) -> Result<Vec<LessonPlan>>;
    async fn list_for_teacher(&self, teacher: StaffId, year: AcademicYearId) -> Result<Vec<LessonPlan>>;
    async fn list_for_date(&self, school: SchoolId, date: NaiveDate) -> Result<Vec<LessonPlan>>;
    async fn insert(&self, p: &LessonPlan) -> Result<()>;
    async fn update(&self, p: &LessonPlan) -> Result<()>;
    async fn delete(&self, id: LessonPlanId) -> Result<()>;
}
```

## StudentRecordRepository

```rust
#[async_trait]
pub trait StudentRecordRepository: Send + Sync {
    async fn get(&self, id: StudentRecordId) -> Result<Option<StudentRecord>>;
    async fn default_for_student(&self, student: StudentId) -> Result<Option<StudentRecord>>;
    async fn list_for_student(&self, student: StudentId) -> Result<Vec<StudentRecord>>;
    async fn list_for_section(&self, class: ClassId, section: SectionId, year: AcademicYearId) -> Result<Vec<StudentRecord>>;
    async fn insert(&self, r: &StudentRecord) -> Result<()>;
    async fn update(&self, r: &StudentRecord) -> Result<()>;
}
```

## StudentPromotionRepository

```rust
#[async_trait]
pub trait StudentPromotionRepository: Send + Sync {
    async fn get(&self, id: StudentPromotionId) -> Result<Option<StudentPromotion>>;
    async fn list_for_student(&self, student: StudentId) -> Result<Vec<StudentPromotion>>;
    async fn insert(&self, p: &StudentPromotion) -> Result<()>;
}
```

## StudentCategoryRepository, StudentGroupRepository, RegistrationFieldRepository, CertificateRepository, IdCardRepository, AdmissionQueryRepository, ClassRoomRepository, ClassTimeRepository

Each follows the same pattern: `get`, `list`, `insert`, `update`, `delete`,
plus domain-specific queries.

## Indexes (recommended)

The default PostgreSQL adapter documents the following indexes; consumers
should declare them in their migrations:

```sql
CREATE INDEX ix_students_school_id_admission_no ON students (school_id, admission_no);
CREATE UNIQUE INDEX ux_students_school_id_admission_no ON students (school_id, admission_no) WHERE admission_no IS NOT NULL;
CREATE INDEX ix_students_school_id_class_id_section_id ON students (school_id, class_id, section_id);
CREATE INDEX ix_student_records_school_id_class_section_year ON student_records (school_id, class_id, section_id, session_id);
CREATE UNIQUE INDEX ux_student_records_school_id_class_section_roll ON student_records (school_id, class_id, section_id, session_id, roll_no);
CREATE INDEX ix_class_sections_school_id_class_id_year ON class_sections (school_id, class_id, academic_id);
CREATE INDEX ix_assign_subjects_school_id_teacher_year ON assign_subjects (school_id, teacher_id, academic_id);
CREATE INDEX ix_class_routine_updates_school_id_teacher_day ON class_routine_updates (school_id, teacher_id, day, academic_id);
CREATE INDEX ix_class_routine_updates_school_id_room_day ON class_routine_updates (school_id, room_id, day, academic_id);
CREATE INDEX ix_homeworks_school_id_class_section_due ON homeworks (school_id, class_id, section_id, submission_date);
CREATE INDEX ix_lesson_planners_school_id_teacher_date ON lesson_planners (school_id, teacher_id, lesson_date);
CREATE INDEX ix_academic_years_school_id_current ON academic_years (school_id) WHERE active_status = 1;
```

The `school_id` predicate is mandatory for tenant isolation.
