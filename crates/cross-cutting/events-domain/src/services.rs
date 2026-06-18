//! # educore-events-domain service structs
//!
//! Per `docs/specs/events/services.md`. 5 service structs:
//! CalendarService, RecurrenceService, HolidayService,
//! CalendarSettingService, IncidentService, WeekendService.

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

use chrono::{Datelike, NaiveDate};

use crate::aggregate::{Holiday, Incident, Weekend};
use crate::value_objects::{
    apply_holiday_overrides, CalendarEventStatus, ForWhom, IncidentStatus, RecurrenceRule,
    Url,
};

// =============================================================================
// === CalendarService section begin (owner: A) ===
// =============================================================================

/// Pure helpers for the CalendarEvent aggregate.
pub struct CalendarService;

impl CalendarService {
    /// Returns true if the date falls within the event's range.
    #[must_use]
    pub fn in_range(event_from: NaiveDate, event_to: NaiveDate, date: NaiveDate) -> bool {
        date >= event_from && date <= event_to
    }

    /// Returns true if the audience resolves to the actor's roles.
    #[must_use]
    pub fn audience_resolves_to(for_whom: ForWhom, role_ids: &[String], actor_roles: &[String]) -> bool {
        match for_whom {
            ForWhom::All => true,
            _ => {
                if role_ids.is_empty() {
                    return false;
                }
                role_ids.iter().any(|r| actor_roles.contains(r))
            }
        }
    }

    /// Returns true if two date ranges overlap.
    #[must_use]
    pub fn overlaps(a_from: NaiveDate, a_to: NaiveDate, b_from: NaiveDate, b_to: NaiveDate) -> bool {
        a_from <= b_to && b_from <= a_to
    }

    /// Returns true if the event is visible to the actor.
    #[must_use]
    pub fn visible_to(for_whom: ForWhom, role_ids: &[String], actor_roles: &[String]) -> bool {
        Self::audience_resolves_to(for_whom, role_ids, actor_roles)
    }

    /// Returns true if the event is published.
    #[must_use]
    pub fn is_published(status: CalendarEventStatus) -> bool {
        matches!(status, CalendarEventStatus::Published)
    }

    /// Validates a URL (returns Err if invalid).
    pub fn validate_url(s: &str) -> Result<(), String> {
        Url::new(s).map(|_| ()).map_err(|e| e.to_string())
    }
}

// === CalendarService section end ===

// =============================================================================
// === RecurrenceService section begin (owner: A) ===
// =============================================================================

/// RRULE subset (RFC 5545) expansion for CalendarEvent recurrence.
pub struct RecurrenceService;

impl RecurrenceService {
    /// Expands the rule starting from `start`, applying holiday
    /// overrides (dates within any holiday range are excluded).
    /// This is the canonical "what dates does this recurring event
    /// actually fire on?" answer.
    #[must_use]
    pub fn expand(
        rule: &RecurrenceRule,
        start: NaiveDate,
        holiday_ranges: &[(NaiveDate, NaiveDate)],
    ) -> Vec<NaiveDate> {
        let dates = rule.expand(start);
        apply_holiday_overrides(&dates, holiday_ranges)
    }
}

// === RecurrenceService section end ===

// =============================================================================
// === HolidayService section begin (owner: B) ===
// =============================================================================

/// Pure helpers for the Holiday aggregate.
pub struct HolidayService;

impl HolidayService {
    /// Returns true if the date falls within the holiday's range.
    #[must_use]
    pub fn contains(holiday: &Holiday, date: NaiveDate) -> bool {
        date >= holiday.from_date && date <= holiday.to_date
    }

    /// Returns true if two holidays overlap.
    #[must_use]
    pub fn overlaps(a: &Holiday, b: &Holiday) -> bool {
        a.from_date <= b.to_date && b.from_date <= a.to_date
    }

    /// Returns true if the school is in session on the date
    /// (i.e. the date is not a weekend day and not a holiday).
    #[must_use]
    pub fn is_instructional(weekends: &[Weekend], holidays: &[Holiday], date: NaiveDate) -> bool {
        let weekday = date.weekday().num_days_from_monday() as i32;
        let is_weekend_day = weekends
            .iter()
            .any(|w| w.is_weekend && w.order == weekday);
        let is_holiday = holidays.iter().any(|h| Self::contains(h, date));
        !is_weekend_day && !is_holiday
    }

    /// Counts the number of instructional days in a date range.
    #[must_use]
    pub fn instructional_days_in(
        weekends: &[Weekend],
        holidays: &[Holiday],
        from: NaiveDate,
        to: NaiveDate,
    ) -> u32 {
        let mut count = 0u32;
        let mut current = from;
        while current <= to {
            if Self::is_instructional(weekends, holidays, current) {
                count += 1;
            }
            current = current.succ_opt().unwrap_or(current);
        }
        count
    }
}

// === HolidayService section end ===

// =============================================================================
// === CalendarSettingService section begin (owner: B) ===
// =============================================================================

/// Pure helpers for the CalendarSetting aggregate.
pub struct CalendarSettingService;

