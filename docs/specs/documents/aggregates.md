# Documents Domain — Aggregates

## FormDownload

**Root type:** `FormDownload`
**Identity:** `FormDownloadId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Documents

### Purpose

A downloadable form published by the school for parents, students, or
staff. The form has a title, a short description, a publish date, an
optional URL, an optional file, and a public-visibility flag.

### Owned Children

- `FormDownloadFile` — optional `FileReference` for the form file.
- `FormDownloadLink` — optional `Url` for an external resource.

### Invariants

1. A `FormDownload` has a non-empty `title`.
2. A `FormDownload` has at least one of `link` or `file` set.
3. A `FormDownload` may be flagged `show_public = false`. Such forms
   are visible only to authenticated staff.
4. A `FormDownload` is never hard-deleted; it is soft-deleted via
   `active_status`.
5. A `FormDownload` is anchored to a school.

### Commands

- `UploadForm`
- `UpdateForm`
- `DeleteForm`

### Events

- `FormUploaded`
- `FormUpdated`
- `FormDeleted`

### Consistency Boundary

All form mutations are serialized through the `FormDownload` aggregate
root. A form is loaded by id, mutated in memory, validated, and
persisted with its events in a single transaction.

---

## PostalDispatch

**Root type:** `PostalDispatch`
**Identity:** `PostalDispatchId(SchoolId, Uuid)`

### Purpose

A postal item dispatched by the school. The dispatch is recorded with
a `to_title`, `from_title`, reference number, address, date, note,
and optional attachment.

### Invariants

1. A `PostalDispatch` has a non-empty `to_title` and `from_title`.
2. The `reference_no` is unique within `(school_id, academic_id)` when
   set.
3. A `PostalDispatch` is anchored to a school and an academic year.
4. The `date` is the dispatch date; it may be in the past for
   back-filling.
5. A `PostalDispatch` is never hard-deleted; it is soft-deleted via
   `active_status`.

### Commands

- `DispatchPostal`
- `UpdatePostalDispatch`
- `DeletePostalDispatch` (admin override; soft delete)

### Events

- `PostalDispatched`
- `PostalDispatchUpdated`
- `PostalDispatchDeleted`

---

## PostalReceive

**Root type:** `PostalReceive`
**Identity:** `PostalReceiveId(SchoolId, Uuid)`

### Purpose

A postal item received by the school. The receive is recorded with a
`from_title`, `to_title`, reference number, address, date, note, and
optional attachment.

### Invariants

1. A `PostalReceive` has a non-empty `from_title` and `to_title`.
2. The `reference_no` is unique within `(school_id, academic_id)` when
   set.
3. A `PostalReceive` is anchored to a school and an academic year.
4. The `date` is the receive date; it may be in the past for
   back-filling.
5. A `PostalReceive` is never hard-deleted; it is soft-deleted via
   `active_status`.

### Commands

- `ReceivePostal`
- `UpdatePostalReceive`
- `DeletePostalReceive` (admin override; soft delete)

### Events

- `PostalReceived`
- `PostalReceiveUpdated`
- `PostalReceiveDeleted`
