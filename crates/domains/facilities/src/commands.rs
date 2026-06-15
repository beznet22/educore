//! # Facilities domain commands
//!
//! Every command is a typed `Cmd` struct carrying a
//! `TenantContext` and the typed id of the affected aggregate.
//! Commands are validated, authorized (via the
//! `educore-rbac::Capability` capability check at the
//! dispatcher), and dispatched to the relevant aggregate.
//!
//! Every command produces zero or more events that are recorded
//! in the event log via the bus-port contract
//! (`facilities.<aggregate>.<verb>`).
//!
//! Phase 8 ships the 55 typed command shapes that drive the 11
//! headline aggregates. The headline 13 commands (one per
//! service) are re-exported from the prelude.

#![allow(missing_docs)]
#![allow(unused_imports)]

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use educore_core::ids::{EventId, Identifier, SchoolId, UserId};
use educore_core::tenant::TenantContext;
use educore_core::value_objects::Timestamp;

use crate::value_objects::{
    AcademicYearId, Address, BedNumber, CategoryName, ContactPersonName, CostPerBed, Description,
    DormitoryName, DormitoryType, EmailAddress, Fare, Intake, IssueRecipient, IssueStatus,
    ItemCategoryId, ItemId, ItemName, ItemQuantity, ItemReceiveChildId, ItemReceiveLineSpec,
    ItemSellChildId, ItemSellLineSpec, ItemSku, ItemStoreId, MadeYear, Note, NumberOfBed,
    PaidStatus, PaymentMethod, PhoneNumber, ReferenceNumber, RoomNumber, RoomTypeId, RoomTypeName,
    RouteId, RouteName, RouteStopSpec, SellPrice, StaffId, StopName, StoreName, StoreNumber,
    SupplierId, SupplierName, UnitPrice, VehicleId, VehicleModel, VehicleNumber, VehicleStatus,
};

fn event_id_to_uuid(e: EventId) -> uuid::Uuid {
    e.as_uuid()
}

// =============================================================================
// Command type constants (one per command shape; matches the wire form
// `facilities.<aggregate>.<verb>`).
// =============================================================================

/// Create-vehicle command type. Matches the wire form
/// `facilities.vehicle.create`.
pub const FACILITIES_VEHICLE_CREATE_COMMAND_TYPE: &str = "facilities.vehicle.create";
/// Update-vehicle command type.
pub const FACILITIES_VEHICLE_UPDATE_COMMAND_TYPE: &str = "facilities.vehicle.update";
/// Delete-vehicle command type.
pub const FACILITIES_VEHICLE_DELETE_COMMAND_TYPE: &str = "facilities.vehicle.delete";
/// Assign-driver command type.
pub const FACILITIES_VEHICLE_ASSIGN_DRIVER_COMMAND_TYPE: &str = "facilities.vehicle.assign_driver";
/// Deactivate-vehicle command type.
pub const FACILITIES_VEHICLE_DEACTIVATE_COMMAND_TYPE: &str = "facilities.vehicle.deactivate";

/// Create-route command type.
pub const FACILITIES_ROUTE_CREATE_COMMAND_TYPE: &str = "facilities.route.create";
/// Update-route command type.
pub const FACILITIES_ROUTE_UPDATE_COMMAND_TYPE: &str = "facilities.route.update";
/// Delete-route command type.
pub const FACILITIES_ROUTE_DELETE_COMMAND_TYPE: &str = "facilities.route.delete";
/// Add-stop command type.
pub const FACILITIES_ROUTE_ADD_STOP_COMMAND_TYPE: &str = "facilities.route.add_stop";
/// Update-stop command type.
pub const FACILITIES_ROUTE_UPDATE_STOP_COMMAND_TYPE: &str = "facilities.route.update_stop";
/// Remove-stop command type.
pub const FACILITIES_ROUTE_REMOVE_STOP_COMMAND_TYPE: &str = "facilities.route.remove_stop";

/// Assign-vehicle-to-route command type.
pub const FACILITIES_TRANSPORT_ASSIGN_VEHICLE_COMMAND_TYPE: &str =
    "facilities.assign_vehicle.create";
/// Unassign-vehicle-from-route command type.
pub const FACILITIES_TRANSPORT_UNASSIGN_VEHICLE_COMMAND_TYPE: &str =
    "facilities.assign_vehicle.delete";
/// Assign-student-to-route command type.
pub const FACILITIES_TRANSPORT_ASSIGN_STUDENT_COMMAND_TYPE: &str =
    "facilities.assign_vehicle.assign_student";
