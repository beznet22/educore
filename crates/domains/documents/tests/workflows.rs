//! Integration tests for the **Documents domain workflows**.
//!
//! Implements: `docs/specs/documents/workflows.md`
//!
//! Each test exercises a spec-mandated workflow end-to-end
//! through the documents aggregate methods and asserts that the
//! expected typed event is emitted (or, on the error path,
//! that the expected [`DocumentsError`] is returned and no
//! state change is applied).
//!
//! The tests are written as **pure synchronous** tests: the
//! documents aggregate methods (`FormDownload::new`,
//! `form.update`, `form.soft_delete`, `PostalDispatch::new`,
//! `dispatch.update`, `dispatch.soft_delete`,
//! `PostalReceive::new`, `receive.update`,
//! `receive.soft_delete`) are sync and return
//! `Result<(), DocumentsError>` for state-machine transitions.
//! The test wires a [`TestClock`] and a [`SystemIdGen`], and
//! constructs the typed events directly from the aggregate +
//! clock instant to verify the event payloads.
//!
//! Per `docs/audit_reports/remediation/03-cluster-c-spec-drift.md`
//! the **handlers** are not yet wired end-to-end (no subscriber
//! fan-out, no outbox commit, no audit row). These tests pin
//! the contract of the **aggregate layer** that the service
//! factory fns (`upload_form_service`, `update_form_service`,
//! `delete_form_service`, `dispatch_postal_service`,
//! `update_postal_dispatch_service`,
//! `delete_postal_dispatch_service`, `receive_postal_service`,
//! `update_postal_receive_service`,
//! `delete_postal_receive_service`) and the eventual
//! dispatcher wrap. When the handlers land, the same test
//! bodies will gain a `+ outbox + bus subscriber` assertion
//! without changes to the assertions on the returned event.
//!
//! Per **WF-016**, the CMS domain's
//! `form_uploaded_public_indexing_subscriber` reads
//! `payload.show_public` off the bus-port envelope produced
//! from `documents.form_download.uploaded`. Section 2 of this
//! file pins the documents-side payload contract: the
//! `FormUploaded` event MUST serialise `show_public` as a JSON
//! boolean so the CMS subscriber can branch on it without
//! further deserialisation work.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::clock::{Clock as _, IdGenerator as _, SystemIdGen, TestClock};
use educore_core::ids::{CorrelationId, EventId};
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_documents::aggregate::{
    NewFormDownload, NewPostalDispatch, NewPostalReceive, PostalDispatch, PostalReceive,
    UpdateFormDownload, UpdatePostalDispatch, UpdatePostalReceive,
};
use educore_documents::prelude::*;
use educore_documents::value_objects::{
    ActiveStatus, DispatchDate, FromAddress, FromTitle, PostalAddress, PostalReferenceNo,
    PostalTitle, PublishDate, ReceiveDate, ShowPublic, ToAddress, ToTitle,
};
use educore_events::domain_event::DomainEvent;

// =============================================================================
// Test fixtures
// =============================================================================

/// A fresh `TenantContext` for a `SchoolAdmin` acting on a
/// freshly-minted school.
fn admin_context() -> (TenantContext, SystemIdGen) {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    (
        TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin),
        g,
    )
}

fn form_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> FormDownloadId {
    FormDownloadId::new(school, g.next_uuid())
}

fn postal_dispatch_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> PostalDispatchId {
    PostalDispatchId::new(school, g.next_uuid())
}

fn postal_receive_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> PostalReceiveId {
    PostalReceiveId::new(school, g.next_uuid())
}

fn date(y: i32, m: u32, d: u32) -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// Construct a fresh active `FormDownload` aggregate for a
/// given school + actor with the supplied link / file / public
/// flag. The aggregate is anchored to the supplied school via
/// the typed `FormDownloadId`.
#[allow(clippy::too_many_arguments)]
fn new_form(
    g: &SystemIdGen,
    school: educore_core::ids::SchoolId,
    actor: educore_core::ids::UserId,
    title: &str,
    link: Option<Url>,
    file: Option<FileReference>,
    show_public: bool,
    publish_date: chrono::NaiveDate,
) -> FormDownload {
    let at = Timestamp::now();
    FormDownload::new(NewFormDownload {
        id: form_id(g, school),
        title: FormTitle::new(title).unwrap(),
        short_description: None,
        publish_date: PublishDate::new(publish_date),
        link,
        file,
        show_public: ShowPublic::new(show_public),
        created_by: actor,
        created_at: at,
        correlation_id: g.next_correlation_id(),
    })
    .expect("FormDownload::new must succeed for valid link-or-file")
}

