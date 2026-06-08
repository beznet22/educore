# Events Domain — Business Analysis

## Purpose

The events domain owns the school's calendar: holidays,
school events, and incidents. It is the school's
temporal map: when does the school open, when does it
close, when are the important dates, when did
something notable happen.

This document describes how calendars, holidays, and
incidents work in real schools, with the edge cases
that real schools hit.

## Key Concepts

- **SchoolEvent** — an internal school event (staff
  meeting, exam, parent-teacher conference, etc.).
- **Holiday** — a date on which the school is closed.
- **Incident** — a notable occurrence (accident,
  disciplinary, achievement, etc.).
- **Calendar** — a view of the school year, combining
  holidays, events, and academic dates.
- **EventType** — a category of event (academic,
  administrative, extracurricular, holiday, incident).
- **EventAudience** — the target group (all, staff,
  students, parents, specific class).

## Real-World Scenarios

### School Calendar

A school's calendar is the master timeline for the
academic year. It includes:
- The academic year start and end dates.
- Term start and end dates.
- Holidays (national, regional, school-specific).
- Exam dates.
- Parent-teacher meetings.
- Annual day, sports day, cultural events.
- Staff meetings.
- Vacation periods.

The engine's `SchoolEvent` aggregate captures each
calendar entry. The events are rendered on the
school's calendar UI and on the parent portal.

### Holiday Management

The school admin maintains a list of holidays:

1. National holidays (Republic Day, Independence Day).
2. Regional holidays (state-specific).
3. School-specific holidays (annual day, founder's
   day).
4. Optional holidays (parents may opt their child
   out of certain school-specific observances).

The engine's `Holiday` aggregate captures each
holiday with:
- A title.
- A date.
- A type (`National`, `Regional`, `School`,
  `Optional`).
- A description (optional).

Holidays are visible on the calendar; the
attendance domain auto-marks "Holiday" for the date;
the assessment domain prevents scheduling exams on
holidays.

### School Events

A school has many events during the year:
- Academic: exams, results, parent-teacher meetings.
- Extracurricular: sports day, annual day, debate.
- Administrative: staff meetings, board meetings.

The engine's `SchoolEvent` aggregate captures each
event with:
- A title.
- A description.
- A start date / time.
- An end date / time.
- A location.
- A type.
- An audience.

The events are visible on the calendar. The
communication domain may subscribe and send
reminders.

### Recurring Events

A school has recurring events:
- Weekly staff meeting every Monday at 4pm.
- Monthly parent-teacher meeting on the first
  Saturday.
- Daily morning assembly at 8am.

The engine's `SchoolEvent` supports recurrence
rules (RRULE in iCalendar format). The engine's
projection expands the recurrence into individual
event instances.

### Exam Scheduling

The school schedules exams. The exam dates are
captured in two places:
- The `Exam` aggregate in the assessment domain
  (operational).
- The `SchoolEvent` aggregate in the events domain
  (calendar).

The two are linked; the exam domain's
`ExamScheduled` event drives the creation of the
calendar event.

### Incident Reporting

A school records notable incidents:
- A student is injured on the playground.
- A parent files a complaint about a teacher.
- A staff member is absent without notice.
- A bus is involved in a minor accident.
- A student wins an external competition.

The engine's `Incident` aggregate captures the
incident with:
- A title.
- A description.
- A date / time.
- A location.
- A type (`Injury`, `Disciplinary`, `Achievement`,
  `Operational`, `Other`).
- A severity (`Low`, `Medium`, `High`).
- An involved party (student, staff, parent, visitor).
- An action taken.
- A resolution.

Incidents are auditable. The school's admin reviews
incidents regularly; the engine's analytics surface
patterns (e.g. recurring injuries on the playground).

### Incident Follow-Up

An injury incident triggers a follow-up:
- The school informs the parent.
- The school arranges medical attention.
- The school files an insurance claim.
- The school reviews the playground safety.

The engine's incident workflow tracks the
follow-up tasks. Each task is a
`IncidentFollowUp` entry with a status
(`Open`, `InProgress`, `Completed`).

### Public Calendar