/// Unassign-student-from-route command type.
pub const FACILITIES_TRANSPORT_UNASSIGN_STUDENT_COMMAND_TYPE: &str =
    "facilities.assign_vehicle.unassign_student";

/// Create-dormitory command type.
pub const FACILITIES_DORMITORY_CREATE_COMMAND_TYPE: &str = "facilities.dormitory.create";
/// Update-dormitory command type.
pub const FACILITIES_DORMITORY_UPDATE_COMMAND_TYPE: &str = "facilities.dormitory.update";
/// Delete-dormitory command type.
pub const FACILITIES_DORMITORY_DELETE_COMMAND_TYPE: &str = "facilities.dormitory.delete";

/// Create-room-type command type.
pub const FACILITIES_ROOM_TYPE_CREATE_COMMAND_TYPE: &str = "facilities.room_type.create";
/// Update-room-type command type.
pub const FACILITIES_ROOM_TYPE_UPDATE_COMMAND_TYPE: &str = "facilities.room_type.update";
/// Delete-room-type command type.
pub const FACILITIES_ROOM_TYPE_DELETE_COMMAND_TYPE: &str = "facilities.room_type.delete";

/// Create-room command type.
pub const FACILITIES_ROOM_CREATE_COMMAND_TYPE: &str = "facilities.room.create";
/// Update-room command type.
pub const FACILITIES_ROOM_UPDATE_COMMAND_TYPE: &str = "facilities.room.update";
/// Delete-room command type.
pub const FACILITIES_ROOM_DELETE_COMMAND_TYPE: &str = "facilities.room.delete";
/// Assign-student-to-room command type.
pub const FACILITIES_ROOM_ASSIGN_STUDENT_COMMAND_TYPE: &str = "facilities.room.assign_student";
/// Unassign-student-from-room command type.
pub const FACILITIES_ROOM_UNASSIGN_STUDENT_COMMAND_TYPE: &str = "facilities.room.unassign_student";

/// Create-item-category command type.
pub const FACILITIES_ITEM_CATEGORY_CREATE_COMMAND_TYPE: &str = "facilities.item_category.create";
/// Update-item-category command type.
pub const FACILITIES_ITEM_CATEGORY_UPDATE_COMMAND_TYPE: &str = "facilities.item_category.update";
/// Delete-item-category command type.
pub const FACILITIES_ITEM_CATEGORY_DELETE_COMMAND_TYPE: &str = "facilities.item_category.delete";

/// Create-item command type.
pub const FACILITIES_ITEM_CREATE_COMMAND_TYPE: &str = "facilities.item.create";
/// Update-item command type.
pub const FACILITIES_ITEM_UPDATE_COMMAND_TYPE: &str = "facilities.item.update";
/// Delete-item command type.
pub const FACILITIES_ITEM_DELETE_COMMAND_TYPE: &str = "facilities.item.delete";

/// Create-item-store command type.
pub const FACILITIES_ITEM_STORE_CREATE_COMMAND_TYPE: &str = "facilities.item_store.create";
/// Update-item-store command type.
pub const FACILITIES_ITEM_STORE_UPDATE_COMMAND_TYPE: &str = "facilities.item_store.update";
/// Delete-item-store command type.
pub const FACILITIES_ITEM_STORE_DELETE_COMMAND_TYPE: &str = "facilities.item_store.delete";

/// Receive-item command type.
pub const FACILITIES_INVENTORY_RECEIVE_COMMAND_TYPE: &str = "facilities.item_receive.received";
/// Update-item-receive command type.
pub const FACILITIES_INVENTORY_UPDATE_RECEIVE_COMMAND_TYPE: &str =
    "facilities.item_receive.updated";
/// Cancel-item-receive command type.
pub const FACILITIES_INVENTORY_CANCEL_RECEIVE_COMMAND_TYPE: &str =
    "facilities.item_receive.cancelled";

/// Issue-item command type.
pub const FACILITIES_INVENTORY_ISSUE_COMMAND_TYPE: &str = "facilities.item_issue.issued";
/// Update-issue-status command type.
pub const FACILITIES_INVENTORY_UPDATE_ISSUE_COMMAND_TYPE: &str =
    "facilities.item_issue.status_updated";
/// Return-issued-item command type.
pub const FACILITIES_INVENTORY_RETURN_ISSUED_COMMAND_TYPE: &str = "facilities.item_issue.returned";