/// Construct a fresh active `PostalDispatch` aggregate for a
/// given school + actor.
#[allow(clippy::too_many_arguments)]
fn new_postal_dispatch(
    g: &SystemIdGen,
    school: educore_core::ids::SchoolId,
    actor: educore_core::ids::UserId,
    to_title: &str,
    from_title: &str,
    reference_no: Option<&str>,
    address: &str,
    date: chrono::NaiveDate,
) -> PostalDispatch {
    let at = Timestamp::now();
    let id = postal_dispatch_id(g, school);
    // The aggregate derives `school_id` from `id.school_id()`
    // (it is never taken from the caller). We pick an academic
    // year id via the id generator so two successive calls
    // never collide on the (school_id, academic_id,
    // reference_no) unique constraint.
    let academic_id = g.next_uuid();
    PostalDispatch::new(NewPostalDispatch {
        id,
        academic_id,
        to_title: ToTitle::new(PostalTitle::new(to_title).unwrap()),
        from_title: FromTitle::new(PostalTitle::new(from_title).unwrap()),
        reference_no: reference_no.map(|s| PostalReferenceNo::new(s).unwrap()),
        address: ToAddress::new(PostalAddress::new(address).unwrap()),
        date: DispatchDate::new(date),
        note: None,
        file: None,
        created_by: actor,
        created_at: at,
        correlation_id: g.next_correlation_id(),
    })
    .expect("PostalDispatch::new must succeed for valid shape")
}

/// Construct a fresh active `PostalReceive` aggregate for a
/// given school + actor.
#[allow(clippy::too_many_arguments)]
fn new_postal_receive(
    g: &SystemIdGen,
    school: educore_core::ids::SchoolId,
    actor: educore_core::ids::UserId,
    from_title: &str,
    to_title: &str,
    reference_no: Option<&str>,
    address: &str,
    date: chrono::NaiveDate,
) -> PostalReceive {
    let at = Timestamp::now();
    let id = postal_receive_id(g, school);
    let academic_id = g.next_uuid();
    PostalReceive::new(NewPostalReceive {
        id,
        academic_id,
        from_title: FromTitle::new(PostalTitle::new(from_title).unwrap()),
        to_title: ToTitle::new(PostalTitle::new(to_title).unwrap()),
        reference_no: reference_no.map(|s| PostalReferenceNo::new(s).unwrap()),
        address: FromAddress::new(PostalAddress::new(address).unwrap()),
        date: ReceiveDate::new(date),
        note: None,
        file: None,
        created_by: actor,
        created_at: at,
        correlation_id: g.next_correlation_id(),
    })
    .expect("PostalReceive::new must succeed for valid shape")
}

// =============================================================================
// 1. Document Upload Lifecycle
//    (`workflows.md` § "Form Download Lifecycle")
// =============================================================================

/// Upload lifecycle step 1: creating a form with a link
/// emits [`FormUploaded`] with the supplied title and
/// `show_public = false` (staff-only).
#[test]
fn form_upload_lifecycle_create_with_link_emits_form_uploaded() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let form = new_form(
        &g,
        school,
        actor,
        "Parent Consent",
        Some(Url::new("https://example.com/consent.pdf").unwrap()),
        None,
        false,
        date(2026, 6, 1),
    );
    let event: FormUploaded = FormUploaded::new(&form, actor, clock.now(), correlation);

    assert_eq!(
        <FormUploaded as DomainEvent>::EVENT_TYPE,
        "documents.form_download.uploaded"
    );
    assert_eq!(event.school_id, school);
    assert_eq!(event.title.as_str(), "Parent Consent");
    assert!(!event.show_public.is_public());
    assert_eq!(event.form_id, form.id);
    assert_eq!(event.uploaded_by, actor);
}

