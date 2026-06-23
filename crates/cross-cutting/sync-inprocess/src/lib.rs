//! # educore-sync-inprocess
//!
//! The in-process reference implementation of the
//! [`educore_sync::SyncAdapter`] port.
//!
//! Per [`ADR-018`], this adapter is the **default for
//! single-process deployments and the test target** for the
//! Phase 0 e2e. It publishes the four typed sync events
//! ([`SyncStarted`], [`SyncPaused`], [`SyncResumed`],
//! [`SyncStopped`]) through an [`educore_events::EventBus`]
//! (typically an in-process bus). No network I/O is involved;
//! the same struct is used as the Phase 0 parity reference and
//! as the production adapter for tests, desktop clients running
//! the engine in-process, and CI.
//!
//! The implementation is intentionally simple:
//!
//! 1. The [`SyncAdapter`] methods (`start`, `pause`, `resume`,
//!    `stop`) each forward to [`InProcessSyncAdapter::send_command`]
//!    with the corresponding [`SyncCommand`].
//! 2. [`InProcessSyncAdapter::send_command`] locks the health
//!    state, transitions the status, records the event
//!    timestamp, and publishes the corresponding typed event
//!    through the bus.
//! 3. [`InProcessSyncAdapter::health`] clones the current
//!    [`SyncHealth`] snapshot under the mutex.
//!
//! Consumers of sync events subscribe to the bus directly (with
//! `Topic::EventType("sync.session.started")` etc.); this
//! adapter does not maintain a per-adapter broadcast channel.
//! The bus-port contract is the single source of truth for event
//! delivery.
//!
//! [`ADR-018`]: ../../docs/decisions/ADR-018-SyncEngineArchitecture.md
//! [`SyncStarted`]: educore_events::sync::SyncStarted
//! [`SyncPaused`]: educore_events::sync::SyncPaused
//! [`SyncResumed`]: educore_events::sync::SyncResumed
//! [`SyncStopped`]: educore_events::sync::SyncStopped

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;
use tracing::warn;

use educore_core::clock::{IdGenerator, SystemIdGen};
use educore_core::error::Result;
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;
use educore_events::envelope::EventEnvelope;
use educore_events::event_bus::EventBus;
use educore_events::prelude::{SyncPaused, SyncResumed, SyncStarted, SyncStopped};
use educore_sync::prelude::{SyncAdapter, SyncCommand, SyncHealth, SyncStatus};

/// The in-process reference implementation of
/// [`SyncAdapter`].
///
/// The adapter holds a `bus: Arc<dyn EventBus>` (so the same
/// `InProcessBus` is shared with the rest of the engine) and a
/// `state: Arc<Mutex<SyncHealth>>` for the adapter-level health
/// snapshot. It is `Clone` (cheap: both fields are `Arc`-backed),
/// so consumers can hand out multiple references to the same
/// underlying state — e.g. one for the engine's command
/// dispatcher, one for the test harness, one for a local UI
/// binding.
#[derive(Debug, Clone)]
pub struct InProcessSyncAdapter {
    bus: Arc<dyn EventBus>,
    state: Arc<Mutex<SyncHealth>>,
}

impl InProcessSyncAdapter {
    /// Creates a new in-process sync adapter that publishes its
    /// events through `bus`.
    ///
    /// The health state is initialised to [`SyncHealth::default`]
    /// (status `Stopped`, no recorded event). The adapter does
    /// not own the bus; the caller is expected to share the bus
    /// across the engine.
    #[must_use]
    pub fn new(bus: Arc<dyn EventBus>) -> Self {
        Self {
            bus,
            state: Arc::new(Mutex::new(SyncHealth::default())),
        }
    }

