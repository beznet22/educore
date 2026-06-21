//! # Notification port
//!
//! The notification port delivers messages to guardians, students,
//! staff, and external recipients. The engine does not own SMTP,
//! SMS, push, or chat transports — the consumer supplies an adapter
//! that implements [`NotificationProvider`].
//!
//! This file declares the port trait and every typed request /
//! response / value type used across the boundary. Per
//! `docs/ports/notifications.md`:
//!
//! - The trait is **object-safe** (it carries no `Self`-typed
//!   associated types and no generic methods), so consumers can
//!   hold `Box<dyn NotificationProvider>` and swap implementations
//!   at the engine boundary.
//! - Every variant and every field of every struct derives
//!   `Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize`
//!   so values can be logged, audited, and persisted by the
//!   engine without losing structure.
//!
//! ## Local newtypes
//!
//! Several types referenced in the spec (typed UUID identifiers,
//! `EmailAddress`, `PhoneNumber`, `LanguageTag`, `AttachmentRef`,
//! `TemplateValue`, `BulkId`, `BulkRecipientIndex`, `Url`,
//! `SecretString`, `Money`, `ContactInfo`, `RecipientExpr`,
//! `GuardianRole`, `ChatProvider`) are defined as local newtypes in
//! this file rather than pulled from elsewhere. The port crate
//! (`educore-notify`) is in the `adapters` tier; per AGENTS.md it
//! may not depend on any crate in the `domains` tier (so it cannot
//! borrow `Money` from `educore-finance`, `StudentId` from
//! `educore-academic`, etc.) and it does not take direct
//! dependencies on `uuid`, `secrecy`, or `url`. The newtypes
//! preserve type safety (cross-id confusion is still a compile
//! error) and serialise as ordinary JSON strings; adapters that
//! need to talk to the real UUID / Secret / Url types do the
//! conversion at their boundary.
//!
//! When the engine eventually lifts one of these wrappers into a
//! canonical location (e.g. `educore-core::value_objects`), the
//! alias sites here are the only places to update.

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

use std::collections::BTreeMap;
use std::fmt;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use educore_core::ids::{CorrelationId, IdempotencyKey, SchoolId, UserId};
use educore_core::tenant::TenantContext;
use educore_core::value_objects::Timestamp;

use crate::errors::NotificationError;

/// Convenience alias for the port's [`Result`] type. Adapters
/// return `Result<T, NotificationError>`.
pub type Result<T> = std::result::Result<T, NotificationError>;

// ---------------------------------------------------------------------------
// Local ID newtypes (opaque, String-backed)
// ---------------------------------------------------------------------------

/// The receipt id returned by every successful send. Opaque to
/// callers; the engine uses it to look up the durable record in
/// `communication_email_sms_logs` and to reconcile webhook status
/// updates from the provider.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NotificationReceiptId(pub String);

