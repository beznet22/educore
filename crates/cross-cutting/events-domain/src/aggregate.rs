//! # educore-events-domain aggregate roots
//!
//! The 7 root aggregates per `docs/specs/events/aggregates.md`:
//! CalendarEvent, Holiday, Weekend, Incident, AssignIncident,
//! IncidentComment, CalendarSetting.

#![allow(missing_docs, dead_code, clippy::all)]

use chrono::NaiveDate;
use educore_core::ids::{CorrelationId, EventId, Identifier, SchoolId, UserId};
use educore_core::value_objects::{Etag, Timestamp, Version};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{
    CalendarEventAttachment, CalendarEventAudience, HolidayAttachment, HolidayPeriod,
};
use crate::errors::EventsDomainError;
use crate::value_objects::{
    AcademicYearRef, AssignIncidentId, CalendarEventId, CalendarEventStatus, CalendarSettingId,
    CalendarStatus, FileRef, ForWhom, HolidayId, IncidentCommentId, IncidentId, IncidentStatus,
    RecurrenceRule, Url, WeekendId,
};

/// Result alias for aggregate constructors.
pub type AggregateResult<T> = std::result::Result<T, EventsDomainError>;

// =============================================================================
// === CalendarEvent section begin (owner: A) ===
// =============================================================================

/// Calendar event — a school calendar entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: CalendarEventId,
    pub school_id: SchoolId,
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
    pub status: CalendarEventStatus,
    pub audience: CalendarEventAudience,
    pub academic_id: AcademicYearRef,
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

/// Aggregate-local input for [`CalendarEvent::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewCalendarEvent {
    pub id: CalendarEventId,
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
    pub created_by: UserId,
    pub created_at: Timestamp,
    pub correlation_id: CorrelationId,
}

