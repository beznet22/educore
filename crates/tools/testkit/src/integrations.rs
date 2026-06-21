//! # In-memory `IntegrationGateway`
//!
//! Test impl of
//! [`IntegrationGateway`](educore_integrations::port::IntegrationGateway)
//! for the testkit. Records every [`IntegrationRequest`] the engine
//! dispatches; returns a canned
//! [`IntegrationResponse::Success`](educore_integrations::port::IntegrationStatus::Success)
//! with `{"ok": true}` and a 1 ms duration. `list_capabilities()`
//! returns three canned entries (`lms`, `video`, `webhook_out`)
//! that match the engine's reference integrations.
//!
//! Consumer tests assert against [`InMemoryIntegrationGateway::invocations`]
//! to verify the engine emitted the right `IntegrationRequest`s.
//!
//! This crate is a member of the Educore workspace. See
//! `docs/ports/integrations.md` and
//! `docs/architecture.md` for behavioral details.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Mutex;

use async_trait::async_trait;
use chrono::Duration as ChronoDuration;
use educore_core::value_objects::Timestamp;
use educore_integrations::errors::{IntegrationError, Result as IntegrationResult};
use educore_integrations::port::{
    HealthStatus, IntegrationAction, IntegrationCapability, IntegrationGateway, IntegrationHealth,
    IntegrationId, IntegrationRequest, IntegrationResponse, IntegrationStatus,
};
use educore_rbac::value_objects::Capability;

/// The in-memory test integration gateway.
///
/// Holds a `Vec<IntegrationRequest>` of every invocation the engine
/// has dispatched since construction. The capability list is static
/// (`lms`, `video`, `webhook_out`) and is populated in
/// [`InMemoryIntegrationGateway::new`].
pub struct InMemoryIntegrationGateway {
    /// Recorded invocations, in the order they were dispatched.
    invocations: Mutex<Vec<IntegrationRequest>>,
    /// Static capability list (3 entries; see
    /// [`InMemoryIntegrationGateway::new`]).
    capabilities: Vec<IntegrationCapability>,
}

impl fmt::Debug for InMemoryIntegrationGateway {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let invocation_count = self
            .invocations
            .lock()
            .map(|g| g.len())
            .unwrap_or_default();
        f.debug_struct("InMemoryIntegrationGateway")
            .field("invocation_count", &invocation_count)
            .field("capability_count", &self.capabilities.len())
            .finish()
    }
}

impl Default for InMemoryIntegrationGateway {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryIntegrationGateway {
    /// Constructs a fresh gateway with an empty invocation log and
    /// three canned capabilities.
    ///
    /// The capabilities mirror the three reference integrations
    /// shipped with the engine:
    /// - `lms` / `roster_sync` — guarded by
    ///   [`Capability::LmsRosterSync`].
    /// - `video` / `schedule` — guarded by
    ///   [`Capability::VideoSchedule`].
    /// - `webhook_out` / `dispatch` — guarded by
    ///   [`Capability::WebhookOut`].
    #[must_use]
    pub fn new() -> Self {
        let capabilities = vec![
            IntegrationCapability {
                integration: IntegrationId::new("lms"),
                action: IntegrationAction::new("roster_sync"),
                description: "Sync student roster to the LMS".to_owned(),
                input_schema: None,
                output_schema: None,
                required_capabilities: vec![Capability::LmsRosterSync],
            },
            IntegrationCapability {
                integration: IntegrationId::new("video"),
                action: IntegrationAction::new("schedule"),
                description: "Schedule a video meeting".to_owned(),
                input_schema: None,
                output_schema: None,
                required_capabilities: vec![Capability::VideoSchedule],
            },
            IntegrationCapability {
                integration: IntegrationId::new("webhook_out"),
                action: IntegrationAction::new("dispatch"),
                description: "Dispatch a signed outbound webhook".to_owned(),
                input_schema: None,
                output_schema: None,
                required_capabilities: vec![Capability::WebhookOut],
            },
        ];
        Self {
            invocations: Mutex::new(Vec::new()),
            capabilities,
        }
    }

    /// Returns a snapshot of every invocation recorded so far, in
    /// the order they were dispatched. Used by tests to assert
    /// that the engine emitted the right `IntegrationRequest`s.
    #[must_use]
    pub fn invocations(&self) -> Vec<IntegrationRequest> {
        match self.invocations.lock() {
            Ok(g) => g.clone(),
            Err(_) => Vec::new(),
        }
    }

    /// Returns the number of invocations recorded so far. Cheaper
    /// than `invocations().len()` because it does not clone the
    /// underlying `Vec`.
    #[must_use]
    pub fn invocation_count(&self) -> usize {
        self.invocations
            .lock()
            .map(|g| g.len())
            .unwrap_or_default()
    }
}

#[async_trait]
impl IntegrationGateway for InMemoryIntegrationGateway {
    async fn invoke(
        &self,
        request: IntegrationRequest,
    ) -> IntegrationResult<IntegrationResponse> {
        match self.invocations.lock() {
            Ok(mut g) => g.push(request),
            Err(_) => {
                return Err(IntegrationError::Infrastructure(Box::new(
                    std::io::Error::other("InMemoryIntegrationGateway: invocations mutex poisoned"),
                )));
            }
        }
        Ok(IntegrationResponse {
            status: IntegrationStatus::Success,
            output: Some(serde_json::json!({"ok": true})),
            error: None,
            duration: ChronoDuration::milliseconds(1),
            cost: None,
            metadata: BTreeMap::new(),
        })
    }