impl NotificationReceiptId {
    /// Wraps an opaque receipt id string.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the inner id string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NotificationReceiptId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for NotificationReceiptId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for NotificationReceiptId {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

/// An opaque id for a bulk send operation. Returned in
/// [`BulkReceipt::bulk_id`]; the engine uses it to fan out
/// per-recipient [`NotificationSent`](docs/specs/communication/events.md)
/// events.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BulkId(pub String);

impl BulkId {
    /// Wraps an opaque bulk id string.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the inner id string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for BulkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for BulkId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for BulkId {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

/// A positional index into the `recipients` vector of a
/// [`SendBulkNotification`]. Reported back in
/// [`BulkReceipt::failed`] so the engine can correlate the failure
/// with the input row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BulkRecipientIndex(pub u32);

impl BulkRecipientIndex {
    /// Constructs an index from a raw `u32`.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the raw index value.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl fmt::Display for BulkRecipientIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// An opaque id for a notification group (a curated list of
/// recipients managed outside the engine, e.g. the staff of a
/// department or the parents of a class section). The
/// communication domain owns the canonical definition; this port
/// uses an opaque string view at the API boundary.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GroupId(pub String);

impl GroupId {
    /// Wraps an opaque group id string.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the inner id string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for GroupId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for GroupId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for GroupId {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

/// An opaque id for a student. The academic domain owns the
/// canonical `StudentId` (a `(SchoolId, Uuid)` pair); this port
/// uses an opaque string view so the `adapters` tier can carry the
/// value without depending on `educore-academic`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StudentId(pub String);

impl StudentId {
    /// Wraps an opaque student id string.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the inner id string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for StudentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for StudentId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for StudentId {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

/// An opaque id for a staff member. The HR domain owns the
/// canonical `StaffId`; this port uses an opaque string view at
/// the API boundary.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StaffId(pub String);

impl StaffId {
    /// Wraps an opaque staff id string.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the inner id string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for StaffId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for StaffId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for StaffId {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

// ---------------------------------------------------------------------------
// Contact / channel value types
// ---------------------------------------------------------------------------

/// An email address. Opaque to the port — the engine does not
/// parse, normalise, or validate the value (that is the consumer's
/// job at the engine boundary). Adapters that need RFC 5322
/// validation do so themselves.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EmailAddress(pub String);

impl EmailAddress {
    /// Wraps a raw email address string.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the inner email address string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for EmailAddress {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for EmailAddress {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

/// A phone number in E.164 format (e.g. `+14155550101`). The port
/// treats the value as opaque; adapters do carrier lookup, format
/// conversion, etc.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PhoneNumber(pub String);

impl PhoneNumber {
    /// Wraps a raw phone number string.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the inner phone number string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PhoneNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for PhoneNumber {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for PhoneNumber {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

/// A BCP-47 language tag (e.g. `en-US`, `fr-FR`). Opaque to the
/// port.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LanguageTag(pub String);

impl LanguageTag {
    /// Wraps a raw BCP-47 tag.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the inner tag string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for LanguageTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for LanguageTag {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for LanguageTag {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl Default for LanguageTag {
    fn default() -> Self {
        Self("en".to_owned())
    }
}

/// The chat provider for [`Channel::Chat`]. The engine treats the
/// provider as opaque; adapters implement the transport for each
/// variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChatProvider {
    /// WhatsApp Business API.
    Whatsapp,
    /// Telegram Bot API.
    Telegram,
    /// Facebook Messenger.
    Messenger,
    /// Signal (via the Signal Business API).
    Signal,
    /// A generic / unspecified chat provider — the adapter picks
    /// the default transport based on the recipient's contact
    /// record.
    Other,
}

impl ChatProvider {
    /// Returns the canonical snake_case wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Whatsapp => "whatsapp",
            Self::Telegram => "telegram",
            Self::Messenger => "messenger",
            Self::Signal => "signal",
            Self::Other => "other",
        }
    }
}

impl fmt::Display for ChatProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

/// A URL. The port treats the value as opaque; adapters parse and
/// validate. Stored as a `String` so the port crate does not take
/// a direct dependency on the `url` crate.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Url(pub String);

impl Url {
    /// Wraps a raw URL string.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the inner URL string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for Url {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Url {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

/// A sensitive string value (webhook signing secret, API key,
/// etc.) that should never appear in logs or debug output.
///
/// The local wrapper redacts on `Debug` and `Display`. Adapters
/// that need a real `secrecy::SecretString` can construct one from
/// the inner string at their boundary; the port crate does not
/// pull in the `secrecy` crate itself.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SecretString(String);

impl SecretString {
    /// Constructs a `SecretString` from a raw value. The caller
    /// is responsible for not logging the input.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Exposes the inner secret. Use only on code paths that need
    /// to put the value on the wire (e.g. an HMAC signing
    /// operation); never log or otherwise leak the returned
    /// value.
    #[must_use]
    pub fn expose_secret(&self) -> &str {
        &self.0
    }

    /// Returns `true` if the wrapped secret is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretString(<redacted>)")
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

impl From<String> for SecretString {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SecretString {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

// ---------------------------------------------------------------------------
// Money and currency (local, finance-free)
// ---------------------------------------------------------------------------

/// An ISO-4217 currency code (e.g. `USD`, `EUR`, `INR`). The port
/// treats the value as opaque; adapters do FX conversion if
/// needed.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CurrencyCode(pub String);

impl CurrencyCode {
    /// Wraps a raw ISO-4217 code. The port does not validate the
    /// 3-letter shape; adapters that need it may validate.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the inner code string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CurrencyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for CurrencyCode {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for CurrencyCode {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl Default for CurrencyCode {
    fn default() -> Self {
        Self("USD".to_owned())
    }
}

/// A monetary amount expressed in `amount_minor` (cents, paisa,
/// etc.) with an associated [`CurrencyCode`].
///
/// Mirrors the shape of `educore_finance::value_objects::Money`
/// without taking a dependency on the finance domain crate. The
/// port crate is in the `adapters` tier; per AGENTS.md it may not
/// depend on any crate in the `domains` tier. Consumers that need
/// to convert to / from the canonical `Money` type do so at their
/// boundary.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Money {
    /// The amount in minor units (cents, paisa, etc.). Always
    /// non-negative in the port surface — refunds are tracked by
    /// the finance domain, not by the notification port.
    pub amount_minor: i64,
    /// The currency.
    pub currency: CurrencyCode,
}

impl Money {
    /// The zero amount in the given currency.
    #[must_use]
    pub fn zero(currency: CurrencyCode) -> Self {
        Self {
            amount_minor: 0,
            currency,
        }
    }

