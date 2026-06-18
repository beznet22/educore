//! Documents-domain typed query stubs.

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

// === FormDownload query section begin (owner: 3A) ===

use serde::{Deserialize, Serialize};

use crate::value_objects::{ActiveStatus, FormTitle, PublishDate, ShowPublic};

/// Typed query builder for `FormDownload`. Mirrors the
/// `StudentField`/`StudentQuery` pattern in Phase 3.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FormDownloadQuery {
    /// Filter by title (exact match).
    pub title: Option<FormTitle>,
    /// Filter by show_public flag.
    pub show_public: Option<ShowPublic>,
    /// Filter by publish_date >= from.
    pub publish_from: Option<PublishDate>,
    /// Filter by publish_date <= to.
    pub publish_to: Option<PublishDate>,
    /// Filter by active_status (default true).
    pub active_status: Option<ActiveStatus>,
}

impl FormDownloadQuery {
    /// Construct a new empty query (returns all active forms).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by title.
    #[must_use]
    pub fn with_title(mut self, title: FormTitle) -> Self {
        self.title = Some(title);
        self
    }

    /// Filter by show_public.
    #[must_use]
    pub fn with_show_public(mut self, sp: ShowPublic) -> Self {
        self.show_public = Some(sp);
        self
    }

    /// Filter by publish_date in [from, to].
    #[must_use]
    pub fn with_publish_range(mut self, from: PublishDate, to: PublishDate) -> Self {
        self.publish_from = Some(from);
        self.publish_to = Some(to);
        self
    }

    /// Filter by active_status.
    #[must_use]
    pub fn with_active(mut self, active: ActiveStatus) -> Self {
        self.active_status = Some(active);
        self
    }
}

// === FormDownload query section end ===

// === PostalDispatch query section begin (owner: 3B) ===

// 3A above already imports `serde::{Deserialize, Serialize}`,
// `crate::value_objects::{ActiveStatus, ...}`, and 3C below
// imports `crate::aggregate::AcademicYearId`. Re-importing
// any of them here is an E0252 duplicate. The
// `DispatchDate`, `FromTitle`, `PostalReferenceNo`, and
// `ToTitle` types below are all new to this section.

use crate::value_objects::{DispatchDate, FromTitle, PostalReferenceNo, ToTitle};

/// Typed query builder for [`PostalDispatch`](crate::aggregate::PostalDispatch).
///
/// Mirrors the engine-wide pattern: every field is `Option`,
/// and `with_*` builder methods set the field and return
/// `self` for chaining. The default `active_status` is `None`,
/// which the storage adapter treats as "active only"; callers
/// that need archived rows must explicitly call
/// [`with_active`](Self::with_active) with
/// `ActiveStatus::new(false)`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PostalDispatchQuery {
    /// Filter by recipient's title.
    pub to_title: Option<ToTitle>,
    /// Filter by sender's title.
    pub from_title: Option<FromTitle>,
    /// Filter by reference number.
    pub reference_no: Option<PostalReferenceNo>,
    /// Filter by dispatch date `>= from`.
    pub date_from: Option<DispatchDate>,
    /// Filter by dispatch date `<= to`.
    pub date_to: Option<DispatchDate>,
    /// Filter by academic year.
    pub academic_id: Option<AcademicYearId>,
    /// Filter by active_status (default `None` = "active only").
    pub active_status: Option<ActiveStatus>,
}

impl PostalDispatchQuery {
    /// Constructs a new `PostalDispatchQuery` with all
    /// filters set to `None`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the recipient-title filter.
    #[must_use]
    pub fn with_to_title(mut self, t: ToTitle) -> Self {
        self.to_title = Some(t);
        self
    }

    /// Set the sender-title filter.
    #[must_use]
    pub fn with_from_title(mut self, t: FromTitle) -> Self {
        self.from_title = Some(t);
        self
    }

    /// Set the reference-number filter.
    #[must_use]
    pub fn with_reference(mut self, r: PostalReferenceNo) -> Self {
        self.reference_no = Some(r);
        self
    }

    /// Set the date-range filter (inclusive on both ends).
    #[must_use]
    pub fn with_date_range(mut self, from: DispatchDate, to: DispatchDate) -> Self {
        self.date_from = Some(from);
        self.date_to = Some(to);
        self
    }

    /// Set the academic-year filter.
    #[must_use]
    pub fn with_academic_year(mut self, year: AcademicYearId) -> Self {
        self.academic_id = Some(year);
        self
    }

    /// Set the active-status filter. Pass
    /// `ActiveStatus::new(false)` to surface archived rows;
    /// the default `None` is treated as "active only" by the
    /// storage adapter.
    #[must_use]
    pub fn with_active(mut self, a: ActiveStatus) -> Self {
        self.active_status = Some(a);
        self
    }
}

// === PostalDispatch query section end ===

