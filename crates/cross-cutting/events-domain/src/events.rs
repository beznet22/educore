//! # educore-events-domain typed events
//!
//! Per `docs/specs/events/events.md`. Wire form:
//! `events.<aggregate>.<verb>` (e.g. `events.calendar_event.created`).
//!
//! Section markers are placed for each workstream; Wave 1
//! workstreams (A/B/C/D) fill in the 24 event structs in
//! their assigned range.

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

// === CalendarEvent events section begin (owner: A) ===
// EventCreated, EventUpdated, EventDeleted — 3 events.
// === CalendarEvent events section end ===

// === Holiday events section begin (owner: B) ===
// HolidayCreated, HolidayUpdated, HolidayDeleted — 3 events.
// === Holiday events section end ===

// === CalendarSetting events section begin (owner: B) ===
// CalendarSettingCreated, CalendarSettingUpdated, CalendarSettingEnabled,
// CalendarSettingDisabled, CalendarSettingDeleted — 5 events.
// === CalendarSetting events section end ===

// === Incident events section begin (owner: C) ===
// IncidentReported, IncidentUpdated, IncidentResolved, IncidentDeleted — 4 events.
// === Incident events section end ===

// === AssignIncident events section begin (owner: C) ===
// IncidentAssigned, IncidentReassigned, IncidentUnassigned — 3 events.
// === AssignIncident events section end ===

// === IncidentComment events section begin (owner: C) ===
// IncidentCommented, IncidentCommentDeleted — 2 events.
// === IncidentComment events section end ===

// === Weekend events section begin (owner: D) ===
// WeekendCreated, WeekendUpdated, WeekendsConfigured, WeekendDeleted — 4 events.
// === Weekend events section end ===
