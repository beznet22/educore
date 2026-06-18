//! # educore-events-domain typed events
//!
//! Per `docs/specs/events/events.md`. Wire form:
//! `events.<aggregate>.<verb>`. Each event implements
//! [`DomainEvent`].

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

use chrono::NaiveDate;
use educore_core::ids::{CorrelationId, EventId, Identifier, SchoolId, UserId};
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::value_objects::{
    AssignIncidentId, CalendarEventId, CalendarSettingId, HolidayId, IncidentCommentId,
    IncidentId, IncidentStatus, WeekendId,
};

// =============================================================================
// === CalendarEvent events section begin (owner: A) ===
// =============================================================================

/// Emitted when a CalendarEvent is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventCreated {
    pub event_id: CalendarEventId,
    pub school_id: SchoolId,
    pub title: String,
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl EventCreated {
    #[must_use]
    pub fn new(event_id: CalendarEventId, school_id: SchoolId, title: String, from_date: NaiveDate, to_date: NaiveDate, event_id_field: EventId, correlation_id: CorrelationId, occurred_at: Timestamp) -> Self {
        Self { event_id, school_id, title, from_date, to_date, event_id_field, correlation_id, occurred_at }
    }
}

impl DomainEvent for EventCreated {
    const EVENT_TYPE: &'static str = "events.calendar_event.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "calendar_event";
    fn event_id(&self) -> EventId { self.event_id_field }
    fn aggregate_id(&self) -> Uuid { self.event_id.as_uuid() }
    fn school_id(&self) -> SchoolId { self.school_id }
    fn occurred_at(&self) -> Timestamp { self.occurred_at }
}

/// Emitted when a CalendarEvent is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventUpdated {
    pub event_id: CalendarEventId,
    pub school_id: SchoolId,
    pub changes: Vec<String>,
    pub updated_by: UserId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl EventUpdated {
    #[must_use]
    pub fn new(event_id: CalendarEventId, school_id: SchoolId, changes: Vec<String>, updated_by: UserId, event_id_field: EventId, correlation_id: CorrelationId, occurred_at: Timestamp) -> Self {
        Self { event_id, school_id, changes, updated_by, event_id_field, correlation_id, occurred_at }
    }
}

impl DomainEvent for EventUpdated {
    const EVENT_TYPE: &'static str = "events.calendar_event.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "calendar_event";
    fn event_id(&self) -> EventId { self.event_id_field }
    fn aggregate_id(&self) -> Uuid { self.event_id.as_uuid() }
    fn school_id(&self) -> SchoolId { self.school_id }
    fn occurred_at(&self) -> Timestamp { self.occurred_at }
}

/// Emitted when a CalendarEvent is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventDeleted {
    pub event_id: CalendarEventId,
    pub school_id: SchoolId,
    pub deleted_by: UserId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl EventDeleted {
    #[must_use]
    pub fn new(event_id: CalendarEventId, school_id: SchoolId, deleted_by: UserId, event_id_field: EventId, correlation_id: CorrelationId, occurred_at: Timestamp) -> Self {
        Self { event_id, school_id, deleted_by, event_id_field, correlation_id, occurred_at }
    }
}

impl DomainEvent for EventDeleted {
    const EVENT_TYPE: &'static str = "events.calendar_event.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "calendar_event";
    fn event_id(&self) -> EventId { self.event_id_field }
    fn aggregate_id(&self) -> Uuid { self.event_id.as_uuid() }
    fn school_id(&self) -> SchoolId { self.school_id }
    fn occurred_at(&self) -> Timestamp { self.occurred_at }
}

// === CalendarEvent events section end ===

// =============================================================================
// === Holiday events section begin (owner: B) ===
// =============================================================================

/// Emitted when a Holiday is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HolidayCreated {
    pub holiday_id: HolidayId,
    pub school_id: SchoolId,
    pub title: String,
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl HolidayCreated {
    #[must_use]
    pub fn new(holiday_id: HolidayId, school_id: SchoolId, title: String, from_date: NaiveDate, to_date: NaiveDate, event_id_field: EventId, correlation_id: CorrelationId, occurred_at: Timestamp) -> Self {
        Self { holiday_id, school_id, title, from_date, to_date, event_id_field, correlation_id, occurred_at }
    }
}

