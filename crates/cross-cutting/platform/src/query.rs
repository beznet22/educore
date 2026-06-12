//! Platform-domain query builders.
//!
//! Phase 2 ships typed query stubs. The full query builder
//! (filter combinators, joins, eager-loads) lands in
//! Phase 3+ alongside the `#[derive(DomainQuery)]` macro
//! emissions. For now, the query types carry the future
//! shape so callers can hold a `SchoolQuery` / `UserQuery`
//! value and the storage adapter can pattern-match on them
//! when the real implementation lands.

use serde::{Deserialize, Serialize};

use educore_core::error::{DomainError, Result};
use educore_core::ids::SchoolId;
use educore_core::tenant::UserType;

use crate::aggregate::{School, User};
use crate::value_objects::RoleId;

/// A query for [`School`] aggregates.
///
/// The Phase 2 shape is a small surface area: the only
/// "real" field is `status_filter`, used by the platform-
/// admin dashboard to list schools in a given lifecycle
/// state. The remaining variants / combinators land in
/// Phase 3+ alongside the query AST.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchoolQuery {
    /// Optional status filter.
    pub status_filter: Option<crate::value_objects::SchoolStatus>,
    /// Optional substring match on `name`.
    pub name_contains: Option<String>,
    /// Optional substring match on `school_code`.
    pub code_contains: Option<String>,
}

impl SchoolQuery {
    /// Constructs an empty `SchoolQuery` (returns every
    /// school, subject to the adapter's tenant filter).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            status_filter: None,
            name_contains: None,
            code_contains: None,
        }
    }

    /// Sets the status filter.
    #[must_use]
    pub fn with_status(mut self, status: crate::value_objects::SchoolStatus) -> Self {
        self.status_filter = Some(status);
        self
    }

    /// Sets the name-substring filter.
    #[must_use]
    pub fn with_name_contains(mut self, needle: impl Into<String>) -> Self {
        self.name_contains = Some(needle.into());
        self
    }

    /// Sets the code-substring filter.
    #[must_use]
    pub fn with_code_contains(mut self, needle: impl Into<String>) -> Self {
        self.code_contains = Some(needle.into());
        self
    }

    /// Stub: the real query executor lands in Phase 3+. For
    /// now this returns `Err(DomainError::NotSupported)` so
    /// callers can be wired up before the implementation
    /// lands and the lints stay clean.
    pub async fn execute(self, _school: SchoolId) -> Result<Vec<School>> {
        let _ = self;
        Err(DomainError::not_supported(
            "SchoolQuery::execute is a Phase 2 stub; the typed query executor lands in Phase 3+",
        ))
    }
}

/// A query for [`User`] aggregates.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserQuery {
    /// Optional `usertype` filter.
    pub usertype_filter: Option<UserType>,
    /// Optional `status` filter.
    pub status_filter: Option<crate::value_objects::UserStatus>,
    /// Optional `role_id` filter.
    pub role_filter: Option<RoleId>,
    /// Optional substring match on `username`.
    pub username_contains: Option<String>,
    /// Optional substring match on `email`.
    pub email_contains: Option<String>,
}

impl UserQuery {
    /// Constructs an empty `UserQuery`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            usertype_filter: None,
            status_filter: None,
            role_filter: None,
            username_contains: None,
            email_contains: None,
        }
    }

    /// Sets the `usertype` filter.
    #[must_use]
    pub fn with_usertype(mut self, usertype: UserType) -> Self {
        self.usertype_filter = Some(usertype);
        self
    }

    /// Sets the `status` filter.
    #[must_use]
    pub fn with_status(mut self, status: crate::value_objects::UserStatus) -> Self {
        self.status_filter = Some(status);
        self
    }

    /// Sets the `role_id` filter.
    #[must_use]
    pub fn with_role(mut self, role: RoleId) -> Self {
        self.role_filter = Some(role);
        self
    }

    /// Stub: the real query executor lands in Phase 3+.
    pub async fn execute(self, _school: SchoolId) -> Result<Vec<User>> {
        let _ = self;
        Err(DomainError::not_supported(
            "UserQuery::execute is a Phase 2 stub; the typed query executor lands in Phase 3+",
        ))
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

    #[test]
    fn school_query_builder_setter_methods() {
        let q = SchoolQuery::new()
            .with_status(crate::value_objects::SchoolStatus::Approved)
            .with_name_contains("Ada")
            .with_code_contains("ADA");
        assert_eq!(
            q.status_filter,
            Some(crate::value_objects::SchoolStatus::Approved)
        );
        assert_eq!(q.name_contains.as_deref(), Some("Ada"));
        assert_eq!(q.code_contains.as_deref(), Some("ADA"));
    }

    #[test]
    fn user_query_builder_setter_methods() {
        let g = SystemIdGen;
        let role = RoleId::new(g.next_school_id(), g.next_uuid());
        let q = UserQuery::new()
            .with_usertype(UserType::Teacher)
            .with_status(crate::value_objects::UserStatus::Active)
            .with_role(role);
        assert_eq!(q.usertype_filter, Some(UserType::Teacher));
        assert_eq!(
            q.status_filter,
            Some(crate::value_objects::UserStatus::Active)
        );
        assert_eq!(q.role_filter, Some(role));
    }

    #[tokio::test]
    async fn school_query_execute_returns_not_supported() {
        let g = SystemIdGen;
        let err = SchoolQuery::new()
            .execute(g.next_school_id())
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::NotSupported(_)));
    }

    #[tokio::test]
    async fn user_query_execute_returns_not_supported() {
        let g = SystemIdGen;
        let err = UserQuery::new()
            .execute(g.next_school_id())
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::NotSupported(_)));
    }
}
