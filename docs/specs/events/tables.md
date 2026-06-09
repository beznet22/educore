# Events Domain — Tables

The events domain is backed by the following tables. Each table maps
to one or more aggregates; the `aggregate` column tells you which
aggregate owns the row.

| Table                              | Aggregate          | Notes                                       |
| ---------------------------------- | ------------------ | ------------------------------------------- |
| `events_calendar_events`                        | CalendarEvent      | School calendar entry                       |
| `events_holidays`                      | Holiday            | School holiday with a date range            |
| `events_weekends`                      | Weekend            | Weekend day configuration                   |
| `incidents`                        | Incident           | Reported incident                           |
| `assign_incidents`                 | AssignIncident     | Incident-to-student or incident-to-staff    |
| `assign_incident_comments`         | IncidentComment    | Comments on an incident                     |
| `events_calendar_settings`             | CalendarSetting    | Calendar UI menu label and color            |

## Notes

- Every school-scoped table includes `school_id` for multi-tenant
  isolation. The `school_id` is `NOT NULL DEFAULT 1` for the bootstrap
  school.
- Every school-scoped table includes `academic_id` referencing
  `academic_academic_years`. The events domain uses `academic_id` to scope
  holidays, weekends, and calendar events.
- Every table includes `created_at`, `updated_at`, `created_by`,
  `updated_by`, `active_status` (where applicable). These are managed
  by the engine's storage adapter.
- The `incidents` and `assign_incident_comments` tables do not include
  `academic_id`; the scope is per-school only. Consumers may add
  `academic_id` for reporting consistency, but the domain does not
  require it.
- The `assign_incidents` table is the join between `incidents` and
  the academic `student_records`. The `record_id` column references
  `student_records.id` and indicates the academic-year scope of the
  assignment.