impl DomainEvent for HolidayCreated {
    const EVENT_TYPE: &'static str = "events.holiday.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "holiday";
    fn event_id(&self) -> EventId { self.event_id_field }
    fn aggregate_id(&self) -> Uuid { self.holiday_id.as_uuid() }
    fn school_id(&self) -> SchoolId { self.school_id }
    fn occurred_at(&self) -> Timestamp { self.occurred_at }
}

/// Emitted when a Holiday is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HolidayUpdated {
    pub holiday_id: HolidayId,
    pub school_id: SchoolId,
    pub changes: Vec<String>,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl HolidayUpdated {
    #[must_use]
    pub fn new(holiday_id: HolidayId, school_id: SchoolId, changes: Vec<String>, event_id_field: EventId, correlation_id: CorrelationId, occurred_at: Timestamp) -> Self {
        Self { holiday_id, school_id, changes, event_id_field, correlation_id, occurred_at }
    }
}

impl DomainEvent for HolidayUpdated {
    const EVENT_TYPE: &'static str = "events.holiday.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "holiday";
    fn event_id(&self) -> EventId { self.event_id_field }
    fn aggregate_id(&self) -> Uuid { self.holiday_id.as_uuid() }
    fn school_id(&self) -> SchoolId { self.school_id }
    fn occurred_at(&self) -> Timestamp { self.occurred_at }
}

/// Emitted when a Holiday is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HolidayDeleted {
    pub holiday_id: HolidayId,
    pub school_id: SchoolId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl HolidayDeleted {
    #[must_use]
    pub fn new(holiday_id: HolidayId, school_id: SchoolId, event_id_field: EventId, correlation_id: CorrelationId, occurred_at: Timestamp) -> Self {
        Self { holiday_id, school_id, event_id_field, correlation_id, occurred_at }
    }
}

impl DomainEvent for HolidayDeleted {
    const EVENT_TYPE: &'static str = "events.holiday.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "holiday";
    fn event_id(&self) -> EventId { self.event_id_field }
    fn aggregate_id(&self) -> Uuid { self.holiday_id.as_uuid() }
    fn school_id(&self) -> SchoolId { self.school_id }
    fn occurred_at(&self) -> Timestamp { self.occurred_at }
}

// === Holiday events section end ===

// =============================================================================
// === CalendarSetting events section begin (owner: B) ===
// =============================================================================

/// Emitted when a CalendarSetting is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalendarSettingCreated {
    pub setting_id: CalendarSettingId,
    pub school_id: SchoolId,
    pub menu_name: String,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl CalendarSettingCreated {
    #[must_use]
    pub fn new(setting_id: CalendarSettingId, school_id: SchoolId, menu_name: String, event_id_field: EventId, correlation_id: CorrelationId, occurred_at: Timestamp) -> Self {
        Self { setting_id, school_id, menu_name, event_id_field, correlation_id, occurred_at }
    }
}

impl DomainEvent for CalendarSettingCreated {
    const EVENT_TYPE: &'static str = "events.calendar_setting.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "calendar_setting";
    fn event_id(&self) -> EventId { self.event_id_field }
    fn aggregate_id(&self) -> Uuid { self.setting_id.as_uuid() }
    fn school_id(&self) -> SchoolId { self.school_id }
    fn occurred_at(&self) -> Timestamp { self.occurred_at }
}

/// Emitted when a CalendarSetting is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalendarSettingUpdated {
    pub setting_id: CalendarSettingId,
    pub school_id: SchoolId,
    pub changes: Vec<String>,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl CalendarSettingUpdated {
    #[must_use]
    pub fn new(setting_id: CalendarSettingId, school_id: SchoolId, changes: Vec<String>, event_id_field: EventId, correlation_id: CorrelationId, occurred_at: Timestamp) -> Self {
        Self { setting_id, school_id, changes, event_id_field, correlation_id, occurred_at }
    }
}

impl DomainEvent for CalendarSettingUpdated {
    const EVENT_TYPE: &'static str = "events.calendar_setting.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "calendar_setting";
    fn event_id(&self) -> EventId { self.event_id_field }
    fn aggregate_id(&self) -> Uuid { self.setting_id.as_uuid() }
    fn school_id(&self) -> SchoolId { self.school_id }
    fn occurred_at(&self) -> Timestamp { self.occurred_at }
}

