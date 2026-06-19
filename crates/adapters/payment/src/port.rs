//! The `PaymentProvider` port — the engine's sole entry point for
//! moving money.
//!
//! Per `docs/ports/payments.md`, the `PaymentProvider` trait is the
//! only sanctioned way for the engine to issue charges, refunds,
//! and settlement queries against a payment gateway or an offline
//! cash-book adapter. Concrete adapters (Stripe, Razorpay, PayPal,
//! offline cash-book, …) are out-of-tree crates that implement
//! this trait. Consumers wire one adapter into the engine at
//! startup; the rest of the engine never touches a card network
//! or a bank API directly.
//!
//! # Object safety
//!
//! The trait is object-safe: the engine holds
//! `Arc<dyn PaymentProvider>` and dispatches commands against it.
//! Every method takes `&self`, has no generic parameters, and
//! returns `Result<T, PaymentError>` directly.
//!
//! # Deviations from `docs/ports/payments.md`
//!
//! The crate's `Cargo.toml` is intentionally minimal (only
//! `core`, `platform`, `events`, `tokio`, `async-trait`), so the
//! port uses **stdlib-only** value representations:
//!
//! - All opaque ID newtypes (`PaymentId`, `InvoiceId`, …) wrap
//!   `String` rather than `uuid::Uuid`. Adapters that need a
//!   parsed UUID parse the inner string at their boundary.
//! - `Url` is represented as `String` (URL-formatted UTF-8).
//! - `NaiveDate` is replaced by a local [`ChequeDate`] value
//!   object wrapping `(year, month, day)`. Adapters that need a
//!   `chrono::NaiveDate` construct it from the three fields.
//! - `SecretString` (wallet PIN) is replaced by `String`. The
//!   adapter MUST redact any string whose field name ends in
//!   `_pin`, `_secret`, or `_card` before the value reaches the
//!   audit log.
//! - `Money` and `CurrencyCode` are defined locally; the spec's
//!   assumption that they live in `educore-finance` is preserved
//!   by using the same `i64` minor-units convention.
//!
//! These deviations are documented here so future ports that gain
//! richer dependencies can adopt the spec's idiomatic types
//! without changing the trait surface.

use std::collections::BTreeMap;
use std::fmt;

use async_trait::async_trait;
use educore_core::ids::{CorrelationId, IdempotencyKey, SchoolId, UserId};
use educore_core::tenant::TenantContext;
use educore_core::value_objects::Timestamp;

// ---------------------------------------------------------------------------
// Newtype identifiers (opaque to consumers)
// ---------------------------------------------------------------------------

/// An opaque engine-issued payment identifier. Wraps a `String`
/// (UUID-formatted) so the port stays decoupled from the `uuid`
/// crate. Adapters that need the parsed UUID should call
/// `uuid::Uuid::parse_str(&payment_id.as_str())` at their boundary.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct PaymentId(String);

