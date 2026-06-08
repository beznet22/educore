# Documents Domain — Commands

Quick reference of every command the documents domain exposes. These
commands cover downloadable forms, postal dispatch, and postal
receive.

The "Events" column lists the events the command emits; consult the
per-domain spec for payload structure.

| Command                  | Capability            | Description                                                                                       | Events                       | Idempotent? | Offline? |
| ------------------------ | --------------------- | ------------------------------------------------------------------------------------------------- | ---------------------------- | ----------- | -------- |
| `UploadForm`             | `Form.Upload`         | Upload a form (file or link) for public or in-app download.                                        | `FormUploaded`               | no          | no       |
| `UpdateForm`             | `Form.Update`         | Patch a form's title, description, link, or file.                                                 | `FormUpdated`                | no          | no       |
| `DeleteForm`             | `Form.Delete`         | Soft-delete a form.                                                                               | `FormDeleted`                | no          | yes      |
| `DispatchPostal`         | `Postal.Dispatch`     | Create an outbound postal record with recipient and address.                                       | `PostalDispatched`           | no          | yes      |
| `UpdatePostalDispatch`   | `Postal.Update`       | Patch an outbound postal record (reference number is immutable).                                  | `PostalDispatchUpdated`      | no          | yes      |
| `DeletePostalDispatch`   | `Postal.Delete`       | Soft-delete an outbound postal record.                                                            | `PostalDispatchDeleted`      | no          | yes      |
| `ReceivePostal`          | `Postal.Receive`      | Create an inbound postal record with sender and address.                                          | `PostalReceived`             | no          | yes      |
| `UpdatePostalReceive`    | `Postal.Update`       | Patch an inbound postal record (reference number is immutable).                                    | `PostalReceiveUpdated`       | no          | yes      |
| `DeletePostalReceive`    | `Postal.Delete`       | Soft-delete an inbound postal record.                                                             | `PostalReceiveDeleted`       | no          | yes      |
| `TrackPostal`            | `Postal.Read`         | Read-only query that lists dispatches and receives matching a reference number.                   | (none — query only)          | yes         | yes      |

**See also:** `docs/specs/documents/commands.md` for full Rust struct
definitions, pre-conditions, effects, and edge-case handling.
