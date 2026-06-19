//! # educore-operations service structs
//!
//! Per `docs/specs/operations/services.md`. 9 service structs +
//! 3 policies + 4 specifications. Includes the headline
//! `JobService::next_backoff` exponential backoff calculator.

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

use std::collections::{BTreeMap, BTreeSet};

use educore_core::ids::{SchoolId, UserId};
use educore_core::value_objects::Timestamp;

use crate::aggregate::{
    Backup, FailedJob, Job, MaintenanceSetting, Sidebar, SystemVersion, UserLog, VersionHistory,
};
use crate::value_objects::{
    BackupId, IpAddress, JobAttempts, JobStatus, LoginOutcome, MaintenanceApplicableFor,
    MaintenanceSubTitle, MaintenanceTitle, RoleId, SidebarPosition, UserAgent, VersionName,
};

// =============================================================================
// === BackupService section begin (owner: B) ===
// =============================================================================

/// Pure helpers for the [`Backup`](crate::aggregate::Backup) aggregate.
pub struct BackupService;

impl BackupService {
    /// Returns true if the backup's `file_type` is `Database`.
    #[must_use]
    pub fn is_database(backup: &Backup) -> bool {
        matches!(
            backup.file_type,
            crate::value_objects::BackupFileType::Database
        )
    }

    /// Returns true if the backup's `file_type` is `File`.
    #[must_use]
    pub fn is_file(backup: &Backup) -> bool {
        matches!(backup.file_type, crate::value_objects::BackupFileType::File)
    }

    /// Returns true if the backup's `file_type` is `Image`.
    #[must_use]
    pub fn is_image(backup: &Backup) -> bool {
        matches!(
            backup.file_type,
            crate::value_objects::BackupFileType::Image
        )
    }

    /// Returns Ok(()) if the backup can be deleted.
    pub fn can_delete(backup: &Backup, restore_in_progress: bool) -> Result<(), String> {
        if backup.restore_in_progress || restore_in_progress {
            return Err("cannot delete backup while restore is in progress".to_owned());
        }
        Ok(())
    }

    /// Returns Ok(()) if the backup can be restored (no concurrent
    /// restore in progress).
    pub fn can_restore(_backup: &Backup, school_active_restore_count: u32) -> Result<(), String> {
        if school_active_restore_count > 0 {
            return Err("another restore is already in progress".to_owned());
        }
        Ok(())
    }

    /// Returns the ids of backups that fall outside the retention
    /// window (keep N most recent). Stable order by `created_at`.
    #[must_use]
    pub fn retention_cutoff(backups: &[Backup], keep: u32) -> Vec<BackupId> {
        if backups.len() <= keep as usize {
            return Vec::new();
        }
        // Order by `created_at` ascending (oldest first); the tail is "keep".
        let mut sorted: Vec<&Backup> = backups.iter().collect();
        sorted.sort_by_key(|b| b.created_at);
        let drop_count = sorted.len().saturating_sub(keep as usize);
        sorted.iter().take(drop_count).map(|b| b.id).collect()
    }
}

// === BackupService section end ===

// =============================================================================
// === JobService section begin (owner: B) ===
// =============================================================================

/// Pure helpers for the [`Job`](crate::aggregate::Job) aggregate.
pub struct JobService;

impl JobService {
    /// Returns true if the job is currently reserved.
    #[must_use]
    pub fn is_reserved(job: &Job) -> bool {
        matches!(job.status, JobStatus::Reserved) && job.reserved_at.is_some()
    }

    /// Returns true if the job is available to run (`available_at <= now`
    /// and status is `Pending`).
    #[must_use]
    pub fn is_available(job: &Job, now: Timestamp) -> bool {
        matches!(job.status, JobStatus::Pending) && job.available_at <= now
    }

    /// Returns true if the job can be retried.
    #[must_use]
    pub fn can_retry(job: &Job, max_attempts: u8) -> bool {
        job.attempts.0 < max_attempts
    }