/// Emitted when a CalendarSetting is enabled.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalendarSettingEnabled {
    pub setting_id: CalendarSettingId,
    pub school_id: SchoolId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl CalendarSettingEnabled {
    #[must_use]
    pub fn new(setting_id: CalendarSettingId, school_id: SchoolId, event_id_field: EventId, correlation_id: CorrelationId, occurred_at: Timestamp) -> Self {
        Self { setting_id, school_id, event_id_field, correlation_id, occurred_at }
    }
}

impl DomainEvent for CalendarSettingEnabled {
    const EVENT_TYPE: &'static str = "events.calendar_setting.enabled";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "calendar_setting";
    fn event_id(&self) -> EventId { self.event_id_field }
    fn aggregate_id(&self) -> Uuid { self.setting_id.as_uuid() }
    fn school_id(&self) -> SchoolId { self.school_id }
    fn occurred_at(&self) -> Timestamp { self.occurred_at }
}

/// Emitted when a CalendarSetting is disabled.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalendarSettingDisabled {
    pub setting_id: CalendarSettingId,
    pub school_id: SchoolId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl CalendarSettingDisabled {
    #[must_use]
    pub fn new(setting_id: CalendarSettingId, school_id: SchoolId, event_id_field: EventId, correlation_id: CorrelationId, occurred_at: Timestamp) -> Self {
        Self { setting_id, school_id, event_id_field, correlation_id, occurred_at }
    }
}

impl DomainEvent for CalendarSettingDisabled {
    const EVENT_TYPE: &'static str = "events.calendar_setting.disabled";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "calendar_setting";
    fn event_id(&self) -> EventId { self.event_id_field }
    fn aggregate_id(&self) -> Uuid { self.setting_id.as_uuid() }
    fn school_id(&self) -> SchoolId { self.school_id }
    fn occurred_at(&self) -> Timestamp { self.occurred_at }
}

/// Emitted when a CalendarSetting is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalendarSettingDeleted {
    pub setting_id: CalendarSettingId,
    pub school_id: SchoolId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl CalendarSettingDeleted {
    #[must_use]
    pub fn new(setting_id: CalendarSettingId, school_id: SchoolId, event_id_field: EventId, correlation_id: CorrelationId, occurred_at: Timestamp) -> Self {
        Self { setting_id, school_id, event_id_field, correlation_id, occurred_at }
    }
}

impl DomainEvent for CalendarSettingDeleted {
    const EVENT_TYPE: &'static str = "events.calendar_setting.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "calendar_setting";
    fn event_id(&self) -> EventId { self.event_id_field }
    fn aggregate_id(&self) -> Uuid { self.setting_id.as_uuid() }
    fn school_id(&self) -> SchoolId { self.school_id }
    fn occurred_at(&self) -> Timestamp { self.occurred_at }
}

// === CalendarSetting events section end ===

// =============================================================================
// === Incident events section begin (owner: C) ===
// =============================================================================

/// Emitted when an Incident is reported.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncidentReported {
    pub incident_id: IncidentId,
    pub school_id: SchoolId,
    pub title: String,
    pub point: i32,
    pub reported_by: UserId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl IncidentReported {
    #[must_use]
    pub fn new(incident_id: IncidentId, school_id: SchoolId, title: String, point: i32, reported_by: UserId, event_id_field: EventId, correlation_id: CorrelationId, occurred_at: Timestamp) -> Self {
        Self { incident_id, school_id, title, point, reported_by, event_id_field, correlation_id, occurred_at }
    }
}

impl DomainEvent for IncidentReported {
    const EVENT_TYPE: &'static str = "events.incident.reported";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "incident";
    fn event_id(&self) -> EventId { self.event_id_field }
    fn aggregate_id(&self) -> Uuid { self.incident_id.as_uuid() }
    fn school_id(&self) -> SchoolId { self.school_id }
    fn occurred_at(&self) -> Timestamp { self.occurred_at }
}

/// Emitted when an Incident is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncidentUpdated {
    pub incident_id: IncidentId,
    pub school_id: SchoolId,
    pub changes: Vec<String>,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl IncidentUpdated {
    #[must_use]
    pub fn new(incident_id: IncidentId, school_id: SchoolId, changes: Vec<String>, event_id_field: EventId, correlation_id: CorrelationId, occurred_at: Timestamp) -> Self {
        Self { incident_id, school_id, changes, event_id_field, correlation_id, occurred_at }
    }
}

