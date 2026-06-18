//! # educore-events-domain
//!
//! Calendar events, holidays, weekends, incidents.
//!
//! ## CRITICAL: Two `events` crates — do NOT confuse
//!
//! This crate (`educore-events-domain`) is the **Calendar** domain:
//! it owns the `CalendarEvent`, `Holiday`, `Weekend`, `Incident`,
//! `AssignIncident`, `IncidentComment`, and `CalendarSetting`
//! aggregates. It is a **cross-cutting** domain crate (Phase 13),
//! not a `crates/domains/` crate, per `docs/build-plan.md` §
//! "Phase 13".
//!
//! The other events crate (`educore-events` at
//! `crates/cross-cutting/events/`) is the **envelope** crate
//! (Phase 2): it owns the `DomainEvent` trait, the
//! `EventEnvelope` wire format, and the `EventBus` port. The
//! envelope crate is locked.
//!
//! See `docs/architecture.md` and the domain spec in
//! `docs/specs/events/` for behavioral details.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod aggregate;
pub mod commands;
pub mod entities;
pub mod errors;
pub mod events;
pub mod query;
pub mod repository;
pub mod services;
pub mod value_objects;

/// Package name constant.
pub const PACKAGE_NAME: &str = "educore-events-domain";

/// Package version at compile time.
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Prelude: the public surface of the events-domain crate.
#[allow(missing_docs)]
pub mod prelude {
    pub use crate::aggregate::{
        AssignIncident, CalendarEvent, CalendarSetting, Holiday, Incident, IncidentComment, Weekend,
    };
    pub use crate::entities::{
        CalendarEventAttachment, CalendarEventAudience, HolidayAttachment, HolidayPeriod,
    };
    pub use crate::errors::{EventsDomainError, Result};
    pub use crate::value_objects::{
        AcademicYearRef, AssignIncidentId, AssignIncidentKind, CalendarEventAttachmentId,
        CalendarEventId, CalendarEventStatus, CalendarSettingId, CalendarStatus, ForWhom,
        HolidayAttachmentId, HolidayId, HolidayPeriodId, IncidentCommentId, IncidentId,
        IncidentStatus, WeekendId,
    };
    pub use educore_core::ids::SchoolId;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn package_metadata_is_set() {
        assert_eq!(PACKAGE_NAME, "educore-events-domain");
        assert!(!PACKAGE_VERSION.is_empty());
    }
    #[test]
    fn prelude_exports_expected_symbols() {
        let _: Option<crate::aggregate::CalendarEvent> = None;
        let _: Option<crate::aggregate::Holiday> = None;
        let _: Option<crate::aggregate::Weekend> = None;
        let _: Option<crate::aggregate::Incident> = None;
        let _: Option<crate::aggregate::AssignIncident> = None;
        let _: Option<crate::aggregate::IncidentComment> = None;
        let _: Option<crate::aggregate::CalendarSetting> = None;
    }
}
