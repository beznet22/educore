//! # educore-operations repository port traits
//!
//! Per `docs/specs/operations/repositories.md`. 8 root repository
//! port traits, one per root aggregate. Each is object-safe (the
//! `_assert_*_object_safe` helpers prove it). Plus 4 port-driven
//! repository traits (documented-as-not-owned) for OAuth,
//! password-reset, and migration infrastructure tables.

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

use async_trait::async_trait;
use uuid::Uuid;

use educore_core::error::Result as StorageResult;
use educore_core::ids::SchoolId;
use educore_core::value_objects::Timestamp;

use crate::aggregate::{
    Backup, FailedJob, Job, MaintenanceSetting, Sidebar, SystemVersion, UserLog, VersionHistory,
};
use crate::value_objects::{
    BackupFileName, BackupFileType, FailedJobConnection, FailedJobQueue, FailedJobUuid,
    HistoryVersion, JobQueue, LoginOutcome, PermissionId, RoleId, SidebarPosition,
    SidebarSectionId,
};

// =============================================================================
// === BackupRepository section begin (owner: B) ===
// =============================================================================

/// Repository port for [`Backup`](crate::aggregate::Backup).
#[async_trait]
pub trait BackupRepository: Send + Sync {
    /// Fetch a `Backup` by id.
    async fn get(&self, id: crate::value_objects::BackupId) -> StorageResult<Option<Backup>>;
    /// Fetch a `Backup` by `(school_id, file_name)`.
    async fn get_by_file_name(
        &self,
        school: SchoolId,
        file_name: &BackupFileName,
    ) -> StorageResult<Option<Backup>>;
    /// List all backups for a school.
    async fn list(&self, school: SchoolId) -> StorageResult<Vec<Backup>>;
    /// List backups of a given `file_type` for a school.
    async fn list_by_type(
        &self,
        school: SchoolId,
        file_type: BackupFileType,
    ) -> StorageResult<Vec<Backup>>;
    /// List active backups for a school.
    async fn list_active(&self, school: SchoolId) -> StorageResult<Vec<Backup>>;
    /// Insert a `Backup` row.
    async fn insert(&self, b: &Backup) -> StorageResult<()>;
    /// Update a `Backup` row.
    async fn update(&self, b: &Backup) -> StorageResult<()>;
    /// Delete a `Backup` row.
    async fn delete(&self, id: crate::value_objects::BackupId) -> StorageResult<()>;
    /// Return the count of restores currently in progress for the school.
    async fn restore_in_progress_count(&self, school: SchoolId) -> StorageResult<u32>;
}

fn _assert_backup_object_safe() {
    fn _f(_: Box<dyn BackupRepository>) {}
}

// === BackupRepository section end ===

// =============================================================================
// === JobRepository section begin (owner: B) ===
// =============================================================================

/// Repository port for [`Job`](crate::aggregate::Job). Global.
#[async_trait]
pub trait JobRepository: Send + Sync {
    /// Fetch a job by id.
    async fn get(&self, id: crate::value_objects::JobId) -> StorageResult<Option<Job>>;
    /// List all jobs.
    async fn list(&self) -> StorageResult<Vec<Job>>;
    /// List jobs for a queue.
    async fn list_for_queue(&self, queue: &JobQueue) -> StorageResult<Vec<Job>>;
    /// List jobs available at or before `now`, up to `limit`.
    async fn list_available(&self, now: Timestamp, limit: u32) -> StorageResult<Vec<Job>>;
    /// List jobs reserved before `before` (stale reservation sweep).
    async fn list_reserved(&self, before: Timestamp) -> StorageResult<Vec<Job>>;
    /// Insert a job.
    async fn insert(&self, j: &Job) -> StorageResult<()>;
    /// Update a job.
    async fn update(&self, j: &Job) -> StorageResult<()>;
    /// Delete a job.
    async fn delete(&self, id: crate::value_objects::JobId) -> StorageResult<()>;
    /// Purge completed jobs older than `before`. Returns the count purged.
    async fn purge_completed(&self, before: Timestamp) -> StorageResult<u64>;
}

fn _assert_job_object_safe() {
    fn _f(_: Box<dyn JobRepository>) {}
}

// === JobRepository section end ===

// =============================================================================
// === FailedJobRepository section begin (owner: B) ===
// =============================================================================

