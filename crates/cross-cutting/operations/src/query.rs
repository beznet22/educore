//! # educore-operations typed query stubs
//!
//! Per `docs/specs/operations/repositories.md`. The operations
//! domain ships 8 typed query builders (one per root aggregate).

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

// =============================================================================
// === BackupQuery section begin (owner: B) ===
// =============================================================================

/// Typed query builder for [`Backup`](crate::aggregate::Backup).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BackupQuery {
    // Fields filled in by the operations domain query layer.
}

impl BackupQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

// === BackupQuery section end ===

// =============================================================================
// === JobQuery section begin (owner: B) ===
// =============================================================================

/// Typed query builder for [`Job`](crate::aggregate::Job).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct JobQuery {}

impl JobQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

// === JobQuery section end ===

// =============================================================================
// === FailedJobQuery section begin (owner: B) ===
// =============================================================================

/// Typed query builder for [`FailedJob`](crate::aggregate::FailedJob).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FailedJobQuery {}

impl FailedJobQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

// === FailedJobQuery section end ===

// =============================================================================
// === SystemVersionQuery section begin (owner: B) ===
// =============================================================================

/// Typed query builder for [`SystemVersion`](crate::aggregate::SystemVersion).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SystemVersionQuery {}

impl SystemVersionQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

// === SystemVersionQuery section end ===

// =============================================================================
// === VersionHistoryQuery section begin (owner: B) ===
// =============================================================================

/// Typed query builder for [`VersionHistory`](crate::aggregate::VersionHistory).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct VersionHistoryQuery {}

impl VersionHistoryQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

// === VersionHistoryQuery section end ===

// =============================================================================
// === UserLogQuery section begin (owner: B) ===
// =============================================================================

/// Typed query builder for [`UserLog`](crate::aggregate::UserLog).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UserLogQuery {}

impl UserLogQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

// === UserLogQuery section end ===

// =============================================================================
// === MaintenanceSettingQuery section begin (owner: B) ===
// =============================================================================

/// Typed query builder for [`MaintenanceSetting`](crate::aggregate::MaintenanceSetting).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MaintenanceSettingQuery {}

impl MaintenanceSettingQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

// === MaintenanceSettingQuery section end ===

// =============================================================================
// === SidebarQuery section begin (owner: B) ===
// =============================================================================

/// Typed query builder for [`Sidebar`](crate::aggregate::Sidebar).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SidebarQuery {}

impl SidebarQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

// === SidebarQuery section end ===
