# Events Domain — Events

Quick reference of every event the events domain emits. Events are
immutable, append-only records. Every event carries a typed
`EventEnvelope` and is durably persisted to the aggregate's event
log.

| Event                              | Aggregate          | Subscribers                                                  | Description                                                                | Durable? | Replicated? | Replayable? |
| ---------------------------------- | ------------------ | ------------------------------------------------------------ | -------------------------------------------------------------------------- | -------- | ----------- | ----------- |
| `EventCreated`                     | `CalendarEvent`    | `communication`, calendar UI port                            | A calendar event was created.                                              | yes      | yes         | yes         |
| `EventUpdated`                     | `CalendarEvent`    | calendar UI port                                             | A calendar event was patched.                                              | yes      | yes         | yes         |
| `EventDeleted`                     | `CalendarEvent`    | calendar UI port                                             | A calendar event was soft-deleted.                                         | yes      | yes         | yes         |
| `HolidayCreated`                   | `Holiday`          | `attendance`                                                 | A holiday was created.                                                     | yes      | yes         | yes         |
| `HolidayUpdated`                   | `Holiday`          | `attendance`                                                 | A holiday was patched.                                                     | yes      | yes         | yes         |
| `HolidayDeleted`                   | `Holiday`          | `attendance`                                                 | A holiday was soft-deleted.                                                | yes      | yes         | yes         |
| `WeekendCreated`                   | `Weekend`          | `attendance`                                                 | A weekend entry was created.                                               | yes      | yes         | yes         |
| `WeekendUpdated`                   | `Weekend`          | `attendance`                                                 | A weekend entry was patched.                                               | yes      | yes         | yes         |
| `WeekendsConfigured`               | `Weekend`          | `attendance`                                                 | The set of weekend entries was replaced.                                   | yes      | yes         | yes         |
| `WeekendDeleted`                   | `Weekend`          | `attendance`                                                 | A weekend entry was deleted.                                               | yes      | yes         | yes         |
| `IncidentReported`                 | `Incident`         | `hr`                                                         | A new incident was reported.                                               | yes      | yes         | yes         |
| `IncidentUpdated`                  | `Incident`         | —                                                            | An open incident was patched.                                              | yes      | yes         | yes         |
| `IncidentResolved`                 | `Incident`         | `hr`                                                         | An incident was resolved.                                                  | yes      | yes         | yes         |
| `IncidentDeleted`                  | `Incident`         | —                                                            | An incident was soft-deleted.                                              | yes      | yes         | yes         |
| `IncidentAssigned`                 | `Incident`         | `hr`                                                         | An incident was assigned to a student or user.                             | yes      | yes         | yes         |
| `IncidentReassigned`               | `Incident`         | `hr`                                                         | An incident assignment's point value was changed.                          | yes      | yes         | yes         |
| `IncidentUnassigned`               | `Incident`         | `hr`                                                         | An incident assignment was removed.                                        | yes      | yes         | yes         |
| `IncidentCommented`                | `Incident`         | `hr`                                                         | A comment was added to an incident.                                         | yes      | yes         | yes         |
| `IncidentCommentDeleted`           | `Incident`         | `hr`                                                         | An incident comment was deleted.                                           | yes      | yes         | yes         |
| `CalendarSettingCreated`           | `CalendarSetting`  | calendar UI port                                             | A calendar menu entry was created.                                          | yes      | yes         | yes         |
| `CalendarSettingUpdated`           | `CalendarSetting`  | calendar UI port                                             | A calendar setting was patched.                                            | yes      | yes         | yes         |
| `CalendarSettingEnabled`           | `CalendarSetting`  | calendar UI port                                             | A calendar setting was enabled.                                            | yes      | yes         | yes         |
| `CalendarSettingDisabled`          | `CalendarSetting`  | calendar UI port                                             | A calendar setting was disabled.                                           | yes      | yes         | yes         |
| `CalendarSettingDeleted`           | `CalendarSetting`  | —                                                            | A calendar setting was soft-deleted.                                       | yes      | yes         | yes         |

**See also:** `docs/specs/events/events.md` for full Rust struct
definitions, the canonical `EventEnvelope`, and per-event
subscribers.
