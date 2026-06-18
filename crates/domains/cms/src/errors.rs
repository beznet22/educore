//! CMS-domain error type.

use thiserror::Error;

/// Convenient `Result` alias scoped to the CMS crate.
pub type Result<T> = core::result::Result<T, CmsError>;

/// CMS-domain error type. All fallible operations in the CMS
/// crate return `Result<T, CmsError>`.
#[derive(Debug, Error)]
pub enum CmsError {
    /// Generic validation failure (e.g. malformed value object).
    #[error("validation: {0}")]
    Validation(String),

    /// A page with the given slug already exists for the school
    /// (invariant 2 of `Page`).
    #[error("page slug '{0}' already exists for school {1}")]
    DuplicateSlug(String, educore_core::ids::SchoolId),

    /// A `home_page = true` page already exists for the school
    /// (invariant 4 of `Page`: at most one per school).
    #[error("home page already set for school {0}")]
    HomePageAlreadySet(educore_core::ids::SchoolId),

    /// The page is `is_default = true` and cannot be deleted
    /// (invariant 5 of `Page`).
    #[error("default page {0} cannot be deleted")]
    DefaultPageNotDeletable(uuid::Uuid),

    /// A `Page` is required to have a non-empty `title` (invariant
    /// 1 of `Page`).
    #[error("page title must not be empty")]
    PageTitleEmpty,

    /// A `News` is required to have a non-empty `news_title`
    /// (invariant 1 of `News`).
    #[error("news title must not be empty")]
    NewsTitleEmpty,

    /// A `NewsComment` is required to have a non-empty `message`
    /// (invariant 2 of `NewsComment`).
    #[error("comment message must not be empty")]
    CommentMessageEmpty,

    /// The news has `is_comment = false`; commenting is disabled.
    #[error("news {0} has is_comment = false; commenting is disabled")]
    CommentingDisabled(uuid::Uuid),

    /// A `ContentShareList` has `valid_upto < share_date`
    /// (invariant 3 of `ContentShareList`).
    #[error("valid_upto must be on or after share_date")]
    ContentShareListInvalidWindow,

    /// A `ContentShareList` is not in `Draft` status, so it
    /// cannot be cancelled.
    #[error("content share list {0} is not in Draft status; cannot be cancelled")]
    ContentShareListNotCancellable(uuid::Uuid),

    /// A `ContentShareList` is not in `Draft` status, so it
    /// cannot be dispatched.
    #[error("content share list {0} is not in Draft status; cannot be dispatched")]
    ContentShareListNotDispatchable(uuid::Uuid),

    /// A `CoursePage` child references a parent id that does not
    /// exist or is in a different school.
    #[error("course page parent {0} not found")]
    CoursePageParentNotFound(uuid::Uuid),

    /// The caller is not authorized to perform the operation.
    #[error("forbidden: {0}")]
    Forbidden(String),

    /// The operation conflicts with the current state of a
    /// domain resource (unique key, state machine, etc.).
    #[error("conflict: {0}")]
    Conflict(String),

    /// An infrastructure adapter (storage, event bus, files)
    /// reported a failure.
    #[error("infrastructure: {0}")]
    Infrastructure(String),

