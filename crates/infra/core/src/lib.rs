//! # educore-core
//!
//! The foundation crate for the Educore engine. Every other crate
//! in the workspace depends on this one. Per
//! `docs/architecture.md` and `AGENTS.md`:
//!
//! - Every engine-wide type lives here: errors, typed identifiers,
//!   value objects, clock and id-generation ports, the tenant
//!   context, and the shared query AST.
//! - The `lint` sub-module (gated behind the `lint` Cargo feature
//!   per `docs/build-plan.md` § "The No-Gaps Gates") is the
//!   build-time enforcer of the tier and dependency-direction
//!   rules. It is not compiled in release by default; consumer
//!   crates opt in via `features = ["lint"]` for the `cargo run
//!   --bin lint` workflow described in `docs/build-plan.md`.
//! - This crate is `#![forbid(unsafe_code)]` and
//!   `#![deny(missing_docs)]`. All public types carry rustdoc.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// Time and id-generation ports. Production code wires
/// [`clock::SystemClock`] and [`clock::SystemIdGen`]; tests wire
/// the [`clock::TestClock`] and [`clock::DeterministicIdGen`]
/// variants.
pub mod clock;

/// The engine's universal error type ([`error::DomainError`]) and
/// its [`error::Result`] alias.
pub mod error;

/// Typed identifiers wrapping UUIDv7. Every aggregate and every
/// cross-cutting concept has a typed id so cross-aggregate id
/// confusion is a compile-time error.
pub mod ids;

/// The dialect-agnostic query AST consumed by the
/// `#[derive(DomainQuery)]` macro and by storage adapters.
pub mod query;

/// The [`tenant::TenantContext`] — the active tenant for a single
/// command or query.
pub mod tenant;

/// Cross-cutting value objects: [`value_objects::Timestamp`],
/// [`value_objects::Version`], [`value_objects::Etag`],
/// [`value_objects::ActiveStatus`], and [`value_objects::Source`].
pub mod value_objects;

/// Build-time enforcer of the no-gaps gates. Gated behind the
/// `lint` Cargo feature. The runner lives at [`lint::run`]; the
/// companion binary (`src/bin/lint.rs`) prints a report and
/// returns a non-zero exit code on violations.
///
/// Per `docs/build-plan.md` § "The No-Gaps Gates" item 2: the
/// canonical invocation is
/// `cargo run -p educore-core --bin lint --features lint`.
#[cfg(feature = "lint")]
pub mod lint;

/// Re-exports of the engine's most-used types. Consumers of the
/// engine typically `use educore_core::prelude::*;` once at the top
/// of a file.
pub mod prelude {
    pub use crate::clock::{
        Clock, DeterministicIdGen, IdGenerator, SystemClock, SystemIdGen, TestClock,
    };
    pub use crate::error::{DomainError, ErrorKind, Result};
    pub use crate::ids::{
        CorrelationId, EventId, IdempotencyKey, Identifier, SchoolId, SessionId, UserId,
        PLATFORM_SCHOOL_ID, SYSTEM_USER_ID,
    };
    pub use crate::query::{
        Field, HasRelations, OrderDirection, OrderNode, Page, Pattern, QueryNode, Relation,
        RelationalField, TenantScope, Value,
    };
    pub use crate::tenant::{Locale, TenantContext, TenantContextBuilder, TimeZone, UserType};
    pub use crate::value_objects::{ActiveStatus, Etag, Source, Timestamp, Version};
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
    use crate::clock::DeterministicIdGen;
    use crate::clock::IdGenerator;

    #[test]
    fn prelude_round_trip() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let user = g.next_user_id();
        let ctx = TenantContext::for_user(school, user, g.next_correlation_id(), UserType::Teacher);
        assert_eq!(ctx.school_id, school);
        assert_eq!(ctx.actor_id, user);
    }

    #[test]
    fn deterministic_gen_works_through_trait() {
        let g = DeterministicIdGen::starting_at(7);
        let id: SchoolId = g.next_school_id();
        assert_eq!(id.as_uuid().get_version_num(), 7);
    }
}