/// Repository port for [`FailedJob`](crate::aggregate::FailedJob). Global.
#[async_trait]
pub trait FailedJobRepository: Send + Sync {
    /// Fetch a `FailedJob` by id.
    async fn get(&self, id: crate::value_objects::FailedJobId) -> StorageResult<Option<FailedJob>>;
    /// Fetch a `FailedJob` by its business uuid.
    async fn get_by_uuid(&self, uuid: &FailedJobUuid) -> StorageResult<Option<FailedJob>>;
    /// List all `FailedJob` rows.
    async fn list(&self) -> StorageResult<Vec<FailedJob>>;
    /// List `FailedJob` rows for a given queue.
    async fn list_for_queue(&self, queue: &FailedJobQueue) -> StorageResult<Vec<FailedJob>>;
    /// List `FailedJob` rows for a given connection.
    async fn list_for_connection(
        &self,
        connection: &FailedJobConnection,
    ) -> StorageResult<Vec<FailedJob>>;
    /// Insert a `FailedJob`.
    async fn insert(&self, j: &FailedJob) -> StorageResult<()>;
    /// Delete a `FailedJob`.
    async fn delete(&self, id: crate::value_objects::FailedJobId) -> StorageResult<()>;
    /// Purge `FailedJob` rows older than `cutoff`. Returns the count purged.
    async fn purge_older_than(&self, cutoff: Timestamp) -> StorageResult<u64>;
}

fn _assert_failed_job_object_safe() {
    fn _f(_: Box<dyn FailedJobRepository>) {}
}

// === FailedJobRepository section end ===

// =============================================================================
// === SystemVersionRepository section begin (owner: B) ===
// =============================================================================

/// Repository port for [`SystemVersion`](crate::aggregate::SystemVersion). Global.
#[async_trait]
pub trait SystemVersionRepository: Send + Sync {
    /// Fetch a `SystemVersion` by id.
    async fn get(
        &self,
        id: crate::value_objects::SystemVersionId,
    ) -> StorageResult<Option<SystemVersion>>;
    /// Fetch a `SystemVersion` by `version_name`.
    async fn get_by_name(
        &self,
        version_name: &crate::value_objects::VersionName,
    ) -> StorageResult<Option<SystemVersion>>;
    /// List all `SystemVersion` rows.
    async fn list(&self) -> StorageResult<Vec<SystemVersion>>;
    /// Fetch the latest `SystemVersion` (highest semver).
    async fn latest(&self) -> StorageResult<Option<SystemVersion>>;
    /// Insert a `SystemVersion`.
    async fn insert(&self, v: &SystemVersion) -> StorageResult<()>;
    /// Update a `SystemVersion`.
    async fn update(&self, v: &SystemVersion) -> StorageResult<()>;
}

fn _assert_system_version_object_safe() {
    fn _f(_: Box<dyn SystemVersionRepository>) {}
}

// === SystemVersionRepository section end ===

// =============================================================================
// === VersionHistoryRepository section begin (owner: B) ===
// =============================================================================

/// Repository port for [`VersionHistory`](crate::aggregate::VersionHistory). Global.
#[async_trait]
pub trait VersionHistoryRepository: Send + Sync {
    /// Fetch a `VersionHistory` row by id.
    async fn get(
        &self,
        id: crate::value_objects::VersionHistoryId,
    ) -> StorageResult<Option<VersionHistory>>;
    /// List all `VersionHistory` rows.
    async fn list(&self) -> StorageResult<Vec<VersionHistory>>;
    /// List `VersionHistory` rows for a given `version`.
    async fn list_for_version(
        &self,
        version: &HistoryVersion,
    ) -> StorageResult<Vec<VersionHistory>>;
    /// Insert a `VersionHistory` row.
    async fn insert(&self, v: &VersionHistory) -> StorageResult<()>;
}

fn _assert_version_history_object_safe() {
    fn _f(_: Box<dyn VersionHistoryRepository>) {}
}

// === VersionHistoryRepository section end ===

// =============================================================================
// === UserLogRepository section begin (owner: B) ===
// =============================================================================