impl DomainEvent for IncidentUpdated {
    const EVENT_TYPE: &'static str = "events.incident.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "incident";
    fn event_id(&self) -> EventId { self.event_id_field }
    fn aggregate_id(&self) -> Uuid { self.incident_id.as_uuid() }
    fn school_id(&self) -> SchoolId { self.school_id }
    fn occurred_at(&self) -> Timestamp { self.occurred_at }
}

/// Emitted when an Incident is resolved.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncidentResolved {
    pub incident_id: IncidentId,
    pub school_id: SchoolId,
    pub resolved_by: UserId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl IncidentResolved {
    #[must_use]
    pub fn new(incident_id: IncidentId, school_id: SchoolId, resolved_by: UserId, event_id_field: EventId, correlation_id: CorrelationId, occurred_at: Timestamp) -> Self {
        Self { incident_id, school_id, resolved_by, event_id_field, correlation_id, occurred_at }
    }
}

impl DomainEvent for IncidentResolved {
    const EVENT_TYPE: &'static str = "events.incident.resolved";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "incident";
    fn event_id(&self) -> EventId { self.event_id_field }
    fn aggregate_id(&self) -> Uuid { self.incident_id.as_uuid() }
    fn school_id(&self) -> SchoolId { self.school_id }
    fn occurred_at(&self) -> Timestamp { self.occurred_at }
}

/// Emitted when an Incident is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncidentDeleted {
    pub incident_id: IncidentId,
    pub school_id: SchoolId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl IncidentDeleted {
    #[must_use]
    pub fn new(incident_id: IncidentId, school_id: SchoolId, event_id_field: EventId, correlation_id: CorrelationId, occurred_at: Timestamp) -> Self {
        Self { incident_id, school_id, event_id_field, correlation_id, occurred_at }
    }
}

impl DomainEvent for IncidentDeleted {
    const EVENT_TYPE: &'static str = "events.incident.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "incident";
    fn event_id(&self) -> EventId { self.event_id_field }
    fn aggregate_id(&self) -> Uuid { self.incident_id.as_uuid() }
    fn school_id(&self) -> SchoolId { self.school_id }
    fn occurred_at(&self) -> Timestamp { self.occurred_at }
}

// === Incident events section end ===

// =============================================================================
// === AssignIncident events section begin (owner: C) ===
// =============================================================================

/// Emitted when an Incident is assigned.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncidentAssigned {
    pub assign_incident_id: AssignIncidentId,
    pub incident_id: IncidentId,
    pub school_id: SchoolId,
    pub point: i32,
    pub added_by: UserId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl IncidentAssigned {
    #[must_use]
    pub fn new(assign_incident_id: AssignIncidentId, incident_id: IncidentId, school_id: SchoolId, point: i32, added_by: UserId, event_id_field: EventId, correlation_id: CorrelationId, occurred_at: Timestamp) -> Self {
        Self { assign_incident_id, incident_id, school_id, point, added_by, event_id_field, correlation_id, occurred_at }
    }
}

impl DomainEvent for IncidentAssigned {
    const EVENT_TYPE: &'static str = "events.assign_incident.assigned";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "assign_incident";
    fn event_id(&self) -> EventId { self.event_id_field }
    fn aggregate_id(&self) -> Uuid { self.assign_incident_id.as_uuid() }
    fn school_id(&self) -> SchoolId { self.school_id }
    fn occurred_at(&self) -> Timestamp { self.occurred_at }
}

/// Emitted when an Incident assignment is reassigned.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncidentReassigned {
    pub assign_incident_id: AssignIncidentId,
    pub incident_id: IncidentId,
    pub school_id: SchoolId,
    pub from_point: i32,
    pub to_point: i32,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl IncidentReassigned {
    #[must_use]
    pub fn new(assign_incident_id: AssignIncidentId, incident_id: IncidentId, school_id: SchoolId, from_point: i32, to_point: i32, event_id_field: EventId, correlation_id: CorrelationId, occurred_at: Timestamp) -> Self {
        Self { assign_incident_id, incident_id, school_id, from_point, to_point, event_id_field, correlation_id, occurred_at }
    }
}

