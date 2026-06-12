//! # Assessment query builders
//!
//! Phase 4 Workstream A ships the [`ExamQuery`] typed query
//! stub. The full query executors land in Phase 5+
//! alongside the `#[derive(DomainQuery)]` macro emissions
//! on the 8 in-scope aggregates.
//!
//! The query stubs return `Err(DomainError::not_supported(...))`
//! from `execute()` — same pattern as the academic crate's
//! Phase 3 query stubs.

use serde::{Deserialize, Serialize};

use educore_core::error::{DomainError, Result};
use educore_core::tenant::TenantContext;

use crate::value_objects::ExamTypeId;
use educore_academic::AcademicYearId;

// =============================================================================
// ExamQuery
// =============================================================================

/// A typed query over the [`Exam`](crate::aggregate::Exam)
/// aggregate. The fields are the canonical filter columns;
/// setting a field to `Some(_)` adds a `WHERE` clause.
///
/// **Phase 4 stub:** the executor is a placeholder. The real
/// `#[derive(DomainQuery)]`-emitted AST lands in a later
/// phase. `execute()` always returns
/// `Err(DomainError::not_supported(...))`.
#[allow(dead_code)]
// The methods are not called from within the assessment crate;
// the typed query executors land in a later phase.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExamQuery {
    /// Filter: `exam_type_id = ?`.
    pub exam_type_id: Option<ExamTypeId>,
    /// Filter: `code` (substring, case-insensitive).
    pub code_contains: Option<String>,
    /// Filter: `name` (substring, case-insensitive).
    pub name_contains: Option<String>,
    /// Filter: `is_published = ?`.
    pub is_published: Option<bool>,
    /// Page offset (0-indexed).
    pub offset: u32,
    /// Page limit.
    pub limit: u32,
}

impl ExamQuery {
    /// Constructs a new `ExamQuery` with `offset = 0,
    /// limit = 50` and no filters.
    #[allow(dead_code)] // Typed query builder; executor is Phase 5+.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            exam_type_id: None,
            code_contains: None,
            name_contains: None,
            is_published: None,
            offset: 0,
            limit: 50,
        }
    }

    /// Sets `exam_type_id`.
    #[allow(dead_code)]
    #[must_use]
    pub const fn with_exam_type(mut self, v: ExamTypeId) -> Self {
        self.exam_type_id = Some(v);
        self
    }

    /// Sets `code_contains` (substring filter).
    #[allow(dead_code)]
    #[must_use]
    pub fn with_code_contains(mut self, v: impl Into<String>) -> Self {
        self.code_contains = Some(v.into());
        self
    }

    /// Sets `name_contains` (substring filter).
    #[allow(dead_code)]
    #[must_use]
    pub fn with_name_contains(mut self, v: impl Into<String>) -> Self {
        self.name_contains = Some(v.into());
        self
    }

    /// Sets `is_published`.
    #[allow(dead_code)]
    #[must_use]
    pub const fn with_is_published(mut self, v: bool) -> Self {
        self.is_published = Some(v);
        self
    }

    /// Sets `offset`.
    #[allow(dead_code)]
    #[must_use]
    pub const fn with_offset(mut self, v: u32) -> Self {
        self.offset = v;
        self
    }

    /// Sets `limit`.
    #[allow(dead_code)]
    #[must_use]
    pub const fn with_limit(mut self, v: u32) -> Self {
        self.limit = v;
        self
    }

    /// Executes the query. **Phase 4 stub:** returns
    /// `Err(DomainError::not_supported(...))`.
    ///
    /// # Errors
    ///
    /// Always returns `DomainError::NotSupported` in Phase 4.
    #[allow(dead_code)]
    pub async fn execute(&self, _ctx: &TenantContext) -> Result<Vec<crate::aggregate::Exam>> {
        Err(DomainError::not_supported(
            "ExamQuery::execute is a Phase 4 stub; the typed query executor lands in Phase 5+",
        ))
    }
}

// =============================================================================
// Workstream B query stubs
// =============================================================================