/// Repository port for [`UserLog`](crate::aggregate::UserLog).
#[async_trait]
pub trait UserLogRepository: Send + Sync {
    /// Fetch a `UserLog` row by id.
    async fn get(&self, id: crate::value_objects::UserLogId) -> StorageResult<Option<UserLog>>;
    /// List all `UserLog` rows for a school.
    async fn list(&self, school: SchoolId) -> StorageResult<Vec<UserLog>>;
    /// List `UserLog` rows for a user.
    async fn list_for_user(&self, user: educore_core::ids::UserId) -> StorageResult<Vec<UserLog>>;
    /// List `UserLog` rows for a role.
    async fn list_for_role(&self, role: RoleId) -> StorageResult<Vec<UserLog>>;
    /// List `UserLog` rows for a school filtered by outcome.
    async fn list_for_outcome(
        &self,
        school: SchoolId,
        outcome: LoginOutcome,
    ) -> StorageResult<Vec<UserLog>>;
    /// List `UserLog` rows for a school in a date range.
    async fn list_for_date_range(
        &self,
        school: SchoolId,
        from: Timestamp,
        to: Timestamp,
    ) -> StorageResult<Vec<UserLog>>;
    /// Insert a `UserLog` row.
    async fn insert(&self, l: &UserLog) -> StorageResult<()>;
    /// Purge `UserLog` rows older than `cutoff`. Returns the count purged.
    async fn purge_older_than(&self, school: SchoolId, cutoff: Timestamp) -> StorageResult<u64>;
}

fn _assert_user_log_object_safe() {
    fn _f(_: Box<dyn UserLogRepository>) {}
}

// === UserLogRepository section end ===

// =============================================================================
// === MaintenanceSettingRepository section begin (owner: B) ===
// =============================================================================

/// Repository port for [`MaintenanceSetting`](crate::aggregate::MaintenanceSetting).
#[async_trait]
pub trait MaintenanceSettingRepository: Send + Sync {
    /// Fetch the per-school `MaintenanceSetting` singleton.
    async fn get(&self, school: SchoolId) -> StorageResult<Option<MaintenanceSetting>>;
    /// Insert the singleton.
    async fn insert(&self, m: &MaintenanceSetting) -> StorageResult<()>;
    /// Update the singleton.
    async fn update(&self, m: &MaintenanceSetting) -> StorageResult<()>;
}

fn _assert_maintenance_setting_object_safe() {
    fn _f(_: Box<dyn MaintenanceSettingRepository>) {}
}

// === MaintenanceSettingRepository section end ===

// =============================================================================
// === SidebarRepository section begin (owner: B) ===
// =============================================================================

/// Repository port for [`Sidebar`](crate::aggregate::Sidebar).
#[async_trait]
pub trait SidebarRepository: Send + Sync {
    /// Fetch a `Sidebar` entry by id.
    async fn get(&self, id: crate::value_objects::SidebarId) -> StorageResult<Option<Sidebar>>;
    /// List `Sidebar` entries for a school.
    async fn list(&self, school: SchoolId) -> StorageResult<Vec<Sidebar>>;
    /// List `Sidebar` entries for a role.
    async fn list_for_role(&self, role: RoleId) -> StorageResult<Vec<Sidebar>>;
    /// List `Sidebar` entries for a permission.
    async fn list_for_permission(&self, permission_id: PermissionId)
        -> StorageResult<Vec<Sidebar>>;
    /// List `Sidebar` entries for a school within a section.
    async fn list_for_section(
        &self,
        school: SchoolId,
        section_id: SidebarSectionId,
    ) -> StorageResult<Vec<Sidebar>>;
    /// Insert a `Sidebar` entry.
    async fn insert(&self, s: &Sidebar) -> StorageResult<()>;
    /// Update a `Sidebar` entry.
    async fn update(&self, s: &Sidebar) -> StorageResult<()>;
    /// Delete a `Sidebar` entry.
    async fn delete(&self, id: crate::value_objects::SidebarId) -> StorageResult<()>;
    /// Reorder `Sidebar` entries within a role by the supplied
    /// `(sidebar_id -> new_position)` map. Returns the count updated.
    async fn reorder_for_role(
        &self,
        role: RoleId,
        new_positions: &std::collections::BTreeMap<
            crate::value_objects::SidebarId,
            SidebarPosition,
        >,
    ) -> StorageResult<u32>;
}

fn _assert_sidebar_object_safe() {
    fn _f(_: Box<dyn SidebarRepository>) {}
}

// === SidebarRepository section end ===