impl PaymentId {
    /// Constructs a new `PaymentId` from a raw string. The
    /// constructor is infallible; the engine treats the inner
    /// string as opaque.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the inner string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PaymentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for PaymentId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for PaymentId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// An opaque invoice identifier (one payment may settle one or more
/// invoices). Same shape as [`PaymentId`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct InvoiceId(String);

impl InvoiceId {
    /// Constructs a new `InvoiceId` from a raw string.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the inner string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for InvoiceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for InvoiceId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for InvoiceId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// A tokenised payment card identifier returned by the consumer's
/// frontend after a gateway SDK call. The engine never sees the
/// raw PAN; only the gateway's opaque token is forwarded.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct CardToken(String);

impl CardToken {
    /// Constructs a new `CardToken` from a raw string.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the inner string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CardToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for CardToken {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for CardToken {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// An opaque gateway-issued token (Stripe `PaymentIntent.id`,
/// PayPal order id, etc.). Returned by the gateway during the
/// redirect flow and forwarded back on the return trip.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct GatewayToken(String);

impl GatewayToken {
    /// Constructs a new `GatewayToken` from a raw string.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the inner string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for GatewayToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for GatewayToken {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for GatewayToken {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// An opaque wallet identifier (school issued or third party).
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct WalletId(String);

impl WalletId {
    /// Constructs a new `WalletId` from a raw string.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the inner string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WalletId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for WalletId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for WalletId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// An opaque bank-account identifier. The engine never stores raw
/// account numbers; only this tokenised form.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct BankAccountId(String);

impl BankAccountId {
    /// Constructs a new `BankAccountId` from a raw string.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the inner string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for BankAccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for BankAccountId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for BankAccountId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// A human-friendly gateway identifier (`"stripe"`, `"razorpay"`,
/// `"paypal"`, …). Used for telemetry, capability routing, and
/// the audit log.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct GatewayName(String);

impl GatewayName {
    /// Constructs a new `GatewayName` from a raw string.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the inner string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for GatewayName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for GatewayName {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for GatewayName {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// An opaque external customer identifier (used when the payer is
/// not a [`UserId`] / student / staff member — e.g. a parent who
/// pays without an account).
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct CustomerId(String);

impl CustomerId {
    /// Constructs a new `CustomerId` from a raw string.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the inner string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CustomerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for CustomerId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for CustomerId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

// ---------------------------------------------------------------------------
// Money and currency
// ---------------------------------------------------------------------------

/// ISO-4217 currency code (three-letter, ASCII uppercase). The
/// type is a thin validated wrapper around `String`; the
/// constructor enforces the format.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct CurrencyCode(String);

impl CurrencyCode {
    /// Constructs a new `CurrencyCode`, validating that the input
    /// is exactly three ASCII uppercase letters. Returns
    /// `Err(PaymentError::InvalidAmount)` (used as the port's
    /// catch-all validation error) on a malformed input.
    pub fn new(code: &str) -> Result<Self, crate::errors::PaymentError> {
        if code.len() == 3 && code.chars().all(|c| c.is_ascii_uppercase()) {
            Ok(Self(code.to_owned()))
        } else {
            Err(crate::errors::PaymentError::InvalidAmount(format!(
                "currency code must be 3 ASCII uppercase letters, got {code:?}"
            )))
        }
    }

    /// Returns the inner string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CurrencyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A monetary amount expressed in `MinorUnits` (i64 cents / paisa)
/// with an associated [`CurrencyCode`].
///
/// Per `docs/code-standards.md` § "Numeric conversions": raw
/// floats are forbidden across the engine boundary. All amounts
/// in the payment port are `Money { amount_minor: i64, currency:
/// CurrencyCode }`; the consumer's frontend is responsible for
/// formatting in major units for display.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Money {
    /// The amount in minor units (cents, paisa, etc.).
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

    /// Constructs a `Money` value, validating `amount_minor >= 0`.
    /// Returns `Err(PaymentError::InvalidAmount)` on negative input.
    pub fn new(
        currency: CurrencyCode,
        amount_minor: i64,
    ) -> Result<Self, crate::errors::PaymentError> {
        if amount_minor < 0 {
            return Err(crate::errors::PaymentError::InvalidAmount(format!(
                "money amount must be non-negative, got {amount_minor}"
            )));
        }
        Ok(Self {
            amount_minor,
            currency,
        })
    }

    /// Returns `true` if this is the zero amount.
    #[must_use]
    pub const fn is_zero(&self) -> bool {
        self.amount_minor == 0
    }

