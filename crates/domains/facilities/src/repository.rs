//! # Facilities domain repository ports
//!
//! The repository traits the storage adapters implement. Every
//! repository takes a `SchoolId` (or operates on a typed
//! identifier that already embeds it) and refuses to return
//! data from another school. Tenant isolation is structural.
//!
//! All 11 headline aggregates have a `pub trait
//! XxxRepository: Send + Sync` port trait with the standard
//! `get` / `list` / `insert` / `update` / `delete` methods plus
//! per-aggregate `find_by_*` and `list_for_*` helpers.
//!
//! Mirrors the Phase 7 finance pattern (44 port traits shipped
//! in commit `3fe575e`).

#![allow(missing_docs)]
#![allow(unused_imports)]

use async_trait::async_trait;
use chrono::NaiveDate;

use educore_core::error::Result;
use educore_core::ids::SchoolId;

use crate::aggregate::{
    AssignVehicle, Dormitory, Item, ItemIssue, ItemReceive, ItemReceiveChild, ItemSell,
    ItemSellChild, ItemStore, Room, RoomType, Supplier, Vehicle,
};
use crate::value_objects::{
    AcademicYearId, AssignVehicleId, DormitoryId, IssueRecipient, ItemCategoryId, ItemId,
    ItemIssueId, ItemReceiveId, ItemStoreId, RoomId, RoomTypeId, RouteId, RouteStopId, StopName,
    StudentId, SupplierId, VehicleId, VehicleNumber,
};

// =============================================================================
// VehicleRepository
// =============================================================================

#[async_trait]
pub trait VehicleRepository: Send + Sync {
    async fn get(&self, id: VehicleId) -> Result<Option<Vehicle>>;
    async fn get_by_number(
        &self,
        school: SchoolId,
        vehicle_no: &VehicleNumber,
    ) -> Result<Option<Vehicle>>;
    async fn insert(&self, vehicle: &Vehicle) -> Result<()>;
    async fn update(&self, vehicle: &Vehicle) -> Result<()>;
    async fn delete(&self, id: VehicleId) -> Result<()>;
    async fn list(&self, school: SchoolId) -> Result<Vec<Vehicle>>;
    async fn list_active(&self, school: SchoolId) -> Result<Vec<Vehicle>>;
    async fn find_by_driver(
        &self,
        school: SchoolId,
        driver_id: crate::value_objects::StaffId,
    ) -> Result<Option<Vehicle>>;
}

// =============================================================================
// RouteRepository
// =============================================================================

#[async_trait]
pub trait RouteRepository: Send + Sync {
    async fn get(&self, id: RouteId) -> Result<Option<crate::aggregate::Route>>;
    async fn find(
        &self,
        school: SchoolId,
        year: AcademicYearId,
        title: &str,
    ) -> Result<Option<crate::aggregate::Route>>;
    async fn list(
        &self,
        school: SchoolId,
        year: AcademicYearId,
    ) -> Result<Vec<crate::aggregate::Route>>;
    async fn insert(&self, route: &crate::aggregate::Route) -> Result<()>;
    async fn update(&self, route: &crate::aggregate::Route) -> Result<()>;
    async fn delete(&self, id: RouteId) -> Result<()>;
    async fn list_stops(&self, route_id: RouteId) -> Result<Vec<RouteStopSpecPair>>;
}

/// A pair of (stop order, stop name) for the list_stops return.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteStopSpecPair {
    pub stop_id: RouteStopId,
    pub stop_order: u32,
    pub stop_name: StopName,
}

// =============================================================================
// AssignVehicleRepository
// =============================================================================