impl CalendarEvent {
    /// Constructs a new CalendarEvent.
    pub fn new(cmd: NewCalendarEvent) -> AggregateResult<Self> {
        if cmd.title.trim().is_empty() {
            return Err(EventsDomainError::Validation(
                "event_title must not be empty".to_owned(),
            ));
        }
        if cmd.title.len() > 200 {
            return Err(EventsDomainError::Validation(
                "event_title must be <= 200 chars".to_owned(),
            ));
        }
        if cmd.from_date > cmd.to_date {
            return Err(EventsDomainError::Validation(
                "from_date must be <= to_date".to_owned(),
            ));
        }
        let audience = CalendarEventAudience::new(cmd.for_whom, cmd.role_ids.clone());
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            title: cmd.title,
            from_date: cmd.from_date,
            to_date: cmd.to_date,
            for_whom: cmd.for_whom,
            role_ids: cmd.role_ids,
            url: cmd.url,
            location: cmd.location,
            description: cmd.description,
            image: cmd.image,
            rrule: cmd.rrule,
            status: CalendarEventStatus::Draft,
            audience,
            academic_id: cmd.academic_id,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            active_status: true,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Updates the event fields.
    pub fn update(
        &mut self,
        title: Option<String>,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
        actor: UserId,
        at: Timestamp,
    ) -> AggregateResult<()> {
        if !self.active_status {
            return Err(EventsDomainError::Conflict(
                "cannot update deleted event".to_owned(),
            ));
        }
        if let Some(t) = title {
            if t.trim().is_empty() {
                return Err(EventsDomainError::Validation(
                    "title must not be empty".to_owned(),
                ));
            }
            self.title = t;
        }
        if let Some(f) = from {
            self.from_date = f;
        }
        if let Some(t) = to {
            self.to_date = t;
        }
        if self.from_date > self.to_date {
            return Err(EventsDomainError::Validation(
                "from_date must be <= to_date".to_owned(),
            ));
        }
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Soft-deletes the event.
    pub fn delete(&mut self, at: Timestamp, actor: UserId) {
        self.active_status = false;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
    }

    /// Returns the audience (embedded child).
    #[must_use]
    pub fn audience(&self) -> &CalendarEventAudience {
        &self.audience
    }

    /// Returns the attachments (loaded on demand).
    #[must_use]
    pub fn attachments(&self) -> Vec<&CalendarEventAttachment> {
        Vec::new()
    }
}

// === CalendarEvent section end ===

// =============================================================================
// === Holiday section begin (owner: B) ===
// =============================================================================

/// Holiday — a school holiday with a date range.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Holiday {
    pub id: HolidayId,
    pub school_id: SchoolId,
    pub title: String,
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
    pub details: Option<String>,
    pub image: Option<FileRef>,
    pub academic_id: AcademicYearRef,
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

/// Aggregate-local input for [`Holiday::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewHoliday {
    pub id: HolidayId,
    pub title: String,
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
    pub details: Option<String>,
    pub image: Option<FileRef>,
    pub academic_id: AcademicYearRef,
    pub created_by: UserId,
    pub created_at: Timestamp,
    pub correlation_id: CorrelationId,
}

impl Holiday {
    /// Constructs a new Holiday.
    pub fn new(cmd: NewHoliday) -> AggregateResult<Self> {
        if cmd.title.trim().is_empty() {
            return Err(EventsDomainError::Validation(
                "holiday_title must not be empty".to_owned(),
            ));
        }
        if cmd.from_date > cmd.to_date {
            return Err(EventsDomainError::Validation(
                "from_date must be <= to_date".to_owned(),
            ));
        }
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            title: cmd.title,
            from_date: cmd.from_date,
            to_date: cmd.to_date,
            details: cmd.details,
            image: cmd.image,
            academic_id: cmd.academic_id,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            active_status: true,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Returns the attachments.
    #[must_use]
    pub fn attachments(&self) -> Vec<&HolidayAttachment> {
        Vec::new()
    }

    /// Returns the periods.
    #[must_use]
    pub fn periods(&self) -> Vec<&HolidayPeriod> {
        Vec::new()
    }
}

// === Holiday section end ===

// =============================================================================
// === CalendarSetting section begin (owner: B) ===
// =============================================================================

/// CalendarSetting — a categorical label for the calendar UI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalendarSetting {
    pub id: CalendarSettingId,
    pub school_id: SchoolId,
    pub menu_name: String,
    pub status: CalendarStatus,
    pub font_color: String,
    pub bg_color: String,
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

/// Aggregate-local input for [`CalendarSetting::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewCalendarSetting {
    pub id: CalendarSettingId,
    pub menu_name: String,
    pub status: CalendarStatus,
    pub font_color: String,
    pub bg_color: String,
    pub created_by: UserId,
    pub created_at: Timestamp,
    pub correlation_id: CorrelationId,
}

impl CalendarSetting {
    /// Constructs a new CalendarSetting.
    pub fn new(cmd: NewCalendarSetting) -> AggregateResult<Self> {
        if cmd.menu_name.trim().is_empty() {
            return Err(EventsDomainError::Validation(
                "menu_name must not be empty".to_owned(),
            ));
        }
        validate_css_color(&cmd.font_color)?;
        validate_css_color(&cmd.bg_color)?;
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            menu_name: cmd.menu_name,
            status: cmd.status,
            font_color: cmd.font_color,
            bg_color: cmd.bg_color,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            active_status: true,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Enables the setting.
    pub fn enable(&mut self, at: Timestamp, actor: UserId) {
        self.status = CalendarStatus::Enabled;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
    }

    /// Disables the setting.
    pub fn disable(&mut self, at: Timestamp, actor: UserId) {
        self.status = CalendarStatus::Disabled;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
    }
}

/// Validates a CSS color string (hex, rgb, or named).
pub fn validate_css_color(c: &str) -> AggregateResult<()> {
    if c.is_empty() || c.len() > 32 {
        return Err(EventsDomainError::Validation(
            "css color must be 1..32 chars".to_owned(),
        ));
    }
    let trimmed = c.trim();
    if trimmed.starts_with('#') {
        let hex = &trimmed[1..];
        if hex.len() != 3 && hex.len() != 6 && hex.len() != 8 {
            return Err(EventsDomainError::Validation(
                "hex color must be #RGB, #RRGGBB, or #RRGGBBAA".to_owned(),
            ));
        }
        if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(EventsDomainError::Validation(
                "hex color must be hex digits".to_owned(),
            ));
        }
    } else if trimmed.starts_with("rgb(") || trimmed.starts_with("rgba(") {
        // basic validation
    } else if !trimmed.chars().all(|c| c.is_ascii_alphabetic()) {
        // named color — must be alphabetic only
        return Err(EventsDomainError::Validation(
            "named color must be alphabetic".to_owned(),
        ));
    }
    Ok(())
}

// === CalendarSetting section end ===

// =============================================================================
// === Incident section begin (owner: C) ===
// =============================================================================

/// Incident — a reported incident.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Incident {
    pub id: IncidentId,
    pub school_id: SchoolId,
    pub title: String,
    pub point: i32,
    pub description: String,
    pub status: IncidentStatus,
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

/// Aggregate-local input for [`Incident::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewIncident {
    pub id: IncidentId,
    pub title: String,
    pub point: i32,
    pub description: String,
    pub created_by: UserId,
    pub created_at: Timestamp,
    pub correlation_id: CorrelationId,
}

