//! # educore-operations value objects
//!
//! Typed ids, value objects, and closed enums per
//! `docs/specs/operations/value-objects.md`.

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::error::{DomainError, Result};
pub use educore_core::ids::SchoolId;

// =============================================================================
// Macro: typed operations id (tenant-scoped)
// =============================================================================

macro_rules! operations_typed_id {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident;
    ) => {
        $(#[$attr])*
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        $vis struct $name {
            /// The owning school (tenant anchor).
            pub school_id: SchoolId,
            /// The local id (UUIDv7).
            pub value: Uuid,
        }

        impl $name {
            /// Constructs a new typed id from its parts.
            #[must_use]
            pub const fn new(school_id: SchoolId, value: Uuid) -> Self {
                Self { school_id, value }
            }
            /// Returns the local UUID.
            #[must_use]
            pub const fn as_uuid(&self) -> Uuid {
                self.value
            }
            /// Returns the owning school id.
            #[must_use]
            pub const fn school_id(&self) -> SchoolId {
                self.school_id
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}/{}", self.school_id, self.value)
            }
        }
    };
}

// =============================================================================
// Macro: typed global operations id (no school_id)
// =============================================================================

macro_rules! operations_global_typed_id {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident;
    ) => {
        $(#[$attr])*
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        $vis struct $name {
            /// The local id (UUIDv7).
            pub value: Uuid,
        }

        impl $name {
            /// Constructs a new global typed id.
            #[must_use]
            pub const fn new(value: Uuid) -> Self {
                Self { value }
            }
            /// Returns the local UUID.
            #[must_use]
            pub const fn as_uuid(&self) -> Uuid {
                self.value
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.value.fmt(f)
            }
        }
    };
}

// =============================================================================
// Typed root ids (8) — tenant-scoped
// =============================================================================

operations_typed_id! {
    /// A typed id for a [`Backup`](crate::aggregate::Backup) row.
    pub struct BackupId;
}
operations_typed_id! {
    /// A typed id for a [`UserLog`](crate::aggregate::UserLog) row.
    pub struct UserLogId;
}
operations_typed_id! {
    /// A typed id for a [`MaintenanceSetting`](crate::aggregate::MaintenanceSetting) row.
    pub struct MaintenanceSettingId;
}
operations_typed_id! {
    /// A typed id for a [`Sidebar`](crate::aggregate::Sidebar) row.
    pub struct SidebarId;
}

// =============================================================================
// Typed root ids (4) — global, no school_id
// =============================================================================

operations_global_typed_id! {
    /// A typed id for a [`Job`](crate::aggregate::Job) row.
    pub struct JobId;
}
operations_global_typed_id! {
    /// A typed id for a [`FailedJob`](crate::aggregate::FailedJob) row.
    pub struct FailedJobId;
}
operations_global_typed_id! {
    /// A typed id for a [`SystemVersion`](crate::aggregate::SystemVersion) row.
    pub struct SystemVersionId;
}
operations_global_typed_id! {
    /// A typed id for a [`VersionHistory`](crate::aggregate::VersionHistory) row.
    pub struct VersionHistoryId;
}

// =============================================================================
// Auxiliary typed ids (4) — global
// =============================================================================

operations_global_typed_id! {
    /// A typed id for a [`JobAttempt`](crate::entities::JobAttempt) row.
    pub struct JobAttemptId;
}
operations_global_typed_id! {
    /// A typed id for a [`BackupSchedule`](crate::entities::BackupSchedule) row.
    pub struct BackupScheduleId;
}
operations_global_typed_id! {
    /// A typed id for a [`BackupRetention`](crate::entities::BackupRetention) row.
    pub struct BackupRetentionId;
}
operations_global_typed_id! {
    /// A typed id for an [`AuditPartition`](crate::entities::AuditPartition) row.
    pub struct AuditPartitionId;
}

// =============================================================================
// Local newtypes for cross-domain refs (no academic / rbac deps)
// =============================================================================

/// A typed reference to an `AcademicYear` aggregate in the
/// `educore-academic` domain. We deliberately do **not** depend on
/// `educore-academic` here; the spec treats the id as opaque from
/// the operations domain's point of view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AcademicYearRef {
    /// The owning school.
    pub school_id: SchoolId,
    /// The local id.
    pub value: Uuid,
}

