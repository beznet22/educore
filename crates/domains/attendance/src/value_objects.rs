//! # Attendance value objects
//!
//! The typed ids (every aggregate is keyed by one) and the
//! validated value objects the attendance aggregates depend
//! on. Per `docs/specs/attendance/value-objects.md`:
//!
//! - Every id is `Id { school_id, value }` — a typed wrapper
//!   that carries the school anchor so the type system catches
//!   cross-tenant confusion at compile time.
//! - The status enums (`AttendanceStatus`, `AttendanceType`,
//!   `AttendanceSource`, `ImportStatus`) are closed and
//!   `Copy`.
//! - Foreign-key typed ids (`StudentId`, `ClassId`, `SectionId`,
//!   `SubjectId`, `AcademicYearId`, `StudentRecordId`,
//!   `ExamId`) are **re-exported** from
//!   [`educore_academic`](::educore_academic) and
//!   [`educore_assessment`](::educore_assessment); the
//!   attendance crate owns only the attendance-specific ids
//!   plus a placeholder `StaffId` (the full `Staff`
//!   aggregate lands in the HR domain, Phase 6).
//!
//! Phase 5 ships the 9 prompt-named typed ids (5 aggregate
//! ids + 3 child entity ids + 1 class-attendance id), the
//! 4 closed enums, and the placeholder `StaffId`.

#![allow(missing_docs)] // The new types in Workstream A
                        // (BulkAttendanceImport, StudentAttendance,
                        // SubjectAttendance, StaffAttendance,
                        // ExamAttendance, plus the child entities)
                        // are described by their constructor
                        // signatures; suppressing this lint for
                        // the file is the pragmatic choice.

use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::error::{DomainError, Result};
use educore_core::ids::SchoolId;

// =============================================================================
// Macro: typed attendance id
// =============================================================================

