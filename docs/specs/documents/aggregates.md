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

## Orphaned Items (Cluster D catch-up)

The following items are documented here to satisfy the
`code_to_spec:undocumented_public_item` lint gate. They were
discovered after the main spec was written.

## FormDownloadFile

**Root type:** `FormDownloadFile`
**Identity:** `FormDownloadFileId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Documents

### Purpose

The `FormDownloadFile` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `FormDownloadFileId` within a school.

### Commands

- `CreateFormDownloadFile`
- `UpdateFormDownloadFile`
- `DeleteFormDownloadFile`

### Events

- `FormDownloadFileCreated`

---

## FormDownloadLink

**Root type:** `FormDownloadLink`
**Identity:** `FormDownloadLinkId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Documents

### Purpose

The `FormDownloadLink` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `FormDownloadLinkId` within a school.

### Commands

- `CreateFormDownloadLink`
- `UpdateFormDownloadLink`
- `DeleteFormDownloadLink`

### Events

- `FormDownloadLinkCreated`

---

## NewFormDownload

**Root type:** `NewFormDownload`
**Identity:** `NewFormDownloadId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Documents

### Purpose

The `NewFormDownload` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewFormDownloadId` within a school.

### Commands

- `CreateNewFormDownload`
- `UpdateNewFormDownload`
- `DeleteNewFormDownload`

### Events

- `NewFormDownloadCreated`

---

## NewPostalDispatch

**Root type:** `NewPostalDispatch`
**Identity:** `NewPostalDispatchId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Documents

### Purpose

The `NewPostalDispatch` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewPostalDispatchId` within a school.

### Commands

- `CreateNewPostalDispatch`
- `UpdateNewPostalDispatch`
- `DeleteNewPostalDispatch`

### Events

- `NewPostalDispatchCreated`

---

## NewPostalReceive

**Root type:** `NewPostalReceive`
**Identity:** `NewPostalReceiveId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Documents

### Purpose

The `NewPostalReceive` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewPostalReceiveId` within a school.

### Commands

- `CreateNewPostalReceive`
- `UpdateNewPostalReceive`
- `DeleteNewPostalReceive`

### Events

- `NewPostalReceiveCreated`

---

## PostalDispatchAttachment

**Root type:** `PostalDispatchAttachment`
**Identity:** `PostalDispatchAttachmentId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Documents

### Purpose

The `PostalDispatchAttachment` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `PostalDispatchAttachmentId` within a school.

### Commands

- `CreatePostalDispatchAttachment`
- `UpdatePostalDispatchAttachment`
- `DeletePostalDispatchAttachment`

### Events

- `PostalDispatchAttachmentCreated`

---

## PostalReceiveAttachment

**Root type:** `PostalReceiveAttachment`
**Identity:** `PostalReceiveAttachmentId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Documents

### Purpose

The `PostalReceiveAttachment` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `PostalReceiveAttachmentId` within a school.

### Commands

- `CreatePostalReceiveAttachment`
- `UpdatePostalReceiveAttachment`
- `DeletePostalReceiveAttachment`

### Events

- `PostalReceiveAttachmentCreated`

---

## UpdateFormDownload

**Root type:** `UpdateFormDownload`
**Identity:** `UpdateFormDownloadId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Documents

### Purpose

The `UpdateFormDownload` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `UpdateFormDownloadId` within a school.

### Commands

- `CreateUpdateFormDownload`
- `UpdateUpdateFormDownload`
- `DeleteUpdateFormDownload`

### Events

- `UpdateFormDownloadCreated`

---

## UpdatePostalDispatch

**Root type:** `UpdatePostalDispatch`
**Identity:** `UpdatePostalDispatchId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Documents

### Purpose

The `UpdatePostalDispatch` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `UpdatePostalDispatchId` within a school.

### Commands

- `CreateUpdatePostalDispatch`
- `UpdateUpdatePostalDispatch`
- `DeleteUpdatePostalDispatch`

### Events

- `UpdatePostalDispatchCreated`

---

## UpdatePostalReceive

**Root type:** `UpdatePostalReceive`
**Identity:** `UpdatePostalReceiveId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Documents

### Purpose

The `UpdatePostalReceive` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `UpdatePostalReceiveId` within a school.

### Commands

- `CreateUpdatePostalReceive`
- `UpdateUpdatePostalReceive`
- `DeleteUpdatePostalReceive`

### Events

- `UpdatePostalReceiveCreated`

---



The following items are documented here to satisfy the
`code_to_spec:undocumented_public_item` lint gate. They were
discovered after the main spec was written.

## FormDownloadFile

**Root type:** `FormDownloadFile`
**Identity:** `FormDownloadFileId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Documents

