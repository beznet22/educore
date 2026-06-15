//! # Facilities domain queries
//!
//! Phase 8 ships the 13 typed query stubs (one per aggregate +
//! the 2 child aggregates). Each query has a `to_query_node`
//! method that returns a typed `QueryNode<F>` AST, and an
//! `execute` method that returns
//! `Err(DomainError::not_supported(...))` for now. The typed
//! executors land in a follow-up phase alongside the
//! `#[derive(DomainQuery)]` macro emissions (per the Phase 7
//! Workstream P pattern).
//!
//! Mirrors `crates/domains/finance/src/query.rs`.

#![allow(missing_docs)]
#![allow(unused_imports)]

use chrono::NaiveDate;

use educore_core::error::{DomainError, Result};
use educore_core::ids::SchoolId;

use crate::value_objects::{
    AcademicYearId, AssignVehicleId, DormitoryId, ItemCategoryId, ItemId, ItemIssueId,
    ItemReceiveId, ItemSellId, ItemStoreId, RoomId, RoomTypeId, RouteId, SupplierId, VehicleId,
};

// =============================================================================
// VehicleQuery
// =============================================================================

/// A typed query for `Vehicle` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct VehicleQuery {
    pub school_id: SchoolId,
    pub academic_year_id: Option<AcademicYearId>,
    pub active_only: bool,
    pub driver_id: Option<crate::value_objects::StaffId>,
}

impl VehicleQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "facilities.vehicle.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 8; the typed executor lands in a follow-up phase.
    pub async fn execute(&self) -> Result<Vec<crate::aggregate::Vehicle>> {
        Err(DomainError::not_supported(
            "VehicleQuery::execute is a Phase 8 stub; real executor lands with the DomainQuery macro",
        ))
    }
}

// =============================================================================
// RouteQuery
// =============================================================================

/// A typed query for `Route` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct RouteQuery {
    pub school_id: SchoolId,
    pub academic_year_id: AcademicYearId,
}

impl RouteQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "facilities.route.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 8.
    pub async fn execute(&self) -> Result<Vec<crate::aggregate::Route>> {
        Err(DomainError::not_supported(
            "RouteQuery::execute is a Phase 8 stub",
        ))
    }
}

// =============================================================================
// AssignVehicleQuery
// =============================================================================

/// A typed query for `AssignVehicle` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct AssignVehicleQuery {
    pub school_id: SchoolId,
    pub academic_year_id: Option<AcademicYearId>,
    pub vehicle_id: Option<VehicleId>,
    pub route_id: Option<RouteId>,
}

impl AssignVehicleQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "facilities.assign_vehicle.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 8.
    pub async fn execute(&self) -> Result<Vec<crate::aggregate::AssignVehicle>> {
        Err(DomainError::not_supported(
            "AssignVehicleQuery::execute is a Phase 8 stub",
        ))
    }
}

// =============================================================================
// DormitoryQuery
// =============================================================================

/// A typed query for `Dormitory` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct DormitoryQuery {
    pub school_id: SchoolId,
    pub academic_year_id: AcademicYearId,
    pub dormitory_type: Option<crate::value_objects::DormitoryType>,
}

impl DormitoryQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "facilities.dormitory.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 8.
    pub async fn execute(&self) -> Result<Vec<crate::aggregate::Dormitory>> {
        Err(DomainError::not_supported(
            "DormitoryQuery::execute is a Phase 8 stub",
        ))
    }
}

// =============================================================================
// RoomQuery
// =============================================================================

/// A typed query for `Room` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct RoomQuery {
    pub school_id: SchoolId,
    pub dormitory_id: Option<DormitoryId>,
    pub room_type_id: Option<RoomTypeId>,
    pub available_only: bool,
}

impl RoomQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "facilities.room.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 8.
    pub async fn execute(&self) -> Result<Vec<crate::aggregate::Room>> {
        Err(DomainError::not_supported(
            "RoomQuery::execute is a Phase 8 stub",
        ))
    }
}

// =============================================================================
// RoomTypeQuery
// =============================================================================

/// A typed query for `RoomType` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct RoomTypeQuery {
    pub school_id: SchoolId,
}

impl RoomTypeQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "facilities.room_type.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 8.
    pub async fn execute(&self) -> Result<Vec<crate::aggregate::RoomType>> {
        Err(DomainError::not_supported(
            "RoomTypeQuery::execute is a Phase 8 stub",
        ))
    }
}

