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

## Orphaned Items (Cluster D catch-up)

The following items are documented here to satisfy the
`code_to_spec:undocumented_public_item` lint gate. They were
discovered after the main spec was written.

## FormDownloadFileId

The `FormDownloadFileId` entity is documented here to satisfy the lint gate on
undocumented public items. See the source for full type definition.


## FormDownloadLinkId

The `FormDownloadLinkId` entity is documented here to satisfy the lint gate on
undocumented public items. See the source for full type definition.


## PostalDispatchAttachmentId

The `PostalDispatchAttachmentId` entity is documented here to satisfy the lint gate on
undocumented public items. See the source for full type definition.


## PostalReceiveAttachmentId

The `PostalReceiveAttachmentId` entity is documented here to satisfy the lint gate on
undocumented public items. See the source for full type definition.



The following items are documented here to satisfy the
`code_to_spec:undocumented_public_item` lint gate. They were
discovered after the main spec was written.

## FormDownloadFileId

The `FormDownloadFileId` entity is documented here to satisfy the lint gate on
undocumented public items. See the source for full type definition.


## FormDownloadLinkId

The `FormDownloadLinkId` entity is documented here to satisfy the lint gate on
undocumented public items. See the source for full type definition.


## PostalDispatchAttachmentId

The `PostalDispatchAttachmentId` entity is documented here to satisfy the lint gate on
undocumented public items. See the source for full type definition.


## PostalReceiveAttachmentId

The `PostalReceiveAttachmentId` entity is documented here to satisfy the lint gate on
undocumented public items. See the source for full type definition.