    /// Returns `true` if the two `Money` values share the same
    /// currency. The adapter should reject mismatches with
    /// [`crate::errors::PaymentError::CurrencyMismatch`].
    #[must_use]
    pub fn same_currency(&self, other: &Self) -> bool {
        self.currency == other.currency
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.amount_minor, self.currency)
    }
}

// ---------------------------------------------------------------------------
// Cheque date (replacement for `chrono::NaiveDate`)
// ---------------------------------------------------------------------------

/// A calendar date used by [`PaymentMethod::Cheque`]. Equivalent
/// to `chrono::NaiveDate` for the engine's purposes; defined as a
/// `(year, month, day)` triple so the port does not depend on
/// `chrono`. Adapters that need a `NaiveDate` construct it via
/// `NaiveDate::from_ymd_opt(self.year, self.month, self.day)`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ChequeDate {
    /// Calendar year (e.g. `2026`).
    pub year: i32,
    /// Calendar month (`1..=12`).
    pub month: u32,
    /// Day of month (`1..=31`, adapter-dependent on month / leap year).
    pub day: u32,
}

impl ChequeDate {
    /// Constructs a `ChequeDate`, validating the month and day
    /// bounds. Day-of-month upper bounds (28 / 29 / 30 / 31) are
    /// the adapter's responsibility; the port only rejects
    /// `month == 0`, `month > 12`, `day == 0`, `day > 31`.
    pub fn new(year: i32, month: u32, day: u32) -> Result<Self, crate::errors::PaymentError> {
        if month == 0 || month > 12 {
            return Err(crate::errors::PaymentError::InvalidAmount(format!(
                "cheque month must be in 1..=12, got {month}"
            )));
        }
        if day == 0 || day > 31 {
            return Err(crate::errors::PaymentError::InvalidAmount(format!(
                "cheque day must be in 1..=31, got {day}"
            )));
        }
        Ok(Self { year, month, day })
    }
}

impl fmt::Display for ChequeDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

// ---------------------------------------------------------------------------
// Payment method (6 variants per spec line 48-56)
// ---------------------------------------------------------------------------

/// The method used to fund a payment. The engine's 6-variant
/// surface is locked to `docs/ports/payments.md` § "PaymentMethod".
///
/// `Card` and `Gateway` variants carry only tokenised references;
/// the engine never sees a raw PAN. `Wallet` carries the wallet
/// PIN as a `String`; adapters MUST redact it on the way to the
/// audit log. `ManualAdjustment` is for back-office corrections
/// and requires an approver [`UserId`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PaymentMethod {
    /// Cash paid in person at the school office.
    Cash,

    /// A physical cheque. Carries the cheque number, the issuing
    /// bank, and the date the cheque was written.
    Cheque {
        /// The cheque's serial number.
        number: String,
        /// The issuing bank's name.
        bank: String,
        /// The date written on the cheque.
        date: ChequeDate,
    },

    /// A direct bank transfer. Carries a transaction reference
    /// and a tokenised bank account.
    BankTransfer {
        /// The transfer's reference number (UTR / IMAD / etc.).
        reference: String,
        /// The receiving bank account (tokenised).
        bank_account: BankAccountId,
    },

    /// A tokenised card payment. The `save` flag requests that
    /// the gateway store the card for future recurring charges.
    Card {
        /// The card token issued by the consumer's frontend.
        token: CardToken,
        /// `true` to vault the card for future use.
        save: bool,
    },

    /// A redirect to an external gateway's hosted page (3DS,
    /// wallets, etc.). The adapter presents `return_url` to the
    /// gateway's SDK; the engine correlates the eventual webhook
    /// to the originating charge via `idempotency_key`.
    Gateway {
        /// The gateway identifier.
        gateway: GatewayName,
        /// The gateway-issued token from the create-order step.
        token: GatewayToken,
        /// The URL the gateway redirects the customer to after
        /// the hosted-page flow completes.
        return_url: String,
    },

    /// A wallet payment (school issued or third party). The PIN
    /// is captured at the consumer's frontend and forwarded only
    /// to the wallet adapter; the engine stores it transiently.
    Wallet {
        /// The wallet identifier.
        wallet_id: WalletId,
        /// The wallet's PIN (already redacted in the audit log).
        pin: String,
    },