impl AcademicYearRef {
    /// Constructs a new `AcademicYearRef`.
    #[must_use]
    pub const fn new(school_id: SchoolId, value: Uuid) -> Self {
        Self { school_id, value }
    }
    /// Returns the local UUID.
    #[must_use]
    pub const fn as_uuid(&self) -> Uuid {
        self.value
    }
}

impl fmt::Display for AcademicYearRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.school_id, self.value)
    }
}

/// A typed reference to a `Role` row in the `educore-rbac` domain.
/// Local newtype; we deliberately do **not** depend on `educore-rbac`
/// for this id — the spec treats the role id as opaque from the
/// operations domain's point of view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoleId {
    /// The owning school.
    pub school_id: SchoolId,
    /// The local id.
    pub value: Uuid,
}

impl RoleId {
    /// Constructs a new `RoleId`.
    #[must_use]
    pub const fn new(school_id: SchoolId, value: Uuid) -> Self {
        Self { school_id, value }
    }
    /// Returns the local UUID.
    #[must_use]
    pub const fn as_uuid(&self) -> Uuid {
        self.value
    }
    /// Returns the owning school id.
    #[must_use]
    pub const fn school_id(&self) -> SchoolId {
        self.school_id
    }
}

impl fmt::Display for RoleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.school_id, self.value)
    }
}

/// A typed reference to a `Permission` row in the `educore-rbac`
/// domain. Local newtype; the operations domain does not depend on
/// `educore-rbac` for this id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PermissionId {
    /// The owning school.
    pub school_id: SchoolId,
    /// The local id.
    pub value: Uuid,
}

impl PermissionId {
    /// Constructs a new `PermissionId`.
    #[must_use]
    pub const fn new(school_id: SchoolId, value: Uuid) -> Self {
        Self { school_id, value }
    }
    /// Returns the local UUID.
    #[must_use]
    pub const fn as_uuid(&self) -> Uuid {
        self.value
    }
    /// Returns the owning school id.
    #[must_use]
    pub const fn school_id(&self) -> SchoolId {
        self.school_id
    }
}

impl fmt::Display for PermissionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.school_id, self.value)
    }
}

// =============================================================================
// FileReference (shared, like settings)
// =============================================================================

/// A general-purpose file reference (image upload, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FileReference(pub String);

impl FileReference {
    /// Constructs a new `FileReference`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s = s.into();
        if s.is_empty() || s.len() > 1024 {
            return Err(DomainError::Validation(format!(
                "file_reference must be 1..1024 chars"
            )));
        }
        Ok(Self(s))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// =============================================================================
// Closed enums
// =============================================================================

/// The `Backup::file_type` discriminator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BackupFileType {
    /// Database backup (0).
    Database,
    /// File backup (1).
    File,
    /// Image backup (2).
    Image,
}

impl BackupFileType {
    /// Returns the canonical wire integer.
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        match self {
            Self::Database => 0,
            Self::File => 1,
            Self::Image => 2,
        }
    }
    /// Returns the canonical wire name.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Database => "Database",
            Self::File => "File",
            Self::Image => "Image",
        }
    }
}

impl fmt::Display for BackupFileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The `Job::status` discriminator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JobStatus {
    /// Pending execution.
    Pending,
    /// Reserved by a worker.
    Reserved,
    /// Completed successfully.
    Completed,
    /// Failed terminally.
    Failed,
}

impl JobStatus {
    /// Returns the canonical wire name.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Reserved => "Reserved",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
        }
    }
}

impl fmt::Display for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The outcome of a login attempt recorded in the `UserLog` audit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LoginOutcome {
    /// The login succeeded.
    Success,
    /// The login failed.
    Failure,
}

impl LoginOutcome {
    /// Returns the canonical wire name.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "Success",
            Self::Failure => "Failure",
        }
    }
}

impl fmt::Display for LoginOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The reason a login failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LoginFailureReason {
    /// Bad credentials.
    InvalidCredentials,
    /// User is inactive.
    InactiveUser,
    /// User is locked out.
    Locked,
    /// Login blocked by maintenance mode.
    MaintenanceMode,
}

impl LoginFailureReason {
    /// Returns the canonical wire name.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::InvalidCredentials => "InvalidCredentials",
            Self::InactiveUser => "InactiveUser",
            Self::Locked => "Locked",
            Self::MaintenanceMode => "MaintenanceMode",
        }
    }
}

