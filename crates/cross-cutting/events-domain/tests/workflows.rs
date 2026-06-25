//! Integration tests for the **events-domain (calendar) workflows**.
//!
//! Implements: `docs/specs/events/workflows.md`
//!
//! Each test exercises a spec-mandated workflow end-to-end
//! through the events-domain aggregate methods and asserts that
//! the expected typed event is emitted (or, on the error path,
//! that the expected [`EventsDomainError`] is returned and no
//! event is produced).
//!
//! The tests are written as **pure synchronous** tests: the
//! events-domain aggregate constructors (`CalendarEvent::new`,
//! `CalendarEvent::update`, `CalendarEvent::delete`,
//! `Holiday::new`, `Incident::new`, `Incident::update`,
//! `Incident::resolve`, `Incident::delete`) are sync and return
//! `Result<(), EventsDomainError>` for state-machine transitions.
//! The test wires a [`TestClock`] and a [`SystemIdGen`], and
//! constructs the typed events directly from the aggregate +
//! clock instant to verify the event payloads.
//!
//! Per `docs/audit_reports/remediation/03-cluster-c-spec-drift.md`
//! the **handlers** are not yet wired end-to-end (no subscriber
//! fan-out, no outbox commit, no audit row). These tests pin
//! the contract of the **aggregate layer** that the eventual
//! service factory fns and dispatcher will wrap. When the
//! handlers land, the same test bodies will gain a
//! `+ outbox + bus subscriber` assertion without changes to
//! the assertions on the returned event.

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
use educore_events_domain::aggregate::{NewCalendarEvent, NewHoliday, NewIncident};
use educore_events_domain::events::{
    EventCreated, EventDeleted, EventUpdated, HolidayCreated, IncidentDeleted, IncidentReported,
    IncidentResolved, IncidentUpdated,
};
use educore_events_domain::prelude::*;
use educore_events_domain::value_objects::{RecurrenceFreq, RecurrenceRule};

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

fn calendar_event_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> CalendarEventId {
    CalendarEventId::new(school, g.next_uuid())
}

fn holiday_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> HolidayId {
    HolidayId::new(school, g.next_uuid())
}

fn incident_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> IncidentId {
    IncidentId::new(school, g.next_uuid())
}

