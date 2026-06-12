//! # educore-platform
//!
//! Multi-tenancy substrate for the Educore engine.
//!
//! Phase 2 ships the [`School`] and [`User`] aggregates, the
//! supporting value objects, commands, and events, and the
//! per-aggregate repository port traits. The 30 secondary
//! aggregates enumerated in `docs/specs/platform/aggregates.md`
//! (Course, OtpCode, Module, Plugin, ...) land in later phases;
//! their commands and events are out of scope for Phase 2.
//!
//! See `docs/specs/platform/aggregates.md`, `commands.md`,
//! `events.md`, and `value-objects.md` for the design contract.
//! This crate depends only on `educore-core` and `educore-events`
//! (the bus port) and is itself a member of the `cross-cutting`
//! tier. It has no dependency on storage or bus adapters; the
//! `SchoolRepository` / `UserRepository` traits are ports the
//! adapter crates implement.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod aggregate;
pub mod commands;
pub mod entities;
mod errors;
pub mod events;
pub mod query;
pub mod repository;
pub mod services;
pub mod value_objects;

/// Package name constant. Re-exported so consumers can assert they
/// are using the right crate version at compile time.
pub const PACKAGE_NAME: &str = "educore-platform";

/// Package version at compile time.
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The aggregate roots: [`School`] and [`User`].
pub use crate::aggregate::{School, User};

/// Typed events emitted by the platform commands.
pub use crate::events::{
    SchoolCreated, SchoolDeactivated, SchoolUpdated, UserDeactivated, UserRegistered, UserUpdated,
};

/// Pure factory functions that turn a command into an aggregate
/// plus a typed event.
pub use crate::services::{
    create_school, deactivate_school, deactivate_user, register_user, update_school, update_user,
};

/// Per-aggregate repository port traits.
pub use crate::repository::{SchoolRepository, UserRepository};

/// The query builder stubs (typed query is wired in Phase 3+).
pub use crate::query::{SchoolQuery, UserQuery};

/// The platform's local error helpers.
pub use crate::errors::PlatformError;

/// Typed value objects (re-exported from `educore-core` and the
/// platform's local newtypes).
pub use crate::value_objects::{
    EmailAddress, HashedPassword, PackageId, PhoneNumber, RoleId, SchoolStatus, UserStatus,
};

/// Re-exports of the engine types the platform crate most commonly
/// reaches for. Consumers should `use educore_platform::prelude::*;`
/// at the top of a file.
pub mod prelude {
    pub use educore_core::clock::{
        Clock, DeterministicIdGen, IdGenerator, SystemClock, SystemIdGen, TestClock,
    };
    pub use educore_core::error::{DomainError, ErrorKind, Result};
    pub use educore_core::ids::{
        CorrelationId, EventId, Identifier, SchoolId, SessionId, UserId, PLATFORM_SCHOOL_ID,
        SYSTEM_USER_ID,
    };
    pub use educore_core::tenant::{
        Locale, TenantContext, TenantContextBuilder, TimeZone, UserType,
    };
    pub use educore_core::value_objects::{ActiveStatus, Etag, Source, Timestamp, Version};

    pub use educore_events::domain_event::{DomainEvent, EmittedEvent, EventFactory};
    pub use educore_events::envelope::EventEnvelope;

    pub use crate::aggregate::{School, User};
    pub use crate::commands::{
        CreateSchoolCommand, DeactivateSchoolCommand, DeactivateUserCommand, RegisterUserCommand,
        UpdateSchoolCommand, UpdateUserCommand,
    };
    pub use crate::entities::{SchoolContact, UserLogin, UserPreference, UserSession};
    pub use crate::errors::PlatformError;
    pub use crate::events::{
        SchoolCreated, SchoolDeactivated, SchoolUpdated, UserDeactivated, UserRegistered,
        UserUpdated,
    };
    pub use crate::query::{SchoolQuery, UserQuery};
    pub use crate::repository::{SchoolRepository, UserRepository};
    pub use crate::services::{
        create_school, deactivate_school, deactivate_user, register_user, update_school,
        update_user,
    };
    pub use crate::value_objects::{
        EmailAddress, HashedPassword, PackageId, PhoneNumber, RoleId, SchoolStatus, UserStatus,
    };
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::prelude::*;
    use super::{PACKAGE_NAME, PACKAGE_VERSION};

    #[test]
    fn package_metadata_is_set() {
        assert_eq!(PACKAGE_NAME, "educore-platform");
        assert!(!PACKAGE_VERSION.is_empty());
    }

    #[test]
    fn prelude_re_exports_aggregate_types() {
        // The platform crate's prelude must surface both
        // aggregates and the engine's cross-cutting types so
        // that consumers can `use educore_platform::prelude::*;`
        // in a single statement.
        let school: fn() -> School = || School {
            id: PLATFORM_SCHOOL_ID,
            name: String::new(),
            domain: None,
            school_code: String::new(),
            status: SchoolStatus::Pending,
            package_id: None,
            version: Version::initial(),
            etag: Etag::new("0123456789abcdef0123456789abcdef").unwrap(),
            created_at: Timestamp::epoch(),
            updated_at: Timestamp::epoch(),
            created_by: SYSTEM_USER_ID,
            updated_by: SYSTEM_USER_ID,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id: CorrelationId(uuid::Uuid::nil()),
        };
        let _ = school;
    }
}
