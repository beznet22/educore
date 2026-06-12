//! # Academic-domain query builders
//!
//! Phase 3 ships typed query stubs. The full query builder
//! (filter combinators, joins, eager-loads) lands in
//! Phase 4+ alongside the `#[derive(DomainQuery)]` macro
//! emissions. For now, the query types carry the future
//! shape so callers can hold a `StudentQuery` /
//! `ClassQuery` / `SectionQuery` / `SubjectQuery` /
//! `AcademicYearQuery` value and the storage adapter can
//! pattern-match on them when the real implementation
//! lands.

use serde::{Deserialize, Serialize};

use educore_core::error::{DomainError, Result};
use educore_core::ids::SchoolId;
use educore_core::tenant::TenantContext;

use crate::aggregate::{AcademicYear, Class, Section, Student, Subject};
use crate::value_objects::{StudentStatus, SubjectType};

// =============================================================================
// StudentQuery
// =============================================================================

/// A query for [`Student`] aggregates.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StudentQuery {
    /// Optional status filter.
    pub status_filter: Option<StudentStatus>,
    /// Optional class id filter.
    pub class_id_filter: Option<crate::value_objects::ClassId>,
    /// Optional section id filter.
    pub section_id_filter: Option<crate::value_objects::SectionId>,
    /// Optional academic year id filter.
    pub academic_year_id_filter: Option<crate::value_objects::AcademicYearId>,
    /// Optional substring match on `first_name`.
    pub first_name_contains: Option<String>,
    /// Optional substring match on `last_name`.
    pub last_name_contains: Option<String>,
    /// Optional substring match on `admission_no`.
    pub admission_no_contains: Option<String>,
}

impl StudentQuery {
    /// Constructs an empty `StudentQuery`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            status_filter: None,
            class_id_filter: None,
            section_id_filter: None,
            academic_year_id_filter: None,
            first_name_contains: None,
            last_name_contains: None,
            admission_no_contains: None,
        }
    }

    /// Sets the status filter.
    #[must_use]
    pub fn with_status(mut self, status: StudentStatus) -> Self {
        self.status_filter = Some(status);
        self
    }

    /// Sets the class id filter.
    #[must_use]
    pub fn with_class_id(mut self, class_id: crate::value_objects::ClassId) -> Self {
        self.class_id_filter = Some(class_id);
        self
    }

    /// Sets the section id filter.
    #[must_use]
    pub fn with_section_id(mut self, section_id: crate::value_objects::SectionId) -> Self {
        self.section_id_filter = Some(section_id);
        self
    }

    /// Sets the academic year id filter.
    #[must_use]
    pub fn with_academic_year_id(
        mut self,
        academic_year_id: crate::value_objects::AcademicYearId,
    ) -> Self {
        self.academic_year_id_filter = Some(academic_year_id);
        self
    }

    /// Sets the first-name substring filter.
    #[must_use]
    pub fn with_first_name_contains(mut self, needle: impl Into<String>) -> Self {
        self.first_name_contains = Some(needle.into());
        self
    }

    /// Sets the last-name substring filter.
    #[must_use]
    pub fn with_last_name_contains(mut self, needle: impl Into<String>) -> Self {
        self.last_name_contains = Some(needle.into());
        self
    }

    /// Sets the admission-number substring filter.
    #[must_use]
    pub fn with_admission_no_contains(mut self, needle: impl Into<String>) -> Self {
        self.admission_no_contains = Some(needle.into());
        self
    }

    /// Stub: the real query executor lands in Phase 4+. For
    /// now this returns `Err(DomainError::NotSupported)` so
    /// callers can be wired up before the implementation
    /// lands and the lints stay clean.
    pub async fn execute(self, _ctx: &TenantContext) -> Result<Vec<Student>> {
        let _ = self;
        Err(DomainError::not_supported(
            "StudentQuery::execute is a Phase 3 stub; the typed query executor lands in Phase 4+",
        ))
    }
}

// =============================================================================
// ClassQuery
// =============================================================================

/// A query for [`Class`] aggregates.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassQuery {
    /// Optional substring match on `name`.
    pub name_contains: Option<String>,
}

impl ClassQuery {
    /// Constructs an empty `ClassQuery`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            name_contains: None,
        }
    }

    /// Sets the name-substring filter.
    #[must_use]
    pub fn with_name_contains(mut self, needle: impl Into<String>) -> Self {
        self.name_contains = Some(needle.into());
        self
    }

    /// Stub: the real query executor lands in Phase 4+.
    pub async fn execute(self, _ctx: &TenantContext) -> Result<Vec<Class>> {
        let _ = self;
        Err(DomainError::not_supported(
            "ClassQuery::execute is a Phase 3 stub; the typed query executor lands in Phase 4+",
        ))
    }
}

// =============================================================================
// SectionQuery
// =============================================================================

/// A query for [`Section`] aggregates.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionQuery {
    /// Optional substring match on `name`.
    pub name_contains: Option<String>,
}