    /// A back-office manual adjustment (write-off, refund of a
    /// refund, fee waiver, …). Carries the reason and the
    /// approving user's id.
    ManualAdjustment {
        /// The reason for the adjustment (free-form, stored in
        /// the audit log verbatim).
        reason: String,
        /// The approving user's id (RBAC: must hold
        /// `Finance.Adjust`).
        approver: UserId,
    },
}

/// A flattened, redacted representation of a [`PaymentMethod`]
/// suitable for storage in [`PaymentReceipt`] and for the audit
/// log. The kind does not carry tokenised card numbers, PINs, or
/// bank account references.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaymentMethodKind {
    /// Cash payment.
    Cash,
    /// Cheque payment.
    Cheque,
    /// Bank transfer.
    BankTransfer,
    /// Card payment.
    Card,
    /// External gateway redirect.
    Gateway,
    /// Wallet payment.
    Wallet,
    /// Manual adjustment.
    ManualAdjustment,
}

impl PaymentMethod {
    /// Returns the redacted kind for this payment method. The
    /// kind is the only payment-method information the engine
    /// persists alongside receipts and emits in events.
    #[must_use]
    pub const fn kind(&self) -> PaymentMethodKind {
        match self {
            Self::Cash => PaymentMethodKind::Cash,
            Self::Cheque { .. } => PaymentMethodKind::Cheque,
            Self::BankTransfer { .. } => PaymentMethodKind::BankTransfer,
            Self::Card { .. } => PaymentMethodKind::Card,
            Self::Gateway { .. } => PaymentMethodKind::Gateway,
            Self::Wallet { .. } => PaymentMethodKind::Wallet,
            Self::ManualAdjustment { .. } => PaymentMethodKind::ManualAdjustment,
        }
    }
}

impl fmt::Display for PaymentMethodKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Cash => "cash",
            Self::Cheque => "cheque",
            Self::BankTransfer => "bank_transfer",
            Self::Card => "card",
            Self::Gateway => "gateway",
            Self::Wallet => "wallet",
            Self::ManualAdjustment => "manual_adjustment",
        };
        f.write_str(s)
    }
}

// ---------------------------------------------------------------------------
// Payment status (8 variants per spec line 87-96)
// ---------------------------------------------------------------------------

/// The lifecycle status of a payment. Locked to
/// `docs/ports/payments.md` § "PaymentStatus".
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PaymentStatus {
    /// The charge has been accepted but neither authorised nor
    /// captured (e.g. queued for 3DS verification).
    Pending,

    /// The card has been authorised but not yet captured. Carries
    /// the issuer's authorisation code and the auth-window
    /// expiry. A capture must happen before `expires_at`.
    Authorized {
        /// The issuer's authorisation code.
        auth_code: String,
        /// When the authorisation expires.
        expires_at: Timestamp,
    },

    /// The card has been captured (funds have left the customer).
    /// Carries the capture timestamp.
    Captured {
        /// When the capture happened.
        at: Timestamp,
    },

    /// The charge failed at the gateway. Carries the gateway's
    /// reason and an optional machine-readable code.
    Failed {
        /// Human-readable reason.
        reason: String,
        /// Machine-readable code (e.g. `do_not_honor`).
        code: Option<String>,
    },

    /// The charge was fully refunded. Carries the refunded amount,
    /// the refund timestamp, and the reason.
    Refunded {
        /// The refunded amount.
        amount: Money,
        /// When the refund was issued.
        at: Timestamp,
        /// The refund reason.
        reason: String,
    },

    /// The charge was partially refunded. Carries the cumulative
    /// refunded amount and the remaining (still-refundable)
    /// amount.
    PartiallyRefunded {
        /// The cumulative refunded amount.
        refunded: Money,
        /// The remaining refundable amount.
        remaining: Money,
    },

    /// A dispute (chargeback) has been opened against the charge.
    /// Carries the gateway's dispute id, the reason, and the
    /// opening timestamp.
    Disputed {
        /// The gateway's dispute id.
        dispute_id: String,
        /// The dispute reason.
        reason: String,
        /// When the dispute was opened.
        opened_at: Timestamp,
    },

    /// The charge was cancelled before capture (e.g. 3DS
    /// abandoned). Carries the cancellation timestamp and the
    /// reason.
    Cancelled {
        /// When the cancellation happened.
        at: Timestamp,
        /// The cancellation reason.
        reason: String,
    },
}

// ---------------------------------------------------------------------------
// Refund destination (4 variants per spec line 111-117)
// ---------------------------------------------------------------------------

/// Where a refund should land. Locked to
/// `docs/ports/payments.md` § "RefundDestination".
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RefundDestination {
    /// Refund back to the original payment method (card, wallet,
    /// …). This is the default for full refunds.
    OriginalMethod,

    /// Refund to a specific wallet (e.g. credit the parent's
    /// school wallet instead of the original card).
    Wallet {
        /// The destination wallet.
        wallet_id: WalletId,
    },

    /// Refund to a specific bank account. Used for offline
    /// settlements where the original payment was cash.
    BankAccount {
        /// The destination bank account.
        bank_account_id: BankAccountId,
    },

    /// Issue a credit note (a future invoice offset). The engine
    /// records the credit note and applies it to the customer's
    /// next invoice.
    CreditNote,
}

// ---------------------------------------------------------------------------
// Customer reference
// ---------------------------------------------------------------------------

/// The party paying for a charge. Used to anchor the receipt and
/// the audit log to a concrete actor.
///
/// The four variants cover the engine's three principal actors
/// (student, staff, user) plus the "external payer" case (a
/// parent or guardian who is not a registered user). Student and
/// staff identifiers are wrapped in their own newtypes so the
/// caller cannot mix them up at the call site.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CustomerRef {
    /// A student is paying (e.g. a university student topping up
    /// their meal card).
    Student {
        /// The student's id (UUID-formatted opaque string).
        student_id: String,
    },

