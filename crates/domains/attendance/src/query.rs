//! # Attendance query builders
//!
//! Phase 5 Workstream A ships 5 typed query stubs:
//!
//! - [`StudentAttendanceQuery`]
//! - [`SubjectAttendanceQuery`]
//! - [`StaffAttendanceQuery`]
//! - [`ExamAttendanceQuery`]
//! - [`BulkAttendanceImportQuery`]
//!
//! All return `Err(DomainError::not_supported(...))` from
//! `execute()` — same pattern as the academic and
//! assessment crates' Phase 3/4 query stubs.

#![allow(missing_docs)] // The query field names are
                        // self-documenting via the parameter
                        // names; suppressing this lint for the
                        // file is the pragmatic choice for the
                        // 5 query stubs Phase 5 ships.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use educore_assessment::ExamId;
use educore_core::error::{DomainError, Result};
use educore_core::tenant::TenantContext;

use crate::value_objects::{
    AcademicYearId, AttendanceSource, AttendanceType, ClassId, SectionId, SubjectId,
};

// =============================================================================
// StudentAttendanceQuery
// =============================================================================

/// A typed query over the
/// [`StudentAttendance`](crate::aggregate::StudentAttendance)
/// aggregate. **Phase 5 stub.**
#[allow(dead_code)]
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StudentAttendanceQuery {
    pub student_id: Option<crate::value_objects::StudentId>,
    pub class_id: Option<ClassId>,
    pub section_id: Option<SectionId>,
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
    pub attendance_type: Option<AttendanceType>,
    pub offset: u32,
    pub limit: u32,
}

impl StudentAttendanceQuery {
    #[allow(dead_code)]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            student_id: None,
            class_id: None,
            section_id: None,
            from_date: None,
            to_date: None,
            attendance_type: None,
            offset: 0,
            limit: 50,
        }
    }

    #[allow(dead_code)]
    #[must_use]
    pub const fn with_student(mut self, v: crate::value_objects::StudentId) -> Self {
        self.student_id = Some(v);
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub const fn with_class(mut self, v: ClassId) -> Self {
        self.class_id = Some(v);
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub const fn with_section(mut self, v: SectionId) -> Self {
        self.section_id = Some(v);
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub const fn with_from_date(mut self, v: NaiveDate) -> Self {
        self.from_date = Some(v);
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub const fn with_to_date(mut self, v: NaiveDate) -> Self {
        self.to_date = Some(v);
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub const fn with_attendance_type(mut self, v: AttendanceType) -> Self {
        self.attendance_type = Some(v);
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub const fn with_offset(mut self, v: u32) -> Self {
        self.offset = v;
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub const fn with_limit(mut self, v: u32) -> Self {
        self.limit = v;
        self
    }

    /// Executes the query. **Phase 5 stub.**
    #[allow(dead_code)]
    pub async fn execute(
        &self,
        _ctx: &TenantContext,
    ) -> Result<Vec<crate::aggregate::StudentAttendance>> {
        Err(DomainError::not_supported(
            "StudentAttendanceQuery::execute is a Phase 5 stub",
        ))
    }
}

// =============================================================================
// SubjectAttendanceQuery
// =============================================================================

/// A typed query over the
/// [`SubjectAttendance`](crate::aggregate::SubjectAttendance)
/// aggregate. **Phase 5 stub.**
#[allow(dead_code)]
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubjectAttendanceQuery {
    pub student_id: Option<crate::value_objects::StudentId>,
    pub subject_id: Option<SubjectId>,
    pub class_id: Option<ClassId>,
    pub section_id: Option<SectionId>,
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
    pub offset: u32,
    pub limit: u32,
}

impl SubjectAttendanceQuery {
    #[allow(dead_code)]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            student_id: None,
            subject_id: None,
            class_id: None,
            section_id: None,
            from_date: None,
            to_date: None,
            offset: 0,
            limit: 50,
        }
    }

    #[allow(dead_code)]
    pub async fn execute(
        &self,
        _ctx: &TenantContext,
    ) -> Result<Vec<crate::aggregate::SubjectAttendance>> {
        Err(DomainError::not_supported(
            "SubjectAttendanceQuery::execute is a Phase 5 stub",
        ))
    }
}

// =============================================================================
// StaffAttendanceQuery
// =============================================================================

/// A typed query over the
/// [`StaffAttendance`](crate::aggregate::StaffAttendance)
/// aggregate. **Phase 5 stub.**
#[allow(dead_code)]
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaffAttendanceQuery {
    pub staff_id: Option<crate::value_objects::StaffId>,
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
    pub attendance_type: Option<AttendanceType>,
    pub offset: u32,
    pub limit: u32,
}

impl StaffAttendanceQuery {
    #[allow(dead_code)]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            staff_id: None,
            from_date: None,
            to_date: None,
            attendance_type: None,
            offset: 0,
            limit: 50,
        }
    }

    #[allow(dead_code)]
    pub async fn execute(
        &self,
        _ctx: &TenantContext,
    ) -> Result<Vec<crate::aggregate::StaffAttendance>> {
        Err(DomainError::not_supported(
            "StaffAttendanceQuery::execute is a Phase 5 stub",
        ))
    }
}

// =============================================================================
// ExamAttendanceQuery
// =============================================================================

/// A typed query over the
/// [`ExamAttendance`](crate::aggregate::ExamAttendance)
/// aggregate. **Phase 5 stub.**
#[allow(dead_code)]
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExamAttendanceQuery {
    pub exam_id: Option<ExamId>,
    pub student_id: Option<crate::value_objects::StudentId>,
    pub class_id: Option<ClassId>,
    pub section_id: Option<SectionId>,
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
    pub offset: u32,
    pub limit: u32,
}

impl ExamAttendanceQuery {
    #[allow(dead_code)]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            exam_id: None,
            student_id: None,
            class_id: None,
            section_id: None,
            from_date: None,
            to_date: None,
            offset: 0,
            limit: 50,
        }
    }

    #[allow(dead_code)]
    pub async fn execute(
        &self,
        _ctx: &TenantContext,
    ) -> Result<Vec<crate::aggregate::ExamAttendance>> {
        Err(DomainError::not_supported(
            "ExamAttendanceQuery::execute is a Phase 5 stub",
        ))
    }
}