    /// Exponential backoff in seconds for the next attempt.
    ///
    /// The schedule is `1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024,
    /// 2048, 4096, ...` (capped at `u32::MAX` to avoid overflow).
    /// This is the canonical operations-domain backoff and the
    /// `JobService::next_backoff` test exercises it.
    #[must_use]
    pub fn next_backoff(attempts: JobAttempts) -> u32 {
        let n = u32::from(attempts.0);
        if n >= 32 {
            // 2^32 overflows u32. Cap at u32::MAX.
            return u32::MAX;
        }
        1u32.checked_shl(n).unwrap_or(u32::MAX)
    }

    /// Returns Ok(()) if the payload is non-empty.
    pub fn validate_payload(payload: &crate::value_objects::JobPayload) -> Result<(), String> {
        if payload.as_str().is_empty() {
            return Err("job_payload must not be empty".to_owned());
        }
        Ok(())
    }

    /// Removes (and returns) the completed jobs from the input vec.
    pub fn purge_completed(jobs: &mut Vec<Job>) -> Vec<Job> {
        let (done, pending): (Vec<Job>, Vec<Job>) = jobs
            .drain(..)
            .partition(|j| matches!(j.status, JobStatus::Completed));
        *jobs = pending;
        done
    }
}

// === JobService section end ===

// =============================================================================
// === FailedJobService section begin (owner: B) ===
// =============================================================================

/// Pure helpers for the [`FailedJob`](crate::aggregate::FailedJob) aggregate.
pub struct FailedJobService;

impl FailedJobService {
    /// Returns true if a `FailedJob` is still retryable given a
    /// maximum age in days.
    #[must_use]
    pub fn can_retry(failed: &FailedJob, max_age_days: u32) -> bool {
        let now = Timestamp::now();
        let age_seconds = (now.as_datetime() - failed.failed_at.as_datetime()).num_seconds();
        let max_seconds = i64::from(max_age_days).saturating_mul(86_400);
        age_seconds <= max_seconds
    }

    /// Removes (and returns) `FailedJob` rows older than `cutoff`.
    pub fn purge_old(failures: &mut Vec<FailedJob>, cutoff: Timestamp) -> Vec<FailedJob> {
        let (old, kept): (Vec<FailedJob>, Vec<FailedJob>) =
            failures.drain(..).partition(|f| f.failed_at < cutoff);
        *failures = kept;
        old
    }

    /// Extracts the exception type (the leading `Type:` segment of
    /// the exception string, or the first line).
    #[must_use]
    pub fn extract_exception_type(exception: &str) -> Option<&str> {
        let first_line = exception.lines().next()?;
        let stripped = first_line.trim();
        if stripped.is_empty() {
            return None;
        }
        // Look for "Type: Name" or "Name: message".
        let mut parts = stripped.splitn(2, ':');
        let head = parts.next()?.trim();
        let tail = parts.next().map(str::trim);
        // If `head` contains whitespace or is a sentence, fall back
        // to the first whitespace-delimited token.
        let token = if head.chars().any(char::is_whitespace) {
            stripped.split_whitespace().next().unwrap_or(head)
        } else {
            head
        };
        let _ = tail;
        Some(token)
    }
}

// === FailedJobService section end ===

// =============================================================================
// === SystemVersionService section begin (owner: B) ===
// =============================================================================

/// Pure helpers for the [`SystemVersion`](crate::aggregate::SystemVersion) aggregate.
pub struct SystemVersionService;

impl SystemVersionService {
    /// Returns true if `a` is strictly newer than `b` (semver compare).
    #[must_use]
    pub fn is_newer(a: &VersionName, b: &VersionName) -> bool {
        let av = parse_semver(a);
        let bv = parse_semver(b);
        av > bv
    }

    /// Returns true if `client` is compatible with `server`
    /// (same major version).
    #[must_use]
    pub fn is_compatible(client: &VersionName, server: &VersionName) -> bool {
        let c = parse_semver(client);
        let s = parse_semver(server);
        c.0 == s.0 && c.0 != 0
    }

    /// Returns the highest-version `SystemVersion` in the list.
    #[must_use]
    pub fn latest(versions: &[SystemVersion]) -> Option<&SystemVersion> {
        versions
            .iter()
            .max_by_key(|v| parse_semver(&v.version_name))
    }
}

fn parse_semver(v: &VersionName) -> (u32, u32, u32) {
    let s = v.as_str();
    let mut parts = s.split('.');
    let major = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let minor = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let patch = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    (major, minor, patch)
}