#[async_trait]
pub trait AssignVehicleRepository: Send + Sync {
    async fn get(&self, id: AssignVehicleId) -> Result<Option<AssignVehicle>>;
    async fn find(&self, vehicle: VehicleId, year: AcademicYearId)
        -> Result<Option<AssignVehicle>>;
    async fn list_for_vehicle(&self, vehicle: VehicleId) -> Result<Vec<AssignVehicle>>;
    async fn list_for_route(
        &self,
        route: RouteId,
        year: AcademicYearId,
    ) -> Result<Vec<AssignVehicle>>;
    async fn insert(&self, assign: &AssignVehicle) -> Result<()>;
    async fn update(&self, assign: &AssignVehicle) -> Result<()>;
    async fn delete(&self, id: AssignVehicleId) -> Result<()>;
    async fn list_members(&self, assign_vehicle_id: AssignVehicleId) -> Result<Vec<StudentId>>;
    async fn add_member(
        &self,
        assign_vehicle_id: AssignVehicleId,
        student_id: StudentId,
    ) -> Result<()>;
    async fn remove_member(
        &self,
        assign_vehicle_id: AssignVehicleId,
        student_id: StudentId,
    ) -> Result<()>;
}

// =============================================================================
// DormitoryRepository
// =============================================================================

#[async_trait]
pub trait DormitoryRepository: Send + Sync {
    async fn get(&self, id: DormitoryId) -> Result<Option<Dormitory>>;
    async fn list(&self, school: SchoolId, year: AcademicYearId) -> Result<Vec<Dormitory>>;
    async fn insert(&self, dorm: &Dormitory) -> Result<()>;
    async fn update(&self, dorm: &Dormitory) -> Result<()>;
    async fn delete(&self, id: DormitoryId) -> Result<()>;
}

// =============================================================================
// RoomRepository
// =============================================================================

#[async_trait]
pub trait RoomRepository: Send + Sync {
    async fn get(&self, id: RoomId) -> Result<Option<Room>>;
    async fn list_for_dormitory(&self, dorm: DormitoryId) -> Result<Vec<Room>>;
    async fn find_by_number(&self, dorm: DormitoryId, number: &str) -> Result<Option<Room>>;
    async fn insert(&self, room: &Room) -> Result<()>;
    async fn update(&self, room: &Room) -> Result<()>;
    async fn delete(&self, id: RoomId) -> Result<()>;
    async fn list_assignments(&self, room: RoomId) -> Result<Vec<StudentId>>;
    async fn current_assignments(&self, room: RoomId) -> Result<Vec<StudentId>>;
    async fn add_assignment(&self, room: RoomId, student_id: StudentId) -> Result<()>;
    async fn release_assignment(&self, room: RoomId, student_id: StudentId) -> Result<()>;
}

// =============================================================================
// RoomTypeRepository
// =============================================================================

#[async_trait]
pub trait RoomTypeRepository: Send + Sync {
    async fn get(&self, id: RoomTypeId) -> Result<Option<RoomType>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<RoomType>>;
    async fn insert(&self, rt: &RoomType) -> Result<()>;
    async fn update(&self, rt: &RoomType) -> Result<()>;
    async fn delete(&self, id: RoomTypeId) -> Result<()>;
}

// =============================================================================
// ItemCategoryRepository
// =============================================================================

#[async_trait]
pub trait ItemCategoryRepository: Send + Sync {
    async fn get(&self, id: ItemCategoryId) -> Result<Option<crate::aggregate::ItemCategory>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<crate::aggregate::ItemCategory>>;
    async fn insert(&self, c: &crate::aggregate::ItemCategory) -> Result<()>;
    async fn update(&self, c: &crate::aggregate::ItemCategory) -> Result<()>;
    async fn delete(&self, id: ItemCategoryId) -> Result<()>;
}

// =============================================================================
// ItemRepository
// =============================================================================

