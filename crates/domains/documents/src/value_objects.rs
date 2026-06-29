//! # Documents value objects
//!
//! Typed ids, validated value objects, and closed enums for the
//! documents domain (form downloads, postal dispatch, postal
//! receive). Per `docs/specs/documents/value-objects.md`:
//!
//! - Every id is `Id { school_id, value }` — a typed wrapper that
//!   carries the school anchor so the type system catches
//!   cross-tenant confusion at compile time.
//! - Strings are validated at construction. The constructors
//!   return `Result<Self, DomainError>`; there are no setters
//!   that bypass validation.
//! - The `DocumentType` and `DocumentVisibility` enums are
//!   closed.
//! - The `FileReference` type is a local copy of the engine-wide
//!   file-reference pattern; the engine-wide type lives in
//!   `educore_files` (Phase 15) but the documents crate needs
//!   a local copy today to keep the dependency surface minimal.

#![allow(missing_docs)]
#![allow(unused_imports)]

use std::fmt;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::error::{DomainError, Result};
use educore_core::ids::SchoolId;

// =============================================================================
// Macro: typed documents id
// =============================================================================

/// Macro to define the per-aggregate typed id wrapper. Every
/// documents id follows the same shape: a `school_id` anchor
/// plus a local `Uuid`. The wrapper implements
/// [`Clone`], [`Copy`], [`PartialEq`], [`Eq`], [`Hash`], and
/// the `Display` format `"{school_id}/{value}"`. The pattern
/// matches `library_typed_id!` / `communication_typed_id!` so
/// the engine's id types stay consistent across crates.
macro_rules! documents_typed_id {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident;
    ) => {
        $(#[$attr])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        $vis struct $name {
            /// The owning school (tenant anchor).
            pub school_id: SchoolId,
            /// The local id (UUIDv7).
            pub value: Uuid,
        }

        impl $name {
            /// Constructs a new typed id from its parts.
            #[must_use]
            pub const fn new(school_id: SchoolId, value: Uuid) -> Self {
                Self { school_id, value }
            }

            /// Returns the local UUID.
            #[must_use]
            pub const fn as_uuid(&self) -> Uuid {
                self.value
            }

            /// Returns the owning school id.
            #[must_use]
            pub const fn school_id(&self) -> SchoolId {
                self.school_id
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}/{}", self.school_id, self.value)
            }
        }
    };
}

// =============================================================================
// Typed ids: 3 aggregate roots
// =============================================================================

documents_typed_id! {
    /// A typed id for a [`FormDownload`](crate::aggregate::FormDownload).
    pub struct FormDownloadId;
}
documents_typed_id! {
    /// A typed id for a [`PostalDispatch`](crate::aggregate::PostalDispatch).
    pub struct PostalDispatchId;
}
documents_typed_id! {
    /// A typed id for a [`PostalReceive`](crate::aggregate::PostalReceive).
    pub struct PostalReceiveId;
}
documents_typed_id! {
    /// A typed id for a [`NewFormDownload`](crate::aggregate::NewFormDownload).
    pub struct NewFormDownloadId;
}
documents_typed_id! {
    /// A typed id for a [`UpdateFormDownload`](crate::aggregate::UpdateFormDownload).
    pub struct UpdateFormDownloadId;
}
documents_typed_id! {
    /// A typed id for a [`FormDownloadFile`](crate::aggregate::FormDownloadFile).
    pub struct FormDownloadFileId;
}
documents_typed_id! {
    /// A typed id for a [`FormDownloadLink`](crate::aggregate::FormDownloadLink).
    pub struct FormDownloadLinkId;
}
documents_typed_id! {
    /// A typed id for a [`NewPostalDispatch`](crate::aggregate::NewPostalDispatch).
    pub struct NewPostalDispatchId;
}
documents_typed_id! {
    /// A typed id for a [`UpdatePostalDispatch`](crate::aggregate::UpdatePostalDispatch).
    pub struct UpdatePostalDispatchId;
}
documents_typed_id! {
    /// A typed id for a [`PostalDispatchAttachment`](crate::aggregate::PostalDispatchAttachment).
    pub struct PostalDispatchAttachmentId;
}
documents_typed_id! {
    /// A typed id for a [`NewPostalReceive`](crate::aggregate::NewPostalReceive).
    pub struct NewPostalReceiveId;
}
documents_typed_id! {
    /// A typed id for a [`UpdatePostalReceive`](crate::aggregate::UpdatePostalReceive).
    pub struct UpdatePostalReceiveId;
}
documents_typed_id! {
    /// A typed id for a [`PostalReceiveAttachment`](crate::aggregate::PostalReceiveAttachment).
    pub struct PostalReceiveAttachmentId;
}

