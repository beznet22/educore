//! # educore-events-domain child entities
//!
//! Per `docs/specs/events/entities.md`:
//!
//! - [`CalendarEventAudience`] — embedded in `CalendarEvent`
//! - [`CalendarEventAttachment`] — owned by `CalendarEvent`
//! - [`HolidayAttachment`] — owned by `Holiday`
//! - [`HolidayPeriod`] — owned by `Holiday`
//!
//! Note: `AssignIncident` and `IncidentComment` are 1st-class
//! root aggregates (per the spec's 7-root interpretation) and
//! live in `aggregate.rs`, not here.

#![allow(missing_docs, dead_code, clippy::all)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::{Identifier, SchoolId};
use educore_core::value_objects::Timestamp;

use crate::value_objects::{
    CalendarEventAttachmentId, CalendarEventId, ForWhom, HolidayAttachmentId, HolidayId,
    HolidayPeriodId,
};

// =============================================================================
// === CalendarEventAudience section begin (owner: A) ===
// =============================================================================

/// Audience descriptor for a [`CalendarEvent`](crate::aggregate::CalendarEvent).
/// Embedded in the parent aggregate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalendarEventAudience {
    /// The audience scope.
    pub for_whom: ForWhom,
    /// The comma-separated role ids.
    pub role_ids: Vec<String>,
}

impl CalendarEventAudience {
    /// Placeholder constructor.
    #[must_use]
    pub fn new(for_whom: ForWhom, role_ids: Vec<String>) -> Self {
        Self { for_whom, role_ids }
    }
}

// === CalendarEventAudience section end ===

// =============================================================================
// === CalendarEventAttachment section begin (owner: A) ===
// =============================================================================

/// Optional attachment for a [`CalendarEvent`](crate::aggregate::CalendarEvent).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalendarEventAttachment {
    /// The school anchor (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The typed id.
    pub id: CalendarEventAttachmentId,
    /// The parent calendar event id.
    pub calendar_event_id: CalendarEventId,
    /// Optional file reference (image).
    pub file: Option<String>,
    /// Optional URL.
    pub url: Option<String>,
    /// Created at.
    pub created_at: Timestamp,
}

impl CalendarEventAttachment {
    /// Placeholder constructor.
    #[must_use]
    pub fn new(id: CalendarEventAttachmentId, calendar_event_id: CalendarEventId) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            calendar_event_id,
            file: None,
            url: None,
            created_at: Timestamp::now(),
        }
    }
}

// === CalendarEventAttachment section end ===

// =============================================================================
// === HolidayAttachment section begin (owner: B) ===
// =============================================================================

/// Optional attachment for a [`Holiday`](crate::aggregate::Holiday).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HolidayAttachment {
    /// The school anchor (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The typed id.
    pub id: HolidayAttachmentId,
    /// The parent holiday id.
    pub holiday_id: HolidayId,
    /// Optional file reference (image).
    pub file: Option<String>,
    /// Created at.
    pub created_at: Timestamp,
}

impl HolidayAttachment {
    /// Placeholder constructor.
    #[must_use]
    pub fn new(id: HolidayAttachmentId, holiday_id: HolidayId) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            holiday_id,
            file: None,
            created_at: Timestamp::now(),
        }
    }
}

// === HolidayAttachment section end ===

// =============================================================================
// === HolidayPeriod section begin (owner: B) ===
// =============================================================================

/// A single day or sub-range within a [`Holiday`](crate::aggregate::Holiday).
/// Supports split holidays (e.g. "Winter break" with a gap).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HolidayPeriod {
    /// The school anchor (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The typed id.
    pub id: HolidayPeriodId,
    /// The parent holiday id.
    pub holiday_id: HolidayId,
    /// Period start date.
    pub from_date: chrono::NaiveDate,
    /// Period end date.
    pub to_date: chrono::NaiveDate,
}

impl HolidayPeriod {
    /// Placeholder constructor.
    #[must_use]
    pub fn new(id: HolidayPeriodId, holiday_id: HolidayId) -> Self {
        let today = chrono::Utc::now().date_naive();
        Self {
            school_id: id.school_id(),
            id,
            holiday_id,
            from_date: today,
            to_date: today,
        }
    }
}

// === HolidayPeriod section end ===

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audience_constructor() {
        let a = CalendarEventAudience::new(ForWhom::All, vec!["role-1".to_owned()]);
        assert_eq!(a.for_whom, ForWhom::All);
        assert_eq!(a.role_ids.len(), 1);
    }
}