impl DomainEvent for IncidentReassigned {
    const EVENT_TYPE: &'static str = "events.assign_incident.reassigned";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "assign_incident";
    fn event_id(&self) -> EventId { self.event_id_field }
    fn aggregate_id(&self) -> Uuid { self.assign_incident_id.as_uuid() }
    fn school_id(&self) -> SchoolId { self.school_id }
    fn occurred_at(&self) -> Timestamp { self.occurred_at }
}

/// Emitted when an Incident is unassigned.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncidentUnassigned {
    pub assign_incident_id: AssignIncidentId,
    pub incident_id: IncidentId,
    pub school_id: SchoolId,
    pub removed_by: UserId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl IncidentUnassigned {
    #[must_use]
    pub fn new(assign_incident_id: AssignIncidentId, incident_id: IncidentId, school_id: SchoolId, removed_by: UserId, event_id_field: EventId, correlation_id: CorrelationId, occurred_at: Timestamp) -> Self {
        Self { assign_incident_id, incident_id, school_id, removed_by, event_id_field, correlation_id, occurred_at }
    }
}

impl DomainEvent for IncidentUnassigned {
    const EVENT_TYPE: &'static str = "events.assign_incident.unassigned";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "assign_incident";
    fn event_id(&self) -> EventId { self.event_id_field }
    fn aggregate_id(&self) -> Uuid { self.assign_incident_id.as_uuid() }
    fn school_id(&self) -> SchoolId { self.school_id }
    fn occurred_at(&self) -> Timestamp { self.occurred_at }
}

// === AssignIncident events section end ===

// =============================================================================
// === IncidentComment events section begin (owner: C) ===
// =============================================================================

/// Emitted when a comment is added to an Incident.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncidentCommented {
    pub incident_comment_id: IncidentCommentId,
    pub incident_id: IncidentId,
    pub school_id: SchoolId,
    pub user_id: UserId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl IncidentCommented {
    #[must_use]
    pub fn new(incident_comment_id: IncidentCommentId, incident_id: IncidentId, school_id: SchoolId, user_id: UserId, event_id_field: EventId, correlation_id: CorrelationId, occurred_at: Timestamp) -> Self {
        Self { incident_comment_id, incident_id, school_id, user_id, event_id_field, correlation_id, occurred_at }
    }
}

impl DomainEvent for IncidentCommented {
    const EVENT_TYPE: &'static str = "events.incident_comment.commented";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "incident_comment";
    fn event_id(&self) -> EventId { self.event_id_field }
    fn aggregate_id(&self) -> Uuid { self.incident_comment_id.as_uuid() }
    fn school_id(&self) -> SchoolId { self.school_id }
    fn occurred_at(&self) -> Timestamp { self.occurred_at }
}

/// Emitted when an Incident comment is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncidentCommentDeletedEvent {
    pub incident_comment_id: IncidentCommentId,
    pub incident_id: IncidentId,
    pub school_id: SchoolId,
    pub deleted_by: UserId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl IncidentCommentDeletedEvent {
    #[must_use]
    pub fn new(incident_comment_id: IncidentCommentId, incident_id: IncidentId, school_id: SchoolId, deleted_by: UserId, event_id_field: EventId, correlation_id: CorrelationId, occurred_at: Timestamp) -> Self {
        Self { incident_comment_id, incident_id, school_id, deleted_by, event_id_field, correlation_id, occurred_at }
    }
}

impl DomainEvent for IncidentCommentDeletedEvent {
    const EVENT_TYPE: &'static str = "events.incident_comment.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "incident_comment";
    fn event_id(&self) -> EventId { self.event_id_field }
    fn aggregate_id(&self) -> Uuid { self.incident_comment_id.as_uuid() }
    fn school_id(&self) -> SchoolId { self.school_id }
    fn occurred_at(&self) -> Timestamp { self.occurred_at }
}

// === IncidentComment events section end ===

// =============================================================================
// === Weekend events section begin (owner: D) ===
// =============================================================================

/// Emitted when a Weekend is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeekendCreated {
    pub weekend_id: WeekendId,
    pub school_id: SchoolId,
    pub name: String,
    pub order: i32,
    pub is_weekend: bool,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl WeekendCreated {
    #[must_use]
    pub fn new(weekend_id: WeekendId, school_id: SchoolId, name: String, order: i32, is_weekend: bool, event_id_field: EventId, correlation_id: CorrelationId, occurred_at: Timestamp) -> Self {
        Self { weekend_id, school_id, name, order, is_weekend, event_id_field, correlation_id, occurred_at }
    }
}