/// Upload lifecycle step 1: creating a form with a file
/// (no link) emits [`FormUploaded`] with `show_public = true`.
/// Per spec invariant 2, the form MUST have at least one of
/// `link` or `file`; here we exercise the file-only path.
#[test]
fn form_upload_lifecycle_create_with_file_only_emits_form_uploaded() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let form = new_form(
        &g,
        school,
        actor,
        "Holiday Notice",
        None,
        Some(FileReference::new("object-key-holiday-2026").unwrap()),
        true,
        date(2026, 6, 1),
    );
    let event: FormUploaded = FormUploaded::new(&form, actor, clock.now(), correlation);

    assert!(form.is_deliverable());
    assert!(form.is_public());
    assert_eq!(
        <FormUploaded as DomainEvent>::EVENT_TYPE,
        "documents.form_download.uploaded"
    );
    assert!(event.show_public.is_public());
}

/// Upload lifecycle failure path: per spec invariant 2, a form
/// with neither `link` nor `file` set MUST be rejected with
/// `DocumentsError::FormHasNoContent`.
#[test]
fn form_upload_lifecycle_no_link_or_file_returns_form_has_no_content() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let at = Timestamp::now();

    let err = FormDownload::new(NewFormDownload {
        id: form_id(&g, school),
        title: FormTitle::new("Empty Form").unwrap(),
        short_description: None,
        publish_date: PublishDate::new(date(2026, 6, 1)),
        link: None,
        file: None,
        show_public: ShowPublic::new(false),
        created_by: actor,
        created_at: at,
        correlation_id: g.next_correlation_id(),
    })
    .expect_err("missing link-or-file must fail validation");
    assert!(
        matches!(err, DocumentsError::FormHasNoContent),
        "got {err:?}"
    );
}

/// Upload lifecycle step 3: updating a form (changing title)
/// emits [`FormUpdated`] with the list of changed field names
/// and bumps the version.
#[test]
fn form_upload_lifecycle_update_title_emits_form_updated() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let mut form = new_form(
        &g,
        school,
        actor,
        "Original Title",
        Some(Url::new("https://example.com/x.pdf").unwrap()),
        None,
        false,
        date(2026, 6, 1),
    );
    let v0 = form.version;
    let actor2 = g.next_user_id();
    let event_id: EventId = g.next_event_id();
    form.update(UpdateFormDownload {
        title: Some(FormTitle::new("Updated Title").unwrap()),
        short_description: None,
        publish_date: None,
        link: None,
        file: None,
        show_public: None,
        actor: actor2,
        at: clock.now(),
        event_id,
    })
    .expect("update must succeed for active form");

    let event: FormUpdated = FormUpdated::new(
        &form,
        vec!["title".to_owned()],
        actor2,
        clock.now(),
        correlation,
    );

    assert_eq!(
        <FormUpdated as DomainEvent>::EVENT_TYPE,
        "documents.form_download.updated"
    );
    assert_eq!(event.changes, vec!["title".to_owned()]);
    assert_eq!(event.form_id, form.id);
    assert_eq!(event.updated_by, actor2);
    assert_eq!(form.title.as_str(), "Updated Title");
    assert_eq!(form.version, v0.next());
    assert_eq!(form.last_event_id, Some(event_id));
}

/// Upload lifecycle failure path: updating a soft-deleted form
/// MUST be rejected with `DocumentsError::Conflict`.
#[test]
fn form_upload_lifecycle_update_after_soft_delete_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let mut form = new_form(
        &g,
        school,
        actor,
        "Archived Form",
        Some(Url::new("https://example.com/x.pdf").unwrap()),
        None,
        false,
        date(2026, 6, 1),
    );
    form.soft_delete(actor, clock.now()).unwrap();
    assert!(!form.is_active());

    let err = form
        .update(UpdateFormDownload {
            title: Some(FormTitle::new("Try to update").unwrap()),
            short_description: None,
            publish_date: None,
            link: None,
            file: None,
            show_public: None,
            actor,
            at: clock.now(),
            event_id: g.next_event_id(),
        })
        .expect_err("update after soft-delete must fail");
    assert!(matches!(err, DocumentsError::Conflict(_)), "got {err:?}");
}

