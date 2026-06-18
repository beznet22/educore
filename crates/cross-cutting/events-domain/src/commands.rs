//! # educore-events-domain typed commands
//!
//! Per `docs/specs/events/commands.md`. 24 typed command
//! shapes across 7 aggregates.

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

use chrono::NaiveDate;
use educore_core::ids::{CorrelationId, Identifier, SchoolId, UserId};
use educore_core::tenant::TenantContext;
use educore_core::value_objects::Timestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::aggregate::{
    NewCalendarEvent, NewCalendarSetting, NewHoliday, NewIncident,
};
use crate::value_objects::{
    AcademicYearRef, AssignIncidentId, CalendarEventId, CalendarSettingId, FileRef,
    ForWhom, HolidayId, IncidentCommentId, IncidentId, RecurrenceRule, Url, WeekendId,
};

// =============================================================================
// === CalendarEvent commands section begin (owner: A) ===
// =============================================================================

/// Create a new CalendarEvent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateEventCommand {
    pub tenant: TenantContext,
    pub title: String,
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
    pub for_whom: ForWhom,
    pub role_ids: Vec<String>,
    pub url: Option<Url>,
    pub location: Option<String>,
    pub description: Option<String>,
    pub image: Option<FileRef>,
    pub rrule: Option<RecurrenceRule>,
    pub academic_id: AcademicYearRef,
}

impl CreateEventCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "events.calendar_event.create";

    /// Converts to a `NewCalendarEvent` aggregate input.
    #[must_use]
    pub fn into_new_event(self, id: CalendarEventId) -> NewCalendarEvent {
        let now = Timestamp::now();
        NewCalendarEvent {
            id,
            title: self.title,
            from_date: self.from_date,
            to_date: self.to_date,
            for_whom: self.for_whom,
            role_ids: self.role_ids,
            url: self.url,
            location: self.location,
            description: self.description,
            image: self.image,
            rrule: self.rrule,
            academic_id: self.academic_id,
            created_by: self.tenant.actor_id,
            created_at: now,
            correlation_id: self.tenant.correlation_id,
        }
    }
}

/// Update an existing CalendarEvent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateEventCommand {
    pub tenant: TenantContext,
    pub event_id: CalendarEventId,
    pub title: Option<String>,
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
}

impl UpdateEventCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "events.calendar_event.update";
}

/// Soft-delete a CalendarEvent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteEventCommand {
    pub tenant: TenantContext,
    pub event_id: CalendarEventId,
}

impl DeleteEventCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "events.calendar_event.delete";
}

// === CalendarEvent commands section end ===

// =============================================================================
// === Holiday commands section begin (owner: B) ===
// =============================================================================

/// Create a new Holiday.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateHolidayCommand {
    pub tenant: TenantContext,
    pub title: String,
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
    pub details: Option<String>,
    pub image: Option<FileRef>,
    pub academic_id: AcademicYearRef,
}

impl CreateHolidayCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "events.holiday.create";

    /// Converts to a `NewHoliday` aggregate input.
    #[must_use]
    pub fn into_new_holiday(self, id: HolidayId) -> NewHoliday {
        let now = Timestamp::now();
        NewHoliday {
            id,
            title: self.title,
            from_date: self.from_date,
            to_date: self.to_date,
            details: self.details,
            image: self.image,
            academic_id: self.academic_id,
            created_by: self.tenant.actor_id,
            created_at: now,
            correlation_id: self.tenant.correlation_id,
        }
    }
}

/// Update a Holiday.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateHolidayCommand {
    pub tenant: TenantContext,
    pub holiday_id: HolidayId,
    pub title: Option<String>,
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
}

impl UpdateHolidayCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "events.holiday.update";
}

/// Soft-delete a Holiday.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteHolidayCommand {
    pub tenant: TenantContext,
    pub holiday_id: HolidayId,
}

impl DeleteHolidayCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "events.holiday.delete";
}

// === Holiday commands section end ===

// =============================================================================
// === CalendarSetting commands section begin (owner: B) ===
// =============================================================================

/// Create a new CalendarSetting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateCalendarSettingCommand {
    pub tenant: TenantContext,
    pub menu_name: String,
    pub font_color: String,
    pub bg_color: String,
}

