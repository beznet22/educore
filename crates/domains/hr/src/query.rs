//! # HR query stubs
//!
//! Phase 6 query stubs. Every `execute()` returns
//! `Err(DomainError::not_supported(...))` until Phase 7+
//! wires the typed executor + storage-port translation.

#![allow(missing_docs)]
#![allow(unused_imports)]
#![allow(dead_code)]

use educore_core::error::{DomainError, Result};
use educore_core::ids::SchoolId;
use educore_core::tenant::TenantContext;

use crate::value_objects::{
    AttendanceType, DepartmentId, DesignationId, LeaveRequestId, LeaveStatus, LeaveTypeId,
    PayrollGenerateId, PayrollStatus, StaffId, StaffStatus,
};

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StaffQuery {
    pub staff_no: Option<u32>,
    pub email: Option<String>,
    pub mobile: Option<String>,
    pub department_id: Option<DepartmentId>,
    pub designation_id: Option<DesignationId>,
    pub status: Option<StaffStatus>,
    pub offset: u32,
    pub limit: u32,
}

impl StaffQuery {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            staff_no: None,
            email: None,
            mobile: None,
            department_id: None,
            designation_id: None,
            status: None,
            offset: 0,
            limit: 50,
        }
    }

    #[must_use]
    pub fn with_staff_no(mut self, v: u32) -> Self {
        self.staff_no = Some(v);
        self
    }
    #[must_use]
    pub fn with_email(mut self, v: String) -> Self {
        self.email = Some(v);
        self
    }
    #[must_use]
    pub fn with_mobile(mut self, v: String) -> Self {
        self.mobile = Some(v);
        self
    }
    #[must_use]
    pub fn with_department(mut self, v: DepartmentId) -> Self {
        self.department_id = Some(v);
        self
    }
    #[must_use]
    pub fn with_designation(mut self, v: DesignationId) -> Self {
        self.designation_id = Some(v);
        self
    }
    #[must_use]
    pub fn with_status(mut self, v: StaffStatus) -> Self {
        self.status = Some(v);
        self
    }

    /// Phase 6 stub.
    pub async fn execute(&self, _ctx: &TenantContext) -> Result<Vec<crate::aggregate::Staff>> {
        Err(DomainError::not_supported(
            "StaffQuery::execute is a Phase 6 stub",
        ))
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LeaveRequestQuery {
    pub staff_id: Option<StaffId>,
    pub type_id: Option<LeaveTypeId>,
    pub status: Option<LeaveStatus>,
    pub offset: u32,
    pub limit: u32,
}

impl LeaveRequestQuery {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            staff_id: None,
            type_id: None,
            status: None,
            offset: 0,
            limit: 50,
        }
    }

    #[must_use]
    pub const fn with_staff(mut self, v: StaffId) -> Self {
        self.staff_id = Some(v);
        self
    }
    #[must_use]
    pub const fn with_type(mut self, v: LeaveTypeId) -> Self {
        self.type_id = Some(v);
        self
    }
    #[must_use]
    pub const fn with_status(mut self, v: LeaveStatus) -> Self {
        self.status = Some(v);
        self
    }

    /// Phase 6 stub.
    pub async fn execute(
        &self,
        _ctx: &TenantContext,
    ) -> Result<Vec<crate::aggregate::LeaveRequest>> {
        Err(DomainError::not_supported(
            "LeaveRequestQuery::execute is a Phase 6 stub",
        ))
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PayrollGenerateQuery {
    pub staff_id: Option<StaffId>,
    pub payroll_month: Option<u8>,
    pub payroll_year: Option<u16>,
    pub status: Option<PayrollStatus>,
    pub offset: u32,
    pub limit: u32,
}

impl PayrollGenerateQuery {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            staff_id: None,
            payroll_month: None,
            payroll_year: None,
            status: None,
            offset: 0,
            limit: 50,
        }
    }

    #[must_use]
    pub const fn with_staff(mut self, v: StaffId) -> Self {
        self.staff_id = Some(v);
        self
    }
    #[must_use]
    pub const fn with_status(mut self, v: PayrollStatus) -> Self {
        self.status = Some(v);
        self
    }

    /// Phase 6 stub.
    pub async fn execute(
        &self,
        _ctx: &TenantContext,
    ) -> Result<Vec<crate::aggregate::PayrollGenerate>> {
        Err(DomainError::not_supported(
            "PayrollGenerateQuery::execute is a Phase 6 stub",
        ))
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StaffAttendanceQuery {
    pub staff_id: Option<StaffId>,
    pub from: Option<chrono::NaiveDate>,
    pub to: Option<chrono::NaiveDate>,
    pub attendance_type: Option<AttendanceType>,
    pub offset: u32,
    pub limit: u32,
}

impl StaffAttendanceQuery {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            staff_id: None,
            from: None,
            to: None,
            attendance_type: None,
            offset: 0,
            limit: 50,
        }
    }

    #[must_use]
    pub const fn with_staff(mut self, v: StaffId) -> Self {
        self.staff_id = Some(v);
        self
    }
    #[must_use]
    pub const fn with_type(mut self, v: AttendanceType) -> Self {
        self.attendance_type = Some(v);
        self
    }

    /// Phase 6 stub.
    pub async fn execute(
        &self,
        _ctx: &TenantContext,
    ) -> Result<Vec<crate::aggregate::StaffAttendance>> {
        Err(DomainError::not_supported(
            "StaffAttendanceQuery::execute is a Phase 6 stub",
        ))
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DepartmentQuery {
    pub name: Option<String>,
    pub offset: u32,
    pub limit: u32,
}

impl DepartmentQuery {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            name: None,
            offset: 0,
            limit: 50,
        }
    }
    pub async fn execute(&self, _ctx: &TenantContext) -> Result<Vec<crate::aggregate::Department>> {
        Err(DomainError::not_supported(
            "DepartmentQuery::execute is a Phase 6 stub",
        ))
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DesignationQuery {
    pub title: Option<String>,
    pub offset: u32,
    pub limit: u32,
}

impl DesignationQuery {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            title: None,
            offset: 0,
            limit: 50,
        }
    }
    pub async fn execute(
        &self,
        _ctx: &TenantContext,
    ) -> Result<Vec<crate::aggregate::Designation>> {
        Err(DomainError::not_supported(
            "DesignationQuery::execute is a Phase 6 stub",
        ))
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LeaveTypeQuery {
    pub name: Option<String>,
    pub offset: u32,
    pub limit: u32,
}

impl LeaveTypeQuery {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            name: None,
            offset: 0,
            limit: 50,
        }
    }
    pub async fn execute(&self, _ctx: &TenantContext) -> Result<Vec<crate::aggregate::LeaveType>> {
        Err(DomainError::not_supported(
            "LeaveTypeQuery::execute is a Phase 6 stub",
        ))
    }
}
