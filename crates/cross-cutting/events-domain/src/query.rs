//! # educore-events-domain typed query stubs
//!
//! Per `docs/specs/events/repositories.md`. The events
//! domain ships 7 typed query builders (one per root
//! aggregate). Wave 1 workstreams fill in the query methods.

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

// =============================================================================
// === CalendarEventQuery section begin (owner: A) ===
// =============================================================================

/// Typed query builder for [`CalendarEvent`](crate::aggregate::CalendarEvent).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CalendarEventQuery {
    // Fields filled in by Workstream A.
}

impl CalendarEventQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

// === CalendarEventQuery section end ===

// =============================================================================
// === HolidayQuery section begin (owner: B) ===
// =============================================================================

/// Typed query builder for [`Holiday`](crate::aggregate::Holiday).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HolidayQuery {
    // Fields filled in by Workstream B.
}

impl HolidayQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

// === HolidayQuery section end ===

// =============================================================================
// === CalendarSettingQuery section begin (owner: B) ===
// =============================================================================

/// Typed query builder for [`CalendarSetting`](crate::aggregate::CalendarSetting).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CalendarSettingQuery {
    // Fields filled in by Workstream B.
}

impl CalendarSettingQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

// === CalendarSettingQuery section end ===

// =============================================================================
// === IncidentQuery section begin (owner: C) ===
// =============================================================================

/// Typed query builder for [`Incident`](crate::aggregate::Incident).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct IncidentQuery {
    // Fields filled in by Workstream C.
}

impl IncidentQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

// === IncidentQuery section end ===

// =============================================================================
// === AssignIncidentQuery section begin (owner: C) ===
// =============================================================================

/// Typed query builder for [`AssignIncident`](crate::aggregate::AssignIncident).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AssignIncidentQuery {
    // Fields filled in by Workstream C.
}

impl AssignIncidentQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

// === AssignIncidentQuery section end ===

// =============================================================================
// === IncidentCommentQuery section begin (owner: C) ===
// =============================================================================

/// Typed query builder for [`IncidentComment`](crate::aggregate::IncidentComment).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct IncidentCommentQuery {
    // Fields filled in by Workstream C.
}

impl IncidentCommentQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

// === IncidentCommentQuery section end ===

// =============================================================================
// === WeekendQuery section begin (owner: D) ===
// =============================================================================

/// Typed query builder for [`Weekend`](crate::aggregate::Weekend).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WeekendQuery {
    // Fields filled in by Workstream D.
}

impl WeekendQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

// === WeekendQuery section end ===