// =============================================================================
// BulkAttendanceImportQuery
// =============================================================================

/// A typed query over the
/// [`BulkAttendanceImport`](crate::aggregate::BulkAttendanceImport)
/// aggregate. **Phase 5 stub.**
#[allow(dead_code)]
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BulkAttendanceImportQuery {
    pub academic_year_id: Option<AcademicYearId>,
    pub source: Option<AttendanceSource>,
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
    pub offset: u32,
    pub limit: u32,
}

impl BulkAttendanceImportQuery {
    #[allow(dead_code)]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            academic_year_id: None,
            source: None,
            from_date: None,
            to_date: None,
            offset: 0,
            limit: 50,
        }
    }

    #[allow(dead_code)]
    pub async fn execute(
        &self,
        _ctx: &TenantContext,
    ) -> Result<Vec<crate::aggregate::BulkAttendanceImport>> {
        Err(DomainError::not_supported(
            "BulkAttendanceImportQuery::execute is a Phase 5 stub",
        ))
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
    use educore_core::ids::SchoolId;
    use educore_core::tenant::UserType;

    #[test]
    fn student_attendance_query_new_has_default_paging() {
        let q = StudentAttendanceQuery::new();
        assert_eq!(q.offset, 0);
        assert_eq!(q.limit, 50);
        assert!(q.student_id.is_none());
        assert!(q.class_id.is_none());
    }

    #[test]
    fn student_attendance_query_setters_chain() {
        let s = SchoolId(uuid::Uuid::now_v7());
        let q = StudentAttendanceQuery::new()
            .with_student(crate::value_objects::StudentId::new(
                s,
                uuid::Uuid::now_v7(),
            ))
            .with_class(crate::value_objects::ClassId::new(s, uuid::Uuid::now_v7()))
            .with_section(crate::value_objects::SectionId::new(
                s,
                uuid::Uuid::now_v7(),
            ))
            .with_from_date(chrono::NaiveDate::from_ymd_opt(2024, 9, 1).unwrap())
            .with_to_date(chrono::NaiveDate::from_ymd_opt(2024, 9, 30).unwrap())
            .with_attendance_type(AttendanceType::Absent)
            .with_offset(10)
            .with_limit(100);
        assert!(q.student_id.is_some());
        assert!(q.class_id.is_some());
        assert!(q.section_id.is_some());
        assert!(q.from_date.is_some());
        assert!(q.to_date.is_some());
        assert_eq!(q.attendance_type, Some(AttendanceType::Absent));
        assert_eq!(q.offset, 10);
        assert_eq!(q.limit, 100);
    }

    #[test]
    fn student_attendance_query_execute_returns_not_supported() {
        let g = SystemIdGen;
        let s = g.next_school_id();
        let ctx = TenantContext::for_user(
            s,
            g.next_user_id(),
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        );
        let q = StudentAttendanceQuery::new();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        let err = rt.block_on(q.execute(&ctx)).unwrap_err();
        assert!(matches!(err, DomainError::NotSupported(_)));
    }

    #[test]
    fn subject_attendance_query_execute_returns_not_supported() {
        let g = SystemIdGen;
        let s = g.next_school_id();
        let ctx = TenantContext::for_user(
            s,
            g.next_user_id(),
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        );
        let q = SubjectAttendanceQuery::new();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        let err = rt.block_on(q.execute(&ctx)).unwrap_err();
        assert!(matches!(err, DomainError::NotSupported(_)));
    }

    #[test]
    fn staff_attendance_query_execute_returns_not_supported() {
        let g = SystemIdGen;
        let s = g.next_school_id();
        let ctx = TenantContext::for_user(
            s,
            g.next_user_id(),
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        );
        let q = StaffAttendanceQuery::new();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        let err = rt.block_on(q.execute(&ctx)).unwrap_err();
        assert!(matches!(err, DomainError::NotSupported(_)));
    }

    #[test]
    fn exam_attendance_query_execute_returns_not_supported() {
        let g = SystemIdGen;
        let s = g.next_school_id();
        let ctx = TenantContext::for_user(
            s,
            g.next_user_id(),
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        );
        let q = ExamAttendanceQuery::new();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        let err = rt.block_on(q.execute(&ctx)).unwrap_err();
        assert!(matches!(err, DomainError::NotSupported(_)));
    }

    #[test]
    fn bulk_attendance_import_query_execute_returns_not_supported() {
        let g = SystemIdGen;
        let s = g.next_school_id();
        let ctx = TenantContext::for_user(
            s,
            g.next_user_id(),
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        );
        let q = BulkAttendanceImportQuery::new();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        let err = rt.block_on(q.execute(&ctx)).unwrap_err();
        assert!(matches!(err, DomainError::NotSupported(_)));
    }
}
