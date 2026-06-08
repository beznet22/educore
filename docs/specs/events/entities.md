# Events Domain — Entities

Entities have identity and lifecycle but are not aggregate roots. They
are loaded and persisted only through their aggregate root.

## AssignIncident

**Identity:** `AssignIncidentId(SchoolId, Uuid)`
**Owner:** `Incident`

The mapping of an `Incident` to a specific student or staff member.
Carries the `point` value, the `record_id` (a `StudentRecord` for
students), the `added_by` staff, the `academic_id`, and the
`school_id`. Soft-deletes are not modeled; reassignment is the
correction path.

## IncidentComment

**Identity:** `IncidentCommentId(SchoolId, Uuid)`
**Owner:** `Incident`

A single comment on an `Incident`, with `user_id`, `comment` body,
`incident_id`, and `school_id`. Comments are append-only.

## CalendarEventAudience

**Identity:** Embedded in `CalendarEvent`
**Owner:** `CalendarEvent`

A materialized audience descriptor: a `ForWhom` enum and a
`Vec<RoleId>` rendered as a comma-separated string at the persistence
boundary. The audience is captured at creation time.

## CalendarEventAttachment

**Identity:** `CalendarEventAttachmentId(SchoolId, Uuid)`
**Owner:** `CalendarEvent`

An optional `FileReference` (image) and an optional `Url` attached to
a calendar event.

## HolidayAttachment

**Identity:** `HolidayAttachmentId(SchoolId, Uuid)`
**Owner:** `Holiday`

An optional `FileReference` (image) attached to a holiday.

## HolidayPeriod

**Identity:** `HolidayPeriodId(SchoolId, Uuid)`
**Owner:** `Holiday`

A single day or sub-range within a holiday. Most holidays have one
period equal to the date range; the entity supports split holidays
(e.g. "Winter break" with a gap).