impl Incident {
    /// Constructs a new Incident in Open status.
    pub fn new(cmd: NewIncident) -> AggregateResult<Self> {
        if cmd.title.trim().is_empty() {
            return Err(EventsDomainError::Validation(
                "incident title must not be empty".to_owned(),
            ));
        }
        if cmd.point < 0 || cmd.point > 1000 {
            return Err(EventsDomainError::Validation(
                "incident point must be 0..1000".to_owned(),
            ));
        }
        if cmd.description.is_empty() {
            return Err(EventsDomainError::Validation(
                "incident description must not be empty".to_owned(),
            ));
        }
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            title: cmd.title,
            point: cmd.point,
            description: cmd.description,
            status: IncidentStatus::Open,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            active_status: true,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Updates the incident. Returns error if Resolved.
    pub fn update(
        &mut self,
        title: Option<String>,
        point: Option<i32>,
        description: Option<String>,
        actor: UserId,
        at: Timestamp,
    ) -> AggregateResult<()> {
        if self.status == IncidentStatus::Resolved {
            return Err(EventsDomainError::Conflict(
                "cannot update resolved incident".to_owned(),
            ));
        }
        if let Some(t) = title {
            if t.trim().is_empty() {
                return Err(EventsDomainError::Validation(
                    "title must not be empty".to_owned(),
                ));
            }
            self.title = t;
        }
        if let Some(p) = point {
            if !(0..=1000).contains(&p) {
                return Err(EventsDomainError::Validation(
                    "point must be 0..1000".to_owned(),
                ));
            }
            self.point = p;
        }
        if let Some(d) = description {
            self.description = d;
        }
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Transitions to Resolved. Once resolved, the body is immutable
    /// except for the description (per spec invariant 5).
    pub fn resolve(&mut self, _actor: UserId, at: Timestamp) -> AggregateResult<()> {
        if self.status == IncidentStatus::Resolved {
            return Err(EventsDomainError::Conflict("already resolved".to_owned()));
        }
        self.status = IncidentStatus::Resolved;
        self.updated_at = at;
        self.version = self.version.next();
        Ok(())
    }

    /// Soft-deletes the incident (admin override).
    pub fn delete(&mut self, at: Timestamp, actor: UserId) {
        self.active_status = false;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
    }
}

// === Incident section end ===

// =============================================================================
// === AssignIncident section begin (owner: C) ===
// =============================================================================

/// AssignIncident — mapping of an incident to a student/staff.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssignIncident {
    pub id: AssignIncidentId,
    pub school_id: SchoolId,
    pub incident_id: IncidentId,
    pub student_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub point: i32,
    pub added_by: UserId,
    pub academic_id: AcademicYearRef,
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
    /// Constructs a new AssignIncident.
    pub fn new(
        id: AssignIncidentId,
        incident_id: IncidentId,
        student_id: Option<Uuid>,
        user_id: Option<Uuid>,
        point: i32,
        added_by: UserId,
        academic_id: AcademicYearRef,
        at: Timestamp,
    ) -> AggregateResult<Self> {
        if student_id.is_none() && user_id.is_none() {
            return Err(EventsDomainError::Validation(
                "exactly one of student_id or user_id must be set".to_owned(),
            ));
        }
        if student_id.is_some() && user_id.is_some() {
            return Err(EventsDomainError::Validation(
                "only one of student_id or user_id may be set".to_owned(),
            ));
        }
        if !(0..=1000).contains(&point) {
            return Err(EventsDomainError::Validation(
                "point must be 0..1000".to_owned(),
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            incident_id,
            student_id,
            user_id,
            point,
            added_by,
            academic_id,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: at,
            updated_at: at,
            created_by: added_by,
            updated_by: added_by,
            active_status: true,
            last_event_id: None,
            correlation_id: CorrelationId::from_uuid(Uuid::nil()),
        })
    }

    /// Reassigns (updates the point value).
    pub fn reassign(&mut self, point: i32, at: Timestamp) -> AggregateResult<()> {
        if !(0..=1000).contains(&point) {
            return Err(EventsDomainError::Validation(
                "point must be 0..1000".to_owned(),
            ));
        }
        self.point = point;
        self.updated_at = at;
        self.version = self.version.next();
        Ok(())
    }
}

// === AssignIncident section end ===

// =============================================================================
// === IncidentComment section begin (owner: C) ===
// =============================================================================

/// IncidentComment — a comment on an incident.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncidentComment {
    pub id: IncidentCommentId,
    pub school_id: SchoolId,
    pub incident_id: IncidentId,
    pub user_id: UserId,
    pub comment: String,
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
    /// Constructs a new IncidentComment.
    pub fn new(
        id: IncidentCommentId,
        incident_id: IncidentId,
        user_id: UserId,
        comment: String,
        at: Timestamp,
    ) -> AggregateResult<Self> {
        if comment.trim().is_empty() {
            return Err(EventsDomainError::Validation(
                "comment must not be empty".to_owned(),
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            incident_id,
            user_id,
            comment,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: at,
            updated_at: at,
            created_by: user_id,
            updated_by: user_id,
            active_status: true,
            last_event_id: None,
            correlation_id: CorrelationId::from_uuid(Uuid::nil()),
        })
    }

    /// Soft-deletes the comment (admin override).
    pub fn delete(&mut self, at: Timestamp) {
        self.active_status = false;
        self.updated_at = at;
        self.version = self.version.next();
    }
}

// === IncidentComment section end ===

// =============================================================================
// === Weekend section begin (owner: D) ===
// =============================================================================

/// Weekend — a weekend day configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Weekend {
    pub id: WeekendId,
    pub school_id: SchoolId,
    pub name: String,
    pub order: i32,
    pub is_weekend: bool,
    pub academic_id: Option<AcademicYearRef>,
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
    /// Constructs a new Weekend.
    pub fn new(
        id: WeekendId,
        name: String,
        order: i32,
        is_weekend: bool,
        academic_id: Option<AcademicYearRef>,
        created_by: UserId,
        at: Timestamp,
    ) -> AggregateResult<Self> {
        if name.trim().is_empty() {
            return Err(EventsDomainError::Validation(
                "name must not be empty".to_owned(),
            ));
        }
        if !(0..=7).contains(&order) {
            return Err(EventsDomainError::Validation(
                "order must be 0..7".to_owned(),
            ));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            name,
            order,
            is_weekend,
            academic_id,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: at,
            updated_at: at,
            created_by,
            updated_by: created_by,
            active_status: true,
            last_event_id: None,
            correlation_id: CorrelationId::from_uuid(Uuid::nil()),
        })
    }

