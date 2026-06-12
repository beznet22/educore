//! # educore-event-bus
//!
//! Event bus port adapter implementations for the Educore
//! engine. This crate is a member of the `adapters` tier; the
//! bus port itself lives in `educore-events`.
//!
//! # Adapters
//!
//! - [`InProcessEventBus`] — the default, always-built, MPMC
//!   bus backed by [`tokio::sync::broadcast`]. MPMC; bounded
//!   channel per subscription; replay log for
//!   [`StartPosition::Earliest`](educore_events::StartPosition::Earliest).
//! - [`NatsEventBus`] — gated behind the `nats` Cargo feature.
//!   Phase 2 stub; the wire-protocol work lands in a future
//!   phase. All trait methods return
//!   [`EventError::NotSupported`](educore_events::errors::EventError::NotSupported).
//! - [`RedisEventBus`] — gated behind the `redis` Cargo feature.
//!   Phase 2 stub; same shape as `NatsEventBus`.
//!
//! # Feature flags
//!
//! ```text
//! default = ["in-process"]
//! in-process = []            // always built; this is the default
//! nats = ["dep:async-nats"]
//! redis = ["dep:redis"]
//! ```
//!
//! # Usage
//!
//! ```rust,no_run
//! use std::sync::Arc;
//! use educore_event_bus::InProcessEventBus;
//! use educore_events::event_bus::EventBus;
//!
//! let bus = Arc::new(InProcessEventBus::new());
//! // ... wire it into the engine via `Arc<dyn EventBus>`.
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// Package name constant. Re-exported so consumers can assert they
/// are using the right crate version at compile time.
pub const PACKAGE_NAME: &str = "educore-event-bus";

/// Package version at compile time.
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Error-mapping helpers (adapter tier).
pub mod errors;

/// The in-process MPMC event bus.
pub mod in_process;

/// The NATS JetStream-backed event bus (Phase 2 stub; gated
/// behind the `nats` Cargo feature).
#[cfg(feature = "nats")]
pub mod nats;

/// The Redis Streams-backed event bus (Phase 2 stub; gated
/// behind the `redis` Cargo feature).
#[cfg(feature = "redis")]
pub mod redis;

// ---- Public re-exports --------------------------------------------------

/// The default in-process bus. Re-exported at the crate root
/// so consumers can `use educore_event_bus::InProcessEventBus`.
pub use crate::in_process::{InProcessConfig, InProcessEventBus};

/// The NATS JetStream-backed bus. Re-exported at the crate
/// root; only present when the `nats` feature is enabled.
#[cfg(feature = "nats")]
pub use crate::nats::NatsEventBus;

/// The Redis Streams-backed bus. Re-exported at the crate
/// root; only present when the `redis` feature is enabled.
#[cfg(feature = "redis")]
pub use crate::redis::RedisEventBus;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_metadata_is_set() {
        assert_eq!(PACKAGE_NAME, "educore-event-bus");
        assert!(!PACKAGE_VERSION.is_empty());
    }

    #[test]
    fn in_process_config_default_is_exported() {
        // The default config is the one used by `InProcessEventBus::new()`.
        let cfg = InProcessConfig::default();
        assert!(cfg.channel_capacity >= 1);
        assert!(cfg.replay_log_capacity >= 1);
    }
}
