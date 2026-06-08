# Documents Domain — Events

Domain events describe facts that have already happened. They are
immutable, append-only records used for cross-domain integration, audit,
and event sourcing.

All events implement:

```rust
pub trait DomainEvent: Serialize + DeserializeOwned + Send + Sync {
    const TYPE: &'static str;
    fn aggregate_id(&self) -> Uuid;
    fn school_id(&self) -> SchoolId;
    fn occurred_at(&self) -> Timestamp;
}
```

The event envelope wraps the event with metadata:

```rust
pub struct EventEnvelope<E> {
    pub event_id: EventId,
    pub event_type: &'static str,
    pub school_id: SchoolId,
    pub aggregate_id: Uuid,
    pub aggregate_type: &'static str,
    pub actor_id: UserId,
    pub correlation_id: CorrelationId,
    pub causation_id: Option<EventId>,
    pub occurred_at: Timestamp,
    pub payload: E,
}
```

## Form Lifecycle

```rust
pub struct FormUploaded {
    pub form_id: FormDownloadId,
    pub title: FormTitle,
    pub publish_date: PublishDate,
    pub show_public: ShowPublic,
    pub uploaded_by: UserId,
}

pub struct FormUpdated { pub form_id: FormDownloadId, pub changes: Vec<&'static str> }
pub struct FormDeleted { pub form_id: FormDownloadId, pub deleted_by: UserId }
```

**Subscribers:**
- The CMS domain may subscribe to `FormUploaded` to surface the form
  on the public site when `show_public = true`.
- The search index port may index the form for in-site search.

## Postal Dispatch Lifecycle

```rust
pub struct PostalDispatched {
    pub postal_dispatch_id: PostalDispatchId,
    pub to_title: ToTitle,
    pub from_title: FromTitle,
    pub reference_no: Option<PostalReferenceNo>,
    pub date: DispatchDate,
    pub dispatched_by: UserId,
}

pub struct PostalDispatchUpdated {
    pub postal_dispatch_id: PostalDispatchId,
    pub changes: Vec<&'static str>,
}

pub struct PostalDispatchDeleted {
    pub postal_dispatch_id: PostalDispatchId,
    pub deleted_by: UserId,
}
```

**Subscribers:**
- The communication domain may subscribe to dispatch a confirmation
  to the recipient.

## Postal Receive Lifecycle

```rust
pub struct PostalReceived {
    pub postal_receive_id: PostalReceiveId,
    pub from_title: FromTitle,
    pub to_title: ToTitle,
    pub reference_no: Option<PostalReferenceNo>,
    pub date: ReceiveDate,
    pub received_by: UserId,
}

pub struct PostalReceiveUpdated {
    pub postal_receive_id: PostalReceiveId,
    pub changes: Vec<&'static str>,
}

pub struct PostalReceiveDeleted {
    pub postal_receive_id: PostalReceiveId,
    pub deleted_by: UserId,
}
```
