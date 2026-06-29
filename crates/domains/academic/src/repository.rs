//! # Academic-domain repository port traits
//!
//! Per `docs/ports/storage.md` and the engine's tier rules,
//! the `domains` tier does not depend on the `adapters`
//! tier; the per-aggregate repository traits are ports the
//! storage adapter crates implement.
//!
//! Each read method takes a [`TenantContext`](educore_core::tenant::TenantContext)
//! and filters by `ctx.school_id` so the adapter cannot
//! accidentally surface a cross-tenant row. The global
//! reads (e.g. `list_for_school`) intentionally take the
//! school id directly and are gated behind the
//! academic-admin capability at the dispatcher; the
//! storage adapter still enforces the read.
//!
//! Phase 3 ships the 5 prompt-named repository ports:
//! [`StudentRepository`], [`ClassRepository`],
//! [`SectionRepository`], [`SubjectRepository`],
//! [`AcademicYearRepository`], plus the mop-up
//! [`StudentRecordRepository`] (added in wave 9.2a)
//! and the 14 placeholder-port traits added in wave 9.2b:
//! [`GuardianRepository`], [`ClassSectionRepository`],
//! [`ClassSubjectRepository`], [`ClassRoutineRepository`],
//! [`HomeworkRepository`], [`LessonPlanRepository`],
//! [`LessonRepository`], [`LessonTopicRepository`],
//! [`StudentPromotionRepository`], [`StudentCategoryRepository`],
//! [`StudentGroupRepository`], [`RegistrationFieldRepository`],
//! [`CertificateRepository`], [`IdCardRepository`].

use async_trait::async_trait;

use educore_core::error::Result;
use educore_core::ids::SchoolId;
use educore_core::tenant::TenantContext;

use crate::aggregate::{
    AcademicYear, Certificate, Class, ClassRoutine, ClassSection, ClassSubject, Guardian, Homework,
    IdCard, Lesson, LessonPlan, LessonTopic, RegistrationField, Section, Student, StudentCategory,
    StudentGroup, StudentPromotion, StudentRecord, Subject,
};
use crate::value_objects::{
    AcademicYearId, CertificateId, ClassId, ClassRoutineId, ClassSectionId, ClassSubjectId,
    GuardianId, HomeworkId, IdCardId, LessonId, LessonPlanId, LessonTopicId, RegistrationFieldId,
    SectionId, StudentCategoryId, StudentGroupId, StudentId, StudentPromotionId, StudentRecordId,
    StudentStatus, SubjectId,
};

// =============================================================================
// StudentRepository
// =============================================================================

/// Repository port for [`Student`] aggregates.
///
/// The trait is `Send + Sync` so consumers can hold an
/// `Arc<dyn StudentRepository>` in a multi-threaded runtime.
#[async_trait]
pub trait StudentRepository: Send + Sync {
    /// Fetches the student with `id` (scoped to `ctx.school_id`).
    /// Returns `Ok(None)` if the student does not exist in the
    /// active tenant.
    async fn get(&self, ctx: &TenantContext, id: StudentId) -> Result<Option<Student>>;

    /// Fetches the student in `school` whose admission number
    /// matches.
    async fn get_by_admission_no(
        &self,
        school: SchoolId,
        admission_no: &str,
    ) -> Result<Option<Student>>;

    /// Fetches the student in `school` whose email matches
    /// (case-insensitive).
    async fn get_by_email(&self, school: SchoolId, email: &str) -> Result<Option<Student>>;

    /// Lists students in the active tenant. Paginated by
    /// `offset` and `limit`.
    async fn list(&self, ctx: &TenantContext, offset: u32, limit: u32) -> Result<Vec<Student>>;

