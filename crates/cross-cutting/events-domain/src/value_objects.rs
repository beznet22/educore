//! # educore-events-domain value objects
//!
//! Typed ids, value objects, and closed enums per
//! `docs/specs/events/value-objects.md`.

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

use std::fmt;

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::error::{DomainError, Result};
pub use educore_core::ids::SchoolId;

// =============================================================================
// Macro: typed events-domain id
// =============================================================================

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
            pub school_id: SchoolId,
            pub value: Uuid,
        }

        impl $name {
            #[must_use]
            pub const fn new(school_id: SchoolId, value: Uuid) -> Self {
                Self { school_id, value }
            }
            #[must_use]
            pub const fn as_uuid(&self) -> Uuid {
                self.value
            }
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

events_typed_id! { pub struct CalendarEventId; }
events_typed_id! { pub struct HolidayId; }
events_typed_id! { pub struct WeekendId; }
events_typed_id! { pub struct IncidentId; }
events_typed_id! { pub struct AssignIncidentId; }
events_typed_id! { pub struct IncidentCommentId; }
events_typed_id! { pub struct CalendarSettingId; }

// =============================================================================
// Typed child entity ids (3)
// =============================================================================

events_typed_id! { pub struct CalendarEventAttachmentId; }
events_typed_id! { pub struct HolidayAttachmentId; }
events_typed_id! { pub struct HolidayPeriodId; }

// =============================================================================
// AcademicYearRef
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AcademicYearRef {
    pub school_id: SchoolId,
    pub value: Uuid,
}

impl AcademicYearRef {
    #[must_use]
    pub const fn new(school_id: SchoolId, value: Uuid) -> Self {
        Self { school_id, value }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ForWhom {
    Teacher,
    Student,
    Parent,
    All,
}

impl ForWhom {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IncidentStatus {
    Open,
    InProgress,
    Resolved,
}

impl IncidentStatus {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::InProgress => "InProgress",
            Self::Resolved => "Resolved",
        }
    }
    /// Returns the next status for a given action.
    #[must_use]
    pub const fn next(self, action: IncidentAction) -> Self {
        match (self, action) {
            (Self::Open, IncidentAction::InProgress) => Self::InProgress,
            (Self::InProgress, IncidentAction::Resolve) => Self::Resolved,
            (current, _) => current,
        }
    }
}

impl fmt::Display for IncidentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Actions that can transition an incident's status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IncidentAction {
    /// Move to InProgress.
    InProgress,
    /// Move to Resolved.
    Resolve,
    /// Re-open a Resolved incident.
    Reopen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CalendarStatus {
    Enabled,
    Disabled,
}

impl CalendarStatus {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssignIncidentKind {
    Student,
    Staff,
}

impl AssignIncidentKind {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CalendarEventStatus {
    Draft,
    Published,
    Cancelled,
}

impl CalendarEventStatus {
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
// RecurrenceRule (RRULE subset per RFC 5545)
// =============================================================================

/// Frequency of recurrence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RecurrenceFreq {
    /// Daily recurrence.
    Daily,
    /// Weekly recurrence.
    Weekly,
    /// Monthly recurrence.
    Monthly,
    /// Yearly recurrence.
    Yearly,
}

impl RecurrenceFreq {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Daily => "DAILY",
            Self::Weekly => "WEEKLY",
            Self::Monthly => "MONTHLY",
            Self::Yearly => "YEARLY",
        }
    }
}

impl fmt::Display for RecurrenceFreq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// RRULE subset (RFC 5545) for CalendarEvent recurrence.
/// Supports FREQ + INTERVAL + COUNT + UNTIL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecurrenceRule {
    /// The frequency.
    pub freq: RecurrenceFreq,
    /// The interval (default 1).
    pub interval: u32,
    /// Optional count (max occurrences).
    pub count: Option<u32>,
    /// Optional until date (inclusive).
    pub until: Option<NaiveDate>,
}

impl RecurrenceRule {
    /// Constructs a new RecurrenceRule with the given frequency.
    #[must_use]
    pub fn new(freq: RecurrenceFreq) -> Self {
        Self {
            freq,
            interval: 1,
            count: None,
            until: None,
        }
    }
    /// Sets the interval.
    #[must_use]
    pub fn with_interval(mut self, interval: u32) -> Self {
        self.interval = interval.max(1);
        self
    }
    /// Sets the count.
    #[must_use]
    pub fn with_count(mut self, count: u32) -> Self {
        self.count = Some(count);
        self
    }
    /// Sets the until date.
    #[must_use]
    pub fn with_until(mut self, until: NaiveDate) -> Self {
        self.until = Some(until);
        self
    }
    /// Expands the rule into a list of dates starting from `start`,
    /// capped by the rule's COUNT and UNTIL constraints.
    #[must_use]
    pub fn expand(&self, start: NaiveDate) -> Vec<NaiveDate> {
        let mut dates = Vec::new();
        let interval = self.interval.max(1);
        let mut current = start;
        let max = self.count.unwrap_or(u32::MAX);
        for _ in 0..max {
            if let Some(until) = self.until {
                if current > until {
                    break;
                }
            }
            dates.push(current);
            current = advance(current, self.freq, interval);
            if dates.len() >= 1000 {
                break;
            }
        }
        dates
    }
}

fn advance(d: NaiveDate, freq: RecurrenceFreq, interval: u32) -> NaiveDate {
    let interval = i64::from(interval);
    match freq {
        RecurrenceFreq::Daily => d + chrono::Duration::days(interval),
        RecurrenceFreq::Weekly => d + chrono::Duration::weeks(interval),
        RecurrenceFreq::Monthly => add_months(d, interval),
        RecurrenceFreq::Yearly => add_months(d, interval * 12),
    }
}

fn add_months(d: NaiveDate, months: i64) -> NaiveDate {
    let total = d.year() as i64 * 12 + d.month() as i64 - 1 + months;
    let new_year = (total.div_euclid(12)) as i32;
    let new_month = (total.rem_euclid(12) + 1) as u32;
    let day = d.day().min(days_in_month(new_year, new_month));
    NaiveDate::from_ymd_opt(new_year, new_month, day).unwrap_or(d)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

// =============================================================================
// Holiday override service helper
// =============================================================================

/// Returns the intersection of `event_dates` excluding any date
/// that falls within a holiday. Holidays override recurring events
/// per `docs/specs/events/aggregates.md` CalendarEvent rule 4.
#[must_use]
pub fn apply_holiday_overrides(
    event_dates: &[NaiveDate],
    holiday_ranges: &[(NaiveDate, NaiveDate)],
) -> Vec<NaiveDate> {
    event_dates
        .iter()
        .copied()
        .filter(|d| !holiday_ranges.iter().any(|(from, to)| d >= from && d <= to))
        .collect()
}

// =============================================================================
// File reference placeholder
// =============================================================================

/// Placeholder for a file reference (image upload).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileRef {
    /// The file name.
    pub name: String,
    /// The file content hash.
    pub content_hash: String,
}

impl FileRef {
    /// Constructs a new FileRef.
    #[must_use]
    pub fn new(name: impl Into<String>, content_hash: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            content_hash: content_hash.into(),
        }
    }
}

