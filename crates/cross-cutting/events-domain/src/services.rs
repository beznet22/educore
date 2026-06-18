//! # educore-events-domain service structs and factory fns
//!
//! Per `docs/specs/events/services.md`. The events domain
//! ships 5 service structs:
//!
//! - [`CalendarService`] — visible_to, audience_resolves_to, etc.
//! - [`RecurrenceService`] — RRULE subset expansion
//! - [`HolidayService`] — is_instructional, contains, overlaps
//! - [`CalendarSettingService`] — validate_color, visible
//! - [`IncidentService`] — next_status (state machine)
//! - [`WeekendService`] — reconcile, is_weekend, ordered
//!
//! Section markers are placed for each workstream; Wave 1
//! workstreams (A/B/C/D) fill in the impls.

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

// =============================================================================
// === CalendarService section begin (owner: A) ===
// =============================================================================

/// Pure helpers for the `CalendarEvent` aggregate.
pub struct CalendarService;

// === CalendarService section end ===

// =============================================================================
// === RecurrenceService section begin (owner: A) ===
// =============================================================================

/// RRULE subset (RFC 5545) expansion for `CalendarEvent` recurrence.
/// Supports FREQ (DAILY/WEEKLY/MONTHLY/YEARLY), INTERVAL, COUNT, UNTIL.
pub struct RecurrenceService;

// === RecurrenceService section end ===

// =============================================================================
// === HolidayService section begin (owner: B) ===
// =============================================================================

/// Pure helpers for the `Holiday` aggregate.
pub struct HolidayService;

// === HolidayService section end ===

// =============================================================================
// === CalendarSettingService section begin (owner: B) ===
// =============================================================================

/// Pure helpers for the `CalendarSetting` aggregate.
pub struct CalendarSettingService;

// === CalendarSettingService section end ===

// =============================================================================
// === IncidentService section begin (owner: C) ===
// =============================================================================

/// Pure helpers for the `Incident` aggregate.
/// `next_status` is the canonical state machine: Open → InProgress → Resolved.
pub struct IncidentService;

// === IncidentService section end ===

// =============================================================================
// === WeekendService section begin (owner: D) ===
// =============================================================================

/// Pure helpers for the `Weekend` aggregate.
/// `reconcile` is the canonical diff for `ConfigureWeekends`.
pub struct WeekendService;

// === WeekendService section end ===