    async fn list_capabilities(&self) -> IntegrationResult<Vec<IntegrationCapability>> {
        Ok(self.capabilities.clone())
    }

    async fn health(&self) -> IntegrationResult<IntegrationHealth> {
        Ok(IntegrationHealth {
            status: HealthStatus::Healthy,
            last_checked_at: Timestamp::now(),
            message: None,
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
    use educore_core::tenant::TenantContext;

    fn sample_request(id_gen: &SystemIdGen, integration: &str) -> IntegrationRequest {
        IntegrationRequest {
            tenant: TenantContext::system(id_gen.next_school_id(), id_gen.next_correlation_id()),
            integration: IntegrationId::new(integration),
            action: IntegrationAction::new("test"),
            input: serde_json::json!({}),
            idempotency_key: id_gen.next_idempotency_key(),
            correlation_id: id_gen.next_correlation_id(),
            timeout: None,
        }
    }

    #[tokio::test]
    async fn invoke_records_request_and_returns_success() {
        let gw = InMemoryIntegrationGateway::new();
        let id_gen = SystemIdGen;
        let req = sample_request(&id_gen, "lms");

        let resp = gw.invoke(req).await.expect("invoke should succeed");

        assert_eq!(resp.status, IntegrationStatus::Success);
        assert!(resp.output.is_some(), "success must carry an output");
        assert!(resp.error.is_none(), "success must not carry an error");
        assert_eq!(
            resp.output.expect("checked above"),
            serde_json::json!({"ok": true})
        );
        assert_eq!(resp.duration, ChronoDuration::milliseconds(1));
        assert!(resp.cost.is_none());
        assert!(resp.metadata.is_empty());

        assert_eq!(gw.invocations().len(), 1);
        assert_eq!(gw.invocation_count(), 1);
    }

    #[tokio::test]
    async fn list_capabilities_returns_3_canned_entries() {
        let gw = InMemoryIntegrationGateway::new();
        let caps = gw.list_capabilities().await.expect("list must succeed");

        assert_eq!(caps.len(), 3);
        let ids: Vec<&str> = caps.iter().map(|c| c.integration.as_str()).collect();
        assert!(ids.contains(&"lms"));
        assert!(ids.contains(&"video"));
        assert!(ids.contains(&"webhook_out"));

        // Each canned capability must carry the matching RBAC cap.
        for cap in &caps {
            match cap.integration.as_str() {
                "lms" => assert_eq!(cap.required_capabilities, vec![Capability::LmsRosterSync]),
                "video" => assert_eq!(cap.required_capabilities, vec![Capability::VideoSchedule]),
                "webhook_out" => {
                    assert_eq!(cap.required_capabilities, vec![Capability::WebhookOut])
                }
                other => panic!("unexpected integration id: {other}"),
            }
        }
    }

    #[tokio::test]
    async fn health_returns_healthy() {
        let gw = InMemoryIntegrationGateway::new();
        let h = gw.health().await.expect("health must succeed");

        assert_eq!(h.status, HealthStatus::Healthy);
        assert!(h.message.is_none(), "healthy gateway carries no message");
        // `last_checked_at` should be recent; the only invariant is
        // "not epoch" so we can sanity-check the constructor wired
        // a real wall-clock value.
        assert_ne!(h.last_checked_at, Timestamp::epoch());
    }

    #[tokio::test]
    async fn multiple_invocations_recorded_in_order() {
        let gw = InMemoryIntegrationGateway::new();
        let id_gen = SystemIdGen;

        for i in 0..5 {
            let req = sample_request(&id_gen, &format!("int_{i}"));
            gw.invoke(req).await.expect("invoke must succeed");
        }

        let recorded = gw.invocations();
        assert_eq!(recorded.len(), 5);
        for (i, r) in recorded.iter().enumerate() {
            assert_eq!(r.integration.as_str(), format!("int_{i}"));
        }
    }

    #[tokio::test]
    async fn invoke_with_specific_integration_id_works() {
        let gw = InMemoryIntegrationGateway::new();
        let id_gen = SystemIdGen;
        let req = sample_request(&id_gen, "custom_lms_provider");

        let resp = gw.invoke(req).await.expect("invoke must succeed");
        assert_eq!(resp.status, IntegrationStatus::Success);

        let recorded = gw.invocations();
        assert_eq!(recorded.len(), 1);
        assert_eq!(recorded[0].integration.as_str(), "custom_lms_provider");
        assert_eq!(recorded[0].action.as_str(), "test");
    }

    #[test]
    fn default_constructs_empty_invocation_log_with_3_capabilities() {
        let gw = InMemoryIntegrationGateway::default();
        assert_eq!(gw.invocation_count(), 0);
        assert_eq!(gw.invocations().len(), 0);
        // The static capability list is exposed through
        // `list_capabilities()` in async contexts, but the field
        // itself is sized and the constructor is `pub` — we
        // exercise the synchronous accessors here.
        assert_eq!(gw.capabilities.len(), 3);
    }

    #[test]
    fn debug_render_does_not_panic() {
        let gw = InMemoryIntegrationGateway::new();
        let dbg = format!("{gw:?}");
        assert!(dbg.contains("InMemoryIntegrationGateway"));
        assert!(dbg.contains("invocation_count"));
        assert!(dbg.contains("capability_count"));
    }
}