#[async_trait]
pub trait ItemRepository: Send + Sync {
    async fn get(&self, id: ItemId) -> Result<Option<Item>>;
    async fn get_by_sku(&self, school: SchoolId, sku: &str) -> Result<Option<Item>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<Item>>;
    async fn list_for_category(
        &self,
        school: SchoolId,
        category: ItemCategoryId,
    ) -> Result<Vec<Item>>;
    async fn insert(&self, item: &Item) -> Result<()>;
    async fn update(&self, item: &Item) -> Result<()>;
    async fn delete(&self, id: ItemId) -> Result<()>;
    /// Atomically adjust the stock by `delta` (positive or
    /// negative). Returns `Err` if the resulting stock would be
    /// negative. Mirrors the spec's PG `SELECT ... FOR UPDATE`
    /// + SQLite write-lock strategy.
    async fn adjust_stock(&self, id: ItemId, delta: i64) -> Result<()>;
}

// =============================================================================
// ItemStoreRepository
// =============================================================================

#[async_trait]
pub trait ItemStoreRepository: Send + Sync {
    async fn get(&self, id: ItemStoreId) -> Result<Option<ItemStore>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<ItemStore>>;
    async fn insert(&self, s: &ItemStore) -> Result<()>;
    async fn update(&self, s: &ItemStore) -> Result<()>;
    async fn delete(&self, id: ItemStoreId) -> Result<()>;
}

// =============================================================================
// ItemIssueRepository
// =============================================================================

#[async_trait]
pub trait ItemIssueRepository: Send + Sync {
    async fn get(&self, id: ItemIssueId) -> Result<Option<ItemIssue>>;
    async fn list_for_item(&self, item: ItemId) -> Result<Vec<ItemIssue>>;
    async fn list_for_recipient(&self, recipient: IssueRecipient) -> Result<Vec<ItemIssue>>;
    async fn list_overdue(&self, school: SchoolId, as_of: NaiveDate) -> Result<Vec<ItemIssue>>;
    async fn list_open(&self, school: SchoolId) -> Result<Vec<ItemIssue>>;
    async fn insert(&self, issue: &ItemIssue) -> Result<()>;
    async fn update(&self, issue: &ItemIssue) -> Result<()>;
}

// =============================================================================
// ItemReceiveRepository
// =============================================================================

