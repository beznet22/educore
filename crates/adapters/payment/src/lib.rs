//! # educore-payment
//!
//! Payment port, cash, cheque, bank, gateway, wallet adapter implementations.
//!
//! This crate is a member of the Educore workspace. See
//! `docs/architecture.md` and the port contract in
//! `docs/ports/payments.md` for behavioral details.
//!
//! # Module map
//!
//! - [`port`] — the [`PaymentProvider`](port::PaymentProvider)
//!   trait (5 methods, object-safe) and all the supporting
//!   request, response, and value types (`ChargeRequest`,
//!   `PaymentReceipt`, `RefundRequest`, `Settlement`, …).
//! - [`errors`] — the [`PaymentError`](errors::PaymentError) enum
//!   returned by every adapter method.
//! - [`stripe`] — the [`StripeProvider`](stripe::StripeProvider)
//!   reference implementation that targets the Stripe REST API
//!   and provides Stripe-Signature HMAC-SHA256 webhook
//!   verification.
//! - [`services`] — pure helper structs
//!   ([`IdempotencyService`](services::IdempotencyService),
//!   [`WebhookSignatureService`](services::WebhookSignatureService),
//!   [`BankSlipService`](services::BankSlipService),
//!   [`SettlementService`](services::SettlementService)) used by
//!   the adapters to keep their hot paths tight.
//!
//! # Deviations from the spec
//!
//! The crate's `Cargo.toml` is intentionally minimal (only
//! `core`, `platform`, `events`, `tokio`, `async-trait`), so the
//! port uses **stdlib-only** value representations. See the
//! module-level doc in [`port`] for the full list. Adapters that
//! want the spec's idiomatic types (`uuid::Uuid`, `url::Url`,
//! `chrono::NaiveDate`, `secrecy::SecretString`) may wrap the
//! stdlib shapes at their boundary.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// Error types for the [`PaymentProvider`](port::PaymentProvider) port.
pub mod errors;

/// The [`PaymentProvider`](port::PaymentProvider) trait and all
/// supporting types (`ChargeRequest`, `PaymentReceipt`,
/// `RefundRequest`, `Settlement`, …).
pub mod port;

/// Stripe reference implementation of [`PaymentProvider`](port::PaymentProvider).
pub mod stripe;

/// Service helpers (idempotency, webhook signature, bank slip, settlement).
pub mod services;

/// Webhook signature verification with constant-time comparison,
/// replay-window enforcement, and signing-key rotation.
pub mod webhook_security;

/// Package name constant. Re-exported so consumers can assert
/// they are using the right crate version at compile time.
pub const PACKAGE_NAME: &str = "educore-payment";

/// Package version at compile time.
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Convenience re-exports of the port's most-used types.
///
/// Consumers of the port should
/// `use educore_payment::prelude::*;` once at the top of a file
/// to pull in the trait, the request/response shapes, and the
/// error type without naming each module.
pub mod prelude {
    pub use crate::errors::{InfrastructureError, PaymentError};
    pub use crate::port::{
        BankAccountId, CardToken, ChargeRequest, ChequeDate, CurrencyCode, CustomerId, CustomerRef,
        GatewayName, GatewayToken, InvoiceId, Money, PaymentFee, PaymentId, PaymentMethod,
        PaymentMethodInfo, PaymentMethodKind, PaymentProvider, PaymentReceipt, PaymentStatus,
        RefundDestination, RefundReceipt, RefundRequest, Settlement, SettlementLine,
        SettlementRequest, WalletId,
    };
    pub use crate::services::{
        BankSlipService, IdempotencyService, SettlementService, WebhookSignatureService,
    };
    pub use crate::stripe::{StripeProvider, StripeProviderBuilder, ThreeDSChallengeResult};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_metadata_is_set() {
        assert_eq!(PACKAGE_NAME, "educore-payment");
        assert!(!PACKAGE_VERSION.is_empty());
    }
}
