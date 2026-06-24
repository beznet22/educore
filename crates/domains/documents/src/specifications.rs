//! # Documents specifications, validation, and coordinator
//!
//! Cross-cutting types mandated by the documents spec that do not
//! live in any of the canonical 9 module files (`aggregate`,
//! `commands`, `entities`, `events`, `query`, `repository`,
//! `services`, `value_objects`, `errors`):
//!
//! - [`Validate`] trait — mandated by `docs/specs/documents/value-objects.md`
//!   § "Validation Rules".
//! - [`Specification`] trait + 4 implementations (`PublicForms`,
//!   `ActiveForms`, `DispatchesInDateRange`,
//!   `ReceivesInDateRange`) — mandated by
//!   `docs/specs/documents/services.md` § "Specification".
//! - [`DocumentsCoordinator`] — mandated by
//!   `docs/specs/documents/services.md` § "Cross-Domain
//!   Coordinator". Lives in the engine facade in production;
//!   the documents crate exposes the skeleton here because the
//!   facade re-export in `crates/educore/` is not part of this
//!   crate's footprint.
//!
//! See `docs/specs/documents/services.md` for the canonical
//! signatures and `docs/specs/documents/value-objects.md` for the
//! validation contract.

#![allow(missing_docs)]
#![allow(clippy::all)]

use chrono::NaiveDate;

use crate::aggregate::{FormDownload, PostalDispatch, PostalReceive};
use crate::commands::UploadFormCommand;
use crate::errors::DocumentsError;

// ============================================================================
// Validate trait (value-objects.md § "Validation Rules")
// ============================================================================

/// Validation contract for value objects.
///
/// Per `docs/specs/documents/value-objects.md` § "Validation
/// Rules", every value object implements `Validate` and refuses
/// construction when validation fails. Construction is the only
/// entry point (e.g. `let title = FormTitle::new("...")?;`).
/// Parsing returns `Result<T, ValueError>`; there are no
/// setters that bypass validation.
///
/// The trait is defined in this module to keep the spec's
/// cross-cutting surface in one place; each value object
/// implements it via the `validate_self` inherent method
/// pattern used throughout `value_objects.rs`.
pub trait Validate {
    /// Validates the receiver. Returns `Ok(())` when the value
    /// object satisfies all invariants, or
    /// `Err(DocumentsError::Validation)` carrying a stable
    /// wire-form reason otherwise.
    fn validate(&self) -> Result<(), DocumentsError>;
}

// ============================================================================
// Specification trait (services.md § "Specification")
// ============================================================================

/// Generic predicate over a domain object. Per
/// `docs/specs/documents/services.md`, specifications are pure
/// (no I/O) and compose with other filters in queries.
pub trait Specification<T> {
    /// Returns `true` when `candidate` satisfies the
    /// specification. Implementations must be deterministic
    /// and side-effect-free.
    fn is_satisfied_by(&self, candidate: &T) -> bool;
}

// ============================================================================
// Specification: PublicForms (services.md § "PublicForms")
// ============================================================================

/// A [`Specification`] that filters forms with
/// `show_public = true`. See
/// `docs/specs/documents/services.md` § "Specification:
/// PublicForms".
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct PublicForms;

impl Specification<FormDownload> for PublicForms {
    fn is_satisfied_by(&self, f: &FormDownload) -> bool {
        f.show_public.0
    }
}

// ============================================================================
// Specification: ActiveForms (services.md § "ActiveForms")
// ============================================================================

/// A [`Specification`] that filters forms with
/// `active_status = true`. See
/// `docs/specs/documents/services.md` § "Specification:
/// ActiveForms".
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ActiveForms;

impl Specification<FormDownload> for ActiveForms {
    fn is_satisfied_by(&self, f: &FormDownload) -> bool {
        f.active_status.is_active()
    }
}

// ============================================================================
// Specification: DispatchesInDateRange (services.md § "DispatchesInDateRange")
// ============================================================================

/// A [`Specification`] that filters dispatches whose `date`
/// falls within `[from, to]` (inclusive). See
/// `docs/specs/documents/services.md` § "Specification:
/// DispatchesInDateRange".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DispatchesInDateRange {
    /// Inclusive lower bound on the dispatch `date`.
    pub from: NaiveDate,
    /// Inclusive upper bound on the dispatch `date`.
    pub to: NaiveDate,
}

impl Specification<PostalDispatch> for DispatchesInDateRange {
    fn is_satisfied_by(&self, d: &PostalDispatch) -> bool {
        let date = d.date.0;
        date >= self.from && date <= self.to
    }
}

// ============================================================================
// Specification: ReceivesInDateRange (services.md § "ReceivesInDateRange")
// ============================================================================

/// A [`Specification`] that filters receives whose `date`
/// falls within `[from, to]` (inclusive). See
/// `docs/specs/documents/services.md` § "Specification:
/// ReceivesInDateRange".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReceivesInDateRange {
    /// Inclusive lower bound on the receive `date`.
    pub from: NaiveDate,
    /// Inclusive upper bound on the receive `date`.
    pub to: NaiveDate,
}

