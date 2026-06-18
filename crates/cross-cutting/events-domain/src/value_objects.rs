//! # educore-events-domain value objects
//!
//! Typed ids, value objects, and closed enums per
//! `docs/specs/events/value-objects.md`.
//!
//! Section markers are placed for each workstream; Wave 1
//! workstreams (A/B/C/D) fill in the VO impls in their
//! assigned range.

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::error::{DomainError, Result};
pub use educore_core::ids::SchoolId;

// =============================================================================
// Macro: typed events-domain id
// =============================================================================

/// Macro to define a per-aggregate typed id wrapper for the
/// events domain. Every id follows the same shape: a
/// `school_id` anchor plus a local `Uuid`.
macro_rules! events_typed_id {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident;
    ) => {
        $(#[$attr])*
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        $vis struct $name {
            /// The owning school (tenant anchor).
            pub school_id: SchoolId,
            /// The local id (UUIDv7).
            pub value: Uuid,
        }

        impl $name {
            /// Constructs a new typed id from its parts.
            #[must_use]
            pub const fn new(school_id: SchoolId, value: Uuid) -> Self {
                Self { school_id, value }
            }
            /// Returns the local UUID.
            #[must_use]
            pub const fn as_uuid(&self) -> Uuid {
                self.value
            }
            /// Returns the owning school id.
            #[must_use]
            pub const fn school_id(&self) -> SchoolId {
                self.school_id
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}/{}", self.school_id, self.value)
            }
        }
    };
}

// =============================================================================
// Typed root ids (7)
// =============================================================================

events_typed_id! {
    /// Typed id for a [`CalendarEvent`](crate::aggregate::CalendarEvent) row.
    pub struct CalendarEventId;
}
events_typed_id! {
    /// Typed id for a [`Holiday`](crate::aggregate::Holiday) row.
    pub struct HolidayId;
}
events_typed_id! {
    /// Typed id for a [`Weekend`](crate::aggregate::Weekend) row.
    pub struct WeekendId;
}
events_typed_id! {
    /// Typed id for an [`Incident`](crate::aggregate::Incident) row.
    pub struct IncidentId;
}
events_typed_id! {
    /// Typed id for an [`AssignIncident`](crate::aggregate::AssignIncident) row.
    pub struct AssignIncidentId;
}
events_typed_id! {
    /// Typed id for an [`IncidentComment`](crate::aggregate::IncidentComment) row.
    pub struct IncidentCommentId;
}
events_typed_id! {
    /// Typed id for a [`CalendarSetting`](crate::aggregate::CalendarSetting) row.
    pub struct CalendarSettingId;
}

// =============================================================================
// Typed child entity ids (3)
// =============================================================================

events_typed_id! {
    /// Typed id for a [`CalendarEventAttachment`](crate::entities::CalendarEventAttachment) child.
    pub struct CalendarEventAttachmentId;
}
events_typed_id! {
    /// Typed id for a [`HolidayAttachment`](crate::entities::HolidayAttachment) child.
    pub struct HolidayAttachmentId;
}
events_typed_id! {
    /// Typed id for a [`HolidayPeriod`](crate::entities::HolidayPeriod) child.
    pub struct HolidayPeriodId;
}

// =============================================================================
// AcademicYearRef — local Uuid newtype (no educore-academic dep)
// =============================================================================

/// Reference to an `academic_years` row. Local `Uuid` newtype
/// to avoid an `educore-academic` dep. The foreign-key
/// relationship is enforced at the storage adapter layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AcademicYearRef {
    /// The school anchor.
    pub school_id: SchoolId,
    /// The academic year id (UUID).
    pub value: Uuid,
}

impl AcademicYearRef {
    /// Constructs a new `AcademicYearRef`.
    #[must_use]
    pub const fn new(school_id: SchoolId, value: Uuid) -> Self {
        Self { school_id, value }
    }
    /// Returns the UUID.
    #[must_use]
    pub const fn as_uuid(&self) -> Uuid {
        self.value
    }
}

impl fmt::Display for AcademicYearRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.school_id, self.value)
    }
}

// =============================================================================
// Closed enums (5)
// =============================================================================

