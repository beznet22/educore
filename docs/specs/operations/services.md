# Operations Domain — Services

Domain services encapsulate business logic that does not fit cleanly
in a single aggregate. They are stateless, sync, and pure (no I/O).

## BackupService

```rust
pub struct BackupService;

impl BackupService {
    pub fn is_database(backup: &Backup) -> bool { ... }
    pub fn is_file(backup: &Backup) -> bool { ... }
    pub fn is_image(backup: &Backup) -> bool { ... }
    pub fn can_delete(backup: &Backup, restore_in_progress: bool) -> Result<(), ConflictError> { ... }
    pub fn can_restore(backup: &Backup, school_active_restore_count: u32) -> Result<(), ConflictError> { ... }
    pub fn retention_cutoff(backups: &[Backup], keep: u32) -> Vec<BackupId> { ... }
}
```

## JobService

```rust
pub struct JobService;

impl JobService {
    pub fn is_reserved(job: &Job, now: Timestamp) -> bool { ... }
    pub fn is_available(job: &Job, now: Timestamp) -> bool { ... }
    pub fn can_retry(job: &Job, max_attempts: u8) -> bool { ... }
    pub fn next_backoff(attempts: JobAttempts) -> u32 { ... }
    pub fn validate_payload(payload: &JobPayload) -> Result<(), ValidationError> { ... }
    pub fn purge_completed(jobs: &mut Vec<Job>) -> Vec<Job> { ... }
}
```

## FailedJobService

```rust
pub struct FailedJobService;

impl FailedJobService {
    pub fn can_retry(failed: &FailedJob, max_age_days: u32) -> bool { ... }
    pub fn purge_old(failures: &mut Vec<FailedJob>, cutoff: Timestamp) -> Vec<FailedJob> { ... }
    pub fn extract_exception_type(exception: &str) -> Option<&'static str> { ... }
}
```

## SystemVersionService

```rust
pub struct SystemVersionService;

impl SystemVersionService {
    pub fn is_newer(a: &VersionName, b: &VersionName) -> bool { ... }
    pub fn is_compatible(client: &VersionName, server: &VersionName) -> bool { ... }
    pub fn latest(versions: &[SystemVersion]) -> Option<&SystemVersion> { ... }
}
```

## VersionHistoryService

```rust
pub struct VersionHistoryService;

impl VersionHistoryService {
    pub fn ordered(records: &[VersionHistory]) -> Vec<&VersionHistory> { ... }
    pub fn since(records: &[VersionHistory], version: &HistoryVersion) -> Vec<&VersionHistory> { ... }
}
```

## UserLogService

```rust
pub struct UserLogService;

impl UserLogService {
    pub fn partition(log: &[UserLog], partition: AuditPartition) -> Vec<&UserLog> { ... }
    pub fn retention_cutoff(now: Timestamp, retention_days: u32) -> Timestamp { ... }
    pub fn is_suspicious(log: &UserLog, prior: &[UserLog]) -> bool { ... }
    pub fn distinct_ips(log: &[UserLog]) -> BTreeSet<IpAddress> { ... }
    pub fn distinct_user_agents(log: &[UserLog]) -> BTreeSet<UserAgent> { ... }
}
```

`UserLogService::is_suspicious` returns true when the current
login's IP or user agent differs from the user's typical pattern
(port-driven anomaly detection).

## MaintenanceService

```rust
pub struct MaintenanceService;

impl MaintenanceService {
    pub fn is_enabled(setting: &MaintenanceSetting) -> bool { ... }
    pub fn applies_to_role(setting: &MaintenanceSetting, role: &Role) -> bool { ... }
    pub fn validate_message(setting: &MaintenanceSetting) -> Result<(), ValidationError> { ... }
    pub fn default_setting(school: SchoolId) -> MaintenanceSetting { ... }
}
```

## SidebarService

```rust
pub struct SidebarService;

impl SidebarService {
    pub fn tree(entries: &[Sidebar], role: RoleId) -> Vec<SidebarNode> { ... }
    pub fn children(parent: SidebarId, entries: &[Sidebar]) -> Vec<&Sidebar> { ... }
    pub fn reorder(entries: &mut [Sidebar], new_positions: &BTreeMap<SidebarId, SidebarPosition>) -> Result<(), ValidationError> { ... }
    pub fn visible(entries: &[Sidebar], role: RoleId) -> Vec<&Sidebar> { ... }
}
```

## AuditService

```rust
pub struct AuditService;

impl AuditService {
    pub fn filter_by_user(log: &[UserLog], user: UserId) -> Vec<&UserLog> { ... }
    pub fn filter_by_outcome(log: &[UserLog], outcome: LoginOutcome) -> Vec<&UserLog> { ... }
    pub fn filter_by_date_range(log: &[UserLog], from: Timestamp, to: Timestamp) -> Vec<&UserLog> { ... }
    pub fn export(log: &[UserLog]) -> AuditExport { ... }
}
```

## Policy: OneRestoreInProgress

```rust
pub struct OneRestoreInProgress;

impl Policy<RestoreBackupCommand> for OneRestoreInProgress {
    type Outcome = Allow | Deny { reason: &'static str };
    fn check(&self, ctx: &Context, cmd: &RestoreBackupCommand) -> Outcome { ... }
}
```

## Policy: MaintenanceLockout

```rust
pub struct MaintenanceLockout;

impl Policy<RegisterUserCommand> for MaintenanceLockout {
    type Outcome = Allow | Deny { reason: &'static str };
    fn check(&self, ctx: &Context, cmd: &RegisterUserCommand) -> Outcome { ... }
}
```

Rejects registrations (or logins) for non-admin users while
maintenance mode is enabled.

## Policy: SelfRevocationGuard (Operations)

```rust
pub struct DisableMaintenanceGuard;

impl Policy<DisableMaintenanceCommand> for DisableMaintenanceGuard {
    type Outcome = Allow | Deny { reason: &'static str };
    fn check(&self, ctx: &Context, cmd: &DisableMaintenanceCommand) -> Outcome { ... }
}
```

Rejects a `DisableMaintenance` from a non-SuperAdmin.

## Specification: ActiveBackups

```rust
pub struct ActiveBackups;

impl Specification<Backup> for ActiveBackups {
    fn is_satisfied_by(&self, b: &Backup) -> bool { ... }
}
```

## Specification: DatabaseBackups

```rust
pub struct DatabaseBackups;

impl Specification<Backup> for DatabaseBackups {
    fn is_satisfied_by(&self, b: &Backup) -> bool { ... }
}
```

Composed with `And`, `Or`, `Not` for queries.

## Specification: SuccessfulLogins

```rust
pub struct SuccessfulLogins;

impl Specification<UserLog> for SuccessfulLogins {
    fn is_satisfied_by(&self, l: &UserLog) -> bool { ... }
}
```

## Specification: FailedLogins

```rust
pub struct FailedLogins;

impl Specification<UserLog> for FailedLogins {
    fn is_satisfied_by(&self, l: &UserLog) -> bool { ... }
}
```

## Cross-Domain Coordinator

The operations domain publishes events; other domains subscribe.
The operations domain does not call other domains' services
directly. Maintenance mode, backup, and audit are platform-level
concerns; coordination happens through events.