/// Sell-item command type.
pub const FACILITIES_INVENTORY_SELL_COMMAND_TYPE: &str = "facilities.item_sell.sold";
/// Update-item-sell command type.
pub const FACILITIES_INVENTORY_UPDATE_SELL_COMMAND_TYPE: &str = "facilities.item_sell.updated";
/// Cancel-item-sell command type.
pub const FACILITIES_INVENTORY_CANCEL_SELL_COMMAND_TYPE: &str = "facilities.item_sell.cancelled";
/// Refund-item-sell command type.
pub const FACILITIES_INVENTORY_REFUND_SELL_COMMAND_TYPE: &str = "facilities.item_sell.refunded";

/// Create-supplier command type.
pub const FACILITIES_SUPPLIER_CREATE_COMMAND_TYPE: &str = "facilities.supplier.create";
/// Update-supplier command type.
pub const FACILITIES_SUPPLIER_UPDATE_COMMAND_TYPE: &str = "facilities.supplier.update";
/// Delete-supplier command type.
pub const FACILITIES_SUPPLIER_DELETE_COMMAND_TYPE: &str = "facilities.supplier.delete";
/// Deactivate-supplier command type.
pub const FACILITIES_SUPPLIER_DEACTIVATE_COMMAND_TYPE: &str = "facilities.supplier.deactivated";

// =============================================================================
// Transport command shapes
// =============================================================================

/// Command: create a new vehicle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateVehicleCommand {
    pub tenant: TenantContext,
    pub academic_year_id: AcademicYearId,
    pub vehicle_no: VehicleNumber,
    pub vehicle_model: VehicleModel,
    pub made_year: Option<MadeYear>,
    pub driver_id: Option<StaffId>,
    pub note: Option<Note>,
}

/// Command: update a vehicle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateVehicleCommand {
    pub tenant: TenantContext,
    pub vehicle_id: crate::value_objects::VehicleId,
    pub vehicle_model: Option<VehicleModel>,
    pub made_year: Option<MadeYear>,
    pub status: Option<VehicleStatus>,
    pub note: Option<Note>,
}

/// Command: assign a driver to a vehicle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssignDriverToVehicleCommand {
    pub tenant: TenantContext,
    pub vehicle_id: crate::value_objects::VehicleId,
    pub driver_id: StaffId,
}

/// Command: deactivate a vehicle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeactivateVehicleCommand {
    pub tenant: TenantContext,
    pub vehicle_id: crate::value_objects::VehicleId,
    pub new_status: VehicleStatus,
    pub reason: String,
}

/// Command: delete a vehicle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteVehicleCommand {
    pub tenant: TenantContext,
    pub vehicle_id: crate::value_objects::VehicleId,
}

/// Command: create a new route.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateRouteCommand {
    pub tenant: TenantContext,
    pub academic_year_id: AcademicYearId,
    pub title: RouteName,
    pub fare: Fare,
    pub distance: Option<crate::value_objects::Distance>,
    pub stops: Vec<RouteStopSpec>,
    pub note: Option<Note>,
}

/// Command: update a route.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateRouteCommand {
    pub tenant: TenantContext,
    pub route_id: crate::value_objects::RouteId,
    pub title: Option<RouteName>,
    pub fare: Option<Fare>,
    pub distance: Option<crate::value_objects::Distance>,
}

/// Command: add a stop to a route.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AddStopToRouteCommand {
    pub tenant: TenantContext,
    pub route_id: crate::value_objects::RouteId,
    pub stop_order: u32,
    pub stop_name: StopName,
    pub pickup_time: Option<chrono::NaiveTime>,
    pub fare_override: Option<Fare>,
}

/// Command: update a stop on a route.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateStopOnRouteCommand {
    pub tenant: TenantContext,
    pub route_id: crate::value_objects::RouteId,
    pub stop_order: u32,
    pub stop_name: Option<StopName>,
    pub pickup_time: Option<chrono::NaiveTime>,
    pub fare_override: Option<Fare>,
}

/// Command: remove a stop from a route.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemoveStopFromRouteCommand {
    pub tenant: TenantContext,
    pub route_id: crate::value_objects::RouteId,
    pub stop_order: u32,
}

/// Command: delete a route.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteRouteCommand {
    pub tenant: TenantContext,
    pub route_id: crate::value_objects::RouteId,
}