impl CalendarSettingService {
    /// Returns true if the CSS color string is valid.
    #[must_use]
    pub fn validate_color(c: &str) -> bool {
        if c.is_empty() || c.len() > 32 {
            return false;
        }
        let trimmed = c.trim();
        if trimmed.starts_with('#') {
            let hex = &trimmed[1..];
            (hex.len() == 3 || hex.len() == 6 || hex.len() == 8)
                && hex.chars().all(|c| c.is_ascii_hexdigit())
        } else {
            true
        }
    }

    /// Returns true if the setting with the given menu_name is
    /// visible in the calendar UI.
    #[must_use]
    pub fn visible(setting_status: crate::value_objects::CalendarStatus) -> bool {
        matches!(setting_status, crate::value_objects::CalendarStatus::Enabled)
    }
}

// === CalendarSettingService section end ===

// =============================================================================
// === IncidentService section begin (owner: C) ===
// =============================================================================

/// Pure helpers for the Incident aggregate.
pub struct IncidentService;

impl IncidentService {
    /// Returns the next status for a given action.
    #[must_use]
    pub fn next_status(current: IncidentStatus, action: super::value_objects::IncidentAction) -> IncidentStatus {
        current.next(action)
    }

    /// Returns the total points across all assignments.
    #[must_use]
    pub fn total_points(points: &[i32]) -> i32 {
        points.iter().sum()
    }

    /// Returns true if the incident is resolved.
    #[must_use]
    pub fn is_resolved(incident: &Incident) -> bool {
        incident.status == IncidentStatus::Resolved
    }
}

// === IncidentService section end ===

// =============================================================================
// === WeekendService section begin (owner: D) ===
// =============================================================================

/// A single change action for a weekend entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WeekendChange {
    /// Create a new weekend entry.
    Create { name: String, order: i32, is_weekend: bool },
    /// Update an existing weekend entry.
    Update { name: String, order: i32, is_weekend: bool },
    /// Delete an existing weekend entry.
    Delete { name: String },
}

/// Pure helpers for the Weekend aggregate.
pub struct WeekendService;

impl WeekendService {
    /// Reconciles the current list with the proposed list,
    /// returning a diff of create/update/delete changes.
    /// The output is the list of `WeekendChange` actions that
    /// the command processor applies.
    #[must_use]
    pub fn reconcile(current: &[Weekend], proposed: &[crate::commands::WeekendEntry]) -> Vec<WeekendChange> {
        let mut changes = Vec::new();
        let current_names: std::collections::HashSet<&str> =
            current.iter().map(|w| w.name.as_str()).collect();
        let proposed_names: std::collections::HashSet<&str> =
            proposed.iter().map(|e| e.name.as_str()).collect();

        for entry in proposed {
            match current.iter().find(|w| w.name == entry.name) {
                None => changes.push(WeekendChange::Create {
                    name: entry.name.clone(),
                    order: entry.order,
                    is_weekend: entry.is_weekend,
                }),
                Some(existing) => {
                    if existing.order != entry.order || existing.is_weekend != entry.is_weekend {
                        changes.push(WeekendChange::Update {
                            name: entry.name.clone(),
                            order: entry.order,
                            is_weekend: entry.is_weekend,
                        });
                    }
                }
            }
        }
        for existing in current {
            if !proposed_names.contains(existing.name.as_str()) {
                changes.push(WeekendChange::Delete {
                    name: existing.name.clone(),
                });
            }
        }
        let _ = current_names; // suppress unused
        changes
    }

    /// Returns true if the given date is a weekend day
    /// (matches the configured weekend entries).
    #[must_use]
    pub fn is_weekend(weekends: &[Weekend], date: NaiveDate) -> bool {
        let weekday = date.weekday().num_days_from_monday() as i32;
        weekends.iter().any(|w| w.is_weekend && w.order == weekday)
    }

    /// Returns the weekends sorted by order ascending.
    #[must_use]
    pub fn ordered(weekends: &[Weekend]) -> Vec<&Weekend> {
        let mut sorted: Vec<&Weekend> = weekends.iter().collect();
        sorted.sort_by_key(|w| w.order);
        sorted
    }
}

