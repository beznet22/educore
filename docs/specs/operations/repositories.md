# Operations Domain — Repositories

Repositories are ports (Rust traits). Adapters implement them. The
default adapter targets PostgreSQL; an SQLite adapter is provided for
embedded deployments.

## BackupRepository

```rust
#[async_trait]
pub trait BackupRepository: Send + Sync {
    async fn get(&self, id: BackupId) -> Result<Option<Backup>>;
    async fn get_by_file_name(&self, school: SchoolId, file_name: &BackupFileName) -> Result<Option<Backup>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<Backup>>;
    async fn list_by_type(&self, school: SchoolId, file_type: BackupFileType) -> Result<Vec<Backup>>;
    async fn list_active(&self, school: SchoolId) -> Result<Vec<Backup>>;
    async fn insert(&self, b: &Backup) -> Result<()>;
    async fn update(&self, b: &Backup) -> Result<()>;
    async fn delete(&self, id: BackupId) -> Result<()>;
    async fn restore_in_progress_count(&self, school: SchoolId) -> Result<u32>;
}
```

## JobRepository

```rust
#[async_trait]
pub trait JobRepository: Send + Sync {
    async fn get(&self, id: JobId) -> Result<Option<Job>>;
    async fn list(&self) -> Result<Vec<Job>>;
    async fn list_for_queue(&self, queue: &JobQueue) -> Result<Vec<Job>>;
    async fn list_available(&self, now: Timestamp, limit: u32) -> Result<Vec<Job>>;
    async fn list_reserved(&self, before: Timestamp) -> Result<Vec<Job>>;
    async fn insert(&self, j: &Job) -> Result<()>;
    async fn update(&self, j: &Job) -> Result<()>;
    async fn delete(&self, id: JobId) -> Result<()>;
    async fn purge_completed(&self, before: Timestamp) -> Result<u64>;
}
```

## FailedJobRepository

```rust
#[async_trait]
pub trait FailedJobRepository: Send + Sync {
    async fn get(&self, id: FailedJobId) -> Result<Option<FailedJob>>;
    async fn get_by_uuid(&self, uuid: &FailedJobUuid) -> Result<Option<FailedJob>>;
    async fn list(&self) -> Result<Vec<FailedJob>>;
    async fn list_for_queue(&self, queue: &FailedJobQueue) -> Result<Vec<FailedJob>>;
    async fn list_for_connection(&self, connection: &FailedJobConnection) -> Result<Vec<FailedJob>>;
    async fn insert(&self, j: &FailedJob) -> Result<()>;
    async fn delete(&self, id: FailedJobId) -> Result<()>;
    async fn purge_older_than(&self, cutoff: Timestamp) -> Result<u64>;
}
```

## SystemVersionRepository

```rust
#[async_trait]
pub trait SystemVersionRepository: Send + Sync {
    async fn get(&self, id: SystemVersionId) -> Result<Option<SystemVersion>>;
    async fn get_by_name(&self, version_name: &VersionName) -> Result<Option<SystemVersion>>;
    async fn list(&self) -> Result<Vec<SystemVersion>>;
    async fn latest(&self) -> Result<Option<SystemVersion>>;
    async fn insert(&self, v: &SystemVersion) -> Result<()>;
    async fn update(&self, v: &SystemVersion) -> Result<()>;
}
```

## VersionHistoryRepository

```rust
#[async_trait]
pub trait VersionHistoryRepository: Send + Sync {
    async fn get(&self, id: VersionHistoryId) -> Result<Option<VersionHistory>>;
    async fn list(&self) -> Result<Vec<VersionHistory>>;
    async fn list_for_version(&self, version: &HistoryVersion) -> Result<Vec<VersionHistory>>;
    async fn insert(&self, v: &VersionHistory) -> Result<()>;
}
```

## UserLogRepository

```rust
#[async_trait]
pub trait UserLogRepository: Send + Sync {
    async fn get(&self, id: UserLogId) -> Result<Option<UserLog>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<UserLog>>;
    async fn list_for_user(&self, user: UserId) -> Result<Vec<UserLog>>;
    async fn list_for_role(&self, role: RoleId) -> Result<Vec<UserLog>>;
    async fn list_for_outcome(&self, school: SchoolId, outcome: LoginOutcome) -> Result<Vec<UserLog>>;
    async fn list_for_date_range(&self, school: SchoolId, from: Timestamp, to: Timestamp) -> Result<Vec<UserLog>>;
    async fn insert(&self, l: &UserLog) -> Result<()>;
    async fn purge_older_than(&self, school: SchoolId, cutoff: Timestamp) -> Result<u64>;
}
```

## MaintenanceSettingRepository

```rust
#[async_trait]
pub trait MaintenanceSettingRepository: Send + Sync {
    async fn get(&self, school: SchoolId) -> Result<Option<MaintenanceSetting>>;
    async fn insert(&self, m: &MaintenanceSetting) -> Result<()>;
    async fn update(&self, m: &MaintenanceSetting) -> Result<()>;
}
```

## SidebarRepository

