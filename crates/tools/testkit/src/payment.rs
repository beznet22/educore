//! In-memory `PaymentProvider` adapter.
//!
//! Implements [`PaymentProvider`] against an in-process
//! `parking_lot::Mutex<HashMap<...>>` so consumer tests can
//! exercise the engine's charge / refund / settlement code paths
//! without a real payment gateway.
//!
//! # Idempotency
//!
//! Both `charge` and `refund` are idempotent on
//! [`ChargeRequest::idempotency_key`](educore_payment::port::ChargeRequest::idempotency_key)
//! and
//! [`RefundRequest::idempotency_key`](educore_payment::port::RefundRequest::idempotency_key).
//! A retry with the same key returns the original receipt without
//! minting a new one. This matches the port contract in
//! `docs/ports/payments.md` § "Idempotency".
//!
//! # Settlement
//!
//! `settlement` always returns an empty settlement batch
//! (`lines = vec![]`, all totals zero in the requested currency).
//! Tests that need realistic settlement data should
//! `provider.charge(...)` first and then post-process the receipts
//! directly; the in-memory adapter does not auto-link charges to
//! settlement lines.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use educore_core::ids::IdempotencyKey;
use educore_core::tenant::TenantContext;
use educore_core::value_objects::Timestamp;
use educore_payment::port::{
    ChargeRequest, CurrencyCode, Money, PaymentId, PaymentMethodInfo, PaymentMethodKind,
    PaymentProvider, PaymentReceipt, PaymentStatus, RefundReceipt, RefundRequest, Settlement,
    SettlementRequest,
};
use parking_lot::Mutex;

/// In-memory `PaymentProvider` backed by a `HashMap`.
#[derive(Debug, Default)]
pub struct InMemoryPaymentProvider {
    /// Idempotency-key → issued `PaymentReceipt`. A retry of the
    /// same `ChargeRequest::idempotency_key` returns the stored
    /// receipt unchanged.
    charges: Mutex<HashMap<IdempotencyKey, PaymentReceipt>>,
    /// Idempotency-key → issued `RefundReceipt`. A retry of the
    /// same `RefundRequest::idempotency_key` returns the stored
    /// receipt unchanged.
    refunds: Mutex<HashMap<IdempotencyKey, RefundReceipt>>,
    /// Monotonic counter used to mint engine-issued `PaymentId`
    /// values (`in-mem-charge-1`, `in-mem-charge-2`, …) and the
    /// paired gateway ids.
    id_seq: AtomicU64,
}

impl InMemoryPaymentProvider {
    /// Constructs a fresh, empty provider.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the next monotonic id without bumping the counter.
    fn peek_id(&self) -> u64 {
        self.id_seq
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_add(1)
    }

    /// Looks up a stored charge by its engine-issued
    /// [`PaymentId`]. Returns the receipt's [`PaymentStatus`] if
    /// found, or a synthetic `Failed { reason: "not found" }`
    /// otherwise.
    fn lookup_status(&self, payment_id: &PaymentId) -> PaymentStatus {
        let charges = self.charges.lock();
        charges
            .values()
            .find(|receipt| receipt.payment_id == *payment_id)
            .map(|receipt| receipt.status.clone())
            .unwrap_or_else(|| PaymentStatus::Failed {
                reason: "not found".to_owned(),
                code: None,
            })
    }
}

#[async_trait]
impl PaymentProvider for InMemoryPaymentProvider {
    async fn charge(
        &self,
        request: ChargeRequest,
    ) -> Result<PaymentReceipt, educore_payment::errors::PaymentError> {
        let mut charges = self.charges.lock();
        if let Some(existing) = charges.get(&request.idempotency_key) {
            return Ok(existing.clone());
        }

        let next = self.peek_id();
        let now = Timestamp::now();
        let receipt = PaymentReceipt {
            payment_id: PaymentId::new(format!("in-mem-charge-{next}")),
            provider_payment_id: Some(format!("in-mem-prov-{next}")),
            status: PaymentStatus::Captured { at: now },
            amount: request.amount.clone(),
            method: request.method.kind(),
            authorized_at: Some(now),
            captured_at: Some(now),
            fees: Vec::new(),
            net: request.amount.clone(),
            receipt_url: None,
            metadata: request.metadata.clone(),
        };

        charges.insert(request.idempotency_key, receipt.clone());
        Ok(receipt)
    }

