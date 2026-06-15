//! # Communication domain errors

#![allow(missing_docs)]
#![allow(unused_imports)]

use educore_core::error::DomainError;
use thiserror::Error;

/// The communication-domain error surface. Most signals are
/// `DomainError` values; the enum exists to anchor the
/// communication-domain typed error vocabulary and to provide a
/// stable `is_*` API for the dispatcher.
#[derive(Debug, Error)]
pub enum CommunicationError {
    /// A domain invariant was violated (e.g. anonymous complaint with identity).
    #[error("communication validation: {0}")]
    Validation(String),
    /// A state-machine conflict (e.g. publishing an already-published notice).
    #[error("communication conflict: {0}")]
    Conflict(String),
    /// A referenced resource was not found.
    #[error("communication not found: {0}")]
    NotFound(String),
    /// The actor lacks the required capability.
    #[error("communication forbidden: {0}")]
    Forbidden(String),
    /// A template body has unresolved `{{var}}` placeholders.
    #[error("communication template render: {0}")]
    TemplateRender(String),
    /// No active email/SMS gateway is configured for the requested channel.
    #[error("communication channel unavailable: {0}")]
    ChannelUnavailable(String),
    /// A notification setting is misconfigured.
    #[error("communication setting misconfigured: {0}")]
    SettingMisconfigured(String),
    /// Anonymous complaint requires no `complaint_by` or `phone` to be set.
    #[error("anonymous complaint requires no identity: {0}")]
    AnonymousRequiresNoIdentity(String),
    /// Notice cannot be deleted because recipients have already received it.
    #[error("notice has recipients: {0}")]
    NoticeHasRecipients(String),
    /// Sender is blocked by recipient (or vice versa).
    #[error("chat blocked: {0}")]
    ChatBlocked(String),
    /// Generic catch-all.
    #[error("communication: {0}")]
    Other(String),
}

impl CommunicationError {
    /// Wrap a `Validation` signal into a `DomainError`.
    #[inline]
    pub fn validation(msg: impl Into<String>) -> DomainError {
        DomainError::Validation(msg.into())
    }

    /// Wrap a `Conflict` signal into a `DomainError`.
    #[inline]
    pub fn conflict(msg: impl Into<String>) -> DomainError {
        DomainError::Conflict(msg.into())
    }

    /// Wrap a `NotFound` signal into a `DomainError`.
    #[inline]
    pub fn not_found(msg: impl Into<String>) -> DomainError {
        DomainError::NotFound(msg.into())
    }

    /// Wrap a `Forbidden` signal into a `DomainError`.
    #[inline]
    pub fn forbidden(msg: impl Into<String>) -> DomainError {
        DomainError::Forbidden(msg.into())
    }

    /// Wrap a `TemplateRender` signal into a `DomainError::Validation`.
    #[inline]
    pub fn template_render(msg: impl Into<String>) -> DomainError {
        DomainError::Validation(format!("template render: {}", msg.into()))
    }

    /// Wrap a `ChannelUnavailable` signal into a `DomainError::Conflict`.
    #[inline]
    pub fn channel_unavailable(msg: impl Into<String>) -> DomainError {
        DomainError::Conflict(format!("channel unavailable: {}", msg.into()))
    }

    /// Wrap a `SettingMisconfigured` signal into a `DomainError::Validation`.
    #[inline]
    pub fn setting_misconfigured(msg: impl Into<String>) -> DomainError {
        DomainError::Validation(format!("setting misconfigured: {}", msg.into()))
    }

    /// Wrap an `AnonymousRequiresNoIdentity` signal into a `DomainError::Validation`.
    #[inline]
    pub fn anonymous_requires_no_identity(msg: impl Into<String>) -> DomainError {
        DomainError::Validation(format!("anonymous requires no identity: {}", msg.into()))
    }

    /// Wrap a `NoticeHasRecipients` signal into a `DomainError::Conflict`.
    #[inline]
    pub fn notice_has_recipients(msg: impl Into<String>) -> DomainError {
        DomainError::Conflict(format!("notice has recipients: {}", msg.into()))
    }

    /// Wrap a `ChatBlocked` signal into a `DomainError::Conflict`.
    #[inline]
    pub fn chat_blocked(msg: impl Into<String>) -> DomainError {
        DomainError::Conflict(format!("chat blocked: {}", msg.into()))
    }

    /// Wrap a generic `Other` signal into a `DomainError`.
    #[inline]
    pub fn other(msg: impl Into<String>) -> DomainError {
        DomainError::Validation(msg.into())
    }
}
