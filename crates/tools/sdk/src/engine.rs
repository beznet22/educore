//! `Engine` and `EngineBuilder` — the consumer-facing wiring surface.

use std::sync::Arc;

use educore_auth::port::AuthProvider;
use educore_core::clock::{Clock, IdGenerator, SystemClock, SystemIdGen};
use educore_events::event_bus::EventBus;
use educore_files::port::FileStorage;
use educore_integrations::port::IntegrationGateway;
use educore_notify::port::NotificationProvider;
use educore_payment::port::PaymentProvider;
use educore_storage::StorageAdapter;
use educore_testkit::TestkitWorld;

use crate::errors::SdkError;
use crate::facade::{AdmissionService, AttendanceService, NotificationService, PaymentService};

/// The engine. All 6 ports are `Arc<dyn ...>` so the engine
/// can be cheaply cloned and shared across threads.
#[derive(Clone)]
pub struct Engine {
    /// The storage adapter.
    storage: Arc<dyn StorageAdapter>,
    /// The auth provider.
    auth: Arc<dyn AuthProvider>,
    /// The notification provider.
    notify: Arc<dyn NotificationProvider>,
    /// The payment provider.
    payment: Arc<dyn PaymentProvider>,
    /// The file storage.
    files: Arc<dyn FileStorage>,
    /// The integration gateway.
    integrations: Arc<dyn IntegrationGateway>,
    /// The event bus.
    bus: Arc<dyn EventBus>,
    /// The clock.
    clock: Arc<dyn Clock>,
    /// The id generator.
    id_gen: Arc<dyn IdGenerator>,
}

impl std::fmt::Debug for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Engine").finish_non_exhaustive()
    }
}

impl Engine {
    /// Constructs a fresh `Engine` with all 7 ports wired to the
    /// in-memory testkit impls and the default `InProcessEventBus`.
    /// Convenience for consumer tests and dogfooding.
    #[must_use]
    pub fn test_world() -> Self {
        let world = TestkitWorld::new();
        let bus: Arc<dyn EventBus> = world.bus.clone();
        Self {
            storage: world.storage.clone(),
            auth: world.auth.clone(),
            notify: world.notify.clone(),
            payment: world.payment.clone(),
            files: world.files.clone(),
            integrations: world.integrations.clone(),
            bus,
            clock: Arc::new(SystemClock),
            id_gen: Arc::new(SystemIdGen),
        }
    }

    /// Returns a reference to the storage adapter.
    #[must_use]
    pub fn storage(&self) -> &Arc<dyn StorageAdapter> {
        &self.storage
    }

    /// Returns a reference to the auth provider.
    #[must_use]
    pub fn auth(&self) -> &Arc<dyn AuthProvider> {
        &self.auth
    }

    /// Returns a reference to the notification provider.
    #[must_use]
    pub fn notify(&self) -> &Arc<dyn NotificationProvider> {
        &self.notify
    }

    /// Returns a reference to the payment provider.
    #[must_use]
    pub fn payment(&self) -> &Arc<dyn PaymentProvider> {
        &self.payment
    }

    /// Returns a reference to the file storage.
    #[must_use]
    pub fn files(&self) -> &Arc<dyn FileStorage> {
        &self.files
    }

    /// Returns a reference to the integration gateway.
    #[must_use]
    pub fn integrations(&self) -> &Arc<dyn IntegrationGateway> {
        &self.integrations
    }

    /// Returns a reference to the event bus.
    #[must_use]
    pub fn bus(&self) -> &Arc<dyn EventBus> {
        &self.bus
    }

    /// Returns a reference to the clock.
    #[must_use]
    pub fn clock(&self) -> &Arc<dyn Clock> {
        &self.clock
    }

    /// Returns a reference to the id generator.
    #[must_use]
    pub fn id_gen(&self) -> &Arc<dyn IdGenerator> {
        &self.id_gen
    }

    /// Returns a handle to the admission facade.
    #[must_use]
    pub fn admission(&self) -> AdmissionService<'_> {
        AdmissionService::new(self)
    }

    /// Returns a handle to the attendance facade.
    #[must_use]
    pub fn attendance(&self) -> AttendanceService<'_> {
        AttendanceService::new(self)
    }

    /// Returns a handle to the payment facade.
    #[must_use]
    pub fn payment_svc(&self) -> PaymentService<'_> {
        PaymentService::new(self)
    }

    /// Returns a handle to the notification facade.
    #[must_use]
    pub fn notify_svc(&self) -> NotificationService<'_> {
        NotificationService::new(self)
    }
}

/// The engine builder. All 6 ports + clock + id_gen are
/// required; `build()` returns `Err(SdkError::MissingPort)` if
/// any required port is not provided.
pub struct EngineBuilder {
    storage: Option<Arc<dyn StorageAdapter>>,
    auth: Option<Arc<dyn AuthProvider>>,
    notify: Option<Arc<dyn NotificationProvider>>,
    payment: Option<Arc<dyn PaymentProvider>>,
    files: Option<Arc<dyn FileStorage>>,
    integrations: Option<Arc<dyn IntegrationGateway>>,
    bus: Option<Arc<dyn EventBus>>,
    clock: Option<Arc<dyn Clock>>,
    id_gen: Option<Arc<dyn IdGenerator>>,
}

impl std::fmt::Debug for EngineBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EngineBuilder").finish_non_exhaustive()
    }
}