impl SectionQuery {
    /// Constructs an empty `SectionQuery`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            name_contains: None,
        }
    }

    /// Sets the name-substring filter.
    #[must_use]
    pub fn with_name_contains(mut self, needle: impl Into<String>) -> Self {
        self.name_contains = Some(needle.into());
        self
    }

    /// Stub: the real query executor lands in Phase 4+.
    pub async fn execute(self, _ctx: &TenantContext) -> Result<Vec<Section>> {
        let _ = self;
        Err(DomainError::not_supported(
            "SectionQuery::execute is a Phase 3 stub; the typed query executor lands in Phase 4+",
        ))
    }
}

// =============================================================================
// SubjectQuery
// =============================================================================

/// A query for [`Subject`] aggregates.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubjectQuery {
    /// Optional subject type filter.
    pub subject_type_filter: Option<SubjectType>,
    /// Optional substring match on `code`.
    pub code_contains: Option<String>,
    /// Optional substring match on `name`.
    pub name_contains: Option<String>,
}

impl SubjectQuery {
    /// Constructs an empty `SubjectQuery`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            subject_type_filter: None,
            code_contains: None,
            name_contains: None,
        }
    }

    /// Sets the subject type filter.
    #[must_use]
    pub fn with_subject_type(mut self, t: SubjectType) -> Self {
        self.subject_type_filter = Some(t);
        self
    }

    /// Sets the code-substring filter.
    #[must_use]
    pub fn with_code_contains(mut self, needle: impl Into<String>) -> Self {
        self.code_contains = Some(needle.into());
        self
    }

    /// Sets the name-substring filter.
    #[must_use]
    pub fn with_name_contains(mut self, needle: impl Into<String>) -> Self {
        self.name_contains = Some(needle.into());
        self
    }

    /// Stub: the real query executor lands in Phase 4+.
    pub async fn execute(self, _ctx: &TenantContext) -> Result<Vec<Subject>> {
        let _ = self;
        Err(DomainError::not_supported(
            "SubjectQuery::execute is a Phase 3 stub; the typed query executor lands in Phase 4+",
        ))
    }
}

// =============================================================================
// AcademicYearQuery
// =============================================================================

/// A query for [`AcademicYear`] aggregates.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcademicYearQuery {
    /// Optional current-flag filter.
    pub is_current: Option<bool>,
    /// Optional closed-flag filter.
    pub is_closed: Option<bool>,
    /// Optional substring match on `year`.
    pub year_contains: Option<String>,
    /// Optional substring match on `title`.
    pub title_contains: Option<String>,
}

impl AcademicYearQuery {
    /// Constructs an empty `AcademicYearQuery`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            is_current: None,
            is_closed: None,
            year_contains: None,
            title_contains: None,
        }
    }

    /// Sets the is_current filter.
    #[must_use]
    pub fn with_is_current(mut self, v: bool) -> Self {
        self.is_current = Some(v);
        self
    }

    /// Sets the is_closed filter.
    #[must_use]
    pub fn with_is_closed(mut self, v: bool) -> Self {
        self.is_closed = Some(v);
        self
    }

    /// Sets the year-substring filter.
    #[must_use]
    pub fn with_year_contains(mut self, needle: impl Into<String>) -> Self {
        self.year_contains = Some(needle.into());
        self
    }

    /// Sets the title-substring filter.
    #[must_use]
    pub fn with_title_contains(mut self, needle: impl Into<String>) -> Self {
        self.title_contains = Some(needle.into());
        self
    }

    /// Stub: the real query executor lands in Phase 4+.
    pub async fn execute(self, _school: SchoolId) -> Result<Vec<AcademicYear>> {
        let _ = self;
        Err(DomainError::not_supported(
            "AcademicYearQuery::execute is a Phase 3 stub; the typed query executor lands in Phase 4+",
        ))
    }
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
    use educore_core::clock::IdGenerator;

    #[test]
    fn student_query_builder_setter_methods() {
        let g = educore_core::clock::SystemIdGen;
        let school = g.next_school_id();
        let class_id = crate::value_objects::ClassId::new(school, g.next_uuid());
        let q = StudentQuery::new()
            .with_status(StudentStatus::Active)
            .with_class_id(class_id);
        assert_eq!(q.status_filter, Some(StudentStatus::Active));
        assert_eq!(q.class_id_filter, Some(class_id));
    }

    #[test]
    fn class_query_builder_setter_methods() {
        let q = ClassQuery::new().with_name_contains("Grade");
        assert_eq!(q.name_contains.as_deref(), Some("Grade"));
    }

    #[test]
    fn section_query_builder_setter_methods() {
        let q = SectionQuery::new().with_name_contains("A");
        assert_eq!(q.name_contains.as_deref(), Some("A"));
    }

    #[test]
    fn subject_query_builder_setter_methods() {
        let q = SubjectQuery::new()
            .with_subject_type(SubjectType::Theory)
            .with_code_contains("MATH")
            .with_name_contains("Math");
        assert_eq!(q.subject_type_filter, Some(SubjectType::Theory));
        assert_eq!(q.code_contains.as_deref(), Some("MATH"));
    }

    #[test]
    fn academic_year_query_builder_setter_methods() {
        let q = AcademicYearQuery::new()
            .with_is_current(true)
            .with_year_contains("2026");
        assert_eq!(q.is_current, Some(true));
        assert_eq!(q.year_contains.as_deref(), Some("2026"));
    }
}
