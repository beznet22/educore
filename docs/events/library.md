# Library Domain — Events

Quick reference of every event the library domain emits. Events are
immutable, append-only records. Every event carries a typed
`EventEnvelope` and is durably persisted to the aggregate's event
log. The library domain does not emit a generic state-change event;
specific events (`BookIssued`, `BookReturned`, `BookRenewed`,
`BookMarkedLost`, `FineCalculated`) carry the transition.

| Event                              | Aggregate           | Subscribers                              | Description                                                                              | Durable? | Replicated? | Replayable? |
| ---------------------------------- | ------------------- | ---------------------------------------- | ---------------------------------------------------------------------------------------- | -------- | ----------- | ----------- |
| `BookCategoryCreated`              | `BookCategory`      | —                                        | A book category was created.                                                             | yes      | yes         | yes         |
| `BookCategoryUpdated`              | `BookCategory`      | —                                        | A book category was patched.                                                             | yes      | yes         | yes         |
| `BookCategoryDeleted`              | `BookCategory`      | —                                        | A book category was soft-deleted.                                                        | yes      | yes         | yes         |
| `BookAdded`                        | `Book`              | —                                        | A book was added to the catalog.                                                         | yes      | yes         | yes         |
| `BookUpdated`                      | `Book`              | —                                        | A book's metadata was patched.                                                           | yes      | yes         | yes         |
| `BookDeleted`                      | `Book`              | —                                        | A book was soft-deleted.                                                                 | yes      | yes         | yes         |
| `BookQuantityAdjusted`             | `Book`              | `communication`                          | A book's stock count was adjusted.                                                      | yes      | yes         | yes         |
| `LibraryMemberRegistered`          | `LibraryMember`     | `communication`                          | A library member was registered.                                                         | yes      | yes         | yes         |
| `LibraryMemberUpdated`             | `LibraryMember`     | —                                        | A library member was patched.                                                            | yes      | yes         | yes         |
| `LibraryMemberDeactivated`         | `LibraryMember`     | —                                        | A library member was deactivated.                                                        | yes      | yes         | yes         |
| `LibraryMemberReactivated`         | `LibraryMember`     | —                                        | A library member was reactivated.                                                        | yes      | yes         | yes         |
| `LibraryMemberDeleted`             | `LibraryMember`     | —                                        | A library member was soft-deleted.                                                       | yes      | yes         | yes         |
| `BookIssued`                       | `BookIssue`         | `communication`                          | A book was issued to a member.                                                           | yes      | yes         | yes         |
| `BookReturned`                     | `BookIssue`         | `finance` (when a fine is also produced) | An issued book was returned.                                                             | yes      | yes         | yes         |
| `BookRenewed`                      | `BookIssue`         | `communication`                          | An issued book was renewed.                                                              | yes      | yes         | yes         |
| `BookMarkedLost`                   | `BookIssue`         | `finance`, `communication`               | An issued book was marked lost.                                                          | yes      | yes         | yes         |
| `FineCalculated`                   | `BookIssue`         | `finance`                                | A fine was calculated on an issue.                                                       | yes      | yes         | yes         |

**See also:** `docs/specs/library/events.md` for full Rust struct
definitions, the canonical `EventEnvelope`, and per-event
subscribers.