// =============================================================================
// Port-driven repository traits (documented-as-not-owned).
// The operations domain documents these for completeness. The engine
// does not own their lifecycle; consumer auth / migration adapters
// implement them.
// =============================================================================

/// Repository port for `oauth_access_tokens`. Port-driven.
#[allow(dead_code)]
#[async_trait]
pub trait OAuthAccessTokenRepository: Send + Sync {
    /// Fetch a token by id.
    async fn get(&self, id: &str) -> StorageResult<Option<OAuthAccessToken>>;
    /// List tokens for a user.
    async fn list_for_user(&self, user_id: Uuid) -> StorageResult<Vec<OAuthAccessToken>>;
    /// Insert a token.
    async fn insert(&self, t: &OAuthAccessToken) -> StorageResult<()>;
    /// Revoke a token by id.
    async fn revoke(&self, id: &str) -> StorageResult<()>;
    /// Purge expired tokens. Returns the count purged.
    async fn purge_expired(&self, before: Timestamp) -> StorageResult<u64>;
}

/// Repository port for `oauth_clients`. Port-driven.
#[allow(dead_code)]
#[async_trait]
pub trait OAuthClientRepository: Send + Sync {
    /// Fetch a client by id.
    async fn get(&self, id: &str) -> StorageResult<Option<OAuthClient>>;
    /// List all clients.
    async fn list(&self) -> StorageResult<Vec<OAuthClient>>;
    /// Insert a client.
    async fn insert(&self, c: &OAuthClient) -> StorageResult<()>;
    /// Revoke a client by id.
    async fn revoke(&self, id: &str) -> StorageResult<()>;
}

/// Repository port for `password_resets`. Port-driven.
#[allow(dead_code)]
#[async_trait]
pub trait PasswordResetRepository: Send + Sync {
    /// Fetch a password-reset row by email.
    async fn get_by_email(&self, email: &str) -> StorageResult<Option<PasswordReset>>;
    /// Insert a password-reset row.
    async fn insert(&self, r: &PasswordReset) -> StorageResult<()>;
    /// Delete a password-reset row by email.
    async fn delete(&self, email: &str) -> StorageResult<()>;
    /// Purge old password-reset rows. Returns the count purged.
    async fn purge_older_than(&self, before: Timestamp) -> StorageResult<u64>;
}

/// Repository port for `migrations`. Port-driven.
#[allow(dead_code)]
#[async_trait]
pub trait MigrationRepository: Send + Sync {
    /// List all migration rows.
    async fn list(&self) -> StorageResult<Vec<Migration>>;
    /// Fetch a migration row by name.
    async fn get_by_name(&self, name: &str) -> StorageResult<Option<Migration>>;
    /// Insert a migration row at the given batch.
    async fn insert(&self, m: &Migration, batch: i32) -> StorageResult<()>;
}

fn _assert_oauth_access_token_object_safe() {
    fn _f(_: Box<dyn OAuthAccessTokenRepository>) {}
}

fn _assert_oauth_client_object_safe() {
    fn _f(_: Box<dyn OAuthClientRepository>) {}
}

fn _assert_password_reset_object_safe() {
    fn _f(_: Box<dyn PasswordResetRepository>) {}
}

fn _assert_migration_object_safe() {
    fn _f(_: Box<dyn MigrationRepository>) {}
}

// =============================================================================
// Port-driven record types (documented as opaque to operations).
// =============================================================================

/// An `oauth_access_tokens` row. Opaque to the operations domain.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OAuthAccessToken {
    pub id: String,
    pub user_id: Uuid,
    pub client_id: String,
    pub scopes: String,
    pub revoked: bool,
    pub expires_at: Option<Timestamp>,
    pub created_at: Timestamp,
}

/// An `oauth_clients` row. Opaque to the operations domain.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OAuthClient {
    pub id: String,
    pub name: String,
    pub secret_hash: String,
    pub redirect_uri: String,
    pub provider: Option<String>,
    pub revoked: bool,
    pub created_at: Timestamp,
}

/// A `password_resets` row. Opaque to the operations domain.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PasswordReset {
    pub email: String,
    pub token_hash: String,
    pub created_at: Timestamp,
    pub expires_at: Option<Timestamp>,
}

/// A `migrations` row. Opaque to the operations domain.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Migration {
    pub migration: String,
    pub batch: i32,
}