    /// Catch-all variant for wrapped errors that do not map to a
    /// CMS-specific case.
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl From<educore_core::error::DomainError> for CmsError {
    fn from(err: educore_core::error::DomainError) -> Self {
        match err {
            educore_core::error::DomainError::Forbidden(msg)
            | educore_core::error::DomainError::TenantViolation(msg) => CmsError::Forbidden(msg),
            educore_core::error::DomainError::Conflict(msg) => CmsError::Conflict(msg),
            educore_core::error::DomainError::Validation(msg)
            | educore_core::error::DomainError::NotFound(msg) => CmsError::Validation(msg),
            educore_core::error::DomainError::NotSupported(msg) => CmsError::Validation(msg),
            educore_core::error::DomainError::Infrastructure(src) => {
                CmsError::Infrastructure(src.to_string())
            }
        }
    }
}

impl From<educore_events::errors::EventError> for CmsError {
    fn from(err: educore_events::errors::EventError) -> Self {
        CmsError::Infrastructure(err.to_string())
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    #![allow(clippy::unnecessary_literal_unwrap)]
    #![allow(unused_imports)]
    #![allow(dead_code)]
    use super::*;

    #[test]
    fn from_domain_error_forbidden_maps_to_cms_forbidden() {
        let src = educore_core::error::DomainError::Forbidden("nope".to_owned());
        let dst: CmsError = src.into();
        assert!(matches!(dst, CmsError::Forbidden(_)));
    }

    #[test]
    fn from_domain_error_conflict_maps_to_cms_conflict() {
        let src = educore_core::error::DomainError::Conflict("dup".to_owned());
        let dst: CmsError = src.into();
        assert!(matches!(dst, CmsError::Conflict(_)));
    }

    #[test]
    fn from_domain_error_validation_maps_to_cms_validation() {
        let src = educore_core::error::DomainError::Validation("bad".to_owned());
        let dst: CmsError = src.into();
        assert!(matches!(dst, CmsError::Validation(_)));
    }

    #[test]
    fn from_event_error_maps_to_cms_infrastructure() {
        let src = educore_events::errors::EventError::PublishFailed("down".to_owned());
        let dst: CmsError = src.into();
        assert!(matches!(dst, CmsError::Infrastructure(_)));
    }

    #[test]
    fn page_title_empty_display_is_stable() {
        let msg = CmsError::PageTitleEmpty.to_string();
        assert_eq!(msg, "page title must not be empty");
    }

    #[test]
    fn news_title_empty_display_is_stable() {
        let msg = CmsError::NewsTitleEmpty.to_string();
        assert_eq!(msg, "news title must not be empty");
    }

    #[test]
    fn comment_message_empty_display_is_stable() {
        let msg = CmsError::CommentMessageEmpty.to_string();
        assert_eq!(msg, "comment message must not be empty");
    }

    #[test]
    fn commenting_disabled_carries_news_id() {
        let id = uuid::Uuid::now_v7();
        let err = CmsError::CommentingDisabled(id);
        assert!(err.to_string().contains(&id.to_string()));
    }

    #[test]
    fn duplicate_slug_carries_payload() {
        let s = educore_core::ids::SchoolId(uuid::Uuid::now_v7());
        let err = CmsError::DuplicateSlug("my-page".to_owned(), s);
        let msg = err.to_string();
        assert!(msg.contains("my-page"));
    }

    #[test]
    fn home_page_already_set_carries_school() {
        let s = educore_core::ids::SchoolId(uuid::Uuid::now_v7());
        let err = CmsError::HomePageAlreadySet(s);
        let msg = err.to_string();
        assert!(msg.contains("home page"));
    }

    #[test]
    fn default_page_not_deletable_carries_id() {
        let id = uuid::Uuid::now_v7();
        let err = CmsError::DefaultPageNotDeletable(id);
        let msg = err.to_string();
        assert!(msg.contains(&id.to_string()));
    }

    #[test]
    fn content_share_list_invalid_window_display_is_stable() {
        let msg = CmsError::ContentShareListInvalidWindow.to_string();
        assert!(msg.contains("valid_upto"));
    }

    #[test]
    fn content_share_list_not_dispatchable_carries_id() {
        let id = uuid::Uuid::now_v7();
        let err = CmsError::ContentShareListNotDispatchable(id);
        let msg = err.to_string();
        assert!(msg.contains(&id.to_string()));
    }

    #[test]
    fn content_share_list_not_cancellable_carries_id() {
        let id = uuid::Uuid::now_v7();
        let err = CmsError::ContentShareListNotCancellable(id);
        let msg = err.to_string();
        assert!(msg.contains(&id.to_string()));
    }

    #[test]
    fn course_page_parent_not_found_carries_id() {
        let id = uuid::Uuid::now_v7();
        let err = CmsError::CoursePageParentNotFound(id);
        let msg = err.to_string();
        assert!(msg.contains(&id.to_string()));
    }

    #[test]
    fn forbidden_carries_message() {
        let err = CmsError::Forbidden("nope".to_owned());
        assert_eq!(err.to_string(), "forbidden: nope");
    }

    #[test]
    fn conflict_carries_message() {
        let err = CmsError::Conflict("stale".to_owned());
        assert_eq!(err.to_string(), "conflict: stale");
    }

    #[test]
    fn validation_carries_message() {
        let err = CmsError::Validation("bad input".to_owned());
        assert_eq!(err.to_string(), "validation: bad input");
    }

    #[test]
    fn infrastructure_carries_message() {
        let err = CmsError::Infrastructure("bus down".to_owned());
        assert_eq!(err.to_string(), "infrastructure: bus down");
    }

    #[test]
    fn other_wraps_arbitrary_error() {
        let inner: Box<dyn std::error::Error + Send + Sync> = std::io::Error::other("disk").into();
        let err: CmsError = inner.into();
        assert!(matches!(err, CmsError::Other(_)));
    }

    #[test]
    fn result_alias_is_standard_result() {
        let r: Result<u32> = Ok(7);
        assert_eq!(r.unwrap(), 7);
    }
}