// === SystemVersionService section end ===

// =============================================================================
// === VersionHistoryService section begin (owner: B) ===
// =============================================================================

/// Pure helpers for the [`VersionHistory`](crate::aggregate::VersionHistory) aggregate.
pub struct VersionHistoryService;

impl VersionHistoryService {
    /// Returns the records ordered by `release_date` ascending
    /// (lexicographic; consumer-format strings sort well when the
    /// consumer uses `YYYY-MM-DD`).
    #[must_use]
    pub fn ordered(records: &[VersionHistory]) -> Vec<&VersionHistory> {
        let mut sorted: Vec<&VersionHistory> = records.iter().collect();
        sorted.sort_by(|a, b| a.release_date.as_str().cmp(b.release_date.as_str()));
        sorted
    }

    /// Returns the records whose version is at least `since`.
    #[must_use]
    pub fn since<'a>(records: &'a [VersionHistory], since: &str) -> Vec<&'a VersionHistory> {
        records
            .iter()
            .filter(|r| r.version.as_str() >= since)
            .collect()
    }
}

// === VersionHistoryService section end ===

// =============================================================================
// === UserLogService section begin (owner: B) ===
// =============================================================================

/// Pure helpers for the [`UserLog`](crate::aggregate::UserLog) aggregate.
pub struct UserLogService;

impl UserLogService {
    /// Returns the rows in the supplied `AuditPartition` (matching
    /// `partition_id == label` for the simple case).
    #[must_use]
    pub fn partition<'a>(log: &'a [UserLog], partition_label: &str) -> Vec<&'a UserLog> {
        log.iter()
            .filter(|l| {
                // Use the `correlation_id` as a stable partition key
                // surrogate when the consumer has not provided one.
                l.correlation_id.to_string() == partition_label
            })
            .collect()
    }

    /// Returns the cutoff timestamp for a retention window in days.
    #[must_use]
    pub fn retention_cutoff(now: Timestamp, retention_days: u32) -> Timestamp {
        let seconds = i64::from(retention_days).saturating_mul(86_400);
        let dt = now.as_datetime() - chrono::Duration::seconds(seconds);
        Timestamp::from_datetime(dt)
    }

    /// Returns true if the current login looks suspicious vs the
    /// user's prior logins (port-driven anomaly detection).
    #[must_use]
    pub fn is_suspicious(log: &UserLog, prior: &[UserLog]) -> bool {
        let prior_for_user: Vec<&UserLog> =
            prior.iter().filter(|l| l.user_id == log.user_id).collect();
        if prior_for_user.is_empty() {
            // First login — not suspicious on its own.
            return false;
        }
        let same_ip = prior_for_user
            .iter()
            .any(|l| l.ip_address.as_str() == log.ip_address.as_str());
        let same_ua = prior_for_user
            .iter()
            .any(|l| l.user_agent.as_str() == log.user_agent.as_str());
        // Suspicious if neither IP nor user agent match any prior login.
        !same_ip && !same_ua
    }

    /// Returns the distinct IP addresses seen in the log slice.
    #[must_use]
    pub fn distinct_ips(log: &[UserLog]) -> BTreeSet<IpAddress> {
        log.iter().map(|l| l.ip_address.clone()).collect()
    }

    /// Returns the distinct user agents seen in the log slice.
    #[must_use]
    pub fn distinct_user_agents(log: &[UserLog]) -> BTreeSet<UserAgent> {
        log.iter().map(|l| l.user_agent.clone()).collect()
    }
}

// === UserLogService section end ===

// =============================================================================
// === MaintenanceService section begin (owner: B) ===
// =============================================================================

/// Pure helpers for the [`MaintenanceSetting`](crate::aggregate::MaintenanceSetting) aggregate.
pub struct MaintenanceService;

impl MaintenanceService {
    /// Returns true if the school's maintenance mode is enabled.
    #[must_use]
    pub fn is_enabled(setting: &MaintenanceSetting) -> bool {
        setting.maintenance_mode
    }

