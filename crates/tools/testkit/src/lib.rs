//! # educore-testkit
//!
//! In-memory test adapters for the engine's seven ports
//! (StorageAdapter + AuthProvider + NotificationProvider +
//! PaymentProvider + FileStorage + IntegrationGateway +
//! EventBus). For unit and integration tests only.
//!
//! Consumer tests wire `TestkitWorld::new()` to get a
//! self-contained in-process world; the engine can then exercise
//! command flows without docker, without a real database, and
//! without a real OAuth / SMTP / Stripe / S3 / LMS / Zoom
//! dependency.
//!
//! This crate is a member of the Educore workspace. See
//! `docs/architecture.md` and the port contracts in
//! `docs/ports/{storage,authentication,notifications,payments,
//! file-storage,integrations,event-bus}.md` for behavioral
//! details.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// Package name constant. Re-exported so consumers can assert they
/// are using the right crate version at compile time.
pub const PACKAGE_NAME: &str = "educore-testkit";

/// Package version at compile time.
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The testkit error type. Concrete adapters (storage, auth, notify,
/// payment, files, integrations, event-bus) return
/// `Result<T, TestkitError>`; the variants below cover every
/// failure mode the in-memory backends can produce.
pub mod errors;
pub use errors::TestkitError;

/// Convenience alias for the testkit's [`Result`] type.
pub type Result<T> = std::result::Result<T, TestkitError>;

// ---- In-memory port impls (each in its own module) -----------------------

/// In-memory `StorageAdapter` + `Transaction` + 5 sub-port impls
/// (`Outbox`, `AuditLog`, `EventLog`, `Idempotency`, change-stream
/// primitives).
pub mod storage;

/// In-memory `AuthProvider` (accepts all `Credential` variants;
/// mints `Session` values from a `TenantContext`).
pub mod auth;

/// In-memory `NotificationProvider` (records sends, returns a
/// `NotificationReceipt` with a synthetic provider id).
pub mod notify;

/// In-memory `PaymentProvider` (charges return a `PaymentReceipt`;
/// idempotent on the `idempotency_key`).
pub mod payment;

/// In-memory `FileStorage` (put/get/delete/exists/head/signed_url/
/// copy/move_to against an in-process `HashMap`).
pub mod files;

/// In-memory `IntegrationGateway` (records invocations, returns a
/// canned `IntegrationResponse::Success`).
pub mod integrations;

/// Thin in-process event bus. Re-exports
/// [`educore_event_bus::InProcessEventBus`] under a testkit-local
/// name so consumers can wire `Arc<dyn EventBus>` without taking
/// a direct dep on `educore-event-bus`.
pub mod event_bus;

/// In-memory `ChangeStream` + per-school `VersionCursor` table for
/// the storage-port sync primitives (`watch_changes`,
/// `apply_snapshot`, `cursor_for`, `advance_cursor`).
pub mod sync;

// ---- Bundle type ---------------------------------------------------------

/// A self-contained, in-process engine world. All seven ports are
/// wired to in-memory impls; the event bus is the default
/// `InProcessEventBus`. Consumer tests construct one of these
/// (typically via [`TestkitWorld::new`]) and use the accessors to
/// hand `Arc<dyn ...>` references to the engine.
#[derive(Clone, Debug)]
pub struct TestkitWorld {
    /// The in-memory storage adapter.
    pub storage: std::sync::Arc<storage::InMemoryStorageAdapter>,
    /// The in-memory auth provider.
    pub auth: std::sync::Arc<auth::InMemoryAuthProvider>,
    /// The in-memory notification provider.
    pub notify: std::sync::Arc<notify::InMemoryNotificationProvider>,
    /// The in-memory payment provider.
    pub payment: std::sync::Arc<payment::InMemoryPaymentProvider>,
    /// The in-memory file storage.
    pub files: std::sync::Arc<files::InMemoryFileStorage>,
    /// The in-memory integration gateway.
    pub integrations: std::sync::Arc<integrations::InMemoryIntegrationGateway>,
    /// The in-process event bus (re-exported from
    /// `educore-event-bus`).
    pub bus: std::sync::Arc<dyn educore_events::event_bus::EventBus>,
}

impl TestkitWorld {
    /// Constructs a fresh `TestkitWorld`. Each call returns an
    /// independent world — the in-memory backends are not shared
    /// across `TestkitWorld` instances. The bus is the default
    /// `InProcessEventBus` (1024-channel-capacity, 4096-replay-log).
    #[must_use]
    pub fn new() -> Self {
        let bus: std::sync::Arc<dyn educore_events::event_bus::EventBus> =
            std::sync::Arc::new(educore_event_bus::InProcessEventBus::new());
        Self {
            storage: std::sync::Arc::new(storage::InMemoryStorageAdapter::new(bus.clone())),
            auth: std::sync::Arc::new(auth::InMemoryAuthProvider::new()),
            notify: std::sync::Arc::new(notify::InMemoryNotificationProvider::new()),
            payment: std::sync::Arc::new(payment::InMemoryPaymentProvider::new()),
            files: std::sync::Arc::new(files::InMemoryFileStorage::new()),
            integrations: std::sync::Arc::new(integrations::InMemoryIntegrationGateway::new()),
            bus,
        }
    }
}

impl Default for TestkitWorld {
    fn default() -> Self {
        Self::new()
    }
}

/// Constructs a fresh [`TestkitWorld`]. Convenience alias for
/// [`TestkitWorld::new`] so consumers can write
/// `educore_testkit::test_world()` without the prefix.
///
/// Each call returns an independent world — the in-memory
/// backends are not shared across `TestkitWorld` instances. The
/// bus is the default `InProcessEventBus` (1024-channel-
/// capacity, 4096-replay-log).
#[must_use]
pub fn test_world() -> TestkitWorld {
    TestkitWorld::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_metadata_is_set() {
        assert_eq!(PACKAGE_NAME, "educore-testkit");
        assert!(!PACKAGE_VERSION.is_empty());
    }

    #[test]
    fn testkit_world_constructs_with_all_seven_ports() {
        let world = TestkitWorld::new();
        // The struct fields are `pub`, so a direct field check is
        // the simplest contract assertion. The Arc<dyn ...> types
        // do not need to be downcast — existence is sufficient
        // proof of construction.
        let _: &std::sync::Arc<storage::InMemoryStorageAdapter> = &world.storage;
        let _: &std::sync::Arc<auth::InMemoryAuthProvider> = &world.auth;
        let _: &std::sync::Arc<notify::InMemoryNotificationProvider> = &world.notify;
        let _: &std::sync::Arc<payment::InMemoryPaymentProvider> = &world.payment;
        let _: &std::sync::Arc<files::InMemoryFileStorage> = &world.files;
        let _: &std::sync::Arc<integrations::InMemoryIntegrationGateway> = &world.integrations;
        // The bus is `Arc<dyn EventBus>`; assert it derefs.
        let _ = &*world.bus;
    }

    #[test]
    fn test_world_function_constructs_testkit_world() {
        let _world = test_world();
    }
}
