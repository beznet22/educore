# Events Domain — Commands

Quick reference of every command the events domain exposes. These
commands cover calendar events, holidays, weekend configuration,
incidents, and calendar settings.

The "Events" column lists the events the command emits; consult the
per-domain spec for payload structure.

| Command                  | Capability                  | Description                                                                                          | Events                        | Idempotent? | Offline? |
| ------------------------ | --------------------------- | ---------------------------------------------------------------------------------------------------- | ----------------------------- | ----------- | -------- |
| `CreateEvent`            | `Event.Create`              | Create a calendar event with audience and date range.                                                 | `EventCreated`                | no          | yes      |
| `UpdateEvent`            | `Event.Update`              | Patch a calendar event.                                                                              | `EventUpdated`                | no          | yes      |
| `DeleteEvent`            | `Event.Delete`              | Soft-delete a calendar event.                                                                        | `EventDeleted`                | no          | yes      |
| `CreateHoliday`          | `Holiday.Create`            | Create a holiday with date range.                                                                    | `HolidayCreated`              | no          | yes      |
| `UpdateHoliday`          | `Holiday.Update`            | Patch a holiday.                                                                                     | `HolidayUpdated`              | no          | yes      |
| `DeleteHoliday`          | `Holiday.Delete`            | Soft-delete a holiday.                                                                               | `HolidayDeleted`              | no          | yes      |
| `ConfigureWeekends`      | `Weekend.Configure`         | Replace the school's set of weekend entries with the supplied list.                                  | `WeekendsConfigured`          | yes         | yes      |
| `CreateIncident`         | `Incident.Create`           | Report a new incident.                                                                               | `IncidentReported`            | no          | yes      |
| `UpdateIncident`         | `Incident.Update`           | Patch an open incident.                                                                              | `IncidentUpdated`            | no          | yes      |
| `AssignIncident`         | `Incident.Assign`           | Assign an incident to a student or a user with a point value.                                         | `IncidentAssigned`            | no          | yes      |
| `ReassignIncident`       | `Incident.Reassign`         | Reassign an existing incident assignment with a new point value.                                     | `IncidentReassigned`          | no          | yes      |
| `UnassignIncident`       | `Incident.Unassign`         | Remove an existing incident assignment.                                                              | `IncidentUnassigned`          | no          | yes      |
| `CommentOnIncident`      | `Incident.Comment`          | Add a comment to an incident.                                                                        | `IncidentCommented`           | no          | yes      |
| `ResolveIncident`        | `Incident.Resolve`          | Close an incident with an optional resolution note.                                                  | `IncidentResolved`            | no          | yes      |
| `DeleteIncidentComment`  | `IncidentComment.Delete`    | Soft-delete an incident comment.                                                                     | `IncidentCommentDeleted`      | no          | yes      |
| `CreateCalendarSetting`  | `CalendarSetting.Create`    | Create a calendar menu entry with display styling.                                                   | `CalendarSettingCreated`      | no          | yes      |
| `UpdateCalendarSetting`  | `CalendarSetting.Update`    | Patch a calendar setting.                                                                            | `CalendarSettingUpdated`      | no          | yes      |
| `EnableCalendarSetting`  | `CalendarSetting.Enable`    | Activate a calendar setting.                                                                         | `CalendarSettingEnabled`      | yes         | yes      |
| `DisableCalendarSetting` | `CalendarSetting.Disable`   | Deactivate a calendar setting.                                                                       | `CalendarSettingDisabled`     | yes         | yes      |
| `DeleteCalendarSetting`  | `CalendarSetting.Delete`    | Soft-delete a calendar setting.                                                                      | `CalendarSettingDeleted`      | no          | yes      |

**See also:** `docs/specs/events/commands.md` for full Rust struct
definitions, pre-conditions, effects, and edge-case handling.