    /// Constructs a `Money` from a minor-unit amount and currency.
    #[must_use]
    pub const fn new(amount_minor: i64, currency: CurrencyCode) -> Self {
        Self {
            amount_minor,
            currency,
        }
    }

    /// Returns `true` if this is the zero amount.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.amount_minor == 0
    }
}

// ---------------------------------------------------------------------------
// Templates and template values
// ---------------------------------------------------------------------------

/// A typed reference to a template stored in the communication
/// domain. Per `docs/ports/notifications.md` § "Templates", the
/// adapter resolves the template body, applies variables, and
/// delivers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TemplateRef {
    /// A direct reference to a template by id.
    Id(crate::errors::NotificationTemplateId),
}

impl TemplateRef {
    /// Convenience constructor for the [`TemplateRef::Id`] variant.
    #[must_use]
    pub fn id(id: crate::errors::NotificationTemplateId) -> Self {
        Self::Id(id)
    }
}

/// A typed value supplied for a template variable. The template
/// declares its variables at creation time; the engine validates
/// that every required variable is provided and that the value's
/// type matches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemplateValue {
    /// A plain string (also used for HTML email bodies).
    Text(String),
    /// A signed 64-bit integer.
    Number(i64),
    /// A decimal value, encoded as a string so the port does not
    /// depend on `rust_decimal`. Adapters parse the value at the
    /// boundary.
    Decimal(String),
    /// A boolean flag.
    Boolean(bool),
    /// A calendar date, encoded as an ISO-8601 string (e.g.
    /// `2026-06-08`).
    Date(String),
    /// A free-form JSON value, encoded as a JSON string.
    Json(String),
}

impl TemplateValue {
    /// Constructs a [`TemplateValue::Text`].
    #[must_use]
    pub fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    /// Constructs a [`TemplateValue::Number`].
    #[must_use]
    pub const fn number(value: i64) -> Self {
        Self::Number(value)
    }

    /// Constructs a [`TemplateValue::Decimal`].
    #[must_use]
    pub fn decimal(value: impl Into<String>) -> Self {
        Self::Decimal(value.into())
    }

    /// Constructs a [`TemplateValue::Boolean`].
    #[must_use]
    pub const fn boolean(value: bool) -> Self {
        Self::Boolean(value)
    }

    /// Constructs a [`TemplateValue::Date`].
    #[must_use]
    pub fn date(value: impl Into<String>) -> Self {
        Self::Date(value.into())
    }

    /// Constructs a [`TemplateValue::Json`].
    #[must_use]
    pub fn json(value: impl Into<String>) -> Self {
        Self::Json(value.into())
    }
}

impl From<&str> for TemplateValue {
    fn from(s: &str) -> Self {
        Self::Text(s.to_owned())
    }
}

impl From<String> for TemplateValue {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl From<i64> for TemplateValue {
    fn from(n: i64) -> Self {
        Self::Number(n)
    }
}

impl From<bool> for TemplateValue {
    fn from(b: bool) -> Self {
        Self::Boolean(b)
    }
}

// ---------------------------------------------------------------------------
// Recipients
// ---------------------------------------------------------------------------

/// The role of a guardian relative to a student. Drives the
/// `Recipient::Guardian` resolution in the communication domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GuardianRole {
    /// The primary guardian — receives all notifications by
    /// default unless the school has a different routing policy.
    Primary,
    /// A secondary guardian — used when the primary is
    /// unreachable or the school targets both.
    Secondary,
    /// Any guardian of the student — the adapter picks the
    /// highest-priority contact per the tenant's policy.
    Any,
}

impl GuardianRole {
    /// Returns the canonical snake_case wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Any => "any",
        }
    }
}

impl fmt::Display for GuardianRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

/// Direct contact info supplied at the call site, used when the
/// recipient is not a known user / student / staff member (e.g. an
/// external vendor on a purchase order, or a one-off addressee).
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContactInfo {
    /// An email address, if any.
    pub email: Option<EmailAddress>,
    /// A phone number (E.164), if any.
    pub phone: Option<PhoneNumber>,
    /// A chat-platform user id (e.g. a Telegram chat id), if any.
    pub chat_id: Option<String>,
    /// The preferred language for this contact. Defaults to the
    /// tenant's locale at the engine boundary.
    pub language: Option<LanguageTag>,
}

impl ContactInfo {
    /// Constructs an empty `ContactInfo`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the email.
    #[must_use]
    pub fn with_email(mut self, email: EmailAddress) -> Self {
        self.email = Some(email);
        self
    }