// === PostalReceive query section begin (owner: 3C) ===

// 3A above already imports `serde::{Deserialize, Serialize}`
// and `crate::value_objects::ActiveStatus`. 3B above imports
// `FromTitle`, `PostalReferenceNo`, and `ToTitle` from
// `crate::value_objects`. Re-importing any of them here is an
// E0252 duplicate. The `AcademicYearId` and `ReceiveDate`
// types below are all new to this section.

use crate::aggregate::AcademicYearId;
use crate::value_objects::ReceiveDate;

/// Typed query builder for `PostalReceive`. Mirrors the
/// `FormDownloadQuery` pattern in Phase 11/3A. The storage
/// adapter translates each `Some` filter into a typed AST
/// node; an all-`None` query returns every active receive for
/// the school.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PostalReceiveQuery {
    /// Filter by from_title (exact match).
    pub from_title: Option<FromTitle>,
    /// Filter by to_title (exact match).
    pub to_title: Option<ToTitle>,
    /// Filter by reference_no (exact match).
    pub reference_no: Option<PostalReferenceNo>,
    /// Filter by receive_date >= from.
    pub date_from: Option<ReceiveDate>,
    /// Filter by receive_date <= to.
    pub date_to: Option<ReceiveDate>,
    /// Filter by academic year scope.
    pub academic_id: Option<AcademicYearId>,
    /// Filter by active_status (default true for ordinary
    /// reads; `Some(ActiveStatus::new(false))` for archived).
    pub active_status: Option<ActiveStatus>,
}

impl PostalReceiveQuery {
    /// Construct a new empty query (returns all active
    /// receives for the school).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by from_title.
    #[must_use]
    pub fn with_from_title(mut self, t: FromTitle) -> Self {
        self.from_title = Some(t);
        self
    }

    /// Filter by to_title.
    #[must_use]
    pub fn with_to_title(mut self, t: ToTitle) -> Self {
        self.to_title = Some(t);
        self
    }

    /// Filter by reference_no.
    #[must_use]
    pub fn with_reference(mut self, r: PostalReferenceNo) -> Self {
        self.reference_no = Some(r);
        self
    }

    /// Filter by receive_date in the inclusive range
    /// `[from, to]`.
    #[must_use]
    pub fn with_date_range(mut self, from: ReceiveDate, to: ReceiveDate) -> Self {
        self.date_from = Some(from);
        self.date_to = Some(to);
        self
    }

    /// Filter by academic year scope.
    #[must_use]
    pub fn with_academic_year(mut self, year: AcademicYearId) -> Self {
        self.academic_id = Some(year);
        self
    }

    /// Filter by active_status.
    #[must_use]
    pub fn with_active(mut self, a: ActiveStatus) -> Self {
        self.active_status = Some(a);
        self
    }
}