    async fn refund(
        &self,
        request: RefundRequest,
    ) -> Result<RefundReceipt, educore_payment::errors::PaymentError> {
        let mut refunds = self.refunds.lock();
        if let Some(existing) = refunds.get(&request.idempotency_key) {
            return Ok(existing.clone());
        }

        let next = self.peek_id();
        let now = Timestamp::now();
        let receipt = RefundReceipt {
            refund_id: PaymentId::new(format!("in-mem-refund-{next}")),
            original_payment_id: request.original_payment_id.clone(),
            provider_refund_id: Some(format!("in-mem-prov-refund-{next}")),
            amount: request.amount.clone(),
            status: PaymentStatus::Captured { at: now },
            refunded_at: Some(now),
            destination: request.refund_to.clone(),
            metadata: Default::default(),
        };

        refunds.insert(request.idempotency_key, receipt.clone());
        Ok(receipt)
    }

    async fn status(
        &self,
        payment_id: PaymentId,
    ) -> Result<PaymentStatus, educore_payment::errors::PaymentError> {
        Ok(self.lookup_status(&payment_id))
    }

    async fn list_methods(
        &self,
        _tenant: TenantContext,
    ) -> Result<Vec<PaymentMethodInfo>, educore_payment::errors::PaymentError> {
        Ok(vec![PaymentMethodInfo {
            kind: PaymentMethodKind::Cash,
            display_name: "Cash".to_owned(),
            enabled: true,
            note: None,
        }])
    }

