//! Documents-domain events.

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

// === Form events section begin (owner: 2A) ===

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;

use crate::aggregate::FormDownload;
use crate::value_objects::{FormDownloadId, FormTitle, PublishDate, ShowPublic};

// =============================================================================
// Form lifecycle events (3)
// =============================================================================

/// Emitted when a new [`FormDownload`] is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormUploaded {
    /// The form id.
    pub form_id: FormDownloadId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The form title.
    pub title: FormTitle,
    /// The publish date.
    pub publish_date: PublishDate,
    /// The public-visibility flag.
    pub show_public: ShowPublic,
    /// The user who uploaded the form.
    pub uploaded_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id of the originating request.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl FormUploaded {
    /// Constructs a `FormUploaded` from a just-built
    /// [`FormDownload`] aggregate, the acting user (used as
    /// `uploaded_by`), the `occurred_at` timestamp, and the
    /// originating request's `correlation_id`. The `event_id` is
    /// minted as a fresh UUIDv7.
    #[must_use]
    pub fn new(
        form: &FormDownload,
        actor: UserId,
        at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            form_id: form.id,
            school_id: form.school_id,
            title: form.title.clone(),
            publish_date: form.publish_date,
            show_public: form.show_public,
            uploaded_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for FormUploaded {
    const EVENT_TYPE: &'static str = "documents.form_download.uploaded";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "form_download";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.form_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`FormDownload`] is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormUpdated {
    /// The form id.
    pub form_id: FormDownloadId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The list of changed field names (e.g. `["title",
    /// "publish_date"]`).
    pub changes: Vec<String>,
    /// The user who updated the form.
    pub updated_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id of the originating request.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl FormUpdated {
    /// Constructs a `FormUpdated` from the target [`FormDownload`]
    /// aggregate, the list of changed field names, the acting
    /// user, the `occurred_at` timestamp, and the originating
    /// request's `correlation_id`. The `event_id` is minted as a
    /// fresh UUIDv7.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        form: &FormDownload,
        changes: Vec<String>,
        actor: UserId,
        at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            form_id: form.id,
            school_id: form.school_id,
            changes,
            updated_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for FormUpdated {
    const EVENT_TYPE: &'static str = "documents.form_download.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "form_download";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.form_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`FormDownload`] is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormDeleted {
    /// The form id.
    pub form_id: FormDownloadId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The user who deleted the form.
    pub deleted_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id of the originating request.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl FormDeleted {
    /// Constructs a `FormDeleted` from the target [`FormDownload`]
    /// aggregate, the acting user, the `occurred_at` timestamp,
    /// and the originating request's `correlation_id`. The
    /// `event_id` is minted as a fresh UUIDv7.
    #[must_use]
    pub fn new(
        form: &FormDownload,
        actor: UserId,
        at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            form_id: form.id,
            school_id: form.school_id,
            deleted_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for FormDeleted {
    const EVENT_TYPE: &'static str = "documents.form_download.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "form_download";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.form_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// === Form events section end ===

// === PostalDispatch events section begin (owner: 2B) ===

// 2A above already imports `serde::{Deserialize, Serialize}`,
// `uuid::Uuid`, `educore_core::ids::{CorrelationId, EventId,
// SchoolId, UserId}`, `educore_core::value_objects::Timestamp`,
// and `educore_events::domain_event::DomainEvent` at the file
// scope. Re-importing them here is an E0252 duplicate. 2A also
// imports `crate::aggregate::FormDownload`; we need the
// `PostalDispatch` sibling and the value-object types that 2A
// did not bring in.

use crate::aggregate::PostalDispatch;
use crate::value_objects::{
    DispatchDate, FromTitle, PostalDispatchId, PostalReferenceNo, ToTitle,
};

// =============================================================================
// PostalDispatch lifecycle events (3)
// =============================================================================

/// Emitted when a new [`PostalDispatch`] is recorded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostalDispatched {
    /// The dispatch id.
    pub postal_dispatch_id: PostalDispatchId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The recipient's title (1..=191 chars).
    pub to_title: ToTitle,
    /// The sender's title (1..=191 chars).
    pub from_title: FromTitle,
    /// The optional reference number (unique within
    /// `(school_id, academic_id)`; immutable once set).
    pub reference_no: Option<PostalReferenceNo>,
    /// The dispatch date (may be in the past for back-filling).
    pub date: DispatchDate,
    /// The user who recorded the dispatch.
    pub dispatched_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id of the originating request.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl PostalDispatched {
    /// Constructs a `PostalDispatched` from a just-built
    /// [`PostalDispatch`] aggregate, the acting user (used as
    /// `dispatched_by`), the `occurred_at` timestamp, and the
    /// originating request's `correlation_id`. The `event_id` is
    /// minted as a fresh UUIDv7.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        dispatch: &PostalDispatch,
        actor: UserId,
        at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            postal_dispatch_id: dispatch.id,
            school_id: dispatch.school_id,
            to_title: dispatch.to_title.clone(),
            from_title: dispatch.from_title.clone(),
            reference_no: dispatch.reference_no.clone(),
            date: dispatch.date,
            dispatched_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for PostalDispatched {
    const EVENT_TYPE: &'static str = "documents.postal_dispatch.dispatched";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "postal_dispatch";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.postal_dispatch_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`PostalDispatch`] is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostalDispatchUpdated {
    /// The dispatch id.
    pub postal_dispatch_id: PostalDispatchId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The list of changed field names (e.g. `["to_title",
    /// "from_title"]`).
    pub changes: Vec<String>,
    /// The user who updated the dispatch.
    pub updated_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id of the originating request.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl PostalDispatchUpdated {
    /// Constructs a `PostalDispatchUpdated` from the target
    /// [`PostalDispatch`] aggregate, the list of changed field
    /// names, the acting user, the `occurred_at` timestamp, and
    /// the originating request's `correlation_id`. The
    /// `event_id` is minted as a fresh UUIDv7.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        dispatch: &PostalDispatch,
        changes: Vec<String>,
        actor: UserId,
        at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            postal_dispatch_id: dispatch.id,
            school_id: dispatch.school_id,
            changes,
            updated_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for PostalDispatchUpdated {
    const EVENT_TYPE: &'static str = "documents.postal_dispatch.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "postal_dispatch";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.postal_dispatch_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`PostalDispatch`] is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostalDispatchDeleted {
    /// The dispatch id.
    pub postal_dispatch_id: PostalDispatchId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The user who soft-deleted the dispatch.
    pub deleted_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id of the originating request.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl PostalDispatchDeleted {
    /// Constructs a `PostalDispatchDeleted` from the target
    /// [`PostalDispatch`] aggregate, the acting user, the
    /// `occurred_at` timestamp, and the originating request's
    /// `correlation_id`. The `event_id` is minted as a fresh
    /// UUIDv7.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        dispatch: &PostalDispatch,
        actor: UserId,
        at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            postal_dispatch_id: dispatch.id,
            school_id: dispatch.school_id,
            deleted_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for PostalDispatchDeleted {
    const EVENT_TYPE: &'static str = "documents.postal_dispatch.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "postal_dispatch";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.postal_dispatch_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// === PostalDispatch events section end ===

// === PostalReceive events section begin (owner: 2C) ===
pub struct PostalReceived;
pub struct PostalReceiveUpdated;
pub struct PostalReceiveDeleted;
// === PostalReceive events section end ===