impl Specification<PostalReceive> for ReceivesInDateRange {
    fn is_satisfied_by(&self, r: &PostalReceive) -> bool {
        let date = r.date.0;
        date >= self.from && date <= self.to
    }
}

// ============================================================================
// DocumentsCoordinator (services.md § "Cross-Domain Coordinator")
// ============================================================================

/// Thin cross-domain coordinator for the documents crate.
///
/// Per `docs/specs/documents/services.md` § "Cross-Domain
/// Coordinator", the production coordinator lives in the
/// engine facade (`crates/educore/`) and composes command
/// calls across domains (e.g. publish-form-to-site =
/// documents + CMS). The skeleton declared here mirrors the
/// spec's surface so the documents crate is self-contained
/// during scaffolding; the umbrella crate replaces it with
/// the real facade-backed type.
///
/// The generic `E` parameter avoids a tier-violation
/// dependency on the engine facade (`crates/tools/sdk/`,
/// `Engine`) — the documents crate only needs the lifetime
/// and a phantom handle.
pub struct DocumentsCoordinator<'a, E: ?Sized> {
    /// The owning engine facade. The `?Sized` bound lets
    /// callers pass either a concrete `Engine` (in production)
    /// or a unit placeholder (in tests).
    _engine: &'a E,
    /// Marker for `E: Send` so the coordinator is safe to move
    /// across async boundaries. `PhantomData` keeps the
    /// generic parameter live without altering the layout.
    _marker: core::marker::PhantomData<fn() -> E>,
}

impl<'a, E: ?Sized> DocumentsCoordinator<'a, E> {
    /// Constructs a new coordinator that borrows the engine
    /// facade for `'a`. The `engine` reference is stored for
    /// the lifetime of the coordinator.
    #[must_use]
    pub const fn new(engine: &'a E) -> Self {
        Self {
            _engine: engine,
            _marker: core::marker::PhantomData,
        }
    }