    /// Lists students in the active tenant with `status =
    /// status`.
    async fn list_by_status(
        &self,
        ctx: &TenantContext,
        status: StudentStatus,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<Student>>;

    /// Lists students in the active tenant admitted into the
    /// given `(class_id, section_id, academic_year_id)`.
    async fn list_in_class_section(
        &self,
        ctx: &TenantContext,
        class_id: ClassId,
        section_id: SectionId,
        academic_year_id: AcademicYearId,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<Student>>;

    /// Inserts a new student row. Returns `Err(Conflict)` if a
    /// row with the same `(school_id, id)` already exists.
    async fn insert(&self, ctx: &TenantContext, student: &Student) -> Result<()>;

    /// Updates an existing student row. Returns
    /// `Err(Conflict)` on optimistic-concurrency mismatch
    /// (the row's `version` does not match `student.version`).
    async fn update(&self, ctx: &TenantContext, student: &Student) -> Result<()>;
}

// =============================================================================
// ClassRepository
// =============================================================================

/// Repository port for [`Class`] aggregates.
#[async_trait]
pub trait ClassRepository: Send + Sync {
    /// Fetches the class with `id` (scoped to `ctx.school_id`).
    async fn get(&self, ctx: &TenantContext, id: ClassId) -> Result<Option<Class>>;

    /// Fetches the class in `school` whose name matches.
    async fn get_by_name(&self, school: SchoolId, name: &str) -> Result<Option<Class>>;

    /// Lists classes in the active tenant. Paginated.
    async fn list(&self, ctx: &TenantContext, offset: u32, limit: u32) -> Result<Vec<Class>>;

    /// Inserts a new class row.
    async fn insert(&self, ctx: &TenantContext, class: &Class) -> Result<()>;

    /// Updates an existing class row.
    async fn update(&self, ctx: &TenantContext, class: &Class) -> Result<()>;
}

// =============================================================================
// SectionRepository
// =============================================================================

/// Repository port for [`Section`] aggregates.
#[async_trait]
pub trait SectionRepository: Send + Sync {
    /// Fetches the section with `id` (scoped to `ctx.school_id`).
    async fn get(&self, ctx: &TenantContext, id: SectionId) -> Result<Option<Section>>;

    /// Fetches the section in `school` whose name matches.
    async fn get_by_name(&self, school: SchoolId, name: &str) -> Result<Option<Section>>;

    /// Lists sections in the active tenant. Paginated.
    async fn list(&self, ctx: &TenantContext, offset: u32, limit: u32) -> Result<Vec<Section>>;

    /// Inserts a new section row.
    async fn insert(&self, ctx: &TenantContext, section: &Section) -> Result<()>;

    /// Updates an existing section row.
    async fn update(&self, ctx: &TenantContext, section: &Section) -> Result<()>;
}

// =============================================================================
// SubjectRepository
// =============================================================================

/// Repository port for [`Subject`] aggregates.
#[async_trait]
pub trait SubjectRepository: Send + Sync {
    /// Fetches the subject with `id` (scoped to `ctx.school_id`).
    async fn get(&self, ctx: &TenantContext, id: SubjectId) -> Result<Option<Subject>>;

    /// Fetches the subject in `school` whose code matches.
    async fn get_by_code(&self, school: SchoolId, code: &str) -> Result<Option<Subject>>;

    /// Lists subjects in the active tenant. Paginated.
    async fn list(&self, ctx: &TenantContext, offset: u32, limit: u32) -> Result<Vec<Subject>>;

    /// Inserts a new subject row.
    async fn insert(&self, ctx: &TenantContext, subject: &Subject) -> Result<()>;

    /// Updates an existing subject row.
    async fn update(&self, ctx: &TenantContext, subject: &Subject) -> Result<()>;
}

// =============================================================================
// AcademicYearRepository
// =============================================================================

/// Repository port for [`AcademicYear`] aggregates.
#[async_trait]
pub trait AcademicYearRepository: Send + Sync {
    /// Fetches the academic year with `id` (scoped to
    /// `ctx.school_id`).
    async fn get(&self, ctx: &TenantContext, id: AcademicYearId) -> Result<Option<AcademicYear>>;

    /// Fetches the current academic year for the active
    /// tenant. Returns `Ok(None)` if no row is marked
    /// `is_current = true`.
    async fn current(&self, ctx: &TenantContext) -> Result<Option<AcademicYear>>;

    /// Lists academic years in the active tenant. Paginated.
    async fn list(&self, ctx: &TenantContext, offset: u32, limit: u32)
        -> Result<Vec<AcademicYear>>;

    /// Inserts a new academic year row.
    async fn insert(&self, ctx: &TenantContext, year: &AcademicYear) -> Result<()>;

    /// Updates an existing academic year row.
    async fn update(&self, ctx: &TenantContext, year: &AcademicYear) -> Result<()>;
}

// =============================================================================
// StudentRecordRepository
// =============================================================================

/// Repository port for the [`StudentRecord`] aggregate.
///
/// Minimal CRUD interface; extend with queries as needed.
#[async_trait]
pub trait StudentRecordRepository: Send + Sync {
    /// Fetch a student record by its typed id. Returns `Ok(None)`
    /// if the row does not exist or is soft-deleted.
    async fn get(&self, ctx: &TenantContext, id: StudentRecordId) -> Result<Option<StudentRecord>>;

    /// Insert a new student record (or upsert on a soft-delete update).
    async fn insert(&self, ctx: &TenantContext, record: &StudentRecord) -> Result<()>;

    /// Update an existing student record.
    async fn update(&self, ctx: &TenantContext, record: &StudentRecord) -> Result<()>;
}

// =============================================================================
// Wave 9.2b: 14 placeholder repository ports.
//
// Each trait mirrors the minimal CRUD shape of
// [`StudentRecordRepository`]: `get`, `insert`, `update`.
// The aggregate structs are themselves minimal `id +
// school_id` placeholders (see `aggregate.rs`'s
// `academic_aggregate_stub!` block) — the full
// implementation (audit footer, domain fields,
// invariants, services, events) lands in subsequent
// workstreams per `docs/build-plan.md`. These trait
// definitions close the 14/132 PORT-STORAGE-REPOS
// items so the storage adapter scaffolding can
// progress against the complete per-aggregate surface.
// =============================================================================

// === Certificate repository section begin ===

/// Repository port for the [`Certificate`](crate::aggregate::Certificate)
/// aggregate. Minimal CRUD interface; extend with queries as needed.
#[async_trait]
pub trait CertificateRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(&self, ctx: &TenantContext, id: CertificateId) -> Result<Option<Certificate>>;
    /// Insert a new certificate.
    async fn insert(&self, ctx: &TenantContext, c: &Certificate) -> Result<()>;
    /// Update an existing certificate.
    async fn update(&self, ctx: &TenantContext, c: &Certificate) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_certificate_object_safe() {
    fn _f(_: Box<dyn CertificateRepository>) {}
}

// === Certificate repository section end ===

// === ClassRoutine repository section begin ===

/// Repository port for the [`ClassRoutine`](crate::aggregate::ClassRoutine)
/// aggregate. Minimal CRUD interface; extend with queries as needed.
#[async_trait]
pub trait ClassRoutineRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(&self, ctx: &TenantContext, id: ClassRoutineId) -> Result<Option<ClassRoutine>>;
    /// Insert a new class routine.
    async fn insert(&self, ctx: &TenantContext, r: &ClassRoutine) -> Result<()>;
    /// Update an existing class routine.
    async fn update(&self, ctx: &TenantContext, r: &ClassRoutine) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_class_routine_object_safe() {
    fn _f(_: Box<dyn ClassRoutineRepository>) {}
}

// === ClassRoutine repository section end ===

// === ClassSection repository section begin ===

/// Repository port for the [`ClassSection`](crate::aggregate::ClassSection)
/// aggregate. Minimal CRUD interface; extend with queries as needed.
#[async_trait]
pub trait ClassSectionRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(&self, ctx: &TenantContext, id: ClassSectionId) -> Result<Option<ClassSection>>;
    /// Insert a new class section.
    async fn insert(&self, ctx: &TenantContext, s: &ClassSection) -> Result<()>;
    /// Update an existing class section.
    async fn update(&self, ctx: &TenantContext, s: &ClassSection) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_class_section_object_safe() {
    fn _f(_: Box<dyn ClassSectionRepository>) {}
}

// === ClassSection repository section end ===

// === ClassSubject repository section begin ===

/// Repository port for the [`ClassSubject`](crate::aggregate::ClassSubject)
/// aggregate. Minimal CRUD interface; extend with queries as needed.
#[async_trait]
pub trait ClassSubjectRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(&self, ctx: &TenantContext, id: ClassSubjectId) -> Result<Option<ClassSubject>>;
    /// Insert a new class subject.
    async fn insert(&self, ctx: &TenantContext, s: &ClassSubject) -> Result<()>;
    /// Update an existing class subject.
    async fn update(&self, ctx: &TenantContext, s: &ClassSubject) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_class_subject_object_safe() {
    fn _f(_: Box<dyn ClassSubjectRepository>) {}
}

// === ClassSubject repository section end ===

// === Guardian repository section begin ===

/// Repository port for the [`Guardian`](crate::aggregate::Guardian)
/// aggregate. Minimal CRUD interface; extend with queries as needed.
#[async_trait]
pub trait GuardianRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(&self, ctx: &TenantContext, id: GuardianId) -> Result<Option<Guardian>>;
    /// Insert a new guardian.
    async fn insert(&self, ctx: &TenantContext, g: &Guardian) -> Result<()>;
    /// Update an existing guardian.
    async fn update(&self, ctx: &TenantContext, g: &Guardian) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_guardian_object_safe() {
    fn _f(_: Box<dyn GuardianRepository>) {}
}

// === Guardian repository section end ===

// === Homework repository section begin ===

/// Repository port for the [`Homework`](crate::aggregate::Homework)
/// aggregate. Minimal CRUD interface; extend with queries as needed.
#[async_trait]
pub trait HomeworkRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(&self, ctx: &TenantContext, id: HomeworkId) -> Result<Option<Homework>>;
    /// Insert a new homework.
    async fn insert(&self, ctx: &TenantContext, h: &Homework) -> Result<()>;
    /// Update an existing homework.
    async fn update(&self, ctx: &TenantContext, h: &Homework) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_homework_object_safe() {
    fn _f(_: Box<dyn HomeworkRepository>) {}
}

// === Homework repository section end ===

// === IdCard repository section begin ===

/// Repository port for the [`IdCard`](crate::aggregate::IdCard)
/// aggregate. Minimal CRUD interface; extend with queries as needed.
#[async_trait]
pub trait IdCardRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(&self, ctx: &TenantContext, id: IdCardId) -> Result<Option<IdCard>>;
    /// Insert a new id card template.
    async fn insert(&self, ctx: &TenantContext, c: &IdCard) -> Result<()>;
    /// Update an existing id card template.
    async fn update(&self, ctx: &TenantContext, c: &IdCard) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_id_card_object_safe() {
    fn _f(_: Box<dyn IdCardRepository>) {}
}

// === IdCard repository section end ===

// === Lesson repository section begin ===

/// Repository port for the [`Lesson`](crate::aggregate::Lesson)
/// aggregate. Minimal CRUD interface; extend with queries as needed.
#[async_trait]
pub trait LessonRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(&self, ctx: &TenantContext, id: LessonId) -> Result<Option<Lesson>>;
    /// Insert a new lesson.
    async fn insert(&self, ctx: &TenantContext, l: &Lesson) -> Result<()>;
    /// Update an existing lesson.
    async fn update(&self, ctx: &TenantContext, l: &Lesson) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_lesson_object_safe() {
    fn _f(_: Box<dyn LessonRepository>) {}
}

// === Lesson repository section end ===

// === LessonPlan repository section begin ===

/// Repository port for the [`LessonPlan`](crate::aggregate::LessonPlan)
/// aggregate. Minimal CRUD interface; extend with queries as needed.
#[async_trait]
pub trait LessonPlanRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(&self, ctx: &TenantContext, id: LessonPlanId) -> Result<Option<LessonPlan>>;
    /// Insert a new lesson plan.
    async fn insert(&self, ctx: &TenantContext, p: &LessonPlan) -> Result<()>;
    /// Update an existing lesson plan.
    async fn update(&self, ctx: &TenantContext, p: &LessonPlan) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_lesson_plan_object_safe() {
    fn _f(_: Box<dyn LessonPlanRepository>) {}
}

// === LessonPlan repository section end ===

// === LessonTopic repository section begin ===

/// Repository port for the [`LessonTopic`](crate::aggregate::LessonTopic)
/// aggregate. Minimal CRUD interface; extend with queries as needed.
#[async_trait]
pub trait LessonTopicRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(&self, ctx: &TenantContext, id: LessonTopicId) -> Result<Option<LessonTopic>>;
    /// Insert a new lesson topic.
    async fn insert(&self, ctx: &TenantContext, t: &LessonTopic) -> Result<()>;
    /// Update an existing lesson topic.
    async fn update(&self, ctx: &TenantContext, t: &LessonTopic) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_lesson_topic_object_safe() {
    fn _f(_: Box<dyn LessonTopicRepository>) {}
}

// === LessonTopic repository section end ===

// === RegistrationField repository section begin ===

/// Repository port for the
/// [`RegistrationField`](crate::aggregate::RegistrationField) aggregate.
/// Minimal CRUD interface; extend with queries as needed.
#[async_trait]
pub trait RegistrationFieldRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: RegistrationFieldId,
    ) -> Result<Option<RegistrationField>>;
    /// Insert a new registration field.
    async fn insert(&self, ctx: &TenantContext, f: &RegistrationField) -> Result<()>;
    /// Update an existing registration field.
    async fn update(&self, ctx: &TenantContext, f: &RegistrationField) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_registration_field_object_safe() {
    fn _f(_: Box<dyn RegistrationFieldRepository>) {}
}

// === RegistrationField repository section end ===

// === StudentCategory repository section begin ===

/// Repository port for the
/// [`StudentCategory`](crate::aggregate::StudentCategory) aggregate.
/// Minimal CRUD interface; extend with queries as needed.
#[async_trait]
pub trait StudentCategoryRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: StudentCategoryId,
    ) -> Result<Option<StudentCategory>>;
    /// Insert a new student category.
    async fn insert(&self, ctx: &TenantContext, c: &StudentCategory) -> Result<()>;
    /// Update an existing student category.
    async fn update(&self, ctx: &TenantContext, c: &StudentCategory) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_student_category_object_safe() {
    fn _f(_: Box<dyn StudentCategoryRepository>) {}
}

