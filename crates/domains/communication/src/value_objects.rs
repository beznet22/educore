//! # Communication value objects
//!
//! The typed ids (one per aggregate), the validated value objects,
//! the embedded value-object lists, and the closed enums the
//! communication aggregates depend on. Per
//! `docs/specs/communication/value-objects.md`:
//!
//! - Every id is `Id { school_id, value }` — a typed wrapper that
//!   carries the school anchor so the type system catches
//!   cross-tenant confusion at compile time.
//! - Strings are validated at construction. The constructors
//!   return `Result<Self, DomainError>`; there are no setters
//!   that bypass validation.
//! - The `Destination` bitflag is a typed wrapper around a `u8`,
//!   with `contains()` for membership tests and `as_str()`
//!   returning comma-joined single-letter codes (`"E"`, `"S"`,
//!   `"W"`, `"A"`, `"E,S"`, ...).
//! - Status enums are closed.
//!
//! Foreign-key typed ids (`StudentId`, `StaffId`, `RoleId`,
//! `ClassId`, `SectionId`, `SubjectId`, `AcademicYearId`) are
//! **re-exported** from [`educore_academic`] and [`educore_hr`].
//! `MessageId` (the cross-domain email/sms message identifier)
//! is defined locally as a single-field newtype because
//! `educore_events::event_bus` does not export one.

#![allow(missing_docs)]
#![allow(unused_imports)]

use std::fmt;
use std::ops::BitOr;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_academic::{AcademicYearId, ClassId, SectionId, StudentId, SubjectId};
use educore_core::error::{DomainError, Result};
use educore_core::ids::{Identifier, SchoolId, UserId};
use educore_hr::value_objects::{RoleId, StaffId};

// =============================================================================
// Macro: typed communication id
// =============================================================================

/// Macro to define the per-aggregate typed id wrapper. Every
/// communication id follows the same shape: a `school_id` anchor
/// plus a local `Uuid`. The wrapper implements
/// [`Clone`], [`Copy`], [`PartialEq`], [`Eq`], [`Hash`], and
/// the `Display` format `"{school_id}/{value}"`. The pattern
/// matches `library_typed_id!` so the engine's id types stay
/// consistent across crates.
macro_rules! communication_typed_id {
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
// Typed ids: 26 aggregate roots
// =============================================================================

communication_typed_id! {
    /// A typed id for a [`Notice`](crate::aggregate::Notice).
    pub struct NoticeId;
}
communication_typed_id! {
    /// A typed id for a [`Complaint`](crate::aggregate::Complaint).
    pub struct ComplaintId;
}
communication_typed_id! {
    /// A typed id for a [`ComplaintType`](crate::aggregate::ComplaintType).
    pub struct ComplaintTypeId;
}
communication_typed_id! {
    /// A typed id for a [`Notification`](crate::aggregate::Notification).
    pub struct NotificationId;
}
communication_typed_id! {
    /// A typed id for an [`EmailLog`](crate::aggregate::EmailLog) row.
    pub struct EmailLogId;
}
communication_typed_id! {
    /// A typed id for an [`SmsLog`](crate::aggregate::SmsLog) row.
    pub struct SmsLogId;
}
communication_typed_id! {
    /// A typed id for an [`SmsTemplate`](crate::aggregate::SmsTemplate).
    pub struct SmsTemplateId;
}
communication_typed_id! {
    /// A typed id for an [`EmailSetting`](crate::aggregate::EmailSetting).
    pub struct EmailSettingId;
}
communication_typed_id! {
    /// A typed id for an [`SmsGateway`](crate::aggregate::SmsGateway).
    pub struct SmsGatewayId;
}
communication_typed_id! {
    /// A typed id for a [`NotificationSetting`](crate::aggregate::NotificationSetting).
    pub struct NotificationSettingId;
}
communication_typed_id! {
    /// A typed id for an [`AbsentNotificationTimeSetup`](crate::aggregate::AbsentNotificationTimeSetup).
    pub struct AbsentNotificationTimeSetupId;
}
communication_typed_id! {
    /// A typed id for a [`ChatMessage`](crate::aggregate::ChatMessage).
    pub struct ChatMessageId;
}
communication_typed_id! {
    /// A typed id for a [`ChatConversation`](crate::aggregate::ChatConversation).
    pub struct ChatConversationId;
}
communication_typed_id! {
    /// A typed id for a [`ChatGroup`](crate::aggregate::ChatGroup).
    pub struct ChatGroupId;
}
communication_typed_id! {
    /// A typed id for a [`ChatGroupUser`](crate::aggregate::ChatGroupUser) membership row.
    pub struct ChatGroupUserId;
}
communication_typed_id! {
    /// A typed id for a [`ChatGroupMessageRecipient`](crate::aggregate::ChatGroupMessageRecipient) row.
    pub struct ChatGroupMessageRecipientId;
}
communication_typed_id! {
    /// A typed id for a [`ChatGroupMessageRemove`](crate::aggregate::ChatGroupMessageRemove) row.
    pub struct ChatGroupMessageRemoveId;
}
communication_typed_id! {
    /// A typed id for a [`ChatBlockUser`](crate::aggregate::ChatBlockUser) row.
    pub struct ChatBlockUserId;
}
communication_typed_id! {
    /// A typed id for a [`ChatInvitation`](crate::aggregate::ChatInvitation).
    pub struct ChatInvitationId;
}
communication_typed_id! {
    /// A typed id for a [`ChatInvitationType`](crate::aggregate::ChatInvitationType) row.
    pub struct ChatInvitationTypeId;
}
communication_typed_id! {
    /// A typed id for a [`ChatStatus`](crate::aggregate::ChatStatus) row.
    pub struct ChatStatusId;
}
communication_typed_id! {
    /// A typed id for a [`SendMessage`](crate::aggregate::SendMessage).
    pub struct SendMessageId;
}
communication_typed_id! {
    /// A typed id for a [`ContactMessage`](crate::aggregate::ContactMessage).
    pub struct ContactMessageId;
}
communication_typed_id! {
    /// A typed id for a [`SpeechSlider`](crate::aggregate::SpeechSlider).
    pub struct SpeechSliderId;
}
communication_typed_id! {
    /// A typed id for a [`PhoneCallLog`](crate::aggregate::PhoneCallLog).
    pub struct PhoneCallLogId;
}
communication_typed_id! {
    /// A typed id for a [`CustomSmsSetting`](crate::aggregate::CustomSmsSetting).
    pub struct CustomSmsSettingId;
}

// =============================================================================
// Typed ids: 11 child entities
// =============================================================================

communication_typed_id! {
    /// A typed id for a [`NoticeAttachment`](crate::entities::NoticeAttachment).
    pub struct NoticeAttachmentId;
}
communication_typed_id! {
    /// A typed id for a [`ComplaintNote`](crate::entities::ComplaintNote).
    pub struct ComplaintNoteId;
}
communication_typed_id! {
    /// A typed id for a [`NotificationDeliveryAttempt`](crate::entities::NotificationDeliveryAttempt).
    pub struct NotificationDeliveryAttemptId;
}
communication_typed_id! {
    /// A typed id for a [`ChatGroupAvatar`](crate::entities::ChatGroupAvatar).
    pub struct ChatGroupAvatarId;
}
communication_typed_id! {
    /// A typed id for a [`ChatGroupMessage`](crate::entities::ChatGroupMessage).
    pub struct ChatGroupMessageId;
}
communication_typed_id! {
    /// A typed id for a [`ChatConversationLastRead`](crate::entities::ChatConversationLastRead).
    pub struct ChatConversationLastReadId;
}
communication_typed_id! {
    /// A typed id for a [`SendMessageRecipient`](crate::entities::SendMessageRecipient).
    pub struct SendMessageRecipientId;
}
communication_typed_id! {
    /// A typed id for an [`EmailSettingSecret`](crate::entities::EmailSettingSecret).
    pub struct EmailSettingSecretId;
}
communication_typed_id! {
    /// A typed id for an [`SmsGatewayCredential`](crate::entities::SmsGatewayCredential).
    pub struct SmsGatewayCredentialId;
}
communication_typed_id! {
    /// A typed id for an [`AbsentNotificationDispatch`](crate::entities::AbsentNotificationDispatch).
    pub struct AbsentNotificationDispatchId;
}
communication_typed_id! {
    /// A typed id for a [`ContactMessageReply`](crate::entities::ContactMessageReply).
    pub struct ContactMessageReplyId;
}

// =============================================================================
// Closed enums (27) from manifest section 4
// =============================================================================

/// The classification of a notice — determines which audience
/// the notice reaches by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NoticeType {
    /// A school-wide general announcement.
    General,
    /// A notice scoped to a single class.
    Class,
    /// A notice targeted at a specific student.
    Student,
    /// A notice targeted at staff.
    Staff,
    /// A notice targeted at parents.
    Parent,
    /// A notice for an upcoming event.
    Event,
}