/// Upload lifecycle step 4: soft-deleting a form emits
/// [`FormDeleted`] and transitions `active_status` to
/// archived. A second soft-delete MUST be rejected with
/// `DocumentsError::Conflict`.
#[test]
fn form_upload_lifecycle_soft_delete_emits_form_deleted_and_double_delete_fails() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let mut form = new_form(
        &g,
        school,
        actor,
        "Retired Form",
        Some(Url::new("https://example.com/old.pdf").unwrap()),
        None,
        false,
        date(2026, 6, 1),
    );
    let v0 = form.version;
    form.soft_delete(actor, clock.now()).unwrap();

    let event: FormDeleted = FormDeleted::new(&form, actor, clock.now(), correlation);

    assert_eq!(
        <FormDeleted as DomainEvent>::EVENT_TYPE,
        "documents.form_download.deleted"
    );
    assert_eq!(event.deleted_by, actor);
    assert_eq!(event.form_id, form.id);
    assert!(!form.is_active());
    assert_eq!(form.version, v0.next());

    // Idempotency: a second soft-delete is rejected.
    let err = form
        .soft_delete(actor, clock.now())
        .expect_err("double soft-delete must fail");
    assert!(matches!(err, DocumentsError::Conflict(_)), "got {err:?}");
}

/// Upload lifecycle invariants: `FormDownload::new` rejects an
/// empty title via the inner `FormTitle` validator so that
/// `FormDownload::new` can never receive an empty title.
#[test]
fn form_upload_lifecycle_empty_title_returns_validation_error() {
    let res = FormTitle::new(String::new());
    assert!(res.is_err(), "empty FormTitle must fail validation");
}

// =============================================================================
// 2. Document Form Download — bus subscriber contract (WF-016)
//    (`workflows.md` § "Form Download Lifecycle" step 2)
//
//    The CMS domain's `form_uploaded_public_indexing_subscriber`
//    reads `payload.show_public` off the bus-port envelope
//    produced from `documents.form_download.uploaded`. The
//    documents side MUST serialise `show_public` as a JSON
//    boolean so the CMS subscriber can branch on it without
//    further deserialisation work. These tests pin that
//    contract from the documents side.
// =============================================================================

/// Form-download contract: a public form's `FormUploaded`
/// event serialises `payload["show_public"]` to the JSON
/// value `true`, matching the CMS subscriber's
/// `payload.get("show_public").and_then(Value::as_bool)`
/// branch (which returns `FormIndexAction::Index`).
#[test]
fn form_download_event_payload_show_public_true_for_public_form() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let form = new_form(
        &g,
        school,
        actor,
        "Public Form",
        Some(Url::new("https://example.com/public.pdf").unwrap()),
        None,
        true,
        date(2026, 6, 1),
    );
    let event: FormUploaded = FormUploaded::new(&form, actor, clock.now(), correlation);
    let payload = <FormUploaded as DomainEvent>::to_value(&event);

    let show_public = payload
        .get("show_public")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    assert!(
        show_public,
        "payload['show_public'] must serialise to JSON true for a public form; payload = {payload}"
    );
}

/// Form-download contract: a staff-only form's `FormUploaded`
/// event serialises `payload["show_public"]` to the JSON
/// value `false`, matching the CMS subscriber's
/// `payload.get("show_public").and_then(Value::as_bool)`
/// branch (which returns `FormIndexAction::Ignore`).
#[test]
fn form_download_event_payload_show_public_false_for_staff_form() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let form = new_form(
        &g,
        school,
        actor,
        "Staff Form",
        Some(Url::new("https://example.com/staff.pdf").unwrap()),
        None,
        false,
        date(2026, 6, 1),
    );
    let event: FormUploaded = FormUploaded::new(&form, actor, clock.now(), correlation);
    let payload = <FormUploaded as DomainEvent>::to_value(&event);

    let show_public = payload
        .get("show_public")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true); // inverted: must be false
    assert!(
        !show_public,
        "payload['show_public'] must serialise to JSON false for a staff-only form; payload = {payload}"
    );
}