impl Default for EngineBuilder {
    #[allow(clippy::derivable_impls)]
    fn default() -> Self {
        Self::new()
    }
}

impl EngineBuilder {
    /// Constructs a fresh `EngineBuilder` with no ports set.
    #[must_use]
    pub fn new() -> Self {
        Self {
            storage: None,
            auth: None,
            notify: None,
            payment: None,
            files: None,
            integrations: None,
            bus: None,
            clock: None,
            id_gen: None,
        }
    }

    /// Sets the storage adapter.
    #[must_use]
    pub fn storage(mut self, storage: Arc<dyn StorageAdapter>) -> Self {
        self.storage = Some(storage);
        self
    }

    /// Sets the auth provider.
    #[must_use]
    pub fn auth(mut self, auth: Arc<dyn AuthProvider>) -> Self {
        self.auth = Some(auth);
        self
    }

    /// Sets the notification provider.
    #[must_use]
    pub fn notify(mut self, notify: Arc<dyn NotificationProvider>) -> Self {
        self.notify = Some(notify);
        self
    }

    /// Sets the payment provider.
    #[must_use]
    pub fn payment(mut self, payment: Arc<dyn PaymentProvider>) -> Self {
        self.payment = Some(payment);
        self
    }

    /// Sets the file storage.
    #[must_use]
    pub fn files(mut self, files: Arc<dyn FileStorage>) -> Self {
        self.files = Some(files);
        self
    }

    /// Sets the integration gateway.
    #[must_use]
    pub fn integrations(mut self, integrations: Arc<dyn IntegrationGateway>) -> Self {
        self.integrations = Some(integrations);
        self
    }

    /// Sets the event bus.
    #[must_use]
    pub fn event_bus(mut self, bus: Arc<dyn EventBus>) -> Self {
        self.bus = Some(bus);
        self
    }

    /// Sets the clock.
    #[must_use]
    pub fn clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = Some(clock);
        self
    }

    /// Sets the id generator.
    #[must_use]
    pub fn id_gen(mut self, id_gen: Arc<dyn IdGenerator>) -> Self {
        self.id_gen = Some(id_gen);
        self
    }

    /// Builds the `Engine`. Returns `Err(SdkError::MissingPort)`
    /// if any required port is not set.
    pub fn build(self) -> Result<Engine, SdkError> {
        let storage = self.storage.ok_or(SdkError::MissingPort("storage"))?;
        let auth = self.auth.ok_or(SdkError::MissingPort("auth"))?;
        let notify = self.notify.ok_or(SdkError::MissingPort("notify"))?;
        let payment = self.payment.ok_or(SdkError::MissingPort("payment"))?;
        let files = self.files.ok_or(SdkError::MissingPort("files"))?;
        let integrations = self
            .integrations
            .ok_or(SdkError::MissingPort("integrations"))?;
        let bus = self.bus.ok_or(SdkError::MissingPort("event_bus"))?;
        let clock = self.clock.ok_or(SdkError::MissingPort("clock"))?;
        let id_gen = self.id_gen.ok_or(SdkError::MissingPort("id_gen"))?;
        Ok(Engine {
            storage,
            auth,
            notify,
            payment,
            files,
            integrations,
            bus,
            clock,
            id_gen,
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

    #[test]
    fn engine_test_world_constructs() {
        let _engine = Engine::test_world();
    }

    #[test]
    fn engine_builder_with_all_ports_succeeds() {
        let world = TestkitWorld::new();
        let bus: Arc<dyn EventBus> = world.bus.clone();
        let engine = EngineBuilder::new()
            .storage(world.storage.clone())
            .auth(world.auth.clone())
            .notify(world.notify.clone())
            .payment(world.payment.clone())
            .files(world.files.clone())
            .integrations(world.integrations.clone())
            .event_bus(bus)
            .clock(Arc::new(SystemClock))
            .id_gen(Arc::new(SystemIdGen))
            .build()
            .unwrap();
        let _: &Arc<dyn StorageAdapter> = engine.storage();
    }

    #[test]
    fn engine_builder_missing_storage_returns_error() {
        let world = TestkitWorld::new();
        let bus: Arc<dyn EventBus> = world.bus.clone();
        let err = EngineBuilder::new()
            .auth(world.auth.clone())
            .notify(world.notify.clone())
            .payment(world.payment.clone())
            .files(world.files.clone())
            .integrations(world.integrations.clone())
            .event_bus(bus)
            .clock(Arc::new(SystemClock))
            .id_gen(Arc::new(SystemIdGen))
            .build();
        assert!(matches!(err, Err(SdkError::MissingPort("storage"))));
    }

    #[test]
    fn engine_test_world_exposes_all_ports() {
        let engine = Engine::test_world();
        let _: &Arc<dyn StorageAdapter> = engine.storage();
        let _: &Arc<dyn AuthProvider> = engine.auth();
        let _: &Arc<dyn NotificationProvider> = engine.notify();
        let _: &Arc<dyn PaymentProvider> = engine.payment();
        let _: &Arc<dyn FileStorage> = engine.files();
        let _: &Arc<dyn IntegrationGateway> = engine.integrations();
        let _: &Arc<dyn EventBus> = engine.bus();
        let _: &Arc<dyn Clock> = engine.clock();
        let _: &Arc<dyn IdGenerator> = engine.id_gen();
    }
}