    /// Returns true if the maintenance setting applies to the given role.
    /// Uses the free-form `applicable_for` string. `"all"` matches
    /// everything; comma-separated role names match exactly.
    #[must_use]
    pub fn applies_to_role(setting: &MaintenanceSetting, role_label: &str) -> bool {
        if setting.applicable_for.is_all() {
            return true;
        }
        setting
            .applicable_for
            .as_str()
            .split(',')
            .map(str::trim)
            .any(|s| s == role_label)
    }

    /// Returns Ok(()) if the setting's message is valid.
    pub fn validate_message(setting: &MaintenanceSetting) -> Result<(), String> {
        if setting.title.as_str().trim().is_empty() {
            return Err("maintenance_title must not be empty".to_owned());
        }
        if setting.sub_title.as_str().trim().is_empty() {
            return Err("maintenance_sub_title must not be empty".to_owned());
        }
        Ok(())
    }

    /// Returns a default `MaintenanceSetting` for a school with the
    /// standard "we are down" message and `maintenance_mode = false`.
    #[must_use]
    pub fn default_setting(
        id: crate::value_objects::MaintenanceSettingId,
        _school: SchoolId,
        created_by: UserId,
        now: Timestamp,
        correlation_id: educore_core::ids::CorrelationId,
    ) -> MaintenanceSetting {
        MaintenanceSetting::configure(crate::aggregate::NewMaintenanceSetting {
            id,
            title: MaintenanceTitle::default_value(),
            sub_title: MaintenanceSubTitle::default_value(),
            image: None,
            applicable_for: MaintenanceApplicableFor::new("all").unwrap_or_else(|_| {
                // Fallback if "all" somehow fails validation (it never should).
                MaintenanceApplicableFor("all".to_owned())
            }),
            created_by,
            created_at: now,
            correlation_id,
        })
        .expect("default MaintenanceSetting must construct")
    }
}

// === MaintenanceService section end ===

// =============================================================================
// === SidebarService section begin (owner: B) ===
// =============================================================================

/// Pure helpers for the [`Sidebar`](crate::aggregate::Sidebar) aggregate.
pub struct SidebarService;

impl SidebarService {
    /// Builds a tree representation of the supplied `Sidebar`
    /// entries (Parents first, then their children, etc.). The
    /// result is a flat list of `(sidebar_id, level)` tuples in
    /// tree order.
    #[must_use]
    pub fn tree_order(entries: &[Sidebar]) -> Vec<(crate::value_objects::SidebarId, i32)> {
        // Sort by level ascending (parents first), then by position.
        let mut sorted: Vec<&Sidebar> = entries.iter().collect();
        sorted.sort_by(|a, b| {
            a.level
                .get()
                .cmp(&b.level.get())
                .then_with(|| a.position.get().cmp(&b.position.get()))
        });
        sorted.iter().map(|e| (e.id, e.level.get())).collect()
    }

    /// Returns the children of `parent`.
    #[must_use]
    pub fn children<'a>(
        parent: crate::value_objects::SidebarId,
        entries: &'a [Sidebar],
    ) -> Vec<&'a Sidebar> {
        entries
            .iter()
            .filter(|e| e.parent == Some(parent))
            .collect()
    }

    /// Validates a reorder map and applies it to the in-memory list.
    pub fn reorder(
        entries: &mut [Sidebar],
        new_positions: &BTreeMap<crate::value_objects::SidebarId, SidebarPosition>,
    ) -> Result<(), String> {
        for (id, pos) in new_positions {
            if pos.get() < 0 {
                return Err(format!(
                    "position for {id:?} must be >= 0, got {}",
                    pos.get()
                ));
            }
            if let Some(entry) = entries.iter_mut().find(|e| e.id == *id) {
                entry.position = *pos;
            }
        }
        Ok(())
    }

    /// Returns the visible entries for a role (ignoring Hide and Disabled).
    #[must_use]
    pub fn visible<'a>(entries: &'a [Sidebar], role: RoleId) -> Vec<&'a Sidebar> {
        entries
            .iter()
            .filter(|e| {
                e.role_id == role
                    && e.active_status.0
                    && !e.ignore.is_hidden()
                    && !e.ignore.is_disabled()
            })
            .collect()
    }
}

// === SidebarService section end ===

// =============================================================================
// === AuditService section begin (owner: B) ===
// =============================================================================

/// Pure helpers for reading the user-log audit stream.
pub struct AuditService;

