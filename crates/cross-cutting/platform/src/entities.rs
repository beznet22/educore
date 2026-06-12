//! Platform entities (non-root aggregates).
//!
//! Per `docs/specs/platform/entities.md`, the platform domain
//! has several child entities that live under an aggregate root
//! but have their own identity. Phase 2 ships the four entities
//! most directly tied to the [`School`](crate::aggregate::School)
//! and [`User`](crate::aggregate::User) aggregates:
//!
//! - [`SchoolContact`] — a secondary contact for a school
//!   (principal, accountant, ...).
//! - [`UserSession`] — an active session for a user (invalidated
//!   on `UserDeactivated`).
//! - [`UserPreference`] — per-user settings the base `User`
//!   row does not carry (timezone override, custom dashboard
//!   layout, ...).
//! - [`UserLogin`] — a historical login record (one row per
//!   successful login).
//!
//! The remaining entities (`UserDocument`, `CourseInstructor`,
//! `CourseMaterial`, ...) are out of scope for Phase 2 and
//! land alongside their owning aggregates in later phases.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::{SchoolId, SessionId, UserId};
use educore_core::tenant::UserType;
use educore_core::value_objects::Timestamp;

/// A point of contact for a school — name, role, phone, email.
///
/// The underlying school row carries the primary phone/email;
/// a school may have many contact persons (e.g. principal,
/// accountant, receptionist).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchoolContact {
    /// The inner row id (UUIDv7). Scoped to the school.
    pub id: Uuid,
    /// The owning school.
    pub school_id: SchoolId,
    /// The contact's display name (1..=200 chars).
    pub name: String,
    /// The contact's role at the school (free-form, 1..=200 chars;
    /// e.g. `"Principal"`, `"Accountant"`).
    pub role: String,
    /// The contact's phone number, in E.164 form.
    pub phone: String,
    /// The contact's email, lowercased on storage.
    pub email: String,
    /// Whether this is the school's primary contact.
    pub is_primary: bool,
    /// Row creation timestamp.
    pub created_at: Timestamp,
}

/// An active session for a user.
///
/// The session id is the [`SessionId`] carried by the
/// `TenantContext`. Sessions are invalidated on
/// `UserDeactivated` (the `rbac` subscriber consumes the event
/// and hard-deletes all `UserSession` rows for the user).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserSession {
    /// The inner row id (UUIDv7). Scoped to the school.
    pub id: Uuid,
    /// The owning school.
    pub school_id: SchoolId,
    /// The user the session belongs to.
    pub user_id: UserId,
    /// The opaque session id (hashed; the plaintext is never
    /// stored).
    pub session_id: SessionId,
    /// The IP address that opened the session. Stored as a
    /// string (IPv4 or IPv6).
    pub ip_address: String,
    /// The user agent that opened the session (truncated to
    /// 1024 chars by the storage adapter).
    pub user_agent: String,
    /// When the session was issued.
    pub issued_at: Timestamp,
    /// When the session expires. The session is rejected
    /// after this point.
    pub expires_at: Timestamp,
}

/// A per-user preference record.
///
/// The base `User` row carries the most-used preferences
/// (`language`, `style_id`, `rtl_ltl`, `selected_session`);
/// the [`UserPreference`] entity is the typed storage for
/// preferences the base row does not hold (timezone override,
/// custom dashboard layout, notification toggles, ...).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserPreference {
    /// The inner row id (UUIDv7).
    pub id: Uuid,
    /// The owning school.
    pub school_id: SchoolId,
    /// The user the preferences belong to.
    pub user_id: UserId,
    /// The preference's stable key (e.g. `"dashboard.layout"`,
    /// `"timezone.override"`). The storage adapter enforces
    /// uniqueness on `(user_id, key)`.
    pub key: String,
    /// The preference's value, stored as a typed JSON blob.
    /// The engine does not parse the value; consumers read it
    /// as opaque JSON. (We use a `String` for the value body
    /// — the engine's value type is JSON-shaped; the storage
    /// adapter parses / serialises the value at the column
    /// boundary.)
    pub value_json: String,
    /// When the preference was last updated.
    pub updated_at: Timestamp,
}

/// A historical login record (one row per successful login).
///
/// The `operations` domain subscribes to a `UserLogged` event
/// to materialise the row; the engine itself does not insert
/// `UserLogin` rows directly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserLogin {
    /// The inner row id (UUIDv7).
    pub id: Uuid,
    /// The owning school.
    pub school_id: SchoolId,
    /// The user that logged in.
    pub user_id: UserId,
    /// The user type at the moment of login.
    pub usertype: UserType,
    /// The IP address that performed the login.
    pub ip_address: String,
    /// The user agent that performed the login.
    pub user_agent: String,
    /// When the login was recorded.
    pub logged_in_at: Timestamp,
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
    fn school_contact_round_trip_serde() {
        let c = SchoolContact {
            id: Uuid::now_v7(),
            school_id: SchoolId(Uuid::now_v7()),
            name: "Grace Hopper".to_owned(),
            role: "Principal".to_owned(),
            phone: "+14155552671".to_owned(),
            email: "grace@example.com".to_owned(),
            is_primary: true,
            created_at: Timestamp::epoch(),
        };
        let s = serde_json::to_string(&c).unwrap();
        let back: SchoolContact = serde_json::from_str(&s).unwrap();
        assert_eq!(c, back);
    }

    #[test]
    fn user_session_carries_session_id() {
        let s = UserSession {
            id: Uuid::now_v7(),
            school_id: SchoolId(Uuid::now_v7()),
            user_id: UserId(Uuid::now_v7()),
            session_id: SessionId(Uuid::now_v7()),
            ip_address: "127.0.0.1".to_owned(),
            user_agent: "curl/8.0".to_owned(),
            issued_at: Timestamp::epoch(),
            expires_at: Timestamp::epoch(),
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: UserSession = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }
}