impl NoticeType {
    /// Returns the wire-form string for the notice type.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Class => "Class",
            Self::Student => "Student",
            Self::Staff => "Staff",
            Self::Parent => "Parent",
            Self::Event => "Event",
        }
    }
}

/// The publication state of a [`Notice`](crate::aggregate::Notice).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NoticeStatus {
    /// The notice is a draft and is not visible to readers.
    Draft,
    /// The notice is scheduled to publish on a future date.
    Scheduled,
    /// The notice is published and visible to its audience.
    Published,
    /// The notice was published and has since been withdrawn.
    Unpublished,
}

impl NoticeStatus {
    /// Returns the wire-form string for the notice status.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "Draft",
            Self::Scheduled => "Scheduled",
            Self::Published => "Published",
            Self::Unpublished => "Unpublished",
        }
    }
}

/// The lifecycle status of a [`Complaint`](crate::aggregate::Complaint).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComplaintStatus {
    /// The complaint is open and awaiting triage.
    Open,
    /// The complaint is being actively worked.
    InProgress,
    /// The complaint has been resolved.
    Resolved,
}

impl ComplaintStatus {
    /// Returns the wire-form string for the complaint status.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::InProgress => "InProgress",
            Self::Resolved => "Resolved",
        }
    }
}

/// The channel through which a complaint was reported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComplaintSource {
    /// Walk-in at reception.
    WalkIn,
    /// Reported by phone.
    Phone,
    /// Reported by email.
    Email,
    /// Reported via the school's web form.
    Web,
    /// Some other channel (catch-all).
    Other,
    /// Reported anonymously (no complainant on record).
    Anonymous,
}

impl ComplaintSource {
    /// Returns the wire-form string for the complaint source.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WalkIn => "WalkIn",
            Self::Phone => "Phone",
            Self::Email => "Email",
            Self::Web => "Web",
            Self::Other => "Other",
            Self::Anonymous => "Anonymous",
        }
    }
}

/// The visual severity of a [`Notification`](crate::aggregate::Notification).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationType {
    /// Informational.
    Info,
    /// A warning that something may need attention.
    Warning,
    /// A success message.
    Success,
    /// An error message.
    Error,
}

impl NotificationType {
    /// Returns the wire-form string for the notification type.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Info => "Info",
            Self::Warning => "Warning",
            Self::Success => "Success",
            Self::Error => "Error",
        }
    }
}

/// The delivery state of a [`Notification`](crate::aggregate::Notification).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationStatus {
    /// The notification is queued but not yet dispatched.
    Pending,
    /// The notification has been dispatched to a channel.
    Dispatched,
    /// The notification has been delivered to the recipient.
    Delivered,
    /// Delivery failed.
    Failed,
    /// The recipient has read the notification.
    Read,
    /// The notification was withdrawn before being read.
    Withdrawn,
}

impl NotificationStatus {
    /// Returns the wire-form string for the notification status.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Dispatched => "Dispatched",
            Self::Delivered => "Delivered",
            Self::Failed => "Failed",
            Self::Read => "Read",
            Self::Withdrawn => "Withdrawn",
        }
    }
}

/// A logical channel label. Multiple channels may be combined
/// into a single [`Destination`] bitflag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Channel {
    /// Email channel.
    Email,
    /// SMS channel.
    Sms,
    /// In-app web notification.
    Web,
    /// In-app mobile/desktop notification.
    App,
    /// Push notification.
    Push,
}

impl Channel {
    /// Returns the wire-form string for the channel.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Email => "Email",
            Self::Sms => "Sms",
            Self::Web => "Web",
            Self::App => "App",
            Self::Push => "Push",
        }
    }
}

/// A typed bitflag struct over the four delivery destinations
/// (email, sms, web, app). Use the `EMAIL` / `SMS` / `WEB` /
/// `APP` constants and the `contains()` method to compose a
/// route, and the `as_str()` method to render it as a
/// comma-joined single-letter code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Destination(u8);

impl Destination {
    /// Email destination flag.
    pub const EMAIL: Self = Self(0b0001);
    /// SMS destination flag.
    pub const SMS: Self = Self(0b0010);
    /// Web destination flag.
    pub const WEB: Self = Self(0b0100);
    /// App destination flag.
    pub const APP: Self = Self(0b1000);

    /// The empty destination (no channel).
    pub const EMPTY: Self = Self(0);

    /// Returns `true` if `self` contains every flag set in
    /// `other`.
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Returns the underlying bit pattern.
    #[must_use]
    pub const fn bits(self) -> u8 {
        self.0
    }

    /// Constructs a destination from a raw `u8` bit pattern.
    /// The bit pattern is masked to the four known flags.
    #[must_use]
    pub const fn from_bits_truncate(bits: u8) -> Self {
        Self(bits & 0b1111)
    }

    /// Returns the wire-form string for the destination. A
    /// single flag returns `"E"`, `"S"`, `"W"`, or `"A"`; a
    /// multi-flag value returns a comma-joined list in the
    /// canonical order (`"E,S,W,A"`). The empty destination
    /// returns `""`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self.0 {
            0b0000 => "",
            0b0001 => "E",
            0b0010 => "S",
            0b0100 => "W",
            0b1000 => "A",
            0b0011 => "E,S",
            0b0101 => "E,W",
            0b1001 => "E,A",
            0b0110 => "S,W",
            0b1010 => "S,A",
            0b1100 => "W,A",
            0b0111 => "E,S,W",
            0b1011 => "E,S,A",
            0b1101 => "E,W,A",
            0b1110 => "S,W,A",
            0b1111 => "E,S,W,A",
            _ => "",
        }
    }
}