impl CreateCalendarSettingCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "events.calendar_setting.create";

    /// Converts to a `NewCalendarSetting` aggregate input.
    #[must_use]
    pub fn into_new_setting(self, id: CalendarSettingId) -> NewCalendarSetting {
        let now = Timestamp::now();
        NewCalendarSetting {
            id,
            menu_name: self.menu_name,
            status: crate::value_objects::CalendarStatus::Enabled,
            font_color: self.font_color,
            bg_color: self.bg_color,
            created_by: self.tenant.actor_id,
            created_at: now,
            correlation_id: self.tenant.correlation_id,
        }
    }
}

/// Update a CalendarSetting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateCalendarSettingCommand {
    pub tenant: TenantContext,
    pub setting_id: CalendarSettingId,
    pub menu_name: Option<String>,
    pub font_color: Option<String>,
    pub bg_color: Option<String>,
}

impl UpdateCalendarSettingCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "events.calendar_setting.update";
}

/// Enable a CalendarSetting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnableCalendarSettingCommand {
    pub tenant: TenantContext,
    pub setting_id: CalendarSettingId,
}

impl EnableCalendarSettingCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "events.calendar_setting.enable";
}

/// Disable a CalendarSetting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisableCalendarSettingCommand {
    pub tenant: TenantContext,
    pub setting_id: CalendarSettingId,
}

impl DisableCalendarSettingCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "events.calendar_setting.disable";
}

/// Soft-delete a CalendarSetting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteCalendarSettingCommand {
    pub tenant: TenantContext,
    pub setting_id: CalendarSettingId,
}

impl DeleteCalendarSettingCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "events.calendar_setting.delete";
}

// === CalendarSetting commands section end ===

// =============================================================================
// === Incident commands section begin (owner: C) ===
// =============================================================================

/// Create (report) a new Incident.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateIncidentCommand {
    pub tenant: TenantContext,
    pub title: String,
    pub point: i32,
    pub description: String,
}

impl CreateIncidentCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "events.incident.create";

    /// Converts to a `NewIncident` aggregate input.
    #[must_use]
    pub fn into_new_incident(self, id: IncidentId) -> NewIncident {
        let now = Timestamp::now();
        NewIncident {
            id,
            title: self.title,
            point: self.point,
            description: self.description,
            created_by: self.tenant.actor_id,
            created_at: now,
            correlation_id: self.tenant.correlation_id,
        }
    }
}

/// Update an Incident.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateIncidentCommand {
    pub tenant: TenantContext,
    pub incident_id: IncidentId,
    pub title: Option<String>,
    pub point: Option<i32>,
    pub description: Option<String>,
}

impl UpdateIncidentCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "events.incident.update";
}

/// Resolve an Incident.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolveIncidentCommand {
    pub tenant: TenantContext,
    pub incident_id: IncidentId,
}

impl ResolveIncidentCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "events.incident.resolve";
}

/// Soft-delete an Incident (admin override).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteIncidentCommand {
    pub tenant: TenantContext,
    pub incident_id: IncidentId,
}

impl DeleteIncidentCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "events.incident.delete";
}

// === Incident commands section end ===

// =============================================================================
// === AssignIncident commands section begin (owner: C) ===
// =============================================================================

/// Assign an Incident to a student or staff.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssignIncidentCommand {
    pub tenant: TenantContext,
    pub incident_id: IncidentId,
    pub student_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub point: i32,
}

impl AssignIncidentCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "events.assign_incident.assign";
}

/// Reassign an Incident (update point).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReassignIncidentCommand {
    pub tenant: TenantContext,
    pub assign_incident_id: AssignIncidentId,
    pub point: i32,
}

impl ReassignIncidentCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "events.assign_incident.reassign";
}

/// Unassign an Incident.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnassignIncidentCommand {
    pub tenant: TenantContext,
    pub assign_incident_id: AssignIncidentId,
}

impl UnassignIncidentCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "events.assign_incident.unassign";
}

// === AssignIncident commands section end ===

// =============================================================================
// === IncidentComment commands section begin (owner: C) ===
// =============================================================================

/// Comment on an Incident.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommentOnIncidentCommand {
    pub tenant: TenantContext,
    pub incident_id: IncidentId,
    pub comment: String,
}

impl CommentOnIncidentCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "events.incident_comment.comment";
}

/// Soft-delete an Incident comment (admin override).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteIncidentCommentCommand {
    pub tenant: TenantContext,
    pub incident_comment_id: IncidentCommentId,
}