#[async_trait]
pub trait ItemReceiveRepository: Send + Sync {
    async fn get(&self, id: ItemReceiveId) -> Result<Option<ItemReceive>>;
    async fn list_for_supplier(&self, supplier: SupplierId) -> Result<Vec<ItemReceive>>;
    async fn list_for_store(&self, store: ItemStoreId) -> Result<Vec<ItemReceive>>;
    async fn list_for_date_range(
        &self,
        school: SchoolId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<ItemReceive>>;
    async fn insert(&self, receive: &ItemReceive) -> Result<()>;
    async fn update(&self, receive: &ItemReceive) -> Result<()>;
    async fn list_lines(&self, receive: ItemReceiveId) -> Result<Vec<ItemReceiveChild>>;
    async fn insert_line(&self, line: &ItemReceiveChild) -> Result<()>;
    async fn update_line(&self, line: &ItemReceiveChild) -> Result<()>;
    async fn delete_line(&self, line: crate::value_objects::ItemReceiveChildId) -> Result<()>;
}

// =============================================================================
// ItemSellRepository
// =============================================================================

#[async_trait]
pub trait ItemSellRepository: Send + Sync {
    async fn get(&self, id: crate::value_objects::ItemSellId) -> Result<Option<ItemSell>>;
    async fn list_for_buyer(&self, buyer: IssueRecipient) -> Result<Vec<ItemSell>>;
    async fn list_for_date_range(
        &self,
        school: SchoolId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<ItemSell>>;
    async fn list_open(&self, school: SchoolId) -> Result<Vec<ItemSell>>;
    async fn insert(&self, sell: &ItemSell) -> Result<()>;
    async fn update(&self, sell: &ItemSell) -> Result<()>;
    async fn list_lines(
        &self,
        sell: crate::value_objects::ItemSellId,
    ) -> Result<Vec<ItemSellChild>>;
    async fn insert_line(&self, line: &ItemSellChild) -> Result<()>;
    async fn update_line(&self, line: &ItemSellChild) -> Result<()>;
    async fn delete_line(&self, line: crate::value_objects::ItemSellChildId) -> Result<()>;
}

// =============================================================================
// SupplierRepository
// =============================================================================

#[async_trait]
pub trait SupplierRepository: Send + Sync {
    async fn get(&self, id: SupplierId) -> Result<Option<Supplier>>;
    async fn find_by_name(&self, school: SchoolId, name: &str) -> Result<Option<Supplier>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<Supplier>>;
    async fn list_active(&self, school: SchoolId) -> Result<Vec<Supplier>>;
    async fn insert(&self, s: &Supplier) -> Result<()>;
    async fn update(&self, s: &Supplier) -> Result<()>;
    async fn delete(&self, id: SupplierId) -> Result<()>;
}

// =============================================================================
// 14. ItemReceiveChildRepository
// =============================================================================

/// Repository port for the `ItemReceiveChild` line aggregate.
/// Mirrors `ItemReceiveRepository::list_lines` / `insert_line`
/// / `update_line` / `delete_line` so adapters may implement
/// either the parent-only API or the dedicated child API.
#[async_trait]
pub trait ItemReceiveChildRepository: Send + Sync {
    async fn get(
        &self,
        id: crate::value_objects::ItemReceiveChildId,
    ) -> Result<Option<ItemReceiveChild>>;
    async fn list_for_receive(&self, receive: ItemReceiveId) -> Result<Vec<ItemReceiveChild>>;
    async fn insert(&self, line: &ItemReceiveChild) -> Result<()>;
    async fn update(&self, line: &ItemReceiveChild) -> Result<()>;
    async fn delete(&self, id: crate::value_objects::ItemReceiveChildId) -> Result<()>;
}

// =============================================================================
// 15. ItemSellChildRepository
// =============================================================================

/// Repository port for the `ItemSellChild` line aggregate.
/// Mirrors `ItemSellRepository::list_lines` / `insert_line` /
/// `update_line` / `delete_line` so adapters may implement
/// either the parent-only API or the dedicated child API.
#[async_trait]
pub trait ItemSellChildRepository: Send + Sync {
    async fn get(&self, id: crate::value_objects::ItemSellChildId)
        -> Result<Option<ItemSellChild>>;
    async fn list_for_sell(
        &self,
        sell: crate::value_objects::ItemSellId,
    ) -> Result<Vec<ItemSellChild>>;
    async fn insert(&self, line: &ItemSellChild) -> Result<()>;
    async fn update(&self, line: &ItemSellChild) -> Result<()>;
    async fn delete(&self, id: crate::value_objects::ItemSellChildId) -> Result<()>;
}

// =============================================================================
// Tests (object-safety smoke tests per the Phase 6 / Phase 7 pattern)
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

    fn _assert_object_safe(_: &dyn VehicleRepository) {}
    fn _assert_object_safe_route(_: &dyn RouteRepository) {}
    fn _assert_object_safe_av(_: &dyn AssignVehicleRepository) {}
    fn _assert_object_safe_dorm(_: &dyn DormitoryRepository) {}
    fn _assert_object_safe_room(_: &dyn RoomRepository) {}
    fn _assert_object_safe_rt(_: &dyn RoomTypeRepository) {}
    fn _assert_object_safe_ic(_: &dyn ItemCategoryRepository) {}
    fn _assert_object_safe_item(_: &dyn ItemRepository) {}
    fn _assert_object_safe_store(_: &dyn ItemStoreRepository) {}
    fn _assert_object_safe_issue(_: &dyn ItemIssueRepository) {}
    fn _assert_object_safe_receive(_: &dyn ItemReceiveRepository) {}
    fn _assert_object_safe_sell(_: &dyn ItemSellRepository) {}
    fn _assert_object_safe_supplier(_: &dyn SupplierRepository) {}
    fn _assert_object_safe_receive_child(_: &dyn ItemReceiveChildRepository) {}
    fn _assert_object_safe_sell_child(_: &dyn ItemSellChildRepository) {}
}