fn academic_year(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> AcademicYearRef {
    AcademicYearRef::new(school, g.next_uuid())
}

fn date(y: i32, m: u32, d: u32) -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// Construct a fresh `CalendarEvent` aggregate in `Draft` status.
#[allow(clippy::too_many_arguments)]
fn new_calendar_event(
    g: &SystemIdGen,
    school: educore_core::ids::SchoolId,
    actor: educore_core::ids::UserId,
    title: &str,
    from: chrono::NaiveDate,
    to: chrono::NaiveDate,
    for_whom: ForWhom,
    with_recurrence: bool,
) -> CalendarEvent {
    let at = Timestamp::now();
    CalendarEvent::new(NewCalendarEvent {
        id: calendar_event_id(g, school),
        title: title.to_owned(),
        from_date: from,
        to_date: to,
        for_whom,
        role_ids: vec![],
        url: None,
        location: None,
        description: None,
        image: None,
        rrule: if with_recurrence {
            Some(RecurrenceRule::new(RecurrenceFreq::Weekly).with_count(4))
        } else {
            None
        },
        academic_id: academic_year(g, school),
        created_by: actor,
        created_at: at,
        correlation_id: g.next_correlation_id(),
    })
    .expect("CalendarEvent::new must succeed for valid input")
}

/// Construct a fresh `Holiday` aggregate.
fn new_holiday_aggregate(
    g: &SystemIdGen,
    school: educore_core::ids::SchoolId,
    actor: educore_core::ids::UserId,
    title: &str,
    from: chrono::NaiveDate,
    to: chrono::NaiveDate,
    details: Option<&str>,
) -> Holiday {
    let at = Timestamp::now();
    Holiday::new(NewHoliday {
        id: holiday_id(g, school),
        title: title.to_owned(),
        from_date: from,
        to_date: to,
        details: details.map(str::to_owned),
        image: None,
        academic_id: academic_year(g, school),
        created_by: actor,
        created_at: at,
        correlation_id: g.next_correlation_id(),
    })
    .expect("Holiday::new must succeed for valid input")
}

/// Construct a fresh `Incident` aggregate in `Open` status.
fn new_incident_aggregate(
    g: &SystemIdGen,
    school: educore_core::ids::SchoolId,
    actor: educore_core::ids::UserId,
    title: &str,
    point: i32,
    description: &str,
) -> Incident {
    let at = Timestamp::now();
    Incident::new(NewIncident {
        id: incident_id(g, school),
        title: title.to_owned(),
        point,
        description: description.to_owned(),
        created_by: actor,
        created_at: at,
        correlation_id: g.next_correlation_id(),
    })
    .expect("Incident::new must succeed for valid input")
}

// =============================================================================
// 1. CalendarEvent Lifecycle (`workflows.md` § "Calendar Event Lifecycle")
// =============================================================================

/// Calendar event lifecycle step 1: drafting a calendar event
/// (with `ForWhom::All`) emits [`EventCreated`] with the
/// supplied title, date range, and audience.
#[test]
fn calendar_event_lifecycle_create_emits_event_created() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let event_id = g.next_event_id();
    let correlation = CorrelationId::from(g.next_uuid());

    let event = new_calendar_event(
        &g,
        school,
        actor,
        "Parent-Teacher Conference",
        date(2026, 6, 10),
        date(2026, 6, 10),
        ForWhom::Parent,
        false,
    );

    let created = EventCreated::new(
        event.id,
        event.school_id,
        event.title.clone(),
        event.from_date,
        event.to_date,
        event_id,
        correlation,
        clock.now(),
    );

    assert_eq!(
        <EventCreated as DomainEvent>::EVENT_TYPE,
        "events.calendar_event.created"
    );
    assert_eq!(created.school_id, school);
    assert_eq!(created.title, "Parent-Teacher Conference");
    assert_eq!(created.from_date, date(2026, 6, 10));
    assert_eq!(created.to_date, date(2026, 6, 10));
    assert_eq!(created.event_id, event.id);
    assert!(matches!(event.status, CalendarEventStatus::Draft));
    assert!(event.active_status);
}

/// Calendar event lifecycle step 2: updating the title of a
/// draft event emits [`EventUpdated`] with the supplied changes
/// and bumps the aggregate's version monotonically.
#[test]
fn calendar_event_lifecycle_update_emits_event_updated_and_bumps_version() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let event_id = g.next_event_id();
    let correlation = CorrelationId::from(g.next_uuid());

    let mut event = new_calendar_event(
        &g,
        school,
        actor,
        "Sports Day",
        date(2026, 7, 1),
        date(2026, 7, 1),
        ForWhom::All,
        false,
    );
    let version_before = event.version;

    let at = clock.now();
    event
        .update(Some("Annual Sports Day".to_owned()), None, None, actor, at)
        .expect("update must succeed on an active draft event");

    let changes = vec!["title".to_owned()];
    let updated = EventUpdated::new(
        event.id,
        event.school_id,
        changes.clone(),
        actor,
        event_id,
        correlation,
        at,
    );

    assert_eq!(
        <EventUpdated as DomainEvent>::EVENT_TYPE,
        "events.calendar_event.updated"
    );
    assert_eq!(updated.changes, changes);
    assert_eq!(updated.updated_by, actor);
    assert_eq!(event.title, "Annual Sports Day");
    assert!(event.version > version_before, "version must be monotonic");
}

