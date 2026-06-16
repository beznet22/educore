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
pub struct PostalDispatchQuery;
// === PostalDispatch query section end ===

// === PostalReceive query section begin (owner: 3C) ===
pub struct PostalReceiveQuery;
// === PostalReceive query section end ===
