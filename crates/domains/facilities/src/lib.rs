//! # educore-facilities
//!
//! Transport vehicles and routes, dormitories and rooms, inventory
//! items and movements, suppliers.
//!
//! This crate is a member of the Educore workspace. See
//! `docs/architecture.md` and the domain spec in
//! `docs/specs/facilities/` for behavioral details.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(unused_imports)]

/// Package name constant. Re-exported so consumers can assert they
/// are using the right crate version at compile time.
pub const PACKAGE_NAME: &str = "educore-facilities";

/// Package version at compile time.
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod value_objects;

mod aggregate;
pub mod commands;
mod entities;
mod errors;
pub mod events;
pub mod query;
mod repository;
pub mod services;

// Prelude: re-export the engine-wide types the facilities services
// reach for, plus the headline symbols.
#[allow(missing_docs)]
pub mod prelude {
    pub use chrono::NaiveDate;
    pub use educore_core::clock::{Clock, IdGenerator, SystemClock, SystemIdGen};
    pub use educore_core::error::{DomainError, Result};
    pub use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
    pub use educore_core::tenant::TenantContext;
    pub use educore_core::value_objects::{Etag, Timestamp, Version};
    pub use educore_events::domain_event::DomainEvent;
    pub use educore_events::envelope::EventEnvelope;
    pub use educore_rbac::value_objects::Capability;

    // Headline 14 aggregate roots (incl. Route)
    pub use crate::aggregate::{
        AssignVehicle, Dormitory, Item, ItemIssue, ItemReceive, ItemReceiveChild, ItemSell,
        ItemSellChild, ItemStore, Room, RoomType, Route, Supplier, Vehicle,
    };

    // Headline 10 child entities
    pub use crate::entities::{
        DormitoryNote, DriverAssignment, ItemIssueLine, ItemReceiveLine, ItemSellLine,
        RoomAssignment, RouteStop, StoreStocktake, SupplierContact, TransportMembership,
    };

    // Headline 49 events (per docs/specs/facilities/events.md).
    pub use crate::events::{
        DormitoryCreated, DormitoryDeleted, DormitoryUpdated, DriverAssignedToVehicle,
        IssuedItemReturned, ItemCategoryCreated, ItemCategoryDeleted, ItemCategoryUpdated,
        ItemCreated, ItemDeleted, ItemIssueStatusUpdated, ItemIssued, ItemReceiveCancelled,
        ItemReceiveUpdated, ItemReceived, ItemSellCancelled, ItemSellRefunded, ItemSellUpdated,
        ItemSold, ItemStoreCreated, ItemStoreDeleted, ItemStoreUpdated, ItemUpdated, RoomCreated,
        RoomDeleted, RoomTypeCreated, RoomTypeDeleted, RoomTypeUpdated, RoomUpdated, RouteCreated,
        RouteDeleted, RouteUpdated, StopAddedToRoute, StopRemovedFromRoute, StopUpdatedOnRoute,
        StudentAssignedToRoom, StudentAssignedToRoute, StudentUnassignedFromRoom,
        StudentUnassignedFromRoute, SupplierCreated, SupplierDeactivated, SupplierDeleted,
        SupplierUpdated, VehicleAssigned, VehicleCreated, VehicleDeactivated, VehicleDeleted,
        VehicleUnassigned, VehicleUpdated,
    };

    // 13 query stubs
    pub use crate::query::{
        AssignVehicleQuery, DormitoryQuery, ItemCategoryQuery, ItemIssueQuery, ItemQuery,
        ItemReceiveQuery, ItemSellQuery, ItemStoreQuery, RoomQuery, RoomTypeQuery, RouteQuery,
        SupplierQuery, VehicleQuery,
    };

    // 13 repository ports
    pub use crate::repository::{
        AssignVehicleRepository, DormitoryRepository, ItemCategoryRepository, ItemIssueRepository,
        ItemReceiveRepository, ItemRepository, ItemSellRepository, ItemStoreRepository,
        RoomRepository, RoomTypeRepository, RouteRepository, RouteStopSpecPair, SupplierRepository,
        VehicleRepository,
    };

