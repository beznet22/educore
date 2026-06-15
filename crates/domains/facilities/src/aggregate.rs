//! # Facilities aggregate roots
//!
//! Phase 8 ships the headline 11 aggregates per the spec at
//! `docs/specs/facilities/aggregates.md`:
//!
//! - `Vehicle` — transport vehicle master
//! - `Route` — transport route master (owns `RouteStop` children)
//! - `AssignVehicle` — vehicle-to-route assignment per academic year
//! - `Dormitory` — hostel building (owns `Room` children)
//! - `Room` — a room in a dormitory (owns `RoomAssignment` children)
//! - `RoomType` — room-type catalog entry
//! - `ItemCategory` — item-category catalog entry
//! - `Item` — inventory master (with `StockOnHand`)
//! - `ItemStore` — physical or logical store
//! - `ItemReceive` — goods-receive note (header; owns `ItemReceiveChild` lines)
//! - `ItemIssue` — goods-issue note (header)
//! - `ItemSell` — item sale (header; owns `ItemSellChild` lines)
//! - `ItemReceiveChild` / `ItemSellChild` — the line aggregates
//! - `Supplier` — vendor contact master
//!
//! Every aggregate follows the standard audit-footer pattern (per
//! `AGENTS.md`):
//!
//! - 1 typed id (e.g. `ItemId`) + 1 derived `school_id` anchor
//! - domain fields
//! - audit-metadata fields: `version`, `etag`, `created_at`,
//!   `updated_at`, `created_by`, `updated_by`, `active_status`,
//!   `last_event_id`, `correlation_id`
//!
//! `school_id` is **derived from `id.school_id()`**, never taken
//! from the caller.

#![allow(missing_docs)]
#![allow(unused_imports)]

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};

use crate::value_objects::{
    AcademicYearId, Address, BedNumber, CategoryName, ContactPersonName, CostPerBed, Description,
    Distance, DormitoryName, DormitoryType, EmailAddress, Fare, Intake, IssueRecipient,
    IssueStatus, ItemId, ItemName, ItemQuantity, ItemSku, MadeYear, Note, NumberOfBed, PaidStatus,
    PaymentMethod, PhoneNumber, ReferenceNumber, RoomNumber, RoomTypeName, SellPrice, StaffId,
    StockOnHand, StoreName, StoreNumber, SupplierName, SupplierStatus, UnitPrice, VehicleId,
    VehicleModel, VehicleNumber, VehicleStatus,
};

fn fresh_etag() -> Etag {
    Etag::placeholder()
}

// =============================================================================
// Transport: Vehicle
// =============================================================================

/// A school-owned or contracted vehicle used to transport
/// students between home stops and the school.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vehicle {
    /// The typed id (school_id + uuid).
    pub id: VehicleId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The academic year this vehicle is scoped to.
    pub academic_year_id: AcademicYearId,
    /// The vehicle's plate number (unique within a school).
    pub vehicle_no: VehicleNumber,
    /// The vehicle's model name.
    pub vehicle_model: VehicleModel,
    /// The model year (1950..=current year).
    pub made_year: Option<MadeYear>,
    /// The optional primary driver (`StaffId` from HR).
    pub driver_id: Option<StaffId>,
    /// The vehicle's operational status.
    pub status: VehicleStatus,
    /// A free-text note.
    pub note: Option<Note>,
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

impl Vehicle {
    /// Constructs a new `Vehicle` in the initial state.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: VehicleId,
        academic_year_id: AcademicYearId,
        vehicle_no: VehicleNumber,
        vehicle_model: VehicleModel,
        made_year: Option<MadeYear>,
        driver_id: Option<StaffId>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            academic_year_id,
            vehicle_no,
            vehicle_model,
            made_year,
            driver_id,
            status: VehicleStatus::Active,
            note: None,
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