    /// A staff member is paying (e.g. a teacher buying a uniform).
    Staff {
        /// The staff member's id (UUID-formatted opaque string).
        staff_id: String,
    },

    /// A registered user is paying (the school's `UserId`).
    User(UserId),

    /// An external payer (parent, guardian, vendor). Carries an
    /// opaque identifier — the consumer's frontend builds this
    /// from the parent's contact record.
    External(CustomerId),
}

// ---------------------------------------------------------------------------
// Per-payment fee
// ---------------------------------------------------------------------------

/// A per-payment fee (gateway fee, processing fee, FX margin, …).
/// Fees reduce the net amount deposited in the school's account.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PaymentFee {
    /// The fee amount.
    pub amount: Money,
    /// A short, human-readable description of what the fee is for.
    pub description: String,
}

// ---------------------------------------------------------------------------
// ChargeRequest
// ---------------------------------------------------------------------------

/// The input to [`PaymentProvider::charge`]. Locked to
/// `docs/ports/payments.md` § "ChargeRequest" with one deviation:
/// `webhook_url` is a `String` (URL-formatted) rather than a
/// `url::Url`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChargeRequest {
    /// The tenant context (school, actor, correlation).
    pub tenant: TenantContext,
    /// The amount to charge.
    pub amount: Money,
    /// The payment method.
    pub method: PaymentMethod,
    /// The invoices being settled by this charge. May be empty
    /// for ad-hoc fees (donations, top-ups).
    pub invoice_ids: Vec<InvoiceId>,
    /// The paying party.
    pub customer: CustomerRef,
    /// A free-form description that surfaces on the receipt and
    /// the customer's statement.
    pub description: String,
    /// Adapter-specific metadata (e.g. `campaign_id`,
    /// `pos_terminal_id`). Adapters may use any well-known keys.
    pub metadata: BTreeMap<String, String>,
    /// The idempotency key. A retry of the same charge returns
    /// the same [`PaymentReceipt`] without re-charging.
    pub idempotency_key: IdempotencyKey,
    /// The correlation id (propagated to the audit log).
    pub correlation_id: CorrelationId,
    /// `false` to authorise only (two-step flow); `true` to
    /// capture immediately. Defaults to `true` via [`Self::new`].
    pub capture: bool,
    /// Optional webhook URL the gateway should POST status
    /// updates to.
    pub webhook_url: Option<String>,
}

impl ChargeRequest {
    /// Constructs a `ChargeRequest` with required fields and
    /// sensible defaults (`capture = true`, empty invoice list,
    /// empty metadata).
    #[must_use]
    pub fn new(
        tenant: TenantContext,
        amount: Money,
        method: PaymentMethod,
        customer: CustomerRef,
        idempotency_key: IdempotencyKey,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            tenant,
            amount,
            method,
            invoice_ids: Vec::new(),
            customer,
            description: String::new(),
            metadata: BTreeMap::new(),
            idempotency_key,
            correlation_id,
            capture: true,
            webhook_url: None,
        }
    }
}

// ---------------------------------------------------------------------------
// RefundRequest
// ---------------------------------------------------------------------------

/// The input to [`PaymentProvider::refund`]. Locked to
/// `docs/ports/payments.md` § "RefundRequest".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefundRequest {
    /// The tenant context (school, actor, correlation).
    pub tenant: TenantContext,
    /// The original payment being refunded.
    pub original_payment_id: PaymentId,
    /// The amount to refund. May be partial (less than the
    /// original charge).
    pub amount: Money,
    /// A free-form reason (shown on the customer's statement).
    pub reason: String,
    /// Where the refund should land.
    pub refund_to: RefundDestination,
    /// The idempotency key.
    pub idempotency_key: IdempotencyKey,
}

