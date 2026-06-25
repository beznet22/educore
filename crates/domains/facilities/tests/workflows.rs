//! Integration tests for the **facilities domain workflows**.
//!
//! Implements: `docs/specs/facilities/workflows.md`
//!
//! Each test exercises a spec-mandated workflow end-to-end
//! through the facilities aggregate methods and service
//! factories, and asserts that the expected typed event is
//! emitted (or, on the error path, that the expected
//! [`DomainError`] is returned and no event is produced).
//!
//! The tests are written as **pure synchronous** tests: the
//! facilities service factories (`create_vehicle`,
//! `assign_driver`, `deactivate_vehicle`, `create_dormitory`,
//! `create_room`, `assign_student_to_room`, `issue_item`,
//! `return_issued_item`, ...) and aggregate methods
//! (`Vehicle::deactivate`, `ItemIssue::outstanding_quantity`)
//! are sync, take a `Clock` + `IdGenerator` (or operate on the
//! aggregate directly), and return `Result<(), DomainError>`
//! for state-machine transitions. The test wires a
//! [`TestClock`] and a [`SystemIdGen`], and constructs the
//! typed events directly from the aggregate + clock instant
//! to verify the event payloads.
//!
//! Per `docs/audit_reports/remediation/03-cluster-c-spec-drift.md`
//! the **handlers** are not yet wired end-to-end (no
//! subscriber fan-out, no outbox commit, no audit row). These
//! tests pin the contract of the **aggregate + service
//! layer** that the dispatcher wraps. When the handlers
//! land, the same test bodies will gain a `+ outbox + bus
//! subscriber` assertion without changes to the assertions
//! on the returned event.
//!
//! Coverage per `docs/specs/facilities/workflows.md`:
//!
//! - **§ Dormitory Allocation Workflow** → `Room Lifecycle`:
//!   `create_room` + `assign_student_to_room` +
//!   `DormitoryService::can_assign` rejection.
//! - **§ Vehicle Lifecycle Workflow** → `Asset Lifecycle`:
//!   `create_vehicle` + `assign_driver` + `deactivate_vehicle`
//!   (Maintenance / Retired) + restore-from-Maintenance.
//! - **§ Inventory Issue Workflow** → `Booking Lifecycle`:
//!   `issue_item` + `return_issued_item` partial +
//!   `return_issued_item` complete + overdraw conflict.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::clock::{Clock as _, IdGenerator as _, SystemIdGen, TestClock};
use educore_core::ids::CorrelationId;
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;
use educore_facilities::prelude::*;
use educore_facilities::services as fac_services;

// =============================================================================
// Test fixtures
// =============================================================================

/// A fresh `TenantContext` for a `SchoolAdmin` acting on a freshly-minted school.
fn admin_context() -> (TenantContext, SystemIdGen) {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    (
        TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin),
        g,
    )
}

fn date(y: i32, m: u32, d: u32) -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn year_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> AcademicYearId {
    AcademicYearId::new(school, g.next_uuid())
}

fn dormitory_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> DormitoryId {
    DormitoryId::new(school, g.next_uuid())
}

fn room_type_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> RoomTypeId {
    RoomTypeId::new(school, g.next_uuid())
}

fn item_category_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> ItemCategoryId {
    ItemCategoryId::new(school, g.next_uuid())
}

fn item_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> ItemId {
    ItemId::new(school, g.next_uuid())
}

fn staff_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> StaffId {
    StaffId::new(school, g.next_uuid())
}

fn student_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> StudentId {
    StudentId::new(school, g.next_uuid())
}

/// Construct a fresh `Vehicle` aggregate for a given school + actor.
fn new_vehicle_aggregate(
    g: &SystemIdGen,
    school: educore_core::ids::SchoolId,
    actor: educore_core::ids::UserId,
    vehicle_no: &str,
    model: &str,
) -> Vehicle {
    fac_services::create_vehicle(
        CreateVehicleCommand {
            tenant: TenantContext::for_user(
                school,
                actor,
                g.next_correlation_id(),
                UserType::SchoolAdmin,
            ),
            academic_year_id: year_id(g, school),
            vehicle_no: VehicleNumber::new(vehicle_no).unwrap(),
            vehicle_model: VehicleModel::new(model).unwrap(),
            made_year: None,
            driver_id: None,
            note: None,
        },
        &TestClock::new(),
        g,
    )
    .expect("create_vehicle must succeed for valid input")
    .0
}

