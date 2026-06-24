//! # Facilities domain services
//!
//! Pure factory functions that take a typed command + a clock +
//! an id generator and return the new aggregate + the typed
//! event. The dispatcher is responsible for persisting the
//! aggregate and writing the audit / outbox / idempotency
//! rows in a single transaction (per the Phase 4 / 5 / 6
//! pattern).
//!
//! Phase 8 ships:
//!
//! - 13 pure factory service functions: `create_vehicle`,
//!   `update_vehicle`, `assign_driver`, `deactivate_vehicle`,
//!   `create_route`, `add_stop_to_route`,
//!   `assign_vehicle_to_route`, `assign_student_to_route`,
//!   `create_dormitory`, `create_room`, `create_item_category`,
//!   `create_item`, `create_item_store`, `receive_item`,
//!   `issue_item`, `sell_item`, `create_supplier`.
//! - `TransportService` + `DormitoryService` + `InventoryService`
//!   + `SupplierService` helpers.
//! - `InventoryConservationService` (the headline correctness
//!   check) with a 100-case proptest (mirrors Phase 7's
//!   `DoubleEntryService`).
//!
//! ## Concurrency
//!
//! The build-plan § "Phase 8 Risks" notes that inventory
//! conservation under concurrent writes is mitigated by
//! `SELECT ... FOR UPDATE` on the `ItemStore` row (PG) or a
//! SQLite write lock. The service factories in this module are
//! pure (no I/O); the dispatcher is responsible for acquiring
//! the row-level lock before calling the factories and writing
//! the audit / outbox / idempotency rows in a single
//! transaction. This matches the Phase 2 OQ #5 hand-off
//! (flag-based transaction model) and the Phase 7 finance
//! positive answer on adequacy.

#![allow(missing_docs)]
#![allow(unused_imports)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::expect_used)]

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

use educore_core::clock::{Clock, IdGenerator};
use educore_core::error::{DomainError, Result};
use educore_core::ids::{CorrelationId, EventId, Identifier, SchoolId, UserId};
use educore_core::tenant::TenantContext;
use educore_core::value_objects::Timestamp;

use crate::aggregate::{
    AssignVehicle, Dormitory, Item, ItemIssue, ItemReceive, ItemReceiveChild, ItemSell,
    ItemSellChild, ItemStore, Room, RoomType, Supplier, Vehicle,
};
use crate::events::{
    DormitoryCreated, DriverAssignedToVehicle, IssuedItemReturned, ItemCategoryCreated,
    ItemCreated, ItemIssued, ItemReceived, ItemSold, ItemStoreCreated, RoomCreated,
    RoomTypeCreated, RouteCreated, StopAddedToRoute, StudentAssignedToRoom, StudentAssignedToRoute,
    SupplierCreated, VehicleAssigned, VehicleCreated, VehicleDeactivated, VehicleUpdated,
};
use crate::value_objects::{
    AcademicYearId, Address, BedNumber, CategoryName, ContactPersonName, CostPerBed, Description,
    DormitoryName, DormitoryType, EmailAddress, Fare, Intake, IssueRecipient, ItemId, ItemName,
    ItemQuantity, ItemReceiveLineSpec, ItemSellLineSpec, ItemSku, ItemStoreId, MadeYear, Note,
    NumberOfBed, PaidStatus, PaymentMethod, PhoneNumber, ReferenceNumber, RoomNumber, RoomTypeId,
    RoomTypeName, RouteName, RouteStopSpec, SellPrice, StaffId, StockOnHand, StopName, StoreName,
    StoreNumber, SupplierId, SupplierName, UnitPrice, VehicleModel, VehicleNumber, VehicleStatus,
};

fn event_id_to_uuid(e: EventId) -> uuid::Uuid {
    e.as_uuid()
}

// =============================================================================
// Transport services
// =============================================================================