/// Form-download contract: the bus-port envelope built from a
/// `FormUploaded` event has `event_type =
/// "documents.form_download.uploaded"` and
/// `aggregate_type = "form_download"`. The CMS subscriber
/// dispatches on `event_type` to find the `show_public` field;
/// if either field drifts, the indexing branch is silently
/// skipped.
#[test]
fn form_download_envelope_event_type_matches_cms_subscriber_topic() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let form = new_form(
        &g,
        school,
        actor,
        "Public Form",
        Some(Url::new("https://example.com/public.pdf").unwrap()),
        None,
        true,
        date(2026, 6, 1),
    );
    let event: FormUploaded = FormUploaded::new(&form, actor, clock.now(), correlation);
    let envelope = event.into_envelope(&TenantContext::for_user(
        school,
        actor,
        correlation,
        UserType::SchoolAdmin,
    ));

    assert_eq!(
        envelope.event_type, "documents.form_download.uploaded",
        "envelope event_type must match the CMS subscriber topic"
    );
    assert_eq!(
        envelope.aggregate_type, "form_download",
        "envelope aggregate_type must match the CMS subscriber's expected aggregate"
    );
    assert_eq!(envelope.schema_version, 1);
    // The payload must contain the show_public boolean (re-
    // verifying the WF-016 contract from the envelope shape).
    let show_public = envelope
        .payload
        .get("show_public")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    assert!(show_public, "envelope payload['show_public'] must be true");
    // The envelope's aggregate topic is `documents.form_download`
    // (the domain prefix is taken from the first segment of
    // event_type). This is the topic key the subscriber uses
    // to filter cross-domain streams.
    assert_eq!(
        envelope.aggregate_topic(),
        "documents.form_download",
        "envelope aggregate topic must be documents.form_download for the CMS subscription filter"
    );
}

// =============================================================================
// 3. Document Permission Lifecycle
//    (`workflows.md` § "Form Download Lifecycle")
//
//    The "permission" surface for a form is the
//    `show_public` flag (controls whether the form is indexed
//    on the public site) and the `active_status` flag
//    (controls whether the form remains queryable at all).
//    Together they form the lifecycle:
//      share    = create with show_public = true
//      revoke   = update show_public from true -> false
//      expire   = soft-delete the form
// =============================================================================

/// Permission lifecycle step 1 ("share"): creating a form
/// with `show_public = true` emits [`FormUploaded`] with
/// `show_public.is_public() == true`. The CMS subscriber
/// downstream will index the form on the public site.
#[test]
fn permission_lifecycle_share_public_form_sets_show_public_true() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let form = new_form(
        &g,
        school,
        actor,
        "Shared Public Form",
        Some(Url::new("https://example.com/share.pdf").unwrap()),
        None,
        true, // share = publish publicly
        date(2026, 6, 1),
    );
    let event: FormUploaded = FormUploaded::new(&form, actor, clock.now(), correlation);

    assert!(form.is_public(), "shared form must be public");
    assert!(event.show_public.is_public());
    assert!(form.is_active(), "shared form must be active");
}

/// Permission lifecycle step 2 ("revoke"): updating a form to
/// set `show_public = false` emits [`FormUpdated`] with
/// `changes = ["show_public"]` and the `is_public()` predicate
/// returns `false`. The CMS subscriber downstream will stop
/// indexing the form on the public site.
#[test]
fn permission_lifecycle_revoke_public_form_clears_show_public_to_false() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let mut form = new_form(
        &g,
        school,
        actor,
        "Revoked Form",
        Some(Url::new("https://example.com/revoke.pdf").unwrap()),
        None,
        true, // initially shared
        date(2026, 6, 1),
    );
    assert!(form.is_public());

    // Revoke = flip show_public to false.
    form.update(UpdateFormDownload {
        title: None,
        short_description: None,
        publish_date: None,
        link: None,
        file: None,
        show_public: Some(ShowPublic::new(false)),
        actor,
        at: clock.now(),
        event_id: g.next_event_id(),
    })
    .expect("update must succeed for active form");

    let event: FormUpdated = FormUpdated::new(
        &form,
        vec!["show_public".to_owned()],
        actor,
        clock.now(),
        correlation,
    );

    assert!(!form.is_public(), "revoked form must not be public");
    assert!(event.changes.contains(&"show_public".to_owned()));
    assert!(
        form.is_active(),
        "revoked form is still active (revoke != expire)"
    );
}