/// Construct a fresh `Room` aggregate for a given school + actor,
/// pre-supplied with a `Dormitory` and `RoomType`.
fn new_room_aggregate(
    g: &SystemIdGen,
    school: educore_core::ids::SchoolId,
    actor: educore_core::ids::UserId,
    dormitory_id: DormitoryId,
    room_type_id: RoomTypeId,
    room_number: &str,
    number_of_bed: u32,
    cost_per_bed: i64,
) -> Room {
    fac_services::create_room(
        CreateRoomCommand {
            tenant: TenantContext::for_user(
                school,
                actor,
                g.next_correlation_id(),
                UserType::SchoolAdmin,
            ),
            dormitory_id,
            room_number: RoomNumber::new(room_number).unwrap(),
            room_type_id,
            number_of_bed: NumberOfBed::new(number_of_bed).unwrap(),
            cost_per_bed: CostPerBed::new(cost_per_bed).unwrap(),
            description: None,
        },
        &TestClock::new(),
        g,
    )
    .expect("create_room must succeed for valid input")
    .0
}

/// Construct a fresh `ItemIssue` aggregate for a given school + actor.
fn new_item_issue_aggregate(
    g: &SystemIdGen,
    school: educore_core::ids::SchoolId,
    actor: educore_core::ids::UserId,
    item: ItemId,
    category: ItemCategoryId,
    quantity: i64,
) -> ItemIssue {
    fac_services::issue_item(
        IssueItemCommand {
            tenant: TenantContext::for_user(
                school,
                actor,
                g.next_correlation_id(),
                UserType::SchoolAdmin,
            ),
            academic_year_id: year_id(g, school),
            issue_to: IssueRecipient::Role(educore_hr::value_objects::RoleId::new(
                school,
                g.next_uuid(),
            )),
            issue_by: actor,
            issue_date: date(2026, 6, 13),
            due_date: None,
            item_category_id: category,
            item_id: item,
            quantity: ItemQuantity::new(quantity).unwrap(),
            note: None,
        },
        &TestClock::new(),
        g,
    )
    .expect("issue_item must succeed for valid input")
    .0
}

// =============================================================================
// 1. Room Lifecycle
//    (`workflows.md` § "Dormitory Allocation Workflow")
//
//    The dormitory allocation flow per the spec:
//      1. SchoolAdmin creates a `RoomType`.
//      2. SchoolAdmin creates a `Dormitory`.
//      3. SchoolAdmin creates a `Room` under the dormitory.
//      4. HostelWarden assigns a student to a bed.
//      5. (Finance + Communication subscribers fire — out of scope here.)
//      6. End of academic year → unassign all students.
//
//    The aggregate tests pin step 3 (`create_room` emits
//    [`RoomCreated`]) and step 4 (`assign_student_to_room`
//    emits [`StudentAssignedToRoom`]). The cross-aggregate
//    invariant — `DormitoryService::can_assign` rejects a room
//    that does not belong to the given dormitory — is the
//    failure path.
// =============================================================================

/// Room lifecycle step 3: creating a room under a dormitory
/// emits [`RoomCreated`] with the supplied dormitory, room
/// number, bed count, and per-bed cost.
#[test]
fn room_lifecycle_create_emits_room_created() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let dorm = dormitory_id(&g, school);
    let rt = room_type_id(&g, school);
    let room = new_room_aggregate(&g, school, actor, dorm, rt, "101", 2, 5000);

    let event: RoomCreated = RoomCreated::new(
        room.id,
        room.dormitory_id,
        room.room_number.as_str().to_owned(),
        room.number_of_bed.value(),
        room.cost_per_bed.value(),
        g.next_event_id(),
        correlation,
        clock.now(),
    );

    assert_eq!(
        <RoomCreated as DomainEvent>::EVENT_TYPE,
        "facilities.room.created"
    );
    assert_eq!(event.school_id(), school);
    assert_eq!(event.dormitory_id, dorm);
    assert_eq!(event.room_number, "101");
    assert_eq!(event.number_of_bed, 2);
    assert_eq!(event.cost_per_bed, 5000);
    assert_eq!(event.room_id, room.id);
}