// ---------------------------------------------------------------------------
// PaymentReceipt
// ---------------------------------------------------------------------------

/// The output of [`PaymentProvider::charge`]. The receipt is
/// durable: the engine stores it and emits a `PaymentReceived`
/// event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentReceipt {
    /// The engine-issued payment id.
    pub payment_id: PaymentId,
    /// The gateway's id for the same payment (None for offline
    /// methods that have no gateway counterpart).
    pub provider_payment_id: Option<String>,
    /// The current status.
    pub status: PaymentStatus,
    /// The amount that was charged.
    pub amount: Money,
    /// The redacted method kind (no card tokens / PINs).
    pub method: PaymentMethodKind,
    /// When the charge was authorised (if applicable).
    pub authorized_at: Option<Timestamp>,
    /// When the charge was captured (if applicable).
    pub captured_at: Option<Timestamp>,
    /// The fees applied (gateway, processing, FX, …).
    pub fees: Vec<PaymentFee>,
    /// The net amount (gross minus fees) deposited in the
    /// school's account.
    pub net: Money,
    /// A URL the customer can visit to download a printable
    /// receipt (if the gateway supports it).
    pub receipt_url: Option<String>,
    /// Adapter-specific metadata echoed back from the gateway.
    pub metadata: BTreeMap<String, String>,
}

// ---------------------------------------------------------------------------
// RefundReceipt
// ---------------------------------------------------------------------------

/// The output of [`PaymentProvider::refund`]. Mirrors
/// [`PaymentReceipt`] but for a refund.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefundReceipt {
    /// The engine-issued refund id.
    pub refund_id: PaymentId,
    /// The original payment's id.
    pub original_payment_id: PaymentId,
    /// The gateway's id for the refund.
    pub provider_refund_id: Option<String>,
    /// The amount refunded.
    pub amount: Money,
    /// The current status of the refund (most adapters return
    /// `Captured` immediately).
    pub status: PaymentStatus,
    /// When the refund was issued.
    pub refunded_at: Option<Timestamp>,
    /// The destination ([`RefundDestination`] in effect).
    pub destination: RefundDestination,
    /// Adapter-specific metadata.
    pub metadata: BTreeMap<String, String>,
}

// ---------------------------------------------------------------------------
// Settlement
// ---------------------------------------------------------------------------

/// The input to [`PaymentProvider::settlement`]. The adapter is
/// asked to report all captured payments that settled into the
/// school's bank account between `period_start` and `period_end`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettlementRequest {
    /// The tenant context (school, actor, correlation).
    pub tenant: TenantContext,
    /// Inclusive start of the settlement window.
    pub period_start: Timestamp,
    /// Inclusive end of the settlement window.
    pub period_end: Timestamp,
    /// The currency the school expects to be settled in. The
    /// adapter rejects mismatches with
    /// [`crate::errors::PaymentError::CurrencyMismatch`].
    pub currency: CurrencyCode,
}

/// A single settlement line — one captured payment that has
/// settled into the school's bank account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettlementLine {
    /// The gateway's id for the underlying payment. The engine
    /// uses this to join settlement lines back to
    /// [`PaymentReceipt`] rows.
    pub provider_payment_id: String,
    /// The engine-issued id of the underlying payment.
    pub payment_id: PaymentId,
    /// The gross captured amount.
    pub gross: Money,
    /// The fee taken by the gateway.
    pub fee: Money,
    /// The net amount deposited.
    pub net: Money,
    /// When the funds landed in the school's bank account.
    pub settled_at: Timestamp,
}

/// A batch of captured payments that settled into the school's
/// bank account over a single reporting period.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settlement {
    /// The adapter-issued id for this settlement batch.
    pub settlement_id: String,
    /// The school this settlement is for (echoed back from the
    /// [`SettlementRequest`] for downstream filtering).
    pub school_id: SchoolId,
    /// The currency all lines are denominated in.
    pub currency: CurrencyCode,
    /// The reporting window's start.
    pub period_start: Timestamp,
    /// The reporting window's end.
    pub period_end: Timestamp,
    /// The individual settlement lines.
    pub lines: Vec<SettlementLine>,
    /// The total gross across all lines.
    pub total_gross: Money,
    /// The total fees across all lines.
    pub total_fees: Money,
    /// The total net across all lines (gross minus fees).
    pub total_net: Money,
}