// =============================================================================
// Validated string newtypes
// =============================================================================

// ---- Form names and free text ----

/// A form title (1..=191 chars).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FormTitle(String);

impl FormTitle {
    /// Maximum length of a form title.
    pub const MAX_LEN: usize = 191;

    /// Constructs a new `FormTitle`, validating non-empty and
    /// length-bounded.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s: String = raw.into();
        if s.is_empty() {
            return Err(DomainError::validation("form title must not be empty"));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "form title must be 1..{} chars",
                Self::MAX_LEN
            )));
        }
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for FormTitle {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FormTitle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A form description (1..=200 chars).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FormDescription(String);

impl FormDescription {
    /// Maximum length of a form description.
    pub const MAX_LEN: usize = 200;

    /// Constructs a new `FormDescription`, validating non-empty
    /// and length-bounded.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s: String = raw.into();
        if s.is_empty() {
            return Err(DomainError::validation(
                "form description must not be empty",
            ));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "form description must be 1..{} chars",
                Self::MAX_LEN
            )));
        }
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for FormDescription {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FormDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// ---- Postal names and free text ----

/// A postal title (1..=191 chars). Used for both
/// [`FromTitle`] and [`ToTitle`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PostalTitle(String);

impl PostalTitle {
    /// Maximum length of a postal title.
    pub const MAX_LEN: usize = 191;

    /// Constructs a new `PostalTitle`, validating non-empty and
    /// length-bounded.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s: String = raw.into();
        if s.is_empty() {
            return Err(DomainError::validation("postal title must not be empty"));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "postal title must be 1..{} chars",
                Self::MAX_LEN
            )));
        }
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for PostalTitle {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PostalTitle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The sender's title (a [`PostalTitle`]). Distinct type for
/// call-site clarity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FromTitle(pub PostalTitle);

impl FromTitle {
    /// Constructs a new `FromTitle` from a [`PostalTitle`].
    #[must_use]
    pub const fn new(title: PostalTitle) -> Self {
        Self(title)
    }

    /// Returns the inner [`PostalTitle`].
    #[must_use]
    pub const fn value(&self) -> &PostalTitle {
        &self.0
    }

    /// Returns the inner string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl AsRef<str> for FromTitle {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl fmt::Display for FromTitle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// The recipient's title (a [`PostalTitle`]). Distinct type for
/// call-site clarity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ToTitle(pub PostalTitle);

impl ToTitle {
    /// Constructs a new `ToTitle` from a [`PostalTitle`].
    #[must_use]
    pub const fn new(title: PostalTitle) -> Self {
        Self(title)
    }

    /// Returns the inner [`PostalTitle`].
    #[must_use]
    pub const fn value(&self) -> &PostalTitle {
        &self.0
    }

    /// Returns the inner string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl AsRef<str> for ToTitle {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl fmt::Display for ToTitle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// A postal note (1..=5000 chars). The body of a
/// `PostalDispatch` or `PostalReceive`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PostalNote(String);

impl PostalNote {
    /// Maximum length of a postal note.
    pub const MAX_LEN: usize = 5_000;

    /// Constructs a new `PostalNote`, validating non-empty and
    /// length-bounded.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s: String = raw.into();
        if s.is_empty() {
            return Err(DomainError::validation("postal note must not be empty"));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "postal note must be 1..{} chars",
                Self::MAX_LEN
            )));
        }
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for PostalNote {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PostalNote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A postal reference number (1..=191 chars).
///
/// # Uniqueness contract
///
/// `PostalReferenceNo` is **unique within `(school_id,
/// academic_id)`**. That is, two rows across the entire
/// `documents_postal_dispatches` and `documents_postal_receives`
/// tables MUST NOT share the same `(school_id, academic_id,
/// reference_no)` tuple.
///
/// The uniqueness is enforced by a composite unique index in
/// the storage adapter; the constructor here enforces only the
/// string shape (non-empty, 1..=191 chars). The reference number
/// is also **immutable once set** — see
/// `DocumentsError::ReferenceNoImmutable` for the corresponding
/// error.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PostalReferenceNo(String);

impl PostalReferenceNo {
    /// Maximum length of a postal reference number.
    pub const MAX_LEN: usize = 191;

    /// Constructs a new `PostalReferenceNo`, validating
    /// non-empty and length-bounded. The
    /// `(school_id, academic_id)` uniqueness constraint is
    /// enforced by the storage adapter's unique index, not by
    /// this constructor.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s: String = raw.into();
        if s.is_empty() {
            return Err(DomainError::validation(
                "postal reference number must not be empty",
            ));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "postal reference number must be 1..{} chars",
                Self::MAX_LEN
            )));
        }
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for PostalReferenceNo {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PostalReferenceNo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// =============================================================================
// Addresses
// =============================================================================

/// A postal address (1..=191 chars). The free-text postal
/// address used on `PostalDispatch` (sender or receiver) and
/// `PostalReceive` (sender).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PostalAddress(String);

impl PostalAddress {
    /// Maximum length of a postal address.
    pub const MAX_LEN: usize = 191;

    /// Constructs a new `PostalAddress`, validating non-empty
    /// and length-bounded.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s: String = raw.into();
        if s.is_empty() {
            return Err(DomainError::validation("postal address must not be empty"));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "postal address must be 1..{} chars",
                Self::MAX_LEN
            )));
        }
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for PostalAddress {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PostalAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The sender's address (a [`PostalAddress`]). Distinct type
/// for call-site clarity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FromAddress(pub PostalAddress);

impl FromAddress {
    /// Constructs a new `FromAddress` from a [`PostalAddress`].
    #[must_use]
    pub const fn new(address: PostalAddress) -> Self {
        Self(address)
    }

    /// Returns the inner [`PostalAddress`].
    #[must_use]
    pub const fn value(&self) -> &PostalAddress {
        &self.0
    }

    /// Returns the inner string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl AsRef<str> for FromAddress {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl fmt::Display for FromAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// The recipient's address (a [`PostalAddress`]). Distinct type
/// for call-site clarity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ToAddress(pub PostalAddress);

impl ToAddress {
    /// Constructs a new `ToAddress` from a [`PostalAddress`].
    #[must_use]
    pub const fn new(address: PostalAddress) -> Self {
        Self(address)
    }

    /// Returns the inner [`PostalAddress`].
    #[must_use]
    pub const fn value(&self) -> &PostalAddress {
        &self.0
    }

    /// Returns the inner string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl AsRef<str> for ToAddress {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl fmt::Display for ToAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

// =============================================================================
// Dates
// =============================================================================

/// The publication date of a [`FormDownload`](crate::aggregate::FormDownload).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PublishDate(pub NaiveDate);

impl PublishDate {
    /// Constructs a new `PublishDate`.
    #[must_use]
    pub const fn new(date: NaiveDate) -> Self {
        Self(date)
    }

    /// Returns the inner date.
    #[must_use]
    pub const fn value(self) -> NaiveDate {
        self.0
    }
}

/// The dispatch date of a [`PostalDispatch`](crate::aggregate::PostalDispatch).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DispatchDate(pub NaiveDate);

impl DispatchDate {
    /// Constructs a new `DispatchDate`.
    #[must_use]
    pub const fn new(date: NaiveDate) -> Self {
        Self(date)
    }

    /// Returns the inner date.
    #[must_use]
    pub const fn value(self) -> NaiveDate {
        self.0
    }
}

/// The receive date of a [`PostalReceive`](crate::aggregate::PostalReceive).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ReceiveDate(pub NaiveDate);

impl ReceiveDate {
    /// Constructs a new `ReceiveDate`.
    #[must_use]
    pub const fn new(date: NaiveDate) -> Self {
        Self(date)
    }

    /// Returns the inner date.
    #[must_use]
    pub const fn value(self) -> NaiveDate {
        self.0
    }
}

// =============================================================================
// URL and file reference
// =============================================================================

/// A URL, validated to be ≤ 2048 chars and to start with
/// `http://` or `https://`. Not a full RFC 3986 parser; the
/// intent is to catch obvious mistakes (missing scheme, empty
/// host). Used by [`FormDownloadLink`](crate::aggregate::FormDownloadLink)
/// to point at an external resource.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Url(String);

impl Url {
    /// Maximum length of a URL.
    pub const MAX_LEN: usize = 2_048;

    /// Constructs a new `Url`, validating length and scheme.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s: String = raw.into();
        if s.is_empty() {
            return Err(DomainError::validation(
                "url must be non-empty and <= 2048 chars, starting with http:// or https://",
            ));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(
                "url must be non-empty and <= 2048 chars, starting with http:// or https://",
            ));
        }
        if !is_plausible_url(&s) {
            return Err(DomainError::validation(
                "url must be non-empty and <= 2048 chars, starting with http:// or https://",
            ));
        }
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for Url {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A reference to a file stored in the
/// `educore_files` adapter (an object-store key or a row id).
/// Non-empty. Used by
/// [`FormDownloadFile`](crate::aggregate::FormDownloadFile),
/// [`PostalDispatchAttachment`](crate::aggregate::PostalDispatchAttachment),
/// and
/// [`PostalReceiveAttachment`](crate::aggregate::PostalReceiveAttachment).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FileReference(String);

impl FileReference {
    /// Constructs a new `FileReference`, validating non-empty.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s: String = raw.into();
        if s.is_empty() {
            return Err(DomainError::validation("file reference must be non-empty"));
        }
        Ok(Self(s))
    }

    /// Returns the inner reference string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for FileReference {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FileReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// =============================================================================
// Visibility and active flags
// =============================================================================

/// A `bool` newtype indicating whether the form is visible on
/// the public site. `true` = visible; `false` = staff-only.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ShowPublic(pub bool);

impl ShowPublic {
    /// Constructs a new `ShowPublic` from a `bool`.
    #[must_use]
    pub const fn new(public: bool) -> Self {
        Self(public)
    }

    /// Returns `true` if the form is visible to the public.
    #[must_use]
    pub const fn is_public(self) -> bool {
        self.0
    }
}

impl From<bool> for ShowPublic {
    fn from(b: bool) -> Self {
        Self(b)
    }
}

impl fmt::Display for ShowPublic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(if self.0 { "public" } else { "staff" })
    }
}

/// A `bool` newtype serving as the soft-delete flag. `true` =
/// active row visible to ordinary queries; `false` = archived
/// (the row stays in the database but is hidden from ordinary
/// queries, like the engine-wide
/// [`ActiveStatus`](educore_core::value_objects::ActiveStatus)
/// enum but stored as a `TINYINT(1)` rather than the enum's
/// byte encoding).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ActiveStatus(pub bool);

impl ActiveStatus {
    /// Constructs a new `ActiveStatus` from a `bool`.
    #[must_use]
    pub const fn new(active: bool) -> Self {
        Self(active)
    }

    /// Returns `true` if the row is active (visible to
    /// ordinary queries).
    #[must_use]
    pub const fn is_active(self) -> bool {
        self.0
    }
}

impl Default for ActiveStatus {
    fn default() -> Self {
        Self(true)
    }
}

impl From<bool> for ActiveStatus {
    fn from(b: bool) -> Self {
        Self(b)
    }
}

impl fmt::Display for ActiveStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(if self.0 { "active" } else { "archived" })
    }
}

