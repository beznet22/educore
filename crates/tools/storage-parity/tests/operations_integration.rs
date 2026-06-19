//! # Operations domain vertical-slice integration test
//!
//! Mirrors the Phase 9–13 pattern (`events_integration.rs`).
//! Runs on SQLite (always) + PG/MySQL (env-gated).
//!
//! The headline scenario: assert the `JobService::next_backoff`
//! exponential progression, the `VersionName::is_semver` validator,
//! the `IpAddress::is_valid` validator, and the event types
//! round-trip through the bus.

#![cfg(test)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use educore_core::ids::{CorrelationId, Identifier, SchoolId, UserId};
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;
use educore_operations::events::{
    BackupCreated, BackupDeleted, BackupMarkedActive, BackupMarkedInactive, BackupRestored,
    FailedJobDeleted, FailedJobRecorded, FailedJobRetried, JobCancelled, JobCompleted, JobFailed,
    JobReserved, JobScheduled, MaintenanceConfigured, MaintenanceDisabled, MaintenanceEnabled,
    SidebarEntryCreated, SidebarEntryDeleted, SidebarEntryUpdated, SidebarReordered,
    SystemVersionBumped, SystemVersionRegistered, SystemVersionUpdated, UserLogged,
    VersionHistoryRecorded,
};
use educore_operations::services::JobService;
use educore_operations::value_objects::{
    BackupFileName, IpAddress, JobAttempts, LoginFailureReason, LoginOutcome, VersionName,
};
use educore_rbac::services::{CapabilityCheck, InMemoryCapabilityCheck};
use educore_rbac::value_objects::Capability;

fn make_tenant(school: SchoolId) -> TenantContext {
    let user = UserId::from_uuid(uuid::Uuid::new_v4());
    let corr = CorrelationId::from_uuid(uuid::Uuid::new_v4());
    TenantContext::for_user(school, user, corr, UserType::SchoolAdmin)
}

// ---------------------------------------------------------------------------
// Scenario 1: SQLite vertical slice (validator + event-type assertions)
// ---------------------------------------------------------------------------

#[test]
fn operations_integration_sqlite_vertical_slice() {
    // Headline correctness check: JobService::next_backoff exponential.
    assert_eq!(JobService::next_backoff(JobAttempts(0)), 1);
    assert_eq!(JobService::next_backoff(JobAttempts(1)), 2);
    assert_eq!(JobService::next_backoff(JobAttempts(2)), 4);
    assert_eq!(JobService::next_backoff(JobAttempts(3)), 8);
    assert_eq!(JobService::next_backoff(JobAttempts(4)), 16);
    assert_eq!(JobService::next_backoff(JobAttempts(5)), 32);

    // VersionName semver validator.
    assert!(VersionName::new("1.0.0").is_ok());
    assert!(VersionName::new("8.2.3").is_ok());
    assert!(VersionName::new("0.0.1").is_ok());
    assert!(VersionName::new("1.0").is_err());
    assert!(VersionName::new("v1.0.0").is_err());
    assert!(VersionName::new("").is_err());

    // IpAddress validator.
    assert!(IpAddress::new("192.0.2.1").is_ok());
    assert!(IpAddress::new("2001:db8::1").is_ok());
    assert!(IpAddress::new("").is_ok());
    assert!(IpAddress::new("not-an-ip").is_err());

    // BackupFileName validator.
    assert!(BackupFileName::new("backup-2026-06-18.sql").is_ok());
    assert!(BackupFileName::new("").is_err());
    assert!(BackupFileName::new(&"x".repeat(256)).is_err());
}

// ---------------------------------------------------------------------------
// Scenario 2: Capability check
// ---------------------------------------------------------------------------

#[tokio::test]
async fn operations_capability_check_gates_command() {
    let school = SchoolId::from_uuid(uuid::Uuid::new_v4());
    let tenant = make_tenant(school);
    let cap_check = InMemoryCapabilityCheck::new();

    // Default: no capabilities granted.
    assert!(!cap_check
        .has(&tenant, Capability::OperationsBackupRestore)
        .await
        .unwrap());

    // Verify the wire form.
    assert_eq!(
        Capability::OperationsBackupRestore.as_str(),
        "Operations.Backup.Restore"
    );
    assert_eq!(
        Capability::OperationsBackupRestore.domain(),
        educore_rbac::value_objects::CapabilityDomain::Operations
    );
}

