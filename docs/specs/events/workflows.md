# Events Domain — Workflows

Workflows orchestrate commands, queries, and policies to fulfill a
business goal. They are documented as ordered, conditional steps.

## Calendar Event Lifecycle

```text
1. Author drafts a calendar event (CreateEvent).
2. Author updates the title, location, or audience (UpdateEvent).
3. Author or admin deletes the event (DeleteEvent) when superseded.
4. Subscribers (communication domain) dispatch notifications on
   EventCreated.
5. The calendar UI port renders the event on the public or staff
   calendar.
```

**Pre-conditions:**
- The actor has at least one role in the audience.
- The event's date range is valid.

**Failure paths:**
- Invalid date range → `ValidationError::InvalidDateRange`.
- Past date with `for_whom != All` → `ValidationError::PastDateRestricted`.

## Holiday Configuration Workflow

```text
1. SchoolAdmin creates a holiday (CreateHoliday) with a date range.
2. The attendance domain subscribes to HolidayCreated and:
   a. Marks the days as non-instructional.
   b. Skips attendance expectations for the date range.
3. SchoolAdmin updates the holiday (UpdateHoliday) when the date
   range shifts.
4. SchoolAdmin deletes the holiday (DeleteHoliday) when the holiday
   is cancelled.
5. The public calendar UI port renders the holiday on the public
   site.
```

## Weekend Configuration Workflow

```text
1. SchoolAdmin submits a complete list of weekend entries
   (ConfigureWeekends).
2. The system reconciles the list:
   a. New names → WeekendCreated.
   b. Existing names with changed fields → WeekendUpdated.
   c. Names missing from the list → WeekendDeleted.
3. The attendance domain and the public calendar UI port pick up the
   new configuration.
4. Reports on instructional-day count per month respect the new
   configuration.
```

**Edge cases:**
- An empty list disables weekend tracking; the system treats every
  day as instructional.
- A duplicate name in the input is a `ValidationError`.

## Incident Reporting Workflow

```text
1. A teacher or staff member reports an incident (CreateIncident).
2. The incident is in status Open.
3. The teacher or discipline lead assigns the incident to a student
   or staff member (AssignIncident) with a point value.
4. Staff comment on the incident (CommentOnIncident).
5. The discipline lead resolves the incident (ResolveIncident):
   a. Status moves to Resolved.
   b. resolution_note is captured.
6. The HR or discipline policy may subscribe to IncidentResolved to
   record a behavior note.
```

**Edge cases:**
- An incident reported with no description is rejected.
- An incident cannot be deleted once assigned; reassignment is the
  correction path.
- A resolved incident is immutable in its body; only comments may be
  added.

## Incident Assignment Workflow

```text
1. The discipline lead opens an incident (CreateIncident).
2. The lead assigns the incident to a student
   (AssignIncident) with a point value.
3. The lead may reassign points (ReassignIncident) when more
   information arrives.
4. The lead may unassign (UnassignIncident) when the assignment is
   found to be incorrect.
5. The student or guardian is notified by the communication domain
   when the assignment is finalized.
```

## Incident Resolution Workflow

```text
1. The discipline lead reviews the incident and its comments.
2. The lead may reassign points or unassign (ReassignIncident /
   UnassignIncident).
3. The lead resolves the incident (ResolveIncident) with a note.
4. The HR domain subscribes and archives the incident for behavior
   records.
5. Reports summarize incidents by type, by student, by date.
```

## Calendar Setting Workflow

```text
1. SchoolAdmin creates a calendar setting (CreateCalendarSetting)
   with menu_name, status, and colors.
2. The setting is enabled (EnableCalendarSetting) and becomes
   available in the calendar UI.
3. The setting is updated (UpdateCalendarSetting) on color changes.
4. The setting is disabled (DisableCalendarSetting) when it is no
   longer needed.
5. The setting is deleted (DeleteCalendarSetting) when removed.
```

## Idempotency

- `CreateEvent` is **not** idempotent on title. Two events with the
  same title are distinct.
- `CreateHoliday` is idempotent on `(school_id, from_date, to_date,
  holiday_title)`. A duplicate is a no-op success.
- `ConfigureWeekends` is idempotent: re-issuing with the same list is
  a no-op.
- `CreateIncident` is **not** idempotent. Each incident is a distinct
  record.
- `AssignIncident` is idempotent on `(incident_id, student_id)` and
  on `(incident_id, user_id)`. A duplicate updates the point value
  (an implicit reassign).

## Audit Requirements

Every state-changing command writes a durable audit record with the
actor, the correlation id, and a hash of the payload. Incident
comments retain author identity; resolution notes are immutable.