    /// Sets the phone.
    #[must_use]
    pub fn with_phone(mut self, phone: PhoneNumber) -> Self {
        self.phone = Some(phone);
        self
    }

    /// Sets the chat id.
    #[must_use]
    pub fn with_chat_id(mut self, chat_id: impl Into<String>) -> Self {
        self.chat_id = Some(chat_id.into());
        self
    }

    /// Sets the language.
    #[must_use]
    pub fn with_language(mut self, language: LanguageTag) -> Self {
        self.language = Some(language);
        self
    }

    /// Returns `true` if at least one contact channel is set.
    #[must_use]
    pub fn has_any(&self) -> bool {
        self.email.is_some() || self.phone.is_some() || self.chat_id.is_some()
    }
}

/// A recipient expression evaluated by the engine using the query
/// layer (e.g. `"all students in class 5A"`, `"all parents of
/// grade 8 students with outstanding fees"`).
///
/// Per `docs/ports/notifications.md` § "Recipient", the engine
/// resolves the expression to a materialised list of [`Recipient`]
/// values before handing the request to the adapter. The port
/// treats the expression as an opaque string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RecipientExpr(pub String);

impl RecipientExpr {
    /// Wraps a raw expression string.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the inner expression string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RecipientExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for RecipientExpr {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for RecipientExpr {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

/// The addressee of a notification.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Recipient {
    /// A direct contact (external recipient not in the engine's
    /// user / student / staff tables).
    Direct(ContactInfo),
    /// A known engine user.
    User(UserId),
    /// A student.
    Student(StudentId),
    /// A specific (or any) guardian of a student.
    Guardian(StudentId, GuardianRole),
    /// A staff member.
    Staff(StaffId),
    /// A pre-curated group of recipients managed outside the
    /// engine.
    Group(GroupId),
    /// A flat list of recipients, delivered as a single logical
    /// send (one `NotificationSent` event, per-recipient receipts).
    List(Vec<Recipient>),
    /// A recipient expression evaluated by the engine before the
    /// adapter sees the request. The engine materialises the
    /// expression into a [`Recipient::List`] before dispatch.
    Expression(RecipientExpr),
}

// ---------------------------------------------------------------------------
// Channel
// ---------------------------------------------------------------------------

/// The transport channel for a notification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Channel {
    /// Email.
    Email {
        /// Optional `From:` address. If absent, the adapter uses
        /// the tenant's default sender.
        from: Option<EmailAddress>,
        /// Optional `Reply-To:` address.
        reply_to: Option<EmailAddress>,
    },
    /// SMS.
    Sms {
        /// Optional originating phone number. If absent, the
        /// adapter uses the tenant's default sender.
        from: Option<PhoneNumber>,
        /// `true` if the body contains non-ASCII characters and
        /// the adapter must use UCS-2 encoding (which halves the
        /// per-segment character limit).
        unicode: bool,
    },
    /// Push notification (FCM, APNs, etc.).
    Push {
        /// Optional FCM topic — when set, the adapter delivers
        /// to the topic instead of the recipient's device token.
        topic: Option<String>,
        /// Optional time-to-live. Messages that cannot be
        /// delivered within the TTL are dropped by the provider.
        ttl: Option<Duration>,
        /// Optional collapse key — used by the provider to
        /// coalesce pending messages.
        collapse_key: Option<String>,
    },
    /// In-app notification (rendered in the recipient's inbox
    /// inside the consumer's web / mobile app).
    InApp,
    /// Chat-app delivery (WhatsApp, Telegram, Messenger, Signal).
    Chat {
        /// The chat provider.
        provider: ChatProvider,
    },
    /// Voice (TTS phone call).
    Voice {
        /// Optional TTS voice id (e.g. `en-US-Wavenet-A`).
        voice_id: Option<String>,
        /// The language tag for the TTS engine.
        language: LanguageTag,
    },
    /// Outbound webhook (HTTP POST).
    Webhook {
        /// The webhook URL.
        url: Url,
        /// Optional HMAC signing secret. When set, the adapter
        /// signs the request body with the secret.
        secret: Option<SecretString>,
    },
}

// ---------------------------------------------------------------------------
// Priority
// ---------------------------------------------------------------------------

/// The priority of a notification.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Priority {
    /// Low priority — best-effort delivery, batched.
    Low,
    /// Normal priority — the default.
    #[default]
    Normal,
    /// High priority — delivered ahead of normal traffic.
    High,
    /// Critical priority — bypasses queues, delivered
    /// synchronously (e.g. emergency alerts). The adapter may
    /// charge a premium for `Critical`.
    Critical,
}

impl Priority {
    /// Returns the canonical snake_case wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Normal => "normal",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

// ---------------------------------------------------------------------------
// Attachments
// ---------------------------------------------------------------------------

/// A reference to a file attached to a notification. The
/// notification port does not own the file — it owns a pointer to
/// a blob in the files domain (see `educore-files`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AttachmentRef {
    /// The file id in the files domain.
    pub id: String,
    /// The download URL (typically a pre-signed S3 / GCS URL with
    /// an expiry).
    pub url: Url,
    /// The MIME type (e.g. `application/pdf`).
    pub mime: String,
    /// The file size in bytes.
    pub size_bytes: u64,
    /// Optional SHA-256 checksum, hex-encoded. Adapters may verify
    /// the checksum after download.
    pub checksum: Option<String>,
    /// Optional display filename (defaults to `id` if absent).
    pub filename: Option<String>,
}

impl AttachmentRef {
    /// Constructs an `AttachmentRef` with the required fields.
    #[must_use]
    pub fn new(id: impl Into<String>, url: Url, mime: impl Into<String>, size_bytes: u64) -> Self {
        Self {
            id: id.into(),
            url,
            mime: mime.into(),
            size_bytes,
            checksum: None,
            filename: None,
        }
    }