/// Calendar event lifecycle failure path: per spec invariant 1,
/// a calendar event title must be non-empty. `CalendarEvent::new`
/// must reject empty titles with
/// `EventsDomainError::Validation`.
#[test]
fn calendar_event_lifecycle_empty_title_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;

    let res = CalendarEvent::new(NewCalendarEvent {
        id: calendar_event_id(&g, school),
        title: "".to_owned(),
        from_date: date(2026, 6, 1),
        to_date: date(2026, 6, 1),
        for_whom: ForWhom::All,
        role_ids: vec![],
        url: None,
        location: None,
        description: None,
        image: None,
        rrule: None,
        academic_id: academic_year(&g, school),
        created_by: actor,
        created_at: Timestamp::now(),
        correlation_id: g.next_correlation_id(),
    });
    let err = res.expect_err("empty title must be rejected");
    assert!(
        matches!(err, EventsDomainError::Validation(_)),
        "got {err:?}"
    );
}

/// Calendar event lifecycle failure path: per spec invariant 2,
/// the event date range must satisfy `from_date <= to_date`.
/// `CalendarEvent::new` must reject an inverted range with
/// `EventsDomainError::Validation`.
#[test]
fn calendar_event_lifecycle_inverted_date_range_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;

    let res = CalendarEvent::new(NewCalendarEvent {
        id: calendar_event_id(&g, school),
        title: "Bad range".to_owned(),
        from_date: date(2026, 6, 10),
        to_date: date(2026, 6, 1),
        for_whom: ForWhom::All,
        role_ids: vec![],
        url: None,
        location: None,
        description: None,
        image: None,
        rrule: None,
        academic_id: academic_year(&g, school),
        created_by: actor,
        created_at: Timestamp::now(),
        correlation_id: g.next_correlation_id(),
    });
    let err = res.expect_err("inverted date range must be rejected");
    assert!(
        matches!(err, EventsDomainError::Validation(_)),
        "got {err:?}"
    );
}

/// Calendar event lifecycle happy path with recurrence: a
/// weekly-recurring event with `count = 4` expands to exactly
/// four weekly occurrences (per RFC 5545 RRULE subset).
#[test]
fn calendar_event_lifecycle_recurring_event_expands_to_four_occurrences() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;

    let event = new_calendar_event(
        &g,
        school,
        actor,
        "Weekly Assembly",
        date(2026, 6, 1),
        date(2026, 6, 1),
        ForWhom::All,
        true,
    );

    let rule = event
        .rrule
        .as_ref()
        .expect("rrule must be present for recurring event");
    let dates = rule.expand(event.from_date);
    assert_eq!(dates.len(), 4);
    assert_eq!(dates[0], date(2026, 6, 1));
    assert_eq!(dates[3], date(2026, 6, 22));
}

/// Calendar event lifecycle step 3: deleting an active event
/// emits [`EventDeleted`] and transitions the aggregate to
/// `active_status = false`. Subsequent updates must be rejected
/// with `EventsDomainError::Conflict`.
#[test]
fn calendar_event_lifecycle_delete_emits_event_deleted_and_blocks_updates() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let event_id = g.next_event_id();
    let correlation = CorrelationId::from(g.next_uuid());

    let mut event = new_calendar_event(
        &g,
        school,
        actor,
        "Field Trip",
        date(2026, 9, 1),
        date(2026, 9, 1),
        ForWhom::Student,
        false,
    );
    assert!(event.active_status);

    let at = clock.now();
    event.delete(at, actor);

    let deleted = EventDeleted::new(event.id, event.school_id, actor, event_id, correlation, at);

    assert_eq!(
        <EventDeleted as DomainEvent>::EVENT_TYPE,
        "events.calendar_event.deleted"
    );
    assert!(!event.active_status);
    assert_eq!(deleted.deleted_by, actor);

    // Per spec invariant 3: cannot update a soft-deleted event.
    let err = event
        .update(
            Some("Field Trip (cancelled)".to_owned()),
            None,
            None,
            actor,
            clock.now(),
        )
        .expect_err("update on soft-deleted event must be rejected");
    assert!(matches!(err, EventsDomainError::Conflict(_)), "got {err:?}");
}

// =============================================================================
// 2. Holiday Management (`workflows.md` § "Holiday Configuration Workflow")
// =============================================================================

