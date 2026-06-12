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
//! [`AcademicYearRepository`]. The remaining ports
//! (`GuardianRepository`, `ClassSectionRepository`,
//! `StudentRecordRepository`, ...) land in later phases.

use async_trait::async_trait;

use educore_core::error::Result;
use educore_core::ids::SchoolId;
use educore_core::tenant::TenantContext;

use crate::aggregate::{AcademicYear, Class, Section, Student, Subject};
use crate::value_objects::{
    AcademicYearId, ClassId, SectionId, StudentId, StudentStatus, SubjectId,
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
    }
}