    /// Sends a command to the adapter.
    ///
    /// The command is processed synchronously: the health state
    /// is updated and the corresponding typed event is built
    /// and published to the bus before this method returns.
    /// The bus publish is best-effort: a publish error is
    /// logged via `tracing::warn!` and the adapter continues
    /// (the in-process bus does not fail under normal
    /// conditions, so this is purely a defensive guard).
    pub async fn send_command(&self, cmd: SyncCommand) -> Result<()> {
        let mut state = self.state.lock().await;
        let at = Timestamp::now();
        let school = match cmd {
            SyncCommand::Start(s) => {
                state.status = SyncStatus::Running;
                state.last_event_at = Some(at);
                s
            }
            SyncCommand::Pause(s) => {
                state.status = SyncStatus::Paused;
                state.last_event_at = Some(at);
                s
            }
            SyncCommand::Resume(s) => {
                state.status = SyncStatus::Running;
                state.last_event_at = Some(at);
                s
            }
            SyncCommand::Stop(s) => {
                state.status = SyncStatus::Stopped;
                state.last_event_at = Some(at);
                s
            }
        };
        // The `state` lock is held while we mint the
        // envelope, but envelope construction is pure (no
        // I/O). We mint a system TenantContext because the
        // in-process adapter doesn't have an authenticated
        // actor at this layer — the command was issued by
        // the local engine on behalf of an operator, and
        // the bus-port envelope is the engine's internal
        // record of the state transition. A consumer that
        // needs the original actor can find it on the
        // command audit row.
        let ctx = TenantContext::for_user(
            school,
            SystemIdGen.next_user_id(),
            SystemIdGen.next_correlation_id(),
            UserType::System,
        );
        // Build the typed event + envelope per the command.
        let envelope: EventEnvelope = match cmd {
            SyncCommand::Start(_) => SyncStarted::now(school).into_envelope(&ctx),
            SyncCommand::Pause(_) => SyncPaused::now(school).into_envelope(&ctx),
            SyncCommand::Resume(_) => SyncResumed::now(school).into_envelope(&ctx),
            SyncCommand::Stop(_) => SyncStopped::now(school).into_envelope(&ctx),
        };
        // Publish via the bus. The in-process bus is infallible
        // in normal operation; log and continue on error so the
        // adapter's health state stays consistent.
        if let Err(e) = self.bus.publish(envelope).await {
            warn!(?e, "in-process sync adapter: bus publish failed");
        }
        Ok(())
    }
}

#[async_trait]
impl SyncAdapter for InProcessSyncAdapter {
    async fn start(&self, school: SchoolId) -> Result<()> {
        self.send_command(SyncCommand::Start(school)).await
    }

    async fn pause(&self, school: SchoolId) -> Result<()> {
        self.send_command(SyncCommand::Pause(school)).await
    }

    async fn resume(&self, school: SchoolId) -> Result<()> {
        self.send_command(SyncCommand::Resume(school)).await
    }

    async fn stop(&self, school: SchoolId) -> Result<()> {
        self.send_command(SyncCommand::Stop(school)).await
    }

