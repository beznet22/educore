# Documents Domain — Entities

Entities have identity and lifecycle but are not aggregate roots. They
are loaded and persisted only through their aggregate root.

## FormDownloadFile

**Identity:** `FormDownloadFileId(SchoolId, Uuid)`
**Owner:** `FormDownload`

An optional `FileReference` for the form file. Forms may have at most
one file; the file content is held in the file storage port.

## FormDownloadLink

**Identity:** `FormDownloadLinkId(SchoolId, Uuid)`
**Owner:** `FormDownload`

An optional `Url` for an external resource. Forms may have at most one
link; the URL is held as a value object.

## PostalDispatchAttachment

**Identity:** `PostalDispatchAttachmentId(SchoolId, Uuid)`
**Owner:** `PostalDispatch`

An optional `FileReference` attached to a postal dispatch, typically
a scanned copy of the letter or its envelope.

## PostalReceiveAttachment

**Identity:** `PostalReceiveAttachmentId(SchoolId, Uuid)`
**Owner:** `PostalReceive`

An optional `FileReference` attached to a postal receive, typically
a scanned copy of the letter or its envelope.