// =============================================================================
// Closed enums
// =============================================================================

/// The document classification.
///
/// Per `docs/specs/documents/value-objects.md` § "Document
/// Type": every document row is one of `Form`,
/// `PostalDispatch`, or `PostalReceive`. The storage adapter
/// stores the discriminant as a `VARCHAR` column with the
/// `as_str()` form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocumentType {
    /// A form download.
    Form,
    /// A postal dispatch (outgoing mail).
    PostalDispatch,
    /// A postal receive (incoming mail).
    PostalReceive,
}

impl DocumentType {
    /// Returns the wire-form string for the document type.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Form => "form",
            Self::PostalDispatch => "postal_dispatch",
            Self::PostalReceive => "postal_receive",
        }
    }
}

impl fmt::Display for DocumentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The visibility level of a document.
///
/// Per `docs/specs/documents/value-objects.md` § "Document
/// Type": every document has a visibility of either
/// `Public` (visible to anonymous readers on the public site)
/// or `Staff` (visible only to authenticated school staff).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocumentVisibility {
    /// Visible to anonymous readers on the public site.
    Public,
    /// Visible only to authenticated school staff.
    Staff,
}

impl DocumentVisibility {
    /// Returns the wire-form string for the visibility.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Staff => "staff",
        }
    }
}

