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
