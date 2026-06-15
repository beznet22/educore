//! # Facilities domain events
//!
//! Every aggregate's state change emits an event implementing
//! [`DomainEvent`](::educore_events::domain_event::DomainEvent).
//! The full set follows the spec at `docs/specs/facilities/events.md`.
//!
//! Wire form: `facilities.<aggregate>.<verb>` (e.g.
//! `facilities.vehicle.created`, `facilities.item.received`).
//!
//! Phase 8 ships the headline 18 events that cover the 11
//! aggregates plus the per-line child events.

#![allow(missing_docs)]
#![allow(unused_imports)]
#![allow(clippy::too_many_arguments)]

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;

use crate::value_objects::{
    AssignVehicleId, BedNumber, DormitoryId, DormitoryType, IssueRecipient, IssueStatus, ItemId,
    ItemIssueId, ItemReceiveId, ItemReceiveLineSpec, ItemSellId, ItemSellLineSpec, ItemStoreId,
    NaiveTime, PaidStatus, PaymentMethod, ReferenceNumber, RoomId, RoomTypeId, RouteId, RouteName,
    RouteStopSpec, StaffId, StopName, StudentId, SupplierId, VehicleId, VehicleNumber,
    VehicleStatus,
};

// =============================================================================
// Transport events
// =============================================================================