/// Macro to define the per-aggregate typed id wrapper. Every
/// attendance id follows the same shape: a `school_id` anchor
/// plus a local `Uuid`. The wrapper implements
/// [`Clone`], [`Copy`], [`PartialEq`], [`Eq`], [`Hash`], and
/// the `Display` format `"{school_id}/{value}"`.
///
/// The pattern matches
/// `educore-academic::value_objects::academic_typed_id!`,
/// `educore-assessment::value_objects::assessment_typed_id!`,
/// and `educore-rbac::ids::rbac_typed_id!` so the engine's
/// id types stay consistent across crates.
macro_rules! attendance_typed_id {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident;
    ) => {
        $(#[$attr])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
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
// Typed ids: 4 prompt-named aggregate roots
// =============================================================================

attendance_typed_id! {
    /// A typed id for a [`StudentAttendance`](crate::aggregate::StudentAttendance)
    /// row — a daily `(school, student, attendance_date)`
    /// attendance mark.
    pub struct StudentAttendanceId;
}

attendance_typed_id! {
    /// A typed id for a [`SubjectAttendance`](crate::aggregate::SubjectAttendance)
    /// row — a per-period (per-subject)
    /// `(school, student, subject, attendance_date)` mark.
    pub struct SubjectAttendanceId;
}

attendance_typed_id! {
    /// A typed id for a [`StaffAttendance`](crate::aggregate::StaffAttendance)
    /// row — a daily `(school, staff, attendance_date)` mark.
    pub struct StaffAttendanceId;
}

attendance_typed_id! {
    /// A typed id for an [`ExamAttendance`](crate::aggregate::ExamAttendance)
    /// row — a per-exam `(school, exam, student, exam_date)`
    /// mark.
    pub struct ExamAttendanceId;
}

// =============================================================================
// Typed ids: bulk import job + child entity ids
// =============================================================================

attendance_typed_id! {
    /// A typed id for a [`BulkAttendanceImport`](crate::aggregate::BulkAttendanceImport)
    /// job — the staging row that captures a CSV / biometric /
    /// API bulk-attendance import batch.
    pub struct BulkAttendanceImportId;
}

attendance_typed_id! {
    /// A typed id for a [`StudentAttendanceImport`](crate::entities::StudentAttendanceImport)
    /// child row — one validated row in a
    /// `BulkAttendanceImport`.
    pub struct StudentAttendanceImportId;
}

attendance_typed_id! {
    /// A typed id for a [`StaffAttendanceImport`](crate::entities::StaffAttendanceImport)
    /// child row — one validated row in a staff-side
    /// `BulkAttendanceImport`.
    pub struct StaffAttendanceImportId;
}

attendance_typed_id! {
    /// A typed id for a `ClassAttendance` projection row — a
    /// per-(class, section, attendance_date) roll-up the engine
    /// materialises for reports and notifications.
    pub struct ClassAttendanceId;
}

// =============================================================================
// Typed ids: bulk import header (Phase 5 placeholder)
// =============================================================================

attendance_typed_id! {
    /// A typed id for a `BulkAttendanceHeader` row (the
    /// upstream import file's metadata). Placeholder until
    /// the bulk-import header aggregate lands in a follow-up
    /// phase. Reserved here so the bulk-import command can
    /// declare its foreign-key fields against a stable type.
    pub struct AttendanceBulkId;
}

// =============================================================================
// Placeholder StaffId (the full `Staff` aggregate lands in
// the HR domain, Phase 6). Mirrors the assessment crate's
// placeholder pattern (see
// `educore-assessment::value_objects::StaffId`).
// =============================================================================

attendance_typed_id! {
    /// A typed id for a `Staff` aggregate (an employee /
    /// teacher). Placeholder until the HR domain ships its
    /// `Staff` aggregate in Phase 6. The full definition
    /// will be wired in via a re-export from `educore-hr`
    /// once it lands; until then the attendance crate
    /// declares its own placeholder so foreign-key fields
    /// in [`StaffAttendance`](crate::aggregate::StaffAttendance)
    /// and [`StaffAttendanceImport`](crate::entities::StaffAttendanceImport)
    /// are typed.
    pub struct StaffId;
}

// =============================================================================
// Closed enums
// =============================================================================

/// The status of a single attendance row, as it appears in
/// reports and the audit log. Per
/// `docs/specs/attendance/value-objects.md` § Attendance
/// Status Enums.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttendanceStatus {
    /// The student was present for the entire school day.
    #[default]
    Present,
    /// The student was absent (single-character code `"A"`).
    Absent,
    /// The student arrived after the bell (code `"L"`).
    Late,
    /// The student was present for half the day (morning or
    /// afternoon) and absent the other half (code `"F"` —
    /// leave/half-day).
    HalfDay,
    /// The school was closed (a holiday, weekend, or
    /// declared leave) — code `"H"`.
    Holiday,
    /// The student was on an authorised leave (sick,
    /// family, medical, etc.). Distinct from
    /// [`HalfDay`](Self::HalfDay) in the audit trail and
    /// in the notification template; not on the
    /// single-character code form.
    OnLeave,
}

impl AttendanceStatus {
    /// Returns the canonical snake_case wire form. The
    /// single-character short codes (`"P"`, `"A"`, `"L"`,
    /// `"F"`, `"H"`) are emitted by
    /// [`AttendanceType::as_str`], not here —
    /// [`AttendanceStatus`] is the human-readable report
    /// enum and uses long words.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Present => "present",
            Self::Absent => "absent",
            Self::Late => "late",
            Self::HalfDay => "half_day",
            Self::Holiday => "holiday",
            Self::OnLeave => "on_leave",
        }
    }

    /// Returns `true` if the status counts as an absence for
    /// reporting (Absent, HalfDay, OnLeave). Holiday is
    /// excluded because the school itself is closed.
    #[must_use]
    pub const fn is_absence(self) -> bool {
        matches!(self, Self::Absent | Self::HalfDay | Self::OnLeave)
    }
}

impl fmt::Display for AttendanceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The single-character attendance code that maps to the
/// `attendance_type` column in the database. Per
/// `docs/specs/attendance/value-objects.md` § Attendance
/// Status Enums.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttendanceType {
    /// Present (code `"P"`).
    #[default]
    Present,
    /// Absent (code `"A"`).
    Absent,
    /// Late (code `"L"`).
    Late,
    /// Half-day / leave (code `"F"`).
    HalfDay,
    /// Holiday (code `"H"`).
    Holiday,
}