// === PostalReceive query section end ===

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

    // ---- FormDownloadQuery ----

    #[test]
    fn form_download_query_default_is_empty() {
        let q = FormDownloadQuery::default();
        assert!(q.title.is_none());
        assert!(q.show_public.is_none());
        assert!(q.publish_from.is_none());
        assert!(q.publish_to.is_none());
        assert!(q.active_status.is_none());
    }

    #[test]
    fn form_download_query_new_is_equivalent_to_default() {
        let a = FormDownloadQuery::new();
        let b = FormDownloadQuery::default();
        assert_eq!(a, b);
    }

    #[test]
    fn form_download_query_with_title_accumulates_filter() {
        let q = FormDownloadQuery::new()
            .with_title(crate::value_objects::FormTitle::new("Consent Form").unwrap());
        assert!(q.title.is_some());
        assert_eq!(q.title.as_ref().unwrap().as_str(), "Consent Form");
    }

    #[test]
    fn form_download_query_with_show_public_accumulates_filter() {
        let q =
            FormDownloadQuery::new().with_show_public(crate::value_objects::ShowPublic::new(true));
        assert!(q.show_public.is_some());
        assert!(q.show_public.unwrap().is_public());
    }

    #[test]
    fn form_download_query_with_publish_range_accumulates_filters() {
        let from = crate::value_objects::PublishDate::new(
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        );
        let to = crate::value_objects::PublishDate::new(
            chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        );
        let q = FormDownloadQuery::new().with_publish_range(from, to);
        assert!(q.publish_from.is_some());
        assert!(q.publish_to.is_some());
    }

    #[test]
    fn form_download_query_with_active_accumulates_filter() {
        let q =
            FormDownloadQuery::new().with_active(crate::value_objects::ActiveStatus::new(false));
        assert!(q.active_status.is_some());
        assert!(!q.active_status.unwrap().is_active());
    }

    #[test]
    fn form_download_query_chained_builders_accumulate_all_filters() {
        let q = FormDownloadQuery::new()
            .with_title(crate::value_objects::FormTitle::new("X").unwrap())
            .with_show_public(crate::value_objects::ShowPublic::new(true))
            .with_publish_range(
                crate::value_objects::PublishDate::new(
                    chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                ),
                crate::value_objects::PublishDate::new(
                    chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
                ),
            )
            .with_active(crate::value_objects::ActiveStatus::new(true));
        assert!(q.title.is_some());
        assert!(q.show_public.is_some());
        assert!(q.publish_from.is_some());
        assert!(q.publish_to.is_some());
        assert!(q.active_status.is_some());
    }

    // ---- PostalDispatchQuery ----

    #[test]
    fn postal_dispatch_query_default_is_empty() {
        let q = PostalDispatchQuery::default();
        assert!(q.to_title.is_none());
        assert!(q.from_title.is_none());
        assert!(q.reference_no.is_none());
        assert!(q.date_from.is_none());
        assert!(q.date_to.is_none());
        assert!(q.academic_id.is_none());
        assert!(q.active_status.is_none());
    }

    #[test]
    fn postal_dispatch_query_with_to_title_accumulates_filter() {
        let q = PostalDispatchQuery::new().with_to_title(crate::value_objects::ToTitle::new(
            crate::value_objects::PostalTitle::new("X").unwrap(),
        ));
        assert!(q.to_title.is_some());
    }

    #[test]
    fn postal_dispatch_query_with_reference_accumulates_filter() {
        let q = PostalDispatchQuery::new()
            .with_reference(crate::value_objects::PostalReferenceNo::new("REF-001").unwrap());
        assert!(q.reference_no.is_some());
    }

    #[test]
    fn postal_dispatch_query_with_date_range_accumulates_filters() {
        let from = crate::value_objects::DispatchDate::new(
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        );
        let to = crate::value_objects::DispatchDate::new(
            chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        );
        let q = PostalDispatchQuery::new().with_date_range(from, to);
        assert!(q.date_from.is_some());
        assert!(q.date_to.is_some());
    }

    #[test]
    fn postal_dispatch_query_with_academic_year_accumulates_filter() {
        let year = uuid::Uuid::now_v7();
        let q = PostalDispatchQuery::new().with_academic_year(year);
        assert_eq!(q.academic_id, Some(year));
    }

    #[test]
    fn postal_dispatch_query_with_active_accumulates_filter() {
        let q =
            PostalDispatchQuery::new().with_active(crate::value_objects::ActiveStatus::new(false));
        assert!(q.active_status.is_some());
        assert!(!q.active_status.unwrap().is_active());
    }

    // ---- PostalReceiveQuery ----

    #[test]
    fn postal_receive_query_default_is_empty() {
        let q = PostalReceiveQuery::default();
        assert!(q.from_title.is_none());
        assert!(q.to_title.is_none());
        assert!(q.reference_no.is_none());
        assert!(q.date_from.is_none());
        assert!(q.date_to.is_none());
        assert!(q.academic_id.is_none());
        assert!(q.active_status.is_none());
    }

    #[test]
    fn postal_receive_query_with_from_title_accumulates_filter() {
        let q = PostalReceiveQuery::new().with_from_title(crate::value_objects::FromTitle::new(
            crate::value_objects::PostalTitle::new("X").unwrap(),
        ));
        assert!(q.from_title.is_some());
    }

    #[test]
    fn postal_receive_query_with_to_title_accumulates_filter() {
        let q = PostalReceiveQuery::new().with_to_title(crate::value_objects::ToTitle::new(
            crate::value_objects::PostalTitle::new("Y").unwrap(),
        ));
        assert!(q.to_title.is_some());
    }

    #[test]
    fn postal_receive_query_with_reference_accumulates_filter() {
        let q = PostalReceiveQuery::new()
            .with_reference(crate::value_objects::PostalReferenceNo::new("REF-001").unwrap());
        assert!(q.reference_no.is_some());
    }

    #[test]
    fn postal_receive_query_with_date_range_accumulates_filters() {
        let from = crate::value_objects::ReceiveDate::new(
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        );
        let to = crate::value_objects::ReceiveDate::new(
            chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        );
        let q = PostalReceiveQuery::new().with_date_range(from, to);
        assert!(q.date_from.is_some());
        assert!(q.date_to.is_some());
    }

    #[test]
    fn postal_receive_query_with_academic_year_accumulates_filter() {
        let year = uuid::Uuid::now_v7();
        let q = PostalReceiveQuery::new().with_academic_year(year);
        assert_eq!(q.academic_id, Some(year));
    }

    #[test]
    fn postal_receive_query_with_active_accumulates_filter() {
        let q =
            PostalReceiveQuery::new().with_active(crate::value_objects::ActiveStatus::new(false));
        assert!(q.active_status.is_some());
        assert!(!q.active_status.unwrap().is_active());
    }
}
