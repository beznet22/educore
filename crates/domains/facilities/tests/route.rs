//! Integration tests for the **Route aggregate** vertical slice.
//!
//! Pins the create contract for
//! [`Route`](educore_facilities::aggregate::Route)
//! end-to-end through the service layer:
//!
//! 1. `create_route` validates the input (the typed
//!    [`RouteName`](educore_facilities::value_objects::RouteName)
//!    enforces non-empty + length bounds; the typed
//!    [`Fare`](educore_facilities::value_objects::Fare)
//!    rejects negative values at construction), constructs
//!    the aggregate with its ordered stops, and emits a
//!    [`RouteCreated`](educore_facilities::events::RouteCreated)
//!    event.
//!
//! The tests use the same fixture pattern as
//! `tests/vehicle.rs` and `tests/workflows.rs` (`TestClock` +
//! `SystemIdGen`). The handlers / outbox / audit fan-out are
//! not yet wired end-to-end; these tests pin the **service
//! layer** contract that the dispatcher will eventually wrap.
//!
//! Mirrors `crates/domains/library/tests/aggregates.rs` and
//! `crates/domains/facilities/tests/vehicle.rs` (lean).
//!
//! Spec: `docs/specs/facilities/aggregates.md` ## Route +
//! `docs/specs/facilities/workflows.md` § Route Assignment
//! Workflow.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::clock::{SystemIdGen, TestClock};
use educore_core::error::DomainError;
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent;
use educore_facilities::prelude::*;
use educore_facilities::services::create_route;
use educore_facilities::value_objects::Distance;

// =============================================================================
// Fixtures
// =============================================================================

/// A fresh `TenantContext` for a `SchoolAdmin` acting on a
/// freshly-minted school. Returns the context plus the
/// generator so tests can mint child ids from the same
/// school.
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

/// Mint a typed `AcademicYearId` for the given school.
fn academic_year_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> AcademicYearId {
    AcademicYearId::new(school, g.next_uuid())
}

// =============================================================================
// Happy path: create a Route end-to-end
// =============================================================================

/// End-to-end happy path for the Route aggregate. Create a
/// route titled "Route 1" with a fare of 100 (minor units)
/// and two ordered stops, asserting that:
///
/// 1. The create flow produces a `Route` aggregate carrying
///    every field on the command (school id derived from the
///    typed id), with `version` initialised to 1 and the
///    audit footer (`created_at` / `updated_at`,
///    `created_by` / `updated_by`) populated by the clock
///    and tenant.
/// 2. The event is a `RouteCreated` carrying the correct
///    `EVENT_TYPE`, `AGGREGATE_TYPE`, `SCHEMA_VERSION`,
///    `aggregate_id`, `school_id`, `title`, and `fare_minor`,
///    plus the ordered `stops` list verbatim from the
///    command.
#[test]
fn route_create_emits_route_created_event_with_stops() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let stops = vec![
        RouteStopSpec {
            stop_order: 1,
            stop_name: StopName::new("Main Gate").expect("non-empty stop name"),
            pickup_time: None,
            fare_override: None,
        },
        RouteStopSpec {
            stop_order: 2,
            stop_name: StopName::new("City Center").expect("non-empty stop name"),
            pickup_time: None,
            fare_override: Some(Fare::new(120).expect("non-negative fare override")),
        },
    ];

    // ---- Create flow ----
    let create_cmd = CreateRouteCommand {
        tenant: tenant.clone(),
        academic_year_id: academic_year_id(&g, school),
        title: RouteName::new("Route 1").expect("non-empty route name"),
        fare: Fare::new(100).expect("non-negative fare"),
        distance: Some(Distance::new(12).expect("non-negative distance")),
        stops: stops.clone(),
        note: None,
    };
    let (route, created_event) = create_route(create_cmd, &clock, &ids).expect("create");

    // Aggregate fields are populated from the command.
    assert_eq!(route.school_id, school);
    assert_eq!(route.title.as_str(), "Route 1");
    assert_eq!(route.fare.value(), 100);
    assert_eq!(route.distance.map(Distance::value), Some(12));
    assert_eq!(route.stops.len(), 2);
    assert_eq!(route.stops[0].stop_name.as_str(), "Main Gate");
    assert_eq!(route.stops[1].stop_name.as_str(), "City Center");
    assert_eq!(route.created_by, tenant.actor_id);
    assert_eq!(route.updated_by, tenant.actor_id);
    // Audit metadata footer is initialised.
    assert_eq!(route.version.get(), 1);
    assert!(route.active_status.is_active());

    // Event metadata matches the aggregate's typed id and
    // the DomainEvent trait's contract.
    assert_eq!(
        <RouteCreated as DomainEvent>::EVENT_TYPE,
        "facilities.route.created"
    );
    assert_eq!(<RouteCreated as DomainEvent>::AGGREGATE_TYPE, "route");
    assert_eq!(<RouteCreated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(created_event.aggregate_id(), route.id.as_uuid());
    assert_eq!(created_event.school_id(), school);
    assert_eq!(created_event.title.as_str(), "Route 1");
    assert_eq!(created_event.fare_minor, 100);
    assert_eq!(created_event.stops.len(), 2);
    assert_eq!(created_event.stops[0].stop_name.as_str(), "Main Gate");
    assert_eq!(created_event.stops[1].stop_name.as_str(), "City Center");
}

// =============================================================================
// Validation failure: empty route_name is rejected
// =============================================================================

/// Validation-failure path on the create flow: when the
/// `title` is empty, the typed
/// [`RouteName::new`](educore_facilities::value_objects::RouteName::new)
/// constructor returns `DomainError::Validation` before the
/// service factory is ever invoked (and therefore no event
/// is minted).
#[test]
fn route_create_with_empty_title_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;

    // The typed `RouteName::new("")` is the validation gate;
    // it rejects empty input with `DomainError::Validation`.
    let err = RouteName::new("").expect_err("empty route name must fail validation");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );

    // A negative `Fare` is likewise rejected at the typed
    // id construction boundary — the service factory is
    // never invoked in this path either.
    let err = Fare::new(-1).expect_err("negative fare must fail validation");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );

    // The service factory is never called in either path: a
    // failed validation on the typed id means the command
    // cannot even be constructed, so no aggregate is built
    // and no event is emitted. Verify the school id we
    // minted is sane (i.e. the test setup worked) and the
    // `AcademicYearId` plumbing is in scope.
    assert_ne!(school, educore_core::ids::SchoolId(uuid::Uuid::nil()));
    let _ = academic_year_id(&g, school);
}