impl fmt::Display for LoginFailureReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// `Sidebar::level` wrapper. 1=Parent, 2=Child, 3=SubChild.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SidebarLevel(pub i32);

impl SidebarLevel {
    /// Constructs a new `SidebarLevel`.
    pub fn new(v: i32) -> Result<Self> {
        if !(1..=3).contains(&v) {
            return Err(DomainError::Validation(format!(
                "sidebar_level must be 1..=3, got {v}"
            )));
        }
        Ok(Self(v))
    }
    /// Returns true if this is a Parent level.
    #[must_use]
    pub const fn is_parent(self) -> bool {
        self.0 == 1
    }
    /// Returns the inner integer.
    #[must_use]
    pub const fn get(self) -> i32 {
        self.0
    }
}

/// `Sidebar::ignore` flag wrapper. 0=Show, 1=Hide, 2=Disabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SidebarIgnoreFlag(pub i32);

impl SidebarIgnoreFlag {
    /// Constructs a new `SidebarIgnoreFlag`.
    pub fn new(v: i32) -> Result<Self> {
        if !(0..=2).contains(&v) {
            return Err(DomainError::Validation(format!(
                "sidebar_ignore_flag must be 0..=2, got {v}"
            )));
        }
        Ok(Self(v))
    }
    /// Returns true if the entry is shown (0).
    #[must_use]
    pub const fn is_shown(self) -> bool {
        self.0 == 0
    }
    /// Returns true if the entry is hidden (1).
    #[must_use]
    pub const fn is_hidden(self) -> bool {
        self.0 == 1
    }
    /// Returns true if the entry is disabled (2).
    #[must_use]
    pub const fn is_disabled(self) -> bool {
        self.0 == 2
    }
    /// Returns the inner integer.
    #[must_use]
    pub const fn get(self) -> i32 {
        self.0
    }
}

// =============================================================================
// Type-safe wrappers (validated at construction)
// =============================================================================

/// `Backup::file_name`. 1..255 chars, unique within `(school_id, file_name)`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BackupFileName(pub String);