// ---------------------------------------------------------------------------
// PaymentMethodInfo (returned by list_methods)
// ---------------------------------------------------------------------------

/// A description of a payment method supported by the adapter
/// (returned by [`PaymentProvider::list_methods`]). Used by the
/// consumer's frontend to render the payment-method picker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentMethodInfo {
    /// The method kind.
    pub kind: PaymentMethodKind,
    /// A human-readable display name (e.g. `"Credit Card"`,
    /// `"Cash at Office"`, `"Stripe"`, …).
    pub display_name: String,
    /// Whether the method is currently enabled for this tenant.
    pub enabled: bool,
    /// A short note shown next to the method in the picker
    /// (e.g. `"2.9% + 30¢ processing fee"`).
    pub note: Option<String>,
}

// ---------------------------------------------------------------------------
// PaymentProvider trait (5 methods, object-safe)
// ---------------------------------------------------------------------------

/// The payment port — the engine's sole entry point for moving
/// money. Adapters are `Send + Sync` so the engine can dispatch
/// against them from any async runtime.
///
/// # Object safety
///
/// The trait is object-safe: every method takes `&self`, has no
/// generic parameters, and returns `Result<T, PaymentError>`
/// directly. The compile-time assertion at the bottom of this
/// module pins the object-safety contract.
///
/// # Idempotency
///
/// `charge` and `refund` are idempotent on `idempotency_key`. A
/// retry with the same key returns the original receipt without
/// re-issuing the charge. Adapters that cannot guarantee
/// idempotency MUST implement it themselves (the engine does not
/// retry).
///
/// # Multi-currency
///
/// `ChargeRequest.amount` MUST match the invoice currency. The
/// adapter rejects mismatches with
/// [`crate::errors::PaymentError::CurrencyMismatch`].
#[async_trait]
pub trait PaymentProvider: Send + Sync + fmt::Debug {
    /// Issues a charge against the given method. The returned
    /// [`PaymentReceipt`] is durable; the engine stores it and
    /// emits a `PaymentReceived` event.
    async fn charge(
        &self,
        request: ChargeRequest,
    ) -> Result<PaymentReceipt, crate::errors::PaymentError>;

    /// Refunds a previous payment. The returned
    /// [`RefundReceipt`] mirrors the original receipt. A retry
    /// with the same `idempotency_key` returns the same receipt.
    async fn refund(
        &self,
        request: RefundRequest,
    ) -> Result<RefundReceipt, crate::errors::PaymentError>;

    /// Returns the current status of the given payment. Used by
    /// the engine to reconcile webhook deliveries against the
    /// in-flight charge.
    async fn status(
        &self,
        payment_id: PaymentId,
    ) -> Result<PaymentStatus, crate::errors::PaymentError>;

    /// Lists the payment methods enabled for the given tenant.
    /// The result is rendered by the consumer's frontend as the
    /// payment-method picker.
    async fn list_methods(
        &self,
        tenant: TenantContext,
    ) -> Result<Vec<PaymentMethodInfo>, crate::errors::PaymentError>;

    /// Reports the settlement batch covering the requested
    /// window. The engine matches settlement lines to
    /// [`PaymentReceipt`] rows by `provider_payment_id` and emits
    /// `PaymentSettled` events for each newly-settled line.
    async fn settlement(
        &self,
        request: SettlementRequest,
    ) -> Result<Settlement, crate::errors::PaymentError>;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;
    use educore_core::clock::IdGenerator;

    // Compile-time object-safety assertion. If the trait gains a
    // generic method, this assignment will fail to compile,
    // surfacing the regression immediately.
    fn _assert_object_safe(_t: Box<dyn PaymentProvider + Sync>) {}

    fn usd() -> CurrencyCode {
        CurrencyCode::new("USD").unwrap()
    }

    fn eur() -> CurrencyCode {
        CurrencyCode::new("EUR").unwrap()
    }

    fn ctx() -> TenantContext {
        use educore_core::clock::{IdGenerator, SystemIdGen};
        let g = SystemIdGen;
        TenantContext::for_user(
            g.next_school_id(),
            g.next_user_id(),
            g.next_correlation_id(),
            educore_core::tenant::UserType::Teacher,
        )
    }