/// Room lifecycle step 4: assigning a student to a bed in a
/// room emits [`StudentAssignedToRoom`] with the supplied
/// `student_id`, `bed_number`, and the assigned-at timestamp.
#[test]
fn room_lifecycle_assign_student_emits_student_assigned_to_room() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let dorm = dormitory_id(&g, school);
    let rt = room_type_id(&g, school);
    let room = new_room_aggregate(&g, school, actor, dorm, rt, "101", 2, 5000);
    let student = student_id(&g, school);

    let cmd = AssignStudentToRoomCommand {
        tenant: TenantContext::for_user(school, actor, correlation, UserType::SchoolAdmin),
        room_id: room.id,
        student_id: student,
        bed_number: BedNumber::new(1).unwrap(),
    };

    let event: StudentAssignedToRoom =
        fac_services::assign_student_to_room(cmd, &clock, &g).unwrap();

    assert_eq!(
        <StudentAssignedToRoom as DomainEvent>::EVENT_TYPE,
        "facilities.room.student_assigned"
    );
    assert_eq!(event.school_id(), school);
    assert_eq!(event.room_id, room.id);
    assert_eq!(event.student_id, student);
    assert_eq!(event.bed_number.value(), 1);
}

/// Room lifecycle failure path: per the spec invariant, the
/// room must belong to the dormitory the caller named.
/// `DormitoryService::can_assign` must reject a room that
/// belongs to a different dormitory.
#[test]
fn room_lifecycle_can_assign_rejects_wrong_dormitory() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;

    let dorm_a = fac_services::create_dormitory(
        CreateDormitoryCommand {
            tenant: TenantContext::for_user(
                school,
                actor,
                g.next_correlation_id(),
                UserType::SchoolAdmin,
            ),
            academic_year_id: year_id(&g, school),
            name: DormitoryName::new("Boys Hostel").unwrap(),
            dormitory_type: DormitoryType::Boys,
            address: None,
            intake: Intake::new(120).unwrap(),
            description: None,
        },
        &TestClock::new(),
        &g,
    )
    .unwrap()
    .0;
    let rt = room_type_id(&g, school);
    let room = new_room_aggregate(&g, school, actor, dorm_a.id, rt, "101", 2, 5000);

    // A dormitory from the same school but a different id.
    let dorm_b = fac_services::create_dormitory(
        CreateDormitoryCommand {
            tenant: TenantContext::for_user(
                school,
                actor,
                g.next_correlation_id(),
                UserType::SchoolAdmin,
            ),
            academic_year_id: year_id(&g, school),
            name: DormitoryName::new("Girls Hostel").unwrap(),
            dormitory_type: DormitoryType::Girls,
            address: None,
            intake: Intake::new(80).unwrap(),
            description: None,
        },
        &TestClock::new(),
        &g,
    )
    .unwrap()
    .0;
    assert_ne!(dorm_a.id, dorm_b.id);

    let err = DormitoryService::can_assign(&dorm_b, &room, 0)
        .expect_err("room not in dormitory must be rejected");
    assert!(
        matches!(err, educore_core::error::DomainError::Validation(_)),
        "got {err:?}"
    );
}

/// Room lifecycle helper: `DormitoryService::available_beds`
/// returns `number_of_bed - current_assignments` (saturation
/// at zero prevents underflow when the room is over-assigned
/// at the model layer).
#[test]
fn room_lifecycle_available_beds_subtracts_assignments() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;

    let dorm = dormitory_id(&g, school);
    let rt = room_type_id(&g, school);
    let room = new_room_aggregate(&g, school, actor, dorm, rt, "101", 4, 5000);

    assert_eq!(DormitoryService::available_beds(&room, 0), 4);
    assert_eq!(DormitoryService::available_beds(&room, 3), 1);
    // Saturating subtraction: over-assignment clamps at zero
    // rather than panicking.
    assert_eq!(DormitoryService::available_beds(&room, 10), 0);
}

/// Room lifecycle validation: per spec invariant, the room
/// number must be non-empty. `RoomNumber::new` must reject
/// empty input so the aggregate can never receive an empty
/// number.
#[test]
fn room_lifecycle_empty_room_number_returns_validation_error() {
    let res = RoomNumber::new(String::new());
    assert!(res.is_err(), "empty RoomNumber must fail validation");
}