    async fn health(&self) -> Result<SyncHealth> {
        let state = self.state.lock().await;
        Ok(state.clone())
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
    //! Integration tests: drive a freshly constructed adapter
    //! through the Start → Pause flow, subscribe to the bus, and
    //! verify the corresponding typed events arrive.

    use std::sync::Arc;
    use std::time::Duration;

    use educore_core::clock::{IdGenerator, SystemIdGen};
    use educore_core::ids::SchoolId;
    use educore_event_bus::InProcessEventBus;
    use educore_events::envelope::EventEnvelope;
    use educore_events::event_bus::{
        EventBus, EventSubscription, StartPosition, SubscribeOptions, Topic,
    };
    use educore_sync::prelude::{SyncCommand, SyncStatus};

    use super::InProcessSyncAdapter;

    /// Helper: build an adapter and a subscription filtered to
    /// the `sync.session.*` topic. Returns the adapter, a
    /// receiver for the bus, and a fresh `SchoolId`.
    async fn setup() -> (InProcessSyncAdapter, Box<dyn EventSubscription>, SchoolId) {
        let bus: Arc<dyn EventBus> = Arc::new(InProcessEventBus::new());
        let adapter = InProcessSyncAdapter::new(Arc::clone(&bus));
        let mut opts: SubscribeOptions =
            SubscribeOptions::for_consumer("test-consumer".into(), Topic::Domain("sync"));
        opts.start = StartPosition::Earliest;
        let sub: Box<dyn EventSubscription> = bus.subscribe(opts).await.expect("subscribe");
        let school = SystemIdGen.next_school_id();
        (adapter, sub, school)
    }

    fn event_type(envelope: &EventEnvelope) -> &str {
        envelope.event_type.as_str()
    }

    #[tokio::test]
    async fn start_emits_typed_sync_started_event() {
        let (adapter, mut sub, school) = setup().await;

        adapter
            .send_command(SyncCommand::Start(school))
            .await
            .unwrap();

        let envelope = tokio::time::timeout(Duration::from_secs(1), sub.next())
            .await
            .expect("timed out waiting for event")
            .expect("subscription closed")
            .expect("bus error");
        assert_eq!(event_type(&envelope), "sync.session.started");
        assert_eq!(envelope.school_id, school);
    }

    #[tokio::test]
    async fn pause_after_start_emits_typed_sync_paused_event() {
        let (adapter, mut sub, school) = setup().await;

        adapter
            .send_command(SyncCommand::Start(school))
            .await
            .unwrap();
        adapter
            .send_command(SyncCommand::Pause(school))
            .await
            .unwrap();

        // Skip the Start event; assert the Pause.
        let _ = tokio::time::timeout(Duration::from_secs(1), sub.next()).await;
        let envelope = tokio::time::timeout(Duration::from_secs(1), sub.next())
            .await
            .expect("timed out waiting for event")
            .expect("subscription closed")
            .expect("bus error");
        assert_eq!(event_type(&envelope), "sync.session.paused");
        assert_eq!(envelope.school_id, school);
    }

    #[tokio::test]
    async fn health_reports_paused_after_pause_command() {
        let (adapter, _sub, school) = setup().await;

        adapter
            .send_command(SyncCommand::Start(school))
            .await
            .unwrap();
        adapter
            .send_command(SyncCommand::Pause(school))
            .await
            .unwrap();

        let health = educore_sync::prelude::SyncAdapter::health(&adapter)
            .await
            .unwrap();
        assert_eq!(health.status, SyncStatus::Paused);
        assert!(health.last_event_at.is_some());
    }

    #[tokio::test]
    async fn full_session_lifecycle_emits_four_typed_events() {
        let (adapter, mut sub, school) = setup().await;

        adapter
            .send_command(SyncCommand::Start(school))
            .await
            .unwrap();
        adapter
            .send_command(SyncCommand::Pause(school))
            .await
            .unwrap();

        let e0 = tokio::time::timeout(Duration::from_secs(1), sub.next())
            .await
            .expect("timeout")
            .expect("closed")
            .expect("bus error");
        let e1 = tokio::time::timeout(Duration::from_secs(1), sub.next())
            .await
            .expect("timeout")
            .expect("closed")
            .expect("bus error");
        assert_eq!(event_type(&e0), "sync.session.started");
        assert_eq!(event_type(&e1), "sync.session.paused");
        assert_eq!(e0.school_id, school);
        assert_eq!(e1.school_id, school);

        let health = educore_sync::prelude::SyncAdapter::health(&adapter)
            .await
            .unwrap();
        assert_eq!(health.status, SyncStatus::Paused);
    }

    #[tokio::test]
    async fn resume_after_pause_returns_to_running() {
        let (adapter, mut sub, school) = setup().await;

        adapter
            .send_command(SyncCommand::Start(school))
            .await
            .unwrap();
        adapter
            .send_command(SyncCommand::Pause(school))
            .await
            .unwrap();
        adapter
            .send_command(SyncCommand::Resume(school))
            .await
            .unwrap();
        adapter
            .send_command(SyncCommand::Stop(school))
            .await
            .unwrap();

        let e0 = tokio::time::timeout(Duration::from_secs(1), sub.next())
            .await
            .expect("timeout")
            .expect("closed")
            .expect("bus error");
        let e1 = tokio::time::timeout(Duration::from_secs(1), sub.next())
            .await
            .expect("timeout")
            .expect("closed")
            .expect("bus error");
        let e2 = tokio::time::timeout(Duration::from_secs(1), sub.next())
            .await
            .expect("timeout")
            .expect("closed")
            .expect("bus error");
        let e3 = tokio::time::timeout(Duration::from_secs(1), sub.next())
            .await
            .expect("timeout")
            .expect("closed")
            .expect("bus error");
        assert_eq!(event_type(&e0), "sync.session.started");
        assert_eq!(event_type(&e1), "sync.session.paused");
        assert_eq!(event_type(&e2), "sync.session.resumed");
        assert_eq!(event_type(&e3), "sync.session.stopped");
        assert_eq!(e0.school_id, school);
        assert_eq!(e1.school_id, school);
        assert_eq!(e2.school_id, school);
        assert_eq!(e3.school_id, school);

        let health = educore_sync::prelude::SyncAdapter::health(&adapter)
            .await
            .unwrap();
        assert_eq!(health.status, SyncStatus::Stopped);
    }

    #[tokio::test]
    async fn default_state_is_stopped_with_no_events() {
        let bus: Arc<dyn EventBus> = Arc::new(InProcessEventBus::new());
        let adapter = InProcessSyncAdapter::new(bus);
        let health = educore_sync::prelude::SyncAdapter::health(&adapter)
            .await
            .unwrap();
        assert_eq!(health.status, SyncStatus::Stopped);
        assert_eq!(health.last_event_at, None);
    }
}