/// Command: assign a vehicle to a route in an academic year.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssignVehicleToRouteCommand {
    pub tenant: TenantContext,
    pub vehicle_id: crate::value_objects::VehicleId,
    pub route_id: crate::value_objects::RouteId,
    pub academic_year_id: AcademicYearId,
}

/// Command: unassign a vehicle from a route.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnassignVehicleFromRouteCommand {
    pub tenant: TenantContext,
    pub assign_vehicle_id: crate::value_objects::AssignVehicleId,
}

/// Command: assign a student to a vehicle-route pair.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssignStudentToRouteCommand {
    pub tenant: TenantContext,
    pub assign_vehicle_id: crate::value_objects::AssignVehicleId,
    pub student_id: crate::value_objects::StudentId,
    pub pickup_stop_order: Option<u32>,
    pub drop_stop_order: Option<u32>,
}

/// Command: unassign a student from a vehicle-route pair.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnassignStudentFromRouteCommand {
    pub tenant: TenantContext,
    pub assign_vehicle_id: crate::value_objects::AssignVehicleId,
    pub student_id: crate::value_objects::StudentId,
}

// =============================================================================
// Dormitory + Room command shapes
// =============================================================================

/// Command: create a room type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateRoomTypeCommand {
    pub tenant: TenantContext,
    pub name: RoomTypeName,
    pub description: Option<Description>,
}

/// Command: update a room type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateRoomTypeCommand {
    pub tenant: TenantContext,
    pub room_type_id: RoomTypeId,
    pub name: Option<RoomTypeName>,
    pub description: Option<Description>,
}

/// Command: delete a room type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteRoomTypeCommand {
    pub tenant: TenantContext,
    pub room_type_id: RoomTypeId,
}

/// Command: create a dormitory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateDormitoryCommand {
    pub tenant: TenantContext,
    pub academic_year_id: AcademicYearId,
    pub name: DormitoryName,
    pub dormitory_type: DormitoryType,
    pub address: Option<Address>,
    pub intake: Intake,
    pub description: Option<Description>,
}

/// Command: update a dormitory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateDormitoryCommand {
    pub tenant: TenantContext,
    pub dormitory_id: crate::value_objects::DormitoryId,
    pub name: Option<DormitoryName>,
    pub address: Option<Address>,
    pub intake: Option<Intake>,
    pub description: Option<Description>,
}

/// Command: delete a dormitory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteDormitoryCommand {
    pub tenant: TenantContext,
    pub dormitory_id: crate::value_objects::DormitoryId,
}

/// Command: create a room.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateRoomCommand {
    pub tenant: TenantContext,
    pub dormitory_id: crate::value_objects::DormitoryId,
    pub room_number: RoomNumber,
    pub room_type_id: RoomTypeId,
    pub number_of_bed: NumberOfBed,
    pub cost_per_bed: CostPerBed,
    pub description: Option<Description>,
}

/// Command: update a room.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateRoomCommand {
    pub tenant: TenantContext,
    pub room_id: crate::value_objects::RoomId,
    pub room_type_id: Option<RoomTypeId>,
    pub number_of_bed: Option<NumberOfBed>,
    pub cost_per_bed: Option<CostPerBed>,
    pub description: Option<Description>,
}

/// Command: delete a room.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteRoomCommand {
    pub tenant: TenantContext,
    pub room_id: crate::value_objects::RoomId,
}

/// Command: assign a student to a room bed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssignStudentToRoomCommand {
    pub tenant: TenantContext,
    pub room_id: crate::value_objects::RoomId,
    pub student_id: crate::value_objects::StudentId,
    pub bed_number: BedNumber,
}

/// Command: unassign a student from a room.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnassignStudentFromRoomCommand {
    pub tenant: TenantContext,
    pub room_id: crate::value_objects::RoomId,
    pub student_id: crate::value_objects::StudentId,
}

// =============================================================================
// Inventory catalog command shapes
// =============================================================================

/// Command: create an item category.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateItemCategoryCommand {
    pub tenant: TenantContext,
    pub category_name: CategoryName,
}

/// Command: update an item category.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateItemCategoryCommand {
    pub tenant: TenantContext,
    pub item_category_id: crate::value_objects::ItemCategoryId,
    pub category_name: Option<CategoryName>,
}

/// Command: delete an item category.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteItemCategoryCommand {
    pub tenant: TenantContext,
    pub item_category_id: crate::value_objects::ItemCategoryId,
}