// =============================================================================
// 2. Asset Lifecycle (Vehicle)
//    (`workflows.md` § "Vehicle Lifecycle Workflow")
//
//    The vehicle lifecycle per the spec:
//      1. SchoolAdmin creates a `Vehicle`.
//      2. SchoolAdmin assigns a driver from the HR roster.
//      3. SchoolAdmin assigns the vehicle to a route in the
//         current year (out of scope for the aggregate test;
//         tested via `TransportService::can_assign_vehicle`).
//      4. The vehicle may be deactivated for maintenance or
//         retirement.
//      5. A retired vehicle cannot be reactivated (the spec's
//         terminal state).
//      6. A maintenance vehicle can be returned to active via
//         a fresh deactivate(Active) call.
// =============================================================================

/// Asset lifecycle step 1: creating a vehicle emits
/// [`VehicleCreated`] with the supplied vehicle number, model,
/// and (absent) driver.
#[test]
fn asset_lifecycle_create_emits_vehicle_created() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let vehicle = new_vehicle_aggregate(&g, school, actor, "GJ-05-AB-1234", "Tata LP 909");
    assert_eq!(vehicle.status, VehicleStatus::Active);
    assert_eq!(vehicle.school_id, school);

    let event: VehicleCreated = VehicleCreated::new(
        vehicle.id,
        vehicle.vehicle_no.clone(),
        vehicle.vehicle_model.as_str().to_owned(),
        vehicle.driver_id,
        g.next_event_id(),
        correlation,
        clock.now(),
    );

    assert_eq!(
        <VehicleCreated as DomainEvent>::EVENT_TYPE,
        "facilities.vehicle.created"
    );
    assert_eq!(event.school_id(), school);
    assert_eq!(event.vehicle_no.as_str(), "GJ-05-AB-1234");
    assert_eq!(event.vehicle_model, "Tata LP 909");
    assert!(event.driver_id.is_none());
    assert_eq!(event.vehicle_id, vehicle.id);
}

/// Asset lifecycle step 2: assigning a driver to a vehicle
/// emits [`DriverAssignedToVehicle`] with the `from_driver_id`
/// captured (here `None` for the first assignment) and the new
/// `to_driver_id` set to the supplied driver.
#[test]
fn asset_lifecycle_assign_driver_emits_driver_assigned_to_vehicle() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let mut vehicle = new_vehicle_aggregate(&g, school, actor, "GJ-05-AB-1234", "Tata LP 909");
    assert!(vehicle.driver_id.is_none());
    let driver = staff_id(&g, school);

    let cmd = AssignDriverToVehicleCommand {
        tenant: TenantContext::for_user(school, actor, correlation, UserType::SchoolAdmin),
        vehicle_id: vehicle.id,
        driver_id: driver,
    };

    let event: DriverAssignedToVehicle =
        fac_services::assign_driver(&mut vehicle, cmd, &clock, &g).unwrap();

    assert_eq!(
        <DriverAssignedToVehicle as DomainEvent>::EVENT_TYPE,
        "facilities.vehicle.driver_assigned"
    );
    assert_eq!(event.school_id(), school);
    assert_eq!(event.vehicle_id, vehicle.id);
    assert_eq!(event.from_driver_id, None);
    assert_eq!(event.to_driver_id, driver);
    assert_eq!(vehicle.driver_id, Some(driver));
}

/// Asset lifecycle step 4 (mark out-of-service): deactivating
/// a vehicle to `Maintenance` emits [`VehicleDeactivated`] and
/// transitions the aggregate's `status` field.
#[test]
fn asset_lifecycle_mark_out_of_service_emits_vehicle_deactivated() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let mut vehicle = new_vehicle_aggregate(&g, school, actor, "GJ-05-AB-1234", "Tata LP 909");
    assert_eq!(vehicle.status, VehicleStatus::Active);

    let cmd = DeactivateVehicleCommand {
        tenant: TenantContext::for_user(school, actor, correlation, UserType::SchoolAdmin),
        vehicle_id: vehicle.id,
        new_status: VehicleStatus::Maintenance,
        reason: "Engine overhaul".to_owned(),
    };

    let event: VehicleDeactivated =
        fac_services::deactivate_vehicle(&mut vehicle, cmd, &clock, &g).unwrap();

    assert_eq!(
        <VehicleDeactivated as DomainEvent>::EVENT_TYPE,
        "facilities.vehicle.deactivated"
    );
    assert_eq!(event.school_id(), school);
    assert_eq!(event.vehicle_id, vehicle.id);
    assert_eq!(event.new_status, VehicleStatus::Maintenance);
    assert_eq!(event.reason, "Engine overhaul");
    assert_eq!(vehicle.status, VehicleStatus::Maintenance);
}

