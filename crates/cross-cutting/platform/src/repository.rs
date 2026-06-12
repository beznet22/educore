//! Platform-domain repository port traits.
//!
//! Per `docs/ports/storage.md` and the engine's tier rules,
//! the `cross-cutting` tier does not depend on the `adapters`
//! tier; the `SchoolRepository` / `UserRepository` traits
//! are ports the storage adapter crates implement.
//!
//! Each method takes a [`TenantContext`](educore_core::tenant::TenantContext)
//! for read and write methods; the read methods
//! (`get`, `get_by_*`, `list_*`) all filter by the context's
//! `school_id` so the adapter cannot accidentally surface a
//! cross-tenant row. The global reads (`list_approved`,
//! `list_pending`) intentionally take no `TenantContext` and
//! are gated behind the platform-administrator capability at
//! the dispatcher; the storage adapter still enforces the
//! read.

use async_trait::async_trait;

use educore_core::error::Result;
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};

use crate::aggregate::{School, User};
use crate::value_objects::RoleId;

/// Repository port for [`School`] aggregates.
///
/// The trait is `Send + Sync` so consumers can hold an
/// `Arc<dyn SchoolRepository>` in a multi-threaded runtime.
#[async_trait]
pub trait SchoolRepository: Send + Sync {
    /// Fetches the school with `id` (no tenant filter — a
    /// school is the tenant root, not nested in a parent
    /// tenant).
    async fn get(&self, id: SchoolId) -> Result<Option<School>>;

    /// Fetches the school whose `domain` matches. Returns
    /// `Ok(None)` if no school has that domain.
    async fn get_by_domain(&self, domain: &str) -> Result<Option<School>>;

    /// Fetches the school whose `school_code` matches.
    /// Returns `Ok(None)` if no school has that code.
    async fn get_by_code(&self, code: &str) -> Result<Option<School>>;

    /// Lists all schools (no tenant filter; gated by RBAC at
    /// the dispatcher). Paginated by `offset` and `limit`.
    async fn list(&self, offset: u32, limit: u32) -> Result<Vec<School>>;

    /// Lists all schools with `status = Approved` or
    /// `status = Active`. Used by the platform-admin
    /// dashboard.
    async fn list_approved(&self) -> Result<Vec<School>>;

    /// Lists all schools with `status = Pending` (i.e.
    /// awaiting onboarding approval).
    async fn list_pending(&self) -> Result<Vec<School>>;

    /// Inserts a new school row. Returns `Err(Conflict)` if a
    /// row with the same `id` already exists.
    async fn insert(&self, school: &School) -> Result<()>;

    /// Updates an existing school row. Returns
    /// `Err(Conflict)` on optimistic-concurrency mismatch
    /// (the row's `version` does not match `school.version`).
    async fn update(&self, school: &School) -> Result<()>;
}

/// Repository port for [`User`] aggregates.
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Fetches the user with `id` (scoped to `ctx.school_id`).
    /// Returns `Ok(None)` if the user does not exist in the
    /// active tenant.
    async fn get(&self, ctx: &TenantContext, id: educore_core::ids::UserId)
        -> Result<Option<User>>;

    /// Fetches the user in `school` whose lowercased email
    /// matches.
    async fn get_by_email(&self, school: SchoolId, email: &str) -> Result<Option<User>>;

    /// Fetches the user in `school` whose username matches.
    async fn get_by_username(&self, school: SchoolId, username: &str) -> Result<Option<User>>;

    /// Fetches the user in `school` whose phone number
    /// matches (E.164 form).
    async fn get_by_phone(&self, school: SchoolId, phone: &str) -> Result<Option<User>>;

    /// Lists users in the active tenant. Paginated by
    /// `offset` and `limit`.
    async fn list(&self, ctx: &TenantContext, offset: u32, limit: u32) -> Result<Vec<User>>;

    /// Lists users in `school` who hold `role_id`. Used by
    /// the RBAC dashboard to enumerate "everyone with role
    /// X".
    async fn list_by_role(&self, school: SchoolId, role_id: RoleId) -> Result<Vec<User>>;

    /// Lists users in `school` with the given `usertype`.
    /// Used by dashboards that filter by role (e.g. "all
    /// teachers").
    async fn list_by_usertype(&self, school: SchoolId, usertype: UserType) -> Result<Vec<User>>;

    /// Inserts a new user row. Returns `Err(Conflict)` if a
    /// row with the same `(school_id, id)` already exists.
    async fn insert(&self, ctx: &TenantContext, user: &User) -> Result<()>;

    /// Updates an existing user row. Returns `Err(Conflict)`
    /// on optimistic-concurrency mismatch.
    async fn update(&self, ctx: &TenantContext, user: &User) -> Result<()>;
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

    /// Compile-time test: confirm the trait is object-safe by
    /// naming a `Box<dyn SchoolRepository>` and
    /// `Box<dyn UserRepository>`.
    #[test]
    fn traits_are_object_safe() {
        fn _is_object_safe(_: Box<dyn SchoolRepository>) {}
        fn _is_object_safe_user(_: Box<dyn UserRepository>) {}
    }
}
