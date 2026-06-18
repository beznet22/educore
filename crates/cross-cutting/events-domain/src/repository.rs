//! # educore-events-domain repository port traits
//!
//! Per `docs/specs/events/repositories.md`. The events
//! domain ships 7 repository port traits (one per root
//! aggregate).
//!
//! Each trait is object-safe (the `_assert_object_safe`
//! helpers prove it).

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

use async_trait::async_trait;

use educore_core::error::Result as StorageResult;
use educore_core::ids::SchoolId;

use crate::aggregate::{
    AssignIncident, CalendarEvent, CalendarSetting, Holiday, Incident, IncidentComment, Weekend,
};
use crate::value_objects::{
    AssignIncidentId, CalendarEventId, CalendarSettingId, HolidayId, IncidentCommentId,
    IncidentId, WeekendId,
};

// =============================================================================
// === CalendarEventRepository section begin (owner: A) ===
// =============================================================================

/// Repository port for [`CalendarEvent`](crate::aggregate::CalendarEvent).
#[async_trait]
pub trait CalendarEventRepository: Send + Sync {
    // Methods filled in by Workstream A.
}

/// Object-safety smoke test.
fn _assert_calendar_event_object_safe() {
    fn _f(_: Box<dyn CalendarEventRepository>) {}
}

// === CalendarEventRepository section end ===

// =============================================================================
// === HolidayRepository section begin (owner: B) ===
// =============================================================================

/// Repository port for [`Holiday`](crate::aggregate::Holiday).
#[async_trait]
pub trait HolidayRepository: Send + Sync {
    // Methods filled in by Workstream B.
}

/// Object-safety smoke test.
fn _assert_holiday_object_safe() {
    fn _f(_: Box<dyn HolidayRepository>) {}
}

// === HolidayRepository section end ===

// =============================================================================
// === CalendarSettingRepository section begin (owner: B) ===
// =============================================================================

/// Repository port for [`CalendarSetting`](crate::aggregate::CalendarSetting).
#[async_trait]
pub trait CalendarSettingRepository: Send + Sync {
    // Methods filled in by Workstream B.
}

/// Object-safety smoke test.
fn _assert_calendar_setting_object_safe() {
    fn _f(_: Box<dyn CalendarSettingRepository>) {}
}

// === CalendarSettingRepository section end ===

// =============================================================================
// === IncidentRepository section begin (owner: C) ===
// =============================================================================

/// Repository port for [`Incident`](crate::aggregate::Incident).
#[async_trait]
pub trait IncidentRepository: Send + Sync {
    // Methods filled in by Workstream C.
}

/// Object-safety smoke test.
fn _assert_incident_object_safe() {
    fn _f(_: Box<dyn IncidentRepository>) {}
}

// === IncidentRepository section end ===

// =============================================================================
// === AssignIncidentRepository section begin (owner: C) ===
// =============================================================================

/// Repository port for [`AssignIncident`](crate::aggregate::AssignIncident).
#[async_trait]
pub trait AssignIncidentRepository: Send + Sync {
    // Methods filled in by Workstream C.
}

/// Object-safety smoke test.
fn _assert_assign_incident_object_safe() {
    fn _f(_: Box<dyn AssignIncidentRepository>) {}
}

// === AssignIncidentRepository section end ===

// =============================================================================
// === IncidentCommentRepository section begin (owner: C) ===
// =============================================================================

/// Repository port for [`IncidentComment`](crate::aggregate::IncidentComment).
#[async_trait]
pub trait IncidentCommentRepository: Send + Sync {
    // Methods filled in by Workstream C.
}

/// Object-safety smoke test.
fn _assert_incident_comment_object_safe() {
    fn _f(_: Box<dyn IncidentCommentRepository>) {}
}

// === IncidentCommentRepository section end ===

// =============================================================================
// === WeekendRepository section begin (owner: D) ===
// =============================================================================

/// Repository port for [`Weekend`](crate::aggregate::Weekend).
#[async_trait]
pub trait WeekendRepository: Send + Sync {
    // Methods filled in by Workstream D.
}

/// Object-safety smoke test.
fn _assert_weekend_object_safe() {
    fn _f(_: Box<dyn WeekendRepository>) {}
}

// === WeekendRepository section end ===

#[allow(dead_code)]
fn _ensure_ids_compile() {
    let _ = AssignIncidentId::new;
    let _ = CalendarEventId::new;
    let _ = CalendarSettingId::new;
    let _ = HolidayId::new;
    let _ = IncidentCommentId::new;
    let _ = IncidentId::new;
    let _ = WeekendId::new;
    let _: Option<SchoolId> = None;
}