// === StudentCategory repository section end ===

// === StudentGroup repository section begin ===

/// Repository port for the [`StudentGroup`](crate::aggregate::StudentGroup)
/// aggregate. Minimal CRUD interface; extend with queries as needed.
#[async_trait]
pub trait StudentGroupRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(&self, ctx: &TenantContext, id: StudentGroupId) -> Result<Option<StudentGroup>>;
    /// Insert a new student group.
    async fn insert(&self, ctx: &TenantContext, g: &StudentGroup) -> Result<()>;
    /// Update an existing student group.
    async fn update(&self, ctx: &TenantContext, g: &StudentGroup) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_student_group_object_safe() {
    fn _f(_: Box<dyn StudentGroupRepository>) {}
}

// === StudentGroup repository section end ===

// === StudentPromotion repository section begin ===

/// Repository port for the
/// [`StudentPromotion`](crate::aggregate::StudentPromotion) aggregate.
/// Minimal CRUD interface; extend with queries as needed.
#[async_trait]
pub trait StudentPromotionRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: StudentPromotionId,
    ) -> Result<Option<StudentPromotion>>;
    /// Insert a new student promotion record.
    async fn insert(&self, ctx: &TenantContext, p: &StudentPromotion) -> Result<()>;
    /// Update an existing student promotion record.
    async fn update(&self, ctx: &TenantContext, p: &StudentPromotion) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_student_promotion_object_safe() {
    fn _f(_: Box<dyn StudentPromotionRepository>) {}
}