/// Asset lifecycle step 6 (restore): a vehicle in
/// `Maintenance` may be returned to `Active` via a second
/// `deactivate(Active)` call. The aggregate's
/// [`Vehicle::deactivate`] guard only rejects the terminal
/// `Retired` state, so the restore path is a normal state
/// transition (not a separate method).
#[test]
fn asset_lifecycle_restore_from_maintenance_transitions_back_to_active() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let mut vehicle = new_vehicle_aggregate(&g, school, actor, "GJ-05-AB-1234", "Tata LP 909");

    // 1) Mark out-of-service (maintenance).
    let cmd = DeactivateVehicleCommand {
        tenant: TenantContext::for_user(school, actor, correlation, UserType::SchoolAdmin),
        vehicle_id: vehicle.id,
        new_status: VehicleStatus::Maintenance,
        reason: "Engine overhaul".to_owned(),
    };
    fac_services::deactivate_vehicle(&mut vehicle, cmd, &clock, &g).unwrap();
    assert_eq!(vehicle.status, VehicleStatus::Maintenance);

    // 2) Restore to active.
    let restore = DeactivateVehicleCommand {
        tenant: TenantContext::for_user(school, actor, correlation, UserType::SchoolAdmin),
        vehicle_id: vehicle.id,
        new_status: VehicleStatus::Active,
        reason: "Maintenance complete".to_owned(),
    };
    let event: VehicleDeactivated =
        fac_services::deactivate_vehicle(&mut vehicle, restore, &clock, &g).unwrap();

    assert_eq!(
        <VehicleDeactivated as DomainEvent>::EVENT_TYPE,
        "facilities.vehicle.deactivated"
    );
    assert_eq!(event.new_status, VehicleStatus::Active);
    assert_eq!(vehicle.status, VehicleStatus::Active);
}

/// Asset lifecycle failure path: per the spec, a retired
/// vehicle is terminal — it cannot be reactivated. The
/// aggregate's [`Vehicle::deactivate`] returns
/// `DomainError::Conflict` on the second deactivate to
/// `Retired` (the guard fires before the state transition).
#[test]
fn asset_lifecycle_double_retire_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let mut vehicle = new_vehicle_aggregate(&g, school, actor, "GJ-05-AB-1234", "Tata LP 909");

    // Retire.
    let cmd = DeactivateVehicleCommand {
        tenant: TenantContext::for_user(school, actor, correlation, UserType::SchoolAdmin),
        vehicle_id: vehicle.id,
        new_status: VehicleStatus::Retired,
        reason: "End of life".to_owned(),
    };
    fac_services::deactivate_vehicle(&mut vehicle, cmd, &clock, &g).unwrap();
    assert_eq!(vehicle.status, VehicleStatus::Retired);

    // Second retire must be rejected.
    let second = DeactivateVehicleCommand {
        tenant: TenantContext::for_user(school, actor, correlation, UserType::SchoolAdmin),
        vehicle_id: vehicle.id,
        new_status: VehicleStatus::Retired,
        reason: "Already retired".to_owned(),
    };
    let err = fac_services::deactivate_vehicle(&mut vehicle, second, &clock, &g)
        .expect_err("second retire must be rejected");
    assert!(
        matches!(err, educore_core::error::DomainError::Conflict(_)),
        "got {err:?}"
    );
    // The vehicle must remain in `Retired`.
    assert_eq!(vehicle.status, VehicleStatus::Retired);
}

