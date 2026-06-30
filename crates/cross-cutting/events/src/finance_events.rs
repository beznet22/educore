//! # Finance domain events
//!
//! Cross-cutting typed events emitted by the finance domain.
//! Lives in the `educore-events` crate so consumers can
//! subscribe without depending on `educore-finance` directly.
//!
//! Per `docs/specs/finance/aggregates.md` and the payment port
//! contract: settlement reconciliation fires a `PaymentSettled`
//! event per settled payment so downstream subscribers (audit
//! mirror, sync engine, notification) can react.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::{EventId, Identifier, SchoolId};
use educore_core::value_objects::Timestamp;

use crate::domain_event::DomainEvent;

/// Internal helper: constructs an `EventId` from a `Uuid`.
#[inline]
fn event_id_from(uuid: Uuid) -> EventId {
    EventId::from_uuid(uuid)
}

/// A payment was settled by the provider. Emitted by the
/// finance domain's settlement reconciliation worker after a
/// successful provider response confirms the payment.
///
/// The event carries:
/// - `payment_id` — the engine-internal payment id
/// - `provider_payment_id` — the provider's id (Stripe
///   `ch_xxx`, Razorpay `pay_xxx`, etc.)
/// - `settlement_id` — the engine-internal settlement id
///   (matches a `SettlementLine.provider_payment_id`)
/// - `amount_minor_units` — the settled amount in the
///   currency's minor units (e.g. cents, paise)
/// - `currency` — ISO 4217 three-letter code
/// - `settled_at` — when the provider confirmed the settlement
/// - `occurred_at` — when the engine emitted the event
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentSettled {
    /// The mint-time event id (UUIDv7).
    pub event_id: Uuid,
    /// The tenant anchor.
    pub school_id: SchoolId,
    /// The engine-internal payment id.
    pub payment_id: Uuid,
    /// The provider's payment id (Stripe `ch_xxx`, etc.).
    pub provider_payment_id: String,
    /// The engine-internal settlement id (matches
    /// `SettlementLine.settlement_id`).
    pub settlement_id: Uuid,
    /// The settled amount in the currency's minor units
    /// (e.g. cents, paise).
    pub amount_minor_units: i64,
    /// ISO 4217 three-letter currency code (e.g. `"USD"`).
    pub currency: String,
    /// When the provider confirmed the settlement.
    pub settled_at: Timestamp,
    /// When the engine emitted the event.
    pub occurred_at: Timestamp,
}

impl PaymentSettled {
    /// Mints a fresh `PaymentSettled` with a new UUIDv7
    /// `event_id`.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        school_id: SchoolId,
        payment_id: Uuid,
        provider_payment_id: String,
        settlement_id: Uuid,
        amount_minor_units: i64,
        currency: String,
        settled_at: Timestamp,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            event_id: Uuid::now_v7(),
            school_id,
            payment_id,
            provider_payment_id,
            settlement_id,
            amount_minor_units,
            currency,
            settled_at,
            occurred_at,
        }
    }
}

impl DomainEvent for PaymentSettled {
    /// Stable dotted event-type string. The subscription key
    /// for consumers is `"finance.payment.settled"`.
    const EVENT_TYPE: &'static str = "finance.payment.settled";
    /// Schema version of the payload. Bumped on
    /// backward-incompatible payload changes.
    const SCHEMA_VERSION: u32 = 1;
    /// The aggregate type. Settlements live on the
    /// `payment_settlement` aggregate.
    const AGGREGATE_TYPE: &'static str = "payment_settlement";

    fn event_id(&self) -> EventId {
        event_id_from(self.event_id)
    }
    fn aggregate_id(&self) -> Uuid {
        self.settlement_id
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
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
    use educore_core::clock::IdGenerator;

    #[test]
    fn event_type_is_finance_payment_settled() {
        assert_eq!(PaymentSettled::EVENT_TYPE, "finance.payment.settled");
        assert_eq!(PaymentSettled::SCHEMA_VERSION, 1);
        assert_eq!(PaymentSettled::AGGREGATE_TYPE, "payment_settlement");
    }

    #[test]
    fn new_mints_v7_event_id_and_carries_fields() {
        let g = educore_core::clock::SystemIdGen;
        let school = g.next_school_id();
        let payment_id = uuid::Uuid::now_v7();
        let settlement_id = uuid::Uuid::now_v7();
        let now = Timestamp::now();
        let e = PaymentSettled::new(
            school,
            payment_id,
            "ch_abc123".to_owned(),
            settlement_id,
            12_500,
            "USD".to_owned(),
            now,
            now,
        );
        assert_eq!(e.school_id, school);
        assert_eq!(e.payment_id, payment_id);
        assert_eq!(e.settlement_id, settlement_id);
        assert_eq!(e.provider_payment_id, "ch_abc123");
        assert_eq!(e.amount_minor_units, 12_500);
        assert_eq!(e.currency, "USD");
        // The mint-time event id must be a UUIDv7.
        assert_eq!(e.event_id.get_version_num(), 7);
        // DomainEvent trait returns the same id and timestamp.
        assert_eq!(e.event_id().as_uuid(), e.event_id);
        assert_eq!(e.school_id(), school);
        assert_eq!(e.occurred_at(), now);
        // aggregate_id points at the settlement id.
        assert_eq!(e.aggregate_id(), settlement_id);
    }

    #[test]
    fn event_serializes_round_trip() {
        let g = educore_core::clock::SystemIdGen;
        let school = g.next_school_id();
        let now = Timestamp::now();
        let e = PaymentSettled::new(
            school,
            uuid::Uuid::now_v7(),
            "ch_xyz".to_owned(),
            uuid::Uuid::now_v7(),
            999,
            "EUR".to_owned(),
            now,
            now,
        );
        let json = serde_json::to_string(&e).expect("serialize");
        let back: PaymentSettled = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(e, back);
    }
}
