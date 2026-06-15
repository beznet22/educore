//! # Facilities child entities
//!
//! Phase 8 ships the 6 child entities from
//! `docs/specs/facilities/entities.md`. Each is owned by an
//! aggregate root and persisted as a child row (loaded through
//! the aggregate repository).
//!
//! The `ItemIssueLine` is the optional per-issue row that
//! carries the partial-return counter for `PartiallyReturned`
//! status. It is omitted when the issue is fully issued (the
//! header itself carries the full quantity); it appears when
//! `returned_quantity > 0`.

#![allow(missing_docs)]
#![allow(unused_imports)]
#![allow(dead_code)]

use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};

use crate::value_objects::{
    BedNumber, ContactPersonName, EmailAddress, Fare, ItemQuantity, PhoneNumber, RouteName,
    StaffId, StopName, StudentId,
};

fn fresh_etag() -> Etag {
    Etag::placeholder()
}

// =============================================================================
// RouteStop (owned by Route)
// =============================================================================

/// A single stop on a transport route, with a `StopOrder` (u32),
/// a `StopName`, an optional `PickupTime`, and an optional `Fare`
/// override (the default is the route's fare).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteStop {
    /// The owning school (derived from `route_id`).
    pub school_id: SchoolId,
    /// The owning route.
    pub route_id: crate::value_objects::RouteId,
    /// The stop order within the route.
    pub stop_order: u32,
    /// The stop name.
    pub stop_name: StopName,
    /// The optional pickup time.
    pub pickup_time: Option<NaiveTime>,
    /// The optional fare override.
    pub fare_override: Option<Fare>,
    /// The audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl RouteStop {
    /// Constructs a new `RouteStop`.
    pub fn fresh(
        route_id: crate::value_objects::RouteId,
        stop_order: u32,
        stop_name: StopName,
        pickup_time: Option<NaiveTime>,
        fare_override: Option<Fare>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: route_id.school_id(),
            route_id,
            stop_order,
            stop_name,
            pickup_time,
            fare_override,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}

// =============================================================================
// TransportMembership (owned by AssignVehicle)
// =============================================================================

/// The membership of a `StudentId` in a vehicle-route
/// assignment. Has `JoinedAt` and `LeftAt?`. The current
/// membership for a student in a vehicle-route pair is the one
/// with `LeftAt = None`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransportMembership {
    /// The owning school (derived from `assign_vehicle_id`).
    pub school_id: SchoolId,
    /// The parent assignment.
    pub assign_vehicle_id: crate::value_objects::AssignVehicleId,
    /// The student.
    pub student_id: StudentId,
    /// The join date.
    pub joined_at: NaiveDate,
    /// The optional leave date.
    pub left_at: Option<NaiveDate>,
    /// The optional pickup stop.
    pub pickup_stop_id: Option<crate::value_objects::RouteStopId>,
    /// The optional drop stop.
    pub drop_stop_id: Option<crate::value_objects::RouteStopId>,
    /// The audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl TransportMembership {
    /// Constructs a new `TransportMembership` in the current
    /// (not-left) state.
    pub fn fresh(
        assign_vehicle_id: crate::value_objects::AssignVehicleId,
        student_id: StudentId,
        joined_at: NaiveDate,
        pickup_stop_id: Option<crate::value_objects::RouteStopId>,
        drop_stop_id: Option<crate::value_objects::RouteStopId>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: assign_vehicle_id.school_id(),
            assign_vehicle_id,
            student_id,
            joined_at,
            left_at: None,
            pickup_stop_id,
            drop_stop_id,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Returns `true` if the membership is current (not left).
    #[must_use]
    pub fn is_current(&self) -> bool {
        self.left_at.is_none()
    }
}

// =============================================================================
// RoomAssignment (owned by Room)
// =============================================================================