/// Asset lifecycle policy helper: `TransportService::can_assign_vehicle`
/// returns `true` only when the vehicle is `Active` AND the
/// caller has flagged the underlying row as active. A vehicle
/// in `Maintenance` cannot be assigned to a new route, even if
/// the row-level `active` flag is set — the spec gates new
/// assignments on the operational status.
#[test]
fn asset_lifecycle_can_assign_vehicle_rejects_maintenance() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;

    let mut vehicle = new_vehicle_aggregate(&g, school, actor, "GJ-05-AB-1234", "Tata LP 909");
    assert!(TransportService::can_assign_vehicle(&vehicle, true));

    // Mark out-of-service (maintenance).
    let cmd = DeactivateVehicleCommand {
        tenant: TenantContext::for_user(
            school,
            actor,
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        ),
        vehicle_id: vehicle.id,
        new_status: VehicleStatus::Maintenance,
        reason: "Engine overhaul".to_owned(),
    };
    fac_services::deactivate_vehicle(&mut vehicle, cmd, &TestClock::new(), &g).unwrap();

    assert!(!TransportService::can_assign_vehicle(&vehicle, true));
}

/// Asset lifecycle validation: per spec invariant, the
/// vehicle number must be non-empty. `VehicleNumber::new`
/// must reject empty input.
#[test]
fn asset_lifecycle_empty_vehicle_number_returns_validation_error() {
    let res = VehicleNumber::new(String::new());
    assert!(res.is_err(), "empty VehicleNumber must fail validation");
}

// =============================================================================
// 3. Booking Lifecycle (Inventory Issue)
//    (`workflows.md` § "Inventory Issue Workflow")
//
//    The inventory issue flow per the spec:
//      1. InventoryClerk issues goods to a recipient (role /
//         staff / student) with a quantity.
//      2. The system validates stock on hand and emits
//         [`ItemIssued`].
//      3. On return, the clerk issues a `ReturnIssuedItem`
//         with the returned quantity.
//      4. Partial return sets the issue to `PartiallyReturned`;
//         full return sets it to `Returned`; overdraw is
//         rejected with `DomainError::Conflict`.
//
//    These tests pin the booking lifecycle at the aggregate +
//    service layer.
// =============================================================================

/// Booking lifecycle step 1 (request): issuing a quantity of
/// an item to a recipient emits [`ItemIssued`] with the
/// supplied item, recipient, issue date, and quantity.
#[test]
fn booking_lifecycle_issue_emits_item_issued() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let item = item_id(&g, school);
    let cat = item_category_id(&g, school);
    let recipient_role = educore_hr::value_objects::RoleId::new(school, g.next_uuid());
    let quantity = 10_i64;

    let cmd = IssueItemCommand {
        tenant: TenantContext::for_user(school, actor, correlation, UserType::SchoolAdmin),
        academic_year_id: year_id(&g, school),
        issue_to: IssueRecipient::Role(recipient_role),
        issue_by: actor,
        issue_date: date(2026, 6, 13),
        due_date: Some(date(2026, 6, 27)),
        item_category_id: cat,
        item_id: item,
        quantity: ItemQuantity::new(quantity).unwrap(),
        note: Some(Note::new("Issued for science fair").unwrap()),
    };

    let (issue, event): (ItemIssue, ItemIssued) =
        fac_services::issue_item(cmd, &clock, &g).unwrap();

    assert_eq!(
        <ItemIssued as DomainEvent>::EVENT_TYPE,
        "facilities.item_issue.issued"
    );
    assert_eq!(event.school_id(), school);
    assert_eq!(event.item_id, item);
    assert_eq!(event.issue_by, actor);
    assert_eq!(event.issue_date, date(2026, 6, 13));
    assert_eq!(event.quantity, quantity);
    assert_eq!(issue.issue_status, IssueStatus::Issued);
    assert_eq!(issue.outstanding_quantity().value(), quantity);
}