A school may publish a public calendar (for
prospective parents, the community). The engine's
public calendar is a read-only projection of
selected events (no internal staff meetings, no
incident details). The consumer's frontend
renders the public calendar.

### Event Reminders

A school sends reminders for upcoming events:
- "Annual Day is tomorrow."
- "Parent-teacher meeting is on Saturday at 10am."

The engine's notification worker subscribes to
upcoming events and sends reminders. The reminder
lead time is configurable per event type.

## Business Rules

1. A `Holiday` requires a title, a date, and a type.
2. Holidays do not overlap within the same school.
3. A `SchoolEvent` requires a title, a start, an
   end, and a type.
4. A `SchoolEvent`'s end is after the start.
5. An `Incident` requires a title, a description,
   a date / time, and a type.
6. An `Incident`'s severity is one of
   `Low`, `Medium`, `High`.
7. A recurring event has an `RRULE` that the engine
   supports (frequency, interval, count, until,
   byday, etc.).
8. The public calendar shows only events marked
   `is_public = true`. Internal events are
   staff-only.
9. The engine's holiday list is **per-school**.
   A school in one state observes a different
   state holiday than a school in another.
10. The engine's incident log is
    **append-only**. Incidents are not edited;
    corrections are new entries.

## Edge Cases

### Holiday on a Weekend

A national holiday falls on a Sunday. The school
may or may not observe the compensatory day off
(usually the next working day). The admin
configures the holiday dates; the engine
respects them.

### School Event Rescheduled

An annual day is scheduled for December 15. A
national holiday is announced for December 15.
The school reschedules to December 22. The
engine's `SchoolEventUpdated` event captures the
change; the communication domain sends a
rescheduling notice.

### Exam Date Conflicts with Holiday

A school schedules an exam for a date that is
later declared a holiday. The engine's
`ExamRescheduled` event fires; the calendar
updates; the exam is moved to the next working
day.

### Incident Involving a Visitor

A visitor (e.g. a parent's vendor) is injured in
the school. The engine's `Incident` aggregate
supports a "visitor" involvement type. The
school's incident log captures the incident with
the visitor's details.

### Recurring Event Exception

A school has a weekly staff meeting on Mondays.
The school cancels the meeting for one Monday
(e.g. for a national holiday). The engine's
recurring event supports exceptions (the
`EXDATE` in iCalendar).

### Incident Linked to Complaint

A parent files a complaint about a teacher. The
complaint is escalated to an incident. The
engine's `Incident` aggregate links to the
original `Complaint` aggregate (cross-domain
reference).

### Public Calendar with Internal Events

A parent requests that the staff meeting
calendar not be public. The engine's
`is_public` flag is per event; staff meetings
are `is_public = false` by default.

### Calendar Across Time Zones

A school has a campus in one time zone and an
event in another. The engine's `SchoolEvent`
carries a time zone; the consumer's frontend
renders in the user's local time.

### Incident Privacy

An incident involves a minor disciplinary
matter. The school's policy is to keep the
details confidential (no public mention). The
engine's incident log is access-controlled;
the public calendar does not show incidents.

## Notes for SMSengine Implementation

- The **events** crate depends on
  `smscore-core` and `smscore-events`. It does
  not depend on operational domains; it is a
  cross-cutting concern.
- The domain's calendar is a **read model**
  over `Holiday` and `SchoolEvent`
  aggregates. The consumer's frontend renders
  the calendar.
- The domain's holidays feed the
  **attendance** domain's auto-marking. A
  `HolidayObserved` event is consumed by
  the attendance domain.
- The domain's incidents are
  **capability-gated**. The school admin can
  read all incidents; a class teacher can
  read incidents involving their students;
  a parent can read incidents involving
  their child.
- The domain's recurring events use
  **iCalendar RRULE**. The engine's
  projection expands the recurrence into
  individual event instances.
- The domain's public calendar is a
  **projection** with `is_public = true`
  filter. The public-facing API is
  unauthenticated.
- The domain's event reminders are
  **event-driven**. A background worker
  scans upcoming events and sends
  reminders at the configured lead time.
- The domain's incidents are linked to
  other aggregates (complaint, attendance,
  transport). The links are enforced at
  the database level.