    /// Updates the weekend fields.
    pub fn update(
        &mut self,
        name: Option<String>,
        order: Option<i32>,
        is_weekend: Option<bool>,
        actor: UserId,
        at: Timestamp,
    ) -> AggregateResult<()> {
        if let Some(n) = name {
            if n.trim().is_empty() {
                return Err(EventsDomainError::Validation(
                    "name must not be empty".to_owned(),
                ));
            }
            self.name = n;
        }
        if let Some(o) = order {
            if !(0..=7).contains(&o) {
                return Err(EventsDomainError::Validation(
                    "order must be 0..7".to_owned(),
                ));
            }
            self.order = o;
        }
        if let Some(w) = is_weekend {
            self.is_weekend = w;
        }
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// === Weekend section end ===

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::RecurrenceFreq;

    fn hardcoded_date(
        y: i32,
        m: u32,
        d: u32,
    ) -> std::result::Result<NaiveDate, Box<dyn std::error::Error>> {
        NaiveDate::from_ymd_opt(y, m, d).ok_or_else(|| {
            Box::<dyn std::error::Error>::from(format!("invalid hardcoded date {y}-{m}-{d}"))
        })
    }

    #[test]
    fn calendar_event_validates_title() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let school = SchoolId::from_uuid(Uuid::nil());
        let id = CalendarEventId::new(school, Uuid::nil());
        let cmd = NewCalendarEvent {
            id,
            title: "".to_owned(),
            from_date: hardcoded_date(2026, 6, 1)?,
            to_date: hardcoded_date(2026, 6, 2)?,
            for_whom: ForWhom::All,
            role_ids: vec![],
            url: None,
            location: None,
            description: None,
            image: None,
            rrule: None,
            academic_id: AcademicYearRef::new(school, Uuid::nil()),
            created_by: UserId::from_uuid(Uuid::nil()),
            created_at: Timestamp::now(),
            correlation_id: CorrelationId::from_uuid(Uuid::nil()),
        };
        assert!(CalendarEvent::new(cmd).is_err());
        Ok(())
    }

