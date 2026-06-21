//! # educore-sdk
//!
//! High-level consumer SDK for the Educore engine. Provides
//! `Engine::builder()` for wiring the engine's 6 ports (storage,
//! auth, notify, payment, files, integrations) + clock + id
//! generator, and facade services that wrap the most common
//! consumer workflows (admit, attendance, payment, notify).
//!
//! See `docs/library-docs.md` for the consumer-facing API and
//! `docs/build-plan.md` § "Phase 16" for the implementation
//! context.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod engine;
pub mod errors;
pub mod facade;

pub use engine::{Engine, EngineBuilder};
pub use errors::SdkError;
pub use facade::{AdmissionService, AttendanceService, NotificationService, PaymentService};

#[cfg(test)]
mod tests {
    #[test]
    fn package_metadata_is_set() {
        assert_eq!(env!("CARGO_PKG_NAME"), "educore-sdk");
    }
}