impl DeleteIncidentCommentCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "events.incident_comment.delete";
}

// === IncidentComment commands section end ===

// =============================================================================
// === Weekend commands section begin (owner: D) ===
// =============================================================================

/// Create a new Weekend entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateWeekendCommand {
    pub tenant: TenantContext,
    pub name: String,
    pub order: i32,
    pub is_weekend: bool,
}

impl CreateWeekendCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "events.weekend.create";
}

/// Update a Weekend entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateWeekendCommand {
    pub tenant: TenantContext,
    pub weekend_id: WeekendId,
    pub name: Option<String>,
    pub order: Option<i32>,
    pub is_weekend: Option<bool>,
}

impl UpdateWeekendCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "events.weekend.update";
}

/// Batch-configure weekends.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigureWeekendsCommand {
    pub tenant: TenantContext,
    pub entries: Vec<WeekendEntry>,
}

impl ConfigureWeekendsCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "events.weekend.configure";
}

/// A single entry in a `ConfigureWeekends` command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeekendEntry {
    pub name: String,
    pub order: i32,
    pub is_weekend: bool,
}

/// Soft-delete a Weekend.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteWeekendCommand {
    pub tenant: TenantContext,
    pub weekend_id: WeekendId,
}

impl DeleteWeekendCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "events.weekend.delete";
}

// === Weekend commands section end ===

#[allow(dead_code)]
fn _ensure_ids_compile(school: SchoolId) {
    let _ = CalendarEventId::new(school, Uuid::nil());
    let _ = HolidayId::new(school, Uuid::nil());
    let _ = CalendarSettingId::new(school, Uuid::nil());
    let _ = IncidentId::new(school, Uuid::nil());
    let _ = AssignIncidentId::new(school, Uuid::nil());
    let _ = IncidentCommentId::new(school, Uuid::nil());
    let _ = WeekendId::new(school, Uuid::nil());
    let _: Option<CorrelationId> = None;
    let _: Option<UserId> = None;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::{CalendarStatus, RecurrenceFreq};

    #[test]
    fn command_types_have_wire_form() {
        assert_eq!(CreateEventCommand::COMMAND_TYPE, "events.calendar_event.create");
        assert_eq!(UpdateEventCommand::COMMAND_TYPE, "events.calendar_event.update");
        assert_eq!(DeleteEventCommand::COMMAND_TYPE, "events.calendar_event.delete");
        assert_eq!(CreateHolidayCommand::COMMAND_TYPE, "events.holiday.create");
        assert_eq!(UpdateHolidayCommand::COMMAND_TYPE, "events.holiday.update");
        assert_eq!(DeleteHolidayCommand::COMMAND_TYPE, "events.holiday.delete");
        assert_eq!(CreateIncidentCommand::COMMAND_TYPE, "events.incident.create");
        assert_eq!(ResolveIncidentCommand::COMMAND_TYPE, "events.incident.resolve");
        assert_eq!(CommentOnIncidentCommand::COMMAND_TYPE, "events.incident_comment.comment");
        assert_eq!(CreateWeekendCommand::COMMAND_TYPE, "events.weekend.create");
        assert_eq!(ConfigureWeekendsCommand::COMMAND_TYPE, "events.weekend.configure");
    }

    #[test]
    fn create_event_command_into_new_event() {
        let school = SchoolId::from_uuid(Uuid::nil());
        let user = UserId::from_uuid(Uuid::nil());
        let corr = CorrelationId::from_uuid(Uuid::nil());
        let tenant = TenantContext::for_user(school, user, corr, educore_core::tenant::UserType::Teacher);
        let cmd = CreateEventCommand {
            tenant,
            title: "Test".to_owned(),
            from_date: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            to_date: NaiveDate::from_ymd_opt(2026, 6, 2).unwrap(),
            for_whom: ForWhom::All,
            role_ids: vec![],
            url: None,
            location: None,
            description: None,
            image: None,
            rrule: Some(RecurrenceRule::new(RecurrenceFreq::Daily)),
            academic_id: AcademicYearRef::new(school, Uuid::nil()),
        };
        let id = CalendarEventId::new(school, Uuid::nil());
        let new = cmd.into_new_event(id);
        assert_eq!(new.title, "Test");
        assert!(new.rrule.is_some());
    }
}
