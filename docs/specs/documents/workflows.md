# Documents Domain — Workflows

Workflows orchestrate commands, queries, and policies to fulfill a
business goal. They are documented as ordered, conditional steps.

## Form Download Lifecycle

```text
1. SchoolAdmin or reception uploads a form (UploadForm) with a title,
   description, publish date, optional URL, optional file, and a
   public visibility flag.
2. The CMS domain subscribes to FormUploaded and:
   a. Surfaces the form on the public site when show_public = true.
   b. Surfaces the form on the parent portal otherwise.
3. SchoolAdmin updates the form (UpdateForm) when the file changes or
   the publish date moves.
4. SchoolAdmin deletes the form (DeleteForm) when it is no longer
   needed. The audit record remains.
5. Parents, students, or staff download the form via the consumer
   adapter.
```

**Pre-conditions:**
- The form has at least one of `link` or `file` set.
- The publish date is on or after the upload date when required by
  school policy.

**Failure paths:**
- Neither link nor file set → `ValidationError::FormHasNoContent`.
- Storage upload failure → `InfrastructureError::FileStorage`.

## Postal Dispatch Tracking

```text
1. Reception or front-office records a dispatch (DispatchPostal) with
   to_title, from_title, reference number, address, date, note, and
   optional attachment.
2. The communication domain may subscribe and dispatch a
   confirmation to the recipient.
3. Reception updates the dispatch (UpdatePostalDispatch) when the
   address or note changes. The reference number is immutable.
4. Reception deletes the dispatch (DeletePostalDispatch) when the
   record is entered in error. The audit record remains.
5. Reports summarize dispatches by date, by recipient, by reference
   number.
```

## Postal Receive Tracking

```text
1. Reception or front-office records a receive (ReceivePostal) with
   from_title, to_title, reference number, address, date, note, and
   optional attachment.
2. The front-office UI subscribes and surfaces the newly received
   item in the inbox.
3. Reception updates the receive (UpdatePostalReceive) when the
   address or note changes. The reference number is immutable.
4. Reception deletes the receive (DeletePostalReceive) when the
   record is entered in error. The audit record remains.
5. Reports summarize receives by date, by sender, by reference
   number.
```

## Postal Tracking Workflow

```text
1. A staff member queries the tracking command (TrackPostal) with a
   reference number.
2. The system returns the list of matching dispatch and receive
   records within the school.
3. The system surfaces a paired view when both dispatch and receive
   records share a reference number and a date proximity.
4. The system emits no domain event for the read; the read is logged
   in the audit sink.
```

## Idempotency

- `UploadForm` is **not** idempotent on title. Two forms with the
  same title are distinct.
- `DispatchPostal` is idempotent on `(school_id, academic_id,
  reference_no)` when `reference_no` is set. A duplicate is a no-op
  success.
- `ReceivePostal` is idempotent on `(school_id, academic_id,
  reference_no)` when `reference_no` is set. A duplicate is a no-op
  success.
- `UpdateForm` and `UpdatePostalDispatch` and `UpdatePostalReceive`
  are idempotent on the same input.

## Audit Requirements

Every state-changing command writes a durable audit record with the
actor, the correlation id, and a hash of the payload. Soft-deleted
records remain queryable for audit but are excluded from default
listings.

File references are recorded as `FileReference`s, not as raw URLs.
The file storage port enforces the actual access control.
