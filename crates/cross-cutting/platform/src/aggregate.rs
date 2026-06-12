//! Platform aggregate roots.
//!
//! Phase 2 ships two aggregates:
//! - [`School`] — the tenant root. Globally unique (not
//!   nested inside another school). Holds the school's
//!   identity, contact info, package binding, and active
//!   status.
//! - [`User`] — an actor in a school. Tenant-scoped to
//!   `SchoolId`. Holds the user's email, username, role
//!   bindings, status, and authentication material.
//!
//! Both aggregates are **structs** (not enums): the engine uses
//! the "aggregate as a single struct" pattern, with the
//! `active_status` field carrying the soft-delete flag and the
//! `version` field carrying the optimistic-concurrency
//! counter.

use serde::{Deserialize, Serialize};

use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::tenant::UserType;
use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};

use crate::value_objects::{
    EmailAddress, HashedPassword, PackageId, PhoneNumber, RoleId, SchoolStatus, UserStatus,
};

/// Returns the default etag for a freshly minted aggregate.
///
/// The string is a compile-time constant: 32 lowercase hex
/// characters. The helper delegates to [`Etag::placeholder`]
/// (an infallible constructor) so the caller does not need
/// to handle a `Result<Self>`.
fn fresh_etag() -> Etag {
    Etag::placeholder()
}

/// The tenant root. A school is the unit of tenancy; every
/// per-tenant aggregate (User, Student, Invoice, etc.) anchors
/// to a `School` via `SchoolId`.
///
/// Schools are **globally unique** — they are not nested inside
/// another aggregate. A school's id has no parent tenant, and
/// the `School` row is the canonical tenant anchor used by
/// `TenantContext::school_id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct School {
    /// The school's typed id.
    pub id: SchoolId,
    /// Human-readable display name (1..=200 chars).
    pub name: String,
    /// Optional public-facing domain (e.g. `"ada-school.example.com"`).
    pub domain: Option<String>,
    /// Unique short code used by URLs and CSV exports
    /// (1..=200 chars, unique across the platform).
    pub school_code: String,
    /// The school's lifecycle status. The `Pending` value is
    /// the initial state on creation; `Approved` is the steady
    /// state once the school passes onboarding.
    pub status: SchoolStatus,
    /// The package plan the school is on. `None` for a brand-
    /// new school that hasn't been assigned a package yet.
    pub package_id: Option<PackageId>,
    /// Optimistic-concurrency counter. Increments on every
    /// mutation; the storage adapter rejects writes whose
    /// `version` does not match the row's current `version`.
    pub version: Version,
    /// Content hash of the row, used for `If-Match` /
    /// conditional-write flows. Recomputed by the storage
    /// adapter on every successful write.
    pub etag: Etag,
    /// Creation timestamp.
    pub created_at: Timestamp,
    /// Last-mutation timestamp.
    pub updated_at: Timestamp,
    /// The actor that created the row.
    pub created_by: UserId,
    /// The actor that last mutated the row.
    pub updated_by: UserId,
    /// Soft-delete flag. The storage adapter filters
    /// `active_status = 0` rows from ordinary queries.
    pub active_status: ActiveStatus,
    /// The id of the last event that mutated the school. Set
    /// by the command handler; consumed by the outbox writer
    /// for transactional outbox pattern.
    pub last_event_id: Option<EventId>,
    /// The correlation id of the request that created or last
    /// mutated the school. Mirrored from `TenantContext` so
    /// that audit-log readers can join without a second hop.
    pub correlation_id: CorrelationId,
}

impl School {
    /// The default etag for a freshly minted school. The
    /// storage adapter will overwrite this with the computed
    /// content hash on the first successful insert.
    pub const FRESH_ETAG: &'static str = "00000000000000000000000000000000";