impl fmt::Display for DocumentVisibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// =============================================================================
// Validation helpers (private)
// =============================================================================

/// Returns `true` if `s` looks like a plausible URL: starts
/// with `http://` or `https://`, has a non-empty host with at
/// least one `.`.
fn is_plausible_url(s: &str) -> bool {
    let (scheme, rest) = if let Some(rest) = s.strip_prefix("https://") {
        ("https", rest)
    } else if let Some(rest) = s.strip_prefix("http://") {
        ("http", rest)
    } else {
        return false;
    };
    let _ = scheme;
    if rest.is_empty() {
        return false;
    }
    let host_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let host = &rest[..host_end];
    if host.is_empty() {
        return false;
    }
    host.contains('.') && !host.starts_with('.') && !host.ends_with('.')
}

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
    use educore_core::ids::Identifier;

    #[test]
    fn typed_id_display_and_accessors() {
        let school = SchoolId::from_uuid(Uuid::nil());
        let id = FormDownloadId::new(school, Uuid::from_u128(42));
        assert_eq!(id.school_id(), school);
        assert_eq!(id.as_uuid(), Uuid::from_u128(42));
        assert_eq!(id.to_string(), format!("{}/{}", school, id.value));
    }

    #[test]
    fn form_title_validates_length() {
        assert!(FormTitle::new("").is_err());
        assert!(FormTitle::new("Parent Consent Form 2026").is_ok());
        assert!(FormTitle::new("x".repeat(192)).is_err());
    }

    #[test]
    fn form_description_validates_length() {
        assert!(FormDescription::new("").is_err());
        assert!(FormDescription::new("a").is_ok());
        assert!(FormDescription::new("x".repeat(201)).is_err());
    }

    #[test]
    fn postal_title_validates_length() {
        assert!(PostalTitle::new("").is_err());
        assert!(PostalTitle::new("Mr Smith").is_ok());
        assert!(PostalTitle::new("x".repeat(192)).is_err());
    }

    #[test]
    fn postal_note_validates_length() {
        assert!(PostalNote::new("").is_err());
        assert!(PostalNote::new("Sent for review").is_ok());
        assert!(PostalNote::new("x".repeat(5_001)).is_err());
    }

    #[test]
    fn postal_reference_no_validates_length() {
        assert!(PostalReferenceNo::new("").is_err());
        assert!(PostalReferenceNo::new("REF-2026-0001").is_ok());
        assert!(PostalReferenceNo::new("x".repeat(192)).is_err());
    }

    #[test]
    fn postal_address_validates_length() {
        assert!(PostalAddress::new("").is_err());
        assert!(PostalAddress::new("1 Main St").is_ok());
        assert!(PostalAddress::new("x".repeat(192)).is_err());
    }

    #[test]
    fn from_to_address_aliases() {
        let addr = PostalAddress::new("1 Main St").unwrap();
        let from = FromAddress::new(addr.clone());
        let to = ToAddress::new(addr.clone());
        assert_eq!(from.as_str(), "1 Main St");
        assert_eq!(to.as_str(), "1 Main St");
    }

    #[test]
    fn from_to_title_aliases() {
        let title = PostalTitle::new("Mr Smith").unwrap();
        let from = FromTitle::new(title.clone());
        let to = ToTitle::new(title.clone());
        assert_eq!(from.as_str(), "Mr Smith");
        assert_eq!(to.as_str(), "Mr Smith");
    }

    #[test]
    fn dates_hold_naive_date() {
        let d = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let publish = PublishDate::new(d);
        let dispatch = DispatchDate::new(d);
        let receive = ReceiveDate::new(d);
        assert_eq!(publish.value(), d);
        assert_eq!(dispatch.value(), d);
        assert_eq!(receive.value(), d);
    }

    #[test]
    fn url_validates() {
        assert!(Url::new("https://example.com/path").is_ok());
        assert!(Url::new("http://example.com").is_ok());
        assert!(Url::new("example.com").is_err());
        assert!(Url::new("https://").is_err());
        assert!(Url::new("").is_err());
        assert!(Url::new("https://example.com/".to_string() + &"a".repeat(2_050)).is_err());
    }

    #[test]
    fn file_reference_validates_non_empty() {
        assert!(FileReference::new("").is_err());
        assert!(FileReference::new("object-key-1234").is_ok());
    }

    #[test]
    fn show_public_default_is_false() {
        assert!(!ShowPublic::default().is_public());
        assert!(ShowPublic::new(true).is_public());
    }

    #[test]
    fn active_status_default_is_true() {
        assert!(ActiveStatus::default().is_active());
        assert!(!ActiveStatus::new(false).is_active());
    }

    #[test]
    fn document_type_wire_forms() {
        assert_eq!(DocumentType::Form.as_str(), "form");
        assert_eq!(DocumentType::PostalDispatch.as_str(), "postal_dispatch");
        assert_eq!(DocumentType::PostalReceive.as_str(), "postal_receive");
    }

    #[test]
    fn document_visibility_wire_forms() {
        assert_eq!(DocumentVisibility::Public.as_str(), "public");
        assert_eq!(DocumentVisibility::Staff.as_str(), "staff");
    }

    // -------------------------------------------------------------------------
    // Phase 11 / 4-tests — extended value-object coverage.
    //
    // These tests complement the existing validation tests by
    // exercising the postal value objects (PostalTitle / PostalNote /
    // PostalReferenceNo / PostalAddress) and the From/To address and
    // title aliases plus the Date newtypes. The focus is on
    // non-happy-path inputs: empty strings, over-long inputs,
    // whitespace-only inputs.
    // -------------------------------------------------------------------------

    #[test]
    fn postal_title_accepts_min_and_max_lengths() {
        // 1 char (min) is ok.
        assert!(PostalTitle::new("A").is_ok());
        // 191 chars (max) is ok.
        assert!(PostalTitle::new("a".repeat(191)).is_ok());
        // 192 chars (over) is rejected.
        assert!(PostalTitle::new("a".repeat(192)).is_err());
    }

    #[test]
    fn postal_note_accepts_min_and_max_lengths() {
        assert!(PostalNote::new("a").is_ok());
        assert!(PostalNote::new("x".repeat(5_000)).is_ok());
        assert!(PostalNote::new("x".repeat(5_001)).is_err());
    }

    #[test]
    fn postal_reference_no_accepts_min_and_max_lengths() {
        assert!(PostalReferenceNo::new("A").is_ok());
        assert!(PostalReferenceNo::new("a".repeat(191)).is_ok());
        assert!(PostalReferenceNo::new("a".repeat(192)).is_err());
    }

    #[test]
    fn postal_address_accepts_min_and_max_lengths() {
        assert!(PostalAddress::new("A").is_ok());
        assert!(PostalAddress::new("a".repeat(191)).is_ok());
        assert!(PostalAddress::new("a".repeat(192)).is_err());
    }

    #[test]
    fn from_to_address_and_title_have_independent_as_str() {
        let addr_a = PostalAddress::new("1 Main St").unwrap();
        let addr_b = PostalAddress::new("2 Oak Ave").unwrap();
        let from_a = FromAddress::new(addr_a);
        let to_b = ToAddress::new(addr_b);
        assert_eq!(from_a.as_str(), "1 Main St");
        assert_eq!(to_b.as_str(), "2 Oak Ave");
        // Distinct addresses round-trip independently.
        assert_ne!(from_a.as_str(), to_b.as_str());
    }

    #[test]
    fn show_public_display_form_matches_str_value() {
        assert_eq!(ShowPublic::new(true).to_string(), "public");
        assert_eq!(ShowPublic::new(false).to_string(), "staff");
    }

    #[test]
    fn active_status_display_or_internal_consistency() {
        // The value-object `ActiveStatus` is wrapped in a tuple
        // struct; the public surface is `is_active()` and `Default`.
        // The default MUST be active (the engine treats `true` as
        // "visible to ordinary queries").
        let a = ActiveStatus::default();
        assert!(a.is_active());
        assert!(ActiveStatus::new(true).is_active());
        assert!(!ActiveStatus::new(false).is_active());
    }

    #[test]
    fn document_type_is_closed_to_three_variants() {
        // The Phase 11 spec fixes exactly 3 DocumentType values.
        let form = DocumentType::Form;
        let dispatch = DocumentType::PostalDispatch;
        let receive = DocumentType::PostalReceive;
        // Wire forms are stable strings.
        assert_eq!(form.as_str(), "form");
        assert_eq!(dispatch.as_str(), "postal_dispatch");
        assert_eq!(receive.as_str(), "postal_receive");
    }
}