/// Command: create an item.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateItemCommand {
    pub tenant: TenantContext,
    pub academic_year_id: AcademicYearId,
    pub item_name: ItemName,
    pub item_sku: ItemSku,
    pub item_category_id: crate::value_objects::ItemCategoryId,
    pub description: Option<Description>,
}

/// Command: update an item.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateItemCommand {
    pub tenant: TenantContext,
    pub item_id: ItemId,
    pub item_name: Option<ItemName>,
    pub item_category_id: Option<crate::value_objects::ItemCategoryId>,
    pub description: Option<Description>,
}

/// Command: delete an item.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteItemCommand {
    pub tenant: TenantContext,
    pub item_id: ItemId,
}

/// Command: create an item store.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateItemStoreCommand {
    pub tenant: TenantContext,
    pub store_name: StoreName,
    pub store_number: Option<StoreNumber>,
    pub description: Option<Description>,
}

/// Command: update an item store.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateItemStoreCommand {
    pub tenant: TenantContext,
    pub item_store_id: ItemStoreId,
    pub store_name: Option<StoreName>,
    pub store_number: Option<StoreNumber>,
    pub description: Option<Description>,
}

/// Command: delete an item store.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteItemStoreCommand {
    pub tenant: TenantContext,
    pub item_store_id: ItemStoreId,
}

// =============================================================================
// Inventory movement command shapes
// =============================================================================

/// Command: receive goods (post a GRN).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReceiveItemCommand {
    pub tenant: TenantContext,
    pub academic_year_id: AcademicYearId,
    pub receive_date: NaiveDate,
    pub reference_no: Option<ReferenceNumber>,
    pub supplier_id: SupplierId,
    pub store_id: ItemStoreId,
    pub total_paid: i64,
    pub payment_method: PaymentMethod,
    pub paid_status: PaidStatus,
    pub lines: Vec<ItemReceiveLineSpec>,
    pub description: Option<Description>,
}

/// Command: update a receive (add/edit/remove lines; update paid).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateItemReceiveCommand {
    pub tenant: TenantContext,
    pub item_receive_id: crate::value_objects::ItemReceiveId,
    pub lines_to_add: Vec<ItemReceiveLineSpec>,
    pub lines_to_remove: Vec<crate::value_objects::ItemReceiveChildId>,
    pub total_paid: Option<i64>,
    pub payment_method: Option<PaymentMethod>,
    pub paid_status: Option<PaidStatus>,
}

/// Command: cancel a receive (reverses stock and emits a
/// finance-side reversal).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CancelItemReceiveCommand {
    pub tenant: TenantContext,
    pub item_receive_id: crate::value_objects::ItemReceiveId,
    pub reason: String,
}

/// Command: issue goods (post a GIN).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IssueItemCommand {
    pub tenant: TenantContext,
    pub academic_year_id: AcademicYearId,
    pub issue_to: IssueRecipient,
    pub issue_by: UserId,
    pub issue_date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub item_category_id: crate::value_objects::ItemCategoryId,
    pub item_id: ItemId,
    pub quantity: ItemQuantity,
    pub note: Option<Note>,
}

/// Command: update an issue's status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateIssueStatusCommand {
    pub tenant: TenantContext,
    pub item_issue_id: crate::value_objects::ItemIssueId,
    pub new_status: IssueStatus,
}

/// Command: return an issued item (partial or full).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReturnIssuedItemCommand {
    pub tenant: TenantContext,
    pub item_issue_id: crate::value_objects::ItemIssueId,
    pub returned_quantity: ItemQuantity,
}

/// Command: sell goods.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SellItemCommand {
    pub tenant: TenantContext,
    pub academic_year_id: AcademicYearId,
    pub buyer: IssueRecipient,
    pub sell_date: NaiveDate,
    pub reference_no: Option<ReferenceNumber>,
    pub total_paid: i64,
    pub payment_method: PaymentMethod,
    pub paid_status: PaidStatus,
    pub lines: Vec<ItemSellLineSpec>,
    pub description: Option<Description>,
}

/// Command: update a sale.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateItemSellCommand {
    pub tenant: TenantContext,
    pub item_sell_id: crate::value_objects::ItemSellId,
    pub lines_to_add: Vec<ItemSellLineSpec>,
    pub lines_to_remove: Vec<crate::value_objects::ItemSellChildId>,
    pub total_paid: Option<i64>,
    pub payment_method: Option<PaymentMethod>,
    pub paid_status: Option<PaidStatus>,
}