/// Permission lifecycle step 3 ("expire"): soft-deleting a
/// form emits [`FormDeleted`] and transitions `active_status`
/// to archived. The form remains queryable for audit
/// (`is_active() == false`) but is excluded from default
/// listings. This is the only "expire" path: per spec
/// invariant 4, records are never hard-deleted.
#[test]
fn permission_lifecycle_expire_form_via_soft_delete_marks_inactive() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let mut form = new_form(
        &g,
        school,
        actor,
        "Expired Form",
        Some(Url::new("https://example.com/expire.pdf").unwrap()),
        None,
        true,             // shared before expiring
        date(2026, 1, 1), // publish date in the past
    );
    assert!(form.is_active());

    form.soft_delete(actor, clock.now()).unwrap();

    let event: FormDeleted = FormDeleted::new(&form, actor, clock.now(), correlation);

    assert!(!form.is_active(), "expired form must be inactive");
    assert!(
        matches!(form.active_status, ActiveStatus(false)),
        "active_status must be archived; got {:?}",
        form.active_status
    );
    assert_eq!(
        <FormDeleted as DomainEvent>::EVENT_TYPE,
        "documents.form_download.deleted"
    );
    assert_eq!(event.deleted_by, actor);
    assert_eq!(event.form_id, form.id);
    assert_eq!(event.correlation_id, correlation);
}

// =============================================================================
// 4. Postal Dispatch Lifecycle
//    (`workflows.md` § "Postal Dispatch Tracking")
//
//    Per the spec, dispatch is recorded with `to_title`,
//    `from_title`, an optional reference number, an address, a
//    date, an optional note, and an optional attachment. The
//    reference number is **immutable once set** (workflow
//    step 3) and **unique within `(school_id, academic_id)`**.
// =============================================================================

/// Dispatch lifecycle step 1: recording a dispatch emits
/// [`PostalDispatched`] with the supplied `to_title`,
/// `from_title`, optional reference number, and dispatch date.
#[test]
fn postal_dispatch_lifecycle_dispatch_emits_postal_dispatched() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let dispatch = new_postal_dispatch(
        &g,
        school,
        actor,
        "Mr Smith",
        "Acme School",
        Some("REF-2026-0001"),
        "1 Main St",
        date(2026, 6, 1),
    );
    let event: PostalDispatched = PostalDispatched::new(&dispatch, actor, clock.now(), correlation);

    assert_eq!(
        <PostalDispatched as DomainEvent>::EVENT_TYPE,
        "documents.postal_dispatch.dispatched"
    );
    assert_eq!(event.school_id, school);
    assert_eq!(event.to_title.as_str(), "Mr Smith");
    assert_eq!(event.from_title.as_str(), "Acme School");
    assert_eq!(
        event
            .reference_no
            .as_ref()
            .map(PostalReferenceNo::as_str)
            .unwrap_or(""),
        "REF-2026-0001"
    );
    assert_eq!(event.dispatched_by, actor);
    assert!(dispatch.is_active());
}

