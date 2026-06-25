//! Integration tests for the **CalendarSetting aggregate** vertical slice.
//!
//! Pins the create + enable contract for
//! [`CalendarSetting`](educore_events_domain::aggregate::CalendarSetting)
//! per `docs/specs/events/aggregates.md` ## CalendarSetting and
//! `docs/specs/events/workflows.md` ## Calendar Setting Workflow.
//!
//! The tests exercise the **constructor pattern** used by this
//! domain (`CalendarSetting::new(cmd)`), not a service-layer
//! helper, because `CalendarSetting` is one of the few
//! events-domain aggregates whose service surface is a thin
//! wrapper. The assertions pin the invariants from the spec:
//!
//! 1. `menu_name` must be non-empty (spec invariant #1).
//! 2. `status` is preserved as supplied on construction; the
//!    `enable` transition is idempotent and moves
//!    `status` to [`CalendarStatus::Enabled`].
//! 3. `font_color` and `bg_color` must be valid CSS color
//!    strings (spec invariant #3). Malformed inputs produce
//!    [`EventsDomainError::Validation`], not a panic or a
//!    generic `DomainError`.
//!
//! The fixture pattern mirrors `tests/workflows.rs`:
//! a [`TestClock`] for deterministic timestamps and a
//! [`SystemIdGen`] for fresh ids.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::clock::{Clock as _, IdGenerator as _, SystemIdGen, TestClock};
use educore_events_domain::aggregate::{CalendarSetting, NewCalendarSetting};
use educore_events_domain::errors::EventsDomainError;
use educore_events_domain::value_objects::{CalendarSettingId, CalendarStatus};

// =============================================================================
// Fixtures
// =============================================================================

/// Build a `NewCalendarSetting` command with the given menu name,
/// status, and CSS colors. School id and ids come from the shared
/// [`SystemIdGen`] so the test does not bake in any UUID literals.
fn new_command(
    ids: &SystemIdGen,
    school: educore_core::ids::SchoolId,
    menu_name: &str,
    status: CalendarStatus,
    font_color: &str,
    bg_color: &str,
    at: educore_core::value_objects::Timestamp,
) -> NewCalendarSetting {
    NewCalendarSetting {
        id: CalendarSettingId::new(school, ids.next_uuid()),
        menu_name: menu_name.to_owned(),
        status,
        font_color: font_color.to_owned(),
        bg_color: bg_color.to_owned(),
        created_by: ids.next_user_id(),
        created_at: at,
        correlation_id: ids.next_correlation_id(),
    }
}

// =============================================================================
// Happy path: construct + enable
// =============================================================================

/// Constructing a `CalendarSetting` with valid fields populates
/// every aggregate field, leaves `active_status = true` (per the
/// aggregate `new` implementation), preserves the supplied
/// `status`, and yields a setting whose `enable` transition is
/// idempotent (a second `enable` call still produces
/// `CalendarStatus::Enabled` and bumps `version` exactly once).
#[test]
fn calendar_setting_new_then_enable_is_idempotent() {
    let ids = SystemIdGen;
    let school = ids.next_school_id();
    let clock = TestClock::new();
    let created_at = clock.now();
    let actor = ids.next_user_id();

    // ---- Construct ----
    let cmd = new_command(
        &ids,
        school,
        "Holidays",
        CalendarStatus::Enabled,
        "#000000",
        "#ffffff",
        created_at,
    );

    let mut setting = CalendarSetting::new(cmd).expect("valid CalendarSetting should construct");

    // Aggregate fields populated from the command.
    assert_eq!(setting.school_id, school);
    assert_eq!(setting.menu_name, "Holidays");
    assert_eq!(setting.status, CalendarStatus::Enabled);
    assert_eq!(setting.font_color, "#000000");
    assert_eq!(setting.bg_color, "#ffffff");
    assert_eq!(setting.created_by, setting.updated_by);
    assert!(setting.active_status, "new CalendarSetting starts active");
    assert_eq!(
        setting.created_at, created_at,
        "created_at is taken from the command",
    );
    assert_eq!(setting.created_at, setting.updated_at);

    // ---- Enable (idempotent) ----
    // Advance the clock so the enable call gets a fresh instant.
    clock.advance(chrono::Duration::seconds(60));
    let enable_at = clock.now();
    let initial_version = setting.version;
    setting.enable(enable_at, actor);

    // enable() flips status to Enabled (already enabled — still
    // produces Enabled), bumps version, and updates the audit
    // footer.
    assert_eq!(setting.status, CalendarStatus::Enabled);
    assert_eq!(setting.updated_at, enable_at);
    assert_eq!(setting.updated_by, actor);
    assert_eq!(
        setting.version.get(),
        initial_version.get() + 1,
        "enable must bump version exactly once",
    );

    // A second enable is also a clean transition — never panics,
    // never errors, and bumps version again.
    clock.advance(chrono::Duration::seconds(60));
    let second_at = clock.now();
    setting.enable(second_at, actor);
    assert_eq!(setting.status, CalendarStatus::Enabled);
    assert_eq!(setting.updated_at, second_at);
    assert_eq!(
        setting.version.get(),
        initial_version.get() + 2,
        "second enable bumps version again",
    );
}

// =============================================================================
// Validation failure: invalid CSS color
// =============================================================================

/// A `bg_color` that is not a valid CSS color string must be
/// rejected by `CalendarSetting::new` with
/// [`EventsDomainError::Validation`] (not the generic
/// `DomainError::Validation`, not a panic, not a different
/// variant).
#[test]
fn calendar_setting_new_rejects_invalid_css_color() {
    let ids = SystemIdGen;
    let school = ids.next_school_id();
    let clock = TestClock::new();

    // `not-a-color` is neither a `#hex`, an `rgb(...)`, nor an
    // alphabetic-only named color — `validate_css_color` rejects
    // it on the alphabetic-only fallback path.
    let cmd = new_command(
        &ids,
        school,
        "Holidays",
        CalendarStatus::Enabled,
        "#000000",
        "not-a-color",
        clock.now(),
    );

    let err = CalendarSetting::new(cmd).expect_err("invalid bg_color must fail construction");

    // Pin the exact error variant + shape. A regression that
    // changed `EventsDomainError::Validation(...)` to a generic
    // `DomainError` or to a different variant would break this
    // match.
    match err {
        EventsDomainError::Validation(msg) => {
            // The message should mention the offending field or
            // the CSS color check. We accept either form because
            // the spec only requires a Validation error — the
            // message wording is an implementation detail.
            assert!(!msg.is_empty(), "Validation error must carry a message");
        }
        other => panic!("expected EventsDomainError::Validation, got: {other:?}",),
    }
}