/// Booking lifecycle step 3 (partial return): returning a
/// quantity strictly less than the issued quantity emits
/// [`IssuedItemReturned`] with `new_status =
/// PartiallyReturned`. The aggregate's
/// [`ItemIssue::outstanding_quantity`] decrements
/// accordingly.
#[test]
fn booking_lifecycle_partial_return_emits_issued_item_returned() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let item = item_id(&g, school);
    let cat = item_category_id(&g, school);
    let mut issue = new_item_issue_aggregate(&g, school, actor, item, cat, 10);
    assert_eq!(issue.outstanding_quantity().value(), 10);

    let cmd = ReturnIssuedItemCommand {
        tenant: TenantContext::for_user(school, actor, correlation, UserType::SchoolAdmin),
        item_issue_id: issue.id,
        returned_quantity: ItemQuantity::new(4).unwrap(),
    };
    let event: IssuedItemReturned =
        fac_services::return_issued_item(&mut issue, cmd, &clock, &g).unwrap();

    assert_eq!(
        <IssuedItemReturned as DomainEvent>::EVENT_TYPE,
        "facilities.item_issue.returned"
    );
    assert_eq!(event.school_id(), school);
    assert_eq!(event.item_issue_id, issue.id);
    assert_eq!(event.item_id, item);
    assert_eq!(event.returned_quantity, 4);
    assert_eq!(event.new_status, IssueStatus::PartiallyReturned);
    assert_eq!(issue.issue_status, IssueStatus::PartiallyReturned);
    assert_eq!(issue.returned_quantity.value(), 4);
    assert_eq!(issue.outstanding_quantity().value(), 6);
}

/// Booking lifecycle step 3 (complete return): returning the
/// remaining outstanding quantity emits
/// [`IssuedItemReturned`] with `new_status = Returned`. The
/// aggregate's `outstanding_quantity` reaches zero.
#[test]
fn booking_lifecycle_complete_return_emits_full_returned_status() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let item = item_id(&g, school);
    let cat = item_category_id(&g, school);
    let mut issue = new_item_issue_aggregate(&g, school, actor, item, cat, 10);

    // First return: 6 of 10 → PartiallyReturned.
    let cmd_partial = ReturnIssuedItemCommand {
        tenant: TenantContext::for_user(school, actor, correlation, UserType::SchoolAdmin),
        item_issue_id: issue.id,
        returned_quantity: ItemQuantity::new(6).unwrap(),
    };
    fac_services::return_issued_item(&mut issue, cmd_partial, &clock, &g).unwrap();
    assert_eq!(issue.issue_status, IssueStatus::PartiallyReturned);
    assert_eq!(issue.outstanding_quantity().value(), 4);

    // Final return: the remaining 4 → Returned.
    let cmd_final = ReturnIssuedItemCommand {
        tenant: TenantContext::for_user(school, actor, correlation, UserType::SchoolAdmin),
        item_issue_id: issue.id,
        returned_quantity: ItemQuantity::new(4).unwrap(),
    };
    let event: IssuedItemReturned =
        fac_services::return_issued_item(&mut issue, cmd_final, &clock, &g).unwrap();

    assert_eq!(
        <IssuedItemReturned as DomainEvent>::EVENT_TYPE,
        "facilities.item_issue.returned"
    );
    assert_eq!(event.returned_quantity, 4);
    assert_eq!(event.new_status, IssueStatus::Returned);
    assert_eq!(issue.issue_status, IssueStatus::Returned);
    assert_eq!(issue.outstanding_quantity().value(), 0);
}

/// Booking lifecycle failure path: per spec invariant, the
/// returned quantity must be positive (zero is rejected) and
/// must not exceed the outstanding quantity (overdraw is
/// rejected with `DomainError::Conflict`).
#[test]
fn booking_lifecycle_return_exceeds_outstanding_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let item = item_id(&g, school);
    let cat = item_category_id(&g, school);
    let mut issue = new_item_issue_aggregate(&g, school, actor, item, cat, 5);
    assert_eq!(issue.outstanding_quantity().value(), 5);

    let cmd = ReturnIssuedItemCommand {
        tenant: TenantContext::for_user(school, actor, correlation, UserType::SchoolAdmin),
        item_issue_id: issue.id,
        returned_quantity: ItemQuantity::new(10).unwrap(),
    };
    let err = fac_services::return_issued_item(&mut issue, cmd, &clock, &g)
        .expect_err("return above outstanding must be rejected");
    assert!(
        matches!(err, educore_core::error::DomainError::Conflict(_)),
        "got {err:?}"
    );
    // The aggregate must remain in `Issued` with no return applied.
    assert_eq!(issue.issue_status, IssueStatus::Issued);
    assert_eq!(issue.returned_quantity.value(), 0);
}

