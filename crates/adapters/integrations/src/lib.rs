//! # educore-integrations
//!
//! Integration port, LMS, video conferencing, custom webhooks,
//! polling adapters.
//!
//! This crate is a member of the Educore workspace. See
//! `docs/architecture.md` and the port spec in
//! [`docs/ports/integrations.md`](https://example.com/integrations.md)
//! for behavioral details.
//!
//! ## Module layout
//!
//! - [`port`] — the [`IntegrationGateway`](port::IntegrationGateway)
//!   trait plus every request/response/value type it touches
//!   (`IntegrationRequest`, `IntegrationResponse`,
//!   `IntegrationStatus`, `IntegrationCapability`,
//!   `IntegrationHealth`, `RetryPolicy`, `SchemaRef`,
//!   `IntegrationId`, `IntegrationAction`, `IntegrationCost`).
//! - [`errors`] — the [`IntegrationError`](errors::IntegrationError)
//!   universal failure type.
//!
//! ## Trait surface
//!
//! ```ignore
//! #[async_trait]
//! pub trait IntegrationGateway: Send + Sync + std::fmt::Debug {
//!     async fn invoke(&self, request: IntegrationRequest)
//!         -> Result<IntegrationResponse>;
//!     async fn list_capabilities(&self) -> Result<Vec<IntegrationCapability>>;
//!     async fn health(&self) -> Result<IntegrationHealth>;
//! }
//! ```
//!
//! The trait is **object-safe** — adapters are typically held as
//! `Arc<dyn IntegrationGateway>` so consumers can swap
//! implementations without recompiling.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// The integration port: [`IntegrationGateway`](port::IntegrationGateway)
/// plus every request, response, capability, health, and retry type
/// it touches.
pub mod port;

/// The universal [`IntegrationError`](errors::IntegrationError)
/// failure type. Every [`IntegrationGateway`](port::IntegrationGateway)
/// method returns `Result<_, IntegrationError>`.
pub mod errors;

/// Reference implementation of the "Custom Webhook (Out)"
/// integration: signs each dispatched event with HMAC-SHA256 and
/// posts it to one or more configured webhook URLs.
///
/// See [`webhook_out::WebhookOutIntegration`] and the port spec in
/// `docs/ports/integrations.md` § "Custom Webhook (Out)".
pub mod webhook_out;

/// Reference integration adapter for video conferencing providers
/// (Zoom, Google Meet, Microsoft Teams).
///
/// Creates a meeting when a `LessonPlan` is scheduled, lists cloud
/// recordings so the engine can emit `VideoRecordingAvailable`, and
/// supports fetching meeting metadata by id. Construct via
/// [`video::VideoConferencingIntegrationBuilder`] and hold behind
/// `Arc<dyn IntegrationGateway>` to swap providers at runtime.
///
/// See `docs/ports/integrations.md` § "Video Conferencing".
pub mod video;

/// LMS (Learning Management System) reference
/// [`IntegrationGateway`](port::IntegrationGateway) implementation.
///
/// Drives a generic LMS over its REST API (default base URL targets
/// Google Classroom `https://classroom.googleapis.com/v1`). Exposes
/// three closed-set actions: course creation, roster sync
/// (driven by the engine's `StudentAdmitted`,
/// `StudentAssignedToSection`, and `StudentWithdrawn` events), and
/// assignment-submission pull (the engine translates each pulled
/// submission into an `OnlineExamSubmitted` event with a
/// `Source::Lms` tag).
///
/// Construct via [`lms::LmsIntegrationBuilder`] and hold behind
/// `Arc<dyn IntegrationGateway>` to swap providers at runtime.
///
/// See `docs/ports/integrations.md` § "LMS Sync".
pub mod lms;

/// Pure helper services reused by the integration adapters:
/// HMAC-SHA256 webhook signing ([`services::WebhookSignatureService`]),
/// cursor + due-date bookkeeping for inbound polling
/// ([`services::PollingService`]), backoff + permanent-failure
/// classification for outbound retries
/// ([`services::RetryService`]), and per-integration token
/// buckets ([`services::RateLimitService`]).
///
/// None of the helpers perform I/O — adapters wrap them with the
/// async transport layer.
pub mod services;