    // Service factories (49 per docs/specs/facilities/commands.md).
    pub use crate::services::{
        add_stop_to_route, assign_driver, assign_student_to_room, assign_student_to_route,
        assign_vehicle_to_route, cancel_item_receive, cancel_item_sell, create_dormitory,
        create_item, create_item_category, create_item_store, create_room, create_room_type,
        create_route, create_supplier, create_vehicle, deactivate_supplier, deactivate_vehicle,
        delete_dormitory, delete_item, delete_item_category, delete_item_store, delete_room,
        delete_room_type, delete_route, delete_supplier, delete_vehicle, issue_item, receive_item,
        refund_item_sell, remove_stop_from_route, return_issued_item, sell_item,
        unassign_student_from_room, unassign_student_from_route, unassign_vehicle_from_route,
        update_dormitory, update_issue_status, update_item, update_item_category,
        update_item_receive, update_item_sell, update_item_store, update_room, update_room_type,
        update_route, update_stop_on_route, update_supplier, update_vehicle, DormitoryService,
        InventoryConservationService, InventoryService, MovementKind, MovementRow,
        ReceiveItemResult, SellItemResult, SupplierService, TransportService,
    };

    // Command shapes
    pub use crate::commands::{
        AssignDriverToVehicleCommand, AssignStudentToRoomCommand, AssignStudentToRouteCommand,
        AssignVehicleToRouteCommand, CreateDormitoryCommand, CreateItemCategoryCommand,
        CreateItemCommand, CreateItemStoreCommand, CreateRoomCommand, CreateRoomTypeCommand,
        CreateRouteCommand, CreateSupplierCommand, CreateVehicleCommand, DeactivateSupplierCommand,
        DeactivateVehicleCommand, DeleteDormitoryCommand, DeleteItemCategoryCommand,
        DeleteItemCommand, DeleteItemStoreCommand, DeleteRoomCommand, DeleteRoomTypeCommand,
        DeleteRouteCommand, DeleteSupplierCommand, DeleteVehicleCommand, IssueItemCommand,
        ReceiveItemCommand, RefundItemSellCommand, ReturnIssuedItemCommand, SellItemCommand,
        UnassignStudentFromRoomCommand, UnassignStudentFromRouteCommand,
        UnassignVehicleFromRouteCommand, UpdateDormitoryCommand, UpdateIssueStatusCommand,
        UpdateItemCategoryCommand, UpdateItemCommand, UpdateItemReceiveCommand,
        UpdateItemSellCommand, UpdateItemStoreCommand, UpdateRoomCommand, UpdateRoomTypeCommand,
        UpdateRouteCommand, UpdateStopOnRouteCommand, UpdateSupplierCommand, UpdateVehicleCommand,
    };

    // Typed ids + value objects
    pub use crate::value_objects::{
        AcademicYearId, ActiveStatus, Address, AssignVehicleId, BedNumber, CategoryName,
        ContactPersonName, CostPerBed, Description, DormitoryId, DormitoryName, DormitoryType,
        EmailAddress, Fare, Intake, IssueRecipient, IssueStatus, ItemCategoryId, ItemId,
        ItemIssueId, ItemName, ItemQuantity, ItemReceiveChildId, ItemReceiveId,
        ItemReceiveLineSpec, ItemSellChildId, ItemSellId, ItemSellLineSpec, ItemSku, ItemStoreId,
        MadeYear, Note, NumberOfBed, PaidStatus, PaymentMethod, PhoneNumber, ReferenceNumber,
        RoleId, RoomAssignmentId, RoomId, RoomNumber, RoomTypeId, RoomTypeName, RouteId, RouteName,
        RouteStopId, RouteStopSpec, SellPrice, StaffId, StockOnHand, StopName, StoreName,
        StoreNumber, StoreStocktakeId, StudentId, SupplierId, SupplierName, SupplierStatus,
        UnitPrice, VehicleId, VehicleModel, VehicleNumber, VehicleStatus,
    };

    // Errors
    pub use crate::errors::FacilitiesError;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn package_metadata_is_set() {
        assert_eq!(PACKAGE_NAME, "educore-facilities");
        assert!(!PACKAGE_VERSION.is_empty());
    }
}