    async fn settlement(
        &self,
        request: SettlementRequest,
    ) -> Result<Settlement, educore_payment::errors::PaymentError> {
        let zero = match Money::new(request.currency.clone(), 0) {
            Ok(m) => m,
            Err(_) => Money::zero(request.currency.clone()),
        };
        Ok(Settlement {
            settlement_id: "in-mem-settlement-1".to_owned(),
            school_id: request.tenant.school_id,
            currency: request.currency.clone(),
            period_start: request.period_start,
            period_end: request.period_end,
            lines: Vec::new(),
            total_gross: zero.clone(),
            total_fees: zero.clone(),
            total_net: zero,
        })
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
    use educore_core::clock::{IdGenerator, SystemIdGen};
    use educore_core::tenant::UserType;
    use educore_payment::port::{CustomerRef, PaymentMethod};

    fn usd() -> CurrencyCode {
        CurrencyCode::new("USD").unwrap()
    }

    fn ctx() -> TenantContext {
        let g = SystemIdGen;
        TenantContext::for_user(
            g.next_school_id(),
            g.next_user_id(),
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        )
    }

    fn charge_request(amount_minor: i64) -> ChargeRequest {
        let g = SystemIdGen;
        let amount = Money::new(usd(), amount_minor).unwrap();
        ChargeRequest::new(
            ctx(),
            amount,
            PaymentMethod::Cash,
            CustomerRef::User(g.next_user_id()),
            g.next_idempotency_key(),
            g.next_correlation_id(),
        )
    }

    fn refund_request(original: PaymentId, amount_minor: i64) -> RefundRequest {
        use educore_payment::port::RefundDestination;
        RefundRequest {
            tenant: ctx(),
            original_payment_id: original,
            amount: Money::new(usd(), amount_minor).unwrap(),
            reason: "customer changed mind".to_owned(),
            refund_to: RefundDestination::OriginalMethod,
            idempotency_key: SystemIdGen.next_idempotency_key(),
        }
    }

    #[test]
    fn charge_mints_payment_receipt() {
        let provider = InMemoryPaymentProvider::new();
        let request = charge_request(1500);
        let receipt = futures::executor::block_on(provider.charge(request)).unwrap();
        assert_eq!(receipt.amount, Money::new(usd(), 1500).unwrap());
        assert!(matches!(receipt.status, PaymentStatus::Captured { .. }));
        assert_eq!(receipt.method, PaymentMethodKind::Cash);
        assert_eq!(receipt.fees, Vec::<educore_payment::port::PaymentFee>::new());
        assert_eq!(receipt.net, receipt.amount);
        assert!(receipt.payment_id.as_str().starts_with("in-mem-charge-"));
        assert!(receipt.provider_payment_id.is_some());
        assert!(receipt.authorized_at.is_some());
        assert!(receipt.captured_at.is_some());
    }

    #[test]
    fn charge_same_idempotency_key_returns_same_receipt() {
        let provider = InMemoryPaymentProvider::new();
        let request = charge_request(2000);
        let key = request.idempotency_key;

        let first = futures::executor::block_on(provider.charge(request.clone())).unwrap();
        let second = futures::executor::block_on(provider.charge(request)).unwrap();

        assert_eq!(first, second);
        assert_eq!(first.payment_id, second.payment_id);
        let charges = provider.charges.lock();
        assert_eq!(charges.len(), 1);
        assert!(charges.contains_key(&key));
    }

    #[test]
    fn refund_mints_refund_receipt() {
        let provider = InMemoryPaymentProvider::new();
        let original = futures::executor::block_on(provider.charge(charge_request(3000)))
            .unwrap()
            .payment_id;

        let refund = futures::executor::block_on(provider.refund(refund_request(original.clone(), 500)))
            .unwrap();
        assert_eq!(refund.original_payment_id, original);
        assert!(matches!(refund.status, PaymentStatus::Captured { .. }));
        assert!(refund.refunded_at.is_some());
        assert!(refund.refund_id.as_str().starts_with("in-mem-refund-"));
    }

    #[test]
    fn status_returns_captured_for_charged_payment() {
        let provider = InMemoryPaymentProvider::new();
        let receipt = futures::executor::block_on(provider.charge(charge_request(999))).unwrap();
        let status =
            futures::executor::block_on(provider.status(receipt.payment_id.clone())).unwrap();
        assert!(matches!(status, PaymentStatus::Captured { .. }));
    }

    #[test]
    fn status_returns_failed_for_unknown_payment_id() {
        let provider = InMemoryPaymentProvider::new();
        let status = futures::executor::block_on(provider.status(PaymentId::new("nope"))).unwrap();
        match status {
            PaymentStatus::Failed { reason, code } => {
                assert_eq!(reason, "not found");
                assert!(code.is_none());
            }
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    #[test]
    fn list_methods_returns_at_least_one_method() {
        let provider = InMemoryPaymentProvider::new();
        let methods =
            futures::executor::block_on(provider.list_methods(ctx())).unwrap();
        assert!(!methods.is_empty());
        let cash = methods
            .iter()
            .find(|m| m.kind == PaymentMethodKind::Cash)
            .expect("Cash method must be present");
        assert!(cash.enabled);
        assert_eq!(cash.display_name, "Cash");
        assert!(cash.note.is_none());
    }

    #[test]
    fn settlement_returns_empty_batch_in_requested_currency() {
        use educore_payment::port::SettlementRequest;
        let provider = InMemoryPaymentProvider::new();
        let req = SettlementRequest {
            tenant: ctx(),
            period_start: Timestamp::now(),
            period_end: Timestamp::now(),
            currency: usd(),
        };
        let settlement = futures::executor::block_on(provider.settlement(req)).unwrap();
        assert_eq!(settlement.settlement_id, "in-mem-settlement-1");
        assert!(settlement.lines.is_empty());
        assert_eq!(settlement.total_gross, Money::new(usd(), 0).unwrap());
        assert_eq!(settlement.total_fees, Money::new(usd(), 0).unwrap());
        assert_eq!(settlement.total_net, Money::new(usd(), 0).unwrap());
        assert_eq!(settlement.currency, usd());
    }

    #[test]
    fn id_seq_starts_at_one_and_increments() {
        let provider = InMemoryPaymentProvider::new();
        assert_eq!(provider.peek_id(), 1);
        assert_eq!(provider.peek_id(), 2);
        assert_eq!(provider.peek_id(), 3);
    }
}