/// Per-tenant OAuth2 client-credentials token cache + refresh helper
/// (see `docs/ports/integrations.md` § OAuth2 Client Credentials).
pub mod oauth2;

/// Re-exports of the engine types and the port's request/response
/// surface. Consumers typically
/// `use educore_integrations::prelude::*;` once at the top of a
/// file.
pub mod prelude {
    pub use crate::errors::IntegrationError;
    pub use crate::lms::{
        LmsIntegration, LmsIntegrationBuilder, ACTION_COURSE_CREATE, ACTION_ROSTER_SYNC,
        ACTION_SUBMISSIONS_PULL, LMS_INTEGRATION_ID,
    };
    pub use crate::port::{
        HealthStatus, IntegrationAction, IntegrationCapability, IntegrationCost,
        IntegrationGateway, IntegrationHealth, IntegrationId, IntegrationRequest,
        IntegrationResponse, IntegrationStatus, RetryPolicy, SchemaFormat, SchemaRef,
    };
    pub use crate::services::{
        PollingService, RateLimitService, RateState, RetryService, Schedule,
        WebhookSignatureService,
    };
    pub use crate::video::{
        VideoConferencingIntegration, VideoConferencingIntegrationBuilder, ACTION_MEETING_CREATE,
        ACTION_MEETING_GET, ACTION_RECORDING_LIST, VIDEO_INTEGRATION_ID,
    };
    pub use crate::webhook_out::{
        WebhookOutIntegration, WebhookOutIntegrationBuilder, WebhookTarget, SIGNATURE_HEADER,
    };
}

/// Package name constant. Re-exported so consumers can assert they
/// are using the right crate version at compile time.
pub const PACKAGE_NAME: &str = "educore-integrations";

/// Package version at compile time.
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use async_trait::async_trait;

    use crate::errors::Result;
    use crate::port::{
        HealthStatus, IntegrationAction, IntegrationCapability, IntegrationGateway,
        IntegrationHealth, IntegrationId, IntegrationRequest, IntegrationResponse,
        IntegrationStatus,
    };

    #[test]
    fn package_metadata_is_set() {
        assert_eq!(PACKAGE_NAME, "educore-integrations");
        assert!(!PACKAGE_VERSION.is_empty());
    }

    #[test]
    fn prelude_round_trip() {
        let _id: IntegrationId = "twilio".into();
        let _action: IntegrationAction = "send_sms".into();
        let _: IntegrationStatus = IntegrationStatus::Success;
        let _: HealthStatus = HealthStatus::Healthy;
    }

    /// Object-safety smoke test: a minimal `IntegrationGateway`
    /// impl can be erased to `Arc<dyn IntegrationGateway>`. The
    /// trait is documented as object-safe in
    /// `docs/ports/integrations.md` § "Object Safety"; this test
    /// fails the build if the trait ever drifts away from that
    /// contract (e.g. by adding a generic associated method).
    #[test]
    fn integration_gateway_is_object_safe() {
        #[derive(Debug)]
        struct NoopGateway;

        #[async_trait]
        impl IntegrationGateway for NoopGateway {
            async fn invoke(&self, _request: IntegrationRequest) -> Result<IntegrationResponse> {
                Ok(IntegrationResponse {
                    status: IntegrationStatus::Success,
                    output: None,
                    error: None,
                    duration: chrono::Duration::zero(),
                    cost: None,
                    metadata: std::collections::BTreeMap::new(),
                })
            }

            async fn list_capabilities(&self) -> Result<Vec<IntegrationCapability>> {
                Ok(Vec::new())
            }

            async fn health(&self) -> Result<IntegrationHealth> {
                Ok(IntegrationHealth {
                    status: HealthStatus::Healthy,
                    last_checked_at: educore_core::value_objects::Timestamp::epoch(),
                    message: None,
                })
            }
        }

        let _: Arc<dyn IntegrationGateway> = Arc::new(NoopGateway);
    }
}