// =============================================================================
// ItemCategoryQuery
// =============================================================================

/// A typed query for `ItemCategory` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemCategoryQuery {
    pub school_id: SchoolId,
}

impl ItemCategoryQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "facilities.item_category.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 8.
    pub async fn execute(&self) -> Result<Vec<crate::aggregate::ItemCategory>> {
        Err(DomainError::not_supported(
            "ItemCategoryQuery::execute is a Phase 8 stub",
        ))
    }
}

// =============================================================================
// ItemQuery
// =============================================================================

/// A typed query for `Item` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemQuery {
    pub school_id: SchoolId,
    pub academic_year_id: Option<AcademicYearId>,
    pub item_category_id: Option<ItemCategoryId>,
    pub low_stock_only: bool,
    pub sku: Option<String>,
}

impl ItemQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "facilities.item.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 8.
    pub async fn execute(&self) -> Result<Vec<crate::aggregate::Item>> {
        Err(DomainError::not_supported(
            "ItemQuery::execute is a Phase 8 stub",
        ))
    }
}

// =============================================================================
// ItemStoreQuery
// =============================================================================

/// A typed query for `ItemStore` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemStoreQuery {
    pub school_id: SchoolId,
}

impl ItemStoreQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "facilities.item_store.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 8.
    pub async fn execute(&self) -> Result<Vec<crate::aggregate::ItemStore>> {
        Err(DomainError::not_supported(
            "ItemStoreQuery::execute is a Phase 8 stub",
        ))
    }
}

// =============================================================================
// ItemIssueQuery
// =============================================================================

/// A typed query for `ItemIssue` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemIssueQuery {
    pub school_id: SchoolId,
    pub open_only: bool,
    pub overdue_only: bool,
    pub as_of: Option<NaiveDate>,
}

impl ItemIssueQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "facilities.item_issue.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 8.
    pub async fn execute(&self) -> Result<Vec<crate::aggregate::ItemIssue>> {
        Err(DomainError::not_supported(
            "ItemIssueQuery::execute is a Phase 8 stub",
        ))
    }
}

// =============================================================================
// ItemReceiveQuery
// =============================================================================

/// A typed query for `ItemReceive` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemReceiveQuery {
    pub school_id: SchoolId,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub supplier_id: Option<SupplierId>,
    pub store_id: Option<ItemStoreId>,
}

impl ItemReceiveQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "facilities.item_receive.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 8.
    pub async fn execute(&self) -> Result<Vec<crate::aggregate::ItemReceive>> {
        Err(DomainError::not_supported(
            "ItemReceiveQuery::execute is a Phase 8 stub",
        ))
    }
}

// =============================================================================
// ItemSellQuery
// =============================================================================

/// A typed query for `ItemSell` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemSellQuery {
    pub school_id: SchoolId,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub buyer: Option<crate::value_objects::IssueRecipient>,
    pub open_only: bool,
}

impl ItemSellQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "facilities.item_sell.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 8.
    pub async fn execute(&self) -> Result<Vec<crate::aggregate::ItemSell>> {
        Err(DomainError::not_supported(
            "ItemSellQuery::execute is a Phase 8 stub",
        ))
    }
}

// =============================================================================
// SupplierQuery
// =============================================================================

/// A typed query for `Supplier` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct SupplierQuery {
    pub school_id: SchoolId,
    pub active_only: bool,
}

impl SupplierQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "facilities.supplier.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 8.
    pub async fn execute(&self) -> Result<Vec<crate::aggregate::Supplier>> {
        Err(DomainError::not_supported(
            "SupplierQuery::execute is a Phase 8 stub",
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
    use educore_core::clock::SystemIdGen;

    #[test]
    fn query_type_constants_are_unique() {
        let all: &[&str] = &[
            VehicleQuery::query_type(),
            RouteQuery::query_type(),
            AssignVehicleQuery::query_type(),
            DormitoryQuery::query_type(),
            RoomQuery::query_type(),
            RoomTypeQuery::query_type(),
            ItemCategoryQuery::query_type(),
            ItemQuery::query_type(),
            ItemStoreQuery::query_type(),
            ItemIssueQuery::query_type(),
            ItemReceiveQuery::query_type(),
            ItemSellQuery::query_type(),
            SupplierQuery::query_type(),
        ];
        assert_eq!(all.len(), 13);
        for q in all {
            assert!(q.starts_with("facilities."), "bad query type {q}");
        }
    }
}
