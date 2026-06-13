//! The bulk-insert row type for `StudentAttendance`.
//!
//! This row type is the wire format for the `bulk_insert_student_attendances`
//! storage port method. It carries the same fields as the
//! `StudentAttendance` aggregate but is decoupled from the aggregate type
//! so the storage port doesn't depend on the attendance domain crate.
//!
//! The mapping from `StudentAttendance` (in `educore-attendance`) to
//! `StudentAttendanceRow` (in `educore-storage`) is a constructor on
//! either side; the storage adapter's `bulk_insert` method only
//! sees the row type.

use chrono::NaiveDate;
use educore_core::ids::{CorrelationId, EventId, Identifier, SchoolId, UserId};
use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A `StudentAttendance` row, flattened to its storage columns. Decoupled
/// from the `StudentAttendance` aggregate so the storage port doesn't
/// import the attendance domain crate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StudentAttendanceRow {
    /// The school the row belongs to. Every adapter rejects rows
    /// whose `school_id` does not match the active
    /// `TenantContext::school_id` (or the transaction's scoped
    /// school).
    pub school_id: SchoolId,
    /// The row's primary key. UUIDv7.
    pub id: Uuid,
    /// The student this attendance row records. References
    /// `students.id` in the academic domain.
    pub student_id: Uuid,
    /// The per-year enrollment handle for the student on the
    /// attendance date. References `student_records.id`.
    pub student_record_id: Uuid,
    /// The class the student was enrolled in on the attendance
    /// date. References `classes.id`.
    pub class_id: Uuid,
    /// The section the student was enrolled in on the attendance
    /// date. References `sections.id`.
    pub section_id: Uuid,
    /// The calendar day the row records attendance for. Stored
    /// as `TEXT` (ISO 8601 `YYYY-MM-DD`) in the database.
    pub attendance_date: NaiveDate,
    /// The single-character attendance code. One of
    /// `"P"` (Present), `"A"` (Absent), `"L"` (Late),
    /// `"F"` (Leave/Excused), `"H"` (Holiday). The engine maps
    /// these to the typed `AttendanceStatus` enum on read.
    pub attendance_type: String,
    /// The wall-clock time the student signed in, if recorded.
    /// `None` when the school does not capture in-time. Stored
    /// as `TEXT` in `HH:MM:SS` form.
    pub in_time: Option<String>,
    /// The wall-clock time the student signed out, if recorded.
    /// `None` when the school does not capture out-time. Stored
    /// as `TEXT` in `HH:MM:SS` form.
    pub out_time: Option<String>,
    /// Free-form notes attached to the row (e.g. absence
    /// reason, late arrival explanation). `None` for unmarked
    /// rows.
    pub notes: Option<String>,
    /// `true` when the student was absent on the attendance
    /// date. Denormalised from `attendance_type = "A"` for
    /// index-only scans; the engine keeps the two in sync.
    pub is_absent: bool,
    /// The user (or `SYSTEM`) who recorded the attendance.
    pub marked_by: UserId,
    /// The instant the attendance was recorded. Used to
    /// reconstruct the "marked late" replay window.
    pub marked_at: Timestamp,
    /// The channel that produced the mark. One of
    /// `"manual"` (teacher typed it), `"biometric"` (fingerprint /
    /// RFID), `"bulk_import"` (CSV upload),
    /// `"api"` (mobile app / integration).
    pub marked_from: String,
    /// The aggregate's optimistic-concurrency version. Starts at
    /// `1` on insert and increments on every update.
    pub version: Version,
    /// The aggregate's content-hash etag. Used for conflict
    /// resolution on concurrent updates.
    pub etag: Etag,
    /// The instant the row was first created.
    pub created_at: Timestamp,
    /// The instant the row was last updated.
    pub updated_at: Timestamp,
    /// The user (or `SYSTEM`) that created the row.
    pub created_by: UserId,
    /// The user (or `SYSTEM`) that last updated the row.
    pub updated_by: UserId,
    /// Soft-delete flag. `Retired` rows are hidden from ordinary
    /// queries.
    pub active_status: ActiveStatus,
    /// The id of the event that most recently mutated this row,
    /// if any. Lets auditors correlate rows to event log rows.
    pub last_event_id: Option<EventId>,
    /// The correlation id of the request that originated the
    /// row. Propagated to every event the row emits.
    pub correlation_id: CorrelationId,
}

impl StudentAttendanceRow {
    /// The number of storage columns in the row (used by the
    /// multi-row INSERT path to size the placeholder expansion and
    /// by the SQLite batcher to compute the per-batch row cap).
    pub const COLUMN_COUNT: usize = 24;