impl AttendanceType {
    /// Returns the canonical single-character wire form
    /// (`"P"`, `"A"`, `"L"`, `"F"`, `"H"`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Present => "P",
            Self::Absent => "A",
            Self::Late => "L",
            Self::HalfDay => "F",
            Self::Holiday => "H",
        }
    }

    /// Returns `true` if the type counts as an absence for
    /// reporting. Mirrors [`AttendanceStatus::is_absence`].
    #[must_use]
    pub const fn is_absent(self) -> bool {
        matches!(self, Self::Absent)
    }

    /// Parses a wire character into an [`AttendanceType`].
    /// Returns `Validation` on unknown values.
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "P" => Ok(Self::Present),
            "A" => Ok(Self::Absent),
            "L" => Ok(Self::Late),
            "F" => Ok(Self::HalfDay),
            "H" => Ok(Self::Holiday),
            other => Err(DomainError::validation(format!(
                "unknown attendance_type: {other:?}"
            ))),
        }
    }
}

impl fmt::Display for AttendanceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The channel that produced the attendance mark. Per
/// `docs/specs/attendance/value-objects.md` § Attendance
/// Status Enums.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttendanceSource {
    /// A teacher typed the mark in the web UI (default).
    #[default]
    Manual,
    /// A biometric device (fingerprint / RFID / face) recorded
    /// the mark.
    Biometric,
    /// A bulk-import job (CSV / vendor feed).
    BulkImport,
    /// A direct API call (mobile app / third-party integration).
    Api,
}

impl AttendanceSource {
    /// Returns the canonical snake_case wire form.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Biometric => "biometric",
            Self::BulkImport => "bulk_import",
            Self::Api => "api",
        }
    }

    /// Parses a wire string into an [`AttendanceSource`].
    /// Returns `Validation` on unknown values.
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "manual" => Ok(Self::Manual),
            "biometric" => Ok(Self::Biometric),
            "bulk_import" => Ok(Self::BulkImport),
            "api" => Ok(Self::Api),
            other => Err(DomainError::validation(format!(
                "unknown attendance_source: {other:?}"
            ))),
        }
    }
}

impl fmt::Display for AttendanceSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The status of a [`BulkAttendanceImport`](crate::aggregate::BulkAttendanceImport)
/// job. Per `docs/specs/attendance/value-objects.md` §
/// Attendance Status Enums.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImportStatus {
    /// The import has been received and is awaiting validation
    /// (default).
    #[default]
    Pending,
    /// The import has been validated; no business-rule
    /// violations were found.
    Validated,
    /// The import's staging rows have been promoted into the
    /// live `StudentAttendance` / `StaffAttendance` tables.
    Committed,
    /// The import failed validation (e.g. duplicate
    /// `(school, student, date)` keys) and no rows were
    /// committed.
    Failed,
    /// The import was cancelled by the operator before
    /// commit. No rows were committed.
    Cancelled,
}

impl ImportStatus {
    /// Returns the canonical snake_case wire form.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Validated => "validated",
            Self::Committed => "committed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    /// Returns `true` if the import is in a terminal state
    /// (Committed, Failed, or Cancelled).
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Committed | Self::Failed | Self::Cancelled)
    }
}

impl fmt::Display for ImportStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// =============================================================================
// Validators
// =============================================================================

/// Validates the optional `notes` field on a mark. The limit
/// is 500 characters per `docs/specs/attendance/value-objects.md`
/// § Names and Codes.
pub fn validate_notes(s: &str) -> Result<()> {
    let n = s.chars().count();
    if n > 500 {
        return Err(DomainError::validation(format!(
            "notes must be at most 500 chars, got {n}"
        )));
    }
    Ok(())
}