/// Validated URL string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Url(pub String);

impl Url {
    /// Constructs a new Url, validating basic format.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s = s.into();
        if s.is_empty() || s.len() > 2048 {
            return Err(DomainError::Validation("url must be 1..2048 chars".to_owned()));
        }
        if !(s.starts_with("http://") || s.starts_with("https://")) {
            return Err(DomainError::Validation("url must start with http:// or https://".to_owned()));
        }
        Ok(Self(s))
    }
    /// Returns the URL string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use educore_core::ids::Identifier;

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

    #[test]
    fn rrule_daily_count() {
        let rule = RecurrenceRule::new(RecurrenceFreq::Daily)
            .with_interval(1)
            .with_count(5);
        let start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let dates = rule.expand(start);
        assert_eq!(dates.len(), 5);
        assert_eq!(dates[0], start);
        assert_eq!(dates[4], NaiveDate::from_ymd_opt(2026, 6, 5).unwrap());
    }

    #[test]
    fn rrule_weekly_interval_2() {
        let rule = RecurrenceRule::new(RecurrenceFreq::Weekly)
            .with_interval(2)
            .with_count(4);
        let start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let dates = rule.expand(start);
        assert_eq!(dates.len(), 4);
        assert_eq!(dates[1], NaiveDate::from_ymd_opt(2026, 6, 15).unwrap());
    }

    #[test]
    fn rrule_until_caps_expansion() {
        let rule = RecurrenceRule::new(RecurrenceFreq::Daily)
            .with_interval(1)
            .with_until(NaiveDate::from_ymd_opt(2026, 6, 3).unwrap());
        let start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let dates = rule.expand(start);
        assert_eq!(dates.len(), 3);
        assert_eq!(dates[2], NaiveDate::from_ymd_opt(2026, 6, 3).unwrap());
    }

    #[test]
    fn rrule_monthly() {
        let rule = RecurrenceRule::new(RecurrenceFreq::Monthly)
            .with_interval(1)
            .with_count(3);
        let start = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let dates = rule.expand(start);
        assert_eq!(dates.len(), 3);
        assert_eq!(dates[2], NaiveDate::from_ymd_opt(2026, 3, 15).unwrap());
    }

    #[test]
    fn rrule_yearly() {
        let rule = RecurrenceRule::new(RecurrenceFreq::Yearly)
            .with_interval(1)
            .with_count(2);
        let start = NaiveDate::from_ymd_opt(2026, 7, 4).unwrap();
        let dates = rule.expand(start);
        assert_eq!(dates.len(), 2);
        assert_eq!(dates[1], NaiveDate::from_ymd_opt(2027, 7, 4).unwrap());
    }

    #[test]
    fn holiday_override_excludes_dates() {
        let dates = vec![
            NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 6, 2).unwrap(),
            NaiveDate::from_ymd_opt(2026, 6, 3).unwrap(),
        ];
        let holidays = vec![(
            NaiveDate::from_ymd_opt(2026, 6, 2).unwrap(),
            NaiveDate::from_ymd_opt(2026, 6, 2).unwrap(),
        )];
        let result = apply_holiday_overrides(&dates, &holidays);
        assert_eq!(result.len(), 2);
        assert!(!result.contains(&NaiveDate::from_ymd_opt(2026, 6, 2).unwrap()));
    }

    #[test]
    fn url_validation() {
        assert!(Url::new("https://example.com").is_ok());
        assert!(Url::new("not-a-url").is_err());
        assert!(Url::new("").is_err());
    }

    #[test]
    fn incident_status_state_machine() {
        assert_eq!(
            IncidentStatus::Open.next(IncidentAction::InProgress),
            IncidentStatus::InProgress
        );
        assert_eq!(
            IncidentStatus::InProgress.next(IncidentAction::Resolve),
            IncidentStatus::Resolved
        );
        assert_eq!(
            IncidentStatus::Resolved.next(IncidentAction::InProgress),
            IncidentStatus::Resolved
        );
    }
}
