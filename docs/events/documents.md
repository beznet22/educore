# Documents Domain — Events

Quick reference of every event the documents domain emits. Events
are immutable, append-only records. Every event carries a typed
`EventEnvelope` and is durably persisted to the aggregate's event
log.

| Event                              | Aggregate           | Subscribers                                                | Description                                                                  | Durable? | Replicated? | Replayable? |
| ---------------------------------- | ------------------- | ---------------------------------------------------------- | ---------------------------------------------------------------------------- | -------- | ----------- | ----------- |
| `FormUploaded`                     | `FormDownload`      | `cms` (when public), search-index port                     | A form was uploaded.                                                        | yes      | yes         | yes         |
| `FormUpdated`                      | `FormDownload`      | `cms` (when public), search-index port                     | A form was patched.                                                          | yes      | yes         | yes         |
| `FormDeleted`                      | `FormDownload`      | `cms`                                                      | A form was soft-deleted.                                                     | yes      | yes         | yes         |
| `PostalDispatched`                 | `PostalDispatch`    | `communication`                                            | An outbound postal record was created.                                       | yes      | yes         | yes         |
| `PostalDispatchUpdated`            | `PostalDispatch`    | —                                                          | An outbound postal record was patched.                                       | yes      | yes         | yes         |
| `PostalDispatchDeleted`            | `PostalDispatch`    | —                                                          | An outbound postal record was soft-deleted.                                  | yes      | yes         | yes         |
| `PostalReceived`                   | `PostalReceive`     | —                                                          | An inbound postal record was created.                                        | yes      | yes         | yes         |
| `PostalReceiveUpdated`             | `PostalReceive`     | —                                                          | An inbound postal record was patched.                                        | yes      | yes         | yes         |
| `PostalReceiveDeleted`             | `PostalReceive`     | —                                                          | An inbound postal record was soft-deleted.                                   | yes      | yes         | yes         |

**See also:** `docs/specs/documents/events.md` for full Rust struct
definitions, the canonical `EventEnvelope`, and per-event
subscribers.