    /// Sets the checksum.
    #[must_use]
    pub fn with_checksum(mut self, checksum: impl Into<String>) -> Self {
        self.checksum = Some(checksum.into());
        self
    }

    /// Sets the display filename.
    #[must_use]
    pub fn with_filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }
}

// ---------------------------------------------------------------------------
// Delivery status
// ---------------------------------------------------------------------------

/// The current delivery status of a notification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryStatus {
    /// The notification has been accepted by the adapter and is
    /// waiting to be handed off to the provider (offline mode).
    Queued,
    /// The notification has been handed off to the provider.
    Sent,
    /// The provider confirmed delivery to the recipient's device
    /// or inbox.
    Delivered,
    /// The recipient opened the notification (email open pixel,
    /// push impression).
    Opened,
    /// The recipient clicked a link in the notification.
    Clicked,
    /// The provider reported a hard bounce. The wrapped string is
    /// the provider's reason (e.g. `"mailbox full"`,
    /// `"unknown user"`).
    Bounced {
        /// The bounce reason reported by the provider.
        reason: String,
    },
    /// The send failed. The wrapped `reason` is the provider's
    /// diagnostic; `retryable` indicates whether the engine
    /// should retry (transient errors) or give up (permanent
    /// errors).
    Failed {
        /// The provider's failure reason.
        reason: String,
        /// `true` if the adapter believes a retry would succeed.
        retryable: bool,
    },
    /// The notification was rejected before delivery (e.g.
    /// recipient opted out, template policy violation).
    Rejected {
        /// The rejection reason.
        reason: String,
    },
}

