//! # educore-events-domain repository port traits
//!
//! Per `docs/specs/events/repositories.md`. 7 repository
//! port traits, one per root aggregate. Each is
//! object-safe (the `_assert_object_safe` helpers prove it).

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

use async_trait::async_trait;
use chrono::NaiveDate;

use educore_core::error::Result as StorageResult;
use educore_core::ids::SchoolId;

use crate::aggregate::{
    AssignIncident, CalendarEvent, CalendarSetting, Holiday, Incident, IncidentComment, Weekend,
};
use crate::query::{
    AssignIncidentQuery, CalendarEventQuery, CalendarSettingQuery, HolidayQuery, IncidentCommentQuery,
    IncidentQuery, WeekendQuery,
};
use crate::value_objects::{
    AcademicYearRef, AssignIncidentId, CalendarEventId, CalendarSettingId, ForWhom, HolidayId,
    IncidentCommentId, IncidentId, WeekendId,
};

// =============================================================================
// === CalendarEventRepository section begin (owner: A) ===
// =============================================================================

/// Repository port for [`CalendarEvent`](crate::aggregate::CalendarEvent).
#[async_trait]
pub trait CalendarEventRepository: Send + Sync {
    /// Fetch a CalendarEvent by id.
    async fn get(&self, id: CalendarEventId) -> StorageResult<Option<CalendarEvent>>;
    /// List CalendarEvents for a school matching the query.
    async fn list(&self, school: SchoolId, q: CalendarEventQuery) -> StorageResult<Vec<CalendarEvent>>;
    /// Insert a new CalendarEvent.
    async fn insert(&self, e: &CalendarEvent) -> StorageResult<()>;
    /// Update an existing CalendarEvent.
    async fn update(&self, e: &CalendarEvent) -> StorageResult<()>;
    /// Soft-delete a CalendarEvent.
    async fn delete(&self, id: CalendarEventId) -> StorageResult<()>;
    /// List CalendarEvents in a date range.
    async fn between(&self, school: SchoolId, from: NaiveDate, to: NaiveDate) -> StorageResult<Vec<CalendarEvent>>;
    /// List CalendarEvents for a specific audience.
    async fn for_audience(&self, school: SchoolId, for_whom: ForWhom, role_id: &str) -> StorageResult<Vec<CalendarEvent>>;
}

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
    /// Fetch a Holiday by id.
    async fn get(&self, id: HolidayId) -> StorageResult<Option<Holiday>>;
    /// List Holidays for a school matching the query.
    async fn list(&self, school: SchoolId, q: HolidayQuery) -> StorageResult<Vec<Holiday>>;
    /// Insert a new Holiday.
    async fn insert(&self, h: &Holiday) -> StorageResult<()>;
    /// Update an existing Holiday.
    async fn update(&self, h: &Holiday) -> StorageResult<()>;
    /// Soft-delete a Holiday.
    async fn delete(&self, id: HolidayId) -> StorageResult<()>;
    /// List Holidays in a date range.
    async fn between(&self, school: SchoolId, from: NaiveDate, to: NaiveDate) -> StorageResult<Vec<Holiday>>;
    /// List Holidays for an academic year.
    async fn in_year(&self, school: SchoolId, year: AcademicYearRef) -> StorageResult<Vec<Holiday>>;
}

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
    /// Fetch a CalendarSetting by id.
    async fn get(&self, id: CalendarSettingId) -> StorageResult<Option<CalendarSetting>>;
    /// List CalendarSettings for a school matching the query.
    async fn list(&self, school: SchoolId, q: CalendarSettingQuery) -> StorageResult<Vec<CalendarSetting>>;
    /// Find a CalendarSetting by menu name.
    async fn find_by_name(&self, school: SchoolId, name: &str) -> StorageResult<Option<CalendarSetting>>;
    /// List enabled CalendarSettings for a school.
    async fn list_enabled(&self, school: SchoolId) -> StorageResult<Vec<CalendarSetting>>;
    /// Insert a new CalendarSetting.
    async fn insert(&self, s: &CalendarSetting) -> StorageResult<()>;
    /// Update an existing CalendarSetting.
    async fn update(&self, s: &CalendarSetting) -> StorageResult<()>;
    /// Soft-delete a CalendarSetting.
    async fn delete(&self, id: CalendarSettingId) -> StorageResult<()>;
}

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
    /// Fetch an Incident by id.
    async fn get(&self, id: IncidentId) -> StorageResult<Option<Incident>>;
    /// List Incidents for a school matching the query.
    async fn list(&self, school: SchoolId, q: IncidentQuery) -> StorageResult<Vec<Incident>>;
    /// Insert a new Incident.
    async fn insert(&self, i: &Incident) -> StorageResult<()>;
    /// Update an existing Incident.
    async fn update(&self, i: &Incident) -> StorageResult<()>;
    /// Soft-delete an Incident.
    async fn delete(&self, id: IncidentId) -> StorageResult<()>;
    /// List open Incidents.
    async fn open(&self, school: SchoolId) -> StorageResult<Vec<Incident>>;
    /// List in-progress Incidents.
    async fn in_progress(&self, school: SchoolId) -> StorageResult<Vec<Incident>>;
    /// List Incidents in a date range.
    async fn between(&self, school: SchoolId, from: NaiveDate, to: NaiveDate) -> StorageResult<Vec<Incident>>;
}

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
    /// Fetch an AssignIncident by id.
    async fn get(&self, id: AssignIncidentId) -> StorageResult<Option<AssignIncident>>;
    /// List AssignIncidents for an Incident.
    async fn list_for_incident(&self, incident: IncidentId) -> StorageResult<Vec<AssignIncident>>;
    /// List AssignIncidents for a student.
    async fn list_for_student(&self, school: SchoolId, student: uuid::Uuid) -> StorageResult<Vec<AssignIncident>>;
    /// List AssignIncidents for a user.
    async fn list_for_user(&self, school: SchoolId, user: uuid::Uuid) -> StorageResult<Vec<AssignIncident>>;
    /// Insert a new AssignIncident.
    async fn insert(&self, a: &AssignIncident) -> StorageResult<()>;
    /// Update an existing AssignIncident.
    async fn update(&self, a: &AssignIncident) -> StorageResult<()>;
    /// Soft-delete an AssignIncident.
    async fn delete(&self, id: AssignIncidentId) -> StorageResult<()>;
}

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
    /// Fetch an IncidentComment by id.
    async fn get(&self, id: IncidentCommentId) -> StorageResult<Option<IncidentComment>>;
    /// List IncidentComments for an Incident.
    async fn list_for_incident(&self, incident: IncidentId) -> StorageResult<Vec<IncidentComment>>;
    /// Insert a new IncidentComment.
    async fn insert(&self, c: &IncidentComment) -> StorageResult<()>;
    /// Soft-delete an IncidentComment.
    async fn delete(&self, id: IncidentCommentId) -> StorageResult<()>;
}

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
    /// Fetch a Weekend by id.
    async fn get(&self, id: WeekendId) -> StorageResult<Option<Weekend>>;
    /// List Weekends for a school.
    async fn list(&self, school: SchoolId) -> StorageResult<Vec<Weekend>>;
    /// Find a Weekend by name.
    async fn find_by_name(&self, school: SchoolId, name: &str) -> StorageResult<Option<Weekend>>;
    /// Insert a new Weekend.
    async fn insert(&self, w: &Weekend) -> StorageResult<()>;
    /// Update an existing Weekend.
    async fn update(&self, w: &Weekend) -> StorageResult<()>;
    /// Soft-delete a Weekend.
    async fn delete(&self, id: WeekendId) -> StorageResult<()>;
}

fn _assert_weekend_object_safe() {
    fn _f(_: Box<dyn WeekendRepository>) {}
}

// === WeekendRepository section end ===