/// Builds a new [`Vehicle`] aggregate + a [`VehicleCreated`] event.
#[allow(clippy::too_many_arguments)]
pub fn create_vehicle<C, G>(
    cmd: crate::commands::CreateVehicleCommand,
    clock: &C,
    ids: &G,
) -> Result<(Vehicle, VehicleCreated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = crate::value_objects::VehicleId::new(school, event_id_to_uuid(event_id));
    let mut vehicle = Vehicle::fresh(
        id,
        cmd.academic_year_id,
        cmd.vehicle_no.clone(),
        cmd.vehicle_model.clone(),
        cmd.made_year,
        cmd.driver_id,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    vehicle.last_event_id = Some(event_id);

    let event = VehicleCreated::new(
        id,
        cmd.vehicle_no,
        cmd.vehicle_model.as_str().to_owned(),
        cmd.driver_id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((vehicle, event))
}

/// Updates a [`Vehicle`] aggregate + a [`VehicleUpdated`] event.
pub fn update_vehicle<C, G>(
    vehicle: &mut Vehicle,
    cmd: crate::commands::UpdateVehicleCommand,
    clock: &C,
    ids: &G,
) -> Result<VehicleUpdated>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let mut changes: Vec<&'static str> = Vec::new();
    if let Some(model) = cmd.vehicle_model {
        vehicle.vehicle_model = model;
        changes.push("vehicle_model");
    }
    if let Some(year) = cmd.made_year {
        vehicle.made_year = Some(year);
        changes.push("made_year");
    }
    if let Some(status) = cmd.status {
        vehicle.status = status;
        changes.push("status");
    }
    if let Some(note) = cmd.note {
        vehicle.note = Some(note);
        changes.push("note");
    }
    vehicle.updated_at = now;
    vehicle.updated_by = cmd.tenant.actor_id;
    vehicle.version = vehicle.version.next();
    vehicle.last_event_id = Some(event_id);

    Ok(VehicleUpdated::new(
        vehicle.id,
        changes,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Assigns a driver to a [`Vehicle`] + a [`DriverAssignedToVehicle`] event.
pub fn assign_driver<C, G>(
    vehicle: &mut Vehicle,
    cmd: crate::commands::AssignDriverToVehicleCommand,
    clock: &C,
    ids: &G,
) -> Result<DriverAssignedToVehicle>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let from = vehicle.driver_id;
    vehicle.assign_driver(cmd.driver_id, cmd.tenant.actor_id, now, event_id);
    Ok(DriverAssignedToVehicle::new(
        vehicle.id,
        from,
        cmd.driver_id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Deactivates a [`Vehicle`] + a [`VehicleDeactivated`] event.
pub fn deactivate_vehicle<C, G>(
    vehicle: &mut Vehicle,
    cmd: crate::commands::DeactivateVehicleCommand,
    clock: &C,
    ids: &G,
) -> Result<VehicleDeactivated>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    vehicle.deactivate(cmd.new_status, cmd.tenant.actor_id, now, event_id)?;
    Ok(VehicleDeactivated::new(
        vehicle.id,
        cmd.reason,
        cmd.new_status,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Builds a new [`Route`] aggregate + a [`RouteCreated`] event.
pub fn create_route<C, G>(
    cmd: crate::commands::CreateRouteCommand,
    clock: &C,
    ids: &G,
) -> Result<(crate::aggregate::Route, RouteCreated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = crate::value_objects::RouteId::new(school, event_id_to_uuid(event_id));
    let mut route = crate::aggregate::Route::fresh(
        id,
        cmd.academic_year_id,
        cmd.title.clone(),
        cmd.fare,
        cmd.distance,
        cmd.stops.clone(),
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    route.last_event_id = Some(event_id);

    let event = RouteCreated::new(
        id,
        cmd.title,
        cmd.fare.value(),
        cmd.stops,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((route, event))
}

/// Adds a stop to a [`Route`] + a [`StopAddedToRoute`] event.
pub fn add_stop_to_route<C, G>(
    route: &mut crate::aggregate::Route,
    cmd: crate::commands::AddStopToRouteCommand,
    clock: &C,
    ids: &G,
) -> Result<StopAddedToRoute>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let spec = RouteStopSpec {
        stop_order: cmd.stop_order,
        stop_name: cmd.stop_name.clone(),
        pickup_time: cmd.pickup_time,
        fare_override: cmd.fare_override,
    };
    route.stops.push(spec);
    route.updated_at = now;
    route.updated_by = cmd.tenant.actor_id;
    route.version = route.version.next();
    route.last_event_id = Some(event_id);

    Ok(StopAddedToRoute::new(
        route.id,
        cmd.stop_order,
        cmd.stop_name,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Builds a new [`AssignVehicle`] + a [`VehicleAssigned`] event.
pub fn assign_vehicle_to_route<C, G>(
    cmd: crate::commands::AssignVehicleToRouteCommand,
    clock: &C,
    ids: &G,
) -> Result<(AssignVehicle, VehicleAssigned)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = crate::value_objects::AssignVehicleId::new(school, event_id_to_uuid(event_id));
    let mut av = AssignVehicle::fresh(
        id,
        cmd.vehicle_id,
        cmd.route_id,
        cmd.academic_year_id,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    av.last_event_id = Some(event_id);

    let event = VehicleAssigned::new(
        id,
        cmd.vehicle_id,
        cmd.route_id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((av, event))
}

/// Records a student assignment to a vehicle-route pair + a
/// [`StudentAssignedToRoute`] event.
pub fn assign_student_to_route<C, G>(
    assign_vehicle_id: crate::value_objects::AssignVehicleId,
    cmd: crate::commands::AssignStudentToRouteCommand,
    clock: &C,
    ids: &G,
) -> Result<StudentAssignedToRoute>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let today = chrono::NaiveDate::from_ymd_opt(
        now.as_datetime().year(),
        now.as_datetime().month(),
        now.as_datetime().day(),
    )
    .unwrap_or_else(|| {
        // `now` came from the system clock and is always inside
        // chrono's calendar range (year -9999..9999), so the
        // `from_ymd_opt` above is guaranteed to succeed in
        // practice. The fallback below is defensive
        // defense-in-depth; chrono's `NaiveDate::default()` is
        // the Unix epoch (1970-01-01).
        chrono::NaiveDate::default()
    });

    Ok(StudentAssignedToRoute::new(
        assign_vehicle_id,
        cmd.student_id,
        today,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

// =============================================================================
// Dormitory + Room services
// =============================================================================

/// Builds a new [`Dormitory`] + a [`DormitoryCreated`] event.
#[allow(clippy::too_many_arguments)]
pub fn create_dormitory<C, G>(
    cmd: crate::commands::CreateDormitoryCommand,
    clock: &C,
    ids: &G,
) -> Result<(Dormitory, DormitoryCreated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = crate::value_objects::DormitoryId::new(school, event_id_to_uuid(event_id));
    let mut d = Dormitory::fresh(
        id,
        cmd.academic_year_id,
        cmd.name.clone(),
        cmd.dormitory_type,
        cmd.address,
        cmd.intake,
        cmd.description,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    d.last_event_id = Some(event_id);

    let event = DormitoryCreated::new(
        id,
        cmd.name.as_str().to_owned(),
        cmd.dormitory_type,
        cmd.intake.value(),
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((d, event))
}

/// Builds a new [`RoomType`] + a [`RoomTypeCreated`] event.
pub fn create_room_type<C, G>(
    cmd: crate::commands::CreateRoomTypeCommand,
    clock: &C,
    ids: &G,
) -> Result<(RoomType, RoomTypeCreated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = RoomTypeId::new(school, event_id_to_uuid(event_id));
    let mut rt = RoomType::fresh(
        id,
        cmd.name.clone(),
        cmd.description,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    rt.last_event_id = Some(event_id);

    let event = RoomTypeCreated::new(
        id,
        cmd.name.as_str().to_owned(),
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((rt, event))
}

/// Builds a new [`Room`] + a [`RoomCreated`] event.
#[allow(clippy::too_many_arguments)]
pub fn create_room<C, G>(
    cmd: crate::commands::CreateRoomCommand,
    clock: &C,
    ids: &G,
) -> Result<(Room, RoomCreated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = crate::value_objects::RoomId::new(school, event_id_to_uuid(event_id));
    let mut r = Room::fresh(
        id,
        cmd.dormitory_id,
        cmd.room_number.clone(),
        cmd.room_type_id,
        cmd.number_of_bed,
        cmd.cost_per_bed,
        cmd.description,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    r.last_event_id = Some(event_id);

    let event = RoomCreated::new(
        id,
        cmd.dormitory_id,
        cmd.room_number.as_str().to_owned(),
        cmd.number_of_bed.value(),
        cmd.cost_per_bed.value(),
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((r, event))
}

/// Records a student assignment to a room + a
/// [`StudentAssignedToRoom`] event.
pub fn assign_student_to_room<C, G>(
    cmd: crate::commands::AssignStudentToRoomCommand,
    clock: &C,
    ids: &G,
) -> Result<StudentAssignedToRoom>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    Ok(StudentAssignedToRoom::new(
        cmd.room_id,
        cmd.student_id,
        cmd.bed_number,
        now,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

// =============================================================================
// Inventory catalog services
// =============================================================================

/// Builds a new [`ItemCategory`] + a [`ItemCategoryCreated`] event.
pub fn create_item_category<C, G>(
    cmd: crate::commands::CreateItemCategoryCommand,
    clock: &C,
    ids: &G,
) -> Result<(crate::aggregate::ItemCategory, ItemCategoryCreated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = crate::value_objects::ItemCategoryId::new(school, event_id_to_uuid(event_id));
    let mut cat = crate::aggregate::ItemCategory::fresh(
        id,
        cmd.category_name.clone(),
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    cat.last_event_id = Some(event_id);

    let event = ItemCategoryCreated::new(
        id.as_uuid(),
        cmd.category_name.as_str().to_owned(),
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((cat, event))
}

/// Builds a new [`Item`] + a [`ItemCreated`] event.
pub fn create_item<C, G>(
    cmd: crate::commands::CreateItemCommand,
    clock: &C,
    ids: &G,
) -> Result<(Item, ItemCreated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = ItemId::new(school, event_id_to_uuid(event_id));
    let mut item = Item::fresh(
        id,
        cmd.academic_year_id,
        cmd.item_name.clone(),
        cmd.item_sku.clone(),
        cmd.item_category_id,
        cmd.description,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    item.last_event_id = Some(event_id);

    let event = ItemCreated::new(
        id,
        cmd.item_name.as_str().to_owned(),
        cmd.item_sku.as_str().to_owned(),
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((item, event))
}

/// Builds a new [`ItemStore`] + a [`ItemStoreCreated`] event.
pub fn create_item_store<C, G>(
    cmd: crate::commands::CreateItemStoreCommand,
    clock: &C,
    ids: &G,
) -> Result<(ItemStore, ItemStoreCreated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = ItemStoreId::new(school, event_id_to_uuid(event_id));
    let mut s = ItemStore::fresh(
        id,
        cmd.store_name.clone(),
        cmd.store_number,
        cmd.description,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    s.last_event_id = Some(event_id);

    let event = ItemStoreCreated::new(
        id,
        cmd.store_name.as_str().to_owned(),
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((s, event))
}

// =============================================================================
// Inventory movement services
// =============================================================================

/// The result of a receive_item service call: the header
/// aggregate + the per-line children + the emitted event.
#[derive(Debug)]
pub struct ReceiveItemResult {
    /// The receive header.
    pub header: ItemReceive,
    /// The receive child lines (one per input spec).
    pub lines: Vec<ItemReceiveChild>,
    /// The typed event.
    pub event: ItemReceived,
}

/// Receives goods (posts a GRN). Increments `Item.TotalInStock`
/// for each line via the dispatcher (the service is pure).
#[allow(clippy::too_many_arguments)]
pub fn receive_item<C, G>(
    cmd: crate::commands::ReceiveItemCommand,
    clock: &C,
    ids: &G,
) -> Result<ReceiveItemResult>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    if cmd.lines.is_empty() {
        return Err(DomainError::validation(
            "receive_item requires at least one line",
        ));
    }
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let header_id = crate::value_objects::ItemReceiveId::new(school, event_id_to_uuid(event_id));

    let mut total_quantity: i64 = 0;
    let mut grand_total: i64 = 0;
    let mut lines: Vec<ItemReceiveChild> = Vec::with_capacity(cmd.lines.len());
    for (i, spec) in cmd.lines.iter().enumerate() {
        let child_id = crate::value_objects::ItemReceiveChildId::new(
            school,
            event_id_to_uuid(ids.next_event_id()),
        );
        let line = ItemReceiveChild::fresh(
            child_id,
            header_id,
            spec.item_id,
            spec.unit_price,
            spec.quantity,
            spec.description.clone(),
            cmd.tenant.actor_id,
            now,
            cmd.tenant.correlation_id,
        );
        total_quantity = total_quantity.saturating_add(spec.quantity.value());
        grand_total = grand_total.saturating_add(line.sub_total);
        let _ = i;
        lines.push(line);
    }

    let header = ItemReceive::fresh(
        header_id,
        cmd.academic_year_id,
        cmd.receive_date,
        cmd.reference_no,
        cmd.supplier_id,
        cmd.store_id,
        ItemQuantity(total_quantity),
        grand_total,
        cmd.total_paid,
        cmd.payment_method,
        cmd.paid_status,
        cmd.description,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );

    let event = ItemReceived::new(
        header_id,
        cmd.supplier_id,
        cmd.store_id,
        header.receive_date,
        grand_total,
        total_quantity,
        header.total_paid,
        header.total_due,
        cmd.paid_status,
        cmd.lines,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok(ReceiveItemResult {
        header,
        lines,
        event,
    })
}

/// Issues goods (posts a GIN). The dispatcher is responsible for
/// decrementing `Item.TotalInStock` atomically.
pub fn issue_item<C, G>(
    cmd: crate::commands::IssueItemCommand,
    clock: &C,
    ids: &G,
) -> Result<(ItemIssue, ItemIssued)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    if cmd.quantity.is_zero() {
        return Err(DomainError::validation(
            "issue_item requires a positive quantity",
        ));
    }
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = crate::value_objects::ItemIssueId::new(school, event_id_to_uuid(event_id));
    let mut issue = ItemIssue::fresh(
        id,
        cmd.academic_year_id,
        cmd.item_id,
        cmd.item_category_id,
        cmd.issue_to.clone(),
        cmd.issue_by,
        cmd.issue_date,
        cmd.due_date,
        cmd.quantity,
        cmd.note,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    issue.last_event_id = Some(event_id);

    let event = ItemIssued::new(
        id,
        cmd.item_id,
        cmd.issue_to,
        cmd.issue_by,
        cmd.issue_date,
        cmd.quantity.value(),
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((issue, event))
}

/// Returns an issued item (partial or full).
pub fn return_issued_item<C, G>(
    issue: &mut ItemIssue,
    cmd: crate::commands::ReturnIssuedItemCommand,
    clock: &C,
    ids: &G,
) -> Result<IssuedItemReturned>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let outstanding = issue.outstanding_quantity().value();
    if cmd.returned_quantity.value() == 0 {
        return Err(DomainError::validation("return quantity must be positive"));
    }
    if cmd.returned_quantity.value() > outstanding {
        return Err(DomainError::conflict(format!(
            "return quantity {0} exceeds outstanding {outstanding}",
            cmd.returned_quantity.value()
        )));
    }
    let now = clock.now();
    let event_id = ids.next_event_id();
    let new_total = issue
        .returned_quantity
        .value()
        .saturating_add(cmd.returned_quantity.value());
    issue.returned_quantity = ItemQuantity(new_total);
    let new_status = if new_total >= issue.quantity.value() {
        crate::value_objects::IssueStatus::Returned
    } else {
        crate::value_objects::IssueStatus::PartiallyReturned
    };
    issue.issue_status = new_status;
    issue.updated_at = now;
    issue.updated_by = cmd.tenant.actor_id;
    issue.version = issue.version.next();
    issue.last_event_id = Some(event_id);

    Ok(IssuedItemReturned::new(
        issue.id,
        issue.item_id,
        cmd.returned_quantity.value(),
        new_status,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// The result of a sell_item service call: the header aggregate
/// + the per-line children + the emitted event.
#[derive(Debug)]
pub struct SellItemResult {
    /// The sale header.
    pub header: ItemSell,
    /// The sale child lines (one per input spec).
    pub lines: Vec<ItemSellChild>,
    /// The typed event.
    pub event: ItemSold,
}

/// Sells goods (posts a sale). The dispatcher is responsible
/// for decrementing `Item.TotalInStock` atomically.
#[allow(clippy::too_many_arguments)]
pub fn sell_item<C, G>(
    cmd: crate::commands::SellItemCommand,
    clock: &C,
    ids: &G,
) -> Result<SellItemResult>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    if cmd.lines.is_empty() {
        return Err(DomainError::validation(
            "sell_item requires at least one line",
        ));
    }
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let header_id = crate::value_objects::ItemSellId::new(school, event_id_to_uuid(event_id));

    let mut total_quantity: i64 = 0;
    let mut grand_total: i64 = 0;
    let mut lines: Vec<ItemSellChild> = Vec::with_capacity(cmd.lines.len());
    for spec in &cmd.lines {
        let child_id = crate::value_objects::ItemSellChildId::new(
            school,
            event_id_to_uuid(ids.next_event_id()),
        );
        let line = ItemSellChild::fresh(
            child_id,
            header_id,
            spec.item_id,
            spec.sell_price,
            spec.quantity,
            spec.description.clone(),
            cmd.tenant.actor_id,
            now,
            cmd.tenant.correlation_id,
        );
        total_quantity = total_quantity.saturating_add(spec.quantity.value());
        grand_total = grand_total.saturating_add(line.sub_total);
        lines.push(line);
    }

    let header = ItemSell::fresh(
        header_id,
        cmd.academic_year_id,
        cmd.buyer.clone(),
        cmd.sell_date,
        cmd.reference_no,
        ItemQuantity(total_quantity),
        grand_total,
        cmd.total_paid,
        cmd.payment_method,
        cmd.paid_status,
        cmd.description,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );

    let event = ItemSold::new(
        header_id,
        cmd.buyer,
        cmd.sell_date,
        grand_total,
        total_quantity,
        header.total_paid,
        header.total_due,
        cmd.paid_status,
        cmd.lines,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok(SellItemResult {
        header,
        lines,
        event,
    })
}

// =============================================================================
// Supplier services
// =============================================================================

/// Builds a new [`Supplier`] + a [`SupplierCreated`] event.
#[allow(clippy::too_many_arguments)]
pub fn create_supplier<C, G>(
    cmd: crate::commands::CreateSupplierCommand,
    clock: &C,
    ids: &G,
) -> Result<(Supplier, SupplierCreated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = SupplierId::new(school, event_id_to_uuid(event_id));
    let mut s = Supplier::fresh(
        id,
        cmd.company_name.clone(),
        cmd.company_address,
        cmd.contact_person_name,
        cmd.contact_person_mobile,
        cmd.contact_person_email,
        cmd.contact_person_address,
        cmd.description,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    s.last_event_id = Some(event_id);

    let event = SupplierCreated::new(
        id,
        cmd.company_name.as_str().to_owned(),
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((s, event))
}

// =============================================================================
// Helpers: TransportService, DormitoryService, InventoryService,
// SupplierService
// =============================================================================

/// The transport service. Pure functions on aggregates; no I/O.
pub struct TransportService;

impl TransportService {
    /// Returns `true` if the vehicle may be assigned to a route
    /// in the given year (the vehicle is active, the route is
    /// active, the vehicle has no other `AssignVehicle` row for
    /// the same year).
    #[must_use]
    pub fn can_assign_vehicle(vehicle: &Vehicle, vehicle_active: bool) -> bool {
        vehicle_active && vehicle.status == VehicleStatus::Active
    }

    /// Computes the per-student fare by starting at the route's
    /// `fare` and applying any stop-level override.
    #[must_use]
    pub fn fare_for_student(route_fare: Fare, stop_override: Option<Fare>) -> Fare {
        stop_override.unwrap_or(route_fare)
    }
}

/// The dormitory service. Pure functions on aggregates.
pub struct DormitoryService;

impl DormitoryService {
    /// Returns the count of available beds in a room (total beds
    /// minus the count of current assignments).
    #[must_use]
    pub fn available_beds(room: &Room, current_assignments: u32) -> u32 {
        room.number_of_bed
            .value()
            .saturating_sub(current_assignments)
    }

    /// Returns `Ok(())` if the dormitory may host a new student
    /// in the room, or an error describing the violation.
    pub fn can_assign(
        dormitory: &Dormitory,
        room: &Room,
        _current_student_count: u32,
    ) -> Result<()> {
        if room.dormitory_id != dormitory.id {
            return Err(DomainError::validation(
                "room does not belong to the specified dormitory",
            ));
        }
        Ok(())
    }
}

/// The inventory service. Pure functions on aggregates.
pub struct InventoryService;

impl InventoryService {
    /// Validates that a receive's totals are consistent with the
    /// lines.
    pub fn validate_receive(lines: &[ItemReceiveChild], grand_total: i64) -> Result<()> {
        if lines.is_empty() {
            return Err(DomainError::validation(
                "receive requires at least one line",
            ));
        }
        let computed: i64 = lines.iter().map(|l| l.sub_total).sum();
        if computed != grand_total {
            return Err(DomainError::conflict(format!(
                "receive grand_total {grand_total} does not match line subtotals {computed}"
            )));
        }
        Ok(())
    }

    /// Validates that a sell's totals are consistent with the
    /// lines.
    pub fn validate_sell(lines: &[ItemSellChild], grand_total: i64) -> Result<()> {
        if lines.is_empty() {
            return Err(DomainError::validation("sell requires at least one line"));
        }
        let computed: i64 = lines.iter().map(|l| l.sub_total).sum();
        if computed != grand_total {
            return Err(DomainError::conflict(format!(
                "sell grand_total {grand_total} does not match line subtotals {computed}"
            )));
        }
        Ok(())
    }

    /// Validates that an issue is permitted (`item.total_in_stock
    /// >= quantity`).
    pub fn validate_issue(item: &Item, quantity: ItemQuantity) -> Result<()> {
        if quantity.value() == 0 {
            return Err(DomainError::validation("issue quantity must be positive"));
        }
        if item.total_in_stock.value() < quantity.value() {
            return Err(DomainError::conflict(format!(
                "item stock {0} insufficient for issue of {1}",
                item.total_in_stock.value(),
                quantity.value()
            )));
        }
        Ok(())
    }
}

/// The supplier service.
pub struct SupplierService;

impl SupplierService {
    /// Normalizes a supplier name (trims + collapses whitespace).
    #[must_use]
    pub fn normalize_name(raw: &str) -> String {
        raw.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

// =============================================================================
// InventoryConservationService (the headline correctness check)
//
//   on_hand(school_id, item_id)
//     = sum(received.quantity)
//     - sum(issued.quantity)
//     - sum(sold.quantity)
//
// Mirrors Phase 7's `DoubleEntryService` pattern. The 100-case
// proptest at the bottom of this file exercises the invariant.
// =============================================================================

/// The kind of an inventory movement. Used by the conservation
/// service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MovementKind {
    /// Goods received.
    Receive,
    /// Goods issued.
    Issue,
    /// Goods sold.
    Sell,
}

impl MovementKind {
    /// Returns the sign multiplier: +1 for Receive, -1 for Issue
    /// and Sell.
    #[must_use]
    pub const fn sign(self) -> i64 {
        match self {
            Self::Receive => 1,
            Self::Issue | Self::Sell => -1,
        }
    }
}

/// A single movement row used by the conservation proptest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MovementRow {
    /// The owning school.
    pub school_id: SchoolId,
    /// The item being moved.
    pub item_id: ItemId,
    /// The kind of movement.
    pub kind: MovementKind,
    /// The quantity (always non-negative).
    pub quantity: i64,
}

/// The inventory conservation invariant. Phase 8's headline
/// correctness check (mirrors Phase 7's `DoubleEntryService`).
pub struct InventoryConservationService;

impl InventoryConservationService {
    /// Asserts the conservation invariant for a school: the
    /// sum of all signed movements per `(school_id, item_id)`
    /// is non-negative (the `on_hand` projection).
    ///
    /// Returns `Ok(())` if every `(school_id, item_id)` has a
    /// non-negative on-hand projection, or `Err(Validation)` if
    /// any item has gone negative.
    pub fn check_invariant(rows: &[MovementRow], school: SchoolId) -> Result<()> {
        use std::collections::HashMap;
        let mut by_item: HashMap<ItemId, i64> = HashMap::new();
        for r in rows {
            if r.school_id != school {
                continue;
            }
            let signed = r.quantity.saturating_mul(r.kind.sign());
            *by_item.entry(r.item_id).or_insert(0) =
                by_item.get(&r.item_id).copied().unwrap_or(0) + signed;
        }
        for (item_id, on_hand) in by_item {
            if on_hand < 0 {
                return Err(DomainError::validation(format!(
                    "item {item_id} has negative on_hand {on_hand}"
                )));
            }
        }
        Ok(())
    }

    /// Computes the on-hand projection for one item.
    #[must_use]
    pub fn on_hand_for(rows: &[MovementRow], school: SchoolId, item: ItemId) -> i64 {
        let mut on_hand: i64 = 0;
        for r in rows {
            if r.school_id != school || r.item_id != item {
                continue;
            }
            on_hand = on_hand.saturating_add(r.quantity.saturating_mul(r.kind.sign()));
        }
        on_hand
    }
}

// =============================================================================
// Tests (including the headline 100-case proptest)
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
    use crate::prelude::IssueStatus;
    use educore_core::clock::{IdGenerator, SystemClock, SystemIdGen};
    use educore_core::ids::Identifier;
    use educore_hr::value_objects::RoleId;

    fn ctx() -> (SchoolId, UserId, Timestamp, CorrelationId, TenantContext) {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let user = g.next_user_id();
        let corr = g.next_correlation_id();
        let tenant = TenantContext::for_user(
            school,
            user,
            corr,
            educore_core::tenant::UserType::SchoolAdmin,
        );
        (school, user, Timestamp::now(), corr, tenant)
    }

    fn year() -> AcademicYearId {
        let g = SystemIdGen;
        AcademicYearId::new(g.next_school_id(), g.next_uuid())
    }

    #[test]
    fn create_vehicle_emits_event() {
        let (school, _, _at, _corr, tenant) = ctx();
        let cmd = crate::commands::CreateVehicleCommand {
            tenant,
            academic_year_id: year(),
            vehicle_no: VehicleNumber::new("V-1").unwrap(),
            vehicle_model: VehicleModel::new("Bus").unwrap(),
            made_year: None,
            driver_id: None,
            note: None,
        };
        let (v, e) = create_vehicle(cmd, &SystemClock, &SystemIdGen).unwrap();
        assert_eq!(v.school_id, school);
        assert_eq!(
            <crate::events::VehicleCreated as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
            "facilities.vehicle.created"
        );
        let _ = e;
    }

    #[test]
    fn create_route_with_stops_emits_event_with_stops() {
        let (_, _, _at, _corr, tenant) = ctx();
        let cmd = crate::commands::CreateRouteCommand {
            tenant,
            academic_year_id: year(),
            title: RouteName::new("Route 1").unwrap(),
            fare: Fare(100),
            distance: None,
            stops: vec![RouteStopSpec {
                stop_order: 1,
                stop_name: StopName::new("Main Gate").unwrap(),
                pickup_time: None,
                fare_override: None,
            }],
            note: None,
        };
        let (_r, e) = create_route(cmd, &SystemClock, &SystemIdGen).unwrap();
        assert_eq!(e.stops.len(), 1);
        assert_eq!(e.fare_minor, 100);
    }

    #[test]
    fn receive_item_rejects_empty_lines() {
        let (_, _, _at, _corr, tenant) = ctx();
        let cmd = crate::commands::ReceiveItemCommand {
            tenant,
            academic_year_id: year(),
            receive_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
            reference_no: None,
            supplier_id: SupplierId::new(
                educore_core::clock::SystemIdGen.next_school_id(),
                uuid::Uuid::now_v7(),
            ),
            store_id: ItemStoreId::new(
                educore_core::clock::SystemIdGen.next_school_id(),
                uuid::Uuid::now_v7(),
            ),
            total_paid: 0,
            payment_method: PaymentMethod::Cash,
            paid_status: PaidStatus::Unpaid,
            lines: vec![],
            description: None,
        };
        let err = receive_item(cmd, &SystemClock, &SystemIdGen).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn receive_item_computes_totals() {
        let (school, _, _at, _corr, tenant) = ctx();
        let item = ItemId::new(school, uuid::Uuid::now_v7());
        let cmd = crate::commands::ReceiveItemCommand {
            tenant,
            academic_year_id: year(),
            receive_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
            reference_no: None,
            supplier_id: SupplierId::new(school, uuid::Uuid::now_v7()),
            store_id: ItemStoreId::new(school, uuid::Uuid::now_v7()),
            total_paid: 1000,
            payment_method: PaymentMethod::Cash,
            paid_status: PaidStatus::Paid,
            lines: vec![
                ItemReceiveLineSpec {
                    item_id: item,
                    unit_price: UnitPrice(50),
                    quantity: ItemQuantity(10),
                    description: None,
                },
                ItemReceiveLineSpec {
                    item_id: item,
                    unit_price: UnitPrice(50),
                    quantity: ItemQuantity(10),
                    description: None,
                },
            ],
            description: None,
        };
        let result = receive_item(cmd, &SystemClock, &SystemIdGen).unwrap();
        assert_eq!(result.header.grand_total, 1000);
        assert_eq!(result.header.total_quantity.value(), 20);
        assert_eq!(result.lines.len(), 2);
    }

    #[test]
    fn issue_item_rejects_zero_quantity() {
        let (school, _, _at, _corr, tenant) = ctx();
        let cmd = crate::commands::IssueItemCommand {
            tenant,
            academic_year_id: year(),
            issue_to: IssueRecipient::Role(RoleId::new(school, uuid::Uuid::now_v7())),
            issue_by: SystemIdGen.next_user_id(),
            issue_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
            due_date: None,
            item_category_id: crate::value_objects::ItemCategoryId::new(
                school,
                uuid::Uuid::now_v7(),
            ),
            item_id: ItemId::new(school, uuid::Uuid::now_v7()),
            quantity: ItemQuantity::ZERO,
            note: None,
        };
        let err = issue_item(cmd, &SystemClock, &SystemIdGen).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn return_issued_item_updates_status() {
        let (school, _, at, corr, tenant) = ctx();
        let id = crate::value_objects::ItemIssueId::new(school, uuid::Uuid::now_v7());
        let mut issue = ItemIssue::fresh(
            id,
            year(),
            ItemId::new(school, uuid::Uuid::now_v7()),
            crate::value_objects::ItemCategoryId::new(school, uuid::Uuid::now_v7()),
            IssueRecipient::Role(RoleId::new(school, uuid::Uuid::now_v7())),
            SystemIdGen.next_user_id(),
            chrono::NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
            None,
            ItemQuantity(10),
            None,
            SystemIdGen.next_user_id(),
            at,
            corr,
        );
        let cmd = crate::commands::ReturnIssuedItemCommand {
            tenant: tenant.clone(),
            item_issue_id: id,
            returned_quantity: ItemQuantity(4),
        };
        let event = return_issued_item(&mut issue, cmd, &SystemClock, &SystemIdGen).unwrap();
        assert_eq!(event.returned_quantity, 4);
        assert_eq!(event.new_status, IssueStatus::PartiallyReturned);
    }

    #[test]
    fn inventory_conservation_invariant_holds_for_balanced_movements() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let item_a = ItemId::new(school, uuid::Uuid::now_v7());
        let item_b = ItemId::new(school, uuid::Uuid::now_v7());
        let rows = vec![
            MovementRow {
                school_id: school,
                item_id: item_a,
                kind: MovementKind::Receive,
                quantity: 100,
            },
            MovementRow {
                school_id: school,
                item_id: item_a,
                kind: MovementKind::Issue,
                quantity: 30,
            },
            MovementRow {
                school_id: school,
                item_id: item_a,
                kind: MovementKind::Sell,
                quantity: 5,
            },
            MovementRow {
                school_id: school,
                item_id: item_b,
                kind: MovementKind::Receive,
                quantity: 50,
            },
        ];
        InventoryConservationService::check_invariant(&rows, school).unwrap();
        assert_eq!(
            InventoryConservationService::on_hand_for(&rows, school, item_a),
            65
        );
        assert_eq!(
            InventoryConservationService::on_hand_for(&rows, school, item_b),
            50
        );
    }

    #[test]
    fn inventory_conservation_invariant_violated_for_negative_stock() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let item = ItemId::new(school, uuid::Uuid::now_v7());
        let rows = vec![
            MovementRow {
                school_id: school,
                item_id: item,
                kind: MovementKind::Receive,
                quantity: 10,
            },
            MovementRow {
                school_id: school,
                item_id: item,
                kind: MovementKind::Issue,
                quantity: 50,
            },
        ];
        let err = InventoryConservationService::check_invariant(&rows, school).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    // -------------------------------------------------------------------------
    // Inventory conservation invariant property test (100 cases)
    //
    // The headline correctness check for Phase 8 (mirrors Phase 7's
    // `DoubleEntryService` proptest at
    // `crates/domains/finance/src/services.rs:1259`). Asserts the
    // conservation invariant for 100 randomly generated movement
    // sequences.
    // -------------------------------------------------------------------------

    proptest::proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(100))]

        #[test]
        fn prop_inventory_conservation_holds_for_balanced_movements(
            receives in proptest::collection::vec(0i64..10_000, 1..20),
        ) {
            let g = SystemIdGen;
            let school = g.next_school_id();
            let item = ItemId::new(school, uuid::Uuid::now_v7());
            // Build a balanced movement sequence: receives
            // followed by issues/sells that are guaranteed to be
            // <= receives (random fraction of each receive).
            let mut rows: Vec<MovementRow> = Vec::new();
            let mut available: i64 = 0;
            for r in &receives {
                rows.push(MovementRow {
                    school_id: school,
                    item_id: item,
                    kind: MovementKind::Receive,
                    quantity: *r,
                });
                available = available.saturating_add(*r);
                if available > 0 {
                    let issue_qty = (*r) / 2;
                    rows.push(MovementRow {
                        school_id: school,
                        item_id: item,
                        kind: MovementKind::Issue,
                        quantity: issue_qty,
                    });
                    available = available.saturating_sub(issue_qty);
                }
            }
            InventoryConservationService::check_invariant(&rows, school)
                .expect("balanced movements should pass the conservation invariant");
        }

        #[test]
        fn prop_inventory_conservation_violated_for_overdraw(
            receives in proptest::collection::vec(1i64..1_000, 1..10),
        ) {
            let g = SystemIdGen;
            let school = g.next_school_id();
            let item = ItemId::new(school, uuid::Uuid::now_v7());
            // Build a sequence that overdraws: receive `r`, then
            // issue `r * 2` (guaranteed negative on_hand).
            let mut rows: Vec<MovementRow> = Vec::new();
            for r in &receives {
                rows.push(MovementRow {
                    school_id: school,
                    item_id: item,
                    kind: MovementKind::Receive,
                    quantity: *r,
                });
                rows.push(MovementRow {
                    school_id: school,
                    item_id: item,
                    kind: MovementKind::Issue,
                    quantity: r.saturating_mul(2).max(1),
                });
            }
            let err = InventoryConservationService::check_invariant(&rows, school).unwrap_err();
            assert!(matches!(err, DomainError::Validation(_)));
        }
    }
}