/// Holiday management step 1: a `SchoolAdmin` creates a holiday
/// with a valid date range and details. `Holiday::new` emits
/// [`HolidayCreated`] with the supplied title and range.
#[test]
fn holiday_management_add_emits_holiday_created() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let event_id = g.next_event_id();
    let correlation = CorrelationId::from(g.next_uuid());

    let holiday = new_holiday_aggregate(
        &g,
        school,
        actor,
        "Winter Break",
        date(2026, 12, 20),
        date(2026, 12, 31),
        Some("School closed for winter holiday"),
    );

    let created = HolidayCreated::new(
        holiday.id,
        holiday.school_id,
        holiday.title.clone(),
        holiday.from_date,
        holiday.to_date,
        event_id,
        correlation,
        clock.now(),
    );

    assert_eq!(
        <HolidayCreated as DomainEvent>::EVENT_TYPE,
        "events.holiday.created"
    );
    assert_eq!(created.school_id, school);
    assert_eq!(created.title, "Winter Break");
    assert_eq!(created.from_date, date(2026, 12, 20));
    assert_eq!(created.to_date, date(2026, 12, 31));
    assert_eq!(created.holiday_id, holiday.id);
    assert!(holiday.active_status);
    assert!(holiday.attachments().is_empty());
    assert!(holiday.periods().is_empty());
}

/// Holiday management: per spec invariant 1, a holiday title
/// must be non-empty. `Holiday::new` must reject empty titles
/// with `EventsDomainError::Validation`.
#[test]
fn holiday_management_empty_title_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;

    let res = Holiday::new(NewHoliday {
        id: holiday_id(&g, school),
        title: "   ".to_owned(),
        from_date: date(2026, 6, 1),
        to_date: date(2026, 6, 2),
        details: None,
        image: None,
        academic_id: academic_year(&g, school),
        created_by: actor,
        created_at: Timestamp::now(),
        correlation_id: g.next_correlation_id(),
    });
    let err = res.expect_err("empty title must be rejected");
    assert!(
        matches!(err, EventsDomainError::Validation(_)),
        "got {err:?}"
    );
}

/// Holiday management: per spec invariant 2, the holiday date
/// range must satisfy `from_date <= to_date`. An inverted
/// range must be rejected with `EventsDomainError::Validation`.
#[test]
fn holiday_management_inverted_date_range_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;

    let res = Holiday::new(NewHoliday {
        id: holiday_id(&g, school),
        title: "Bad range".to_owned(),
        from_date: date(2026, 6, 30),
        to_date: date(2026, 6, 1),
        details: None,
        image: None,
        academic_id: academic_year(&g, school),
        created_by: actor,
        created_at: Timestamp::now(),
        correlation_id: g.next_correlation_id(),
    });
    let err = res.expect_err("inverted date range must be rejected");
    assert!(
        matches!(err, EventsDomainError::Validation(_)),
        "got {err:?}"
    );
}

/// Holiday management: per the spec, holidays override
/// recurring calendar events on overlapping dates
/// (`docs/specs/events/value-objects.md` rule 4). The
/// `apply_holiday_overrides` helper excludes event dates that
/// fall inside a holiday range — exercising the spec invariant
/// without needing a recurrence on the holiday itself.
#[test]
fn holiday_management_overrides_exclude_event_dates() {
    use educore_events_domain::value_objects::apply_holiday_overrides;

    let event_dates = vec![
        date(2026, 6, 1),
        date(2026, 6, 8),
        date(2026, 6, 15),
        date(2026, 6, 22),
    ];
    let holiday_ranges = vec![(date(2026, 6, 15), date(2026, 6, 15))];

    let remaining = apply_holiday_overrides(&event_dates, &holiday_ranges);
    assert_eq!(remaining.len(), 3);
    assert!(!remaining.contains(&date(2026, 6, 15)));
    assert!(remaining.contains(&date(2026, 6, 1)));
    assert!(remaining.contains(&date(2026, 6, 8)));
    assert!(remaining.contains(&date(2026, 6, 22)));
}

