//! # In-memory [`NotificationProvider`](educore_notify::port::NotificationProvider)
//!
//! Test-only
//! [`NotificationProvider`](educore_notify::port::NotificationProvider)
//! that records every send in a process-local `Vec` and returns
//! synthetic `in-memory-`-prefixed receipt ids.
//!
//! ## Behaviour
//!
//! - [`send`](educore_notify::port::NotificationProvider::send)
//!   pushes the request onto an internal `Vec` and returns a
//!   [`NotificationReceipt`] with a synthetic receipt id
//!   (`"in-memory-<uuid>"`), a synthetic
//!   `"in-memory-provider-msg-id"` provider message id, and
//!   [`DeliveryStatus::Sent`](educore_notify::port::DeliveryStatus::Sent).
//! - [`send_bulk`](educore_notify::port::NotificationProvider::send_bulk)
//!   generates one [`NotificationReceipt`] per
//!   [`BulkRecipient`](educore_notify::port::BulkRecipient) and
//!   stores the resulting
//!   [`BulkReceipt`](educore_notify::port::BulkReceipt) in an
//!   internal `HashMap<BulkId, BulkReceipt>` keyed by a
//!   synthetic bulk id.
//! - [`status`](educore_notify::port::NotificationProvider::status)
//!   always returns
//!   [`DeliveryStatus::Sent`](educore_notify::port::DeliveryStatus::Sent).
//!   The in-memory adapter does not track per-receipt status
//!   beyond the synthetic `Sent` reported at send-time.

#![allow(clippy::missing_docs_in_private_items)]

use std::collections::HashMap;
use std::sync::{Mutex, PoisonError};

use async_trait::async_trait;
use educore_core::value_objects::Timestamp;
use educore_notify::port::{
    BulkId, BulkReceipt, DeliveryStatus, NotificationProvider, NotificationReceipt,
    NotificationReceiptId, Result, SendBulkNotification, SendNotification,
};
use uuid::Uuid;

/// The synthetic provider message id stamped on every
/// in-memory receipt. A constant string rather than a UUID so
/// test assertions are stable.
const IN_MEMORY_PROVIDER_MSG_ID: &str = "in-memory-provider-msg-id";

/// The prefix prepended to every synthetic receipt id, bulk id,
/// and similar opaque identifier minted by this adapter.
const IN_MEMORY_ID_PREFIX: &str = "in-memory-";

/// In-memory [`NotificationProvider`] backed by process-local
/// `Vec` and `HashMap` stores. Cheap to construct, safe to share
/// across tasks via `Arc`.
#[derive(Debug, Default)]
pub struct InMemoryNotificationProvider {
    sends: Mutex<Vec<SendNotification>>,
    bulks: Mutex<HashMap<BulkId, BulkReceipt>>,
}

impl InMemoryNotificationProvider {
    /// Constructs a fresh in-memory notification provider with
    /// empty send and bulk stores.
    #[must_use]
    pub fn new() -> Self {
        Self {
            sends: Mutex::new(Vec::new()),
            bulks: Mutex::new(HashMap::new()),
        }
    }

    /// Returns the number of single sends recorded so far.
    #[must_use]
    pub fn send_count(&self) -> usize {
        self.sends
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .len()
    }

    /// Returns the number of bulk sends recorded so far.
    #[must_use]
    pub fn bulk_count(&self) -> usize {
        self.bulks
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .len()
    }

    /// Builds the synthetic [`NotificationReceipt`] returned by
    /// both [`send`](Self::send) and
    /// [`send_bulk`](Self::send_bulk). Centralised so the
    /// receipt shape stays consistent across the two methods.
    fn make_receipt(channel: &educore_notify::port::Channel) -> NotificationReceipt {
        let receipt_id =
            NotificationReceiptId::new(format!("{IN_MEMORY_ID_PREFIX}{}", Uuid::new_v4()));
        NotificationReceipt {
            receipt_id,
            provider_message_id: Some(IN_MEMORY_PROVIDER_MSG_ID.to_owned()),
            channel: channel.clone(),
            status: DeliveryStatus::Sent,
            cost: None,
            sent_at: Timestamp::now(),
            metadata: std::collections::BTreeMap::new(),
        }
    }
}

#[async_trait]
impl NotificationProvider for InMemoryNotificationProvider {
    async fn send(&self, request: SendNotification) -> Result<NotificationReceipt> {
        let receipt = Self::make_receipt(&request.channel);
        let mut sends = self
            .sends
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        sends.push(request);
        Ok(receipt)
    }

    async fn send_bulk(
        &self,
        request: SendBulkNotification,
    ) -> Result<BulkReceipt> {
        let bulk_id = BulkId::new(format!(
            "in-memory-bulk-{}",
            Uuid::new_v4()
        ));
        let receipts: Vec<NotificationReceipt> = request
            .recipients
            .iter()
            .map(|_row| Self::make_receipt(&request.channel))
            .collect();

        let bulk_receipt = BulkReceipt {
            bulk_id: bulk_id.clone(),
            receipts,
            failed: Vec::new(),
        };

        let mut bulks = self
            .bulks
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        bulks.insert(bulk_id, bulk_receipt.clone());
        Ok(bulk_receipt)
    }

    async fn status(
        &self,
        _receipt_id: NotificationReceiptId,
    ) -> Result<DeliveryStatus> {
        Ok(DeliveryStatus::Sent)
    }
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

    use std::collections::BTreeMap;

    use educore_core::ids::{CorrelationId, SchoolId, UserId};
    use educore_core::tenant::{TenantContext, UserType};
    use educore_notify::errors::NotificationTemplateId;
    use educore_notify::port::{
        BulkRecipient, Channel, ContactInfo, EmailAddress, Priority, Recipient,
        TemplateRef,
    };
    use uuid::Uuid;