impl BitOr for Destination {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// The kind of payload carried by a chat message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageType {
    /// Plain text body.
    Text,
    /// An image attachment.
    Image,
    /// A PDF document.
    Pdf,
    /// A generic document.
    Document,
    /// A voice note.
    Voice,
}

impl MessageType {
    /// Returns the wire-form string for the message type.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Text => "Text",
            Self::Image => "Image",
            Self::Pdf => "Pdf",
            Self::Document => "Document",
            Self::Voice => "Voice",
        }
    }
}

/// The kind of phone call logged in a
/// [`PhoneCallLog`](crate::aggregate::PhoneCallLog).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CallType {
    /// An incoming call.
    Incoming,
    /// An outgoing call.
    Outgoing,
    /// A missed call.
    Missed,
}

impl CallType {
    /// Returns the wire-form string for the call type.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Incoming => "Incoming",
            Self::Outgoing => "Outgoing",
            Self::Missed => "Missed",
        }
    }
}

/// The SMS gateway vendor. The variant determines which
/// [`SmsGatewayCredentials`] shape is required.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GatewayType {
    /// Clickatell.
    Clickatell,
    /// Twilio.
    Twilio,
    /// Msg91.
    Msg91,
    /// Textlocal.
    Textlocal,
    /// AfricaTalking.
    AfricaTalking,
    /// Custom HTTP gateway.
    Custom,
}

impl GatewayType {
    /// Returns the wire-form string for the gateway type.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Clickatell => "Clickatell",
            Self::Twilio => "Twilio",
            Self::Msg91 => "Msg91",
            Self::Textlocal => "Textlocal",
            Self::AfricaTalking => "AfricaTalking",
            Self::Custom => "Custom",
        }
    }
}

/// The encryption mode for an SMTP mail driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MailEncryption {
    /// No encryption (plaintext).
    None,
    /// TLS (implicit, e.g. port 465).
    Tls,
    /// STARTTLS (opportunistic, e.g. port 587).
    StartTls,
}

impl MailEncryption {
    /// Returns the wire-form string for the mail encryption.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Tls => "Tls",
            Self::StartTls => "StartTls",
        }
    }
}

/// The mail transport driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MailDriver {
    /// Standard SMTP.
    Smtp,
    /// Local sendmail binary.
    Sendmail,
    /// Mailgun HTTP API.
    Mailgun,
    /// Amazon SES.
    Ses,
    /// Postmark HTTP API.
    Postmark,
}

impl MailDriver {
    /// Returns the wire-form string for the mail driver.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Smtp => "Smtp",
            Self::Sendmail => "Sendmail",
            Self::Mailgun => "Mailgun",
            Self::Ses => "Ses",
            Self::Postmark => "Postmark",
        }
    }
}

/// The HTTP method for a custom SMS gateway.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RequestMethod {
    /// HTTP GET.
    Get,
    /// HTTP POST.
    Post,
}

impl RequestMethod {
    /// Returns the wire-form string for the request method.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Get => "Get",
            Self::Post => "Post",
        }
    }
}

/// The enabled/disabled state of an SMS template.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SmsTemplateStatus {
    /// The template is enabled and may be used.
    Enabled,
    /// The template is disabled and may not be used.
    Disabled,
}

impl SmsTemplateStatus {
    /// Returns the wire-form string for the template status.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Enabled => "Enabled",
            Self::Disabled => "Disabled",
        }
    }
}

/// The enabled/disabled state of an absent-notification
/// schedule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AbsentNotificationStatus {
    /// The schedule is enabled.
    Enabled,
    /// The schedule is disabled.
    Disabled,
}

impl AbsentNotificationStatus {
    /// Returns the wire-form string for the schedule status.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Enabled => "Enabled",
            Self::Disabled => "Disabled",
        }
    }
}

/// The privacy level of a chat group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChatGroupPrivacy {
    /// Anyone in the school can see and join.
    Public,
    /// Visible but join must be approved.
    Private,
    /// Tied to a class — only class members can join.
    Class,
}

impl ChatGroupPrivacy {
    /// Returns the wire-form string for the group privacy.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Public => "Public",
            Self::Private => "Private",
            Self::Class => "Class",
        }
    }
}

/// The join mode of a chat group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChatGroupType {
    /// Anyone in the school can join.
    Open,
    /// An admin must approve joins.
    Closed,
}

impl ChatGroupType {
    /// Returns the wire-form string for the group type.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::Closed => "Closed",
        }
    }
}

/// The role of a member within a chat group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChatGroupRole {
    /// A regular member.
    Member,
    /// An admin (can moderate).
    Admin,
}

impl ChatGroupRole {
    /// Returns the wire-form string for the group role.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Member => "Member",
            Self::Admin => "Admin",
        }
    }
}

/// The presence status of a chat user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChatStatus {
    /// The user is offline.
    Inactive,
    /// The user is online and reachable.
    Active,
    /// The user is online but away.
    Away,
    /// The user is online but marked busy.
    Busy,
}

impl ChatStatus {
    /// Returns the wire-form string for the chat status.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Inactive => "Inactive",
            Self::Active => "Active",
            Self::Away => "Away",
            Self::Busy => "Busy",
        }
    }
}

/// The state of a chat invitation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChatInvitationStatus {
    /// The invitation has been sent and is awaiting a response.
    Pending,
    /// The invitation has been accepted; the two parties are
    /// connected.
    Connected,
    /// The invitation has been rejected or the user is blocked.
    Blocked,
}

impl ChatInvitationStatus {
    /// Returns the wire-form string for the invitation status.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Connected => "Connected",
            Self::Blocked => "Blocked",
        }
    }
}

/// The classification of a chat invitation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChatInvitationTypeEnum {
    /// A direct (1:1) chat.
    OneToOne,
    /// A group chat.
    Group,
    /// A class-teacher chat (e.g. parent ↔ class teacher).
    ClassTeacher,
}

impl ChatInvitationTypeEnum {
    /// Returns the wire-form string for the invitation type.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OneToOne => "OneToOne",
            Self::Group => "Group",
            Self::ClassTeacher => "ClassTeacher",
        }
    }
}

/// The read state of a chat message by a recipient.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChatMessageStatus {
    /// The message has not been read.
    Unread,
    /// The message has been seen by the recipient.
    Seen,
}

impl ChatMessageStatus {
    /// Returns the wire-form string for the message status.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unread => "Unread",
            Self::Seen => "Seen",
        }
    }
}

/// The lifecycle status of a [`SendMessage`](crate::aggregate::SendMessage)
/// broadcast.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SendMessageStatus {
    /// The broadcast is a draft.
    Draft,
    /// The broadcast has been dispatched to its recipients.
    Dispatched,
    /// The broadcast was cancelled before dispatch.
    Cancelled,
    /// The broadcast has finished dispatching.
    Completed,
}

impl SendMessageStatus {
    /// Returns the wire-form string for the broadcast status.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "Draft",
            Self::Dispatched => "Dispatched",
            Self::Cancelled => "Cancelled",
            Self::Completed => "Completed",
        }
    }
}

/// The view state of a contact-form submission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContactMessageViewStatus {
    /// The message has not been viewed by staff.
    Unviewed,
    /// The message has been viewed by staff.
    Viewed,
}

impl ContactMessageViewStatus {
    /// Returns the wire-form string for the view status.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unviewed => "Unviewed",
            Self::Viewed => "Viewed",
        }
    }
}

