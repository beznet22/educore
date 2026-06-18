//! # educore-events-domain aggregate roots
//!
//! The 7 root aggregates per `docs/specs/events/aggregates.md`:
//!
//! - [`CalendarEvent`] — school calendar entry
//! - [`Holiday`] — school holiday with a date range
//! - [`Weekend`] — weekend day configuration
//! - [`Incident`] — reported incident
//! - [`AssignIncident`] — mapping of an incident to a student/staff
//! - [`IncidentComment`] — comment on an incident
//! - [`CalendarSetting`] — calendar UI menu label and color
//!
//! Each follows the standard 17-field audit-footer pattern
//! (per AGENTS.md). `school_id` is derived from
//! `id.school_id()`, never taken from the caller.

#![allow(missing_docs, dead_code, clippy::all)]

use chrono::{DateTime, NaiveDate, Utc};
use educore_core::ids::{CorrelationId, EventId, Identifier, SchoolId, UserId};
use educore_core::value_objects::{Etag, Timestamp, Version};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::EventsDomainError;
use crate::value_objects::{
    AcademicYearRef, AssignIncidentId, CalendarEventId, CalendarEventStatus, CalendarSettingId,
    HolidayId, IncidentCommentId, IncidentId, IncidentStatus, WeekendId,
};

/// Result alias for aggregate constructors.
pub type AggregateResult<T> = std::result::Result<T, EventsDomainError>;

// =============================================================================
// === CalendarEvent section begin (owner: A) ===
// =============================================================================

/// Calendar event — a school calendar entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalendarEvent {
    /// The typed id.
    pub id: CalendarEventId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// Domain fields — filled in by Workstream A.
    pub title: String,
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
    pub for_whom: crate::value_objects::ForWhom,
    pub academic_id: AcademicYearRef,
    /// Audit-footer fields.
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: bool,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl CalendarEvent {
    /// Placeholder constructor (Phase 13 Workstream A fills in).
    pub fn new(id: CalendarEventId) -> AggregateResult<Self> {
        Ok(Self {
            school_id: id.school_id(),
            id,
            title: String::new(),
            from_date: chrono::Utc::now().date_naive(),
            to_date: chrono::Utc::now().date_naive(),
            for_whom: crate::value_objects::ForWhom::All,
            academic_id: AcademicYearRef::new(id.school_id(), Uuid::nil()),
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            created_by: UserId::from_uuid(Uuid::nil()),
            updated_by: UserId::from_uuid(Uuid::nil()),
            active_status: true,
            last_event_id: None,
            correlation_id: CorrelationId::from_uuid(Uuid::nil()),
        })
    }
}

// === CalendarEvent section end ===

// =============================================================================
// === Holiday section begin (owner: B) ===
// =============================================================================

/// Holiday — a school holiday with a date range.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Holiday {
    /// The typed id.
    pub id: HolidayId,
    /// The owning school.
    pub school_id: SchoolId,
    /// Domain fields — filled in by Workstream B.
    pub title: String,
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
    pub academic_id: AcademicYearRef,
    /// Audit-footer fields.
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: bool,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl Holiday {
    /// Placeholder constructor.
    pub fn new(id: HolidayId) -> AggregateResult<Self> {
        Ok(Self {
            school_id: id.school_id(),
            id,
            title: String::new(),
            from_date: chrono::Utc::now().date_naive(),
            to_date: chrono::Utc::now().date_naive(),
            academic_id: AcademicYearRef::new(id.school_id(), Uuid::nil()),
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            created_by: UserId::from_uuid(Uuid::nil()),
            updated_by: UserId::from_uuid(Uuid::nil()),
            active_status: true,
            last_event_id: None,
            correlation_id: CorrelationId::from_uuid(Uuid::nil()),
        })
    }
}

// === Holiday section end ===

// =============================================================================
// === CalendarSetting section begin (owner: B) ===
// =============================================================================

/// CalendarSetting — a categorical label for the calendar UI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalendarSetting {
    /// The typed id.
    pub id: CalendarSettingId,
    /// The owning school.
    pub school_id: SchoolId,
    /// Domain fields — filled in by Workstream B.
    pub menu_name: String,
    pub status: crate::value_objects::CalendarStatus,
    pub font_color: String,
    pub bg_color: String,
    /// Audit-footer fields.
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: bool,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl CalendarSetting {
    /// Placeholder constructor.
    pub fn new(id: CalendarSettingId) -> AggregateResult<Self> {
        Ok(Self {
            school_id: id.school_id(),
            id,
            menu_name: String::new(),
            status: crate::value_objects::CalendarStatus::Enabled,
            font_color: "#000000".to_owned(),
            bg_color: "#ffffff".to_owned(),
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            created_by: UserId::from_uuid(Uuid::nil()),
            updated_by: UserId::from_uuid(Uuid::nil()),
            active_status: true,
            last_event_id: None,
            correlation_id: CorrelationId::from_uuid(Uuid::nil()),
        })
    }
}

// === CalendarSetting section end ===

// =============================================================================
// === Incident section begin (owner: C) ===
// =============================================================================