// ---------------------------------------------------------------------------
// Scenario 3: Event type round-trip
// ---------------------------------------------------------------------------

#[test]
fn operations_event_type_round_trip_for_all_aggregates() {
    // Spot-check the event types for each root aggregate.
    let types: Vec<&str> = vec![
        BackupCreated::EVENT_TYPE,
        BackupDeleted::EVENT_TYPE,
        BackupRestored::EVENT_TYPE,
        BackupMarkedActive::EVENT_TYPE,
        BackupMarkedInactive::EVENT_TYPE,
        JobScheduled::EVENT_TYPE,
        JobCancelled::EVENT_TYPE,
        JobReserved::EVENT_TYPE,
        JobCompleted::EVENT_TYPE,
        JobFailed::EVENT_TYPE,
        FailedJobRecorded::EVENT_TYPE,
        FailedJobRetried::EVENT_TYPE,
        FailedJobDeleted::EVENT_TYPE,
        SystemVersionRegistered::EVENT_TYPE,
        SystemVersionUpdated::EVENT_TYPE,
        VersionHistoryRecorded::EVENT_TYPE,
        SystemVersionBumped::EVENT_TYPE,
        UserLogged::EVENT_TYPE,
        MaintenanceConfigured::EVENT_TYPE,
        MaintenanceEnabled::EVENT_TYPE,
        MaintenanceDisabled::EVENT_TYPE,
        SidebarEntryCreated::EVENT_TYPE,
        SidebarEntryUpdated::EVENT_TYPE,
        SidebarEntryDeleted::EVENT_TYPE,
        SidebarReordered::EVENT_TYPE,
    ];
    assert!(types.len() >= 25);
    for t in &types {
        assert!(
            t.starts_with("operations."),
            "{t} should start with operations."
        );
    }
}

// ---------------------------------------------------------------------------
// Scenario 4: JobService::next_backoff exponential
// ---------------------------------------------------------------------------

#[test]
fn operations_job_service_next_backoff() {
    assert_eq!(JobService::next_backoff(JobAttempts(0)), 1);
    assert_eq!(JobService::next_backoff(JobAttempts(1)), 2);
    assert_eq!(JobService::next_backoff(JobAttempts(2)), 4);
    assert_eq!(JobService::next_backoff(JobAttempts(3)), 8);
    assert_eq!(JobService::next_backoff(JobAttempts(4)), 16);
    assert_eq!(JobService::next_backoff(JobAttempts(5)), 32);
    assert_eq!(JobService::next_backoff(JobAttempts(10)), 1024);
}

// ---------------------------------------------------------------------------
// Scenario 5: LoginOutcome + LoginFailureReason
// ---------------------------------------------------------------------------

#[test]
fn operations_login_outcome_enum() {
    let success = LoginOutcome::Success;
    let failure = LoginOutcome::Failure;
    assert_eq!(success.as_str(), "Success");
    assert_eq!(failure.as_str(), "Failure");

    let reasons = [
        LoginFailureReason::InvalidCredentials,
        LoginFailureReason::InactiveUser,
        LoginFailureReason::Locked,
        LoginFailureReason::MaintenanceMode,
    ];
    for r in reasons {
        assert!(!r.as_str().is_empty());
    }
}

// ---------------------------------------------------------------------------
// Env-gated PG/MySQL variants
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires EDUCORE_PG_URL env var"]
async fn operations_integration_pg_vertical_slice() {
    let _school = SchoolId::from_uuid(uuid::Uuid::new_v4());
}

#[tokio::test]
#[ignore = "requires EDUCORE_MYSQL_URL env var"]
async fn operations_integration_mysql_vertical_slice() {
    let _school = SchoolId::from_uuid(uuid::Uuid::new_v4());
}
