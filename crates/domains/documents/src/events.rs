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

use crate::aggregate::PostalReceive;
use crate::value_objects::{PostalReceiveId, ReceiveDate};

// =============================================================================
// PostalReceive lifecycle events (3)
// =============================================================================

/// Emitted when a new [`PostalReceive`] is recorded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostalReceived {
    /// The receive id.
    pub postal_receive_id: PostalReceiveId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The sender's title (1..=191 chars).
    pub from_title: FromTitle,
    /// The recipient's title (1..=191 chars).
    pub to_title: ToTitle,
    /// The optional reference number (unique within
    /// `(school_id, academic_id)`; immutable once set).
    pub reference_no: Option<PostalReferenceNo>,
    /// The receive date (may be in the past for back-filling).
    pub date: ReceiveDate,
    /// The user who recorded the receive.
    pub received_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id of the originating request.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl PostalReceived {
    /// Constructs a `PostalReceived` from a just-built
    /// [`PostalReceive`] aggregate, the acting user (used as
    /// `received_by`), the `occurred_at` timestamp, and the
    /// originating request's `correlation_id`. The `event_id` is
    /// minted as a fresh UUIDv7.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        receive: &PostalReceive,
        actor: UserId,
        at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            postal_receive_id: receive.id,
            school_id: receive.school_id,
            from_title: receive.from_title.clone(),
            to_title: receive.to_title.clone(),
            reference_no: receive.reference_no.clone(),
            date: receive.date,
            received_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for PostalReceived {
    const EVENT_TYPE: &'static str = "documents.postal_receive.received";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "postal_receive";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.postal_receive_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`PostalReceive`] is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostalReceiveUpdated {
    /// The receive id.
    pub postal_receive_id: PostalReceiveId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The list of changed field names (e.g. `["from_title",
    /// "to_title"]`).
    pub changes: Vec<String>,
    /// The user who updated the receive.
    pub updated_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id of the originating request.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl PostalReceiveUpdated {
    /// Constructs a `PostalReceiveUpdated` from the target
    /// [`PostalReceive`] aggregate, the list of changed field
    /// names, the acting user, the `occurred_at` timestamp, and
    /// the originating request's `correlation_id`. The
    /// `event_id` is minted as a fresh UUIDv7.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        receive: &PostalReceive,
        changes: Vec<String>,
        actor: UserId,
        at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            postal_receive_id: receive.id,
            school_id: receive.school_id,
            changes,
            updated_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for PostalReceiveUpdated {
    const EVENT_TYPE: &'static str = "documents.postal_receive.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "postal_receive";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.postal_receive_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`PostalReceive`] is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostalReceiveDeleted {
    /// The receive id.
    pub postal_receive_id: PostalReceiveId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The user who soft-deleted the receive.
    pub deleted_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id of the originating request.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl PostalReceiveDeleted {
    /// Constructs a `PostalReceiveDeleted` from the target
    /// [`PostalReceive`] aggregate, the acting user, the
    /// `occurred_at` timestamp, and the originating request's
    /// `correlation_id`. The `event_id` is minted as a fresh
    /// UUIDv7.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        receive: &PostalReceive,
        actor: UserId,
        at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            postal_receive_id: receive.id,
            school_id: receive.school_id,
            deleted_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for PostalReceiveDeleted {
    const EVENT_TYPE: &'static str = "documents.postal_receive.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "postal_receive";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.postal_receive_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// === PostalReceive events section end ===

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;
    use educore_core::clock::IdGenerator as _;

    fn ids() -> (
        educore_core::ids::SchoolId,
        educore_core::ids::UserId,
        educore_core::ids::EventId,
        educore_core::ids::CorrelationId,
        educore_core::value_objects::Timestamp,
    ) {
        let g = educore_core::clock::SystemIdGen;
        let s = g.next_school_id();
        let u = g.next_user_id();
        let e = g.next_event_id();
        let c = g.next_correlation_id();
        let t = educore_core::value_objects::Timestamp::now();
        (s, u, e, c, t)
    }

    fn title() -> crate::value_objects::FormTitle {
        crate::value_objects::FormTitle::new("Consent Form").unwrap()
    }

    fn publish_date() -> crate::value_objects::PublishDate {
        crate::value_objects::PublishDate::new(
            chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        )
    }

    fn url() -> crate::value_objects::Url {
        crate::value_objects::Url::new("https://example.com/form.pdf").unwrap()
    }

    fn file_ref() -> crate::value_objects::FileReference {
        crate::value_objects::FileReference::new("object-key-1234").unwrap()
    }

    fn make_form() -> crate::aggregate::FormDownload {
        let (s, u, _e, c, t) = ids();
        let id = crate::value_objects::FormDownloadId::new(s, uuid::Uuid::now_v7());
        let cmd = crate::aggregate::NewFormDownload {
            id,
            title: title(),
            short_description: None,
            publish_date: publish_date(),
            link: Some(url()),
            file: None,
            show_public: crate::value_objects::ShowPublic::default(),
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        crate::aggregate::FormDownload::new(cmd).expect("ok")
    }

    fn make_dispatch() -> crate::aggregate::PostalDispatch {
        let (s, u, _e, c, t) = ids();
        let id = crate::value_objects::PostalDispatchId::new(s, uuid::Uuid::now_v7());
        let cmd = crate::aggregate::NewPostalDispatch {
            id,
            academic_id: uuid::Uuid::now_v7(),
            to_title: crate::value_objects::ToTitle::new(
                crate::value_objects::PostalTitle::new("Mr Smith").unwrap(),
            ),
            from_title: crate::value_objects::FromTitle::new(
                crate::value_objects::PostalTitle::new("Acme School").unwrap(),
            ),
            reference_no: Some(
                crate::value_objects::PostalReferenceNo::new("REF-2026-0001").unwrap(),
            ),
            address: crate::value_objects::ToAddress::new(
                crate::value_objects::PostalAddress::new("1 Main St").unwrap(),
            ),
            date: crate::value_objects::DispatchDate::new(
                chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            ),
            note: None,
            file: None,
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        crate::aggregate::PostalDispatch::new(cmd).expect("ok")
    }

    fn make_receive() -> crate::aggregate::PostalReceive {
        let (s, u, _e, c, t) = ids();
        let id = crate::value_objects::PostalReceiveId::new(s, uuid::Uuid::now_v7());
        let cmd = crate::aggregate::NewPostalReceive {
            id,
            academic_id: uuid::Uuid::now_v7(),
            from_title: crate::value_objects::FromTitle::new(
                crate::value_objects::PostalTitle::new("Acme Vendor").unwrap(),
            ),
            to_title: crate::value_objects::ToTitle::new(
                crate::value_objects::PostalTitle::new("Acme School").unwrap(),
            ),
            reference_no: Some(
                crate::value_objects::PostalReferenceNo::new("REF-IN-0001").unwrap(),
            ),
            address: crate::value_objects::FromAddress::new(
                crate::value_objects::PostalAddress::new("5 Vendor Rd").unwrap(),
            ),
            date: crate::value_objects::ReceiveDate::new(
                chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            ),
            note: None,
            file: None,
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        crate::aggregate::PostalReceive::new(cmd).expect("ok")
    }

    // ---- FormDownload events ----

    #[test]
    fn form_uploaded_event_metadata_round_trip() {
        let form = make_form();
        let s = form.school_id;
        let u = educore_core::clock::SystemIdGen.next_user_id();
        let c = educore_core::clock::SystemIdGen.next_correlation_id();
        let t = educore_core::value_objects::Timestamp::now();
        let event = FormUploaded::new(&form, u, t, c);
        assert_eq!(
            <FormUploaded as DomainEvent>::EVENT_TYPE,
            "documents.form_download.uploaded"
        );
        assert_eq!(<FormUploaded as DomainEvent>::SCHEMA_VERSION, 1);
        assert_eq!(<FormUploaded as DomainEvent>::AGGREGATE_TYPE, "form_download");
        assert_eq!(event.form_id, form.id);
        assert_eq!(event.school_id, s);
        assert_eq!(event.title, form.title);
        assert_eq!(event.uploaded_by, u);
        assert_eq!(event.occurred_at, t);
        assert_eq!(event.correlation_id, c);
    }

    #[test]
    fn form_updated_event_metadata_round_trip() {
        let form = make_form();
        let s = form.school_id;
        let u = educore_core::clock::SystemIdGen.next_user_id();
        let c = educore_core::clock::SystemIdGen.next_correlation_id();
        let t = educore_core::value_objects::Timestamp::now();
        let event = FormUpdated::new(
            &form,
            vec!["title".to_owned(), "link".to_owned()],
            u,
            t,
            c,
        );
        assert_eq!(
            <FormUpdated as DomainEvent>::EVENT_TYPE,
            "documents.form_download.updated"
        );
        assert_eq!(<FormUpdated as DomainEvent>::SCHEMA_VERSION, 1);
        assert_eq!(<FormUpdated as DomainEvent>::AGGREGATE_TYPE, "form_download");
        assert_eq!(event.form_id, form.id);
        assert_eq!(event.school_id, s);
        assert_eq!(event.changes, vec!["title".to_owned(), "link".to_owned()]);
    }

    #[test]
    fn form_deleted_event_metadata_round_trip() {
        let form = make_form();
        let s = form.school_id;
        let u = educore_core::clock::SystemIdGen.next_user_id();
        let c = educore_core::clock::SystemIdGen.next_correlation_id();
        let t = educore_core::value_objects::Timestamp::now();
        let event = FormDeleted::new(&form, u, t, c);
        assert_eq!(
            <FormDeleted as DomainEvent>::EVENT_TYPE,
            "documents.form_download.deleted"
        );
        assert_eq!(<FormDeleted as DomainEvent>::SCHEMA_VERSION, 1);
        assert_eq!(<FormDeleted as DomainEvent>::AGGREGATE_TYPE, "form_download");
        assert_eq!(event.form_id, form.id);
        assert_eq!(event.school_id, s);
        assert_eq!(event.deleted_by, u);
    }

    // ---- PostalDispatch events ----

    #[test]
    fn postal_dispatched_event_metadata_round_trip() {
        let dispatch = make_dispatch();
        let s = dispatch.school_id;
        let u = educore_core::clock::SystemIdGen.next_user_id();
        let c = educore_core::clock::SystemIdGen.next_correlation_id();
        let t = educore_core::value_objects::Timestamp::now();
        let event = PostalDispatched::new(&dispatch, u, t, c);
        assert_eq!(
            <PostalDispatched as DomainEvent>::EVENT_TYPE,
            "documents.postal_dispatch.dispatched"
        );
        assert_eq!(<PostalDispatched as DomainEvent>::SCHEMA_VERSION, 1);
        assert_eq!(
            <PostalDispatched as DomainEvent>::AGGREGATE_TYPE,
            "postal_dispatch"
        );
        assert_eq!(event.postal_dispatch_id, dispatch.id);
        assert_eq!(event.school_id, s);
        assert_eq!(event.reference_no, dispatch.reference_no);
    }

    #[test]
    fn postal_dispatch_updated_event_metadata_round_trip() {
        let dispatch = make_dispatch();
        let s = dispatch.school_id;
        let u = educore_core::clock::SystemIdGen.next_user_id();
        let c = educore_core::clock::SystemIdGen.next_correlation_id();
        let t = educore_core::value_objects::Timestamp::now();
        let event = PostalDispatchUpdated::new(
            &dispatch,
            vec!["to_title".to_owned()],
            u,
            t,
            c,
        );
        assert_eq!(
            <PostalDispatchUpdated as DomainEvent>::EVENT_TYPE,
            "documents.postal_dispatch.updated"
        );
        assert_eq!(<PostalDispatchUpdated as DomainEvent>::SCHEMA_VERSION, 1);
        assert_eq!(
            <PostalDispatchUpdated as DomainEvent>::AGGREGATE_TYPE,
            "postal_dispatch"
        );
        assert_eq!(event.postal_dispatch_id, dispatch.id);
        assert_eq!(event.school_id, s);
    }

    #[test]
    fn postal_dispatch_deleted_event_metadata_round_trip() {
        let dispatch = make_dispatch();
        let s = dispatch.school_id;
        let u = educore_core::clock::SystemIdGen.next_user_id();
        let c = educore_core::clock::SystemIdGen.next_correlation_id();
        let t = educore_core::value_objects::Timestamp::now();
        let event = PostalDispatchDeleted::new(&dispatch, u, t, c);
        assert_eq!(
            <PostalDispatchDeleted as DomainEvent>::EVENT_TYPE,
            "documents.postal_dispatch.deleted"
        );
        assert_eq!(<PostalDispatchDeleted as DomainEvent>::SCHEMA_VERSION, 1);
        assert_eq!(
            <PostalDispatchDeleted as DomainEvent>::AGGREGATE_TYPE,
            "postal_dispatch"
        );
        assert_eq!(event.postal_dispatch_id, dispatch.id);
        assert_eq!(event.school_id, s);
        assert_eq!(event.deleted_by, u);
    }

    // ---- PostalReceive events ----

    #[test]
    fn postal_received_event_metadata_round_trip() {
        let receive = make_receive();
        let s = receive.school_id;
        let u = educore_core::clock::SystemIdGen.next_user_id();
        let c = educore_core::clock::SystemIdGen.next_correlation_id();
        let t = educore_core::value_objects::Timestamp::now();
        let event = PostalReceived::new(&receive, u, t, c);
        assert_eq!(
            <PostalReceived as DomainEvent>::EVENT_TYPE,
            "documents.postal_receive.received"
        );
        assert_eq!(<PostalReceived as DomainEvent>::SCHEMA_VERSION, 1);
        assert_eq!(
            <PostalReceived as DomainEvent>::AGGREGATE_TYPE,
            "postal_receive"
        );
        assert_eq!(event.postal_receive_id, receive.id);
        assert_eq!(event.school_id, s);
        assert_eq!(event.reference_no, receive.reference_no);
    }

    #[test]
    fn postal_receive_updated_event_metadata_round_trip() {
        let receive = make_receive();
        let s = receive.school_id;
        let u = educore_core::clock::SystemIdGen.next_user_id();
        let c = educore_core::clock::SystemIdGen.next_correlation_id();
        let t = educore_core::value_objects::Timestamp::now();
        let event =
            PostalReceiveUpdated::new(&receive, vec!["from_title".to_owned()], u, t, c);
        assert_eq!(
            <PostalReceiveUpdated as DomainEvent>::EVENT_TYPE,
            "documents.postal_receive.updated"
        );
        assert_eq!(<PostalReceiveUpdated as DomainEvent>::SCHEMA_VERSION, 1);
        assert_eq!(
            <PostalReceiveUpdated as DomainEvent>::AGGREGATE_TYPE,
            "postal_receive"
        );
        assert_eq!(event.postal_receive_id, receive.id);
        assert_eq!(event.school_id, s);
    }

    #[test]
    fn postal_receive_deleted_event_metadata_round_trip() {
        let receive = make_receive();
        let s = receive.school_id;
        let u = educore_core::clock::SystemIdGen.next_user_id();
        let c = educore_core::clock::SystemIdGen.next_correlation_id();
        let t = educore_core::value_objects::Timestamp::now();
        let event = PostalReceiveDeleted::new(&receive, u, t, c);
        assert_eq!(
            <PostalReceiveDeleted as DomainEvent>::EVENT_TYPE,
            "documents.postal_receive.deleted"
        );
        assert_eq!(<PostalReceiveDeleted as DomainEvent>::SCHEMA_VERSION, 1);
        assert_eq!(
            <PostalReceiveDeleted as DomainEvent>::AGGREGATE_TYPE,
            "postal_receive"
        );
        assert_eq!(event.postal_receive_id, receive.id);
        assert_eq!(event.school_id, s);
        assert_eq!(event.deleted_by, u);
    }

    // ---- Cross-event wire-form coverage ----

    #[test]
    fn all_nine_event_types_resolve_to_documents_domain_wire_form() {
        // Smoke test: the EVENT_TYPE for every event matches
        // the `documents.<aggregate>.<verb>` pattern.
        assert_eq!(
            <FormUploaded as DomainEvent>::EVENT_TYPE,
            "documents.form_download.uploaded"
        );
        assert_eq!(
            <FormUpdated as DomainEvent>::EVENT_TYPE,
            "documents.form_download.updated"
        );
        assert_eq!(
            <FormDeleted as DomainEvent>::EVENT_TYPE,
            "documents.form_download.deleted"
        );
        assert_eq!(
            <PostalDispatched as DomainEvent>::EVENT_TYPE,
            "documents.postal_dispatch.dispatched"
        );
        assert_eq!(
            <PostalDispatchUpdated as DomainEvent>::EVENT_TYPE,
            "documents.postal_dispatch.updated"
        );
        assert_eq!(
            <PostalDispatchDeleted as DomainEvent>::EVENT_TYPE,
            "documents.postal_dispatch.deleted"
        );
        assert_eq!(
            <PostalReceived as DomainEvent>::EVENT_TYPE,
            "documents.postal_receive.received"
        );
        assert_eq!(
            <PostalReceiveUpdated as DomainEvent>::EVENT_TYPE,
            "documents.postal_receive.updated"
        );
        assert_eq!(
            <PostalReceiveDeleted as DomainEvent>::EVENT_TYPE,
            "documents.postal_receive.deleted"
        );
    }

    #[test]
    fn all_nine_event_schemas_are_at_version_1() {
        assert_eq!(<FormUploaded as DomainEvent>::SCHEMA_VERSION, 1);
        assert_eq!(<FormUpdated as DomainEvent>::SCHEMA_VERSION, 1);
        assert_eq!(<FormDeleted as DomainEvent>::SCHEMA_VERSION, 1);
        assert_eq!(<PostalDispatched as DomainEvent>::SCHEMA_VERSION, 1);
        assert_eq!(<PostalDispatchUpdated as DomainEvent>::SCHEMA_VERSION, 1);
        assert_eq!(<PostalDispatchDeleted as DomainEvent>::SCHEMA_VERSION, 1);
        assert_eq!(<PostalReceived as DomainEvent>::SCHEMA_VERSION, 1);
        assert_eq!(<PostalReceiveUpdated as DomainEvent>::SCHEMA_VERSION, 1);
        assert_eq!(<PostalReceiveDeleted as DomainEvent>::SCHEMA_VERSION, 1);
    }
}