    /// Assigns a new driver and bumps the version.
    pub fn assign_driver(
        &mut self,
        driver_id: StaffId,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) {
        self.driver_id = Some(driver_id);
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Deactivates the vehicle. Returns `Err` if the vehicle is
    /// already retired.
    pub fn deactivate(
        &mut self,
        new_status: VehicleStatus,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) -> educore_core::error::Result<()> {
        if self.status == VehicleStatus::Retired {
            return Err(educore_core::error::DomainError::conflict(
                "vehicle is already retired",
            ));
        }
        self.status = new_status;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
        Ok(())
    }
}

// =============================================================================
// Transport: Route
// =============================================================================

/// A transport route describes a path between a starting area
/// and the school, carries a fare, and is the unit to which a
/// vehicle is assigned.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Route {
    /// The typed id (school_id + uuid).
    pub id: RouteId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The academic year this route is scoped to.
    pub academic_year_id: AcademicYearId,
    /// The route's name (unique within a school-year).
    pub title: RouteName,
    /// The route's default fare.
    pub fare: Fare,
    /// The route's distance (optional).
    pub distance: Option<Distance>,
    /// The ordered list of stops on the route.
    pub stops: Vec<RouteStopSpec>,
    /// A free-text note.
    pub note: Option<Note>,
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

impl Route {
    /// Constructs a new `Route` with the given ordered stops.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: RouteId,
        academic_year_id: AcademicYearId,
        title: RouteName,
        fare: Fare,
        distance: Option<Distance>,
        stops: Vec<RouteStopSpec>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            academic_year_id,
            title,
            fare,
            distance,
            stops,
            note: None,
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

// Bring in the typed id after the `Route` struct so the macro
// can pick up the local `RouteName` reference.
use crate::value_objects::RouteName;
use crate::value_objects::RouteStopSpec;
use crate::value_objects::{AssignVehicleId, RouteId};

// =============================================================================
// Transport: AssignVehicle
// =============================================================================

/// Represents the assignment of a `Vehicle` to a `Route` for a
/// given `AcademicYear`. It is the join entity that transport
/// desk and attendance workflows operate on.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssignVehicle {
    /// The typed id (school_id + uuid).
    pub id: AssignVehicleId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The vehicle being assigned.
    pub vehicle_id: VehicleId,
    /// The route being assigned to.
    pub route_id: RouteId,
    /// The academic year this assignment is scoped to.
    pub academic_year_id: AcademicYearId,
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

impl AssignVehicle {
    /// Constructs a new `AssignVehicle` in the initial state.
    pub fn fresh(
        id: AssignVehicleId,
        vehicle_id: VehicleId,
        route_id: RouteId,
        academic_year_id: AcademicYearId,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            vehicle_id,
            route_id,
            academic_year_id,
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
// Housing: Dormitory
// =============================================================================

/// A residential building with a defined intake, gender scope,
/// and address.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dormitory {
    /// The typed id (school_id + uuid).
    pub id: DormitoryId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The academic year this dormitory is scoped to.
    pub academic_year_id: AcademicYearId,
    /// The dormitory's name (unique within a school-year).
    pub name: DormitoryName,
    /// The gender scope.
    pub dormitory_type: DormitoryType,
    /// The optional address.
    pub address: Option<Address>,
    /// The dormitory's intake (positive).
    pub intake: Intake,
    /// A free-text description.
    pub description: Option<Description>,
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

impl Dormitory {
    /// Constructs a new `Dormitory` in the initial state.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: DormitoryId,
        academic_year_id: AcademicYearId,
        name: DormitoryName,
        dormitory_type: DormitoryType,
        address: Option<Address>,
        intake: Intake,
        description: Option<Description>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            academic_year_id,
            name,
            dormitory_type,
            address,
            intake,
            description,
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
// Housing: RoomType
// =============================================================================

/// A catalog entry describing a room classification (e.g.
/// "Single", "Double", "Dormitory Style").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoomType {
    /// The typed id (school_id + uuid).
    pub id: RoomTypeId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The room-type name (unique within a school).
    pub name: RoomTypeName,
    /// An optional description.
    pub description: Option<Description>,
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

impl RoomType {
    /// Constructs a new `RoomType`.
    pub fn fresh(
        id: RoomTypeId,
        name: RoomTypeName,
        description: Option<Description>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            name,
            description,
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

use crate::value_objects::RoomTypeId;

// =============================================================================
// Housing: Room
// =============================================================================

/// A room within a `Dormitory`, with a type, number of beds, and
/// a per-bed cost.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Room {
    /// The typed id (school_id + uuid).
    pub id: RoomId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The dormitory this room belongs to.
    pub dormitory_id: DormitoryId,
    /// The room number (unique within a dormitory).
    pub room_number: RoomNumber,
    /// The room type.
    pub room_type_id: RoomTypeId,
    /// The number of beds (positive).
    pub number_of_bed: NumberOfBed,
    /// The cost per bed (non-negative).
    pub cost_per_bed: CostPerBed,
    /// An optional description.
    pub description: Option<Description>,
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

impl Room {
    /// Constructs a new `Room`.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: RoomId,
        dormitory_id: DormitoryId,
        room_number: RoomNumber,
        room_type_id: RoomTypeId,
        number_of_bed: NumberOfBed,
        cost_per_bed: CostPerBed,
        description: Option<Description>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            dormitory_id,
            room_number,
            room_type_id,
            number_of_bed,
            cost_per_bed,
            description,
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

use crate::value_objects::DormitoryId;
use crate::value_objects::RoomId;

// =============================================================================
// Inventory catalog: ItemCategory
// =============================================================================

/// A grouping of items (e.g. "Stationery", "Lab Equipment").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemCategory {
    /// The typed id (school_id + uuid).
    pub id: ItemCategoryId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The category's name (unique within a school).
    pub category_name: CategoryName,
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

impl ItemCategory {
    /// Constructs a new `ItemCategory`.
    pub fn fresh(
        id: ItemCategoryId,
        category_name: CategoryName,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            category_name,
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

use crate::value_objects::ItemCategoryId;

// =============================================================================
// Inventory catalog: Item
// =============================================================================

/// An inventory master record, owned by one `ItemCategory`, with
/// a unique SKU and a maintained `total_in_stock` value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    /// The typed id (school_id + uuid).
    pub id: ItemId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The academic year this item is scoped to.
    pub academic_year_id: AcademicYearId,
    /// The item's name.
    pub item_name: ItemName,
    /// The item's SKU (unique within a school).
    pub item_sku: ItemSku,
    /// The owning category.
    pub item_category_id: ItemCategoryId,
    /// The current stock on hand (non-negative).
    pub total_in_stock: StockOnHand,
    /// A free-text description.
    pub description: Option<Description>,
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

impl Item {
    /// Constructs a new `Item` with zero stock on hand.
    pub fn fresh(
        id: ItemId,
        academic_year_id: AcademicYearId,
        item_name: ItemName,
        item_sku: ItemSku,
        item_category_id: ItemCategoryId,
        description: Option<Description>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            academic_year_id,
            item_name,
            item_sku,
            item_category_id,
            total_in_stock: StockOnHand::ZERO,
            description,
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

    /// Applies a stock delta. Returns `Err` if the resulting
    /// stock would be negative.
    pub fn apply_stock_delta(
        &mut self,
        delta: i64,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) -> educore_core::error::Result<()> {
        let current = self.total_in_stock.value();
        let new = current.saturating_add(delta);
        if new < 0 {
            return Err(educore_core::error::DomainError::conflict(format!(
                "item stock cannot go negative: current={current}, delta={delta}"
            )));
        }
        self.total_in_stock = StockOnHand(new);
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
        Ok(())
    }
}

// =============================================================================
// Inventory catalog: ItemStore
// =============================================================================

/// A physical or logical location where items are kept.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemStore {
    /// The typed id (school_id + uuid).
    pub id: ItemStoreId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The store's name (unique within a school).
    pub store_name: StoreName,
    /// The optional store number.
    pub store_number: Option<StoreNumber>,
    /// A free-text description.
    pub description: Option<Description>,
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

impl ItemStore {
    /// Constructs a new `ItemStore`.
    pub fn fresh(
        id: ItemStoreId,
        store_name: StoreName,
        store_number: Option<StoreNumber>,
        description: Option<Description>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            store_name,
            store_number,
            description,
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

use crate::value_objects::ItemStoreId;

// =============================================================================
// Inventory movements: ItemReceive (header)
// =============================================================================

/// A goods-receive note (GRN). Records the receipt of items from
/// a `Supplier` into an `ItemStore`, with totals, payment state,
/// and one or more `ItemReceiveChild` lines.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemReceive {
    /// The typed id (school_id + uuid).
    pub id: ItemReceiveId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The academic year this receive is scoped to.
    pub academic_year_id: AcademicYearId,
    /// The date the receive was posted.
    pub receive_date: NaiveDate,
    /// An optional reference number (PO #, etc.).
    pub reference_no: Option<ReferenceNumber>,
    /// The supplier.
    pub supplier_id: SupplierId,
    /// The receiving store.
    pub store_id: ItemStoreId,
    /// The total quantity received.
    pub total_quantity: ItemQuantity,
    /// The grand total in minor units.
    pub grand_total: i64,
    /// The amount paid.
    pub total_paid: i64,
    /// The amount due.
    pub total_due: i64,
    /// The payment method.
    pub payment_method: PaymentMethod,
    /// The paid status.
    pub paid_status: PaidStatus,
    /// An optional description.
    pub description: Option<Description>,
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

impl ItemReceive {
    /// Constructs a new `ItemReceive` from a set of pre-computed
    /// totals and child lines.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: ItemReceiveId,
        academic_year_id: AcademicYearId,
        receive_date: NaiveDate,
        reference_no: Option<ReferenceNumber>,
        supplier_id: SupplierId,
        store_id: ItemStoreId,
        total_quantity: ItemQuantity,
        grand_total: i64,
        total_paid: i64,
        payment_method: PaymentMethod,
        paid_status: PaidStatus,
        description: Option<Description>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        let total_due = grand_total.saturating_sub(total_paid);
        Self {
            school_id: id.school_id(),
            id,
            academic_year_id,
            receive_date,
            reference_no,
            supplier_id,
            store_id,
            total_quantity,
            grand_total,
            total_paid,
            total_due,
            payment_method,
            paid_status,
            description,
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

use crate::value_objects::ItemReceiveId;
use crate::value_objects::SupplierId;

// =============================================================================
// Inventory movements: ItemReceiveChild (line aggregate)
// =============================================================================

/// A single line on an `ItemReceive`. Records the unit price,
/// quantity, and subtotal for a specific `Item` received in the
/// GRN.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemReceiveChild {
    /// The typed id (school_id + uuid).
    pub id: ItemReceiveChildId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The parent receive.
    pub item_receive_id: ItemReceiveId,
    /// The item being received.
    pub item_id: ItemId,
    /// The unit price.
    pub unit_price: UnitPrice,
    /// The quantity.
    pub quantity: ItemQuantity,
    /// The line subtotal.
    pub sub_total: i64,
    /// An optional description.
    pub description: Option<Description>,
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

impl ItemReceiveChild {
    /// Constructs a new `ItemReceiveChild`.
    pub fn fresh(
        id: ItemReceiveChildId,
        item_receive_id: ItemReceiveId,
        item_id: ItemId,
        unit_price: UnitPrice,
        quantity: ItemQuantity,
        description: Option<Description>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        let sub_total = unit_price.value().saturating_mul(quantity.value());
        Self {
            school_id: id.school_id(),
            id,
            item_receive_id,
            item_id,
            unit_price,
            quantity,
            sub_total,
            description,
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

use crate::value_objects::ItemReceiveChildId;

// =============================================================================
// Inventory movements: ItemIssue (header)
// =============================================================================

/// A goods-issue note. Records an issue of a quantity of an
/// `Item` to a recipient.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemIssue {
    /// The typed id (school_id + uuid).
    pub id: ItemIssueId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The academic year this issue is scoped to.
    pub academic_year_id: AcademicYearId,
    /// The item being issued.
    pub item_id: ItemId,
    /// The item's category (denormalized for fast lookups).
    pub item_category_id: ItemCategoryId,
    /// The recipient (staff / student / role).
    pub issue_to: IssueRecipient,
    /// The user who issued the item.
    pub issue_by: UserId,
    /// The date of issue.
    pub issue_date: NaiveDate,
    /// The optional due-back date.
    pub due_date: Option<NaiveDate>,
    /// The quantity issued.
    pub quantity: ItemQuantity,
    /// The issue status.
    pub issue_status: IssueStatus,
    /// The optional quantity returned so far.
    pub returned_quantity: ItemQuantity,
    /// A free-text note.
    pub note: Option<Note>,
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

impl ItemIssue {
    /// Constructs a new `ItemIssue` in the initial state.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: ItemIssueId,
        academic_year_id: AcademicYearId,
        item_id: ItemId,
        item_category_id: ItemCategoryId,
        issue_to: IssueRecipient,
        issue_by: UserId,
        issue_date: NaiveDate,
        due_date: Option<NaiveDate>,
        quantity: ItemQuantity,
        note: Option<Note>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            academic_year_id,
            item_id,
            item_category_id,
            issue_to,
            issue_by,
            issue_date,
            due_date,
            quantity,
            issue_status: IssueStatus::Issued,
            returned_quantity: ItemQuantity::ZERO,
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

    /// Returns a quantity that is still outstanding
    /// (issued - returned).
    #[must_use]
    pub fn outstanding_quantity(&self) -> ItemQuantity {
        ItemQuantity(
            self.quantity
                .value()
                .saturating_sub(self.returned_quantity.value()),
        )
    }
}

use crate::value_objects::ItemIssueId;

// =============================================================================
// Inventory movements: ItemSell (header)
// =============================================================================

/// A sale header. Records the sale of one or more items to a
/// staff member or a student, with totals, payment state, and
/// one or more `ItemSellChild` lines.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemSell {
    /// The typed id (school_id + uuid).
    pub id: ItemSellId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The academic year this sale is scoped to.
    pub academic_year_id: AcademicYearId,
    /// The buyer (staff / student / role).
    pub buyer: IssueRecipient,
    /// The sell date.
    pub sell_date: NaiveDate,
    /// An optional reference number.
    pub reference_no: Option<ReferenceNumber>,
    /// The total quantity sold.
    pub total_quantity: ItemQuantity,
    /// The grand total in minor units.
    pub grand_total: i64,
    /// The amount paid.
    pub total_paid: i64,
    /// The amount due.
    pub total_due: i64,
    /// The payment method.
    pub payment_method: PaymentMethod,
    /// The paid status.
    pub paid_status: PaidStatus,
    /// An optional description.
    pub description: Option<Description>,
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

impl ItemSell {
    /// Constructs a new `ItemSell` from pre-computed totals and
    /// child lines.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: ItemSellId,
        academic_year_id: AcademicYearId,
        buyer: IssueRecipient,
        sell_date: NaiveDate,
        reference_no: Option<ReferenceNumber>,
        total_quantity: ItemQuantity,
        grand_total: i64,
        total_paid: i64,
        payment_method: PaymentMethod,
        paid_status: PaidStatus,
        description: Option<Description>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        let total_due = grand_total.saturating_sub(total_paid);
        Self {
            school_id: id.school_id(),
            id,
            academic_year_id,
            buyer,
            sell_date,
            reference_no,
            total_quantity,
            grand_total,
            total_paid,
            total_due,
            payment_method,
            paid_status,
            description,
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

use crate::value_objects::ItemSellId;

// =============================================================================
// Inventory movements: ItemSellChild (line aggregate)
// =============================================================================

/// A single line on an `ItemSell`. Records the sell price,
/// quantity, and subtotal for a specific `Item` sold in the
/// sale.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemSellChild {
    /// The typed id (school_id + uuid).
    pub id: ItemSellChildId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The parent sale.
    pub item_sell_id: ItemSellId,
    /// The item being sold.
    pub item_id: ItemId,
    /// The sell price.
    pub sell_price: SellPrice,
    /// The quantity.
    pub quantity: ItemQuantity,
    /// The line subtotal.
    pub sub_total: i64,
    /// An optional description.
    pub description: Option<Description>,
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

impl ItemSellChild {
    /// Constructs a new `ItemSellChild`.
    pub fn fresh(
        id: ItemSellChildId,
        item_sell_id: ItemSellId,
        item_id: ItemId,
        sell_price: SellPrice,
        quantity: ItemQuantity,
        description: Option<Description>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        let sub_total = sell_price.value().saturating_mul(quantity.value());
        Self {
            school_id: id.school_id(),
            id,
            item_sell_id,
            item_id,
            sell_price,
            quantity,
            sub_total,
            description,
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

use crate::value_objects::ItemSellChildId;

// =============================================================================
// Supplier
// =============================================================================

/// A vendor contact master. Used as the supplier on
/// `ItemReceive` records and as the counterparty on finance
/// payables.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Supplier {
    /// The typed id (school_id + uuid).
    pub id: SupplierId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The supplier's name (unique within a school).
    pub company_name: SupplierName,
    /// The supplier's address.
    pub company_address: Option<Address>,
    /// The primary contact's name.
    pub contact_person_name: Option<ContactPersonName>,
    /// The primary contact's mobile.
    pub contact_person_mobile: Option<PhoneNumber>,
    /// The primary contact's email.
    pub contact_person_email: Option<EmailAddress>,
    /// The primary contact's address.
    pub contact_person_address: Option<Address>,
    /// A free-text description.
    pub description: Option<Description>,
    /// The supplier's status.
    pub status: SupplierStatus,
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

impl Supplier {
    /// Constructs a new `Supplier` in the initial active state.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: SupplierId,
        company_name: SupplierName,
        company_address: Option<Address>,
        contact_person_name: Option<ContactPersonName>,
        contact_person_mobile: Option<PhoneNumber>,
        contact_person_email: Option<EmailAddress>,
        contact_person_address: Option<Address>,
        description: Option<Description>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            company_name,
            company_address,
            contact_person_name,
            contact_person_mobile,
            contact_person_email,
            contact_person_address,
            description,
            status: SupplierStatus::Active,
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

    /// Deactivates the supplier. Returns `Err` if the supplier
    /// is already blacklisted.
    pub fn deactivate(
        &mut self,
        new_status: SupplierStatus,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) -> educore_core::error::Result<()> {
        if self.status == SupplierStatus::Blacklisted {
            return Err(educore_core::error::DomainError::conflict(
                "supplier is already blacklisted",
            ));
        }
        self.status = new_status;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
        Ok(())
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

    fn ctx() -> (SchoolId, UserId, Timestamp, CorrelationId) {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let actor = g.next_user_id();
        let corr = g.next_correlation_id();
        (school, actor, Timestamp::now(), corr)
    }

    fn year() -> AcademicYearId {
        let g = SystemIdGen;
        AcademicYearId::new(g.next_school_id(), g.next_uuid())
    }

    #[test]
    fn vehicle_fresh_starts_active() {
        let (school, user, at, corr) = ctx();
        let id = VehicleId::new(school, uuid::Uuid::now_v7());
        let v = Vehicle::fresh(
            id,
            year(),
            VehicleNumber::new("GJ-05-AB-1234").unwrap(),
            VehicleModel::new("Tata LP 909").unwrap(),
            None,
            None,
            user,
            at,
            corr,
        );
        assert_eq!(v.status, VehicleStatus::Active);
        assert_eq!(v.school_id, school);
    }

    #[test]
    fn vehicle_assign_driver_bumps_version() {
        let (school, user, at, corr) = ctx();
        let id = VehicleId::new(school, uuid::Uuid::now_v7());
        let mut v = Vehicle::fresh(
            id,
            year(),
            VehicleNumber::new("V-1").unwrap(),
            VehicleModel::new("Bus").unwrap(),
            None,
            None,
            user,
            at,
            corr,
        );
        let initial = v.version;
        let driver = StaffId::new(school, uuid::Uuid::now_v7());
        let ev = SystemIdGen.next_event_id();
        v.assign_driver(driver, user, at, ev);
        assert_eq!(v.driver_id, Some(driver));
        assert!(v.version > initial);
    }

    #[test]
    fn item_apply_stock_delta_rejects_negative() {
        let (school, user, at, corr) = ctx();
        let id = ItemId::new(school, uuid::Uuid::now_v7());
        let mut item = Item::fresh(
            id,
            year(),
            ItemName::new("Pen").unwrap(),
            ItemSku::new("PEN-001").unwrap(),
            ItemCategoryId::new(school, uuid::Uuid::now_v7()),
            None,
            user,
            at,
            corr,
        );
        let ev = SystemIdGen.next_event_id();
        item.apply_stock_delta(50, user, at, ev).unwrap();
        assert_eq!(item.total_in_stock.value(), 50);
        let err = item.apply_stock_delta(-100, user, at, ev).unwrap_err();
        assert!(matches!(err, educore_core::error::DomainError::Conflict(_)));
    }

    #[test]
    fn item_receive_line_subtotal_computes() {
        let (school, user, at, corr) = ctx();
        let id = ItemReceiveChildId::new(school, uuid::Uuid::now_v7());
        let parent = ItemReceiveId::new(school, uuid::Uuid::now_v7());
        let item = ItemId::new(school, uuid::Uuid::now_v7());
        let line = ItemReceiveChild::fresh(
            id,
            parent,
            item,
            UnitPrice(50),
            ItemQuantity(10),
            None,
            user,
            at,
            corr,
        );
        assert_eq!(line.sub_total, 500);
    }

    #[test]
    fn item_issue_outstanding_quantity() {
        let (school, user, at, corr) = ctx();
        let id = ItemIssueId::new(school, uuid::Uuid::now_v7());
        let item = ItemId::new(school, uuid::Uuid::now_v7());
        let cat = ItemCategoryId::new(school, uuid::Uuid::now_v7());
        let recipient = IssueRecipient::Role(educore_hr::value_objects::RoleId::new(
            school,
            uuid::Uuid::now_v7(),
        ));
        let issue = ItemIssue::fresh(
            id,
            year(),
            item,
            cat,
            recipient,
            user,
            chrono::NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
            None,
            ItemQuantity(10),
            None,
            user,
            at,
            corr,
        );
        assert_eq!(issue.outstanding_quantity().value(), 10);
    }

    #[test]
    fn supplier_rejects_already_blacklisted() {
        let (school, user, at, corr) = ctx();
        let id = SupplierId::new(school, uuid::Uuid::now_v7());
        let mut s = Supplier::fresh(
            id,
            SupplierName::new("Acme").unwrap(),
            None,
            None,
            None,
            None,
            None,
            None,
            user,
            at,
            corr,
        );
        s.deactivate(
            SupplierStatus::Blacklisted,
            user,
            at,
            SystemIdGen.next_event_id(),
        )
        .unwrap();
        let err = s
            .deactivate(
                SupplierStatus::Blacklisted,
                user,
                at,
                SystemIdGen.next_event_id(),
            )
            .unwrap_err();
        assert!(matches!(err, educore_core::error::DomainError::Conflict(_)));
    }
}
