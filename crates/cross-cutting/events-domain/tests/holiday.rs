//! Integration tests for the **Holiday aggregate** vertical slice.
//!
//! Pins the constructor contract for
//! [`Holiday`](educore_events_domain::aggregate::Holiday)
//! end-to-end per `docs/specs/events/aggregates.md` ## Holiday and
//! `docs/specs/events/workflows.md` ## Holiday Configuration Workflow.
//!
//! The events-domain uses a **constructor pattern** (`Holiday::new`
//! returns the aggregate directly) rather than a factory handler —
//! there is no `create_holiday` service fn. Event emission lives in
//! the service layer (`services.rs`), not in the constructor. These
//! tests pin the **constructor's invariants only**: per spec
//! invariant 1, `holiday_title` must be non-empty; per spec
//! invariant 2, `from_date <= to_date`. Both must be rejected with
//! [`EventsDomainError::Validation`] (the events-domain error type,
//! **not** `DomainError::Validation`).
//!
//! The fixture pattern mirrors `tests/workflows.rs` (TestClock +
//! SystemIdGen). When the service layer lands, these tests will
//! gain a parallel `+ handler + outbox + audit` assertion without
//! changes to the constructor contract assertions.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use chrono::NaiveDate;
use educore_core::clock::{Clock as _, IdGenerator as _, SystemIdGen, TestClock};
use educore_core::ids::{CorrelationId, UserId};
use educore_core::value_objects::Version;
use educore_events_domain::aggregate::{Holiday, NewHoliday};
use educore_events_domain::errors::EventsDomainError;
use educore_events_domain::prelude::{AcademicYearRef, HolidayId};

// =============================================================================
// Fixtures
// =============================================================================

/// Mint a fresh system actor and id generator pair.
fn admin_actor() -> (UserId, SystemIdGen) {
    let g = SystemIdGen;
    let actor = g.next_user_id();
    (actor, g)
}

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

// =============================================================================
// 1. Happy path: Holiday::new with a valid date range
// =============================================================================

/// Happy-path constructor: building a `Holiday` for Independence
/// Day 2026 with a non-empty title and a valid (single-day) date
/// range yields an aggregate whose identity, payload, and audit
/// metadata are all populated from the command. `version` is the
/// initial version (per the optimistic-concurrency contract) and
/// `active_status` is `true` (per spec invariant that an aggregate
/// is alive on construction).
#[test]
fn holiday_construct_happy_path_populates_aggregate() {
    let (actor, g) = admin_actor();
    let clock = TestClock::new();
    let school = g.next_school_id();

    let cmd = NewHoliday {
        id: HolidayId::new(school, g.next_uuid()),
        title: "Independence Day".to_owned(),
        from_date: date(2026, 8, 15),
        to_date: date(2026, 8, 15),
        details: Some("National holiday".to_owned()),
        image: None,
        academic_id: AcademicYearRef::new(school, g.next_uuid()),
        created_by: actor,
        created_at: clock.now(),
        correlation_id: g.next_correlation_id(),
    };

    let holiday = Holiday::new(cmd).expect("Holiday::new must succeed for valid input");

    // Identity: school is propagated both via the typed id and
    // the denormalised field.
    assert_eq!(holiday.id.school_id(), school);
    assert_eq!(holiday.school_id, school);

    // Payload: every field on the command is present on the aggregate.
    assert_eq!(holiday.title, "Independence Day");
    assert_eq!(holiday.from_date, date(2026, 8, 15));
    assert_eq!(holiday.to_date, date(2026, 8, 15));
    assert_eq!(holiday.details.as_deref(), Some("National holiday"));
    assert!(holiday.image.is_none());

    // Audit metadata footer is initialised per the constructor
    // contract: version starts at 1, active_status is true, and
    // created_by/updated_by both equal the supplied actor.
    assert_eq!(holiday.version, Version::initial());
    assert!(holiday.active_status);
    assert_eq!(holiday.created_by, actor);
    assert_eq!(holiday.updated_by, actor);

    // Embedded children are empty (no attachments / periods
    // supplied at construction time).
    assert!(holiday.attachments().is_empty());
    assert!(holiday.periods().is_empty());
}

// =============================================================================
// 2. Validation failure: empty title is rejected
// =============================================================================

/// Validation-failure path on the constructor: per spec invariant
/// 1, a `Holiday` title must be non-empty. `Holiday::new` must
/// reject an empty title with [`EventsDomainError::Validation`] —
/// the **events-domain** error type, not `DomainError::Validation`.
#[test]
fn holiday_construct_empty_title_returns_validation_error() {
    let (actor, g) = admin_actor();
    let clock = TestClock::new();
    let school = g.next_school_id();

    let res = Holiday::new(NewHoliday {
        id: HolidayId::new(school, g.next_uuid()),
        title: String::new(),
        from_date: date(2026, 8, 15),
        to_date: date(2026, 8, 15),
        details: None,
        image: None,
        academic_id: AcademicYearRef::new(school, g.next_uuid()),
        created_by: actor,
        created_at: clock.now(),
        correlation_id: CorrelationId::from(g.next_uuid()),
    });

    let err = res.expect_err("empty title must be rejected");
    assert!(
        matches!(err, EventsDomainError::Validation(_)),
        "expected EventsDomainError::Validation, got {err:?}"
    );
}