// === StudentPromotion repository section end ===

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;

    /// Compile-time check: confirm the repository traits are
    /// object-safe by naming `Box<dyn ...>` for each.
    #[test]
    fn traits_are_object_safe() {
        fn _student(_: Box<dyn StudentRepository>) {}
        fn _class(_: Box<dyn ClassRepository>) {}
        fn _section(_: Box<dyn SectionRepository>) {}
        fn _subject(_: Box<dyn SubjectRepository>) {}
        fn _year(_: Box<dyn AcademicYearRepository>) {}
        fn _record(_: Box<dyn StudentRecordRepository>) {}
        // Wave 9.2b: 14 placeholder ports.
        fn _certificate(_: Box<dyn CertificateRepository>) {}
        fn _class_routine(_: Box<dyn ClassRoutineRepository>) {}
        fn _class_section(_: Box<dyn ClassSectionRepository>) {}
        fn _class_subject(_: Box<dyn ClassSubjectRepository>) {}
        fn _guardian(_: Box<dyn GuardianRepository>) {}
        fn _homework(_: Box<dyn HomeworkRepository>) {}
        fn _id_card(_: Box<dyn IdCardRepository>) {}
        fn _lesson(_: Box<dyn LessonRepository>) {}
        fn _lesson_plan(_: Box<dyn LessonPlanRepository>) {}
        fn _lesson_topic(_: Box<dyn LessonTopicRepository>) {}
        fn _registration_field(_: Box<dyn RegistrationFieldRepository>) {}
        fn _student_category(_: Box<dyn StudentCategoryRepository>) {}
        fn _student_group(_: Box<dyn StudentGroupRepository>) {}
        fn _student_promotion(_: Box<dyn StudentPromotionRepository>) {}
    }
}