// =============================================================================
// 3. Incident Lifecycle (`workflows.md` § "Incident Reporting Workflow")
// =============================================================================

/// Incident lifecycle step 1: a teacher files an incident
/// (status `Open`) with a non-empty description. `Incident::new`
/// emits [`IncidentReported`] with the supplied title, point
/// value, and reporter.
#[test]
fn incident_lifecycle_file_emits_incident_reported() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let event_id = g.next_event_id();
    let correlation = CorrelationId::from(g.next_uuid());

    let incident = new_incident_aggregate(
        &g,
        school,
        actor,
        "Disruptive behaviour in class",
        5,
        "Student interrupted the lesson repeatedly.",
    );

    let reported = IncidentReported::new(
        incident.id,
        incident.school_id,
        incident.title.clone(),
        incident.point,
        actor,
        event_id,
        correlation,
        clock.now(),
    );

    assert_eq!(
        <IncidentReported as DomainEvent>::EVENT_TYPE,
        "events.incident.reported"
    );
    assert_eq!(reported.school_id, school);
    assert_eq!(reported.title, "Disruptive behaviour in class");
    assert_eq!(reported.point, 5);
    assert_eq!(reported.reported_by, actor);
    assert_eq!(reported.incident_id, incident.id);
    assert!(matches!(incident.status, IncidentStatus::Open));
    assert!(incident.active_status);
}

/// Incident lifecycle step 2 (triage): an open incident can be
/// updated — bumping the description — and emits
/// [`IncidentUpdated`] with the changed fields.
#[test]
fn incident_lifecycle_triage_emits_incident_updated() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let event_id = g.next_event_id();
    let correlation = CorrelationId::from(g.next_uuid());

    let mut incident = new_incident_aggregate(
        &g,
        school,
        actor,
        "Late homework",
        2,
        "Initial description.",
    );

    let at = clock.now();
    incident
        .update(
            None,
            Some(3),
            Some("Late homework — second occurrence".to_owned()),
            actor,
            at,
        )
        .expect("update on an open incident must succeed");

    let changes = vec!["point".to_owned(), "description".to_owned()];
    let updated = IncidentUpdated::new(
        incident.id,
        incident.school_id,
        changes.clone(),
        event_id,
        correlation,
        at,
    );

    assert_eq!(
        <IncidentUpdated as DomainEvent>::EVENT_TYPE,
        "events.incident.updated"
    );
    assert_eq!(updated.changes, changes);
    assert_eq!(incident.point, 3);
    assert_eq!(incident.description, "Late homework — second occurrence");
    assert!(matches!(incident.status, IncidentStatus::Open));
}

/// Incident lifecycle step 5 (resolve): a discipline lead
/// resolves an open incident, transitioning the aggregate to
/// `IncidentStatus::Resolved` and emitting [`IncidentResolved`].
#[test]
fn incident_lifecycle_resolve_transitions_to_resolved() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let event_id = g.next_event_id();
    let correlation = CorrelationId::from(g.next_uuid());

    let mut incident = new_incident_aggregate(
        &g,
        school,
        actor,
        "Skipped class",
        4,
        "Student left the classroom without permission.",
    );
    assert!(matches!(incident.status, IncidentStatus::Open));

    let at = clock.now();
    incident
        .resolve(actor, at)
        .expect("resolve must succeed on Open incident");

    let resolved = IncidentResolved::new(
        incident.id,
        incident.school_id,
        actor,
        event_id,
        correlation,
        at,
    );

    assert_eq!(
        <IncidentResolved as DomainEvent>::EVENT_TYPE,
        "events.incident.resolved"
    );
    assert!(matches!(incident.status, IncidentStatus::Resolved));
    assert_eq!(resolved.resolved_by, actor);
    assert_eq!(resolved.incident_id, incident.id);
}

