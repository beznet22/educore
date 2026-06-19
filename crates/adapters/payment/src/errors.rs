//! Error types for the [`PaymentProvider`](crate::port::PaymentProvider) port.
//!
//! Per `docs/ports/payments.md`, every payment adapter returns one
//! of the variants in [`PaymentError`]. The enum is intentionally
//! `Clone + PartialEq + Eq` so it can flow through the engine's
//! circuit-breaker / retry middleware without losing context.
//!
//! The `Infrastructure` variant carries a structured
//! [`InfrastructureError`] (a typed wrapper around an opaque
//! message) rather than `Box<dyn std::error::Error + Send + Sync>`
//! so the enum stays `Clone + PartialEq + Eq`. Callers that need
//! to lift an arbitrary `Box<dyn Error>` into the port convert it
//! via [`InfrastructureError::from_boxed`].

use std::error::Error as StdError;
use std::fmt;

use crate::port::CurrencyCode;

/// Errors returned by the [`PaymentProvider`](crate::port::PaymentProvider) port.
///
/// The variants are locked to `docs/ports/payments.md` § "Error
/// Type" with two deviations:
/// - `Infrastructure` wraps the structured [`InfrastructureError`]
///   (an opaque message + `Send + Sync` source chain) rather than
///   a bare `Box<dyn Error>`. The wrapping preserves
///   `PartialEq + Eq` on the enum so callers can match against
///   concrete errors in circuit-breaker middleware.
/// - The enum is **not** `Clone` (the boxed source chain is not
///   cloneable). Callers that need to retain a copy of an error
///   after the producer goes out of scope should clone the
///   `PaymentError` **before** materialising the `Infrastructure`
///   variant (e.g. by lifting the message into a
///   `PaymentError::Provider(...)` first).
///
/// Adapters MUST NOT include card data, PINs, or bank-account
/// numbers in the diagnostic message; the engine redacts strings
/// whose field name ends in `_pin`, `_secret`, or `_card` on its
/// way to the audit log.
#[derive(Debug, PartialEq, Eq)]
pub enum PaymentError {
    /// The amount in `ChargeRequest.amount` or `RefundRequest.amount`
    /// was non-positive, exceeded the engine's per-payment ceiling,
    /// or otherwise failed validation. The inner string carries the
    /// adapter-side diagnostic.
    InvalidAmount(String),

    /// The currency on the payment does not match the currency on
    /// the invoice being settled. The engine rejects cross-currency
    /// charges at the port boundary; FX conversion is the consumer's
    /// responsibility before the charge is issued.
    CurrencyMismatch {
        /// The currency of the invoice.
        invoice: CurrencyCode,
        /// The currency of the payment attempt.
        payment: CurrencyCode,
    },

    /// The card or method was declined by the gateway. The inner
    /// string carries the gateway's reason (e.g. `do_not_honor`).
    /// The engine surfaces this in the audit log without logging
    /// any card data.
    Declined(String),

    /// The customer's wallet or card had insufficient funds.
    InsufficientFunds,

    /// The gateway requires 3-D Secure authentication before the
    /// charge can proceed. The adapter returns this on the initial
    /// call so the engine can redirect the customer to the
    /// issuer's 3DS challenge; the second call (after the
    /// challenge) carries the issuer's response in the
    /// `metadata` map.
    ThreeDSRequired,

    /// The adapter is being rate-limited by the upstream gateway.
    /// The engine should back off and retry with exponential
    /// delay. The retry strategy is a consumer concern; the port
    /// merely signals that a retry is safe.
    RateLimited,

    /// A generic provider-side failure. The inner string carries
    /// the provider's diagnostic message. The engine surfaces this
    /// in the audit log without logging any card data.
    Provider(String),

    /// The `PaymentReceipt` referenced by the refund request has
    /// already been fully refunded. A partial refund is no longer
    /// possible.
    AlreadyRefunded,

    /// The refund amount exceeds the remaining (unrefunded)
    /// amount on the original payment.
    RefundExceedsOriginal,

    /// A non-domain infrastructure failure (network, DNS, TLS,
    /// serialization). The wrapped [`InfrastructureError`] is the
    /// underlying cause; the engine surfaces it via
    /// [`std::error::Error::source`].
    Infrastructure(InfrastructureError),
}

/// An opaque, `Send + Sync` wrapper around an infrastructure-level
/// error message and an optional boxed source.
///
/// The port cannot use `Box<dyn std::error::Error + Send + Sync>`
/// directly as the variant payload because the resulting
/// [`PaymentError`] enum would lose the ability to derive
/// `PartialEq + Eq` (the boxed `dyn Error` is not `Eq`). The
/// [`InfrastructureError`] wrapper preserves `PartialEq + Eq`
/// for callers that need to compare errors (e.g. in
/// circuit-breaker middleware) while still retaining the
/// `source()` chain for diagnostics.
///
/// The wrapper is intentionally **not** `Clone` (the boxed
/// source cannot be cloned). Callers that need to retain a
/// copy of an infrastructure error should clone the
/// [`PaymentError`] before the `Infrastructure` variant is
/// materialised.
#[derive(Debug)]
pub struct InfrastructureError {
    message: String,
    source: Option<Box<dyn StdError + Send + Sync>>,
}