/// A current or historical assignment of a `StudentId` to a
/// specific bed in a `Room`. Has `AssignedAt`, `ReleasedAt?`, and
/// a `BedNumber` (1..N where N is `Room.NumberOfBed`). The
/// current assignment for a student is the one with
/// `ReleasedAt = None`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoomAssignment {
    /// The owning school (derived from `room_id`).
    pub school_id: SchoolId,
    /// The owning room.
    pub room_id: crate::value_objects::RoomId,
    /// The student.
    pub student_id: StudentId,
    /// The bed number (1-indexed).
    pub bed_number: BedNumber,
    /// The assignment date.
    pub assigned_at: Timestamp,
    /// The optional release date.
    pub released_at: Option<Timestamp>,
    /// The audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl RoomAssignment {
    /// Constructs a new `RoomAssignment` in the current
    /// (not-released) state.
    pub fn fresh(
        room_id: crate::value_objects::RoomId,
        student_id: StudentId,
        bed_number: BedNumber,
        assigned_at: Timestamp,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: room_id.school_id(),
            room_id,
            student_id,
            bed_number,
            assigned_at,
            released_at: None,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Returns `true` if the assignment is current (not
    /// released).
    #[must_use]
    pub fn is_current(&self) -> bool {
        self.released_at.is_none()
    }
}

// =============================================================================
// ItemIssueLine (owned by ItemIssue — optional child for partial returns)
// =============================================================================

/// The line on an `ItemIssue`. In the canonical model the line
/// is the `Item` itself, but an issue may carry a single child
/// entity that encodes the `Note` and a returned-quantity counter
/// for `PartiallyReturned` status. The header carries the
/// full-issue `ItemQuantity`; the line carries the per-issue
/// returned counter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemIssueLine {
    /// The owning school (derived from `item_issue_id`).
    pub school_id: SchoolId,
    /// The parent issue.
    pub item_issue_id: crate::value_objects::ItemIssueId,
    /// The quantity returned so far (a per-line counter for
    /// `PartiallyReturned`).
    pub returned_quantity: ItemQuantity,
    /// An optional note.
    pub note: Option<crate::value_objects::Note>,
    /// The audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl ItemIssueLine {
    /// Constructs a new `ItemIssueLine` in the initial state.
    pub fn fresh(
        item_issue_id: crate::value_objects::ItemIssueId,
        returned_quantity: ItemQuantity,
        note: Option<crate::value_objects::Note>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: item_issue_id.school_id(),
            item_issue_id,
            returned_quantity,
            note,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}

// =============================================================================
// ItemReceiveLine (owned by ItemReceive)
// =============================================================================

/// A non-rooted child entity that pairs a received `Item` with a
/// `BatchNumber` and an optional `ExpiryDate` (for perishable
/// items). Backs the `ItemReceiveChild` aggregate when per-line
/// metadata exceeds the four scalar fields of the canonical
/// child.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemReceiveLine {
    /// The owning school (derived from `item_receive_id`).
    pub school_id: SchoolId,
    /// The parent receive.
    pub item_receive_id: crate::value_objects::ItemReceiveId,
    /// The item.
    pub item_id: crate::value_objects::ItemId,
    /// The batch number.
    pub batch_number: String,
    /// The optional expiry date (perishable items only).
    pub expiry_date: Option<NaiveDate>,
    /// The audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl ItemReceiveLine {
    /// Constructs a new `ItemReceiveLine`.
    pub fn fresh(
        item_receive_id: crate::value_objects::ItemReceiveId,
        item_id: crate::value_objects::ItemId,
        batch_number: String,
        expiry_date: Option<NaiveDate>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: item_receive_id.school_id(),
            item_receive_id,
            item_id,
            batch_number,
            expiry_date,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}

// =============================================================================
// ItemSellLine (owned by ItemSell)
// =============================================================================

/// A non-rooted child entity that pairs a sold `Item` with a
/// discount override and a `SerialNumber` (for serialized
/// assets). Backs the `ItemSellChild` aggregate when per-line
/// metadata exceeds the scalar fields of the canonical child.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemSellLine {
    /// The owning school (derived from `item_sell_id`).
    pub school_id: SchoolId,
    /// The parent sale.
    pub item_sell_id: crate::value_objects::ItemSellId,
    /// The item.
    pub item_id: crate::value_objects::ItemId,
    /// The discount override (minor units; 0 = no override).
    pub discount_minor: i64,
    /// The optional serial number (for serialized assets).
    pub serial_number: Option<String>,
    /// The audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl ItemSellLine {
    /// Constructs a new `ItemSellLine`.
    pub fn fresh(
        item_sell_id: crate::value_objects::ItemSellId,
        item_id: crate::value_objects::ItemId,
        discount_minor: i64,
        serial_number: Option<String>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: item_sell_id.school_id(),
            item_sell_id,
            item_id,
            discount_minor,
            serial_number,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}

// =============================================================================
// DriverAssignment (owned by Vehicle)
// =============================================================================

/// The current and historical assignments of a `StaffId` to a
/// `Vehicle` as the primary driver. Has `AssignedAt` and
/// `ReleasedAt?`. A vehicle may have at most one current driver
/// assignment at a time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DriverAssignment {
    /// The owning school (derived from `vehicle_id`).
    pub school_id: SchoolId,
    /// The owning vehicle.
    pub vehicle_id: crate::value_objects::VehicleId,
    /// The driver.
    pub driver_id: StaffId,
    /// The assignment date.
    pub assigned_at: Timestamp,
    /// The optional release date.
    pub released_at: Option<Timestamp>,
    /// The audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl DriverAssignment {
    /// Constructs a new `DriverAssignment`.
    pub fn fresh(
        vehicle_id: crate::value_objects::VehicleId,
        driver_id: StaffId,
        assigned_at: Timestamp,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: vehicle_id.school_id(),
            vehicle_id,
            driver_id,
            assigned_at,
            released_at: None,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}

// =============================================================================
// SupplierContact (owned by Supplier)
// =============================================================================

/// An additional contact at a `Supplier`. The supplier aggregate
/// exposes a primary contact through its own fields; this entity
/// is used when multiple individuals are reachable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SupplierContact {
    /// The owning school (derived from `supplier_id`).
    pub school_id: SchoolId,
    /// The parent supplier.
    pub supplier_id: crate::value_objects::SupplierId,
    /// The contact's name.
    pub name: ContactPersonName,
    /// The contact's mobile.
    pub mobile: Option<PhoneNumber>,
    /// The contact's email.
    pub email: Option<EmailAddress>,
    /// The audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl SupplierContact {
    /// Constructs a new `SupplierContact`.
    pub fn fresh(
        supplier_id: crate::value_objects::SupplierId,
        name: ContactPersonName,
        mobile: Option<PhoneNumber>,
        email: Option<EmailAddress>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: supplier_id.school_id(),
            supplier_id,
            name,
            mobile,
            email,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}

// =============================================================================
// DormitoryNote (owned by Dormitory)
// =============================================================================

/// An administrative note about a dormitory (renovation,
/// fire-safety audit, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DormitoryNote {
    /// The owning school (derived from `dormitory_id`).
    pub school_id: SchoolId,
    /// The parent dormitory.
    pub dormitory_id: crate::value_objects::DormitoryId,
    /// The author.
    pub author: UserId,
    /// The body.
    pub body: String,
    /// Whether the note is visible to parents.
    pub visible_to_parents: bool,
    /// The audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl DormitoryNote {
    /// Constructs a new `DormitoryNote`.
    pub fn fresh(
        dormitory_id: crate::value_objects::DormitoryId,
        author: UserId,
        body: String,
        visible_to_parents: bool,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: dormitory_id.school_id(),
            dormitory_id,
            author,
            body,
            visible_to_parents,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}

// =============================================================================
// StoreStocktake (owned by ItemStore)
// =============================================================================

/// A snapshot of a count exercise over a store. Has `StartedAt`,
/// `CompletedAt?`, `CountedBy`, and one or more
/// `StoreStocktakeLine` entities. Used to correct drift between
/// `Item.TotalInStock` and physical counts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoreStocktake {
    /// The owning school (derived from `store_id`).
    pub school_id: SchoolId,
    /// The owning store.
    pub store_id: crate::value_objects::ItemStoreId,
    /// The start timestamp.
    pub started_at: Timestamp,
    /// The optional completion timestamp.
    pub completed_at: Option<Timestamp>,
    /// The user who performed the count.
    pub counted_by: UserId,
    /// The audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl StoreStocktake {
    /// Constructs a new `StoreStocktake`.
    pub fn fresh(
        store_id: crate::value_objects::ItemStoreId,
        started_at: Timestamp,
        counted_by: UserId,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: store_id.school_id(),
            store_id,
            started_at,
            completed_at: None,
            counted_by,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Returns `true` if the stocktake is complete.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.completed_at.is_some()
    }
}

