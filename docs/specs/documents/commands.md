# Documents Domain — Commands

Commands describe intent. They are validated, authorized, and dispatched
to the relevant aggregate. Every command produces zero or more events
that are recorded in the event log.

All commands carry a `TenantContext` (school + actor + correlation) and
are rejected if the actor lacks the required capability.

## UploadForm

```rust
pub struct UploadFormCommand {
    pub tenant: TenantContext,
    pub title: FormTitle,
    pub short_description: Option<FormDescription>,
    pub publish_date: PublishDate,
    pub link: Option<Url>,
    pub file: Option<FileReference>,
    pub show_public: ShowPublic,
}
```

**Capability:** `Form.Upload`
**Pre-conditions:** At least one of `link` or `file` is set.
**Effects:** Creates a `FormDownload` and emits `FormUploaded`.

## UpdateForm

```rust
pub struct UpdateFormCommand {
    pub tenant: TenantContext,
    pub form_id: FormDownloadId,
    pub title: Option<FormTitle>,
    pub short_description: Option<FormDescription>,
    pub publish_date: Option<PublishDate>,
    pub link: Option<Url>,
    pub file: Option<FileReference>,
    pub show_public: Option<ShowPublic>,
}
```

**Capability:** `Form.Update`
**Pre-conditions:** Form exists and is not soft-deleted.
**Effects:** Emits `FormUpdated`.

## DeleteForm

```rust
pub struct DeleteFormCommand {
    pub tenant: TenantContext,
    pub form_id: FormDownloadId,
}
```

**Capability:** `Form.Delete`
**Effects:** Soft-deletes the form; emits `FormDeleted`. The audit
record remains.

## DispatchPostal

```rust
pub struct DispatchPostalCommand {
    pub tenant: TenantContext,
    pub to_title: ToTitle,
    pub from_title: FromTitle,
    pub reference_no: Option<PostalReferenceNo>,
    pub address: ToAddress,
    pub date: DispatchDate,
    pub note: Option<PostalNote>,
    pub file: Option<FileReference>,
}
```

**Capability:** `Postal.Dispatch`
**Pre-conditions:** When `reference_no` is set, it is unique within
`(school_id, academic_id)`.
**Effects:** Creates a `PostalDispatch` and emits `PostalDispatched`.

## UpdatePostalDispatch

```rust
pub struct UpdatePostalDispatchCommand {
    pub tenant: TenantContext,
    pub postal_dispatch_id: PostalDispatchId,
    pub to_title: Option<ToTitle>,
    pub from_title: Option<FromTitle>,
    pub address: Option<ToAddress>,
    pub date: Option<DispatchDate>,
    pub note: Option<PostalNote>,
    pub file: Option<FileReference>,
}
```

**Capability:** `Postal.Update`
**Pre-conditions:** Dispatch exists and is not soft-deleted.
**Effects:** Emits `PostalDispatchUpdated`. The `reference_no` is
immutable and may not be updated here.

## DeletePostalDispatch

```rust
pub struct DeletePostalDispatchCommand {
    pub tenant: TenantContext,
    pub postal_dispatch_id: PostalDispatchId,
}
```

**Capability:** `Postal.Delete`
**Effects:** Soft-deletes the dispatch; emits `PostalDispatchDeleted`.
The audit record remains.

## ReceivePostal

```rust
pub struct ReceivePostalCommand {
    pub tenant: TenantContext,
    pub from_title: FromTitle,
    pub to_title: ToTitle,
    pub reference_no: Option<PostalReferenceNo>,
    pub address: FromAddress,
    pub date: ReceiveDate,
    pub note: Option<PostalNote>,
    pub file: Option<FileReference>,
}
```

**Capability:** `Postal.Receive`
**Pre-conditions:** When `reference_no` is set, it is unique within
`(school_id, academic_id)`.
**Effects:** Creates a `PostalReceive` and emits `PostalReceived`.

## UpdatePostalReceive

```rust
pub struct UpdatePostalReceiveCommand {
    pub tenant: TenantContext,
    pub postal_receive_id: PostalReceiveId,
    pub from_title: Option<FromTitle>,
    pub to_title: Option<ToTitle>,
    pub address: Option<FromAddress>,
    pub date: Option<ReceiveDate>,
    pub note: Option<PostalNote>,
    pub file: Option<FileReference>,
}
```

**Capability:** `Postal.Update`
**Pre-conditions:** Receive exists and is not soft-deleted.
**Effects:** Emits `PostalReceiveUpdated`. The `reference_no` is
immutable.

## DeletePostalReceive

```rust
pub struct DeletePostalReceiveCommand {
    pub tenant: TenantContext,
    pub postal_receive_id: PostalReceiveId,
}
```

**Capability:** `Postal.Delete`
**Effects:** Soft-deletes the receive; emits `PostalReceiveDeleted`.

## TrackPostal

```rust
pub struct TrackPostalCommand {
    pub tenant: TenantContext,
    pub reference_no: PostalReferenceNo,
}
```

**Capability:** `Postal.Read`
**Effects:** Read-only query that surfaces a list of dispatch and
receive records matching the reference number within the school. This
is a query command and does not produce a domain event.