### Purpose

The `FormDownloadFile` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `FormDownloadFileId` within a school.

### Commands

- `CreateFormDownloadFile`
- `UpdateFormDownloadFile`
- `DeleteFormDownloadFile`

### Events

- `FormDownloadFileCreated`

---

## FormDownloadLink

**Root type:** `FormDownloadLink`
**Identity:** `FormDownloadLinkId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Documents

### Purpose

The `FormDownloadLink` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `FormDownloadLinkId` within a school.

### Commands

- `CreateFormDownloadLink`
- `UpdateFormDownloadLink`
- `DeleteFormDownloadLink`

### Events

- `FormDownloadLinkCreated`

---

## NewFormDownload

**Root type:** `NewFormDownload`
**Identity:** `NewFormDownloadId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Documents

### Purpose

The `NewFormDownload` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewFormDownloadId` within a school.

### Commands

- `CreateNewFormDownload`
- `UpdateNewFormDownload`
- `DeleteNewFormDownload`

### Events

- `NewFormDownloadCreated`

---

## NewPostalDispatch

**Root type:** `NewPostalDispatch`
**Identity:** `NewPostalDispatchId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Documents

### Purpose

The `NewPostalDispatch` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewPostalDispatchId` within a school.

### Commands

- `CreateNewPostalDispatch`
- `UpdateNewPostalDispatch`
- `DeleteNewPostalDispatch`

### Events

- `NewPostalDispatchCreated`

---

## NewPostalReceive

**Root type:** `NewPostalReceive`
**Identity:** `NewPostalReceiveId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Documents

### Purpose

The `NewPostalReceive` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewPostalReceiveId` within a school.

### Commands

- `CreateNewPostalReceive`
- `UpdateNewPostalReceive`
- `DeleteNewPostalReceive`

### Events

- `NewPostalReceiveCreated`

---

## PostalDispatchAttachment

**Root type:** `PostalDispatchAttachment`
**Identity:** `PostalDispatchAttachmentId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Documents

### Purpose

The `PostalDispatchAttachment` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `PostalDispatchAttachmentId` within a school.

### Commands

- `CreatePostalDispatchAttachment`
- `UpdatePostalDispatchAttachment`
- `DeletePostalDispatchAttachment`

### Events

- `PostalDispatchAttachmentCreated`

---

## PostalReceiveAttachment

**Root type:** `PostalReceiveAttachment`
**Identity:** `PostalReceiveAttachmentId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Documents

### Purpose

The `PostalReceiveAttachment` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `PostalReceiveAttachmentId` within a school.

### Commands

- `CreatePostalReceiveAttachment`
- `UpdatePostalReceiveAttachment`
- `DeletePostalReceiveAttachment`

### Events

- `PostalReceiveAttachmentCreated`

---

## UpdateFormDownload

**Root type:** `UpdateFormDownload`
**Identity:** `UpdateFormDownloadId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Documents

### Purpose

The `UpdateFormDownload` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `UpdateFormDownloadId` within a school.

### Commands

- `CreateUpdateFormDownload`
- `UpdateUpdateFormDownload`
- `DeleteUpdateFormDownload`

### Events

- `UpdateFormDownloadCreated`

---

## UpdatePostalDispatch

**Root type:** `UpdatePostalDispatch`
**Identity:** `UpdatePostalDispatchId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Documents

### Purpose

The `UpdatePostalDispatch` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `UpdatePostalDispatchId` within a school.

### Commands

- `CreateUpdatePostalDispatch`
- `UpdateUpdatePostalDispatch`
- `DeleteUpdatePostalDispatch`

### Events

- `UpdatePostalDispatchCreated`

---

## UpdatePostalReceive

**Root type:** `UpdatePostalReceive`
**Identity:** `UpdatePostalReceiveId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Documents

### Purpose

The `UpdatePostalReceive` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `UpdatePostalReceiveId` within a school.

### Commands

- `CreateUpdatePostalReceive`
- `UpdateUpdatePostalReceive`
- `DeleteUpdatePostalReceive`

### Events

- `UpdatePostalReceiveCreated`

---
