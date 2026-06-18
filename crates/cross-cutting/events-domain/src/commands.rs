//! # educore-events-domain typed commands
//!
//! Per `docs/specs/events/commands.md`. Wire form:
//! `events.<aggregate>.<verb>` (e.g. `events.calendar_event.create`).
//!
//! Section markers are placed for each workstream; Wave 1
//! workstreams (A/B/C/D) fill in the 24 command structs in
//! their assigned range.

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

// === CalendarEvent commands section begin (owner: A) ===
// CreateEventCommand, UpdateEventCommand, DeleteEventCommand — 3 commands.
// === CalendarEvent commands section end ===

// === Holiday commands section begin (owner: B) ===
// CreateHolidayCommand, UpdateHolidayCommand, DeleteHolidayCommand — 3 commands.
// === Holiday commands section end ===

// === CalendarSetting commands section begin (owner: B) ===
// CreateCalendarSettingCommand, UpdateCalendarSettingCommand, EnableCalendarSettingCommand,
// DisableCalendarSettingCommand, DeleteCalendarSettingCommand — 5 commands.
// === CalendarSetting commands section end ===

// === Incident commands section begin (owner: C) ===
// CreateIncidentCommand, UpdateIncidentCommand, ResolveIncidentCommand, DeleteIncidentCommand — 4 commands.
// === Incident commands section end ===

// === AssignIncident commands section begin (owner: C) ===
// AssignIncidentCommand, ReassignIncidentCommand, UnassignIncidentCommand — 3 commands.
// === AssignIncident commands section end ===

// === IncidentComment commands section begin (owner: C) ===
// CommentOnIncidentCommand, DeleteIncidentCommentCommand — 2 commands.
// === IncidentComment commands section end ===

// === Weekend commands section begin (owner: D) ===
// CreateWeekendCommand, UpdateWeekendCommand, ConfigureWeekendsCommand, DeleteWeekendCommand — 4 commands.
// === Weekend commands section end ===