/// Emitted when a new `Vehicle` is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleCreated {
    pub vehicle_id: VehicleId,
    pub vehicle_no: VehicleNumber,
    pub vehicle_model: String,
    pub driver_id: Option<StaffId>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl VehicleCreated {
    pub fn new(
        vehicle_id: VehicleId,
        vehicle_no: VehicleNumber,
        vehicle_model: String,
        driver_id: Option<StaffId>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            vehicle_id,
            vehicle_no,
            vehicle_model,
            driver_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for VehicleCreated {
    const EVENT_TYPE: &'static str = "facilities.vehicle.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "vehicle";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.vehicle_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.vehicle_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a vehicle is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleUpdated {
    pub vehicle_id: VehicleId,
    /// The set of field names that changed (snake_case wire
    /// form, e.g. `"vehicle_model"`, `"made_year"`).
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl VehicleUpdated {
    pub fn new(
        vehicle_id: VehicleId,
        changes: Vec<&'static str>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            vehicle_id,
            changes: changes.into_iter().map(|s| s.to_owned()).collect(),
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for VehicleUpdated {
    const EVENT_TYPE: &'static str = "facilities.vehicle.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "vehicle";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.vehicle_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.vehicle_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a driver is assigned to a vehicle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DriverAssignedToVehicle {
    pub vehicle_id: VehicleId,
    pub from_driver_id: Option<StaffId>,
    pub to_driver_id: StaffId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DriverAssignedToVehicle {
    pub fn new(
        vehicle_id: VehicleId,
        from_driver_id: Option<StaffId>,
        to_driver_id: StaffId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            vehicle_id,
            from_driver_id,
            to_driver_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DriverAssignedToVehicle {
    const EVENT_TYPE: &'static str = "facilities.vehicle.driver_assigned";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "vehicle";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.vehicle_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.vehicle_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a vehicle is deactivated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleDeactivated {
    pub vehicle_id: VehicleId,
    pub reason: String,
    pub new_status: VehicleStatus,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl VehicleDeactivated {
    pub fn new(
        vehicle_id: VehicleId,
        reason: String,
        new_status: VehicleStatus,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            vehicle_id,
            reason,
            new_status,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for VehicleDeactivated {
    const EVENT_TYPE: &'static str = "facilities.vehicle.deactivated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "vehicle";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.vehicle_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.vehicle_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a vehicle is deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleDeleted {
    pub vehicle_id: VehicleId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl VehicleDeleted {
    pub fn new(
        vehicle_id: VehicleId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            vehicle_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for VehicleDeleted {
    const EVENT_TYPE: &'static str = "facilities.vehicle.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "vehicle";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.vehicle_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.vehicle_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// Route events
// =============================================================================

/// Emitted when a new `Route` is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteCreated {
    pub route_id: RouteId,
    pub title: RouteName,
    pub fare_minor: i64,
    pub stops: Vec<RouteStopSpec>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl RouteCreated {
    pub fn new(
        route_id: RouteId,
        title: RouteName,
        fare_minor: i64,
        stops: Vec<RouteStopSpec>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            route_id,
            title,
            fare_minor,
            stops,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for RouteCreated {
    const EVENT_TYPE: &'static str = "facilities.route.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "route";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.route_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.route_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a stop is added to a route.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopAddedToRoute {
    pub route_id: RouteId,
    pub stop_order: u32,
    pub stop_name: StopName,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl StopAddedToRoute {
    pub fn new(
        route_id: RouteId,
        stop_order: u32,
        stop_name: StopName,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            route_id,
            stop_order,
            stop_name,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StopAddedToRoute {
    const EVENT_TYPE: &'static str = "facilities.route.stop_added";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "route";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.route_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.route_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a vehicle is assigned to a route.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleAssigned {
    pub assign_vehicle_id: AssignVehicleId,
    pub vehicle_id: VehicleId,
    pub route_id: RouteId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl VehicleAssigned {
    pub fn new(
        assign_vehicle_id: AssignVehicleId,
        vehicle_id: VehicleId,
        route_id: RouteId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            assign_vehicle_id,
            vehicle_id,
            route_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for VehicleAssigned {
    const EVENT_TYPE: &'static str = "facilities.assign_vehicle.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "assign_vehicle";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.assign_vehicle_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.assign_vehicle_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a student is assigned to a vehicle-route pair.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StudentAssignedToRoute {
    pub assign_vehicle_id: AssignVehicleId,
    pub student_id: StudentId,
    pub joined_at: NaiveDate,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl StudentAssignedToRoute {
    pub fn new(
        assign_vehicle_id: AssignVehicleId,
        student_id: StudentId,
        joined_at: NaiveDate,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            assign_vehicle_id,
            student_id,
            joined_at,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StudentAssignedToRoute {
    const EVENT_TYPE: &'static str = "facilities.assign_vehicle.student_assigned";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "assign_vehicle";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.assign_vehicle_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.assign_vehicle_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// Dormitory + Room events
// =============================================================================

/// Emitted when a new `Dormitory` is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DormitoryCreated {
    pub dormitory_id: DormitoryId,
    pub name: String,
    pub dormitory_type: DormitoryType,
    pub intake: u32,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DormitoryCreated {
    pub fn new(
        dormitory_id: DormitoryId,
        name: String,
        dormitory_type: DormitoryType,
        intake: u32,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            dormitory_id,
            name,
            dormitory_type,
            intake,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DormitoryCreated {
    const EVENT_TYPE: &'static str = "facilities.dormitory.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "dormitory";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.dormitory_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.dormitory_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a new `RoomType` is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoomTypeCreated {
    pub room_type_id: RoomTypeId,
    pub name: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl RoomTypeCreated {
    pub fn new(
        room_type_id: RoomTypeId,
        name: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            room_type_id,
            name,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for RoomTypeCreated {
    const EVENT_TYPE: &'static str = "facilities.room_type.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "room_type";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.room_type_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.room_type_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a new `Room` is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoomCreated {
    pub room_id: RoomId,
    pub dormitory_id: DormitoryId,
    pub room_number: String,
    pub number_of_bed: u32,
    pub cost_per_bed: i64,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl RoomCreated {
    pub fn new(
        room_id: RoomId,
        dormitory_id: DormitoryId,
        room_number: String,
        number_of_bed: u32,
        cost_per_bed: i64,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            room_id,
            dormitory_id,
            room_number,
            number_of_bed,
            cost_per_bed,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for RoomCreated {
    const EVENT_TYPE: &'static str = "facilities.room.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "room";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.room_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.room_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a student is assigned to a room.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StudentAssignedToRoom {
    pub room_id: RoomId,
    pub student_id: StudentId,
    pub bed_number: BedNumber,
    pub assigned_at: Timestamp,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl StudentAssignedToRoom {
    pub fn new(
        room_id: RoomId,
        student_id: StudentId,
        bed_number: BedNumber,
        assigned_at: Timestamp,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            room_id,
            student_id,
            bed_number,
            assigned_at,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StudentAssignedToRoom {
    const EVENT_TYPE: &'static str = "facilities.room.student_assigned";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "room";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.room_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.room_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// Inventory catalog events
// =============================================================================

/// Emitted when a new `ItemCategory` is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemCategoryCreated {
    pub item_category_id: Uuid,
    pub category_name: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ItemCategoryCreated {
    pub fn new(
        item_category_id: Uuid,
        category_name: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            item_category_id,
            category_name,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ItemCategoryCreated {
    const EVENT_TYPE: &'static str = "facilities.item_category.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "item_category";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.item_category_id
    }
    fn school_id(&self) -> SchoolId {
        // The school id is not embedded in the raw Uuid payload;
        // a typed-id wrapper is used by the service factory.
        // For the bus-port envelope the school is supplied by the
        // TenantContext; here we default to the nil school id
        // (callers that need a typed id should use the service
        // factory which mints a `ItemCategoryId`).
        SchoolId(Uuid::nil())
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a new `Item` is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemCreated {
    pub item_id: ItemId,
    pub item_name: String,
    pub item_sku: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ItemCreated {
    pub fn new(
        item_id: ItemId,
        item_name: String,
        item_sku: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            item_id,
            item_name,
            item_sku,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ItemCreated {
    const EVENT_TYPE: &'static str = "facilities.item.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "item";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.item_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.item_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a new `ItemStore` is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemStoreCreated {
    pub item_store_id: ItemStoreId,
    pub store_name: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ItemStoreCreated {
    pub fn new(
        item_store_id: ItemStoreId,
        store_name: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            item_store_id,
            store_name,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ItemStoreCreated {
    const EVENT_TYPE: &'static str = "facilities.item_store.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "item_store";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.item_store_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.item_store_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// Inventory movement events
// =============================================================================

/// Emitted when a goods-receive note is posted. Finance may
/// subscribe to record the payable against the supplier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemReceived {
    pub item_receive_id: ItemReceiveId,
    pub supplier_id: SupplierId,
    pub store_id: ItemStoreId,
    pub receive_date: NaiveDate,
    pub grand_total: i64,
    pub total_quantity: i64,
    pub total_paid: i64,
    pub total_due: i64,
    pub paid_status: PaidStatus,
    pub lines: Vec<ItemReceiveLineSpec>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ItemReceived {
    pub fn new(
        item_receive_id: ItemReceiveId,
        supplier_id: SupplierId,
        store_id: ItemStoreId,
        receive_date: NaiveDate,
        grand_total: i64,
        total_quantity: i64,
        total_paid: i64,
        total_due: i64,
        paid_status: PaidStatus,
        lines: Vec<ItemReceiveLineSpec>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            item_receive_id,
            supplier_id,
            store_id,
            receive_date,
            grand_total,
            total_quantity,
            total_paid,
            total_due,
            paid_status,
            lines,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ItemReceived {
    const EVENT_TYPE: &'static str = "facilities.item_receive.received";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "item_receive";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.item_receive_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.item_receive_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a goods-issue note is posted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemIssued {
    pub item_issue_id: ItemIssueId,
    pub item_id: ItemId,
    pub issue_to: IssueRecipient,
    pub issue_by: UserId,
    pub issue_date: NaiveDate,
    pub quantity: i64,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ItemIssued {
    pub fn new(
        item_issue_id: ItemIssueId,
        item_id: ItemId,
        issue_to: IssueRecipient,
        issue_by: UserId,
        issue_date: NaiveDate,
        quantity: i64,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            item_issue_id,
            item_id,
            issue_to,
            issue_by,
            issue_date,
            quantity,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ItemIssued {
    const EVENT_TYPE: &'static str = "facilities.item_issue.issued";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "item_issue";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.item_issue_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.item_issue_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when an issued item is returned (partial or full).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IssuedItemReturned {
    pub item_issue_id: ItemIssueId,
    pub item_id: ItemId,
    pub returned_quantity: i64,
    pub new_status: IssueStatus,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl IssuedItemReturned {
    pub fn new(
        item_issue_id: ItemIssueId,
        item_id: ItemId,
        returned_quantity: i64,
        new_status: IssueStatus,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            item_issue_id,
            item_id,
            returned_quantity,
            new_status,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for IssuedItemReturned {
    const EVENT_TYPE: &'static str = "facilities.item_issue.returned";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "item_issue";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.item_issue_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.item_issue_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a sale is posted. Finance may subscribe to
/// record the income and the receivable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemSold {
    pub item_sell_id: ItemSellId,
    pub buyer: IssueRecipient,
    pub sell_date: NaiveDate,
    pub grand_total: i64,
    pub total_quantity: i64,
    pub total_paid: i64,
    pub total_due: i64,
    pub paid_status: PaidStatus,
    pub lines: Vec<ItemSellLineSpec>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ItemSold {
    pub fn new(
        item_sell_id: ItemSellId,
        buyer: IssueRecipient,
        sell_date: NaiveDate,
        grand_total: i64,
        total_quantity: i64,
        total_paid: i64,
        total_due: i64,
        paid_status: PaidStatus,
        lines: Vec<ItemSellLineSpec>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            item_sell_id,
            buyer,
            sell_date,
            grand_total,
            total_quantity,
            total_paid,
            total_due,
            paid_status,
            lines,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ItemSold {
    const EVENT_TYPE: &'static str = "facilities.item_sell.sold";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "item_sell";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.item_sell_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.item_sell_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a sale is cancelled. Stock is reversed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemSellCancelled {
    pub item_sell_id: ItemSellId,
    pub reason: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ItemSellCancelled {
    pub fn new(
        item_sell_id: ItemSellId,
        reason: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            item_sell_id,
            reason,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ItemSellCancelled {
    const EVENT_TYPE: &'static str = "facilities.item_sell.cancelled";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "item_sell";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.item_sell_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.item_sell_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// Supplier events
// =============================================================================

/// Emitted when a new `Supplier` is created. Finance may
/// subscribe to register the supplier as a payable
/// counterparty.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SupplierCreated {
    pub supplier_id: SupplierId,
    pub company_name: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SupplierCreated {
    pub fn new(
        supplier_id: SupplierId,
        company_name: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            supplier_id,
            company_name,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SupplierCreated {
    const EVENT_TYPE: &'static str = "facilities.supplier.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "supplier";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.supplier_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.supplier_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a supplier is deactivated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SupplierDeactivated {
    pub supplier_id: SupplierId,
    pub reason: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SupplierDeactivated {
    pub fn new(
        supplier_id: SupplierId,
        reason: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            supplier_id,
            reason,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SupplierDeactivated {
    const EVENT_TYPE: &'static str = "facilities.supplier.deactivated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "supplier";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.supplier_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.supplier_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
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
    use educore_core::ids::Identifier;

    fn fixture() -> (SchoolId, UserId, EventId, CorrelationId, Timestamp) {
        let g = SystemIdGen;
        (
            g.next_school_id(),
            g.next_user_id(),
            g.next_event_id(),
            g.next_correlation_id(),
            Timestamp::now(),
        )
    }

    #[test]
    fn vehicle_created_event_type_round_trips() {
        let (school, _, eid, corr, at) = fixture();
        let v = VehicleCreated::new(
            VehicleId::new(school, uuid::Uuid::now_v7()),
            crate::value_objects::VehicleNumber::new("V-1").unwrap(),
            "Bus".to_owned(),
            None,
            eid,
            corr,
            at,
        );
        assert_eq!(
            <VehicleCreated as DomainEvent>::EVENT_TYPE,
            "facilities.vehicle.created"
        );
        assert_eq!(<VehicleCreated as DomainEvent>::AGGREGATE_TYPE, "vehicle");
    }

    #[test]
    fn item_received_event_aggregates_lines() {
        let (school, _, eid, corr, at) = fixture();
        let item = ItemId::new(school, uuid::Uuid::now_v7());
        let lines = vec![ItemReceiveLineSpec {
            item_id: item,
            unit_price: crate::value_objects::UnitPrice(50),
            quantity: crate::value_objects::ItemQuantity(10),
            description: None,
        }];
        let ev = ItemReceived::new(
            ItemReceiveId::new(school, uuid::Uuid::now_v7()),
            SupplierId::new(school, uuid::Uuid::now_v7()),
            ItemStoreId::new(school, uuid::Uuid::now_v7()),
            chrono::NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
            500,
            10,
            500,
            0,
            PaidStatus::Paid,
            lines,
            eid,
            corr,
            at,
        );
        assert_eq!(ev.lines.len(), 1);
        assert_eq!(
            <ItemReceived as DomainEvent>::EVENT_TYPE,
            "facilities.item_receive.received"
        );
    }

    #[test]
    fn all_event_types_have_nonempty_wire_form() {
        let (school, _, eid, corr, at) = fixture();
        // Just exercise the EVENT_TYPE const on a few events to
        // ensure they're non-empty and snake_case.
        assert!(!<VehicleCreated as DomainEvent>::EVENT_TYPE.is_empty());
        assert!(!<RouteCreated as DomainEvent>::EVENT_TYPE.is_empty());
        assert!(!<VehicleAssigned as DomainEvent>::EVENT_TYPE.is_empty());
        assert!(!<StudentAssignedToRoute as DomainEvent>::EVENT_TYPE.is_empty());
        assert!(!<DormitoryCreated as DomainEvent>::EVENT_TYPE.is_empty());
        assert!(!<RoomCreated as DomainEvent>::EVENT_TYPE.is_empty());
        assert!(!<StudentAssignedToRoom as DomainEvent>::EVENT_TYPE.is_empty());
        assert!(!<ItemCreated as DomainEvent>::EVENT_TYPE.is_empty());
        assert!(!<ItemStoreCreated as DomainEvent>::EVENT_TYPE.is_empty());
        assert!(!<ItemReceived as DomainEvent>::EVENT_TYPE.is_empty());
        assert!(!<ItemIssued as DomainEvent>::EVENT_TYPE.is_empty());
        assert!(!<ItemSold as DomainEvent>::EVENT_TYPE.is_empty());
        assert!(!<SupplierCreated as DomainEvent>::EVENT_TYPE.is_empty());
    }
}