/// The reply state of a contact-form submission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContactMessageReplyStatus {
    /// The message has not been replied to.
    Unreplied,
    /// The message has been replied to.
    Replied,
}

impl ContactMessageReplyStatus {
    /// Returns the wire-form string for the reply status.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unreplied => "Unreplied",
            Self::Replied => "Replied",
        }
    }
}

/// The action a complaint workflow can take. Mirrors the
/// `ComplaintStatus` transitions (open ↔ in-progress → resolve)
/// but as a verb-shaped enum for command dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComplaintAction {
    /// Move the complaint to the `Open` state.
    Open,
    /// Move the complaint to the `InProgress` state.
    InProgress,
    /// Resolve the complaint.
    Resolve,
}

impl ComplaintAction {
    /// Returns the wire-form string for the complaint action.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::InProgress => "InProgress",
            Self::Resolve => "Resolve",
        }
    }
}

// =============================================================================
// Validated value objects (32 from manifest section 3)
// =============================================================================

// ---- String newtypes (1..N chars) ----

/// A notice title (1..=200 chars).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NoticeTitle(String);

impl NoticeTitle {
    /// Maximum length of a notice title.
    pub const MAX_LEN: usize = 200;

    /// Constructs a new `NoticeTitle`, validating non-empty and
    /// length-bounded.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s: String = raw.into();
        if s.is_empty() {
            return Err(DomainError::validation("notice title must not be empty"));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "notice title must be 1..{} chars",
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

/// A notice body (1..=5000 chars).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NoticeBody(String);

impl NoticeBody {
    /// Maximum length of a notice body.
    pub const MAX_LEN: usize = 5_000;

    /// Constructs a new `NoticeBody`, validating non-empty and
    /// length-bounded.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s: String = raw.into();
        if s.is_empty() {
            return Err(DomainError::validation("notice body must not be empty"));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "notice body must be 1..{} chars",
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

/// A complaint description (1..=5000 chars).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ComplaintDescription(String);

impl ComplaintDescription {
    /// Maximum length of a complaint description.
    pub const MAX_LEN: usize = 5_000;

    /// Constructs a new `ComplaintDescription`, validating
    /// non-empty and length-bounded.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s: String = raw.into();
        if s.is_empty() {
            return Err(DomainError::validation(
                "complaint description must not be empty",
            ));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "complaint description must be 1..{} chars",
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

/// A speech-slider text (1..=5000 chars).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SpeechText(String);

impl SpeechText {
    /// Maximum length of a speech text.
    pub const MAX_LEN: usize = 5_000;

    /// Constructs a new `SpeechText`, validating non-empty and
    /// length-bounded.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s: String = raw.into();
        if s.is_empty() {
            return Err(DomainError::validation("speech text must not be empty"));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "speech text must be 1..{} chars",
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

/// A chat message body (1..=10000 chars).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ChatMessageBody(String);

impl ChatMessageBody {
    /// Maximum length of a chat message body.
    pub const MAX_LEN: usize = 10_000;

    /// Constructs a new `ChatMessageBody`, validating non-empty
    /// and length-bounded.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s: String = raw.into();
        if s.is_empty() {
            return Err(DomainError::validation(
                "chat message body must not be empty",
            ));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "chat message body must be 1..{} chars",
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

/// An SMS/email template body (1..=65535 chars). The upper
/// bound is the maximum length of a single SMS concatenated
/// (GSM-7 153 × n segments) and matches the maximum size of
/// common mail servers' body field.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TemplateBody(String);

impl TemplateBody {
    /// Maximum length of a template body.
    pub const MAX_LEN: usize = 65_535;

    /// Constructs a new `TemplateBody`, validating non-empty
    /// and length-bounded.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s: String = raw.into();
        if s.is_empty() {
            return Err(DomainError::validation("template body must not be empty"));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "template body must be 1..{} chars",
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

/// An email subject (1..=200 chars).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EmailSubject(String);

impl EmailSubject {
    /// Maximum length of an email subject.
    pub const MAX_LEN: usize = 200;

    /// Constructs a new `EmailSubject`, validating non-empty and
    /// length-bounded.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s: String = raw.into();
        if s.is_empty() {
            return Err(DomainError::validation(
                "email subject must not be empty",
            ));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "email subject must be 1..{} chars",
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

/// A call description / call notes (1..=5000 chars).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CallDescription(String);

impl CallDescription {
    /// Maximum length of a call description.
    pub const MAX_LEN: usize = 5_000;

    /// Constructs a new `CallDescription`, validating non-empty
    /// and length-bounded.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s: String = raw.into();
        if s.is_empty() {
            return Err(DomainError::validation(
                "call description must not be empty",
            ));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "call description must be 1..{} chars",
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

/// A notification message body (1..=1000 chars).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NotificationMessage(String);

impl NotificationMessage {
    /// Maximum length of a notification message.
    pub const MAX_LEN: usize = 1_000;

    /// Constructs a new `NotificationMessage`, validating
    /// non-empty and length-bounded.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s: String = raw.into();
        if s.is_empty() {
            return Err(DomainError::validation(
                "notification message must not be empty",
            ));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "notification message must be 1..{} chars",
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

/// A mail driver name (1..=50 chars).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MailDriverName(String);

impl MailDriverName {
    /// Maximum length of a mail driver name.
    pub const MAX_LEN: usize = 50;

    /// Constructs a new `MailDriverName`, validating non-empty
    /// and length-bounded.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s: String = raw.into();
        if s.is_empty() {
            return Err(DomainError::validation(
                "mail driver name must not be empty",
            ));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "mail driver name must be 1..{} chars",
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

/// An SMS gateway name (1..=100 chars).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GatewayName(String);

impl GatewayName {
    /// Maximum length of a gateway name.
    pub const MAX_LEN: usize = 100;

    /// Constructs a new `GatewayName`, validating non-empty and
    /// length-bounded.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s: String = raw.into();
        if s.is_empty() {
            return Err(DomainError::validation("gateway name must not be empty"));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "gateway name must be 1..{} chars",
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

/// A reference to a secret (API key, password, token) stored
/// outside the database — e.g. a key in a vault, an env var, or
/// a value resolved by a secret manager. The reference string
/// is a non-empty opaque handle of at most 256 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SecretReference(String);

impl SecretReference {
    /// Maximum length of a secret reference.
    pub const MAX_LEN: usize = 256;

    /// Constructs a new `SecretReference`, validating non-empty
    /// and length-bounded.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s: String = raw.into();
        if s.is_empty() {
            return Err(DomainError::validation(
                "secret reference must be non-empty and <= 256 chars",
            ));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(
                "secret reference must be non-empty and <= 256 chars",
            ));
        }
        Ok(Self(s))
    }

    /// Returns the inner reference string. Callers MUST NOT log
    /// or display this value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A reference to a file stored in the [`educore-files`](::educore_files)
/// adapter (an object-store key or a row id). Non-empty.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FileReference(String);

impl FileReference {
    /// Constructs a new `FileReference`, validating non-empty.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s: String = raw.into();
        if s.is_empty() {
            return Err(DomainError::validation(
                "file reference must be non-empty",
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

/// A person name (1..=200 chars). Mirrors the academic
/// [`PersonName`](educore_academic::PersonName) shape; re-defined
/// here so the communication crate's value-objects surface is
/// self-contained.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PersonName(String);

impl PersonName {
    /// Maximum length of a person name.
    pub const MAX_LEN: usize = 200;

    /// Constructs a new `PersonName`, validating non-empty and
    /// length-bounded.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s: String = raw.into();
        if s.is_empty() {
            return Err(DomainError::validation("person name must not be empty"));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "person name must be 1..{} chars",
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

/// A template key — a stable identifier used by the renderer
/// to look up a template (1..=100 chars).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TemplateKey(String);

impl TemplateKey {
    /// Maximum length of a template key.
    pub const MAX_LEN: usize = 100;

    /// Constructs a new `TemplateKey`, validating non-empty and
    /// length-bounded.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s: String = raw.into();
        if s.is_empty() {
            return Err(DomainError::validation(
                "template key must not be empty",
            ));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "template key must be 1..{} chars",
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

/// A URL-safe slug (1..=200 chars, matching `[a-z0-9-]`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Slug(String);

impl Slug {
    /// Maximum length of a slug.
    pub const MAX_LEN: usize = 200;

    /// Constructs a new `Slug`, validating non-empty,
    /// length-bounded, and character-class.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s: String = raw.into();
        if s.is_empty() {
            return Err(DomainError::validation("slug must not be empty"));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(
                "slug must be 1..200 chars matching [a-z0-9-]",
            ));
        }
        if !s
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(DomainError::validation(
                "slug must be 1..200 chars matching [a-z0-9-]",
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

// ---- Numeric / scalar value objects ----

/// A star rating (1..=5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StarRating(pub u8);

impl StarRating {
    /// Constructs a new `StarRating`, validating the 1..=5 range.
    pub fn new(value: u8) -> Result<Self> {
        if !(1..=5).contains(&value) {
            return Err(DomainError::validation("star rating must be 1..5"));
        }
        Ok(Self(value))
    }

    /// Returns the inner value.
    #[must_use]
    pub const fn value(self) -> u8 {
        self.0
    }
}

// ---- Time / time-window value objects ----

/// A 24-hour time-of-day in `HH:MM` form. Validated at
/// construction.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TimeOfDay(String);

impl TimeOfDay {
    /// Constructs a new `TimeOfDay`, validating `HH:MM` 24-hour
    /// format.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s: String = raw.into();
        if !is_hhmm_24h(&s) {
            return Err(DomainError::validation("time of day must be HH:MM 24h"));
        }
        Ok(Self(s))
    }

    /// Returns the hour component (0..=23).
    #[must_use]
    pub fn hour(&self) -> u8 {
        self.0[..2].parse::<u8>().unwrap_or(0)
    }

    /// Returns the minute component (0..=59).
    #[must_use]
    pub fn minute(&self) -> u8 {
        self.0[3..5].parse::<u8>().unwrap_or(0)
    }

    /// Returns the inner string in `HH:MM` form.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A `(from, to)` time window. The `from` value must be
/// strictly before the `to` value (both are
/// [`TimeOfDay`]). `from == to` is rejected.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TimeWindow {
    /// The window start (inclusive).
    pub from: TimeOfDay,
    /// The window end (exclusive).
    pub to: TimeOfDay,
}

impl TimeWindow {
    /// Constructs a new `TimeWindow`, validating that `from` is
    /// strictly before `to`.
    pub fn new(from: TimeOfDay, to: TimeOfDay) -> Result<Self> {
        if !(from < to) {
            return Err(DomainError::validation(
                "time window from must be strictly before to",
            ));
        }
        Ok(Self { from, to })
    }
}

/// A call duration in `HH:MM:SS` form (1..=100 chars including
/// the colons). Validated at construction.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CallDuration(String);

impl CallDuration {
    /// Maximum length of a call duration string.
    pub const MAX_LEN: usize = 100;

    /// Constructs a new `CallDuration`, validating `HH:MM:SS`
    /// format and length.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s: String = raw.into();
        if s.is_empty() {
            return Err(DomainError::validation(
                "call duration must be 1..100 chars in HH:MM:SS format",
            ));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(
                "call duration must be 1..100 chars in HH:MM:SS format",
            ));
        }
        if !is_hhmmss(&s) {
            return Err(DomainError::validation(
                "call duration must be 1..100 chars in HH:MM:SS format",
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

// ---- Email / phone / URL value objects ----

/// An email address validated as a single `@`-separated local
/// and domain, with at least one `.` in the domain. ≤ 200
/// chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EmailAddress(String);

impl EmailAddress {
    /// Maximum length of an email address.
    pub const MAX_LEN: usize = 200;

    /// Constructs a new `EmailAddress`, validating basic RFC
    /// 5322 structure and length.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s: String = raw.into();
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(
                "email address must be valid RFC 5322 and <= 200 chars",
            ));
        }
        if !is_plausible_email(&s) {
            return Err(DomainError::validation(
                "email address must be valid RFC 5322 and <= 200 chars",
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

/// A phone number in either E.164 form (e.g. `+15551234567`)
/// or a national format (digits with optional spaces, hyphens,
/// and parentheses). ≤ 20 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PhoneNumber(String);

impl PhoneNumber {
    /// Maximum length of a phone number.
    pub const MAX_LEN: usize = 20;

    /// Constructs a new `PhoneNumber`, validating the format
    /// and length.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s: String = raw.into();
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(
                "phone number must be E.164 or national format, <= 20 chars",
            ));
        }
        if !is_plausible_phone(&s) {
            return Err(DomainError::validation(
                "phone number must be E.164 or national format, <= 20 chars",
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

/// A URL, validated to be ≤ 2048 chars and to start with
/// `http://` or `https://`. Not a full RFC 3986 parser; the
/// intent is to catch obvious mistakes (missing scheme, empty
/// host).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Url(String);

impl Url {
    /// Maximum length of a URL.
    pub const MAX_LEN: usize = 2_048;

    /// Constructs a new `Url`, validating length and scheme.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s: String = raw.into();
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation("url must be valid and <= 2048 chars"));
        }
        if !is_plausible_url(&s) {
            return Err(DomainError::validation("url must be valid and <= 2048 chars"));
        }
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// ---- Template variable ----

/// A single `(name, description)` template variable
/// declaration. The name is 1..=50 chars; the description is
/// 0..=200 chars (may be empty).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TemplateVariable {
    /// The variable name (the placeholder used in the template
    /// body, e.g. `"student_name"`).
    pub name: String,
    /// A human-readable description of what the variable
    /// represents. May be empty.
    pub description: String,
}

impl TemplateVariable {
    /// Maximum length of a template variable name.
    pub const NAME_MAX_LEN: usize = 50;
    /// Maximum length of a template variable description.
    pub const DESCRIPTION_MAX_LEN: usize = 200;

    /// Constructs a new `TemplateVariable`, validating the name
    /// and description lengths.
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Result<Self> {
        let name: String = name.into();
        let description: String = description.into();
        if name.is_empty() {
            return Err(DomainError::validation(
                "template variable name must be 1..50 chars",
            ));
        }
        if name.chars().count() > Self::NAME_MAX_LEN {
            return Err(DomainError::validation(
                "template variable name must be 1..50 chars",
            ));
        }
        if description.chars().count() > Self::DESCRIPTION_MAX_LEN {
            return Err(DomainError::validation(
                "template variable description must be <= 200 chars",
            ));
        }
        Ok(Self { name, description })
    }
}

// ---- Date / rendering value objects ----

/// A dispatch date (the day a notification / email / SMS was
/// sent). `NaiveDate` is the engine's standard date type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DispatchDate(pub NaiveDate);

impl DispatchDate {
    /// Constructs a new `DispatchDate`.
    pub const fn new(date: NaiveDate) -> Self {
        Self(date)
    }

    /// Returns the inner date.
    #[must_use]
    pub const fn value(self) -> NaiveDate {
        self.0
    }
}

/// The publication date of a [`Notice`](crate::aggregate::Notice).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NoticeDate(pub NaiveDate);

impl NoticeDate {
    /// Constructs a new `NoticeDate`.
    pub const fn new(date: NaiveDate) -> Self {
        Self(date)
    }

    /// Returns the inner date.
    #[must_use]
    pub const fn value(self) -> NaiveDate {
        self.0
    }
}

/// A publish-on schedule. `None` means "publish immediately";
/// `Some(date)` means "publish on this date" (the dispatcher
/// picks it up).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PublishOn(pub Option<NaiveDate>);

impl PublishOn {
    /// Publish immediately (no scheduled date).
    pub const fn immediate() -> Self {
        Self(None)
    }

    /// Publish on the given date.
    pub const fn scheduled(date: NaiveDate) -> Self {
        Self(Some(date))
    }

    /// Returns the inner optional date.
    #[must_use]
    pub const fn value(self) -> Option<NaiveDate> {
        self.0
    }
}

/// A rendered template body — the output of the template
/// service after substitution. The constructor is intentionally
/// `pub(crate)`; only the template service is allowed to
/// produce a `RenderedBody`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RenderedBody(String);

impl RenderedBody {
    /// Constructs a new `RenderedBody` from an already-rendered
    /// string. `pub(crate)` — only the template service
    /// produces these.
    pub(crate) fn from_rendered(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Returns the inner rendered string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// ---- Audience / routing value objects ----

/// A typed descriptor for who a notice or send-message
/// reaches. The constructors validate that `Vec` variants are
/// non-empty.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AudienceDescriptor {
    /// Reach all users with any of the given roles. The vec
    /// must be non-empty.
    Roles(Vec<RoleId>),
    /// Reach users in a given class, optionally narrowed to a
    /// specific section.
    ClassSection {
        /// The class id.
        class: ClassId,
        /// The optional section id. `None` means all sections
        /// of the class.
        section: Option<SectionId>,
    },
    /// Reach a specific set of users. The vec must be non-empty.
    Users(Vec<UserId>),
    /// Broadcast to every user in the school.
    All,
}

impl AudienceDescriptor {
    /// Constructs an `AudienceDescriptor::Roles` with a
    /// non-empty `roles` vec.
    pub fn roles(roles: Vec<RoleId>) -> Result<Self> {
        if roles.is_empty() {
            return Err(DomainError::validation(
                "audience roles must be a non-empty vec",
            ));
        }
        Ok(Self::Roles(roles))
    }

    /// Constructs an `AudienceDescriptor::ClassSection` for a
    /// class and optional section.
    pub fn class_section(class: ClassId, section: Option<SectionId>) -> Self {
        Self::ClassSection { class, section }
    }

    /// Constructs an `AudienceDescriptor::Users` with a
    /// non-empty `users` vec.
    pub fn users(users: Vec<UserId>) -> Result<Self> {
        if users.is_empty() {
            return Err(DomainError::validation(
                "audience users must be a non-empty vec",
            ));
        }
        Ok(Self::Users(users))
    }

    /// Constructs an `AudienceDescriptor::All` (school-wide
    /// broadcast).
    pub fn all() -> Self {
        Self::All
    }
}

/// The composed routing decision for a notification event:
/// which event, which destinations, and which recipient.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NotificationRoute {
    /// The event name (e.g. `"student.absent"`).
    pub event: String,
    /// The destinations to dispatch to.
    pub destination: Destination,
    /// The recipient user id.
    pub recipient: UserId,
}

impl NotificationRoute {
    /// Constructs a new `NotificationRoute`.
    pub const fn new(event: String, destination: Destination, recipient: UserId) -> Self {
        Self {
            event,
            destination,
            recipient,
        }
    }
}

/// An advisory warning emitted by the template service. These
/// do not fail rendering — they are surfaced to the operator
/// for visibility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderWarning {
    /// A declared template variable was not used in the body.
    UnusedVar(String),
    /// The body had unbalanced `{{` / `}}` braces at the given
    /// byte position.
    MismatchedBraces {
        /// The byte position of the imbalance.
        position: usize,
        /// The original (or partially rendered) body.
        body: String,
    },
    /// The body contained HTML but the target channel is SMS
    /// (where HTML is not rendered).
    HtmlInSms {
        /// The HTML snippet that triggered the warning.
        html: String,
        /// The SMS body (with the HTML stripped or escaped).
        sms_body: String,
    },
}

impl RenderWarning {
    /// Returns the wire-form string for the render warning kind.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::UnusedVar(_) => "UnusedVar",
            Self::MismatchedBraces { .. } => "MismatchedBraces",
            Self::HtmlInSms { .. } => "HtmlInSms",
        }
    }
}

// ---- SmsGatewayCredentials ----

/// The credential payload for an SMS gateway. The variant must
/// match the [`GatewayType`] of the parent
/// [`SmsGateway`](crate::aggregate::SmsGateway).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SmsGatewayCredentials {
    /// Clickatell credentials.
    Clickatell {
        /// The API username.
        username: String,
        /// The API password (stored as a [`SecretReference`]).
        password: SecretReference,
        /// The Clickatell api_id.
        api_id: String,
    },
    /// Twilio credentials.
    Twilio {
        /// The Twilio account SID.
        account_sid: String,
        /// The Twilio auth token (stored as a
        /// [`SecretReference`]).
        auth_token: SecretReference,
        /// The registered sending number (E.164).
        registered_no: String,
    },
    /// Msg91 credentials.
    Msg91 {
        /// The Msg91 auth key.
        auth_key: String,
        /// The Msg91 sender id.
        sender_id: String,
        /// The Msg91 route (transactional / promotional / ...).
        route: String,
    },
    /// Textlocal credentials.
    Textlocal {
        /// The Textlocal API key.
        api_key: String,
        /// The Textlocal sender id.
        sender: String,
    },
    /// AfricaTalking credentials.
    AfricaTalking {
        /// The AfricaTalking username.
        username: String,
        /// The AfricaTalking API key (stored as a
        /// [`SecretReference`]).
        api_key: SecretReference,
    },
    /// Custom HTTP gateway.
    Custom {
        /// The gateway endpoint URL.
        url: Url,
        /// The HTTP method.
        request_method: RequestMethod,
        /// The query/body parameters as `(name, value)` pairs.
        params: Vec<(String, String)>,
    },
}

impl SmsGatewayCredentials {
    /// Returns the [`GatewayType`] corresponding to this
    /// credential variant.
    #[must_use]
    pub const fn gateway_type(&self) -> GatewayType {
        match self {
            Self::Clickatell { .. } => GatewayType::Clickatell,
            Self::Twilio { .. } => GatewayType::Twilio,
            Self::Msg91 { .. } => GatewayType::Msg91,
            Self::Textlocal { .. } => GatewayType::Textlocal,
            Self::AfricaTalking { .. } => GatewayType::AfricaTalking,
            Self::Custom { .. } => GatewayType::Custom,
        }
    }
}

// =============================================================================
// Embedded value-object lists (4 children of aggregates)
// =============================================================================

/// The audience of a notice — the set of roles it reaches. At
/// least one role is required.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NoticeAudience(pub Vec<RoleId>);

impl NoticeAudience {
    /// Constructs a new `NoticeAudience` from a non-empty
    /// `roles` vec.
    pub fn new(roles: Vec<RoleId>) -> Result<Self> {
        if roles.is_empty() {
            return Err(DomainError::validation(
                "notice audience must contain at least one role",
            ));
        }
        Ok(Self(roles))
    }

    /// Returns the inner roles.
    #[must_use]
    pub fn as_slice(&self) -> &[RoleId] {
        &self.0
    }
}

/// The list of template variable declarations for an
/// [`SmsTemplate`](crate::aggregate::SmsTemplate). At least
/// one variable is required (an empty template would be
/// pointless). Each entry is itself a [`TemplateVariable`],
/// which validates the name and description lengths.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SmsTemplateVariable(pub Vec<TemplateVariable>);

impl SmsTemplateVariable {
    /// Constructs a new `SmsTemplateVariable` from a non-empty
    /// `vars` vec. Each variable is validated by
    /// [`TemplateVariable::new`].
    pub fn new(vars: Vec<TemplateVariable>) -> Result<Self> {
        if vars.is_empty() {
            return Err(DomainError::validation(
                "sms template variables must be a non-empty vec",
            ));
        }
        Ok(Self(vars))
    }

    /// Returns the inner variable list.
    #[must_use]
    pub fn as_slice(&self) -> &[TemplateVariable] {
        &self.0
    }
}

/// A type alias for the audience descriptor of a
/// [`NotificationSetting`](crate::aggregate::NotificationSetting).
/// The shape is identical to [`AudienceDescriptor`]; the alias
/// is provided so the domain code can speak about
/// notification-setting audiences with a domain-flavoured
/// type name.
pub type NotificationSettingAudience = AudienceDescriptor;

/// The list of `(name, value)` request parameters for a
/// [`CustomSmsSetting`](crate::aggregate::CustomSmsSetting).
/// At least one parameter is required. Each key is 1..=100
/// chars; each value is 1..=1000 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CustomSmsSettingParam(pub Vec<(String, String)>);

impl CustomSmsSettingParam {
    /// Maximum length of a parameter key.
    pub const KEY_MAX_LEN: usize = 100;
    /// Maximum length of a parameter value.
    pub const VALUE_MAX_LEN: usize = 1_000;

    /// Constructs a new `CustomSmsSettingParam` from a
    /// non-empty `params` vec, validating key and value
    /// lengths.
    pub fn new(params: Vec<(String, String)>) -> Result<Self> {
        if params.is_empty() {
            return Err(DomainError::validation(
                "custom sms setting params must be a non-empty vec",
            ));
        }
        for (k, v) in &params {
            if k.is_empty() || k.chars().count() > Self::KEY_MAX_LEN {
                return Err(DomainError::validation(
                    "custom sms setting param key must be 1..100 chars",
                ));
            }
            if v.is_empty() || v.chars().count() > Self::VALUE_MAX_LEN {
                return Err(DomainError::validation(
                    "custom sms setting param value must be 1..1000 chars",
                ));
            }
        }
        Ok(Self(params))
    }

    /// Returns the inner params.
    #[must_use]
    pub fn as_slice(&self) -> &[(String, String)] {
        &self.0
    }
}

// =============================================================================
// Local MessageId
// =============================================================================

/// A cross-domain message identifier (an email or SMS
/// message). The `educore_events::event_bus` module does not
/// export a `MessageId`, so it is defined locally as a
/// single-field newtype around `Uuid`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MessageId(pub Uuid);

impl MessageId {
    /// Constructs a new `MessageId` from a `Uuid`.
    #[must_use]
    pub const fn new(value: Uuid) -> Self {
        Self(value)
    }

    /// Returns the inner UUID.
    #[must_use]
    pub const fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

// =============================================================================
// Validation helpers (private)
// =============================================================================

/// Returns `true` if `s` is a 24-hour `HH:MM` time-of-day.
fn is_hhmm_24h(s: &str) -> bool {
    if s.len() != 5 {
        return false;
    }
    let bytes = s.as_bytes();
    if bytes[2] != b':' {
        return false;
    }
    if !bytes[0].is_ascii_digit() || !bytes[1].is_ascii_digit() {
        return false;
    }
    if !bytes[3].is_ascii_digit() || !bytes[4].is_ascii_digit() {
        return false;
    }
    let hour = (bytes[0] - b'0') * 10 + (bytes[1] - b'0');
    let minute = (bytes[3] - b'0') * 10 + (bytes[4] - b'0');
    hour < 24 && minute < 60
}

/// Returns `true` if `s` is a `HH:MM:SS` duration.
fn is_hhmmss(s: &str) -> bool {
    if s.len() != 8 {
        return false;
    }
    let bytes = s.as_bytes();
    if bytes[2] != b':' || bytes[5] != b':' {
        return false;
    }
    let digits_ok = bytes
        .iter()
        .enumerate()
        .all(|(i, b)| i == 2 || i == 5 || b.is_ascii_digit());
    if !digits_ok {
        return false;
    }
    let hour = (bytes[0] - b'0') * 10 + (bytes[1] - b'0');
    let minute = (bytes[3] - b'0') * 10 + (bytes[4] - b'0');
    let second = (bytes[6] - b'0') * 10 + (bytes[7] - b'0');
    hour < 24 && minute < 60 && second < 60
}

/// Returns `true` if `s` looks like a plausible email address
/// (single `@`, non-empty local and domain, the domain
/// contains a `.`).
fn is_plausible_email(s: &str) -> bool {
    if s.is_empty() || s.len() > 200 {
        return false;
    }
    let mut parts = s.splitn(2, '@');
    let local = parts.next().unwrap_or("");
    let domain = parts.next().unwrap_or("");
    if parts.next().is_some() {
        return false;
    }
    if local.is_empty() || domain.is_empty() {
        return false;
    }
    if local.contains(' ') || domain.contains(' ') {
        return false;
    }
    domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
}

/// Returns `true` if `s` looks like a plausible phone number:
/// starts with `+`, a digit, or `(`, contains only digits,
/// spaces, hyphens, and parentheses. At least one digit is
/// required.
fn is_plausible_phone(s: &str) -> bool {
    if s.is_empty() || s.len() > 20 {
        return false;
    }
    let bytes = s.as_bytes();
    let first = bytes[0];
    if first != b'+' && !first.is_ascii_digit() && first != b'(' {
        return false;
    }
    let mut has_digit = false;
    for (i, b) in bytes.iter().enumerate() {
        match *b {
            b'0'..=b'9' => has_digit = true,
            b'+' if i == 0 => {}
            b' ' | b'-' | b'(' | b')' => {}
            _ => return false,
        }
    }
    has_digit
}

/// Returns `true` if `s` looks like a plausible URL: starts
/// with `http://` or `https://`, has a non-empty host with at
/// least one `.`.
fn is_plausible_url(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let (scheme, rest) = if let Some(rest) = s.strip_prefix("https://") {
        ("https", rest)
    } else if let Some(rest) = s.strip_prefix("http://") {
        ("http", rest)
    } else {
        return false;
    };
    let _ = scheme; // silence unused warning; the prefix already gates this
    if rest.is_empty() {
        return false;
    }
    // The host is the substring up to the first `/`, `?`, `#`, or
    // end of string.
    let host_end = rest
        .find(|c: char| c == '/' || c == '?' || c == '#')
        .unwrap_or(rest.len());
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

    #[test]
    fn typed_id_display_and_accessors() {
        let school = SchoolId::from_uuid(Uuid::nil());
        let id = NoticeId::new(school, Uuid::from_u128(42));
        assert_eq!(id.school_id(), school);
        assert_eq!(id.as_uuid(), Uuid::from_u128(42));
        assert_eq!(id.to_string(), format!("{}/{}", school, id.value));
    }

    #[test]
    fn notice_title_validates_length() {
        assert!(NoticeTitle::new("").is_err());
        assert!(NoticeTitle::new("hello world").is_ok());
        assert!(NoticeTitle::new(&"x".repeat(201)).is_err());
    }

    #[test]
    fn email_address_validates() {
        assert!(EmailAddress::new("a@b.co").is_ok());
        assert!(EmailAddress::new("no-at-sign").is_err());
        assert!(EmailAddress::new("a@b").is_err());
        assert!(EmailAddress::new("a @b.co").is_err());
        assert!(EmailAddress::new(&format!("{}@x.co", "x".repeat(200))).is_err());
    }

    #[test]
    fn phone_number_validates() {
        assert!(PhoneNumber::new("+15551234567").is_ok());
        assert!(PhoneNumber::new("555-1234").is_ok());
        assert!(PhoneNumber::new("(555) 123-4567").is_ok());
        assert!(PhoneNumber::new("not a phone!").is_err());
        assert!(PhoneNumber::new("").is_err());
    }

    #[test]
    fn url_validates() {
        assert!(Url::new("https://example.com/path").is_ok());
        assert!(Url::new("http://example.com").is_ok());
        assert!(Url::new("example.com").is_err());
        assert!(Url::new("https://").is_err());
    }

    #[test]
    fn slug_validates() {
        assert!(Slug::new("hello-world-1").is_ok());
        assert!(Slug::new("UPPER").is_err());
        assert!(Slug::new("with space").is_err());
        assert!(Slug::new("").is_err());
    }

    #[test]
    fn star_rating_validates() {
        assert!(StarRating::new(0).is_err());
        assert!(StarRating::new(1).is_ok());
        assert!(StarRating::new(5).is_ok());
        assert!(StarRating::new(6).is_err());
    }

    #[test]
    fn time_of_day_validates() {
        assert!(TimeOfDay::new("00:00").is_ok());
        assert!(TimeOfDay::new("23:59").is_ok());
        assert!(TimeOfDay::new("24:00").is_err());
        assert!(TimeOfDay::new("12:60").is_err());
        assert!(TimeOfDay::new("1:00").is_err());
    }

    #[test]
    fn time_window_validates_ordering() {
        let a = TimeOfDay::new("09:00").unwrap();
        let b = TimeOfDay::new("17:00").unwrap();
        let c = TimeOfDay::new("09:00").unwrap();
        assert!(TimeWindow::new(a, b).is_ok());
        assert!(TimeWindow::new(b, a).is_err());
        assert!(TimeWindow::new(a, c).is_err());
    }

    #[test]
    fn call_duration_validates() {
        assert!(CallDuration::new("00:00:00").is_ok());
        assert!(CallDuration::new("23:59:59").is_ok());
        assert!(CallDuration::new("24:00:00").is_err());
        assert!(CallDuration::new("12:34").is_err());
    }

    #[test]
    fn template_variable_validates() {
        assert!(TemplateVariable::new("name", "desc").is_ok());
        assert!(TemplateVariable::new("name", "").is_ok());
        assert!(TemplateVariable::new("", "desc").is_err());
        assert!(TemplateVariable::new(&"x".repeat(51), "d").is_err());
    }

    #[test]
    fn destination_bitflag_works() {
        let d = Destination::EMAIL | Destination::SMS;
        assert!(d.contains(Destination::EMAIL));
        assert!(d.contains(Destination::SMS));
        assert!(!d.contains(Destination::WEB));
        assert_eq!(d.as_str(), "E,S");
        assert_eq!(Destination::WEB.as_str(), "W");
        assert_eq!(
            (Destination::EMAIL | Destination::SMS | Destination::WEB | Destination::APP).as_str(),
            "E,S,W,A"
        );
        assert_eq!(Destination::EMPTY.as_str(), "");
    }

    #[test]
    fn audience_descriptor_validates_non_empty() {
        let role = RoleId::new(SchoolId::from_uuid(Uuid::nil()), Uuid::from_u128(1));
        let user = UserId::from_uuid(Uuid::from_u128(2));
        assert!(AudienceDescriptor::roles(vec![]).is_err());
        assert!(AudienceDescriptor::roles(vec![role]).is_ok());
        assert!(AudienceDescriptor::users(vec![]).is_err());
        assert!(AudienceDescriptor::users(vec![user]).is_ok());
        assert!(matches!(AudienceDescriptor::all(), AudienceDescriptor::All));
    }

    #[test]
    fn notice_audience_validates_non_empty() {
        let role = RoleId::new(SchoolId::from_uuid(Uuid::nil()), Uuid::from_u128(1));
        assert!(NoticeAudience::new(vec![]).is_err());
        assert!(NoticeAudience::new(vec![role]).is_ok());
    }

    #[test]
    fn sms_gateway_credentials_gateway_type_matches_variant() {
        let creds = SmsGatewayCredentials::Clickatell {
            username: "u".to_string(),
            password: SecretReference::new("p").unwrap(),
            api_id: "a".to_string(),
        };
        assert_eq!(creds.gateway_type(), GatewayType::Clickatell);
        let creds = SmsGatewayCredentials::Custom {
            url: Url::new("https://example.com/send").unwrap(),
            request_method: RequestMethod::Post,
            params: vec![("to".to_string(), "{phone}".to_string())],
        };
        assert_eq!(creds.gateway_type(), GatewayType::Custom);
    }

    #[test]
    fn notice_status_wire_forms() {
        assert_eq!(NoticeStatus::Draft.as_str(), "Draft");
        assert_eq!(NoticeStatus::Published.as_str(), "Published");
    }

    #[test]
    fn complaint_status_wire_forms() {
        assert_eq!(ComplaintStatus::Open.as_str(), "Open");
        assert_eq!(ComplaintStatus::InProgress.as_str(), "InProgress");
        assert_eq!(ComplaintStatus::Resolved.as_str(), "Resolved");
    }

    #[test]
    fn complaint_action_wire_forms() {
        assert_eq!(ComplaintAction::Open.as_str(), "Open");
        assert_eq!(ComplaintAction::InProgress.as_str(), "InProgress");
        assert_eq!(ComplaintAction::Resolve.as_str(), "Resolve");
    }

    #[test]
    fn message_id_display() {
        let m = MessageId::new(Uuid::from_u128(7));
        assert_eq!(m.as_uuid(), Uuid::from_u128(7));
        assert_eq!(m.to_string(), "00000000-0000-0000-0000-000000000007");
    }
}