impl AuditService {
    /// Filter the audit log by user id.
    #[must_use]
    pub fn filter_by_user<'a>(log: &'a [UserLog], user: UserId) -> Vec<&'a UserLog> {
        log.iter().filter(|l| l.user_id == user).collect()
    }

    /// Filter the audit log by login outcome.
    #[must_use]
    pub fn filter_by_outcome<'a>(log: &'a [UserLog], outcome: LoginOutcome) -> Vec<&'a UserLog> {
        log.iter()
            .filter(|l| outcome_matches(&l.outcome, &outcome))
            .collect()
    }

    /// Filter the audit log by date range (inclusive on both ends).
    #[must_use]
    pub fn filter_by_date_range<'a>(
        log: &'a [UserLog],
        from: Timestamp,
        to: Timestamp,
    ) -> Vec<&'a UserLog> {
        log.iter()
            .filter(|l| l.logged_at >= from && l.logged_at <= to)
            .collect()
    }

    /// Builds an `AuditExport` shape (counts + distinct values) for
    /// the supplied log slice. Useful for compliance exports.
    #[must_use]
    pub fn export(log: &[UserLog]) -> AuditExport {
        AuditExport {
            total: log.len(),
            successes: log
                .iter()
                .filter(|l| matches!(l.outcome, LoginOutcome::Success))
                .count(),
            failures: log
                .iter()
                .filter(|l| matches!(l.outcome, LoginOutcome::Failure))
                .count(),
            distinct_ips: UserLogService::distinct_ips(log).len(),
            distinct_user_agents: UserLogService::distinct_user_agents(log).len(),
        }
    }
}

/// A summary of a `UserLog` slice, used for compliance exports.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditExport {
    /// Total rows.
    pub total: usize,
    /// Count of successful logins.
    pub successes: usize,
    /// Count of failed logins.
    pub failures: usize,
    /// Number of distinct IPs in the slice.
    pub distinct_ips: usize,
    /// Number of distinct user agents in the slice.
    pub distinct_user_agents: usize,
}

// === AuditService section end ===

fn outcome_matches(a: &LoginOutcome, b: &LoginOutcome) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

// =============================================================================
// === Policy: OneRestoreInProgress section begin (owner: B) ===
// =============================================================================

/// Policy: rejects a `RestoreBackup` command if a restore is
/// already in progress for the school.
pub struct OneRestoreInProgress;

impl OneRestoreInProgress {
    /// Returns Ok(()) if no other restore is in progress.
    pub fn check(restore_in_progress_count: u32) -> Result<(), String> {
        if restore_in_progress_count > 0 {
            return Err(format!(
                "another restore is already in progress (count={restore_in_progress_count})"
            ));
        }
        Ok(())
    }
}

// === OneRestoreInProgress section end ===

// =============================================================================
// === Policy: MaintenanceLockout section begin (owner: B) ===
// =============================================================================

/// Policy: rejects non-admin logins for a school whose maintenance
/// mode is enabled.
pub struct MaintenanceLockout;

impl MaintenanceLockout {
    /// Returns Ok(()) if the role is allowed to log in while
    /// maintenance mode is enabled (i.e. the role is an admin).
    pub fn check(is_maintenance_mode: bool, role_label: &str) -> Result<(), String> {
        if !is_maintenance_mode {
            return Ok(());
        }
        // Only SuperAdmin / SchoolAdmin can log in while maintenance is on.
        if role_label.eq_ignore_ascii_case("super_admin")
            || role_label.eq_ignore_ascii_case("school_admin")
            || role_label.eq_ignore_ascii_case("superadmin")
            || role_label.eq_ignore_ascii_case("schooladmin")
        {
            Ok(())
        } else {
            Err(format!(
                "maintenance mode blocks login for role {role_label:?}"
            ))
        }
    }
}

// === MaintenanceLockout section end ===

// =============================================================================
// === Policy: DisableMaintenanceGuard section begin (owner: B) ===
// =============================================================================

/// Policy: rejects a `DisableMaintenance` command from a non-admin actor.
pub struct DisableMaintenanceGuard;