// === WeekendService section end ===

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::{RecurrenceFreq, IncidentAction, WeekendId};
    use educore_core::ids::{Identifier, SchoolId, UserId, CorrelationId};
    use educore_core::value_objects::{Etag, Timestamp, Version};
    use chrono::Datelike;

    #[test]
    fn calendar_service_in_range() {
        assert!(CalendarService::in_range(
            NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 6, 10).unwrap(),
            NaiveDate::from_ymd_opt(2026, 6, 5).unwrap(),
        ));
        assert!(!CalendarService::in_range(
            NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 6, 10).unwrap(),
            NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
        ));
    }

    #[test]
    fn calendar_service_audience_resolves() {
        assert!(CalendarService::audience_resolves_to(
            ForWhom::All,
            &[],
            &["teacher".to_owned()],
        ));
        assert!(CalendarService::audience_resolves_to(
            ForWhom::Teacher,
            &["teacher".to_owned()],
            &["teacher".to_owned()],
        ));
        assert!(!CalendarService::audience_resolves_to(
            ForWhom::Teacher,
            &["admin".to_owned()],
            &["teacher".to_owned()],
        ));
    }

    #[test]
    fn calendar_service_overlaps() {
        assert!(CalendarService::overlaps(
            NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 6, 10).unwrap(),
            NaiveDate::from_ymd_opt(2026, 6, 5).unwrap(),
            NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
        ));
        assert!(!CalendarService::overlaps(
            NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 6, 5).unwrap(),
            NaiveDate::from_ymd_opt(2026, 6, 10).unwrap(),
            NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
        ));
    }

    #[test]
    fn calendar_service_url_validation() {
        assert!(CalendarService::validate_url("https://example.com").is_ok());
        assert!(CalendarService::validate_url("not-a-url").is_err());
    }

    #[test]
    fn recurrence_service_expand_with_holiday_override() {
        let rule = RecurrenceRule::new(RecurrenceFreq::Daily).with_count(5);
        let start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let holidays = vec![(
            NaiveDate::from_ymd_opt(2026, 6, 3).unwrap(),
            NaiveDate::from_ymd_opt(2026, 6, 3).unwrap(),
        )];
        let dates = RecurrenceService::expand(&rule, start, &holidays);
        assert_eq!(dates.len(), 4);
        assert!(!dates.contains(&NaiveDate::from_ymd_opt(2026, 6, 3).unwrap()));
    }

    #[test]
    fn calendar_setting_service_validate_color() {
        assert!(CalendarSettingService::validate_color("#fff"));
        assert!(CalendarSettingService::validate_color("#ff0000"));
        assert!(CalendarSettingService::validate_color("red"));
        assert!(!CalendarSettingService::validate_color(""));
        assert!(!CalendarSettingService::validate_color("#zz"));
    }

    #[test]
    fn incident_service_next_status() {
        assert_eq!(
            IncidentService::next_status(IncidentStatus::Open, IncidentAction::InProgress),
            IncidentStatus::InProgress
        );
        assert_eq!(
            IncidentService::next_status(IncidentStatus::InProgress, IncidentAction::Resolve),
            IncidentStatus::Resolved
        );
        assert_eq!(
            IncidentService::next_status(IncidentStatus::Resolved, IncidentAction::InProgress),
            IncidentStatus::Resolved
        );
    }

    #[test]
    fn weekend_service_reconcile_diff() {
        use crate::commands::WeekendEntry;
    use crate::value_objects::WeekendId;
    use educore_core::ids::{Identifier, SchoolId, UserId, CorrelationId};
    use educore_core::value_objects::{Etag, Timestamp, Version};
        let current = vec![
            Weekend {
                id: WeekendId::new(SchoolId::from_uuid(uuid::Uuid::nil()), uuid::Uuid::nil()),
                school_id: SchoolId::from_uuid(uuid::Uuid::nil()),
                name: "Saturday".to_owned(),
                order: 5,
                is_weekend: true,
                academic_id: None,
                version: Version::initial(),
                etag: Etag::placeholder(),
                created_at: Timestamp::now(),
                updated_at: Timestamp::now(),
                created_by: UserId::from_uuid(uuid::Uuid::nil()),
                updated_by: UserId::from_uuid(uuid::Uuid::nil()),
                active_status: true,
                last_event_id: None,
                correlation_id: CorrelationId::from_uuid(uuid::Uuid::nil()),
            },
        ];
        let proposed = vec![
            WeekendEntry { name: "Friday".to_owned(), order: 4, is_weekend: true },
        ];
        let changes = WeekendService::reconcile(&current, &proposed);
        assert_eq!(changes.len(), 2);
        let has_create_friday = changes.iter().any(|c| matches!(c, WeekendChange::Create { name, .. } if name == "Friday"));
        let has_delete_saturday = changes.iter().any(|c| matches!(c, WeekendChange::Delete { name } if name == "Saturday"));
        assert!(has_create_friday, "expected Create(Friday) in {changes:?}");
        assert!(has_delete_saturday, "expected Delete(Saturday) in {changes:?}");
    }

    #[test]
    fn weekend_service_ordered() {
        let w1 = Weekend::new(
            WeekendId::new(SchoolId::from_uuid(uuid::Uuid::nil()), uuid::Uuid::nil()),
            "Sat".to_owned(),
            5,
            true,
            None,
            UserId::from_uuid(uuid::Uuid::nil()),
            Timestamp::now(),
        ).unwrap();
        let w2 = Weekend::new(
            WeekendId::new(SchoolId::from_uuid(uuid::Uuid::nil()), uuid::Uuid::nil()),
            "Sun".to_owned(),
            6,
            true,
            None,
            UserId::from_uuid(uuid::Uuid::nil()),
            Timestamp::now(),
        ).unwrap();
        let bindings = vec![w2, w1];
        let ordered = WeekendService::ordered(&bindings);
        assert_eq!(ordered[0].name, "Sat");
        assert_eq!(ordered[1].name, "Sun");
    }
}