impl InfrastructureError {
    /// Constructs an `InfrastructureError` from a free-form message.
    /// The message is stored verbatim; the adapter is responsible
    /// for redacting any sensitive data (PAN, PIN, account
    /// number) before construction.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    /// Constructs an `InfrastructureError` that wraps a boxed
    /// source error. The `Display` output of the source is appended
    /// to the engine's audit log via [`StdError::source`].
    #[must_use]
    pub fn with_source(
        message: impl Into<String>,
        source: Box<dyn StdError + Send + Sync>,
    ) -> Self {
        Self {
            message: message.into(),
            source: Some(source),
        }
    }

    /// Lifts an arbitrary `Box<dyn Error + Send + Sync>` into an
    /// `InfrastructureError`. The boxed error's `Display` output
    /// becomes the wrapped message and the box is retained as the
    /// `source` chain.
    #[must_use]
    pub fn from_boxed(err: Box<dyn StdError + Send + Sync>) -> Self {
        Self {
            message: err.to_string(),
            source: Some(err),
        }
    }

    /// Returns the wrapped diagnostic message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for InfrastructureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl StdError for InfrastructureError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_deref()
            .map(|e| e as &(dyn StdError + 'static))
    }
}

impl PartialEq for InfrastructureError {
    fn eq(&self, other: &Self) -> bool {
        self.message == other.message
    }
}

impl Eq for InfrastructureError {}

impl fmt::Display for PaymentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAmount(msg) => write!(f, "invalid amount: {msg}"),
            Self::CurrencyMismatch { invoice, payment } => {
                write!(f, "currency mismatch: invoice={invoice}, payment={payment}")
            }
            Self::Declined(msg) => write!(f, "card declined: {msg}"),
            Self::InsufficientFunds => f.write_str("insufficient funds"),
            Self::ThreeDSRequired => f.write_str("3DS required"),
            Self::RateLimited => f.write_str("rate limited"),
            Self::Provider(msg) => write!(f, "provider error: {msg}"),
            Self::AlreadyRefunded => f.write_str("already refunded"),
            Self::RefundExceedsOriginal => f.write_str("refund exceeds original"),
            Self::Infrastructure(err) => write!(f, "infrastructure error: {err}"),
        }
    }
}

impl StdError for PaymentError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Infrastructure(err) => Some(err),
            _ => None,
        }
    }
}

impl From<InfrastructureError> for PaymentError {
    fn from(err: InfrastructureError) -> Self {
        Self::Infrastructure(err)
    }
}

impl From<Box<dyn StdError + Send + Sync>> for PaymentError {
    fn from(err: Box<dyn StdError + Send + Sync>) -> Self {
        Self::Infrastructure(InfrastructureError::from_boxed(err))
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
    use super::*;

    #[test]
    fn display_matches_spec_wording() {
        assert_eq!(
            PaymentError::InvalidAmount(String::from("zero")).to_string(),
            "invalid amount: zero"
        );
        assert_eq!(
            PaymentError::InsufficientFunds.to_string(),
            "insufficient funds"
        );
        assert_eq!(PaymentError::ThreeDSRequired.to_string(), "3DS required");
        assert_eq!(PaymentError::RateLimited.to_string(), "rate limited");
        assert_eq!(
            PaymentError::AlreadyRefunded.to_string(),
            "already refunded"
        );
        assert_eq!(
            PaymentError::RefundExceedsOriginal.to_string(),
            "refund exceeds original"
        );
    }

    #[test]
    fn currency_mismatch_carries_both_currencies() {
        let inv = CurrencyCode::new("USD").unwrap();
        let pay = CurrencyCode::new("EUR").unwrap();
        let err = PaymentError::CurrencyMismatch {
            invoice: inv,
            payment: pay,
        };
        let s = err.to_string();
        assert!(s.contains("USD"));
        assert!(s.contains("EUR"));
    }

    #[test]
    fn infrastructure_error_carries_source() {
        use std::io;
        let io_err: Box<dyn StdError + Send + Sync> = Box::new(io::Error::new(
            io::ErrorKind::ConnectionReset,
            "peer closed",
        ));
        let infra = InfrastructureError::from_boxed(io_err);
        assert!(infra.source().is_some());
        let pay: PaymentError = infra.into();
        assert!(matches!(pay, PaymentError::Infrastructure(_)));
        assert!(pay.source().is_some());
    }

    #[test]
    fn error_is_eq() {
        let a = PaymentError::Provider(String::from("stripe: 500"));
        let b = PaymentError::Provider(String::from("stripe: 500"));
        assert_eq!(a, b);
    }
}