/// Dispatch lifecycle failure path: changing an existing
/// reference number MUST be rejected with
/// `DocumentsError::ReferenceNoImmutable` (workflow step 3:
/// "The reference number is immutable."). Setting a *new*
/// value via `Some(Some(_))` is a change attempt.
#[test]
fn postal_dispatch_lifecycle_changing_reference_no_returns_immutable() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let mut dispatch = new_postal_dispatch(
        &g,
        school,
        actor,
        "Mr Smith",
        "Acme School",
        Some("REF-2026-0001"),
        "1 Main St",
        date(2026, 6, 1),
    );

    let err = dispatch
        .update(UpdatePostalDispatch {
            academic_id: None,
            to_title: None,
            from_title: None,
            reference_no: Some(Some(PostalReferenceNo::new("REF-OTHER").unwrap())),
            address: None,
            date: None,
            note: None,
            file: None,
            actor,
            at: clock.now(),
            event_id: g.next_event_id(),
        })
        .expect_err("reference_no change must be rejected");
    assert!(
        matches!(err, DocumentsError::ReferenceNoImmutable),
        "got {err:?}"
    );

    // Clearing an existing reference_no (`Some(None)`) is
    // also rejected.
    let err = dispatch
        .update(UpdatePostalDispatch {
            academic_id: None,
            to_title: None,
            from_title: None,
            reference_no: Some(None),
            address: None,
            date: None,
            note: None,
            file: None,
            actor,
            at: clock.now(),
            event_id: g.next_event_id(),
        })
        .expect_err("reference_no clear must be rejected");
    assert!(
        matches!(err, DocumentsError::ReferenceNoImmutable),
        "got {err:?}"
    );
}

/// Dispatch lifecycle step 3: updating a non-reference field
/// (e.g. `to_title`) emits [`PostalDispatchUpdated`] with the
/// changed field name and bumps the version. The reference
/// number remains untouched.
#[test]
fn postal_dispatch_lifecycle_update_to_title_emits_postal_dispatch_updated() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let mut dispatch = new_postal_dispatch(
        &g,
        school,
        actor,
        "Mr Smith",
        "Acme School",
        Some("REF-2026-0002"),
        "1 Main St",
        date(2026, 6, 1),
    );
    let v0 = dispatch.version;

    dispatch
        .update(UpdatePostalDispatch {
            academic_id: None,
            to_title: Some(ToTitle::new(PostalTitle::new("Mr Jones").unwrap())),
            from_title: None,
            reference_no: None,
            address: None,
            date: None,
            note: None,
            file: None,
            actor,
            at: clock.now(),
            event_id: g.next_event_id(),
        })
        .expect("update must succeed for active dispatch");

    let event: PostalDispatchUpdated = PostalDispatchUpdated::new(
        &dispatch,
        vec!["to_title".to_owned()],
        actor,
        clock.now(),
        correlation,
    );

    assert_eq!(
        <PostalDispatchUpdated as DomainEvent>::EVENT_TYPE,
        "documents.postal_dispatch.updated"
    );
    assert_eq!(event.changes, vec!["to_title".to_owned()]);
    assert_eq!(event.updated_by, actor);
    assert_eq!(dispatch.to_title.as_str(), "Mr Jones");
    assert_eq!(dispatch.version, v0.next());
    // Reference number is untouched.
    assert_eq!(
        dispatch
            .reference_no
            .as_ref()
            .map(PostalReferenceNo::as_str)
            .unwrap_or(""),
        "REF-2026-0002"
    );
}

/// Dispatch lifecycle step 4: soft-deleting a dispatch emits
/// [`PostalDispatchDeleted`] and a second soft-delete MUST be
/// rejected with `DocumentsError::Conflict`. The reference
/// number (immutable) is preserved through the soft-delete.
#[test]
fn postal_dispatch_lifecycle_soft_delete_emits_postal_dispatch_deleted() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let mut dispatch = new_postal_dispatch(
        &g,
        school,
        actor,
        "Mr Smith",
        "Acme School",
        Some("REF-2026-0003"),
        "1 Main St",
        date(2026, 6, 1),
    );
    dispatch.soft_delete(actor, clock.now()).unwrap();

    let event: PostalDispatchDeleted =
        PostalDispatchDeleted::new(&dispatch, actor, clock.now(), correlation);

    assert_eq!(
        <PostalDispatchDeleted as DomainEvent>::EVENT_TYPE,
        "documents.postal_dispatch.deleted"
    );
    assert_eq!(event.deleted_by, actor);
    assert!(!dispatch.is_active());

    // Double soft-delete is rejected.
    let err = dispatch
        .soft_delete(actor, clock.now())
        .expect_err("double soft-delete must fail");
    assert!(matches!(err, DocumentsError::Conflict(_)), "got {err:?}");
}

// =============================================================================
// 5. Postal Receive Lifecycle
//    (`workflows.md` § "Postal Receive Tracking")
// =============================================================================