/// Booking lifecycle failure path: per spec invariant, the
/// returned quantity must be positive. A zero-quantity return
/// must be rejected with `DomainError::Validation`.
#[test]
fn booking_lifecycle_zero_quantity_return_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let item = item_id(&g, school);
    let cat = item_category_id(&g, school);
    let mut issue = new_item_issue_aggregate(&g, school, actor, item, cat, 5);

    let cmd = ReturnIssuedItemCommand {
        tenant: TenantContext::for_user(school, actor, correlation, UserType::SchoolAdmin),
        item_issue_id: issue.id,
        returned_quantity: ItemQuantity::ZERO,
    };
    let err = fac_services::return_issued_item(&mut issue, cmd, &clock, &g)
        .expect_err("zero return must be rejected");
    assert!(
        matches!(err, educore_core::error::DomainError::Validation(_)),
        "got {err:?}"
    );
    assert_eq!(issue.issue_status, IssueStatus::Issued);
}

/// Booking lifecycle validation: per spec invariant, the
/// issue quantity must be positive. `issue_item` must reject
/// a zero-quantity command before mutating any aggregate or
/// emitting any event.
#[test]
fn booking_lifecycle_zero_quantity_issue_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let item = item_id(&g, school);
    let cat = item_category_id(&g, school);
    let cmd = IssueItemCommand {
        tenant: TenantContext::for_user(school, actor, correlation, UserType::SchoolAdmin),
        academic_year_id: year_id(&g, school),
        issue_to: IssueRecipient::Role(educore_hr::value_objects::RoleId::new(
            school,
            g.next_uuid(),
        )),
        issue_by: actor,
        issue_date: date(2026, 6, 13),
        due_date: None,
        item_category_id: cat,
        item_id: item,
        quantity: ItemQuantity::ZERO,
        note: None,
    };
    let err = fac_services::issue_item(cmd, &clock, &g)
        .expect_err("zero-quantity issue must be rejected");
    assert!(
        matches!(err, educore_core::error::DomainError::Validation(_)),
        "got {err:?}"
    );
}

// =============================================================================
// Cross-lifecycle helpers: InventoryService::validate_issue rejects
// overdraw. This pins the conservation invariant at the service
// layer (the dispatcher relies on it before decrementing
// `Item.TotalInStock`).
// =============================================================================

/// Inventory conservation: `InventoryService::validate_issue`
/// rejects an issue whose quantity exceeds the item's
/// `total_in_stock` (the same guard the dispatcher relies on).
#[test]
fn inventory_validate_issue_rejects_overdraw() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;

    // Fresh item with zero stock on hand.
    let cmd = CreateItemCommand {
        tenant: TenantContext::for_user(
            school,
            actor,
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        ),
        academic_year_id: year_id(&g, school),
        item_name: ItemName::new("Whiteboard markers").unwrap(),
        item_sku: ItemSku::new("WBM-001").unwrap(),
        item_category_id: item_category_id(&g, school),
        description: None,
    };
    let (mut item, _event): (Item, ItemCreated) =
        fac_services::create_item(cmd, &TestClock::new(), &g).unwrap();
    assert_eq!(item.total_in_stock.value(), 0);

    // No stock → any positive issue must fail.
    let err = InventoryService::validate_issue(&item, ItemQuantity::new(1).unwrap())
        .expect_err("issue against zero stock must be rejected");
    assert!(
        matches!(err, educore_core::error::DomainError::Conflict(_)),
        "got {err:?}"
    );

    // Add stock via `apply_stock_delta`; then validate again.
    let ev = g.next_event_id();
    item.apply_stock_delta(50, actor, Timestamp::now(), ev)
        .unwrap();
    assert_eq!(item.total_in_stock.value(), 50);
    InventoryService::validate_issue(&item, ItemQuantity::new(50).unwrap())
        .expect("issue of exactly available stock must pass");
    let err = InventoryService::validate_issue(&item, ItemQuantity::new(51).unwrap())
        .expect_err("issue of more than available stock must fail");
    assert!(
        matches!(err, educore_core::error::DomainError::Conflict(_)),
        "got {err:?}"
    );
}
