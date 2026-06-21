//! High-level facade services. Each holds `&'a Engine` and
//! exposes 1-2 methods that delegate to the appropriate port.

use educore_core::tenant::TenantContext;

use crate::engine::Engine;
use crate::errors::SdkError;

/// Admission service — wraps academic student admission flows.
pub struct AdmissionService<'a> {
    engine: &'a Engine,
}

impl<'a> AdmissionService<'a> {
    /// Constructs a new admission service bound to `engine`.
    #[must_use]
    pub fn new(engine: &'a Engine) -> Self {
        Self { engine }
    }

    /// Admits a student. The full command flow lives in the
    /// academic domain crate; this facade is a thin
    /// re-export of the storage adapter for now.
    ///
    /// Returns the engine's `Arc<dyn StorageAdapter>` so the
    /// caller can run the academic-domain command surface
    /// against it.
    #[must_use]
    pub fn storage(&self) -> &std::sync::Arc<dyn educore_storage::StorageAdapter> {
        self.engine.storage()
    }
}

/// Attendance service — wraps bulk attendance marking.
pub struct AttendanceService<'a> {
    engine: &'a Engine,
}

impl<'a> AttendanceService<'a> {
    /// Constructs a new attendance service bound to `engine`.
    #[must_use]
    pub fn new(engine: &'a Engine) -> Self {
        Self { engine }
    }

    /// Marks bulk attendance. Delegates to
    /// `StorageAdapter::bulk_insert_student_attendances`.
    ///
    /// # Errors
    /// Returns `SdkError::Engine` if the storage adapter
    /// rejects the rows (tenant mismatch, duplicate, or
    /// infrastructure failure).
    pub async fn mark_bulk(
        &self,
        ctx: &TenantContext,
        rows: &[educore_storage::student_attendance_row::StudentAttendanceRow],
    ) -> Result<(), SdkError> {
        self.engine
            .storage()
            .bulk_insert_student_attendances(ctx, rows)
            .await
            .map_err(|e| SdkError::Facade {
                service: "AttendanceService",
                message: e.to_string(),
            })
    }
}

/// Payment service — wraps charge/refund flows.
pub struct PaymentService<'a> {
    engine: &'a Engine,
}

impl<'a> PaymentService<'a> {
    /// Constructs a new payment service bound to `engine`.
    #[must_use]
    pub fn new(engine: &'a Engine) -> Self {
        Self { engine }
    }

    /// Issues a charge. Delegates to `PaymentProvider::charge`.
    ///
    /// # Errors
    /// Returns `SdkError::Engine` if the payment provider
    /// rejects the request.
    pub async fn charge(
        &self,
        request: educore_payment::port::ChargeRequest,
    ) -> Result<educore_payment::port::PaymentReceipt, SdkError> {
        self.engine
            .payment()
            .charge(request)
            .await
            .map_err(|e| SdkError::Facade {
                service: "PaymentService",
                message: e.to_string(),
            })
    }
}

/// Notification service — wraps send flows.
pub struct NotificationService<'a> {
    engine: &'a Engine,
}

impl<'a> NotificationService<'a> {
    /// Constructs a new notification service bound to `engine`.
    #[must_use]
    pub fn new(engine: &'a Engine) -> Self {
        Self { engine }
    }

    /// Sends a single notification. Delegates to
    /// `NotificationProvider::send`.
    ///
    /// # Errors
    /// Returns `SdkError::Engine` if the provider rejects the
    /// request.
    pub async fn send(
        &self,
        request: educore_notify::port::SendNotification,
    ) -> Result<educore_notify::port::NotificationReceipt, SdkError> {
        self.engine
            .notify()
            .send(request)
            .await
            .map_err(|e| SdkError::Facade {
                service: "NotificationService",
                message: e.to_string(),
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
    use educore_core::ids::SchoolId;
    use educore_core::tenant::UserType;
    use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};
    use educore_storage::student_attendance_row::StudentAttendanceRow;
    use uuid::Uuid;

    fn ctx(school: SchoolId) -> TenantContext {
        let g = SystemIdGen;
        TenantContext::for_user(
            school,
            g.next_user_id(),
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        )
    }

    #[tokio::test]
    async fn attendance_service_marks_bulk_rows() {
        let engine = Engine::test_world();
        let school = SchoolId::from(Uuid::new_v4());
        let g = SystemIdGen;
        let row = StudentAttendanceRow {
            school_id: school,
            id: Uuid::new_v4(),
            student_id: Uuid::new_v4(),
            student_record_id: Uuid::new_v4(),
            class_id: Uuid::new_v4(),
            section_id: Uuid::new_v4(),
            attendance_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 21).unwrap(),
            attendance_type: "P".to_owned(),
            in_time: None,
            out_time: None,
            notes: None,
            is_absent: false,
            marked_by: g.next_user_id(),
            marked_at: Timestamp::now(),
            marked_from: "manual".to_owned(),
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            created_by: g.next_user_id(),
            updated_by: g.next_user_id(),
            active_status: ActiveStatus::Active,
            correlation_id: g.next_correlation_id(),
            last_event_id: Some(g.next_event_id()),
        };
        engine
            .attendance()
            .mark_bulk(&ctx(school), std::slice::from_ref(&row))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn payment_service_charges_cash() {
        let engine = Engine::test_world();
        let school = SchoolId::from(Uuid::new_v4());
        let g = SystemIdGen;
        let req = educore_payment::port::ChargeRequest::new(
            ctx(school),
            educore_payment::port::Money::new(
                educore_payment::port::CurrencyCode::new("USD").unwrap(),
                1500,
            )
            .unwrap(),
            educore_payment::port::PaymentMethod::Cash,
            educore_payment::port::CustomerRef::User(g.next_user_id()),
            g.next_idempotency_key(),
            g.next_correlation_id(),
        );
        let receipt = engine.payment_svc().charge(req).await.unwrap();
        assert!(!format!("{:?}", receipt.status).is_empty());
    }

    #[tokio::test]
    async fn notification_service_sends_email() {
        let engine = Engine::test_world();
        let school = SchoolId::from(Uuid::new_v4());
        let g = SystemIdGen;
        let req = educore_notify::port::SendNotification {
            tenant: ctx(school),
            channel: educore_notify::port::Channel::InApp,
            template: educore_notify::port::TemplateRef::id(
                educore_notify::errors::NotificationTemplateId::new("tpl-1"),
            ),
            recipient: educore_notify::port::Recipient::User(g.next_user_id()),
            variables: std::collections::BTreeMap::new(),
            attachments: vec![],
            priority: educore_notify::port::Priority::default(),
            scheduled_at: None,
            idempotency_key: None,
            correlation_id: None,
            school_id: school,
        };
        let receipt = engine.notify_svc().send(req).await.unwrap();
        assert!(receipt.receipt_id.as_str().starts_with("in-memory-"));
    }

    #[test]
    fn admission_service_exposes_storage() {
        let engine = Engine::test_world();
        let _storage = engine.admission().storage();
    }
}