// =============================================================================
// TransportSpec, HostelSpec, MoneySpec (read-model helpers per the spec)
// =============================================================================

/// A transport specification per
/// `docs/specs/facilities/value-objects.md` § Specification
/// Helpers. Captures the per-student transport assignment view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransportSpec {
    /// The route the student is on.
    pub route_id: crate::value_objects::RouteId,
    /// The vehicle-route assignment.
    pub assign_vehicle_id: crate::value_objects::AssignVehicleId,
    /// The pickup stop.
    pub pickup_stop: Option<crate::value_objects::RouteStopId>,
    /// The drop stop.
    pub drop_stop: Option<crate::value_objects::RouteStopId>,
}

/// A hostel specification per the spec's specification helpers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostelSpec {
    /// The dormitory.
    pub dormitory_id: crate::value_objects::DormitoryId,
    /// The optional room.
    pub room_id: Option<crate::value_objects::RoomId>,
    /// The optional bed number.
    pub bed_number: Option<BedNumber>,
}

/// A money spec per the spec's specification helpers. Captures a
/// per-line price/quantity/subtotal triple.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MoneySpec {
    /// The unit price.
    pub unit_price: i64,
    /// The quantity.
    pub quantity: i64,
    /// The subtotal.
    pub sub_total: i64,
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

    #[test]
    fn route_stop_carries_route_id() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let route_id = crate::value_objects::RouteId::new(school, g.next_uuid());
        let stop = RouteStop::fresh(
            route_id,
            1,
            StopName::new("Main Gate").unwrap(),
            None,
            None,
            g.next_user_id(),
            Timestamp::now(),
            CorrelationId(g.next_uuid()),
        );
        assert_eq!(stop.route_id, route_id);
        assert_eq!(stop.school_id, school);
    }

    #[test]
    fn transport_membership_starts_current() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let aid = crate::value_objects::AssignVehicleId::new(school, g.next_uuid());
        let student = StudentId::new(school, g.next_uuid());
        let m = TransportMembership::fresh(
            aid,
            student,
            chrono::NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
            None,
            None,
            g.next_user_id(),
            Timestamp::now(),
            CorrelationId(g.next_uuid()),
        );
        assert!(m.is_current());
    }

    #[test]
    fn room_assignment_starts_current() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let rid = crate::value_objects::RoomId::new(school, g.next_uuid());
        let student = StudentId::new(school, g.next_uuid());
        let a = RoomAssignment::fresh(
            rid,
            student,
            BedNumber::new(1).unwrap(),
            Timestamp::now(),
            g.next_user_id(),
            Timestamp::now(),
            CorrelationId(g.next_uuid()),
        );
        assert!(a.is_current());
    }
}