impl DisableMaintenanceGuard {
    /// Returns Ok(()) if the actor is allowed to disable maintenance.
    pub fn check(actor_role_label: &str) -> Result<(), String> {
        if actor_role_label.eq_ignore_ascii_case("super_admin")
            || actor_role_label.eq_ignore_ascii_case("school_admin")
            || actor_role_label.eq_ignore_ascii_case("superadmin")
            || actor_role_label.eq_ignore_ascii_case("schooladmin")
        {
            Ok(())
        } else {
            Err(format!(
                "only SuperAdmin / SchoolAdmin may disable maintenance, got {actor_role_label:?}"
            ))
        }
    }
}

// === DisableMaintenanceGuard section end ===

// =============================================================================
// === Specification: ActiveBackups section begin (owner: B) ===
// =============================================================================

/// Specification: filters backups to those whose `active_status` is true.
pub struct ActiveBackups;

impl ActiveBackups {
    /// Returns true if the backup is active.
    #[must_use]
    pub fn is_satisfied_by(b: &Backup) -> bool {
        b.active_status
    }
}

// === ActiveBackups section end ===

// =============================================================================
// === Specification: DatabaseBackups section begin (owner: B) ===
// =============================================================================

/// Specification: filters backups to those whose `file_type` is `Database`.
pub struct DatabaseBackups;

impl DatabaseBackups {
    /// Returns true if the backup is a database backup.
    #[must_use]
    pub fn is_satisfied_by(b: &Backup) -> bool {
        BackupService::is_database(b)
    }
}

// === DatabaseBackups section end ===

// =============================================================================
// === Specification: SuccessfulLogins section begin (owner: B) ===
// =============================================================================

/// Specification: filters `UserLog` rows to those whose outcome is `Success`.
pub struct SuccessfulLogins;

impl SuccessfulLogins {
    /// Returns true if the user log row is a successful login.
    #[must_use]
    pub fn is_satisfied_by(l: &UserLog) -> bool {
        matches!(l.outcome, LoginOutcome::Success)
    }
}

// === SuccessfulLogins section end ===

// =============================================================================
// === Specification: FailedLogins section begin (owner: B) ===
// =============================================================================

/// Specification: filters `UserLog` rows to those whose outcome is `Failure`.
pub struct FailedLogins;

impl FailedLogins {
    /// Returns true if the user log row is a failed login.
    #[must_use]
    pub fn is_satisfied_by(l: &UserLog) -> bool {
        matches!(l.outcome, LoginOutcome::Failure)
    }
}