/// Audience scope for a [`CalendarEvent`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ForWhom {
    /// Teacher audience.
    Teacher,
    /// Student audience.
    Student,
    /// Parent audience.
    Parent,
    /// All audiences.
    All,
}

impl ForWhom {
    /// Returns the wire-form string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Teacher => "Teacher",
            Self::Student => "Student",
            Self::Parent => "Parent",
            Self::All => "All",
        }
    }
}

impl fmt::Display for ForWhom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Status of an [`Incident`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IncidentStatus {
    /// Open — just reported.
    Open,
    /// In progress — under investigation.
    InProgress,
    /// Resolved — closed.
    Resolved,
}

impl IncidentStatus {
    /// Returns the wire-form string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::InProgress => "InProgress",
            Self::Resolved => "Resolved",
        }
    }
}

impl fmt::Display for IncidentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Status of a [`CalendarSetting`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CalendarStatus {
    /// Enabled — visible in the calendar UI.
    Enabled,
    /// Disabled — hidden from the calendar UI.
    Disabled,
}

impl CalendarStatus {
    /// Returns the wire-form string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Enabled => "Enabled",
            Self::Disabled => "Disabled",
        }
    }
}

impl fmt::Display for CalendarStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Kind of assignee for an [`AssignIncident`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssignIncidentKind {
    /// Assigned to a student.
    Student,
    /// Assigned to a staff member.
    Staff,
}

impl AssignIncidentKind {
    /// Returns the wire-form string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Student => "Student",
            Self::Staff => "Staff",
        }
    }
}

impl fmt::Display for AssignIncidentKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Status of a [`CalendarEvent`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CalendarEventStatus {
    /// Draft — not yet published.
    Draft,
    /// Published — visible in the calendar.
    Published,
    /// Cancelled — explicitly cancelled.
    Cancelled,
}

impl CalendarEventStatus {
    /// Returns the wire-form string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "Draft",
            Self::Published => "Published",
            Self::Cancelled => "Cancelled",
        }
    }
}

impl fmt::Display for CalendarEventStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// =============================================================================
// Section markers for Wave 1 workstreams
// =============================================================================
//
// Each workstream (A/B/C/D) owns a section in this file and
// adds the VOs for its aggregates. P0 places the section
// markers; workstreams fill in the VOs.

/// CalendarEvent VOs (owner: A)
#[allow(dead_code)]
pub mod calendar_event_vos {
    // EventTitle, EventDescription, EventLocation, EventDate,
    // EventDateRange, RoleIdList, RecurrenceRule, DispatchState
    // — added by Workstream A.
}

/// Holiday VOs (owner: B)
#[allow(dead_code)]
pub mod holiday_vos {
    // HolidayTitle, HolidayDetails — added by Workstream B.
}

/// CalendarSetting VOs (owner: B)
#[allow(dead_code)]
pub mod calendar_setting_vos {
    // CalendarMenuName, CssColor, FontColor, BackgroundColor
    // — added by Workstream B.
}

/// Incident VOs (owner: C)
#[allow(dead_code)]
pub mod incident_vos {
    // IncidentTitle, IncidentDescription, IncidentPoint,
    // IncidentCommentBody — added by Workstream C.
}

/// Weekend VOs (owner: D)
#[allow(dead_code)]
pub mod weekend_vos {
    // WeekendName, WeekendOrder, IsWeekend, WeekendDay
    // — added by Workstream D.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_ids_smoke_test() {
        let school = SchoolId::from_uuid(Uuid::nil());
        let id = CalendarEventId::new(school, Uuid::nil());
        assert_eq!(id.school_id(), school);
        assert_eq!(id.as_uuid(), Uuid::nil());
    }

    #[test]
    fn enums_display() {
        assert_eq!(ForWhom::All.to_string(), "All");
        assert_eq!(IncidentStatus::Open.to_string(), "Open");
        assert_eq!(CalendarStatus::Enabled.to_string(), "Enabled");
        assert_eq!(AssignIncidentKind::Student.to_string(), "Student");
        assert_eq!(CalendarEventStatus::Draft.to_string(), "Draft");
    }
}