    #[test]
    fn currency_code_validates_iso_4217_shape() {
        assert!(CurrencyCode::new("USD").is_ok());
        assert!(CurrencyCode::new("usd").is_err());
        assert!(CurrencyCode::new("US").is_err());
        assert!(CurrencyCode::new("USDD").is_err());
        assert!(CurrencyCode::new("US1").is_err());
    }

    #[test]
    fn money_rejects_negative_amount() {
        let m = Money::new(usd(), 1500);
        assert!(m.is_ok());
        let m = Money::new(usd(), -1);
        assert!(m.is_err());
    }

    #[test]
    fn money_same_currency_compare() {
        let a = Money::new(usd(), 100).unwrap();
        let b = Money::new(usd(), 200).unwrap();
        let c = Money::new(eur(), 100).unwrap();
        assert!(a.same_currency(&b));
        assert!(!a.same_currency(&c));
    }

    #[test]
    fn cheque_date_validates_bounds() {
        assert!(ChequeDate::new(2026, 6, 19).is_ok());
        assert!(ChequeDate::new(2026, 0, 1).is_err());
        assert!(ChequeDate::new(2026, 13, 1).is_err());
        assert!(ChequeDate::new(2026, 6, 0).is_err());
        assert!(ChequeDate::new(2026, 6, 32).is_err());
    }

    #[test]
    fn cheque_date_displays_iso_8601() {
        let d = ChequeDate::new(2026, 6, 19).unwrap();
        assert_eq!(d.to_string(), "2026-06-19");
    }

    #[test]
    fn payment_method_kind_covers_all_six_variants() {
        let g = educore_core::clock::SystemIdGen;
        // Cash
        assert_eq!(PaymentMethod::Cash.kind(), PaymentMethodKind::Cash);
        // Cheque
        let cheque = PaymentMethod::Cheque {
            number: String::from("000123"),
            bank: String::from("HDFC"),
            date: ChequeDate::new(2026, 6, 19).unwrap(),
        };
        assert_eq!(cheque.kind(), PaymentMethodKind::Cheque);
        // BankTransfer
        assert_eq!(
            PaymentMethod::BankTransfer {
                reference: String::from("UTR123"),
                bank_account: BankAccountId::new("acct_123"),
            }
            .kind(),
            PaymentMethodKind::BankTransfer
        );
        // Card
        assert_eq!(
            PaymentMethod::Card {
                token: CardToken::new("tok_123"),
                save: false,
            }
            .kind(),
            PaymentMethodKind::Card
        );
        // Gateway
        assert_eq!(
            PaymentMethod::Gateway {
                gateway: GatewayName::new("stripe"),
                token: GatewayToken::new("pi_123"),
                return_url: String::from("https://example.com/return"),
            }
            .kind(),
            PaymentMethodKind::Gateway
        );
        // Wallet
        assert_eq!(
            PaymentMethod::Wallet {
                wallet_id: WalletId::new("wal_123"),
                pin: String::from("0000"),
            }
            .kind(),
            PaymentMethodKind::Wallet
        );
        // ManualAdjustment
        assert_eq!(
            PaymentMethod::ManualAdjustment {
                reason: String::from("fee waiver"),
                approver: g.next_user_id(),
            }
            .kind(),
            PaymentMethodKind::ManualAdjustment
        );
    }

    #[test]
    fn customer_ref_external_carries_opaque_id() {
        let ext = CustomerRef::External(CustomerId::new("parent_42"));
        match ext {
            CustomerRef::External(id) => assert_eq!(id.as_str(), "parent_42"),
            _ => panic!("expected External variant"),
        }
    }

    #[test]
    fn charge_request_defaults_capture_true() {
        let g = educore_core::clock::SystemIdGen;
        let req = ChargeRequest::new(
            ctx(),
            Money::new(usd(), 1500).unwrap(),
            PaymentMethod::Cash,
            CustomerRef::User(g.next_user_id()),
            g.next_idempotency_key(),
            g.next_correlation_id(),
        );
        assert!(req.capture);
        assert!(req.invoice_ids.is_empty());
        assert!(req.metadata.is_empty());
        assert!(req.webhook_url.is_none());
    }
}