impl BackupFileName {
    /// Constructs a new `BackupFileName`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 255 {
            return Err(DomainError::Validation(format!(
                "backup_file_name must be 1..255 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// `Backup::source_link`. URL or file-storage reference (1..255 chars).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BackupSourceLink(pub String);

impl BackupSourceLink {
    /// Constructs a new `BackupSourceLink`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 255 {
            return Err(DomainError::Validation(format!(
                "backup_source_link must be 1..255 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// `Backup::lang_type`. An `i32` consumer-defined hint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BackupLangType(pub i32);

impl BackupLangType {
    /// Constructs a new `BackupLangType`.
    #[must_use]
    pub const fn new(v: i32) -> Self {
        Self(v)
    }
    /// Returns the inner integer.
    #[must_use]
    pub const fn get(self) -> i32 {
        self.0
    }
}

/// `SystemVersion::version_name`. A valid semantic version (`MAJOR.MINOR.PATCH`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VersionName(pub String);

impl VersionName {
    /// Constructs a new `VersionName`, validating the semver shape.
    pub fn new(s: &str) -> Result<Self> {
        if !Self::is_semver(s) {
            return Err(DomainError::Validation(format!(
                "version_name must be a semver string (MAJOR.MINOR.PATCH), got {s:?}"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns true if `s` is a valid `MAJOR.MINOR.PATCH` semver string.
    /// Pre-release and build-metadata segments are not accepted; this is
    /// a deliberately small validator.
    #[must_use]
    pub fn is_semver(s: &str) -> bool {
        if s.is_empty() || s.len() > 191 {
            return false;
        }
        let mut parts = s.split('.');
        let Some(major) = parts.next() else {
            return false;
        };
        let Some(minor) = parts.next() else {
            return false;
        };
        let Some(patch) = parts.next() else {
            return false;
        };
        if parts.next().is_some() {
            return false;
        }
        is_non_negative_int(major) && is_non_negative_int(minor) && is_non_negative_int(patch)
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn is_non_negative_int(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    let first = match chars.next() {
        Some(c) => c,
        None => return false,
    };
    if !first.is_ascii_digit() {
        return false;
    }
    if first == '0' && s.len() > 1 {
        return false;
    }
    chars.all(|c| c.is_ascii_digit())
}

/// `SystemVersion::title`. 1..255 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VersionTitle(pub String);

impl VersionTitle {
    /// Constructs a new `VersionTitle`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 255 {
            return Err(DomainError::Validation(format!(
                "version_title must be 1..255 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// `SystemVersion::features`. 1..255 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VersionFeatures(pub String);

impl VersionFeatures {
    /// Constructs a new `VersionFeatures`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 255 {
            return Err(DomainError::Validation(format!(
                "version_features must be 1..255 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// `VersionHistory::version`. 1..191 chars (free-form).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HistoryVersion(pub String);

impl HistoryVersion {
    /// Constructs a new `HistoryVersion`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 191 {
            return Err(DomainError::Validation(format!(
                "history_version must be 1..191 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// `VersionHistory::release_date`. 1..191 chars (consumer format).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HistoryReleaseDate(pub String);

impl HistoryReleaseDate {
    /// Constructs a new `HistoryReleaseDate`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 191 {
            return Err(DomainError::Validation(format!(
                "history_release_date must be 1..191 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// `VersionHistory::url`. URL or empty, max 2048 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HistoryUrl(pub String);

impl HistoryUrl {
    /// Constructs a new `HistoryUrl`. Empty strings are allowed.
    pub fn new(s: &str) -> Result<Self> {
        if s.len() > 2048 {
            return Err(DomainError::Validation(format!(
                "history_url must be <= 2048 chars"
            )));
        }
        if !s.is_empty()
            && !(s.starts_with("http://") || s.starts_with("https://") || s.starts_with('/'))
        {
            return Err(DomainError::Validation(format!(
                "history_url must be empty or a URL, got {s:?}"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
    /// Returns true if set (non-empty).
    #[must_use]
    pub fn is_set(&self) -> bool {
        !self.0.is_empty()
    }
}

/// `VersionHistory::notes`. 1..191 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HistoryNotes(pub String);

impl HistoryNotes {
    /// Constructs a new `HistoryNotes`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 191 {
            return Err(DomainError::Validation(format!(
                "history_notes must be 1..191 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// `UserLog::ip_address`. Valid IPv4 or IPv6 or empty.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct IpAddress(pub String);

impl IpAddress {
    /// Constructs a new `IpAddress`. Empty strings are allowed.
    pub fn new(s: &str) -> Result<Self> {
        if !Self::is_valid(s) {
            return Err(DomainError::Validation(format!(
                "ip_address must be empty or a valid IPv4/IPv6 string, got {s:?}"
            )));
        }
        Ok(Self(s.to_owned()))
    }

    /// Returns true if `s` is a valid IPv4, IPv6, or empty string.
    #[must_use]
    pub fn is_valid(s: &str) -> bool {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return true;
        }
        // Try IPv4 first (fast path): four 0..=255 octets separated by '.'.
        if trimmed.contains(':') {
            return is_valid_ipv6(trimmed);
        }
        is_valid_ipv4(trimmed)
    }

    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn is_valid_ipv4(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    for part in parts {
        if part.is_empty() || part.len() > 3 {
            return false;
        }
        if part.len() > 1 && part.starts_with('0') {
            return false;
        }
        if !part.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
        // Parse to u16 to safely check the value range.
        let n: u32 = match part.parse() {
            Ok(n) => n,
            Err(_) => return false,
        };
        if n > 255 {
            return false;
        }
    }
    true
}

fn is_valid_ipv6(s: &str) -> bool {
    // Accept only canonical / zero-compressed forms. Allow at most one
    // "::" zero-compression run, and 8 groups total (each 1..=4 hex).
    if s.is_empty() {
        return false;
    }
    if s.len() > 45 {
        return false;
    }
    let parts: Vec<&str> = s.split("::").collect();
    let double_colon_count = if s.contains("::") { 1 } else { 0 };
    if double_colon_count > 1 {
        return false;
    }
    if parts.len() == 2 {
        // Two sides of the "::"; each side is a colon-separated list.
        let left = if parts[0].is_empty() {
            Vec::new()
        } else {
            parts[0].split(':').collect()
        };
        let right = if parts[1].is_empty() {
            Vec::new()
        } else {
            parts[1].split(':').collect()
        };
        for g in left.iter().chain(right.iter()) {
            if !is_valid_ipv6_group(g) {
                return false;
            }
        }
        // Compressed form: total groups <= 7.
        if left.len() + right.len() > 7 {
            return false;
        }
        true
    } else {
        // No compression — exactly 8 groups.
        let groups: Vec<&str> = s.split(':').collect();
        if groups.len() != 8 {
            return false;
        }
        groups.iter().all(|g| is_valid_ipv6_group(g))
    }
}

fn is_valid_ipv6_group(s: &str) -> bool {
    if s.is_empty() || s.len() > 4 {
        return false;
    }
    s.chars().all(|c| c.is_ascii_hexdigit())
}

/// `UserLog::user_agent`. 1..191 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UserAgent(pub String);

impl UserAgent {
    /// Constructs a new `UserAgent`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 191 {
            return Err(DomainError::Validation(format!(
                "user_agent must be 1..191 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// `FailedJob::uuid`. UUIDv4, unique.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FailedJobUuid(pub Uuid);

impl FailedJobUuid {
    /// Constructs a new `FailedJobUuid`.
    #[must_use]
    pub const fn new(value: Uuid) -> Self {
        Self(value)
    }
    /// Returns the inner UUID.
    #[must_use]
    pub const fn as_uuid(&self) -> Uuid {
        self.0
    }
}

/// `FailedJob::connection`. 1..191 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FailedJobConnection(pub String);

impl FailedJobConnection {
    /// Constructs a new `FailedJobConnection`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 191 {
            return Err(DomainError::Validation(format!(
                "failed_job_connection must be 1..191 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// `FailedJob::queue`. 1..191 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FailedJobQueue(pub String);

impl FailedJobQueue {
    /// Constructs a new `FailedJobQueue`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 191 {
            return Err(DomainError::Validation(format!(
                "failed_job_queue must be 1..191 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// `FailedJob::exception`. Longtext (1..65000 chars).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FailedJobException(pub String);

impl FailedJobException {
    /// Constructs a new `FailedJobException`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 65000 {
            return Err(DomainError::Validation(format!(
                "failed_job_exception must be 1..65000 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// `Job::queue`. 1..191 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobQueue(pub String);

impl JobQueue {
    /// Constructs a new `JobQueue`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 191 {
            return Err(DomainError::Validation(format!(
                "job_queue must be 1..191 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// `Job::payload`. A serialized command envelope.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobPayload(pub String);

impl JobPayload {
    /// Constructs a new `JobPayload`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s = s.into();
        if s.is_empty() || s.len() > 65000 {
            return Err(DomainError::Validation(format!(
                "job_payload must be 1..65000 chars"
            )));
        }
        Ok(Self(s))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// `Job::attempts`. A `u8` (0..=255).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobAttempts(pub u8);

impl JobAttempts {
    /// Constructs a new `JobAttempts`.
    #[must_use]
    pub const fn new(v: u8) -> Self {
        Self(v)
    }
    /// Returns the inner u8.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
    /// Returns the inner u32.
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self.0 as u32
    }
}

/// Alias for `Job::available_at` and related timestamps.
pub use educore_core::value_objects::Timestamp as JobAvailableAt;

/// `MaintenanceSetting::title`. 1..191 chars (default: `"We will be back soon!"`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MaintenanceTitle(pub String);

impl MaintenanceTitle {
    /// Default title when none is supplied.
    pub const DEFAULT: &'static str = "We will be back soon!";

    /// Constructs a new `MaintenanceTitle`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 191 {
            return Err(DomainError::Validation(format!(
                "maintenance_title must be 1..191 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the default title.
    #[must_use]
    pub fn default_value() -> Self {
        Self(Self::DEFAULT.to_owned())
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// `MaintenanceSetting::sub_title`. 1..191 chars (default: `"Sorry for the inconvenience..."`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MaintenanceSubTitle(pub String);

impl MaintenanceSubTitle {
    /// Default sub-title when none is supplied.
    pub const DEFAULT: &'static str = "Sorry for the inconvenience...";

    /// Constructs a new `MaintenanceSubTitle`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 191 {
            return Err(DomainError::Validation(format!(
                "maintenance_sub_title must be 1..191 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the default sub-title.
    #[must_use]
    pub fn default_value() -> Self {
        Self(Self::DEFAULT.to_owned())
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// `MaintenanceSetting::image`. A file reference.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MaintenanceImage(pub FileReference);

impl MaintenanceImage {
    /// Constructs a new `MaintenanceImage`.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        // No validation here: callers may pre-validate via FileReference::new.
        Self(FileReference(name.into()))
    }
}

/// `MaintenanceSetting::applicable_for`. Free-form (e.g. `all`, `student,parent`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MaintenanceApplicableFor(pub String);

impl MaintenanceApplicableFor {
    /// Constructs a new `MaintenanceApplicableFor`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 191 {
            return Err(DomainError::Validation(format!(
                "maintenance_applicable_for must be 1..191 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
    /// Returns true if the string equals `"all"`.
    #[must_use]
    pub fn is_all(&self) -> bool {
        self.0 == "all"
    }
}

/// `Sidebar::position`. Sort order (i32).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SidebarPosition(pub i32);

impl SidebarPosition {
    /// Constructs a new `SidebarPosition`.
    pub fn new(v: i32) -> Result<Self> {
        if v < 0 {
            return Err(DomainError::Validation(format!(
                "sidebar_position must be >= 0, got {v}"
            )));
        }
        Ok(Self(v))
    }
    /// Returns the inner integer.
    #[must_use]
    pub const fn get(self) -> i32 {
        self.0
    }
}

/// `Sidebar::section_id`. A section id, consumer-defined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SidebarSectionId(pub i32);

impl SidebarSectionId {
    /// Constructs a new `SidebarSectionId`.
    #[must_use]
    pub const fn new(v: i32) -> Self {
        Self(v)
    }
    /// Returns the inner integer.
    #[must_use]
    pub const fn get(self) -> i32 {
        self.0
    }
}

/// `Sidebar::parent_route`. An optional parent route reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SidebarParentRoute(pub i32);

impl SidebarParentRoute {
    /// Constructs a new `SidebarParentRoute`.
    #[must_use]
    pub const fn new(v: i32) -> Self {
        Self(v)
    }
    /// Returns the inner integer.
    #[must_use]
    pub const fn get(self) -> i32 {
        self.0
    }
    /// Returns true if the parent route is unset (0).
    #[must_use]
    pub const fn is_unset(self) -> bool {
        self.0 == 0
    }
}

/// `Sidebar::is_system_defined`. Boolean wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SidebarIsSystemDefined(pub bool);

impl SidebarIsSystemDefined {
    /// Constructs a new `SidebarIsSystemDefined`.
    #[must_use]
    pub const fn new(v: bool) -> Self {
        Self(v)
    }
    /// Returns true if system-defined.
    #[must_use]
    pub const fn is_system_defined(self) -> bool {
        self.0
    }
}

/// `Sidebar::active_status`. Boolean wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SidebarActiveStatus(pub bool);

impl SidebarActiveStatus {
    /// Constructs a new `SidebarActiveStatus`.
    #[must_use]
    pub const fn new(v: bool) -> Self {
        Self(v)
    }
}

/// `JobReserved::worker_id`. Worker identifier string (1..191 chars).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkerId(pub String);

impl WorkerId {
    /// Constructs a new `WorkerId`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 191 {
            return Err(DomainError::Validation(format!(
                "worker_id must be 1..191 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use educore_core::ids::Identifier;

    #[test]
    fn typed_ids_smoke_test() {
        let school = SchoolId::from_uuid(Uuid::nil());
        let id = BackupId::new(school, Uuid::nil());
        assert_eq!(id.school_id(), school);
        assert_eq!(id.as_uuid(), Uuid::nil());
        let gid = JobId::new(Uuid::nil());
        assert_eq!(gid.as_uuid(), Uuid::nil());
    }

    #[test]
    fn role_and_permission_ids_are_local() {
        let school = SchoolId::from_uuid(Uuid::nil());
        let r = RoleId::new(school, Uuid::nil());
        let p = PermissionId::new(school, Uuid::nil());
        assert_eq!(r.school_id(), school);
        assert_eq!(p.school_id(), school);
    }

    #[test]
    fn ip_address_validator() {
        // IPv4
        assert!(IpAddress::is_valid(""));
        assert!(IpAddress::is_valid("192.0.2.1"));
        assert!(IpAddress::is_valid("0.0.0.0"));
        assert!(IpAddress::is_valid("255.255.255.255"));
        assert!(!IpAddress::is_valid("256.0.0.1"));
        assert!(!IpAddress::is_valid("192.0.2"));
        assert!(!IpAddress::is_valid("192.0.02.1")); // leading zero
        assert!(!IpAddress::is_valid("abc.def.ghi.jkl"));
        // IPv6
        assert!(IpAddress::is_valid("2001:db8::1"));
        assert!(IpAddress::is_valid("::1"));
        assert!(IpAddress::is_valid("fe80::1ff:fe23:4567:890a"));
        assert!(!IpAddress::is_valid("2001:db8:::1"));
        assert!(!IpAddress::is_valid("2001:db8::g"));
    }

    #[test]
    fn version_name_semver() {
        assert!(VersionName::is_semver("8.2.3"));
        assert!(VersionName::is_semver("0.0.0"));
        assert!(VersionName::is_semver("10.20.30"));
        assert!(!VersionName::is_semver("8.2"));
        assert!(!VersionName::is_semver("8.2.3.4"));
        assert!(!VersionName::is_semver(""));
        assert!(!VersionName::is_semver("a.b.c"));
        assert!(!VersionName::is_semver("8.2.x"));
        assert!(!VersionName::is_semver("01.2.3"));
        assert!(!VersionName::is_semver("8.-2.3"));
        assert!(VersionName::new("8.2.3").is_ok());
        assert!(VersionName::new("not-semver").is_err());
    }

    #[test]
    fn sidebar_level_constrains_to_1_3() -> std::result::Result<(), Box<dyn std::error::Error>> {
        assert!(SidebarLevel::new(1)?.is_parent());
        assert!(!SidebarLevel::new(2)?.is_parent());
        assert!(SidebarLevel::new(3).is_ok());
        assert!(SidebarLevel::new(4).is_err());
        assert!(SidebarLevel::new(0).is_err());
        Ok(())
    }

    #[test]
    fn sidebar_ignore_flag_states() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let shown = SidebarIgnoreFlag::new(0)?;
        let hidden = SidebarIgnoreFlag::new(1)?;
        let disabled = SidebarIgnoreFlag::new(2)?;
        assert!(shown.is_shown() && !shown.is_hidden() && !shown.is_disabled());
        assert!(!hidden.is_shown() && hidden.is_hidden() && !hidden.is_disabled());
        assert!(!disabled.is_shown() && !disabled.is_hidden() && disabled.is_disabled());
        assert!(SidebarIgnoreFlag::new(3).is_err());
        Ok(())
    }

    #[test]
    fn enums_display() {
        assert_eq!(BackupFileType::Database.to_string(), "Database");
        assert_eq!(BackupFileType::File.to_string(), "File");
        assert_eq!(BackupFileType::Image.to_string(), "Image");
        assert_eq!(JobStatus::Pending.to_string(), "Pending");
        assert_eq!(LoginOutcome::Success.to_string(), "Success");
        assert_eq!(LoginOutcome::Failure.to_string(), "Failure");
        assert_eq!(
            LoginFailureReason::InvalidCredentials.to_string(),
            "InvalidCredentials"
        );
    }

    #[test]
    fn file_name_validation() {
        assert!(BackupFileName::new("backup-2026-06-08.sql").is_ok());
        assert!(BackupFileName::new("").is_err());
        assert!(BackupFileName::new(&"x".repeat(256)).is_err());
    }

    #[test]
    fn history_url_empty_allowed() {
        assert!(HistoryUrl::new("").is_ok());
        assert!(HistoryUrl::new("https://example.com").is_ok());
        assert!(HistoryUrl::new("not-a-url").is_err());
    }

    #[test]
    fn maintenance_title_defaults() {
        assert_eq!(MaintenanceTitle::DEFAULT, "We will be back soon!");
        assert_eq!(
            MaintenanceSubTitle::DEFAULT,
            "Sorry for the inconvenience..."
        );
        let t = MaintenanceTitle::default_value();
        assert_eq!(t.as_str(), "We will be back soon!");
        assert!(MaintenanceTitle::new("We are down").is_ok());
        assert!(MaintenanceTitle::new("").is_err());
    }

    #[test]
    fn maintenance_applicable_for_is_all() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let all = MaintenanceApplicableFor::new("all")?;
        assert!(all.is_all());
        let student_parent = MaintenanceApplicableFor::new("student,parent")?;
        assert!(!student_parent.is_all());
        Ok(())
    }
}
