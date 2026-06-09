# Events Domain — Aggregates

## CalendarEvent

**Root type:** `CalendarEvent`
**Identity:** `CalendarEventId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Events

### Purpose

A school calendar entry — a one-off event, an exam, a parent meeting,
a staff workshop — with a date range, a location, an audience, and an
optional image and URL.

### Owned Children

- `CalendarEventAudience` — embedded role list and `ForWhom` enum.
- `CalendarEventAttachment` — optional `FileReference` and `Url`.

### Invariants

1. A `CalendarEvent` has a non-empty `event_title`.
2. The `for_whom` field is one of `Teacher`, `Student`, `Parent`,
   `All`. The `role_ids` field is a comma-separated list of role
   identifiers.
3. `from_date` is on or before `to_date`.
4. A `CalendarEvent` is anchored to a school and an academic year.
5. A `CalendarEvent` cannot be deleted if it has been delivered to
   recipients (the audit record remains).

### Commands

- `CreateEvent`
- `UpdateEvent`
- `DeleteEvent`

### Events

- `EventCreated`
- `EventUpdated`
- `EventDeleted`

### Consistency Boundary

All event mutations are serialized through the `CalendarEvent`
aggregate root. A calendar event is loaded by id, mutated in memory,
validated, and persisted with its events in a single transaction.

---

## Holiday

**Root type:** `Holiday`
**Identity:** `HolidayId(SchoolId, Uuid)`

### Purpose

A school holiday with a date range. Holidays are anchored to a school
and an academic year. The attendance domain subscribes to
`HolidayCreated` to mark non-instructional days.

### Invariants

1. A `Holiday` has a non-empty `holiday_title`.
2. `from_date` is on or before `to_date`.
3. A `Holiday` is anchored to a school and an academic year.
4. Holidays are non-overlapping within the same school by convention;
   the engine does not enforce this strictly but the UI may warn.

### Commands

- `CreateHoliday`
- `UpdateHoliday`
- `DeleteHoliday`

### Events

- `HolidayCreated`
- `HolidayUpdated`
- `HolidayDeleted`

---

## Weekend

**Root type:** `Weekend`
**Identity:** `WeekendId(SchoolId, Uuid)`

### Purpose

The ordered list of weekdays that constitute non-instructional days
for the school. The school may have a single weekend (typically
Saturday and Sunday) or a custom configuration (e.g. Friday and
Saturday in some regions).

### Invariants

1. A `Weekend` has a unique `name` within a school.
2. The `order` field is a positive integer; lower orders sort first
   in a UI.
3. The `is_weekend` flag distinguishes weekend days (1) from regular
   days (0). The default is 0.
4. A weekend is anchored to a school and may scope to an academic
   year.

### Commands

- `CreateWeekend`
- `UpdateWeekend`
- `ConfigureWeekends` (batch)
- `DeleteWeekend`

### Events

- `WeekendCreated`
- `WeekendUpdated`
- `WeekendsConfigured`
- `WeekendDeleted`

---

## Incident

**Root type:** `Incident`
**Identity:** `IncidentId(SchoolId, Uuid)`

### Purpose

A reported incident — a behavioral, safety, or operational event
recorded against the school. Incidents may be assigned to students or
staff via the `AssignIncident` aggregate.

### Invariants

1. An `Incident` has a non-empty `title`.
2. The `point` field is an integer used as a discipline or severity
   score; it is non-negative.
3. An `Incident` has a `Status` of `Open`, `InProgress`, or
   `Resolved`.
4. An `Incident` is anchored to a school.
5. An `Incident` is immutable after `Status` is `Resolved` except for
   the `description` field (which may be annotated) and the comments
   list.

### Commands

- `CreateIncident`
- `UpdateIncident`
- `ResolveIncident`
- `DeleteIncident` (admin override; soft delete)

### Events

- `IncidentReported`
- `IncidentUpdated`
- `IncidentResolved`
- `IncidentDeleted`

---

## AssignIncident

**Identity:** `AssignIncidentId(SchoolId, Uuid)`
**Owner:** `Incident`

### Purpose

A mapping of an `Incident` to a specific `StudentId` or staff member
(via the platform's `UserId` or a domain `StaffId`). The assignment
carries the points attributed to the assignee and the assigning user.

### Invariants

1. An `AssignIncident` is unique by `(incident_id, student_id)` or
   `(incident_id, user_id)`.
2. The `point` field is a non-negative integer.
3. The `record_id` references a `StudentRecord` (from the academic
   domain) at the time of the incident.
4. The assignment is anchored to a school and an academic year.

### Commands

- `AssignIncident`
- `ReassignIncident`
- `UnassignIncident`

### Events

- `IncidentAssigned`
- `IncidentReassigned`
- `IncidentUnassigned`

---

## IncidentComment

**Identity:** `IncidentCommentId(SchoolId, Uuid)`
**Owner:** `Incident`

### Purpose

A comment on an `Incident`. Comments are appended by staff for
threaded discussion and resolution context. Each comment is anchored
to a school and an author `UserId`.

### Invariants

1. An `IncidentComment` is uniquely identified within a school.
2. The `comment` field is non-empty.
3. A comment is append-only; edits are not modeled.

### Commands

- `CommentOnIncident`
- `DeleteIncidentComment` (admin override; soft delete)

### Events

- `IncidentCommented`
- `IncidentCommentDeleted`

---

## CalendarSetting

**Root type:** `CalendarSetting`
**Identity:** `CalendarSettingId(SchoolId, Uuid)`

### Purpose

A categorical label for the calendar UI (e.g. "Holiday", "Exam",
"Meeting") with a display color and a `font_color` and `bg_color`.
Settings drive the visual categorization of calendar entries.

### Invariants

1. A `CalendarSetting` has a non-empty `menu_name`.
2. The `status` field is `Enabled` or `Disabled`. Disabled settings
   are not surfaced in the calendar UI.
3. The settings_colors are CSS color strings (hex codes or named settings_colors).
4. A setting is uniquely named within a school.

### Commands

- `CreateCalendarSetting`
- `UpdateCalendarSetting`
- `EnableCalendarSetting`
- `DisableCalendarSetting`
- `DeleteCalendarSetting`

### Events

- `CalendarSettingCreated`
- `CalendarSettingUpdated`
- `CalendarSettingEnabled`
- `CalendarSettingDisabled`
- `CalendarSettingDeleted`