```rust
#[async_trait]
pub trait SidebarRepository: Send + Sync {
    async fn get(&self, id: SidebarId) -> Result<Option<Sidebar>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<Sidebar>>;
    async fn list_for_role(&self, role: RoleId) -> Result<Vec<Sidebar>>;
    async fn list_for_permission(&self, permission_id: PermissionId) -> Result<Vec<Sidebar>>;
    async fn list_for_section(&self, school: SchoolId, section_id: SidebarSectionId) -> Result<Vec<Sidebar>>;
    async fn insert(&self, s: &Sidebar) -> Result<()>;
    async fn update(&self, s: &Sidebar) -> Result<()>;
    async fn delete(&self, id: SidebarId) -> Result<()>;
    async fn reorder_for_role(&self, role: RoleId, new_positions: &BTreeMap<SidebarId, SidebarPosition>) -> Result<()>;
}
```

## OAuth Repositories (Port-Driven, Documented)

```rust
#[async_trait]
pub trait OAuthAccessTokenRepository: Send + Sync {
    async fn get(&self, id: &str) -> Result<Option<OAuthAccessToken>>;
    async fn list_for_user(&self, user_id: Uuid) -> Result<Vec<OAuthAccessToken>>;
    async fn insert(&self, t: &OAuthAccessToken) -> Result<()>;
    async fn revoke(&self, id: &str) -> Result<()>;
    async fn purge_expired(&self, before: Timestamp) -> Result<u64>;
}

#[async_trait]
pub trait OAuthClientRepository: Send + Sync {
    async fn get(&self, id: &str) -> Result<Option<OAuthClient>>;
    async fn list(&self) -> Result<Vec<OAuthClient>>;
    async fn insert(&self, c: &OAuthClient) -> Result<()>;
    async fn revoke(&self, id: &str) -> Result<()>;
}
```

The other OAuth repositories follow the same pattern.

## PasswordResetRepository (Port-Driven, Documented)

```rust
#[async_trait]
pub trait PasswordResetRepository: Send + Sync {
    async fn get(&self, email: &PasswordResetEmail) -> Result<Option<PasswordReset>>;
    async fn insert(&self, r: &PasswordReset) -> Result<()>;
    async fn delete(&self, email: &PasswordResetEmail) -> Result<()>;
    async fn purge_older_than(&self, before: Timestamp) -> Result<u64>;
}
```

## MigrationRepository (Port-Driven, Documented)

```rust
#[async_trait]
pub trait MigrationRepository: Send + Sync {
    async fn list(&self) -> Result<Vec<Migration>>;
    async fn get(&self, name: &MigrationName) -> Result<Option<Migration>>;
    async fn insert(&self, m: &Migration, batch: MigrationBatch) -> Result<()>;
}
```

## Indexes (recommended)

```sql
CREATE UNIQUE INDEX ux_backups_school_id_file_name ON backups (school_id, file_name);
CREATE INDEX ix_backups_school_id_file_type ON backups (school_id, file_type);
CREATE INDEX ix_backups_school_id_active_status ON backups (school_id, active_status);
CREATE INDEX ix_backups_school_id_academic_id ON backups (school_id, academic_id);
CREATE INDEX ix_jobs_queue ON jobs (queue);
CREATE INDEX ix_jobs_available_at ON jobs (available_at);
CREATE INDEX ix_jobs_reserved_at ON jobs (reserved_at);
CREATE UNIQUE INDEX ux_failed_jobs_uuid ON failed_jobs (uuid);
CREATE INDEX ix_failed_jobs_queue ON failed_jobs (queue);
CREATE INDEX ix_failed_jobs_connection ON failed_jobs (connection);
CREATE INDEX ix_failed_jobs_failed_at ON failed_jobs (failed_at);
CREATE UNIQUE INDEX ux_system_versions_version_name ON system_versions (version_name);
CREATE INDEX ix_version_histories_version ON operations_version_histories (version);
CREATE INDEX ix_user_logs_school_id_user_id ON user_logs (school_id, user_id);
CREATE INDEX ix_user_logs_school_id_role_id ON user_logs (school_id, role_id);
CREATE INDEX ix_user_logs_school_id_outcome ON user_logs (school_id, outcome);
CREATE INDEX ix_user_logs_school_id_logged_at ON user_logs (school_id, logged_at);
CREATE INDEX ix_user_logs_school_id_ip_address ON user_logs (school_id, ip_address);
CREATE UNIQUE INDEX ux_maintenance_settings_school_id ON operations_maintenance_settings (school_id);
CREATE INDEX ix_sidebars_school_id_role_id ON sidebars (school_id, role_id);
CREATE INDEX ix_sidebars_school_id_permission_id ON sidebars (school_id, permission_id);
CREATE INDEX ix_sidebars_school_id_section_id ON sidebars (school_id, section_id);
CREATE INDEX ix_sidebars_school_id_parent ON sidebars (school_id, parent);
CREATE UNIQUE INDEX ux_oauth_access_tokens_id ON oauth_access_tokens (id);
CREATE INDEX ix_oauth_access_tokens_user_id ON oauth_access_tokens (user_id);
CREATE INDEX ix_oauth_refresh_tokens_access_token_id ON oauth_refresh_tokens (access_token_id);
CREATE UNIQUE INDEX ux_oauth_clients_id ON oauth_clients (id);
CREATE INDEX ix_oauth_clients_user_id ON oauth_clients (user_id);
CREATE INDEX ix_password_resets_email ON password_resets (email);
CREATE UNIQUE INDEX ux_migrations_migration ON migrations (migration);
```

The `school_id` predicate is mandatory for tenant isolation on
per-school tables. Job, system-version, version-history, OAuth,
and migration tables are global.
