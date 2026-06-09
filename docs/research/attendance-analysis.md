# Attendance Domain — Business Analysis

## Purpose

The attendance domain tracks daily attendance for
students and staff. It is one of the highest-frequency
operational domains: a school with 1,000 students
marks attendance five times a day. The domain must
be fast, simple, and reliable.

This document describes how attendance is taken,
reported, and notified in real schools, with the
edge cases that real schools hit.

## Key Concepts

- **AttendanceSession** — an instance of attendance
  taken for a class-section at a specific date and
  time period.
- **AttendanceEntry** — a single student's status in
  an attendance session. Status: `Present`, `Absent`,
  `Late`, `Leave`, `HalfDay`.
- **StaffAttendance** — a staff member's clock-in /
  clock-out for a day.
- **Holiday** — a date on which the school is closed
  for a holiday. Types: national, regional, school-
  specific, optional.
- **AttendancePolicy** — per-school rules for late,
  half-day, and absence thresholds.

## Real-World Scenarios

### Daily Student Attendance

The teacher takes attendance at the start of each
period (or once at the start of the day, depending
on the school's policy):

1. The teacher opens the attendance screen for
   their class-section.
2. The system shows the roster of students.
3. The teacher marks each student as Present,
   Absent, Late, or on Leave.
4. The teacher submits the attendance.
5. The system records the session and emits
   `AttendanceMarked`.

In real schools, attendance is taken:
- **Once per day** for younger classes.
- **Per period** for older classes (especially in
  high schools where students move between
  classrooms).
- **Biometrically** in some schools (fingerprint
  or face scan). The biometric device is a port-
  driven integration; the engine receives the
  attendance events.

### Late Arrival

A student arrives after the bell. The teacher marks
them as "Late." The engine records the time of
arrival (if captured). The school's late policy
may convert "Late" to "Half-day" after a threshold
(e.g. late by more than 30 minutes). The policy is
configurable per school.

### Leave Application

A student is absent due to a pre-approved leave.
The parent has applied for leave via the parent
portal; the school has approved it. When the
teacher takes attendance, the student is
automatically marked as "Leave." The engine
cross-references with the `LeaveApplication` (in
the academic domain's records or a dedicated
leave domain) and adjusts the status.

### Absent Notification

A student is marked absent. The engine emits
`AttendanceMarked`. The communication domain
subscribes and sends a "Your child is absent
today" SMS / email / push to the parent.

The notification is **batched**: the engine waits
until all classes have been marked, then sends the
notifications once. This avoids spamming parents
with multiple messages.

### Attendance Report

At the end of the month, the school runs an
attendance report:
- Per-student attendance percentage.
- Per-class attendance percentage.
- Students with attendance below threshold
  (e.g. < 75%).
- Consecutive absences (e.g. 3+ days).

The engine's `Report.Generate` command produces
these reports. The report is capability-gated
(class teacher for their classes, principal for
all).

### Staff Attendance

Staff members clock in when they arrive and clock
out when they leave. The engine's `StaffAttendance`
aggregate tracks clock-in, clock-out, working hours,
and overtime.

A real school may have:
- **Biometric clock-in** (fingerprint, face scan).
- **Mobile check-in** (via the staff app).
- **Manual entry** (for staff who forget to clock
  in).

The engine's port receives the clock-in events
from the biometric device or the app. Manual
entry is a command.

### Holiday Handling

A school is closed on a holiday. The engine's
`Holiday` aggregate defines the holiday. The
attendance for that date is automatically
"Holiday" (no session is required). The teacher's
attendance screen shows "Today is a holiday."

### Half-Day

A school has a half-day (e.g. every Saturday, or
on the last day before a break). The engine's
`SchoolCalendar` distinguishes "full day" and
"half day." On a half day, attendance is taken
once; the rest of the day is implicit "Leave" or
"School Over."

### Late Pickup (Day-Care / Kindergarten)

For younger classes, a parent may be late to pick
up. The school records the late pickup. The
engine's `LatePickup` event triggers a fee (per
the school's policy) and a notification.

## Business Rules

1. An attendance session exists for every scheduled
   class meeting, unless the date is a holiday.
2. A student has exactly one status per
   `(class-section, date, period)`.
3. Attendance cannot be marked for a date in the
   future.
4. Attendance can be updated for a past date with
   a `correction_reason`; the update is audited.
5. An absent student triggers a notification (if
   the parent is registered for attendance
   notifications).
6. Staff attendance is independent of student
   attendance. A staff member may be present
   even on a student holiday (e.g. parent-teacher
   meeting day).
7. Holidays are per-school. A school may observe
   a regional holiday that another school does
   not.
8. The attendance percentage is computed as
   `(present + late) / (total_sessions -
   holidays - leave)`. The formula is per-school
   configuration.
9. A student with attendance below a threshold
   (per school policy) is flagged for follow-up.

## Edge Cases

### Mid-Session Correction

A teacher marks a student absent, then realizes
the student is in the classroom. The teacher
corrects the mark with a `correction_reason`.
The engine's audit log captures the change.

### Late Student

A student arrives late. The teacher marks them
as "Late" with a time of arrival. The engine's
"late" status is converted to "Half-day" per the
school's policy (if configured).

### Mass Absence (Bus Breakdown)

The school bus breaks down; 30 students are
absent. The teacher marks all 30 as absent with
a reason "Bus breakdown." The engine emits
`AttendanceMarked`; the communication domain
sends a single "Bus breakdown" notice to all
parents (instead of 30 individual notifications).

### Student with Approved Leave

A student is on a pre-approved leave. The
teacher marks them as "Leave" (or the engine
auto-marks them as "Leave" if the leave is
linked to the attendance system). The
notification to the parent is suppressed
(they already know).

### Student Who Leaves Early

A student leaves school mid-day (e.g. for a
medical appointment). The teacher records the
leave time. The engine's status becomes
"Half-day" for the remaining periods.

### Staff on Multiple Roles

A staff member who is also a parent. The
engine's `TenantContext` resolves the active
role based on the action: a
`StaffAttendance.ClockIn` command activates
the staff role; a `StudentAttendance.Read` for
their child activates the parent role.

### Holiday on a Scheduled Exam Day

A national holiday falls on an exam day. The
school reschedules. The engine's
`ExamRescheduled` event updates the schedule;
the attendance system automatically marks
"Holiday" for the original date.

### Attendance Audit

A parent disputes the attendance. The school's
admin reviews the audit log. The engine's audit
log shows the original entry, any corrections,
and the reasons. The parent sees a summary.

### Multi-Campus Attendance

A school has multiple campuses. A student is
enrolled in the main campus but attends a
workshop at the satellite campus. The engine
supports per-campus attendance via the
`Campus` field on the class-section.

## Notes for SMSengine Implementation

- The **attendance** crate depends on
  `smsengine-academic` for `StudentId`, `ClassId`,
  `SectionId`, `ClassSectionId`, `AcademicYearId`.
- The domain is **high-frequency**. The engine's
  command pipeline must be fast: a single
  `MarkAttendance` command for 30-50 students
  must complete in < 100ms.
- The engine's bulk mark command is
  **all-or-nothing**. A teacher who wants to
  mark 30 students, 28 as present and 2 as
  absent, sends one command with 30 entries.
- The engine's notifications are **batched**. A
  background worker collects all
  `AttendanceMarked` events for the day and
  sends the parent notifications in a single
  digest at the end of the day (or at a
  configured cutoff time).
- The engine's holidays are per-school
  configuration. The engine reads the
  `SchoolCalendar` at command dispatch time.
- The engine's leave integration is via
  cross-domain events. The HR domain's
  `LeaveApplicationApproved` event (for staff
  leave) and a similar academic-domain event
  (for student leave) feed the attendance
  auto-marking.
- The engine's reports are **port-driven**.
  The engine's `Report.Generate` command
  produces capability-gated analytics; the
  consumer chooses the format.
- The domain's events (`AttendanceMarked`,
  `StudentAbsent`, `StaffClockedIn`,
  `StaffClockedOut`, `HolidayObserved`) drive
  downstream projections (daily attendance
  summary, monthly report, parent
  notifications).