    /// Returns a `School` in its just-minted state (not yet
    /// persisted). The `etag` is the all-zero placeholder;
    /// `version` is the initial `1`.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn fresh(
        id: SchoolId,
        name: String,
        school_code: String,
        domain: Option<String>,
        package_id: Option<PackageId>,
        created_by: UserId,
        updated_by: UserId,
        now: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        let etag = fresh_etag();
        Self {
            id,
            name,
            domain,
            school_code,
            status: SchoolStatus::Pending,
            package_id,
            version: Version::initial(),
            etag,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}

/// An actor in a school. A user belongs to exactly one school
/// (the `school_id` field). The user's id is **globally unique**;
/// the per-school id embedding used by other aggregates
/// (Student, etc.) is not applied here because users are
/// referenced cross-tenant by the platform (audit log,
/// outbox payload, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    /// The user's typed id.
    pub id: UserId,
    /// The owning school. Mandatory.
    pub school_id: SchoolId,
    /// The user's email (validated, lowercased).
    pub email: EmailAddress,
    /// The user's chosen username (unique within the school).
    pub username: String,
    /// Optional phone number (E.164 form).
    pub phone_number: Option<PhoneNumber>,
    /// Display name shown in the UI. Defaults to the
    /// registration's full name; the user can change it
    /// without changing the underlying identity.
    pub display_name: String,
    /// The actor's role in the school. Drives default RBAC
    /// bindings.
    pub usertype: UserType,
    /// The user's role bindings. Empty for a newly minted user
    /// (the role is set by a follow-up `ChangeUserRole`
    /// command). Multi-role is permitted.
    pub role_ids: Vec<RoleId>,
    /// The user's lifecycle status. Defaults to `Active` on
    /// registration; toggled to `Inactive` / `Suspended` by
    /// the `DeactivateUser` command.
    pub status: UserStatus,
    /// The password hash. Wrapped in `SecretString` so the
    /// `Debug` impl redacts the value. The plaintext is never
    /// stored; the adapter populates this from the engine's
    /// password-hashing port.
    pub password_hash: HashedPassword,
    /// Optimistic-concurrency counter.
    pub version: Version,
    /// Content hash for conditional writes.
    pub etag: Etag,
    /// Creation timestamp.
    pub created_at: Timestamp,
    /// Last-mutation timestamp.
    pub updated_at: Timestamp,
    /// The actor that created the user (typically the school
    /// admin or the registering user themselves).
    pub created_by: UserId,
    /// The actor that last mutated the user.
    pub updated_by: UserId,
    /// Soft-delete flag.
    pub active_status: ActiveStatus,
    /// The id of the last event that mutated the user.
    pub last_event_id: Option<EventId>,
    /// The correlation id of the request that created or last
    /// mutated the user.
    pub correlation_id: CorrelationId,
}

impl User {
    /// The default etag for a freshly minted user.
    pub const FRESH_ETAG: &'static str = "00000000000000000000000000000000";

    /// Returns a `User` in its just-minted state (not yet
    /// persisted).
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn fresh(
        id: UserId,
        school_id: SchoolId,
        email: EmailAddress,
        username: String,
        display_name: String,
        phone_number: Option<PhoneNumber>,
        usertype: UserType,
        role_ids: Vec<RoleId>,
        password_hash: HashedPassword,
        created_by: UserId,
        updated_by: UserId,
        now: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        let etag = fresh_etag();
        Self {
            id,
            school_id,
            email,
            username,
            phone_number,
            display_name,
            usertype,
            role_ids,
            status: UserStatus::Active,
            password_hash,
            version: Version::initial(),
            etag,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    dead_code
)]
mod tests {
    use super::*;

    #[test]
    fn school_fresh_starts_at_initial_version() {
        let s = School::fresh(
            SchoolId(uuid::Uuid::nil()),
            "Ada".to_owned(),
            "ADA".to_owned(),
            None,
            None,
            UserId(uuid::Uuid::nil()),
            UserId(uuid::Uuid::nil()),
            Timestamp::epoch(),
            CorrelationId(uuid::Uuid::nil()),
        );
        assert_eq!(s.version, Version::initial());
        assert_eq!(s.status, SchoolStatus::Pending);
        assert!(s.active_status.is_active());
        assert_eq!(s.last_event_id, None);
    }

    #[test]
    fn user_fresh_starts_active_with_no_roles() {
        let u = User::fresh(
            UserId(uuid::Uuid::nil()),
            SchoolId(uuid::Uuid::nil()),
            EmailAddress::new("ada@example.com").unwrap(),
            "ada".to_owned(),
            "Ada".to_owned(),
            None,
            UserType::Student,
            Vec::new(),
            HashedPassword::from_hash("$argon2id$dummy"),
            UserId(uuid::Uuid::nil()),
            UserId(uuid::Uuid::nil()),
            Timestamp::epoch(),
            CorrelationId(uuid::Uuid::nil()),
        );
        assert_eq!(u.version, Version::initial());
        assert!(u.status.can_authenticate());
        assert!(u.role_ids.is_empty());
        assert!(u.active_status.is_active());
    }

    #[test]
    fn etag_fresh_constant_is_32_lowercase_hex() {
        Etag::new(School::FRESH_ETAG).expect("FRESH_ETAG must be a valid etag");
        Etag::new(User::FRESH_ETAG).expect("FRESH_ETAG must be a valid etag");
    }
}