/// Incident — a reported incident.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Incident {
    /// The typed id.
    pub id: IncidentId,
    /// The owning school.
    pub school_id: SchoolId,
    /// Domain fields — filled in by Workstream C.
    pub title: String,
    pub point: i32,
    pub description: String,
    pub status: IncidentStatus,
    /// Audit-footer fields.
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: bool,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl Incident {
    /// Placeholder constructor.
    pub fn new(id: IncidentId) -> AggregateResult<Self> {
        Ok(Self {
            school_id: id.school_id(),
            id,
            title: String::new(),
            point: 0,
            description: String::new(),
            status: IncidentStatus::Open,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            created_by: UserId::from_uuid(Uuid::nil()),
            updated_by: UserId::from_uuid(Uuid::nil()),
            active_status: true,
            last_event_id: None,
            correlation_id: CorrelationId::from_uuid(Uuid::nil()),
        })
    }
}

// === Incident section end ===

// =============================================================================
// === AssignIncident section begin (owner: C) ===
// =============================================================================

/// AssignIncident — mapping of an incident to a student/staff.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssignIncident {
    /// The typed id.
    pub id: AssignIncidentId,
    /// The owning school.
    pub school_id: SchoolId,
    /// Domain fields — filled in by Workstream C.
    pub incident_id: IncidentId,
    pub student_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub point: i32,
    pub added_by: UserId,
    pub academic_id: AcademicYearRef,
    /// Audit-footer fields.
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: bool,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl AssignIncident {
    /// Placeholder constructor.
    pub fn new(id: AssignIncidentId) -> AggregateResult<Self> {
        Ok(Self {
            school_id: id.school_id(),
            id,
            incident_id: IncidentId::new(id.school_id(), Uuid::nil()),
            student_id: None,
            user_id: None,
            point: 0,
            added_by: UserId::from_uuid(Uuid::nil()),
            academic_id: AcademicYearRef::new(id.school_id(), Uuid::nil()),
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            created_by: UserId::from_uuid(Uuid::nil()),
            updated_by: UserId::from_uuid(Uuid::nil()),
            active_status: true,
            last_event_id: None,
            correlation_id: CorrelationId::from_uuid(Uuid::nil()),
        })
    }
}

// === AssignIncident section end ===

// =============================================================================
// === IncidentComment section begin (owner: C) ===
// =============================================================================

/// IncidentComment — a comment on an incident.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncidentComment {
    /// The typed id.
    pub id: IncidentCommentId,
    /// The owning school.
    pub school_id: SchoolId,
    /// Domain fields — filled in by Workstream C.
    pub incident_id: IncidentId,
    pub user_id: UserId,
    pub comment: String,
    /// Audit-footer fields.
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: bool,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl IncidentComment {
    /// Placeholder constructor.
    pub fn new(id: IncidentCommentId) -> AggregateResult<Self> {
        Ok(Self {
            school_id: id.school_id(),
            id,
            incident_id: IncidentId::new(id.school_id(), Uuid::nil()),
            user_id: UserId::from_uuid(Uuid::nil()),
            comment: String::new(),
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            created_by: UserId::from_uuid(Uuid::nil()),
            updated_by: UserId::from_uuid(Uuid::nil()),
            active_status: true,
            last_event_id: None,
            correlation_id: CorrelationId::from_uuid(Uuid::nil()),
        })
    }
}

// === IncidentComment section end ===

// =============================================================================
// === Weekend section begin (owner: D) ===
// =============================================================================

/// Weekend — a weekend day configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Weekend {
    /// The typed id.
    pub id: WeekendId,
    /// The owning school.
    pub school_id: SchoolId,
    /// Domain fields — filled in by Workstream D.
    pub name: String,
    pub order: i32,
    pub is_weekend: bool,
    pub academic_id: Option<AcademicYearRef>,
    /// Audit-footer fields.
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: bool,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl Weekend {
    /// Placeholder constructor.
    pub fn new(id: WeekendId) -> AggregateResult<Self> {
        Ok(Self {
            school_id: id.school_id(),
            id,
            name: String::new(),
            order: 0,
            is_weekend: true,
            academic_id: None,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            created_by: UserId::from_uuid(Uuid::nil()),
            updated_by: UserId::from_uuid(Uuid::nil()),
            active_status: true,
            last_event_id: None,
            correlation_id: CorrelationId::from_uuid(Uuid::nil()),
        })
    }
}

// === Weekend section end ===

#[cfg(test)]
mod tests {
    use super::*;
    use educore_core::ids::Identifier;

    #[test]
    fn calendar_event_school_id_derived_from_id() {
        let school = SchoolId::from_uuid(Uuid::nil());
        let id = CalendarEventId::new(school, Uuid::nil());
        let event = CalendarEvent::new(id).unwrap();
        assert_eq!(event.school_id, school);
    }

    #[test]
    fn holiday_school_id_derived_from_id() {
        let school = SchoolId::from_uuid(Uuid::nil());
        let id = HolidayId::new(school, Uuid::nil());
        let h = Holiday::new(id).unwrap();
        assert_eq!(h.school_id, school);
    }

    #[test]
    fn incident_school_id_derived_from_id() {
        let school = SchoolId::from_uuid(Uuid::nil());
        let id = IncidentId::new(school, Uuid::nil());
        let i = Incident::new(id).unwrap();
        assert_eq!(i.school_id, school);
    }

    #[test]
    fn weekend_school_id_derived_from_id() {
        let school = SchoolId::from_uuid(Uuid::nil());
        let id = WeekendId::new(school, Uuid::nil());
        let w = Weekend::new(id).unwrap();
        assert_eq!(w.school_id, school);
    }
}