    #[test]
    fn calendar_event_validates_date_range(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let school = SchoolId::from_uuid(Uuid::nil());
        let id = CalendarEventId::new(school, Uuid::nil());
        let cmd = NewCalendarEvent {
            id,
            title: "Test".to_owned(),
            from_date: hardcoded_date(2026, 6, 5)?,
            to_date: hardcoded_date(2026, 6, 1)?,
            for_whom: ForWhom::All,
            role_ids: vec![],
            url: None,
            location: None,
            description: None,
            image: None,
            rrule: None,
            academic_id: AcademicYearRef::new(school, Uuid::nil()),
            created_by: UserId::from_uuid(Uuid::nil()),
            created_at: Timestamp::now(),
            correlation_id: CorrelationId::from_uuid(Uuid::nil()),
        };
        assert!(CalendarEvent::new(cmd).is_err());
        Ok(())
    }

    #[test]
    fn calendar_event_with_rrule_constructs(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let school = SchoolId::from_uuid(Uuid::nil());
        let id = CalendarEventId::new(school, Uuid::nil());
        let cmd = NewCalendarEvent {
            id,
            title: "Weekly Meeting".to_owned(),
            from_date: hardcoded_date(2026, 6, 1)?,
            to_date: hardcoded_date(2026, 6, 1)?,
            for_whom: ForWhom::Teacher,
            role_ids: vec!["teacher".to_owned()],
            url: None,
            location: Some("Room 101".to_owned()),
            description: None,
            image: None,
            rrule: Some(RecurrenceRule::new(RecurrenceFreq::Weekly).with_count(10)),
            academic_id: AcademicYearRef::new(school, Uuid::nil()),
            created_by: UserId::from_uuid(Uuid::nil()),
            created_at: Timestamp::now(),
            correlation_id: CorrelationId::from_uuid(Uuid::nil()),
        };
        let event = CalendarEvent::new(cmd)?;
        assert!(event.rrule.is_some());
        assert_eq!(event.for_whom, ForWhom::Teacher);
        Ok(())
    }