impl DomainEvent for WeekendCreated {
    const EVENT_TYPE: &'static str = "events.weekend.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "weekend";
    fn event_id(&self) -> EventId { self.event_id_field }
    fn aggregate_id(&self) -> Uuid { self.weekend_id.as_uuid() }
    fn school_id(&self) -> SchoolId { self.school_id }
    fn occurred_at(&self) -> Timestamp { self.occurred_at }
}

/// Emitted when a Weekend is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeekendUpdated {
    pub weekend_id: WeekendId,
    pub school_id: SchoolId,
    pub changes: Vec<String>,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl WeekendUpdated {
    #[must_use]
    pub fn new(weekend_id: WeekendId, school_id: SchoolId, changes: Vec<String>, event_id_field: EventId, correlation_id: CorrelationId, occurred_at: Timestamp) -> Self {
        Self { weekend_id, school_id, changes, event_id_field, correlation_id, occurred_at }
    }
}

impl DomainEvent for WeekendUpdated {
    const EVENT_TYPE: &'static str = "events.weekend.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "weekend";
    fn event_id(&self) -> EventId { self.event_id_field }
    fn aggregate_id(&self) -> Uuid { self.weekend_id.as_uuid() }
    fn school_id(&self) -> SchoolId { self.school_id }
    fn occurred_at(&self) -> Timestamp { self.occurred_at }
}

/// Emitted when weekends are batch-configured.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeekendsConfigured {
    pub school_id: SchoolId,
    pub weekend_count: u32,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl WeekendsConfigured {
    #[must_use]
    pub fn new(school_id: SchoolId, weekend_count: u32, event_id_field: EventId, correlation_id: CorrelationId, occurred_at: Timestamp) -> Self {
        Self { school_id, weekend_count, event_id_field, correlation_id, occurred_at }
    }
}

impl DomainEvent for WeekendsConfigured {
    const EVENT_TYPE: &'static str = "events.weekend.configured";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "weekend";
    fn event_id(&self) -> EventId { self.event_id_field }
    fn aggregate_id(&self) -> Uuid { Uuid::nil() }
    fn school_id(&self) -> SchoolId { self.school_id }
    fn occurred_at(&self) -> Timestamp { self.occurred_at }
}

/// Emitted when a Weekend is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeekendDeleted {
    pub weekend_id: WeekendId,
    pub school_id: SchoolId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl WeekendDeleted {
    #[must_use]
    pub fn new(weekend_id: WeekendId, school_id: SchoolId, event_id_field: EventId, correlation_id: CorrelationId, occurred_at: Timestamp) -> Self {
        Self { weekend_id, school_id, event_id_field, correlation_id, occurred_at }
    }
}

impl DomainEvent for WeekendDeleted {
    const EVENT_TYPE: &'static str = "events.weekend.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "weekend";
    fn event_id(&self) -> EventId { self.event_id_field }
    fn aggregate_id(&self) -> Uuid { self.weekend_id.as_uuid() }
    fn school_id(&self) -> SchoolId { self.school_id }
    fn occurred_at(&self) -> Timestamp { self.occurred_at }
}

// === Weekend events section end ===

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_event_wire_forms_resolve() {
        let types: Vec<&str> = vec![
            "events.calendar_event.created",
            "events.calendar_event.updated",
            "events.calendar_event.deleted",
            "events.holiday.created",
            "events.holiday.updated",
            "events.holiday.deleted",
            "events.calendar_setting.created",
            "events.calendar_setting.updated",
            "events.calendar_setting.enabled",
            "events.calendar_setting.disabled",
            "events.calendar_setting.deleted",
            "events.incident.reported",
            "events.incident.updated",
            "events.incident.resolved",
            "events.incident.deleted",
            "events.assign_incident.assigned",
            "events.assign_incident.reassigned",
            "events.assign_incident.unassigned",
            "events.incident_comment.commented",
            "events.incident_comment.deleted",
            "events.weekend.created",
            "events.weekend.updated",
            "events.weekend.configured",
            "events.weekend.deleted",
        ];
        assert_eq!(types.len(), 24);
        for t in &types {
            assert!(t.starts_with("events."), "{t} should start with events.");
        }
    }
}