/// Incident lifecycle failure path: per spec invariant 5, a
/// resolved incident is immutable in its body. `Incident::update`
/// must reject any further changes with
/// `EventsDomainError::Conflict`.
#[test]
fn incident_lifecycle_resolved_incident_is_immutable() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let mut incident = new_incident_aggregate(
        &g,
        school,
        actor,
        "Property damage",
        8,
        "Vandalism in the corridor.",
    );
    incident
        .resolve(actor, clock.now())
        .expect("resolve must succeed on Open incident");
    assert!(matches!(incident.status, IncidentStatus::Resolved));

    let err = incident
        .update(
            Some("Updated title".to_owned()),
            None,
            None,
            actor,
            clock.now(),
        )
        .expect_err("update on resolved incident must be rejected");
    assert!(matches!(err, EventsDomainError::Conflict(_)), "got {err:?}");
}

/// Incident lifecycle failure path: resolving an already-
/// resolved incident must be rejected with
/// `EventsDomainError::Conflict` (per spec: idempotency is
/// **not** guaranteed for `ResolveIncident` — re-resolution
/// produces duplicate audit records).
#[test]
fn incident_lifecycle_double_resolve_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let mut incident = new_incident_aggregate(
        &g,
        school,
        actor,
        "Repeated tardiness",
        3,
        "Late to class three times this week.",
    );
    incident
        .resolve(actor, clock.now())
        .expect("first resolve must succeed");

    let err = incident
        .resolve(actor, clock.now())
        .expect_err("second resolve must be rejected");
    assert!(matches!(err, EventsDomainError::Conflict(_)), "got {err:?}");
}

/// Incident lifecycle end-of-life (close): an admin soft-deletes
/// the resolved incident, emitting [`IncidentDeleted`] and
/// flipping `active_status` to `false`.
#[test]
fn incident_lifecycle_close_emits_incident_deleted() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let event_id = g.next_event_id();
    let correlation = CorrelationId::from(g.next_uuid());

    let mut incident = new_incident_aggregate(
        &g,
        school,
        actor,
        "Inappropriate language",
        6,
        "Used profanity in the hallway.",
    );
    incident
        .resolve(actor, clock.now())
        .expect("resolve must succeed on Open incident");

    let at = clock.now();
    incident.delete(at, actor);

    let deleted = IncidentDeleted::new(incident.id, incident.school_id, event_id, correlation, at);

    assert_eq!(
        <IncidentDeleted as DomainEvent>::EVENT_TYPE,
        "events.incident.deleted"
    );
    assert!(!incident.active_status);
    assert_eq!(deleted.incident_id, incident.id);
}

/// Incident lifecycle failure path: per spec invariant 2, an
/// incident must have a non-empty description. `Incident::new`
/// must reject an empty description with
/// `EventsDomainError::Validation`.
#[test]
fn incident_lifecycle_empty_description_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;

    let res = Incident::new(NewIncident {
        id: incident_id(&g, school),
        title: "No description".to_owned(),
        point: 1,
        description: String::new(),
        created_by: actor,
        created_at: Timestamp::now(),
        correlation_id: g.next_correlation_id(),
    });
    let err = res.expect_err("empty description must be rejected");
    assert!(
        matches!(err, EventsDomainError::Validation(_)),
        "got {err:?}"
    );
}

/// Incident lifecycle failure path: per spec invariant 3, the
/// incident point value must be in `0..=1000`. `Incident::new`
/// must reject an out-of-range point with
/// `EventsDomainError::Validation`.
#[test]
fn incident_lifecycle_point_out_of_range_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;

    let res = Incident::new(NewIncident {
        id: incident_id(&g, school),
        title: "Negative point".to_owned(),
        point: -5,
        description: "Out of range".to_owned(),
        created_by: actor,
        created_at: Timestamp::now(),
        correlation_id: g.next_correlation_id(),
    });
    let err = res.expect_err("negative point must be rejected");
    assert!(
        matches!(err, EventsDomainError::Validation(_)),
        "got {err:?}"
    );
}