impl DeliveryStatus {
    /// Returns `true` if the notification has reached a terminal
    /// state (no further updates expected from the provider).
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Bounced { .. } | Self::Failed { .. } | Self::Rejected { .. }
        )
    }

    /// Returns `true` if the delivery is in flight (not yet
    /// terminal).
    #[must_use]
    pub fn is_in_flight(&self) -> bool {
        !self.is_terminal()
    }

    /// Returns `true` if the recipient engaged with the
    /// notification (opened or clicked).
    #[must_use]
    pub fn is_engagement(&self) -> bool {
        matches!(self, Self::Opened | Self::Clicked)
    }
}

// ---------------------------------------------------------------------------
// Send requests
// ---------------------------------------------------------------------------

/// A request to send a single notification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendNotification {
    /// The active tenant. Drives RBAC, template scoping, and
    /// per-tenant rate limits.
    pub tenant: TenantContext,
    /// The transport channel.
    pub channel: Channel,
    /// The template to render.
    pub template: TemplateRef,
    /// The recipient.
    pub recipient: Recipient,
    /// Template variable values, keyed by variable name. The
    /// engine validates that every required variable is present
    /// and that the value's type matches the template's
    /// declared type.
    pub variables: BTreeMap<String, TemplateValue>,
    /// Optional file attachments.
    pub attachments: Vec<AttachmentRef>,
    /// The delivery priority.
    pub priority: Priority,
    /// Optional scheduled delivery time. `None` means "send
    /// now".
    pub scheduled_at: Option<Timestamp>,
    /// Optional idempotency key. Per
    /// `docs/ports/notifications.md` § "Idempotency", the engine
    /// generates a deterministic key from `(command_id,
    /// recipient, template_version)`; the adapter uses the key
    /// to deduplicate retries.
    pub idempotency_key: Option<IdempotencyKey>,
    /// Optional correlation id propagated through every event
    /// emitted by the send.
    pub correlation_id: Option<CorrelationId>,
    /// The school that owns the send (denormalised off the
    /// tenant for adapter convenience).
    pub school_id: SchoolId,
}

impl SendNotification {
    /// Returns the active school's id.
    #[must_use]
    pub fn active_school_id(&self) -> SchoolId {
        self.school_id
    }

    /// Returns the actor's user id (from the tenant context).
    #[must_use]
    pub fn actor_id(&self) -> UserId {
        self.tenant.actor_id
    }
}

/// One recipient in a bulk send, with its own variable values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BulkRecipient {
    /// The recipient for this row.
    pub recipient: Recipient,
    /// The template variable values for this row.
    pub variables: BTreeMap<String, TemplateValue>,
}

impl BulkRecipient {
    /// Constructs a `BulkRecipient` with an empty variable map.
    #[must_use]
    pub fn new(recipient: Recipient) -> Self {
        Self {
            recipient,
            variables: BTreeMap::new(),
        }
    }

    /// Sets the variable map.
    #[must_use]
    pub fn with_variables(mut self, variables: BTreeMap<String, TemplateValue>) -> Self {
        self.variables = variables;
        self
    }
}

/// A request to send the same template to many recipients.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendBulkNotification {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The template to render.
    pub template: TemplateRef,
    /// The per-recipient rows.
    pub recipients: Vec<BulkRecipient>,
    /// When `true`, the engine expects each [`BulkRecipient`] to
    /// carry its own `variables` map. When `false`, every row
    /// uses the same variables and the adapter may apply the
    /// shared map internally.
    pub variables_per_recipient: bool,
    /// The transport channel.
    pub channel: Channel,
    /// The delivery priority.
    pub priority: Priority,
    /// Optional scheduled delivery time.
    pub scheduled_at: Option<Timestamp>,
    /// Optional idempotency key.
    pub idempotency_key: Option<IdempotencyKey>,
    /// Optional correlation id.
    pub correlation_id: Option<CorrelationId>,
    /// The school that owns the send.
    pub school_id: SchoolId,
}

// ---------------------------------------------------------------------------
// Receipts
// ---------------------------------------------------------------------------

/// The durable receipt returned by a successful send.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationReceipt {
    /// The receipt id. Stored in
    /// `communication_email_sms_logs` and emitted on the
    /// `NotificationSent` event.
    pub receipt_id: NotificationReceiptId,
    /// The provider's message id (e.g. SES `MessageId`,
    /// Twilio `MessageSid`). Used to reconcile webhook status
    /// updates.
    pub provider_message_id: Option<String>,
    /// The channel that was used to deliver the notification.
    pub channel: Channel,
    /// The current delivery status.
    pub status: DeliveryStatus,
    /// The provider's charge for this send, when known. Used for
    /// tenant-level cost reporting and budget control.
    pub cost: Option<Money>,
    /// The wall-clock time the adapter accepted the send. (In
    /// offline mode this is the time the notification was
    /// queued.)
    pub sent_at: Timestamp,
    /// Provider-specific metadata (e.g. SES `RequestId`, FCM
    /// `message_id`). The engine stores the metadata as-is.
    pub metadata: BTreeMap<String, String>,
}