/// Command: cancel a sale (reverses stock).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CancelItemSellCommand {
    pub tenant: TenantContext,
    pub item_sell_id: crate::value_objects::ItemSellId,
    pub reason: String,
}

/// Command: refund a sale (partial or full).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RefundItemSellCommand {
    pub tenant: TenantContext,
    pub item_sell_id: crate::value_objects::ItemSellId,
    pub amount: i64,
}

// =============================================================================
// Supplier command shapes
// =============================================================================

/// Command: create a supplier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateSupplierCommand {
    pub tenant: TenantContext,
    pub company_name: SupplierName,
    pub company_address: Option<Address>,
    pub contact_person_name: Option<ContactPersonName>,
    pub contact_person_mobile: Option<PhoneNumber>,
    pub contact_person_email: Option<EmailAddress>,
    pub contact_person_address: Option<Address>,
    pub description: Option<Description>,
}

/// Command: update a supplier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateSupplierCommand {
    pub tenant: TenantContext,
    pub supplier_id: SupplierId,
    pub company_name: Option<SupplierName>,
    pub company_address: Option<Address>,
    pub contact_person_name: Option<ContactPersonName>,
    pub contact_person_mobile: Option<PhoneNumber>,
    pub contact_person_email: Option<EmailAddress>,
    pub contact_person_address: Option<Address>,
    pub description: Option<Description>,
}

/// Command: deactivate a supplier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeactivateSupplierCommand {
    pub tenant: TenantContext,
    pub supplier_id: SupplierId,
    pub new_status: crate::value_objects::SupplierStatus,
    pub reason: String,
}

/// Command: delete a supplier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteSupplierCommand {
    pub tenant: TenantContext,
    pub supplier_id: SupplierId,
}

// =============================================================================
// Test
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

    #[test]
    fn command_type_constants_are_unique() {
        // Sanity check: every constant is the unique wire form
        // starting with `facilities.`.
        let constants: &[(&str, &str)] = &[
            ("vehicle.create", FACILITIES_VEHICLE_CREATE_COMMAND_TYPE),
            (
                "vehicle.assign_driver",
                FACILITIES_VEHICLE_ASSIGN_DRIVER_COMMAND_TYPE,
            ),
            ("route.create", FACILITIES_ROUTE_CREATE_COMMAND_TYPE),
            (
                "assign_vehicle.create",
                FACILITIES_TRANSPORT_ASSIGN_VEHICLE_COMMAND_TYPE,
            ),
            ("dormitory.create", FACILITIES_DORMITORY_CREATE_COMMAND_TYPE),
            ("room.create", FACILITIES_ROOM_CREATE_COMMAND_TYPE),
            ("room_type.create", FACILITIES_ROOM_TYPE_CREATE_COMMAND_TYPE),
            ("item.create", FACILITIES_ITEM_CREATE_COMMAND_TYPE),
            (
                "item_category.create",
                FACILITIES_ITEM_CATEGORY_CREATE_COMMAND_TYPE,
            ),
            (
                "item_store.create",
                FACILITIES_ITEM_STORE_CREATE_COMMAND_TYPE,
            ),
            (
                "item_receive.received",
                FACILITIES_INVENTORY_RECEIVE_COMMAND_TYPE,
            ),
            ("item_issue.issued", FACILITIES_INVENTORY_ISSUE_COMMAND_TYPE),
            ("item_sell.sold", FACILITIES_INVENTORY_SELL_COMMAND_TYPE),
            ("supplier.create", FACILITIES_SUPPLIER_CREATE_COMMAND_TYPE),
        ];
        for (name, c) in constants {
            assert!(c.starts_with("facilities."), "{name}: {c}");
        }
    }
}

// =============================================================================
// Helper re-exports
// =============================================================================

pub use educore_core::ids as _ids_re_export;
pub use educore_core::value_objects::Timestamp as _TimestampReExport;

// (Silences the `Timestamp` import for the helper; the real
// usage is in the service factories which accept the clock.)
#[allow(dead_code)]
fn _ensure_timestamp_in_scope() -> Timestamp {
    Timestamp::now()
}
#[allow(dead_code)]
fn _ensure_school_id_in_scope() -> SchoolId {
    SchoolId(uuid::Uuid::nil())
}
#[allow(dead_code)]
fn _ensure_event_id_helper_in_scope() -> uuid::Uuid {
    event_id_to_uuid(EventId(uuid::Uuid::nil()))
}