/// Validates the optional `source` (AttendanceSource) string
/// for the bulk-import path. The limit is 100 characters.
pub fn validate_source(s: &str) -> Result<()> {
    let n = s.chars().count();
    if n > 100 {
        return Err(DomainError::validation(format!(
            "source must be at most 100 chars, got {n}"
        )));
    }
    Ok(())
}

// =============================================================================
// Re-exports of the `educore-academic` and `educore-assessment`
// types the attendance crate consumes (avoids redefinition;
// the academic and assessment crates own the canonical
// definitions).
// =============================================================================

pub use educore_academic::{ClassId, SectionId, StudentId, StudentRecordId, SubjectId};

pub use educore_academic::value_objects::AcademicYearId;

pub use educore_assessment::ExamId;

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;

    #[test]
    fn attendance_status_default_is_present() {
        assert_eq!(AttendanceStatus::default(), AttendanceStatus::Present);
    }

    #[test]
    fn attendance_status_as_str_is_snake_case() {
        assert_eq!(AttendanceStatus::Present.as_str(), "present");
        assert_eq!(AttendanceStatus::Absent.as_str(), "absent");
        assert_eq!(AttendanceStatus::Late.as_str(), "late");
        assert_eq!(AttendanceStatus::HalfDay.as_str(), "half_day");
        assert_eq!(AttendanceStatus::Holiday.as_str(), "holiday");
        assert_eq!(AttendanceStatus::OnLeave.as_str(), "on_leave");
    }

    #[test]
    fn attendance_status_is_absence_excludes_holiday() {
        assert!(AttendanceStatus::Absent.is_absence());
        assert!(AttendanceStatus::HalfDay.is_absence());
        assert!(AttendanceStatus::OnLeave.is_absence());
        assert!(!AttendanceStatus::Present.is_absence());
        assert!(!AttendanceStatus::Late.is_absence());
        assert!(!AttendanceStatus::Holiday.is_absence());
    }

    #[test]
    fn attendance_type_default_is_present() {
        assert_eq!(AttendanceType::default(), AttendanceType::Present);
    }

    #[test]
    fn attendance_type_as_str_is_single_char() {
        assert_eq!(AttendanceType::Present.as_str(), "P");
        assert_eq!(AttendanceType::Absent.as_str(), "A");
        assert_eq!(AttendanceType::Late.as_str(), "L");
        assert_eq!(AttendanceType::HalfDay.as_str(), "F");
        assert_eq!(AttendanceType::Holiday.as_str(), "H");
    }

    #[test]
    fn attendance_type_is_absent_only_for_absent() {
        assert!(AttendanceType::Absent.is_absent());
        assert!(!AttendanceType::Present.is_absent());
        assert!(!AttendanceType::Late.is_absent());
        assert!(!AttendanceType::HalfDay.is_absent());
        assert!(!AttendanceType::Holiday.is_absent());
    }

    #[test]
    fn attendance_type_parse_round_trip() {
        for t in [
            AttendanceType::Present,
            AttendanceType::Absent,
            AttendanceType::Late,
            AttendanceType::HalfDay,
            AttendanceType::Holiday,
        ] {
            assert_eq!(AttendanceType::parse(t.as_str()).unwrap(), t);
        }
    }

    #[test]
    fn attendance_type_parse_rejects_unknown() {
        let err = AttendanceType::parse("X").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn attendance_source_default_is_manual() {
        assert_eq!(AttendanceSource::default(), AttendanceSource::Manual);
    }

    #[test]
    fn attendance_source_as_str_is_snake_case() {
        assert_eq!(AttendanceSource::Manual.as_str(), "manual");
        assert_eq!(AttendanceSource::Biometric.as_str(), "biometric");
        assert_eq!(AttendanceSource::BulkImport.as_str(), "bulk_import");
        assert_eq!(AttendanceSource::Api.as_str(), "api");
    }

    #[test]
    fn attendance_source_parse_round_trip() {
        for s in [
            AttendanceSource::Manual,
            AttendanceSource::Biometric,
            AttendanceSource::BulkImport,
            AttendanceSource::Api,
        ] {
            assert_eq!(AttendanceSource::parse(s.as_str()).unwrap(), s);
        }
    }

    #[test]
    fn attendance_source_parse_rejects_unknown() {
        let err = AttendanceSource::parse("cli").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn import_status_default_is_pending() {
        assert_eq!(ImportStatus::default(), ImportStatus::Pending);
    }

    #[test]
    fn import_status_as_str_is_snake_case() {
        assert_eq!(ImportStatus::Pending.as_str(), "pending");
        assert_eq!(ImportStatus::Validated.as_str(), "validated");
        assert_eq!(ImportStatus::Committed.as_str(), "committed");
        assert_eq!(ImportStatus::Failed.as_str(), "failed");
        assert_eq!(ImportStatus::Cancelled.as_str(), "cancelled");
    }

    #[test]
    fn import_status_is_terminal_covers_committed_failed_cancelled() {
        assert!(!ImportStatus::Pending.is_terminal());
        assert!(!ImportStatus::Validated.is_terminal());
        assert!(ImportStatus::Committed.is_terminal());
        assert!(ImportStatus::Failed.is_terminal());
        assert!(ImportStatus::Cancelled.is_terminal());
    }

    #[test]
    fn validate_notes_accepts_short() {
        validate_notes("late arrival due to bus delay").expect("ok");
    }

    #[test]
    fn validate_notes_accepts_max_len() {
        let s: String = "a".repeat(500);
        validate_notes(&s).expect("ok");
    }

    #[test]
    fn validate_notes_rejects_too_long() {
        let s: String = "a".repeat(501);
        let err = validate_notes(&s).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn validate_source_accepts_short() {
        validate_source("manual").expect("ok");
    }

    #[test]
    fn validate_source_rejects_too_long() {
        let s: String = "a".repeat(101);
        let err = validate_source(&s).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn typed_ids_construct_and_display() {
        let school = SchoolId(Uuid::now_v7());
        let value = Uuid::now_v7();
        let id = StudentAttendanceId::new(school, value);
        assert_eq!(id.school_id(), school);
        assert_eq!(id.as_uuid(), value);
        assert_eq!(id.to_string(), format!("{school}/{value}"));
    }

    #[test]
    fn typed_ids_carry_their_school_anchor() {
        let school_a = SchoolId(Uuid::now_v7());
        let school_b = SchoolId(Uuid::now_v7());
        let value = Uuid::now_v7();
        let id = StudentAttendanceId::new(school_a, value);
        assert_eq!(id.school_id(), school_a);
        assert_ne!(id.school_id(), school_b);
    }

    #[test]
    fn typed_ids_for_every_aggregate() {
        let school = SchoolId(Uuid::now_v7());
        let v = Uuid::now_v7();
        // Construct one of every aggregate id type; the constructor
        // signature is identical for all 9, but this ensures the
        // macro emits the expected shape for each.
        let _sa = StudentAttendanceId::new(school, v);
        let _sb = SubjectAttendanceId::new(school, v);
        let _sc = StaffAttendanceId::new(school, v);
        let _se = ExamAttendanceId::new(school, v);
        let _ba = BulkAttendanceImportId::new(school, v);
        let _si = StudentAttendanceImportId::new(school, v);
        let _wi = StaffAttendanceImportId::new(school, v);
        let _ca = ClassAttendanceId::new(school, v);
        let _ah = AttendanceBulkId::new(school, v);
        let _st = StaffId::new(school, v);
    }

    #[test]
    fn all_three_re_exports_resolve() {
        // Compile-time check: the three foreign-key id types
        // re-exported from `educore-academic` and
        // `educore-assessment` are usable from the attendance
        // crate's value_objects module.
        let s = SchoolId(Uuid::now_v7());
        let v = Uuid::now_v7();
        let _c: ClassId = ClassId::new(s, v);
        let _sec: SectionId = SectionId::new(s, v);
        let _stu: StudentId = StudentId::new(s, v);
        let _sr: StudentRecordId = StudentRecordId::new(s, v);
        let _sub: SubjectId = SubjectId::new(s, v);
        let _ay: AcademicYearId = AcademicYearId::new(s, v);
        let _ex: ExamId = ExamId::new(s, v);
    }
}