impl NotificationReceipt {
    /// Constructs a minimal `NotificationReceipt` with the
    /// required fields and sensible defaults.
    #[must_use]
    pub fn new(
        receipt_id: NotificationReceiptId,
        channel: Channel,
        status: DeliveryStatus,
        sent_at: Timestamp,
    ) -> Self {
        Self {
            receipt_id,
            provider_message_id: None,
            channel,
            status,
            cost: None,
            sent_at,
            metadata: BTreeMap::new(),
        }
    }

    /// Sets the provider message id.
    #[must_use]
    pub fn with_provider_message_id(mut self, id: impl Into<String>) -> Self {
        self.provider_message_id = Some(id.into());
        self
    }

    /// Sets the cost.
    #[must_use]
    pub fn with_cost(mut self, cost: Money) -> Self {
        self.cost = Some(cost);
        self
    }

    /// Inserts a metadata entry.
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// The aggregate receipt returned by a bulk send.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BulkReceipt {
    /// The bulk send id.
    pub bulk_id: BulkId,
    /// The per-recipient receipts (one per successful send). The
    /// engine stores these alongside the corresponding
    /// `NotificationSent` events.
    pub receipts: Vec<NotificationReceipt>,
    /// The per-recipient failures, with the original
    /// [`BulkRecipientIndex`] preserved so the engine can
    /// correlate the failure with the input row.
    pub failed: Vec<(BulkRecipientIndex, NotificationError)>,
}

impl BulkReceipt {
    /// Constructs an empty `BulkReceipt` with the given bulk id.
    #[must_use]
    pub fn new(bulk_id: BulkId) -> Self {
        Self {
            bulk_id,
            receipts: Vec::new(),
            failed: Vec::new(),
        }
    }

    /// Returns the total number of recipients in the request
    /// (successes + failures).
    #[must_use]
    pub fn total(&self) -> usize {
        self.receipts.len() + self.failed.len()
    }

    /// Returns the number of successful sends.
    #[must_use]
    pub fn success_count(&self) -> usize {
        self.receipts.len()
    }

    /// Returns the number of failed sends.
    #[must_use]
    pub fn failure_count(&self) -> usize {
        self.failed.len()
    }
}

// ---------------------------------------------------------------------------
// The port trait
// ---------------------------------------------------------------------------

/// The notification port. Implemented by every consumer-side
/// adapter (SES, Twilio, FCM, Telegram, etc.) and held by the
/// engine as `Arc<dyn NotificationProvider>`.
///
/// The trait is **object-safe**: there are no associated types
/// keyed by `Self`, no generic methods, and no `where Self:
/// Sized` bounds on the methods. Consumers can store the
/// adapter as `Box<dyn NotificationProvider>` and swap
/// implementations at the engine boundary.
#[async_trait]
pub trait NotificationProvider: Send + Sync + std::fmt::Debug {
    /// Sends a single notification. Returns a [`NotificationReceipt`]
    /// describing the provider's acceptance of the send.
    async fn send(&self, request: SendNotification) -> Result<NotificationReceipt>;

    /// Sends the same template to many recipients. The adapter
    /// is responsible for batching (e.g. 100 SMS per request)
    /// and for reporting per-recipient status.
    async fn send_bulk(&self, request: SendBulkNotification) -> Result<BulkReceipt>;

    /// Looks up the current delivery status of a previously
    /// sent notification. Used by the engine to reconcile
    /// webhook status updates that arrived after the original
    /// send.
    async fn status(&self, receipt_id: NotificationReceiptId) -> Result<DeliveryStatus>;
}

// Object-safety compile-test: a `Box<dyn NotificationProvider>`
// must be a usable type.
#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod object_safety {
    use super::*;

    /// Compile-time check that the trait is object-safe.
    #[allow(dead_code)]
    fn _assert_object_safe(_: Box<dyn NotificationProvider>) {}
}
