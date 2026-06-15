//! # HR child entities
//!
//! Phase 6 ships the bulk-import staging rows as
//! child entities. Other "child entities" from the spec
//! (StaffBankDetail, StaffAddress, etc.) are deferred to
//! a follow-up phase per the Phase 6 open questions list.

#![allow(missing_docs)]
#![allow(unused_imports)]
#![allow(dead_code)]

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};

use crate::value_objects::{StaffAttendanceId, StaffAttendanceImportId, StaffId};

/// A single stage of a bulk staff-attendance import (the
/// payload of a `StaffAttendanceImport` aggregate). The
/// staging row promotes to a `StaffAttendance` aggregate on
/// `PromoteStaffAttendance`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffAttendanceImportRow {
    pub staff_id: StaffId,
    pub source: crate::value_objects::AttendanceSource,
    pub attendance_date: NaiveDate,
    pub attendance_type: crate::value_objects::AttendanceType,
    pub in_time: Option<String>,
    pub out_time: Option<String>,
    pub notes: Option<String>,
}

/// Materialised promotion record: links a `StaffAttendanceImport`
/// staging row to the `StaffAttendance` aggregate it promoted
/// to. Stored as a child entity so consumers can audit
/// "which import row produced which attendance row".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffAttendancePromotion {
    pub school_id: SchoolId,
    pub import_id: StaffAttendanceImportId,
    pub staff_attendance_id: StaffAttendanceId,
    pub promoted_by: UserId,
    pub promoted_at: Timestamp,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl StaffAttendancePromotion {
    pub fn fresh(
        import_id: StaffAttendanceImportId,
        staff_attendance_id: StaffAttendanceId,
        promoted_by: UserId,
        promoted_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: import_id.school_id(),
            import_id,
            staff_attendance_id,
            promoted_by,
            promoted_at,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: promoted_at,
            updated_at: promoted_at,
            created_by: promoted_by,
            updated_by: promoted_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}

/// A staff note (a generic per-staff annotation; not the
/// same as `StaffTimeline`, which is a richer event-log
/// projection).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffNote {
    pub school_id: SchoolId,
    pub staff_id: StaffId,
    pub note_id: Uuid,
    pub note: String,
    pub author: UserId,
    pub created_at: Timestamp,
}
