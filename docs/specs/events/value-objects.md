# Events Domain — Value Objects

Value objects are immutable, validated at construction, and have no
identity. They are compared by value.

## Identifiers

All identifiers in the events domain are typed and tenant-scoped. The
generic `Id<S, T>` wrapper carries the `SchoolId` of the owning school
and the local id (`Uuid`).

| Identifier                  | Backing Type           | Source Column                  |
| --------------------------- | ---------------------- | ------------------------------ |
| `CalendarEventId`           | `Id<CalendarEvent>`    | `sm_events.id`                 |
| `HolidayId`                 | `Id<Holiday>`          | `sm_holidays.id`               |
| `WeekendId`                 | `Id<Weekend>`          | `sm_weekends.id`               |
| `IncidentId`                | `Id<Incident>`         | `incidents.id`                 |
| `AssignIncidentId`          | `Id<...>`              | `assign_incidents.id`          |
| `IncidentCommentId`         | `Id<...>`              | `assign_incident_comments.id`  |
| `CalendarSettingId`         | `Id<CalendarSetting>`  | `sm_calendar_settings.id`      |

## Event and Holiday

| Type                  | Constraints                                                       |
| --------------------- | ----------------------------------------------------------------- |
| `EventTitle`          | 1..200 chars                                                      |
| `EventDescription`    | 1..500 chars                                                      |
| `EventLocation`       | 1..200 chars                                                      |
| `EventDateRange`      | `(from: NaiveDate, to: NaiveDate)`, `from ≤ to`                   |
| `ForWhom`             | `Teacher`, `Student`, `Parent`, `All`                             |
| `RoleIdList`          | Comma-separated list of `RoleId` (decoded into `Vec<RoleId>`)     |
| `HolidayTitle`        | 1..200 chars                                                      |
| `HolidayDetails`      | 1..500 chars                                                      |

## Weekend

| Type                  | Constraints                                                       |
| --------------------- | ----------------------------------------------------------------- |
| `WeekendName`         | 1..191 chars, unique within school                                |
| `WeekendOrder`        | `i32` in `0..7`                                                   |
| `IsWeekend`           | `bool` — `true` when the day is a non-instructional day           |
| `WeekendDay`          | Enum of the seven ISO days                                        |

## Incident

| Type                    | Constraints                                                     |
| ----------------------- | --------------------------------------------------------------- |
| `IncidentTitle`         | 1..191 chars                                                    |
| `IncidentDescription`   | 1..5000 chars                                                   |
| `IncidentPoint`         | `i32` in `0..1000`                                              |
| `IncidentStatus`        | `Open`, `InProgress`, `Resolved`                                |
| `IncidentCommentBody`   | 1..5000 chars                                                   |

## Calendar Setting

| Type                  | Constraints                                                       |
| --------------------- | ----------------------------------------------------------------- |
| `CalendarMenuName`    | 1..191 chars, unique within school                                |
| `CalendarStatus`      | `Enabled`, `Disabled`                                             |
| `CssColor`            | 1..32 chars; validated as a CSS color (hex, rgb, or named)        |
| `FontColor`           | `CssColor`                                                        |
| `BackgroundColor`     | `CssColor`                                                        |

## Time and Schedule

| Type                  | Notes                                                              |
| --------------------- | ------------------------------------------------------------------ |
| `EventDate`           | `NaiveDate`                                                        |
| `DateRange`           | `(from: NaiveDate, to: NaiveDate)`, `from ≤ to`                    |
| `AcademicYearId`      | From `smscore-academic`                                            |
| `CreatedByUserId`     | From `smscore-platform`                                            |
| `TenantContext`       | `(SchoolId, UserId, ...)` from `smscore-platform`                 |

## URL and File

| Type                 | Notes                                                          |
| -------------------- | -------------------------------------------------------------- |
| `Url`                | Validated URL, max 2048 chars                                  |
| `FileReference`      | From `smscore-platform`                                        |

## Status Enums

| Type                  | Values                                                              |
| --------------------- | ------------------------------------------------------------------- |
| `CalendarEventStatus` | `Draft`, `Published`, `Cancelled`                                   |
| `IncidentStatus`      | `Open`, `InProgress`, `Resolved`                                    |
| `CalendarStatus`      | `Enabled`, `Disabled`                                               |
| `ForWhom`             | `Teacher`, `Student`, `Parent`, `All`                               |
| `AssignIncidentKind`  | `Student`, `Staff`                                                  |

## Validation Rules

All value objects implement `Validate` and refuse construction when
validation fails:

```rust
pub trait Validate {
    fn validate(&self) -> Result<(), ValueError>;
}
```

Construction is the only entry point:

```rust
let title = IncidentTitle::new("Bullying in classroom 3B")?;
```

Parsing returns `Result<IncidentTitle, ValueError>`. There are no
setters that bypass validation.