/// A typed query over the [`ExamSchedule`](crate::aggregate::ExamSchedule)
/// aggregate. **Phase 4 stub.**
#[allow(dead_code)]
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExamScheduleQuery {
    pub exam_id: Option<crate::value_objects::ExamId>,
    pub class_id: Option<educore_academic::ClassId>,
    pub section_id: Option<educore_academic::SectionId>,
    pub from_date: Option<chrono::NaiveDate>,
    pub to_date: Option<chrono::NaiveDate>,
    pub offset: u32,
    pub limit: u32,
}
impl ExamScheduleQuery {
    #[allow(dead_code)]
    #[must_use] pub const fn new() -> Self { Self { exam_id: None, class_id: None, section_id: None, from_date: None, to_date: None, offset: 0, limit: 50 } }
    #[allow(dead_code)]
    pub async fn execute(&self, _ctx: &TenantContext) -> Result<Vec<crate::aggregate::ExamSchedule>> {
        Err(DomainError::not_supported("ExamScheduleQuery::execute is a Phase 4 stub"))
    }
}

/// A typed query over the [`SeatPlan`](crate::aggregate::SeatPlan)
/// aggregate. **Phase 4 stub.**
#[allow(dead_code)]
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeatPlanQuery {
    pub exam_id: Option<crate::value_objects::ExamId>,
    pub class_id: Option<educore_academic::ClassId>,
    pub section_id: Option<educore_academic::SectionId>,
    pub offset: u32,
    pub limit: u32,
}
impl SeatPlanQuery {
    #[allow(dead_code)]
    #[must_use] pub const fn new() -> Self { Self { exam_id: None, class_id: None, section_id: None, offset: 0, limit: 50 } }
    #[allow(dead_code)]
    pub async fn execute(&self, _ctx: &TenantContext) -> Result<Vec<crate::aggregate::SeatPlan>> {
        Err(DomainError::not_supported("SeatPlanQuery::execute is a Phase 4 stub"))
    }
}

/// A typed query over the [`AdmitCard`](crate::aggregate::AdmitCard)
/// aggregate. **Phase 4 stub.**
#[allow(dead_code)]
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdmitCardQuery {
    pub student_record_id: Option<crate::value_objects::StudentRecordId>,
    pub exam_type_id: Option<crate::value_objects::ExamTypeId>,
    pub academic_year_id: Option<AcademicYearId>,
    pub offset: u32,
    pub limit: u32,
}
impl AdmitCardQuery {
    #[allow(dead_code)]
    #[must_use] pub const fn new() -> Self { Self { student_record_id: None, exam_type_id: None, academic_year_id: None, offset: 0, limit: 50 } }
    #[allow(dead_code)]
    pub async fn execute(&self, _ctx: &TenantContext) -> Result<Vec<crate::aggregate::AdmitCard>> {
        Err(DomainError::not_supported("AdmitCardQuery::execute is a Phase 4 stub"))
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;
    use educore_core::clock::{IdGenerator, SystemIdGen};
    use educore_core::tenant::UserType;

    #[test]
    fn exam_query_new_has_default_paging() {
        let q = ExamQuery::new();
        assert_eq!(q.offset, 0);
        assert_eq!(q.limit, 50);
        assert!(q.exam_type_id.is_none());
        assert!(q.code_contains.is_none());
        assert!(q.name_contains.is_none());
        assert!(q.is_published.is_none());
    }

    #[test]
    fn exam_query_setters_chain() {
        let s = educore_core::ids::SchoolId(uuid::Uuid::now_v7());
        let q = ExamQuery::new()
            .with_exam_type(ExamTypeId::new(s, uuid::Uuid::now_v7()))
            .with_code_contains("MTH-")
            .with_name_contains("Mid")
            .with_is_published(true)
            .with_offset(10)
            .with_limit(100);
        assert!(q.exam_type_id.is_some());
        assert_eq!(q.code_contains.as_deref(), Some("MTH-"));
        assert_eq!(q.name_contains.as_deref(), Some("Mid"));
        assert_eq!(q.is_published, Some(true));
        assert_eq!(q.offset, 10);
        assert_eq!(q.limit, 100);
    }

    #[test]
    fn exam_query_execute_returns_not_supported() {
        let g = SystemIdGen;
        let s = g.next_school_id();
        let ctx = educore_core::tenant::TenantContext::for_user(
            s,
            g.next_user_id(),
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        );
        let q = ExamQuery::new();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        let err = rt.block_on(q.execute(&ctx)).unwrap_err();
        assert!(matches!(err, DomainError::NotSupported(_)));
    }
}