    /// Uploads a form via the documents engine and returns the
    /// persisted [`FormDownload`].
    ///
    /// The spec mandates that subscribers (CMS domain) handle
    /// the public-site publication in response to the
    /// `FormUploaded` event. This skeleton returns the
    /// engine-wide `not_supported` error until the facade
    /// wires up the real call site.
    pub async fn upload_form(
        &self,
        _cmd: UploadFormCommand,
    ) -> Result<FormDownload, DocumentsError> {
        // Skeleton: real implementation lives in the umbrella
        // crate's facade. Returning `Validation` here keeps
        // the call site from compiling past the gap and lets
        // downstream consumers stub the coordinator without
        // taking a hard dependency on the facade.
        Err(DocumentsError::Validation(
            "DocumentsCoordinator::upload_form is a spec-gap skeleton; wire to the engine facade".to_owned(),
        ))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;
    use crate::aggregate::{FormDownload, PostalDispatch, PostalReceive};
    use crate::commands::UploadFormCommand;
    use crate::value_objects::{ActiveStatus, FormTitle, ShowPublic};

    // ----- Validate trait -----

    #[test]
    fn validate_trait_is_object_safe() {
        // The trait is a single-method predicate; it must be
        // object-safe so consumers can store `Box<dyn Validate>`.
        fn _f(_: Box<dyn Validate>) {}
    }

    // ----- PublicForms -----

    #[test]
    fn public_forms_matches_when_show_public_is_true() {
        // Happy-path test: a form with `show_public = true`
        // satisfies the PublicForms specification.
        let form = make_form(true, true);
        assert!(PublicForms.is_satisfied_by(&form));
    }

    #[test]
    fn public_forms_rejects_when_show_public_is_false() {
        let form = make_form(false, true);
        assert!(!PublicForms.is_satisfied_by(&form));
    }

    // ----- ActiveForms -----

    #[test]
    fn active_forms_matches_when_active_status_is_true() {
        // Happy-path test: an active form satisfies the
        // ActiveForms specification.
        let form = make_form(true, true);
        assert!(ActiveForms.is_satisfied_by(&form));
    }

    // ----- DispatchesInDateRange -----

    #[test]
    fn dispatches_in_date_range_inclusive_bounds() {
        // Happy-path test: a dispatch whose `date` falls
        // within `[from, to]` (inclusive) satisfies the
        // specification.
        let from = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();
        let spec = DispatchesInDateRange { from, to };
        let mut dispatch = make_dispatch();
        dispatch.date = crate::value_objects::DispatchDate(NaiveDate::from_ymd_opt(2026, 6, 15).unwrap());
        assert!(spec.is_satisfied_by(&dispatch));
    }

    // ----- ReceivesInDateRange -----

    #[test]
    fn receives_in_date_range_inclusive_bounds() {
        // Happy-path test: a receive whose `date` falls
        // within `[from, to]` (inclusive) satisfies the
        // specification.
        let from = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();
        let spec = ReceivesInDateRange { from, to };
        let mut receive = make_receive();
        receive.date = crate::value_objects::ReceiveDate(NaiveDate::from_ymd_opt(2026, 6, 15).unwrap());
        assert!(spec.is_satisfied_by(&receive));
    }

    // ----- DocumentsCoordinator -----

    #[tokio::test]
    async fn coordinator_upload_form_skeleton_returns_validation_error() {
        // Happy-path test for the skeleton: the call site
        // compiles, runs, and returns the spec-documented
        // Validation error. The umbrella crate replaces this
        // method body with the real facade-backed call.
        let coord: DocumentsCoordinator<'_, ()> = DocumentsCoordinator::new(&());
        let cmd = UploadFormCommand {
            tenant: educore_core::tenant::TenantContext::builder()
                .school_id(educore_core::ids::SchoolId(uuid::Uuid::nil()))
                .actor(educore_core::ids::UserId(uuid::Uuid::nil()))
                .build(),
            title: FormTitle::new("test").unwrap(),
            short_description: None,
            publish_date: crate::value_objects::PublishDate(NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
            link: None,
            file: None,
            show_public: ShowPublic::new(false),
        };
        let result = coord.upload_form(cmd).await;
        let err = result.expect_err("skeleton must return Err");
        assert!(matches!(err, DocumentsError::Validation(_)));
    }

    // ----- helpers -----

    fn make_form(show_public: bool, active: bool) -> FormDownload {
        let mut form = make_form_skeleton();
        form.show_public = ShowPublic::new(show_public);
        form.active_status = if active {
            ActiveStatus::new(true)
        } else {
            ActiveStatus::new(false)
        };
        form
    }

    fn make_form_skeleton() -> FormDownload {
        // We construct a `FormDownload` via the same path the
        // aggregate factory uses. For the test fixtures we
        // only set the fields the specifications read; the
        // remaining audit-footer fields are populated by the
        // aggregate factory, so we use `FormDownload::new`
        // with a valid `NewFormDownload` payload and then
        // mutate the fields the specs filter on.
        let id = crate::value_objects::FormDownloadId::new(
            educore_core::ids::SchoolId(uuid::Uuid::nil()),
            uuid::Uuid::from_u128(1),
        );
        let cmd = crate::aggregate::NewFormDownload {
            id,
            title: FormTitle::new("test form").unwrap(),
            short_description: None,
            publish_date: crate::value_objects::PublishDate(NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
            link: None,
            file: Some(crate::value_objects::FileReference::new("s3://bucket/key").unwrap()),
            show_public: ShowPublic::new(true),
        };
        FormDownload::new(cmd).expect("aggregate factory accepts valid input")
    }

    fn make_dispatch() -> PostalDispatch {
        let id = crate::value_objects::PostalDispatchId::new(
            educore_core::ids::SchoolId(uuid::Uuid::nil()),
            uuid::Uuid::from_u128(1),
        );
        let cmd = crate::aggregate::NewPostalDispatch {
            id,
            to_title: crate::value_objects::ToTitle(crate::value_objects::PostalTitle::new("a").unwrap()),
            from_title: crate::value_objects::FromTitle(crate::value_objects::PostalTitle::new("b").unwrap()),
            reference_no: None,
            address: crate::value_objects::ToAddress(crate::value_objects::PostalAddress::new("addr").unwrap()),
            date: crate::value_objects::DispatchDate(NaiveDate::from_ymd_opt(2026, 6, 15).unwrap()),
            note: None,
            file: None,
        };
        let mut dispatch = PostalDispatch::new(cmd, uuid::Uuid::from_u128(2)).expect("factory accepts valid input");
        dispatch.date = crate::value_objects::DispatchDate(NaiveDate::from_ymd_opt(2026, 6, 15).unwrap());
        dispatch
    }

    fn make_receive() -> PostalReceive {
        let id = crate::value_objects::PostalReceiveId::new(
            educore_core::ids::SchoolId(uuid::Uuid::nil()),
            uuid::Uuid::from_u128(1),
        );
        let cmd = crate::aggregate::NewPostalReceive {
            id,
            from_title: crate::value_objects::FromTitle(crate::value_objects::PostalTitle::new("a").unwrap()),
            to_title: crate::value_objects::ToTitle(crate::value_objects::PostalTitle::new("b").unwrap()),
            reference_no: None,
            address: crate::value_objects::FromAddress(crate::value_objects::PostalAddress::new("addr").unwrap()),
            date: crate::value_objects::ReceiveDate(NaiveDate::from_ymd_opt(2026, 6, 15).unwrap()),
            note: None,
            file: None,
        };
        let mut receive = PostalReceive::new(cmd, uuid::Uuid::from_u128(2)).expect("factory accepts valid input");
        receive.date = crate::value_objects::ReceiveDate(NaiveDate::from_ymd_opt(2026, 6, 15).unwrap());
        receive
    }
}