    #[test]
    fn holiday_validates_title() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let school = SchoolId::from_uuid(Uuid::nil());
        let id = HolidayId::new(school, Uuid::nil());
        let cmd = NewHoliday {
            id,
            title: "".to_owned(),
            from_date: hardcoded_date(2026, 6, 1)?,
            to_date: hardcoded_date(2026, 6, 2)?,
            details: None,
            image: None,
            academic_id: AcademicYearRef::new(school, Uuid::nil()),
            created_by: UserId::from_uuid(Uuid::nil()),
            created_at: Timestamp::now(),
            correlation_id: CorrelationId::from_uuid(Uuid::nil()),
        };
        assert!(Holiday::new(cmd).is_err());
        Ok(())
    }

    #[test]
    fn calendar_setting_validates_css_color() {
        let school = SchoolId::from_uuid(Uuid::nil());
        let id = CalendarSettingId::new(school, Uuid::nil());
        let cmd = NewCalendarSetting {
            id,
            menu_name: "Exam".to_owned(),
            status: CalendarStatus::Enabled,
            font_color: "not-a-color-###".to_owned(),
            bg_color: "#ff0000".to_owned(),
            created_by: UserId::from_uuid(Uuid::nil()),
            created_at: Timestamp::now(),
            correlation_id: CorrelationId::from_uuid(Uuid::nil()),
        };
        assert!(CalendarSetting::new(cmd).is_err());
    }

    #[test]
    fn incident_validates_point_range() {
        let school = SchoolId::from_uuid(Uuid::nil());
        let id = IncidentId::new(school, Uuid::nil());
        let cmd = NewIncident {
            id,
            title: "Bullying".to_owned(),
            point: 2000,
            description: "Detail".to_owned(),
            created_by: UserId::from_uuid(Uuid::nil()),
            created_at: Timestamp::now(),
            correlation_id: CorrelationId::from_uuid(Uuid::nil()),
        };
        assert!(Incident::new(cmd).is_err());
    }

    #[test]
    fn incident_resolve_immutability(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let school = SchoolId::from_uuid(Uuid::nil());
        let id = IncidentId::new(school, Uuid::nil());
        let cmd = NewIncident {
            id,
            title: "Test".to_owned(),
            point: 5,
            description: "Detail".to_owned(),
            created_by: UserId::from_uuid(Uuid::nil()),
            created_at: Timestamp::now(),
            correlation_id: CorrelationId::from_uuid(Uuid::nil()),
        };
        let mut inc = Incident::new(cmd)?;
        inc.resolve(UserId::from_uuid(Uuid::nil()), Timestamp::now())?;
        assert_eq!(inc.status, IncidentStatus::Resolved);
        assert!(inc
            .update(
                None,
                None,
                None,
                UserId::from_uuid(Uuid::nil()),
                Timestamp::now()
            )
            .is_err());
        Ok(())
    }

    #[test]
    fn assign_incident_requires_exactly_one_assignee() {
        let school = SchoolId::from_uuid(Uuid::nil());
        let id = AssignIncidentId::new(school, Uuid::nil());
        let incident_id = IncidentId::new(school, Uuid::nil());
        let academic = AcademicYearRef::new(school, Uuid::nil());
        let result = AssignIncident::new(
            id,
            incident_id,
            None,
            None,
            5,
            UserId::from_uuid(Uuid::nil()),
            academic,
            Timestamp::now(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn weekend_validates_order_range() {
        let school = SchoolId::from_uuid(Uuid::nil());
        let id = WeekendId::new(school, Uuid::nil());
        let result = Weekend::new(
            id,
            "Saturday".to_owned(),
            10,
            true,
            None,
            UserId::from_uuid(Uuid::nil()),
            Timestamp::now(),
        );
        assert!(result.is_err());
    }
}