    /// Returns the row's `school_id` as a 16-byte big-endian
    /// `Vec<u8>`. Storage adapters bind UUID columns as raw bytes
    /// (`BYTEA` / `VARBINARY` / `BLOB`) per the
    /// `attendance_student_attendances` DDL, so this is the
    /// canonical wire form. `Vec<u8>` (not `bytes::Bytes`) is
    /// used because sqlx does not implement `Type<DB>` for
    /// `bytes::Bytes`.
    #[must_use]
    pub fn school_id_bytes(&self) -> Vec<u8> {
        self.school_id.as_uuid().as_bytes().to_vec()
    }

    /// Returns the row's `id` as a 16-byte big-endian `Vec<u8>`.
    #[must_use]
    pub fn id_bytes(&self) -> Vec<u8> {
        self.id.as_bytes().to_vec()
    }

    /// Returns the row's `student_id` as a 16-byte big-endian
    /// `Vec<u8>`.
    #[must_use]
    pub fn student_id_bytes(&self) -> Vec<u8> {
        self.student_id.as_bytes().to_vec()
    }

    /// Returns the row's `student_record_id` as a 16-byte
    /// big-endian `Vec<u8>`.
    #[must_use]
    pub fn student_record_id_bytes(&self) -> Vec<u8> {
        self.student_record_id.as_bytes().to_vec()
    }

    /// Returns the row's `class_id` as a 16-byte big-endian
    /// `Vec<u8>`.
    #[must_use]
    pub fn class_id_bytes(&self) -> Vec<u8> {
        self.class_id.as_bytes().to_vec()
    }

    /// Returns the row's `section_id` as a 16-byte big-endian
    /// `Vec<u8>`.
    #[must_use]
    pub fn section_id_bytes(&self) -> Vec<u8> {
        self.section_id.as_bytes().to_vec()
    }

    /// Returns the row's `marked_by` as a 16-byte big-endian
    /// `Vec<u8>`.
    #[must_use]
    pub fn marked_by_bytes(&self) -> Vec<u8> {
        self.marked_by.as_uuid().as_bytes().to_vec()
    }

    /// Returns the row's `created_by` as a 16-byte big-endian
    /// `Vec<u8>`.
    #[must_use]
    pub fn created_by_bytes(&self) -> Vec<u8> {
        self.created_by.as_uuid().as_bytes().to_vec()
    }

    /// Returns the row's `updated_by` as a 16-byte big-endian
    /// `Vec<u8>`.
    #[must_use]
    pub fn updated_by_bytes(&self) -> Vec<u8> {
        self.updated_by.as_uuid().as_bytes().to_vec()
    }

    /// Returns the row's `correlation_id` as a 16-byte big-endian
    /// `Vec<u8>`.
    #[must_use]
    pub fn correlation_id_bytes(&self) -> Vec<u8> {
        self.correlation_id.as_uuid().as_bytes().to_vec()
    }

    /// Returns the row's `last_event_id` as a 16-byte big-endian
    /// `Vec<u8>`, or `None` if the row was inserted without an
    /// originating event (e.g. a backfill).
    #[must_use]
    pub fn last_event_id_bytes(&self) -> Option<Vec<u8>> {
        self.last_event_id.map(|e| e.as_uuid().as_bytes().to_vec())
    }

    /// Returns the row's `attendance_date` in ISO 8601 form
    /// (`YYYY-MM-DD`), the canonical wire form for the
    /// `attendance_date` TEXT column on all three dialects.
    #[must_use]
    pub fn attendance_date_string(&self) -> String {
        self.attendance_date.to_string()
    }

    /// Returns the `marked_at` timestamp in RFC 3339 UTC form.
    #[must_use]
    pub fn marked_at_string(&self) -> String {
        self.marked_at.to_rfc3339()
    }

    /// Returns the `created_at` timestamp in RFC 3339 UTC form.
    #[must_use]
    pub fn created_at_string(&self) -> String {
        self.created_at.to_rfc3339()
    }

    /// Returns the `updated_at` timestamp in RFC 3339 UTC form.
    #[must_use]
    pub fn updated_at_string(&self) -> String {
        self.updated_at.to_rfc3339()
    }

    /// Returns the version's raw counter as an `i64` for binding
    /// to the INTEGER column.
    #[must_use]
    pub fn version_value(&self) -> i64 {
        self.version.get()
    }

    /// Returns the `active_status` byte (`1` = Active, `0` =
    /// Retired) for binding to the INTEGER column.
    #[must_use]
    pub fn active_status_byte(&self) -> i64 {
        i64::from(self.active_status.to_byte())
    }

    /// Returns the `is_absent` flag as an `i64` (0 or 1) for
    /// binding to the INTEGER column.
    #[must_use]
    pub fn is_absent_value(&self) -> i64 {
        i64::from(self.is_absent)
    }
}
