# Events Domain Overview

## Purpose

The events domain owns the school's calendar of activities, the
configuration of holidays and weekends, and the lifecycle of incident
reports. It is the operational time-axis of the school: the domain that
answers "what is happening on date X?" and "what happened with the
incident raised on date Y?".

The domain is intentionally **port-agnostic**. It models calendar
entries, holidays, weekends, and incidents. Surface rendering to web or
mobile is performed by consumer adapters.

## Responsibilities

- Calendar event creation, update, and deletion, with audience scope
  (teachers, students, parents, all).
- Holiday definition, with date ranges and per-year scope.
- Weekend configuration, with an ordered list of days-of-week that
  constitute non-instructional days for the school.
- Calendar UI menu configuration (the categorical labels used by the
  calendar widget, with display settings_colors).
- Incident reporting, with severity scoring, description, and school
  scope.
- Incident assignment of points to students or staff for an incident
  event, with attribution to the assigner.
- Incident comments for threaded discussion among staff.
- Incident resolution, including closure and status transitions.

## Boundaries

The events domain does **not** own:

- Academic scheduling of class periods (see `specs/academic/`).
- Attendance records (see `specs/attendance/`).
- Notifications triggered by events (see `specs/communication/`). The
  events domain emits facts; subscribers dispatch notifications.
- Disciplinary records or behavior tracking as a separate domain. An
  incident is a logged event with optional point assignment; a
  discipline policy is a configuration value managed by the consumer.
- Public calendar publication as a stand-alone domain. Public
  calendars are surfaced by the CMS domain or by a port adapter.

The events domain **does** provide identifier types and value objects
that other domains depend on: `CalendarEventId`, `HolidayId`,
`WeekendId`, `IncidentId`, `AssignIncidentId`, `IncidentCommentId`,
`CalendarSettingId`.

## Dependencies

- `educore-core` — error types, result, identifier trait.
- `educore-platform` — `SchoolId`, `UserId`, `TenantContext`.
- `educore-rbac` — capability checks.
- `educore-events` — domain event publishing.
- `educore-academic` — `ClassId`, `SectionId`, `SubjectId` for audience
  scope on calendar events (read-only references).

## Domain Invariants

1. A `CalendarEvent` belongs to exactly one school and one academic
   year.
2. A `CalendarEvent` may be addressed to one of `Teacher`, `Student`,
   `Parent`, or `All`; the `role_ids` field is a comma-separated list
   of role identifiers used by the notification adapter.
3. A `CalendarEvent`'s `from_date` is on or before `to_date`.
4. A `Holiday` is anchored to a school and an academic year; the
   `from_date` is on or before `to_date`.
5. A `Weekend` is an ordered list of weekdays. Each day appears at
   most once per school.
6. A `Weekend` is uniquely identified by name within a school.
7. An `Incident` is anchored to a school.
8. An `Incident` is assigned to students or staff via `AssignIncident`;
   a single incident may have multiple assignees.
9. An `IncidentComment` belongs to one incident and is anchored to a
   school.
10. An `Incident` has a `Status` of `Open`, `InProgress`, or
    `Resolved`.
11. A `CalendarSetting` defines a categorical label for the calendar
    widget (e.g. "Holiday", "Exam") with a display color. Settings are
    uniquely identified by `menu_name` within a school.

## Aggregate Roots

| Aggregate           | Root Type                | Purpose                                       |
| ------------------- | ------------------------ | --------------------------------------------- |
| CalendarEvent       | `CalendarEvent`          | School calendar entry                         |
| Holiday             | `Holiday`                | A school holiday with a date range            |
| Weekend             | `Weekend`                | A weekend day configuration                   |
| Incident            | `Incident`               | A reported incident                           |
| AssignIncident      | `AssignIncident`         | Mapping of an incident to a student/staff     |
| IncidentComment     | `IncidentComment`        | A comment on an incident                      |
| CalendarSetting     | `CalendarSetting`        | Calendar UI menu label and color              |

Each aggregate is documented in detail under
`docs/specs/events/aggregates.md`.

## Cross-Domain Impact

When a `CalendarEvent` is created, the events domain emits
`EventCreated`. The communication domain may subscribe to dispatch a
notification to the audience.

When a `Holiday` is created or removed, the events domain emits
`HolidayCreated` / `HolidayDeleted`. The attendance domain may
subscribe to mark non-instructional days and exempt students from
attendance requirements on holidays.

When an `Incident` is created, the events domain emits
`IncidentReported`. The HR or discipline policy may subscribe to
record a behavior note against the assigned student or staff.

## Consumers

- Web admin UI (manage events, holidays, incidents).
- Web public site (render the public calendar — typically via the CMS
  domain or a port adapter).
- Mobile parent app (view upcoming events, holidays).
- Mobile teacher app (raise incidents, comment).
- AI agent (raise incidents, resolve incidents, configure weekends).

## Anti-Goals

- The events domain does not present data to humans. It exposes
  commands, events, and queries.
- The events domain does not implement a calendar renderer. Rendering
  is a port adapter.
- The events domain does not decide school policy (e.g. "Saturday is a
  weekend"). That is a configuration value managed by the consumer
  through the `Weekend` aggregate.
- The events domain does not own discipline points. The `point` field
  on `AssignIncident` is a hint captured for reporting; the discipline
  policy lives in the consumer.