/// Receive lifecycle step 1: recording a receive emits
/// [`PostalReceived`] with the supplied `from_title`,
/// `to_title`, optional reference number, and receive date.
#[test]
fn postal_receive_lifecycle_receive_emits_postal_received() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let receive = new_postal_receive(
        &g,
        school,
        actor,
        "Acme Vendor",
        "Acme School",
        Some("REF-IN-0001"),
        "5 Vendor Rd",
        date(2026, 6, 1),
    );
    let event: PostalReceived = PostalReceived::new(&receive, actor, clock.now(), correlation);

    assert_eq!(
        <PostalReceived as DomainEvent>::EVENT_TYPE,
        "documents.postal_receive.received"
    );
    assert_eq!(event.school_id, school);
    assert_eq!(event.from_title.as_str(), "Acme Vendor");
    assert_eq!(event.to_title.as_str(), "Acme School");
    assert_eq!(
        event
            .reference_no
            .as_ref()
            .map(PostalReferenceNo::as_str)
            .unwrap_or(""),
        "REF-IN-0001"
    );
    assert_eq!(event.received_by, actor);
    assert!(receive.is_active());
}

/// Receive lifecycle step 3: updating a non-reference field
/// (e.g. `from_title`) emits [`PostalReceiveUpdated`] with
/// the changed field name and bumps the version.
#[test]
fn postal_receive_lifecycle_update_from_title_emits_postal_receive_updated() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let mut receive = new_postal_receive(
        &g,
        school,
        actor,
        "Acme Vendor",
        "Acme School",
        Some("REF-IN-0002"),
        "5 Vendor Rd",
        date(2026, 6, 1),
    );
    let v0 = receive.version;

    receive
        .update(UpdatePostalReceive {
            academic_id: None,
            from_title: Some(FromTitle::new(PostalTitle::new("New Vendor").unwrap())),
            to_title: None,
            reference_no: None,
            address: None,
            date: None,
            note: None,
            file: None,
            actor,
            at: clock.now(),
            event_id: g.next_event_id(),
        })
        .expect("update must succeed for active receive");

    let event: PostalReceiveUpdated = PostalReceiveUpdated::new(
        &receive,
        vec!["from_title".to_owned()],
        actor,
        clock.now(),
        correlation,
    );

    assert_eq!(
        <PostalReceiveUpdated as DomainEvent>::EVENT_TYPE,
        "documents.postal_receive.updated"
    );
    assert_eq!(event.changes, vec!["from_title".to_owned()]);
    assert_eq!(receive.from_title.as_str(), "New Vendor");
    assert_eq!(receive.version, v0.next());
    // Reference number is preserved across the update.
    assert_eq!(
        receive
            .reference_no
            .as_ref()
            .map(PostalReferenceNo::as_str)
            .unwrap_or(""),
        "REF-IN-0002"
    );
}

/// Receive lifecycle step 4: soft-deleting a receive emits
/// [`PostalReceiveDeleted`] and transitions `active_status` to
/// archived. A second soft-delete MUST be rejected with
/// `DocumentsError::Conflict`.
#[test]
fn postal_receive_lifecycle_soft_delete_emits_postal_receive_deleted() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let mut receive = new_postal_receive(
        &g,
        school,
        actor,
        "Acme Vendor",
        "Acme School",
        Some("REF-IN-0003"),
        "5 Vendor Rd",
        date(2026, 6, 1),
    );
    receive.soft_delete(actor, clock.now()).unwrap();

    let event: PostalReceiveDeleted =
        PostalReceiveDeleted::new(&receive, actor, clock.now(), correlation);

    assert_eq!(
        <PostalReceiveDeleted as DomainEvent>::EVENT_TYPE,
        "documents.postal_receive.deleted"
    );
    assert_eq!(event.deleted_by, actor);
    assert_eq!(event.postal_receive_id, receive.id);
    assert!(!receive.is_active());

    let err = receive
        .soft_delete(actor, clock.now())
        .expect_err("double soft-delete must fail");
    assert!(matches!(err, DocumentsError::Conflict(_)), "got {err:?}");
}