// === FailedLogins section end ===

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::{
        BackupFileName, BackupFileType, BackupSourceLink, FailedJobConnection, FailedJobException,
        FailedJobQueue, FailedJobUuid, FileReference, HistoryNotes, HistoryReleaseDate,
        HistoryVersion, IpAddress, JobPayload, MaintenanceApplicableFor, MaintenanceImage,
        MaintenanceSubTitle, MaintenanceTitle, PermissionId, SidebarActiveStatus,
        SidebarIgnoreFlag, SidebarIsSystemDefined, SidebarLevel, SidebarPosition, SidebarSectionId,
        UserAgent, VersionFeatures, VersionTitle,
    };
    use educore_core::ids::{CorrelationId, EventId, Identifier, SchoolId, UserId};
    use uuid::Uuid;

    #[test]
    fn job_service_next_backoff_headline() {
        assert_eq!(JobService::next_backoff(JobAttempts::new(0)), 1);
        assert_eq!(JobService::next_backoff(JobAttempts::new(1)), 2);
        assert_eq!(JobService::next_backoff(JobAttempts::new(2)), 4);
        assert_eq!(JobService::next_backoff(JobAttempts::new(3)), 8);
        assert_eq!(JobService::next_backoff(JobAttempts::new(4)), 16);
        assert_eq!(JobService::next_backoff(JobAttempts::new(5)), 32);
        assert_eq!(JobService::next_backoff(JobAttempts::new(6)), 64);
        assert_eq!(JobService::next_backoff(JobAttempts::new(7)), 128);
        assert_eq!(JobService::next_backoff(JobAttempts::new(8)), 256);
        assert_eq!(JobService::next_backoff(JobAttempts::new(9)), 512);
        assert_eq!(JobService::next_backoff(JobAttempts::new(10)), 1024);
        // Overflow guard.
        assert_eq!(JobService::next_backoff(JobAttempts::new(32)), u32::MAX);
        assert_eq!(JobService::next_backoff(JobAttempts::new(255)), u32::MAX);
    }

    #[test]
    fn backup_service_can_delete_blocks_during_restore() {
        let school = SchoolId::from_uuid(Uuid::nil());
        let mut b = Backup::new(crate::aggregate::NewBackup {
            id: crate::value_objects::BackupId::new(school, Uuid::nil()),
            file_name: BackupFileName::new("backup.sql").unwrap(),
            source_link: BackupSourceLink::new("s3://bucket/backup.sql").unwrap(),
            file_type: BackupFileType::Database,
            lang_type: None,
            active_status: true,
            restore_in_progress: false,
            created_by: UserId::from_uuid(Uuid::nil()),
            created_at: Timestamp::now(),
            correlation_id: CorrelationId::from_uuid(Uuid::nil()),
        })
        .unwrap();
        assert!(BackupService::can_delete(&b, false).is_ok());
        b.mark_restoring(
            UserId::from_uuid(Uuid::nil()),
            Timestamp::now(),
            EventId::from_uuid(Uuid::nil()),
        );
        assert!(BackupService::can_delete(&b, false).is_err());
    }

    #[test]
    fn system_version_service_is_newer_and_compatible() {
        let a = VersionName::new("8.2.3").unwrap();
        let b = VersionName::new("8.2.4").unwrap();
        let c = VersionName::new("9.0.0").unwrap();
        assert!(SystemVersionService::is_newer(&b, &a));
        assert!(!SystemVersionService::is_newer(&a, &b));
        assert!(SystemVersionService::is_newer(&c, &b));
        // Compatibility: same major (8.x with 8.y) is compatible; 8.x with 9.x is not.
        assert!(SystemVersionService::is_compatible(&a, &b));
        assert!(!SystemVersionService::is_compatible(&a, &c));
    }

    #[test]
    fn one_restore_in_progress_policy() {
        assert!(OneRestoreInProgress::check(0).is_ok());
        assert!(OneRestoreInProgress::check(1).is_err());
    }

    #[test]
    fn maintenance_lockout_blocks_non_admin() {
        assert!(MaintenanceLockout::check(false, "student").is_ok());
        assert!(MaintenanceLockout::check(true, "super_admin").is_ok());
        assert!(MaintenanceLockout::check(true, "school_admin").is_ok());
        assert!(MaintenanceLockout::check(true, "student").is_err());
        assert!(MaintenanceLockout::check(true, "teacher").is_err());
    }

    #[test]
    fn disable_maintenance_guard_rejects_non_admin() {
        assert!(DisableMaintenanceGuard::check("super_admin").is_ok());
        assert!(DisableMaintenanceGuard::check("school_admin").is_ok());
        assert!(DisableMaintenanceGuard::check("teacher").is_err());
        assert!(DisableMaintenanceGuard::check("student").is_err());
    }

    #[test]
    fn backup_specifications_partition_correctly() {
        let school = SchoolId::from_uuid(Uuid::nil());
        let mk = |file_type: BackupFileType, active: bool| {
            Backup::new(crate::aggregate::NewBackup {
                id: crate::value_objects::BackupId::new(school, Uuid::from_u128(rand_u128())),
                file_name: BackupFileName::new("backup.sql").unwrap(),
                source_link: BackupSourceLink::new("s3://bucket/backup.sql").unwrap(),
                file_type,
                lang_type: None,
                active_status: active,
                restore_in_progress: false,
                created_by: UserId::from_uuid(Uuid::nil()),
                created_at: Timestamp::now(),
                correlation_id: CorrelationId::from_uuid(Uuid::nil()),
            })
        };
        let db = mk(BackupFileType::Database, true).unwrap();
        let img = mk(BackupFileType::Image, true).unwrap();
        let inactive_db = mk(BackupFileType::Database, false).unwrap();
        assert!(ActiveBackups::is_satisfied_by(&db));
        assert!(!ActiveBackups::is_satisfied_by(&inactive_db));
        assert!(DatabaseBackups::is_satisfied_by(&db));
        assert!(!DatabaseBackups::is_satisfied_by(&img));
    }

    fn rand_u128() -> u128 {
        Uuid::new_v4().as_u128()
    }

    #[test]
    fn user_log_specifications_partition_correctly() {
        let school = SchoolId::from_uuid(Uuid::nil());
        let mk = |outcome: LoginOutcome| {
            UserLog::new(
                crate::value_objects::UserLogId::new(school, Uuid::from_u128(rand_u128())),
                crate::entities::UserLogInput::new(
                    school,
                    UserId::from_uuid(Uuid::nil()),
                    RoleId::new(school, Uuid::nil()),
                    IpAddress::new("192.0.2.1").unwrap(),
                    UserAgent::new("Mozilla/5.0").unwrap(),
                    outcome,
                ),
                UserId::from_uuid(Uuid::nil()),
                CorrelationId::from_uuid(Uuid::nil()),
                Timestamp::now(),
            )
        };
        let success = mk(LoginOutcome::Success);
        let failure = mk(LoginOutcome::Failure);
        assert!(SuccessfulLogins::is_satisfied_by(&success));
        assert!(!SuccessfulLogins::is_satisfied_by(&failure));
        assert!(FailedLogins::is_satisfied_by(&failure));
        assert!(!FailedLogins::is_satisfied_by(&success));
    }

    #[test]
    fn sidebar_service_reorder_validates_and_updates() {
        let school = SchoolId::from_uuid(Uuid::nil());
        let id = crate::value_objects::SidebarId::new(school, Uuid::from_u128(1));
        let sidebar = Sidebar::new(crate::aggregate::NewSidebar {
            id,
            permission_id: PermissionId::new(school, Uuid::from_u128(2)),
            role_id: RoleId::new(school, Uuid::from_u128(3)),
            position: SidebarPosition::new(0).unwrap(),
            section_id: SidebarSectionId::new(1),
            parent: None,
            parent_route: None,
            level: SidebarLevel::new(1).unwrap(),
            is_system_defined: SidebarIsSystemDefined::new(false),
            ignore: SidebarIgnoreFlag::new(0).unwrap(),
            active_status: SidebarActiveStatus::new(true),
            user_id: UserId::from_uuid(Uuid::from_u128(4)),
            created_by: UserId::from_uuid(Uuid::from_u128(4)),
            created_at: Timestamp::now(),
            correlation_id: CorrelationId::from_uuid(Uuid::nil()),
        })
        .unwrap();
        let mut entries = vec![sidebar];
        let mut map = BTreeMap::new();
        map.insert(id, SidebarPosition::new(5).unwrap());
        SidebarService::reorder(&mut entries, &map).unwrap();
        assert_eq!(entries[0].position.get(), 5);
    }

    #[test]
    fn failed_job_extracts_exception_type() {
        let ex = "RuntimeError: kaboom at line 42";
        assert_eq!(
            FailedJobService::extract_exception_type(ex),
            Some("RuntimeError")
        );
        let ex2 = "RuntimeError";
        assert_eq!(
            FailedJobService::extract_exception_type(ex2),
            Some("RuntimeError")
        );
        assert_eq!(FailedJobService::extract_exception_type(""), None);
    }

    #[test]
    fn audit_service_export_summary() {
        let school = SchoolId::from_uuid(Uuid::nil());
        let mk = |outcome: LoginOutcome| {
            UserLog::new(
                crate::value_objects::UserLogId::new(school, Uuid::from_u128(rand_u128())),
                crate::entities::UserLogInput::new(
                    school,
                    UserId::from_uuid(Uuid::nil()),
                    RoleId::new(school, Uuid::nil()),
                    IpAddress::new("192.0.2.1").unwrap(),
                    UserAgent::new("Mozilla/5.0").unwrap(),
                    outcome,
                ),
                UserId::from_uuid(Uuid::nil()),
                CorrelationId::from_uuid(Uuid::nil()),
                Timestamp::now(),
            )
        };
        let log = vec![mk(LoginOutcome::Success), mk(LoginOutcome::Failure)];
        let exp = AuditService::export(&log);
        assert_eq!(exp.total, 2);
        assert_eq!(exp.successes, 1);
        assert_eq!(exp.failures, 1);
    }
}