    fn system_tenant() -> TenantContext {
        let school = SchoolId(Uuid::nil());
        let user = UserId(Uuid::nil());
        let corr = CorrelationId(Uuid::nil());
        TenantContext::for_user(school, user, corr, UserType::System)
    }

    fn make_send(channel: Channel) -> SendNotification {
        SendNotification {
            tenant: system_tenant(),
            channel,
            template: TemplateRef::Id(NotificationTemplateId::new("tpl-test")),
            recipient: Recipient::Direct(
                ContactInfo::new()
                    .with_email(EmailAddress::new("recipient@example.test")),
            ),
            variables: BTreeMap::new(),
            attachments: Vec::new(),
            priority: Priority::Normal,
            scheduled_at: None,
            idempotency_key: None,
            correlation_id: None,
            school_id: SchoolId(Uuid::nil()),
        }
    }

    fn make_bulk(
        recipients: Vec<BulkRecipient>,
        channel: Channel,
    ) -> SendBulkNotification {
        SendBulkNotification {
            tenant: system_tenant(),
            template: TemplateRef::Id(NotificationTemplateId::new("tpl-bulk")),
            recipients,
            variables_per_recipient: false,
            channel,
            priority: Priority::Normal,
            scheduled_at: None,
            idempotency_key: None,
            correlation_id: None,
            school_id: SchoolId(Uuid::nil()),
        }
    }

    fn block_on<F: std::future::Future>(future: F) -> F::Output {
        futures::executor::block_on(future)
    }

    #[test]
    fn send_records_request_and_returns_receipt() {
        let provider = InMemoryNotificationProvider::new();
        let receipt =
            block_on(provider.send(make_send(Channel::InApp))).unwrap();

        assert_eq!(provider.send_count(), 1);
        assert_eq!(receipt.status, DeliveryStatus::Sent);
        assert_eq!(
            receipt.provider_message_id.as_deref(),
            Some(IN_MEMORY_PROVIDER_MSG_ID),
        );
        assert!(
            receipt.receipt_id.as_str().starts_with(IN_MEMORY_ID_PREFIX),
            "receipt id must be prefixed with {IN_MEMORY_ID_PREFIX:?}, got {:?}",
            receipt.receipt_id.as_str(),
        );
    }

    #[test]
    fn send_bulk_returns_receipt_for_each_recipient() {
        let provider = InMemoryNotificationProvider::new();
        let recipients = vec![
            BulkRecipient::new(Recipient::Direct(
                ContactInfo::new()
                    .with_email(EmailAddress::new("a@example.test")),
            )),
            BulkRecipient::new(Recipient::Direct(
                ContactInfo::new()
                    .with_email(EmailAddress::new("b@example.test")),
            )),
            BulkRecipient::new(Recipient::Direct(
                ContactInfo::new()
                    .with_email(EmailAddress::new("c@example.test")),
            )),
        ];
        let bulk = make_bulk(recipients, Channel::InApp);
        let receipt = block_on(provider.send_bulk(bulk)).unwrap();

        assert_eq!(receipt.receipts.len(), 3);
        assert!(receipt.failed.is_empty());
        assert_eq!(receipt.success_count(), 3);
        assert_eq!(receipt.failure_count(), 0);
        assert_eq!(provider.bulk_count(), 1);

        for r in &receipt.receipts {
            assert_eq!(r.status, DeliveryStatus::Sent);
            assert_eq!(
                r.provider_message_id.as_deref(),
                Some(IN_MEMORY_PROVIDER_MSG_ID),
            );
            assert!(r.receipt_id.as_str().starts_with(IN_MEMORY_ID_PREFIX));
        }
    }

    #[test]
    fn status_returns_sent_for_any_receipt() {
        let provider = InMemoryNotificationProvider::new();
        let id = NotificationReceiptId::new("in-memory-anything");
        let status = block_on(provider.status(id)).unwrap();
        assert_eq!(status, DeliveryStatus::Sent);
    }

    #[test]
    fn multiple_sends_are_recorded_in_order() {
        let provider = InMemoryNotificationProvider::new();
        let r1 = block_on(provider.send(make_send(Channel::InApp))).unwrap();
        let r2 = block_on(provider.send(make_send(Channel::InApp))).unwrap();
        let r3 = block_on(provider.send(make_send(Channel::InApp))).unwrap();

        assert_eq!(provider.send_count(), 3);
        assert_ne!(r1.receipt_id, r2.receipt_id);
        assert_ne!(r2.receipt_id, r3.receipt_id);
        assert_ne!(r1.receipt_id, r3.receipt_id);
    }

    #[test]
    fn send_with_email_channel_succeeds() {
        let provider = InMemoryNotificationProvider::new();
        let channel = Channel::Email {
            from: None,
            reply_to: None,
        };
        let receipt = block_on(provider.send(make_send(channel))).unwrap();

        assert!(matches!(receipt.channel, Channel::Email { .. }));
        assert_eq!(receipt.status, DeliveryStatus::Sent);
        assert_eq!(provider.send_count(), 1);
    }

    #[test]
    fn send_with_sms_channel_succeeds() {
        let provider = InMemoryNotificationProvider::new();
        let channel = Channel::Sms {
            from: None,
            unicode: false,
        };
        let receipt = block_on(provider.send(make_send(channel))).unwrap();

        assert!(matches!(receipt.channel, Channel::Sms { .. }));
        assert_eq!(receipt.status, DeliveryStatus::Sent);
        assert_eq!(provider.send_count(), 1);
    }
}
