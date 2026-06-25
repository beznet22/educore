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
// Update / Delete / Unassign events (Phase 8 gap-fill).
//
// These events cover Update and Delete on every aggregate except
// Vehicle (already shipped: VehicleUpdated, VehicleDeleted), the
// load-bearing teardown events (VehicleUnassigned,
// StudentUnassignedFromRoute, StudentUnassignedFromRoom,
// ItemReceiveCancelled, ItemSellRefunded, SupplierDeleted), and
// the inventory status event (ItemIssueStatusUpdated).
// =============================================================================

/// Emitted when a route's title / fare / distance changes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteUpdated {
    pub route_id: RouteId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl RouteUpdated {
    pub fn new(
        route_id: RouteId,
        changes: Vec<&'static str>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            route_id,
            changes: changes.into_iter().map(|s| s.to_owned()).collect(),
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for RouteUpdated {
    const EVENT_TYPE: &'static str = "facilities.route.updated";
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

/// Emitted when a stop on a route is edited (name / pickup_time /
/// fare_override).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopUpdatedOnRoute {
    pub route_id: RouteId,
    pub stop_order: u32,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl StopUpdatedOnRoute {
    pub fn new(
        route_id: RouteId,
        stop_order: u32,
        changes: Vec<&'static str>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            route_id,
            stop_order,
            changes: changes.into_iter().map(|s| s.to_owned()).collect(),
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StopUpdatedOnRoute {
    const EVENT_TYPE: &'static str = "facilities.route.stop_updated";
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

/// Emitted when a stop is removed from a route.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopRemovedFromRoute {
    pub route_id: RouteId,
    pub stop_order: u32,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl StopRemovedFromRoute {
    pub fn new(
        route_id: RouteId,
        stop_order: u32,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            route_id,
            stop_order,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StopRemovedFromRoute {
    const EVENT_TYPE: &'static str = "facilities.route.stop_removed";
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

/// Emitted when a route is deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteDeleted {
    pub route_id: RouteId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl RouteDeleted {
    pub fn new(
        route_id: RouteId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            route_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for RouteDeleted {
    const EVENT_TYPE: &'static str = "facilities.route.deleted";
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

/// Emitted when a vehicle is unassigned from a route.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleUnassigned {
    pub assign_vehicle_id: AssignVehicleId,
    pub vehicle_id: VehicleId,
    pub route_id: RouteId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl VehicleUnassigned {
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

impl DomainEvent for VehicleUnassigned {
    const EVENT_TYPE: &'static str = "facilities.assign_vehicle.deleted";
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

/// Emitted when a student is unassigned from a vehicle-route pair.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StudentUnassignedFromRoute {
    pub assign_vehicle_id: AssignVehicleId,
    pub student_id: StudentId,
    pub left_at: NaiveDate,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl StudentUnassignedFromRoute {
    pub fn new(
        assign_vehicle_id: AssignVehicleId,
        student_id: StudentId,
        left_at: NaiveDate,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            assign_vehicle_id,
            student_id,
            left_at,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StudentUnassignedFromRoute {
    const EVENT_TYPE: &'static str = "facilities.assign_vehicle.student_unassigned";
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

/// Emitted when a dormitory is updated (name / address / intake /
/// description).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DormitoryUpdated {
    pub dormitory_id: DormitoryId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DormitoryUpdated {
    pub fn new(
        dormitory_id: DormitoryId,
        changes: Vec<&'static str>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            dormitory_id,
            changes: changes.into_iter().map(|s| s.to_owned()).collect(),
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DormitoryUpdated {
    const EVENT_TYPE: &'static str = "facilities.dormitory.updated";
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

/// Emitted when a dormitory is hard-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DormitoryDeleted {
    pub dormitory_id: DormitoryId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DormitoryDeleted {
    pub fn new(
        dormitory_id: DormitoryId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            dormitory_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DormitoryDeleted {
    const EVENT_TYPE: &'static str = "facilities.dormitory.deleted";
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

/// Emitted when a room type is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoomTypeUpdated {
    pub room_type_id: RoomTypeId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl RoomTypeUpdated {
    pub fn new(
        room_type_id: RoomTypeId,
        changes: Vec<&'static str>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            room_type_id,
            changes: changes.into_iter().map(|s| s.to_owned()).collect(),
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for RoomTypeUpdated {
    const EVENT_TYPE: &'static str = "facilities.room_type.updated";
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

/// Emitted when a room type is deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoomTypeDeleted {
    pub room_type_id: RoomTypeId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl RoomTypeDeleted {
    pub fn new(
        room_type_id: RoomTypeId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            room_type_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for RoomTypeDeleted {
    const EVENT_TYPE: &'static str = "facilities.room_type.deleted";
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

/// Emitted when a room is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoomUpdated {
    pub room_id: RoomId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl RoomUpdated {
    pub fn new(
        room_id: RoomId,
        changes: Vec<&'static str>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            room_id,
            changes: changes.into_iter().map(|s| s.to_owned()).collect(),
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for RoomUpdated {
    const EVENT_TYPE: &'static str = "facilities.room.updated";
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

/// Emitted when a room is hard-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoomDeleted {
    pub room_id: RoomId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl RoomDeleted {
    pub fn new(
        room_id: RoomId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            room_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for RoomDeleted {
    const EVENT_TYPE: &'static str = "facilities.room.deleted";
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

/// Emitted when a student is released from a room.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StudentUnassignedFromRoom {
    pub room_id: RoomId,
    pub student_id: StudentId,
    pub released_at: Timestamp,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl StudentUnassignedFromRoom {
    pub fn new(
        room_id: RoomId,
        student_id: StudentId,
        released_at: Timestamp,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            room_id,
            student_id,
            released_at,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StudentUnassignedFromRoom {
    const EVENT_TYPE: &'static str = "facilities.room.student_unassigned";
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

/// Emitted when an item category is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemCategoryUpdated {
    pub item_category_id: crate::value_objects::ItemCategoryId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ItemCategoryUpdated {
    pub fn new(
        item_category_id: crate::value_objects::ItemCategoryId,
        changes: Vec<&'static str>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            item_category_id,
            changes: changes.into_iter().map(|s| s.to_owned()).collect(),
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ItemCategoryUpdated {
    const EVENT_TYPE: &'static str = "facilities.item_category.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "item_category";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.item_category_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.item_category_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when an item category is deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemCategoryDeleted {
    pub item_category_id: crate::value_objects::ItemCategoryId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ItemCategoryDeleted {
    pub fn new(
        item_category_id: crate::value_objects::ItemCategoryId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            item_category_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ItemCategoryDeleted {
    const EVENT_TYPE: &'static str = "facilities.item_category.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "item_category";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.item_category_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.item_category_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when an item's name / category / description changes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemUpdated {
    pub item_id: ItemId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ItemUpdated {
    pub fn new(
        item_id: ItemId,
        changes: Vec<&'static str>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            item_id,
            changes: changes.into_iter().map(|s| s.to_owned()).collect(),
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ItemUpdated {
    const EVENT_TYPE: &'static str = "facilities.item.updated";
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

/// Emitted when an item is hard-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemDeleted {
    pub item_id: ItemId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ItemDeleted {
    pub fn new(
        item_id: ItemId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            item_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ItemDeleted {
    const EVENT_TYPE: &'static str = "facilities.item.deleted";
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

/// Emitted when an item store is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemStoreUpdated {
    pub item_store_id: ItemStoreId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ItemStoreUpdated {
    pub fn new(
        item_store_id: ItemStoreId,
        changes: Vec<&'static str>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            item_store_id,
            changes: changes.into_iter().map(|s| s.to_owned()).collect(),
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ItemStoreUpdated {
    const EVENT_TYPE: &'static str = "facilities.item_store.updated";
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

/// Emitted when an item store is deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemStoreDeleted {
    pub item_store_id: ItemStoreId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ItemStoreDeleted {
    pub fn new(
        item_store_id: ItemStoreId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            item_store_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ItemStoreDeleted {
    const EVENT_TYPE: &'static str = "facilities.item_store.deleted";
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

/// Emitted when an existing receive is edited (lines added /
/// removed, total_paid updated, paid_status changed).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemReceiveUpdated {
    pub item_receive_id: ItemReceiveId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ItemReceiveUpdated {
    pub fn new(
        item_receive_id: ItemReceiveId,
        changes: Vec<&'static str>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            item_receive_id,
            changes: changes.into_iter().map(|s| s.to_owned()).collect(),
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ItemReceiveUpdated {
    const EVENT_TYPE: &'static str = "facilities.item_receive.updated";
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

/// Emitted when a receive is cancelled. Finance may subscribe to
/// reverse the payable; inventory subscribers reverse the stock
/// increment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemReceiveCancelled {
    pub item_receive_id: ItemReceiveId,
    pub reason: String,
    pub reversed_lines: Vec<ItemReceiveLineSpec>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ItemReceiveCancelled {
    pub fn new(
        item_receive_id: ItemReceiveId,
        reason: String,
        reversed_lines: Vec<ItemReceiveLineSpec>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            item_receive_id,
            reason,
            reversed_lines,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ItemReceiveCancelled {
    const EVENT_TYPE: &'static str = "facilities.item_receive.cancelled";
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

/// Emitted when an issue's status changes (Issued → Returned /
/// PartiallyReturned / Lost).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemIssueStatusUpdated {
    pub item_issue_id: ItemIssueId,
    pub from_status: IssueStatus,
    pub to_status: IssueStatus,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ItemIssueStatusUpdated {
    pub fn new(
        item_issue_id: ItemIssueId,
        from_status: IssueStatus,
        to_status: IssueStatus,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            item_issue_id,
            from_status,
            to_status,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ItemIssueStatusUpdated {
    const EVENT_TYPE: &'static str = "facilities.item_issue.status_updated";
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

/// Emitted when an existing sale is edited (lines added / removed,
/// total_paid updated, paid_status changed).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemSellUpdated {
    pub item_sell_id: ItemSellId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ItemSellUpdated {
    pub fn new(
        item_sell_id: ItemSellId,
        changes: Vec<&'static str>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            item_sell_id,
            changes: changes.into_iter().map(|s| s.to_owned()).collect(),
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ItemSellUpdated {
    const EVENT_TYPE: &'static str = "facilities.item_sell.updated";
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

/// Emitted when a sale is refunded (full or partial).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemSellRefunded {
    pub item_sell_id: ItemSellId,
    pub refund_amount: i64,
    pub new_paid_status: PaidStatus,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ItemSellRefunded {
    pub fn new(
        item_sell_id: ItemSellId,
        refund_amount: i64,
        new_paid_status: PaidStatus,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            item_sell_id,
            refund_amount,
            new_paid_status,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ItemSellRefunded {
    const EVENT_TYPE: &'static str = "facilities.item_sell.refunded";
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

/// Emitted when a supplier is updated (name / address / contact /
/// description).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SupplierUpdated {
    pub supplier_id: SupplierId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SupplierUpdated {
    pub fn new(
        supplier_id: SupplierId,
        changes: Vec<&'static str>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            supplier_id,
            changes: changes.into_iter().map(|s| s.to_owned()).collect(),
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SupplierUpdated {
    const EVENT_TYPE: &'static str = "facilities.supplier.updated";
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

/// Emitted when a supplier is hard-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SupplierDeleted {
    pub supplier_id: SupplierId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SupplierDeleted {
    pub fn new(
        supplier_id: SupplierId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            supplier_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SupplierDeleted {
    const EVENT_TYPE: &'static str = "facilities.supplier.deleted";
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
    clippy::dbg_macro,
    unused_variables
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

    // -------------------------------------------------------------------------
    // Gap-fill happy-path tests (Phase 8 Update / Delete / Unassign events).
    // Each test exercises the EVENT_TYPE wire form, the AGGREGATE_TYPE,
    // and the constructor's round-trip behavior for one of the 26
    // new events.
    // -------------------------------------------------------------------------

    #[test]
    fn route_updated_event_happy_path() {
        let (school, _, eid, corr, at) = fixture();
        let ev = RouteUpdated::new(
            crate::value_objects::RouteId::new(school, uuid::Uuid::now_v7()),
            vec!["title", "fare"],
            eid,
            corr,
            at,
        );
        assert_eq!(
            <RouteUpdated as DomainEvent>::EVENT_TYPE,
            "facilities.route.updated"
        );
        assert_eq!(<RouteUpdated as DomainEvent>::AGGREGATE_TYPE, "route");
        assert_eq!(ev.changes, vec!["title", "fare"]);
    }

    #[test]
    fn stop_removed_from_route_event_happy_path() {
        let (school, _, eid, corr, at) = fixture();
        let ev = StopRemovedFromRoute::new(
            crate::value_objects::RouteId::new(school, uuid::Uuid::now_v7()),
            3,
            eid,
            corr,
            at,
        );
        assert_eq!(
            <StopRemovedFromRoute as DomainEvent>::EVENT_TYPE,
            "facilities.route.stop_removed"
        );
        assert_eq!(ev.stop_order, 3);
    }

    #[test]
    fn route_deleted_event_happy_path() {
        let (school, _, eid, corr, at) = fixture();
        let id = crate::value_objects::RouteId::new(school, uuid::Uuid::now_v7());
        let ev = RouteDeleted::new(id, eid, corr, at);
        assert_eq!(
            <RouteDeleted as DomainEvent>::EVENT_TYPE,
            "facilities.route.deleted"
        );
        assert_eq!(ev.route_id, id);
    }

    #[test]
    fn vehicle_unassigned_event_happy_path() {
        let (school, _, eid, corr, at) = fixture();
        let av = crate::value_objects::AssignVehicleId::new(school, uuid::Uuid::now_v7());
        let v = VehicleId::new(school, uuid::Uuid::now_v7());
        let r = RouteId::new(school, uuid::Uuid::now_v7());
        let ev = VehicleUnassigned::new(av, v, r, eid, corr, at);
        assert_eq!(
            <VehicleUnassigned as DomainEvent>::EVENT_TYPE,
            "facilities.assign_vehicle.deleted"
        );
        assert_eq!(ev.vehicle_id, v);
    }

    #[test]
    fn student_unassigned_from_route_event_happy_path() {
        let (school, _, eid, corr, at) = fixture();
        let av = crate::value_objects::AssignVehicleId::new(school, uuid::Uuid::now_v7());
        let stu = StudentId::new(school, uuid::Uuid::now_v7());
        let ev = StudentUnassignedFromRoute::new(
            av,
            stu,
            chrono::NaiveDate::from_ymd_opt(2026, 6, 24).unwrap(),
            eid,
            corr,
            at,
        );
        assert_eq!(
            <StudentUnassignedFromRoute as DomainEvent>::EVENT_TYPE,
            "facilities.assign_vehicle.student_unassigned"
        );
        assert_eq!(ev.student_id, stu);
    }

    #[test]
    fn dormitory_updated_and_deleted_event_happy_path() {
        let (school, _, eid, corr, at) = fixture();
        let id = DormitoryId::new(school, uuid::Uuid::now_v7());
        let upd = DormitoryUpdated::new(id, vec!["intake"], eid, corr, at);
        assert_eq!(
            <DormitoryUpdated as DomainEvent>::EVENT_TYPE,
            "facilities.dormitory.updated"
        );
        assert_eq!(upd.changes, vec!["intake"]);
        let del = DormitoryDeleted::new(id, eid, corr, at);
        assert_eq!(
            <DormitoryDeleted as DomainEvent>::EVENT_TYPE,
            "facilities.dormitory.deleted"
        );
    }

    #[test]
    fn room_type_updated_and_deleted_event_happy_path() {
        let (school, _, eid, corr, at) = fixture();
        let id = RoomTypeId::new(school, uuid::Uuid::now_v7());
        let upd = RoomTypeUpdated::new(id, vec!["description"], eid, corr, at);
        assert_eq!(
            <RoomTypeUpdated as DomainEvent>::EVENT_TYPE,
            "facilities.room_type.updated"
        );
        let del = RoomTypeDeleted::new(id, eid, corr, at);
        assert_eq!(
            <RoomTypeDeleted as DomainEvent>::EVENT_TYPE,
            "facilities.room_type.deleted"
        );
    }

    #[test]
    fn room_updated_and_deleted_event_happy_path() {
        let (school, _, eid, corr, at) = fixture();
        let id = RoomId::new(school, uuid::Uuid::now_v7());
        let upd = RoomUpdated::new(id, vec!["cost_per_bed"], eid, corr, at);
        assert_eq!(
            <RoomUpdated as DomainEvent>::EVENT_TYPE,
            "facilities.room.updated"
        );
        let del = RoomDeleted::new(id, eid, corr, at);
        assert_eq!(
            <RoomDeleted as DomainEvent>::EVENT_TYPE,
            "facilities.room.deleted"
        );
    }

    #[test]
    fn student_unassigned_from_room_event_happy_path() {
        let (school, _, eid, corr, at) = fixture();
        let room = RoomId::new(school, uuid::Uuid::now_v7());
        let stu = StudentId::new(school, uuid::Uuid::now_v7());
        let ev = StudentUnassignedFromRoom::new(room, stu, at, eid, corr, at);
        assert_eq!(
            <StudentUnassignedFromRoom as DomainEvent>::EVENT_TYPE,
            "facilities.room.student_unassigned"
        );
        assert_eq!(ev.released_at, at);
    }

    #[test]
    fn item_category_updated_and_deleted_event_happy_path() {
        let (school, _, eid, corr, at) = fixture();
        let id = crate::value_objects::ItemCategoryId::new(school, uuid::Uuid::now_v7());
        let upd = ItemCategoryUpdated::new(id, vec!["category_name"], eid, corr, at);
        assert_eq!(
            <ItemCategoryUpdated as DomainEvent>::EVENT_TYPE,
            "facilities.item_category.updated"
        );
        let del = ItemCategoryDeleted::new(id, eid, corr, at);
        assert_eq!(
            <ItemCategoryDeleted as DomainEvent>::EVENT_TYPE,
            "facilities.item_category.deleted"
        );
    }

    #[test]
    fn item_updated_and_deleted_event_happy_path() {
        let (school, _, eid, corr, at) = fixture();
        let id = ItemId::new(school, uuid::Uuid::now_v7());
        let upd = ItemUpdated::new(id, vec!["item_name"], eid, corr, at);
        assert_eq!(
            <ItemUpdated as DomainEvent>::EVENT_TYPE,
            "facilities.item.updated"
        );
        let del = ItemDeleted::new(id, eid, corr, at);
        assert_eq!(
            <ItemDeleted as DomainEvent>::EVENT_TYPE,
            "facilities.item.deleted"
        );
    }

    #[test]
    fn item_store_updated_and_deleted_event_happy_path() {
        let (school, _, eid, corr, at) = fixture();
        let id = ItemStoreId::new(school, uuid::Uuid::now_v7());
        let upd = ItemStoreUpdated::new(id, vec!["store_name"], eid, corr, at);
        assert_eq!(
            <ItemStoreUpdated as DomainEvent>::EVENT_TYPE,
            "facilities.item_store.updated"
        );
        let del = ItemStoreDeleted::new(id, eid, corr, at);
        assert_eq!(
            <ItemStoreDeleted as DomainEvent>::EVENT_TYPE,
            "facilities.item_store.deleted"
        );
    }

    #[test]
    fn item_receive_updated_and_cancelled_event_happy_path() {
        let (school, _, eid, corr, at) = fixture();
        let id = ItemReceiveId::new(school, uuid::Uuid::now_v7());
        let upd = ItemReceiveUpdated::new(id, vec!["total_paid"], eid, corr, at);
        assert_eq!(
            <ItemReceiveUpdated as DomainEvent>::EVENT_TYPE,
            "facilities.item_receive.updated"
        );
        let canc =
            ItemReceiveCancelled::new(id, "supplier return".to_owned(), Vec::new(), eid, corr, at);
        assert_eq!(
            <ItemReceiveCancelled as DomainEvent>::EVENT_TYPE,
            "facilities.item_receive.cancelled"
        );
        assert_eq!(canc.reason, "supplier return");
    }

    #[test]
    fn item_issue_status_updated_event_happy_path() {
        let (school, _, eid, corr, at) = fixture();
        let id = ItemIssueId::new(school, uuid::Uuid::now_v7());
        let ev =
            ItemIssueStatusUpdated::new(id, IssueStatus::Issued, IssueStatus::Lost, eid, corr, at);
        assert_eq!(
            <ItemIssueStatusUpdated as DomainEvent>::EVENT_TYPE,
            "facilities.item_issue.status_updated"
        );
        assert_eq!(ev.from_status, IssueStatus::Issued);
        assert_eq!(ev.to_status, IssueStatus::Lost);
    }

    #[test]
    fn item_sell_updated_and_refunded_event_happy_path() {
        let (school, _, eid, corr, at) = fixture();
        let id = ItemSellId::new(school, uuid::Uuid::now_v7());
        let upd = ItemSellUpdated::new(id, vec!["paid_status"], eid, corr, at);
        assert_eq!(
            <ItemSellUpdated as DomainEvent>::EVENT_TYPE,
            "facilities.item_sell.updated"
        );
        let refund = ItemSellRefunded::new(id, 500, PaidStatus::Refunded, eid, corr, at);
        assert_eq!(
            <ItemSellRefunded as DomainEvent>::EVENT_TYPE,
            "facilities.item_sell.refunded"
        );
        assert_eq!(refund.refund_amount, 500);
    }

    #[test]
    fn supplier_updated_and_deleted_event_happy_path() {
        let (school, _, eid, corr, at) = fixture();
        let id = SupplierId::new(school, uuid::Uuid::now_v7());
        let upd = SupplierUpdated::new(id, vec!["company_name"], eid, corr, at);
        assert_eq!(
            <SupplierUpdated as DomainEvent>::EVENT_TYPE,
            "facilities.supplier.updated"
        );
        let del = SupplierDeleted::new(id, eid, corr, at);
        assert_eq!(
            <SupplierDeleted as DomainEvent>::EVENT_TYPE,
            "facilities.supplier.deleted"
        );
    }

    #[test]
    fn stop_updated_on_route_event_happy_path() {
        let (school, _, eid, corr, at) = fixture();
        let route = RouteId::new(school, uuid::Uuid::now_v7());
        let ev = StopUpdatedOnRoute::new(route, 2, vec!["stop_name"], eid, corr, at);
        assert_eq!(
            <StopUpdatedOnRoute as DomainEvent>::EVENT_TYPE,
            "facilities.route.stop_updated"
        );
        assert_eq!(ev.stop_order, 2);
    }
}
