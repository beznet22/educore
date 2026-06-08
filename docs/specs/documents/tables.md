# Documents Domain — Tables

The documents domain is backed by the following tables. Each table
maps to one or more aggregates; the `aggregate` column tells you which
aggregate owns the row.

| Table                       | Aggregate       | Notes                                          |
| --------------------------- | --------------- | ---------------------------------------------- |
| `sm_form_downloads`         | FormDownload    | Downloadable form for parents, students, staff |
| `sm_postal_dispatches`      | PostalDispatch  | Postal item dispatched by the school            |
| `sm_postal_receives`        | PostalReceive   | Postal item received by the school              |

## Notes

- Every school-scoped table includes `school_id` for multi-tenant
  isolation. The `school_id` is `NOT NULL DEFAULT 1` for the bootstrap
  school.
- Every school-scoped table includes `academic_id` referencing
  `sm_academic_years`. The documents domain uses `academic_id` to
  scope postal dispatch and receive.
- Every table includes `created_at`, `updated_at`, `created_by`,
  `updated_by`, `active_status` (where applicable). These are managed
  by the engine's storage adapter.
- The `sm_form_downloads` table does not include `academic_id`; the
  scope is per-school only. Forms are not academic-year-bounded.
- File references (`file` column) and URLs (`link` column) are
  captured as plain strings at the persistence boundary. The domain
  enforces their shape through value objects at construction time.
